use bevy_ecs::prelude::*;

use super::font::{CharQuad, FontBank};
use super::msgdata::MsgLenState;
use super::msglayout::MsgLayoutState;
use super::msgget::{MsgArchive, MsgGet};
use super::msgproc::{
    MsgGameInfo, MeterMsgHio, MsgIconPic, fop_msgm_out_font_draw, fop_msgm_out_font_set,
    fop_msgm_value_increase,
};
use super::{
    FontSlot, ICON_ARROW_DOWN, ICON_ARROW_LEFT, ICON_ARROW_RIGHT, ICON_ARROW_UP,
    ICON_FLASHING_A_BUTTON_2, ICON_NONE, ICON_SELECT_YOKO_RIGHT, MesgEntry, Metrics, MsgData,
    STATUS_BOX_CLOSED, STATUS_BOX_OPENING, STATUS_CLOSE_WAIT, STATUS_MSG_DESTROYED,
    STATUS_MSG_DISPLAYED, STATUS_MSG_ENDS, STATUS_STOP, STATUS_TYPING, STATUS_UNK5,
};

pub const PANE_TYPE_ID_WINDOW: u16 = 17;
pub const PANE_TYPE_ID_TEXT_BOX: u16 = 19;
pub const TEXT_BOX_MAGIC: u32 = u32::from_be_bytes(*b"TBX1");

pub const HBIND_CENTER: u8 = 0;
pub const HBIND_RIGHT: u8 = 1;
pub const HBIND_LEFT: u8 = 2;

pub const VBIND_CENTER: u8 = 0;
pub const VBIND_BOTTOM: u8 = 1;
pub const VBIND_TOP: u8 = 2;

pub const ESC_BYTE: u8 = 0x1B;
pub const PRINT_STR_BUFF_SIZE: usize = 0x400;
pub const PRINT_LINE_SLOTS: usize = 0x100;
pub const PRINT_LINE_SPACE_DEFAULT: f32 = 32.0f32;
pub const PRINT_TAB_STOP_NULL: f32 = 80.0f32;

const CTRL_BACKSPACE: i32 = 8;
const CTRL_TAB: i32 = 9;
const CTRL_NEWLINE: i32 = 10;
const CTRL_RETURN: i32 = 0x0D;
const CTRL_CURSOR_RIGHT: i32 = 0x1C;
const CTRL_CURSOR_LEFT: i32 = 0x1D;
const CTRL_CURSOR_UP: i32 = 0x1E;
const CTRL_CURSOR_DOWN: i32 = 0x1F;

pub const ESC_CU: u16 = 0x4355;
pub const ESC_CD: u16 = 0x4344;
pub const ESC_CL: u16 = 0x434C;
pub const ESC_CR: u16 = 0x4352;
pub const ESC_LU: u16 = 0x4C55;
pub const ESC_LD: u16 = 0x4C44;
pub const ESC_ST: u16 = 0x5354;
pub const ESC_CC: u16 = 0x4343;
pub const ESC_GC: u16 = 0x4743;
pub const ESC_FX: u16 = 0x4658;
pub const ESC_FY: u16 = 0x4659;
pub const ESC_SH: u16 = 0x5348;
pub const ESC_SV: u16 = 0x5356;
pub const ESC_GM: u16 = 0x474D;
pub const ESC_HM: u16 = 0x484D;

pub const MSG2_PANES: usize = 3;
pub const MSG2_BUTTONS: usize = 8;
pub const MSG2_LINE_WIDTH: i32 = 503;
pub const MSG2_CENTER_LINE_WIDTH: i32 = 486;
pub const MSG2_PANE_HEIGHT_BONUS: f32 = 10.0f32;
pub const MSG2_ARROW_PERIOD: i32 = 12;
pub const MSG2_ARROW_CENTER: i32 = 6;
pub const MSG2_ARROW_BOB: f32 = 3.0f32;
pub const MSG2_RUBY_OFF_DY: f32 = -4.0f32;
pub const MSG2_SDW_DX: i32 = 2;
pub const MSG2_SDW_DY: i32 = 2;
pub const MSG2_SCROLL_STEP: i32 = 12;
pub const MSG2_SCROLL_WRAP: i32 = 2040;
pub const MSG2_SCROLL_LIMIT: i32 = 1020;
pub const MSG2_CAPTURE_W: i32 = 576;
pub const MSG2_CAPTURE_H: i32 = 240;
pub const MSG2_CAPTURE_CENTER_X: f32 = 288.0f32;
pub const MSG2_CAPTURE_CENTER_Y: f32 = 120.0f32;
pub const MSG2_SHADE_LOW: i32 = 58;
pub const MSG2_SHADE_HIGH: i32 = 187;
pub const MSG2_SHADE_TABLE_MAX: i32 = 239;
pub const MSG2_STICK_THRESHOLD: f32 = 0.7f32;
pub const MSG2_MSG_NO_33A: u32 = 0x33A;
pub const MSG2_MSG_NO_33B: u32 = 0x33B;
pub const MSG2_HEAP_LOCK: u8 = 8;
pub const MSG2_BOARD_BASE_TEX: &str = "hukidashi_0212.bti";
pub const MSG2_BOARD_TEX_TYPE_2: &str = "hukidashi_02.bti";
pub const MSG2_BOARD_TEX_TYPE_6: &str = "hukidashi_06.bti";
pub const MSG2_BOARD_TEX_TYPE_7: &str = "hukidashi_07.bti";

const MSG2_COLOR_TABLE: [u32; 9] = [
    0x0000_0000,
    0xB400_0000,
    0x8282_8200,
    0x0000_AA00,
    0xF0F0_1E00,
    0x82FF_FF00,
    0x6400_FF00,
    0x5050_5000,
    0xFFB4_0000,
];

const MSG2_TEX_I4_COLOR: [u8; 240] = [
    0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x02, 0x02, 0x02, 0x03, 0x04, 0x06, 0x07, 0x09, 0x0B, 0x0D,
    0x0F, 0x12, 0x16, 0x1C, 0x22, 0x2A, 0x32, 0x39, 0x41, 0x4B, 0x54, 0x5E, 0x67, 0x72, 0x7C, 0x87,
    0x8F, 0x98, 0xA2, 0xAC, 0xB3, 0xBB, 0xC2, 0xCA, 0xD0, 0xD8, 0xDD, 0xE1, 0xE7, 0xEA, 0xEE, 0xF1,
    0xF3, 0xF5, 0xF7, 0xF9, 0xFB, 0xFE, 0xFD, 0xFD, 0xFD, 0xFD, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFD, 0xFD, 0xFD,
    0xFD, 0xFB, 0xFB, 0xF9, 0xF7, 0xF5, 0xF3, 0xF1, 0xEE, 0xEA, 0xE7, 0xE1, 0xDD, 0xD8, 0xD0, 0xC9,
    0xC2, 0xBC, 0xB3, 0xAB, 0xA2, 0x99, 0x8F, 0x87, 0x7C, 0x72, 0x68, 0x5E, 0x54, 0x4A, 0x41, 0x3A,
    0x32, 0x2A, 0x22, 0x1D, 0x15, 0x12, 0x0F, 0x0D, 0x0B, 0x09, 0x07, 0x05, 0x04, 0x04, 0x03, 0x02,
];

pub trait QuadFont {
    fn qmetrics(&self, slot: usize) -> Metrics;
    fn qchar_width(&self, slot: usize, code: u16) -> i32;
    fn qis_lead_byte(&self, slot: usize, chr: i32) -> bool;
    fn qis_fixed(&self, slot: usize) -> bool;
    fn qfixed_width(&self, slot: usize) -> i32;
    fn qset_grad_color(&mut self, slot: usize, col1: u32, col2: u32);
    fn qchar_quad(
        &mut self,
        slot: usize,
        pos_x: f32,
        pos_y: f32,
        scale_x: f32,
        scale_y: f32,
        chr: u16,
        visible: bool,
    ) -> CharQuad;
}

impl QuadFont for FontBank {
    fn qmetrics(&self, slot: usize) -> Metrics {
        self.get(slot).map_or_else(Metrics::default, |font| font.metrics())
    }

    fn qchar_width(&self, slot: usize, code: u16) -> i32 {
        self.get(slot).map_or(0, |font| font.char_width(code))
    }

    fn qis_lead_byte(&self, slot: usize, chr: i32) -> bool {
        self.get(slot).is_some_and(|font| font.is_lead_byte(chr))
    }

    fn qis_fixed(&self, slot: usize) -> bool {
        self.get(slot).is_some_and(|font| font.fixed)
    }

    fn qfixed_width(&self, slot: usize) -> i32 {
        self.get(slot).map_or(0, |font| font.fixed_width)
    }

    fn qset_grad_color(&mut self, slot: usize, col1: u32, col2: u32) {
        if let Some(font) = self.get_mut(slot) {
            font.set_grad_color(col1, col2);
        }
    }

    fn qchar_quad(
        &mut self,
        slot: usize,
        pos_x: f32,
        pos_y: f32,
        scale_x: f32,
        scale_y: f32,
        chr: u16,
        visible: bool,
    ) -> CharQuad {
        let fallback = CharQuad {
            advance: 0.0f32,
            pos: [[0.0f32; 2]; 4],
            color: [0u32; 4],
            uv: [[0i32; 2]; 4],
        };
        self.get_mut(slot).map_or(fallback, |font| {
            font.char_quad(pos_x, pos_y, scale_x, scale_y, chr, visible)
        })
    }
}

fn byte_at(text: &[u8], at: usize) -> u8 {
    text.get(at).copied().unwrap_or(0)
}

fn cstr_len(text: &[u8]) -> usize {
    text.iter().position(|&b| b == 0).unwrap_or(text.len())
}

fn scale_alpha(color: u32, alpha: u8) -> u32 {
    let a = (color >> 24) & 0xFF;
    (((a * alpha as u32) / 0xFF) << 24) | (color & 0x00FF_FFFF)
}

fn strtoll(text: &[u8], at: &mut usize, base: i32) -> (i32, usize) {
    let mut i = *at;
    let mut negative = false;
    while byte_at(text, i).is_ascii_whitespace() {
        i += 1;
    }
    if byte_at(text, i) == b'-' {
        negative = true;
        i += 1;
    } else if byte_at(text, i) == b'+' {
        i += 1;
    }
    if base == 16
        && byte_at(text, i) == b'0'
        && (byte_at(text, i + 1) == b'x' || byte_at(text, i + 1) == b'X')
    {
        i += 2;
    }
    let start = i;
    let mut acc: u32 = 0;
    loop {
        let d = byte_at(text, i);
        let v = match d {
            b'0'..=b'9' => u32::from(d - b'0'),
            b'a'..=b'f' if base == 16 => u32::from(d - b'a') + 10,
            b'A'..=b'F' if base == 16 => u32::from(d - b'A') + 10,
            _ => break,
        };
        if v >= base as u32 {
            break;
        }
        acc = acc.wrapping_mul(base as u32).wrapping_add(v);
        i += 1;
    }
    if i == start {
        return (0, 0);
    }
    let consumed = i - start;
    *at = i;
    if negative {
        (acc.wrapping_neg() as i32, consumed)
    } else {
        (acc as i32, consumed)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pane {
    pub type_id: u16,
    pub bounds: [f32; 4],
    pub alpha: u8,
    pub visible: bool,
    pub connect_parent: bool,
    pub influenced_alpha: bool,
    pub parent_draw_alpha: u8,
    pub white: u32,
}

impl Default for Pane {
    fn default() -> Self {
        Self {
            type_id: PANE_TYPE_ID_WINDOW,
            bounds: [0.0f32; 4],
            alpha: 0xFF,
            visible: true,
            connect_parent: false,
            influenced_alpha: true,
            parent_draw_alpha: 0xFF,
            white: 0xFFFF_FFFF,
        }
    }
}

impl Pane {
    pub fn new(type_id: u16) -> Self {
        Self {
            type_id,
            ..Self::default()
        }
    }

    pub fn width(&self) -> f32 {
        self.bounds[2] - self.bounds[0]
    }

    pub fn height(&self) -> f32 {
        self.bounds[3] - self.bounds[1]
    }

    pub fn glb_bounds(&self) -> [f32; 4] {
        self.bounds
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        let w = self.width();
        let h = self.height();
        self.bounds[0] = x;
        self.bounds[1] = y;
        self.bounds[2] = x + w;
        self.bounds[3] = y + h;
    }

    pub fn resize(&mut self, w: f32, h: f32) {
        self.bounds[2] = self.bounds[0] + w;
        self.bounds[3] = self.bounds[1] + h;
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_alpha(&mut self, alpha: u8) {
        self.alpha = alpha;
    }

    pub fn draw_alpha(&self) -> u8 {
        if !self.influenced_alpha {
            return self.alpha;
        }
        ((self.alpha as u32 * self.parent_draw_alpha as u32) / 0xFF) as u8
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaneState {
    pub pos_top_left_orig: [f32; 2],
    pub pos_top_left: [f32; 2],
    pub pos_center_orig: [f32; 2],
    pub pos_center: [f32; 2],
    pub size_orig: [f32; 2],
    pub size: [f32; 2],
    pub init_alpha: u8,
    pub now_alpha: u8,
    pub user_area: i16,
    pub pane: Pane,
}

impl PaneState {
    pub fn set_pane_data(&mut self, pane: Pane) {
        self.pos_top_left_orig = [pane.bounds[0], pane.bounds[1]];
        self.size_orig = [pane.width(), pane.height()];
        self.pos_center_orig = [
            self.pos_top_left_orig[0] + self.size_orig[0] / 2.0f32,
            self.pos_top_left_orig[1] + self.size_orig[1] / 2.0f32,
        ];
        self.init_alpha = pane.alpha;
        self.pos_top_left = self.pos_top_left_orig;
        self.pos_center = self.pos_center_orig;
        self.size = self.size_orig;
        self.now_alpha = 0;
        self.user_area = 0;
        self.pane = pane;
    }

    pub fn pane_trans(&mut self, trans_x: f32, trans_y: f32) {
        self.pos_center = [
            self.pos_center_orig[0] + trans_x,
            self.pos_center_orig[1] + trans_y,
        ];
        self.cpos_move();
    }

    pub fn pane_scale(&mut self, scale_x: f32, scale_y: f32) {
        self.size = [self.size_orig[0] * scale_x, self.size_orig[1] * scale_y];
        self.cpos_move();
    }

    pub fn cpos_move(&mut self) {
        self.pos_top_left = [
            self.pos_center[0] - self.size[0] / 2.0f32,
            self.pos_center[1] - self.size[1] / 2.0f32,
        ];
        self.pane.move_to(self.pos_top_left[0], self.pos_top_left[1]);
        self.pane.resize(self.size[0], self.size[1]);
    }

    pub fn set_alpha(&mut self) {
        self.pane.set_alpha(self.now_alpha);
    }

    pub fn set_init_alpha(&mut self) {
        self.now_alpha = self.init_alpha;
    }

    pub fn set_now_alpha(&mut self, alpha: f32) {
        self.now_alpha = ((self.init_alpha as f32) * alpha) as u8;
    }

    pub fn set_now_alpha_zero(&mut self) {
        self.now_alpha = 0;
    }

    pub fn set_white(&mut self, white: u32) {
        self.pane.white = white;
    }

    pub fn show(&mut self) {
        self.pane.show();
    }

    pub fn hide(&mut self) {
        self.pane.hide();
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphRun {
    pub code: u16,
    pub pen: [f32; 2],
    pub scale: [f32; 2],
    pub offset: i16,
    pub quad: CharQuad,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextLayout {
    pub quads: Vec<GlyphRun>,
    pub line_offsets: Vec<u16>,
    pub size: [f32; 2],
    pub bottom: f32,
}

#[derive(Clone, Debug)]
pub struct TextPrint {
    pub font_slot: Option<usize>,
    pub str_buff_size: usize,
    pub buffer_not_enough: bool,
    pub char_space: f32,
    pub line_space: f32,
    pub tab_stop: i16,
    pub grad_mode_init: bool,
    pub font_size: [f32; 2],
    pub char_color: u32,
    pub grad_color: u32,
    pub black: u32,
    pub white: u32,
    pub char_color_now: u32,
    pub grad_color_now: u32,
    pub char_space_now: f32,
    pub line_space_now: f32,
    pub font_size_x_now: f32,
    pub font_size_y_now: f32,
    pub tab_stop_now: i16,
    pub grad_mode: bool,
    pub line_start_x: f32,
    pub field_0x28: f32,
    pub cursor_x: f32,
    pub cursor_v: f32,
    pub last_width: f32,
}

impl Default for TextPrint {
    fn default() -> Self {
        Self {
            font_slot: None,
            str_buff_size: PRINT_STR_BUFF_SIZE,
            buffer_not_enough: false,
            char_space: 0.0f32,
            line_space: PRINT_LINE_SPACE_DEFAULT,
            tab_stop: 0,
            grad_mode_init: true,
            font_size: [0.0f32; 2],
            char_color: 0xFFFF_FFFF,
            grad_color: 0xFFFF_FFFF,
            black: 0x0000_0000,
            white: 0xFFFF_FFFF,
            char_color_now: 0xFFFF_FFFF,
            grad_color_now: 0xFFFF_FFFF,
            char_space_now: 0.0f32,
            line_space_now: PRINT_LINE_SPACE_DEFAULT,
            font_size_x_now: 0.0f32,
            font_size_y_now: 0.0f32,
            tab_stop_now: 0,
            grad_mode: true,
            line_start_x: 0.0f32,
            field_0x28: 0.0f32,
            cursor_x: 0.0f32,
            cursor_v: 0.0f32,
            last_width: 0.0f32,
        }
    }
}

impl TextPrint {
    #[allow(clippy::too_many_arguments)]
    pub fn private_initiate<F: QuadFont + ?Sized>(
        &mut self,
        font: &F,
        slot: Option<usize>,
        char_space: f32,
        line_space: f32,
        char_color: u32,
        grad_color: u32,
        black: u32,
        white: u32,
        use_font_leading: bool,
    ) {
        self.font_slot = slot;
        self.char_space = char_space;
        self.line_space = PRINT_LINE_SPACE_DEFAULT;
        if let Some(s) = slot {
            let m = font.qmetrics(s);
            self.line_space = if use_font_leading {
                m.leading as f32
            } else {
                line_space
            };
            self.tab_stop = (m.width << 2) as i16;
            self.font_size = [m.cell_width as f32, m.cell_height as f32];
        } else {
            self.tab_stop = PRINT_TAB_STOP_NULL as i16;
            self.font_size = [0.0f32; 2];
        }
        self.grad_mode_init = true;
        self.locate(0.0f32, 0.0f32);
        self.char_color = char_color;
        self.grad_color = grad_color;
        self.black = black;
        self.white = white;
        self.initchar();
    }

    pub fn set_font_size(&mut self, size_x: f32, size_y: f32) {
        self.font_size[0] = if size_x > 0.0f32 { size_x } else { 0.0f32 };
        self.font_size[1] = if size_y > 0.0f32 { size_y } else { 0.0f32 };
    }

    pub fn locate(&mut self, x: f32, y: f32) {
        self.line_start_x = x;
        self.field_0x28 = y;
        self.cursor_x = x;
        self.cursor_v = y;
        self.last_width = 0.0f32;
    }

    pub fn initchar(&mut self) {
        self.char_color_now = self.char_color;
        self.grad_color_now = self.grad_color;
        self.char_space_now = self.char_space;
        self.line_space_now = self.line_space;
        self.grad_mode = self.grad_mode_init;
        self.tab_stop_now = self.tab_stop;
        self.font_size_x_now = self.font_size[0];
        self.font_size_y_now = self.font_size[1];
    }

    fn get_number_s32(text: &[u8], p: &mut usize, fallback: i32, keep: i32, base: i32) -> i32 {
        if byte_at(text, *p) != b'[' {
            return fallback;
        }
        let start = *p;
        *p += 1;
        let (value, consumed) = strtoll(text, p, base);
        if base == 16 && consumed != 8 {
            if consumed == 6 {
                return (value << 8) | 0xFF;
            }
            *p = start;
            return keep;
        }
        if byte_at(text, *p) != b']' {
            *p = start;
            return keep;
        }
        let end = *p;
        *p = end + 1;
        if end == start + 1 {
            return fallback;
        }
        value
    }

    fn get_number_f32(text: &[u8], p: &mut usize, fallback: f32, keep: f32, base: i32) -> f32 {
        Self::get_number_s32(text, p, fallback as i32, keep as i32, base) as f32
    }

    fn do_ctrl_code(&mut self, code: i32) {
        match code {
            CTRL_BACKSPACE => {
                self.cursor_x -= self.last_width;
                self.last_width = 0.0f32;
            }
            CTRL_TAB => {
                if self.tab_stop_now > 0 {
                    let step = self.tab_stop_now as i32;
                    let prev = self.cursor_x;
                    self.cursor_x = self.tab_stop_now as f32
                        + self.tab_stop_now as f32 * ((prev as i32) / step) as f32;
                    self.last_width = self.cursor_x - prev;
                }
            }
            CTRL_NEWLINE => {
                self.last_width = 0.0f32;
                self.cursor_x = self.line_start_x;
                self.cursor_v += self.line_space_now;
            }
            CTRL_RETURN => {
                self.last_width = 0.0f32;
                self.cursor_x = self.line_start_x;
            }
            CTRL_CURSOR_RIGHT => self.cursor_x += 1.0f32,
            CTRL_CURSOR_LEFT => self.cursor_x -= 1.0f32,
            CTRL_CURSOR_UP => self.cursor_v -= 1.0f32,
            CTRL_CURSOR_DOWN => self.cursor_v += 1.0f32,
            _ => {}
        }
    }

    fn do_escape_code<F: QuadFont + ?Sized>(
        &mut self,
        font: &F,
        text: &[u8],
        p: &mut usize,
    ) -> u16 {
        let slot = self.font_slot.unwrap_or(0);
        let start = *p;
        let mut code: u16 = 0;
        for _ in 0..2 {
            let cur = byte_at(text, *p) as i32;
            let value = if font.qis_lead_byte(slot, cur) {
                let hi = cur;
                let lo = byte_at(text, *p + 1) as i32;
                *p += 2;
                (hi << 8) | lo
            } else {
                *p += 1;
                cur
            };
            if !(0x20..0x80).contains(&value) {
                *p = start;
                return 0;
            }
            code = ((code << 8) | value as u16) & 0xFFFF;
        }

        match code {
            ESC_CU => {
                let n = Self::get_number_f32(text, p, 1.0f32, 0.0f32, 10);
                self.cursor_v -= n;
            }
            ESC_CD => {
                let n = Self::get_number_f32(text, p, 1.0f32, 0.0f32, 10);
                self.cursor_v += n;
            }
            ESC_CL => {
                let n = Self::get_number_f32(text, p, 1.0f32, 0.0f32, 10);
                self.cursor_x -= n;
            }
            ESC_CR => {
                let n = Self::get_number_f32(text, p, 1.0f32, 0.0f32, 10);
                self.cursor_x += n;
            }
            ESC_LU => self.cursor_v -= self.line_space_now,
            ESC_LD => self.cursor_v += self.line_space_now,
            ESC_ST => {
                let val = Self::get_number_s32(
                    text,
                    p,
                    self.tab_stop_now as i32,
                    self.tab_stop_now as i32,
                    10,
                );
                if val > 0 {
                    self.tab_stop_now = val as i16;
                }
            }
            ESC_CC => {
                self.char_color_now = Self::get_number_s32(
                    text,
                    p,
                    self.char_color as i32,
                    self.char_color_now as i32,
                    16,
                ) as u32;
            }
            ESC_GC => {
                self.grad_color_now = Self::get_number_s32(
                    text,
                    p,
                    self.grad_color as i32,
                    self.grad_color_now as i32,
                    16,
                ) as u32;
            }
            ESC_FX => {
                let n = Self::get_number_f32(text, p, self.font_size[0], self.font_size_x_now, 10);
                if n >= 0.0f32 {
                    self.font_size_x_now = n;
                }
            }
            ESC_FY => {
                let n = Self::get_number_f32(text, p, self.font_size[1], self.font_size_y_now, 10);
                if n >= 0.0f32 {
                    self.font_size_y_now = n;
                }
            }
            ESC_SH => {
                self.char_space_now =
                    Self::get_number_f32(text, p, self.char_space, self.char_space_now, 10);
            }
            ESC_SV => {
                self.line_space_now =
                    Self::get_number_f32(text, p, self.line_space, self.line_space_now, 10);
            }
            ESC_GM => {
                let fallback = i32::from(!self.grad_mode);
                let keep = i32::from(self.grad_mode);
                let val = Self::get_number_s32(text, p, fallback, keep, 10);
                self.grad_mode = val != 0;
            }
            ESC_HM => {}
            _ => {
                *p = start;
                return 0;
            }
        }
        code
    }

    #[allow(clippy::too_many_arguments)]
    fn parse<F: QuadFont + ?Sized>(
        &mut self,
        font: &mut F,
        text: &[u8],
        text_len: i32,
        max_width: i32,
        offsets: &mut Vec<u16>,
        size: &mut [f32; 2],
        alpha: u8,
        draw: bool,
        quads: &mut Vec<GlyphRun>,
    ) -> f32 {
        let Some(slot) = self.font_slot else {
            return 0.0f32;
        };
        let m = font.qmetrics(slot);

        let mut p = 0usize;
        let mut r29 = 0usize;
        let f30 = self.cursor_x;
        let f28 = self.cursor_v;
        let mut f31 = 0.0f32;
        let mut local_a8 = 0.0f32;
        let mut chr = byte_at(text, p) as i32;
        p += 1;
        let mut f29 = self.cursor_x;
        let mut local_ac = self.cursor_x;
        let mut local_b0 = self.cursor_v;
        let mut local_b4 = self.cursor_v;

        loop {
            let mut local_f0 = false;
            if font.qis_lead_byte(slot, chr) {
                chr = (chr << 8) | byte_at(text, p) as i32;
                p += 1;
                local_f0 = true;
            }

            let mut r25 = true;
            let local_c0 = self.cursor_x;

            if chr == 0 || (p as i32) > text_len {
                if !draw {
                    offsets.push((0.5f32 + f31) as u16);
                }
                r29 += 1;
                break;
            } else if chr < 0x20 {
                if chr == ESC_BYTE as i32 {
                    let code = self.do_escape_code(font, text, &mut p);
                    if code == ESC_HM {
                        if !draw {
                            offsets.push((0.5f32 + f31) as u16);
                        }
                        self.cursor_x = f30;
                        self.cursor_v = f28;
                        r29 += 1;
                        if r29 == PRINT_LINE_SLOTS {
                            break;
                        }
                        f31 = 0.0f32;
                    }
                    if code != 0 {
                        r25 = false;
                    }
                } else {
                    self.do_ctrl_code(chr);
                    r25 = false;
                    if chr == CTRL_NEWLINE {
                        if !draw {
                            offsets.push((0.5f32 + f31) as u16);
                        }
                        r29 += 1;
                        if r29 == PRINT_LINE_SLOTS {
                            break;
                        }
                        f31 = 0.0f32;
                    }
                }
            } else if local_f0 && (p as i32) > text_len {
                if !draw {
                    offsets.push((0.5f32 + f31) as u16);
                }
                r29 += 1;
                break;
            } else {
                self.last_width = if font.qis_fixed(slot) {
                    font.qfixed_width(slot) as f32
                } else {
                    font.qchar_width(slot, chr as u16) as f32
                };
                self.last_width *= self.font_size_x_now / m.cell_width as f32;
                let raw = (self.cursor_x + self.last_width) - self.line_start_x;
                let scaled = 10000.0f32 * raw;
                let truncated = ((scaled as i32) as f32) / 10000.0f32;
                if truncated > max_width as f32 && self.cursor_x > f30 {
                    p = p.wrapping_sub(if local_f0 { 2 } else { 1 });
                    self.cursor_v += self.line_space_now;
                    if !draw {
                        offsets.push((0.5f32 + f31) as u16);
                    }
                    r29 += 1;
                    if r29 == PRINT_LINE_SLOTS {
                        break;
                    }
                    self.cursor_x = self.line_start_x;
                    f31 = 0.0f32;
                    r25 = false;
                } else {
                    if draw {
                        let offset = offsets.get(r29).copied().unwrap_or(0);
                        let gx = self.cursor_x + offset as i16 as f32;
                        let gy = self.cursor_v;
                        let sx = self.font_size_x_now;
                        let sy = self.font_size_y_now;
                        let c0 = scale_alpha(self.char_color_now, alpha);
                        let c1 = scale_alpha(self.grad_color_now, alpha);
                        let sel = if self.grad_mode { c1 } else { c0 };
                        font.qset_grad_color(slot, c0, sel);
                        quads.push(GlyphRun {
                            code: chr as u16,
                            pen: [gx, gy],
                            scale: [sx, sy],
                            offset: offset as i16,
                            quad: font.qchar_quad(slot, gx, gy, sx, sy, chr as u16, true),
                        });
                    }
                    self.cursor_x += self.last_width;
                }
            }

            if r25 {
                if self.cursor_x - f30 > f31 {
                    f31 = self.cursor_x - f30;
                }
                self.cursor_x += self.char_space_now;
                self.last_width += self.char_space_now;
                let local_cc =
                    (self.font_size_y_now / (m.ascent + m.descent) as f32) * m.descent as f32;
                if local_a8 < self.cursor_v + local_cc {
                    local_a8 = self.cursor_v + local_cc;
                }
                if self.cursor_x > local_ac {
                    local_ac = self.cursor_x;
                }
                if self.cursor_x < f29 {
                    f29 = self.cursor_x;
                }
                if local_c0 < f29 {
                    f29 = local_c0;
                }
                if self.cursor_v > local_b4 {
                    local_b4 = self.cursor_v;
                }
                if self.cursor_v < local_b0 {
                    local_b0 = self.cursor_v;
                }
            }
            chr = byte_at(text, p) as i32;
            p += 1;
        }

        if offsets.last() != Some(&0xFFFF) {
            offsets.push(0xFFFF);
        }
        size[0] = local_ac - f29;
        size[1] = local_b4 - local_b0 + m.leading as f32;
        if !draw {
            self.cursor_x = f30;
            self.cursor_v = f28;
        }
        local_a8
    }

    #[allow(clippy::too_many_arguments)]
    pub fn print_return<F: QuadFont + ?Sized>(
        &mut self,
        font: &mut F,
        text: &[u8],
        width: f32,
        height: f32,
        hbind: u8,
        vbind: u8,
        shift_x: f32,
        mut shift_y: f32,
        alpha: u8,
    ) -> TextLayout {
        let mut layout = TextLayout::default();
        let Some(slot) = self.font_slot else {
            return layout;
        };

        self.initchar();
        self.line_start_x = (self.cursor_x as i32) as f32;
        self.field_0x28 = (self.cursor_v as i32) as f32;

        let mut len = cstr_len(text);
        if len >= self.str_buff_size {
            len = self.str_buff_size - 1;
            self.buffer_not_enough = true;
        }
        let len = len as i32;

        let mut size = [0.0f32; 2];
        let mut offsets: Vec<u16> = Vec::with_capacity(PRINT_LINE_SLOTS + 1);
        let bottom = self.parse(
            font,
            text,
            len,
            width as i32,
            &mut offsets,
            &mut size,
            alpha,
            false,
            &mut layout.quads,
        );

        let m = font.qmetrics(slot);
        let d_var12 = m.ascent as f32 * (self.font_size[1] / m.cell_height as f32);
        let d_var13 = bottom + d_var12;
        match vbind {
            VBIND_TOP => {}
            VBIND_BOTTOM => shift_y += height - ((d_var13 + 0.5f32) as i32) as f32,
            VBIND_CENTER => shift_y += (height - ((d_var13 + 0.5f32) as i32) as f32) / 2.0f32,
            _ => {}
        }

        let mut i = 0usize;
        while offsets.get(i).copied() != Some(0xFFFF) {
            let v = offsets.get(i).copied().unwrap_or(0) as f32;
            let nv = match hbind {
                HBIND_LEFT => 0.0f32,
                HBIND_RIGHT => width - v,
                HBIND_CENTER => (width - v) * 0.5f32,
                _ => v,
            };
            if let Some(slot_ref) = offsets.get_mut(i) {
                *slot_ref = nv as u16;
            }
            i += 1;
        }

        self.initchar();
        self.cursor_x += shift_x;
        self.cursor_v += shift_y + d_var12;
        self.line_start_x = (self.cursor_x as i32) as f32;
        self.field_0x28 = (self.cursor_v as i32) as f32;
        let _ = self.parse(
            font,
            text,
            len,
            width as i32,
            &mut offsets,
            &mut size,
            alpha,
            true,
            &mut layout.quads,
        );

        layout.line_offsets = offsets;
        layout.size = size;
        layout.bottom = bottom;
        layout
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextBox {
    pub font: Option<usize>,
    pub char_color: u32,
    pub grad_color: u32,
    pub shift: [f32; 2],
    pub char_space: f32,
    pub line_space: f32,
    pub font_size: [f32; 2],
    pub string: Vec<u8>,
    pub white: u32,
    pub black: u32,
    pub binding_h: u8,
    pub binding_v: u8,
    pub connect_parent: bool,
    pub text_font_owned: bool,
    pub magic: u32,
}

impl TextBox {
    pub fn initiate<F: QuadFont + ?Sized>(
        &mut self,
        font: &F,
        font_slot: Option<usize>,
        string: &[u8],
        binding_h: u8,
        binding_v: u8,
    ) {
        self.font = font_slot;
        self.char_color = 0xFFFF_FFFF;
        self.grad_color = 0xFFFF_FFFF;
        self.black = 0x0000_0000;
        self.white = 0xFFFF_FFFF;
        self.binding_h = binding_h;
        self.binding_v = binding_v;
        self.set_string(string);
        self.shift = [0.0f32; 2];
        self.char_space = 0.0f32;
        match font_slot {
            None => {
                self.line_space = 0.0f32;
                self.font_size = [0.0f32; 2];
            }
            Some(slot) => {
                let m = font.qmetrics(slot);
                self.line_space = m.leading as f32;
                self.font_size = [m.width as f32, m.ascent as f32 + m.descent as f32];
            }
        }
        self.magic = TEXT_BOX_MAGIC;
        self.text_font_owned = true;
    }

    pub fn set_font(&mut self, font_slot: usize) {
        self.font = Some(font_slot);
        self.text_font_owned = false;
    }

    pub fn font_slot(&self) -> usize {
        self.font.unwrap_or(0)
    }

    pub fn set_font_size(&mut self, size_x: f32, size_y: f32) {
        self.font_size[0] = if size_x > 0.0f32 { size_x } else { 0.0f32 };
        self.font_size[1] = if size_y > 0.0f32 { size_y } else { 0.0f32 };
    }

    pub fn shift_set(&mut self, x: f32, y: f32) {
        self.shift = [x, y];
    }

    pub fn set_string(&mut self, string: &[u8]) -> i32 {
        let mut next = string.to_vec();
        if next.last() != Some(&0) {
            next.push(0);
        }
        self.string = next;
        (self.string.len() - 1) as i32
    }

    pub fn get_string(&self) -> &[u8] {
        &self.string
    }

    pub fn set_char_color(&mut self, color: u32) {
        self.char_color = color;
    }

    pub fn set_grad_color(&mut self, color: u32) {
        self.grad_color = color;
    }

    pub fn set_font_color(&mut self, char_color: u32, grad_color: u32) {
        self.char_color = char_color;
        self.grad_color = grad_color;
    }

    pub fn set_white(&mut self, white: u32) {
        self.white = white;
    }

    pub fn set_black(&mut self, black: u32) {
        self.black = black;
    }

    pub fn set_line_space(&mut self, space: f32) {
        self.line_space = space;
    }

    pub fn set_char_space(&mut self, space: f32) {
        self.char_space = space;
    }

    pub fn get_line_space(&self) -> f32 {
        self.line_space
    }

    pub fn get_char_space(&self) -> f32 {
        self.char_space
    }

    pub fn get_font_size(&self) -> [f32; 2] {
        self.font_size
    }

    pub fn set_connect_parent(&mut self, parent_type_id: Option<u16>, value: bool) -> bool {
        let Some(parent) = parent_type_id else {
            return false;
        };
        if parent != PANE_TYPE_ID_WINDOW {
            return false;
        }
        self.connect_parent = value;
        value
    }

    pub fn resize(
        &mut self,
        parent: Option<&mut Pane>,
        current: [f32; 2],
        w: f32,
        h: f32,
    ) -> [f32; 2] {
        if self.connect_parent
            && let Some(parent) = parent
            && parent.type_id == PANE_TYPE_ID_WINDOW
        {
            return [
                parent.width() + (w - (current[2] - current[0])),
                parent.height() + (h - (current[3] - current[1])),
            ];
        }
        [w, h]
    }

    pub fn make_print<F: QuadFont + ?Sized>(&self, font: &F) -> TextPrint {
        let mut print = TextPrint::default();
        print.private_initiate(
            font,
            self.font,
            self.char_space,
            self.line_space,
            self.char_color,
            self.grad_color,
            self.black,
            self.white,
            false,
        );
        print.set_font_size(self.font_size[0], self.font_size[1]);
        print
    }

    pub fn draw_self<F: QuadFont + ?Sized>(
        &self,
        font: &mut F,
        bounds: [f32; 4],
        x: f32,
        y: f32,
        alpha: u8,
    ) -> TextLayout {
        let mut print = self.make_print(font);
        print.locate(x, y);
        print.print_return(
            font,
            self.get_string(),
            (bounds[2] - bounds[0]) + 0.0001f32,
            bounds[3] - bounds[1],
            self.binding_h,
            self.binding_v,
            self.shift[0],
            self.shift[1],
            alpha,
        )
    }

    pub fn draw<F: QuadFont + ?Sized>(
        &self,
        font: &mut F,
        visible: bool,
        width: f32,
        hbind: u8,
        alpha: u8,
    ) -> TextLayout {
        if !visible {
            return TextLayout::default();
        }
        let mut print = self.make_print(font);
        print.print_return(
            font,
            self.get_string(),
            width,
            0.0f32,
            hbind,
            VBIND_TOP,
            0.0f32,
            -self.font_size[1],
            alpha,
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextPane {
    pub state: PaneState,
    pub text: TextBox,
}

impl TextPane {
    pub fn set_pane_data(&mut self, pane: Pane) {
        self.state.set_pane_data(pane);
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.state.pane.move_to(x, y);
    }

    pub fn resize(&mut self, w: f32, h: f32) {
        self.state.pane.resize(w, h);
    }

    pub fn cpos_move(&mut self) {
        self.state.cpos_move();
    }

    pub fn pane_trans(&mut self, trans_x: f32, trans_y: f32) {
        self.state.pane_trans(trans_x, trans_y);
    }

    pub fn set_alpha(&mut self) {
        self.state.set_alpha();
    }

    pub fn set_init_alpha(&mut self) {
        self.state.set_init_alpha();
    }

    pub fn set_now_alpha_zero(&mut self) {
        self.state.set_now_alpha_zero();
    }

    pub fn show(&mut self) {
        self.state.pane.show();
    }

    pub fn hide(&mut self) {
        self.state.pane.hide();
    }

    pub fn is_visible(&self) -> bool {
        self.state.pane.is_visible()
    }

    pub fn draw_alpha(&self) -> u8 {
        self.state.pane.draw_alpha()
    }

    pub fn draw_self<F: QuadFont + ?Sized>(&self, font: &mut F, x: f32, y: f32) -> TextLayout {
        let alpha = self.draw_alpha();
        self.text
            .draw_self(font, self.state.pane.glb_bounds(), x, y, alpha)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaneSetup {
    pub bounds: [f32; 4],
    pub alpha: u8,
    pub visible: bool,
}

impl Default for PaneSetup {
    fn default() -> Self {
        Self {
            bounds: [0.0f32; 4],
            alpha: 0xFF,
            visible: true,
        }
    }
}

impl PaneSetup {
    pub fn pane(&self, type_id: u16) -> Pane {
        Pane {
            type_id,
            bounds: self.bounds,
            alpha: self.alpha,
            visible: self.visible,
            ..Pane::default()
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Msg2Screen {
    pub text: [PaneSetup; MSG2_PANES],
    pub ruby: [PaneSetup; MSG2_PANES],
    pub text_sdw: [PaneSetup; MSG2_PANES],
    pub ruby_sdw: [PaneSetup; MSG2_PANES],
    pub board: [PaneSetup; MSG2_PANES],
    pub board_sdw: [PaneSetup; MSG2_PANES],
    pub text_binding_h: u8,
    pub text_binding_v: u8,
    pub arrow_up: Option<PaneSetup>,
    pub arrow_up_kage: Option<PaneSetup>,
    pub arrow_down: Option<PaneSetup>,
    pub arrow_down_kage: Option<PaneSetup>,
    pub dot_top: Option<PaneSetup>,
    pub dot_bottom: Option<PaneSetup>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Msg2Stick {
    pub pad0_y: f32,
    pub pad3_y: f32,
    pub trig_a: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Msg2Se {
    Scroll1,
    TalkWinClose,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Msg2Env {
    pub opt_ruby: bool,
    pub event_bit_2d80: bool,
    pub dif_col: [u8; 3],
    pub meter_mode: u8,
    pub heap_lock_flag: u8,
    pub brightness: u8,
    pub se: Option<Msg2Se>,
}

impl Msg2Env {
    pub fn kankyo_brightness(&self) -> u8 {
        ((self.dif_col[0] as f32) * 0.299f32
            + (self.dif_col[1] as f32) * 0.587f32
            + (self.dif_col[2] as f32) * 0.114f32) as u8
    }
}

pub use super::msghio::MsgHio as Msg2Hio;

impl Msg2Hio {
    pub fn aim_brightness(&self, env: &Msg2Env) -> u8 {
        let brightness = env.kankyo_brightness() as u32;
        if self.field_0x29 as u32 <= brightness {
            0xFF
        } else {
            (0xFFu32 - (brightness - self.field_0x29 as u32)) as u8
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Msg2Board {
    pub pos: [i16; 4],
    pub black: [u8; 4],
    pub white: [u8; 4],
    pub alpha: f32,
    pub scroll: [[i32; 3]; 2],
}

impl Msg2Board {
    pub fn init(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.pos = [x1 as i16, y1 as i16, x2 as i16, y2 as i16];
    }

    pub fn set_pos(&mut self, x1: i16, y1: i16, x2: i16, y2: i16) {
        self.pos = [x1, y1, x2, y2];
    }

    pub fn set_black_color(&mut self, color: [u8; 4]) {
        self.black = color;
    }

    pub fn set_white_color(&mut self, color: [u8; 4]) {
        self.white = color;
    }

    pub fn set_alpha(&mut self, alpha: f32) {
        self.alpha = alpha;
    }

    pub fn set_scroll(&mut self, index: usize, a: i32, b: i32) {
        if let Some(slot) = self.scroll.get_mut(index) {
            slot[0] = a;
            slot[1] = b;
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Msg2Phase {
    #[default]
    Init,
    Complete,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Msg2DrawOutput {
    pub board: Msg2Board,
    pub text: [TextLayout; MSG2_PANES],
    pub ruby: [TextLayout; MSG2_PANES],
    pub text_sdw: [TextLayout; MSG2_PANES],
    pub ruby_sdw: [TextLayout; MSG2_PANES],
}

pub struct Msg2Ctx<'a, F: MsgArchive + ?Sized> {
    pub hio: &'a Msg2Hio,
    pub out_hio: &'a MeterMsgHio,
    pub env: &'a mut Msg2Env,
    pub archive: &'a F,
    pub fonts: &'a FontBank,
    pub mx_font: usize,
    pub rx_font: usize,
    pub screen: &'a Msg2Screen,
    pub len: &'a MsgLenState<'a>,
    pub layout: &'a MsgLayoutState<'a>,
    pub stick: Msg2Stick,
}



#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Msg2Box {
    pub msg_no: u32,
    pub status: u8,
    pub mesg_entry: MesgEntry,
    pub mesg_number: u32,
    pub color_no: u16,
    pub message: Vec<u8>,
    pub msg_get: MsgGet,
    pub msg_data: [MsgData; MSG2_PANES],
    pub output_text: [String; MSG2_PANES],
    pub output_ruby: [String; MSG2_PANES],
    pub output_text_sdw: [String; MSG2_PANES],
    pub output_ruby_sdw: [String; MSG2_PANES],
    pub text_pane: [TextPane; MSG2_PANES],
    pub text_sdw_pane: [TextPane; MSG2_PANES],
    pub ruby_pane: [TextPane; MSG2_PANES],
    pub ruby_sdw_pane: [TextPane; MSG2_PANES],
    pub arrow_up: PaneState,
    pub arrow_up_kage: PaneState,
    pub arrow_down: PaneState,
    pub arrow_down_kage: PaneState,
    pub dot_top: PaneState,
    pub dot_bottom: PaneState,
    pub board_pane: [PaneState; MSG2_PANES],
    pub board_sdw_pane: [PaneState; MSG2_PANES],
    pub bbutton_icon: [[MsgIconPic; MSG2_PANES]; MSG2_BUTTONS],
    pub bbutton_kage: [[MsgIconPic; MSG2_PANES]; MSG2_BUTTONS],
    pub bbutton_timer: [[i16; MSG2_PANES]; MSG2_BUTTONS],
    pub mx: usize,
    pub rx: usize,
    pub mx_height: i32,
    pub rx_height: i32,
    pub tex_base: &'static str,
    pub tex_variant: &'static str,
    pub board: Msg2Board,
    pub pane_height: f32,
    pub open_t: i32,
    pub pane_above: u8,
    pub pane_center: u8,
    pub pane_below: u8,
    pub half_line_space: i32,
    pub font_size: i32,
    pub ruby_font_size: i32,
    pub arrow_timer: i32,
    pub scroll_phase: u8,
    pub scroll_line: i32,
    pub scroll_offset: i32,
    pub line_shift: [i32; MSG2_PANES],
    pub aim_alpha: f32,
    pub box_alpha: f32,
    pub stick_state: u8,
    pub shade: u8,
    pub char_alpha: [[u8; MSG2_PANES]; 4],
    pub pop_speed: i32,
    pub heap_locked: bool,
    pub alive: bool,
}

impl Default for Msg2Box {
    fn default() -> Self {
        Self {
            msg_no: 0,
            status: 0,
            mesg_entry: MesgEntry::default(),
            mesg_number: 0,
            color_no: 0,
            message: Vec::new(),
            msg_get: MsgGet::default(),
            msg_data: std::array::from_fn(|_| MsgData::new_data_proc()),
            output_text: Default::default(),
            output_ruby: Default::default(),
            output_text_sdw: Default::default(),
            output_ruby_sdw: Default::default(),
            text_pane: Default::default(),
            text_sdw_pane: Default::default(),
            ruby_pane: Default::default(),
            ruby_sdw_pane: Default::default(),
            arrow_up: PaneState::default(),
            arrow_up_kage: PaneState::default(),
            arrow_down: PaneState::default(),
            arrow_down_kage: PaneState::default(),
            dot_top: PaneState::default(),
            dot_bottom: PaneState::default(),
            board_pane: Default::default(),
            board_sdw_pane: Default::default(),
            bbutton_icon: Default::default(),
            bbutton_kage: Default::default(),
            bbutton_timer: [[-1; MSG2_PANES]; MSG2_BUTTONS],
            mx: 0,
            rx: 0,
            mx_height: 0,
            rx_height: 0,
            tex_base: MSG2_BOARD_BASE_TEX,
            tex_variant: MSG2_BOARD_TEX_TYPE_2,
            board: Msg2Board::default(),
            pane_height: 0.0f32,
            open_t: 0,
            pane_above: 2,
            pane_center: 0,
            pane_below: 1,
            half_line_space: 0,
            font_size: 0,
            ruby_font_size: 0,
            arrow_timer: 0,
            scroll_phase: 0,
            scroll_line: 0,
            scroll_offset: 0,
            line_shift: [0; MSG2_PANES],
            aim_alpha: 0.0f32,
            box_alpha: 0.0f32,
            stick_state: 0,
            shade: 0,
            char_alpha: [[0; MSG2_PANES]; 4],
            pop_speed: 0,
            heap_locked: false,
            alive: false,
        }
    }
}

impl Msg2Box {
    pub fn value_init(&mut self, index: usize) {
        let color = MSG2_COLOR_TABLE
            .get(self.color_no as usize)
            .copied()
            .unwrap_or(0);
        let (a, b, c, d) = match self.msg_data.get(index) {
            Some(d) => (
                d.field_0x290 as u32,
                d.field_0x291 as u32,
                d.field_0x292 as u32,
                d.field_0x293 as u32,
            ),
            None => (0, 0, 0, 0),
        };
        let temp_a = color | a;
        let temp_b = color | b;
        let temp_c = color | c;
        let temp_d = color | d;
        if let Some(s) = self.output_text.get_mut(index) {
            *s = format!("\u{1b}CC[{temp_a:08x}]\u{1b}GC[{temp_b:08x}]");
        }
        if let Some(s) = self.output_ruby.get_mut(index) {
            *s = format!("\u{1b}CC[{temp_c:08x}]\u{1b}GC[{temp_d:08x}]");
        }
        if let Some(s) = self.output_text_sdw.get_mut(index) {
            *s = format!("\u{1b}CC[{a:08x}]\u{1b}GC[{b:08x}]");
        }
        if let Some(s) = self.output_ruby_sdw.get_mut(index) {
            *s = format!("\u{1b}CC[{c:08x}]\u{1b}GC[{d:08x}]");
        }
    }

    pub fn set_string(&mut self, index: usize) {
        let text = self.output_text.get(index).cloned().unwrap_or_default();
        let ruby = self.output_ruby.get(index).cloned().unwrap_or_default();
        let text_sdw = self.output_text_sdw.get(index).cloned().unwrap_or_default();
        let ruby_sdw = self.output_ruby_sdw.get(index).cloned().unwrap_or_default();
        if let Some(p) = self.text_pane.get_mut(index) {
            p.text.set_string(text.as_bytes());
        }
        if let Some(p) = self.ruby_pane.get_mut(index) {
            p.text.set_string(ruby.as_bytes());
        }
        if let Some(p) = self.text_sdw_pane.get_mut(index) {
            p.text.set_string(text_sdw.as_bytes());
        }
        if let Some(p) = self.ruby_sdw_pane.get_mut(index) {
            p.text.set_string(ruby_sdw.as_bytes());
        }
    }

    pub fn sync_out_from_data(&mut self, index: usize) {
        let out = match self.msg_data.get(index) {
            Some(d) => d.out.clone(),
            None => return,
        };
        if let Some(slot) = self.output_text.get_mut(index) {
            *slot = out.first().cloned().flatten().unwrap_or_default();
        }
        if let Some(slot) = self.output_ruby.get_mut(index) {
            *slot = out.get(1).cloned().flatten().unwrap_or_default();
        }
        if let Some(slot) = self.output_text_sdw.get_mut(index) {
            *slot = out.get(2).cloned().flatten().unwrap_or_default();
        }
        if let Some(slot) = self.output_ruby_sdw.get_mut(index) {
            *slot = out.get(3).cloned().flatten().unwrap_or_default();
        }
    }

    pub fn message_pane_show(&mut self, index: usize) {
        if let Some(p) = self.text_pane.get_mut(index) {
            p.show();
        }
        if let Some(p) = self.ruby_pane.get_mut(index) {
            p.show();
        }
    }

    pub fn message_pane_hide(&mut self, index: usize) {
        if let Some(p) = self.text_pane.get_mut(index) {
            p.hide();
        }
        if let Some(p) = self.ruby_pane.get_mut(index) {
            p.hide();
        }
    }

    pub fn out_font_hide(&mut self, index: usize) {
        for i in 0..MSG2_BUTTONS {
            if let Some(icon) = self.bbutton_icon.get_mut(i).and_then(|row| row.get_mut(index)) {
                icon.visible = false;
                icon.rotate(0.0f32);
            }
            if let Some(kage) = self.bbutton_kage.get_mut(i).and_then(|row| row.get_mut(index)) {
                kage.visible = false;
                kage.rotate(0.0f32);
            }
            if let Some(timer) = self.bbutton_timer.get_mut(i).and_then(|row| row.get_mut(index)) {
                *timer = -1;
            }
        }
    }

    pub fn arrow_up_show(&mut self) {
        self.arrow_up.set_init_alpha();
        self.arrow_up_kage.set_init_alpha();
    }

    pub fn arrow_up_hide(&mut self) {
        self.arrow_up.set_now_alpha_zero();
        self.arrow_up_kage.set_now_alpha_zero();
    }

    pub fn arrow_down_show(&mut self) {
        self.arrow_down.set_init_alpha();
        self.arrow_down_kage.set_init_alpha();
    }

    pub fn arrow_down_hide(&mut self) {
        self.arrow_down.set_now_alpha_zero();
        self.arrow_down_kage.set_now_alpha_zero();
    }

    pub fn dot_show(&mut self) {
        self.dot_top.set_init_alpha();
        self.dot_bottom.set_init_alpha();
    }

    pub fn dot_hide(&mut self) {
        self.dot_top.set_now_alpha_zero();
        self.dot_bottom.set_now_alpha_zero();
    }

    pub fn multi_tex_init(&mut self) {
        self.tex_base = MSG2_BOARD_BASE_TEX;
        self.tex_variant = match self.mesg_entry.textbox_type {
            6 => MSG2_BOARD_TEX_TYPE_6,
            7 => MSG2_BOARD_TEX_TYPE_7,
            _ => MSG2_BOARD_TEX_TYPE_2,
        };
        self.board.init(1.0f32, 1.0f32, 1.0f32, 1.0f32);
    }

    pub fn fontdata_init(&mut self, mx: usize, rx: usize) {
        self.mx = mx;
        self.rx = rx;
    }

    pub fn screen_data_set(&mut self, index: usize) {
        self.value_init(index);
        self.set_string(index);
    }

    pub fn screen_data_init(
        &mut self,
        index: usize,
        screen: &Msg2Screen,
        hio: &Msg2Hio,
        out_hio: &MsgMeterHio,
        fonts: &FontBank,
    ) {
        if let Some(p) = self.text_pane.get_mut(index) {
            let setup = screen.text.get(index).copied().unwrap_or_default();
            p.set_pane_data(setup.pane(PANE_TYPE_ID_TEXT_BOX));
        }
        if let Some(p) = self.ruby_pane.get_mut(index) {
            let setup = screen.ruby.get(index).copied().unwrap_or_default();
            p.set_pane_data(setup.pane(PANE_TYPE_ID_TEXT_BOX));
        }
        if let Some(p) = self.text_sdw_pane.get_mut(index) {
            let setup = screen.text_sdw.get(index).copied().unwrap_or_default();
            p.set_pane_data(setup.pane(PANE_TYPE_ID_TEXT_BOX));
        }
        if let Some(p) = self.ruby_sdw_pane.get_mut(index) {
            let setup = screen.ruby_sdw.get(index).copied().unwrap_or_default();
            p.set_pane_data(setup.pane(PANE_TYPE_ID_TEXT_BOX));
        }

        if let Some(p) = self.text_sdw_pane.get_mut(index) {
            p.hide();
        }
        if let Some(p) = self.ruby_sdw_pane.get_mut(index) {
            p.hide();
        }

        if let Some(p) = self.text_pane.get_mut(index) {
            p.state.size_orig[1] += MSG2_PANE_HEIGHT_BONUS;
            p.state.size[1] = p.state.size_orig[1];
            p.cpos_move();
        }
        if let Some(p) = self.text_sdw_pane.get_mut(index) {
            p.state.size_orig[1] += MSG2_PANE_HEIGHT_BONUS;
            p.state.size[1] = p.state.size_orig[1];
            p.cpos_move();
        }

        if let Some(p) = self.board_pane.get_mut(index) {
            let setup = screen.board.get(index).copied().unwrap_or_default();
            p.set_pane_data(setup.pane(PANE_TYPE_ID_WINDOW));
            p.set_white(0);
        }
        if let Some(p) = self.board_sdw_pane.get_mut(index) {
            let setup = screen.board_sdw.get(index).copied().unwrap_or_default();
            p.set_pane_data(setup.pane(PANE_TYPE_ID_WINDOW));
            p.set_white(0);
        }

        if index == 0 {
            let setups = [
                screen.arrow_up,
                screen.arrow_down,
                screen.dot_top,
                screen.arrow_up_kage,
                screen.arrow_down_kage,
                screen.dot_bottom,
            ];
            let panes: [Pane; 6] =
                setups.map(|setup| setup.unwrap_or_default().pane(PANE_TYPE_ID_WINDOW));
            self.arrow_up.set_pane_data(panes[0]);
            self.arrow_down.set_pane_data(panes[1]);
            self.dot_top.set_pane_data(panes[2]);
            self.arrow_up_kage.set_pane_data(panes[3]);
            self.arrow_down_kage.set_pane_data(panes[4]);
            self.dot_bottom.set_pane_data(panes[5]);
            self.arrow_up_kage.set_white(0x0000_00FF);
            self.arrow_down_kage.set_white(0x0000_00FF);
            self.dot_bottom.set_white(0x0000_00FF);
        } else {
            for pane in [
                &mut self.arrow_up,
                &mut self.arrow_down,
                &mut self.dot_top,
                &mut self.arrow_up_kage,
                &mut self.arrow_down_kage,
                &mut self.dot_bottom,
            ] {
                pane.pane.hide();
            }
        }

        let mx = self.mx;
        let rx = self.rx;
        let bind_h = screen.text_binding_h;
        let bind_v = screen.text_binding_v;
        for pane in [
            self.text_pane.get_mut(index),
            self.ruby_pane.get_mut(index),
            self.text_sdw_pane.get_mut(index),
            self.ruby_sdw_pane.get_mut(index),
        ]
        .into_iter()
        .flatten()
        {
            pane.text.binding_h = bind_h;
            pane.text.binding_v = bind_v;
            pane.text.set_char_space(0.0f32);
        }
        if let Some(p) = self.text_pane.get_mut(index) {
            p.text.set_font(mx);
            p.text.set_line_space(out_hio.field_0x5e as f32);
        }
        if let Some(p) = self.text_sdw_pane.get_mut(index) {
            p.text.set_font(mx);
            p.text.set_line_space(out_hio.field_0x5e as f32);
        }
        if let Some(p) = self.ruby_pane.get_mut(index) {
            p.text.set_font(rx);
        }
        if let Some(p) = self.ruby_sdw_pane.get_mut(index) {
            p.text.set_font(rx);
        }

        let size_x = out_hio.field_0x70 as f32;
        if let Some(p) = self.text_pane.get_mut(index) {
            p.text.set_font_size(size_x, size_x);
        }
        if let Some(p) = self.text_sdw_pane.get_mut(index) {
            p.text.set_font_size(size_x, size_x);
        }

        self.font_size = out_hio.field_0x70 as i32;
        self.ruby_font_size = 0;
        self.mx_height = fonts.get(mx).map_or(0, |f| f.height());
        self.rx_height = fonts.get(rx).map_or(0, |f| f.height());
    }

    pub fn screen_data_value_init(&mut self, hio: &Msg2Hio) {
        let line_space = self
            .text_pane
            .first()
            .map(|p| p.text.get_line_space())
            .unwrap_or(0.0f32);
        self.half_line_space = (line_space / 2.0f32) as i32;
        let height = self
            .text_pane
            .first()
            .map(|p| p.state.pane.height())
            .unwrap_or(0.0f32);
        self.pane_height = height + hio.field_0x3c as f32;

        let pane_height = self.pane_height;
        if let Some(p) = self.text_pane.get_mut(0) {
            p.pane_trans(0.0f32, 0.0f32);
        }
        if let Some(p) = self.text_pane.get_mut(1) {
            p.pane_trans(0.0f32, pane_height);
        }
        if let Some(p) = self.text_pane.get_mut(2) {
            p.pane_trans(0.0f32, -pane_height);
        }

        self.arrow_up.set_now_alpha_zero();
        self.arrow_up_kage.set_now_alpha_zero();
        self.arrow_down.set_now_alpha_zero();
        self.arrow_down_kage.set_now_alpha_zero();
        self.dot_top.set_now_alpha_zero();
        self.dot_bottom.set_now_alpha_zero();

        for i in 0..MSG2_PANES {
            self.message_data_init(i);
        }
    }

    pub fn stick_info_init(&mut self, stick: &Msg2Stick) {
        if stick.pad0_y > MSG2_STICK_THRESHOLD || stick.pad3_y > MSG2_STICK_THRESHOLD {
            self.stick_state = 1;
        } else if stick.pad0_y < MSG2_STICK_THRESHOLD || stick.pad3_y < MSG2_STICK_THRESHOLD {
            self.stick_state = 2;
        } else {
            self.stick_state = 0;
        }
    }

    pub fn stick_info_check(&mut self, stick: &Msg2Stick) {
        if self.stick_state == 1 {
            if stick.pad0_y <= 0.0f32 || stick.pad3_y <= 0.0f32 {
                self.stick_state = 0;
            }
        } else if self.stick_state == 2 && (stick.pad0_y >= 0.0f32 || stick.pad3_y >= 0.0f32) {
            self.stick_state = 0;
        }
    }

    pub fn multi_tex_draw(&mut self, hio: &Msg2Hio, x1: i16, y1: i16, w: i16, h: i16) {
        let r6 = x1.wrapping_add(w);
        let r7 = y1.wrapping_add(h);
        let var_f31 = (self.box_alpha * hio.field_0x5b as f32) / 255.0f32;
        self.board.set_pos(x1, y1, r6, r7);
        if self.mesg_entry.textbox_type == 7 {
            self.board.set_black_color([0xFF, 0xFF, 0xFF, 0xFF]);
            self.board.set_white_color([0x00, 0x00, 0x00, 0x00]);
        } else {
            self.board.set_black_color([0xF0, 0xF0, 0xF0, 0xDC]);
            self.board.set_white_color([0x78, 0x78, 0x8C, 0x00]);
        }
        self.board.set_alpha(var_f31);
        self.board.set_scroll(0, 0, 0);
        let offset = self.scroll_offset;
        self.board.set_scroll(1, 0, offset);
    }

    pub fn message_data_init(&mut self, index: usize) {
        let line_width = MSG2_LINE_WIDTH;
        let center_line_width = MSG2_CENTER_LINE_WIDTH;
        let char_space = self
            .text_pane
            .get(index)
            .map(|p| p.text.get_char_space() as i32)
            .unwrap_or(0);
        let ruby_space = self
            .ruby_pane
            .get(index)
            .map(|p| p.text.get_char_space() as i32)
            .unwrap_or(0);
        let line_space = self
            .text_pane
            .get(index)
            .map(|p| p.text.get_line_space() as i32)
            .unwrap_or(0);

        let out = [
            self.output_text.get(index).cloned().flatten(),
            self.output_ruby.get(index).cloned().flatten(),
            self.output_text_sdw.get(index).cloned().flatten(),
            self.output_ruby_sdw.get(index).cloned().flatten(),
        ];

        let message = self.message.clone();
        let entry = self.mesg_entry;
        let mx = self.mx;
        let rx = self.rx;
        let font_size = self.font_size;
        let ruby_font_size = self.ruby_font_size;

        let Some(data) = self.msg_data.get_mut(index) else {
            return;
        };
        data.data_init();
        data.set_bmg(message);
        data.out = out;
        data.font = mx;
        data.ruby_font = rx;
        data.char_space = char_space;
        data.ruby_char_space = ruby_space;
        data.line_space = line_space;
        data.entry = entry;
        data.font_size = font_size;
        data.ruby_font_size = ruby_font_size;
        data.line_width = line_width;
        data.center_line_width = center_line_width;
        data.space_flag = 0;
    }

    pub fn yose_select<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        index: usize,
        len: &MsgLenState,
    ) {
        let line_count = match self.msg_data.get_mut(index) {
            Some(data) => {
                data.count = data.string_length(font, *len) as i32;
                data.line_count
            }
            None => return,
        };
        if let Some(slot) = self.line_shift.get_mut(index) {
            *slot = line_count;
        }
        if let Some(data) = self.msg_data.get_mut(index) {
            data.line_count = 0;
            data.string_shift();
        }
        self.text_position(index);
    }

    pub fn text_position(&mut self, index: usize) {
        let shift_y = self.half_line_space.wrapping_mul(
            3i32.wrapping_sub(self.line_shift.get(index).copied().unwrap_or(0)),
        ) as f32;
        for pane in [
            self.text_pane.get_mut(index),
            self.ruby_pane.get_mut(index),
            self.text_sdw_pane.get_mut(index),
            self.ruby_sdw_pane.get_mut(index),
        ]
        .into_iter()
        .flatten()
        {
            pane.text.shift_set(0.0f32, shift_y);
        }
    }

    pub fn message_out<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        index: usize,
        aim_line: i32,
        len: &MsgLenState,
        layout: &MsgLayoutState,
        hio: &Msg2Hio,
    ) {
        if let Some(slot) = self.line_shift.get_mut(index) {
            *slot = 0;
        }
        self.message_pane_show(index);
        self.message_data_init(index);

        if let Some(data) = self.msg_data.get_mut(index) {
            data.count = 0;
            data.now_line = 0;
            data.aim_line = aim_line;
        }

        self.yose_select(font, index, len);

        if let Some(data) = self.msg_data.get_mut(index) {
            data.short_cut();
        }
        self.set_char_alpha(index, hio);
        if let Some(data) = self.msg_data.get_mut(index) {
            data.string_set(font, layout);
        }

        for i in 0..MSG2_BUTTONS {
            let (icon_num, icon_color) = match self.msg_data.get(index) {
                Some(data) => (
                    data.icon_num.get(i).copied().unwrap_or(ICON_NONE),
                    data.icon_color.get(i).copied().unwrap_or(0),
                ),
                None => (ICON_NONE, 0),
            };
            let timer = self
                .bbutton_timer
                .get(i)
                .and_then(|row| row.get(index))
                .copied()
                .unwrap_or(-1);
            if icon_num == ICON_NONE || timer != -1 {
                continue;
            }
            let mut icon_color = icon_color;
            if icon_color == 0
                && matches!(
                    icon_num,
                    ICON_ARROW_LEFT
                        | ICON_ARROW_RIGHT
                        | ICON_ARROW_UP
                        | ICON_ARROW_DOWN
                        | ICON_SELECT_YOKO_RIGHT
                        | ICON_FLASHING_A_BUTTON_2
                )
            {
                icon_color = 0xFFFF_FFFF;
            }
            let icon = self.bbutton_icon.get_mut(i).and_then(|row| row.get_mut(index));
            let kage = self.bbutton_kage.get_mut(i).and_then(|row| row.get_mut(index));
            let timer = self
                .bbutton_timer
                .get_mut(i)
                .and_then(|row| row.get_mut(index));
            if let (Some(icon), Some(kage), Some(timer)) = (icon, kage, timer) {
                fop_msgm_out_font_set(icon, kage, timer, icon_color, icon_num);
            }
        }

        self.sync_out_from_data(index);
        self.set_string(index);
    }

    pub fn message_show<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        len: &MsgLenState,
        layout: &MsgLayoutState,
        hio: &Msg2Hio,
    ) {
        for i in 0..MSG2_PANES {
            self.screen_data_set(i);
        }

        if self.scroll_line > 0 {
            let above = self.pane_above;
            let extra = i32::from(self.mesg_entry.field_0x16);
            let target = self.scroll_line.wrapping_sub(extra);
            self.message_out(font, above, target, len, layout, hio);
        }

        let center = self.pane_center;
        let line = self.scroll_line;
        self.message_out(font, center, line, len, layout, hio);

        let stop = self
            .msg_data
            .get(center)
            .is_some_and(|d| d.mesg_status == STATUS_STOP);
        if stop {
            let below = self.pane_below;
            let extra = i32::from(self.mesg_entry.field_0x16);
            self.message_out(font, below, line.wrapping_add(extra), len, layout, hio);
        }
    }

    pub fn aim_alpha_square(&mut self, hio: &Msg2Hio, env: &Msg2Env, max: i32, value: i32) {
        let v = if value < 0 {
            0
        } else if value > max {
            max
        } else {
            value
        };
        let f31 = ((v as f32) * (v as f32)) / ((max as f32) * (max as f32));
        let f0 = 255.0f32 - hio.aim_brightness(env) as f32;
        self.aim_alpha = 255.0f32 - f0 * f31;
    }

    pub fn aim_alpha_sqrt(&mut self, hio: &Msg2Hio, env: &Msg2Env, max: i32, value: i32) {
        let v = if value < 0 {
            0
        } else if value > max {
            max
        } else {
            value
        };
        let f31 = ((v as f32) / (max as f32)).sqrt();
        let f0 = 255.0f32 - hio.aim_brightness(env) as f32;
        self.aim_alpha = 255.0f32 - f0 * f31;
    }

    pub fn arrow_move(&mut self) {
        self.arrow_timer += 1;
        if self.arrow_timer > MSG2_ARROW_PERIOD {
            self.arrow_timer -= MSG2_ARROW_PERIOD;
        }
        let bounce = (MSG2_ARROW_CENTER - self.arrow_timer).abs() as f32;
        let down = self.arrow_down.pos_top_left_orig;
        let up = self.arrow_up.pos_top_left_orig;
        let dy = down[1] - MSG2_ARROW_BOB - bounce;
        let uy = up[1] - MSG2_ARROW_BOB + bounce;
        self.arrow_down.pane.move_to(down[0], dy);
        self.arrow_down_kage.pane.move_to(down[0], dy);
        self.arrow_up.pane.move_to(up[0], uy);
        self.arrow_up_kage.pane.move_to(up[0], uy);
    }

    fn shade_low(&self, y: i32) -> u8 {
        if y >= 0 {
            MSG2_TEX_I4_COLOR.get(y as usize).copied().unwrap_or(0)
        } else {
            0
        }
    }

    fn shade_high(&self, y: i32) -> u8 {
        if y <= MSG2_SHADE_TABLE_MAX {
            MSG2_TEX_I4_COLOR.get(y as usize).copied().unwrap_or(0)
        } else {
            0
        }
    }

    pub fn set_char_alpha(&mut self, index: usize, hio: &Msg2Hio) {
        let temp_r6 = self
            .text_pane
            .first()
            .map(|p| p.text.get_line_space() as i32)
            .unwrap_or(0);
        let board_y = self
            .board_pane
            .first()
            .map(|p| p.pos_top_left_orig[1])
            .unwrap_or(0.0f32);
        let shadow_y = self
            .board_sdw_pane
            .first()
            .map(|p| p.pos_top_left_orig[1])
            .unwrap_or(0.0f32);
        let pane_y = self
            .text_pane
            .get(index)
            .map(|p| p.state.pos_top_left[1])
            .unwrap_or(0.0f32);
        let shift = self.line_shift.get(index).copied().unwrap_or(0);
        let temp_f2 =
            (board_y - shadow_y) + pane_y + (self.half_line_space as f32 * 2i32.saturating_sub(shift) as f32);
        let temp_r27 = temp_r6.wrapping_mul(shift);
        let var_r26 = (temp_f2 as i32).wrapping_add(temp_r27);
        let var_r30 = (temp_f2 - hio.field_0x38 as f32) as i32;
        let var_r29 = var_r30.wrapping_add(temp_r27);

        let mx_height = self.mx_height;
        let rx_height = self.rx_height;

        let (mut a0, mut a1);
        if var_r26 < MSG2_SHADE_LOW {
            a0 = self.shade_low(var_r26);
            a1 = self.shade_low(var_r26.wrapping_add(mx_height));
        } else if temp_r27 > MSG2_SHADE_HIGH {
            a0 = self.shade_high(temp_r27);
            a1 = self.shade_high(temp_r27.wrapping_add(mx_height));
        } else {
            a0 = 0xFF;
            a1 = 0xFF;
        }

        let (mut a2, mut a3);
        if var_r29 < MSG2_SHADE_LOW {
            a2 = self.shade_low(var_r29);
            a3 = self.shade_low(var_r29.wrapping_add(rx_height));
        } else if var_r30 > MSG2_SHADE_HIGH {
            a2 = self.shade_high(var_r30);
            a3 = self.shade_high(var_r30.wrapping_add(rx_height));
        } else {
            a2 = 0xFF;
            a3 = 0xFF;
        }

        let cap = self.box_alpha as u8;
        if a0 > cap {
            a0 = cap;
        }
        if a1 > cap {
            a1 = cap;
        }
        if a2 > cap {
            a2 = cap;
        }
        if a3 > cap {
            a3 = cap;
        }

        for (slot, value) in self.char_alpha.iter_mut().zip([a0, a1, a2, a3]) {
            if let Some(cell) = slot.get_mut(index) {
                *cell = value;
            }
        }

        if let Some(data) = self.msg_data.get_mut(index) {
            data.set_char_alpha(a0, a1, a2, a3);
        }
    }

    pub fn ruby_set(&mut self, hio: &Msg2Hio) {
        if !matches!(
            self.status,
            STATUS_UNK5 | STATUS_TYPING | STATUS_STOP | STATUS_CLOSE_WAIT
        ) {
            return;
        }
        for i in 0..MSG2_PANES {
            if hio.field_0x4b == 0 {
                if let Some(p) = self.ruby_pane.get_mut(i) {
                    p.hide();
                }
            } else if let Some(p) = self.ruby_pane.get_mut(i) {
                p.show();
            }
        }
    }

    pub fn stop_proc(&mut self, env: &mut Msg2Env, stick: &Msg2Stick) -> i32 {
        if self.stick_state != 0 {
            self.stick_info_check(stick);
            return 1;
        }
        if stick.pad0_y > MSG2_STICK_THRESHOLD || stick.pad3_y > MSG2_STICK_THRESHOLD {
            if self.scroll_line != 0 {
                let below = self.pane_below;
                self.stick_state = 1;
                self.screen_data_set(below);
                self.scroll_phase = 3;
                self.scroll_line =
                    self.scroll_line.wrapping_sub(i32::from(self.mesg_entry.field_0x16));
                self.status = STATUS_UNK5;
                env.se = Some(Msg2Se::Scroll1);
            }
        } else if stick.trig_a
            || stick.pad0_y < -MSG2_STICK_THRESHOLD
            || stick.pad3_y < -MSG2_STICK_THRESHOLD
        {
            let above = self.pane_above;
            self.stick_state = 2;
            self.screen_data_set(above);
            self.scroll_phase = 1;
            self.scroll_line = self.scroll_line.wrapping_add(i32::from(self.mesg_entry.field_0x16));
            self.status = STATUS_UNK5;
            env.se = Some(Msg2Se::Scroll1);
        } else {
            env.meter_mode = 1;
        }
        1
    }

    pub fn closewait_proc(&mut self, env: &mut Msg2Env, stick: &Msg2Stick) -> i32 {
        if self.stick_state != 0 {
            self.stick_info_check(stick);
            return 1;
        }
        if stick.pad0_y > MSG2_STICK_THRESHOLD || stick.pad3_y > MSG2_STICK_THRESHOLD {
            if self.scroll_line != 0 {
                let below = self.pane_below;
                self.stick_state = 1;
                self.screen_data_set(below);
                self.scroll_phase = 3;
                self.scroll_line =
                    self.scroll_line.wrapping_sub(i32::from(self.mesg_entry.field_0x16));
                self.status = STATUS_UNK5;
                env.se = Some(Msg2Se::Scroll1);
            }
        } else if stick.trig_a
            || stick.pad0_y < -MSG2_STICK_THRESHOLD
            || stick.pad3_y < -MSG2_STICK_THRESHOLD
        {
            self.stick_state = 2;
            env.se = Some(Msg2Se::TalkWinClose);
            self.status = STATUS_MSG_ENDS;
        } else {
            env.meter_mode = 4;
        }
        1
    }

    pub fn open_proc<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        hio: &Msg2Hio,
        env: &mut Msg2Env,
        len: &MsgLenState,
        layout: &MsgLayoutState,
    ) -> i32 {
        let center = self.pane_center;
        if self.open_t == 0 {
            let line = self.scroll_line;
            self.message_out(font, center, line, len, layout, hio);
            let stop = self
                .msg_data
                .get(center)
                .is_some_and(|d| d.mesg_status == STATUS_STOP);
            if stop {
                let below = self.pane_below;
                let extra = i32::from(self.mesg_entry.field_0x16);
                self.message_out(font, below, line.wrapping_add(extra), len, layout, hio);
            }
        } else if self.open_t == self.pop_speed {
            self.status = self
                .msg_data
                .get(center)
                .map(|d| d.mesg_status)
                .unwrap_or(STATUS_TYPING);
        }
        let t = self.open_t;
        let ramp = fop_msgm_value_increase(self.pop_speed, t, 0).unwrap_or_default();
        self.box_alpha = ramp * 255.0f32;
        if self.open_t != 0 {
            self.message_show(font, len, layout, hio);
        }
        let stop = self
            .msg_data
            .get(center)
            .is_some_and(|d| d.mesg_status == STATUS_STOP);
        let alpha = self.box_alpha as u8;
        if stop {
            self.arrow_down.now_alpha = alpha;
            self.arrow_down_kage.now_alpha = alpha;
        } else {
            self.dot_top.now_alpha = alpha;
            self.dot_bottom.now_alpha = alpha;
        }
        let t = self.open_t;
        self.aim_alpha_sqrt(hio, env, self.pop_speed, t);
        env.brightness = self.aim_alpha as u8;
        self.open_t += 1;
        1
    }

    pub fn close_proc(&mut self, hio: &Msg2Hio, env: &mut Msg2Env) -> i32 {
        if self.open_t == 0 {
            for i in 0..MSG2_PANES {
                self.message_pane_hide(i);
                self.out_font_hide(i);
            }
            self.status = STATUS_BOX_CLOSED;
        }
        self.open_t -= 1;
        if self.open_t < 0 {
            self.open_t = 0;
        }
        let t = self.open_t;
        let ramp = fop_msgm_value_increase(self.pop_speed, t, 0).unwrap_or_default();
        self.box_alpha = ramp * 255.0f32;
        let alpha = self.box_alpha as u8;
        if self.arrow_up.now_alpha != 0 {
            self.arrow_up.now_alpha = alpha;
            self.arrow_up_kage.now_alpha = alpha;
        }
        if self.arrow_down.now_alpha != 0 {
            self.arrow_down.now_alpha = alpha;
            self.arrow_down_kage.now_alpha = alpha;
        }
        if self.dot_top.now_alpha != 0 {
            self.dot_top.now_alpha = alpha;
            self.dot_bottom.now_alpha = alpha;
        }
        let t = self.open_t;
        self.aim_alpha_square(hio, env, self.pop_speed, t);
        env.brightness = self.aim_alpha as u8;
        1
    }

    pub fn outwait_proc<F: FontSlot + ?Sized>(
        &mut self,
        font: &F,
        len: &MsgLenState,
        layout: &MsgLayoutState,
        hio: &Msg2Hio,
    ) -> i32 {
        match self.scroll_phase {
            1 | 2 => self.scroll_offset += MSG2_SCROLL_STEP,
            3 | 4 => self.scroll_offset -= MSG2_SCROLL_STEP,
            _ => {}
        }
        if self.scroll_offset > MSG2_SCROLL_LIMIT {
            self.scroll_offset -= MSG2_SCROLL_WRAP;
        } else if self.scroll_offset < -MSG2_SCROLL_LIMIT {
            self.scroll_offset += MSG2_SCROLL_WRAP;
        }

        let origin_y = self
            .text_pane
            .first()
            .map(|p| p.state.pos_top_left_orig[1])
            .unwrap_or(0.0f32);
        let board_height = self
            .board_pane
            .first()
            .map(|p| p.size_orig[1])
            .unwrap_or(0.0f32);

        if self.scroll_phase == 1 {
            for p in self.text_pane.iter_mut() {
                p.state.pos_top_left[1] -= 12.0f32;
            }
            self.shade = self.shade.wrapping_sub(85);
            let above = self.pane_above;
            let y = self
                .text_pane
                .get(above)
                .map(|p| p.state.pos_top_left[1])
                .unwrap_or(0.0f32);
            if y < -self.pane_height {
                self.scroll_phase = 2;
                let old_below = self.pane_below;
                let old_center = self.pane_center;
                self.pane_center = old_below;
                self.pane_below = above;
                self.pane_above = old_center;
                let new_below = self.pane_below;
                let stop = self
                    .msg_data
                    .get(self.pane_center)
                    .is_some_and(|d| d.mesg_status == STATUS_STOP);
                let line = self.scroll_line;
                let extra = i32::from(self.mesg_entry.field_0x16);
                if stop {
                    self.message_out(font, new_below, line.wrapping_add(extra), len, layout, hio);
                } else {
                    self.message_pane_hide(new_below);
                    self.out_font_hide(new_below);
                }
                self.shade = 0;
                self.arrow_up_hide();
                self.arrow_down_hide();
                self.dot_hide();
            }
        } else if self.scroll_phase == 3 {
            for p in self.text_pane.iter_mut() {
                p.state.pos_top_left[1] += 12.0f32;
            }
            self.shade = self.shade.wrapping_sub(127);
            let below = self.pane_below;
            let y = self
                .text_pane
                .get(below)
                .map(|p| p.state.pos_top_left[1])
                .unwrap_or(0.0f32);
            if y > board_height {
                self.scroll_phase = 4;
                let old_above = self.pane_above;
                let old_center = self.pane_center;
                self.pane_center = old_above;
                self.pane_above = below;
                self.pane_below = old_center;
                let new_above = self.pane_above;
                let line = self.scroll_line;
                let extra = i32::from(self.mesg_entry.field_0x16);
                if self.scroll_line > 0 {
                    self.message_out(font, new_above, line.wrapping_sub(extra), len, layout, hio);
                } else {
                    self.message_pane_hide(new_above);
                    self.out_font_hide(new_above);
                }
                self.shade = 0;
                self.arrow_up_hide();
                self.arrow_down_hide();
                self.dot_hide();
            }
        } else if self.scroll_phase == 2 {
            for p in self.text_pane.iter_mut() {
                p.state.pos_top_left[1] -= 12.0f32;
            }
            self.shade = self.shade.wrapping_add(43);
            let center = self.pane_center;
            let center_y = self
                .text_pane
                .get(center)
                .map(|p| p.state.pos_top_left[1])
                .unwrap_or(0.0f32);
            if center_y <= origin_y {
                self.place_panes_vertically(origin_y);
                self.scroll_phase = 0;
                let stop = self
                    .msg_data
                    .get(center)
                    .is_some_and(|d| d.mesg_status == STATUS_STOP);
                if stop {
                    self.status = STATUS_STOP;
                    self.arrow_down_show();
                } else {
                    self.status = STATUS_MSG_DISPLAYED;
                    self.dot_show();
                }
                self.shade = 0xFF;
                if self.scroll_line != 0 {
                    self.arrow_up_show();
                }
            }
            self.message_show(font, len, layout, hio);
        } else if self.scroll_phase == 4 {
            for p in self.text_pane.iter_mut() {
                p.state.pos_top_left[1] += 12.0f32;
            }
            self.shade = self.shade.wrapping_add(37);
            let center = self.pane_center;
            let center_y = self
                .text_pane
                .get(center)
                .map(|p| p.state.pos_top_left[1])
                .unwrap_or(0.0f32);
            if center_y >= origin_y {
                self.place_panes_vertically(origin_y);
                self.scroll_phase = 0;
                self.status = STATUS_STOP;
                self.shade = 0xFF;
                self.arrow_down_show();
                if self.scroll_line != 0 {
                    self.arrow_up_show();
                }
            }
            self.message_show(font, len, layout, hio);
        } else {
            self.message_show(font, len, layout, hio);
        }
        1
    }

    fn place_panes_vertically(&mut self, origin_y: f32) {
        let above = self.pane_above;
        let center = self.pane_center;
        let below = self.pane_below;
        let height = self.pane_height;
        if let Some(p) = self.text_pane.get_mut(above) {
            p.state.pos_top_left[1] = origin_y - height;
        }
        if let Some(p) = self.text_pane.get_mut(center) {
            p.state.pos_top_left[1] = origin_y;
        }
        if let Some(p) = self.text_pane.get_mut(below) {
            p.state.pos_top_left[1] = origin_y + height;
        }
    }

    pub fn execute<F: MsgArchive + ?Sized>(
        &mut self,
        ctx: &mut Msg2Ctx<F>,
        info: &mut MsgGameInfo,
    ) -> bool {
        let hio = *ctx.hio;
        let stick = ctx.stick;
        let fonts: &FontBank = ctx.fonts;
        let len = *ctx.len;
        let layout = *ctx.layout;
        let opt_ruby = ctx.env.opt_ruby;
        let env: &mut Msg2Env = ctx.env;
        env.se = None;

        match self.status {
            STATUS_BOX_OPENING => {
                self.open_proc(fonts, &hio, env, &len, &layout);
            }
            STATUS_UNK5 => {
                self.outwait_proc(fonts, &len, &layout, &hio);
            }
            STATUS_STOP => {
                self.stop_proc(env, &stick);
            }
            STATUS_MSG_DISPLAYED => {
                self.closewait_proc(env, &stick);
            }
            STATUS_MSG_ENDS => {
                self.close_proc(&hio, env);
            }
            STATUS_MSG_DESTROYED => {
                self.delete(env, info);
            }
            _ => {}
        }

        self.arrow_move();

        let ruby_dy = if opt_ruby {
            MSG2_RUBY_OFF_DY
        } else {
            0.0f32
        };

        let hio_x = hio.field_0x58 as f32;
        let hio_ruby = hio.field_0x38 as i32;
        for i in 0..MSG2_PANES {
            let base_x = self
                .text_pane
                .get(i)
                .map(|p| p.state.pos_top_left[0])
                .unwrap_or(0.0f32);
            let base_y = self
                .text_pane
                .get(i)
                .map(|p| p.state.pos_top_left[1])
                .unwrap_or(0.0f32);
            let sp64 = (base_x + hio_x) as i32;
            let sp74 = (base_y + ruby_dy) as i32;
            if let Some(p) = self.text_pane.get_mut(i) {
                p.move_to(sp64 as f32, sp74 as f32);
            }
            if let Some(p) = self.ruby_pane.get_mut(i) {
                p.move_to(sp64 as f32, (sp74 - hio_ruby) as f32);
            }
            if let Some(p) = self.text_sdw_pane.get_mut(i) {
                p.move_to((sp64 + MSG2_SDW_DX) as f32, (sp74 + MSG2_SDW_DY) as f32);
            }
            if let Some(p) = self.ruby_sdw_pane.get_mut(i) {
                p.move_to(
                    (sp64 + MSG2_SDW_DX) as f32,
                    (sp74 + MSG2_SDW_DY - hio_ruby) as f32,
                );
            }
        }

        self.ruby_set(&hio);

        for i in 0..MSG2_PANES {
            if let Some(p) = self.text_pane.get_mut(i) {
                p.text.set_font_color(hio.field_0x5, hio.field_0x5);
                p.text.set_white(hio.field_0x9);
                p.text.set_black(hio.field_0xd);
            }
            if let Some(p) = self.ruby_pane.get_mut(i) {
                p.text.set_font_color(hio.field_0x1d, hio.field_0x1d);
                p.text.set_white(hio.field_0x21);
                p.text.set_black(hio.field_0x25);
            }
            if let Some(p) = self.text_sdw_pane.get_mut(i) {
                p.text.set_font_color(hio.field_0x11, hio.field_0x11);
                p.text.set_white(hio.field_0x15);
                p.text.set_black(hio.field_0x19);
            }
            if let Some(p) = self.ruby_sdw_pane.get_mut(i) {
                p.text.set_font_color(hio.field_0x11, hio.field_0x11);
                p.text.set_white(hio.field_0x15);
                p.text.set_black(hio.field_0x19);
            }
        }

        let ruby_visible = self
            .ruby_pane
            .get(1)
            .is_some_and(|p| p.is_visible());
        for i in 0..MSG2_PANES {
            if opt_ruby {
                if let Some(p) = self.ruby_pane.get_mut(i) {
                    p.hide();
                }
                if let Some(p) = self.ruby_sdw_pane.get_mut(i) {
                    p.hide();
                }
            } else {
                if ruby_visible {
                    if let Some(p) = self.ruby_pane.get_mut(i) {
                        p.show();
                    }
                }
                if self
                    .ruby_sdw_pane
                    .get(i)
                    .is_some_and(|p| p.is_visible())
                    && let Some(p) = self.ruby_sdw_pane.get_mut(i)
                {
                    p.show();
                }
            }
        }

        info.mesg_status = self.status;
        self.alive
    }

    pub fn draw(&mut self, hio: &Msg2Hio) {
        let board_x = self
            .board_sdw_pane
            .first()
            .map(|p| p.pos_top_left_orig[0])
            .unwrap_or(0.0f32);
        let board_y = self
            .board_sdw_pane
            .first()
            .map(|p| p.pos_top_left_orig[1])
            .unwrap_or(0.0f32);
        let r4 = (board_x + MSG2_CAPTURE_CENTER_X) as i32;
        let r5 = (board_y + MSG2_CAPTURE_CENTER_Y) as i32;
        let sp8 = r4 - (MSG2_CAPTURE_W / 2);
        let sp10 = r5 - (MSG2_CAPTURE_H / 2);

        let alpha = self.box_alpha as u8;
        for i in 0..MSG2_PANES {
            for pane in [
                self.text_pane.get_mut(i),
                self.ruby_pane.get_mut(i),
                self.text_sdw_pane.get_mut(i),
                self.ruby_sdw_pane.get_mut(i),
            ]
            .into_iter()
            .flatten()
            {
                pane.state.now_alpha = alpha;
                pane.set_alpha();
            }
        }

        for pane in [
            &mut self.arrow_up,
            &mut self.arrow_up_kage,
            &mut self.arrow_down,
            &mut self.arrow_down_kage,
            &mut self.dot_top,
            &mut self.dot_bottom,
        ] {
            pane.set_alpha();
        }

        self.multi_tex_draw(
            hio,
            sp8 as i16,
            sp10 as i16,
            MSG2_CAPTURE_W as i16,
            MSG2_CAPTURE_H as i16,
        );
    }

    pub fn out_font_draw(&mut self, out_hio: &MeterMsgHio) {
        let var_f31 = self
            .board_pane
            .first()
            .map(|p| p.pane.glb_bounds()[1])
            .unwrap_or(0.0f32);
        let var_f30 = self
            .board_pane
            .first()
            .map(|p| p.pane.glb_bounds()[3])
            .unwrap_or(0.0f32);

        for i in 0..MSG2_PANES {
            for j in 0..MSG2_BUTTONS {
                let (icon_num, pos_x, pos_y, scale) = match self.msg_data.get(i) {
                    Some(d) => (
                        d.icon_num.get(j).copied().unwrap_or(ICON_NONE),
                        d.icon_pos_x.get(j).copied().unwrap_or(0),
                        d.icon_pos_y.get(j).copied().unwrap_or(0),
                        d.icon_scale.get(j).copied().unwrap_or(0),
                    ),
                    None => (ICON_NONE, 0, 0, 0),
                };
                if icon_num == ICON_NONE {
                    continue;
                }
                let glb = self
                    .text_pane
                    .get(i)
                    .map(|p| p.state.pane.glb_bounds())
                    .unwrap_or([0.0f32; 4]);
                let line_shift = self.line_shift.get(i).copied().unwrap_or(0);
                let r18 = (pos_x as f32 + glb[0]) as i32;
                let r17 = if scale > self.font_size {
                    if line_shift > 1 {
                        let temp = self.half_line_space as f32 * (3.0f32 - pos_y as f32);
                        (temp + glb[1] - (scale / 2) as f32) as i32
                    } else {
                        let temp = self.half_line_space as f32 * 4.0f32;
                        (temp + glb[1] - (scale / 2) as f32) as i32
                    }
                } else {
                    let temp = self.half_line_space as f32
                        * (3.0f32 - line_shift as f32 + pos_y as f32 * 2.0f32);
                    (temp + glb[1]) as i32
                };
                let r14 = self.box_alpha as u8;

                if (r17 as f32) > var_f31 && (r17 as f32) < var_f30 - scale as f32 {
                    let icon = self.bbutton_icon.get_mut(j).and_then(|row| row.get_mut(i));
                    let kage = self.bbutton_kage.get_mut(j).and_then(|row| row.get_mut(i));
                    let timer = self
                        .bbutton_timer
                        .get_mut(j)
                        .and_then(|row| row.get_mut(i));
                    if let (Some(icon), Some(kage), Some(timer)) = (icon, kage, timer) {
                        fop_msgm_out_font_draw(
                            out_hio, icon, kage, r18, r17, scale, timer, r14, icon_num,
                        );
                    }
                }
            }
        }
    }

    pub fn draw_list(&mut self, out_hio: &MeterMsgHio, fonts: &mut FontBank) -> Msg2DrawOutput {
        self.out_font_draw(out_hio);

        let mut out = Msg2DrawOutput {
            board: self.board,
            ..Default::default()
        };
        for i in 0..MSG2_PANES {
            let panes: [Option<&TextPane>; 4] = [
                self.text_pane.get(i),
                self.ruby_pane.get(i),
                self.text_sdw_pane.get(i),
                self.ruby_sdw_pane.get(i),
            ];
            let layouts: [TextLayout; 4] = panes.map(|p| {
                p.map_or_else(TextLayout::default, |p| p.draw_self(fonts, 0.0f32, 0.0f32))
            });
            out.text[i] = layouts[0].clone();
            out.ruby[i] = layouts[1].clone();
            out.text_sdw[i] = layouts[2].clone();
            out.ruby_sdw[i] = layouts[3].clone();
        }
        out
    }

    pub fn create<F: MsgArchive + ?Sized>(
        &mut self,
        ctx: &mut Msg2Ctx<F>,
        info: &mut MsgGameInfo,
        msg_no: u32,
    ) -> Msg2Phase {
        if ctx.env.heap_lock_flag != 0 && ctx.env.heap_lock_flag != MSG2_HEAP_LOCK {
            return Msg2Phase::Init;
        }

        self.heap_locked = true;

        let mx = ctx.mx_font;
        let rx = ctx.rx_font;
        self.fontdata_init(mx, rx);

        for i in 0..MSG2_PANES {
            for j in 0..MSG2_BUTTONS {
                if let Some(icon) = self.bbutton_icon.get_mut(j).and_then(|row| row.get_mut(i)) {
                    icon.visible = false;
                    icon.set_alpha(0);
                }
                if let Some(kage) = self.bbutton_kage.get_mut(j).and_then(|row| row.get_mut(i)) {
                    kage.visible = false;
                    kage.set_alpha(0);
                }
                if let Some(timer) = self.bbutton_timer.get_mut(j).and_then(|row| row.get_mut(i)) {
                    *timer = -1;
                }
            }
        }

        let msg_no = if msg_no == MSG2_MSG_NO_33A && ctx.env.event_bit_2d80 {
            MSG2_MSG_NO_33B
        } else {
            msg_no
        };

        let archive = ctx.archive;
        let clear_count = ctx.len.clear_count;
        let mut get = MsgGet::default();
        let Some(head) = get.get_mesg_header(archive, clear_count, msg_no) else {
            return Msg2Phase::Init;
        };
        let Some(message) = get.get_message(head) else {
            return Msg2Phase::Init;
        };
        self.message = message.to_vec();
        if let Some(entry) = get.get_mesg_entry(head) {
            self.mesg_entry = entry;
        }
        self.mesg_number = get.get_mesg_number();
        self.msg_get = get;
        self.msg_no = msg_no;
        self.status = STATUS_BOX_OPENING;

        self.multi_tex_init();

        let screen = *ctx.screen;
        let hio = *ctx.hio;
        let out_hio = *ctx.out_hio;
        let fonts: &FontBank = ctx.fonts;
        for i in 0..MSG2_PANES {
            self.screen_data_init(i, &screen, &hio, &out_hio, fonts);
            self.screen_data_set(i);
            self.message_pane_hide(i);
        }

        self.screen_data_value_init(&hio);
        let stick = ctx.stick;
        self.stick_info_init(&stick);
        self.scroll_phase = 0;
        self.pane_above = 2;
        self.pane_center = 0;
        self.pane_below = 1;
        self.pop_speed = hio.field_0x3b as i32;
        self.alive = true;
        info.mesg_status = self.status;
        Msg2Phase::Complete
    }

    pub fn delete(&mut self, env: &mut Msg2Env, info: &mut MsgGameInfo) -> bool {
        info.mesg_status = 0;
        env.brightness = 0xFF;
        for i in 0..MSG2_PANES {
            self.message_pane_hide(i);
            self.out_font_hide(i);
        }
        self.heap_locked = false;
        self.alive = false;
        true
    }

    pub fn is_delete(&self) -> bool {
        true
    }
}