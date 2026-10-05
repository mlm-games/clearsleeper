use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::{Quat, Vec3};

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::flags::{Collect, EventFlags, StoryBits};
use super::inventory::{Equipped, Got};
use super::player::Player;
use super::stream::{add_calc_angle_s2, calc_speed};
use super::transition::{NextStage, Target};
use super::walk::{add_calc_angle_s, chase_f, from_s16, to_s16};
use crate::assets::MsgArchives;
use crate::text::driver::TextFlags;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgGameInfo, MsgProcs,
    fop_msgm_message_set, fop_msgm_message_set_actor,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_BOX_OPENING, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED,
    STATUS_MSG_DISPLAYED, STATUS_MSG_ENDS, STATUS_TYPING,
};

pub const GRAMMARIAN_ACTOR: &str = "Ba1";
pub const PARTNER_ACTOR: &str = "Sfairy";
pub const ARC_NAME: &str = "Ba";

pub const EVENT_NAMES: [&str; 6] = [
    "Use_Fairy",
    "Ba1_Get_Itm",
    "Ganbaru",
    "tale_1",
    "None",
    "tale_2",
];
pub const EVENT_USE_FAIRY: usize = 0;
pub const EVENT_GET_ITM: usize = 1;
pub const EVENT_GANBARU: usize = 2;
pub const EVENT_TALE_1: usize = 3;
pub const EVENT_NONE: usize = 4;
pub const EVENT_TALE_2: usize = 5;
pub const EVENT_ID_COUNT: usize = 6;

const _: () = assert!(EVENT_NAMES.len() == EVENT_ID_COUNT);

pub const CUT_NAMES: [&str; 2] = ["ACTION", "START_TALE1"];
pub const CUT_ACTION: i8 = 0;
pub const CUT_START_TALE1: i8 = 1;

pub const PRM_SHIFT: u32 = 0x00;
pub const PRM_MASK: u32 = 0xFF;

pub const HIO_MAX_HEAD_X: i16 = 0x1554;
pub const HIO_MAX_HEAD_Y: i16 = 0x2000;
pub const HIO_MIN_HEAD_X: i16 = -5000;
pub const HIO_MIN_HEAD_Y: i16 = -8192;
pub const HIO_MAX_BACKBONE_X: i16 = 0x0000;
pub const HIO_MAX_BACKBONE_Y: i16 = 0x1000;
pub const HIO_MIN_BACKBONE_X: i16 = 0x0000;
pub const HIO_MIN_BACKBONE_Y: i16 = -4096;
pub const HIO_MAX_TURN_STEP: i16 = 0x0400;
pub const HIO_CALC_ANGLE_TARGET: i16 = 0x0400;
pub const HIO_ATT_POS_OFFSET_Y: f32 = 115.0;
pub const HIO_DEBUG_MARK: f32 = 0.0;
pub const HIO_FAIRY_AREA: f32 = 500.0;
pub const HIO_IDLE_AREA: f32 = 300.0;

pub const EYE_OFFSET_OFF: [f32; 3] = [20.0, -16.0, 0.0];
pub const PLAYER_EYE_OFFSET_Y: f32 = -20.0;

pub const ATTN_DIST_TALK: i16 = 171;
pub const ATTN_DIST_SPEAK: i16 = 171;
pub const ATTN_DIST_TALK_SICK: i16 = 0x1B;
pub const ATTN_DIST_SPEAK_SICK: i16 = 0x1C;
pub const ATTN_DIST_TALK_HEALED: i16 = 0x5B;
pub const ATTN_DIST_SPEAK_HEALED: i16 = 0x59;
pub const ATTN_DIST_TALK_FINALE: i16 = 0xAB;
pub const ATTN_DIST_SPEAK_FINALE: i16 = 0xAB;

pub const GRAVITY: f32 = -4.5;
pub const GRAVITY_SICK: f32 = 0.0;
pub const MAX_FALL_SPEED: f32 = -100.0;

pub const ANM_NUM_INITIAL: i8 = 10;
pub const ANM_NUM_GETUP: i32 = 9;
pub const ANM_NUM_WAKE: i32 = 10;
pub const ANM_NUM_HOLD: i8 = 4;
pub const BTP_NUM_INITIAL: i8 = 1;
pub const BTP_NUM_WAKE: i8 = 9;
pub const BTP_NUM_GETUP: i8 = 0x0B;

pub const LOOP_NULL: i32 = -1;
pub const LOOP_NONE: i32 = 0;
pub const LOOP_LOOP: i32 = 2;

pub const ANM_NUM_ROWS: usize = 11;
pub const ANM_STT_ROWS: usize = 8;
pub const ANM_ATR_ROWS: usize = 21;
pub const BCK_ROWS: usize = 10;
pub const BTP_ROWS: usize = 12;

pub const STT_WAIT: i8 = 0;
pub const STT_WAIT_1: i8 = 1;
pub const STT_TALK: i8 = 2;
pub const STT_WAIT_3: i8 = 3;
pub const STT_SICK: i8 = 4;
pub const STT_TALK_5: i8 = 5;
pub const STT_WAIT_6: i8 = 6;
pub const STT_WAIT_7: i8 = 7;

pub const STAGE_INIT: i8 = 0;
pub const STAGE_RUN: i8 = 1;
pub const STAGE_SEARCHED: i8 = 2;
pub const STAGE_PLAY: i8 = 3;
pub const STAGE_TEARDOWN: i8 = 9;

pub const ORDER_NONE: i8 = 0;
pub const ORDER_SPEAK: i8 = 1;
pub const ORDER_CANTALK: i8 = 2;
pub const ORDER_EVENT_0: i8 = 3;
pub const ORDER_OFFSET: i8 = 3;
pub const ORDER_MARRIED: i8 = 4;
pub const ORDER_APPROACH: i8 = 6;
pub const EVENT_IDX_SPEAK: usize = 0x1EA;

pub const LOOK_NONE: i8 = 0;
pub const LOOK_PLAYER: i8 = 1;
pub const LOOK_STORED: i8 = 2;
pub const LOOK_ANGLE: i8 = 3;

pub const ATR_NONE: u8 = 0xFF;
pub const ATR_MAX: u8 = 0x14;
pub const ATR_LIMIT: u8 = 0x15;
pub const MESG_TAG_NONE: u8 = 0xFF;
pub const ANM_ATR_PARAM_6: u16 = 6;
pub const ANM_ATR_PARAM_14: u16 = 14;

pub const NO_STATUS: u8 = 0xFF;
pub const NO_INDEX: i16 = -1;
pub const NO_STAFF: usize = usize::MAX;
pub const NO_ACTION: i8 = -1;
pub const NO_ACT_NO: i8 = -1;

pub const BCK_05_BA_CUT8_WAIT: i32 = 0x0;
pub const BCK_HOLD: i32 = 0x1;
pub const BCK_SLEEP: i32 = 0x2;
pub const BCK_TALK01: i32 = 0x3;
pub const BCK_TALK02: i32 = 0x4;
pub const BCK_TALK03: i32 = 0x5;
pub const BCK_TALK04: i32 = 0x6;
pub const BCK_WAIT01: i32 = 0x7;
pub const BCK_WAIT02: i32 = 0x8;
pub const BCK_GETUP: i32 = 0x17;

pub const BTP_MABA01: i32 = 0x16;
pub const BTP_T01: i32 = 0xB;
pub const BTP_T02: i32 = 0xC;
pub const BTP_T03: i32 = 0xD;
pub const BTP_T04: i32 = 0xE;
pub const BTP_T05: i32 = 0xF;
pub const BTP_T06: i32 = 0x10;
pub const BTP_T07: i32 = 0x11;
pub const BTP_T08: i32 = 0x12;
pub const BTP_T09: i32 = 0x13;
pub const BTP_T10: i32 = 0x14;
pub const BTP_WAKE: i32 = 0x15;
pub const BTP_DEMO_T02_RES: i8 = 0xC;

pub const MSG_7E4: u32 = 0x7E4;
pub const MSG_7E5: u32 = 0x7E5;
pub const MSG_7E6: u32 = 0x7E6;
pub const MSG_7E7: u32 = 0x7E7;
pub const MSG_7E8: u32 = 0x7E8;
pub const MSG_7E9: u32 = 0x7E9;
pub const MSG_7EA: u32 = 0x7EA;
pub const MSG_7EB: u32 = 0x7EB;
pub const MSG_7EC: u32 = 0x7EC;
pub const MSG_7ED: u32 = 0x7ED;
pub const MSG_7EE: u32 = 0x7EE;
pub const MSG_7EF: u32 = 0x7EF;
pub const MSG_7F0: u32 = 0x7F0;
pub const MSG_7F1: u32 = 0x7F1;
pub const MSG_7F3: u32 = 0x7F3;
pub const MSG_7F4: u32 = 0x7F4;
pub const MSG_7F5: u32 = 0x7F5;
pub const MSG_7F6: u32 = 0x7F6;
pub const MSG_7F7: u32 = 0x7F7;
pub const MSG_7F9: u32 = 0x7F9;
pub const MSG_7FA: u32 = 0x7FA;
pub const MSG_7FB: u32 = 0x7FB;
pub const MSG_7FC: u32 = 0x7FC;
pub const MSG_7FD: u32 = 0x7FD;
pub const MSG_7FE: u32 = 0x7FE;
pub const MSG_7FF: u32 = 0x7FF;
pub const MSG_800: u32 = 0x800;
pub const MSG_801: u32 = 0x801;
pub const MSG_802: u32 = 0x802;
pub const MSG_803: u32 = 0x803;
pub const MSG_80B: u32 = 0x80B;
pub const MSG_7D8: u32 = 0x7d8;

pub const EVENT_BIT_0001: u16 = 0x0001;
pub const EVENT_BIT_0310: u16 = 0x0310;
pub const EVENT_BIT_0520: u16 = 0x0520;
pub const EVENT_BIT_0601: u16 = 0x0601;
pub const EVENT_BIT_0602: u16 = 0x0602;
pub const EVENT_BIT_0608: u16 = 0x0608;
pub const EVENT_BIT_0740: u16 = 0x0740;
pub const EVENT_BIT_0780: u16 = 0x0780;
pub const EVENT_BIT_0E20: u16 = 0x0E20;
pub const EVENT_BIT_2A20: u16 = 0x2A20;
pub const EVENT_BIT_2A80: u16 = 0x2A80;
pub const EVENT_BIT_3202: u16 = 0x3202;
pub const REG_A60F: u16 = 0xA60F;
pub const REG_A60F_LIMIT: u8 = 3;
pub const REG_LETTER: u16 = 0x9D03;
pub const LETTER_SEND: u8 = 1;

pub const TMP_BIT_0310: u32 = 0x0310;

pub const COLLECT_ARYLL: u8 = 0;
pub const COLLECT_SICK: u8 = 1;

pub const ITEM_NONE: u8 = 0xFF;
pub const ITEM_BOTTLE_SLOT_0: usize = 14;
pub const ITEM_BOTTLE_SLOTS: usize = 4;
pub const ITEM_EMPTY_BOTTLE: u8 = 0x50;
pub const ITEM_HALF_SOUP_BOTTLE: u8 = 0x54;
pub const ITEM_SOUP_BOTTLE: u8 = 0x55;
pub const ITEM_FAIRY_BOTTLE: u8 = 0x57;
pub const ITEM_BUTTON_COUNT: usize = 3;
pub const MSG_ACTOR_SLOT: u32 = 0;

pub const HEAP_SIZE: [u32; 1] = [0x272E0];

pub const CULL_MIN: [f32; 3] = [-50.0, -20.0, -50.0];
pub const CULL_MAX: [f32; 3] = [50.0, 120.0, 50.0];

pub const COLLISION_RADIUS: f32 = 50.0;
pub const COLLISION_HEIGHT: f32 = 110.0;
pub const COLLISION_RADIUS_SLIM: f32 = 30.0;

pub const MOVE_TARGET: [f32; 3] = [503.0, 0.0, -91.0];
pub const MOVE_REACH: f32 = 4.0;
pub const MOVE_SPEED: f32 = 4.0;
pub const MOVE_SPEED_STEP: f32 = 0.4;
pub const MOVE_EYE_STEP: f32 = 2.0;
pub const AREA_HEIGHT: f32 = 100.0;
pub const AREA_ANGLE: f32 = 61.0;
pub const AREA_DY_STEP: f32 = 1.0;

pub const EV_GOAL_OFFSET: [f32; 3] = [0.0, 0.0, 50.0];
pub const EV_PLAYER_INIT_OFFSET: [f32; 3] = [0.0, 0.0, 180.0];
pub const EV_FINALE_OFFSET: [f32; 3] = [0.0, 0.0, 90.0];

pub const EV_ACT_LOOK: i8 = 0;
pub const EV_ACT_GOAL: i8 = 1;
pub const EV_ACT_PLAYER_INIT: i8 = 2;
pub const EV_ACT_FAIRY_END: i8 = 3;
pub const EV_ACT_MOVE: i8 = 4;
pub const EV_ACT_PLAYER_TURN: i8 = 5;
pub const EV_ACT_DRAW: i8 = 6;
pub const EV_ACT_CHK_FAIRY: i8 = 7;
pub const EV_ACT_EYE_OFF: i8 = 8;
pub const EV_ACT_EYE_OFF_ZERO: i8 = 9;
pub const EV_ACT_CHK_FAIRY_MOVE: i8 = 10;

pub const CUT_KEY_ACT_NO: &str = "ActNo";
pub const CUT_KEY_PRM_0: &str = "prm_0";
pub const CUT_KEY_PRM_1: &str = "prm_1";
pub const CUT_KEY_TIMER: &str = "Timer";
pub const CUT_KEY_ATTEN: &str = "Atten";
pub const CUT_KEY_SPEED: &str = "Speed";

pub const SICK_POS: [f32; 3] = [-290.0, 0.0, 110.0];
pub const SICK_ANGLE: i16 = 0;
pub const TALE_ANGLE: i16 = -0x8000;

pub const TURN_SCALE: i16 = 4;
pub const TURN_MAX_STEP: i16 = 0x800;
pub const MOVE_TURN_MAX_STEP: i16 = 0x400;
pub const TURN_MIN_STEP: i16 = 0;
pub const TALK_TURN_SCALE: i16 = 2;
pub const TALK_TURN_MAX_STEP: i16 = 0x1000;
pub const TALK_TURN_MIN_STEP: i16 = 1;
pub const LOOK_VEL_SCALE: i16 = 4;
pub const LOOK_VEL_MAX_STEP: i16 = 0x800;
pub const HOLD_LOOK_OFFSET: i16 = -0x638E;
pub const JNT_FOLLOW_SCALE: i16 = 4;
pub const JNT_SCALE: i32 = 4;
pub const JNT_MIN_STEP: i32 = 1;
pub const ANG_MIN_STEP: i16 = 0x60;

pub const DEG2S: f32 = 32768.0 / 180.0;

pub const IDLE_TIMER: i16 = 60;
pub const BLINK_TIMER_BASE: i16 = 30;
pub const BLINK_TIMER_SPREAD: i16 = 60;

pub const SHADOW_LIFT: f32 = 150.0;
pub const SHADOW_RADIUS: f32 = 800.0;
pub const SHADOW_HEIGHT: f32 = 40.0;

pub const NEXT_STAGE_LINK_RM: &str = "LinkRM";
pub const NEXT_STAGE_TALE_CLEARED: u8 = 202;
pub const NEXT_STAGE_TALE: u8 = 200;
pub const NEXT_STAGE_TALE_ROOM: u8 = 0;
pub const NEXT_STAGE_MARRIED: u8 = 201;
pub const NEXT_STAGE_MARRIED_ROOM: u8 = 0;

pub const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
pub const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
pub const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
pub const RND_SEED: i32 = 100;

pub const JNT_ROWS: usize = 2;
pub const JNT_HEAD: usize = 0;
pub const JNT_SPINE: usize = 1;
pub const JNT_X: usize = 0;
pub const JNT_Y: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnmPrm {
    pub anm: i8,
    pub btp: i8,
    pub morf: f32,
    pub speed: f32,
    pub loop_mode: i32,
}

const BCK: [i32; BCK_ROWS] = [
    BCK_WAIT01,
    BCK_TALK01,
    BCK_TALK02,
    BCK_TALK03,
    BCK_HOLD,
    BCK_WAIT02,
    BCK_TALK04,
    BCK_SLEEP,
    BCK_05_BA_CUT8_WAIT,
    BCK_GETUP,
];

const BTP: [i32; BTP_ROWS] = [
    BTP_MABA01,
    BTP_T01,
    BTP_T02,
    BTP_T03,
    BTP_T04,
    BTP_T05,
    BTP_T06,
    BTP_T07,
    BTP_T08,
    BTP_T09,
    BTP_T10,
    BTP_WAKE,
];

const ANM_NUM: [AnmPrm; ANM_NUM_ROWS] = [
    AnmPrm { anm: 0, btp: 0, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 1, btp: 0, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 2, btp: 0, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 3, btp: 0, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 4, btp: 0, morf: 8.0, speed: 1.0, loop_mode: LOOP_NONE },
    AnmPrm { anm: 5, btp: 0, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 6, btp: 0, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 7, btp: 0, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 8, btp: 0, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 9, btp: 0x0B, morf: 8.0, speed: 1.0, loop_mode: LOOP_NONE },
    AnmPrm { anm: 0, btp: 9, morf: 0.0, speed: 1.0, loop_mode: LOOP_LOOP },
];

const ANM_STT: [AnmPrm; ANM_STT_ROWS] = [
    AnmPrm { anm: -1, btp: -1, morf: 0.0, speed: 0.0, loop_mode: LOOP_NULL },
    AnmPrm { anm: 0, btp: 0, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: -1, btp: -1, morf: 0.0, speed: 0.0, loop_mode: LOOP_NULL },
    AnmPrm { anm: 0, btp: 2, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 7, btp: 7, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: -1, btp: -1, morf: 0.0, speed: 0.0, loop_mode: LOOP_NULL },
    AnmPrm { anm: 0, btp: 9, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 0, btp: 0, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
];

const ANM_ATR: [AnmPrm; ANM_ATR_ROWS] = [
    AnmPrm { anm: 0, btp: 0, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 1, btp: 0, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 2, btp: 0, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 3, btp: 0, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 0, btp: 1, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 2, btp: 2, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 1, btp: 3, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 3, btp: 2, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 2, btp: 3, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 2, btp: 4, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 2, btp: 6, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 3, btp: 5, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 3, btp: 6, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 7, btp: 7, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 6, btp: 8, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 6, btp: 1, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 5, btp: 0x0A, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 5, btp: 1, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 5, btp: 9, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 5, btp: 0, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
    AnmPrm { anm: 0, btp: 8, morf: 8.0, speed: 1.0, loop_mode: LOOP_LOOP },
];

const _: () = assert!(ANM_NUM.len() == ANM_NUM_ROWS);
const _: () = assert!(ANM_STT.len() == ANM_STT_ROWS);
const _: () = assert!(ANM_ATR.len() == ANM_ATR_ROWS);
const _: () = assert!(BCK.len() == BCK_ROWS);
const _: () = assert!(BTP.len() == BTP_ROWS);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proc {
    None,
    Wait1,
    Wait2,
    Demo,
    Wait3,
    Wait4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct JntParam {
    pub max_angles: [[i16; JNT_ROWS]; JNT_ROWS],
    pub min_angles: [[i16; JNT_ROWS]; JNT_ROWS],
    pub max_turn_step: [[i16; JNT_ROWS]; JNT_ROWS],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Jnt {
    pub param: JntParam,
    pub angles: [[i16; JNT_ROWS]; JNT_ROWS],
    pub head_lock: bool,
    pub backbone_lock: bool,
    pub trn: bool,
}

impl Jnt {
    pub fn set_trn(&mut self) {
        self.trn = true;
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Grammarian {
    pub prm: i8,
    pub kind: i8,
    pub kind_type: i8,
    pub decided: bool,
    pub proc_state: Proc,
    pub stage: i8,
    pub entered: bool,
    pub event_ids: [i16; EVENT_ID_COUNT],
    pub event_index: i16,
    pub staff_idx: usize,
    pub partner: Option<Entity>,
    pub partner_baba: bool,
    pub order: i8,
    pub cantalk_item: bool,
    pub xy_callbacks: bool,
    pub stt: i8,
    pub prev_stt: i8,
    pub order_talk: bool,
    pub look_state: i8,
    pub look_angle: i16,
    pub look_vel: i16,
    pub action_index: i8,
    pub act_no: i8,
    pub item_no: u8,
    pub atr: u8,
    pub atr_ready: u8,
    pub mesg_anime_tag: u8,
    pub anm_num: i8,
    pub btp_num: i8,
    pub btp_frame: u8,
    pub btp_frame_max: u8,
    pub blink_timer: i16,
    pub ev_timer: i16,
    pub idle_timer: i16,
    pub hold_event: bool,
    pub fairy_used: bool,
    pub attention: bool,
    pub set_eye_pos: bool,
    pub in_demo: bool,
    pub head_only_follow: bool,
    pub morf_stopped: bool,
    pub primed: bool,
    pub draw_skip: bool,
    pub hide_model: bool,
    pub angle_locked: bool,
    pub move_locked: bool,
    pub no_collision: bool,
    pub slim_collision: bool,
    pub heal_reg: u8,
    pub heal_told: bool,
    pub unk_7f0: bool,
    pub unk_7fb: bool,
    pub unk_7ff: bool,
    pub joint_head_y: i16,
    pub joint_backbone_y: i16,
    pub actor_angle_y: i16,
    pub speed_f: f32,
    pub speed_y: f32,
    pub gravity: f32,
    pub prev_morf_frame: f32,
    pub eye_offset: f32,
    pub eye_offset_zero: f32,
    pub home: Vec3,
    pub home_angle: [i16; 3],
    pub angle: [i16; 3],
    pub shape_angle: [i16; 3],
    pub init_pos: Vec3,
    pub transformed_eye: Vec3,
    pub attn_pos: Vec3,
    pub eye_pos: Vec3,
    pub look_origin: Vec3,
    pub look_target: Option<Vec3>,
    pub collision_radius: f32,
    pub collision_height: f32,
    pub attn_dist_talk: i16,
    pub attn_dist_speak: i16,
    pub msg_bs_id: u32,
    pub msg_no: u32,
    pub msg_bound: bool,
    pub jnt: Jnt,
    pub seed: [i32; 3],
}

impl Grammarian {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            prm: ((parameters >> PRM_SHIFT) & PRM_MASK) as i8,
            kind: -1,
            kind_type: 0,
            decided: false,
            proc_state: Proc::None,
            stage: STAGE_INIT,
            entered: false,
            event_ids: [NO_INDEX; EVENT_ID_COUNT],
            event_index: NO_INDEX,
            staff_idx: NO_STAFF,
            partner: None,
            partner_baba: false,
            order: ORDER_NONE,
            cantalk_item: false,
            xy_callbacks: false,
            stt: 0,
            prev_stt: 0,
            order_talk: false,
            look_state: LOOK_NONE,
            look_angle: 0,
            look_vel: 0,
            action_index: NO_ACTION,
            act_no: NO_ACT_NO,
            item_no: ITEM_NONE,
            atr: ATR_NONE,
            atr_ready: 0,
            mesg_anime_tag: MESG_TAG_NONE,
            anm_num: ANM_NUM_INITIAL,
            btp_num: BTP_NUM_INITIAL,
            btp_frame: 0,
            btp_frame_max: 0,
            blink_timer: 0,
            ev_timer: 0,
            idle_timer: 0,
            hold_event: false,
            fairy_used: false,
            attention: false,
            set_eye_pos: false,
            in_demo: false,
            head_only_follow: false,
            morf_stopped: false,
            primed: false,
            draw_skip: false,
            hide_model: false,
            angle_locked: false,
            move_locked: false,
            no_collision: false,
            slim_collision: false,
            heal_reg: 0,
            heal_told: false,
            unk_7f0: false,
            unk_7fb: false,
            unk_7ff: false,
            joint_head_y: 0,
            joint_backbone_y: 0,
            actor_angle_y: 0,
            speed_f: 0.0,
            speed_y: 0.0,
            gravity: GRAVITY,
            prev_morf_frame: 0.0,
            eye_offset: 0.0,
            eye_offset_zero: 0.0,
            home: Vec3::ZERO,
            home_angle: [0; 3],
            angle: [0; 3],
            shape_angle: [0; 3],
            init_pos: Vec3::ZERO,
            transformed_eye: Vec3::ZERO,
            attn_pos: Vec3::ZERO,
            eye_pos: Vec3::ZERO,
            look_origin: Vec3::ZERO,
            look_target: None,
            collision_radius: 0.0,
            collision_height: 0.0,
            attn_dist_talk: ATTN_DIST_TALK,
            attn_dist_speak: ATTN_DIST_SPEAK,
            msg_bs_id: FPC_M_ERROR_PROCESS_ID,
            msg_no: 0,
            msg_bound: false,
            jnt: Jnt::default(),
            seed: [RND_SEED; 3],
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        !(self.draw_skip || self.hide_model)
    }

    pub fn default_jnt_param() -> JntParam {
        JntParam {
            max_angles: [
                [HIO_MAX_HEAD_X, HIO_MAX_HEAD_Y],
                [HIO_MAX_BACKBONE_X, HIO_MAX_BACKBONE_Y],
            ],
            min_angles: [
                [HIO_MIN_HEAD_X, HIO_MIN_HEAD_Y],
                [HIO_MIN_BACKBONE_X, HIO_MIN_BACKBONE_Y],
            ],
            max_turn_step: [[HIO_MAX_TURN_STEP; JNT_ROWS]; JNT_ROWS],
        }
    }
}

const _: () = assert!(BCK_ROWS == 10);
const _: () = assert!(BTP_ROWS == 12);

pub fn is_grammarian(name: &str) -> bool {
    name == GRAMMARIAN_ACTOR
}

pub fn cut_action_of(name: &str) -> Option<i8> {
    CUT_NAMES
        .iter()
        .position(|cut| *cut == name)
        .map(|index| index as i8)
}

fn table_of<T, const N: usize>(table: &[T; N], index: i32) -> Option<&T> {
    usize::try_from(index).ok().and_then(|slot| table.get(slot))
}

fn calc_timer(timer: &mut i16) -> i16 {
    if *timer != 0 {
        *timer -= 1;
    }
    *timer
}

fn deg2s(deg: f32) -> i16 {
    (deg * DEG2S) as i32 as i16
}

fn target_angle_y(from: Vec3, to: Vec3) -> i16 {
    to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn target_angle_x(from: Vec3, to: Vec3) -> i16 {
    let delta = to - from;
    to_s16(delta.y.atan2((delta.x * delta.x + delta.z * delta.z).sqrt()))
}

fn abs_xz(value: Vec3) -> f32 {
    (value.x * value.x + value.z * value.z).sqrt()
}

fn rotated_y(offset: Vec3, pos: Vec3, yaw: i16) -> Vec3 {
    pos + Quat::from_rotation_y(from_s16(yaw)) * offset
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

fn distance_angle_s(x: i16, y: i16) -> i32 {
    i32::from(x.wrapping_sub(y)).abs()
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

fn rnd_value(seed: &mut [i32; 3], min: i16, range: i16) -> i16 {
    min.wrapping_add((rnd(seed) * (range) as f32) as i16)
}

fn bottle_flag(item: u8) -> Option<(usize, u8)> {
    Some(match item {
        0x4F => (0, 1),
        ITEM_EMPTY_BOTTLE => (0, 2),
        0x51 => (0, 3),
        0x52 => (0, 4),
        0x53 => (0, 5),
        ITEM_HALF_SOUP_BOTTLE => (0, 6),
        ITEM_SOUP_BOTTLE => (0, 7),
        0x56 => (1, 1),
        ITEM_FAIRY_BOTTLE => (1, 2),
        0x58 => (1, 3),
        0x59 => (1, 4),
        _ => return None,
    })
}

fn is_get_bottle_item(got: &Got, item: u8) -> bool {
    bottle_flag(item).is_some_and(|(field, bit)| got.has(field, bit))
}

fn check_bottle(equipped: &Equipped, item: u8) -> u8 {
    let mut count = 0_u8;
    for slot in ITEM_BOTTLE_SLOT_0..ITEM_BOTTLE_SLOT_0 + ITEM_BOTTLE_SLOTS {
        if equipped.item(slot) == item {
            count = count.wrapping_add(1);
        }
    }
    count
}

fn check_empty_bottle(equipped: &Equipped) -> bool {
    check_bottle(equipped, ITEM_EMPTY_BOTTLE) != 0
}

fn set_bottle_item_in(equipped: &mut Equipped, prev: u8, next: u8) {
    for slot in ITEM_BOTTLE_SLOT_0..ITEM_BOTTLE_SLOT_0 + ITEM_BOTTLE_SLOTS {
        if equipped.item(slot) == prev {
            equipped.set(slot, next);
            return;
        }
    }
}

fn set_empty_bottle_item_in(equipped: &mut Equipped, item: u8) {
    set_bottle_item_in(equipped, ITEM_EMPTY_BOTTLE, item);
}

impl Grammarian {
    fn jnt_angle(&self, row: usize, axis: usize) -> i16 {
        self.jnt
            .angles
            .get(row)
            .and_then(|slot| slot.get(axis))
            .copied()
            .unwrap_or(0)
    }

    fn jnt_set_angle(&mut self, row: usize, axis: usize, value: i16) {
        if let Some(slot) = self.jnt.angles.get_mut(row) {
            if let Some(cell) = slot.get_mut(axis) {
                *cell = value;
            }
        }
    }

    fn jnt_max(&self, row: usize, axis: usize) -> i16 {
        self.jnt
            .param
            .max_angles
            .get(row)
            .and_then(|slot| slot.get(axis))
            .copied()
            .unwrap_or(0)
    }

    fn jnt_min(&self, row: usize, axis: usize) -> i16 {
        self.jnt
            .param
            .min_angles
            .get(row)
            .and_then(|slot| slot.get(axis))
            .copied()
            .unwrap_or(0)
    }

    fn jnt_max_step(&self, row: usize, axis: usize) -> i16 {
        self.jnt
            .param
            .max_turn_step
            .get(row)
            .and_then(|slot| slot.get(axis))
            .copied()
            .unwrap_or(0)
    }

    fn jnt_ang_calc_s(&mut self, out: &mut i16, target_y: i16, speed: i16, max_vel: i16) -> bool {
        let mut diff = out.wrapping_sub(target_y);
        let orig_diff = diff;
        diff = add_calc_angle_s(diff, 0, speed, max_vel, ANG_MIN_STEP);
        if i32::from(diff).abs() > i32::from(speed) {
            *out = out.wrapping_add(diff.wrapping_sub(orig_diff));
        } else {
            *out = target_y;
        }
        target_y.wrapping_sub(*out) == 0
    }

    fn jnt_follow(&mut self, out_y: &mut i16, target_y: i16, max_vel: i16) -> bool {
        let orig_y = *out_y;
        let _ = self.jnt_ang_calc_s(out_y, target_y, JNT_FOLLOW_SCALE, max_vel);
        let delta_y = out_y.wrapping_sub(orig_y);
        let mut s_var6 = 0_i16;
        let mut s_var5 = 0_i16;
        let s_var1 = target_y.wrapping_sub(*out_y);
        for row in [JNT_SPINE, JNT_HEAD] {
            s_var5 = s_var5.wrapping_add(self.jnt_angle(row, JNT_Y));
            if (delta_y > 0 && s_var5 > s_var1) || (delta_y < 0 && s_var5 < s_var1) {
                let max = self.jnt_max(row, JNT_Y);
                let min = self.jnt_min(row, JNT_Y);
                let mut value = s_var1.wrapping_sub(s_var6);
                if value > max {
                    value = max;
                }
                if value < min {
                    value = min;
                }
                self.jnt_set_angle(row, JNT_Y, value);
            }
            s_var6 = s_var6.wrapping_add(self.jnt_angle(row, JNT_Y));
        }
        target_y != *out_y
    }

    fn jnt_move(&mut self, input: i16, axis: usize) -> bool {
        let mut input = input;
        let mut angles = [0_i16; JNT_ROWS];
        for row in [JNT_HEAD, JNT_SPINE] {
            let locked = if row == JNT_HEAD {
                self.jnt.head_lock
            } else {
                self.jnt.backbone_lock
            };
            let mut angle = if locked { 0 } else { input };
            let max = self.jnt_max(row, axis);
            let min = self.jnt_min(row, axis);
            if angle > max {
                angle = max;
            }
            if angle < min {
                angle = min;
            }
            input = input.wrapping_sub(angle);
            if let Some(slot) = angles.get_mut(row) {
                *slot = angle;
            }
            let current = i32::from(self.jnt_angle(row, axis));
            let max_step = self.jnt_max_step(row, axis);
            let next = add_calc_angle_l(
                current,
                i32::from(angle),
                JNT_SCALE,
                i32::from(max_step),
                JNT_MIN_STEP,
            );
            self.jnt_set_angle(row, axis, next as i16);
        }
        let mut reached = 0;
        for row in [JNT_HEAD, JNT_SPINE] {
            let want = angles.get(row).copied().unwrap_or(0);
            let gap = distance_angle_s(want, self.jnt_angle(row, axis));
            if gap <= i32::from(self.jnt_max_step(row, axis)) / 2 {
                reached += 1;
            }
        }
        reached == JNT_ROWS as i32
    }

    fn jnt_look_at_target(
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
        let mut out_y = out_init;
        let delta_y = target_y.wrapping_sub(out_y);
        let max_y = self
            .jnt_max(JNT_SPINE, JNT_Y)
            .wrapping_add(self.jnt_max(JNT_HEAD, JNT_Y));
        let min_y = self
            .jnt_min(JNT_SPINE, JNT_Y)
            .wrapping_add(self.jnt_min(JNT_HEAD, JNT_Y));
        let reached = self.jnt_move(delta_y, JNT_Y);
        if (self.jnt.trn || ((delta_y >= max_y || delta_y <= min_y) && reached)) && !no_follow {
            self.jnt.trn = self.jnt_follow(&mut out_y, target_y, max_vel);
        }
        let _ = self.jnt_move(target_x, JNT_X);
        out_y
    }

    pub fn set_btp(&mut self, btp: i8) -> bool {
        if table_of(&BTP, i32::from(btp)).is_none() {
            return false;
        }
        self.btp_frame = 0;
        self.blink_timer = 0;
        true
    }

    pub fn ini_tex_pttrn_anm(&mut self, btp: i8) -> bool {
        self.set_btp(btp)
    }

    pub fn ply_tex_pttrn_anm(&mut self) {
        if self.btp_num != 0 || calc_timer(&mut self.blink_timer) == 0 {
            self.btp_frame = self.btp_frame.wrapping_add(1);
            if self.btp_frame >= self.btp_frame_max {
                if self.btp_num != 0 {
                    self.btp_frame = self.btp_frame_max;
                } else {
                    let timer = rnd_value(&mut self.seed, BLINK_TIMER_BASE, BLINK_TIMER_SPREAD);
                    self.blink_timer = timer;
                    self.btp_frame = 0;
                }
            }
        }
    }

    pub fn set_anm_tex(&mut self, btp: i8) {
        if self.btp_num != btp {
            self.btp_num = btp;
            let btp = self.btp_num;
            self.ini_tex_pttrn_anm(btp);
        }
    }

    pub fn set_anm_anm(&mut self, prm: AnmPrm) {
        if self.anm_num == prm.anm {
            return;
        }
        self.anm_num = prm.anm;
        let _ = table_of(&BCK, i32::from(prm.anm)).copied();
        let _ = (prm.morf, prm.speed, prm.loop_mode);
        self.prev_morf_frame = 0.0;
        self.morf_stopped = false;
    }

    pub fn set_anm_num(&mut self, index: i32, modify: bool) {
        let Some(prm) = table_of(&ANM_NUM, index).copied() else {
            return;
        };
        if modify {
            self.set_anm_tex(prm.btp);
        }
        self.set_anm_anm(prm);
    }

    pub fn set_anm(&mut self) {
        let Some(prm) = table_of(&ANM_STT, i32::from(self.stt)).copied() else {
            return;
        };
        if prm.btp >= 0 {
            self.set_anm_tex(prm.btp);
        }
        if prm.anm >= 0 {
            self.set_anm_anm(prm);
        }
    }

    pub fn chng_anm_tag(&mut self) {}

    pub fn ctrl_anm_tag(&mut self) {}

    pub fn chng_anm_atr(&mut self, atr: u8) {
        if atr >= ATR_LIMIT || atr == self.atr {
            return;
        }
        self.atr = atr;
        self.set_anm_atr(true);
    }

    pub fn ctrl_anm_atr(&mut self) {
        match self.atr {
            5 | 6 => {
                if self.look_state != LOOK_PLAYER {
                    self.look_state = LOOK_PLAYER;
                    self.jnt.set_trn();
                }
            }
            _ => {}
        }
    }

    pub fn set_anm_atr(&mut self, modify: bool) {
        let Some(prm) = table_of(&ANM_ATR, i32::from(self.atr)).copied() else {
            return;
        };
        if modify {
            self.set_anm_tex(prm.btp);
        }
        self.set_anm_anm(prm);
    }

    pub fn anm_atr(&mut self, param: u16, ctx: &mut Ctx) {
        match param {
            ANM_ATR_PARAM_6 => {
                if self.atr_ready == 0 {
                    let atr = ctx.text.info.mesg_anime;
                    self.chng_anm_atr(atr);
                    self.atr_ready = self.atr_ready.wrapping_add(1);
                }
                let tag = ctx.text.info.mesg_anime_tag;
                ctx.text.info.clear_mesg_anime_tag_info();
                if tag != MESG_TAG_NONE && u32::from(self.mesg_anime_tag) != u32::from(tag) {
                    self.mesg_anime_tag = tag;
                    self.chng_anm_tag();
                }
            }
            ANM_ATR_PARAM_14 => self.atr_ready = 0,
            _ => {}
        }
        self.ctrl_anm_tag();
        self.ctrl_anm_atr();
    }

    pub fn set_stt(&mut self, stt: i8) {
        let prev = self.stt;
        self.idle_timer = 0;
        self.stt = stt;
        match self.stt {
            STT_TALK => {
                self.atr = ATR_NONE;
                if self.anm_num != ANM_NUM_HOLD {
                    self.look_state = LOOK_PLAYER;
                    self.jnt.set_trn();
                }
                self.prev_stt = prev;
                return;
            }
            STT_TALK_5 => {
                self.atr = ATR_NONE;
                self.prev_stt = prev;
                return;
            }
            STT_WAIT_1 | STT_WAIT_3 | STT_SICK | STT_WAIT_6 | STT_WAIT_7 => {
                self.head_only_follow = true;
            }
            _ => {}
        }
        self.set_anm();
    }

    pub fn set_attention(&mut self, pos: Vec3, set_eye_pos: bool) {
        self.attn_pos = Vec3::new(pos.x, pos.y + HIO_ATT_POS_OFFSET_Y, pos.z);
        if !self.set_eye_pos && !set_eye_pos {
            return;
        }
        self.eye_pos = self.transformed_eye;
        self.eye_pos.y += self.eye_offset;
    }

    pub fn attention_pos(&self) -> Vec3 {
        self.attn_pos
    }

    pub fn item(&self) -> u8 {
        self.item_no
    }

    pub fn cantalk_item(&self) -> bool {
        self.cantalk_item
    }

    pub fn has_xy_callbacks(&self) -> bool {
        self.xy_callbacks
    }

    pub fn hold_event_on(&mut self) {
        self.hold_event = true;
    }

    pub fn heap_size(&self) -> u32 {
        HEAP_SIZE
            .get(usize::from(self.kind_type as u8))
            .copied()
            .unwrap_or(HEAP_SIZE[0])
    }
}

impl Grammarian {
    pub fn event_slot(&self) -> Option<usize> {
        usize::try_from(self.event_index).ok()
    }

    pub fn event_slot_of(&self, index: i16) -> Option<usize> {
        let slot = usize::try_from(index).ok()?;
        let id = self.event_ids.get(slot).copied().unwrap_or(NO_INDEX);
        usize::try_from(id).ok()
    }

    pub fn is_event_entry(&self) -> i32 {
        i32::try_from(self.staff_idx).unwrap_or(-1)
    }

    pub fn end_event(&mut self) {
        self.atr = ATR_NONE;
    }

    pub fn xy_check_cb(&self, item_btn: u8, select_item: [u8; ITEM_BUTTON_COUNT]) -> bool {
        select_item.get(usize::from(item_btn)).copied() == Some(ITEM_FAIRY_BOTTLE)
    }

    pub fn xy_event_cb(&self) -> i16 {
        self.event_ids.get(EVENT_USE_FAIRY).copied().unwrap_or(NO_INDEX)
    }

    pub fn chk_talk(&mut self, ctx: &Ctx) -> bool {
        let mut ret = true;
        self.item_no = ITEM_NONE;
        if ctx.talk_xy {
            if ctx.present_done {
                self.item_no = ctx.present_item;
            } else {
                ret = false;
            }
            self.item_no = ctx.present_item;
        }
        ret
    }

    pub fn check_command_talk(&self, ctx: &Ctx) -> bool {
        ctx.talk_command && !ctx.talk_xy
    }

    pub fn chk_attention(&self, ctx: &Ctx) -> bool {
        ctx.attention
    }

    pub fn chk_drct(&self, actor: &Actor, ctx: &Ctx, deg: f32) -> bool {
        let target = target_angle_y(actor.pos, player_pos(ctx)).wrapping_sub(actor.angle[1]);
        i32::from(target).abs() < i32::from(deg2s(deg))
    }

    pub fn chk_parts_not_mov(&self, actor: &Actor) -> bool {
        self.joint_head_y == self.jnt_angle(JNT_HEAD, JNT_Y)
            && self.joint_backbone_y == self.jnt_angle(JNT_SPINE, JNT_Y)
            && self.actor_angle_y == actor.angle[1]
    }

    pub fn next_msg_status(&self, msg_no: &mut u32, ctx: &Ctx) -> u8 {
        let mut ret = STATUS_MSG_CONTINUES;
        match *msg_no {
            MSG_7EB => *msg_no = MSG_7EC,
            MSG_7EF => *msg_no = MSG_7F0,
            MSG_7F3 => *msg_no = MSG_7F4,
            MSG_7F4 => {
                if check_empty_bottle(ctx.equipped) {
                    *msg_no = MSG_7F5;
                } else {
                    *msg_no = MSG_7F9;
                }
            }
            MSG_7F6 => *msg_no = MSG_7F7,
            MSG_7FB | MSG_7FC | MSG_7FD | MSG_7FA => {
                if is_get_bottle_item(ctx.got, ITEM_SOUP_BOTTLE) {
                    if !check_empty_bottle(ctx.equipped) {
                        *msg_no = MSG_803;
                    } else if check_bottle(ctx.equipped, ITEM_SOUP_BOTTLE) != 0 {
                        *msg_no = MSG_7FE;
                    } else {
                        *msg_no = MSG_7FF;
                    }
                    return ret;
                }
                if check_empty_bottle(ctx.equipped) {
                    *msg_no = MSG_7F5;
                } else {
                    *msg_no = MSG_7F9;
                }
            }
            MSG_7FF => {
                match ctx.text.select_num() {
                    0 => *msg_no = MSG_800,
                    1 => *msg_no = MSG_802,
                    _ => {}
                }
            }
            _ => ret = STATUS_MSG_ENDS,
        }
        ret
    }

    pub fn get_msg_ba1_0(&self, ctx: &Ctx) -> u32 {
        if self.heal_told {
            return MSG_7D8;
        }
        if self.hold_event {
            return if ctx.stories.test(EVENT_BIT_0608) {
                MSG_7E7
            } else {
                MSG_7E6
            };
        }
        if self.unk_7fb {
            MSG_7E5
        } else {
            MSG_7E4
        }
    }

    pub fn get_msg_ba1_1(&self, ctx: &Ctx) -> u32 {
        if ctx.stories.test(EVENT_BIT_0E20) {
            if ctx.collect_at(COLLECT_SICK) != 0 {
                return MSG_80B;
            }
            return if ctx.stories.test(EVENT_BIT_0740) {
                MSG_7EE
            } else {
                MSG_7EF
            };
        }
        if ctx.stories.test(EVENT_BIT_0780) {
            return MSG_7ED;
        }
        if ctx.collect_at(COLLECT_ARYLL) != 0 {
            return if ctx.stories.test(EVENT_BIT_0602) {
                MSG_7EA
            } else {
                MSG_7EB
            };
        }
        if ctx.stories.test(EVENT_BIT_0601) {
            MSG_7E9
        } else {
            MSG_7E8
        }
    }

    pub fn get_msg_ba1_3(&mut self, ctx: &mut Ctx) -> u32 {
        if self.unk_7f0 {
            if ctx.stories.test(EVENT_BIT_2A20) {
                return MSG_801;
            }
            return MSG_7F6;
        }
        if ctx.stories.test(EVENT_BIT_2A20) {
            ctx.stories.set_reg(REG_A60F, 0);
            if self.heal_reg >= REG_A60F_LIMIT {
                if self.heal_told {
                    return MSG_7FD;
                }
                self.heal_told = true;
                return MSG_7FC;
            }
            if self.heal_told {
                return MSG_7FA;
            }
            self.heal_told = true;
            return MSG_7FB;
        }
        MSG_7F1
    }

    pub fn get_msg_ba1_4(&mut self, ctx: &mut Ctx) -> u32 {
        self.get_msg_ba1_3(ctx)
    }

    pub fn get_msg(&mut self, ctx: &mut Ctx) -> u32 {
        match self.kind {
            0 => self.get_msg_ba1_0(ctx),
            1 => self.get_msg_ba1_1(ctx),
            3 => self.get_msg_ba1_3(ctx),
            4 => self.get_msg_ba1_4(ctx),
            _ => 0,
        }
    }

    pub fn npc_talk(&mut self, ctx: &mut Ctx) -> u8 {
        let mut status = NO_STATUS;
        if self.msg_bs_id == FPC_M_ERROR_PROCESS_ID {
            let msg_no = self.get_msg(ctx);
            self.msg_no = msg_no;
            let eye = self.eye_pos;
            self.msg_bs_id = ctx.text.message_set_actor(self.msg_no, eye);
            self.msg_bound = false;
        } else if self.msg_bound {
            status = ctx.text.status().unwrap_or(NO_STATUS);
            if status == STATUS_MSG_DISPLAYED {
                let mut next_no = self.msg_no;
                let next = self.next_msg_status(&mut next_no, ctx);
                self.msg_no = next_no;
                ctx.text.set_status(next);
                if next == STATUS_MSG_CONTINUES {
                    let msg_no = self.msg_no;
                    ctx.text.message_set(msg_no, Vec3::ZERO);
                }
            } else if status == STATUS_BOX_CLOSED {
                ctx.text.set_status(STATUS_MSG_DESTROYED);
                self.msg_bs_id = FPC_M_ERROR_PROCESS_ID;
            }
            self.anm_atr(u16::from(status), ctx);
        } else {
            self.msg_bound = ctx.text.procs.search_by_id(self.msg_bs_id).is_some();
        }
        status
    }

    pub fn talk_1(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        let ret = self.chk_parts_not_mov(actor);
        if self.anm_num == ANM_NUM_HOLD {
            actor.angle[1] = add_calc_angle_s(
                actor.angle[1],
                self.look_angle,
                TALK_TURN_SCALE,
                TALK_TURN_MAX_STEP,
                TALK_TURN_MIN_STEP,
            );
            if !self.morf_stopped {
                return true;
            }
        }
        self.npc_talk(ctx);
        if ctx.text.status() == Some(STATUS_MSG_DESTROYED) {
            match self.msg_no {
                MSG_7E4 => self.unk_7fb = true,
                MSG_7E6 => ctx.stories.set(EVENT_BIT_0608),
                MSG_7E8 => ctx.stories.set(EVENT_BIT_0601),
                MSG_7EC => ctx.stories.set(EVENT_BIT_0602),
                MSG_7F0 => ctx.stories.set(EVENT_BIT_0740),
                MSG_800 => {
                    if self.msg_no != MSG_7F9 {
                        self.order = ORDER_MARRIED;
                    }
                    self.fairy_used = false;
                }
                MSG_801 => self.unk_7f0 = false,
                _ => {}
            }
            self.item_no = ITEM_NONE;
            self.order_talk = false;
            let prev = self.prev_stt;
            self.set_stt(prev);
            self.idle_timer = IDLE_TIMER;
            self.end_event();
            self.hold_event = false;
        }
        ret
    }

    pub fn talk_2(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        self.talk_1(actor, ctx)
    }
}

impl Grammarian {
    fn turn_home(&self, actor: &mut Actor) {
        actor.angle[1] = add_calc_angle_s(
            actor.angle[1],
            self.home_angle[1],
            TURN_SCALE,
            TURN_MAX_STEP,
            TURN_MIN_STEP,
        );
    }

    fn idle_look(&mut self, actor: &Actor, ctx: &Ctx) {
        self.order = ORDER_CANTALK;
        if self.attention {
            self.idle_timer = IDLE_TIMER;
        }
        if calc_timer(&mut self.idle_timer) != 0 && self.chk_drct(actor, ctx, AREA_ANGLE) {
            self.look_state = LOOK_PLAYER;
        } else {
            self.look_state = LOOK_ANGLE;
            self.look_angle = self.home_angle[1];
            self.jnt.set_trn();
        }
    }

    pub fn check_use_fairy_area(&self, actor: &Actor, ctx: &Ctx) -> bool {
        let player = player_pos(ctx);
        let mag = abs_xz(player - actor.pos);
        (player.y - actor.pos.y).abs() < AREA_HEIGHT && mag < HIO_FAIRY_AREA
    }

    pub fn wait_0(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        self.turn_home(actor);
        if self.order == ORDER_SPEAK || self.order >= ORDER_OFFSET {
            return true;
        }
        self.order = ORDER_NONE;
        if !ctx.tmp.test(TMP_BIT_0310) {
            let player = player_pos(ctx);
            let dist = abs_xz(player - actor.pos);
            if (player.y - actor.pos.y).abs() < AREA_HEIGHT && dist < HIO_IDLE_AREA {
                ctx.tmp.set(TMP_BIT_0310);
                self.order = ORDER_APPROACH;
            }
        }
        true
    }

    pub fn wait_1(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        self.turn_home(actor);
        if self.order == ORDER_SPEAK || self.order >= ORDER_OFFSET {
            return true;
        }
        if self.order_talk {
            if self.chk_talk(ctx) {
                self.set_stt(STT_TALK);
                self.head_only_follow = false;
            }
            return true;
        }
        if self.hold_event {
            self.set_anm_num(i32::from(ANM_NUM_HOLD), true);
            self.order = ORDER_SPEAK;
            self.look_state = LOOK_NONE;
            self.look_angle = actor.angle[1].wrapping_add(HOLD_LOOK_OFFSET);
        } else {
            self.idle_look(actor, ctx);
        }
        true
    }

    pub fn wait_2(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        self.turn_home(actor);
        if self.order == ORDER_SPEAK || self.order >= ORDER_OFFSET {
            return true;
        }
        if self.order_talk {
            if self.unk_7ff {
                ctx.next.target = Some(Target {
                    stage: NEXT_STAGE_LINK_RM.to_owned(),
                    room: NEXT_STAGE_MARRIED_ROOM,
                    start: NEXT_STAGE_MARRIED,
                });
                return true;
            }
            if self.chk_talk(ctx) {
                self.set_stt(STT_TALK);
                self.head_only_follow = false;
            }
            return true;
        }
        if ctx.collect_at(COLLECT_SICK) == 0 && ctx.stories.test(EVENT_BIT_3202) {
            actor.pos = Vec3::from_array(SICK_POS);
            actor.angle[1] = TALE_ANGLE;
            self.home = actor.pos;
            self.home_angle = actor.angle;
            if player_pos(ctx).y - actor.pos.y < AREA_DY_STEP {
                self.order = ORDER_SPEAK;
                self.unk_7ff = true;
            }
            self.look_state = LOOK_ANGLE;
            self.look_angle = self.home_angle[1];
            self.jnt.set_trn();
            return true;
        }
        self.idle_look(actor, ctx);
        true
    }

    pub fn wait_3(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        self.wait_2(actor, ctx)
    }

    pub fn zzzwai(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        if self.order == ORDER_SPEAK || self.order >= ORDER_OFFSET {
            return true;
        }
        if self.fairy_used {
            self.fairy_used = false;
            self.order = ORDER_MARRIED;
            return true;
        }
        if self.order_talk {
            if self.chk_talk(ctx) {
                self.set_stt(STT_TALK_5);
            }
            return true;
        }
        if ctx.stories.test(EVENT_BIT_2A20) {
            self.idle_look(actor, ctx);
        } else {
            self.order = ORDER_NONE;
            if self.check_use_fairy_area(actor, ctx) {
                self.order = ORDER_CANTALK;
            }
        }
        true
    }

    pub fn look_back(&mut self, actor: &mut Actor, ctx: &Ctx) {
        self.joint_head_y = self.jnt_angle(JNT_HEAD, JNT_Y);
        self.joint_backbone_y = self.jnt_angle(JNT_SPINE, JNT_Y);
        self.actor_angle_y = actor.angle[1];
        let mut src_pos = Vec3::new(actor.pos.x, actor.pos.y, actor.pos.z);
        src_pos.y = self.eye_pos.y;
        let mut dst_pos = Vec3::ZERO;
        let mut dst_pos_p: Option<Vec3> = None;
        let mut desired_yrot = actor.angle[1];
        match self.look_state {
            LOOK_PLAYER => {
                dst_pos = player_eye_pos(ctx, PLAYER_EYE_OFFSET_Y);
                dst_pos_p = Some(dst_pos);
                src_pos = Vec3::new(actor.pos.x, actor.pos.y, actor.pos.z);
                src_pos.y = self.eye_pos.y;
            }
            LOOK_STORED => {
                dst_pos = self.init_pos;
                dst_pos_p = Some(dst_pos);
                src_pos = Vec3::new(actor.pos.x, actor.pos.y, actor.pos.z);
                src_pos.y = self.eye_pos.y;
            }
            LOOK_ANGLE => desired_yrot = self.look_angle,
            _ => {}
        }
        add_calc_angle_s2(
            &mut self.look_vel,
            HIO_CALC_ANGLE_TARGET,
            LOOK_VEL_SCALE,
            LOOK_VEL_MAX_STEP,
        );
        if !self.jnt.trn {
            self.look_vel = 0;
        }
        let max_vel = self.look_vel;
        let no_follow = self.head_only_follow;
        self.look_origin = src_pos;
        self.look_target = dst_pos_p;
        actor.angle[1] = self.jnt_look_at_target(
            actor.angle[1],
            dst_pos_p,
            src_pos,
            desired_yrot,
            max_vel,
            no_follow,
        );
    }

    pub fn decide_type(&mut self) -> bool {
        if self.decided {
            return true;
        }
        self.decided = true;
        self.kind = -1;
        match self.prm {
            0..=4 => self.kind = self.prm,
            _ => {}
        }
        self.kind >= 0
    }

    pub fn create_init(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        for (slot, name) in EVENT_NAMES.iter().enumerate() {
            let id = ctx
                .list
                .event_index(name)
                .and_then(|index| i16::try_from(index).ok())
                .unwrap_or(NO_INDEX);
            if let Some(entry) = self.event_ids.get_mut(slot) {
                *entry = id;
            }
        }
        self.staff_idx = ctx.list.staff_of(GRAMMARIAN_ACTOR).unwrap_or(NO_STAFF);
        self.cantalk_item = false;
        self.attn_dist_talk = ATTN_DIST_TALK;
        self.attn_dist_speak = ATTN_DIST_SPEAK;
        self.gravity = GRAVITY;
        self.anm_num = ANM_NUM_INITIAL;
        self.init_pos = ctx.pos;
        let init_success = match self.kind {
            0 => self.init_ba1_0(actor, ctx),
            1 => self.init_ba1_1(actor, ctx),
            2 => self.init_ba1_2(actor, ctx),
            3 => self.init_ba1_3(actor, ctx),
            4 => self.init_ba1_4(actor, ctx),
            _ => false,
        };
        if !init_success {
            return false;
        }
        self.angle = actor.angle;
        self.shape_angle = actor.angle;
        self.heal_reg = ctx.stories.reg(REG_A60F);
        self.set_attention(ctx.pos, true);
        true
    }

    pub fn init_ba1_0(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        if !ctx.stories.test(EVENT_BIT_0520) && !ctx.stories.test(EVENT_BIT_0001) {
            self.set_action(Proc::Wait1, actor, ctx);
            return true;
        }
        false
    }

    pub fn init_ba1_1(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        let mut ret = !ctx.stories.test(EVENT_BIT_0520);
        if !ret {
            return ret;
        }
        ret = ctx.stories.test(EVENT_BIT_0001);
        if ret {
            if ctx.collect_at(COLLECT_SICK) != 0 {
                actor.pos = Vec3::from_array(SICK_POS);
                actor.angle[1] = SICK_ANGLE;
            }
            self.set_action(Proc::Wait2, actor, ctx);
        }
        ret
    }

    pub fn init_ba1_2(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        self.set_action(Proc::Demo, actor, ctx);
        true
    }

    pub fn init_ba1_3(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        let mut ret = ctx.stories.test(EVENT_BIT_0520);
        if !ret {
            return ret;
        }
        ret = !ctx.stories.test(EVENT_BIT_2A20);
        if ret {
            self.attn_dist_talk = ATTN_DIST_TALK_SICK;
            self.attn_dist_speak = ATTN_DIST_SPEAK_SICK;
            self.draw_skip = true;
            self.set_action(Proc::Wait3, actor, ctx);
            self.gravity = GRAVITY_SICK;
            self.xy_callbacks = true;
        }
        ret
    }

    pub fn init_ba1_4(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        let mut ret = ctx.stories.test(EVENT_BIT_0520);
        if !ret {
            return ret;
        }
        ret = ctx.stories.test(EVENT_BIT_2A20);
        if ret {
            self.set_action(Proc::Wait4, actor, ctx);
        }
        ret
    }

    pub fn set_action(&mut self, next: Proc, actor: &mut Actor, ctx: &mut Ctx) {
        if self.proc_state == next {
            return;
        }
        if self.proc_state != Proc::None {
            self.stage = STAGE_TEARDOWN;
            self.run_proc(actor, ctx);
        }
        self.proc_state = next;
        self.stage = STAGE_INIT;
        self.run_proc(actor, ctx);
    }

    pub fn run_proc(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        match self.proc_state {
            Proc::Wait1 => self.wait_action1(actor, ctx),
            Proc::Wait2 => self.wait_action2(actor, ctx),
            Proc::Demo => self.demo_action1(),
            Proc::Wait3 => self.wait_action3(actor, ctx),
            Proc::Wait4 => self.wait_action4(actor, ctx),
            Proc::None => {}
        }
    }

    pub fn wait_action1(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        match self.stage {
            STAGE_INIT => {
                if ctx.stories.test(EVENT_BIT_2A80) {
                    self.heal_told = ctx.tmp.test(TMP_BIT_0310);
                    if self.heal_told {
                        ctx.tmp.clear(TMP_BIT_0310);
                    }
                    self.set_stt(STT_WAIT_1);
                } else {
                    self.set_stt(STT_WAIT_7);
                }
                self.stage = self.stage.wrapping_add(1);
            }
            STAGE_RUN | STAGE_SEARCHED | STAGE_PLAY => {
                self.attention = self.chk_attention(ctx);
                let result = match self.stt {
                    STT_WAIT_7 => self.wait_0(actor, ctx),
                    STT_WAIT_1 => self.wait_1(actor, ctx),
                    STT_TALK => self.talk_1(actor, ctx),
                    _ => false,
                };
                self.set_eye_pos = result;
                self.look_back(actor, ctx);
            }
            _ => {}
        }
    }

    pub fn wait_action2(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        match self.stage {
            STAGE_INIT => {
                self.set_stt(STT_WAIT_3);
                self.stage = self.stage.wrapping_add(1);
            }
            STAGE_RUN | STAGE_SEARCHED | STAGE_PLAY => {
                self.attention = self.chk_attention(ctx);
                let result = match self.stt {
                    STT_WAIT_3 => self.wait_2(actor, ctx),
                    STT_TALK => self.talk_1(actor, ctx),
                    _ => false,
                };
                self.set_eye_pos = result;
                self.look_back(actor, ctx);
            }
            _ => {}
        }
    }

    pub fn demo_action1(&mut self) {
        match self.stage {
            STAGE_INIT => self.stage = self.stage.wrapping_add(1),
            _ => {}
        }
    }

    pub fn wait_action3(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        match self.stage {
            STAGE_INIT => {
                self.set_stt(STT_SICK);
                self.stage = self.stage.wrapping_add(1);
            }
            STAGE_RUN | STAGE_SEARCHED | STAGE_PLAY => {
                self.attention = self.chk_attention(ctx);
                let result = match self.stt {
                    STT_SICK => self.zzzwai(actor, ctx),
                    STT_TALK_5 => self.talk_2(actor, ctx),
                    _ => false,
                };
                self.set_eye_pos = result;
                self.look_back(actor, ctx);
            }
            _ => {}
        }
    }

    pub fn wait_action4(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        match self.stage {
            STAGE_INIT => {
                self.set_stt(STT_WAIT_6);
                self.stage = self.stage.wrapping_add(1);
            }
            STAGE_RUN | STAGE_SEARCHED | STAGE_PLAY => {
                self.attention = self.chk_attention(ctx);
                let result = match self.stt {
                    STT_WAIT_6 => self.wait_3(actor, ctx),
                    STT_TALK => self.talk_1(actor, ctx),
                    _ => false,
                };
                self.set_eye_pos = result;
                self.look_back(actor, ctx);
            }
            _ => {}
        }
    }
}

impl Grammarian {
    pub fn event_order(&mut self, ctx: &mut Ctx) {
        let condition = self.order;
        if condition == ORDER_SPEAK || condition == ORDER_CANTALK {
            if self.kind == 3 && !ctx.stories.test(EVENT_BIT_2A20) {
                self.cantalk_item = true;
            }
            if condition == ORDER_SPEAK {
                ctx.active.order(ctx.list, EVENT_IDX_SPEAK, ctx.entity);
            }
        } else if condition >= ORDER_OFFSET {
            self.event_index = condition.wrapping_sub(ORDER_OFFSET);
            if let Some(index) = self.event_slot_of(self.event_index) {
                ctx.active.order(ctx.list, index, ctx.entity);
            }
        }
    }

    pub fn check_order(&mut self, ctx: &Ctx) {
        if ctx.demo_accept {
            if let Some(index) = self.event_slot_of(self.event_index)
                && ctx.active.start_check(index)
            {
                self.order = ORDER_NONE;
            }
        } else if self.check_command_talk(ctx)
            && (self.order == ORDER_SPEAK || self.order == ORDER_CANTALK)
        {
            self.order = ORDER_NONE;
            self.order_talk = true;
        }
    }

    pub fn partner_search_sub(&mut self, others: &Query<&Actor, Without<Grammarian>>) -> bool {
        self.partner = None;
        let found = others
            .iter()
            .find(|(_, actor)| actor.name == PARTNER_ACTOR)
            .map(|(entity, _)| *entity);
        self.partner = found;
        found.is_some()
    }

    pub fn partner_search(&mut self) {
        if self.stage == STAGE_RUN {
            self.stage = self.stage.wrapping_add(1);
        }
    }

    pub fn demo(&mut self, actor: &Actor, ctx: &Ctx) -> bool {
        if ctx.demo_actor.unwrap_or(0) == 0 {
            if self.in_demo {
                self.in_demo = false;
            }
        } else {
            self.in_demo = true;
            self.btp_frame = self.btp_frame.wrapping_add(1);
            if self.btp_frame >= self.btp_frame_max {
                self.btp_frame = self.btp_frame_max;
            }
            self.btp_num = BTP_DEMO_T02_RES;
            self.btp_frame = 0;
            self.angle = actor.angle;
            self.shape_angle = self.angle;
        }
        self.in_demo
    }

    pub fn set_mtx(&mut self, actor: &Actor) {
        if !self.in_demo {
            self.ply_tex_pttrn_anm();
        }
        self.set_attention(actor.pos, false);
    }

    pub fn ev_set_timer(&mut self, timer: Option<i32>) {
        self.ev_timer = 0;
        if let Some(value) = timer {
            self.ev_timer = value as i16;
        }
    }

    pub fn ev_goal(&mut self, ctx: &mut Ctx) {
        ctx.event_goal = Some(rotated_y(
            Vec3::from_array(EV_GOAL_OFFSET),
            ctx.pos,
            ctx.angle[1],
        ));
    }

    pub fn ev_player_init(&mut self, ctx: &mut Ctx) {
        let target = rotated_y(
            Vec3::from_array(EV_PLAYER_INIT_OFFSET),
            ctx.pos,
            ctx.angle[1],
        );
        let yaw = target_angle_y(target, ctx.pos);
        if let Some(player) = ctx.player.as_deref_mut() {
            player.pos = target;
            player.yaw = from_s16(yaw);
            player.face = 0.0;
            player.speed = 0.0;
            player.speed_y = 0.0;
            player.drift = Vec3::ZERO;
        }
    }

    pub fn ev_player_turn(&mut self, ctx: &mut Ctx) {
        ctx.player_demo_angle = Some(target_angle_y(player_pos(ctx), ctx.pos));
    }

    pub fn ev_fairy_end(&mut self) {
        self.attn_dist_talk = ATTN_DIST_TALK_HEALED;
        self.attn_dist_speak = ATTN_DIST_SPEAK_HEALED;
        self.fairy_used = true;
    }

    pub fn ev_move_init(&mut self) {
        self.angle_locked = true;
        self.hide_model = true;
        self.no_collision = true;
    }

    pub fn ev_draw_control(
        &mut self,
        actor: &mut Actor,
        ctx: &Ctx,
        prm_0: Option<i32>,
        prm_1: Option<i32>,
    ) {
        if let Some(value) = prm_0 {
            match value {
                0 => ctx.player_no_draw = true,
                1 => ctx.player_no_draw = false,
                _ => {}
            }
        }
        if let Some(value) = prm_1 {
            match value {
                0 => self.hide_model = true,
                1 => self.hide_model = false,
                2 => {
                    self.set_anm_num(ANM_NUM_WAKE, true);
                    self.hide_model = true;
                    return;
                }
                3 => {
                    actor.angle[1] = target_angle_y(actor.pos, player_pos(ctx));
                    self.attn_dist_talk = ATTN_DIST_TALK_FINALE;
                    self.attn_dist_speak = ATTN_DIST_SPEAK_FINALE;
                    actor.pos = rotated_y(
                        Vec3::from_array(EV_FINALE_OFFSET),
                        actor.pos,
                        actor.angle[1],
                    );
                    self.hide_model = false;
                }
                _ => {}
            }
        }
    }

    pub fn event_action_init(
        &mut self,
        cut: Option<usize>,
        actor: &mut Actor,
        ctx: &mut Ctx,
    ) {
        let Some(act_no) = cut.and_then(|at| ctx.list.cut_number(at, CUT_KEY_ACT_NO)) else {
            return;
        };
        self.act_no = act_no as i8;
        match self.act_no {
            EV_ACT_LOOK => self.set_anm_num(ANM_NUM_GETUP, true),
            EV_ACT_GOAL => self.ev_goal(ctx),
            EV_ACT_PLAYER_INIT => self.ev_player_init(ctx),
            EV_ACT_FAIRY_END => self.ev_fairy_end(),
            EV_ACT_MOVE => self.ev_move_init(),
            EV_ACT_PLAYER_TURN => self.ev_player_turn(ctx),
            EV_ACT_DRAW => {
                let prm_0 = cut.and_then(|at| ctx.list.cut_number(at, CUT_KEY_PRM_0));
                let prm_1 = cut.and_then(|at| ctx.list.cut_number(at, CUT_KEY_PRM_1));
                self.ev_draw_control(actor, ctx, prm_0, prm_1);
            }
            EV_ACT_CHK_FAIRY => {
                let timer = cut.and_then(|at| ctx.list.cut_number(at, CUT_KEY_TIMER));
                self.ev_set_timer(timer);
            }
            EV_ACT_EYE_OFF => {
                let atten = cut.and_then(|at| ctx.list.cut_float(at, CUT_KEY_ATTEN));
                self.eye_offset = atten.unwrap_or(0.0);
            }
            EV_ACT_EYE_OFF_ZERO => {
                let speed = cut.and_then(|at| ctx.list.cut_float(at, CUT_KEY_SPEED));
                self.eye_offset_zero = speed.unwrap_or(0.0);
            }
            EV_ACT_CHK_FAIRY_MOVE => {
                let timer = cut.and_then(|at| ctx.list.cut_number(at, CUT_KEY_TIMER));
                self.ev_set_timer(timer);
            }
            _ => {}
        }
    }

    pub fn ev_move_mov_pos(&mut self, actor: &mut Actor, ctx: &Ctx) -> bool {
        let temp = Vec3::from_array(MOVE_TARGET);
        if calc_timer(&mut self.ev_timer) != 0 {
            return false;
        }
        if self.eye_offset as i32 != 0 {
            chase_f(&mut self.eye_offset, 0.0, MOVE_EYE_STEP);
            return false;
        }
        let mag = abs_xz(temp - actor.pos);
        let ret = mag < MOVE_REACH;
        self.angle[1] = target_angle_y(actor.pos, player_pos(ctx));
        self.angle_locked = true;
        if ret {
            actor.angle[1] = self.angle[1];
            self.angle_locked = false;
            self.speed_f = 0.0;
            self.eye_offset = 0.0;
        } else {
            actor.angle[1] = add_calc_angle_s(
                actor.angle[1],
                target_angle_y(actor.pos, temp),
                TURN_SCALE,
                MOVE_TURN_MAX_STEP,
                TURN_MIN_STEP,
            );
            chase_f(&mut self.speed_f, MOVE_SPEED, MOVE_SPEED_STEP);
        }
        ret
    }

    pub fn ev_move_chk_fairy(&mut self, others: &Query<&Actor, Without<Grammarian>>) -> bool {
        if calc_timer(&mut self.ev_timer) == 0 {
            return !self.partner_search_sub(others);
        }
        false
    }

    pub fn ev_move_eye_off_zero(&mut self) -> bool {
        let mut ret = self.eye_offset_zero as i32 == 0;
        if ret {
            self.eye_offset = 0.0;
        } else {
            let step = self.eye_offset_zero;
            chase_f(&mut self.eye_offset, 0.0, step);
            ret = self.eye_offset as i32 == 0;
            if ret {
                self.eye_offset = 0.0;
            }
        }
        ret
    }

    pub fn ev_move_chk_fairy_mov(
        &mut self,
        others: &Query<&Actor, Without<Grammarian>>,
        ctx: &mut Ctx,
    ) -> bool {
        let mut ret = false;
        if calc_timer(&mut self.ev_timer) == 0
            && self.partner_search_sub(others)
            && self.partner_baba
        {
            ret = true;
        }
        if ret {
            self.ev_player_init(ctx);
        }
        ret
    }

    pub fn event_action(
        &mut self,
        actor: &mut Actor,
        others: &Query<&Actor, Without<Grammarian>>,
        ctx: &mut Ctx,
    ) -> bool {
        match self.act_no {
            EV_ACT_MOVE => self.ev_move_mov_pos(actor, ctx),
            EV_ACT_CHK_FAIRY => self.ev_move_chk_fairy(others),
            EV_ACT_EYE_OFF_ZERO => self.ev_move_eye_off_zero(),
            EV_ACT_CHK_FAIRY_MOVE => self.ev_move_chk_fairy_mov(others, ctx),
            _ => true,
        }
    }

    pub fn cut_init_start_tale1(&mut self, cut: Option<usize>, ctx: &Ctx) {
        let timer = cut.and_then(|at| ctx.list.cut_number(at, CUT_KEY_TIMER));
        self.ev_set_timer(timer);
    }

    pub fn cut_move_start_tale1(&mut self, ctx: &mut Ctx) -> bool {
        if calc_timer(&mut self.ev_timer) == 0 {
            let start = if ctx.clear_count != 0 {
                NEXT_STAGE_TALE_CLEARED
            } else {
                NEXT_STAGE_TALE
            };
            ctx.next.target = Some(Target {
                stage: NEXT_STAGE_LINK_RM.to_owned(),
                room: NEXT_STAGE_TALE_ROOM,
                start,
            });
        }
        self.ev_timer == 0
    }

    fn current_cut(&self, staff: i32, ctx: &Ctx) -> Option<usize> {
        if staff < 0 {
            return None;
        }
        let event = self.event_slot_of(self.event_index)?;
        let slot = usize::try_from(staff).ok()?;
        let cut = ctx.active.cut_of(event, slot)?;
        usize::try_from(cut).ok()
    }

    pub fn private_cut(
        &mut self,
        staff: i32,
        actor: &mut Actor,
        others: &Query<&Actor, Without<Grammarian>>,
        ctx: &mut Ctx,
    ) {
        if staff < 0 {
            return;
        }
        let cut = self.current_cut(staff, ctx);
        ctx.cut_action = cut
            .and_then(|at| ctx.list.cut_name(at))
            .and_then(cut_action_of);
        let Some(action) = ctx.cut_action else {
            return;
        };
        self.action_index = action;
        if ctx.cut_advance {
            match action {
                CUT_ACTION => self.event_action_init(cut, actor, ctx),
                CUT_START_TALE1 => self.cut_init_start_tale1(cut, ctx),
                _ => {}
            }
        }
        let cut_end = match action {
            CUT_ACTION => self.event_action(actor, others, ctx),
            CUT_START_TALE1 => self.cut_move_start_tale1(ctx),
            _ => true,
        };
        let _ = cut_end;
    }

    pub fn event_proc(
        &mut self,
        staff: i32,
        actor: &mut Actor,
        others: &Query<&Actor, Without<Grammarian>>,
        ctx: &mut Ctx,
    ) {
        let ended = self
            .event_slot_of(self.event_index)
            .is_some_and(|index| ctx.active.end_check(index));
        if ended {
            match self.event_slot() {
                Some(EVENT_GET_ITM) => {
                    if check_bottle(ctx.equipped, ITEM_HALF_SOUP_BOTTLE) != 0 {
                        set_bottle_item_in(ctx.equipped, ITEM_HALF_SOUP_BOTTLE, ITEM_SOUP_BOTTLE);
                    } else {
                        set_empty_bottle_item_in(ctx.equipped, ITEM_SOUP_BOTTLE);
                    }
                    if ctx.stories.test(EVENT_BIT_2A20) {
                        self.order = ORDER_SPEAK;
                        self.unk_7f0 = true;
                    } else {
                        self.order = ORDER_MARRIED;
                    }
                }
                Some(EVENT_GANBARU) => {
                    ctx.stories.set_reg(REG_LETTER, LETTER_SEND);
                    ctx.stories.set(EVENT_BIT_2A20);
                    self.set_action(Proc::Wait4, actor, ctx);
                    self.home = actor.pos;
                    self.home_angle = actor.angle;
                    self.no_collision = false;
                }
                _ => {}
            }
            self.end_event();
            return;
        }
        if !ctx.cut_proc {
            self.private_cut(staff, actor, others, ctx);
        }
        self.look_back(actor, ctx);
    }

    pub fn execute(
        &mut self,
        actor: &mut Actor,
        others: &Query<&Actor, Without<Grammarian>>,
        ctx: &mut Ctx,
    ) {
        if !self.primed {
            self.home = actor.pos;
            self.home_angle = actor.angle;
            self.primed = true;
        }
        self.jnt.param = Self::default_jnt_param();
        if self.draw_skip && ctx.demo_actor.unwrap_or(0) == 0 {
            return;
        }
        self.move_locked = false;
        self.draw_skip = false;
        self.partner_search();
        self.check_order(ctx);
        if !self.demo(actor, ctx) {
            let mut cond: i32 = -1;
            if ctx.text.info.event_run_check() && !self.check_command_talk(ctx) {
                cond = self.is_event_entry();
            }
            if cond >= 0 {
                self.set_eye_pos = true;
                self.event_proc(cond, actor, others, ctx);
            } else {
                self.run_proc(actor, ctx);
            }
            if !self.move_locked {
                let (delta, next_speed) = calc_speed(
                    self.speed_f,
                    actor.angle[1],
                    self.speed_y,
                    self.gravity,
                    MAX_FALL_SPEED,
                );
                actor.pos += delta;
                self.speed_f = next_speed;
                self.speed_y = delta.y;
            }
            if !self.angle_locked {
                self.angle = actor.angle;
                self.shape_angle = self.angle;
            }
        }
        self.event_order(ctx);
        self.set_mtx(actor);
        if !self.in_demo && !self.no_collision {
            self.collision_radius = if self.slim_collision {
                COLLISION_RADIUS_SLIM
            } else {
                COLLISION_RADIUS
            };
            self.collision_height = COLLISION_HEIGHT;
        }
    }
}

fn player_pos(ctx: &Ctx) -> Vec3 {
    ctx.player
        .as_deref()
        .map_or(ctx.pos, |player| player.pos)
}

fn player_eye_pos(ctx: &Ctx, offset_y: f32) -> Vec3 {
    ctx.player.as_deref().map_or(ctx.pos, |player| {
        Vec3::new(player.pos.x, player.pos.y + offset_y, player.pos.z)
    })
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
    pub fn proc_name(&self) -> Option<u16> {
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

    pub fn message_set_actor(&mut self, msg_no: u32, eye: Vec3) -> u32 {
        let Some(archive) = self.archive else {
            return FPC_M_ERROR_PROCESS_ID;
        };
        fop_msgm_message_set_actor(
            self.procs,
            self.info,
            archive,
            self.clear_count,
            msg_no,
            MSG_ACTOR_SLOT,
            eye,
        )
    }
}

pub struct Ctx<'a> {
    pub entity: Entity,
    pub pos: Vec3,
    pub angle: [i16; 3],
    pub player: Option<&'a mut Player>,
    pub clear_count: u8,
    pub collect: &'a Collect,
    pub equipped: &'a mut Equipped,
    pub got: &'a Got,
    pub stories: &'a mut StoryBits,
    pub tmp: &'a mut EventFlags,
    pub list: &'a EventList,
    pub active: &'a mut ActiveEvent,
    pub next: &'a mut NextStage,
    pub text: &'a mut Text<'a>,
    pub attention: bool,
    pub demo_accept: bool,
    pub talk_command: bool,
    pub talk_xy: bool,
    pub present_done: bool,
    pub present_item: u8,
    pub demo_actor: Option<i32>,
    pub select_item: [u8; ITEM_BUTTON_COUNT],
    pub cut_action: Option<i8>,
    pub cut_advance: bool,
    pub cut_proc: bool,
    pub event_goal: Option<Vec3>,
    pub player_demo_angle: Option<i16>,
    pub player_no_draw: bool,
}

impl Ctx<'_> {
    pub fn collect_at(&self, idx: u8) -> u8 {
        self.collect.check(idx)
    }

    pub fn event_goal(&self) -> Option<Vec3> {
        self.event_goal
    }

    pub fn player_demo_angle(&self) -> Option<i16> {
        self.player_demo_angle
    }

    pub fn player_draw_disabled(&self) -> bool {
        self.player_no_draw
    }
}

#[derive(SystemParam)]
pub struct World<'w> {
    pub stories: ResMut<'w, StoryBits>,
    pub list: Res<'w, EventList>,
    pub active: ResMut<'w, ActiveEvent>,
    pub equipped: ResMut<'w, Equipped>,
    pub got: Res<'w, Got>,
    pub collect: Res<'w, Collect>,
    pub tmp: ResMut<'w, EventFlags>,
    pub next: ResMut<'w, NextStage>,
}

#[derive(SystemParam)]
pub struct Msg<'w, 's> {
    pub procs: ResMut<'w, MsgProcs>,
    pub info: ResMut<'w, MsgGameInfo>,
    pub one: ResMut<'w, Msg1Box>,
    pub two: ResMut<'w, Msg2Box>,
    pub text_flags: Res<'w, TextFlags>,
    pub archive: Option<Res<'w, MsgArchives>>,
    pub players: Query<'w, 's, &'static mut Player>,
}

pub fn control(
    world: World,
    msg: Msg,
    others: Query<&Actor, Without<Grammarian>>,
    mut actors: Query<(Entity, &mut Actor, &mut Grammarian)>,
) {
    let mut player = msg.players.iter_mut().next();
    let clear_count = msg.text_flags.clear_count;
    let archive = msg
        .archive
        .as_deref()
        .map(|archive| archive as &dyn MsgArchive);
    for (entity, actor, gram) in &mut actors {
        let actor = &mut *actor;
        let gram = &mut *gram;
        let mut text = Text {
            procs: &mut msg.procs,
            info: &mut msg.info,
            one: &mut msg.one,
            two: &mut msg.two,
            clear_count,
            archive,
        };
        let mut ctx = Ctx {
            entity,
            pos: actor.pos,
            angle: actor.angle,
            player: player.as_deref_mut(),
            clear_count,
            collect: &*world.collect,
            equipped: &mut *world.equipped,
            got: &*world.got,
            stories: &mut *world.stories,
            tmp: &mut *world.tmp,
            list: &*world.list,
            active: &mut *world.active,
            next: &mut *world.next,
            text: &mut text,
            attention: false,
            demo_accept: false,
            talk_command: false,
            talk_xy: false,
            present_done: false,
            present_item: ITEM_NONE,
            demo_actor: None,
            select_item: [ITEM_NONE; ITEM_BUTTON_COUNT],
            cut_action: None,
            cut_advance: false,
            cut_proc: false,
            event_goal: None,
            player_demo_angle: None,
            player_no_draw: false,
        };
        if !gram.entered {
            gram.entered = true;
            if !gram.decide_type() {
                continue;
            }
            if !gram.create_init(actor, &mut ctx) {
                continue;
            }
        }
        gram.execute(actor, &others, &mut ctx);
    }
}