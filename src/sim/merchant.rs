
use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::event::{ActiveEvent, EventList, ItemPartner, PresentItem};
use super::flags::{EventFlags, OrderFlags, StoryBits};
use super::ground;
use super::intent::DoIntent;
use super::pickup::Rupees;
use super::player::Player;
use super::shop;
use super::stream;
use super::walk;
use super::wind::WindTag;
use crate::assets::MsgArchives;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, MsgGameInfo, MsgProcs, fop_msgm_message_set,
    fop_msgm_message_set_look_at,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED,
    STATUS_MSG_ENDS,
};

pub const MERCHANT_ACTORS: [&str; 3] = ["RotenA", "RotenB", "RotenC"];

pub const MERCHANT_COUNT: usize = 3;
pub const ITEM_SLOTS: usize = 12;

const PRM_RAIL_ID_SHIFT: u32 = 0x10;
const PRM_RAIL_ID_MASK: u32 = 0xFF;
const PRM_NPC_NO_SHIFT: u32 = 0x18;
const PRM_NPC_NO_MASK: u32 = 0xFF;

const NO_RAIL: u8 = 0xFF;
const MSG_CLEAR_COUNT: u8 = 0;
const NO_EVENT: i16 = -1;
const NO_MSG: u32 = 0;
const TALK_IDLE: u8 = 0xFF;
const DEMO_ITEM_ACTOR: &str = "PresentItem";

const BIT_ENABLED: u16 = 0x1108;
const BIT_MAP_TAKEN: u16 = 0x3E04;

const SAVE_TALKED: [u16; MERCHANT_COUNT] = [0x1320, 0x1310, 0x1308];
const SAVE_EXCHANGED: [u16; MERCHANT_COUNT] = [0xCB03, 0xCA03, 0xC903];
const SAVE_OFFERED: [u16; MERCHANT_COUNT] = [0x1304, 0x1302, 0x1301];

const EV_EXCHANGE_1ST: &str = "ROTEN_EXCHANGE_1ST";
const EV_EXCHANGE_2ND: &str = "ROTEN_EXCHANGE_2ND";
const EV_CHANGE_ITEM: &str = "ROTEN_CHANGE_ITEM";
const EV_GET_MAP: &str = "ROTEN_GET_MAP";
const EV_DEFAULT_TALK_XY: &str = "DEFAULT_TALK_XY";

const ORDER_TALK_ITEM: u8 = 2;
const ORDER_1ST: u8 = 3;
const ORDER_2ND: u8 = 4;
const ORDER_ITEM: u8 = 5;
const ORDER_MAP: u8 = 6;

const EVT_RESET_FLAG: u32 = 8;
const EVT_CND_CANTALK: u16 = 0x0001;
const EVT_CND_CANTALKITEM: u16 = 0x0020;

const BIT_SELECT_YES: u16 = 0x0008;
const BIT_SELECT_NO: u16 = 0x0010;
const BIT_NO_RUPEE: u16 = 0x0020;
const BIT_BAG_FULL: u16 = 0x0040;
const BIT_EVENT_RUN: u16 = 0x4000;
const BIT_EVENT_END: u16 = 0x8000;
const BIT_EVENT_MASK: u16 = 0xC000;

const ITEM_TOWN_FLOWER: u8 = 0x8C;
const ITEM_SHOP_GURU_STATUE: u8 = 0x97;
const ITEM_XXX_039: u8 = 0xA2;
const EXCHANGE_LIMIT: u8 = 3;
const RESERVE_BITS: u32 = 32;
const GET_ITEM_NO: [u8; 1] = [shop::ITEM_HEART_PIECE];

const RESERVE_NONE: u8 = 0xFF;
const BTN_NONE: u8 = 3;
const BTN_X: u8 = 0;
const BTN_Y: u8 = 1;
const BTN_Z: u8 = 2;
const TALK_BTN_X: u8 = 1;
const TALK_BTN_Y: u8 = 2;
const TALK_BTN_Z: u8 = 3;

const SEQ_NONE: u32 = 0;
const SEQ_SELECT: u32 = 1;
const SEQ_BUY: u32 = 2;
const SEQ_END: u32 = 3;

const CMD_NONE: u8 = 0;
const CMD_TALK: u8 = 1;
const CMD_DEMO: u8 = 2;

const ATTN_NONE: u8 = 0;
const ATTN_PLAYER: u8 = 1;
const ATTN_HOME: u8 = 2;

const LOOK_DIST: f32 = 250.0;
const LOOK_ANGLE: i32 = 0x7FFF;
const LOOK_ANGLE_HELD: i32 = 0x071C;
const LOOK_RANGE_TALK: i32 = 0x7FFF;
const LOOK_RANGE_HALF: i32 = 0x4000;
const LOOK_RANGE_BONUS: f32 = 40.0;
const LOOK_SPEED: i16 = 0x0352;
const TURN_SPEED_WALK: i16 = 0x0190;
const LOOK_EYE_Y: f32 = 0.0;
const PLAYER_HEAD_TOP: f32 = 60.0;
const LOOK_SCALE: i16 = 4;
const LOOK_MAX_STEP: i16 = 0x800;
const HEAD_MOVE_SCALE: i32 = 4;
const HEAD_MOVE_MIN: i32 = 1;
const HEAD_TURN_MIN_STEP: i16 = 0x60;
const HEAD_LIMIT_X: i16 = 0x09C4;
const HEAD_LIMIT_Y: i16 = 0x1388;
const HEAD_MIN_X: i16 = 0xF63C;
const HEAD_MIN_Y: i16 = 0xEC78;
const HEAD_LOCK_VALUE: i16 = 0;
const HEAD_TURN_STEP: i16 = 0x03E8;

const WALK_SPEED: f32 = 1.0;
const BAG_OFFSET: f32 = 40.0;
const HEAD_SPHERE_R: f32 = 30.0;
const HEAD_SPHERE_DX: f32 = 1.0;
const HEAD_SPHERE_R_B: f32 = 50.0;
const EYE_Y_OFFSET: f32 = 150.0;
const CULL_MIN: [f32; 3] = [-200.0, 0.0, -200.0];
const CULL_MAX: [f32; 3] = [200.0, 300.0, 200.0];

const WAIT_MIN: i16 = 0x003C;
const WAIT_RANGE: i16 = 0x001E;
const WALK_MIN: i16 = 0x00C8;
const WALK_RANGE: i16 = 0x0064;
const IDLE_FRAMES: i16 = 0x1E;

const TEX_PATTERN_MAX: u8 = 0xFF;
const TEX_PATTERN_STEP: i16 = 0x78;

const TBL_WAIT: u8 = 0;
const TBL_TALK: u8 = 1;
const TBL_WALK: u8 = 2;
const TBL_TALK2: u8 = 3;
const TBL_TALK3: u8 = 4;
const TBL_TALK4: u8 = 5;
const TBL_TALK5: u8 = 6;
const TBL_WATASU: u8 = 7;
const TBL_ITEMWAIT: u8 = 8;
const TBL_WIND: u8 = 9;

const ANM_END: u8 = 0xFF;
const ANM_NONE: u8 = 0xFF;
const ANM_ONCE: i32 = 0;
const ANM_LOOP: i32 = 2;
const ANM_RESTART: f32 = -1.0;
const PATH_TRIG_SCALE: f32 = 32767.0;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const MESG_ANIME_CLEAR: u8 = 0xFF;

const MSG_KOUKAN_END: [u32; MERCHANT_COUNT] = [0x293D, 0x29A1, 0x2A05];
const MSG_KOUKAN_FIRST: [u32; MERCHANT_COUNT] = [0x293E, 0x29A2, 0x2A06];
const MSG_KOUKAN_NO: [u32; MERCHANT_COUNT] = [0x292F, 0x2993, 0x29F7];
const MSG_KOUKAN_RUPEE: [u32; MERCHANT_COUNT] = [0x293C, 0x29A0, 0x2A04];

const MSG_FIRST_TALK_A: [u32; 6] = [0x2905, 0x2906, 0x2907, 0x2908, 0x2909, 0x0000];
const MSG_FIRST_TALK_B: [u32; 6] = [0x2969, 0x296A, 0x296B, 0x296C, 0x296D, 0x0000];
const MSG_FIRST_TALK_C: [u32; 6] = [0x29CD, 0x29CE, 0x29CF, 0x29D0, 0x29D1, 0x0000];

const MSG_EXCHANGE3_A: [u32; 4] = [0x290C, 0x290D, 0x290E, 0x0000];
const MSG_EXCHANGE3_B: [u32; 4] = [0x2970, 0x2971, 0x2972, 0x0000];
const MSG_EXCHANGE3_C: [u32; 4] = [0x29D4, 0x29D5, 0x29D6, 0x0000];

const MSG_EXCHANGE_A: [u32; 3] = [0x290F, 0x2910, 0x0000];
const MSG_EXCHANGE_B: [u32; 3] = [0x2973, 0x2974, 0x0000];
const MSG_EXCHANGE_C: [u32; 3] = [0x29D7, 0x29D8, 0x0000];

const MSG_ETC_A: [u32; 3] = [0x290A, 0x290B, 0x0000];
const MSG_ETC_B: [u32; 3] = [0x296E, 0x296F, 0x0000];
const MSG_ETC_C: [u32; 3] = [0x29D2, 0x29D3, 0x0000];

const MSG_XY_EXCHANGE3_A: [u32; 3] = [0x2911, 0x2912, 0x0000];
const MSG_XY_EXCHANGE3_B: [u32; 3] = [0x2975, 0x2976, 0x0000];
const MSG_XY_EXCHANGE3_C: [u32; 3] = [0x29D9, 0x29DA, 0x0000];

const MSG_XY_NO_ITEM_A: [u32; 2] = [0x2914, 0x0000];
const MSG_XY_NO_ITEM_B: [u32; 2] = [0x2978, 0x0000];
const MSG_XY_NO_ITEM_C: [u32; 2] = [0x29DC, 0x0000];

const MSG_XY_BAD_ITEM_A: [u32; 2] = [0x2915, 0x0000];
const MSG_XY_BAD_ITEM_B: [u32; 2] = [0x2979, 0x0000];
const MSG_XY_BAD_ITEM_C: [u32; 2] = [0x29DD, 0x0000];

const MSG_AFTER_GET_A: [u32; 3] = [0x293F, 0x2940, 0x0000];
const MSG_AFTER_GET_B: [u32; 3] = [0x29A3, 0x29A4, 0x0000];
const MSG_AFTER_GET_C: [u32; 3] = [0x2A07, 0x2A08, 0x0000];

const MSG_AFTER_GET2_A: [u32; 2] = [0x2941, 0x0000];
const MSG_AFTER_GET2_B: [u32; 2] = [0x29A5, 0x0000];
const MSG_AFTER_GET2_C: [u32; 2] = [0x2A09, 0x0000];

const MSG_COLLECT_MAP: [u32; 2] = [0x29A6, 0x0000];

const MSG_TRY_FORCE: [u32; 7] = [0x29A7, 0x29A8, 0x29A9, 0x29AA, 0x0003, 0x29AB, 0x0000];

const ITEM_DAT: [[u8; ITEM_SLOTS]; MERCHANT_COUNT] = [
    [0x01, 0x00, 0x07, 0x04, 0x09, 0x03, 0x07, 0x09, 0x09, 0x07, 0x09, 0x0A],
    [0x01, 0x02, 0x01, 0x0A, 0x09, 0x03, 0x02, 0x09, 0x09, 0x08, 0x09, 0x0A],
    [0x01, 0x00, 0x06, 0x05, 0x07, 0x04, 0x02, 0x04, 0x05, 0x05, 0x0B, 0x0A],
];

const MSG_KOUKAN_ITEM: [[u32; 3]; MERCHANT_COUNT * ITEM_SLOTS] = [
    [0x2913, 0x2916, 0x0000],
    [0x2913, 0x2917, 0x0000],
    [0x2913, 0x2918, 0x0000],
    [0x2913, 0x2919, 0x0000],
    [0x2913, 0x291A, 0x0000],
    [0x2913, 0x291B, 0x0000],
    [0x2913, 0x291C, 0x0000],
    [0x2913, 0x291D, 0x0000],
    [0x2913, 0x291E, 0x0000],
    [0x2913, 0x291F, 0x0000],
    [0x2913, 0x2920, 0x0000],
    [0x2913, 0x2921, 0x0000],
    [0x2977, 0x297A, 0x0000],
    [0x2977, 0x297B, 0x0000],
    [0x2977, 0x297C, 0x0000],
    [0x2977, 0x297D, 0x0000],
    [0x2977, 0x297E, 0x0000],
    [0x2977, 0x297F, 0x0000],
    [0x2977, 0x2980, 0x0000],
    [0x2977, 0x2981, 0x0000],
    [0x2977, 0x2982, 0x0000],
    [0x2977, 0x2983, 0x0000],
    [0x2977, 0x2984, 0x0000],
    [0x2977, 0x2985, 0x0000],
    [0x29DB, 0x29DE, 0x0000],
    [0x29DB, 0x29DF, 0x0000],
    [0x29DB, 0x29E0, 0x0000],
    [0x29DB, 0x29E1, 0x0000],
    [0x29DB, 0x29E2, 0x0000],
    [0x29DB, 0x29E3, 0x0000],
    [0x29DB, 0x29E4, 0x0000],
    [0x29DB, 0x29E5, 0x0000],
    [0x29DB, 0x29E6, 0x0000],
    [0x29DB, 0x29E7, 0x0000],
    [0x29DB, 0x29E8, 0x0000],
    [0x29DB, 0x29E9, 0x0000],
];

const MSG_KOUKAN_ITEM2: [[u32; 4]; MERCHANT_COUNT * ITEM_SLOTS] = [
    [0x2922, 0x0001, 0x2930, 0x0000],
    [0x2923, 0x0001, 0x2931, 0x0000],
    [0x2924, 0x0001, 0x2932, 0x0000],
    [0x2925, 0x0001, 0x2933, 0x0000],
    [0x2926, 0x0001, 0x2934, 0x0000],
    [0x2927, 0x0001, 0x2935, 0x0000],
    [0x2928, 0x0001, 0x2936, 0x0000],
    [0x2929, 0x0001, 0x2937, 0x0000],
    [0x292A, 0x0001, 0x2938, 0x0000],
    [0x292B, 0x0001, 0x2939, 0x0000],
    [0x292C, 0x0001, 0x293A, 0x0000],
    [0x292D, 0x0001, 0x293B, 0x0000],
    [0x2986, 0x0001, 0x2994, 0x0000],
    [0x2987, 0x0001, 0x2995, 0x0000],
    [0x2988, 0x0001, 0x2996, 0x0000],
    [0x2989, 0x0001, 0x2997, 0x0000],
    [0x298A, 0x0001, 0x2998, 0x0000],
    [0x298B, 0x0001, 0x2999, 0x0000],
    [0x298C, 0x0001, 0x299A, 0x0000],
    [0x298D, 0x0001, 0x299B, 0x0000],
    [0x298E, 0x0001, 0x299C, 0x0000],
    [0x298F, 0x0001, 0x299D, 0x0000],
    [0x2990, 0x0001, 0x299E, 0x0000],
    [0x2991, 0x0001, 0x299F, 0x0000],
    [0x29EA, 0x0001, 0x29F8, 0x0000],
    [0x29EB, 0x0001, 0x29F9, 0x0000],
    [0x29EC, 0x0001, 0x29FA, 0x0000],
    [0x29ED, 0x0001, 0x29FB, 0x0000],
    [0x29EE, 0x0001, 0x29FC, 0x0000],
    [0x29EF, 0x0001, 0x29FD, 0x0000],
    [0x29F0, 0x0001, 0x29FE, 0x0000],
    [0x29F1, 0x0001, 0x29FF, 0x0000],
    [0x29F2, 0x0001, 0x2A00, 0x0000],
    [0x29F3, 0x0001, 0x2A01, 0x0000],
    [0x29F4, 0x0001, 0x2A02, 0x0000],
    [0x29F5, 0x0001, 0x2A03, 0x0000],
];

const MSG_KOUKAN_ITEM3: [[u32; 4]; MERCHANT_COUNT * ITEM_SLOTS] = [
    [0x292E, 0x2930, 0x0002, 0x0000],
    [0x292E, 0x2931, 0x0002, 0x0000],
    [0x292E, 0x2932, 0x0002, 0x0000],
    [0x292E, 0x2933, 0x0002, 0x0000],
    [0x292E, 0x2934, 0x0002, 0x0000],
    [0x292E, 0x2935, 0x0002, 0x0000],
    [0x292E, 0x2936, 0x0002, 0x0000],
    [0x292E, 0x2937, 0x0002, 0x0000],
    [0x292E, 0x2938, 0x0002, 0x0000],
    [0x292E, 0x2939, 0x0002, 0x0000],
    [0x292E, 0x293A, 0x0002, 0x0000],
    [0x292E, 0x293B, 0x0002, 0x0000],
    [0x2992, 0x2994, 0x0002, 0x0000],
    [0x2992, 0x2995, 0x0002, 0x0000],
    [0x2992, 0x2996, 0x0002, 0x0000],
    [0x2992, 0x2997, 0x0002, 0x0000],
    [0x2992, 0x2998, 0x0002, 0x0000],
    [0x2992, 0x2999, 0x0002, 0x0000],
    [0x2992, 0x299A, 0x0002, 0x0000],
    [0x2992, 0x299B, 0x0002, 0x0000],
    [0x2992, 0x299C, 0x0002, 0x0000],
    [0x2992, 0x299D, 0x0002, 0x0000],
    [0x2992, 0x299E, 0x0002, 0x0000],
    [0x2992, 0x299F, 0x0002, 0x0000],
    [0x29F6, 0x29F8, 0x0002, 0x0000],
    [0x29F6, 0x29F9, 0x0002, 0x0000],
    [0x29F6, 0x29FA, 0x0002, 0x0000],
    [0x29F6, 0x29FB, 0x0002, 0x0000],
    [0x29F6, 0x29FC, 0x0002, 0x0000],
    [0x29F6, 0x29FD, 0x0002, 0x0000],
    [0x29F6, 0x29FE, 0x0002, 0x0000],
    [0x29F6, 0x29FF, 0x0002, 0x0000],
    [0x29F6, 0x2A00, 0x0002, 0x0000],
    [0x29F6, 0x2A01, 0x0002, 0x0000],
    [0x29F6, 0x2A02, 0x0002, 0x0000],
    [0x29F6, 0x2A03, 0x0002, 0x0000],
];

const ANM_WAIT: [AnmDat; 1] = [AnmDat {
    index: 0x00,
    start: 0x0E,
    count: 0xFF,
}];
const ANM_TALK: [AnmDat; 1] = [AnmDat {
    index: 0x01,
    start: 0x0E,
    count: 0xFF,
}];
const ANM_WALK: [AnmDat; 1] = [AnmDat {
    index: 0x02,
    start: 0x0E,
    count: 0xFF,
}];
const ANM_TALK2: [AnmDat; 1] = [AnmDat {
    index: 0x03,
    start: 0x0E,
    count: 0xFF,
}];
const ANM_TALK3: [AnmDat; 1] = [AnmDat {
    index: 0x04,
    start: 0x0E,
    count: 0xFF,
}];
const ANM_TALK4: [AnmDat; 1] = [AnmDat {
    index: 0x05,
    start: 0x06,
    count: 0xFF,
}];
const ANM_TALK5: [AnmDat; 1] = [AnmDat {
    index: 0x06,
    start: 0x0A,
    count: 0xFF,
}];
const ANM_WATASU: [AnmDat; 2] = [
    AnmDat {
        index: 0x07,
        start: 0x0E,
        count: 0x01,
    },
    AnmDat {
        index: 0x08,
        start: 0x0E,
        count: 0xFF,
    },
];
const ANM_ITEMWAIT: [AnmDat; 1] = [AnmDat {
    index: 0x08,
    start: 0x0E,
    count: 0xFF,
}];
const ANM_WIND: [AnmDat; 3] = [
    AnmDat {
        index: 0x09,
        start: 0x0E,
        count: 0x01,
    },
    AnmDat {
        index: 0x00,
        start: 0x14,
        count: 0x01,
    },
    AnmDat {
        index: 0xFF,
        start: 0x08,
        count: 0x00,
    },
];

const ANM_TABLES: [&[AnmDat]; 10] = [
    &ANM_WAIT,
    &ANM_TALK,
    &ANM_WALK,
    &ANM_TALK2,
    &ANM_TALK3,
    &ANM_TALK4,
    &ANM_TALK5,
    &ANM_WATASU,
    &ANM_ITEMWAIT,
    &ANM_WIND,
];

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Merchant {
    pub npc_no: u8,
    pub rail_id: u8,
    pub step: Step,
    pub command: u8,
    pub order: u8,
    pub bits: u16,
    pub talking: bool,
    pub attn_hold: u8,
    pub attn_live: u8,
    pub attn_mode: u8,
    pub item_mode: u8,
    pub head_lock: u8,
    pub wind_hit: u8,
    pub cut_idx: i8,
    pub item_idx: u8,
    pub shown_btn: u8,
    pub reserve_item: u8,
    pub timer: i16,
    pub idle_timer: i16,
    pub tex_timer: i16,
    pub look_speed: i16,
    pub head_angle: i16,
    pub target_y: i16,
    pub current_y: i16,
    pub shape_y: i16,
    pub spawn_y: i16,
    pub anm_idx: u8,
    pub anm_done: u8,
    pub anm_timer: i8,
    pub anm_tbl: u8,
    pub anm_pos: u8,
    pub tex_pattern: u8,
    pub ev_1st: i16,
    pub ev_2nd: i16,
    pub ev_item: i16,
    pub ev_map: i16,
    pub msg_no: u32,
    pub msg_pid: u32,
    pub seq: Option<(Seq, u8)>,
    pub reserve_given: u32,
    pub rupees_paid: i32,
    pub items_given: u32,
    pub talk_xy: bool,
    pub talk_xy_btn: u8,
    pub pre_item: u8,
    pub cut_turn_speed: i16,
    pub speed_f: f32,
    pub anm_speed: f32,
    pub anm_loop: i32,
    pub head_on: bool,
    pub cut_attn: bool,
    pub started: bool,
    pub enabled: bool,
    pub eye: Vec3,
    pub look_pos: Vec3,
    pub hand_pos: Vec3,
    pub home_pos: Vec3,
    pub cut_attn_pos: Vec3,
    pub present_item: Option<Entity>,
    pub seed: [i32; 3],
    pub path: PathRun,
    pub angles: [[i16; 2]; 2],
    pub max_angles: [[i16; 2]; 2],
    pub min_angles: [[i16; 2]; 2],
    pub max_turn_step: [[i16; 2]; 2],
    pub trn: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Wait,
    Talk,
    Walk,
    Turn,
    Wind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Wait,
    Talk,
    Walk,
    Turn,
    Wind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnmDat {
    pub index: u8,
    pub start: u8,
    pub count: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seq {
    FirstTalk,
    Exchange3,
    Exchange,
    Etc,
    XyExchange3,
    XyNoItem,
    XyBadItem,
    KoukanItem,
    KoukanItem2,
    KoukanItem3,
    AfterGet,
    AfterGet2,
    CollectMap,
    TryForce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathRun {
    pub count: usize,
    pub idx: usize,
    pub dir: bool,
    pub closed: bool,
}

impl PathRun {
    pub fn is_path(&self) -> bool {
        self.count != 0
    }

    pub fn set(&mut self, count: usize, forwards: bool) -> bool {
        self.count = count;
        self.idx = 0;
        self.dir = forwards;
        self.count != 0
    }

    pub fn point(&self, path: &[Vec3], at: usize) -> Vec3 {
        if self.is_path() {
            path.get(at).copied().unwrap_or(Vec3::ZERO)
        } else {
            Vec3::ZERO
        }
    }

    pub fn turn_dir(&mut self) {
        self.dir ^= true;
    }

    pub fn inc_idx(&mut self) -> bool {
        if !self.is_path() {
            return true;
        }
        self.idx += 1;
        if self.idx >= self.count {
            self.idx = self.count - 1;
            return false;
        }
        true
    }

    pub fn inc_idx_loop(&mut self) -> bool {
        if !self.is_path() {
            return true;
        }
        self.idx += 1;
        if self.idx >= self.count {
            self.idx = 0;
            return false;
        }
        true
    }

    pub fn dec_idx(&mut self) -> bool {
        if !self.is_path() {
            return true;
        }
        self.idx = self.idx.wrapping_sub(1);
        if self.idx >= self.count {
            self.idx = 0;
            return false;
        }
        true
    }

    pub fn inc_idx_auto(&mut self) -> bool {
        if !self.is_path() {
            return true;
        }
        if self.closed {
            self.idx += 1;
            if self.idx >= self.count {
                self.idx = 0;
            }
        } else {
            self.idx += 1;
            if self.idx >= self.count {
                self.idx = self.count - 1;
                return false;
            }
        }
        true
    }

    pub fn dec_idx_auto(&mut self) -> bool {
        if !self.is_path() {
            return true;
        }
        if self.closed {
            self.idx = self.idx.wrapping_sub(1);
            if self.idx >= self.count {
                self.idx = self.count - 1;
            }
        } else {
            self.idx = self.idx.wrapping_sub(1);
            if self.idx >= self.count {
                self.idx = 0;
                return false;
            }
        }
        true
    }

    pub fn next_idx_auto(&mut self) -> bool {
        if self.dir {
            let hit = self.inc_idx_auto();
            if !hit {
                self.dec_idx();
            }
            hit
        } else {
            let hit = self.dec_idx_auto();
            if !hit {
                self.inc_idx();
            }
            hit
        }
    }

    pub fn chk_point_pass(&self, path: &[Vec3], curr: Vec3) -> bool {
        if !self.is_path() {
            return false;
        }
        let target = self.point(path, self.idx);
        let here = self.point(path, self.idx);
        let next = self.point(path, self.idx + 1);
        let prev = path.get(self.idx.wrapping_sub(1)).copied().unwrap_or(here);
        let (delta_x, delta_z) = if self.idx == 0 {
            (next.x - here.x, next.z - here.z)
        } else if self.idx + 1 >= self.count {
            (here.x - prev.x, here.z - prev.z)
        } else {
            (next.x - prev.x, next.z - prev.z)
        };
        let heading = walk::to_s16(delta_x.atan2(delta_z));
        let f29 = walk::from_s16(heading).sin() * PATH_TRIG_SCALE;
        let f2 = walk::from_s16(heading).cos() * PATH_TRIG_SCALE;
        let f3 = -(f29 * target.x + f2 * target.z);
        let side = f29 * curr.x + f2 * curr.z + f3;
        if self.dir {
            side > 0.0
        } else {
            side <= 0.0
        }
    }
}

pub struct Shop<'a> {
    pub procs: &'a mut MsgProcs,
    pub info: &'a mut MsgGameInfo,
    pub archive: Option<&'a MsgArchives>,
    pub stories: &'a mut StoryBits,
    pub rupees: &'a mut Rupees,
    pub ev_flags: &'a mut EventFlags,
    pub status: u8,
    pub select_num: u8,
    pub present_end: bool,
}

impl Merchant {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            npc_no: ((parameters >> PRM_NPC_NO_SHIFT) & PRM_NPC_NO_MASK) as u8,
            rail_id: ((parameters >> PRM_RAIL_ID_SHIFT) & PRM_RAIL_ID_MASK) as u8,
            step: Step::Wait,
            command: CMD_NONE,
            order: 0,
            bits: 0,
            talking: false,
            attn_hold: 0,
            attn_live: 0,
            attn_mode: ATTN_NONE,
            item_mode: 0,
            head_lock: 0,
            wind_hit: 0,
            cut_idx: -1,
            item_idx: 0,
            shown_btn: BTN_NONE,
            reserve_item: RESERVE_NONE,
            timer: 0,
            idle_timer: 0,
            tex_timer: 0,
            look_speed: 0,
            head_angle: 0,
            target_y: 0,
            current_y: 0,
            shape_y: 0,
            spawn_y: 0,
            anm_idx: 0,
            anm_done: 0,
            anm_timer: 0,
            anm_tbl: ANM_NONE,
            anm_pos: 0,
            tex_pattern: 0,
            ev_1st: NO_EVENT,
            ev_2nd: NO_EVENT,
            ev_item: NO_EVENT,
            ev_map: NO_EVENT,
            msg_no: NO_MSG,
            msg_pid: FPC_M_ERROR_PROCESS_ID,
            seq: None,
            reserve_given: 0,
            rupees_paid: 0,
            items_given: 0,
            talk_xy: false,
            talk_xy_btn: 0,
            pre_item: 0,
            cut_turn_speed: 0,
            speed_f: 0.0,
            anm_speed: ANM_RESTART,
            anm_loop: 0,
            head_on: false,
            cut_attn: false,
            started: false,
            enabled: true,
            eye: Vec3::ZERO,
            look_pos: Vec3::ZERO,
            hand_pos: Vec3::ZERO,
            home_pos: Vec3::ZERO,
            cut_attn_pos: Vec3::ZERO,
            present_item: None,
            seed: [RND_SEED; 3],
            path: PathRun {
                count: 0,
                idx: 0,
                dir: false,
                closed: false,
            },
            angles: [[0; 2]; 2],
            max_angles: [[0; 2]; 2],
            min_angles: [[0; 2]; 2],
            max_turn_step: [[0; 2]; 2],
            trn: false,
        }
    }

    pub fn get_prm_npc_no(&self) -> u8 {
        self.npc_no
    }

    pub fn get_prm_rail_id(&self) -> u8 {
        self.rail_id
    }

    pub fn is_haitatu_item(item: u8) -> bool {
        item >= ITEM_TOWN_FLOWER && item <= ITEM_XXX_039
    }

    pub fn is_koukan_item(item: u8) -> bool {
        item >= ITEM_TOWN_FLOWER && item <= ITEM_SHOP_GURU_STATUE
    }

    pub fn is_get_map(&self, item: u8, stories: &StoryBits) -> bool {
        self.npc_no == 1 && !stories.test(BIT_MAP_TAKEN) && item == ITEM_SHOP_GURU_STATUE
    }

    pub fn item_index(&self, idx: usize) -> u8 {
        ITEM_DAT
            .get(usize::from(self.npc_no))
            .and_then(|row| row.get(idx))
            .copied()
            .unwrap_or(0)
    }

    pub fn item_of(&self) -> u8 {
        self.item_index(usize::from(self.item_idx))
    }

    pub fn item_no(&self) -> u8 {
        self.item_of().wrapping_add(ITEM_TOWN_FLOWER)
    }

    pub fn reserve_bit(&self, idx: u8) -> bool {
        if u32::from(idx) < RESERVE_BITS {
            self.reserve_given & (1u32 << idx) != 0
        } else {
            false
        }
    }

    pub fn on_reserve_bit(&mut self, idx: u8) {
        if u32::from(idx) < RESERVE_BITS {
            self.reserve_given |= 1u32 << idx;
        }
    }

    pub fn save_talked(&self) -> u16 {
        SAVE_TALKED
            .get(usize::from(self.npc_no))
            .copied()
            .unwrap_or(0)
    }

    pub fn save_exchanged(&self) -> u16 {
        SAVE_EXCHANGED
            .get(usize::from(self.npc_no))
            .copied()
            .unwrap_or(0)
    }

    pub fn save_offered(&self) -> u16 {
        SAVE_OFFERED
            .get(usize::from(self.npc_no))
            .copied()
            .unwrap_or(0)
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn set_message(&mut self, msg_no: u32) {
        self.msg_pid = FPC_M_ERROR_PROCESS_ID;
        self.msg_no = msg_no;
    }

    fn seq_value(&self, at: u8) -> u32 {
        match self.seq {
            Some((seq, pos)) => {
                seq_at(seq, self.npc_no, usize::from(self.item_idx), usize::from(at))
                    .unwrap_or(NO_MSG)
            }
            None => NO_MSG,
        }
    }

    fn message_set(&mut self, shop: &mut Shop<'_>) -> u32 {
        let Some(archive) = shop.archive else {
            return FPC_M_ERROR_PROCESS_ID;
        };
        let eye = self.eye;
        let msg_no = self.msg_no;
        fop_msgm_message_set(shop.procs, shop.info, archive, MSG_CLEAR_COUNT, msg_no, None, eye)
    }

    fn message_set_follow(&mut self, shop: &mut Shop<'_>) -> u32 {
        let Some(archive) = shop.archive else {
            return FPC_M_ERROR_PROCESS_ID;
        };
        let msg_no = self.msg_no;
        fop_msgm_message_set_look_at(
            shop.procs,
            shop.info,
            archive,
            MSG_CLEAR_COUNT,
            msg_no,
            Vec3::ZERO,
        )
    }

    pub fn talk(&mut self, param: i32, shop: &mut Shop<'_>) -> u8 {
        let mut result = TALK_IDLE;
        if self.msg_pid == FPC_M_ERROR_PROCESS_ID {
            if param == 1 {
                self.get_msg(shop);
            }
            self.msg_pid = self.message_set(shop);
        } else if shop.procs.is_executing(self.msg_pid) {
            result = shop.status;
            if shop.status == STATUS_MSG_DISPLAYED {
                let next = self.next_msg_status(shop);
                shop.info.mesg_status = next;
                if next == STATUS_MSG_CONTINUES {
                    self.message_set_follow(shop);
                }
            } else if shop.status == STATUS_BOX_CLOSED {
                shop.info.mesg_status = STATUS_MSG_DESTROYED;
                self.msg_pid = FPC_M_ERROR_PROCESS_ID;
            }
        }
        result
    }

    pub fn next_msg_status(&mut self, shop: &mut Shop<'_>) -> u8 {
        let mut status = STATUS_MSG_CONTINUES;
        if MSG_KOUKAN_END.contains(&self.msg_no) {
            return STATUS_MSG_ENDS;
        }
        let Some((seq, pos)) = self.seq else {
            return STATUS_MSG_ENDS;
        };
        let next = pos.wrapping_add(1);
        let Some(step) = seq_at(seq, self.npc_no, usize::from(self.item_idx), usize::from(next))
        else {
            self.seq = None;
            return STATUS_MSG_ENDS;
        };
        match step {
            SEQ_NONE => {
                self.seq = None;
                status = STATUS_MSG_ENDS;
            }
            SEQ_SELECT => {
                if shop.select_num != 0 {
                    self.bits |= BIT_SELECT_YES;
                }
                self.seq = None;
                status = STATUS_MSG_ENDS;
            }
            SEQ_BUY => {
                if shop.select_num != 0 {
                    self.bits |= BIT_SELECT_NO;
                    self.seq = None;
                    return STATUS_MSG_ENDS;
                }
                let price = shop.info.message_rupee;
                if i32::try_from(shop.rupees.count).unwrap_or(i32::MAX) < i32::from(price) {
                    self.bits |= BIT_NO_RUPEE;
                    self.seq = None;
                    return STATUS_MSG_ENDS;
                }
                let paid = u32::try_from(i32::from(price)).unwrap_or(0);
                shop.rupees.count = shop.rupees.count.saturating_sub(paid);
                self.rupees_paid = self.rupees_paid.wrapping_add(i32::from(price));
                let temp = self.item_of();
                let reg = self.save_exchanged();
                let count = shop.stories.reg(reg).wrapping_add(1);
                shop.stories.set_reg(reg, count);
                if self.reserve_bit(temp) {
                    self.msg_no = MSG_KOUKAN_END
                        .get(usize::from(self.npc_no))
                        .copied()
                        .unwrap_or(NO_MSG);
                    self.bits |= BIT_BAG_FULL;
                } else {
                    self.msg_no = MSG_KOUKAN_FIRST
                        .get(usize::from(self.npc_no))
                        .copied()
                        .unwrap_or(NO_MSG);
                }
                self.reserve_item = temp.wrapping_add(ITEM_TOWN_FLOWER);
                self.seq = None;
            }
            SEQ_END => {
                status = STATUS_MSG_ENDS;
            }
            other => {
                self.msg_no = other;
            }
        }
        status
    }

    pub fn get_msg(&mut self, shop: &mut Shop<'_>) -> u32 {
        let mut msg_no = NO_MSG;
        self.seq = None;
        if self.bits & BIT_SELECT_YES != 0 {
            self.bits &= !BIT_SELECT_YES;
            msg_no = MSG_KOUKAN_NO
                .get(usize::from(self.npc_no))
                .copied()
                .unwrap_or(NO_MSG);
        } else if self.bits & BIT_SELECT_NO != 0 {
            self.bits &= !BIT_SELECT_NO;
            msg_no = MSG_KOUKAN_NO
                .get(usize::from(self.npc_no))
                .copied()
                .unwrap_or(NO_MSG);
        } else if self.bits & BIT_NO_RUPEE != 0 {
            self.bits &= !BIT_NO_RUPEE;
            msg_no = MSG_KOUKAN_RUPEE
                .get(usize::from(self.npc_no))
                .copied()
                .unwrap_or(NO_MSG);
        } else if self.talk_xy {
            let item = self.pre_item;
            if self.is_get_map(item, shop.stories) {
                self.seq = Some((Seq::TryForce, 0));
                self.reserve_item = RESERVE_NONE;
                shop.stories.set(BIT_MAP_TAKEN);
            } else if shop.stories.reg(self.save_exchanged()) >= EXCHANGE_LIMIT {
                self.seq = Some((Seq::XyExchange3, 0));
            } else if !Self::is_haitatu_item(item) {
                self.seq = Some((Seq::XyNoItem, 0));
            } else if !Self::is_koukan_item(item) {
                self.seq = Some((Seq::XyBadItem, 0));
            } else {
                self.item_idx = item.wrapping_sub(ITEM_TOWN_FLOWER);
                self.seq = Some((Seq::KoukanItem, 0));
                shop.stories.set(self.save_offered());
            }
        } else if !shop.stories.test(self.save_talked()) {
            self.seq = Some((Seq::FirstTalk, 0));
            shop.stories.set(self.save_talked());
        } else if shop.stories.reg(self.save_exchanged()) >= EXCHANGE_LIMIT {
            self.seq = Some((Seq::Exchange3, 0));
        } else if self.npc_no == 1 && shop.stories.test(BIT_MAP_TAKEN) {
            self.seq = Some((Seq::CollectMap, 0));
        } else if shop.stories.test(self.save_offered()) {
            self.seq = Some((Seq::Exchange, 0));
        } else {
            self.seq = Some((Seq::Etc, 0));
        }
        if let Some((seq, pos)) = self.seq {
            msg_no = seq_at(seq, self.npc_no, usize::from(self.item_idx), usize::from(pos))
                .unwrap_or(NO_MSG);
        }
        msg_no
    }

    pub fn event_mes_set_init(&mut self, msg_data: Option<u32>, shop: &mut Shop<'_>) {
        match msg_data {
            Some(0x0) => {
                self.seq = Some((Seq::KoukanItem, 0));
                let msg = self.seq_value(0);
                self.set_message(msg);
                shop.stories.set(self.save_offered());
            }
            Some(0x1) => {
                self.seq = Some((Seq::KoukanItem2, 0));
                let msg = self.seq_value(0);
                self.set_message(msg);
            }
            Some(0xA) => {
                self.seq = Some((Seq::KoukanItem3, 0));
                let msg = self.seq_value(0);
                self.set_message(msg);
            }
            Some(0x14) => {
                self.seq = Some((Seq::AfterGet, 0));
                let msg = self.seq_value(0);
                self.set_message(msg);
            }
            Some(0x15) => {
                self.seq = Some((Seq::AfterGet2, 0));
                let msg = self.seq_value(0);
                self.set_message(msg);
            }
            Some(0x63) => {
                let msg = self.get_msg(shop);
                self.set_message(msg);
            }
            Some(other) => self.set_message(other),
            None => {
                if let Some((seq, pos)) = self.seq {
                    let next = pos.wrapping_add(1);
                    self.seq = Some((seq, next));
                    let msg = self.seq_value(next);
                    self.set_message(msg);
                }
            }
        }
    }

    pub fn event_mes_set(&mut self, shop: &mut Shop<'_>) -> bool {
        self.talk(0, shop) == STATUS_BOX_CLOSED
    }

    pub fn event_set_item_init(&mut self, actor: &Actor, commands: &mut Commands) {
        let item = self.item_no();
        let entity = commands
            .spawn((
                Actor {
                    name: DEMO_ITEM_ACTOR.to_owned(),
                    parameters: 0,
                    pos: Vec3::ZERO,
                    angle: [0; 3],
                    set_id: u16::MAX,
                    room: -1,
                },
                PresentItem {
                    item,
                    hidden: false,
                },
            ))
            .id();
        self.present_item = Some(entity);
        self.hand_pos = self.eye;
        let _ = actor;
    }

    pub fn event_clr_item_init(&mut self) {
        self.item_mode = 1;
        self.head_lock = 0;
        self.set_anm_tbl(TBL_WAIT, 0);
    }

    pub fn event_get_item_init(
        &mut self,
        item_data: Option<u32>,
        commands: &mut Commands,
        partner: &mut ItemPartner,
    ) -> Option<u8> {
        let idx = self.item_of();
        let item = match item_data {
            Some(raw) => GET_ITEM_NO
                .get(usize::try_from(raw).unwrap_or(0))
                .copied()
                .unwrap_or(shop::NO_ITEM),
            None => {
                self.on_reserve_bit(idx);
                idx.wrapping_add(ITEM_TOWN_FLOWER)
            }
        };
        if item == RESERVE_NONE {
            return None;
        }
        self.items_given = self.items_given.wrapping_add(1);
        let entity = commands
            .spawn((
                Actor {
                    name: DEMO_ITEM_ACTOR.to_owned(),
                    parameters: 0,
                    pos: self.eye,
                    angle: [0; 3],
                    set_id: u16::MAX,
                    room: -1,
                },
                PresentItem {
                    item,
                    hidden: false,
                },
            ))
            .id();
        self.present_item = Some(entity);
        partner.entity = Some(entity);
        partner.item = item;
        Some(item)
    }

    pub fn event_init(&mut self) {
        self.cut_idx = 0;
    }

    pub fn event_set_angle_init(&self, pos: Vec3, player: Vec3) -> i16 {
        let delta = pos - player;
        stream::angle_s(delta.x.atan2(delta.z))
    }

    pub fn event_on_plr_init(&mut self) {
        self.cut_turn_speed = 0;
    }

    pub fn event_off_plr_init(&mut self) {
        self.cut_turn_speed = 0;
    }

    pub fn xy_event_cb(
        &mut self,
        select_item: u8,
        talk_btn: u8,
        list: &EventList,
        stories: &StoryBits,
    ) -> i16 {
        self.pre_item = select_item;
        self.talk_xy_btn = talk_btn;
        self.talk_xy = true;
        self.item_idx = select_item.wrapping_sub(ITEM_TOWN_FLOWER);
        let reg = self.save_exchanged();
        if Self::is_koukan_item(select_item)
            && stories.reg(reg) < EXCHANGE_LIMIT
            && !self.is_get_map(select_item, stories)
        {
            self.shown_btn = match talk_btn {
                TALK_BTN_X => BTN_X,
                TALK_BTN_Y => BTN_Y,
                TALK_BTN_Z => BTN_Z,
                _ => BTN_NONE,
            };
            self.bits |= BIT_EVENT_MASK;
            self.ev_1st
        } else if self.is_get_map(select_item, stories) {
            self.bits |= BIT_EVENT_MASK;
            self.ev_map
        } else {
            event_index(list, EV_DEFAULT_TALK_XY)
        }
    }

    pub fn set_anm_from_msg_tag(&mut self, info: &mut MsgGameInfo) {
        let table = match info.mesg_anime {
            0 => Some(TBL_WAIT),
            1 => Some(TBL_TALK),
            2 => Some(TBL_TALK2),
            3 => Some(TBL_TALK3),
            4 => Some(TBL_WALK),
            5 => Some(TBL_WATASU),
            6 => Some(TBL_ITEMWAIT),
            7 => Some(TBL_TALK4),
            8 => Some(TBL_TALK5),
            _ => None,
        };
        if let Some(table) = table {
            self.set_anm_tbl(table, 0);
        }
        info.mesg_anime = MESG_ANIME_CLEAR;
    }

    pub fn set_anm(&mut self, index: u8, loop_mode: i32, start: f32) {
        let mut start = start;
        if self.anm_speed >= 0.0 {
            start = self.anm_speed;
            self.anm_speed = ANM_RESTART;
        }
        let _ = start;
        self.anm_idx = index;
        self.anm_loop = loop_mode;
    }

    pub fn set_anm_tbl(&mut self, table: u8, pos: u8) -> bool {
        self.anm_done &= !1;
        let Some(entry) = anm_at(table, pos) else {
            self.anm_tbl = ANM_NONE;
            return true;
        };
        if entry.index == ANM_END {
            self.anm_tbl = ANM_NONE;
            return true;
        }
        self.anm_tbl = table;
        self.anm_pos = pos;
        self.anm_timer = entry.count as i8;
        let mut mode = ANM_LOOP;
        if self.anm_timer > 0 {
            mode = ANM_ONCE;
        }
        if self.anm_idx != entry.index || mode == 0 || self.anm_loop == 0 {
            self.set_anm(entry.index, mode, f32::from(entry.start));
        }
        false
    }

    pub fn play_anm(&mut self, played: bool) {
        if played && self.anm_tbl != ANM_NONE && self.anm_timer > 0 {
            self.anm_timer -= 1;
            if self.anm_timer == 0 {
                let next = self.anm_pos.wrapping_add(1);
                if self.set_anm_tbl(self.anm_tbl, next) {
                    self.anm_done |= 1;
                }
            } else if let Some(entry) = anm_at(self.anm_tbl, self.anm_pos) {
                self.set_anm(entry.index, ANM_ONCE, 0.0);
            }
        }
    }

    pub fn play_tex_pattern_anm(&mut self) {
        if calc_timer(&mut self.tex_timer) == 0 {
            if self.tex_pattern >= TEX_PATTERN_MAX {
                self.tex_pattern = self.tex_pattern.wrapping_sub(TEX_PATTERN_MAX);
                self.tex_timer = TEX_PATTERN_STEP;
            } else {
                self.tex_pattern = self.tex_pattern.wrapping_add(1);
            }
        }
    }

    pub fn execute_common(&mut self, shop: &mut Shop<'_>, pos: Vec3, path: &[Vec3]) -> bool {
        if self.attn_live != 0 && self.step != Step::Wind {
            self.order = 1;
        } else {
            self.order = 0;
        }
        if self.talking {
            self.execute_set_mode(Mode::Talk, shop, pos, path);
        }
        self.talking
    }

    pub fn execute_set_mode(
        &mut self,
        mode: Mode,
        shop: &mut Shop<'_>,
        pos: Vec3,
        path: &[Vec3],
    ) {
        self.speed_f = 0.0;
        self.step = match mode {
            Mode::Wait => {
                self.execute_wait_init();
                Step::Wait
            }
            Mode::Talk => {
                self.execute_talk_init();
                Step::Talk
            }
            Mode::Walk => {
                self.execute_walk_init();
                Step::Walk
            }
            Mode::Turn => self.execute_turn_init(shop, pos, path),
            Mode::Wind => {
                self.execute_wind_init();
                Step::Wind
            }
        };
    }

    pub fn execute_wait_init(&mut self) {
        self.set_anm_tbl(TBL_WAIT, 0);
        self.timer = rnd_range(&mut self.seed, WAIT_MIN, WAIT_RANGE);
    }

    pub fn execute_wait(&mut self, shop: &mut Shop<'_>, pos: Vec3, path: &[Vec3]) {
        if !self.execute_common(shop, pos, path)
            && self.path.is_path()
            && self.timer != 0
            && self.attn_hold == 0
        {
            self.timer -= 1;
            if self.timer == 0 {
                self.execute_set_mode(Mode::Turn, shop, pos, path);
            }
        }
    }

    pub fn execute_talk_init(&mut self) {}

    pub fn execute_talk(&mut self, shop: &mut Shop<'_>, pos: Vec3, path: &[Vec3]) {
        self.execute_common(shop, pos, path);
        if !self.talk_xy || shop.present_end {
            if self.talk(1, shop) == STATUS_BOX_CLOSED {
                self.talking = false;
                self.execute_set_mode(Mode::Wait, shop, pos, path);
                shop.ev_flags.set(EVT_RESET_FLAG);
                self.bits &= !BIT_EVENT_RUN;
                self.item_mode = 0;
            } else {
                self.set_anm_from_msg_tag(shop.info);
            }
        }
    }

    pub fn execute_walk_init(&mut self) {
        self.set_anm_tbl(TBL_WALK, 0);
        self.timer = rnd_range(&mut self.seed, WALK_MIN, WALK_RANGE);
    }

    pub fn execute_walk(&mut self, shop: &mut Shop<'_>, pos: Vec3, path: &[Vec3]) {
        if self.execute_common(shop, pos, path) {
            return;
        }
        let passed = self.path.chk_point_pass(path, pos) && !self.path.next_idx_auto();
        if self.attn_hold != 0 {
            self.execute_set_mode(Mode::Wait, shop, pos, path);
        } else if !passed {
            let point = self.path.point(path, self.path.idx);
            let (_, angle) = dis_xz_ang_y(pos, point);
            self.target_y = angle;
            self.head_on = false;
            self.look_speed = TURN_SPEED_WALK;
            self.attn_mode = ATTN_HOME;
            self.set_trn();
            self.speed_f = WALK_SPEED;
            if self.timer != 0 {
                self.timer -= 1;
                if self.timer == 0 {
                    self.execute_set_mode(Mode::Wait, shop, pos, path);
                }
            }
        } else {
            self.path.turn_dir();
            self.execute_set_mode(Mode::Wait, shop, pos, path);
        }
    }

    pub fn execute_turn_init(&mut self, _shop: &mut Shop<'_>, pos: Vec3, path: &[Vec3]) -> Step {
        let point = self.path.point(path, self.path.idx);
        let (_, angle) = dis_xz_ang_y(pos, point);
        if angle == self.current_y {
            self.set_anm_tbl(TBL_WALK, 0);
            self.timer = rnd_range(&mut self.seed, WALK_MIN, WALK_RANGE);
            return Step::Walk;
        }
        Step::Turn
    }

    pub fn execute_turn(&mut self, shop: &mut Shop<'_>, pos: Vec3, path: &[Vec3]) {
        if self.execute_common(shop, pos, path) {
            return;
        }
        let point = self.path.point(path, self.path.idx);
        let (_, angle) = dis_xz_ang_y(pos, point);
        self.target_y = angle;
        self.head_on = false;
        self.attn_mode = ATTN_HOME;
        if !self.trn_chk() {
            self.execute_set_mode(Mode::Walk, shop, pos, path);
        }
    }

    pub fn execute_wind_init(&mut self) {
        self.set_anm_tbl(TBL_WIND, 0);
    }

    pub fn execute_wind(&mut self, shop: &mut Shop<'_>, pos: Vec3, path: &[Vec3]) {
        if !self.execute_common(shop, pos, path) && self.anm_done & 1 != 0 {
            self.execute_set_mode(Mode::Wait, shop, pos, path);
            self.wind_hit = 0;
        }
    }

    pub fn check_order(&mut self, active: &ActiveEvent, near: bool, pressed: bool) {
        if self.command == CMD_DEMO {
            if self.started_check(active, self.ev_1st) && self.order == ORDER_1ST {
                self.order = 0;
            } else if self.started_check(active, self.ev_2nd) && self.order == ORDER_2ND {
                self.order = 0;
            } else if self.started_check(active, self.ev_item) && self.order == ORDER_ITEM {
                self.order = 0;
            } else if self.started_check(active, self.ev_map) && self.order == ORDER_MAP {
                self.order = 0;
            }
        } else if near && pressed && (self.order == ORDER_TALK_ITEM || self.order == 1) && !self.talking
        {
            self.talking = true;
            self.speed_f = 0.0;
            self.item_mode = 1;
        }
    }

    fn started_check(&self, active: &ActiveEvent, index: i16) -> bool {
        i16::try_from(index).is_ok_and(|value| active.start_check(usize::from(value)))
    }

    pub fn event_order(
        &mut self,
        entity: Entity,
        list: &EventList,
        active: &mut ActiveEvent,
        stories: &StoryBits,
        order_flags: &mut OrderFlags,
    ) {
        match self.order {
            1 | ORDER_TALK_ITEM => {
                order_flags.set(EVT_CND_CANTALK);
                if stories.test(self.save_talked()) {
                    order_flags.set(EVT_CND_CANTALKITEM);
                }
            }
            ORDER_1ST => {
                active.order(list, usize::from(self.ev_1st.max(0) as u16), entity);
                self.bits |= BIT_EVENT_RUN;
            }
            ORDER_2ND => {
                active.order(list, usize::from(self.ev_2nd.max(0) as u16), entity);
                self.bits |= BIT_EVENT_RUN;
            }
            ORDER_ITEM => {
                active.order(list, usize::from(self.ev_item.max(0) as u16), entity);
                self.bits |= BIT_EVENT_RUN;
            }
            ORDER_MAP => {
                active.order(list, usize::from(self.ev_map.max(0) as u16), entity);
                self.bits |= BIT_EVENT_RUN;
                self.reserve_item = RESERVE_NONE;
            }
            _ => {}
        }
    }

    pub fn event_move(
        &mut self,
        shop: &mut Shop<'_>,
        active: &ActiveEvent,
        pos: Vec3,
        path: &[Vec3],
    ) {
        if self.bits & BIT_EVENT_RUN == 0 {
            return;
        }
        if self.ended_check(active, self.ev_1st) {
            shop.ev_flags.set(EVT_RESET_FLAG);
            self.bits &= !BIT_EVENT_MASK;
            self.execute_set_mode(Mode::Wait, shop, pos, path);
            self.talking = false;
            if self.bits & BIT_SELECT_YES != 0 {
                self.order = ORDER_TALK_ITEM;
                self.event_on_plr_init();
            } else {
                self.order = ORDER_2ND;
            }
        } else if self.ended_check(active, self.ev_2nd) {
            shop.ev_flags.set(EVT_RESET_FLAG);
            self.bits &= !BIT_EVENT_RUN;
            self.execute_set_mode(Mode::Wait, shop, pos, path);
            if self.bits & BIT_BAG_FULL != 0 {
                self.bits &= !BIT_BAG_FULL;
                self.event_on_plr_init();
            } else if self.bits & (BIT_NO_RUPEE | BIT_SELECT_NO) != 0 {
                self.order = ORDER_TALK_ITEM;
                self.event_on_plr_init();
            } else {
                self.order = ORDER_ITEM;
            }
        } else if self.ended_check(active, self.ev_item) {
            self.item_mode = 0;
            shop.ev_flags.set(EVT_RESET_FLAG);
            self.bits &= !BIT_EVENT_RUN;
            self.execute_set_mode(Mode::Wait, shop, pos, path);
        } else if self.ended_check(active, self.ev_map) {
            self.talking = false;
            self.item_mode = 0;
            shop.ev_flags.set(EVT_RESET_FLAG);
            self.bits &= !BIT_EVENT_MASK;
            self.execute_set_mode(Mode::Wait, shop, pos, path);
        } else if self.cut_idx < 0 {
            self.set_anm_from_msg_tag(shop.info);
        }
    }

    fn ended_check(&self, active: &ActiveEvent, index: i16) -> bool {
        i16::try_from(index).is_ok_and(|value| active.end_check(usize::from(value)))
    }

    pub fn chk_attention(&mut self, pos: Vec3, player: Vec3) {
        if self.cut_attn {
            self.look_pos = self.cut_attn_pos;
            self.attn_mode = ATTN_PLAYER;
            if self.item_mode != 0 {
                self.head_on = false;
                self.set_trn();
            } else {
                self.head_on = true;
            }
            if self.attn_hold == 0 {
                self.attn_hold = 1;
                self.attn_live = 1;
            }
        } else {
            let mut reach = LOOK_DIST;
            let mut window = LOOK_ANGLE;
            let mut lock = LOOK_RANGE_HALF;
            if self.talking {
                lock = LOOK_RANGE_TALK;
            }
            let (dist, to_player) = dis_xz_ang_y(pos, player);
            if self.attn_hold != 0 {
                reach += LOOK_RANGE_BONUS;
                window += LOOK_ANGLE_HELD;
            }
            let delta = to_player.wrapping_sub(self.current_y);
            let diff = i32::from(delta).abs();
            if reach > dist && self.wind_hit == 0 && lock > diff {
                self.look_pos = player_eye_pos(player, LOOK_EYE_Y);
                self.attn_mode = ATTN_PLAYER;
                if self.item_mode != 0 {
                    self.head_on = false;
                } else {
                    self.head_on = true;
                }
                if self.attn_hold == 0 {
                    self.attn_hold = 1;
                }
                self.attn_live = 1;
            } else if reach > dist && self.wind_hit == 0 && window > diff {
                if self.attn_live == 0 {
                    self.attn_live = 1;
                }
                if self.attn_hold == 1 {
                    self.attn_hold = 0;
                    self.idle_timer = IDLE_FRAMES;
                }
            } else {
                if self.attn_hold == 1 {
                    self.attn_hold = 0;
                    self.idle_timer = IDLE_FRAMES;
                }
                self.attn_live = 0;
                self.attn_mode = ATTN_NONE;
                if !self.path.is_path() {
                    if self.idle_timer != 0 {
                        self.idle_timer -= 1;
                    } else {
                        self.target_y = self.home_y();
                        self.head_on = false;
                        self.attn_mode = ATTN_HOME;
                        self.set_trn();
                    }
                }
            }
        }
        self.look_speed = LOOK_SPEED;
    }

    pub fn home_y(&self) -> i16 {
        self.spawn_y
    }

    pub fn look_back(&mut self) {
        let mut temp1 = self.look_speed;
        let mut target_y = self.current_y;
        let mut dst: Option<Vec3> = None;
        let mut follow = self.head_on;
        match self.attn_mode {
            ATTN_NONE => {}
            ATTN_PLAYER => dst = Some(self.look_pos),
            ATTN_HOME => target_y = self.target_y,
            _ => {}
        }
        if self.talking && self.item_mode != 0 {
            follow = false;
            self.set_trn();
        }
        if self.trn_chk() {
            if self.cut_turn_speed != 0 {
                temp1 = self.cut_turn_speed;
            }
            stream::add_calc_angle_s2(&mut self.head_angle, temp1, LOOK_SCALE, LOOK_MAX_STEP);
        } else {
            self.head_angle = 0;
        }
        let mut yaw = self.current_y;
        self.look_at_target(&mut yaw, dst, self.eye, target_y, self.head_angle, follow);
        self.current_y = yaw;
        self.shape_y = self.current_y;
    }

    fn look_at_target(
        &mut self,
        out_y: &mut i16,
        dst: Option<Vec3>,
        src: Vec3,
        default_y: i16,
        max_vel: i16,
        free: bool,
    ) {
        let (target_y, pitch) = match dst {
            Some(point) => {
                let delta = point - src;
                let flat = (delta.x * delta.x + delta.z * delta.z).sqrt();
                (
                    walk::to_s16(delta.x.atan2(delta.z)),
                    walk::to_s16(delta.y.atan2(flat)),
                )
            }
            None => (default_y, 0),
        };
        let delta_y = i32::from(target_y) - i32::from(*out_y);
        let max_y = i32::from(self.max_angles[1][1]) + i32::from(self.max_angles[0][1]);
        let min_y = i32::from(self.min_angles[1][1]) + i32::from(self.min_angles[0][1]);
        let mut moved = delta_y;
        let settled = self.jnt_move(&mut moved, 1);
        if (self.trn || (delta_y >= max_y || delta_y <= min_y) && settled) && !free {
            self.trn = self.follow(out_y, target_y, max_vel, 1);
        }
        let mut tilt = i32::from(pitch);
        self.jnt_move(&mut tilt, 0);
    }

    fn jnt_move(&mut self, value: &mut i32, axis: usize) -> bool {
        let mut rest = *value as i16;
        let mut done = 0;
        for row in 0..2 {
            let mut angle = rest;
            limitter(
                &mut angle,
                self.max_angles[row][axis],
                self.min_angles[row][axis],
            );
            rest = rest.wrapping_sub(angle);
            let current = i32::from(self.angles[row][axis]);
            let max_step = self.max_turn_step[row][axis];
            let next = add_calc_angle_l(
                current,
                i32::from(angle),
                HEAD_MOVE_SCALE,
                i32::from(max_step),
                HEAD_MOVE_MIN,
            );
            self.angles[row][axis] = next as i16;
            if distance_angle_s(angle, self.angles[row][axis]) <= i32::from(max_step / 2) {
                done += 1;
            }
        }
        done == 2
    }

    fn follow(&mut self, out_y: &mut i16, target_y: i16, max_vel: i16, axis: usize) -> bool {
        let orig = *out_y;
        self.ang_calc(out_y, target_y, 4, max_vel);
        let delta = out_y.wrapping_sub(orig);
        let mut acc = 0;
        let mut prev = 0;
        let want = target_y.wrapping_sub(*out_y);
        for row in (0..2).rev() {
            acc += self.angles[row][axis];
            if (delta > 0 && acc > want) || (delta < 0 && acc < want) {
                self.angles[row][axis] = want - prev;
                limitter(
                    &mut self.angles[row][axis],
                    self.max_angles[row][axis],
                    self.min_angles[row][axis],
                );
            }
            prev += self.angles[row][axis];
        }
        target_y != *out_y
    }

    fn ang_calc(&mut self, out: &mut i16, target: i16, speed: i16, max_vel: i16) -> bool {
        let orig = out.wrapping_sub(target);
        let diff = walk::add_calc_angle_s(orig, 0, speed, max_vel, HEAD_TURN_MIN_STEP);
        if i32::from(diff).abs() > i32::from(speed) {
            *out = out.wrapping_add(diff.wrapping_sub(orig));
        } else {
            *out = target;
        }
        target.wrapping_sub(*out) == 0
    }

    pub fn set_trn(&mut self) {
        self.trn = true;
    }

    pub fn trn_chk(&self) -> bool {
        self.trn
    }

    pub fn set_jnt_param(&mut self) {
        let backbone = if self.head_lock != 0 {
            [HEAD_LOCK_VALUE, HEAD_LOCK_VALUE, HEAD_LOCK_VALUE, HEAD_LOCK_VALUE]
        } else {
            [
                HEAD_LIMIT_Y,
                HEAD_LIMIT_Y,
                HEAD_MIN_Y,
                HEAD_MIN_Y,
            ]
        };
        self.set_param(
            backbone[0],
            backbone[1],
            backbone[2],
            backbone[3],
            HEAD_LIMIT_X,
            HEAD_LIMIT_Y,
            HEAD_MIN_X,
            HEAD_MIN_Y,
            HEAD_TURN_STEP,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn set_param(
        &mut self,
        max_backbone_x: i16,
        max_backbone_y: i16,
        min_backbone_x: i16,
        min_backbone_y: i16,
        max_head_x: i16,
        max_head_y: i16,
        min_head_x: i16,
        min_head_y: i16,
        max_turn_step: i16,
    ) {
        self.max_angles[1] = [max_backbone_x, max_backbone_y];
        self.min_angles[1] = [min_backbone_x, min_backbone_y];
        self.max_angles[0] = [max_head_x, max_head_y];
        self.min_angles[0] = [min_head_x, min_head_y];
        for row in 0..2 {
            self.max_turn_step[row] = [max_turn_step, max_turn_step];
        }
    }

    pub fn bag_centre(&self, pos: Vec3, bag: Vec3) -> Vec3 {
        let (_, angle) = dis_xz_ang_y(pos, bag);
        Vec3::new(
            bag.x + BAG_OFFSET * stream::sin_s(angle),
            pos.y,
            bag.z + BAG_OFFSET * stream::cos_s(angle),
        )
    }

    pub fn head_sphere(&self, head: Vec3) -> Vec3 {
        Vec3::new(head.x + HEAD_SPHERE_DX, head.y, head.z)
    }

    pub fn head_sphere_radius(&self) -> f32 {
        if self.npc_no == 1 {
            HEAD_SPHERE_R_B
        } else {
            HEAD_SPHERE_R
        }
    }

    pub fn in_wind(&self, gusts: &Query<(&Actor, &WindTag), Without<Merchant>>) -> bool {
        let head = self.head_sphere(self.eye);
        for (actor, tag) in gusts.iter() {
            if tag.inside(actor.pos, head) {
                return true;
            }
        }
        false
    }

    pub fn begin(&mut self, actor: &Actor, list: &EventList, stories: &StoryBits, path: &[Vec3]) {
        self.started = true;
        self.home_pos = actor.pos;
        self.current_y = actor.angle[1];
        self.shape_y = actor.angle[1];
        self.spawn_y = actor.angle[1];
        self.eye = Vec3::new(actor.pos.x, actor.pos.y + EYE_Y_OFFSET, actor.pos.z);
        self.ev_1st = event_index(list, EV_EXCHANGE_1ST);
        self.ev_2nd = event_index(list, EV_EXCHANGE_2ND);
        self.ev_item = event_index(list, EV_CHANGE_ITEM);
        self.ev_map = event_index(list, EV_GET_MAP);
        self.enabled = stories.test(BIT_ENABLED);
        if self.rail_id != NO_RAIL {
            self.path.set(path.len(), true);
            self.timer = 1;
        }
        self.set_anm_tbl(TBL_WAIT, 0);
    }

    pub fn move_forward(&mut self, pos: &mut Vec3) {
        let heading = walk::from_s16(self.current_y);
        pos.x += self.speed_f * heading.sin();
        pos.z += self.speed_f * heading.cos();
    }
}

fn seq_at(seq: Seq, npc: u8, item: usize, pos: usize) -> Option<u32> {
    let row = usize::from(npc);
    let slot = row.saturating_mul(ITEM_SLOTS).saturating_add(item);
    let slice: &[u32] = match seq {
        Seq::FirstTalk => match row {
            0 => &MSG_FIRST_TALK_A,
            1 => &MSG_FIRST_TALK_B,
            2 => &MSG_FIRST_TALK_C,
            _ => return None,
        },
        Seq::Exchange3 => match row {
            0 => &MSG_EXCHANGE3_A,
            1 => &MSG_EXCHANGE3_B,
            2 => &MSG_EXCHANGE3_C,
            _ => return None,
        },
        Seq::Exchange => match row {
            0 => &MSG_EXCHANGE_A,
            1 => &MSG_EXCHANGE_B,
            2 => &MSG_EXCHANGE_C,
            _ => return None,
        },
        Seq::Etc => match row {
            0 => &MSG_ETC_A,
            1 => &MSG_ETC_B,
            2 => &MSG_ETC_C,
            _ => return None,
        },
        Seq::XyExchange3 => match row {
            0 => &MSG_XY_EXCHANGE3_A,
            1 => &MSG_XY_EXCHANGE3_B,
            2 => &MSG_XY_EXCHANGE3_C,
            _ => return None,
        },
        Seq::XyNoItem => match row {
            0 => &MSG_XY_NO_ITEM_A,
            1 => &MSG_XY_NO_ITEM_B,
            2 => &MSG_XY_NO_ITEM_C,
            _ => return None,
        },
        Seq::XyBadItem => match row {
            0 => &MSG_XY_BAD_ITEM_A,
            1 => &MSG_XY_BAD_ITEM_B,
            2 => &MSG_XY_BAD_ITEM_C,
            _ => return None,
        },
        Seq::KoukanItem => {
            return MSG_KOUKAN_ITEM
                .get(slot)
                .copied()
                .and_then(|row| row.get(pos).copied());
        }
        Seq::KoukanItem2 => {
            return MSG_KOUKAN_ITEM2
                .get(slot)
                .copied()
                .and_then(|row| row.get(pos).copied());
        }
        Seq::KoukanItem3 => {
            return MSG_KOUKAN_ITEM3
                .get(slot)
                .copied()
                .and_then(|row| row.get(pos).copied());
        }
        Seq::AfterGet => match row {
            0 => &MSG_AFTER_GET_A,
            1 => &MSG_AFTER_GET_B,
            2 => &MSG_AFTER_GET_C,
            _ => return None,
        },
        Seq::AfterGet2 => match row {
            0 => &MSG_AFTER_GET2_A,
            1 => &MSG_AFTER_GET2_B,
            2 => &MSG_AFTER_GET2_C,
            _ => return None,
        },
        Seq::CollectMap => &MSG_COLLECT_MAP,
        Seq::TryForce => &MSG_TRY_FORCE,
    };
    slice.get(pos).copied()
}

fn anm_at(table: u8, pos: u8) -> Option<AnmDat> {
    ANM_TABLES
        .get(usize::from(table))
        .and_then(|entries| entries.get(usize::from(pos)))
        .copied()
}

fn event_index(list: &EventList, name: &str) -> i16 {
    list.event_index(name)
        .and_then(|index| i16::try_from(index).ok())
        .unwrap_or(NO_EVENT)
}

fn dis_xz_ang_y(from: Vec3, to: Vec3) -> (f32, i16) {
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    (
        (dx * dx + dz * dz).sqrt(),
        walk::to_s16(dx.atan2(dz)),
    )
}

fn player_eye_pos(player: Vec3, offset_y: f32) -> Vec3 {
    Vec3::new(player.x, player.y + PLAYER_HEAD_TOP + offset_y, player.z)
}

fn limitter(value: &mut i16, max_diff: i16, min_diff: i16) {
    if *value > max_diff {
        *value = max_diff;
    }
    if *value < min_diff {
        *value = min_diff;
    }
}

fn add_calc_angle_l(value: i32, target: i32, scale: i32, max_step: i32, min_step: i32) -> i32 {
    let diff = target - value;
    if value == target {
        return value;
    }
    let mut step = diff / scale;
    if step > min_step || step < -min_step {
        if step > max_step {
            step = max_step;
        }
        if step < -max_step {
            step = -max_step;
        }
        value + step
    } else if diff >= 0 {
        let next = value + min_step;
        if target - next <= 0 { target } else { next }
    } else {
        let next = value - min_step;
        if target - next >= 0 { target } else { next }
    }
}

fn distance_angle_s(a: i16, b: i16) -> i32 {
    i32::from(a.wrapping_sub(b)).abs()
}

fn calc_timer(value: &mut i16) -> i16 {
    if *value != 0 {
        *value -= 1;
    }
    *value
}

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = f32::from(seed[0]) / RND_SCALE[0]
        + f32::from(seed[1]) / RND_SCALE[1]
        + f32::from(seed[2]) / RND_SCALE[2];
    (sum % 1.0).abs()
}

fn rnd_range(seed: &mut [i32; 3], min: i16, range: i16) -> i16 {
    (f32::from(min) + rnd(seed) * f32::from(range)) as i16
}

pub fn is_merchant(name: &str) -> bool {
    MERCHANT_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    do_intent: Res<DoIntent>,
    list: Res<EventList>,
    archive: Option<Res<MsgArchives>>,
    mut active: ResMut<ActiveEvent>,
    mut procs: ResMut<MsgProcs>,
    mut info: ResMut<MsgGameInfo>,
    mut boxes: ResMut<Msg1Box>,
    mut stories: ResMut<StoryBits>,
    mut ev_flags: ResMut<EventFlags>,
    mut order_flags: ResMut<OrderFlags>,
    mut rupees: ResMut<Rupees>,
    players: Query<&Player>,
    gusts: Query<(&Actor, &WindTag), Without<Merchant>>,
    mut actors: Query<(Entity, &mut Actor, &mut Merchant), Without<Player>>,
) {
    let solid = collision.as_deref();
    let archive = archive.as_deref();
    let path: &[Vec3] = &[];
    let status = info.mesg_status;
    let select_num = boxes.select_num;
    let player = players.iter().next().copied();
    let pressed = do_intent.0;
    for (entity, actor, merchant) in &mut actors {
        let actor = &mut *actor;
        let merchant = &mut *merchant;
        if !merchant.started {
            merchant.begin(actor, &list, &stories, path);
        }
        if !merchant.enabled {
            continue;
        }
        let player_pos = player.map_or(actor.pos, |link| link.pos);
        let near = player.is_some_and(|link| {
            let dx = link.pos.x - actor.pos.x;
            let dz = link.pos.z - actor.pos.z;
            dx * dx + dz * dz <= LOOK_DIST * LOOK_DIST
        });
        merchant.check_order(&active, near, pressed);
        merchant.chk_attention(actor.pos, player_pos);
        {
            let mut shop = Shop {
                procs: &mut procs,
                info: &mut info,
                archive,
                stories: &mut stories,
                rupees: &mut rupees,
                ev_flags: &mut ev_flags,
                status,
                select_num,
                present_end: true,
            };
            if !shop.info.event_run_check()
                || merchant.command == CMD_TALK && merchant.bits & BIT_EVENT_END == 0
            {
                match merchant.step {
                    Step::Wait => merchant.execute_wait(&mut shop, actor.pos, path),
                    Step::Talk => merchant.execute_talk(&mut shop, actor.pos, path),
                    Step::Walk => merchant.execute_walk(&mut shop, actor.pos, path),
                    Step::Turn => merchant.execute_turn(&mut shop, actor.pos, path),
                    Step::Wind => merchant.execute_wind(&mut shop, actor.pos, path),
                }
            } else {
                merchant.event_move(&mut shop, &active, actor.pos, path);
            }
            if merchant.wind_hit == 0 && merchant.in_wind(&gusts) {
                merchant.execute_set_mode(Mode::Wind, &mut shop, actor.pos, path);
                merchant.wind_hit = 1;
            }
        }
        merchant.event_order(entity, &list, &mut active, &stories, &mut order_flags);
        merchant.play_tex_pattern_anm();
        merchant.play_anm(true);
        merchant.move_forward(&mut actor.pos);
        if let Some(surface) = ground::surface(solid, actor.pos)
            && actor.pos.y <= surface
        {
            actor.pos.y = surface;
        }
        merchant.set_jnt_param();
        merchant.look_back();
        actor.angle[1] = merchant.current_y;
    }
}