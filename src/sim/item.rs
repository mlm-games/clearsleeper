use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::combat::{Combat, HitPoints};
use super::ground::grounded;
use super::health::Health;
use super::hop::Hop;
use super::intent::UseIntent;
use super::jump::Jump;
use super::player::Player;
use super::roll::Roll;
use super::ship::Ship;
use super::swim::Swim;
use super::talk::Dialogue;

pub const FIRST_BOMB: u8 = 0x0B;
pub const LAST_BOMB: u8 = 0x0E;
pub const FIRST_ARROW: u8 = 0x10;
pub const LAST_ARROW: u8 = 0x12;

const BOMB_COUNTS: [u8; 4] = [5, 10, 20, 30];
const ARROW_COUNTS: [u8; 3] = [10, 20, 30];

const BOMB_FUSE: u32 = 100;
const BOMB_RADIUS: f32 = 200.0;
const BOMB_POWER: u8 = 4;
const BOMB_THROW: f32 = 90.0;
const BOMB_LIFT: f32 = 40.0;
const BOMB_GRAVITY: f32 = -2.5;
const ARROW_SPEED: f32 = 200.0;
const ARROW_GRAVITY: f32 = -2.0;
const ARROW_FALL: f32 = -100.0;
const ARROW_POWER: i32 = 2;
const ARROW_REACH: f32 = 30.0;
const ARROW_LIFE: u32 = 120;
const THROW_OFFSET: f32 = 20.0;

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

impl Held {
    pub fn equip(&mut self, item: u8) {
        match item {
            FIRST_BOMB..=LAST_BOMB => {
                self.item = FIRST_BOMB;
                self.count = BOMB_COUNTS[(item - FIRST_BOMB) as usize];
            }
            FIRST_ARROW..=LAST_ARROW => {
                self.item = FIRST_ARROW;
                self.count = ARROW_COUNTS[(item - FIRST_ARROW) as usize];
            }
            _ => {}
        }
    }

    pub fn kind(&self) -> Option<Kind> {
        match self.item {
            FIRST_BOMB..=LAST_BOMB => Some(Kind::Bomb),
            FIRST_ARROW..=LAST_ARROW => Some(Kind::Arrow),
            _ => None,
        }
    }
}

fn surface(collision: Option<&Collision>, pos: Vec3) -> Option<f32> {
    match collision {
        Some(collision) => collision.ground_at(pos.x, pos.z),
        None => Some(0.0),
    }
}

fn near(from: Vec3, to: Vec3, radius: f32) -> bool {
    (to - from).length_squared() <= radius * radius
}

fn segment(from: Vec3, to: Vec3, radius: f32) -> bool {
    let travel = to - from;
    let length = travel.length();
    if length < 1e-4 {
        return near(from, to, radius);
    }
    let dir = travel / length;
    let gap = from - to;
    let along = (gap.dot(dir)).clamp(0.0, length);
    near(from + dir * along, to, radius)
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
            commands.spawn(Bomb {
                pos,
                fuse: BOMB_FUSE,
                vel: Vec3::new(BOMB_THROW * sin, BOMB_LIFT, BOMB_THROW * cos),
                rest: false,
            });
        }
        Kind::Arrow => {
            commands.spawn(Arrow {
                pos,
                vel: Vec3::new(ARROW_SPEED * sin, 0.0, ARROW_SPEED * cos),
                life: ARROW_LIFE,
            });
        }
    }
    held.count -= 1;
}

pub fn control(
    collision: Option<Res<Collision>>,
    intent: Res<UseIntent>,
    swim: Res<Swim>,
    roll: Res<Roll>,
    hop: Res<Hop>,
    jump: Res<Jump>,
    combat: Res<Combat>,
    dialogue: Res<Dialogue>,
    ships: Query<&Ship>,
    players: Query<&Player>,
    mut health: ResMut<Health>,
    mut held: ResMut<Held>,
    mut bombs: Query<(Entity, &mut Bomb), Without<Arrow>>,
    mut arrows: Query<(Entity, &mut Arrow), Without<Bomb>>,
    mut foes: Query<(Entity, &Actor, &mut HitPoints), Without<Ship>>,
    mut commands: Commands,
) {
    let pressed = intent.0 && !held.pressed;
    held.pressed = intent.0;
    if pressed
        && health.units > 0
        && !swim.active
        && !roll.active
        && !hop.active
        && !jump.active
        && !combat.active
        && !dialogue.open
        && !ships.iter().any(|ship| ship.riding)
        && grounded(
            collision.as_deref(),
            players.iter().next().map_or(Vec3::ZERO, |p| p.pos),
        )
    {
        for player in &players {
            launch(player, &mut held, &mut commands);
        }
    }

    let placed: Vec<(Entity, Vec3)> = bombs.iter().map(|(e, bomb)| (e, bomb.pos)).collect();
    let mut dead: Vec<Entity> = Vec::new();
    let mut chain: Vec<Entity> = Vec::new();
    for (entity, mut bomb) in &mut bombs {
        if !bomb.rest {
            bomb.vel.y += BOMB_GRAVITY;
            let step = bomb.vel;
            bomb.pos += step;
            if let Some(floor) = surface(collision.as_deref(), bomb.pos)
                && bomb.pos.y <= floor
            {
                bomb.pos.y = floor;
                bomb.vel = Vec3::ZERO;
                bomb.rest = true;
            }
        }
        bomb.fuse = bomb.fuse.saturating_sub(1);
        if bomb.fuse > 0 {
            continue;
        }
        for (other, pos) in &placed {
            if *other != entity && near(bomb.pos, *pos, BOMB_RADIUS) {
                chain.push(*other);
            }
        }
        for (foe_entity, foe, mut hp) in &mut foes {
            if near(bomb.pos, foe.pos, BOMB_RADIUS) {
                hp.0 -= BOMB_POWER as i32;
                if hp.0 <= 0 {
                    dead.push(foe_entity);
                }
            }
        }
        health.damage(BOMB_POWER);
        let _ = commands.entity(entity).despawn();
    }
    for entity in chain {
        if let Ok((_, mut bomb)) = bombs.get_mut(entity) {
            bomb.fuse = 0;
        }
    }
    for entity in dead {
        let _ = commands.entity(entity).despawn();
    }

    let mut spent: Vec<Entity> = Vec::new();
    for (entity, mut arrow) in &mut arrows {
        let from = arrow.pos;
        arrow.vel.y = (arrow.vel.y + ARROW_GRAVITY).max(ARROW_FALL);
        let step = arrow.vel;
        arrow.pos += step;
        for (_, foe, mut hp) in &mut foes {
            if segment(from, foe.pos, ARROW_REACH) {
                hp.0 -= ARROW_POWER;
                spent.push(entity);
                break;
            }
        }
        arrow.life = arrow.life.saturating_sub(1);
        if !spent.contains(&entity)
            && (arrow.life == 0 || grounded(collision.as_deref(), arrow.pos))
        {
            spent.push(entity);
        }
    }
    for entity in spent {
        let _ = commands.entity(entity).despawn();
    }
}
