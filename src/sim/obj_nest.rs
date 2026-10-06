use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};

use super::actor::Actor;
use super::stream::cos_s;
use super::walk::{from_s16, to_s16};

pub const OBJ_NEST_ACTORS: [&str; 1] = ["MtoriSU"];

pub fn is_obj_nest(name: &str) -> bool {
    OBJ_NEST_ACTORS.contains(&name)
}

pub const ARC: &str = "MtoriSU";
pub const MODEL_ATTR: u32 = 0x1100_0022;
pub const MOVE_BG_HEAP: u32 = 0xAA0;

pub const RIDE_SPEED_DIVISOR: f32 = 30.0;
pub const RIDE_TRIGGER_SQUARED: f32 = 100.0 * 100.0;
pub const RIDE_SPEED_CAP: f32 = 1.0;

pub const VIB_GAIN_X: f32 = 400.0;
pub const VIB_GAIN_Y: f32 = 400.0;
pub const VIB_GAIN_Z: f32 = 400.0;
pub const VIB_LIMIT_X: f32 = 50.0;
pub const VIB_LIMIT_Y: f32 = 100.0;
pub const VIB_LIMIT_Z: f32 = 50.0;
pub const VIB_PHASE: [i16; 3] = [0x4650, 0x4650, 0x4650];
pub const VIB_SETTLED: f32 = 100.0;
pub const VIB_DAMP: f32 = 0.92;

pub const CULL_Y: f32 = 20.0;
pub const CULL_RADIUS: f32 = 210.0;

pub fn axis_x(radians: f32) -> Quat {
    Quat::from_axis_angle(Vec3::X, radians)
}

pub fn axis_y(radians: f32) -> Quat {
    Quat::from_axis_angle(Vec3::Y, radians)
}

pub fn axis_z(radians: f32) -> Quat {
    Quat::from_axis_angle(Vec3::Z, radians)
}

pub fn zxy_rot(angle: [i16; 3]) -> Quat {
    axis_y(from_s16(angle[1])) * axis_x(from_s16(angle[0])) * axis_z(from_s16(angle[2]))
}

fn min_limit(value: f32, limit: f32) -> f32 {
    if value > limit { limit } else { value }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ObjNest {
    pub vib: Vec3,
    pub phase: [i16; 3],
    pub tilt: Vec3,
    pub model_rot: Quat,
    pub warp_rot: Quat,
    pub entered: bool,
}

impl Default for ObjNest {
    fn default() -> Self {
        Self {
            vib: Vec3::ZERO,
            phase: [0; 3],
            tilt: Vec3::ZERO,
            model_rot: Quat::IDENTITY,
            warp_rot: Quat::IDENTITY,
            entered: false,
        }
    }
}

impl ObjNest {
    pub fn spawn() -> Self {
        Self::default()
    }

    pub fn create_init(&mut self) {
        self.vib = Vec3::ZERO;
        self.phase = [0; 3];
        self.tilt = Vec3::ZERO;
        self.entered = true;
    }

    pub fn cull(&self) -> (f32, f32, f32) {
        (0.0, CULL_Y, CULL_RADIUS)
    }

    pub fn vib_set(&mut self, amount: f32) {
        self.vib.x = min_limit(self.vib.x + amount * VIB_GAIN_X, VIB_LIMIT_X);
        self.vib.y = min_limit(self.vib.y + amount * VIB_GAIN_Y, VIB_LIMIT_Y);
        self.vib.z = min_limit(self.vib.z + amount * VIB_GAIN_Z, VIB_LIMIT_Z);
    }

    pub fn ride_cb(&mut self, distance_squared: f32, speed_f: f32) {
        if distance_squared > RIDE_TRIGGER_SQUARED {
            let amount = min_limit(speed_f * (1.0 / RIDE_SPEED_DIVISOR), RIDE_SPEED_CAP);
            self.vib_set(amount);
        }
    }

    pub fn vib_proc(&mut self) {
        self.phase[0] = self.phase[0].wrapping_add(VIB_PHASE[0]);
        self.phase[1] = self.phase[1].wrapping_add(VIB_PHASE[1]);
        self.phase[2] = self.phase[2].wrapping_add(VIB_PHASE[2]);
        self.tilt = if self.vib.length() < VIB_SETTLED {
            Vec3::ZERO
        } else {
            Vec3::new(
                self.vib.x * cos_s(self.phase[0]),
                self.vib.y * cos_s(self.phase[1]),
                self.vib.z * cos_s(self.phase[2]),
            )
        };
        self.vib *= VIB_DAMP;
    }

    pub fn set_mtx(&mut self, angle: [i16; 3]) {
        let shape = zxy_rot(angle);
        let tilt = [to_s16(self.tilt.x), to_s16(self.tilt.y), to_s16(self.tilt.z)];
        self.warp_rot = shape;
        self.model_rot = shape * zxy_rot(tilt);
    }
}

pub fn control(mut nests: Query<(&Actor, &mut ObjNest)>) {
    for (actor, mut nest) in &mut nests {
        if !nest.entered {
            nest.create_init();
        }
        nest.vib_proc();
        nest.set_mtx(actor.angle);
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
    };
    let nest = ObjNest::spawn();
    world.spawn((actor, nest)).id()
}