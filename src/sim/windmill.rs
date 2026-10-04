use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::stream;
use super::walk;

pub const WINDMILL_ACTORS: [&str; 2] = ["Hpu1", "Hpu2"];

pub const TYPE_TOWER: u8 = 0;
pub const TYPE_PINWHEEL: u8 = 1;

pub const HEAP: [u32; 2] = [0x3A40, 0xA00];
pub const CULL_MIN: [[f32; 3]; 2] = [[-1400.0, 0.0, -1400.0], [-500.0, -500.0, -50.0]];
pub const CULL_MAX: [[f32; 3]; 2] = [[1400.0, 500.0, 1400.0], [500.0, 500.0, 150.0]];
pub const CULL_FAR: f32 = 3.0;

pub const HUB: Vec3 = Vec3::new(0.0, 0.0, 70.0);
pub const BLADE: [Vec3; 4] = [
    Vec3::new(450.0, 0.0, 70.0),
    Vec3::new(-450.0, 0.0, 70.0),
    Vec3::new(0.0, 450.0, 70.0),
    Vec3::new(0.0, -450.0, 70.0),
];
pub const GATE_IN: [Vec3; 4] = [
    Vec3::new(-1400.0, 290.0, 130.0),
    Vec3::new(-1400.0, 290.0, -130.0),
    Vec3::new(130.0, 290.0, -1400.0),
    Vec3::new(-130.0, 290.0, -1400.0),
];
pub const GATE_OUT: [Vec3; 4] = [
    Vec3::new(1400.0, 290.0, 130.0),
    Vec3::new(1400.0, 290.0, -130.0),
    Vec3::new(130.0, 290.0, 1400.0),
    Vec3::new(-130.0, 290.0, 1400.0),
];
pub const SPOKE: [Vec3; 9] = [
    Vec3::new(150.0, 0.0, 70.0),
    Vec3::new(-150.0, 0.0, 70.0),
    Vec3::new(0.0, 150.0, 70.0),
    Vec3::new(0.0, -150.0, 70.0),
    Vec3::new(0.0, 0.0, 70.0),
    Vec3::new(350.0, 0.0, 70.0),
    Vec3::new(-350.0, 0.0, 70.0),
    Vec3::new(0.0, 350.0, 70.0),
    Vec3::new(0.0, -350.0, 70.0),
];

pub const SPOKE_RADII: [f32; 9] = [60.0, 60.0, 60.0, 60.0, 90.0, 90.0, 90.0, 90.0, 90.0];
pub const SPOKE_AT: u32 = 1 << 13;
pub const CAPSULE_RADIUS: f32 = 200.0;
pub const CAPSULE_HIT_RADIUS: f32 = 70.0;
pub const CAPSULE_AT: u32 = 1 << 11;
pub const CAPSULE_SPLASH: u32 = 1;
pub const CAPSULE_SPLASH_RADIUS: f32 = 170.0;
pub const POST_RADIUS: f32 = 250.0;
pub const POST_HEIGHT: f32 = 400.0;

pub const SWING_TOP: i16 = 2500;
pub const SWING_SCALE: i16 = 0xF;
pub const SWING_MAX_STEP: i16 = 100;
pub const SWING_MIN_STEP: i16 = 10;
pub const ARMED: i16 = 1000;

pub const SE_TOWER: u32 = 0x7043;
pub const SE_PINWHEEL: u32 = 0x7044;

const TYPE_MASK: u32 = 0xF;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Wind {
    pub strength: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Windmill {
    pub kind: u8,
    pub angle: i16,
    pub rate: i16,
    pub previous: i16,
    pub gusting: bool,
}

impl Windmill {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: (parameters & TYPE_MASK) as u8,
            angle: 0,
            rate: 0,
            previous: 0,
            gusting: false,
        }
    }

    pub fn barred(&self) -> bool {
        self.kind == TYPE_TOWER
    }

    pub fn spinning(&self) -> bool {
        self.rate > ARMED
    }

    pub fn stopped(&self) -> bool {
        self.rate <= ARMED
    }

    pub fn vane_step(&mut self, wind: f32) -> bool {
        self.previous = self.rate;
        let target = stream::units_s(wind * f32::from(SWING_TOP));
        self.rate = walk::add_calc_angle_s(
            self.rate,
            target,
            SWING_SCALE,
            SWING_MAX_STEP,
            SWING_MIN_STEP,
        );
        self.angle = self.angle.wrapping_add(self.rate);
        self.gusting = self.previous <= self.rate && self.rate != 0;
        self.gusting
    }

    pub fn spin(&self) -> i16 {
        self.angle.wrapping_add(self.rate)
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        let index = usize::from(self.kind) % CULL_MIN.len();
        (CULL_MIN[index], CULL_MAX[index])
    }

    pub fn hull(&self) -> (f32, f32) {
        (POST_HEIGHT, POST_RADIUS)
    }
}

pub fn is_windmill(name: &str) -> bool {
    WINDMILL_ACTORS.contains(&name)
}

pub fn control(wind: Res<Wind>, mut mills: Query<(&mut Actor, &mut Windmill)>) {
    for (mut actor, mut mill) in &mut mills {
        mill.vane_step(wind.strength);
        actor.angle[1] = mill.spin();
    }
}

pub fn heap(kind: u8) -> u32 {
    HEAP[usize::from(kind) % HEAP.len()]
}

pub fn sound(kind: u8) -> u32 {
    if kind == TYPE_TOWER {
        SE_TOWER
    } else {
        SE_PINWHEEL
    }
}
