use bevy_ecs::prelude::*;

use super::collision::Collision;
use super::ground::grounded;
use super::health::Health;
use super::intent::{CameraYaw, DoIntent, MoveIntent};
use super::player::Player;
use super::swim::Swim;
use super::walk::{from_s16, target_yaw};

const ROLL_STICK: f32 = 0.75;
const ROLL_IDLE: f32 = 0.05;
const ROLL_SPEED_SCALE: f32 = 1.5;
const ROLL_SPEED_BASE: f32 = 0.5;
const ROLL_SPEED_MIN: f32 = 5.0;
const ROLL_SPEED_MAX: f32 = 26.0;
const ROLL_RATE: f32 = 1.1;
const ROLL_END: f32 = 19.0;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Roll {
    pub active: bool,
    pub frame: f32,
    held: bool,
}

pub fn control(
    do_intent: Res<DoIntent>,
    intent: Res<MoveIntent>,
    camera: Res<CameraYaw>,
    swim: Res<Swim>,
    health: Res<Health>,
    collision: Option<Res<Collision>>,
    mut roll: ResMut<Roll>,
    mut players: Query<&mut Player>,
) {
    for mut player in &mut players {
        let player = &mut *player;
        let pressed = do_intent.0 && !roll.held;
        roll.held = do_intent.0;
        if roll.active {
            roll.frame += ROLL_RATE;
            if roll.frame >= ROLL_END {
                roll.active = false;
                if intent.axis.length() <= ROLL_IDLE {
                    player.speed -= ROLL_SPEED_MIN;
                }
                continue;
            }
        }
        if roll.active || !pressed || swim.active || health.units == 0 {
            continue;
        }
        let stick = intent.axis.length().min(1.0);
        if stick <= ROLL_STICK || !grounded(collision.as_deref(), player.pos) {
            continue;
        }
        player.yaw = from_s16(target_yaw(&intent, &camera));
        player.speed = (player.speed * ROLL_SPEED_SCALE + ROLL_SPEED_BASE)
            .clamp(ROLL_SPEED_MIN, ROLL_SPEED_MAX);
        roll.active = true;
        roll.frame = 0.0;
    }
}
