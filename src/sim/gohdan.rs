use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::arrow;
use super::boomerang;
use super::bomb;
use super::bombflower;
use super::collision::Collision;
use super::combat::Combat;
use super::events::{SimEvent, SimEvents};
use super::flags::MemBits;
use super::hammer;
use super::item::{self, Held};
use super::msg::{MsgBox, MsgTable};
use super::pickup;
use super::player::Player;
use super::stream::{add_calc_angle_s2, cos_s, sin_s};
use super::walk::{from_s16, to_s16};

pub const GOHDAN_ACTOR: &str = "Bst";

pub fn is_gohdan(name: &str) -> bool {
    name == GOHDAN_ACTOR
}

const PRM_BYTE_MASK: u32 = 0xFF;
const PRM_PART_MAX: u8 = 2;

const PART_HEAD: u8 = 0;
const PART_LEFT: u8 = 1;
const PART_RIGHT: u8 = 2;

const ACTION_STAY: i16 = 0;
const ACTION_FLY: i16 = 1;
const ACTION_DAMAGE: i16 = 5;
const ACTION_SLEEP: i16 = 6;
const ACTION_HEAD_DAMAGE: i16 = 7;
const ACTION_HEAD_HUKKI: i16 = 8;
const ACTION_DOWN_ATTACK: i16 = 10;
const ACTION_PAA_ATTACK: i16 = 11;
const ACTION_KUMI_ATTACK: i16 = 12;
const ACTION_HARAI_ATTACK: i16 = 13;
const ACTION_BEAM_ATTACK: i16 = 14;
const ACTION_HANA_DEMO: i16 = 20;
const ACTION_END_DEMO: i16 = 22;

const BATTLE_BOSS_BEAM: i8 = 11;
const DEMO_HAND_L: i8 = 1;
const DEMO_STAGE: i8 = 10;
const DEMO_WAKE: i8 = 50;
const DEMO_ITEM: i8 = 57;
const DEMO_BEAM: i8 = 14;
const DEMO_HAND_0: i8 = 12;
const DEMO_HAND_1: i8 = 13;
const DEMO_STAGE_OUT: i8 = 15;

const HIO_M5: u8 = 0;
const HIO_M6: u8 = 0;

const STTS_SHOWMAP: u32 = 0x0000_0020;
const STTS_UNK4000: u32 = 0x0000_4000;
const ATTN_LOCKON_BATTLE: u32 = 0x0000_0004;

const EMODE_NONE: u8 = 0;
const EMODE_LOOP: u8 = 2;
const STATE_STOP: u8 = 1;
const STATE_LOOP: u8 = 2;

const BCK_BEAM: i32 = 0x6;
const BCK_BEAM_MAE: i32 = 0x7;
const BCK_BEAM_TAME: i32 = 0x8;
const BCK_BOM_BAKUHATU: i32 = 0x9;
const BCK_BOM_NOMI: i32 = 0xA;
const BCK_BST_DEAD: i32 = 0xB;
const BCK_BST_FLY: i32 = 0xC;
const BCK_BST_HIRAKU: i32 = 0xD;
const BCK_BST_SLEEP: i32 = 0xE;
const BCK_BST_START: i32 = 0xF;
const BCK_HANA_DEMO: i32 = 0x14;
const BCK_KUTI_OPEN: i32 = 0x15;
const BCK_KUTI_OPEN_LOOP: i32 = 0x16;
const BCK_LH_AWASE: i32 = 0x18;
const BCK_LH_DAMAGE: i32 = 0x19;
const BCK_LH_DOWN: i32 = 0x1A;
const BCK_LH_FLY: i32 = 0x1B;
const BCK_LH_GOO: i32 = 0x1C;
const BCK_LH_PAA: i32 = 0x1D;
const BCK_LH_SLEEP: i32 = 0x1E;
const BCK_LH_START: i32 = 0x1F;
const BCK_LH_START2: i32 = 0x20;
const BCK_RH_AWASU: i32 = 0x21;
const BCK_RH_DAMAGE: i32 = 0x22;
const BCK_RH_DOWN: i32 = 0x23;
const BCK_RH_FLY: i32 = 0x24;
const BCK_RH_GOO: i32 = 0x25;
const BCK_RH_PAA: i32 = 0x26;
const BCK_RH_SLEEP: i32 = 0x27;
const BCK_RH_START: i32 = 0x28;
const BCK_RH_START2: i32 = 0x29;

const BRK_BST: i32 = 0x37;
const BRK_BST_DAMAGE: i32 = 0x38;
const BRK_BST_END: i32 = 0x39;
const BRK_LHAND: i32 = 0x3D;
const BRK_LH_DAMAGE: i32 = 0x3E;
const BRK_LH_END: i32 = 0x40;
const BRK_MUNE: i32 = 0x42;
const BRK_MUNE_END: i32 = 0x43;
const BRK_RHAND: i32 = 0x45;
const BRK_RH_DAMAGE: i32 = 0x46;
const BRK_RH_END: i32 = 0x48;
const BRK_UDEL: i32 = 0x4A;
const BRK_UDEL_END: i32 = 0x4B;
const BRK_UDER: i32 = 0x4D;
const BRK_UDER_END: i32 = 0x4E;

const BTK_BST: i32 = 0x52;
const BTK_BST_DAMAGE: i32 = 0x53;
const BTK_BST_END: i32 = 0x54;
const BTK_LHAND: i32 = 0x57;
const BTK_LH_DAMAGE: i32 = 0x58;
const BTK_LH_END: i32 = 0x5A;
const BTK_MUNE: i32 = 0x5C;
const BTK_MUNE_END: i32 = 0x5D;
const BTK_RHAND: i32 = 0x5F;
const BTK_RH_DAMAGE: i32 = 0x60;
const BTK_RH_END: i32 = 0x62;
const BTK_UDEL: i32 = 0x64;
const BTK_UDEL_END: i32 = 0x65;
const BTK_UDER: i32 = 0x67;
const BTK_UDER_END: i32 = 0x68;

const JNT_HEAD: u16 = 0;
const JNT_BUKI2: u16 = 3;
const JNT_EYEL: u16 = 4;
const JNT_HANAR: u16 = 7;
const JNT_HANAL: u16 = 6;
const JNT_KUTI: u16 = 8;
const JNT_FINGER_FIRST: u16 = 2;
const JNT_FINGER_LAST: u16 = 16;
const JNT_FINGER_KNUCKLE: u16 = 17;

const FRAME_BEAM: i16 = 4;
const FRAME_BEAM_MAE: i16 = 29;
const FRAME_BEAM_TAME: i16 = 4;
const FRAME_BOM_BAKUHATU: i16 = 39;
const FRAME_BOM_NOMI: i16 = 59;
const FRAME_BST_DEAD: i16 = 49;
const FRAME_BST_FLY: i16 = 5;
const FRAME_BST_HIRAKU: i16 = 5;
const FRAME_BST_SLEEP: i16 = 10;
const FRAME_BST_START: i16 = 5;
const FRAME_HANA_DEMO: i16 = 39;
const FRAME_KUTI_OPEN: i16 = 10;
const FRAME_KUTI_OPEN_LOOP: i16 = 5;
const FRAME_HAND: i16 = 10;

const SET_BCK: [i32; 3] = [BCK_BST_SLEEP, BCK_LH_SLEEP, BCK_RH_SLEEP];
const SET_BTK: [i32; 3] = [BCK_BST_SLEEP, BTK_LH_END, BTK_RH_END];
const SET_BRK: [i32; 3] = [BRK_BST, BRK_LH_END, BRK_RH_END];
const START_BCK: [i32; 3] = [BCK_BST_START, BCK_LH_START, BCK_RH_START];
const START2_BCK: [i32; 3] = [0, BCK_LH_START2, BCK_RH_START2];
const FLY_BCK: [i32; 3] = [BCK_BST_FLY, BCK_LH_FLY, BCK_RH_FLY];
const FLY_BTK: [i32; 3] = [BTK_BST, BTK_LHAND, BTK_RHAND];
const FLY_BRK: [i32; 3] = [BRK_BST, BRK_LHAND, BRK_RHAND];
const DAMAGE_BCK: [i32; 3] = [BCK_BST_FLY, BCK_LH_DAMAGE, BCK_RH_DAMAGE];
const DAMAGE_BTK: [i32; 3] = [BTK_BST_DAMAGE, BTK_LH_DAMAGE, BTK_RH_DAMAGE];
const DAMAGE_BRK: [i32; 3] = [BRK_BST_DAMAGE, BRK_LH_DAMAGE, BRK_RH_DAMAGE];
const DOWN_BCK: [i32; 3] = [BCK_BST_FLY, BCK_LH_DOWN, BCK_RH_DOWN];
const DOWNA_BCK: [i32; 3] = [BCK_BST_FLY, BCK_LH_GOO, BCK_RH_GOO];
const PAA_BCK: [i32; 3] = [0, BCK_LH_PAA, BCK_RH_PAA];
const AWASU_BCK: [i32; 3] = [0, BCK_LH_AWASE, BCK_RH_AWASU];

const HEAD_HEALTH: i32 = 3;
const HAND_HEALTH: i32 = 4;
const EYE_HEALTH: i8 = 2;
const EYE_COUNT: usize = 2;
const FINGER_COUNT: usize = 19;
const FINGER_ROWS: usize = 15;
const BEAM_COUNT: usize = 10;
const TIMER_COUNT: usize = 5;
const PHASE_COUNT: usize = 3;
const CHARGE_COUNT: usize = 2;
const BOMB_EFF_COUNT: usize = 5;
const JOINTS: usize = 20;
const ATT_SPL_UNK1: u8 = 1;

const TE_X: [f32; 4] = [70.0, 70.0, 70.0, 70.0];
const TE_Y: [f32; 4] = [70.0, -70.0, 70.0, -70.0];
const TE_Z: [f32; 4] = [90.0, 90.0, -30.0, -30.0];

const HEAD_OFF: [f32; 3] = [-30.0, 0.0, -100.0];
const HEAD_HEIGHT: f32 = 600.0;
const HEAD_RADIUS: f32 = 180.0;
const HAND_OFF: [f32; 3] = [150.0, 0.0, 70.0];
const HAND_HEIGHT_HEAD: f32 = 100.0;
const HAND_RADIUS_HEAD: f32 = 100.0;
const EYE_OFF: [f32; 3] = [250.0, 0.0, 0.0];
const EYE_RADIUS: f32 = 60.0;
const FINGER_RADIUS: f32 = 50.0;
const FINGER_RADIUS_ATTACK: f32 = 50.0;
const KNUCKLE_RADIUS: f32 = 65.0;
const HAND_HEIGHT: f32 = 140.0;
const HAND_RADIUS: f32 = 150.0;
const HAND_SIDE: f32 = 100.0;
const HAND_FAR: f32 = 40.0;
const HAND_UP: f32 = 10.0;

const BEAM_SCALE_UP: f32 = 5.0;
const BEAM_SCALE_STEP: f32 = 0.5;
const BEAM_GROUND_LIFT: f32 = 500.0;
const BEAM_WALL_LIFT: f32 = 50.0;
const BEAM_CEILING: f32 = 2000.0;

const SHAKE_YAW_A: i32 = 0x2100;
const SHAKE_YAW_B: i32 = 0x2300;
const SHAKE_YAW_C: i32 = 0x3600;
const SHAKE_YAW_D: i32 = 0x4300;
const SHAKE_YAW_E: i32 = 0x3A00;
const BOB_X: i32 = 700;
const BOB_Y: i32 = 750;
const BOB_Z: i32 = 720;
const HUKKI_X: i32 = 600;
const CAM_SHAKE_A: i32 = 0x3300;
const CAM_SHAKE_B: i32 = 0x3000;
const CAM_SHAKE_C: i32 = 0x3500;

const DEMO_YAW_OFFSET: i16 = 0xAF0;
const CAM_PITCH: i16 = 4000;
const DEMO_FOV: f32 = 55.0;
const DEMO_LIFT: f32 = 75.0;
const DEMO_STEP: f32 = 1.4;
const DEMO_MSG_A: i16 = 180;
const DEMO_MSG_B: i16 = 200;
const DEMO_MSG_C: i16 = 290;
const DEMO_MSG_D: i16 = 390;
const DEMO_MSG_E: i16 = 410;
const DEMO_MSG_F: i16 = 240;
const DEMO_MSG_G: i16 = 260;
const DEMO_MSG_H: i16 = 350;
const DEMO_MSG_I: i16 = 500;
const DEMO_MSG_J: i16 = 460;
const DEMO_MSG_K: i16 = 200;
const DEMO_MSG_L: i16 = 170;
const DEMO_PLACE_ANGLE: i16 = 0x4000;

const MSG_BEAM_MAE: u32 = 0x170D;
const MSG_BEAM_TAME: u32 = 0x170E;
const MSG_BEAM_END: u32 = 0x170F;
const MSG_DEAD_A: u32 = 0x1710;
const MSG_DEAD_B: u32 = 0x1711;
const MSG_DEAD_C: u32 = 0x1712;
const MSG_DEAD_D: u32 = 0x1713;

const ITEM_ARROW_10: u8 = 0x10;
const ITEM_BOMB_5: u8 = 0x0B;
const ITEM_HEART_CONTAINER: u8 = 0x08;
const ITEM_NO_NONE: i32 = -35;
const ITEM_ACTOR: &str = "item";
const WARP_ACTOR: &str = "Warpf";

const BOMB_REST_TIME: i16 = 200;
const BOMB_MOUTH_LIFT: f32 = 90.0;

const CULL_MIN: [f32; 3] = [-300.0, -150.0, -300.0];
const CULL_MAX: [f32; 3] = [300.0, 600.0, 300.0];

const REG0_F0: f32 = 0.0;
const REG0_F1: f32 = 0.0;
const REG0_F2: f32 = 0.0;
const REG0_F3: f32 = 0.0;
const REG0_F4: f32 = 0.0;
const REG0_F5: f32 = 0.0;
const REG0_F6: f32 = 0.0;
const REG0_F7: f32 = 0.0;
const REG0_F8: f32 = 0.0;
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
const REG0_S0: i16 = 0;
const REG0_S1: i16 = 0;
const REG0_S2: i16 = 0;
const REG0_S3: i16 = 0;
const REG0_S4: i16 = 0;
const REG0_S5: i16 = 0;
const REG0_S6: i16 = 0;
const REG0_S7: i16 = 0;
const REG0_S8: i16 = 0;

const SE_LK_SW_HIT_S: u32 = 0x2803;
const SE_LK_LAST_HIT: u32 = 0x2828;
const SE_LK_W_WEP_HIT: u32 = 0x2833;
const SE_LK_MS_WEP_HIT: u32 = 0x2834;
const SE_LK_HAMMER_HIT: u32 = 0x2855;
const SE_LK_HS_SPIKE: u32 = 0x286F;
const SE_LK_ARROW_HIT: u32 = 0x2879;
const SE_CM_BST_BEAM_PREPARE: u32 = 0x50DF;
const SE_CM_BST_HAND_LIGHT: u32 = 0x58D4;
const SE_CM_BST_HAND_EYE_OPEN: u32 = 0x58D5;
const SE_CM_BST_HEAD_LIGHT: u32 = 0x58D6;
const SE_CM_BST_HEAD_EYE_OPEN: u32 = 0x58D7;
const SE_CM_BST_HAND_GRIP: u32 = 0x58D8;
const SE_CM_BST_HAND_GRIP_OPEN: u32 = 0x58D9;
const SE_CM_BST_HAND_SHAKE: u32 = 0x58DA;
const SE_CM_BST_HAND_RELEASE: u32 = 0x58DB;
const SE_CM_BST_HAND_EYE_CLOSE: u32 = 0x58DC;
const SE_CM_BST_HEAD_EYE_CLOSE: u32 = 0x58DD;
const SE_CM_BST_MOUTH_OPEN: u32 = 0x58DE;
const SE_CM_BST_BEAM_FIRE: u32 = 0x58E0;
const SE_CM_BST_MOUTH_CLOSE: u32 = 0x58E1;
const SE_CM_BST_HAND_STRAIN: u32 = 0x58E2;
const SE_CM_BST_BOMB_EAT: u32 = 0x58E3;
const SE_OBJ_BOMB_EXPLODE: u32 = 0x6901;
const SE_CM_BST_HAND_OUT_WALL: u32 = 0x6983;
const SE_CM_BST_HEAD_OUT_WALL: u32 = 0x6984;
const SE_CM_BST_EYE_HIT: u32 = 0x6985;
const SE_CM_BST_BEAM_BOMB: u32 = 0x6986;
const SE_CM_BST_ITEM_OUT_NOSE: u32 = 0x6987;
const SE_CM_BST_HEAD_FALL: u32 = 0x698A;
const SE_CM_BST_BEF_EXPLODE: u32 = 0x698B;
const SE_CM_BST_EXPLODE: u32 = 0x698C;
const SE_CM_BST_JUMP_S: u32 = 0x698D;
const SE_CM_BST_JUMP_L: u32 = 0x698E;
const SE_CM_BST_HEAD_IN_WALL: u32 = 0x6990;
const SE_CM_BST_1_HAND_ATTACK: u32 = 0x6992;
const SE_CM_BST_2_HAND_ATTACK: u32 = 0x6993;
const SE_CM_BST_HAND_CLAP: u32 = 0x6994;
const SE_CM_BST_EYE_BREAK: u32 = 0x6995;
const SE_CM_BST_HAND_WORKING: u32 = 0x7036;
const SE_CM_BST_HEAD_WORKING: u32 = 0x7038;
const SE_CM_BST_GOKOU: u32 = 0x703A;

const BGM_BST_BATTLE: u32 = 0x8000_0023;
const BGM_UNK_120: u32 = 0x8000_0120;
const BGM_UNK_121: u32 = 0x8000_0121;
const BGM_UNK_122: u32 = 0x8000_0122;
const BGM_UNK_123: u32 = 0x8000_0123;
const STRM_BST_CLEAR: u32 = 0xC000_0019;

const REACH_SWORD: f32 = 20.0;
const REACH_ARROW: f32 = 35.0;
const REACH_BOOMERANG: f32 = 90.0;
const REACH_HAMMER: f32 = 50.0;
const REACH_HOOKSHOT: f32 = 5.0;
const REACH_BOMB: f32 = 150.0;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

fn frame_max(bck: i32) -> i16 {
    match bck {
        BCK_BEAM => FRAME_BEAM,
        BCK_BEAM_MAE => FRAME_BEAM_MAE,
        BCK_BEAM_TAME => FRAME_BEAM_TAME,
        BCK_BOM_BAKUHATU => FRAME_BOM_BAKUHATU,
        BCK_BOM_NOMI => FRAME_BOM_NOMI,
        BCK_BST_DEAD => FRAME_BST_DEAD,
        BCK_BST_FLY => FRAME_BST_FLY,
        BCK_BST_HIRAKU => FRAME_BST_HIRAKU,
        BCK_BST_SLEEP => FRAME_BST_SLEEP,
        BCK_BST_START => FRAME_BST_START,
        BCK_HANA_DEMO => FRAME_HANA_DEMO,
        BCK_KUTI_OPEN => FRAME_KUTI_OPEN,
        BCK_KUTI_OPEN_LOOP => FRAME_KUTI_OPEN_LOOP,
        _ => FRAME_HAND,
    }
}

fn pick(table: &[i32; 3], part: u8) -> i32 {
    table
        .get(usize::from(part))
        .copied()
        .unwrap_or(table[0])
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

fn posed(yaw: i16, pitch: i16, offset: [f32; 3]) -> Vec3 {
    tilted(pitch, turned(yaw, offset))
}

fn bearing(from: Vec3, to: Vec3) -> i16 {
    to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn reach_flat(from: Vec3, to: Vec3) -> f32 {
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    (dx * dx + dz * dz).sqrt()
}

fn span(value: Vec3) -> f32 {
    value.length()
}

fn touches(centre: Vec3, radius: f32, at: Vec3, reach: f32) -> bool {
    (at - centre).length() <= radius + reach
}

fn slide(solid: Option<&Collision>, from: Vec3, to: Vec3) -> (Vec3, bool) {
    match solid {
        Some(solid) => {
            let next = solid.slide(from, to);
            let hit = next.x != to.x || next.z != to.z;
            (next, hit)
        }
        None => (to, false),
    }
}

fn ang(value: f32) -> i16 {
    value as i32 as i16
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
        self.loop_point = ang(frame);
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
            Self::Sword => REACH_SWORD,
            Self::Arrow | Self::FireArrow | Self::IceArrow | Self::LightArrow => REACH_ARROW,
            Self::Boomerang => REACH_BOOMERANG,
            Self::Hammer => REACH_HAMMER,
            Self::Hookshot => REACH_HOOKSHOT,
            Self::Bomb => REACH_BOMB,
            Self::Idle => 0.0,
        }
    }

    fn at_se(self) -> u32 {
        match self {
            Self::Hammer => SE_LK_HAMMER_HIT,
            Self::Arrow | Self::FireArrow | Self::IceArrow | Self::LightArrow => SE_LK_ARROW_HIT,
            Self::Hookshot => SE_LK_HS_SPIKE,
            Self::Boomerang => SE_LK_W_WEP_HIT,
            Self::Sword | Self::Idle => SE_LK_SW_HIT_S,
            Self::Bomb => SE_LK_MS_WEP_HIT,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blow {
    pub at: At,
}

struct BombView {
    entity: Entity,
    pos: Vec3,
    free: bool,
    rest_time: i16,
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
    arrow_num: u8,
    bomb_num: u8,
}

impl Env<'_> {
    fn pos(&self) -> Vec3 {
        self.player.pos
    }

    fn yaw(&self) -> i16 {
        to_s16(self.player.face)
    }

    fn distance(&self, from: Vec3) -> f32 {
        (self.player.pos - from).length()
    }

    fn place(&mut self, pos: Vec3, angle: i16, gate: bool) {
        if !gate {
            return;
        }
        self.player.pos = pos;
        self.player.face = from_s16(angle);
        self.player.yaw = from_s16(angle);
    }
}

struct Sink<'a> {
    events: &'a mut SimEvents,
    msg: &'a mut MsgBox,
    table: &'a MsgTable,
    spawns: &'a mut Vec<Spawn>,
    taken: &'a mut Vec<Entity>,
    eaten: &'a mut Vec<Entity>,
    dragged: &'a mut Option<(Entity, Vec3)>,
    mem: &'a mut MemBits,
}

impl Sink<'_> {
    fn is_stage_boss_enemy(&self) -> bool {
        self.mem.is_stage_boss_enemy()
    }

    fn is_stage_boss_demo(&self) -> bool {
        self.mem.is_stage_boss_demo()
    }

    fn on_stage_boss_enemy(&mut self) {
        self.mem.on_stage_boss_enemy();
    }
}

impl Sink<'_> {
    fn se(&mut self, id: u32) {
        self.events.list.push(SimEvent::Se(id));
    }

    fn bgm(&mut self, id: u32) {
        self.events.list.push(SimEvent::Bgm(id));
    }

    fn stop_bgm(&mut self) {
        self.events.list.push(SimEvent::Bgm(0));
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
}

#[derive(Clone, Copy)]
struct Part {
    entity: Entity,
    state: Gohdan,
}

impl Part {
    fn touch<'a>(
        writes: &'a mut Vec<Part>,
        peers: &[Part],
        kind: u8,
    ) -> Option<&'a mut Gohdan> {
        if let Some(index) = writes.iter().position(|slot| slot.state.part == kind) {
            return writes.get_mut(index).map(|slot| &mut slot.state);
        }
        let source = peers.iter().find(|slot| slot.state.part == kind)?;
        writes.push(*source);
        let index = writes.len() - 1;
        writes.get_mut(index).map(|slot| &mut slot.state)
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Gohdan {
    pub part: u8,
    pub pos: Vec3,
    pub angle: [i16; 3],
    pub speed: Vec3,
    pub speed_f: f32,
    pub eye: Vec3,
    pub home_pos: Vec3,
    pub home_angle: [i16; 3],
    pub shape_angle: [i16; 3],
    pub room: i8,
    pub set_id: u16,
    pub morph: Frame,
    pub anm: i32,
    pub brk_on: i8,
    pub za_on: i8,
    pub tev_key: i32,
    pub tev_frame: f32,
    pub srt_key: i32,
    pub za_brk: i32,
    pub za_btk: i32,
    pub za_speed: f32,
    pub za_loop: bool,
    pub stage_play: f32,
    pub stage_loop: bool,
    pub hidden: bool,
    pub bind: [Vec3; JOINTS],
    pub bob: Vec3,
    pub model_yaw: i16,
    pub sway_clock: i16,
    pub action: i16,
    pub dormant: i16,
    pub damage: i16,
    pub target_pos: Vec3,
    pub player_facing: i16,
    pub offset: Vec3,
    pub sway: f32,
    pub timers: [i16; TIMER_COUNT],
    pub hurt_timer: i16,
    pub state: i16,
    pub wait_timer: i16,
    pub roll: f32,
    pub lean: f32,
    pub twist: f32,
    pub facing: i16,
    pub recoil_speed: f32,
    pub recoil_y: i16,
    pub recoil_x: i16,
    pub shake: i16,
    pub shake_alt: i16,
    pub shake_far: i16,
    pub grip_yaw: i16,
    pub roll_yaw: i16,
    pub eye_health: [i8; EYE_COUNT],
    pub eye_timer: [i16; EYE_COUNT],
    pub eye_hinge: [i16; EYE_COUNT],
    pub battle: i8,
    pub phase_timer: [i16; PHASE_COUNT],
    pub drop_timer: i16,
    pub bomb: Option<Entity>,
    pub bomb_pos: Vec3,
    pub item: Option<Entity>,
    pub item_pos: Vec3,
    pub eye_target: [Option<Entity>; EYE_COUNT],
    pub blure: i16,
    pub demo: i8,
    pub demo_count: i16,
    pub demo_ordered: bool,
    pub nose: i8,
    pub cam_centre: Vec3,
    pub cam_eye: Vec3,
    pub cam_step: f32,
    pub cam_fov: f32,
    pub cam_shake: f32,
    pub kan_amount: f32,
    pub kan_state: i8,
    pub charge: [bool; CHARGE_COUNT],
    pub bomb_eff: [bool; BOMB_EFF_COUNT],
    pub halo: bool,
    pub stage_eff: bool,
    pub cam_test: i8,
    pub hio_set: u8,
    pub hio_5: u8,
    pub hio_6: u8,
    pub msg_live: bool,
    pub msg_end: bool,
    pub health: i32,
    pub max_health: i32,
    pub status: u32,
    pub attention_flags: u32,
    pub attention: Vec3,
    pub wall_radius: f32,
    pub wall_height: f32,
    pub wall_hit: bool,
    pub cc_move: Vec3,
    pub beam_pos: [Vec3; BEAM_COUNT],
    pub beam_dir: [Vec3; BEAM_COUNT],
    pub beam_angle: [[i16; 3]; BEAM_COUNT],
    pub beam_live: [i8; BEAM_COUNT],
    pub beam_scale: [f32; BEAM_COUNT],
    pub head_centre: Vec3,
    pub head_height: f32,
    pub head_radius: f32,
    pub head_at: bool,
    pub hand_centre: Vec3,
    pub hand_height: f32,
    pub hand_radius: f32,
    pub hand_tg: bool,
    pub finger: [Vec3; FINGER_COUNT],
    pub finger_radius: [f32; FINGER_COUNT],
    pub finger_at: [bool; FINGER_COUNT],
    pub finger_spl: [u8; FINGER_COUNT],
    pub finger_co: [bool; FINGER_COUNT],
    pub eye_sph: [Vec3; EYE_COUNT],
    pub eye_radius: [f32; EYE_COUNT],
    pub entered: bool,
    pub seed: [i32; 3],
}

impl Gohdan {
    pub fn spawn(parameters: u32) -> Self {
        let raw = (parameters & PRM_BYTE_MASK) as u8;
        let part = if raw > PRM_PART_MAX { PART_HEAD } else { raw };
        let mut morph = Frame::idle();
        morph.init(frame_max(pick(&SET_BCK, part)));
        morph.attribute = EMODE_LOOP;
        let health = if part == PART_HEAD {
            HEAD_HEALTH
        } else {
            HAND_HEALTH
        };
        let mut seed = [RND_SEED; 3];
        let sway_clock = ang(0x8000 as f32 * (rnd(&mut seed) - 0.5) * 2.0);
        Self {
            part,
            pos: Vec3::ZERO,
            angle: [0; 3],
            speed: Vec3::ZERO,
            speed_f: 0.0,
            eye: Vec3::ZERO,
            home_pos: Vec3::ZERO,
            home_angle: [0; 3],
            shape_angle: [0; 3],
            room: 0,
            set_id: 0,
            morph,
            anm: pick(&SET_BCK, part),
            brk_on: 0,
            za_on: 0,
            tev_key: pick(&SET_BRK, part),
            tev_frame: 0.0,
            srt_key: pick(&SET_BTK, part),
            za_brk: BRK_MUNE,
            za_btk: BTK_MUNE,
            za_speed: 0.0,
            za_loop: false,
            stage_play: 0.0,
            stage_loop: false,
            hidden: false,
            bind: [Vec3::ZERO; JOINTS],
            bob: Vec3::ZERO,
            model_yaw: 0,
            sway_clock,
            action: ACTION_STAY,
            dormant: 0,
            damage: 0,
            target_pos: Vec3::ZERO,
            player_facing: 0,
            offset: Vec3::ZERO,
            sway: 0.0,
            timers: [0; TIMER_COUNT],
            hurt_timer: 0,
            state: 0,
            wait_timer: 0,
            roll: 0.0,
            lean: 0.0,
            twist: 0.0,
            facing: 0,
            recoil_speed: 0.0,
            recoil_y: 0,
            recoil_x: 0,
            shake: 0,
            shake_alt: 0,
            shake_far: 0,
            grip_yaw: 0,
            roll_yaw: 0,
            eye_health: if part == PART_HEAD {
                [EYE_HEALTH, EYE_HEALTH]
            } else {
                [0, 0]
            },
            eye_timer: [0; EYE_COUNT],
            eye_hinge: [0; EYE_COUNT],
            battle: 0,
            phase_timer: [0; PHASE_COUNT],
            drop_timer: 0,
            bomb: None,
            bomb_pos: Vec3::ZERO,
            item: None,
            item_pos: Vec3::ZERO,
            eye_target: [None; EYE_COUNT],
            blure: 0,
            demo: 0,
            demo_count: 0,
            demo_ordered: false,
            nose: 0,
            cam_centre: Vec3::ZERO,
            cam_eye: Vec3::ZERO,
            cam_step: 0.0,
            cam_fov: DEMO_FOV,
            cam_shake: 0.0,
            kan_amount: 0.0,
            kan_state: 0,
            charge: [false; CHARGE_COUNT],
            bomb_eff: [false; BOMB_EFF_COUNT],
            halo: false,
            stage_eff: false,
            cam_test: 0,
            hio_set: 0,
            hio_5: HIO_M5,
            hio_6: HIO_M6,
            msg_live: false,
            msg_end: false,
            health,
            max_health: health,
            status: STTS_UNK4000,
            attention_flags: ATTN_LOCKON_BATTLE,
            attention: Vec3::ZERO,
            wall_radius: 200.0,
            wall_height: 10.0,
            wall_hit: false,
            cc_move: Vec3::ZERO,
            beam_pos: [Vec3::ZERO; BEAM_COUNT],
            beam_dir: [Vec3::ZERO; BEAM_COUNT],
            beam_angle: [[0; 3]; BEAM_COUNT],
            beam_live: [0; BEAM_COUNT],
            beam_scale: [0.0; BEAM_COUNT],
            head_centre: Vec3::ZERO,
            head_height: HEAD_HEIGHT,
            head_radius: HEAD_RADIUS,
            head_at: false,
            hand_centre: Vec3::ZERO,
            hand_height: HAND_HEIGHT_HEAD,
            hand_radius: HAND_RADIUS_HEAD,
            hand_tg: false,
            finger: [Vec3::ZERO; FINGER_COUNT],
            finger_radius: [FINGER_RADIUS; FINGER_COUNT],
            finger_at: [false; FINGER_COUNT],
            finger_spl: [ATT_SPL_UNK1; FINGER_COUNT],
            finger_co: [false; FINGER_COUNT],
            eye_sph: [Vec3::ZERO; EYE_COUNT],
            eye_radius: [EYE_RADIUS; EYE_COUNT],
            entered: false,
            seed,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        !self.hidden
    }

    fn rnd_f(&mut self, range: f32) -> f32 {
        rnd(&mut self.seed) * range
    }

    fn rnd_fx(&mut self, range: f32) -> f32 {
        range * (rnd(&mut self.seed) - 0.5) * 2.0
    }

    fn rnd_ang(&mut self, range: f32, base: f32) -> i16 {
        ang(rnd(&mut self.seed) * range + base)
    }

    fn peer(&self, kind: u8, peers: &[Part]) -> Gohdan {
        peers
            .iter()
            .find(|slot| slot.state.part == kind)
            .map(|slot| slot.state)
            .unwrap_or_else(|| Gohdan::spawn(u32::from(kind)))
    }

    fn head(&self, peers: &[Part]) -> Gohdan {
        if self.part == PART_HEAD {
            *self
        } else {
            self.peer(PART_HEAD, peers)
        }
    }

    fn other_hand(&self) -> u8 {
        3 - self.part
    }

    fn joint(&self, no: u16) -> Vec3 {
        let bind = self
            .bind
            .get(usize::from(no))
            .copied()
            .unwrap_or(Vec3::ZERO);
        self.pos + self.bob + turned(self.model_yaw, bind.to_array())
    }

    fn joint_off(&self, no: u16, offset: [f32; 3]) -> Vec3 {
        self.joint(no) + turned(self.model_yaw, offset)
    }

    fn anm_init(&mut self, bck: i32, morf: f32, loop_mode: u8, speed: f32) {
        self.anm = bck;
        self.morph.init(frame_max(bck));
        self.morph.attribute = loop_mode;
        self.morph.rate = speed;
        self.morph.set_loop(0.0);
        self.morph.set_morf(morf);
    }

    fn tev_init(&mut self, key: i32) {
        self.tev_key = key;
        self.tev_frame = 0.0;
    }

    fn srt_init(&mut self, key: i32) {
        self.srt_key = key;
    }

    fn message_set(&mut self, msg_no: u32, sink: &mut Sink<'_>) -> bool {
        if !self.msg_live {
            sink.msg.message_set(msg_no, sink.table);
            self.msg_live = true;
            self.msg_end = false;
            return true;
        }
        false
    }

    fn message_cont(&mut self, sink: &mut Sink<'_>) {
        if !self.msg_live {
            return;
        }
        let status = sink.msg.status();
        if status == super::msg::MSG_DISPLAYED {
            if self.msg_end {
                sink.msg.set_status(super::msg::MSG_ENDS);
            }
        } else if status == super::msg::MSG_BOX_CLOSED {
            sink.msg.set_status(super::msg::MSG_DESTROYED);
            self.msg_live = false;
        }
    }

    fn set_hand_at(&mut self, on: bool) {
        for index in 0..FINGER_COUNT {
            self.finger_at[index] = on;
            self.finger_spl[index] = ATT_SPL_UNK1;
        }
    }

    fn set_hand_co(&mut self, on: bool) {
        for index in 0..FINGER_COUNT {
            self.finger_co[index] = on;
        }
    }

    fn pos_move(&mut self, turn: bool) {
        if turn {
            let vec = self.target_pos - self.pos;
            let y_angle = bearing(Vec3::ZERO, vec);
            let dist = reach_flat(Vec3::ZERO, vec);
            let unk_angle = -to_s16(vec.y.atan2(dist));
            let cap = ang(self.offset.x * self.offset.y);
            add_calc_angle_s2(&mut self.angle[1], y_angle, REG0_S3 + 5, cap);
            add_calc_angle_s2(&mut self.angle[0], unk_angle, REG0_S3 + 5, cap);
        }
        add_calc2(&mut self.offset.y, 1.0, 1.0, 0.05);
        let step = posed(self.angle[1], self.angle[0], [0.0, 0.0, self.speed_f]);
        self.speed = step;
        self.pos += step;
    }

    fn stay(&mut self, peers: &[Part], writes: &mut Vec<Part>) {
        self.status = 0;
        self.attention_flags = 0;
        self.state = 10;
        let part = self.part;
        let mut arm = self.damage;
        loop {
            match arm {
                0 => {
                    self.anm_init(pick(&SET_BCK, part), 2.0, EMODE_LOOP, 1.0);
                    self.damage = self.damage.wrapping_add(1);
                    break;
                }
                2 => {
                    self.anm_init(pick(&START_BCK, part), 2.0, EMODE_NONE, 1.0);
                    self.damage = self.damage.wrapping_add(1);
                    break;
                }
                4 => {
                    let mut vec = Vec3::ZERO;
                    if part != PART_HEAD {
                        self.anm_init(pick(&START2_BCK, part), 10.0, EMODE_NONE, 1.0);
                        vec.z = REG0_F11 + 400.0;
                    } else {
                        vec.z = 10000.0;
                    }
                    if self.part == PART_HEAD {
                        self.cam_shake = REG0_F10 + 15.0;
                    } else if let Some(boss) = Part::touch(writes, peers, PART_HEAD) {
                        boss.cam_shake = REG0_F10 + 15.0;
                    }
                    self.damage = self.damage.wrapping_add(1);
                    self.target_pos = self.pos + posed(self.angle[1], 0, vec.to_array());
                    self.timers[3] = 0xF;
                    arm = 5;
                }
                5 => {
                    if part != PART_HEAD && self.timers[3] != 0 {
                        for no in JNT_FINGER_FIRST..=JNT_FINGER_LAST {
                            let _ = self.joint(no);
                        }
                    }
                    add_calc2(&mut self.pos.x, self.target_pos.x, 0.05, 10.0);
                    add_calc2(&mut self.pos.z, self.target_pos.z, 0.05, 10.0);
                    break;
                }
                _ => break,
            }
        }
    }

    fn fly(&mut self, env: &Env<'_>, peers: &[Part]) {
        let boss = self.head(peers);
        if self.part != PART_HEAD && self.dormant == 6 {
            self.health = HAND_HEALTH;
        }
        self.dormant = 1;
        if self.state == 0 {
            self.set_hand_co(true);
        }
        if boss.demo != 0 || (self.sway_clock & 0x1F) == 0 && self.rnd_f(1.0) < 0.5 {
            self.player_facing = env.yaw();
        }
        let part = self.part;
        let mut arm = self.damage;
        loop {
            match arm {
                0 => {
                    self.anm_init(pick(&FLY_BCK, part), 10.0, EMODE_LOOP, 1.0);
                    self.tev_init(pick(&DAMAGE_BRK, part));
                    self.tev_frame = f32::from(REG0_S8) + 29.0;
                    self.brk_on = 1;
                    self.za_on = 1;
                    self.damage = self.damage.wrapping_add(1);
                    break;
                }
                1 => {
                    self.tev_init(pick(&FLY_BRK, part));
                    self.srt_init(pick(&FLY_BTK, part));
                    self.damage = self.damage.wrapping_add(1);
                    self.offset.y = 0.0;
                    arm = 2;
                }
                2 => {
                    let mut vec = Vec3::new(0.0, REG0_F3 + 200.0, REG0_F4 + 800.0);
                    if part == PART_HEAD {
                        vec.x = 0.0;
                        vec.z = REG0_F4 + 1000.0;
                        if boss.battle == BATTLE_BOSS_BEAM {
                            vec.y = REG0_F3 + 350.0;
                        }
                    } else if part == PART_LEFT {
                        vec.x = -(REG0_F2 + 800.0);
                    } else if part == PART_RIGHT {
                        vec.x = REG0_F2 + 800.0;
                    }
                    self.target_pos = env.pos() + turned(self.player_facing, vec.to_array());
                    if part == PART_HEAD {
                        let mut flat = self.target_pos;
                        flat.y = 0.0;
                        let dist = span(flat);
                        if dist > REG0_F9 + 1350.0 {
                            let tmp = (REG0_F9 + 1350.0) / dist;
                            self.target_pos.x *= tmp;
                            self.target_pos.z *= tmp;
                        }
                    }
                    if span(self.target_pos - self.pos) > REG0_F12 + 300.0 {
                        let speed_target = if part == PART_HEAD && boss.battle == BATTLE_BOSS_BEAM {
                            10.0
                        } else {
                            30.0
                        };
                        add_calc2(&mut self.speed_f, speed_target, 1.0, REG0_F13 + 2.0);
                    } else {
                        add_calc0(&mut self.speed_f, 1.0, REG0_F14 + 1.0);
                    }
                    self.offset.x = REG0_F5 + 1500.0;
                    add_calc_angle_s2(
                        &mut self.shape_angle[1],
                        bearing(self.pos, env.pos()),
                        10,
                        0x400,
                    );
                    let target_x = if part == PART_HEAD && boss.battle == BATTLE_BOSS_BEAM {
                        0
                    } else {
                        3000
                    };
                    add_calc_angle_s2(&mut self.shape_angle[0], target_x, 10, 256);
                    add_calc_angle_s2(&mut self.shape_angle[2], 0, 8, 512);
                    add_calc2(&mut self.offset.z, REG0_F11 + 50.0, 1.0, 2.0);
                    break;
                }
                _ => break,
            }
        }
        self.pos_move(false);
        if part == PART_HEAD && self.drop_timer == 0 && (env.arrow_num == 0 || env.bomb_num == 0) {
            self.drop_timer = 600;
            self.action = ACTION_HANA_DEMO;
            self.damage = 0;
        }
    }

    fn down_attack(
        &mut self,
        env: &Env<'_>,
        peers: &[Part],
        writes: &mut Vec<Part>,
        sink: &mut Sink<'_>,
    ) {
        let mut b_var2 = true;
        let mut b_var4 = false;
        add_calc_angle_s2(&mut self.shape_angle[0], 0, 10, 0x200);
        let part = self.part;
        let mut arm = self.damage;
        loop {
            match arm {
                0 => {
                    self.damage = self.damage.wrapping_add(1);
                    self.offset.y = 0.0;
                    self.target_pos = env.pos();
                    self.target_pos.y += REG0_F7 + 300.0;
                    self.timers[0] = 100;
                    arm = 1;
                }
                1 => {
                    add_calc2(&mut self.speed_f, 40.0, 1.0, REG0_F13 + 1.0);
                    self.offset.x = REG0_F5 + 2000.0;
                    let target_diff = self.target_pos - self.pos;
                    if span(target_diff) < REG0_F12 + 200.0 || self.timers[0] == 0 {
                        self.damage = self.damage.wrapping_add(1);
                    }
                    add_calc_angle_s2(&mut self.shape_angle[1], self.angle[1], 4, 0x400);
                    break;
                }
                2 => {
                    b_var2 = false;
                    add_calc2(&mut self.pos.x, self.target_pos.x, 0.5, self.speed.x.abs());
                    add_calc2(&mut self.pos.y, self.target_pos.y, 0.5, self.speed.y.abs());
                    add_calc2(&mut self.pos.z, self.target_pos.z, 0.5, self.speed.z.abs());
                    add_calc_angle_s2(&mut self.shape_angle[1], self.angle[1], 4, 0x4000);
                    let target_diff = self.target_pos - self.pos;
                    if span(target_diff) < REG0_F13 + 50.0 {
                        self.anm_init(pick(&DOWNA_BCK, part), 10.0, EMODE_LOOP, 1.0);
                        self.damage = self.damage.wrapping_add(1);
                        self.speed.y = 0.0;
                        self.timers[0] = 30;
                        self.grip_yaw = if REG0_S0 == 1 {
                            self.shape_angle[1].wrapping_add(0x4000)
                        } else {
                            self.shape_angle[1].wrapping_sub(0x4000)
                        };
                        self.roll_yaw = if part == PART_RIGHT {
                            0x4000
                        } else {
                            -0x4000
                        };
                        sink.se(SE_CM_BST_HAND_GRIP);
                    }
                    break;
                }
                3 => {
                    b_var2 = false;
                    add_calc2(&mut self.lean, -200.0, 1.0, 25.0);
                    add_calc2(&mut self.pos.x, self.target_pos.x, 0.5, self.speed.x.abs());
                    add_calc2(&mut self.pos.z, self.target_pos.z, 0.5, self.speed.z.abs());
                    add_calc_angle_s2(&mut self.shape_angle[2], self.roll_yaw, 4, 0x1000);
                    add_calc_angle_s2(&mut self.shape_angle[1], self.grip_yaw, 4, 0x1000);
                    self.pos.y += self.speed.y;
                    if self.timers[0] == 0 {
                        self.set_hand_at(true);
                        self.speed.y -= REG0_F3 + 60.0;
                        if self.pos.y <= REG0_F4 + 150.0 {
                            self.pos.y = REG0_F4 + 150.0;
                            self.speed.y = 0.0;
                            self.shake_alt = REG0_S7 + 10;
                            sink.se(SE_CM_BST_1_HAND_ATTACK);
                            self.damage = self.damage.wrapping_add(1);
                            self.timers[0] = 30;
                        }
                    } else {
                        self.speed.y += REG0_F14 + 0.5;
                    }
                    break;
                }
                4 => {
                    b_var2 = false;
                    add_calc2(&mut self.lean, -200.0, 1.0, 25.0);
                    if self.timers[0] == 0 {
                        b_var4 = true;
                    }
                    break;
                }
                _ => break,
            }
        }
        if b_var2 {
            self.pos_move(false);
        }
        if b_var4 || self.wall_hit {
            self.action = ACTION_FLY;
            self.damage = 0;
            sink.se(SE_CM_BST_HAND_GRIP_OPEN);
            if !b_var4 {
                if let Some(other) = Part::touch(writes, peers, self.other_hand()) {
                    if other.action == ACTION_DOWN_ATTACK {
                        other.action = ACTION_FLY;
                        other.damage = 0;
                        other.speed_f = 0.0;
                    }
                }
            }
        }
    }

    fn paa_attack(&mut self, env: &Env<'_>, peers: &[Part], writes: &mut Vec<Part>, sink: &mut Sink<'_>) {
        let mut r27 = true;
        let mut r26 = false;
        add_calc_angle_s2(&mut self.shape_angle[0], 0, 10, 0x200);
        add_calc_angle_s2(&mut self.shape_angle[2], 0, 10, 0x400);
        add_calc2(&mut self.twist, -40.0, 1.0, 10.0);
        let part = self.part;
        let mut arm = self.damage;
        loop {
            match arm {
                0 => {
                    self.damage = self.damage.wrapping_add(1);
                    self.offset.y = 0.0;
                    self.facing = env.yaw();
                    self.anm_init(pick(&PAA_BCK, part), 10.0, EMODE_LOOP, 1.0);
                    self.timers[0] = REG0_S1 + 80;
                    arm = 1;
                }
                1 => {
                    let local_38 = self.target_pos - self.pos;
                    if span(local_38) > REG0_F12 + 200.0 {
                        add_calc2(&mut self.speed_f, 40.0, 1.0, REG0_F13 + 2.0);
                    } else {
                        add_calc0(&mut self.speed_f, 1.0, REG0_F13 + 2.0);
                    }
                    self.offset.x = REG0_F5 + 8000.0;
                    let mut local_38 = Vec3::ZERO;
                    let s_var6;
                    if part == PART_LEFT {
                        local_38.x = -(REG0_F2 + 700.0);
                        s_var6 = 0x4000;
                    } else {
                        local_38.x = REG0_F2 + 700.0;
                        s_var6 = -0x4000;
                    }
                    local_38.y = REG0_F3;
                    local_38.z = REG0_F4;
                    self.target_pos = env.pos() + turned(self.facing, local_38.to_array());
                    add_calc_angle_s2(
                        &mut self.shape_angle[1],
                        self.facing.wrapping_add(s_var6),
                        10,
                        0x800,
                    );
                    if self.timers[0] == 0 {
                        self.damage = self.damage.wrapping_add(1);
                        self.target_pos = env.pos();
                        self.angle[1] = bearing(self.pos, env.pos());
                    }
                    break;
                }
                2 => {
                    add_calc2(&mut self.speed_f, REG0_F14 + 140.0, 1.0, REG0_F15 + 10.0);
                    self.offset.x = REG0_F5 + 10000.0;
                    let local_38 = self.target_pos - self.pos;
                    if span(local_38) < REG0_F16 + 200.0 {
                        self.damage = self.damage.wrapping_add(1);
                        self.timers[0] = REG0_S7 + 0x1e;
                    }
                    break;
                }
                3 => {
                    r27 = false;
                    if self.timers[0] >= 0x1C {
                        if self.timers[0] == 0x1c {
                            if part == PART_LEFT {
                                sink.se(SE_CM_BST_HAND_CLAP);
                            }
                            self.shake_alt = REG0_S8 + 5;
                        }
                        self.set_hand_at(true);
                    }
                    let s_var6 = if part == PART_LEFT {
                        0x4000
                    } else {
                        -0x4000
                    };
                    add_calc_angle_s2(
                        &mut self.shape_angle[1],
                        self.facing.wrapping_add(s_var6),
                        1,
                        0x1000,
                    );
                    add_calc2(&mut self.pos.x, self.target_pos.x, 1.0, self.speed.x.abs());
                    add_calc2(&mut self.pos.z, self.target_pos.z, 1.0, self.speed.z.abs());
                    if self.timers[0] == 0 {
                        self.damage = self.damage.wrapping_add(1);
                        self.speed_f = 0.0;
                        self.timers[0] = REG0_S3 + 0x1e;
                        self.angle[1] = self.angle[1].wrapping_add(0x8000_u16 as i16);
                        let mut local_38 = Vec3::ZERO;
                        local_38.x = if part == PART_LEFT {
                            -(REG0_F2 + 4000.0)
                        } else {
                            REG0_F2 + 4000.0
                        };
                        local_38.y = REG0_F3;
                        local_38.z = REG0_F4;
                        self.target_pos += turned(self.facing, local_38.to_array());
                    }
                    break;
                }
                4 => {
                    add_calc2(&mut self.speed_f, 40.0, 1.0, REG0_F13 + 1.0);
                    self.offset.x = REG0_F5 + 2000.0;
                    if self.timers[0] == 0 {
                        r26 = true;
                    }
                    break;
                }
                _ => break,
            }
        }
        if r27 {
            self.pos_move(false);
        }
        if self.pos.y <= REG0_F4 + 30.0 {
            self.pos.y = REG0_F4 + 30.0;
        }
        if r26 || self.wall_hit {
            self.action = ACTION_FLY;
            self.damage = 0;
            self.wait_timer = self.rnd_ang(100.0, 100.0);
            if !r26 {
                let wait = self.rnd_ang(100.0, 100.0);
                if let Some(other) = Part::touch(writes, peers, self.other_hand()) {
                    if other.action == ACTION_PAA_ATTACK {
                        other.action = ACTION_FLY;
                        other.damage = 0;
                        other.wait_timer = wait;
                    }
                }
            }
        }
    }

    fn kumi_attack(
        &mut self,
        env: &Env<'_>,
        peers: &[Part],
        writes: &mut Vec<Part>,
        sink: &mut Sink<'_>,
    ) {
        let mut r27 = true;
        let mut r26 = false;
        add_calc_angle_s2(&mut self.shape_angle[0], 0, 10, 0x200);
        add_calc2(&mut self.roll, 30.0, 1.0, 10.0);
        add_calc2(&mut self.twist, -80.0, 1.0, 10.0);
        add_calc2(&mut self.lean, -200.0, 1.0, 30.0);
        let part = self.part;
        let mut arm = self.damage;
        loop {
            match arm {
                0 => {
                    self.damage = self.damage.wrapping_add(1);
                    self.offset.y = 0.0;
                    self.facing = env.yaw();
                    self.anm_init(pick(&AWASU_BCK, part), 10.0, EMODE_LOOP, 1.0);
                    self.timers[0] = REG0_S1 + 0x46;
                    arm = 1;
                }
                1 => {
                    let local_38 = self.target_pos - self.pos;
                    if span(local_38) > REG0_F12 + 200.0 {
                        add_calc2(&mut self.speed_f, 40.0, 1.0, REG0_F13 + 2.0);
                    } else {
                        add_calc0(&mut self.speed_f, 1.0, REG0_F13 + 2.0);
                    }
                    self.offset.x = REG0_F5 + 8000.0;
                    let mut local_38 = Vec3::ZERO;
                    let s_var5;
                    if part == PART_LEFT {
                        local_38.x = -(REG0_F2 + 500.0);
                        s_var5 = 0x4000;
                    } else {
                        local_38.x = REG0_F2 + 500.0;
                        s_var5 = -0x4000;
                    }
                    local_38.y = REG0_F3 + 400.0;
                    local_38.z = 0.0;
                    self.target_pos = env.pos() + turned(self.facing, local_38.to_array());
                    add_calc_angle_s2(
                        &mut self.shape_angle[1],
                        self.facing.wrapping_add(s_var5),
                        10,
                        0x800,
                    );
                    if self.timers[0] == 0 {
                        self.damage = self.damage.wrapping_add(1);
                        self.target_pos = env.pos();
                        self.target_pos.y += REG0_F3 + 500.0;
                        self.angle[1] = bearing(self.pos, env.pos());
                    }
                    break;
                }
                2 => {
                    add_calc2(&mut self.speed_f, REG0_F14 + 100.0, 1.0, REG0_F15 + 8.0);
                    self.offset.x = REG0_F5 + 10000.0;
                    let local_38 = self.target_pos - self.pos;
                    if span(local_38) < REG0_F16 + 100.0 {
                        self.damage = self.damage.wrapping_add(1);
                    }
                    break;
                }
                3 => {
                    r27 = false;
                    let s_var5 = if part == PART_LEFT {
                        0x4000
                    } else {
                        -0x4000
                    };
                    add_calc_angle_s2(
                        &mut self.shape_angle[1],
                        self.facing.wrapping_add(s_var5),
                        1,
                        0x1000,
                    );
                    add_calc2(&mut self.pos.x, self.target_pos.x, 1.0, self.speed.x.abs());
                    add_calc2(&mut self.pos.y, self.target_pos.y, 1.0, self.speed.y.abs() + 10.0);
                    add_calc2(&mut self.pos.z, self.target_pos.z, 1.0, self.speed.z.abs());
                    let left = if part == PART_LEFT {
                        self.pos
                    } else {
                        self.peer(PART_LEFT, peers).pos
                    };
                    let right = if part == PART_RIGHT {
                        self.pos
                    } else {
                        self.peer(PART_RIGHT, peers).pos
                    };
                    let local_38 = left - right;
                    if span(local_38) < 1.0 {
                        self.damage = self.damage.wrapping_add(1);
                        self.timers[0] = REG0_S3 + 10;
                        sink.se(SE_CM_BST_HAND_SHAKE);
                        self.speed.y = 0.0;
                    }
                    break;
                }
                4 => {
                    r27 = false;
                    self.pos.y += self.speed.y;
                    if self.timers[0] <= 1 {
                        self.roll_yaw = if part == PART_RIGHT {
                            0x4000
                        } else {
                            -0x4000
                        };
                        add_calc_angle_s2(&mut self.shape_angle[2], self.roll_yaw, 1, 0x1000);
                        self.set_hand_at(true);
                        self.speed.y -= REG0_F3 + 60.0;
                        if self.pos.y <= REG0_F4 + 150.0 {
                            self.pos.y = REG0_F4 + 150.0;
                            self.speed.y = 0.0;
                            if part == PART_LEFT {
                                sink.se(SE_CM_BST_2_HAND_ATTACK);
                            }
                            self.shake_alt = REG0_S7 + 10;
                            self.damage = self.damage.wrapping_add(1);
                            self.timers[0] = 0x28;
                        }
                    } else {
                        self.speed.y += REG0_F14 + 1.0;
                        self.roll_yaw = if part == PART_RIGHT {
                            -0x1000
                        } else {
                            0x1000
                        };
                        add_calc_angle_s2(&mut self.shape_angle[2], self.roll_yaw, 10, 0x200);
                    }
                    break;
                }
                5 => {
                    r27 = false;
                    if self.timers[0] == 0 {
                        r26 = true;
                    }
                    break;
                }
                _ => break,
            }
        }
        if r27 {
            self.pos_move(false);
        }
        if r26 || self.wall_hit {
            self.action = ACTION_FLY;
            self.damage = 0;
            self.speed_f = 0.0;
            self.angle[1] = self.angle[1].wrapping_add(0x8000_u16 as i16);
            self.wait_timer = self.rnd_ang(100.0, 100.0);
            self.state = 0x28;
            sink.se(SE_CM_BST_HAND_RELEASE);
            if !r26 {
                let wait = self.rnd_ang(100.0, 100.0);
                if let Some(slot) = Part::touch(writes, peers, self.other_hand()) {
                    if slot.action == ACTION_KUMI_ATTACK {
                        slot.action = ACTION_FLY;
                        slot.damage = 0;
                        slot.speed_f = 0.0;
                        slot.angle[1] = slot.angle[1].wrapping_add(0x8000_u16 as i16);
                        slot.wait_timer = wait;
                        slot.state = 0x28;
                        sink.se(SE_CM_BST_HAND_RELEASE);
                    }
                }
            }
        }
    }

    fn harai_attack(&mut self, env: &Env<'_>) {
        let mut c_var3 = 0;
        self.set_hand_co(true);
        add_calc_angle_s2(&mut self.shape_angle[0], 0, 10, 0x200);
        add_calc_angle_s2(&mut self.shape_angle[1], self.angle[1], 4, 0x1000);
        let d_var4;
        if self.part == PART_RIGHT {
            self.roll_yaw = 0x4000;
            d_var4 = -(REG0_F11 + 50.0);
        } else {
            self.roll_yaw = -0x4000;
            d_var4 = REG0_F11 + 50.0;
        }
        add_calc_angle_s2(&mut self.shape_angle[2], self.roll_yaw, 8, 0x400);
        add_calc2(&mut self.roll, d_var4, 1.0, 10.0);
        let part = self.part;
        let mut arm = self.damage;
        loop {
            match arm {
                0 => {
                    self.damage = self.damage.wrapping_add(1);
                    self.offset.y = 0.0;
                    self.speed_f = 0.0;
                    self.timers[0] = REG0_S5 + 20;
                    self.angle[1] = bearing(self.pos, env.pos());
                    self.anm_init(pick(&FLY_BCK, part), 10.0, EMODE_LOOP, 1.0);
                    arm = 1;
                }
                1 => {
                    c_var3 = 1;
                    add_calc2(&mut self.speed_f, 60.0, 1.0, REG0_F13 + 2.0);
                    self.offset.x = REG0_F5 + 2000.0;
                    if self.timers[0] == 0 {
                        self.damage = self.damage.wrapping_add(1);
                        self.timers[0] = REG0_S6 + 60;
                    }
                    break;
                }
                2 => {
                    c_var3 = 1;
                    let s_var2 = REG0_S4 + 350;
                    if part == PART_RIGHT {
                        self.angle[1] = self.angle[1].wrapping_add(s_var2);
                    } else {
                        self.angle[1] = self.angle[1].wrapping_sub(s_var2);
                    }
                    let mut local_34 = self.pos;
                    local_34.y = 0.0;
                    if self.timers[0] == 0 || span(local_34) > 1700.0 {
                        self.timers[0] = 30;
                        self.damage = self.damage.wrapping_add(1);
                    }
                    break;
                }
                3 => {
                    c_var3 = 1;
                    add_calc0(&mut self.speed_f, 1.0, REG0_F13 + 3.0);
                    if self.timers[0] == 0 {
                        self.action = ACTION_FLY;
                        self.damage = 0;
                        self.wait_timer = self.rnd_ang(60.0, 60.0);
                    }
                    break;
                }
                _ => break,
            }
        }
        self.angle[0] = 0;
        add_calc2(&mut self.pos.y, REG0_F7 + 180.0, 0.2, 10.0);
        self.pos_move(c_var3 != 0);
    }

    fn sleep(&mut self, sink: &mut Sink<'_>) {
        self.dormant = 6;
        self.set_hand_co(true);
        self.state = 10;
        self.status = 0;
        self.attention_flags = 0;
        add_calc2(&mut self.offset.z, REG0_F11 + 50.0, 1.0, 2.0);
        let part = self.part;
        let mut arm = self.damage;
        loop {
            match arm {
                0 => {
                    self.anm_init(pick(&DOWN_BCK, part), 20.0, EMODE_LOOP, 1.0);
                    self.tev_init(pick(&SET_BRK, part));
                    self.srt_init(pick(&SET_BTK, part));
                    self.damage = self.damage.wrapping_add(1);
                    sink.se(SE_CM_BST_HAND_EYE_CLOSE);
                    arm = 1;
                }
                1 => {
                    if self.timers[0] == 0 {
                        self.target_pos.x = self.rnd_fx(1500.0);
                        self.target_pos.y = self.rnd_f(200.0) + 600.0;
                        if part == PART_LEFT {
                            self.target_pos.y += 350.0;
                        }
                        self.target_pos.z = self.rnd_fx(1500.0);
                        self.timers[0] = self.rnd_ang(30.0, 0.0);
                    }
                    self.offset.x = REG0_F5 + 500.0;
                    self.speed_f = REG0_F6 + 10.0;
                    if self.timers[1] == 0 {
                        self.action = ACTION_FLY;
                        self.damage = 0;
                        sink.se(SE_CM_BST_HAND_EYE_OPEN);
                    }
                    break;
                }
                _ => break,
            }
        }
        self.pos_move(false);
    }

    fn beam_set(&mut self) {
        for index in 0..BEAM_COUNT {
            if self.beam_live[index] != 0 {
                continue;
            }
            self.beam_live[index] = 1;
            self.beam_pos[index] = self.joint(JNT_BUKI2);
            self.beam_angle[index] = self.shape_angle;
            self.beam_dir[index] = turned(self.shape_angle[1], [0.0, 0.0, REG0_F12 + 160.0]);
            self.beam_scale[index] = 1.0;
            return;
        }
    }

    fn beam_attack(&mut self, env: &Env<'_>, sink: &mut Sink<'_>) {
        let mut b_var3 = false;
        self.eye_timer[0] = 10;
        self.eye_timer[1] = 10;
        if self.drop_timer < 120 {
            self.drop_timer = 120;
        }
        let mut actor_vec = env.pos() - self.pos;
        actor_vec.y += self.sway;
        let angle = to_s16(actor_vec.x.atan2(actor_vec.z));
        add_calc_angle_s2(&mut self.shape_angle[1], angle, 4, 0x800);
        let flat = reach_flat(Vec3::ZERO, actor_vec);
        add_calc_angle_s2(
            &mut self.shape_angle[0],
            -to_s16(actor_vec.y.atan2(flat)),
            4,
            0x800,
        );
        add_calc2(&mut self.pos.y, REG0_F11 + 400.0, 0.05, 30.0);
        let arm = self.damage;
        loop {
            match arm {
                0 => {
                    self.damage = self.damage.wrapping_add(1);
                    self.anm_init(BCK_BEAM_MAE, 5.0, EMODE_NONE, 1.0);
                    sink.se(SE_CM_BST_MOUTH_OPEN);
                    break;
                }
                1 => {
                    if self.morph.is_stop() {
                        self.damage = self.damage.wrapping_add(1);
                        self.anm_init(BCK_BEAM_TAME, 1.0, EMODE_LOOP, 1.0);
                        self.timers[0] = 0x2d;
                        for slot in self.charge.iter_mut() {
                            *slot = true;
                        }
                    }
                    break;
                }
                2 => {
                    sink.se(SE_CM_BST_BEAM_PREPARE);
                    self.kan_state = 5;
                    if self.timers[0] == 0 {
                        for slot in self.charge.iter_mut() {
                            *slot = false;
                        }
                        self.damage = self.damage.wrapping_add(1);
                        self.anm_init(BCK_BEAM, 1.0, EMODE_LOOP, 1.0);
                        self.timers[0] = REG0_S4 + 50;
                    }
                    break;
                }
                3 => {
                    add_calc2(&mut self.sway, REG0_F6, 0.1, REG0_F5 + 20.0);
                    if (self.timers[0] & 3) == 0 {
                        self.kan_state = 6;
                        self.beam_set();
                        sink.se(SE_CM_BST_BEAM_FIRE);
                    }
                    if self.timers[0] == 0 {
                        b_var3 = true;
                    }
                    break;
                }
                _ => break,
            }
        }
        if env.distance(self.pos) < REG0_F9 + 500.0 || b_var3 {
            self.kan_state = 8;
            self.action = ACTION_FLY;
            self.damage = 0;
            self.speed_f = 0.0;
            sink.se(SE_CM_BST_MOUTH_CLOSE);
            let s_var6 = if self.health == 0 {
                self.rnd_ang(50.0, 100.0)
            } else if self.health == 1 {
                self.rnd_ang(70.0, 120.0)
            } else if self.health == 2 {
                self.rnd_ang(80.0, 140.0)
            } else if self.health == 3 {
                self.rnd_ang(90.0, 150.0)
            } else {
                self.rnd_ang(100.0, 150.0)
            };
            self.phase_timer[1] = s_var6;
        }
    }

    fn damage(&mut self, sink: &mut Sink<'_>) {
        self.state = 10;
        let part = self.part;
        let arm = self.damage;
        loop {
            match arm {
                0 => {
                    if part != PART_HEAD {
                        self.anm_init(pick(&DAMAGE_BCK, part), 1.0, EMODE_NONE, 1.0);
                    }
                    self.tev_init(pick(&DAMAGE_BRK, part));
                    self.srt_init(pick(&DAMAGE_BTK, part));
                    self.damage = self.damage.wrapping_add(1);
                    self.timers[0] = 30;
                    break;
                }
                _ => break,
            }
        }
        if self.timers[0] == 0 {
            if part != PART_HEAD && self.health <= 0 {
                self.action = ACTION_SLEEP;
                self.timers[1] = 300;
                if part == PART_LEFT {
                    if self.action != ACTION_SLEEP {
                        sink.bgm(BGM_UNK_120);
                    }
                } else if self.action != ACTION_SLEEP {
                    sink.bgm(BGM_UNK_121);
                }
            } else {
                self.action = ACTION_FLY;
                self.speed_f = 0.0;
            }
            self.damage = 0;
        }
    }

    fn bom_eat_check(&mut self, shells: &[BombView], sink: &mut Sink<'_>) -> bool {
        for shell in shells.iter() {
            if !shell.free || shell.rest_time <= 1 {
                continue;
            }
            if !touches(self.hand_centre, self.hand_radius, shell.pos, 0.0) {
                continue;
            }
            self.bomb = Some(shell.entity);
            self.bomb_pos = shell.pos;
            sink.eaten.push(shell.entity);
            return true;
        }
        false
    }

    fn head_damage(
        &mut self,
        peers: &[Part],
        shells: &[BombView],
        writes: &mut Vec<Part>,
        sink: &mut Sink<'_>,
    ) {
        self.state = 10;
        self.eye_timer[0] = 10;
        self.eye_timer[1] = 10;
        add_calc_angle_s2(&mut self.shape_angle[0], 0, 8, 512);
        add_calc_angle_s2(&mut self.shape_angle[2], 0, 8, 512);
        add_calc0(&mut self.offset.z, 1.0, 5.0);
        let mut arm = self.damage;
        loop {
            match arm {
                0 => {
                    self.damage = self.damage.wrapping_add(1);
                    self.timers[0] = 40;
                    self.speed_f = 0.0;
                    self.speed.y = 0.0;
                    self.tev_init(BRK_BST_END);
                    self.srt_init(BTK_BST_END);
                    sink.se(SE_CM_BST_HEAD_EYE_CLOSE);
                    sink.bgm(BGM_UNK_123);
                    arm = 1;
                }
                1 => {
                    let mut v = self.pos;
                    v.y = 0.0;
                    let mut dist = span(v);
                    if dist > REG0_F9 + 1350.0 {
                        dist = (REG0_F9 + 1350.0) / dist;
                        add_calc2(&mut self.pos.x, v.x * dist, 0.2, 30.0);
                        add_calc2(&mut self.pos.z, v.z * dist, 0.2, 30.0);
                    }
                    if self.timers[0] == 0 {
                        self.pos.y += self.speed.y;
                        self.speed.y -= REG0_F13 + 5.0;
                        if self.pos.y <= REG0_F14 {
                            self.pos.y = REG0_F14;
                            self.shake_alt = REG0_S7 + 4;
                            sink.se(SE_CM_BST_HEAD_FALL);
                            sink.se(SE_CM_BST_MOUTH_OPEN);
                            self.anm_init(BCK_KUTI_OPEN, 1.0, EMODE_NONE, 1.0);
                            self.damage = self.damage.wrapping_add(1);
                            self.speed.y = 0.0;
                        }
                    }
                    break;
                }
                2 => {
                    if self.morph.is_stop() {
                        self.anm_init(BCK_KUTI_OPEN_LOOP, 1.0, EMODE_LOOP, 1.0);
                        self.timers[0] = 300;
                        self.damage = self.damage.wrapping_add(1);
                    }
                    break;
                }
                3 => {
                    if self.bom_eat_check(shells, sink) {
                        self.damage = self.damage.wrapping_add(1);
                        self.timers[0] = 10;
                        arm = 4;
                    } else if self.timers[0] == 0 {
                        self.action = ACTION_HEAD_HUKKI;
                        self.damage = 0;
                        sink.bgm(BGM_UNK_122);
                        break;
                    } else {
                        break;
                    }
                }
                4 => {
                    if self.bomb.is_some() {
                        add_calc2(&mut self.bomb_pos.x, self.pos.x, 1.0, 30.0);
                        add_calc2(
                            &mut self.bomb_pos.y,
                            self.pos.y + BOMB_MOUTH_LIFT + REG0_F11,
                            1.0,
                            30.0,
                        );
                        add_calc2(&mut self.bomb_pos.z, self.pos.z, 1.0, 30.0);
                        *sink.dragged = self.bomb.map(|entity| (entity, self.bomb_pos));
                    }
                    if self.timers[0] != 0 {
                        break;
                    }
                    self.anm_init(BCK_BOM_NOMI, 1.0, EMODE_NONE, 1.0);
                    self.damage = self.damage.wrapping_add(1);
                    if let Some(slot) = self.bomb_eff.get_mut(0) {
                        *slot = true;
                    }
                    sink.se(SE_CM_BST_MOUTH_CLOSE);
                    self.timers[0] = 60;
                    arm = 5;
                }
                5 => {
                    if self.timers[0] == 50 {
                        sink.se(SE_CM_BST_BOMB_EAT);
                    }
                    if self.timers[0] == 0x1e {
                        sink.se(SE_CM_BST_BEF_EXPLODE);
                    }
                    if self.morph.is_stop() {
                        if let Some(slot) = self.bomb_eff.get_mut(0) {
                            *slot = false;
                        }
                        if let Some(entity) = self.bomb.take() {
                            sink.taken.push(entity);
                            self.bomb_pos = Vec3::ZERO;
                        }
                        self.health -= 1;
                        if self.health <= 0 || self.hio_6 != HIO_M6 {
                            self.demo = DEMO_WAKE;
                            self.action = ACTION_END_DEMO;
                            self.damage = 0;
                            self.anm_init(BCK_BST_DEAD, 1.0, EMODE_NONE, 1.0);
                            for kind in [PART_LEFT, PART_RIGHT] {
                                if let Some(slot) = Part::touch(writes, peers, kind) {
                                    slot.action = ACTION_END_DEMO;
                                    slot.damage = 10;
                                }
                            }
                            sink.se(SE_LK_LAST_HIT);
                            sink.stop_bgm();
                            sink.bgm(STRM_BST_CLEAR);
                        } else {
                            self.anm_init(BCK_BOM_BAKUHATU, 1.0, EMODE_NONE, 1.0);
                            self.damage = self.damage.wrapping_add(1);
                        }
                        sink.se(SE_CM_BST_EXPLODE);
                        for index in 1..BOMB_EFF_COUNT {
                            if let Some(slot) = self.bomb_eff.get_mut(index) {
                                *slot = true;
                            }
                        }
                        self.timers[3] = 0x51;
                    }
                    break;
                }
                6 => {
                    if self.morph.frame as i32 == 8 {
                        sink.se(SE_CM_BST_JUMP_S);
                    }
                    if self.morph.is_stop() {
                        self.timers[0] = 0x1e;
                        self.damage = self.damage.wrapping_add(1);
                        sink.bgm(BGM_BST_BATTLE);
                    }
                    break;
                }
                7 => {
                    if self.timers[0] == 0 {
                        self.action = ACTION_HEAD_HUKKI;
                        self.damage = 0;
                    }
                    break;
                }
                _ => break,
            }
        }
        for index in 0..BOMB_EFF_COUNT {
            if self.bomb_eff[index] && index >= 1 && self.timers[3] == 1 {
                self.bomb_eff[index] = false;
            }
        }
    }

    fn head_hukki(&mut self) {
        let mut arm = self.damage;
        loop {
            match arm {
                0 => {
                    self.damage = self.damage.wrapping_add(1);
                    self.timers[0] = REG0_S0 + 40;
                    self.sway = 0.0;
                    self.offset.y = 0.0;
                    arm = 1;
                }
                1 => {
                    add_calc2(&mut self.sway, REG0_F4 + 4000.0, 1.0, REG0_F5 + 100.0);
                    if self.timers[0] == 0 {
                        self.damage = self.damage.wrapping_add(1);
                        self.timers[0] = REG0_S1 + 80;
                    }
                    break;
                }
                2 => {
                    add_calc0(&mut self.sway, 1.0, REG0_F6 + 50.0);
                    if self.timers[0] == 0 {
                        self.action = ACTION_FLY;
                        self.damage = 0;
                        self.speed_f = 0.0;
                        self.offset.y = 0.0;
                        self.eye_health[1] = EYE_HEALTH;
                        self.eye_health[0] = EYE_HEALTH;
                    }
                    break;
                }
                _ => break,
            }
        }
        self.shape_angle[0] = ang(
            self.sway * sin_s(((self.sway_clock as i32) * (REG0_S6 as i32 + BOB_X)) as i16),
        );
        self.shape_angle[2] = ang(
            self.sway * sin_s(((self.sway_clock as i32) * (REG0_S6 as i32 + HUKKI_X)) as i16),
        );
        add_calc2(&mut self.pos.y, REG0_F11 + 400.0, 0.05, self.offset.y * 40.0);
        add_calc2(&mut self.offset.z, REG0_F11 + 50.0, 1.0, 2.0);
        add_calc2(&mut self.offset.y, 1.0, 1.0, 0.02);
    }

    fn col_set(&mut self) {
        if self.part == PART_HEAD {
            self.head_centre = self.joint_off(JNT_KUTI, HEAD_OFF);
            self.head_height = REG0_F3 + HEAD_HEIGHT;
            self.head_radius = REG0_F4 + HEAD_RADIUS;
            self.hand_tg = false;
            self.hand_centre = self.joint_off(JNT_KUTI, HAND_OFF);
            self.hand_height = REG0_F8 + HAND_HEIGHT_HEAD;
            self.hand_radius = REG0_F9 + HAND_RADIUS_HEAD;
            self.eye = self.joint_off(JNT_KUTI, EYE_OFF);
            for index in 0..EYE_COUNT {
                let mut pos_vec = self.joint(JNT_EYEL + index as u16);
                if self.eye_timer[index] != 0 || i32::from(self.eye_health[index]) <= 0 {
                    if self.eye_timer[index] != 0 {
                        self.eye_timer[index] -= 1;
                    }
                    pos_vec.y += 10000.0;
                    add_calc_angle_s2(&mut self.eye_hinge[index], REG0_S8 + 11000, 1, 0x400);
                } else {
                    add_calc_angle_s2(&mut self.eye_hinge[index], 0, 8, 0x200);
                }
                self.eye_sph[index] = pos_vec;
                self.eye_radius[index] = REG0_F14 + EYE_RADIUS;
            }
        } else {
            let mut radius = 0.0;
            if self.action == ACTION_HARAI_ATTACK {
                radius = REG0_F16 + FINGER_RADIUS_ATTACK;
            }
            for index in 0..FINGER_ROWS {
                self.finger[index] = self.joint(JNT_FINGER_FIRST + index as u16);
                self.finger_radius[index] = REG0_F14 + FINGER_RADIUS + radius;
            }
            let knuckle = self.joint(JNT_FINGER_KNUCKLE);
            for index in 0..4 {
                let mut vec = Vec3::new(TE_X[index], TE_Y[index], TE_Z[index]);
                if self.part == PART_RIGHT {
                    vec.x *= -1.0;
                }
                if let Some(slot) = self.finger.get_mut(FINGER_ROWS + index) {
                    *slot = knuckle + turned(self.model_yaw, vec.to_array());
                }
                self.finger_radius[FINGER_ROWS + index] = REG0_F15 + KNUCKLE_RADIUS + radius;
            }
            self.eye = self.joint_off(
                JNT_FINGER_KNUCKLE,
                if self.part == PART_RIGHT {
                    [-(REG0_F15 + HAND_SIDE), REG0_F16, REG0_F18 + HAND_UP]
                } else {
                    [REG0_F15 + HAND_SIDE, REG0_F16, -(REG0_F18 + HAND_UP)]
                },
            );
            self.hand_centre = self.joint_off(
                JNT_FINGER_KNUCKLE,
                [REG0_F15 + HAND_SIDE, REG0_F16, REG0_F17 + HAND_FAR],
            );
            self.hand_height = REG0_F8 + HAND_HEIGHT;
            self.hand_radius = REG0_F9 + HAND_RADIUS;
        }
        self.attention = self.eye;
    }

    fn player_way_check(&self, env: &Env<'_>) -> bool {
        let diff = self.shape_angle[1].wrapping_sub(env.yaw());
        let diff = if diff < 0 { -diff } else { diff };
        (diff as u16) >= 0x4000
    }

    fn damage_check(
        &mut self,
        env: &Env<'_>,
        peers: &[Part],
        writes: &mut Vec<Part>,
        sink: &mut Sink<'_>,
    ) {
        let reach = env.blow.at.reach();
        let player = env.pos();
        if self.hurt_timer == 0 {
            let mut b_var1 = false;
            if self.part == PART_HEAD {
                if touches(self.head_centre, self.head_radius, player, reach) {
                    b_var1 = true;
                }
            } else {
                for index in 0..FINGER_COUNT {
                    if touches(self.finger[index], self.finger_radius[index], player, reach) {
                        b_var1 = true;
                    }
                }
            }
            if b_var1 {
                sink.se(env.blow.at.at_se());
                self.hurt_timer = 7;
            }
        }
        if self.state != 0 {
            return;
        }
        if self.part == PART_HEAD {
            let hit0 = touches(self.eye_sph[0], self.eye_radius[0], player, reach);
            let hit1 = touches(self.eye_sph[1], self.eye_radius[1], player, reach);
            if !hit0 && !hit1 {
                return;
            }
            self.state = 10;
            if !self.player_way_check(env) {
                return;
            }
            let eye_index = if hit0 { 0usize } else { 1usize };
            if self.kan_state != 1 {
                self.kan_state = 8;
            }
            self.action = ACTION_DAMAGE;
            self.damage = 0;
            sink.se(SE_CM_BST_EYE_HIT);
            self.eye_health[eye_index] = self.eye_health[eye_index].wrapping_sub(1);
            if i32::from(self.eye_health[eye_index]) <= 0 {
                if i32::from(self.eye_health[1 - eye_index]) <= 0 {
                    self.action = ACTION_HEAD_DAMAGE;
                    self.damage = 0;
                }
                sink.se(SE_CM_BST_EYE_BREAK);
            }
            self.shake = ang(REG0_F11 + 70.0);
            self.shake_alt = REG0_S2 + 7;
            let player_vec = self.pos - player;
            self.recoil_y = to_s16(player_vec.x.atan2(player_vec.z));
            self.recoil_x = -to_s16(player_vec.y.atan2(reach_flat(Vec3::ZERO, player_vec)));
            self.speed_f = 0.0;
            return;
        }
        if !touches(self.hand_centre, self.hand_radius, player, reach) {
            return;
        }
        self.state = 10;
        if !self.player_way_check(env) {
            return;
        }
        sink.se(SE_CM_BST_EYE_HIT);
        if self.health <= 0 {
            sink.se(SE_CM_BST_EYE_BREAK);
        }
        self.shake = ang(REG0_F11 + 70.0);
        self.shake_alt = REG0_S2 + 0xf;
        let player_vec = self.pos - player;
        self.recoil_y = to_s16(player_vec.x.atan2(player_vec.z));
        self.recoil_x = -to_s16(player_vec.y.atan2(reach_flat(Vec3::ZERO, player_vec)));
        self.speed_f = 0.0;
        self.action = ACTION_DAMAGE;
        self.damage = 0;
        if let Some(slot) = Part::touch(writes, peers, self.other_hand()) {
            if slot.action >= ACTION_DOWN_ATTACK {
                slot.action = ACTION_FLY;
                slot.damage = 0;
                slot.speed_f = 0.0;
            }
        }
    }

    fn hana_demo(&mut self, env: &Env<'_>, sink: &mut Sink<'_>) {
        self.state = 10;
        let angle = to_s16((-self.pos.x).atan2(-self.pos.z));
        add_calc_angle_s2(&mut self.shape_angle[1], angle, 10, 0x400);
        add_calc_angle_s2(&mut self.shape_angle[0], 0, 10, 0x400);
        add_calc_angle_s2(&mut self.shape_angle[2], 0, 10, 0x400);
        add_calc2(&mut self.pos.y, REG0_F11 + 400.0, 0.05, 30.0);
        let mut arm = self.damage;
        loop {
            match arm {
                0 => {
                    self.damage = self.damage.wrapping_add(1);
                    self.timers[0] = 40;
                    self.speed_f = 0.0;
                    self.speed.y = 0.0;
                    self.anm_init(BCK_HANA_DEMO, 5.0, EMODE_NONE, 1.0);
                    self.demo = DEMO_HAND_L;
                    sink.se(SE_CM_BST_HAND_STRAIN);
                    arm = 1;
                }
                1 => {
                    let mut vec = self.pos;
                    vec.y = 0.0;
                    let dist = span(vec);
                    if dist > REG0_F9 + 1350.0 {
                        let angle = (REG0_F9 + 1350.0) / dist;
                        add_calc2(&mut self.pos.x, vec.x * angle, 0.2, 30.0);
                        add_calc2(&mut self.pos.z, vec.z * angle, 0.2, 30.0);
                    }
                    if self.morph.frame as i32 == 26 {
                        let item_no;
                        let joint;
                        if env.arrow_num == 0 {
                            item_no = ITEM_ARROW_10;
                            joint = JNT_HANAL;
                            self.nose = 0;
                        } else {
                            item_no = ITEM_BOMB_5;
                            joint = JNT_HANAR;
                            self.nose = 1;
                        }
                        if i32::from(item_no) != ITEM_NO_NONE {
                            sink.se(SE_OBJ_BOMB_EXPLODE);
                            let pos_vec =
                                self.joint_off(joint, [REG0_F2 + 60.0, REG0_F3, REG0_F4]);
                            self.item_pos = pos_vec;
                            sink.spawn_item(
                                u32::from(item_no),
                                pos_vec,
                                [0, 0, 0],
                                self.room,
                                self.set_id,
                            );
                            sink.se(SE_CM_BST_ITEM_OUT_NOSE);
                            let _ = self.joint(JNT_HEAD);
                        }
                    }
                    break;
                }
                _ => break,
            }
        }
    }

    fn end_demo(&mut self, sink: &mut Sink<'_>) {
        self.state = 10;
        self.status = 0;
        self.attention_flags = 0;
        let part = self.part;
        let mut arm = self.damage;
        loop {
            match arm {
                0 => {
                    self.damage = self.damage.wrapping_add(1);
                    arm = 1;
                }
                1 => {
                    if self.morph.frame as i32 == 20 {
                        sink.se(SE_CM_BST_JUMP_L);
                    }
                    break;
                }
                2 => {
                    self.anm_init(BCK_BST_HIRAKU, 1.0, EMODE_NONE, 1.0);
                    self.damage = self.damage.wrapping_add(1);
                    self.halo = true;
                    let _ = self.joint(JNT_HEAD);
                    arm = 3;
                }
                3 => break,
                10 => {
                    if part == PART_HEAD {
                        self.anm_init(pick(&FLY_BCK, part), 2.0, EMODE_LOOP, 1.0);
                    } else {
                        self.anm_init(pick(&SET_BCK, part), 2.0, EMODE_NONE, 1.0);
                    }
                    self.damage = self.damage.wrapping_add(1);
                    arm = 11;
                }
                11 => {
                    add_calc2(&mut self.pos.x, self.home_pos.x, 0.1, REG0_F9 + 5.0);
                    add_calc2(&mut self.pos.y, self.home_pos.y, 0.1, REG0_F9 + 5.0);
                    add_calc2(&mut self.pos.z, self.home_pos.z, 0.1, REG0_F9 + 5.0);
                    add_calc_angle_s2(&mut self.shape_angle[1], self.home_angle[1], 10, 0x80);
                    add_calc_angle_s2(&mut self.shape_angle[0], self.home_angle[0], 10, 0x80);
                    add_calc_angle_s2(&mut self.shape_angle[2], self.home_angle[2], 10, 0x80);
                    break;
                }
                _ => break,
            }
        }
        for index in 1..4 {
            if self.bomb_eff[index] && self.timers[3] == 1 {
                self.bomb_eff[index] = false;
            }
        }
    }

    fn action_move(
        &mut self,
        prev: Vec3,
        env: &Env<'_>,
        peers: &[Part],
        shells: &[BombView],
        writes: &mut Vec<Part>,
        sink: &mut Sink<'_>,
    ) {
        let boss = self.head(peers);
        if self.part != PART_HEAD && boss.demo > 0 && boss.demo < 10 {
            if boss.battle == BATTLE_BOSS_BEAM {
                self.action = ACTION_FLY;
            } else {
                self.action = ACTION_SLEEP;
            }
            self.damage = 0;
            self.speed_f = 0.0;
            return;
        }
        self.set_hand_at(false);
        self.set_hand_co(false);
        self.head_at = false;
        let mut b_var1 = false;
        match self.action {
            ACTION_STAY => {
                b_var1 = true;
                self.stay(peers, writes);
            }
            ACTION_FLY => {
                b_var1 = true;
                self.fly(env, peers);
            }
            ACTION_DAMAGE => self.damage(sink),
            ACTION_SLEEP => self.sleep(sink),
            ACTION_HEAD_DAMAGE => self.head_damage(peers, shells, writes, sink),
            ACTION_HEAD_HUKKI => self.head_hukki(),
            ACTION_DOWN_ATTACK => {
                b_var1 = true;
                self.down_attack(env, peers, writes, sink);
            }
            ACTION_PAA_ATTACK => {
                b_var1 = true;
                self.paa_attack(env, peers, writes, sink);
            }
            ACTION_KUMI_ATTACK => {
                b_var1 = true;
                self.kumi_attack(env, peers, writes, sink);
            }
            ACTION_HARAI_ATTACK => {
                b_var1 = true;
                self.harai_attack(env);
            }
            ACTION_BEAM_ATTACK => self.beam_attack(env, sink),
            ACTION_HANA_DEMO => self.hana_demo(env, sink),
            ACTION_END_DEMO => self.end_demo(sink),
            _ => {}
        }
        if b_var1 {
            let dist = (span(self.pos - prev) * 3.5) as u32;
            let dist = dist.min(100);
            let _ = dist;
            if self.part == PART_HEAD {
                sink.se(SE_CM_BST_HEAD_WORKING);
            } else {
                sink.se(SE_CM_BST_HAND_WORKING);
            }
        }
        self.damage_check(env, peers, writes, sink);
        self.pos += self.cc_move;
        if self.recoil_speed > 0.01 {
            let pos_vec = posed(self.recoil_y, self.recoil_x, [0.0, 0.0, self.recoil_speed]);
            self.pos += pos_vec;
            add_calc0(&mut self.recoil_speed, 1.0, 4.0);
        }
    }

    fn try_attack(writes: &mut Vec<Part>, peers: &[Part], kind: u8, action: i16) {
        if let Some(slot) = Part::touch(writes, peers, kind) {
            if slot.action < ACTION_DOWN_ATTACK && slot.wait_timer == 0 {
                slot.action = action;
                slot.damage = 0;
            }
        }
    }

    fn zap_za(writes: &mut Vec<Part>, peers: &[Part], kind: u8, brk: i32, btk: i32) {
        if let Some(slot) = Part::touch(writes, peers, kind) {
            slot.za_brk = brk;
            slot.za_btk = btk;
        }
    }

    fn zap_za_head(&mut self, brk: i32, btk: i32) {
        self.za_brk = brk;
        self.za_btk = btk;
    }

    fn main_cont(
        &mut self,
        env: &Env<'_>,
        peers: &[Part],
        writes: &mut Vec<Part>,
        sink: &mut Sink<'_>,
    ) {
        if self.demo != 0 {
            return;
        }
        if self.drop_timer != 0 {
            self.drop_timer -= 1;
        }
        for index in 0..PHASE_COUNT {
            if self.phase_timer[index] != 0 {
                self.phase_timer[index] -= 1;
            }
        }
        let arm = self.battle;
        loop {
            match arm {
                0 => {
                    self.battle = 1;
                    break;
                }
                1 => {
                    if sink.is_stage_boss_enemy() && REG0_S5 == 0 {
                        break;
                    }
                    self.status |= STTS_SHOWMAP;
                    if sink.is_stage_boss_demo() {
                        self.cam_test = 1;
                        sink.bgm(BGM_BST_BATTLE);
                        self.action = ACTION_FLY;
                        self.damage = 0;
                        Self::zap_za(writes, peers, PART_LEFT, BRK_UDEL, BTK_UDEL);
                        Self::zap_za(writes, peers, PART_RIGHT, BRK_UDER, BTK_UDER);
                        self.zap_za_head(BRK_MUNE, BTK_MUNE);
                        for kind in [PART_RIGHT, PART_LEFT] {
                            if let Some(slot) = Part::touch(writes, peers, kind) {
                                slot.damage = 0;
                            }
                        }
                        self.battle = 10;
                        self.drop_timer = 400;
                        self.stage_play = 1.0;
                    } else {
                        self.state = 10;
                        Self::zap_state(writes, peers, PART_RIGHT, 10);
                        Self::zap_state(writes, peers, PART_LEFT, 10);
                        if span(env.pos()) < 700.0 {
                            self.demo = DEMO_STAGE;
                            self.battle = 2;
                        }
                    }
                    break;
                }
                9 => break,
                10 => {
                    self.battle += 1;
                    self.status &= !STTS_UNK4000;
                    self.phase_timer[0] = 100;
                    break;
                }
                11 => {
                    if self.phase_timer[0] == 0 && self.action != 14 {
                        self.wall_radius = 200.0;
                        self.wall_height = 300.0;
                        Self::zap_wall(writes, peers, PART_LEFT);
                        Self::zap_wall(writes, peers, PART_RIGHT);
                        let forced = self.rnd_f(1.0) < 0.2 || REG0_S1 != 0;
                        if forced {
                            if self.rnd_f(1.0) < 0.5 {
                                Self::try_attack(writes, peers, PART_LEFT, ACTION_HARAI_ATTACK);
                            } else {
                                Self::try_attack(writes, peers, PART_RIGHT, ACTION_HARAI_ATTACK);
                            }
                        } else {
                            let threshold = if self.health == 0 {
                                1.0 - 0.3
                            } else if self.health == 1 {
                                0.5
                            } else if self.health == 2 {
                                0.3
                            } else if self.health == 3 {
                                0.2
                            } else {
                                0.0
                            };
                            let left = Self::peek(writes, peers, PART_LEFT);
                            let right = Self::peek(writes, peers, PART_RIGHT);
                            let both_free = left.action < ACTION_DAMAGE
                                && right.action < ACTION_DAMAGE
                                && left.wait_timer == 0
                                && right.wait_timer == 0;
                            if self.rnd_f(1.0) < threshold || REG0_S0 != 0 {
                                if both_free {
                                    Self::try_attack(writes, peers, PART_LEFT, ACTION_KUMI_ATTACK);
                                    Self::try_attack(writes, peers, PART_RIGHT, ACTION_KUMI_ATTACK);
                                }
                            } else if self.rnd_f(1.0) < 0.5 {
                                if both_free {
                                    Self::try_attack(writes, peers, PART_LEFT, ACTION_PAA_ATTACK);
                                    Self::try_attack(writes, peers, PART_RIGHT, ACTION_PAA_ATTACK);
                                }
                            } else if self.rnd_f(1.0) < 0.5 {
                                let left = Self::peek(writes, peers, PART_LEFT);
                                if left.action < ACTION_DAMAGE && left.wait_timer == 0 {
                                    Self::try_attack(writes, peers, PART_LEFT, ACTION_DOWN_ATTACK);
                                }
                            } else {
                                let right = Self::peek(writes, peers, PART_RIGHT);
                                if right.action < ACTION_DAMAGE && right.wait_timer == 0 {
                                    Self::try_attack(writes, peers, PART_RIGHT, ACTION_DOWN_ATTACK);
                                }
                            }
                        }
                        let unk = if self.health == 0 {
                            self.rnd_ang(30.0, 30.0)
                        } else if self.health == 1 {
                            self.rnd_ang(40.0, 50.0)
                        } else {
                            self.rnd_ang(50.0, 70.0)
                        };
                        self.phase_timer[0] = unk;
                    }
                    self.eye_timer[1] = 10;
                    self.eye_timer[0] = 10;
                    let left = Self::peek(writes, peers, PART_LEFT);
                    let right = Self::peek(writes, peers, PART_RIGHT);
                    if self.phase_timer[2] == 0
                        && left.action == ACTION_SLEEP
                        && right.action == ACTION_SLEEP
                    {
                        self.battle = 12;
                        Self::zap_timer(writes, peers, PART_LEFT, 100);
                        Self::zap_timer(writes, peers, PART_RIGHT, 100);
                        sink.se(SE_CM_BST_HEAD_EYE_OPEN);
                        sink.bgm(BGM_UNK_122);
                        self.phase_timer[1] = self.rnd_ang(50.0, 150.0);
                    }
                    break;
                }
                12 => {
                    if self.phase_timer[1] == 0 && self.action == ACTION_FLY {
                        self.action = ACTION_BEAM_ATTACK;
                        self.damage = 0;
                        self.sway = REG0_F7 + (-500.0);
                    }
                    Self::zap_sleep(writes, peers, PART_LEFT);
                    Self::zap_timer(writes, peers, PART_RIGHT, 3);
                    if self.action == ACTION_HEAD_DAMAGE && self.damage >= 7 {
                        self.battle = 11;
                        self.phase_timer[2] = 50;
                    }
                    break;
                }
                100 => break,
                _ => break,
            }
        }
    }

    fn zap_state(writes: &mut Vec<Part>, peers: &[Part], kind: u8, value: i16) {
        if let Some(slot) = Part::touch(writes, peers, kind) {
            slot.state = value;
        }
    }

    fn zap_wall(writes: &mut Vec<Part>, peers: &[Part], kind: u8) {
        if let Some(slot) = Part::touch(writes, peers, kind) {
            slot.wall_radius = 200.0;
            slot.wall_height = 200.0;
        }
    }

    fn zap_timer(writes: &mut Vec<Part>, peers: &[Part], kind: u8, value: i16) {
        if let Some(slot) = Part::touch(writes, peers, kind) {
            slot.timers[1] = value;
        }
    }

    fn zap_sleep(writes: &mut Vec<Part>, peers: &[Part], kind: u8) {
        if let Some(slot) = Part::touch(writes, peers, kind) {
            slot.action = ACTION_SLEEP;
            slot.timers[1] = 3;
        }
    }

    fn beam_wall_check(&mut self, env: &Env<'_>, index: usize) -> bool {
        let pos_1 = self.beam_pos[index];
        let pos_2 = self.beam_dir[index];
        let mut start = pos_1 - pos_2;
        start.y += BEAM_WALL_LIFT;
        let end = Vec3::new(pos_1.x, start.y, pos_1.z);
        if let Some(solid) = env.solid {
            if solid.slide(start, end) != end {
                self.beam_pos[index] = end;
                return true;
            }
        }
        false
    }

    fn beam_move(&mut self, env: &Env<'_>, sink: &mut Sink<'_>) {
        for index in 0..BEAM_COUNT {
            if self.beam_live[index] == 0 {
                continue;
            }
            self.beam_pos[index] += self.beam_dir[index];
            add_calc2(
                &mut self.beam_scale[index],
                REG0_F4 + BEAM_SCALE_UP,
                1.0,
                REG0_F5 + BEAM_SCALE_STEP,
            );
            let mut beam_set = false;
            let probe = Vec3::new(
                self.beam_pos[index].x,
                self.beam_pos[index].y + BEAM_GROUND_LIFT,
                self.beam_pos[index].z,
            );
            let ground_cross = env
                .solid
                .and_then(|solid| solid.ground_at(probe.x, probe.z))
                .unwrap_or(0.0);
            if self.beam_pos[index].y <= ground_cross {
                self.beam_pos[index].y = ground_cross;
                beam_set = true;
            } else if self.beam_wall_check(env, index) {
                beam_set = true;
            }
            if self.beam_pos[index].y > BEAM_CEILING {
                self.beam_live[index] = 0;
            }
            if beam_set {
                self.beam_live[index] = 0;
                sink.se(SE_CM_BST_BEAM_BOMB);
            }
            if self.beam_live[index] == 1 {
                self.beam_live[index] += 1;
            }
        }
    }

    fn end_brkbtk_set(&mut self, peers: &[Part], writes: &mut Vec<Part>) {
        if let Some(slot) = Part::touch(writes, peers, PART_LEFT) {
            slot.tev_key = BRK_LH_END;
            slot.srt_key = BTK_LH_END;
            slot.za_brk = BRK_UDEL_END;
            slot.za_btk = BTK_UDEL_END;
        }
        if let Some(slot) = Part::touch(writes, peers, PART_RIGHT) {
            slot.tev_key = BRK_RH_END;
            slot.srt_key = BTK_RH_END;
            slot.za_brk = BRK_UDER_END;
            slot.za_btk = BTK_UDER_END;
        }
        self.tev_key = BRK_BST_END;
        self.srt_key = BTK_BST_END;
        self.za_brk = BRK_MUNE_END;
        self.za_btk = BTK_MUNE_END;
    }

    fn demo_gate(&mut self) -> bool {
        if !self.demo_ordered {
            self.demo_ordered = true;
            return false;
        }
        true
    }

    fn demo_camera(
        &mut self,
        env: &mut Env<'_>,
        peers: &[Part],
        writes: &mut Vec<Part>,
        sink: &mut Sink<'_>,
    ) {
        let mut b_var2 = true;
        let mut arm = self.demo;
        loop {
            match arm {
                1 => {
                    if !self.demo_gate() {
                        b_var2 = false;
                        break;
                    }
                    self.demo += 1;
                    self.demo_count = 0;
                    self.cam_fov = DEMO_FOV;
                    self.cam_step = 0.0;
                    self.blure = 100;
                    arm = 2;
                }
                2 => {
                    if self.demo_count > REG0_S4 + 33 {
                        add_calc2(&mut self.cam_step, -650.0, 0.5, REG0_F3 + 50.0);
                        if self.demo_count > REG0_S4 + 45 {
                            add_calc2(&mut self.cam_fov, 30.0, 0.5, REG0_F3 + 10.0);
                        }
                    }
                    let sp_a4 = turned(
                        self.shape_angle[1]
                            .wrapping_add(DEMO_YAW_OFFSET)
                            .wrapping_add(REG0_S1),
                        [
                            if self.nose != 0 {
                                REG0_F9 + -100.0
                            } else {
                                REG0_F10
                            },
                            REG0_F11 + self.cam_step,
                            REG0_F12,
                        ],
                    );
                    self.cam_eye = self.eye + sp_a4;
                    let sp_a4 = turned(REG0_S2 + CAM_PITCH, [0.0, REG0_F7, REG0_F8 + 850.0]);
                    self.cam_centre = self.pos + sp_a4;
                    if self.demo_count > 80 {
                        self.action = ACTION_FLY;
                        self.damage = 0;
                        self.demo = 0;
                        self.blure = 1;
                    }
                    break;
                }
                10 => {
                    if !self.demo_gate() {
                        b_var2 = false;
                        break;
                    }
                    self.demo += 1;
                    self.demo_count = 0;
                    self.cam_fov = DEMO_FOV;
                    self.cam_step = 0.0;
                    self.cam_centre = Vec3::new(
                        (self.pos.x + REG0_F1) - 3200.0,
                        (self.pos.y + REG0_F2) - 1100.0,
                        (self.pos.z + REG0_F3) - 300.0,
                    );
                    self.cam_eye = self.pos;
                    self.cam_eye.y += REG0_F0 - 250.0;
                    sink.bgm(STRM_BST_CLEAR);
                    self.stage_play = 1.0;
                    self.blure = 0x78;
                    arm = 11;
                }
                11 => {
                    env.place(
                        Vec3::new(-635.0, 0.0, 0.0),
                        DEMO_PLACE_ANGLE,
                        self.demo_ordered,
                    );
                    add_calc2(
                        &mut self.cam_centre.z,
                        self.pos.z + REG0_F3 + 300.0,
                        0.02,
                        DEMO_STEP,
                    );
                    if self.demo_count == 60 {
                        self.message_set(MSG_BEAM_MAE, sink);
                    }
                    if self.demo_count == DEMO_MSG_A {
                        self.msg_end = true;
                    }
                    if self.demo_count == REG0_S4 + DEMO_MSG_B {
                        self.message_set(MSG_BEAM_TAME, sink);
                    }
                    if self.demo_count == REG0_S4 + DEMO_MSG_C {
                        self.msg_end = true;
                    }
                    if self.demo_count == REG0_S4 + DEMO_MSG_C + (REG0_S4 + 20) {
                        self.message_set(MSG_BEAM_END, sink);
                    }
                    if self.demo_count == REG0_S4 + DEMO_MSG_D + (REG0_S4 + 20) {
                        self.msg_end = true;
                    }
                    if self.demo_count != REG0_S4 + DEMO_MSG_E + (REG0_S4 + 20) {
                        break;
                    }
                    self.demo = 0xc;
                    self.demo_count = 0;
                    self.cam_step = 0.0;
                    let left = self.peer(PART_LEFT, peers).pos;
                    self.cam_eye = left;
                    self.cam_eye.y += REG0_F4 + 200.0;
                    self.cam_centre = left;
                    self.cam_centre.x += REG0_F5 - 2000.0;
                    self.cam_centre.y += REG0_F6;
                    arm = DEMO_HAND_0;
                }
                DEMO_HAND_0 => {
                    let left = self.peer(PART_LEFT, peers).pos;
                    add_calc2(&mut self.cam_centre.x, left.x - 1200.0, 0.1, self.cam_step * 40.0);
                    add_calc2(&mut self.cam_step, 1.0, 1.0, 0.01);
                    add_calc2(&mut self.cam_eye.x, left.x, 0.02, 50.0);
                    add_calc2(&mut self.cam_eye.z, left.z, 0.02, 50.0);
                    if self.demo_count == 20 {
                        Self::zap_demo(writes, peers, PART_LEFT);
                        sink.se(SE_CM_BST_HAND_LIGHT);
                    }
                    if self.demo_count == 59 {
                        self.kan_state = 1;
                    }
                    if self.demo_count == 60 {
                        Self::zap_damage(writes, peers, PART_LEFT, 2);
                        sink.se(SE_CM_BST_HAND_EYE_OPEN);
                    }
                    if self.demo_count == 90 {
                        Self::zap_damage(writes, peers, PART_LEFT, 4);
                        sink.se(SE_CM_BST_HAND_OUT_WALL);
                    }
                    if self.demo_count == 169 {
                        self.kan_state = 0;
                    }
                    if self.demo_count != 170 {
                        break;
                    }
                    self.demo = 0xd;
                    self.demo_count = 0;
                    self.cam_step = 0.0;
                    let right = self.peer(PART_RIGHT, peers).pos;
                    self.cam_eye = right;
                    self.cam_eye.y += REG0_F4 + 200.0;
                    self.cam_centre = right;
                    self.cam_centre.x += REG0_F5 - 2000.0;
                    self.cam_centre.y += REG0_F6;
                    arm = DEMO_HAND_1;
                }
                DEMO_HAND_1 => {
                    let right = self.peer(PART_RIGHT, peers).pos;
                    add_calc2(&mut self.cam_centre.x, right.x - 1200.0, 0.1, self.cam_step * 40.0);
                    add_calc2(&mut self.cam_step, 1.0, 1.0, 0.01);
                    add_calc2(&mut self.cam_eye.x, right.x, 0.02, 50.0);
                    add_calc2(&mut self.cam_eye.z, right.z, 0.02, 50.0);
                    if self.demo_count == 20 {
                        Self::zap_demo(writes, peers, PART_RIGHT);
                        sink.se(SE_CM_BST_HAND_LIGHT);
                    }
                    if self.demo_count == 59 {
                        self.kan_state = 1;
                        self.kan_amount = 0.0;
                    }
                    if self.demo_count == 60 {
                        Self::zap_damage(writes, peers, PART_RIGHT, 2);
                        sink.se(SE_CM_BST_HAND_EYE_OPEN);
                    }
                    if self.demo_count == 90 {
                        Self::zap_damage(writes, peers, PART_RIGHT, 4);
                        sink.se(SE_CM_BST_HAND_OUT_WALL);
                    }
                    if self.demo_count == 169 {
                        self.kan_state = 0;
                    }
                    if self.demo_count != 170 {
                        break;
                    }
                    self.demo = 0xe;
                    self.demo_count = 0;
                    self.cam_step = 0.0;
                    self.cam_eye = self.pos;
                    self.cam_eye.y += REG0_F7 + 300.0;
                    self.cam_centre = self.pos;
                    self.cam_centre.x += REG0_F8 - 2000.0;
                    self.cam_centre.y += REG0_F7 + 300.0;
                    arm = DEMO_BEAM;
                }
                DEMO_BEAM => {
                    if self.demo_count >= 30 {
                        if self.demo_count <= 90 {
                            add_calc2(
                                &mut self.cam_centre.x,
                                self.pos.x - 1000.0,
                                0.2,
                                self.cam_step * 100.0,
                            );
                            add_calc2(&mut self.cam_step, 1.0, 1.0, 0.05);
                        }
                        if self.demo_count == 30 {
                            self.za_on = 1;
                            self.brk_on = 1;
                            sink.se(SE_CM_BST_HEAD_LIGHT);
                        }
                        if self.demo_count == 59 {
                            self.kan_state = 1;
                            self.kan_amount = 0.0;
                        }
                        if self.demo_count == 60 {
                            self.damage = 2;
                            sink.se(SE_CM_BST_HEAD_EYE_OPEN);
                        }
                        if self.demo_count == 90 {
                            self.damage = 4;
                            sink.se(SE_CM_BST_HEAD_OUT_WALL);
                        }
                        if self.demo_count == 150 {
                            self.demo = 0xf;
                            self.action = ACTION_FLY;
                            self.damage = 0;
                            for kind in [PART_LEFT, PART_RIGHT] {
                                if let Some(slot) = Part::touch(writes, peers, kind) {
                                    slot.action = ACTION_FLY;
                                    slot.damage = 0;
                                }
                            }
                            self.cam_eye = self.eye;
                            self.cam_eye.y += REG0_F0;
                            self.cam_centre = env.pos();
                            self.cam_centre.x -= REG0_F1 + 500.0;
                            self.cam_centre.y += REG0_F2 + 100.0;
                            self.cam_fov = REG0_F4 + DEMO_LIFT;
                            Self::zap_za(writes, peers, PART_LEFT, BRK_UDEL, BTK_UDEL);
                            Self::zap_za(writes, peers, PART_RIGHT, BRK_UDER, BTK_UDER);
                            self.zap_za_head(BRK_MUNE, BTK_MUNE);
                            self.drop_timer = 500;
                        }
                    }
                    break;
                }
                DEMO_STAGE_OUT => {
                    add_calc2(&mut self.cam_eye.x, self.pos.x, 0.1, 50.0);
                    add_calc2(&mut self.cam_eye.y, self.eye.y + REG0_F0, 0.1, 50.0);
                    add_calc2(&mut self.cam_eye.z, self.pos.z, 0.1, 50.0);
                    self.cam_centre = env.pos();
                    self.cam_centre.x -= REG0_F1 + 500.0;
                    self.cam_centre.y += REG0_F2 + 100.0;
                    add_calc2(&mut self.cam_fov, DEMO_FOV, 0.1, 0.4);
                    if self.demo_count == 200 {
                        self.stage_eff = true;
                    }
                    if self.demo_count == 300 {
                        self.demo = 0;
                        self.battle = 10;
                        sink.mem.on_stage_boss_demo();
                        sink.bgm(BGM_BST_BATTLE);
                        self.cam_test = 1;
                        self.blure = 1;
                    }
                    break;
                }
                DEMO_WAKE => {
                    if !self.demo_gate() {
                        b_var2 = false;
                        break;
                    }
                    self.demo += 1;
                    self.demo_count = 0;
                    self.cam_fov = DEMO_FOV;
                    self.cam_step = 0.0;
                    self.cam_centre.x = (REG0_F2 + -500.0) - 800.0;
                    self.end_brkbtk_set(peers, writes);
                    if self.stage_eff {
                        self.stage_eff = false;
                    }
                    self.cam_test = 0;
                    self.blure = 150;
                    arm = 51;
                }
                51 => {
                    env.place(
                        Vec3::new(-300.0, 0.0, 0.0),
                        DEMO_PLACE_ANGLE,
                        self.demo_ordered,
                    );
                    self.pos.x = 300.0;
                    self.pos.z = 0.0;
                    self.shape_angle[1] = -0x4000;
                    self.cam_eye = self.pos;
                    self.cam_eye.y += REG0_F0 + 300.0;
                    self.cam_centre.y = REG0_F3 + 200.0;
                    self.cam_centre.z = 0.0;
                    add_calc2(
                        &mut self.cam_centre.x,
                        (REG0_F2 + -500.0) - 100.0,
                        0.2,
                        self.cam_step * 10.0,
                    );
                    add_calc2(&mut self.cam_step, 1.0, 1.0, 0.05);
                    if self.demo_count < 80 {
                        self.cam_shake = REG0_F9 + 10.0;
                    }
                    if self.demo_count == 90 {
                        self.damage = 2;
                        self.kan_state = 2;
                        self.kan_amount = 0.0;
                    }
                    if self.demo_count == 100 {
                        self.kan_state = 3;
                    }
                    if self.demo_count > 100 && self.demo_count < 530 {
                        sink.se(SE_CM_BST_GOKOU);
                    }
                    if self.demo_count == 120 {
                        self.message_set(MSG_DEAD_A, sink);
                    }
                    if self.demo_count == DEMO_MSG_F {
                        self.msg_end = true;
                    }
                    if self.demo_count == REG0_S4 + DEMO_MSG_G {
                        self.message_set(MSG_DEAD_B, sink);
                    }
                    if self.demo_count == REG0_S4 + DEMO_MSG_H {
                        self.msg_end = true;
                    }
                    if self.demo_count == REG0_S4 + DEMO_MSG_H + (REG0_S4 + 20) {
                        self.message_set(MSG_DEAD_C, sink);
                    }
                    if self.demo_count == REG0_S4 + DEMO_MSG_I + (REG0_S4 + 20) {
                        self.msg_end = true;
                    }
                    if self.demo_count
                        == REG0_S4 + DEMO_MSG_I + (REG0_S4 + 20) + REG0_S4 + 20
                    {
                        self.message_set(MSG_DEAD_D, sink);
                    }
                    if self.demo_count
                        == REG0_S4 + DEMO_MSG_L + (REG0_S4 + DEMO_MSG_J + REG0_S4)
                    {
                        self.msg_end = true;
                    }
                    if self.demo_count
                        != REG0_S4 + DEMO_MSG_K + (REG0_S4 + DEMO_MSG_J + REG0_S4)
                    {
                        break;
                    }
                    self.demo += 1;
                    self.demo_count = 0;
                    self.cam_step = 0.0;
                    self.cam_eye = Vec3::new(0.0, REG0_F16 + 4800.0, 0.0);
                    self.cam_centre = Vec3::new(
                        REG0_F13 + -700.0,
                        REG0_F14 + 100.0,
                        REG0_F15 + -300.0,
                    );
                    sink.bgm(STRM_BST_CLEAR);
                    self.blure = 1;
                    arm = 52;
                }
                52 => {
                    if self.demo_count <= 30 {
                        break;
                    }
                    self.demo += 1;
                    self.demo_count = 0;
                    self.cam_step = 0.0;
                    arm = 53;
                }
                53 => {
                    self.cam_eye = env.pos();
                    self.cam_eye.y += REG0_F10 + 100.0;
                    self.cam_centre = env.pos();
                    self.cam_centre.x += REG0_F11 + 150.0;
                    self.cam_centre.y += REG0_F12 + 50.0;
                    if self.demo_count <= REG0_S3 + 65 {
                        break;
                    }
                    self.demo += 1;
                    self.demo_count = 0;
                    self.cam_step = 0.0;
                    self.cam_eye = Vec3::new(0.0, REG0_F16 + 4800.0, 0.0);
                    self.cam_centre = Vec3::new(
                        REG0_F13 + -700.0,
                        REG0_F14 + 100.0,
                        REG0_F15 + -300.0,
                    );
                    self.damage = 10;
                    self.kan_state = 4;
                    self.kan_amount = 1.0;
                    sink.on_stage_boss_enemy();
                    sink.spawn_warp(Vec3::ZERO, [0; 3], self.room, self.set_id);
                    arm = 54;
                }
                54 => {
                    add_calc2(&mut self.cam_eye.y, REG0_F8 + 200.0, 0.8, REG0_F9 + 300.0);
                    if self.demo_count != REG0_S6 + 250 {
                        break;
                    }
                    self.demo += 1;
                    self.demo_count = 0;
                    self.cam_step = 0.0;
                    self.cam_eye = self.pos;
                    self.cam_eye.y += REG0_F10 + 100.0;
                    self.blure = 100;
                    arm = 55;
                }
                55 => {
                    add_calc2(&mut self.cam_eye.x, self.pos.x, 0.2, 20.0);
                    add_calc2(&mut self.cam_eye.y, self.pos.y + 100.0 + REG0_F10, 0.2, 20.0);
                    add_calc2(&mut self.cam_eye.z, self.pos.z, 0.2, 20.0);
                    self.cam_centre = self.pos;
                    self.cam_centre.x -= REG0_F11 + 1300.0;
                    self.cam_centre.y -= REG0_F12 + 450.0;
                    self.cam_centre.z -= REG0_F13 + 500.0;
                    if span(self.home_pos - self.pos) < 10.0 {
                        sink.se(SE_CM_BST_HEAD_IN_WALL);
                        self.cam_shake = REG0_F9 + 10.0;
                        self.shake_alt = REG0_S8 + 3;
                        self.pos = self.home_pos;
                        self.demo = 0x38;
                        self.demo_count = 0;
                        self.end_brkbtk_set(peers, writes);
                    }
                    break;
                }
                56 => {
                    if self.demo_count == REG0_S4 + 50 {
                        self.anm_init(BCK_HANA_DEMO, 5.0, EMODE_NONE, 1.0);
                        sink.se(SE_CM_BST_HAND_STRAIN);
                        self.blure = 1;
                    }
                    if self.morph.frame as i32 == 26 {
                        sink.se(SE_OBJ_BOMB_EXPLODE);
                        let sp_a4 = self.joint_off(JNT_HANAR, [REG0_F2 + 60.0, REG0_F3, REG0_F4]);
                        let mut item_angle = self.shape_angle;
                        item_angle[1] = (item_angle[1] as i32 + REG0_S7 as i32 + -300) as i16;
                        self.item_pos = sp_a4;
                        sink.spawn_item(
                            u32::from(ITEM_HEART_CONTAINER),
                            sp_a4,
                            item_angle,
                            self.room,
                            self.set_id,
                        );
                        sink.se(SE_CM_BST_ITEM_OUT_NOSE);
                        let _ = self.joint(JNT_HEAD);
                    }
                    if self.morph.frame as i32 > 26 && self.item.is_some() {
                        self.demo = DEMO_ITEM;
                    }
                    break;
                }
                DEMO_ITEM => {
                    if self.item.is_some() {
                        add_calc2(&mut self.cam_eye.x, self.item_pos.x, 0.1, 100.0);
                        add_calc2(&mut self.cam_eye.y, self.item_pos.y, 0.1, 100.0);
                        add_calc2(&mut self.cam_eye.z, self.item_pos.z, 0.1, 100.0);
                        self.cam_fov = REG0_F4 + DEMO_FOV;
                    }
                    if self.demo_count == 170 {
                        self.demo = 0;
                        self.battle = 100;
                        self.blure = 1;
                    }
                    break;
                }
                _ => break,
            }
        }
        if self.demo != 0 && b_var2 {
            let x = self.cam_shake * sin_s(((self.demo_count as i32) * CAM_SHAKE_A) as i16);
            let y = self.cam_shake * cos_s(((self.demo_count as i32) * CAM_SHAKE_B) as i16);
            let z = self.cam_shake * cos_s(((self.demo_count as i32) * CAM_SHAKE_C) as i16);
            self.cam_centre += Vec3::new(x, y, z);
            self.cam_eye -= Vec3::new(x, y, z);
            add_calc0(&mut self.cam_shake, 1.0, 1.0);
            self.demo_count = self.demo_count.wrapping_add(1);
        }
    }

    fn zap_demo(writes: &mut Vec<Part>, peers: &[Part], kind: u8) {
        if let Some(slot) = Part::touch(writes, peers, kind) {
            slot.za_on = 1;
            slot.brk_on = 1;
        }
    }

    fn peek(writes: &[Part], peers: &[Part], kind: u8) -> Gohdan {
        writes
            .iter()
            .find(|slot| slot.state.part == kind)
            .map(|slot| slot.state)
            .or_else(|| peers.iter().find(|slot| slot.state.part == kind).map(|s| s.state))
            .unwrap_or_else(|| Gohdan::spawn(u32::from(kind)))
    }

    fn zap_damage(writes: &mut Vec<Part>, peers: &[Part], kind: u8, value: i16) {
        if let Some(slot) = Part::touch(writes, peers, kind) {
            slot.damage = value;
        }
    }

    fn bst_kankyo(&mut self) {
        match self.kan_state {
            1 => add_calc2(&mut self.kan_amount, 1.0, 1.0, 0.1),
            2 => add_calc2(&mut self.kan_amount, 1.0, 1.0, 0.25),
            3 => add_calc0(&mut self.kan_amount, 1.0, 0.02),
            4 => add_calc0(&mut self.kan_amount, 1.0, 0.05),
            5 => add_calc0(&mut self.kan_amount, 1.0, 0.1),
            6 => {
                add_calc2(&mut self.kan_amount, 1.0, 1.0, 0.5);
                if self.kan_amount >= 1.0 {
                    self.kan_state = 7;
                }
            }
            7 => add_calc0(&mut self.kan_amount, 1.0, 0.5),
            8 => {
                add_calc2(&mut self.kan_amount, 1.0, 1.0, 0.1);
                if self.kan_amount >= 1.0 {
                    self.kan_state = 1;
                }
            }
            _ => {}
        }
    }

    fn eye_actors(&mut self) {
        for index in 0..EYE_COUNT {
            let Some(target) = self.eye_target[index] else {
                continue;
            };
            let _ = target;
            let live = self.eye_timer[index] == 0
                && self.eye_health[index] > 0
                && self.action != ACTION_HEAD_DAMAGE
                && self.action != ACTION_END_DEMO
                && self.action != ACTION_STAY;
            if live {
                let _ = self.joint(JNT_EYEL + index as u16);
            }
        }
    }

    fn execute(
        &mut self,
        prev: Vec3,
        env: &mut Env<'_>,
        peers: &[Part],
        shells: &[BombView],
        writes: &mut Vec<Part>,
        sink: &mut Sink<'_>,
    ) {
        self.sway_clock = self.sway_clock.wrapping_add(1);
        for index in 0..TIMER_COUNT {
            if self.timers[index] != 0 {
                self.timers[index] -= 1;
            }
        }
        if self.state != 0 {
            self.state -= 1;
        }
        if self.hurt_timer != 0 {
            self.hurt_timer -= 1;
        }
        if self.shake != 0 {
            self.shake -= 1;
        }
        if self.shake_alt != 0 {
            self.shake_alt -= 1;
        }
        if self.shake_far != 0 {
            self.shake_far -= 1;
        }
        if self.wait_timer != 0 {
            self.wait_timer -= 1;
        }
        let boss = self.head(peers);
        if boss.hio_5 == HIO_M5 {
            self.attention_flags = ATTN_LOCKON_BATTLE;
            if self.part == PART_HEAD {
                self.main_cont(env, peers, writes, sink);
                if self.action != ACTION_HEAD_DAMAGE {
                    self.status = 0;
                    self.attention_flags = 0;
                }
                if self.action != ACTION_BEAM_ATTACK {
                    for index in 0..CHARGE_COUNT {
                        self.charge[index] = false;
                    }
                    if self.kan_state == 7 {
                        self.kan_state = 8;
                    }
                }
                if env.pos().y <= -150.0 {
                    self.stage_play = 1.0;
                    self.stage_loop = true;
                } else {
                    self.stage_loop = false;
                }
            }
            self.action_move(prev, env, peers, shells, writes, sink);
            if self.action != 0 && self.action != ACTION_END_DEMO {
                let (next, hit) = slide(env.solid, prev, self.pos);
                self.wall_hit = hit;
                self.pos = next;
            }
            self.morph.play();
            let _ = (self.brk_on, self.za_on);
        }
        self.bob = Vec3::new(
            self.offset.z * sin_s(((self.sway_clock as i32) * (REG0_S6 as i32 + BOB_X)) as i16),
            self.offset.z * sin_s(((self.sway_clock as i32) * (REG0_S6 as i32 + BOB_Y)) as i16),
            self.offset.z * cos_s(((self.sway_clock as i32) * (REG0_S6 as i32 + BOB_Z)) as i16),
        );
        add_calc0(&mut self.offset.z, 1.0, 1.0);
        let boss = self.head(peers);
        let f1 = self.shake as f32 * (REG0_F14 + 500.0);
        let r29 = ang(f1 * sin_s(((self.sway_clock as i32) * SHAKE_YAW_A) as i16));
        let mut r27 = ang(f1 * cos_s(((self.sway_clock as i32) * SHAKE_YAW_B) as i16));
        let f1_2 = self.shake_alt as f32 * (REG0_F14 + 150.0);
        let r29 = r29
            .wrapping_add(ang(f1_2 * sin_s(((boss.sway_clock as i32) * SHAKE_YAW_C) as i16)));
        r27 = r27
            .wrapping_add(ang(f1_2 * cos_s(((boss.sway_clock as i32) * SHAKE_YAW_D) as i16)));
        let f1_3 = self.shake_far as f32 * (REG0_F14 + 100.0);
        r27 = r27
            .wrapping_add(ang(f1_3 * cos_s(((boss.sway_clock as i32) * SHAKE_YAW_E) as i16)));
        self.model_yaw = self.shape_angle[1].wrapping_add(r29);
        add_calc0(&mut self.sway, 1.0, REG0_F3 + 30.0);
        add_calc0(&mut self.roll, 1.0, 5.0);
        add_calc0(&mut self.lean, 1.0, 10.0);
        add_calc0(&mut self.twist, 1.0, 5.0);
        self.col_set();
        if self.part == PART_HEAD {
            self.beam_move(env, sink);
            self.demo_camera(env, peers, writes, sink);
            self.bst_kankyo();
            self.message_cont(sink);
            self.eye_actors();
        }
        let boss = self.head(peers);
        self.hidden = self.part != PART_HEAD && boss.demo > 0 && boss.demo < 10;
        if self.blure == 1 {
            self.blure = 0;
        }
        let _ = r27;
    }
}

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
    Blow { at }
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    combat: Res<'w, Combat>,
    hammer: Res<'w, hammer::Cut>,
    held: Res<'w, Held>,
    blasts: Res<'w, bombflower::Blasts>,
    players: Query<'w, 's, (Entity, &'static mut Player)>,
    shafts: Query<'w, 's, &'static arrow::Arrow>,
    bolts: Query<'w, 's, &'static boomerang::Boomerang>,
    bombs: Query<
        'w,
        's,
        (Entity, &'static mut Actor, &'static mut bomb::Bomb),
        (
            With<bomb::Bomb>,
            Without<Gohdan>,
            Without<pickup::Pickup>,
        ),
    >,
    items: Query<
        'w,
        's,
        (Entity, &'static Actor),
        (With<pickup::Pickup>, Without<Gohdan>),
    >,
}

pub fn control(
    mut watch: Watch<'_, '_>,
    table: Res<MsgTable>,
    mut events: ResMut<SimEvents>,
    mut msgbox: ResMut<MsgBox>,
    mut mem: ResMut<MemBits>,
    mut commands: Commands,
    mut bosses: Query<(Entity, &mut Actor, &mut Gohdan), (With<Gohdan>, Without<Player>)>,
) {
    let solid = watch.collision.as_deref();
    let blow = blow_of(&watch);
    let arrow_num = match watch.held.item {
        item::FIRST_ARROW..=item::LAST_ARROW => watch.held.count,
        _ => 0,
    };
    let bomb_num = match watch.held.item {
        item::FIRST_BOMB..=item::LAST_BOMB => watch.held.count,
        _ => 0,
    };
    let shells: Vec<BombView> = watch
        .bombs
        .iter()
        .map(|(entity, actor, shell)| BombView {
            entity,
            pos: actor.pos,
            free: shell.life != bomb::Life::Carry,
            rest_time: shell.timer,
        })
        .collect();
    let items: Vec<(Entity, Vec3)> = watch
        .items
        .iter()
        .map(|(entity, actor)| (entity, actor.pos))
        .collect();
    let mut players = watch.players;
    let Some(entry) = players.iter_mut().next() else {
        return;
    };
    let mut player = entry.1;
    let mut parts: Vec<Part> = Vec::new();
    for (entity, actor, mut goh) in &mut bosses {
        if !goh.entered {
            goh.entered = true;
            goh.pos = actor.pos;
            goh.home_pos = actor.pos;
            goh.angle = actor.angle;
            goh.home_angle = actor.angle;
            goh.room = actor.room;
            goh.set_id = actor.set_id;
            goh.hio_set = 1;
        }
        parts.push(Part {
            entity,
            state: *goh,
        });
    }
    parts.sort_by_key(|part| part.state.part);
    let table = &*table;
    let mut spawns: Vec<Spawn> = Vec::new();
    let mut taken: Vec<Entity> = Vec::new();
    let mut eaten: Vec<Entity> = Vec::new();
    let mut dragged: Option<(Entity, Vec3)> = None;
    let mut writes: Vec<Part> = Vec::new();
    let mut index = 0;
    while index < parts.len() {
        let mut me = parts[index];
        {
            let mut env = Env {
                player: &mut player,
                solid,
                blow,
                arrow_num,
                bomb_num,
            };
            let mut sink = Sink {
                events: &mut events,
                msg: &mut msgbox,
                table,
                spawns: &mut spawns,
                taken: &mut taken,
                eaten: &mut eaten,
                dragged: &mut dragged,
                mem: &mut mem,
            };
            let prev = me.state.pos;
            me.state.execute(prev, &mut env, &parts, &shells, &mut writes, &mut sink);
        }
        parts[index] = me;
        for slot in writes.drain(..) {
            if let Some(target) = parts
                .iter_mut()
                .find(|target| target.state.part == slot.state.part)
            {
                *target = slot;
            }
        }
        index += 1;
    }
    for (entity, mut actor, mut goh) in &mut bosses {
        if let Some(part) = parts.iter().find(|part| part.entity == entity) {
            *goh = part.state;
            actor.pos = goh.pos;
            actor.angle = goh.angle;
        }
    }
    for entity in eaten {
        if let Ok((_, _, mut shell)) = watch.bombs.get_mut(entity) {
            shell.life = bomb::Life::Carry;
            shell.no_gravity = true;
            shell.timer = BOMB_REST_TIME;
        }
    }
    if let Some((entity, pos)) = dragged {
        if let Ok((_, mut actor, _)) = watch.bombs.get_mut(entity) {
            actor.pos = pos;
        }
    }
    for entity in taken {
        let _ = commands.entity(entity).despawn();
    }
    for spawn in spawns {
        let actor = Actor {
            name: spawn.name.to_owned(),
            parameters: spawn.parameters,
            pos: spawn.pos,
            angle: spawn.angle,
            set_id: spawn.set_id,
            room: spawn.room,
        };
        if spawn.pickup {
            commands.spawn((actor, pickup::Pickup::spawn(spawn.parameters)));
        } else {
            commands.spawn(actor);
        }
    }
    let _ = items;
}
