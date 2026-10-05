use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::carry::Carry;
use super::collision::Collision;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, StoryBits};
use super::ground;
use super::intent::{CameraYaw, DoIntent, MoveIntent, TargetIntent};
use super::flags::Collect;
use super::inventory::{check_get_item as item_obtained, Equipped, ITEM_WIND_WAKER};
use super::lock::Lock;
use super::player::Player;
use super::stream;
use super::transition::{NextStage, Target as StageTarget};
use super::walk;
use crate::assets::MsgArchives;
use crate::text::driver::TextFlags;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    DA_EVT_MODE_NONE, FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgFlags, MsgGameInfo,
    MsgProcs, fop_msgm_message_set, fop_msgm_tact_message_set,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_INPUT, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED,
    STATUS_MSG_DISPLAYED, STATUS_MSG_ENDS, STATUS_TYPING,
};

pub const MAKAR_ACTOR: &str = "Cb1";
pub const ARC_NAME: &str = "Cb";
pub const STAFF_NAME: &str = "Cb1";
pub const STAGE_SEA: &str = "sea";
pub const NEXT_STAGE: &str = "Otkura";
pub const NEXT_STAGE_POINT: u8 = 0xE6;
pub const NEXT_STAGE_ROOM: u8 = 0;
pub const NEXT_STAGE_LAYER: i8 = 8;

pub const PRM_TYPE_MASK: u32 = 0xFFFF_FFFF;
pub const PRM_TYPE_BOSS_DIE: u32 = 0;
pub const PRM_TYPE_FOREST: u32 = 1;
pub const PRM_TYPE_WATERFALL: u32 = 2;
pub const PRM_TYPE_EKAZE: u32 = 3;
pub const PRM_TYPE_KAZE: u32 = 4;
pub const PRM_TYPE_KAZE_BOSS: u32 = 5;

pub const STTS_NO_CARRY_ACTION: u16 = 0x0001;
pub const STTS_TACT: u16 = 0x0002;
pub const STTS_TACT_CORRECT: u16 = 0x0004;
pub const STTS_TACT_CANCEL: u16 = 0x0008;
pub const STTS_MUSIC: u16 = 0x0010;
pub const STTS_NUT: u16 = 0x0020;
pub const STTS_SHIP_RIDE: u16 = 0x0040;
pub const STTS_PLAYER_FIND: u16 = 0x0080;
pub const STTS_UNK_0100: u16 = 0x0100;

pub const ATTN_NO_CARRY: u32 = 0x0000_0004;
pub const ATTN_ACTION_SPEAK: u32 = 0x0000_0002;
pub const ATTN_LOCKON_TALK: u32 = 0x0000_0001;
pub const ATTN_ACTION_CARRY: u32 = 0x0000_1000;
pub const ATTN_SPEAK_TALK_MASK: u32 = ATTN_ACTION_SPEAK | ATTN_LOCKON_TALK;
pub const ATTN_UNK_10000000: u32 = 0x1000_0000;

pub const ATTN_DIST_TALK: u32 = 0xAB;
pub const ATTN_DIST_SPEAK: u32 = 0xAB;
pub const ATTN_DIST_CARRY: u32 = 8;
pub const ATTN_DIST_SHIP: u32 = 0xAF;

pub const HIO_EYE_OFFSET: f32 = -20.0;
pub const HIO_MAX_HEAD_X: i16 = 0;
pub const HIO_MAX_HEAD_Y: i16 = 0;
pub const HIO_MAX_BACKBONE_X: i16 = 0xBB8;
pub const HIO_MAX_BACKBONE_Y: i16 = 0x1770;
pub const HIO_MIN_HEAD_X: i16 = 0;
pub const HIO_MIN_HEAD_Y: i16 = 0;
pub const HIO_MIN_BACKBONE_X: i16 = -0x3E8;
pub const HIO_MIN_BACKBONE_Y: i16 = -0x1770;
pub const HIO_MAX_TURN_STEP: i16 = 0x7D0;
pub const HIO_MAX_HEAD_TURN_VEL: i16 = 0x7D0;
pub const HIO_ATTN_Y_OFFSET: f32 = 65.0;
pub const HIO_MAX_ATTN_ANGLE_Y: i32 = 0x4000;
pub const HIO_MAX_ATTN_DIST_XZ: f32 = 180.0;
pub const HIO_EYE_LIFT: f32 = 50.0;
pub const HIO_PLAYER_CHASE_DISTANCE: f32 = 140.0;
pub const HIO_CHASE_DIST_SCALE: f32 = 0.05;
pub const HIO_MAX_WALK_SPEED: f32 = 9.0;
pub const HIO_MIN_WALK_SPEED: f32 = 3.0;
pub const HIO_FORWARD_ACCEL: f32 = 0.4;
pub const HIO_WALK_SCALE: i16 = 0xEC;
pub const HIO_WALK_MAX_STEP: i16 = 0x8FC;
pub const HIO_WALK_MIN_STEP: i16 = 0x320;
pub const HIO_DECEL_SCALE: f32 = 0.9;
pub const HIO_MAX_DECEL: f32 = 1.0;
pub const HIO_DECEL: f32 = 1.0;
pub const HIO_WALK_ANM_SPEED_SCALE: f32 = 0.45;
pub const HIO_MAX_WALK_ANM_SPEED: f32 = 0.9;
pub const HIO_NPC_FLY_LAUNCH_SPEED_F: f32 = 50.0;
pub const HIO_NPC_FLY_LAUNCH_SPEED_Y: f32 = 20.0;
pub const HIO_LANDING_BOUNCE: f32 = 8.0;
pub const HIO_HIT_SPEED_SCALE_F: f32 = -1.0;
pub const HIO_HIT_SPEED_SCALE_Y: f32 = 3.5;
pub const HIO_GRAVITY: f32 = -4.0;
pub const HIO_CARRY_X: f32 = -1.5;
pub const HIO_CARRY_Y: f32 = -6.0;
pub const HIO_CARRY_Z: f32 = -7.6;
pub const HIO_STICK_FACE: f32 = 0.1;
pub const HIO_STICK_WALK: f32 = 0.2;
pub const HIO_STICK_WALK_SPEED_SCALE: f32 = 10.0;
pub const HIO_MAX_FLYING_TIMER: i16 = 450;
pub const HIO_PLAYER_FLY_TIMER: i16 = 450;
pub const HIO_LANDING_LIMIT: i16 = 0x64;
pub const HIO_FLY_END_LIMIT: i16 = 0xF;
pub const HIO_FLY_SPEED_MAX: i16 = 0x4E20;
pub const HIO_FLY_SPEED_STOP: i16 = 0x190;
pub const HIO_FLY_SPEED_STEP: i16 = 0x9C4;
pub const HIO_FLY_LAND_STEP: i16 = 0xC8;
pub const HIO_FLY_GRAVITY_SCALE: f32 = 0.0002;
pub const HIO_FLY_MAX_FALL: f32 = -15.0;
pub const HIO_FLY_Y_SPEED: f32 = 10.0;
pub const HIO_FLY_XZ_SPEED: f32 = 10.0;
pub const HIO_FLY_GRAVITY: f32 = -2.5;
pub const HIO_FLY_FALL_STEP: f32 = 1;
pub const HIO_FLY_TILT_LIMIT: f32 = 100.0;
pub const HIO_FLY_TILT_SCALE: f32 = 6000.0;
pub const HIO_FLY_TILT_MAX: f32 = 14000.0;
pub const HIO_FLY_TURN_STEP: i16 = 0xC8;
pub const HIO_STICK_FLY_SPEED_SCALE: f32 = 0.5;
pub const HIO_FLY_SPEED_RESET: i16 = 0x2968;
pub const HIO_FLY_LIFT: f32 = 34.0;
pub const HIO_FLY_SCALE_STEP: f32 = 2.0;
pub const WALK_REVERSE_ACCEL: f32 = 2.0;
pub const HIO_NPC_FLY_TIMER: i16 = 60;
pub const HIO_FLY_FRAME_LIMIT: i16 = 0xD;
pub const HIO_WIND_POWER_SCALE: f32 = 2.0;
pub const HIO_FLY_LAUNCH_SPEED_Y: f32 = 50.0;
pub const HIO_NPC_CALL_DISTANCE: f32 = 400.0;
pub const HIO_RND_MIN: i16 = 0x5;
pub const HIO_RND_RANGE: i16 = 0x14;
pub const HIO_WANDER_ANGLE: i16 = 0xBB8;
pub const HIO_HIT_SPEED_F: f32 = 20.0;
pub const HIO_DAMAGE_TIMER: i16 = 60;

pub const MAX_FLYING_TIMER: i16 = 450;

pub const ANM_00: u8 = 0x00;
pub const ANM_01: u8 = 0x01;
pub const ANM_02: u8 = 0x02;
pub const ANM_03: u8 = 0x03;
pub const ANM_04: u8 = 0x04;
pub const ANM_05: u8 = 0x05;
pub const ANM_06: u8 = 0x06;
pub const ANM_07: u8 = 0x07;
pub const ANM_08: u8 = 0x08;
pub const ANM_09: u8 = 0x09;
pub const ANM_0A: u8 = 0x0A;
pub const ANM_0B: u8 = 0x0B;
pub const ANM_0C: u8 = 0x0C;
pub const ANM_0D: u8 = 0x0D;
pub const ANM_0E: u8 = 0x0E;
pub const ANM_0F: u8 = 0x0F;
pub const ANM_10: u8 = 0x10;
pub const ANM_11: u8 = 0x11;
pub const ANM_12: u8 = 0x12;
pub const ANM_13: u8 = 0x13;
pub const ANM_14: u8 = 0x14;
pub const ANM_15: u8 = 0x15;
pub const ANM_16: u8 = 0x16;
pub const ANM_17: u8 = 0x17;

pub const ANM_ROWS: usize = 24;

pub const ANM_TABLE: [[i8; 6]; ANM_ROWS] = [
    [0x17, 0x02, 0x08, 0x01, -1, -1],
    [0x1A, 0x02, 0x04, 0x01, -1, -1],
    [0x0C, 0x00, 0x04, 0x01, -1, -1],
    [0x0B, 0x02, 0x04, 0x01, -1, -1],
    [0x10, 0x00, 0x04, 0x01, -1, -1],
    [0x11, 0x02, 0x00, 0x01, -1, -1],
    [0x09, 0x02, 0x0F, 0x01, -1, -1],
    [-0x18, 0x02, 0x10, 0x01, -1, -1],
    [-0x13, 0x02, 0x08, 0x01, -1, -1],
    [-0x12, 0x02, 0x08, 0x01, -1, -1],
    [0x0D, 0x02, 0x08, 0x01, -1, -1],
    [0x15, 0x00, 0x08, 0x01, -1, -1],
    [0x14, 0x02, 0x08, 0x01, -1, -1],
    [-0x18, 0x02, 0x08, 0x01, 0x02, 0x0E],
    [-0x04, 0x00, 0x10, 0x01, -1, 0x0F],
    [-0x18, 0x02, 0x00, 0x01, 0x02, -1],
    [-0x13, 0x00, 0x10, 0x01, -1, 0x11],
    [-0x12, 0x02, 0x00, 0x01, 0x18, 0x12],
    [-0x13, 0x00, 0x00, 0x01, -1, 0x13],
    [-0x18, 0x02, 0x10, 0x01, 0x03, 0x10],
    [0x06, 0x02, 0x08, 0x01, 0x04, 0x00],
    [0x05, 0x00, 0x08, 0x01, -1, 0x00],
    [0x07, 0x00, 0x08, 0x01, -1, 0x17],
    [0x06, 0x02, 0x08, 0x01, -1, -1],
];

pub const ANM_PRM: [i8; ANM_ROWS] = [
    0, 1, 2, 3, 4, 5, 4, 6, 7, 0x10, 8, 0xA, 0xB, 0xC, 0xE, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19,
    0x1A, 0x1B, 0x09,
];

pub const TALK_ANM: [u8; 11] = [
    ANM_00, ANM_08, ANM_0A, ANM_0E, ANM_10, ANM_11, ANM_12, ANM_13, ANM_14, ANM_0D, ANM_17,
];

pub const CUT_NAMES: [&str; 15] = [
    "WAIT",
    "MSG_SET",
    "MSG_END",
    "POS_MOV",
    "OFFSET_LINK",
    "WALK",
    "TO_LINK",
    "TACT",
    "CELLOPLAY",
    "TURN",
    "SOW",
    "SET_ANM",
    "SET_GOAL",
    "WARP",
    "END",
];

pub const EVENT_NAMES: [&str; 5] = [
    "cb_rescue",
    "OPTION_CHAR_END",
    "cb_tact",
    "cb_tactCancel",
    "cb_sow",
];
pub const EVENT_SLOTS: usize = 5;

pub const BIT_1604: u16 = 0x1604;
pub const BIT_1610: u16 = 0x1610;
pub const BIT_1820: u16 = 0x1820;
pub const BIT_1840: u16 = 0x1840;
pub const BIT_1880: u16 = 0x1880;
pub const BIT_1904: u16 = 0x1904;
pub const BIT_2910: u16 = 0x2910;
pub const BIT_2E02: u16 = 0x2E02;

pub const ITEM_MASTER_SWORD_2: u8 = 0x3A;
pub const ITEM_MASTER_SWORD_3: u8 = 0x3E;
pub const ITEM_PEARL_FARORE: u8 = 0x6B;

pub const MSG_BOSS_DIE: u32 = 0x1456;
pub const MSG_FOREST_FIRST: u32 = 0x14C2;
pub const MSG_FOREST_CHAIN_LAST: u32 = 0x14C3;
pub const MSG_FOREST_SOW_END: u32 = 0x14C4;
pub const MSG_FOREST_AFTER: u32 = 0x14C5;
pub const MSG_WATERFALL_FIRST: u32 = 0x1519;
pub const MSG_WATERFALL_CHAIN_LAST: u32 = 0x151D;
pub const MSG_WATERFALL_AFTER: u32 = 0x151F;
pub const MSG_WATERFALL_GIFT: u32 = 0x151E;
pub const MSG_WATERFALL_TACT_FIRST: u32 = 0x1520;
pub const MSG_WATERFALL_TACT_NEXT: u32 = 0x1521;
pub const MSG_WATERFALL_TACT_ALT: u32 = 0x1522;
pub const MSG_TACT_WRONG: u32 = 0x1523;
pub const MSG_TACT_CANCEL: u32 = 0x1525;
pub const MSG_TACT_CORRECT: u32 = 0x1526;
pub const MSG_SHIP_RIDE: u32 = 0x152F;
pub const MSG_TACT_INPUT: u32 = 0x5AC;

pub const SE_CV_CB_LIFT_UP: u32 = 0x48BC;
pub const SE_CV_CB_THROW: u32 = 0x48BD;
pub const SE_CV_CB_DAMAGE_S: u32 = 0x48BE;
pub const SE_CV_CB_DAMAGE: u32 = 0x48BF;
pub const SE_CV_CB_LEAF_OUT: u32 = 0x48C0;
pub const SE_CV_CB_LANDING: u32 = 0x48C1;
pub const SE_CV_CB_LEFT_ALONE: u32 = 0x491A;
pub const SE_CV_CB_TROUBLE: u32 = 0x491C;
pub const SE_CV_CB_FLY_END: u32 = 0x4922;
pub const SE_CM_PRAPELLO_ROLLING: u32 = 0x50BF;
pub const SE_CM_PRAPELLO_OPEN: u32 = 0x58C0;
pub const SE_CM_CB_BOUND: u32 = 0x5945;
pub const SE_CTRL_NPC_TO_LINK: u32 = 0x0886;
pub const BGM_MAKORE_TAKT_8: u32 = 0x8000_005F;

pub const CYL_RADIUS: f32 = 20.0;
pub const CYL_HEIGHT: f32 = 60.0;
pub const CARRY_ANM_SPEED: f32 = 2.0;
pub const CARRY_SET_MORF: f32 = 4.0;
pub const JUMP_LIFT: f32 = 10.0;
pub const JUMP_SPEED: f32 = 4.0;
pub const MUSIC_GRAVITY: f32 = -0.1;
pub const NO_TACT_MUSIC: i32 = -1;
pub const DO_STATUS_NAGERU: u8 = 0x2D;
pub const ACT_STTS_BLANK: u8 = 0x00;
pub const ACT_STTS_FLY: u8 = 0x23;
pub const ACT_STTS_LET_GO: u8 = 0x06;
pub const ACT_STTS_RETURN: u8 = 0x07;
pub const ACT_STTS_HIDDEN: u8 = 0x3E;
pub const ATTENTION_ANGLE_ALL: i32 = 0x10000;
pub const FLY_ATTENTION_RANGE: f32 = 4000.0;
pub const FLY_ANM_ROW: i8 = 0x14;
pub const NO_PLAYER_DIST: f32 = 1.0e12;
pub const NO_FLOOR: f32 = -1.0e7;
pub const WIND_CATCH_SPEED: f32 = 30.0;
pub const WIND_CATCH_LIMIT: f32 = 30.0;

pub const ACT_INIT: i8 = 0;
pub const ACT_EXIT: i8 = -1;

pub const EV_NONE: i16 = -1;
pub const EV_NONE_U8: u8 = 0xFF;
pub const ORDER_OFF: i16 = -1;
pub const ORDER_DRAW: i16 = 0;
pub const ORDER_TO_LINK: i16 = 1;
pub const ORDER_TACT_PRM: i16 = 3;
pub const ORDER_OTHER: i32 = 4;
pub const ORDER_SPEAK: i16 = 5;
pub const ORDER_TALK: i16 = 6;
pub const ORDER_TALK_ITEM: i16 = 7;

pub const NO_STAFF: usize = usize::MAX;
pub const NO_INDEX: i16 = -1;

pub const DEG2S: f32 = 32768.0 / 180.0;
pub const NO_GROUND: f32 = -1.0e7;
pub const IS_ZERO: f32 = 8e-11;
pub const SQUARE_400: f32 = 400.0 * 400.0;
pub const SQUARE_600: f32 = 600.0 * 600.0;

pub const TURN_WALK: i16 = 0x400;
pub const TURN_EVENT: i16 = 0x800;
pub const TURN_SHAPE: i16 = 0x2000;
pub const TURN_FLIP: i16 = 0x8000;
pub const ROUTE_ANGLE: i32 = 0x4000;
pub const BLOCK_PUSH: i16 = 0x4000;
pub const ROUTE_TURN_WALL: i32 = 0x6000;
pub const ROUTE_CROSS_MIN: f32 = 1.0;
pub const ROUTE_JUMP_MIN: f32 = 0.0;
pub const ROUTE_JUMP_MAX: f32 = 80.0;
pub const ROUTE_JUMP_SCALE: f32 = 3.8;
pub const ROUTE_JUMP_NONE: f32 = -1.0;
pub const ROUTE_PROBE: f32 = 80.0;
pub const ROUTE_LIFT: f32 = 100.0;
pub const ROUTE_DROP: f32 = -30.0;
pub const ROUTE_FAR_LIMIT: f32 = 100.0;
pub const ROUTE_CONE: i32 = 0x2000;
pub const STICK_POS_REACH: f32 = 100.0;

pub const JNT_HEAD: usize = 0;
pub const JNT_SPINE: usize = 1;
pub const JNT_PITCH: usize = 0;
pub const JNT_YAW: usize = 1;
pub const JNT_SCALE: i16 = 4;
pub const JNT_MIN_STEP: i32 = 4;
pub const JNT_MIN_ANGLE: i16 = 0x100;
pub const JNT_TURN_MIN: i32 = 4;

pub const TURN_SCALE: i16 = 8;
pub const TURN_MAX_STEP: i16 = 0x2000;
pub const TURN_MIN_STEP: i16 = 0x400;

pub const FREE_SCALE: i16 = 0x19;
pub const FREE_MAX_STEP: i16 = 0x7FFF;
pub const FREE_MIN_STEP: i16 = 1;

pub const FLOOR_COUNT_MAX: i8 = 0x1E;
pub const FALL_ACC_RESET: f32 = -400.0;
pub const GROUND_CODE_INVALID: i32 = 4;
pub const GROUND_CODE_NONE: i32 = 0;
pub const SPECIAL_CODE_NONE: i32 = 1;
pub const ROOM_STATUS_LOADED: u8 = 0x10;
pub const ROOM_NO_RESET: i8 = 0xFF;
pub const ROOM_NO_GONE: i8 = -1;
pub const ROOM_OFF_COUNT_MAX: i8 = 30;
pub const LANDING_LIMIT: f32 = 200.0;

pub const ATTENTION_BONUS_DIST: f32 = 40.0;
pub const ATTENTION_BONUS_ANGLE: i32 = 0x71C;
pub const MUSIC_ATTENTION_RANGE: f32 = 500.0;
pub const SHIP_ATTENTION_RANGE: f32 = 500.0;

pub const WARP_UP_SPEED: f32 = 2.5;
pub const WARP_HOLD_FRAMES: i16 = 0x14;
pub const WARP_DOWN_SPEED: f32 = 20.0;
pub const WARP_DOWN_STEP: f32 = 0.5;
pub const WARP_SCALE_SPEED: f32 = 0.05;
pub const WARP_SCALE_Y: f32 = 4.0;
pub const WARP_SPIN_STEP: i16 = 8000;
pub const WARP_SPIN_RATE: i16 = 400;
pub const WARP_BRACE: f32 = 0.1;
pub const WARP_AIR_STEP: f32 = 0.1;

pub const SHOCK_FRAME: i32 = 4;
pub const SHOCK_MAGNITUDE: i32 = -0x21;

pub const MAP_NONE: u32 = 0x12;
pub const MAP_CB1: u32 = 0xD;
pub const MAP_NPC: u32 = 0x7;

pub const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
pub const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
pub const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
pub const RND_SEED: i32 = 100;

pub const NUT_STIFF: f32 = 14.0;
pub const NUT_DAMP: f32 = -0.15f32;
pub const NUT_GRAVITY: f32 = -1.3f32;
pub const NUT_DRAG: f32 = 5.8f32;
pub const NUT_ORBIT: f32 = 100.0;
pub const NUT_END_FRAME: f32 = 8.0;
pub const NUT_SOW_STOP: i8 = 6;
pub const NUT_SOW_RESET: i8 = 4;

pub const PROPELLER_FRAME: f32 = 11.0;
pub const PROPELLER_LAND_FRAME: f32 = 8.0;
pub const PROPELLER_ROLL_SCALE: f32 = 100.0;
pub const FLY_ANM_END_FRAME: f32 = 6.0;
pub const MORPH_END_FRAME: f32 = 8.0;
pub const ANM_LOOP: i8 = 0x02;

pub const SHIP_TRANS_X: f32 = 10.92;
pub const SHIP_TRANS_Y: f32 = 0.57;
pub const SHIP_TRANS_Z: f32 = -14.0;
pub const SHIP_ROT_X: i16 = -0x3AAA;
pub const SHIP_ROT_Z: i16 = 0x40FE;
pub const BASE_POS_Y: f32 = 25.0;
pub const BASE_POS_DOWN: f32 = -25.0;
pub const PROPELLER_TRANS_Y: f32 = 24.8;
pub const PROPELLER_TRANS_Z: f32 = 1.5;

pub const LAND_BIG_SCALE: f32 = 1.25;
pub const LAND_BIG_Y: f32 = 1.5;
pub const LAND_BIG_Z: f32 = 1.0;
pub const LAND_SMALL_SCALE: f32 = 0.625;
pub const LAND_SMALL_Y: f32 = 0.75;
pub const LAND_SMALL_Z: f32 = 0.5;
pub const LAND_KEY: i32 = 7;

pub const SLOPE_MAX_FALL: f32 = -4.0;
pub const SLOPE_MAX_FALL_SCALE: f32 = 100.0;
pub const SLOPE_LIFT_SPEED: f32 = -40.0;
pub const SLOPE_NORMAL_SCALE: f32 = 40.0;
pub const SLOPE_DRIFT_SCALE: f32 = 2.0;
pub const MOVE_FALL_SPEED: f32 = -100.0;
pub const WIND_CATCH_FLOOR: f32 = 1.0;
pub const WIND_CATCH_SCALE: f32 = 30.0;
pub const WIND_MAX: f32 = 30.0;
pub const WIND_POWER_MIN: f32 = 30.0;
pub const WIND_PUSH_SCALE: f32 = 0.5;
pub const WIND_PUSH_STEP: f32 = 0.5;
pub const WIND_SAMPLE_STEP: f32 = 3.0;

pub const HEAP_SIZE: u32 = 0x9400;
pub const SHOCK_HOLD: f32 = 0.0;
pub const ANM_END_SONG: i8 = 0x13;
pub const ANM_START_SONG: i8 = 0x11;
pub const ANM_KAZE_BOSS_END: i8 = 0x18;
pub const ANM_KAZE_BOSS_ALT: i8 = 0xD;
pub const SOW_CHECK_VALUE: u8 = 0x2D;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    None,
    Wait,
    Search,
    Talk,
    Carry,
    Fly,
    Hit,
    Jump,
    Rescue,
    Music,
    Ship,
    WaitPlayer,
    WalkPlayer,
    HitPlayer,
    JumpPlayer,
    FlyPlayer,
    CarryPlayer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Param {
    Null,
    Marked,
    Speed(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cut {
    Wait,
    MsgSet,
    MsgEnd,
    MovePos,
    OffsetLink,
    Walk,
    ToLink,
    Tact,
    CelloPlay,
    Turn,
    Sow,
    SetAnm,
    SetGoal,
    Warp,
    End,
}

impl Cut {
    pub fn of_name(name: &str) -> Option<Self> {
        let index = CUT_NAMES.iter().position(|cut| *cut == name)?;
        Self::of_index(index)
    }

    fn of_index(index: usize) -> Option<Self> {
        Some(match index {
            0 => Self::Wait,
            1 => Self::MsgSet,
            2 => Self::MsgEnd,
            3 => Self::MovePos,
            4 => Self::OffsetLink,
            5 => Self::Walk,
            6 => Self::ToLink,
            7 => Self::Tact,
            8 => Self::CelloPlay,
            9 => Self::Turn,
            10 => Self::Sow,
            11 => Self::SetAnm,
            12 => Self::SetGoal,
            13 => Self::Warp,
            14 => Self::End,
            _ => return None,
        })
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Makar {
    pub prm: u32,
    pub status: u16,
    pub npc_action: Step,
    pub player_action: Step,
    pub act_step: i8,
    pub act_param: Param,
    pub entered: bool,
    pub anm: u8,
    pub anm_row: i8,
    pub anm_frame: f32,
    pub anm_last: f32,
    pub anm_end: f32,
    pub anm_speed: f32,
    pub anm_last_speed: f32,
    pub anm_loop: bool,
    pub anm_done: bool,
    pub anm_count: i8,
    pub nopause: bool,
    pub attn_count: u8,
    pub attn_seen: bool,
    pub attn_flags: u32,
    pub attn_dist_talk: u32,
    pub attn_dist_speak: u32,
    pub attn_dist_carry: u32,
    pub cyl_radius: f32,
    pub cyl_height: f32,
    pub move_block: i8,
    pub do_status: u8,
    pub ship_atn: Vec3,
    pub changed_player: bool,
    pub head_turn: i16,
    pub angles: [[i16; 2]; 2],
    pub turning: bool,
    pub head_lock: bool,
    pub backbone_lock: bool,
    pub head_y_rest: i16,
    pub spine_y_rest: i16,
    pub head_x_rest: i16,
    pub spine_x_rest: i16,
    pub msg_no: u32,
    pub msg_id: u32,
    pub timer: i16,
    pub ev_order: i16,
    pub ev_slot: i16,
    pub ev_index: [i16; EVENT_SLOTS],
    pub ev_accepted: bool,
    pub ev_demo_accept: bool,
    pub npc_call: bool,
    pub npc_change: bool,
    pub return_link: bool,
    pub talk_xy: bool,
    pub talk_cmd: bool,
    pub rnd_flip: i8,
    pub rnd_timer: i16,
    pub seed: [i32; 3],
    pub carry_angle: i16,
    pub land_timer: i16,
    pub fly_speed: i16,
    pub propeller: i16,
    pub fly_frame: i8,
    pub steer: i8,
    pub fall_sum: f32,
    pub drop: f32,
    pub carry_offset: Vec3,
    pub normal: Vec3,
    pub block: Vec3,
    pub push: Vec3,
    pub eye_pos: Vec3,
    pub nut_pos: Vec3,
    pub nut_speed: Vec3,
    pub nut_frame: f32,
    pub nut_done: bool,
    pub attn_pos: Vec3,
    pub speed: Vec3,
    pub speed_f: f32,
    pub gravity: f32,
    pub max_fall: f32,
    pub pos: Vec3,
    pub old: Vec3,
    pub yaw: i16,
    pub pitch: i16,
    pub roll: i16,
    pub shape_yaw: i16,
    pub shape_pitch: i16,
    pub shape_roll: i16,
    pub scale: Vec3,
    pub home: Vec3,
    pub home_angle: [i16; 3],
    pub home_room: i8,
    pub ground_h: f32,
    pub floor_count: i8,
    pub damage_timer: i16,
    pub flying: bool,
    pub fly_timer: i16,
    pub player_room: bool,
    pub staff_idx: usize,
    pub grounded: bool,
    pub ship_present: bool,
    pub link_present: bool,
    pub as_player: bool,
}

impl Makar {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            prm: parameters & PRM_TYPE_MASK,
            status: 0,
            npc_action: Step::None,
            player_action: Step::None,
            act_step: ACT_INIT,
            act_param: Param::Null,
            entered: false,
            anm: EV_NONE_U8,
            anm_row: 0,
            anm_frame: 0.0,
            anm_last: 0.0,
            anm_end: 0.0,
            anm_speed: 1.0,
            anm_last_speed: 1.0,
            anm_loop: false,
            anm_done: false,
            anm_count: 0,
            nopause: false,
            attn_count: 0,
            attn_seen: false,
            attn_flags: 0,
            attn_dist_talk: ATTN_DIST_TALK,
            attn_dist_speak: ATTN_DIST_SPEAK,
            attn_dist_carry: ATTN_DIST_CARRY,
            cyl_radius: CYL_RADIUS,
            cyl_height: CYL_HEIGHT,
            move_block: 0,
            do_status: 0,
            ship_atn: Vec3::ZERO,
            changed_player: false,
            head_turn: 0,
            angles: [[0; 2]; 2],
            turning: false,
            head_lock: false,
            backbone_lock: false,
            head_y_rest: 0,
            spine_y_rest: 0,
            head_x_rest: 0,
            spine_x_rest: 0,
            msg_no: 0,
            msg_id: FPC_M_ERROR_PROCESS_ID,
            timer: 0,
            ev_order: ORDER_OFF,
            ev_slot: EV_NONE,
            ev_index: [NO_INDEX; EVENT_SLOTS],
            ev_accepted: false,
            ev_demo_accept: false,
            npc_call: false,
            npc_change: false,
            return_link: false,
            talk_xy: false,
            talk_cmd: false,
            rnd_flip: 0,
            rnd_timer: 0,
            seed: [RND_SEED; 3],
            carry_angle: 0,
            land_timer: 0,
            fly_speed: 0,
            propeller: 0,
            fly_frame: 0,
            steer: 0,
            fall_sum: 0.0,
            drop: 0.0,
            carry_offset: Vec3::ZERO,
            normal: Vec3::ZERO,
            block: Vec3::ZERO,
            push: Vec3::ZERO,
            eye_pos: Vec3::ZERO,
            nut_pos: Vec3::ZERO,
            nut_speed: Vec3::ZERO,
            nut_frame: 0.0,
            nut_done: false,
            attn_pos: Vec3::ZERO,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            gravity: 0.0,
            max_fall: MOVE_FALL_SPEED,
            pos: Vec3::ZERO,
            old: Vec3::ZERO,
            yaw: 0,
            pitch: 0,
            roll: 0,
            shape_yaw: 0,
            shape_pitch: 0,
            shape_roll: 0,
            scale: Vec3::ONE,
            home: Vec3::ZERO,
            home_angle: [0; 3],
            home_room: 0,
            ground_h: NO_GROUND,
            floor_count: 0,
            damage_timer: 0,
            flying: false,
            fly_timer: 0,
            player_room: false,
            staff_idx: NO_STAFF,
            grounded: false,
            ship_present: false,
            link_present: false,
            as_player: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (
            [CULL_MIN[0], CULL_MIN[1], CULL_MIN[2]],
            [CULL_MAX[0], CULL_MAX[1], CULL_MAX[2]],
        )
    }

    pub fn drawn(&self) -> bool {
        !(self.ev_order == ORDER_DRAW
            || (self.npc_action == Step::Rescue && self.act_step == ACT_INIT))
    }

    pub fn prm_type(&self) -> u32 {
        self.prm & PRM_TYPE_MASK
    }

    pub fn is_type_boss_die(&self) -> bool {
        self.prm_type() == PRM_TYPE_BOSS_DIE
    }

    pub fn is_type_forest(&self) -> bool {
        self.prm_type() == PRM_TYPE_FOREST
    }

    pub fn is_type_waterfall(&self) -> bool {
        self.prm_type() == PRM_TYPE_WATERFALL
    }

    pub fn is_type_ekaze(&self) -> bool {
        self.prm_type() == PRM_TYPE_EKAZE
    }

    pub fn is_type_kaze(&self) -> bool {
        self.prm_type() == PRM_TYPE_KAZE
    }

    pub fn is_type_kaze_boss(&self) -> bool {
        self.prm_type() == PRM_TYPE_KAZE_BOSS
    }

    pub fn is_no_carry_action(&self) -> bool {
        self.status & STTS_NO_CARRY_ACTION != 0
    }

    pub fn on_no_carry_action(&mut self) {
        self.status |= STTS_NO_CARRY_ACTION;
    }

    pub fn off_no_carry_action(&mut self) {
        self.status &= !STTS_NO_CARRY_ACTION;
    }

    pub fn is_tact(&self) -> bool {
        self.status & STTS_TACT != 0
    }

    pub fn on_tact(&mut self) {
        self.status |= STTS_TACT;
    }

    pub fn off_tact(&mut self) {
        self.status &= !STTS_TACT;
    }

    pub fn is_tact_correct(&self) -> bool {
        self.status & STTS_TACT_CORRECT != 0
    }

    pub fn on_tact_correct(&mut self) {
        self.status |= STTS_TACT_CORRECT;
    }

    pub fn off_tact_correct(&mut self) {
        self.status &= !STTS_TACT_CORRECT;
    }

    pub fn is_tact_cancel(&self) -> bool {
        self.status & STTS_TACT_CANCEL != 0
    }

    pub fn on_tact_cancel(&mut self) {
        self.status |= STTS_TACT_CANCEL;
    }

    pub fn off_tact_cancel(&mut self) {
        self.status &= !STTS_TACT_CANCEL;
    }

    pub fn is_music(&self) -> bool {
        self.status & STTS_MUSIC != 0
    }

    pub fn on_music(&mut self) {
        self.status |= STTS_MUSIC;
    }

    pub fn off_music(&mut self) {
        self.status &= !STTS_MUSIC;
    }

    pub fn is_nut(&self) -> bool {
        self.status & STTS_NUT != 0
    }

    pub fn on_nut(&mut self) {
        self.status |= STTS_NUT;
    }

    pub fn off_nut(&mut self) {
        self.status &= !STTS_NUT;
    }

    pub fn is_ship_ride(&self) -> bool {
        self.status & STTS_SHIP_RIDE != 0
    }

    pub fn on_ship_ride(&mut self) {
        self.status |= STTS_SHIP_RIDE;
    }

    pub fn off_ship_ride(&mut self) {
        self.status &= !STTS_SHIP_RIDE;
    }

    pub fn is_player_find(&self) -> bool {
        self.status & STTS_PLAYER_FIND != 0
    }

    pub fn on_player_find(&mut self) {
        self.status |= STTS_PLAYER_FIND;
    }

    pub fn off_player_find(&mut self) {
        self.status &= !STTS_PLAYER_FIND;
    }

    pub fn is_floor_seen(&self) -> bool {
        self.status & STTS_UNK_0100 != 0
    }

    pub fn on_floor_seen(&mut self) {
        self.status |= STTS_UNK_0100;
    }

    pub fn off_floor_seen(&mut self) {
        self.status &= !STTS_UNK_0100;
    }

    pub fn backbone_x(&self) -> i16 {
        self.angles[JNT_SPINE][JNT_PITCH]
    }

    pub fn backbone_y(&self) -> i16 {
        self.angles[JNT_SPINE][JNT_YAW]
    }

    pub fn head_y(&self) -> i16 {
        self.angles[JNT_HEAD][JNT_YAW]
    }

    pub fn inc_attn_count(&mut self) {
        if self.attn_count != 0xFF {
            self.attn_count = self.attn_count.wrapping_add(1);
        }
    }

    pub fn xy_check_cb(&self, item: u8, stories: &StoryBits) -> bool {
        item == ITEM_WIND_WAKER && stories.test(BIT_1880)
    }

    pub fn xy_event_cb(&self) -> i16 {
        self.ev_index.get(2).copied().unwrap_or(NO_INDEX)
    }
}

const CULL_MIN: [f32; 3] = [-70.0, -100.0, -80.0];
const CULL_MAX: [f32; 3] = [70.0, 240.0, 100.0];

fn is_zero(value: f32) -> bool {
    value.abs() < IS_ZERO
}

fn ssin(angle: i16) -> f32 {
    stream::sin_s(angle)
}

fn scos(angle: i16) -> f32 {
    stream::cos_s(angle)
}

fn atan2s(x: f32, z: f32) -> i16 {
    stream::angle_s(x.atan2(z))
}

fn deg2s(deg: f32) -> i16 {
    (deg * DEG2S) as i32 as i16
}

fn target_angle_y(from: Vec3, to: Vec3) -> i16 {
    atan2s(to.x - from.x, to.z - from.z)
}

fn target_angle_x(from: Vec3, to: Vec3) -> i16 {
    let delta = to - from;
    atan2s(delta.y, (delta.x * delta.x + delta.z * delta.z).sqrt())
}

fn abs_xz(v: Vec3) -> f32 {
    (v.x * v.x + v.z * v.z).sqrt()
}

fn abs2(v: Vec3) -> f32 {
    v.x * v.x + v.y * v.y + v.z * v.z
}

fn abs2_xz(v: Vec3) -> f32 {
    v.x * v.x + v.z * v.z
}

fn max_limit(value: f32, max: f32) -> f32 {
    if value > max { max } else { value }
}

fn min_limit(value: f32, min: f32) -> f32 {
    if value < min { min } else { value }
}

fn distance_angle_s(x: i16, y: i16) -> u16 {
    x.wrapping_sub(y).unsigned_abs()
}

fn chase_s(value: &mut i16, target: i16, step: i16) -> bool {
    walk::chase_angle_s(value, target, step)
}

fn calc_timer(value: &mut i16) -> i16 {
    if *value != 0 {
        *value -= 1;
    }
    *value
}

fn offset_pos(src: Vec3, angle: i16, v: Vec3) -> Vec3 {
    let (sin, cos) = (ssin(angle), scos(angle));
    Vec3::new(
        src.x + (v.x * cos + v.z * sin),
        src.y + v.y,
        src.z + (v.z * cos - v.x * sin),
    )
}

fn add_calc_pos_xz(value: &mut Vec3, target: Vec3, scale: f32, max_step: f32, min_step: f32) -> f32 {
    if value.x != target.x || value.z != target.z {
        let mut diff = *value - target;
        let mut step = abs_xz(diff);
        if step < min_step {
            value.x = target.x;
            value.z = target.z;
        } else {
            step *= scale;
            diff *= scale;
            if !is_zero(step) {
                if step > max_step {
                    diff *= max_step / step;
                } else if step < min_step {
                    diff *= min_step / step;
                }
                value.x -= diff.x;
                value.z -= diff.z;
            } else {
                value.x = target.x;
                value.z = target.z;
            }
        }
    }
    abs_xz(*value - target)
}

fn add_calc_angle_l(value: i32, target: i32, scale: i32, max_step: i32, min_step: i32) -> i32 {
    let diff = target - value;
    let mut value = value;
    if value != target {
        let mut step = diff / scale;
        if step > min_step || step < -min_step {
            step = step.min(max_step).max(-max_step);
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
    value
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

fn get_rnd_value(seed: &mut [i32; 3], min: i16, range: i16) -> i16 {
    ((min) as f32 + rnd_f(seed, (range) as f32)) as i32 as i16
}

fn chk_lim(angle: i16, max: i16, min: i16) -> i16 {
    angle.min(max).max(min)
}

impl Makar {
    fn set_trn(&mut self) {
        self.turning = false;
    }

    fn trn_chk(&self) -> bool {
        self.turning
    }

    fn turn_from_spine_to_head(&mut self, input: i16) -> (i16, i16) {
        let mut spine = 0_i16;
        if !self.backbone_lock {
            spine = chk_lim(input, HIO_MAX_BACKBONE_Y, HIO_MIN_BACKBONE_Y);
            if self.spine_y_rest != 0 && spine < 0 {
                spine = 0;
            }
        }
        let mut head = 0_i16;
        if !self.head_lock {
            head = chk_lim(
                input.wrapping_sub(spine),
                HIO_MAX_HEAD_Y,
                HIO_MIN_HEAD_Y,
            );
        }
        (head, spine)
    }

    fn turn_from_head_to_spine(&mut self, input: i16) -> (i16, i16) {
        let mut head = 0_i16;
        if !self.head_lock {
            head = chk_lim(
                input.wrapping_sub(self.spine_y_rest),
                HIO_MAX_HEAD_Y,
                HIO_MIN_HEAD_Y,
            );
        }
        let mut spine = 0_i16;
        if !self.backbone_lock {
            spine = chk_lim(
                input.wrapping_sub(head),
                HIO_MAX_BACKBONE_Y,
                HIO_MIN_BACKBONE_Y,
            );
        }
        (head, spine)
    }

    fn follow_current(&mut self, current: usize, diff: i16) {
        let before = self.angles[current][JNT_YAW];
        let mut next = before.wrapping_sub(diff);
        if (before > 0 && next < 0) || (before < 0 && next > 0) {
            next = 0;
        }
        self.angles[current][JNT_YAW] = next;
    }

    pub fn look_at_target(
        &mut self,
        out_init: i16,
        dst: Option<Vec3>,
        src: Vec3,
        default_y: i16,
        max_vel: i16,
        no_follow: bool,
    ) -> i16 {
        let (target_y, target_x) = match dst {
            Some(point) => (target_angle_y(src, point), target_angle_x(src, point)),
            None => (default_y, 0),
        };
        let mut out = out_init;
        let delta_y = target_y.wrapping_sub(out);
        let (head, spine) = if self.spine_y_rest >= 0 {
            if delta_y >= self.spine_y_rest || self.spine_y_rest == 0 {
                self.turn_from_head_to_spine(delta_y)
            } else {
                self.turn_from_spine_to_head(delta_y)
            }
        } else if delta_y <= self.spine_y_rest || self.spine_y_rest == 0 {
            self.turn_from_head_to_spine(delta_y)
        } else {
            self.turn_from_spine_to_head(delta_y)
        };
        self.head_y_rest = head;
        self.spine_y_rest = spine;
        for (row, want) in [(JNT_HEAD, head), (JNT_SPINE, spine)] {
            self.angles[row][JNT_YAW] = add_calc_angle_l(
                i32::from(self.angles[row][JNT_YAW]),
                i32::from(want),
                i32::from(JNT_SCALE),
                i32::from(HIO_MAX_TURN_STEP),
                JNT_MIN_STEP,
            ) as i16;
        }
        if self.turning && !no_follow {
            let before = out;
            out = walk::add_calc_angle_s(out, target_y, JNT_SCALE, max_vel, JNT_MIN_ANGLE);
            let diff = out.wrapping_sub(before);
            self.turning = diff != 0;
            if !self.backbone_lock {
                self.follow_current(JNT_SPINE, diff);
            }
            if !self.head_lock {
                self.follow_current(JNT_HEAD, diff);
            }
        } else {
            let total = self.spine_y_rest.wrapping_add(self.head_y_rest);
            self.turning = if delta_y >= 0 {
                delta_y > total
            } else {
                delta_y < total
            };
        }
        let head_x = chk_lim(target_x, HIO_MAX_HEAD_X, HIO_MIN_HEAD_X);
        let spine_x = chk_lim(
            target_x.wrapping_sub(head_x),
            HIO_MAX_BACKBONE_X,
            HIO_MIN_BACKBONE_X,
        );
        self.head_x_rest = head_x;
        self.spine_x_rest = spine_x;
        for (row, want) in [(JNT_HEAD, head_x), (JNT_SPINE, spine_x)] {
            self.angles[row][JNT_PITCH] = add_calc_angle_l(
                i32::from(self.angles[row][JNT_PITCH]),
                i32::from(want),
                i32::from(JNT_SCALE),
                i32::from(HIO_MAX_TURN_STEP),
                JNT_MIN_STEP,
            ) as i16;
        }
        out
    }

    pub fn look_back(&mut self, watch: bool, player: Option<Vec3>) {
        let desired = self.shape_yaw;
        if self.trn_chk() {
            stream::add_calc_angle_s2(
                &mut self.head_turn,
                HIO_MAX_HEAD_TURN_VEL,
                JNT_SCALE,
                HIO_MAX_HEAD_TURN_VEL,
            );
        } else {
            self.head_turn = 0;
        }
        let mut dst = None;
        let mut src = Vec3::ZERO;
        if watch {
            if let Some(at) = player {
                let mut point = at;
                point.y += HIO_EYE_OFFSET;
                dst = Some(point);
            }
            src = self.pos;
            src.y = self.eye_pos.y;
        }
        self.shape_yaw = self.look_at_target(self.shape_yaw, dst, src, desired, self.head_turn, false);
    }
}

impl Makar {
    pub fn get_anm_type(&self, index: i32) -> u8 {
        if index >= 0 {
            if let Some(anm) = usize::try_from(index)
                .ok()
                .and_then(|at| TALK_ANM.get(at).copied())
            {
                return anm;
            }
        }
        TALK_ANM[0]
    }

    pub fn init_anm(&mut self, row: i8, secondary: bool) {
        if row < 0 {
            return;
        }
        self.anm_row = row;
        let index = usize::try_from(row).unwrap_or(0);
        let data = ANM_TABLE.get(index).copied().unwrap_or(ANM_TABLE[0]);
        self.anm_frame = 0.0;
        self.anm_last = 0.0;
        self.anm_end = MORPH_END_FRAME;
        self.anm_speed = (data[3]) as f32.max(1.0);
        self.anm_loop = data[1] == ANM_LOOP;
        let _ = (data[0].unsigned_abs(), if secondary { data[2] } else { 0 }, data[5]);
        self.anm_done = false;
        self.anm_count = 0;
        if data[4] < 0 {
            self.on_music();
        } else {
            self.off_music();
        }
    }

    pub fn set_anm(&mut self, anm: u8, ctx: &mut Ctx) -> bool {
        if self.anm == anm {
            return false;
        }
        if self.anm == ANM_09 {
            self.music_stop(ctx);
        }
        self.anm = anm;
        self.nopause = self.anm == ANM_09;
        let index = usize::from(self.anm);
        let prm = ANM_PRM.get(index).copied().unwrap_or(ANM_PRM[0]);
        self.init_anm(prm, true);
        true
    }

    pub fn play_anm(&mut self, ctx: &mut Ctx) {
        if self.anm_end > 0.0 && self.anm_frame >= self.anm_end {
            if self.anm_loop {
                self.anm_frame = 0.0;
                self.anm_count += 1;
                self.anm_done = true;
            }
        } else if self.anm_end > 0.0 {
            self.anm_frame += self.anm_speed;
        }
        if self.anm_frame < self.anm_last {
            self.anm_count += 1;
            self.anm_done = true;
        }
        if self.anm_done {
            let index = usize::try_from(self.anm_row).unwrap_or(0);
            let data = ANM_TABLE.get(index).copied().unwrap_or(ANM_TABLE[0]);
            let mut limit = data[5];
            if self.is_type_kaze_boss() && limit == ANM_KAZE_BOSS_END {
                limit = ANM_KAZE_BOSS_ALT;
            }
            if self.anm_count >= limit {
                if data[5] == ANM_END_SONG {
                    self.music_stop(ctx);
                } else if data[5] == ANM_START_SONG {
                    self.music_play(ctx);
                }
                self.init_anm(data[5], true);
            }
        }
        self.anm_last = self.anm_frame;
        self.nut_spring();
    }

    fn nut_spring(&mut self) {
        if !self.is_nut() {
            return;
        }
        self.nut_frame += 1.0;
        if self.nut_frame >= NUT_END_FRAME {
            self.nut_done = true;
        }
        let joint = Vec3::new(
            ssin(self.propeller) * NUT_ORBIT,
            0.0,
            scos(self.propeller) * NUT_ORBIT,
        );
        let mut delta = self.nut_pos - joint;
        let gap = delta.length();
        if !is_zero(gap) {
            delta *= NUT_STIFF / gap;
        }
        self.nut_pos = joint + delta;
        self.nut_speed += delta * NUT_DAMP;
        self.nut_speed.y += NUT_GRAVITY;
        let drag = self.nut_speed.length();
        if !is_zero(drag) {
            self.nut_speed *= NUT_DRAG / drag;
        }
    }

    fn music_play(&mut self, ctx: &mut Ctx) {
        if self.is_type_kaze_boss() {
            ctx.events.list.push(SimEvent::Bgm(BGM_MAKORE_TAKT_8));
        }
    }

    fn music_stop(&mut self, ctx: &mut Ctx) {
        let _ = ctx;
    }
}

impl Makar {
    pub fn set_npc_action(&mut self, next: Step, param: Param, ctx: &mut Ctx) {
        self.player_action = Step::None;
        self.swap(true, next, param, ctx);
    }

    pub fn set_player_action(&mut self, next: Step, ctx: &mut Ctx) {
        self.npc_action = Step::None;
        self.swap(false, next, Param::Null, ctx);
    }

    fn swap(&mut self, npc: bool, next: Step, param: Param, ctx: &mut Ctx) {
        let current = if npc {
            self.npc_action
        } else {
            self.player_action
        };
        self.fly_timer = 0;
        self.off_flying();
        if current == next {
            return;
        }
        self.act_param = param;
        if current != Step::None {
            self.act_step = ACT_EXIT;
            self.exit_action(current, ctx);
        }
        if npc {
            self.npc_action = next;
        } else {
            self.player_action = next;
        }
        self.act_step = ACT_INIT;
        self.carry_angle = 0;
        self.land_timer = 0;
        self.fly_speed = 0;
        self.propeller = 0;
        self.fall_sum = 0.0;
        self.fly_frame = 0;
        self.steer = 0;
        self.shape_pitch = 0;
        self.shape_roll = 0;
        self.attn_flags = 0;
        if npc {
            self.run_npc(ctx);
        } else {
            self.run_player(ctx);
        }
        self.act_step += 1;
    }

    fn exit_action(&mut self, action: Step, ctx: &mut Ctx) {
        let _ = ctx;
        if action == Step::Carry {
            self.carry_offset.y = 0.0;
            self.carry_offset.z = 0.0;
        }
    }

    fn run_npc(&mut self, ctx: &mut Ctx) {
        match self.npc_action {
            Step::Wait => self.wait_npc_action(ctx),
            Step::Search => self.search_npc_action(ctx),
            Step::Talk => self.talk_npc_action(ctx),
            Step::Carry => self.carry_npc_action(ctx),
            Step::Fly => self.fly_npc_action(ctx),
            Step::Hit => self.hit_npc_action(ctx),
            Step::Jump => self.jump_npc_action(ctx),
            Step::Rescue => self.rescue_npc_action(ctx),
            Step::Music => self.music_npc_action(ctx),
            Step::Ship => self.ship_npc_action(ctx),
            _ => {}
        }
    }

    fn run_player(&mut self, ctx: &mut Ctx) {
        match self.player_action {
            Step::WaitPlayer => self.wait_player_action(ctx),
            Step::WalkPlayer => self.walk_player_action(ctx),
            Step::HitPlayer => self.hit_player_action(ctx),
            Step::JumpPlayer => self.jump_player_action(ctx),
            Step::FlyPlayer => self.fly_player_action(ctx),
            Step::CarryPlayer => self.carry_player_action(ctx),
            _ => {}
        }
    }

    pub fn set_wait_npc_action(&mut self, ctx: &mut Ctx) {
        if self.is_ship_ride() {
            self.set_npc_action(Step::Ship, Param::Null, ctx);
        } else if self.is_music() {
            self.set_npc_action(Step::Music, Param::Null, ctx);
        } else {
            self.set_npc_action(Step::Wait, Param::Null, ctx);
        }
    }

    pub fn set_wait_action(&mut self, ctx: &mut Ctx) {
        if self.as_player {
            self.set_player_action(Step::WaitPlayer, ctx);
        } else {
            self.set_wait_npc_action(ctx);
        }
    }

    pub fn npc_action(&mut self, ctx: &mut Ctx) {
        if self.npc_action == Step::None {
            self.speed_f = 0.0;
            self.set_wait_npc_action(ctx);
        }
        self.run_npc(ctx);
    }

    pub fn player_action(&mut self, ctx: &mut Ctx) {
        if self.player_action == Step::None {
            self.speed_f = 0.0;
            self.set_player_action(Step::WaitPlayer, ctx);
        }
        self.run_player(ctx);
    }

    pub fn is_fly_action(&self) -> bool {
        self.player_action == Step::FlyPlayer || self.npc_action == Step::Fly
    }

    pub fn carry_check(&mut self, ctx: &mut Ctx) {
        if ctx.carry_now {
            self.set_npc_action(Step::Carry, Param::Null, ctx);
        }
    }

    pub fn off_flying(&mut self) {
        self.flying = false;
    }

    pub fn on_flying(&mut self) {
        self.flying = true;
    }

    pub fn breaking(&mut self) {
        walk::add_calc(
            &mut self.speed_f,
            0.0,
            HIO_DECEL_SCALE,
            HIO_MAX_DECEL,
            HIO_DECEL,
        );
    }

    pub fn calc_flying_timer(&mut self) -> bool {
        if self.fly_timer != 0 && calc_timer(&mut self.fly_timer) != 0 {
            return true;
        }
        false
    }

    pub fn chk_attention(&mut self, player: Vec3, dist: f32, angle: i32) -> bool {
        let delta = Vec3::new(player.x - self.pos.x, 0.0, player.z - self.pos.z);
        let gap = abs_xz(delta);
        let mut want = atan2s(delta.x, delta.z);
        let mut reach = dist;
        let mut cone = angle;
        if self.attn_seen {
            reach += ATTENTION_BONUS_DIST;
            cone += ATTENTION_BONUS_ANGLE;
        }
        let head = self
            .shape_yaw
            .wrapping_add(self.head_y())
            .wrapping_add(self.backbone_y());
        want = want.wrapping_sub(head);
        cone > i32::from(want.unsigned_abs()) && reach > gap
    }

    pub fn check_landing(&mut self, ctx: &mut Ctx) {
        if self.drop > LANDING_LIMIT {
            if ctx.special_code != SPECIAL_CODE_NONE {
                ctx.events.list.push(SimEvent::Se(SE_CV_CB_LANDING));
            }
        }
    }

    pub fn walk_action(&mut self, target_speed: f32, accel: f32, target_angle: i16, ctx: &mut Ctx) -> bool {
        walk::chase_angle_s(&mut self.yaw, target_angle, TURN_WALK);
        self.look_back(true, ctx.player);
        self.shape_yaw = self.yaw;
        if walk::chase_f(&mut self.speed_f, target_speed, accel) && target_speed == 0.0 {
            self.set_npc_action(Step::Wait, Param::Null, ctx);
            return true;
        }
        let play = min_limit(self.speed_f.abs() * HIO_WALK_ANM_SPEED_SCALE, HIO_MAX_WALK_ANM_SPEED);
        self.anm_speed = play;
        false
    }

    pub fn return_link_player(&mut self, ctx: &mut Ctx) {
        self.changed_player = true;
        self.off_flying();
        self.set_wait_npc_action(ctx);
    }
}

impl Makar {
    fn wait_npc_action(&mut self, ctx: &mut Ctx) {
        if self.act_step == ACT_INIT {
            if ctx.stories.test(BIT_1610) {
                self.attn_flags |= ATTN_ACTION_CARRY;
            }
            let mut anm = ANM_00;
            if !matches!(self.act_param, Param::Null) {
                if self.is_player_find() {
                    anm = ANM_0F;
                    ctx.events.list.push(SimEvent::Se(SE_CV_CB_LEFT_ALONE));
                } else {
                    anm = ANM_14;
                    ctx.events.list.push(SimEvent::Se(SE_CV_CB_TROUBLE));
                }
            }
            self.set_anm(anm, ctx);
            self.off_player_find();
            self.speed_f = 0.0;
        } else if self.act_step != ACT_EXIT {
            if self.move_block != 0 {
                self.set_npc_action(Step::Search, Param::Null, ctx);
            } else {
                let dist = if self.anm_row == FLY_ANM_ROW {
                    FLY_ATTENTION_RANGE
                } else {
                    HIO_MAX_ATTN_DIST_XZ
                };
                self.attn_seen = self.chk_attention(
                    ctx.player.unwrap_or(Vec3::ZERO),
                    dist,
                    ATTENTION_ANGLE_ALL,
                );
                let mut watch = self.attn_seen;
                if watch && self.get_msg(ctx) != 0 {
                    self.ev_order = ORDER_TALK;
                    self.attn_flags |= ATTN_SPEAK_TALK_MASK;
                } else {
                    self.attn_flags &= !ATTN_SPEAK_TALK_MASK;
                }
                let dist_sq = match ctx.player {
                    Some(at) => abs2(at - self.pos),
                    None => NO_PLAYER_DIST,
                };
                if !self.npc_call {
                    if ctx.stories.test(BIT_1610) && dist_sq < SQUARE_400 {
                        watch = true;
                        watch = true;
                    }
                } else {
                    if dist_sq >= HIO_PLAYER_CHASE_DISTANCE * HIO_PLAYER_CHASE_DISTANCE {
                        self.set_npc_action(Step::Search, Param::Null, ctx);
                    }
                    watch = true;
                }
                self.look_back(watch, ctx.player);
                self.yaw = self.shape_yaw;
            }
        }
    }

    fn talk_npc_action(&mut self, ctx: &mut Ctx) {
        if self.act_step == ACT_INIT {
            self.msg_id = FPC_M_ERROR_PROCESS_ID;
            self.msg_no = self.get_msg(ctx);
            self.attn_flags &= !ATTN_ACTION_CARRY;
            if !self.is_music() {
                let anm = if self.is_ship_ride() { ANM_0D } else { ANM_00 };
                self.set_anm(anm, ctx);
            }
        } else if self.act_step != ACT_EXIT {
            self.attn_seen = true;
            if self.act_step == 1 {
                if self.init_talk(ctx) {
                    self.act_step = 2;
                }
            } else if self.act_step == 2 && self.exec_talk(false, ctx) {
                self.set_wait_npc_action(ctx);
                ctx.reset_event();
            }
            if self.is_ship_ride() {
                if let (Some(_ship), Some(link)) = (ctx.ship_pos, ctx.link) {
                    self.ship_atn = self.pos - (link - self.pos);
                }
            } else {
                self.set_trn();
                self.look_back(true, ctx.player);
            }
        }
    }

    fn carry_npc_action(&mut self, ctx: &mut Ctx) {
        if self.act_step == ACT_INIT {
            self.attn_flags &= !ATTN_ACTION_CARRY;
            self.npc_call = false;
            self.carry_angle = self.to_player_shape_angle(ctx);
            if self.is_no_carry_action() {
                ctx.events.list.push(SimEvent::Se(SE_CV_CB_DAMAGE));
                self.set_anm(ANM_0B, ctx);
            } else {
                ctx.events.list.push(SimEvent::Se(SE_CV_CB_LIFT_UP));
                self.set_anm(ANM_03, ctx);
            }
            self.npc_change = true;
        } else if self.act_step == ACT_EXIT {
            self.carry_offset.y = 0.0;
            self.carry_offset.z = 0.0;
            self.off_no_carry_action();
            self.npc_change = false;
        } else {
            if self.is_no_carry_action() {
                self.anm_speed = CARRY_ANM_SPEED;
            } else {
                if ctx.player_speed == 0.0 && ctx.player_anim <= ctx.player_anim_end {
                    self.anm_frame = ctx.player_anim;
                    self.anm_speed = 0.0;
                } else {
                    self.anm_speed = 1.0;
                }
                if self.anm_speed != self.anm_last_speed {
                    self.anm_frame = CARRY_SET_MORF;
                }
                self.anm_last_speed = self.anm_speed;
                walk::chase_f(&mut self.carry_offset.x, HIO_CARRY_X, 1.0);
                walk::chase_f(&mut self.carry_offset.y, HIO_CARRY_Y, 1.0);
                walk::chase_f(&mut self.carry_offset.z, HIO_CARRY_Z, 1.0);
                walk::chase_angle_s(&mut self.carry_angle, 0, TURN_EVENT);
                self.shape_yaw = ctx.player_shape_yaw.wrapping_add(self.carry_angle);
            }
            if !ctx.carry_now {
                if self.speed_f > 0.0 {
                    self.set_npc_action(Step::Fly, Param::Null, ctx);
                } else {
                    self.set_npc_action(Step::Wait, Param::Null, ctx);
                }
            }
            self.shape_yaw =
                self.look_at_target(self.shape_yaw, None, Vec3::ZERO, self.shape_yaw, 0, false);
            self.yaw = self.shape_yaw;
        }
    }

    fn fly_frame_bit(&self) -> bool {
        self.fly_frame != 0
    }

    fn fly_npc_action(&mut self, ctx: &mut Ctx) {
        let step = self.act_step;
        if step == ACT_INIT {
            self.speed_f = HIO_NPC_FLY_LAUNCH_SPEED_F;
            self.speed.y = HIO_NPC_FLY_LAUNCH_SPEED_Y;
            self.set_anm(ANM_04, ctx);
            self.fly_timer = HIO_NPC_FLY_TIMER;
            self.fall_sum = (HIO_FLY_FALL_STEP) as f32 + 1.0;
            ctx.events.list.push(SimEvent::Se(SE_CV_CB_THROW));
        } else if step != ACT_EXIT {
            self.fly_action(self.steer == 0, 0.0, 0, false, ctx);
            if step != 1 && HIO_FLY_FRAME_LIMIT != 0 {
                let limit = i32::from(HIO_FLY_FRAME_LIMIT);
                self.steer = (((i32::from(self.steer) + 1) % limit) as i8);
            }
        }
    }

    fn hit_npc_action(&mut self, ctx: &mut Ctx) {
        if self.act_step == ACT_INIT {
            self.attn_flags &= !ATTN_ACTION_CARRY;
            self.speed_f = max_limit(self.speed_f, HIO_MAX_WALK_SPEED * 0.5);
            self.speed.y = HIO_HIT_SPEED_SCALE_Y * self.speed_f.abs();
            self.speed_f *= HIO_HIT_SPEED_SCALE_F;
            ctx.events.list.push(SimEvent::Se(SE_CM_CB_BOUND));
            self.set_anm(ANM_02, ctx);
        } else if self.act_step != ACT_EXIT && self.grounded {
            self.speed_f = 0.0;
            self.set_npc_action(Step::Wait, Param::Null, ctx);
        }
    }

    fn jump_npc_action(&mut self, ctx: &mut Ctx) {
        if self.act_step == ACT_INIT {
            self.speed.y = match self.act_param {
                Param::Speed(value) => value,
                _ => JUMP_LIFT,
            };
            self.speed_f = JUMP_SPEED;
            self.drop = 0.0;
            self.set_anm(ANM_16, ctx);
        } else if self.act_step != ACT_EXIT && self.grounded {
            self.check_landing(ctx);
            self.speed_f = 0.0;
            self.set_npc_action(Step::Wait, Param::Null, ctx);
        }
    }

    fn rescue_npc_action(&mut self, ctx: &mut Ctx) {
        if self.act_step != ACT_EXIT {
            if self.act_step == ACT_INIT {
                self.set_anm(ANM_00, ctx);
                self.ev_order = ORDER_DRAW;
            }
            self.ev_order = ORDER_DRAW;
        }
    }

    fn music_npc_action(&mut self, ctx: &mut Ctx) {
        if self.act_step == ACT_INIT {
            self.set_anm(ANM_09, ctx);
        } else if self.act_step != ACT_EXIT && !self.is_type_kaze_boss() {
            self.shape_yaw = walk::add_calc_angle_s(
                self.shape_yaw,
                self.yaw,
                TURN_SCALE,
                TURN_MAX_STEP,
                TURN_MIN_STEP,
            );
            self.yaw = self.shape_yaw;
            self.attn_seen =
                self.chk_attention(ctx.player.unwrap_or(Vec3::ZERO), HIO_MAX_ATTN_DIST_XZ, ATTENTION_ANGLE_ALL);
            if self.attn_seen {
                if self.is_type_waterfall() {
                    self.ev_order = ORDER_TALK_ITEM;
                } else {
                    self.ev_order = ORDER_TALK;
                }
                self.attn_flags |= ATTN_SPEAK_TALK_MASK;
            } else {
                self.attn_flags &= !ATTN_SPEAK_TALK_MASK;
            }
        }
    }

    fn ship_npc_action(&mut self, ctx: &mut Ctx) {
        if self.act_step == ACT_INIT {
            self.npc_change = true;
            self.set_anm(ANM_0D, ctx);
            self.speed_f = 0.0;
        } else if self.act_step != ACT_EXIT {
            if !ctx.stories.test(BIT_1604) {
                self.set_npc_action(Step::Wait, Param::Null, ctx);
                self.off_ship_ride();
            } else {
                self.attn_seen = self.chk_attention(
                    ctx.player.unwrap_or(Vec3::ZERO),
                    SHIP_ATTENTION_RANGE,
                    HIO_MAX_ATTN_ANGLE_Y,
                );
                if self.attn_seen && !ctx.player_ship_ride && self.get_msg(ctx) != 0 {
                    self.ev_order = ORDER_TALK;
                    self.attn_flags |= ATTN_SPEAK_TALK_MASK;
                } else {
                    self.attn_flags &= !ATTN_SPEAK_TALK_MASK;
                }
            }
        }
    }
}

impl Makar {
    fn search_npc_action(&mut self, ctx: &mut Ctx) {
        if self.act_step == ACT_INIT {
            self.attn_flags |= ATTN_ACTION_CARRY;
            self.set_anm(ANM_01, ctx);
        } else if self.act_step != ACT_EXIT {
            let mut walk_speed = 0.0_f32;
            let mut angle;
            if self.move_block != 0 {
                let push: i16 = if self.move_block < 0 {
                    -BLOCK_PUSH
                } else {
                    BLOCK_PUSH
                };
                angle = push.wrapping_add(atan2s(self.block.x, self.block.z));
            } else if !self.npc_call {
                if self.speed_f == 0.0 {
                    self.set_npc_action(Step::Wait, Param::Null, ctx);
                    return;
                }
                angle = self.yaw;
            } else {
                self.attn_seen = true;
                let player = ctx.player.unwrap_or(Vec3::ZERO);
                let door = ctx.player_door;
                let delta = player - self.pos;
                let dist_sq = abs2(delta);
                let dist_xz_sq = abs2_xz(delta);
                if !door && dist_sq < SQUARE_400 {
                    walk_speed = 0.0;
                } else {
                    walk_speed = max_limit(
                        HIO_CHASE_DIST_SCALE * dist_xz_sq.sqrt(),
                        HIO_MAX_WALK_SPEED,
                    );
                }
                angle = target_angle_y(self.pos, player);
                let blocked = door
                    || !self.route_check(dist_xz_sq, &mut angle, ctx)
                    || ctx.player_hang
                    || ctx.player_lock;
                if blocked {
                    walk_speed = 0.0;
                    if self.speed_f == 0.0 {
                        self.npc_call = false;
                        self.set_npc_action(Step::Wait, Param::Marked, ctx);
                        return;
                    }
                    angle = self.yaw;
                }
                if dist_xz_sq < SQUARE_400
                    && i32::from(distance_angle_s(self.shape_yaw, angle)) < ROUTE_CONE
                    && delta.y.abs() < ROUTE_FAR_LIMIT
                {
                    self.on_player_find();
                }
                if calc_timer(&mut self.rnd_timer) == 0 {
                    self.rnd_flip ^= 1;
                    self.rnd_timer = get_rnd_value(
                        &mut self.seed,
                        HIO_RND_MIN,
                        HIO_RND_RANGE,
                    );
                }
                angle = if self.rnd_flip != 0 {
                    angle.wrapping_sub(HIO_WANDER_ANGLE)
                } else {
                    angle.wrapping_add(HIO_WANDER_ANGLE)
                };
            }
            if ctx.co_hit {
                self.set_npc_action(Step::Hit, Param::Null, ctx);
            } else {
                self.walk_action(walk_speed, HIO_FORWARD_ACCEL, angle, ctx);
            }
        }
    }

    fn wait_player_action(&mut self, ctx: &mut Ctx) {
        if self.act_step == ACT_INIT {
            self.set_anm(ANM_00, ctx);
            self.attn_flags |= u32::MAX;
        } else if self.act_step != ACT_EXIT && !self.sow_check(ctx) {
            if ctx.stick >= HIO_STICK_FACE || ctx.player_lock_on {
                let target = self.get_stick_angle(ctx);
                self.yaw = walk::add_calc_angle_s(
                    self.yaw,
                    target,
                    FREE_SCALE,
                    FREE_MAX_STEP,
                    FREE_MIN_STEP,
                );
                let (stick_pos, aim) = self.calc_stick_pos(target, ctx);
                if stick_pos == 0 || ctx.stick >= HIO_STICK_WALK {
                    self.shape_yaw = self.yaw;
                }
                let turned = self.look_at_target(
                    self.shape_yaw,
                    Some(aim),
                    self.pos,
                    self.shape_yaw,
                    HIO_MAX_TURN_STEP,
                    false,
                );
                if stick_pos > 0 {
                    self.shape_yaw = turned;
                }
                self.yaw = self.shape_yaw;
                if ctx.stick >= HIO_STICK_WALK && stick_pos == 0 {
                    self.yaw = target;
                    self.set_player_action(Step::WalkPlayer, ctx);
                }
            } else {
                self.shape_yaw = self.look_at_target(
                    self.shape_yaw,
                    None,
                    Vec3::ZERO,
                    self.shape_yaw,
                    0,
                    false,
                );
                self.yaw = self.shape_yaw;
            }
            self.breaking();
        }
    }

    fn walk_player_action(&mut self, ctx: &mut Ctx) {
        if self.act_step == ACT_INIT {
            self.set_anm(ANM_01, ctx);
            self.attn_flags |= u32::MAX;
        } else if self.act_step != ACT_EXIT && !self.sow_check(ctx) {
            let mut target = ctx.stick * HIO_STICK_WALK_SPEED_SCALE;
            let mut aim_angle = self.get_stick_angle(ctx);
            let gap = distance_angle_s(aim_angle, self.yaw);
            let mut accel = HIO_FORWARD_ACCEL;
            if target > self.speed_f {
                if target < HIO_MIN_WALK_SPEED {
                    target = HIO_MIN_WALK_SPEED;
                }
            } else {
                accel = HIO_DECEL;
            }
            if i32::from(gap) > ROUTE_TURN_WALL {
                target = 0.0;
                if self.speed_f != 0.0 {
                    aim_angle = self.yaw;
                }
                accel = WALK_REVERSE_ACCEL;
            }
            self.yaw = walk::add_calc_angle_s(
                self.yaw,
                aim_angle,
                HIO_WALK_SCALE,
                HIO_WALK_MAX_STEP,
                HIO_WALK_MIN_STEP,
            );
            let (stick_pos, aim) = self.calc_stick_pos(aim_angle, ctx);
            if stick_pos == 0 {
                self.shape_yaw = walk::add_calc_angle_s(
                    self.shape_yaw,
                    self.yaw,
                    TURN_SCALE,
                    TURN_MAX_STEP,
                    TURN_MIN_STEP,
                );
            }
            let turned = self.look_at_target(
                self.shape_yaw,
                Some(aim),
                self.pos,
                self.shape_yaw,
                HIO_MAX_TURN_STEP,
                false,
            );
            if stick_pos > 0 {
                self.shape_yaw = turned;
            }
            target = max_limit(target, HIO_MAX_WALK_SPEED);
            if walk::chase_f(&mut self.speed_f, target, accel) && target == 0.0 {
                self.set_player_action(Step::WaitPlayer, ctx);
            } else {
                self.anm_speed =
                    min_limit(self.speed_f * HIO_WALK_ANM_SPEED_SCALE, HIO_MAX_WALK_ANM_SPEED);
                if !self.grounded {
                    self.set_player_action(Step::JumpPlayer, ctx);
                } else if self.chk_wall_hit(ctx) {
                    let jump = self.check_wall_jump(self.yaw, ctx);
                    if jump >= 0.0 {
                        self.set_player_action(Step::JumpPlayer, ctx);
                        self.speed.y = jump;
                    }
                }
            }
        }
    }

    fn hit_player_action(&mut self, ctx: &mut Ctx) {
        if self.act_step == ACT_INIT {
            self.speed_f = max_limit(self.speed_f, 0.5 * HIO_MAX_WALK_SPEED);
            self.speed.y = HIO_HIT_SPEED_SCALE_Y * self.speed_f.abs();
            self.speed_f *= HIO_HIT_SPEED_SCALE_F;
            ctx.events.list.push(SimEvent::Se(SE_CM_CB_BOUND));
            self.set_anm(ANM_02, ctx);
        } else if self.act_step != ACT_EXIT && self.grounded {
            self.return_link = true;
            self.speed_f = 0.0;
        }
    }

    fn jump_player_action(&mut self, ctx: &mut Ctx) {
        if self.act_step == ACT_INIT {
            self.speed.y = match self.act_param {
                Param::Speed(value) => value,
                _ => JUMP_LIFT,
            };
            self.drop = 0.0;
            self.set_anm(ANM_16, ctx);
        } else if self.act_step != ACT_EXIT && !self.fly_check(ctx) && self.grounded {
            self.check_landing(ctx);
            self.speed_f = 0.0;
            self.set_player_action(Step::WaitPlayer, ctx);
        }
    }

    fn carry_player_action(&mut self, ctx: &mut Ctx) {
        if self.act_step == ACT_INIT {
            self.set_anm(ANM_0B, ctx);
            ctx.events.list.push(SimEvent::Se(SE_CV_CB_DAMAGE_S));
        }
    }

    fn fly_player_action(&mut self, ctx: &mut Ctx) {
        if self.act_step == ACT_INIT {
            self.speed.y = HIO_FLY_LAUNCH_SPEED_Y;
            self.set_anm(ANM_04, ctx);
            self.fly_timer = HIO_PLAYER_FLY_TIMER;
            ctx.events.list.push(SimEvent::Se(SE_CV_CB_LEAF_OUT));
        } else if self.act_step != ACT_EXIT {
            let speed = ctx.stick * HIO_STICK_FLY_SPEED_SCALE;
            let angle = self.get_stick_angle(ctx);
            self.fly_action(ctx.trig_a, speed, angle, ctx.trig_b, ctx);
        }
    }

    fn fly_check(&mut self, ctx: &mut Ctx) -> bool {
        if !ctx.trig_a {
            return false;
        }
        self.set_player_action(Step::FlyPlayer, ctx);
        true
    }

    fn sow_check(&mut self, ctx: &mut Ctx) -> bool {
        if !ctx.event_mode && !self.return_link {
            if let Some((speak, allowed)) = ctx.btn_b
                && speak
                && allowed
            {
                if self.do_status != SOW_CHECK_VALUE {
                    ctx.shock = Some(SHOCK_FRAME);
                }
                self.do_status = DO_STATUS_NAGERU;
                if ctx.trig_a {
                    self.talk_cmd = true;
                    self.talk_xy = false;
                    return true;
                }
                return false;
            }
        }
        self.fly_check(ctx)
    }

    fn get_stick_angle(&self, ctx: &Ctx) -> i16 {
        TURN_FLIP
            .wrapping_add(ctx.stick_angle)
            .wrapping_add(walk::to_s16(ctx.camera))
    }

    fn calc_stick_pos(&self, mut angle: i16, ctx: &Ctx) -> (i32, Vec3) {
        if let Some(aim) = ctx.lock_eye {
            return (1, aim);
        }
        if ctx.player_lock_on {
            angle = self.shape_yaw;
        }
        (
            0,
            Vec3::new(
                self.pos.x + STICK_POS_REACH * ssin(angle),
                self.pos.y,
                self.pos.z + STICK_POS_REACH * scos(angle),
            ),
        )
    }

    fn to_player_shape_angle(&self, ctx: &Ctx) -> i16 {
        match ctx.player {
            Some(at) => self
                .shape_yaw
                .wrapping_sub(target_angle_y(self.pos, at)),
            None => 0,
        }
    }
}

impl Makar {
    pub fn fly_action(
        &mut self,
        hold: bool,
        drive: f32,
        drive_angle: i16,
        brake: bool,
        ctx: &mut Ctx,
    ) -> bool {
        let as_player = self.as_player;
        let mut y_limit = HIO_FLY_Y_SPEED;
        let lift = (self.fly_speed) as f32 * HIO_FLY_GRAVITY_SCALE;
        let mut caught = false;
        if self.grounded {
            self.breaking();
            walk::chase_angle_s(&mut self.shape_pitch, 0, HIO_FLY_TURN_STEP);
            walk::chase_angle_s(&mut self.shape_roll, 0, HIO_FLY_TURN_STEP);
            self.gravity = HIO_FLY_GRAVITY + lift;
        } else if self.act_step < 3 {
            let timed = self.calc_flying_timer();
            if as_player {
                if timed {
                    ctx.events.list.push(SimEvent::Se(SE_CV_CB_FLY_END));
                }
                self.on_flying();
            }
            self.drop = 0.0;
            let mut push = Vec3::new(0.0, lift + HIO_FLY_GRAVITY, 0.0);
            let mut flat = HIO_FLY_XZ_SPEED;
            if self.fly_frame != 0 {
                push.x += drive * ssin(drive_angle);
                push.z += drive * scos(drive_angle);
            }
            if !self.is_music() && ctx.wind_tg {
                let hit = ctx.wind_push;
                push += hit * 0.01;
                flat = WIND_CATCH_SPEED;
                y_limit = WIND_CATCH_LIMIT;
                self.fly_speed = HIO_FLY_SPEED_RESET;
                caught = true;
            } else {
                let (wind, mut power) = ctx.wind;
                power *= HIO_WIND_POWER_SCALE;
                push += wind * power;
                flat += power;
                y_limit += power;
                let limit = (power * 400.0) as i32 as i16;
                if self.fly_speed < limit {
                    self.fly_speed = limit;
                }
            }
            let mut angle = atan2s(push.x, push.z);
            let reach = max_limit(
                (push.x * push.x + push.z * push.z).sqrt(),
                flat,
            );
            angle = angle.wrapping_sub(self.shape_yaw);
            let mut tilt = reach * HIO_FLY_TILT_SCALE;
            if tilt > HIO_FLY_TILT_MAX {
                tilt = HIO_FLY_TILT_MAX;
            }
            let cos = (tilt * scos(angle)) as i32 as i16;
            let sin = (-tilt * ssin(angle)) as i32 as i16;
            walk::chase_angle_s(&mut self.shape_pitch, cos, HIO_FLY_TURN_STEP);
            walk::chase_angle_s(&mut self.shape_roll, sin, HIO_FLY_TURN_STEP);
            self.look_back(false, ctx.player);
            self.speed += push;
            let magnitude = self.speed.length();
            if !is_zero(magnitude) && magnitude > flat {
                let mut cap = magnitude;
                walk::chase_f(&mut cap, flat, 1.0);
                if cap > WIND_MAX {
                    cap = WIND_MAX;
                }
                self.speed *= cap / magnitude;
            }
            self.yaw = atan2s(self.speed.x, self.speed.z);
            self.speed_f = (self.speed.x * self.speed.x + self.speed.z * self.speed.z).sqrt();
            if self.speed.y > y_limit {
                self.speed.y = y_limit;
            }
            self.gravity = 0.0;
        }
        if self.act_step == 1 {
            if self.anm_done {
                if !self.grounded {
                    self.fly_speed = HIO_FLY_SPEED_RESET;
                    self.speed.y += HIO_FLY_LIFT;
                    self.set_anm(ANM_07, ctx);
                } else {
                    self.set_anm(ANM_05, ctx);
                }
                self.act_step += 1;
            } else if self.anm_frame >= FLY_ANM_END_FRAME {
                ctx.events.list.push(SimEvent::Se(SE_CM_PRAPELLO_OPEN));
            }
        } else if self.act_step == 2 {
            if as_player && (caught || self.fly_timer < HIO_FLY_END_LIMIT) {
                if self.set_anm(ANM_15, ctx) && caught {
                    ctx.events.list.push(SimEvent::Se(SE_CV_CB_FLY_END));
                }
            } else if self.anm_row != NUT_SOW_STOP && self.fall_sum > HIO_FLY_FALL_STEP as f32 {
                self.set_anm(ANM_07, ctx);
            }
            if self.fly_timer != 0 && hold {
                chase_s(&mut self.fly_speed, HIO_FLY_SPEED_MAX, HIO_FLY_SPEED_STEP);
                self.land_timer = 0;
            } else {
                let stop = if self.grounded { HIO_FLY_SPEED_STOP } else { 0 };
                chase_s(&mut self.fly_speed, stop, HIO_FLY_LAND_STEP);
            }
            self.propeller = self.propeller.wrapping_add(self.fly_speed);
            self.fall_sum += self.pos.y - self.old.y;
            if self.fall_sum > HIO_FLY_TILT_LIMIT {
                self.fly_frame = 1;
            }
            ctx.events.list.push(SimEvent::Se(SE_CM_PRAPELLO_ROLLING));
            let landed = brake
                || (!as_player && ctx.wall_hit)
                || (!self.grounded && self.fly_speed == 0)
                || (self.grounded
                    && self.fly_speed <= HIO_FLY_SPEED_STOP
                    && (ctx.stick >= HIO_STICK_FACE || (self.land_timer += 1) > HIO_LANDING_LIMIT));
            if landed {
                self.set_anm(ANM_06, ctx);
                self.anm_frame = PROPELLER_LAND_FRAME;
                self.act_step += 1;
                self.off_flying();
            } else if self.grounded && self.anm_row == NUT_SOW_STOP {
                self.set_anm(ANM_05, ctx);
            }
            self.max_fall = HIO_FLY_MAX_FALL;
        } else {
            if self.anm_row == NUT_SOW_RESET && self.anm_done {
                self.set_anm(ANM_02, ctx);
                ctx.events.list.push(SimEvent::Se(SE_CM_PRAPELLO_OPEN));
            }
            if self.grounded {
                if self.ev_order == ORDER_OFF && self.act_step == 3 {
                    if ctx.special_code != SPECIAL_CODE_NONE {
                        self.speed.y = HIO_LANDING_BOUNCE;
                        self.check_landing(ctx);
                    }
                    self.act_step += 1;
                } else if self.anm_row != NUT_SOW_RESET {
                    if as_player {
                        self.set_player_action(Step::WaitPlayer, ctx);
                    } else {
                        self.set_npc_action(Step::Wait, Param::Null, ctx);
                    }
                    return false;
                }
            }
        }
        true
    }

    fn route_ang_check(&mut self, plane: Vec3, angle: &mut i16, player: Option<Vec3>) {
        let cross = self.normal.cross(plane);
        let mut next = atan2s(cross.x, cross.z);
        let far = i32::from(distance_angle_s(next, *angle)) > ROUTE_ANGLE;
        let height = player.map_or(0.0, |at| at.y - self.pos.y);
        if (self.normal.y >= ROUTE_CROSS_MIN && far) || height * cross.y < 0.0 {
            next = next.wrapping_add(TURN_FLIP);
        }
        *angle = next;
    }

    pub fn check_forward_ground_y(&self, angle: i16, ctx: &Ctx) -> f32 {
        let normal = ground::slope_normal(ctx.collision, self.pos);
        if i32::from(distance_angle_s(angle, atan2s(normal.x, normal.z))) > ROUTE_ANGLE {
            let probe = Vec3::new(
                self.pos.x + ssin(angle) * ROUTE_PROBE,
                self.pos.y + ROUTE_LIFT,
                self.pos.z + scos(angle) * ROUTE_PROBE,
            );
            return match ground::surface(ctx.collision, probe) {
                Some(height) if height > f32::NEG_INFINITY => height,
                _ => NO_FLOOR,
            };
        }
        NO_FLOOR
    }

    pub fn check_wall_jump(&self, angle: i16, ctx: &Ctx) -> f32 {
        let delta = self.check_forward_ground_y(angle, ctx) - self.pos.y;
        if ROUTE_JUMP_MIN < delta && delta < ROUTE_JUMP_MAX {
            return delta.sqrt() * ROUTE_JUMP_SCALE;
        }
        ROUTE_JUMP_NONE
    }

    pub fn chk_wall_hit(&self, ctx: &Ctx) -> bool {
        match ctx.collision {
            Some(collision) => {
                let probe = Vec3::new(
                    self.pos.x + ssin(self.shape_yaw) * ROUTE_PROBE,
                    self.pos.y,
                    self.pos.z + scos(self.shape_yaw) * ROUTE_PROBE,
                );
                let slid = collision.slide(self.pos, probe);
                i32::from(distance_angle_s(self.shape_yaw, atan2s(slid.x - self.pos.x, slid.z - self.pos.z)))
                    > ROUTE_TURN_WALL
            }
            None => false,
        }
    }

    pub fn route_check(&mut self, dist_sq: f32, angle: &mut i16, ctx: &mut Ctx) -> bool {
        if !self.grounded {
            let temp = self.pos;
            self.pos = self.old;
            self.speed_f = 0.0;
            self.grounded = true;
            if self.normal != Vec3::ZERO
                && i32::from(distance_angle_s(*angle, atan2s(self.normal.x, self.normal.z)))
                    > ROUTE_ANGLE
            {
                return true;
            }
            if self.ground_h - temp.y < ROUTE_DROP {
                return false;
            }
            self.set_npc_action(Step::Jump, Param::Null, ctx);
        } else if self.chk_wall_hit(ctx) {
            let jump = self.check_wall_jump(*angle, ctx);
            if jump >= 0.0 {
                self.set_npc_action(Step::Jump, Param::Speed(jump), ctx);
                return true;
            }
            if dist_sq > SQUARE_600 {
                return false;
            }
            let gap = ctx.player.map_or(0.0, |at| at.y - self.pos.y);
            if gap.abs() > ROUTE_FAR_LIMIT {
                return false;
            }
            let from = Vec3::new(self.pos.x, self.pos.y + ROUTE_LIFT, self.pos.z);
            let to = Vec3::new(
                ssin(*angle) * ROUTE_PROBE + self.pos.x,
                self.pos.y + ROUTE_LIFT,
                scos(*angle) * ROUTE_PROBE + self.pos.z,
            );
            if let Some(collision) = ctx.collision
                && collision.slide(from, to) != to
            {
                let normal = ground::slope_normal(ctx.collision, to);
                self.route_ang_check(normal, angle, ctx.player);
            }
        }
        true
    }
}

impl Makar {
    pub fn get_msg(&self, ctx: &Ctx) -> u32 {
        if self.is_type_boss_die() {
            return MSG_BOSS_DIE;
        }
        if self.is_type_forest() {
            return if ctx.stories.test(BIT_1904) {
                MSG_FOREST_AFTER
            } else {
                MSG_FOREST_FIRST
            };
        }
        if self.is_type_waterfall() {
            if self.is_tact() {
                return if ctx.stories.test(BIT_1840) {
                    MSG_WATERFALL_TACT_ALT
                } else {
                    MSG_WATERFALL_TACT_FIRST
                };
            }
            return if ctx.stories.test(BIT_1880) {
                MSG_WATERFALL_AFTER
            } else {
                MSG_WATERFALL_FIRST
            };
        }
        if self.is_ship_ride() {
            return MSG_SHIP_RIDE;
        }
        0
    }

    pub fn next_msg_status(&mut self, ctx: &mut Ctx) -> u8 {
        if self.msg_no == MSG_TACT_INPUT {
            return STATUS_MSG_ENDS;
        }
        if self.msg_no == MSG_WATERFALL_TACT_FIRST
            || (MSG_WATERFALL_FIRST..=MSG_WATERFALL_CHAIN_LAST).contains(&self.msg_no)
            || self.msg_no == MSG_TACT_WRONG
            || (MSG_FOREST_FIRST..=MSG_FOREST_CHAIN_LAST).contains(&self.msg_no)
        {
            self.msg_no = self.msg_no.wrapping_add(1);
            return STATUS_MSG_CONTINUES;
        }
        if self.msg_no == MSG_WATERFALL_GIFT {
            ctx.stories.set(BIT_1880);
            return STATUS_MSG_ENDS;
        }
        if self.msg_no == MSG_WATERFALL_TACT_NEXT {
            ctx.stories.set(BIT_1840);
            return STATUS_MSG_ENDS;
        }
        if self.msg_no == MSG_FOREST_SOW_END {
            ctx.stories.set(BIT_1904);
            return STATUS_MSG_ENDS;
        }
        STATUS_MSG_ENDS
    }

    pub fn init_talk(&mut self, ctx: &mut Ctx) -> bool {
        if self.msg_id == FPC_M_ERROR_PROCESS_ID {
            self.msg_id = if self.msg_no == MSG_TACT_INPUT {
                ctx.text.tact_message_set()
            } else {
                ctx.text.message_set(self.msg_no)
            };
            false
        } else {
            ctx.text.procs.search_by_id(self.msg_id).is_some()
        }
    }

    pub fn exec_talk(&mut self, tact: bool, ctx: &mut Ctx) -> bool {
        let Some(status) = ctx.text.status() else {
            return false;
        };
        if status == STATUS_MSG_DISPLAYED {
            let next = self.next_msg_status(ctx);
            ctx.text.set_status(next);
            if next == STATUS_MSG_CONTINUES {
                ctx.text.message_set(self.msg_no);
            }
        } else if status == STATUS_INPUT {
            if tact {
                if ctx.cancel_button {
                    ctx.text.set_status(STATUS_MSG_ENDS);
                    ctx.text.flags.message_send_on();
                    self.on_tact_cancel();
                }
                if self.is_tact_correct() {
                    ctx.text.set_status(STATUS_MSG_ENDS);
                    ctx.text.flags.message_send_on();
                }
            }
        } else if status == STATUS_TYPING {
            if self.msg_no != MSG_TACT_INPUT {
                let attr = i32::from(ctx.text.info.mesg_anime);
                let anm = self.get_anm_type(attr);
                self.set_anm(anm, ctx);
            }
        } else if status == STATUS_BOX_CLOSED {
            ctx.text.set_status(STATUS_MSG_DESTROYED);
            return true;
        }
        false
    }

    pub fn check_command_talk(&mut self) -> bool {
        if self.talk_cmd {
            if !self.talk_xy {
                if self.ev_order == ORDER_TALK_ITEM {
                    self.ev_order = ORDER_OFF;
                }
                self.on_tact();
                return false;
            }
            return true;
        }
        false
    }

    pub fn check_order(&mut self, ctx: &mut Ctx) {
        if !self.talk_cmd {
            return;
        }
        if matches!(
            self.ev_order,
            ORDER_SPEAK | ORDER_TALK | ORDER_TALK_ITEM
        ) {
            self.ev_order = ORDER_OFF;
            if !self.talk_xy {
                self.on_tact();
            } else {
                self.set_npc_action(Step::Talk, Param::Null, ctx);
            }
        }
    }

    pub fn event_order(&mut self, ctx: &mut Ctx) {
        if self.ev_order == ORDER_SPEAK || self.ev_order == ORDER_TALK {
        } else if self.ev_order == ORDER_TALK_ITEM {
        } else if self.ev_order != ORDER_OFF && i32::from(self.ev_order) < ORDER_OTHER {
            self.ev_slot = self.ev_order;
            let index = self.ev_index.get(usize::try_from(self.ev_slot).unwrap_or(0)).copied();
            if let Some(index) = index.filter(|value| *value >= 0) {
                let slot = usize::try_from(index as u16).unwrap_or(0);
                if ctx.active.stage_of(slot).is_none() {
                    ctx.active.order(ctx.list, slot, ctx.entity);
                }
            }
        }
    }

    pub fn event_proc(&mut self, ctx: &mut Ctx) -> bool {
        if self.ev_demo_accept && self.ev_order != ORDER_OFF {
            if self.ev_order == ORDER_DRAW {
                ctx.drawn = true;
            } else if self.ev_order == ORDER_TO_LINK {
                ctx.events.list.push(SimEvent::Se(SE_CTRL_NPC_TO_LINK));
            }
            self.ev_accepted = true;
            self.ev_demo_accept = false;
            self.ev_order = ORDER_OFF;
        }
        let running = ctx.event_mode && (self.as_player || !self.check_command_talk());
        if running {
            if let Some(cut) = ctx.cut {
                if ctx.advance {
                    self.ev_init(cut, ctx);
                    self.speed_f = 0.0;
                }
                if self.ev_run(cut, ctx) {
                    ctx.active.end_staff_cut(ctx.list, ctx.ev_flags, STAFF_NAME);
                }
            }
            self.player_action = Step::None;
            self.npc_action = Step::None;
            if self.ev_accepted {
                let index = self
                    .ev_index
                    .get(usize::try_from(self.ev_slot).unwrap_or(0))
                    .copied()
                    .unwrap_or(NO_INDEX);
                if index >= 0 && ctx.active.end_check(usize::try_from(index as u16).unwrap_or(0)) {
                    ctx.reset_event();
                    self.ev_accepted = false;
                    if self.ev_slot == 1 {
                        self.return_link_player(ctx);
                        self.return_link = false;
                    }
                    self.ev_slot = NO_INDEX;
                }
                return true;
            }
            if ctx.cut.is_some() {
                return true;
            }
            if !ctx.player_door {
                return true;
            }
        }
        false
    }

    fn ev_init(&mut self, cut: Cut, ctx: &mut Ctx) {
        match cut {
            Cut::Wait => self.ev_init_wait(ctx),
            Cut::MsgSet => self.ev_init_msg_set(ctx),
            Cut::MsgEnd => {}
            Cut::MovePos => self.ev_init_move_pos(ctx),
            Cut::OffsetLink => self.ev_init_offset_link(ctx),
            Cut::Walk => self.ev_init_walk(ctx),
            Cut::ToLink => self.ev_init_to_link(ctx),
            Cut::Tact => self.ev_init_tact(ctx),
            Cut::CelloPlay => self.ev_init_cello_play(ctx),
            Cut::Turn => self.ev_init_turn(ctx),
            Cut::Sow => self.ev_init_sow(ctx),
            Cut::SetAnm => self.ev_init_set_anm(ctx),
            Cut::SetGoal => self.ev_init_set_goal(ctx),
            Cut::Warp => self.ev_init_warp(ctx),
            Cut::End => self.ev_init_end(ctx),
        }
    }

    fn ev_run(&mut self, cut: Cut, ctx: &mut Ctx) -> bool {
        match cut {
            Cut::Wait => self.ev_act_wait(ctx),
            Cut::MsgSet => self.init_talk(ctx),
            Cut::MsgEnd => self.ev_act_msg_end(ctx),
            Cut::MovePos => true,
            Cut::OffsetLink => true,
            Cut::Walk => self.ev_act_walk(ctx),
            Cut::ToLink => self.ev_act_to_link(ctx),
            Cut::Tact => self.ev_act_tact(ctx),
            Cut::CelloPlay => ctx.tact_end,
            Cut::Turn => self.ev_act_turn(ctx),
            Cut::Sow => self.ev_act_sow(ctx),
            Cut::SetAnm => true,
            Cut::SetGoal => {
                self.ev_init_set_goal(ctx);
                true
            }
            Cut::Warp => self.ev_act_warp(ctx),
            Cut::End => true,
        }
    }

    fn ev_check_disp(&mut self, ctx: &mut Ctx) {
        match ctx.cut_number("Disp") {
            Some(0) => ctx.drawn = false,
            Some(_) => ctx.drawn = true,
            None => ctx.drawn = true,
        }
    }

    fn ev_init_wait(&mut self, ctx: &mut Ctx) {
        self.ev_check_disp(ctx);
        self.timer = ctx.cut_number("Timer").and_then(as_i16).unwrap_or(0);
        let anm = if self.is_music() { ANM_08 } else { ANM_00 };
        self.set_anm(anm, ctx);
    }

    fn ev_act_wait(&mut self, ctx: &mut Ctx) -> bool {
        let before = self.timer;
        if chase_s(&mut self.timer, 0, 1) {
            if before < 0 {
                ctx.reset_event();
            }
            return true;
        }
        false
    }

    fn ev_init_msg_set(&mut self, ctx: &mut Ctx) {
        self.msg_id = FPC_M_ERROR_PROCESS_ID;
        if let Some(msg) = ctx.cut_number("MsgNo").and_then(as_u32) {
            self.msg_no = msg;
            if self.msg_no == MSG_WATERFALL_TACT_FIRST && ctx.stories.test(BIT_1840) {
                self.msg_no = MSG_WATERFALL_TACT_ALT;
            }
        }
    }

    fn ev_act_msg_end(&mut self, ctx: &mut Ctx) -> bool {
        let result = self.exec_talk(false, ctx);
        if result {
            let _ = ctx.cut_number("EndMode");
            self.set_wait_action(ctx);
        } else {
            self.set_trn();
            self.look_back(true, ctx.player);
        }
        result
    }

    fn ev_init_move_pos(&mut self, ctx: &mut Ctx) {
        if let Some(deg) = ctx.cut_float("Angle") {
            let angle = deg2s(deg);
            self.yaw = angle;
            self.shape_yaw = angle;
        }
        if let Some(at) = ctx.cut_vec("Pos") {
            self.pos = at;
        }
        match ctx.cut_text("ShipRide") {
            Some("on") => self.on_ship_ride(),
            Some("off") => self.off_ship_ride(),
            _ => {}
        }
    }

    fn ev_init_offset_link(&mut self, ctx: &mut Ctx) {
        if let Some(to) = ctx.cut_vec("To") {
            let dest = offset_pos(self.pos, self.shape_yaw, to);
        }
        if let Some(from) = ctx.cut_vec("From") {
            if let Some(link) = ctx.link {
                self.pos = offset_pos(link, ctx.player_shape_yaw, from);
            }
        }
        if let Some(player) = ctx.player {
            self.yaw = target_angle_y(self.pos, player);
            self.shape_yaw = self.yaw;
        }
    }

    fn ev_init_walk(&mut self, ctx: &mut Ctx) {
        self.set_anm(ANM_01, ctx);
        self.speed_f = 0.0;
    }

    fn ev_act_walk(&mut self, ctx: &mut Ctx) -> bool {
        let speed = ctx.cut_float("Speed").unwrap_or(0.0);
        let dist = ctx.cut_float("Dist");
        let (target, face) = if let Some(at) = ctx.cut_vec("Pos") {
            (at, at)
        } else if let Some(offset) = ctx.cut_vec("Offset") {
            match ctx.partner_pos {
                Some(base) => {
                    let angle = target_angle_y(self.pos, base);
                    (offset_pos(base, angle, offset), base)
                }
                None => (ctx.goal.unwrap_or(self.pos), ctx.goal.unwrap_or(self.pos)),
            }
        } else {
            let goal = ctx.goal.unwrap_or(self.pos);
            (goal, goal)
        };
        let delta = target - self.pos;
        let mut angle = atan2s(delta.x, delta.z);
        match dist {
            None => {
                if abs_xz(delta) <= speed {
                    self.set_anm(ANM_00, ctx);
                    self.pos = target;
                    self.speed_f = 0.0;
                    return true;
                }
            }
            Some(dist) => {
                if abs_xz(delta) < dist {
                    self.set_anm(ANM_00, ctx);
                    self.speed_f = 0.0;
                    return true;
                }
            }
        }
        let mut walk_speed = speed;
        if i32::from(distance_angle_s(self.shape_yaw, angle)) > ROUTE_ANGLE {
            walk_speed = -walk_speed;
            angle = angle.wrapping_add(TURN_FLIP);
        }
        if self.grounded {
            self.walk_action(walk_speed, HIO_FORWARD_ACCEL, angle, ctx);
        } else if self.ev_order != ORDER_OFF {
            self.speed.y = JUMP_LIFT;
        }
        let _ = face;
        false
    }

    fn ev_init_to_link(&mut self, ctx: &mut Ctx) {
        self.set_anm(ANM_01, ctx);
        self.speed_f = 0.0;
        if let Some(player) = ctx.player {
            self.yaw = target_angle_y(self.pos, player);
            self.shape_yaw = self.yaw;
        }
    }

    fn ev_act_to_link(&mut self, ctx: &mut Ctx) -> bool {
        let speed = ctx.cut_float("Speed").unwrap_or(0.0);
        let dist = ctx.cut_float("Dist").unwrap_or(0.0);
        let player = ctx.player.unwrap_or(Vec3::ZERO);
        if abs_xz(player - self.pos) < dist {
            self.set_anm(ANM_00, ctx);
            self.speed_f = 0.0;
            return true;
        }
        self.walk_action(speed, HIO_FORWARD_ACCEL, target_angle_y(self.pos, player), ctx);
        false
    }

    fn ev_init_tact(&mut self, ctx: &mut Ctx) {
        self.set_anm(ANM_08, ctx);
    }

    fn ev_act_tact(&mut self, ctx: &mut Ctx) -> bool {
        let prm = ctx.cut_number("prm0").unwrap_or(0);
        let song = ctx.tact_music;
        if song >= 0 {
            self.on_tact_correct();
        }
        let result = self.exec_talk(true, ctx);
        if result {
            if self.is_tact_correct() {
                self.off_tact_correct();
                if song == prm {
                    self.msg_no = MSG_TACT_CORRECT;
                } else {
                    self.msg_no = MSG_TACT_WRONG;
                }
            } else {
                self.ev_order = ORDER_TACT_PRM;
                if self.is_tact_cancel() {
                    self.msg_no = MSG_TACT_CANCEL;
                    self.off_tact_cancel();
                }
                ctx.drawn = true;
                ctx.reset_event();
            }
        }
        result
    }

    fn ev_init_cello_play(&mut self, ctx: &mut Ctx) {
        self.set_anm(ANM_17, ctx);
    }

    fn ev_init_turn(&mut self, ctx: &mut Ctx) {
        self.ev_check_disp(ctx);
        self.set_anm(ANM_00, ctx);
    }

    fn ev_act_turn(&mut self, ctx: &mut Ctx) -> bool {
        let angle = if let Some(deg) = ctx.cut_float("Angle") {
            deg2s(deg)
        } else if ctx.cut_number("Target").is_some() {
            match ctx.link {
                Some(at) => target_angle_y(self.pos, at),
                None => self.shape_yaw,
            }
        } else {
            match ctx.partner_pos {
                Some(at) => target_angle_y(self.pos, at),
                None => self.shape_yaw,
            }
        };
        if walk::chase_angle_s(&mut self.shape_yaw, angle, TURN_EVENT) {
            self.shape_yaw = angle;
            self.yaw = angle;
            return true;
        }
        self.yaw = self.shape_yaw;
        false
    }

    fn ev_init_sow(&mut self, ctx: &mut Ctx) {
        self.set_anm(ANM_0C, ctx);
        self.nut_frame = 0.0;
        self.nut_done = false;
        self.on_nut();
        self.timer = ctx.cut_number("Timer").and_then(as_i16).unwrap_or(0);
    }

    fn ev_act_sow(&mut self, ctx: &mut Ctx) -> bool {
        if self.nut_done && self.is_nut() {
            self.off_nut();
        }
        if self.anm_done {
            self.set_anm(ANM_00, ctx);
        }
        calc_timer(&mut self.timer) == 0
    }

    fn ev_init_set_anm(&mut self, ctx: &mut Ctx) {
        if let Some(number) = ctx.cut_number("Number") {
            let anm = self.get_anm_type(number);
            self.set_anm(anm, ctx);
        }
    }

    fn ev_init_set_goal(&mut self, ctx: &mut Ctx) {
        if let Some(offset) = ctx.cut_vec("Offset") {
            ctx.goal = Some(offset_pos(self.pos, self.shape_yaw, offset));
        }
    }

    fn ev_init_warp(&mut self, ctx: &mut Ctx) {
        self.set_anm(ANM_00, ctx);
        self.speed.y = WARP_UP_SPEED;
        self.timer = -(WARP_HOLD_FRAMES);
        let _ = ctx;
    }

    fn ev_act_warp(&mut self, ctx: &mut Ctx) -> bool {
        self.gravity = 0.0;
        if self.timer < 0 {
            if walk::chase_f(&mut self.speed.y, 0.0, WARP_BRACE)
                && chase_s(&mut self.timer, 0, 1)
            {
                self.timer = 0;
            }
        } else {
            if chase_s(&mut self.timer, WARP_SPIN_STEP, WARP_SPIN_RATE) {
                walk::chase_f(&mut self.speed.y, WARP_DOWN_SPEED, WARP_DOWN_STEP);
                walk::chase_f(&mut self.scale.x, 0.0, WARP_SCALE_SPEED);
                self.scale.z = self.scale.x;
                if walk::chase_f(&mut self.scale.y, WARP_SCALE_Y, WARP_SCALE_SPEED) {
                    return true;
                }
            }
            self.yaw = self.yaw.wrapping_add(self.timer);
            self.shape_yaw = self.yaw;
        }
        let _ = ctx;
        false
    }

    fn ev_init_end(&mut self, ctx: &mut Ctx) {
        let mode = ctx.cut_number("EndMode").unwrap_or(0);
        if mode == -2 {
            if self.msg_no == MSG_TACT_CORRECT {
                ctx.next.target = Some(StageTarget {
                    stage: NEXT_STAGE.to_owned(),
                    room: NEXT_STAGE_ROOM,
                    start: NEXT_STAGE_POINT,
                });
                ctx.stories.set(BIT_1610);
                ctx.stories.set(BIT_1604);
            }
            ctx.reset_event();
            self.set_wait_action(ctx);
        } else if mode == -1 {
            ctx.reset_event();
            self.set_wait_action(ctx);
        } else {
            self.set_wait_action(ctx);
        }
    }
}

impl Makar {
    pub fn ship_ride_check(&mut self, ctx: &mut Ctx) -> bool {
        if ctx.stories.test(BIT_1604) && ctx.ship_pos.is_some() {
            self.set_npc_action(Step::Ship, Param::Null, ctx);
            self.on_ship_ride();
            ctx.drawn = false;
            return true;
        }
        false
    }

    pub fn init(&mut self, actor: &Actor, ctx: &mut Ctx) {
        self.pos = actor.pos;
        self.old = actor.pos;
        self.home = actor.pos;
        self.home_angle = actor.angle;
        self.yaw = actor.angle[1];
        self.shape_yaw = actor.angle[1];
        self.pitch = actor.angle[0];
        self.shape_pitch = actor.angle[0];
        self.roll = actor.angle[2];
        self.shape_roll = actor.angle[2];
        self.home_room = actor.room;
        self.gravity = HIO_GRAVITY;
        self.player_room = false;
        self.off_flying();
        self.fly_timer = 0;
        self.status = 0;
        self.attn_dist_talk = ATTN_DIST_TALK;
        self.attn_dist_speak = ATTN_DIST_SPEAK;
        self.attn_dist_carry = ATTN_DIST_CARRY;
        for (slot, name) in EVENT_NAMES.iter().enumerate() {
            self.ev_index[slot] = ctx
                .list
                .event_index(*name)
                .and_then(|index| i16::try_from(index).ok())
                .unwrap_or(NO_INDEX);
        }
        self.ev_slot = NO_INDEX;
        self.ev_order = ORDER_OFF;
        self.npc_change = true;
        if self.is_type_kaze_boss() || self.is_type_forest() || self.is_type_waterfall() {
            self.set_npc_action(Step::Music, Param::Null, ctx);
            self.home_room = ROOM_NO_GONE;
            self.gravity = MUSIC_GRAVITY;
        } else if self.is_type_boss_die() {
            if ctx.boss_enemy {
                self.set_npc_action(Step::Wait, Param::Null, ctx);
            } else {
                self.set_npc_action(Step::Rescue, Param::Null, ctx);
            }
        } else if !self.is_type_ekaze() && !self.is_type_kaze() && self.ship_ride_check(ctx) {
            self.attn_dist_talk = ATTN_DIST_SHIP;
            self.attn_dist_speak = ATTN_DIST_SHIP;
        } else {
            if ctx.stories.test(BIT_1610) {
                self.npc_change = false;
            }
            if self.is_type_ekaze() {
                self.npc_call = true;
            }
            self.set_npc_action(Step::Wait, Param::Null, ctx);
            self.off_music();
        }
        self.cyl_radius = CYL_RADIUS;
        self.cyl_height = CYL_HEIGHT;
        self.staff_idx = ctx.list.staff_of(STAFF_NAME).unwrap_or(NO_STAFF);
    }

    pub fn execute(&mut self, ctx: &mut Ctx) -> bool {
        self.old = self.pos;
        self.off_floor_seen();
        self.player_room = ctx.player_room;
        if !self.is_ship_ride() && !ctx.carry_now {
            self.grounded = ground::grounded(ctx.collision, self.pos);
            self.home_room = match ctx.room_loaded {
                Some(room) => room,
                None => self.home_room,
            };
            if self.home_room < 0 {
                return true;
            }
            if !ctx.initial_room_ok {
                self.home_room = ROOM_NO_RESET;
                return true;
            }
            if self.grounded && ctx.safe_poly && ctx.move_bg {
                ctx.move_bg_pos = Some(self.old);
            }
        }
        if ctx.now_pos_move {
            self.play_anm(ctx);
        }
        let mut push = Vec3::ZERO;
        let mut push_scale = WIND_SAMPLE_STEP;
        if !ctx.carry_now && !self.is_ship_ride() {
            if ctx.now_pos_move && !self.return_link {
                let before = self.pos.y;
                self.move_step(ctx);
                self.drop += before - self.pos.y;
                self.pos.x += self.push.x;
                self.pos.z += self.push.z;
                self.max_fall = MOVE_FALL_SPEED;
                self.gravity = HIO_GRAVITY;
            }
            self.grounded = ground::grounded(ctx.collision, self.pos);
            self.ground_h = ground::surface(ctx.collision, self.pos).unwrap_or(f32::NEG_INFINITY);
            if self.grounded && self.ground_h != f32::NEG_INFINITY {
                self.pos.y = self.ground_h;
            }
            self.on_floor_seen();
            if self.ground_h == f32::NEG_INFINITY || ctx.ground_code == GROUND_CODE_INVALID {
                if self.is_fly_action() && self.fall_sum > FALL_ACC_RESET {
                    self.floor_count = 0;
                } else if self.floor_count < FLOOR_COUNT_MAX {
                    self.floor_count += 1;
                } else if ctx.room_loaded.is_none() {
                    self.pos = self.home;
                    self.yaw = self.home_angle[1];
                    self.shape_yaw = self.home_angle[1];
                    self.pitch = self.home_angle[0];
                    self.shape_pitch = self.home_angle[0];
                    self.roll = self.home_angle[2];
                    self.shape_roll = self.home_angle[2];
                    self.speed_f = 0.0;
                    self.floor_count = 0;
                    ctx.events.list.push(SimEvent::Se(SE_CV_CB_LEFT_ALONE));
                }
            } else {
                self.floor_count = 0;
            }
            if self.ground_h != f32::NEG_INFINITY {
                let normal = ground::slope_normal(ctx.collision, self.pos);
                if normal != Vec3::ZERO {
                    self.normal = normal;
                    if self.grounded {
                        self.max_fall = SLOPE_MAX_FALL - (1.0 - normal.y) * SLOPE_MAX_FALL_SCALE;
                        if ctx.special_code == SPECIAL_CODE_NONE {
                            let flat = abs2_xz(Vec3::new(normal.x, 0.0, normal.z));
                            push.x = normal.x * SLOPE_NORMAL_SCALE;
                            push.z = normal.z * SLOPE_NORMAL_SCALE;
                            self.speed.y = (1.0 - flat).sqrt() * SLOPE_LIFT_SPEED;
                            self.max_fall = self.speed.y;
                            push_scale = flat * SLOPE_DRIFT_SCALE;
                            self.check_landing(ctx);
                        }
                    }
                }
            }
        }
        if ctx.now_pos_move {
            self.play_anm(ctx);
        }
        if self.as_player {
            ctx.do_status = ACT_STTS_BLANK;
        }
        if !self.event_proc(ctx) {
            self.off_tact();
            if !self.is_fly_action() && !self.is_music() && ctx.wind_tg {
                push = ctx.wind_push;
                let mut reach = abs_xz(ctx.wind_push);
                if reach < WIND_CATCH_FLOOR {
                    push = (self.pos - ctx.wind_tg_pos) * WIND_CATCH_SCALE;
                }
                if reach > WIND_CATCH_SPEED {
                    push *= WIND_CATCH_SPEED / reach;
                }
                push_scale = 1.0;
            }
            add_calc_pos_xz(&mut self.push, push, WIND_PUSH_SCALE, push_scale, WIND_PUSH_STEP);
            if self.as_player {
                if self.return_link {
                    self.ev_order = ORDER_TO_LINK;
                } else {
                    if self.is_no_carry_action() {
                        self.set_player_action(Step::CarryPlayer, ctx);
                        self.return_link = true;
                    }
                    if !ctx.event_mode && ctx.return_pressed {
                        self.return_link = true;
                    }
                    self.player_action(ctx);
                    if self.damage_timer == 0 && !self.is_music() && ctx.tg_hit {
                        let delta = self.pos - ctx.tg_pos;
                        self.yaw = atan2s(delta.x, delta.z);
                        self.speed_f = HIO_HIT_SPEED_F;
                        self.set_player_action(Step::HitPlayer, ctx);
                        self.damage_timer = HIO_DAMAGE_TIMER;
                        ctx.events.list.push(SimEvent::Se(SE_CV_CB_DAMAGE));
                    }
                }
                self.do_status = ctx.do_status;
            } else {
                self.move_block = self.chk_move_block(ctx);
                self.carry_check(ctx);
                self.check_order(ctx);
                self.npc_action(ctx);
                if self.npc_action != Step::Fly {
                    self.yaw = self.shape_yaw;
                }
                if self.damage_timer == 0 && !self.is_music() && ctx.tg_hit {
                    let delta = self.pos - ctx.tg_pos;
                    self.yaw = atan2s(delta.x, delta.z);
                    self.speed_f = HIO_HIT_SPEED_F;
                    self.set_npc_action(Step::Hit, Param::Null, ctx);
                    self.damage_timer = HIO_DAMAGE_TIMER;
                    ctx.events.list.push(SimEvent::Se(SE_CV_CB_DAMAGE));
                }
                self.do_status = 0;
            }
        }
        self.event_order(ctx);
        self.attn_pos = offset_pos(
            self.pos,
            self.shape_yaw,
            Vec3::new(0.0, HIO_ATTN_Y_OFFSET, 0.0),
        );
        self.eye_pos = offset_pos(
            self.pos,
            self.shape_yaw,
            Vec3::new(0.0, HIO_EYE_LIFT, 0.0),
        );
        if self.damage_timer > 0 {
            self.damage_timer -= 1;
        }
        true
    }

    fn move_step(&mut self, ctx: &Ctx) {
        let mut y = self.speed.y + self.gravity;
        if y < self.max_fall {
            y = self.max_fall;
        }
        self.speed = Vec3::new(
            self.speed_f * ssin(self.yaw),
            y,
            self.speed_f * scos(self.yaw),
        );
        self.pos += self.speed + ctx.move_step;
    }

    fn chk_move_block(&self, ctx: &Ctx) -> i8 {
        let _ = ctx;
        0
    }
}

fn as_i16(value: i32) -> Option<i16> {
    i16::try_from(value).ok()
}

fn as_u32(value: i32) -> Option<u32> {
    u32::try_from(value).ok()
}

pub struct Text<'a> {
    pub procs: &'a mut MsgProcs,
    pub info: &'a mut MsgGameInfo,
    pub flags: &'a mut MsgFlags,
    pub one: &'a mut Msg1Box,
    pub two: &'a mut Msg2Box,
    pub archive: Option<&'a dyn MsgArchive>,
    pub clear_count: u8,
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

    pub fn message_set(&mut self, msg_no: u32) -> u32 {
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
            Vec3::ZERO,
        )
    }

    pub fn tact_message_set(&mut self) -> u32 {
        let Some(archive) = self.archive else {
            return FPC_M_ERROR_PROCESS_ID;
        };
        fop_msgm_tact_message_set(
            self.procs,
            self.flags,
            self.info,
            archive,
            self.clear_count,
        )
    }
}

impl Makar {
    pub fn stage_ok(
        prm: u32,
        equipped: &Equipped,
        collect: &Collect,
        stories: &StoryBits,
        start_stage: &str,
    ) -> bool {
        if prm & PRM_TYPE_MASK != PRM_TYPE_BOSS_DIE {
            if item_obtained(equipped, collect, ITEM_MASTER_SWORD_3) {
                if prm & PRM_TYPE_MASK != PRM_TYPE_KAZE_BOSS {
                    return false;
                }
            } else if stories.test(BIT_2910) {
                if prm & PRM_TYPE_MASK != PRM_TYPE_KAZE {
                    return false;
                }
            } else if stories.test(BIT_2E02) {
                if prm & PRM_TYPE_MASK != PRM_TYPE_EKAZE {
                    return false;
                }
            } else if stories.test(BIT_1610) {
                if start_stage != STAGE_SEA || !stories.test(BIT_1604) {
                    return false;
                }
            } else if item_obtained(equipped, collect, ITEM_MASTER_SWORD_2) {
                if prm & PRM_TYPE_MASK != PRM_TYPE_WATERFALL {
                    return false;
                }
            } else if stories.test(BIT_1820) {
                return false;
            } else if item_obtained(equipped, collect, ITEM_PEARL_FARORE) {
                if prm & PRM_TYPE_MASK != PRM_TYPE_FOREST {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

pub struct Ctx<'a> {
    pub entity: Entity,
    pub player: Option<Vec3>,
    pub link: Option<Vec3>,
    pub player_shape_yaw: i16,
    pub partner_pos: Option<Vec3>,
    pub goal: Option<Vec3>,
    pub collision: Option<&'a Collision>,
    pub list: &'a EventList,
    pub active: &'a mut ActiveEvent,
    pub stories: &'a mut StoryBits,
    pub ev_flags: &'a mut EventFlags,
    pub events: &'a mut SimEvents,
    pub next: &'a mut NextStage,
    pub text: Text<'a>,
    pub cut: Option<Cut>,
    pub cut_index: Option<usize>,
    pub advance: bool,
    pub event_mode: bool,
    pub player_door: bool,
    pub initial_room_ok: bool,
    pub now_pos_move: bool,
    pub room_loaded: Option<i8>,
    pub player_room: bool,
    pub move_step: Vec3,
    pub safe_poly: bool,
    pub move_bg: bool,
    pub move_bg_pos: Option<Vec3>,
    pub ground_code: i32,
    pub special_code: i32,
    pub wall_hit: bool,
    pub co_hit: bool,
    pub tg_hit: bool,
    pub tg_pos: Vec3,
    pub wind_tg: bool,
    pub wind_tg_pos: Vec3,
    pub wind_push: Vec3,
    pub wind: (Vec3, f32),
    pub ship_pos: Option<Vec3>,
    pub player_ship_ride: bool,
    pub tact_music: i32,
    pub tact_end: bool,
    pub cancel_button: bool,
    pub player_lock: bool,
    pub player_lock_on: bool,
    pub player_hang: bool,
    pub lock_eye: Option<Vec3>,
    pub btn_b: Option<(bool, bool)>,
    pub stick: f32,
    pub stick_angle: i16,
    pub camera: f32,
    pub trig_a: bool,
    pub trig_b: bool,
    pub return_pressed: bool,
    pub carry_now: bool,
    pub player_speed: f32,
    pub player_anim: f32,
    pub player_anim_end: f32,
    pub do_status: u8,
    pub shock: Option<i32>,
    pub boss_enemy: bool,
    pub drawn: bool,
}

impl Ctx<'_> {
    pub fn cut_number(&self, key: &str) -> Option<i32> {
        let cut = self.cut_index?;
        self.list.cut_number(cut, key)
    }

    pub fn cut_float(&self, key: &str) -> Option<f32> {
        let cut = self.cut_index?;
        self.list.cut_float(cut, key)
    }

    pub fn cut_vec(&self, key: &str) -> Option<Vec3> {
        let cut = self.cut_index?;
        self.list.cut_vec(cut, key)
    }

    pub fn cut_text(&self, key: &str) -> Option<&str> {
        let cut = self.cut_index?;
        self.list.cut_text(cut, key)
    }

    pub fn reset_event(&mut self) {
        self.text.info.event_mode = DA_EVT_MODE_NONE;
    }
}

pub fn is_makar(name: &str) -> bool {
    name == MAKAR_ACTOR
}

fn radians_to_s16(radians: f32) -> i16 {
    super::walk::to_s16(radians)
}

#[derive(SystemParam)]
pub struct World<'w> {
    pub list: Res<'w, EventList>,
    pub active: ResMut<'w, ActiveEvent>,
    pub stories: ResMut<'w, StoryBits>,
    pub events: ResMut<'w, SimEvents>,
    pub next: ResMut<'w, NextStage>,
    pub event_flags: ResMut<'w, EventFlags>,
}

#[derive(SystemParam)]
pub struct Talk<'w, 's> {
    pub procs: ResMut<'w, MsgProcs>,
    pub info: ResMut<'w, MsgGameInfo>,
    pub flags: ResMut<'w, MsgFlags>,
    pub one: ResMut<'w, Msg1Box>,
    pub two: ResMut<'w, Msg2Box>,
    pub text_flags: Res<'w, TextFlags>,
    pub archive: Option<Res<'w, MsgArchives>>,
    pub players: Query<'w, 's, &'static Player>,
}

#[derive(SystemParam)]
pub struct Pad<'w> {
    pub intent: Res<'w, MoveIntent>,
    pub camera: Res<'w, CameraYaw>,
    pub do_intent: Res<'w, DoIntent>,
    pub target_intent: Res<'w, TargetIntent>,
    pub carry: Res<'w, Carry>,
    pub lock: Res<'w, Lock>,
}

pub fn control(
    mut world: World,
    mut talk: Talk,
    pad: Pad,
    collision: Option<Res<Collision>>,
    mut actors: Query<(Entity, &mut Actor, &mut Makar)>,
) {
    let solid = collision.as_deref();
    let link = talk.players.iter().next().copied();
    let axis = pad.intent.axis;
    let stick = axis.length();
    let stick_angle = if axis.x == 0.0 && axis.y == 0.0 {
        0
    } else {
        walk::to_s16(axis.x.atan2(axis.y))
    };
    let clear_count = talk.text_flags.clear_count;
    let event_mode = talk.info.event_run_check();
    let archive = talk.archive.as_deref().map(|held| held as &dyn MsgArchive);
    for (entity, mut actor, mut makar) in &mut actors {
        let actor = &mut *actor;
        let makar = &mut *makar;
        let current = ActiveEvent::staff_cut(&world.active, &world.list, STAFF_NAME)
            .and_then(|(index, advance)| {
                world
                    .list
                    .cut_name(index)
                    .map(|name| (index, advance, Cut::of_name(name)))
            });
        let mut ctx = Ctx {
            entity,
            player: link.map(|player| player.pos),
            link: link.map(|player| player.pos),
            player_shape_yaw: link.map_or(0, |player| radians_to_s16(player.face)),
            partner_pos: None,
            goal: None,
            collision: solid,
            list: &world.list,
            active: &mut world.active,
            stories: &mut world.stories,
            ev_flags: &mut world.event_flags,
            events: &mut world.events,
            next: &mut world.next,
            text: Text {
                procs: &mut talk.procs,
                info: &mut talk.info,
                flags: &mut talk.flags,
                one: &mut talk.one,
                two: &mut talk.two,
                archive,
                clear_count,
            },
            cut: current.and_then(|(_, _, cut)| cut),
            cut_index: current.map(|(index, _, _)| index),
            advance: current.is_some_and(|(_, advance, _)| advance),
            event_mode,
            player_door: false,
            initial_room_ok: true,
            now_pos_move: true,
            room_loaded: ground::surface(solid, actor.pos).map(|_| actor.room),
            player_room: false,
            move_step: Vec3::ZERO,
            safe_poly: false,
            move_bg: false,
            move_bg_pos: None,
            ground_code: GROUND_CODE_NONE,
            special_code: SPECIAL_CODE_NONE,
            wall_hit: false,
            co_hit: false,
            tg_hit: false,
            tg_pos: Vec3::ZERO,
            wind_tg: false,
            wind_tg_pos: Vec3::ZERO,
            wind_push: Vec3::ZERO,
            wind: (Vec3::ZERO, 0.0),
            ship_pos: None,
            player_ship_ride: false,
            tact_music: NO_TACT_MUSIC,
            tact_end: false,
            cancel_button: false,
            player_lock: pad.lock.target.is_some(),
            player_lock_on: false,
            player_hang: false,
            lock_eye: None,
            btn_b: None,
            stick,
            stick_angle,
            camera: pad.camera.0,
            trig_a: pad.do_intent.0,
            trig_b: pad.target_intent.0,
            return_pressed: pad.target_intent.0,
            carry_now: pad.carry.held.is_some(),
            player_speed: link.map_or(0.0, |player| player.speed),
            player_anim: 0.0,
            player_anim_end: 0.0,
            do_status: 0,
            shock: None,
            boss_enemy: false,
            drawn: true,
        };
        makar.as_player = pad.carry.held == Some(entity) && !makar.changed_player;
        makar.ship_present = ctx.ship_pos.is_some();
        makar.link_present = ctx.link.is_some();
        if !makar.entered {
            makar.entered = true;
            makar.init(actor, &mut ctx);
        } else {
            makar.execute(&mut ctx);
        }
        actor.pos = makar.pos;
        actor.angle = [makar.pitch, makar.yaw, makar.roll];
        if let Some(pos) = ctx.move_bg_pos {
            actor.pos = pos;
        }
    }
}
