//! Ground locomotion: move intent → camera-relative target yaw → turn/speed rates.

use std::f32::consts::TAU;

use bevy_ecs::prelude::*;
use glam::{Vec2, Vec3};

use super::collision::Collision;
use super::combat::Combat;
use super::event::ItemPartner;
use super::events::{STRIDE_UNITS, SimEvent, SimEvents, Stride};
use super::ground;
use super::hop::{HOP_ACCEL, Hop};
use super::intent::{CameraYaw, MoveIntent};
use super::jump::{BACK_GRAVITY, BACK_RISE, BACK_SPEED, Jump, JumpKind, SIDE_GRAVITY, SIDE_OFFSET};
use super::link_proc::{Proc, WALK_ALLOWED, allows};
use super::lock::Lock;
use super::player::Player;
use super::roll::Roll;
use super::ship::Ship;
use super::swim::{
    BACKWARD_LIMIT, SWIM_DECEL_MAX, SWIM_DECEL_MIN, SWIM_DECEL_SCALE, SWIM_DRIVE, SWIM_MAX_SPEED,
    SWIM_TURN_MAX, SWIM_TURN_MIN, SWIM_TURN_SCALE, Swim,
};
use super::water::Water;

const STICK_GATE: f32 = 0.05;
pub(crate) const MAX_NORMAL_SPEED: f32 = 17.0;
const MOVE_ACCEL: f32 = 3.5;
const DECEL_SCALE: f32 = 0.6;
const DECEL_MAX_STEP: f32 = 2.5;
const DECEL_MIN_STEP: f32 = 1.8;
const ACCEL_EPSILON: f32 = 3.8146973e-6;
/// `mMove.m.field_0x6`, `main.cpp:2800`: the divisor every turn of the body runs on.
const TURN_SCALE: i16 = 5;
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

pub(crate) fn within(from: Vec3, to: Vec3, radius: f32) -> bool {
    (to - from).length_squared() <= radius * radius
}

pub(crate) fn swept(from: Vec3, to: Vec3, target: Vec3, radius: f32) -> bool {
    let travel = to - from;
    let length_squared = travel.length_squared();
    if length_squared < 1e-8 {
        return within(from, target, radius);
    }
    let along = ((target - from).dot(travel) / length_squared).clamp(0.0, 1.0);
    within(from + travel * along, target, radius)
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

pub(super) fn chase_f(value: &mut f32, target: f32, step: f32) -> bool {
    if step == 0.0 {
        let done = *value == target;
        if done {
            *value = target;
        }
        return done;
    }
    let step = if *value > target { -step } else { step };
    *value += step;
    if step * (*value - target) >= 0.0 {
        *value = target;
        true
    } else {
        false
    }
}

pub(super) fn chase_angle_s(value: &mut i16, target: i16, step: i16) -> bool {
    if step == 0 {
        let done = *value == target;
        if done {
            *value = target;
        }
        return done;
    }
    let step = if value.wrapping_sub(target) > 0 {
        -step
    } else {
        step
    };
    *value = value.wrapping_add(step);
    if i32::from(step) * i32::from(value.wrapping_sub(target)) >= 0 {
        *value = target;
        true
    } else {
        false
    }
}

fn set_normal_speed(
    player: &mut Player,
    desired: f32,
    drive: f32,
    scale: f32,
    min_step: f32,
    max_step: f32,
) {
    // `setNormalSpeedF`, `main.cpp:2325-2348`: a target under the current speed drops the drive and
    // runs the deceleration instead of the push.
    let (drive, step_cap, decel_target) = if desired < player.speed {
        let span = player.speed - desired;
        (0.0, span.clamp(min_step, max_step), desired)
    } else {
        (drive, max_step, 0.0)
    };
    if drive.abs() < ACCEL_EPSILON {
        add_calc(&mut player.speed, decel_target, scale, step_cap, min_step);
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

pub(super) fn turn_toward(value: i16, target: i16) -> i16 {
    add_calc_angle_s(value, target, TURN_SCALE, TURN_PARAM, TURN_RATE)
}

pub fn walk(
    intent: Res<MoveIntent>,
    camera: Res<CameraYaw>,
    swim: Res<Swim>,
    hop: Res<Hop>,
    roll: Res<Roll>,
    jump: Res<Jump>,
    combat: Res<Combat>,
    lock: Res<Lock>,
    mut partner: ResMut<ItemPartner>,
    collision: Option<Res<Collision>>,
    water: Res<Water>,
    proc: Option<Res<super::link_proc::LinkProc>>,
    ships: Query<&Ship>,
    mut players: Query<&mut Player>,
    mut stride: ResMut<Stride>,
    mut events: ResMut<SimEvents>,
) {
    if ships.iter().any(|ship| ship.riding) {
        return;
    }
    // `setSpeedAndAngleNormal`, `main.cpp:2833`: the model only turns onto the stick while the
    // player is not locked on and is not in the turn that drives the model itself.
    let model_turns = lock.target.is_none()
        && proc
            .as_ref()
            .map_or(true, |state| state.current != Proc::MoveTurn);
    for mut player in &mut players {
        let player = &mut *player;
        if !allows(proc.as_deref(), WALK_ALLOWED) {
            continue;
        }
        if partner.demo_fall {
            slowfall_tick(player, &mut partner, collision.as_deref(), &water);
        } else if partner.demo_backjump {
            backjump_tick(player, &mut partner, collision.as_deref(), &water);
        } else if partner.demo_stick.is_some() {
            demo_walk_tick(player, &partner, &mut stride, &mut events);
        } else if swim.active {
            swim_tick(player, &intent, &camera);
        } else if hop.active {
            hop_tick(player);
        } else if roll.active {
            roll_tick(player);
        } else if jump.active {
            jump_tick(player, &jump);
        } else if combat.active {
            swing_tick(player);
        } else {
            walk_tick(
                player,
                &intent,
                &camera,
                model_turns,
                &mut stride,
                &mut events,
            );
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
    // `procAutoJump`, `main.cpp:7151-7172`: the extra half-step off the rise belongs to the tick
    // that is still holding a grabbed item, and a plain ledge hop never takes it (`mProcVar0.m34D0`
    // is -1 from `main.cpp:7116`), so the rise only carries the gravity.
    player.speed_y = (player.speed_y + GRAVITY).max(MAX_FALL_SPEED);
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

fn swing_tick(player: &mut Player) {
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

fn walk_tick(
    player: &mut Player,
    intent: &MoveIntent,
    camera: &CameraYaw,
    model_turns: bool,
    stride: &mut Stride,
    events: &mut SimEvents,
) {
    let stick = intent.axis.length().min(1.0);
    locomote(
        player,
        target_yaw(intent, camera),
        stick,
        model_turns,
        stride,
        events,
    );
}

fn demo_walk_tick(
    player: &mut Player,
    partner: &ItemPartner,
    stride: &mut Stride,
    events: &mut SimEvents,
) {
    locomote(
        player,
        partner.move_angle,
        partner.demo_stick.unwrap_or(0.0),
        false,
        stride,
        events,
    );
}

fn backjump_tick(
    player: &mut Player,
    partner: &mut ItemPartner,
    collision: Option<&Collision>,
    water: &Water,
) {
    if partner.demo_landed {
        return;
    }
    if partner.demo_launch {
        partner.demo_launch = false;
        player.speed_y = BACK_RISE;
    }
    let heading = from_s16(partner.move_angle);
    player.speed = BACK_SPEED;
    player.pos.x += BACK_SPEED * heading.sin();
    player.pos.z += BACK_SPEED * heading.cos();
    let surface = match ground::surface(collision, player.pos) {
        Some(height) => Some(height),
        None if water.in_area => Some(water.surface),
        None => None,
    };
    player.speed_y = (player.speed_y + BACK_GRAVITY).max(MAX_FALL_SPEED);
    player.pos.y += player.speed_y;
    if let Some(surface) = surface
        && player.pos.y <= surface
    {
        player.pos.y = surface;
        player.speed_y = 0.0;
        player.speed = 0.0;
        partner.demo_landed = true;
    }
}

fn slowfall_tick(
    player: &mut Player,
    partner: &mut ItemPartner,
    collision: Option<&Collision>,
    water: &Water,
) {
    if partner.demo_landed {
        return;
    }
    let surface = match ground::surface(collision, player.pos) {
        Some(height) => Some(height),
        None if water.in_area => Some(water.surface),
        None => None,
    };
    if let Some(surface) = surface
        && player.pos.y - surface < 200.0
    {
        partner.fall_cap = (partner.fall_cap + 1.0).min(-5.0);
    }
    player.speed_y = (player.speed_y + GRAVITY).max(partner.fall_cap);
    player.pos.y += player.speed_y;
    if let Some(surface) = surface
        && player.pos.y <= surface
    {
        player.pos.y = surface;
        player.speed_y = 0.0;
        partner.demo_landed = true;
    }
}

fn locomote(
    player: &mut Player,
    target: i16,
    stick: f32,
    model_turns: bool,
    stride: &mut Stride,
    events: &mut SimEvents,
) {
    let stick_sq = stick * stick;
    let desired = MAX_NORMAL_SPEED * stick_sq;
    let accel = if stick > STICK_GATE {
        let yaw = to_s16(player.yaw);
        if angle_distance(target, yaw) > REVERSAL_LIMIT {
            player.yaw = from_s16(add_calc_angle_s(
                yaw,
                target,
                TURN_SCALE,
                REVERSAL_PARAM,
                REVERSAL_RATE,
            ));
            0.0
        } else {
            let max_step = ((TURN_PARAM as f32 * stick_sq) as i16).max(TURN_PARAM_FLOOR);
            let min_step = ((TURN_RATE as f32 * stick_sq) as i16).max(TURN_RATE_FLOOR);
            player.yaw = from_s16(add_calc_angle_s(
                yaw, target, TURN_SCALE, max_step, min_step,
            ));
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
    // `setSpeedAndAngleNormal`, `main.cpp:2833-2844`: the model turns onto the stick at twice the
    // body's step and snaps onto the body the tick it crosses it.
    if model_turns && stick > STICK_GATE {
        let body = to_s16(player.yaw);
        let before = to_s16(player.face).wrapping_sub(body);
        let turned = add_calc_angle_s(
            to_s16(player.face),
            target,
            TURN_SCALE,
            TURN_PARAM * 2,
            TURN_RATE * 2,
        );
        let after = turned.wrapping_sub(body);
        player.face = from_s16(if i32::from(before) * i32::from(after) <= 0 {
            body
        } else {
            turned
        });
    }
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
