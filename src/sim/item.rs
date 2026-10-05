use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::arrow;
use super::boomerang;
use super::collision::Collision;
use super::combat::{Combat, HitPoints};
use super::damage;
use super::enemy::Hurt;
use super::events::{SimEvent, SimEvents};
use super::ground::{self, grounded};
use super::health::Health;
use super::hop::Hop;
use super::hookshot;
use super::intent::UseIntent;
use super::jump::Jump;
use super::player::Player;
use super::roll::Roll;
use super::ship::Ship;
use super::swim::Swim;
use super::talk::Dialogue;
use super::walk::within;

pub const FIRST_BOMB: u8 = 0x0B;
pub const LAST_BOMB: u8 = 0x0E;
pub const FIRST_ARROW: u8 = 0x10;
pub const LAST_ARROW: u8 = 0x12;
pub const BOOMERANG: u8 = 0x2D;
pub const HOOKSHOT: u8 = 0x2F;

const BOMB_COUNTS: [u8; 4] = [5, 10, 20, 30];
const ARROW_COUNTS: [u8; 3] = [10, 20, 30];
const BOMB_CAP: u8 = 30;
const ARROW_CAP: u8 = 30;

const BOMB_FUSE: u32 = 100;
const BOMB_RADIUS: f32 = 200.0;
const BOMB_POWER: i32 = damage::BOMB;
const BOMB_THROW: f32 = 50.0;
const BOMB_LIFT: f32 = 40.0;
const BOMB_GRAVITY: f32 = -2.5;
const THROW_OFFSET: f32 = 20.0;
const BOMB_FLINCH: u32 = 10;

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
    Hookshot,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bomb {
    pub pos: Vec3,
    pub fuse: u32,
    pub vel: Vec3,
    pub rest: bool,
}

impl Held {
    pub fn equip(&mut self, item: u8) {
        let (slot, count) = match item {
            FIRST_BOMB..=LAST_BOMB => (FIRST_BOMB, BOMB_COUNTS[(item - FIRST_BOMB) as usize]),
            FIRST_ARROW..=LAST_ARROW => (FIRST_ARROW, ARROW_COUNTS[(item - FIRST_ARROW) as usize]),
            BOOMERANG => (BOOMERANG, 1),
            HOOKSHOT => (HOOKSHOT, 1),
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
            HOOKSHOT => Some(Kind::Hookshot),
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
    lock: Res<'w, super::lock::Lock>,
    ships: Query<'w, 's, &'static Ship>,
}

impl Gates<'_, '_> {
    pub fn locked(&self) -> usize {
        usize::from(self.lock.target.is_some())
    }
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
    bombs: Query<'w, 's, (Entity, &'static mut Bomb), Without<arrow::Arrow>>,
    nocked: Query<'w, 's, &'static arrow::Arrow>,
    spin: Query<'w, 's, (Entity, &'static mut boomerang::Boomerang), Without<Bomb>>,
    hooked: Query<'w, 's, &'static hookshot::Hookshot>,
}

impl Shots<'_, '_> {
    fn stowed(&self) -> bool {
        self.hooked
            .iter()
            .all(|hook| matches!(hook.phase, hookshot::Phase::Wait))
    }
}

impl Shots<'_, '_> {
    fn spinning(&self) -> bool {
        self.spin.iter().next().is_some()
    }

    fn recall(&mut self) {
        for (_, mut boom) in &mut self.spin {
            boom.returning();
        }
    }
}

fn launch(
    player: &Player,
    held: &mut Held,
    rounds: u8,
    locks: usize,
    commands: &mut Commands,
) {
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
            let mut shaft = arrow::Arrow::nocked(arrow::TYPE_NORMAL, rounds);
            shaft.pos = pos;
            shaft.yaw = player.yaw;
            shaft.shoot(0.0, player.yaw);
            commands.spawn((shaft, InFlight));
        }
        Kind::Hookshot => {
            let mut hook = hookshot::Hookshot::nocked(pos);
            hook.launch(0.0, player.face);
            commands.spawn((hook, InFlight));
        }
        Kind::Boomerang => {
            let mut boom = boomerang::Boomerang::nocked();
            boom.toss(pos, player.face, 0.0, locks);
            commands.spawn((boom, InFlight));
        }
    }
    held.count -= 1;
}

pub fn control(
    collision: Option<Res<Collision>>,
    intent: Res<UseIntent>,
    mut health: ResMut<Health>,
    mut held: ResMut<Held>,
    mut zones: ResMut<super::flags::ZoneActors>,
    players: Query<&Player>,
    gates: Gates,
    mut shots: Shots,
    mut foes: Query<(Entity, &Actor, &mut HitPoints, Option<&mut Hurt>), Without<Ship>>,
    mut events: ResMut<SimEvents>,
    mut blasts: ResMut<super::bombflower::Blasts>,
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
        } else if ready && shots.stowed() {
            let rounds = shots
                .nocked
                .iter()
                .next()
                .map_or(0, |arrow| (arrow.round + 1) % arrow::ROUNDS);
            let locks = gates.locked();
            for player in &players {
                launch(player, &mut held, rounds, locks, &mut commands);
            }
        }
    }

    let mut dead: Vec<(Entity, i8, u16)> = Vec::new();
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
                hp.0 -= BOMB_POWER;
                if let Some(hurt) = hurt.as_deref_mut() {
                    hurt.0 = BOMB_FLINCH;
                }
                if hp.0 <= 0 {
                    dead.push((foe_entity, foe.room, foe.set_id));
                }
            }
        }
        blasts.list.push(blast);
        if owner.is_some_and(|pos| within(blast, pos, BOMB_RADIUS)) {
            health.damage(BOMB_POWER as u8);
        }
        events.list.push(SimEvent::Explode);
        let _ = commands.entity(entity).despawn();
    }
    for entity in chain {
        if let Ok((_, mut bomb)) = shots.bombs.get_mut(entity) {
            bomb.fuse = 0;
        }
    }
    for (entity, room, set_id) in dead {
        zones.mark_done(room, set_id);
        let _ = commands.entity(entity).despawn();
    }

}
