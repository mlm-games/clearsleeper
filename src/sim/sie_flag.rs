use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::stream::{cos_s, sin_s};

pub const SIE_FLAG_ACTORS: [&str; 1] = ["SieFlag"];

pub fn is_sie_flag(name: &str) -> bool {
    SIE_FLAG_ACTORS.contains(&name)
}

pub const ARC: &str = "Eshata";
pub const CLOTH_ARC: &str = "Cloth";
pub const HEAP: u32 = 0x1020;

pub const FLY_GRID: i32 = 5;
pub const HOIST_GRID: i32 = 5;
pub const FLY_LENGTH: f32 = 700.0;
pub const HOIST_LENGTH: f32 = 350.0;

pub const FLAG_OFFSET: [f32; 3] = [0.0, 900.0, 0.0];
pub const WIND_OFFSET: [f32; 3] = [0.0, 725.0, 0.0];

pub const CLOTH_SPRING: f32 = 0.4;
pub const CLOTH_GRAVITY: f32 = -0.75;
pub const CLOTH_DRAG: f32 = 0.9;
pub const CLOTH_FLY_FLEX: f32 = 1.0;
pub const CLOTH_HOIST_FLEX: f32 = 1.0;
pub const CLOTH_WAVE: i16 = 0x400;
pub const CLOTH_EXTRA: i16 = 0;
pub const CLOTH_RIPPLE: i16 = 900;
pub const CLOTH_ROTATE: i16 = -800;
pub const CLOTH_WIND: f32 = 7.0;
pub const CLOTH_WIND_WAVE: f32 = 6.0;

pub const GLOBAL_WIND: f32 = 13.0;
pub const GLOBAL_WIND_WAVE: f32 = 8.0;

pub const WIND_SCALE: f32 = 0.1;
pub const WIND_MAX_STEP: f32 = 0.1;

pub const CYL_RADIUS: f32 = 50.0;
pub const CYL_HEIGHT: f32 = 1000.0;

pub const CULL_MIN: [f32; 3] = [-700.0, 0.0, -700.0];
pub const CULL_MAX: [f32; 3] = [700.0, 1100.0, 700.0];

pub fn rot_x(value: Vec3, angle: i16) -> Vec3 {
    if angle == 0 {
        return value;
    }
    let (sin_x, cos_x) = sin_cos(angle);
    Vec3::new(value.x, value.y * cos_x - value.z * sin_x, value.y * sin_x + value.z * cos_x)
}

pub fn rot_y(value: Vec3, angle: i16) -> Vec3 {
    if angle == 0 {
        return value;
    }
    let (sin_y, cos_y) = sin_cos(angle);
    Vec3::new(value.x * cos_y + value.z * sin_y, value.y, -value.x * sin_y + value.z * cos_y)
}

pub fn rot_z(value: Vec3, angle: i16) -> Vec3 {
    if angle == 0 {
        return value;
    }
    let (sin_z, cos_z) = sin_cos(angle);
    Vec3::new(value.x * cos_z - value.y * sin_z, value.x * sin_z + value.y * cos_z, value.z)
}

pub fn zxy_rot_m(value: Vec3, angle: [i16; 3]) -> Vec3 {
    rot_y(rot_x(rot_z(value, angle[2]), angle[0]), angle[1])
}

fn sin_cos(angle: i16) -> (f32, f32) {
    (sin_s(angle), cos_s(angle))
}

pub fn add_calc_pos2(value: &mut Vec3, target: Vec3, scale: f32, max_step: f32) {
    if *value == target {
        return;
    }
    let mut diff = (*value - target) * scale;
    if diff.length() > max_step {
        let flat = Vec3::new(diff.x, 0.0, diff.z);
        let len = flat.length();
        if len != 0.0 {
            diff = flat * (max_step / len);
        } else {
            diff = Vec3::new(0.0, diff.y, 0.0);
        }
    }
    *value -= diff;
}

#[derive(Component, Debug, Clone, Copy)]
pub struct SieFlag {
    pub wind: Vec3,
    pub cloth: Vec3,
    pub entered: bool,
}

impl SieFlag {
    pub fn spawn() -> Self {
        Self {
            wind: Vec3::ZERO,
            cloth: Vec3::ZERO,
            entered: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn cylinder(&self) -> (f32, f32) {
        (CYL_RADIUS, CYL_HEIGHT)
    }

    pub fn set_mtx(&mut self, pos: Vec3, angle: [i16; 3]) {
        self.cloth = pos + zxy_rot_m(Vec3::from_array(FLAG_OFFSET), angle);
    }

    pub fn wind_point(&self, pos: Vec3) -> Vec3 {
        pos + Vec3::from_array(WIND_OFFSET)
    }

    pub fn wind_pull(&mut self, all_wind: Vec3) {
        if all_wind.length() > self.wind.length() {
            self.wind = all_wind;
        } else {
            add_calc_pos2(&mut self.wind, all_wind, WIND_SCALE, WIND_MAX_STEP);
        }
    }
}

pub fn control(mut flags: Query<(&Actor, &mut SieFlag)>) {
    for (actor, mut flag) in &mut flags {
        flag.set_mtx(actor.pos, actor.angle);
    }
}

pub fn spawn(world: &mut World, source: &crate::dzs::StageActor) -> Entity {
    let actor = Actor {
        name: source.name.clone(),
        parameters: source.parameters,
        pos: glam::Vec3::from_array(source.pos),
        angle: source.angle,
        set_id: source.set_id,
        room: source.room,
        ..Default::default()
    };
    let mut flag = SieFlag::spawn();
    flag.set_mtx(actor.pos, actor.angle);
    world.spawn((actor, flag)).id()
}