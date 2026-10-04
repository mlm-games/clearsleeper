use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::{Quat, Vec3};

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{Collect, StoryBits};
use super::frame::CurrentFrame;
use super::inventory::Equipped;
use super::player::Player;
use super::walk::{add_calc, add_calc_angle_s, from_s16, to_s16};
use crate::assets::MsgArchives;
use crate::text::driver::TextFlags;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    DA_PY_STTS0_TELESCOPE_LOOK, FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgFlags,
    MsgGameInfo, MsgProcs, fop_msgm_message_set, fop_msgm_message_set_actor,
    fop_msgm_scope_message_set,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_BOX_CLOSING, STATUS_BOX_OPENING, STATUS_CLOSE_WAIT,
    STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED, STATUS_MSG_ENDS,
    STATUS_MSG_TYPING, STATUS_SCOPE_WAIT, STATUS_UNK0,
};

pub const ARYLL_ACTOR: &str = "Ls1";
pub const PARTNER_ACTOR: &str = "Bm1";

pub const EVENT_NAMES: [&str; 4] = ["zelda_fly", "omedeto", "get_telescope", "eTalk"];
pub const EVENT_ZELDA_FLY: usize = 0;
pub const EVENT_OMEDETO: usize = 1;
pub const EVENT_GET_TELESCOPE: usize = 2;
pub const EVENT_ETALK: usize = 3;
pub const EVENT_ID_COUNT: usize = 4;

const _: () = assert!(EVENT_NAMES.len() == EVENT_ETALK + 1);

pub const CUT_LOK_PLYER: i8 = 0;
pub const CUT_PLYER_MOV: i8 = 1;
pub const CUT_WAI: i8 = 2;
pub const CUT_ANM_CHG: i8 = 3;

const PRM_SHIFT: u32 = 0x00;
const PRM_MASK: u32 = 0xFF;

pub const HIO_ATT_POS_OFFSET_Y: f32 = 110.0;
pub const HIO_TELESCOPE_RANGE: f32 = 700.0;
pub const HIO_WATCH_RANGE: f32 = 200.0;
pub const HIO_ZOOM_BIRD: f32 = 2.0;
pub const HIO_ZOOM_FOCUS: f32 = 9.0;
pub const HIO_SPH_RANGE: f32 = 50.0;
pub const HIO_EYE_SCALE_DENOM: i16 = 0x1A2C;
pub const HIO_PLAYER_EYE_OFFSET_Y: f32 = 9.0;

pub const EYE_MULT: f32 = 0.1;
pub const EYE_MIN: f32 = -1.0;
pub const EYE_MAX: f32 = 1.0;
pub const EYE_CALC_SCALE: f32 = 0.5;
pub const EYE_CALC_MAX_STEP: f32 = 0.1;
pub const EYE_CALC_MIN_STEP: f32 = 0.03;
pub const IS_NEAR_ZERO_SQUARE: f32 = 8e-11;
pub const MAT_ANM_SLOTS: usize = 2;

pub const LOOP_NULL: i32 = -1;
pub const LOOP_NONE: i32 = 0;
pub const LOOP_LOOP: i32 = 2;

pub const ANMNUM_WAIT01: i32 = 0;
pub const ANMNUM_WAIT02: i32 = 1;
pub const ANMNUM_WAIT05: i32 = 4;
pub const ANMNUM_WAIT06: i32 = 5;
pub const ANMNUM_GET: i32 = 9;
pub const ANMNUM_WAIT07: i32 = 10;
pub const ANMNUM_DEMOWAIT: i32 = 12;

pub const BCKNUM_WAIT01: i8 = 0;
pub const BCKNUM_WAIT02: i8 = 1;
pub const BCKNUM_WAIT03: i8 = 2;
pub const BCKNUM_WAIT04: i8 = 3;
pub const BCKNUM_WAIT05: i8 = 4;
pub const BCKNUM_WAIT06: i8 = 5;
pub const BCKNUM_WATASU: i8 = 6;
pub const BCKNUM_FURIMUKI: i8 = 7;
pub const BCKNUM_NOZOKU: i8 = 8;
pub const BCKNUM_GET: i8 = 9;
pub const BCKNUM_WAIT07: i8 = 10;
pub const BCKNUM_TALK01: i8 = 11;
pub const BCKNUM_DEMOWAIT: i8 = 12;
pub const BCKNUM_DEMOLOOK: i8 = 13;
pub const BCKNUM_NULL: i8 = 14;

pub const BTPNUM_FUAN: i8 = 0;
pub const BTPNUM_MABA: i8 = 1;
pub const BTPNUM_FUAN02: i8 = 2;
pub const BTPNUM_KIZUKU: i8 = 3;
pub const BTPNUM_GET: i8 = 4;
pub const BTPNUM_NGWARAI: i8 = 5;
pub const BTPNUM_NOZOKU: i8 = 6;
pub const BTPNUM_WARAI: i8 = 8;
pub const BTPNUM_DEMOWAIT: i8 = 10;

pub const BTKNUM_DEFAULT: i8 = 0;
pub const BTKNUM_DEMOLOOK: i8 = 1;
pub const BTKNUM_DEMOWAIT: i8 = 2;

pub const STAGE_INIT: i8 = 0;
pub const STAGE_PENDING: i8 = 1;
pub const STAGE_SEARCHED: i8 = 2;
pub const STAGE_RUN: i8 = 3;
pub const STAGE_TEARDOWN: i8 = 9;

pub const ORDER_NONE: i8 = 0;
pub const ORDER_SPEAK: i8 = 1;
pub const ORDER_CANTALK: i8 = 2;
pub const ORDER_EVENT: i8 = 3;
pub const ORDER_EVENT_1: i8 = 4;
pub const ORDER_EVENT_2: i8 = 5;

pub const STT_WAIT: i8 = 1;
pub const STT_SCOPE: i8 = 2;
pub const STT_POST: i8 = 3;
pub const STT_TELESCOPE: i8 = 4;
pub const STT_TALK: i8 = 5;

pub const LOOK_NONE: i8 = 0;
pub const LOOK_PLAYER: i8 = 1;
pub const LOOK_STORED: i8 = 2;
pub const LOOK_ANGLE: i8 = 3;
pub const LOOK_ACTOR: i8 = 4;

pub const SCOPE_PHASE_NONE: u8 = 0;
pub const SCOPE_PHASE_START: u8 = 1;
pub const SCOPE_PHASE_ZOOM: u8 = 2;
pub const SCOPE_PHASE_FOCUS: u8 = 3;
pub const SCOPE_PHASE_END: u8 = 4;

pub const MSG_BB9: u32 = 0xBB9;
pub const MSG_BBA: u32 = 0xBBA;
pub const MSG_BDA: u32 = 0xBDA;
pub const MSG_BBB: u32 = 0xBBB;
pub const MSG_BBC: u32 = 0xBBC;
pub const MSG_BBD: u32 = 0xBBD;
pub const MSG_BDC: u32 = 0xBDC;
pub const MSG_BBE: u32 = 0xBBE;
pub const MSG_BBF: u32 = 0xBBF;
pub const MSG_BC0: u32 = 0xBC0;
pub const MSG_BC1: u32 = 0xBC1;
pub const MSG_BC2: u32 = 0xBC2;
pub const MSG_BC3: u32 = 0xBC3;
pub const MSG_BC4: u32 = 0xBC4;
pub const MSG_BC5: u32 = 0xBC5;
pub const MSG_BC6: u32 = 0xBC6;
pub const MSG_BC7: u32 = 0xBC7;
pub const MSG_BC8: u32 = 0xBC8;
pub const MSG_BC9: u32 = 0xBC9;
pub const MSG_BCA: u32 = 0xBCA;
pub const MSG_BCB: u32 = 0xBCB;
pub const MSG_BCC: u32 = 0xBCC;
pub const MSG_BCD: u32 = 0xBCD;
pub const MSG_BD9: u32 = 0xBD9;

pub const EVENT_BIT_0001: u16 = 0x0001;
pub const EVENT_BIT_0280: u16 = 0x0280;
pub const EVENT_BIT_0310: u16 = 0x0310;
pub const EVENT_BIT_2A80: u16 = 0x2A80;

pub const ITEM_TELESCOPE: u8 = 0x20;
pub const ITEM_NONE: u8 = 0xFF;
pub const TELESCOPE_SLOT: usize = 0;
pub const ITEM_BUTTON_COUNT: usize = 3;
pub const COLLECT_SLOT: u8 = 0;

pub const NO_STATUS: u8 = 0xFF;
pub const NO_INDEX: i16 = -1;
pub const NO_STAFF: usize = usize::MAX;
pub const NO_ACTION: i8 = -1;
pub const MESG_TAG_NONE: u8 = 0xFF;
pub const MESG_TAG_FUAN: u8 = 13;
pub const ATR_NONE: u8 = 0xFF;
pub const ATR_MAX: u8 = 17;
pub const ANM_ATR_PARAM_6: u16 = 6;
pub const ANM_ATR_PARAM_14: u16 = 14;

pub const CULL_MIN: [f32; 3] = [-50.0, -20.0, -50.0];
pub const CULL_MAX: [f32; 3] = [50.0, 140.0, 50.0];

pub const AREA_HEIGHT: f32 = 100.0;
pub const AREA_ANGLE: i16 = 0x7FFF;

pub const PLAYER_EVN_POS: [[f32; 3]; 2] = [[200.0, 0.0, -200.0], [100.0, 0.0, -80.0]];

pub const SCOPE_WATCH_EYE: [f32; 3] = [20.0, 0.0, 50.0];
pub const SCOPE_WATCH_EYE_HIGH: f32 = 120.0;
pub const POST_EYE: [f32; 3] = [40.0, 140.0, 100.0];

pub const STAR_X: f32 = 319.5;
pub const STAR_Z: f32 = 186.5;
pub const TELESCOPE_X_TOL: f32 = 60.0;
pub const TELESCOPE_Z_MIN: f32 = -30.0;
pub const FOV_DIVISOR: f32 = 40.0;

pub const SCOPE_TIMER: i16 = 150;
pub const POST_TIMER_BASE: i16 = 30;
pub const POST_TIMER_SPREAD: i16 = 30;
pub const IDLE_TIMER_BASE: i16 = 60;
pub const IDLE_TIMER_SPREAD: i16 = 90;
pub const WATCH_TIMER_BASE: i16 = 15;
pub const WATCH_TIMER_SPREAD: i16 = 30;
pub const BTP_TIMER_BASE: i16 = 60;
pub const BTP_TIMER_SPREAD: i16 = 90;
pub const ATR_TIMER_FIXED: i16 = 2;
pub const ATR_TICK_BASE: i16 = 3;
pub const ATR_TICK_MASK: u32 = 3;

pub const TURN_SCALE: i16 = 4;
pub const TURN_MAX_STEP: i16 = 0x800;
pub const TURN_MIN_STEP: i16 = 0x80;

pub const SCOPE_TYPE_TELESCOPE: u8 = 1;
pub const CAMERA_FLAG_FOCUS: u32 = 0x0008_0000;
pub const PLAYER_RESET_FLAG: u32 = 0xCC70;
pub const SCOPE_FOCUS_LIFT: f32 = 2000.0;
pub const HIO_TURN_VEL: i16 = 0x0960;
pub const HIO_MAX_HEAD_X: i16 = 0x2EE0;
pub const HIO_MAX_HEAD_Y: i16 = 0x4650;
pub const HIO_MIN_HEAD_X: i16 = -0x03E8;
pub const HIO_MIN_HEAD_Y: i16 = -0x4650;
pub const HIO_MAX_BACKBONE_X: i16 = 0x0BB8;
pub const HIO_MAX_BACKBONE_Y: i16 = 0x1B58;
pub const HIO_MIN_BACKBONE_X: i16 = 0x0000;
pub const HIO_MIN_BACKBONE_Y: i16 = -0x1B58;
pub const HIO_MAX_TURN_STEP: i16 = 0x1000;
pub const ATTN_DIST_TALK: i16 = 0xA9;
pub const ATTN_DIST_SPEAK: i16 = 0xA9;
pub const CUT_NAMES: [&str; 4] = ["LOK_PLYER", "PLYER_MOV", "WAI", "ANM_CHG"];
pub const ARC_NAME: &str = "Ls";
pub const BGM_STOP_FADE: u32 = 4;
pub const COLLISION_RADIUS: f32 = 40.0;
pub const COLLISION_HEIGHT: f32 = 100.0;
pub const GRAVITY: f32 = 0.0;

pub const BGM_ISLAND_LINK_2: u32 = 0x8000_000E;
pub const STRM_DEMO_TETRA_FLY: u32 = 0xC000_0001;

pub const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
pub const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
pub const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
pub const RND_SEED: i32 = 100;

pub const ANM_NUM_ROWS: usize = 14;
pub const ANM_STT_ROWS: usize = 6;
pub const ANM_ATR_ROWS: usize = 17;
pub const BTK_OF_BTP_ROWS: usize = 11;

pub const LOK_PLYER_BIT_TURN: i32 = 1 << 0;
pub const LOK_PLYER_BIT_BACKBONE: i32 = 1 << 1;
pub const LOK_PLYER_BIT_HEAD: i32 = 1 << 2;
pub const LOK_PLYER_BIT_TRN: i32 = 1 << 3;

pub const CUT_PRM_0_KEY: &str = "prm_0";
pub const CUT_ANM_NO_KEY: &str = "AnmNo";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnmPrm {
    pub bck: i8,
    pub btp: i8,
    pub morf: f32,
    pub speed: f32,
    pub loop_mode: i32,
}

const ANM_NUM: [AnmPrm; ANM_NUM_ROWS] = [
    AnmPrm {
        bck: BCKNUM_WAIT01,
        btp: BTPNUM_MABA,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_WAIT02,
        btp: BTPNUM_MABA,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_NONE,
    },
    AnmPrm {
        bck: BCKNUM_WAIT03,
        btp: BTPNUM_MABA,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_WAIT04,
        btp: BTPNUM_MABA,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_WAIT05,
        btp: BTPNUM_MABA,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_WAIT06,
        btp: BTPNUM_MABA,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_WATASU,
        btp: BTPNUM_MABA,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_NONE,
    },
    AnmPrm {
        bck: BCKNUM_FURIMUKI,
        btp: BTPNUM_MABA,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_NONE,
    },
    AnmPrm {
        bck: BCKNUM_NOZOKU,
        btp: BTPNUM_NOZOKU,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_GET,
        btp: BTPNUM_GET,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_NONE,
    },
    AnmPrm {
        bck: BCKNUM_WAIT07,
        btp: BTPNUM_MABA,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_TALK01,
        btp: BTPNUM_MABA,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_DEMOWAIT,
        btp: BTPNUM_DEMOWAIT,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_DEMOLOOK,
        btp: BTPNUM_DEMOWAIT,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_NONE,
    },
];

const ANM_STT: [AnmPrm; ANM_STT_ROWS] = [
    AnmPrm {
        bck: -1,
        btp: -1,
        morf: 0.0,
        speed: 0.0,
        loop_mode: LOOP_NULL,
    },
    AnmPrm {
        bck: BCKNUM_WAIT01,
        btp: BTPNUM_MABA,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_WAIT06,
        btp: BTPNUM_MABA,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_WAIT07,
        btp: BTPNUM_FUAN02,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_WAIT06,
        btp: BTPNUM_MABA,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: -1,
        btp: -1,
        morf: 0.0,
        speed: 0.0,
        loop_mode: LOOP_NULL,
    },
];

const ANM_ATR: [AnmPrm; ANM_ATR_ROWS] = [
    AnmPrm {
        bck: BCKNUM_WAIT06,
        btp: BTPNUM_MABA,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_WAIT01,
        btp: BTPNUM_MABA,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_WAIT02,
        btp: BTPNUM_MABA,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_NONE,
    },
    AnmPrm {
        bck: BCKNUM_WAIT03,
        btp: BTPNUM_WARAI,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_WAIT04,
        btp: BTPNUM_FUAN02,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_WAIT05,
        btp: BTPNUM_WARAI,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_FURIMUKI,
        btp: BTPNUM_MABA,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_NONE,
    },
    AnmPrm {
        bck: BCKNUM_WATASU,
        btp: BTPNUM_WARAI,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_NONE,
    },
    AnmPrm {
        bck: BCKNUM_NOZOKU,
        btp: BTPNUM_NOZOKU,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_GET,
        btp: BTPNUM_GET,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_NONE,
    },
    AnmPrm {
        bck: BCKNUM_WAIT07,
        btp: BTPNUM_FUAN,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_WAIT07,
        btp: BTPNUM_FUAN02,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_WAIT04,
        btp: BTPNUM_FUAN02,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_WAIT06,
        btp: BTPNUM_WARAI,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_TALK01,
        btp: BTPNUM_WARAI,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_WAIT04,
        btp: BTPNUM_WARAI,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
    AnmPrm {
        bck: BCKNUM_TALK01,
        btp: BTPNUM_WARAI,
        morf: 8.0,
        speed: 1.0,
        loop_mode: LOOP_LOOP,
    },
];

const BTK_OF_BTP: [i8; BTK_OF_BTP_ROWS] = [
    BTKNUM_DEFAULT,
    BTKNUM_DEFAULT,
    BTKNUM_DEFAULT,
    BTKNUM_DEFAULT,
    BTKNUM_DEFAULT,
    BTKNUM_DEFAULT,
    BTKNUM_DEFAULT,
    BTKNUM_DEFAULT,
    BTKNUM_DEFAULT,
    BTKNUM_DEMOLOOK,
    BTKNUM_DEMOWAIT,
];

const _: () = assert!(BTK_OF_BTP.len() == BTPNUM_DEMOWAIT as usize + 1);
const _: () = assert!(ANM_ATR.len() == ANM_ATR_ROWS);
const _: () = assert!(ANM_NUM.len() == ANM_NUM_ROWS);
const _: () = assert!(ANM_STT.len() == ANM_STT_ROWS);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proc {
    None,
    Wait,
    Demo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct JntParam {
    pub max_backbone_x: i16,
    pub max_backbone_y: i16,
    pub min_backbone_x: i16,
    pub min_backbone_y: i16,
    pub max_head_x: i16,
    pub max_head_y: i16,
    pub min_head_x: i16,
    pub min_head_y: i16,
    pub max_turn_step: i16,
    pub turn_vel: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Jnt {
    pub param: JntParam,
    pub head_x: i16,
    pub head_y: i16,
    pub backbone_x: i16,
    pub backbone_y: i16,
    pub backbone_lock: bool,
    pub head_lock: bool,
    pub trn: bool,
}

impl Aryll {
    pub fn default_jnt_param() -> JntParam {
        JntParam {
            max_backbone_x: HIO_MAX_BACKBONE_X,
            max_backbone_y: HIO_MAX_BACKBONE_Y,
            min_backbone_x: HIO_MIN_BACKBONE_X,
            min_backbone_y: HIO_MIN_BACKBONE_Y,
            max_head_x: HIO_MAX_HEAD_X,
            max_head_y: HIO_MAX_HEAD_Y,
            min_head_x: HIO_MIN_HEAD_X,
            min_head_y: HIO_MIN_HEAD_Y,
            max_turn_step: HIO_MAX_TURN_STEP,
            turn_vel: HIO_TURN_VEL,
        }
    }
}

impl Jnt {
    pub fn on_backbone_lock(&mut self) {
        self.backbone_lock = true;
    }

    pub fn off_backbone_lock(&mut self) {
        self.backbone_lock = false;
    }

    pub fn on_head_lock(&mut self) {
        self.head_lock = true;
    }

    pub fn off_head_lock(&mut self) {
        self.head_lock = false;
    }

    pub fn set_trn(&mut self) {
        self.trn = true;
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Aryll {
    pub prm: i8,
    pub kind: i8,
    pub decided: bool,
    pub proc_state: Proc,
    pub stage: i8,
    pub entered: bool,
    pub event_ids: [i16; EVENT_ID_COUNT],
    pub event_index: i16,
    pub staff_idx: usize,
    pub order: i8,
    pub stt: i8,
    pub prev_stt: i8,
    pub order_talk: bool,
    pub scope_active: bool,
    pub scope_used: bool,
    pub telescope_given: bool,
    pub scope_phase: u8,
    pub scope_msg_no: u32,
    pub draw_skip: bool,
    pub primed: bool,
    pub post_shaped: bool,
    pub m840: bool,
    pub m841: bool,
    pub kamome_demo: bool,
    pub eye_live: bool,
    pub eye_ctrl_set: bool,
    pub attention: bool,
    pub anm_flag: u8,
    pub atr_ready: u8,
    pub atr: u8,
    pub mesg_anime_tag: u8,
    pub btp: i8,
    pub btk: i8,
    pub bck: i8,
    pub btp_frame: u8,
    pub btk_frame: u8,
    pub btp_frame_max: u8,
    pub btk_frame_max: u8,
    pub timer1: i16,
    pub timer2: i16,
    pub timer3: i16,
    pub timer4: i16,
    pub timer5: i16,
    pub look_mode: i8,
    pub look_angle: i16,
    pub action_index: i8,
    pub jnt: Jnt,
    pub half_head_x: i16,
    pub half_head_y: i16,
    pub eye_pitch: i16,
    pub eye_yaw: i16,
    pub joint_head_y: i16,
    pub joint_backbone_y: i16,
    pub actor_angle_y: i16,
    pub set_eye_pos: bool,
    pub morf_stopped: bool,
    pub prev_morf_frame: f32,
    pub item_no: u8,
    pub msg_no: u32,
    pub msg_bs_id: u32,
    pub msg_bound: bool,
    pub home: Vec3,
    pub home_angle: [i16; 3],
    pub angle: [i16; 3],
    pub shape_angle: [i16; 3],
    pub collision_radius: f32,
    pub collision_height: f32,
    pub look_origin: Vec3,
    pub look_target: Option<Vec3>,
    pub look_yaw: i16,
    pub event_pos: [Vec3; 2],
    pub player_eye: Vec3,
    pub transformed_eye: Vec3,
    pub attn_pos: Vec3,
    pub eye_pos: Vec3,
    pub mat_offset: [[f32; 2]; MAT_ANM_SLOTS],
    pub telescope_scale: f32,
    pub arc_name: &'static str,
    pub attn_dist_talk: i16,
    pub attn_dist_speak: i16,
    pub gravity: f32,
    pub cam_focus_flag: u32,
    pub player_reset: u32,
    pub bgm_fade: u32,
    pub partner: Option<Entity>,
    pub partner_focus: bool,
    pub partner_telescope_demo: bool,
    pub cam_subject: Option<Entity>,
    pub scope_type: u8,
    pub seed: [i32; 3],
}

impl Aryll {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            prm: ((parameters >> PRM_SHIFT) & PRM_MASK) as i8,
            kind: 0,
            decided: false,
            proc_state: Proc::None,
            stage: STAGE_INIT,
            entered: false,
            event_ids: [NO_INDEX; EVENT_ID_COUNT],
            event_index: NO_INDEX,
            staff_idx: NO_STAFF,
            order: ORDER_NONE,
            stt: 0,
            prev_stt: 0,
            order_talk: false,
            scope_active: false,
            scope_used: false,
            telescope_given: false,
            scope_phase: SCOPE_PHASE_NONE,
            scope_msg_no: 0,
            draw_skip: false,
            primed: false,
            post_shaped: false,
            m840: false,
            m841: false,
            kamome_demo: false,
            eye_live: false,
            eye_ctrl_set: false,
            attention: false,
            anm_flag: 0,
            atr_ready: 0,
            atr: ATR_NONE,
            mesg_anime_tag: MESG_TAG_NONE,
            btp: -1,
            btk: -1,
            bck: BCKNUM_NULL,
            btp_frame: 0,
            btk_frame: 0,
            btp_frame_max: 0,
            btk_frame_max: 0,
            timer1: 0,
            timer2: 0,
            timer3: 0,
            timer4: 0,
            timer5: 0,
            look_mode: LOOK_NONE,
            look_angle: 0,
            action_index: NO_ACTION,
            jnt: Jnt::default(),
            half_head_x: 0,
            half_head_y: 0,
            eye_pitch: 0,
            eye_yaw: 0,
            joint_head_y: 0,
            joint_backbone_y: 0,
            actor_angle_y: 0,
            set_eye_pos: false,
            morf_stopped: false,
            prev_morf_frame: 0.0,
            item_no: ITEM_NONE,
            msg_no: 0,
            msg_bs_id: FPC_M_ERROR_PROCESS_ID,
            msg_bound: false,
            home: Vec3::ZERO,
            home_angle: [0; 3],
            angle: [0; 3],
            shape_angle: [0; 3],
            collision_radius: 0.0,
            collision_height: 0.0,
            look_origin: Vec3::ZERO,
            look_target: None,
            look_yaw: 0,
            event_pos: [Vec3::ZERO; 2],
            player_eye: Vec3::ZERO,
            transformed_eye: Vec3::ZERO,
            attn_pos: Vec3::ZERO,
            eye_pos: Vec3::ZERO,
            mat_offset: [[0.0; 2]; MAT_ANM_SLOTS],
            telescope_scale: 0.0,
            arc_name: ARC_NAME,
            attn_dist_talk: ATTN_DIST_TALK,
            attn_dist_speak: ATTN_DIST_SPEAK,
            gravity: GRAVITY,
            cam_focus_flag: 0,
            player_reset: 0,
            bgm_fade: 0,
            partner: None,
            partner_focus: false,
            partner_telescope_demo: false,
            cam_subject: None,
            scope_type: 0,
            seed: [RND_SEED; 3],
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        !self.draw_skip
    }

    pub fn attention_pos(&self) -> Vec3 {
        self.attn_pos
    }

    pub fn item(&self) -> u8 {
        self.item_no
    }

    pub fn decide_type(&mut self, stories: &StoryBits) -> bool {
        if self.decided {
            return true;
        }
        self.decided = true;
        self.kind = -1;
        match self.prm {
            0 => {
                if stories.test(EVENT_BIT_2A80) {
                    self.kind = 0;
                }
            }
            1 => self.kind = 1,
            2 => self.kind = 2,
            3 => {
                if !stories.test(EVENT_BIT_2A80) {
                    self.kind = 3;
                }
            }
            4 => self.kind = 4,
            _ => {}
        }
        if self.kind < 0 {
            return false;
        }
        self.kind != -1
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
        self.staff_idx = ctx.list.staff_of(ARYLL_ACTOR).unwrap_or(NO_STAFF);
        self.arc_name = ARC_NAME;
        self.attn_dist_talk = ATTN_DIST_TALK;
        self.attn_dist_speak = ATTN_DIST_SPEAK;
        self.gravity = GRAVITY;
        self.bck = BCKNUM_NULL;
        let init_result = match self.kind {
            0 => self.init_ls1_0(actor, ctx),
            1 => self.init_ls1_1(actor, ctx),
            2 => self.init_ls1_2(actor, ctx),
            3 => self.init_ls1_3(actor, ctx),
            4 => self.init_ls1_4(actor, ctx),
            _ => false,
        };
        if !init_result {
            return false;
        }
        self.angle = actor.angle;
        self.shape_angle = actor.angle;
        self.play_animation(ctx.ground_hit, ctx.reverb);
        self.set_attention(ctx.pos, true);
        true
    }

    pub fn init_ls1_0(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        self.set_action(Proc::Wait, actor, ctx);
        true
    }

    pub fn init_ls1_1(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        self.set_action(Proc::Demo, actor, ctx);
        true
    }

    pub fn init_ls1_2(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        self.init_ls1_1(actor, ctx)
    }

    pub fn init_ls1_3(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        self.init_ls1_0(actor, ctx)
    }

    pub fn init_ls1_4(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        self.init_ls1_1(actor, ctx)
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
            Proc::Wait => self.wait_action1(actor, ctx),
            Proc::Demo => self.demo_action1(),
            Proc::None => {}
        }
    }

    pub fn partner_search_sub(&mut self, others: &Query<&Actor, Without<Aryll>>) -> bool {
        self.partner = others
            .iter()
            .find(|(_, actor)| actor.name == PARTNER_ACTOR)
            .map(|(entity, _)| *entity);
        self.partner.is_some()
    }

    pub fn partner_search(&mut self, others: &Query<&Actor, Without<Aryll>>) {
        if self.stage != STAGE_PENDING {
            return;
        }
        let mut found = false;
        match self.kind {
            0 => found = self.partner_search_sub(others),
            1 | 2 | 3 | 4 => found = true,
            _ => {}
        }
        if found {
            self.stage = self.stage.wrapping_add(1);
        }
    }

    pub fn search_actor_kamome_set_nostop_demo(&mut self) {
        self.kamome_demo = true;
    }

    pub fn search_actor_kamome_clr_nostop_demo(&mut self) {
        self.kamome_demo = false;
    }

    pub fn event_slot(&self) -> Option<usize> {
        usize::try_from(self.event_index).ok()
    }

    pub fn event_slot_of(&self, index: i16) -> Option<usize> {
        let slot = usize::try_from(index).ok()?;
        let id = self.event_ids.get(slot).copied().unwrap_or(NO_INDEX);
        if id < 0 {
            return None;
        }
        usize::try_from(id).ok()
    }

    pub fn is_event_entry(&self) -> i32 {
        i32::try_from(self.staff_idx).unwrap_or(-1)
    }

    pub fn end_event(&mut self) {
        self.atr = ATR_NONE;
        self.mesg_anime_tag = MESG_TAG_NONE;
    }

    pub fn event_proc(&mut self, staff: i32, ctx: &mut Ctx) {
        let ended = self
            .event_slot_of(self.event_index)
            .is_some_and(|index| ctx.active.end_check(index));
        if ended {
            match self.event_slot() {
                Some(EVENT_ZELDA_FLY) => {
                    if self.scope_used {
                        if ctx.text.info.scope_mesg_status == STATUS_UNK0 {
                            ctx.stories.set(EVENT_BIT_0001);
                            self.set_stt(STT_POST);
                            self.order = ORDER_SPEAK;
                            self.event_index = 3;
                            self.jnt.off_backbone_lock();
                            self.scope_active = false;
                            self.m840 = false;
                            self.post_shaped = false;
                        } else {
                            return;
                        }
                    } else if ctx.text.info.release_scope_mode() {
                        ctx.camera_running = true;
                        self.player_reset_flag();
                        self.bgm_fade = BGM_STOP_FADE;
                        ctx.events.list.push(SimEvent::Bgm(BGM_ISLAND_LINK_2));
                        self.scope_used = true;
                        return;
                    } else {
                        return;
                    }
                }
                Some(EVENT_OMEDETO) => {
                    self.order = ORDER_EVENT_2;
                }
                Some(EVENT_GET_TELESCOPE) => {
                    self.set_stt(STT_SCOPE);
                    self.order = ORDER_SPEAK;
                    self.telescope_given = true;
                }
                _ => {}
            }
            self.end_event();
        } else if !ctx.cut_proc {
            self.private_cut(staff, ctx);
        }
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

    pub fn private_cut(&mut self, staff: i32, ctx: &mut Ctx) {
        if staff < 0 {
            return;
        }
        let cut = self.current_cut(staff, ctx);
        ctx.cut_action = cut
            .and_then(|cut| ctx.list.cut_name(cut))
            .and_then(cut_action_of);
        let Some(action) = ctx.cut_action else {
            return;
        };
        self.action_index = action;
        if ctx.cut_advance {
            match action {
                CUT_LOK_PLYER => {
                    let prm = cut.and_then(|cut| ctx.list.cut_number(cut, CUT_PRM_0_KEY));
                    self.cut_init_lok_plyer(prm);
                }
                CUT_PLYER_MOV => self.cut_init_plyer_mov(),
                CUT_WAI => self.cut_init_wai(),
                CUT_ANM_CHG => {
                    let anm = cut.and_then(|cut| ctx.list.cut_number(cut, CUT_ANM_NO_KEY));
                    self.cut_init_anm_chg(anm);
                }
                _ => {}
            }
        }
        let cut_end = match action {
            CUT_LOK_PLYER => self.cut_move_lok_plyer(),
            CUT_PLYER_MOV => self.cut_move_plyer_mov(),
            CUT_WAI => self.cut_move_wai(),
            CUT_ANM_CHG => self.cut_move_anm_chg(),
            _ => true,
        };
        let _ = cut_end;
    }

    pub fn cut_init_lok_plyer(&mut self, prm: Option<i32>) {
        self.look_mode = LOOK_PLAYER;
        self.m840 = false;
        self.jnt.off_backbone_lock();
        self.jnt.off_head_lock();
        if let Some(prm) = prm {
            if prm & LOK_PLYER_BIT_TRN != 0 {
                self.jnt.set_trn();
            } else {
                if prm & LOK_PLYER_BIT_TURN != 0 {
                    self.m840 = true;
                }
                if prm & LOK_PLYER_BIT_BACKBONE != 0 {
                    self.jnt.on_backbone_lock();
                }
                if prm & LOK_PLYER_BIT_HEAD != 0 {
                    self.jnt.on_head_lock();
                }
            }
        }
    }

    pub fn cut_move_lok_plyer(&self) -> bool {
        true
    }

    pub fn cut_init_plyer_mov(&mut self) {}

    pub fn cut_move_plyer_mov(&self) -> bool {
        true
    }

    pub fn cut_init_wai(&mut self) {}

    pub fn cut_move_wai(&mut self) -> bool {
        if self.bck != BCKNUM_WAIT02 {
            return true;
        }
        if self.morf_stopped {
            self.set_anm_num(ANMNUM_WAIT01, true);
            return true;
        }
        false
    }

    pub fn cut_init_anm_chg(&mut self, anm_no: Option<i32>) {
        let anm = anm_no.unwrap_or(i32::from(self.bck));
        self.set_anm_num(anm, true);
    }

    pub fn cut_move_anm_chg(&self) -> bool {
        true
    }

    pub fn set_stt(&mut self, stt: i8) {
        let prev_stt = self.stt as u8;
        self.stt = stt;
        match self.stt {
            STT_WAIT => {
                self.order = ORDER_NONE;
                let timer = rnd_value(&mut self.seed, IDLE_TIMER_BASE, IDLE_TIMER_SPREAD);
                self.timer5 = timer;
            }
            STT_SCOPE => {
                self.order = ORDER_NONE;
                self.scope_phase = SCOPE_PHASE_NONE;
                self.timer3 = 0;
            }
            STT_POST => {
                self.order = ORDER_NONE;
            }
            STT_TELESCOPE => {
                self.order = ORDER_NONE;
                self.telescope_scale = 1.0;
            }
            STT_TALK => {
                self.search_actor_kamome_set_nostop_demo();
                self.order = ORDER_NONE;
                self.atr = ATR_NONE;
                self.mesg_anime_tag = MESG_TAG_NONE;
                self.atr_ready = 0;
                self.prev_stt = prev_stt as i8;
            }
            _ => {}
        }
        self.set_anm();
    }

    pub fn set_btp(&mut self, btp: i8) -> bool {
        if btp < 0 {
            return false;
        }
        self.btp = btp;
        self.btp_frame = 0;
        self.timer1 = 0;
        true
    }

    pub fn set_btk(&mut self, btk: i8) -> bool {
        if btk < 0 {
            return false;
        }
        self.btk = btk;
        self.btk_frame = 0;
        true
    }

    pub fn init_tex_pttrn_anm(&mut self, btp: i8) -> bool {
        if !self.set_btp(btp) {
            return false;
        }
        let slot = usize::try_from(btp).unwrap_or(usize::MAX);
        let Some(btk) = BTK_OF_BTP.get(slot).copied() else {
            return false;
        };
        self.set_btk(btk)
    }

    pub fn play_btp_anm(&mut self) {
        let frame_max = self.btp_frame_max;
        if self.btp != BTPNUM_MABA || calc_timer(&mut self.timer1) == 0 {
            self.btp_frame = self.btp_frame.wrapping_add(1);
            if self.btp_frame >= frame_max {
                if self.btp != BTPNUM_MABA {
                    self.btp_frame = frame_max;
                } else {
                    let timer = rnd_value(&mut self.seed, BTP_TIMER_BASE, BTP_TIMER_SPREAD);
                    self.timer1 = timer;
                    self.btp_frame = 0;
                }
            }
        }
    }

    pub fn play_btk_anm(&mut self) {
        let frame_max = self.btk_frame_max;
        if self.eye_live {
            self.eye_ctrl();
        } else {
            self.btk_frame = self.btk_frame.wrapping_add(1);
            if self.btk_frame >= frame_max {
                self.btk_frame = frame_max;
            }
        }
    }

    pub fn set_eye_ctrl(&mut self) {
        self.eye_ctrl_set = true;
    }

    pub fn clr_eye_ctrl(&mut self) {
        self.eye_ctrl_set = false;
    }

    pub fn eye_ctrl(&mut self) {
        let mut fvar2;
        let mut fvar1;
        if self.eye_ctrl_set {
            fvar1 = f32::from(self.eye_yaw) / f32::from(HIO_EYE_SCALE_DENOM);
            fvar2 = f32::from(self.eye_pitch) / f32::from(HIO_EYE_SCALE_DENOM);
            fvar1 = EYE_MULT * fvar1;
            fvar2 = EYE_MULT * fvar2;
            fvar1 = fvar1.clamp(EYE_MIN, EYE_MAX);
            fvar2 = fvar2.clamp(EYE_MIN, EYE_MAX);
        } else {
            fvar1 = 0.0;
            fvar2 = fvar1;
        }
        self.eye_calc(0, fvar2, fvar1);
        fvar2 *= -1.0;
        self.eye_calc(1, fvar2, fvar1);
    }

    fn eye_calc(&mut self, slot: usize, x: f32, y: f32) {
        let Some(entry) = self.mat_offset.get_mut(slot) else {
            return;
        };
        if let Some(value) = entry.first_mut() {
            add_calc(
                value,
                x,
                EYE_CALC_SCALE,
                EYE_CALC_MAX_STEP,
                EYE_CALC_MIN_STEP,
            );
        }
        if let Some(value) = entry.get_mut(1) {
            add_calc(
                value,
                y,
                EYE_CALC_SCALE,
                EYE_CALC_MAX_STEP,
                EYE_CALC_MIN_STEP,
            );
        }
    }

    pub fn set_anm_anm(&mut self, prm: AnmPrm) {
        if prm.bck < 0 || self.bck == prm.bck {
            return;
        }
        self.bck = prm.bck;
        self.morf_stopped = false;
        self.anm_flag = 0;
        self.prev_morf_frame = 0.0;
    }

    pub fn set_anm_num(&mut self, index: i32, modify: bool) {
        let Some(prm) = table_of(&ANM_NUM, index).copied() else {
            return;
        };
        if modify {
            self.init_tex_pttrn_anm(prm.btp);
        }
        self.set_anm_anm(prm);
    }

    pub fn set_anm(&mut self) {
        let Some(prm) = table_of(&ANM_STT, i32::from(self.stt)).copied() else {
            return;
        };
        self.init_tex_pttrn_anm(prm.btp);
        self.set_anm_anm(prm);
    }

    pub fn chng_anm_tag(&mut self) {
        match self.mesg_anime_tag {
            0 => {
                self.init_tex_pttrn_anm(BTPNUM_WARAI);
            }
            11 => {
                self.init_tex_pttrn_anm(BTPNUM_KIZUKU);
            }
            12 => {
                self.init_tex_pttrn_anm(BTPNUM_NGWARAI);
            }
            MESG_TAG_FUAN => {
                self.init_tex_pttrn_anm(BTPNUM_FUAN);
            }
            _ => {}
        }
    }

    pub fn ctrl_anm_tag(&mut self) {}

    pub fn chng_anm_atr(&mut self, atr: u8, frame: u32) {
        if (self.atr == 3 || self.atr == 5) && (atr == 3 || atr == 5) {
            self.init_tex_pttrn_anm(BTPNUM_WARAI);
            self.atr = atr;
            return;
        }
        if atr == self.atr || atr > ATR_MAX {
            return;
        }
        self.atr = atr;
        self.set_anm_atr(true);
        match atr {
            1 => {
                let timer = rnd_value(&mut self.seed, IDLE_TIMER_BASE, IDLE_TIMER_SPREAD);
                self.timer2 = timer;
            }
            3 | 5 => {
                let timer = atr_tick(frame);
                self.timer2 = timer;
            }
            7 => self.telescope_scale = 0.0,
            12 | 14 | 15 | 16 => self.timer2 = ATR_TIMER_FIXED,
            _ => {}
        }
    }

    pub fn ctrl_anm_atr(&mut self, frame: u32) {
        match self.atr {
            1 => {
                if calc_timer(&mut self.timer2) == 0 {
                    self.atr = 2;
                    self.set_anm_atr(true);
                }
            }
            2 => {
                if self.morf_stopped {
                    self.atr = 1;
                    self.set_anm_atr(true);
                }
            }
            3 | 5 => {
                if self.morf_stopped && calc_timer(&mut self.timer2) == 0 {
                    let timer = atr_tick(frame);
                    self.timer2 = timer;
                    match self.atr {
                        3 => self.atr = 5,
                        5 => self.atr = 3,
                        _ => {}
                    }
                    self.set_anm_atr(false);
                }
            }
            12 | 14 | 15 | 16 => {
                if self.morf_stopped && calc_timer(&mut self.timer2) == 0 {
                    self.ctrl_anm_atr_post();
                }
            }
            _ => {}
        }
    }

    fn ctrl_anm_atr_post(&mut self) {
        match self.atr {
            12 => {
                if self.bck != BCKNUM_WAIT07 {
                    self.set_anm_num(ANMNUM_WAIT07, true);
                    self.init_tex_pttrn_anm(BTPNUM_FUAN02);
                }
            }
            14 => {
                if self.bck != BCKNUM_WAIT06 {
                    self.set_anm_num(ANMNUM_WAIT06, true);
                    self.init_tex_pttrn_anm(BTPNUM_WARAI);
                }
            }
            15 => {
                if self.bck != BCKNUM_TALK01 {
                    self.set_anm_num(i32::from(BCKNUM_TALK01), true);
                    self.init_tex_pttrn_anm(BTPNUM_WARAI);
                }
            }
            16 => {
                if self.bck != BCKNUM_WAIT05 {
                    self.set_anm_num(ANMNUM_WAIT05, true);
                    self.init_tex_pttrn_anm(BTPNUM_WARAI);
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
            self.init_tex_pttrn_anm(prm.btp);
        }
        self.set_anm_anm(prm);
    }

    pub fn anm_atr(&mut self, param: u16, ctx: &mut Ctx) {
        match param {
            ANM_ATR_PARAM_6 => {
                if self.atr_ready == 0 {
                    let attr = ctx.text.info.mesg_anime;
                    let frame = ctx.frame;
                    self.chng_anm_atr(attr, frame);
                    self.atr_ready = self.atr_ready.wrapping_add(1);
                }
                let mesg_anime_tag = ctx.text.info.mesg_anime_tag;
                if mesg_anime_tag != MESG_TAG_NONE && mesg_anime_tag != self.mesg_anime_tag {
                    ctx.text.info.clear_mesg_anime_tag_info();
                    self.mesg_anime_tag = mesg_anime_tag;
                    self.chng_anm_tag();
                }
            }
            ANM_ATR_PARAM_14 => self.atr_ready = 0,
            _ => {}
        }
        let frame = ctx.frame;
        self.ctrl_anm_atr(frame);
        self.ctrl_anm_tag();
    }

    pub fn play_animation(&mut self, ground_sound: bool, reverb: u8) {
        if self.btk == BTKNUM_DEFAULT && self.eye_live {
            self.set_eye_ctrl();
        } else {
            self.clr_eye_ctrl();
        }
        self.play_btp_anm();
        self.play_btk_anm();
        let _ = (ground_sound, reverb);
        let _ = self.prev_morf_frame;
    }

    pub fn next_msg_status(&self, msg_no: &mut u32, clear_count: u8) -> u8 {
        let mut status = STATUS_MSG_CONTINUES;
        match *msg_no {
            MSG_BB9 => {
                if clear_count != 0 {
                    *msg_no = MSG_BDA;
                } else {
                    *msg_no = MSG_BBA;
                }
            }
            MSG_BBA | MSG_BDA => *msg_no = MSG_BBB,
            MSG_BBB => *msg_no = MSG_BBC,
            MSG_BBC => *msg_no = MSG_BBD,
            MSG_BBD => *msg_no = MSG_BDC,
            MSG_BBE => *msg_no = MSG_BBF,
            MSG_BC5 => *msg_no = MSG_BC6,
            _ => status = STATUS_MSG_ENDS,
        }
        status
    }

    pub fn get_msg_ls1_0(&mut self, ctx: &Ctx) -> u32 {
        if ctx.stories.test(EVENT_BIT_0001) {
            if self.scope_used {
                self.scope_used = false;
                return if collect0(ctx) != 0 { MSG_BC4 } else { MSG_BC5 };
            }
            if ctx.stories.test(EVENT_BIT_0280) {
                return MSG_BC9;
            }
            return if collect0(ctx) != 0 { MSG_BC8 } else { MSG_BC7 };
        }
        if self.telescope_given {
            self.telescope_given = false;
            return MSG_BBE;
        }
        if ctx.select_item.first().copied() == Some(ITEM_TELESCOPE) {
            return MSG_BCB;
        }
        if ctx.select_item.get(1).copied() == Some(ITEM_TELESCOPE) {
            return MSG_BCC;
        }
        if ctx.select_item.get(2).copied() == Some(ITEM_TELESCOPE) {
            return MSG_BCD;
        }
        MSG_BC0
    }

    pub fn get_msg_ls1_3(&mut self) -> u32 {
        MSG_BD9
    }

    pub fn get_msg(&mut self, ctx: &Ctx) -> u32 {
        match self.kind {
            0 => self.get_msg_ls1_0(ctx),
            3 => self.get_msg_ls1_3(),
            _ => 0,
        }
    }

    pub fn event_order(&mut self, ctx: &mut Ctx) {
        if self.order == ORDER_SPEAK || self.order == ORDER_CANTALK {
            return;
        }
        if self.order >= ORDER_EVENT {
            self.event_index = self.order.wrapping_sub(ORDER_EVENT);
            let slot = self.event_slot_of(self.event_index);
            let name = slot
                .and_then(|slot| EVENT_NAMES.get(slot).copied())
                .unwrap_or(ARYLL_ACTOR);
            if let Some(index) = ctx.list.event_index(name) {
                ctx.active.order(ctx.list, index, ctx.entity);
            }
        }
    }

    pub fn check_order(&mut self, ctx: &mut Ctx) {
        if ctx.demo_accept {
            if self
                .event_slot_of(self.event_index)
                .is_some_and(|index| ctx.active.start_check(index))
                && self.order >= ORDER_EVENT
            {
                match self.event_slot() {
                    Some(EVENT_ZELDA_FLY) => {
                        ctx.camera_running = false;
                        ctx.events.list.push(SimEvent::Bgm(STRM_DEMO_TETRA_FLY));
                        self.scope_active = true;
                    }
                    Some(EVENT_OMEDETO) => {
                        let _ = target_angle_y(self.event_pos[0], ctx.pos);
                        self.post_shaped = true;
                    }
                    Some(EVENT_GET_TELESCOPE) => {
                        self.telescope_scale = 0.0;
                        self.set_anm_num(ANMNUM_GET, true);
                    }
                    _ => {}
                }
                self.order = ORDER_NONE;
                self.atr = ATR_NONE;
                self.mesg_anime_tag = MESG_TAG_NONE;
            }
        } else if ctx.talk_command && (self.order == ORDER_SPEAK || self.order == ORDER_CANTALK) {
            self.order = ORDER_NONE;
            self.order_talk = true;
        }
    }

    pub fn demo(&mut self, demo_actor: Option<i32>) -> bool {
        if demo_actor.unwrap_or(0) == 0 {
            if self.m841 {
                if self.kind == 3 {
                    self.bck = BCKNUM_NULL;
                    self.set_anm_num(ANMNUM_WAIT06, true);
                }
                self.m841 = false;
            }
        } else {
            self.btp_frame = self.btp_frame.wrapping_add(1);
            if self.btp_frame >= self.btp_frame_max {
                self.btp_frame = self.btp_frame_max;
            }
            self.btk_frame = self.btk_frame.wrapping_add(1);
            if self.btk_frame >= self.btk_frame_max {
                self.btk_frame = self.btk_frame_max;
            }
            self.m841 = true;
            self.post_shaped = false;
            self.jnt.head_x = 0;
            self.jnt.head_y = 0;
            self.jnt.backbone_x = 0;
            self.jnt.backbone_y = 0;
        }
        self.m841
    }

    pub fn chk_talk(&mut self, ctx: &Ctx) -> bool {
        if ctx.talk_xy {
            if ctx.present_done {
                self.item_no = ctx.present_item;
                return true;
            }
            return false;
        }
        self.item_no = ITEM_NONE;
        true
    }

    pub fn chk_parts_not_mov(&self) -> bool {
        self.joint_head_y != self.jnt.head_y
            || self.joint_backbone_y != self.jnt.backbone_y
            || self.actor_angle_y != self.angle[1]
    }

    pub fn chk_attention(&self, ctx: &Ctx) -> bool {
        ctx.attention
    }

    pub fn set_attention(&mut self, pos: Vec3, set_eye_pos: bool) {
        self.attn_pos = Vec3::new(pos.x, pos.y + HIO_ATT_POS_OFFSET_Y, pos.z);
        if !self.set_eye_pos && !set_eye_pos {
            return;
        }
        self.eye_pos = self.transformed_eye;
    }

    pub fn chk_area_in(
        &self,
        radius: f32,
        height: f32,
        angle: i16,
        origin: Vec3,
        player: Vec3,
    ) -> bool {
        let gap = player - origin;
        let abs_xz = (gap.x * gap.x + gap.z * gap.z).sqrt();
        let fvar1 = player.y - origin.y;
        let svar = target_angle_y(origin, player).wrapping_sub(self.angle[1]);
        abs_xz < radius && fvar1.abs() < height && i32::from(svar).abs() < i32::from(angle)
    }

    pub fn get_player_evn_pos(&self, index: usize, pos: Vec3, yaw: i16) -> Vec3 {
        let offset = PLAYER_EVN_POS
            .get(index)
            .copied()
            .unwrap_or(PLAYER_EVN_POS[0]);
        pos + Quat::from_rotation_y(from_s16(yaw)) * Vec3::from_array(offset)
    }

    pub fn chk_telescope_sph(&self, target: Option<Vec3>, radius: f32, range: f32) -> bool {
        let Some(point) = target else {
            return false;
        };
        let sp18 = Vec3::new(point.x, 0.0, point.y);
        let mut sp0c = Vec3::new(sp18.x - STAR_X, 0.0, sp18.z - STAR_Z);
        let square = sp0c.x * sp0c.x + sp0c.y * sp0c.y + sp0c.z * sp0c.z;
        if square < IS_NEAR_ZERO_SQUARE {
            return false;
        }
        sp0c *= 1.0 / square.sqrt();
        sp0c *= radius;
        let sp18 = sp18 + sp0c;
        let dx = sp18.x - STAR_X;
        let dz = sp18.z - STAR_Z;
        range > (dx * dx + dz * dz).sqrt()
    }

    pub fn chk_telescope(
        &self,
        target: Option<Vec3>,
        fov_y: Option<f32>,
        x_tol: f32,
        z_min: f32,
    ) -> bool {
        let Some(point) = target else {
            return false;
        };
        let Some(fovy) = fov_y else {
            return false;
        };
        let mut fov = fovy / FOV_DIVISOR;
        if fov > 1.0 {
            fov = 1.0;
        }
        let sp14 = Vec3::new(point.x, 0.0, point.y);
        let sp08 = Vec3::new(fov * (sp14.x - STAR_X), 0.0, fov * (sp14.z - STAR_Z));
        sp08.x.abs() < x_tol && sp08.z > z_min
    }

    pub fn telescope_proc(&mut self, ctx: &mut Ctx) -> bool {
        if self.partner.is_none() {
            return false;
        }
        let scope_mesg_status = ctx.text.info.scope_mesg_status;
        if scope_mesg_status == STATUS_UNK0 {
            if ctx
                .text
                .info
                .check_player_status0(DA_PY_STTS0_TELESCOPE_LOOK)
            {
                if ctx.scope_wipe {
                    self.player_reset_flag();
                }
                self.scope_type = SCOPE_TYPE_TELESCOPE;
                return true;
            }
            return false;
        }
        if self.scope_phase == SCOPE_PHASE_NONE {
            ctx.events.list.push(SimEvent::Bgm(STRM_DEMO_TETRA_FLY));
            self.partner_telescope_demo = true;
            self.timer5 = SCOPE_TIMER;
            self.scope_msg_no = u32::MAX;
            self.scope_phase = SCOPE_PHASE_START;
            ctx.text.info.get_scope_mode();
            return true;
        }
        let current = ctx.text.info.scope_mesg_status;
        match current {
            STATUS_BOX_CLOSED => {
                ctx.text.info.scope_mesg_status = STATUS_BOX_CLOSING;
                if self.scope_phase == SCOPE_PHASE_FOCUS && self.scope_msg_no != 0 {
                    self.cam_clr_flag_focus();
                    self.cam_subject_lock_off();
                    ctx.stories.set(EVENT_BIT_0310);
                    self.timer5 = 0;
                    self.scope_phase = SCOPE_PHASE_END;
                    ctx.text.info.get_scope_mode();
                    return true;
                }
                self.scope_step(scope_mesg_status, ctx);
            }
            STATUS_CLOSE_WAIT | STATUS_SCOPE_WAIT => {
                self.scope_step(scope_mesg_status, ctx);
            }
            _ => {}
        }
        ctx.text.info.get_scope_mode();
        true
    }

    fn partner_set_focus(&mut self) {
        self.partner_focus = true;
}

fn player_reset_flag(&mut self) {
        self.player_reset = PLAYER_RESET_FLAG;
}

fn cam_clr_flag_focus(&mut self) {
    self.cam_focus_flag &= !CAMERA_FLAG_FOCUS;
}

fn cam_set_flag_focus(&mut self) {
    self.cam_focus_flag |= CAMERA_FLAG_FOCUS;
}

    fn cam_subject_lock_off(&mut self) {
        self.cam_subject = None;
    }

    fn cam_subject_lock_on(&mut self, partner: Option<Entity>) {
        self.cam_subject = partner;
    }

    fn scope_step(&mut self, scope_mesg_status: u8, ctx: &mut Ctx) {
        let mut temp_msg_no: u32 = 0;
        match self.scope_phase {
            SCOPE_PHASE_START => {
                temp_msg_no = self.scope_msg_no;
                self.scope_msg_no = MSG_BC1;
                if self.chk_telescope_sph(ctx.project, 0.0, HIO_SPH_RANGE) {
                    self.scope_msg_no = if ctx
                        .scope_zoom
                        .is_some_and(|zoom| zoom >= HIO_ZOOM_BIRD)
                    {
                        MSG_BCB
                    } else {
                        MSG_BCA
                    };
                }
            }
            SCOPE_PHASE_ZOOM => {
                self.scope_msg_no = MSG_BCA;
                if ctx
                    .scope_zoom
                    .is_some_and(|zoom| zoom >= HIO_ZOOM_FOCUS)
                {
                    if scope_mesg_status == STATUS_CLOSE_WAIT {
                        ctx.text.force_send_on();
                    }
                    self.cam_set_flag_focus();
                    self.partner_set_focus();
                    self.timer5 = 0;
                    self.scope_msg_no = 0;
                    self.scope_phase = SCOPE_PHASE_FOCUS;
                    ctx.text.info.get_scope_mode();
                    return;
                }
            }
            SCOPE_PHASE_FOCUS => {
                if self.scope_msg_no == 0 && ctx.partner_odoroki {
                    if scope_mesg_status == STATUS_CLOSE_WAIT {
                        ctx.text.force_send_on();
                    } else if ctx.text.info.release_scope_mode() {
                        self.scope_msg_no = MSG_BC2;
                        let msg_no = self.scope_msg_no;
                        ctx.text.scope_message_set(msg_no);
                    }
                }
                ctx.text.info.get_scope_mode();
                return;
            }
            SCOPE_PHASE_END => {
                let target = ctx.project_high.map(|point| {
                    Vec3::new(point.x, point.y + SCOPE_FOCUS_LIFT, point.z)
                });
                let hit = self.chk_telescope(target, ctx.fov_y, TELESCOPE_X_TOL, TELESCOPE_Z_MIN);
                if hit {
                    if scope_mesg_status == STATUS_CLOSE_WAIT {
                        ctx.text.force_send_on();
                    } else if ctx.text.info.release_scope_mode() {
                        self.order = ORDER_EVENT;
                    }
                }
                self.scope_msg_no = MSG_BC3;
            }
            _ => {}
        }
        if (self.scope_phase == SCOPE_PHASE_START && temp_msg_no != self.scope_msg_no)
            || calc_timer(&mut self.timer5) == 0
        {
            if scope_mesg_status == STATUS_CLOSE_WAIT {
                ctx.text.force_send_on();
                self.timer5 = 0;
            } else if ctx.text.info.release_scope_mode() {
                self.timer5 = SCOPE_TIMER;
                if self.scope_phase == SCOPE_PHASE_START && self.scope_msg_no == MSG_BCB {
                    let partner = self.partner;
                    self.cam_subject_lock_on(partner);
                    self.scope_msg_no = MSG_BCA;
                    self.timer5 = SCOPE_TIMER;
                    self.scope_phase = SCOPE_PHASE_ZOOM;
                }
                if self.scope_msg_no == MSG_BC3 {
                    let timer = rnd_value(&mut self.seed, POST_TIMER_BASE, POST_TIMER_SPREAD);
                    self.timer3 = timer;
                }
                let msg_no = self.scope_msg_no;
                ctx.text.scope_message_set(msg_no);
            }
        }
    }

    pub fn wait_1(&mut self, ctx: &mut Ctx) -> bool {
        if self.order < ORDER_EVENT {
            self.order = ORDER_NONE;
            if self.chk_area_in(
                HIO_WATCH_RANGE,
                AREA_HEIGHT,
                AREA_ANGLE,
                ctx.pos,
                player_pos(ctx),
            ) {
                self.order = ORDER_EVENT_1;
            }
        }
        self.look_mode = LOOK_NONE;
        if self.bck == BCKNUM_WAIT01 {
            if calc_timer(&mut self.timer5) == 0 {
                self.set_anm_num(ANMNUM_WAIT02, true);
            }
            return true;
        }
        if self.morf_stopped {
            let timer = rnd_value(&mut self.seed, IDLE_TIMER_BASE, IDLE_TIMER_SPREAD);
            self.timer5 = timer;
            self.set_anm_num(ANMNUM_WAIT01, true);
        }
        true
    }

    pub fn wait_2(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        actor.angle[1] = add_calc_angle_s(
            actor.angle[1],
            self.home_angle[1],
            TURN_SCALE,
            TURN_MAX_STEP,
            TURN_MIN_STEP,
        );
        if actor.angle[1] == self.home_angle[1] && self.bck != BCKNUM_WAIT01 {
            self.set_anm_num(ANMNUM_WAIT01, true);
        }
        if self.order_talk {
            if self.chk_talk(ctx) {
                self.set_stt(STT_TALK);
                self.set_anm_num(ANMNUM_WAIT06, true);
                self.look_mode = LOOK_PLAYER;
                self.jnt.off_backbone_lock();
                self.m840 = false;
                self.jnt.set_trn();
            }
            return true;
        }
        let mut telescope_proc_result = false;
        if self.order != ORDER_SPEAK && self.order < ORDER_EVENT {
            self.order = ORDER_NONE;
            if self.chk_area_in(
                HIO_TELESCOPE_RANGE,
                AREA_HEIGHT,
                AREA_ANGLE,
                ctx.pos,
                player_pos(ctx),
            ) && ctx
                .text
                .info
                .check_player_status0(DA_PY_STTS0_TELESCOPE_LOOK)
            {
                telescope_proc_result = self.telescope_proc(ctx);
            }
            if !telescope_proc_result {
                self.order = ORDER_CANTALK;
            }
        }
        self.jnt.on_backbone_lock();
        self.m840 = true;
        if self.scope_phase != SCOPE_PHASE_NONE {
            let mut base_pos = Vec3::from_array(SCOPE_WATCH_EYE);
            if self.scope_phase >= SCOPE_PHASE_END {
                if self.btp != BTPNUM_FUAN02 {
                    self.init_tex_pttrn_anm(BTPNUM_FUAN02);
                }
                if calc_timer(&mut self.timer3) != 0 {
                    self.look_mode = LOOK_PLAYER;
                    return true;
                }
                base_pos.y = SCOPE_WATCH_EYE_HIGH;
            }
            self.player_eye = rotated_y(base_pos, ctx.pos, actor.angle[1]);
            self.look_mode = LOOK_STORED;
            self.eye_live = true;
            return true;
        }
        if self.attention && self.bck == BCKNUM_WAIT01 {
            self.look_mode = LOOK_PLAYER;
        } else {
            self.look_mode = LOOK_NONE;
        }
        true
    }

    pub fn wait_3(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        let local_18 = Vec3::from_array(POST_EYE);
        self.player_eye = rotated_y(local_18, ctx.pos, self.home_angle[1]);
        actor.angle[1] = add_calc_angle_s(
            actor.angle[1],
            target_angle_y(ctx.pos, self.player_eye),
            TURN_SCALE,
            TURN_MAX_STEP,
            TURN_MIN_STEP,
        );
        if self.order_talk {
            if self.chk_talk(ctx) {
                self.set_stt(STT_TALK);
                self.look_mode = LOOK_PLAYER;
                self.m840 = false;
                self.jnt.set_trn();
            }
            return true;
        }
        if self.order != ORDER_SPEAK {
            self.order = ORDER_CANTALK;
        }
        self.m840 = true;
        self.look_mode = LOOK_STORED;
        self.eye_live = true;
        true
    }

    pub fn wait_4(&mut self, ctx: &mut Ctx) -> bool {
        if self.order_talk {
            if self.chk_talk(ctx) {
                self.set_stt(STT_TALK);
                self.look_mode = LOOK_PLAYER;
                self.m840 = false;
                self.jnt.set_trn();
            }
            return true;
        }
        self.order = ORDER_CANTALK;
        self.m840 = true;
        if self.attention {
            let timer = rnd_value(&mut self.seed, WATCH_TIMER_BASE, WATCH_TIMER_SPREAD);
            self.timer4 = timer;
        }
        if calc_timer(&mut self.timer4) != 0 {
            self.look_mode = LOOK_PLAYER;
            return true;
        }
        self.look_mode = LOOK_NONE;
        true
    }

    pub fn talk_1(&mut self, ctx: &mut Ctx) -> bool {
        let res = self.chk_parts_not_mov();
        self.npc_talk(ctx);
        let status = ctx.text.status();
        if matches!(status, Some(STATUS_BOX_OPENING) | Some(STATUS_MSG_TYPING)) {
            return res;
        }
        if status != Some(STATUS_MSG_DESTROYED) {
            return res;
        }
        match self.msg_no {
            MSG_BC2 | MSG_BC3 | MSG_BC5 => {}
            MSG_BC4 | MSG_BC6 => self.event_index = NO_INDEX,
            _ => {}
        }
        self.search_actor_kamome_clr_nostop_demo();
        self.item_no = ITEM_NONE;
        self.order_talk = false;
        let prev_stt = self.prev_stt;
        self.set_stt(prev_stt);
        let timer = rnd_value(&mut self.seed, WATCH_TIMER_BASE, WATCH_TIMER_SPREAD);
        self.timer4 = timer;
        self.end_event();
        res
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
                let next = self.next_msg_status(&mut next_no, ctx.clear_count);
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
            self.anm_atr(status as u16, ctx);
        } else {
            self.msg_bound = ctx.text.procs.search_by_id(self.msg_bs_id).is_some();
        }
        status
    }

    pub fn look_back(&mut self, actor: &Actor, ctx: &Ctx) {
        self.joint_head_y = self.jnt.head_y;
        self.joint_backbone_y = self.jnt.backbone_y;
        self.actor_angle_y = actor.angle[1];
        self.look_origin = Vec3::new(ctx.pos.x, self.eye_pos.y, ctx.pos.z);
        let mut player_eye_pos = Vec3::ZERO;
        let mut player_eye_pos_p: Option<Vec3> = None;
        self.look_yaw = actor.angle[1];
        match self.look_mode {
            LOOK_PLAYER => {
                self.player_eye = player_eye_pos_of(ctx, HIO_PLAYER_EYE_OFFSET_Y);
                player_eye_pos = self.player_eye;
                player_eye_pos_p = Some(player_eye_pos);
                self.eye_live = true;
            }
            LOOK_STORED => {
                player_eye_pos = self.player_eye;
                player_eye_pos_p = Some(player_eye_pos);
            }
            LOOK_ANGLE => {
                self.look_yaw = self.look_angle;
                self.eye_live = false;
            }
            LOOK_ACTOR => {}
            _ => {
                self.eye_live = false;
            }
        }
        self.look_target = player_eye_pos_p;
        let _turn_vel = self.jnt.param.turn_vel;
        self.half_head_x = self.jnt.head_x / 2;
        self.half_head_y = self.jnt.head_y / 2;
        self.eye_yaw = self.half_head_x;
        self.eye_pitch = self.half_head_y.wrapping_neg();
    }

    pub fn wait_action1(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        match self.stage {
            STAGE_INIT => {
                let first = self.get_player_evn_pos(0, ctx.pos, actor.angle[1]);
                let second = self.get_player_evn_pos(1, ctx.pos, actor.angle[1]);
                self.event_pos[0] = first;
                self.event_pos[1] = second;
                self.stage = self.stage.wrapping_add(1);
                if ctx.stories.test(EVENT_BIT_2A80) {
                    if ctx.stories.test(EVENT_BIT_0001) {
                        self.set_stt(STT_POST);
                    } else if ctx.equipped.item(TELESCOPE_SLOT) == ITEM_TELESCOPE {
                        self.home_angle[1] = actor.angle[1];
                        self.set_stt(STT_SCOPE);
                    } else {
                        self.set_stt(STT_WAIT);
                    }
                } else {
                    self.set_stt(STT_TELESCOPE);
                }
            }
            STAGE_TEARDOWN => {}
            _ => {
                self.attention = self.chk_attention(ctx);
                let result = match self.stt {
                    STT_WAIT => self.wait_1(ctx),
                    STT_SCOPE => self.wait_2(actor, ctx),
                    STT_POST => self.wait_3(actor, ctx),
                    STT_TELESCOPE => self.wait_4(ctx),
                    STT_TALK => self.talk_1(ctx),
                    _ => false,
                };
                self.set_eye_pos = result;
            }
        }
    }

    pub fn demo_action1(&mut self) {
        match self.stage {
            STAGE_INIT => {
                if self.kind == 1 {
                    self.set_anm_num(ANMNUM_DEMOWAIT, true);
                } else {
                    self.set_anm_num(ANMNUM_WAIT06, true);
                }
                self.stage = self.stage.wrapping_add(1);
            }
            STAGE_PENDING | STAGE_SEARCHED | STAGE_RUN | STAGE_TEARDOWN => {}
            _ => {}
        }
    }

    pub fn execute(
        &mut self,
        actor: &mut Actor,
        ctx: &mut Ctx,
        others: &Query<&Actor, Without<Aryll>>,
    ) {
        self.jnt.param = Self::default_jnt_param();
        if !self.primed {
            self.home = ctx.pos;
            self.home_angle = actor.angle;
            self.primed = true;
        }
        if self.draw_skip && ctx.demo_actor.unwrap_or(0) == 0 {
            return;
        }
        self.partner_search(others);
        self.check_order(ctx);
        if !self.demo(ctx.demo_actor) {
            let mut staff = -1;
            if ctx.text.info.event_run_check() && !ctx.talk_command {
                staff = self.is_event_entry();
            }
            if staff >= 0 || self.scope_active {
                self.event_proc(staff, ctx);
            } else {
                self.run_proc(actor, ctx);
            }
            self.look_back(actor, ctx);
            self.play_animation(ctx.ground_hit, ctx.reverb);
        } else {
            self.draw_skip = false;
        }
        self.event_order(ctx);
        self.angle = actor.angle;
        if !self.post_shaped {
            self.shape_angle = actor.angle;
        }
        self.set_attention(ctx.pos, false);
        if !self.m841 {
            self.collision_radius = COLLISION_RADIUS;
            self.collision_height = COLLISION_HEIGHT;
        }
    }
}

pub struct Text<'a> {
    pub procs: &'a mut MsgProcs,
    pub info: &'a mut MsgGameInfo,
    pub flags: &'a mut MsgFlags,
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
            None,
            eye,
        )
    }

    pub fn scope_message_set(&mut self, msg_no: u32) -> u32 {
        fop_msgm_scope_message_set(self.procs, self.info, msg_no)
    }

    pub fn force_send_on(&mut self) -> bool {
        let info = &*self.info;
        self.flags.force_send_on(info)
    }
}

pub struct Ctx<'a> {
    pub entity: Entity,
    pub pos: Vec3,
    pub player: Option<Player>,
    pub clear_count: u8,
    pub collect0: Option<u8>,
    pub select_item: [u8; ITEM_BUTTON_COUNT],
    pub frame: u32,
    pub attention: bool,
    pub demo_accept: bool,
    pub talk_command: bool,
    pub talk_xy: bool,
    pub present_done: bool,
    pub present_item: u8,
    pub demo_actor: Option<i32>,
    pub partner_chirp: bool,
    pub partner_odoroki: bool,
    pub scope_zoom: Option<f32>,
    pub scope_wipe: bool,
    pub camera_running: bool,
    pub ground_hit: bool,
    pub reverb: u8,
    pub project: Option<Vec3>,
    pub project_high: Option<Vec3>,
    pub fov_y: Option<f32>,
    pub cut_action: Option<i8>,
    pub cut_advance: bool,
    pub cut_proc: bool,
    pub stories: &'a mut StoryBits,
    pub list: &'a EventList,
    pub active: &'a mut ActiveEvent,
    pub events: &'a mut SimEvents,
    pub equipped: &'a Equipped,
    pub text: &'a mut Text<'a>,
}

fn collect0(ctx: &Ctx) -> u8 {
    ctx.collect0.unwrap_or(COLLECT_SLOT)
}

fn player_pos(ctx: &Ctx) -> Vec3 {
    ctx.player.map_or(ctx.pos, |player| player.pos)
}

fn player_eye_pos_of(ctx: &Ctx, offset_y: f32) -> Vec3 {
    ctx.player
        .map_or(ctx.pos, |player| {
            Vec3::new(player.pos.x, player.pos.y + offset_y, player.pos.z)
        })
}

fn rotated_y(offset: Vec3, pos: Vec3, yaw: i16) -> Vec3 {
    pos + Quat::from_rotation_y(from_s16(yaw)) * offset
}

fn target_angle_y(from: Vec3, to: Vec3) -> i16 {
    to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn table_of<T, const N: usize>(table: &[T; N], index: i32) -> Option<&T> {
    usize::try_from(index).ok().and_then(|slot| table.get(slot))
}

fn atr_tick(frame: u32) -> i16 {
    ATR_TICK_BASE.wrapping_add((frame & ATR_TICK_MASK) as i16)
}

fn calc_timer(timer: &mut i16) -> i16 {
    if *timer != 0 {
        *timer -= 1;
    }
    *timer
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

fn rnd_value(seed: &mut [i32; 3], min: i16, range: i16) -> i16 {
    min.wrapping_add((rnd(seed) * f32::from(range)) as i16)
}

pub fn is_aryll(name: &str) -> bool {
    name == ARYLL_ACTOR
}

pub fn cut_action_of(name: &str) -> Option<i8> {
    CUT_NAMES
        .iter()
        .position(|cut| *cut == name)
        .map(|index| index as i8)
}

#[derive(SystemParam)]
pub struct World<'w> {
    pub stories: ResMut<'w, StoryBits>,
    pub list: Res<'w, EventList>,
    pub active: ResMut<'w, ActiveEvent>,
    pub events: ResMut<'w, SimEvents>,
    pub equipped: Res<'w, Equipped>,
    pub collect: Res<'w, Collect>,
    pub frame: Res<'w, CurrentFrame>,
}

#[derive(SystemParam)]
pub struct Msg<'w, 's> {
    pub procs: ResMut<'w, MsgProcs>,
    pub info: ResMut<'w, MsgGameInfo>,
    pub flags: ResMut<'w, MsgFlags>,
    pub one: ResMut<'w, Msg1Box>,
    pub two: ResMut<'w, Msg2Box>,
    pub text_flags: Res<'w, TextFlags>,
    pub archive: Option<Res<'w, MsgArchives>>,
    pub players: Query<'w, 's, &'static Player>,
}

pub fn control(
    world: World,
    msg: Msg,
    mut others: Query<&Actor, Without<Aryll>>,
    mut actors: Query<(Entity, &mut Actor, &mut Aryll)>,
) {
    let player = msg.players.iter().next().copied();
    let frame = world.frame.0 as u32;
    let clear_count = msg.text_flags.clear_count;
    for (entity, actor, aryll) in &mut actors {
        let actor = &mut *actor;
        let aryll = &mut *aryll;
        let mut text = Text {
            procs: &mut msg.procs,
            info: &mut msg.info,
            flags: &mut msg.flags,
            one: &mut msg.one,
            two: &mut msg.two,
            clear_count,
            archive: msg
                .archive
                .as_deref()
                .map(|archive| archive as &dyn MsgArchive),
        };
        let mut ctx = Ctx {
            entity,
            pos: actor.pos,
            player,
            clear_count,
            collect0: Some(world.collect.check(0)),
            select_item: [ITEM_NONE; ITEM_BUTTON_COUNT],
            frame,
            attention: false,
            demo_accept: false,
            talk_command: false,
            talk_xy: false,
            present_done: false,
            present_item: ITEM_NONE,
            demo_actor: None,
            partner_chirp: false,
            partner_odoroki: false,
            scope_zoom: None,
            scope_wipe: false,
            camera_running: true,
            ground_hit: false,
            reverb: 0,
            project: None,
            project_high: None,
            fov_y: None,
            cut_action: None,
            cut_advance: false,
            cut_proc: false,
            stories: &mut world.stories,
            list: &world.list,
            active: &mut world.active,
            events: &mut world.events,
            equipped: &world.equipped,
            text: &mut text,
        };
        if !aryll.entered {
            aryll.entered = true;
            if !aryll.decide_type(ctx.stories) {
                continue;
            }
            if !aryll.create_init(actor, &mut ctx) {
                continue;
            }
        }
        aryll.execute(actor, &mut ctx, &others);
    }
}