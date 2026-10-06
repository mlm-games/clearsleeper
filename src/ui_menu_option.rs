use bevy_ecs::prelude::*;

use crate::j2d::{Kind, Screen, ScreenPane};
use crate::sim::vibration::{RND_MOD, RND_MUL, RND_SCALE, RND_SEED, Vibration};

pub const SCREEN_LABEL: &str = "option.blo";
pub const SCREEN_ARCHIVE: &str = "files/res/Msg/optres.arc";
pub const ICON_COUNT: usize = 0xF;
pub const LABEL_COUNT: usize = 15;
pub const CURSOR_PARTS: usize = 6;
pub const CURSOR_BAR_PARTS: usize = 2;
pub const TYPE_PARTS: usize = 2;
pub const SOUND_PARTS: usize = 3;
pub const MOVE_TABLE: [i32; 6] = [0, 2, 8, 18, 8, 2];
pub const OPEN_FADE: u8 = 7;
pub const OPEN_DONE: u8 = 0xE;
pub const CLOSE_FADE: u8 = 10;
pub const NOTE_LINE_WIDTH: i32 = 0x1D0;
pub const NOTE_SEND_SPEED: i32 = 2;
pub const NOTE_SPACE_TIMER: i32 = 0;
pub const SCALE_FADE: i32 = 196;
pub const BAR_OFFSET: f32 = 6.0;
pub const BAR_PERIOD: i16 = 20;
pub const NOTE_TITLE_ANGLE: f32 = 20.0;
pub const NOTE_DY: f32 = 35.0;
pub const TITLE_DY: f32 = -50.0;
pub const NOTE_TAIL_FRAMES: u8 = 4;
pub const NOTE_HEAD_FRAMES: u8 = 10;
pub const SOUND_MODE: [u32; 3] = [0, 1, 2];

pub const SE_MENU_CURSOR: u32 = 0x080E;
pub const SE_MENU_OPTION_OUT: u32 = 0x084D;
pub const SE_MENU_OPTION_STRING: u32 = 0x084E;
pub const SE_MENU_OPTION_SWITCH: u32 = 0x084F;

pub const MSG_OPTION_NOTE: u32 = 0x262;
pub const MSG_TITLE: u32 = 0x19D;
pub const MSG_ATTENTION_0: u32 = 0x19B;
pub const MSG_ATTENTION_1: u32 = 0x1A9;
pub const MSG_VIBRATION_0: u32 = 0x1D5;
pub const MSG_VIBRATION_1: u32 = 0x1D4;
pub const MSG_SOUND_0: u32 = 0x1D7;
pub const MSG_SOUND_1: u32 = 0x1D8;
pub const MSG_SOUND_2: u32 = 0x1EB;
pub const MSG_ROW_0: u32 = 0x19A;
pub const MSG_ROW_1: u32 = 0x19A;
pub const MSG_ROW_2: u32 = 0x1D1;
pub const MSG_ROW_3: u32 = 0x1D6;

pub const ICON_OUT_TEXTURE: &str = "font_07_02.bti";
pub const ICON_BLEND_TEXTURE: &str = "font_00.bti";

pub const TEXT_ROW: [&str; LABEL_COUNT] = [
    "ft00", "ft01", "ft02", "ft03", "ft04", "ft05", "ft06", "ft07", "ft08", "ft09", "ft10",
    "ft11", "ft12", "ft13", "ft14",
];
pub const TEXT_SHADOW: [&str; LABEL_COUNT] = [
    "fd00", "fd01", "fd02", "fd03", "fd04", "fd05", "fd06", "fd07", "fd08", "fd09", "fd10",
    "fd11", "fd12", "fd13", "fd14",
];

pub const STICK_MAX_VALUE: f32 = 0.9;
pub const STICK_MIN_VALUE: f32 = 0.5;
pub const STICK_HALF_DEAD: i16 = 0x2000;

pub const RUMBLE_PATTERN: u8 = 0;
pub const RUMBLE_STRENGTH: u8 = 0x3c;
pub const RUMBLE_LOOP_ONCE: bool = true;

pub const ATTENTION_RANGE: u8 = 1;
pub const RUBY_RANGE: u8 = 1;
pub const VIBRATION_RANGE: u8 = 1;
pub const SOUND_RANGE: u8 = 2;

pub const ATTENTION_DEFAULT: u8 = 0;
pub const RUBY_DEFAULT: u8 = 1;
pub const VIBRATION_DEFAULT: u8 = 1;
pub const SOUND_DEFAULT_MONO: u8 = 0;
pub const SOUND_DEFAULT_STEREO: u8 = 1;

pub const row_attention: u8 = 0;
pub const row_ruby: u8 = 1;
pub const row_vibration: u8 = 2;
pub const row_sound: u8 = 3;
pub const ROW_COUNT: u8 = 4;

pub const OPTION_NOTE_RAMP: i16 = 10;
pub const OPTION_NOTE_STEP: i16 = 4;

pub const QUIT_NONE: u8 = 0;
pub const QUIT_OPEN: u8 = 1;
pub const QUIT_CLOSING: u8 = 2;
pub const QUIT_CLOSED: u8 = 3;

pub const EXIT_NONE: u8 = 0;
pub const EXIT_SAVED: u8 = 1;
pub const EXIT_DISCARDED: u8 = 2;

pub const MSG_BUFFER: usize = 20;
pub const MSG_BUFFER_COUNT: usize = 4;

pub const note_attention: u8 = 0;
pub const note_sound: u8 = 1;
pub const note_vibration: u8 = 2;

pub const STICK_UP: u8 = 4;
pub const STICK_DOWN: u8 = 8;
pub const STICK_RIGHT: u8 = 2;
pub const STICK_LEFT: u8 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelSlot {
    Attention0,
    Attention1,
    Vibration0,
    Vibration1,
    Sound0,
    Sound1,
    Sound2,
    Row0,
    Row2,
    Row3,
    Row4,
    Title,
}

pub const LABEL_SLOTS: [LabelSlot; 10] = [
    LabelSlot::Attention0,
    LabelSlot::Attention1,
    LabelSlot::Vibration0,
    LabelSlot::Vibration1,
    LabelSlot::Sound0,
    LabelSlot::Sound1,
    LabelSlot::Sound2,
    LabelSlot::Row0,
    LabelSlot::Row2,
    LabelSlot::Row3,
];

pub const ROW_LABEL_SLOTS: [LabelSlot; 4] = [
    LabelSlot::Row0,
    LabelSlot::Row2,
    LabelSlot::Row3,
    LabelSlot::Row4,
];

pub const ROW_LABEL_MESSAGES: [u32; 4] = [MSG_ROW_0, MSG_ROW_1, MSG_ROW_2, MSG_ROW_3];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionRow {
    Attention,
    Ruby,
    Vibration,
    Sound,
}

impl OptionRow {
    pub fn index(self) -> u8 {
        match self {
            OptionRow::Attention => row_attention,
            OptionRow::Ruby => row_ruby,
            OptionRow::Vibration => row_vibration,
            OptionRow::Sound => row_sound,
        }
    }

    pub fn from_index(index: u8) -> Self {
        match index {
            0 => OptionRow::Attention,
            1 => OptionRow::Ruby,
            2 => OptionRow::Vibration,
            3 => OptionRow::Sound,
            _ => OptionRow::Attention,
        }
    }

    pub fn range(self) -> u8 {
        match self {
            OptionRow::Attention => ATTENTION_RANGE,
            OptionRow::Ruby => RUBY_RANGE,
            OptionRow::Vibration => VIBRATION_RANGE,
            OptionRow::Sound => SOUND_RANGE,
        }
    }

    pub fn default(self) -> u8 {
        match self {
            OptionRow::Attention => ATTENTION_DEFAULT,
            OptionRow::Ruby => RUBY_DEFAULT,
            OptionRow::Vibration => VIBRATION_DEFAULT,
            OptionRow::Sound => SOUND_DEFAULT_STEREO,
        }
    }

    pub fn value_count(self) -> usize {
        usize::from(self.range()) + 1
    }
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerConfig {
    pub ruby: u8,
    pub sound: u8,
    pub attention: u8,
    pub vibration: u8,
    pub now_vibration: u8,
    pub sound_mode: u32,
    pub ruby_unused: u8,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            ruby: RUBY_DEFAULT,
            sound: SOUND_DEFAULT_STEREO,
            attention: ATTENTION_DEFAULT,
            vibration: VIBRATION_DEFAULT,
            now_vibration: VIBRATION_DEFAULT,
            sound_mode: 0,
            ruby_unused: 0,
        }
    }
}

impl PlayerConfig {
    pub fn init(&mut self, system_sound_mode: u32) {
        self.ruby = RUBY_DEFAULT;
        if system_sound_mode == 0 {
            self.sound = SOUND_DEFAULT_MONO;
            self.sound_mode = 0;
        } else {
            self.sound = SOUND_DEFAULT_STEREO;
            self.sound_mode = 1;
        }
        self.attention = ATTENTION_DEFAULT;
        self.vibration = VIBRATION_DEFAULT;
        self.now_vibration = self.vibration;
    }

    pub fn check_vibration(&self, rumble_supported: bool) -> i32 {
        if rumble_supported {
            i32::from(self.now_vibration)
        } else {
            0
        }
    }
}

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rumbles {
    pub supported: bool,
    pub pattern: [u8; 4],
    pub length: i16,
    pub loop_once: bool,
    pub strength: u8,
    pub active: bool,
}

impl Rumbles {
    pub fn start_patterned(&mut self, pattern: u8, loop_once: bool, strength: u8) {
        self.pattern = [pattern, 0, 0, 0];
        self.length = 10;
        self.loop_once = loop_once;
        self.strength = strength;
        self.active = true;
    }

    pub fn stop(&mut self) {
        self.active = false;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct F2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug)]
pub struct Pane {
    pub tag: &'static str,
    pub kind: Kind,
    pub found: bool,
    pub visible: bool,
    pub pos_top_left_orig: F2,
    pub pos_top_left: F2,
    pub pos_center_orig: F2,
    pub pos_center: F2,
    pub size_orig: F2,
    pub size: F2,
    pub init_alpha: u8,
    pub now_alpha: u8,
    pub user_area: i16,
    pub texture: Option<&'static str>,
    pub appended: Vec<&'static str>,
    pub blend: [f32; 4],
    pub rotation: u16,
    pub rotate_x: f32,
    pub rotate_y: f32,
    pub text: String,
    pub font_size_x: f32,
    pub font_size_y: f32,
    pub char_space: f32,
    pub line_space: f32,
    pub shift_x: f32,
    pub shift_y: f32,
    pub color: u32,
}

impl Default for Pane {
    fn default() -> Self {
        Self {
            tag: "",
            kind: Kind::Pane,
            found: false,
            visible: true,
            pos_top_left_orig: F2::default(),
            pos_top_left: F2::default(),
            pos_center_orig: F2::default(),
            pos_center: F2::default(),
            size_orig: F2::default(),
            size: F2::default(),
            init_alpha: 0xFF,
            now_alpha: 0,
            user_area: 0,
            texture: None,
            appended: Vec::new(),
            blend: [0.0, 1.0, 1.0, 1.0],
            rotation: 0,
            rotate_x: 0.0,
            rotate_y: 0.0,
            text: String::new(),
            font_size_x: 0.0,
            font_size_y: 0.0,
            char_space: 0.0,
            line_space: 0.0,
            shift_x: 0.0,
            shift_y: 0.0,
            color: 0xFFFF_FFFF,
        }
    }
}

impl Pane {
    pub fn new(tag: &'static str) -> Self {
        Self {
            tag,
            ..Default::default()
        }
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

    pub fn change_texture(&mut self, texture: &'static str) {
        self.texture = Some(texture);
        self.appended.clear();
    }

    pub fn append(&mut self, texture: &'static str) {
        self.appended.push(texture);
    }

    pub fn blend_init(&mut self, texture: &'static str) {
        self.appended.clear();
        self.appended.push(texture);
        self.blend = [0.0, 1.0, 1.0, 1.0];
    }

    pub fn set_string(&mut self, text: &str) {
        self.text.clear();
        self.text.push_str(text);
    }

    pub fn set_font_size(&mut self, x: f32, y: f32) {
        self.font_size_x = x;
        self.font_size_y = y;
    }

    pub fn get_font_size(&self) -> (f32, f32) {
        (self.font_size_x, self.font_size_y)
    }

    pub fn set_char_space(&mut self, space: f32) {
        self.char_space = space;
    }

    pub fn get_char_space(&self) -> f32 {
        self.char_space
    }

    pub fn set_line_space(&mut self, space: f32) {
        self.line_space = space;
    }

    pub fn get_line_space(&self) -> f32 {
        self.line_space
    }

    pub fn shift_set(&mut self, x: f32, y: f32) {
        self.shift_x = x;
        self.shift_y = y;
    }

    pub fn width(&self) -> f32 {
        self.size.x
    }

    pub fn height(&self) -> f32 {
        self.size.y
    }

    pub fn parts_set(&mut self, layout: &ScreenPane) {
        self.kind = layout.kind;
        self.found = true;
        self.visible = layout.visible;
        self.pos_top_left_orig = F2 {
            x: layout.bounds.x,
            y: layout.bounds.y,
        };
        self.size_orig = F2 {
            x: layout.bounds.w,
            y: layout.bounds.h,
        };
        self.pos_center_orig = F2 {
            x: layout.bounds.x + layout.bounds.w / 2.0,
            y: layout.bounds.y + layout.bounds.h / 2.0,
        };
        self.init_alpha = layout.alpha;
        self.pos_top_left = self.pos_top_left_orig;
        self.pos_center = self.pos_center_orig;
        self.size = self.size_orig;
        self.now_alpha = 0;
        self.user_area = 0;
        self.rotation = layout.rotation;
        self.rotate_x = 0.0;
        self.rotate_y = 0.0;
        self.texture = None;
        self.appended.clear();
        self.text = layout.text.clone();
        self.font_size_x = layout.font_size_x as f32;
        self.font_size_y = layout.font_size_y as f32;
        self.char_space = layout.char_space as f32;
        self.line_space = layout.line_space as f32;
        self.shift_x = 0.0;
        self.shift_y = 0.0;
        self.color = layout.char_color;
    }

    pub fn cpos_move(&mut self) {
        self.pos_top_left.x = self.pos_center.x - self.size.x / 2.0;
        self.pos_top_left.y = self.pos_center.y - self.size.y / 2.0;
    }

    pub fn pane_trans(&mut self, x: f32, y: f32) {
        self.pos_center.x = self.pos_center_orig.x + x;
        self.pos_center.y = self.pos_center_orig.y + y;
        self.cpos_move();
    }

    pub fn pane_scale_xy(&mut self, scale: f32) {
        self.size.x = self.size_orig.x * scale;
        self.size.y = self.size_orig.y * scale;
        self.cpos_move();
    }

    pub fn rotate(&mut self, x: f32, y: f32, angle: f32) {
        self.rotate_x = x;
        self.rotate_y = y;
        self.rotation = ((angle * 32768.0 / 180.0) as i16 as u16) & 0xFFFF;
    }

    pub fn rotate_preserve(&mut self) {
        let angle = f32::from(self.rotation as i16) * 180.0 / 32768.0;
        self.rotation = ((angle * 32768.0 / 180.0) as i16 as u16) & 0xFFFF;
    }

    pub fn set_init_alpha(&mut self) {
        self.now_alpha = self.init_alpha;
    }

    pub fn set_now_alpha(&mut self, alpha: f32) {
        self.now_alpha = (f32::from(self.init_alpha) * alpha) as u8;
    }

    pub fn set_now_alpha_zero(&mut self) {
        self.now_alpha = 0;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StickControl {
    pub max_value: f32,
    pub min_value: f32,
    pub prev_bits: u8,
    pub bits: u8,
    pub horizontal_wait: i16,
    pub vertical_wait: i16,
    pub first_wait: i16,
    pub second_wait: i16,
    pub step: i16,
    pub min_step: i16,
    pub repeat_count: i16,
    pub repeat_left: i16,
    pub angle_base: i16,
    pub repeat_angle: i16,
    pub half_dead: i16,
}

impl Default for StickControl {
    fn default() -> Self {
        Self::new(5, 2, 3, 2)
    }
}

impl StickControl {
    pub fn new(first: i16, second: i16, step: i16, min_step: i16) -> Self {
        let mut stick = Self {
            max_value: STICK_MAX_VALUE,
            min_value: STICK_MIN_VALUE,
            prev_bits: 0,
            bits: 0,
            horizontal_wait: first,
            vertical_wait: first,
            first_wait: second,
            second_wait: second,
            step,
            min_step,
            repeat_count: 0,
            repeat_left: 0,
            angle_base: 0,
            repeat_angle: STICK_HALF_DEAD,
            half_dead: STICK_HALF_DEAD,
        };
        stick.init();
        stick
    }

    pub fn init(&mut self) {
        self.repeat_count = 0;
        self.vertical_wait = 0;
        self.prev_bits = 0;
        self.bits = 0;
        self.repeat_left = 0;
        self.horizontal_wait = self.first_wait;
        self.vertical_wait = self.first_wait;
        self.repeat_count = self.second_wait;
        self.repeat_left = self.second_wait;
    }

    pub fn x_init(&mut self) {
        self.repeat_count = 0;
        self.bits &= 0xFC;
        self.horizontal_wait = self.first_wait;
        self.repeat_left = self.second_wait;
    }

    pub fn y_init(&mut self) {
        self.vertical_wait = 0;
        self.bits &= !0x0C;
        self.vertical_wait = self.first_wait;
        self.repeat_left = self.second_wait;
    }

    pub fn check_trigger(&mut self, value: f32, angle: i16) -> u8 {
        self.prev_bits = self.bits;
        let half = (0x2000 - self.half_dead) >> 1;
        let mut bits = 0u8;
        if value != 0.0 {
            let base = self.angle_base;
            if angle < base - 0x7000 + half {
                bits |= 4;
            } else if angle < base - 0x5000 - half {
                bits |= 4;
                bits |= 1;
            } else if angle < base - 0x3000 + half {
                bits |= 1;
            } else if angle < base - 0x1000 - half {
                bits |= 1;
                bits |= 8;
            } else if angle < base + 0x1000 + half {
                bits |= 8;
            } else if angle < base + 0x3000 - half {
                bits |= 8;
                bits |= 2;
            } else if angle < base + 0x5000 + half {
                bits |= 2;
            } else if angle < base + 0x7000 - half {
                bits |= 2;
                bits |= 4;
            } else {
                bits |= 4;
            }

            if value >= self.max_value {
                self.bits = bits;
            } else if value < self.min_value {
                self.bits = 0;
            } else {
                self.bits &= !bits;
            }

            if self.bits != self.prev_bits {
                if self.bits == 0 {
                    self.angle_base = 0;
                } else if (angle & 0x1FFF) > 0x1000 {
                    self.angle_base = self.repeat_angle;
                } else {
                    self.angle_base = -self.repeat_angle;
                }
            }

            if self.bits & 0x03 == 0 {
                self.x_init();
            }
            if self.bits & 0x0C == 0 {
                self.y_init();
            }
        } else {
            self.bits = 0;
            self.x_init();
            self.y_init();
        }

        if self.prev_bits & self.bits & 0x03 != 0 && self.repeat_count > 0 {
            self.repeat_count -= 1;
        }
        if self.prev_bits & self.bits & 0x0C != 0 && self.vertical_wait > 0 {
            self.vertical_wait -= 1;
        }
        self.bits
    }

    fn repeat_step(&mut self, wait: &mut i16, left: &mut i16) {
        if *left == 0 {
            *wait -= self.step;
            if *wait < self.min_step {
                *wait = self.min_step;
            }
        } else {
            *left -= 1;
        }
    }

    pub fn check_left_trigger(&mut self) -> bool {
        if self.repeat_count == 0 && self.bits & 1 != 0 {
            self.repeat_count = self.horizontal_wait;
            let mut wait = self.horizontal_wait;
            let mut left = self.repeat_left;
            self.repeat_step(&mut wait, &mut left);
            self.horizontal_wait = wait;
            self.repeat_left = left;
            true
        } else {
            false
        }
    }

    pub fn check_right_trigger(&mut self) -> bool {
        if self.repeat_count == 0 && self.bits & 2 != 0 {
            self.repeat_count = self.horizontal_wait;
            let mut wait = self.horizontal_wait;
            let mut left = self.repeat_left;
            self.repeat_step(&mut wait, &mut left);
            self.horizontal_wait = wait;
            self.repeat_left = left;
            true
        } else {
            false
        }
    }

    pub fn check_up_trigger(&mut self) -> bool {
        if self.vertical_wait == 0 && self.bits & 4 != 0 {
            self.vertical_wait = self.vertical_wait;
            let mut wait = self.vertical_wait;
            let mut left = self.repeat_left;
            self.repeat_step(&mut wait, &mut left);
            self.vertical_wait = wait;
            self.repeat_left = left;
            true
        } else {
            false
        }
    }

    pub fn check_down_trigger(&mut self) -> bool {
        if self.vertical_wait == 0 && self.bits & 8 != 0 {
            self.vertical_wait = self.vertical_wait;
            let mut wait = self.vertical_wait;
            let mut left = self.repeat_left;
            self.repeat_step(&mut wait, &mut left);
            self.vertical_wait = wait;
            self.repeat_left = left;
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PadInput {
    pub trig_a: bool,
    pub trig_b: bool,
    pub trig_start: bool,
    pub trig_up: bool,
    pub trig_down: bool,
    pub trig_left: bool,
    pub trig_right: bool,
    pub trig_z3: bool,
    pub trig_a2: bool,
    pub hold_up: bool,
    pub hold_down: bool,
    pub hold_b: bool,
    pub hold_a: bool,
    pub stick_value: f32,
    pub stick_angle: i16,
}

pub trait MsgSource {
    fn message(&self, msg_no: u32) -> String;
    fn item_message(&self, msg_no: u32) -> String;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OutIcon {
    pub slot: u8,
    pub x: f32,
    pub y: f32,
    pub icon_no: u8,
    pub line: i16,
    pub visible: bool,
    pub color: u32,
}

#[derive(Resource, Clone, Debug)]
pub struct OptionPanes {
    pub title: Pane,
    pub note_frame: Pane,
    pub note_flag: Pane,
    pub icon: [Pane; ICON_COUNT],
    pub icon_shadow: [Pane; ICON_COUNT],
    pub note_text: [Pane; TYPE_PARTS],
    pub note_anchor: Pane,
    pub no_key: Pane,
    pub note_bar: [Pane; CURSOR_BAR_PARTS],
    pub row: [Pane; 4],
    pub attention: [Pane; TYPE_PARTS],
    pub sound: [Pane; SOUND_PARTS],
    pub vibration: [Pane; TYPE_PARTS],
    pub cursor: [Pane; CURSOR_PARTS],
    pub backing: [Pane; TYPE_PARTS],
    pub backdrop: Pane,
}

impl Default for OptionPanes {
    fn default() -> Self {
        Self {
            title: Pane::new("topt"),
            note_frame: Pane::new("sp01"),
            note_flag: Pane::new("sk01"),
            icon: std::array::from_fn(|i| Pane::new(TEXT_ROW[i])),
            icon_shadow: std::array::from_fn(|i| Pane::new(TEXT_SHADOW[i])),
            note_text: [
                Pane::new("st00"),
                Pane::new("str0"),
            ],
            note_anchor: Pane::new("nt00"),
            no_key: Pane::new("nk00"),
            note_bar: [Pane::new("yaz2"), Pane::new("yaz1")],
            row: [
                Pane::new("ttyu"),
                Pane::new("tvib"),
                Pane::new("tmon"),
                Pane::new("tsou"),
            ],
            attention: [Pane::new("thld"), Pane::new("tsic")],
            sound: [
                Pane::new("tmon"),
                Pane::new("tste"),
                Pane::new("tsur"),
            ],
            vibration: [Pane::new("tnas"), Pane::new("tari")],
            cursor: [
                Pane::new("cr32"),
                Pane::new("cr31"),
                Pane::new("cc22"),
                Pane::new("cc21"),
                Pane::new("cc12"),
                Pane::new("cr11"),
            ],
            backing: [Pane::new("bs09"), Pane::new("bs00")],
            backdrop: Pane::new("blak"),
        }
    }
}

impl OptionPanes {
    pub fn bind(&mut self, screen: &Screen) {
        self.title.parts_set_opt(screen);
        self.note_frame.parts_set_opt(screen);
        self.note_flag.parts_set_opt(screen);
        for index in 0..ICON_COUNT {
            self.icon[index].parts_set_opt(screen);
            self.icon_shadow[index].parts_set_opt(screen);
        }
        for index in 0..TYPE_PARTS {
            self.note_text[index].parts_set_opt(screen);
        }
        self.note_anchor.parts_set_opt(screen);
        self.no_key.parts_set_opt(screen);
        for index in 0..CURSOR_BAR_PARTS {
            self.note_bar[index].parts_set_opt(screen);
        }
        for index in 0..4 {
            self.row[index].parts_set_opt(screen);
        }
        for index in 0..TYPE_PARTS {
            self.attention[index].parts_set_opt(screen);
            self.vibration[index].parts_set_opt(screen);
        }
        for index in 0..SOUND_PARTS {
            self.sound[index].parts_set_opt(screen);
        }
        for index in 0..CURSOR_PARTS {
            self.cursor[index].parts_set_opt(screen);
        }
        for index in 0..TYPE_PARTS {
            self.backing[index].parts_set_opt(screen);
        }
        self.backdrop.parts_set_opt(screen);
    }

    pub fn value_pane(&self, row: OptionRow, index: u8) -> Option<&Pane> {
        match row {
            OptionRow::Attention | OptionRow::Ruby => self.attention.get(usize::from(index)),
            OptionRow::Vibration => self.vibration.get(usize::from(index)),
            OptionRow::Sound => self.sound.get(usize::from(index)),
        }
    }

    pub fn value_pane_mut(&mut self, row: OptionRow, index: u8) -> Option<&mut Pane> {
        match row {
            OptionRow::Attention | OptionRow::Ruby => self.attention.get_mut(usize::from(index)),
            OptionRow::Vibration => self.vibration.get_mut(usize::from(index)),
            OptionRow::Sound => self.sound.get_mut(usize::from(index)),
        }
    }

    pub fn value_count(&self, row: OptionRow) -> usize {
        match row {
            OptionRow::Attention | OptionRow::Ruby => TYPE_PARTS,
            OptionRow::Vibration => TYPE_PARTS,
            OptionRow::Sound => SOUND_PARTS,
        }
    }
}

impl Pane {
    fn parts_set_opt(&mut self, screen: &Screen) {
        if let Some(layout) = screen.find(self.tag) {
            self.parts_set(layout);
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct OptionMenu {
    pub screen: Option<Screen>,
    pub panes: OptionPanes,
    pub stick: StickControl,
    pub attention: u8,
    pub ruby: u8,
    pub sound: u8,
    pub vibration: u8,
    pub quit_status: u8,
    pub exit_status: u8,
    pub rumble_pattern: u8,
    pub rumble_strength: u8,
    pub rumble_kind: u8,
    pub rumble_delay: u8,
    pub main_text: String,
    pub text_row: [String; MSG_BUFFER_COUNT],
    pub note_text: String,
    pub note_ruby: String,
    pub note_main: String,
    pub note_spare: String,
    pub note_line_count: i16,
    pub note_line_offset: f32,
    pub note_font_size: f32,
    pub note_ruby_font_size: f32,
    pub note_sound_speed: i32,
    pub icons: [OutIcon; ICON_COUNT],
    pub icon_count: u8,
    pub text_area: [String; MSG_BUFFER_COUNT],
    pub sounds: Vec<u32>,
    pub rng_seed: [i32; 3],
}

impl Default for OptionMenu {
    fn default() -> Self {
        Self {
            screen: None,
            panes: OptionPanes::default(),
            stick: StickControl::new(5, 2, 3, 2),
            attention: ATTENTION_DEFAULT,
            ruby: RUBY_DEFAULT,
            sound: SOUND_DEFAULT_STEREO,
            vibration: VIBRATION_DEFAULT,
            quit_status: QUIT_NONE,
            exit_status: EXIT_NONE,
            rumble_pattern: 0,
            rumble_strength: 10,
            rumble_kind: 0xFF,
            rumble_delay: 0xC0,
            main_text: String::new(),
            text_row: Default::default(),
            note_text: String::new(),
            note_ruby: String::new(),
            note_main: String::new(),
            note_spare: String::new(),
            note_line_count: 0,
            note_line_offset: 0.0,
            note_font_size: 0.0,
            note_ruby_font_size: 0.0,
            note_sound_speed: 0,
            icons: std::array::from_fn(|_| OutIcon::default()),
            icon_count: 0,
            text_area: Default::default(),
            sounds: Vec::new(),
            rng_seed: [RND_SEED; 3],
        }
    }
}

impl OptionMenu {
    pub fn get_quit_status(&self) -> u8 {
        self.quit_status
    }

    pub fn get_exit_status(&self) -> u8 {
        self.exit_status
    }

    pub fn row(&self) -> OptionRow {
        OptionRow::from_index(self.panes.cursor[1].user_area as u8)
    }

    pub fn value(&self, row: OptionRow) -> u8 {
        match row {
            OptionRow::Attention => self.attention,
            OptionRow::Ruby => self.ruby,
            OptionRow::Vibration => self.vibration,
            OptionRow::Sound => self.sound,
        }
    }

    pub fn value_mut(&mut self, row: OptionRow) -> &mut u8 {
        match row {
            OptionRow::Attention => &mut self.attention,
            OptionRow::Ruby => &mut self.ruby,
            OptionRow::Vibration => &mut self.vibration,
            OptionRow::Sound => &mut self.sound,
        }
    }

    pub fn sound(&mut self, se: u32) {
        self.sounds.push(se);
    }

    pub fn create(
        &mut self,
        screen: Screen,
        msgs: &dyn MsgSource,
        config: &PlayerConfig,
        rumbles: &mut Rumbles,
    ) {
        self.screen = Some(screen);
        self.screen_set(msgs);
        self.initialize(config, rumbles.supported);
        self.quit_status = 0;
        self.rumble_pattern = 0;
        self.rumble_strength = 10;
        self.rumble_kind = 0xFF;
        self.rumble_delay = 0xC0;
    }

    pub fn delete(&mut self) {
        self.screen = None;
        self.panes = OptionPanes::default();
        self.stick = StickControl::new(5, 2, 3, 2);
    }

    pub fn set_text_area(&mut self, main: String, ruby: String, spare: String, other: String) {
        self.text_area = [main, ruby, spare, other];
    }

    pub fn screen_set(&mut self, msgs: &dyn MsgSource) {
        let Some(screen) = self.screen.clone() else {
            return;
        };
        self.panes.bind(&screen);

        self.main_text = msgs.message(MSG_TITLE);
        self.text_row[0] = msgs.message(MSG_ATTENTION_0);
        self.text_row[1] = msgs.message(MSG_ATTENTION_1);
        self.text_row[2] = msgs.message(MSG_VIBRATION_0);
        self.text_row[3] = msgs.message(MSG_VIBRATION_1);

        for index in 0..ICON_COUNT {
            self.panes.icon[index].change_texture(ICON_OUT_TEXTURE);
            self.panes.icon[index].blend_init(ICON_BLEND_TEXTURE);
        }

        self.panes.title.set_string(&self.main_text);

        for index in 0..TYPE_PARTS {
            let text = self.text_row[index].clone();
            self.panes.attention[index].set_string(&text);
            self.change_scale_center(LabelSlot::Attention0, &text.clone());
            self.change_scale_center(LabelSlot::Vibration0, &text);
            let _ = index;
        }

        let vibration = [self.text_row[2].clone(), self.text_row[3].clone()];
        for index in 0..TYPE_PARTS {
            self.panes.vibration[index].set_string(&vibration[index]);
            let slot = if index == 0 {
                LabelSlot::Vibration0
            } else {
                LabelSlot::Vibration1
            };
            self.change_scale_center(slot, &vibration[index].clone());
        }

        let sound = [
            msgs.message(MSG_SOUND_0),
            msgs.message(MSG_SOUND_1),
            msgs.message(MSG_SOUND_2),
        ];
        let sound_slots = [
            LabelSlot::Sound0,
            LabelSlot::Sound1,
            LabelSlot::Sound2,
        ];
        for index in 0..SOUND_PARTS {
            self.panes.sound[index].set_string(&sound[index]);
            self.change_scale_center(sound_slots[index], &sound[index].clone());
        }

        for (index, slot) in ROW_LABEL_SLOTS.into_iter().enumerate() {
            let text = msgs.message(ROW_LABEL_MESSAGES[index]);
            self.change_scale_right(slot, &text);
        }

        self.panes.note_anchor.user_area =
            normalize_angle(rotation_degrees(self.panes.note_anchor.rotation));
        self.panes.title.user_area = normalize_angle(rotation_degrees(self.panes.title.rotation));
        self.panes.note_frame.user_area =
            normalize_angle(rotation_degrees(self.panes.note_frame.rotation));
        self.panes.note_flag.user_area =
            normalize_angle(rotation_degrees(self.panes.note_flag.rotation));
    }

    pub fn initialize(&mut self, config: &PlayerConfig, rumble_supported: bool) {
        self.main_init();
        self.note_init();
        self.title_init();
        self.out_font_init();

        self.quit_status = QUIT_OPEN;
        self.backdrop_user_area_reset();

        self.attention = config.attention;
        self.ruby = config.ruby;
        self.sound = config.sound;
        self.vibration = if rumble_supported {
            config.now_vibration
        } else {
            0
        };

        self.exit_status = EXIT_NONE;

        let mode = SOUND_MODE[usize::from(self.sound)];
        self.rumble_kind = mode as u8;
    }

    fn backdrop_user_area_reset(&mut self) {
        self.panes.backdrop.user_area = 0;
        self.panes.backing[0].user_area = 0;
        self.panes.backing[1].user_area = 0;
    }

    pub fn main_init(&mut self) {
        self.panes.backdrop.set_now_alpha_zero();

        for index in 0..CURSOR_BAR_PARTS {
            self.panes.note_bar[index].pane_trans(0.0, 0.0);
            self.panes.note_bar[0].rotate(
                self.panes.note_bar[0].size.x / 2.0,
                self.panes.note_bar[0].size.y / 2.0,
                90.0,
            );
            self.panes.backing[index].set_now_alpha_zero();
            self.panes.note_bar[index].set_now_alpha_zero();
            self.panes.attention[index].set_now_alpha_zero();
            self.panes.vibration[index].set_now_alpha_zero();
        }

        for index in 0..SOUND_PARTS {
            self.panes.sound[index].set_now_alpha_zero();
        }

        for index in 0..2 {
            self.panes.row[index].set_now_alpha_zero();
        }
        self.panes.row[3].set_now_alpha_zero();

        for index in 0..CURSOR_PARTS {
            self.panes.cursor[index].pane_trans(0.0, 0.0);
            self.panes.cursor[index].set_now_alpha_zero();
        }

        self.panes.cursor[0].user_area = 0;
        self.panes.cursor[1].user_area = 0;
        self.panes.cursor[2].user_area = 0;
        self.panes.note_bar[0].user_area = 0;
        self.panes.note_bar[1].user_area = 0;

        self.cursor_move();
    }

    pub fn note_init(&mut self) {
        let f31 = -12.0;

        self.panes.note_anchor.pane_trans(0.0, NOTE_DY);

        self.panes.note_anchor.rotate(
            self.panes.note_anchor.size.x / 2.0,
            self.panes.note_anchor.size.y / 2.0,
            f31 + f32::from(self.panes.note_anchor.user_area as i16),
        );

        self.panes.note_text[0].set_now_alpha_zero();
        self.panes.note_text[1].set_now_alpha_zero();
        self.panes.note_anchor.set_now_alpha_zero();
        self.panes.no_key.set_now_alpha_zero();
        self.panes.backdrop.set_now_alpha_zero();
    }

    pub fn title_init(&mut self) {
        let mut rotate_angle = f32::from(self.panes.title.user_area as i16) + NOTE_TITLE_ANGLE;
        if rotate_angle > 180.0 {
            rotate_angle -= 360.0;
        }

        if rotate_angle < -180.0 {
            rotate_angle += 360.0;
        }

        self.panes.title.pane_trans(0.0, TITLE_DY);
        self.panes.note_frame.pane_trans(0.0, TITLE_DY);
        self.panes.note_flag.pane_trans(0.0, TITLE_DY);

        self.panes.title.rotate(
            self.panes.title.size.x / 2.0,
            self.panes.title.size.y / 2.0,
            rotate_angle + f32::from(self.panes.title.user_area as i16),
        );
        self.panes.note_frame.rotate(
            self.panes.note_frame.size.x / 2.0,
            self.panes.note_frame.size.y / 2.0,
            rotate_angle + f32::from(self.panes.note_frame.user_area as i16),
        );
        self.panes.note_flag.rotate(
            self.panes.note_flag.size.x / 2.0,
            self.panes.note_flag.size.y / 2.0,
            rotate_angle + f32::from(self.panes.note_flag.user_area as i16),
        );

        self.panes.title.set_now_alpha_zero();
        self.panes.note_frame.set_now_alpha_zero();
        self.panes.note_flag.set_now_alpha_zero();
    }

    pub fn main_move(&mut self) {
        let alpha = value_increase(
            OPEN_FADE.into(),
            i32::from(self.panes.backing[0].user_area),
            0,
        );

        self.panes.backdrop.set_now_alpha(alpha);

        if let Some(pane) = self.panes.attention.get_mut(usize::from(self.attention)) {
            pane.set_now_alpha(alpha);
        }
        if let Some(pane) = self.panes.vibration.get_mut(usize::from(self.vibration)) {
            pane.set_now_alpha(alpha);
        }
        if let Some(pane) = self.panes.sound.get_mut(usize::from(self.sound)) {
            pane.set_now_alpha(alpha);
        }

        for index in 0..TYPE_PARTS {
            self.panes.backing[index].set_now_alpha(alpha);
            self.panes.note_bar[index].set_now_alpha(alpha);
        }

        for index in 0..2 {
            self.panes.row[index].set_now_alpha(alpha);
        }
        self.panes.row[3].set_now_alpha(alpha);

        self.panes.cursor[0].set_now_alpha(alpha);
        self.panes.cursor[1].set_now_alpha(alpha);
    }

    pub fn note_move(&mut self) {
        let alpha = value_increase(
            OPEN_FADE.into(),
            self.panes.backing[0].user_area as i32 - OPEN_FADE as i32,
            0,
        );
        let f30 = (1.0 - alpha) * -12.0;
        self.panes.note_anchor.pane_trans(0.0, (1.0 - alpha) * NOTE_DY);

        self.panes.note_anchor.rotate(
            self.panes.note_anchor.size.x / 2.0,
            self.panes.note_anchor.size.y / 2.0,
            f30 + f32::from(self.panes.note_anchor.user_area as i16),
        );

        self.panes.note_text[0].set_now_alpha(alpha);
        self.panes.note_text[1].set_now_alpha(alpha);
        self.panes.note_anchor.set_now_alpha(alpha);
        self.panes.no_key.set_now_alpha(alpha);
    }

    pub fn title_move(&mut self) {
        let alpha = value_increase(
            OPEN_FADE.into(),
            self.panes.backing[0].user_area as i32 - OPEN_FADE as i32,
            0,
        );

        let mut rotate_angle = f32::from(self.panes.title.user_area as i16) + NOTE_TITLE_ANGLE;
        if rotate_angle > 180.0 {
            rotate_angle -= 360.0;
        }

        if rotate_angle < -180.0 {
            rotate_angle += 360.0;
        }

        let f30 = rotate_angle * (1.0 - alpha);

        if self.panes.backing[0].user_area as u8 == OPEN_FADE + 1 {
            self.sound(SE_MENU_OPTION_STRING);
        }

        let y = 1.0 - alpha;

        self.panes.title.pane_trans(0.0, y * TITLE_DY);
        self.panes.note_frame.pane_trans(0.0, y * TITLE_DY);
        self.panes.note_flag.pane_trans(0.0, y * TITLE_DY);

        self.panes.title.rotate(
            self.panes.title.size.x / 2.0,
            self.panes.title.size.y / 2.0,
            f30 + f32::from(self.panes.title.user_area as i16),
        );
        self.panes.note_frame.rotate(
            self.panes.note_frame.size.x / 2.0,
            self.panes.note_frame.size.y / 2.0,
            f30 + f32::from(self.panes.note_frame.user_area as i16),
        );
        self.panes.note_flag.rotate(
            self.panes.note_flag.size.x / 2.0,
            self.panes.note_flag.size.y / 2.0,
            f30 + f32::from(self.panes.note_flag.user_area as i16),
        );

        self.panes.title.set_now_alpha(alpha);
        self.panes.note_frame.set_now_alpha(alpha);
        self.panes.note_flag.set_now_alpha(alpha);
    }

    pub fn cursor_move(&mut self) {
        let row = usize::from(self.panes.cursor[1].user_area as u8).min(3);

        for index in 0..CURSOR_PARTS {
            let pane = &mut self.panes.cursor[index];
            pane.pos_center.x = self.panes.row[row].pos_center_orig.x;
            pane.pos_center.y = self.panes.row[row].pos_center_orig.y;
            pane.size.x = self.panes.row[row].size_orig.x;
            pane.cpos_move();
            let rotation = rotation_degrees(pane.rotation);
            pane.rotate(pane.size.x / 2.0, pane.size.y / 2.0, rotation);
        }

        self.cursor_scale();
    }

    pub fn cursor_scale(&mut self) {
        let mut x = [0.0f32; CURSOR_BAR_PARTS];
        let mut y = 0.0f32;
        let row = self.row();
        let target = match row {
            OptionRow::Attention => self.value_pane(row, self.attention),
            OptionRow::Ruby => self.value_pane(row, self.ruby),
            OptionRow::Vibration => self.value_pane(row, self.vibration),
            OptionRow::Sound => self.value_pane(row, self.sound),
        };
        if let Some(pane) = target {
            x[0] = pane.pos_top_left_orig.x - 20.0;
            x[1] = pane.pos_top_left_orig.x + pane.size_orig.x + 20.0;
            y = pane.pos_center_orig.y;
        }

        for index in 0..CURSOR_BAR_PARTS {
            self.panes.note_bar[index].pos_center.x = x[index];
            self.panes.note_bar[index].pos_center.y = y;
            self.panes.note_bar[index].cpos_move();
        }
    }

    pub fn type_move(&mut self) {
        let row = self.row();
        let value = self.value(row);
        let count = self.panes.value_count(row);
        let panes: &mut [Pane] = match row {
            OptionRow::Attention | OptionRow::Ruby => &mut self.panes.attention[..TYPE_PARTS],
            OptionRow::Vibration => &mut self.panes.vibration[..TYPE_PARTS],
            OptionRow::Sound => &mut self.panes.sound[..SOUND_PARTS],
        };
        for index in 0..count.min(panes.len()) {
            if index == value {
                panes[index].set_init_alpha();
            } else {
                panes[index].set_now_alpha_zero();
            }
        }

        self.cursor_scale();
    }

    pub fn bar_anime(&mut self) {
        let fvar1 = if self.panes.note_bar[0].user_area < BAR_PERIOD {
            BAR_OFFSET
        } else {
            -BAR_OFFSET
        };

        self.panes.note_bar[0].rotate(
            self.panes.note_bar[0].size.x / 2.0,
            self.panes.note_bar[0].size.y / 2.0,
            90.0 + fvar1,
        );
        self.panes.note_bar[1].rotate(
            self.panes.note_bar[1].size.x / 2.0,
            self.panes.note_bar[1].size.y / 2.0,
            90.0 + fvar1,
        );

        self.panes.note_bar[0].user_area += 1;

        if self.panes.note_bar[1].user_area > 0 {
            self.panes.note_bar[1].user_area -= 1;
            let x = (self.panes.note_bar[1].pos_top_left.x + 0.5) as i32
                + MOVE_TABLE[self.panes.note_bar[1].user_area as usize];
            let y = (self.panes.note_bar[1].pos_top_left.y + 0.5) as i32;
            self.panes.note_bar[1].move_to_opt(x as f32, y as f32);
        } else if self.panes.note_bar[1].user_area < 0 {
            self.panes.note_bar[1].user_area += 1;
            let x = (self.panes.note_bar[0].pos_top_left.x + 0.5) as i32
                - MOVE_TABLE[self.panes.note_bar[1].user_area.unsigned_abs() as usize];
            let y = (self.panes.note_bar[0].pos_top_left.y + 0.5) as i32;
            self.panes.note_bar[0].move_to_opt(x as f32, y as f32);
        }

        if self.panes.note_bar[0].user_area >= BAR_PERIOD {
            self.panes.note_bar[0].user_area = 0;
        }
    }

    pub fn cursor_anime(&mut self) {
        let r29 = self.panes.cursor[0].user_area / 7;
        for index in 0..3 {
            if index == r29 {
                self.panes.cursor[index * 2].set_init_alpha();
                self.panes.cursor[index * 2 + 1].set_init_alpha();
            } else {
                self.panes.cursor[index * 2].set_now_alpha_zero();
                self.panes.cursor[index * 2 + 1].set_now_alpha_zero();
            }
        }

        self.panes.cursor[0].user_area += 1;

        if self.panes.cursor[0].user_area >= 0x15 {
            self.panes.cursor[0].user_area = 0;
        }
    }

    pub fn stick_move(
        &mut self,
        trigger: u8,
        config: &mut PlayerConfig,
        rumbles: &mut Rumbles,
        vibration: &mut Vibration,
    ) {
        match trigger {
            STICK_UP => {
                if self.panes.cursor[1].user_area > 0 {
                    self.panes.cursor[1].user_area -= 1;
                    self.cursor_move();
                    self.note_set();
                    self.sound(SE_MENU_CURSOR);
                }
            }
            STICK_DOWN => {
                if self.panes.cursor[1].user_area < 3 {
                    self.panes.cursor[1].user_area += 1;
                    self.cursor_move();
                    self.note_set();
                    self.sound(SE_MENU_CURSOR);
                }
            }
            STICK_RIGHT => {
                let row = self.row();
                match row {
                    OptionRow::Attention => {
                        if self.attention != 0 {
                            self.attention = 0;
                        } else {
                            self.attention = 1;
                        }
                        self.panes.note_bar[1].user_area = 6;
                        self.sound(SE_MENU_OPTION_SWITCH);
                    }
                    OptionRow::Ruby => {
                        if self.ruby != 0 {
                            self.ruby = 0;
                        } else {
                            self.ruby = 1;
                        }
                        self.panes.note_bar[1].user_area = 6;
                        self.sound(SE_MENU_OPTION_SWITCH);
                    }
                    OptionRow::Sound => {
                        if self.sound < 2 {
                            self.sound += 1;
                        } else {
                            self.sound = 0;
                        }
                        self.panes.note_bar[1].user_area = 6;
                        self.sound_effect(SE_MENU_OPTION_SWITCH);
                        self.set_sound_mode(config);
                    }
                    OptionRow::Vibration => {
                        if rumbles.supported {
                            if self.vibration != 0 {
                                self.vibration = 0;
                            } else {
                                self.vibration = 1;
                                rumbles.start_patterned(
                                    RUMBLE_PATTERN,
                                    RUMBLE_LOOP_ONCE,
                                    RUMBLE_STRENGTH,
                                );
                                vibration.motor_shock.pattern_idx =
                                    rumbles.pattern[0] as i32;
                                vibration.motor_shock.length = i32::from(rumbles.length);
                            }
                            self.panes.note_bar[1].user_area = 6;
                            self.sound(SE_MENU_OPTION_SWITCH);
                        }
                    }
                }
                self.type_move();
            }
            STICK_LEFT => {
                let row = self.row();
                match row {
                    OptionRow::Attention => {
                        if self.attention != 0 {
                            self.attention = 0;
                        } else {
                            self.attention = 1;
                        }
                        self.panes.note_bar[1].user_area = -6;
                        self.sound(SE_MENU_OPTION_SWITCH);
                    }
                    OptionRow::Ruby => {
                        if self.ruby != 0 {
                            self.ruby = 0;
                        } else {
                            self.ruby = 1;
                        }
                        self.panes.note_bar[1].user_area = -6;
                        self.sound(SE_MENU_OPTION_SWITCH);
                    }
                    OptionRow::Sound => {
                        if self.sound != 0 {
                            self.sound -= 1;
                        } else {
                            self.sound = 2;
                        }
                        self.panes.note_bar[1].user_area = -6;
                        self.sound_effect(SE_MENU_OPTION_SWITCH);
                        self.set_sound_mode(config);
                    }
                    OptionRow::Vibration => {
                        if rumbles.supported {
                            if self.vibration != 0 {
                                self.vibration = 0;
                            } else {
                                self.vibration = 1;
                                rumbles.start_patterned(
                                    RUMBLE_PATTERN,
                                    RUMBLE_LOOP_ONCE,
                                    RUMBLE_STRENGTH,
                                );
                                vibration.motor_shock.pattern_idx =
                                    rumbles.pattern[0] as i32;
                                vibration.motor_shock.length = i32::from(rumbles.length);
                            }
                            self.panes.note_bar[1].user_area = -6;
                            self.sound(SE_MENU_OPTION_SWITCH);
                        }
                    }
                }
                self.type_move();
            }
            _ => {}
        }
    }

    fn sound_effect(&mut self, se: u32) {
        self.sounds.push(se);
    }

    fn set_sound_mode(&mut self, config: &mut PlayerConfig) {
        let mode = SOUND_MODE[usize::from(self.sound)];
        config.sound = self.sound;
        config.sound_mode = if mode == 0 { 0 } else { 1 };
        self.rumble_kind = mode as u8;
    }

    pub fn note_set(&mut self, msgs: &dyn MsgSource) {
        self.note_main = self.text_area.first().cloned().unwrap_or_default();
        self.note_spare = self.text_area.get(1).cloned().unwrap_or_default();
        self.note_text = msgs.item_message(MSG_OPTION_NOTE);
        self.note_ruby = self.note_text.clone();

        self.out_font_init();

        self.note_font_size = self.panes.note_text[0].font_size_x;
        self.note_ruby_font_size = self.panes.note_text[1].font_size_x;

        self.note_line_count = count_lines(&self.note_text) as i16;
        self.note_line_offset = (self.panes.note_text[0].height()
            - self.note_font_size
            - f32::from(self.note_line_count) * self.panes.note_text[0].get_line_space())
            / 2.0;
        self.note_sound_speed = NOTE_SEND_SPEED;

        self.panes.note_text[0].shift_set(0.0, self.note_line_offset);
        self.panes.note_text[1].shift_set(0.0, self.note_line_offset);

        self.panes.note_text[0].set_string(&self.note_text);
        self.panes.note_text[1].set_string(&self.note_ruby);

        let half_line_space = self.panes.note_text[0].get_line_space() / 2.0;
        for index in 0..ICON_COUNT {
            let icon = self.icons[index];
            if icon.slot == 0xFF {
                continue;
            }
            if self.panes.icon[index].user_area != -1 {
                continue;
            }
            if icon.slot == 0x16 {
                continue;
            }
            self.panes.icon[index].pos_top_left.x = icon.x;
            self.panes.icon[index].pos_top_left.y =
                self.note_line_offset + icon.y * 2.0 * half_line_space;
            self.panes.icon[index].size_orig.y = f32::from(icon.slot);
            self.panes.icon[index].blend = [0.0, 1.0, 1.0, 1.0];
            self.panes.icon[index].color = icon.color;
            self.icon_count = self.icon_count.saturating_add(1);
        }
    }

    pub fn out_font_init(&mut self) {
        for index in 0..ICON_COUNT {
            self.panes.icon[index].user_area = -1;
            self.panes.icon[index].rotate(0.0, 0.0, 0.0);
            self.panes.icon[index].set_now_alpha_zero();
        }
    }

    pub fn out_font_move(&mut self) {
        for index in 0..ICON_COUNT {
            if self.panes.icon[index].user_area != -1 {
                let base_x = self.panes.note_text[0].pos_top_left.x
                    + (self.panes.note_anchor.pos_top_left.x
                        + self.panes.note_anchor.pos_top_left.x);
                let base_y = self.panes.note_text[0].pos_top_left.y
                    + (self.panes.note_anchor.pos_top_left.y
                        + self.panes.note_anchor.pos_top_left.y);
                self.panes.icon_shadow[index].pos_top_left.x =
                    self.panes.icon[index].pos_top_left.x + base_x;
                self.panes.icon_shadow[index].pos_top_left.y =
                    self.panes.icon[index].pos_top_left.y + base_y;

                let offset_x =
                    self.panes.note_anchor.pos_center.x - self.panes.icon_shadow[index].pos_top_left.x;
                let offset_y = self.panes.note_anchor.pos_center.y
                    - self.panes.icon_shadow[index].pos_top_left.y;
                let rotation = f32::from(self.panes.note_anchor.rotation as i16) * 180.0 / 32768.0;
                self.panes.icon_shadow[index].rotate(offset_x, offset_y, rotation);
            } else {
                self.panes.icon[index].set_now_alpha_zero();
            }
        }
    }

    pub fn out_font_draw(&mut self) {
        for index in 0..ICON_COUNT {
            if self.panes.icon[index].user_area != -1 {
                self.panes.icon[index].now_alpha = self.panes.note_text[0].now_alpha;
            }
        }
    }

    pub fn string_length(&mut self, slot: LabelSlot, text: &str) -> f32 {
        let (cell, char_space) = {
            let pane = self.label(slot);
            (pane.font_size_x, pane.char_space)
        };
        let mut total = 0.0f32;
        let mut first = true;
        let bytes: Vec<char> = text.chars().collect();
        let mut at = 0usize;
        while at < bytes.len() {
            if bytes[at] == '\u{1a}' {
                let len = bytes.get(at + 1).map_or(0, |c| *c as usize);
                at += 1 + len;
            } else {
                let code = u32::from(bytes[at] as u32);
                at += 1;
                let width = char_width(code);
                if first {
                    total = f32::from(width);
                    first = false;
                } else {
                    total += f32::from(width) + char_space;
                }
            }
        }
        let ratio = if cell == 0.0 { 0.0 } else { 1.0 / cell };
        let mut scaled = if first { 0.0 } else { total * ratio };
        if !first {
            total = total * ratio;
        }
        if scaled > f32::from(SCALE_FADE) {
            self.label_mut(slot).font_size_x =
                self.label(slot).font_size_x * (f32::from(SCALE_FADE) / scaled);
            scaled = f32::from(SCALE_FADE);
        }
        let _ = total;
        scaled
    }

    fn label(&self, slot: LabelSlot) -> &Pane {
        match slot {
            LabelSlot::Attention0 => &self.panes.attention[0],
            LabelSlot::Attention1 => &self.panes.attention[1],
            LabelSlot::Vibration0 => &self.panes.vibration[0],
            LabelSlot::Vibration1 => &self.panes.vibration[1],
            LabelSlot::Sound0 => &self.panes.sound[0],
            LabelSlot::Sound1 => &self.panes.sound[1],
            LabelSlot::Sound2 => &self.panes.sound[2],
            LabelSlot::Row0 => &self.panes.row[0],
            LabelSlot::Row2 => &self.panes.row[1],
            LabelSlot::Row3 => &self.panes.row[2],
            LabelSlot::Row4 => &self.panes.row[3],
            LabelSlot::Title => &self.panes.title,
        }
    }

    fn label_mut(&mut self, slot: LabelSlot) -> &mut Pane {
        match slot {
            LabelSlot::Attention0 => &mut self.panes.attention[0],
            LabelSlot::Attention1 => &mut self.panes.attention[1],
            LabelSlot::Vibration0 => &mut self.panes.vibration[0],
            LabelSlot::Vibration1 => &mut self.panes.vibration[1],
            LabelSlot::Sound0 => &mut self.panes.sound[0],
            LabelSlot::Sound1 => &mut self.panes.sound[1],
            LabelSlot::Sound2 => &mut self.panes.sound[2],
            LabelSlot::Row0 => &mut self.panes.row[0],
            LabelSlot::Row2 => &mut self.panes.row[1],
            LabelSlot::Row3 => &mut self.panes.row[2],
            LabelSlot::Row4 => &mut self.panes.row[3],
            LabelSlot::Title => &mut self.panes.title,
        }
    }

    pub fn change_scale_center(&mut self, slot: LabelSlot, text: &str) {
        let length = self.string_length(slot, text);
        let pane = self.label_mut(slot);
        pane.size.x = length;
        pane.size_orig.x = length;
        pane.cpos_move();
        let left = pane.pos_top_left.x;
        pane.pos_top_left_orig.x = left;
    }

    pub fn change_scale_right(&mut self, slot: LabelSlot, text: &str) {
        let length = self.string_length(slot, text);
        let pos = self.label(slot).pos_top_left.x + self.label(slot).size.x;
        let pane = self.label_mut(slot);
        pane.size.x = length;
        pane.size_orig.x = length;
        let new_pos = pos - pane.size.x / 2.0;
        pane.pos_center.x = new_pos;
        pane.pos_center_orig.x = new_pos;
        pane.cpos_move();
    }

    pub fn move_step(
        &mut self,
        pad: &PadInput,
        config: &mut PlayerConfig,
        rumbles: &mut Rumbles,
        vibration: &mut Vibration,
    ) {
        let check_trigger = self
            .stick
            .check_trigger(pad.stick_value, pad.stick_angle);

        if self.vibration != 0 && !rumbles.supported {
            self.vibration = 0;

            for index in 0..TYPE_PARTS {
                if index == self.vibration {
                    self.panes.vibration[index].set_init_alpha();
                } else {
                    self.panes.vibration[index].set_now_alpha_zero();
                }
            }

            self.cursor_scale();
        }

        if pad.trig_a
            && !pad.trig_start
            && !pad.trig_b
            && !pad.trig_up
            && !pad.trig_down
            && !pad.trig_left
            && !pad.trig_right
        {
            self.quit_status = QUIT_CLOSED;
            self.panes.backing[0].user_area = 0;
            self.exit_status = EXIT_SAVED;
            config.attention = self.attention;
            config.ruby = self.ruby;
            if rumbles.supported {
                config.vibration = self.vibration;
            }
            config.now_vibration = self.vibration;
            config.sound = self.sound;
            self.set_sound_mode(config);
            self.sound_effect(SE_MENU_OPTION_OUT);
        } else if pad.trig_b
            && !pad.trig_start
            && !pad.trig_a
            && !pad.trig_up
            && !pad.trig_down
            && !pad.trig_left
            && !pad.trig_right
        {
            self.quit_status = QUIT_CLOSED;
            self.exit_status = EXIT_DISCARDED;
            self.panes.backing[0].user_area = 0;
            self.sound_effect(SE_MENU_OPTION_OUT);
        } else if check_trigger == 0 {
            self.panes.note_anchor.user_area = 0;
            self.panes.backing[1].user_area = 0;
        } else {
            if self.panes.backing[1].user_area == 0 {
                self.stick_move(check_trigger, config, rumbles, vibration);
            }

            let mut svar2 = if self.panes.note_anchor.user_area < 2 {
                OPTION_NOTE_RAMP
            } else {
                OPTION_NOTE_STEP
            };

            self.panes.backing[1].user_area += 1;

            if self.panes.backing[1].user_area > svar2 {
                svar2 = i32::from(self.panes.note_anchor.user_area);

                if svar2 < 2 {
                    self.panes.note_anchor.user_area = (svar2 + 1) as i16;
                }

                self.panes.backing[1].user_area = 0;
            }
        }

        self.bar_anime();

        if self.panes.cursor[1].user_area != 2 || rumbles.supported {
            self.panes.note_bar[0].show();
            self.panes.note_bar[1].show();
        } else {
            self.panes.note_bar[0].hide();
            self.panes.note_bar[1].hide();
        }

        self.cursor_anime();

        self.out_font_move();

        if self.ruby != 0 {
            self.panes.note_text[1].hide();
        } else {
            self.panes.note_text[1].show();
        }
    }

    pub fn open(
        &mut self,
        push_menu_button: &mut Option<u8>,
        msgs: &dyn MsgSource,
        pad: &PadInput,
        config: &mut PlayerConfig,
        rumbles: &mut Rumbles,
        vibration: &mut Vibration,
    ) -> bool {
        let mut ret = false;

        *push_menu_button = Some(0);

        if self.panes.backing[0].user_area == 0 {
            self.note_set(msgs);
        }

        self.panes.backing[0].user_area += 1;

        if self.panes.backing[0].user_area <= OPEN_FADE {
            self.main_move();
        } else if self.panes.backing[0].user_area <= OPEN_DONE {
            self.note_move();

            self.title_move();

            self.move_step(pad, config, rumbles, vibration);

            if self.panes.backing[0].user_area == OPEN_DONE {
                self.quit_status = QUIT_CLOSING;

                self.panes.backing[0].user_area = 0;

                ret = true;
            }
        }

        ret
    }

    pub fn close(
        &mut self,
        push_menu_button: &mut Option<u8>,
        config: &PlayerConfig,
        rumbles: &Rumbles,
    ) -> bool {
        let r30 = CLOSE_FADE;

        let value = {
            let frame = &mut self.panes.backing[0].user_area;
            *frame += 1;
            *frame
        };

        value_increase(i32::from(CLOSE_FADE), i32::from(value), 0);

        *push_menu_button = Some(2);

        if i32::from(r30) <= i32::from(value) {
            self.initialize(config, rumbles.supported);
            self.quit_status = QUIT_NONE;
            return true;
        }

        false
    }

    pub fn draw(&mut self) {
        if self.exit_status != EXIT_NONE {
            let r4 = self.panes.backing[0].user_area;
            let alpha_scale = 1.0 - value_increase(i32::from(CLOSE_FADE), i32::from(r4), 0);
            self.scale_alpha(alpha_scale);
        }

        self.out_font_draw();
    }

    pub fn scale_alpha(&mut self, alpha_scale: f32) {
        for (pane, value) in [
            (&mut self.panes.title, self.panes.title.now_alpha),
            (&mut self.panes.note_frame, self.panes.note_frame.now_alpha),
            (&mut self.panes.note_flag, self.panes.note_flag.now_alpha),
            (&mut self.panes.note_anchor, self.panes.note_anchor.now_alpha),
            (&mut self.panes.no_key, self.panes.no_key.now_alpha),
            (&mut self.panes.backdrop, self.panes.backdrop.now_alpha),
        ] {
            pane.now_alpha = (f32::from(value) * alpha_scale) as u8;
        }

        for index in 0..TYPE_PARTS {
            let text = self.panes.note_text[index].now_alpha;
            self.panes.note_text[index].now_alpha = (f32::from(text) * alpha_scale) as u8;
            let bar = self.panes.note_bar[index].now_alpha;
            self.panes.note_bar[index].now_alpha = (f32::from(bar) * alpha_scale) as u8;
            let value = self.panes.attention[index].now_alpha;
            self.panes.attention[index].now_alpha = (f32::from(value) * alpha_scale) as u8;
            let vib = self.panes.vibration[index].now_alpha;
            self.panes.vibration[index].now_alpha = (f32::from(vib) * alpha_scale) as u8;
            let back = self.panes.backing[index].now_alpha;
            self.panes.backing[index].now_alpha = (f32::from(back) * alpha_scale) as u8;
        }

        for index in 0..4 {
            let row = self.panes.row[index].now_alpha;
            self.panes.row[index].now_alpha = (f32::from(row) * alpha_scale) as u8;
        }

        for index in 0..SOUND_PARTS {
            let sound = self.panes.sound[index].now_alpha;
            self.panes.sound[index].now_alpha = (f32::from(sound) * alpha_scale) as u8;
        }

        for index in 0..CURSOR_PARTS {
            let cursor = self.panes.cursor[index].now_alpha;
            self.panes.cursor[index].now_alpha = (f32::from(cursor) * alpha_scale) as u8;
        }
    }

    pub fn rnd(&mut self) -> f32 {
        for lane in 0..3 {
            self.rng_seed[lane] = (self.rng_seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
        }
        let sum = f32::from(self.rng_seed[0]) / RND_SCALE[0]
            + f32::from(self.rng_seed[1]) / RND_SCALE[1]
            + f32::from(self.rng_seed[2]) / RND_SCALE[2];
        (sum % 1.0).abs()
    }
}

impl Pane {
    fn move_to_opt(&mut self, x: f32, y: f32) {
        self.pos_top_left.x = x;
        self.pos_top_left.y = y;
    }
}

pub fn rotation_degrees(rotation: u16) -> f32 {
    f32::from(rotation as i16) * 180.0 / 32768.0
}

pub fn normalize_angle(angle: f32) -> i16 {
    let mut out = angle;
    if out > 180.0 {
        out -= 360.0;
    }
    if out > 180.0 {
        out -= 360.0;
    }
    if out < -180.0 {
        out += 360.0;
    }
    (out * 32768.0 / 180.0) as i16
}

pub fn count_lines(text: &str) -> usize {
    if text.is_empty() {
        return 1;
    }
    let mut count = 1;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\n' && chars.peek().is_some_and(|next| *next != '\0') {
            count += 1;
        }
    }
    count
}

pub fn char_width(code: u32) -> i32 {
    let _ = code;
    16
}

pub fn value_increase(max: i32, value: i32, mode: u8) -> f32 {
    if max <= 0 {
        return 1.0;
    }
    let value = if value < 0 {
        0
    } else if value > max {
        max
    } else {
        value
    };
    let v = value as f32 / max as f32;
    match mode {
        0 => v * v,
        1 => v.sqrt(),
        3 => (v * 2.0 - 1.0) * 2.0 - 1.0,
        4 => (v * 0x800 * 0.5).sin() * (v * 0x800 * 0.5).sin(),
        5 => (v * 0xFFFF * 0.5).sin() * (v * 0xFFFF * 0.5).sin(),
        _ => v,
    }
}

pub fn row_label(row: OptionRow) -> &'static str {
    match row {
        OptionRow::Attention => "ttyu",
        OptionRow::Ruby => "trub",
        OptionRow::Vibration => "tvib",
        OptionRow::Sound => "tsou",
    }
}

pub fn option_entries() -> [OptionRow; 4] = [
    OptionRow::Attention,
    OptionRow::Ruby,
    OptionRow::Vibration,
    OptionRow::Sound,
]

pub fn option_messages(row: OptionRow) -> [u32; 3] {
    match row {
        OptionRow::Attention => [MSG_ATTENTION_0, MSG_ATTENTION_1, 0],
        OptionRow::Ruby => [MSG_ATTENTION_0, MSG_ATTENTION_1, 0],
        OptionRow::Vibration => [MSG_VIBRATION_0, MSG_VIBRATION_1, 0],
        OptionRow::Sound => [MSG_SOUND_0, MSG_SOUND_1, MSG_SOUND_2],
    }
}
