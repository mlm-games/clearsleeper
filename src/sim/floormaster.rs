use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::arrow;
use super::boomerang;
use super::bombflower;
use super::carry;
use super::collision::Collision;
use super::combat::Combat;
use super::damage;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{Switches, ZoneActors};
use super::hammer;
use super::hookshot;
use super::iball;
use super::item::Held;
use super::jump;
use super::player::Player;
use super::ship::Ship;
use super::roll;
use super::stream;
use super::tsubo;
use super::walk;

pub const FLOORMASTER_ACTORS: [&str; 3] = ["Fmaster", "Fmastr1", "Fmastr2"];

const BYTE_MASK: u32 = 0xFF;
const KIND_SHIFT: u32 = 0x08;
const KIND_MASK: u32 = 0x3;
const TARGET_SHIFT: u32 = 0x0A;
const TARGET_MASK: u32 = 0x3;
const PATH_SHIFT: u32 = 0x10;
const PATH_MASK: u32 = 0xFF;
const SWITCH_SHIFT: u32 = 0x18;
const HOME_X_SHIFT: u32 = 0x08;
const HOME_X_MASK: u32 = 0xFF;
const NO_BYTE: u8 = 0xFF;
const KIND_THREE: u8 = 3;
const TARGET_THREE: u8 = 3;

const HIO_068: i16 = 0x1e;
const HIO_SP_VJUMP_HEIGHT: f32 = 240.0;
const HIO_SP_VJUMP_RADIUS: f32 = 100.0;
const HIO_SP_VJUMP_START: f32 = 0.0;
const HIO_SP_VJUMP_END: f32 = 15.0;
const HIO_SP_JUMP_HEIGHT: f32 = 120.0;
const HIO_SP_JUMP_RADIUS: f32 = 100.0;
const HIO_SP_JUMP_START: f32 = 5.0;
const HIO_SP_JUMP_END: f32 = 15.0;
const HIO_11C: i16 = 100;
const HIO_FREEZE: i32 = 0x78;

const HIO_BODY_OFFSET: [f32; 3] = [0.0, 0.0, -30.0];
const HIO_HAND_OFFSET: [f32; 3] = [40.0, -20.0, 0.0];
const HIO_TIP_OFFSET: [f32; 3] = [0.0, -20.0, 0.0];
const HOLE_SCALE: f32 = 0.1;
const PIYO_FIRST: i32 = 0x14;
const PIYO_LAST: i32 = 0x3C;
const NPC_FACE_GATE: i32 = 0x500;
const THROW_SETTLE: i32 = 0x3c;
const CHASE_STEP: f32 = 5.0;
const TSUBO_ACTOR: &str = "Tsubo";

const HIO_0A8: f32 = 600.0;
const HIO_MAX_HEALTH: i32 = 0xC;
const HIO_066: i16 = 0x28;
const HIO_084: i16 = 0x5A;
const HIO_086: i16 = 0xF0;
const HIO_088: i16 = 0x3C;
const HIO_08A: i16 = 1;
const HIO_08C: i16 = 0x3C;
const HIO_08E: i16 = 0x78;
const HIO_090: i16 = 10;
const HIO_092: i16 = 0x11;
const HIO_094: i16 = 0;
const HIO_096: i16 = 500;
const HIO_098: i16 = 18000;
const HIO_09A: i16 = 7000;
const HIO_120: f32 = 0.01;
const HIO_124: f32 = 1.0;
const HIO_128: f32 = 0.1;
const HIO_12C: f32 = 0.02;
const HIO_0C4: f32 = 290.0;
const HIO_0C8: f32 = 80.0;
const HIO_0CC: f32 = 7.0;
const HIO_0DC: f32 = 100.0;
const HIO_0E0: f32 = 150.0;
const HIO_0E4: f32 = 500.0;
const HIO_0E8: f32 = 800.0;
const HIO_0EC: f32 = 30.0;
const HIO_0F0: f32 = 25.0;
const HIO_0F4: f32 = 9.0;
const HIO_0F8: f32 = -2.5;
const HIO_0FC: f32 = 16.0;
const DROP_SCALE: f32 = 1000.0;
const TSKU_DROP_KINDS: [u8; 6] = [0, 1, 2, 4, 5, 6];
const HIO_09C: i16 = 5;
const HIO_100: i16 = 0x1e;
const HIO_110: f32 = 20.0;
const HIO_114: f32 = 50.0;
const HIO_118: f32 = -20.0;
const HIO_0B0: f32 = 30.0;
const HIO_0C0: f32 = 15.0;
const HIO_0A0: i16 = 0x14;
const HIO_0B4: f32 = 1000.0;
const HIO_0B8: f32 = 1000.0;
const HIO_09E: i16 = 0x96;
const HIO_0AC: f32 = 300.0;
const HIO_0BC: f32 = 750.0;
const HIO_0A6: i16 = 10;

const NO_TARGET: u8 = 0;
const TARGET_PLAYER: u8 = 1;
const TARGET_BOMB: u8 = 2;
const TARGET_TSUBO: u8 = 3;
const TARGET_NPC: u8 = 4;

const CARRY_NONE: u8 = 0;
const CARRY_PLAYER: u8 = 1;
const CARRY_BOMB: u8 = 2;
const CARRY_TSUBO: u8 = 3;
const CARRY_NPC: u8 = 4;

const TSUBO_POT: u8 = 0;
const TSUBO_BIG: u8 = 1;
const TSUBO_JUG: u8 = 2;
const TSUBO_STOOL: u8 = 4;
const TSUBO_SKULL: u8 = 5;
const TSUBO_PAIL: u8 = 6;
const SKULL_PARAM: u32 = 0x57f3f3f;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

pub const CULL_MIN: [f32; 3] = [-300.0, -50.0, -300.0];
pub const CULL_MAX: [f32; 3] = [300.0, 400.0, 300.0];

const SE_SPOT: u32 = 0x7032;
const SE_LK_SW_HIT_S: u32 = 0x2803;
const SE_LK_W_WEP_HIT: u32 = 0x2833;
const SE_LK_HAMMER_HIT: u32 = 0x2855;
const SE_LK_HS_SPIKE: u32 = 0x286F;
const SE_LK_ARROW_HIT: u32 = 0x2879;
const SE_CV_FM_DAMAGE: u32 = 0x48B1;
const SE_CV_FM_DIE: u32 = 0x48B2;
const SE_CV_MD_DAMAGE: u32 = 0x48AA;
const SE_CV_CB_DAMAGE: u32 = 0x48BF;
const SE_CM_MD_PIYO: u32 = 0x50BC;
const SE_CM_FREEZE: u32 = 0x58FA;
const SE_CM_L_ARROW_SHRINK: u32 = 0x5904;
const SE_CM_L_ARROW_PASS: u32 = 0x5905;
const SE_CM_FM_GRAB: u32 = 0x58AD;
const SE_CM_FM_GRAB_FOOT: u32 = 0x58AF;
const SE_CM_FM_PULL_IN: u32 = 0x58B0;
const SE_CM_FM_GRAB_HAND: u32 = 0x58B1;

pub const BCK_WAIT: i32 = 0;
pub const BCK_DERU: i32 = 1;
pub const BCK_MODORU: i32 = 2;
pub const BCK_TUKAMU: i32 = 3;
pub const BCK_TUKAMU2WAIT: i32 = 4;
pub const BCK_HIKIKOMU: i32 = 5;
pub const BCK_DAMAGE: i32 = 6;
pub const BCK_TUMAMIWAIT: i32 = 7;
pub const BCK_ASITUKAMI: i32 = 8;
pub const BCK_NAGERU: i32 = 9;
pub const BCK_YAYU: i32 = 10;
pub const BCK_MAHI: i32 = 11;
pub const BCK_KUSIZASI: i32 = 12;

const FRAME_WAIT: i16 = 44;
const FRAME_DERU: i16 = 44;
const FRAME_MODORU: i16 = 44;
const FRAME_TUKAMU: i16 = 19;
const FRAME_TUKAMU2WAIT: i16 = 30;
const FRAME_HIKIKOMU: i16 = 49;
const FRAME_DAMAGE: i16 = 42;
const FRAME_TUMAMIWAIT: i16 = 44;
const FRAME_ASITUKAMI: i16 = 41;
const FRAME_NAGERU: i16 = 31;
const FRAME_YAYU: i16 = 44;
const FRAME_MAHI: i16 = 15;
const FRAME_KUSIZASI: i16 = 8;

const PRM_COUNT: usize = 15;
const EMODE_NONE: u8 = 0;
const EMODE_LOOP: u8 = 2;
const STATE_STOP: u8 = 1;
const STATE_LOOP: u8 = 2;
const ANM_NONE: u8 = 0xF;
const ANM_NO_INDEX: i8 = -1;
const ANM_NO_NEXT: i8 = -1;

const ANM_INDEX: [i8; PRM_COUNT] = [
    ANM_NO_INDEX,
    0,
    1,
    2,
    3,
    4,
    5,
    6,
    1,
    7,
    8,
    9,
    10,
    11,
    12,
];

const ANM_BCK: [i32; PRM_COUNT] = [
    BCK_WAIT,
    BCK_DERU,
    BCK_MODORU,
    BCK_TUKAMU,
    BCK_TUKAMU2WAIT,
    BCK_HIKIKOMU,
    BCK_DAMAGE,
    BCK_TUMAMIWAIT,
    BCK_ASITUKAMI,
    BCK_NAGERU,
    BCK_YAYU,
    BCK_MAHI,
    BCK_KUSIZASI,
    BCK_MAHI,
    BCK_KUSIZASI,
];

const ANM_NEXT: [i8; PRM_COUNT] = [
    ANM_NO_NEXT,
    ANM_NO_NEXT,
    1,
    8,
    ANM_NO_NEXT,
    1,
    ANM_NO_NEXT,
    1,
    ANM_NO_NEXT,
    ANM_NO_NEXT,
    1,
    9,
    ANM_NO_NEXT,
    ANM_NO_NEXT,
    ANM_NO_NEXT,
];

const ANM_MORF: [f32; PRM_COUNT] = [
    8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 0.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0,
];

const ANM_SPEED: [f32; PRM_COUNT] = [
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
];

const ANM_LOOP: [u8; PRM_COUNT] = [
    EMODE_NONE,
    EMODE_LOOP,
    EMODE_NONE,
    EMODE_NONE,
    EMODE_NONE,
    EMODE_NONE,
    EMODE_NONE,
    EMODE_NONE,
    EMODE_NONE,
    EMODE_LOOP,
    EMODE_NONE,
    EMODE_NONE,
    EMODE_NONE,
    EMODE_LOOP,
    EMODE_LOOP,
];

const HIT_NONE: u8 = 0;
const HIT_CUT: u8 = 1;
const HIT_WIND: u8 = 3;
const HIT_BOOMERANG: u8 = 4;
const HIT_HOOKSHOT: u8 = 5;
const HIT_ARROW: u8 = 6;
const HIT_BOMB: u8 = 7;
const HIT_HAMMER: u8 = 8;
const HIT_HAMMER_TURN: u8 = 9;
const HIT_ICE: u8 = 10;
const HIT_LIGHT: u8 = 11;
const HIT_GRAPPLE: u8 = 0xD;
const HIT_JUMP_CUT: u8 = 0xC;
const GRAPPLE_HEALTH: i32 = 10;
const ARM_TIMER: i32 = 0x28;
const ARM_SWAY: i16 = 0x2B00;

const REACH_SWORD: f32 = 20.0;
const REACH_ARROW: f32 = 35.0;
const REACH_HAMMER: f32 = 50.0;
const REACH_HOOKSHOT: f32 = 5.0;
const REACH_BOOMERANG: f32 = 90.0;
const HOLE_GATE: f32 = 0.015;
const ATTENTION_LIFT: f32 = 20.0;
const LINE_LIFT: f32 = 100.0;
const AREA_RAYS: usize = 12;
const AREA_STEP: i16 = 0x1555;
const AREA_DOWN: f32 = -30.0;
const AREA_PROBE: f32 = 60.0;
const RADIUS_FIXED: f32 = 3000.0;
const RADIUS_STEP: f32 = 100.0;
const MOVE_CAP: f32 = 40.0;
const FOOT_Y: f32 = 10.0;
const FOOT_SPEED: f32 = 4.0;
const ESCAPE_SPAN: f32 = 100.0;
const ESCAPE_ROLL: f32 = 33.0;
const HEAD_TOP: f32 = 60.0;
const STEAL_ITEMS: i16 = 3;
const RANDOM_SPAN: f32 = 9.0;
const RANDOM_BASE: f32 = 1.0;
const GRAB_LIMIT: f32 = 100000.0;
const RATE_DASH: f32 = 0.1;

const TURN_DIV: i16 = 4;
const TURN_CAP: i16 = 0x800;
const TURN_DIV_SLOW: i16 = 8;
const TURN_CAP_SLOW: i16 = 0x300;
const TURN_DIV_FACE: i16 = 10;
const TURN_CAP_FACE: i16 = 0x100;
const TURN_CAP_NPC: i16 = 0x500;
const ARM_DIV: i16 = 10;
const ARM_CAP: i16 = 0x1C8;
const ARM_RATE: i16 = 2;
const ARM_BASE: i16 = 0x1830;
const WANDER_SPAN_A: f32 = 3.0;
const WANDER_SCALE_A: f32 = 4096.0;
const WANDER_SPAN_B: f32 = 5.0;
const WANDER_TURN: i16 = 0x3000;
const WANDER_BASE: i16 = 0x5000;

const LIGHT_START: u8 = 1;
const LIGHT_SHRINK_END: u8 = 70;
const LIGHT_GONE: u8 = 90;
const LIGHT_DEAD: i32 = -0x80;
const LIGHT_SCALE_XZ: f32 = 0.1;
const LIGHT_SCALE_MAX: f32 = 0.01;
const LIGHT_RISE_TARGET: f32 = 5.0;
const LIGHT_RISE_SCALE: f32 = 0.1;
const LIGHT_RISE_MAX: f32 = 1.0;
const LIGHT_FADE_SCALE: f32 = 0.1;
const LIGHT_FADE_MAX: f32 = 0.05;
const FREEZE_DAMAGE: i32 = 4;

const PARRY_NONE: u8 = 0;
const PARRY_JUMP: u8 = 1;
const PARRY_VERTICAL: u8 = 3;
const PARRY_REACH: f32 = 700.0;

const AT_TYPE_ALL: u32 = u32::MAX;
const AT_TYPE_UNK8: u32 = 1 << 3;
const AT_TYPE_UNK800: u32 = 1 << 11;
const AT_TYPE_WATER: u32 = 1 << 8;
const AT_TYPE_UNK20000: u32 = 1 << 17;
const AT_TYPE_WIND: u32 = 1 << 21;
const AT_TYPE_UNK400000: u32 = 1 << 22;
const AT_TYPE_LIGHT: u32 = 1 << 23;

const SPH_SRC_RADIUS: f32 = 60.0;
const CYL_SRC_RADIUS: f32 = 60.0;
const CYL_HEIGHT: f32 = 150.0;
const SPH_TG_TYPE: u32 = AT_TYPE_ALL
    & !AT_TYPE_UNK8
    & !AT_TYPE_WATER
    & !AT_TYPE_UNK20000
    & !AT_TYPE_UNK400000;
const CYL_TG_TYPE: u32 = AT_TYPE_ALL
    & !AT_TYPE_WATER
    & !AT_TYPE_UNK20000
    & !AT_TYPE_WIND
    & !AT_TYPE_UNK400000
    & !AT_TYPE_LIGHT;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodySph {
    pub centre: Vec3,
    pub radius: f32,
    pub registered: bool,
    pub at_type: u32,
    pub tg_type: u32,
    pub tg_set: bool,
    pub tg_vs_other: bool,
    pub tg_hit: bool,
    pub co_set: bool,
    pub co_vs_enemy: bool,
    pub co_no_crr: bool,
}

impl BodySph {
    const SRC: BodySph = BodySph {
        centre: Vec3::ZERO,
        radius: SPH_SRC_RADIUS,
        registered: false,
        at_type: 0,
        tg_type: SPH_TG_TYPE,
        tg_set: true,
        tg_vs_other: true,
        tg_hit: false,
        co_set: true,
        co_vs_enemy: true,
        co_no_crr: true,
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoleCyl {
    pub centre: Vec3,
    pub radius: f32,
    pub height: f32,
    pub registered: bool,
    pub at_type: u32,
    pub tg_type: u32,
    pub tg_set: bool,
    pub co_set: bool,
    pub co_vs_enemy: bool,
    pub co_no_crr: bool,
}

impl HoleCyl {
    const SRC: HoleCyl = HoleCyl {
        centre: Vec3::ZERO,
        radius: CYL_SRC_RADIUS,
        height: 0.0,
        registered: false,
        at_type: AT_TYPE_UNK800,
        tg_type: CYL_TG_TYPE,
        tg_set: false,
        co_set: true,
        co_vs_enemy: true,
        co_no_crr: true,
    };
}

const EVENT_APPEAR: &str = "DEFAULT_FM_SW_APEEAR";
const EVENT_GRAB: &str = "DEFAULT_FM_GRAB";
const EVENT_GRAB_FOOT: &str = "DEFAULT_FM_GRAB_FOOT";
const EVENT_GRAB_NPC: &str = "DEFAULT_FM_NPC_GRAB";
const EVENT_SUIKOMI: &str = "DEFAULT_FM_SUIKOMI_NPC";

const CUT_FRAME_PULL_GRAB: f32 = 15.0;
const CUT_FRAME_PULL_FOOT: f32 = 25.0;
const CUT_FRAME_HAND: f32 = 10.0;
const CUT_FRAME_END: f32 = 40.0;
const CUT_REACH: f32 = 3.0;
const CUT_TWEEN: f32 = 0.3;
const CUT_TWEEN_STEP: f32 = 10.0;

const REG12_S2: i16 = 0;
const REG12_S3: i16 = 0;
const REG12_F0: f32 = 0.0;

fn frame_max(bck: i32) -> i16 {
    match bck {
        BCK_WAIT => FRAME_WAIT,
        BCK_DERU => FRAME_DERU,
        BCK_MODORU => FRAME_MODORU,
        BCK_TUKAMU => FRAME_TUKAMU,
        BCK_TUKAMU2WAIT => FRAME_TUKAMU2WAIT,
        BCK_HIKIKOMU => FRAME_HIKIKOMU,
        BCK_DAMAGE => FRAME_DAMAGE,
        BCK_TUMAMIWAIT => FRAME_TUMAMIWAIT,
        BCK_ASITUKAMI => FRAME_ASITUKAMI,
        BCK_NAGERU => FRAME_NAGERU,
        BCK_YAYU => FRAME_YAYU,
        BCK_MAHI => FRAME_MAHI,
        BCK_KUSIZASI => FRAME_KUSIZASI,
        _ => FRAME_WAIT,
    }
}

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = (seed[0]) as f32 / RND_SCALE[0]
        + (seed[1]) as f32 / RND_SCALE[1]
        + (seed[2]) as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

fn rnd_f(seed: &mut [i32; 3], range: f32) -> f32 {
    rnd(seed) * range
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

fn add_calc_pos_xz2(pos: &mut Vec3, target: Vec3, scale: f32, max_step: f32) {
    if pos.x != target.x || pos.z != target.z {
        let mut diff = (*pos - target) * scale;
        let step = (diff.x * diff.x + diff.z * diff.z).sqrt();
        if step != 0.0 {
            if step > max_step {
                diff *= max_step / step;
            }
            pos.x -= diff.x;
            pos.z -= diff.z;
        }
    }
}

fn distance_angle_s(x: i16, y: i16) -> i32 {
    i32::from(x.wrapping_sub(y)).abs()
}

fn bearing(from: Vec3, to: Vec3) -> i16 {
    walk::to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn turned(yaw: i16, offset: [f32; 3]) -> Vec3 {
    let (sin, cos) = walk::from_s16(yaw).sin_cos();
    Vec3::new(
        offset[0] * cos + offset[2] * sin,
        offset[1],
        offset[2] * cos - offset[0] * sin,
    )
}

fn flat(from: Vec3, to: Vec3) -> f32 {
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    (dx * dx + dz * dz).sqrt()
}

fn touches(centre: Vec3, radius: f32, at: Vec3, reach: f32) -> bool {
    walk::within(centre, at, radius + reach)
}

fn timer(value: &mut i32) -> bool {
    if *value != 0 {
        *value -= 1;
    }
    *value == 0
}

fn slide_and_hit(solid: Option<&Collision>, from: Vec3, to: Vec3) -> (Vec3, bool) {
    match solid {
        Some(solid) => {
            let next = solid.slide(from, to);
            let hit = next.x != to.x || next.z != to.z;
            (next, hit)
        }
        None => (to, false),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    SwWait,
    Hide,
    UnderFoot,
    PathMove,
    GoalKeeper,
    Appear,
    Disappear,
    Wait,
    Attack,
    Throw,
    Damage,
    PrepareItem,
    Death,
    Grab,
    GrabDemo,
    GrabFootDemo,
    GrabNpcDemo,
    PlayerStartDemo,
    Delete,
    Paralysis,
    Bikubiku,
}

const PHASES: [Phase; 21] = [
    Phase::SwWait,
    Phase::Hide,
    Phase::UnderFoot,
    Phase::PathMove,
    Phase::GoalKeeper,
    Phase::Appear,
    Phase::Disappear,
    Phase::Wait,
    Phase::Attack,
    Phase::Throw,
    Phase::Damage,
    Phase::PrepareItem,
    Phase::Death,
    Phase::Grab,
    Phase::GrabDemo,
    Phase::GrabFootDemo,
    Phase::GrabNpcDemo,
    Phase::PlayerStartDemo,
    Phase::Delete,
    Phase::Paralysis,
    Phase::Bikubiku,
];

impl Phase {
    pub fn code(self) -> u8 {
        match self {
            Self::SwWait => 0,
            Self::Hide => 1,
            Self::UnderFoot => 2,
            Self::PathMove => 3,
            Self::GoalKeeper => 4,
            Self::Appear => 5,
            Self::Disappear => 6,
            Self::Wait => 7,
            Self::Attack => 8,
            Self::Throw => 9,
            Self::Damage => 10,
            Self::PrepareItem => 11,
            Self::Death => 12,
            Self::Grab => 13,
            Self::GrabDemo => 14,
            Self::GrabFootDemo => 15,
            Self::GrabNpcDemo => 16,
            Self::PlayerStartDemo => 17,
            Self::Delete => 18,
            Self::Paralysis => 19,
            Self::Bikubiku => 20,
        }
    }

    fn lock_off(self) -> bool {
        matches!(
            self,
            Self::SwWait
                | Self::Hide
                | Self::UnderFoot
                | Self::PathMove
                | Self::GoalKeeper
                | Self::Disappear
                | Self::PrepareItem
                | Self::Death
                | Self::GrabDemo
                | Self::GrabFootDemo
                | Self::GrabNpcDemo
                | Self::PlayerStartDemo
                | Self::Delete
        )
    }

    pub fn by_code(code: u8) -> Phase {
        PHASES
            .get(usize::from(code))
            .copied()
            .unwrap_or(Phase::SwWait)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum At {
    Idle,
    Sword,
    Boomerang,
    Hookshot,
    Grapple,
    Wind,
    Arrow,
    FireArrow,
    IceArrow,
    LightArrow,
    Hammer,
    Bomb,
}

impl At {
    fn reach(self) -> f32 {
        match self {
            Self::Idle => 0.0,
            Self::Sword => REACH_SWORD,
            Self::Boomerang => REACH_BOOMERANG,
            Self::Hookshot => REACH_HOOKSHOT,
            Self::Grapple | Self::Wind | Self::LightArrow => 0.0,
            Self::Arrow | Self::FireArrow | Self::IceArrow => REACH_ARROW,
            Self::Hammer => REACH_HAMMER,
            Self::Bomb => bombflower::BLAST_RADIUS,
        }
    }

    fn power(self, blade: u8, soup: bool) -> i32 {
        match self {
            Self::Sword => damage::sword_power(blade, soup),
            Self::Arrow | Self::FireArrow | Self::IceArrow => damage::ARROW,
            Self::Bomb => damage::BOMB,
            Self::Boomerang => damage::BOMERANG,
            Self::Hammer => hammer::AT_POWER,
            Self::Idle
            | Self::Hookshot
            | Self::Grapple
            | Self::Wind
            | Self::LightArrow => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blow {
    pub at: At,
    pub blade: u8,
    pub soup: bool,
    pub hammer_side: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub attribute: u8,
    pub state: u8,
    pub start: i16,
    pub end: i16,
    pub loop_point: i16,
    pub rate: f32,
    pub frame: f32,
    pub cur_morf: f32,
    pub prev_morf: f32,
    pub morf_step: f32,
}

impl Frame {
    fn idle() -> Self {
        Self {
            attribute: EMODE_LOOP,
            state: 0,
            start: 0,
            end: 0,
            loop_point: 0,
            rate: 1.0,
            frame: 0.0,
            cur_morf: 1.0,
            prev_morf: 1.0,
            morf_step: 0.0,
        }
    }

    fn init(&mut self, end: i16) {
        self.attribute = EMODE_LOOP;
        self.state = 0;
        self.start = 0;
        self.end = end;
        self.loop_point = 0;
        self.rate = 1.0;
        self.frame = 0.0;
    }

    fn set_loop(&mut self, frame: f32) {
        self.loop_point = frame as i16;
    }

    fn set_morf(&mut self, morf: f32) {
        if self.prev_morf < 0.0 || morf <= 0.0 {
            self.cur_morf = 1.0;
        } else {
            self.cur_morf = 0.0;
            self.morf_step = 1.0 / morf;
        }
        self.prev_morf = self.cur_morf;
    }

    fn update(&mut self) {
        self.state = 0;
        self.frame += self.rate;
        let start = (self.start) as f32;
        let end = (self.end) as f32;
        match self.attribute {
            EMODE_NONE => {
                if self.frame < start {
                    self.frame = start;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
                if self.frame >= end {
                    self.frame = end - 0.001;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
            }
            EMODE_LOOP => {
                let low = i32::from(self.loop_point) - i32::from(self.start);
                let high = i32::from(self.end) - i32::from(self.loop_point);
                while self.frame < start {
                    self.state |= STATE_LOOP;
                    if low <= 0 {
                        break;
                    }
                    self.frame += (low) as f32;
                }
                while self.frame >= end {
                    self.state |= STATE_LOOP;
                    if high <= 0 {
                        break;
                    }
                    self.frame -= (high) as f32;
                }
            }
            _ => {}
        }
    }

    fn play(&mut self) {
        if self.cur_morf < 1.0 {
            self.prev_morf = self.cur_morf;
            walk::chase_f(&mut self.cur_morf, 1.0, self.morf_step);
        }
        self.update();
    }

    fn is_stop(&self) -> bool {
        self.state & STATE_STOP != 0 || self.rate == 0.0
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Floormaster {
    pub next_stage: u8,
    pub point_restart: i16,
    pub kind: u8,
    pub target_kind: u8,
    pub path_no: u8,
    pub switch_no: u8,
    pub kill_switch: u8,
    pub area_radius: f32,
    pub demo_mode: bool,
    pub boss_won: bool,
    pub caught: bool,
    pub phase: Phase,
    pub morph: Frame,
    pub anm_prm: u8,
    pub anm_bck: i32,
    pub old_anm_prm: u8,
    pub hit_type: u8,
    pub lock_on: bool,
    pub steal_left: i16,
    pub health: i32,
    pub max_health: i32,
    pub base: bool,
    pub base_angle: i16,
    pub base_dist: f32,
    pub home_pos: Vec3,
    pub body_offset: Vec3,
    pub attention: Vec3,
    pub hand: Vec3,
    pub tip: Vec3,
    pub player_slot: Vec3,
    pub nearest: Vec3,
    pub wander_pos: Vec3,
    pub grab_pos: Vec3,
    pub move_dest: Vec3,
    pub move_speed: f32,
    pub hole_scale: f32,
    pub wander_yaw: i16,
    pub arm_timer: i32,
    pub arm_phase: i16,
    pub arm_sway: i16,
    pub carry_kind: u8,
    pub target: Option<Entity>,
    pub target_flag: u8,
    pub target_sub: u8,
    pub spawn_item: Option<Entity>,
    pub release: Option<Release>,
    pub player_busy: bool,
    pub retreat_timer: i32,
    pub mode_timer: i32,
    pub far_timer: i32,
    pub grab_timer: i32,
    pub sink_timer: i32,
    pub hit_timer: i32,
    pub in_area: u8,
    pub line_clear: u8,
    pub grab_demo_link: bool,
    pub freeze: i32,
    pub freeze_want: i32,
    pub light: u8,
    pub light_scale_xz: f32,
    pub light_scale_y: f32,
    pub parry_height: f32,
    pub parry_radius: f32,
    pub parry_max_dis: f32,
    pub parry_start: f32,
    pub parry_end: f32,
    pub parry_now: f32,
    pub parry_kind: u8,
    pub body_sph: BodySph,
    pub hole_cyl: HoleCyl,
    pub entered: bool,
    pub doomed: bool,
    pub killed: bool,
    pub wall_hit: bool,
    pub seed: [i32; 3],
}

impl Floormaster {
    pub fn spawn(parameters: u32) -> Self {
        let mut kind = ((parameters >> KIND_SHIFT) & KIND_MASK) as u8;
        let mut target_kind = ((parameters >> TARGET_SHIFT) & TARGET_MASK) as u8;
        if kind == KIND_THREE {
            kind = 0;
        }
        if target_kind == TARGET_THREE {
            target_kind = 0;
        }
        let mut morph = Frame::idle();
        morph.init(frame_max(BCK_ASITUKAMI));
        morph.attribute = EMODE_NONE;
        morph.rate = 0.0;
        Self {
            next_stage: (parameters & BYTE_MASK) as u8,
            point_restart: 0,
            kind,
            target_kind,
            path_no: ((parameters >> PATH_SHIFT) & PATH_MASK) as u8,
            switch_no: ((parameters >> SWITCH_SHIFT) & BYTE_MASK) as u8,
            kill_switch: 0,
            area_radius: RADIUS_FIXED,
            demo_mode: false,
            boss_won: false,
            caught: false,
            phase: Phase::SwWait,
            morph,
            anm_prm: 0,
            anm_bck: BCK_WAIT,
            old_anm_prm: 0,
            hit_type: HIT_NONE,
            lock_on: false,
            steal_left: STEAL_ITEMS,
            health: HIO_MAX_HEALTH,
            max_health: HIO_MAX_HEALTH,
            base: false,
            base_angle: 0,
            base_dist: 0.0,
            home_pos: Vec3::ZERO,
            body_offset: Vec3::from_array(HIO_BODY_OFFSET),
            attention: Vec3::ZERO,
            hand: Vec3::ZERO,
            tip: Vec3::ZERO,
            player_slot: Vec3::ZERO,
            nearest: Vec3::ZERO,
            wander_pos: Vec3::ZERO,
            grab_pos: Vec3::ZERO,
            move_dest: Vec3::ZERO,
            move_speed: 0.0,
            hole_scale: 0.0,
            wander_yaw: 0,
            arm_timer: 0,
            arm_phase: 0,
            arm_sway: 0,
            carry_kind: CARRY_NONE,
            target: None,
            target_flag: NO_TARGET,
            target_sub: 0,
            spawn_item: None,
            release: None,
            player_busy: false,
            retreat_timer: 0,
            mode_timer: 0,
            far_timer: 0,
            grab_timer: 0,
            sink_timer: 0,
            hit_timer: 0,
            in_area: 0,
            line_clear: 0,
            grab_demo_link: false,
            freeze: 0,
            freeze_want: 0,
            light: 0,
            light_scale_xz: 1.0,
            light_scale_y: 1.0,
            parry_height: HIO_SP_VJUMP_HEIGHT,
            parry_radius: HIO_SP_VJUMP_RADIUS,
            parry_max_dis: PARRY_REACH,
            parry_start: HIO_SP_VJUMP_START,
            parry_end: HIO_SP_VJUMP_END,
            parry_now: 0.0,
            parry_kind: PARRY_NONE,
            body_sph: BodySph::SRC,
            hole_cyl: HoleCyl::SRC,
            entered: false,
            doomed: false,
            killed: false,
            wall_hit: false,
            seed: [RND_SEED; 3],
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.hole_scale > HOLE_GATE || self.is_body_appear()
    }

    pub fn hull(&self) -> ([f32; 3], [f32; 3]) {
        (
            [
                self.body_sph.centre.x - self.body_sph.radius,
                self.body_sph.centre.y - self.body_sph.radius,
                self.body_sph.centre.z - self.body_sph.radius,
            ],
            [
                self.body_sph.centre.x + self.body_sph.radius,
                self.body_sph.centre.y + self.body_sph.radius,
                self.body_sph.centre.z + self.body_sph.radius,
            ],
        )
    }

    pub fn read_args(&mut self, actor: &mut Actor) {
        let params = actor.parameters;
        let home_x = actor.angle[0];
        let home_y = actor.angle[1];
        let home_z = actor.angle[2];
        self.next_stage = (params & BYTE_MASK) as u8;
        self.kind = ((params >> KIND_SHIFT) & KIND_MASK) as u8;
        self.target_kind = ((params >> TARGET_SHIFT) & TARGET_MASK) as u8;
        self.path_no = ((params >> PATH_SHIFT) & PATH_MASK) as u8;
        self.switch_no = ((params >> SWITCH_SHIFT) & BYTE_MASK) as u8;
        self.kill_switch = (home_x & BYTE_MASK as i16) as u8;
        self.point_restart = ((((home_x as u32) >> HOME_X_SHIFT) & HOME_X_MASK) as u16) as i16;
        let band = (home_z as u32) & BYTE_MASK;
        self.area_radius = if band as u32 == BYTE_MASK {
            0.0
        } else {
            band as f32 * RADIUS_STEP
        };
        if walk::from_s16(home_y) != 0.0 && self.switch_no != NO_BYTE {
            self.demo_mode = true;
        }
        if self.kind == KIND_THREE {
            self.kind = 0;
        }
        if self.target_kind == TARGET_THREE {
            self.target_kind = 0;
        }
        if self.area_radius == 0.0 {
            self.area_radius = RADIUS_FIXED;
        }
        actor.angle = [0, home_y, 0];
        actor.shape_angle[1] = home_y;
    }

    pub fn is_body_appear(&self) -> bool {
        !matches!(
            self.phase,
            Phase::Hide
                | Phase::Delete
                | Phase::SwWait
                | Phase::UnderFoot
                | Phase::PathMove
                | Phase::GoalKeeper
                | Phase::PlayerStartDemo
        )
    }

    pub fn is_hole_appear(&self) -> bool {
        self.hole_scale > HOLE_GATE
    }

    pub fn set_hole_scale(&mut self, target: f32, scale: f32, max_step: f32) -> bool {
        add_calc2(&mut self.hole_scale, target, scale, max_step);
        (self.hole_scale - target).abs() <= max_step
    }

    fn rnd_f(&mut self, range: f32) -> f32 {
        rnd_f(&mut self.seed, range)
    }

    fn set_rnd(&mut self, low: i32, high: i32) -> i32 {
        low + (self.rnd_f(((high - low) as f32).abs()) as i32)
    }

    fn set_anm(&mut self, prm: u8, force: bool) {
        if prm != ANM_NONE {
            self.anm_prm = prm;
        }
        let index = usize::from(self.anm_prm).min(PRM_COUNT - 1);
        if (self.old_anm_prm != self.anm_prm && ANM_INDEX[index] != ANM_NO_INDEX) || force {
            self.anm_bck = ANM_BCK[index];
            self.morph.init(frame_max(self.anm_bck));
            self.morph.attribute = ANM_LOOP[index];
            self.morph.rate = ANM_SPEED[index];
            self.morph.frame = 0.0;
            self.morph.set_loop(self.morph.frame);
            self.morph.set_morf(ANM_MORF[index]);
        }
        self.old_anm_prm = self.anm_prm;
        if self.morph.is_stop() && ANM_NEXT[index] != ANM_NO_NEXT && ANM_LOOP[index] == EMODE_NONE {
            self.anm_prm = ANM_NEXT[index] as u8;
        }
    }

    fn arm_wobble(&mut self) {
        if !timer(&mut self.arm_timer) {
            return;
        }
        self.arm_phase = self
            .arm_phase
            .wrapping_add(ARM_RATE)
            .wrapping_add(ARM_BASE)
            .wrapping_add(REG12_S2);
        stream::add_calc_angle_s2(&mut self.arm_sway, 0, ARM_DIV, ARM_CAP);
    }

    fn commit_anm(&mut self) {
        self.set_anm(ANM_NONE, false);
    }

    pub fn is_link(&self) -> bool {
        self.target_flag == TARGET_PLAYER
    }

    fn is_npc(&self) -> bool {
        self.target_flag == TARGET_NPC
    }

    fn is_link_control(&self) -> bool {
        false
    }

    pub fn mode_proc_init(&mut self, phase: Phase, actor: &mut Actor, player: &Player) {
        self.phase = phase;
        self.lock_on = !phase.lock_off();
        match phase {
            Phase::SwWait => self.mode_sw_wait_init(),
            Phase::Hide => self.mode_hide_init(),
            Phase::UnderFoot => self.mode_under_foot_init(actor, player),
            Phase::PathMove => self.mode_path_move_init(),
            Phase::GoalKeeper => self.mode_goal_keeper_init(),
            Phase::Appear => self.mode_appear_init(actor),
            Phase::Disappear => self.mode_disappear_init(),
            Phase::Wait => self.mode_wait_init(actor),
            Phase::Attack => self.mode_attack_init(actor, player),
            Phase::Throw => self.mode_throw_init(),
            Phase::Damage => self.mode_damage_init(),
            Phase::PrepareItem => self.mode_prepare_item_init(),
            Phase::Death => self.mode_death_init(),
            Phase::Grab => self.mode_grab_init(),
            Phase::GrabDemo => self.mode_grab_demo_init(),
            Phase::GrabFootDemo => self.mode_grab_foot_demo_init(player),
            Phase::GrabNpcDemo => self.mode_grab_npc_demo_init(),
            Phase::PlayerStartDemo => self.mode_player_start_demo_init(player),
            Phase::Delete => self.mode_delete_init(),
            Phase::Paralysis => self.mode_paralysis_init(),
            Phase::Bikubiku => self.mode_bikubiku_init(),
        }
    }

    fn mode_sw_wait_init(&mut self) {
        self.set_anm(8, false);
        self.hole_scale = HIO_120;
    }

    fn mode_hide_init(&mut self) {
        self.set_anm(8, false);
        self.mode_timer = self.set_rnd(i32::from(HIO_084), i32::from(HIO_086));
    }

    fn mode_under_foot_init(&mut self, actor: &mut Actor, player: &Player) {
        self.set_anm(8, false);
        self.mode_timer = i32::from(HIO_066);
        actor.pos.x = player.pos.x;
        actor.pos.z = player.pos.z;
        self.hole_scale = HIO_120;
    }

    fn mode_path_move_init(&mut self) {
        self.set_anm(8, false);
        self.hole_scale = HIO_124;
    }

    fn mode_goal_keeper_init(&mut self) {
        self.set_anm(8, false);
        self.hole_scale = HIO_124;
    }

    fn mode_appear_init(&mut self, actor: &mut Actor) {
        self.set_anm(2, false);
        self.move_dest = actor.pos;
        actor.shape_angle[1] = self.base_angle;
        self.mode_timer = 30;
    }

    fn mode_disappear_init(&mut self) {
        if self.anm_prm != 8 && self.anm_prm != 6 {
            self.set_anm(3, false);
        }
        if self.anm_prm == 6 {
            self.set_anm(8, false);
        }
    }

    fn mode_wait_init(&mut self, actor: &Actor) {
        self.far_timer = i32::from(HIO_088);
        self.mode_timer = self.set_rnd(i32::from(HIO_08C), i32::from(HIO_08E));
        self.sink_timer = i32::from(HIO_08A);
        self.grab_timer = i32::from(HIO_100);
        self.move_dest = actor.pos;
    }

    fn mode_attack_init(&mut self, actor: &mut Actor, player: &Player) {
        if self.player_busy {
            self.mode_proc_init(Phase::Wait, actor, player);
        } else {
            self.set_anm(4, false);
            self.mode_timer = i32::from(HIO_094);
        }
    }

    fn mode_throw_init(&mut self) {
        self.set_anm(0xB, false);
    }

    fn mode_damage_init(&mut self) {
        self.set_anm(7, true);
        self.mode_timer = i32::from(HIO_0A6);
    }

    fn mode_prepare_item_init(&mut self) {
        self.set_anm(3, false);
        self.mode_timer = i32::from(HIO_092);
        self.cancel_grab();
    }

    fn mode_death_init(&mut self) {
        self.set_anm(3, false);
    }

    fn mode_grab_init(&mut self) {
        self.sink_timer = if self.is_link() {
            i32::from(HIO_090)
        } else {
            i32::from(HIO_09E)
        };
        self.mode_timer = i32::from(HIO_092);
        if self.is_npc() {
            self.set_anm(0xC, false);
        } else if self.target_flag == TARGET_BOMB || self.target_flag == TARGET_TSUBO {
            self.set_anm(9, false);
        }
        self.move_dest = self.wander_pos;
    }

    fn mode_grab_demo_init(&mut self) {
        self.set_anm(6, false);
        self.grab_demo_link = self.is_link();
    }

    fn mode_grab_foot_demo_init(&mut self, player: &Player) {
        self.body_offset = Vec3::ZERO;
        self.player_slot = player.pos;
    }

    fn mode_grab_npc_demo_init(&mut self) {
        self.set_anm(9, false);
    }

    fn mode_player_start_demo_init(&mut self, player: &Player) {
        self.hole_scale = HIO_120;
        self.player_slot = player.pos;
    }

    fn mode_delete_init(&mut self) {}

    fn mode_paralysis_init(&mut self) {
        self.set_anm(0xD, true);
        self.mode_timer = i32::from(HIO_11C);
    }

    fn mode_bikubiku_init(&mut self) {
        self.mode_timer = i32::from(HIO_068);
        self.sink_timer = 5;
        self.set_anm(0xE, true);
        self.mode_timer = i32::from(HIO_068);
    }

    fn set_base_target(&mut self, actor: &Actor, player: &Player) {
        self.base = match self.target_kind {
            1 => true,
            2 => false,
            _ => true,
        };
        if self.base {
            self.base_angle = bearing(actor.pos, player.pos);
            self.base_dist = flat(actor.pos, player.pos);
        } else {
            self.base_angle = 0;
            self.base_dist = 0.0;
        }
    }

    fn turn_to_base_target(&mut self, actor: &mut Actor) {
        stream::add_calc_angle_s2(
            &mut actor.shape_angle[1],
            self.base_angle,
            TURN_DIV,
            TURN_CAP,
        );
    }

    fn check_height_actor(&self, pos: Vec3) -> bool {
        (pos.y - self.home_pos.y).abs() <= HIO_0A8 / 2.0
    }

    fn line_check(&self, from: Vec3, to: Vec3, solid: Option<&Collision>) -> (bool, Vec3) {
        match solid {
            Some(solid) => {
                let clamped = solid.slide(from, to);
                if clamped != to {
                    (true, clamped)
                } else {
                    (false, to)
                }
            }
            None => (false, to),
        }
    }

    fn area_check(&mut self, actor: &Actor, solid: Option<&Collision>) -> bool {
        let mut blocked = [false; AREA_RAYS];
        let mut ok = true;
        for index in 0..AREA_RAYS {
            let Some(slot) = blocked.get_mut(index) else {
                break;
            };
            let probe = turned(
                actor.shape_angle[1].wrapping_add(AREA_STEP.wrapping_mul(index as i16)),
                [HIO_0E0, AREA_DOWN, 0.0],
            ) + actor.pos;
            if self.kind != 2 {
                if flat(probe, self.home_pos) > self.area_radius {
                    *slot = false;
                    ok = false;
                } else {
                    *slot = true;
                }
            }
            if *slot || self.kind == 2 {
                let from = Vec3::new(probe.x, probe.y + AREA_PROBE, probe.z);
                let (hit, _) = self.line_check(from, probe, solid);
                *slot = hit;
                if !hit {
                    ok = false;
                }
            }
        }
        ok
    }

    fn offset_pos(&self) -> Vec3 {
        if self.is_link() {
            return Vec3::new(0.0, -10.0, 0.0);
        }
        match self.target_sub {
            TSUBO_POT | TSUBO_SKULL => Vec3::new(0.0, 80.0, 0.0),
            TSUBO_STOOL | TSUBO_PAIL => Vec3::new(0.0, 80.0, 0.0),
            TSUBO_BIG | TSUBO_JUG => Vec3::new(0.0, 90.0, 0.0),
            _ => Vec3::ZERO,
        }
    }

    fn set_grab_pos(&mut self, ctx: &Ctx<'_, '_>) {
        if self.target_flag == NO_TARGET {
            self.grab_pos = ctx.actor_pos;
            return;
        }
        if self.is_link() {
            self.grab_pos = ctx.player.pos + Vec3::new(0.0, HEAD_TOP, 0.0) + self.offset_pos();
        } else {
            self.grab_pos = ctx.target_pos + self.offset_pos();
        }
    }

    fn is_grab_pos(&self, actor: &Actor, ctx: &Ctx<'_, '_>) -> bool {
        if self.target_flag == NO_TARGET {
            return false;
        }
        let gap = flat(actor.pos, self.grab_pos);
        if self.carry_kind != CARRY_NONE {
            if ctx.carry_bomb {
                return false;
            }
            if ctx.carry_held != Some(ctx.self_entity) {
                return false;
            }
        }
        if self.is_link() {
            if ctx.carry_held.is_some() {
                return false;
            }
            if self.is_link_control() {
                return false;
            }
            if gap <= HIO_0E0 {
                return false;
            }
        } else if gap <= HIO_0E0 || gap > HIO_0E4 {
            return false;
        }
        if (self.grab_pos.y - actor.pos.y).abs() > HIO_0A8 {
            return false;
        }
        if distance_angle_s(actor.shape_angle[1], bearing(actor.pos, self.grab_pos))
            > i32::from(HIO_098)
        {
            return false;
        }
        if self.kind == 0 && flat(self.home_pos, self.grab_pos) > self.area_radius {
            return false;
        }
        true
    }

    fn is_grab(&self, ctx: &Ctx<'_, '_>) -> bool {
        if self.target_flag == NO_TARGET {
            return false;
        }
        if self.is_link() && ctx.carry_bomb {
            return false;
        }
        if (self.grab_pos.y - ctx.actor_pos.y).abs() > HIO_0A8 {
            return false;
        }
        let limit = if self.demo_mode {
            HIO_0EC * 2.0
        } else {
            HIO_0EC
        };
        (self.grab_pos - self.hand).length() <= limit
    }

    fn is_grab_foot(&self, ctx: &Ctx<'_, '_>) -> bool {
        if (ctx.actor_pos.y - ctx.player.pos.y).abs() > FOOT_Y {
            return false;
        }
        if flat(ctx.actor_pos, ctx.player.pos) > HIO_0DC {
            return false;
        }
        if ctx.player.speed > FOOT_SPEED {
            return false;
        }
        if ctx.jumping || ctx.rolling || ctx.carry_held.is_some() || self.is_link_control() {
            return false;
        }
        true
    }

    fn player_grab_bomb(&self, ctx: &Ctx<'_, '_>) -> bool {
        ctx.carry_held.is_some() && ctx.carry_bomb
    }

    fn wander_yaw_step(&mut self, actor: &Actor, open: bool) {
        if open {
            let add = ((WANDER_BASE) as f32
                + (self.wander_yaw) as f32
                + self.rnd_f(WANDER_SPAN_A) * WANDER_SCALE_A) as i32 as i16;
            self.wander_yaw = self.wander_yaw.wrapping_add(add);
        } else {
            let value = f32::from(actor.shape_angle[1])
                + (WANDER_TURN) as f32
                + self.rnd_f(WANDER_SPAN_B) * WANDER_SCALE_A;
            self.wander_yaw = value as i32 as i16;
        }
    }

    fn move_rnd_back(&mut self, actor: &mut Actor, ctx: &Ctx<'_, '_>) {
        let wall = self.wall_hit;
        let mut reach = GRAB_LIMIT;
        if self.target_flag != NO_TARGET {
            reach = flat(actor.pos, ctx.target_pos);
        }
        if self.base_dist <= HIO_0E0 || reach <= HIO_0E0 {
            if !self.is_grab_foot(ctx)
                && (timer(&mut self.retreat_timer) || self.in_area == 0 || wall)
                && (self.in_area == 0 || wall)
            {
                self.wander_yaw_step(actor, true);
                self.move_speed = self.rnd_f(RANDOM_SPAN) + RANDOM_BASE;
                let (sin, cos) = walk::from_s16(self.wander_yaw).sin_cos();
                self.move_dest.z += HIO_0AC * cos;
                self.move_dest.x += HIO_0AC * sin;
                self.retreat_timer = i32::from(HIO_0A0);
            }
        }
        if self.in_area != 0 && !wall {
            add_calc_pos_xz2(&mut actor.pos, self.move_dest, RATE_DASH, HIO_0B0);
        }
    }

    fn move_rnd_escape(&mut self, actor: &mut Actor, ctx: &Ctx<'_, '_>) {
        let wall = self.wall_hit;
        let reach = flat(actor.pos, ctx.player.pos);
        if (timer(&mut self.retreat_timer) || self.in_area == 0 || wall) && reach < HIO_0B4 {
            self.wander_yaw_step(actor, self.in_area == 0 || wall);
            self.move_dest = self.wander_pos;
            let (sin, cos) = walk::from_s16(self.wander_yaw).sin_cos();
            self.move_dest.z += HIO_0B8 * cos;
            self.move_dest.x += HIO_0B8 * sin;
            self.retreat_timer = i32::from(HIO_0A0);
        }
        if (timer(&mut self.retreat_timer) || self.in_area == 0 || wall)
            && self.rnd_f(ESCAPE_SPAN) < ESCAPE_ROLL
        {
            self.set_anm(0xC, false);
        }
        if self.anm_prm != 0xC && self.in_area != 0 && !wall {
            add_calc_pos_xz2(
                &mut actor.pos,
                self.move_dest,
                (HIO_0CC + 1.0) * 0.005 * self.move_speed,
                HIO_0B0,
            );
            self.wander_pos = actor.pos;
        }
    }

    fn cancel_grab(&mut self) {
        if self.target_flag != NO_TARGET {
            self.carry_kind = CARRY_NONE;
        }
    }

    fn sp_attack_v_jump(&mut self) {
        self.parry_height = HIO_SP_VJUMP_HEIGHT;
        self.parry_radius = HIO_SP_VJUMP_RADIUS;
        self.parry_start = HIO_SP_VJUMP_START;
        self.parry_end = HIO_SP_VJUMP_END;
        self.parry_max_dis = PARRY_REACH;
        self.parry_kind = PARRY_VERTICAL;
        self.parry_now = self.morph.frame;
    }

    fn sp_attack_jump(&mut self) {
        self.parry_height = HIO_SP_JUMP_HEIGHT;
        self.parry_radius = HIO_SP_JUMP_RADIUS;
        self.parry_start = HIO_SP_JUMP_START;
        self.parry_end = HIO_SP_JUMP_END;
        self.parry_max_dis = PARRY_REACH;
        self.parry_kind = PARRY_JUMP;
        self.parry_now = self.morph.frame;
    }

    fn sp_attack_none(&mut self) {
        self.parry_start = 100.0;
        self.parry_end = 100.0;
        self.parry_max_dis = 10000.0;
        self.parry_kind = PARRY_NONE;
        self.parry_now = 0.0;
    }

    fn enemy_ice(&mut self, ctx: &mut Ctx<'_, '_>) -> bool {
        if self.light != 0 {
            if self.light == LIGHT_START {
                self.light = self.light.wrapping_add(1);
                ctx.se(SE_CM_L_ARROW_SHRINK);
                self.lock_on = false;
                return true;
            }
            self.light = self.light.wrapping_add(1);
            if self.light < LIGHT_SHRINK_END {
                add_calc0(&mut self.light_scale_xz, LIGHT_SCALE_XZ, LIGHT_SCALE_MAX);
                add_calc0(&mut self.light_scale_y, LIGHT_SCALE_XZ, LIGHT_SCALE_MAX);
            } else {
                if self.light == LIGHT_SHRINK_END {
                    ctx.se(SE_CM_L_ARROW_PASS);
                }
                add_calc2(
                    &mut self.light_scale_y,
                    LIGHT_RISE_TARGET,
                    LIGHT_RISE_SCALE,
                    LIGHT_RISE_MAX,
                );
                add_calc0(&mut self.light_scale_xz, LIGHT_FADE_SCALE, LIGHT_FADE_MAX);
                if self.light > LIGHT_GONE {
                    self.health = LIGHT_DEAD;
                }
            }
            return true;
        }
        if self.freeze_want != 0 {
            self.freeze = self.freeze_want;
            self.freeze_want = 0;
            self.health -= FREEZE_DAMAGE;
            ctx.se(SE_CM_FREEZE);
        }
        if self.freeze > 0 {
            self.freeze -= 1;
            return true;
        }
        false
    }

    fn ice_proc(&mut self, actor: &Actor) {
        self.set_attention(actor);
        if self.light != 0 || self.health <= 0 {
            self.set_hole_scale(HIO_120, HOLE_SCALE, HIO_128);
        }
        self.cancel_grab();
    }

    fn set_attention(&mut self, actor: &Actor) {
        if self.demo_mode
            || matches!(
                self.phase,
                Phase::GrabDemo
                    | Phase::GrabNpcDemo
                    | Phase::GrabFootDemo
                    | Phase::SwWait
                    | Phase::PlayerStartDemo
            )
        {
            self.attention = actor.pos;
            return;
        }
        self.attention = self.hand;
        self.attention.y += ATTENTION_LIFT;
    }

    fn set_collision(&mut self, actor: &Actor, busy: bool) {
        if self.is_body_appear() {
            self.body_sph.radius = if busy {
                HIO_0C8 * 1.5
            } else {
                HIO_0C8
            };
            self.body_sph.centre = self.hand;
            self.body_sph.registered = true;
        } else {
            self.body_sph.registered = false;
        }
        if self.is_hole_appear() {
            self.hole_cyl.radius = self.hole_scale;
            self.hole_cyl.height = CYL_HEIGHT;
            self.hole_cyl.centre = actor.pos;
            self.hole_cyl.registered = true;
        } else {
            self.hole_cyl.registered = false;
        }
    }

    fn calc_tg_hit(&mut self, ctx: &Ctx<'_, '_>, at: At) {
        let reach = at.reach();
        let hit = self.body_sph.tg_set
            && self.body_sph.registered
            && touches(
                self.body_sph.centre,
                self.body_sph.radius,
                ctx.player.pos,
                reach,
            );
        self.body_sph.tg_hit = hit;
    }

    fn search_target(&mut self, actor: &Actor, ctx: &mut Ctx<'_, '_>) {
        if ctx.event_running() && !self.demo_mode {
            return;
        }
        let mut found: Option<(Entity, u8, u8)> = None;
        for other in ctx.carryables.iter() {
            if !other.is_actor || !self.check_height_actor(other.pos) {
                continue;
            }
            if flat(actor.pos, other.pos) >= HIO_0E4 {
                continue;
            }
            match other.flag {
                TARGET_BOMB => {
                    self.target = Some(other.entity);
                    self.target_flag = TARGET_BOMB;
                    self.target_sub = 0;
                    found = Some((other.entity, TARGET_BOMB, 0));
                }
                TARGET_TSUBO => {
                    let near = flat(actor.pos, ctx.player.pos);
                    let reach = flat(other.pos, ctx.player.pos);
                    if near < HIO_0E8
                        && (reach > REG12_F0 + 80.0 || ctx.carry_held == Some(other.entity))
                        && matches!(other.sub, TSUBO_POT | TSUBO_BIG | TSUBO_JUG | TSUBO_STOOL | TSUBO_SKULL | TSUBO_PAIL)
                    {
                        self.target = Some(other.entity);
                        self.target_flag = TARGET_TSUBO;
                        self.target_sub = other.sub;
                        found = Some((other.entity, TARGET_TSUBO, other.sub));
                    }
                }
                _ => {}
            }
            if found.is_some() {
                return;
            }
        }
        if found.is_none() && self.target_kind != 2 && !ctx.player_hang && !ctx.bow_aim {
            if !self.player_grab_bomb(ctx) && ctx.carry_held.is_none() {
                if flat(actor.pos, ctx.player.pos) < HIO_0E4 || self.target_flag == NO_TARGET {
                    found = Some((ctx.player_entity, TARGET_PLAYER, 0));
                }
            }
        }
        match found {
            Some((entity, flag, sub)) => {
                self.target = Some(entity);
                self.target_flag = flag;
                self.target_sub = sub;
            }
            None => {
                self.target = None;
                self.target_flag = NO_TARGET;
                self.target_sub = 0;
            }
        }
    }

    fn check_tg_hit(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) -> bool {
        if !timer(&mut self.hit_timer) {
            return false;
        }
        if ctx.blow.at == At::Idle || !self.body_sph.tg_hit {
            return false;
        }
        self.hit_timer = i32::from(HIO_09C);
        let mut temp = true;
        match ctx.blow.at {
            At::Sword => {
                ctx.se(SE_LK_SW_HIT_S);
                self.hit_type = HIT_CUT;
            }
            At::Wind => {
                temp = false;
                self.hit_type = HIT_WIND;
            }
            At::Boomerang => {
                ctx.se(SE_LK_W_WEP_HIT);
                temp = false;
                self.hit_type = HIT_BOOMERANG;
                self.mode_proc_init(Phase::Paralysis, actor, ctx.player);
            }
            At::Hookshot => {
                ctx.se(SE_LK_HS_SPIKE);
                temp = false;
                self.hit_type = HIT_HOOKSHOT;
                self.mode_proc_init(Phase::Damage, actor, ctx.player);
            }
            At::Hammer => {
                ctx.se(SE_LK_HAMMER_HIT);
                self.hit_type = if ctx.blow.hammer_side {
                    HIT_HAMMER_TURN
                } else {
                    HIT_HAMMER
                };
            }
            At::Bomb => {
                self.hit_type = HIT_BOMB;
            }
            At::IceArrow => {
                temp = false;
                ctx.se(SE_LK_ARROW_HIT);
                self.freeze_want = HIO_FREEZE;
                self.hit_type = HIT_ICE;
                self.mode_proc_init(Phase::Wait, actor, ctx.player);
            }
            At::LightArrow => {
                temp = false;
                ctx.se(SE_LK_ARROW_HIT);
                self.light = LIGHT_START;
                self.hit_type = HIT_LIGHT;
            }
            At::Arrow | At::FireArrow => {
                ctx.se(SE_LK_ARROW_HIT);
                self.hit_type = HIT_ARROW;
            }
            At::Grapple => {
                ctx.se(SE_CV_FM_DAMAGE);
                ctx.se(SE_LK_W_WEP_HIT);
                self.hit_type = HIT_GRAPPLE;
                temp = false;
            }
            At::Idle => {}
        }
        let power = ctx.blow.at.power(ctx.blow.blade, ctx.blow.soup);
        if temp {
            if power > 0 {
                self.health -= power;
            }
            let strong = matches!(
                self.hit_type,
                HIT_CUT | HIT_HAMMER | HIT_HAMMER_TURN | HIT_JUMP_CUT
            ) || self.health <= 0;
            if strong {
                if self.health <= 0 {
                    ctx.se(SE_CV_FM_DIE);
                } else {
                    ctx.se(SE_CV_FM_DAMAGE);
                }
                if matches!(self.hit_type, HIT_JUMP_CUT | HIT_HAMMER) {
                    self.mode_proc_init(Phase::Bikubiku, actor, ctx.player);
                    return true;
                }
                if self.health <= 0 {
                    self.mode_proc_init(Phase::Death, actor, ctx.player);
                } else {
                    self.mode_proc_init(Phase::Damage, actor, ctx.player);
                }
            } else {
                ctx.se(SE_CV_FM_DAMAGE);
                self.mode_proc_init(Phase::Damage, actor, ctx.player);
            }
        } else if self.hit_type == HIT_GRAPPLE {
            self.mode_proc_init(Phase::Damage, actor, ctx.player);
            let old = self.health;
            self.health = GRAPPLE_HEALTH;
            if power > 0 {
                self.health -= power;
            }
            self.health = old;
            self.set_anm(1, false);
            self.arm_timer = ARM_TIMER;
            self.arm_sway = ARM_SWAY;
        }
        true
    }

    fn push_player(&self, ctx: &mut Ctx<'_, '_>, power: f32, angle: i16) {
        ctx.player.speed = power;
        ctx.player.yaw = walk::from_s16(angle);
        ctx.player.face = ctx.player.yaw;
    }

    fn hand_push(&self, ctx: &mut Ctx<'_, '_>) {
        let gap = flat(self.tip, self.grab_pos);
        if gap <= HIO_110 || gap >= HIO_114 {
            return;
        }
        let span = (HIO_114 - HIO_110).abs();
        let span = if span == 0.0 { 1.0 } else { span };
        let power = HIO_118 * ((gap - HIO_110).abs() / span);
        self.push_player(ctx, power, bearing(self.tip, self.grab_pos));
    }

    pub fn begin(&mut self, actor: &mut Actor, player: &Player, switches: &Switches) {
        self.entered = true;
        self.read_args(actor);
        if self.kill_switch != NO_BYTE && switches.test(self.kill_switch) {
            self.killed = true;
            return;
        }
        if self.demo_mode && self.switch_no != NO_BYTE && switches.test(self.switch_no) {
            self.killed = true;
            return;
        }
        self.grab_pos = actor.pos;
        self.home_pos = actor.pos;
        self.wander_pos = actor.pos;
        self.move_dest = actor.pos;
        self.nearest = Vec3::ZERO;
        self.set_anm(8, false);
        self.move_speed = self.rnd_f(RANDOM_SPAN) + RANDOM_BASE;
        self.body_offset = Vec3::from_array(HIO_BODY_OFFSET);
        if self.switch_no != NO_BYTE && !switches.test(self.switch_no) {
            self.mode_proc_init(Phase::SwWait, actor, player);
        } else {
            match self.kind {
                0 => self.mode_proc_init(Phase::UnderFoot, actor, player),
                1 => self.mode_proc_init(Phase::PathMove, actor, player),
                2 => self.mode_proc_init(Phase::GoalKeeper, actor, player),
                _ => {}
            }
        }
    }

    fn execute(
        &mut self,
        actor: &mut Actor,
        mut ctx: Ctx<'_, '_>,
        mut commands: &mut Commands,
    ) {
        self.player_busy = ctx.busy;
        self.hand = actor.pos + turned(actor.shape_angle[1], HIO_HAND_OFFSET);
        self.tip = self.hand + turned(actor.shape_angle[1], HIO_TIP_OFFSET);
        if self.is_hole_appear() {
            ctx.se(SE_SPOT);
        }
        if self.target_flag != NO_TARGET && self.target.is_none() {
            self.target_flag = NO_TARGET;
            self.target_sub = 0;
            self.carry_kind = CARRY_NONE;
            self.mode_proc_init(Phase::Wait, actor, ctx.player);
        }
        if self.enemy_ice(&mut ctx) {
            self.ice_proc(actor);
            self.morph.play();
            return;
        }
        self.set_base_target(actor, ctx.player);
        if !self.base {
            self.set_attention(actor);
            self.morph.play();
            return;
        }
        self.calc_tg_hit(&ctx, ctx.blow.at);
        let from = actor.pos;
        let in_area = self.area_check(actor, ctx.solid);
        self.in_area = u8::from(in_area);
        self.line_clear = 0;
        let raised = actor.pos + Vec3::new(0.0, LINE_LIFT, 0.0);
        let far = ctx.player.pos + Vec3::new(0.0, LINE_LIFT, 0.0);
        if self.base_dist < HIO_0E8 && self.line_check(raised, far, ctx.solid).0 {
            self.line_clear = 1;
        }
        if self.demo_mode {
            self.line_clear = 0;
        }
        if self.target_flag != NO_TARGET {
            if self.phase == Phase::Attack && self.anm_prm == 4 && self.is_link() {
                self.sp_attack_v_jump();
            } else if self.anm_prm == 0xB || self.anm_prm == 0xC {
                self.sp_attack_jump();
            } else {
                self.sp_attack_none();
            }
        }
        self.arm_wobble();
        match self.phase {
            Phase::SwWait => self.mode_sw_wait(actor, &mut ctx),
            Phase::Hide => self.mode_hide(actor, &mut ctx),
            Phase::UnderFoot => self.mode_under_foot(actor, &mut ctx),
            Phase::PathMove => self.mode_path_move(actor, &mut ctx),
            Phase::GoalKeeper => self.mode_goal_keeper(actor, &mut ctx),
            Phase::Appear => self.mode_appear(actor, &mut ctx),
            Phase::Disappear => self.mode_disappear(actor, &mut ctx),
            Phase::Wait => self.mode_wait(actor, &mut ctx),
            Phase::Attack => self.mode_attack(actor, &mut ctx),
            Phase::Throw => self.mode_throw(actor, &mut ctx),
            Phase::Damage => self.mode_damage(actor, &mut ctx),
            Phase::PrepareItem => self.mode_prepare_item(actor, &mut ctx, commands),
            Phase::Death => self.mode_death(actor, &mut commands),
            Phase::Grab => self.mode_grab(actor, &mut ctx),
            Phase::GrabDemo => self.mode_grab_demo(actor, &mut ctx),
            Phase::GrabFootDemo => self.mode_grab_foot_demo(actor, &mut ctx),
            Phase::GrabNpcDemo => self.mode_grab_npc_demo(actor, &mut ctx),
            Phase::PlayerStartDemo => self.mode_player_start_demo(actor, &mut ctx),
            Phase::Delete => self.mode_delete(actor, &mut ctx),
            Phase::Paralysis => self.mode_paralysis(actor, &mut ctx),
            Phase::Bikubiku => self.mode_bikubiku(actor, &mut ctx),
        }
        self.commit_anm();
        self.morph.play();
        self.move_dest.y = actor.pos.y;
        let (next, wall) = slide_and_hit(ctx.solid, from, actor.pos);
        actor.pos = next;
        self.wall_hit = wall;
        self.set_attention(actor);
        self.set_collision(actor, ctx.busy);
    }

    fn mode_sw_wait(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.switch_no == NO_BYTE || ctx.switches.test(self.switch_no) {
            if self.demo_mode {
                self.set_hole_scale(HIO_124, HOLE_SCALE, HIO_128);
                if !ctx.running(EVENT_APPEAR) {
                    ctx.order(EVENT_APPEAR);
                    return;
                }
            } else if ctx.running(EVENT_APPEAR) {
                self.boss_won = true;
            }
            match self.kind {
                0 => self.mode_proc_init(Phase::UnderFoot, actor, ctx.player),
                1 => {
                    if self.set_hole_scale(HIO_124, HOLE_SCALE, HIO_128) {
                        self.mode_proc_init(Phase::PathMove, actor, ctx.player);
                    }
                }
                2 => {
                    if self.set_hole_scale(HIO_124, HOLE_SCALE, HIO_128) {
                        self.mode_proc_init(Phase::GoalKeeper, actor, ctx.player);
                    }
                }
                _ => {}
            }
        }
    }

    fn mode_hide(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        self.wander_pos = actor.pos;
        if self.set_hole_scale(HIO_120, HOLE_SCALE, HIO_128) {
            actor.pos.x = ctx.player.pos.x;
            actor.pos.z = ctx.player.pos.z;
            if timer(&mut self.mode_timer)
                && (self.home_pos.y - ctx.player.pos.y).abs() <= HIO_0A8
                && self.in_area == 1
            {
                self.mode_proc_init(Phase::UnderFoot, actor, ctx.player);
            }
        }
    }

    fn mode_under_foot(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if timer(&mut self.retreat_timer) {
            self.move_dest = ctx.player.pos;
            self.retreat_timer = i32::from(HIO_08A);
        }
        add_calc_pos_xz2(&mut actor.pos, self.move_dest, RATE_DASH, HIO_0C0);
        self.wander_pos = actor.pos;
        if (self.home_pos.y - ctx.player.pos.y).abs() > HIO_0A8 || self.in_area == 0 {
            self.mode_proc_init(Phase::Hide, actor, ctx.player);
        } else if self.set_hole_scale(HIO_124, HOLE_SCALE, HIO_12C) {
            if timer(&mut self.mode_timer) {
                if self.is_grab_foot(ctx) && !ctx.event_running() {
                    self.mode_proc_init(Phase::GrabFootDemo, actor, ctx.player);
                } else if self.base_dist < HIO_0E4 && self.line_clear == 0 {
                    self.mode_proc_init(Phase::Appear, actor, ctx.player);
                } else {
                    self.mode_proc_init(Phase::Hide, actor, ctx.player);
                }
            }
        }
    }

    fn mode_path_move(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        self.wander_pos = actor.pos;
        self.turn_to_base_target(actor);
        if self.is_grab_foot(ctx) && !ctx.event_running() {
            self.mode_proc_init(Phase::GrabFootDemo, actor, ctx.player);
        } else if self.base_dist < HIO_0E4
            && (ctx.player.pos.y - actor.pos.y).abs() < HIO_0A8
            && self.line_clear == 0
        {
            self.mode_proc_init(Phase::Appear, actor, ctx.player);
        }
    }

    fn mode_goal_keeper(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.boss_won && ctx.event_done(EVENT_SUIKOMI) {
            self.mode_proc_init(Phase::Disappear, actor, ctx.player);
            return;
        }
        self.turn_to_base_target(actor);
        if self.is_grab_foot(ctx) && !ctx.event_running() {
            self.mode_proc_init(Phase::GrabFootDemo, actor, ctx.player);
        } else if self.demo_mode {
            self.mode_proc_init(Phase::Appear, actor, ctx.player);
        } else {
            if self.base_dist < HIO_0E8
                && (actor.pos.y - ctx.player.pos.y).abs() < HIO_0A8
                && flat(ctx.player.pos, self.home_pos) < self.area_radius
                && self.line_clear == 0
            {
                self.mode_proc_init(Phase::Appear, actor, ctx.player);
            }
            self.wander_pos = self.home_pos;
            add_calc_pos_xz2(
                &mut actor.pos,
                self.wander_pos,
                (HIO_0CC + 1.0) * 0.005 * self.move_speed,
                MOVE_CAP,
            );
        }
    }

    fn mode_appear(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.kind == 0 {
            self.move_rnd_back(actor, ctx);
        }
        if !self.check_tg_hit(actor, ctx) {
            self.search_target(actor, ctx);
            if self.target_flag != NO_TARGET {
                self.set_grab_pos(ctx);
                if self.demo_mode && !timer(&mut self.mode_timer) {
                    return;
                }
                if self.is_grab_pos(actor, ctx) {
                    self.mode_proc_init(Phase::Attack, actor, ctx.player);
                    return;
                }
            }
            if self.anm_prm == 1 {
                self.mode_proc_init(Phase::Wait, actor, ctx.player);
            }
        }
    }

    fn mode_disappear(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.kind != 2 {
            self.wander_pos = actor.pos;
        }
        self.cancel_grab();
        if self.anm_prm == 8 && self.morph.is_stop() {
            if self.demo_mode || self.boss_won {
                self.mode_proc_init(Phase::Delete, actor, ctx.player);
            } else if self.is_grab_foot(ctx) && !ctx.event_running() {
                self.mode_proc_init(Phase::GrabFootDemo, actor, ctx.player);
            } else {
                match self.kind {
                    0 => self.mode_proc_init(Phase::Hide, actor, ctx.player),
                    1 => self.mode_proc_init(Phase::PathMove, actor, ctx.player),
                    2 => self.mode_proc_init(Phase::GoalKeeper, actor, ctx.player),
                    _ => {}
                }
            }
        }
    }

    fn mode_wait(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        self.cancel_grab();
        if self.check_tg_hit(actor, ctx) {
            return;
        }
        if !self.area_check(actor, ctx.solid) {
            self.mode_proc_init(Phase::Disappear, actor, ctx.player);
            return;
        }
        if self.is_grab_foot(ctx) {
            self.mode_proc_init(Phase::Disappear, actor, ctx.player);
            return;
        }
        self.search_target(actor, ctx);
        let facing = distance_angle_s(actor.shape_angle[1], self.base_angle);
        if self.target_flag != NO_TARGET {
            if self.base_dist > HIO_0E0 {
                let aim = bearing(actor.pos, self.grab_pos);
                if self.is_npc() || self.is_link() {
                    stream::add_calc_angle_s2(
                        &mut actor.shape_angle[1],
                        aim,
                        TURN_DIV_SLOW,
                        TURN_CAP_SLOW,
                    );
                } else {
                    stream::add_calc_angle_s2(&mut actor.shape_angle[1], aim, TURN_DIV, TURN_CAP);
                }
            }
            self.set_grab_pos(ctx);
            if self.is_grab_pos(actor, ctx) {
                if ctx.carry_held == Some(ctx.self_entity) {
                    if timer(&mut self.grab_timer) {
                        self.mode_proc_init(Phase::Attack, actor, ctx.player);
                        return;
                    }
                } else if self.is_npc() {
                    if !timer(&mut self.grab_timer) {
                        self.mode_proc_init(Phase::Attack, actor, ctx.player);
                        return;
                    }
                } else {
                    self.mode_proc_init(Phase::Attack, actor, ctx.player);
                    return;
                }
            }
        } else if facing > i32::from(HIO_09A) {
            stream::add_calc_angle_s2(
                &mut actor.shape_angle[1],
                self.base_angle,
                TURN_DIV_FACE,
                TURN_CAP_FACE,
            );
        }
        if self.kind == 2 && flat(ctx.player.pos, self.home_pos) > self.area_radius {
            self.mode_proc_init(Phase::Disappear, actor, ctx.player);
        } else {
            self.move_rnd_back(actor, ctx);
        }
        if self.base_dist > HIO_0E4 && self.base_dist < HIO_0E8 {
            self.turn_to_base_target(actor);
        }
        if self.base_dist > HIO_0E8 {
            if timer(&mut self.far_timer) {
                self.mode_proc_init(Phase::Disappear, actor, ctx.player);
            }
        } else {
            self.far_timer = i32::from(HIO_088);
        }
    }

    fn mode_attack(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.anm_prm != 5 && !self.is_grab(ctx) {
            let gap = flat(actor.pos, self.grab_pos);
            if !(gap > HIO_0E0 && gap < HIO_0C4) && gap < HIO_0E4 + 10.0 {
                self.move_dest = self.grab_pos;
                add_calc_pos_xz2(
                    &mut actor.pos,
                    self.move_dest,
                    (HIO_0CC + 1.0) * 0.005 * self.move_speed,
                    MOVE_CAP,
                );
            }
        }
        if self.check_tg_hit(actor, ctx) {
            return;
        }
        if self.anm_prm == 5 && self.morph.is_stop() {
            self.mode_proc_init(Phase::Wait, actor, ctx.player);
            return;
        }
        if self.target_flag == NO_TARGET {
            self.set_anm(5, false);
            return;
        }
        let aim = bearing(actor.pos, ctx.target_pos);
        if distance_angle_s(actor.shape_angle[1], aim) > i32::from(HIO_09A) {
            stream::add_calc_angle_s2(&mut actor.shape_angle[1], aim, TURN_DIV, HIO_096);
        }
        if self.in_area == 0 {
            self.mode_proc_init(Phase::Disappear, actor, ctx.player);
            return;
        }
        if ctx.busy {
            let lift = self.grab_pos.y;
            self.grab_pos = ctx.player.pos;
            self.grab_pos.y = lift;
            return;
        }
        if !self.is_grab_pos(actor, ctx) {
            self.set_anm(5, false);
            return;
        }
        self.set_grab_pos(ctx);
        if !self.is_grab(ctx) {
            if self.is_link() && self.anm_prm == 4 {
                let gap = flat(self.tip, self.grab_pos);
                if gap > HIO_110 && gap < HIO_114 {
                    if ctx.player.speed >= 1.0 {
                        self.hand_push(ctx);
                    }
                } else if self.morph.is_stop() && timer(&mut self.mode_timer) {
                    self.set_anm(5, false);
                }
            } else if self.anm_prm == 4 && self.morph.is_stop() && timer(&mut self.mode_timer) {
                self.set_anm(5, false);
            }
            return;
        }
        if self.is_link() && self.anm_prm == 4 {
            if self.morph.is_stop() {
                self.mode_proc_init(Phase::GrabDemo, actor, ctx.player);
                ctx.se(SE_CM_FM_GRAB_HAND);
            } else if ctx.player.speed >= 1.0 {
                self.hand_push(ctx);
            }
        } else if self.is_npc() {
            if self.carry_kind != CARRY_NONE {
                return;
            }
            ctx.se(SE_CM_FM_GRAB_HAND);
            self.carry_kind = CARRY_NPC;
            if self.demo_mode {
                ctx.order(EVENT_APPEAR);
                self.mode_proc_init(Phase::GrabDemo, actor, ctx.player);
            } else if flat(actor.pos, ctx.player.pos) > HIO_0BC
                || (ctx.player.pos.y - actor.pos.y).abs() > HIO_0A8
                || ctx.event_running()
            {
                self.mode_proc_init(Phase::GrabNpcDemo, actor, ctx.player);
            } else {
                self.mode_proc_init(Phase::GrabDemo, actor, ctx.player);
            }
        } else if self.target_flag == TARGET_BOMB {
            if self.carry_kind == CARRY_NONE {
                self.carry_kind = CARRY_BOMB;
                ctx.se(SE_CM_FM_GRAB);
                self.mode_proc_init(Phase::GrabDemo, actor, ctx.player);
            } else {
                self.mode_proc_init(Phase::Wait, actor, ctx.player);
            }
        } else if self.target_flag == TARGET_TSUBO && self.carry_kind == CARRY_NONE {
            self.carry_kind = CARRY_TSUBO;
            ctx.se(SE_CM_FM_GRAB);
            self.mode_proc_init(Phase::GrabDemo, actor, ctx.player);
        }
    }

    fn mode_throw(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.anm_prm == 1 && timer(&mut self.mode_timer) {
            self.mode_proc_init(Phase::Wait, actor, ctx.player);
        }
        if self.carry_kind != CARRY_NONE && self.target_flag == NO_TARGET {
            self.mode_proc_init(Phase::Damage, actor, ctx.player);
            return;
        }
        if self.carry_kind == CARRY_NONE {
            return;
        }
        if self.check_tg_hit(actor, ctx) {
            return;
        }
        if self.kind != 2 || !self.is_npc() {
            stream::add_calc_angle_s2(
                &mut actor.shape_angle[1],
                bearing(actor.pos, ctx.player.pos),
                TURN_DIV,
                TURN_CAP,
            );
        }
        if self.morph.frame == HIO_0FC {
            self.carry_kind = CARRY_NONE;
            self.grab_pos = actor.pos;
            self.release(actor);
        }
        if self.anm_prm == 11 && self.morph.is_stop() {
            self.mode_timer = THROW_SETTLE;
        }
    }

    fn release(&mut self, actor: &Actor) {
        let Some(target) = self.target else {
            return;
        };
        let dropspeed = ((self.base_dist / DROP_SCALE).clamp(0.0, 1.0)) * HIO_0F4;
        self.release = Some(Release {
            target,
            yaw: actor.shape_angle[1],
            speed_f: HIO_0F0,
            dropspeed,
            gravity: HIO_0F8,
        });
    }

    fn mode_damage(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        self.cancel_grab();
        if self.anm_prm == 1 && self.arm_timer == 0 {
            self.search_target(actor, ctx);
            if self.target_flag != NO_TARGET && !self.is_npc() {
                self.mode_proc_init(Phase::Wait, actor, ctx.player);
            }
            if timer(&mut self.mode_timer) {
                self.mode_proc_init(Phase::Wait, actor, ctx.player);
            }
        }
        self.check_tg_hit(actor, ctx);
    }

    fn mode_paralysis(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.mode_timer <= i32::from(HIO_11C) - PIYO_FIRST
            && self.mode_timer >= i32::from(HIO_11C) - PIYO_LAST
        {
            ctx.se(SE_CM_MD_PIYO);
        }
        if self.mode_timer == i32::from(HIO_11C) - PIYO_LAST {
            self.set_anm(1, false);
            self.arm_timer = ARM_TIMER;
            self.arm_sway = ARM_SWAY;
        }
        if timer(&mut self.mode_timer) {
            self.mode_proc_init(Phase::Wait, actor, ctx.player);
        }
        self.cancel_grab();
        self.check_tg_hit(actor, ctx);
    }

    fn mode_bikubiku(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        self.grab_pos = ctx.player.pos;
        if self.anm_prm == 0xE {
            if timer(&mut self.mode_timer) {
                if self.health <= 0 {
                    self.mode_proc_init(Phase::Death, actor, ctx.player);
                    return;
                }
                self.set_anm(1, true);
                self.arm_timer = ARM_TIMER + i32::from(REG12_S3);
                self.arm_sway = ARM_SWAY;
            }
        } else if self.anm_prm == 1 && self.arm_timer == 0 {
            self.mode_proc_init(Phase::Wait, actor, ctx.player);
        }
    }

    fn mode_grab(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if !self.is_link() && (self.kind == 0 || self.kind == 1) && self.is_npc() {
            self.move_rnd_escape(actor, ctx);
        }
        if self.target_flag == NO_TARGET {
            self.mode_proc_init(Phase::Damage, actor, ctx.player);
            return;
        }
        if self.check_tg_hit(actor, ctx) {
            return;
        }
        if self.is_link() {
            let aim = bearing(actor.pos, self.grab_pos);
            if distance_angle_s(actor.shape_angle[1], aim) > i32::from(HIO_09A) {
                stream::add_calc_angle_s2(
                    &mut actor.shape_angle[1],
                    aim,
                    TURN_DIV,
                    TURN_CAP_NPC,
                );
            }
            self.set_grab_pos(ctx);
            if self.player_grab_bomb(ctx) {
                self.set_anm(7, false);
                self.mode_proc_init(Phase::Wait, actor, ctx.player);
            } else if timer(&mut self.sink_timer) && !ctx.event_running() {
                self.mode_proc_init(Phase::GrabDemo, actor, ctx.player);
                self.carry_kind = CARRY_PLAYER;
            } else {
                let gap = flat(self.tip, self.grab_pos);
                if gap > HIO_110 && gap < HIO_114 {
                    self.hand_push(ctx);
                } else if !self.is_grab(ctx) {
                    self.set_anm(5, false);
                    self.mode_proc_init(Phase::Wait, actor, ctx.player);
                }
                if self.is_grab_foot(ctx) {
                    self.mode_proc_init(Phase::Disappear, actor, ctx.player);
                }
            }
        } else if self.is_npc() {
            self.grab_pos = self.hand;
            if self.demo_mode {
                self.mode_proc_init(Phase::GrabNpcDemo, actor, ctx.player);
            } else {
                let reach = flat(actor.pos, ctx.player.pos);
                if reach < HIO_0B4 && self.kind == 2 {
                    let aim = if self.nearest.x != 0.0 || self.nearest.z != 0.0 {
                        bearing(actor.pos, self.nearest)
                    } else {
                        bearing(actor.pos, ctx.player.pos)
                    };
                    if distance_angle_s(actor.shape_angle[1], aim) < NPC_FACE_GATE
                        && reach < HIO_0C4
                    {
                        self.mode_proc_init(Phase::Throw, actor, ctx.player);
                        return;
                    }
                }
                if timer(&mut self.sink_timer) {
                    self.mode_proc_init(Phase::GrabDemo, actor, ctx.player);
                } else if self.target_kind == 1 {
                    self.mode_proc_init(Phase::Throw, actor, ctx.player);
                }
            }
        } else if self.target_flag == TARGET_BOMB || self.target_flag == TARGET_TSUBO {
            self.move_rnd_back(actor, ctx);
            self.grab_pos = self.hand;
            if timer(&mut self.mode_timer) {
                self.mode_proc_init(Phase::Throw, actor, ctx.player);
            }
        }
    }

    fn mode_grab_demo(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if ctx.running(EVENT_GRAB) {
            if self.anm_prm == 6 && self.morph.frame == CUT_FRAME_PULL_GRAB {
                ctx.se(SE_CM_FM_PULL_IN);
            }
            if self.grab_demo_link {
                walk::chase_f(&mut self.body_offset.x, 0.0, CHASE_STEP);
                walk::chase_f(&mut self.body_offset.y, 0.0, CHASE_STEP);
                walk::chase_f(&mut self.body_offset.z, 0.0, CHASE_STEP);
                if ctx.event_done(EVENT_GRAB) {
                    self.mode_proc_init(Phase::Delete, actor, ctx.player);
                }
            } else {
                if self.anm_prm == 6 && self.morph.frame == CUT_FRAME_END {
                    if self.target_sub == 1 {
                        ctx.se(SE_CV_MD_DAMAGE);
                    } else {
                        ctx.se(SE_CV_CB_DAMAGE);
                    }
                }
                self.caught = true;
                if ctx.event_done(EVENT_SUIKOMI) {
                    if self.point_restart != i16::from(NO_BYTE) {
                        self.cancel_grab();
                    }
                    self.mode_proc_init(Phase::Delete, actor, ctx.player);
                }
            }
        } else if self.is_link() {
            ctx.order(EVENT_GRAB);
        } else if self.is_npc() {
            ctx.order(EVENT_SUIKOMI);
        }
    }

    fn mode_grab_foot_demo(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if ctx.running(EVENT_GRAB_FOOT) {
            if self.anm_prm == 10 {
                if self.morph.frame == CUT_FRAME_PULL_FOOT {
                    ctx.se(SE_CM_FM_PULL_IN);
                }
                if self.morph.frame == CUT_FRAME_HAND {
                    ctx.se(SE_CM_FM_GRAB_FOOT);
                }
                if (CUT_FRAME_HAND..=CUT_FRAME_END).contains(&self.morph.frame) {
                    ctx.player.pos = self.hand;
                }
            }
            if ctx.event_done(EVENT_GRAB_FOOT) {
                self.mode_proc_init(Phase::Delete, actor, ctx.player);
            }
        } else if self.anm_prm != 10 {
            if flat(actor.pos, ctx.player.pos) < CUT_REACH {
                self.set_anm(10, false);
            } else {
                add_calc_pos_xz2(&mut self.player_slot, actor.pos, CUT_TWEEN, CUT_TWEEN_STEP);
                ctx.player.pos = self.player_slot;
            }
        } else {
            ctx.order(EVENT_GRAB_FOOT);
        }
    }

    fn mode_grab_npc_demo(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        self.grab_pos = self.hand;
        if ctx.running(EVENT_GRAB_NPC) {
            if ctx.event_done(EVENT_GRAB_NPC) {
                if self.demo_mode {
                    self.mode_proc_init(Phase::GrabDemo, actor, ctx.player);
                } else {
                    self.mode_proc_init(Phase::Grab, actor, ctx.player);
                }
            }
        } else {
            ctx.order(EVENT_GRAB_NPC);
        }
    }

    fn mode_player_start_demo(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        let _ = ctx;
        if self.hole_scale > HOLE_GATE {
            self.set_hole_scale(HIO_120, HOLE_SCALE, HIO_128);
            self.doomed = true;
        }
        let _ = actor;
    }

    fn mode_delete(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        let _ = actor;
        if self.is_hole_appear()
            && self.set_hole_scale(HIO_120, HOLE_SCALE, HIO_128)
            && !ctx.event_running()
        {
            self.doomed = true;
        }
    }

    fn mode_death(&mut self, actor: &mut Actor, commands: &mut Commands) {
        if self.anm_prm == 8 && self.set_hole_scale(HIO_120, HOLE_SCALE, HIO_128) {
            commands.spawn((
                Actor {
                    name: super::iball::IBALL_ACTORS[0].to_owned(),
                    parameters: 0,
                    pos: actor.pos,
                    angle: [0; 3],
                    set_id: 0,
                    room: actor.room,
                    ..Default::default()
                },
                iball::Iball::spawn(),
            ));
            self.doomed = true;
        }
    }

    fn mode_prepare_item(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>, commands: &mut Commands) {
        if self.check_tg_hit(actor, ctx) {
            return;
        }
        let step = self.anm_prm;
        if step == 3 && self.morph.is_stop() {
            self.spawn_item = Some(
                commands
                    .spawn((
                        Actor {
                            name: TSUBO_ACTOR.to_owned(),
                            parameters: SKULL_PARAM,
                            pos: self.hand,
                            angle: [0; 3],
                            set_id: 0,
                            room: actor.room,
                            ..Default::default()
                        },
                        tsubo::Tsubo::spawn(SKULL_PARAM),
                    ))
                    .id(),
            );
            return;
        }
        if step == 8 {
            self.target = self.spawn_item;
            self.target_flag = TARGET_TSUBO;
            self.target_sub = TSUBO_SKULL;
            self.carry_kind = CARRY_TSUBO;
            self.set_anm(2, false);
            self.set_grab_pos(ctx);
        } else if step == 2 && self.morph.is_stop() {
            self.set_anm(9, false);
        } else if step == 9 && timer(&mut self.mode_timer) {
            self.mode_proc_init(Phase::Throw, actor, ctx.player);
        }
    }
}

struct Loot {
    entity: Entity,
    pos: Vec3,
    flag: u8,
    sub: u8,
    is_actor: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Release {
    target: Entity,
    yaw: i16,
    speed_f: f32,
    dropspeed: f32,
    gravity: f32,
}

struct Ctx<'a, 'b> {
    player: &'a mut Player,
    solid: Option<&'b Collision>,
    switches: &'b Switches,
    blow: Blow,
    busy: bool,
    jumping: bool,
    rolling: bool,
    player_hang: bool,
    bow_aim: bool,
    carry_held: Option<Entity>,
    carry_bomb: bool,
    carryables: &'b [Loot],
    player_entity: Entity,
    actor_pos: Vec3,
    target_pos: Vec3,
    list: &'b EventList,
    active: &'a mut ActiveEvent,
    events: &'a mut SimEvents,
    self_entity: Entity,
}

impl Ctx<'_, '_> {
    fn se(&mut self, id: u32) {
        self.events.list.push(SimEvent::Se(id));
    }

    fn event_index(&self, name: &str) -> Option<usize> {
        self.list.event_index(name)
    }

    fn order(&mut self, name: &str) -> bool {
        let Some(index) = self.event_index(name) else {
            return false;
        };
        let entity = self.self_entity;
        self.active.order(self.list, index, entity)
    }

    fn event_running(&self) -> bool {
        self.event_index(EVENT_GRAB)
            .or_else(|| self.event_index(EVENT_SUIKOMI))
            .or_else(|| self.event_index(EVENT_GRAB_NPC))
            .or_else(|| self.event_index(EVENT_GRAB_FOOT))
            .or_else(|| self.event_index(EVENT_APPEAR))
            .is_some_and(|index| self.active.stage_of(index).is_some())
    }

    fn event_done(&self, name: &str) -> bool {
        self.event_index(name)
            .is_some_and(|index| self.active.end_check(index))
    }

    fn running(&self, name: &str) -> bool {
        self.event_index(name)
            .is_some_and(|index| self.active.start_check(index))
    }
}

pub fn is_floormaster(name: &str) -> bool {
    FLOORMASTER_ACTORS.contains(&name)
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    switches: Res<'w, Switches>,
    combat: Res<'w, Combat>,
    hammer: Res<'w, hammer::Cut>,
    held: Res<'w, Held>,
    blasts: Res<'w, bombflower::Blasts>,
    carry: Res<'w, carry::Carry>,
    list: Res<'w, EventList>,
    hop: Res<'w, super::hop::Hop>,
    jump: Res<'w, jump::Jump>,
    roll: Res<'w, roll::Roll>,
    players: Query<'w, 's, (Entity, &'static mut Player)>,
    shafts: Query<'w, 's, &'static arrow::Arrow>,
    bolts: Query<'w, 's, &'static boomerang::Boomerang>,
    chains: Query<'w, 's, &'static hookshot::Hookshot>,
    pots: Query<
        'w,
        's,
        (Entity, &'static Actor, &'static tsubo::Tsubo),
        (With<tsubo::Tsubo>, Without<Floormaster>),
    >,
    bombs: Query<
        'w,
        's,
        (Entity, &'static Actor),
        (With<bombflower::Crawler>, Without<Floormaster>),
    >,
}

pub fn control(
    watch: Watch<'_, '_>,
    mut active: ResMut<ActiveEvent>,
    mut events: ResMut<SimEvents>,
    mut zones: ResMut<ZoneActors>,
    mut bosses: Query<
        (Entity, &mut Actor, &mut Floormaster),
        (With<Floormaster>, Without<Player>),
    >,
    mut loot_actors: Query<&mut Actor, (Without<Floormaster>, Without<Player>, Without<Ship>)>,
    mut pots: Query<&mut tsubo::Tsubo, Without<Floormaster>>,
    mut commands: Commands,
) {
    let solid = watch.collision.as_deref();
    let switches = *watch.switches;
    let carried = watch.carry.held;
    let busy = watch.combat.active || watch.roll.active || watch.jump.active || watch.hop.active;
    let blasts: Vec<Vec3> = watch.blasts.list.clone();
    let hammer_side = watch.hammer.kind == hammer::CUT_HAMMER_SIDESWING;
    let loot: Vec<Loot> = watch
        .pots
        .iter()
        .map(|(entity, actor, pot)| Loot {
            entity,
            pos: actor.pos,
            flag: TARGET_TSUBO,
            sub: pot.kind,
            is_actor: true,
        })
        .chain(watch.bombs.iter().map(|(entity, actor)| Loot {
            entity,
            pos: actor.pos,
            flag: TARGET_BOMB,
            sub: 0,
            is_actor: true,
        }))
        .collect();
    let carry_bomb = carried.is_some_and(|id| watch.bombs.get(id).is_ok());
    let at = {
        let mut found = At::Idle;
        if watch
            .chains
            .iter()
            .any(|chain| chain.phase == hookshot::Phase::Pull)
        {
            found = At::Grapple;
        } else if watch
            .chains
            .iter()
            .any(|chain| chain.phase != hookshot::Phase::Wait)
        {
            found = At::Hookshot;
        } else {
            for shaft in watch.shafts.iter() {
                if shaft.phase != arrow::Phase::Flying {
                    continue;
                }
                found = match shaft.kind {
                    arrow::TYPE_LIGHT => At::LightArrow,
                    arrow::TYPE_FIRE => At::FireArrow,
                    arrow::TYPE_ICE => At::IceArrow,
                    _ => At::Arrow,
                };
                break;
            }
            if found == At::Idle
                && watch
                    .bolts
                    .iter()
                    .any(|bolt| bolt.parameter == boomerang::MODE_THROW)
            {
                found = At::Boomerang;
            }
            if found == At::Idle
                && (watch.hammer.kind == hammer::CUT_HAMMER_SIDESWING
                    || watch.hammer.kind == hammer::CUT_HAMMER_FRONTSWING)
            {
                found = At::Hammer;
            }
            if found == At::Idle && watch.combat.active && watch.combat.frame >= 1 && watch.combat.frame < 6 {
                found = At::Sword;
            }
            if found == At::Idle && !blasts.is_empty() {
                found = At::Bomb;
            }
        }
        found
    };
    let blow = Blow {
        at,
        blade: watch.held.item,
        soup: watch.combat.soup,
        hammer_side,
    };
    let mut players = watch.players;
    let mut first = None;
    for (entity, body) in players.iter_mut() {
        first = Some((entity, body));
        break;
    }
    let Some((player_entity, mut player)) = first else {
        return;
    };
    let mut spent: Vec<(Entity, i8, u16)> = Vec::new();
    for (entity, mut actor, mut boss) in &mut bosses {
        let actor = &mut *actor;
        let boss = &mut *boss;
        if !boss.entered {
            boss.begin(actor, &player, &switches);
        }
        if boss.killed {
            spent.push((entity, actor.room, actor.set_id));
            continue;
        }
        let target_pos = boss
            .target
            .map(|id| {
                if id == player_entity {
                    player.pos
                } else {
                    loot
                        .iter()
                        .find(|loot| loot.entity == id)
                        .map_or(player.pos, |loot| loot.pos)
                }
            })
            .unwrap_or(player.pos);
        let ctx = Ctx {
            player: &mut player,
            solid,
            switches: &switches,
            blow,
            busy,
            jumping: watch.jump.active,
            rolling: watch.roll.active,
            player_hang: false,
            bow_aim: false,
            carry_held: carried,
            carry_bomb,
            carryables: &loot,
            player_entity,
            actor_pos: actor.pos,
            target_pos,
            list: &watch.list,
            active: &mut active,
            events: &mut events,
            self_entity: entity,
        };
        boss.execute(actor, ctx, &mut commands);
        if let Some(release) = boss.release.take() {
            if let Ok(mut pot) = pots.get_mut(release.target) {
                if TSKU_DROP_KINDS.contains(&pot.kind) {
                    pot.speed_y = release.dropspeed;
                }
            }
            if let Ok(mut item) = loot_actors.get_mut(release.target) {
                item.angle[1] = release.yaw;
                item.shape_angle[1] = release.yaw;
                item.speed_f = release.speed_f;
                item.speed.y = release.dropspeed;
                item.gravity = release.gravity;
            }
        }
        if boss.doomed {
            spent.push((entity, actor.room, actor.set_id));
        }
    }
    for (entity, room, set_id) in spent {
        zones.mark_done(room, set_id);
        let _ = commands.entity(entity).despawn();
    }
}
