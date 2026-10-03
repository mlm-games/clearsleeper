use std::f32::consts::FRAC_PI_2;
use std::f32::consts::PI;

use bevy_ecs::prelude::*;

use super::collision::Collision;
use super::ground::grounded;
use super::health::Health;
use super::intent::{CameraYaw, DoIntent, MoveIntent};
use super::lock::Lock;
use super::player::Player;
use super::roll::Roll;
use super::swim::Swim;
use super::walk::{RAD_PER_UNIT, target_yaw, to_s16};

const STICK_GATE: f32 = 0.05;
const BACK_SPEED: f32 = 22.5;
const BACK_RISE: f32 = 19.0;
pub(crate) const BACK_GRAVITY: f32 = -3.0;
const SIDE_PITCH: f32 = 6200.0 * RAD_PER_UNIT;
const SIDE_SPEED: f32 = 30.0;
pub(crate) const SIDE_GRAVITY: f32 = -2.4;
pub(crate) const SIDE_OFFSET: f32 = FRAC_PI_2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JumpKind {
    #[default]
    Back,
    Left,
    Right,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Jump {
    pub active: bool,
    pub kind: JumpKind,
    held: bool,
}

enum Direction {
    Forward,
    Left,
    Right,
    Back,
}

fn direction(desired: i16, face: i16) -> Direction {
    let diff = desired.wrapping_sub(face);
    if diff.unsigned_abs() > 0x6000 {
        Direction::Back
    } else if diff >= 0x2000 {
        Direction::Left
    } else if diff <= -0x2000 {
        Direction::Right
    } else {
        Direction::Forward
    }
}

pub fn control(
    collision: Option<Res<Collision>>,
    do_intent: Res<DoIntent>,
    intent: Res<MoveIntent>,
    camera: Res<CameraYaw>,
    health: Res<Health>,
    swim: Res<Swim>,
    lock: Res<Lock>,
    mut roll: ResMut<Roll>,
    mut jump: ResMut<Jump>,
    mut players: Query<&mut Player>,
) {
    let pressed = do_intent.0 && !jump.held;
    jump.held = do_intent.0;
    for mut player in &mut players {
        let player = &mut *player;
        if swim.active || health.units == 0 {
            jump.active = false;
            continue;
        }
        if jump.active {
            if grounded(collision.as_deref(), player.pos) && player.speed_y <= 0.0 {
                jump.active = false;
                player.speed = 0.0;
                player.yaw = player.face;
            }
            continue;
        }
        if !pressed
            || lock.target.is_none()
            || !grounded(collision.as_deref(), player.pos)
            || intent.axis.length().min(1.0) <= STICK_GATE
        {
            continue;
        }
        let desired = target_yaw(&intent, &camera);
        match direction(desired, to_s16(player.face)) {
            Direction::Forward => continue,
            Direction::Back => {
                player.yaw = player.face + PI;
                player.speed = BACK_SPEED;
                player.speed_y = BACK_RISE;
                jump.kind = JumpKind::Back;
            }
            Direction::Left => {
                player.yaw = player.face + SIDE_OFFSET;
                player.speed = SIDE_PITCH.cos() * SIDE_SPEED;
                player.speed_y = SIDE_PITCH.sin() * SIDE_SPEED;
                jump.kind = JumpKind::Left;
            }
            Direction::Right => {
                player.yaw = player.face - SIDE_OFFSET;
                player.speed = SIDE_PITCH.cos() * SIDE_SPEED;
                player.speed_y = SIDE_PITCH.sin() * SIDE_SPEED;
                jump.kind = JumpKind::Right;
            }
        }
        jump.active = true;
        roll.active = false;
    }
}
