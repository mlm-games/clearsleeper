use bevy_ecs::prelude::*;
use glam::Vec3;

use super::font::FontBank;
use super::msgbox::{
    PANE_TYPE_ID_TEXT_BOX, PANE_TYPE_ID_WINDOW, PaneSetup, PaneState, TextBox, TextPane,
};
use super::msgdata::{MsgLenState, MsgStickFlags};
use super::msglayout::{MsgLayoutState, fop_msgm_get_color_table};
use super::msgget::{MsgArchive, MsgGet, hyrule_language_check};
use super::msgproc::{
    MsgGameInfo, MeterMsgHio, MsgIconPic, MyPicture, fop_msgm_out_font_draw, fop_msgm_out_font_set,
    fop_msgm_value_increase,
};
use super::{
    FontSlot, ICON_ARROW_DOWN, ICON_ARROW_LEFT, ICON_ARROW_RIGHT, ICON_ARROW_UP,
    ICON_FLASHING_A_BUTTON_2, ICON_INPUT, ICON_NONE, ICON_SELECT_YOKO_LEFT,
    ICON_SELECT_YOKO_RIGHT, MesgEntry, MsgData, SELECT_OFF, STATUS_BOX_CLOSED,
    STATUS_BOX_CLOSING, STATUS_BOX_OPENING, STATUS_CLOSE_WAIT, STATUS_DEMO, STATUS_INPUT,
    STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED, STATUS_MSG_ENDS,
    STATUS_PREPARING, STATUS_TYPING, STATUS_SELECT_2, STATUS_SELECT_3, STATUS_SELECT_YOKO,
    STATUS_STOP, STATUS_TACT,
};

pub const MSG1_HEAP_LOCK: u8 = 7;

pub const MSG1_CURSOR_TEX: &str = "cursor_00_02.bti";
pub const MSG1_BUTTON_TEX: &str = "font_07_02.bti";

pub const MSG1_BEAT_NUM: [u8; 8] = [0x03, 0x04, 0x04, 0x06, 0x06, 0x03, 0x03, 0x04];

pub const MSG1_TEXT_OFFSET_Y: i32 = 2;
pub const MSG1_SCREEN_H: f32 = 480.0f32;
pub const MSG1_SCREEN_MID: f32 = 240.0f32;
pub const MSG1_LINE_WIDTH: i32 = 0x1F7;
pub const MSG1_CENTER_LINE_WIDTH: i32 = 0x1E6;
pub const MSG1_ITEM_LINE_WIDTH: i32 = 0x1A3;
pub const MSG1_ITEM_CENTER_LINE_WIDTH: i32 = 0x192;
pub const MSG1_ITEM_TEXT_W: f32 = 486.0f32;
pub const MSG1_FRAME_W: f32 = 100.0f32;
pub const MSG1_FRAME_OVERSHOOT_W: f32 = 580.0f32;
pub const MSG1_FRAME_CLOSE_W: f32 = 620.0f32;
pub const MSG1_ITEM_Y_SHIFT: f32 = 225.0f32;
pub const MSG1_MSG_BOTTOM: f32 = 240.0f32;
pub const MSG1_TOP_BOTTOM_DY: f32 = 32.0f32;
pub const MSG1_TOP_BOTTOM_DY_BOTTOM: f32 = -26.0f32;
pub const MSG1_SUB_FRAME_WIDE: f32 = 282.0f32;
pub const MSG1_SPACE_TIMER: i32 = 2;
pub const MSG1_WAIT_TIMER_SEND: i32 = 30;
pub const MSG1_DO_STATUS_CANCEL: u8 = 0x27;
pub const MSG1_OUTPUT_TEXT_SIZE: usize = 0x385;
pub const MSG1_OUTPUT_SELECT_SIZE: usize = 0x65;
pub const MSG1_ITEM_BUFFER_SIZE: u32 = 0xC00;
pub const MSG1_TACT_BUFFER_SIZE: u32 = 0x1A280;

pub const MSG1_MSG_NO_TACT_INPUT: u32 = 0x5AC;
pub const MSG1_MSG_NO_TACT_DEMO_1: u32 = 0x5B3;
pub const MSG1_MSG_NO_TACT_DEMO_2: u32 = 0x5B4;
pub const MSG1_MSG_NO_TACT_ALT_FIRST: u32 = 0x5AC;
pub const MSG1_MSG_NO_TACT_ALT_LAST: u32 = 0x5B2;
pub const MSG1_MSG_NO_NUMBER_3: u32 = 0x1CFA;
pub const MSG1_ITEM_FIRST: u32 = 0xC6;
pub const MSG1_ITEM_LAST: u32 = 0xCD;

pub const MSG1_FRAME_OPEN_TALK: i32 = 0xD;
pub const MSG1_FRAME_OPEN_ITEM: i32 = 8;
pub const MSG1_FRAME_CLOSE: i32 = 10;
pub const MSG1_FRAME_OPEN_TALK_RAMP: i32 = 0xA;
pub const MSG1_FRAME_OPEN_ITEM_RAMP: i32 = 5;
pub const MSG1_FRAME_OPEN_ITEM_SETTLE: i32 = 3;
pub const MSG1_ITEM_SCALE_MIN: f32 = 0.6f32;
pub const MSG1_ITEM_SCALE_SLOPE: f32 = 0.5f32;
pub const MSG1_ITEM_SCALE_OVERSHOOT: f32 = -0.100_000_024f32;
pub const MSG1_ITEM_SCALE_OVERSHOOT_BASE: f32 = 1.1f32;
pub const MSG1_FADE_RAMP: i32 = 10;

pub const MSG1_JMA_SIN_SIZE: u32 = 4096;
pub const MSG1_JMA_SIN_SHIFT: u32 = 4;
pub const MSG1_ARROW_SIN_SCALE: f32 = 65_536.0f32 / 6.0f32;
pub const MSG1_ARROW_TIME: [i32; 6] = [0x00, 0x3C, 0x07, 0x04, 0x03, 0x02];
pub const MSG1_ARROW_SCALE_X: [f32; 5] = [1.0f32, 1.3f32, 0.8f32, 1.2f32, 1.0f32];
pub const MSG1_ARROW_SCALE_Y: [f32; 5] = [1.0f32, 0.3f32, 1.1f32, 0.8f32, 1.0f32];

pub const MSG1_DOT_PERIOD: i32 = 60;
pub const MSG1_DOT_HALF: i32 = 30;
pub const MSG1_DOT_BLACK_TARGET: f32 = 150.0f32;
pub const MSG1_DOT_WHITE_TARGET: f32 = 255.0f32;
pub const MSG1_DOT_SILVER_TARGET: f32 = 220.0f32;
pub const MSG1_HALF_ALPHA: f32 = 0.5f32;

pub const MSG1_RING_PERIOD: i16 = 120;
pub const MSG1_RING_HALF: i16 = 60;
pub const MSG1_RING_MIN: f32 = 0.25f32;
pub const MSG1_RING_SPAN: f32 = 0.25f32;

pub const MSG1_LIGHT_PERIOD: i16 = 800;
pub const MSG1_LIGHT_SPAN: f32 = 360.0f32;

pub const MSG1_CORNER_PERIOD: i16 = 240;
pub const MSG1_CORNER_STEP: i32 = 60;

pub const MSG1_CORNER_R: [u8; 4] = [200, 0, 0, 0];
pub const MSG1_CORNER_G: [u8; 4] = [128, 0, 0, 0];
pub const MSG1_CORNER_B: [u8; 4] = [255, 0, 255, 0];
pub const MSG1_CORNER2_R: [u8; 4] = [50, 50, 100, 50];
pub const MSG1_CORNER2_G: [u8; 4] = [50, 50, 100, 50];
pub const MSG1_CORNER2_B: [u8; 4] = [180, 180, 120, 180];
pub const MSG1_CORNER2_A: [u8; 4] = [255, 150, 255, 255];

pub const MSG1_TACT_FRAME_SCALE: [f32; 3] = [1.5f32, 0.96f32, 1.0f32];
pub const MSG1_GUIDE_HIT_MAX: i16 = 5;
pub const MSG1_GUIDE_DEMO_MAX: i16 = 0x23;
pub const MSG1_TACT_RAMP: i32 = 5;
pub const MSG1_TACT_SINE_1: u8 = 1;
pub const MSG1_TACT_SINE_5: u8 = 5;
pub const MSG1_TACT_SINE_6: u8 = 6;
pub const MSG1_TACT_SINE_7: u8 = 7;

pub const MSG1_SELECT_CURSOR_PERIOD: i16 = 0x14;
pub const MSG1_SELECT_CURSOR_SPLIT: i16 = 10;
pub const MSG1_SUB_TEXT_BLEND: [f32; 4] = [1.0f32, 0.0f32, 1.0f32, 1.0f32];
pub const MSG1_SUB_TEXT_BLEND_ALT: [f32; 4] = [0.0f32, 1.0f32, 1.0f32, 1.0f32];

pub const MSG1_NUM_STR: [&str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

pub const MSG1_CAMERA_TAG_MARK: u8 = 0x1A;
pub const MSG1_CAMERA_TAG_CAMERA: u8 = 0x02;
pub const MSG1_CAMERA_TAG_COLOR: u8 = 0xFF;
pub const MSG1_COLOR_INDEX_MAX: u8 = 8;
pub const MSG1_CAMERA_ATTR_MAX: u8 = 5;

pub const MSG1_TCOLOR_BLACK: [u8; 4] = [0x00, 0x00, 0x00, 0x00];
pub const MSG1_TCOLOR_WHITE: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
pub const MSG1_COLOR_WHITE: u32 = 0xFFFF_FFFF;

pub const MSG1_ITEM_BGM: [u32; 6] = [
    0x8000_0002,
    0x8000_0024,
    0x8000_0025,
    0x8000_0027,
    0x8000_004E,
    0x8000_004F,
];

pub const MSG1_TACT_SCISSOR_W_NUM: f32 = 372.0f32;
pub const MSG1_TACT_SCISSOR_H_NUM: f32 = 72.0f32;
pub const MSG1_TACT_SCISSOR_X_MUL: f32 = 1.029_687_5f32;
pub const MSG1_TACT_SCISSOR_Y_MUL: f32 = 1.091_666_7f32;
pub const MSG1_TACT_SCISSOR_X_ADD: f32 = -9.0f32;
pub const MSG1_TACT_SCISSOR_Y_ADD: f32 = -21.0f32;

pub const MSG1_RUBY_OFF_DY: f32 = -4.0f32;

fn to_u8(v: f32) -> u8 {
    ((v.trunc() as i64) & 0xFF) as u8
}

fn pack_color(c: [u8; 4]) -> u32 {
    u32::from_le_bytes(c)
}

fn new_picture() -> MyPicture {
    MyPicture {
        m124: 0.0f32,
        m128: 0.0f32,
        m12c: 0.0f32,
        m130: 0.0f32,
        m134: 0,
        global_bounds: [0.0f32; 2],
        bounds_size: [0.0f32; 2],
        num_texture: 0,
        binding: 0,
        mirror: 0,
        tumble: false,
    }
}

fn c_m_ssin(v: i16) -> f32 {
    let idx = f32::from((v as u16) >> MSG1_JMA_SIN_SHIFT);
    (idx * (std::f32::consts::TAU / MSG1_JMA_SIN_SIZE as f32)).sin()
}

fn item_colors(msg_no: u32) -> Option<(u32, u32)> {
    let black = match msg_no {
        0x66 => [0x00, 0x0A, 0x28, 0x0A],
        0x67 => [0x00, 0x14, 0xA0, 0x14],
        0x68 => [0x00, 0x00, 0x50, 0x50],
        0x69 => [0x00, 0x28, 0x28, 0x50],
        0x6A => [0x00, 0x82, 0x1E, 0x50],
        0x6B => [0x00, 0x28, 0x28, 0x50],
        0x74 => MSG1_TCOLOR_BLACK,
        0x7F => [0x00, 0x00, 0x50, 0x50],
        0x118 | 0x119 | 0x11A | 0x11B | 0x11C => [0x00, 0x82, 0x1E, 0x50],
        0x11D => [0x00, 0x28, 0x28, 0x50],
        0xD2..=0xD7 => MSG1_TCOLOR_BLACK,
        _ => return None,
    };
    let white = match msg_no {
        0x66 => [0xFF, 0xFF, 0x28, 0x28],
        0x67 => [0xFF, 0xC8, 0xFF, 0x32],
        0x68 => [0xFF, 0xFF, 0x00, 0x50],
        0x69 => [0xFF, 0x50, 0x50, 0x50],
        0x6A => [0xFF, 0x78, 0xBE, 0x50],
        0x6B => [0xFF, 0x8C, 0x00, 0x50],
        0x74 => MSG1_TCOLOR_WHITE,
        0x7F => [0xFF, 0xFF, 0x00, 0x50],
        0x118 | 0x119 | 0x11A | 0x11B | 0x11C => [0xFF, 0x78, 0xBE, 0x50],
        0x11D => [0xFF, 0x8C, 0x00, 0x50],
        0xD2 => [0xFF, 0xFF, 0xFF, 0x50],
        0xD3 => [0xFF, 0xFF, 0xAA, 0x50],
        0xD4 => [0xFF, 0x00, 0xC8, 0xFF],
        0xD5 => [0xFF, 0x50, 0x50, 0xFF],
        0xD6 => [0xFF, 0xFF, 0x96, 0xDC],
        0xD7 => MSG1_TCOLOR_WHITE,
        _ => return None,
    };
    Some((pack_color(black), pack_color(white)))
}

fn item_icon_size(msg_no: u32) -> Option<[f32; 2]> {
    Some(match msg_no {
        0xC6 => [32.0f32, 32.0f32],
        199 => [48.0f32, 40.0f32],
        200 => [32.0f32, 32.0f32],
        0xC9 => [32.0f32, 32.0f32],
        0xCA => [16.0f32, 40.0f32],
        0xCB => [56.0f32, 32.0f32],
        0xCC => [40.0f32, 32.0f32],
        0xCD => [40.0f32, 40.0f32],
        _ => return None,
    })
}

pub use super::msghio::MsgMeterHio as Msg1Hio;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Msg1Tact {
    pub melody_num: u8,
    pub beat: u8,
    pub pattern_time: [f32; 8],
    pub pattern: [i32; 8],
}

impl Msg1Tact {
    pub fn beat_num(&self) -> usize {
        MSG1_BEAT_NUM.get(self.melody_num as usize).copied().unwrap_or(0) as usize
    }

    pub fn pattern_time_at(&self, i: usize) -> f32 {
        self.pattern_time.get(i).copied().unwrap_or(0.0f32)
    }

    pub fn pattern_at(&self, i: usize) -> i32 {
        self.pattern.get(i).copied().unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Msg1Se {
    TalkWinOpen,
    TalkNext,
    TalkSelCancel,
    TalkWinClose,
    TalkSelWinOpen,
    Message(u8),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Msg1Env {
    pub opt_ruby: bool,
    pub heap_lock_flag: u8,
    pub clear_count: u8,
    pub triforce_num: u32,
    pub meter_mode: u8,
    pub mesg_status: u8,
    pub mesg_bgm: bool,
    pub sub_bgm: u32,
    pub message_send: bool,
    pub mesg_send_button: u8,
    pub mesg_cancel_button: u8,
    pub mesg_camera_tag_info: i32,
    pub mesg_anime_tag_info: u8,
    pub mesg_camera_attr_info: u8,
    pub do_status_force: u8,
    pub a_status: u8,
    pub message_rupee: i16,
    pub message_set_number: i32,
    pub buy_item_max: u8,
    pub buy_item: i32,
    pub actor_screen: [f32; 2],
    pub player_eye_screen: [f32; 2],
    pub item_texture: bool,
    pub camera_attention: bool,
    pub demo_msg_flag: bool,
    pub tact_msg_flag: bool,
    pub next_msg_flag: bool,
    pub tact: Msg1Tact,
    pub tact_input: bool,
    pub tact_last_input: bool,
    pub tact_left_hand_pos: i16,
    pub se: Option<Msg1Se>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Msg1Stick {
    pub trig_a: bool,
    pub trig_b: bool,
    pub stick_x: f32,
    pub stick_y: f32,
    pub trigger: MsgStickFlags,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Msg1Rot {
    pub cx: f32,
    pub cy: f32,
    pub angle: f32,
}

impl Msg1Rot {
    pub fn set(&mut self, cx: f32, cy: f32, angle: f32) {
        self.cx = cx;
        self.cy = cy;
        self.angle = angle;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Msg1Screen {
    pub tex: &'static str,
    pub frame: Option<PaneSetup>,
    pub arrow: Option<PaneSetup>,
    pub dot: Option<PaneSetup>,
    pub text: [Option<PaneSetup>; 4],
    pub select: [Option<PaneSetup>; 4],
    pub cursor1: Option<PaneSetup>,
    pub cursor2: Option<PaneSetup>,
    pub sub_frame: [Option<PaneSetup>; 10],
    pub item: [Option<PaneSetup>; 10],
    pub tact_frame: Option<PaneSetup>,
    pub tact_text: [Option<PaneSetup>; 4],
    pub tact_frame_alt: Option<PaneSetup>,
    pub tact_text_alt: [Option<PaneSetup>; 4],
    pub tact_ar: [Option<PaneSetup>; 6],
    pub tact_wn: [Option<PaneSetup>; 6],
    pub tact_bs: [Option<PaneSetup>; 6],
    pub tact_i0: [Option<PaneSetup>; 6],
    pub dot_black: u32,
    pub dot_white: u32,
    pub text_binding_h: u8,
    pub text_binding_v: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Msg1DataOpts {
    pub set_string_color: bool,
    pub set_actor: bool,
    pub set_line_space: bool,
    pub line_width: i32,
    pub center_line_width: i32,
}

pub const MSG1_DATA_DEMO: Msg1DataOpts = Msg1DataOpts {
    set_string_color: false,
    set_actor: false,
    set_line_space: false,
    line_width: MSG1_LINE_WIDTH,
    center_line_width: MSG1_CENTER_LINE_WIDTH,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Msg1Phase {
    #[default]
    Init,
    Complete,
}

pub struct Msg1Ctx<'a, F: MsgArchive + ?Sized> {
    pub hio: &'a Msg1Hio,
    pub env: &'a mut Msg1Env,
    pub archive: &'a F,
    pub fonts: &'a FontBank,
    pub mx_font: usize,
    pub rx_font: usize,
    pub screen: &'a Msg1Screen,
    pub len: &'a MsgLenState<'a>,
    pub layout: &'a MsgLayoutState<'a>,
    pub out_hio: &'a MeterMsgHio,
    pub stick: Msg1Stick,
}

pub fn msg_pane_parts_set(pane: &mut PaneState, part: u8) {
    match part {
        1 => {
            pane.pos_top_left[1] = (MSG1_SCREEN_H - (pane.pos_top_left_orig[1] + pane.size_orig[1]))
                + MSG1_TOP_BOTTOM_DY_BOTTOM;
        }
        2 => {
            pane.pos_top_left[1] = MSG1_SCREEN_MID - pane.size_orig[1] * 0.5f32;
        }
        _ => {
            pane.pos_top_left[1] = pane.pos_top_left_orig[1] + MSG1_TOP_BOTTOM_DY;
        }
    }
    pane.pos_center[0] = pane.pos_top_left[0] + pane.size_orig[0] * 0.5f32;
    pane.pos_center[1] = pane.pos_top_left[1] + pane.size_orig[1] * 0.5f32;
}

pub fn arw_pane_parts_set(dst: &mut PaneState, src: &PaneState) {
    dst.pos_top_left[1] =
        (src.pos_top_left[1] + dst.pos_top_left_orig[1]) - src.pos_top_left_orig[1];
    dst.pos_center[1] = dst.pos_top_left[1] + dst.size_orig[1] * 0.5f32;
    dst.cpos_move();
}

fn pane_scale_xy(pane: &mut PaneState, scale: f32) {
    pane.size[0] = pane.size_orig[0] * scale;
    pane.size[1] = pane.size_orig[1] * scale;
    pane.cpos_move();
}

fn pane_data(slot: &mut PaneState, setup: Option<PaneSetup>, type_id: u16) {
    slot.set_pane_data(setup.unwrap_or_default().pane(type_id));
}

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Msg1Box {
    pub msg_no: u32,
    pub field_0xf0: u32,
    pub field_0xf4: u32,
    pub status: u8,
    pub select_num: u8,
    pub pos: Vec3,

    pub mesg_entry: MesgEntry,
    pub msg_data: MsgData,
    pub msg_get: MsgGet,
    pub message: Vec<u8>,
    pub mesg_offset: usize,
    pub msg_id: u32,

    pub m011c: [TextPane; 4],
    pub m011c_rot: [Msg1Rot; 4],
    pub m01fc: PaneState,
    pub m0234: PaneState,
    pub m026c: [PaneState; 10],
    pub m026c_rot: [Msg1Rot; 10],
    pub m026c_tex: [MyPicture; 10],
    pub m049c: PaneState,
    pub m049c_rot: Msg1Rot,
    pub m_pane_arrow: PaneState,
    pub m050c: PaneState,
    pub m0544: [TextPane; 4],
    pub m0624: [PaneState; 10],
    pub m0624_rot: [Msg1Rot; 10],
    pub m08c4: [PaneState; 6],
    pub m0a14: [PaneState; 6],
    pub m0b64: [PaneState; 6],
    pub m0cb4: [PaneState; 6],

    pub frame_white: u32,
    pub frame_black: u32,
    pub frame_corner: [u32; 4],
    pub color_bc: u32,
    pub color_white: u32,
    pub color_bc_now: u32,
    pub color_white_now: u32,
    pub item_black: u32,
    pub item_white: u32,
    pub sub_frame_white: u32,
    pub sub_frame_black: u32,
    pub cursor1_blend: [f32; 4],
    pub cursor2_blend: [f32; 4],
    pub cursor1_tex: &'static str,
    pub cursor2_tex: &'static str,
    pub arrow_rot: Msg1Rot,
    pub arrow_size: [f32; 2],
    pub mask_size: [f32; 2],
    pub input_mask: [i32; 4],
    pub number_cell_w: f32,
    pub number_pane: [TextBox; 3],

    pub m10d0: f32,
    pub m10d4: f32,
    pub m10d8: i32,
    pub m10dc: i32,
    pub m10e0: [f32; 2],
    pub m10ec: i32,
    pub m10f0: i32,
    pub m10f8: i32,
    pub m10fc: i32,
    pub m1100: i32,
    pub m1104: i32,
    pub m1108: i32,
    pub m110c: i32,
    pub m1110: i32,
    pub m1114: i32,

    pub output_text: String,
    pub output_rub: String,
    pub output_text_sdw: String,
    pub output_rub_sdw: String,
    pub select_text: String,
    pub select_rub: String,
    pub select_text_sdw: String,
    pub select_rub_sdw: String,

    pub mesg_camera_tag_info: i32,
    pub m1144: i32,
    pub m1164: u32,
    pub m1168: u8,
    pub m1169: u8,
    pub m116a: u8,
    pub m116b: u8,

    pub mx: usize,
    pub rx: usize,
    pub text_offset_y: i32,
    pub font_size_x: f32,
    pub ruby_size_x: f32,
    pub select_height: f32,
    pub font_flag: bool,
    pub talk_pos_x: f32,
    pub talk_pos_y: f32,
    pub screen2_set: bool,
    pub tact_alt_hidden: bool,
    pub demo_pos_y: [i32; 4],
    pub demo_flag: bool,
    pub demo_cnt: i16,
    pub buffer_size: u32,
    pub item_texture_loaded: bool,
    pub item_particles: u8,
    pub button_icon: [MsgIconPic; 8],
    pub button_kage: [MsgIconPic; 8],
    pub button_timer: [i16; 8],
    pub heap_locked: bool,
    pub alive: bool,
}

impl Default for Msg1Box {
    fn default() -> Self {
        Self {
            msg_no: 0,
            field_0xf0: 0,
            field_0xf4: 0,
            status: STATUS_PREPARING,
            select_num: 0,
            pos: Vec3::ZERO,

            mesg_entry: MesgEntry::default(),
            msg_data: MsgData::new_data_proc(),
            msg_get: MsgGet::default(),
            message: Vec::new(),
            mesg_offset: 0,
            msg_id: 0,

            m011c: Default::default(),
            m011c_rot: Default::default(),
            m01fc: PaneState::default(),
            m0234: PaneState::default(),
            m026c: Default::default(),
            m026c_rot: Default::default(),
            m026c_tex: std::array::from_fn(|_| new_picture()),
            m049c: PaneState::default(),
            m049c_rot: Msg1Rot::default(),
            m_pane_arrow: PaneState::default(),
            m050c: PaneState::default(),
            m0544: Default::default(),
            m0624: Default::default(),
            m0624_rot: Default::default(),
            m08c4: Default::default(),
            m0a14: Default::default(),
            m0b64: Default::default(),
            m0cb4: Default::default(),

            frame_white: 0,
            frame_black: 0,
            frame_corner: [0; 4],
            color_bc: 0,
            color_white: 0,
            color_bc_now: 0,
            color_white_now: 0,
            item_black: 0,
            item_white: 0,
            sub_frame_white: 0,
            sub_frame_black: 0,
            cursor1_blend: [0.0f32; 4],
            cursor2_blend: [0.0f32; 4],
            cursor1_tex: MSG1_CURSOR_TEX,
            cursor2_tex: MSG1_CURSOR_TEX,
            arrow_rot: Msg1Rot::default(),
            arrow_size: [0.0f32; 2],
            mask_size: [0.0f32; 2],
            input_mask: [0; 4],
            number_cell_w: 0.0f32,
            number_pane: Default::default(),

            m10d0: 0.0f32,
            m10d4: 0.0f32,
            m10d8: 0,
            m10dc: 0,
            m10e0: [0.0f32; 2],
            m10ec: 0,
            m10f0: 0,
            m10f8: 0,
            m10fc: 0,
            m1100: 0,
            m1104: 0,
            m1108: 0,
            m110c: 0,
            m1110: 0,
            m1114: 0,

            output_text: String::with_capacity(MSG1_OUTPUT_TEXT_SIZE),
            output_rub: String::with_capacity(MSG1_OUTPUT_TEXT_SIZE),
            output_text_sdw: String::with_capacity(MSG1_OUTPUT_TEXT_SIZE),
            output_rub_sdw: String::with_capacity(MSG1_OUTPUT_TEXT_SIZE),
            select_text: String::with_capacity(MSG1_OUTPUT_SELECT_SIZE),
            select_rub: String::with_capacity(MSG1_OUTPUT_SELECT_SIZE),
            select_text_sdw: String::with_capacity(MSG1_OUTPUT_SELECT_SIZE),
            select_rub_sdw: String::with_capacity(MSG1_OUTPUT_SELECT_SIZE),

            mesg_camera_tag_info: 0,
            m1144: 0,
            m1164: 0,
            m1168: 0,
            m1169: 0,
            m116a: 0,
            m116b: 0,

            mx: 0,
            rx: 0,
            text_offset_y: 0,
            font_size_x: 0.0f32,
            ruby_size_x: 0.0f32,
            select_height: 0.0f32,
            font_flag: false,
            talk_pos_x: 0.0f32,
            talk_pos_y: 0.0f32,
            screen2_set: false,
            tact_alt_hidden: false,
            demo_pos_y: [0; 4],
            demo_flag: false,
            demo_cnt: 0,
            buffer_size: 0,
            item_texture_loaded: false,
            item_particles: 0,
            button_icon: Default::default(),
            button_kage: Default::default(),
            button_timer: [-1; 8],
            heap_locked: false,
            alive: false,
        }
    }
}

impl Msg1Box {
    fn text_pane_data(&mut self, i: usize, setup: Option<PaneSetup>, type_id: u16) {
        if let Some(p) = self.m0544.get_mut(i) {
            p.set_pane_data(setup.unwrap_or_default().pane(type_id));
        }
    }

    fn select_pane_data(&mut self, i: usize, setup: Option<PaneSetup>, type_id: u16) {
        if let Some(p) = self.m011c.get_mut(i) {
            p.set_pane_data(setup.unwrap_or_default().pane(type_id));
        }
    }

    fn text_move(&mut self, i: usize, x: f32, y: f32) {
        if let Some(p) = self.m0544.get_mut(i) {
            p.move_to(x, y);
        }
    }

    fn text_bounds(&self, i: usize) -> [f32; 4] {
        self.m0544
            .get(i)
            .map(|p| p.state.pane.glb_bounds())
            .unwrap_or([0.0f32; 4])
    }

    fn text_shift(&mut self, i: usize, x: f32, y: f32) {
        if let Some(p) = self.m0544.get_mut(i) {
            p.text.shift_set(x, y);
        }
    }

    fn text_init_alpha(&mut self, i: usize) {
        if let Some(p) = self.m0544.get_mut(i) {
            p.set_init_alpha();
        }
    }

    fn text_now_alpha(&mut self, i: usize, alpha: f32) {
        if let Some(p) = self.m0544.get_mut(i) {
            p.state.set_now_alpha(alpha);
        }
    }

    fn text_now_alpha_zero(&mut self, i: usize) {
        self.text_now_alpha(i, 0.0f32);
    }

    fn text_show(&mut self, i: usize) {
        if let Some(p) = self.m0544.get_mut(i) {
            p.show();
        }
    }

    fn text_hide(&mut self, i: usize) {
        if let Some(p) = self.m0544.get_mut(i) {
            p.hide();
        }
    }

    fn text_visible(&self, i: usize) -> bool {
        self.m0544.get(i).is_some_and(|p| p.is_visible())
    }

    fn select_visible(&self, i: usize) -> bool {
        self.m011c.get(i).is_some_and(|p| p.is_visible())
    }

    fn text_char_space(&self, i: usize) -> f32 {
        self.m0544
            .get(i)
            .map(|p| p.text.get_char_space())
            .unwrap_or(0.0f32)
    }

    fn text_line_space(&self, i: usize) -> f32 {
        self.m0544
            .get(i)
            .map(|p| p.text.get_line_space())
            .unwrap_or(0.0f32)
    }

    fn text_font_size(&self, i: usize) -> [f32; 2] {
        self.m0544
            .get(i)
            .map(|p| p.text.get_font_size())
            .unwrap_or([0.0f32; 2])
    }

    fn text_set_font(&mut self, i: usize, slot: usize) {
        if let Some(p) = self.m0544.get_mut(i) {
            p.text.set_font(slot);
        }
    }

    fn text_set_font_size(&mut self, i: usize, x: f32, y: f32) {
        if let Some(p) = self.m0544.get_mut(i) {
            p.text.set_font_size(x, y);
        }
    }

    fn text_set_char_space(&mut self, i: usize, space: f32) {
        if let Some(p) = self.m0544.get_mut(i) {
            p.text.set_char_space(space);
        }
    }

    fn text_set_line_space(&mut self, i: usize, space: f32) {
        if let Some(p) = self.m0544.get_mut(i) {
            p.text.set_line_space(space);
        }
    }

    fn text_set_string(&mut self, i: usize, s: &str) {
        if let Some(p) = self.m0544.get_mut(i) {
            p.text.set_string(s.as_bytes());
        }
    }

    fn icon_num(&self, i: usize) -> u8 {
        self.msg_data.icon_num.get(i).copied().unwrap_or(ICON_NONE)
    }

    fn icon_pos_x(&self, i: usize) -> i32 {
        self.msg_data.icon_pos_x.get(i).copied().unwrap_or(0)
    }

    fn icon_pos_y(&self, i: usize) -> i32 {
        self.msg_data.icon_pos_y.get(i).copied().unwrap_or(0)
    }

    fn icon_scale(&self, i: usize) -> i32 {
        self.msg_data.icon_scale.get(i).copied().unwrap_or(0)
    }

    fn icon_color(&self, i: usize) -> u32 {
        self.msg_data.icon_color.get(i).copied().unwrap_or(0)
    }

    fn icon_uses_text_color(icon_num: u8, flashing_a: bool) -> bool {
        matches!(
            icon_num,
            ICON_ARROW_LEFT | ICON_ARROW_RIGHT | ICON_ARROW_UP | ICON_ARROW_DOWN
                | ICON_SELECT_YOKO_RIGHT
        ) || (flashing_a && icon_num == ICON_FLASHING_A_BUTTON_2)
    }

    fn icon_setup_loop(&mut self, flashing_a: bool) {
        for i in 0..8 {
            let icon_num = self.icon_num(i);
            let mut icon_color = self.icon_color(i);
            if Self::icon_uses_text_color(icon_num, flashing_a) && icon_color == 0 {
                icon_color = MSG1_COLOR_WHITE;
            }
            let timer = self.button_timer.get(i).copied().unwrap_or(-1);
            if icon_num != ICON_NONE
                && timer == -1
                && icon_num != ICON_SELECT_YOKO_LEFT
                && icon_num != ICON_SELECT_YOKO_RIGHT
                && icon_num != ICON_INPUT
            {
                if let (Some(icon), Some(kage), Some(timer)) = (
                    self.button_icon.get_mut(i),
                    self.button_kage.get_mut(i),
                    self.button_timer.get_mut(i),
                ) {
                    fop_msgm_out_font_set(icon, kage, timer, icon_color, icon_num);
                }
            }
        }
    }

    fn data_opts(&self, set_string_color: bool, item_width: bool) -> Msg1DataOpts {
        let narrow =
            item_width && self.mesg_entry.textbox_type == 9 && self.m0624[8].user_area == 0;
        Msg1DataOpts {
            set_string_color,
            set_actor: true,
            set_line_space: true,
            line_width: if narrow {
                MSG1_ITEM_LINE_WIDTH
            } else {
                MSG1_LINE_WIDTH
            },
            center_line_width: if narrow {
                MSG1_ITEM_CENTER_LINE_WIDTH
            } else {
                MSG1_CENTER_LINE_WIDTH
            },
        }
    }

    fn push_msg_strings(&mut self) {
        self.msg_data.out = [
            Some(self.output_text.clone()),
            Some(self.output_rub.clone()),
            Some(self.output_text_sdw.clone()),
            Some(self.output_rub_sdw.clone()),
        ];
        self.msg_data.select_message = [
            Some(self.select_text.clone()),
            Some(self.select_rub.clone()),
            Some(self.select_text_sdw.clone()),
            Some(self.select_rub_sdw.clone()),
        ];
    }

    fn data_setup(
        &mut self,
        opts: &Msg1DataOpts,
        char_space: i32,
        ruby_char_space: i32,
        line_space: i32,
        hio: &Msg1Hio,
    ) {
        self.msg_data.data_init();
        if opts.set_string_color {
            self.msg_data.string_color = self.m1164;
        }
        if opts.set_actor {
            self.msg_data.actor_position = self.pos;
        }
        let bmg = self.message.get(self.mesg_offset..).unwrap_or(&[]).to_vec();
        self.msg_data.set_bmg(bmg);
        self.push_msg_strings();
        self.msg_data.font = self.mx;
        self.msg_data.ruby_font = self.rx;
        self.msg_data.char_space = char_space;
        self.msg_data.ruby_char_space = ruby_char_space;
        if opts.set_line_space {
            self.msg_data.line_space = line_space;
        }
        self.msg_data.entry = self.mesg_entry;
        self.msg_data.font_size = self.m110c;
        self.msg_data.ruby_font_size = self.m1110;
        self.msg_data.line_width = opts.line_width;
        self.msg_data.center_line_width = opts.center_line_width;
        self.msg_data.send_speed = hio.field_0x82;
        self.msg_data.space_timer = MSG1_SPACE_TIMER;
        self.msg_data.space_flag = u8::from(hio.field_0x83 != 0);
    }

    fn common_data_setup(&mut self, opts: &Msg1DataOpts, hio: &Msg1Hio) {
        let char_space = self.text_char_space(0) as i32;
        let ruby_char_space = self.text_char_space(1) as i32;
        let line_space = self.text_line_space(0) as i32;
        self.data_setup(opts, char_space, ruby_char_space, line_space, hio);
    }

    fn common_data_setup_f32(&mut self, opts: &Msg1DataOpts, hio: &Msg1Hio) {
        let char_space = self.text_char_space(0) as i32;
        let ruby_char_space = self.text_char_space(1) as i32;
        self.data_setup(opts, char_space, ruby_char_space, 0, hio);
    }

    fn number_pane_setup(&mut self, hio: &Msg1Hio) {
        for i in 0..3 {
            self.number_pane[i].set_font(self.mx);
        }
        let size_x = hio.field_0x70 as f32;
        for i in 0..3 {
            self.number_pane[i].set_font_size(size_x, size_x);
        }
        self.mask_size = [size_x, size_x];
        for i in 0..3 {
            self.number_pane[i].set_char_space(hio.field_0x5a as f32);
        }
        for i in 0..3 {
            self.number_pane[i].set_line_space(hio.field_0x5e as f32);
        }
    }

    fn text_common_setup(&mut self, hio: &Msg1Hio) {
        self.text_set_font(0, self.mx);
        self.text_set_font(1, self.rx);
        self.text_set_font(2, self.mx);
        self.text_set_font(3, self.rx);
        self.number_pane_setup(hio);
        self.text_offset_y = MSG1_TEXT_OFFSET_Y;
        let size_x = hio.field_0x70 as f32;
        self.text_set_font_size(0, size_x, size_x);
        self.text_set_font_size(2, size_x, size_x);
        self.text_set_char_space(0, hio.field_0x5a as f32);
        self.text_set_char_space(1, hio.field_0x5c as f32);
        self.text_set_char_space(2, hio.field_0x5a as f32);
        self.text_set_char_space(3, hio.field_0x5c as f32);
        self.text_set_line_space(0, hio.field_0x5e as f32);
        self.text_set_line_space(2, hio.field_0x5e as f32);
        self.m110c = self.text_font_size(0)[0] as i32;
    }

    fn text_ruby_setup(&mut self, hio: &Msg1Hio) {
        self.ruby_size_x = hio.field_0x68 as f32;
        let ruby = self.ruby_size_x;
        self.text_set_font_size(1, ruby, ruby);
        self.text_set_font_size(3, ruby, ruby);
        self.m1110 = self.ruby_size_x as i32;
    }

    pub fn screen_data_set_talk(&mut self, screen: &Msg1Screen, hio: &Msg1Hio) {
        pane_data(&mut self.m049c, screen.frame, PANE_TYPE_ID_WINDOW);
        pane_data(&mut self.m_pane_arrow, screen.arrow, PANE_TYPE_ID_WINDOW);
        pane_data(&mut self.m050c, screen.dot, PANE_TYPE_ID_WINDOW);
        for i in 0..4 {
            self.text_pane_data(i, screen.text.get(i).copied().flatten(), PANE_TYPE_ID_TEXT_BOX);
            self.select_pane_data(
                i,
                screen.select.get(i).copied().flatten(),
                PANE_TYPE_ID_TEXT_BOX,
            );
        }
        pane_data(&mut self.m01fc, screen.cursor1, PANE_TYPE_ID_WINDOW);
        pane_data(&mut self.m0234, screen.cursor2, PANE_TYPE_ID_WINDOW);
        for i in 0..10 {
            pane_data(
                self.m026c.get_mut(i),
                screen.sub_frame.get(i).copied().flatten(),
                PANE_TYPE_ID_WINDOW,
            );
            if let Some(p) = self.m026c.get_mut(i) {
                p.init_alpha = 0xAA;
            }
        }
        self.frame_white = hio.field_0x5;
        self.frame_black = hio.field_0x9;
        self.sub_frame_white = 0;
        self.sub_frame_black = 0;
        self.text_common_setup(hio);
        self.cursor1_tex = MSG1_CURSOR_TEX;
        self.cursor2_tex = MSG1_CURSOR_TEX;
        self.m1110 = self.ruby_size_x as i32;
        self.color_bc = screen.dot_black;
        self.color_white = screen.dot_white;
        self.color_bc_now = self.color_bc;
        self.color_white_now = self.color_white;
        self.m026c[0].user_area = 0;
    }

    pub fn screen_data_set_item(&mut self, screen: &Msg1Screen, hio: &Msg1Hio, env: &Msg1Env) {
        pane_data(&mut self.m049c, screen.frame, PANE_TYPE_ID_WINDOW);
        pane_data(&mut self.m_pane_arrow, screen.arrow, PANE_TYPE_ID_WINDOW);
        pane_data(&mut self.m050c, screen.dot, PANE_TYPE_ID_WINDOW);
        for i in 0..4 {
            self.text_pane_data(i, screen.text.get(i).copied().flatten(), PANE_TYPE_ID_TEXT_BOX);
        }
        for (slot, idx) in [
            (8usize, 8usize),
            (9, 9),
            (0, 0),
            (1, 1),
            (2, 2),
            (3, 3),
            (4, 4),
            (5, 5),
            (6, 6),
            (7, 7),
        ] {
            pane_data(
                self.m0624.get_mut(slot),
                screen.item.get(idx).copied().flatten(),
                PANE_TYPE_ID_WINDOW,
            );
        }
        self.buffer_size = MSG1_ITEM_BUFFER_SIZE;
        self.item_texture_loaded = self.mesg_entry.textbox_type == 9 && env.item_texture;
        if self.item_texture_loaded {
            self.m0624[8].user_area = 0;
        } else {
            for p in self.m0624.iter_mut() {
                p.pane.hide();
            }
            self.m0624[8].user_area = 1;
            let d_var12 =
                self.m049c.pos_center_orig[0] - self.m0544[0].state.pos_center_orig[0];
            for p in self.m0544.iter_mut() {
                p.state.size[0] = MSG1_ITEM_TEXT_W;
                p.state.pane_trans(d_var12, 0.0f32);
                p.state.pos_top_left_orig[0] = p.state.pos_top_left[0];
                p.state.pos_center_orig[0] = p.state.pos_center[0];
                p.state.size_orig[0] = p.state.size[0];
            }
        }
        if let Some((black, white)) = item_colors(self.msg_no) {
            self.item_black = black;
            self.item_white = white;
        }
        if let Some(size) = item_icon_size(self.msg_no) {
            let p = &mut self.m0624[8];
            p.state.size = size;
            p.state.pane_trans(0.0f32, 0.0f32);
        }
        if self.mesg_entry.textbox_type == 1 {
            self.frame_white = pack_color(MSG1_TCOLOR_WHITE);
            self.frame_black = pack_color(MSG1_TCOLOR_BLACK);
            self.frame_corner = [
                0x3232_B4FF,
                0x3232_B4FF,
                0x3232_B496,
                0x6464_78FF,
            ];
        }
        self.text_common_setup(hio);
        self.text_ruby_setup(hio);
        self.color_bc = screen.dot_black;
        self.color_white = screen.dot_white;
        self.color_bc_now = self.color_bc;
        self.color_white_now = self.color_white;
        self.m026c[0].user_area = 0;
    }

    pub fn screen_data_set_demo(&mut self, screen: &Msg1Screen, hio: &Msg1Hio) {
        for i in 0..4 {
            self.text_pane_data(i, screen.text.get(i).copied().flatten(), PANE_TYPE_ID_TEXT_BOX);
        }
        self.text_set_font(0, self.mx);
        self.text_set_font(1, self.rx);
        self.text_set_font(2, self.mx);
        self.text_set_font(3, self.rx);
        self.text_offset_y = MSG1_TEXT_OFFSET_Y;
        let size_x = hio.field_0x70 as f32;
        self.text_set_font_size(0, size_x, size_x);
        self.text_set_font_size(2, size_x, size_x);
        self.text_set_char_space(0, hio.field_0x5a as f32);
        self.text_set_char_space(1, hio.field_0x5c as f32);
        self.text_set_char_space(2, hio.field_0x5a as f32);
        self.text_set_char_space(3, hio.field_0x5c as f32);
        self.text_set_line_space(0, hio.field_0x5e as f32);
        self.text_set_line_space(2, hio.field_0x5e as f32);
        self.number_pane_setup(hio);
        self.m110c = self.text_font_size(0)[0] as i32;
        self.m1110 = self.ruby_size_x as i32;
        self.demo_pos_y = [
            self.text_bounds(0)[1] as i32,
            self.text_bounds(1)[1] as i32,
            self.text_bounds(2)[1] as i32,
            self.text_bounds(3)[1] as i32,
        ];
        self.m026c[0].user_area = 0;
    }

    pub fn screen_data_set_tact(&mut self, screen: &Msg1Screen, hio: &Msg1Hio, env: &Msg1Env) {
        if (MSG1_MSG_NO_TACT_ALT_FIRST..=MSG1_MSG_NO_TACT_ALT_LAST).contains(&self.msg_no) {
            pane_data(&mut self.m049c, screen.tact_frame_alt, PANE_TYPE_ID_WINDOW);
            for i in 0..4 {
                self.text_pane_data(
                    i,
                    screen.tact_text_alt.get(i).copied().flatten(),
                    PANE_TYPE_ID_TEXT_BOX,
                );
            }
            self.tact_alt_hidden = false;
        } else {
            pane_data(&mut self.m049c, screen.tact_frame, PANE_TYPE_ID_WINDOW);
            for i in 0..4 {
                self.text_pane_data(
                    i,
                    screen.tact_text.get(i).copied().flatten(),
                    PANE_TYPE_ID_TEXT_BOX,
                );
            }
            self.tact_alt_hidden = true;
        }
        if self.msg_no == MSG1_MSG_NO_TACT_DEMO_1 || self.msg_no == MSG1_MSG_NO_TACT_DEMO_2 {
            self.screen2_set = true;
            for i in 0..env.tact.beat_num() {
                pane_data(
                    self.m08c4.get_mut(i),
                    screen.tact_ar.get(i).copied().flatten(),
                    PANE_TYPE_ID_WINDOW,
                );
                pane_data(
                    self.m0a14.get_mut(i),
                    screen.tact_wn.get(i).copied().flatten(),
                    PANE_TYPE_ID_WINDOW,
                );
                pane_data(
                    self.m0b64.get_mut(i),
                    screen.tact_bs.get(i).copied().flatten(),
                    PANE_TYPE_ID_WINDOW,
                );
                if env.tact.melody_num >= 5 {
                    pane_data(
                        self.m0cb4.get_mut(i),
                        screen.tact_i0.get(i).copied().flatten(),
                        PANE_TYPE_ID_WINDOW,
                    );
                }
            }
        } else {
            self.screen2_set = false;
        }
        self.text_common_setup(hio);
        self.text_ruby_setup(hio);
        self.m1168 = env.tact.beat;
        self.m1144 = 0;
        self.buffer_size = MSG1_TACT_BUFFER_SIZE;
    }

    pub fn screen_data_set(
        &mut self,
        screen: &Msg1Screen,
        hio: &Msg1Hio,
        env: &Msg1Env,
        mx_font: usize,
        rx_font: usize,
    ) {
        self.screen2_set = false;
        if hyrule_language_check(env.clear_count, self.msg_no) {
            self.mx = rx_font;
            self.font_flag = true;
        } else {
            self.mx = mx_font;
            self.font_flag = false;
        }
        self.rx = self.mx;
        self.font_size_x = hio.field_0x70 as f32;
        match self.mesg_entry.textbox_type {
            5 => self.screen_data_set_demo(screen, hio),
            1 | 9 => self.screen_data_set_item(screen, hio, env),
            0xE => self.screen_data_set_tact(screen, hio, env),
            _ => self.screen_data_set_talk(screen, hio),
        }
    }

    pub fn message_pane_show(&mut self) {
        for i in 0..4 {
            self.text_show(i);
            self.text_init_alpha(i);
        }
        if self.mesg_entry.textbox_type == 9 || self.mesg_entry.textbox_type == 1 {
            for p in self.m0624.iter_mut() {
                p.set_init_alpha();
            }
        }
    }

    pub fn message_pane_hide(&mut self, env: &Msg1Env) {
        for i in 0..4 {
            self.text_hide(i);
        }
        if self.mesg_entry.textbox_type == 9 || self.mesg_entry.textbox_type == 1 {
            for p in self.m0624.iter_mut() {
                p.set_now_alpha_zero();
            }
        } else if self.mesg_entry.textbox_type == 0xE
            && (self.msg_no == MSG1_MSG_NO_TACT_DEMO_1
                || self.msg_no == MSG1_MSG_NO_TACT_DEMO_2)
        {
            for i in 0..env.tact.beat_num() {
                if let Some(p) = self.m08c4.get_mut(i) {
                    p.set_now_alpha_zero();
                }
                if let Some(p) = self.m0a14.get_mut(i) {
                    p.set_now_alpha_zero();
                }
                if let Some(p) = self.m0b64.get_mut(i) {
                    p.set_now_alpha_zero();
                }
                if env.tact.melody_num >= 5 {
                    if let Some(p) = self.m0cb4.get_mut(i) {
                        p.set_now_alpha_zero();
                    }
                }
            }
        }
    }

    pub fn out_font_hide(&mut self) {
        for i in 0..8 {
            if let Some(icon) = self.button_icon.get_mut(i) {
                icon.visible = false;
                icon.rotate(0.0f32);
            }
            if let Some(kage) = self.button_kage.get_mut(i) {
                kage.visible = false;
                kage.rotate(0.0f32);
            }
            if let Some(timer) = self.button_timer.get_mut(i) {
                *timer = -1;
            }
        }
    }

    pub fn screen_data_value_init_talk(&mut self, hio: &Msg1Hio, env: &mut Msg1Env) {
        if self.pos == Vec3::ZERO {
            self.m10e0 = [self.m049c.pos_center_orig[0], self.m049c.pos_center_orig[1]];
        } else {
            self.m10e0 = env.actor_screen;
        }
        self.mesg_out_pos(hio, env);
        self.m10d8 = self.m049c.pos_center[0] as i32;
        self.m10dc = self.m049c.pos_center[1] as i32;
        self.m1104 = (self.text_line_space(0) * 0.5f32) as i32;
        self.color_bc_now = self.color_bc;
        self.color_white_now = self.color_white;
        if self.pos != Vec3::ZERO && self.mesg_entry.textbox_type != 10 {
            self.m049c.pos_center[0] = self.m10e0[0];
            self.m049c.pos_center[1] = self.m10e0[1];
        } else {
            self.m10e0 = [self.m049c.pos_center[0], self.m049c.pos_center[1]];
        }
        self.frame_rescale();
        let opts = self.data_opts(true, false);
        self.common_data_setup(&opts, hio);
        self.m10d0 = (self.m10d8 as f32) - self.m049c.pos_center[0];
        self.m10d4 = (self.m10dc as f32) - self.m049c.pos_center[1];
    }

    pub fn screen_data_value_init_item(&mut self, hio: &Msg1Hio, env: &mut Msg1Env) {
        msg_pane_parts_set(&mut self.m049c, 1);
        if self.mesg_entry.textbox_position != 1 {
            self.m049c.pos_center[1] += MSG1_ITEM_Y_SHIFT;
            self.m049c.pos_top_left[1] += MSG1_ITEM_Y_SHIFT;
        }
        let frame = self.m049c.clone();
        arw_pane_parts_set(&mut self.m_pane_arrow, &frame);
        let frame = self.m049c.clone();
        arw_pane_parts_set(&mut self.m050c, &frame);
        self.m1114 = (self.m_pane_arrow.pos_top_left[1] + self.m_pane_arrow.size_orig[1]) as i32;
        let u_var1 = self.m0544[0].state.pos_top_left_orig[0];
        let i_var2 = (MSG1_SCREEN_H
            - (self.m0544[0].state.pos_top_left_orig[1] + self.m0544[0].state.size_orig[1]))
            as i32;
        let i_var9 = i_var2 - 0x13;
        self.m10f8 = u_var1 as i32;
        self.m10fc = if self.mesg_entry.textbox_position != 1 {
            i_var9 + 0xE1
        } else {
            i_var9
        };
        let y = (i_var9 - hio.field_0x62 as i32) - self.text_offset_y;
        let yr = (i_var9 - hio.field_0x60 as i32) - self.text_offset_y;
        self.text_move(0, u_var1, y as f32);
        self.text_move(1, u_var1, yr as f32);
        self.text_move(2, u_var1 + 2.0f32, (y + 2) as f32);
        self.text_move(3, u_var1 + 2.0f32, (yr + 2) as f32);
        self.m10d8 = self.m049c.pos_center[0] as i32;
        self.m10dc = self.m049c.pos_center[1] as i32;
        self.m1104 = (self.text_line_space(0) * 0.5f32) as i32;
        self.color_bc_now = self.color_bc;
        self.color_white_now = self.color_white;
        self.m10e0 = [self.m049c.pos_center[0], self.m049c.pos_center[1]];
        self.m049c.pos_center[0] = self.m10e0[0];
        self.m049c.pos_center[1] = self.m10e0[1];
        self.frame_rescale();
        let opts = self.data_opts(false, true);
        self.common_data_setup(&opts, hio);
        self.m10d0 = (self.m10d8 as f32) - self.m049c.pos_center[0];
        self.m10d4 = (self.m10dc as f32) - self.m049c.pos_center[1];
        self.m0624[9].user_area = 0;
        self.m0624[0].user_area = 0;
        self.m049c.user_area = 0;
    }

    pub fn screen_data_value_init_tact(&mut self, hio: &Msg1Hio) {
        let u_var1 = self.m0544[0].state.pos_top_left_orig[0];
        let i_var2 = self.m0544[0].state.pos_top_left_orig[1];
        self.m10f8 = u_var1 as i32;
        self.m10fc = i_var2 as i32;
        let y = (i_var2 as i32 - hio.field_0x62 as i32) - self.text_offset_y;
        let yr = (i_var2 as i32 - hio.field_0x60 as i32) - self.text_offset_y;
        self.text_move(0, u_var1, y as f32);
        self.text_move(1, u_var1, yr as f32);
        self.text_move(2, u_var1 + 2.0f32, (y + 2) as f32);
        self.text_move(3, u_var1 + 2.0f32, (yr + 2) as f32);
        self.m10d8 = self.m049c.pos_center[0] as i32;
        self.m10dc = self.m049c.pos_center[1] as i32;
        self.m1104 = 0;
        self.color_bc_now = self.color_bc;
        self.color_white_now = self.color_white;
        self.m10e0 = [self.m049c.pos_center[0], self.m049c.pos_center[1]];
        self.m049c.pos_center[0] = self.m10e0[0];
        self.m049c.pos_center[1] = self.m10e0[1];
        self.frame_rescale();
        let opts = self.data_opts(false, true);
        self.common_data_setup(&opts, hio);
        self.m10d0 = (self.m10d8 as f32) - self.m049c.pos_center[0];
        self.m10d4 = (self.m10dc as f32) - self.m049c.pos_center[1];
        self.m0624[9].user_area = 0;
        self.m0624[0].user_area = 0;
        self.m0624[1].user_area = 0;
        self.m049c.user_area = 0;
    }

    pub fn screen_data_value_init_demo(&mut self, hio: &Msg1Hio) {
        self.m1104 = (self.text_line_space(0) * 0.5f32) as i32;
        self.common_data_setup_f32(&MSG1_DATA_DEMO, hio);
    }

    fn frame_rescale(&mut self) {
        self.m049c.size[0] = MSG1_FRAME_W;
        self.m049c.size[1] = (self.m049c.size_orig[1] / self.m049c.size_orig[0]) * MSG1_FRAME_W;
        self.m049c.cpos_move();
    }

    pub fn value_init(&mut self) {
        let text = format!("\u{1b}CC[{:08x}]\u{1b}GM[0]", self.m1164);
        let rub = "\u{1b}CC[FFFFFFFF]\u{1b}GM[0]";
        let sdw = "\u{1b}CC[000000FF]\u{1b}GM[0]";
        self.output_text.clone_from(&text);
        self.output_rub.clone_from(&rub);
        self.output_text_sdw.clone_from(&sdw);
        self.output_rub_sdw.clone_from(&sdw);
        self.select_text.clone_from(&text);
        self.select_rub.clone_from(&rub);
        self.select_text_sdw.clone_from(&sdw);
        self.select_rub_sdw.clone_from(&sdw);
        self.msg_data.string_color = self.m1164;
        self.out_font_hide();
        self.m1108 = 0;
    }

    pub fn set_string(&mut self) {
        let out = [
            self.output_text.clone(),
            self.output_rub.clone(),
            self.output_text_sdw.clone(),
            self.output_rub_sdw.clone(),
        ];
        for (i, s) in out.iter().enumerate() {
            self.text_set_string(i, s);
        }
        if !matches!(self.mesg_entry.textbox_type, 9 | 0xE | 1 | 5) {
            let sel = [
                self.select_text.clone(),
                self.select_rub.clone(),
                self.select_text_sdw.clone(),
                self.select_rub_sdw.clone(),
            ];
            for (i, s) in sel.iter().enumerate() {
                if let Some(p) = self.m011c.get_mut(i) {
                    p.text.set_string(s.as_bytes());
                }
            }
        }
    }

    fn sync_out_from_data(&mut self) {
        let out = self.msg_data.out.clone();
        self.output_text = out[0].clone().unwrap_or_default();
        self.output_rub = out[1].clone().unwrap_or_default();
        self.output_text_sdw = out[2].clone().unwrap_or_default();
        self.output_rub_sdw = out[3].clone().unwrap_or_default();
        let sel = self.msg_data.select_message.clone();
        self.select_text = sel[0].clone().unwrap_or_default();
        self.select_rub = sel[1].clone().unwrap_or_default();
        self.select_text_sdw = sel[2].clone().unwrap_or_default();
        self.select_rub_sdw = sel[3].clone().unwrap_or_default();
    }

    pub fn yose_select<F: FontSlot + ?Sized>(&mut self, font: &F, len: &MsgLenState<'_>) {
        let count = self.msg_data.string_length(font, *len) as i32;
        self.msg_data.count = count;
        self.m1108 = self.msg_data.line_count;
        self.msg_data.line_count = 0;
        self.msg_data.string_shift();
        self.msg_data.icon_idx_refresh();
        self.text_position();
    }

    pub fn text_position(&mut self) {
        let shift = match self.mesg_entry.textbox_type {
            5 => (self.m1104 * 2i32.saturating_sub(self.m1108)) as f32,
            0xE => 0.0f32,
            _ => (self.m1104 * 3i32.saturating_sub(self.m1108)) as f32,
        };
        for i in 0..4 {
            self.text_shift(i, 0.0f32, shift);
        }
    }

    pub fn frame_open_talk(&mut self) {
        self.m1100 = self.m1100.wrapping_add(1);
        if self.m1100 >= MSG1_FRAME_OPEN_TALK {
            self.m049c.pos_center[0] = self.m10d8 as f32;
            self.m049c.pos_center[1] = self.m10dc as f32;
            pane_scale_xy(&mut self.m049c, 1.0f32);
            self.m049c.set_init_alpha();
        } else {
            if self.m1100 < MSG1_FRAME_OPEN_TALK_RAMP {
                let temp_f31 =
                    MSG1_FRAME_W * (self.m049c.size_orig[1] / self.m049c.size_orig[0]);
                let temp_f30 = self.m049c.size_orig[0] - MSG1_FRAME_W;
                let temp_f29 = self.m049c.size_orig[1] - temp_f31;
                let temp_f1 =
                    fop_msgm_value_increase(MSG1_FRAME_OPEN_TALK_RAMP, self.m1100, 0)
                        .unwrap_or_default();
                self.m049c.size[0] = MSG1_FRAME_W + (temp_f30 * temp_f1);
                self.m049c.size[1] = temp_f31 + (temp_f29 * temp_f1);
                self.m049c.pos_center[0] = self.m10e0[0] + (self.m10d0 * temp_f1);
                self.m049c.pos_center[1] = self.m10e0[1] + (self.m10d4 * temp_f1);
                self.m049c.cpos_move();
            } else {
                let temp_f2_2 = MSG1_SCREEN_H / (self.m049c.size_orig[0] - MSG1_FRAME_W);
                let temp_f1_2 = (temp_f2_2 * self.m10d0) - self.m10d0;
                let temp_f1_3 = (temp_f2_2 * self.m10d4) - self.m10d4;
                let temp_f1_4 = (MSG1_FRAME_OVERSHOOT_W
                    * (self.m049c.size_orig[1] / self.m049c.size_orig[0]))
                    - self.m049c.size_orig[1];
                let f0 = self.m1100 as f32 - 10.0f32;
                let temp_f3_2 = c_m_ssin((f0 * MSG1_ARROW_SIN_SCALE) as i16);
                self.m049c.size[0] = self.m049c.size_orig[0]
                    + ((MSG1_FRAME_OVERSHOOT_W - self.m049c.size_orig[0]) * temp_f3_2);
                self.m049c.size[1] = self.m049c.size_orig[1] + (temp_f1_4 * temp_f3_2);
                self.m049c.pos_center[0] = self.m10e0[0] + self.m10d0 + (temp_f1_2 * temp_f3_2);
                self.m049c.pos_center[1] = self.m10e0[1] + self.m10d4 + (temp_f1_3 * temp_f3_2);
                self.m049c.cpos_move();
            }
            let ramp = fop_msgm_value_increase(MSG1_FRAME_OPEN_TALK, self.m1100, 0)
                .unwrap_or_default();
            self.m049c.set_now_alpha(ramp);
        }
    }

    pub fn frame_open_item(&mut self) {
        self.m1100 = self.m1100.wrapping_add(1);
        if self.m1100 >= MSG1_FRAME_OPEN_ITEM {
            self.m049c.pos_center[0] = self.m10d8 as f32;
            self.m049c.pos_center[1] = self.m10dc as f32;
            pane_scale_xy(&mut self.m049c, 1.0f32);
            self.m049c.set_init_alpha();
        } else if self.m1100 <= MSG1_FRAME_OPEN_ITEM_RAMP {
            let d_var2 = fop_msgm_value_increase(MSG1_FRAME_OPEN_ITEM_RAMP, self.m1100, 0)
                .unwrap_or_default();
            self.m049c.pos_center[0] = self.m10e0[0] + (self.m10d0 * d_var2);
            self.m049c.pos_center[1] = self.m10e0[1] + (self.m10d4 * d_var2);
            pane_scale_xy(
                &mut self.m049c,
                (d_var2 * MSG1_ITEM_SCALE_SLOPE) + MSG1_ITEM_SCALE_MIN,
            );
            self.m049c.set_now_alpha(d_var2);
        } else {
            let d_var2 = fop_msgm_value_increase(
                MSG1_FRAME_OPEN_ITEM_SETTLE,
                self.m1100 - MSG1_FRAME_OPEN_ITEM_RAMP,
                0,
            )
            .unwrap_or_default();
            pane_scale_xy(
                &mut self.m049c,
                (d_var2 * MSG1_ITEM_SCALE_OVERSHOOT) + MSG1_ITEM_SCALE_OVERSHOOT_BASE,
            );
            self.m049c.set_init_alpha();
        }
    }

    pub fn frame_close(&mut self, env: &mut Msg1Env) {
        self.message_pane_hide(env);
        self.out_font_hide();
        self.m1100 = self.m1100.wrapping_add(1);
        if self.m1100 == MSG1_FRAME_CLOSE {
            self.m049c.set_now_alpha_zero();
            self.status = STATUS_BOX_CLOSED;
            self.m049c.pane.hide();
        } else {
            let f1 =
                fop_msgm_value_increase(MSG1_FRAME_CLOSE, self.m1100, 0).unwrap_or_default();
            let tmp2 = MSG1_FRAME_CLOSE_W - self.m049c.size_orig[0];
            let f29 = (self.m049c.size_orig[1] / self.m049c.size_orig[0]) * MSG1_FRAME_CLOSE_W
                - self.m049c.size_orig[1];
            self.m049c.size[0] = self.m049c.size_orig[0] + f1 * tmp2;
            self.m049c.size[1] = self.m049c.size_orig[1] + f1 * f29;
            self.m049c.pos_center[0] = self.m10d8 as f32;
            self.m049c.pos_center[1] = self.m10dc as f32;
            self.m049c.cpos_move();
            let d_var3 =
                fop_msgm_value_increase(MSG1_FRAME_CLOSE, MSG1_FRAME_CLOSE - self.m1100, 0)
                    .unwrap_or_default();
            if d_var3 < (self.m049c.now_alpha as f32 / 255.0f32) {
                self.m049c.set_now_alpha(d_var3);
            }
            if self.m1100 == 1 {
                self.set_close_sound(env, 0);
            }
        }
    }

    pub fn ruby_set(&mut self, opt_ruby: bool) {
        let special = matches!(self.mesg_entry.textbox_type, 9 | 0xE | 1 | 5);
        if opt_ruby {
            self.text_hide(1);
            self.text_hide(3);
            if special {
                return;
            }
            if let Some(p) = self.m011c.get_mut(1) {
                p.hide();
            }
            if let Some(p) = self.m011c.get_mut(3) {
                p.hide();
            }
        } else {
            if self.text_visible(1) {
                self.text_show(1);
            }
            if self.text_visible(3) {
                self.text_show(3);
            }
            if special {
                return;
            }
            if self.select_visible(1) {
                if let Some(p) = self.m011c.get_mut(1) {
                    p.show();
                }
            }
            if !self.select_visible(3) {
                return;
            }
            if let Some(p) = self.m011c.get_mut(3) {
                p.show();
            }
        }
    }

    pub fn mesg_out_pos(&mut self, hio: &Msg1Hio, env: &Msg1Env) {
        let mut b_var4 = self.mesg_entry.textbox_position;
        if b_var4 == 0 {
            let local_68 = env.player_eye_screen;
            self.talk_pos_x = (local_68[0] + self.m10e0[0]) * 0.5f32;
            self.talk_pos_y = (local_68[1] + self.m10e0[1]) * 0.5f32;
            let local_58 = hio.field_0x6e as i32;
            b_var4 = if self.talk_pos_y < local_58 as f32 {
                3
            } else {
                1
            };
        }
        msg_pane_parts_set(&mut self.m049c, b_var4);
        let b_var2 = self.mesg_entry.textbox_type;
        if b_var2 != 5 && b_var2 != 0xE {
            let frame = self.m049c.clone();
            arw_pane_parts_set(&mut self.m_pane_arrow, &frame);
            let frame = self.m049c.clone();
            arw_pane_parts_set(&mut self.m050c, &frame);
            self.m1114 =
                (self.m_pane_arrow.pos_top_left[1] + self.m_pane_arrow.size_orig[1]) as i32;
        }
        let u_var1 = self.m0544[0].state.pos_top_left_orig[0];
        let i_var3 = if b_var4 == 1 {
            (MSG1_SCREEN_H
                - (self.m0544[0].state.pos_top_left_orig[1]
                    + self.m0544[0].state.size_orig[1])) as i32
                - 0x13
        } else if b_var4 == 2 {
            (MSG1_SCREEN_MID - self.m0544[0].state.size_orig[1] * 0.5f32) as i32
        } else {
            self.m0544[0].state.pos_top_left_orig[1] as i32 + 0x20
        };
        self.m10f8 = u_var1 as i32;
        self.m10fc = i_var3;
        let y = (i_var3 - hio.field_0x62 as i32) - self.text_offset_y;
        let yr = (i_var3 - hio.field_0x60 as i32) - self.text_offset_y;
        self.text_move(0, u_var1, y as f32);
        self.text_move(1, u_var1, yr as f32);
        self.text_move(2, u_var1 + 2.0f32, (y + 2) as f32);
        self.text_move(3, u_var1 + 2.0f32, (yr + 2) as f32);
    }

    pub fn arrow_init(&mut self) {
        self.m_pane_arrow.set_now_alpha_zero();
        self.m_pane_arrow.pos_center[0] = self.m_pane_arrow.pos_center_orig[0];
        self.m_pane_arrow.pos_center[1] = self.m_pane_arrow.pos_center_orig[1];
        self.m_pane_arrow.size[0] = self.m_pane_arrow.size_orig[0];
        self.m_pane_arrow.size[1] = self.m_pane_arrow.size_orig[1];
        self.m1100 = 0;
    }

    pub fn arrow_move(&mut self, hio: &Msg1Hio) {
        let t9 = MSG1_ARROW_TIME[1];
        let t10 = t9 + MSG1_ARROW_TIME[2];
        let t11 = t10 + MSG1_ARROW_TIME[3];
        let t12 = t11 + MSG1_ARROW_TIME[4];
        let t30 = t12 + MSG1_ARROW_TIME[5];
        self.m1100 = self.m1100.wrapping_add(1);
        if self.m_pane_arrow.now_alpha < self.m_pane_arrow.init_alpha {
            let d_var4 = fop_msgm_value_increase(MSG1_FADE_RAMP, self.m1100, 0).unwrap_or_default();
            self.m_pane_arrow.set_now_alpha(d_var4);
            if self.m1100 == MSG1_FADE_RAMP {
                self.m1100 = 0;
            }
        } else {
            let (f_var1, f_var2) = if self.m1100 <= t9 {
                (1.0f32, 1.0f32)
            } else if self.m1100 <= t10 {
                let span = MSG1_ARROW_TIME[2] as f32;
                let t = self.m1100 as f32 - t9 as f32;
                let f = (t * t) / (span * span);
                (
                    MSG1_ARROW_SCALE_X[0] + (f * (MSG1_ARROW_SCALE_X[1] - MSG1_ARROW_SCALE_X[0])),
                    MSG1_ARROW_SCALE_Y[0] + (f * (MSG1_ARROW_SCALE_Y[1] - MSG1_ARROW_SCALE_Y[0])),
                )
            } else if self.m1100 <= t11 {
                let span = MSG1_ARROW_TIME[3] as f32;
                let t = self.m1100 as f32 - t10 as f32;
                let f = (t * t) / (span * span);
                (
                    MSG1_ARROW_SCALE_X[1] + (f * (MSG1_ARROW_SCALE_X[2] - MSG1_ARROW_SCALE_X[1])),
                    MSG1_ARROW_SCALE_Y[1] + (f * (MSG1_ARROW_SCALE_Y[2] - MSG1_ARROW_SCALE_Y[1])),
                )
            } else if self.m1100 <= t12 {
                let span = MSG1_ARROW_TIME[4] as f32;
                let t = self.m1100 as f32 - t11 as f32;
                let f = (t * t) / (span * span);
                (
                    MSG1_ARROW_SCALE_X[2] + (f * (MSG1_ARROW_SCALE_X[3] - MSG1_ARROW_SCALE_X[2])),
                    MSG1_ARROW_SCALE_Y[2] + (f * (MSG1_ARROW_SCALE_Y[3] - MSG1_ARROW_SCALE_Y[2])),
                )
            } else if self.m1100 <= t30 {
                let span = MSG1_ARROW_TIME[5] as f32;
                let t = self.m1100 as f32 - t12 as f32;
                let f = (t * t) / (span * span);
                (
                    MSG1_ARROW_SCALE_X[3] + (f * (MSG1_ARROW_SCALE_X[4] - MSG1_ARROW_SCALE_X[3])),
                    MSG1_ARROW_SCALE_Y[3] + (f * (MSG1_ARROW_SCALE_Y[4] - MSG1_ARROW_SCALE_Y[3])),
                )
            } else {
                self.m1100 = 0;
                (1.0f32, 1.0f32)
            };
            self.m_pane_arrow.size[0] = self.m_pane_arrow.size_orig[0] * f_var1;
            self.m_pane_arrow.size[1] = self.m_pane_arrow.size_orig[1] * f_var2;
            self.m_pane_arrow.pos_center[1] =
                hio.field_0x66 as f32 + ((self.m1114 as f32) - self.m_pane_arrow.size[1] / 2.0f32);
            self.m_pane_arrow.cpos_move();
        }
    }

    pub fn dot_move(&mut self) {
        self.m1100 = self.m1100.wrapping_add(1);
        if self.m050c.now_alpha < self.m050c.init_alpha {
            let d_var5 = fop_msgm_value_increase(MSG1_FADE_RAMP, self.m1100, 0).unwrap_or_default();
            self.m050c.set_now_alpha(d_var5);
            if self.m1100 == MSG1_FADE_RAMP {
                self.m1100 = 0;
            }
        } else {
            let i_var4 = self.m1100;
            if i_var4 >= MSG1_DOT_PERIOD {
                self.color_bc_now = self.color_bc;
                self.color_white_now = self.color_white;
                self.m050c.set_init_alpha();
                self.m1100 = 0;
            } else {
                let step = if i_var4 > MSG1_DOT_HALF {
                    MSG1_DOT_PERIOD - i_var4
                } else {
                    i_var4
                };
                let t =
                    fop_msgm_value_increase(MSG1_DOT_HALF, step, 0).unwrap_or_default();
                let black = self.color_bc.to_le_bytes();
                let white = self.color_white.to_le_bytes();
                let white_target = [
                    MSG1_DOT_WHITE_TARGET,
                    MSG1_DOT_WHITE_TARGET,
                    MSG1_DOT_SILVER_TARGET,
                ];
                let mut next_black = [0u8; 4];
                let mut next_white = [0u8; 4];
                for ch in 0..3 {
                    next_black[ch] = to_u8(
                        black[ch] as f32
                            + ((MSG1_DOT_BLACK_TARGET - black[ch] as f32) * t),
                    );
                    next_white[ch] =
                        to_u8(white[ch] as f32 + ((white_target[ch] - white[ch] as f32) * t));
                }
                next_black[3] = black[3];
                next_white[3] = white[3];
                self.color_bc_now = pack_color(next_black);
                self.color_white_now = pack_color(next_white);
                let init = self.m050c.init_alpha as f32;
                self.m050c.now_alpha = to_u8(init + ((255.0f32 - init) * t));
            }
        }
    }

    pub fn ring_move(&mut self) {
        let p = &mut self.m0624[9];
        p.user_area = p.user_area.wrapping_add(1);
        if p.user_area >= MSG1_RING_PERIOD {
            p.user_area = 0;
        }
        let user_area = self.m0624[9].user_area as i32;
        let d_var2 = if user_area < MSG1_RING_HALF as i32 {
            fop_msgm_value_increase(MSG1_RING_HALF as i32, user_area, 0).unwrap_or_default()
        } else {
            fop_msgm_value_increase(
                MSG1_RING_HALF as i32,
                MSG1_RING_PERIOD as i32 - user_area,
                0,
            )
            .unwrap_or_default()
        };
        let p = &mut self.m0624[9];
        p.set_now_alpha((d_var2 * MSG1_RING_SPAN) + MSG1_RING_MIN);
    }

    pub fn light_move(&mut self) {
        let p = &mut self.m0624[0];
        p.user_area = p.user_area.wrapping_add(1);
        if p.user_area >= MSG1_LIGHT_PERIOD {
            p.user_area = 0;
        }
        let user_area = self.m0624[0].user_area as f32;
        let f31 = (user_area / MSG1_LIGHT_PERIOD as f32) * MSG1_LIGHT_SPAN;
        let pivots: [(usize, f32, f32); 8] = [
            (0, self.m0624[0].size[0], 0.0f32),
            (1, 0.0f32, 0.0f32),
            (2, self.m0624[2].size[0], self.m0624[2].size[1]),
            (3, 0.0f32, self.m0624[3].size[1]),
            (4, self.m0624[4].size[0], 0.0f32),
            (5, 0.0f32, 0.0f32),
            (6, self.m0624[6].size[0], self.m0624[6].size[1]),
            (7, 0.0f32, self.m0624[7].size[1]),
        ];
        for (n, (i, cx, cy)) in pivots.iter().enumerate() {
            let angle = if n < 4 { -f31 } else { f31 };
            if let Some(r) = self.m0624_rot.get_mut(*i) {
                r.set(*cx, *cy, angle);
            }
        }
        for i in 0..4 {
            if let Some(p) = self.m0624.get_mut(i) {
                p.set_init_alpha();
            }
        }
    }

    pub fn corner_move(&mut self) {
        self.m049c.user_area = self.m049c.user_area.wrapping_add(1);
        if self.m049c.user_area >= MSG1_CORNER_PERIOD {
            self.m049c.user_area = 0;
        }
        let user_area = self.m049c.user_area;
        let tmp = fop_msgm_value_increase(
            MSG1_CORNER_STEP,
            (user_area % MSG1_CORNER_STEP as i16) as i32,
            2,
        )
        .unwrap_or_default();
        let mut local_68 = [[0u8; 4]; 4];
        if self.mesg_entry.textbox_type == 1 {
            for (i, slot) in local_68.iter_mut().enumerate() {
                let r9 = (i + 1) % 4;
                slot[0] = to_u8(
                    MSG1_CORNER2_R[i] as f32
                        + (tmp * (MSG1_CORNER2_R[r9] as f32 - MSG1_CORNER2_R[i] as f32)),
                );
                slot[1] = to_u8(
                    MSG1_CORNER2_G[i] as f32
                        + (tmp * (MSG1_CORNER2_G[r9] as f32 - MSG1_CORNER2_G[i] as f32)),
                );
                slot[2] = to_u8(
                    MSG1_CORNER2_B[i] as f32
                        + (tmp * (MSG1_CORNER2_B[r9] as f32 - MSG1_CORNER2_B[i] as f32)),
                );
                slot[3] = to_u8(
                    MSG1_CORNER2_A[i] as f32
                        + (tmp * (MSG1_CORNER2_A[r9] as f32 - MSG1_CORNER2_A[i] as f32)),
                );
            }
        } else {
            for (i, slot) in local_68.iter_mut().enumerate() {
                let r9 = (i + 1) % 4;
                slot[0] = to_u8(
                    MSG1_CORNER_R[i] as f32
                        + (tmp * (MSG1_CORNER_R[r9] as f32 - MSG1_CORNER_R[i] as f32)),
                );
                slot[1] = to_u8(
                    MSG1_CORNER_G[i] as f32
                        + (tmp * (MSG1_CORNER_G[r9] as f32 - MSG1_CORNER_G[i] as f32)),
                );
                slot[2] = to_u8(
                    MSG1_CORNER_B[i] as f32
                        + (tmp * (MSG1_CORNER_B[r9] as f32 - MSG1_CORNER_B[i] as f32)),
                );
                slot[3] = 255;
            }
        }
        let order = if user_area < 60 {
            [0usize, 1, 3, 2]
        } else if user_area < 120 {
            [1usize, 2, 0, 3]
        } else if user_area < 180 {
            [2usize, 3, 1, 0]
        } else {
            [3usize, 0, 2, 1]
        };
        self.frame_corner = [
            pack_color(local_68[order[0]]),
            pack_color(local_68[order[1]]),
            pack_color(local_68[order[2]]),
            pack_color(local_68[order[3]]),
        ];
    }

    pub fn tact_guide_show(&mut self, env: &Msg1Env, param_2: u8) {
        for i in 0..env.tact.beat_num() {
            if i < param_2 as usize {
                if env.tact.melody_num < 5 {
                    if let Some(p) = self.m08c4.get_mut(i) {
                        p.set_init_alpha();
                    }
                    if let Some(p) = self.m0a14.get_mut(i) {
                        p.set_now_alpha(MSG1_HALF_ALPHA);
                    }
                    if let Some(p) = self.m0b64.get_mut(i) {
                        p.set_now_alpha_zero();
                    }
                } else {
                    if let Some(p) = self.m0b64.get_mut(i) {
                        p.set_init_alpha();
                    }
                    if let Some(p) = self.m0a14.get_mut(i) {
                        p.set_now_alpha(MSG1_HALF_ALPHA);
                    }
                    if let Some(p) = self.m08c4.get_mut(i) {
                        p.set_now_alpha(MSG1_HALF_ALPHA);
                    }
                    if let Some(p) = self.m0cb4.get_mut(i) {
                        p.set_now_alpha(MSG1_HALF_ALPHA);
                    }
                }
            } else {
                if let Some(p) = self.m08c4.get_mut(i) {
                    p.pane.hide();
                }
                if let Some(p) = self.m0a14.get_mut(i) {
                    p.pane.hide();
                }
                if let Some(p) = self.m0b64.get_mut(i) {
                    p.pane.hide();
                }
                if env.tact.melody_num >= 5 {
                    if let Some(p) = self.m0cb4.get_mut(i) {
                        p.pane.hide();
                    }
                }
            }
        }
    }

    pub fn number_input(&mut self, env: &mut Msg1Env, stick: &Msg1Stick) {
        for i in 0..8 {
            if self.icon_num(i) == ICON_INPUT {
                let b = self.text_bounds(0);
                self.m10ec = b[0] as i32 + self.icon_pos_x(i);
                self.m10f0 =
                    b[1] as i32 + (self.m1104 * (3i32.saturating_sub(self.m1108) + (self.icon_pos_y(i) * 2)));
            }
        }
        let columns = if self.msg_no == MSG1_MSG_NO_NUMBER_3 {
            3
        } else {
            2
        };
        let result = self.msg_data.input_number(
            columns,
            env.message_set_number as i16,
            env.buy_item_max,
            stick.trigger,
        );
        self.select_num = result.index;
        env.message_set_number = result.number as i32;
        self.msg_data.color_anime();
        let num = env.message_set_number;
        let digits = [(num / 100) as usize, ((num % 100) / 10) as usize, (num % 10) as usize];
        for slot in [2usize, 1, 0] {
            let s = MSG1_NUM_STR
                .get(digits.get(slot).copied().unwrap_or(0))
                .copied()
                .unwrap_or("0");
            self.number_pane[slot].set_string(s.as_bytes());
        }
        env.buy_item = num;
        if stick.trig_a || stick.trig_b || env.message_send {
            self.set_cancel_mode(env, stick.trig_b);
        } else {
            env.meter_mode = 3;
        }
    }

    pub fn tact_input(&mut self, env: &mut Msg1Env, stick: &Msg1Stick) {
        let melody_no = env.tact.melody_num;
        let beat_num = env.tact.beat_num();
        if (self.m1100 as usize) < beat_num {
            if env.tact.beat as usize != beat_num || self.m1168 != env.tact.beat {
                self.m1100 = 0;
                if let Some(p) = self.m0624.get_mut(0) {
                    p.user_area = 0;
                }
                self.m1168 = env.tact.beat;
                self.tact_guide_show(env, env.tact.beat);
            }
            if env.tact_input {
                let s_var1 = env.tact_left_hand_pos;
                let local_38 = env.tact.pattern_at(self.m1100 as usize);
                self.m1100 = self.m1100.wrapping_add(1);
                if s_var1 as i32 == local_38 && env.tact.beat as usize == beat_num {
                    let i_var3 = self.m0624[0].user_area as i32;
                    if i_var3 == self.m1100 - 1 {
                        if let Some(p) = self.m0b64.get_mut(i_var3 as usize) {
                            p.user_area = 1;
                        }
                        if let Some(p) = self.m0624.get_mut(0) {
                            p.user_area = p.user_area.wrapping_add(1);
                        }
                    }
                }
                if self.m1100 >= env.tact.beat as i32 && !env.tact_last_input {
                    self.m1100 = 0;
                    if let Some(p) = self.m0624.get_mut(0) {
                        p.user_area = 0;
                    }
                    self.tact_guide_show(env, env.tact.beat);
                }
            } else if melody_no != MSG1_TACT_SINE_1
                && melody_no != MSG1_TACT_SINE_5
                && melody_no != MSG1_TACT_SINE_6
                && melody_no != MSG1_TACT_SINE_7
                && (stick.trig_b || env.message_send)
            {
                self.set_cancel_mode(env, stick.trig_b);
            }
        }
        for i in 0..beat_num {
            let s_var1 = self.m0b64.get(i).map(|p| p.user_area).unwrap_or(0);
            if s_var1 > 0 && s_var1 <= MSG1_GUIDE_HIT_MAX {
                let alpha =
                    (fop_msgm_value_increase(MSG1_TACT_RAMP, s_var1 as i32, 0).unwrap_or_default()
                        * MSG1_HALF_ALPHA)
                        + MSG1_HALF_ALPHA;
                if env.tact.melody_num < 5 {
                    if let Some(p) = self.m0a14.get_mut(i) {
                        p.set_now_alpha(alpha);
                    }
                    if let Some(p) = self.m0b64.get_mut(i) {
                        p.set_now_alpha(alpha);
                    }
                } else {
                    if let Some(p) = self.m0a14.get_mut(i) {
                        p.set_now_alpha(alpha);
                    }
                    if let Some(p) = self.m08c4.get_mut(i) {
                        p.set_now_alpha(alpha);
                    }
                    if let Some(p) = self.m0cb4.get_mut(i) {
                        p.set_now_alpha(alpha);
                    }
                }
                let last = self
                    .m0b64
                    .get(beat_num.wrapping_sub(1))
                    .map(|p| p.user_area)
                    .unwrap_or(0);
                if last == 5 {
                    self.status = STATUS_MSG_DISPLAYED;
                    self.m1100 = 0;
                }
                if let Some(p) = self.m0b64.get_mut(i) {
                    p.user_area = p.user_area.wrapping_add(1);
                }
            }
        }
    }

    pub fn tact_input2(&mut self, env: &mut Msg1Env, stick: &Msg1Stick) {
        if !stick.trig_b {
            return;
        }
        env.mesg_cancel_button = 1;
    }

    pub fn sub_text_scale(&mut self, param_2: f32) {
        for i in 1..10 {
            if let Some(p) = self.m026c.get_mut(i) {
                let x = p.pos_top_left[0] * param_2;
                let y = p.pos_top_left[1] * param_2;
                let w = p.size[0] * param_2;
                let h = p.size[1] * param_2;
                p.pane.move_to(x, y);
                p.pane.resize(w, h);
            }
        }
    }

    pub fn sub_text_size_set(&mut self, hio: &Msg1Hio) {
        let i_var7 = self.text_line_space(0) as i32;
        let select_height = match self.status {
            STATUS_SELECT_2 => (i_var7 + self.m110c) as f32,
            STATUS_SELECT_3 => (self.m110c + i_var7 * 2) as f32,
            _ => self.select_height,
        };
        self.select_height = select_height;
        let mut f_var1 = self.m049c.pos_center[1];
        if f_var1 >= MSG1_MSG_BOTTOM {
            f_var1 = (hio.field_0x7c as i32 as f32) + (f_var1 - self.m049c.size[1] / 2.0f32);
        } else {
            f_var1 = (f_var1 + self.m049c.size[1] / 2.0f32) - (hio.field_0x7c as i32 as f32);
        }
        let local_68 = hio.field_0x78 as i32 as f32;
        let mut f_var3 = local_68
            + ((hio.field_0x7e as f32) * 2.0f32
                + (self.msg_data.select_length as f32)
                - self.m011c[0].state.size_orig[0]);
        for i in 0..4 {
            self.m011c[i].state.size[0] = self.m011c[i].state.size_orig[0] + f_var3;
        }
        self.m026c[5].size[0] = self.m026c[5].size_orig[0] + f_var3;
        self.m026c[8].size[0] = self.m026c[8].size_orig[0] + f_var3;
        self.m026c[9].size[0] = self.m026c[9].size_orig[0] + f_var3;
        f_var3 = (hio.field_0x7a as i32 as f32) + (select_height - self.m011c[0].state.size_orig[1]);
        for i in 0..4 {
            self.m011c[i].state.size[1] = self.m011c[i].state.size_orig[1] + f_var3;
        }
        self.m026c[6].size[1] = self.m026c[6].size_orig[1] + f_var3;
        self.m026c[7].size[1] = self.m026c[7].size_orig[1] + f_var3;
        self.m026c[9].size[1] = self.m026c[9].size_orig[1] + f_var3;
        f_var3 = (self.m026c[9].size_orig[0] - self.m026c[9].size[0]) / 2.0f32;
        self.m026c[0].pos_center[0] = self.m026c[0].pos_center_orig[0] + f_var3;
        self.m026c[1].pos_center[0] = self.m026c[1].pos_center_orig[0] + f_var3;
        self.m026c[2].pos_center[0] = self.m026c[2].pos_center_orig[0] - f_var3;
        self.m026c[3].pos_center[0] = self.m026c[3].pos_center_orig[0] + f_var3;
        self.m026c[4].pos_center[0] = self.m026c[4].pos_center_orig[0] - f_var3;
        self.m026c[6].pos_center[0] = self.m026c[6].pos_center_orig[0] + f_var3;
        self.m026c[7].pos_center[0] = self.m026c[7].pos_center_orig[0] - f_var3;
        f_var3 = (self.m011c[0].state.size_orig[0] - self.m011c[0].state.size[0]) / 2.0f32;
        for i in 0..4 {
            self.m011c[i].state.pos_center[0] =
                (hio.field_0x7e as f32 + self.m011c[i].state.pos_center_orig[0]) + f_var3;
        }
        let f_var5 = (self.m026c[9].size_orig[1] - self.m026c[9].size[1]) / 2.0f32;
        let f_var2 = f_var1 - self.m026c[0].pos_center_orig[1];
        self.m026c[0].pos_center[1] = f_var1;
        self.m026c[1].pos_center[1] = self.m026c[1].pos_center_orig[1] - f_var5;
        self.m026c[2].pos_center[1] = self.m026c[2].pos_center_orig[1] - f_var5;
        self.m026c[3].pos_center[1] = self.m026c[3].pos_center_orig[1] + f_var5;
        self.m026c[4].pos_center[1] = self.m026c[4].pos_center_orig[1] + f_var5;
        self.m026c[5].pos_center[1] = self.m026c[5].pos_center_orig[1] - f_var5;
        self.m026c[8].pos_center[1] = self.m026c[8].pos_center_orig[1] + f_var5;
        for i in 0..4 {
            self.m011c[i].state.pos_center[1] =
                self.m011c[i].state.pos_center_orig[1] + f_var2;
        }
        self.m01fc.pos_center_orig[0] = self.m01fc.pos_top_left_orig[0]
            + self.m01fc.size_orig[0] / 2.0f32
            + f_var3;
        let f4 = (hio.field_0x7e as f32) * 2.0f32;
        let f3 = self.m01fc.pos_center_orig[0] + self.m01fc.size_orig[0];
        self.m0234.pos_center_orig[0] = f3 + (self.msg_data.select_length as f32) + f4;
        self.m01fc.pos_center_orig[1] =
            self.m01fc.pos_top_left_orig[1] + self.m01fc.size_orig[1] / 2.0f32 + f_var5;
        self.m0234.pos_center_orig[1] =
            self.m0234.pos_top_left_orig[1] + self.m0234.size_orig[1] / 2.0f32 + f_var5;
        for i in 0..4 {
            if let Some(p) = self.m011c.get_mut(i) {
                p.cpos_move();
            }
        }
        for i in 0..10 {
            if let Some(p) = self.m026c.get_mut(i) {
                p.cpos_move();
            }
        }
        let corner_angle = if self.m026c[9].size[0] >= MSG1_SUB_FRAME_WIDE {
            2.0f32
        } else {
            3.0f32
        };
        let cx = self.m026c[0].size[0] / 2.0f32;
        let cy = self.m026c[0].size[1] / 2.0f32;
        self.m026c_rot[0].set(cx, cy, corner_angle);
        for i in 0..4 {
            let half_h = self.m011c[i].state.size[1] / 2.0f32;
            let anchor = half_h + self.m026c[0].pos_center[1];
            let dy = anchor - self.m011c[i].state.pos_center[1];
            let dx = (self.m011c[i].state.size[0] / 2.0f32 + self.m026c[0].pos_center[0])
                - self.m011c[i].state.pos_center[0];
            self.m011c_rot[i].set(dx, dy, corner_angle);
        }
        let scale_x = {
            let v = self.m026c[9].size[0];
            let max = hio.field_0x74 as i32;
            if max as f32 <= v {
                1.0f32
            } else {
                ((v / max as f32) + 0.5f32) as i32 as f32
            }
        };
        let scale_y = {
            let v = self.m026c[9].size[1];
            let max = hio.field_0x74 as i32;
            if max as f32 <= v {
                1.0f32
            } else {
                ((v / max as f32) + 0.5f32) as i32 as f32
            }
        };
        for (i, uv) in [(5usize, scale_x), (8, scale_x), (6, scale_y), (7, scale_y)] {
            if let Some(tex) = self.m026c_tex.get_mut(i) {
                tex.m134 = 1;
                tex.m124 = 0.0f32;
                tex.m128 = 0.0f32;
                tex.m12c = uv;
                tex.m130 = 1.0f32;
            }
        }
        self.m026c[9].user_area = 0;
    }

    pub fn sub_text_open(&mut self, hio: &Msg1Hio) {
        let tmp = hio.field_0x44 as i32 + hio.field_0x46 as i32;
        self.m026c[9].user_area = self.m026c[9].user_area.wrapping_add(1);
        let d_var4 =
            fop_msgm_value_increase(tmp, self.m026c[9].user_area as i32, 0).unwrap_or_default();
        let i_var3 = self.m026c[9].user_area as i32;
        let i_var2 = hio.field_0x44 as i32;
        let f_var1 = if i_var3 < i_var2 {
            let d_var5 = fop_msgm_value_increase(i_var2, i_var3, 0).unwrap_or_default();
            (d_var5 * (MSG1_TACT_FRAME_SCALE[1] - MSG1_TACT_FRAME_SCALE[0]))
                + MSG1_TACT_FRAME_SCALE[0]
        } else {
            let d_var5 =
                fop_msgm_value_increase(hio.field_0x46 as i32, i_var3 - i_var2, 0).unwrap_or_default();
            (d_var5 * (MSG1_TACT_FRAME_SCALE[2] - MSG1_TACT_FRAME_SCALE[1]))
                + MSG1_TACT_FRAME_SCALE[1]
        };
        self.sub_text_scale(f_var1);
        for p in self.m026c.iter_mut() {
            p.set_now_alpha(d_var4);
        }
        let d_var5 = 1.0f32 - ((1.0f32 - (hio.field_0x81 as f32 / 255.0f32)) * d_var4);
        self.m049c
            .set_now_alpha(1.0f32 - ((1.0f32 - (hio.field_0x80 as f32 / 255.0f32)) * d_var4));
        for i in 0..4 {
            self.text_now_alpha(i, d_var5);
        }
    }

    pub fn sub_text_close(&mut self, hio: &Msg1Hio) {
        self.m026c[9].user_area = self.m026c[9].user_area.wrapping_add(1);
        let i_var1 = self.m026c[9].user_area as i32;
        let i_var2 = hio.field_0x48 as i32;
        if i_var1 <= i_var2 {
            let d_var3 = fop_msgm_value_increase(i_var2, i_var2 - i_var1, 0).unwrap_or_default();
            let d_var4 =
                fop_msgm_value_increase(hio.field_0x48 as i32, self.m026c[9].user_area as i32, 0)
                    .unwrap_or_default();
            self.sub_text_scale(((hio.field_0x40 - 1.0f32) * d_var4) + 1.0f32);
            for p in self.m026c.iter_mut() {
                p.set_now_alpha(d_var3);
            }
            self.m01fc.set_now_alpha_zero();
            self.m0234.set_now_alpha_zero();
            for p in self.m011c.iter_mut() {
                p.state.set_now_alpha_zero();
            }
        } else {
            let d_var3 = fop_msgm_value_increase(
                hio.field_0x76 as i32,
                hio.field_0x76 as i32 - (i_var1 - i_var2),
                0,
            )
            .unwrap_or_default();
            let d_var4 = 1.0f32 - ((1.0f32 - (hio.field_0x81 as f32 / 255.0f32)) * d_var3);
            self.m049c
                .set_now_alpha(1.0f32 - ((1.0f32 - (hio.field_0x80 as f32 / 255.0f32)) * d_var3));
            for i in 0..4 {
                self.text_now_alpha(i, d_var4);
            }
        }
    }

    pub fn set_close_sound(&mut self, env: &mut Msg1Env, param_2: u8) {
        env.se = Some(if param_2 != 0 || env.do_status_force == MSG1_DO_STATUS_CANCEL {
            Msg1Se::TalkSelCancel
        } else {
            Msg1Se::TalkWinClose
        });
    }

    pub fn set_cancel_mode(&mut self, env: &mut Msg1Env, trig_b: bool) {
        self.status = STATUS_MSG_DISPLAYED;
        self.m1100 = 0;
        if trig_b {
            self.m1169 = 1;
            env.mesg_cancel_button = 1;
        } else {
            self.m1169 = 0;
        }
        self.set_close_sound(env, 0);
        self.m116a = self.m116a.wrapping_add(1);
        env.mesg_send_button = self.m116a;
    }

    pub fn message_show<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        len: &MsgLenState<'_>,
        layout: &MsgLayoutState<'_>,
        env: &mut Msg1Env,
    ) {
        pane_scale_xy(&mut self.m049c, 1.0f32);
        self.m049c.set_init_alpha();
        self.message_pane_show();
        self.yose_select(font, len);
        if self.mesg_entry.initial_sound != 0 {
            env.se = Some(Msg1Se::Message(self.mesg_entry.initial_sound));
        }
        self.msg_data.short_cut();
        self.msg_data.string_set(font, layout);
        self.icon_setup_loop(true);
        self.sync_out_from_data();
        self.set_string();
        self.m1100 = 0;
        for i in 0..env.tact.beat_num() {
            if let Some(p) = self.m0b64.get_mut(i) {
                p.user_area = 0;
            }
            if env.tact.melody_num < 5 {
                if let Some(p) = self.m0a14.get_mut(i) {
                    p.set_now_alpha(MSG1_HALF_ALPHA);
                }
                if let Some(p) = self.m0b64.get_mut(i) {
                    p.set_now_alpha_zero();
                }
            } else {
                if let Some(p) = self.m0a14.get_mut(i) {
                    p.set_now_alpha(MSG1_HALF_ALPHA);
                }
                if let Some(p) = self.m08c4.get_mut(i) {
                    p.set_now_alpha(MSG1_HALF_ALPHA);
                }
                if let Some(p) = self.m0cb4.get_mut(i) {
                    p.set_now_alpha(MSG1_HALF_ALPHA);
                }
            }
        }
    }

    fn send_button(&mut self, env: &mut Msg1Env) {
        self.m116a = self.m116a.wrapping_add(1);
        env.mesg_send_button = self.m116a;
    }

    fn talk_next_se(&mut self, env: &mut Msg1Env) {
        env.se = Some(if env.do_status_force == MSG1_DO_STATUS_CANCEL {
            Msg1Se::TalkSelCancel
        } else {
            Msg1Se::TalkNext
        });
    }

    fn arrow_or_idle(&mut self, hio: &Msg1Hio, env: &mut Msg1Env) {
        if self.mesg_entry.textbox_type != 5 && self.mesg_entry.textbox_type != 0xE {
            self.arrow_move(hio);
        }
        env.meter_mode = 1;
    }

    fn dot_or_idle(&mut self, env: &mut Msg1Env, meter: u8) {
        if self.mesg_entry.textbox_type != 5 && self.mesg_entry.textbox_type != 0xE {
            self.dot_move();
        }
        env.meter_mode = meter;
    }

    fn dot_now_alpha_zero(&mut self) {
        if self.mesg_entry.textbox_type != 5 && self.mesg_entry.textbox_type != 0xE {
            self.m050c.set_now_alpha_zero();
        }
    }

    fn short_cut_check(&mut self) {
        if self.msg_data.field_0x296 == 0 {
            self.msg_data.short_cut();
        }
        if self.msg_data.auto_send_flag == 0
            && self.msg_data.hand_send_flag == 0
            && self.msg_data.wait_timer != 0
        {
            self.msg_data.wait_timer = 0;
            self.msg_data.field_0x296 = 0;
        }
    }

    fn advance_page<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        len: &MsgLenState<'_>,
        env: &mut Msg1Env,
    ) {
        self.value_init();
        self.yose_select(font, len);
        self.set_string();
        if self.mesg_entry.textbox_type != 5 && self.mesg_entry.textbox_type != 0xE {
            self.arrow_init();
        }
        env.meter_mode = 2;
        self.talk_next_se(env);
    }

    pub fn stop_proc<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        len: &MsgLenState<'_>,
        hio: &Msg1Hio,
        env: &mut Msg1Env,
        stick: &Msg1Stick,
    ) -> i32 {
        if self.msg_data.auto_send_flag != 0 {
            if self.msg_data.dec_wait_timer() == 0 {
                self.msg_data.auto_send_flag = 0;
                self.msg_data.field_0x296 = 0;
                self.status = STATUS_TYPING;
                self.advance_page(font, len, env);
                self.send_button(env);
            } else {
                self.arrow_or_idle(hio, env);
            }
        } else if (stick.trig_a || stick.trig_b) && !env.mesg_bgm {
            self.status = STATUS_TYPING;
            self.advance_page(font, len, env);
            if self.mesg_entry.draw_type == 0 && stick.trig_b && !env.mesg_bgm {
                self.short_cut_check();
            }
            self.send_button(env);
        } else {
            self.arrow_or_idle(hio, env);
        }
        1
    }

    fn select_cursor_blend(&mut self) {
        if let Some(p) = self.m026c.get_mut(8) {
            p.user_area = p.user_area.wrapping_add(1);
            if p.user_area >= MSG1_SELECT_CURSOR_PERIOD {
                p.user_area = 0;
            }
        }
        let s_var3 = self.m026c[8].user_area;
        let blend = if s_var3 < MSG1_SELECT_CURSOR_SPLIT {
            MSG1_SUB_TEXT_BLEND
        } else if s_var3 < MSG1_SELECT_CURSOR_PERIOD {
            MSG1_SUB_TEXT_BLEND_ALT
        } else {
            return;
        };
        self.cursor1_blend = blend;
        self.cursor2_blend = blend;
    }

    pub fn select_proc(
        &mut self,
        hio: &Msg1Hio,
        env: &mut Msg1Env,
        stick: &Msg1Stick,
    ) -> i32 {
        let bounds = self.text_bounds(0);
        let i_var10 = bounds[0] as i32;
        let i_var11 = bounds[1] as i32;
        let u_var1 = self.text_line_space(0) as i32;
        let arrow_size = hio.field_0x70;
        let s_var3 = self.m026c[0].user_area;
        if s_var3 == 0 {
            if self.status == STATUS_SELECT_2 || self.status == STATUS_SELECT_3 {
                if stick.trig_a || stick.trig_b || env.message_send {
                    self.sub_text_size_set(hio);
                    self.m026c[0].user_area = 1;
                    env.se = Some(Msg1Se::TalkSelWinOpen);
                    return 1;
                }
                self.arrow_or_idle(hio, env);
            } else if self.status == STATUS_SELECT_YOKO {
                self.m026c[0].user_area = 2;
            }
            return 1;
        }
        if s_var3 == 1 {
            if (self.m026c[9].user_area as i32) < (hio.field_0x44 as i32 + hio.field_0x46 as i32) {
                self.sub_text_open(hio);
                return 1;
            }
            self.m01fc.set_init_alpha();
            self.m0234.set_init_alpha();
            for p in self.m011c.iter_mut() {
                p.set_init_alpha();
            }
            self.m026c[9].user_area = 0;
            self.m01fc.pane_trans(0.0f32, 0.0f32);
            self.m0234.pane_trans(0.0f32, 0.0f32);
            self.m026c[0].user_area = 2;
            if self.mesg_entry.textbox_type == 5 || self.mesg_entry.textbox_type == 0xE {
                return 1;
            }
            self.arrow_init();
            return 1;
        }
        if s_var3 != 2 {
            return 1;
        }
        if self.status == STATUS_SELECT_2 {
            self.select_num = self.msg_data.select_check2(
                self.arrow_size[0],
                self.arrow_size[1],
                arrow_size,
                i_var10,
                i_var11 + u_var1 + (u_var1 / 2),
                u_var1,
                stick.stick_y,
            );
            self.select_cursor_blend();
            let dy = (self.select_num as i32 * u_var1) as f32;
            self.m01fc.pane_trans(0.0f32, dy);
            self.m0234.pane_trans(0.0f32, dy);
        } else if self.status == STATUS_SELECT_3 {
            self.select_num = self.msg_data.select_check3(
                self.arrow_size[0],
                self.arrow_size[1],
                arrow_size,
                i_var10,
                i_var11,
                u_var1,
                stick.stick_y,
            );
            self.select_cursor_blend();
            let dy = (self.select_num as i32 * u_var1) as f32;
            self.m01fc.pane_trans(0.0f32, dy);
            self.m0234.pane_trans(0.0f32, dy);
        } else if self.status == STATUS_SELECT_YOKO {
            let mut var_r5 = 0i32;
            let mut var_r6 = 0i32;
            let mut var_r11 = 0i32;
            for i in 0..8 {
                let icon_num = self.icon_num(i);
                if icon_num == ICON_SELECT_YOKO_LEFT {
                    var_r5 = bounds[0] as i32 + self.icon_pos_x(i);
                    var_r6 =
                        bounds[1] as i32 + (self.m1104 * (2i32.saturating_sub(self.m1108) + (self.icon_pos_y(i) * 2)));
                } else if icon_num == ICON_SELECT_YOKO_RIGHT {
                    var_r11 = bounds[0] as i32 + self.icon_pos_x(i);
                }
            }
            self.select_num = self.msg_data.select_check_yoko(
                self.arrow_size[0],
                self.arrow_size[1],
                arrow_size,
                var_r5,
                var_r6,
                var_r11 - var_r5,
                stick.stick_x,
            );
        }
        if !stick.trig_a
            && ((!stick.trig_b || env.a_status != MSG1_DO_STATUS_CANCEL) && !env.message_send)
        {
            return 1;
        }
        if self.status == STATUS_SELECT_2 || self.status == STATUS_SELECT_3 {
            self.m026c[0].user_area = 3;
            self.m01fc.set_now_alpha_zero();
            self.m0234.set_now_alpha_zero();
            for p in self.m011c.iter_mut() {
                p.state.set_now_alpha_zero();
            }
        } else {
            self.m026c[0].user_area = 0;
        }
        self.m1100 = 0;
        self.m026c[9].user_area = 0;
        if stick.trig_b {
            self.m1169 = 1;
            env.mesg_cancel_button = 1;
        } else {
            self.m1169 = 0;
        }
        let u_var9 = if self.mesg_entry.next_msg_no == 1 {
            u8::from(self.select_num == 0)
        } else if self.mesg_entry.next_msg_no == 2 {
            0
        } else if (self.status == STATUS_SELECT_2 && self.select_num == 1)
            || (self.status == STATUS_SELECT_3 && self.select_num == 2)
        {
            1
        } else {
            0
        };
        self.set_close_sound(env, u_var9);
        self.status = STATUS_MSG_DISPLAYED;
        self.send_button(env);
        1
    }

    pub fn input_proc(&mut self, env: &mut Msg1Env, stick: &Msg1Stick) -> i32 {
        if self.mesg_entry.textbox_type == 0xE {
            if env.tact_msg_flag {
                self.tact_input2(env, stick);
            } else {
                self.tact_input(env, stick);
            }
        } else {
            self.number_input(env, stick);
        }
        1
    }

    pub fn demo_proc<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        len: &MsgLenState<'_>,
        layout: &MsgLayoutState<'_>,
        env: &mut Msg1Env,
    ) -> i32 {
        if self.m116b == 0 {
            let beat_num = env.tact.beat_num();
            let mut local_64 = [0.0f32; 7];
            for i in 0..beat_num {
                local_64[i + 1] = local_64[i] + env.tact.pattern_time_at(i);
            }
            if self.m1100 == 0 {
                self.demo_cnt = 0;
            }
            let total = (local_64.get(beat_num).copied().unwrap_or(0.0f32) as i16) as i32;
            if self.m1100 < total {
                self.m1100 = self.m1100.wrapping_add(1);
                let mark = (local_64
                    .get(self.demo_cnt as usize)
                    .copied()
                    .unwrap_or(0.0f32) as i16) as i32;
                if self.m1100 >= mark {
                    if let Some(p) = self.m0b64.get_mut(self.demo_cnt as usize) {
                        p.user_area = 1;
                    }
                    self.demo_cnt = self.demo_cnt.wrapping_add(1);
                }
            }
            for i in 0..beat_num {
                let user_area = self.m0b64.get(i).map(|p| p.user_area).unwrap_or(0);
                if user_area > 0 && user_area <= MSG1_GUIDE_DEMO_MAX {
                    if user_area < MSG1_GUIDE_HIT_MAX {
                        let f_var10 =
                            (fop_msgm_value_increase(MSG1_TACT_RAMP, user_area as i32, 0)
                                .unwrap_or_default()
                                * MSG1_HALF_ALPHA)
                                + MSG1_HALF_ALPHA;
                        if env.tact.melody_num < 5 {
                            if let Some(p) = self.m0a14.get_mut(i) {
                                p.set_now_alpha(f_var10);
                            }
                            if let Some(p) = self.m0b64.get_mut(i) {
                                p.set_now_alpha(f_var10);
                            }
                        } else {
                            if let Some(p) = self.m0a14.get_mut(i) {
                                p.set_now_alpha(f_var10);
                            }
                            if let Some(p) = self.m08c4.get_mut(i) {
                                p.set_now_alpha(f_var10);
                            }
                            if let Some(p) = self.m0cb4.get_mut(i) {
                                p.set_now_alpha(f_var10);
                            }
                        }
                    } else {
                        if let Some(p) = self.m0a14.get_mut(i) {
                            p.set_init_alpha();
                        }
                        if let Some(p) = self.m0b64.get_mut(i) {
                            p.set_init_alpha();
                        }
                        if env.tact.melody_num >= 5 {
                            if let Some(p) = self.m0cb4.get_mut(i) {
                                p.set_init_alpha();
                            }
                        }
                    }
                    if let Some(p) = self.m0b64.get_mut(i) {
                        p.user_area = p.user_area.wrapping_add(1);
                    }
                    let last = self
                        .m0b64
                        .get(beat_num.wrapping_sub(1))
                        .map(|p| p.user_area)
                        .unwrap_or(0);
                    if last > MSG1_GUIDE_DEMO_MAX {
                        if self.demo_flag {
                            self.message_pane_show();
                            self.yose_select(font, len);
                            if self.mesg_entry.initial_sound != 0 {
                                env.se =
                                    Some(Msg1Se::Message(self.mesg_entry.initial_sound));
                            }
                            self.msg_data.short_cut();
                            self.msg_data.string_set(font, layout);
                            self.icon_setup_loop(true);
                            self.sync_out_from_data();
                            self.set_string();
                            for j in 0..4 {
                                self.text_now_alpha_zero(j);
                            }
                            self.m116b = 1;
                            self.demo_flag = false;
                        } else {
                            self.demo_flag = true;
                        }
                        self.m1100 = 0;
                        for j in 0..beat_num {
                            if let Some(p) = self.m0b64.get_mut(j) {
                                p.user_area = 0;
                            }
                            if env.tact.melody_num < 5 {
                                if let Some(p) = self.m0a14.get_mut(j) {
                                    p.set_now_alpha(MSG1_HALF_ALPHA);
                                }
                                if let Some(p) = self.m0b64.get_mut(j) {
                                    p.set_now_alpha_zero();
                                }
                            } else {
                                if let Some(p) = self.m0a14.get_mut(j) {
                                    p.set_now_alpha(MSG1_HALF_ALPHA);
                                }
                                if let Some(p) = self.m08c4.get_mut(j) {
                                    p.set_now_alpha(MSG1_HALF_ALPHA);
                                }
                                if let Some(p) = self.m0cb4.get_mut(j) {
                                    p.set_now_alpha(MSG1_HALF_ALPHA);
                                }
                            }
                        }
                    }
                }
            }
        } else {
            let d_var9 =
                fop_msgm_value_increase(MSG1_TACT_RAMP, self.m116b as i32, 0).unwrap_or_default();
            for i in 0..4 {
                self.text_now_alpha(i, d_var9);
            }
            if self.m116b as i32 >= MSG1_TACT_RAMP {
                self.status = STATUS_INPUT;
            } else {
                self.m116b = self.m116b.wrapping_add(1);
            }
        }
        1
    }

    pub fn continue_proc<F: MsgArchive + ?Sized, G: FontSlot + ?Sized>(
        &mut self,
        archive: &F,
        font: &G,
        hio: &Msg1Hio,
        env: &mut Msg1Env,
        stick: &Msg1Stick,
        len: &MsgLenState<'_>,
    ) -> i32 {
        let opt = self.msg_data.select_flag != SELECT_OFF;
        if (stick.trig_a
            || stick.trig_b
            || opt
            || env.message_send
            || (self.msg_no == MSG1_MSG_NO_TACT_INPUT && env.mesg_cancel_button != 0))
            && !env.mesg_bgm
        {
            self.msg_data.select_off();
            self.m1100 = 0;
            if self.mesg_entry.textbox_type != 5 && self.mesg_entry.textbox_type != 0xE {
                self.arrow_init();
            }
            if stick.trig_b {
                self.m1169 = 1;
                env.mesg_cancel_button = 1;
            }
            if self.m1169 != 0 {
                self.msg_no = self.field_0xf0;
                self.m1169 = 0;
            }
            let mut get = MsgGet::default();
            let Some(head) = get.get_mesg_header(archive, env.clear_count, self.msg_no) else {
                return 1;
            };
            let Some(message) = get.get_message(head) else {
                return 1;
            };
            self.message = message.to_vec();
            self.mesg_offset = 0;
            if let Some(entry) = get.get_mesg_entry(head) {
                self.mesg_entry = entry;
            }
            self.msg_get = get;
            self.msg_id = self.msg_get.get_mesg_number();
            self.m116a = 0;
            self.m1164 = u32::MAX;
            self.value_init();
            let opts = self.data_opts(true, true);
            self.common_data_setup(&opts, hio);
            self.yose_select(font, len);
            if self.mesg_entry.initial_sound != 0 {
                env.se = Some(Msg1Se::Message(self.mesg_entry.initial_sound));
            }
            self.set_string();
            env.meter_mode = 2;
            self.talk_next_se(env);
            self.send_button(env);
            env.mesg_cancel_button = 0;
            env.mesg_anime_tag_info = self.mesg_entry.initial_animation;
            env.mesg_camera_attr_info = self.mesg_entry.initial_camera;
            env.message_rupee = self.mesg_entry.item_price;
            if (1..=MSG1_CAMERA_ATTR_MAX).contains(&self.mesg_entry.initial_camera) {
                env.camera_attention = true;
            }
            if self.mesg_entry.textbox_type == 1 {
                msg_pane_parts_set(&mut self.m049c, 1);
                if self.mesg_entry.textbox_position != 1 {
                    self.m049c.pos_center[1] += MSG1_ITEM_Y_SHIFT;
                    self.m049c.pos_top_left[1] += MSG1_ITEM_Y_SHIFT;
                }
                let frame = self.m049c.clone();
                arw_pane_parts_set(&mut self.m_pane_arrow, &frame);
                let frame = self.m049c.clone();
                arw_pane_parts_set(&mut self.m050c, &frame);
                self.m1114 = (self.m_pane_arrow.pos_top_left[1]
                    + self.m_pane_arrow.size_orig[1]) as i32;
                let d_var3 = self.m0544[0].state.pos_top_left_orig[0];
                let f_var5 = (MSG1_SCREEN_H
                    - (self.m0544[0].state.pos_top_left_orig[1]
                        + self.m0544[0].state.size_orig[1])) as i32
                    - 0x13;
                self.m10f8 = d_var3 as i32;
                self.m10fc = if self.mesg_entry.textbox_position != 1 {
                    (f_var5 + MSG1_ITEM_Y_SHIFT as i32) as i32
                } else {
                    f_var5
                };
                let offset_y = self.text_offset_y as f32;
                self.text_move(0, d_var3, f_var5 as f32 - hio.field_0x62 as f32 - offset_y);
                self.text_move(1, d_var3, f_var5 as f32 - hio.field_0x60 as f32 - offset_y);
                self.text_move(
                    2,
                    d_var3 + 2.0f32,
                    (f_var5 as f32 - hio.field_0x62 as f32 - offset_y) + 2.0f32,
                );
                self.text_move(
                    3,
                    d_var3 + 2.0f32,
                    (f_var5 as f32 - hio.field_0x60 as f32 - offset_y) + 2.0f32,
                );
                self.m10d8 = self.m049c.pos_center[0] as i32;
                self.m10dc = self.m049c.pos_center[1] as i32;
                self.m049c.cpos_move();
            } else if self.mesg_entry.textbox_type != 0xE {
                self.mesg_out_pos(hio, env);
                self.m10d8 = self.m049c.pos_center[0] as i32;
                self.m10dc = self.m049c.pos_center[1] as i32;
            }
            if self.mesg_entry.draw_type == 0 && stick.trig_b && !env.mesg_bgm {
                self.short_cut_check();
            }
            self.status = STATUS_TYPING;
        } else {
            self.arrow_or_idle(hio, env);
        }
        1
    }

    pub fn closewait_proc(&mut self, env: &mut Msg1Env, stick: &Msg1Stick) -> i32 {
        if self.msg_data.auto_send_flag != 0 {
            if self.msg_data.dec_wait_timer() == 0 || env.message_send {
                self.msg_data.auto_send_flag = 0;
                self.msg_data.field_0x296 = 0;
                self.status = STATUS_BOX_CLOSING;
                self.m1100 = 0;
                self.msg_data.wait_timer = 0;
                self.send_button(env);
            }
        } else if self.msg_data.hand_send_flag != 0 {
            if self.msg_data.dec_wait_timer() != 0 {
                if (stick.trig_a || stick.trig_b || env.message_send) && !env.mesg_bgm {
                    self.msg_data.hand_send_flag = 0;
                    self.status = STATUS_BOX_CLOSING;
                    self.dot_now_alpha_zero();
                    self.m1100 = 0;
                    env.meter_mode = 0;
                    self.send_button(env);
                } else {
                    self.dot_or_idle(env, 4);
                }
            } else {
                self.msg_data.hand_send_flag = 0;
                self.status = STATUS_BOX_CLOSING;
                self.dot_now_alpha_zero();
                self.m1100 = 0;
                env.meter_mode = 0;
                self.send_button(env);
            }
        }
        1
    }

    pub fn finish_proc(&mut self, env: &mut Msg1Env, stick: &Msg1Stick) -> i32 {
        let opt = self.msg_data.select_flag != SELECT_OFF;
        if (stick.trig_a
            || stick.trig_b
            || opt
            || env.message_send
            || (self.msg_no == MSG1_MSG_NO_TACT_INPUT && env.mesg_cancel_button != 0))
            && !env.mesg_bgm
        {
            self.msg_data.select_off();
            self.status = STATUS_BOX_CLOSING;
            self.dot_now_alpha_zero();
            self.m1100 = 0;
            env.meter_mode = 0;
            self.send_button(env);
        } else {
            self.dot_or_idle(env, 4);
        }
        1
    }

    pub fn open_talk_proc<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        len: &MsgLenState<'_>,
        env: &mut Msg1Env,
    ) -> i32 {
        if self.m1100 == MSG1_FRAME_OPEN_TALK {
            self.status = STATUS_TYPING;
            self.message_pane_show();
            self.yose_select(font, len);
            if self.mesg_entry.initial_sound != 0 {
                env.se = Some(Msg1Se::Message(self.mesg_entry.initial_sound));
            }
            self.set_string();
            env.meter_mode = 2;
        } else {
            self.frame_open_talk();
        }
        1
    }

    pub fn open_item_proc<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        len: &MsgLenState<'_>,
        env: &mut Msg1Env,
    ) -> i32 {
        if self.m1100 == MSG1_FRAME_OPEN_ITEM {
            self.status = STATUS_TYPING;
            self.message_pane_show();
            self.yose_select(font, len);
            if self.mesg_entry.initial_sound != 0 {
                env.se = Some(Msg1Se::Message(self.mesg_entry.initial_sound));
            }
            self.set_string();
            self.item_particles = match self.msg_no {
                0x74 | 0xBE => {
                    if self.msg_no == 0xBE {
                        2
                    } else {
                        1
                    }
                }
                0x9A => 1,
                _ => 0,
            };
            env.meter_mode = 2;
        } else {
            self.frame_open_item();
        }
        1
    }

    pub fn open_tact_proc<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        len: &MsgLenState<'_>,
        layout: &MsgLayoutState<'_>,
        env: &mut Msg1Env,
    ) -> i32 {
        if self.msg_no == MSG1_MSG_NO_TACT_INPUT {
            self.status = STATUS_INPUT;
        } else if self.msg_no == MSG1_MSG_NO_TACT_DEMO_1 || self.msg_no == MSG1_MSG_NO_TACT_DEMO_2 {
            let u_var2 = env.tact.melody_num;
            if u_var2 != MSG1_TACT_SINE_1 && u_var2 != MSG1_TACT_SINE_5 {
                self.tact_guide_show(env, env.tact.beat);
                self.status = STATUS_DEMO;
                self.m116b = 0;
            } else {
                self.tact_guide_show(env, env.tact.beat);
                self.message_show(font, len, layout, env);
                self.status = STATUS_INPUT;
                self.m1168 = env.tact.beat;
                return 1;
            }
        } else {
            self.status = STATUS_TACT;
        }
        pane_scale_xy(&mut self.m049c, 1.0f32);
        self.m049c.set_init_alpha();
        if self.status != STATUS_DEMO {
            self.message_pane_show();
            self.yose_select(font, len);
            if self.mesg_entry.initial_sound != 0 {
                env.se = Some(Msg1Se::Message(self.mesg_entry.initial_sound));
            }
            self.msg_data.short_cut();
            self.msg_data.string_set(font, layout);
            self.icon_setup_loop(true);
            self.sync_out_from_data();
            self.set_string();
        }
        self.m1100 = 0;
        1
    }

    pub fn open_demo_proc<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        len: &MsgLenState<'_>,
        env: &mut Msg1Env,
    ) -> i32 {
        self.status = STATUS_TYPING;
        self.yose_select(font, len);
        if self.mesg_entry.initial_sound != 0 {
            env.se = Some(Msg1Se::Message(self.mesg_entry.initial_sound));
        }
        self.set_string();
        self.message_pane_show();
        1
    }

    pub fn close_proc(&mut self, env: &mut Msg1Env) -> i32 {
        self.frame_close(env);
        1
    }

    pub fn close_proc2(&mut self, env: &mut Msg1Env) -> i32 {
        self.status = STATUS_BOX_CLOSED;
        self.message_pane_hide(env);
        self.out_font_hide();
        1
    }

    pub fn init_proc(&mut self, clt1: &[u8], hio: &Msg1Hio, env: &mut Msg1Env) -> i32 {
        if env.camera_attention || env.demo_msg_flag {
            let base = self.mesg_offset;
            let mut b = [0u8; 6];
            for (i, slot) in b.iter_mut().enumerate() {
                *slot = self.message.get(base.wrapping_add(i)).copied().unwrap_or(0);
            }
            if b[0] == MSG1_CAMERA_TAG_MARK && b[2] == MSG1_CAMERA_TAG_CAMERA {
                let tag = ((b[3] as u16) << 8) | (b[4] as u16);
                env.mesg_camera_tag_info = tag as i32;
                self.mesg_offset = base.wrapping_add(b[1] as usize);
                self.mesg_camera_tag_info = env.mesg_camera_tag_info;
            } else if b[0] == MSG1_CAMERA_TAG_MARK
                && b[2] == MSG1_CAMERA_TAG_COLOR
                && b[3] == 0
                && b[4] == 0
            {
                if b[5] <= MSG1_COLOR_INDEX_MAX {
                    self.m1164 = fop_msgm_get_color_table(clt1, b[5] as u16);
                    let tag = format!("\u{1b}CC[{:08x}]\u{1b}GM[0]", self.m1164);
                    self.output_text.push_str(&tag);
                }
                self.mesg_offset = (base as i32).wrapping_add(b[1] as i8 as i32) as usize;
            } else {
                match self.mesg_entry.textbox_type {
                    5 => self.screen_data_value_init_demo(hio),
                    1 | 9 => self.screen_data_value_init_item(hio, env),
                    0xE => self.screen_data_value_init_tact(hio),
                    _ => self.screen_data_value_init_talk(hio, env),
                }
                self.status = STATUS_BOX_OPENING;
                env.se = Some(Msg1Se::TalkWinOpen);
            }
        }
        1
    }

    pub fn tact_proc(&mut self, env: &mut Msg1Env) -> i32 {
        if env.message_send {
            self.msg_data.auto_send_flag = 1;
            self.msg_data.wait_timer = MSG1_WAIT_TIMER_SEND;
            self.status = STATUS_CLOSE_WAIT;
        }
        1
    }

    pub fn outnow_proc<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        layout: &MsgLayoutState<'_>,
        hio: &Msg1Hio,
        env: &mut Msg1Env,
        stick: &Msg1Stick,
    ) -> i32 {
        if self.mesg_camera_tag_info != env.mesg_camera_tag_info
            && (env.camera_attention || env.demo_msg_flag)
        {
            self.mesg_out_pos(hio, env);
            self.m10d8 = self.m049c.pos_center[0] as i32;
            self.m10dc = self.m049c.pos_center[1] as i32;
            self.m049c.set_init_alpha();
            self.mesg_camera_tag_info = env.mesg_camera_tag_info;
        }
        if self.mesg_entry.draw_type == 0 {
            if (stick.trig_a || stick.trig_b) && !env.mesg_bgm {
                self.msg_data.short_cut();
                if self.msg_data.auto_send_flag == 0
                    && self.msg_data.hand_send_flag == 0
                    && self.msg_data.wait_timer != 0
                {
                    self.msg_data.wait_timer = 0;
                    self.msg_data.field_0x296 = 0;
                }
            }
        } else if self.mesg_entry.draw_type == 1 && self.msg_data.field_0x296 == 0 {
            self.msg_data.short_cut();
        }
        if self.msg_data.auto_send_flag == 0
            && self.msg_data.hand_send_flag == 0
            && self.msg_data.wait_timer != 0
        {
            self.msg_data.dec_wait_timer();
        } else {
            self.msg_data.string_set(font, layout);
            self.m1164 = self.msg_data.string_color;
            self.status = self.msg_data.mesg_status;
            self.icon_setup_loop(false);
            if self.status == STATUS_STOP {
                self.m1100 = 0;
            } else if self.status == STATUS_CLOSE_WAIT && self.msg_data.auto_send_flag == 0 {
                self.m1100 = 0;
            }
        }
        self.sync_out_from_data();
        self.set_string();
        1
    }

    fn execute_move_text(&mut self, hio: &Msg1Hio, d_var15: f32, tact_pairing: bool) {
        let dy66 = hio.field_0x66 as i32 as f32;
        let offset_y = self.text_offset_y as f32;
        let y0 = dy66 + (d_var15 + ((self.m10fc - hio.field_0x62 as i32) as f32 - offset_y));
        let y1 = dy66 + (d_var15 + ((self.m10fc - hio.field_0x60 as i32) as f32 - offset_y));
        self.text_move(0, self.m10f8 as f32, y0);
        self.text_move(1, self.m10f8 as f32, y1);
        if tact_pairing {
            self.text_move(
                2,
                (self.m10f8 + 2) as f32,
                dy66
                    + (d_var15
                        + (self.m10fc as f32
                            - (hio.field_0x62 as i32 as f32 + 2.0f32 - offset_y))),
            );
        } else {
            self.text_move(
                2,
                (self.m10f8 + 2) as f32,
                dy66
                    + (d_var15
                        + (((self.m10fc - hio.field_0x62 as i32) + 2) as f32 - offset_y)),
            );
        }
        self.text_move(
            3,
            (self.m10f8 + 2) as f32,
            dy66
                + (d_var15 + (((self.m10fc - hio.field_0x60 as i32) + 2) as f32 - offset_y)),
        );
    }

    fn execute_frame_bob(&mut self, hio: &Msg1Hio, dot: bool, corner: bool) {
        if corner {
            self.frame_white = hio.field_0x5;
            self.frame_black = hio.field_0x9;
            self.m049c.pane.white = hio.field_0x5;
        }
        let dy66 = hio.field_0x66 as i32 as f32;
        self.m049c
            .pane
            .move_to(self.m049c.pos_top_left[0], self.m049c.pos_top_left[1] + dy66);
        if dot {
            self.m050c
                .pane
                .move_to(self.m050c.pos_top_left[0], self.m050c.pos_top_left[1] + dy66);
        }
    }

    pub fn execute<F: MsgArchive + ?Sized>(
        &mut self,
        ctx: &mut Msg1Ctx<'_, F>,
        info: &mut MsgGameInfo,
    ) -> bool {
        let hio = *ctx.hio;
        let stick = ctx.stick;
        let fonts: &FontBank = ctx.fonts;
        let len = *ctx.len;
        let layout = *ctx.layout;
        let archive: &F = ctx.archive;
        let clt1: &[u8] = ctx.layout.color_clt1;
        let env: &mut Msg1Env = ctx.env;

        env.se = None;
        env.message_send = false;

        if env.mesg_send_button != 0 {
            env.mesg_send_button = 0;
        }
        if env.mesg_bgm
            && (self.mesg_entry.textbox_type != 9 || !MSG1_ITEM_BGM.contains(&env.sub_bgm))
        {
            env.mesg_bgm = false;
        }

        let user_area = self.m026c[0].user_area;
        if user_area == 3
            || (self.status != STATUS_SELECT_2
                && self.status != STATUS_SELECT_3
                && user_area != 0)
        {
            if (self.m026c[9].user_area as i32) < (hio.field_0x48 as i32 + hio.field_0x76 as i32) {
                self.sub_text_close(&hio);
            } else {
                self.m026c[9].user_area = 0;
                self.m026c[0].user_area = 0;
            }
        }

        match self.status {
            STATUS_PREPARING => {
                self.init_proc(clt1, &hio, env);
            }
            STATUS_TYPING => {
                self.outnow_proc(fonts, &layout, &hio, env, &stick);
            }
            STATUS_STOP => {
                self.stop_proc(fonts, &len, &hio, env, &stick);
            }
            STATUS_SELECT_2 | STATUS_SELECT_3 | STATUS_SELECT_YOKO => {
                self.select_proc(&hio, env, &stick);
            }
            STATUS_INPUT => {
                self.input_proc(env, &stick);
            }
            STATUS_DEMO => {
                self.demo_proc(fonts, &len, &layout, env);
            }
            STATUS_CLOSE_WAIT => {
                self.closewait_proc(env, &stick);
            }
            STATUS_BOX_OPENING => match self.mesg_entry.textbox_type {
                5 => {
                    self.open_demo_proc(fonts, &len, env);
                }
                1 | 9 => {
                    self.open_item_proc(fonts, &len, env);
                }
                0xE => {
                    self.open_tact_proc(fonts, &len, &layout, env);
                }
                _ => {
                    self.open_talk_proc(fonts, &len, env);
                }
            },
            STATUS_MSG_CONTINUES => {
                self.continue_proc(archive, fonts, &hio, env, &stick, &len);
            }
            STATUS_MSG_ENDS => {
                self.finish_proc(env, &stick);
            }
            STATUS_BOX_CLOSING => {
                if self.mesg_entry.textbox_type == 5 {
                    self.close_proc2(env);
                } else {
                    self.close_proc(env);
                }
            }
            STATUS_MSG_DESTROYED => {
                self.delete(env, info);
            }
            STATUS_TACT => {
                self.tact_proc(env);
            }
            _ => {}
        }

        let textbox_type = self.mesg_entry.textbox_type;
        if textbox_type == 9 || textbox_type == 1 {
            if self.m0624[8].now_alpha == self.m0624[8].init_alpha {
                self.ring_move();
                self.light_move();
                self.corner_move();
            }
        } else if textbox_type == 0xE {
            self.corner_move();
        }

        self.ruby_set(env.opt_ruby);

        let d_var15 = if env.opt_ruby {
            MSG1_RUBY_OFF_DY
        } else {
            0.0f32
        };
        if textbox_type == 5 {
            for i in 0..4 {
                let x = self.text_bounds(i)[0] as i32 as f32;
                let y = self.demo_pos_y.get(i).copied().unwrap_or(0) as f32;
                self.text_move(i, x, y);
            }
        } else if textbox_type == 9 || textbox_type == 1 {
            self.execute_frame_bob(&hio, true, false);
            self.execute_move_text(&hio, d_var15, false);
        } else if textbox_type == 0xE {
            self.execute_frame_bob(&hio, false, false);
            self.execute_move_text(&hio, d_var15, true);
        } else {
            self.execute_frame_bob(&hio, true, true);
            self.execute_move_text(&hio, d_var15, false);
        }
        env.mesg_status = self.status;
        info.mesg_status = self.status;
        self.alive
    }

    pub fn draw(&mut self, env: &Msg1Env) {
        let textbox_type = self.mesg_entry.textbox_type;
        if textbox_type != 5 {
            self.m049c.set_alpha();
            if textbox_type != 0xE {
                self.m_pane_arrow.set_alpha();
                self.m050c.set_alpha();
            }
        }
        for i in 0..4 {
            if let Some(p) = self.m0544.get_mut(i) {
                p.set_alpha();
            }
        }
        if textbox_type == 9 || textbox_type == 1 {
            for p in self.m0624.iter_mut() {
                p.set_alpha();
            }
        } else if textbox_type == 0xE
            && (self.msg_no == MSG1_MSG_NO_TACT_DEMO_1 || self.msg_no == MSG1_MSG_NO_TACT_DEMO_2)
        {
            for i in 0..env.tact.beat_num() {
                if let Some(p) = self.m08c4.get_mut(i) {
                    p.set_alpha();
                }
                if let Some(p) = self.m0a14.get_mut(i) {
                    p.set_alpha();
                }
                if let Some(p) = self.m0b64.get_mut(i) {
                    p.set_alpha();
                }
                if env.tact.melody_num >= 5 {
                    if let Some(p) = self.m0cb4.get_mut(i) {
                        p.set_alpha();
                    }
                }
            }
        } else if textbox_type != 5 {
            for p in self.m026c.iter_mut() {
                p.set_alpha();
            }
            self.m01fc.set_alpha();
            self.m0234.set_alpha();
            for p in self.m011c.iter_mut() {
                p.set_alpha();
            }
        }
    }

    pub fn out_font_draw(&mut self, hio: &Msg1Hio, out_hio: &MeterMsgHio) {
        let line_space = hio.field_0x5e as i32;
        let bounds = self.text_bounds(0);
        let pane_alpha = self.m0544[0].state.now_alpha;
        for i in 0..8 {
            let icon_num = self.icon_num(i);
            if icon_num == ICON_NONE
                || icon_num == ICON_SELECT_YOKO_LEFT
                || icon_num == ICON_SELECT_YOKO_RIGHT
                || icon_num == ICON_INPUT
            {
                continue;
            }
            let pos_x = self.icon_pos_x(i);
            let pos_y = self.icon_pos_y(i);
            let scale = self.icon_scale(i);
            let r5 = pos_x + bounds[0] as i32;
            let r6 = if self.mesg_entry.textbox_type == 5 {
                let local_30 = (line_space * pos_y) as f32;
                let local_38 = (self.m1104 as f32) * 2i32.saturating_sub(self.m1108) as f32;
                (local_38 + bounds[1] + local_30) as i32
            } else if self.mesg_entry.textbox_type == 0xE {
                let local_28 = (line_space * pos_y) as f32;
                (self.m1104 as f32 + bounds[1] + local_28) as i32
            } else if scale > self.m110c {
                if self.m1108 > 1 {
                    let temp1 = (self.m1104 * (3i32.saturating_sub(pos_y) - (self.m1108 + -2))) as f32;
                    ((scale - self.m110c) / 2) + ((temp1 + bounds[1]) - ((scale / 2) as f32)) as i32
                } else {
                    (((self.m1104 << 2) as f32 + bounds[1]) - ((scale / 2) as f32)) as i32
                }
            } else {
                let local_28 = (line_space * pos_y) as f32;
                let local_30 = (self.m1104 as f32) * 3i32.saturating_sub(self.m1108) as f32;
                (local_30 + bounds[1] + local_28) as i32
            };
            if let (Some(icon), Some(kage), Some(timer)) = (
                self.button_icon.get_mut(i),
                self.button_kage.get_mut(i),
                self.button_timer.get_mut(i),
            ) {
                fop_msgm_out_font_draw(
                    out_hio, icon, kage, r5, r6, scale, timer, pane_alpha, icon_num,
                );
            }
        }
    }

    pub fn draw_msg(&mut self, hio: &Msg1Hio, out_hio: &MeterMsgHio) {
        self.out_font_draw(hio, out_hio);
        if self.status == STATUS_SELECT_YOKO {
            self.arrow_rot
                .set(self.arrow_size[0] / 2.0f32, self.arrow_size[1] / 2.0f32, 90.0f32);
            self.msg_data
                .select_arrow(self.arrow_size[0], self.arrow_size[1], 0.0f32, 0.0f32);
        } else if self.status == STATUS_INPUT
            && (self.mesg_entry.textbox_type == 0 || self.mesg_entry.textbox_type == 10)
        {
            self.number_cell_w = self.number_pane[0].get_font_size()[0]
                + self.number_pane[0].get_char_space();
            let columns = if self.msg_no == MSG1_MSG_NO_NUMBER_3 {
                3
            } else {
                2
            };
            self.input_mask = [
                (self.m10ec as f32
                    + self.number_cell_w * ((columns - 1 - self.select_num as i32) as f32))
                    as i32,
                self.m10f0,
                self.mask_size[0] as i32,
                self.mask_size[1] as i32,
            ];
        }
    }

    pub fn draw_tact(&self) -> [i32; 4] {
        [
            (0.0f32 * MSG1_TACT_SCISSOR_X_MUL + MSG1_TACT_SCISSOR_X_ADD) as i32,
            (0.0f32 * MSG1_TACT_SCISSOR_Y_MUL + MSG1_TACT_SCISSOR_Y_ADD) as i32,
            (MSG1_TACT_SCISSOR_W_NUM * MSG1_TACT_SCISSOR_X_MUL) as i32,
            (MSG1_TACT_SCISSOR_H_NUM * MSG1_TACT_SCISSOR_Y_MUL) as i32,
        ]
    }

    pub fn create<F: MsgArchive + ?Sized>(
        &mut self,
        ctx: &mut Msg1Ctx<'_, F>,
        info: &mut MsgGameInfo,
    ) -> Msg1Phase {
        let archive: &F = ctx.archive;
        let screen: Msg1Screen = *ctx.screen;
        let hio: Msg1Hio = *ctx.hio;
        let mx_font = ctx.mx_font;
        let rx_font = ctx.rx_font;
        let env: &mut Msg1Env = ctx.env;
        if env.heap_lock_flag != 0 && env.heap_lock_flag != MSG1_HEAP_LOCK {
            return Msg1Phase::Init;
        }
        self.heap_locked = true;
        env.heap_lock_flag = MSG1_HEAP_LOCK;

        let msg_no = if (MSG1_ITEM_FIRST..=MSG1_ITEM_LAST).contains(&self.msg_no) {
            env.triforce_num + MSG1_ITEM_FIRST
        } else {
            self.msg_no
        };
        let mut get = MsgGet::default();
        let Some(head) = get.get_mesg_header(archive, env.clear_count, msg_no) else {
            return Msg1Phase::Init;
        };
        let Some(message) = get.get_message(head) else {
            return Msg1Phase::Init;
        };
        self.message = message.to_vec();
        self.mesg_offset = 0;
        if let Some(entry) = get.get_mesg_entry(head) {
            self.mesg_entry = entry;
        }
        self.msg_get = get;
        self.msg_id = self.msg_get.get_mesg_number();
        self.mesg_camera_tag_info = env.mesg_camera_tag_info;

        self.screen_data_set(&screen, &hio, env, mx_font, rx_font);
        for p in self.m0544.iter_mut() {
            p.text.binding_h = screen.text_binding_h;
            p.text.binding_v = screen.text_binding_v;
        }

        for i in 0..8 {
            if let Some(icon) = self.button_icon.get_mut(i) {
                icon.visible = false;
                icon.set_alpha(0);
                icon.tex = Some(MSG1_BUTTON_TEX);
            }
            if let Some(kage) = self.button_kage.get_mut(i) {
                kage.visible = false;
                kage.set_alpha(0);
                kage.tex = Some(MSG1_BUTTON_TEX);
            }
            if let Some(timer) = self.button_timer.get_mut(i) {
                *timer = -1;
            }
        }

        self.status = STATUS_PREPARING;
        self.m1164 = u32::MAX;
        self.value_init();
        self.set_string();
        self.m116a = 0;
        env.mesg_send_button = self.m116a;
        env.mesg_cancel_button = 0;
        env.mesg_status = self.status;
        info.mesg_status = self.status;
        self.alive = true;
        Msg1Phase::Complete
    }

    pub fn delete(&mut self, env: &mut Msg1Env, info: &mut MsgGameInfo) -> bool {
        if env.demo_msg_flag {
            env.demo_msg_flag = false;
        }
        if env.tact_msg_flag {
            env.tact_msg_flag = false;
        }
        if env.next_msg_flag {
            env.next_msg_flag = false;
        }
        self.m116a = 0;
        env.mesg_send_button = self.m116a;
        env.mesg_cancel_button = 0;
        env.mesg_status = 0;
        info.mesg_status = 0;
        if env.mesg_bgm {
            env.mesg_bgm = false;
        }
        self.heap_locked = false;
        env.heap_lock_flag = 0;
        self.alive = false;
        true
    }

    pub fn is_delete(&self) -> bool {
        true
    }
}