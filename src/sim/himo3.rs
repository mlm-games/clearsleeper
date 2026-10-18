use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::{Actor, atan2s};
use super::area::StageName;
use super::bridge::BridgeWeather;
use super::collision::Collision;
use super::flags::EventFlags;
use super::player::Player;
use super::stream::{add_calc_angle_s2, sin_s};
use super::walk::{self, from_s16};

pub const HIMO3_ACTORS: [&str; 1] = ["RopeR"];

pub fn is_himo3(name: &str) -> bool {
    HIMO3_ACTORS.contains(&name)
}

pub const PRM_TYPE_SHIFT: u32 = 0;
pub const PRM_TYPE_MASK: u32 = 0xFF;
pub const PRM_NODE_SHIFT: u32 = 8;
pub const PRM_NODE_MASK: u32 = 0xFF;
pub const PRM_SPEED_SHIFT: u32 = 16;
pub const PRM_SPEED_MASK: u32 = 0xFF;
pub const PRM_PATH_SHIFT: u32 = 24;
pub const PRM_PATH_MASK: u32 = 0xFF;

pub const TYPE_TORCH: u8 = 0x00;
pub const TYPE_HOOK: u8 = 0x01;
pub const TYPE_BGN: u8 = 0x0F;
pub const PATH_ARG_NONE: u8 = 0xFF;

pub const PARAM_HANG_PLAYER: u32 = 1;
pub const PARAM_HANG_POINT: u32 = 2;
pub const PARAM_RELEASE: u32 = 3;

pub const NODE_MAX: usize = 200;
pub const NODE_LIMIT: i32 = 0xC8;

pub const SCALE_NORMAL: f32 = 20.0;
pub const SCALE_BGN: f32 = 22.0;
pub const MOVE_LENGTH_TRIM: f32 = 50.0;
pub const TORCH_LENGTH_TRIM: f32 = 50.0;

pub const HANG_Y_SENTINEL: f32 = -23535.0;
pub const HANG_HIDE_FRAMES: i16 = 0x1E;

pub const HANG_ANGLE_STEP: i16 = 2;
pub const HANG_ANGLE_MAX: i16 = 0x2000;
pub const HANG_TILT: f32 = -10.0;
pub const HANG_TILT_STEP: f32 = 10.0;
pub const HANG_SWAY_SCALE: f32 = 0.1;
pub const HANG_GAIN_STEP: f32 = 0.05;
pub const HANG_GAIN_IDLE: f32 = 0.1;
pub const HANG_GAIN_TEST: f32 = 1.0;

pub const SWAY_DAMP: f32 = -0.003;
pub const SWAY_DAMP_SCALE: f32 = 0.05;
pub const SWAY_DAMP_MAX: f32 = 1.0;

pub const WIND_HELD_SCALE: f32 = 0.5;
pub const WIND_HELD_STEP: f32 = 0.05;
pub const WIND_FREE_STEP: f32 = 0.001;

pub const SWAY_GATE: f32 = 0.01;
pub const SWAY_CHASE: f32 = 0.2;
pub const SWAY_CHASE_STEP: f32 = 0.05;
pub const SWAY_DECAY: f32 = 0.005;
pub const SWAY_FREQ: i32 = 500;
pub const SWAY_AMPLITUDE: f32 = 100.0;

pub const NODE_GRAVITY: f32 = -200.0;
pub const NODE_SPHERE_STRIDE: i32 = 3;
pub const NODE_SPHERE_MOD: i32 = 0x0F;
pub const NODE_SPHERE_MAX: usize = 5;
pub const NODE_SPHERE_LIVE_RADIUS: f32 = 30.0;
pub const NODE_SPHERE_HANG_RADIUS: f32 = -200.0;

pub const LINE_FADE_READY: f32 = 0.999;
pub const LINE_FADE_RATE: f32 = 100.0;
pub const LINE_FADE_STEP: f32 = 0.05;

pub const HOOK_TILT: i16 = -0x4000;

pub const CYL_SRC_RADIUS: f32 = 50.0;
pub const CYL_SRC_HEIGHT: f32 = 4000.0;
pub const NODE_SPH_SRC_RADIUS: f32 = 50.0;
pub const HEAD_SPH_SRC_RADIUS: f32 = 80.0;
pub const HEAD_SPH_TYPE1_RADIUS: f32 = 20.0;

pub const TORCH_OFFSET: [f32; 3] = [10.0, -140.0, -15.0];
pub const HOOK_OFFSET: [f32; 3] = [0.0, 0.0, 35.0];

pub const TORCH_WALL_HALF: f32 = 40.0;
pub const TORCH_WALL_RADIUS: f32 = 50.0;
pub const HOOK_WALL_HALF: f32 = 20.0;
pub const HOOK_WALL_RADIUS: f32 = 20.0;
pub const TORCH_SHADOW: f32 = 20.0;
pub const HOOK_SHADOW: f32 = 10.0;

pub const TORCH_TIMER0_SPAN: f32 = 10.0;
pub const TORCH_TIMER0_MIN: f32 = 5.0;
pub const TORCH_TIMER1_SPAN: f32 = 6.0;
pub const TORCH_TIMER1_MIN: f32 = 3.0;
pub const TORCH_ALPHA_SPAN: f32 = 16.0;
pub const TORCH_ALPHA_MIN: f32 = 4.0;
pub const TORCH_ALPHA_STEP: f32 = 0.1;
pub const TORCH_FLICKER_SPAN: f32 = 0.2;
pub const TORCH_FLICKER_SCALE: f32 = 0.5;
pub const TORCH_FLICKER_STEP: f32 = 0.02;

pub const GA_TRAVEL: f32 = 10.0;
pub const GA_TURN_STEP: i16 = 2;
pub const GA_TURN_MAX: i16 = 0x1000;
pub const GA_PHASE_STEP: i16 = 0x3E00;
pub const GA_TIMER_SPAN: f32 = 10.0;
pub const GA_JITTER_XZ: f32 = 150.0;
pub const GA_JITTER_Y: f32 = 100.0;
pub const GA_SCALE_SPAN: f32 = 0.3;
pub const GA_SCALE_MIN: f32 = 0.3;
pub const GA_PHASE_SPAN: f32 = 30000.0;

pub const PATH_ARG_SCALE: f32 = 0.1;
pub const PATH_ARRIVE_SCALE: f32 = 2.0;
pub const PATH_STOP_SCALE: f32 = 1.0;
pub const PATH_NEXT_NONE: u16 = 0xFFFF;

pub const EVENT_HANG_MAJROOM: u32 = 0x0402;
pub const MAJROOM_STAGES: [&str; 3] = ["majroom", "Majroom", "MajyuE"];

pub const AT_TYPE_BOOMERANG: u32 = 1 << 6;
pub const AT_TYPE_WATER: u32 = 1 << 8;
pub const AT_TYPE_UNK20000: u32 = 1 << 17;
pub const AT_TYPE_WIND: u32 = 1 << 21;
pub const AT_TYPE_UNK400000: u32 = 1 << 22;
pub const AT_TYPE_LIGHT: u32 = 1 << 23;
pub const AT_TYPE_ALL: u32 = !0u32;

pub const TG_NODE_ALL: u32 = AT_TYPE_ALL
    & !AT_TYPE_BOOMERANG
    & !AT_TYPE_WATER
    & !AT_TYPE_UNK20000
    & !AT_TYPE_WIND
    & !AT_TYPE_UNK400000
    & !AT_TYPE_LIGHT;
pub const TG_HEAD_ALL: u32 = AT_TYPE_ALL
    & !AT_TYPE_BOOMERANG
    & !AT_TYPE_WATER
    & !AT_TYPE_UNK20000
    & !AT_TYPE_UNK400000
    & !AT_TYPE_LIGHT;

const REG0_F1: f32 = 0.0;
const REG0_F2: f32 = 0.0;
const REG0_F3: f32 = 0.0;
const REG0_F4: f32 = 0.0;
const REG0_F5: f32 = 0.0;
const REG0_F6: f32 = 0.0;
const REG0_F7: f32 = 0.0;
const REG0_F8: f32 = 0.0;
const REG0_F9: f32 = 0.0;
const REG0_S0: i32 = 0;
const REG0_S5: i32 = 0;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];

const HIO_NO: i8 = 0;
const HIO_M06: i16 = 0;
const HIO_M08: f32 = 0.75;
const HIO_M0C: f32 = 0.5;
const HIO_M10: u32 = 0xEB;
const HIO_M12: u32 = 0x7D;
const HIO_M14: u32 = 0;
const HIO_M18: f32 = 5.0;

const CULL_FALLBACK: f32 = 100.0;

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Himo3Hio {
    pub no: i8,
    pub m06: i16,
    pub m08: f32,
    pub m0c: f32,
    pub m10: u32,
    pub m12: u32,
    pub m14: u32,
    pub m18: f32,
}

impl Default for Himo3Hio {
    fn default() -> Self {
        Self {
            no: HIO_NO,
            m06: HIO_M06,
            m08: HIO_M08,
            m0c: HIO_M0C,
            m10: HIO_M10,
            m12: HIO_M12,
            m14: HIO_M14,
            m18: HIO_M18,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Himo3Statics {
    pub hio: Himo3Hio,
    pub hio_set: bool,
    pub scale: f32,
}

impl Default for Himo3Statics {
    fn default() -> Self {
        Self {
            hio: Himo3Hio::default(),
            hio_set: false,
            scale: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Himo3Point {
    pub arg3: u8,
    pub pos: Vec3,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Himo3Path {
    pub id: u16,
    pub room: i8,
    pub next_id: u16,
    pub closed: bool,
    pub points: Vec<Himo3Point>,
}

#[derive(Resource, Debug, Default)]
pub struct Himo3Paths {
    pub paths: Vec<Himo3Path>,
}

impl Himo3Paths {
    pub fn get(&self, id: u16, room: i8) -> Option<&Himo3Path> {
        self.paths
            .iter()
            .find(|path| path.id == id && path.room == room)
    }
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub struct Himo3WindTouch {
    pub touching: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Himo3Sph {
    pub centre: Vec3,
    pub radius: f32,
    pub registered: bool,
    pub at_type: u32,
    pub atp: i32,
    pub tg_type: u32,
    pub tg_set: bool,
    pub tg_is_enemy: bool,
    pub tg_no_con_hit: bool,
    pub tg_shield: bool,
    pub co_set: bool,
    pub co_player: bool,
    pub co_all: bool,
}

impl Himo3Sph {
    pub fn blocks(&self, point: Vec3) -> bool {
        self.registered
            && self.co_set
            && self.co_player
            && walk::within(self.centre, point, self.radius)
    }

    pub fn is_target(&self, point: Vec3) -> bool {
        self.registered && self.tg_set && walk::within(self.centre, point, self.radius)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Himo3Cyl {
    pub centre: Vec3,
    pub radius: f32,
    pub height: f32,
    pub registered: bool,
    pub at_type: u32,
    pub atp: i32,
    pub tg_type: u32,
    pub tg_set: bool,
    pub tg_is_enemy: bool,
    pub co_set: bool,
    pub co_player: bool,
    pub co_all: bool,
}

impl Default for Himo3Cyl {
    fn default() -> Self {
        Self {
            centre: Vec3::ZERO,
            radius: CYL_SRC_RADIUS,
            height: CYL_SRC_HEIGHT,
            registered: false,
            at_type: 0,
            atp: 0,
            tg_type: TG_HEAD_ALL,
            tg_set: true,
            tg_is_enemy: true,
            co_set: true,
            co_player: true,
            co_all: true,
        }
    }
}

impl Himo3Cyl {
    pub fn blocks(&self, point: Vec3) -> bool {
        let flat = point - self.centre;
        self.registered
            && self.co_set
            && self.co_player
            && (flat.y).abs() <= self.height * 0.5
            && flat.x * flat.x + flat.z * flat.z <= self.radius * self.radius
    }

    pub fn is_target(&self, point: Vec3) -> bool {
        self.registered && self.tg_set && walk::within(self.centre, point, self.radius)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Himo3ModelBase {
    pub set: bool,
    pub pos: Vec3,
    pub pitch: i16,
    pub yaw: i16,
    pub tilted: bool,
    pub scale: f32,
}

impl Default for Himo3ModelBase {
    fn default() -> Self {
        Self {
            set: false,
            pos: Vec3::ZERO,
            pitch: 0,
            yaw: 0,
            tilted: false,
            scale: HIO_M08,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Himo3 {
    pub m0298: u8,
    pub m0299: u8,
    pub m029a: u8,
    pub m029b: i8,
    pub m029c: i8,
    pub m029d: i8,
    pub m02a4: u8,
    pub m02a8: Vec3,
    pub m02b4: [i16; 3],
    pub m02ba: i16,
    pub m02bc: i16,
    pub m02be: u8,
    pub node: [[Vec3; 2]; NODE_MAX],
    pub line_pos: [Vec3; NODE_MAX],
    pub m15c0: i32,
    pub m15c4: Vec3,
    pub m15d0: f32,
    pub m15d4: f32,
    pub m15d8: f32,
    pub m15dc: f32,
    pub m15e0: f32,
    pub m15e4: f32,
    pub m15e8: f32,
    pub m15f0: f32,
    pub m15f4: f32,
    pub m15f8: i16,
    pub m15fa: i16,
    pub m15fc: f32,
    pub m1620: f32,
    pub m1624: Vec3,
    pub m1630: Vec3,
    pub m1870: f32,
    pub m1878: f32,
    pub m187c: f32,
    pub m20f8: f32,
    pub fire_lit: bool,
    pub m2110: u8,
    pub m2111: u8,
    pub ga_pos: Vec3,
    pub ga_target: Vec3,
    pub ga_angle: [i16; 2],
    pub ga_scale: f32,
    pub ga_bob: f32,
    pub ga_phase: i16,
    pub ga_active: u8,
    pub ga_timer: u8,
    pub m21f4: Vec3,
    pub m2200: i16,
    pub path_id: u16,
    pub path_room: i8,
    pub model: Himo3ModelBase,
    pub cyl: Himo3Cyl,
    pub sphs: [Himo3Sph; NODE_SPHERE_MAX],
    pub sphere_count: usize,
    pub sph: Himo3Sph,
    pub health: i32,
    pub max_health: i32,
    pub seed: [i32; 3],
    pub created: bool,
    pub majroom_event: bool,
}

impl Himo3 {
    pub fn spawn(parameters: u32) -> Self {
        let raw = ((parameters >> PRM_TYPE_SHIFT) & PRM_TYPE_MASK) as u8;
        let m0298 = if raw == 0xFF { 0 } else { raw };
        let m0299 = ((parameters >> PRM_SPEED_SHIFT) & PRM_SPEED_MASK) as u8;
        let m029a = ((parameters >> PRM_PATH_SHIFT) & PRM_PATH_MASK) as u8;
        let mut m15c0 = ((parameters >> PRM_NODE_SHIFT) & PRM_NODE_MASK) as i32;
        let mut scale = SCALE_NORMAL;
        let mut m029a = m029a;
        if m0298 == TYPE_BGN {
            scale = SCALE_BGN + REG0_F4;
            m15c0 = REG0_S0 + 200;
            m029a = 0xFF;
        }
        let mut m15fc = m15c0 as f32 * scale - MOVE_LENGTH_TRIM;
        if m0298 == TYPE_TORCH {
            m15fc -= TORCH_LENGTH_TRIM;
        }
        Self {
            m0298,
            m0299,
            m029a,
            m029b: 0,
            m029c: 0,
            m029d: 0,
            m02a4: 0,
            m02a8: Vec3::ZERO,
            m02b4: [0; 3],
            m02ba: 0,
            m02bc: 0,
            m02be: 0,
            node: [[Vec3::ZERO; 2]; NODE_MAX],
            line_pos: [Vec3::ZERO; NODE_MAX],
            m15c0,
            m15c4: Vec3::ZERO,
            m15d0: 0.0,
            m15d4: 0.0,
            m15d8: 0.0,
            m15dc: 0.0,
            m15e0: 0.0,
            m15e4: 0.0,
            m15e8: 0.0,
            m15f0: 0.0,
            m15f4: 0.0,
            m15f8: 0,
            m15fa: 0,
            m15fc,
            m1620: 0.0,
            m1624: Vec3::ZERO,
            m1630: Vec3::ZERO,
            m1870: 0.0,
            m1878: 0.0,
            m187c: 0.0,
            m20f8: 0.0,
            fire_lit: false,
            m2110: 0,
            m2111: 0,
            ga_pos: Vec3::ZERO,
            ga_target: Vec3::ZERO,
            ga_angle: [0; 2],
            ga_scale: 0.0,
            ga_bob: 0.0,
            ga_phase: 0,
            ga_active: 0,
            ga_timer: 0,
            m21f4: Vec3::ZERO,
            m2200: 0,
            path_id: 0,
            path_room: 0,
            model: Himo3ModelBase::default(),
            cyl: Himo3Cyl::default(),
            sphs: [Himo3Sph::default(); NODE_SPHERE_MAX],
            sphere_count: 0,
            sph: Himo3Sph::default(),
            health: 0,
            max_health: 0,
            seed: [0; 3],
            created: false,
            majroom_event: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        let count = self.node_count();
        if count == 0 {
            let p = self.line_pos[0];
            return (
                [
                    p.x - CULL_FALLBACK,
                    p.y - CULL_FALLBACK,
                    p.z - CULL_FALLBACK,
                ],
                [
                    p.x + CULL_FALLBACK,
                    p.y + CULL_FALLBACK,
                    p.z + CULL_FALLBACK,
                ],
            );
        }
        let mut min = self.line_pos[0];
        let mut max = self.line_pos[0];
        for index in 0..count {
            let p = self.line_pos[index];
            min = min.min(p);
            max = max.max(p);
        }
        ([min.x, min.y, min.z], [max.x, max.y, max.z])
    }

    pub fn drawn(&self) -> bool {
        true
    }

    pub fn node_count(&self) -> usize {
        if self.m15c0 <= 0 {
            return 0;
        }
        (self.m15c0 as usize).min(NODE_MAX)
    }

    pub fn get_player_move_length(&self) -> f32 {
        self.m15fc
    }

    pub fn set_actor_hang(&mut self, pos: Vec3, angle: i16) {
        self.m21f4 = pos;
        self.m2200 = angle;
    }

    pub fn rope_cylinder(&self) -> &Himo3Cyl {
        &self.cyl
    }

    pub fn head_sphere(&self) -> &Himo3Sph {
        &self.sph
    }

    pub fn node_sphere(&self, index: usize) -> Option<&Himo3Sph> {
        if index < self.sphere_count {
            self.sphs.get(index)
        } else {
            None
        }
    }

    pub fn blocks(&self, point: Vec3) -> bool {
        if self.cyl.blocks(point) {
            return true;
        }
        if self.sph.blocks(point) {
            return true;
        }
        (0..self.sphere_count)
            .any(|index| self.sphs.get(index).is_some_and(|sph| sph.blocks(point)))
    }

    pub fn is_target(&self, point: Vec3) -> bool {
        if self.cyl.is_target(point) {
            return true;
        }
        if self.sph.is_target(point) {
            return true;
        }
        (0..self.sphere_count)
            .any(|index| self.sphs.get(index).is_some_and(|sph| sph.is_target(point)))
    }

    pub fn damages_player(&self) -> bool {
        false
    }
}

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = (seed[0] as f32 / RND_SCALE[0])
        + (seed[1] as f32 / RND_SCALE[1])
        + (seed[2] as f32 / RND_SCALE[2]);
    (sum % 1.0).abs()
}

fn rnd_f(seed: &mut [i32; 3], span: f32) -> f32 {
    rnd(seed) * span
}

fn rnd_fx(seed: &mut [i32; 3], span: f32) -> f32 {
    span * (rnd(seed) - 0.5) * 2.0
}

fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value != target {
        let mut step = scale * (target - *value);
        if step > max_step {
            step = max_step;
        } else if step < -max_step {
            step = -max_step;
        }
        *value += step;
    }
}

fn add_calc0(value: &mut f32, scale: f32, max_step: f32) {
    let mut step = *value * scale;
    if step > max_step {
        step = max_step;
    } else if step < -max_step {
        step = -max_step;
    }
    *value -= step;
}

fn yrot(v: Vec3, angle: i16) -> Vec3 {
    let (s, c) = from_s16(angle).sin_cos();
    Vec3::new(c * v.x - s * v.z, v.y, s * v.x + c * v.z)
}

fn xrot(v: Vec3, angle: i16) -> Vec3 {
    let (s, c) = from_s16(angle).sin_cos();
    Vec3::new(v.x, c * v.y + s * v.z, -s * v.y + c * v.z)
}

fn node_sph_src() -> Himo3Sph {
    Himo3Sph {
        centre: Vec3::ZERO,
        radius: NODE_SPH_SRC_RADIUS,
        registered: false,
        at_type: 0,
        atp: 1,
        tg_type: TG_NODE_ALL,
        tg_set: true,
        tg_is_enemy: true,
        tg_no_con_hit: false,
        tg_shield: false,
        co_set: true,
        co_player: true,
        co_all: true,
    }
}

fn head_sph_src() -> Himo3Sph {
    Himo3Sph {
        centre: Vec3::ZERO,
        radius: HEAD_SPH_SRC_RADIUS,
        registered: false,
        at_type: 0,
        atp: 1,
        tg_type: TG_HEAD_ALL,
        tg_set: false,
        tg_is_enemy: false,
        tg_no_con_hit: false,
        tg_shield: true,
        co_set: true,
        co_player: true,
        co_all: true,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Himo3Env {
    pub hand_pos: Vec3,
    pub hand_angle: i16,
    pub wind: Vec3,
    pub wind_power: f32,
    pub wind_touch: bool,
    pub ground: Option<f32>,
    pub majroom: bool,
}

impl Default for Himo3Env {
    fn default() -> Self {
        Self {
            hand_pos: Vec3::ZERO,
            hand_angle: 0,
            wind: Vec3::ZERO,
            wind_power: 0.0,
            wind_touch: false,
            ground: None,
            majroom: false,
        }
    }
}

impl Himo3 {
    pub fn create(
        &mut self,
        actor: &mut Actor,
        statics: &mut Himo3Statics,
        paths: &Himo3Paths,
    ) -> bool {
        statics.scale = if self.m0298 == TYPE_BGN {
            SCALE_BGN + REG0_F4
        } else {
            SCALE_NORMAL
        };
        self.m15fc = self.m15c0 as f32 * statics.scale - MOVE_LENGTH_TRIM;
        if self.m0298 == TYPE_TORCH {
            self.m15fc -= TORCH_LENGTH_TRIM;
        }
        if self.m15c0 > NODE_LIMIT {
            return false;
        }
        self.path_room = actor.room;
        if self.m029a != 0xFF {
            let Some(path) = paths.get(u16::from(self.m029a), actor.room) else {
                return false;
            };
            self.path_id = path.id;
            self.m029b = (i32::from(self.m029a) + 1) as i8;
            self.m029d = 1;
        }
        self.m15e0 = 1.0;
        if self.m0298 == TYPE_BGN {
            self.cyl = Himo3Cyl::default();
            for sph in &mut self.sphs {
                *sph = Himo3Sph::default();
            }
            self.sphere_count = 0;
            self.sph = Himo3Sph::default();
            self.m20f8 = 0.0;
        } else {
            for sph in &mut self.sphs {
                *sph = node_sph_src();
            }
            self.sphere_count = 0;
            self.sph = head_sph_src();
            self.m20f8 = 0.0;
            if self.m0298 == TYPE_TORCH {
                self.m20f8 = TORCH_SHADOW;
            } else if self.m0298 == TYPE_HOOK {
                self.sph.radius = HEAD_SPH_TYPE1_RADIUS;
                self.m20f8 = HOOK_SHADOW;
            }
        }
        if !statics.hio_set {
            statics.hio_set = true;
            statics.hio.no = HIO_NO;
            self.m2110 = 0;
        }
        self.m2111 = 0;
        self.m1624 = actor.pos;
        self.m1630 = actor.pos;
        self.m1870 = statics.hio.m18;
        self.ga_pos = actor.pos;
        self.ga_target = actor.pos;
        self.ga_angle = [0; 2];
        self.ga_timer = 0;
        self.ga_bob = 0.0;
        self.ga_scale = rnd_f(&mut self.seed, GA_SCALE_SPAN) + GA_SCALE_MIN;
        self.ga_phase = rnd_f(&mut self.seed, GA_PHASE_SPAN) as i16;
        if self.m0298 == TYPE_TORCH {
            self.ga_active = 1;
        }
        self.created = true;
        let statics = *statics;
        for _ in 0..10 {
            self.execute(actor, &Himo3Env::default(), &statics, paths);
        }
        true
    }

    pub fn execute(
        &mut self,
        actor: &mut Actor,
        env: &Himo3Env,
        statics: &Himo3Statics,
        paths: &Himo3Paths,
    ) {
        if self.m02bc != 0 {
            self.m02bc -= 1;
        }
        if self.m029b != 0 {
            self.path_move(actor, paths);
        }
        self.m02ba = self.m02ba.wrapping_add(1);
        self.node[0][0] = actor.pos;
        self.control(actor, env, statics);
        if self.m0298 != TYPE_BGN {
            if self.sph.registered && env.wind_touch {
                self.m15f8 = env.hand_angle;
                self.m15f4 = 1.0;
            }
            self.acch_crr(env);
            self.m1870 = statics.hio.m18;
            self.m1624 = self.light_anchor();
            self.sph.centre = self.m1624;
            self.sph.registered = true;
            if self.m0298 == TYPE_TORCH {
                for index in 0..self.m02b4.len() {
                    if self.m02b4[index] != 0 {
                        self.m02b4[index] -= 1;
                    }
                }
                if self.m02b4[0] == 0 {
                    self.m02b4[0] = (rnd_f(&mut self.seed, REG0_F2 + TORCH_TIMER0_SPAN)
                        + TORCH_TIMER0_MIN
                        + REG0_F3) as i16;
                    self.m187c = rnd_f(&mut self.seed, REG0_F6 + TORCH_ALPHA_SPAN)
                        + TORCH_ALPHA_MIN
                        + REG0_F7;
                }
                if self.m02b4[1] == 0 {
                    self.m02b4[1] =
                        (rnd_f(&mut self.seed, TORCH_TIMER1_SPAN) + TORCH_TIMER1_MIN) as i16;
                }
                add_calc2(&mut self.m1878, self.m187c, 1.0, REG0_F4 + TORCH_ALPHA_STEP);
                if !self.fire_lit {
                    self.fire_lit = true;
                    self.m1620 = 1.0;
                }
                if self.fire_lit {
                    add_calc2(
                        &mut self.m1620,
                        rnd_f(&mut self.seed, TORCH_FLICKER_SPAN) + 1.0,
                        TORCH_FLICKER_SCALE,
                        TORCH_FLICKER_STEP,
                    );
                } else {
                    self.m1620 = 0.0;
                }
                self.ga_move(actor, statics);
            }
        }
        self.m1630 = self.m1624;
    }

    fn light_anchor(&self) -> Vec3 {
        if !self.model.set {
            return self.m1624;
        }
        let pivot = if self.m0298 == TYPE_TORCH {
            Vec3::new(
                REG0_F6 + TORCH_OFFSET[0],
                REG0_F7 + TORCH_OFFSET[1],
                REG0_F8 + TORCH_OFFSET[2],
            )
        } else {
            Vec3::new(REG0_F6, REG0_F7, REG0_F8 + HOOK_OFFSET[2])
        };
        let mut v = xrot(pivot, self.model.pitch);
        v = yrot(v, self.model.yaw);
        if self.model.tilted {
            v = xrot(v, HOOK_TILT);
        }
        self.model.pos + v
    }

    fn acch_crr(&mut self, env: &Himo3Env) {
        if let Some(ground) = env.ground {
            self.m1624.y = ground;
        }
    }

    fn path_move(&mut self, actor: &mut Actor, paths: &Himo3Paths) {
        let state = self.m02a4;
        match state {
            0 => {
                self.m029c = self.m029c.wrapping_add(self.m029d);
                let path = paths.get(self.path_id, self.path_room);
                if let Some(path) = path {
                    let num = path.points.len();
                    let num_s8 = (num as u16) as i8;
                    if self.m029c >= num_s8 {
                        if path.closed {
                            self.m029c = 0;
                        } else {
                            self.m029d = -1;
                            self.m029c = ((num as i32 - 2) & 0xFF) as u8 as i8;
                        }
                        if path.next_id != PATH_NEXT_NONE
                            && let Some(next) = paths.get(path.next_id, self.path_room)
                        {
                            self.path_id = next.id;
                        }
                    } else if self.m029c < 0 {
                        self.m029d = 1;
                        self.m029c = 1;
                    }
                }
                self.m02a4 = 1;
                let path = paths.get(self.path_id, self.path_room);
                if let Some(point) = path.and_then(|p| self.path_point(p)) {
                    self.m02a8 = point;
                }
                let arg3 = path.and_then(|p| self.path_arg3(p));
                self.path_step(actor, self.m02a8, arg3);
            }
            1 => {
                let arg3 = paths
                    .get(self.path_id, self.path_room)
                    .and_then(|p| self.path_arg3(p));
                self.path_step(actor, self.m02a8, arg3);
            }
            _ => {}
        }
    }

    fn path_point(&self, path: &Himo3Path) -> Option<Vec3> {
        let index = usize::from(self.m029c as u8);
        path.points.get(index).map(|point| point.pos)
    }

    fn path_arg3(&self, path: &Himo3Path) -> Option<u8> {
        let index = usize::from(self.m029c as u8);
        path.points.get(index).map(|point| point.arg3)
    }

    fn path_step(&mut self, actor: &mut Actor, point: Vec3, arg3: Option<u8>) {
        let delta = point - actor.pos;
        if self.m02be == 0 {
            let speed = match arg3 {
                Some(value) if value != 0 && value != PATH_ARG_NONE => value,
                _ => self.m0299,
            };
            add_calc2(
                &mut actor.speed_f,
                f32::from(speed),
                1.0,
                f32::from(speed) * PATH_ARG_SCALE,
            );
        } else {
            add_calc0(&mut actor.speed_f, 1.0, 1.0);
        }
        let flat = (delta.x * delta.x + delta.z * delta.z).sqrt();
        let yaw = atan2s(delta.x, delta.z);
        let pitch = -atan2s(delta.y, flat);
        let up = Vec3::new(0.0, 0.0, actor.speed_f);
        actor.speed = yrot(xrot(up, pitch), yaw);
        actor.pos += actor.speed;
        if delta.length() < actor.speed_f * PATH_ARRIVE_SCALE {
            self.m02a4 = 0;
        }
    }

    fn ga_move(&mut self, _actor: &Actor, _statics: &Himo3Statics) {
        if self.ga_active == 0 {
            return;
        }
        if self.ga_timer != 0 {
            self.ga_timer -= 1;
        } else {
            self.ga_timer = rnd_f(&mut self.seed, GA_TIMER_SPAN) as u8;
            self.ga_target = Vec3::new(
                self.m1624.x + rnd_fx(&mut self.seed, GA_JITTER_XZ),
                self.m1624.y + rnd_fx(&mut self.seed, GA_JITTER_Y),
                self.m1624.z + rnd_fx(&mut self.seed, GA_JITTER_XZ),
            );
        }
        let delta = self.ga_target - self.ga_pos;
        add_calc_angle_s2(
            &mut self.ga_angle[1],
            atan2s(delta.x, delta.z),
            GA_TURN_STEP,
            GA_TURN_MAX,
        );
        add_calc_angle_s2(
            &mut self.ga_angle[0],
            -atan2s(delta.y, (delta.x * delta.x + delta.z * delta.z).sqrt()),
            GA_TURN_STEP,
            GA_TURN_MAX,
        );
        let up = Vec3::new(0.0, 0.0, GA_TRAVEL);
        let moved = xrot(yrot(up, self.ga_angle[1]), self.ga_angle[0]);
        self.ga_pos += moved;
        self.ga_bob = sin_s(self.ga_phase);
        self.ga_phase = self.ga_phase.wrapping_add(GA_PHASE_STEP);
    }

    #[allow(clippy::too_many_arguments)]
    fn control(&mut self, actor: &mut Actor, env: &Himo3Env, statics: &Himo3Statics) {
        let scale = statics.scale;
        let mut index: i32 = 1;
        let mut fraction = 0.0f32;
        let mut last_pitch: i16 = 0;
        let mut last_yaw: i16 = 0;

        if self.m02be != 0 {
            let target = if self.m02be == 1 {
                env.hand_angle
            } else {
                self.m2200
            };
            let anchor = if self.m02be == 1 {
                env.hand_pos
            } else {
                self.m21f4
            };
            if self.m15c4.y == HANG_Y_SENTINEL {
                self.m15c4 = anchor;
            }
            let span = anchor - actor.pos;
            let length = span.length();
            let whole = (length / scale) as i32;
            fraction = length / scale - whole as f32;
            let mut cap = whole;
            if cap >= self.m15c0 - 1 {
                cap = self.m15c0 - 1;
            }
            if cap > 1 {
                while index < cap {
                    let t = index as f32 / (cap - 1) as f32;
                    self.node[index as usize][0] = actor.pos + span * t;
                    index += 1;
                }
            }
            add_calc_angle_s2(&mut actor.angle[1], target, HANG_ANGLE_STEP, HANG_ANGLE_MAX);
            let turned = yrot(anchor - self.m15c4, target.wrapping_neg());
            let tilt = HANG_TILT + REG0_F5;
            let limit = HANG_TILT_STEP + REG0_F6;
            let max_step = limit * self.m15e4;
            add_calc2(&mut self.m15d0, turned.z * tilt, HANG_SWAY_SCALE, max_step);
            add_calc2(&mut self.m15d8, turned.x * tilt, HANG_SWAY_SCALE, max_step);
            if turned.z.abs() > HANG_GAIN_TEST || turned.x.abs() > HANG_GAIN_TEST {
                add_calc2(&mut self.m15e4, 1.0, 1.0, HANG_GAIN_STEP);
            } else {
                add_calc2(&mut self.m15e4, HANG_GAIN_IDLE, 1.0, HANG_GAIN_STEP);
            }
            if actor.parameters == PARAM_RELEASE {
                self.m02be = 0;
                self.m02bc = HANG_HIDE_FRAMES;
                self.m15e0 = 0.0;
            }
            self.m15c4 = anchor;
        } else {
            self.m15d0 += self.m15d4;
            let decay = self.m15d0 * (SWAY_DAMP + REG0_F7);
            self.m15d4 += decay;
            add_calc0(&mut self.m15d0, SWAY_DAMP_SCALE, SWAY_DAMP_MAX + REG0_F9);
            self.m15d8 += self.m15dc;
            let decay = self.m15d8 * (SWAY_DAMP + REG0_F7);
            self.m15dc += decay;
            add_calc0(&mut self.m15d8, SWAY_DAMP_SCALE, SWAY_DAMP_MAX + REG0_F9);
            if actor.parameters == PARAM_HANG_PLAYER {
                self.m02be = 1;
                self.m15e0 = 1.0;
                if env.majroom {
                    self.majroom_event = true;
                }
            } else if actor.parameters == PARAM_HANG_POINT {
                self.m02be = 2;
                self.m15e0 = 1.0;
            }
            self.m15c4.y = HANG_Y_SENTINEL;
        }

        let drift = yrot(Vec3::new(self.m15d8, 0.0, self.m15d0), actor.angle[1]);
        if self.m02be != 0 {
            add_calc2(
                &mut self.m15e8,
                env.wind_power * WIND_HELD_SCALE,
                1.0,
                WIND_HELD_STEP,
            );
        } else {
            add_calc2(&mut self.m15e8, env.wind_power, 1.0, WIND_FREE_STEP);
        }

        if self.m15f0 > SWAY_GATE {
            self.m15fa = self.m15fa.wrapping_add(1);
        } else {
            self.m15fa = 0;
        }
        add_calc2(
            &mut self.m15f0,
            self.m15f4,
            SWAY_CHASE,
            SWAY_CHASE_STEP + REG0_F2,
        );
        add_calc0(&mut self.m15f4, 1.0, SWAY_DECAY + REG0_F3);

        let gust_phase = (i32::from(self.m15fa) * (SWAY_FREQ + REG0_S5)) as i16;
        let gust = sin_s(gust_phase) * (SWAY_AMPLITUDE + REG0_F4) * self.m15f0;
        let gust_vec = yrot(Vec3::new(0.0, 0.0, gust), self.m15f8);

        let wind_yaw = atan2s(env.wind.x, env.wind.z);
        let blow_phase = (i32::from(self.m02ba) * (SWAY_FREQ + REG0_S5)) as i16;
        let blow = sin_s(blow_phase) * (SWAY_AMPLITUDE + REG0_F4) * self.m15e8;
        let blow_vec = yrot(Vec3::new(0.0, 0.0, blow), wind_yaw) + gust_vec;

        let gravity = NODE_GRAVITY + REG0_F8;
        let reg = REG0_F1;

        let mut sphere_index = 0usize;
        let mut first = true;
        while index < self.m15c0 {
            let slot = index as usize;
            let previous = self.node[slot - 1][0];
            let current = self.node[slot][0];
            let velocity = self.node[slot][1];
            let direction = Vec3::new(
                current.x - previous.x + velocity.x + drift.x + blow_vec.x,
                current.y - previous.y + gravity + velocity.y,
                current.z - previous.z + velocity.z + drift.z + blow_vec.z,
            );
            last_pitch = atan2s(direction.y, direction.z).wrapping_neg();
            last_yaw = atan2s(
                direction.x,
                (direction.y * direction.y + direction.z * direction.z).sqrt(),
            );
            let mut reach = scale;
            if first {
                reach -= fraction * scale;
                first = false;
            }
            let up = Vec3::new(0.0, 0.0, reach);
            let step = yrot(xrot(up, last_pitch), last_yaw);
            self.node[slot][1] = current;
            self.node[slot][0] = previous + step;
            self.node[slot][1] = (self.node[slot][0] - self.node[slot][1]) * reg;

            if self.m0298 == TYPE_BGN && index == self.m15c0 - 1 {
                self.cyl.centre = if self.m02be == 0 {
                    self.node[slot][0]
                } else {
                    Vec3::new(-10000.0, -10000.0, 0.0)
                };
                self.cyl.registered = true;
            } else if (index + i32::from(self.m02ba.wrapping_mul(NODE_SPHERE_STRIDE as i16)))
                & NODE_SPHERE_MOD
                == 0
                && sphere_index < NODE_SPHERE_MAX
            {
                let radius = if self.m02be == 0 && self.m02bc == 0 {
                    NODE_SPHERE_LIVE_RADIUS + REG0_F2 + 20.0 - 20.0
                } else {
                    NODE_SPHERE_HANG_RADIUS
                };
                if let Some(sph) = self.sphs.get_mut(sphere_index) {
                    sph.radius = radius;
                    sph.centre = self.node[slot][0];
                    sph.registered = true;
                }
                sphere_index += 1;
            }
            index += 1;
        }
        self.sphere_count = sphere_index;

        let count = self.node_count();
        let fade = self.m15e0;
        for i in 0..count {
            let target = self.node[i][0];
            let slot = self.line_pos.get_mut(i);
            let Some(slot) = slot else {
                break;
            };
            if fade >= LINE_FADE_READY {
                *slot = target;
            } else {
                let step = fade * LINE_FADE_RATE;
                add_calc2(&mut slot.x, target.x, 1.0, step);
                add_calc2(&mut slot.y, target.y, 1.0, step);
                add_calc2(&mut slot.z, target.z, 1.0, step);
            }
            if self.m0298 != TYPE_BGN && i == count - 1 {
                self.model = Himo3ModelBase {
                    set: true,
                    pos: self.line_pos[i],
                    pitch: last_pitch,
                    yaw: last_yaw,
                    tilted: self.m0298 != TYPE_HOOK,
                    scale: if self.m0298 != TYPE_HOOK {
                        HIO_M0C
                    } else {
                        HIO_M08
                    },
                };
            }
        }
        if self.m0298 == TYPE_BGN {
            self.model.set = false;
        }
        add_calc2(&mut self.m15e0, 1.0, 1.0, LINE_FADE_STEP + REG0_F3);
    }
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    statics: Res<'w, Himo3Statics>,
    paths: Res<'w, Himo3Paths>,
    touch: Res<'w, Himo3WindTouch>,
    weather: Res<'w, BridgeWeather>,
    stage: Res<'w, StageName>,
    players: Query<'w, 's, (&'static Player, &'static Actor), Without<Himo3>>,
    collision: Option<Res<'w, Collision>>,
}

pub fn control(
    watch: Watch,
    mut flags: ResMut<EventFlags>,
    mut ropes: Query<(Entity, &mut Actor, &mut Himo3), (With<Himo3>, Without<Player>)>,
) {
    let hand = watch
        .players
        .iter()
        .next()
        .map(|(player, actor)| (player.pos, actor.shape_angle[1]));
    let stage = watch.stage.0.clone();
    let weather = watch.weather;
    let touching = watch.touch.touching;
    let env = Himo3Env {
        hand_pos: hand.map(|(pos, _)| pos).unwrap_or(Vec3::ZERO),
        hand_angle: hand.map(|(_, angle)| angle).unwrap_or(0),
        wind: weather.vec,
        wind_power: weather.power,
        wind_touch: touching,
        ground: None,
        majroom: MAJROOM_STAGES.contains(&stage.as_str()),
    };
    let mut statics = *watch.statics;
    let paths = &*watch.paths;
    let has_ground = watch.collision.is_some();
    for (_, mut actor, mut rope) in &mut ropes {
        if !rope.created && !rope.create(&mut *actor, &mut statics, paths) {
            continue;
        }
        let mut env = env;
        if has_ground {
            env.ground = watch
                .collision
                .as_deref()
                .and_then(|scene| scene.ground_at(rope.m1624.x, rope.m1624.z));
        }
        rope.execute(&mut *actor, &env, &statics, paths);
        if rope.majroom_event {
            rope.majroom_event = false;
            flags.set(EVENT_HANG_MAJROOM);
        }
    }
}
