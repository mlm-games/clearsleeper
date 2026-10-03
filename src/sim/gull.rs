use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::player::Player;
use super::walk::{from_s16, to_s16};

pub const GULL_ACTOR: &str = "Kamome";

const GROUND_TYPE: u32 = 1;
const TAKE_OFF_RANGE: f32 = 400.0;
const SCATTER_RANGE: f32 = 300.0;
const SCATTER_TICKS: u32 = 50;
const ORBIT_RANGE: f32 = 300.0;
const CRUISE_HEIGHT: f32 = 100.0;
const CLIMB: f32 = 20.0;
const FLY_SPEED: f32 = 20.0;
const MOVE_SCALE: f32 = 0.25;
const TURN_RATE: f32 = 0.012;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Gull {
    pub home: Vec3,
    pub angle: f32,
    pub height: f32,
    pub flying: bool,
    pub startled: u32,
}

pub fn spawn(parameters: u32, actor: &Actor) -> Gull {
    let grounded = parameters & 0xF == GROUND_TYPE;
    Gull {
        home: actor.pos,
        angle: from_s16(actor.angle[1]),
        height: 0.0,
        flying: !grounded,
        startled: 0,
    }
}

pub fn control(
    collision: Option<Res<Collision>>,
    players: Query<&Player>,
    mut gulls: Query<(&mut Actor, &mut Gull)>,
) {
    let Some(player) = players.iter().next().copied() else {
        return;
    };
    let floored = collision.is_some();
    for (mut actor, mut gull) in &mut gulls {
        let gap = player.pos - actor.pos;
        let reach = gap.length();
        if !gull.flying {
            if reach >= TAKE_OFF_RANGE || !floored {
                continue;
            }
            gull.flying = true;
        }
        if reach < SCATTER_RANGE {
            gull.startled = SCATTER_TICKS;
        }
        if gull.startled > 0 {
            gull.startled -= 1;
            gull.angle += TURN_RATE * 4.0;
        } else {
            gull.angle += TURN_RATE;
        }
        gull.height = (gull.height + CLIMB).min(CRUISE_HEIGHT);
        let (sin, cos) = gull.angle.sin_cos();
        let target = if gull.startled > 0 {
            actor.pos + Vec3::new(gap.x, 0.0, gap.z).normalize_or_zero() * ORBIT_RANGE
        } else {
            gull.home + Vec3::new(ORBIT_RANGE * sin, gull.height, ORBIT_RANGE * cos)
        };
        let (dx, dz) = (target.x - actor.pos.x, target.z - actor.pos.z);
        if dx * dx + dz * dz > 1.0 {
            let yaw = dx.atan2(dz);
            actor.angle[1] = to_s16(yaw);
            actor.pos.x += FLY_SPEED * MOVE_SCALE * yaw.sin();
            actor.pos.z += FLY_SPEED * MOVE_SCALE * yaw.cos();
        }
        actor.pos.y = gull.home.y + gull.height;
    }
}
