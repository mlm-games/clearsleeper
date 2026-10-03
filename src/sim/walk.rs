//! Ground locomotion: move intent → camera-relative target yaw → turn/speed rates.

use std::f32::consts::TAU;

use bevy_ecs::prelude::*;
use glam::Vec2;

use super::events::{SimEvent, SimEvents, Stride, STRIDE_UNITS};
use super::hop::{HOP_ACCEL, HOP_RISE_STEP, Hop};
use super::intent::{CameraYaw, MoveIntent};
use super::jump::{BACK_GRAVITY, Jump, JumpKind, SIDE_GRAVITY, SIDE_OFFSET};
use super::player::Player;
use super::roll::Roll;
use super::ship::Ship;
use super::swim::{
    Swim, BACKWARD_LIMIT, SWIM_DECEL_MAX, SWIM_DECEL_MIN, SWIM_DECEL_SCALE, SWIM_DRIVE,
    SWIM_MAX_SPEED, SWIM_TURN_MAX, SWIM_TURN_MIN, SWIM_TURN_SCALE,
};

const STICK_GATE: f32 = 0.05;
pub(crate) const MAX_NORMAL_SPEED: f32 = 17.0;
const MOVE_ACCEL: f32 = 3.5;
const DECEL_SCALE: f32 = 0.6;
const DECEL_MAX_STEP: f32 = 2.5;
const DECEL_MIN_STEP: f32 = 1.8;
const ACCEL_EPSILON: f32 = 3.8146973e-6;
const TURN_SCALE: i16 = 4;
const TURN_PARAM: i16 = 3000;
const TURN_PARAM_FLOOR: i16 = 10;
const TURN_RATE: i16 = 100;
const TURN_RATE_FLOOR: i16 = 1;
const REVERSAL_LIMIT: u16 = 0x7800;
const REVERSAL_PARAM: i16 = 3000;
const REVERSAL_RATE: i16 = 100;
const SPEED_F_EPS: f32 = 0.05;
const GRAVITY: f32 = -2.5;
const MAX_FALL_SPEED: f32 = -100.0;
const UNITS_PER_RAD: f32 = 65536.0 / TAU;
pub(crate) const RAD_PER_UNIT: f32 = TAU / 65536.0;

pub(super) fn to_s16(radians: f32) -> i16 {
    (radians * UNITS_PER_RAD).round().rem_euclid(65536.0) as i32 as i16
}

pub(crate) fn from_s16(units: i16) -> f32 {
    units as f32 * RAD_PER_UNIT
}

fn angle_distance(a: i16, b: i16) -> u16 {
    a.wrapping_sub(b).unsigned_abs()
}

pub(super) fn add_calc(value: &mut f32, target: f32, scale: f32, max_step: f32, min_step: f32) {
    if *value != target {
        let mut step = scale * (target - *value);
        if step >= min_step || step <= -min_step {
            step = step.clamp(-max_step, max_step);
            *value += step;
        } else if step > 0.0 {
            *value += min_step;
            if *value > target {
                *value = target;
            }
        } else {
            *value -= min_step;
            if *value < target {
                *value = target;
            }
        }
    }
}

pub(super) fn add_calc_angle_s(
    value: i16,
    target: i16,
    scale: i16,
    max_step: i16,
    min_step: i16,
) -> i16 {
    let mut value = value;
    let mut diff = target.wrapping_sub(value);
    if value != target {
        let mut step = diff / scale;
        if step > min_step || step < -min_step {
            step = step.clamp(-max_step, max_step);
            value = value.wrapping_add(step);
        } else if diff >= 0 {
            value = value.wrapping_add(min_step);
            diff = target.wrapping_sub(value);
            if diff <= 0 {
                value = target;
            }
        } else {
            value = value.wrapping_sub(min_step);
            diff = target.wrapping_sub(value);
            if diff >= 0 {
                value = target;
            }
        }
    }
    value
}

fn set_normal_speed(
    player: &mut Player,
    desired: f32,
    drive: f32,
    scale: f32,
    min_step: f32,
    max_step: f32,
) {
    let (step_cap, decel_target) = if desired < player.speed {
        let span = player.speed - desired;
        (span.clamp(min_step, max_step), desired)
    } else {
        (max_step, 0.0)
    };
    if drive.abs() < ACCEL_EPSILON {
        add_calc(
            &mut player.speed,
            decel_target,
            scale,
            step_cap,
            min_step,
        );
    } else {
        player.speed += drive;
        if player.speed > desired {
            player.speed = desired;
        }
    }
}

pub(crate) fn target_yaw(intent: &MoveIntent, camera: &CameraYaw) -> i16 {
    let forward = Vec2::new(camera.0.sin(), camera.0.cos());
    let right = Vec2::new(-camera.0.cos(), camera.0.sin());
    let direction = forward * intent.axis.y + right * intent.axis.x;
    to_s16(direction.x.atan2(direction.y))
}

pub(crate) fn facing_error(intent: &MoveIntent, camera: &CameraYaw, yaw: f32) -> u16 {
    angle_distance(target_yaw(intent, camera), to_s16(yaw))
}

pub fn walk(
    intent: Res<MoveIntent>,
    camera: Res<CameraYaw>,
    swim: Res<Swim>,
    hop: Res<Hop>,
    roll: Res<Roll>,
    jump: Res<Jump>,
    ships: Query<&Ship>,
    mut players: Query<&mut Player>,
    mut stride: ResMut<Stride>,
    mut events: ResMut<SimEvents>,
) {
    if ships.iter().any(|ship| ship.riding) {
        return;
    }
    for mut player in &mut players {
        let player = &mut *player;
        if swim.active {
            swim_tick(player, &intent, &camera);
        } else if hop.active {
            hop_tick(player);
        } else if roll.active {
            roll_tick(player);
        } else if jump.active {
            jump_tick(player, &jump);
        } else {
            walk_tick(player, &intent, &camera, &mut stride, &mut events);
        }
    }
}

fn hop_tick(player: &mut Player) {
    if player.speed < MAX_NORMAL_SPEED {
        player.speed = (player.speed + HOP_ACCEL).min(MAX_NORMAL_SPEED);
    } else if player.speed > MAX_NORMAL_SPEED {
        player.speed = (player.speed - HOP_ACCEL).max(MAX_NORMAL_SPEED);
    }
    let speed_f = if player.speed.abs() < SPEED_F_EPS {
        0.0
    } else {
        player.speed
    };
    player.pos.x += speed_f * player.yaw.sin();
    player.pos.z += speed_f * player.yaw.cos();
    player.speed_y = (player.speed_y + GRAVITY).max(MAX_FALL_SPEED);
    if player.speed_y > 0.0 {
        player.speed_y = (player.speed_y - HOP_RISE_STEP).max(0.0);
    }
    player.pos.y += player.speed_y;
}

fn roll_tick(player: &mut Player) {
    let speed_f = if player.speed.abs() < SPEED_F_EPS {
        0.0
    } else {
        player.speed
    };
    player.pos.x += speed_f * player.yaw.sin();
    player.pos.z += speed_f * player.yaw.cos();
    player.speed_y = (player.speed_y + GRAVITY).max(MAX_FALL_SPEED);
    player.pos.y += player.speed_y;
}

fn jump_tick(player: &mut Player, jump: &Jump) {
    let side = match jump.kind {
        JumpKind::Back => None,
        JumpKind::Left => Some(SIDE_OFFSET),
        JumpKind::Right => Some(-SIDE_OFFSET),
    };
    if let Some(offset) = side {
        player.yaw = player.face + offset;
    }
    let gravity = match jump.kind {
        JumpKind::Back => BACK_GRAVITY,
        JumpKind::Left | JumpKind::Right => SIDE_GRAVITY,
    };
    let speed_f = if player.speed.abs() < SPEED_F_EPS {
        0.0
    } else {
        player.speed
    };
    player.pos.x += speed_f * player.yaw.sin();
    player.pos.z += speed_f * player.yaw.cos();
    player.speed_y = (player.speed_y + gravity).max(MAX_FALL_SPEED);
    player.pos.y += player.speed_y;
}

fn walk_tick(
    player: &mut Player,
    intent: &MoveIntent,
    camera: &CameraYaw,
    stride: &mut Stride,
    events: &mut SimEvents,
) {
    let stick = intent.axis.length().min(1.0);
    let stick_sq = stick * stick;
    let desired = MAX_NORMAL_SPEED * stick_sq;
    let accel = if stick > STICK_GATE {
        let target = target_yaw(intent, camera);
        let yaw = to_s16(player.yaw);
        if angle_distance(target, yaw) > REVERSAL_LIMIT {
            player.yaw =
                from_s16(add_calc_angle_s(yaw, target, TURN_SCALE, REVERSAL_PARAM, REVERSAL_RATE));
            0.0
        } else {
            let max_step = ((TURN_PARAM as f32 * stick_sq) as i16).max(TURN_PARAM_FLOOR);
            let min_step = ((TURN_RATE as f32 * stick_sq) as i16).max(TURN_RATE_FLOOR);
            player.yaw = from_s16(add_calc_angle_s(yaw, target, TURN_SCALE, max_step, min_step));
            let mut drive = (from_s16(target) - player.yaw).cos();
            if player.speed > 0.5 * MAX_NORMAL_SPEED {
                drive = drive.max(0.7);
            } else {
                drive = drive.max(0.0);
            }
            let gate = 0.5 - 0.5 * (player.speed / MAX_NORMAL_SPEED).abs();
            if stick > gate {
                drive * MOVE_ACCEL * stick_sq
            } else {
                0.0
            }
        }
    } else {
        0.0
    };
    set_normal_speed(
        player,
        desired,
        accel,
        DECEL_SCALE,
        DECEL_MIN_STEP,
        DECEL_MAX_STEP,
    );
    let speed_f = if player.speed.abs() < SPEED_F_EPS {
        0.0
    } else {
        player.speed
    };
    player.pos.x += speed_f * player.yaw.sin();
    player.pos.z += speed_f * player.yaw.cos();
    stride.0 += speed_f;
    while stride.0 >= STRIDE_UNITS {
        stride.0 -= STRIDE_UNITS;
        events.list.push(SimEvent::Footstep);
    }
    player.speed_y = (player.speed_y + GRAVITY).max(MAX_FALL_SPEED);
    player.pos.y += player.speed_y;
}

fn swim_tick(player: &mut Player, intent: &MoveIntent, camera: &CameraYaw) {
    let stick = intent.axis.length().min(1.0);
    let drive = if stick > STICK_GATE {
        let target = target_yaw(intent, camera);
        let before = player.yaw;
        let yaw = to_s16(player.yaw);
        if angle_distance(target, yaw) > BACKWARD_LIMIT {
            player.yaw = from_s16(target);
        } else {
            player.yaw = from_s16(add_calc_angle_s(
                yaw,
                target,
                SWIM_TURN_SCALE,
                SWIM_TURN_MAX,
                SWIM_TURN_MIN,
            ));
        }
        SWIM_DRIVE * stick * (player.yaw - before).cos()
    } else {
        0.0
    };
    set_normal_speed(
        player,
        SWIM_MAX_SPEED * stick * stick,
        drive,
        SWIM_DECEL_SCALE,
        SWIM_DECEL_MIN,
        SWIM_DECEL_MAX,
    );
    let speed_f = if player.speed.abs() < SPEED_F_EPS {
        0.0
    } else {
        player.speed
    };
    player.pos.x += speed_f * player.yaw.sin();
    player.pos.z += speed_f * player.yaw.cos();
}
