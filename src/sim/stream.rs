use std::f32::consts::TAU;

use glam::Vec3;

use super::walk;

pub const UNITS_PER_RAD: f32 = 65536.0 / TAU;

#[derive(Debug, Clone, Copy)]
pub struct Grade {
    pub normal: Option<Vec3>,
    pub friction: f32,
    pub no_grade_cos: f32,
    pub accel: Option<Vec3>,
}

impl Grade {
    pub fn level() -> Self {
        Self {
            normal: None,
            friction: 0.0,
            no_grade_cos: 0.0,
            accel: None,
        }
    }

    pub fn sloped(normal: Vec3, friction: f32, no_grade_cos: f32) -> Self {
        Self {
            normal: Some(normal),
            friction,
            no_grade_cos,
            accel: Some(Vec3::new(normal.x, 0.0, normal.z)),
        }
    }
}

pub fn angle_s(radians: f32) -> i16 {
    (radians * UNITS_PER_RAD) as i32 as i16
}

pub fn units_s(units: f32) -> i16 {
    angle_s(units * walk::RAD_PER_UNIT)
}

pub fn cos_s(units: i16) -> f32 {
    walk::from_s16(units).cos()
}

pub fn resist(speed: Vec3, stream: Vec3, damp: f32, drag: f32) -> Vec3 {
    let delta = speed - stream;
    -(delta * damp + delta * delta.abs() * drag)
}

pub fn craw(normal: Vec3, vector: Vec3) -> Vec3 {
    vector + normal * vector.dot(normal).abs()
}

fn pull(
    resist: Vec3,
    speed: Vec3,
    gravity: f32,
    grade: &Grade,
) -> Vec3 {
    let Some(normal) = grade.normal else {
        return Vec3::ZERO;
    };
    let mut accel = resist;
    accel.y += gravity;
    if let Some(extra) = grade.accel {
        accel += extra;
    }
    let mut out = Vec3::ZERO;
    if accel.dot(normal) < 0.0 {
        if normal.y <= grade.no_grade_cos {
            out += craw(normal, accel);
        }
        out -= craw(normal, speed) * grade.friction;
    }
    out
}

pub fn grade(
    speed: Vec3,
    stream: Vec3,
    heading: i16,
    speed_f: f32,
    gravity: f32,
    max_fall: f32,
    damp: f32,
    drag: f32,
    surface: &Grade,
) -> (Vec3, f32, i16) {
    let resist = resist(speed, stream, damp, drag);
    let held = pull(resist, speed, gravity, surface);
    let forward = Vec3::new(walk::from_s16(heading).sin(), 0.0, walk::from_s16(heading).cos());
    let mut next = Vec3::new(
        held.x + resist.x + speed_f * forward.x,
        held.y + resist.y + speed.y + gravity,
        held.z + resist.z + speed_f * forward.z,
    );
    if let Some(extra) = surface.accel {
        next += extra;
    }
    if next.y < max_fall {
        next.y = max_fall;
    }
    let magnitude = (next.x * next.x + next.z * next.z).sqrt();
    let facing = angle_s(next.x.atan2(next.z));
    (next, magnitude, facing)
}

pub fn move_stream(
    speed: Vec3,
    stream: Vec3,
    heading: i16,
    speed_f: f32,
    gravity: f32,
    max_fall: f32,
    damp: f32,
    drag: f32,
) -> (Vec3, f32, i16) {
    grade(
        speed,
        stream,
        heading,
        speed_f,
        gravity,
        max_fall,
        damp,
        drag,
        &Grade::level(),
    )
}

pub fn calc_speed(
    speed_f: f32,
    heading: i16,
    speed_y: f32,
    gravity: f32,
    max_fall: f32,
) -> (Vec3, f32) {
    let radians = walk::from_s16(heading);
    let mut next = Vec3::new(
        speed_f * radians.sin(),
        speed_y + gravity,
        speed_f * radians.cos(),
    );
    if next.y < max_fall {
        next.y = max_fall;
    }
    (next, speed_f)
}

