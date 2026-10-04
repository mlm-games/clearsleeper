use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::combat::HitPoints;
use super::enemy::Hurt;
use super::events::{SimEvent, SimEvents};
use super::flags::{Switches, ZoneActors};
use super::ground::grounded;
use super::player::Player;
use super::water::Water;

pub const BOMB_ACTORS: [&str; 2] = ["Bomb", "Bomb2"];

pub const START_INSTANT: u8 = 0;
pub const START_INTERVAL: u8 = 1;

pub const MODE_WAIT: u8 = 0;
pub const MODE_CARRY: u8 = 1;
pub const MODE_EXPLODE: u8 = 2;
pub const MODE_SINK: u8 = 3;

pub const FUSE_FRAMES: i16 = 0x96;
pub const FUSE_CAP: i16 = 0x1E;
pub const EXPLODE_FRAMES: i16 = 4;
pub const GRAVITY: f32 = -2.9;
pub const MAX_FALL: f32 = -100.0;
pub const STTS_WEIGHT: u16 = 200;
pub const STTS_POWER: u8 = 0xFF;

pub const NO_GRAVITY: u32 = 1 << 16;
pub const CARRYABLE: u32 = 1 << 8;

const START_MASK: u32 = 0x3;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Life {
    Wait,
    Carry,
    Spent,
    Sunk,
}

impl Life {
    pub fn mode(self) -> u8 {
        match self {
            Self::Wait => MODE_WAIT,
            Self::Carry => MODE_CARRY,
            Self::Spent => MODE_EXPLODE,
            Self::Sunk => MODE_SINK,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bomb {
    pub life: Life,
    pub timer: i16,
    pub frames: i16,
    pub carried: bool,
    pub no_gravity: bool,
    pub instant: bool,
    pub fall: f32,
}

impl Bomb {
    pub fn spawn(parameters: u32) -> Self {
        let start = (parameters & START_MASK) as u8;
        Self {
            life: match start {
                START_INSTANT => Life::Spent,
                START_INTERVAL => Life::Wait,
                _ => Life::Carry,
            },
            timer: FUSE_FRAMES,
            frames: 0,
            carried: parameters & CARRYABLE == 0,
            no_gravity: parameters & NO_GRAVITY != 0,
            instant: start == START_INSTANT,
            fall: 0.0,
        }
    }

    pub fn release(&mut self) {
        if self.life == Life::Carry {
            self.life = Life::Wait;
        }
    }

    pub fn fuse(&mut self) {
        self.timer = self.timer.min(FUSE_CAP);
    }

    pub fn tick(&mut self) -> bool {
        if self.timer > 0 {
            self.timer -= 1;
        }
        self.timer == 0
    }

    pub fn speed(&mut self) -> f32 {
        self.fall += GRAVITY;
        self.fall.max(MAX_FALL)
    }
}

pub fn is_bomb(name: &str) -> bool {
    BOMB_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    water: Option<Res<Water>>,
    mut bombs: Query<(Entity, &mut Actor, &mut Bomb)>,
    mut foes: Query<
        (Entity, &Actor, &mut HitPoints, Option<&mut Hurt>),
        (
            Without<Bomb>,
            Without<super::bombflower::Bombflower>,
            Without<super::bombflower::Crawler>,
        ),
    >,
    players: Query<&Player>,
    mut zones: ResMut<ZoneActors>,
    mut events: ResMut<SimEvents>,
    mut published: ResMut<super::bombflower::Blasts>,
    mut fences: Query<(&Actor, &mut super::barricade::Barricade)>,
    mut floats: Query<
        (Entity, &Actor, &mut super::buoybarrel::BuoyBarrel),
        Without<Bomb>,
    >,
    mut switches: ResMut<Switches>,
    mut commands: Commands,
) {
    let solid = collision.as_deref();
    let wet = water.as_deref().map(|water| water.in_area).unwrap_or(false);
    let owner = players.iter().next().map(|player| player.pos);
    let mut burst: Vec<Vec3> = Vec::new();
    for (entity, mut actor, mut bomb) in &mut bombs {
        if bomb.life == Life::Carry {
            if bomb.carried && grounded(solid, actor.pos) {
                bomb.release();
            }
            continue;
        }
        if !bomb.no_gravity && (!wet || !grounded(solid, actor.pos)) {
            actor.pos.y += bomb.speed();
        }
        bomb.frames += 1;
        if bomb.life == Life::Wait && bomb.tick() {
            bomb.life = Life::Spent;
        }
        if bomb.life == Life::Spent && bomb.frames >= EXPLODE_FRAMES {
            burst.push(actor.pos);
            let _ = commands.entity(entity).despawn();
        }
    }
    for origin in burst {
        events.list.push(SimEvent::Explode);
        super::bombflower::blast(
            origin,
            &mut published,
            &mut foes,
            owner,
            &mut zones,
            &mut events,
            &mut fences,
            &mut floats,
            &mut switches,
            &mut commands,
        );
    }
}
