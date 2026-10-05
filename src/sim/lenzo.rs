use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};

use super::actor::Actor;
use super::clock::Clock;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::StoryBits;
use super::inventory::Got;
use super::intent::DoIntent;
use super::pickup::Rupees;
use super::player::Player;
use super::stream;
use super::walk;
use crate::assets::MsgArchives;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG2, MsgGameInfo, MsgProcs, fop_msgm_message_set,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED,
    STATUS_MSG_ENDS, STATUS_TYPING,
};

pub const LENZO_ACTOR: &str = "Po";
pub const TAG_PHOTO_ACTOR: &str = "TagPo";
pub const UB4_ACTOR: &str = "Ub4";

pub const PRM_RAIL_SHIFT: u32 = 0x08;
pub const PRM_RAIL_MASK: u32 = 0xFF;
pub const PRM_ARG0_SHIFT: u32 = 0x00;
pub const PRM_ARG0_MASK: u32 = 0xFF;
pub const PRM_TAG_NO_SHIFT: u32 = 0x08;
pub const PRM_TAG_NO_MASK: u32 = 0xFF;

pub const NO_INDEX: i16 = 0xFF;
pub const NO_ARG0: u8 = 0xFF;
pub const NO_TEM: i8 = -1;
pub const NO_ITEM: u8 = 0xFF;
pub const NO_CUT: i32 = -1;
pub const NO_PATH: u8 = 0xFF;
pub const NO_STATUS: u8 = 0xFF;
pub const BOX_CLOSED: u16 = 0x12;
pub const ANM_TERMINATOR: u8 = 0xFF;
pub const ANM_MODE_HOLD: i32 = 0;
pub const ANM_MODE_RESET: i32 = 2;

pub const MODE_WAIT: u8 = 0;
pub const MODE_TALK: u8 = 1;
pub const MODE_WALK: u8 = 2;
pub const MODE_TURN: u8 = 3;

pub const MODE_TIMER_WAIT_MIN: i16 = 0x001E;
pub const MODE_TIMER_WAIT_SPAN: i16 = 0x0000;
pub const MODE_TIMER_WALK_MIN: i16 = 0x270F;
pub const MODE_TIMER_WALK_SPAN: i16 = 0x0000;

pub const ORDER_IDLE: u8 = 0;
pub const ORDER_ATTENTION: u8 = 1;
pub const ORDER_TOUCH: u8 = 2;
pub const ORDER_LINK_BACK: u8 = 3;
pub const ORDER_ITEM_FILM: u8 = 4;
pub const ORDER_ITEM_BOTTLE: u8 = 5;
pub const ORDER_GIVE_PHOTO: u8 = 6;
pub const ORDER_GALLERY: u8 = 7;
pub const ORDER_DATE_UB4: u8 = 10;

pub const REQUEST_ITEM_FILM: u8 = 0x01;
pub const REQUEST_ITEM_BOTTLE: u8 = 0x02;
pub const REQUEST_SPOIT: u8 = 0x04;
pub const REQUEST_UB4_DONE: u8 = 0x10;
pub const REQUEST_PRICE: u8 = 0x20;
pub const REQUEST_LINK_BACK: u8 = 0x40;
pub const REQUEST_CLEAR: u8 = 0xFE;

pub const ANM_FLAG_TAIL: u8 = 0x01;
pub const ANM_FLAG_SPOIT_LOCK: u8 = 0x04;
pub const ANM_FLAG_TEX_FREEZE: u8 = 0x80;

pub const ATTN_FULL: u8 = 0x10 | 0x01 | 0x04;
pub const ATTN_BUY: u8 = 0x01 | 0x04;

pub const SAVE_TALKED_ONCE: u16 = 0x1208;
pub const SAVE_PATH_ARRIVED: u16 = 0x1601;
pub const SAVE_POST_TALK: u16 = 0x1701;
pub const SAVE_ORDER_PROGRESS: u16 = 0xC407;
pub const SAVE_PHOTO_DX_DONE: u16 = 0x3808;

pub const WEEKDAY_ZELDA_AWAKENED: u16 = 0x2D02;
pub const WEEKDAY_FAIRY: u16 = 0x1001;
pub const WEEKDAY_GANON: u16 = 0x3910;
pub const WEEKDAY_PERL: u16 = 0x3920;
pub const WEEKDAY_KOKIRI: u16 = 0x2D20;
pub const WEEKDAY_ZOLA: u16 = 0x2D40;
pub static WEEKDAY_TOWN: [u16; 7] = [
    0x3002, 0x3001, 0x3008, 0x3004, 0x3020, 0x3010, 0x3180,
];
pub static WEEKDAY_FLAGS: [&[u16]; 7] = [
    &[WEEKDAY_ZELDA_AWAKENED],
    &[WEEKDAY_GANON],
    &WEEKDAY_TOWN,
    &[WEEKDAY_PERL],
    &[WEEKDAY_FAIRY],
    &[WEEKDAY_KOKIRI],
    &[WEEKDAY_ZOLA],
];

pub const MSG_CLEAR_COUNT: u8 = 0;
pub const MSG_CAM_NONE: i8 = -1;
pub const MSG_CAM_FOV: f32 = 60.0;
pub const MSG_CAM_ROLL: i32 = 0;

pub const ITEM_PICTO_BOX: u8 = 0x23;
pub const ITEM_DELUXE_PICTO_BOX: u8 = 0x26;
pub const ITEM_FIREFLY_BOTTLE: u8 = 0x58;
pub const ITEM_LEGENDARY_PICTOGRAPH: u8 = 0x9F;
pub const ITEM_GIFT_PENDANT: u8 = 31;
pub const ITEM_GIFT_DELUXE_BOX: u8 = 38;
pub const ITEM_COLLECT_MAP_20: u8 = 0xEB;

pub const GOT_PICTO_BOX_SLOT: usize = 8;
pub const GOT_DELUXE_SLOT: usize = 15;
pub const GOT_FIREFLY_SLOT: usize = 16;
pub const GOT_LEGENDARY_SLOT: usize = 17;
pub const GOT_MAP20_SLOT: usize = 18;
pub const GOT_BIT: u8 = 0;

pub const SE_UTSUSHIE_TO_DX: u32 = 0x0907;

pub const GRAVITY: f32 = -9.0;
pub const COLLIDE_HEIGHT: f32 = 150.0;
pub const RADIUS_NEAR: f32 = 60.0;
pub const RADIUS_WIDE: f32 = 150.0;
pub const COUNTER_RADIUS: f32 = 110.0;
pub const CHASE_STEP_WALK: f32 = 0.5;
pub const CHASE_STEP_RUN: f32 = 0.1;
pub const PLAY_SPEED_MIN: f32 = 0.5;
pub const TALK_REACH: f32 = 173.0;
pub const DROP_SPAN: f32 = 400.0;
pub const UB4_MIN_Y: f32 = 10.0;
pub const UB4_MIN_X: f32 = -300.0;
pub const UB4_MIN_Z: f32 = -500.0;
pub const GALLERY_MIN_Y: f32 = 400.0;
pub const GALLERY_MIN_X: f32 = -600.0;
pub const DESCEND_STEP: f32 = 50.0;
pub const PHOTO_PROBE: f32 = 50.0;
pub const EYE_ORBIT: f32 = 100.0;
pub const EYE_ORBIT_SWING: f32 = 12288.0;
pub const EYE_ORBIT_RATE: i16 = 0x400;
pub const EYE_ORBIT_TILT: i16 = -0x1000;

pub const NPC_ATTN_OFFSET: Vec3 = Vec3::new(0.0, 215.0, 30.0);
pub const NPC_EYE_LIFT: f32 = 300.0;
pub const NPC_PLAYER_EYE_LIFT: f32 = 0.0;
pub const NPC_COLLIDE_Z: f32 = -50.0;
pub const NPC_ATTN_DIST: f32 = 200.0;
pub const NPC_ATTN_DIST_BONUS: f32 = 40.0;
pub const NPC_ATTN_RANGE: i16 = 0x7FFF;
pub const NPC_ATTN_ANGLE_BONUS: i32 = 1820;
pub const NPC_ANM_SCALE: f32 = 0.3;
pub const NPC_WALK_SPEED: f32 = 5.0;
pub const NPC_WALK_SPEED_SCALE: f32 = 1.3;
pub const NPC_TURN_STEP: i16 = 0x0708;
pub const NPC_TURN_TARGET: i16 = 0x0320;
pub const NPC_ATTN_RELEASE: i16 = 30;

pub const JNT_HEAD_MAX_X: i16 = 0x09C4;
pub const JNT_HEAD_MAX_Y: i16 = 0x07D0;
pub const JNT_HEAD_MIN_X: i16 = -150;
pub const JNT_HEAD_MIN_Y: i16 = -512;
pub const JNT_SPINE_MAX_X: i16 = 0x1388;
pub const JNT_SPINE_MAX_Y: i16 = 0x1F40;
pub const JNT_SPINE_MIN_X: i16 = -1000;
pub const JNT_SPINE_MIN_Y: i16 = -8000;
pub const JNT_MAX_STEP: i16 = 0x01F4;
pub const JNT_MOVE_SCALE: i16 = 4;
pub const JNT_MOVE_MIN: i16 = 1;
pub const JNT_FOLLOW_SCALE: i16 = 4;
pub const JNT_FOLLOW_MIN: i16 = 0x60;
pub const TURN_RATE_SCALE: i16 = 4;
pub const TURN_RATE_MAX: i16 = 0x800;

pub const LOOK_NONE: i8 = 0;
pub const LOOK_POINT: i8 = 1;
pub const LOOK_ANGLE: i8 = 2;

pub const STAGE_NORMAL: u8 = 0;
pub const STAGE_UB4: u8 = 4;
pub const STAGE_PATH_DONE: u8 = 2;
pub const STAGE_DESCEND: u8 = 3;

pub const PHOTO_LIMIT: u8 = 3;
pub const PHOTO_WAIT: i16 = 10;
pub const ORDER_DONE_ALL: u8 = 6;
pub const TEX_PATTERN_MAX: i32 = 2;

pub const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
pub const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
pub const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
pub const RND_SEED: i32 = 100;

pub const EV_LINK_BACK: usize = 0;
pub const EV_GET_ITEM: usize = 1;
pub const EV_GET_ITEM2: usize = 2;
pub const EV_GET_PHOTO: usize = 3;
pub const EV_GALLERY: usize = 4;
pub const EV_COUNTER_TALK0: usize = 5;
pub const EV_COUNTER_TALK1: usize = 6;
pub const EV_DATE_UB4: usize = 7;

pub const EVENT_NAMES: [&str; 8] = [
    "PHOTO_LINK_BACK",
    "PHOTO_GET_ITEM",
    "PHOTO_GET_ITEM2",
    "PHOTO_GET_PHOTO",
    "PHOTO_GALLERY",
    "PHOTO_COUNTER_TALK0",
    "PHOTO_COUNTER_TALK1",
    "PHOTO_DATE_UB4",
];
pub const EVENT_DEFAULT_TALK_XY: &str = "DEFAULT_TALK_XY";

pub const CUT_MES_SET: i8 = 0;
pub const CUT_SE_SET: i8 = 1;
pub const CUT_POS_SET: i8 = 2;
pub const CUT_GET_ITEM: i8 = 3;
pub const CUT_SET_ANGLE: i8 = 4;
pub const CUT_SET_EYE: i8 = 5;
pub const CUT_TURN_TO_PLAYER: i8 = 6;
pub const CUT_CLR_HANME: i8 = 7;
pub const CUT_GET_PHOTO: i8 = 8;
pub const CUT_MES_SET_UB: i8 = 9;
pub const CUT_LOOK_UB: i8 = 10;

pub const CUT_NAMES: [&str; 11] = [
    "MES_SET",
    "SE_SET",
    "POS_SET",
    "GET_ITEM",
    "SET_ANGLE",
    "SET_EYE",
    "TURN_TO_PLAYER",
    "CLR_HANME",
    "GET_PHOTO",
    "MES_SET_UB",
    "LOOK_UB",
];

pub const MSG_TAIL: u32 = 0x0000;
pub const MSG_CANCEL: u32 = 0x0001;
pub const MSG_FIRST_END: u32 = 0x0002;
pub const MSG_SECOND_END: u32 = 0x0003;
pub const MSG_SECOND_END_2: u32 = 0x0004;
pub const MSG_THIRD_END: u32 = 0x0005;
pub const MSG_THIRD_END_2: u32 = 0x0006;
pub const MSG_COLOR_XY_A: u32 = 0x0007;
pub const MSG_COLOR_XY_B: u32 = 0x0008;
pub const MSG_BUY_ASK: u32 = 0x0009;
pub const MSG_BUY_TAIL: u32 = 0x000A;
pub const MSG_BUY_TAIL_2: u32 = 0x000B;
pub const MSG_ANSWER_BUY: u32 = 0x378A;
pub const MSG_NO_FUNDS: u32 = 0x378B;
pub const MSG_GIVE_UP: u32 = 0x2A5C;
pub const MSG_RETRY: u32 = 0x2A5D;
pub const MSG_RETRY_2: u32 = 0x2A57;
pub const MSG_FINISHED: u32 = 0x2A63;
pub const MSG_FULL: u32 = 0x3787;
pub const MSG_UNRELATED: u32 = 0x2A56;
pub const MSG_CHOOSE_ASK: u32 = 0x2A3A;
pub const MSG_CHOOSE_ASK_2: u32 = 0x2A3D;
pub const MSG_CHOOSE_TAIL: u32 = 0x2A3B;

pub const MSG_2F: [u32; 3] = [0x2A82, 0x2A83, 0x2A84];

pub static MSG_FIRST_TALK: [u32; 4] = [0x2A31, 0x2A32, 0x2A33, MSG_TAIL];
pub static MSG_SECOND_TALK: [u32; 2] = [0x2A34, MSG_TAIL];
pub static MSG_FIRST_PHOTO: [u32; 3] = [0x2A35, 0x2A36, MSG_TAIL];
pub static MSG_SECOND_PHOTO: [u32; 3] = [0x2A37, 0x2A38, MSG_TAIL];
pub static MSG_FIRST_ORDER: [u32; 11] = [
    0x2A39, 0x2A3A, 0x2A3C, 0x2A3D, 0x2A3E, 0x2A3F, 0x2A40, 0x2A41, 0x2A42, 0x2A43, MSG_CANCEL,
];
pub static MSG_FIRST_ORDER_NOT_END: [u32; 3] = [0x2A44, 0x2A45, MSG_TAIL];
pub static MSG_SECOND_ORDER: [u32; 4] = [0x2A46, 0x2A47, 0x2A48, MSG_SECOND_END];
pub static MSG_SECOND_ORDER_NOT_END: [u32; 3] = [0x2A49, 0x2A4A, MSG_TAIL];
pub static MSG_THIRD_ORDER: [u32; 5] = [0x2A4B, 0x2A4C, 0x2A4D, 0x2A4E, MSG_TAIL];
pub static MSG_THIRD_ORDER_NOT_END: [u32; 3] = [0x2A4F, 0x2A50, MSG_TAIL];
pub static MSG_THIRD_ORDER_END: [u32; 2] = [0x2A51, MSG_TAIL];
pub static MSG_NOT_COLOR: [u32; 3] = [0x2A52, 0x2A53, MSG_TAIL];
pub static MSG_COLOR: [u32; 3] = [0x2A54, 0x2A55, MSG_TAIL];

pub static MSG_FIRST_ORDER_CAM: [u8; 5] = [0xFF, 0x00, 0x01, 0xFF, 0x02];
pub static MSG_FIRST_ORDER_NOT_END_CAM: [u8; 1] = [0x02];
pub static MSG_SECOND_ORDER_CAM: [u8; 3] = [0xFF, 0x00, 0xFF];
pub static MSG_SECOND_ORDER_NOT_END_CAM: [u8; 1] = [0x02];
pub static MSG_THIRD_ORDER_CAM: [u8; 4] = [0x02, 0xFF, 0x00, 0xFF];
pub static MSG_THIRD_ORDER_NOT_END_CAM: [u8; 2] = [0xFF, 0xFF];
pub static MSG_NOT_COLOR_CAM: [u8; 2] = [0x00, 0xFF];
pub static MSG_COLOR_CAM: [u8; 2] = [0x01, 0xFF];

pub static MSG_WEEK_FIRST: [u32; 6] = [0x3781, 0x3782, 0x3783, 0x3784, 0x3785, MSG_TAIL];
pub static MSG_WEEK: [[u32; 4]; 7] = [
    [0x3779, 0x377A, 0x3785, MSG_TAIL],
    [0x3779, 0x377B, 0x3785, MSG_TAIL],
    [0x3779, 0x377C, 0x3785, MSG_TAIL],
    [0x3779, 0x377D, 0x3785, MSG_TAIL],
    [0x3779, 0x377E, 0x3785, MSG_TAIL],
    [0x3779, 0x377F, 0x3785, MSG_TAIL],
    [0x3779, 0x3780, 0x3785, MSG_TAIL],
];

pub static MSG_FIRST_TALK_PHOTO: [u32; 5] = [0x2A69, 0x2A6A, 0x2A6B, 0x2A6C, MSG_TAIL];
pub static MSG_TALK_PHOTO: [[u32; 3]; 7] = [
    [0x2A6E, 0x2A6F, MSG_TAIL],
    [0x2A71, 0x2A72, MSG_TAIL],
    [0x2A74, 0x2A75, MSG_TAIL],
    [0x2A77, 0x2A78, MSG_TAIL],
    [0x2A7A, 0x2A7B, MSG_TAIL],
    [0x2A7D, 0x2A7E, MSG_TAIL],
    [0x2A80, 0x2A81, MSG_TAIL],
];

pub static MSG_BUY_PHOTO: [u32; 2] = [0x3786, MSG_TAIL];
pub static MSG_GET_PHOTO: [u32; 2] = [0x3785, MSG_TAIL];
pub static MSG_DOWN: [u32; 2] = [0x2A86, MSG_TAIL];
pub static MSG_UB4: [u32; 2] = [0x2D62, MSG_TAIL];
pub static MSG_FIRST_ORDER_XY: [u32; 4] = [0x2A5E, 0x2A5F, 0x2A60, MSG_FIRST_END];
pub static MSG_SECOND_ORDER_XY: [u32; 4] = [0x2A61, 0x2A62, 0x2A60, MSG_SECOND_END_2];
pub static MSG_THIRD_ORDER_XY: [u32; 3] = [0x2A64, 0x2A60, MSG_THIRD_END_2];
pub static MSG_COLOR_XY: [u32; 8] = [
    0x2A58, 0x2A59, 0x2A5A, MSG_COLOR_XY_A, 0x2A85, MSG_COLOR_XY_B, 0x2A5B, MSG_TAIL,
];
pub static MSG_XY_BUY_PHOTO: [u32; 9] = [
    0x3788, 0x3789, MSG_BUY_ASK, 0x378C, MSG_BUY_TAIL, 0x378D, MSG_BUY_TAIL_2, 0x3786, MSG_TAIL,
];

pub static MSG_CAMERA: [[Vec3; 2]; 3] = [
    [Vec3::new(-1336.0, -497.0, 928.0), Vec3::new(185.0, -26.0, 0.0)],
    [Vec3::new(-1213.0, -902.0, -244.0), Vec3::new(284.0, 36.0, 280.0)],
    [Vec3::new(-29.0, 252.0, -1634.0), Vec3::new(5.0, -91.0, 178.0)],
];

pub static COUNTER_POS: [Vec3; 2] = [
    Vec3::new(-490.0, 0.0, -10.0),
    Vec3::new(-260.0, 0.0, -250.0),
];
pub const GALLERY_POS: Vec3 = Vec3::new(-260.0, 500.0, 400.0);
pub static NO_POINTS: [Vec3; 0] = [];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhotoAnmDat {
    pub bck: u8,
    pub speed: u8,
    pub frames: u8,
}

pub const ANM_WAIT: &[PhotoAnmDat] = &[PhotoAnmDat {
    bck: 0x00,
    speed: 0x08,
    frames: 0xFF,
}];
pub const ANM_TALK: &[PhotoAnmDat] = &[PhotoAnmDat {
    bck: 0x01,
    speed: 0x08,
    frames: 0xFF,
}];
pub const ANM_WALK: &[PhotoAnmDat] = &[PhotoAnmDat {
    bck: 0x03,
    speed: 0x08,
    frames: 0xFF,
}];
pub const ANM_SPIT: &[PhotoAnmDat] = &[
    PhotoAnmDat {
        bck: 0x04,
        speed: 0x08,
        frames: 0x01,
    },
    PhotoAnmDat {
        bck: 0x00,
        speed: 0x08,
        frames: 0xFF,
    },
];
pub const ANM_TALK2: &[PhotoAnmDat] = &[
    PhotoAnmDat {
        bck: 0x02,
        speed: 0x08,
        frames: 0x03,
    },
    PhotoAnmDat {
        bck: 0x00,
        speed: 0x08,
        frames: 0xFF,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Wait,
    Talk,
    Walk,
    Turn,
}

impl Step {
    pub fn from_raw(raw: u8) -> Self {
        match raw & 3 {
            MODE_WAIT => Self::Wait,
            MODE_TALK => Self::Talk,
            MODE_WALK => Self::Walk,
            _ => Self::Turn,
        }
    }

    pub fn raw(self) -> u8 {
        match self {
            Self::Wait => MODE_WAIT,
            Self::Talk => MODE_TALK,
            Self::Walk => MODE_WALK,
            Self::Turn => MODE_TURN,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Lenzo {
    pub rail: u8,
    pub arg0: u8,
    pub kind: u8,
    pub step: Step,
    pub request: u8,
    pub order: u8,
    pub item_no: u8,
    pub bck_index: u8,
    pub anm_flags: u8,
    pub anm_hold: i8,
    pub act_idx: i8,
    pub btp_index: i8,
    pub btp_frame: u8,
    pub btp_timer: i16,
    pub mode_timer: i16,
    pub attn_release: i16,
    pub home_angle: i16,
    pub eye_timer: i16,
    pub attn_range: i16,
    pub attn_flags: u8,
    pub turn_target: i16,
    pub turn_rate: i16,
    pub desired_angle: i16,
    pub talking: bool,
    pub attention: bool,
    pub demo_talk: bool,
    pub order_done: bool,
    pub counter_hit: bool,
    pub stage_flag: bool,
    pub head_blend: bool,
    pub eye_on: bool,
    pub back_on: bool,
    pub cam_paused: bool,
    pub cam_armed: u8,
    pub look_mode: i8,
    pub speed_f: f32,
    pub speed_target: f32,
    pub play_speed: f32,
    pub anm_blend: f32,
    pub col_radius: f32,
    pub col_offset: Vec3,
    pub eye_pos: Vec3,
    pub look_at: Vec3,
    pub attn_pos: Vec3,
    pub cam_eye: Vec3,
    pub cam_look: Vec3,
    pub jnt: [[i16; 2]; 2],
    pub rng_msg_no: i32,
    pub curr_msg_no: u32,
    pub msg_id: u32,
    pub msg_live: bool,
    pub msg_tbl: Option<&'static [u32]>,
    pub msg_at: usize,
    pub cam_tbl: Option<&'static [u8]>,
    pub cam_at: usize,
    pub event_idx: [i16; 8],
    pub path_idx: u8,
    pub path_dir: u8,
    pub path_on: bool,
    pub path_point: Option<Vec3>,
    pub path_ended: bool,
    pub turn_ready: bool,
    pub anm_tbl: &'static [PhotoAnmDat],
    pub anm_at: usize,
    pub cut_seen: i32,
    pub seed: [i32; 3],
    pub entered: bool,
    pub in_photo: bool,
    pub xy_active: bool,
    pub xy_item: u8,
    pub present_end: bool,
    pub start_stage_point: i16,
    pub pictures: u8,
    pub picture_result: u8,
    pub picture_get_day: u8,
    pub erase_requested: bool,
    pub pending_present: Option<u8>,
    pub bottle_spent: bool,
    pub link_move_angle: i16,
    pub tmp_bought_film: bool,
    pub tmp_took_photo: bool,
}

impl Lenzo {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            rail: ((parameters >> PRM_RAIL_SHIFT) & PRM_RAIL_MASK) as u8,
            arg0: ((parameters >> PRM_ARG0_SHIFT) & PRM_ARG0_MASK) as u8,
            kind: STAGE_NORMAL,
            step: Step::Wait,
            request: 0,
            order: ORDER_IDLE,
            item_no: 0,
            bck_index: 0,
            anm_flags: 0,
            anm_hold: NO_TEM,
            act_idx: NO_TEM,
            btp_index: 0,
            btp_frame: 0,
            btp_timer: 0,
            mode_timer: 0,
            attn_release: 0,
            home_angle: 0,
            eye_timer: 0,
            attn_range: NPC_ATTN_RANGE,
            attn_flags: ATTN_FULL,
            turn_target: 0,
            turn_rate: 0,
            desired_angle: 0,
            talking: false,
            attention: false,
            demo_talk: true,
            order_done: false,
            counter_hit: false,
            stage_flag: false,
            head_blend: true,
            eye_on: true,
            back_on: true,
            cam_paused: false,
            cam_armed: 0,
            look_mode: LOOK_NONE,
            speed_f: 0.0,
            speed_target: 0.0,
            play_speed: 0.0,
            anm_blend: -1.0,
            col_radius: RADIUS_NEAR,
            col_offset: Vec3::ZERO,
            eye_pos: Vec3::ZERO,
            look_at: Vec3::ZERO,
            attn_pos: Vec3::ZERO,
            cam_eye: Vec3::ZERO,
            cam_look: Vec3::ZERO,
            jnt: [[0; 2]; 2],
            rng_msg_no: 0,
            curr_msg_no: 0,
            msg_id: FPC_M_ERROR_PROCESS_ID,
            msg_live: false,
            msg_tbl: None,
            msg_at: 0,
            cam_tbl: None,
            cam_at: 0,
            event_idx: [NO_INDEX; 8],
            path_idx: 0,
            path_dir: 0,
            path_on: false,
            path_point: None,
            path_ended: false,
            turn_ready: false,
            anm_tbl: ANM_WAIT,
            anm_at: 0,
            cut_seen: NO_CUT,
            seed: [RND_SEED; 3],
            entered: false,
            in_photo: false,
            xy_active: false,
            xy_item: NO_ITEM,
            present_end: false,
            start_stage_point: 0,
            pictures: 0,
            picture_result: 0,
            picture_get_day: 0,
            erase_requested: false,
            pending_present: None,
            bottle_spent: false,
            link_move_angle: 0,
            tmp_bought_film: false,
            tmp_took_photo: false,
        }
    }

    pub fn phase1(&mut self, got: &Got) -> bool {
        let arg0 = self.arg0;
        if arg0 == NO_ARG0 {
            self.stage_flag = true;
            return true;
        }
        let point = i32::from(self.start_stage_point);
        let arg = i32::from(arg0);
        let map20 = got.has(GOT_MAP20_SLOT, GOT_BIT);
        if arg0 == 0 {
            if map20 || arg != point {
                return false;
            }
        } else {
            if !map20 && arg == point {
                return false;
            }
            self.kind = STAGE_UB4;
        }
        self.stage_flag = true;
        true
    }

    pub fn create_init(&mut self, stories: &mut StoryBits, list: &EventList) {
        if self.rail != NO_PATH {
            self.path_on = true;
            self.path_dir = 1;
            if stories.test(SAVE_PATH_ARRIVED) {
                self.kind = STAGE_PATH_DONE;
            }
        }
        for (slot, name) in EVENT_NAMES.iter().enumerate() {
            self.event_idx[slot] = list
                .event_index(name)
                .and_then(|index| i16::try_from(index).ok())
                .unwrap_or(NO_INDEX);
        }
        self.turn_target = 0;
        if self.kind == STAGE_UB4 {
            self.eye_on = false;
            self.back_on = false;
        }
        self.attn_range = NPC_ATTN_RANGE;
        self.speed_target = 0.0;
    }

    pub fn gravity(&self) -> f32 {
        GRAVITY
    }

    pub fn collide_height(&self) -> f32 {
        COLLIDE_HEIGHT
    }

    pub fn stage_normal(&self) -> bool {
        self.kind == STAGE_NORMAL
    }

    pub fn event_index(&self, slot: usize) -> i16 {
        self.event_idx.get(slot).copied().unwrap_or(NO_INDEX)
    }

    pub fn order_reg(stories: &StoryBits) -> u8 {
        stories.reg(SAVE_ORDER_PROGRESS)
    }

    pub fn is_photo_ok(&self, stories: &StoryBits) -> bool {
        match Self::order_reg(stories) & 0xFF {
            1 => self.picture_result == 1,
            3 => self.picture_result == 2,
            5 => self.picture_result == 3,
            _ => false,
        }
    }

    pub fn is_photo_dx_ok(&self, stories: &StoryBits, weekday: u32) -> bool {
        WEEKDAY_FLAGS
            .get(weekday as usize)
            .copied()
            .unwrap_or(&[])
            .iter()
            .any(|flag| stories.test(*flag))
    }

    fn message_at(&self) -> u32 {
        self.msg_tbl
            .and_then(|table| table.get(self.msg_at).copied())
            .unwrap_or(MSG_TAIL)
    }

    fn camera_at(&self) -> i8 {
        self.cam_tbl
            .and_then(|table| table.get(self.cam_at).copied())
            .map_or(MSG_CAM_NONE, i8::from)
    }

    fn bump_message(&mut self) {
        self.msg_at += 1;
        self.cam_at += 1;
    }

    fn drop_message(&mut self) {
        self.msg_tbl = None;
        self.cam_tbl = None;
    }

    fn rnd(&mut self) -> f32 {
        for lane in 0..3 {
            self.seed[lane] = (self.seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
        }
        let sum = (self.seed[0]) as f32 / RND_SCALE[0]
            + (self.seed[1]) as f32 / RND_SCALE[1]
            + (self.seed[2]) as f32 / RND_SCALE[2];
        (sum % 1.0).abs()
    }

    fn jitter(&mut self, min: i16, span: i16) -> i16 {
        let base = (min) as f32;
        let range = (span) as f32;
        (base + self.rnd() * range) as i16
    }

    fn set_anm(&mut self, bck: u8, _mode: i32, _speed: f32) {
        if self.anm_blend >= 0.0 {
            self.anm_blend = -1.0;
        }
        self.bck_index = bck;
    }

    fn set_anm_tbl(&mut self, table: &'static [PhotoAnmDat]) -> bool {
        if self.anm_flags & ANM_FLAG_SPOIT_LOCK != 0 {
            return false;
        }
        let Some(first) = table.first().copied() else {
            return false;
        };
        if first.bck == ANM_TERMINATOR {
            self.anm_tbl = &[];
            self.anm_at = 0;
            return true;
        }
        self.anm_tbl = table;
        self.anm_at = 0;
        self.anm_hold = first.frames as i8;
        let mode = if self.anm_hold > 0 {
            ANM_MODE_HOLD
        } else {
            ANM_MODE_RESET
        };
        if self.bck_index != first.bck || mode == ANM_MODE_HOLD {
            self.set_anm(first.bck, mode, first.speed as f32);
        }
        false
    }

    pub fn init_tex_pattern_anm(&mut self, stories: &StoryBits, _modify: bool, param2: i32) -> bool {
        let index = if param2 == -1 {
            if stories.test(SAVE_POST_TALK) {
                1
            } else {
                0
            }
        } else {
            param2
        };
        self.btp_index = index.clamp(0, TEX_PATTERN_MAX) as i8;
        self.btp_frame = 0;
        self.btp_timer = 0;
        true
    }

    pub fn play_anm(&mut self, anim_done: bool) {
        self.anm_flags &= !ANM_FLAG_TAIL;
        if !anim_done {
            return;
        }
        self.anm_flags &= !ANM_FLAG_SPOIT_LOCK;
        if self.anm_tbl.is_empty() || self.anm_hold <= 0 {
            return;
        }
        self.anm_hold -= 1;
        if self.anm_hold == 0 {
            let rest = self.anm_tbl.get(self.anm_at + 1..).unwrap_or(&[]);
            if self.set_anm_tbl(rest) {
                self.anm_flags |= ANM_FLAG_TAIL;
            }
        } else if let Some(row) = self.anm_tbl.get(self.anm_at).copied() {
            self.set_anm(row.bck, 0, 0.0);
        }
    }

    pub fn set_anm_from_msg_tag(&mut self, info: &mut MsgGameInfo, stories: &StoryBits) {
        match info.mesg_anime {
            0 => {
                self.set_anm_tbl(ANM_WAIT);
                self.anm_flags &= 0x7F;
            }
            1 => {
                self.set_anm_tbl(ANM_TALK);
                self.anm_flags &= 0x7F;
            }
            2 => {
                self.set_anm_tbl(ANM_TALK2);
                self.anm_flags &= 0x7F;
            }
            3 => {
                self.set_anm_tbl(ANM_WAIT);
                self.init_tex_pattern_anm(stories, true, 0);
                self.anm_flags &= 0x7F;
            }
            4 => {
                self.set_anm_tbl(ANM_TALK2);
                self.anm_flags &= 0x7F;
            }
            5 => {
                self.set_anm_tbl(ANM_WAIT);
                self.init_tex_pattern_anm(stories, true, 1);
                self.anm_flags |= ANM_FLAG_TEX_FREEZE;
                self.btp_frame = 1;
            }
            6 => {
                self.set_anm_tbl(ANM_TALK);
                self.init_tex_pattern_anm(stories, true, 0);
                self.anm_flags &= 0x7F;
            }
            9 => {
                self.set_anm_tbl(ANM_SPIT);
                self.anm_flags &= 0x7F;
            }
            _ => {}
        }
        info.set_mesg_anime_attr_info(0xFF);
    }

    pub fn execute_common(&mut self, actor: &Actor) -> bool {
        self.order = if self.attention {
            ORDER_ATTENTION
        } else {
            ORDER_IDLE
        };
        if self.order == ORDER_ATTENTION {
            self.execute_set_mode(Step::Talk, actor.pos, actor.angle[1]);
        }
        self.order == ORDER_ATTENTION
    }

    pub fn execute_set_mode(&mut self, mode: Step, pos: Vec3, yaw: i16) {
        self.speed_target = 0.0;
        let mut next = mode;
        match mode {
            Step::Wait => {
                self.set_anm_tbl(ANM_WAIT);
                self.mode_timer = self.jitter(MODE_TIMER_WAIT_MIN, MODE_TIMER_WAIT_SPAN);
            }
            Step::Walk => {
                self.set_anm_tbl(ANM_WALK);
                self.mode_timer = self.jitter(MODE_TIMER_WALK_MIN, MODE_TIMER_WALK_SPAN);
            }
            Step::Turn => {
                let point = self.path_point.unwrap_or(Vec3::ZERO);
                if dis_xz_ang_y(pos, point).1 == yaw {
                    next = Step::Walk;
                    self.set_anm_tbl(ANM_WALK);
                    self.mode_timer =
                        self.jitter(MODE_TIMER_WALK_MIN, MODE_TIMER_WALK_SPAN);
                }
            }
            Step::Talk => {}
        }
        self.step = next;
    }

    pub fn execute_wait(&mut self, actor: &Actor, ctx: &Ctx<'_>) {
        if self.execute_common(actor) {
            return;
        }
        self.col_radius = RADIUS_NEAR;
        self.col_offset = Vec3::ZERO;
        self.counter_hit = false;
        let link = ctx.player;
        if !ctx.stories.test(SAVE_PATH_ARRIVED) {
            self.col_radius = RADIUS_WIDE;
            self.col_offset = Vec3::new(0.0, 0.0, NPC_COLLIDE_Z);
            if link.is_some_and(|at| {
                inside_cylinder(actor.pos + self.col_offset, self.col_radius, at)
            }) {
                self.order = ORDER_TOUCH;
            }
        } else if self.kind == STAGE_UB4 && (self.request & REQUEST_UB4_DONE) == 0 {
            if link.is_some_and(|at| at.y < UB4_MIN_Y && at.x > UB4_MIN_X && at.z > UB4_MIN_Z) {
                self.order = ORDER_DATE_UB4;
            }
        } else {
            if ctx.start_stage_point == 0
                && !ctx.stories.test(SAVE_ORDER_PROGRESS)
                && self.order != ORDER_GALLERY
                && link.is_some_and(|at| at.y > GALLERY_MIN_Y && at.x > GALLERY_MIN_X)
            {
                self.order = ORDER_GALLERY;
            }
            if self.kind == STAGE_PATH_DONE {
                self.demo_talk = true;
                for spot in COUNTER_POS.iter() {
                    if link.is_some_and(|at| inside_cylinder(*spot, COUNTER_RADIUS, at)) {
                        self.counter_hit = true;
                        break;
                    }
                }
            }
            if self.kind == 1 && !self.attention
                && link.is_some_and(|at| at.y < actor.pos.y - DESCEND_STEP)
            {
                self.execute_set_mode(Step::Turn, actor.pos, actor.angle[1]);
                self.kind = STAGE_DESCEND;
            }
            if self.kind == STAGE_DESCEND && !self.attention {
                if self.mode_timer == 0 {
                    self.execute_set_mode(Step::Turn, actor.pos, actor.angle[1]);
                } else {
                    self.mode_timer -= 1;
                }
            }
        }
    }

    pub fn execute_talk(&mut self, actor: &Actor, ctx: &mut Ctx<'_>) {
        self.execute_common(actor);
        if ctx.xy_active && !ctx.present_end {
            return;
        }
        if self.talk2(ctx, 1) == BOX_CLOSED {
            self.talking = false;
            self.execute_set_mode(Step::Wait, actor.pos, actor.angle[1]);
            let temp = self.request;
            if temp & REQUEST_ITEM_FILM != 0 {
                self.request = temp & REQUEST_CLEAR;
                self.item_no = ITEM_GIFT_PENDANT;
                self.order = ORDER_ITEM_FILM;
            } else if temp & REQUEST_ITEM_BOTTLE != 0 {
                self.request = temp & !REQUEST_ITEM_BOTTLE;
                self.item_no = ITEM_GIFT_DELUXE_BOX;
                self.order = ORDER_ITEM_BOTTLE;
            } else if temp & REQUEST_SPOIT != 0 {
                self.request = temp & !REQUEST_SPOIT;
                self.order = ORDER_LINK_BACK;
                self.set_anm_tbl(ANM_SPIT);
                self.anm_flags |= REQUEST_SPOIT;
            } else {
                self.request = temp | REQUEST_LINK_BACK;
            }
        } else {
            let (info, stories) = (&mut *ctx.text.info, &*ctx.stories);
            self.set_anm_from_msg_tag(info, stories);
        }
    }

    pub fn execute_walk(&mut self, actor: &Actor) {
        if self.execute_common(actor) {
            return;
        }
        if self.attention {
            self.execute_set_mode(Step::Wait, actor.pos, actor.angle[1]);
        } else if !self.path_ended {
            let point = self.path_point.unwrap_or(Vec3::ZERO);
            self.desired_angle = dis_xz_ang_y(actor.pos, point).1;
            self.head_blend = false;
            self.turn_target = NPC_TURN_STEP;
            self.look_mode = LOOK_ANGLE;
            self.turn_ready = true;
            self.speed_target = NPC_WALK_SPEED;
            if point.y - point.y > DROP_SPAN {
                self.speed_target = NPC_WALK_SPEED * NPC_WALK_SPEED_SCALE;
            }
        } else {
            self.execute_set_mode(Step::Wait, actor.pos, actor.angle[1]);
            self.kind = STAGE_PATH_DONE;
        }
    }

    pub fn execute_turn(&mut self, actor: &Actor) {
        if self.execute_common(actor) {
            return;
        }
        let point = self.path_point.unwrap_or(Vec3::ZERO);
        self.desired_angle = dis_xz_ang_y(actor.pos, point).1;
        self.head_blend = false;
        self.look_mode = LOOK_ANGLE;
        if !self.turn_ready {
            self.execute_set_mode(Step::Walk, actor.pos, actor.angle[1]);
        }
    }

    pub fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.chk_attention(actor, ctx);
        self.eye_pos = Vec3::new(actor.pos.x, actor.pos.y + NPC_EYE_LIFT, actor.pos.z);
        let demo_accept = ctx.demo_accept;
        let talk_command = ctx.talk_command;
        self.check_order(ctx, demo_accept, talk_command);
        if !ctx.running() || (talk_command && self.demo_talk) {
            match self.step {
                Step::Wait => self.execute_wait(actor, ctx),
                Step::Talk => self.execute_talk(actor, ctx),
                Step::Walk => self.execute_walk(actor),
                Step::Turn => self.execute_turn(actor),
            }
        } else {
            self.event_move(ctx);
        }
        self.event_order(ctx);
        let anim_done = ctx.anim_done;
        self.play_anm(anim_done);
        if self.bck_index == 3 {
            walk::chase_f(&mut self.speed_f, self.speed_target, CHASE_STEP_RUN);
            self.play_speed = self.speed_f * NPC_ANM_SCALE;
            if self.play_speed < PLAY_SPEED_MIN {
                self.play_speed = PLAY_SPEED_MIN;
            }
        } else {
            walk::chase_f(&mut self.speed_f, self.speed_target, CHASE_STEP_WALK);
            self.play_speed = self.speed_f * NPC_ANM_SCALE;
        }
        let heading = walk::from_s16(actor.angle[1]);
        actor.pos.x += self.speed_f * heading.sin();
        actor.pos.z += self.speed_f * heading.cos();
        self.attn_pos = turned_y(NPC_ATTN_OFFSET, actor.angle[1]) + actor.pos;
        self.look_back(actor);
    }

    pub fn check_order(&mut self, ctx: &mut Ctx<'_>, demo_accept: bool, talk_command: bool) {
        if demo_accept {
            let pairs = [
                (EV_LINK_BACK, ORDER_LINK_BACK),
                (EV_GET_ITEM, ORDER_ITEM_FILM),
                (EV_GET_ITEM2, ORDER_ITEM_BOTTLE),
                (EV_GET_PHOTO, ORDER_GIVE_PHOTO),
                (EV_GALLERY, ORDER_GALLERY),
                (EV_DATE_UB4, ORDER_DATE_UB4),
            ];
            for (slot, order) in pairs {
                let idx = self.event_index(slot);
                if idx < 0 {
                    continue;
                }
                if ctx.active.start_check(usize::from(idx as u16)) && self.order == order {
                    self.order = ORDER_IDLE;
                }
            }
        } else if talk_command && (self.order == ORDER_TOUCH || self.order == ORDER_ATTENTION) {
            self.talking = true;
            self.execute_set_mode(Step::Talk, Vec3::ZERO, 0);
        }
    }

    pub fn event_order(&mut self, ctx: &mut Ctx<'_>) {
        if self.request & REQUEST_LINK_BACK != 0 {
            self.request &= !REQUEST_LINK_BACK;
            let stories = &*ctx.stories;
            self.init_tex_pattern_anm(stories, true, -1);
            self.anm_flags &= 0x7F;
            self.demo_talk = true;
            self.talking = false;
            self.execute_set_mode(Step::Wait, Vec3::ZERO, 0);
        }
        let slot = match self.order {
            ORDER_LINK_BACK => Some(EV_LINK_BACK),
            ORDER_ITEM_FILM => Some(EV_GET_ITEM),
            ORDER_ITEM_BOTTLE => Some(EV_GET_ITEM2),
            ORDER_GIVE_PHOTO => Some(EV_GET_PHOTO),
            ORDER_GALLERY => Some(EV_GALLERY),
            ORDER_DATE_UB4 => Some(EV_DATE_UB4),
            _ => None,
        };
        if let Some(slot) = slot {
            ctx.order_event(self.event_index(slot));
        }
    }

    pub fn event_move(&mut self, ctx: &mut Ctx<'_>) {
        let watched = [
            (EV_LINK_BACK, 0_u8),
            (EV_COUNTER_TALK0, 1),
            (EV_GET_ITEM, 0),
            (EV_GET_ITEM2, 0),
            (EV_GET_PHOTO, 0),
            (EV_GALLERY, 0),
            (EV_DATE_UB4, 2),
        ];
        for (slot, kind) in watched {
            if !ctx.event_ended(self.event_index(slot)) {
                continue;
            }
            self.request |= REQUEST_LINK_BACK;
            match kind {
                1 => {
                    self.demo_talk = true;
                    self.order = ORDER_IDLE;
                    self.talking = false;
                    self.execute_set_mode(Step::Wait, Vec3::ZERO, 0);
                }
                2 => {
                    self.request |= REQUEST_UB4_DONE;
                    self.eye_on = true;
                    self.back_on = true;
                }
                _ => {}
            }
            return;
        }
        self.private_cut(ctx);
        let (info, stories) = (&mut *ctx.text.info, &*ctx.stories);
        self.set_anm_from_msg_tag(info, stories);
    }

    fn private_cut(&mut self, ctx: &mut Ctx<'_>) {
        let Some(staff) = ctx.staff() else {
            return;
        };
        let idx = self.event_index(EV_COUNTER_TALK0);
        if idx < 0 {
            return;
        }
        let Some(cut) = ctx.active.cut_of(usize::from(idx as u16), staff) else {
            return;
        };
        let Some(row) = ctx.list.cuts.get(cut.max(0) as usize) else {
            return;
        };
        let Some(act) = CUT_NAMES.iter().position(|name| *name == row.name) else {
            return;
        };
        let act = act as i8;
        self.act_idx = act;
        if cut != self.cut_seen {
            self.cut_seen = cut;
            self.cut_init(act, ctx);
        }
        match act {
            CUT_MES_SET => {
                self.event_mes_set(ctx);
            }
            CUT_MES_SET_UB => {
                self.event_look_ub(ctx);
                self.event_mes_set(ctx);
            }
            CUT_LOOK_UB => {
                self.event_look_ub(ctx);
            }
            CUT_SET_EYE => {
                self.event_set_eye(ctx);
            }
            CUT_TURN_TO_PLAYER => {
                self.event_turn_to_player(ctx);
            }
            CUT_GET_PHOTO => {
                self.event_get_photo();
            }
            _ => {}
        }
    }

    fn cut_init(&mut self, act: i8, ctx: &mut Ctx<'_>) {
        match act {
            CUT_MES_SET | CUT_MES_SET_UB => self.event_mes_set_init(ctx),
            CUT_SE_SET => ctx.events.list.push(SimEvent::Se(SE_UTSUSHIE_TO_DX)),
            CUT_POS_SET => self.event_pos_set_init(ctx),
            CUT_GET_ITEM => {
                self.pending_present = Some(self.item_no);
            }
            CUT_SET_ANGLE => self.event_set_angle_init(ctx),
            CUT_SET_EYE => self.eye_timer = 0,
            CUT_TURN_TO_PLAYER => self.event_turn_to_player_init(ctx),
            CUT_CLR_HANME => {
                let stories = &*ctx.stories;
                self.init_tex_pattern_anm(stories, true, -1);
                self.anm_flags &= 0x7F;
            }
            CUT_GET_PHOTO => self.event_get_photo_init(ctx.weekday),
            _ => {}
        }
    }

    fn event_mes_set_init(&mut self, ctx: &mut Ctx<'_>) {
        let cut = self.cut_seen;
        let key = ctx.list.cut_number(cut.max(0) as usize, "MsgNo");
        match key {
            Some(msg_no) => {
                self.drop_message();
                let msg_no = msg_no as u32;
                match msg_no {
                    0 => {
                        if !ctx.stories.test(SAVE_ORDER_PROGRESS) {
                            self.msg_tbl = Some(&MSG_FIRST_TALK_PHOTO[..]);
                            self.cam_tbl = None;
                            ctx.stories.set(SAVE_ORDER_PROGRESS);
                        } else {
                            let tag = ctx.tag_photo.map_or(0, |(no, _)| no);
                            self.msg_tbl =
                                MSG_TALK_PHOTO.get(usize::from(tag)).map(|row| &row[..]);
                            self.cam_tbl = None;
                        }
                        let msg = self.message_at();
                        self.set_message(msg);
                    }
                    1 => {
                        let msg = self.get_msg(ctx);
                        self.set_message(msg);
                    }
                    99 => {
                        ctx.events.list.push(SimEvent::Se(SE_UTSUSHIE_TO_DX));
                        self.bump_message();
                        let msg = self.message_at();
                        self.set_message(msg);
                    }
                    other => self.set_message(other),
                }
            }
            None => {
                self.bump_message();
                let msg = self.message_at();
                self.set_message(msg);
            }
        }
    }

    fn event_mes_set(&mut self, ctx: &mut Ctx<'_>) {
        self.talk2(ctx, 0);
    }

    fn event_pos_set_init(&mut self, ctx: &mut Ctx<'_>) {
        let angle = match ctx.tag_photo {
            Some((tag, at)) => {
                let mut pos = ctx.path_point(tag).unwrap_or(ctx.actor_pos);
                let probe = pos + Vec3::new(0.0, PHOTO_PROBE, 0.0);
                if let Some(floor) = ctx.ground_y(probe) {
                    pos.y = floor;
                }
                ctx.set_pos(pos);
                dis_xz_ang_y(pos, at).1
            }
            None => {
                ctx.set_pos(GALLERY_POS);
                dis_xz_ang_y(GALLERY_POS, ctx.player_pos).1
            }
        };
        ctx.set_angle(angle);
        self.home_angle = angle;
        self.path_idx = ctx.path_count().saturating_sub(2);
        self.path_dir = 0;
        self.execute_set_mode(Step::Wait, ctx.actor_pos, ctx.actor_angle);
        self.kind = 1;
        let mut slot = (self.rnd() * 3.0) as usize;
        if slot == 3 {
            slot = 0;
        }
        self.rng_msg_no = MSG_2F.get(slot).copied().map_or(0, i32::from);
    }

    fn event_set_angle_init(&mut self, ctx: &Ctx<'_>) {
        let delta = ctx.actor_pos - ctx.player_pos;
        self.link_move_angle = walk::to_s16(delta.x.atan2(delta.z));
    }

    fn event_set_eye(&mut self, ctx: &Ctx<'_>) {
        let swing = walk::from_s16(self.eye_timer).sin() * EYE_ORBIT_SWING;
        let yaw = ((ctx.player_face) as f32 + swing) as i16;
        self.eye_timer = self.eye_timer.wrapping_add(EYE_ORBIT_RATE);
        let turn = Quat::from_rotation_y(walk::from_s16(yaw))
            * Quat::from_rotation_x(walk::from_s16(EYE_ORBIT_TILT));
        self.eye_pos = turn * Vec3::new(0.0, 0.0, EYE_ORBIT) + ctx.player_eye;
    }

    fn event_turn_to_player_init(&mut self, ctx: &Ctx<'_>) {
        self.home_angle = dis_xz_ang_y(ctx.actor_pos, ctx.player_pos).1;
    }

    fn event_turn_to_player(&mut self, ctx: &Ctx<'_>) -> bool {
        ctx.actor_angle == self.home_angle
    }

    fn event_get_photo_init(&mut self, weekday: u32) {
        self.picture_get_day = weekday as u8;
        self.eye_timer = PHOTO_WAIT;
    }

    fn event_get_photo(&mut self) -> bool {
        self.eye_timer -= 1;
        self.eye_timer == 0
    }

    fn event_look_ub(&mut self, ctx: &Ctx<'_>) {
        let Some(eye) = ctx.ub4_eye else {
            return;
        };
        self.look_at = eye;
        self.look_mode = LOOK_POINT;
        self.head_blend = false;
    }

    pub fn talk2(&mut self, ctx: &mut Ctx<'_>, param: i32) -> u16 {
        let mut status = NO_STATUS;
        let yaw = ctx.actor_angle;
        if self.msg_id == FPC_M_ERROR_PROCESS_ID {
            if param == 1 {
                self.curr_msg_no = self.get_msg(ctx);
            }
            let at = self.attn_pos;
            self.msg_id = ctx.message_set(self.curr_msg_no, at);
            self.msg_live = false;
            self.cam_armed = 1;
            self.set_msg_camera(yaw);
        } else if self.msg_live {
            status = ctx.text.status();
            match status {
                STATUS_MSG_DISPLAYED => {
                    let next = self.next_msg_status(ctx);
                    ctx.text.set_status(next);
                    if next == STATUS_MSG_CONTINUES {
                        let msg = self.curr_msg_no;
                        ctx.message_set(msg, Vec3::ZERO);
                    }
                }
                STATUS_TYPING => {
                    if self.cam_armed != 0 {
                        self.cam_armed = 0;
                        self.set_msg_camera(yaw);
                    }
                }
                STATUS_BOX_CLOSED => {
                    self.cam_paused = false;
                    if self.request & REQUEST_PRICE != 0 {
                        self.request &= !REQUEST_PRICE;
                        self.request |= REQUEST_LINK_BACK;
                    }
                    ctx.text.set_status(STATUS_MSG_DESTROYED);
                    self.msg_id = FPC_M_ERROR_PROCESS_ID;
                }
                _ => {}
            }
            if status != STATUS_TYPING {
                self.cam_armed = 1;
            }
        } else {
            self.msg_live = ctx.text.procs.search_by_id(self.msg_id).is_some();
        }
        u16::from(status)
    }

    fn set_msg_camera(&mut self, yaw: i16) {
        if self.cam_tbl.is_none() {
            return;
        }
        let index = self.camera_at();
        if index < 0 {
            self.cam_paused = false;
            return;
        }
        let Some(row) = MSG_CAMERA.get(index as usize) else {
            return;
        };
        let base = self.attn_pos;
        self.cam_eye = turned_y(row[0], yaw) + base;
        self.cam_look = turned_y(row[1], yaw) + base;
        self.cam_paused = true;
    }

    pub fn next_msg_status(&mut self, ctx: &mut Ctx<'_>) -> u8 {
        match self.curr_msg_no {
            0x378A | 0x378B => {
                self.request |= REQUEST_PRICE;
                STATUS_MSG_ENDS
            }
            MSG_CHOOSE_ASK | MSG_CHOOSE_ASK_2 => {
                if ctx.text.select_num() != 0 {
                    self.curr_msg_no = MSG_CHOOSE_TAIL;
                    self.drop_message();
                    STATUS_MSG_CONTINUES
                } else {
                    self.advance_table(ctx)
                }
            }
            _ => self.advance_table(ctx),
        }
    }

    fn advance_table(&mut self, ctx: &mut Ctx<'_>) -> u8 {
        if self.msg_tbl.is_none() {
            return STATUS_MSG_ENDS;
        }
        self.bump_message();
        match self.message_at() {
            7 | 8 | 11 => STATUS_MSG_ENDS,
            9 => {
                if ctx.text.select_num() == 0 {
                    let price = ctx.text.info.message_rupee;
                    if i64::from(ctx.rupees.count) < i64::from(price) {
                        self.curr_msg_no = MSG_NO_FUNDS;
                    } else {
                        self.bump_message();
                        self.curr_msg_no = self.message_at();
                        ctx.rupees.count = ctx.rupees.count.saturating_sub(price.max(0) as u32);
                        self.tmp_bought_film = true;
                    }
                } else {
                    self.msg_tbl = None;
                    self.curr_msg_no = MSG_ANSWER_BUY;
                }
                STATUS_MSG_CONTINUES
            }
            10 => STATUS_MSG_ENDS,
            6 => {
                self.request |= REQUEST_ITEM_FILM;
                self.order_done = true;
                self.erase_requested = true;
                bump_order_reg(ctx);
                STATUS_MSG_CONTINUES
            }
            2 | 4 => {
                self.erase_requested = true;
                bump_order_reg(ctx);
                STATUS_MSG_CONTINUES
            }
            1 | 3 | 5 => {
                bump_order_reg(ctx);
                STATUS_MSG_CONTINUES
            }
            0 => {
                self.drop_message();
                STATUS_MSG_ENDS
            }
            other => {
                self.curr_msg_no = other;
                STATUS_MSG_CONTINUES
            }
        }
    }

    pub fn get_msg(&mut self, ctx: &mut Ctx<'_>) -> u32 {
        let mut msg_no = MSG_TAIL;
        let order_reg = Self::order_reg(ctx.stories);
        self.msg_tbl = None;
        self.cam_tbl = None;
        self.msg_at = 0;
        self.cam_at = 0;
        if self.in_photo {
            msg_no = self.photo_msg(ctx, order_reg);
        } else if ctx.xy_active {
            msg_no = self.xy_msg(ctx, order_reg);
        } else if self.kind == STAGE_DESCEND {
            self.msg_tbl = Some(&MSG_DOWN[..]);
        } else if self.kind == STAGE_UB4 {
            self.msg_tbl = Some(&MSG_UB4[..]);
        } else if ctx.stories.test(SAVE_ORDER_PROGRESS) {
            msg_no = if self.kind == 1 {
                self.rng_msg_no as u32
            } else {
                self.progress_msg(ctx)
            };
        } else if ctx.got.has(GOT_PICTO_BOX_SLOT, GOT_BIT)
            || ctx.got.has(GOT_DELUXE_SLOT, GOT_BIT)
        {
            if ctx.stories.test(SAVE_POST_TALK) {
                self.msg_tbl = Some(&MSG_SECOND_PHOTO[..]);
            } else {
                self.msg_tbl = Some(&MSG_FIRST_PHOTO[..]);
                ctx.stories.set(SAVE_POST_TALK);
                self.init_tex_pattern_anm(ctx.stories, true, 1);
            }
        } else if !ctx.stories.test(SAVE_TALKED_ONCE) {
            self.msg_tbl = Some(&MSG_FIRST_TALK[..]);
            ctx.stories.set(SAVE_TALKED_ONCE);
            self.request |= REQUEST_SPOIT;
        } else {
            self.msg_tbl = Some(&MSG_SECOND_TALK[..]);
            self.request |= REQUEST_SPOIT;
        }
        if self.msg_tbl.is_some() {
            msg_no = self.message_at();
        }
        msg_no
    }

    fn photo_msg(&mut self, ctx: &Ctx<'_>, order_reg: u8) -> u32 {
        if order_reg < 1 {
            return MSG_GIVE_UP;
        }
        if order_reg < 3 {
            if order_reg == 2 {
                return MSG_RETRY_2;
            }
            if self.is_photo_ok(ctx.stories) {
                self.msg_tbl = Some(&MSG_FIRST_ORDER_XY[..]);
            } else {
                return MSG_RETRY;
            }
        } else if order_reg < 5 {
            if order_reg == 4 {
                return MSG_RETRY_2;
            }
            if self.is_photo_ok(ctx.stories) {
                self.msg_tbl = Some(&MSG_SECOND_ORDER_XY[..]);
            } else {
                return MSG_RETRY;
            }
        } else if order_reg < ORDER_DONE_ALL {
            if self.is_photo_ok(ctx.stories) {
                self.msg_tbl = Some(&MSG_THIRD_ORDER_XY[..]);
            } else {
                return MSG_RETRY;
            }
        } else {
            return MSG_FINISHED;
        }
        MSG_TAIL
    }

    fn xy_msg(&mut self, ctx: &Ctx<'_>, order_reg: u8) -> u32 {
        if ctx.xy_item == ITEM_DELUXE_PICTO_BOX && self.tmp_took_photo {
            if self.pictures < PHOTO_LIMIT {
                self.msg_tbl = Some(&MSG_XY_BUY_PHOTO[..]);
                self.item_no = ITEM_LEGENDARY_PICTOGRAPH;
            } else {
                return MSG_FULL;
            }
        } else if ctx.xy_item == ITEM_FIREFLY_BOTTLE {
            if order_reg >= ORDER_DONE_ALL && !ctx.got.has(GOT_DELUXE_SLOT, GOT_BIT) {
                self.msg_tbl = Some(&MSG_COLOR_XY[..]);
                self.item_no = ITEM_DELUXE_PICTO_BOX;
                self.bottle_spent = true;
            } else {
                return MSG_RETRY_2;
            }
        } else {
            return MSG_UNRELATED;
        }
        MSG_TAIL
    }

    fn progress_msg(&mut self, ctx: &mut Ctx<'_>) -> u32 {
        if ctx.got.has(GOT_DELUXE_SLOT, GOT_BIT) {
            if self.tmp_bought_film {
                self.msg_tbl = Some(&MSG_BUY_PHOTO[..]);
            } else if self.tmp_took_photo {
                self.msg_tbl = Some(&MSG_GET_PHOTO[..]);
            } else if self.is_photo_dx_ok(ctx.stories, ctx.weekday) {
                self.tmp_took_photo = true;
                if !ctx.stories.test(SAVE_PHOTO_DX_DONE) {
                    ctx.stories.set(SAVE_PHOTO_DX_DONE);
                    self.msg_tbl = Some(&MSG_WEEK_FIRST[..]);
                } else {
                    self.msg_tbl = MSG_WEEK.get(ctx.weekday as usize).map(|row| &row[..]);
                }
            } else {
                self.msg_tbl = Some(&MSG_COLOR[..]);
                self.cam_tbl = Some(&MSG_COLOR_CAM[..]);
            }
            return MSG_TAIL;
        }
        let order_reg = Self::order_reg(ctx.stories);
        if order_reg < 1 {
            self.msg_tbl = Some(&MSG_FIRST_ORDER[..]);
            self.cam_tbl = Some(&MSG_FIRST_ORDER_CAM[..]);
        } else if order_reg < 2 {
            self.msg_tbl = Some(&MSG_FIRST_ORDER_NOT_END[..]);
            self.cam_tbl = Some(&MSG_FIRST_ORDER_NOT_END_CAM[..]);
        } else if order_reg < 3 {
            self.msg_tbl = Some(&MSG_SECOND_ORDER[..]);
            self.cam_tbl = Some(&MSG_SECOND_ORDER_CAM[..]);
        } else if order_reg < 4 {
            self.msg_tbl = Some(&MSG_SECOND_ORDER_NOT_END[..]);
            self.cam_tbl = Some(&MSG_SECOND_ORDER_NOT_END_CAM[..]);
        } else if order_reg < 5 {
            self.msg_tbl = Some(&MSG_THIRD_ORDER[..]);
            self.cam_tbl = Some(&MSG_THIRD_ORDER_CAM[..]);
        } else if order_reg < ORDER_DONE_ALL {
            self.msg_tbl = Some(&MSG_THIRD_ORDER_NOT_END[..]);
            self.cam_tbl = Some(&MSG_THIRD_ORDER_NOT_END_CAM[..]);
        } else if self.order_done {
            self.msg_tbl = Some(&MSG_THIRD_ORDER_END[..]);
        } else {
            self.msg_tbl = Some(&MSG_NOT_COLOR[..]);
            self.cam_tbl = Some(&MSG_NOT_COLOR_CAM[..]);
        }
        MSG_TAIL
    }

    pub fn set_message(&mut self, msg: u32) {
        self.msg_id = FPC_M_ERROR_PROCESS_ID;
        self.curr_msg_no = msg;
    }

    pub fn xy_check_cb(&mut self) -> i16 {
        if self.tmp_took_photo && !self.tmp_bought_film {
            self.attn_flags = ATTN_BUY;
        } else {
            self.attn_flags = ATTN_FULL;
        }
        1
    }

    pub fn xy_event_cb(&mut self, ctx: &Ctx<'_>, item_no: u8) -> i16 {
        if item_no == ITEM_DELUXE_PICTO_BOX && self.tmp_took_photo && self.pictures < PHOTO_LIMIT {
            self.demo_talk = false;
            self.event_index(EV_GET_PHOTO)
        } else if item_no == ITEM_FIREFLY_BOTTLE
            && Self::order_reg(ctx.stories) >= ORDER_DONE_ALL
            && !ctx.got.has(GOT_DELUXE_SLOT, GOT_BIT)
        {
            self.demo_talk = false;
            self.event_index(EV_GET_ITEM2)
        } else {
            ctx.event_index(EVENT_DEFAULT_TALK_XY)
        }
    }

    pub fn chk_attention(&mut self, actor: &Actor, ctx: &Ctx<'_>) {
        let mut dist = NPC_ATTN_DIST;
        let mut limit_units = i32::from(self.attn_range);
        let (range, angle) = dis_xz_ang_y(actor.pos, ctx.player_pos);
        if self.attention {
            dist += NPC_ATTN_DIST_BONUS;
            limit_units += NPC_ATTN_ANGLE_BONUS;
        }
        let delta = angle.wrapping_sub(actor.angle[1]);
        if (dist > range && limit_units > i32::from(delta.unsigned_abs())) || self.counter_hit {
            self.look_at = ctx.player_eye + Vec3::new(0.0, NPC_PLAYER_EYE_LIFT, 0.0);
            self.look_mode = LOOK_POINT;
            if self.eye_on {
                self.head_blend = false;
                self.turn_ready = true;
            } else {
                self.head_blend = true;
            }
            if !self.back_on {
                self.desired_angle = self.home_angle;
                self.head_blend = false;
                self.look_mode = LOOK_ANGLE;
                self.turn_ready = true;
            }
            if !self.attention {
                self.attention = true;
            }
        } else {
            if self.attention {
                self.attention = false;
                self.attn_release = NPC_ATTN_RELEASE;
            }
            self.look_mode = LOOK_NONE;
            if self.step == Step::Wait && self.kind != STAGE_DESCEND {
                if self.attn_release != 0 {
                    self.attn_release -= 1;
                } else {
                    self.desired_angle = self.home_angle;
                    self.head_blend = false;
                    self.look_mode = LOOK_ANGLE;
                    self.turn_ready = true;
                }
            }
        }
        self.turn_target = NPC_TURN_TARGET;
    }

    pub fn look_back(&mut self, actor: &mut Actor) {
        let target = self.turn_target;
        let mut desired = actor.angle[1];
        let mut dst: Option<Vec3> = None;
        let src = self.eye_pos;
        match self.look_mode {
            LOOK_POINT => dst = Some(self.look_at),
            LOOK_ANGLE => desired = self.desired_angle,
            _ => {}
        }
        let mut blend = self.head_blend;
        if self.talking && self.eye_on {
            blend = false;
            self.turn_ready = true;
        }
        if self.turn_ready {
            stream::add_calc_angle_s2(&mut self.turn_rate, target, TURN_RATE_SCALE, TURN_RATE_MAX);
        } else {
            self.turn_rate = 0;
        }
        self.jnt_look_at_target(actor, dst, src, desired, self.turn_rate, blend);
    }

    fn jnt_look_at_target(
        &mut self,
        actor: &mut Actor,
        dst: Option<Vec3>,
        src: Vec3,
        default_y: i16,
        max_vel: i16,
        blend: bool,
    ) {
        let (target_y, target_x) = match dst {
            Some(at) => {
                let delta = at - src;
                let flat = (delta.x * delta.x + delta.z * delta.z).sqrt();
                (
                    walk::to_s16(delta.x.atan2(delta.z)),
                    walk::to_s16(delta.y.atan2(flat)),
                )
            }
            None => (default_y, 0),
        };
        let delta_y = target_y.wrapping_sub(actor.angle[1]);
        let max_y = i32::from(jnt_max(1, 1)) + i32::from(jnt_max(0, 1));
        let min_y = i32::from(jnt_min(1, 1)) + i32::from(jnt_min(0, 1));
        let reached = self.jnt_move(delta_y, 1);
        let spread = i32::from(delta_y) >= max_y || i32::from(delta_y) <= min_y;
        if (self.turn_ready || (spread && reached)) && !blend {
            self.turn_ready = self.jnt_follow(&mut actor.angle[1], target_y, max_vel);
        }
        self.jnt_move(target_x, 0);
    }

    fn jnt_move(&mut self, delta: i16, axis: usize) -> bool {
        let mut rest = delta;
        let mut angles = [0_i16; 2];
        for row in 0..2 {
            let mut angle = rest;
            angle = limit(angle, jnt_max(row, axis), jnt_min(row, axis));
            rest = rest.wrapping_sub(angle);
            angles[row] = angle;
            let held = self.jnt[row][axis];
            self.jnt[row][axis] = add_calc_angle_l(held, angle);
        }
        let mut done = 0_i32;
        for row in 0..2 {
            let gap = i32::from(angle_distance(angles[row], self.jnt[row][axis]));
            if gap <= i32::from(JNT_MAX_STEP) / 2 {
                done += 1;
            }
        }
        done == 2
    }

    fn jnt_follow(&mut self, out_y: &mut i16, target_y: i16, max_vel: i16) -> bool {
        let orig = *out_y;
        let mut diff = out_y.wrapping_sub(target_y);
        let orig_diff = diff;
        diff = walk::add_calc_angle_s(diff, 0, JNT_FOLLOW_SCALE, max_vel, JNT_FOLLOW_MIN);
        if i32::from(diff.unsigned_abs()) > i32::from(JNT_FOLLOW_SCALE) {
            *out_y = out_y.wrapping_add(diff.wrapping_sub(orig_diff));
        } else {
            *out_y = target_y;
        }
        let delta_y = out_y.wrapping_sub(orig);
        let mut sum = 0_i16;
        let mut acc = 0_i16;
        let left = target_y.wrapping_sub(*out_y);
        for row in (0..2).rev() {
            sum = sum.wrapping_add(self.jnt[row][1]);
            if (delta_y > 0 && sum > left) || (delta_y < 0 && sum < left) {
                self.jnt[row][1] = limit(left.wrapping_sub(acc), jnt_max(row, 1), jnt_min(row, 1));
            }
            acc = acc.wrapping_add(self.jnt[row][1]);
        }
        target_y != *out_y
    }
}

pub fn is_lenzo(name: &str) -> bool {
    name == LENZO_ACTOR
}

fn jnt_max(row: usize, axis: usize) -> i16 {
    match (row, axis) {
        (0, 0) => JNT_HEAD_MAX_X,
        (0, _) => JNT_HEAD_MAX_Y,
        (_, 0) => JNT_SPINE_MAX_X,
        _ => JNT_SPINE_MAX_Y,
    }
}

fn jnt_min(row: usize, axis: usize) -> i16 {
    match (row, axis) {
        (0, 0) => JNT_HEAD_MIN_X,
        (0, _) => JNT_HEAD_MIN_Y,
        (_, 0) => JNT_SPINE_MIN_X,
        _ => JNT_SPINE_MIN_Y,
    }
}

fn limit(value: i16, max: i16, min: i16) -> i16 {
    if value > max {
        return max;
    }
    if value < min {
        return min;
    }
    value
}

fn angle_distance(a: i16, b: i16) -> u16 {
    a.wrapping_sub(b).unsigned_abs()
}

fn add_calc_angle_l(value: i16, target: i16) -> i16 {
    let mut held = i32::from(value);
    let goal = i32::from(target);
    if held != goal {
        let diff = goal - held;
        let mut step = diff / i32::from(JNT_MOVE_SCALE);
        if step > i32::from(JNT_MOVE_MIN) || step < -i32::from(JNT_MOVE_MIN) {
            step = step.clamp(-i32::from(JNT_MAX_STEP), i32::from(JNT_MAX_STEP));
            held += step;
        } else if diff >= 0 {
            held += i32::from(JNT_MOVE_MIN);
            if goal - held <= 0 {
                held = goal;
            }
        } else {
            held -= i32::from(JNT_MOVE_MIN);
            if goal - held >= 0 {
                held = goal;
            }
        }
    }
    held as i16
}

fn dis_xz_ang_y(from: Vec3, to: Vec3) -> (f32, i16) {
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    ((dx * dx + dz * dz).sqrt(), walk::to_s16(dx.atan2(dz)))
}

fn turned_y(offset: Vec3, yaw: i16) -> Vec3 {
    Quat::from_rotation_y(walk::from_s16(yaw)) * offset
}

fn inside_cylinder(centre: Vec3, radius: f32, at: Vec3) -> bool {
    let dx = at.x - centre.x;
    let dz = at.z - centre.z;
    dx * dx + dz * dz <= radius * radius
}

fn in_talk_range(pos: Vec3, player: &Player) -> bool {
    let dx = player.pos.x - pos.x;
    let dz = player.pos.z - pos.z;
    dx * dx + dz * dz <= TALK_REACH * TALK_REACH
}

fn bump_order_reg(ctx: &mut Ctx<'_>) {
    let reg = Lenzo::order_reg(ctx.stories).wrapping_add(1);
    ctx.stories.set_reg(SAVE_ORDER_PROGRESS, reg);
}

pub struct Text<'a> {
    procs: &'a mut MsgProcs,
    info: &'a mut MsgGameInfo,
    one: &'a mut Msg1Box,
    two: &'a mut Msg2Box,
}

impl Text<'_> {
    fn is_two(&self) -> bool {
        self.procs.search_by_name(FPC_NM_MSG2).is_some()
    }

    fn status(&self) -> u8 {
        if self.is_two() {
            self.two.status
        } else {
            self.one.status
        }
    }

    fn set_status(&mut self, status: u8) {
        if self.is_two() {
            self.two.status = status;
        } else {
            self.one.status = status;
        }
    }

    fn select_num(&self) -> u8 {
        if self.is_two() {
            0
        } else {
            self.one.select_num
        }
    }
}

pub struct Ctx<'a> {
    entity: Entity,
    stories: &'a mut StoryBits,
    got: &'a Got,
    rupees: &'a mut Rupees,
    events: &'a mut SimEvents,
    list: &'a EventList,
    active: &'a mut ActiveEvent,
    text: Text<'a>,
    archive: &'a dyn MsgArchive,
    player: Option<Vec3>,
    player_eye: Vec3,
    player_face: i16,
    player_pos: Vec3,
    actor_pos: Vec3,
    actor_angle: i16,
    start_stage_point: i16,
    tag_photo: Option<(u8, Vec3)>,
    ub4_eye: Option<Vec3>,
    xy_active: bool,
    xy_item: u8,
    present_end: bool,
    path_points: &'static [Vec3],
    ground: Option<f32>,
    weekday: u32,
    talk_command: bool,
    demo_accept: bool,
    anim_done: bool,
}

impl Ctx<'_> {
    fn running(&self) -> bool {
        self.text.info.event_run_check()
    }

    fn message_set(&mut self, msg_no: u32, at: Vec3) -> u32 {
        fop_msgm_message_set(
            self.text.procs,
            self.text.info,
            self.archive,
            MSG_CLEAR_COUNT,
            msg_no,
            None,
            at,
        )
    }

    fn order_event(&mut self, index: i16) {
        if index < 0 {
            return;
        }
        let slot = usize::from(index as u16);
        if self.active.stage_of(slot).is_none() {
            self.active.order(self.list, slot, self.entity);
        }
    }

    fn event_ended(&self, index: i16) -> bool {
        index >= 0 && self.active.end_check(usize::from(index as u16))
    }

    fn staff(&self) -> Option<usize> {
        self.list.staff_of(LENZO_ACTOR)
    }

    fn event_index(&self, name: &str) -> i16 {
        self.list
            .event_index(name)
            .and_then(|index| i16::try_from(index).ok())
            .unwrap_or(NO_INDEX)
    }

    fn path_point(&self, tag: u8) -> Option<Vec3> {
        self.path_points.get(usize::from(tag)).copied()
    }

    fn path_count(&self) -> u8 {
        u8::try_from(self.path_points.len()).unwrap_or(0xFF)
    }

    fn ground_y(&self, _probe: Vec3) -> Option<f32> {
        self.ground
    }

    fn set_pos(&mut self, pos: Vec3) {
        self.actor_pos = pos;
    }

    fn set_angle(&mut self, angle: i16) {
        self.actor_angle = angle;
    }
}

pub fn control(
    do_intent: Res<DoIntent>,
    facts: (Res<Clock>, Res<Got>),
    archive: Option<Res<MsgArchives>>,
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut stories: ResMut<StoryBits>,
    mut rupees: ResMut<Rupees>,
    mut procs: ResMut<MsgProcs>,
    mut info: ResMut<MsgGameInfo>,
    mut one: ResMut<Msg1Box>,
    mut two: ResMut<Msg2Box>,
    mut events: ResMut<SimEvents>,
    players: Query<&Player>,
    mut lenses: Query<(Entity, &mut Actor, &mut Lenzo), Without<Player>>,
    others: Query<&Actor, Without<Lenzo>>,
) {
    let (clock, got) = &facts;
    let Some(archive) = archive.as_deref() else {
        return;
    };
    let weekday = clock.weekday();
    let player = players.iter().next().copied();
    let player_pos = player.map_or(Vec3::ZERO, |link| link.pos);
    let player_face = player.map_or(0, |link| walk::to_s16(link.face));
    let mut tag_photo = None;
    let mut ub4_eye = None;
    for actor in others.iter() {
        if actor.name == TAG_PHOTO_ACTOR {
            let tag = ((actor.parameters >> PRM_TAG_NO_SHIFT) & PRM_TAG_NO_MASK) as u8;
            tag_photo = Some((tag, actor.pos));
        } else if actor.name == UB4_ACTOR {
            ub4_eye = Some(actor.pos + Vec3::new(0.0, NPC_EYE_LIFT, 0.0));
        }
    }
    let pressed = do_intent.0;
    let running = info.event_run_check();

    for (entity, mut actor, mut lenz) in &mut lenses {
        let actor = &mut *actor;
        let lenz = &mut *lenz;
        if !lenz.entered {
            lenz.entered = true;
            lenz.home_angle = actor.angle[1];
            if !lenz.phase1(&got) {
                continue;
            }
            lenz.create_init(&mut stories, &list);
        }
        let talk_command = pressed && player.is_some_and(|link| in_talk_range(actor.pos, &link));
        let mut ctx = Ctx {
            entity,
            stories: &mut stories,
            got: &got,
            rupees: &mut rupees,
            events: &mut events,
            list: &list,
            active: &mut active,
            text: Text {
                procs: &mut procs,
                info: &mut info,
                one: &mut one,
                two: &mut two,
            },
            archive,
            player: player.map(|link| link.pos),
            player_eye: player_pos,
            player_face,
            player_pos,
            actor_pos: actor.pos,
            actor_angle: actor.angle[1],
            start_stage_point: lenz.start_stage_point,
            tag_photo,
            ub4_eye,
            xy_active: lenz.xy_active,
            xy_item: lenz.xy_item,
            present_end: lenz.present_end,
            path_points: &NO_POINTS,
            ground: None,
            weekday,
            talk_command,
            demo_accept: running,
            anim_done: false,
        };
        lenz.execute(actor, &mut ctx);
        actor.pos = ctx.actor_pos;
        actor.angle[1] = ctx.actor_angle;
    }
}