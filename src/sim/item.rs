use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::combat::{Combat, HitPoints};
use super::enemy::Hurt;
use super::events::{SimEvent, SimEvents};
use super::ground::{self, grounded};
use super::health::Health;
use super::hop::Hop;
use super::intent::UseIntent;
use super::jump::Jump;
use super::player::Player;
use super::roll::Roll;
use super::ship::Ship;
use super::swim::Swim;
use super::talk::Dialogue;
use super::walk::{swept, within};

pub const FIRST_BOMB: u8 = 0x0B;
pub const LAST_BOMB: u8 = 0x0E;
pub const FIRST_ARROW: u8 = 0x10;
pub const LAST_ARROW: u8 = 0x12;
pub const BOOMERANG: u8 = 0x2D;

const BOMB_COUNTS: [u8; 4] = [5, 10, 20, 30];
const ARROW_COUNTS: [u8; 3] = [10, 20, 30];
const BOMB_CAP: u8 = 30;
const ARROW_CAP: u8 = 30;

const BOMB_FUSE: u32 = 100;
const BOMB_RADIUS: f32 = 200.0;
const BOMB_POWER: u8 = 4;
const BOMB_THROW: f32 = 50.0;
const BOMB_LIFT: f32 = 40.0;
const BOMB_GRAVITY: f32 = -2.5;
const ARROW_SPEED: f32 = 200.0;
const ARROW_GRAVITY: f32 = -2.0;
const ARROW_FALL: f32 = -100.0;
const ARROW_POWER: i32 = 2;
const ARROW_REACH: f32 = 30.0;
const ARROW_LIFE: u32 = 120;
const BOOM_SPEED: f32 = 60.0;
const BOOM_LIFETIME: u32 = 300;
const BOOM_LOCKS: usize = 2;
const BOOM_CATCH: f32 = 120.0;
const BOOM_RETURN_TICKS: u32 = 60;
const BOOM_REACH: f32 = 30.0;
const BOOM_POWER: i32 = 1;
const THROW_OFFSET: f32 = 20.0;
const BOMB_FLINCH: u32 = 10;
const ARROW_FLINCH: u32 = 10;
const BOOM_FLINCH: u32 = 6;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InFlight;

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Held {
    pub item: u8,
    pub count: u8,
    pub pressed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Bomb,
    Arrow,
    Boomerang,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bomb {
    pub pos: Vec3,
    pub fuse: u32,
    pub vel: Vec3,
    pub rest: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Arrow {
    pub pos: Vec3,
    pub vel: Vec3,
    pub life: u32,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Boomerang {
    pub pos: Vec3,
    pub vel: Vec3,
    pub returning: bool,
    pub out: u32,
    pub life: u32,
    pub struck: Vec<Entity>,
}

impl Held {
    pub fn equip(&mut self, item: u8) {
        let (slot, count) = match item {
            FIRST_BOMB..=LAST_BOMB => (FIRST_BOMB, BOMB_COUNTS[(item - FIRST_BOMB) as usize]),
            FIRST_ARROW..=LAST_ARROW => (FIRST_ARROW, ARROW_COUNTS[(item - FIRST_ARROW) as usize]),
            BOOMERANG => (BOOMERANG, 1),
            _ => return,
        };
        if self.item == slot {
            let cap = match slot {
                BOOMERANG => 1,
                FIRST_BOMB => BOMB_CAP,
                _ => ARROW_CAP,
            };
            self.count = self.count.saturating_add(count).min(cap);
            return;
        }
        self.item = slot;
        self.count = count;
    }

    pub fn kind(&self) -> Option<Kind> {
        match self.item {
            FIRST_BOMB..=LAST_BOMB => Some(Kind::Bomb),
            FIRST_ARROW..=LAST_ARROW => Some(Kind::Arrow),
            BOOMERANG => Some(Kind::Boomerang),
            _ => None,
        }
    }
}

#[derive(SystemParam)]
pub struct Gates<'w, 's> {
    swim: Res<'w, Swim>,
    roll: Res<'w, Roll>,
    hop: Res<'w, Hop>,
    jump: Res<'w, Jump>,
    combat: Res<'w, Combat>,
    dialogue: Res<'w, Dialogue>,
    ships: Query<'w, 's, &'static Ship>,
}

impl Gates<'_, '_> {
    fn busy(&self) -> bool {
        self.swim.active
            || self.roll.active
            || self.hop.active
            || self.jump.active
            || self.combat.active
            || self.dialogue.open
            || self.ships.iter().any(|ship| ship.riding)
    }
}

#[derive(SystemParam)]
pub struct Shots<'w, 's> {
    bombs: Query<'w, 's, (Entity, &'static mut Bomb), Without<Arrow>>,
    arrows: Query<'w, 's, (Entity, &'static mut Arrow), Without<Bomb>>,
    spin: Query<'w, 's, (Entity, &'static mut Boomerang), Without<Bomb>>,
}

impl Shots<'_, '_> {
    fn spinning(&self) -> bool {
        self.spin.iter().next().is_some()
    }

    fn recall(&mut self) {
        for (_, mut boom) in &mut self.spin {
            boom.returning = true;
        }
    }
}

fn launch(player: &Player, held: &mut Held, commands: &mut Commands) {
    let Some(kind) = held.kind() else {
        return;
    };
    if held.count == 0 {
        return;
    }
    let (sin, cos) = player.yaw.sin_cos();
    let pos = player.pos + Vec3::new(THROW_OFFSET * sin, 0.0, THROW_OFFSET * cos);
    match kind {
        Kind::Bomb => {
            commands.spawn((
                Bomb {
                    pos,
                    fuse: BOMB_FUSE,
                    vel: Vec3::new(BOMB_THROW * sin, BOMB_LIFT, BOMB_THROW * cos),
                    rest: false,
                },
                InFlight,
            ));
        }
        Kind::Arrow => {
            commands.spawn((
                Arrow {
                    pos,
                    vel: Vec3::new(ARROW_SPEED * sin, 0.0, ARROW_SPEED * cos),
                    life: ARROW_LIFE,
                },
                InFlight,
            ));
        }
        Kind::Boomerang => {
            commands.spawn((
                Boomerang {
                    pos,
                    vel: Vec3::new(BOOM_SPEED * sin, 0.0, BOOM_SPEED * cos),
                    returning: false,
                    out: 0,
                    life: BOOM_LIFETIME,
                    struck: Vec::new(),
                },
                InFlight,
            ));
        }
    }
    held.count -= 1;
}

pub fn control(
    collision: Option<Res<Collision>>,
    intent: Res<UseIntent>,
    mut health: ResMut<Health>,
    mut held: ResMut<Held>,
    players: Query<&Player>,
    gates: Gates,
    mut shots: Shots,
    mut foes: Query<(Entity, &Actor, &mut HitPoints, Option<&mut Hurt>), Without<Ship>>,
    mut events: ResMut<SimEvents>,
    mut commands: Commands,
) {
    let pressed = intent.0 && !held.pressed;
    held.pressed = intent.0;
    let solid = collision.as_deref();
    let ready = health.units > 0
        && !gates.busy()
        && players
            .iter()
            .next()
            .is_some_and(|player| grounded(solid, player.pos));

    if pressed {
        if shots.spinning() {
            shots.recall();
        } else if ready {
            for player in &players {
                launch(player, &mut held, &mut commands);
            }
        }
    }

    let mut dead: Vec<Entity> = Vec::new();
    let mut chain: Vec<Entity> = Vec::new();
    let mut burst: Vec<Entity> = Vec::new();
    for (entity, mut bomb) in &mut shots.bombs {
        if !bomb.rest {
            bomb.vel.y += BOMB_GRAVITY;
            let step = bomb.vel;
            bomb.pos += step;
            if let Some(floor) = ground::surface(solid, bomb.pos)
                && bomb.pos.y <= floor
            {
                bomb.pos.y = floor;
                bomb.vel = Vec3::ZERO;
                bomb.rest = true;
            }
        }
        bomb.fuse = bomb.fuse.saturating_sub(1);
        if bomb.fuse == 0 {
            burst.push(entity);
        }
    }
    let owner = players.iter().next().map(|player| player.pos);
    for &entity in &burst {
        let Ok((_, bomb)) = shots.bombs.get(entity) else {
            continue;
        };
        let blast = bomb.pos;
        for (other, neighbour) in shots.bombs.iter() {
            if other != entity
                && !burst.contains(&other)
                && within(blast, neighbour.pos, BOMB_RADIUS)
            {
                chain.push(other);
            }
        }
        for (foe_entity, foe, mut hp, mut hurt) in &mut foes {
            if within(blast, foe.pos, BOMB_RADIUS) {
                hp.0 -= BOMB_POWER as i32;
                if let Some(hurt) = hurt.as_deref_mut() {
                    hurt.0 = BOMB_FLINCH;
                }
                if hp.0 <= 0 {
                    dead.push(foe_entity);
                }
            }
        }
        if owner.is_some_and(|pos| within(blast, pos, BOMB_RADIUS)) {
            health.damage(BOMB_POWER);
        }
        events.list.push(SimEvent::Explode);
        let _ = commands.entity(entity).despawn();
    }
    for entity in chain {
        if let Ok((_, mut bomb)) = shots.bombs.get_mut(entity) {
            bomb.fuse = 0;
        }
    }
    for entity in dead {
        let _ = commands.entity(entity).despawn();
    }

    let mut spent: Vec<Entity> = Vec::new();
    for (entity, mut arrow) in &mut shots.arrows {
        let from = arrow.pos;
        arrow.vel.y = (arrow.vel.y + ARROW_GRAVITY).max(ARROW_FALL);
        let step = arrow.vel;
        arrow.pos += step;
        let mut hit = false;
        for (_, foe, mut hp, mut hurt) in &mut foes {
            if swept(from, foe.pos, ARROW_REACH) {
                hp.0 -= ARROW_POWER;
                if let Some(hurt) = hurt.as_deref_mut() {
                    hurt.0 = ARROW_FLINCH;
                }
                hit = true;
                break;
            }
        }
        arrow.life = arrow.life.saturating_sub(1);
        if hit || arrow.life == 0 || grounded(solid, arrow.pos) {
            spent.push(entity);
        }
    }

    let home = players.iter().next().map(|player| player.pos);
    for (entity, mut boom) in &mut shots.spin {
        let from = boom.pos;
        boom.out = boom.out.saturating_add(1);
        if boom.returning {
            let target = home.unwrap_or(boom.pos);
            let delta = target - boom.pos;
            if delta.length() > BOOM_CATCH {
                boom.vel = delta.normalize() * BOOM_SPEED;
            }
        }
        let step = boom.vel;
        boom.pos += step;
        if !boom.returning {
            for (foe_entity, foe, mut hp, mut hurt) in &mut foes {
                if !boom.struck.contains(&foe_entity) && swept(from, foe.pos, BOOM_REACH) {
                    hp.0 -= BOOM_POWER;
                    if let Some(hurt) = hurt.as_deref_mut() {
                        hurt.0 = BOOM_FLINCH;
                    }
                    boom.struck.push(foe_entity);
                }
            }
            if boom.struck.len() >= BOOM_LOCKS || boom.out >= BOOM_RETURN_TICKS {
                boom.returning = true;
            }
        }
        boom.life = boom.life.saturating_sub(1);
        let caught = boom.returning && home.is_some_and(|pos| within(boom.pos, pos, BOOM_CATCH));
        if caught || boom.life == 0 {
            if held.item == BOOMERANG {
                held.count = 1;
            }
            spent.push(entity);
        }
    }

    for entity in spent {
        let _ = commands.entity(entity).despawn();
    }
}
