use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::chest::{Chests, SEA2_STAGE};
use super::collision::Collision;
use super::event::{ActiveEvent, EventList, PresentItem};
use super::events::{SimEvent, SimEvents};
use super::flags::{Collect, EventFlags, StoryBits};
use super::ground;
use super::intent::DoIntent;
use super::pickup::Rupees;
use super::player::Player;
use super::stream;
use super::walk;
use crate::assets::MsgArchives;
use crate::text::driver::TextFlags;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgGameInfo, MsgProcs, fop_msgm_message_set,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED,
    STATUS_MSG_ENDS, STATUS_TYPING,
};

pub const NPC_TC_ACTORS: [&str; 1] = ["Tc"];

pub fn is_npc_tc(name: &str) -> bool {
    NPC_TC_ACTORS.contains(&name)
}

pub const STAFF_NAME: &str = "Tc";

pub const TYPE_MASK: u32 = 0xF;
pub const TYPE_FALLBACK: u8 = 0xF;
pub const PRM_TYPE_SHIFT: u32 = 8;
pub const PRM_TYPE_MASK: u32 = 0xFF;

pub const TYPE_NORMAL2: u8 = 0;
pub const TYPE_RED: u8 = 1;
pub const TYPE_WHITE: u8 = 2;
pub const TYPE_BLUE: u8 = 3;
pub const TYPE_NORMAL: u8 = 4;

pub const STATUS_WAIT: i8 = 1;
pub const STATUS_SIT: i8 = 2;
pub const STATUS_TALK: i8 = 3;
pub const STATUS_JUMP: i8 = 4;
pub const STATUS_WALK_TO_JAIL: i8 = 5;
pub const STATUS_TALK_NEAR_JAIL: i8 = 6;
pub const STATUS_WAIT_NEAR_JAIL: i8 = 7;
pub const STATUS_WALK_TO_STOOL: i8 = 8;
pub const STATUS_SIT_TO_STOOL: i8 = 9;
pub const STATUS_DEMO_JUMP: i8 = 10;
pub const STATUS_DEMO_RESCUE: i8 = 11;
pub const STATUS_DEMO_TALK: i8 = 12;
pub const STATUS_PAY_RUPEE: i8 = 13;
pub const STATUS_DEMO_PAY_RUPEE: i8 = 14;
pub const STATUS_GET_RUPEE: i8 = 15;
pub const STATUS_DEMO_GET_RUPEE: i8 = 16;
pub const STATUS_MONUMENT_COMPLETE: i8 = 17;
pub const STATUS_DEMO_MONUMENT_COMPLETE: i8 = 18;

pub const ACTION_STARTING: i8 = 0;
pub const ACTION_ONGOING: i8 = 1;
pub const ACTION_ENDING: i8 = -1;

pub const BCK_IDX_WAIT01: i8 = 0;
pub const BCK_IDX_WAIT03: i8 = 1;
pub const BCK_IDX_WAIT02: i8 = 2;
pub const BCK_IDX_TALK01: i8 = 3;
pub const BCK_IDX_WALK01: i8 = 4;
pub const BCK_IDX_JAMP_A: i8 = 5;
pub const BCK_IDX_JAMP_B: i8 = 6;
pub const BCK_IDX_JAMP_C: i8 = 7;
pub const BCK_IDX_GUARD: i8 = 8;
pub const BCK_IDX_JTBT: i8 = 9;
pub const BCK_IDX_HAPPY: i8 = 10;
pub const BCK_IDX_DANCE01: i8 = 11;
pub const BCK_IDX_DANCE02: i8 = 12;
pub const BCK_IDX_GET: i8 = 13;
pub const BCK_IDX_MAWASU: i8 = 14;
pub const BCK_IDX_WAIT04: i8 = 15;
pub const BCK_IDX_TALK02: i8 = 16;

pub const ANM_PRM_IDX_NONE: i8 = 0;
pub const ANM_PRM_IDX_WAIT01: i8 = 1;
pub const ANM_PRM_IDX_WAIT03: i8 = 2;
pub const ANM_PRM_IDX_WAIT02: i8 = 3;
pub const ANM_PRM_IDX_TALK01: i8 = 4;
pub const ANM_PRM_IDX_WALK01: i8 = 5;
pub const ANM_PRM_IDX_JAMP_A: i8 = 6;
pub const ANM_PRM_IDX_JAMP_B: i8 = 7;
pub const ANM_PRM_IDX_JAMP_C1: i8 = 8;
pub const ANM_PRM_IDX_JAMP_C2: i8 = 9;
pub const ANM_PRM_IDX_GUARD: i8 = 10;
pub const ANM_PRM_IDX_JTBT: i8 = 11;
pub const ANM_PRM_IDX_HAPPY: i8 = 12;
pub const ANM_PRM_IDX_DANCE01: i8 = 13;
pub const ANM_PRM_IDX_DANCE02: i8 = 14;
pub const ANM_PRM_IDX_GET: i8 = 15;
pub const ANM_PRM_IDX_HAPPY2: i8 = 16;
pub const ANM_PRM_IDX_DANCE01_TO_TALK01: i8 = 17;
pub const ANM_PRM_IDX_DANCE02_TO_WAIT03: i8 = 18;
pub const ANM_PRM_IDX_MAWASU: i8 = 19;
pub const ANM_PRM_IDX_WAIT04: i8 = 20;
pub const ANM_PRM_IDX_TALK02: i8 = 21;
pub const ANM_PRM_IDX_TALK01_WAIT01: i8 = 22;
pub const ANM_PRM_IDX_TALK01_TALK02: i8 = 23;

pub const EMODE_NONE: u8 = 0;
pub const EMODE_LOOP: u8 = 2;
pub const STATE_STOP: u8 = 1;
pub const STATE_LOOP: u8 = 2;

pub const BCK_WAIT01_FRAMES: i16 = 39;
pub const BCK_WAIT03_FRAMES: i16 = 29;
pub const BCK_WAIT02_FRAMES: i16 = 49;
pub const BCK_TALK01_FRAMES: i16 = 29;
pub const BCK_WALK01_FRAMES: i16 = 29;
pub const BCK_JAMP_A_FRAMES: i16 = 64;
pub const BCK_JAMP_B_FRAMES: i16 = 0;
pub const BCK_JAMP_C_FRAMES: i16 = 50;
pub const BCK_GUARD_FRAMES: i16 = 32;
pub const BCK_JTBT_FRAMES: i16 = 19;
pub const BCK_HAPPY_FRAMES: i16 = 29;
pub const BCK_DANCE01_FRAMES: i16 = 179;
pub const BCK_DANCE02_FRAMES: i16 = 121;
pub const BCK_GET_FRAMES: i16 = 0;
pub const BCK_MAWASU_FRAMES: i16 = 50;
pub const BCK_WAIT04_FRAMES: i16 = 40;
pub const BCK_TALK02_FRAMES: i16 = 55;

pub const BTP_FRAME_MAX: u8 = 2;
pub const BTP_RANDOM: f32 = 100.0;
pub const BTP_DELAY: f32 = 30.0;

pub const HIO_M04: f32 = -20.0;
pub const HIO_MAX_HEAD_X: i32 = 0x1FFE;
pub const HIO_MAX_HEAD_Y: i32 = 10000;
pub const HIO_MAX_BACKBONE_X: i32 = 0;
pub const HIO_MAX_BACKBONE_Y: i32 = 0x1C70;
pub const HIO_MIN_HEAD_X: i32 = -0x9C4;
pub const HIO_MIN_HEAD_Y: i32 = -10000;
pub const HIO_MIN_BACKBONE_X: i32 = 0;
pub const HIO_MIN_BACKBONE_Y: i32 = -0x1C70;
pub const HIO_MAX_TURN_STEP: i32 = 3000;
pub const HIO_MAX_HEAD_TURN_VEL: i16 = 0x800;
pub const HIO_ATTN_Y_OFFSET: f32 = 65.0;
pub const HIO_MAX_ATTN_ANGLE_Y: i32 = 0x4000;
pub const HIO_MAX_ATTN_DIST_XZ: f32 = 400.0;
pub const HIO_FIELD_2C: f32 = 900.0;
pub const HIO_FIELD_30: f32 = 400.0;
pub const HIO_FIELD_38: f32 = 20.0;
pub const HIO_FIELD_3C: f32 = -2.5;
pub const HIO_FIELD_40: f32 = 14.0;
pub const HIO_FIELD_44: bool = false;
pub const HIO_FIELD_48: f32 = -25.0;
pub const HIO_FIELD_4C: f32 = -2.5;
pub const HIO_FIELD_50: f32 = 8.0;
pub const HIO_FIELD_54: f32 = 0.35;
pub const HIO_FIELD_58: f32 = 4.0;
pub const HIO_FIELD_5C: f32 = 8.0;
pub const HIO_FIELD_78: i16 = -0x8000;
pub const HIO_FIELD_7A: i16 = 0;

pub const HIO_MORF: [f32; 24] = [
    0.0, 8.0, 8.0, 5.0, 8.0, 4.0, 0.0, 6.0, 6.0, 0.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0,
    8.0, 8.0, 8.0, 8.0, 8.0,
];

pub const HIO_FIELD_60: Vec3 = Vec3::new(-100.0, 1700.0, -50.0);
pub const HIO_FIELD_6C: Vec3 = Vec3::new(100.0, 1700.0, 50.0);

pub const ATTENTION_BONUS_DIST: f32 = 40.0;
pub const ATTENTION_BONUS_ANGLE: i32 = 0x71C;
pub const TALK_RANGE: f32 = HIO_MAX_ATTN_DIST_XZ;
pub const PLAYER_CIRCLE_H: f32 = 100.0;
pub const CHASE_STEP: f32 = 0.1;
pub const MOVE_SCALE: f32 = 0.1;
pub const MOVE_REACH: f32 = 5.0;
pub const JUMP_LAND_TIMER: i8 = 0x33;
pub const SIT_TURN_TOL: f32 = 16.0;
pub const SIT_APPROACH_STEP: f32 = 4.0;
pub const SIT_REACH: f32 = 5.0;
pub const SIT_RISE: f32 = 22.0;
pub const SIT_GRAVITY: f32 = -2.5;
pub const WALL_COS_LIMIT: f32 = 0.9;
pub const MAX_FALL: f32 = -100.0;
pub const PAY_RUPEE_COST: u32 = 398;
pub const TURN_TOLERANCE: f32 = 5376.0;
pub const DEFAULT_MOVE_SPEED: f32 = 2.0;
pub const DEFAULT_TURN_TIMER: i16 = 1;
pub const NO_CUT: i8 = -1;
pub const NO_EVENT: usize = usize::MAX;
pub const NO_STATUS: u8 = 0xFF;
pub const NO_ITEM: u8 = 0xFF;
pub const MONUMENT_TBOX: u8 = 0xF;
pub const COLL_MAP_COUNT: u8 = 8;
pub const COLL_MAP_MAX: u8 = 8;

pub const STAGE_DRC: u8 = 3;
pub const STAGE_FW: u8 = 4;
pub const STAGE_TOTG: u8 = 5;
pub const STAGE_ET: u8 = 6;
pub const STAGE_WT: u8 = 7;

pub const BIT_0B40: u16 = 0x0B40;
pub const BIT_0B80: u16 = 0x0B80;
pub const BIT_1240: u16 = 0x1240;
pub const BIT_1708: u16 = 0x1708;
pub const BIT_1A20: u16 = 0x1A20;
pub const BIT_1D01: u16 = 0x1D01;
pub const BIT_1D02: u16 = 0x1D02;
pub const BIT_1D04: u16 = 0x1D04;
pub const BIT_1D08: u16 = 0x1D08;
pub const BIT_3020: u16 = 0x3020;

pub const MSG_NORMAL_10CD: u32 = 0x10CD;
pub const MSG_NORMAL_10CE: u32 = 0x10CE;
pub const MSG_NORMAL_10CF: u32 = 0x10CF;
pub const MSG_NORMAL_10D0: u32 = 0x10D0;
pub const MSG_NORMAL_10D1: u32 = 0x10D1;
pub const MSG_NORMAL_10D2: u32 = 0x10D2;
pub const MSG_NORMAL_FIRST: u32 = 0x10D3;
pub const MSG_NORMAL_10D4: u32 = 0x10D4;
pub const MSG_NORMAL_10D5: u32 = 0x10D5;
pub const MSG_NORMAL_10D6: u32 = 0x10D6;
pub const MSG_NORMAL_10D7: u32 = 0x10D7;
pub const MSG_NORMAL_10D8: u32 = 0x10D8;
pub const MSG_NORMAL_SECOND: u32 = 0x10D9;
pub const MSG_NORMAL_10DA: u32 = 0x10DA;
pub const MSG_NORMAL_10DB: u32 = 0x10DB;
pub const MSG_NORMAL_10DC: u32 = 0x10DC;
pub const MSG_NORMAL_10DD: u32 = 0x10DD;
pub const MSG_NORMAL_10DE: u32 = 0x10DE;
pub const MSG_NORMAL_10DF: u32 = 0x10DF;
pub const MSG_DAD: u32 = 0xDAD;
pub const MSG_DAE: u32 = 0xDAE;
pub const MSG_DAF: u32 = 0xDAF;
pub const MSG_DB0: u32 = 0xDB0;

pub const MSG_DB2: u32 = 0xDB2;
pub const MSG_DB3: u32 = 0xDB3;
pub const MSG_DB4: u32 = 0xDB4;
pub const MSG_DB5: u32 = 0xDB5;
pub const MSG_DB7: u32 = 0xDB7;
pub const MSG_DB8: u32 = 0xDB8;
pub const MSG_DBA: u32 = 0xDBA;
pub const MSG_DBB: u32 = 0xDBB;
pub const MSG_DBC: u32 = 0xDBC;
pub const MSG_DBD: u32 = 0xDBD;
pub const MSG_DBE: u32 = 0xDBE;
pub const MSG_DBF: u32 = 0xDBF;
pub const MSG_DC0: u32 = 0xDC0;
pub const MSG_DC1: u32 = 0xDC1;
pub const MSG_DC3: u32 = 0xDC3;
pub const MSG_DC4: u32 = 0xDC4;
pub const MSG_DC5: u32 = 0xDC5;
pub const MSG_DC6: u32 = 0xDC6;
pub const MSG_DC7: u32 = 0xDC7;
pub const MSG_DC8: u32 = 0xDC8;
pub const MSG_DC9: u32 = 0xDC9;
pub const MSG_DCA: u32 = 0xDCA;
pub const MSG_DCB: u32 = 0xDCB;
pub const MSG_DCC: u32 = 0xDCC;
pub const MSG_DCD: u32 = 0xDCD;
pub const MSG_DCE: u32 = 0xDCE;
pub const MSG_DCF: u32 = 0xDCF;
pub const MSG_DD0: u32 = 0xDD0;
pub const MSG_DD1: u32 = 0xDD1;
pub const MSG_DD2: u32 = 0xDD2;
pub const MSG_DD3: u32 = 0xDD3;
pub const MSG_DD4: u32 = 0xDD4;
pub const MSG_DD5: u32 = 0xDD5;
pub const MSG_DD6: u32 = 0xDD6;
pub const MSG_DD7: u32 = 0xDD7;
pub const MSG_DD8: u32 = 0xDD8;
pub const MSG_DD9: u32 = 0xDD9;
pub const MSG_DDA: u32 = 0xDDA;
pub const MSG_DDB: u32 = 0xDDB;
pub const MSG_DDC: u32 = 0xDDC;
pub const MSG_DDD: u32 = 0xDDD;
pub const MSG_DDE: u32 = 0xDDE;
pub const MSG_DDF: u32 = 0xDDF;
pub const MSG_DE0: u32 = 0xDE0;
pub const MSG_DE1: u32 = 0xDE1;
pub const MSG_DE2: u32 = 0xDE2;
pub const MSG_DE3: u32 = 0xDE3;
pub const MSG_DE4: u32 = 0xDE4;
pub const MSG_DE5: u32 = 0xDE5;
pub const MSG_DE6: u32 = 0xDE6;
pub const MSG_DE7: u32 = 0xDE7;
pub const MSG_DE8: u32 = 0xDE8;
pub const MSG_DE9: u32 = 0xDE9;
pub const MSG_DEA: u32 = 0xDEA;
pub const MSG_DEB: u32 = 0xDEB;
pub const MSG_DEC: u32 = 0xDEC;
pub const MSG_DED: u32 = 0xDED;
pub const MSG_DEE: u32 = 0xDEE;
pub const MSG_DEF: u32 = 0xDEF;
pub const MSG_DF0: u32 = 0xDF0;
pub const MSG_DF1: u32 = 0xDF1;
pub const MSG_DF2: u32 = 0xDF2;
pub const MSG_DF3: u32 = 0xDF3;
pub const MSG_DF4: u32 = 0xDF4;
pub const MSG_DF5: u32 = 0xDF5;
pub const MSG_DF9: u32 = 0xDF9;
pub const MSG_DFA: u32 = 0xDFA;
pub const MSG_DFB: u32 = 0xDFB;
pub const MSG_38AA: u32 = 0x38AA;
pub const MSG_38AB: u32 = 0x38AB;
pub const MSG_38AC: u32 = 0x38AC;
pub const MSG_38AD: u32 = 0x38AD;

pub const SE_TC_B_PUSH_BAR: u32 = 0x48FE;
pub const SE_TC_T_PUSH_BAR: u32 = 0x48FF;
pub const SE_CMN_NOTICE: u32 = 0x58BD;
pub const SE_DOOR_N_OPEN: u32 = 0x696E;
pub const SE_DOOR_N_CLOSE_1: u32 = 0x696F;
pub const SE_DOOR_N_CLOSE_2: u32 = 0x6970;

pub const ITEM_TINGLE_TUNER: u8 = 0x21;
pub const ITEM_TRIFORCE_MAP_1: u8 = 0x79;
pub const ITEM_TINGLE_RUPEE_1: u8 = 0xB3;
pub const ITEM_TINGLE_RUPEE_6: u8 = 0xB8;
pub const ITEM_COLLECT_MAP_35: u8 = 0xDC;

pub const EVENT_NAMES: [&str; 7] = [
    "TC_JUMP_DEMO",
    "TC_RESCUE",
    "TC_TALK_NEAR_JAIL",
    "TC_TALK_NEAR_JAIL_S",
    "TC_TALK_NEAR_JAIL2",
    "TC_GET_RUPEE",
    "TC_PAY_RUPEE",
];

pub const EVENT_IDX_NONE: i8 = 0;
pub const EVENT_IDX_SPEAK: i8 = 1;
pub const EVENT_IDX_TALK: i8 = 2;
pub const EVENT_IDX_FIRST: i8 = 3;

pub const CUT_ACTIONS: [&str; 9] = [
    "SIT_TO_JUMP",
    "PRESENT",
    "SET_ANM",
    "BACKJUMP",
    "EFFECT",
    "DOOR_OPEN",
    "DOOR_CLOSE",
    "DOOR_CLOSE2",
    "PAY",
];

pub const EVENT_CUT_ACTIONS: [&str; 7] = [
    "WAIT",
    "TURN_TO_ACTOR",
    "MOVE_TO_ACTOR",
    "TURN_TO_POS",
    "MOVE_TO_POS",
    "TALK_MSG",
    "CONTINUE_TALK",
];

pub const CUT_WAIT: i8 = 0;
pub const CUT_MOVE_TO_ACTOR: i8 = 2;
pub const CUT_MOVE_TO_POS: i8 = 4;

pub const ANM_ATR: [i8; 16] = [
    ANM_PRM_IDX_WAIT01,
    ANM_PRM_IDX_WAIT02,
    ANM_PRM_IDX_WAIT03,
    ANM_PRM_IDX_TALK01,
    ANM_PRM_IDX_GUARD,
    ANM_PRM_IDX_JTBT,
    ANM_PRM_IDX_HAPPY,
    ANM_PRM_IDX_DANCE01,
    ANM_PRM_IDX_DANCE02,
    ANM_PRM_IDX_HAPPY2,
    ANM_PRM_IDX_DANCE02_TO_WAIT03,
    ANM_PRM_IDX_DANCE01_TO_TALK01,
    ANM_PRM_IDX_TALK01_WAIT01,
    ANM_PRM_IDX_TALK01_TALK02,
    ANM_PRM_IDX_TALK02,
    ANM_PRM_IDX_WAIT04,
];

pub const TEX_PATTERN_NUM_TBL: [i8; 2] = [-1, 0];

pub const CULL_MIN: [f32; 3] = [-100.0, 0.0, -100.0];
pub const CULL_MAX: [f32; 3] = [100.0, 180.0, 100.0];

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const IS_ZERO: f32 = 8e-11;

const JNT_DIV: i16 = 4;
const JNT_MIN_STEP: i32 = 1;
const JNT_ANG_STEP: i16 = 0x60;
const JNT_TURN_SPEED: i16 = 4;
const HEAD_TURN_SCALE: i16 = 4;
const HEAD_TURN_MAX: i16 = 0x800;
const TOWER_TURN_SCALE: i16 = 4;
const TOWER_TURN_MAX: i16 = 0x400;
const TOWER_TURN_TOL: i32 = 0x600;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnmPrm {
    pub bck: i8,
    pub next: i8,
    pub morf: f32,
    pub speed: f32,
    pub loop_mode: u8,
}

const fn prm(bck: i8, next: i8, morf: f32, speed: f32, loop_mode: u8) -> AnmPrm {
    AnmPrm {
        bck,
        next,
        morf,
        speed,
        loop_mode,
    }
}

pub const ANM_PRM_TBL: [AnmPrm; 24] = [
    prm(-1, -1, HIO_MORF[0], 0.0, EMODE_NONE),
    prm(BCK_IDX_WAIT01, -1, HIO_MORF[1], 1.0, EMODE_LOOP),
    prm(BCK_IDX_WAIT03, -1, HIO_MORF[2], 1.0, EMODE_LOOP),
    prm(BCK_IDX_WAIT02, -1, HIO_MORF[3], 1.0, EMODE_LOOP),
    prm(BCK_IDX_TALK01, -1, HIO_MORF[4], 1.0, EMODE_LOOP),
    prm(BCK_IDX_WALK01, -1, HIO_MORF[5], 1.0, EMODE_LOOP),
    prm(
        BCK_IDX_JAMP_A,
        ANM_PRM_IDX_JAMP_B,
        HIO_MORF[6],
        1.0,
        EMODE_NONE,
    ),
    prm(BCK_IDX_JAMP_B, -1, HIO_MORF[7], 1.0, EMODE_NONE),
    prm(BCK_IDX_JAMP_C, -1, HIO_MORF[8], 0.0, EMODE_NONE),
    prm(BCK_IDX_JAMP_C, -1, HIO_MORF[9], 1.0, EMODE_NONE),
    prm(BCK_IDX_GUARD, -1, HIO_MORF[10], 1.0, EMODE_LOOP),
    prm(BCK_IDX_JTBT, -1, HIO_MORF[11], 1.0, EMODE_LOOP),
    prm(
        BCK_IDX_HAPPY,
        ANM_PRM_IDX_HAPPY2,
        HIO_MORF[12],
        1.0,
        EMODE_NONE,
    ),
    prm(BCK_IDX_DANCE01, -1, HIO_MORF[13], 1.0, EMODE_NONE),
    prm(BCK_IDX_DANCE02, -1, HIO_MORF[14], 1.0, EMODE_NONE),
    prm(BCK_IDX_GET, -1, HIO_MORF[15], 1.0, EMODE_NONE),
    prm(
        BCK_IDX_HAPPY,
        ANM_PRM_IDX_TALK01,
        HIO_MORF[16],
        1.0,
        EMODE_NONE,
    ),
    prm(
        BCK_IDX_DANCE01,
        ANM_PRM_IDX_TALK01,
        HIO_MORF[17],
        1.0,
        EMODE_NONE,
    ),
    prm(
        BCK_IDX_DANCE02,
        ANM_PRM_IDX_WAIT03,
        HIO_MORF[18],
        1.0,
        EMODE_NONE,
    ),
    prm(BCK_IDX_MAWASU, -1, HIO_MORF[19], 1.0, EMODE_LOOP),
    prm(BCK_IDX_WAIT04, -1, HIO_MORF[20], 1.0, EMODE_LOOP),
    prm(BCK_IDX_TALK02, -1, HIO_MORF[21], 1.0, EMODE_LOOP),
    prm(BCK_IDX_TALK01, -1, HIO_MORF[22], 1.0, EMODE_LOOP),
    prm(BCK_IDX_TALK01, -1, HIO_MORF[23], 1.0, EMODE_LOOP),
];

pub fn bck_frames(bck: i8) -> i16 {
    match bck {
        BCK_IDX_WAIT01 => BCK_WAIT01_FRAMES,
        BCK_IDX_WAIT03 => BCK_WAIT03_FRAMES,
        BCK_IDX_WAIT02 => BCK_WAIT02_FRAMES,
        BCK_IDX_TALK01 => BCK_TALK01_FRAMES,
        BCK_IDX_WALK01 => BCK_WALK01_FRAMES,
        BCK_IDX_JAMP_A => BCK_JAMP_A_FRAMES,
        BCK_IDX_JAMP_B => BCK_JAMP_B_FRAMES,
        BCK_IDX_JAMP_C => BCK_JAMP_C_FRAMES,
        BCK_IDX_GUARD => BCK_GUARD_FRAMES,
        BCK_IDX_JTBT => BCK_JTBT_FRAMES,
        BCK_IDX_HAPPY => BCK_HAPPY_FRAMES,
        BCK_IDX_DANCE01 => BCK_DANCE01_FRAMES,
        BCK_IDX_DANCE02 => BCK_DANCE02_FRAMES,
        BCK_IDX_GET => BCK_GET_FRAMES,
        BCK_IDX_MAWASU => BCK_MAWASU_FRAMES,
        BCK_IDX_WAIT04 => BCK_WAIT04_FRAMES,
        BCK_IDX_TALK02 => BCK_TALK02_FRAMES,
        _ => BCK_WAIT01_FRAMES,
    }
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
}

impl Frame {
    pub const fn idle() -> Self {
        Self {
            attribute: EMODE_LOOP,
            state: 0,
            start: 0,
            end: 0,
            loop_point: 0,
            rate: 1.0,
            frame: 0.0,
        }
    }

    pub fn set_anm(&mut self, loop_mode: u8, rate: f32, end: i16) {
        self.attribute = loop_mode;
        self.state = 0;
        self.start = 0;
        self.end = end;
        self.loop_point = 0;
        self.rate = rate;
        self.frame = 0.0;
    }

    pub fn play(&mut self) {
        self.state = 0;
        self.frame += self.rate;
        if self.attribute == EMODE_NONE {
            if self.frame < self.start as f32 {
                self.frame = self.start as f32;
                self.rate = 0.0;
                self.state |= STATE_STOP;
            }
            if self.frame >= self.end as f32 {
                self.frame = self.end as f32 - 0.001;
                self.rate = 0.0;
                self.state |= STATE_STOP;
            }
        } else {
            while self.frame < self.start as f32 {
                self.state |= STATE_LOOP;
                if self.loop_point as f32 - self.start as f32 <= 0.0 {
                    break;
                }
                self.frame += self.loop_point as f32 - self.start as f32;
            }
            while self.frame >= self.end as f32 {
                self.state |= STATE_LOOP;
                if self.end as f32 - self.loop_point as f32 <= 0.0 {
                    break;
                }
                self.frame -= self.end as f32 - self.loop_point as f32;
            }
        }
    }

    pub fn is_stop(&self) -> bool {
        self.state & STATE_STOP != 0
    }

    pub fn is_loop(&self) -> bool {
        self.state & STATE_LOOP != 0
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct NpcTc {
    pub created: bool,
    pub deleted: bool,
    pub status: i8,
    pub action_status: i8,
    pub ty: u8,
    pub event_idx: i8,
    pub anm_prm_idx: i8,
    pub old_anm_prm_idx: i8,
    pub bck_idx: i8,
    pub tex_pattern_num: i8,
    pub tex_pattern_num_idx: i8,
    pub msg_anime_started: bool,
    pub frame: Frame,
    pub jtbt_timer: u8,
    pub talk01_wait01_timer: u8,
    pub talk01_talk02_timer: u8,
    pub jump_landing_timer: i8,
    pub blink_frame: u8,
    pub blink_timer: i16,
    pub head_lock: bool,
    pub backbone_lock: bool,
    pub jnt_trn: bool,
    pub head_x: i16,
    pub head_y: i16,
    pub backbone_x: i16,
    pub backbone_y: i16,
    pub seed: [i32; 3],
    pub start_angle: [i16; 3],
    pub max_head_turn_velocity: i16,
    pub target_speed: f32,
    pub speed_f: f32,
    pub speed: Vec3,
    pub gravity: f32,
    pub eye_pos: Vec3,
    pub attn_eye: Vec3,
    pub attn_pos: Vec3,
    pub attn_target: Vec3,
    pub stool_look_pos: Vec3,
    pub stool_pos: Vec3,
    pub sit_pos: Vec3,
    pub walk_to_stool_pos: Vec3,
    pub delta_pos: Vec3,
    pub tower_pos: Vec3,
    pub tower_angle: [i16; 3],
    pub tower_stopped: bool,
    pub has_attention: bool,
    pub talking_near_jail: bool,
    pub has_talked_near_jail: bool,
    pub has_entered_sit_radius: bool,
    pub flag_808: bool,
    pub flag_809: bool,
    pub flag_80b: bool,
    pub flag_80c: u8,
    pub flag_80d: bool,
    pub flag_80e: bool,
    pub flag_80f: bool,
    pub flag_810: bool,
    pub flag_811: bool,
    pub flag_812: bool,
    pub flag_813: bool,
    pub flag_814: bool,
    pub cut_act: i8,
    pub cut_timer: i16,
    pub cut_speed: f32,
    pub cut_attention: bool,
    pub cut_no_turn: bool,
    pub cut_turn_speed: i16,
    pub cut_attn_pos: Vec3,
    pub cut_prev_angle: f32,
    pub msg_no: u32,
    pub msg_id: u32,
    pub ordered_event: usize,
    pub do_held: bool,
}

impl NpcTc {
    pub fn spawn(parameters: u32) -> Self {
        let raw = (parameters & TYPE_MASK) as u8;
        let ty = if raw == TYPE_FALLBACK {
            TYPE_NORMAL
        } else {
            raw
        };
        Self {
            created: false,
            deleted: false,
            status: 0,
            action_status: ACTION_STARTING,
            ty,
            event_idx: EVENT_IDX_NONE,
            anm_prm_idx: ANM_PRM_IDX_NONE,
            old_anm_prm_idx: 0,
            bck_idx: BCK_IDX_WAIT01,
            tex_pattern_num: 0,
            tex_pattern_num_idx: 1,
            msg_anime_started: false,
            frame: Frame::idle(),
            jtbt_timer: 0,
            talk01_wait01_timer: 0,
            talk01_talk02_timer: 0,
            jump_landing_timer: 0,
            blink_frame: 0,
            blink_timer: 0,
            head_lock: false,
            backbone_lock: false,
            jnt_trn: false,
            head_x: 0,
            head_y: 0,
            backbone_x: 0,
            backbone_y: 0,
            seed: [RND_SEED; 3],
            start_angle: [0; 3],
            max_head_turn_velocity: 0,
            target_speed: 0.0,
            speed_f: 0.0,
            speed: Vec3::ZERO,
            gravity: 0.0,
            eye_pos: Vec3::ZERO,
            attn_eye: Vec3::ZERO,
            attn_pos: Vec3::ZERO,
            attn_target: Vec3::ZERO,
            stool_look_pos: Vec3::ZERO,
            stool_pos: Vec3::ZERO,
            sit_pos: Vec3::ZERO,
            walk_to_stool_pos: Vec3::ZERO,
            delta_pos: Vec3::ZERO,
            tower_pos: Vec3::ZERO,
            tower_angle: [0; 3],
            tower_stopped: false,
            has_attention: false,
            talking_near_jail: false,
            has_talked_near_jail: false,
            has_entered_sit_radius: false,
            flag_808: false,
            flag_809: false,
            flag_80b: false,
            flag_80c: 0,
            flag_80d: false,
            flag_80e: false,
            flag_80f: false,
            flag_810: false,
            flag_811: false,
            flag_812: false,
            flag_813: false,
            flag_814: false,
            cut_act: NO_CUT,
            cut_timer: 0,
            cut_speed: 0.0,
            cut_attention: false,
            cut_no_turn: false,
            cut_turn_speed: 0,
            cut_attn_pos: Vec3::ZERO,
            cut_prev_angle: 0.0,
            msg_no: 0,
            msg_id: FPC_M_ERROR_PROCESS_ID,
            ordered_event: NO_EVENT,
            do_held: false,
        }
    }

    pub fn is_create(&self, stories: &StoryBits) -> bool {
        match self.ty {
            TYPE_BLUE => stories.test(BIT_1708),
            TYPE_NORMAL2 => stories.test(BIT_0B80),
            _ => true,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
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

fn is_zero(value: f32) -> bool {
    value.abs() < IS_ZERO
}

fn ssin(angle: i16) -> f32 {
    walk::from_s16(angle).sin()
}

fn scos(angle: i16) -> f32 {
    walk::from_s16(angle).cos()
}

fn atan2s(x: f32, z: f32) -> i16 {
    walk::to_s16(x.atan2(z))
}

fn target_angle_y(from: Vec3, to: Vec3) -> i16 {
    atan2s(to.x - from.x, to.z - from.z)
}

fn abs_xz(value: Vec3) -> f32 {
    (value.x * value.x + value.z * value.z).sqrt()
}

fn distance_angle_s(a: i16, b: i16) -> i32 {
    i32::from((a.wrapping_sub(b) as i16).unsigned_abs())
}

fn calc_timer(value: &mut i8) -> i8 {
    if *value != 0 {
        *value -= 1;
    }
    *value
}

fn calc_timer_i16(value: &mut i16) -> i16 {
    if *value != 0 {
        *value -= 1;
    }
    *value
}

fn chase_f(value: &mut f32, target: f32, step: f32) {
    if step != 0.0 {
        let step = if *value > target { -step } else { step };
        *value += step;
        if step * (*value - target) >= 0.0 {
            *value = target;
        }
    }
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

fn add_calc_pos_xz2(value: &mut Vec3, target: Vec3, scale: f32, max_step: f32) {
    if value.x != target.x || value.z != target.z {
        let mut diff = (*value - target) * scale;
        let step = abs_xz(diff);
        if !is_zero(step) {
            if step > max_step {
                diff *= max_step / step;
            }
            value.x -= diff.x;
            value.z -= diff.z;
        }
    }
}

fn add_calc_angle_s2(value: &mut i16, target: i16, scale: i16, max_step: i16) {
    let step = target.wrapping_sub(*value) / scale;
    if step > max_step {
        *value = value.wrapping_add(max_step);
    } else if step < -max_step {
        *value = value.wrapping_sub(max_step);
    } else {
        *value = value.wrapping_add(step);
    }
}

fn add_calc_angle_l(value: i16, target: i16, scale: i32, max_step: i32, min_step: i32) -> i16 {
    let mut value = i32::from(value);
    let target = i32::from(target);
    let diff = target - value;
    if value != target {
        let mut step = diff / scale;
        if step > min_step || step < -min_step {
            if step > max_step {
                step = max_step;
            }
            if step < -max_step {
                step = -max_step;
            }
            value += step;
        } else if diff >= 0 {
            value += min_step;
            if target - value <= 0 {
                value = target;
            }
        } else {
            value -= min_step;
            if target - value >= 0 {
                value = target;
            }
        }
    }
    value as i16
}

fn add_calc_angle_s(value: &mut i16, target: i16, scale: i16, max_step: i16, min_step: i16) {
    let mut diff = target.wrapping_sub(*value);
    if *value != target {
        let mut step = diff / scale;
        if step > min_step || step < -min_step {
            if step > max_step {
                step = max_step;
            }
            if step < -max_step {
                step = -max_step;
            }
            *value = value.wrapping_add(step);
        } else if diff >= 0 {
            *value = value.wrapping_add(min_step);
            diff = target.wrapping_sub(*value);
            if diff <= 0 {
                *value = target;
            }
        } else {
            *value = value.wrapping_sub(min_step);
            diff = target.wrapping_sub(*value);
            if diff >= 0 {
                *value = target;
            }
        }
    }
}

fn ang_calc_s(out: &mut i16, target_y: i16, speed: i16, max_vel: i16) {
    let orig_diff = out.wrapping_sub(target_y);
    let mut diff = orig_diff;
    add_calc_angle_s(&mut diff, 0, speed, max_vel, JNT_ANG_STEP);
    if i32::from(diff.unsigned_abs()) > i32::from(speed) {
        *out = out.wrapping_add(diff.wrapping_sub(orig_diff));
    } else {
        *out = target_y;
    }
}

fn limitter(value: i16, max_diff: i32, min_diff: i32) -> i16 {
    let mut value = i32::from(value);
    if value > max_diff {
        value = max_diff;
    }
    if value < min_diff {
        value = min_diff;
    }
    value as i16
}

fn rotate_y(value: Vec3, angle: i16) -> Vec3 {
    let rad = walk::from_s16(angle);
    let (s, c) = (rad.sin(), rad.cos());
    Vec3::new(
        value.x * c + value.z * s,
        value.y,
        -value.x * s + value.z * c,
    )
}

fn rotate_x(value: Vec3, angle: i16) -> Vec3 {
    let rad = walk::from_s16(angle);
    let (s, c) = (rad.sin(), rad.cos());
    Vec3::new(
        value.x,
        value.y * c - value.z * s,
        value.y * s + value.z * c,
    )
}

fn rotate_z(value: Vec3, angle: i16) -> Vec3 {
    let rad = walk::from_s16(angle);
    let (s, c) = (rad.sin(), rad.cos());
    Vec3::new(
        value.x * c - value.y * s,
        value.x * s + value.y * c,
        value.z,
    )
}

fn rotate_zxy(value: Vec3, angle: [i16; 3]) -> Vec3 {
    let out = rotate_z(value, angle[2]);
    let out = rotate_x(out, angle[0]);
    rotate_y(out, angle[1])
}

fn is_stage_tbox(chests: &Chests, stage: u8, no: u8) -> bool {
    match stage {
        STAGE_DRC => chests.is_open(0, no),
        STAGE_FW => chests.is_open(SEA2_STAGE, no),
        _ => false,
    }
}

fn get_collect_map(collect: &Collect, no: u8) -> bool {
    if no == 0 || no > COLL_MAP_COUNT {
        return false;
    }
    collect.check(no - 1) != 0
}

fn is_collect_map_triforce(collect: &Collect, no: u8) -> bool {
    if no == 0 || no > COLL_MAP_MAX {
        return false;
    }
    collect.is_triforce(no - 1)
}

fn on_collect_map_triforce(collect: &mut Collect, no: u8) {
    if no >= 1 && no <= COLL_MAP_MAX {
        collect.on_triforce(no - 1);
    }
}

pub struct Text<'a> {
    pub procs: &'a mut MsgProcs,
    pub info: &'a mut MsgGameInfo,
    pub one: &'a mut Msg1Box,
    pub two: &'a mut Msg2Box,
    pub clear_count: u8,
    pub archive: Option<&'a dyn MsgArchive>,
}

impl Text<'_> {
    fn proc_name(&self) -> Option<u16> {
        if self.procs.search_by_name(FPC_NM_MSG).is_some() {
            Some(FPC_NM_MSG)
        } else if self.procs.search_by_name(FPC_NM_MSG2).is_some() {
            Some(FPC_NM_MSG2)
        } else {
            None
        }
    }

    pub fn status(&self) -> Option<u8> {
        match self.proc_name() {
            Some(FPC_NM_MSG) => Some(self.one.status),
            Some(FPC_NM_MSG2) => Some(self.two.status),
            _ => None,
        }
    }

    pub fn set_status(&mut self, status: u8) {
        match self.proc_name() {
            Some(FPC_NM_MSG) => self.one.status = status,
            Some(FPC_NM_MSG2) => self.two.status = status,
            _ => {}
        }
    }

    pub fn select_num(&self) -> u8 {
        match self.proc_name() {
            Some(FPC_NM_MSG) => self.one.select_num,
            _ => 0,
        }
    }

    pub fn message_set(&mut self, msg_no: u32, look_at: Vec3) -> u32 {
        let Some(archive) = self.archive else {
            return FPC_M_ERROR_PROCESS_ID;
        };
        fop_msgm_message_set(
            self.procs,
            self.info,
            archive,
            self.clear_count,
            msg_no,
            None,
            look_at,
        )
    }
}

pub struct Ctx<'a, 'b> {
    pub entity: Entity,
    pub player: Option<Vec3>,
    pub player_eye: Vec3,
    pub talk_request: bool,
    pub stool: Option<Vec3>,
    pub towers: &'b [(Vec3, [i16; 3])],
    pub solid: Option<&'b Collision>,
    pub events: &'a mut SimEvents,
    pub stories: &'a mut StoryBits,
    pub flags: &'a mut EventFlags,
    pub collect: &'a mut Collect,
    pub rupees: &'a mut Rupees,
    pub chests: &'a Chests,
    pub wallet: u8,
    pub list: &'a EventList,
    pub active: &'a mut ActiveEvent,
    pub text: &'a mut Text<'a>,
    pub give_item: Option<(Vec3, u8)>,
}

fn cut_number_or(ctx: &Ctx<'_, '_>, cut: usize, key: &str, fallback: i16) -> i16 {
    ctx.list
        .cut_number(cut, key)
        .map_or(fallback, |value| value as i16)
}

impl NpcTc {
    fn rnd_f(&mut self, range: f32) -> f32 {
        rnd(&mut self.seed) * range
    }

    fn init_tex_pattern_anm(&mut self) {
        self.blink_frame = 0;
        self.blink_timer = 0;
    }

    pub fn play_tex_pattern_anm(&mut self) {
        if calc_timer_i16(&mut self.blink_timer) == 0 {
            if self.blink_frame >= BTP_FRAME_MAX {
                self.blink_frame = self.blink_frame.wrapping_sub(BTP_FRAME_MAX);
                self.blink_timer = (self.rnd_f(BTP_RANDOM) + BTP_DELAY) as i16;
            } else {
                self.blink_frame = self.blink_frame.wrapping_add(1);
            }
        }
    }

    pub fn set_tex_anm(&mut self) {
        let idx = if self.tex_pattern_num_idx >= 0 {
            self.tex_pattern_num_idx as usize
        } else {
            usize::MAX
        };
        let tex_pattern_num = TEX_PATTERN_NUM_TBL.get(idx).copied().unwrap_or(-1);
        if tex_pattern_num != self.tex_pattern_num && tex_pattern_num != -1 {
            self.tex_pattern_num = tex_pattern_num;
            self.init_tex_pattern_anm();
        }
    }
}

impl NpcTc {
    pub fn set_anm(&mut self, ctx: &mut Ctx<'_, '_>) {
        if self.anm_prm_idx == ANM_PRM_IDX_TALK01_WAIT01 {
            if self.frame.is_loop() {
                self.talk01_wait01_timer = self.talk01_wait01_timer.wrapping_add(1);
                if self.talk01_wait01_timer >= 3 {
                    self.talk01_wait01_timer = 0;
                    self.anm_prm_idx = ANM_PRM_IDX_WAIT01;
                }
            }
        } else {
            self.talk01_wait01_timer = 0;
        }

        if self.anm_prm_idx == ANM_PRM_IDX_TALK01_TALK02 {
            if self.frame.is_loop() {
                self.talk01_talk02_timer = self.talk01_talk02_timer.wrapping_add(1);
                if self.talk01_talk02_timer >= 5 {
                    self.talk01_talk02_timer = 0;
                    self.anm_prm_idx = ANM_PRM_IDX_TALK02;
                }
            }
        } else {
            self.talk01_talk02_timer = 0;
        }

        if self.anm_prm_idx == ANM_PRM_IDX_JTBT {
            if self.frame.is_loop() {
                self.jtbt_timer = self.jtbt_timer.wrapping_add(1);
                if self.jtbt_timer >= 4 {
                    self.jtbt_timer = 0;
                    self.anm_prm_idx = ANM_PRM_IDX_TALK01;
                }
            }
        } else {
            self.jtbt_timer = 0;
        }

        if self.bck_idx == BCK_IDX_MAWASU && self.frame.frame as i32 == 1 {
            if self.ty == TYPE_RED {
                ctx.events.list.push(SimEvent::Se(SE_TC_B_PUSH_BAR));
            } else if self.ty == TYPE_WHITE {
                ctx.events.list.push(SimEvent::Se(SE_TC_T_PUSH_BAR));
            }
        }

        self.dlib_bcks_set_anm();
    }

    fn dlib_bcks_set_anm(&mut self) {
        let at = if self.anm_prm_idx >= 0 {
            self.anm_prm_idx as usize
        } else {
            usize::MAX
        };
        let Some(entry) = ANM_PRM_TBL.get(at).copied() else {
            return;
        };
        if self.old_anm_prm_idx != self.anm_prm_idx && entry.bck != -1 {
            self.bck_idx = entry.bck;
            self.frame
                .set_anm(entry.loop_mode, entry.speed, bck_frames(entry.bck));
        }
        self.old_anm_prm_idx = self.anm_prm_idx;
        if self.frame.is_stop() && entry.next != -1 && entry.loop_mode == EMODE_NONE {
            self.anm_prm_idx = entry.next;
        }
    }
}

impl NpcTc {
    pub fn chk_attention(&self, pos: Vec3, adjustment: i16, player: Option<Vec3>) -> bool {
        let Some(player) = player else {
            return false;
        };
        let mut max_attn_dist = HIO_MAX_ATTN_DIST_XZ;
        if self.ty == TYPE_RED || self.ty == TYPE_WHITE {
            max_attn_dist = HIO_MAX_ATTN_DIST_XZ;
            max_attn_dist *= 0.5;
        }
        if self.status == STATUS_WALK_TO_JAIL
            || self.status == STATUS_JUMP
            || self.status == STATUS_SIT
        {
            max_attn_dist = HIO_FIELD_2C;
        }
        let mut max_attn_angle = HIO_MAX_ATTN_ANGLE_Y;
        let dx = player.x - pos.x;
        let dz = player.z - pos.z;
        let dist = (dx * dx + dz * dz).sqrt();
        let mut angle = atan2s(dx, dz);
        if self.has_attention {
            max_attn_dist += ATTENTION_BONUS_DIST;
            max_attn_angle += ATTENTION_BONUS_ANGLE;
        }
        angle = angle.wrapping_sub(adjustment);
        max_attn_angle > i32::from(angle.unsigned_abs()) && max_attn_dist > dist
    }

    pub fn event_order(&mut self, ctx: &mut Ctx<'_, '_>) {
        if self.event_idx == EVENT_IDX_SPEAK || self.event_idx == EVENT_IDX_TALK {
            return;
        }
        if self.event_idx >= EVENT_IDX_FIRST {
            let slot = self.event_idx - EVENT_IDX_FIRST;
            let at = if slot >= 0 { slot as usize } else { usize::MAX };
            let Some(name) = EVENT_NAMES.get(at).copied() else {
                return;
            };
            let Some(index) = ctx.list.event_index(name) else {
                return;
            };
            ctx.active.order(ctx.list, index, ctx.entity);
            self.ordered_event = index;
        }
    }

    pub fn check_order(&mut self, ctx: &Ctx<'_, '_>) {
        if self.ordered_event != NO_EVENT && ctx.active.start_check(self.ordered_event) {
            self.event_idx = EVENT_IDX_NONE;
            self.ordered_event = NO_EVENT;
            return;
        }
        if !ctx.talk_request {
            return;
        }
        if self.event_idx != EVENT_IDX_SPEAK && self.event_idx != EVENT_IDX_TALK {
            return;
        }
        self.event_idx = EVENT_IDX_NONE;
        self.talking_near_jail = true;
    }

    pub fn exist_tc_monument(&self, ctx: &Ctx<'_, '_>) -> bool {
        is_stage_tbox(ctx.chests, STAGE_DRC, MONUMENT_TBOX)
            || is_stage_tbox(ctx.chests, STAGE_FW, MONUMENT_TBOX)
            || is_stage_tbox(ctx.chests, STAGE_TOTG, MONUMENT_TBOX)
            || is_stage_tbox(ctx.chests, STAGE_WT, MONUMENT_TBOX)
            || is_stage_tbox(ctx.chests, STAGE_ET, MONUMENT_TBOX)
    }

    pub fn anm_atr(&mut self, status: u8, attr: u8) {
        match status {
            STATUS_TYPING => {
                if self.msg_anime_started {
                    return;
                }
                self.msg_anime_started = true;
                let at = usize::from(attr);
                let target = ANM_ATR.get(at).copied().unwrap_or(ANM_PRM_IDX_NONE);
                if target == ANM_PRM_IDX_HAPPY2 && self.anm_prm_idx == ANM_PRM_IDX_TALK01 {
                    return;
                }
                self.anm_prm_idx = target;
            }
            STATUS_MSG_DISPLAYED => {
                self.msg_anime_started = false;
            }
            _ => {}
        }
    }

    pub fn stop_tower(&mut self) {
        match self.ty {
            TYPE_RED | TYPE_WHITE => {
                self.head_lock = false;
                self.anm_prm_idx = ANM_PRM_IDX_WAIT04;
                self.tower_stopped = true;
            }
            TYPE_NORMAL2 => {
                self.tower_stopped = true;
            }
            _ => {}
        }
    }

    pub fn start_tower(&mut self, actor: &mut Actor) {
        let temp2 = match self.ty {
            TYPE_WHITE => HIO_FIELD_7A,
            _ => HIO_FIELD_78,
        };
        match self.ty {
            TYPE_RED | TYPE_WHITE => {
                self.head_lock = true;
                let target = self.tower_angle[1].wrapping_add(temp2);
                add_calc_angle_s2(
                    &mut actor.angle[1],
                    target,
                    TOWER_TURN_SCALE,
                    TOWER_TURN_MAX,
                );
                if distance_angle_s(actor.angle[1], target) < TOWER_TURN_TOL {
                    self.tower_stopped = false;
                    self.anm_prm_idx = ANM_PRM_IDX_MAWASU;
                }
            }
            _ => {
                self.tower_stopped = false;
            }
        }
    }

    pub fn set_tower(&mut self, actor: &mut Actor, ctx: &Ctx<'_, '_>) {
        let Some((pos, angle)) = ctx.towers.first().copied() else {
            return;
        };
        self.tower_pos = pos;
        self.tower_angle = angle;
        let (temp, temp2) = match self.ty {
            TYPE_WHITE => (HIO_FIELD_60, HIO_FIELD_7A),
            _ => (HIO_FIELD_6C, HIO_FIELD_78),
        };
        if self.ty == TYPE_RED || self.ty == TYPE_WHITE {
            actor.pos = pos + rotate_zxy(temp, angle);
            let temp3 = angle[1].wrapping_add(temp2);
            if self.tower_stopped {
                if self.anm_prm_idx == ANM_PRM_IDX_MAWASU || self.anm_prm_idx == ANM_PRM_IDX_WAIT04
                {
                    add_calc_angle_s2(&mut actor.angle[1], temp3, 4, 0x800);
                }
            } else {
                actor.angle[1] = temp3;
            }
        }
    }
}

impl NpcTc {
    pub fn next_msg_status(&mut self, ctx: &mut Ctx<'_, '_>) -> u8 {
        match self.ty {
            TYPE_NORMAL => self.next_msg_status_normal(),
            TYPE_NORMAL2 => self.next_msg_status_normal2(ctx),
            TYPE_BLUE => self.next_msg_status_blue(ctx),
            TYPE_RED => self.next_msg_status_red(ctx),
            TYPE_WHITE => self.next_msg_status_white(ctx),
            _ => STATUS_MSG_ENDS,
        }
    }

    fn dance_hold(&self) -> bool {
        (self.anm_prm_idx == ANM_PRM_IDX_DANCE01_TO_TALK01
            || self.anm_prm_idx == ANM_PRM_IDX_DANCE02_TO_WAIT03)
            && !self.frame.is_stop()
    }

    pub fn next_msg_status_blue(&mut self, ctx: &mut Ctx<'_, '_>) -> u8 {
        let mut status = STATUS_MSG_CONTINUES;
        if self.dance_hold() {
            status = STATUS_MSG_DISPLAYED;
        } else {
            match self.msg_no {
                MSG_DDD => {
                    if self.exist_tc_monument(ctx) {
                        self.msg_no = MSG_DDE;
                    } else {
                        self.msg_no = MSG_DF9;
                    }
                }
                MSG_DDE => self.msg_no = MSG_DDF,
                MSG_DDF => self.msg_no = MSG_DE0,
                MSG_DF9 => self.msg_no = MSG_DFA,
                _ => status = STATUS_MSG_ENDS,
            }
        }
        status
    }

    pub fn next_msg_status_normal(&mut self) -> u8 {
        let mut status = STATUS_MSG_CONTINUES;
        if self.dance_hold() {
            status = STATUS_MSG_DISPLAYED;
        } else {
            match self.msg_no {
                MSG_NORMAL_10CD | MSG_NORMAL_10DF => self.msg_no = MSG_NORMAL_10CE,
                MSG_NORMAL_10CE => self.msg_no = MSG_NORMAL_10CF,
                MSG_NORMAL_10CF => self.msg_no = MSG_NORMAL_10D0,
                MSG_NORMAL_10D0 => self.msg_no = MSG_NORMAL_10D1,
                MSG_NORMAL_10D1 => self.msg_no = MSG_NORMAL_10D2,
                MSG_NORMAL_FIRST => self.msg_no = MSG_NORMAL_10D2,
                MSG_NORMAL_10D4 => self.msg_no = MSG_NORMAL_10D5,
                MSG_NORMAL_10D5 => self.msg_no = MSG_NORMAL_10D6,
                MSG_NORMAL_10D6 => self.msg_no = MSG_NORMAL_10D7,
                MSG_NORMAL_10D7 => self.msg_no = MSG_NORMAL_10D8,
                MSG_NORMAL_SECOND => self.msg_no = MSG_NORMAL_10D8,
                MSG_NORMAL_10DA => self.msg_no = MSG_NORMAL_10DB,
                MSG_NORMAL_10DB => self.msg_no = MSG_NORMAL_10DC,
                MSG_NORMAL_10DD => self.msg_no = MSG_NORMAL_10DE,
                MSG_DAD => self.msg_no = MSG_DAE,
                MSG_DAF => self.msg_no = MSG_DB0,
                _ => status = STATUS_MSG_ENDS,
            }
        }
        status
    }

    pub fn next_msg_status_normal2(&mut self, ctx: &mut Ctx<'_, '_>) -> u8 {
        let mut status = STATUS_MSG_CONTINUES;
        if self.dance_hold() {
            status = STATUS_MSG_DISPLAYED;
        } else {
            match self.msg_no {
                MSG_38AA | MSG_38AB => {
                    if ctx.stories.test(BIT_3020) {
                        self.msg_no = MSG_38AC;
                    } else {
                        self.msg_no = MSG_38AD;
                    }
                }
                MSG_38AC | MSG_38AD => {
                    if self.exist_known_collect_map(ctx) {
                        self.msg_no = MSG_DBA;
                    } else {
                        self.msg_no = MSG_DB5;
                    }
                }
                MSG_DB2 | MSG_DB3 | MSG_DFB => self.msg_no = MSG_DB4,
                MSG_DB4 => {
                    let next = self.get_msg_normal2(ctx);
                    self.msg_no = next;
                }
                MSG_DB5 => self.msg_no = MSG_DBA,
                MSG_DBB => self.msg_no = MSG_DBC,
                MSG_DBC => self.msg_no = MSG_DBD,
                MSG_DBD => self.msg_no = MSG_DBE,
                MSG_DBE => {
                    if ctx.text.select_num() == 0 {
                        if ctx.rupees.count >= PAY_RUPEE_COST {
                            self.status = STATUS_PAY_RUPEE;
                            self.flag_80d = true;
                            self.flag_80c = 1;
                            status = STATUS_MSG_ENDS;
                        } else {
                            self.msg_no = MSG_DC0;
                        }
                    } else {
                        self.msg_no = MSG_DBF;
                    }
                }
                MSG_DC0 => {
                    if self.flag_80b {
                        self.msg_no = MSG_DC4;
                    } else {
                        self.msg_no = MSG_DC1;
                    }
                }
                MSG_DBF => self.msg_no = MSG_DC1,
                MSG_DC3 => {
                    if ctx.text.select_num() == 0 {
                        if ctx.rupees.count >= PAY_RUPEE_COST {
                            self.status = STATUS_PAY_RUPEE;
                            self.flag_80d = true;
                            self.flag_80c = 1;
                            status = STATUS_MSG_ENDS;
                        } else {
                            self.flag_80b = true;
                            self.msg_no = MSG_DC0;
                        }
                    } else {
                        self.msg_no = MSG_DC4;
                    }
                }
                MSG_DC4 => {
                    self.flag_80d = false;
                    self.flag_80b = false;
                    status = STATUS_MSG_ENDS;
                }
                _ => status = STATUS_MSG_ENDS,
            }
        }
        status
    }

    pub fn next_msg_status_white(&mut self, ctx: &mut Ctx<'_, '_>) -> u8 {
        self.head_lock = false;
        let mut status = STATUS_MSG_CONTINUES;
        if self.dance_hold() {
            status = STATUS_MSG_DISPLAYED;
        } else {
            match self.msg_no {
                MSG_DC5 => self.msg_no = MSG_DC6,
                MSG_DC6 => {
                    let next = self.set_finish_msg();
                    self.msg_no = next;
                }
                MSG_DC8 => {
                    if ctx.stories.test(BIT_0B80) {
                        if !self.flag_80e {
                            self.flag_80e = true;
                            self.msg_no = MSG_DD7;
                        }
                    } else if !self.flag_80f {
                        self.flag_80f = true;
                        self.msg_no = MSG_DC9;
                    }
                }
                MSG_DD7 => self.msg_no = MSG_DD8,
                MSG_DD8 => {
                    let next = self.set_finish_msg();
                    self.msg_no = next;
                }
                MSG_DC9 => {
                    if ctx.text.select_num() == 0 {
                        self.msg_no = MSG_DCA;
                    } else {
                        self.msg_no = MSG_DD3;
                    }
                }
                MSG_DCA => self.msg_no = MSG_DCB,
                MSG_DCB => self.msg_no = MSG_DCC,
                MSG_DCC => self.msg_no = MSG_DCD,
                MSG_DCD => self.msg_no = MSG_DCE,
                MSG_DCE => self.msg_no = MSG_DCF,
                MSG_DCF => self.msg_no = MSG_DD0,
                MSG_DD0 => self.msg_no = MSG_DD1,
                MSG_DD1 => self.msg_no = MSG_DD2,
                MSG_DD2 => {
                    let next = self.set_finish_msg();
                    self.msg_no = next;
                }
                MSG_DD3 => self.msg_no = MSG_DD4,
                MSG_DD4 => self.msg_no = MSG_DD5,
                MSG_DD5 => {
                    let next = self.set_finish_msg();
                    self.msg_no = next;
                }
                MSG_DC7 => {
                    self.head_lock = true;
                    status = STATUS_MSG_ENDS;
                }
                MSG_DD9 => self.msg_no = MSG_DDA,
                MSG_DDA => {
                    if self.exist_tc_monument(ctx) {
                        let next = self.set_finish_msg();
                        self.msg_no = next;
                    } else {
                        self.msg_no = MSG_DDB;
                    }
                }
                MSG_DDB => self.msg_no = MSG_DDC,
                MSG_DDC => {
                    let next = self.set_finish_msg();
                    self.msg_no = next;
                }
                _ => status = STATUS_MSG_ENDS,
            }
        }
        status
    }

    pub fn next_msg_status_red(&mut self, ctx: &mut Ctx<'_, '_>) -> u8 {
        let mut status = STATUS_MSG_CONTINUES;
        if self.dance_hold() {
            status = STATUS_MSG_DISPLAYED;
        } else {
            match self.msg_no {
                MSG_DB7 | MSG_DE1 => self.msg_no = MSG_DE2,
                MSG_DE2 => {
                    if ctx.text.clear_count != 0 {
                        self.msg_no = MSG_DB8;
                    } else {
                        self.msg_no = MSG_DE3;
                    }
                }
                MSG_DB8 | MSG_DE3 => self.msg_no = MSG_DE4,
                MSG_DE4 => {
                    let next = self.set_finish_msg();
                    self.msg_no = next;
                }
                MSG_DE5 => {
                    let next = self.set_finish_msg();
                    self.msg_no = next;
                }
                MSG_DE6 => {
                    if ctx.stories.test(BIT_1A20) {
                        self.msg_no = MSG_DEB;
                    } else {
                        self.msg_no = MSG_DE7;
                    }
                    self.flag_814 = true;
                }
                MSG_DEB => {
                    if self.flag_814 {
                        if ctx.stories.test(BIT_1A20) {
                            self.msg_no = MSG_DEC;
                        } else {
                            let next = self.set_finish_msg();
                            self.msg_no = next;
                        }
                    } else {
                        let next = self.set_finish_msg();
                        self.msg_no = next;
                    }
                }
                MSG_DEC => self.msg_no = MSG_DED,
                MSG_DED => {
                    let next = self.set_finish_msg();
                    self.msg_no = next;
                }
                MSG_DE7 => self.msg_no = MSG_DE8,
                MSG_DE8 => self.msg_no = MSG_DE9,
                MSG_DE9 => self.msg_no = MSG_DEA,
                MSG_DEA => {
                    let next = self.set_finish_msg();
                    self.msg_no = next;
                }
                MSG_DEE => self.msg_no = MSG_DEF,
                MSG_DEF => {
                    self.status = STATUS_GET_RUPEE;
                    status = STATUS_MSG_ENDS;
                }
                MSG_DF1 => self.msg_no = MSG_DF2,
                MSG_DF2 => {
                    self.status = STATUS_MONUMENT_COMPLETE;
                    status = STATUS_MSG_ENDS;
                }
                MSG_DF3 => self.msg_no = MSG_DF4,
                MSG_DF4 => self.msg_no = MSG_DF5,
                MSG_DF5 => {
                    let next = self.set_finish_msg();
                    self.msg_no = next;
                }
                MSG_DF0 => {
                    let next = self.set_finish_msg();
                    self.msg_no = next;
                }
                MSG_DC7 => {
                    self.head_lock = true;
                    status = STATUS_MSG_ENDS;
                    self.flag_814 = false;
                }
                _ => status = STATUS_MSG_ENDS,
            }
        }
        status
    }

    pub fn set_finish_msg(&mut self) -> u32 {
        self.head_lock = true;
        MSG_DC7
    }

    fn set_first_msg(flag: &mut bool, first: u32, second: u32) -> u32 {
        if !*flag {
            *flag = true;
            first
        } else {
            second
        }
    }

    pub fn get_msg_normal(&mut self, ctx: &mut Ctx<'_, '_>) -> u32 {
        if !ctx.stories.test(BIT_0B40) {
            self.has_talked_near_jail = true;
            MSG_NORMAL_FIRST
        } else {
            MSG_NORMAL_SECOND
        }
    }

    pub fn get_msg(&mut self, ctx: &mut Ctx<'_, '_>) -> u32 {
        match self.ty {
            TYPE_NORMAL => self.get_msg_normal(ctx),
            TYPE_NORMAL2 => self.get_msg_normal2(ctx),
            TYPE_BLUE => MSG_DDD,
            TYPE_RED => self.get_msg_red(ctx),
            TYPE_WHITE => self.get_msg_white(ctx),
            _ => 0,
        }
    }

    pub fn get_msg_white(&mut self, ctx: &mut Ctx<'_, '_>) -> u32 {
        self.stop_tower();
        if ctx.stories.test(BIT_1708) {
            self.head_lock = true;
            MSG_DD9
        } else if ctx.stories.test(BIT_0B80) {
            if !self.flag_80e {
                self.head_lock = true;
                MSG_DC8
            } else {
                MSG_DD6
            }
        } else if !self.flag_80f {
            self.head_lock = true;
            MSG_DC8
        } else {
            self.head_lock = true;
            MSG_DC5
        }
    }

    pub fn get_msg_red(&mut self, ctx: &mut Ctx<'_, '_>) -> u32 {
        self.stop_tower();
        if self.flag_813 {
            self.flag_813 = false;
            return MSG_DF3;
        }
        if self.flag_812 {
            self.flag_812 = false;
            return if self.check_all_monument_fee(ctx) {
                MSG_DF1
            } else {
                MSG_DF0
            };
        }
        if ctx.stories.test(BIT_0B80) {
            if !self.flag_810 {
                self.flag_810 = true;
                if is_stage_tbox(ctx.chests, STAGE_DRC, MONUMENT_TBOX)
                    && !ctx.stories.test(BIT_1240)
                {
                    return MSG_DEE;
                }
                if is_stage_tbox(ctx.chests, STAGE_FW, MONUMENT_TBOX) && !ctx.stories.test(BIT_1D08)
                {
                    return MSG_DEE;
                }
                if is_stage_tbox(ctx.chests, STAGE_TOTG, MONUMENT_TBOX)
                    && !ctx.stories.test(BIT_1D04)
                {
                    return MSG_DEE;
                }
                if is_stage_tbox(ctx.chests, STAGE_WT, MONUMENT_TBOX) && !ctx.stories.test(BIT_1D02)
                {
                    return MSG_DEE;
                }
                if is_stage_tbox(ctx.chests, STAGE_ET, MONUMENT_TBOX) && !ctx.stories.test(BIT_1D01)
                {
                    return MSG_DEE;
                }
                return MSG_DE6;
            }
            return if ctx.stories.test(BIT_1A20) {
                MSG_DEB
            } else {
                MSG_DEA
            };
        }
        if ctx.text.clear_count != 0 {
            return Self::set_first_msg(&mut self.flag_811, MSG_DB7, MSG_DE5);
        }
        Self::set_first_msg(&mut self.flag_811, MSG_DE1, MSG_DE5)
    }

    pub fn get_msg_normal2(&mut self, ctx: &mut Ctx<'_, '_>) -> u32 {
        self.stop_tower();
        if self.flag_80d {
            if self.exist_unknown_collect_map(ctx) {
                self.flag_80d = false;
                MSG_DC3
            } else {
                MSG_DC4
            }
        } else if !self.flag_808 {
            self.flag_808 = true;
            if ctx.stories.test(BIT_1A20) {
                if ctx.stories.test(BIT_1708) {
                    MSG_DB3
                } else {
                    MSG_DFB
                }
            } else {
                MSG_DB2
            }
        } else if self.exist_unknown_collect_map(ctx) {
            if !self.flag_809 {
                self.flag_809 = true;
                MSG_DBB
            } else {
                MSG_DBE
            }
        } else {
            self.set_rupee_size_msg(ctx)
        }
    }

    pub fn set_rupee_size_msg(&mut self, ctx: &mut Ctx<'_, '_>) -> u32 {
        let mut msg_no = MSG_DB5;
        if ctx.wallet == 0 {
            msg_no = MSG_38AA;
        } else if ctx.wallet == 1 {
            msg_no = MSG_38AB;
        } else if ctx.wallet == 2 {
            if self.exist_known_collect_map(ctx) {
                msg_no = MSG_DBA;
            } else {
                msg_no = MSG_DB5;
            }
        }
        msg_no
    }

    pub fn exist_unknown_collect_map(&self, ctx: &Ctx<'_, '_>) -> bool {
        let mut no: u8 = 1;
        while no <= COLL_MAP_COUNT {
            if !is_collect_map_triforce(ctx.collect, no) && get_collect_map(ctx.collect, no) {
                return true;
            }
            no += 1;
        }
        false
    }

    pub fn analysis_collect_map(&self, ctx: &mut Ctx<'_, '_>) -> i32 {
        let mut no: u8 = 1;
        while no <= COLL_MAP_COUNT {
            if !is_collect_map_triforce(ctx.collect, no) && get_collect_map(ctx.collect, no) {
                on_collect_map_triforce(ctx.collect, no);
                return i32::from(no);
            }
            no += 1;
        }
        0
    }

    pub fn exist_known_collect_map(&self, ctx: &Ctx<'_, '_>) -> bool {
        let mut no: u8 = 1;
        while no <= COLL_MAP_COUNT {
            if get_collect_map(ctx.collect, no) && is_collect_map_triforce(ctx.collect, no) {
                return true;
            }
            no += 1;
        }
        false
    }

    pub fn check_monument_fee(&self, ctx: &Ctx<'_, '_>, stage: u8, event_id: u16) -> bool {
        if is_stage_tbox(ctx.chests, stage, MONUMENT_TBOX) {
            return ctx.stories.test(event_id);
        }
        false
    }

    pub fn check_all_monument_fee(&self, ctx: &Ctx<'_, '_>) -> bool {
        self.check_monument_fee(ctx, STAGE_DRC, BIT_1240)
            && self.check_monument_fee(ctx, STAGE_FW, BIT_1D08)
            && self.check_monument_fee(ctx, STAGE_TOTG, BIT_1D04)
            && self.check_monument_fee(ctx, STAGE_WT, BIT_1D02)
            && self.check_monument_fee(ctx, STAGE_ET, BIT_1D01)
    }

    pub fn check_monument_pay(&self, ctx: &mut Ctx<'_, '_>, stage: u8, event_id: u16) -> bool {
        if ctx.stories.test(event_id) {
            return false;
        }
        if is_stage_tbox(ctx.chests, stage, MONUMENT_TBOX) {
            ctx.stories.set(event_id);
            return true;
        }
        false
    }

    pub fn check_all_monument_pay(&self, ctx: &mut Ctx<'_, '_>) -> i32 {
        let mut paid = 0;
        if self.check_monument_pay(ctx, STAGE_DRC, BIT_1240) {
            paid += 1;
        }
        if self.check_monument_pay(ctx, STAGE_FW, BIT_1D08) {
            paid += 1;
        }
        if self.check_monument_pay(ctx, STAGE_TOTG, BIT_1D04) {
            paid += 1;
        }
        if self.check_monument_pay(ctx, STAGE_WT, BIT_1D02) {
            paid += 1;
        }
        if self.check_monument_pay(ctx, STAGE_ET, BIT_1D01) {
            paid += 1;
        }
        paid
    }

    pub fn talk(&mut self, ctx: &mut Ctx<'_, '_>) -> u8 {
        let mut status = NO_STATUS;
        if self.msg_id == FPC_M_ERROR_PROCESS_ID {
            let msg_no = self.get_msg(ctx);
            self.msg_no = msg_no;
            let eye = self.eye_pos;
            self.msg_id = ctx.text.message_set(self.msg_no, eye);
        } else if ctx.text.procs.search_by_id(self.msg_id).is_some() {
            status = ctx.text.status().unwrap_or(NO_STATUS);
            if status == STATUS_MSG_DISPLAYED {
                let next = self.next_msg_status(ctx);
                ctx.text.set_status(next);
                if next == STATUS_MSG_CONTINUES {
                    let msg_no = self.msg_no;
                    let eye = self.eye_pos;
                    ctx.text.message_set(msg_no, eye);
                }
            } else if status == STATUS_BOX_CLOSED {
                ctx.text.set_status(STATUS_MSG_DESTROYED);
                self.msg_id = FPC_M_ERROR_PROCESS_ID;
            }
            let attr = ctx.text.info.mesg_anime;
            self.anm_atr(status, attr);
        }
        status
    }

    fn event_reset(&mut self) {}
}

impl NpcTc {
    pub fn calc_move(&mut self) {
        if self.anm_prm_idx != ANM_PRM_IDX_WALK01 {
            return;
        }
        chase_f(&mut self.speed_f, self.target_speed, CHASE_STEP);
        let mut speed;
        if self.cut_speed != 0.0 {
            speed = 0.0;
            match self.cut_act {
                CUT_MOVE_TO_ACTOR | CUT_MOVE_TO_POS => {
                    speed = HIO_FIELD_54 * self.cut_speed;
                }
                _ => {}
            }
        } else {
            speed = self.speed_f * HIO_FIELD_54;
        }
        if speed < 0.5 {
            speed = 0.5;
        }
        self.frame.rate = speed;
    }

    fn jnt_limits(&self, index: usize, axis: i16) -> (i32, i32) {
        match (index, axis) {
            (0, 0) => (HIO_MAX_HEAD_X, HIO_MIN_HEAD_X),
            (0, 1) => (HIO_MAX_HEAD_Y, HIO_MIN_HEAD_Y),
            (1, 0) => (HIO_MAX_BACKBONE_X, HIO_MIN_BACKBONE_X),
            _ => (HIO_MAX_BACKBONE_Y, HIO_MIN_BACKBONE_Y),
        }
    }

    fn jnt_angle(&self, index: usize, axis: i16) -> i16 {
        match (index, axis) {
            (0, 0) => self.head_x,
            (0, 1) => self.head_y,
            (1, 0) => self.backbone_x,
            _ => self.backbone_y,
        }
    }

    fn set_jnt_angle(&mut self, index: usize, axis: i16, value: i16) {
        match (index, axis) {
            (0, 0) => self.head_x = value,
            (0, 1) => self.head_y = value,
            (1, 0) => self.backbone_x = value,
            _ => self.backbone_y = value,
        }
    }

    fn jnt_move(&mut self, param_1: i16, axis: i16) -> bool {
        let mut angles = [0_i16; 2];
        for index in 0..2 {
            let mut angle = if (index == 0 && self.head_lock) || (index == 1 && self.backbone_lock)
            {
                0
            } else {
                param_1
            };
            let (max_diff, min_diff) = self.jnt_limits(index, axis);
            angle = limitter(angle, max_diff, min_diff);
            angles[index] = angle;
            let current = self.jnt_angle(index, axis);
            let next = add_calc_angle_l(
                current,
                angle,
                i32::from(JNT_DIV),
                HIO_MAX_TURN_STEP,
                JNT_MIN_STEP,
            );
            self.set_jnt_angle(index, axis, next);
        }
        let mut count = 0;
        for index in 0..2 {
            let current = self.jnt_angle(index, axis);
            if distance_angle_s(angles[index], current) <= HIO_MAX_TURN_STEP / 2 {
                count += 1;
            }
        }
        count == 2
    }

    fn jnt_follow(&mut self, out_y: &mut i16, target_y: i16, max_vel: i16) -> bool {
        let orig = *out_y;
        ang_calc_s(out_y, target_y, JNT_TURN_SPEED, max_vel);
        let delta_y = out_y.wrapping_sub(orig);
        let rest = target_y.wrapping_sub(*out_y);
        let mut prev_sum = 0_i16;
        let mut sum = 0_i16;
        for index in [1_usize, 0] {
            sum = sum.wrapping_add(self.jnt_angle(index, 1));
            if (delta_y > 0 && sum > rest) || (delta_y < 0 && sum < rest) {
                let (max_diff, min_diff) = self.jnt_limits(index, 1);
                let value = limitter(rest.wrapping_sub(prev_sum), max_diff, min_diff);
                self.set_jnt_angle(index, 1, value);
            }
            prev_sum = prev_sum.wrapping_add(self.jnt_angle(index, 1));
        }
        target_y != *out_y
    }

    fn jnt_look_at_target(
        &mut self,
        out_y: &mut i16,
        dst_pos: Option<Vec3>,
        src_pos: Vec3,
        default_y: i16,
        max_vel: i16,
        no_force: bool,
    ) {
        let (target_y, r23) = match dst_pos {
            Some(dst) => {
                let delta = dst - src_pos;
                let dist_xz = (delta.x * delta.x + delta.z * delta.z).sqrt();
                (atan2s(delta.x, delta.z), atan2s(delta.y, dist_xz))
            }
            None => (default_y, 0),
        };
        let delta_y = target_y.wrapping_sub(*out_y);
        let max_y = HIO_MAX_BACKBONE_Y + HIO_MAX_HEAD_Y;
        let min_y = HIO_MIN_BACKBONE_Y + HIO_MIN_HEAD_Y;
        let r3 = self.jnt_move(delta_y, 1);
        if (self.jnt_trn || (i32::from(delta_y) > max_y || i32::from(delta_y) < min_y) && r3)
            && !no_force
        {
            self.jnt_trn = self.jnt_follow(out_y, target_y, max_vel);
        }
        self.jnt_move(r23, 0);
    }

    pub fn look_back(&mut self, actor: &mut Actor, ctx: &Ctx<'_, '_>) {
        let desired_y_rot = actor.angle[1];
        if self.status == STATUS_TALK {
            self.jnt_trn = true;
        }
        if self.status == STATUS_WALK_TO_JAIL
            || self.status == STATUS_WAIT_NEAR_JAIL
            || self.status == STATUS_SIT
        {
            self.jnt_trn = true;
            self.has_attention = true;
        }
        if self.ty == TYPE_WHITE || self.ty == TYPE_RED {
            if self.status == STATUS_WAIT {
                if self.anm_prm_idx == ANM_PRM_IDX_MAWASU || self.anm_prm_idx == ANM_PRM_IDX_WAIT04
                {
                    self.backbone_lock = true;
                    self.jnt_trn = false;
                }
            } else if self.anm_prm_idx == ANM_PRM_IDX_MAWASU
                || self.anm_prm_idx == ANM_PRM_IDX_WAIT04
            {
                self.backbone_lock = true;
                self.jnt_trn = false;
            } else {
                self.backbone_lock = false;
                self.jnt_trn = true;
                self.has_attention = true;
            }
        }

        let mut dst_pos: Option<Vec3> = None;
        let mut src_pos = Vec3::ZERO;
        if self.status == STATUS_WALK_TO_STOOL {
            self.jnt_trn = true;
            dst_pos = Some(self.stool_look_pos);
            src_pos = Vec3::new(actor.pos.x, self.attn_eye.y, actor.pos.z);
        } else if self.cut_attention {
            self.jnt_trn = true;
            dst_pos = Some(self.cut_attn_pos);
            src_pos = Vec3::new(actor.pos.x, self.attn_eye.y, actor.pos.z);
        } else if self.has_attention {
            dst_pos = Some(ctx.player_eye);
            src_pos = Vec3::new(actor.pos.x, self.attn_eye.y, actor.pos.z);
        }

        if self.jnt_trn {
            let mut target = HIO_MAX_HEAD_TURN_VEL;
            let turn = self.cut_turn_speed;
            if turn != 0 {
                target = turn;
            }
            add_calc_angle_s2(
                &mut self.max_head_turn_velocity,
                target,
                HEAD_TURN_SCALE,
                HEAD_TURN_MAX,
            );
        } else {
            self.max_head_turn_velocity = 0;
        }

        let max_vel = self.max_head_turn_velocity;
        let out_y = &mut actor.angle[1];
        self.jnt_look_at_target(out_y, dst_pos, src_pos, desired_y_rot, max_vel, false);
    }

    pub fn set_attention(&mut self) {
        let temp = self.anm_prm_idx;
        if temp != ANM_PRM_IDX_WAIT01
            && temp != ANM_PRM_IDX_WALK01
            && temp != ANM_PRM_IDX_TALK01
            && temp != ANM_PRM_IDX_GUARD
            && temp != ANM_PRM_IDX_MAWASU
            && temp != ANM_PRM_IDX_TALK02
            && temp != ANM_PRM_IDX_WAIT04
        {
            return;
        }
        self.attn_eye = Vec3::new(self.eye_pos.x, self.eye_pos.y, self.eye_pos.z);
        self.attn_target = Vec3::new(
            self.attn_pos.x,
            self.attn_pos.y + HIO_ATTN_Y_OFFSET,
            self.attn_pos.z,
        );
    }

    pub fn calc_sitpos(&mut self, ctx: &Ctx<'_, '_>) {
        let src_sit = Vec3::new(0.0, 80.0, -25.0);
        let src_walk = Vec3::new(0.0, 0.0, -60.0);
        let player = ctx.player.unwrap_or(Vec3::ZERO);
        let angle = target_angle_y(player, self.stool_pos);
        self.sit_pos = rotate_y(src_sit, angle) + self.stool_pos;
        self.walk_to_stool_pos = rotate_y(src_walk, angle) + self.stool_pos;
    }

    fn player_in_circle(
        &self,
        ctx: &Ctx<'_, '_>,
        pos: Vec3,
        radius: f32,
        half_height: f32,
    ) -> bool {
        let Some(player) = ctx.player else {
            return false;
        };
        let dx = player.x - pos.x;
        let dz = player.z - pos.z;
        let dist_xz = (dx * dx + dz * dz).sqrt();
        let dist_y = (pos.y - player.y).abs();
        dist_xz < radius && dist_y < half_height
    }

    fn ground_hit(&self, ctx: &Ctx<'_, '_>, pos: Vec3) -> bool {
        ground::grounded(ctx.solid, pos)
    }

    fn wall_hit(&self, ctx: &Ctx<'_, '_>, pos: Vec3) -> bool {
        ground::surface(ctx.solid, pos).is_none()
    }

    fn demo_end_check(&self, ctx: &Ctx<'_, '_>, name: &str) -> bool {
        let Some(index) = ctx.list.event_index(name) else {
            return false;
        };
        ctx.active.end_check(index)
    }
}

impl NpcTc {
    pub fn status_wait(&mut self) {
        if self.ty == TYPE_WHITE || self.ty == TYPE_RED {
            if self.tower_stopped {
                self.anm_prm_idx = ANM_PRM_IDX_WAIT04;
            } else {
                self.anm_prm_idx = ANM_PRM_IDX_MAWASU;
            }
        }
        if self.talking_near_jail {
            self.status = STATUS_TALK;
        } else if self.has_attention {
            self.event_idx = EVENT_IDX_TALK;
        }
    }

    pub fn status_sit(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        self.calc_sitpos(ctx);
        actor.pos = self.sit_pos;
        self.gravity = 0.0;
        if self.anm_prm_idx != ANM_PRM_IDX_JAMP_A
            && self.player_in_circle(ctx, actor.pos, HIO_FIELD_2C, PLAYER_CIRCLE_H)
        {
            if !self.has_entered_sit_radius {
                self.has_entered_sit_radius = true;
                self.event_idx = 3;
                self.status = STATUS_DEMO_JUMP;
                return;
            }
            self.anm_prm_idx = ANM_PRM_IDX_JAMP_A;
        }
        if self.frame.is_stop() && self.anm_prm_idx == ANM_PRM_IDX_JAMP_A {
            self.speed_f = HIO_FIELD_38;
            self.speed.y = HIO_FIELD_40;
            self.gravity = HIO_FIELD_3C;
            self.anm_prm_idx = ANM_PRM_IDX_JAMP_B;
            self.status = STATUS_JUMP;
        }
    }

    pub fn status_jump(&mut self, actor: &Actor, ctx: &Ctx<'_, '_>) {
        if self.speed.y <= 0.0 && self.anm_prm_idx == ANM_PRM_IDX_JAMP_B {
            self.anm_prm_idx = ANM_PRM_IDX_JAMP_C1;
        }
        if self.ground_hit(ctx, actor.pos) && self.anm_prm_idx == ANM_PRM_IDX_JAMP_C1 {
            self.speed_f = 0.0;
            self.anm_prm_idx = ANM_PRM_IDX_JAMP_C2;
            self.jump_landing_timer = JUMP_LAND_TIMER;
            return;
        }
        if calc_timer(&mut self.jump_landing_timer) == 0 && self.anm_prm_idx == ANM_PRM_IDX_JAMP_C2
        {
            self.anm_prm_idx = ANM_PRM_IDX_WALK01;
            self.status = STATUS_WALK_TO_JAIL;
        }
    }

    pub fn status_walk_to_jail(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        let wall_hit = self.wall_hit(ctx, actor.pos);
        if wall_hit && self.talking_near_jail {
            self.target_speed = 0.0;
            self.speed_f = 0.0;
            if !self.has_talked_near_jail || HIO_FIELD_44 {
                self.anm_prm_idx = ANM_PRM_IDX_WAIT03;
                self.event_reset();
                self.status = STATUS_DEMO_TALK;
                if !ctx.stories.test(BIT_0B40) {
                    if ctx.text.clear_count != 0 {
                        self.event_idx = 6;
                    } else {
                        self.event_idx = 5;
                    }
                } else {
                    self.event_idx = 7;
                }
                ctx.stories.set(BIT_0B40);
                self.has_talked_near_jail = true;
                return;
            }
            self.status = STATUS_TALK_NEAR_JAIL;
        } else if wall_hit && self.has_attention {
            self.event_idx = EVENT_IDX_TALK;
        }

        let angle = ctx
            .player
            .map(|player| target_angle_y(actor.pos, player))
            .unwrap_or(0);
        if wall_hit && scos(angle).abs() > WALL_COS_LIMIT {
            self.target_speed = 0.0;
            if self.speed_f == 0.0 {
                self.anm_prm_idx = ANM_PRM_IDX_WAIT01;
            }
        } else {
            self.anm_prm_idx = ANM_PRM_IDX_WALK01;
            self.target_speed = HIO_FIELD_58;
            if self.speed_f == 0.0 {
                self.speed_f = HIO_FIELD_5C;
            }
        }
        if wall_hit && !self.player_in_circle(ctx, actor.pos, HIO_FIELD_30, PLAYER_CIRCLE_H) {
            self.status = STATUS_WALK_TO_STOOL;
        }
    }

    pub fn status_talk_near_jail(&mut self, ctx: &mut Ctx<'_, '_>) {
        self.target_speed = 0.0;
        self.speed_f = 0.0;
        if self.talk(ctx) == STATUS_BOX_CLOSED {
            self.status = STATUS_WAIT_NEAR_JAIL;
            self.event_reset();
            self.talking_near_jail = false;
        }
    }

    pub fn status_wait_near_jail(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        let wall_hit = self.wall_hit(ctx, actor.pos);
        if wall_hit && self.talking_near_jail {
            self.target_speed = 0.0;
            self.speed_f = 0.0;
            if !self.has_talked_near_jail || HIO_FIELD_44 {
                self.anm_prm_idx = ANM_PRM_IDX_WAIT03;
                self.event_reset();
                self.status = STATUS_DEMO_TALK;
                if !ctx.stories.test(BIT_0B40) {
                    self.event_idx = 5;
                } else {
                    self.event_idx = 7;
                }
                ctx.stories.set(BIT_0B40);
                self.has_talked_near_jail = true;
                return;
            }
            self.status = STATUS_TALK_NEAR_JAIL;
        } else if wall_hit && self.has_attention {
            self.event_idx = EVENT_IDX_TALK;
        }

        let angle = ctx
            .player
            .map(|player| target_angle_y(actor.pos, player))
            .unwrap_or(0);
        if wall_hit && scos(angle).abs() > WALL_COS_LIMIT {
            self.target_speed = 0.0;
            if self.speed_f == 0.0 {
                self.anm_prm_idx = ANM_PRM_IDX_WAIT01;
            }
        } else {
            self.anm_prm_idx = ANM_PRM_IDX_WALK01;
            self.target_speed = HIO_FIELD_58;
            if self.speed_f == 0.0 {
                self.speed_f = HIO_FIELD_5C;
            }
        }
    }

    pub fn status_walk_to_stool(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        self.calc_sitpos(ctx);
        if self.player_in_circle(ctx, actor.pos, HIO_FIELD_30, PLAYER_CIRCLE_H) {
            self.target_speed = 0.0;
            if self.speed_f == 0.0 {
                self.status = STATUS_WALK_TO_JAIL;
            }
        } else {
            self.stool_look_pos = self.stool_pos;
            add_calc_pos_xz2(
                &mut actor.pos,
                self.walk_to_stool_pos,
                MOVE_SCALE,
                self.speed_f,
            );
            let temp = abs_xz(self.walk_to_stool_pos - actor.pos);
            if temp < MOVE_REACH {
                self.target_speed = 0.0;
                if self.speed_f == 0.0 {
                    self.anm_prm_idx = ANM_PRM_IDX_WAIT01;
                    self.status = STATUS_SIT_TO_STOOL;
                }
            } else {
                self.anm_prm_idx = ANM_PRM_IDX_WALK01;
                self.target_speed = HIO_FIELD_58;
            }
        }
    }

    pub fn status_sit_to_stool(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        self.calc_sitpos(ctx);
        add_calc_angle_s2(&mut actor.angle[1], self.start_angle[1], 4, 0x800);
        if (self.start_angle[1] as f32 - actor.angle[1] as f32).abs() < SIT_TURN_TOL {
            if self.speed.y == 0.0 && self.ground_hit(ctx, actor.pos) {
                self.speed.y = SIT_RISE;
                self.gravity = SIT_GRAVITY;
            }
            if self.speed.y < 0.0 {
                self.anm_prm_idx = ANM_PRM_IDX_WAIT02;
            }
            if self.speed.y < 0.0 && actor.pos.y < self.sit_pos.y {
                self.gravity = 0.0;
                self.speed.y = 0.0;
                actor.pos.y = self.sit_pos.y;
            }
            add_calc_pos_xz2(&mut actor.pos, self.sit_pos, MOVE_SCALE, SIT_APPROACH_STEP);
            let diff = abs_xz(self.sit_pos - actor.pos);
            if actor.pos.y == self.sit_pos.y && diff < SIT_REACH {
                self.status = STATUS_SIT;
            }
        }
    }

    pub fn status_talk(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.talk(ctx) != STATUS_BOX_CLOSED {
            return;
        }
        match self.ty {
            TYPE_RED | TYPE_WHITE => {
                self.start_tower(actor);
                self.anm_prm_idx = ANM_PRM_IDX_MAWASU;
                self.start_tower(actor);
            }
            TYPE_NORMAL2 => {
                self.start_tower(actor);
            }
            _ => {}
        }
        self.anm_prm_idx = ANM_PRM_IDX_WAIT01;
        self.status = STATUS_WAIT;
        self.event_reset();
        self.talking_near_jail = false;
    }

    pub fn status_pay_rupee(&mut self, ctx: &mut Ctx<'_, '_>) {
        if self.talk(ctx) == STATUS_BOX_CLOSED {
            self.event_reset();
            self.event_idx = 9;
            self.status = STATUS_DEMO_PAY_RUPEE;
        }
    }

    pub fn status_demo_pay_rupee(&mut self, ctx: &mut Ctx<'_, '_>) {
        if self.demo_end_check(ctx, "TC_PAY_RUPEE") {
            self.event_reset();
            self.event_idx = EVENT_IDX_SPEAK;
            self.status = STATUS_TALK;
        }
    }

    pub fn status_get_rupee(&mut self, ctx: &mut Ctx<'_, '_>) {
        if self.talk(ctx) == STATUS_BOX_CLOSED {
            self.event_reset();
            self.event_idx = 8;
            self.status = STATUS_DEMO_GET_RUPEE;
        }
    }

    pub fn status_demo_get_rupee(&mut self, ctx: &mut Ctx<'_, '_>) {
        if self.demo_end_check(ctx, "TC_GET_RUPEE") {
            self.flag_812 = true;
            self.event_reset();
            self.event_idx = EVENT_IDX_SPEAK;
            self.status = STATUS_TALK;
        }
    }

    pub fn status_monument_complete(&mut self, ctx: &mut Ctx<'_, '_>) {
        if self.talk(ctx) == STATUS_BOX_CLOSED {
            self.event_reset();
            self.event_idx = 8;
            self.status = STATUS_DEMO_MONUMENT_COMPLETE;
        }
    }

    pub fn status_demo_monument_complete(&mut self, ctx: &mut Ctx<'_, '_>) {
        if self.demo_end_check(ctx, "TC_GET_RUPEE") {
            self.flag_813 = true;
            self.event_reset();
            self.event_idx = EVENT_IDX_SPEAK;
            self.status = STATUS_TALK;
        }
    }

    pub fn status_demo_jump(&mut self, ctx: &mut Ctx<'_, '_>) {
        if self.demo_end_check(ctx, "TC_JUMP_DEMO") {
            self.status = STATUS_WALK_TO_JAIL;
            self.event_reset();
        }
    }

    pub fn status_demo_rescue(&mut self, ctx: &mut Ctx<'_, '_>) {
        if self.demo_end_check(ctx, "TC_RESCUE") {
            ctx.stories.set(BIT_0B80);
            self.event_reset();
            self.deleted = true;
        }
    }

    pub fn status_demo_talk(&mut self, ctx: &mut Ctx<'_, '_>) {
        if self.demo_end_check(ctx, "TC_TALK_NEAR_JAIL")
            || self.demo_end_check(ctx, "TC_TALK_NEAR_JAIL_S")
            || self.demo_end_check(ctx, "TC_TALK_NEAR_JAIL2")
        {
            self.status = STATUS_WAIT_NEAR_JAIL;
            self.event_reset();
            self.talking_near_jail = false;
        }
    }

    pub fn help_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) -> bool {
        if self.action_status == ACTION_STARTING {
            self.status = STATUS_SIT;
            self.action_status += 1;
        } else if self.action_status != ACTION_ENDING {
            if ctx.stories.test(BIT_0B80) {
                self.deleted = true;
            }
            if self.demo_end_check(ctx, "OpenDoor") {
                self.target_speed = 0.0;
                self.speed_f = 0.0;
                self.event_idx = 4;
                self.status = STATUS_DEMO_RESCUE;
                return true;
            } else {
                let temp = self
                    .head_y
                    .wrapping_add(actor.angle[1])
                    .wrapping_add(self.backbone_y);
                self.has_attention = self.chk_attention(actor.pos, temp, ctx.player);
                self.event_idx = EVENT_IDX_NONE;
                match self.status {
                    STATUS_WAIT => self.status_wait(),
                    STATUS_TALK => self.status_talk(actor, ctx),
                    STATUS_SIT => self.status_sit(actor, ctx),
                    STATUS_JUMP => self.status_jump(actor, ctx),
                    STATUS_WALK_TO_JAIL => self.status_walk_to_jail(actor, ctx),
                    STATUS_TALK_NEAR_JAIL => self.status_talk_near_jail(ctx),
                    STATUS_WAIT_NEAR_JAIL => self.status_wait_near_jail(actor, ctx),
                    STATUS_WALK_TO_STOOL => self.status_walk_to_stool(actor, ctx),
                    STATUS_SIT_TO_STOOL => self.status_sit_to_stool(actor, ctx),
                    STATUS_DEMO_JUMP => self.status_demo_jump(ctx),
                    STATUS_DEMO_RESCUE => self.status_demo_rescue(ctx),
                    STATUS_DEMO_TALK => self.status_demo_talk(ctx),
                    _ => {}
                }
                self.calc_move();
                self.look_back(actor, ctx);
                self.set_attention();
            }
        }
        true
    }

    pub fn wait_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) -> bool {
        if self.action_status == ACTION_STARTING {
            self.status = STATUS_WAIT;
            self.action_status += 1;
        } else if self.action_status != ACTION_ENDING {
            let temp = self
                .head_y
                .wrapping_add(actor.angle[1])
                .wrapping_add(self.backbone_y);
            self.has_attention = self.chk_attention(actor.pos, temp, ctx.player);
            self.event_idx = EVENT_IDX_NONE;
            match self.status {
                STATUS_WAIT => self.status_wait(),
                STATUS_TALK => self.status_talk(actor, ctx),
                STATUS_PAY_RUPEE => self.status_pay_rupee(ctx),
                STATUS_DEMO_PAY_RUPEE => self.status_demo_pay_rupee(ctx),
                STATUS_GET_RUPEE => self.status_get_rupee(ctx),
                STATUS_DEMO_GET_RUPEE => self.status_demo_get_rupee(ctx),
                STATUS_MONUMENT_COMPLETE => self.status_monument_complete(ctx),
                STATUS_DEMO_MONUMENT_COMPLETE => self.status_demo_monument_complete(ctx),
                _ => {}
            }
            self.calc_move();
            self.look_back(actor, ctx);
            self.set_attention();
        }
        true
    }

    pub fn run_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.ty == TYPE_NORMAL {
            self.help_action(actor, ctx);
        } else {
            self.wait_action(actor, ctx);
        }
    }

    pub fn create_init(&mut self, actor: &Actor, ctx: &mut Ctx<'_, '_>) {
        self.created = true;
        self.start_angle = actor.angle;
        self.attn_pos = actor.pos;
        self.eye_pos = actor.pos;
        self.attn_eye = actor.pos;
        self.has_talked_near_jail = false;
        self.tex_pattern_num_idx = 1;
        match self.ty {
            TYPE_NORMAL => {
                self.anm_prm_idx = ANM_PRM_IDX_WAIT02;
                if let Some(stool) = ctx.stool {
                    self.stool_pos = stool;
                }
                self.status = STATUS_SIT;
                self.action_status = ACTION_ONGOING;
                self.gravity = 0.0;
            }
            TYPE_BLUE | TYPE_NORMAL2 => {
                self.gravity = -9.0;
                self.anm_prm_idx = ANM_PRM_IDX_WAIT01;
                self.status = STATUS_WAIT;
                self.action_status = ACTION_ONGOING;
            }
            TYPE_RED | TYPE_WHITE => {
                self.gravity = -9.0;
                self.head_lock = true;
                self.anm_prm_idx = ANM_PRM_IDX_MAWASU;
                self.status = STATUS_WAIT;
                self.action_status = ACTION_ONGOING;
            }
            _ => {}
        }
        self.old_anm_prm_idx = 0;
        self.set_tex_anm();
        self.set_anm(ctx);
    }
}

impl NpcTc {
    pub fn event_cut_proc(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) -> bool {
        let Some((cut, advance)) = ctx.active.staff_cut(ctx.list, STAFF_NAME) else {
            return false;
        };
        let Some(name) = ctx.list.cut_name(cut).map(str::to_owned) else {
            return false;
        };
        let Some(at) = EVENT_CUT_ACTIONS.iter().position(|action| *action == name) else {
            return false;
        };
        self.cut_act = i8::try_from(at).unwrap_or(NO_CUT);
        if advance {
            self.cut_attention = false;
            self.event_cut_start(cut, at, ctx);
        }
        self.event_cut_run(cut, at, actor, ctx);
        true
    }

    fn event_cut_start(&mut self, cut: usize, at: usize, ctx: &mut Ctx<'_, '_>) {
        match EVENT_CUT_ACTIONS.get(at).copied() {
            Some("WAIT") => {
                self.cut_timer = cut_number_or(ctx, cut, "Timer", 0);
            }
            Some("TURN_TO_ACTOR") => {
                let offset = ctx.list.cut_vec(cut, "OffsetPos").unwrap_or(Vec3::ZERO);
                self.cut_attn_pos = ctx.player_eye + offset;
                self.cut_no_turn = ctx.list.cut_number(cut, "NoTurn").is_some();
                self.cut_timer = cut_number_or(ctx, cut, "Timer", DEFAULT_TURN_TIMER);
                self.cut_turn_speed = cut_number_or(ctx, cut, "TurnSpeed", 0);
                self.turn_locks(ctx, cut);
                self.cut_prev_angle = 0.0;
            }
            Some("TURN_TO_POS") => {
                self.cut_attn_pos = ctx.list.cut_vec(cut, "Pos").unwrap_or(Vec3::ZERO);
                self.cut_no_turn = ctx.list.cut_number(cut, "NoTurn").is_some();
                self.cut_timer = cut_number_or(ctx, cut, "Timer", DEFAULT_TURN_TIMER);
                self.cut_turn_speed = cut_number_or(ctx, cut, "TurnSpeed", 0);
                self.turn_locks(ctx, cut);
                self.cut_prev_angle = 0.0;
            }
            Some("MOVE_TO_ACTOR") => {
                self.cut_attention = ctx.list.cut_number(cut, "Attention").is_some();
                self.cut_no_turn = ctx.list.cut_number(cut, "NoTurn").is_some();
                self.cut_speed = ctx
                    .list
                    .cut_float(cut, "Speed")
                    .unwrap_or(DEFAULT_MOVE_SPEED);
            }
            Some("MOVE_TO_POS") => {
                self.cut_attn_pos = ctx.list.cut_vec(cut, "Pos").unwrap_or(Vec3::ZERO);
                self.cut_attention = ctx.list.cut_number(cut, "Attention").is_some();
                self.cut_no_turn = ctx.list.cut_number(cut, "NoTurn").is_some();
                self.cut_speed = ctx
                    .list
                    .cut_float(cut, "Speed")
                    .unwrap_or(DEFAULT_MOVE_SPEED);
            }
            _ => {}
        }
    }

    fn turn_locks(&mut self, ctx: &Ctx<'_, '_>, cut: usize) {
        let turn_type = ctx
            .list
            .cut_number(cut, "TurnType")
            .map_or(1, |value| value);
        match turn_type {
            2 => {
                self.jnt_trn = true;
                self.backbone_lock = false;
            }
            1 => self.backbone_lock = false,
            0 => self.backbone_lock = true,
            _ => self.backbone_lock = false,
        }
    }

    fn event_cut_run(&mut self, cut: usize, at: usize, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        match EVENT_CUT_ACTIONS.get(at).copied() {
            Some("WAIT") => {
                if calc_timer_i16(&mut self.cut_timer) == 0 {
                    ctx.active.end_staff_cut(ctx.list, ctx.flags, STAFF_NAME);
                }
            }
            Some("TURN_TO_ACTOR") | Some("TURN_TO_POS") => {
                self.cut_attention = true;
                let target = target_angle_y(actor.pos, self.cut_attn_pos);
                let temp2 = target.wrapping_sub(actor.angle[1]) as i32;
                let temp = (temp2 as f32).abs();
                if self.cut_prev_angle == temp && (temp2 as f32).abs() < TURN_TOLERANCE {
                    calc_timer_i16(&mut self.cut_timer);
                }
                if self.cut_timer == 0 {
                    self.cut_turn_speed = 0;
                    ctx.active.end_staff_cut(ctx.list, ctx.flags, STAFF_NAME);
                }
                let diff = target.wrapping_sub(actor.angle[1]) as i32;
                self.cut_prev_angle = (diff as f32).abs();
            }
            Some("MOVE_TO_ACTOR") | Some("MOVE_TO_POS") => {
                let angle = target_angle_y(actor.pos, self.cut_attn_pos);
                let del = ctx.list.cut_float(cut, "DelDistance").unwrap_or(0.0);
                let mut temp = self.cut_attn_pos;
                temp.x -= del * ssin(angle);
                temp.z -= del * scos(angle);
                if self.cut_speed == 0.0 {
                    actor.pos = temp;
                    ctx.active.end_staff_cut(ctx.list, ctx.flags, STAFF_NAME);
                }
                let dist = abs_xz(temp - actor.pos);
                add_calc2(&mut actor.pos.x, temp.x, MOVE_SCALE, self.cut_speed);
                add_calc2(&mut actor.pos.z, temp.z, MOVE_SCALE, self.cut_speed);
                if dist < MOVE_REACH {
                    self.cut_speed = 0.0;
                    ctx.active.end_staff_cut(ctx.list, ctx.flags, STAFF_NAME);
                }
            }
            _ => {
                ctx.active.end_staff_cut(ctx.list, ctx.flags, STAFF_NAME);
            }
        }
    }

    pub fn cut_proc(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        let Some((cut, advance)) = ctx.active.staff_cut(ctx.list, STAFF_NAME) else {
            return;
        };
        let Some(name) = ctx.list.cut_name(cut).map(str::to_owned) else {
            return;
        };
        let Some(at) = CUT_ACTIONS.iter().position(|action| *action == name) else {
            ctx.active.end_staff_cut(ctx.list, ctx.flags, STAFF_NAME);
            return;
        };
        if advance {
            self.cut_start(cut, at, ctx);
        }
        self.cut_run(at, actor, ctx);
    }

    fn cut_start(&mut self, cut: usize, at: usize, ctx: &mut Ctx<'_, '_>) {
        match CUT_ACTIONS.get(at).copied() {
            Some("SIT_TO_JUMP") => self.anm_prm_idx = ANM_PRM_IDX_JAMP_A,
            Some("PRESENT") => {}
            Some("SET_ANM") => {
                if ctx.list.cut_text(cut, "Name") == Some("GET") {
                    self.anm_prm_idx = ANM_PRM_IDX_GET;
                } else {
                    self.anm_prm_idx = ANM_PRM_IDX_WAIT01;
                }
            }
            Some("BACKJUMP") => {
                self.target_speed = 0.0;
                self.speed_f = HIO_FIELD_48;
                self.speed.y = HIO_FIELD_50;
                self.gravity = HIO_FIELD_4C;
                self.anm_prm_idx = ANM_PRM_IDX_GUARD;
            }
            Some("EFFECT") => {
                ctx.events.list.push(SimEvent::Se(SE_CMN_NOTICE));
            }
            Some("DOOR_OPEN") => {
                ctx.events.list.push(SimEvent::Se(SE_DOOR_N_OPEN));
            }
            Some("DOOR_CLOSE") => {
                ctx.events.list.push(SimEvent::Se(SE_DOOR_N_CLOSE_1));
            }
            Some("DOOR_CLOSE2") => {
                ctx.events.list.push(SimEvent::Se(SE_DOOR_N_CLOSE_2));
            }
            Some("PAY") => {}
            _ => {}
        }
    }

    fn cut_run(&mut self, at: usize, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        match CUT_ACTIONS.get(at).copied() {
            Some("SIT_TO_JUMP") => self.cut_sit_to_jump_proc(actor, ctx),
            Some("PRESENT") => self.cut_present_proc(actor, ctx),
            Some("SET_ANM") => {
                if self.anm_prm_idx == ANM_PRM_IDX_GET {
                    ctx.active.end_staff_cut(ctx.list, ctx.flags, STAFF_NAME);
                }
                if self.frame.is_stop() {
                    ctx.active.end_staff_cut(ctx.list, ctx.flags, STAFF_NAME);
                }
            }
            Some("BACKJUMP") => self.cut_back_jump_proc(actor, ctx),
            _ => {
                ctx.active.end_staff_cut(ctx.list, ctx.flags, STAFF_NAME);
            }
        }
    }

    fn cut_sit_to_jump_proc(&mut self, actor: &Actor, ctx: &mut Ctx<'_, '_>) {
        if self.frame.is_stop() && self.anm_prm_idx == ANM_PRM_IDX_JAMP_A {
            self.speed_f = HIO_FIELD_38;
            self.speed.y = HIO_FIELD_40;
            self.gravity = HIO_FIELD_3C;
            self.anm_prm_idx = ANM_PRM_IDX_JAMP_B;
        }
        if self.speed.y <= 0.0 && self.anm_prm_idx == ANM_PRM_IDX_JAMP_B {
            self.anm_prm_idx = ANM_PRM_IDX_JAMP_C1;
        }
        if self.ground_hit(ctx, actor.pos) && self.anm_prm_idx == ANM_PRM_IDX_JAMP_C1 {
            self.speed_f = 0.0;
            self.anm_prm_idx = ANM_PRM_IDX_JAMP_C2;
            self.jump_landing_timer = JUMP_LAND_TIMER;
            return;
        }
        if calc_timer(&mut self.jump_landing_timer) == 0 && self.anm_prm_idx == ANM_PRM_IDX_JAMP_C2
        {
            self.anm_prm_idx = ANM_PRM_IDX_WAIT01;
            ctx.active.end_staff_cut(ctx.list, ctx.flags, STAFF_NAME);
        }
    }

    fn cut_back_jump_proc(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.ground_hit(ctx, actor.pos) && self.speed.y != 0.0 {
            self.speed_f = 0.0;
            self.speed.y = 0.0;
        } else if self.wall_hit(ctx, actor.pos) && self.speed.y == 0.0 {
            self.speed_f = 0.0;
            ctx.active.end_staff_cut(ctx.list, ctx.flags, STAFF_NAME);
        }
    }

    fn cut_present_proc(&mut self, actor: &Actor, ctx: &mut Ctx<'_, '_>) {
        let Some((cut, _)) = ctx.active.staff_cut(ctx.list, STAFF_NAME) else {
            return;
        };
        let temp = ctx.list.cut_number(cut, "ItemType").unwrap_or(0);
        let mut item_no = NO_ITEM;
        match temp {
            0 => item_no = ITEM_TINGLE_TUNER,
            1 => item_no = ITEM_COLLECT_MAP_35,
            2 => {
                if self.ty == TYPE_RED {
                    if self.check_all_monument_fee(ctx) {
                        item_no = ITEM_TINGLE_RUPEE_6;
                    } else {
                        match self.check_all_monument_pay(ctx) {
                            1 => item_no = ITEM_TINGLE_RUPEE_1,
                            2 => item_no = ITEM_TINGLE_RUPEE_1 + 1,
                            3 => item_no = ITEM_TINGLE_RUPEE_1 + 2,
                            4 => item_no = ITEM_TINGLE_RUPEE_1 + 3,
                            5 => item_no = ITEM_TINGLE_RUPEE_1 + 4,
                            _ => {}
                        }
                    }
                } else if self.ty == TYPE_NORMAL2 {
                    match self.analysis_collect_map(ctx) {
                        1 => item_no = ITEM_TRIFORCE_MAP_1,
                        2 => item_no = ITEM_TRIFORCE_MAP_1 + 1,
                        3 => item_no = ITEM_TRIFORCE_MAP_1 + 2,
                        4 => item_no = ITEM_TRIFORCE_MAP_1 + 3,
                        5 => item_no = ITEM_TRIFORCE_MAP_1 + 4,
                        6 => item_no = ITEM_TRIFORCE_MAP_1 + 5,
                        7 => item_no = ITEM_TRIFORCE_MAP_1 + 6,
                        8 => item_no = ITEM_TRIFORCE_MAP_1 + 7,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        if item_no == NO_ITEM {
            return;
        }
        ctx.give_item = Some((actor.pos, item_no));
        ctx.active.end_staff_cut(ctx.list, ctx.flags, STAFF_NAME);
    }
}

impl NpcTc {
    pub fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_, '_>) {
        if self.ty == TYPE_WHITE || self.ty == TYPE_RED || self.ty == TYPE_NORMAL2 {
            self.set_tower(actor, ctx);
        }
        self.play_tex_pattern_anm();
        self.frame.play();
        self.check_order(ctx);
        self.run_action(actor, ctx);
        if self.status != STATUS_WAIT {
            if !self.event_cut_proc(actor, ctx) {
                self.cut_proc(actor, ctx);
            } else {
                match self.cut_act {
                    CUT_MOVE_TO_ACTOR | CUT_MOVE_TO_POS => {
                        if self.cut_speed == 0.0 {
                            self.anm_prm_idx = ANM_PRM_IDX_WAIT01;
                        } else {
                            self.anm_prm_idx = ANM_PRM_IDX_WALK01;
                        }
                    }
                    NO_CUT => {
                        self.anm_prm_idx = ANM_PRM_IDX_WAIT01;
                    }
                    CUT_WAIT => {
                        self.anm_prm_idx = ANM_PRM_IDX_WAIT01;
                    }
                    _ => {}
                }
            }
        }
        if !ctx.text.info.event_run_check() {
            self.cut_no_turn = false;
            self.cut_attention = false;
        }
        self.event_order(ctx);
        self.set_anm(ctx);
        self.set_tex_anm();
        if self.status != STATUS_WALK_TO_STOOL {
            self.move_actor(actor, ctx);
        }
        let temp_pos = actor.pos;
        self.correct_pos(actor, ctx);
        self.delta_pos = actor.pos - temp_pos;
        self.set_collision();
    }

    fn move_actor(&mut self, actor: &mut Actor, ctx: &Ctx<'_, '_>) {
        let _ = ctx;
        let (next, speed_f) = stream::calc_speed(
            self.speed_f,
            actor.angle[1],
            self.speed.y,
            self.gravity,
            MAX_FALL,
        );
        self.speed = next;
        self.speed_f = speed_f;
        actor.pos += self.speed;
    }

    fn correct_pos(&mut self, actor: &mut Actor, ctx: &Ctx<'_, '_>) {
        if let Some(solid) = ctx.solid {
            let from = actor.pos - self.speed;
            actor.pos = solid.slide(from, actor.pos);
        }
        if let Some(floor) = ground::surface(ctx.solid, actor.pos)
            && actor.pos.y < floor
        {
            actor.pos.y = floor;
            self.speed.y = 0.0;
        }
    }

    fn set_collision(&mut self) {}
}

#[derive(SystemParam)]
pub struct World<'w, 's> {
    pub stories: ResMut<'w, StoryBits>,
    pub flags: ResMut<'w, EventFlags>,
    pub collect: ResMut<'w, Collect>,
    pub rupees: ResMut<'w, Rupees>,
    pub chests: Res<'w, Chests>,
    pub list: Res<'w, EventList>,
    pub active: ResMut<'w, ActiveEvent>,
    pub events: ResMut<'w, SimEvents>,
    pub intent: Res<'w, DoIntent>,
    pub collision: Option<Res<'w, Collision>>,
    pub others: Query<'w, 's, &'static Actor, Without<NpcTc>>,
}

#[derive(SystemParam)]
pub struct Msg<'w, 's> {
    pub procs: ResMut<'w, MsgProcs>,
    pub info: ResMut<'w, MsgGameInfo>,
    pub one: ResMut<'w, Msg1Box>,
    pub two: ResMut<'w, Msg2Box>,
    pub text_flags: Res<'w, TextFlags>,
    pub archive: Option<Res<'w, MsgArchives>>,
    pub players: Query<'w, 's, &'static Player>,
}

pub fn control(
    mut world: World,
    mut msg: Msg,
    mut actors: Query<(Entity, &mut Actor, &mut NpcTc)>,
    mut commands: Commands,
) {
    let player = msg.players.iter().next().copied();
    let pressed = world.intent.0;
    let clear_count = msg.text_flags.clear_count;
    let stool: Option<Vec3> = world
        .others
        .iter()
        .filter(|other| super::tsubo::is_tsubo(&other.name))
        .map(|other| other.pos)
        .last();
    let towers: Vec<(Vec3, [i16; 3])> = world
        .others
        .iter()
        .filter(|other| {
            super::qtkhd::is_qtkhd(&other.name)
                && ((other.parameters >> PRM_TYPE_SHIFT) & PRM_TYPE_MASK) == 0
        })
        .map(|other| (other.pos, other.angle))
        .collect();
    let solid = world.collision.as_deref();
    for (entity, mut actor, mut tc) in &mut actors {
        let actor = &mut *actor;
        let tc = &mut *tc;
        let tap = pressed && !tc.do_held;
        tc.do_held = pressed;
        if !tc.created && !tc.is_create(&world.stories) {
            commands.entity(entity).despawn();
            continue;
        }
        let mut text = Text {
            procs: &mut msg.procs,
            info: &mut msg.info,
            one: &mut msg.one,
            two: &mut msg.two,
            clear_count,
            archive: msg
                .archive
                .as_deref()
                .map(|archive| archive as &dyn MsgArchive),
        };
        let player_eye = player.map_or(Vec3::ZERO, |link| {
            Vec3::new(link.pos.x, link.pos.y + HIO_M04, link.pos.z)
        });
        let talk_request = tap
            && player.is_some_and(|link| {
                let gap = link.pos - actor.pos;
                gap.x * gap.x + gap.z * gap.z < TALK_RANGE * TALK_RANGE
            });
        let mut ctx = Ctx {
            entity,
            player: player.map(|link| link.pos),
            player_eye,
            talk_request,
            stool,
            towers: &towers,
            solid,
            events: &mut world.events,
            stories: &mut world.stories,
            flags: &mut world.flags,
            collect: &mut world.collect,
            rupees: &mut world.rupees,
            chests: &world.chests,
            wallet: 0,
            list: &world.list,
            active: &mut world.active,
            text: &mut text,
            give_item: None,
        };
        if !tc.created {
            tc.create_init(actor, &mut ctx);
        } else {
            tc.execute(actor, &mut ctx);
        }
        if let Some((pos, item)) = ctx.give_item {
            let _ = commands.spawn((
                Actor {
                    name: super::event::PRESENT_ACTOR.to_owned(),
                    parameters: u32::from(item),
                    pos,
                    angle: [0; 3],
                    set_id: u16::MAX,
                    room: actor.room,
                    ..Default::default()
                },
                PresentItem {
                    item,
                    hidden: false,
                },
            ));
        }
        if tc.deleted {
            commands.entity(entity).despawn();
        }
    }
}
