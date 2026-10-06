use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::{Mat4, Vec3};

use super::actor::Actor;
use super::combat::Combat;
use super::damage;
use super::events::{SimEvent, SimEvents};
use super::item::Held;
use super::player::Player;
use super::stream::add_calc_angle_s2;
use super::vibration::Vibration;
use super::walk::{from_s16, to_s16};
use crate::text::msgproc::MsgGameInfo;

pub const BRIDGE_ACTORS: [&str; 1] = ["bridge"];

pub fn is_bridge(name: &str) -> bool {
    BRIDGE_ACTORS.contains(&name)
}

const PRM_BYTE: u32 = 0xFF;
const PRM_SUB_SHIFT: u32 = 8;
const PRM_SUB_WIDTH: u32 = 0xFF;
const PRM_PATH_SHIFT: u32 = 16;
const PRM_PATH_WIDTH: u32 = 0xFF;

const TYPE_CHAIN: u8 = 1;
const TYPE_PAIR: u8 = 2;
const TYPE_CHAIN2: u8 = 4;
const TYPE_TXM: u8 = 8;
const TYPE_SOFT: u8 = 5;
const TYPE_MATE: u8 = 0x82;

const PLANK_COUNT_MAX: i32 = 50;
const PLANK_LEN_STEP: f32 = 1.5;
const PLANK_LEN_BASE: f32 = 47.0;
const PLANK_LEN_FAR: f32 = 1300.0;
const PLANK_LEN_FAR_STEP: f32 = 3.0;

const MODE_DONE: i16 = 1;
const MODE_START: i16 = 0;
const MODE_INIT: i16 = 2;
const MODE_SWAY: i16 = 3;
const MODE_BREAK: i16 = 4;
const MODE_FALL: i16 = 5;

const PLANK_LEN_RIDE: f32 = 76.5; // <- d_a_bridge.cpp:33
const RIDE_PLAYER: f32 = 100.0; // <- d_a_bridge.cpp:56
const RIDE_MO2: f32 = 150.0; // <- d_a_bridge.cpp:60
const RIDE_BOOK: f32 = 100.0; // <- d_a_bridge.cpp:66
const RIDE_OTHER: f32 = 50.0; // <- d_a_bridge.cpp:83
const RIDE_SOFT: f32 = 0.85;
const TENSION_PLAYER: f32 = -31.0; // <- d_a_bridge.cpp:57
const TENSION_MO2: f32 = -40.0; // <- d_a_bridge.cpp:61
const TENSION_BOOK: f32 = -25.0; // <- d_a_bridge.cpp:67
const TENSION_OTHER: f32 = -10.0; // <- d_a_bridge.cpp:84
const TENSION_BOMB: f32 = -300.0; // <- d_a_bridge.cpp:89
const RIDE_FALL: f32 = -5.0; // <- d_a_bridge.cpp:52
const RIDE_FALL_MO2: f32 = -20.0; // <- d_a_bridge.cpp:62
const RIDE_SWAY_FACTOR: f32 = 0.03;
const RIDE_TWIST_MAX: f32 = 20.0; // <- d_a_bridge.cpp:117
const RIDE_TWIST_SCALE: f32 = 0.3; // <- d_a_bridge.cpp:107
const RIDE_ROLL_MAX: f32 = 50.0; // <- d_a_bridge.cpp:118
const RIDE_ROLL_STEP: f32 = 0.5;
const BOMB_REST_LIMIT: i32 = 1;
const BOMB_RELAX: f32 = 20.0;

const SWAY_SHORT: i16 = 4000; // <- d_a_bridge.cpp:396
const SWAY_LONG: i16 = 8000; // <- d_a_bridge.cpp:398
const SWAY_STEP: i16 = 3000; // <- d_a_bridge.cpp:649
const SWAY_STEP_BREAK: i16 = 4000; // <- d_a_bridge.cpp:811
const SWAY_LEAD: i16 = 1000;
const SWAY_SALT: i16 = 0x578; // <- d_a_bridge.cpp:772
const SWAY_SALT2: i16 = 0x578; // <- d_a_bridge.cpp:1108
const SWAY_SUB: i16 = 0x5DC; // <- d_a_bridge.cpp:1106
const SWAY_SUB_MUL: f32 = 5.0; // <- d_a_bridge.cpp:409
const SWAY_WIND_GAIN: f32 = 2.0; // <- d_a_bridge.cpp:1108
const SWAY_WIND_BIAS: f32 = 5.0; // <- d_a_bridge.cpp:1108
const SWAY_WIND_OFF: f32 = 0.3; // <- d_a_bridge.cpp:1107
const SWAY_SETTLE: i16 = 3000; // <- d_a_bridge.cpp:771

const SEG_LEN: f32 = 75.0; // <- d_a_bridge.cpp:413
const SEG_LEN_WIND: f32 = 5.0; // <- d_a_bridge.cpp:409
const SEG_LEN_HIMO: f32 = 23.0; // <- d_a_bridge.cpp:592
const SEG_LEN_HIMO_WIND: f32 = 7.0; // <- d_a_bridge.cpp:587
const SEG_LEN_CHAIN: f32 = 50.0;
const SEG_LEN_CHAIN_END: f32 = -50.0;
const SEG_LEN_CHAIN_MID: f32 = -40.0;
const SEG_LEN_VTX: f32 = -40.0;

const HALF_TURN: i16 = 0x8000; // <- d_a_bridge.cpp:518
const QUARTER_TURN: i16 = 0x4000; // <- d_a_bridge.cpp:78
const EIGHTH_TURN: i16 = 0x2000; // <- d_a_bridge.cpp:79

const TENSION_CUT: f32 = -200.0; // <- d_a_bridge.cpp:732
const EVENT_WAIT_MAX: i32 = 100;
const EVENT_STEP: i32 = 2;
const SHAKE_FLAGS: i32 = -0x21;
const SHAKE_DIR: Vec3 = Vec3::new(0.0, 1.0, 0.0);
const CUT_FLOOR: f32 = 30.0;
const CUT_DROP: f32 = -15.0;
const CUT_RATE: f32 = -50.0;
const CUT_HANG: f32 = -10000.0;
const BREAK_FRAMES: i16 = 50; // <- d_a_bridge.cpp:802
const BREAK_SPREAD: f32 = 50.0; // <- d_a_bridge.cpp:789

const PLANK_HP: i8 = 3; // <- d_a_bridge.cpp:644
const HULL_RADIUS: f32 = 5.0; // <- d_a_bridge.cpp:1276
const HULL_HEIGHT: f32 = 1000.0; // <- d_a_bridge.cpp:1277
const HULL_HEIGHT_PLANK: f32 = 200.0;
const TG_GATE_CUT: i16 = 20; // <- d_a_bridge.cpp:942
const TG_GATE_STRIKE: i16 = 15; // <- d_a_bridge.cpp:992
const ROPE_HEIGHT_DROP: f32 = -10000.0;
const ROPE_HEIGHT_GAIN: f32 = 100.0;
const ROPE_FLAME_TIME: i16 = 30; // <- d_a_bridge.cpp:1040
const ROPE_FLAME_REACH: f32 = 50.0;
const ROPE_FLAME_SQUASH: f32 = 0.4; // <- d_a_bridge.cpp:1037
const ROPE_HEIGHT_CHAIN: f32 = 1000.0;
const ROPE_HEIGHT_PLANK: f32 = 200.0;
const ROPE_SWAY_CHAIN: f32 = 1000.0; // <- d_a_bridge.cpp:915
const ROPE_SWAY_PLANK: f32 = 200.0; // <- d_a_bridge.cpp:915
const ROPE_SPAN: f32 = 99.0; // <- d_a_bridge.cpp:898
const ROPE_SPAN_OFF: f32 = -30.0;
const HANG_DROP: f32 = -10.0;
const RIDE_SLACK: f32 = 0.75;

const AT_TYPE_ALL: u32 = !0xE20100;
const AT_SPL_UNK0: u8 = 0;

const CULL_MIN: [f32; 3] = [-120.0, -30.0, -60.0];
const CULL_MAX: [f32; 3] = [120.0, 30.0, 60.0];
const CULL_FAR: f32 = 10.0;

const STOP_DRAW_DIST: f32 = 5000.0;

const SE_CUT_ROPE: u32 = 0x2838;
const SE_HIT_ROPE: u32 = 0x2837;
const SE_HIT_CHAIN: u32 = 0x2839;
const SE_BREAK: u32 = 0x6933;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const ITA_Z_P: [f32; 11] = [
    0.1, 0.3, 0.5, 0.75, 0.9, 1.0, 0.9, 0.75, 0.5, 0.3, 0.1,
]; // <- d_a_bridge.cpp:26

const LINE_MAIN_LINES: usize = 2;
const LINE_MAIN_SEGS: usize = 14;
const LINE_ROPE_LINES: usize = 4;
const LINE_ROPE_SEGS: usize = 5;
const HIMO_POINTS: usize = 5;

const SPARK_MIN: i32 = -5;
const SPARK_MAX: i32 = 5;
const SPARK_GATE: i32 = 2;
const SPARK_RANGE_MIN: i32 = -2;
const SPARK_RANGE_MAX: i32 = 2;

const SWORD_OFFSET: f32 = 20.0;
const SWORD_RADIUS: f32 = 20.0;
const SWORD_LO: f32 = -20.0;
const SWORD_HI: f32 = 100.0;
const CUT_START: u32 = 1;
const CUT_END: u32 = 6;
const CUT_DAMAGE_MIN: i32 = 1;
const CUT_DAMAGE_CAP: i32 = 4;

const PAIR_SHORT: i32 = 12;
const PAIR_TALL: i32 = 16;
const PAIR_TINY_COUNT: i32 = 7;
const PAIR_SHORT_COUNT: i32 = 11;
const PAIR_TALL_COUNT: i32 = 15;

const RIDE_REACH: f32 = 75.0;
const RIDE_HEIGHT: f32 = 120.0;
const RIDE_DOWN: f32 = -40.0;

const COVERED_PLANKS: u8 = 7;
const ROPE_LIVE: u8 = 0x0E;
const ROPE_LIVE_ALT: u8 = 0x0D;

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = seed[0] as f32 / RND_SCALE[0]
        + seed[1] as f32 / RND_SCALE[1]
        + seed[2] as f32 / RND_SCALE[2];
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

fn timer(value: &mut i16) {
    if *value != 0 {
        *value -= 1;
    }
}

fn timer_i(value: &mut i32) {
    if *value != 0 {
        *value -= 1;
    }
}

fn timer_u8(value: &mut u8) {
    if *value != 0 {
        *value -= 1;
    }
}

fn rot_y(angle: i16) -> Mat4 {
    Mat4::from_rotation_y(from_s16(angle))
}

fn rot_x(angle: i16) -> Mat4 {
    Mat4::from_rotation_x(from_s16(angle))
}

fn rot_z(angle: i16) -> Mat4 {
    Mat4::from_rotation_z(from_s16(angle))
}

fn place(mtx: &Mat4, v: Vec3) -> Vec3 {
    mtx.transform_point3(v)
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct BridgeWeather {
    pub vec: Vec3,
    pub power: f32,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct BridgeCamera {
    pub eye: Vec3,
    pub center: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cyl {
    pub centre: Vec3,
    pub radius: f32,
    pub height: f32,
    pub at_type: u32,
    pub at_set: bool,
    pub at_spl: u8,
    pub tg_type: u32,
    pub tg_set: bool,
    pub tg_shield: bool,
    pub co_set: bool,
    pub co_player: bool,
    pub co_all: bool,
    pub registered: bool,
}

impl Cyl {
    const SRC: Cyl = Cyl {
        centre: Vec3::ZERO,
        radius: HULL_RADIUS,
        height: HULL_HEIGHT,
        at_type: 0,
        at_set: false,
        at_spl: AT_SPL_UNK0,
        tg_type: AT_TYPE_ALL,
        tg_set: true,
        tg_shield: true,
        co_set: true,
        co_player: true,
        co_all: true,
        registered: false,
    };

    pub fn top(&self) -> f32 {
        self.centre.y + self.height
    }

    pub fn strikes_at(&self, cut: Option<Cut>) -> bool {
        let Some(cut) = cut else { return false };
        if !cut.live {
            return false;
        }
        self.strikes(cut.pos, SWORD_RADIUS)
    }

    pub fn strikes(&self, at: Vec3, reach: f32) -> bool {
        if !self.registered || !self.tg_set || !self.tg_shield {
            return false;
        }
        let dx = at.x - self.centre.x;
        let dz = at.z - self.centre.z;
        if dx * dx + dz * dz > (self.radius + reach) * (self.radius + reach) {
            return false;
        }
        let low = at.y + SWORD_LO;
        let high = at.y + SWORD_HI;
        low < self.top() && high > self.centre.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Br {
    pub m0F8: [Vec3; 3],
    pub m11C: [Vec3; 3],
    pub cyl: [Cyl; 2],
    pub m3A0: [i16; 2],
    pub m3A4: i8,
    pub m3A5: i8,
    pub m3A8: [Vec3; 2],
    pub m3C0: i16,
    pub m3C2: i16,
    pub m3CC: Vec3,
    pub mPosition: Vec3,
    pub mRotation: [i16; 3],
    pub mRotationYExtra: i16,
    pub m3EC: f32,
    pub m3F0: f32,
    pub m3F4: f32,
    pub m3F8: f32,
    pub m3FC: f32,
    pub m400: i16,
    pub m402: i16,
    pub m404: i16,
    pub m406: u8,
    pub m407: u8,
    pub m408: u8,
    pub m409: u8,
    pub mScale: Vec3,
    pub m418: i16,
    pub line1: [[Vec3; LINE_ROPE_SEGS]; LINE_ROPE_LINES],
}

impl Br {
    const fn zero() -> Self {
        Br {
            m0F8: [Vec3::ZERO; 3],
            m11C: [Vec3::ZERO; 3],
            cyl: [Cyl::SRC; 2],
            m3A0: [0; 2],
            m3A4: 0,
            m3A5: 0,
            m3A8: [Vec3::ZERO; 2],
            m3C0: 0,
            m3C2: 0,
            m3CC: Vec3::ZERO,
            mPosition: Vec3::ZERO,
            mRotation: [0; 3],
            mRotationYExtra: 0,
            m3EC: 0.0,
            m3F0: 0.0,
            m3F4: 0.0,
            m3F8: 0.0,
            m3FC: 0.0,
            m400: 0,
            m402: 0,
            m404: 0,
            m406: 0,
            m407: 0,
            m408: 0,
            m409: 0,
            mScale: Vec3::ZERO,
            m418: 0,
            line1: [[Vec3::ZERO; LINE_ROPE_SEGS]; LINE_ROPE_LINES],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rider {
    Player,
    Mo2,
    Book,
    Bomb,
    BombBlast,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ride {
    pub rider: Entity,
    pub kind: Rider,
    pub anchor: usize,
    pub yaw: i16,
    pub back: i16,
    pub mode: i16,
    pub plank: usize,
    pub owner: Entity,
    pub speed_y: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bridge {
    pub mMoveProcMode: i16,
    pub mTypeBits: u8,
    pub home_pos: Vec3,
    pub home_yaw: i16,
    pub m02D9: u8,
    pub mPathId: u8,
    pub mPathIdP: u8,
    pub mBrCount: i8,
    pub m02DD: i8,
    pub m02E0: f32,
    pub m02E4: f32,
    pub m02EC: i16,
    pub m02EE: i16,
    pub m02F0: i16,
    pub m02F2: i16,
    pub m02F4: f32,
    pub m02F8: f32,
    pub m02FC: f32,
    pub m0300: i16,
    pub m0302: i16,
    pub m0304: i32,
    pub m0308: i32,
    pub m030C: i32,
    pub m0312: i16,
    pub mEndPos: Vec3,
    pub m0320: Vec3,
    pub m032C: Vec3,
    pub m033C: i8,
    pub mbStopDraw: i8,
    pub cull_off: bool,
    pub mate: Option<Entity>,
    pub ride: Option<Ride>,
    pub spark: Vec3,
    pub seed: [i32; 3],
    pub entered: bool,
    pub line: [[Vec3; LINE_MAIN_SEGS]; LINE_MAIN_LINES],
    pub br: [Br; 50],
}

impl Bridge {
    pub fn spawn(parameters: u32) -> Self {
        let type_bits = (parameters & PRM_BYTE) as u8;
        let path = ((parameters >> PRM_PATH_SHIFT) & PRM_PATH_WIDTH) as u8;
        Bridge {
            mMoveProcMode: MODE_START,
            mTypeBits: if type_bits == PRM_BYTE as u8 { 0 } else { type_bits },
            home_pos: Vec3::ZERO,
            home_yaw: 0,
            m02D9: ((parameters >> PRM_SUB_SHIFT) & PRM_SUB_WIDTH) as u8,
            mPathId: path,
            mPathIdP: 0,
            mBrCount: 0,
            m02DD: 0,
            m02E0: 0.0,
            m02E4: 0.0,
            m02EC: 0,
            m02EE: 0,
            m02F0: 0,
            m02F2: 0,
            m02F4: 0.0,
            m02F8: 0.0,
            m02FC: 0.0,
            m0300: 0,
            m0302: 0,
            m0304: 0,
            m0308: 0,
            m030C: 0,
            m0312: 0,
            mEndPos: Vec3::ZERO,
            m0320: Vec3::ZERO,
            m032C: Vec3::ZERO,
            m033C: 0,
            mbStopDraw: 0,
            cull_off: false,
            mate: None,
            ride: None,
            spark: Vec3::ZERO,
            seed: [RND_SEED; 3],
            entered: false,
            line: [[Vec3::ZERO; LINE_MAIN_SEGS]; LINE_MAIN_LINES],
            br: [Br::zero(); 50],
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.mbStopDraw == 0
    }

    pub fn cull_far(&self) -> f32 {
        CULL_FAR
    }

    pub fn br_count(&self) -> usize {
        self.mBrCount.max(0) as usize
    }

    fn home_angle_y(&self) -> i16 {
        self.home_yaw
    }

    pub fn create_init(&mut self, actor: &mut Actor, path: Option<(Vec3, Vec3)>) {
        if path.is_none() {
            self.entered = true;
            return;
        }
        let (first, second) = match path {
            Some(pair) => pair,
            None => {
                self.entered = true;
                return;
            }
        };
        actor.pos = first;
        self.home_pos = first;
        self.mEndPos = second;
        self.mPathIdP = self.mPathId.wrapping_add(1);
        let delta = second - first;
        actor.angle[1] = to_s16(delta.x.atan2(delta.z));
        actor.angle[0] = -to_s16(delta.y.atan2((delta.x * delta.x + delta.z * delta.z).sqrt()));
        let far = if delta.length() > PLANK_LEN_FAR {
            PLANK_LEN_FAR_STEP
        } else {
            0.0
        };
        self.mBrCount =
            (delta.length() / ((far + PLANK_LEN_BASE) * PLANK_LEN_STEP)) as i8;
        if self.mTypeBits & TYPE_PAIR != 0 {
            self.m02DD = if self.mBrCount as i32 >= PAIR_TALL {
                PAIR_TALL_COUNT as i8
            } else if self.mBrCount as i32 >= PAIR_SHORT {
                PAIR_SHORT_COUNT as i8
            } else {
                PAIR_TINY_COUNT as i8
            };
        } else {
            self.m02DD = self.mBrCount;
        }
        self.home_yaw = actor.angle[1];
        self.create_init_colliders();
        self.entered = true;
    }

    fn create_init_colliders(&mut self) {
        let chain = self.mTypeBits & TYPE_CHAIN != 0;
        let stride = if chain { 0 } else { 2 };
        for i in 0..self.br_count() {
            let bit_set = (i + stride) & 3 == 0;
            let covered = self.mTypeBits & TYPE_CHAIN2 == 0 && bit_set;
            let mut scale_y = 1.0;
            let mut scale_x = 1.0;
            if !chain {
                scale_y = rnd_f(&mut self.seed, 0.3) + 1.0;
                scale_x = if bit_set {
                    1.05
                } else {
                    rnd_f(&mut self.seed, 0.1) + 1.0
                };
            }
            let mut extra = 0;
            if rnd_f(&mut self.seed, 1.0) < 0.5 {
                extra = -32768;
            }
            let br = &mut self.br[i];
            if covered {
                br.m408 = COVERED_PLANKS;
                if chain {
                    br.m418 = 0x32;
                } else {
                    br.m418 = -1;
                }
            }
            br.mScale = Vec3::new(scale_x, scale_y, 1.5);
            br.mRotationYExtra = extra;
            for slot in 0..2 {
                br.cyl[slot] = Cyl::SRC;
                br.cyl[slot].registered = false;
                if !chain {
                    br.cyl[slot].height = HULL_HEIGHT_PLANK;
                    br.cyl[slot].tg_shield = false;
                }
            }
        }
    }

    pub fn ride_call_back(&mut self, at: Vec3, old: Vec3, kind: Rider, rider: Entity) {
        let count = self.br_count();
        if count == 0 {
            return;
        }
        let head = self.br[0].mPosition;
        let flat = (at.x - head.x).powi(2) + (at.z - head.z).powi(2);
        let mut index = ((flat.sqrt() / PLANK_LEN_RIDE + 0.5) as i32).min(count as i32 - 1);
        if index < 0 {
            index = 0;
        }
        let index = index as usize;
        let soft = if self.mTypeBits & TYPE_SOFT != 0 {
            RIDE_SOFT
        } else {
            1.0
        };
        let yaw = self.br[index].mRotation[1].wrapping_neg();
        let mtx = rot_y(yaw);
        let anchor = self.br[index].mPosition;
        let sp4c = place(&mtx, at - anchor);
        let sp40 = place(&mtx, old - anchor);
        let mut speed_y = RIDE_FALL;
        let tension;
        let mut magnitude;
        match kind {
            Rider::Player => {
                magnitude = RIDE_PLAYER;
                tension = TENSION_PLAYER;
                self.m033C = 5;
                self.br[index].m3F4 = tension;
            }
            Rider::Mo2 => {
                magnitude = RIDE_MO2;
                tension = TENSION_MO2;
                speed_y = RIDE_FALL_MO2;
                self.br[index].m3F4 = tension;
            }
            Rider::Book => {
                speed_y = RIDE_FALL_MO2;
                magnitude = RIDE_BOOK;
                tension = TENSION_BOOK;
                let right = sp4c.x > 0.0;
                let mut yaw_offset = self.br[index].mRotation[1];
                let back = if right {
                    yaw_offset = yaw_offset.wrapping_sub(QUARTER_TURN);
                    -EIGHTH_TURN
                } else {
                    yaw_offset = yaw_offset.wrapping_add(QUARTER_TURN);
                    EIGHTH_TURN
                };
                self.br[index].m3F4 = tension;
                self.ride = Some(Ride {
                    rider,
                    kind,
                    anchor: if right { 1 } else { 0 },
                    yaw: yaw_offset,
                    back,
                    mode: 8,
                    plank: index,
                    owner: Entity::PLACEHOLDER,
                    speed_y,
                });
            }
            Rider::Bomb => {
                magnitude = RIDE_OTHER;
                tension = TENSION_OTHER;
                self.br[index].m3F4 = tension;
            }
            Rider::BombBlast => {
                let _ = TENSION_BOMB;
                self.m02E0 = BOMB_RELAX;
                return;
            }
            Rider::Other => {
                magnitude = RIDE_OTHER;
                tension = TENSION_OTHER;
                self.br[index].m3F4 = tension;
            }
        }
        magnitude *= soft;
        let swing = sin_s(self.m0300);
        self.br[index].m3F4 = self.br[index].m3F4 * soft
            + self.m02FC * swing * RIDE_SWAY_FACTOR * magnitude;
        self.br[index].m400 = (-sp4c.x * magnitude) as i16;
        self.br[index].m406 = 2;
        let delta = sp4c - sp40;
        let mut twist = delta.length() * RIDE_TWIST_SCALE * soft;
        if twist > RIDE_TWIST_MAX {
            twist = RIDE_TWIST_MAX;
        }
        if self.m02E0 <= twist {
            self.m02E0 = twist;
        }
        let mut roll = soft * delta.x.abs();
        if roll > RIDE_ROLL_MAX {
            roll = RIDE_ROLL_MAX;
        }
        if self.m02E4 <= roll {
            self.m02E4 += RIDE_ROLL_STEP;
        }
        if let Some(ride) = self.ride.as_mut() {
            ride.speed_y = speed_y;
        }
    }

    fn sway_control1(&mut self, wind_power: f32, wind_yaw: i16, home_yaw: i16) {
        let count = self.br_count();
        self.m02EC = self.m02EC.wrapping_add(self.m02F0);
        self.m02EE = self.m02EE.wrapping_add(self.m02F2);
        let step = if self.mBrCount > 10 {
            SWAY_SHORT
        } else {
            SWAY_LONG
        };
        let sp24 = place(&rot_y(home_yaw), Vec3::new(self.m02F8 * cos_s(self.m02EC), 0.0, 0.0));
        let sp18 = place(&rot_y(home_yaw), Vec3::new(1.0, 0.0, 0.0));
        let sp0c = place(
            &rot_y(wind_yaw),
            Vec3::new(0.0, 0.0, wind_power * SEG_LEN_WIND),
        );
        for i in 1..count {
            let here = self.br[i];
            let prev = self.br[i - 1];
            let rise = here.m3F8 * 0.5 + (here.m3FC * here.m3F0 * 0.5 + here.m3CC.y);
            let sway_x = self.m02F4
                * sin_s(self.m02EC.wrapping_add(trunc(i32::from(step) * i as i32)))
                * here.m3F0;
            let sway_y = self.m02FC
                * sin_s(
                    self.m02EE
                        .wrapping_add(trunc(i32::from(step.wrapping_add(SWAY_LEAD)) * i as i32)),
                )
                * here.m3F0;
            let x = (here.m3CC.x - prev.m3CC.x)
                + sway_x * sp18.x
                + sp24.x * here.m3F0
                + sp0c.x;
            let y = sway_y + (rise - prev.m3CC.y);
            let z = (here.m3CC.z - prev.m3CC.z)
                + sway_x * sp18.z
                + sp24.z * here.m3F0
                + sp0c.z;
            let atan = to_s16(x.atan2(z));
            let atan2 = -to_s16(y.atan2((x * x + z * z).sqrt()));
            let mtx = rot_y(atan) * rot_x(atan2);
            let sp30 = place(&mtx, Vec3::new(0.0, 0.0, SEG_LEN));
            self.br[i].m3CC = prev.m3CC + sp30;
        }
    }

    fn sway_control2(&mut self) {
        let count = self.br_count();
        if count < 2 {
            return;
        }
        for i in 0..count - 1 {
            let here = self.br[i];
            let next = self.br[i + 1];
            let rise = here.m3F8 * 0.5 + (here.m3FC * here.m3F0 * 0.5 + here.m3CC.y);
            let y = rise - next.m3CC.y;
            let x = here.m3CC.x - next.m3CC.x;
            let z = here.m3CC.z - next.m3CC.z;
            let atan = to_s16(x.atan2(z));
            let atan2 = -to_s16(y.atan2((x * x + z * z).sqrt()));
            self.br[i + 1].mRotation[1] = atan;
            self.br[i + 1].mRotation[0] = atan2;
            let mtx = rot_y(atan) * rot_x(atan2);
            let sp0c = place(&mtx, Vec3::new(0.0, 0.0, SEG_LEN));
            self.br[i].m3CC = next.m3CC + sp0c;
        }
    }

    fn sway_control3(&mut self) {
        let count = self.br_count();
        if count < 2 {
            return;
        }
        let here = self.br[0];
        let next = self.br[1];
        let x = here.m3CC.x - next.m3CC.x;
        let y = here.m3CC.y - next.m3CC.y;
        let z = here.m3CC.z - next.m3CC.z;
        self.br[0].mRotation[1] = to_s16(x.atan2(z));
        self.br[0].mRotation[0] = -to_s16(y.atan2((x * x + z * z).sqrt()));
    }

    fn cut_control1(&mut self, home_yaw: i16) {
        let spc = place(&rot_y(home_yaw), Vec3::new(0.0, 0.0, 1.0));
        let span = self.m0304.max(0) as usize;
        for i in 1..span {
            let here = self.br[i];
            let prev = self.br[i - 1];
            let mut top = here.m3CC.y + here.m3FC;
            let floor = here.m3EC + CUT_FLOOR;
            if top < floor {
                top = floor;
                self.br[i].m407 = self.br[i].m407.wrapping_add(1);
            }
            let y = top - prev.m3CC.y;
            let x = spc.x + (here.m3CC.x - prev.m3CC.x);
            let z = spc.z + (here.m3CC.z - prev.m3CC.z);
            let atan = to_s16(x.atan2(z));
            let atan2 = -to_s16(y.atan2((x * x + z * z).sqrt()));
            self.br[i - 1].mRotation[1] = atan.wrapping_add(HALF_TURN);
            self.br[i - 1].mRotation[0] = -atan2;
            if i == span - 1 {
                self.br[i].mRotation[1] = atan.wrapping_add(HALF_TURN);
                self.br[i].mRotation[0] = -atan2;
            }
            let mtx = rot_y(atan) * rot_x(atan2);
            let sp18 = place(&mtx, Vec3::new(0.0, 0.0, SEG_LEN));
            self.br[i].m3CC = prev.m3CC + sp18;
        }
    }

    fn cut_control2(&mut self, home_yaw: i16) {
        let count = self.br_count();
        if count < 2 {
            return;
        }
        let spc = place(&rot_y(home_yaw), Vec3::new(0.0, 0.0, -1.0));
        let span = ((count as i32 - 1) - self.m0304).max(0) as usize;
        for i in 0..span {
            let here = self.br[i];
            let next = self.br[i + 1];
            let mut top = here.m3CC.y + here.m3FC;
            let floor = here.m3EC + CUT_FLOOR;
            if top < floor {
                top = floor;
                self.br[i].m407 = self.br[i].m407.wrapping_add(1);
            }
            let y = top - next.m3CC.y;
            let x = spc.x + (here.m3CC.x - next.m3CC.x);
            let z = spc.z + (here.m3CC.z - next.m3CC.z);
            let atan = to_s16(x.atan2(z));
            let atan2 = -to_s16(y.atan2((x * x + z * z).sqrt()));
            self.br[i + 1].mRotation[1] = atan;
            self.br[i + 1].mRotation[0] = atan2;
            if i == span - 1 {
                self.br[i].mRotation[1] = atan;
                self.br[i].mRotation[0] = atan2;
            }
            let mtx = rot_y(atan) * rot_x(atan2);
            let sp18 = place(&mtx, Vec3::new(0.0, 0.0, SEG_LEN));
            self.br[i].m3CC = next.m3CC + sp18;
        }
    }

    pub fn bridge_move(
        &mut self,
        actor: &mut Actor,
        weather: &BridgeWeather,
        events: &mut SimEvents,
        vibration: &mut Vibration,
        running: bool,
    ) {
        match self.mMoveProcMode {
            MODE_DONE => {}
            MODE_START => {
                self.move_mode_start();
                self.move_mode_init();
                self.move_mode_sway(actor, weather, vibration, running);
            }
            MODE_INIT => {
                self.move_mode_init();
                self.move_mode_sway(actor, weather, vibration, running);
            }
            MODE_SWAY => self.move_mode_sway(actor, weather, vibration, running),
            MODE_BREAK => {
                self.move_mode_break(events);
                self.move_mode_fall();
            }
            MODE_FALL => self.move_mode_fall(),
            _ => {}
        }
    }

    fn move_mode_start(&mut self) {
        self.m02D9 = 0;
        self.mMoveProcMode = MODE_INIT;
        self.cull_off = true;
    }

    fn move_mode_init(&mut self) {
        let count = self.br_count();
        let chain = self.mTypeBits & TYPE_CHAIN != 0;
        let denom = (i32::from(self.mBrCount) - 1) as f32;
        for i in 0..count {
            let gain = if chain {
                1.0
            } else {
                ((i as f32) / denom * std::f32::consts::PI).sin().abs()
            };
            self.br[i].m3F0 = gain;
            self.br[i].m3A4 = PLANK_HP;
            self.br[i].m3A5 = PLANK_HP;
        }
        self.mMoveProcMode = MODE_SWAY;
    }

    fn move_mode_sway(
        &mut self,
        actor: &mut Actor,
        weather: &BridgeWeather,
        vibration: &mut Vibration,
        running: bool,
    ) {
        let count = self.br_count();
        if count == 0 {
            return;
        }
        let wind_yaw = to_s16(weather.vec.x.atan2(weather.vec.z));
        self.m0300 = self.m0300.wrapping_add(SWAY_STEP);
        self.br[0].m3CC = actor.pos;
        let mut sp2c = Vec3::ZERO;
        let chain = self.mTypeBits & TYPE_CHAIN != 0;
        if chain {
            sp2c = place(
                &rot_y(self.home_yaw),
                Vec3::new(self.m02F8 * cos_s(self.m02EC) * -2.0, 0.0, 0.0),
            );
            self.br[0].m3CC += sp2c;
        }
        self.sway_control1(weather.power, wind_yaw, self.home_yaw);
        self.br[count - 1].m3CC = self.mEndPos;
        if chain {
            self.br[count - 1].m3CC -= sp2c;
        }
        self.sway_control2();
        self.sway_control3();
        let sp14 = actor.pos - self.br[0].m3CC;
        actor.pos = self.br[0].m3CC;
        actor.angle = self.br[0].mRotation;
        for i in 0..count {
            let ratio = ((count - i) as f32) / (count as f32) * RIDE_SLACK;
            self.br[i].mPosition = self.br[i].m3CC + sp14 * ratio;
            let live = self.br[i].m406 != 0;
            let twist = self.br[i].m400;
            let roll = self.br[i].m3F4;
            let flags = self.br[i].m408;
            if live {
                for j in SPARK_MIN..=SPARK_MAX {
                    let at = i as i32 + j;
                    if at < 0 || at >= count as i32 {
                        continue;
                    }
                    let at = at as usize;
                    let weight = ITA_Z_P[(5 + j) as usize];
                    let target = (twist as f32 * weight * self.br[at].m3F0) as i16;
                    add_calc_angle_s2(&mut self.br[at].m402, target, 4, 0x800);
                    let goal = roll * weight;
                    add_calc2(&mut self.br[at].m3F8, goal, 1.0, 10.0);
                }
            }
            if (flags & 4) != 0 && (flags & 3) != 3 {
                let lean = if (flags & 3) == 1 {
                    (7000.0, -30.0)
                } else if (flags & 3) == 2 {
                    (-7000.0, -30.0)
                } else {
                    (0.0, -80.0)
                };
                for j in SPARK_MIN..=SPARK_MAX {
                    let at = i as i32 + j;
                    if at < 0 || at >= count as i32 {
                        continue;
                    }
                    let at = at as usize;
                    let weight = ITA_Z_P[(5 + j) as usize];
                    let target = (lean.0 * weight * self.br[at].m3F0) as i16;
                    add_calc_angle_s2(&mut self.br[at].m404, target, 4, 0x800);
                    let goal = lean.1 * weight;
                    add_calc2(&mut self.br[at].m3F8, goal, 1.0, 15.0);
                    if (flags & 3) == 0
                        && (self.mTypeBits & TYPE_CHAIN2) == 0
                        && (SPARK_RANGE_MIN..=SPARK_RANGE_MAX).contains(&j)
                        && self.br[at].m406 != 0
                    {
                        if running {
                            self.m0308 = 0;
                        } else {
                            self.m0308 += EVENT_STEP;
                        }
                        if self.m0308 > EVENT_WAIT_MAX {
                            self.mMoveProcMode = MODE_BREAK;
                            self.m0304 = at as i32;
                            if self.m033C != 0 {
                                let _ = vibration.start_shock(5, SHAKE_FLAGS, SHAKE_DIR);
                            }
                            break;
                        }
                    }
                    if self.br[at].m3F4 < TENSION_CUT {
                        self.mMoveProcMode = MODE_BREAK;
                        self.m0304 = at as i32;
                        break;
                    }
                }
            }
            if (flags & 4) != 0 && (self.br[i].m3A0[0] != 0 || self.br[i].m3A0[1] != 0) {
                let swing = (i32::from(self.br[i].m3A0[0]) | i32::from(self.br[i].m3A0[1])) as f32
                    * 150.0;
                let push = swing * sin_s(trunc(i32::from(self.m0300) * 4));
                for j in SPARK_MIN..=SPARK_MAX {
                    let at = i as i32 + j;
                    if at < 0 || at >= count as i32 {
                        continue;
                    }
                    let at = at as usize;
                    let weight = ITA_Z_P[(5 + j) as usize];
                    let delta = (push * weight * self.br[at].m3F0) as i16;
                    self.br[at].m404 = self.br[at].m404.wrapping_add(delta);
                }
            }
            if self.br[i].m406 != 0 {
                self.br[i].m406 -= 1;
            }
            self.br[i].m400 = 0;
            self.br[i].mRotation[2] = self.br[i].m402.wrapping_add(self.br[i].m404);
            add_calc_angle_s2(&mut self.br[i].m402, 0, 4, 0x400);
            add_calc_angle_s2(&mut self.br[i].m404, 0, 4, 0x400);
            add_calc2(&mut self.br[i].m3FC, CUT_DROP, 1.0, 5.0);
            add_calc0(&mut self.br[i].m3F8, 1.0, 5.0);
        }
        timer_i(&mut self.m0308);
        self.m02FC = self.m02E0;
        self.m02F4 = self.m02E0;
        self.m02F8 = self.m02E4;
        self.m02F2 = SWAY_SETTLE;
        self.m02F0 = SWAY_SALT;
        let goal = if weather.power > 0.1 { 2.0 } else { 0.0 };
        add_calc2(&mut self.m02E0, goal, 0.1, 0.1);
        add_calc2(&mut self.m02E4, goal * 0.3, 0.1, 0.05);
    }

    fn move_mode_break(&mut self, events: &mut SimEvents) {
        let count = self.br_count();
        let pivot = self.m0304;
        for i in 0..count {
            self.br[i].m3FC = 0.0;
            let at = i as i32;
            if at == pivot || at == pivot - 1 || at == pivot + 1 {
                let mut spark = self.br[i].mPosition;
                spark.x += rnd_fx(&mut self.seed, BREAK_SPREAD);
                spark.z += rnd_fx(&mut self.seed, BREAK_SPREAD);
                self.spark = spark;
                let mut again = self.br[i].mPosition;
                again.x += rnd_fx(&mut self.seed, BREAK_SPREAD);
                again.z += rnd_fx(&mut self.seed, BREAK_SPREAD);
                self.spark = again;
            }
        }
        self.mMoveProcMode = MODE_FALL;
        self.m0312 = BREAK_FRAMES;
        events.list.push(SimEvent::Se(SE_BREAK));
    }

    fn move_mode_fall(&mut self) {
        let count = self.br_count();
        if count == 0 {
            return;
        }
        timer(&mut self.m0312);
        self.m0300 = self.m0300.wrapping_add(SWAY_STEP_BREAK);
        self.br[0].m3CC = self.home_pos;
        self.cut_control1(self.home_yaw);
        self.br[count - 1].m3CC = self.mEndPos;
        self.cut_control2(self.home_yaw);
        for i in 0..count {
            self.br[i].mPosition = self.br[i].m3CC;
            add_calc2(&mut self.br[i].m3FC, CUT_RATE, 1.0, 5.0);
            self.br[i].m3EC = CUT_HANG;
        }
    }

    pub fn execute(
        &mut self,
        actor: &mut Actor,
        weather: &BridgeWeather,
        camera: &BridgeCamera,
        events: &mut SimEvents,
        vibration: &mut Vibration,
        running: bool,
        flame: Option<Vec3>,
        cut: Option<Cut>,
        mates: &[(Entity, u8, Vec3, Vec3)],
    ) {
        if self.m033C != 0 {
            self.m033C -= 1;
        }
        let eye_dir = actor.pos - camera.eye;
        if eye_dir.length() > STOP_DRAW_DIST {
            let look = camera.center - camera.eye;
            let atan = to_s16(look.x.atan2(look.z));
            if place(&rot_y(atan.wrapping_neg()), eye_dir).z < 0.0 {
                self.mbStopDraw = 1;
                return;
            }
        }
        self.mbStopDraw = 0;
        self.m0302 = self.m0302.wrapping_add(1);
        if self.mTypeBits & TYPE_PAIR != 0 && self.mate.is_none() {
            self.mate = mates
                .iter()
                .find(|(_, bits, _, _)| *bits & TYPE_MATE == TYPE_PAIR)
                .map(|(entity, _, _, _)| *entity);
        }
        self.bridge_move(actor, weather, events, vibration, running);
        let count = self.br_count();
        self.m030C = 0;
        let wind_yaw = to_s16(weather.vec.x.atan2(weather.vec.z));
        for i in 0..count {
            let mut mtx = Mat4::from_translation(self.br[i].mPosition);
            mtx *= rot_y(self.br[i].mRotation[1]);
            mtx *= rot_x(self.br[i].mRotation[0]);
            mtx *= rot_z(self.br[i].mRotation[2]);
            let span = self.br[i].mScale.x * ROPE_SPAN;
            self.br[i].m11C[1] = place(&mtx, Vec3::new(span, 0.0, 0.0));
            self.br[i].m0F8[1] = place(&mtx, Vec3::new(-span, 0.0, 0.0));
            self.br[i].m0F8[2] = place(&mtx, Vec3::new(-span, ROPE_SPAN_OFF, 0.0));
            self.br[i].m11C[2] = place(&mtx, Vec3::new(span, ROPE_SPAN_OFF, 0.0));
            if self.br[i].m408 & 4 != 0 {
                self.plank_rope(i, count, flame, cut, events);
                for slot in 0..2 {
                    self.br[i].cyl[slot].registered = true;
                }
            } else {
                for slot in 0..2 {
                    self.br[i].cyl[slot].registered = false;
                }
            }
            for slot in 0..2 {
                timer(&mut self.br[i].m3A0[slot]);
            }
            if i as i32 >= i32::from(self.m02DD) {
                self.br[i].mScale = Vec3::ZERO;
                self.br[i].m408 = 0;
            }
            if self.mTypeBits & TYPE_CHAIN == 0 && self.br[i].m408 & 4 != 0 {
                self.rope_lines(i, wind_yaw, weather.power);
            }
        }
    }

    fn plank_rope(
        &mut self,
        i: usize,
        count: usize,
        flame: Option<Vec3>,
        cut: Option<Cut>,
        events: &mut SimEvents,
    ) {
        if self.br[i].m418 != 0 {
            if self.br[i].m418 > 0 {
                self.br[i].m418 -= 1;
            }
            let rise = if self.mTypeBits & TYPE_CHAIN != 0 {
                ROPE_SWAY_CHAIN
            } else {
                ROPE_SWAY_PLANK
            };
            self.br[i].m11C[0] = self.br[i].m11C[1] + Vec3::new(0.0, rise, 0.0);
            self.br[i].m0F8[0] = self.br[i].m0F8[1] + Vec3::new(0.0, rise, 0.0);
            if self.mTypeBits & TYPE_PAIR != 0 && i == (i32::from(self.m02DD) - 1) as usize {
                self.m0320 = self.br[i].m11C[0];
                self.m032C = self.br[i].m0F8[0];
            }
        }
        let mut sound = 0u32;
        let mut b_var1 = false;
        let plain = self.mTypeBits & 9 == 0;
        if plain {
            if count - i < 3 {
                self.br[i].m408 = 0;
            }
            if self.m0304 != 0 {
                self.br[i].m408 &= 0x0C;
            }
            if self.br[i].cyl[0].strikes_at(cut) && self.br[i].m3A0[0] < TG_GATE_CUT {
                self.br[i].m3A0[0] = TG_GATE_CUT;
                sound = SE_CUT_ROPE;
                self.br[i].m3A4 = (i32::from(self.br[i].m3A4) - cut_damage(cut)) as i8;
                if self.br[i].m3A4 <= 0 {
                    self.br[i].m408 &= ROPE_LIVE;
                } else {
                    b_var1 = true;
                }
            }
            if self.br[i].cyl[1].strikes_at(cut) && self.br[i].m3A0[1] < TG_GATE_CUT {
                self.br[i].m3A0[1] = TG_GATE_CUT;
                sound = SE_CUT_ROPE;
                self.br[i].m3A5 = (i32::from(self.br[i].m3A5) - cut_damage(cut)) as i8;
                if self.br[i].m3A5 <= 0 {
                    self.br[i].m408 &= ROPE_LIVE_ALT;
                } else {
                    b_var1 = true;
                }
            }
        } else {
            if self.br[i].cyl[0].strikes_at(cut) && self.br[i].m3A0[0] < TG_GATE_STRIKE {
                self.br[i].m3A0[0] = TG_GATE_STRIKE;
                sound = SE_HIT_CHAIN;
            }
            if self.br[i].cyl[1].strikes_at(cut) && self.br[i].m3A0[1] < TG_GATE_STRIKE {
                self.br[i].m3A0[1] = TG_GATE_STRIKE;
                sound = SE_HIT_CHAIN;
            }
        }
        if sound != 0 && (self.mTypeBits & TYPE_TXM != 0 || b_var1) {
            sound = SE_HIT_ROPE;
        }
        if sound != 0 {
            events.list.push(SimEvent::Se(sound));
        }
        let mut top = self.br[i].m11C[1];
        if self.br[i].m408 & 1 == 0 {
            top.y += ROPE_HEIGHT_DROP;
        }
        self.br[i].cyl[0].centre = top;
        if self.br[i].m3C0 != 0 {
            self.br[i].m3C0 -= 1;
            if self.br[i].m3C0 == 0 {
                self.br[i].m408 &= ROPE_LIVE;
            }
        } else if plain && flame.is_some_and(|tip| flame_near(top, tip)) {
            self.br[i].m3C0 = ROPE_FLAME_TIME;
        }
        let mut rest = self.br[i].m0F8[1];
        if self.br[i].m408 & 2 == 0 {
            rest.y += ROPE_HEIGHT_DROP;
        }
        self.br[i].cyl[1].centre = rest;
        if self.br[i].m3C2 != 0 {
            self.br[i].m3C2 -= 1;
            if self.br[i].m3C2 == 0 {
                self.br[i].m408 &= ROPE_LIVE_ALT;
            }
        } else if plain && flame.is_some_and(|tip| flame_near(rest, tip)) {
            self.br[i].m3C2 = ROPE_FLAME_TIME;
        }
    }

    fn rope_lines(&mut self, i: usize, wind_yaw: i16, wind_power: f32) {
        let index = self.m030C + 1;
        let phase = trunc(
            i32::from(self.m02D9) * i32::from(SWAY_SUB)
                + i32::from(self.m0302) * i32::from(SWAY_SALT2),
        );
        let gust =
            (wind_power + SWAY_WIND_OFF) * (sin_s(phase) * SWAY_WIND_GAIN + SWAY_WIND_BIAS);
        let sp84 = place(&rot_y(wind_yaw), Vec3::new(0.0, 0.0, gust));
        let wobble = sin_s(trunc(i32::from(self.m0300) * 6));
        if self.br[i].m408 & 1 != 0 {
            let live = self.br[i].m11C[0];
            self.write_line(0, index, live);
        } else {
            let before = self.read_line(0, index - 1);
            let after = self.read_line(0, index + 1);
            let delta = before - after;
            let mid = Vec3::new(
                sp84.x + (delta.x * 0.5 + after.x),
                ((self.br[i].m3A0[0] as f32) * wobble + (delta.y * 0.5 + after.y)) + HANG_DROP,
                sp84.z + (delta.z * 0.5 + after.z),
            );
            self.write_line(0, index, mid);
            himo_cut_control1(&mut self.br[i].line1[0], wind_power, wind_yaw, mid);
            let head = self.br[i].m11C[1];
            himo_cut_control1(&mut self.br[i].line1[2], wind_power, wind_yaw, head);
        }
        if self.br[i].m408 & 2 != 0 {
            let live = self.br[i].m0F8[0];
            self.write_line(1, index, live);
        } else {
            let before = self.read_line(1, index - 1);
            let after = self.read_line(1, index + 1);
            let delta = before - after;
            let mid = Vec3::new(
                sp84.x + (delta.x * 0.5 + after.x),
                ((self.br[i].m3A0[1] as f32) * wobble + (delta.y * 0.5 + after.y)) + HANG_DROP,
                sp84.z + (delta.z * 0.5 + after.z),
            );
            self.write_line(1, index, mid);
            himo_cut_control1(&mut self.br[i].line1[1], wind_power, wind_yaw, mid);
            let head = self.br[i].m0F8[1];
            himo_cut_control1(&mut self.br[i].line1[3], wind_power, wind_yaw, head);
        }
        self.m030C += 1;
    }

    fn read_line(&self, slot: usize, index: i32) -> Vec3 {
        self.line
            .get(slot)
            .and_then(|row| row.get(index.max(0) as usize))
            .copied()
            .unwrap_or(Vec3::ZERO)
    }

    fn write_line(&mut self, slot: usize, index: i32, value: Vec3) {
        if let Some(cell) = self
            .line
            .get_mut(slot)
            .and_then(|row| row.get_mut(index.max(0) as usize))
        {
            *cell = value;
        }
    }
}

fn sin_s(units: i16) -> f32 {
    super::stream::sin_s(units)
}

fn cos_s(units: i16) -> f32 {
    super::stream::cos_s(units)
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cut {
    pub live: bool,
    pub pos: Vec3,
    pub damage: i32,
}

fn himo_cut_control1(
    pts: &mut [Vec3; HIMO_POINTS],
    wind_power: f32,
    wind_yaw: i16,
    head: Vec3,
) {
    pts[0] = head;
    let moved = place(
        &rot_y(wind_yaw),
        Vec3::new(0.0, 0.0, wind_power * SEG_LEN_HIMO_WIND),
    );
    for i in 1..HIMO_POINTS {
        let here = pts[i];
        let prev = pts[i - 1];
        let x = moved.x + (here.x - prev.x);
        let y = (here.y - prev.y) - HANG_DROP;
        let z = moved.z + (here.z - prev.z);
        let atan = to_s16(x.atan2(z));
        let atan2 = -to_s16(y.atan2((x * x + z * z).sqrt()));
        let mtx = rot_y(atan) * rot_x(atan2);
        let sp18 = place(&mtx, Vec3::new(0.0, 0.0, SEG_LEN_HIMO));
        pts[i] = prev + sp18;
    }
}

fn trunc(value: i32) -> i16 {
    to_s16(value as f32)
}

fn cut_damage(cut: Option<Cut>) -> i32 {
    let power = cut.map_or(0, |c| c.damage);
    if power > CUT_DAMAGE_MIN {
        CUT_DAMAGE_CAP
    } else {
        power
    }
}

fn flame_near(anchor: Vec3, tip: Vec3) -> bool {
    let mut probe = anchor;
    probe.y += ROPE_HEIGHT_GAIN;
    let mut delta = probe - tip;
    delta.y *= ROPE_FLAME_SQUASH;
    delta.length() < ROPE_FLAME_REACH
}

fn rider_kind(name: &str) -> Rider {
    match name {
        "mo2" => Rider::Mo2,
        "Bk" => Rider::Book,
        "Bomb" => Rider::Bomb,
        "Player" => Rider::Player,
        _ => Rider::Other,
    }
}

fn on_deck(br: &[Br; 50], count: usize, at: Vec3) -> bool {
    if count == 0 {
        return false;
    }
    let span = (count - 1) as f32 * SEG_LEN;
    for i in 0..count {
        if br[i].m408 & 4 == 0 {
            continue;
        }
        let gap = at - br[i].mPosition;
        if gap.x * gap.x + gap.z * gap.z > span * span {
            continue;
        }
        if gap.y.abs() <= RIDE_HEIGHT && gap.y >= RIDE_DOWN {
            return true;
        }
    }
    false
}

#[derive(SystemParam)]
pub struct Env<'w> {
    pub weather: Res<'w, BridgeWeather>,
    pub camera: Res<'w, BridgeCamera>,
    pub combat: Res<'w, Combat>,
    pub held: Res<'w, Held>,
    pub info: Res<'w, MsgGameInfo>,
}

pub fn control(
    env: Env,
    mut events: ResMut<SimEvents>,
    mut vibration: ResMut<Vibration>,
    mut bridges: Query<(Entity, &mut Actor, &mut Bridge)>,
    riders: Query<(Entity, &Actor), Without<Bridge>>,
    mut players: Query<&mut Player>,
) {
    let running = env.info.event_run_check();
    let blade = env.held.item;
    let soup = env.combat.soup;
    let live = env.combat.active
        && env.combat.frame >= CUT_START
        && env.combat.frame < CUT_END;
    let player = players.iter_mut().next().map(|p| {
        let cut = Cut {
            live,
            pos: Vec3::new(
                p.pos.x + SWORD_OFFSET * p.yaw.sin(),
                p.pos.y,
                p.pos.z + SWORD_OFFSET * p.yaw.cos(),
            ),
            damage: damage::sword_power(blade, soup),
        };
        (p, cut)
    });
    let (mut player, cut) = match player {
        Some((player, cut)) => (Some(player), Some(cut)),
        None => (None, None),
    };
    let mates: Vec<(Entity, u8, Vec3, Vec3)> = bridges
        .iter()
        .map(|(entity, _, bridge)| (entity, bridge.mTypeBits, bridge.m0320, bridge.m032C))
        .collect();
    let deck: Vec<(Entity, Vec3, Rider)> = riders
        .iter()
        .map(|(entity, actor)| (entity, actor.pos, rider_kind(&actor.name)))
        .collect();
    for (_, mut actor, mut bridge) in &mut bridges {
        if !bridge.entered {
            bridge.create_init(&mut *actor, None);
        }
        let count = bridge.br_count();
        let mut carry: Option<(Entity, f32)> = None;
        if let Some(link) = player.as_deref_mut() {
            if on_deck(&bridge.br, count, link.pos) {
                bridge.ride_call_back(link.pos, link.pos, Rider::Player, Entity::PLACEHOLDER);
                carry = bridge.ride.as_ref().map(|r| (r.rider, r.speed_y));
            }
        }
        for (entity, at, kind) in &deck {
            if *kind == Rider::Player || !on_deck(&bridge.br, count, *at) {
                continue;
            }
            bridge.ride_call_back(*at, *at, *kind, *entity);
            carry = bridge.ride.as_ref().map(|r| (r.rider, r.speed_y));
        }
        bridge.execute(
            &mut *actor,
            &env.weather,
            &env.camera,
            &mut events,
            &mut vibration,
            running,
            None,
            cut,
            &mates,
        );
        if let Some((_, speed_y)) = carry
            && let Some(link) = player.as_deref_mut()
        {
            link.speed_y = speed_y;
        }
    }
}
