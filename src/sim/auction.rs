use bevy_ecs::prelude::*;

use super::isle_timer::{self, IsleTimer};
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList, ItemPartner, PresentItem};
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, OrderFlags, StoryBits};
use super::intent::{DoIntent, TargetIntent};
use super::pickup::Rupees;
use super::player::Player;
use super::transition::{NextStage, Target};
use super::walk;
use crate::assets::MsgArchives;
use crate::text::driver::{Counters, Layout, TextFlags};
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgGameInfo, MsgProcs, fop_msgm_message_set,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED,
    STATUS_MSG_ENDS,
};

pub const AUCTION_ACTORS: [&str; 1] = ["Auction"];

pub fn is_auction(name: &str) -> bool {
    AUCTION_ACTORS.contains(&name)
}

const PROC_ERROR: u32 = 0xFFFF_FFFF;

const NO_EVENT: i16 = -1;
const NO_BYTE: u8 = 0xFF;
const NO_MSG: u32 = 0;
const TALK_IDLE: u8 = 0xFF;

const NPC_COUNT: usize = 8;
const GAUGE_COUNT: usize = 7;
const REST_MSG_MAX: usize = 3;
const AFTER_BET_MAX: i32 = 4;
const ITEM_COUNT: usize = 4;
const ITEM_PICK_MAX: usize = 3;
const ITEM2_COUNT: usize = 2;
const CAMERA_PRESETS: usize = 3;
const SHUFFLE_ROUNDS: i32 = 100;
const SHUFFLE_RANGE: i32 = 6;
const SHUFFLE_OFFSET: usize = 1;
const GAUGE_LO: usize = 1;
const GAUGE_HI: usize = 7;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const CULL_MIN: [f32; 3] = [-500.0, -300.0, -700.0];
const CULL_MAX: [f32; 3] = [500.0, 700.0, 500.0];

const SE_AUC_ITEM_SHOW: u32 = 0x08DC;
const SE_AUC_END: u32 = 0x08DE;
const SE_AUC_BID_OK: u32 = 0x08E3;
const SE_AUC_BID_GAUGE: u32 = 0x08E4;
const SE_AUC_BID_NG: u32 = 0x08E5;
const SE_AUC_QUIT: u32 = 0x08E6;
const SE_AUC_LAST_SPURT: u32 = 0x08E7;
const SE_AUC_FEW_TIME: u32 = 0x08E8;
const SE_AUC_5_SEC: u32 = 0x08E9;
const SE_AUC_BID_GAUGE_MAX: u32 = 0x08EB;
const SE_CM_AUC_PARALYZED: u32 = 0x5120;
const SE_CM_AUC_CALL_BID: u32 = 0x591F;

const M834_HOLD: u8 = 1;
const M834_TALKED: u8 = 2;
const M834_CAM_SET: u8 = 4;
const M834_CAM_LOCK: u8 = 8;
const M834_GET_ITEM: u8 = 0x10;
const M834_REST_MSG: u8 = 0x20;
const M834_KEEP: u8 = 0x29;
const M834_CAM_PAIR: u8 = 6;

const M835_STOP: u8 = 1;
const M835_START: u8 = 2;
const M835_SET: u8 = 4;
const M835_ON: u8 = 9;
const M835_CLEAR: u8 = 0xE;

const EVT_CND_CANTALK: u16 = 0x0001;

const CMD_NONE: u8 = 0x0000;
const CMD_TALK: u8 = 0x0001;
const CMD_DEMO: u8 = 0x0002;

const ITEM_JOY_PENDANT: u8 = 0x1F;
const ITEM_COLLECT_MAP_27: u8 = 0xE4;
const ITEM_COLLECT_MAP_18: u8 = 0xED;
const ITEM_HEART_PIECE: u8 = 0x07;
const ITEM_POSTMAN_STATUE: u8 = 0x96;
const ITEM_SHOP_GURU_STATUE: u8 = 0x97;
const ITEM_MAGIC_ARMOR: u8 = 0x2A;
const ITEM_COLLECT_MAP_06: u8 = 0xF9;

const BIT_TALKED_NPC: u16 = 0x4008;
const BIT_TALKED_NPC_REG: u16 = 0x790F;
const REG_ITEM_INDEX: u16 = 0xCD03;

const DEMO_N_WAIT: u8 = 0;
const DEMO_UNK_029: u8 = 29;
const DEMO_PRESENT: u8 = 0;
const DEMO_UNK_072: u8 = 72;
const DEMO_UNK_074: u8 = 74;
const DEMO_L_AROUND2: u8 = 0;
const DEMO_PARAM0: u8 = 3;

const A_STATUS_HIDDEN: u8 = 0x3E;
const A_STATUS_BLANK: u8 = 0x00;
const A_STATUS_BID: u8 = 0x25;
const A_STATUS_CANCEL: u8 = 0x27;
const DO_STATUS_HIDDEN: u8 = 0x3E;

const EV_START: &str = "AUCTION_START";
const EV_GET_ITEM: &str = "AUCTION_GET_ITEM";
const EV_NO_ITEM: &str = "AUCTION_NO_ITEM";
const EV_START2: &str = "AUCTION_START2";
const EV_END2: &str = "AUCTION_END2";
const STAFF_AUCTION: &str = "Auction";

const CUT_MSG_NO: &str = "MsgNo";
const CUT_TIMER: &str = "Timer";

const NEXT_STAGE: &str = "sea";
const NEXT_STAGE_ROOM: u8 = 11;
const NEXT_STAGE_START: u8 = 3;


const ITEM_ROOM: i8 = -1;
const ITEM_SET_ID: u16 = 0xFFFF;

const TIMER_MODE_BID: i32 = 2;
const TIMER_MODE_AUCTION: i32 = 4;
const TIMER_SHORT_MS: i32 = 30000;
const TIMER_LONG_MS: i32 = 60000;

const GAUGE_MAX: f32 = 100.0;
const GAUGE_IDLE: f32 = 0.02;
const GAUGE_RESET: f32 = 50.0;
const GAUGE_HOLD: f32 = 1.0;
const GAUGE_HOLD_LOW: f32 = -1.0;
const BID_TICK: f32 = 60.0;
const NEXT_BET_SPAN: i32 = 60;
const NEXT_BET_CAP: f32 = 50.0;
const NEXT_BET_SCALE: f32 = 1.0;
const NEXT_BET_BONUS: u8 = 2;
const REST_MSG_GATE: i32 = 60000;
const REST_MSG_TABLE: usize = 4;
const BID_LIMIT_LOW: i16 = 100;
const BID_LIMIT_MID: i16 = 150;
const BID_LIMIT_HIGH: i16 = 200;
const BID_MULT_S: f32 = 2.0;
const BID_MULT_M: f32 = 1.5;
const BID_RISE_S: i32 = 30000;
const BID_RISE_M: i32 = 60000;
const BID_ALL: i16 = 999;
const AFTER_BET_COUNT: f32 = 60.0;
const ITEM_LIFT: f32 = 30.0;
const BIKON_SCALE: f32 = 0.4;
const BIKON_TARGET: f32 = 700.0;
const BIKON_TOL: f32 = 0.1;
const BIKON_WAIT: i16 = 30;
const PARALYZE_HEIGHT: f32 = 50.0;
const PARALYZE_DEPTH: f32 = -210.0;
const PARALYZE_LIFT: f32 = 100.0;
const PARALYZE_SWAY: f32 = 256.0;
const PARALYZE_STEP: i16 = 200;
const CAM_YAW_JITTER: f32 = 8000.0;
const CAM_LOOK_HEIGHT: f32 = 0.0;
const EYE_LIFT: f32 = 0.0;
const BLEND_STEP: f32 = 0.05;
const BLEND_MAX: f32 = 1.0;
const MSG_RANGE_TALK_LO: u32 = 0x1D3A;
const MSG_RANGE_TALK_HI: u32 = 0x1D4B;
const MSG_RANGE_SHOP_LO: u32 = 0x339D;
const MSG_RANGE_SHOP_HI: u32 = 0x33A8;
const RAND_THREE: i32 = 3;
const RAND_SIX: i32 = 6;
const RAND_EIGHT: i32 = 8;
const NPC_RND_BIAS: i32 = 1;
const ITEM_IDX_MASK: u8 = 0x01;

const MSG_OPEN: u32 = 0x1CF2;
const MSG_OPEN_NEXT: u32 = 0x1CF3;
const MSG_ITEM_SHOW: u32 = 0x1CF4;
const MSG_SHOW_1: u32 = 0x1CF5;
const MSG_SHOW_2: u32 = 0x1CF6;
const MSG_SLOT: u32 = 0x1CF7;
const MSG_SLOT_NEXT: u32 = 0x1CF8;
const MSG_NEXT_BID: u32 = 0x1CF9;
const MSG_BID_OPEN: u32 = 0x1CFA;
const MSG_BID_FAIL_LOW: u32 = 0x1CFB;
const MSG_BID_LOW: u32 = 0x1CFD;
const MSG_BID_MID: u32 = 0x1CFE;
const MSG_BID_HIGH: u32 = 0x1CFF;
const MSG_BID_TOP: u32 = 0x1D00;
const MSG_FINISH: u32 = 0x1D05;
const MSG_BID_FAIL_HIGH: u32 = 0x1D1E;
const MSG_QUIT_SELECT: u32 = 0x1CFC;
const MSG_SHOW_TALK2: u32 = 0x1D1B;
const MSG_CANCEL: u32 = 0x1D1A;
const MSG_QUIT_LEAVE: u32 = 0x1D1F;
const MSG_ITEM_COST: u32 = 0x1D1C;
const MSG_PAY: u32 = 0x1D07;
const MSG_PAY_NEXT: u32 = 0x1D08;
const MSG_PAY_CANCEL: u32 = 0x1D1D;
const MSG_PAY_DONE: u32 = 0x1D09;
const MSG_QUIT_END: u32 = 0x1D24;
const MSG_REST4: u32 = 0x1D20;
const MSG_REST3: u32 = 0x1D21;
const MSG_REST2: u32 = 0x1D22;
const MSG_REST1: u32 = 0x1D23;

const CUT_TABLE: [&str; 7] = [
    "WAIT",
    "TURN_TO_ACTOR",
    "MOVE_TO_ACTOR",
    "TURN_TO_POS",
    "MOVE_TO_POS",
    "TALK_MSG",
    "CONTINUE_TALK",
];

const ACT_TABLE: [&str; 11] = [
    "MES_SET",
    "MES_END",
    "START",
    "MAIN",
    "GET_ITEM",
    "CAMERA_OFF",
    "GET_ITEM_NPC",
    "GET_ITEM_MES",
    "CAMERA_OFF_NPC",
    "END",
    "CAMERA_TEST",
];

const PROC_KAI: u8 = 0;
const PROC_URI: u8 = 1;
const PROC_MSG_SET: u8 = 2;
const PROC_MSG_END: u8 = 3;
const PROC_BIKON_C: u8 = 4;
const PROC_BIKON_W: u8 = 5;

const MOVE_WAIT: u8 = 0;
const MOVE_NORMAL: u8 = 1;
const MOVE_START: u8 = 2;

const ORDER_TALK: u8 = 1;
const ORDER_TALK2: u8 = 2;
const ORDER_START: i8 = 3;
const ORDER_GET_ITEM: i8 = 4;
const ORDER_NO_ITEM: i8 = 5;
const ORDER_START2: i8 = 6;
const ORDER_END2: i8 = 7;

const GAUGE_TABLE: [f32; NPC_COUNT] = [120.0, 180.0, 180.0, 180.0, 180.0, 180.0, 210.0, 180.0];

const CAMERA_PRESET: [[[f32; 3]; 2]; CAMERA_PRESETS] = [
    [[-265.0, 48.0, -631.0], [332.0, 232.0, 286.0]],
    [[-50.0, 202.0, 137.0], [-87.0, 239.0, 330.0]],
    [[-216.0, 205.0, 5.0], [-379.0, 238.0, 115.0]],
];

const REST_MSG_TIME1: [i16; REST_MSG_TABLE] = [30000, 10000, 5000, -1000];
const REST_MSG_TIME2: [i16; REST_MSG_TABLE] = [15000, 10000, 5000, -1000];
const REST_MSG_NO: [u32; REST_MSG_MAX] = [0x1D01, 0x1D02, 0x1D03];
const REST_SE_NO: [u32; REST_MSG_MAX] = [SE_AUC_LAST_SPURT, SE_AUC_FEW_TIME, SE_AUC_5_SEC];

const AFTER_BET_RATE: [f32; 4] = [2.0, 1.5, 1.2, 1.1];
const AFTER_BET_CHK: [i16; 4] = [0x0014, 0x0014, 0x0028, 0x0028];
const AFTER_BET_MSG_NO: [u32; 4] = [0x1D23, 0x1D22, 0x1D21, 0x1D20];
const AFTER_BET_WAIT: [[i16; 2]; 4] = [[8, 10], [7, 10], [6, 9], [5, 8]];

const LINK_FACE: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ItemData {
    item: u8,
    name_msg: u16,
    bid: i16,
    bit: u16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct NpcDat {
    low: f32,
    high: f32,
    inc_min: i16,
    inc_max: i16,
    limit: i16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NpcMsgDat {
    name: u16,
    mid: u16,
    low: u16,
    high: u16,
    tail: u16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct NpcCameraDat {
    dist: f32,
    height: f32,
    pitch: i16,
    yaw: i16,
}

const ITEM_DAT: [ItemData; ITEM_COUNT] = [
    ItemData {
        item: ITEM_JOY_PENDANT,
        name_msg: 0x1D10,
        bid: 40,
        bit: 0x0F01,
    },
    ItemData {
        item: ITEM_COLLECT_MAP_27,
        name_msg: 0x1D11,
        bid: 5,
        bit: 0x1080,
    },
    ItemData {
        item: ITEM_COLLECT_MAP_18,
        name_msg: 0x1D12,
        bid: 60,
        bit: 0x1040,
    },
    ItemData {
        item: ITEM_HEART_PIECE,
        name_msg: 0x1D13,
        bid: 80,
        bit: 0x1020,
    },
];

const ITEM_DAT2: [ItemData; ITEM2_COUNT] = [
    ItemData {
        item: ITEM_POSTMAN_STATUE,
        name_msg: 0x1D14,
        bid: 30,
        bit: 0x1008,
    },
    ItemData {
        item: ITEM_SHOP_GURU_STATUE,
        name_msg: 0x1D15,
        bid: 40,
        bit: 0x1004,
    },
];

const ITEM_DAT22: [u8; ITEM2_COUNT] = [ITEM_MAGIC_ARMOR, ITEM_COLLECT_MAP_06];

const NPC_DAT: [NpcDat; 7] = [
    NpcDat {
        low: 2.0,
        high: 2.1,
        inc_min: 0,
        inc_max: 0,
        limit: 1000,
    },
    NpcDat {
        low: 0.3,
        high: 0.5,
        inc_min: 4,
        inc_max: 6,
        limit: 450,
    },
    NpcDat {
        low: 0.3,
        high: 0.6,
        inc_min: 3,
        inc_max: 7,
        limit: 500,
    },
    NpcDat {
        low: 0.4,
        high: 0.7,
        inc_min: 2,
        inc_max: 4,
        limit: 250,
    },
    NpcDat {
        low: 0.5,
        high: 1.0,
        inc_min: 2,
        inc_max: 3,
        limit: 990,
    },
    NpcDat {
        low: 0.4,
        high: 0.5,
        inc_min: 3,
        inc_max: 4,
        limit: 990,
    },
    NpcDat {
        low: 0.7,
        high: 1.2,
        inc_min: 2,
        inc_max: 2,
        limit: 150,
    },
];

const NPC_MSG_DAT: [NpcMsgDat; 12] = [
    NpcMsgDat {
        name: 0x33AC,
        mid: 0x1D3A,
        low: 0x1D3B,
        high: 0x1D3C,
        tail: 0x33B6,
    },
    NpcMsgDat {
        name: 0x33AD,
        mid: 0x1D3D,
        low: 0x1D3E,
        high: 0x1D3F,
        tail: 0x33B7,
    },
    NpcMsgDat {
        name: 0x33AE,
        mid: 0x1D40,
        low: 0x1D41,
        high: 0x1D42,
        tail: 0x33B8,
    },
    NpcMsgDat {
        name: 0x33AF,
        mid: 0x1D43,
        low: 0x1D44,
        high: 0x1D45,
        tail: 0x33B9,
    },
    NpcMsgDat {
        name: 0x33B0,
        mid: 0x1D46,
        low: 0x1D47,
        high: 0x1D48,
        tail: 0x33BA,
    },
    NpcMsgDat {
        name: 0x33B1,
        mid: 0x1D49,
        low: 0x1D4A,
        high: 0x1D4B,
        tail: 0x33BB,
    },
    NpcMsgDat {
        name: 0x33B2,
        mid: 0x339D,
        low: 0x339E,
        high: 0x339F,
        tail: 0x33BC,
    },
    NpcMsgDat {
        name: 0x33B3,
        mid: 0x33A0,
        low: 0x33A1,
        high: 0x33A2,
        tail: 0x33BD,
    },
    NpcMsgDat {
        name: 0x33B4,
        mid: 0x33A3,
        low: 0x33A4,
        high: 0x33A5,
        tail: 0x33BE,
    },
    NpcMsgDat {
        name: 0x33B5,
        mid: 0x33A6,
        low: 0x33A7,
        high: 0x33A8,
        tail: 0x33BF,
    },
    NpcMsgDat {
        name: 0x1CE9,
        mid: 0,
        low: 0,
        high: 0,
        tail: 0,
    },
    NpcMsgDat {
        name: 0x1CE9,
        mid: 0,
        low: 0,
        high: 0,
        tail: 0,
    },
];

const NPC_CAMERA_DAT: [NpcCameraDat; 7] = [
    NpcCameraDat {
        dist: 150.0,
        height: 70.0,
        pitch: -500,
        yaw: 0,
    },
    NpcCameraDat {
        dist: 200.0,
        height: 100.0,
        pitch: 0,
        yaw: 0,
    },
    NpcCameraDat {
        dist: 200.0,
        height: 100.0,
        pitch: 0,
        yaw: 0,
    },
    NpcCameraDat {
        dist: 200.0,
        height: 100.0,
        pitch: 0,
        yaw: 0,
    },
    NpcCameraDat {
        dist: 200.0,
        height: 100.0,
        pitch: 0,
        yaw: 0,
    },
    NpcCameraDat {
        dist: 200.0,
        height: 100.0,
        pitch: 0,
        yaw: 0,
    },
    NpcCameraDat {
        dist: 200.0,
        height: 150.0,
        pitch: 0,
        yaw: 0,
    },
];

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Auction {
    pub entered: bool,
    pub npc_pid: [u32; NPC_COUNT],
    pub order_tbl: [u8; NPC_COUNT],
    pub auc_mdl_no: [u8; NPC_COUNT],
    pub item_entity: Option<Entity>,
    pub item_pid: u32,
    pub timer_id: u32,
    pub gauge_id: u32,
    pub timer_ready: bool,
    pub timer_show: u8,
    pub timer_limit: i32,
    pub timer_now: i32,
    pub timer_mode: i32,
    pub gauge: [f32; GAUGE_COUNT],
    pub wave: f32,
    pub wave_dir: f32,
    pub blend: f32,
    pub msg_cache: u32,
    pub msg_next: u32,
    pub ev_start: i16,
    pub ev_get_item: i16,
    pub ev_no_item: i16,
    pub ev_start2: i16,
    pub ev_end2: i16,
    pub curr_bid: i16,
    pub bid_value: i16,
    pub bid_timer: i16,
    pub hold: i16,
    pub cam_sway: i16,
    pub move_state: u8,
    pub bid_mode: u8,
    pub start_request: u8,
    pub which: u8,
    pub item_index: u8,
    pub speaker: u8,
    pub prev_speaker: u8,
    pub seen_mask: u8,
    pub pending_speaker: u8,
    pub bidder: u8,
    pub pending_prev: u8,
    pub rest_index: i32,
    pub after_bet: i32,
    pub finish_flag: u8,
    pub uri_seen: u8,
    pub link_anm: u8,
    pub link_param0: u8,
    pub cam_slot: u8,
    pub msg_hold: u8,
    pub next_bet_hold: u8,
    pub face_live: u8,
    pub face: u8,
    pub m834: u8,
    pub m835: u8,
    pub flags: u8,
    pub order: i8,
    pub act: i8,
    pub command: u8,
    pub item_rupee_count: i32,
    pub auction_rupee: i16,
    pub auction_gauge: i16,
    pub msg_set_no: i16,
    pub mini_game: bool,
    pub slot_show: bool,
    pub gauge_show: bool,
    pub talk_live: bool,
    pub cam_from: Vec3,
    pub cam_to: Vec3,
    pub effect_pos: Vec3,
    pub eye: Vec3,
    pub do_status: u8,
    pub a_status: u8,
    pub msg_pid: u32,
    pub msg_no: u32,
    pub msg_open: bool,
    pub seed: [i32; 3],
}

impl Auction {
    pub fn spawn(_parameters: u32) -> Self {
        let mut order_tbl = [0_u8; NPC_COUNT];
        for (index, slot) in order_tbl.iter_mut().enumerate() {
            *slot = index as u8;
        }
        let mut seed = [RND_SEED; 3];
        for _ in 0..SHUFFLE_ROUNDS {
            let rnd1 = get_rand(&mut seed, SHUFFLE_RANGE) as usize + SHUFFLE_OFFSET;
            let rnd2 = get_rand(&mut seed, SHUFFLE_RANGE) as usize + SHUFFLE_OFFSET;
            let tmp = order_tbl[rnd1];
            order_tbl[rnd1] = order_tbl[rnd2];
            order_tbl[rnd2] = tmp;
        }
        Self {
            entered: false,
            npc_pid: [PROC_ERROR; NPC_COUNT],
            order_tbl,
            auc_mdl_no: [NO_BYTE; NPC_COUNT],
            item_entity: None,
            item_pid: PROC_ERROR,
            timer_id: PROC_ERROR,
            gauge_id: PROC_ERROR,
            timer_ready: false,
            timer_show: 0,
            timer_limit: 0,
            timer_now: 0,
            timer_mode: 0,
            gauge: [0.0; GAUGE_COUNT],
            wave: 0.0,
            wave_dir: 0.0,
            blend: 0.0,
            msg_cache: 0,
            msg_next: 0,
            ev_start: NO_EVENT,
            ev_get_item: NO_EVENT,
            ev_no_item: NO_EVENT,
            ev_start2: NO_EVENT,
            ev_end2: NO_EVENT,
            curr_bid: 0,
            bid_value: 0,
            bid_timer: 0,
            hold: 0,
            cam_sway: 0,
            move_state: MOVE_WAIT,
            bid_mode: PROC_KAI,
            start_request: 0,
            which: 0,
            item_index: 0,
            speaker: 0,
            prev_speaker: 0,
            seen_mask: NO_BYTE,
            pending_speaker: 0,
            bidder: 0,
            pending_prev: 0,
            rest_index: 0,
            after_bet: 0,
            finish_flag: 0,
            uri_seen: 0,
            link_anm: DEMO_N_WAIT,
            link_param0: 0,
            cam_slot: 0,
            msg_hold: 0,
            next_bet_hold: 0,
            face_live: 0,
            face: 0,
            m834: 0,
            m835: 0,
            flags: 0,
            order: 0,
            act: 0,
            command: CMD_NONE,
            item_rupee_count: 0,
            auction_rupee: 0,
            auction_gauge: 0,
            msg_set_no: 0,
            mini_game: false,
            slot_show: false,
            gauge_show: false,
            talk_live: false,
            cam_from: Vec3::ZERO,
            cam_to: Vec3::ZERO,
            effect_pos: Vec3::ZERO,
            eye: Vec3::ZERO,
            do_status: 0,
            a_status: 0,
            msg_pid: FPC_M_ERROR_PROCESS_ID,
            msg_no: NO_MSG,
            msg_open: false,
            seed,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    pub fn request_start(&mut self, which: u8) {
        self.start_request = which;
    }

    fn create_init(&mut self, ctx: &mut Ctx<'_, '_, '_>) {
        self.ev_start = event_slot(ctx.list, EV_START);
        self.ev_get_item = event_slot(ctx.list, EV_GET_ITEM);
        self.ev_no_item = event_slot(ctx.list, EV_NO_ITEM);
        self.ev_start2 = event_slot(ctx.list, EV_START2);
        self.ev_end2 = event_slot(ctx.list, EV_END2);
        self.speaker = 0;
        self.seen_mask = NO_BYTE;
        ctx.layout.npc_name_msg_no = u32::from(self.msg_row().name);
        self.item_index = 0;
        let data = item_dat(self.item_index);
        self.curr_bid = data.bid;
        ctx.layout.item_name_msg_no = u32::from(data.name_msg);
        ctx.counters.message_count = i32::from(self.curr_bid);
        self.eye = ctx.actor_pos;
    }

    fn rnd_f(&mut self, range: f32) -> f32 {
        get_rand_f(&mut self.seed, range)
    }

    fn get_rand(&mut self, max: i32) -> i32 {
        get_rand(&mut self.seed, max)
    }

    fn mdl_no(&self) -> u8 {
        self.auc_mdl_no
            .get(usize::from(self.speaker))
            .copied()
            .unwrap_or(NO_BYTE)
    }

    fn msg_row(&self) -> NpcMsgDat {
        self.msg_of(self.speaker)
    }

    fn msg_of(&self, speaker: u8) -> NpcMsgDat {
        let mdl = self
            .auc_mdl_no
            .get(usize::from(speaker))
            .copied()
            .unwrap_or(NO_BYTE);
        *NPC_MSG_DAT.get(usize::from(mdl)).unwrap_or(&NPC_MSG_DAT[0])
    }

    fn execute(&mut self, ctx: &mut Ctx<'_, '_, '_>) {
        self.command = self.event_command(ctx);
        self.check_order(ctx);
        if !ctx.event_run || self.command == CMD_TALK {
            self.move_proc();
        } else {
            self.event_move(ctx);
        }
        self.event_order(ctx);
        if self.timer_mode == TIMER_MODE_AUCTION {
            self.timer_now = self.timer_limit - ctx.timer.rest_time();
        } else {
            ctx.timer.delete_request();
        }
    }

    fn move_proc(&mut self) {
        match self.move_state {
            MOVE_WAIT => self.execute_wait(),
            MOVE_NORMAL => self.execute_normal(),
            MOVE_START => self.execute_start(),
            _ => self.execute_normal(),
        }
    }

    fn execute_wait(&mut self) {
        for pid in self.npc_pid {
            if pid == PROC_ERROR {
                return;
            }
        }
        self.move_state = MOVE_NORMAL;
    }

    fn execute_normal(&mut self) {}

    fn execute_start(&mut self) {}

    fn event_command(&self, ctx: &Ctx<'_, '_, '_>) -> u8 {
        let idx = [self.ev_start, self.ev_start2];
        if idx.iter().any(|slot| self.started(ctx, *slot)) {
            return CMD_DEMO;
        }
        let idx = [self.ev_get_item, self.ev_no_item, self.ev_end2];
        if idx.iter().any(|slot| self.started(ctx, *slot)) {
            return CMD_DEMO;
        }
        if self.order == 1 || self.order == 2 {
            return CMD_TALK;
        }
        CMD_NONE
    }

    fn check_order(&mut self, ctx: &Ctx<'_, '_, '_>) {
        if self.command != CMD_DEMO {
            return;
        }
        if self.started(ctx, self.ev_start) && self.order == ORDER_START {
            self.order = 0;
        } else if self.started(ctx, self.ev_get_item) && self.order == ORDER_GET_ITEM {
            self.order = 0;
        } else if self.started(ctx, self.ev_no_item) && self.order == ORDER_NO_ITEM {
            self.order = 0;
        }
        if self.started(ctx, self.ev_start2) && self.order == ORDER_START2 {
            self.order = 0;
        } else if self.started(ctx, self.ev_end2) && self.order == ORDER_END2 {
            self.order = 0;
        }
    }

    fn started(&self, ctx: &Ctx<'_, '_, '_>, index: i16) -> bool {
        index >= 0 && ctx.active.start_check(usize::from(index as u16))
    }

    fn ended(&self, ctx: &Ctx<'_, '_, '_>, index: i16) -> bool {
        index >= 0 && ctx.active.end_check(usize::from(index as u16))
    }

    fn event_order(&mut self, ctx: &mut Ctx<'_, '_, '_>) {
        if self.start_request == ORDER_TALK {
            self.move_state = MOVE_START;
            self.order = ORDER_START;
            self.which = self.start_request;
            self.start_request = 0;
        } else if self.start_request == ORDER_TALK2 {
            self.move_state = MOVE_START;
            self.order = ORDER_START2;
            self.which = self.start_request;
            self.start_request = 0;
        }
        match self.order {
            1 => {
                ctx.order_flags.set(EVT_CND_CANTALK);
            }
            2 => {
                ctx.order_flags.set(EVT_CND_CANTALK);
            }
            ORDER_START => {
                if ctx.stories.test(BIT_TALKED_NPC) {
                    self.item_index = ctx.stories.reg(REG_ITEM_INDEX);
                    self.curr_bid = self.curr_bid.wrapping_add(10);
                } else {
                    self.item_index = self.get_item_no(ctx);
                }
                let data = item_dat(self.item_index);
                ctx.layout.item_name_msg_no = u32::from(data.name_msg);
                self.curr_bid = data.bid;
                ctx.counters.message_count = i32::from(self.curr_bid);
                self.order_event(ctx, self.ev_start);
                self.mini_game = true;
            }
            ORDER_GET_ITEM => self.order_event(ctx, self.ev_get_item),
            ORDER_NO_ITEM => self.order_event(ctx, self.ev_no_item),
            ORDER_START2 => {
                self.item_index &= ITEM_IDX_MASK;
                let data = item_dat2(self.item_index);
                ctx.layout.item_name_msg_no = u32::from(data.name_msg);
                self.curr_bid = data.bid;
                ctx.counters.message_count = i32::from(self.curr_bid);
                self.order_event(ctx, self.ev_start2);
                self.mini_game = true;
            }
            ORDER_END2 => self.order_event(ctx, self.ev_end2),
            _ => {}
        }
    }

    fn order_event(&mut self, ctx: &mut Ctx<'_, '_, '_>, index: i16) {
        if index >= 0 {
            let entity = ctx.entity;
            ctx.active.order(ctx.list, usize::from(index as u16), entity);
        }
    }

    fn event_move(&mut self, ctx: &mut Ctx<'_, '_, '_>) {
        if self.ended(ctx, self.ev_start) {
            self.event_reset(ctx, self.ev_start);
            self.order = if self.speaker != 0 {
                ORDER_NO_ITEM
            } else {
                ORDER_GET_ITEM
            };
            return;
        }
        if self.ended(ctx, self.ev_get_item)
            || self.ended(ctx, self.ev_no_item)
            || self.ended(ctx, self.ev_end2)
        {
            for index in [self.ev_get_item, self.ev_no_item, self.ev_end2] {
                self.event_reset(ctx, index);
            }
            self.mini_game = false;
            self.timer_id = PROC_ERROR;
            self.timer_ready = false;
            self.gauge_id = PROC_ERROR;
            return;
        }
        if self.ended(ctx, self.ev_start2) {
            self.event_reset(ctx, self.ev_start2);
            self.order = ORDER_END2;
            return;
        }
        self.private_cut(ctx);
    }

    fn event_reset(&mut self, ctx: &mut Ctx<'_, '_, '_>, index: i16) {
        if index >= 0 {
            ctx.active.end(usize::from(index as u16));
        }
        *ctx.ev_flags = EventFlags::default();
    }

    fn private_cut(&mut self, ctx: &mut Ctx<'_, '_, '_>) {
        let Some((cut, advance)) = ctx.active.staff_cut(ctx.list, STAFF_AUCTION) else {
            return;
        };
        let name = ctx.list.cut_name(cut).unwrap_or("");
        if CUT_TABLE.contains(&name) {
            return;
        }
        self.act = match ACT_TABLE.iter().position(|act| *act == name) {
            Some(index) => index as i8,
            None => -1,
        };
        if self.act < 0 {
            ctx.active.end_staff_cut(ctx.list, ctx.ev_flags, STAFF_AUCTION);
            return;
        }
        if advance {
            match name {
                "MES_SET" => self.event_talk_init(ctx, cut),
                "START" => self.event_start_init(ctx),
                "MAIN" => self.event_main_init(),
                "GET_ITEM" => self.event_get_item_init(ctx),
                "CAMERA_OFF" => self.event_camera_off_init(),
                "GET_ITEM_NPC" => self.event_get_item_npc_init(ctx, cut),
                "GET_ITEM_MES" => self.event_get_item_mes_init(),
                "END" => self.event_end_init(),
                "CAMERA_TEST" => self.event_camera_test_init(),
                _ => {}
            }
        }
        let evt_res = match name {
            "MES_SET" | "GET_ITEM_MES" => self.event_mes_set(ctx),
            "MES_END" | "CAMERA_OFF_NPC" => self.event_mes_end(ctx),
            "START" => self.event_start(),
            "MAIN" => self.event_main(ctx),
            "GET_ITEM" => self.event_get_item(),
            "END" => self.event_end(ctx),
            "CAMERA_TEST" => self.event_camera_test(),
            _ => true,
        };
        if evt_res {
            ctx.active.end_staff_cut(ctx.list, ctx.ev_flags, STAFF_AUCTION);
        }
        if self.m835 & M835_STOP != 0 {
            self.m835 &= !M835_STOP;
            self.m835 |= M835_SET;
        }
        if self.m835 & M835_START != 0 {
            self.m835 &= !M835_CLEAR;
        }
    }

    fn event_talk_init(&mut self, ctx: &mut Ctx<'_, '_, '_>, cut: usize) {
        match ctx.list.cut_number(cut, CUT_MSG_NO) {
            Some(value) => {
                let value = value as u32;
                if value == MSG_ITEM_SHOW {
                    ctx.events.list.push(SimEvent::Se(SE_AUC_ITEM_SHOW));
                }
                self.set_message(value);
            }
            None => self.set_message(NO_MSG),
        }
    }

    fn set_message(&mut self, msg: u32) {
        self.msg_pid = FPC_M_ERROR_PROCESS_ID;
        self.msg_no = msg;
    }

    fn set_message2(&mut self, msg: u32) {
        self.set_message(msg);
        self.bid_mode = PROC_MSG_SET;
        self.gauge_show = false;
    }

    fn event_mes_set(&mut self, ctx: &mut Ctx<'_, '_, '_>) -> bool {
        self.talk(ctx);
        self.msg_pid != FPC_M_ERROR_PROCESS_ID
    }

    fn event_mes_end(&mut self, ctx: &mut Ctx<'_, '_, '_>) -> bool {
        self.talk(ctx) == STATUS_BOX_CLOSED
    }

    fn talk(&mut self, ctx: &mut Ctx<'_, '_, '_>) -> u8 {
        if self.msg_pid == FPC_M_ERROR_PROCESS_ID {
            let eye = self.eye;
            if let Some(archive) = ctx.archive {
                self.msg_pid = fop_msgm_message_set(
                    ctx.procs,
                    ctx.info,
                    archive,
                    ctx.clear_count,
                    self.msg_no,
                    None,
                    eye,
                );
            }
            self.msg_open = false;
            return TALK_IDLE;
        }
        let status = ctx.box_status();
        if status == TALK_IDLE {
            self.msg_open = true;
            return TALK_IDLE;
        }
        if status == STATUS_MSG_DISPLAYED {
            let next = self.next_msg_status(ctx);
            ctx.set_box_status(next);
            if next == STATUS_MSG_CONTINUES {
                if let Some(archive) = ctx.archive {
                    self.msg_pid = fop_msgm_message_set(
                        ctx.procs,
                        ctx.info,
                        archive,
                        ctx.clear_count,
                        self.msg_no,
                        None,
                        Vec3::ZERO,
                    );
                }
            }
        } else if status == STATUS_BOX_CLOSED {
            ctx.set_box_status(STATUS_MSG_DESTROYED);
            self.msg_pid = FPC_M_ERROR_PROCESS_ID;
        }
        status
    }

    fn event_start_init(&mut self, ctx: &mut Ctx<'_, '_, '_>) {
        self.timer_id = PROC_ERROR;
        self.timer_limit = if self.which == ORDER_TALK2 {
            TIMER_SHORT_MS
        } else {
            TIMER_LONG_MS
        };
        let limit = self.timer_limit as u16;
        if ctx
            .timer
            .create_timer(isle_timer::MODE_AUCTION, limit, 0, 0, 0.0, 0.0, 0.0, 0.0)
        {
            ctx.timer.timer_start(isle_timer::MODE_AUCTION, 0);
        }
        self.gauge_id = PROC_ERROR;
        self.timer_ready = false;
        self.msg_hold = 1;
        ctx.counters.mini_game_rupee = 0;
        self.auction_rupee = 0;
        self.slot_show = false;
        self.gauge_show = false;
        self.talk_live = false;
        let data = item_dat(self.item_index);
        self.item_entity = Some(
            ctx.commands
                .spawn(Actor {
                    name: super::sitem::SITEM_ACTORS[0].to_owned(),
                    parameters: u32::from(data.item),
                    pos: ctx.actor_pos,
                    angle: [0; 3],
                    set_id: ITEM_SET_ID,
                    room: ITEM_ROOM,
                    ..Default::default()
                })
                .id(),
        );
        self.flags |= 4;
        self.talk_live = false;
    }

    fn event_start(&mut self) -> bool {
        if !self.timer_ready {
            self.timer_ready = true;
            self.timer_show = 0;
        }
        self.do_status = DO_STATUS_HIDDEN;
        self.a_status = A_STATUS_HIDDEN;
        self.timer_ready
    }

    fn event_main_init(&mut self) {
        self.gauge = [0.0; GAUGE_COUNT];
        self.bid_mode = if self.which == ORDER_TALK2 {
            PROC_URI
        } else {
            PROC_KAI
        };
        self.wave = 0.0;
        self.wave_dir = 1.0;
        self.rest_index = 0;
        for index in 0..REST_MSG_MAX {
            let time = if self.which == ORDER_TALK2 {
                REST_MSG_TIME2[index]
            } else {
                REST_MSG_TIME1[index]
            };
            if REST_MSG_GATE > i32::from(time) {
                break;
            }
            self.rest_index += 1;
        }
        self.finish_flag = 0;
        self.m835 = 0;
        self.m834 = 0;
        self.uri_seen = 0;
        self.timer_mode = TIMER_MODE_AUCTION;
        self.timer_now = 0;
        if self.which == ORDER_TALK {
            self.gauge_show = true;
        }
        self.link_anm = DEMO_N_WAIT;
        self.cam_slot = 0;
        self.cam_from = preset(self.cam_slot).0;
        self.cam_to = preset(self.cam_slot).1;
        self.link_param0 = DEMO_PARAM0;
        self.talk_live = false;
    }

    fn event_main(&mut self, ctx: &mut Ctx<'_, '_, '_>) -> bool {
        if ctx.link_rate == 0.0
            && self.link_anm != DEMO_N_WAIT
            && self.link_anm != DEMO_UNK_029
            && self.link_anm != DEMO_PRESENT
        {
            self.set_link_anm(DEMO_N_WAIT);
        }
        self.flags &= 4;
        if self.timer_mode == TIMER_MODE_BID {
            self.timer_mode = TIMER_MODE_AUCTION;
        }
        match self.bid_mode {
            PROC_KAI => self.event_main_kai(ctx),
            PROC_URI => self.event_main_uri(ctx),
            PROC_MSG_SET => self.event_main_msg_set(ctx),
            PROC_MSG_END => self.event_main_msg_end(ctx),
            PROC_BIKON_C => self.event_main_bikon_c(ctx),
            PROC_BIKON_W => self.event_main_bikon_w(ctx),
            _ => {}
        }
        if self.face_live != 0 {
            ctx.set_link_face(self.face);
        }
        if self.which == ORDER_TALK {
            self.auction_gauge = self.gauge[0] as i16;
        }
        if self.bid_mode <= PROC_URI && self.finish_flag != 0 {
            ctx.layout.npc_name_msg_no = u32::from(self.msg_row().name);
            if self.timer_mode == TIMER_MODE_BID {
                self.timer_mode = TIMER_MODE_AUCTION;
            }
            self.slot_show = false;
            self.gauge_show = false;
            self.finish_flag = 1;
            self.m834 = 0;
            self.link_anm = DEMO_N_WAIT;
            if self.m835 & M835_SET != 0 {
                self.m835 |= M835_START;
                self.m835 &= !M835_STOP;
            }
            ctx.events.list.push(SimEvent::Se(SE_AUC_END));
            return true;
        }
        false
    }

    fn event_main_kai(&mut self, ctx: &mut Ctx<'_, '_, '_>) {
        if self.timer_mode == TIMER_MODE_BID {
            self.timer_mode = TIMER_MODE_AUCTION;
        }
        if self.rest_ms() <= 0 {
            self.finish_flag = 1;
        } else if ctx.trig_b {
            self.set_message2(MSG_QUIT_SELECT);
            ctx.events.list.push(SimEvent::Se(SE_AUC_QUIT));
        } else {
            self.main_bid_pass(ctx);
        }
        if self.m834 & M834_TALKED != 0 {
            self.m834 |= M834_CAM_SET;
            self.cam_to = Vec3::new(0.0, PARALYZE_HEIGHT, PARALYZE_DEPTH);
            self.cam_from = ctx.npc_view(0).map_or(ctx.actor_pos, |view| view.0);
            self.cam_from.x += walk::from_s16(self.cam_sway).sin() * PARALYZE_SWAY;
            self.cam_from.y += PARALYZE_LIFT;
            self.cam_sway = self.cam_sway.wrapping_add(PARALYZE_STEP);
            ctx.events.list.push(SimEvent::Se(SE_CM_AUC_PARALYZED));
        }
        self.do_status = A_STATUS_BID;
        self.a_status = A_STATUS_CANCEL;
    }

    fn main_bid_pass(&mut self, ctx: &mut Ctx<'_, '_, '_>) {
        self.wave += self.wave_dir;
        if self.wave >= GAUGE_HOLD {
            self.wave = GAUGE_HOLD;
            self.wave_dir = GAUGE_HOLD_LOW;
        } else if self.wave <= GAUGE_HOLD_LOW {
            self.wave = GAUGE_HOLD_LOW;
            self.wave_dir = GAUGE_HOLD;
        }
        if self.gauge[0] < GAUGE_MAX {
            self.gauge[0] += GAUGE_IDLE;
        }
        if ctx.trig_a {
            let npc = NPC_DAT[0];
            self.gauge[0] += npc.low + self.rnd_f(npc.high - npc.low);
            ctx.events.list.push(SimEvent::Se(SE_AUC_BID_GAUGE));
        }
        if self.gauge[0] > GAUGE_MAX {
            self.gauge[0] = GAUGE_MAX;
        }
        if self.hold != 0 {
            self.hold -= 1;
            if self.hold == 0 {
                if self.m834 & M834_TALKED != 0 {
                    self.cam_slot = self.get_rand(RAND_THREE) as u8;
                    self.cam_from = preset(self.cam_slot).0;
                    self.cam_to = preset(self.cam_slot).1;
                }
                self.m834 &= !M834_CAM_PAIR;
            }
        } else {
            for index in GAUGE_LO..GAUGE_HI {
                let npc = NPC_DAT[index];
                self.gauge[index] += self.rnd_f(npc.high - npc.low) + npc.low;
            }
        }
        let mut end = GAUGE_COUNT as i32;
        if self.m834 & M834_TALKED != 0 {
            end = 1;
        }
        let mut idx = 0;
        while idx < end {
            if self.gauge_at(idx) >= GAUGE_MAX && self.curr_bid < NPC_DAT[idx as usize].limit {
                break;
            }
            idx += 1;
        }
        if idx >= end {
            if self.rest_ms() < i32::from(rest_time(&REST_MSG_TIME1, self.rest_index)) {
                self.set_message2(rest_msg_no(self.rest_index));
                let se = rest_se_no(self.rest_index);
                ctx.events.list.push(SimEvent::Se(se));
                self.rest_index += 1;
            }
            return;
        }
        self.bidder = idx as u8;
        let prev_item_name = self.curr_bid;
        let next_msg;
        if idx != 0 {
            let mult = self.bid_multiplier();
            let npc = NPC_DAT[idx as usize];
            let inc =
                (self.rnd_f(f32::from(npc.inc_max) - f32::from(npc.inc_min)) + f32::from(npc.inc_min))
                    as i16;
            self.curr_bid = self.curr_bid.wrapping_add((mult * f32::from(inc)) as i16);
            self.bid_value = self.curr_bid;
            next_msg = MSG_NEXT_BID;
            self.after_bet = 4;
            self.gauge_set(idx as usize, 0.0);
            self.next_bet();
        } else {
            self.bid_value = self.curr_bid.wrapping_add(1);
            self.msg_set_no = self.curr_bid.wrapping_add(1);
            next_msg = MSG_BID_OPEN;
            self.set_link_anm(DEMO_UNK_072);
            self.timer_show = 1;
            self.gauge_show = true;
            ctx.shake(5, 1);
            ctx.shake(4, 0x3E);
        }
        self.hold = ((self.rnd_f(0.0) + 1.0) * BID_TICK) as i16;
        self.msg_next = next_msg;
        self.bid_mode = PROC_BIKON_C;
        if self.msg_hold != 0 {
            self.msg_hold = 0;
        } else if self.seen_mask != NO_BYTE
            && idx != 0
            && self.rest_ms() > i32::from(REST_MSG_TIME1[0])
            && self.get_rand(RAND_THREE) == 0
        {
            ctx.counters.message_count = i32::from(prev_item_name);
            ctx.layout.npc_name_msg_no = u32::from(self.msg_row().name);
            self.set_message2(self.bid_comment());
            self.set_camera_npc(ctx, usize::from(self.speaker), 0);
            if self.speaker == 0 && self.link_anm != DEMO_UNK_029 {
                self.set_link_anm(DEMO_L_AROUND2);
            }
            self.m834 |= M834_REST_MSG;
            self.pending_prev = self.pending_speaker;
            self.msg_hold = 1;
        }
        if self.bid_mode == PROC_BIKON_C {
            self.pending_speaker = self.bidder;
            self.pending_prev = self.bidder;
            ctx.counters.message_count = i32::from(self.bid_value);
            ctx.layout.npc_name_msg_no = u32::from(self.msg_of(self.pending_speaker).name);
        }
        self.m835 |= M835_ON;
        self.m835 &= !M835_START;
    }

    fn event_main_uri(&mut self, ctx: &mut Ctx<'_, '_, '_>) {
        if self.timer_mode == TIMER_MODE_BID {
            self.timer_mode = TIMER_MODE_AUCTION;
        }
        if self.rest_ms() <= 0 {
            self.finish_flag = 1;
        } else if self.uri_seen != 0 && ctx.trig_a {
            self.set_message2(MSG_CANCEL);
        } else {
            if self.hold != 0 {
                self.hold -= 1;
            } else {
                for index in GAUGE_LO..GAUGE_HI {
                    let npc = NPC_DAT[index];
                    self.gauge[index] += BID_MULT_S * (self.rnd_f(npc.high - npc.low) + npc.low);
                }
            }
            let mut idx = GAUGE_LO;
            while idx < GAUGE_HI {
                if self.gauge[idx] >= GAUGE_MAX {
                    self.gauge[idx] = 0.0;
                    if self.curr_bid < NPC_DAT[idx].limit {
                        break;
                    }
                }
                idx += 1;
            }
            if idx >= GAUGE_HI {
                if self.rest_ms() < i32::from(rest_time(&REST_MSG_TIME2, self.rest_index)) {
                    self.set_message2(rest_msg_no(self.rest_index));
                    let se = rest_se_no(self.rest_index);
                    ctx.events.list.push(SimEvent::Se(se));
                    self.rest_index += 1;
                }
            } else {
                self.bidder = idx as u8;
                let mult = self.bid_multiplier();
                let npc = NPC_DAT[idx];
                let inc = (self.rnd_f(f32::from(npc.inc_max) - f32::from(npc.inc_min))
                    + f32::from(npc.inc_min)) as i16;
                self.curr_bid = self
                    .curr_bid
                    .wrapping_add((mult * f32::from(2 * inc)) as i16);
                self.bid_value = self.curr_bid;
                self.msg_next = MSG_NEXT_BID;
                self.after_bet = 4;
                self.uri_seen = 1;
                self.hold = (self.rnd_f(NEXT_BET_SCALE) * BID_TICK) as i16;
                self.bid_mode = PROC_BIKON_C;
                if self.msg_hold != 0 {
                    self.msg_hold = 0;
                } else if self.rest_ms() > i32::from(REST_MSG_TIME1[0])
                    && self.get_rand(RAND_THREE) == 0
                {
                    ctx.layout.npc_name_msg_no = u32::from(self.msg_row().name);
                    self.set_message2(self.bid_comment());
                    self.set_camera_npc(ctx, usize::from(self.speaker), 0);
                    if self.speaker == 0 {
                        self.set_link_anm(DEMO_L_AROUND2);
                    }
                    self.m834 |= M834_REST_MSG;
                    self.pending_prev = self.pending_speaker;
                    self.msg_hold = 1;
                }
                if self.bid_mode == PROC_BIKON_C {
                    self.pending_speaker = self.bidder;
                    self.pending_prev = self.bidder;
                    ctx.counters.message_count = i32::from(self.bid_value);
                    ctx.layout.npc_name_msg_no = u32::from(self.msg_of(self.pending_speaker).name);
                }
                self.m835 |= M835_ON;
                self.m835 &= !M835_START;
            }
        }
        if self.uri_seen != 0 {
            self.do_status = A_STATUS_BID;
        }
    }

    fn bid_comment(&self) -> u32 {
        if self.curr_bid <= BID_LIMIT_LOW {
            MSG_BID_LOW
        } else if self.curr_bid <= BID_LIMIT_MID {
            MSG_BID_MID
        } else if self.curr_bid <= BID_LIMIT_HIGH {
            MSG_BID_HIGH
        } else {
            MSG_BID_TOP
        }
    }

    fn event_main_msg_set(&mut self, ctx: &mut Ctx<'_, '_, '_>) {
        if self.event_mes_set(ctx) {
            self.bid_mode = PROC_MSG_END;
        }
    }

    fn event_main_msg_end(&mut self, ctx: &mut Ctx<'_, '_, '_>) {
        if self.event_mes_end(ctx) {
            if self.link_anm != DEMO_N_WAIT && self.link_anm != DEMO_UNK_029 {
                self.set_link_anm(DEMO_N_WAIT);
            }
            if self.m834 & M834_REST_MSG != 0 {
                self.bid_mode = PROC_BIKON_C;
            } else {
                self.bid_mode = if self.which == ORDER_TALK2 {
                    PROC_URI
                } else {
                    PROC_KAI
                };
                self.eye = ctx.actor_pos;
                self.face_live = 0;
            }
            self.wave = 0.0;
            if self.m834 & M834_TALKED == 0 {
                self.cam_slot = self.get_rand(RAND_THREE) as u8;
                self.cam_from = preset(self.cam_slot).0;
                self.cam_to = preset(self.cam_slot).1;
            }
            self.timer_show = 0;
            self.m834 &= !M834_KEEP;
            self.talk_live = false;
            self.gauge_show = true;
            return;
        }
        if ctx.mesg_send
            && ((self.msg_cache >= MSG_RANGE_TALK_LO && self.msg_cache <= MSG_RANGE_TALK_HI)
                || (self.msg_cache >= MSG_RANGE_SHOP_LO && self.msg_cache <= MSG_RANGE_SHOP_HI))
        {
            self.set_camera_npc(ctx, usize::from(self.prev_speaker), 0);
            self.m834 |= M834_CAM_LOCK;
            self.pending_prev = self.prev_speaker;
        }
    }

    fn event_main_bikon_c(&mut self, ctx: &mut Ctx<'_, '_, '_>) {
        self.pending_speaker = self.bidder;
        self.pending_prev = self.bidder;
        ctx.counters.message_count = i32::from(self.bid_value);
        ctx.layout.npc_name_msg_no = u32::from(self.msg_of(self.pending_speaker).name);
        let Some(npc_pos) = ctx.npc_pos(self.pending_speaker) else {
            self.do_status = A_STATUS_BLANK;
            self.a_status = A_STATUS_HIDDEN;
            return;
        };
        let mut aim = npc_pos;
        aim.y += self.picon_ofs(self.pending_speaker) + ITEM_LIFT;
        let gap = add_calc_pos(&mut self.cam_from, aim, BIKON_SCALE, BIKON_TARGET, BIKON_TOL);
        if gap == 0.0 {
            self.bid_mode = PROC_BIKON_W;
            let mut effect = npc_pos;
            effect.y += self.picon_ofs(self.pending_speaker);
            self.effect_pos = effect;
            self.bid_timer = BIKON_WAIT;
            if self.pending_speaker == 0 {
                ctx.events.list.push(SimEvent::Se(SE_AUC_BID_GAUGE_MAX));
            } else {
                ctx.events.list.push(SimEvent::Se(SE_CM_AUC_CALL_BID));
                self.face_live = 1;
                let slot = self.get_rand(RAND_EIGHT) as usize;
                self.face = LINK_FACE.get(slot).copied().unwrap_or(0);
            }
            self.eye = ctx.npc_eye(self.pending_speaker);
        }
        self.do_status = A_STATUS_BLANK;
        self.a_status = A_STATUS_HIDDEN;
    }

    fn event_main_bikon_w(&mut self, ctx: &mut Ctx<'_, '_, '_>) {
        if self.bid_timer != 0 {
            self.bid_timer -= 1;
        } else {
            self.set_message2(self.msg_next);
            self.set_camera_npc(ctx, usize::from(self.pending_speaker), 0);
            self.talk_live = true;
            if self.pending_speaker != 0 {
                self.m834 |= M834_HOLD;
            }
        }
        self.do_status = A_STATUS_BLANK;
        self.a_status = A_STATUS_HIDDEN;
    }

    fn event_get_item_init(&mut self, ctx: &mut Ctx<'_, '_, '_>) {
        let (item, bit) = if self.which == ORDER_TALK2 {
            let slot = ITEM_DAT22
                .get(usize::from(self.item_index))
                .copied()
                .unwrap_or(0);
            (slot, item_dat2(self.item_index).bit)
        } else {
            let data = item_dat(self.item_index);
            (data.item, data.bit)
        };
        let entity = ctx
            .commands
            .spawn((
                Actor {
                    name: super::event::PRESENT_ACTOR.to_owned(),
                    parameters: 0,
                    pos: ctx.actor_pos,
                    angle: [0; 3],
                    set_id: ITEM_SET_ID,
                    room: ITEM_ROOM,
                    ..Default::default()
                },
                PresentItem {
                    item,
                    hidden: false,
                },
            ))
            .id();
        ctx.item_partner.entity = Some(entity);
        ctx.item_partner.item = item;
        ctx.stories.set(bit);
        self.set_camera_npc(ctx, 0, 0);
    }

    fn event_get_item(&mut self) -> bool {
        true
    }

    fn event_camera_off_init(&mut self) {
        self.off_camera();
    }

    fn off_camera(&mut self) {
        self.m835 |= M835_START;
        self.m835 &= !M835_STOP;
    }

    fn event_get_item_npc_init(&mut self, ctx: &mut Ctx<'_, '_, '_>, cut: usize) {
        self.set_camera_npc(ctx, usize::from(self.speaker), 0);
        self.bid_timer = match ctx.list.cut_number(cut, CUT_TIMER) {
            Some(value) => value as i16,
            None => 0,
        };
        self.m834 = M834_GET_ITEM;
        self.pending_prev = self.speaker;
    }

    fn event_get_item_mes_init(&mut self) {
        let row = self.msg_row();
        self.set_message(u32::from(row.tail));
    }

    fn event_end_init(&mut self) {
        self.blend = 0.0;
        self.flags &= !4;
    }

    fn event_end(&mut self, ctx: &mut Ctx<'_, '_, '_>) -> bool {
        self.blend += BLEND_STEP;
        if self.blend > BLEND_MAX {
            self.blend = BLEND_MAX;
        }
        ctx.set_blend(self.blend);
        self.blend == BLEND_MAX
    }

    fn event_camera_test_init(&mut self) {}

    fn event_camera_test(&mut self) -> bool {
        true
    }

    fn get_item_no(&mut self, ctx: &Ctx<'_, '_, '_>) -> u8 {
        let mut rand_max = 0;
        for index in 0..ITEM_COUNT {
            if !ctx.stories.test(ITEM_DAT[index].bit) {
                rand_max += 1;
            }
        }
        let mut rand = self.get_rand(rand_max);
        let mut index = 0;
        while index < ITEM_PICK_MAX {
            if !ctx.stories.test(ITEM_DAT[index].bit) {
                if rand == 0 {
                    break;
                }
                rand -= 1;
            }
            index += 1;
        }
        index as u8
    }

    fn picon_ofs(&self, speaker: u8) -> f32 {
        GAUGE_TABLE
            .get(usize::from(speaker))
            .copied()
            .unwrap_or(0.0)
    }

    fn set_link_anm(&mut self, anm: u8) {
        let mut link_anm = anm;
        if link_anm == DEMO_N_WAIT && self.seen_mask == 0 {
            link_anm = DEMO_UNK_029;
        }
        self.link_anm = link_anm;
        self.link_param0 = DEMO_PARAM0;
    }

    fn bid_multiplier(&self) -> f32 {
        let rest = self.rest_ms();
        if rest < BID_RISE_S {
            BID_MULT_S
        } else if rest < BID_RISE_M {
            BID_MULT_M
        } else {
            1.0
        }
    }

    fn rest_ms(&self) -> i32 {
        self.timer_limit - self.timer_now
    }

    fn gauge_at(&self, index: i32) -> f32 {
        self.gauge.get(index as usize).copied().unwrap_or(0.0)
    }

    fn gauge_set(&mut self, index: usize, value: f32) {
        if let Some(slot) = self.gauge.get_mut(index) {
            *slot = value;
        }
    }

    fn set_camera_npc(&mut self, ctx: &mut Ctx<'_, '_, '_>, idx: usize, param: i16) {
        let Some(cam) = NPC_CAMERA_DAT.get(idx).copied() else {
            return;
        };
        let Some((pos, yaw)) = ctx.npc_view(idx) else {
            self.cam_from = ctx.actor_pos;
            self.cam_from.y += cam.height;
            self.cam_to = Vec3::new(0.0, CAM_LOOK_HEIGHT, 0.0);
            self.on_camera();
            return;
        };
        self.cam_from = pos;
        self.cam_from.y += cam.height;
        let angle = walk::from_s16(yaw);
        let mut to = Vec3::new(cam.dist * angle.sin(), 0.0, cam.dist * angle.cos());
        let mut rot_y = cam.yaw;
        if param != 0 {
            rot_y = rot_y.wrapping_add(param);
        } else {
            rot_y = rot_y.wrapping_add(self.rnd_f(CAM_YAW_JITTER) as i16);
        }
        to = rot_x(rot_y, walk::from_s16(cam.pitch), to);
        self.cam_to = to + self.cam_from;
        self.on_camera();
    }

    fn on_camera(&mut self) {
        self.m835 |= M835_ON;
        self.m835 &= !M835_START;
    }

    fn next_bet(&mut self) {
        if self.next_bet_hold != 0 {
            self.next_bet_hold -= 1;
            return;
        }
        for index in GAUGE_LO..GAUGE_HI {
            let span = self.rest_ms() / 1000;
            let mut rnd = self.get_rand(NEXT_BET_SPAN - span) as f32;
            if rnd > NEXT_BET_CAP {
                rnd = NEXT_BET_CAP;
            }
            self.gauge[index] += rnd;
            if self.gauge[index] >= GAUGE_MAX {
                self.next_bet_hold = self.next_bet_hold.saturating_add(1);
            }
        }
        self.next_bet_hold = self.next_bet_hold.saturating_add(NEXT_BET_BONUS);
    }

    fn next_msg_status(&mut self, ctx: &mut Ctx<'_, '_, '_>) -> u8 {
        let mut status = STATUS_MSG_CONTINUES;
        match self.msg_no {
            MSG_OPEN => self.msg_no = MSG_OPEN_NEXT,
            MSG_ITEM_SHOW => {
                if self.which == ORDER_TALK2 {
                    self.msg_no = MSG_SHOW_TALK2;
                } else if ctx.stories.test(BIT_TALKED_NPC) {
                    let reg = ctx.stories.reg(BIT_TALKED_NPC_REG);
                    let row = NPC_MSG_DAT.get(usize::from(reg)).copied();
                    let row = row.unwrap_or(NPC_MSG_DAT[0]);
                    ctx.layout.npc_name_msg_no = u32::from(row.name);
                    self.msg_no = MSG_SHOW_2;
                } else {
                    self.msg_no = MSG_SHOW_1;
                }
            }
            MSG_SLOT => {
                ctx.counters.mini_game_rupee = i32::from(self.curr_bid);
                self.auction_rupee = self.curr_bid;
                self.slot_show = true;
                self.msg_no = MSG_SLOT_NEXT;
            }
            MSG_NEXT_BID => {
                ctx.counters.mini_game_rupee = i32::from(self.curr_bid);
                self.auction_rupee = self.curr_bid;
                if self.after_bet < AFTER_BET_MAX {
                    self.msg_no = after_bet_msg(self.after_bet);
                } else {
                    let row = self.msg_row();
                    if self.pending_speaker == 0 && row.mid != 0 {
                        if self.rest_ms() < BID_RISE_M {
                            self.msg_no = u32::from(row.high);
                        } else {
                            self.msg_no = u32::from(row.mid);
                        }
                    } else {
                        status = STATUS_MSG_ENDS;
                    }
                }
                self.prev_speaker = self.speaker;
                self.speaker = self.pending_speaker;
                self.seen_mask = self.pending_speaker;
                if self.seen_mask != 0 {
                    self.set_link_anm(DEMO_N_WAIT);
                }
            }
            MSG_BID_OPEN => {
                let msg_set_no = self.msg_set_no;
                if i32::try_from(ctx.rupees.count).unwrap_or(i32::MAX) < i32::from(msg_set_no) {
                    ctx.events.list.push(SimEvent::Se(SE_AUC_BID_NG));
                    self.set_link_anm(DEMO_UNK_074);
                    self.msg_no = MSG_BID_FAIL_LOW;
                    self.gauge[0] = GAUGE_RESET;
                } else if msg_set_no <= self.curr_bid {
                    ctx.events.list.push(SimEvent::Se(SE_AUC_BID_NG));
                    self.set_link_anm(DEMO_UNK_074);
                    self.msg_no = MSG_BID_FAIL_HIGH;
                    self.gauge[0] = GAUGE_RESET;
                } else {
                    ctx.events.list.push(SimEvent::Se(SE_AUC_BID_OK));
                    self.gauge[0] = 0.0;
                    if msg_set_no == BID_ALL {
                        self.msg_no = MSG_QUIT_END;
                        self.prev_speaker = self.speaker;
                        self.speaker = self.pending_speaker;
                        self.seen_mask = self.pending_speaker;
                    } else {
                        self.msg_no = MSG_NEXT_BID;
                        self.after_bet = 0;
                        let mut index = 0;
                        while index < 4 {
                            let rate = AFTER_BET_RATE[index];
                            let chk = AFTER_BET_CHK[index];
                            let wait = AFTER_BET_WAIT[index];
                            if msg_set_no >= (f32::from(self.curr_bid) * rate) as i16
                                && self.curr_bid >= chk
                            {
                                self.hold = (AFTER_BET_COUNT
                                    * (f32::from(wait[0])
                                        + self.rnd_f(f32::from(wait[1]) - f32::from(wait[0]))))
                                    as i16;
                                break;
                            }
                            index += 1;
                        }
                        self.after_bet = index as i32;
                    }
                    self.curr_bid = msg_set_no;
                    ctx.counters.message_count = i32::from(self.curr_bid);
                }
            }
            MSG_QUIT_SELECT => {
                if ctx.select_num == 0 {
                    self.msg_no = MSG_QUIT_LEAVE;
                } else {
                    status = STATUS_MSG_ENDS;
                }
            }
            MSG_CANCEL => {
                if ctx.select_num == 0 {
                    self.finish_flag = 1;
                }
                status = STATUS_MSG_ENDS;
            }
            MSG_QUIT_LEAVE => {
                ctx.next.target = Some(Target {
                    stage: NEXT_STAGE.to_owned(),
                    room: NEXT_STAGE_ROOM,
                    start: NEXT_STAGE_START,
                });
                self.finish_flag = 1;
                status = STATUS_MSG_ENDS;
            }
            MSG_QUIT_END => {
                self.finish_flag = 1;
                status = STATUS_MSG_ENDS;
            }
            MSG_REST4 | MSG_REST3 | MSG_REST2 | MSG_REST1 => {
                self.set_link_anm(DEMO_L_AROUND2);
                let rnd = self.get_rand(RAND_SIX) + NPC_RND_BIAS;
                self.msg_no = u32::from(self.msg_of(rnd as u8).tail);
                self.prev_speaker = rnd as u8;
            }
            0x1D48 | 0x33A2 | 0x33A8 | 0x33A5 | 0x339F | 0x1D4B | 0x1D3F | 0x1D45 | 0x1D42
            | 0x1D3C => {
                self.m834 |= M834_TALKED;
                self.cam_sway = 0;
                status = STATUS_MSG_ENDS;
            }
            MSG_FINISH => {
                if self.which == ORDER_TALK2 {
                    self.msg_no = MSG_ITEM_COST;
                } else {
                    if self.speaker != 0 {
                        ctx.stories.set(BIT_TALKED_NPC);
                        ctx.stories.set_reg(BIT_TALKED_NPC_REG, self.mdl_no());
                        ctx.stories.set_reg(REG_ITEM_INDEX, self.item_index);
                    } else {
                        ctx.stories.clear(BIT_TALKED_NPC);
                    }
                    if let Some(entity) = self.item_entity {
                        ctx.commands.entity(entity).despawn();
                        self.item_entity = None;
                    }
                    self.item_pid = PROC_ERROR;
                    status = STATUS_MSG_ENDS;
                }
            }
            MSG_ITEM_COST => {
                self.item_rupee_count += i32::from(self.curr_bid);
                status = STATUS_MSG_ENDS;
            }
            MSG_PAY => {
                self.item_rupee_count -= i32::from(self.curr_bid);
                self.msg_no = MSG_PAY_NEXT;
            }
            MSG_PAY_CANCEL | MSG_PAY_NEXT => self.msg_no = MSG_PAY_DONE,
            _ => status = STATUS_MSG_ENDS,
        }
        if status == STATUS_MSG_CONTINUES {
            self.msg_cache = self.msg_no;
        } else {
            self.msg_cache = 0;
        }
        status
    }
}

fn event_slot(list: &EventList, name: &str) -> i16 {
    list.event_index(name)
        .and_then(|index| i16::try_from(index).ok())
        .unwrap_or(NO_EVENT)
}

fn item_dat(index: u8) -> ItemData {
    ITEM_DAT
        .get(usize::from(index))
        .copied()
        .unwrap_or(ITEM_DAT[0])
}

fn item_dat2(index: u8) -> ItemData {
    ITEM_DAT2
        .get(usize::from(index))
        .copied()
        .unwrap_or(ITEM_DAT2[0])
}

fn preset(slot: u8) -> (Vec3, Vec3) {
    let row = CAMERA_PRESET
        .get(usize::from(slot))
        .copied()
        .unwrap_or(CAMERA_PRESET[0]);
    (Vec3::from_array(row[0]), Vec3::from_array(row[1]))
}

fn rest_time(table: &[i16; REST_MSG_TABLE], index: i32) -> i16 {
    let at = index.clamp(0, REST_MSG_TABLE as i32 - 1) as usize;
    table.get(at).copied().unwrap_or(0)
}

fn rest_msg_no(index: i32) -> u32 {
    let at = index.clamp(0, REST_MSG_MAX as i32 - 1) as usize;
    REST_MSG_NO.get(at).copied().unwrap_or(0)
}

fn rest_se_no(index: i32) -> u32 {
    let at = index.clamp(0, REST_MSG_MAX as i32 - 1) as usize;
    REST_SE_NO.get(at).copied().unwrap_or(0)
}

fn after_bet_msg(index: i32) -> u32 {
    let at = index.clamp(0, 3) as usize;
    AFTER_BET_MSG_NO.get(at).copied().unwrap_or(0)
}

fn get_rand(seed: &mut [i32; 3], max: i32) -> i32 {
    let mut rnd = get_rand_f(seed, max as f32) as i32;
    if rnd == max {
        rnd = 0;
    }
    rnd
}

fn get_rand_f(seed: &mut [i32; 3], range: f32) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = (seed[0]) as f32 / RND_SCALE[0]
        + (seed[1]) as f32 / RND_SCALE[1]
        + (seed[2]) as f32 / RND_SCALE[2];
    (sum % 1.0).abs() * range
}

fn rot_x(rot_y: i16, pitch: f32, value: Vec3) -> Vec3 {
    let (sx, cx) = pitch.sin_cos();
    let x = value.x;
    let y = value.y * cx - value.z * sx;
    let z = value.y * sx + value.z * cx;
    let (sy, cy) = walk::from_s16(rot_y).sin_cos();
    Vec3::new(x * cy + z * sy, y, z * cy - x * sy)
}

fn add_calc_pos(value: &mut Vec3, target: Vec3, scale: f32, max_step: f32, min_step: f32) -> f32 {
    if *value != target {
        let mut diff = *value - target;
        let mut step = diff.length();
        if step < min_step {
            *value = target;
        } else {
            step *= scale;
            diff *= scale;
            if step != 0.0 {
                if step > max_step {
                    diff *= max_step / step;
                } else if step < min_step {
                    diff *= min_step / step;
                }
                *value -= diff;
            } else {
                *value = target;
            }
        }
    }
    (*value - target).length()
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    list: Res<'w, EventList>,
    flags: Res<'w, TextFlags>,
    do_intent: Res<'w, DoIntent>,
    target: Res<'w, TargetIntent>,
    archives: Option<Res<'w, MsgArchives>>,
    counters: ResMut<'w, Counters>,
    layout: ResMut<'w, Layout>,
    procs: ResMut<'w, MsgProcs>,
    info: ResMut<'w, MsgGameInfo>,
    npcs: Query<'w, 's, &'static Actor>,
    players: Query<'w, 's, &'static Player>,
}

#[derive(Clone, Copy)]
struct NpcView {
    pos: Vec3,
    yaw: i16,
}

struct Ctx<'a, 'w, 's> {
    entity: Entity,
    actor_pos: Vec3,
    event_run: bool,
    trig_a: bool,
    trig_b: bool,
    mesg_send: bool,
    select_num: u8,
    clear_count: u8,
    link_rate: f32,
    link_face: u8,
    blend: f32,
    shake_a: (u8, u8),
    shake_b: (u8, u8),
    npcs: Vec<NpcView>,
    player: Option<Player>,
    stories: &'a mut StoryBits,
    ev_flags: &'a mut EventFlags,
    events: &'a mut SimEvents,
    list: &'a EventList,
    active: &'a mut ActiveEvent,
    next: &'a mut NextStage,
    rupees: &'a mut Rupees,
    order_flags: &'a mut OrderFlags,
    item_partner: &'a mut ItemPartner,
    counters: &'a mut Counters,
    timer: &'a mut IsleTimer,
    layout: &'a mut Layout,
    procs: &'a mut MsgProcs,
    info: &'a mut MsgGameInfo,
    archive: Option<&'a MsgArchives>,
    one: &'a mut Msg1Box,
    two: &'a mut Msg2Box,
    commands: &'a mut Commands<'w, 's>,
}

impl Ctx<'_, '_, '_> {
    fn box_status(&self) -> u8 {
        if self.procs.search_by_name(FPC_NM_MSG).is_some() {
            self.one.status
        } else if self.procs.search_by_name(FPC_NM_MSG2).is_some() {
            self.two.status
        } else {
            TALK_IDLE
        }
    }

    fn set_box_status(&mut self, status: u8) {
        if self.procs.search_by_name(FPC_NM_MSG).is_some() {
            self.one.status = status;
        } else if self.procs.search_by_name(FPC_NM_MSG2).is_some() {
            self.two.status = status;
        }
    }

    fn npc_view(&self, idx: usize) -> Option<(Vec3, i16)> {
        if idx == 0 {
            return self
                .player
                .as_ref()
                .map(|link| (link.pos, walk::to_s16(link.yaw)));
        }
        self.npcs
            .get(idx)
            .copied()
            .map(|npc| (npc.pos, npc.yaw))
    }

    fn npc_pos(&self, idx: u8) -> Option<Vec3> {
        self.npc_view(usize::from(idx)).map(|(pos, _)| pos)
    }

    fn npc_eye(&self, idx: u8) -> Vec3 {
        self.npc_view(usize::from(idx))
            .map(|(pos, _)| pos + Vec3::new(0.0, EYE_LIFT, 0.0))
            .unwrap_or(Vec3::ZERO)
    }

    fn set_link_face(&mut self, face: u8) {
        self.link_face = face;
    }

    fn shake(&mut self, kind: u8, time: u8) {
        if kind == 5 {
            self.shake_a = (kind, time);
        } else {
            self.shake_b = (kind, time);
        }
    }

    fn set_blend(&mut self, blend: f32) {
        self.blend = blend;
    }

}

pub fn control(
    mut watch: Watch<'_, '_>,
    mut active: ResMut<ActiveEvent>,
    mut ev_flags: ResMut<EventFlags>,
    mut stories: ResMut<StoryBits>,
    mut events: ResMut<SimEvents>,
    mut next: ResMut<NextStage>,
    mut rupees: ResMut<Rupees>,
    mut order_flags: ResMut<OrderFlags>,
    mut item_partner: ResMut<ItemPartner>,
    mut one: ResMut<Msg1Box>,
    mut two: ResMut<Msg2Box>,
    mut isle_timer: ResMut<IsleTimer>,
    mut actors: Query<(Entity, &Actor, &mut Auction), (With<Auction>, Without<Player>)>,
    mut commands: Commands,
) {
    let archive = watch.archives.as_deref();
    let Watch {
        mut counters,
        mut layout,
        mut procs,
        mut info,
        ..
    } = watch;
    let clear_count = watch.flags.clear_count;
    let player = watch.players.iter().next().copied();
    let views: Vec<NpcView> = watch
        .npcs
        .iter()
        .filter(|actor| is_npc_auction(&actor.name))
        .map(|actor| NpcView {
            pos: actor.pos,
            yaw: actor.angle[1],
        })
        .collect();
    let select_num = one.select_num;
    let event_run = info.event_run_check();
    for (entity, actor, mut auc) in &mut actors {
        let actor = &*actor;
        let auc = &mut *auc;
        let mut ctx = Ctx {
            entity,
            actor_pos: actor.pos,
            event_run,
            trig_a: watch.do_intent.0,
            trig_b: watch.target.0,
            mesg_send: watch.do_intent.0,
            select_num,
            clear_count,
            link_rate: 0.0,
            link_face: 0,
            blend: 0.0,
            shake_a: (0, 0),
            shake_b: (0, 0),
            npcs: views.clone(),
            player,
            stories: &mut stories,
            ev_flags: &mut ev_flags,
            events: &mut events,
            list: &watch.list,
            active: &mut active,
            next: &mut next,
            rupees: &mut rupees,
            order_flags: &mut order_flags,
            item_partner: &mut item_partner,
            counters: &mut counters,
            timer: &mut isle_timer,
            layout: &mut layout,
            procs: &mut procs,
            info: &mut info,
            archive,
            one: &mut one,
            two: &mut two,
            commands: &mut commands,
        };
        if !auc.entered {
            auc.entered = true;
            auc.create_init(&mut ctx);
        }
        auc.execute(&mut ctx);
    }
}

fn is_npc_auction(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("Auc") else {
        return false;
    };
    matches!(
        rest.parse::<u8>(),
        Ok(0) | Ok(1) | Ok(2) | Ok(3) | Ok(4) | Ok(5) | Ok(6) | Ok(7)
    )
}