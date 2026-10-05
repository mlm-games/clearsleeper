use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::ground;
use super::stream::{self, Grade};
use super::walk;

pub const BARREL_ACTOR: &str = "Ktarux";

pub const MODE_WAIT: u8 = 0;
pub const MODE_CARRY: u8 = 1;
pub const MODE_VIB0: u8 = 2;
pub const MODE_VIB1: u8 = 3;
pub const MODE_VIB2: u8 = 4;
pub const MODE_JUMP: u8 = 5;
pub const MODE_WALK: u8 = 6;

pub const LYING: i16 = 0x4000;
pub const TILT_THRESHOLD: i16 = 0x1000;
pub const TILT_STEP: i16 = 0xC00;
pub const CARRY_STEP: i16 = 0x1000;
pub const YAW_STEP: i16 = 0x600;
pub const CARRY_SETTLE: u8 = 15;

pub const RADIUS: f32 = 50.0;
pub const BODY_HEIGHT: f32 = 100.0;
pub const SHORT_RADIUS: f32 = 45.0;

pub const GRAVITY: f32 = -6.0;
pub const VIB0_GRAVITY: f32 = -14.0;
pub const VIB1_GRAVITY: f32 = -1.0;
pub const VIB2_GRAVITY: f32 = -12.0;
pub const VIB0_LIFT: f32 = 80.0;
pub const JUMP_LIFT: f32 = 30.0;
pub const MAX_FALL: f32 = -100.0;
pub const WEIGHT: u8 = 200;
pub const ROLLING_WEIGHT: u8 = 140;

pub const VIB_RATE: f32 = 16000.0;
pub const VIB_RATE_GAIN: f32 = 0.1;
pub const VIB_BIAS: f32 = 0.3;
pub const VIB_SLOPE: f32 = 8.544922e-05;
pub const VIB_SPIN: f32 = 0.9;
pub const VIB_PEAK: i16 = 0x800;
pub const VIB_SEED: i16 = 3500;
pub const VIB_RELAX: i16 = 150;
pub const GROUND_FACTOR: f32 = 0.8;
pub const VIB0_FRAMES: u8 = 5;
pub const VIB1_FRAMES: u8 = 12;
pub const VIB2_FRAMES: u8 = 20;

pub const GROUND_FRICTION: f32 = 0.1;
pub const GROUND_ANGLE: i16 = 0xA4F;
pub const SLOPE_EPSILON: f32 = 0.001;
pub const VISCOUS: f32 = 0.006;
pub const INERT: f32 = 0.001;
pub const MAX_MOVE: f32 = 30.0;
pub const MIN_MOVE_DIR: f32 = 5.0;
pub const MIN_MOVE_DIR_HALF: f32 = MIN_MOVE_DIR / 2.0;
pub const ROLL_RADIUS_GAIN: f32 = 5.0;
pub const ROLL_TURN: f32 = 6.28;
pub const ROLL_SPIN: f32 = 3.0;
pub const MOVE_DECAY: f32 = 0.95;
pub const MOVE_EPSILON: f32 = 0.1;

pub const FALL_DAMAGE: f32 = 200.0;
pub const CULL_SHIFT: u32 = 0x1C;
pub const CULL_MASK: u32 = 0x03;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Wait,
    Carry,
    Vib0,
    Vib1,
    Vib2,
    Jump,
    Walk,
}

impl Mode {
    pub fn code(self) -> u8 {
        match self {
            Self::Wait => MODE_WAIT,
            Self::Carry => MODE_CARRY,
            Self::Vib0 => MODE_VIB0,
            Self::Vib1 => MODE_VIB1,
            Self::Vib2 => MODE_VIB2,
            Self::Jump => MODE_JUMP,
            Self::Walk => MODE_WALK,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Barrel {
    pub mode: Mode,
    pub cull: u8,
    pub tilt: i16,
    pub yaw: i16,
    pub roll: i16,
    pub spin: i16,
    pub drift: i16,
    pub carry_tilt: i16,
    pub speed: Vec3,
    pub speed_f: f32,
    pub gravity: f32,
    pub drive: Vec3,
    pub timer: u8,
    pub weight: u8,
    pub last_ground: f32,
    pub broken: bool,
}

impl Barrel {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            mode: Mode::Wait,
            cull: ((parameters >> CULL_SHIFT) & CULL_MASK) as u8,
            tilt: 0,
            yaw: 0,
            roll: 0,
            spin: 0,
            drift: 0,
            carry_tilt: 0,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            gravity: GRAVITY,
            drive: Vec3::ZERO,
            timer: 0,
            weight: WEIGHT,
            last_ground: 0.0,
            broken: false,
        }
    }

    pub fn enter(&mut self, mode: Mode) {
        self.mode = mode;
        match mode {
            Mode::Wait => {
                self.gravity = GRAVITY;
                self.weight = WEIGHT;
            }
            Mode::Walk => self.gravity = GRAVITY,
            Mode::Carry => self.timer = CARRY_SETTLE,
            Mode::Vib0 => {
                self.gravity = VIB0_GRAVITY;
                self.speed.y = VIB0_LIFT;
                self.yaw = 0;
                self.tilt = 0;
                self.roll = VIB_SEED;
                self.spin = 0;
                self.drift = 0;
                self.timer = VIB0_FRAMES;
            }
            Mode::Vib1 => {
                self.gravity = VIB1_GRAVITY;
                self.timer = VIB1_FRAMES;
            }
            Mode::Vib2 => {
                self.gravity = VIB2_GRAVITY;
                self.timer = VIB2_FRAMES;
            }
            Mode::Jump => {
                self.gravity = GRAVITY;
                self.speed.y = JUMP_LIFT;
                self.yaw = 0;
                self.tilt = 0;
                self.roll = 0;
                self.spin = 0;
                self.drift = 0;
            }
        }
    }



    pub fn liftable(&self, grounded: bool) -> bool {
        !self.broken && grounded
    }

    pub fn pick_up(&mut self, tilt: i16) {
        self.carry_tilt = tilt;
        self.enter(Mode::Carry);
    }

    pub fn dropped(&mut self) {
        self.enter(Mode::Jump);
    }


    pub fn carry_step(&mut self, still_carried: bool) -> Option<Mode> {
        self.timer = self.timer.saturating_sub(1);
        let tilt = self.carry_tilt;
        walk::chase_angle_s(&mut self.tilt, tilt, CARRY_STEP);
        if still_carried {
            return None;
        }
        self.roll = 0;
        self.drift = 0;
        Some(Mode::Wait)
    }

    pub fn rest_step(&mut self, grounded: bool, sloped: bool) -> Option<Mode> {
        let target = if self.tilt > TILT_THRESHOLD { LYING } else { 0 };
        walk::chase_angle_s(&mut self.tilt, target, TILT_STEP);
        if !grounded {
            return None;
        }
        let rolling = self.tilt == LYING;
        let next = if rolling && sloped { Some(Mode::Walk) } else { None };
        self.weight = if rolling { ROLLING_WEIGHT } else { WEIGHT };
        next
    }

    pub fn integrate(&mut self, heading: i16) {
        let (speed, speed_f) = stream::calc_speed(
            self.speed_f,
            heading,
            self.speed.y,
            self.gravity,
            MAX_FALL,
        );
        self.speed = speed;
        self.speed_f = speed_f;
    }

    pub fn drag_step(&mut self, heading: i16, slope: Option<Vec3>) {
        let surface = match slope {
            Some(slope) => Grade::sloped(slope, GROUND_FRICTION, stream::cos_s(GROUND_ANGLE)),
            None => Grade::level(),
        };
        let (speed, speed_f, _) = stream::grade(
            self.speed,
            self.drive,
            heading,
            self.speed_f,
            self.gravity,
            MAX_FALL,
            VISCOUS,
            INERT,
            &surface,
        );
        self.speed = speed;
        self.speed_f = speed_f;
    }

    pub fn wobble(&mut self, grounded: bool) {
        let factor = if grounded { GROUND_FACTOR } else { 1.0 };
        let gain =
            ((self.roll) as f32 * VIB_SLOPE + VIB_BIAS) * VIB_RATE * VIB_RATE_GAIN * factor;
        self.yaw = self.yaw.wrapping_add(stream::units_s(gain));
        let spin = stream::units_s(VIB_RATE * VIB_SPIN * factor);
        self.spin = self.spin.wrapping_add(spin);
    }

    pub fn wobble_tick(&mut self) -> Option<Mode> {
        self.timer = self.timer.saturating_sub(1);
        if self.timer != 0 {
            return None;
        }
        match self.mode {
            Mode::Vib0 => Some(Mode::Vib1),
            Mode::Vib1 => Some(Mode::Vib2),
            _ => {
                self.tilt = 0;
                self.roll = 0;
                self.spin = 0;
                Some(Mode::Wait)
            }
        }
    }

    pub fn rock_step(&mut self) {
        let relax = VIB_RELAX;
        walk::chase_angle_s(&mut self.roll, 0, relax);
    }

    pub fn roll_step(&mut self, sloped: bool) -> Option<Mode> {
        let tilt = LYING;
        walk::chase_angle_s(&mut self.tilt, tilt, TILT_STEP);
        if sloped && self.speed_f * self.speed_f < GROUND_FRICTION {
            return Some(Mode::Wait);
        }
        None
    }

    pub fn face_step(&mut self, step: Vec3) -> i16 {
        let travel = step.x.hypot(step.z);
        let facing = if step.x == 0.0 && step.z == 0.0 {
            None
        } else {
            Some(stream::angle_s(step.x.atan2(step.z)))
        };
        if let Some(facing) = facing {
            let wrapped = if facing < 0 {
                facing.wrapping_add(0x8000)
            } else {
                facing
            };
            let gate = if self.mode == Mode::Wait {
                MIN_MOVE_DIR_HALF
            } else {
                MIN_MOVE_DIR
            };
            if travel > gate {
                walk::chase_angle_s(&mut self.yaw, wrapped, YAW_STEP);
            }
        }
        let circumference =
            (stream::cos_s(self.tilt) * ROLL_RADIUS_GAIN + SHORT_RADIUS) * ROLL_TURN;
        let arc = stream::units_s(travel / circumference * 65535.0);
        let spin = stream::units_s((arc) as f32 * ROLL_SPIN);
        if facing.is_some_and(|facing| facing < 0) {
            self.spin = self.spin.wrapping_add(spin);
            self.drift = self.drift.wrapping_add(arc);
        } else {
            self.spin = self.spin.wrapping_sub(spin);
            self.drift = self.drift.wrapping_sub(arc);
        }
        self.roll = stream::units_s((travel / MAX_MOVE).min(1.0) * (VIB_PEAK) as f32);
        facing.unwrap_or(self.yaw)
    }

    pub fn smashed(&mut self) {
        self.broken = true;
        self.enter(Mode::Wait);
    }

    pub fn smash_check(&mut self, grounded: bool, landed: bool, pos: f32) -> bool {
        if matches!(self.mode, Mode::Wait | Mode::Jump | Mode::Walk) {
            if landed && self.last_ground - pos > FALL_DAMAGE {
                self.smashed();
                return true;
            }
            if grounded {
                self.last_ground = pos;
            }
        }
        false
    }

    pub fn decay_drive(&mut self) {
        self.drive *= MOVE_DECAY;
        if self.drive.length_squared() < MOVE_EPSILON {
            self.drive = Vec3::ZERO;
        }
    }

}

pub fn is_barrel(name: &str) -> bool {
    name == BARREL_ACTOR
}

pub fn control(
    collision: Option<Res<Collision>>,
    mut barrels: Query<(Entity, &mut Actor, &mut Barrel)>,
    mut commands: Commands,
) {
    let solid = collision.as_deref();
    for (entity, mut actor, mut barrel) in &mut barrels {
        if barrel.broken {
            continue;
        }
        let before = actor.pos;
        let grounded = ground::grounded(solid, actor.pos);
        let slope = ground::slope_normal(solid, actor.pos);
        let tiltable = slope.x.abs() > SLOPE_EPSILON || slope.z.abs() > SLOPE_EPSILON;
        let heading = actor.angle[1];
        let next = match barrel.mode {
            Mode::Carry => continue,
            Mode::Vib0 | Mode::Vib1 => {
                barrel.integrate(heading);
                barrel.wobble(grounded);
                barrel.wobble_tick()
            }
            Mode::Vib2 => {
                barrel.rock_step();
                barrel.integrate(heading);
                barrel.wobble(grounded);
                barrel.wobble_tick()
            }
            Mode::Wait => {
                barrel.integrate(heading);
                barrel.rest_step(grounded, tiltable)
            }
            Mode::Jump => {
                if grounded {
                    Some(Mode::Walk)
                } else {
                    barrel.drag_step(heading, None);
                    let tilt = LYING;
                    walk::chase_angle_s(&mut barrel.tilt, tilt, TILT_STEP);
                    None
                }
            }
            Mode::Walk => {
                if grounded {
                    barrel.drag_step(heading, Some(slope));
                    barrel.roll_step(tiltable && slope.x.abs() >= SLOPE_EPSILON && slope.z.abs() >= SLOPE_EPSILON)
                } else {
                    barrel.drag_step(heading, None);
                    let tilt = LYING;
                    walk::chase_angle_s(&mut barrel.tilt, tilt, TILT_STEP);
                    None
                }
            }
        };
        actor.pos += barrel.speed;
        if barrel.smash_check(grounded, grounded, actor.pos.y) {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        if let Some(next) = next {
            barrel.enter(next);
        }
        if matches!(barrel.mode, Mode::Wait | Mode::Jump | Mode::Walk) {
            actor.angle[1] = barrel.face_step(actor.pos - before);
        }
        barrel.decay_drive();
    }
}

pub fn hull() -> (f32, f32) {
    (BODY_HEIGHT, RADIUS)
}

