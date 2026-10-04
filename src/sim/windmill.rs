use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::stream;
use super::walk;

pub const WINDMILL_ACTORS: [&str; 2] = ["Hpu1", "Hpu2"];

pub const TYPE_TOWER: u8 = 0;
pub const TYPE_PINWHEEL: u8 = 1;

pub const HEAP: [u32; 2] = [0x3A40, 0xA00];
pub const EXTENT_MIN: [[f32; 3]; 2] = [[-1400.0, 0.0, -1400.0], [-500.0, -500.0, -50.0]];
pub const EXTENT_MAX: [[f32; 3]; 2] = [[1400.0, 500.0, 1400.0], [500.0, 500.0, 150.0]];
pub const EXTENT_FAR: f32 = 3.0;

pub const POST_RADIUS: f32 = 250.0;
pub const POST_HEIGHT: f32 = 400.0;

pub const BLADE_TIP: f32 = 450.0;
pub const BLADE_AXIS: f32 = 70.0;
pub const BLADE_RADIUS: f32 = 70.0;
pub const GATE_SPAN: f32 = 1400.0;
pub const GATE_LIFT: f32 = 290.0;
pub const GATE_SIDE: f32 = 130.0;
pub const GATE_RADIUS: f32 = 200.0;
pub const GATE_SWEEP: f32 = 170.0;
pub const SPOKE_TIP: f32 = 350.0;
pub const SPOKE_NEAR: f32 = 150.0;
pub const SPOKE_RADIUS: f32 = 90.0;
pub const SPOKE_AT: u32 = 1 << 13;
pub const GATE_AT: u32 = 1 << 11;
pub const GATE_SPLASH: u32 = 1;

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
}

impl Windmill {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: (parameters & TYPE_MASK) as u8,
            angle: 0,
            rate: 0,
            previous: 0,
        }
    }

    pub fn tower(&self) -> bool {
        self.kind == TYPE_TOWER
    }

    pub fn spinning(&self) -> bool {
        self.rate > ARMED
    }

    pub fn idling(&self) -> bool {
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
        self.previous <= self.rate && self.rate != 0
    }

    pub fn spin(&mut self) -> i16 {
        self.angle = self.angle.wrapping_add(self.rate);
        self.angle
    }

    pub fn extent(&self) -> ([f32; 3], [f32; 3]) {
        let index = usize::from(self.kind) % EXTENT_MIN.len();
        (EXTENT_MIN[index], EXTENT_MAX[index])
    }

    pub fn hub(&self) -> Vec3 {
        self.place(Vec3::new(0.0, 0.0, BLADE_AXIS))
    }

    pub fn blades(&self) -> [Vec3; 4] {
        let span = BLADE_TIP;
        let hub = Vec3::new(0.0, 0.0, BLADE_AXIS);
        [
            Vec3::new(span, 0.0, BLADE_AXIS),
            Vec3::new(-span, 0.0, BLADE_AXIS),
            Vec3::new(0.0, span, BLADE_AXIS),
            Vec3::new(0.0, -span, BLADE_AXIS),
        ]
        .map(|tip| self.place(tip - hub))
    }

    pub fn gates(&self) -> ([Vec3; 4], [Vec3; 4]) {
        let inner = [
            Vec3::new(-GATE_SPAN, GATE_LIFT, GATE_SIDE),
            Vec3::new(-GATE_SPAN, GATE_LIFT, -GATE_SIDE),
            Vec3::new(GATE_SIDE, GATE_LIFT, -GATE_SPAN),
            Vec3::new(-GATE_SIDE, GATE_LIFT, -GATE_SPAN),
        ];
        let outer = [
            Vec3::new(GATE_SPAN, GATE_LIFT, GATE_SIDE),
            Vec3::new(GATE_SPAN, GATE_LIFT, -GATE_SIDE),
            Vec3::new(GATE_SIDE, GATE_LIFT, GATE_SPAN),
            Vec3::new(-GATE_SIDE, GATE_LIFT, GATE_SPAN),
        ];
        (
            inner.map(|gate| self.place(gate)),
            outer.map(|gate| self.place(gate)),
        )
    }

    pub fn spokes(&self) -> [Vec3; 9] {
        [
            Vec3::new(SPOKE_NEAR, 0.0, BLADE_AXIS),
            Vec3::new(-SPOKE_NEAR, 0.0, BLADE_AXIS),
            Vec3::new(0.0, SPOKE_NEAR, BLADE_AXIS),
            Vec3::new(0.0, -SPOKE_NEAR, BLADE_AXIS),
            Vec3::new(0.0, 0.0, BLADE_AXIS),
            Vec3::new(SPOKE_TIP, 0.0, BLADE_AXIS),
            Vec3::new(-SPOKE_TIP, 0.0, BLADE_AXIS),
            Vec3::new(0.0, SPOKE_TIP, BLADE_AXIS),
            Vec3::new(0.0, -SPOKE_TIP, BLADE_AXIS),
        ]
        .map(|spoke| self.place(spoke))
    }

    pub fn place(&self, local: Vec3) -> Vec3 {
        let spin = walk::from_s16(self.angle.wrapping_add(self.rate));
        let (sin_spin, cos_spin) = spin.sin_cos();
        if self.tower() {
            Vec3::new(
                local.x * cos_spin + local.y * sin_spin,
                -local.x * sin_spin + local.y * cos_spin,
                local.z,
            )
        } else {
            Vec3::new(
                local.x,
                local.y * cos_spin + local.z * sin_spin,
                -local.y * sin_spin + local.z * cos_spin,
            )
        }
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
