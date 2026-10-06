use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::{Quat, Vec3};

use super::actor::Actor;
use super::area::StageName;
use super::collision::Collision;
use super::event::{ActiveEvent, EventList, ItemPartner, PresentItem};
use super::event_manager::EventManager;
use super::events::{SimEvent, SimEvents};
use super::flags::{Collect, EventFlags, MemBits, StoryBits};
use super::inventory::{self, Equipped};
use super::player::Player;
use super::pickup::Rupees;
use super::walk::from_s16;
use crate::assets::MsgArchives;
use crate::text::driver::{Counters, TextFlags};
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgGameInfo, MsgProcs, fop_msgm_message_set,
    fop_msgm_message_set_actor,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED,
    STATUS_MSG_ENDS,
};

pub const TORIPOST_ACTORS: [&str; 1] = ["Tpost"];

pub fn is_toripost(name: &str) -> bool {
    TORIPOST_ACTORS.contains(&name)
}

pub const ARC_NAME: &str = "Toripost";

const PARAM_SHIFT: u32 = 0x00;
const PARAM_MASK: u32 = 0x00;

const PROC_INIT: u32 = 0;
const PROC_EXEC: u32 = 1;

const MODE_WAIT: u32 = 0x0;
const MODE_TALK: u32 = 0x1;
const MODE_TALK_XY: u32 = 0x2;
const MODE_RECEIVE: u32 = 0x3;
const MODE_RECEIVE_DEMO: u32 = 0x4;
const MODE_NULL: u32 = 5;

const BCK_POST_GET: i8 = 0x0;
const BCK_POST_PUTOUT: i8 = 0x1;
const BCK_POST_WAIT: i8 = 0x2;

const ANMPRM_POST_GET0: i8 = 0x1;
const ANMPRM_POST_GET1: i8 = 0x2;
const ANMPRM_POST_PUTOUT: i8 = 0x3;
const ANMPRM_POST_WAIT: i8 = 0x4;
const ANMPRM_NULL: i8 = 0x5;

const EMODE_NONE: u32 = 0;
const EMODE_LOOP: u32 = 2;

const RES_BCK_POST_GET: i32 = 0x4;
const RES_BCK_POST_PUTOUT: i32 = 0x5;
const RES_BCK_POST_WAIT: i32 = 0x6;

const BCK_FRAME_MAX_POST_GET: f32 = 55.0;
const BCK_FRAME_MAX_POST_PUTOUT: f32 = 67.0;
const BCK_FRAME_MAX_POST_WAIT: f32 = 34.0;

const CUT_NAMES: [&str; 3] = ["PRESENT", "SET_ANM", "DISP_LETTER"];
const CUT_PRESENT: i8 = 0;
const CUT_SET_ANM: i8 = 1;
const CUT_DISP_LETTER: i8 = 2;
const CUT_KEY_NAME: &str = "Name";
const CUT_VALUE_HAKIDASU: &str = "HAKIDASU";

const DEMO_NAME_DEFAULT_POST: &str = "DEFAULT_POST";
const DEMO_NAME_TABLE: [&str; 1] = ["DEFAULT_POST"];

const NO_ACTION: i8 = -1;
const NO_STAFF: i32 = -1;

const SE_OBJ_POST_EAT_LUGGAGE: u32 = 0x6973;
const SE_OBJ_POST_LUGGAGE_OUT: u32 = 0x6974;

const PA_ID_IT_JN_LK_GEPPU00: u32 = 0x0057;
const PA_ID_IT_SN_POST_TSUBA00: u32 = 0x8190;

const ITEM_NONE: u8 = 0;
const ITEM_GREEN_RUPEE: u8 = 0x01;
const ITEM_RED_RUPEE: u8 = 0x04;
const ITEM_HEART_PIECE: u8 = 0x07;
const ITEM_BOMB_BAG: u8 = 0x31;
const ITEM_HEART_PIECE_ALT: u8 = 0x3F;
const ITEM_FATHER_LETTER: u8 = 0x98;
const ITEM_NOTE_TO_MOM: u8 = 0x99;
const ITEM_MAGGIES_LETTER: u8 = 0x9A;
const ITEM_MOBLINS_LETTER: u8 = 0x9B;
const ITEM_COMPLIMENTARY_ID: u8 = 0x9D;
const ITEM_FILL_UP_COUPON: u8 = 0x9E;
const ITEM_COLLECT_MAP_60: u8 = 0xC3;
const ITEM_COLLECT_MAP_52: u8 = 0xCB;
const ITEM_NO_ITEM: u8 = 0xFF;

const LETTER_NOSEND: u8 = 0;
const LETTER_SEND: u8 = 1;
const LETTER_STOCK: u8 = 2;
const LETTER_READ: u8 = 3;

const LETTER_ROCK_SPIRE_SHOP_AD: u16 = 0x7A03;
const LETTER_ORCA: u16 = 0x7B03;
const LETTER_BAITO: u16 = 0x7C03;
const LETTER_BOMBS_AD: u16 = 0x7D03;
const LETTER_ARYLL: u16 = 0x8B03;
const LETTER_GRANDMA: u16 = 0x9D03;
const LETTER_BAITOS_MOM: u16 = 0xAC03;
const LETTER_HOSKITS_GIRLFRIEND: u16 = 0xAE03;
const LETTER_GOLD_MEMBERSHIP: u16 = 0xAF03;
const LETTER_SILVER_MEMBERSHIP: u16 = 0xB003;
const LETTER_TINGLE: u16 = 0xB203;
const LETTER_KOMALIS_FATHER: u16 = 0xB503;

const EVENT_BIT_UNK_1220: u16 = 0x1220;
const EVENT_BIT_UNK_1E80: u16 = 0x1E80;

const SYMBOL_NAYRU: u8 = 0;
const SYMBOL_DIN: u8 = 1;
const SYMBOL_FARORE: u8 = 2;

const STAGE_SEA_NAME: &str = "sea";

const MSG_CE5: u32 = 0x0CE5;
const MSG_CE6: u32 = 0x0CE6;
const MSG_CE7: u32 = 0x0CE7;
const MSG_CE8: u32 = 0x0CE8;
const MSG_CE9: u32 = 0x0CE9;
const MSG_CEA: u32 = 0x0CEA;
const MSG_CEB: u32 = 0x0CEB;
const MSG_CEC: u32 = 0x0CEC;
const MSG_CED: u32 = 0x0CED;
const MSG_CEE: u32 = 0x0CEE;
const MSG_CEF: u32 = 0x0CEF;
const MSG_CF0: u32 = 0x0CF0;
const MSG_CF1: u32 = 0x0CF1;
const MSG_CF2: u32 = 0x0CF2;
const MSG_CF3: u32 = 0x0CF3;
const MSG_CF4: u32 = 0x0CF4;
const MSG_CF5: u32 = 0x0CF5;
const MSG_CF6: u32 = 0x0CF6;
const MSG_CF7: u32 = 0x0CF7;
const MSG_CF8: u32 = 0x0CF8;

const PAY_MSG_NUM: [u32; 3] = [0x0CEC, 0x0CED, 0x0CEE];

const SEND_PRICE: [i32; 3] = [0x05, 0x0A, 0x14];

const SEND_PRICE_LETTER: i32 = 0x0A;
const SEND_PRICE_LETTER_TINGLE: i32 = 0xC9;

const ISLE_IDX_COUNT: usize = 49;

const PAY_TYPE: [u8; ISLE_IDX_COUNT] = [
    2, 1, 1, 0, 0, 0, 0, 2,
    1, 1, 0, 0, 0, 0, 2, 1,
    1, 0, 0, 0, 0, 2, 1, 1,
    0, 0, 0, 0, 2, 1, 1, 1,
    1, 1, 1, 2, 1, 1, 1, 1,
    1, 1, 2, 2, 2, 2, 2, 2,
    2,
];

const EVT_CND_CANTALK: u16 = 0x0001;
// `dEvtCnd_CANTALKITEM_e = 0x0020` (`f_op_actor.h:150`). This was 0x0002, which is
// `dEvtCnd_UNK2_e` (`f_op_actor.h:146`) - the sibling CANTALK constant copied off the
// wrong enum row, so `order_flags.set(...)` was raising a bit nothing tests.
const EVT_CND_CANTALKITEM: u16 = 0x0020; // <- f_op_actor.h:150

const EVENT_IDX_NONE: i8 = 0;
const EVENT_IDX_TALK: i8 = 1;
const EVENT_IDX_TALK_ITEM: i8 = 2;
const EVENT_IDX_RECEIVE: i8 = 3;
const EVENT_IDX_OTHER_BASE: i8 = 3;

const SCENE_EVENT_ORDER: u16 = 0x14F;

const COLLISION_RADIUS: f32 = 40.0;
const COLLISION_HEIGHT: f32 = 140.0;
const CULL_FAR: f32 = 10.0;
const CULL_MIN: [f32; 3] = [-50.0, 0.0, -50.0];
const CULL_MAX: [f32; 3] = [70.0, 200.0, 70.0];

const ROLL_PLAYER_CRASH_RADIUS: f32 = 40.0;
const ROLL_PLAYER_CRASH_KIND: u8 = 7;
const HIT_SE_KIND: u8 = 0x0B;

const ATTENTION_TALK: u8 = 5;
const ATTENTION_SPEAK: u8 = 6;

const GRAVITY: f32 = -4.5;
const MAX_FALL_SPEED: f32 = -100.0;
const ACCH_CIR_WALL: f32 = 30.0;

const TIMER_SURPRISE: i32 = 10;
const TIMER_DISABLED: i32 = -1;

const LETTER_COUNT: usize = 13;

const JOINT_COUNT: usize = 3;
const JOINT_ROOT_APOST: [f32; 3] = [0.0, 0.0, 0.0];
const JOINT_BODY: [f32; 3] = [0.0, 14.90, 0.0];
const JOINT_MOUTH: [f32; 3] = [0.01, 93.76, 33.09];
const JOINT_MOUTH_INDEX: usize = 2;

const MESSAGE_COUNT_BASE: u32 = 1;

const PARTICLE_SCALE: f32 = 1.0;
const PARTICLE_SCALE_XY: f32 = 2.0;
const PARTICLE_LIFE: i32 = -1;
const PARTICLE_ALPHA: u8 = 0xFF;
const PARTICLE_COLOR_ADD: i32 = 0x80;
const PARTICLE_COLOR_CLAMP: i32 = 255;

const REG12_F0: f32 = 0.0;
const REG12_F1: f32 = 0.0;
const REG12_F2: f32 = 0.0;
const REG12_S0: i32 = 0;
const REG12_S1: i32 = 0;
const REG12_S2: i32 = 0;

const DAY_TIME_DAY: i32 = 0;

const NO_STATUS: u8 = 0xFF;
const TALK_PARAM_NO_GET_MSG: u8 = 0;
const TALK_PARAM_GET_MSG: u8 = 1;
const TALK_MODE_LETTER_EYE: u8 = 5;

const PRESENT_ITEM_ACTOR: &str = "PresentItem";
const PRESENT_ITEM_SET_ID: u16 = 0;
const PRESENT_ITEM_ROOM: i8 = -1;

const RESERVE_ITEM_BTN: usize = 0;

const CYL_AT_TYPE_SRC: u32 = 0;
const CYL_AT_ATP_SRC: u8 = 0;
const CYL_AT_SPRM_SRC: u32 = 0;
const CYL_TG_TYPE_SRC: u32 = !0x0040;
const CYL_TG_SPRM_SRC: u32 = 0x01 | 0x0E;
const CYL_CO_SPRM_SRC: u32 = 0x01 | 0x08 | 0x0070;

const CC_TG_SPRM_SET: u32 = 0x01;
const CC_CO_SPRM_SET: u32 = 0x01;
const CC_CO_SPRM_IS_OTHER: u32 = 0x08;

const COLLIDE_REACH: f32 = 20.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Letter {
    pub flag_0x00: bool,
    pub msg_no: u32,
    pub item_no: u8,
    pub event_reg: u16,
}

const LETTERS: [Letter; LETTER_COUNT] = [
    Letter {
        flag_0x00: false,
        msg_no: 0x1AAF,
        item_no: ITEM_HEART_PIECE,
        event_reg: LETTER_BAITOS_MOM,
    },
    Letter {
        flag_0x00: false,
        msg_no: 0x0CF9,
        item_no: ITEM_HEART_PIECE,
        event_reg: LETTER_KOMALIS_FATHER,
    },
    Letter {
        flag_0x00: false,
        msg_no: 0x0CFA,
        item_no: ITEM_COLLECT_MAP_60,
        event_reg: LETTER_BOMBS_AD,
    },
    Letter {
        flag_0x00: false,
        msg_no: 0x0CFC,
        item_no: ITEM_RED_RUPEE,
        event_reg: LETTER_ORCA,
    },
    Letter {
        flag_0x00: false,
        msg_no: 0x0805,
        item_no: ITEM_RED_RUPEE,
        event_reg: LETTER_GRANDMA,
    },
    Letter {
        flag_0x00: false,
        msg_no: 0x0CFD,
        item_no: ITEM_GREEN_RUPEE,
        event_reg: LETTER_ROCK_SPIRE_SHOP_AD,
    },
    Letter {
        flag_0x00: true,
        msg_no: 0x0DB6,
        item_no: ITEM_COLLECT_MAP_52,
        event_reg: LETTER_TINGLE,
    },
    Letter {
        flag_0x00: false,
        msg_no: 0x1148,
        item_no: ITEM_RED_RUPEE,
        event_reg: LETTER_ARYLL,
    },
    Letter {
        flag_0x00: false,
        msg_no: 0x1AAF,
        item_no: ITEM_HEART_PIECE,
        event_reg: LETTER_BAITOS_MOM,
    },
    Letter {
        flag_0x00: true,
        msg_no: 0x0F76,
        item_no: ITEM_COMPLIMENTARY_ID,
        event_reg: LETTER_SILVER_MEMBERSHIP,
    },
    Letter {
        flag_0x00: false,
        msg_no: 0x19A6,
        item_no: ITEM_HEART_PIECE_ALT,
        event_reg: LETTER_HOSKITS_GIRLFRIEND,
    },
    Letter {
        flag_0x00: true,
        msg_no: 0x0CFB,
        item_no: ITEM_RED_RUPEE,
        event_reg: LETTER_BAITO,
    },
    Letter {
        flag_0x00: true,
        msg_no: 0x0F77,
        item_no: ITEM_FILL_UP_COUPON,
        event_reg: LETTER_GOLD_MEMBERSHIP,
    },
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnmPrm {
    pub anm_idx: i8,
    pub next_prm_idx: i8,
    pub field_0x02: i16,
    pub morf: f32,
    pub play_speed: f32,
    pub loop_mode: u32,
}

const ANM_PRM_ROWS: usize = 5;
const ANM_BCKS_ROWS: usize = 3;

const ANM_BCKS: [i32; ANM_BCKS_ROWS] =
    [RES_BCK_POST_GET, RES_BCK_POST_PUTOUT, RES_BCK_POST_WAIT];

const ANM_PRM: [AnmPrm; ANM_PRM_ROWS] = [
    AnmPrm {
        anm_idx: -1,
        next_prm_idx: -1,
        field_0x02: 0,
        morf: 8.0,
        play_speed: 1.0,
        loop_mode: EMODE_LOOP,
    },
    AnmPrm {
        anm_idx: BCK_POST_GET,
        next_prm_idx: -1,
        field_0x02: 0,
        morf: 8.0,
        play_speed: 0.0,
        loop_mode: EMODE_NONE,
    },
    AnmPrm {
        anm_idx: BCK_POST_GET,
        next_prm_idx: -1,
        field_0x02: 0,
        morf: 8.0,
        play_speed: 1.0,
        loop_mode: EMODE_NONE,
    },
    AnmPrm {
        anm_idx: BCK_POST_PUTOUT,
        next_prm_idx: -1,
        field_0x02: 0,
        morf: 8.0,
        play_speed: 1.0,
        loop_mode: EMODE_NONE,
    },
    AnmPrm {
        anm_idx: BCK_POST_WAIT,
        next_prm_idx: -1,
        field_0x02: 0,
        morf: 8.0,
        play_speed: 1.0,
        loop_mode: EMODE_LOOP,
    },
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cyl {
    pub centre: Vec3,
    pub radius: f32,
    pub height: f32,
    pub at_type: u32,
    pub at_atp: u8,
    pub at_sprm: u32,
    pub tg_type: u32,
    pub tg_sprm: u32,
    pub co_sprm: u32,
    pub stts_set: bool,
}

impl Cyl {
    const SRC: Cyl = Cyl {
        centre: Vec3::ZERO,
        radius: 0.0,
        height: 0.0,
        at_type: CYL_AT_TYPE_SRC,
        at_atp: CYL_AT_ATP_SRC,
        at_sprm: CYL_AT_SPRM_SRC,
        tg_type: CYL_TG_TYPE_SRC,
        tg_sprm: CYL_TG_SPRM_SRC,
        co_sprm: CYL_CO_SPRM_SRC,
        stts_set: false,
    };

    pub fn set(&mut self, other: Cyl) {
        *self = other;
    }

    pub fn set_stts(&mut self) {
        self.stts_set = true;
    }

    pub fn at_flag(&self) -> u32 {
        if self.at_type == 0 {
            return 0;
        }
        self.at_sprm
    }

    pub fn co_flag(&self) -> u32 {
        self.co_sprm
    }

    pub fn blocks(&self, at: Vec3) -> bool {
        let co = self.co_flag();
        if co & CC_CO_SPRM_SET == 0
            || co & CC_CO_SPRM_IS_OTHER == 0
            || self.radius <= 0.0
        {
            return false;
        }
        let dx = self.centre.x - at.x;
        let dz = self.centre.z - at.z;
        if (dx * dx + dz * dz).sqrt() >= self.radius {
            return false;
        }
        let bottom = self.centre.y;
        let top = self.centre.y + self.height;
        at.y + COLLIDE_REACH >= bottom && at.y - COLLIDE_REACH <= top
    }

    pub fn chk_tg_hit(&self, player: Option<Vec3>) -> bool {
        if self.tg_type == 0 || self.tg_sprm & CC_TG_SPRM_SET == 0 || self.radius <= 0.0 {
            return false;
        }
        let Some(at) = player else {
            return false;
        };
        self.blocks(at)
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Statics {
    pub no: i8,
    pub debug_draw: bool,
    pub field_0x06: i8,
    pub field_0x07: u8,
    pub attn_pos_offset: f32,
    pub eye_pos_offset: f32,
    pub talk_distance: f32,
    pub field_0x14: i16,
    pub field_0x16: i16,
    pub talk_xy: bool,
    pub pre_item_no: u8,
}

impl Default for Statics {
    fn default() -> Self {
        Self {
            no: -1,
            debug_draw: false,
            field_0x06: 0,
            field_0x07: 0,
            attn_pos_offset: 140.0,
            eye_pos_offset: 100.0,
            talk_distance: 300.0,
            field_0x14: 30,
            field_0x16: 30,
            talk_xy: false,
            pre_item_no: ITEM_NO_ITEM,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Toripost {
    pub prm: u32,
    pub mode: u32,
    pub bck_idx: i8,
    pub anm_prm_idx: i8,
    pub old_anm_prm_idx: i8,
    pub field_0x8dc: i32,
    pub field_0x8e0: i32,
    pub field_0x8e4: i32,
    pub pay_type: u8,
    pub field_0x8e9: u8,
    pub field_0x8ea: u8,
    pub field_0x8eb: u8,
    pub num_readable: i32,
    pub field_0x8f0: u32,
    pub field_0x8f4: u8,
    pub field_0x8f5: u8,
    pub pre_item_no: u8,
    pub event_idx: i8,
    pub staff_idx: i32,
    pub curr_msg_no: u32,
    pub curr_msg_bs_pc_id: u32,
    pub msg_bound: bool,
    pub event_conditions: u16,
    pub attention_pos: Vec3,
    pub eye_pos: Vec3,
    pub attention_talk: u8,
    pub attention_speak: u8,
    pub shape_angle: [i16; 3],
    pub model_pos: Vec3,
    pub model_scale: Vec3,
    pub collider: Cyl,
    pub collision_radius: f32,
    pub collision_height: f32,
    pub cull_far: f32,
    pub ground_h: f32,
    pub gravity: f32,
    pub speed: Vec3,
    pub speed_f: f32,
    pub wall_x: f32,
    pub wall_y: f32,
    pub morf_frame: f32,
    pub morf_cur: f32,
    pub morf_max: f32,
    pub morf_speed: f32,
    pub morf_loop: bool,
    pub morf_stopped: bool,
    pub bind: [Vec3; JOINT_COUNT],
    pub status: u32,
    pub entered: bool,
}

impl Toripost {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            prm: (parameters >> PARAM_SHIFT) & PARAM_MASK,
            mode: MODE_NULL,
            bck_idx: 0,
            anm_prm_idx: 0,
            old_anm_prm_idx: 0,
            field_0x8dc: 0,
            field_0x8e0: 0,
            field_0x8e4: 0,
            pay_type: 0,
            field_0x8e9: 0,
            field_0x8ea: 0,
            field_0x8eb: 0,
            num_readable: 0,
            field_0x8f0: MESSAGE_COUNT_BASE,
            field_0x8f4: 0,
            field_0x8f5: 0,
            pre_item_no: ITEM_NO_ITEM,
            event_idx: EVENT_IDX_NONE,
            staff_idx: NO_STAFF,
            curr_msg_no: 0,
            curr_msg_bs_pc_id: FPC_M_ERROR_PROCESS_ID,
            msg_bound: false,
            event_conditions: 0,
            attention_pos: Vec3::ZERO,
            eye_pos: Vec3::ZERO,
            attention_talk: ATTENTION_TALK,
            attention_speak: ATTENTION_SPEAK,
            shape_angle: [0; 3],
            model_pos: Vec3::ZERO,
            model_scale: Vec3::ONE,
            collider: Cyl::SRC,
            collision_radius: 0.0,
            collision_height: 0.0,
            cull_far: 0.0,
            ground_h: 0.0,
            gravity: 0.0,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            wall_x: ACCH_CIR_WALL,
            wall_y: ACCH_CIR_WALL,
            morf_frame: 0.0,
            morf_cur: 0.0,
            morf_max: 0.0,
            morf_speed: 0.0,
            morf_loop: false,
            morf_stopped: true,
            bind: [
                Vec3::from_array(JOINT_ROOT_APOST),
                Vec3::from_array(JOINT_BODY),
                Vec3::from_array(JOINT_MOUTH),
            ],
            status: 0x18 | 0x0000_0020 | 0x0000_0100 | 0x0004_0000 | 0x0020_0000,
            entered: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (
            CULL_MIN,
            [
                CULL_MAX[0].max(self.collision_radius),
                CULL_MAX[1].max(self.collision_height),
                CULL_MAX[2].max(self.collision_radius),
            ],
        )
    }

    pub fn drawn(&self) -> bool {
        true
    }

    pub fn is_anm(&self, idx: i8) -> bool {
        self.anm_prm_idx == idx
    }

    pub fn send_price(&self) -> i32 {
        SEND_PRICE
            .get(usize::from(self.pay_type))
            .copied()
            .unwrap_or(0)
    }

    pub fn letter(&self, index: i32) -> Option<Letter> {
        LETTERS.get(usize::try_from(index).ok()?).copied()
    }

    pub fn letter_flag(&self, index: i32) -> bool {
        self.letter(index).is_some_and(|entry| entry.flag_0x00)
    }

    pub fn letter_msg(&self, index: i32) -> u32 {
        self.letter(index).map_or(0, |entry| entry.msg_no)
    }

    pub fn letter_item(&self, index: i32) -> u8 {
        self.letter(index).map_or(0, |entry| entry.item_no)
    }

    pub fn letter_reg(&self, index: i32) -> u16 {
        self.letter(index).map_or(0, |entry| entry.event_reg)
    }

    pub fn event_set(&mut self, event_idx: i8) {
        self.event_idx = event_idx;
    }

    pub fn get_arg(&mut self) {}

    pub fn set_collision(&mut self, radius: f32, height: f32) {
        self.collision_radius = radius;
        self.collision_height = height;
    }

    pub fn set_attention(&mut self, pos: Vec3, statics: &Statics) {
        self.attention_pos = Vec3::new(pos.x, pos.y + statics.attn_pos_offset, pos.z);
        self.eye_pos = Vec3::new(pos.x, pos.y + statics.eye_pos_offset, pos.z);
    }

    pub fn set_mtx(&mut self, pos: Vec3) {
        self.model_pos = pos;
    }

    pub fn set_base_scale(&mut self, scale: Vec3) {
        self.model_scale = scale;
    }

    pub fn mouth_world(&self, local: Vec3) -> Vec3 {
        let yaw = from_s16(self.shape_angle[1]);
        self.model_pos + Quat::from_rotation_y(yaw) * local
    }

    pub fn mouth_bind(&self) -> Vec3 {
        self.bind
            .get(JOINT_MOUTH_INDEX)
            .copied()
            .unwrap_or(Vec3::ZERO)
    }

    pub fn check_talk(&self, pos: Vec3, player: Option<Vec3>, statics: &Statics) -> bool {
        let target = player.unwrap_or(pos);
        let dx = target.x - pos.x;
        let dz = target.z - pos.z;
        (dx * dx + dz * dz).sqrt() < statics.talk_distance
    }

    pub fn deliver_letter(&mut self, ctx: &mut Ctx) {
        match self.pre_item_no {
            ITEM_MAGGIES_LETTER => {
                ctx.stories.set(EVENT_BIT_UNK_1220);
            }
            ITEM_NOTE_TO_MOM => {
                letter_send(ctx.stories, LETTER_BAITOS_MOM);
            }
            _ => {}
        }
    }

    pub fn get_receive_letter_num(&self, stories: &StoryBits) -> i16 {
        let mut num: i16 = 0;
        for entry in LETTERS.iter().skip(1) {
            if stories.reg(entry.event_reg) == LETTER_STOCK {
                num += 1;
            }
        }
        num
    }

    pub fn get_readable_letter_num(&self, stories: &StoryBits) -> i32 {
        let mut num: i32 = 0;
        let letter_count = i32::try_from(LETTER_COUNT).unwrap_or(0);
        let start_idx = if self.num_readable != 0 {
            self.num_readable
        } else {
            1
        };
        let readable = self.num_readable;

        if readable < letter_count {
            for i in start_idx..letter_count {
                let Some(entry) = self.letter(i) else {
                    continue;
                };
                if stories.reg(entry.event_reg) != LETTER_STOCK {
                    continue;
                }
                num = i;
                break;
            }
        }

        num
    }

    pub fn check_send_price(&self, stage_name: &str, room: i8) -> u8 {
        if stage_name != STAGE_SEA_NAME {
            return 0;
        }
        PAY_TYPE
            .get(usize::try_from(room).unwrap_or(usize::MAX))
            .copied()
            .unwrap_or(0)
    }

    pub fn get_msg_xy(&mut self, ctx: &mut Ctx) -> u32 {
        let pos = Vec3::new(REG12_F0, REG12_F1, REG12_F2);
        let scale = Vec3::splat(PARTICLE_SCALE_XY);
        let msg_no;

        match self.pre_item_no {
            ITEM_NOTE_TO_MOM | ITEM_MAGGIES_LETTER => {
                msg_no = MSG_CE8;
                let red = (REG12_S0 + PARTICLE_COLOR_ADD).clamp(0, PARTICLE_COLOR_CLAMP) as u8;
                let green = (REG12_S1 + PARTICLE_COLOR_ADD).clamp(0, PARTICLE_COLOR_CLAMP) as u8;
                let blue = (REG12_S2 + PARTICLE_COLOR_ADD).clamp(0, PARTICLE_COLOR_CLAMP) as u8;
                let colour = [red, green, blue, PARTICLE_ALPHA];
                let pos = self.mouth_world(pos);
                ctx.particles.push(Particle {
                    id: PA_ID_IT_JN_LK_GEPPU00,
                    pos,
                    angle: self.shape_angle,
                    scale,
                    life: PARTICLE_LIFE,
                    colour: Some(colour),
                });
            }
            ITEM_FATHER_LETTER | ITEM_MOBLINS_LETTER => {
                self.set_anm(ANMPRM_POST_PUTOUT, false, ctx);
                self.field_0x8ea = 1;
                msg_no = MSG_CEA;
            }
            _ => {
                self.set_anm(ANMPRM_POST_PUTOUT, false, ctx);
                self.field_0x8ea = 1;
                msg_no = MSG_CE9;
            }
        }

        msg_no
    }

    pub fn get_msg_normal(&mut self, ctx: &mut Ctx) -> u32 {
        if self.field_0x8eb != 0 {
            let count = i32::try_from(self.field_0x8f0).unwrap_or(0);
            ctx.set_message_count_number(count);
            let msg_no = MSG_CF7;
            self.field_0x8eb = 0;
            msg_no
        } else if ctx.daynight == DAY_TIME_DAY {
            MSG_CE5
        } else {
            MSG_CE6
        }
    }

    pub fn get_msg(&mut self, ctx: &mut Ctx) -> u32 {
        if ctx.chk_talk_xy() {
            self.get_msg_xy(ctx)
        } else {
            self.get_msg_normal(ctx)
        }
    }

    pub fn next_msg_status(&mut self, ctx: &mut Ctx, msg_no: &mut u32, select_num: u8) -> u8 {
        let mut status = STATUS_MSG_CONTINUES;
        match *msg_no {
            MSG_CE5 | MSG_CE6 => {
                if self.num_readable != 0 {
                    let count = i32::from(self.get_receive_letter_num(&*ctx.stories));
                    ctx.set_message_count_number(count);
                    *msg_no = MSG_CEB;
                } else {
                    *msg_no = MSG_CE7;
                }
            }
            MSG_CEB => {
                let count = i32::try_from(self.field_0x8f0).unwrap_or(0);
                ctx.set_message_count_number(count);
                *msg_no = MSG_CF7;
            }
            MSG_CF7 => {
                if ctx.statics.field_0x07 != 0 || self.letter_flag(self.num_readable) {
                    *msg_no = MSG_CF3;
                } else {
                    self.field_0x8e9 = 1;
                    status = STATUS_MSG_ENDS;
                }
            }
            MSG_CE8 => {
                *msg_no = PAY_MSG_NUM
                    .get(usize::from(self.pay_type))
                    .copied()
                    .unwrap_or(0);
            }
            MSG_CEC | MSG_CED | MSG_CEE => {
                *msg_no = MSG_CEF;
            }
            MSG_CEF => {
                if select_num == 0 {
                    let price = self.send_price();
                    if ctx.rupees() >= price {
                        ctx.add_rupees(-price);
                        ctx.set_reserve_item_empty();
                        self.deliver_letter(ctx);
                        *msg_no = MSG_CF2;
                    } else {
                        self.set_anm(ANMPRM_POST_PUTOUT, false, ctx);
                        self.field_0x8ea = 1;
                        *msg_no = MSG_CF1;
                    }
                } else {
                    self.set_anm(ANMPRM_POST_PUTOUT, false, ctx);
                    self.field_0x8ea = 1;
                    *msg_no = MSG_CF0;
                }
            }
            MSG_CF3 => {
                if self.letter_reg(self.num_readable) == LETTER_TINGLE {
                    *msg_no = MSG_CF8;
                } else {
                    *msg_no = MSG_CF4;
                }
            }
            MSG_CF4 | MSG_CF8 => {
                if select_num == 0 {
                    let mut price = SEND_PRICE_LETTER;
                    if self.letter_reg(self.num_readable) == LETTER_TINGLE {
                        price = SEND_PRICE_LETTER_TINGLE;
                    }
                    if ctx.rupees() >= price {
                        ctx.add_rupees(-price);
                        self.field_0x8e9 = 1;
                        status = STATUS_MSG_ENDS;
                    } else {
                        *msg_no = MSG_CF6;
                    }
                } else {
                    *msg_no = MSG_CF5;
                }
            }
            MSG_CF5 | MSG_CF6 => {
                self.num_readable += 1;
                self.field_0x8f0 += 1;
                self.num_readable = self.get_readable_letter_num(&*ctx.stories);
                if self.num_readable != 0 {
                    let count = i32::try_from(self.field_0x8f0).unwrap_or(0);
                    ctx.set_message_count_number(count);
                    *msg_no = MSG_CF7;
                } else {
                    status = STATUS_MSG_ENDS;
                }
            }
            _ => {
                status = STATUS_MSG_ENDS;
            }
        }

        if self.curr_msg_no == self.letter_msg(self.num_readable) && status == STATUS_MSG_ENDS {
            status = STATUS_MSG_DISPLAYED;
        }

        status
    }

    pub fn set_anm(&mut self, anm_prm_idx: i8, force: bool, ctx: &mut Ctx) {
        if anm_prm_idx != ANMPRM_NULL {
            self.anm_prm_idx = anm_prm_idx;
        }

        if self.bck_idx == BCK_POST_GET && self.morf_frame == 1.0 {
            ctx.events.list.push(SimEvent::Se(SE_OBJ_POST_EAT_LUGGAGE));
        }

        if self.bck_idx == BCK_POST_PUTOUT && self.morf_frame == 1.0 {
            let pos = ctx.pos;
            let angle = ctx.angle;
            ctx.particles.push(Particle {
                id: PA_ID_IT_SN_POST_TSUBA00,
                pos,
                angle,
                scale: Vec3::splat(PARTICLE_SCALE),
                life: PARTICLE_LIFE,
                colour: None,
            });
            ctx.events.list.push(SimEvent::Se(SE_OBJ_POST_LUGGAGE_OUT));
        }

        self.lib_bcks_set_anm(force);
    }

    pub fn lib_bcks_set_anm(&mut self, force: bool) {
        let Some(prm) = ANM_PRM
            .get(usize::try_from(self.anm_prm_idx).unwrap_or(usize::MAX))
            .copied()
        else {
            return;
        };
        if (self.old_anm_prm_idx != self.anm_prm_idx && prm.anm_idx != -1) || force {
            self.bck_idx = prm.anm_idx;
            self.morf_max = bck_frame_max(self.bck_idx);
            self.morf_cur = 0.0;
            self.morf_frame = 0.0;
            self.morf_speed = prm.play_speed;
            self.morf_loop = prm.loop_mode == EMODE_LOOP;
            self.morf_stopped = false;
        }

        self.old_anm_prm_idx = self.anm_prm_idx;

        if self.morf_stopped && prm.next_prm_idx != -1 && prm.loop_mode == EMODE_NONE {
            self.anm_prm_idx = prm.next_prm_idx;
        }
    }

    pub fn mode_proc_init(&mut self, new_mode: u32, ctx: &mut Ctx) {
        self.mode = new_mode;
        match self.mode {
            MODE_WAIT => self.mode_wait_init(ctx),
            MODE_TALK => self.mode_talk_init(ctx),
            MODE_TALK_XY => self.mode_talk_xy_init(ctx),
            MODE_RECEIVE => self.mode_receive_init(ctx),
            MODE_RECEIVE_DEMO => self.mode_receive_demo_init(ctx),
            _ => {}
        }
    }

    pub fn mode_proc_exec(&mut self, ctx: &mut Ctx) {
        match self.mode {
            MODE_WAIT => self.mode_wait(ctx),
            MODE_TALK => self.mode_talk(ctx),
            MODE_TALK_XY => self.mode_talk_xy(ctx),
            MODE_RECEIVE => self.mode_receive(ctx),
            MODE_RECEIVE_DEMO => self.mode_receive_demo(ctx),
            _ => {}
        }
    }

    pub fn mode_wait_init(&mut self, ctx: &mut Ctx) {
        self.field_0x8f0 = MESSAGE_COUNT_BASE;
        if self.num_readable != 0 {
            self.set_anm(ANMPRM_POST_WAIT, false, ctx);
        } else {
            self.set_anm(ANMPRM_POST_GET0, false, ctx);
        }
    }

    pub fn mode_wait(&mut self, ctx: &mut Ctx) {
        if self.num_readable != 0 {
            self.set_anm(ANMPRM_POST_WAIT, false, ctx);
        } else {
            self.set_anm(ANMPRM_POST_GET0, false, ctx);
        }

        if self.field_0x8f4 != 0 {
            self.mode_proc_init(MODE_TALK, ctx);
        } else if self.field_0x8f5 != 0 {
            self.mode_proc_init(MODE_TALK_XY, ctx);
        } else if self.check_talk(ctx.pos, ctx.player, ctx.statics) {
            self.event_set(EVENT_IDX_TALK_ITEM);
        }
    }

    pub fn mode_talk_init(&mut self, ctx: &mut Ctx) {
        self.set_anm(ANMPRM_POST_GET0, false, ctx);
    }

    pub fn mode_talk(&mut self, ctx: &mut Ctx) {
        let status = self.talk(TALK_PARAM_GET_MSG, ctx);
        if status == STATUS_BOX_CLOSED {
            if self.field_0x8e9 != 0 {
                self.mode_proc_init(MODE_RECEIVE, ctx);
                self.field_0x8e9 = 0;
            } else {
                self.mode_proc_init(MODE_WAIT, ctx);
            }

            ctx.event_reset();
            self.field_0x8f4 = 0;
        }
    }

    pub fn mode_talk_xy_init(&mut self, ctx: &mut Ctx) {
        self.set_anm(ANMPRM_POST_GET0, false, ctx);
        self.pre_item_no = ctx.pre_item_no();
        self.field_0x8dc = i32::from(ctx.statics.field_0x14);
        self.field_0x8e0 = i32::from(ctx.statics.field_0x16);
    }

    pub fn mode_talk_xy(&mut self, ctx: &mut Ctx) {
        if self.is_anm(ANMPRM_POST_PUTOUT) {
            if self.field_0x8ea != 0 {
                self.field_0x8e4 = TIMER_SURPRISE;
                self.field_0x8ea = 0;
            }

            if self.field_0x8e4 != TIMER_DISABLED && calc_timer(&mut self.field_0x8e4) == 0 {
                ctx.change_original_demo();
                ctx.change_demo_mode(2);

                self.field_0x8e4 = TIMER_DISABLED;
            }

            match self.curr_msg_no {
                MSG_CE9 | MSG_CEA | MSG_CF0 | MSG_CF1 => {
                    if ctx.base_anime_frame_rate() == 0.0 {
                        ctx.change_original_demo();
                        ctx.change_demo_mode(1);
                    }
                }
                _ => {}
            }
        }

        if self.is_anm(ANMPRM_POST_GET0)
            && ctx.chk_present_end()
            && calc_timer(&mut self.field_0x8e0) == 0
        {
            ctx.cancel_present();
            self.set_anm(ANMPRM_POST_GET1, false, ctx);
        }

        if self.is_anm(ANMPRM_POST_GET1) || self.is_anm(ANMPRM_POST_PUTOUT) {
            if self.morf_stopped
                && calc_timer(&mut self.field_0x8dc) == 0
                && self.talk(TALK_PARAM_GET_MSG, ctx) == STATUS_BOX_CLOSED
            {
                self.mode_proc_init(MODE_WAIT, ctx);
                ctx.event_reset();
                self.field_0x8f5 = 0;
            }
        }
    }

    pub fn mode_receive_init(&mut self, ctx: &mut Ctx) {
        ctx.event_reset();
        self.event_set(EVENT_IDX_RECEIVE);
        self.set_anm(ANMPRM_POST_GET0, false, ctx);
    }

    pub fn mode_receive(&mut self, ctx: &mut Ctx) {
        self.mode_proc_init(MODE_RECEIVE_DEMO, ctx);
    }

    pub fn mode_receive_demo_init(&mut self, _ctx: &mut Ctx) {}

    pub fn mode_receive_demo(&mut self, ctx: &mut Ctx) {
        if ctx.end_check(DEMO_NAME_DEFAULT_POST) {
            let reg = self.letter_reg(self.num_readable);
            ctx.stories.set_reg(reg, LETTER_READ);
            self.num_readable = self.get_readable_letter_num(&*ctx.stories);
            ctx.event_reset();

            if self.num_readable != 0 {
                self.mode_proc_init(MODE_TALK, ctx);
                self.event_set(EVENT_IDX_TALK);
                self.field_0x8eb = 1;
                self.field_0x8f0 += 1;
            } else {
                self.mode_proc_init(MODE_WAIT, ctx);
                self.field_0x8f0 = MESSAGE_COUNT_BASE;
            }
        }
    }

    pub fn event_order(&mut self, ctx: &mut Ctx) {
        if self.event_idx == EVENT_IDX_TALK || self.event_idx == EVENT_IDX_TALK_ITEM {
            self.event_conditions |= EVT_CND_CANTALK;
            self.event_conditions |= EVT_CND_CANTALKITEM;

            if self.event_idx == EVENT_IDX_TALK {
                ctx.order_speak_event();
            }
        } else if self.event_idx >= EVENT_IDX_OTHER_BASE {
            let index = i32::from(self.event_idx) - 3;
            let name = DEMO_NAME_TABLE
                .get(usize::try_from(index).unwrap_or(usize::MAX))
                .copied()
                .unwrap_or(DEMO_NAME_TABLE[0]);
            ctx.order_other_event(name, SCENE_EVENT_ORDER);
        }
    }

    pub fn check_order(&mut self, ctx: &mut Ctx) {
        if ctx.check_command_demo_accept() {
            self.event_set(EVENT_IDX_NONE);
            return;
        }

        if ctx.check_command_talk()
            && (self.event_idx == EVENT_IDX_TALK || self.event_idx == EVENT_IDX_TALK_ITEM)
        {
            self.event_set(EVENT_IDX_NONE);
            if ctx.chk_talk_xy() {
                self.field_0x8f5 = 1;
            } else {
                self.field_0x8f4 = 1;
            }
        }
    }

    pub fn cut_proc(&mut self, ctx: &mut Ctx) {
        let Some(staff_idx) = ctx.staff_of(ARC_NAME) else {
            return;
        };
        self.staff_idx = i32::try_from(staff_idx).unwrap_or(NO_STAFF);

        let Some((cut, advance)) = ctx.staff_cut() else {
            ctx.cut_end();
            return;
        };
        ctx.cut = cut;

        let act_idx = ctx
            .cut_name(cut)
            .and_then(cut_action_of)
            .unwrap_or(NO_ACTION);

        if act_idx == NO_ACTION {
            ctx.cut_end();
            return;
        }

        if advance {
            match act_idx {
                CUT_PRESENT => self.cut_present_start(ctx),
                CUT_SET_ANM => self.cut_set_anm_start(ctx),
                CUT_DISP_LETTER => self.cut_disp_letter_start(ctx),
                _ => {}
            }
        }

        match act_idx {
            CUT_PRESENT => self.cut_present_proc(ctx),
            CUT_SET_ANM => self.cut_set_anm_proc(ctx),
            CUT_DISP_LETTER => self.cut_disp_letter_proc(ctx),
            _ => {}
        }
    }

    pub fn cut_present_start(&mut self, _ctx: &mut Ctx) {}

    pub fn cut_present_proc(&mut self, ctx: &mut Ctx) {
        let item = self.letter_item(self.num_readable);
        let pos = ctx.pos;
        ctx.create_item_for_present_demo(pos, item);
        ctx.cut_end();
    }

    pub fn cut_set_anm_start(&mut self, ctx: &mut Ctx) {
        let name = ctx.cut_text(ctx.cut, CUT_KEY_NAME).unwrap_or("");
        if name == CUT_VALUE_HAKIDASU {
            self.set_anm(ANMPRM_POST_PUTOUT, false, ctx);
        } else {
            self.set_anm(ANMPRM_POST_GET0, false, ctx);
        }
    }

    pub fn cut_set_anm_proc(&mut self, ctx: &mut Ctx) {
        if self.morf_stopped {
            ctx.cut_end();
        }
    }

    pub fn cut_disp_letter_start(&mut self, _ctx: &mut Ctx) {
        self.curr_msg_no = self.letter_msg(self.num_readable);
    }

    pub fn cut_disp_letter_proc(&mut self, ctx: &mut Ctx) {
        let mode = self.talk(TALK_PARAM_NO_GET_MSG, ctx);
        if mode == TALK_MODE_LETTER_EYE {
            ctx.on_letter_read_eye_move();
        }
        if mode == STATUS_BOX_CLOSED {
            ctx.cut_end();
        }
    }

    pub fn talk(&mut self, param: u8, ctx: &mut Ctx) -> u8 {
        let mut status = NO_STATUS;

        if self.curr_msg_bs_pc_id == FPC_M_ERROR_PROCESS_ID {
            if param == TALK_PARAM_GET_MSG {
                self.curr_msg_no = self.get_msg(ctx);
            }

            let msg_no = self.curr_msg_no;
            let eye = self.eye_pos;
            self.curr_msg_bs_pc_id = ctx.message_set_actor(msg_no, eye);
            self.msg_bound = false;
        } else if self.msg_bound {
            status = ctx.status().unwrap_or(status);
            if status == STATUS_MSG_DISPLAYED {
                let mut msg_no = self.curr_msg_no;
                let select_num = ctx.select_num();
                let next = self.next_msg_status(ctx, &mut msg_no, select_num);
                self.curr_msg_no = msg_no;
                ctx.set_status(next);
                if next == STATUS_MSG_CONTINUES {
                    let msg_no = self.curr_msg_no;
                    ctx.message_set(msg_no, Vec3::ZERO);
                }
            } else if status == STATUS_BOX_CLOSED {
                ctx.set_status(STATUS_MSG_DESTROYED);
                self.curr_msg_bs_pc_id = FPC_M_ERROR_PROCESS_ID;
            }
        } else {
            self.msg_bound = ctx.search_by_id(self.curr_msg_bs_pc_id);
        }

        status
    }

    pub fn create_init(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        if ctx.collect.is_symbol(SYMBOL_FARORE) {
            letter_auto_stock(ctx.stories, LETTER_KOMALIS_FATHER);
        }

        if inventory::check_get_item(&*ctx.equipped, ctx.collect, ITEM_BOMB_BAG) {
            letter_auto_stock(ctx.stories, LETTER_BOMBS_AD);
        }

        if letter_is_delivery(&*ctx.stories, LETTER_BAITOS_MOM) && ctx.mem.is_stage_boss_enemy() {
            letter_auto_stock(ctx.stories, LETTER_BAITO);
        }

        if ctx.stories.test(EVENT_BIT_UNK_1E80) {
            letter_auto_stock(ctx.stories, LETTER_ORCA);
        }

        if ctx.wallet == 1 || ctx.wallet == 2 {
            letter_auto_stock(ctx.stories, LETTER_ROCK_SPIRE_SHOP_AD);
        }

        self.field_0x8f0 = MESSAGE_COUNT_BASE;
        self.curr_msg_bs_pc_id = FPC_M_ERROR_PROCESS_ID;
        self.msg_bound = false;

        self.attention_talk = ATTENTION_TALK;
        self.attention_speak = ATTENTION_SPEAK;

        self.set_anm(ANMPRM_POST_GET0, false, ctx);
        self.set_base_scale(Vec3::ONE);
        self.set_mtx(actor.pos);
        self.morf_play();

        self.set_collision(COLLISION_RADIUS, COLLISION_HEIGHT);
        self.cull_far = CULL_FAR;

        self.collider.set(Cyl::SRC);
        self.collider.set_stts();

        let room = actor.room;
        self.pay_type = self.check_send_price(ctx.stage_name, room);
        self.mode_proc_init(MODE_WAIT, ctx);

        self.wall_x = ACCH_CIR_WALL;
        self.wall_y = ACCH_CIR_WALL;
        self.speed = Vec3::ZERO;
        self.speed_f = 0.0;
        self.gravity = GRAVITY;
    }

    pub fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        if ctx.collect.is_symbol(SYMBOL_DIN) {
            self.num_readable = self.get_readable_letter_num(&*ctx.stories);
        } else {
            self.num_readable = 0;
        }

        self.check_order(ctx);
        self.set_attention(actor.pos, ctx.statics);
        self.set_collision(COLLISION_RADIUS, COLLISION_HEIGHT);
        self.mode_proc_exec(ctx);

        if ctx.event_run_check() && !ctx.talk_command {
            self.cut_proc(ctx);
        }
        self.event_order(ctx);

        self.pos_move_f(actor);
        self.crr_pos(actor, ctx);
        self.stts_move(actor);
        self.collider.centre = actor.pos;
        if self.collider.chk_tg_hit(ctx.player) {
            let eye = self.eye_pos;
            let room = actor.room;
            ctx.hit_se_start(eye, room, HIT_SE_KIND);
        }
        let pos = actor.pos;
        ctx.hit_eff_kikuzu(pos);
        ctx.roll_player_crash(pos, ROLL_PLAYER_CRASH_RADIUS, ROLL_PLAYER_CRASH_KIND);

        self.morf_play();
        self.set_mtx(actor.pos);
        self.set_anm(ANMPRM_NULL, false, ctx);
    }

    pub fn pos_move_f(&mut self, actor: &mut Actor) {
        let speed_f = self.speed_f;
        let angle = actor.angle[1];
        self.speed.x = speed_f * ssin(angle);
        self.speed.y += self.gravity;
        self.speed.z = speed_f * scos(angle);
        if self.speed.y < MAX_FALL_SPEED {
            self.speed.y = MAX_FALL_SPEED;
        }
        actor.pos += self.speed;
    }

    pub fn crr_pos(&mut self, actor: &mut Actor, ctx: &mut Ctx) {
        let ground = ctx.ground_at(actor.pos);
        let Some(ground) = ground else {
            return;
        };
        self.ground_h = ground;
        if actor.pos.y <= ground {
            actor.pos.y = ground;
            self.speed.y = 0.0;
        }
    }

    pub fn stts_move(&mut self, _actor: &mut Actor) {}

    pub fn morf_play(&mut self) {
        if self.morf_max == 0.0 {
            self.morf_stopped = true;
            return;
        }
        self.morf_cur += self.morf_speed;
        if self.morf_cur >= self.morf_max {
            if self.morf_loop {
                self.morf_cur %= self.morf_max;
                self.morf_stopped = false;
            } else {
                self.morf_cur = self.morf_max;
                self.morf_stopped = true;
            }
        } else {
            self.morf_stopped = false;
        }
        self.morf_frame = self.morf_cur;
    }
}

fn cut_action_of(name: &str) -> Option<i8> {
    CUT_NAMES
        .iter()
        .position(|cut| *cut == name)
        .and_then(|index| i8::try_from(index).ok())
}

fn bck_frame_max(index: i8) -> f32 {
    match index {
        BCK_POST_GET => BCK_FRAME_MAX_POST_GET,
        BCK_POST_PUTOUT => BCK_FRAME_MAX_POST_PUTOUT,
        BCK_POST_WAIT => BCK_FRAME_MAX_POST_WAIT,
        _ => 0.0,
    }
}

fn ssin(angle: i16) -> f32 {
    from_s16(angle).sin()
}

fn scos(angle: i16) -> f32 {
    from_s16(angle).cos()
}

fn calc_timer(timer: &mut i32) -> i32 {
    if *timer > 0 {
        *timer -= 1;
    }
    *timer
}

fn letter_send(stories: &mut StoryBits, index: u16) {
    stories.set_reg(index, LETTER_SEND);
}

fn letter_stock_set(stories: &mut StoryBits, index: u16) {
    stories.set_reg(index, LETTER_STOCK);
}

fn letter_is_no_send(stories: &StoryBits, index: u16) -> bool {
    stories.reg(index) == LETTER_NOSEND
}

fn letter_is_delivery(stories: &StoryBits, index: u16) -> bool {
    !letter_is_no_send(stories, index)
}

fn letter_auto_stock(stories: &mut StoryBits, index: u16) {
    if letter_is_no_send(stories, index) {
        letter_stock_set(stories, index);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Particle {
    pub id: u32,
    pub pos: Vec3,
    pub angle: [i16; 3],
    pub scale: Vec3,
    pub life: i32,
    pub colour: Option<[u8; 4]>,
}

pub struct Ctx<'a, 'w, 's> {
    pub entity: Entity,
    pub pos: Vec3,
    pub angle: [i16; 3],
    pub room: i8,
    pub player: Option<Vec3>,
    pub daynight: i32,
    pub wallet: u8,
    pub event_run: bool,
    pub talk_command: bool,
    pub demo_accept: bool,
    pub cut: usize,
    pub statics: &'a Statics,
    pub stage_name: &'a str,
    pub stories: &'a mut StoryBits,
    pub event_flags: &'a mut EventFlags,
    pub collect: &'a Collect,
    pub mem: &'a MemBits,
    pub equipped: &'a mut Equipped,
    pub rupees: &'a mut Rupees,
    pub counters: &'a mut Counters,
    pub events: &'a mut SimEvents,
    pub list: &'a EventList,
    pub active: &'a mut ActiveEvent,
    pub manager: &'a mut EventManager,
    pub item_partner: &'a mut ItemPartner,
    pub collision: Option<&'a Collision>,
    pub particles: &'a mut Vec<Particle>,
    pub hit_se: &'a mut Vec<(Vec3, i8, u8)>,
    pub kikuzu: &'a mut Vec<Vec3>,
    pub roll_crash: &'a mut Vec<(Vec3, f32, u8)>,
    pub letter_read_eye: bool,
    pub demo_mode: Vec<i32>,
    pub frame_rate: f32,
    pub commands: &'a mut Commands<'w, 's>,
    pub procs: &'a mut MsgProcs,
    pub info: &'a mut MsgGameInfo,
    pub one: &'a mut Msg1Box,
    pub two: &'a mut Msg2Box,
    pub archive: Option<&'a dyn MsgArchive>,
    pub clear_count: u8,
}

impl Ctx<'_, '_, '_> {
    fn rupees(&self) -> i32 {
        i32::try_from(self.rupees.count).unwrap_or(0)
    }

    fn add_rupees(&mut self, delta: i32) {
        self.counters.pay_rupee = delta;
        if delta < 0 {
            let paid = u32::try_from(-delta).unwrap_or(0);
            self.rupees.count = self.rupees.count.saturating_sub(paid);
        } else if let Ok(gain) = u32::try_from(delta) {
            self.rupees.count = self.rupees.count.saturating_add(gain);
        }
    }

    fn set_message_count_number(&mut self, count: i32) {
        self.counters.message_count = count;
    }

    fn chk_talk_xy(&self) -> bool {
        self.statics.talk_xy
    }

    fn pre_item_no(&self) -> u8 {
        self.statics.pre_item_no
    }

    fn set_reserve_item_empty(&mut self) {
        self.equipped.set(RESERVE_ITEM_BTN, inventory::NO_ITEM);
    }

    fn event_reset(&mut self) {}

    fn chk_present_end(&mut self) -> bool {
        self.manager.chk_present_end()
    }

    fn cancel_present(&mut self) {
        self.manager.cancel_present();
    }

    fn change_original_demo(&mut self) {
        self.demo_mode.push(-1);
    }

    fn change_demo_mode(&mut self, mode: i32) {
        self.demo_mode.push(mode);
    }

    fn base_anime_frame_rate(&self) -> f32 {
        self.frame_rate
    }

    fn on_letter_read_eye_move(&mut self) {
        self.letter_read_eye = true;
    }

    fn check_command_demo_accept(&self) -> bool {
        self.demo_accept
    }

    fn check_command_talk(&self) -> bool {
        self.talk_command
    }

    fn order_speak_event(&mut self) {
        self.order_event(ARC_NAME);
    }

    fn order_other_event(&mut self, name: &str, _hind: u16) {
        self.order_event(name);
    }

    fn order_event(&mut self, name: &str) {
        let Some(index) = self.list.event_index(name) else {
            return;
        };
        let actor = self.entity;
        self.active.order(self.list, index, actor);
    }

    fn staff_of(&self, name: &str) -> Option<usize> {
        self.list.staff_of(name)
    }

    fn staff_cut(&self) -> Option<(usize, bool)> {
        self.active.staff_cut(self.list, ARC_NAME)
    }

    fn cut_name(&self, cut: usize) -> Option<&str> {
        self.list.cut_name(cut)
    }

    fn cut_text(&self, cut: usize, key: &str) -> Option<&str> {
        self.list.cut_text(cut, key)
    }

    fn cut_end(&mut self) {
        self.active
            .end_staff_cut(self.list, self.event_flags, ARC_NAME);
    }

    fn event_run_check(&self) -> bool {
        self.event_run
    }

    fn end_check(&mut self, name: &str) -> bool {
        let Some(index) = self.list.event_index(name) else {
            return false;
        };
        self.active.end_check(index)
    }

    fn ground_at(&self, pos: Vec3) -> Option<f32> {
        self.collision
            .and_then(|collision| collision.ground_at(pos.x, pos.z))
    }

    fn hit_se_start(&mut self, eye: Vec3, room: i8, kind: u8) {
        self.hit_se.push((eye, room, kind));
    }

    fn hit_eff_kikuzu(&mut self, pos: Vec3) {
        self.kikuzu.push(pos);
    }

    fn roll_player_crash(&mut self, pos: Vec3, radius: f32, kind: u8) {
        self.roll_crash.push((pos, radius, kind));
    }

    fn proc_name(&self) -> Option<u16> {
        if self.procs.search_by_name(FPC_NM_MSG).is_some() {
            Some(FPC_NM_MSG)
        } else if self.procs.search_by_name(FPC_NM_MSG2).is_some() {
            Some(FPC_NM_MSG2)
        } else {
            None
        }
    }

    fn status(&self) -> Option<u8> {
        match self.proc_name() {
            Some(FPC_NM_MSG) => Some(self.one.status),
            Some(FPC_NM_MSG2) => Some(self.two.status),
            _ => None,
        }
    }

    fn set_status(&mut self, status: u8) {
        match self.proc_name() {
            Some(FPC_NM_MSG) => self.one.status = status,
            Some(FPC_NM_MSG2) => self.two.status = status,
            _ => {}
        }
    }

    fn select_num(&self) -> u8 {
        self.one.select_num
    }

    fn search_by_id(&mut self, id: u32) -> bool {
        self.procs.search_by_id(id).is_some()
    }

    fn message_set_actor(&mut self, msg_no: u32, eye: Vec3) -> u32 {
        let Some(archive) = self.archive else {
            return FPC_M_ERROR_PROCESS_ID;
        };
        let clear_count = self.clear_count;
        fop_msgm_message_set_actor(
            self.procs,
            self.info,
            archive,
            clear_count,
            msg_no,
            None,
            eye,
        )
    }

    fn message_set(&mut self, msg_no: u32, look_at: Vec3) -> u32 {
        let Some(archive) = self.archive else {
            return FPC_M_ERROR_PROCESS_ID;
        };
        let clear_count = self.clear_count;
        fop_msgm_message_set(
            self.procs,
            self.info,
            archive,
            clear_count,
            msg_no,
            None,
            look_at,
        )
    }

    fn create_item_for_present_demo(&mut self, pos: Vec3, item: u8) {
        if item == ITEM_NONE {
            return;
        }
        let entity = self
            .commands
            .spawn((
                Actor {
                    name: PRESENT_ITEM_ACTOR.to_owned(),
                    parameters: 0,
                    pos,
                    angle: [0; 3],
                    set_id: PRESENT_ITEM_SET_ID,
                    room: PRESENT_ITEM_ROOM,
                    ..Default::default()
                },
                PresentItem { item, hidden: false },
            ))
            .id();
        self.item_partner.entity = Some(entity);
        self.item_partner.item = item;
    }
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    statics: Res<'w, Statics>,
    list: Res<'w, EventList>,
    stage: Res<'w, StageName>,
    collision: Option<Res<'w, Collision>>,
    flags: Res<'w, TextFlags>,
    archives: Option<Res<'w, MsgArchives>>,
    players: Query<'w, 's, &'static Player, Without<Toripost>>,
}

#[derive(SystemParam)]
pub struct World<'w> {
    stories: ResMut<'w, StoryBits>,
    event_flags: ResMut<'w, EventFlags>,
    collect: Res<'w, Collect>,
    mem: Res<'w, MemBits>,
    equipped: ResMut<'w, Equipped>,
    rupees: ResMut<'w, Rupees>,
    counters: ResMut<'w, Counters>,
    events: ResMut<'w, SimEvents>,
    active: ResMut<'w, ActiveEvent>,
    manager: ResMut<'w, EventManager>,
    item_partner: ResMut<'w, ItemPartner>,
    procs: ResMut<'w, MsgProcs>,
    info: ResMut<'w, MsgGameInfo>,
    one: ResMut<'w, Msg1Box>,
    two: ResMut<'w, Msg2Box>,
}

pub fn control(
    watch: Watch<'_, '_>,
    mut world: World,
    mut commands: Commands,
    mut actors: Query<(Entity, &mut Actor, &mut Toripost), (With<Toripost>, Without<Player>)>,
) {
    let player = watch.players.iter().next().map(|link| link.pos);
    let clear_count = watch.flags.clear_count;
    let archive = watch
        .archives
        .as_deref()
        .map(|archives| archives as &dyn MsgArchive);
    let stage_name: &str = &watch.stage.0;
    let mut particles: Vec<Particle> = Vec::new();
    let mut hit_se: Vec<(Vec3, i8, u8)> = Vec::new();
    let mut kikuzu: Vec<Vec3> = Vec::new();
    let mut roll_crash: Vec<(Vec3, f32, u8)> = Vec::new();

    for (entity, mut actor, mut post) in &mut actors {
        let actor = &mut *actor;
        let post = &mut *post;
        let mut ctx = Ctx {
            entity,
            pos: actor.pos,
            angle: actor.angle,
            room: actor.room,
            player,
            daynight: 0,
            wallet: 0,
            event_run: world.info.event_run_check(),
            talk_command: false,
            demo_accept: false,
            cut: 0,
            statics: &watch.statics,
            stage_name,
            stories: &mut world.stories,
            event_flags: &mut world.event_flags,
            collect: &world.collect,
            mem: &world.mem,
            equipped: &mut world.equipped,
            rupees: &mut world.rupees,
            counters: &mut world.counters,
            events: &mut world.events,
            list: &watch.list,
            active: &mut world.active,
            manager: &mut world.manager,
            item_partner: &mut world.item_partner,
            collision: watch.collision.as_deref(),
            particles: &mut particles,
            hit_se: &mut hit_se,
            kikuzu: &mut kikuzu,
            roll_crash: &mut roll_crash,
            letter_read_eye: false,
            demo_mode: Vec::new(),
            frame_rate: 0.0,
            commands: &mut commands,
            procs: &mut world.procs,
            info: &mut world.info,
            one: &mut world.one,
            two: &mut world.two,
            archive,
            clear_count,
        };
        if !post.entered {
            post.entered = true;
            post.get_arg();
            post.create_init(actor, &mut ctx);
        }
        post.execute(actor, &mut ctx);
    }
}

const _: () = assert!(ANM_BCKS.len() == ANM_BCKS_ROWS);
const _: () = assert!(ANM_PRM.len() == ANM_PRM_ROWS);
const _: () = assert!(LETTERS.len() == LETTER_COUNT);
const _: () = assert!(PAY_TYPE.len() == ISLE_IDX_COUNT);
const _: () = assert!(CUT_NAMES.len() == 3);
const _: () = assert!(MODE_NULL == 5);
const _: () = assert!(PROC_INIT == 0 && PROC_EXEC == 1);
const _: () = assert!(ITEM_NONE == 0);
const _: () = assert!(SYMBOL_NAYRU == 0);
const _: () = assert!(DEMO_NAME_TABLE.len() == 1);
const _: () = assert!(PAY_MSG_NUM.len() == 3);
const _: () = assert!(SEND_PRICE.len() == 3);
