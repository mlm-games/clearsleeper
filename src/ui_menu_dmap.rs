use bevy_ecs::prelude::*;

use crate::j2d::{Kind, Screen, ScreenPane};
use crate::sim::flags::{DI_BOSS_KEY, DI_COMPASS, DI_MAP};
use crate::text::font::FontBank;
use crate::text::msgdata::MsgLenState;
use crate::text::msgget::{ItemMsgGet, MsgArchive};
use crate::text::msglayout::MsgLayoutState;
use crate::text::{MesgEntry, MsgData};

pub const SCREEN_ARCHIVE: &str = "dmapres.arc";
pub const SCREEN_LABEL: &str = "menu_map_d.blo";
pub const EXPLAIN_LABEL: &str = "menu_explanation.blo";
pub const ITEM_ICON_ARCHIVE: &str = "itemicon.arc";
pub const ITEM_ICON_TEXTURE: [&str; 3] = ["dungeon_map.bti", "boss_key.bti", "compass.bti"];
pub const ICON_BUFFER_SIZE: usize = 0xC00;

pub const TREASURE_TEXTURE: &str = "treasurebox.bti";
pub const DOOR_TEXTURE: &str = "black_white_3.bti";
pub const PRIEST_TEXTURE: &str = "f_otmicon.bti";
pub const BOSS_TEXTURE: &str = "boss_small.bti";
pub const CURSOR_TEXTURE_IN: &str = "cursor_00_01.bti";
pub const CURSOR_TEXTURE_OUT: &str = "cursor_00_02.bti";
pub const LINK_EYE_TEXTURE: [&str; 2] = ["link_eye1.bti", "link_eye2.bti"];
pub const LINK_MOUTH_TEXTURE: [&str; 2] = ["link_mouth1.bti", "link_mouth2.bti"];
pub const DEBUG_WINDOW_NAME: &str = "ダンジョンマップ画面";

pub const SCREEN_X: f32 = 320.0;
pub const SCREEN_Y: f32 = 240.0;
pub const TRANSITION_SLIDE: f32 = 480.0;
pub const MAP_DRAW_OFFSET_CLOSED: i16 = 0x1E0;
pub const OPEN_STEPS: i16 = 10;
pub const OPEN_STEP_LEN: i32 = 10;
pub const SCROLL_STEP_LEN: i32 = 6;
pub const NOTE_OPEN_STEPS: i16 = 17;
pub const NOTE_OPEN_HOLD: i16 = 18;
pub const NOTE_OPEN_TOTAL: i32 = 12;
pub const NOTE_OPEN_BIAS: f32 = 0.19999999;
pub const NOTE_OPEN_BASE: f32 = 0.8;
pub const NOTE_OPEN_RAMP: i16 = 5;
pub const NOTE_CLOSE_STEPS: i16 = 18;
pub const NOTE_CLOSE_LEN: i32 = 10;
pub const NOTE_CLOSE_LIMIT: i16 = 0x1C;
pub const NOTE_CLOSE_ANGLE: f32 = -117.0;
pub const NOTE_OPEN_ANGLE: f32 = -90.0;
pub const NOTE_EDGE_ANGLE: f32 = -3.0;
pub const NOTE_LINE_WIDTH: i32 = 0x1FE;
pub const NOTE_CENTER_LINE_WIDTH: i32 = 0x1E6;
pub const NOTE_SEND_SPEED: i32 = 2;
pub const NOTE_SPACE_TIMER: i32 = 0;
pub const NOTE_UNUSED_LINES: i32 = 3;
pub const RUBY_OPTION_DY: f32 = -4.0;

pub const MASK_ALPHA: u8 = 0x82;
pub const ITEM_SCALE: f32 = 1.4;
pub const FLOOR_SCALE: f32 = 1.15;
pub const CURSOR_STEP_FIRST: i16 = 10;
pub const CURSOR_STEP_MAX: i16 = 20;
pub const CURSOR_STEP_SIZE: i16 = 7;
pub const FLOOR_TEXTURE_BASE: i32 = 0x76;
pub const FLOOR_TEXTURE_COUNT: i32 = 20;
pub const FLOOR_LIST_SLOTS: f32 = 6.0;
pub const MAP_FLOOR_BASE: i32 = 0x80;
pub const RAND_SEED: [i32; 3] = [100, 100, 100];

pub const STAGE_TYPE_NONE: u8 = 0;
pub const STAGE_TYPE_BOSS: u8 = 3;
pub const STAGE_TYPE_MINIBOSS: u8 = 6;

pub const SAVE_SEA: u8 = 0;
pub const SAVE_SEA2: u8 = 1;
pub const SAVE_FF: u8 = 2;
pub const SAVE_DRC: u8 = 3;
pub const SAVE_FW: u8 = 4;
pub const SAVE_TOTG: u8 = 5;
pub const SAVE_ET: u8 = 6;
pub const SAVE_WT: u8 = 7;
pub const SAVE_GT: u8 = 8;

pub const MSG_NAME_FF: u32 = 0x320;
pub const MSG_NAME_DRC: u32 = 0x59;
pub const MSG_NAME_FW: u32 = 0x5A;
pub const MSG_NAME_TOTG: u32 = 0x5B;
pub const MSG_NAME_ET: u32 = 0x5C;
pub const MSG_NAME_WT: u32 = 0x5D;
pub const MSG_NAME_GT: u32 = 0x5E;
pub const MSG_NAME_DEFAULT: u32 = 0x59;

pub const MSG_ITEM_MAP: i16 = 0x197;
pub const MSG_ITEM_COMPASS: i16 = 0x198;
pub const MSG_ITEM_BOSS_KEY: i16 = 0x199;
pub const NOTE_MESSAGE_BASE: i16 = 0xC8;

pub const ICON_NONE: u8 = 0xFF;
pub const ICON_INPUT: u8 = 0x16;
pub const ICON_ARROW_LEFT: u8 = 0x0A;
pub const ICON_ARROW_RIGHT: u8 = 0x0B;
pub const ICON_ARROW_UP: u8 = 0x0C;
pub const ICON_ARROW_DOWN: u8 = 0x0D;
pub const ICON_FLASHING_A_BUTTON_2: u8 = 0x17;
pub const ICON_HEART_2: u8 = 0x18;
pub const ICON_MUSIC_NOTE_2: u8 = 0x19;
pub const ICON_STARBURST_2: u8 = 0x1A;

pub const ICON_TEXTURE: [&str; 0x18] = [
    "font_00.bti",
    "font_01.bti",
    "font_09.bti",
    "font_04.bti",
    "font_05.bti",
    "font_02.bti",
    "font_03.bti",
    "font_06.bti",
    "font_08.bti",
    "font_07_01.bti",
    "font_10.bti",
    "font_10.bti",
    "font_10.bti",
    "font_10.bti",
    "font_07_01.bti",
    "font_07_01.bti",
    "font_07_01.bti",
    "font_07_01.bti",
    "font_07_01.bti",
    "font_07_01.bti",
    "font_12.bti",
    "font_13.bti",
    "font_14.bti",
    "font_15.bti",
];

pub const ICON_COLOR: [[u8; 4]; 0x18] = [
    [0x00, 0xFF, 0xB4, 0xFF],
    [0xFF, 0x32, 0x32, 0xFF],
    [0xFF, 0xC8, 0x32, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xC8, 0x96, 0xFF, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xFF, 0xC8, 0x32, 0xFF],
    [0xFF, 0xC8, 0x32, 0xFF],
    [0xFF, 0xC8, 0x32, 0xFF],
    [0xFF, 0xC8, 0x32, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0xC8, 0xC8, 0xC8, 0xFF],
    [0x00, 0xFF, 0xB4, 0xFF],
    [0xFF, 0x32, 0x32, 0xFF],
    [0xFF, 0xFF, 0xFF, 0xFF],
    [0xFF, 0xFF, 0xFF, 0xFF],
];

pub const ICON_COLOR_USES_TEXT: [u8; 6] = [
    ICON_ARROW_LEFT,
    ICON_ARROW_RIGHT,
    ICON_ARROW_UP,
    ICON_ARROW_DOWN,
    0x15,
    0x17,
];

pub const SE_MENU_CURSOR: u32 = 0x080E;
pub const SE_MENU_ITEMS_IN: u32 = 0x0813;
pub const SE_MENU_EXP_IN: u32 = 0x0814;
pub const SE_MENU_EXP_OUT: u32 = 0x0816;
pub const SE_ITEM_EXP_OPEN: u32 = 0x08B6;

pub const DO_STATUS_BLANK: u8 = 0x00;
pub const DO_STATUS_INFO: u8 = 0x21;

pub const FLOOR_SLOT: u8 = 3;
pub const TREASURE_COUNT: usize = 32;
pub const DOOR_COUNT: usize = 32;
pub const OUT_FONT_COUNT: usize = 15;
pub const FLOOR_ROW_COUNT: usize = 6;
pub const CURSOR_SLOT_COUNT: usize = 4;
pub const ITEM_ROW_COUNT: usize = 3;
pub const ROW_COUNT: usize = 3;
pub const CC_COUNT: usize = 7;
pub const CC_SCROLL: usize = 0;
pub const CC_ALPHA: usize = 5;
pub const NAME_BUFFER_COUNT: usize = 2;
pub const DUNGEON_NAME_SIZE: usize = 12;
pub const BOSS_FLOOR_NONE: u8 = 0xFF;
pub const INIT_ALPHA_SLOT: u8 = 0xFF;

pub const COLLECT_SLOT: u8 = 0;
pub const COLLECT_PRIEST_SAVE: u8 = 2;
pub const COLLECT_PRIEST_TUNIC: u8 = 3;
pub const PRIEST_FLAG_NONE: u8 = 0;
pub const PRIEST_FLAG_TUNIC: u8 = 1;
pub const PRIEST_FLAG_SAVE: u8 = 2;

pub const VISITED_ROOM_WORDS: usize = 2;
pub const VISITED_ROOM_LIMIT: i32 = 64;
pub const TBOX_LIMIT: i32 = 32;
pub const TBOX_SKIP_A: i32 = 0xF;
pub const TBOX_SKIP_B: i32 = 0x10;
pub const TBOX_SHIFT: u32 = 7;
pub const TBOX_MASK: i32 = 0x1F;
pub const DOOR_ROOM_MASK: i32 = 0x3F;
pub const DOOR_ROOM_SHIFT: u32 = 6;
pub const COLLECT_ITEMS: u8 = 8;

pub const BOSS_DOOR_NAME_A: &str = "door20";
pub const BOSS_DOOR_NAME_B: &str = "door12";
pub const BOSS_DOOR_Z_ANGLE_A: i16 = 12;
pub const BOSS_DOOR_Z_ANGLE_B: i16 = 9;

pub const S2DEG_CONSTANT: f32 = 180.0 / 0x8000;
pub const DEG2S_CONSTANT: f32 = 32768.0 / 180.0;

pub const STICK_FIRST: i16 = 5;
pub const STICK_SECOND: i16 = 2;
pub const STICK_STEP: i16 = 3;
pub const STICK_MIN_STEP: i16 = 2;
pub const STICK_MAX_VALUE: f32 = 1.0;
pub const STICK_MIN_VALUE: f32 = -1.0;
pub const STICK_HALF_DEAD: i16 = 0x1000;
pub const STICK_HALF: i16 = 0x7000;

pub const FT_TAGS: [&str; 15] = [
    "ft00", "ft01", "ft02", "ft03", "ft04", "ft05", "ft06", "ft07", "ft08", "ft09", "ft10", "ft11",
    "ft12", "ft13", "ft14",
];

pub const FD_TAGS: [&str; 15] = [
    "fd00", "fd01", "fd02", "fd03", "fd04", "fd05", "fd06", "fd07", "fd08", "fd09", "fd10", "fd11",
    "fd12", "fd13", "fd14",
];

pub const CAR_TAGS: [&str; 4] = ["car1", "car2", "car3", "car4"];
pub const IT_TAGS: [&str; 3] = ["it00", "it01", "it02"];
pub const IK_TAGS: [&str; 3] = ["ik00", "ik01", "ik02"];
pub const IP_TAGS: [&str; 3] = ["ip06", "ip04", "ip02"];
pub const IP01_TAG: &str = "ip01";
pub const LNK_TAGS: [&str; 3] = ["lnk0", "lnk1", "lnk2"];
pub const FL_TAGS: [&str; 6] = ["f001", "f002", "f003", "f004", "f005", "f006"];
pub const FB_TAGS: [&str; 6] = ["fb01", "fb02", "fb03", "fb04", "fb05", "fb06"];
pub const FBK_TAGS: [&str; 6] = ["fbk1", "fbk2", "fbk3", "fbk4", "fbk5", "fbk6"];
pub const CC_TAGS: [&str; 7] = ["cc38", "cc31", "cc24", "cc17", "cc10", "cc05", "cc00"];

pub const BEY1_TAG: &str = "bey1";
pub const BEY2_TAG: &str = "bey2";
pub const BOSS_TAG: &str = "boss";
pub const BOS2_TAG: &str = "bos2";
pub const FW00_TAG: &str = "fw00";
pub const FW01_TAG: &str = "fw01";
pub const MSK_TAG: &str = "msk";
pub const MPMK_TAG: &str = "mpmk";
pub const MPP1_TAG: &str = "mpp1";
pub const MPP2_TAG: &str = "mpp2";
pub const MPP0_TAG: &str = "mpp0";
pub const NM00_TAG: &str = "nm00";
pub const NM01_TAG: &str = "nm01";
pub const ITNM_TAG: &str = "itnm";
pub const ITNK_TAG: &str = "itnk";
pub const STR0_TAG: &str = "str0";
pub const ST00_TAG: &str = "st00";
pub const NT00_TAG: &str = "nt00";
pub const NK00_TAG: &str = "nk00";
pub const NO11_TAG: &str = "no11";
pub const MSK0_TAG: &str = "msk0";
pub const DTLE_TAG: &str = "dtle";
pub const DT00_TAG: &str = "dt00";
pub const DK00_TAG: &str = "dk00";

pub const FLOOR_TEXTURE: [&str; 20] = [
    "hierarchy_num_B3.bti",
    "hierarchy_num_B3.bti",
    "hierarchy_num_B3.bti",
    "hierarchy_num_B3.bti",
    "hierarchy_num_B3.bti",
    "hierarchy_num_B3.bti",
    "hierarchy_num_B3.bti",
    "hierarchy_num_B3.bti",
    "hierarchy_num_B2.bti",
    "hierarchy_num_B1.bti",
    "hierarchy_num_1.bti",
    "hierarchy_num_2.bti",
    "hierarchy_num_3.bti",
    "hierarchy_num_4.bti",
    "hierarchy_num_5.bti",
    "hierarchy_num_6.bti",
    "hierarchy_num_7.bti",
    "hierarchy_num_7.bti",
    "hierarchy_num_7.bti",
    "hierarchy_num_7.bti",
];

pub const BOSS_OFFSET_X: [f32; 16] = [
    0.0, 0.0, 0.0, 0.0, 20.0, 21.0, -21.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
];

pub const BOSS_OFFSET_Y: [f32; 16] = [
    0.0, 0.0, 0.0, -22.0, 0.0, 0.0, 0.0, 21.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
];

pub const BOSS_ROW_TIME: [i16; 15] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2];

pub const BOSS_ANIM_STEP: f32 = 2.0;

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
        4 => {
            let s = (v * 0x8000 as f32 * 0.5).sin();
            s * s
        }
        5 => {
            let s = (v * 0xFFFF as f32 * 0.5).sin();
            s * s
        }
        _ => v,
    }
}

pub fn deg2s(deg: f32) -> i16 {
    (deg * DEG2S_CONSTANT) as i16
}

pub fn s2deg(angle: i16) -> f32 {
    f32::from(angle) * S2DEG_CONSTANT
}

pub fn rotation_to_deg(rotation: u16) -> i16 {
    (f32::from(rotation) * (360.0 / 65536.0)) as i16
}

pub fn pack_color(c: [u8; 4]) -> u32 {
    (u32::from(c[0]) << 24) | (u32::from(c[1]) << 16) | (u32::from(c[2]) << 8) | u32::from(c[3])
}

pub fn unpack_color(color: u32) -> [u8; 4] {
    [
        (color >> 24) as u8,
        (color >> 16) as u8,
        (color >> 8) as u8,
        color as u8,
    ]
}

pub fn icon_texture(icon: u8) -> &'static str {
    ICON_TEXTURE
        .get(usize::from(icon))
        .copied()
        .unwrap_or(ICON_TEXTURE[0])
}

pub fn icon_color(icon: u8) -> u32 {
    ICON_COLOR
        .get(usize::from(icon))
        .map(|c| pack_color(*c))
        .unwrap_or(0xFFFF_FFFF)
}

pub fn icon_uses_text_color(icon: u8) -> bool {
    ICON_COLOR_USES_TEXT.contains(&icon)
}

pub fn out_font_slot(icon: u8) -> u8 {
    match icon {
        ICON_FLASHING_A_BUTTON_2 => 0x14,
        ICON_HEART_2 => 0x15,
        ICON_MUSIC_NOTE_2 => 0x16,
        ICON_STARBURST_2 => 0x17,
        other => other,
    }
}

pub fn dungeon_name_message(save_index: u8) -> u32 {
    match save_index {
        SAVE_FF => MSG_NAME_FF,
        SAVE_DRC => MSG_NAME_DRC,
        SAVE_FW => MSG_NAME_FW,
        SAVE_TOTG => MSG_NAME_TOTG,
        SAVE_ET => MSG_NAME_ET,
        SAVE_WT => MSG_NAME_WT,
        SAVE_GT => MSG_NAME_GT,
        _ => MSG_NAME_DEFAULT,
    }
}

pub fn item_name_message(
    select_item: u8,
    has_map: bool,
    has_boss_key: bool,
    has_compass: bool,
) -> u32 {
    match select_item {
        0 => {
            if has_map {
                u32::from(MSG_ITEM_MAP as u16)
            } else {
                0
            }
        }
        1 => {
            if has_boss_key {
                u32::from(MSG_ITEM_BOSS_KEY as u16)
            } else {
                0
            }
        }
        2 => {
            if has_compass {
                u32::from(MSG_ITEM_COMPASS as u16)
            } else {
                0
            }
        }
        _ => 0,
    }
}

pub fn item_note_message(
    select_item: u8,
    has_map: bool,
    has_boss_key: bool,
    has_compass: bool,
) -> u32 {
    match select_item {
        0 => {
            if has_map {
                u32::from(MSG_ITEM_MAP.wrapping_add(NOTE_MESSAGE_BASE) as u16)
            } else {
                0
            }
        }
        1 => {
            if has_boss_key {
                u32::from(MSG_ITEM_BOSS_KEY.wrapping_add(NOTE_MESSAGE_BASE) as u16)
            } else {
                0
            }
        }
        2 => {
            if has_compass {
                u32::from(MSG_ITEM_COMPASS.wrapping_add(NOTE_MESSAGE_BASE) as u16)
            } else {
                0
            }
        }
        _ => 0,
    }
}

pub fn is_boss_door(name: &str, angle: [i16; 3]) -> bool {
    if name == BOSS_DOOR_NAME_A {
        return true;
    }
    if name == BOSS_DOOR_NAME_B {
        match ((u32::from(angle[2]) >> 8) & 0xFF) as i16 {
            BOSS_DOOR_Z_ANGLE_A | BOSS_DOOR_Z_ANGLE_B => return true,
            _ => {}
        }
    }
    false
}

pub fn font_cell_width(font: &FontBank, slot: usize) -> f32 {
    font.get(slot)
        .map_or(0.0, |res| res.metrics().cell_width as f32)
}

pub fn font_char_width(font: &FontBank, slot: usize, code: u16) -> i32 {
    font.get(slot).map_or(0, |res| res.char_width(code))
}

pub fn font_char_offset(font: &FontBank, slot: usize, code: u16) -> i32 {
    font.get(slot).map_or(0, |res| res.char_offset(code))
}

fn append_bytes(dst: &mut Vec<u8>, src: &[u8], limit: usize) {
    for byte in src {
        if dst.len() + 1 >= limit {
            break;
        }
        dst.push(*byte);
    }
}

pub fn message_name_walk(
    message: &[u8],
    font: &FontBank,
    slot: usize,
    size_y: f32,
    limit: usize,
) -> (Vec<u8>, f32) {
    let mut name: Vec<u8> = Vec::with_capacity(limit);
    let mut str_width = 0.0f32;
    let mut first = false;
    let cell = font_cell_width(font, slot);
    let scale = if cell != 0.0 { size_y / cell } else { 0.0 };
    let mut at = 0usize;
    while message.get(at).copied().unwrap_or(0) != 0 {
        let c = message[at];
        if c == 0x1A {
            at += 1;
            let skip = i32::from(message.get(at).copied().unwrap_or(0)) - 1;
            at = (at as i32 + skip).max(0) as usize;
        } else {
            let hi_nibble = (c >> 4) & 0xF;
            let code;
            if hi_nibble == 8 || hi_nibble == 9 {
                let second = message.get(at + 1).copied().unwrap_or(0);
                code = (u32::from(second) | (u32::from(c) << 8)) as u16;
                append_bytes(&mut name, &[c, second], limit);
                at += 2;
            } else {
                code = u16::from(c);
                append_bytes(&mut name, &[c], limit);
                at += 1;
            }
            let width = font_char_width(font, slot, code);
            if !first {
                str_width =
                    scale * (f32::from(width) + f32::from(font_char_offset(font, slot, code)));
                first = true;
            } else {
                str_width += f32::from(width) * scale;
            }
        }
    }
    (name, str_width)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rng {
    pub r0: i32,
    pub r1: i32,
    pub r2: i32,
}

impl Default for Rng {
    fn default() -> Self {
        Self::new(RAND_SEED[0], RAND_SEED[1], RAND_SEED[2])
    }
}

impl Rng {
    pub fn new(r0: i32, r1: i32, r2: i32) -> Self {
        Self { r0, r1, r2 }
    }

    pub fn rnd(&mut self) -> f32 {
        self.r0 = (self.r0 * 0xAB) % 0x763D;
        self.r1 = (self.r1 * 0xAC) % 0x7663;
        self.r2 = (self.r2 * 0xAA) % 0x7673;
        let sum = self.r0 as f32 / 30269.0 + self.r1 as f32 / 30307.0 + self.r2 as f32 / 30323.0;
        (sum % 1.0).abs()
    }

    pub fn rnd_f(&mut self, max: f32) -> f32 {
        self.rnd() * max
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct F2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ContentsColor {
    pub tl: [u8; 4],
    pub tr: [u8; 4],
    pub bl: [u8; 4],
    pub br: [u8; 4],
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DrawItem {
    pub tag: &'static str,
    pub kind: Kind,
    pub visible: bool,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub alpha: u8,
    pub rotation: u16,
    pub texture: Option<&'static str>,
    pub color: [u8; 4],
    pub color_black: [u8; 4],
    pub blend: [f32; 4],
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
    pub alpha: u8,
    pub user_area: i16,
    pub texture: Option<&'static str>,
    pub font: Option<usize>,
    pub white: u32,
    pub black: u32,
    pub char_color: u32,
    pub grad_color: u32,
    pub rotate_x: f32,
    pub rotate_y: f32,
    pub rotation: u16,
    pub blend: [f32; 4],
    pub contents_color: ContentsColor,
    pub shift_x: f32,
    pub shift_y: f32,
    pub text: String,
    pub font_size_x: f32,
    pub font_size_y: f32,
    pub char_space: f32,
    pub line_space: f32,
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
            alpha: 0,
            user_area: 0,
            texture: None,
            font: None,
            white: 0xFFFF_FFFF,
            black: 0x0000_0000,
            char_color: 0xFFFF_FFFF,
            grad_color: 0xFFFF_FFFF,
            rotate_x: 0.0,
            rotate_y: 0.0,
            rotation: 0,
            blend: [0.0, 1.0, 1.0, 1.0],
            contents_color: ContentsColor::default(),
            shift_x: 0.0,
            shift_y: 0.0,
            text: String::new(),
            font_size_x: 0.0,
            font_size_y: 0.0,
            char_space: 0.0,
            line_space: 0.0,
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

    pub fn bind(&mut self, screen: &Screen) {
        if let Some(layout) = screen.find(self.tag) {
            self.parts_set(layout);
        }
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
        self.alpha = 0;
        self.user_area = 0;
        self.rotation = layout.rotation;
        self.rotate_x = 0.0;
        self.rotate_y = 0.0;
        self.texture = None;
        self.char_color = layout.char_color;
        self.grad_color = layout.grad_color;
        self.text = layout.text.clone();
        self.font_size_x = f32::from(layout.font_size_x);
        self.font_size_y = f32::from(layout.font_size_y);
        self.char_space = f32::from(layout.char_space);
        self.line_space = f32::from(layout.line_space);
        self.shift_x = 0.0;
        self.shift_y = 0.0;
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn change_texture(&mut self, texture: &'static str) {
        self.texture = Some(texture);
    }

    pub fn blend_draw(&mut self, texture: &'static str) {
        self.show();
        self.texture = Some(texture);
    }

    pub fn set_font(&mut self, slot: usize) {
        self.font = Some(slot);
    }

    pub fn set_white(&mut self, color: u32) {
        self.white = color;
    }

    pub fn set_black(&mut self, color: u32) {
        self.black = color;
    }

    pub fn set_char_color(&mut self, color: u32) {
        self.char_color = color;
    }

    pub fn set_grad_color(&mut self, color: u32) {
        self.grad_color = color;
    }

    pub fn set_contents_color(&mut self, color: ContentsColor) {
        self.contents_color = color;
    }

    pub fn set_blend(&mut self, blend: [f32; 4]) {
        self.blend = blend;
    }

    pub fn set_alpha(&mut self) {
        self.alpha = self.now_alpha;
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

    pub fn dec_alpha(&mut self, alpha: f32) {
        self.now_alpha = (f32::from(self.now_alpha) * alpha) as u8;
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

    pub fn resize_orig(&mut self, width: f32, height: f32) {
        self.size_orig.x = width;
        self.size_orig.y = height;
        self.size.x = width;
        self.size.y = height;
    }

    pub fn rotate(&mut self, x: f32, y: f32, angle: f32) {
        self.rotate_x = x;
        self.rotate_y = y;
        self.rotation = (deg2s(angle) as u16) & 0xFFFF;
    }

    pub fn draw_item(&self) -> DrawItem {
        DrawItem {
            tag: self.tag,
            kind: self.kind,
            visible: self.visible,
            x: self.pos_top_left.x,
            y: self.pos_top_left.y,
            width: self.size.x,
            height: self.size.y,
            alpha: self.alpha,
            rotation: self.rotation,
            texture: self.texture,
            color: unpack_color(self.white),
            color_black: unpack_color(self.black),
            blend: self.blend,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapDriver {
    pub created: bool,
    pub archive_bound: bool,
    pub mask_pos_x: i16,
    pub mask_pos_y: i16,
    pub mask_width: i16,
    pub mask_height: i16,
    pub map_pos_x: i16,
    pub map_pos_y: i16,
    pub pane_width: i16,
    pub pane_height: i16,
    pub gap_y: i16,
    pub top_floor: u8,
    pub bottom_floor: u8,
    pub stay_room: u8,
    pub floor_no: u8,
    pub alpha_slot: u8,
    pub alpha: u8,
    pub tlut_double_buffer: i32,
    pub draw_calls: u32,
}

impl MapDriver {
    pub fn set_archive(&mut self) {
        self.archive_bound = true;
    }

    pub fn init(&mut self, rect: [i16; 4], player: [i16; 4], gap_y: i16, floors: [u8; 5]) {
        self.created = true;
        self.mask_pos_x = rect[0];
        self.mask_pos_y = rect[1];
        self.mask_width = rect[2];
        self.mask_height = rect[3];
        self.map_pos_x = player[0];
        self.map_pos_y = player[1];
        self.pane_width = player[2];
        self.pane_height = player[3];
        self.gap_y = gap_y;
        self.top_floor = floors[0];
        self.bottom_floor = floors[1];
        self.stay_room = floors[2];
        self.floor_no = floors[3];
        self.alpha_slot = floors[4];
        self.alpha = 0;
        self.tlut_double_buffer = 0;
    }

    pub fn set_mask_pos(&mut self, x: i16, y: i16) {
        self.mask_pos_x = x;
        self.mask_pos_y = y;
    }

    pub fn set_map_pos(&mut self, x: i16, y: i16) {
        self.map_pos_x = x;
        self.map_pos_y = y;
    }

    pub fn set_alpha(&mut self, alpha: u8) {
        self.alpha = alpha;
    }

    pub fn change_tlut_double_buffer(&mut self) {
        self.tlut_double_buffer = 1 - self.tlut_double_buffer;
    }

    pub fn draw(&mut self) {
        self.change_tlut_double_buffer();
        self.draw_calls += 1;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapMark {
    pub texture: &'static str,
    pub pos_x: f32,
    pub pos_y: f32,
    pub width: f32,
    pub height: f32,
    pub angle: i16,
    pub exists: u8,
    pub visible: bool,
    pub white: u32,
    pub black: u32,
    pub alpha: u8,
    pub rotate_x: f32,
    pub rotate_y: f32,
    pub rotation: u16,
}

impl MapMark {
    pub fn new(texture: &'static str) -> Self {
        Self {
            texture,
            pos_x: 0.0,
            pos_y: 0.0,
            width: 0.0,
            height: 0.0,
            angle: 0,
            exists: 0,
            visible: true,
            white: 0xFFFF_FFFF,
            black: 0x0000_0000,
            alpha: 0,
            rotate_x: 0.0,
            rotate_y: 0.0,
            rotation: 0,
        }
    }

    pub fn set_white(&mut self, color: u32) {
        self.white = color;
    }

    pub fn set_black(&mut self, color: u32) {
        self.black = color;
    }

    pub fn set_alpha(&mut self, alpha: u8) {
        self.alpha = alpha;
    }

    pub fn rotate(&mut self, x: f32, y: f32, angle: f32) {
        self.rotate_x = x;
        self.rotate_y = y;
        self.rotation = (deg2s(angle) as u16) & 0xFFFF;
    }

    pub fn draw_item(&self, x: f32, y: f32) -> DrawItem {
        DrawItem {
            tag: self.texture,
            kind: Kind::Picture,
            visible: self.visible,
            x,
            y,
            width: self.width,
            height: self.height,
            alpha: self.alpha,
            rotation: self.rotation,
            texture: Some(self.texture),
            color: unpack_color(self.white),
            color_black: unpack_color(self.black),
            blend: [0.0, 1.0, 1.0, 1.0],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapTuning {
    pub boss_ofs_x: i16,
    pub boss_ofs_y: i16,
    pub treasure_width: i16,
    pub treasure_height: i16,
    pub treasure_white: u32,
    pub treasure_black: u32,
    pub boss_width: i16,
    pub boss_height: i16,
    pub boss_white: u32,
    pub boss_black: u32,
    pub door_width: i16,
    pub door_height: i16,
    pub door_white: u32,
    pub door_black: u32,
    pub npc_width: i16,
    pub npc_height: i16,
    pub npc_white: u32,
    pub npc_black: u32,
    pub boss_up_time: i16,
    pub boss_wait_time: i16,
    pub boss_down_time: i16,
    pub boss_move_time1: i16,
    pub boss_move_time2: i16,
    pub boss_move_time3: i16,
    pub boss_move_time4: i16,
}

impl Default for MapTuning {
    fn default() -> Self {
        Self {
            boss_ofs_x: 0,
            boss_ofs_y: 0,
            treasure_width: 14,
            treasure_height: 14,
            treasure_white: pack_color([0xFF, 0xC4, 0x00, 0xFF]),
            treasure_black: pack_color([0x20, 0x20, 0x00, 0x00]),
            boss_width: 32,
            boss_height: 32,
            boss_white: pack_color([0xFF, 0xFF, 0xFF, 0xFF]),
            boss_black: pack_color([0x00, 0x00, 0x00, 0x00]),
            door_width: 16,
            door_height: 8,
            door_white: pack_color([0xA8, 0xA8, 0xA8, 0xFF]),
            door_black: pack_color([0x40, 0x20, 0x00, 0x00]),
            npc_width: 16,
            npc_height: 16,
            npc_white: pack_color([0xFF, 0x00, 0xFF, 0xFF]),
            npc_black: pack_color([0x00, 0x00, 0x00, 0x00]),
            boss_up_time: 4,
            boss_wait_time: 10,
            boss_down_time: 2,
            boss_move_time1: 2,
            boss_move_time2: 2,
            boss_move_time3: 2,
            boss_move_time4: 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StagePrm {
    pub parameters: u32,
    pub position: [f32; 3],
    pub angle: [i16; 3],
    pub set_id: u16,
}

pub fn stage_name(name: &[u8; 8]) -> String {
    let end = name
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(name.len());
    String::from_utf8_lossy(&name[..end]).into_owned()
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DoorEntry {
    pub name: [u8; 8],
    pub base: StagePrm,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TreasureEntry {
    pub name: [u8; 8],
    pub base: StagePrm,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StageMapEntry {
    pub origin_x: f32,
    pub origin_z: f32,
    pub scale: f32,
    pub offset_y: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StageMapInfo {
    pub entries: Vec<StageMapEntry>,
}

pub trait StageFloors {
    fn floor_no(&self, y: f32) -> u8;
    fn floor_no_for_room(&self, room: i32, y: f32) -> u8;
    fn top_bottom_floor(&self, highest: &mut u8, lowest: &mut u8) -> bool;
}

pub struct MapEnv<'a> {
    pub floors: &'a dyn StageFloors,
    pub player_pos: [f32; 3],
    pub priest_flag: u8,
    pub priest_pos: [f32; 3],
    pub cb1_player_pos: Option<[f32; 3]>,
    pub stage_type: u8,
    pub save_index: u8,
    pub stage_map: Option<&'a StageMapInfo>,
    pub stay_room: i32,
    pub doors: &'a [DoorEntry],
    pub treasures: &'a [TreasureEntry],
    pub dungeon_item: u8,
    pub visited_room: [u8; VISITED_ROOM_WORDS],
    pub tbox_bits: u32,
    pub collect_bits: [u8; 8],
    pub opt_ruby: bool,
}

impl MapEnv<'_> {
    pub fn has_map(&self) -> bool {
        self.dungeon_item & (1 << DI_MAP) != 0
    }

    pub fn has_compass(&self) -> bool {
        self.dungeon_item & (1 << DI_COMPASS) != 0
    }

    pub fn has_boss_key(&self) -> bool {
        self.dungeon_item & (1 << DI_BOSS_KEY) != 0
    }

    pub fn is_visited_room(&self, no: i32) -> bool {
        if no < 0 || no >= VISITED_ROOM_LIMIT {
            return false;
        }
        let word = self
            .visited_room
            .get((no >> 5) as usize)
            .copied()
            .unwrap_or(0);
        word & (1 << (no & 0x1F)) != 0
    }

    pub fn is_tbox(&self, no: i32) -> bool {
        if no < 0 || no >= TBOX_LIMIT {
            return false;
        }
        self.tbox_bits & (1u32 << no) != 0
    }

    pub fn is_collect(&self, slot: u8, item: u8) -> bool {
        if item >= COLLECT_ITEMS {
            return false;
        }
        self.collect_bits
            .get(usize::from(slot))
            .is_some_and(|byte| byte & (1 << item) != 0)
    }

    pub fn stage_map_offset_y(&self) -> f32 {
        let mut ret = 0.0;
        if self.stage_type == STAGE_TYPE_BOSS || self.stage_type == STAGE_TYPE_MINIBOSS {
            if let Some(info) = self.stage_map {
                for entry in &info.entries {
                    ret = entry.offset_y;
                }
            }
        }
        ret
    }

    pub fn stage_map_basis(&self) -> (f32, f32, f32) {
        let mut origin_x = 0.0;
        let mut origin_z = 0.0;
        let mut scale = 0.0;
        if let Some(info) = self.stage_map {
            for entry in &info.entries {
                origin_x = entry.origin_x;
                origin_z = entry.origin_z;
                scale = entry.scale;
            }
        }
        (origin_x, origin_z, scale)
    }
}

pub struct MapText<'a> {
    pub archive: &'a dyn MsgArchive,
    pub clear_count: u8,
    pub font: &'a FontBank,
    pub main_font_slot: usize,
    pub ruby_font_slot: usize,
    pub note_font_size: i16,
    pub note_line_space: i16,
    pub len_state: MsgLenState<'a>,
    pub layout_state: MsgLayoutState<'a>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PadInput {
    pub trig_a: bool,
    pub trig_b: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub stick_value: f32,
    pub stick_angle: i16,
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
        Self::new(STICK_FIRST, STICK_SECOND, STICK_STEP, STICK_MIN_STEP)
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
            if angle < base - STICK_HALF + half {
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

#[derive(Clone, Debug)]
pub struct MapPanes {
    pub ft: [Pane; OUT_FONT_COUNT],
    pub fd: [Pane; OUT_FONT_COUNT],
    pub car: [Pane; CURSOR_SLOT_COUNT],
    pub nm00: Pane,
    pub nm01: Pane,
    pub itnm: Pane,
    pub itnk: Pane,
    pub msk0: Pane,
    pub str0: Pane,
    pub st00: Pane,
    pub nt00: Pane,
    pub nk00: Pane,
    pub no11: Pane,
    pub dtle: Pane,
    pub dt00: Pane,
    pub dk00: Pane,
    pub it: [Pane; ITEM_ROW_COUNT],
    pub ik: [Pane; ITEM_ROW_COUNT],
    pub ip: [Pane; ITEM_ROW_COUNT],
    pub ip01: Pane,
    pub lnk: [Pane; ROW_COUNT],
    pub bey1: Pane,
    pub bey2: Pane,
    pub boss: Pane,
    pub bos2: Pane,
    pub fl: [Pane; FLOOR_ROW_COUNT],
    pub fb: [Pane; FLOOR_ROW_COUNT],
    pub fbk: [Pane; FLOOR_ROW_COUNT],
    pub fw00: Pane,
    pub fw01: Pane,
    pub cc: [Pane; CC_COUNT],
    pub msk: Pane,
    pub marker_mask: Pane,
    pub player_pos: Pane,
    pub map_pos_prev: Pane,
    pub map_pos_next: Pane,
}

impl Default for MapPanes {
    fn default() -> Self {
        Self {
            ft: std::array::from_fn(|i| Pane::new(FT_TAGS[i])),
            fd: std::array::from_fn(|i| Pane::new(FD_TAGS[i])),
            car: std::array::from_fn(|i| Pane::new(CAR_TAGS[i])),
            nm00: Pane::new(NM00_TAG),
            nm01: Pane::new(NM01_TAG),
            itnm: Pane::new(ITNM_TAG),
            itnk: Pane::new(ITNK_TAG),
            msk0: Pane::new(MSK0_TAG),
            str0: Pane::new(STR0_TAG),
            st00: Pane::new(ST00_TAG),
            nt00: Pane::new(NT00_TAG),
            nk00: Pane::new(NK00_TAG),
            no11: Pane::new(NO11_TAG),
            dtle: Pane::new(DTLE_TAG),
            dt00: Pane::new(DT00_TAG),
            dk00: Pane::new(DK00_TAG),
            it: std::array::from_fn(|i| Pane::new(IT_TAGS[i])),
            ik: std::array::from_fn(|i| Pane::new(IK_TAGS[i])),
            ip: std::array::from_fn(|i| Pane::new(IP_TAGS[i])),
            ip01: Pane::new(IP01_TAG),
            lnk: std::array::from_fn(|i| Pane::new(LNK_TAGS[i])),
            bey1: Pane::new(BEY1_TAG),
            bey2: Pane::new(BEY2_TAG),
            boss: Pane::new(BOSS_TAG),
            bos2: Pane::new(BOS2_TAG),
            fl: std::array::from_fn(|i| Pane::new(FL_TAGS[i])),
            fb: std::array::from_fn(|i| Pane::new(FB_TAGS[i])),
            fbk: std::array::from_fn(|i| Pane::new(FBK_TAGS[i])),
            fw00: Pane::new(FW00_TAG),
            fw01: Pane::new(FW01_TAG),
            cc: std::array::from_fn(|i| Pane::new(CC_TAGS[i])),
            msk: Pane::new(MSK_TAG),
            marker_mask: Pane::new(MPMK_TAG),
            player_pos: Pane::new(MPP1_TAG),
            map_pos_prev: Pane::new(MPP2_TAG),
            map_pos_next: Pane::new(MPP0_TAG),
        }
    }
}

impl MapPanes {
    pub fn bind(&mut self, screen: &Screen, explain: &Screen) {
        for pane in self.ft.iter_mut() {
            pane.bind(explain);
        }
        for pane in self.fd.iter_mut() {
            pane.bind(explain);
        }
        self.nm00.bind(explain);
        self.nm01.bind(explain);
        self.itnm.bind(explain);
        self.itnk.bind(explain);
        self.msk0.bind(explain);
        self.str0.bind(explain);
        self.st00.bind(explain);
        self.nt00.bind(explain);
        self.nk00.bind(explain);
        self.no11.bind(explain);
        self.dtle.bind(explain);
        self.dt00.bind(explain);
        self.dk00.bind(explain);
        for pane in self.car.iter_mut() {
            pane.bind(screen);
        }
        for pane in self.it.iter_mut() {
            pane.bind(screen);
        }
        for pane in self.ik.iter_mut() {
            pane.bind(screen);
        }
        for pane in self.ip.iter_mut() {
            pane.bind(screen);
        }
        self.ip01.bind(screen);
        for pane in self.lnk.iter_mut() {
            pane.bind(screen);
        }
        self.bey1.bind(screen);
        self.bey2.bind(screen);
        self.boss.bind(screen);
        self.bos2.bind(screen);
        for pane in self.fl.iter_mut() {
            pane.bind(screen);
        }
        for pane in self.fb.iter_mut() {
            pane.bind(screen);
        }
        for pane in self.fbk.iter_mut() {
            pane.bind(screen);
        }
        self.fw00.bind(screen);
        self.fw01.bind(screen);
        for pane in self.cc.iter_mut() {
            pane.bind(screen);
        }
        self.msk.bind(screen);
        self.marker_mask.bind(screen);
        self.player_pos.bind(screen);
        self.map_pos_prev.bind(screen);
        self.map_pos_next.bind(screen);
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct MapOutput {
    pub sounds: Vec<u32>,
    pub do_status_force: Option<u8>,
    pub do_status: Option<u8>,
    pub frames: u32,
}

impl MapOutput {
    pub fn reset(&mut self) {
        self.sounds.clear();
        self.do_status_force = None;
        self.do_status = None;
        self.frames = 0;
    }
}

#[derive(Clone, Debug, Default)]
pub struct MapDraw {
    pub main: Vec<DrawItem>,
    pub map: MapDriver,
    pub markers: Vec<DrawItem>,
    pub explain: Vec<DrawItem>,
}

#[derive(Resource, Clone, Debug)]
pub struct DungeonMap {
    pub screen: Option<Screen>,
    pub explain_screen: Option<Screen>,
    pub panes: MapPanes,
    pub stick: StickControl,
    pub map: MapDriver,
    pub tuning: MapTuning,
    pub rng: Rng,
    pub treasure: Vec<MapMark>,
    pub doors: Vec<MapMark>,
    pub npc: MapMark,
    pub boss: MapMark,
    pub item_icon: [&'static str; ITEM_ROW_COUNT],
    pub item_icon_size: usize,
    pub msg: MsgData,
    pub dungeon_name: Vec<u8>,
    pub save_index: u8,
    pub name_buf: [String; NAME_BUFFER_COUNT],
    pub note_buf: [String; NAME_BUFFER_COUNT],
    pub dummy_buf: [String; NAME_BUFFER_COUNT],
    pub car_ofs_x: [f32; CURSOR_SLOT_COUNT],
    pub car_ofs_y: [f32; CURSOR_SLOT_COUNT],
    pub car_floor_ofs_x: [f32; CURSOR_SLOT_COUNT],
    pub car_floor_ofs_y: [f32; CURSOR_SLOT_COUNT],
    pub fb0_contents_color: ContentsColor,
    pub fb0_white: u32,
    pub fb0_black: u32,
    pub fb5_contents_color: ContentsColor,
    pub fb5_white: u32,
    pub fb5_black: u32,
    pub out: MapOutput,
    pub note_timer: i16,
    pub map_draw_offset_y: i16,
    pub cur_floor_map_y: i16,
    pub floor_list_offset_y: i16,
    pub boss_floor_list_offset_y: i16,
    pub mpmk_pos_x: i16,
    pub mpmk_pos_y: i16,
    pub mpmk_size_x: i16,
    pub mpmk_size_y: i16,
    pub mpp1_pos_x: i16,
    pub mpp1_pos_y: i16,
    pub mpp1_size_x: i16,
    pub mpp1_size_y: i16,
    pub mpp_gap_y: i16,
    pub lnk_timer2: i16,
    pub lnk_timer1: i16,
    pub boss_eye_timer: i16,
    pub select_item: u8,
    pub cur_floor: u8,
    pub boss_floor: u8,
    pub top_floor: u8,
    pub bottom_floor: u8,
    pub lnk_tex_flip2: bool,
    pub lnk_tex_flip1: bool,
    pub cur_floor_only: bool,
    pub boss_eye_state: u8,
    pub debug_window: i8,
}

impl Default for DungeonMap {
    fn default() -> Self {
        Self {
            screen: None,
            explain_screen: None,
            panes: MapPanes::default(),
            stick: StickControl::new(STICK_FIRST, STICK_SECOND, STICK_STEP, STICK_MIN_STEP),
            map: MapDriver::default(),
            tuning: MapTuning::default(),
            rng: Rng::default(),
            treasure: (0..TREASURE_COUNT)
                .map(|_| MapMark::new(TREASURE_TEXTURE))
                .collect(),
            doors: (0..DOOR_COUNT)
                .map(|_| MapMark::new(DOOR_TEXTURE))
                .collect(),
            npc: MapMark::new(PRIEST_TEXTURE),
            boss: MapMark::new(BOSS_TEXTURE),
            item_icon: [
                ITEM_ICON_TEXTURE[0],
                ITEM_ICON_TEXTURE[1],
                ITEM_ICON_TEXTURE[2],
            ],
            item_icon_size: ICON_BUFFER_SIZE,
            msg: MsgData::new_data_proc(),
            dungeon_name: Vec::new(),
            save_index: 0,
            name_buf: [String::new(), String::new()],
            note_buf: [String::new(), String::new()],
            dummy_buf: [String::new(), String::new()],
            car_ofs_x: [0.0; CURSOR_SLOT_COUNT],
            car_ofs_y: [0.0; CURSOR_SLOT_COUNT],
            car_floor_ofs_x: [0.0; CURSOR_SLOT_COUNT],
            car_floor_ofs_y: [0.0; CURSOR_SLOT_COUNT],
            fb0_contents_color: ContentsColor::default(),
            fb0_white: 0xFFFF_FFFF,
            fb0_black: 0x0000_0000,
            fb5_contents_color: ContentsColor::default(),
            fb5_white: 0xFFFF_FFFF,
            fb5_black: 0x0000_0000,
            out: MapOutput::default(),
            note_timer: 0,
            map_draw_offset_y: 0,
            cur_floor_map_y: 0,
            floor_list_offset_y: 0,
            boss_floor_list_offset_y: 0,
            mpmk_pos_x: 0,
            mpmk_pos_y: 0,
            mpmk_size_x: 0,
            mpmk_size_y: 0,
            mpp1_pos_x: 0,
            mpp1_pos_y: 0,
            mpp1_size_x: 0,
            mpp1_size_y: 0,
            mpp_gap_y: 0,
            lnk_timer2: 0,
            lnk_timer1: 0,
            boss_eye_timer: 0,
            select_item: FLOOR_SLOT,
            cur_floor: 0,
            boss_floor: BOSS_FLOOR_NONE,
            top_floor: 0,
            bottom_floor: 0,
            lnk_tex_flip2: false,
            lnk_tex_flip1: false,
            cur_floor_only: true,
            boss_eye_state: 0,
            debug_window: 0,
        }
    }
}

impl DungeonMap {
    pub fn sound(&mut self, se: u32) {
        self.out.sounds.push(se);
    }

    pub fn screen_ref(&self) -> Option<&Screen> {
        self.screen.as_ref()
    }

    pub fn explain_screen_ref(&self) -> Option<&Screen> {
        self.explain_screen.as_ref()
    }

    pub fn floor_row_count(&self) -> usize {
        let rows = i32::from(self.top_floor) - i32::from(self.bottom_floor) + 1;
        if rows < 0 { 0 } else { rows as usize }
    }

    pub fn floor_row_y(&self, floor: u8) -> i16 {
        (i32::from(self.mpp1_pos_y)
            + (i32::from(floor) - MAP_FLOOR_BASE)
                * (i32::from(self.mpp1_size_y) + i32::from(self.mpp_gap_y))) as i16
    }

    pub fn create(
        &mut self,
        screen: Screen,
        explain: Screen,
        env: &MapEnv<'_>,
        text: &MapText<'_>,
    ) {
        self.screen = Some(screen);
        self.explain_screen = Some(explain);
        self.map.set_archive();
        for mark in self.treasure.iter_mut() {
            mark.exists = 0;
        }
        for mark in self.doors.iter_mut() {
            mark.exists = 0;
        }
        self.npc.exists = 0;
        self.boss.exists = 0;
        self.note_timer = 0;
        self.select_item = FLOOR_SLOT;
        self.cur_floor_map_y = 0;
        self.cur_floor_only = true;
        self.save_index = env.save_index;
        self.dungeon_name.clear();
        self.msg = MsgData::new_data_proc();
        self.screen_set(env, text);
        self.initialize(env);
        self.treasure_set(env);
    }

    pub fn delete(&mut self) {
        self.screen = None;
        self.explain_screen = None;
        self.panes = MapPanes::default();
        self.stick = StickControl::new(STICK_FIRST, STICK_SECOND, STICK_STEP, STICK_MIN_STEP);
        self.map = MapDriver::default();
        for mark in self.treasure.iter_mut() {
            mark.exists = 0;
        }
        for mark in self.doors.iter_mut() {
            mark.exists = 0;
        }
        self.npc.exists = 0;
        self.boss.exists = 0;
        self.debug_window = 0;
    }

    pub fn set_debug_window(&mut self, id: i8) {
        self.debug_window = id;
    }

    pub fn set_item_icons(&mut self, icons: [&'static str; ITEM_ROW_COUNT]) {
        self.item_icon = icons;
        for i in 0..ITEM_ROW_COUNT {
            self.panes.it[i].change_texture(self.item_icon[i]);
            self.panes.ik[i].change_texture(self.item_icon[i]);
        }
    }

    pub fn screen_set(&mut self, env: &MapEnv<'_>, text: &MapText<'_>) {
        let Some(screen) = self.screen.clone() else {
            return;
        };
        let Some(explain) = self.explain_screen.clone() else {
            return;
        };
        self.panes.bind(&screen, &explain);

        self.panes.nm00.set_font(text.main_font_slot);
        self.panes.nm01.set_font(text.main_font_slot);
        self.panes.str0.set_font(text.ruby_font_slot);
        self.panes.st00.set_font(text.main_font_slot);
        self.panes.dtle.set_font(text.main_font_slot);

        if env.opt_ruby {
            self.panes.st00.pane_trans(0.0, RUBY_OPTION_DY);
        }

        self.panes.str0.set_white(0xFFFF_FFFF);
        self.panes.str0.set_char_color(0x0000_00FF);
        self.panes.str0.set_grad_color(0x0000_00FF);
        self.panes.st00.set_white(0xFFFF_FFFF);
        self.panes.st00.set_char_color(0x0000_00FF);
        self.panes.st00.set_grad_color(0x0000_00FF);

        self.panes.no11.user_area = rotation_to_deg(self.panes.no11.rotation);

        let fb0_color = self.panes.fb[0].contents_color;
        let fb0_white = self.panes.fb[0].white;
        let fb0_black = self.panes.fb[0].black;
        let fb5_color = self.panes.fb[5].contents_color;
        let fb5_white = self.panes.fb[5].white;
        let fb5_black = self.panes.fb[5].black;
        self.fb0_contents_color = fb0_color;
        self.fb0_white = fb0_white;
        self.fb0_black = fb0_black;
        self.fb5_contents_color = fb5_color;
        self.fb5_white = fb5_white;
        self.fb5_black = fb5_black;

        let fl0 = self.panes.fl[0].size_orig;
        let fb0 = self.panes.fb[0].size_orig;
        let fbk0 = self.panes.fbk[0].size_orig;
        for i in 4..FLOOR_ROW_COUNT {
            self.panes.fl[i].resize_orig(fl0.x, fl0.y);
            self.panes.fl[i].cpos_move();
            self.panes.fb[i].resize_orig(fb0.x, fb0.y);
            self.panes.fb[i].set_contents_color(self.fb0_contents_color);
            self.panes.fb[i].set_white(self.fb0_white);
            self.panes.fb[i].set_black(self.fb0_black);
            self.panes.fb[i].cpos_move();
            self.panes.fbk[i].resize_orig(fbk0.x, fbk0.y);
            self.panes.fbk[i].cpos_move();
        }

        self.panes.marker_mask.hide();
        self.panes.map_pos_next.hide();
        self.panes.player_pos.hide();
        self.panes.map_pos_prev.hide();

        self.panes.msk.init_alpha = MASK_ALPHA;
        self.panes.msk0.init_alpha = MASK_ALPHA;

        self.mpmk_pos_x = self.panes.marker_mask.pos_center_orig.x as i16;
        self.mpmk_pos_y = self.panes.marker_mask.pos_center_orig.y as i16;
        self.mpmk_size_x = self.panes.marker_mask.size_orig.x as i16;
        self.mpmk_size_y = self.panes.marker_mask.size_orig.y as i16;
        self.mpp1_pos_x = self.panes.player_pos.pos_center_orig.x as i16;
        self.mpp1_pos_y = self.panes.player_pos.pos_center_orig.y as i16;
        self.mpp1_size_x = self.panes.player_pos.size_orig.x as i16;
        self.mpp1_size_y = self.panes.player_pos.size_orig.y as i16;

        self.mpp_gap_y = (self.panes.player_pos.pos_top_left_orig.y
            - (self.panes.map_pos_prev.pos_top_left_orig.y + self.panes.map_pos_prev.size_orig.y))
            as i16;

        self.dungeon_name_set(env, text);
    }

    pub fn dungeon_name_set(&mut self, env: &MapEnv<'_>, text: &MapText<'_>) {
        let msg_no = dungeon_name_message(env.save_index);
        let mut get = ItemMsgGet::default();
        let head = get
            .get_mesg_header(text.archive, text.clear_count, msg_no)
            .map(|head| head.to_vec());
        let (name, str_width) = match head {
            Some(head) => match get.get_message(&head).map(|bytes| bytes.to_vec()) {
                Some(message) => {
                    let size_y = self.panes.dtle.get_font_size().1;
                    message_name_walk(
                        &message,
                        text.font,
                        text.main_font_slot,
                        size_y,
                        DUNGEON_NAME_SIZE,
                    )
                }
                None => (Vec::new(), 0.0),
            },
            None => (Vec::new(), 0.0),
        };

        let mut font_size = F2 {
            x: self.panes.dtle.get_font_size().1,
            y: self.panes.dtle.get_font_size().1,
        };
        if self.panes.nm00.width() < str_width {
            font_size.x = (font_size.x * self.panes.nm00.width() / str_width) as f32;
        }
        self.dungeon_name = name;
        let out = String::from_utf8_lossy(&self.dungeon_name).into_owned();
        self.panes.dtle.set_font_size(font_size.x, font_size.y);
        self.panes.dtle.set_char_space(0.0);
        self.panes.dtle.set_string(&out);
    }

    pub fn initialize(&mut self, env: &MapEnv<'_>) {
        let y_pos = env.player_pos[1];
        self.panes.nm00.user_area = 0;
        self.cur_floor = env.floors.floor_no(y_pos + env.stage_map_offset_y());
        self.boss_floor = BOSS_FLOOR_NONE;

        for door in env.doors {
            if is_boss_door(&stage_name(&door.name), door.base.angle) {
                self.boss_floor = env.floors.floor_no(door.base.position[1]);
            }
        }

        env.floors
            .top_bottom_floor(&mut self.top_floor, &mut self.bottom_floor);
        let stay_no = env.stay_room;
        let floor_no = env
            .floors
            .floor_no_for_room(stay_no, y_pos + env.stage_map_offset_y());
        self.map.init(
            [
                self.mpmk_pos_x,
                self.mpmk_pos_y,
                self.mpmk_size_x,
                self.mpmk_size_y,
            ],
            [
                self.mpp1_pos_x,
                self.mpp1_pos_y,
                self.mpp1_size_x,
                self.mpp1_size_y,
            ],
            self.mpp_gap_y,
            [
                self.top_floor,
                self.bottom_floor,
                stay_no as u8,
                floor_no,
                INIT_ALPHA_SLOT,
            ],
        );
        self.cur_floor_map_y = self.floor_row_y(self.cur_floor);
        self.map_draw_offset_y = MAP_DRAW_OFFSET_CLOSED;
        let row_step = self.panes.fb[0].pos_center_orig.y - self.panes.fb[1].pos_center_orig.y;
        let cur_steps = (i32::from(self.bottom_floor) - i32::from(self.cur_floor)) as i16;
        self.floor_list_offset_y = (row_step * f32::from(cur_steps)) as i16;
        if self.boss_floor != BOSS_FLOOR_NONE {
            let boss_steps = (i32::from(self.bottom_floor) - i32::from(self.boss_floor)) as i16;
            self.boss_floor_list_offset_y = (row_step * f32::from(boss_steps)) as i16;
        } else {
            self.boss_floor_list_offset_y = 0;
        }

        let mask_x = self.mpmk_pos_x;
        let mask_y = self.mpmk_pos_y + self.map_draw_offset_y;
        self.map.set_mask_pos(mask_x, mask_y);
        let map_x = self.mpp1_pos_x;
        let map_y = self.cur_floor_map_y + self.map_draw_offset_y;
        self.map.set_map_pos(map_x, map_y);

        let rows = self.floor_row_count();
        for i in 0..rows {
            let floor = i32::from(self.bottom_floor) - FLOOR_TEXTURE_BASE + i as i32;
            self.change_floor_texture(i, floor);
        }

        self.lnk_timer2 = (self.rng.rnd_f(100.0) + 100.0) as i16;
        self.lnk_timer1 = (self.rng.rnd_f(50.0) + 100.0) as i16;
        self.lnk_tex_flip2 = false;
        self.lnk_tex_flip1 = false;
        self.panes.lnk[1].user_area = 0;
        self.panes.lnk[2].user_area = 0;
        self.panes.boss.user_area = 100;
        self.panes.bos2.user_area = (self.rng.rnd_f(200.0) + 300.0) as i16;
        self.boss_eye_state = 0;
        self.boss_eye_timer = ((self.rng.rnd_f(18.0) + 40.0) as i32) << 1;

        let it_w = self.panes.it[0].size_orig.x;
        let it_h = self.panes.it[0].size_orig.y;
        self.car_ofs_x[0] = -it_w / 2.0;
        self.car_ofs_y[0] = it_h / 2.0;
        self.car_ofs_x[1] = it_w / 2.0;
        self.car_ofs_y[1] = it_h / 2.0;
        self.car_ofs_x[2] = -it_w / 2.0;
        self.car_ofs_y[2] = -it_h / 2.0;
        self.car_ofs_x[3] = it_w / 2.0;
        self.car_ofs_y[3] = -it_h / 2.0;

        let fb_w = self.panes.fb[0].size_orig.x;
        let fb_h = self.panes.fb[0].size_orig.y;
        self.car_floor_ofs_x[0] = -fb_w / 2.0;
        self.car_floor_ofs_y[0] = fb_h / 2.0;
        self.car_floor_ofs_x[1] = fb_w / 2.0;
        self.car_floor_ofs_y[1] = fb_h / 2.0;
        self.car_floor_ofs_x[2] = -fb_w / 2.0;
        self.car_floor_ofs_y[2] = -fb_h / 2.0;
        self.car_floor_ofs_x[3] = fb_w / 2.0;
        self.car_floor_ofs_y[3] = -fb_h / 2.0;

        self.floor_init();
        self.item_scale(env);

        if !env.has_map() {
            self.panes.it[0].hide();
            self.panes.ik[0].hide();
            self.panes.ip[0].hide();
        }
        if !env.has_boss_key() {
            self.panes.it[1].hide();
            self.panes.ik[1].hide();
            self.panes.ip[1].hide();
        }
        if !env.has_compass() {
            self.panes.it[2].hide();
            self.panes.ik[2].hide();
            self.panes.ip[2].hide();
        }
        if !env.has_compass() || self.boss_floor == BOSS_FLOOR_NONE {
            self.panes.boss.hide();
        }

        self.note_init();

        if (self.select_item == 0 && env.has_map())
            || (self.select_item == 1 && env.has_boss_key())
            || (self.select_item == 2 && env.has_compass())
        {
            self.out.do_status_force = Some(DO_STATUS_INFO);
        } else {
            self.out.do_status_force = Some(DO_STATUS_BLANK);
            self.out.do_status = Some(DO_STATUS_BLANK);
        }
    }

    pub fn change_floor_texture(&mut self, pane: usize, floor: i32) {
        let mut floor = floor;
        if floor < 0 || floor >= FLOOR_TEXTURE_COUNT {
            floor = 0;
        }
        let Some(texture) = FLOOR_TEXTURE.get(floor as usize).copied() else {
            return;
        };
        if let Some(pane) = self.panes.fl.get_mut(pane) {
            pane.change_texture(texture);
        }
    }

    pub fn treasure_set(&mut self, env: &MapEnv<'_>) {
        for mark in self.treasure.iter_mut() {
            mark.exists = 0;
        }
        for mark in self.doors.iter_mut() {
            mark.exists = 0;
        }
        self.boss.exists = 0;
        self.npc.exists = 0;

        let Some(_) = env.stage_map else {
            return;
        };
        let (origin_x, origin_z, scale) = env.stage_map_basis();
        let save_index = env.save_index;
        let row_gap = f32::from(self.mpp1_size_y + self.mpp_gap_y);

        if env.has_compass() {
            if (save_index == SAVE_ET
                && !env.is_collect(COLLECT_SLOT, COLLECT_PRIEST_SAVE)
                && env.priest_flag == PRIEST_FLAG_SAVE)
                || (save_index == SAVE_WT
                    && !env.is_collect(COLLECT_SLOT, COLLECT_PRIEST_TUNIC)
                    && env.priest_flag == PRIEST_FLAG_TUNIC)
            {
                let npc_pos = env.cb1_player_pos.unwrap_or(env.priest_pos);
                let npc_floor = env.floors.floor_no(npc_pos[1]);
                if self.cur_floor == npc_floor {
                    self.place_npc(origin_x, origin_z, scale, npc_pos);
                } else if !self.cur_floor_only && self.cur_floor.wrapping_sub(1) == npc_floor {
                    self.place_npc(origin_x, origin_z, scale, npc_pos);
                    self.npc.pos_y += row_gap;
                } else if !self.cur_floor_only && self.cur_floor.wrapping_add(1) == npc_floor {
                    self.place_npc(origin_x, origin_z, scale, npc_pos);
                    self.npc.pos_y -= row_gap;
                }
            }

            for tentry in env.treasures {
                let tbox_no = ((tentry.base.parameters >> TBOX_SHIFT) as i32) & TBOX_MASK;
                let floor = env.floors.floor_no(tentry.base.position[1]);
                let opened = env.is_tbox(tbox_no);
                let skipped = tbox_no == TBOX_SKIP_A || tbox_no == TBOX_SKIP_B;
                if self.cur_floor == floor {
                    if !opened && !skipped {
                        self.place_treasure(origin_x, origin_z, scale, tentry);
                        continue;
                    }
                }
                if !self.cur_floor_only && self.cur_floor.wrapping_sub(1) == floor {
                    if !opened && !skipped {
                        self.place_treasure(origin_x, origin_z, scale, tentry);
                        self.nudge_last_treasure(row_gap);
                        continue;
                    }
                }
                if !self.cur_floor_only && self.cur_floor.wrapping_add(1) == floor {
                    if !opened && !skipped {
                        self.place_treasure(origin_x, origin_z, scale, tentry);
                        self.nudge_last_treasure(-row_gap);
                    }
                }
            }
        }

        for dentry in env.doors {
            let floor = env.floors.floor_no(dentry.base.position[1]);
            let room_a = i32::from(dentry.base.angle[0]) & DOOR_ROOM_MASK;
            let room_b =
                (u32::from(dentry.base.angle[0]) >> DOOR_ROOM_SHIFT) as i32 & DOOR_ROOM_MASK;
            if self.cur_floor == floor {
                if env.has_map() || env.is_visited_room(room_a) || env.is_visited_room(room_b) {
                    self.place_door(origin_x, origin_z, scale, dentry);
                }
            } else if !self.cur_floor_only && self.cur_floor.wrapping_sub(1) == floor {
                if env.has_map() {
                    self.place_door(origin_x, origin_z, scale, dentry);
                    self.nudge_last_door(row_gap);
                }
            } else if !self.cur_floor_only && self.cur_floor.wrapping_add(1) == floor {
                if env.has_map() {
                    self.place_door(origin_x, origin_z, scale, dentry);
                    self.nudge_last_door(-row_gap);
                }
            }

            if env.has_compass() && is_boss_door(&stage_name(&dentry.name), dentry.base.angle) {
                let ofs_x = BOSS_OFFSET_X
                    .get(usize::from(save_index))
                    .copied()
                    .unwrap_or(0.0)
                    + f32::from(self.tuning.boss_ofs_x);
                let ofs_y = BOSS_OFFSET_Y
                    .get(usize::from(save_index))
                    .copied()
                    .unwrap_or(0.0)
                    + f32::from(self.tuning.boss_ofs_y);
                if self.cur_floor == floor {
                    self.place_boss(origin_x, origin_z, scale, dentry, ofs_x, ofs_y);
                } else if !self.cur_floor_only && self.cur_floor.wrapping_sub(1) == floor {
                    self.place_boss(origin_x, origin_z, scale, dentry, ofs_x, ofs_y);
                    self.boss.pos_y += row_gap;
                } else if !self.cur_floor_only && self.cur_floor.wrapping_add(1) == floor {
                    self.place_boss(origin_x, origin_z, scale, dentry, ofs_x, ofs_y);
                    self.boss.pos_y -= row_gap;
                }
            }
        }
    }

    fn place_npc(&mut self, origin_x: f32, origin_z: f32, scale: f32, pos: [f32; 3]) {
        self.npc.exists = 1;
        self.npc.set_white(self.tuning.npc_white);
        self.npc.set_black(self.tuning.npc_black);
        self.npc.width = f32::from(self.tuning.npc_width);
        self.npc.height = f32::from(self.tuning.npc_height);
        self.npc.pos_x =
            f32::from(self.mpp1_pos_x) + (pos[0] - origin_x) / scale - self.npc.width / 2.0;
        self.npc.pos_y =
            f32::from(self.mpp1_pos_y) + (pos[2] - origin_z) / scale - self.npc.height / 2.0;
    }

    fn place_treasure(&mut self, origin_x: f32, origin_z: f32, scale: f32, entry: &TreasureEntry) {
        let white = self.tuning.treasure_white;
        let black = self.tuning.treasure_black;
        let width = self.tuning.treasure_width;
        let height = self.tuning.treasure_height;
        let base_x = self.mpp1_pos_x;
        let base_y = self.mpp1_pos_y;
        let Some(mark) = self.treasure.iter_mut().find(|mark| mark.exists == 0) else {
            return;
        };
        mark.exists = 1;
        mark.set_white(white);
        mark.set_black(black);
        mark.width = f32::from(width);
        mark.height = f32::from(height);
        mark.pos_x =
            f32::from(base_x) + (entry.base.position[0] - origin_x) / scale - mark.width / 2.0;
        mark.pos_y =
            f32::from(base_y) + (entry.base.position[2] - origin_z) / scale - mark.height / 2.0;
    }

    fn nudge_last_treasure(&mut self, delta: f32) {
        if let Some(mark) = self.treasure.iter_mut().rev().find(|mark| mark.exists == 1) {
            mark.pos_y += delta;
        }
    }

    fn place_door(&mut self, origin_x: f32, origin_z: f32, scale: f32, entry: &DoorEntry) {
        let white = self.tuning.door_white;
        let black = self.tuning.door_black;
        let width = self.tuning.door_width;
        let height = self.tuning.door_height;
        let angle = entry.base.angle[1];
        let position = entry.base.position;
        let base_x = self.mpp1_pos_x;
        let base_y = self.mpp1_pos_y;
        let Some(mark) = self.doors.iter_mut().find(|mark| mark.exists == 0) else {
            return;
        };
        mark.exists = 1;
        mark.set_white(white);
        mark.set_black(black);
        mark.width = f32::from(width);
        mark.height = f32::from(height);
        mark.angle = angle;
        mark.pos_x = f32::from(base_x) + (position[0] - origin_x) / scale - mark.width / 2.0;
        mark.pos_y = f32::from(base_y) + (position[2] - origin_z) / scale - mark.height / 2.0;
    }

    fn nudge_last_door(&mut self, delta: f32) {
        if let Some(mark) = self.doors.iter_mut().rev().find(|mark| mark.exists == 1) {
            mark.pos_y += delta;
        }
    }

    fn place_boss(
        &mut self,
        origin_x: f32,
        origin_z: f32,
        scale: f32,
        entry: &DoorEntry,
        ofs_x: f32,
        ofs_y: f32,
    ) {
        let position = entry.base.position;
        self.boss.exists = 1;
        self.boss.set_white(self.tuning.boss_white);
        self.boss.set_black(self.tuning.boss_black);
        self.boss.width = f32::from(self.tuning.boss_width);
        self.boss.height = f32::from(self.tuning.boss_height);
        self.boss.pos_x = f32::from(self.mpp1_pos_x) + (position[0] - origin_x) / scale
            - self.boss.width / 2.0
            + ofs_x;
        self.boss.pos_y = f32::from(self.mpp1_pos_y) + (position[2] - origin_z) / scale
            - self.boss.height / 2.0
            + ofs_y;
    }

    pub fn treasure_draw(&mut self) -> Vec<DrawItem> {
        let offset_y =
            f32::from(self.cur_floor_map_y) - f32::from(self.floor_row_y(self.cur_floor));
        let draw_offset_y = offset_y + f32::from(self.map_draw_offset_y);
        let top = f32::from(self.mpp1_pos_y) - f32::from(self.mpp1_size_y) / 2.0;
        let bottom = f32::from(self.mpp1_pos_y) + f32::from(self.mpp1_size_y) / 2.0;
        let alpha = self.panes.cc[CC_ALPHA].now_alpha;
        let mut out: Vec<DrawItem> = Vec::new();

        for mark in self.treasure.iter_mut() {
            if mark.exists != 0 {
                let y = mark.pos_y;
                if y + offset_y > top && y + offset_y < bottom {
                    mark.set_alpha(alpha);
                    out.push(mark.draw_item(mark.pos_x, y + draw_offset_y));
                }
            }
        }

        for mark in self.doors.iter_mut() {
            if mark.exists != 0 {
                let y = mark.pos_y;
                if y + offset_y > top && y + offset_y < bottom {
                    let pivot_x = mark.width / 2.0;
                    let pivot_y = mark.height / 2.0;
                    let angle = s2deg(mark.angle);
                    mark.rotate(pivot_x, pivot_y, angle);
                    mark.set_alpha(alpha);
                    out.push(mark.draw_item(mark.pos_x, y + draw_offset_y));
                }
            }
        }

        if self.npc.exists != 0 {
            let y = self.npc.pos_y;
            if y + offset_y > top && y + offset_y < bottom {
                self.npc.set_alpha(alpha);
                out.push(self.npc.draw_item(self.npc.pos_x, y + draw_offset_y));
            }
        }

        if self.boss.exists != 0 {
            let y = self.boss.pos_y;
            if y + offset_y > top && y + offset_y < bottom {
                self.boss.set_alpha(alpha);
                out.push(self.boss.draw_item(self.boss.pos_x, y + draw_offset_y));
            }
        }

        out
    }

    pub fn pane_move(&mut self, y: f32) {
        self.panes.dt00.pane_trans(0.0, y);
        self.panes.dk00.pane_trans(0.0, y);
        self.panes.nm00.pane_trans(0.0, y);
        self.panes.nm01.pane_trans(0.0, y);
        self.panes.itnk.pane_trans(0.0, y);

        for i in 0..CURSOR_SLOT_COUNT {
            self.panes.car[i].pane_trans(0.0, y);
        }

        for i in 0..ITEM_ROW_COUNT {
            self.panes.it[i].pane_trans(0.0, y);
            self.panes.ik[i].pane_trans(0.0, y);
            self.panes.ip[i].pane_trans(0.0, y);
        }

        self.panes.ip01.pane_trans(0.0, y);
        self.panes.lnk[0].pane_trans(0.0, f32::from(self.floor_list_offset_y) + y);
        self.panes
            .boss
            .pane_trans(0.0, f32::from(self.boss_floor_list_offset_y) + y);

        for i in 0..FLOOR_ROW_COUNT {
            self.panes.fl[i].pane_trans(0.0, y);
            self.panes.fb[i].pane_trans(0.0, y);
            self.panes.fbk[i].pane_trans(0.0, y);
        }

        self.panes.fw00.pane_trans(0.0, y);
        self.panes.fw01.pane_trans(0.0, y);
        for i in 0..CC_COUNT {
            self.panes.cc[i].pane_trans(0.0, y);
        }
    }

    pub fn pane_alpha(&mut self, alpha: f32) {
        self.panes.dtle.set_now_alpha(alpha);
        self.panes.dt00.set_now_alpha(alpha);
        self.panes.dk00.set_now_alpha(alpha);
        self.panes.itnm.set_now_alpha(alpha);
        self.panes.itnk.set_now_alpha(alpha);

        for i in 0..CURSOR_SLOT_COUNT {
            self.panes.car[i].set_now_alpha(alpha);
        }

        for i in 0..ITEM_ROW_COUNT {
            self.panes.it[i].set_now_alpha(alpha);
            self.panes.ik[i].set_now_alpha(alpha);
            self.panes.ip[i].set_now_alpha(alpha);
        }

        self.panes.ip01.set_now_alpha(alpha);
        self.panes.lnk[0].set_now_alpha(alpha);
        self.panes.boss.set_now_alpha(alpha);

        for i in 0..self.floor_row_count() {
            if let Some(pane) = self.panes.fl.get_mut(i) {
                pane.set_now_alpha(alpha);
            }
            if let Some(pane) = self.panes.fb.get_mut(i) {
                pane.set_now_alpha(alpha);
            }
            if let Some(pane) = self.panes.fbk.get_mut(i) {
                pane.set_now_alpha(alpha);
            }
        }

        self.panes.fw00.set_now_alpha(alpha);
        self.panes.fw01.set_now_alpha(alpha);
        for i in 0..CC_COUNT {
            self.panes.cc[i].set_now_alpha(alpha);
        }
        self.panes.msk.set_now_alpha(alpha);
    }

    pub fn dec_alpha(&mut self, alpha: f32) {
        for i in 0..OUT_FONT_COUNT {
            self.panes.ft[i].dec_alpha(alpha);
        }

        for i in 0..CURSOR_SLOT_COUNT {
            self.panes.car[i].dec_alpha(alpha);
        }

        self.panes.nm00.dec_alpha(alpha);
        self.panes.nm01.dec_alpha(alpha);
        self.panes.itnm.dec_alpha(alpha);
        self.panes.itnk.dec_alpha(alpha);
        self.panes.msk0.dec_alpha(alpha);
        self.panes.str0.dec_alpha(alpha);
        self.panes.st00.dec_alpha(alpha);
        self.panes.nt00.dec_alpha(alpha);
        self.panes.nk00.dec_alpha(alpha);
        self.panes.no11.dec_alpha(alpha);
        self.panes.dtle.dec_alpha(alpha);
        self.panes.dt00.dec_alpha(alpha);
        self.panes.dk00.dec_alpha(alpha);

        for i in 0..ITEM_ROW_COUNT {
            self.panes.it[i].dec_alpha(alpha);
            self.panes.ik[i].dec_alpha(alpha);
            self.panes.ip[i].dec_alpha(alpha);
        }

        self.panes.ip01.dec_alpha(alpha);
        self.panes.lnk[0].dec_alpha(alpha);
        self.panes.bey1.dec_alpha(alpha);
        self.panes.bey2.dec_alpha(alpha);
        self.panes.boss.dec_alpha(alpha);

        for i in 0..FLOOR_ROW_COUNT {
            self.panes.fl[i].dec_alpha(alpha);
            self.panes.fb[i].dec_alpha(alpha);
            self.panes.fbk[i].dec_alpha(alpha);
        }

        self.panes.fw00.dec_alpha(alpha);
        self.panes.fw01.dec_alpha(alpha);
        for i in 0..CC_COUNT {
            self.panes.cc[i].dec_alpha(alpha);
        }
        self.panes.msk.dec_alpha(alpha);
    }

    pub fn cursor_move(&mut self, env: &MapEnv<'_>, text: &MapText<'_>, pad: &mut PadInput) {
        let select_item = self.select_item;
        let cur_floor = self.cur_floor;
        self.stick.check_trigger(pad.stick_value, pad.stick_angle);

        if self.stick.check_left_trigger() {
            if self.select_item < 3 && self.select_item != 0 {
                self.select_item = self.select_item.wrapping_sub(1);
                self.sound(SE_MENU_CURSOR);
            }
        } else if self.stick.check_right_trigger() {
            if self.select_item < 3 && self.select_item < 2 {
                self.select_item = self.select_item.wrapping_add(1);
                self.sound(SE_MENU_CURSOR);
            }
        } else if self.stick.check_up_trigger() {
            if self.select_item < 3 {
                self.select_item = 3;
                self.panes.cc[CC_SCROLL].user_area = 6;
                self.sound(SE_MENU_CURSOR);
            } else if self.cur_floor < self.top_floor {
                self.cur_floor = self.cur_floor.wrapping_add(1);
                self.panes.cc[CC_SCROLL].user_area = 6;
                self.sound(SE_MENU_CURSOR);
            }
        } else if self.stick.check_down_trigger() {
            if self.select_item == 3 {
                if self.cur_floor > self.bottom_floor {
                    self.cur_floor = self.cur_floor.wrapping_sub(1);
                    self.panes.cc[CC_SCROLL].user_area = -6;
                    self.sound(SE_MENU_CURSOR);
                } else {
                    self.select_item = 1;
                    self.panes.cc[CC_SCROLL].user_area = -6;
                    self.sound(SE_MENU_CURSOR);
                }
            }
        }

        if self.select_item != select_item || self.cur_floor != cur_floor {
            self.panes.nm00.user_area = 0;
            self.item_name_set(env, text);
            if self.select_item < 3 {
                for i in 0..self.floor_row_count() {
                    if let Some(pane) = self.panes.fb.get_mut(i) {
                        pane.set_init_alpha();
                    }
                }
            }
            self.item_scale(env);
            if self.cur_floor != cur_floor {
                self.treasure_set(env);
            }
            self.sound(SE_MENU_CURSOR);
        }

        if self.select_item < 3 {
            if (self.select_item == 0 && env.has_map())
                || (self.select_item == 1 && env.has_boss_key())
                || (self.select_item == 2 && env.has_compass())
            {
                self.out.do_status_force = Some(DO_STATUS_INFO);
            } else {
                self.out.do_status_force = Some(DO_STATUS_BLANK);
                self.out.do_status = Some(DO_STATUS_BLANK);
            }
        }
    }

    pub fn cursor_anime(&mut self) {
        let mut x_trans = 0i16;
        let mut y_trans = 0i16;

        if self.panes.car[0].user_area < CURSOR_STEP_FIRST {
            for i in 0..CURSOR_SLOT_COUNT {
                self.panes.car[i].change_texture(CURSOR_TEXTURE_IN);
            }
            x_trans = CURSOR_STEP_SIZE;
            y_trans = CURSOR_STEP_SIZE;
            if self.select_item == 3 {
                self.panes.car[0].user_area =
                    value_increase(10, i32::from(self.panes.car[0].user_area), 0) as i16;
            }
        } else if self.panes.car[0].user_area < CURSOR_STEP_MAX {
            for i in 0..CURSOR_SLOT_COUNT {
                self.panes.car[i].change_texture(CURSOR_TEXTURE_OUT);
            }
            x_trans = CURSOR_STEP_SIZE * 2;
            y_trans = CURSOR_STEP_SIZE * 2;
            if self.select_item == 3 {
                self.panes.car[0].user_area =
                    value_increase(10, 20 - i32::from(self.panes.car[0].user_area), 0) as i16;
            }
        }
        self.panes.car[0].pane_trans(-f32::from(x_trans), f32::from(y_trans));
        self.panes.car[1].pane_trans(f32::from(x_trans), f32::from(y_trans));
        self.panes.car[2].pane_trans(-f32::from(x_trans), -f32::from(y_trans));
        self.panes.car[3].pane_trans(f32::from(x_trans), -f32::from(y_trans));

        self.panes.car[0].user_area += 1;
        if self.panes.car[0].user_area >= CURSOR_STEP_MAX {
            self.panes.car[0].user_area = 0;
        }
    }

    pub fn note_init(&mut self) {
        self.panes.nk00.user_area = 0;
        self.panes.nt00.user_area = 0;

        self.panes.no11.rotate(
            self.panes.no11.size_orig.x / 2.0,
            self.panes.no11.size_orig.y / 2.0,
            f32::from(self.panes.no11.user_area),
        );

        self.panes.no11.pane_trans(0.0, 0.0);
    }

    pub fn note_check(&self) -> bool {
        self.panes.nk00.user_area == 1
    }

    pub fn note_appear(&mut self, pad: &mut PadInput) {
        if self.panes.nk00.user_area == 1 {
            let r0 = NOTE_OPEN_STEPS;
            if self.panes.nt00.user_area <= r0 {
                self.note_open();
                return;
            }
            let r0 = NOTE_OPEN_HOLD;
            if self.panes.nt00.user_area > r0 {
                self.note_close();
                return;
            }
            if pad.trig_a || pad.trig_b {
                self.panes.nt00.user_area += 1;
                self.sound(SE_MENU_EXP_OUT);
            }
        }
    }

    pub fn note_open(&mut self) {
        let f1 = SCREEN_X;
        let f2 = SCREEN_Y;
        let x = f1 - self.panes.no11.pos_center_orig.x;
        let y = f2 - self.panes.no11.pos_center_orig.y;
        let mut alpha = 1.0
            - value_increase(
                i32::from(NOTE_OPEN_STEPS),
                i32::from(NOTE_OPEN_STEPS) - i32::from(self.panes.nt00.user_area),
                0,
            );

        let r0 = NOTE_OPEN_STEPS;
        if self.panes.nt00.user_area >= r0 {
            self.panes.msk0.set_init_alpha();
            self.panes.str0.set_init_alpha();
            self.panes.st00.set_init_alpha();
            self.sound(SE_MENU_EXP_IN);
        } else {
            self.panes.msk0.set_now_alpha(alpha);
            self.panes.str0.set_now_alpha(alpha);
            self.panes.st00.set_now_alpha(alpha);
        }

        self.panes.no11.rotate(
            self.panes.no11.size.x / 2.0,
            self.panes.no11.size.y / 2.0,
            f32::from(self.panes.no11.user_area)
                + alpha * (NOTE_EDGE_ANGLE - f32::from(self.panes.no11.user_area)),
        );

        if self.panes.nt00.user_area < NOTE_OPEN_RAMP {
            alpha =
                (f32::from(self.panes.nt00.user_area) * NOTE_OPEN_BASE) / f32::from(NOTE_OPEN_RAMP);
        } else {
            let f29 = NOTE_OPEN_BIAS;
            alpha =
                (1.0 - value_increase(
                    NOTE_OPEN_TOTAL,
                    i32::from(NOTE_OPEN_STEPS) - i32::from(self.panes.nt00.user_area),
                    0,
                )) * f29
                    + NOTE_OPEN_BASE;
        }

        self.panes.no11.pane_trans(x * alpha, y * alpha);
        self.panes.nt00.user_area += 1;
    }

    pub fn note_close(&mut self) {
        let f3 = SCREEN_X;
        let f4 = SCREEN_Y;
        let f5 = (f3 * 2.0) - self.panes.no11.pos_center_orig.x;
        let f0 = (f4 * 2.0) - self.panes.no11.pos_center_orig.y;
        let out_x = f5 - f3;
        let out_y = f0 - f4;
        let x = f3 - self.panes.no11.pos_center_orig.x;
        let y = f4 - self.panes.no11.pos_center_orig.y;

        let alpha = value_increase(
            NOTE_CLOSE_LEN,
            i32::from(self.panes.nt00.user_area) - i32::from(NOTE_CLOSE_STEPS),
            0,
        );
        let f26 = (alpha * NOTE_CLOSE_ANGLE) + NOTE_EDGE_ANGLE;

        self.panes
            .no11
            .pane_trans((out_x * alpha) + x, (out_y * alpha) + y);

        self.panes.no11.rotate(
            self.panes.no11.size.x / 2.0,
            self.panes.no11.size.y / 2.0,
            f26,
        );

        self.panes.nt00.user_area += 1;

        let r0 = NOTE_CLOSE_LIMIT;
        if self.panes.nt00.user_area > r0 {
            self.panes.nt00.set_init_alpha();
            self.panes.nk00.set_init_alpha();
            self.panes.no11.set_init_alpha();
            self.panes.msk0.set_now_alpha_zero();
            self.panes.str0.set_now_alpha_zero();
            self.panes.st00.set_now_alpha_zero();

            self.panes.nk00.user_area = 0;
            self.panes.nt00.user_area = 0;

            self.panes.no11.rotate(
                self.panes.no11.size.x / 2.0,
                self.panes.no11.size.y / 2.0,
                f32::from(self.panes.no11.user_area),
            );

            self.panes.no11.pane_trans(0.0, 0.0);
        } else {
            self.panes.nt00.set_now_alpha(1.0 - alpha);
            self.panes.nk00.set_now_alpha(1.0 - alpha);
            self.panes.no11.set_now_alpha(1.0 - alpha);
            self.panes.msk0.set_now_alpha(1.0 - alpha);
            self.panes.str0.set_now_alpha(1.0 - alpha);
            self.panes.st00.set_now_alpha(1.0 - alpha);
        }
    }

    pub fn note_open_proc(&mut self, step: i16) {
        if step <= OPEN_STEPS {
            let alpha = value_increase(OPEN_STEP_LEN, i32::from(step), 0);
            let angle = value_increase(OPEN_STEP_LEN, OPEN_STEP_LEN - i32::from(step), 0)
                * (NOTE_OPEN_ANGLE - f32::from(self.panes.no11.user_area))
                + f32::from(self.panes.no11.user_area);
            self.panes.no11.rotate(
                self.panes.no11.size.x / 2.0,
                self.panes.no11.size.y / 2.0,
                angle,
            );
            self.panes.nt00.set_now_alpha(alpha);
            self.panes.nk00.set_now_alpha(alpha);
        }
    }

    pub fn item_scale(&mut self, env: &MapEnv<'_>) {
        if self.select_item < 3 {
            for i in 0..ITEM_ROW_COUNT {
                if i == usize::from(self.select_item) {
                    self.panes.it[i].pane_scale_xy(ITEM_SCALE);
                    self.panes.ik[i].pane_scale_xy(ITEM_SCALE);
                } else {
                    self.panes.it[i].pane_scale_xy(1.0);
                    self.panes.ik[i].pane_scale_xy(1.0);
                }
            }

            let selected = usize::from(self.select_item);
            for i in 0..CURSOR_SLOT_COUNT {
                self.panes.car[i].pos_center_orig.x =
                    self.panes.it[selected].pos_center_orig.x + self.car_ofs_x[i];
                self.panes.car[i].pos_center_orig.y =
                    self.panes.it[selected].pos_center_orig.y + self.car_ofs_y[i];
            }

            let fb0_color = self.fb0_contents_color;
            let fb0_white = self.fb0_white;
            let fb0_black = self.fb0_black;
            for i in 0..self.floor_row_count() {
                if let Some(pane) = self.panes.fl.get_mut(i) {
                    pane.size.x = pane.size_orig.x;
                    pane.size.y = pane.size_orig.y;
                    pane.cpos_move();
                }
                if let Some(pane) = self.panes.fb.get_mut(i) {
                    pane.size.x = pane.size_orig.x;
                    pane.size.y = pane.size_orig.y;
                    pane.set_contents_color(fb0_color);
                    pane.set_white(fb0_white);
                    pane.set_black(fb0_black);
                    pane.cpos_move();
                }
                if let Some(pane) = self.panes.fbk.get_mut(i) {
                    pane.size.x = pane.size_orig.x;
                    pane.size.y = pane.size_orig.y;
                    pane.cpos_move();
                }
            }

            let y_pos = env.player_pos[1];
            self.cur_floor = env.floors.floor_no(y_pos + env.stage_map_offset_y());
            self.cur_floor_map_y = self.floor_row_y(self.cur_floor);
        } else {
            for i in 0..ITEM_ROW_COUNT {
                self.panes.it[i].pane_scale_xy(1.0);
                self.panes.ik[i].pane_scale_xy(1.0);
            }

            let row = (i32::from(self.cur_floor) - i32::from(self.bottom_floor)).max(0) as usize;
            let (center_x, center_y) = self
                .panes
                .fb
                .get(row)
                .map(|pane| (pane.pos_center.x, pane.pos_center.y))
                .unwrap_or((0.0, 0.0));
            for i in 0..CURSOR_SLOT_COUNT {
                self.panes.car[i].pos_center_orig.x = center_x + self.car_floor_ofs_x[i];
                self.panes.car[i].pos_center_orig.y = center_y + self.car_floor_ofs_y[i];
            }

            let fb0_color = self.fb0_contents_color;
            let fb0_white = self.fb0_white;
            let fb0_black = self.fb0_black;
            let fb5_color = self.fb5_contents_color;
            let fb5_white = self.fb5_white;
            let fb5_black = self.fb5_black;
            for i in 0..self.floor_row_count() {
                let floor_no = self.bottom_floor + i as u8;
                if floor_no == self.cur_floor {
                    if let Some(pane) = self.panes.fl.get_mut(i) {
                        pane.size.x = pane.size_orig.x * FLOOR_SCALE;
                        pane.size.y = pane.size_orig.y * FLOOR_SCALE;
                        pane.cpos_move();
                    }
                    if let Some(pane) = self.panes.fb.get_mut(i) {
                        pane.size.x = pane.size_orig.x * FLOOR_SCALE;
                        pane.size.y = pane.size_orig.y * FLOOR_SCALE;
                        pane.set_contents_color(fb5_color);
                        pane.set_white(fb5_white);
                        pane.set_black(fb5_black);
                        pane.cpos_move();
                    }
                    if let Some(pane) = self.panes.fbk.get_mut(i) {
                        pane.size.x = pane.size_orig.x * FLOOR_SCALE;
                        pane.size.y = pane.size_orig.y * FLOOR_SCALE;
                        pane.cpos_move();
                    }
                } else {
                    if let Some(pane) = self.panes.fl.get_mut(i) {
                        pane.size.x = pane.size_orig.x;
                        pane.size.y = pane.size_orig.y;
                        pane.cpos_move();
                    }
                    if let Some(pane) = self.panes.fb.get_mut(i) {
                        pane.size.x = pane.size_orig.x;
                        pane.size.y = pane.size_orig.y;
                        pane.set_contents_color(fb0_color);
                        pane.set_white(fb0_white);
                        pane.set_black(fb0_black);
                        pane.cpos_move();
                    }
                    if let Some(pane) = self.panes.fbk.get_mut(i) {
                        pane.size.x = pane.size_orig.x;
                        pane.size.y = pane.size_orig.y;
                        pane.cpos_move();
                    }
                }
            }
        }
    }

    pub fn floor_init(&mut self) {
        let temp = (i32::from(self.top_floor) - i32::from(self.bottom_floor) + 1) & 0xFF;
        let temp2 = self.panes.fb[0].pos_top_left_orig.y - self.panes.fb[1].pos_top_left_orig.y;
        let y_dif = temp2 * (FLOOR_LIST_SLOTS - temp as f32);
        let fw00_height = self.panes.fw00.size_orig.y;
        let fw01_height = self.panes.fw01.size_orig.y;
        self.panes.fw00.size_orig.y = fw00_height - y_dif;
        self.panes.fw00.size.y = fw00_height - y_dif;
        self.panes.fw01.size_orig.y = fw01_height - y_dif;
        self.panes.fw01.size.y = fw01_height - y_dif;
        self.panes.fw00.pos_center_orig.y =
            (self.panes.fw00.pos_top_left_orig.y + fw00_height) - self.panes.fw00.size_orig.y / 2.0;
        self.panes.fw00.pos_center.y = self.panes.fw00.pos_center_orig.y;
        self.panes.fw01.pos_center_orig.y =
            (self.panes.fw01.pos_top_left_orig.y + fw01_height) - self.panes.fw01.size_orig.y / 2.0;
        self.panes.fw01.pos_center.y = self.panes.fw01.pos_center_orig.y;
        self.panes.fw00.cpos_move();
        self.panes.fw01.cpos_move();
    }

    pub fn map_move(&mut self) {
        if self.panes.cc[CC_SCROLL].user_area > 0 {
            self.panes.cc[CC_SCROLL].user_area -= 1;
            let a = value_increase(
                SCROLL_STEP_LEN,
                SCROLL_STEP_LEN - i32::from(self.panes.cc[CC_SCROLL].user_area),
                0,
            );
            let target = self.floor_row_y(self.cur_floor);
            self.cur_floor_map_y +=
                (a * (f32::from(target) - f32::from(self.cur_floor_map_y))) as i16;
        } else if self.panes.cc[CC_SCROLL].user_area < 0 {
            self.panes.cc[CC_SCROLL].user_area += 1;
            let a = value_increase(
                SCROLL_STEP_LEN,
                SCROLL_STEP_LEN - i32::from(self.panes.cc[CC_SCROLL].user_area.abs()),
                0,
            );
            let target = self.floor_row_y(self.cur_floor);
            self.cur_floor_map_y +=
                (a * (f32::from(target) - f32::from(self.cur_floor_map_y))) as i16;
        }
        let map_x = self.mpp1_pos_x;
        let map_y = self.cur_floor_map_y;
        self.map.set_map_pos(map_x, map_y);
    }

    pub fn item_name_move(&mut self) {
        self.panes.nm00.user_area += 1;
        if self.panes.nm00.user_area <= 10 {
            let alpha = value_increase(10, i32::from(self.panes.nm00.user_area), 0);
            self.panes.nm00.set_now_alpha(alpha);
            self.panes.nm01.set_now_alpha(1.0 - alpha);
        }
    }

    pub fn item_name_set(&mut self, env: &MapEnv<'_>, text: &MapText<'_>) {
        let msg_no = item_name_message(
            self.select_item,
            env.has_map(),
            env.has_boss_key(),
            env.has_compass(),
        );

        let font_size0 = self.panes.nm00.get_font_size();
        self.panes.nm01.set_font_size(font_size0.0, font_size0.1);
        let char_space = self.panes.nm00.get_char_space();
        self.panes.nm01.set_char_space(char_space);

        let source = std::mem::take(&mut self.name_buf[0]).into_bytes();
        let mut copy: Vec<u8> = Vec::with_capacity(source.len());
        let mut i = 0usize;
        while i < source.len() && source[i] != 0 {
            copy.push(source[i]);
            i += 1;
        }
        let swapped = String::from_utf8_lossy(&copy).into_owned();
        self.name_buf[1] = swapped.clone();
        self.panes.nm01.set_string(&swapped);
        self.name_buf[0] = String::new();

        if msg_no == 0 {
            let empty = self.name_buf[0].clone();
            self.panes.nm00.set_string(&empty);
            return;
        }

        let r30 = 0.0f32;

        let mut get = ItemMsgGet::default();
        let head = get
            .get_mesg_header(text.archive, text.clear_count, msg_no)
            .map(|head| head.to_vec());
        let message = head.and_then(|head| get.get_message(&head).map(|bytes| bytes.to_vec()));

        let size_y = self.panes.nm00.get_font_size().1;
        let mut font_size = F2 {
            x: size_y,
            y: size_y,
        };
        let mut name: Vec<u8> = Vec::new();
        let mut str_width = 0.0f32;
        if let Some(message) = message {
            let (walked, width) =
                message_name_walk(&message, text.font, text.main_font_slot, size_y, usize::MAX);
            name = walked;
            str_width = width;
        }
        if self.panes.nm00.width() < str_width {
            font_size.x = (font_size.x * self.panes.nm00.width() / str_width) as f32;
        }

        self.name_buf[0] = String::from_utf8_lossy(&name).into_owned();
        self.panes.nm00.set_font_size(font_size.x, font_size.y);
        self.panes.nm00.set_char_space(r30);
        let out = self.name_buf[0].clone();
        self.panes.nm00.set_string(&out);
    }

    pub fn item_note_set(&mut self, env: &MapEnv<'_>, text: &MapText<'_>) {
        self.note_buf[0] = String::new();
        self.note_buf[1] = String::new();
        self.dummy_buf[0] = String::new();
        self.dummy_buf[1] = String::new();

        self.out_font_init();

        if env.opt_ruby {
            self.panes.str0.hide();
        } else {
            self.panes.str0.show();
        }

        let ruby_size = self.panes.str0.font_size_x;
        let font_size = f32::from(text.note_font_size);
        self.panes.st00.set_font_size(font_size, font_size);
        self.panes
            .st00
            .set_line_space(f32::from(text.note_line_space));

        let msg_no = item_note_message(
            self.select_item,
            env.has_map(),
            env.has_boss_key(),
            env.has_compass(),
        );

        if msg_no == 0 {
            return;
        }

        let mut get = ItemMsgGet::default();
        let Some(head) = get.get_mesg_header(text.archive, text.clear_count, msg_no) else {
            return;
        };
        let head = head.to_vec();
        let Some(mesg) = get.get_message(&head).map(|bytes| bytes.to_vec()) else {
            return;
        };
        let entry: MesgEntry = get.get_mesg_entry(&head).unwrap_or_default();

        let mut msg = MsgData::new_data_proc();
        msg.set_bmg(mesg);
        msg.out = [
            Some(String::new()),
            Some(String::new()),
            Some(String::new()),
            Some(String::new()),
        ];
        msg.entry = entry;
        msg.font = text.main_font_slot;
        msg.ruby_font = text.ruby_font_slot;
        msg.char_space = self.panes.st00.get_char_space() as i32;
        msg.ruby_char_space = self.panes.str0.get_char_space() as i32;
        msg.line_space = self.panes.st00.get_line_space() as i32;
        msg.font_size = font_size as i32;
        msg.ruby_font_size = ruby_size as i32;
        msg.line_width = NOTE_LINE_WIDTH;
        msg.center_line_width = NOTE_CENTER_LINE_WIDTH;
        msg.send_speed = NOTE_SEND_SPEED;
        msg.space_timer = NOTE_SPACE_TIMER;
        msg.short_cut();
        msg.space_flag = 0;

        msg.string_length(text.font, text.len_state);
        msg.string_shift();
        msg.icon_idx_refresh();

        let line_count = msg.line_count as i16;
        msg.line_count = 0;
        let line_space = self.panes.st00.get_line_space();
        let unused_lines = NOTE_UNUSED_LINES - i32::from(line_count);
        let shift_y = unused_lines as f32 * (line_space / 2.0);
        self.panes.str0.shift_set(0.0, shift_y);
        self.panes.st00.shift_set(0.0, shift_y);
        msg.string_set(text.font, &text.layout_state);

        self.note_buf[0] = msg.out[0].clone().unwrap_or_default();
        self.note_buf[1] = msg.out[1].clone().unwrap_or_default();
        self.dummy_buf[0] = msg.out[2].clone().unwrap_or_default();
        self.dummy_buf[1] = msg.out[3].clone().unwrap_or_default();
        self.msg = msg;

        self.panes.st00.set_string(self.note_buf[0].clone());
        self.panes.str0.set_string(self.note_buf[1].clone());

        let half_space = self.panes.st00.get_line_space() / 2.0;
        for i in 0..OUT_FONT_COUNT {
            let icon_no = self.msg.icon_num.get(i).copied().unwrap_or(ICON_NONE);
            let mut color = self.msg.icon_color.get(i).copied().unwrap_or(0);
            if color == 0xFFFF_FFFF {
                color = 0xFF;
            }
            if icon_no == ICON_NONE {
                continue;
            }
            if self.panes.ft[i].user_area != -1 {
                continue;
            }
            if icon_no == ICON_INPUT {
                continue;
            }
            self.panes.ft[i].pos_top_left.x =
                self.msg.icon_pos_x.get(i).copied().unwrap_or(0) as f32;
            self.panes.ft[i].pos_top_left.y = half_space
                * (unused_lines as f32
                    + f32::from(self.msg.icon_pos_y.get(i).copied().unwrap_or(0)) * 2.0);
            self.panes.ft[i].pos_top_left_orig.y = f32::from(icon_no);

            let slot = out_font_slot(icon_no);
            let texture = icon_texture(slot);
            self.panes.ft[i].show();
            self.panes.ft[i].blend_draw(texture);
            if icon_uses_text_color(slot) {
                let raw = unpack_color(color);
                self.panes.ft[i].set_white(pack_color([raw[0], raw[1], raw[2], 0xFF]));
                self.panes.ft[i].set_black(0x0000_0000);
            } else {
                self.panes.ft[i].set_white(icon_color(slot));
                self.panes.ft[i].set_black(0x0000_0000);
            }
            self.panes.ft[i].set_blend([0.0, 1.0, 1.0, 1.0]);
            self.panes.ft[i].user_area = 0;
        }
    }

    pub fn out_font_init(&mut self) {
        for i in 0..OUT_FONT_COUNT {
            self.panes.ft[i].user_area = -1;
            self.panes.ft[i].rotate(0.0, 0.0, 0.0);
            self.panes.ft[i].set_now_alpha_zero();
        }
    }

    pub fn link_anime(&mut self) {
        self.panes.lnk[1].user_area += 1;
        self.panes.lnk[2].user_area += 1;
        if self.panes.lnk[1].user_area > self.lnk_timer1 {
            self.panes.lnk[1].user_area = 0;
            if !self.lnk_tex_flip1 {
                self.lnk_timer1 = 5;
                self.lnk_tex_flip1 = true;
            } else {
                self.lnk_timer1 = (self.rng.rnd_f(50.0) + 100.0) as i16;
                self.lnk_tex_flip1 = false;
            }
        }

        if self.panes.lnk[2].user_area > self.lnk_timer2 {
            self.panes.lnk[2].user_area = 0;
            self.lnk_timer2 = (self.rng.rnd_f(100.0) + 100.0) as i16;
            if !self.lnk_tex_flip2 {
                self.lnk_tex_flip2 = true;
            } else {
                self.lnk_tex_flip2 = false;
            }
        }

        if !self.lnk_tex_flip1 {
            self.panes.lnk[1].change_texture(LINK_EYE_TEXTURE[0]);
        } else {
            self.panes.lnk[1].change_texture(LINK_EYE_TEXTURE[1]);
        }

        if !self.lnk_tex_flip2 {
            self.panes.lnk[2].change_texture(LINK_MOUTH_TEXTURE[0]);
        } else {
            self.panes.lnk[2].change_texture(LINK_MOUTH_TEXTURE[1]);
        }
    }

    pub fn boss_anime(&mut self) {
        let mut move_y = 0.0f32;
        let mut time = BOSS_ROW_TIME;
        time[0] = self.tuning.boss_up_time;
        time[1] = self.tuning.boss_wait_time;
        time[2] = self.tuning.boss_down_time;
        time[4] = self.tuning.boss_move_time1;
        time[6] = self.tuning.boss_move_time2;
        time[8] = self.tuning.boss_move_time3;
        time[10] = self.tuning.boss_move_time4;

        let mut total = [0i16; 15];
        total[0] = time[0];
        for i in 1..15 {
            total[i] = total[i - 1] + time[i];
        }

        self.panes.boss.user_area += 1;
        if self.panes.boss.user_area >= self.panes.bos2.user_area {
            self.panes.boss.user_area = 0;
            self.panes.bos2.user_area = (self.rng.rnd_f(200.0) + 300.0) as i16;
        } else if self.panes.boss.user_area < total[10] {
            let step = i32::from(self.panes.boss.user_area);
            if self.panes.boss.user_area < total[0] {
                move_y = value_increase(i32::from(time[0]), step, 0) * BOSS_ANIM_STEP;
            } else if self.panes.boss.user_area < total[1] {
                move_y = BOSS_ANIM_STEP;
            } else if self.panes.boss.user_area < total[2] {
                move_y = value_increase(i32::from(time[2]), i32::from(total[2]) - step, 0)
                    * BOSS_ANIM_STEP;
            } else if self.panes.boss.user_area < total[3] {
                move_y = 0.0;
            } else if self.panes.boss.user_area < total[4] {
                move_y = value_increase(i32::from(time[4]), i32::from(total[4]) - step, 0)
                    * BOSS_ANIM_STEP;
            } else if self.panes.boss.user_area < total[5] {
                move_y = BOSS_ANIM_STEP;
            } else if self.panes.boss.user_area < total[6] {
                move_y = value_increase(i32::from(time[6]), i32::from(total[6]) - step, 0)
                    * BOSS_ANIM_STEP;
            } else if self.panes.boss.user_area < total[7] {
                move_y = 0.0;
            } else if self.panes.boss.user_area < total[8] {
                move_y = value_increase(i32::from(time[8]), i32::from(total[8]) - step, 0)
                    * BOSS_ANIM_STEP;
            } else if self.panes.boss.user_area < total[9] {
                move_y = BOSS_ANIM_STEP;
            } else if self.panes.boss.user_area < total[10] {
                move_y = value_increase(i32::from(time[10]), i32::from(total[10]) - step, 0)
                    * BOSS_ANIM_STEP;
            } else if self.panes.boss.user_area < total[11] {
                move_y = 0.0;
            } else if self.panes.boss.user_area < total[12] {
                move_y = value_increase(i32::from(time[12]), i32::from(total[12]) - step, 0)
                    * BOSS_ANIM_STEP;
            } else if self.panes.boss.user_area < total[13] {
                move_y = BOSS_ANIM_STEP;
            } else if self.panes.boss.user_area < total[14] {
                move_y = value_increase(i32::from(time[14]), i32::from(total[14]) - step, 0)
                    * BOSS_ANIM_STEP;
            }
        }

        self.panes
            .boss
            .pane_trans(0.0, f32::from(self.boss_floor_list_offset_y) - move_y);
        self.panes.bos2.pane_trans(0.0, move_y * 2.0);
        self.boss_eye_anime();
    }

    pub fn boss_eye_anime(&mut self) {
        let i_max = i32::from(self.boss_eye_timer / 2);
        self.panes.bey1.user_area += 1;

        let alpha;
        if self.boss_eye_state != 0 {
            if i32::from(self.panes.bey1.user_area) < i_max {
                alpha = value_increase(i_max, i32::from(self.panes.bey1.user_area), 2);
            } else {
                alpha = value_increase(
                    i_max,
                    i32::from(self.boss_eye_timer) - i32::from(self.panes.bey1.user_area),
                    2,
                );
            }
        } else {
            alpha = 0.0;
        }

        if self.panes.bey1.user_area == self.boss_eye_timer {
            self.panes.bey1.user_area = 0;
            self.boss_eye_timer = ((self.rng.rnd_f(18.0) + 40.0) as i32 * 2) as i16;
            if self.boss_eye_state != 0 {
                self.boss_eye_state = 0;
            } else {
                self.boss_eye_state = 1;
            }
        }

        self.panes.bey1.set_now_alpha(alpha);
        self.panes.bey2.set_now_alpha(alpha);
    }

    pub fn move_frame(&mut self, env: &MapEnv<'_>, text: &MapText<'_>, pad: &mut PadInput) {
        if !self.note_check() {
            if !pad.up && !pad.trig_b {
                if pad.trig_a {
                    if (self.select_item == 0 && env.has_map())
                        || (self.select_item == 1 && env.has_boss_key())
                        || (self.select_item == 2 && env.has_compass())
                    {
                        self.panes.str0.set_init_alpha();
                        self.panes.st00.set_init_alpha();
                        self.panes.nt00.set_init_alpha();
                        self.panes.nk00.set_init_alpha();
                        self.panes.no11.set_init_alpha();
                        self.panes.nk00.user_area = 1;
                        self.panes.nt00.user_area = 1;
                        self.item_note_set(env, text);
                        self.sound(SE_ITEM_EXP_OPEN);
                    }
                } else {
                    self.cursor_move(env, text, pad);
                }
            }
        } else {
            self.note_appear(pad);
        }

        if self.panes.nm00.user_area < 10 {
            self.item_name_move();
        }
        self.map_move();
        self.link_anime();
        self.boss_anime();
        self.cursor_anime();
    }

    pub fn draw(&mut self) -> MapDraw {
        let mut main: Vec<DrawItem> = Vec::new();
        let mut explain: Vec<DrawItem> = Vec::new();

        for i in 0..OUT_FONT_COUNT {
            self.panes.ft[i].set_alpha();
            explain.push(self.panes.ft[i].draw_item());
        }

        for i in 0..CURSOR_SLOT_COUNT {
            self.panes.car[i].set_alpha();
            main.push(self.panes.car[i].draw_item());
        }

        self.panes.nm00.set_alpha();
        explain.push(self.panes.nm00.draw_item());
        self.panes.nm01.set_alpha();
        explain.push(self.panes.nm01.draw_item());
        self.panes.itnm.set_alpha();
        explain.push(self.panes.itnm.draw_item());
        self.panes.itnk.set_alpha();
        explain.push(self.panes.itnk.draw_item());
        self.panes.msk0.set_alpha();
        explain.push(self.panes.msk0.draw_item());
        self.panes.str0.set_alpha();
        explain.push(self.panes.str0.draw_item());
        self.panes.st00.set_alpha();
        explain.push(self.panes.st00.draw_item());
        self.panes.nt00.set_alpha();
        explain.push(self.panes.nt00.draw_item());
        self.panes.nk00.set_alpha();
        explain.push(self.panes.nk00.draw_item());
        self.panes.no11.set_alpha();
        explain.push(self.panes.no11.draw_item());
        self.panes.dtle.set_alpha();
        explain.push(self.panes.dtle.draw_item());
        self.panes.dt00.set_alpha();
        explain.push(self.panes.dt00.draw_item());
        self.panes.dk00.set_alpha();
        explain.push(self.panes.dk00.draw_item());

        for i in 0..ITEM_ROW_COUNT {
            self.panes.it[i].set_alpha();
            main.push(self.panes.it[i].draw_item());
            self.panes.ik[i].set_alpha();
            main.push(self.panes.ik[i].draw_item());
            self.panes.ip[i].set_alpha();
            main.push(self.panes.ip[i].draw_item());
        }

        self.panes.ip01.set_alpha();
        main.push(self.panes.ip01.draw_item());
        self.panes.lnk[0].set_alpha();
        main.push(self.panes.lnk[0].draw_item());
        self.panes.bey1.set_alpha();
        main.push(self.panes.bey1.draw_item());
        self.panes.bey2.set_alpha();
        main.push(self.panes.bey2.draw_item());
        self.panes.boss.set_alpha();
        main.push(self.panes.boss.draw_item());

        for i in 0..FLOOR_ROW_COUNT {
            self.panes.fl[i].set_alpha();
            main.push(self.panes.fl[i].draw_item());
            self.panes.fb[i].set_alpha();
            main.push(self.panes.fb[i].draw_item());
            self.panes.fbk[i].set_alpha();
            main.push(self.panes.fbk[i].draw_item());
        }

        self.panes.fw00.set_alpha();
        main.push(self.panes.fw00.draw_item());
        self.panes.fw01.set_alpha();
        main.push(self.panes.fw01.draw_item());
        for i in 0..CC_COUNT {
            self.panes.cc[i].set_alpha();
            main.push(self.panes.cc[i].draw_item());
        }
        self.panes.msk.set_alpha();
        main.push(self.panes.msk.draw_item());

        let alpha = self.panes.cc[CC_ALPHA].now_alpha;
        self.map.set_alpha(alpha);
        self.map.draw();
        let markers = self.treasure_draw();
        self.out.frames += 1;

        MapDraw {
            main,
            map: self.map,
            markers,
            explain,
        }
    }

    pub fn open(&mut self, env: &MapEnv<'_>, text: &MapText<'_>) -> bool {
        let mut ret = false;

        if self.note_timer == 0 {
            self.name_buf[0] = String::new();
            self.name_buf[1] = String::new();
            self.note_buf[0] = String::new();
            self.note_buf[1] = String::new();
            self.panes.nm00.set_string(&self.name_buf[0]);
            self.panes.nm01.set_string(&self.name_buf[1]);

            self.panes.st00.set_string(&self.note_buf[0]);
            self.panes.str0.set_string(&self.note_buf[1]);
            self.item_name_set(env, text);
            self.item_note_set(env, text);
        }

        self.note_timer += 1;

        let r0 = OPEN_STEPS;
        if self.note_timer <= r0 {
            let alpha = value_increase(OPEN_STEP_LEN, i32::from(self.note_timer), 0);
            let rate = value_increase(OPEN_STEP_LEN, OPEN_STEP_LEN - i32::from(self.note_timer), 0);
            self.map_draw_offset_y = (rate * TRANSITION_SLIDE) as i16;
            self.pane_move(rate * TRANSITION_SLIDE);

            let mask_x = self.mpmk_pos_x;
            let mask_y = self.mpmk_pos_y + self.map_draw_offset_y;
            self.map.set_mask_pos(mask_x, mask_y);
            let map_x = self.mpp1_pos_x;
            let map_y = self.cur_floor_map_y + self.map_draw_offset_y;
            self.map.set_map_pos(map_x, map_y);

            self.pane_alpha(alpha);
            self.note_open_proc(self.note_timer);
            self.cur_floor_only = true;
        }

        let r0 = OPEN_STEPS;
        if self.note_timer >= r0 {
            ret = true;
            self.cur_floor_only = false;
            self.sound(SE_MENU_ITEMS_IN);
        }
        ret
    }

    pub fn close(&mut self) -> bool {
        let mut ret = false;
        self.note_timer -= 1;
        if self.note_timer > 0 {
            let alpha = value_increase(OPEN_STEP_LEN, i32::from(self.note_timer), 0);
            let rate = value_increase(OPEN_STEP_LEN, OPEN_STEP_LEN - i32::from(self.note_timer), 0);
            self.map_draw_offset_y = (rate * TRANSITION_SLIDE) as i16;
            self.pane_move(rate * TRANSITION_SLIDE);

            let mask_x = self.mpmk_pos_x;
            let mask_y = self.mpmk_pos_y + self.map_draw_offset_y;
            self.map.set_mask_pos(mask_x, mask_y);
            let map_x = self.mpp1_pos_x;
            let map_y = self.cur_floor_map_y + self.map_draw_offset_y;
            self.map.set_map_pos(map_x, map_y);

            self.out.do_status = Some(DO_STATUS_BLANK);
            self.dec_alpha(alpha);
            self.cur_floor_only = true;
        } else {
            self.pane_move(TRANSITION_SLIDE);
            self.dec_alpha(0.0);
            ret = true;
            self.cur_floor_only = false;
        }
        ret
    }
}
