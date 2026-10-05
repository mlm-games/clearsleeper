use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList, PresentItem};
use super::flags::{EventFlags, StoryBits};
use super::intent::DoIntent;
use super::player::Player;
use super::stream;
use super::transition::NextStage;
use super::walk;
use crate::assets::MsgArchives;
use crate::text::driver::TextFlags;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgGameInfo, MsgProcs,
    fop_msgm_message_set_actor,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED,
    STATUS_MSG_ENDS, STATUS_TYPING,
};

pub const NPC_MT_ACTORS: [&str; 1] = ["Mt"];
pub const ARC_NAME: &str = "Niten";
pub const STAFF_NAME: &str = "Mt";
pub const EVENT_GET_ITEM: &str = "MT_GET_ITEM";
pub const PRESENT_ITEM_ACTOR: &str = "PresentItem";
pub const CUT_MES_SET: &str = "MES_SET";
pub const CUT_GET_ITEM: &str = "GET_ITEM";

pub const PRM_ARG_SHIFT: u32 = 0;
pub const PRM_ARG_MASK: u32 = 0xFF;
pub const PRM_NPC_MAX: i32 = 1;

pub const TOTAL_FIGURE_COUNT: u8 = 0x86;
pub const FIGURE_GROUPS: u8 = 0x11;
pub const FIGURE_COMP: [u16; 17] = [
    0x95FF, 0x94FF, 0x93FF, 0x92FF, 0x91FF, 0x90FF, 0x8FFF, 0x8EFF, 0x8DFF, 0x8CFF, 0xB1FF, 0x9CFF,
    0x84FF, 0x83FF, 0x82FF, 0x81FF, 0x80FF,
];
pub const FIGURE_ROOM_NAME: [&str; 8] = [
    "Pfigure",
    "figureA",
    "figureB",
    "figureC",
    "figureD",
    "figureE",
    "figureF",
    "figureG",
];
pub const FIGURE_ROOM_FIRST: i32 = 1;
pub const FIGURE_ROOM_LAST: i32 = 7;
pub const CHARA_DATA_SIZE: u8 = 0x86;
pub const PHOTO_INDEX_BASE: u8 = 0x49;

pub const BIT_2F01: u16 = 0x2F01;
pub const BIT_2F02: u16 = 0x2F02;
pub const BIT_3080: u16 = 0x3080;
pub const BIT_3401: u16 = 0x3401;
pub const BIT_3A01: u16 = 0x3A01;
pub const BIT_3D08: u16 = 0x3D08;
pub const BIT_3F01: u16 = 0x3F01;
pub const BIT_4040: u16 = 0x4040;
pub const BIT_4080: u16 = 0x4080;
pub const BIT_A9FF: u16 = 0xA9FF;
pub const TMP_0202: u32 = 0x0202;

pub const DSNAP_BIKUTSUKI: u8 = 0x02;
pub const DSNAP_UNK04: u8 = 0x04;
pub const DSNAP_UNK06: u8 = 0x06;
pub const DSNAP_UNK48: u8 = 0x48;
pub const DSNAP_NPC_PHOTO: u8 = 0x5E;
pub const DSNAP_NPC_RSH1: u8 = 0x5F;
pub const DSNAP_NPC_UW2: u8 = 0x67;
pub const DSNAP_NPC_UO2: u8 = 0x69;
pub const DSNAP_NPC_UB4: u8 = 0x6D;
pub const DSNAP_UNKCF: u8 = 0xCF;
pub const PHOTO_NONE: u8 = 0xFF;

pub const GX_TF_I4: u8 = 0x0;
pub const GX_TF_I8: u8 = 0x1;

pub const ITEM_PURPLE_RUPEE: u8 = 0x05;
pub const ITEM_HEART_PIECE: u8 = 0x07;
pub const ITEM_PICTO_BOX: u8 = 0x23;
pub const ITEM_DELUXE_PICTO_BOX: u8 = 0x26;
pub const ITEM_NONE: u8 = 0xFF;
pub const ITEM_BUTTON_COUNT: usize = 3;
pub const ITEM_KIND_HEART: u8 = 1;
pub const ITEM_KIND_RUPEE: u8 = 2;

pub const MSHAPE_TALK: u16 = 0x6E;
pub const MSHAPE_SPEAK: u16 = 0x6E;
pub const ATTN_LOCKON_TALK: u32 = 0x0000_0002;
pub const ATTN_ACTION_SPEAK: u32 = 0x0000_0008;
pub const ATTN_UNK1000000: u32 = 0x1000_0000;

pub const NPC_MAX_HEAD_X: i16 = 0x0FA0;
pub const NPC_MAX_HEAD_Y: i16 = 0x1B58;
pub const NPC_MAX_BACKBONE_X: i16 = 0;
pub const NPC_MAX_BACKBONE_Y: i16 = 0;
pub const NPC_MIN_HEAD_X: i16 = -0x0BB8;
pub const NPC_MIN_HEAD_Y: i16 = -0x1B58;
pub const NPC_MIN_BACKBONE_X: i16 = 0;
pub const NPC_MIN_BACKBONE_Y: i16 = 0;
pub const NPC_MAX_TURN_STEP: i16 = 0x01F4;
pub const NPC_DAT_0X12: i16 = 0;
pub const NPC_EYE_OFFSET_Y: f32 = 190.0;
pub const NPC_ATTN_POS_Y: f32 = 150.0;
pub const NPC_EYE_POS_Y: f32 = 250.0;
pub const NPC_ATTN_DIST: f32 = 400.0;
pub const NPC_UNK24: i16 = 0x4000;
pub const NPC_ATTN_ANGLE: i32 = 0x0800;
pub const NPC_HEAD_TARGET: i16 = 0x0190;
pub const NPC_UNK2C: i16 = 0;
pub const NPC_CYL_RADIUS: f32 = 60.0;
pub const NPC_UNK34: f32 = 0.5;
pub const NPC_UNK38: f32 = 3.0;
pub const NPC_UNK3C: i16 = 0x0006;
pub const NPC_UNK3E: i16 = 0x0400;
pub const NPC_UNK40: i16 = 0x003C;
pub const NPC_UNK42: i16 = 0x005A;
pub const NPC_UNK44: i16 = 0x00C8;
pub const NPC_UNK46: i16 = 0x012C;
pub const NPC_TURN_HOLD: i16 = 0x001E;
pub const NPC_NO_BODY: u8 = 0;
pub const NPC_LOOK_BACK: u8 = 1;
pub const CYL_HEIGHT: f32 = 150.0;
pub const CULL_MIN: [f32; 3] = [-70.0, 0.0, -70.0];
pub const CULL_MAX: [f32; 3] = [70.0, 200.0, 70.0];
pub const GRAVITY: f32 = -9.0;
pub const HEAP_SIZE: u32 = 0x2900;

pub const ANM_WAIT: usize = 0;
pub const ANM_TALK: usize = 1;
pub const ANM_TALK2: usize = 2;
pub const ANM_TALK3: usize = 3;
pub const ANM_TALK4: usize = 4;
pub const ANM_TALK5: usize = 5;
pub const ANM_TUKURU: usize = 6;
pub const ANM_TABLE_COUNT: usize = 7;
pub const ANM_END: u8 = 0xFF;
pub const ANM_LOOP_MODE: i32 = 2;
pub const ANM_ONCE_MODE: i32 = 0;

pub const BCK_WAIT01: u8 = 0;
pub const BCK_TALK01: u8 = 1;
pub const BCK_TALK02: u8 = 2;
pub const BCK_TALK03: u8 = 3;
pub const BCK_TALK04: u8 = 4;
pub const BCK_TUKURU: u8 = 5;
pub const BCK_COUNT: usize = 6;
pub const BCK_FRAME_MAX: [i16; BCK_COUNT] = [40, 50, 30, 60, 30, 24];
pub const BTP_FRAME_MAX: u8 = 4;
pub const BTP_HOLD: i16 = 0x78;

pub const MODE_WAIT: u8 = 0;
pub const MODE_TALK: u8 = 1;
pub const STEP_WAIT: u8 = 0;
pub const STEP_TALK: u8 = 1;

pub const STATE_NONE: u8 = 0;
pub const STATE_ONE: u8 = 1;
pub const STATE_TWO: u8 = 2;
pub const STATE_THREE: u8 = 3;

pub const MSG_STATUS_NONE: u8 = 0xFF;
pub const MSG_ACTOR_SLOT: u32 = 0;
pub const NO_INDEX: i16 = -1;
pub const NO_STAFF: usize = usize::MAX;
pub const NO_TABLE: u8 = 0xFF;
pub const NO_ROW: usize = 0;
pub const COMP_TOTAL: i32 = 0x84;
pub const COMP_TOTAL_UNLIT: i32 = 0x85;
pub const COMP_FIGURE_UNLIT: u8 = 0x32;
pub const COMP_PHASE_FIGURE: u8 = 0x40;
pub const EV_FLAG_8: u8 = 8;
pub const EV_FLAG_ALL: u8 = 0xFF;
pub const TURN_SPEED: i16 = 4;
pub const TURN_MAX_STEP: i16 = 0x800;
pub const TURN_EPS: f32 = 0.001;
pub const ATTN_BONUS_DIST: f32 = 40.0;
pub const ATTN_BONUS_ANGLE: i32 = 0x71C;
pub const PLAYER_HEAD_TOP_Y: f32 = 0.0;
pub const EMODE_NONE: u8 = 0;
pub const EMODE_LOOP: u8 = 2;
pub const STATE_STOP: u8 = 1;
pub const STATE_LOOPED: u8 = 2;
pub const PLAY_SPEED: f32 = 1.0;
pub const PLAY_START: f32 = 0.0;
pub const PLAY_END: f32 = -1.0;
pub const ANM_FIRST: i16 = 0;
pub const NO_SPEED: f32 = -1.0;

pub const TBL_1ST_TALK: u8 = 0;
pub const TBL_2ND_TALK: u8 = 1;
pub const TBL_3RD_TALK: u8 = 2;
pub const TBL_4TH_TALK: u8 = 3;
pub const TBL_MAKING: u8 = 4;
pub const TBL_1ST_KANSEI: u8 = 5;
pub const TBL_2ND_KANSEI: u8 = 6;
pub const TBL_ROOM: u8 = 7;
pub const TBL_GALLERY: u8 = 8;
pub const TBL_ROOM_IN: u8 = 9;
pub const TBL_KANSEI_END: u8 = 10;
pub const TBL_NO_COLOR: u8 = 11;
pub const TBL_NO_FIGURE: u8 = 12;
pub const TBL_MAKED: u8 = 13;
pub const TBL_FACE_NG: u8 = 14;
pub const TBL_BODY_NG: u8 = 15;
pub const TBL_MAKE_1ST: u8 = 16;
pub const TBL_MAKE_2ND: u8 = 17;
pub const TBL_MAKE_1ST_MT: u8 = 18;
pub const TBL_MAKE_2ND_MT: u8 = 19;
pub const TBL_MAKE_1ST_MN: u8 = 20;
pub const TBL_MAKE_2ND_MN: u8 = 21;

pub const MSG_1ST_TALK: [u32; 8] = [
    0x365E, 0x365F, 0x3660, 0x3661, 0x3662, 0x3663, 0x3664, 0x0000,
];
pub const MSG_2ND_TALK: [u32; 4] = [0x3665, 0x3666, 0x3667, 0x0000];
pub const MSG_3RD_TALK: [u32; 2] = [0x3668, 0x0000];
pub const MSG_4TH_TALK: [u32; 2] = [0x3669, 0x0000];
pub const MSG_MAKING: [u32; 2] = [0x3654, 0x0000];
pub const MSG_1ST_KANSEI: [u32; 5] = [0x364D, 0x364E, 0x3650, 0x0001, 0x0000];
pub const MSG_2ND_KANSEI: [u32; 5] = [0x364D, 0x364F, 0x3650, 0x0001, 0x0000];
pub const MSG_ROOM: [u32; 8] = [
    0x366B, 0x366B, 0x366C, 0x366D, 0x366E, 0x366F, 0x3670, 0x3671,
];
pub const MSG_GALLERY: [u32; 2] = [0x3651, 0x0000];
pub const MSG_ROOM_IN: [u32; 2] = [0x3652, 0x0000];
pub const MSG_KANSEI_END: [u32; 2] = [0x3653, 0x0000];
pub const MSG_NO_COLOR: [u32; 2] = [0x3655, 0x0000];
pub const MSG_NO_FIGURE: [u32; 2] = [0x3656, 0x0000];
pub const MSG_MAKED: [u32; 2] = [0x3658, 0x0000];
pub const MSG_FACE_NG: [u32; 2] = [0x3659, 0x0000];
pub const MSG_BODY_NG: [u32; 2] = [0x365A, 0x0000];
pub const MSG_MAKE_1ST: [u32; 4] = [0x365B, 0x3674, 0x365C, 0x0002];
pub const MSG_MAKE_2ND: [u32; 4] = [0x365B, 0x3674, 0x365D, 0x0002];
pub const MSG_MAKE_1ST_MT: [u32; 4] = [0x3672, 0x3674, 0x365C, 0x0002];
pub const MSG_MAKE_2ND_MT: [u32; 4] = [0x3672, 0x3674, 0x365D, 0x0002];
pub const MSG_MAKE_1ST_MN: [u32; 4] = [0x3673, 0x3674, 0x365C, 0x0002];
pub const MSG_MAKE_2ND_MN: [u32; 4] = [0x3673, 0x3674, 0x365D, 0x0002];

pub const MSG_TABLES: [&[u32]; 22] = [
    &MSG_1ST_TALK,
    &MSG_2ND_TALK,
    &MSG_3RD_TALK,
    &MSG_4TH_TALK,
    &MSG_MAKING,
    &MSG_1ST_KANSEI,
    &MSG_2ND_KANSEI,
    &MSG_ROOM,
    &MSG_GALLERY,
    &MSG_ROOM_IN,
    &MSG_KANSEI_END,
    &MSG_NO_COLOR,
    &MSG_NO_FIGURE,
    &MSG_MAKED,
    &MSG_FACE_NG,
    &MSG_BODY_NG,
    &MSG_MAKE_1ST,
    &MSG_MAKE_2ND,
    &MSG_MAKE_1ST_MT,
    &MSG_MAKE_2ND_MT,
    &MSG_MAKE_1ST_MN,
    &MSG_MAKE_2ND_MN,
];

pub const MSG_TAG_ROOM: u32 = 0x0001;
pub const MSG_TAG_END: u32 = 0x0002;
pub const MSG_TAG_STOP: u32 = 0x0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnmDat {
    pub bck: u8,
    pub morf: u8,
    pub next: u8,
}

pub const ANM_TBL_WAIT: [AnmDat; 1] = [AnmDat {
    bck: BCK_WAIT01,
    morf: 0x08,
    next: ANM_END,
}];
pub const ANM_TBL_TALK: [AnmDat; 1] = [AnmDat {
    bck: BCK_TALK01,
    morf: 0x08,
    next: ANM_END,
}];
pub const ANM_TBL_TALK2: [AnmDat; 1] = [AnmDat {
    bck: BCK_TALK02,
    morf: 0x08,
    next: ANM_END,
}];
pub const ANM_TBL_TALK3: [AnmDat; 1] = [AnmDat {
    bck: BCK_TALK03,
    morf: 0x08,
    next: ANM_END,
}];
pub const ANM_TBL_TALK4: [AnmDat; 1] = [AnmDat {
    bck: BCK_TALK04,
    morf: 0x08,
    next: ANM_END,
}];
pub const ANM_TBL_TALK5: [AnmDat; 2] = [
    AnmDat {
        bck: BCK_TALK04,
        morf: 0x08,
        next: 0x02,
    },
    AnmDat {
        bck: BCK_TALK01,
        morf: 0x08,
        next: ANM_END,
    },
];
pub const ANM_TBL_TUKURU: [AnmDat; 2] = [
    AnmDat {
        bck: BCK_TUKURU,
        morf: 0x08,
        next: 0x01,
    },
    AnmDat {
        bck: BCK_WAIT01,
        morf: 0x08,
        next: ANM_END,
    },
];
pub const ANM_TABLES: [&[AnmDat]; ANM_TABLE_COUNT] = [
    &ANM_TBL_WAIT,
    &ANM_TBL_TALK,
    &ANM_TBL_TALK2,
    &ANM_TBL_TALK3,
    &ANM_TBL_TALK4,
    &ANM_TBL_TALK5,
    &ANM_TBL_TUKURU,
];

pub const CHARA_FIG_ROOM: [u8; CHARA_DATA_SIZE as usize] = [
    0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x02, 0x02,
    0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
    0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
    0x01, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06,
    0x06, 0x06, 0x06, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07,
    0x07, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05,
    0x05, 0x05, 0x05, 0x05, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04,
    0x04, 0x04, 0x04, 0x04, 0x04, 0x04
];

pub fn is_npc_mt(name: &str) -> bool {
    NPC_MT_ACTORS.contains(&name)
}

pub fn anm_table(id: u8, row: usize) -> Option<AnmDat> {
    let table = ANM_TABLES.get(usize::from(id)).copied()?;
    table.get(row).copied()
}

pub fn msg_table_at(id: Option<u8>, row: usize) -> u32 {
    let Some(id) = id else {
        return MSG_TAG_STOP;
    };
    let table = MSG_TABLES.get(usize::from(id)).copied().unwrap_or(&[]);
    table.get(row).copied().unwrap_or(MSG_TAG_STOP)
}

pub fn msg_table_first(id: Option<u8>) -> u32 {
    msg_table_at(id, NO_ROW)
}

pub fn photo_index_to_table_index(photo_index: u8) -> u8 {
    photo_index.wrapping_sub(PHOTO_INDEX_BASE)
}

pub fn fig_room_id(table_index: u8) -> u8 {
    if table_index >= CHARA_DATA_SIZE {
        return PHOTO_NONE;
    }
    CHARA_FIG_ROOM.get(usize::from(table_index)).copied().unwrap_or(0)
}

pub fn is_figure_get(stories: &StoryBits, figure_no: u8) -> u8 {
    if figure_no >= TOTAL_FIGURE_COUNT {
        return 0;
    }
    if figure_no / 8 < FIGURE_GROUPS {
        let bit = figure_no % 8;
        let reg = stories.reg(FIGURE_COMP.get(usize::from(figure_no / 8)).copied().unwrap_or(0));
        return u8::from(reg & (1 << bit) != 0);
    }
    0
}

pub fn set_figure(stories: &mut StoryBits, figure: u8) {
    if figure < TOTAL_FIGURE_COUNT {
        let bit = figure % 8;
        let id = FIGURE_COMP.get(usize::from(figure / 8)).copied().unwrap_or(0);
        let mut reg = stories.reg(id);
        reg |= 1 << bit;
        stories.set_reg(id, reg);
        stories.set(BIT_3A01);
    }
}

pub fn get_figure_make_num(stories: &StoryBits) -> i32 {
    let mut num = 0;
    for figure in 0..TOTAL_FIGURE_COUNT {
        if is_figure_get(stories, figure) != 0 {
            num += 1;
        }
    }
    num
}

pub fn is_comp(stories: &StoryBits) -> bool {
    let owned = get_figure_make_num(stories);
    let mut total = COMP_TOTAL;
    if is_figure_get(stories, COMP_FIGURE_UNLIT) != 0 {
        total = COMP_TOTAL_UNLIT;
    }
    owned >= total
}

pub fn picture_num(stories: &StoryBits) -> u8 {
    u8::try_from(get_figure_make_num(stories)).unwrap_or(u8::MAX)
}

pub fn change_photo_no(photo_no: u8) -> u8 {
    match photo_no {
        DSNAP_BIKUTSUKI => DSNAP_NPC_UO2,
        DSNAP_UNK04 => DSNAP_NPC_UW2,
        DSNAP_UNK06 => DSNAP_NPC_UB4,
        other => other,
    }
}

pub fn get_prm_npc_no(argument: i8) -> u8 {
    if (0..PRM_NPC_MAX).contains(&i32::from(argument)) {
        argument as u8
    } else {
        0
    }
}

pub fn clear_kansei(stories: &mut StoryBits) {
    stories.clear(BIT_3080);
    stories.clear(BIT_2F01);
    stories.clear(BIT_4080);
    stories.clear(BIT_3F01);
    stories.clear(BIT_4040);
}

pub fn calc_timer(value: &mut i16) -> bool {
    if *value != 0 {
        *value -= 1;
    }
    *value == 0
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
    pub fn idle() -> Self {
        Self {
            attribute: EMODE_LOOP,
            state: 0,
            start: 0,
            end: 0,
            loop_point: 0,
            rate: PLAY_SPEED,
            frame: 0.0,
            cur_morf: PLAY_SPEED,
            prev_morf: PLAY_SPEED,
            morf_step: 0.0,
        }
    }

    pub fn init(&mut self, end: i16) {
        self.attribute = EMODE_NONE;
        self.state = 0;
        self.start = 0;
        self.end = end;
        self.loop_point = 0;
        self.rate = 0.0;
        self.frame = 0.0;
    }

    pub fn set_anm(&mut self, attribute: u8, end: i16, loop_point: i16) {
        self.attribute = attribute;
        self.state = 0;
        self.start = ANM_FIRST;
        self.frame = 0.0;
        self.end = end;
        self.loop_point = loop_point;
        self.rate = PLAY_SPEED;
    }

    pub fn set_morf(&mut self, morf: f32) {
        if self.prev_morf < 0.0 || morf <= 0.0 {
            self.cur_morf = PLAY_SPEED;
        } else {
            self.cur_morf = 0.0;
            self.morf_step = PLAY_SPEED / morf;
        }
        self.prev_morf = self.cur_morf;
    }

    pub fn update(&mut self) {
        self.state = 0;
        self.frame += self.rate;
        let start = f32::from(self.start);
        let end = f32::from(self.end);
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
                    self.state |= STATE_LOOPED;
                    if low <= 0 {
                        break;
                    }
                    self.frame += low as f32;
                }
                while self.frame >= end {
                    self.state |= STATE_LOOPED;
                    if high <= 0 {
                        break;
                    }
                    self.frame -= high as f32;
                }
            }
            _ => {}
        }
    }

    pub fn play(&mut self) -> bool {
        if self.cur_morf < PLAY_SPEED {
            self.prev_morf = self.cur_morf;
            walk::chase_f(&mut self.cur_morf, PLAY_SPEED, self.morf_step);
        }
        self.update();
        self.is_stop()
    }

    pub fn is_stop(&self) -> bool {
        self.state & STATE_STOP != 0 || self.rate == 0.0
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct NpcMt {
    pub entered: bool,
    pub done: bool,
    pub npc_no: u8,
    pub unk_6f0: u32,
    pub look_at_pos: Vec3,
    pub anm_tbl: Option<u8>,
    pub anm_row: usize,
    pub msg_tbl: Option<u8>,
    pub msg_row: usize,
    pub anim_718: f32,
    pub anim_71c: f32,
    pub anim_720: f32,
    pub item_no: i32,
    pub head_only: bool,
    pub get_item_event_idx: i16,
    pub wait_timer: i16,
    pub attn_angle: i32,
    pub home_angle: i16,
    pub msg_last_status: u16,
    pub timer: i16,
    pub head_target: i16,
    pub turn_vel: i16,
    pub trn_angle: i16,
    pub mode: u8,
    pub attn_seen: u8,
    pub state: u8,
    pub frame: u8,
    pub step: u8,
    pub res_flag: u8,
    pub item_kind: u8,
    pub bck_index: u8,
    pub anm_flag: u8,
    pub anm_count: i8,
    pub cut_index: i8,
    pub look_mode: i8,
    pub no_body: u8,
    pub look_back: u8,
    pub far_seen: u8,
    pub attn_forced: bool,
    pub staff_idx: usize,
    pub cut_attn: bool,
    pub cut_attn_pos: Vec3,
    pub cut_turn_speed: i16,
    pub cut_proc: bool,
    pub cut_seen: i32,
    pub ev_flag: u8,
    pub curr_msg_no: u32,
    pub msg_id: u32,
    pub msg_live: bool,
    pub morph: Frame,
    pub speed_f: f32,
    pub eye: Vec3,
    pub shape_angle: [i16; 3],
    pub jnt_angle: [[i16; 2]; 2],
    pub jnt_min: [[i16; 2]; 2],
    pub jnt_max: [[i16; 2]; 2],
    pub jnt_step: [[i16; 2]; 2],
    pub jnt_trn: bool,
    pub start_stage_point: bool,
    pub picture_mode: bool,
    pub picture_result: u8,
    pub picture_detail: u8,
    pub picture_format: u8,
    pub picture_num: u8,
    pub select_item: [u8; ITEM_BUTTON_COUNT],
    pub speak_order: bool,
    pub torn_down: bool,
    pub talk_command: bool,
    pub demo_accept: bool,
    pub talk_xy: bool,
    pub event_mode: bool,
    pub erase_picture: bool,
    pub reset_event: bool,
    pub give_item: Option<(Vec3, u8)>,
}

impl NpcMt {
    pub fn spawn(parameters: u32) -> Self {
        let argument = ((parameters >> PRM_ARG_SHIFT) & PRM_ARG_MASK) as u8 as i8;
        let mut morph = Frame::idle();
        morph.init(BCK_FRAME_MAX[BCK_WAIT01 as usize]);
        morph.attribute = EMODE_LOOP;
        morph.rate = PLAY_SPEED;
        Self {
            entered: false,
            done: false,
            npc_no: get_prm_npc_no(argument),
            unk_6f0: 0,
            look_at_pos: Vec3::ZERO,
            anm_tbl: None,
            anm_row: NO_ROW,
            msg_tbl: None,
            msg_row: NO_ROW,
            anim_718: 0.0,
            anim_71c: NO_SPEED,
            anim_720: NPC_ATTN_DIST,
            item_no: 0,
            head_only: true,
            get_item_event_idx: NO_INDEX,
            wait_timer: 0,
            attn_angle: NPC_ATTN_ANGLE,
            home_angle: 0,
            msg_last_status: u16::MAX,
            timer: 0,
            head_target: NPC_HEAD_TARGET,
            turn_vel: 0,
            trn_angle: 0,
            mode: MODE_WAIT,
            attn_seen: 0,
            state: STATE_NONE,
            frame: 0,
            step: STEP_WAIT,
            res_flag: 0,
            item_kind: 0,
            bck_index: BCK_WAIT01,
            anm_flag: 0,
            anm_count: 0,
            cut_index: -1,
            look_mode: 0,
            no_body: NPC_NO_BODY,
            look_back: NPC_LOOK_BACK,
            far_seen: 0,
            attn_forced: false,
            staff_idx: NO_STAFF,
            cut_attn: false,
            cut_attn_pos: Vec3::ZERO,
            cut_turn_speed: 0,
            cut_proc: false,
            cut_seen: -1,
            ev_flag: 0,
            curr_msg_no: 0,
            msg_id: FPC_M_ERROR_PROCESS_ID,
            msg_live: false,
            morph,
            speed_f: 0.0,
            eye: Vec3::ZERO,
            shape_angle: [0; 3],
            jnt_angle: [[0; 2]; 2],
            jnt_min: [
                [NPC_MIN_HEAD_X, NPC_MIN_HEAD_Y],
                [NPC_MIN_BACKBONE_X, NPC_MIN_BACKBONE_Y],
            ],
            jnt_max: [
                [NPC_MAX_HEAD_X, NPC_MAX_HEAD_Y],
                [NPC_MAX_BACKBONE_X, NPC_MAX_BACKBONE_Y],
            ],
            jnt_step: [[NPC_MAX_TURN_STEP; 2]; 2],
            jnt_trn: false,
            start_stage_point: false,
            picture_mode: false,
            picture_result: 0,
            picture_detail: 0,
            picture_format: 0,
            picture_num: 0,
            select_item: [ITEM_NONE; ITEM_BUTTON_COUNT],
            speak_order: false,
            torn_down: false,
            talk_command: false,
            demo_accept: false,
            talk_xy: false,
            event_mode: false,
            erase_picture: false,
            reset_event: false,
            give_item: None,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        !self.done
    }

    pub fn attn_on(&mut self) {
        self.attn_forced = true;
    }

    pub fn attn_off(&mut self) {
        self.attn_forced = false;
    }

    pub fn set_res_flag(&mut self, flag: u8) {
        self.res_flag = flag;
    }

    pub fn jnt_set_param(&mut self) {
        self.jnt_max[1][0] = NPC_MAX_BACKBONE_X;
        self.jnt_max[1][1] = NPC_MAX_BACKBONE_Y;
        self.jnt_min[1][0] = NPC_MIN_BACKBONE_X;
        self.jnt_min[1][1] = NPC_MIN_BACKBONE_Y;
        self.jnt_max[0][0] = NPC_MAX_HEAD_X;
        self.jnt_max[0][1] = NPC_MAX_HEAD_Y;
        self.jnt_min[0][0] = NPC_MIN_HEAD_X;
        self.jnt_min[0][1] = NPC_MIN_HEAD_Y;
        for row in self.jnt_step.iter_mut() {
            row[0] = NPC_MAX_TURN_STEP;
            row[1] = NPC_MAX_TURN_STEP;
        }
    }

    pub fn jnt_set_trn(&mut self) {
        self.jnt_trn = true;
    }

    pub fn jnt_trn_chk(&self) -> bool {
        self.jnt_trn
    }

    fn jnt_limit(&self, target: i16, slot: usize, axis: usize) -> i16 {
        let max = self.jnt_max.get(slot).and_then(|row| row.get(axis)).copied();
        let min = self.jnt_min.get(slot).and_then(|row| row.get(axis)).copied();
        match (max, min) {
            (Some(max), Some(min)) => {
                if target > max {
                    max
                } else if target < min {
                    min
                } else {
                    target
                }
            }
            _ => 0,
        }
    }

    fn jnt_angle_calc(&mut self, out: &mut i16, target: i16, speed: i16, max_vel: i16) {
        let orig = out.wrapping_sub(target);
        let diff = walk::add_calc_angle_s(orig, 0, speed, max_vel, 0x60);
        if i32::from(diff.unsigned_abs()) > i32::from(speed) {
            *out = out.wrapping_add(diff.wrapping_sub(orig));
        } else {
            *out = target;
        }
    }

    fn jnt_follow(&mut self, out: &mut i16, target: i16, max_vel: i16) {
        let orig = *out;
        self.jnt_angle_calc(out, target, TURN_SPEED, max_vel);
        let delta = out.wrapping_sub(orig);
        let want = target.wrapping_sub(*out);
        let mut sum = 0_i16;
        let mut total = 0_i16;
        for slot in (0..2).rev() {
            let angle = self.jnt_angle[slot][1];
            sum = sum.wrapping_add(angle);
            if (delta > 0 && sum > want) || (delta < 0 && sum < want) {
                let value = self.jnt_limit(want.wrapping_sub(total), slot, 1);
                self.jnt_angle[slot][1] = value;
            }
            total = total.wrapping_add(self.jnt_angle[slot][1]);
        }
    }

    fn jnt_move(&mut self, param_1: i16, axis: usize) -> bool {
        let mut rest = param_1;
        let mut angles = [0_i16; 2];
        for slot in 0..2 {
            let mut angle = rest;
            angle = self.jnt_limit(angle, slot, axis);
            rest -= angle;
            angles[slot] = angle;
            let current = self.jnt_angle[slot][axis];
            let max_step = self.jnt_step[slot][axis];
            self.jnt_angle[slot][axis] =
                walk::add_calc_angle_s(current, angle, TURN_SPEED, max_step, 1);
        }
        let mut done = 0;
        for slot in 0..2 {
            let max_step = self.jnt_step[slot][axis];
            if angles[slot].wrapping_sub(self.jnt_angle[slot][axis]).unsigned_abs()
                <= (max_step / 2) as u16
            {
                done += 1;
            }
        }
        done == 2
    }

    fn jnt_look_at(
        &mut self,
        out: &mut i16,
        dst: Option<Vec3>,
        src: Vec3,
        default_y: i16,
        max_vel: i16,
        hold: bool,
    ) {
        let (target_y, target_x) = match dst {
            Some(point) => {
                let delta = point - src;
                let dist_xz = (delta.x * delta.x + delta.z * delta.z).sqrt();
                (
                    walk::to_s16(delta.x.atan2(delta.z)),
                    walk::to_s16(delta.y.atan2(dist_xz)),
                )
            }
            None => (default_y, 0),
        };
        let delta_y = target_y.wrapping_sub(*out);
        let max_y = self.jnt_max[1][1].wrapping_add(self.jnt_max[0][1]);
        let min_y = self.jnt_min[1][1].wrapping_add(self.jnt_min[0][1]);
        let done = self.jnt_move(delta_y, 1);
        if (self.jnt_trn || (delta_y >= max_y || delta_y <= min_y) && done) && !hold {
            self.jnt_follow(out, target_y, max_vel);
        }
        self.jnt_move(target_x, 0);
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
            Some(MSG_ACTOR_SLOT),
            eye,
        )
    }

    pub fn message_set(&mut self, msg_no: u32) -> u32 {
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
            Vec3::ZERO,
        )
    }

    pub fn live(&self, id: u32) -> bool {
        self.procs.search_by_id(id).is_some()
    }
}

pub struct Ctx<'a> {
    pub entity: Entity,
    pub pos: Vec3,
    pub player: Option<Vec3>,
    pub stories: &'a mut StoryBits,
    pub tmp: &'a mut EventFlags,
    pub list: &'a EventList,
    pub active: &'a mut ActiveEvent,
    pub next_stage: Option<&'a str>,
    pub text: &'a mut Text<'a>,
    pub event_mode: bool,
    pub demo_accept: bool,
    pub talk_command: bool,
    pub talk_xy: bool,
    pub cut_proc: bool,
    pub speak_order: bool,
    pub photo_mode: bool,
    pub picture_result: u8,
    pub picture_detail: u8,
    pub picture_format: u8,
    pub picture_num: u8,
    pub select_item: [u8; ITEM_BUTTON_COUNT],
    pub erase_picture: bool,
    pub reset_event: bool,
    pub give_item: Option<(Vec3, u8)>,
}

impl Ctx<'_> {
    pub fn running(&self) -> bool {
        self.event_mode
    }

    pub fn talk_xy(&self) -> bool {
        self.talk_xy
    }
}

impl NpcMt {
    pub fn phase_1(&mut self, stories: &mut StoryBits, tmp: &mut EventFlags, start_point: bool) -> bool {
        let mut stop = false;
        if stories.test(BIT_3D08) {
            stop = true;
        } else if !start_point {
            if is_comp(stories) {
                stop = true;
                stories.set(BIT_3D08);
            } else {
                tmp.clear(TMP_0202);
                if stories.test(BIT_4080) {
                    clear_kansei(stories);
                }
            }
        }
        if stop {
            set_figure(stories, COMP_PHASE_FIGURE);
            true
        } else {
            self.set_res_flag(1);
            false
        }
    }

    pub fn create_init(&mut self, actor: &Actor, list: &EventList) {
        self.anim_718 = 0.0;
        self.set_anm_tbl(Some(ANM_WAIT as u8), NO_ROW);
        self.get_item_event_idx = list
            .event_index(EVENT_GET_ITEM)
            .and_then(|index| i16::try_from(index).ok())
            .unwrap_or(NO_INDEX);
        self.turn_vel = 0;
        self.mode = MODE_WAIT;
        self.attn_seen = 0;
        self.far_seen = 0;
        self.attn_off();
        self.home_angle = actor.angle[1];
        self.jnt_set_param();
        self.no_body = NPC_NO_BODY;
        self.look_back = NPC_LOOK_BACK;
        self.anim_720 = NPC_ATTN_DIST;
        self.attn_angle = NPC_ATTN_ANGLE;
        self.eye = Vec3::new(actor.pos.x, actor.pos.y + NPC_EYE_POS_Y, actor.pos.z);
        self.shape_angle = actor.angle;
    }

    pub fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.chk_attention(actor, ctx);
        self.check_order(ctx);
        if !ctx.running() || ctx.talk_command {
            match self.step {
                STEP_TALK => self.execute_talk(ctx),
                _ => self.execute_wait(),
            }
        } else {
            self.event_move(ctx);
        }
        self.event_order(ctx);
        self.play_tex_pattern_anm();
        self.play_anm();
        self.eye = Vec3::new(actor.pos.x, actor.pos.y + NPC_EYE_POS_Y, actor.pos.z);
        self.look_back(actor, ctx);
    }

    pub fn execute_common(&mut self) -> u8 {
        if self.attn_seen != 0 {
            self.state = STATE_ONE;
        } else {
            self.state = STATE_NONE;
        }
        if self.mode == MODE_TALK && self.step != STEP_TALK {
            self.execute_set_mode(MODE_TALK);
        }
        self.mode
    }

    pub fn execute_set_mode(&mut self, param_1: u8) {
        self.anim_718 = 0.0;
        self.step = match param_1 {
            MODE_TALK => {
                self.execute_talk_init();
                STEP_TALK
            }
            _ => {
                self.execute_wait_init();
                STEP_WAIT
            }
        };
    }

    pub fn execute_wait_init(&mut self) {
        self.speed_f = 0.0;
        self.set_anm_tbl(Some(ANM_WAIT as u8), NO_ROW);
        self.jnt_set_param();
    }

    pub fn execute_wait(&mut self) {
        self.execute_common();
        if self.bck_index != BCK_WAIT01 {
            self.set_anm_tbl(Some(ANM_WAIT as u8), NO_ROW);
        }
    }

    pub fn execute_talk_init(&mut self) -> i32 {
        1
    }

    pub fn execute_talk(&mut self, ctx: &mut Ctx<'_>) {
        self.execute_common();
        if self.talk2(1, ctx) == u16::from(STATUS_BOX_CLOSED) {
            self.mode = MODE_WAIT;
            self.execute_set_mode(MODE_WAIT);
            ctx.reset_event = true;
        } else {
            self.set_anm_from_msg_tag(ctx);
        }
    }

    pub fn check_order(&mut self, ctx: &mut Ctx<'_>) {
        if ctx.demo_accept {
            let started = self.get_item_event_idx >= 0
                && ctx
                    .active
                    .start_check(usize::from(self.get_item_event_idx as u16));
            if started && self.state == STATE_THREE {
                self.state = STATE_NONE;
            }
        } else if ctx.talk_command && (self.state == STATE_TWO || self.state == STATE_ONE) {
            self.mode = MODE_TALK;
            self.execute_set_mode(MODE_TALK);
        }
    }

    pub fn event_order(&mut self, ctx: &mut Ctx<'_>) {
        if self.state == STATE_TWO || self.state == STATE_ONE {
            if self.state == STATE_TWO {
                self.speak_order = true;
            }
        } else if self.state == STATE_THREE && self.get_item_event_idx >= 0 {
            ctx.active.order(
                ctx.list,
                usize::from(self.get_item_event_idx as u16),
                ctx.entity,
            );
        }
    }

    pub fn event_move(&mut self, ctx: &mut Ctx<'_>) {
        if !self.chk_end_event(ctx) {
            let old_flag = self.cut_attn;
            if ctx.cut_proc {
                if !self.cut_attn {
                    self.cut_attn = old_flag;
                }
            } else {
                self.private_cut(ctx);
                self.set_anm_from_msg_tag(ctx);
            }
        }
    }

    fn cut_of(&self, ctx: &Ctx<'_>) -> Option<usize> {
        let cut = ctx.active.staff_cut(ctx.list, STAFF_NAME)?;
        usize::try_from(cut.0).ok()
    }

    fn cut_advance(&self, ctx: &Ctx<'_>) -> bool {
        ctx.active
            .staff_cut(ctx.list, STAFF_NAME)
            .map(|(_, advance)| advance)
            .unwrap_or(false)
    }

    pub fn private_cut(&mut self, ctx: &mut Ctx<'_>) {
        let Some(cut) = self.cut_of(ctx) else {
            return;
        };
        let name = ctx.list.cut_name(cut).unwrap_or("");
        let index = if name == CUT_MES_SET {
            0_i8
        } else if name == CUT_GET_ITEM {
            1_i8
        } else {
            -1_i8
        };
        self.cut_index = index;
        if index == -1 {
            ctx.active.end_staff_cut(ctx.list, ctx.tmp, STAFF_NAME);
            return;
        }
        if self.cut_advance(ctx) {
            match index {
                0 => self.event_mes_set_init(cut, ctx),
                1 => self.event_get_item_init(ctx),
                _ => {}
            }
        }
        let temp = match index {
            0 => self.event_mes_set(ctx),
            _ => true,
        };
        if temp {
            ctx.active.end_staff_cut(ctx.list, ctx.tmp, STAFF_NAME);
        }
    }

    pub fn event_mes_set_init(&mut self, cut: usize, ctx: &mut Ctx<'_>) {
        let key = ctx.list.cut_number(cut, "MsgNo");
        if let Some(msg) = key {
            self.msg_tbl = None;
            self.msg_row = NO_ROW;
            let msg = msg as u32;
            match msg {
                1 => {}
                0 => {
                    let next = self.get_msg(ctx);
                    self.set_message(next);
                }
                _ => self.set_message(msg),
            }
            if let Some(tbl) = self.msg_tbl {
                let value = msg_table_at(Some(tbl), self.msg_row);
                self.set_message(value);
            }
        } else {
            self.msg_row += 1;
            let value = msg_table_at(self.msg_tbl, self.msg_row);
            self.set_message(value);
        }
    }

    pub fn event_mes_set(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let temp = self.talk2(0, ctx);
        let closed = temp == u16::from(STATUS_BOX_CLOSED);
        if closed {
            if self.item_kind & ITEM_KIND_HEART != 0 {
                self.item_kind &= !ITEM_KIND_HEART;
                self.item_no = i32::from(ITEM_HEART_PIECE);
                self.state = STATE_THREE;
            } else if self.item_kind & ITEM_KIND_RUPEE != 0 {
                self.item_kind &= !ITEM_KIND_RUPEE;
                self.item_no = i32::from(ITEM_PURPLE_RUPEE);
                self.state = STATE_THREE;
            }
        }
        closed
    }

    pub fn event_get_item_init(&mut self, ctx: &mut Ctx<'_>) {
        let item = u8::try_from(self.item_no).unwrap_or(ITEM_NONE);
        ctx.give_item = Some((ctx.pos, item));
    }

    pub fn talk2(&mut self, param: i32, ctx: &mut Ctx<'_>) -> u16 {
        let mut status = u16::from(MSG_STATUS_NONE);
        if self.msg_id == FPC_M_ERROR_PROCESS_ID {
            if param == 1 {
                let next = self.get_msg(ctx);
                self.curr_msg_no = next;
            }
            let eye = self.eye;
            self.msg_id = ctx.text.message_set_actor(self.curr_msg_no, eye);
            self.msg_live = false;
            self.msg_last_status = u16::MAX;
        } else if self.msg_live {
            status = u16::from(ctx.text.status().unwrap_or(0));
            match status {
                s if s == u16::from(STATUS_MSG_DISPLAYED) => {
                    let mut msg_no = self.curr_msg_no;
                    let next = self.next_msg_status(&mut msg_no, ctx);
                    self.curr_msg_no = msg_no;
                    ctx.text.set_status(next as u8);
                    if next == u16::from(STATUS_MSG_CONTINUES) {
                        let _ = ctx.text.message_set(self.curr_msg_no);
                    }
                }
                s if s == u16::from(STATUS_TYPING) => {
                    if self.msg_last_status == u16::from(STATUS_MSG_CONTINUES) {
                        self.chk_msg();
                    }
                }
                s if s == u16::from(STATUS_BOX_CLOSED) => {
                    ctx.text.set_status(STATUS_MSG_DESTROYED);
                    self.msg_id = FPC_M_ERROR_PROCESS_ID;
                }
                _ => {}
            }
            self.msg_last_status = status;
        } else {
            self.msg_live = ctx.text.live(self.msg_id);
        }
        status
    }

    pub fn chk_msg(&mut self) {}

    pub fn set_message(&mut self, msg_no: u32) {
        self.curr_msg_no = msg_no;
    }

    pub fn next_msg_status(&mut self, msg_no: &mut u32, ctx: &mut Ctx<'_>) -> u16 {
        let mut status = u16::from(STATUS_MSG_CONTINUES);
        if self.msg_tbl.is_some() {
            self.msg_row += 1;
            let value = msg_table_at(self.msg_tbl, self.msg_row);
            if value == MSG_TAG_END {
                ctx.erase_picture = true;
                self.msg_tbl = None;
                status = u16::from(STATUS_MSG_ENDS);
            } else if value == MSG_TAG_STOP {
                self.msg_tbl = None;
                status = u16::from(STATUS_MSG_ENDS);
            } else if value == MSG_TAG_ROOM {
                let reg = ctx.stories.reg(BIT_A9FF);
                let room = fig_room_id(reg);
                *msg_no = MSG_ROOM.get(usize::from(room)).copied().unwrap_or(0);
            } else {
                *msg_no = value;
            }
        } else {
            status = u16::from(STATUS_MSG_ENDS);
        }
        status
    }

    pub fn get_msg(&mut self, ctx: &mut Ctx<'_>) -> u32 {
        let mut msg_no: u32 = 0;
        self.msg_tbl = None;
        self.msg_row = NO_ROW;
        if ctx.photo_mode {
            let mut photo_no = change_photo_no(ctx.picture_result);
            if photo_no <= DSNAP_UNK48 || photo_no >= DSNAP_UNKCF {
                photo_no = PHOTO_NONE;
            } else {
                photo_no = photo_index_to_table_index(photo_no);
            }
            if ctx.picture_format == GX_TF_I4 || ctx.picture_format == GX_TF_I8 {
                self.msg_tbl = Some(TBL_NO_COLOR);
            } else if photo_no == PHOTO_NONE {
                self.msg_tbl = Some(TBL_NO_FIGURE);
            } else if is_figure_get(ctx.stories, photo_no) != 0 {
                self.msg_tbl = Some(TBL_MAKED);
            } else if ctx.picture_detail == 1 {
                self.msg_tbl = Some(TBL_FACE_NG);
            } else if ctx.picture_detail == 2 {
                self.msg_tbl = Some(TBL_BODY_NG);
            } else {
                ctx.stories.set(BIT_2F01);
                ctx.stories.set(BIT_3401);
                ctx.stories.set_reg(BIT_A9FF, photo_no);
                let b = u8::from(ctx.stories.test(BIT_3A01));
                self.msg_tbl = Some(match photo_no {
                    DSNAP_NPC_PHOTO => TBL_MAKE_1ST_MT + b,
                    DSNAP_NPC_RSH1 => TBL_MAKE_1ST_MN + b,
                    _ => TBL_MAKE_1ST + b,
                });
                if ctx.stories.test(BIT_4080) {
                    ctx.stories.clear(BIT_3080);
                    ctx.stories.clear(BIT_4080);
                    ctx.stories.clear(BIT_3F01);
                    ctx.stories.clear(BIT_4040);
                }
            }
        } else if ctx.talk_xy() {
        } else if ctx.stories.test(BIT_3080) {
            if !ctx.stories.test(BIT_4080) {
                if !ctx.stories.test(BIT_3A01) {
                    self.msg_tbl = Some(TBL_1ST_KANSEI);
                } else {
                    self.msg_tbl = Some(TBL_2ND_KANSEI);
                }
                let ev_reg = ctx.stories.reg(BIT_A9FF);
                set_figure(ctx.stories, ev_reg);
                if ev_reg == photo_index_to_table_index(0x4a) {
                    set_figure(ctx.stories, photo_index_to_table_index(0x49));
                } else if ev_reg == photo_index_to_table_index(0x72) {
                    set_figure(ctx.stories, photo_index_to_table_index(0x73));
                    set_figure(ctx.stories, photo_index_to_table_index(0x74));
                    set_figure(ctx.stories, photo_index_to_table_index(0x75));
                    set_figure(ctx.stories, photo_index_to_table_index(0x76));
                    set_figure(ctx.stories, photo_index_to_table_index(0x77));
                    set_figure(ctx.stories, photo_index_to_table_index(0x78));
                } else if ev_reg == photo_index_to_table_index(0x8c) {
                    set_figure(ctx.stories, photo_index_to_table_index(0x8d));
                } else if ev_reg == photo_index_to_table_index(0x88) {
                    set_figure(ctx.stories, photo_index_to_table_index(0x87));
                }
                ctx.stories.set(BIT_4080);
                ctx.stories.clear(BIT_2F01);
            } else if !ctx.stories.test(BIT_3F01) {
                self.msg_tbl = Some(TBL_GALLERY);
            } else if !ctx.stories.test(BIT_4040) {
                ctx.stories.set(BIT_4040);
                self.msg_tbl = Some(TBL_ROOM_IN);
            } else {
                self.msg_tbl = Some(TBL_KANSEI_END);
                ctx.stories.clear(BIT_3080);
                ctx.stories.clear(BIT_4080);
                ctx.stories.clear(BIT_3F01);
                ctx.stories.clear(BIT_4040);
                ctx.tmp.set(TMP_0202);
            }
        } else if ctx.stories.test(BIT_2F01) {
            self.msg_tbl = Some(TBL_MAKING);
        } else if !ctx.stories.test(BIT_2F02) {
            ctx.stories.set(BIT_2F02);
            self.msg_tbl = Some(TBL_1ST_TALK);
        } else if !ctx.stories.test(BIT_3A01) {
            self.msg_tbl = Some(TBL_2ND_TALK);
        } else if !ctx.tmp.test(TMP_0202) {
            ctx.tmp.set(TMP_0202);
            self.msg_tbl = Some(TBL_3RD_TALK);
        } else {
            self.msg_tbl = Some(TBL_4TH_TALK);
        }
        if self.msg_tbl.is_some() {
            msg_no = msg_table_first(self.msg_tbl);
        }
        msg_no
    }

    pub fn set_anm_from_msg_tag(&mut self, ctx: &mut Ctx<'_>) {
        let tag = ctx.text.info.mesg_anime;
        match tag {
            0 => {
                self.set_anm_tbl(Some(ANM_WAIT as u8), NO_ROW);
            }
            1 => {
                self.set_anm_tbl(Some(ANM_TALK as u8), NO_ROW);
            }
            2 => {
                self.set_anm_tbl(Some(ANM_TALK2 as u8), NO_ROW);
            }
            3 => {
                self.set_anm_tbl(Some(ANM_TALK3 as u8), NO_ROW);
            }
            4 => {
                self.set_anm_tbl(Some(ANM_TALK4 as u8), NO_ROW);
            }
            5 => {
                self.set_anm_tbl(Some(ANM_TALK5 as u8), NO_ROW);
            }
            6 => {
                self.set_anm_tbl(Some(ANM_TUKURU as u8), NO_ROW);
            }
            _ => {}
        }
        ctx.text.info.set_mesg_anime_attr_info(0xFF);
    }

    pub fn play_tex_pattern_anm(&mut self) {
        if calc_timer(&mut self.timer) {
            if self.frame >= BTP_FRAME_MAX {
                self.frame = self.frame.wrapping_sub(BTP_FRAME_MAX);
                self.timer = BTP_HOLD;
            } else {
                self.frame = self.frame.wrapping_add(1);
            }
        }
    }

    pub fn play_anm(&mut self) {
        self.anm_flag &= !1;
        if self.morph.play() {
            if let Some(tbl) = self.anm_tbl {
                if self.anm_count > 0 {
                    self.anm_count -= 1;
                    if self.anm_count == 0 {
                        self.anm_row += 1;
                        if self.set_anm_tbl(Some(tbl), self.anm_row) {
                            self.anm_flag |= 1;
                        }
                    } else if let Some(entry) = anm_table(tbl, self.anm_row) {
                        self.set_anm(entry.bck, ANM_ONCE_MODE, 0.0);
                    }
                }
            }
        }
    }

    pub fn set_anm(&mut self, param_1: u8, param_2: i32, morf: f32) {
        let mut blend = morf;
        if self.anim_71c >= 0.0 {
            blend = self.anim_71c;
            self.anim_71c = NO_SPEED;
        }
        let attribute = if param_2 == ANM_LOOP_MODE {
            EMODE_LOOP
        } else {
            EMODE_NONE
        };
        let end = BCK_FRAME_MAX.get(usize::from(param_1)).copied().unwrap_or(0);
        self.morph.set_anm(attribute, end, ANM_FIRST);
        self.morph.set_morf(blend);
        self.bck_index = param_1;
    }

    pub fn set_anm_tbl(&mut self, tbl: Option<u8>, row: usize) -> bool {
        let Some(id) = tbl else {
            self.anm_tbl = None;
            return true;
        };
        let Some(entry) = anm_table(id, row) else {
            self.anm_tbl = None;
            return true;
        };
        if entry.bck == ANM_END {
            self.anm_tbl = None;
            return true;
        }
        self.anm_tbl = Some(id);
        self.anm_row = row;
        self.anm_count = entry.next as i8;
        let mut temp = ANM_LOOP_MODE;
        if self.anm_count > 0 {
            temp = ANM_ONCE_MODE;
        }
        if self.bck_index != entry.bck || temp == ANM_ONCE_MODE {
            self.set_anm(entry.bck, temp, f32::from(entry.morf));
        }
        false
    }

    pub fn chk_end_event(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.get_item_event_idx < 0 {
            return false;
        }
        if ctx.active.end_check(usize::from(self.get_item_event_idx as u16)) {
            self.ev_flag |= EV_FLAG_8;
            return true;
        }
        false
    }

    pub fn chk_attention(&mut self, actor: &Actor, ctx: &Ctx<'_>) {
        self.far_seen = 0;
        if self.cut_attn {
            self.look_at_pos = self.cut_attn_pos;
            self.look_mode = 1;
            if self.no_body != 0 {
                self.head_only = false;
                self.jnt_set_trn();
            } else {
                self.head_only = true;
            }
            if self.attn_seen == 0 {
                self.attn_seen = 1;
            }
        } else {
            let Some(player) = ctx.player else {
                self.head_target = NPC_HEAD_TARGET;
                return;
            };
            let mut temp = self.anim_720;
            let mut temp2 = self.attn_angle;
            let delta = player - actor.pos;
            let dist = (delta.x * delta.x + delta.z * delta.z).sqrt();
            let mut temp5 = walk::to_s16(delta.x.atan2(delta.z));
            if self.attn_seen != 0 {
                temp += ATTN_BONUS_DIST;
                temp2 += ATTN_BONUS_ANGLE;
            }
            temp5 = temp5.wrapping_sub(self.shape_angle[1]);
            let eye = Vec3::new(
                player.x,
                player.y + PLAYER_HEAD_TOP_Y + NPC_EYE_OFFSET_Y,
                player.z,
            );
            if temp > dist && temp2 > i32::from(temp5.unsigned_abs()) {
                self.look_at_pos = eye;
                self.look_mode = 1;
                if self.no_body != 0 {
                    self.head_only = false;
                } else {
                    self.head_only = true;
                }
                if self.look_back == 0 {
                    self.trn_angle = self.home_angle;
                    self.head_only = false;
                    self.look_mode = 2;
                    self.jnt_set_trn();
                }
                if self.attn_seen == 0 {
                    self.attn_seen = 1;
                }
            } else {
                if self.attn_seen == 1 {
                    self.attn_seen = 0;
                    self.wait_timer = NPC_TURN_HOLD;
                }
                if f32::from(NPC_UNK24) > dist || self.attn_forced {
                    self.look_at_pos = eye;
                    self.look_mode = 1;
                    if self.no_body != 0 {
                        self.head_only = false;
                    } else {
                        self.head_only = true;
                    }
                    if self.look_back == 0 {
                        self.trn_angle = self.home_angle;
                        self.head_only = false;
                        self.look_mode = 2;
                        self.jnt_set_trn();
                    }
                    self.far_seen = 1;
                } else {
                    self.look_mode = 0;
                    if self.unk_6f0 == 0 {
                        if self.wait_timer != 0 {
                            self.wait_timer -= 1;
                        } else {
                            self.trn_angle = self.home_angle;
                            self.head_only = false;
                            self.look_mode = 2;
                            self.jnt_set_trn();
                        }
                    }
                }
            }
        }
        self.head_target = NPC_HEAD_TARGET;
    }

    pub fn look_back(&mut self, actor: &Actor, ctx: &Ctx<'_>) {
        let mut target = self.head_target;
        let mut desired_y_rot = actor.angle[1];
        let dst_pos = self.eye;
        let mut dst: Option<Vec3> = None;
        let mut head_only_follow = self.head_only;
        match self.look_mode {
            1 => dst = Some(self.look_at_pos),
            2 => desired_y_rot = self.trn_angle,
            _ => {}
        }
        if self.mode == MODE_TALK && self.no_body != 0 {
            head_only_follow = false;
            self.jnt_set_trn();
        }
        if self.jnt_trn_chk() {
            if self.cut_turn_speed != 0 {
                target = self.cut_turn_speed;
            }
            stream::add_calc_angle_s2(
                &mut self.turn_vel,
                target,
                TURN_SPEED,
                TURN_MAX_STEP,
            );
        } else {
            self.turn_vel = 0;
        }
        let max_vel = self.turn_vel;
        let mut out = self.jnt_angle[1][1];
        self.jnt_look_at(
            &mut out,
            dst,
            dst_pos,
            desired_y_rot,
            max_vel,
            head_only_follow,
        );
        self.jnt_angle[1][1] = out;
        self.shape_angle = actor.angle;
        let _ = ctx;
    }

    pub fn delete(&mut self, stories: &mut StoryBits, tmp: &mut EventFlags, next_stage: Option<&str>) {
        let Some(next) = next_stage else {
            return;
        };
        let mut room_id = FIGURE_ROOM_FIRST;
        while room_id <= FIGURE_ROOM_LAST {
            let index = usize::try_from(room_id).unwrap_or(usize::MAX);
            if next == FIGURE_ROOM_NAME.get(index).copied().unwrap_or("") {
                break;
            }
            room_id += 1;
        }
        if room_id <= FIGURE_ROOM_LAST {
            if stories.test(BIT_4080) {
                let temp = fig_room_id(stories.reg(BIT_A9FF));
                if temp == room_id as u8 {
                    stories.set(BIT_3F01);
                }
            }
        } else {
            tmp.clear(TMP_0202);
            if stories.test(BIT_4080) {
                clear_kansei(stories);
            }
        }
    }
}

pub fn xy_check_cb(
    stories: &StoryBits,
    item_btn: i32,
    select_item: [u8; ITEM_BUTTON_COUNT],
) -> bool {
    if stories.test(BIT_2F02) && !stories.test(BIT_2F01) {
        let index = usize::try_from(item_btn).unwrap_or(usize::MAX);
        let item = select_item.get(index).copied().unwrap_or(ITEM_NONE);
        if item == ITEM_PICTO_BOX || item == ITEM_DELUXE_PICTO_BOX {
            return picture_num(stories) != 0;
        }
    }
    false
}

#[derive(SystemParam)]
pub struct World<'w> {
    pub stories: ResMut<'w, StoryBits>,
    pub tmp: ResMut<'w, EventFlags>,
    pub list: Res<'w, EventList>,
    pub active: ResMut<'w, ActiveEvent>,
    pub next: Res<'w, NextStage>,
    pub do_intent: Res<'w, DoIntent>,
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
    mut actors: Query<(Entity, &mut Actor, &mut NpcMt)>,
    mut commands: Commands,
) {
    let player = msg.players.iter().next().copied().map(|link| link.pos);
    let clear_count = msg.text_flags.clear_count;
    let archive = msg
        .archive
        .as_deref()
        .map(|held| held as &dyn MsgArchive);
    let event_mode = msg.info.event_run_check();
    let pressed = world.do_intent.0;
    let next_stage = world
        .next
        .target
        .as_ref()
        .map(|target| target.stage.as_str());
    for (entity, mut actor, mut npc) in &mut actors {
        let actor = &mut *actor;
        let npc = &mut *npc;
        if !npc.entered {
            npc.entered = true;
            let done = npc.phase_1(
                &mut world.stories,
                &mut world.tmp,
                npc.start_stage_point,
            );
            if done {
                npc.done = true;
                continue;
            }
            npc.staff_idx = world.list.staff_of(STAFF_NAME).unwrap_or(NO_STAFF);
            npc.create_init(actor, &world.list);
        }
        npc.event_mode = event_mode;
        npc.talk_command = pressed;
        npc.picture_num = picture_num(&world.stories);
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
            player,
            stories: &mut world.stories,
            tmp: &mut world.tmp,
            list: &world.list,
            active: &mut world.active,
            next_stage,
            text: &mut text,
            event_mode,
            demo_accept: event_mode,
            talk_command: pressed,
            talk_xy: false,
            cut_proc: npc.cut_proc,
            speak_order: false,
            photo_mode: npc.picture_mode,
            picture_result: npc.picture_result,
            picture_detail: npc.picture_detail,
            picture_format: npc.picture_format,
            picture_num: npc.picture_num,
            select_item: npc.select_item,
            erase_picture: false,
            reset_event: false,
            give_item: None,
        };
        npc.execute(actor, &mut ctx);
        npc.erase_picture = ctx.erase_picture;
        npc.speak_order = ctx.speak_order;
        if ctx.reset_event {
            npc.msg_id = FPC_M_ERROR_PROCESS_ID;
            npc.msg_live = false;
        }
        if ctx.next_stage.is_some() && !npc.torn_down {
            npc.torn_down = true;
            npc.delete(ctx.stories, ctx.tmp, ctx.next_stage);
        }
        if let Some((pos, item)) = ctx.give_item {
            let _ = commands.spawn((
                Actor {
                    name: PRESENT_ITEM_ACTOR.to_owned(),
                    parameters: u32::from(item),
                    pos,
                    angle: [0; 3],
                    set_id: u16::MAX,
                    room: actor.room,
                },
                PresentItem {
                    item,
                    hidden: false,
                },
            ));
        }
    }
}