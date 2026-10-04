use bevy_ecs::prelude::*;

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

pub const STATE_NORMAL: u8 = 0;
pub const STATE_FUSED: u8 = 1;
pub const STATE_BURNING: u8 = 2;
pub const STATE_WATER: u8 = 3;
pub const STATE_LIT: u8 = 4;
pub const STATE_CARRIED: u8 = 5;
pub const STATE_NUT: u8 = 6;
pub const STATE_BALLOON: u8 = 7;
pub const STATE_BLUE: u8 = 8;

pub const FUSE_FRAMES: i16 = 0x96;
pub const RESET_FRAMES: i16 = 30;
pub const GRAVITY: f32 = -2.9;
pub const MAX_FALL: f32 = -100.0;
pub const NO_GRAVITY_FRAMES: i16 = 30;
pub const FUSE_HEIGHT: f32 = 60.0;
pub const PARAM_VERSION: u32 = 1;
pub const PARAM_MARK: u32 = 0x8000_0000;

const STATE_SHIFT: u32 = 0;


#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Life {
    Rolling,
    Fused,
    Spent,
    Sunk,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bomb {
    pub state: u8,
    pub rest: i16,
    pub no_gravity: i16,
    pub life: Life,
    fall: f32,
}

impl Bomb {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            state: ((parameters >> STATE_SHIFT) & 0xFF) as u8,
            rest: FUSE_FRAMES,
            no_gravity: NO_GRAVITY_FRAMES,
            life: Life::Rolling,
            fall: 0.0,
        }
    }

    pub fn portable(&self) -> bool {
        self.state != STATE_LIT
    }

    pub fn lit(&mut self) {
        self.state = STATE_BLUE;
    }

    pub fn arm(&mut self) {
        self.state = STATE_NORMAL;
    }
}

pub fn is_bomb(name: &str) -> bool {
    BOMB_ACTORS.contains(&name)
}

pub fn make_parameters(state: u8, no_gravity: bool, version: bool) -> u32 {
    PARAM_MARK
        | (u32::from(version) << 31)
        | (u32::from(no_gravity) << 16)
        | u32::from(state)
}

pub fn fuse_height(y: f32) -> f32 {
    y + FUSE_HEIGHT
}

pub fn arm(bombs: &mut Query<&mut Bomb>, entity: Entity) {
    if let Ok(mut bomb) = bombs.get_mut(entity) {
        bomb.arm();
    }
}

pub fn control(
    collision: Option<Res<Collision>>,
    water: Option<Res<Water>>,
    mut bombs: Query<(Entity, &mut Actor, &mut Bomb)>,
    mut foes: Query<
        (Entity, &Actor, &mut HitPoints, Option<&mut Hurt>),
        (Without<super::bombflower::Bombflower>, Without<super::bombflower::Crawler>),
    >,
    players: Query<&Player>,
    mut zones: ResMut<ZoneActors>,
    mut events: ResMut<SimEvents>,
    mut fences: Query<&mut super::barricade::Barricade>,
    mut blocks: Query<&mut super::ice::Ice>,
    mut floats: Query<(Entity, &Actor, &mut super::buoybarrel::BuoyBarrel)>,
    mut switches: ResMut<Switches>,
    mut commands: Commands,
) {
    let solid = collision.as_deref();
    let wet = water.as_deref().map(|water| water.in_area).unwrap_or(false);
    let owner = players.iter().next().map(|player| player.pos);
    let mut burst: Vec<Vec3> = Vec::new();
    for (entity, mut actor, mut bomb) in &mut bombs {
        if bomb.no_gravity > 0 {
            bomb.no_gravity -= 1;
        } else if !wet || !grounded(solid, actor.pos) {
            actor.pos.y += bomb.speed();
        }
        match bomb.life {
            Life::Spent | Life::Sunk | Life::Rolling => continue,
            Life::Fused => {
                if bomb.rest > 0 {
                    bomb.rest -= 1;
                }
                if bomb.rest == 0 {
                    bomb.life = Life::Spent;
                    burst.push(actor.pos);
                    let _ = commands.entity(entity).despawn();
                }
            }
        }
    }
    for origin in burst {
        events.list.push(SimEvent::Explode);
        super::bombflower::blast(
            origin,
            &mut foes,
            owner,
            &mut zones,
            &mut events,
            &mut fences,
            &mut blocks,
            &mut floats,
            &mut switches,
            &mut commands,
        );
    }
}

impl Bomb {
    fn speed(&mut self) -> f32 {
        self.fall += GRAVITY;
        self.fall.max(MAX_FALL)
    }
}