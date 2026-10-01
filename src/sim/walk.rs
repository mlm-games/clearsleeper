//! Ground locomotion: move intent → camera-relative target yaw → turn/speed rates.

use std::f32::consts::TAU;

use bevy_ecs::prelude::*;
use glam::Vec2;

use super::intent::{CameraYaw, MoveIntent};
use super::player::Player;

const STICK_GATE: f32 = 0.05;
const MAX_NORMAL_SPEED: f32 = 17.0;
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
const RAD_PER_UNIT: f32 = TAU / 65536.0;

fn to_s16(radians: f32) -> i16 {
    (radians * UNITS_PER_RAD).round().rem_euclid(65536.0) as i32 as i16
}

fn from_s16(units: i16) -> f32 {
    units as f32 * RAD_PER_UNIT
}

fn angle_distance(a: i16, b: i16) -> u16 {
    a.wrapping_sub(b).unsigned_abs()
}

fn add_calc(value: &mut f32, target: f32, scale: f32, max_step: f32, min_step: f32) {
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

fn add_calc_angle_s(value: i16, target: i16, scale: i16, max_step: i16, min_step: i16) -> i16 {
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

fn set_normal_speed(player: &mut Player, desired: f32, accel: f32) {
    let (step_cap, decel_target) = if desired < player.speed {
        let span = player.speed - desired;
        (span.clamp(DECEL_MIN_STEP, DECEL_MAX_STEP), desired)
    } else {
        (DECEL_MAX_STEP, 0.0)
    };
    if accel.abs() < ACCEL_EPSILON {
        add_calc(
            &mut player.speed,
            decel_target,
            DECEL_SCALE,
            step_cap,
            DECEL_MIN_STEP,
        );
    } else {
        player.speed += accel;
        if player.speed > desired {
            player.speed = desired;
        }
    }
}

pub fn walk(
    intent: Res<MoveIntent>,
    camera: Res<CameraYaw>,
    mut players: Query<&mut Player>,
) {
    let stick = intent.axis.length().min(1.0);
    let stick_sq = stick * stick;
    let desired = MAX_NORMAL_SPEED * stick_sq;
    for mut player in &mut players {
        let player = &mut *player;
        let accel = if stick > STICK_GATE {
            let forward = Vec2::new(camera.0.sin(), camera.0.cos());
            let right = Vec2::new(-camera.0.cos(), camera.0.sin());
            let direction = forward * intent.axis.y + right * intent.axis.x;
            let target = to_s16(direction.x.atan2(direction.y));
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
        set_normal_speed(player, desired, accel);
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
}
