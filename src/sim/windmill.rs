use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::walk;

pub const WINDMILL_ACTORS: [&str; 2] = ["Hpu1", "Hpu2"];

pub const TYPE_TOWER: u8 = 0;

pub const CULL_MIN: [Vec3; 2] = [
    Vec3::new(-1400.0, 0.0, -1400.0),
    Vec3::new(-500.0, -500.0, -50.0),
];
pub const CULL_MAX: [Vec3; 2] = [
    Vec3::new(1400.0, 500.0, 1400.0),
    Vec3::new(500.0, 500.0, 150.0),
];
pub const CULL_FAR: f32 = 3.0;

pub const HUB_AXIS: f32 = 70.0;
pub const BLADE_TIP: f32 = 450.0;
pub const BLADE_RADIUS: f32 = 70.0;

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
pub const GATE_RADIUS: f32 = 170.0;

pub const SPOKE_NEAR: [Vec3; 4] = [
    Vec3::new(150.0, 0.0, HUB_AXIS),
    Vec3::new(-150.0, 0.0, HUB_AXIS),
    Vec3::new(0.0, 150.0, HUB_AXIS),
    Vec3::new(0.0, -150.0, HUB_AXIS),
];
pub const SPOKE_HUB: Vec3 = Vec3::new(0.0, 0.0, HUB_AXIS);
pub const SPOKE_FAR: [Vec3; 4] = [
    Vec3::new(350.0, 0.0, HUB_AXIS),
    Vec3::new(-350.0, 0.0, HUB_AXIS),
    Vec3::new(0.0, 350.0, HUB_AXIS),
    Vec3::new(0.0, -350.0, HUB_AXIS),
];
pub const SPOKE_NARROW: f32 = 60.0;
pub const SPOKE_WIDE: f32 = 200.0;
pub const SPOKE_COUNT: usize = 9;

pub const VANE_TOP: i16 = 2500;
pub const VANE_SCALE: i16 = 0xF;
pub const VANE_MAX_STEP: i16 = 100;
pub const VANE_MIN_STEP: i16 = 10;
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
    pub spin: i16,
    pub vane: i16,
    pub previous: i16,
}

impl Windmill {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: ((parameters & TYPE_MASK) % 2) as u8,
            spin: 0,
            vane: 0,
            previous: 0,
        }
    }

    pub fn tower(&self) -> bool {
        self.kind == TYPE_TOWER
    }

    pub fn armed(&self) -> bool {
        self.vane > ARMED
    }

    pub fn cull(&self) -> (Vec3, Vec3) {
        let index = usize::from(self.kind) % 2;
        (CULL_MIN[index], CULL_MAX[index])
    }

    pub fn vane_step(&mut self, strength: f32) {
        self.previous = self.vane;
        let target = (strength * f32::from(VANE_TOP)) as i16;
        self.vane = walk::add_calc_angle_s(
            self.vane,
            target,
            VANE_SCALE,
            VANE_MAX_STEP,
            VANE_MIN_STEP,
        );
    }

    pub fn whirring(&self) -> bool {
        self.previous <= self.vane && self.vane != 0
    }

    pub fn sound(&self) -> u32 {
        if self.tower() {
            SE_TOWER
        } else {
            SE_PINWHEEL
        }
    }

    pub fn turn(&mut self) -> i16 {
        self.spin = self.spin.wrapping_add(self.vane);
        self.spin
    }

    pub fn blades(&self) -> [Vec3; 4] {
        let reach = BLADE_TIP;
        [
            Vec3::new(reach, 0.0, HUB_AXIS),
            Vec3::new(-reach, 0.0, HUB_AXIS),
            Vec3::new(0.0, reach, HUB_AXIS),
            Vec3::new(0.0, -reach, HUB_AXIS),
        ]
        .map(|tip| self.spin_tip(tip))
    }

    pub fn gates(&self) -> ([Vec3; 4], [Vec3; 4]) {
        (
            GATE_IN.map(|gate| self.spin_tip(gate)),
            GATE_OUT.map(|gate| self.spin_tip(gate)),
        )
    }

    pub fn spokes(&self) -> [Vec3; SPOKE_COUNT] {
        let near = SPOKE_NEAR.map(|spoke| self.spin_tip(spoke));
        let far = SPOKE_FAR.map(|spoke| self.spin_tip(spoke));
        [
            near[0],
            near[1],
            near[2],
            near[3],
            self.spin_tip(SPOKE_HUB),
            far[0],
            far[1],
            far[2],
            far[3],
        ]
    }

    pub fn spoke_radii() -> [f32; SPOKE_COUNT] {
        [
            SPOKE_NARROW,
            SPOKE_NARROW,
            SPOKE_NARROW,
            SPOKE_NARROW,
            SPOKE_WIDE,
            SPOKE_WIDE,
            SPOKE_WIDE,
            SPOKE_WIDE,
            SPOKE_WIDE,
        ]
    }

    pub fn spin_tip(&self, local: Vec3) -> Vec3 {
        let yaw = walk::from_s16(self.spin);
        let (sin_yaw, cos_yaw) = yaw.sin_cos();
        if self.tower() {
            Vec3::new(
                local.x * cos_yaw + local.z * sin_yaw,
                local.y,
                local.z * cos_yaw - local.x * sin_yaw,
            )
        } else {
            Vec3::new(
                local.x * cos_yaw - local.y * sin_yaw,
                local.x * sin_yaw + local.y * cos_yaw,
                local.z,
            )
        }
    }

    pub fn hub(&self) -> Vec3 {
        self.spin_tip(SPOKE_HUB)
    }

    pub fn blast(&self) -> f32 {
        f32::from(self.vane) / f32::from(VANE_TOP) * HULL_RADIUS
    }
}

pub fn is_windmill(name: &str) -> bool {
    WINDMILL_ACTORS.contains(&name)
}

fn sweep(from: Vec3, to: Vec3, centre: Vec3, radius: f32) -> bool {
    let span = to - from;
    let length_sq = span.length_squared();
    if length_sq == 0.0 {
        return (centre - from).length_squared() <= radius * radius;
    }
    let along = ((centre - from).dot(span) / length_sq).clamp(0.0, 1.0);
    (from + span * along - centre).length_squared() <= radius * radius
}

pub fn striking(mill: &Windmill, base: Vec3, player: Vec3) -> bool {
    if mill.armed() {
        if mill.tower() {
            let (inner, outer) = mill.gates();
            inner
                .iter()
                .zip(outer.iter())
                .any(|(a, b)| sweep(*a, *b, player, GATE_RADIUS))
        } else {
            let hub = mill.hub();
            mill
                .blades()
                .iter()
                .any(|tip| sweep(hub, *tip, player, BLADE_RADIUS))
        }
    } else if mill.tower() {
        false
    } else {
        let radii = Windmill::spoke_radii();
        mill
            .spokes()
            .iter()
            .zip(radii.iter())
            .any(|(spoke, radius)| (*spoke - player).length_squared() <= radius * radius)
    }
}

pub fn control(
    wind: Res<Wind>,
    mut mills: Query<(&mut Actor, &mut Windmill)>,
    mut events: ResMut<SimEvents>,
) {
    for (mut actor, mut mill) in &mut mills {
        mill.vane_step(wind.strength);
        if mill.whirring() {
            events.list.push(SimEvent::Se(mill.sound()));
        }
        actor.angle[1] = mill.turn();
    }
}