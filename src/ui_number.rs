use bevy_ecs::prelude::*;
use bevy_ecs::world::World;

use crate::j2d;
use crate::sim::isle_timer::IsleTimer;
use crate::text::font::FontBank;
use crate::text::msgget::{ItemMsgGet, MsgArchive};
use crate::text::msglayout::fop_msgm_get_color_table;
use crate::text::msgproc::{
    fop_msgm_out_font_color_white, fop_msgm_out_font_tex, fop_msgm_value_increase,
};
use crate::text::{
    CTRL_A_BUTTON, CTRL_ARROW_DOWN, CTRL_ARROW_LEFT, CTRL_ARROW_RIGHT, CTRL_ARROW_UP,
    CTRL_B_BUTTON, CTRL_C_STICK, CTRL_DPAD, CTRL_FLASHING_A_BUTTON, CTRL_HEART, CTRL_L_BUTTON,
    CTRL_MAIN_STICK, CTRL_MAIN_STICK_DOWN, CTRL_MAIN_STICK_LEFT, CTRL_MAIN_STICK_LEFT_RIGHT,
    CTRL_MAIN_STICK_RIGHT, CTRL_MAIN_STICK_UP, CTRL_MAIN_STICK_UP_DOWN, CTRL_MUSIC_NOTE,
    CTRL_R_BUTTON, CTRL_X_BUTTON, CTRL_Y_BUTTON, CTRL_Z_BUTTON, ICON_A_BUTTON, ICON_ARROW_DOWN,
    ICON_ARROW_LEFT, ICON_ARROW_RIGHT, ICON_ARROW_UP, ICON_B_BUTTON, ICON_C_STICK, ICON_DPAD,
    ICON_FLASHING_A_BUTTON, ICON_HEART, ICON_L_BUTTON, ICON_MAIN_STICK, ICON_MAIN_STICK_DOWN,
    ICON_MAIN_STICK_LEFT, ICON_MAIN_STICK_LEFT_RIGHT, ICON_MAIN_STICK_RIGHT, ICON_MAIN_STICK_UP,
    ICON_MAIN_STICK_UP_DOWN, ICON_MUSIC_NOTE, ICON_R_BUTTON, ICON_X_BUTTON, ICON_Y_BUTTON,
    ICON_Z_BUTTON, SPCL_COLOR, SPCL_RUBY,
};

pub const PIC_SLOTS: usize = 4;
pub const OUT_FONT_ICONS: usize = 3;
pub const OUT_FONT_BUF: usize = 100;
pub const OUT_FONT_RUBY_BUF: usize = 52;
pub const OUT_FONT_TAG_BUF: usize = 16;
pub const OUT_FONT_PIC_TEX: &str = "font_07_02.bti";
pub const DIGIT_TEX: &str = "rupy_num_00.bti";
pub const ROTATE_Z: u8 = b'z';

pub const FLAG_LEADING_ZERO: u8 = 1;
pub const FLAG_HIDDEN: u8 = 2;

pub const TIMER_DIGITS: i32 = 6;
pub const RUPEE_DIGITS: i32 = 3;
pub const TIMER_FLAG: u8 = FLAG_LEADING_ZERO;

pub const BATTERY_TENS: usize = 0;
pub const BATTERY_ONES: usize = 1;
pub const BATTERY_DEGREE: usize = 2;
pub const BATTERY_SHADOW: usize = 1;

const TITLE_SCALE: f32 = 1.33;
const SCORE_SCALE: f32 = 1.16;
const SHADOW_OFFS: f32 = 4.0;

const BATTERY_ALPHA: u8 = 200;
const BATTERY_BASE_X: f32 = 551.0;
const BATTERY_BASE_Y: f32 = 399.0;
const BATTERY_MIN_ROT: f32 = 20.0;
const BATTERY_CORNER: u32 = 0x0000_00FF;
const BATTERY_WHITE: u32 = 0xFFC8_00FF;
const BATTERY_SHADOW_ALPHA: u8 = 80;
const BATTERY_RULE_ALPHA: u8 = 128;
const BATTERY_RULE_WHITE: u32 = 0xFFFF_FFFF;
const BATTERY_RULE_BLACK: u32 = 0xFFFF_FF00;
const BATTERY_ROT_LOW: f32 = 345.0;
const BATTERY_ROT_HIGH: f32 = 310.0;
const BATTERY_ROT_BASE: f32 = 360.0;
const BATTERY_ROT_SPAN: f32 = 22.5;
const BATTERY_ROT_SCALE: f32 = 35.0;
const BATTERY_ROT_TEXT: f32 = 1.4;
const BATTERY_TEXT_BASE: f32 = 15.0;
const BATTERY_ROT_MAX: f32 = 45.0;
const BATTERY_RULE_OFFS: [f32; 2] = [519.0, 367.0];
const BATTERY_OFFS: [f32; 2] = [498.0, 397.0];
const BATTERY_DRAW_OFFS: [f32; 2] = [551.0, 399.0];
const BATTERY_TENS_OFFS: f32 = 10.0;
const BATTERY_ONES_GAP: f32 = 5.0;

pub fn digit_texture(digit: i32) -> String {
    format!("rupy_num_{digit:02}.bti")
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rotation {
    pub offset: [f32; 2],
    pub axis: u8,
    pub angle: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TimG {
    pub name: String,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Quad {
    pub texture: Option<String>,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub alpha: u8,
    pub white: u32,
    pub black: u32,
    pub corner: [u32; 4],
    pub blend: [f32; 4],
    pub rotation: Option<Rotation>,
}

impl Default for Quad {
    fn default() -> Self {
        Self {
            texture: None,
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
            alpha: 0xFF,
            white: 0xFFFF_FFFF,
            black: 0xFFFF_FFFF,
            corner: [0xFFFF_FFFF; 4],
            blend: [0.0; 4],
            rotation: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Picture {
    pub texture: Option<String>,
    pub extra: Vec<(String, f32)>,
    pub visible: bool,
    pub alpha: u8,
    pub white: u32,
    pub black: u32,
    pub corner: [u32; 4],
    pub blend_color: [f32; 4],
    pub blend_alpha: [f32; 4],
    pub rotation: Option<Rotation>,
    pub pos: [f32; 2],
    pub size: [f32; 2],
}

impl Default for Picture {
    fn default() -> Self {
        Self {
            texture: None,
            extra: Vec::new(),
            visible: true,
            alpha: 0xFF,
            white: 0xFFFF_FFFF,
            black: 0xFFFF_FFFF,
            corner: [0xFFFF_FFFF; 4],
            blend_color: [0.0; 4],
            blend_alpha: [0.0; 4],
            rotation: None,
            pos: [0.0; 2],
            size: [0.0; 2],
        }
    }
}

impl Picture {
    pub fn new(texture: &str) -> Self {
        Self {
            texture: Some(texture.to_string()),
            ..Self::default()
        }
    }

    pub fn from_pane(pane: &j2d::ScreenPane) -> Self {
        Self {
            texture: pane.texture.clone(),
            visible: pane.visible,
            alpha: pane.alpha,
            pos: [pane.bounds.x, pane.bounds.y],
            size: [pane.bounds.w, pane.bounds.h],
            ..Self::default()
        }
    }

    pub fn change_texture(&mut self, name: &str) {
        self.texture = Some(name.to_string());
    }

    pub fn append(&mut self, name: &str, ratio: f32) {
        self.extra.push((name.to_string(), ratio));
    }

    pub fn remove(&mut self) {
        self.extra.pop();
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn set_alpha(&mut self, alpha: u8) {
        self.alpha = alpha;
    }

    pub fn set_white(&mut self, white: u32) {
        self.white = white;
    }

    pub fn set_black(&mut self, black: u32) {
        self.black = black;
    }

    pub fn set_corner_color(&mut self, color: u32) {
        self.corner = [color; 4];
    }

    pub fn set_blend_ratio(&mut self, f1: f32, f2: f32, f3: f32, f4: f32) {
        self.blend_color = [f1, f2, f3, f4];
        self.blend_alpha = [f1, f2, f3, f4];
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.pos = [x, y];
    }

    pub fn resize(&mut self, w: f32, h: f32) {
        self.size = [w, h];
    }

    pub fn rotate(&mut self, offset_x: f32, offset_y: f32, axis: u8, angle: f32) {
        self.rotation = Some(Rotation {
            offset: [offset_x, offset_y],
            axis,
            angle,
        });
    }

    pub fn quad(&self, x: f32, y: f32, w: f32, h: f32) -> Quad {
        Quad {
            texture: self.texture.clone(),
            x,
            y,
            w,
            h,
            alpha: self.alpha,
            white: self.white,
            black: self.black,
            corner: self.corner,
            blend: self.blend_color,
            rotation: self.rotation,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextBuf<const N: usize> {
    pub bytes: [u8; N],
    pub len: usize,
}

impl<const N: usize> TextBuf<N> {
    pub fn new() -> Self {
        Self {
            bytes: [0; N],
            len: 0,
        }
    }

    pub fn clear(&mut self) {
        self.bytes = [0; N];
        self.len = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push(&mut self, byte: u8) {
        if self.len + 1 < N {
            self.bytes[self.len] = byte;
            self.len += 1;
            self.bytes[self.len] = 0;
        }
    }

    pub fn push_str(&mut self, text: &str) {
        for byte in text.bytes() {
            self.push(byte);
        }
    }

    pub fn cat(&mut self, other: &[u8]) {
        for byte in other.iter().copied() {
            self.push(byte);
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.get(..self.len).unwrap_or(&[])
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextBox {
    pub bounds: [f32; 4],
    pub font_size: [u32; 2],
    pub char_space: f32,
    pub line_space: f32,
    pub string: Vec<u8>,
    pub shift: [f32; 2],
    pub visible: bool,
    pub alpha: u8,
}

impl TextBox {
    pub fn from_pane(pane: &j2d::ScreenPane) -> Self {
        Self {
            bounds: [
                pane.bounds.x,
                pane.bounds.y,
                pane.bounds.right(),
                pane.bounds.bottom(),
            ],
            font_size: [u32::from(pane.font_size_x), u32::from(pane.font_size_y)],
            char_space: pane.char_space as f32,
            line_space: pane.line_space as f32,
            string: pane.text.as_bytes().to_vec(),
            shift: [0.0, 0.0],
            visible: pane.visible,
            alpha: pane.alpha,
        }
    }

    pub fn width(&self) -> f32 {
        self.bounds[2] - self.bounds[0]
    }

    pub fn height(&self) -> f32 {
        self.bounds[3] - self.bounds[1]
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn set_alpha(&mut self, alpha: u8) {
        self.alpha = alpha;
    }

    pub fn shift_set(&mut self, x: f32, y: f32) {
        self.shift = [x, y];
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        let w = self.width();
        let h = self.height();
        self.bounds[0] = x;
        self.bounds[1] = y;
        self.bounds[2] = x + w;
        self.bounds[3] = y + h;
    }

    pub fn set_string(&mut self, text: &[u8]) {
        self.string = text.to_vec();
    }
}

pub const STICK_PHASE_87: i16 = 10;
pub const STICK_PHASE_88: i16 = 0;
pub const STICK_PHASE_89: i16 = 0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StickTiming {
    pub field_0x87: i16,
    pub field_0x88: i16,
    pub field_0x89: i16,
}

impl Default for StickTiming {
    fn default() -> Self {
        Self {
            field_0x87: STICK_PHASE_87,
            field_0x88: STICK_PHASE_88,
            field_0x89: STICK_PHASE_89,
        }
    }
}

impl StickTiming {
    pub fn new(field_0x87: i16, field_0x88: i16, field_0x89: i16) -> Self {
        Self {
            field_0x87,
            field_0x88,
            field_0x89,
        }
    }

    pub fn period(&self) -> i16 {
        self.field_0x88 + self.field_0x87 + self.field_0x89 + self.field_0x87
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NumberSprite {
    pub pictures: [Option<Picture>; PIC_SLOTS],
    pub pos: [[f32; 2]; PIC_SLOTS],
    pub width: f32,
    pub height: f32,
    pub value: i32,
    pub flag: u8,
    pub digit_num: i32,
    pub digits: Option<Vec<i32>>,
}

impl NumberSprite {
    pub fn init(&mut self, digit_num: i32, x: i16, y: i16, w: i16, h: i16, flag: u8) -> bool {
        self.digit_num = digit_num;
        self.flag = flag;
        self.width = f32::from(w);
        self.height = f32::from(h);

        for i in 0..digit_num {
            let slot = usize::try_from(i).unwrap_or(0);
            if slot >= PIC_SLOTS {
                break;
            }
            self.pictures[slot] = Some(Picture::new(DIGIT_TEX));
            if self.pictures[slot].is_none() {
                return false;
            }
            self.pos[slot] = [(i32::from(x) - i32::from(w) * i) as f32, f32::from(y)];
        }

        true
    }

    pub fn set_value(&mut self, value: i32) {
        self.value = value;
    }

    pub fn set_digits(&mut self, digits: &[i32]) {
        self.digits = Some(digits.to_vec());
    }

    pub fn clear_digits(&mut self) {
        self.digits = None;
    }

    pub fn set_flag(&mut self, flag: u8) {
        self.flag = flag;
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn digit_num(&self) -> i32 {
        self.digit_num
    }

    pub fn flag(&self) -> u8 {
        self.flag
    }

    pub fn digits(&self) -> Vec<i32> {
        if let Some(list) = &self.digits {
            return list.clone();
        }
        let mut value = self.value;
        let mut out = Vec::new();
        for i in 0..self.digit_num {
            if i != 0 && value == 0 && (self.flag & FLAG_LEADING_ZERO) == 0 {
                break;
            }
            out.push(value % 10);
            value /= 10;
        }
        out
    }

    pub fn digit_texture(&self, slot: usize) -> Option<String> {
        Some(digit_texture(*self.digits().get(slot)?))
    }

    pub fn draw(&mut self) -> Vec<Quad> {
        let mut quads = Vec::new();
        if (self.flag & FLAG_HIDDEN) != 0 {
            return quads;
        }
        let digits = self.digits();
        for i in 0..self.digit_num {
            let slot = usize::try_from(i).unwrap_or(0);
            if slot >= PIC_SLOTS {
                break;
            }
            let Some(&digit) = digits.get(slot) else {
                break;
            };
            let name = digit_texture(digit);
            let Some(pic) = self.pictures[slot].as_mut() else {
                continue;
            };
            pic.change_texture(&name);
            quads.push(pic.quad(
                self.pos[slot][0],
                self.pos[slot][1],
                self.width,
                self.height,
            ));
        }
        quads
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MinigamePlate {
    pub title: Option<Picture>,
    pub score: Option<Picture>,
    pub score_shadow: Option<Picture>,
    pub title_pos: [f32; 2],
    pub title_size: [f32; 2],
    pub score_pos: [f32; 2],
    pub score_size: [f32; 2],
}

impl MinigamePlate {
    pub fn init(&mut self, title: &TimG, score: &TimG) -> bool {
        self.title = Some(Picture::new(&title.name));
        if self.title.is_none() {
            return false;
        }

        self.score = Some(Picture::new(&score.name));
        if self.score.is_none() {
            return false;
        }

        self.score_shadow = Some(Picture::new(&score.name));
        if self.score_shadow.is_none() {
            return false;
        }

        self.title_size = [
            title.width as f32 * TITLE_SCALE,
            title.height as f32 * TITLE_SCALE,
        ];
        self.score_size = [
            score.width as f32 * SCORE_SCALE,
            score.height as f32 * SCORE_SCALE,
        ];

        if let Some(shadow) = self.score_shadow.as_mut() {
            shadow.set_corner_color(0x0000_00FF);
        }
        true
    }

    pub fn set_title_pos(&mut self, x: f32, y: f32) {
        self.title_pos = [x, y];
    }

    pub fn set_score_pos(&mut self, x: f32, y: f32) {
        self.score_pos = [x, y];
    }

    pub fn set_title_alpha(&mut self, alpha: u8) {
        if let Some(title) = self.title.as_mut() {
            title.set_alpha(alpha);
        }
    }

    pub fn set_score_alpha(&mut self, alpha: u8) {
        if let Some(score) = self.score.as_mut() {
            score.set_alpha(0xFF);
        }
        if let Some(shadow) = self.score_shadow.as_mut() {
            shadow.set_alpha(alpha);
        }
    }

    pub fn draw(&self) -> Vec<Quad> {
        let mut quads = Vec::new();
        if let Some(shadow) = self.score_shadow.as_ref() {
            quads.push(shadow.quad(
                self.score_pos[0] + SHADOW_OFFS,
                self.score_pos[1] + SHADOW_OFFS,
                self.score_size[0],
                self.score_size[1],
            ));
        }
        if let Some(score) = self.score.as_ref() {
            quads.push(score.quad(
                self.score_pos[0],
                self.score_pos[1],
                self.score_size[0],
                self.score_size[1],
            ));
        }
        if let Some(title) = self.title.as_ref() {
            quads.push(title.quad(
                self.title_pos[0],
                self.title_pos[1],
                self.title_size[0],
                self.title_size[1],
            ));
        }
        quads
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BatteryGauge {
    pub rule: Option<Picture>,
    pub battery: Option<Picture>,
    pub battery_base: Option<Picture>,
    pub num: [[Option<Picture>; 2]; 3],
    pub picture_pos: [f32; 2],
    pub num_pos: [f32; 2],
    pub rule_size: [f32; 2],
    pub battery_size: [f32; 2],
    pub battery_base_size: [f32; 2],
    pub num_size: [f32; 2],
    pub rotation: f32,
}

impl BatteryGauge {
    pub fn init(
        &mut self,
        rule: &TimG,
        battery: &TimG,
        battery_base: &TimG,
        degree: &TimG,
    ) -> bool {
        self.rule = Some(Picture::new(&rule.name));
        if self.rule.is_none() {
            return false;
        }
        self.rule_size = [rule.width as f32, rule.height as f32];

        self.battery = Some(Picture::new(&battery.name));
        if self.battery.is_none() {
            return false;
        }
        self.battery_size = [battery.width as f32, battery.height as f32];

        self.battery_base = Some(Picture::new(&battery_base.name));
        if self.battery_base.is_none() {
            return false;
        }
        self.battery_base_size = [battery_base.width as f32, battery_base.height as f32];

        for i in 0..2 {
            self.num[BATTERY_DEGREE][i] = Some(Picture::new(&degree.name));
            if self.num[BATTERY_DEGREE][i].is_none() {
                return false;
            }
            self.num[BATTERY_ONES][i] = Some(Picture::new(DIGIT_TEX));
            if self.num[BATTERY_ONES][i].is_none() {
                return false;
            }
            self.num[BATTERY_TENS][i] = Some(Picture::new(DIGIT_TEX));
            if self.num[BATTERY_TENS][i].is_none() {
                return false;
            }
        }

        if let Some(cell) = self.battery.as_mut() {
            cell.set_alpha(BATTERY_ALPHA);
        }

        self.num_size = [degree.width as f32, degree.height as f32];

        self.picture_pos = [0.0, 0.0];
        self.num_pos = [
            BATTERY_BASE_X,
            self.battery_base_size[1] / 2.0 + BATTERY_BASE_Y,
        ];

        if let Some(cell) = self.rule.as_mut() {
            cell.set_alpha(BATTERY_RULE_ALPHA);
            cell.set_white(BATTERY_RULE_WHITE);
            cell.set_black(BATTERY_RULE_BLACK);
        }

        for i in 0..3 {
            if let Some(cell) = self.num[i][0].as_mut() {
                cell.set_white(BATTERY_WHITE);
            }
            if let Some(cell) = self.num[i][BATTERY_SHADOW].as_mut() {
                cell.set_corner_color(BATTERY_CORNER);
                cell.set_alpha(BATTERY_SHADOW_ALPHA);
            }
        }

        self.rotation = BATTERY_MIN_ROT;
        true
    }

    pub fn set_rotate(&mut self, rot: f32) {
        let angle =
            ((rot - BATTERY_ROT_SPAN) / BATTERY_ROT_SPAN) * BATTERY_ROT_SCALE + BATTERY_TEXT_BASE;
        let tens = (angle + 0.5) as i32 / 10;
        let ones = (angle + 0.5) as i32 % 10;
        let tens_tex = digit_texture(tens);
        let ones_tex = digit_texture(ones);
        for i in 0..2 {
            if let Some(cell) = self.num[BATTERY_TENS][i].as_mut() {
                cell.change_texture(&tens_tex);
            }
            if let Some(cell) = self.num[BATTERY_ONES][i].as_mut() {
                cell.change_texture(&ones_tex);
            }
        }
        self.rotation = if rot < BATTERY_MIN_ROT {
            BATTERY_ROT_LOW
        } else if rot > BATTERY_ROT_MAX {
            BATTERY_ROT_HIGH
        } else {
            // The visual angle has its own 1.4 factor, unlike the 35.0 of the text angle
            // (`d_2dnumber.cpp:174`).
            BATTERY_ROT_BASE - ((rot - BATTERY_MIN_ROT) * BATTERY_ROT_TEXT + BATTERY_TEXT_BASE)
        };
    }

    pub fn draw(&mut self) -> Vec<Quad> {
        let mut quads = Vec::new();
        let pivot = [self.battery_size[0], self.battery_size[1] / 2.0];
        let rotation = self.rotation;
        if let Some(cell) = self.battery.as_mut() {
            cell.rotate(pivot[0], pivot[1], ROTATE_Z, rotation);
        }
        if let Some(cell) = self.rule.as_ref() {
            quads.push(cell.quad(
                self.picture_pos[0] + BATTERY_RULE_OFFS[0],
                self.picture_pos[1] + BATTERY_RULE_OFFS[1],
                self.rule_size[0],
                self.rule_size[1],
            ));
        }
        if let Some(cell) = self.battery.as_ref() {
            quads.push(cell.quad(
                self.picture_pos[0] + BATTERY_OFFS[0],
                self.picture_pos[1] + BATTERY_OFFS[1],
                self.battery_size[0],
                self.battery_size[1],
            ));
        }
        if let Some(cell) = self.battery_base.as_ref() {
            quads.push(cell.quad(
                self.picture_pos[0] + BATTERY_DRAW_OFFS[0],
                self.picture_pos[1] + BATTERY_DRAW_OFFS[1],
                self.battery_base_size[0],
                self.battery_base_size[1],
            ));
        }

        let shadow = SHADOW_OFFS;

        if let Some(cell) = self.num[BATTERY_TENS][BATTERY_SHADOW].as_ref() {
            quads.push(cell.quad(
                self.num_pos[0] + shadow + BATTERY_TENS_OFFS,
                self.num_pos[1] + shadow,
                self.num_size[0],
                self.num_size[1],
            ));
        }
        if let Some(cell) = self.num[BATTERY_ONES][BATTERY_SHADOW].as_ref() {
            quads.push(cell.quad(
                self.num_pos[0] + self.num_size[0] + shadow + BATTERY_ONES_GAP,
                self.num_pos[1] + shadow,
                self.num_size[0],
                self.num_size[1],
            ));
        }
        if let Some(cell) = self.num[BATTERY_DEGREE][BATTERY_SHADOW].as_ref() {
            quads.push(cell.quad(
                self.num_pos[0] + self.num_size[0] * 2.0 + shadow,
                self.num_pos[1] + shadow,
                self.num_size[0],
                self.num_size[1],
            ));
        }

        if let Some(cell) = self.num[BATTERY_TENS][0].as_ref() {
            quads.push(cell.quad(
                self.num_pos[0] + BATTERY_TENS_OFFS,
                self.num_pos[1],
                self.num_size[0],
                self.num_size[1],
            ));
        }
        if let Some(cell) = self.num[BATTERY_ONES][0].as_ref() {
            quads.push(cell.quad(
                self.num_pos[0] + self.num_size[0] + BATTERY_ONES_GAP,
                self.num_pos[1],
                self.num_size[0],
                self.num_size[1],
            ));
        }
        if let Some(cell) = self.num[BATTERY_DEGREE][0].as_ref() {
            quads.push(cell.quad(
                self.num_pos[0] + self.num_size[0] * 2.0,
                self.num_pos[1],
                self.num_size[0],
                self.num_size[1],
            ));
        }
        quads
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpriteObject {
    pub picture: [Option<Picture>; 2],
    pub pos: [f32; 2],
    pub size: [[f32; 2]; 2],
    pub scale: f32,
    pub current_no: u8,
}

impl SpriteObject {
    pub fn init(&mut self, img1: &TimG, img2: &TimG) -> bool {
        self.picture[0] = Some(Picture::new(&img1.name));
        if self.picture[0].is_none() {
            return false;
        }
        self.size[0] = [img1.width as f32, img1.height as f32];

        self.picture[1] = Some(Picture::new(&img2.name));
        if self.picture[1].is_none() {
            return false;
        }
        self.size[1] = [img2.width as f32, img2.height as f32];
        self.scale = 1.0;
        self.current_no = 0;

        true
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.pos = [x, y];
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    pub fn on_before_tex(&mut self) {
        self.current_no = 0;
    }

    pub fn off_before_tex(&mut self) {
        self.current_no = 1;
    }

    pub fn draw(&self) -> Vec<Quad> {
        let slot = usize::from(self.current_no);
        match self.picture.get(slot).and_then(Option::as_ref) {
            Some(cell) => vec![cell.quad(
                self.pos[0],
                self.pos[1],
                self.size[slot][0] * self.scale,
                self.size[slot][1] * self.scale,
            )],
            None => Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OutFont {
    pub font_size: [u32; 2],
    pub pics: [Option<Picture>; OUT_FONT_ICONS],
    pub text_box: Option<TextBox>,
    pub ruby_box: Option<TextBox>,
    pub font: usize,
    pub ruby_font: Option<usize>,
    pub external: bool,
    pub string: TextBuf<OUT_FONT_BUF>,
    pub ruby_string: TextBuf<OUT_FONT_BUF>,
    pub pos_top_left: [f32; 2],
    pub offset: [[f32; 2]; OUT_FONT_ICONS],
    pub align: [[f32; 2]; OUT_FONT_ICONS],
    pub width: f32,
    pub size_x: f32,
    pub char_space: f32,
    pub shrink: u8,
    pub line: [i16; OUT_FONT_ICONS],
    pub timer: [i16; OUT_FONT_ICONS],
    pub line_count: i16,
    pub icon_num: u8,
    pub icon_no: [u8; OUT_FONT_ICONS],
    pub ruby_run: TextBuf<OUT_FONT_RUBY_BUF>,
    pub ruby_width: f32,
    pub ruby_repeat: i32,
}

impl Default for OutFont {
    fn default() -> Self {
        Self::new()
    }
}

impl OutFont {
    pub fn new() -> Self {
        Self {
            font_size: [0, 0],
            pics: [None, None, None],
            text_box: None,
            ruby_box: None,
            font: 0,
            ruby_font: None,
            external: false,
            string: TextBuf::new(),
            ruby_string: TextBuf::new(),
            pos_top_left: [0.0, 0.0],
            offset: [[0.0, 0.0]; OUT_FONT_ICONS],
            align: [[0.0, 0.0]; OUT_FONT_ICONS],
            width: 0.0,
            size_x: 0.0,
            char_space: 0.0,
            shrink: 0,
            line: [0; OUT_FONT_ICONS],
            timer: [0; OUT_FONT_ICONS],
            line_count: 0,
            icon_num: 0,
            icon_no: [0; OUT_FONT_ICONS],
            ruby_run: TextBuf::new(),
            ruby_width: 0.0,
            ruby_repeat: 0,
        }
    }

    pub fn set_left_up_pos(&mut self, x: f32, y: f32) {
        self.pos_top_left[0] = x;
        self.pos_top_left[1] = y;
    }

    pub fn initial(&mut self) {
        self.width = 0.0;
        self.icon_num = 0;
        self.pos_top_left = [0.0, 0.0];
        self.line_count = 0;

        if let Some(box_) = self.text_box.as_mut() {
            box_.show();
            self.font_size = box_.font_size;
            self.size_x = self.font_size[0] as f32;
            self.char_space = box_.char_space;
        }

        for i in 0..OUT_FONT_ICONS {
            let Some(cell) = self.pics[i].as_mut() else {
                continue;
            };
            self.offset[i] = [0.0, 0.0];
            self.align[i] = [0.0, 0.0];
            cell.hide();
            cell.rotate(0.0, 0.0, ROTATE_Z, 0.0);
            if self.shrink != 0 {
                cell.resize(self.font_size[0] as f32, self.font_size[1] as f32);
            } else {
                cell.resize(self.size_x, self.size_x);
            }
            self.timer[i] = 0;
            self.icon_no[i] = ICON_A_BUTTON;
            self.line[i] = -1;
        }
    }

    pub fn set_pane(
        &mut self,
        font: usize,
        text_box: Option<TextBox>,
        pic1: Option<Picture>,
        pic2: Option<Picture>,
        pic3: Option<Picture>,
    ) {
        self.font = font;
        self.text_box = text_box;
        self.pics = [pic1, pic2, pic3];
        self.external = false;
        self.initial();
        self.attach_pics();
    }

    pub fn set_pane_ex(
        &mut self,
        font: usize,
        text_box: Option<TextBox>,
        pic1: Option<Picture>,
        pic2: Option<Picture>,
        pic3: Option<Picture>,
    ) {
        self.font = font;
        self.text_box = text_box;
        self.pics = [pic1, pic2, pic3];
        self.external = true;
        self.initial();
        self.attach_pics();
    }

    fn attach_pics(&mut self) {
        for i in 0..OUT_FONT_ICONS {
            if let Some(cell) = self.pics[i].as_mut() {
                cell.change_texture(OUT_FONT_PIC_TEX);
                cell.append(OUT_FONT_PIC_TEX, 1.0);
            }
        }
    }

    pub fn set_ruby(&mut self, font: usize, ruby_box: Option<TextBox>) {
        self.ruby_font = Some(font);
        self.ruby_box = ruby_box;
    }

    pub fn set_ruby_ex(&mut self, font: usize, ruby_box: Option<TextBox>) {
        self.ruby_font = Some(font);
        self.ruby_box = ruby_box;
    }

    pub fn char_width(&self, fonts: &FontBank, code: i32) -> f32 {
        let Some(font) = fonts.get(self.font) else {
            return self.char_space;
        };
        let f1 = font.char_width(code as u16) as f32;
        let f0 = self.size_x / font.width() as f32;
        self.char_space + f1 * f0
    }

    pub fn ruby_char_width(&self, fonts: &FontBank, code: i32) -> f32 {
        let Some(box_) = self.ruby_box.as_ref() else {
            return self.char_space;
        };
        let Some(font) = fonts.get(self.ruby_font.unwrap_or(self.font)) else {
            return self.char_space;
        };
        let f30 = font.char_width(code as u16) as f32;
        let f0 = box_.font_size[0] as f32 / font.width() as f32;
        self.char_space + f30 * f0
    }

    pub fn iconset(&mut self, i_icon_no: u8, dst: &mut TextBuf<OUT_FONT_BUF>) -> f32 {
        let mut var_f31 = 0.0f32;
        if (self.icon_num as usize) < OUT_FONT_ICONS {
            let n = self.icon_num as usize;
            self.offset[n][0] = self.width;
            if let Some(cell) = self.pics[n].as_mut() {
                cell.show();
                cell.remove();
                if let Some(name) = fop_msgm_out_font_tex(i32::from(i_icon_no)) {
                    cell.append(name, 1.0);
                }
                cell.set_blend_ratio(0.0, 1.0, 1.0, 1.0);
                let white = fop_msgm_out_font_color_white(i32::from(i_icon_no)).unwrap_or([0; 4]);
                cell.set_white(u32::from_be_bytes(white));
                cell.set_black(0);
            }
            self.icon_no[n] = i_icon_no;
            self.line[n] = self.line_count;

            match i_icon_no {
                ICON_ARROW_LEFT => {
                    self.offset[n][1] += self.size_x;
                    if let Some(cell) = self.pics[n].as_mut() {
                        cell.rotate(0.0, 0.0, ROTATE_Z, 270.0);
                    }
                }
                ICON_ARROW_RIGHT => {
                    self.offset[n][0] += self.size_x;
                    if let Some(cell) = self.pics[n].as_mut() {
                        cell.rotate(0.0, 0.0, ROTATE_Z, 90.0);
                    }
                }
                ICON_ARROW_UP => {
                    self.offset[n][1] += self.size_x;
                    self.offset[n][0] += self.size_x;
                    if let Some(cell) = self.pics[n].as_mut() {
                        cell.rotate(0.0, 0.0, ROTATE_Z, 180.0);
                    }
                }
                _ => {}
            }

            var_f31 = self.size_x + self.char_space;
            let tag = format!("\u{1b}CR[{}]", (var_f31 + 0.5) as i32);
            dst.push_str(&tag);

            self.icon_num += 1;
        }
        var_f31
    }

    fn icon(&mut self, i_icon_no: u8, dst: &mut TextBuf<OUT_FONT_BUF>) {
        let step = self.iconset(i_icon_no, dst);
        self.width += step;
    }

    fn anime_piece(&mut self, timing: &StickTiming, slot: usize, piece: i16) {
        let period = i32::from(timing.period());
        let at = i32::from(timing.field_0x88) + period * i32::from(piece);
        let after_x = i32::from(timing.field_0x87) + at;
        let after_y = i32::from(timing.field_0x89) + after_x;
        let after_loop = i32::from(timing.field_0x87) + after_y;
        let timer = i32::from(self.timer[slot]);
        let Some(cell) = self.pics[slot].as_mut() else {
            return;
        };
        if timer < at {
            cell.set_blend_ratio(0.0, 1.0, 1.0, 1.0);
        } else if timer < after_x {
            let t = fop_msgm_value_increase(period, timer - at, 0).unwrap_or_default();
            cell.set_blend_ratio(t, 1.0 - t, 1.0, 1.0);
        } else if timer < after_y {
            cell.set_blend_ratio(1.0, 0.0, 1.0, 1.0);
        } else if timer < after_loop {
            let t = fop_msgm_value_increase(period, timer - after_y, 0).unwrap_or_default();
            cell.set_blend_ratio(1.0 - t, t, 1.0, 1.0);
        }
    }

    pub fn out_font_stick_anime_piece(&mut self, timing: &StickTiming, slot: usize, piece: i16) {
        self.anime_piece(timing, slot, piece);
    }

    pub fn out_font_stick_anime1(&mut self, timing: &StickTiming, slot: usize) {
        let period = i32::from(timing.period());
        let timer = i32::from(self.timer[slot]);
        if timer < period {
            self.anime_piece(timing, slot, 0);
            if self.shrink != 0 {
                self.align[slot] = [self.font_size[0] as f32, 0.0];
                if let Some(cell) = self.pics[slot].as_mut() {
                    cell.resize(self.font_size[0] as f32, self.font_size[1] as f32);
                }
            } else {
                self.align[slot] = [self.size_x, 0.0];
            }
        } else if timer < period * 2 {
            self.anime_piece(timing, slot, 1);
            if self.shrink != 0 {
                self.align[slot] = [self.font_size[0] as f32, 0.0];
                if let Some(cell) = self.pics[slot].as_mut() {
                    cell.resize(self.font_size[1] as f32, self.font_size[0] as f32);
                }
            } else {
                self.align[slot] = [self.size_x, 0.0];
            }
        } else if timer < period * 3 {
            self.anime_piece(timing, slot, 2);
            if self.shrink != 0 {
                self.align[slot] = [self.font_size[0] as f32, self.font_size[1] as f32];
                if let Some(cell) = self.pics[slot].as_mut() {
                    cell.resize(self.font_size[0] as f32, self.font_size[1] as f32);
                }
            } else {
                self.align[slot] = [self.size_x, self.size_x];
            }
        } else if timer < period * 4 {
            self.anime_piece(timing, slot, 3);
            if self.shrink != 0 {
                self.align[slot] = [0.0, self.font_size[1] as f32];
                if let Some(cell) = self.pics[slot].as_mut() {
                    cell.resize(self.font_size[1] as f32, self.font_size[0] as f32);
                }
            } else {
                self.align[slot] = [0.0, self.size_x];
            }
        }

        if period != 0 && timer % period == 0 {
            let angle = (timer / period) as f32 * -90.0;
            if let Some(cell) = self.pics[slot].as_mut() {
                cell.rotate(0.0, 0.0, ROTATE_Z, angle);
            }
        }

        self.timer[slot] += 1;
        if i32::from(self.timer[slot]) >= period * 4 {
            self.timer[slot] = 0;
        }
    }

    pub fn out_font_stick_anime2(&mut self, timing: &StickTiming, slot: usize, face: u8) {
        let period = i32::from(timing.period());
        let timer = i32::from(self.timer[slot]);
        if timer < period {
            self.anime_piece(timing, slot, 0);
            match face {
                0 => {
                    if self.shrink != 0 {
                        self.align[slot] = [self.font_size[0] as f32, 0.0];
                        if let Some(cell) = self.pics[slot].as_mut() {
                            cell.resize(self.font_size[0] as f32, self.font_size[1] as f32);
                        }
                    } else {
                        self.align[slot] = [0.0, 0.0];
                    }
                }
                1 => {
                    if self.shrink != 0 {
                        self.align[slot] = [self.font_size[0] as f32, 0.0];
                        if let Some(cell) = self.pics[slot].as_mut() {
                            cell.resize(self.font_size[1] as f32, self.font_size[0] as f32);
                        }
                    } else {
                        self.align[slot] = [self.size_x, 0.0];
                    }
                }
                2 => {
                    if self.shrink != 0 {
                        self.align[slot] = [self.font_size[0] as f32, self.font_size[1] as f32];
                        if let Some(cell) = self.pics[slot].as_mut() {
                            cell.resize(self.font_size[0] as f32, self.font_size[1] as f32);
                        }
                    } else {
                        self.align[slot] = [self.size_x, self.size_x];
                    }
                }
                3 => {
                    if self.shrink != 0 {
                        self.align[slot] = [0.0, self.font_size[1] as f32];
                        if let Some(cell) = self.pics[slot].as_mut() {
                            cell.resize(self.font_size[1] as f32, self.font_size[0] as f32);
                        }
                    } else {
                        self.align[slot] = [0.0, self.size_x];
                    }
                }
                _ => {}
            }
        }

        if let Some(cell) = self.pics[slot].as_mut() {
            cell.rotate(0.0, 0.0, ROTATE_Z, f32::from(face) * -90.0);
        }

        self.timer[slot] += 1;
        if i32::from(self.timer[slot]) >= period {
            self.timer[slot] = 0;
        }
    }

    pub fn out_font_stick_anime3(&mut self, timing: &StickTiming, slot: usize, face: u8) {
        let period = i32::from(timing.period());
        let timer = i32::from(self.timer[slot]);
        let mut var_r0: i32;
        if face == 0 {
            if timer < period {
                self.anime_piece(timing, slot, 0);
                if self.shrink != 0 {
                    self.align[slot] = [self.font_size[0] as f32, 0.0];
                    if let Some(cell) = self.pics[slot].as_mut() {
                        cell.resize(self.font_size[1] as f32, self.font_size[0] as f32);
                    }
                } else {
                    self.align[slot] = [self.size_x, 0.0];
                }
                var_r0 = 1;
            } else if timer < period * 2 {
                self.anime_piece(timing, slot, 1);
                if self.shrink != 0 {
                    self.align[slot] = [0.0, self.font_size[1] as f32];
                    if let Some(cell) = self.pics[slot].as_mut() {
                        cell.resize(self.font_size[1] as f32, self.font_size[0] as f32);
                    }
                } else {
                    self.align[slot] = [0.0, self.size_x];
                }
                var_r0 = 3;
            } else {
                var_r0 = 0;
            }
        } else if timer < period {
            self.anime_piece(timing, slot, 0);
            if self.shrink != 0 {
                self.align[slot] = [self.font_size[0] as f32, self.font_size[1] as f32];
                if let Some(cell) = self.pics[slot].as_mut() {
                    cell.resize(self.font_size[0] as f32, self.font_size[1] as f32);
                }
            } else {
                self.align[slot] = [0.0, 0.0];
            }
            var_r0 = 0;
        } else if timer < period * 2 {
            self.anime_piece(timing, slot, 1);
            if self.shrink != 0 {
                self.align[slot] = [self.font_size[0] as f32, self.font_size[1] as f32];
                if let Some(cell) = self.pics[slot].as_mut() {
                    cell.resize(self.font_size[1] as f32, self.font_size[0] as f32);
                }
            } else {
                self.align[slot] = [self.size_x, self.size_x];
            }
            var_r0 = 2;
        } else {
            var_r0 = 0;
        }

        if let Some(cell) = self.pics[slot].as_mut() {
            cell.rotate(0.0, 0.0, ROTATE_Z, var_r0 as f32 * -90.0);
        }

        self.timer[slot] += 1;
        if i32::from(self.timer[slot]) >= period * 2 {
            self.timer[slot] = 0;
        }
    }

    pub fn move_fonts(&mut self, timing: &StickTiming) {
        for i in 0..self.icon_num as usize {
            match self.icon_no[i] {
                ICON_MAIN_STICK => self.out_font_stick_anime1(timing, i),
                ICON_MAIN_STICK_UP => self.out_font_stick_anime2(timing, i, 1),
                ICON_MAIN_STICK_DOWN => self.out_font_stick_anime2(timing, i, 3),
                ICON_MAIN_STICK_LEFT => self.out_font_stick_anime2(timing, i, 0),
                ICON_MAIN_STICK_RIGHT => self.out_font_stick_anime2(timing, i, 2),
                ICON_MAIN_STICK_UP_DOWN => self.out_font_stick_anime3(timing, i, 0),
                ICON_MAIN_STICK_LEFT_RIGHT => self.out_font_stick_anime3(timing, i, 1),
                _ => {}
            }
        }

        for i in 0..OUT_FONT_ICONS {
            let x = self.pos_top_left[0] + self.offset[i][0] + self.align[i][0];
            let y = (self.pos_top_left[1] + self.offset[i][1] + self.align[i][1]) - 1.0;
            if let Some(cell) = self.pics[i].as_mut() {
                cell.move_to(x, y);
            }
        }

        if let Some(box_) = self.text_box.as_mut() {
            box_.move_to(self.pos_top_left[0], self.pos_top_left[1]);
        }
    }

    pub fn set_alpha(&mut self, alpha: u8) {
        for i in 0..OUT_FONT_ICONS {
            if let Some(cell) = self.pics[i].as_mut() {
                cell.set_alpha(alpha);
            }
        }
        if let Some(box_) = self.text_box.as_mut() {
            box_.set_alpha(alpha);
        }
    }

    pub fn draw(&self) -> Vec<Quad> {
        let mut quads = Vec::new();
        for cell in self.pics.iter().flatten() {
            if !cell.visible {
                continue;
            }
            quads.push(cell.quad(cell.pos[0], cell.pos[1], cell.size[0], cell.size[1]));
        }
        quads
    }

    fn icon_for(code: u32) -> Option<u8> {
        match code {
            x if x == u32::from(CTRL_A_BUTTON) => Some(ICON_A_BUTTON),
            x if x == u32::from(CTRL_B_BUTTON) => Some(ICON_B_BUTTON),
            x if x == u32::from(CTRL_C_STICK) => Some(ICON_C_STICK),
            x if x == u32::from(CTRL_L_BUTTON) => Some(ICON_L_BUTTON),
            x if x == u32::from(CTRL_R_BUTTON) => Some(ICON_R_BUTTON),
            x if x == u32::from(CTRL_X_BUTTON) => Some(ICON_X_BUTTON),
            x if x == u32::from(CTRL_Y_BUTTON) => Some(ICON_Y_BUTTON),
            x if x == u32::from(CTRL_Z_BUTTON) => Some(ICON_Z_BUTTON),
            x if x == u32::from(CTRL_DPAD) => Some(ICON_DPAD),
            x if x == u32::from(CTRL_MAIN_STICK) => Some(ICON_MAIN_STICK),
            x if x == u32::from(CTRL_ARROW_LEFT) => Some(ICON_ARROW_LEFT),
            x if x == u32::from(CTRL_ARROW_RIGHT) => Some(ICON_ARROW_RIGHT),
            x if x == u32::from(CTRL_ARROW_UP) => Some(ICON_ARROW_UP),
            x if x == u32::from(CTRL_ARROW_DOWN) => Some(ICON_ARROW_DOWN),
            x if x == u32::from(CTRL_MAIN_STICK_UP) => Some(ICON_MAIN_STICK_UP),
            x if x == u32::from(CTRL_MAIN_STICK_DOWN) => Some(ICON_MAIN_STICK_DOWN),
            x if x == u32::from(CTRL_MAIN_STICK_LEFT) => Some(ICON_MAIN_STICK_LEFT),
            x if x == u32::from(CTRL_MAIN_STICK_RIGHT) => Some(ICON_MAIN_STICK_RIGHT),
            x if x == u32::from(CTRL_MAIN_STICK_UP_DOWN) => Some(ICON_MAIN_STICK_UP_DOWN),
            x if x == u32::from(CTRL_MAIN_STICK_LEFT_RIGHT) => Some(ICON_MAIN_STICK_LEFT_RIGHT),
            x if x == u32::from(CTRL_FLASHING_A_BUTTON) => Some(ICON_FLASHING_A_BUTTON),
            x if x == u32::from(CTRL_HEART) => Some(ICON_HEART),
            x if x == u32::from(CTRL_MUSIC_NOTE) => Some(ICON_MUSIC_NOTE),
            _ => None,
        }
    }

    pub fn message_set<A: MsgArchive + ?Sized>(
        &mut self,
        archive: &A,
        fonts: &FontBank,
        clt1: &[u8],
        clear_count: u8,
        i_msg_no: u32,
    ) {
        let mut get = ItemMsgGet::default();
        let message = get
            .get_mesg_header(archive, clear_count, i_msg_no)
            .and_then(|head| get.get_message(head))
            .map(|text| text.to_vec())
            .unwrap_or_default();

        let mut line = TextBuf::<OUT_FONT_BUF>::new();
        let mut ruby_line = TextBuf::<OUT_FONT_BUF>::new();

        self.initial();
        self.string.clear();
        ruby_line.clear();
        self.ruby_string.clear();

        let mut at = 0usize;
        while at < message.len() && message[at] != 0 {
            if message[at] == 0x1A {
                let b1 = *message.get(at + 1).unwrap_or(&0);
                let b2 = *message.get(at + 2).unwrap_or(&0);
                let b3 = *message.get(at + 3).unwrap_or(&0);
                let b4 = *message.get(at + 4).unwrap_or(&0);
                let code = (u32::from(b2) << 16) | (u32::from(b3) << 8) | u32::from(b4);
                if code == (0xFF0000 | u32::from(SPCL_COLOR)) {
                    let color = fop_msgm_get_color_table(
                        clt1,
                        u16::from(*message.get(at + 1 + 4).unwrap_or(&0)),
                    );
                    let tag = format!("\u{1b}CC[{color:08x}]\u{1b}GM[0]");
                    line.push_str(&tag);
                } else if code == (0xFF0000 | u32::from(SPCL_RUBY)) {
                    if self.ruby_font.is_some() {
                        let temp = b1;
                        if temp != 5 {
                            let count = i32::from(temp) - 6;
                            let mut i = 0i32;
                            let mut src = at + 1 + 5;
                            self.ruby_run.clear();
                            self.ruby_width = 0.0;
                            self.ruby_repeat = i32::from(*message.get(at + 1 + 4).unwrap_or(&0));
                            while i < count {
                                let lead = *message.get(src).unwrap_or(&0);
                                let trail = *message.get(src + 1).unwrap_or(&0);
                                self.ruby_run.push(lead);
                                self.ruby_run.push(trail);
                                src += 2;
                                i += 2;
                                let ruby_code = i32::from(trail) << 8 | i32::from(lead);
                                let advance = self.ruby_char_width(fonts, ruby_code);
                                self.ruby_width += advance;
                            }
                        }
                    }
                } else if let Some(icon) = Self::icon_for(code) {
                    self.icon(icon, &mut line);
                }
                at = (at as i32 + i32::from(b1) - 1).max(0) as usize;
            } else {
                let ch = message[at];
                line.push(ch);
                at += 1;

                if ch == b'\n' {
                    let mut tag = TextBuf::<OUT_FONT_TAG_BUF>::new();
                    let box_width = self.text_box.as_ref().map_or(0.0, TextBox::width);
                    let temp_r24 = ((box_width - self.width) / 2.0) as i32;
                    if temp_r24 > 0 {
                        tag.push_str(&format!("\u{1b}CR[{temp_r24}]"));
                        for i in 0..OUT_FONT_ICONS {
                            if self.line_count == self.line[i] {
                                self.offset[i][0] += temp_r24 as f32;
                            }
                        }
                    }
                    self.string.cat(tag.as_bytes());
                    self.string.cat(line.as_bytes());
                    self.line_count += 1;
                    self.width = 0.0;
                    line.clear();

                    if self.ruby_font.is_some() {
                        self.ruby_string.cat(tag.as_bytes());
                        self.ruby_string.cat(ruby_line.as_bytes());
                        self.ruby_string.push_str("\n");
                        ruby_line.clear();
                    }
                } else {
                    let advance = self.char_width(fonts, i32::from(ch));
                    self.width += advance;
                }
            }
        }

        let mut var_f29 = 0.0f32;
        if self.shrink == 0 {
            let height = self.text_box.as_ref().map_or(0.0, TextBox::height);
            let line_space = self.text_box.as_ref().map_or(0.0, |b| b.line_space);
            var_f29 = ((height - self.size_x) - f32::from(self.line_count) * line_space) / 2.0;
        }

        let mut tag = TextBuf::<OUT_FONT_TAG_BUF>::new();
        let box_width = self.text_box.as_ref().map_or(0.0, TextBox::width);
        let line_space = self.text_box.as_ref().map_or(0.0, |b| b.line_space);
        let temp_r30 = ((box_width - self.width) / 2.0) as i32;
        if temp_r30 > 0 {
            tag.push_str(&format!("\u{1b}CR[{temp_r30}]"));
            for i in 0..OUT_FONT_ICONS {
                let slot = self.line[i];
                if self.line_count == slot {
                    self.offset[i][0] += temp_r30 as f32;
                }
                if slot != -1 {
                    self.offset[i][1] += var_f29 + f32::from(slot) * line_space;
                }
            }
        }

        self.string.cat(tag.as_bytes());
        self.string.cat(line.as_bytes());

        let string = self.string.as_bytes().to_vec();
        if let Some(box_) = self.text_box.as_mut() {
            box_.shift_set(0.0, var_f29);
            if !self.external {
                box_.set_string(&string);
            }
        }

        if self.ruby_box.is_some() {
            self.ruby_string.cat(tag.as_bytes());
            self.ruby_string.cat(ruby_line.as_bytes());
            let ruby_string = self.ruby_string.as_bytes().to_vec();
            if let Some(box_) = self.ruby_box.as_mut() {
                box_.shift_set(0.0, var_f29);
                if !self.external {
                    box_.set_string(&ruby_string);
                }
            }
        }

        if self.shrink != 0 && self.width != 0.0 && box_width < self.width {
            self.font_size[0] = (self.font_size[0] as f32 * (box_width / self.width)) as u32;
            if let Some(box_) = self.text_box.as_mut() {
                box_.font_size = self.font_size;
            }
            self.message_set(archive, fonts, clt1, clear_count, i_msg_no);
        }
    }
}

#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct Numbers {
    pub timer: NumberSprite,
    pub rupee: NumberSprite,
}

impl Numbers {
    pub fn init_timer(&mut self, x: i16, y: i16, w: i16, h: i16) -> bool {
        self.timer.init(TIMER_DIGITS, x, y, w, h, TIMER_FLAG)
    }

    pub fn init_rupee(&mut self, x: i16, y: i16, w: i16, h: i16, flag: u8) -> bool {
        self.rupee.init(RUPEE_DIGITS, x, y, w, h, flag)
    }

    pub fn draw(&mut self) -> Vec<Quad> {
        let mut quads = self.timer.draw();
        quads.append(&mut self.rupee.draw());
        quads
    }
}

pub fn control(mut numbers: ResMut<Numbers>, timer: Res<IsleTimer>) {
    let digits = timer.timer_digits();
    numbers.timer.set_digits(&digits);
    numbers.rupee.set_digits(&timer.rupee_digits);
}

pub fn init(world: &mut World) {
    world.init_resource::<Numbers>();
}
