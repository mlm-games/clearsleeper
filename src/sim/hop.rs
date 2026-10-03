use bevy_ecs::prelude::*;

use super::collision::Collision;
use super::combat::Combat;
use super::ground::grounded;
use super::health::Health;
use super::intent::{CameraYaw, MoveIntent};
use super::jump::Jump;
use super::player::Player;
use super::roll::Roll;
use super::swim::Swim;
use super::walk::{MAX_NORMAL_SPEED, RAD_PER_UNIT, facing_error};

const HOP_DROP: f32 = 30.1;
const HOP_STICK: f32 = 0.85;
const HOP_MIN_SPEED: f32 = 9.0;
const HOP_FACING: u16 = 0x800;
const HOP_ANGLE: f32 = 11000.0 * RAD_PER_UNIT;
const HOP_SCALE: f32 = 1.6;
pub(crate) const HOP_ACCEL: f32 = 0.4;
pub(crate) const HOP_RISE_STEP: f32 = 0.5;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Hop {
    pub active: bool,
    prev_grounded: bool,
}

pub fn launch(speed_f: f32) -> (f32, f32) {
    (
        speed_f * HOP_ANGLE.cos() * HOP_SCALE,
        speed_f * HOP_ANGLE.sin() * HOP_SCALE,
    )
}

pub fn control(
    collision: Option<Res<Collision>>,
    intent: Res<MoveIntent>,
    camera: Res<CameraYaw>,
    swim: Res<Swim>,
    health: Res<Health>,
    jump: Res<Jump>,
    combat: Res<Combat>,
    mut hop: ResMut<Hop>,
    mut roll: ResMut<Roll>,
    mut players: Query<&mut Player>,
) {
    for mut player in &mut players {
        let player = &mut *player;
        let grounded_now = grounded(collision.as_deref(), player.pos);
        let left_ground = hop.prev_grounded && !grounded_now;
        hop.prev_grounded = grounded_now;
        if swim.active || health.units == 0 || jump.active || combat.active {
            hop.active = false;
            continue;
        }
        if hop.active {
            if grounded_now {
                hop.active = false;
                player.speed = 0.0;
            }
            continue;
        }
        if !left_ground {
            continue;
        }
        let Some(collision) = collision.as_deref() else {
            continue;
        };
        let Some(surface) = collision.ground_at(player.pos.x, player.pos.z) else {
            continue;
        };
        if surface - player.pos.y >= -HOP_DROP {
            continue;
        }
        if player.speed < HOP_MIN_SPEED {
            continue;
        }
        if intent.axis.length().min(1.0) <= HOP_STICK {
            continue;
        }
        if facing_error(&intent, &camera, player.yaw) >= HOP_FACING {
            continue;
        }
        let (forward, up) = launch(player.speed.clamp(HOP_MIN_SPEED, MAX_NORMAL_SPEED));
        player.speed = forward;
        player.speed_y = up;
        hop.active = true;
        roll.active = false;
    }
}
