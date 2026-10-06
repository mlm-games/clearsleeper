use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::area::StageName;
use super::arrow;
use super::boomerang;
use super::bombflower;
use super::collision::Collision;
use super::combat::Combat;
use super::damage;
use super::events::{SimEvent, SimEvents};
use super::flags::{Collect, EventFlags, MemBits, StoryBits};
use super::hammer;
use super::health::Health;
use super::inventory::{self, Equipped, ITEM_PEARL_FARORE};
use super::item::Held;
use super::pickup;
use super::player::Player;
use super::stream::{add_calc_angle_s2, cos_s, sin_s};
use super::transition::NextStage;
use super::walk::{from_s16, to_s16};
use crate::text::msgproc::{DA_EVT_MODE_NONE, MsgGameInfo};

pub const KALLE_DEMOS_ACTOR: &str = "Bkm";

pub fn is_kalle_demos(name: &str) -> bool {
    name == KALLE_DEMOS_ACTOR
}

const ANGLE_X: usize = 0;
const ANGLE_Y: usize = 1;
const ANGLE_Z: usize = 2;

const HIO_M08: f32 = 0.8;
const HIO_M0C: f32 = 1.0;
const HIO_M14: i16 = 100;

const MODE_WAIT: i16 = 0;
const MODE_ATTACK_1: i16 = 1;
const MODE_ATTACK_2: i16 = 2;
const MODE_DAMAGE: i16 = 3;
const MODE_EAT: i16 = 5;
const MODE_START: i16 = 10;
const MODE_END: i16 = 11;

const CORE_IDLE: i16 = 0;
const CORE_HIT: i16 = 1;
const CORE_HIT_WAIT: i16 = 2;
const CORE_DYING: i16 = 100;

const MAKAR_OFF: i8 = 0;
const MAKAR_FIRST: i8 = 1;
const MAKAR_SECOND: i8 = 2;
const MAKAR_ENTER: i8 = 3;
const MAKAR_FOUR: i8 = 4;
const MAKAR_DROP: i8 = 5;
const MAKAR_DROPING: i8 = 6;
const MAKAR_DONE: i8 = 10;

const DEMO_OFF: i16 = 0;
const DEMO_START: i16 = 1;
const DEMO_MOVE: i16 = 2;
const DEMO_LOOK: i16 = 5;
const DEMO_CB1: i16 = 6;
const DEMO_FALL: i16 = 7;
const DEMO_CORE: i16 = 8;
const DEMO_ENTER: i16 = 9;
const DEMO_SMALL: i16 = 10;
const DEMO_SIT: i16 = 11;
const DEMO_LAUGH: i16 = 12;
const DEMO_RETURN: i16 = 15;
const DEMO_LAST: i16 = 100;
const DEMO_CORE_FALL: i16 = 101;
const DEMO_CATCH: i16 = 102;
const DEMO_ORBIT: i16 = 103;
const DEMO_CLEAN: i16 = 150;

const VINE_NONE: i16 = 0;
const VINE_OUT: i16 = 1;
const VINE_UP: i16 = 2;
const VINE_FULL: i16 = 3;

const ENV_NONE: u8 = 0;
const ENV_FADE: u8 = 1;
const ENV_DARK: u8 = 2;

const STTS_SHOWMAP: u32 = 0x0000_0020;
const STTS_UNK4000: u32 = 0x0000_4000;
const STTS_UNK40000: u32 = 0x0004_0000;
const STTS_BOSS: u32 = 0x0400_0000;
const ATTN_LOCKON_BATTLE: u32 = 0x0000_0004;
const ATTN_BATTLE_DISTANCE: i32 = 4;

const EMODE_NONE: u8 = 0;
const EMODE_LOOP: u8 = 2;
const STATE_STOP: u8 = 1;
const STATE_LOOP: u8 = 2;

const BCK_CALL_01: i32 = 0x08;
const BCK_CALL_02: i32 = 0x09;
const BCK_CALL_03: i32 = 0x0A;
const BCK_COA_DAMAGE: i32 = 0x0B;
const BCK_COA_DEAD1: i32 = 0x0C;
const BCK_COA_DEAD2: i32 = 0x0D;
const BCK_COA_LDEAD: i32 = 0x0E;
const BCK_COA_NDEAD1: i32 = 0x0F;
const BCK_COA_NDEAD2: i32 = 0x10;
const BCK_COA_START1: i32 = 0x11;
const BCK_COA_START2: i32 = 0x12;
const BCK_COA_START3: i32 = 0x13;
const BCK_COA_WAIT: i32 = 0x14;
const BCK_DROP: i32 = 0x15;
const BCK_HANA_ATTACK: i32 = 0x16;
const BCK_HANA_DEAD1: i32 = 0x17;
const BCK_HANA_DEAD2: i32 = 0x18;
const BCK_HANA_DEAD3: i32 = 0x19;
const BCK_HANA_EAT: i32 = 0x1A;
const BCK_HANA_HAKIDASU: i32 = 0x1B;
const BCK_HANA_OTIRU: i32 = 0x1C;
const BCK_HANA_START1: i32 = 0x1D;
const BCK_HANA_START2: i32 = 0x1E;
const BCK_HANA_TOJIRU: i32 = 0x1F;
const BCK_HANA_WAIT: i32 = 0x20;
const BCK_HIRAKU_WAIT: i32 = 0x21;
const BCK_NEW_SDEMO1: i32 = 0x22;
const BCK_NEW_SDEMO2: i32 = 0x23;
const BCK_NEW_SDEMO3: i32 = 0x24;
const BCK_NEW_SDEMO4: i32 = 0x25;

const FRAME_CALL_01: i16 = 19;
const FRAME_CALL_02: i16 = 19;
const FRAME_CALL_03: i16 = 25;
const FRAME_COA_DAMAGE: i16 = 14;
const FRAME_COA_DEAD1: i16 = 42;
const FRAME_COA_DEAD2: i16 = 28;
const FRAME_COA_LDEAD: i16 = 59;
const FRAME_COA_NDEAD1: i16 = 19;
const FRAME_COA_NDEAD2: i16 = 39;
const FRAME_COA_START1: i16 = 60;
const FRAME_COA_START2: i16 = 52;
const FRAME_COA_START3: i16 = 30;
const FRAME_COA_WAIT: i16 = 30;
const FRAME_DROP: i16 = 38;
const FRAME_HANA_ATTACK: i16 = 20;
const FRAME_HANA_DEAD1: i16 = 20;
const FRAME_HANA_DEAD2: i16 = 15;
const FRAME_HANA_DEAD3: i16 = 209;
const FRAME_HANA_EAT: i16 = 9;
const FRAME_HANA_HAKIDASU: i16 = 30;
const FRAME_HANA_OTIRU: i16 = 39;
const FRAME_HANA_START1: i16 = 10;
const FRAME_HANA_START2: i16 = 42;
const FRAME_HANA_TOJIRU: i16 = 49;
const FRAME_HANA_WAIT: i16 = 30;
const FRAME_HIRAKU_WAIT: i16 = 30;
const FRAME_NEW_SDEMO1: i16 = 55;
const FRAME_NEW_SDEMO2: i16 = 42;
const FRAME_NEW_SDEMO3: i16 = 46;
const FRAME_NEW_SDEMO4: i16 = 84;

const TIMER_COUNT: usize = 4;
const CORE_FX_COUNT: usize = 3;
const VINE_COUNT: i32 = 5;
const HAND_COUNT: i32 = 20;
const FOOT_COUNT: i32 = 8;
const SHAKE_FRAME: i16 = 5;

const CORE_HEALTH: i32 = 0x0F;
const BODY_RADIUS: f32 = 300.0;
const BODY_LIFT: f32 = 200.0;
const COCYL_SRC_RADIUS: f32 = 100.0;
const COCYL_SRC_HEIGHT: f32 = 400.0;
const COCYL_HEIGHT: f32 = 3000.0;
const COCYL_RANGE: f32 = 800.0;
const OFFSCREEN_X: f32 = -20000.0;
const OFFSCREEN_Y: f32 = 10000.0;
const OFFSCREEN_Z: f32 = -10000.0;
const AT_POWER_CO_CYL: u8 = 1;
const PLAYER_TG_RADIUS: f32 = 30.0;
const PLAYER_TG_HEIGHT: f32 = 125.0;
const CORE_SRC_RADIUS: f32 = 40.0;
const CORE_RADIUS: f32 = 50.0;
const CORE_LIFT: f32 = 300.0;
const HEAD_OFFSET: [f32; 3] = [0.0, -20.0, 30.0];
const ATTENTION_LIFT: f32 = 50.0;
const THROW_LIFT: f32 = 1000.0;
const THROW_SPEED: f32 = 50.0;
const THROW_RISE: f32 = 120.0;
const THROW_DAMAGE: u8 = 4;
const THROW_ANGLE: i16 = 0x5000;
const MAKAR_OFFSET: [f32; 3] = [0.0, REG0_F1 + 20.0, REG0_F2 + 90.0];
const MAKAR_OFFSET_TALL: [f32; 3] = [0.0, 70.0, 1000.0];
const MAKAR_OFFSET_CATCH: [f32; 3] = [0.0, 0.0, 850.0];
const MAKAR_YAW_OFFSET: i16 = 3000;
const MAKAR_HURT_YAW: i16 = -8000;
const MAKAR_HURT_PITCH: i16 = -5000;
const MAKAR_PAN_Y: f32 = 100.0;
const MAKAR_CATCH_DIST: f32 = 200.0;
const EAT_STAND: f32 = 300.0;
const THROW_REACH: f32 = 350.0;
const WIND_RADIUS: f32 = 2000.0;
const WIND_FULL: f32 = 1500.0;
const WIND_MAX: f32 = 2000.0;
const WIND_STEP: f32 = 50.0;
const WIND_RATE: f32 = 0.002;
const VINE_YAW_STEP: f32 = 13107.2;
const VINE_REACH: f32 = 200.0;
const VINE_LIFT: f32 = 15.0;
const VINE_LIFT_SCALE: f32 = 2.5;
const VINE_LIFT_BIAS: f32 = 250.0;
const ENV_STEP_A: f32 = 0.025;
const ENV_STEP_B: f32 = 0.02;
const RIDE_STEP: f32 = 400.0;
const RIDE_RADIUS: f32 = 60.0;
const SHAKE_ANGLE: f32 = 0.3300;
const SHAKE_ANGLE_Z: f32 = 12288.0;
const SHAKE_CLOCK: i16 = 0x1C00;
const SHAKE_SWING: f32 = 7.5;
const CAM_ENTER: [f32; 3] = [950.0, 150.0, 920.0];
const CAM_SIT: [f32; 3] = [-139.0, 125.0, 475.0];
const CAM_LIFTED: [f32; 3] = [2.0, 75.0, 403.0];
const CAM_LOOK_A: [f32; 3] = [401.0, 228.0, 957.0];
const CAM_CB1: [f32; 3] = [74.0, 283.0, 745.0];
const CAM_CB1_LOOK: [f32; 3] = [-87.0, 58.0, 102.0];
const CAM_FALL: [f32; 3] = [-47.0, 115.0, 263.0];
const CAM_FALL_LOOK: [f32; 3] = [-199.0, -89.0, -349.0];
const CAM_START_POS: [f32; 3] = [61.0, 0.0, 492.0];
const CAM_START_ANGLE: i16 = -0x75FB;
const CAM_LOOK_MOVE: [f32; 3] = [0.0, 100.0, -1100.0];
const DEMO_HEAD_LIFT: f32 = 30.0;
const DEMO_SDEMO_LIFT: f32 = 60.0;
const DEMO_SMILE: i32 = 0;
const DEMO_SURPRISE: i32 = 1;
const DEMO_LOOK_UP: i32 = 2;
const DEMO_SURPRISE_SMALL: i32 = 3;
const DEMO_UNKNOWN_29: i32 = 29;
const DEMO_ORIGINAL: i32 = -1;
const VOICE_CB_EATEN: u32 = 0x48C6;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const EVENT_KALLE_DEMOS_TRIALS_CLEAR: u16 = 0x3220;
const EVENT_TMP_0480: u32 = 0x0480;

const SE_LK_LAST_HIT: u32 = 0x2828;
const SE_LK_B_BOSS_WARP: u32 = 0x2888;
const SE_CV_BKM_CUT_VINE_ALL: u32 = 0x4854;
const SE_CV_BKM_DAMAGE: u32 = 0x4855;
const SE_CV_BKM_LAST_DAMAGE: u32 = 0x4856;
const SE_CV_BKM_FALLDOWN: u32 = 0x4857;
const SE_CV_BKM_JITABATA: u32 = 0x4858;
const SE_CV_BKM_DIE: u32 = 0x4859;
const SE_CV_BKM_ENTER: u32 = 0x485A;
const SE_CV_BKM_LAUGH: u32 = 0x485B;
const SE_CV_CB_HELP: u32 = 0x48C5;
const SE_CV_CB_SAVED: u32 = 0x48C7;
const SE_CM_BKM_CORE_ENTER: u32 = 0x584A;
const SE_CM_BKM_ATKVINE_ENTER: u32 = 0x584C;
const SE_CM_BKM_LINK_OUT: u32 = 0x584F;
const SE_CM_BKM_END_CORE_DROP: u32 = 0x5855;
const SE_CM_BKM_END_CORE_LEAP: u32 = 0x5856;
const SE_CM_BKM_FLW_WAVING: u32 = 0x780A;
const SE_CM_BKM_FLW_TO_BUD: u32 = 0x780B;
const SE_CM_BKM_VINE_OUT: u32 = 0x780C;
const SE_CM_BKM_BODY_UP: u32 = 0x780D;
const SE_CM_BKM_BODY_FALL: u32 = 0x780E;
const SE_CM_BKM_BODY_CLOSE: u32 = 0x780F;
const SE_CM_BKM_BODY_EATING: u32 = 0x7810;
const SE_CM_BKM_END_FLW_WAVING: u32 = 0x7815;
const SE_CM_BKM_END_FLW_LIFTUP: u32 = 0x7816;
const SE_CM_BKM_END_FLW_DIE: u32 = 0x7817;

const BGM_KINDAN_BOSS: u32 = 0x8000_0005;
const BGM_PAST_BKM: u32 = 0x8000_004A;
const BGM_UNK_105: u32 = 0x8000_0105;
const BGM_UNK_151: u32 = 0x8000_0151;
const STRM_BOSS_CLEAR: u32 = 0xC000_0003;

const BGM_STOP: u32 = 0;

const ITEM_HEART_CONTAINER: u8 = 0x08;
const ITEM_FOREST_WATER: u8 = 0x59;
const BOTTLE_SLOT_0: usize = 14;
const BOTTLE_SLOTS: usize = 4;
const ITEM_ACTOR: &str = "item";
const WARP_ACTOR: &str = "Warpf";
const MAKAR_ACTOR: &str = "Cb1";
const HAND_ACTOR: &str = "Bmdhand";
const FOOT_ACTOR: &str = "Bmdfoot";
const NO_SET_ID: u16 = 0xFFFF;

const REACH_SWORD: f32 = 20.0;
const REACH_ARROW: f32 = 35.0;
const REACH_BOOMERANG: f32 = 90.0;
const REACH_HAMMER: f32 = 50.0;
const REACH_HOOKSHOT: f32 = 5.0;
const REACH_BOMB: f32 = 150.0;

const AT_TYPE_ALL: u32 = u32::MAX;
const AT_TYPE_BOOMERANG: u32 = 1 << 6;
const AT_TYPE_WATER: u32 = 1 << 8;
const AT_TYPE_UNK800: u32 = 1 << 11;
const AT_TYPE_UNK20000: u32 = 1 << 17;
const AT_TYPE_WIND: u32 = 1 << 21;
const AT_TYPE_UNK400000: u32 = 1 << 22;
const AT_TYPE_LIGHT: u32 = 1 << 23;

const CULL_MIN: [f32; 3] = [-300.0, 0.0, -300.0];
const CULL_MAX: [f32; 3] = [300.0, 900.0, 300.0];

const REG0_F1: f32 = 0.0;
const REG0_F2: f32 = 0.0;
const REG0_F3: f32 = 0.0;
const REG0_F4: f32 = 0.0;
const REG0_F5: f32 = 0.0;
const REG0_F6: f32 = 0.0;
const REG0_F7: f32 = 0.0;
const REG0_F8: f32 = 0.0;
const REG8_F8: f32 = 0.0;
const REG0_F9: f32 = 0.0;
const REG0_F10: f32 = 0.0;
const REG0_F11: f32 = 0.0;
const REG0_F12: f32 = 0.0;
const REG0_F13: f32 = 0.0;
const REG0_F14: f32 = 0.0;
const REG0_F15: f32 = 0.0;
const REG0_F16: f32 = 0.0;
const REG0_F17: f32 = 0.0;
const REG0_F18: f32 = 0.0;
const REG17_F0: f32 = 0.0;
const REG17_F1: f32 = 0.0;
const REG17_F2: f32 = 0.0;
const REG0_S0: i16 = 0;
const REG0_S1: i16 = 0;
const REG0_S2: i16 = 0;
const REG0_S3: i16 = 0;
const REG0_S4: i16 = 0;
const REG0_S5: i16 = 0;
const REG0_S6: i16 = 0;
const REG0_S7: i16 = 0;
const REG0_S9: i16 = 0;

const US_SWAY_RATE: f32 = 250.0;
const US_SWAY_RATE_ALT: f32 = 200.0;

fn frame_max(bck: i32) -> i16 {
    match bck {
        BCK_CALL_01 => FRAME_CALL_01,
        BCK_CALL_02 => FRAME_CALL_02,
        BCK_CALL_03 => FRAME_CALL_03,
        BCK_COA_DAMAGE => FRAME_COA_DAMAGE,
        BCK_COA_DEAD1 => FRAME_COA_DEAD1,
        BCK_COA_DEAD2 => FRAME_COA_DEAD2,
        BCK_COA_LDEAD => FRAME_COA_LDEAD,
        BCK_COA_NDEAD1 => FRAME_COA_NDEAD1,
        BCK_COA_NDEAD2 => FRAME_COA_NDEAD2,
        BCK_COA_START1 => FRAME_COA_START1,
        BCK_COA_START2 => FRAME_COA_START2,
        BCK_COA_START3 => FRAME_COA_START3,
        BCK_COA_WAIT => FRAME_COA_WAIT,
        BCK_DROP => FRAME_DROP,
        BCK_HANA_ATTACK => FRAME_HANA_ATTACK,
        BCK_HANA_DEAD1 => FRAME_HANA_DEAD1,
        BCK_HANA_DEAD2 => FRAME_HANA_DEAD2,
        BCK_HANA_DEAD3 => FRAME_HANA_DEAD3,
        BCK_HANA_EAT => FRAME_HANA_EAT,
        BCK_HANA_HAKIDASU => FRAME_HANA_HAKIDASU,
        BCK_HANA_OTIRU => FRAME_HANA_OTIRU,
        BCK_HANA_START1 => FRAME_HANA_START1,
        BCK_HANA_START2 => FRAME_HANA_START2,
        BCK_HANA_TOJIRU => FRAME_HANA_TOJIRU,
        BCK_HANA_WAIT => FRAME_HANA_WAIT,
        BCK_HIRAKU_WAIT => FRAME_HIRAKU_WAIT,
        BCK_NEW_SDEMO1 => FRAME_NEW_SDEMO1,
        BCK_NEW_SDEMO2 => FRAME_NEW_SDEMO2,
        BCK_NEW_SDEMO3 => FRAME_NEW_SDEMO3,
        BCK_NEW_SDEMO4 => FRAME_NEW_SDEMO4,
        _ => FRAME_HANA_WAIT,
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

fn turned(yaw: i16, offset: [f32; 3]) -> Vec3 {
    let (sin, cos) = from_s16(yaw).sin_cos();
    Vec3::new(
        offset[0] * cos + offset[2] * sin,
        offset[1],
        offset[2] * cos - offset[0] * sin,
    )
}

fn tilted(pitch: i16, value: Vec3) -> Vec3 {
    let (sin, cos) = from_s16(pitch).sin_cos();
    Vec3::new(
        value.x,
        value.y * cos - value.z * sin,
        value.y * sin + value.z * cos,
    )
}

fn sn(value: f32) -> i16 {
    value as i32 as i16
}

fn sni(value: i32) -> i16 {
    value as i16
}

fn reach_flat(from: Vec3, to: Vec3) -> f32 {
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    (dx * dx + dz * dz).sqrt()
}

fn touches(centre: Vec3, radius: f32, at: Vec3, reach: f32) -> bool {
    (at - centre).length() <= radius + reach
}

fn cyls_touch(a: Vec3, a_radius: f32, a_height: f32, b: Vec3, b_radius: f32, b_height: f32) -> bool {
    let sum = a_radius + b_radius;
    let dx = a.x - b.x;
    let dz = a.z - b.z;
    if dx * dx + dz * dz > sum * sum {
        return false;
    }
    a.y + a_height >= b.y && a.y <= b.y + b_height
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
            add_calc2(&mut self.cur_morf, 1.0, 1.0, self.morf_step);
        }
        self.update();
    }

    fn is_stop(&self) -> bool {
        self.state & STATE_STOP != 0 || self.rate == 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum At {
    Idle,
    Sword,
    Boomerang,
    Hookshot,
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
            At::Sword => REACH_SWORD,
            At::Arrow | At::FireArrow | At::IceArrow | At::LightArrow => REACH_ARROW,
            At::Boomerang => REACH_BOOMERANG,
            At::Hammer => REACH_HAMMER,
            At::Hookshot => REACH_HOOKSHOT,
            At::Bomb => REACH_BOMB,
            At::Idle => 0.0,
        }
    }

    fn power(self, blade: u8, soup: bool) -> i32 {
        match self {
            At::Sword => damage::sword_power(blade, soup),
            At::Arrow | At::FireArrow | At::IceArrow => damage::ARROW,
            At::Bomb => damage::BOMB,
            At::Hammer => hammer::AT_POWER,
            At::Boomerang | At::LightArrow | At::Idle | At::Hookshot => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blow {
    pub at: At,
    pub blade: u8,
    pub soup: bool,
}

struct Spawn {
    name: &'static str,
    parameters: u32,
    pos: Vec3,
    angle: [i16; 3],
    room: i8,
    set_id: u16,
    pickup: bool,
}

struct Env<'a> {
    player: &'a mut Player,
    solid: Option<&'a Collision>,
    blow: Blow,
    stage_x: bool,
    stage_boss_enemy: bool,
    stage_boss_demo: bool,
    farore_pearl: bool,
    forest_water: bool,
    event_run: bool,
}

impl Env<'_> {
    fn pos(&self) -> Vec3 {
        self.player.pos
    }

    fn yaw(&self) -> i16 {
        to_s16(self.player.yaw)
    }

    fn distance(&self, from: Vec3) -> f32 {
        (self.player.pos - from).length()
    }

    fn distance_flat(&self, from: Vec3) -> f32 {
        reach_flat(from, self.player.pos)
    }

    fn angle_y(&self, from: Vec3) -> i16 {
        let diff = self.player.pos - from;
        to_s16(diff.x.atan2(diff.z))
    }

    fn place(&mut self, pos: Vec3, angle: i16) {
        if !self.event_run {
            return;
        }
        self.player.pos = pos;
        self.player.face = from_s16(angle);
        self.player.yaw = from_s16(angle);
    }
}

struct Sink<'a> {
    events: &'a mut SimEvents,
    mem: &'a mut MemBits,
    stories: &'a mut StoryBits,
    flags: &'a mut EventFlags,
    health: &'a mut Health,
    info: &'a mut MsgGameInfo,
    spawns: Vec<Spawn>,
    leave: Option<u8>,
}

impl Sink<'_> {
    fn se(&mut self, id: u32) {
        self.events.list.push(SimEvent::Se(id));
    }

    fn bgm(&mut self, id: u32) {
        self.events.list.push(SimEvent::Bgm(id));
    }

    fn stop_bgm(&mut self) {
        self.events.list.push(SimEvent::Bgm(BGM_STOP));
    }

    fn stage_bgm(&mut self, stage_x: bool) {
        if stage_x {
            self.bgm(BGM_PAST_BKM);
        } else {
            self.bgm(BGM_KINDAN_BOSS);
        }
    }

    fn event_reset(&mut self) {
        self.info.event_mode = DA_EVT_MODE_NONE;
    }

    fn hurt_player(&mut self, units: u8) {
        self.health.damage(units);
        self.events.list.push(SimEvent::Hurt);
    }

    fn on_stage_boss_enemy(&mut self) {
        self.mem.on_stage_boss_enemy();
    }

    fn on_stage_boss_demo(&mut self) {
        self.mem.on_stage_boss_demo();
    }

    fn spawn_item(&mut self, parameters: u32, pos: Vec3, angle: [i16; 3], room: i8, set_id: u16) {
        self.spawns.push(Spawn {
            name: ITEM_ACTOR,
            parameters,
            pos,
            angle,
            room,
            set_id,
            pickup: true,
        });
    }

    fn spawn_warp(&mut self, pos: Vec3, angle: [i16; 3], room: i8, set_id: u16) {
        self.spawns.push(Spawn {
            name: WARP_ACTOR,
            parameters: 0,
            pos,
            angle,
            room,
            set_id,
            pickup: false,
        });
    }

    fn spawn_actor(
        &mut self,
        name: &'static str,
        parameters: u32,
        pos: Vec3,
        angle: i16,
        room: i8,
    ) {
        self.spawns.push(Spawn {
            name,
            parameters,
            pos,
            angle: [0, angle, 0],
            room,
            set_id: NO_SET_ID,
            pickup: false,
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sph {
    pub centre: Vec3,
    pub radius: f32,
    pub at_set: bool,
    pub at_vs_player: bool,
    pub at_hit: bool,
    pub tg_set: bool,
    pub tg_type: u32,
    pub tg_hit: bool,
    pub co_set: bool,
}

impl Sph {
    const BODY_SRC: Sph = Sph {
        centre: Vec3::ZERO,
        radius: BODY_RADIUS,
        at_set: false,
        at_vs_player: false,
        at_hit: false,
        tg_set: true,
        tg_type: AT_TYPE_ALL & !AT_TYPE_BOOMERANG & !AT_TYPE_WATER & !AT_TYPE_UNK20000 & !AT_TYPE_WIND
            & !AT_TYPE_UNK400000
            & !AT_TYPE_LIGHT,
        tg_hit: false,
        co_set: false,
    };

    const CORE_SRC: Sph = Sph {
        centre: Vec3::ZERO,
        radius: CORE_SRC_RADIUS,
        at_set: false,
        at_vs_player: false,
        at_hit: false,
        tg_set: true,
        tg_type: AT_TYPE_ALL
            & !AT_TYPE_WATER
            & !AT_TYPE_UNK20000
            & !AT_TYPE_WIND
            & !AT_TYPE_UNK400000
            & !AT_TYPE_LIGHT,
        tg_hit: false,
        co_set: true,
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cyl {
    pub centre: Vec3,
    pub radius: f32,
    pub height: f32,
    pub at_type: u32,
    pub at_atp: u8,
    pub at_set: bool,
    pub at_vs_player: bool,
    pub at_hit: bool,
    pub tg_set: bool,
    pub co_set: bool,
}

impl Cyl {
    const SRC: Cyl = Cyl {
        centre: Vec3::new(OFFSCREEN_X, OFFSCREEN_Y, OFFSCREEN_Z),
        radius: COCYL_SRC_RADIUS,
        height: COCYL_SRC_HEIGHT,
        at_type: AT_TYPE_UNK800,
        at_atp: AT_POWER_CO_CYL,
        at_set: true,
        at_vs_player: true,
        at_hit: false,
        tg_set: false,
        co_set: true,
    };
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct KalleDemos {
    pub pos: Vec3,
    pub home_pos: Vec3,
    pub angle: [i16; 3],
    pub shape_angle: [i16; 3],
    pub speed: Vec3,
    pub speed_f: f32,
    pub gravity: f32,
    pub eye_pos: Vec3,
    pub room: i8,
    pub set_id: u16,
    pub health: i32,
    pub max_health: i32,
    pub status: u32,
    pub attention_flags: u32,
    pub attention: Vec3,
    pub attention_distance: i32,
    pub hio_08: f32,
    pub hio_0c: f32,
    pub hio_14: i16,
    pub env_frame: f32,
    pub makar_mode: i8,
    pub makar_pos: Vec3,
    pub makar_yaw: i16,
    pub makar_anm: i32,
    pub makar_morph: Frame,
    pub clock: i16,
    pub mode: i16,
    pub state: i16,
    pub vine: i16,
    pub core_state: i16,
    pub timers: [i16; TIMER_COUNT],
    pub core_guard: i16,
    pub cut_timer: i16,
    pub invuln: i16,
    pub target_pos: Vec3,
    pub bob: f32,
    pub ground: f32,
    pub smoke_on: u8,
    pub open_smoke: bool,
    pub vine_count: i8,
    pub camera_type: i8,
    pub attack_timer: i16,
    pub shake_yaw: i16,
    pub shake_speed: f32,
    pub body_tg: i16,
    pub body_tg_radius: f32,
    pub body_sph: Sph,
    pub co_cyl: Cyl,
    pub core_sph: Sph,
    pub core_sway: [Vec3; 2],
    pub sway_clock: i16,
    pub sway_a: f32,
    pub sway_b: f32,
    pub core_pos: Vec3,
    pub core_step: Vec3,
    pub core_yaw: i16,
    pub core_pitch: i16,
    pub plant_lean: i16,
    pub play_count: i8,
    pub core_fx: [bool; CORE_FX_COUNT],
    pub core_fx_life: [i16; CORE_FX_COUNT],
    pub brk_on: u8,
    pub camera_on: i8,
    pub blure: i16,
    pub demo: i16,
    pub demo_count: i16,
    pub demo_step: i16,
    pub cam_eye: Vec3,
    pub cam_centre: Vec3,
    pub cam_pitch: i16,
    pub cam_yaw: i16,
    pub cam_fov: f32,
    pub cam_zoom: f32,
    pub cam_dist: f32,
    pub cam_shake: f32,
    pub cam_roll: i16,
    pub cam_hold_eye: Vec3,
    pub cam_hold_centre: Vec3,
    pub wind_gap: f32,
    pub wind_radius: f32,
    pub wind_strength: f32,
    pub env_amount: f32,
    pub env_kind: u8,
    pub env_from: i32,
    pub env_to: i32,
    pub env_timeset: f32,
    pub anm: i32,
    pub body_morph: Frame,
    pub head_anm: i32,
    pub head_morph: Frame,
    pub dead_anm: i32,
    pub dead_morph: Frame,
    pub player_demo: i32,
    pub player_riding: bool,
    pub player_damage: u8,
    pub pause_timer: i16,
    pub shock_time: i16,
    pub shock_pitch: i16,
    pub shock_dir: Vec3,
    pub shock_timer: i16,
    pub entered: bool,
    pub created: bool,
    pub blocked: bool,
    pub camera_kind: u8,
    pub demo_ordered: bool,
    pub seed: [i32; 3],
}

impl KalleDemos {
    pub fn spawn(_parameters: u32) -> Self {
        let mut body = Frame::idle();
        body.init(frame_max(BCK_HANA_WAIT));
        let mut head = Frame::idle();
        head.init(frame_max(BCK_COA_WAIT));
        let mut dead = Frame::idle();
        dead.init(frame_max(BCK_COA_DEAD1));
        let mut makar = Frame::idle();
        makar.init(frame_max(BCK_CALL_01));
        Self {
            pos: Vec3::ZERO,
            home_pos: Vec3::ZERO,
            angle: [0; 3],
            shape_angle: [0; 3],
            speed: Vec3::ZERO,
            speed_f: 0.0,
            gravity: 0.0,
            eye_pos: Vec3::ZERO,
            room: 0,
            set_id: 0,
            health: CORE_HEALTH,
            max_health: CORE_HEALTH,
            status: 0,
            attention_flags: 0,
            attention: Vec3::ZERO,
            attention_distance: ATTN_BATTLE_DISTANCE,
            hio_08: HIO_M08,
            hio_0c: HIO_M0C,
            hio_14: HIO_M14,
            env_frame: 0.0,
            makar_mode: MAKAR_OFF,
            makar_pos: Vec3::ZERO,
            makar_yaw: 0,
            makar_anm: BCK_CALL_01,
            makar_morph: makar,
            clock: 0,
            mode: MODE_START,
            state: 0,
            vine: VINE_FULL,
            core_state: CORE_IDLE,
            timers: [0; TIMER_COUNT],
            core_guard: 0,
            cut_timer: 0,
            invuln: 0,
            target_pos: Vec3::ZERO,
            bob: 0.0,
            ground: 0.0,
            smoke_on: 0,
            open_smoke: false,
            vine_count: 0,
            camera_type: 0,
            attack_timer: 0,
            shake_yaw: 0,
            shake_speed: 0.0,
            body_tg: 0,
            body_tg_radius: 0.0,
            body_sph: Sph::BODY_SRC,
            co_cyl: Cyl::SRC,
            core_sph: Sph::CORE_SRC,
            core_sway: [Vec3::ZERO; 2],
            sway_clock: 0,
            sway_a: 0.0,
            sway_b: 0.0,
            core_pos: Vec3::ZERO,
            core_step: Vec3::ZERO,
            core_yaw: 0,
            core_pitch: 0,
            plant_lean: 0,
            play_count: 0,
            core_fx: [false; CORE_FX_COUNT],
            core_fx_life: [0; CORE_FX_COUNT],
            brk_on: 0,
            camera_on: 0,
            blure: 0,
            demo: DEMO_OFF,
            demo_count: 0,
            demo_step: 0,
            cam_eye: Vec3::ZERO,
            cam_centre: Vec3::ZERO,
            cam_pitch: 0,
            cam_yaw: 0,
            cam_fov: 55.0,
            cam_zoom: 0.0,
            cam_dist: 0.0,
            cam_shake: 0.0,
            cam_roll: 0,
            cam_hold_eye: Vec3::ZERO,
            cam_hold_centre: Vec3::ZERO,
            wind_gap: 0.0,
            wind_radius: WIND_RADIUS,
            wind_strength: 0.0,
            env_amount: 0.0,
            env_kind: ENV_NONE,
            env_from: 0,
            env_to: 6,
            env_timeset: HIO_M0C,
            anm: BCK_HANA_WAIT,
            body_morph: body,
            head_anm: BCK_COA_WAIT,
            head_morph: head,
            dead_anm: BCK_COA_DEAD1,
            dead_morph: dead,
            player_demo: 0,
            player_riding: false,
            player_damage: 0,
            pause_timer: 0,
            shock_time: 0,
            shock_pitch: 0,
            shock_dir: Vec3::new(0.0, 1.0, 0.0),
            shock_timer: 0,
            entered: false,
            created: false,
            blocked: false,
            camera_kind: CAMERA_BOSS,
            demo_ordered: false,
            seed: [RND_SEED; 3],
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    fn rnd_f(&mut self, range: f32) -> f32 {
        rnd(&mut self.seed) * range
    }

    fn rnd_fx(&mut self, range: f32) -> f32 {
        range * (rnd(&mut self.seed) - 0.5) * 2.0
    }

    fn anm_init(&mut self, bck: i32, morf: f32, loop_mode: u8, speed: f32) {
        self.anm = bck;
        self.body_morph.init(frame_max(bck));
        self.body_morph.attribute = loop_mode;
        self.body_morph.rate = speed;
        self.body_morph.set_morf(morf);
    }

    fn head_anm_init(&mut self, bck: i32, morf: f32, loop_mode: u8, speed: f32) {
        self.head_anm = bck;
        self.head_morph.init(frame_max(bck));
        self.head_morph.attribute = loop_mode;
        self.head_morph.rate = speed;
        self.head_morph.set_morf(morf);
    }

    fn dead_anm_init(&mut self, bck: i32, morf: f32, loop_mode: u8, speed: f32) {
        self.dead_anm = bck;
        self.dead_morph.init(frame_max(bck));
        self.dead_morph.attribute = loop_mode;
        self.dead_morph.rate = speed;
        self.dead_morph.set_morf(morf);
    }

    fn makar_anm_init(&mut self, bck: i32, morf: f32, loop_mode: u8, speed: f32) {
        self.makar_anm = bck;
        self.makar_morph.init(frame_max(bck));
        self.makar_morph.attribute = loop_mode;
        self.makar_morph.rate = speed;
        self.makar_morph.set_morf(morf);
    }

    fn body_frame(&self) -> i16 {
        self.body_morph.frame as i32 as i16
    }

    fn dead_frame(&self) -> i32 {
        self.dead_morph.frame as i32
    }

    fn makar_frame(&self) -> i32 {
        self.makar_morph.frame as i32
    }

    fn player_demo_set(&mut self, mode: i32) {
        self.player_demo = mode;
    }

    fn camera_hold(&mut self, env: &Env<'_>) {
        self.cam_hold_eye = env.pos();
        self.cam_hold_centre = env.pos();
    }

    fn damage(&mut self, env: &mut Env<'_>, sink: &mut Sink<'_>) {
        let mut grab = false;
        match self.state {
            0 => {
                self.state = 1;
                self.timers[0] = 0x28;
                self.speed.y = 0.0;
                self.gravity = 0.0;
                self.damage(env, sink);
            }
            1 => {
                if self.timers[0] == 0x1e {
                    self.gravity = -3.0;
                    self.camera_type = 3;
                }
                if self.pos.y <= self.ground + 5.0 {
                    self.anm_init(BCK_HANA_OTIRU, 2.0, EMODE_NONE, 1.0);
                    self.vine = VINE_OUT;
                    self.state = 2;
                    self.core_yaw = env.angle_y(self.pos);
                    self.wind_gap = WIND_MAX;
                    if env.stage_x {
                        sink.bgm(BGM_UNK_151);
                    } else {
                        sink.bgm(BGM_UNK_105);
                    }
                }
            }
            2 => {
                if self.body_frame() == REG0_S1 + 10
                    && env.distance(self.pos) < REG0_F8 + 400.0
                {
                    self.body_tg = REG0_S2 + 0x14;
                    self.invuln = 0x1e;
                }
                if self.body_frame() == 30 {
                    self.open_smoke = true;
                }
                if self.body_frame() == 10 {
                    self.wind_gap = WIND_MAX;
                }
                if self.body_morph.is_stop() {
                    self.anm_init(BCK_HIRAKU_WAIT, 5.0, EMODE_LOOP, 1.0);
                    self.state = 3;
                    self.timers[0] = self.hio_14;
                    self.play_count = 0;
                }
                if self.body_frame() > 25 {
                    add_calc2(&mut self.env_amount, 1.0, 1.0, 0.1);
                }
            }
            3 => {
                self.camera_on = 2;
                if self.timers[0] == 0 {
                    self.state = 4;
                    self.anm_init(BCK_HANA_TOJIRU, 5.0, EMODE_NONE, 1.0);
                    self.vine = VINE_UP;
                    sink.se(SE_CM_BKM_FLW_WAVING);
                }
                add_calc2(&mut self.env_amount, 1.0, 1.0, 0.1);
            }
            4 => {
                if self.body_frame() == 23 {
                    sink.se(SE_CM_BKM_BODY_CLOSE);
                }
                if self.body_frame() == 27 {
                    sink.stage_bgm(env.stage_x);
                }
                let frame = self.body_frame();
                if frame <= 15 {
                    add_calc2(&mut self.env_amount, 1.0, 1.0, 0.1);
                }
                if frame == 15 {
                    self.wind_gap = WIND_MAX;
                }
                let step;
                if frame < REG0_S5 + 0x14 {
                    step = REG0_S3 + 200;
                    self.play_count = 0;
                } else {
                    self.vine = VINE_FULL;
                    step = REG0_S4 + 0x290;
                    if self.play_count != 0 && REG0_S9 == 0 {
                        self.play_count -= 1;
                        self.mode = MODE_EAT;
                        self.state = 0;
                        return;
                    }
                }
                add_calc_angle_s2(&mut self.plant_lean, -0x4000, 1, step);
                if self.body_morph.is_stop() {
                    self.state = 5;
                    self.timers[0] = 0x1e;
                    grab = true;
                }
            }
            5 => {
                add_calc_angle_s2(&mut self.plant_lean, -0x4000, 1, 0x290);
                if self.timers[0] == 0 {
                    self.mode = MODE_WAIT;
                    self.state = -1;
                    self.timers[0] = 0x3c;
                    self.camera_type = 4;
                    grab = true;
                }
            }
            _ => {}
        }
        if self.state >= 2 && self.state <= 4 {
            self.core_fx_life[0] = 3;
        }
        add_calc_angle_s2(&mut self.shape_angle[ANGLE_X], 0, 0x10, 0x200);
        add_calc_angle_s2(&mut self.shape_angle[ANGLE_Z], 0, 0x10, 0x200);
        let step = turned(self.angle[ANGLE_Y], [0.0, 0.0, self.speed_f]);
        self.pos.x += step.x;
        self.pos.z += step.z;
        self.pos.y += self.speed.y;
        self.speed.y = self.speed.y + self.gravity;
        if self.pos.y < self.ground {
            self.pos.y = self.ground;
            if self.speed.y < -20.0 {
                self.shock_time = REG0_S2 + SHAKE_FRAME;
                self.shock_pitch = -0x21;
                self.shock_dir = Vec3::new(0.0, 1.0, 0.0);
                sink.se(SE_CM_BKM_BODY_FALL);
                self.speed.y = 10.0;
                self.smoke_on = 1;
            } else {
                self.speed.y = 0.0;
            }
            add_calc2(&mut self.speed_f, 0.0, 1.0, 0.2);
        }
        if grab && env.distance(self.pos) < REG8_F8 + 200.0 {
            self.mode = MODE_EAT;
            self.state = 0;
            self.anm_init(BCK_HANA_TOJIRU, 1.0, EMODE_NONE, 5.0);
        }
    }

    fn eat(&mut self, env: &mut Env<'_>, sink: &mut Sink<'_>) {
        match self.state {
            0 => {
                self.state = 1;
                self.demo = 1;
                self.vine = VINE_UP;
                self.blure = 0x96;
                self.eat(env, sink);
            }
            1 => {
                add_calc_angle_s2(&mut self.plant_lean, -0x4000, 1, 0x1000);
                if self.body_morph.is_stop() {
                    self.state = 2;
                    self.anm_init(BCK_HANA_EAT, 10.0, EMODE_LOOP, 1.0);
                    self.timers[0] = 0x32;
                    sink.stage_bgm(env.stage_x);
                    sink.se(SE_CM_BKM_BODY_EATING);
                }
            }
            2 => {
                if self.timers[0] != 0 {
                    return;
                }
                self.state = 3;
                self.anm_init(BCK_HANA_HAKIDASU, 2.0, EMODE_NONE, 1.0);
                sink.se(SE_CM_BKM_LINK_OUT);
                return;
            }
            3 => {
                if self.demo_step == REG0_S2 + 0x50 {
                    self.smoke_on = 1;
                    self.invuln = 100;
                }
                if self.demo_step == 0x55 {
                    let mut at = self.pos;
                    at.y += REG17_F0 + THROW_LIFT;
                    let angle = self.cam_yaw.wrapping_add(THROW_ANGLE).wrapping_add(REG0_S6);
                    if env.event_run {
                        env.place(at, angle);
                        env.player.speed = REG17_F1 + THROW_SPEED;
                        env.player.speed_y = REG17_F2 + THROW_RISE;
                        self.player_damage = THROW_DAMAGE;
                        sink.hurt_player(THROW_DAMAGE);
                    }
                    self.body_tg = REG0_S2 + 0x14;
                }
                if self.body_morph.is_stop() {
                    self.state = 4;
                }
            }
            4 => {
                if self.demo_count > REG0_S7 + 0x82 {
                    self.state = 5;
                    self.timers[0] = 0x1a;
                }
            }
            5 => {
                if env.distance_flat(self.pos) < EAT_STAND {
                    let spot = turned(self.cam_yaw, [0.0, 0.0, THROW_REACH]) + self.pos;
                    env.place(spot, env.yaw());
                }
                if self.timers[0] == 0 {
                    self.mode = MODE_WAIT;
                    self.state = -1;
                    self.timers[0] = 0x3c;
                    self.demo = 0x96;
                    sink.event_reset();
                    self.plant_lean = -0x4000;
                    self.camera_type = 4;
                    self.invuln = 0;
                    self.blure = 1;
                }
            }
            _ => {}
        }
    }

    fn move1(&mut self) {
        let step = turned(self.angle[ANGLE_Y], [0.0, 0.0, self.speed_f]);
        self.speed = step;
        self.pos += self.speed;
        let clock = i32::from(self.clock);
        add_calc2(
            &mut self.bob,
            sin_s(sni(clock * 400)) * 20.0,
            0.1,
            10.0,
        );
        add_calc_angle_s2(
            &mut self.shape_angle[ANGLE_X],
            sn(sin_s(sni(clock * 500)) * 700.0),
            0x10,
            0x80,
        );
        add_calc_angle_s2(
            &mut self.shape_angle[ANGLE_Z],
            sn(cos_s(sni(clock * 300)) * 700.0),
            0x10,
            0x80,
        );
    }

    fn start(&mut self, env: &mut Env<'_>, sink: &mut Sink<'_>) {
        self.core_guard = 8;
        match self.state {
            0 => {
                self.anm_init(BCK_HIRAKU_WAIT, 1.0, EMODE_LOOP, 1.0);
                self.head_anm_init(BCK_COA_START1, 1.0, EMODE_LOOP, 1.0);
                self.state = 1;
                self.vine = VINE_UP;
                self.makar_mode = 1;
                self.start(env, sink);
            }
            1 => {
                self.env_amount = 1.0;
                let yaw = env.angle_y(self.pos);
                self.core_yaw = yaw;
                self.makar_pos = turned(yaw, MAKAR_OFFSET) + self.pos;
                self.makar_yaw = self.core_yaw;
                if env.distance(self.pos) < 500.0 {
                    self.state = 2;
                    self.demo = 5;
                    self.plant_lean = -0x4000;
                    sink.bgm(BGM_KINDAN_BOSS);
                }
            }
            2 => {
                self.invuln = 0x1e;
                let yaw = env.angle_y(self.pos);
                self.core_yaw = yaw;
                self.makar_pos = turned(yaw, MAKAR_OFFSET) + self.pos;
                self.makar_yaw = self.core_yaw;
                self.env_amount = 1.0;
            }
            3 => {
                self.invuln = 0x1e;
                if self.demo_count < 0xfa {
                    self.env_amount = 1.0;
                }
                if self.demo_count == 0x78 {
                    self.anm_init(BCK_HANA_START1, 1.0, EMODE_LOOP, 1.0);
                    sink.se(SE_CM_BKM_FLW_WAVING);
                }
                if self.demo_count == 0x96 {
                    self.anm_init(BCK_HANA_START2, 10.0, EMODE_NONE, 1.0);
                    self.wind_gap = WIND_MAX;
                    sink.se(SE_CM_BKM_FLW_TO_BUD);
                }
                if self.demo_count == 0xb1 {
                    self.smoke_on = 1;
                }
                if self.demo_count >= 0x96 {
                    if self.vine_count > 0 {
                        self.move1();
                    }
                    if self.demo_count == 0xbe {
                        self.anm_init(BCK_HANA_WAIT, 20.0, EMODE_LOOP, 1.0);
                    }
                }
                if self.demo_count == 0xd2 {
                    self.camera_type = 5;
                    sink.se(SE_CM_BKM_VINE_OUT);
                }
                if self.demo_count == 0x1c2 {
                    self.camera_type = 6;
                    sink.se(SE_CM_BKM_ATKVINE_ENTER);
                }
                if self.demo_count == 0x168 {
                    sink.se(SE_CM_BKM_BODY_UP);
                }
            }
            _ => {}
        }
    }

    fn end(&mut self, env: &mut Env<'_>, sink: &mut Sink<'_>) {
        self.plant_lean = 0;
        self.vine = VINE_OUT;
        match self.state {
            0 => {
                if env.stage_x {
                    sink.leave = Some(self.room as u8);
                    sink.se(SE_LK_B_BOSS_WARP);
                    sink.stories.set(EVENT_KALLE_DEMOS_TRIALS_CLEAR);
                    sink.flags.set(EVENT_TMP_0480);
                    return;
                }
                sink.bgm(STRM_BOSS_CLEAR);
                self.anm_init(BCK_HANA_DEAD1, 10.0, EMODE_LOOP, 1.0);
                self.state = 1;
                self.camera_type = 8;
                self.invuln = 30000;
                self.end(env, sink);
            }
            1 => {
                if self.demo_count < 0x1a4 {
                    self.env_amount = 1.0;
                }
                if self.demo_count == REG0_S2 + 300 {
                    self.player_demo_set(DEMO_UNKNOWN_29);
                }
                if self.demo_count == REG0_S4 + 0x17c {
                    self.anm_init(BCK_HANA_DEAD2, 10.0, EMODE_LOOP, 1.0);
                    sink.se(SE_CM_BKM_END_FLW_WAVING);
                }
                if self.demo_count == REG0_S5 + 0x1c2 {
                    self.anm_init(BCK_HANA_DEAD3, 10.0, EMODE_NONE, 1.0);
                    sink.se(SE_CM_BKM_END_FLW_LIFTUP);
                    self.wind_gap = WIND_MAX;
                    self.camera_type = 9;
                    self.smoke_on = 1;
                }
                if self.demo_count == 0x230 {
                    sink.se(SE_CM_BKM_END_FLW_DIE);
                }
                if self.demo_count == REG0_S6 + 0x276 {
                    sink.on_stage_boss_enemy();
                    sink.spawn_warp(self.pos, [0; 3], self.room, self.set_id);
                    self.state = 2;
                    self.camera_on = 0;
                }
            }
            _ => {}
        }
    }

    fn cc_calc(&mut self, env: &mut Env<'_>, sink: &mut Sink<'_>) {
        let reach = env.blow.at.reach();
        self.core_sph.tg_hit = self.core_sph.tg_set
            && touches(self.core_sph.centre, self.core_sph.radius, env.pos(), reach);
        self.co_cyl.at_hit = self.co_cyl.at_set
            && self.co_cyl.at_vs_player
            && cyls_touch(
                env.pos(),
                PLAYER_TG_RADIUS,
                PLAYER_TG_HEIGHT,
                self.co_cyl.centre,
                self.co_cyl.radius,
                self.co_cyl.height,
            );
        if self.co_cyl.at_hit {
            sink.hurt_player(self.co_cyl.at_atp);
        }
    }

    fn core_damage_check(&mut self, env: &mut Env<'_>, sink: &mut Sink<'_>) {
        if self.core_guard != 0 {
            return;
        }
        let mut dead = false;
        if self.core_sph.tg_hit {
            self.core_guard = 8;
            sink.se(SE_CV_BKM_DAMAGE);
            self.core_state = CORE_HIT;
            let power = env.blow.at.power(env.blow.blade, env.blow.soup);
            if power > 0 {
                self.health -= power;
            }
            let diff = self.eye_pos - env.pos();
            let yaw = to_s16((-diff.x).atan2(-diff.z));
            self.core_sway[0].y = yaw as f32;
            self.core_fx_life[1] = 10;
            if self.health <= 0 {
                dead = true;
            }
        }
        let flat = env.pos() - self.pos;
        let flat = Vec3::new(flat.x, 0.0, flat.z);
        let watered = env.forest_water && self.vine != 0 && flat.length() < 300.0;
        if !watered && !dead {
            return;
        }
        if !dead {
            sink.se(SE_LK_LAST_HIT);
        }
        self.core_state = CORE_DYING;
        self.mode = MODE_END;
        self.state = 0;
        self.demo = 100;
        self.pause_timer = 8;
        self.core_guard = 20000;
        self.env_kind = ENV_DARK;
        self.env_amount = 1.0;
        sink.stop_bgm();
    }

    fn core_move(&mut self, env: &mut Env<'_>, sink: &mut Sink<'_>) {
        let mut leap = false;
        self.core_damage_check(env, sink);
        let arm = self.core_state;
        loop {
            match arm {
                CORE_IDLE => break,
                CORE_HIT => {
                    self.head_anm_init(BCK_COA_DAMAGE, 1.0, EMODE_NONE, 1.0);
                    self.core_state = CORE_HIT_WAIT;
                    self.sway_clock = 0;
                    self.sway_a = REG0_F9 + 6000.0;
                    self.sway_b = 9000.0;
                    break;
                }
                CORE_HIT_WAIT => {
                    if self.sway_a.abs() < 200.0 {
                        self.core_state = CORE_IDLE;
                        self.head_anm_init(BCK_COA_WAIT, 5.0, EMODE_LOOP, 1.0);
                    }
                    break;
                }
                CORE_DYING => {
                    self.dead_anm_init(BCK_COA_DEAD1, 5.0, EMODE_NONE, 1.0);
                    self.core_state = 0x65;
                    self.core_pos = self.pos;
                    let yaw = self
                        .shape_angle[ANGLE_Y]
                        .wrapping_add(MAKAR_YAW_OFFSET)
                        .wrapping_add(REG0_S4);
                    self.core_step = turned(yaw, [0.0, 0.0, REG0_F15 + 7.0]);
                    self.core_step.y = REG0_F11 + 50.0;
                    self.core_pitch = 0;
                    self.core_yaw = 0;
                    self.timers[2] = 0;
                    sink.se(SE_CV_BKM_LAST_DAMAGE);
                    break;
                }
                0x65 => {
                    self.core_fx_life[2] = 3;
                    self.core_yaw = self.core_yaw.wrapping_add(0x800);
                    self.core_pitch = self.core_pitch.wrapping_add(0x500);
                    self.core_pos += self.core_step;
                    self.core_step.y -= REG0_F12 + 1.0;
                    if self.core_pos.y <= (self.ground - 10.0) + REG0_F13 {
                        self.core_pos.y = (self.ground - 10.0) + REG0_F13;
                        self.core_state = 0x66;
                        sink.bgm(BGM_KINDAN_BOSS);
                        self.dead_anm_init(BCK_COA_DEAD2, 1.0, EMODE_NONE, 1.0);
                        self.core_step = Vec3::ZERO;
                        leap = true;
                        sink.se(SE_CV_BKM_FALLDOWN);
                        sink.se(SE_CM_BKM_END_CORE_DROP);
                    }
                    break;
                }
                0x66 => {
                    if self.dead_frame() == 14 {
                        leap = true;
                        sink.se(SE_CV_BKM_JITABATA);
                    }
                    self.core_fx_life[2] = 3;
                    if self.dead_morph.is_stop() {
                        self.dead_anm_init(BCK_COA_NDEAD1, 1.0, EMODE_NONE, 1.0);
                        self.core_state = 0x67;
                        self.core_step.z = 10.0;
                        self.core_step.x = 10.0;
                        self.timers[1] = REG0_S6 + 0xc3;
                    }
                    self.core_drift();
                    break;
                }
                0x67 => {
                    if self.dead_frame() == 18 {
                        leap = true;
                        sink.se(SE_CV_BKM_JITABATA);
                    }
                    self.core_fx_life[2] = 3;
                    if !self.dead_morph.is_stop() {
                        self.core_drift();
                        break;
                    }
                    self.dead_anm_init(BCK_COA_NDEAD2, 1.0, EMODE_NONE, 1.0);
                    self.core_state = 0x68;
                    self.core_drift();
                    break;
                }
                0x68 => {
                    if self.dead_frame() == 17 || self.dead_frame() == 37 {
                        leap = true;
                        sink.se(SE_CV_BKM_JITABATA);
                    }
                    self.core_fx_life[2] = 10;
                    if !self.dead_morph.is_stop() {
                        self.core_drift();
                        break;
                    }
                    self.dead_anm_init(BCK_COA_LDEAD, 1.0, EMODE_NONE, 1.0);
                    self.core_state = 0x69;
                    self.core_step.z = 0.0;
                    self.core_step.x = 0.0;
                    self.core_drift();
                    break;
                }
                0x69 => {
                    if self.dead_frame() == 37 {
                        self.core_fx[1] = true;
                        self.core_fx_life[1] = 10;
                        sink.se(SE_CV_BKM_DIE);
                    }
                    if self.dead_morph.is_stop() {
                        let mut at = self.core_pos;
                        at.y += REG0_F2 + 80.0;
                        sink.spawn_item(
                            u32::from(ITEM_HEART_CONTAINER),
                            at,
                            [0; 3],
                            self.room,
                            self.set_id,
                        );
                        self.core_state = 0x6e;
                        self.timers[2] = 0xaa;
                    }
                    self.core_drift();
                    break;
                }
                0x6e => {
                    if self.timers[2] == 1 {
                        self.demo = self.demo.wrapping_add(1);
                        self.demo_step = 0;
                        self.core_state = 0x6f;
                    }
                    break;
                }
                _ => break,
            }
        }

        if self.core_state < 100 {
            let yaw = self
                .core_yaw
                .wrapping_add(-32768)
                .wrapping_sub(sn(self.core_sway[0].y));
            let step = turned(yaw, [0.0, 0.0, self.sway_a * sin_s(self.sway_clock)]);
            self.core_sway[0].x = step.z;
            self.core_sway[0].z = step.x;
            let other = self
                .sway_clock
                .wrapping_sub(20000)
                .wrapping_add(REG0_S6);
            let step = turned(yaw, [0.0, 0.0, self.sway_b * sin_s(other)]);
            self.core_sway[1].x = step.z;
            self.core_sway[1].z = step.x;
            self.sway_clock = self.sway_clock.wrapping_add(REG0_S5 + 6000);
            add_calc2(
                &mut self.sway_a,
                0.0,
                1.0,
                REG0_F10 + US_SWAY_RATE,
            );
            add_calc2(
                &mut self.sway_b,
                0.0,
                1.0,
                REG0_F10 + US_SWAY_RATE_ALT,
            );
            self.eye_pos = self.pos
                + turned(self.core_yaw, [0.0, CORE_LIFT + HEAD_OFFSET[1], HEAD_OFFSET[2]]);
            self.attention = self.eye_pos;
            self.attention.y += ATTENTION_LIFT;
            if self.vine != 0 && self.plant_lean == 0 && self.mode != MODE_START {
                self.attention_flags = ATTN_LOCKON_BATTLE;
            }
        } else {
            self.eye_pos = self.core_pos;
            if self.core_state >= 0x6F {
                self.eye_pos.y += 20000.0;
            }
        }
        self.core_sph.centre = self.eye_pos;
        self.core_sph.radius = CORE_RADIUS;
        if leap {
            self.core_fx[2] = true;
            self.core_fx_life[2] = 3;
            sink.se(SE_CM_BKM_END_CORE_LEAP);
        }
    }

    fn core_drift(&mut self) {
        self.core_pos += self.core_step;
        if self.timers[1] & 0xF == 0 {
            self.core_step.x *= -1.0;
            self.core_step.z *= -1.0;
        }
        add_calc_angle_s2(&mut self.core_yaw, 0, 2, 0x800);
        add_calc_angle_s2(&mut self.core_pitch, 0, 2, 0x800);
    }

    fn makar_voice(&mut self, sink: &mut Sink<'_>, id: u32) {
        sink.se(id);
    }

    fn mk_move(&mut self, sink: &mut Sink<'_>) {
        if self.makar_mode != 0 {
            match self.makar_mode {
                MAKAR_OFF => {
                    self.timers[3] = 0x96;
                }
                MAKAR_FIRST => {
                    if self.timers[3] == 0 {
                        self.makar_anm_init(BCK_CALL_01, 5.0, EMODE_LOOP, 1.0);
                        self.timers[3] = sn(self.rnd_f(90.0) + 60.0);
                        self.makar_mode = MAKAR_SECOND;
                        self.makar_voice(sink, SE_CV_CB_HELP);
                    }
                }
                MAKAR_SECOND => {
                    if self.timers[3] == 0 {
                        self.makar_anm_init(BCK_CALL_02, 5.0, EMODE_LOOP, 1.0);
                        self.timers[3] = sn(self.rnd_f(90.0) + 60.0);
                        self.makar_mode = MAKAR_FIRST;
                        self.makar_voice(sink, SE_CV_CB_HELP);
                    }
                }
                MAKAR_ENTER => {
                    self.makar_anm_init(BCK_CALL_01, 5.0, EMODE_LOOP, 1.0);
                    self.makar_mode = MAKAR_DONE;
                    self.makar_voice(sink, SE_CV_CB_HELP);
                }
                MAKAR_FOUR => {
                    self.makar_anm_init(BCK_CALL_03, 5.0, EMODE_LOOP, 1.0);
                    self.makar_mode = MAKAR_DONE;
                    self.makar_voice(sink, SE_CV_CB_HELP);
                }
                MAKAR_DROP => {
                    self.makar_anm_init(BCK_DROP, 1.0, EMODE_NONE, 1.0);
                    self.makar_mode = MAKAR_DROPING;
                }
                MAKAR_DROPING => {
                    if self.makar_frame() == 6 {
                        self.core_fx[2] = true;
                        self.core_fx_life[2] = 3;
                        self.makar_voice(sink, SE_CV_CB_SAVED);
                    }
                }
                MAKAR_DONE => {}
                _ => {}
            }
            self.makar_morph.play();
        }
    }

    fn damage_check(&mut self, sink: &mut Sink<'_>) {
        if self.vine_count == 0 && self.cut_timer == 1 {
            self.mode = MODE_DAMAGE;
            self.state = 0;
            sink.se(SE_CV_BKM_CUT_VINE_ALL);
        }
    }

    fn wait(&mut self, env: &mut Env<'_>, sink: &mut Sink<'_>) {
        if self.timers[3] == 1 {
            sink.se(SE_CM_BKM_BODY_UP);
        }
        match self.state {
            -1 => {
                self.shake_yaw = self.shape_angle[ANGLE_Y];
                self.timers[1] = 100;
                self.timers[2] = 0x28;
                if self.timers[0] == 0 {
                    self.state = 0;
                    self.vine = VINE_NONE;
                    self.anm_init(BCK_HANA_WAIT, 20.0, EMODE_LOOP, 1.0);
                    sink.se(SE_CM_BKM_VINE_OUT);
                    self.timers[3] = 0x5a;
                }
            }
            0 => {
                self.plant_lean = -0x4000;
                self.state = 1;
                self.target_pos = self.home_pos;
                self.target_pos.x += self.rnd_fx(500.0);
                self.target_pos.z += self.rnd_fx(500.0);
                let diff = self.target_pos - env.pos();
                let far = (diff.x * diff.x + diff.z * diff.z).sqrt();
                if far > REG0_F11 + 1000.0 {
                    let step = turned(
                        to_s16(diff.x.atan2(diff.z)),
                        [0.0, 0.0, REG0_F12 + 1300.0],
                    );
                    self.target_pos = env.pos() + step;
                }
                self.wait(env, sink);
            }
            1 => {
                let speed = f32::from(self.vine_count) * 0.15;
                add_calc2(&mut self.speed_f, speed, 1.0, 0.1);
                let diff = self.target_pos - self.pos;
                add_calc_angle_s2(
                    &mut self.angle[ANGLE_Y],
                    to_s16(diff.x.atan2(diff.z)),
                    0x10,
                    0x800,
                );
                if reach_flat(self.pos, self.target_pos) < 50.0 {
                    self.state = 2;
                    self.timers[0] = sn(self.rnd_f(150.0) + 150.0);
                }
            }
            2 => {
                self.smoke_on = 1;
                add_calc2(&mut self.speed_f, 0.0, 1.0, 0.1);
                if self.timers[0] == 0 {
                    self.state = 0;
                }
            }
            _ => {}
        }
        if self.state >= 0 {
            self.move1();
            if self.timers[1] == 0 {
                self.timers[1] = sn(self.rnd_f(200.0) + 100.0);
                self.shake_yaw = sn(self.rnd_fx(32768.0));
                self.shake_speed = 0.0;
            }
            add_calc_angle_s2(
                &mut self.shape_angle[ANGLE_Y],
                self.shake_yaw,
                0x20,
                sn(self.shake_speed),
            );
            add_calc2(&mut self.shake_speed, 64.0, 1.0, 1.0);
            if self.timers[2] == 0 {
                let near = env.distance(self.pos) < 1300.0;
                self.timers[2] = sn(self.rnd_f(50.0) + 30.0);
                if self.rnd_f(1.0) < self.hio_08 {
                    self.mode = if near { MODE_ATTACK_1 } else { MODE_ATTACK_2 };
                    self.state = 0;
                }
            }
        }
    }

    fn attack_1(&mut self) {
        match self.state {
            0 => {
                self.timers[0] = 0x32;
                self.state = 1;
                self.camera_type = 1;
            }
            1 => {
                if self.timers[0] == 0 {
                    self.mode = MODE_WAIT;
                    self.state = 0;
                }
            }
            _ => {}
        }
        add_calc2(&mut self.speed_f, 0.0, 1.0, 0.1);
        self.move1();
    }

    fn attack_2(&mut self) {
        match self.state {
            0 => {
                self.state = 1;
                self.camera_type = 2;
                self.attack_timer = 10;
                self.anm_init(BCK_HANA_ATTACK, 20.0, EMODE_LOOP, 1.0);
            }
            1 => {
                if self.attack_timer == 0 {
                    self.camera_type = 0;
                    self.mode = MODE_WAIT;
                    self.state = 0;
                    self.anm_init(BCK_HANA_WAIT, 20.0, EMODE_LOOP, 1.0);
                }
            }
            _ => {}
        }
        add_calc2(&mut self.speed_f, 0.0, 1.0, 0.1);
        self.move1();
    }

    fn move_state(&mut self, prev: Vec3, env: &mut Env<'_>, sink: &mut Sink<'_>) {
        self.status = 0;
        self.attention_flags = 0;
        self.camera_type = 0;
        let arm = self.mode;
        loop {
            match arm {
                MODE_WAIT => {
                    self.wait(env, sink);
                    break;
                }
                MODE_ATTACK_1 => {
                    self.attack_1();
                    break;
                }
                MODE_ATTACK_2 => {
                    self.attack_2();
                    break;
                }
                MODE_DAMAGE => {
                    self.damage(env, sink);
                    break;
                }
                MODE_EAT => {
                    self.eat(env, sink);
                    break;
                }
                MODE_START => {
                    self.start(env, sink);
                    break;
                }
                MODE_END => {
                    self.end(env, sink);
                    break;
                }
                _ => break,
            }
        }
        if let Some(solid) = env.solid {
            let next = solid.slide(prev, self.pos);
            if next.x != self.pos.x || next.z != self.pos.z {
                self.pos = next;
            }
        }
        if self.mode != MODE_DAMAGE {
            let mut lift = 0.0;
            if 0 < self.vine_count {
                lift = REG0_F18
                    + (f32::from(self.vine_count) * (REG0_F17 + VINE_LIFT_SCALE)
                        + VINE_LIFT_BIAS);
                if self.mode == MODE_START {
                    lift += 100.0;
                }
            }
            add_calc2(
                &mut self.pos.y,
                lift + (self.home_pos.y + self.bob),
                0.1,
                5.0,
            );
            self.damage_check(sink);
        } else {
            self.bob = 0.0;
        }
    }

    fn ride(&mut self, env: &mut Env<'_>) {
        if self.vine == VINE_FULL {
            self.player_riding = false;
            if self.touch_vine(env.pos()) {
                self.player_riding = true;
            }
        }
        if !self.player_riding {
            return;
        }
        add_calc2(&mut env.player.pos.x, self.pos.x, 1.0, REG0_F2 + RIDE_STEP);
        add_calc2(&mut env.player.pos.y, self.pos.y, 1.0, REG0_F2 + RIDE_STEP);
        add_calc2(&mut env.player.pos.z, self.pos.z, 1.0, REG0_F2 + RIDE_STEP);
        self.play_count = 5;
    }

    fn touch_vine(&self, at: Vec3) -> bool {
        for index in 0..VINE_COUNT {
            if reach_flat(self.vine_tip(index), at) < RIDE_RADIUS {
                return true;
            }
        }
        false
    }

    fn vine_tip(&self, index: i32) -> Vec3 {
        let extra = sn(index as f32 * VINE_YAW_STEP + REG0_F11);
        let yaw = self.shape_angle[ANGLE_Y].wrapping_add(extra);
        let lean = from_s16(self.plant_lean);
        let (sin, cos) = lean.sin_cos();
        let local = Vec3::new(
            VINE_REACH - VINE_REACH * cos,
            VINE_LIFT - VINE_REACH * sin,
            0.0,
        );
        if 0 < self.plant_lean || self.invuln != 0 {
            return self.pos;
        }
        self.pos + turned(yaw, local.to_array())
    }

    fn eff_cont(&mut self) {
        for index in 0..CORE_FX_COUNT {
            if self.core_fx[index] {
                if self.core_fx_life[index] != 0 {
                    self.core_fx_life[index] -= 1;
                    if self.core_fx_life[index] == 0 {
                        self.core_fx[index] = false;
                    }
                }
            } else if self.core_fx_life[index] != 0 {
                self.core_fx[index] = true;
            }
        }
    }

    fn demo_camera(&mut self, env: &mut Env<'_>, sink: &mut Sink<'_>) {
        let mut run = true;
        loop {
            match self.demo {
                DEMO_OFF => break,
                DEMO_START => {
                    if !self.demo_gate() {
                        run = false;
                        break;
                    }
                    self.demo = self.demo.wrapping_add(1);
                    self.camera_hold(env);
                    let diff = env.pos() - self.cam_hold_eye;
                    self.cam_yaw = to_s16(diff.x.atan2(diff.z));
                    self.cam_fov = 55.0;
                    self.player_demo_set(DEMO_ORIGINAL);
                }
                DEMO_MOVE => {
                    if self.demo_step < REG0_S0 + 0x55 {
                        add_calc2(&mut self.cam_centre.x, self.pos.x, 0.1, 200.0);
                        add_calc2(
                            &mut self.cam_centre.y,
                            env.pos().y + 300.0 + REG0_F5,
                            0.2,
                            200.0,
                        );
                        add_calc2(&mut self.cam_centre.z, self.pos.z, 0.2, 200.0);
                        let back = turned(
                            self.cam_yaw,
                            [
                                0.0,
                                REG0_F7 + CAM_LOOK_MOVE[1],
                                REG0_F6 + CAM_LOOK_MOVE[2],
                            ],
                        );
                        add_calc2(&mut self.cam_eye.x, self.pos.x + back.x, 0.1, 200.0);
                        add_calc2(&mut self.cam_eye.y, env.pos().y + back.y, 0.1, 200.0);
                        add_calc2(&mut self.cam_eye.z, self.pos.z + back.z, 0.1, 200.0);
                    } else if self.demo_step < REG0_S1 + 100 {
                        add_calc2(
                            &mut self.cam_centre.y,
                            self.pos.y + 1200.0 + REG0_F5,
                            0.2,
                            REG0_F6 + 100.0,
                        );
                    } else {
                        add_calc2(&mut self.cam_centre.x, env.pos().x, 0.3, 300.0);
                        add_calc2(
                            &mut self.cam_centre.y,
                            env.pos().y + 70.0 + REG0_F5,
                            0.3,
                            300.0,
                        );
                        add_calc2(&mut self.cam_centre.z, env.pos().z, 0.3, 300.0);
                    }
                    break;
                }
                DEMO_LOOK => {
                    if !self.demo_gate() {
                        run = false;
                        break;
                    }
                    self.demo = self.demo.wrapping_add(1);
                    self.cam_fov = 60.0;
                    self.player_demo_set(DEMO_ORIGINAL);
                }
                DEMO_CB1 => {
                    self.cam_eye = Vec3::from(CAM_CB1);
                    self.cam_centre = Vec3::from(CAM_CB1_LOOK);
                    env.place(
                        Vec3::new(
                            REG0_F7 + CAM_START_POS[0],
                            CAM_START_POS[1],
                            REG0_F8 + CAM_START_POS[2],
                        ),
                        CAM_START_ANGLE,
                    );
                    if self.demo_step == 0x1e {
                        self.demo_step = 0;
                        self.demo = self.demo.wrapping_add(1);
                        self.makar_mode = MAKAR_ENTER;
                    }
                    break;
                }
                DEMO_FALL => {
                    self.cam_eye = Vec3::from(CAM_FALL);
                    self.cam_centre = Vec3::from(CAM_FALL_LOOK);
                    if self.demo_step == 0x1e {
                        self.demo_step = 0;
                        self.demo = self.demo.wrapping_add(1);
                    }
                    break;
                }
                DEMO_CORE => {
                    self.cam_eye = Vec3::from(CAM_LIFTED);
                    self.cam_centre = Vec3::from(CAM_LOOK_A);
                    if self.demo_step == 5 {
                        self.player_demo_set(DEMO_SMILE);
                    }
                    if self.demo_step != 0x1e {
                        break;
                    }
                    self.demo_step = 0;
                    self.demo = self.demo.wrapping_add(1);
                    self.cam_eye = Vec3::from(CAM_FALL);
                    self.cam_centre = self.eye_pos;
                    self.cam_centre.y += REG0_F11 + DEMO_HEAD_LIFT;
                    self.makar_mode = MAKAR_FOUR;
                }
                DEMO_ENTER => {
                    add_calc2(
                        &mut self.cam_centre.y,
                        self.eye_pos.y + DEMO_HEAD_LIFT,
                        0.2,
                        REG0_F4 + 5.0,
                    );
                    if self.demo_step == 0x1e {
                        self.head_anm_init(BCK_NEW_SDEMO1, 1.0, EMODE_NONE, 1.0);
                        sink.se(SE_CM_BKM_CORE_ENTER);
                        sink.se(SE_CV_BKM_ENTER);
                    }
                    if self.demo_step > 0x1e && self.head_morph.is_stop() {
                        self.demo_step = 0;
                        self.demo = self.demo.wrapping_add(1);
                        self.head_anm_init(BCK_NEW_SDEMO2, 1.0, EMODE_LOOP, 1.0);
                    }
                    break;
                }
                DEMO_SMALL => {
                    self.cam_eye = Vec3::from(CAM_LIFTED);
                    self.cam_centre = Vec3::from(CAM_LOOK_A);
                    if self.demo_step == 5 {
                        self.player_demo_set(DEMO_SURPRISE_SMALL);
                    }
                    if self.demo_step != 0x19 {
                        break;
                    }
                    self.demo_step = 0;
                    self.demo = self.demo.wrapping_add(1);
                    self.cam_centre = self.eye_pos;
                    self.cam_centre.y += REG0_F11 + DEMO_HEAD_LIFT;
                }
                DEMO_SIT => {
                    self.cam_eye = Vec3::from(CAM_SIT);
                    self.cam_centre = Vec3::new(-93.0, 164.0, 104.0);
                    if self.demo_step == 0x1e {
                        self.head_anm_init(BCK_NEW_SDEMO3, REG0_F16 + 4.0, EMODE_NONE, 1.0);
                    }
                    if self.demo_step > 0x1b && self.head_morph.is_stop() {
                        self.demo_step = 0;
                        self.demo = self.demo.wrapping_add(1);
                        self.makar_mode = MAKAR_OFF;
                        self.makar_voice(sink, VOICE_CB_EATEN);
                        self.head_anm_init(BCK_NEW_SDEMO4, 1.0, EMODE_NONE, 1.0);
                        self.cam_centre = self.eye_pos;
                        self.cam_centre.y += REG0_F15 + DEMO_SDEMO_LIFT;
                    }
                    break;
                }
                DEMO_LAUGH => {
                    if self.demo_step >= 0x14 && self.demo_step <= 0x28 {
                        self.cam_eye = Vec3::from(CAM_LIFTED);
                        self.cam_centre = Vec3::from(CAM_LOOK_A);
                        if self.demo_step == 0x14 {
                            self.player_demo_set(DEMO_SURPRISE);
                        }
                        self.head_morph.rate = 0.0;
                        if self.demo_step == 0x28 {
                            self.cam_eye = Vec3::from(CAM_SIT);
                            self.cam_centre = self.eye_pos;
                            self.cam_centre.y += REG0_F15 + DEMO_SDEMO_LIFT;
                            self.head_morph.rate = 1.0;
                        }
                    } else {
                        add_calc2(&mut self.cam_fov, REG0_F12 + 35.0, 0.2, REG0_F13 + 1.0);
                        add_calc2(&mut self.cam_centre.x, self.eye_pos.x, 0.2, REG0_F4 + 10.0);
                        add_calc2(
                            &mut self.cam_centre.y,
                            self.eye_pos.y + 10.0 + REG0_F14,
                            0.2,
                            REG0_F4 + 10.0,
                        );
                        add_calc2(&mut self.cam_centre.z, self.eye_pos.z, 0.2, REG0_F4 + 10.0);
                        if self.head_morph.is_stop() {
                            self.demo_step = REG0_S6;
                            self.state = 3;
                            self.head_anm_init(BCK_COA_START3, 1.0, EMODE_LOOP, 1.0);
                            self.timers[0] = 10;
                            sink.se(SE_CV_BKM_LAUGH);
                            self.demo = 0xf;
                            self.cam_fov = REG0_F8 + 55.0;
                            self.demo_count = REG0_S7 + 0x6e;
                            self.cam_centre = Vec3::new(0.0, REG0_F3 + 500.0, 0.0);
                            self.cam_eye = turned(
                                self.shape_angle[ANGLE_Y],
                                [
                                    REG0_F5 + CAM_ENTER[0],
                                    REG0_F6 + CAM_ENTER[1],
                                    REG0_F7 + CAM_ENTER[2],
                                ],
                            );
                            self.cam_zoom = 0.0;
                        }
                    }
                    break;
                }
                DEMO_RETURN => {
                    if self.demo_count == REG0_S6 + 0x37 {
                        self.player_demo_set(DEMO_SURPRISE);
                    }
                    if self.demo_count == REG0_S7 + 100 {
                        self.player_demo_set(DEMO_LOOK_UP);
                    }
                    if self.demo_count >= REG0_S5 + 0xd2 {
                        if self.demo_count <= REG0_S6 + 0x159 {
                            add_calc2(&mut self.cam_fov, REG0_F6 + 40.0, 0.05, REG0_F7 + 0.1);
                        } else {
                            add_calc2(&mut self.cam_fov, REG0_F8 + 70.0, 0.2, REG0_F9 + 2.0);
                        }
                    }
                    if self.demo_count >= REG0_S5 + 0x96 {
                        add_calc2(
                            &mut self.cam_centre.y,
                            REG0_F3 + 450.0 + 150.0 + 50.0 + 100.0,
                            0.05,
                            self.cam_zoom * 4.5,
                        );
                        add_calc2(
                            &mut self.cam_eye.y,
                            (REG0_F4 + 50.0) - 50.0,
                            0.05,
                            self.cam_zoom * 3.0,
                        );
                        add_calc2(&mut self.cam_zoom, 1.0, 1.0, REG0_F10 + 0.01);
                    }
                    if self.demo_count == 0x24e {
                        self.demo = 0x96;
                        sink.event_reset();
                        self.camera_type = 7;
                        self.mode = MODE_WAIT;
                        self.state = -1;
                        self.timers[0] = 0x3c;
                        self.env_kind = ENV_NONE;
                        sink.stage_bgm(env.stage_x);
                        self.camera_on = 1;
                        sink.on_stage_boss_demo();
                    }
                    break;
                }
                DEMO_LAST => {
                    if !self.demo_gate() {
                        run = false;
                        break;
                    }
                    self.demo = self.demo.wrapping_add(1);
                    self.camera_hold(env);
                    self.cam_fov = 55.0;
                    self.player_demo_set(DEMO_ORIGINAL);
                }
                DEMO_CORE_FALL => {
                    if self.demo_step >= 0x140 {
                        add_calc2(&mut self.cam_fov, REG0_F8 + 85.0, 0.2, 1.0);
                        if self.demo_step == 0x140 {
                            self.makar_mode = MAKAR_DROP;
                        }
                    }
                    if self.core_pos.y < self.ground + 10.0 {
                        add_calc2(&mut self.cam_centre.x, self.eye_pos.x, 0.2, 200.0);
                        add_calc2(
                            &mut self.cam_centre.y,
                            self.eye_pos.y + 20.0 + REG0_F5,
                            0.5,
                            200.0,
                        );
                        add_calc2(&mut self.cam_centre.z, self.eye_pos.z, 0.2, 200.0);
                    } else {
                        add_calc2(&mut self.cam_centre.x, self.core_pos.x, 0.2, 200.0);
                        add_calc2(&mut self.cam_centre.y, self.core_pos.y, 0.5, 200.0);
                        add_calc2(&mut self.cam_centre.z, self.core_pos.z, 0.2, 200.0);
                    }
                    let yaw = self
                        .shape_angle[ANGLE_Y]
                        .wrapping_add(MAKAR_YAW_OFFSET)
                        .wrapping_add(REG0_S4);
                    let mut back = turned(yaw, MAKAR_OFFSET_TALL);
                    back += self.pos;
                    if back.y > self.ground + 300.0 + REG0_F8 {
                        back.y = self.ground + 300.0 + REG0_F8;
                    }
                    add_calc2(&mut self.cam_eye.x, back.x, 0.5, 200.0);
                    add_calc2(&mut self.cam_eye.y, back.y, 0.8, 200.0);
                    add_calc2(&mut self.cam_eye.z, back.z, 0.5, 200.0);
                    if self.demo_count > 0x3c {
                        let near = turned(
                            self.shape_angle[ANGLE_Y]
                                .wrapping_add(MAKAR_YAW_OFFSET)
                                .wrapping_add(REG0_S7),
                            [REG0_F11 + MAKAR_CATCH_DIST, 0.0, 0.0],
                        );
                        env.place(
                            near + self.pos,
                            self.shape_angle[ANGLE_Y]
                                .wrapping_add(MAKAR_YAW_OFFSET)
                                .wrapping_add(REG0_S6),
                        );
                    }
                    self.makar_pos = turned(yaw, [0.0, 0.0, MAKAR_OFFSET_CATCH[2]]) + self.pos;
                    self.makar_yaw = yaw;
                    break;
                }
                DEMO_CATCH => {
                    self.cam_fov = REG0_F8 + 55.0;
                    self.cam_yaw = self.shape_angle[ANGLE_Y].wrapping_add(REG0_S2);
                    self.cam_pitch = MAKAR_HURT_YAW;
                    self.cam_dist = 1500.0;
                    self.cam_zoom = 0.0;
                    self.demo = 0x67;
                    self.brk_on = 1;
                    self.makar_mode = MAKAR_OFF;
                    let angle = self.shape_angle[ANGLE_Y].wrapping_add(0x8BB8_u16 as i16);
                    sink.spawn_actor(MAKAR_ACTOR, 0, self.makar_pos, angle, self.room);
                }
                DEMO_ORBIT => {
                    let rate = sn(self.cam_zoom * 22384.0);
                    add_calc_angle_s2(
                        &mut self.cam_yaw,
                        self.shape_angle[ANGLE_Y]
                            .wrapping_add(0x5770)
                            .wrapping_add(REG0_S2),
                        0x10,
                        rate,
                    );
                    let rate = sn(self.cam_zoom * 3000.0);
                    add_calc_angle_s2(&mut self.cam_pitch, MAKAR_HURT_PITCH, 0x10, rate);
                    add_calc2(
                        &mut self.cam_dist,
                        500.0,
                        0.0625,
                        self.cam_zoom * 1000.0,
                    );
                    add_calc2(&mut self.cam_zoom, REG0_F10 + 0.0048, 0.1, 0.000048);
                    let back = tilted(
                        self.cam_pitch,
                        turned(self.cam_yaw, [0.0, 0.0, self.cam_dist]),
                    );
                    self.cam_eye = self.pos + back;
                    self.cam_centre = self.pos;
                    self.cam_centre.y += REG0_S3 as f32 + MAKAR_PAN_Y;
                    if self.demo_count == 800 {
                        self.demo = 0x96;
                        sink.event_reset();
                        self.invuln = 0;
                        self.player_demo = 0;
                    }
                    break;
                }
                DEMO_CLEAN => {
                    self.demo = DEMO_OFF;
                    break;
                }
                _ => break,
            }
            if !run {
                break;
            }
        }
        if !run {
            return;
        }
        if self.demo != 0 {
            let step = self.cam_shake * sin_s(sn(self.demo_step as f32 * SHAKE_ANGLE));
            let lift = self.cam_shake * cos_s(sn(self.demo_step as f32 * SHAKE_ANGLE_Z));
            self.cam_eye.x += step;
            self.cam_eye.y += lift;
            self.cam_centre.x += step;
            self.cam_centre.y += lift;
            let roll = sn(
                self.cam_shake * cos_s(self.clock.wrapping_mul(SHAKE_CLOCK)) * SHAKE_SWING,
            );
            self.cam_roll = roll;
            add_calc2(&mut self.cam_shake, 0.0, 1.0, REG0_F16 + 2.0);
            self.demo_step = self.demo_step.wrapping_add(1);
            self.demo_count = self.demo_count.wrapping_add(1);
        }
    }

    fn demo_gate(&mut self) -> bool {
        if !self.demo_ordered {
            self.demo_ordered = true;
            return false;
        }
        true
    }

    fn bmd_kankyo(&mut self) {
        self.env_timeset = self.hio_0c;
        match self.env_kind {
            ENV_NONE => {
                self.env_from = 0;
                self.env_to = 6;
                add_calc0(&mut self.env_amount, 1.0, ENV_STEP_A);
                self.env_frame = (1.0 - self.env_amount) * 11.0;
            }
            ENV_FADE => {
                self.env_from = 0;
                self.env_to = 4;
                add_calc0(&mut self.env_amount, 1.0, ENV_STEP_A);
                self.env_frame = (1.0 - self.env_amount) * 11.0;
            }
            ENV_DARK => {
                self.env_from = 5;
                self.env_to = 6;
                add_calc0(&mut self.env_amount, 1.0, ENV_STEP_B);
                self.env_frame = self.env_amount * 11.0;
            }
            _ => {}
        }
    }

    fn wind(&mut self) {
        add_calc2(&mut self.wind_gap, 0.0, 1.0, WIND_STEP);
        self.wind_radius = WIND_RADIUS - self.wind_gap;
        if self.wind_gap > 1.0 {
            self.wind_strength = 1.0;
            if self.wind_radius > WIND_FULL {
                self.wind_strength = (WIND_RADIUS - self.wind_radius) * WIND_RATE;
            }
        } else {
            self.wind_strength = 0.0;
        }
    }

    fn execute(&mut self, prev: Vec3, env: &mut Env<'_>, sink: &mut Sink<'_>) {
        if env.stage_x {
            self.blure = 0x32;
        }
        self.clock = self.clock.wrapping_add(1);
        self.smoke_on = 0;
        self.cc_calc(env, sink);
        for index in 0..TIMER_COUNT {
            if self.timers[index] != 0 {
                self.timers[index] -= 1;
            }
        }
        if self.core_guard != 0 {
            self.core_guard -= 1;
        }
        if self.invuln != 0 {
            self.invuln -= 1;
        }
        if self.cut_timer != 0 {
            self.cut_timer -= 1;
        }
        if self.attack_timer != 0 {
            self.attack_timer -= 1;
        }
        if self.pause_timer != 0 {
            self.pause_timer -= 1;
        }
        if self.shock_timer != 0 {
            self.shock_timer -= 1;
        }
        self.move_state(prev, env, sink);
        self.core_move(env, sink);
        self.ride(env);
        self.body_morph.play();
        let mut centre = self.pos;
        centre.y += REG0_F7 + BODY_LIFT;
        if self.vine != 0 {
            centre.y += 10000.0;
        }
        self.body_sph.centre = centre;
        self.body_sph.radius = REG0_F8 + BODY_RADIUS;
        if self.body_tg != 0 {
            self.body_tg -= 1;
            centre = self.pos;
            centre.y = 0.0;
            add_calc2(
                &mut self.body_tg_radius,
                REG0_F4 + COCYL_RANGE,
                1.0,
                REG0_F5 + 50.0,
            );
        } else {
            self.body_tg_radius = 0.0;
            centre = Vec3::new(OFFSCREEN_X, OFFSCREEN_Y, OFFSCREEN_Z);
        }
        self.co_cyl.centre = centre;
        self.co_cyl.radius = self.body_tg_radius;
        self.co_cyl.height = REG0_F3 + COCYL_HEIGHT;
        self.demo_camera(env, sink);
        self.bmd_kankyo();
        self.eff_cont();
        self.wind();
        self.mk_move(sink);
        if self.camera_on != 0 {
            if self.pos.y > 100.0 {
                self.camera_kind = CAMERA_BOSS;
            } else if self.camera_on == 2 {
                self.camera_kind = CAMERA_FIELD;
                self.camera_on = 1;
            } else {
                self.camera_kind = CAMERA_DUNGEON;
            }
        }
    }

    fn create(&mut self, env: &mut Env<'_>, sink: &mut Sink<'_>) {
        self.status = STTS_SHOWMAP | STTS_UNK4000 | STTS_UNK40000 | STTS_BOSS;
        if env.stage_boss_enemy && !env.stage_x {
            if !env.farore_pearl || REG0_S6 != 0 {
                sink.spawn_actor(
                    MAKAR_ACTOR,
                    0,
                    Vec3::new(100.0, 0.0, 800.0),
                    0,
                    self.room,
                );
            }
            self.blocked = true;
            return;
        }
        self.health = CORE_HEALTH;
        self.max_health = CORE_HEALTH;
        for index in 0..HAND_COUNT {
            sink.spawn_actor(HAND_ACTOR, index as u32, self.home_pos, 0, self.room);
        }
        for index in 0..FOOT_COUNT {
            sink.spawn_actor(FOOT_ACTOR, index as u32, self.home_pos, 0, self.room);
        }
        self.home_pos.y += REG0_F2 + 20.0;
        self.pos.y = self.home_pos.y;
        self.ground = self.pos.y;
        self.timers[2] = 200;
        if !env.stage_boss_demo && !env.stage_x {
            sink.flags.clear(EVENT_TMP_0480);
            self.mode = MODE_START;
            self.env_kind = ENV_FADE;
            self.env_amount = 1.0;
        } else {
            sink.stage_bgm(env.stage_x);
            self.camera_on = 1;
        }
        self.attention_distance = ATTN_BATTLE_DISTANCE;
        self.execute(self.pos, env, sink);
    }
}

const CAMERA_BOSS: u8 = 0;
const CAMERA_FIELD: u8 = 1;
const CAMERA_DUNGEON: u8 = 2;

fn blow_of(watch: &Watch<'_, '_>) -> Blow {
    let mut at = At::Idle;
    if let Some(shaft) = watch
        .shafts
        .iter()
        .find(|shaft| shaft.phase == arrow::Phase::Flying)
    {
        at = match shaft.kind {
            arrow::TYPE_LIGHT => At::LightArrow,
            arrow::TYPE_FIRE => At::FireArrow,
            arrow::TYPE_ICE => At::IceArrow,
            _ => At::Arrow,
        };
    } else if watch
        .bolts
        .iter()
        .any(|bolt| bolt.parameter == boomerang::MODE_THROW)
    {
        at = At::Boomerang;
    } else if watch.hammer.kind == hammer::CUT_HAMMER_SIDESWING
        || watch.hammer.kind == hammer::CUT_HAMMER_FRONTSWING
    {
        at = At::Hammer;
    } else if watch.combat.active && watch.combat.frame >= 1 && watch.combat.frame < 6 {
        at = At::Sword;
    } else if !watch.blasts.list.is_empty() {
        at = At::Bomb;
    }
    Blow {
        at,
        blade: watch.held.item,
        soup: watch.combat.soup,
    }
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    combat: Res<'w, Combat>,
    hammer: Res<'w, hammer::Cut>,
    held: Res<'w, Held>,
    blasts: Res<'w, bombflower::Blasts>,
    equipped: Res<'w, Equipped>,
    collect: Res<'w, Collect>,
    stage: Res<'w, StageName>,
    players: Query<'w, 's, (Entity, &'static mut Player)>,
    shafts: Query<'w, 's, &'static arrow::Arrow>,
    bolts: Query<'w, 's, &'static boomerang::Boomerang>,
}

pub fn control(
    watch: Watch<'_, '_>,
    mut events: ResMut<SimEvents>,
    mut mem: ResMut<MemBits>,
    mut stories: ResMut<StoryBits>,
    mut flags: ResMut<EventFlags>,
    mut health: ResMut<Health>,
    mut info: ResMut<MsgGameInfo>,
    mut next: ResMut<NextStage>,
    mut commands: Commands,
    mut bosses: Query<(Entity, &mut Actor, &mut KalleDemos), (With<KalleDemos>, Without<Player>)>,
) {
    let solid = watch.collision.as_deref();
    let blow = blow_of(&watch);
    let stage_x = watch.stage.0.starts_with('X');
    let farore_pearl =
        inventory::check_get_item(&watch.equipped, &watch.collect, ITEM_PEARL_FARORE);
    let forest_water = (BOTTLE_SLOT_0..BOTTLE_SLOT_0 + BOTTLE_SLOTS)
        .any(|slot| watch.equipped.item(slot) == ITEM_FOREST_WATER);
    let event_run = info.event_run_check();
    let mut players = watch.players;
    let Some(entry) = players.iter_mut().next() else {
        return;
    };
    let mut entry = entry;
    let stage_boss_enemy = mem.is_stage_boss_enemy();
    let stage_boss_demo = mem.is_stage_boss_demo();
    for (_, mut actor, mut boss) in &mut bosses {
        if !boss.entered {
            boss.entered = true;
            boss.pos = actor.pos;
            boss.home_pos = actor.pos;
            boss.angle = actor.angle;
            boss.room = actor.room;
            boss.set_id = actor.set_id;
        }
        let prev = boss.pos;
        let mut env = Env {
            player: &mut entry.1,
            solid,
            blow,
            stage_x,
            stage_boss_enemy,
            stage_boss_demo,
            farore_pearl,
            forest_water,
            event_run,
        };
        let mut sink = Sink {
            events: &mut events,
            mem: &mut mem,
            stories: &mut stories,
            flags: &mut flags,
            health: &mut health,
            info: &mut info,
            spawns: Vec::new(),
            leave: None,
        };
        if !boss.blocked {
            if !boss.created {
                boss.created = true;
                boss.create(&mut env, &mut sink);
            } else {
                boss.execute(prev, &mut env, &mut sink);
            }
        }
        actor.pos = boss.pos;
        actor.angle = boss.angle;
        let spawns = std::mem::take(&mut sink.spawns);
        let leave = sink.leave;
        for spawn in spawns {
            let fresh = Actor {
                name: spawn.name.to_owned(),
                parameters: spawn.parameters,
                pos: spawn.pos,
                angle: spawn.angle,
                set_id: spawn.set_id,
                room: spawn.room,
                ..Default::default()
            };
            if spawn.pickup {
                commands.spawn((fresh, pickup::Pickup::spawn(spawn.parameters)));
            } else {
                commands.spawn(fresh);
            }
        }
        if let Some(room) = leave {
            next.exit = Some(room);
        }
    }
}
