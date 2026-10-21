use core::f32::consts::PI;

use crate::j2d::{self, Kind, Screen};
use crate::sim::flags::{Charts, Collect, StoryBits};
use crate::sim::health::Health;
use crate::sim::inventory::Equipped;
use crate::sim::snap::SnapPacket;
use crate::text::font::FontBank;
use crate::text::msgdata::MsgLenState;
use crate::text::msgget::{ItemMsgGet, MsgArchive};
use crate::text::msglayout::MsgLayoutState;
use crate::text::msgproc::{
    MeterMsgHio, MsgIconPic, fop_msgm_out_font_draw, fop_msgm_out_font_draw2,
    fop_msgm_out_font_set, fop_msgm_out_font_set1,
};
use crate::text::{ICON_NONE, MesgEntry, MsgData};

pub const SCREEN_LABEL: &str = "menu_collect_01.blo";
pub const ALBUM_MAIN_LABEL: &str = "wipe_01_01.blo";
pub const ALBUM_SHUTTER_LABEL: &str = "wipe_01_02.blo";
pub const ALBUM_MESSAGE_LABEL: &str = "hukidashi_08.blo";

pub const PANE_COUNT: usize = 166;
pub const ALBUM_PANE_COUNT: usize = 74;
pub const NOTE_ICON_COUNT: usize = 15;
pub const OUT_FONT_ICON_COUNT: usize = 3;
pub const CURSOR_CORNER_COUNT: usize = 4;
pub const SONG_COUNT: usize = 6;
pub const SHUTTER_SEGMENT_COUNT: usize = 12;
pub const SLOT_COUNT: usize = 3;
pub const PHOTO_BUFFER_COUNT: usize = 4;
pub const NAME_BUFFER_COUNT: usize = 2;
pub const MESSAGE_BUFFER_COUNT: usize = 4;

pub const ITEM_NONE: u8 = 0xFF;
pub const ITEM_PICTO_BOX: u8 = 0x23;
pub const ITEM_SLOT_CAMERA: u8 = 8;

pub const P_FT15: usize = 0;
pub const P_FT16: usize = 1;
pub const P_FT17: usize = 2;
pub const P_FT0: usize = 3;
pub const P_FD0: usize = 18;
pub const P_STR0: usize = 33;
pub const P_ST00: usize = 34;
pub const P_NT00: usize = 35;
pub const P_NK00: usize = 36;
pub const P_NO11: usize = 37;
pub const P_WD: usize = 38;
pub const P_NM00: usize = 39;
pub const P_NM01: usize = 40;
pub const P_ITNM: usize = 41;
pub const P_ITNK: usize = 42;
pub const P_MSK0: usize = 43;
pub const P_TL00: usize = 44;
pub const P_TK00: usize = 45;
pub const P_CAR0: usize = 46;
pub const P_WDSV: usize = 50;
pub const P_WDOP: usize = 51;
pub const P_CORNER0: usize = 52;
pub const P_NUM1: usize = 56;
pub const P_NUM2: usize = 57;
pub const P_NUMC: usize = 58;
pub const P_CMAP: usize = 59;
pub const P_WK1: usize = 60;
pub const P_WK0: usize = 61;
pub const P_TRIM: usize = 62;
pub const P_TRIL: usize = 63;
pub const P_TRI0: usize = 64;
pub const P_TRIB: usize = 72;
pub const P_SIT: usize = 73;
pub const P_SIK: usize = 76;
pub const P_CI: usize = 79;
pub const P_HTP: usize = 90;
pub const P_HTPB: usize = 93;
pub const P_TIT: usize = 94;
pub const P_TIK: usize = 100;
pub const P_CN: usize = 106;
pub const P_WN: usize = 112;
pub const P_I0: usize = 118;
pub const P_I1: usize = 124;
pub const P_BS: usize = 130;
pub const P_HMB: usize = 136;
pub const P_I111: usize = 142;
pub const P_I122: usize = 143;
pub const P_CC43: usize = 144;
pub const P_CC32: usize = 145;
pub const P_CC21: usize = 146;
pub const P_WIT: usize = 147;
pub const P_WIK: usize = 152;
pub const P_IP: usize = 157;
pub const P_IP00: usize = 162;
pub const P_CC14: usize = 163;
pub const P_CC07: usize = 164;
pub const P_CC00: usize = 165;

pub const A_SB0: usize = 0;
pub const A_ST0: usize = 12;
pub const A_YLIG: usize = 24;
pub const A_YLEF: usize = 25;
pub const A_ICN0: usize = 26;
pub const A_EMP0: usize = 29;
pub const A_CT1: usize = 32;
pub const A_CT2: usize = 33;
pub const A_SP0: usize = 34;
pub const A_NO0: usize = 42;
pub const A_NOB0: usize = 45;
pub const A_NOK0: usize = 48;
pub const A_B1: usize = 51;
pub const A_B2: usize = 52;
pub const A_WNUM: usize = 53;
pub const A_WNUK: usize = 54;
pub const A_WPBA: usize = 55;
pub const A_WBAP: usize = 56;
pub const A_WPSC: usize = 57;
pub const A_YRTN: usize = 58;
pub const A_YZOM: usize = 59;
pub const A_CRTN: usize = 60;
pub const A_CZOM: usize = 61;
pub const A_LRTN: usize = 62;
pub const A_RZOM: usize = 63;
pub const A_SHUT: usize = 64;
pub const A_FD00: usize = 65;
pub const A_WP03: usize = 66;
pub const A_WP02: usize = 67;
pub const A_WP01: usize = 68;
pub const A_WP04: usize = 69;
pub const A_TX0: usize = 70;

pub const TACT_BEAT: [i16; SONG_COUNT] = [3, 4, 4, 6, 6, 3];

pub const TACT_ROTATE_ANGLE: [i16; 5] = [225, 225, 135, 45, 315];

pub const TACT_STICK_WAIT_FIRST: i16 = 5;
pub const TACT_STICK_WAIT_SECOND: i16 = 2;
pub const TACT_STICK_STEP: i16 = 3;
pub const TACT_STICK_MIN_STEP: i16 = 2;
pub const TACT_STICK_MAX_VALUE: f32 = 0.9;
pub const TACT_STICK_MIN_VALUE: f32 = 0.5;
pub const TACT_STICK_REPEAT_ANGLE: i16 = 0;
pub const TACT_STICK_HALF_DEAD: i16 = 0x800;

pub const MELODY_PATTERN: [[u8; 7]; 8] = [
    [0x03, 0x01, 0x04, 0x02, 0xFF, 0xFF, 0xFF],
    [0x04, 0x03, 0x02, 0x04, 0x01, 0xFF, 0xFF],
    [0x04, 0x04, 0x00, 0x02, 0x00, 0xFF, 0xFF],
    [0x06, 0x03, 0x03, 0x00, 0x02, 0x04, 0x00],
    [0x06, 0x01, 0x01, 0x03, 0x02, 0x04, 0x02],
    [0x03, 0x02, 0x04, 0x03, 0xFF, 0xFF, 0xFF],
    [0x03, 0x01, 0x03, 0x02, 0xFF, 0xFF, 0xFF],
    [0x04, 0x01, 0x02, 0x04, 0x03, 0xFF, 0xFF],
];

pub const C_POS_NOTE_3: [u8; 5] = [0x30, 0x3C, 0x39, 0x3E, 0x35];
pub const C_POS_NOTE_4: [u8; 5] = [0x3B, 0x3C, 0x39, 0x3E, 0x35];
pub const C_POS_NOTE_61: [u8; 5] = [0x3E, 0x45, 0x42, 0x39, 0x40];
pub const C_POS_NOTE_62: [u8; 5] = [0x3E, 0x45, 0x42, 0x43, 0x40];

pub const BEAT_LEN_3: f32 = 3.0;
pub const BEAT_LEN_4: f32 = 3.42857;
pub const BEAT_LEN_6: f32 = 4.5;
pub const BEAT_FRAMES_3: f32 = 60.0;
pub const BEAT_FRAMES_4: f32 = 70.0;
pub const BEAT_FRAMES_6: f32 = 80.0;
pub const BEAT_SCALE_3: f32 = 1.0;
pub const BEAT_SCALE_4: f32 = 0.85714;
pub const BEAT_SCALE_6: f32 = 0.75;
pub const BEAT_DEFAULT: i32 = 3;
pub const METRONOME_FALLBACK_NOTE: u16 = 100;

pub const NOTE_FALLBACK: u16 = 0xFFFF;

pub const TRIFORCE_TEX: [&str; 8] = [
    "triforce_00.bti",
    "triforce_01.bti",
    "triforce_02.bti",
    "triforce_03.bti",
    "triforce_04.bti",
    "triforce_05.bti",
    "triforce_06.bti",
    "triforce_07.bti",
];
pub const TRIFORCE_FULL_TEX: &str = "triforce.bti";
pub const TRIFORCE_FULL_WIDTH: f32 = 80.0;
pub const TRIFORCE_FULL_HEIGHT: f32 = 64.0;
pub const TRIFORCE_FULL_SCALE: f32 = 1.45;
pub const SYMBOL_TEX: [&str; 3] = [
    "god_symbol_02.bti",
    "god_symbol_00.bti",
    "god_symbol_01.bti",
];
pub const WEAPON_TEX: [&str; 5] = [
    "sword_00.bti",
    "shield_00.bti",
    "gloves_00.bti",
    "amulet_00.bti",
    "amulet_01.bti",
];
pub const SWORD_TEX: [&str; 4] = [
    "sword_00.bti",
    "sword_01.bti",
    "sword_02.bti",
    "sword_03.bti",
];
pub const SHIELD_TEX: [&str; 2] = ["shield_00.bti", "shield_01.bti"];
pub const TACT_TEX: &str = "baton.bti";
pub const MAP_TEX: &str = "cmap_treasure2.bti";
pub const CURSOR_TEX: &str = "cursor_00_02.bti";
pub const OUT_FONT_TEX: &str = "font_07_02.bti";
pub const OUT_FONT_BLEND_TEX: &str = "font_00.bti";
pub const SELECT_ARROW_TEX: &str = "font_10.bti";
pub const CORNER_TEX: &str = "font_09_02.bti";
pub const RUPEE_NUM_TEX: [&str; 10] = [
    "rupy_num_00.bti",
    "rupy_num_01.bti",
    "rupy_num_02.bti",
    "rupy_num_03.bti",
    "rupy_num_04.bti",
    "rupy_num_05.bti",
    "rupy_num_06.bti",
    "rupy_num_07.bti",
    "rupy_num_08.bti",
    "rupy_num_09.bti",
];

pub const NOTE_TAG_LIST: [&str; 15] = [
    "ft00", "ft01", "ft02", "ft03", "ft04", "ft05", "ft06", "ft07", "ft08", "ft09", "ft10", "ft11",
    "ft12", "ft13", "ft14",
];
pub const COLLECT_TAG_TABLE: [&str; PANE_COUNT] = [
    "ft15", "ft16", "ft17", "ft00", "ft01", "ft02", "ft03", "ft04", "ft05", "ft06", "ft07", "ft08",
    "ft09", "ft10", "ft11", "ft12", "ft13", "ft14", "fd00", "fd01", "fd02", "fd03", "fd04", "fd05",
    "fd06", "fd07", "fd08", "fd09", "fd10", "fd11", "fd12", "fd13", "fd14", "str0", "st00", "nt00",
    "nk00", "no11", "wd", "nm00", "nm01", "itnm", "itnk", "msk0", "tl00", "tk00", "car1", "car2",
    "car3", "car4", "wdsv", "wdop", "wk00", "wk01", "wk02", "wk03", "num1", "num2", "numc", "cmap",
    "wk1", "wk0", "trim", "tril", "tri0", "tri1", "tri2", "tri3", "tri4", "tri5", "tri6", "tri7",
    "trib", "it16", "it14", "it15", "ik16", "ik14", "ik15", "ci10", "ci09", "ci08", "ci07", "ci06",
    "ci05", "ci04", "ci03", "ci02", "ci01", "ci00", "htp2", "htp1", "htp3", "htpb", "it08", "it09",
    "it10", "it11", "it12", "it13", "ik08", "ik09", "ik10", "ik11", "ik12", "ik13", "cn01", "cn02",
    "cn03", "cn04", "cn05", "cn06", "wn01", "wn02", "wn03", "wn04", "wn05", "wn06", "i011", "i021",
    "i031", "i041", "i051", "i061", "i012", "i022", "i032", "i042", "i052", "i062", "bs01", "bs02",
    "bs03", "bs04", "bs05", "bs06", "hmb1", "hmb2", "hmb3", "hmb4", "hmb5", "hmb6", "i111", "i122",
    "cc43", "cc32", "cc21", "it00", "it01", "it02", "it03", "it04", "ik00", "ik01", "ik02", "ik03",
    "ik04", "ip09", "ip07", "ip05", "ip03", "ip01", "ip00", "cc14", "cc07", "cc00",
];

pub const ALBUM_TAG_TABLE: [&str; ALBUM_PANE_COUNT] = [
    "sb01", "sb02", "sb03", "sb04", "sb05", "sb06", "sb07", "sb08", "sb09", "sb10", "sb11", "sb12",
    "st01", "st02", "st03", "st04", "st05", "st06", "st07", "st08", "st09", "st10", "st11", "st12",
    "ylig", "ylef", "icn1", "icn2", "icn3", "emp1", "emp2", "emp3", "ct1", "ct2", "sp00", "sp01",
    "sp02", "sp03", "sp04", "sp05", "sp06", "sp07", "no1", "no2", "no3", "no1b", "no2b", "no3b",
    "no1k", "no2k", "no3k", "b1", "b2", "wnum", "wnuk", "wpba", "wbap", "wpsc", "yrtn", "yzom",
    "crtn", "czom", "lrtn", "rzom", "shut", "fd00", "wp03", "wp02", "wp01", "wp04", "tx82", "tx83",
    "tx80", "tx81",
];

pub const ALBUM_HIDDEN_TAG_LIST: [&str; 2] = ["yz80", "dt80"];

pub const ITEM_MESSAGE: [i16; 256] = [
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0194, 0x0000, 0x0000, 0x0000, 0x019F,
    0x019F, 0x019F, 0x019F, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x01A0, 0x0191, 0x019C, 0x01A1, 0x01A2,
    0x01A3, 0x0192, 0x01A4, 0x01A5, 0x01A6, 0x01A7, 0x01A8, 0x01A9, 0x01AA, 0x01AB, 0x0000, 0x01AC,
    0x01AD, 0x019F, 0x0000, 0x01AE, 0x01AF, 0x01B0, 0x01B1, 0x0000, 0x0195, 0x01B2, 0x01B5, 0x0196,
    0x01B4, 0x0000, 0x01B3, 0x0194, 0x0000, 0x0000, 0x0193, 0x019E, 0x01B5, 0x01B6, 0x01B7, 0x01B8,
    0x01B9, 0x01BA, 0x01BB, 0x01BC, 0x0197, 0x0198, 0x0199, 0x01BD, 0x01BE, 0x01BF, 0x01C0, 0x01C1,
    0x01F0, 0x01EE, 0x01C2, 0x01EF, 0x01EC, 0x01ED, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x01C3, 0x01C3, 0x01C3, 0x01C3, 0x01C3, 0x01C3, 0x01C3, 0x01C3, 0x01C5, 0x01C6, 0x01C7,
    0x01C8, 0x01C9, 0x01CA, 0x01CB, 0x01CC, 0x01CD, 0x01CE, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x01CF, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x01D3, 0x01D2,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x01D9, 0x01DA, 0x01DB, 0x01DC,
    0x01DD, 0x01DE, 0x01DF, 0x01E0, 0x01E1, 0x01E2, 0x01E3, 0x01E4, 0x01E5, 0x01F6, 0x01E6, 0x01E7,
    0x01E8, 0x01E9, 0x01EA, 0x01A4, 0x01EC, 0x01ED, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000,
];

pub const MSG_SAVE_WORD: u32 = 0x19D;
pub const MSG_SONG_WORD: u32 = 0x1F8;
pub const MSG_MAP_WORD: u32 = 0x1F7;
pub const MSG_TRIFORCE_FULL: u32 = 0x1C4;
pub const MSG_NO_EQUIP: u32 = 0x25C;
pub const MSG_MAP_NOTE: u32 = 0x2BF;
pub const MSG_TRIFORCE_NOTE: u32 = 0x28C;
pub const MSG_SAVE_NOTE: u32 = 0x265;
pub const MSG_SONG_NOTE: u32 = 0x2C0;
pub const MSG_NO_EQUIP_NOTE: u32 = 0x25C;
pub const MSG_OUT_FONT_SONG: u32 = 0x1FA;
pub const MSG_OUT_FONT_SAVE: u32 = 0x1FB;
pub const MSG_OUT_FONT_SONG_FALLBACK: u32 = 0x1FC;
pub const MSG_OUT_FONT_BOTTLE: u32 = 0x1FD;
pub const MSG_HEART_ITEM: i16 = 7;
pub const MSG_TACT_ITEM: [i16; SONG_COUNT] = [109, 110, 112, 113, 114, 111];
pub const MSG_TACT_NOTE_ITEM: [i16; SONG_COUNT] = [109, 110, 111, 112, 113, 114];
pub const MSG_SYMBOL_ITEM: [i16; 3] = [105, 106, 107];
pub const MSG_TRIFORCE_ITEM: i16 = 97;
pub const MSG_HOOD_ITEM: i16 = 66;
pub const MSG_BOTTLE_ITEM: i16 = 67;

pub const NOTE_MESSAGE_BASE: u32 = 200;
pub const NOTE_LINE_WIDTH: i32 = 0x1FE;
pub const NOTE_CENTER_LINE_WIDTH: i32 = 0x1E6;
pub const NOTE_UNUSED_LINES: i32 = 3;
pub const NOTE_SEND_SPEED: i32 = 2;
pub const NOTE_SPACE_TIMER: i32 = 0;
pub const MSG_FONT_SIZE: i32 = 0x17;
pub const MSG_LINE_SPACE: i32 = 0x1E;
pub const MSG_SEND_SPEED: i32 = 2;
pub const MSG_SPACE_TIMER: i32 = 0;
pub const MSG_SPACE_FLAG: i32 = 0;
pub const NAME_CELL_SCALE: f32 = 29.0;
pub const RUBY_OPTION_DY: f32 = -4.0;
pub const ALBUM_LINE_WIDTH: i32 = 0x1F7;
pub const ALBUM_CENTER_LINE_WIDTH: i32 = 0x1E6;
pub const ALBUM_UNUSED_LINES: i32 = 2;
pub const ALBUM_RUBY_DY: f32 = -14.0;
pub const ALBUM_PLAIN_DY: f32 = -10.0;
pub const ALBUM_RUBY_HEAD_DY: f32 = -3.0;
pub const ARROW_ANIME_PERIOD: i16 = 45;
pub const ARROW_ANIME_SPAN: i16 = 0x16;
pub const ARROW_ALPHA_LOW: u8 = 0x96;
pub const ARROW_ALPHA_HIGH: u8 = 0xFF;

pub const EVENT_PLACED_DIN_PEARL: u16 = 0x1480;
pub const EVENT_PLACED_FARORE_PEARL: u16 = 0x1440;
pub const EVENT_PLACED_NAYRU_PEARL: u16 = 0x1410;
pub const EVENT_MAP_OPEN: u16 = 0x0908;
pub const EVENT_PICTURE_ERASE: u16 = 0x89FF;

pub const ACT_BLANK: u8 = 0x00;
pub const ACT_RETURN: u8 = 0x07;
pub const ACT_CHOOSE: u8 = 0x17;
pub const ACT_NEXT: u8 = 0x19;
pub const ACT_TAKE_PHOTO: u8 = 0x20;
pub const ACT_INFO: u8 = 0x21;
pub const ACT_SWAP_MODES: u8 = 0x22;
pub const ACT_HIDDEN: u8 = 0x3E;

pub const STATUS_STOP: u8 = 0x07;
pub const STATUS_SCOPE_ACTIVE: u8 = 0x0B;
pub const STATUS_UNKNOWN0: u8 = 0x00;
pub const STATUS_BOX_CLOSED: u8 = 0x12;
pub const STATUS_MSG_DESTROYED: u8 = 0x13;

pub const SE_CURSOR: u32 = 0x080E;
pub const SE_TELESCOPE_ZOOM: u32 = 0x0008;
pub const SE_TELESCOPE_STOP: u32 = 0x0809;
pub const SE_PANEL_IN: u32 = 0x0811;
pub const SE_ITEMS_IN: u32 = 0x0813;
pub const SE_EXP_IN: u32 = 0x0814;
pub const SE_EXP_OUT: u32 = 0x0816;
pub const SE_SUBMENU_IN_2: u32 = 0x0823;
pub const SE_SUBMENU_OUT: u32 = 0x0824;
pub const SE_TALK_CURSOR: u32 = 0x083D;
pub const SE_CANCEL: u32 = 0x084B;
pub const SE_OPT_IN: u32 = 0x084C;
pub const SE_SHUTTER: u32 = 0x0878;
pub const SE_DANCE_01: u32 = 0x0889;
pub const SE_PULSE_3: u32 = 0x0893;
pub const SE_PULSE_4: u32 = 0x0894;
pub const SE_PULSE_6: u32 = 0x0895;
pub const SE_EXP_OPEN: u32 = 0x08B6;
pub const SE_EQUIP_ON: u32 = 0x08B7;
pub const SE_EQUIP_OFF: u32 = 0x08B8;
pub const SE_CHANGE_MODE: u32 = 0x08D3;
pub const SE_SAVE_PIC: u32 = 0x08D4;
pub const SE_DEL_PIC: u32 = 0x08D5;
pub const SE_SWITCH_PIC: u32 = 0x08D6;
pub const SE_PIC_OK: u32 = 0x08D7;
pub const SE_B_DEL_PIC: u32 = 0x08D8;
pub const SE_B_LEAVE_PIC: u32 = 0x08D9;
pub const SE_TACT_RIGHT: u32 = 0x08F6;
pub const SE_TACT_WRONG: u32 = 0x08F7;
pub const SE_TRIFORCE_BECOME_ONE: u32 = 0x0908;
pub const SE_B_SHOW: u32 = 0x090C;
pub const SE_METRONOME: u32 = 0x0887;

pub const CURSOR_FAST_SCALE: f32 = 1.4;
pub const CURSOR_OPEN_FRAMES: i16 = 7;
pub const SHUTTER_TIME: i16 = 5;
pub const SHUTTER_ANGLE: f32 = 150.0;
pub const SHUTTER_LINE_ALPHA: u8 = 0x8C;
pub const SHUTTER_LINE_WIDTH: i32 = 0xC;
pub const SCREEN_W: f32 = 640.0;
pub const SCREEN_H: f32 = 480.0;
pub const CENTER_X: f32 = 320.0;
pub const CENTER_Y: f32 = 240.0;

pub const PHOTO_INDEX: [u8; 7] = [0x88, 0xCE, 0x85, 0x53, 0x86, 0xA9, 0x9B];
pub const PHOTO_FILE: [&str; 7] = [
    "/res/Photo/KOKUOU.DDS",
    "/res/Photo/GANONDROF.DDS",
    "/res/Photo/DAIYOUSEI.DDS",
    "/res/Photo/JAV.DDS",
    "/res/Photo/CHOUYOUSEI.DDS",
    "/res/Photo/KAZEKENJA.DDS",
    "/res/Photo/TIKENJA.DDS",
];
pub const PHOTO_TEX_WIDTH: u16 = 0x98;
pub const PHOTO_TEX_HEIGHT: u16 = 0x68;
pub const PHOTO_TEX_SIZE: usize = 0x1EE0;
pub const PHOTO_RECORD_SIZE: usize = 0x2000;
pub const PHOTO_RECORD_TAIL: usize = 0x118;
pub const PHOTO_QUAD_HALF_W: i32 = 0x98;
pub const PHOTO_QUAD_HALF_H: i32 = 0x68;
pub const PHOTO_ORTHO_LEFT: f32 = -21.0;
pub const PHOTO_ORTHO_RIGHT: f32 = 503.0;
pub const PHOTO_ORTHO_BOTTOM: f32 = -9.0;
pub const PHOTO_ORTHO_TOP: f32 = 650.0;
pub const PHOTO_ORTHO_NEAR: f32 = 0.0;
pub const PHOTO_ORTHO_FAR: f32 = 10.0;

pub const MODE_CAMERA: u8 = 0x0;
pub const MODE_BROWSE: u8 = 0x1;
pub const MODE_GET: u8 = 0x2;

pub const SUB_IDLE: u8 = 0x0;
pub const SUB_CONFIRM: u8 = 0x1;
pub const SUB_SHIFT_1_TO_0: u8 = 0x2;
pub const SUB_SHIFT_0_TO_2: u8 = 0x3;
pub const SUB_SHIFT_0_TO_1: u8 = 0x4;
pub const SUB_SHIFT_2_TO_0: u8 = 0x5;

pub const STATE_CAMERA_OPEN: u8 = 0x0;
pub const STATE_CAMERA_MOVE: u8 = 0x1;
pub const STATE_BROWSE_OPEN: u8 = 0x2;
pub const STATE_BROWSE_MOVE: u8 = 0x3;
pub const STATE_GET_OPEN: u8 = 0x4;
pub const STATE_GET_MOVE: u8 = 0x5;
pub const STATE_CLOSE: u8 = 0x6;
pub const STATE_CLOSED: u8 = 0x7;

pub const TEXTURE_I8: u8 = 1;
pub const TEXTURE_RGB565: u8 = 4;
pub const TEXTURE_CMPR: u8 = 6;

pub const MSG_REMAIN_EMPTY: u32 = 0xED9;
pub const MSG_REMAIN_ONE: u32 = 0xEDA;
pub const MSG_REMAIN_TWO: u32 = 0xEDB;
pub const MSG_REMAIN_THREE: u32 = 0xEDC;
pub const MSG_EXIST_THREE: u32 = 0xEDE;
pub const MSG_EXIST_TWO: u32 = 0xEDF;
pub const MSG_EXIST_ONE: u32 = 0xEE0;
pub const MSG_EXIST_EMPTY: u32 = 0xEE1;
pub const MSG_ERASE_ASK: u32 = 0xEE2;
pub const MSG_SHOW_ASK: u32 = 0xEE3;
pub const MSG_NO_PHOTO: u32 = 0xEE4;
pub const MSG_GET_PHOTO: u32 = 0xEE5;
pub const MSG_SAVED: u32 = 0xEDD;

pub const COLLECT_MODE_BROWSE: u8 = 0;
pub const COLLECT_MODE_DEMO: u8 = 1;
pub const COLLECT_MODE_PLAY: u8 = 2;
pub const COLLECT_MODE_SAVE: u8 = 3;
pub const COLLECT_MODE_OPTION: u8 = 4;
pub const COLLECT_MODE_TRIFORCE: u8 = 5;

pub const TRIGGER_NONE: u8 = 0;
pub const TRIGGER_LEFT: u8 = 1;
pub const TRIGGER_RIGHT: u8 = 2;

pub const NOTE_OPEN_LIMIT: i16 = 0x11;
pub const NOTE_CLOSE_START: i16 = 0x12;
pub const NOTE_CLOSE_LIMIT: i16 = 0x1C;
pub const NOTE_FADE: f32 = 0.19999999;
pub const NOTE_SWING: f32 = 45.0;
pub const NOTE_ROTATE_END: f32 = 123.0;
pub const NOTE_SLIDE_STEP: f32 = 7.0;
pub const NAME_HOLD_LIMIT: i16 = 0x82;
pub const NAME_HOLD_RESET: i16 = 0x0B;

pub const CORNER_PERIOD: i16 = 0xF0;
pub const CORNER_SEGMENT: i16 = 60;
pub const CORNER_PULSE_PEAK: f32 = 255.0;
pub const CORNER_PULSE_GREEN: f32 = 128.0;

pub const MODE_SCROLL_X: f32 = 10.0;
pub const MODE_SCROLL_FRAMES: i16 = 5;
pub const MODE_SWAP_FRAMES: i16 = 5;
pub const MODE_SKIP_FRAMES: i16 = 10;

pub const ITEM_SWORD: u8 = 0;
pub const ITEM_SHIELD: u8 = 1;
pub const ITEM_HOOD: u8 = 3;
pub const ITEM_BOTTLE: u8 = 4;

pub const COLLECT_ITEM_ROWS: u8 = 5;

pub const PAD_DEAD_ZONE: f32 = 0.0;
pub const SELECT_ARROW_ANGLE: f32 = 90.0;

pub const PARTICLE_RUPY_LIGHT: u32 = 0;

pub const SORT_EMPTY: u8 = 0xFF;

pub const ITEM_MESSAGE_NONE: u32 = 0;

pub const MAX_SHUTTER_FRAME: i16 = 10;

pub const ICON_SHEET_LEN: usize = 15;

pub const BLEND_FULL: [f32; 4] = [0.0, 1.0, 1.0, 1.0];
pub const BLEND_ALPHA: [f32; 4] = [1.0, 0.0, 1.0, 1.0];

pub const CURSOR_OFFSET_X: [f32; CURSOR_CORNER_COUNT] = [-4.0, 4.0, -4.0, 4.0];
pub const CURSOR_OFFSET_Y: [f32; CURSOR_CORNER_COUNT] = [-4.0, -4.0, 4.0, 4.0];

pub const ADJACENT_ITEM: [[u8; 8]; 21] = [
    [0x0E, 0x0F, 0x06, 0x06, 0x06, 0x01, 0xFF, 0xFF],
    [0xFF, 0x00, 0x06, 0x06, 0x06, 0x02, 0xFF, 0xFF],
    [0xFF, 0x01, 0x06, 0x06, 0x08, 0x03, 0xFF, 0xFF],
    [0xFF, 0x02, 0x06, 0x08, 0x08, 0x04, 0xFF, 0xFF],
    [0xFF, 0x03, 0x08, 0x08, 0x09, 0x05, 0xFF, 0xFF],
    [0xFF, 0x04, 0x08, 0x09, 0x09, 0x0A, 0xFF, 0xFF],
    [0x00, 0x10, 0x07, 0x07, 0x07, 0x08, 0x01, 0x01],
    [0x06, 0x11, 0xFF, 0xFF, 0xFF, 0x08, 0x06, 0x06],
    [0x06, 0x06, 0x08, 0xFF, 0x09, 0x09, 0x09, 0x03],
    [0x08, 0x08, 0x08, 0xFF, 0x0C, 0x0C, 0x0B, 0x05],
    [0x05, 0x05, 0x09, 0x0B, 0x0C, 0x0E, 0x0E, 0xFF],
    [0x05, 0x09, 0x0C, 0x0C, 0x0D, 0x0F, 0x10, 0x0A],
    [0x09, 0x09, 0x09, 0xFF, 0x0D, 0x0D, 0x0B, 0x0B],
    [0x0B, 0x0C, 0x0C, 0xFF, 0x11, 0x11, 0x10, 0x0B],
    [0x0A, 0x0A, 0x0A, 0x0F, 0x0F, 0x00, 0x00, 0xFF],
    [0x0A, 0x0A, 0x0B, 0x10, 0x10, 0x00, 0x00, 0x0E],
    [0x0A, 0x0B, 0x0D, 0x11, 0x11, 0x06, 0x06, 0x0F],
    [0x0B, 0x0D, 0x12, 0x12, 0x12, 0x06, 0x10, 0x10],
    [0x0D, 0x0D, 0xFF, 0xFF, 0xFF, 0x07, 0x11, 0x11],
    [0x0E, 0x0E, 0x0F, 0x06, 0x0A, 0x0A, 0x0A, 0xFF],
    [0x05, 0x09, 0x09, 0xFF, 0x11, 0x10, 0x0F, 0x0A],
];

pub const TRIFORCE_SLOT_COUNT: usize = 8;

pub const PARTIAL_TRANSPARENT: f32 = 0.5;
pub const PEARL_NOTE_X: f32 = 4.0;
pub const PEARL_NOTE_Y: f32 = 4.0;

pub const NAME_SHIFT: f32 = 50.0;
pub const MAIN_SHIFT: f32 = 100.0;
pub const TITLE_SHIFT: f32 = -100.0;
pub const NOTE_ROTATE: f32 = 200.0;

pub const WINDOW_BACK_COLOR: u32 = 0xFFB400FF;
pub const WINDOW_BACK_WHITE: u32 = 0xFFB400FF;
pub const WINDOW_BACK_BLACK: u32 = 0x50500000;
pub const WINDOW_DIM_COLOR: u32 = 0x505000FF;
pub const WINDOW_DIM_WHITE: u32 = 0x505050FF;
pub const WINDOW_DIM_BLACK: u32 = 0x50505000;

pub const TEXT_WHITE: u32 = 0xFFFF_FFFF;
pub const TEXT_BLUE: u32 = 0x0000_00FF;

pub const ARROW_COLOR_LOW: [u8; 3] = [0xFF, 0x50, 0x50];
pub const ARROW_COLOR_HIGH: [u8; 3] = [0xFF, 0x96, 0x96];

pub const EMITTER_COUNT: usize = 3;
pub const EMITTER_SOURCE_PANE: [usize; EMITTER_COUNT] = [6, 1, 7];

pub fn value_increase(max: i32, value: i32, mode: u8) -> f32 {
    if max <= 0 {
        return 1.0;
    }
    let value = value.clamp(0, max);
    let v = value as f32 / max as f32;
    match mode {
        0 => v * v,
        1 => v.sqrt(),
        3 => (v * 2.0 - 1.0) * 2.0 - 1.0,
        _ => v,
    }
}

pub fn deg2s(deg: f32) -> i16 {
    (deg * 32768.0 / 180.0) as i16
}

pub fn pack_color(c: [u8; 4]) -> u32 {
    ((c[0] as u32) << 24) | ((c[1] as u32) << 16) | ((c[2] as u32) << 8) | c[3] as u32
}

pub use crate::j2d::unpack_color;

pub fn item_message(item: u8) -> u32 {
    ITEM_MESSAGE
        .get(usize::from(item))
        .copied()
        .unwrap_or(0)
        .max(0) as u32
}

pub fn triforce_num(collect: &Collect) -> u8 {
    collect
        .triforce
        .count_ones()
        .min(u32::from(TRIFORCE_SLOT_COUNT as u32)) as u8
}

pub fn collect_map_num(charts: &Charts) -> usize {
    charts.total()
}

pub fn check_collect(collect: &Collect, idx: u8) -> bool {
    (0..COLLECT_ITEM_ROWS).any(|item| collect.test(idx, item))
}

pub fn melody_beat(song: u8) -> u8 {
    MELODY_PATTERN
        .get(usize::from(song))
        .map(|row| row[0])
        .unwrap_or(0)
}

pub fn melody_pattern(song: u8, step: usize, out: &mut i32) -> f32 {
    let Some(row) = MELODY_PATTERN.get(usize::from(song)) else {
        return 0.0;
    };
    if step >= usize::from(row[0]) {
        return 0.0;
    }
    let entry = row[step + 1];
    if entry == 0xFF {
        return 0.0;
    }
    *out = i32::from(entry);
    melody_g_frames(song)
}

pub fn melody_g_frames(song: u8) -> f32 {
    let scale = match melody_beat(song) {
        3 => BEAT_SCALE_3,
        4 => BEAT_SCALE_4,
        6 => BEAT_SCALE_6,
        _ => 1.0,
    };
    scale * 30.0
}

pub fn tact_direction(stick: i32, prev: i32, value: f32, angle: i16, hold: [bool; 4]) -> i32 {
    if value < TACT_STICK_MAX_VALUE {
        if stick == 0 {
            return prev;
        }
        return if hold[3] {
            3
        } else if hold[2] {
            2
        } else if hold[1] {
            4
        } else if hold[0] {
            1
        } else {
            0
        };
    }
    let angi = angle.unsigned_abs() as i32;
    if prev == 0 {
        if angi > 0x6000 {
            return 1;
        } else if angle >= 0x2000 {
            return 2;
        } else if angle <= -0x2000 {
            return 4;
        }
        return 3;
    }
    if angi > 0x7000 {
        return 1;
    } else if (0x3000..=0x5000).contains(&angle) {
        return 2;
    } else if (-0x5000..=-0x3000).contains(&angle) {
        return 4;
    } else if angi < 0x1000 {
        return 3;
    }
    prev
}

pub fn tact_note(beat: i32, pos: i32, variant: bool) -> u16 {
    let table: &[u8; 5] = match (beat, variant) {
        (3, _) => &C_POS_NOTE_3,
        (4, _) => &C_POS_NOTE_4,
        (6, true) => &C_POS_NOTE_62,
        (6, false) => &C_POS_NOTE_61,
        _ => return METRONOME_FALLBACK_NOTE,
    };
    if pos < 0 || pos > 4 {
        return NOTE_FALLBACK;
    }
    u16::from(table[pos as usize])
}

pub fn tact_pulse_sound(beat: i32) -> u32 {
    match beat {
        3 => SE_PULSE_3,
        4 => SE_PULSE_4,
        6 => SE_PULSE_6,
        _ => SE_PULSE_3,
    }
}

pub use crate::j2d::F2;

pub use crate::j2d::ContentsColor;

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
    pub blend_alpha: [f32; 4],
    pub white: u32,
    pub black: u32,
    pub rotate_x: f32,
    pub rotate_y: f32,
    pub rotation: u16,
    pub shift_x: f32,
    pub shift_y: f32,
    pub text: String,
    pub font_size_x: f32,
    pub font_size_y: f32,
    pub char_space: f32,
    pub line_space: f32,
    pub char_color: u32,
    pub grad_color: u32,
    pub contents_color: ContentsColor,
}

impl Default for Pane {
    fn default() -> Self {
        Self {
            tag: "",
            kind: Kind::Picture,
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
            blend_alpha: [0.0, 1.0, 1.0, 1.0],
            white: 0xFFFF_FFFF,
            black: 0x0000_0000,
            rotate_x: 0.0,
            rotate_y: 0.0,
            rotation: 0,
            shift_x: 0.0,
            shift_y: 0.0,
            text: String::new(),
            font_size_x: 0.0,
            font_size_y: 0.0,
            char_space: 0.0,
            line_space: 0.0,
            char_color: 0xFFFF_FFFF,
            grad_color: 0xFFFF_FFFF,
            contents_color: ContentsColor::default(),
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

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.pos_top_left.x = x;
        self.pos_top_left.y = y;
        self.cpos_move();
    }

    pub fn change_texture(&mut self, texture: &'static str) {
        self.texture = Some(texture);
        self.appended.clear();
    }

    pub fn remove(&mut self) {
        self.texture = None;
        self.appended.clear();
    }

    pub fn append(&mut self, texture: &'static str) {
        self.appended.push(texture);
    }

    pub fn blend_init(&mut self, texture: &'static str) {
        self.append(texture);
        self.blend = [0.0, 1.0, 1.0, 1.0];
        self.blend_alpha = [0.0, 1.0, 1.0, 1.0];
    }

    pub fn blend_draw(&mut self, texture: &'static str) {
        self.visible = true;
        self.remove();
        self.append(texture);
    }

    pub fn set_blend_ratio(&mut self, f1: f32, f2: f32, f3: f32, f4: f32) {
        self.blend = [f1, f2, f3, f4];
        self.blend_alpha = [f1, f2, f3, f4];
    }

    pub fn set_blend_color_ratio(&mut self, f1: f32, f2: f32, f3: f32, f4: f32) {
        self.blend = [f1, f2, f3, f4];
    }

    pub fn set_blend_alpha_ratio(&mut self, f1: f32, f2: f32, f3: f32, f4: f32) {
        self.blend_alpha = [f1, f2, f3, f4];
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

    pub fn set_contents_color(&mut self, color: u32) {
        self.contents_color.set_uniform(color);
    }

    pub fn get_contents_color(&self) -> ContentsColor {
        self.contents_color.get_color()
    }

    pub fn rotate(&mut self, x: f32, y: f32, angle: f32) {
        self.rotate_x = x;
        self.rotate_y = y;
        self.rotation = angle as i16 as u16;
    }

    pub fn rotate_deg(&mut self, x: f32, y: f32, deg: f32) {
        self.rotate(x, y, f32::from(deg2s(deg)));
    }

    pub fn get_rotate(&self) -> f32 {
        f32::from(self.rotation as i16)
    }

    pub fn set_string(&mut self, text: &str) {
        self.text = text.to_owned();
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

    pub fn parts_set(&mut self, layout: &j2d::ScreenPane) {
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
        self.texture = None;
        self.appended.clear();
        self.text = layout.text.clone();
        self.font_size_x = layout.font_size_x as f32;
        self.font_size_y = layout.font_size_y as f32;
        self.char_space = layout.char_space as f32;
        self.line_space = layout.line_space as f32;
        self.char_color = layout.char_color;
        self.grad_color = layout.grad_color;
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

    pub fn alpha_change(&mut self, alpha: f32) {
        self.init_alpha = (f32::from(self.init_alpha) * alpha) as u8;
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

    pub fn get_alpha(&self) -> u8 {
        self.now_alpha
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoteIcon {
    pub pane: usize,
    pub shadow: usize,
    pub pic: MsgIconPic,
    pub shd: MsgIconPic,
}

impl NoteIcon {
    pub fn new(pane: usize, shadow: usize) -> Self {
        Self {
            pane,
            shadow,
            pic: MsgIconPic::default(),
            shd: MsgIconPic::default(),
        }
    }

    pub fn push(&mut self, panes: &mut [Pane]) {
        let pic = self.pic;
        let shd = self.shd;
        if let Some(pane) = panes.get_mut(self.pane) {
            pane.visible = pic.visible;
            pane.texture = pic.tex;
            pane.appended = pic.tex.into_iter().collect::<Vec<&'static str>>();
            pane.white = pack_color(pic.white);
            pane.black = pack_color(pic.black);
            pane.blend = pic.blend_color;
            pane.blend_alpha = pic.blend_alpha;
            pane.now_alpha = pic.alpha;
            pane.rotation = pic.rot_z as i16 as u16;
        }
        if let Some(shadow) = panes.get_mut(self.shadow) {
            shadow.visible = shd.visible;
            shadow.texture = shd.tex;
            shadow.white = pack_color(shd.white);
            shadow.black = pack_color(shd.black);
            shadow.blend = shd.blend_color;
            shadow.blend_alpha = shd.blend_alpha;
            shadow.now_alpha = shd.alpha;
            shadow.rotation = shd.rot_z as i16 as u16;
        }
    }

    pub fn pull(&mut self, panes: &[Pane]) {
        if let Some(pane) = panes.get(self.pane) {
            self.pic.move_to(pane.pos_top_left.x, pane.pos_top_left.y);
            self.pic.resize(pane.size.x, pane.size.y);
            self.pic.rot_z = f32::from(pane.rotation as i16);
            self.pic.set_alpha(pane.now_alpha);
        }
        if let Some(shadow) = panes.get(self.shadow) {
            self.shd
                .move_to(shadow.pos_top_left.x, shadow.pos_top_left.y);
            self.shd.resize(shadow.size.x, shadow.size.y);
            self.shd.rot_z = f32::from(shadow.rotation as i16);
            self.shd.set_alpha(shadow.now_alpha);
        }
    }

    pub fn set_blend(&mut self, panes: &mut [Pane], blend: [f32; 4]) {
        for index in [self.pane, self.shadow] {
            if let Some(pane) = panes.get_mut(index) {
                pane.blend = blend;
                pane.blend_alpha = blend;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OutFontIcon {
    pub x: f32,
    pub y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub timer: i16,
    pub icon_no: u8,
    pub line: i16,
    pub texture: &'static str,
    pub blend: [f32; 4],
    pub rotation: u16,
    pub visible: bool,
}

#[derive(Clone, Debug)]
pub struct OutFont {
    pub box_pane: usize,
    pub pic: [usize; OUT_FONT_ICON_COUNT],
    pub ruby_pane: Option<usize>,
    pub ruby_msg: u32,
    pub cell: f32,
    pub char_space: f32,
    pub line_width: f32,
    pub line_space: f32,
    pub box_height: f32,
    pub left_up_x: f32,
    pub left_up_y: f32,
    pub icons: [OutFontIcon; OUT_FONT_ICON_COUNT],
    pub icon_count: u8,
    pub line: i16,
    pub advance: f32,
    pub text: String,
    pub ruby_text: String,
    pub alpha: u8,
    pub pics: [MsgIconPic; OUT_FONT_ICON_COUNT],
    pub ruby_flag: u8,
}

impl Default for OutFont {
    fn default() -> Self {
        Self {
            box_pane: P_WD,
            pic: [P_FT15, P_FT16, P_FT17],
            ruby_pane: None,
            ruby_msg: 0,
            cell: 0.0,
            char_space: 0.0,
            line_width: 0.0,
            line_space: 0.0,
            box_height: 0.0,
            left_up_x: 0.0,
            left_up_y: 0.0,
            icons: [OutFontIcon::default(); OUT_FONT_ICON_COUNT],
            icon_count: 0,
            line: 0,
            advance: 0.0,
            text: String::new(),
            ruby_text: String::new(),
            alpha: 0xFF,
            pics: [MsgIconPic::default(); OUT_FONT_ICON_COUNT],
            ruby_flag: 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
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

impl StickControl {
    pub fn new(
        first: i16,
        second: i16,
        step: i16,
        min_step: i16,
        max_value: f32,
        min_value: f32,
        repeat_angle: i16,
        half_dead: i16,
    ) -> Self {
        let mut stick = Self {
            max_value,
            min_value,
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
            repeat_angle,
            half_dead,
        };
        stick.init();
        stick
    }

    pub fn set_wait_parm(
        &mut self,
        first: i16,
        second: i16,
        step: i16,
        min_step: i16,
        max_value: f32,
        min_value: f32,
        repeat_angle: i16,
        half_dead: i16,
    ) {
        self.first_wait = second;
        self.second_wait = second;
        self.step = step;
        self.min_step = min_step;
        self.max_value = max_value;
        self.min_value = min_value;
        self.repeat_angle = repeat_angle;
        self.half_dead = half_dead;
        self.horizontal_wait = first;
        self.vertical_wait = first;
        self.repeat_count = min_step;
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
        if value != PAD_DEAD_ZONE {
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
                self.angle_base = if self.bits == 0 {
                    0
                } else if (angle & 0x1FFF) > 0x1000 {
                    self.repeat_angle
                } else {
                    -self.repeat_angle
                };
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

    fn step_wait(&mut self, wait: &mut i16, left: &mut i16) {
        if *left == 0 {
            *wait -= self.step;
            if *wait < self.min_step {
                *wait = self.min_step;
            }
        } else {
            *left -= 1;
        }
    }

    fn fire(&mut self, mask: u8, wait: &mut i16) -> bool {
        if *wait == 0 && self.bits & mask != 0 {
            *wait = self.horizontal_wait;
            let mut left = self.repeat_left;
            self.step_wait(wait, &mut left);
            self.repeat_left = left;
            true
        } else {
            false
        }
    }

    fn fire_vertical(&mut self, mask: u8, wait: &mut i16) -> bool {
        if *wait == 0 && self.bits & mask != 0 {
            *wait = self.vertical_wait;
            let mut left = self.repeat_left;
            self.step_wait(wait, &mut left);
            self.repeat_left = left;
            true
        } else {
            false
        }
    }

    pub fn check_left_trigger(&mut self) -> bool {
        let mut wait = self.horizontal_wait;
        let fired = self.fire(1, &mut wait);
        self.horizontal_wait = wait;
        fired
    }

    pub fn check_right_trigger(&mut self) -> bool {
        let mut wait = self.horizontal_wait;
        let fired = self.fire(2, &mut wait);
        self.horizontal_wait = wait;
        fired
    }

    pub fn check_up_trigger(&mut self) -> bool {
        let mut wait = self.vertical_wait;
        let fired = self.fire_vertical(4, &mut wait);
        self.vertical_wait = wait;
        fired
    }

    pub fn check_down_trigger(&mut self) -> bool {
        let mut wait = self.vertical_wait;
        let fired = self.fire_vertical(8, &mut wait);
        self.vertical_wait = wait;
        fired
    }
}

pub fn meter_msg_tune() -> MeterMsgHio {
    MeterMsgHio::default()
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Emitter {
    pub spawned: bool,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub alpha: f32,
    pub resource: u32,
    pub playing: bool,
    pub quit: bool,
}

#[derive(Clone, Debug)]
pub struct PaneQuad {
    pub tag: &'static str,
    pub kind: Kind,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub alpha: u8,
    pub texture: Option<&'static str>,
    pub appended: Vec<&'static str>,
    pub blend: [f32; 4],
    pub blend_alpha: [f32; 4],
    pub white: u32,
    pub black: u32,
    pub char_color: u32,
    pub grad_color: u32,
    pub rotate_x: f32,
    pub rotate_y: f32,
    pub rotation: u16,
    pub visible: bool,
}

#[derive(Clone, Debug)]
pub struct PaneText {
    pub tag: &'static str,
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub shift_x: f32,
    pub shift_y: f32,
    pub font_size: (f32, f32),
    pub char_space: f32,
    pub line_space: f32,
    pub alpha: u8,
    pub visible: bool,
}

#[derive(Clone, Debug, Default)]
pub struct PaneDraw {
    pub quads: Vec<PaneQuad>,
    pub texts: Vec<PaneText>,
    pub windows: Vec<(usize, ContentsColor)>,
}

impl PaneDraw {
    pub fn emit(&mut self, panes: &[Pane], order: &[usize]) {
        for (index, pane) in panes.iter().enumerate() {
            if !pane.found {
                continue;
            }
            let rank = order
                .iter()
                .position(|entry| *entry == index)
                .unwrap_or(usize::MAX);
            match pane.kind {
                Kind::TextBox => self.texts.push(PaneText {
                    tag: pane.tag,
                    text: pane.text.clone(),
                    x: pane.pos_top_left.x + pane.shift_x,
                    y: pane.pos_top_left.y + pane.shift_y,
                    w: pane.size.x,
                    h: pane.size.y,
                    shift_x: pane.shift_x,
                    shift_y: pane.shift_y,
                    font_size: (pane.font_size_x, pane.font_size_y),
                    char_space: pane.char_space,
                    line_space: pane.line_space,
                    alpha: pane.now_alpha,
                    visible: pane.visible,
                }),
                _ => self.quads.push(PaneQuad {
                    tag: pane.tag,
                    kind: pane.kind,
                    x: pane.pos_top_left.x,
                    y: pane.pos_top_left.y,
                    w: pane.size.x,
                    h: pane.size.y,
                    alpha: pane.now_alpha,
                    texture: pane.texture,
                    appended: pane.appended.clone(),
                    blend: pane.blend,
                    blend_alpha: pane.blend_alpha,
                    white: pane.white,
                    black: pane.black,
                    char_color: pane.char_color,
                    grad_color: pane.grad_color,
                    rotate_x: pane.rotate_x,
                    rotate_y: pane.rotate_y,
                    rotation: pane.rotation,
                    visible: pane.visible && pane.kind != Kind::Window,
                }),
            }
            if pane.kind == Kind::Window {
                self.windows.push((rank, pane.contents_color));
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PadInput {
    pub a: bool,
    pub b: bool,
    pub z: bool,
    pub x: bool,
    pub y: bool,
    pub l: bool,
    pub r: bool,
    pub start: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub lock_l: bool,
    pub lock_r: bool,
    pub stick_value: f32,
    pub stick_angle: i16,
    pub sub_stick_value: f32,
    pub sub_stick_angle: i16,
}

impl PadInput {
    pub fn hold(&self) -> [bool; 4] {
        [self.up, self.left, self.right, self.down]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CollectTune {
    pub input_hold: i16,
    pub warp_frames: i16,
    pub hold_frames: i16,
    pub name_frames: i16,
    pub pulse_period: i16,
    pub triforce_period: i16,
    pub corner_pulse: i16,
    pub triforce_alpha: u8,
    pub triforce_fade_alpha: u8,
    pub scale: f32,
    pub pulse_color: [u8; 4],
}

impl Default for CollectTune {
    fn default() -> Self {
        Self {
            input_hold: 13,
            warp_frames: 40,
            hold_frames: 10,
            name_frames: 120,
            pulse_period: 100,
            triforce_period: 250,
            corner_pulse: 20,
            triforce_alpha: 130,
            triforce_fade_alpha: 70,
            scale: 4.0,
            pulse_color: [0xFF, 0xAA, 0x00, 0x00],
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CollectOutput {
    pub sounds: Vec<u32>,
    pub do_status_force: Option<u8>,
    pub do_status: Option<u8>,
    pub a_status_force: Option<u8>,
    pub emitters: [Emitter; EMITTER_COUNT],
    pub on_collect: Vec<(u8, u8)>,
    pub off_collect: Vec<(u8, u8)>,
    pub melody_play: Option<u8>,
    pub melody_stop: bool,
    pub tact_note: Option<u16>,
    pub tact_beats: (f32, f32, f32),
    pub sub_screen: Option<SubScreen>,
    pub emitters_quit: bool,
}

impl CollectOutput {
    pub fn sound(&mut self, id: u32) {
        self.sounds.push(id);
    }

    pub fn clear_do(&mut self) {
        self.do_status = Some(ACT_BLANK);
        self.do_status_force = Some(ACT_BLANK);
    }
}

#[derive(Clone, Debug, Default)]
pub struct DrawOrder {
    pub order: Vec<usize>,
}

impl DrawOrder {
    pub fn new(count: usize) -> Self {
        Self {
            order: (0..count).collect(),
        }
    }

    pub fn insert_child(&mut self, parent: usize, child: usize) {
        self.order.retain(|entry| *entry != child);
        let at = self
            .order
            .iter()
            .position(|entry| *entry == parent)
            .map(|index| index + 1)
            .unwrap_or(self.order.len());
        self.order.insert(at.min(self.order.len()), child);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CollectInput {
    pub collect: Collect,
    pub story: StoryBits,
    pub charts: Charts,
    pub equip: Equipped,
    pub health: Health,
    pub opt_ruby: bool,
}

impl CollectInput {
    pub fn map_open(&self) -> bool {
        self.story.test(EVENT_MAP_OPEN) || collect_map_num(&self.charts) != 0
    }

    pub fn life(&self) -> u8 {
        self.health.max
    }

    pub fn equip_item(&self, button: u8) -> u8 {
        self.equip.item(usize::from(button))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SubScreen {
    #[default]
    None,
    Option,
    Save,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SubPhase {
    #[default]
    Idle,
    Open,
    Move,
    Close,
}

#[derive(Clone, Debug)]
pub struct CollectScreen {
    pub panes: [Pane; PANE_COUNT],
    pub note: [NoteIcon; NOTE_ICON_COUNT],
    pub stick: StickControl,
    pub out_font: OutFont,
    pub name_buf: [String; NAME_BUFFER_COUNT],
    pub note_buf: [String; NAME_BUFFER_COUNT],
    pub dummy_buf: [String; NAME_BUFFER_COUNT],
    pub contents_color: ContentsColor,
    pub corner_offset_x: [f32; CURSOR_CORNER_COUNT],
    pub corner_offset_y: [f32; CURSOR_CORNER_COUNT],
    pub tri_offset_x: f32,
    pub tri_offset_y: f32,
    pub note_proc: MsgData,
    pub msg_entry: MesgEntry,
    pub order: DrawOrder,
    pub emitters: [Emitter; EMITTER_COUNT],
    pub tac_direction: i32,
    pub icon_slot: i16,
    pub timer: i16,
    pub input_timer: i16,
    pub corner_timer: i16,
    pub pulse_timer: i16,
    pub fade_flag: u8,
    pub select_flag: u8,
    pub trigger_info: u8,
    pub now_item: u8,
    pub collect_mode: u8,
    pub select_index: u8,
    pub last_select: u8,
    pub reserve: u8,
    pub triforce_count: u8,
    pub beat: i32,
    pub beat_variant: u8,
    pub arrow_timer: i16,
    pub map_open: bool,
    pub sub_screen: SubScreen,
    pub sub_phase: SubPhase,
}

impl CollectScreen {
    pub fn new(screen: &Screen) -> Self {
        let mut pane = Self {
            panes: std::array::from_fn(|index| Pane::new(COLLECT_TAG_TABLE[index])),
            note: std::array::from_fn(|index| NoteIcon::new(P_FT0 + index, P_FD0 + index)),
            stick: StickControl::new(
                TACT_STICK_WAIT_FIRST,
                TACT_STICK_WAIT_SECOND,
                TACT_STICK_STEP,
                TACT_STICK_MIN_STEP,
                TACT_STICK_MAX_VALUE,
                TACT_STICK_MIN_VALUE,
                TACT_STICK_REPEAT_ANGLE,
                TACT_STICK_HALF_DEAD,
            ),
            out_font: OutFont::default(),
            name_buf: [String::new(), String::new()],
            note_buf: [String::new(), String::new()],
            dummy_buf: [String::new(), String::new()],
            contents_color: ContentsColor::default(),
            corner_offset_x: CURSOR_OFFSET_X,
            corner_offset_y: CURSOR_OFFSET_Y,
            tri_offset_x: 0.0,
            tri_offset_y: 0.0,
            note_proc: MsgData::new_data_proc(),
            msg_entry: MesgEntry::default(),
            order: DrawOrder::new(PANE_COUNT),
            emitters: [Emitter::default(); EMITTER_COUNT],
            tac_direction: 0,
            icon_slot: 0,
            timer: 0,
            input_timer: 0,
            corner_timer: 0,
            pulse_timer: 0,
            fade_flag: 0,
            select_flag: 0,
            trigger_info: TRIGGER_NONE,
            now_item: 0,
            collect_mode: COLLECT_MODE_BROWSE,
            select_index: 0,
            last_select: 0,
            reserve: 0,
            triforce_count: 0,
            beat: BEAT_DEFAULT,
            beat_variant: 0,
            arrow_timer: 0,
            map_open: false,
            sub_screen: SubScreen::None,
            sub_phase: SubPhase::Idle,
        };
        pane.bind(screen);
        pane
    }

    pub fn pane(&self, index: usize) -> &Pane {
        &self.panes[index]
    }

    pub fn pane_mut(&mut self, index: usize) -> &mut Pane {
        &mut self.panes[index]
    }

    pub fn set_now_item(&mut self, item: u8) {
        self.now_item = item;
    }

    pub fn get_now_item(&self) -> u8 {
        self.now_item
    }

    pub fn get_collect_mode(&self) -> u8 {
        self.collect_mode
    }

    pub fn set_collect_mode(&mut self, mode: u8) {
        self.collect_mode = mode;
    }

    pub fn set_timer(&mut self, value: i16) {
        self.timer = value;
    }

    pub fn set_trigger_info(&mut self, info: u8) {
        self.trigger_info = info;
    }

    pub fn alpha_change(&mut self, index: usize, alpha: f32) {
        self.panes[index].alpha_change(alpha);
    }

    pub fn bind(&mut self, screen: &Screen) {
        for index in 0..PANE_COUNT {
            if let Some(layout) = screen.find(COLLECT_TAG_TABLE[index]) {
                self.panes[index].parts_set(layout);
            }
        }
    }

    pub fn screen_set(&mut self, input: &CollectInput) {
        let mut anchor = self.panes[P_NO11].get_rotate() as i32;
        if anchor > 180 {
            anchor -= 360;
        }
        self.panes[P_NO11].user_area = anchor as i16;

        for index in 0..NOTE_ICON_COUNT {
            self.panes[P_FT0 + index].user_area = -1;
        }

        for index in [P_STR0, P_ST00] {
            self.panes[index].set_white(TEXT_WHITE);
            self.panes[index].set_char_color(TEXT_BLUE);
            self.panes[index].set_grad_color(TEXT_BLUE);
        }

        self.out_font.set_pane(
            &mut self.panes,
            P_WD,
            [P_FT15, P_FT16, P_FT17],
            OUT_FONT_TEX,
            OUT_FONT_BLEND_TEX,
        );

        self.panes[P_TRIM].show();
        self.panes[P_TRIL].show();

        if input.story.test(EVENT_PLACED_DIN_PEARL) {
            self.panes[P_SIT + 1].alpha_change(PARTIAL_TRANSPARENT);
            self.panes[P_SIK + 1].alpha_change(PARTIAL_TRANSPARENT);
        }
        if input.story.test(EVENT_PLACED_FARORE_PEARL) {
            self.panes[P_SIT + 2].alpha_change(PARTIAL_TRANSPARENT);
            self.panes[P_SIK + 2].alpha_change(PARTIAL_TRANSPARENT);
        }
        if input.story.test(EVENT_PLACED_NAYRU_PEARL) {
            self.panes[P_SIT].alpha_change(PARTIAL_TRANSPARENT);
            self.panes[P_SIK].alpha_change(PARTIAL_TRANSPARENT);
        }

        for index in 0..SONG_COUNT {
            self.panes[P_TIT + index].change_texture(TACT_TEX);
            self.panes[P_TIK + index].change_texture(TACT_TEX);
            self.panes[P_WN + index].pos_center_orig.x -= PEARL_NOTE_X;
            self.panes[P_WN + index].pos_center_orig.y -= PEARL_NOTE_Y;
            self.panes[P_WN + index].pane_trans(0.0, 0.0);
        }

        self.panes[P_CMAP].change_texture(MAP_TEX);

        for index in 0..TRIFORCE_SLOT_COUNT {
            self.panes[P_TRI0 + index].change_texture(TRIFORCE_TEX[index]);
        }

        self.panes[P_TRI0 + 4].user_area = -1;
        self.tri_offset_x =
            self.panes[P_TRI0 + 4].pos_center_orig.x - self.panes[P_TRIB].pos_center_orig.x;
        self.tri_offset_y =
            self.panes[P_TRI0 + 4].pos_center_orig.y - self.panes[P_TRIB].pos_center_orig.y;

        for index in 0..3 {
            self.panes[P_SIT + index].change_texture(SYMBOL_TEX[index]);
            self.panes[P_SIK + index].change_texture(SYMBOL_TEX[index]);
        }

        for index in 0..5 {
            self.panes[P_WIT + index].change_texture(WEAPON_TEX[index]);
            self.panes[P_WIK + index].change_texture(WEAPON_TEX[index]);
        }

        let maps = collect_map_num(&input.charts);
        self.panes[P_NUM1].change_texture(RUPEE_NUM_TEX[(maps / 10).min(9)]);
        self.panes[P_NUM2].change_texture(RUPEE_NUM_TEX[(maps % 10).min(9)]);
    }

    pub fn create(&mut self, input: &CollectInput, tune: &CollectTune) {
        self.stick.set_wait_parm(
            TACT_STICK_WAIT_FIRST,
            TACT_STICK_WAIT_SECOND,
            TACT_STICK_STEP,
            TACT_STICK_MIN_STEP,
            TACT_STICK_MAX_VALUE,
            TACT_STICK_MIN_VALUE,
            TACT_STICK_REPEAT_ANGLE,
            TACT_STICK_HALF_DEAD,
        );
        self.out_font.ruby_flag = 1;
        self.screen_set(input);
        self.initialize(input, tune);

        for index in 0..CURSOR_CORNER_COUNT {
            self.panes[P_CAR0 + index].append(CURSOR_TEX);
            self.corner_offset_x[index] =
                self.panes[P_CAR0 + index].pos_center_orig.x - self.panes[P_TIT].pos_center_orig.x;
            self.corner_offset_y[index] =
                self.panes[P_CAR0 + index].pos_center_orig.y - self.panes[P_TIT].pos_center_orig.y;
        }

        for index in 0..NOTE_ICON_COUNT {
            self.panes[P_FT0 + index].change_texture(OUT_FONT_TEX);
            self.panes[P_FT0 + index].blend_init(OUT_FONT_BLEND_TEX);
        }

        self.cursor_anime(input.map_open());
    }

    pub fn create_triforce(&mut self, input: &CollectInput, tune: &CollectTune) {
        self.stick.set_wait_parm(
            TACT_STICK_WAIT_FIRST,
            TACT_STICK_WAIT_SECOND,
            TACT_STICK_STEP,
            TACT_STICK_MIN_STEP,
            TACT_STICK_MAX_VALUE,
            TACT_STICK_MIN_VALUE,
            TACT_STICK_REPEAT_ANGLE,
            TACT_STICK_HALF_DEAD,
        );
        self.screen_set(input);

        self.timer = 0;
        self.reserve = 0;
        self.tac_direction = 0;
        self.panes[P_WD].user_area = 10;
        self.input_timer = 0;
        self.select_flag = 0;
        self.trigger_info = 0;
        self.collect_mode = COLLECT_MODE_TRIFORCE;
        self.beat_variant = 0;

        self.tact_guide_hide();
        self.tact_base_show();
        self.note_init();
        self.out_font_init();
        self.item_bit_check(input, tune);

        self.panes[P_MSK0].init_alpha = 0x82;
        self.corner_timer = 0;
        self.pulse_timer = 0;

        self.contents_color = self.panes[P_CORNER0 + 1].get_contents_color();

        for index in 0..CURSOR_CORNER_COUNT {
            self.panes[P_CAR0 + index].hide();
        }

        for index in 0..NOTE_ICON_COUNT {
            self.panes[P_FT0 + index].change_texture(OUT_FONT_TEX);
            self.panes[P_FT0 + index].blend_init(OUT_FONT_BLEND_TEX);
        }
    }

    pub fn initialize(&mut self, input: &CollectInput, tune: &CollectTune) {
        self.timer = 0;
        self.reserve = 0;
        self.tac_direction = 0;
        self.panes[P_WD].user_area = 10;
        self.panes[P_CAR0].user_area = 0;
        self.panes[P_CAR0 + 1].user_area = 0;
        self.input_timer = 0;
        for index in 0..SONG_COUNT {
            self.panes[P_TIT + index].user_area = 0;
        }
        self.panes[P_TIK].user_area = 0;
        for index in 0..SONG_COUNT {
            self.panes[P_HMB + index].user_area = 0;
        }
        self.panes[P_TIT].user_area = 0;
        self.select_flag = 0;
        self.trigger_info = 0;
        self.collect_mode = COLLECT_MODE_BROWSE;
        self.beat_variant = 0;
        self.beat = BEAT_DEFAULT;

        self.tact_guide_hide();

        if self.now_item <= 5 && input.collect.is_tact(self.now_item) {
            for index in 0..SONG_COUNT {
                self.panes[P_HMB + index].user_area = 1;
            }
            self.tact_guide_show(self.now_item, false);
        } else {
            self.tact_base_show();
        }

        if self.now_item <= 0x0D || self.now_item >= 0x13 {
            self.collect_priority();
        } else {
            self.wepon_priority();
        }

        self.note_init();
        self.out_font_init();
        self.item_bit_check(input, tune);

        self.panes[P_MSK0].init_alpha = 0x82;
        self.corner_timer = 0;
        self.pulse_timer = 0;

        self.contents_color = self.panes[P_CORNER0 + 1].get_contents_color();
    }

    pub fn cursor_anime(&mut self, map_open: bool) {
        self.map_open = map_open;
        let item = self.now_item;
        let cursor = P_CAR0;
        match item {
            0..=5 => {
                let anchor = P_TIT + usize::from(item);
                for index in 0..CURSOR_CORNER_COUNT {
                    self.panes[cursor + index].pos_center_orig.x =
                        self.panes[anchor].pos_center.x + self.corner_offset_x[index];
                    self.panes[cursor + index].pos_center_orig.y =
                        self.panes[anchor].pos_center.y + self.corner_offset_y[index];
                }
            }
            6 => self.cursor_on_box(P_WDOP, cursor),
            7 => self.cursor_on_box(P_WDSV, cursor),
            8 => {
                let anchor = P_HTP + 2;
                for index in 0..CURSOR_CORNER_COUNT {
                    self.panes[cursor + index].pos_center_orig.x =
                        self.panes[anchor].pos_top_left.x + self.corner_offset_x[index];
                    self.panes[cursor + index].pos_center_orig.y =
                        self.panes[anchor].pos_top_left.y + self.corner_offset_y[index];
                }
            }
            9 => {
                if map_open {
                    let center = self.panes[P_CMAP].pos_center;
                    for index in 0..CURSOR_CORNER_COUNT {
                        self.panes[cursor + index].pos_center_orig.x =
                            center.x + self.corner_offset_x[index];
                        self.panes[cursor + index].pos_center_orig.y =
                            center.y + self.corner_offset_y[index];
                    }
                } else {
                    self.cursor_on_box(P_CORNER0, cursor);
                }
            }
            10 => {
                let tri = P_TRI0 + 4;
                let frame = P_TRIB;
                let half_x = self.panes[frame].size.x / 2.0;
                let half_y = self.panes[frame].size.y / 2.0;
                let center = self.panes[tri].pos_center;
                let xs = [-half_x, half_x, -half_x, half_x];
                let ys = [half_y, half_y, -half_y, -half_y];
                for index in 0..CURSOR_CORNER_COUNT {
                    self.panes[cursor + index].pos_center_orig.x = center.x + xs[index];
                    self.panes[cursor + index].pos_center_orig.y = center.y + ys[index];
                }
            }
            11..=13 => {
                let anchor = P_SIT + (usize::from(item) - 11);
                for index in 0..CURSOR_CORNER_COUNT {
                    self.panes[cursor + index].pos_center_orig.x =
                        self.panes[anchor].pos_center.x + self.corner_offset_x[index];
                    self.panes[cursor + index].pos_center_orig.y =
                        self.panes[anchor].pos_center.y + self.corner_offset_y[index];
                }
            }
            14..=18 => {
                let anchor = P_BS + (usize::from(item) - 14);
                for index in 0..CURSOR_CORNER_COUNT {
                    self.panes[cursor + index].pos_center_orig.x =
                        self.panes[anchor].pos_center.x + self.corner_offset_x[index];
                    self.panes[cursor + index].pos_center_orig.y =
                        self.panes[anchor].pos_center.y + self.corner_offset_y[index];
                }
            }
            19 => {
                let x1 =
                    (self.panes[P_TIT + 2].pos_center.x + self.panes[P_TIT + 3].pos_center.x) / 2.0;
                let y1 = self.panes[P_TIT].pos_center.y;
                let x2 = ((self.panes[P_TIT + 5].pos_top_left.x + self.panes[P_TIT + 5].size.x)
                    - self.panes[P_TIT].pos_top_left.x)
                    / 2.0;
                let y2 = self.panes[P_TIT].size.y / 2.0;
                let xs = [-x2, x2, -x2, x2];
                let ys = [y2, y2, -y2, -y2];
                for index in 0..CURSOR_CORNER_COUNT {
                    self.panes[cursor + index].pos_center_orig.x = x1 + xs[index];
                    self.panes[cursor + index].pos_center_orig.y = y1 + ys[index];
                }
            }
            20 => self.cursor_on_box(P_CI + 8, cursor),
            _ => {}
        }

        let mut x_trans = 0i16;
        let mut y_trans = 0i16;
        if self.panes[cursor].user_area < 10 {
            for index in 0..CURSOR_CORNER_COUNT {
                self.panes[cursor + index].set_blend_color_ratio(1.0, 0.0, 1.0, 1.0);
                self.panes[cursor + index].set_blend_alpha_ratio(1.0, 0.0, 1.0, 1.0);
            }
            x_trans = 7;
            y_trans = 7;
        } else if self.panes[cursor].user_area < 20 {
            for index in 0..CURSOR_CORNER_COUNT {
                self.panes[cursor + index].set_blend_color_ratio(0.0, 1.0, 1.0, 1.0);
                self.panes[cursor + index].set_blend_alpha_ratio(0.0, 1.0, 1.0, 1.0);
            }
            x_trans = 14;
            y_trans = 14;
        }

        self.panes[cursor].pane_trans(-f32::from(x_trans), f32::from(y_trans));
        self.panes[cursor + 1].pane_trans(f32::from(x_trans), f32::from(y_trans));
        self.panes[cursor + 2].pane_trans(-f32::from(x_trans), -f32::from(y_trans));
        self.panes[cursor + 3].pane_trans(f32::from(x_trans), -f32::from(y_trans));

        self.panes[cursor].user_area += 1;
        if self.panes[cursor].user_area >= 20 {
            self.panes[cursor].user_area = 0;
        }
    }

    fn cursor_on_box(&mut self, box_index: usize, cursor: usize) {
        let center = self.panes[box_index].pos_center;
        let half_x = self.panes[box_index].size.x / 2.0;
        let half_y = self.panes[box_index].size.y / 2.0;
        let xs = [-half_x, half_x, -half_x, half_x];
        let ys = [half_y, half_y, -half_y, -half_y];
        for index in 0..CURSOR_CORNER_COUNT {
            self.panes[cursor + index].pos_center_orig.x = center.x + xs[index];
            self.panes[cursor + index].pos_center_orig.y = center.y + ys[index];
        }
    }

    pub fn stick_direction(&self, bits: u8) -> i32 {
        match bits {
            5 => 0,
            1 => 1,
            9 => 2,
            8 => 3,
            10 => 4,
            2 => 5,
            6 => 6,
            4 => 7,
            _ => -1,
        }
    }

    pub fn cursor_main_move(
        &mut self,
        input: &CollectInput,
        pad: &PadInput,
        out: &mut CollectOutput,
    ) -> bool {
        let mut direction: i32 = -1;
        let previous_item = self.now_item;
        let trigger = self.stick.check_trigger(pad.stick_value, pad.stick_angle);
        let mut moved = false;

        if self.stick.check_right_trigger() || self.stick.check_left_trigger() {
            moved = true;
        }
        if self.stick.check_up_trigger() || self.stick.check_down_trigger() {
            moved = true;
        }
        if moved {
            direction = self.stick_direction(trigger);
        }

        let mut next_item = ITEM_NONE;
        if direction != -1 {
            let old_item = self.now_item;
            let dir = direction as usize;
            next_item = ADJACENT_ITEM
                .get(usize::from(old_item))
                .map(|row| row[dir])
                .unwrap_or(ITEM_NONE);
            next_item = self.resolve_tact(&input.collect, next_item, dir);
            next_item = self.resolve_symbol(&input.collect, next_item, dir);
        }

        if next_item != ITEM_NONE {
            self.now_item = next_item;
        }

        if self.now_item != previous_item {
            let outside = |value: u8| value <= 0x0D || value >= 0x13;
            if (outside(self.now_item) && !outside(previous_item))
                || (!outside(self.now_item) && outside(previous_item))
            {
                self.panes[P_CAR0 + 1].user_area = 10;
            }

            self.tact_guide_hide();

            if self.now_item <= 5 && input.collect.is_tact(self.now_item) {
                for index in 0..SONG_COUNT {
                    self.panes[P_HMB + index].user_area = 1;
                }
                self.tact_guide_show(self.now_item, true);
            } else {
                self.tact_base_show();
            }

            self.panes[P_WD].user_area = 0;
            out.sound(SE_CURSOR);
        }

        if self.now_item <= 7 {
            if self.collect_item_get_check(input, self.now_item) {
                out.do_status_force = Some(ACT_CHOOSE);
            } else {
                out.clear_do();
            }
        } else if self.collect_item_get_check(input, self.now_item) {
            out.do_status_force = Some(ACT_INFO);
        } else {
            out.clear_do();
        }

        if self.panes[P_CAR0 + 1].user_area != 0 {
            let mut trans = MODE_SCROLL_X;
            self.panes[P_CAR0 + 1].user_area -= 1;
            let user_area = self.panes[P_CAR0 + 1].user_area;
            let amount = MODE_SCROLL_FRAMES - (MODE_SCROLL_FRAMES - user_area).abs();
            trans *= value_increase(i32::from(MODE_SCROLL_FRAMES), i32::from(amount), 0);
            self.main_trans(input, -trans, 0.0);
            self.sub_trans(trans, 0.0);
            if self.panes[P_CAR0 + 1].user_area == MODE_SWAP_FRAMES {
                if self.now_item <= 0x0D || self.now_item >= 0x13 {
                    self.collect_priority();
                } else {
                    self.wepon_priority();
                }
            }
        }

        moved
    }

    fn resolve_tact(&self, collect: &Collect, next_item: u8, direction: usize) -> u8 {
        let old_item = self.now_item;
        let horiz = direction == 0 || direction == 1;
        let vert = direction == 5 || direction == 6;
        if next_item > 5 {
            return next_item;
        }
        let all: &[u8] = match next_item {
            0 => &[0, 1, 2, 3, 4, 5],
            1 => &[1, 0, 2, 3, 4, 5],
            2 => &[2, 1, 3, 0, 4, 5],
            3 => &[3, 2, 4, 1, 5, 0],
            4 => &[4, 3, 5, 2, 1, 0],
            _ => &[5, 4, 3, 2, 1, 0],
        };
        let side: &[u8] = match next_item {
            0 => &[],
            1 => &[0],
            2 => &[1, 0],
            3 => &[2, 1, 0],
            4 => &[3, 2, 1, 0],
            _ => &[4, 3, 2, 1, 0],
        };
        let vertical: &[u8] = match next_item {
            0 => &[1, 2, 3, 4, 5],
            1 => &[2, 3, 4, 5],
            2 => &[3, 4, 5],
            3 => &[4, 5],
            4 => &[5],
            _ => &[],
        };

        if old_item > 5 {
            return all
                .iter()
                .find(|entry| collect.is_tact(**entry))
                .copied()
                .unwrap_or(0x13);
        }
        if old_item != next_item && collect.is_tact(next_item) {
            return next_item;
        }
        if horiz {
            return side
                .iter()
                .find(|entry| collect.is_tact(**entry))
                .copied()
                .unwrap_or(0x0E);
        }
        if vert {
            return vertical
                .iter()
                .find(|entry| collect.is_tact(**entry))
                .copied()
                .unwrap_or(10);
        }
        next_item
    }

    fn resolve_symbol(&self, collect: &Collect, next_item: u8, direction: usize) -> u8 {
        let old_item = self.now_item;
        let right = direction == 1;
        let vert = direction == 5 || direction == 6;
        let left = direction == 0 || direction == 7;
        match next_item {
            0x0B => {
                if old_item != 0x0B && old_item != 0x0C && old_item != 0x0D {
                    let order = [0x0B, 0x0C, 0x0D];
                    return order
                        .iter()
                        .find(|entry| collect.is_symbol(**entry - 0x0B))
                        .copied()
                        .unwrap_or(0x14);
                }
                if old_item == next_item || !collect.is_symbol(next_item - 0x0B) {
                    if right {
                        return 9;
                    } else if vert {
                        return 0x11;
                    } else if left {
                        return 10;
                    }
                }
                next_item
            }
            0x0C => {
                if old_item != 0x0B && old_item != 0x0C && old_item != 0x0D {
                    let order = [0x0C, 0x0B, 0x0D];
                    return order
                        .iter()
                        .find(|entry| collect.is_symbol(**entry - 0x0B))
                        .copied()
                        .unwrap_or(0x14);
                }
                if old_item == next_item || !collect.is_symbol(next_item - 0x0B) {
                    if right {
                        return 9;
                    } else if vert {
                        return if collect.is_symbol(0) { 0x0B } else { 0x11 };
                    } else if left {
                        return if collect.is_symbol(0) { 0x0B } else { 10 };
                    }
                }
                next_item
            }
            0x0D => {
                if old_item != 0x0B && old_item != 0x0C && old_item != 0x0D {
                    let order = [0x0D, 0x0C, 0x0B];
                    return order
                        .iter()
                        .find(|entry| collect.is_symbol(**entry - 0x0B))
                        .copied()
                        .unwrap_or(0x14);
                }
                if old_item == next_item || !collect.is_symbol(next_item - 0x0B) {
                    if right {
                        return if collect.is_symbol(1) { 0x0C } else { 9 };
                    } else if vert {
                        return 0x11;
                    } else if left {
                        return if collect.is_symbol(0) { 0x0B } else { 9 };
                    }
                }
                next_item
            }
            _ => next_item,
        }
    }

    pub fn note_check(&self) -> bool {
        self.panes[P_NK00].user_area == 1
    }

    pub fn note_init(&mut self) {
        self.panes[P_NK00].user_area = 0;
        self.panes[P_NT00].user_area = 0;
        let size = self.panes[P_NO11].size_orig;
        let angle = f32::from(self.panes[P_NO11].user_area);
        self.panes[P_NO11].rotate(size.x / 2.0, size.y / 2.0, angle);
        self.panes[P_NO11].pane_trans(0.0, 0.0);
    }

    pub fn note_appear(
        &mut self,
        input: &mut CollectInput,
        pad: &PadInput,
        out: &mut CollectOutput,
    ) {
        if self.panes[P_NK00].user_area != 1 {
            return;
        }
        let state = self.panes[P_NT00].user_area;
        if state <= NOTE_OPEN_LIMIT {
            self.note_open(out);
            return;
        }
        if state > NOTE_CLOSE_START {
            self.note_close();
            return;
        }
        self.panes[P_MSK0].now_alpha = 0x82;
        self.panes[P_MSK0].init_alpha = 0x82;

        if self.select_flag != 0 {
            let slot = self.icon_slot as usize;
            let x = self.panes[P_FT0 + slot].pos_center.x;
            let y = self.panes[P_FT0 + slot].pos_center.y;
            let width = self.panes[P_FT0 + slot].size.x;
            let height = self.panes[P_FT0 + slot].size.y;
            let left = self.panes[P_FT0 + slot].pos_top_left.x;
            let picked = self.note_proc.select_check_yoko(
                width,
                height,
                MSG_FONT_SIZE as i16,
                x as i32,
                y as i32,
                (left - x) as i32,
                0.0,
            );
            self.select_index = picked;
            if self.select_index != self.last_select {
                out.sound(SE_TALK_CURSOR);
                self.last_select = self.select_index;
            }
        }

        if pad.a {
            if self.now_item == 0x12 {
                if self.select_index == 0 {
                    input.collect.set(ITEM_BOTTLE, 1);
                    out.on_collect.push((ITEM_BOTTLE, 1));
                    out.sound(SE_EQUIP_ON);
                } else {
                    input.collect.clear(ITEM_BOTTLE, 1);
                    out.off_collect.push((ITEM_BOTTLE, 1));
                    out.sound(SE_EQUIP_OFF);
                }
            }
            self.panes[P_NT00].user_area += 1;
            out.sound(SE_EXP_OUT);
        } else if pad.b {
            self.panes[P_NT00].user_area += 1;
            if self.now_item == 0x12 {
                out.sound(SE_CANCEL);
            }
            out.sound(SE_EXP_OUT);
        }
    }

    pub fn note_open(&mut self, out: &mut CollectOutput) {
        let dx = CENTER_X - self.panes[P_NO11].pos_center_orig.x;
        let dy = CENTER_Y - self.panes[P_NO11].pos_center_orig.y;
        let state = self.panes[P_NT00].user_area;
        let fade = 1.0
            - value_increase(
                i32::from(NOTE_OPEN_LIMIT),
                i32::from(NOTE_OPEN_LIMIT - state),
                0,
            );

        if state >= NOTE_OPEN_LIMIT {
            self.panes[P_MSK0].set_init_alpha();
            self.panes[P_STR0].set_init_alpha();
            self.panes[P_ST00].set_init_alpha();
            out.sound(SE_EXP_IN);
        } else {
            self.panes[P_MSK0].set_now_alpha(fade);
            self.panes[P_STR0].set_now_alpha(fade);
            self.panes[P_ST00].set_now_alpha(fade);
        }

        let size = self.panes[P_NO11].size;
        let base = f32::from(self.panes[P_NO11].user_area);
        self.panes[P_NO11].rotate(size.x / 2.0, size.y / 2.0, base + fade * (-3.0 - base));

        let scale = if state < MODE_SKIP_FRAMES / 2 {
            (f32::from(state) * 0.8) / 5.0
        } else {
            (1.0 - value_increase(0xC, i32::from(NOTE_OPEN_LIMIT - state), 0)) * NOTE_FADE + 0.8
        };

        self.panes[P_NO11].pane_trans(dx * scale, dy * scale);
        self.panes[P_NT00].user_area += 1;
    }

    pub fn note_close(&mut self) {
        let far_x = (SCREEN_W - self.panes[P_NO11].pos_center_orig.x) - CENTER_X;
        let far_y = (SCREEN_H - self.panes[P_NO11].pos_center_orig.y) - CENTER_Y;
        let near_x = CENTER_X - self.panes[P_NO11].pos_center_orig.x;
        let near_y = CENTER_Y - self.panes[P_NO11].pos_center_orig.y;
        let state = self.panes[P_NT00].user_area;
        let fade = value_increase(10, i32::from(state - NOTE_CLOSE_START), 0);

        self.panes[P_NO11].pane_trans(far_x * fade + near_x, far_y * fade + near_y);

        let size = self.panes[P_NO11].size;
        self.panes[P_NO11].rotate(size.x / 2.0, size.y / 2.0, fade * NOTE_ROTATE_END - 3.0);

        self.panes[P_NT00].user_area += 1;

        if self.panes[P_NT00].user_area > NOTE_CLOSE_LIMIT {
            self.panes[P_NT00].set_init_alpha();
            self.panes[P_NK00].set_init_alpha();
            self.panes[P_NO11].set_init_alpha();
            self.panes[P_MSK0].set_now_alpha_zero();
            self.panes[P_STR0].set_now_alpha_zero();
            self.panes[P_ST00].set_now_alpha_zero();
            self.panes[P_NK00].user_area = 0;
            self.panes[P_NT00].user_area = 0;
            let size = self.panes[P_NO11].size;
            let angle = f32::from(self.panes[P_NO11].user_area);
            self.panes[P_NO11].rotate(size.x / 2.0, size.y / 2.0, angle);
            self.note_init();
        } else {
            let inv = 1.0 - fade;
            self.panes[P_NT00].set_now_alpha(inv);
            self.panes[P_NK00].set_now_alpha(inv);
            self.panes[P_NO11].set_now_alpha(inv);
            self.panes[P_MSK0].set_now_alpha(inv);
            self.panes[P_STR0].set_now_alpha(inv);
            self.panes[P_ST00].set_now_alpha(inv);
        }
    }

    pub fn main_trans(&mut self, input: &CollectInput, x: f32, y: f32) {
        for index in 0..CURSOR_CORNER_COUNT {
            self.panes[P_CAR0 + index].pane_trans(x, y);
        }
        self.panes[P_WDSV].pane_trans(x, y);
        self.panes[P_WDOP].pane_trans(x, y);
        for index in 0..4 {
            self.panes[P_CORNER0 + index].pane_trans(x, y);
        }
        self.panes[P_NUM1].pane_trans(x, y);
        self.panes[P_NUM2].pane_trans(x, y);
        self.panes[P_NUMC].pane_trans(x, y);
        self.panes[P_CMAP].pane_trans(x, y);
        self.panes[P_WK1].pane_trans(x, y);
        self.panes[P_WK0].pane_trans(x, y);
        for index in 0..TRIFORCE_SLOT_COUNT {
            self.panes[P_TRI0 + index].pane_trans(x, y);
        }
        self.panes[P_TRIB].pane_trans(x, y);
        for index in 0..3 {
            self.panes[P_SIT + index].pane_trans(x, y);
            self.panes[P_SIK + index].pane_trans(x, y);
        }
        for index in 0..11 {
            self.panes[P_CI + index].pane_trans(x, y);
        }
        for index in 0..3 {
            self.panes[P_HTP + index].pane_trans(x, y);
        }
        self.panes[P_HTPB].pane_trans(x, y);

        let tact = self.now_item <= 5
            && input.collect.is_tact(self.now_item)
            && self.collect_mode != COLLECT_MODE_TRIFORCE;
        for index in 0..SONG_COUNT {
            self.panes[P_TIT + index].pane_trans(x, y);
            self.panes[P_TIK + index].pane_trans(x, y);
            if tact {
                self.tact_trans(self.now_item, x, y);
            } else {
                self.panes[P_CN + index].pane_trans(x, y);
                self.panes[P_WN + index].pane_trans(x, y);
                self.panes[P_I0 + index].pane_trans(x, y);
                self.panes[P_I1 + index].pane_trans(x, y);
                self.panes[P_BS + index].pane_trans(x, y);
                self.panes[P_HMB + index].pane_trans(x, y);
            }
        }
        self.panes[P_I111].pane_trans(x, y);
        self.panes[P_I122].pane_trans(x, y);
        self.panes[P_CC21].pane_trans(x, y);
    }

    pub fn sub_trans(&mut self, x: f32, y: f32) {
        for index in 0..5 {
            self.panes[P_WIT + index].pane_trans(x, y);
            self.panes[P_WIK + index].pane_trans(x, y);
            self.panes[P_IP + index].pane_trans(x, y);
        }
        self.panes[P_IP00].pane_trans(x, y);
        self.panes[P_CC00].pane_trans(x, y);
    }

    pub fn title_trans(&mut self, x: f32, y: f32) {
        self.panes[P_TK00].pane_trans(x, y);
    }

    pub fn note_rotate(&mut self, y: f32, angle: f32) {
        self.panes[P_NO11].pane_trans(0.0, y);
        let size = self.panes[P_NO11].size;
        self.panes[P_NO11].rotate(size.x / 2.0, size.y / 2.0, angle);
    }

    pub fn name_trans(&mut self, x: f32, y: f32) {
        self.panes[P_WD].pane_trans(x, y);
        let pos = self.panes[P_WD].pos_top_left;
        self.out_font.set_left_up_pos(pos.x, pos.y);
        self.out_font.move_out_font(&mut self.panes);
        self.panes[P_ITNK].pane_trans(x, y);
    }

    pub fn main_open_proc(&mut self, input: &CollectInput, step: i16, max: i16, offset: i16) {
        if max < step {
            return;
        }
        let open_t = value_increase(i32::from(max), i32::from(step), 0);
        let slide = f32::from(offset) * value_increase(i32::from(max), i32::from(max - step), 0);
        self.main_trans(input, slide, 0.0);
        for index in 0..CURSOR_CORNER_COUNT {
            self.panes[P_CAR0 + index].set_now_alpha(open_t);
        }
        self.panes[P_WDSV].set_now_alpha(open_t);
        self.panes[P_WDOP].set_now_alpha(open_t);
        for index in 0..4 {
            self.panes[P_CORNER0 + index].set_now_alpha(open_t);
        }
        for index in [P_NUM1, P_NUM2, P_NUMC, P_CMAP, P_WK1, P_WK0, P_TRIB, P_HTPB] {
            self.panes[index].set_now_alpha(open_t);
        }
        for index in 0..TRIFORCE_SLOT_COUNT {
            self.panes[P_TRI0 + index].set_now_alpha(open_t);
        }
        for index in 0..3 {
            self.panes[P_SIT + index].set_now_alpha(open_t);
            self.panes[P_SIK + index].set_now_alpha(open_t);
            self.panes[P_HTP + index].set_now_alpha(open_t);
        }
        for index in 0..11 {
            self.panes[P_CI + index].set_now_alpha(open_t);
        }
        for index in 0..SONG_COUNT {
            for base in [P_TIT, P_TIK, P_CN, P_WN, P_I0, P_I1, P_BS, P_HMB] {
                self.panes[base + index].set_now_alpha(open_t);
            }
        }
        for index in [P_I111, P_I122, P_CC43, P_CC32, P_CC21] {
            self.panes[index].set_now_alpha(open_t);
        }
    }

    pub fn sub_open_proc(&mut self, step: i16, max: i16, offset: i16) {
        if max < step {
            return;
        }
        let open_t = value_increase(i32::from(max), i32::from(step), 0);
        let slide = f32::from(offset) * value_increase(i32::from(max), i32::from(max - step), 0);
        self.sub_trans(slide, 0.0);
        for index in 0..5 {
            self.panes[P_WIT + index].set_now_alpha(open_t);
            self.panes[P_WIK + index].set_now_alpha(open_t);
            self.panes[P_IP + index].set_now_alpha(open_t);
        }
        for index in [P_IP00, P_CC14, P_CC07, P_CC00] {
            self.panes[index].set_now_alpha(open_t);
        }
    }

    pub fn title_open_proc(&mut self, step: i16, max: i16, out: &mut CollectOutput) {
        if max < step {
            return;
        }
        value_increase(i32::from(max), i32::from(step), 0);
        let rise = value_increase(i32::from(max), i32::from(max - step), 0) * TITLE_SHIFT;
        self.title_trans(0.0, rise);
        self.panes[P_TL00].set_init_alpha();
        self.panes[P_TK00].set_init_alpha();
        if step == 1 {
            out.sound(SE_PANEL_IN);
        }
    }

    pub fn note_open_proc(&mut self, step: i16, max: i16) {
        if max < step {
            return;
        }
        value_increase(i32::from(max), i32::from(step), 0);
        let t = value_increase(i32::from(max), i32::from(max - step), 0);
        let base = f32::from(self.panes[P_NO11].user_area);
        let angle = base + t * (NOTE_SWING - base);
        self.note_rotate(t * NOTE_ROTATE, angle);
        self.panes[P_NT00].set_init_alpha();
        self.panes[P_NK00].set_init_alpha();
    }

    pub fn name_open_proc(&mut self, step: i16, max: i16) {
        if max < step {
            return;
        }
        value_increase(i32::from(max), i32::from(step), 0);
        let rise = value_increase(i32::from(max), i32::from(max - step), 0) * NAME_SHIFT;
        self.name_trans(0.0, rise);
        self.panes[P_NM00].set_init_alpha();
        self.panes[P_ITNM].set_init_alpha();
        self.panes[P_ITNK].set_init_alpha();
    }

    pub fn item_bit_check(&mut self, input: &CollectInput, _tune: &CollectTune) {
        for index in 0..SONG_COUNT {
            let known = input.collect.is_tact(index as u8);
            self.set_show(P_TIT + index, known);
            self.set_show(P_TIK + index, known);
            if index == 3 {
                self.set_show(P_I111, known);
            } else if index == 4 {
                self.set_show(P_I122, known);
            }
        }

        let life = input.life();
        for index in 0..3u8 {
            self.set_show(P_HTP + usize::from(index), index < life % 4);
        }

        let map = input.map_open();
        for entry in [P_NUM1, P_NUM2, P_NUMC, P_CMAP, P_WK1] {
            self.set_show(entry, map);
        }

        self.map_open = map;
        self.triforce_count = triforce_num(&input.collect);

        if self.triforce_count == 8 && self.collect_mode != COLLECT_MODE_TRIFORCE {
            if self.triforce_count != self.panes[P_TRI0 + 4].user_area as u8 {
                for index in 0..TRIFORCE_SLOT_COUNT {
                    self.set_show(P_TRI0 + index, index == 4);
                }
                self.panes[P_TRI0 + 4].change_texture(TRIFORCE_FULL_TEX);
                self.panes[P_TRI0 + 4].pos_center_orig = self.panes[P_TRIB].pos_center_orig;
                self.panes[P_TRI0 + 4].pos_center = self.panes[P_TRIB].pos_center;
                self.panes[P_TRI0 + 4].size.x = TRIFORCE_FULL_WIDTH * TRIFORCE_FULL_SCALE;
                self.panes[P_TRI0 + 4].size.y = TRIFORCE_FULL_HEIGHT * TRIFORCE_FULL_SCALE;
                self.panes[P_TRI0 + 4].cpos_move();
                self.panes[P_TRI0 + 4].user_area = i16::from(self.triforce_count);
            }
        } else {
            if self.triforce_count != self.panes[P_TRI0 + 4].user_area as u8 {
                self.panes[P_TRI0 + 4].change_texture(TRIFORCE_TEX[4]);
                self.panes[P_TRI0 + 4].pos_center_orig.x =
                    self.panes[P_TRIB].pos_center_orig.x - self.tri_offset_x;
                self.panes[P_TRI0 + 4].pos_center_orig.y =
                    self.panes[P_TRIB].pos_center_orig.y - self.tri_offset_y;
                self.panes[P_TRI0 + 4].pos_center.x =
                    self.panes[P_TRIB].pos_center.x - self.tri_offset_x;
                self.panes[P_TRI0 + 4].pos_center.y =
                    self.panes[P_TRIB].pos_center.y - self.tri_offset_y;
                self.panes[P_TRI0 + 4].size = self.panes[P_TRI0 + 4].size_orig;
                self.panes[P_TRI0 + 4].cpos_move();
            }
            for index in 0..TRIFORCE_SLOT_COUNT {
                let known = input.collect.is_triforce(index as u8);
                self.set_show(P_TRI0 + index, known);
            }
            self.panes[P_TRI0 + 4].user_area = i16::from(self.triforce_count);
        }

        for index in 0..3u8 {
            let known = input.collect.is_symbol(index);
            self.set_show(P_SIT + usize::from(index), known);
            self.set_show(P_SIK + usize::from(index), known);
        }

        for index in 0..COLLECT_ITEM_ROWS {
            let known = check_collect(&input.collect, index);
            self.set_show(P_WIT + usize::from(index), known);
            self.set_show(P_WIK + usize::from(index), known);
            self.set_show(P_IP + usize::from(index), known);
        }

        if input.collect.test(ITEM_BOTTLE, 0) && !input.collect.test(ITEM_BOTTLE, 1) {
            self.panes[P_IP + 4].set_contents_color(WINDOW_DIM_COLOR);
            self.panes[P_IP + 4].set_white(WINDOW_DIM_WHITE);
            self.panes[P_IP + 4].set_black(WINDOW_DIM_BLACK);
        } else {
            self.panes[P_IP + 4].set_contents_color(WINDOW_BACK_COLOR);
            self.panes[P_IP + 4].set_white(WINDOW_BACK_WHITE);
            self.panes[P_IP + 4].set_black(WINDOW_BACK_BLACK);
        }
    }

    fn set_show(&mut self, index: usize, show: bool) {
        if show {
            self.panes[index].show();
        } else {
            self.panes[index].hide();
        }
    }

    pub fn item_scale(&mut self) {
        let now = self.now_item;
        for index in 0..0x15usize {
            match index {
                0..=5 => {
                    let scale = if index as u8 == now {
                        CURSOR_FAST_SCALE
                    } else {
                        1.0
                    };
                    self.panes[P_TIT + index].pane_scale_xy(scale);
                    self.panes[P_TIK + index].pane_scale_xy(scale);

                    if now == 3 {
                        self.order.insert_child(P_TIT, P_I111);
                        self.panes[P_I111].pane_scale_xy(CURSOR_FAST_SCALE);
                    } else {
                        self.order.insert_child(P_TIK + 5, P_I111);
                        self.panes[P_I111].pane_scale_xy(1.0);
                    }
                    let size = self.panes[P_I111].size;
                    let angle = self.panes[P_I111].get_rotate();
                    self.panes[P_I111].rotate(size.x / 2.0, size.y / 2.0, angle);

                    if now == 4 {
                        self.order.insert_child(P_TIT, P_I122);
                        self.panes[P_I122].pane_scale_xy(CURSOR_FAST_SCALE);
                    } else {
                        self.order.insert_child(P_TIK + 5, P_I122);
                        self.panes[P_I122].pane_scale_xy(1.0);
                    }
                    let size = self.panes[P_I122].size;
                    let angle = self.panes[P_I122].get_rotate();
                    self.panes[P_I122].rotate(size.x / 2.0, size.y / 2.0, angle);
                }
                9 => {
                    let scale = if index as u8 == now {
                        CURSOR_FAST_SCALE
                    } else {
                        1.0
                    };
                    self.panes[P_CMAP].pane_scale_xy(scale);
                    self.panes[P_CMAP].pane_scale_xy(scale);
                }
                11..=13 => {
                    let slot = index - 11;
                    let scale = if index as u8 == now {
                        CURSOR_FAST_SCALE
                    } else {
                        1.0
                    };
                    self.panes[P_SIT + slot].pane_scale_xy(scale);
                    self.panes[P_SIK + slot].pane_scale_xy(scale);
                }
                14..=18 => {
                    let slot = index - 14;
                    let scale = if index as u8 == now {
                        CURSOR_FAST_SCALE
                    } else {
                        1.0
                    };
                    self.panes[P_WIT + slot].pane_scale_xy(scale);
                    self.panes[P_WIK + slot].pane_scale_xy(scale);
                }
                _ => {}
            }
        }
    }

    pub fn collect_priority(&mut self) {
        // tww/src/d/d_menu_collect.cpp:1893-1982 (the insertChild chain fixes the draw order).
        let mut chain: Vec<usize> = vec![P_CAR0, P_WDSV, P_WDOP];
        for index in (0..4).rev() {
            chain.push(P_CORNER0 + index);
        }
        chain.extend([P_NUM1, P_NUM2, P_NUMC, P_CMAP, P_WK1, P_WK0, P_TRIM, P_TRIL]);
        for index in 0..TRIFORCE_SLOT_COUNT {
            chain.push(P_TRI0 + index);
        }
        chain.push(P_TRIB);
        for index in 0..3 {
            chain.push(P_SIT + index);
        }
        for index in 0..3 {
            chain.push(P_SIK + index);
        }
        for index in 0..11 {
            chain.push(P_CI + index);
        }
        for index in 0..3 {
            chain.push(P_HTP + index);
        }
        chain.push(P_HTPB);
        for index in 0..SONG_COUNT {
            chain.push(P_TIT + index);
        }
        for index in 0..SONG_COUNT {
            chain.push(P_TIK + index);
        }
        chain.push(P_I111);
        chain.push(P_I122);
        for base in [P_CN, P_WN, P_I0, P_I1, P_BS, P_HMB] {
            for index in 0..SONG_COUNT {
                chain.push(base + index);
            }
        }
        chain.push(P_CC21);
        self.rebuild_order(&chain);
    }

    pub fn wepon_priority(&mut self) {
        let mut chain: Vec<usize> = vec![P_CAR0];
        for index in 0..5 {
            chain.push(P_WIT + index);
        }
        for index in 0..5 {
            chain.push(P_WIK + index);
        }
        for index in 0..5 {
            chain.push(P_IP + index);
        }
        chain.push(P_IP00);
        chain.push(P_CC00);
        self.rebuild_order(&chain);
    }

    fn rebuild_order(&mut self, chain: &[usize]) {
        self.order = DrawOrder::new(PANE_COUNT);
        let mut previous = chain[0];
        for entry in chain.iter().skip(1) {
            self.order.insert_child(previous, *entry);
            previous = *entry;
        }
    }

    pub fn tact_guide_show(&mut self, song: u8, fade_in: bool) {
        let beats = TACT_BEAT
            .get(usize::from(song))
            .copied()
            .unwrap_or(0)
            .max(0);
        let step_gap = self.panes[P_CN + 1].pos_center_orig.x - self.panes[P_CN].pos_center_orig.x;
        let x_offset = if beats == 3 {
            step_gap * 1.5
        } else if beats == 4 {
            step_gap
        } else {
            0.0
        };
        let right_offset = x_offset + NOTE_SLIDE_STEP;

        let mut pattern = [0i32; SONG_COUNT];
        for index in 0..SONG_COUNT {
            let frames = melody_pattern(song, index, &mut pattern[index]);
            self.panes[P_TIT + index].user_area = frames as i16;
            if fade_in && index < beats as usize {
                for base in [P_CN, P_WN, P_I0, P_I1, P_BS, P_HMB] {
                    self.panes[base + index].set_init_alpha();
                }
            }
            if index < beats as usize {
                self.panes[P_HMB + index].show();
            }
        }

        for index in 0..beats as usize {
            if self.panes[P_HMB + index].user_area != 1 {
                continue;
            }
            let size = self.panes[P_I0 + index].size_orig;
            let angle = f32::from(
                TACT_ROTATE_ANGLE
                    .get(pattern[index] as usize)
                    .copied()
                    .unwrap_or(0),
            );
            self.panes[P_I0 + index].rotate(size.x / 2.0, size.y / 2.0, angle);

            if pattern[index] == 0 {
                for base in [P_CN, P_WN, P_I0, P_I1, P_BS, P_HMB] {
                    self.panes[base + index].show();
                }
                self.panes[P_CN + index].pane_trans(x_offset, 0.0);
            } else {
                for base in [P_CN, P_WN, P_I0, P_BS, P_HMB] {
                    self.panes[base + index].show();
                }
                self.panes[P_I1 + index].hide();
                let (dx, dy) = match pattern[index] {
                    1 => (x_offset, -NOTE_SLIDE_STEP),
                    2 => (right_offset, 0.0),
                    3 => (x_offset, NOTE_SLIDE_STEP),
                    4 => (x_offset - NOTE_SLIDE_STEP, 0.0),
                    _ => (x_offset, 0.0),
                };
                self.panes[P_CN + index].pane_trans(dx, dy);
            }

            for base in [P_WN, P_I0, P_I1, P_BS, P_HMB] {
                self.panes[base + index].pane_trans(x_offset, 0.0);
            }
        }
    }

    pub fn tact_base_show(&mut self) {
        for index in 0..SONG_COUNT {
            self.panes[P_HMB + index].pane_trans(0.0, 0.0);
            self.panes[P_HMB + index].show();
        }
    }

    pub fn tact_guide_hide(&mut self) {
        for index in 0..SONG_COUNT {
            self.panes[P_HMB + index].user_area = 0;
            for base in [P_CN, P_WN, P_I0, P_I1, P_BS, P_HMB] {
                self.panes[base + index].hide();
            }
        }
    }

    pub fn tact_trans(&mut self, song: u8, x: f32, y: f32) {
        let beats = TACT_BEAT
            .get(usize::from(song))
            .copied()
            .unwrap_or(0)
            .max(0);
        let step_gap = self.panes[P_CN + 1].pos_center_orig.x - self.panes[P_CN].pos_center_orig.x;
        let x = if beats == 3 {
            x + step_gap * 1.5
        } else if beats == 4 {
            x + step_gap
        } else {
            x
        };

        let mut pattern = [0i32; SONG_COUNT];
        for index in 0..SONG_COUNT {
            let frames = melody_pattern(song, index, &mut pattern[index]);
            self.panes[P_TIT + index].user_area = frames as i16;
        }

        for index in 0..beats as usize {
            let (dx, dy) = match pattern[index] {
                0 => (0.0, 0.0),
                1 => (0.0, -NOTE_SLIDE_STEP),
                2 => (NOTE_SLIDE_STEP, 0.0),
                3 => (0.0, NOTE_SLIDE_STEP),
                4 => (-NOTE_SLIDE_STEP, 0.0),
                _ => (0.0, 0.0),
            };
            self.panes[P_CN + index].pane_trans(x + dx, y + dy);
            for base in [P_WN, P_I0, P_I1, P_BS, P_HMB] {
                self.panes[base + index].pane_trans(x, y);
            }
        }
    }

    pub fn tact_demo_mode(
        &mut self,
        input: &CollectInput,
        pad: &PadInput,
        out: &mut CollectOutput,
    ) {
        let mut step_mark = [0i16; 8];
        let mut sum = 0i16;
        for index in 0..SONG_COUNT {
            sum += self.panes[P_TIT + index].user_area;
            step_mark[index + 1] = sum;
        }
        step_mark[7] = sum + 10;

        if self.cursor_main_move(input, pad, out) {
            self.item_scale();
            if self.collect_item_get_check(input, self.now_item) {
                self.item_name_move();
            } else {
                self.clear_name_alpha();
            }
            out.melody_stop = true;
            self.out_font_move();
            self.collect_mode = COLLECT_MODE_BROWSE;
        } else {
            let counter = self.panes[P_TIK].user_area;
            if counter < step_mark[6] {
                self.panes[P_TIK].user_area = counter + 1;
                for index in 0..SONG_COUNT {
                    self.panes[P_HMB + index].user_area =
                        i16::from(step_mark[index] <= self.panes[P_TIK].user_area);
                }
                self.tact_guide_show(self.now_item, true);
            } else if counter < step_mark[7] {
                self.panes[P_TIK].user_area = counter + 1;
                for index in 0..SONG_COUNT {
                    self.panes[P_HMB + index].user_area = 1;
                }
            } else {
                for index in 0..SONG_COUNT {
                    self.panes[P_HMB + index].user_area = 0;
                }
                self.panes[P_TIK].user_area = 0;
                self.tact_guide_show(self.now_item, true);
                self.collect_mode = COLLECT_MODE_PLAY;
                out.a_status_force = Some(ACT_BLANK);
            }
        }

        out.a_status_force = Some(ACT_RETURN);
    }

    pub fn tact_play_mode(
        &mut self,
        input: &CollectInput,
        pad: &PadInput,
        out: &mut CollectOutput,
        tune: &CollectTune,
    ) {
        let song = self.now_item;
        let beats = TACT_BEAT
            .get(usize::from(song))
            .copied()
            .unwrap_or(0)
            .max(0);
        let progress = self.panes[P_TIK].user_area;
        if progress >= beats {
            self.input_timer -= 1;
            if self.input_timer <= 0 {
                out.sound(SE_TACT_RIGHT);
                for index in 0..SONG_COUNT {
                    self.panes[P_HMB + index].user_area = 0;
                }
                self.panes[P_TIK].user_area = 0;
                self.tact_guide_show(song, true);
                self.collect_mode = COLLECT_MODE_PLAY;
            }
            return;
        }

        let step_gap = self.panes[P_CN + 1].pos_center_orig.x - self.panes[P_CN].pos_center_orig.x;
        let x_offset = if beats == 3 {
            self.beat = 3;
            step_gap * 1.5
        } else if beats == 4 {
            self.beat = 4;
            step_gap
        } else {
            self.beat = 6;
            0.0
        };

        let mut pattern = [0i32; SONG_COUNT];
        for index in 0..SONG_COUNT {
            let frames = melody_pattern(song, index, &mut pattern[index]);
            self.panes[P_TIT + index].user_area = frames as i16;
            if index < beats as usize {
                if (index as i16) < progress {
                    for base in [P_CN, P_WN, P_I0, P_I1, P_BS] {
                        self.panes[base + index].set_init_alpha();
                    }
                } else {
                    for base in [P_CN, P_WN, P_I0, P_I1, P_BS] {
                        self.panes[base + index].set_now_alpha(PARTIAL_TRANSPARENT);
                    }
                }
                self.panes[P_HMB + index].set_init_alpha();
                self.panes[P_HMB + index].show();
            }
        }

        let direction = tact_direction(
            0,
            self.tac_direction,
            pad.sub_stick_value,
            pad.sub_stick_angle,
            pad.hold(),
        );

        let mut input_ready = false;
        if direction != self.tac_direction {
            self.tac_direction = direction;
            if direction != 0 {
                input_ready = true;
                self.input_timer = 0;
            } else if pattern[progress.max(0) as usize] == 0 {
                self.input_timer = tune.input_hold;
            } else {
                self.input_timer = 0;
            }
        } else if direction == 0 {
            if self.input_timer > 0 {
                self.input_timer -= 1;
                if self.input_timer == 0 {
                    input_ready = true;
                }
            }
        } else {
            self.input_timer = 0;
        }

        if input_ready {
            out.sound(tact_pulse_sound(self.beat));
            out.sound(SE_METRONOME);
            let note = tact_note(self.beat, self.tac_direction, self.beat_variant != 0);
            out.tact_note = Some(note);
            if progress == 0 {
                self.beat_variant = u8::from(self.beat == 6 && self.tac_direction == 1);
            }
            if progress == 5 {
                self.beat_variant = 0;
            }

            let slot = progress.max(0) as usize;
            if slot < SONG_COUNT && self.tac_direction == pattern[slot] {
                self.panes[P_HMB + slot].user_area = 1;
                self.panes[P_TIK].user_area += 1;
                if self.panes[P_TIK].user_area == beats {
                    self.input_timer = MODE_SKIP_FRAMES;
                }
                self.reveal_beat(x_offset, &pattern, beats);
            } else {
                for index in 0..SONG_COUNT {
                    self.panes[P_HMB + index].user_area = 0;
                }
                self.panes[P_TIK].user_area = 0;
                out.sound(SE_TACT_WRONG);
            }
        }

        if pad.a {
            self.collect_mode = COLLECT_MODE_DEMO;
            self.panes[P_TIK].user_area = 0;
            self.tact_guide_hide();
            for index in 0..SONG_COUNT {
                if (index as i16) < TACT_BEAT[usize::from(self.now_item)] {
                    self.panes[P_HMB + index].show();
                }
            }
            out.melody_play = Some(song);
            out.sound(SE_DANCE_01 + u32::from(song));
        } else if self.cursor_main_move(input, pad, out) {
            self.item_scale();
            if self.collect_item_get_check(input, self.now_item) {
                self.item_name_move();
            } else {
                self.clear_name_alpha();
            }
            self.out_font_move();
            self.collect_mode = COLLECT_MODE_BROWSE;
        }
    }

    fn reveal_beat(&mut self, x_offset: f32, pattern: &[i32; SONG_COUNT], beats: i16) {
        let right_offset = x_offset + NOTE_SLIDE_STEP;
        for index in 0..beats as usize {
            if self.panes[P_HMB + index].user_area != 1 {
                continue;
            }
            let size = self.panes[P_I0 + index].size_orig;
            let angle = f32::from(
                TACT_ROTATE_ANGLE
                    .get(pattern[index] as usize)
                    .copied()
                    .unwrap_or(0),
            );
            self.panes[P_I0 + index].rotate(size.x / 2.0, size.y / 2.0, angle);
            if pattern[index] == 0 {
                for base in [P_CN, P_WN, P_I0, P_I1, P_BS, P_HMB] {
                    self.panes[base + index].show();
                }
                self.panes[P_CN + index].pane_trans(x_offset, 0.0);
            } else {
                for base in [P_CN, P_WN, P_I0, P_BS, P_HMB] {
                    self.panes[base + index].show();
                }
                self.panes[P_I1 + index].hide();
                let (dx, dy) = match pattern[index] {
                    1 => (x_offset, -NOTE_SLIDE_STEP),
                    2 => (right_offset, 0.0),
                    3 => (x_offset, NOTE_SLIDE_STEP),
                    4 => (x_offset - NOTE_SLIDE_STEP, 0.0),
                    _ => (x_offset, 0.0),
                };
                self.panes[P_CN + index].pane_trans(dx, dy);
            }
            for base in [P_WN, P_I0, P_I1, P_BS, P_HMB] {
                self.panes[base + index].pane_trans(x_offset, 0.0);
            }
            for base in [P_CN, P_WN, P_I0, P_I1, P_BS] {
                self.panes[base + index].set_init_alpha();
            }
        }
    }

    pub fn corner_move(&mut self, tune: &CollectTune) {
        self.corner_timer += 1;
        if self.corner_timer >= CORNER_PERIOD {
            self.corner_timer = 0;
        }
        let t = value_increase(
            i32::from(CORNER_SEGMENT),
            i32::from(self.corner_timer % CORNER_SEGMENT),
            2,
        );
        let base = self.contents_color;
        let window = P_CORNER0 + 1;
        let out_color = if self.corner_timer < CORNER_SEGMENT {
            ContentsColor {
                tl: lerp(base.tl, base.bl, t),
                tr: lerp(base.tr, base.tl, t),
                bl: lerp(base.bl, base.br, t),
                br: lerp(base.br, base.tr, t),
            }
        } else if self.corner_timer < CORNER_SEGMENT * 2 {
            ContentsColor {
                tl: lerp(base.bl, base.br, t),
                tr: lerp(base.tl, base.bl, t),
                bl: lerp(base.br, base.tr, t),
                br: lerp(base.tr, base.tl, t),
            }
        } else if self.corner_timer < CORNER_SEGMENT * 3 {
            ContentsColor {
                tl: lerp(base.br, base.tr, t),
                tr: lerp(base.bl, base.br, t),
                bl: lerp(base.tr, base.tl, t),
                br: lerp(base.tl, base.bl, t),
            }
        } else {
            ContentsColor {
                tl: lerp(base.tr, base.tl, t),
                tr: lerp(base.br, base.tr, t),
                bl: lerp(base.tl, base.bl, t),
                br: lerp(base.bl, base.br, t),
            }
        };
        self.panes[window].contents_color = out_color;

        self.pulse_timer += 1;
        let limit = tune.corner_pulse;
        if self.pulse_timer >= limit * 2 {
            self.pulse_timer = 0;
        }
        let pulse = if self.pulse_timer < limit {
            value_increase(i32::from(limit), i32::from(self.pulse_timer), 0)
        } else {
            value_increase(i32::from(limit), i32::from(limit * 2 - self.pulse_timer), 0)
        };

        let color = tune.pulse_color;
        let full = CORNER_PULSE_PEAK;
        let fade = (full - f32::from(color[0])) * pulse;
        let r = (full - fade) as u8;
        let g = (CORNER_PULSE_GREEN - (CORNER_PULSE_GREEN - f32::from(color[1])) * pulse) as u8;
        let b = (f32::from(color[2]) * pulse) as u8;
        self.panes[window].set_white(pack_color([r, g, b, 0xFF]));
    }

    pub fn triforce_anime(&mut self, tune: &CollectTune, max_alpha: u8) {
        let half = tune.triforce_period;
        self.panes[P_TRIL].user_area += 1;
        if self.panes[P_TRIL].user_area >= half * 2 {
            self.panes[P_TRIL].user_area = 0;
        }
        let user_area = self.panes[P_TRIL].user_area;
        if half > user_area {
            self.panes[P_TRIL].now_alpha = (f32::from(max_alpha)
                * value_increase(i32::from(half), i32::from(user_area), 0))
                as u8;
        } else if half * 2 > user_area {
            self.panes[P_TRIL].now_alpha = (f32::from(max_alpha)
                * value_increase(i32::from(half), i32::from(half * 2 - user_area), 0))
                as u8;
        }
    }

    pub fn clear_name_alpha(&mut self) {
        self.panes[P_NM00].set_now_alpha_zero();
        self.panes[P_NM01].set_now_alpha_zero();
        self.panes[P_WD].set_now_alpha_zero();
    }

    pub fn item_name_move(&mut self) {
        let now = self.now_item;
        match now {
            0..=7 | 0x12 => {
                self.panes[P_WD].user_area += 1;
                if self.panes[P_WD].user_area > NAME_HOLD_LIMIT {
                    self.panes[P_WD].user_area = NAME_HOLD_RESET;
                }
                let timer = i32::from(self.panes[P_WD].user_area);
                if timer <= 10 {
                    let t = value_increase(10, timer, 0);
                    self.panes[P_NM00].set_now_alpha(t);
                    self.panes[P_NM01].set_now_alpha(1.0 - t);
                    self.panes[P_WD].set_now_alpha_zero();
                } else if timer <= 60 {
                    self.panes[P_NM00].set_init_alpha();
                    self.panes[P_WD].set_now_alpha_zero();
                } else if timer <= 70 {
                    let t = value_increase(10, timer - 60, 0);
                    self.panes[P_NM00].set_now_alpha(1.0 - t);
                    self.panes[P_WD].set_now_alpha(t);
                } else if timer <= 120 {
                    self.panes[P_NM00].set_now_alpha_zero();
                    self.panes[P_WD].set_init_alpha();
                } else if timer <= 130 {
                    let t = value_increase(10, timer - 120, 0);
                    self.panes[P_NM00].set_now_alpha(t);
                    self.panes[P_WD].set_now_alpha(1.0 - t);
                }
                self.out_font.move_out_font(&mut self.panes);
            }
            _ => {
                let timer = self.panes[P_WD].user_area;
                if timer < 10 {
                    self.panes[P_WD].user_area = timer + 1;
                }
                let alpha_timer = i32::from(self.panes[P_WD].user_area);
                if alpha_timer <= 10 {
                    let t = value_increase(10, alpha_timer, 0);
                    self.panes[P_NM00].set_now_alpha(t);
                    self.panes[P_NM01].set_now_alpha(1.0 - t);
                    self.panes[P_WD].set_now_alpha_zero();
                } else {
                    self.panes[P_NM00].set_init_alpha();
                    self.panes[P_NM01].set_now_alpha_zero();
                    self.panes[P_WD].set_now_alpha_zero();
                }
            }
        }
    }

    pub fn collect_item_get_check(&self, input: &CollectInput, item: u8) -> bool {
        let collect = &input.collect;
        match item {
            0..=5 => collect.is_tact(item),
            6 | 7 => true,
            8 => input.life() % 4 != 0,
            9 => input.map_open(),
            10 => (0..TRIFORCE_SLOT_COUNT).any(|index| collect.is_triforce(index as u8)),
            0x0B..=0x0D => collect.is_symbol(item - 0x0B),
            0x0E..=0x12 => check_collect(collect, item - 0x0E),
            _ => false,
        }
    }
}

pub const MSG_STICK_A: i16 = 10;
pub const MSG_STICK_B: i16 = 0;
pub const MSG_STICK_C: i16 = 0;

pub fn stick_period() -> i16 {
    MSG_STICK_B + MSG_STICK_A + MSG_STICK_C + MSG_STICK_A
}

fn stick_blend(timer: i16, piece: i16) -> [f32; 4] {
    let period = stick_period();
    let start = MSG_STICK_B + period * piece;
    let fade_in = MSG_STICK_A + start;
    let solid = MSG_STICK_C + fade_in;
    let fade_out = MSG_STICK_A + solid;
    if timer < start {
        BLEND_FULL
    } else if timer < fade_in {
        let v = value_increase(i32::from(period), i32::from(timer - start), 0);
        [v, 1.0 - v, 1.0, 1.0]
    } else if timer < solid {
        BLEND_ALPHA
    } else if timer < fade_out {
        let v = value_increase(i32::from(period), i32::from(timer - solid), 0);
        [1.0 - v, v, 1.0, 1.0]
    } else {
        BLEND_FULL
    }
}

fn lerp(from: [u8; 4], to: [u8; 4], t: f32) -> [u8; 4] {
    [
        (f32::from(from[0]) + (f32::from(to[0]) - f32::from(from[0])) * t) as u8,
        (f32::from(from[1]) + (f32::from(to[1]) - f32::from(from[1])) * t) as u8,
        (f32::from(from[2]) + (f32::from(to[2]) - f32::from(from[2])) * t) as u8,
        (f32::from(from[3]) + (f32::from(to[3]) - f32::from(from[3])) * t) as u8,
    ]
}
impl OutFont {
    pub fn set_pane(
        &mut self,
        panes: &mut [Pane],
        text_pane: usize,
        pic: [usize; OUT_FONT_ICON_COUNT],
        texture: &'static str,
        blend_texture: &'static str,
    ) {
        self.box_pane = text_pane;
        self.pic = pic;
        self.initial(panes);
        for index in pic {
            if let Some(pane) = panes.get_mut(index) {
                pane.change_texture(texture);
                pane.append(blend_texture);
            }
        }
    }

    pub fn set_left_up_pos(&mut self, x: f32, y: f32) {
        self.left_up_x = x;
        self.left_up_y = y;
    }

    pub fn initial(&mut self, panes: &mut [Pane]) {
        self.advance = 0.0;
        self.icon_count = 0;
        self.left_up_y = 0.0;
        self.left_up_x = 0.0;
        self.line = 0;
        self.text.clear();
        self.ruby_text.clear();
        if let Some(pane) = panes.get_mut(self.box_pane) {
            pane.show();
            self.cell = pane.font_size_x;
            self.char_space = pane.char_space;
            self.line_width = pane.width();
            self.line_space = pane.line_space;
            self.box_height = pane.height();
        }
        for index in 0..OUT_FONT_ICON_COUNT {
            self.icons[index] = OutFontIcon {
                x: 0.0,
                y: 0.0,
                scale_x: self.cell,
                scale_y: self.cell,
                timer: 0,
                icon_no: OUT_FONT_ICON_A,
                line: -1,
                texture: "",
                blend: [0.0, 1.0, 1.0, 1.0],
                rotation: 0,
                visible: false,
            };
            self.pics[index] = MsgIconPic::default();
            if let Some(pane) = panes.get_mut(self.pic[index]) {
                pane.hide();
                pane.rotate_deg(0.0, 0.0, 0.0);
                pane.size_orig = F2 {
                    x: self.cell,
                    y: self.cell,
                };
                pane.size = pane.size_orig;
                pane.cpos_move();
            }
        }
    }

    pub fn char_width(&self, font: &FontBank, slot: usize, code: u16) -> f32 {
        let Some(res) = font.get(slot) else {
            return 0.0;
        };
        let ratio = self.cell / res.cell_width() as f32;
        self.char_space + res.char_width(code) as f32 * ratio
    }

    fn icon_set(&mut self, panes: &mut [Pane], icon_no: u8, out: &mut String) -> f32 {
        let mut advance = 0.0;
        if self.icon_count >= OUT_FONT_ICON_COUNT as u8 {
            return advance;
        }
        let index = usize::from(self.icon_count);
        self.icons[index].x = self.advance;
        self.icons[index].y = 0.0;
        self.icons[index].icon_no = icon_no;
        self.icons[index].line = self.line;
        self.icons[index].blend = [0.0, 1.0, 1.0, 1.0];
        self.icons[index].visible = true;
        let texture = out_font_texture(icon_no);
        self.icons[index].texture = texture;
        let pane_index = self.pic[index];
        self.pics[index] = MsgIconPic::default();
        if let Some(pane) = panes.get_mut(pane_index) {
            pane.show();
            pane.remove();
            pane.append(texture);
            pane.set_blend_ratio(0.0, 1.0, 1.0, 1.0);
            pane.set_white(out_font_white(icon_no));
            pane.set_black(0);
            match icon_no {
                OUT_FONT_ICON_LEFT => {
                    self.icons[index].x += self.cell;
                    pane.rotate_deg(0.0, 0.0, 270.0);
                    self.icons[index].rotation = pane.rotation;
                }
                OUT_FONT_ICON_RIGHT => {
                    self.icons[index].y += self.cell;
                    pane.rotate_deg(0.0, 0.0, 90.0);
                    self.icons[index].rotation = pane.rotation;
                }
                OUT_FONT_ICON_UP => {
                    self.icons[index].x += self.cell;
                    self.icons[index].y += self.cell;
                    pane.rotate_deg(0.0, 0.0, 180.0);
                    self.icons[index].rotation = pane.rotation;
                }
                _ => {}
            }
        }
        advance = self.cell + self.char_space;
        out.push_str("\u{1b}CR[");
        out.push_str(&format!("{}", (advance + 0.5) as i32));
        out.push(']');
        self.icon_count += 1;
        advance
    }

    pub fn message_set(
        &mut self,
        panes: &mut [Pane],
        font: &mut FontBank,
        archive: &dyn MsgArchive,
        clear_count: u8,
        msg_no: u32,
    ) {
        let mut get = ItemMsgGet::default();
        let Some(head) = get.get_mesg_header(archive, clear_count, msg_no) else {
            return;
        };
        let Some(message) = get.get_message(head) else {
            return;
        };
        let message = message.to_vec();
        self.initial(panes);
        let mut line_text = String::new();
        let mut line_no = self.line;
        let mut index = 0usize;

        while index < message.len() {
            let byte = message[index];
            if byte == 0x1A {
                index += 1;
                let code = message
                    .get(index..index + 4)
                    .map(|raw| u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]) & 0x00FF_FFFF)
                    .unwrap_or(0);
                let length = i32::from(message.get(index).copied().unwrap_or(0));
                let icon_no = out_font_control_icon(code);
                if let Some(icon_no) = icon_no {
                    let width = self.icon_set(panes, icon_no, &mut line_text);
                    self.advance += width;
                }
                index += length.max(0) as usize;
            } else {
                let hi = (byte >> 4) & 0xF;
                if hi == 8 || hi == 9 {
                    if index + 1 >= message.len() {
                        break;
                    }
                    let trail = message[index + 1];
                    line_text.push(byte as char);
                    line_text.push(trail as char);
                    index += 2;
                    let code = ((byte as u16) << 8) | trail as u16;
                    self.advance += self.char_width(font, 0, code);
                } else {
                    line_text.push(byte as char);
                    index += 1;
                    if byte == b'\n' {
                        let shift = ((self.line_width - self.advance) / 2.0) as i32;
                        let prefix = if shift > 0 {
                            format!("\u{1b}CR[{shift}]")
                        } else {
                            String::new()
                        };
                        if shift > 0 {
                            for icon in 0..OUT_FONT_ICON_COUNT {
                                if line_no == self.icons[icon].line {
                                    self.icons[icon].x += shift as f32;
                                }
                            }
                        }
                        self.text.push_str(&prefix);
                        self.text.push_str(&line_text);
                        line_no += 1;
                        self.advance = 0.0;
                        line_text.clear();
                    } else {
                        self.advance += self.char_width(font, 0, u16::from(byte));
                    }
                }
            }
        }

        let shift_y = ((self.box_height - self.cell) - f32::from(line_no) * self.line_space) / 2.0;
        let shift = ((self.line_width - self.advance) / 2.0) as i32;
        let tail = if shift > 0 {
            format!("\u{1b}CR[{shift}]")
        } else {
            String::new()
        };
        if shift > 0 {
            for icon in 0..OUT_FONT_ICON_COUNT {
                if line_no == self.icons[icon].line {
                    self.icons[icon].x += shift as f32;
                }
                if self.icons[icon].line != -1 {
                    self.icons[icon].y +=
                        shift_y + f32::from(self.icons[icon].line) * self.line_space;
                }
            }
        }
        self.text.push_str(&tail);
        self.text.push_str(&line_text);
        if let Some(pane) = panes.get_mut(self.box_pane) {
            pane.shift_set(0.0, shift_y);
            let text = self.text.clone();
            pane.set_string(&text);
        }
    }

    pub fn move_out_font(&mut self, panes: &mut [Pane]) {
        for index in 0..usize::from(self.icon_count).min(OUT_FONT_ICON_COUNT) {
            let blend = self.animate(index, panes);
            let pane_index = self.pic[index];
            if let Some(pane) = panes.get_mut(pane_index) {
                pane.set_blend_ratio(blend[0], blend[1], blend[2], blend[3]);
            }
        }
        for index in 0..OUT_FONT_ICON_COUNT {
            if !self.icons[index].visible {
                continue;
            }
            let x = self.left_up_x + self.icons[index].x;
            let y = self.left_up_y + self.icons[index].y - 1.0;
            let rotation = self.icons[index].rotation;
            let size = F2 {
                x: self.icons[index].scale_x,
                y: self.icons[index].scale_y,
            };
            let texture = self.icons[index].texture;
            let icon_no = self.icons[index].icon_no;
            let alpha = self.pics[index].alpha;
            if let Some(pane) = panes.get_mut(self.pic[index]) {
                pane.pos_top_left.x = x;
                pane.pos_top_left.y = y;
                pane.size_orig = size;
                pane.size = size;
                pane.cpos_move();
                pane.rotation = rotation;
                pane.texture = Some(texture);
                pane.appended.clear();
                pane.now_alpha = alpha;
                pane.set_white(out_font_white(icon_no));
                pane.set_black(0);
            }
        }
        if let Some(pane) = panes.get_mut(self.box_pane) {
            pane.pos_top_left.x = self.left_up_x;
            pane.pos_top_left.y = self.left_up_y;
        }
    }

    fn animate(&mut self, index: usize, panes: &mut [Pane]) -> [f32; 4] {
        let icon = self.icons[index].icon_no;
        let period = stick_period();
        let timer = self.icons[index].timer;
        let span = |entry: &Self| (entry.icons[index].scale_x, entry.icons[index].scale_y);
        let (mut sx, mut sy) = span(self);
        let mut facing = 0.0f32;
        let mut cycles = 1i16;

        match icon {
            OUT_FONT_ICON_STICK => {
                cycles = 4;
                if timer < period {
                    self.icons[index].blend = stick_blend(timer, 0);
                    std::mem::swap(&mut sx, &mut sy);
                    self.icons[index].x += sx;
                    self.icons[index].y += sy;
                    facing = 3.0;
                } else if timer < period * 2 {
                    self.icons[index].blend = stick_blend(timer, 1);
                    std::mem::swap(&mut sx, &mut sy);
                    self.icons[index].y += sy;
                    facing = 3.0;
                } else if timer < period * 3 {
                    self.icons[index].blend = stick_blend(timer, 2);
                    self.icons[index].x += sx;
                    facing = 1.0;
                } else {
                    self.icons[index].blend = stick_blend(timer, 3);
                    self.icons[index].y += sy;
                    facing = 0.0;
                }
            }
            OUT_FONT_ICON_STICK_UP_DOWN => {
                cycles = 2;
                if timer < period {
                    self.icons[index].blend = stick_blend(timer, 0);
                    std::mem::swap(&mut sx, &mut sy);
                    self.icons[index].x += sx;
                    facing = 1.0;
                } else {
                    self.icons[index].blend = stick_blend(timer, 1);
                    self.icons[index].x += sx;
                    facing = 2.0;
                }
            }
            OUT_FONT_ICON_STICK_LEFT_RIGHT => {
                cycles = 2;
                if timer < period {
                    self.icons[index].blend = stick_blend(timer, 0);
                    std::mem::swap(&mut sx, &mut sy);
                    self.icons[index].x += sx;
                    self.icons[index].y += sy;
                    facing = 2.0;
                } else {
                    self.icons[index].blend = stick_blend(timer, 1);
                    self.icons[index].x += sx;
                    self.icons[index].y += sy;
                    facing = 2.0;
                }
            }
            OUT_FONT_ICON_STICK_UP => {
                if timer < period {
                    self.icons[index].blend = stick_blend(timer, 0);
                    self.icons[index].x += sx;
                    facing = 1.0;
                }
            }
            OUT_FONT_ICON_STICK_DOWN => {
                if timer < period {
                    self.icons[index].blend = stick_blend(timer, 0);
                    self.icons[index].y += sy;
                    facing = 3.0;
                }
            }
            OUT_FONT_ICON_STICK_LEFT => {
                if timer < period {
                    self.icons[index].blend = stick_blend(timer, 0);
                    self.icons[index].x += sx;
                    facing = 0.0;
                }
            }
            OUT_FONT_ICON_STICK_RIGHT => {
                if timer < period {
                    self.icons[index].blend = stick_blend(timer, 0);
                    self.icons[index].x += sx;
                    self.icons[index].y += sy;
                    facing = 2.0;
                }
            }
            _ => return self.icons[index].blend,
        }

        self.icons[index].scale_x = sx;
        self.icons[index].scale_y = sy;
        let angle = facing * -90.0;
        self.icons[index].rotation = deg2s(angle) as u16;
        if let Some(pane) = panes.get_mut(self.pic[index]) {
            pane.rotate_deg(0.0, 0.0, angle);
        }
        self.icons[index].timer += 1;
        if self.icons[index].timer >= period * cycles {
            self.icons[index].timer = 0;
        }
        self.icons[index].blend
    }

    pub fn set_alpha(&mut self, panes: &mut [Pane], alpha: u8) {
        self.alpha = alpha;
        for index in 0..OUT_FONT_ICON_COUNT {
            self.pics[index].set_alpha(alpha);
            if let Some(pane) = panes.get_mut(self.pic[index]) {
                pane.now_alpha = alpha;
            }
        }
        if let Some(pane) = panes.get_mut(self.box_pane) {
            pane.now_alpha = alpha;
        }
    }
}

pub const OUT_FONT_ICON_A: u8 = 0x00;
pub const OUT_FONT_ICON_B: u8 = 0x01;
pub const OUT_FONT_ICON_C: u8 = 0x02;
pub const OUT_FONT_ICON_L: u8 = 0x03;
pub const OUT_FONT_ICON_R: u8 = 0x04;
pub const OUT_FONT_ICON_X: u8 = 0x05;
pub const OUT_FONT_ICON_Y: u8 = 0x06;
pub const OUT_FONT_ICON_Z: u8 = 0x07;
pub const OUT_FONT_ICON_DPAD: u8 = 0x08;
pub const OUT_FONT_ICON_STICK: u8 = 0x09;
pub const OUT_FONT_ICON_LEFT: u8 = 0x0A;
pub const OUT_FONT_ICON_RIGHT: u8 = 0x0B;
pub const OUT_FONT_ICON_UP: u8 = 0x0C;
pub const OUT_FONT_ICON_DOWN: u8 = 0x0D;
pub const OUT_FONT_ICON_STICK_UP: u8 = 0x0E;
pub const OUT_FONT_ICON_STICK_DOWN: u8 = 0x0F;
pub const OUT_FONT_ICON_STICK_LEFT: u8 = 0x10;
pub const OUT_FONT_ICON_STICK_RIGHT: u8 = 0x11;
pub const OUT_FONT_ICON_STICK_UP_DOWN: u8 = 0x12;
pub const OUT_FONT_ICON_STICK_LEFT_RIGHT: u8 = 0x13;
pub const OUT_FONT_ICON_FLASHING_A: u8 = 0x14;
pub const OUT_FONT_ICON_HEART: u8 = 0x15;
pub const OUT_FONT_ICON_NOTE: u8 = 0x16;
pub const OUT_FONT_ICON_STARBURST: u8 = 0x17;

pub const OUT_FONT_TEXTURE: [&str; 27] = [
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
    "font_07_01.bti",
    "font_07_01.bti",
    "font_07_01.bti",
];

pub const OUT_FONT_WHITE: [[u8; 4]; 27] = [
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
    [0x00, 0xFF, 0xB4, 0xFF],
    [0xFF, 0x32, 0x32, 0xFF],
    [0xFF, 0xFF, 0xFF, 0xFF],
    [0xFF, 0xFF, 0xFF, 0xFF],
    [0xFF, 0xFF, 0xFF, 0xFF],
    [0xFF, 0xFF, 0xFF, 0xFF],
    [0xFF, 0xFF, 0xFF, 0xFF],
    [0xFF, 0xFF, 0xFF, 0xFF],
];

pub fn out_font_texture(icon: u8) -> &'static str {
    OUT_FONT_TEXTURE
        .get(usize::from(icon))
        .copied()
        .unwrap_or("")
}

pub fn out_font_white(icon: u8) -> u32 {
    let raw = OUT_FONT_WHITE
        .get(usize::from(icon))
        .copied()
        .unwrap_or([0xFF; 4]);
    pack_color(raw)
}

pub fn out_font_control_icon(code: u32) -> Option<u8> {
    const MAP: [(u32, u8); 24] = [
        (0x0A, OUT_FONT_ICON_A),
        (0x0B, OUT_FONT_ICON_B),
        (0x0C, OUT_FONT_ICON_C),
        (0x0D, OUT_FONT_ICON_L),
        (0x0E, OUT_FONT_ICON_R),
        (0x0F, OUT_FONT_ICON_X),
        (0x10, OUT_FONT_ICON_Y),
        (0x11, OUT_FONT_ICON_Z),
        (0x12, OUT_FONT_ICON_DPAD),
        (0x13, OUT_FONT_ICON_STICK),
        (0x14, OUT_FONT_ICON_LEFT),
        (0x15, OUT_FONT_ICON_RIGHT),
        (0x16, OUT_FONT_ICON_UP),
        (0x17, OUT_FONT_ICON_DOWN),
        (0x18, OUT_FONT_ICON_STICK_UP),
        (0x19, OUT_FONT_ICON_STICK_DOWN),
        (0x1A, OUT_FONT_ICON_STICK_LEFT),
        (0x1B, OUT_FONT_ICON_STICK_RIGHT),
        (0x1C, OUT_FONT_ICON_STICK_UP_DOWN),
        (0x1D, OUT_FONT_ICON_STICK_LEFT_RIGHT),
        (0x27, OUT_FONT_ICON_FLASHING_A),
        (0x39, OUT_FONT_ICON_HEART),
        (0x3A, OUT_FONT_ICON_NOTE),
        (0x2A, OUT_FONT_ICON_STARBURST),
    ];
    MAP.iter()
        .find(|(key, _)| *key == code)
        .map(|(_, icon)| *icon)
}

impl CollectScreen {
    pub fn item_name_set(
        &mut self,
        font: &mut FontBank,
        archive: &dyn MsgArchive,
        clear_count: u8,
        input: &CollectInput,
    ) {
        self.panes[P_NM01].set_font_size(
            self.panes[P_NM00].font_size_x,
            self.panes[P_NM00].font_size_y,
        );
        self.panes[P_NM01].set_char_space(self.panes[P_NM00].get_char_space());

        self.name_buf[1] = self.name_buf[0].clone();
        self.name_buf[0].clear();
        let text = self.name_buf[1].clone();
        self.panes[P_NM01].set_string(&text);

        self.panes[P_WD].set_font_size(NAME_CELL_SCALE, NAME_CELL_SCALE);

        let mut msg_no = ITEM_MESSAGE_NONE;
        if self.collect_mode != COLLECT_MODE_TRIFORCE {
            msg_no = self.name_message(input);
        } else {
            msg_no = u32::from(u16::from(self.now_item) + 0x1C2);
        }

        if msg_no == ITEM_MESSAGE_NONE {
            let name = self.name_buf[0].clone();
            self.panes[P_NM00].set_string(&name);
            return;
        }

        let mut get = ItemMsgGet::default();
        let Some(head) = get.get_mesg_header(archive, clear_count, msg_no) else {
            return;
        };
        let Some(message) = get.get_message(head) else {
            return;
        };
        let message = message.to_vec();

        let (mut size_x, size_y) = self.panes[P_NM00].get_font_size();
        size_x = size_y;
        let Some(font_ref) = font.get(0) else {
            return;
        };
        let scale = size_y / font_ref.cell_width() as f32;
        let mut str_width = 0.0f32;
        let mut first = false;
        let mut index = 0usize;
        let mut raw: Vec<u8> = Vec::new();
        while index < message.len() {
            let byte = message[index];
            if byte == 0 {
                break;
            }
            if byte == 0x1A {
                index += 1;
                let skip = i32::from(message.get(index).copied().unwrap_or(0)) - 1;
                index = index.wrapping_add(skip.max(0) as usize);
            } else {
                let hi = (byte >> 4) & 0xF;
                let (code, step) = if hi == 8 || hi == 9 {
                    let trail = message.get(index + 1).copied().unwrap_or(0);
                    (((byte as u16) << 8) | trail as u16, 2)
                } else {
                    (byte as u16, 1)
                };
                for slot in 0..step {
                    raw.push(message.get(index + slot).copied().unwrap_or(0));
                }
                index += step;
                let width = font_ref.char_width(code) as f32;
                str_width = if !first {
                    first = true;
                    scale * (width + font_ref.char_offset(code) as f32)
                } else {
                    str_width + width * scale
                };
            }
        }
        let text: String = raw.iter().map(|byte| *byte as char).collect();
        self.name_buf[0] = text;
        let box_width = self.panes[P_NM00].width();
        if box_width < str_width {
            size_x = size_x * (box_width / str_width);
        }
        self.panes[P_NM00].set_font_size(size_x, size_y);
        self.panes[P_NM00].set_char_space(0.0);
        let name = self.name_buf[0].clone();
        self.panes[P_NM00].set_string(&name);

        let msg = match self.now_item {
            0..=5 => MSG_OUT_FONT_SONG,
            6 => MSG_OUT_FONT_SAVE,
            7 => MSG_OUT_FONT_SONG_FALLBACK,
            0x12 => MSG_OUT_FONT_BOTTLE,
            _ => 0,
        };
        if msg != 0 {
            self.out_font
                .message_set(&mut self.panes, font, archive, clear_count, msg);
            let pos = self.panes[P_WD].pos_top_left;
            self.out_font.set_left_up_pos(pos.x, pos.y);
        }
    }

    fn name_message(&self, input: &CollectInput) -> u32 {
        let collect = &input.collect;
        match self.now_item {
            0 => {
                if collect.is_tact(0) {
                    return item_message(u8::try_from(MSG_TACT_ITEM[0]).unwrap_or(0));
                }
                0
            }
            1 => {
                if collect.is_tact(1) {
                    return item_message(u8::try_from(MSG_TACT_ITEM[1]).unwrap_or(0));
                }
                0
            }
            2 => {
                if collect.is_tact(2) {
                    return item_message(u8::try_from(MSG_TACT_ITEM[2]).unwrap_or(0));
                }
                0
            }
            3 => {
                if collect.is_tact(3) {
                    return item_message(u8::try_from(MSG_TACT_ITEM[3]).unwrap_or(0));
                }
                0
            }
            4 => {
                if collect.is_tact(4) {
                    return item_message(u8::try_from(MSG_TACT_ITEM[4]).unwrap_or(0));
                }
                0
            }
            5 => {
                if collect.is_tact(5) {
                    return item_message(u8::try_from(MSG_TACT_ITEM[5]).unwrap_or(0));
                }
                0
            }
            6 => MSG_SAVE_WORD,
            7 => MSG_SONG_WORD,
            8 => {
                if input.life() % 4 != 0 {
                    item_message(u8::try_from(MSG_HEART_ITEM).unwrap_or(0))
                } else {
                    0
                }
            }
            9 => {
                if input.map_open() {
                    MSG_MAP_WORD
                } else {
                    0
                }
            }
            10 => match triforce_num(collect) {
                8 => MSG_TRIFORCE_FULL,
                0 => 0,
                _ => item_message(u8::try_from(MSG_TRIFORCE_ITEM).unwrap_or(0)),
            },
            0x0B => {
                if collect.is_symbol(0) {
                    item_message(u8::try_from(MSG_SYMBOL_ITEM[0]).unwrap_or(0))
                } else {
                    0
                }
            }
            0x0C => {
                if collect.is_symbol(1) {
                    item_message(u8::try_from(MSG_SYMBOL_ITEM[1]).unwrap_or(0))
                } else {
                    0
                }
            }
            0x0D => {
                if collect.is_symbol(2) {
                    item_message(u8::try_from(MSG_SYMBOL_ITEM[2]).unwrap_or(0))
                } else {
                    0
                }
            }
            0x0E | 0x0F | 0x10 => item_message(input.equip_item(self.now_item - 0x0E)),
            0x11 => {
                if collect.test(ITEM_HOOD, 0) {
                    item_message(u8::try_from(MSG_HOOD_ITEM).unwrap_or(0))
                } else {
                    0
                }
            }
            0x12 => {
                if collect.test(ITEM_BOTTLE, 0) {
                    item_message(u8::try_from(MSG_BOTTLE_ITEM).unwrap_or(0))
                } else {
                    0
                }
            }
            0x13 => {
                if (0..SONG_COUNT).all(|index| !collect.is_tact(index as u8)) {
                    0
                } else {
                    0
                }
            }
            0x14 => {
                if (0..3u8).all(|index| !collect.is_symbol(index)) {
                    0
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    pub fn item_note_set(
        &mut self,
        font: &mut FontBank,
        archive: &dyn MsgArchive,
        clear_count: u8,
        input: &CollectInput,
        _tune: &CollectTune,
    ) {
        if input.opt_ruby {
            self.panes[P_ST00].pane_trans(0.0, RUBY_OPTION_DY);
        } else {
            self.panes[P_ST00].pane_trans(0.0, 0.0);
        }

        self.note_buf = [String::new(), String::new()];
        self.dummy_buf = [String::new(), String::new()];

        self.out_font_init();

        if input.opt_ruby {
            self.panes[P_STR0].hide();
        } else {
            self.panes[P_STR0].show();
        }

        let ruby_font_size = self.panes[P_STR0].font_size_x;
        self.panes[P_ST00].set_font_size(MSG_FONT_SIZE as f32, MSG_FONT_SIZE as f32);
        self.panes[P_ST00].set_line_space(MSG_LINE_SPACE as f32);

        let msg_no = self.note_message(input);
        if msg_no == ITEM_MESSAGE_NONE {
            return;
        }

        let mut get = ItemMsgGet::default();
        let Some(head) = get.get_mesg_header(archive, clear_count, msg_no) else {
            return;
        };
        let Some(message) = get.get_message(head) else {
            return;
        };
        let message = message.to_vec();
        let entry: MesgEntry = get.get_mesg_entry(head).unwrap_or_default();
        self.msg_entry = entry;

        self.note_proc = MsgData::new_data_proc();
        self.note_proc.set_bmg(message);
        self.note_proc.entry = self.msg_entry;
        self.note_proc.out = [
            Some(String::new()),
            Some(String::new()),
            Some(String::new()),
            Some(String::new()),
        ];
        self.note_proc.char_space = self.panes[P_ST00].get_char_space() as i32;
        self.note_proc.ruby_char_space = self.panes[P_STR0].get_char_space() as i32;
        self.note_proc.line_space = self.panes[P_ST00].get_line_space() as i32;
        self.note_proc.font_size = MSG_FONT_SIZE;
        self.note_proc.ruby_font_size = ruby_font_size as i32;
        self.note_proc.line_width = NOTE_LINE_WIDTH;
        self.note_proc.center_line_width = NOTE_CENTER_LINE_WIDTH;
        self.note_proc.send_speed = NOTE_SEND_SPEED;
        self.note_proc.space_timer = NOTE_SPACE_TIMER;
        self.note_proc.short_cut();
        self.note_proc.space_flag = 0;
        let state = MsgLenState {
            clear_count,
            ..Default::default()
        };
        self.note_proc.string_length(font, state);
        self.note_proc.string_shift();
        self.note_proc.icon_idx_refresh();

        let line_count = self.note_proc.line_count;
        self.note_proc.line_count = 0;
        let line_space = self.panes[P_ST00].get_line_space();
        let unused = NOTE_UNUSED_LINES - line_count;
        let shift_y = unused as f32 * (line_space / 2.0);
        self.panes[P_STR0].shift_set(0.0, shift_y);
        self.panes[P_ST00].shift_set(0.0, shift_y);
        self.note_proc.string_set(font, &MsgLayoutState::default());
        let note = self.note_proc.out[0].clone().unwrap_or_default();
        let ruby = self.note_proc.out[1].clone().unwrap_or_default();
        self.note_buf = [note.clone(), note];
        self.panes[P_ST00].set_string(&self.note_buf[0]);
        self.panes[P_STR0].set_string(&self.note_buf[1]);

        let half_line = self.panes[P_ST00].get_line_space() / 2.0;
        for index in 0..NOTE_ICON_COUNT {
            let icon_no = self.note_proc.icon_num[index];
            let mut icon_color = self.note_proc.icon_color[index];
            if icon_color == 0xFFFF_FFFF {
                icon_color = 0xFF;
            }
            if icon_no == ICON_NONE || self.panes[P_FT0 + index].user_area != -1 {
                continue;
            }
            if icon_no == 0x14 {
                if self.select_flag == 0 {
                    self.icon_slot = index as i16;
                    self.panes[P_FT0 + index as usize].user_area = 0;
                }
                let slot = self.icon_slot as usize;
                self.panes[P_FT0 + slot].pos_center.x = self.note_proc.icon_pos_x[index] as f32;
                self.panes[P_FT0 + slot].pos_center.y = half_line
                    * ((unused as f32) + (self.note_proc.icon_pos_y[index as usize] as f32) * 2.0);
                self.select_flag = 1;
            } else if icon_no == 0x15 {
                if self.select_flag == 0 {
                    self.select_flag = 1;
                    self.select_index = 0;
                    self.icon_slot = index as i16;
                    self.panes[P_FT0 + index as usize].user_area = 0;
                } else {
                    self.select_index = 1;
                }
                let slot = self.icon_slot as usize;
                self.panes[P_FT0 + slot].pos_top_left.x = self.note_proc.icon_pos_x[index] as f32;
                self.panes[P_FT0 + slot].pos_top_left.y =
                    half_line * ((unused as f32) + (self.note_proc.icon_pos_y[index] as f32) * 2.0);
                self.panes[P_FT0 + slot].blend_draw(SELECT_ARROW_TEX);
                self.panes[P_FT0 + slot].set_blend_color_ratio(0.0, 1.0, 1.0, 1.0);
                self.panes[P_FT0 + slot].set_blend_alpha_ratio(0.0, 1.0, 1.0, 1.0);
                if self.now_item == 0x12 {
                    if input.collect.test(ITEM_BOTTLE, 1) {
                        self.select_index = 0;
                        self.note_proc.field_0x27d = 0;
                    } else {
                        self.select_index = 1;
                        self.note_proc.field_0x27d = 1;
                    }
                }
                self.last_select = self.select_index;
            } else if icon_no != 0x16 {
                self.panes[P_FT0 + index].pos_top_left.x = self.note_proc.icon_pos_x[index] as f32;
                self.panes[P_FT0 + index].pos_top_left.y =
                    half_line * ((unused as f32) + (self.note_proc.icon_pos_y[index] as f32) * 2.0);
                self.panes[P_FT0 + index].pos_top_left_orig.y = f32::from(icon_no);
                let mut timer = self.panes[P_FT0 + index].user_area;
                fop_msgm_out_font_set1(&mut self.note[index].pic, &mut timer, icon_color, icon_no);
                self.note[index].push(&mut self.panes);
                self.panes[P_FT0 + index].user_area = timer;
            }
        }
    }

    fn note_message(&self, input: &CollectInput) -> u32 {
        let collect = &input.collect;
        let msg = match self.now_item {
            0 => {
                if collect.is_tact(0) {
                    item_message(u8::try_from(MSG_TACT_NOTE_ITEM[0]).unwrap_or(0))
                        + NOTE_MESSAGE_BASE
                } else {
                    0
                }
            }
            1 => {
                if collect.is_tact(1) {
                    item_message(u8::try_from(MSG_TACT_NOTE_ITEM[1]).unwrap_or(0))
                        + NOTE_MESSAGE_BASE
                } else {
                    0
                }
            }
            2 => {
                if collect.is_tact(2) {
                    item_message(u8::try_from(MSG_TACT_NOTE_ITEM[2]).unwrap_or(0))
                        + NOTE_MESSAGE_BASE
                } else {
                    0
                }
            }
            3 => {
                if collect.is_tact(3) {
                    item_message(u8::try_from(MSG_TACT_NOTE_ITEM[3]).unwrap_or(0))
                        + NOTE_MESSAGE_BASE
                } else {
                    0
                }
            }
            4 => {
                if collect.is_tact(4) {
                    item_message(u8::try_from(MSG_TACT_NOTE_ITEM[4]).unwrap_or(0))
                        + NOTE_MESSAGE_BASE
                } else {
                    0
                }
            }
            5 => {
                if collect.is_tact(5) {
                    item_message(u8::try_from(MSG_TACT_NOTE_ITEM[5]).unwrap_or(0))
                        + NOTE_MESSAGE_BASE
                } else {
                    0
                }
            }
            6 => MSG_SAVE_NOTE,
            7 => MSG_SONG_NOTE,
            8 => {
                if input.life() % 4 != 0 {
                    item_message(u8::try_from(MSG_HEART_ITEM).unwrap_or(0)) + NOTE_MESSAGE_BASE
                } else {
                    0
                }
            }
            9 => {
                if input.map_open() {
                    MSG_MAP_NOTE
                } else {
                    0
                }
            }
            10 => {
                let count = triforce_num(collect);
                if count == 8 {
                    MSG_TRIFORCE_NOTE
                } else if count > 0 {
                    item_message(u8::try_from(MSG_TRIFORCE_ITEM).unwrap_or(0)) + NOTE_MESSAGE_BASE
                } else {
                    0
                }
            }
            0x0B => {
                if collect.is_symbol(0) {
                    item_message(u8::try_from(MSG_SYMBOL_ITEM[0]).unwrap_or(0)) + NOTE_MESSAGE_BASE
                } else {
                    0
                }
            }
            0x0C => {
                if collect.is_symbol(1) {
                    item_message(u8::try_from(MSG_SYMBOL_ITEM[1]).unwrap_or(0)) + NOTE_MESSAGE_BASE
                } else {
                    0
                }
            }
            0x0D => {
                if collect.is_symbol(2) {
                    item_message(u8::try_from(MSG_SYMBOL_ITEM[2]).unwrap_or(0)) + NOTE_MESSAGE_BASE
                } else {
                    0
                }
            }
            0x0E | 0x0F | 0x10 => {
                let base = item_message(input.equip_item(self.now_item - 0x0E));
                if base == 0 {
                    MSG_NO_EQUIP
                } else {
                    base + NOTE_MESSAGE_BASE
                }
            }
            0x11 => {
                if collect.test(ITEM_HOOD, 0) {
                    item_message(u8::try_from(MSG_HOOD_ITEM).unwrap_or(0)) + NOTE_MESSAGE_BASE
                } else {
                    0
                }
            }
            0x12 => {
                if collect.test(ITEM_BOTTLE, 0) {
                    item_message(u8::try_from(MSG_BOTTLE_ITEM).unwrap_or(0)) + NOTE_MESSAGE_BASE
                } else {
                    0
                }
            }
            0x13 => {
                if (0..SONG_COUNT).all(|index| !collect.is_tact(index as u8)) {
                    0
                } else {
                    MSG_NO_EQUIP_NOTE
                }
            }
            0x14 => {
                if (0..3u8).all(|index| !collect.is_symbol(index)) {
                    0
                } else {
                    MSG_NO_EQUIP_NOTE
                }
            }
            _ => 0,
        };
        msg
    }

    pub fn item_set(&mut self, collect: &Collect) {
        for level in (0..4u8).rev() {
            if collect.test(ITEM_SWORD, level) {
                self.panes[P_WIT].change_texture(SWORD_TEX[usize::from(level)]);
                self.panes[P_WIK].change_texture(SWORD_TEX[usize::from(level)]);
                break;
            }
        }
        for level in (0..2u8).rev() {
            if collect.test(ITEM_SHIELD, level) {
                self.panes[P_WIT].change_texture(SHIELD_TEX[usize::from(level)]);
                self.panes[P_WIK].change_texture(SHIELD_TEX[usize::from(level)]);
                break;
            }
        }
    }

    pub fn out_font_init(&mut self) {
        for index in 0..NOTE_ICON_COUNT {
            self.panes[P_FT0 + index].user_area = -1;
            self.panes[P_FT0 + index].rotate(0.0, 0.0, 0.0);
            self.panes[P_FT0 + index].set_now_alpha_zero();
        }
        self.icon_slot = -1;
        self.select_flag = 0;
        self.select_index = 0;
    }

    pub fn out_font_move(&mut self) {
        for index in 0..NOTE_ICON_COUNT {
            if self.panes[P_FT0 + index].user_area != -1 {
                let box_pos = self.panes[P_ST00].pos_top_left;
                let anchor = self.panes[P_NO11].pos_top_left;
                let frame = self.panes[P_NT00].pos_top_left;
                let x =
                    self.panes[P_FT0 + index].pos_top_left.x + (box_pos.x + (anchor.x + frame.x));
                let y =
                    self.panes[P_FT0 + index].pos_top_left.y + (box_pos.y + (anchor.y + frame.y));
                self.panes[P_FD0 + index].pos_top_left.x = x;
                self.panes[P_FD0 + index].pos_top_left.y = y;

                if index as i16 == self.icon_slot && self.select_index == 0 {
                    self.panes[P_FD0 + index].pos_top_left.x -=
                        self.panes[P_FT0 + index].pos_top_left.x
                            - self.panes[P_FT0 + index].pos_center.x;
                }

                let rot_x = self.panes[P_NO11].pos_center.x - x;
                let rot_y = self.panes[P_NO11].pos_center.y - y;
                let angle = self.panes[P_NO11].get_rotate();
                self.panes[P_FD0 + index].rotate(rot_x, rot_y, angle);
            } else {
                self.panes[P_FT0 + index].set_now_alpha_zero();
            }
        }
    }

    pub fn out_font_draw(&mut self, font_ctx: &MeterMsgHio) {
        for index in 0..NOTE_ICON_COUNT {
            if self.panes[P_FT0 + index].user_area == -1 {
                continue;
            }
            let alpha = self.panes[P_ST00].get_alpha();
            self.panes[P_FT0 + index].now_alpha = alpha;
            if index as i16 == self.icon_slot {
                let size = self.panes[P_FT0 + index].size_orig;
                let color = select_arrow_anime(&mut self.arrow_timer);
                self.panes[P_FT0 + index].rotate(size.x / 2.0, size.y / 2.0, SELECT_ARROW_ANGLE);
                self.panes[P_FT0 + index].set_white(pack_color(color.0));
                self.panes[P_FT0 + index].set_black(pack_color(color.1));
                self.panes[P_FT0 + index].now_alpha = color.2;
                self.panes[P_FD0 + index].move_to(
                    self.panes[P_FD0 + index].pos_top_left.x,
                    self.panes[P_FD0 + index].pos_top_left.y,
                );
            } else {
                self.note[index].pull(&mut self.panes);
                let mut timer = self.panes[P_FT0 + index].user_area;
                let mut x = self.panes[P_FD0 + index].pos_top_left.x as i32;
                let mut y = self.panes[P_FD0 + index].pos_top_left.y as i32;
                let mut w = MSG_FONT_SIZE;
                let mut h = MSG_FONT_SIZE;
                let icon_no = self.panes[P_FT0 + index].pos_top_left_orig.y as u8;
                fop_msgm_out_font_draw2(
                    font_ctx,
                    &mut self.note[index].pic,
                    &mut self.note[index].shd,
                    x,
                    y,
                    w,
                    h,
                    &mut timer,
                    alpha,
                    icon_no,
                );
                x += 2;
                y += 1;
                w = w.max(1);
                h = h.max(1);
                self.note[index].pic.move_to(x as f32, y as f32);
                self.note[index].pic.resize(w as f32, h as f32);
                self.note[index].pic.set_alpha(alpha);
                self.note[index].push(&mut self.panes);
                self.panes[P_FT0 + index].user_area = timer;
            }
        }
    }

    pub fn collect_move(
        &mut self,
        input: &mut CollectInput,
        pad: &PadInput,
        out: &mut CollectOutput,
        tune: &CollectTune,
        font: &mut FontBank,
        archive: &dyn MsgArchive,
        clear_count: u8,
    ) {
        self.item_bit_check(input, tune);

        if self.triforce_count == 8 {
            self.triforce_anime(tune, tune.triforce_alpha);
        }

        if self.collect_mode == COLLECT_MODE_BROWSE {
            if !self.note_check() {
                if !pad.start && !pad.lock_l && !pad.lock_r {
                    if pad.a {
                        if self.now_item <= 5 && input.collect.is_tact(self.now_item) {
                            self.collect_mode = COLLECT_MODE_DEMO;
                            self.panes[P_TIK].user_area = 0;
                            self.tact_guide_hide();
                            for index in 0..SONG_COUNT {
                                if (index as i16) < TACT_BEAT[usize::from(self.now_item)] {
                                    self.panes[P_HMB + index].show();
                                }
                            }
                            out.melody_play = Some(self.now_item);
                            out.sound(SE_DANCE_01 + u32::from(self.now_item));
                        } else if self.now_item == 6 {
                            self.collect_mode = COLLECT_MODE_OPTION;
                            out.sub_screen = Some(SubScreen::Option);
                            out.sound(SE_OPT_IN);
                        } else if self.now_item == 7 {
                            self.collect_mode = COLLECT_MODE_SAVE;
                            out.sub_screen = Some(SubScreen::Save);
                            out.sound(SE_OPT_IN);
                        } else if self.now_item == 0x11 {
                            if input.collect.test(ITEM_HOOD, 0) {
                                input.collect.set(ITEM_HOOD, 1);
                                out.on_collect.push((ITEM_HOOD, 1));
                            }
                        }

                        let in_range = (self.now_item >= 0x0A && self.now_item <= 0x12)
                            || (self.now_item == 8 && input.life() % 4 != 0)
                            || (self.now_item == 9 && input.map_open());
                        if in_range && self.collect_item_get_check(input, self.now_item) {
                            for index in [P_STR0, P_ST00, P_NT00, P_NK00, P_NO11] {
                                self.panes[index].set_init_alpha();
                            }
                            self.panes[P_NK00].user_area = 1;
                            self.panes[P_NT00].user_area = 1;
                            self.item_note_set(font, archive, clear_count, input, tune);
                            out.sound(SE_EXP_OPEN);
                        }
                    } else if !pad.b {
                        self.cursor_main_move(input, pad, out);
                        self.item_scale();
                    }
                }
            } else {
                self.note_appear(input, pad, out);
            }

            if self.collect_item_get_check(input, self.now_item) {
                self.item_name_move();
            } else {
                self.clear_name_alpha();
            }

            self.out_font_move();
        } else if self.collect_mode == COLLECT_MODE_DEMO {
            self.tact_demo_mode(input, pad, out);
            self.item_name_move();
        } else if self.collect_mode == COLLECT_MODE_PLAY {
            self.tact_play_mode(input, pad, out, tune);
            self.item_name_move();
        } else if self.collect_mode == COLLECT_MODE_SAVE {
            out.a_status_force = Some(ACT_RETURN);
        } else if self.collect_mode == COLLECT_MODE_OPTION {
            out.a_status_force = Some(ACT_RETURN);
        }

        if self.collect_mode != COLLECT_MODE_OPTION && self.collect_mode != COLLECT_MODE_SAVE {
            self.cursor_anime(input.map_open());
            self.corner_move(tune);
        }
    }

    pub fn collect_move_triforce(&mut self, tune: &CollectTune, out: &mut CollectOutput) {
        let step1 = tune.warp_frames;
        let step2 = tune.hold_frames;
        let step3 = tune.name_frames;
        let step4 = tune.pulse_period;
        let t1 = step1;
        let t2 = t1 + step2;
        let t3 = t2 + step3;
        let t4 = t3 + step4;

        self.timer += 1;

        if self.timer <= t1 {
            if self.timer == 1 {
                out.sound(SE_TRIFORCE_BECOME_ONE);
            }
            self.anime_step1(step1, self.timer, out);
        } else if self.timer <= t2 {
            self.anime_step2(step2, self.timer - t1, tune);
        } else if self.timer <= t3 {
            self.anime_step3(step3, self.timer - t2, out);
        } else if self.timer <= t4 {
            self.anime_step4(step4, self.timer - t3, tune);
            if self.timer == t4 {
                self.collect_mode = COLLECT_MODE_BROWSE;
            }
        }
    }

    pub fn anime_step1(&mut self, step_max: i16, step: i16, out: &mut CollectOutput) {
        let alpha_scale = value_increase(i32::from(step_max), i32::from(step), 0);
        self.panes[P_TRIL].now_alpha = (alpha_scale * 255.0) as u8;
        self.panes[P_TRIM].now_alpha = (255.0 * alpha_scale) as u8;

        if step_max - 8 == step {
            self.spawn_emitter(2, P_TRI0 + 7);
        } else if step_max - 4 == step {
            self.spawn_emitter(1, P_TRI0 + 1);
        }

        if step_max == step {
            for index in 0..TRIFORCE_SLOT_COUNT {
                self.set_show(P_TRI0 + index, index == 4);
            }
            self.panes[P_TRI0 + 4].change_texture(TRIFORCE_FULL_TEX);
            self.panes[P_TRI0 + 4].pos_center_orig.x = self.panes[P_TRIB].pos_center_orig.x;
            self.panes[P_TRI0 + 4].pos_center_orig.y = self.panes[P_TRIB].pos_center_orig.y;
            self.panes[P_TRI0 + 4].pos_center.x = self.panes[P_TRIB].pos_center.x;
            self.panes[P_TRI0 + 4].pos_center.y = self.panes[P_TRIB].pos_center.y;
            self.panes[P_TRI0 + 4].size.x = TRIFORCE_FULL_WIDTH * TRIFORCE_FULL_SCALE;
            self.panes[P_TRI0 + 4].size.y = TRIFORCE_FULL_HEIGHT * TRIFORCE_FULL_SCALE;
            self.panes[P_TRI0 + 4].cpos_move();
            self.spawn_emitter(0, P_TRI0 + 6);
        }
    }

    fn spawn_emitter(&mut self, index: usize, pane: usize) {
        if self.emitters[index].spawned {
            return;
        }
        self.emitters[index].spawned = true;
        self.emitters[index].alpha = 1.0;
        self.emitters[index].resource = PARTICLE_RUPY_LIGHT;
        self.emitters[index].x = self.panes[pane].pos_center.x - CENTER_X;
        self.emitters[index].y = self.panes[pane].pos_center.y - CENTER_Y;
        self.emitters[index].z = 0.0;
    }

    pub fn anime_step2(&mut self, step_max: i16, step: i16, tune: &CollectTune) {
        let t_in = value_increase(i32::from(step_max), i32::from(step), 0);
        let t_out = value_increase(i32::from(step_max), i32::from(step_max - step), 2);
        let tune_scale = tune.scale;
        self.panes[P_TRIM].pane_scale_xy((tune_scale - 1.0) * t_in + 1.0);
        self.panes[P_TRIM].now_alpha = (255.0 * t_out) as u8;
        if step_max == step {
            self.now_item = 2;
            self.panes[P_NM00].set_now_alpha_zero();
            self.panes[P_NM01].set_init_alpha();
        }
    }

    pub fn anime_step3(&mut self, step_max: i16, step: i16, _out: &mut CollectOutput) {
        let alpha_t = value_increase(i32::from(step_max), i32::from(step_max - step), 1);
        let name_t = value_increase(i32::from(step_max), i32::from(step), 2);
        self.panes[P_TRIL].now_alpha = (alpha_t * 255.0) as u8;
        self.panes[P_NM00].set_now_alpha(name_t);
        self.panes[P_NM01].set_now_alpha(1.0 - name_t);

        for index in 0..EMITTER_COUNT {
            let pane = P_TRI0 + EMITTER_SOURCE_PANE[index];
            let x = self.panes[pane].pos_center.x - CENTER_X;
            let y = self.panes[pane].pos_center.y - CENTER_Y;
            if self.emitters[index].spawned {
                self.emitters[index].x = x;
                self.emitters[index].y = y;
                self.emitters[index].z = 0.0;
                self.emitters[index].playing = true;
                self.emitters[index].alpha = 1.0;
            }
        }
    }

    pub fn anime_step4(&mut self, step_max: i16, step: i16, tune: &CollectTune) {
        let quarter = step_max / 4;
        if step == quarter {
            self.emitters[0].alpha = 0.0;
        } else if step == quarter * 2 {
            self.emitters[1].alpha = 0.0;
        } else if step == quarter * 3 {
            self.emitters[2].alpha = 0.0;
        }
        self.triforce_anime(tune, tune.triforce_fade_alpha);
    }

    pub fn collect_open(
        &mut self,
        font: &mut FontBank,
        archive: &dyn MsgArchive,
        clear_count: u8,
        input: &CollectInput,
        out: &mut CollectOutput,
        tune: &CollectTune,
    ) -> bool {
        if self.timer == 0 {
            self.note_reset();
            self.item_name_set(font, archive, clear_count, input);
            self.item_note_set(font, archive, clear_count, input, tune);
            self.item_set(&input.collect);
            self.apply_status(input, out);
        }
        self.timer += 1;
        if self.timer <= MODE_SKIP_FRAMES {
            let offset = if self.trigger_info == TRIGGER_RIGHT {
                -100
            } else {
                100
            };
            self.main_open_proc(input, self.timer, MODE_SKIP_FRAMES, offset);
            self.sub_open_proc(self.timer, MODE_SKIP_FRAMES, offset);
            self.title_open_proc(self.timer, MODE_SKIP_FRAMES, out);
            self.note_open_proc(self.timer, MODE_SKIP_FRAMES);
            self.name_open_proc(self.timer, MODE_SKIP_FRAMES);
        }
        let mut done = false;
        if self.timer >= MODE_SKIP_FRAMES {
            done = true;
            out.sound(SE_ITEMS_IN);
        }
        done
    }

    pub fn collect_open_page(
        &mut self,
        font: &mut FontBank,
        archive: &dyn MsgArchive,
        clear_count: u8,
        input: &CollectInput,
        out: &mut CollectOutput,
    ) -> bool {
        if self.timer == 0 {
            self.note_reset();
            self.item_name_set(font, archive, clear_count, input);
            self.item_note_set(font, archive, clear_count, input, &CollectTune::default());
            self.item_set(&input.collect);
            self.apply_status(input, out);
        }
        self.timer += 1;
        if self.timer <= CURSOR_OPEN_FRAMES {
            let offset = if self.trigger_info == TRIGGER_RIGHT {
                -100
            } else {
                100
            };
            self.main_open_proc(input, self.timer, CURSOR_OPEN_FRAMES, offset);
            self.sub_open_proc(self.timer, CURSOR_OPEN_FRAMES, offset);
            self.title_open_proc(self.timer, CURSOR_OPEN_FRAMES, out);
            self.note_open_proc(self.timer, CURSOR_OPEN_FRAMES);
            self.name_open_proc(self.timer, CURSOR_OPEN_FRAMES);
        }
        self.timer >= CURSOR_OPEN_FRAMES
    }

    pub fn collect_open_triforce(
        &mut self,
        font: &mut FontBank,
        archive: &dyn MsgArchive,
        clear_count: u8,
        input: &CollectInput,
        tune: &CollectTune,
        out: &mut CollectOutput,
    ) -> bool {
        if self.timer == 0 {
            self.note_reset();
            self.now_item = 1;
            self.item_name_set(font, archive, clear_count, input);
            self.item_note_set(font, archive, clear_count, input, tune);
            self.item_set(&input.collect);
            out.do_status_force = Some(ACT_HIDDEN);
            out.a_status_force = Some(ACT_HIDDEN);
        }
        self.timer += 1;
        if self.timer <= MODE_SKIP_FRAMES {
            self.main_open_proc(input, self.timer, MODE_SKIP_FRAMES, 100);
            self.sub_open_proc(self.timer, MODE_SKIP_FRAMES, 100);
            self.title_open_proc(self.timer, MODE_SKIP_FRAMES, out);
            self.note_open_proc(self.timer, MODE_SKIP_FRAMES);
            self.name_open_proc(self.timer, MODE_SKIP_FRAMES);
        }
        let mut done = false;
        if self.timer >= MODE_SKIP_FRAMES {
            self.timer = 0;
            done = true;
            out.sound(SE_ITEMS_IN);
        }
        done
    }

    fn apply_status(&mut self, input: &CollectInput, out: &mut CollectOutput) {
        if self.now_item <= 7 {
            if self.collect_item_get_check(input, self.now_item) {
                out.do_status_force = Some(ACT_CHOOSE);
            } else {
                out.clear_do();
            }
        } else if self.collect_item_get_check(input, self.now_item) {
            out.do_status_force = Some(ACT_INFO);
        } else {
            out.clear_do();
        }
    }

    pub fn note_reset(&mut self) {
        for index in 0..NAME_BUFFER_COUNT {
            self.name_buf[index].clear();
            self.note_buf[index].clear();
            let text = String::new();
            self.panes[P_NM00 + index].set_string(&text);
        }
        let empty = String::new();
        self.panes[P_ST00].set_string(&empty);
        self.panes[P_STR0].set_string(&empty);
    }

    pub fn collect_close(&mut self, input: &CollectInput, out: &mut CollectOutput) -> bool {
        self.close_common(input, out, MODE_SKIP_FRAMES)
    }

    pub fn collect_close_page(&mut self, input: &CollectInput, out: &mut CollectOutput) -> bool {
        self.close_common(input, out, CURSOR_OPEN_FRAMES)
    }

    fn close_common(&mut self, input: &CollectInput, out: &mut CollectOutput, span: i16) -> bool {
        self.timer -= 1;
        let t = value_increase(i32::from(span), i32::from(span - self.timer), 0);
        let main = -MAIN_SHIFT * t;
        let title = -MAIN_SHIFT * t;
        let name = NAME_SHIFT * t;

        let base = f32::from(self.panes[P_NO11].user_area);
        let angle = base + (NOTE_SWING - base) * t;
        let rotation = NOTE_ROTATE * t;

        if self.trigger_info == TRIGGER_RIGHT {
            self.main_trans(input, -main, 0.0);
            self.sub_trans(-main, 0.0);
            self.title_trans(0.0, title);
            self.note_rotate(rotation, angle);
            self.name_trans(0.0, name);
        } else if self.trigger_info == TRIGGER_LEFT {
            self.main_trans(input, main, 0.0);
            self.sub_trans(main, 0.0);
            self.title_trans(0.0, title);
            self.note_rotate(rotation, angle);
            self.name_trans(0.0, name);
        }

        if self.timer > 0 {
            self.fade_flag = 1;
            false
        } else {
            self.fade_flag = 0;
            out.do_status = Some(ACT_BLANK);
            self.zero_all_alpha();
            true
        }
    }

    fn zero_all_alpha(&mut self) {
        for index in 0..NOTE_ICON_COUNT {
            self.panes[P_FT0 + index].set_now_alpha_zero();
        }
        for index in [
            P_STR0, P_ST00, P_NT00, P_NK00, P_NO11, P_WD, P_NM00, P_NM01, P_ITNM, P_ITNK, P_MSK0,
            P_TL00, P_TK00,
        ] {
            self.panes[index].set_now_alpha_zero();
        }
        for index in 0..CURSOR_CORNER_COUNT {
            self.panes[P_CAR0 + index].set_now_alpha_zero();
        }
        self.panes[P_WDSV].set_now_alpha_zero();
        self.panes[P_WDOP].set_now_alpha_zero();
        for index in 0..4 {
            self.panes[P_CORNER0 + index].set_now_alpha_zero();
        }
        for index in [P_NUM1, P_NUM2, P_NUMC, P_CMAP, P_WK1, P_WK0, P_TRIM, P_TRIL] {
            self.panes[index].set_now_alpha_zero();
        }
        for index in 0..TRIFORCE_SLOT_COUNT {
            self.panes[P_TRI0 + index].set_now_alpha_zero();
        }
        self.panes[P_TRIB].set_now_alpha_zero();
        for index in 0..3 {
            self.panes[P_SIT + index].set_now_alpha_zero();
            self.panes[P_SIK + index].set_now_alpha_zero();
            self.panes[P_HTP + index].set_now_alpha_zero();
        }
        for index in 0..11 {
            self.panes[P_CI + index].set_now_alpha_zero();
        }
        self.panes[P_HTPB].set_now_alpha_zero();
        for index in 0..SONG_COUNT {
            for base in [P_TIT, P_TIK, P_CN, P_WN, P_I0, P_I1, P_BS, P_HMB] {
                self.panes[base + index].set_now_alpha_zero();
            }
        }
        for index in [P_I111, P_I122, P_CC43, P_CC32, P_CC21] {
            self.panes[index].set_now_alpha_zero();
        }
        for index in 0..5 {
            self.panes[P_WIT + index].set_now_alpha_zero();
            self.panes[P_WIK + index].set_now_alpha_zero();
            self.panes[P_IP + index].set_now_alpha_zero();
        }
        for index in [P_IP00, P_CC14, P_CC07, P_CC00] {
            self.panes[index].set_now_alpha_zero();
        }
    }

    pub fn collect_draw(&mut self, font_ctx: &MeterMsgHio) -> PaneDraw {
        self.out_font_draw(font_ctx);
        let alpha_scale = if self.fade_flag == 0 {
            1.0
        } else {
            value_increase(10, i32::from(self.timer), 0)
        };
        self.scale_alpha(alpha_scale);
        let mut draw = PaneDraw::default();
        draw.emit(&self.panes, &self.order.order);
        draw
    }

    fn scale_alpha(&mut self, scale: f32) {
        let all = self.alpha_panes();
        for index in all {
            self.panes[index].now_alpha = (f32::from(self.panes[index].now_alpha) * scale) as u8;
        }
        if self.trigger_info != 1 && self.trigger_info != 2 {
            self.out_font.alpha = (f32::from(self.out_font.alpha) * scale) as u8;
        }
    }

    fn alpha_panes(&self) -> Vec<usize> {
        let mut list: Vec<usize> = Vec::new();
        list.extend_from_slice(&[
            P_STR0, P_ST00, P_NT00, P_NK00, P_NO11, P_WD, P_NM00, P_NM01, P_ITNM, P_ITNK, P_MSK0,
            P_TL00, P_TK00,
        ]);
        list.extend(P_CAR0..P_CAR0 + CURSOR_CORNER_COUNT);
        list.extend_from_slice(&[P_WDSV, P_WDOP]);
        list.extend(P_CORNER0..P_CORNER0 + 4);
        list.extend_from_slice(&[P_NUM1, P_NUM2, P_NUMC, P_CMAP, P_WK1, P_WK0, P_TRIM, P_TRIL]);
        list.extend(P_TRI0..P_TRI0 + TRIFORCE_SLOT_COUNT);
        list.push(P_TRIB);
        list.extend(P_SIT..P_SIT + 3);
        list.extend(P_SIK..P_SIK + 3);
        list.extend(P_HTP..P_HTP + 3);
        list.extend(P_CI..P_CI + 11);
        list.push(P_HTPB);
        for base in [P_TIT, P_TIK, P_CN, P_WN, P_I0, P_I1, P_BS, P_HMB] {
            list.extend(base..base + SONG_COUNT);
        }
        list.extend_from_slice(&[P_I111, P_I122, P_CC43, P_CC32, P_CC21]);
        for base in [P_WIT, P_WIK, P_IP] {
            list.extend(base..base + 5);
        }
        list.extend_from_slice(&[P_IP00, P_CC14, P_CC07, P_CC00]);
        list
    }

    pub fn delete(&mut self, out: &mut CollectOutput) {
        for index in 0..EMITTER_COUNT {
            if self.emitters[index].spawned {
                self.emitters[index].quit = true;
                out.emitters[index] = self.emitters[index];
            }
        }
        out.emitters_quit = true;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// One picture slot's `card_pictdata` (`d_picture_box.cpp`'s `pict` records): the captured
/// texture, the judged figure and its detail bits, and the checksum the slot save carries.
/// `snap_result`/`snap_result_detail` are written from `dSnap_GetResult`/
/// `dSnap_GetResultDetail` at `d_picture_box.cpp:616-617` - the shell fills
/// [`AlbumInput::capture_from`] from the snapshot packet for that - and the pair is stored
/// here by [`AlbumBox::select_mode`] before [`AlbumBox::change_data`] hands the records to the
/// save.
pub struct PhotoRecord {
    pub tex_buffer: Vec<u8>,
    pub snap_result: u32,
    pub snap_result_detail: u8,
    pub capture_format: u8,
    pub field: Vec<u8>,
    pub csum: u16,
}

impl Default for PhotoRecord {
    fn default() -> Self {
        Self {
            tex_buffer: vec![0; PHOTO_TEX_SIZE],
            snap_result: 0,
            snap_result_detail: 0,
            capture_format: 0,
            field: vec![0; PHOTO_RECORD_TAIL],
            csum: 0,
        }
    }
}

impl PhotoRecord {
    pub fn from_texture(data: &[u8]) -> Self {
        let mut buffer = data
            .iter()
            .copied()
            .take(PHOTO_TEX_SIZE)
            .collect::<Vec<u8>>();
        buffer.resize(PHOTO_TEX_SIZE, 0);
        Self {
            tex_buffer: buffer,
            snap_result: 0,
            snap_result_detail: 0,
            capture_format: TEXTURE_CMPR,
            field: vec![0; PHOTO_RECORD_TAIL],
            csum: 0,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let byte = |at: usize| bytes.get(at).copied().unwrap_or(0);
        let word = |at: usize| u16::from_be_bytes([byte(at), byte(at + 1)]);
        let long =
            |at: usize| u32::from_be_bytes([byte(at), byte(at + 1), byte(at + 2), byte(at + 3)]);
        let mut field = bytes
            .iter()
            .copied()
            .skip(PHOTO_TEX_SIZE + 6)
            .take(PHOTO_RECORD_TAIL)
            .collect::<Vec<u8>>();
        field.resize(PHOTO_RECORD_TAIL, 0);
        Self {
            tex_buffer: PhotoRecord::from_texture(&bytes[..bytes.len().min(PHOTO_TEX_SIZE)])
                .tex_buffer,
            snap_result: long(PHOTO_TEX_SIZE),
            snap_result_detail: byte(PHOTO_TEX_SIZE + 4),
            capture_format: byte(PHOTO_TEX_SIZE + 5),
            field,
            csum: word(PHOTO_RECORD_SIZE - 2),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(PHOTO_RECORD_SIZE);
        bytes.extend_from_slice(&self.tex_buffer);
        bytes.extend_from_slice(&self.snap_result.to_be_bytes());
        bytes.push(self.snap_result_detail);
        bytes.push(self.capture_format);
        bytes.extend_from_slice(&self.field);
        bytes.resize(PHOTO_RECORD_SIZE - 2, 0);
        bytes.extend_from_slice(&self.csum.to_be_bytes());
        bytes
    }

    pub fn calc_checksum(&self) -> u16 {
        let mut sum = 0u16;
        for byte in &self.tex_buffer {
            sum = sum.wrapping_add(u16::from(*byte));
        }
        for byte in self.snap_result.to_be_bytes() {
            sum = sum.wrapping_add(u16::from(byte));
        }
        sum = sum.wrapping_add(u16::from(self.snap_result_detail));
        sum = sum.wrapping_add(u16::from(self.capture_format));
        for byte in &self.field {
            sum = sum.wrapping_add(u16::from(*byte));
        }
        sum
    }

    pub fn set_checksum(&mut self) {
        self.csum = self.calc_checksum();
    }

    pub fn test_checksum(&self) -> bool {
        self.csum == self.calc_checksum()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhotoQuad {
    pub left: i32,
    pub right: i32,
    pub bottom: i32,
    pub top: i32,
    pub slot: usize,
    pub alpha: u8,
}

impl PhotoQuad {
    pub const TEXTURE_CORNERS: [(u8, u8); 4] = [(0, 0), (1, 0), (1, 1), (0, 1)];

    pub fn tex_width(&self) -> u16 {
        PHOTO_TEX_WIDTH
    }

    pub fn tex_height(&self) -> u16 {
        PHOTO_TEX_HEIGHT
    }

    pub fn ortho(&self) -> [f32; 6] {
        [
            PHOTO_ORTHO_LEFT,
            PHOTO_ORTHO_RIGHT,
            PHOTO_ORTHO_BOTTOM,
            PHOTO_ORTHO_TOP,
            PHOTO_ORTHO_NEAR,
            PHOTO_ORTHO_FAR,
        ]
    }
}

pub const ICON_YOKO_LEFT: u8 = 0x14;
pub const ICON_YOKO_RIGHT: u8 = 0x15;
pub const ALBUM_CORNER_SHIFT: f32 = 12.0;
pub const ALBUM_LINE_SPACE: f32 = 28.0;
pub const ZOOM_MIN: f32 = 1.0;
pub const ZOOM_MAX: f32 = 9.0;
pub const ZOOM_MID: f32 = 5.0;
pub const ZOOM_SPAN: f32 = 4.0;
pub const ZOOM_BASE: f32 = 5.0;
pub const FADE_FRAMES: i16 = 10;
pub const ICON_TURN_PERIOD: i16 = 20;
pub const CAPTURE_READY: i32 = 5;
pub const CAPTURE_DONE: i32 = 6;
pub const CAPTURE_CANCEL: i32 = -1;
pub const PICTURE_FLAG_ALL: u8 = 0b111;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlbumTune {
    pub shutter_time: i16,
    pub shutter_angle: f32,
    pub shutter_line_alpha: u8,
    pub shutter_line_width: i32,
}

impl Default for AlbumTune {
    fn default() -> Self {
        Self {
            shutter_time: SHUTTER_TIME,
            shutter_angle: SHUTTER_ANGLE,
            shutter_line_alpha: SHUTTER_LINE_ALPHA,
            shutter_line_width: SHUTTER_LINE_WIDTH,
        }
    }
}

pub struct MessageEnv<'a> {
    pub font: &'a mut FontBank,
    pub archive: &'a dyn MsgArchive,
    pub clear_count: u8,
    pub color_table: &'a [u8],
}

#[derive(Clone, Debug, Default)]
pub struct AlbumInput {
    pub collect: Collect,
    pub story: StoryBits,
    pub opt_ruby: bool,
    pub camera_zoom: f32,
    pub camera_zoom_focus: f32,
    pub camera_attention: bool,
    pub capture_step: i32,
    pub capture_texture: Vec<u8>,
    pub capture_result: u32,
    pub capture_result_detail: u8,
    pub picture_status: u8,
    pub picture_num: u8,
    pub get_picture_num: u8,
    pub import_ready: bool,
    pub import_texture: Vec<u8>,
    pub camera_slot_item: u8,
    pub saved: [PhotoRecord; SLOT_COUNT],
}

impl AlbumInput {
    pub fn camera_on_button(&self, button: u8) -> bool {
        button < 3 && self.camera_slot_item == ITEM_SLOT_CAMERA
    }

    /// `pict->snap_result = dSnap_GetResult(); pict->snap_result_detail =
    /// dSnap_GetResultDetail();` (`d_picture_box.cpp:616-617`): the judged figure the snapshot
    /// packet holds, which [`AlbumBox::select_mode`] stores in the picture slot
    /// ([`PhotoRecord::snap_result`]) and `change_data` hands to the save. Retail stores the
    /// result whatever it is, `0` included, so this does too.
    pub fn capture_from(&mut self, snap: &SnapPacket) {
        self.capture_result = u32::try_from(snap.result()).unwrap_or(0);
        self.capture_result_detail = snap.result_detail();
    }
}

#[derive(Clone, Debug, Default)]
pub struct AlbumOutput {
    pub sounds: Vec<u32>,
    pub do_status_force: Option<u8>,
    pub do_status: Option<u8>,
    pub a_status_force: Option<u8>,
    pub r_status_force: Option<u8>,
    pub r_status: Option<u8>,
    pub capture_step: Option<i32>,
    pub capture_format: Option<u8>,
    pub capture_cansel: bool,
    /// `dSnap_ReleaseShutter()` (`d_picture_box.cpp:673`): the shell arms
    /// `sim::snap::SnapPacket` with this when the shutter fires.
    pub release_shutter: bool,
    pub telescope_pitch: Option<i32>,
    pub picture_status: Option<u8>,
    pub select_picture: Option<u8>,
    pub picture_format: Option<u8>,
    pub picture_result: Option<u32>,
    pub picture_result_detail: Option<u8>,
    pub picture_num_count: i32,
    pub picture_num: Option<u8>,
    pub menu_flag: Option<u8>,
    pub scope_status: Option<u8>,
    pub photo_box_cancel: bool,
    pub no_reset_flag0: bool,
    pub camera_attention: bool,
    pub photo_deleted: bool,
    pub erase_bits: Option<u8>,
    pub saved: [Option<PhotoRecord>; SLOT_COUNT],
    pub heap_lock: Option<u8>,
}

impl AlbumOutput {
    pub fn sound(&mut self, id: u32) {
        self.sounds.push(id);
    }
}

#[derive(Clone, Debug)]
pub struct AlbumBox {
    pub panes: [Pane; ALBUM_PANE_COUNT],
    pub stick: StickControl,
    pub msg_proc: MsgData,
    pub msg_entry: MesgEntry,
    pub icon_main: MsgIconPic,
    pub icon_sub: MsgIconPic,
    pub select_arrow: MsgIconPic,
    pub icon_timer: i16,
    pub arrow_timer: i16,
    pub text_buffer: [String; MESSAGE_BUFFER_COUNT],
    pub photos: [PhotoRecord; PHOTO_BUFFER_COUNT],
    pub occupied: [bool; SLOT_COUNT],
    pub sort_map: [u8; SLOT_COUNT],
    pub line_x1: [f32; SHUTTER_SEGMENT_COUNT],
    pub line_y1: [f32; SHUTTER_SEGMENT_COUNT],
    pub line_x2: [f32; SHUTTER_SEGMENT_COUNT],
    pub line_y2: [f32; SHUTTER_SEGMENT_COUNT],
    pub icn_white: u32,
    pub icn_black: u32,
    pub emp_white: u32,
    pub emp_black: u32,
    pub zoom_scale: f32,
    pub choice_cursor_x0: i32,
    pub choice_cursor_x1: i32,
    pub choice_cursor_y: i32,
    pub choice_cursor_y_alt: i32,
    pub fade_timer: i16,
    pub shutter_counter: i16,
    pub line_count: i16,
    pub exec_state: u8,
    pub sub_state: u8,
    pub view_mode: u8,
    pub selected_slot: u8,
    pub selected_choice: u8,
    pub mode_swap: u8,
    pub photo_deleted: u8,
    pub choice_arrow_idx: i8,
    pub capture_format: u8,
    pub picture_num: u8,
    pub ruby_enabled: bool,
    pub pending_message: Option<u32>,
    pub hidden_tags: Vec<&'static str>,
}

impl AlbumBox {
    pub fn new(main: &Screen, shutter: &Screen, message: &Screen) -> Self {
        let mut box_ = Self {
            panes: std::array::from_fn(|index| Pane::new(ALBUM_TAG_TABLE[index])),
            stick: StickControl::new(
                TACT_STICK_WAIT_FIRST,
                TACT_STICK_WAIT_SECOND,
                TACT_STICK_STEP,
                TACT_STICK_MIN_STEP,
                TACT_STICK_MAX_VALUE,
                TACT_STICK_MIN_VALUE,
                TACT_STICK_REPEAT_ANGLE,
                TACT_STICK_HALF_DEAD,
            ),
            msg_proc: MsgData::new_data_proc(),
            msg_entry: MesgEntry::default(),
            icon_main: MsgIconPic::default(),
            icon_sub: MsgIconPic::default(),
            select_arrow: MsgIconPic::default(),
            icon_timer: -1,
            arrow_timer: 0,
            text_buffer: [String::new(), String::new(), String::new(), String::new()],
            photos: std::array::from_fn(|_| PhotoRecord::default()),
            occupied: [false; SLOT_COUNT],
            sort_map: [SORT_EMPTY; SLOT_COUNT],
            line_x1: [0.0; SHUTTER_SEGMENT_COUNT],
            line_y1: [0.0; SHUTTER_SEGMENT_COUNT],
            line_x2: [0.0; SHUTTER_SEGMENT_COUNT],
            line_y2: [0.0; SHUTTER_SEGMENT_COUNT],
            icn_white: 0xFFFF_FFFF,
            icn_black: 0x0000_0000,
            emp_white: 0xFFFF_FFFF,
            emp_black: 0x0000_0000,
            zoom_scale: 0.0,
            choice_cursor_x0: 0,
            choice_cursor_x1: 0,
            choice_cursor_y: 0,
            choice_cursor_y_alt: 0,
            fade_timer: 0,
            shutter_counter: 0,
            line_count: 0,
            exec_state: STATE_CAMERA_OPEN,
            sub_state: SUB_IDLE,
            view_mode: MODE_CAMERA,
            selected_slot: 0,
            selected_choice: 0,
            mode_swap: 0,
            photo_deleted: 0,
            choice_arrow_idx: -1,
            capture_format: TEXTURE_I8,
            picture_num: 0,
            ruby_enabled: false,
            pending_message: None,
            hidden_tags: ALBUM_HIDDEN_TAG_LIST.to_vec(),
        };
        for index in 0..ALBUM_PANE_COUNT {
            if let Some(layout) = main.find(ALBUM_TAG_TABLE[index]) {
                box_.panes[index].parts_set(layout);
                continue;
            }
            if let Some(layout) = shutter.find(ALBUM_TAG_TABLE[index]) {
                box_.panes[index].parts_set(layout);
            }
        }
        for index in 0..MESSAGE_BUFFER_COUNT {
            if let Some(layout) = message.find(ALBUM_TAG_TABLE[A_TX0 + index]) {
                box_.panes[A_TX0 + index].parts_set(layout);
            }
        }
        box_
    }

    pub fn pane(&self, index: usize) -> &Pane {
        &self.panes[index]
    }

    pub fn pane_mut(&mut self, index: usize) -> &mut Pane {
        &mut self.panes[index]
    }

    pub fn get_status(&self) -> u8 {
        self.exec_state
    }

    pub fn set_status(&mut self, state: u8) {
        self.exec_state = state;
    }

    pub fn screen_set(&mut self) {
        for index in 0..SHUTTER_SEGMENT_COUNT {
            let angle = self.panes[A_SB0 + index].get_rotate() as i32;
            self.panes[A_SB0 + index].user_area = angle as i16;
        }
        self.shutter_hide();
        self.panes[A_CRTN].blend_init(CORNER_TEX);
        self.panes[A_CZOM].blend_init(CORNER_TEX);
        let size = self.panes[A_CZOM].size_orig;
        self.panes[A_CZOM].rotate(size.x / 2.0, size.y / 2.0, 180.0);
        for index in [A_YLIG, A_YLEF, A_YRTN, A_YZOM] {
            self.panes[index].blend_init(CURSOR_TEX);
        }
        for index in [A_YRTN, A_YZOM, A_CRTN, A_CZOM] {
            self.panes[index].pos_center_orig.x -= ALBUM_CORNER_SHIFT;
            self.panes[index].pane_trans(0.0, 0.0);
        }
    }

    pub fn screen_set2(&mut self, opt_ruby: bool) {
        let head_dy = if opt_ruby {
            ALBUM_RUBY_DY
        } else {
            ALBUM_PLAIN_DY
        };
        for index in [0usize, 2usize] {
            let pane = &self.panes[A_TX0 + index];
            let x = pane.pos_top_left.x;
            let y = pane.pos_top_left.y;
            self.panes[A_TX0 + index].move_to(x, y + head_dy);
        }
        for index in [1usize, 3usize] {
            let pane = &self.panes[A_TX0 + index];
            let x = pane.pos_top_left.x;
            let y = pane.pos_top_left.y;
            self.panes[A_TX0 + index].move_to(x, y + ALBUM_RUBY_HEAD_DY);
        }
        for index in 0..MESSAGE_BUFFER_COUNT {
            self.panes[A_TX0 + index].set_font_size(MSG_FONT_SIZE as f32, MSG_FONT_SIZE as f32);
            self.panes[A_TX0 + index].set_char_space(0.0);
        }
        for index in [0usize, 2usize] {
            self.panes[A_TX0 + index].set_line_space(ALBUM_LINE_SPACE);
        }
    }

    pub fn camera_alpha_inc(&mut self, alpha: f32) {
        for index in 0..SHUTTER_SEGMENT_COUNT {
            self.panes[A_SB0 + index].set_now_alpha(alpha);
            self.panes[A_ST0 + index].set_now_alpha(alpha);
        }
        for index in [
            A_WNUM, A_WNUK, A_WPBA, A_WBAP, A_WPSC, A_YRTN, A_YZOM, A_CRTN, A_CZOM, A_LRTN, A_RZOM,
            A_FD00, A_WP03, A_WP02, A_WP01, A_WP04,
        ] {
            self.panes[index].set_now_alpha(alpha);
        }
        for index in 0..MESSAGE_BUFFER_COUNT {
            self.panes[A_TX0 + index].set_now_alpha(alpha);
        }
    }

    pub fn browse_alpha_inc(&mut self, alpha: f32) {
        for index in 0..SHUTTER_SEGMENT_COUNT {
            self.panes[A_SB0 + index].set_now_alpha(alpha);
            self.panes[A_ST0 + index].set_now_alpha(alpha);
        }
        self.panes[A_YLIG].set_now_alpha(alpha);
        self.panes[A_YLEF].set_now_alpha(alpha);
        for index in 0..SLOT_COUNT {
            if self.occupied[index] {
                self.panes[A_ICN0 + index].set_now_alpha(alpha);
            }
            self.panes[A_EMP0 + index].set_now_alpha(alpha);
        }
        self.panes[A_CT1].set_now_alpha(alpha);
        self.panes[A_CT2].set_now_alpha(alpha);
        for index in 0..8 {
            self.panes[A_SP0 + index].set_now_alpha(alpha);
        }
        for index in 0..SLOT_COUNT {
            self.panes[A_NO0 + index].set_now_alpha(alpha);
            self.panes[A_NOB0 + index].set_now_alpha(alpha);
            self.panes[A_NOK0 + index].set_now_alpha(alpha);
        }
        for index in [A_B1, A_B2, A_SHUT, A_WP03, A_WP02, A_WP01, A_WP04] {
            self.panes[index].set_now_alpha(alpha);
        }
        for index in 0..MESSAGE_BUFFER_COUNT {
            self.panes[A_TX0 + index].set_now_alpha(alpha);
        }
    }

    pub fn get_alpha_inc(&mut self, alpha: f32) {
        self.panes[A_CT1].set_now_alpha(alpha);
        self.panes[A_CT2].set_now_alpha(alpha);
        for index in 0..8 {
            self.panes[A_SP0 + index].set_now_alpha(alpha);
        }
        for index in 0..SLOT_COUNT {
            self.panes[A_NO0 + index].set_now_alpha(alpha);
            self.panes[A_NOB0 + index].set_now_alpha(alpha);
            self.panes[A_NOK0 + index].set_now_alpha(alpha);
        }
        for index in [A_B1, A_B2, A_SHUT, A_WP03, A_WP02, A_WP01, A_WP04] {
            self.panes[index].set_now_alpha(alpha);
        }
        for index in 0..MESSAGE_BUFFER_COUNT {
            self.panes[A_TX0 + index].set_now_alpha(alpha);
        }
    }

    pub fn alpha_dec(&mut self, scale: f32) {
        for index in 0..SHUTTER_SEGMENT_COUNT {
            scale_pane(&mut self.panes[A_SB0 + index], scale);
            scale_pane(&mut self.panes[A_ST0 + index], scale);
        }
        for index in [
            A_YLIG, A_YLEF, A_WNUM, A_WNUK, A_WPBA, A_WBAP, A_WPSC, A_YRTN, A_YZOM, A_CRTN, A_CZOM,
            A_LRTN, A_RZOM, A_SHUT, A_FD00, A_WP03, A_WP02, A_WP01, A_WP04,
        ] {
            scale_pane(&mut self.panes[index], scale);
        }
        for index in 0..MESSAGE_BUFFER_COUNT {
            scale_pane(&mut self.panes[A_TX0 + index], scale);
        }
    }

    pub fn zoom_scale(&mut self, input: &AlbumInput, out: &mut AlbumOutput) {
        let mut scale = input.camera_zoom;
        if scale < ZOOM_MIN {
            scale = ZOOM_MIN;
        }
        if scale > ZOOM_MAX {
            scale = ZOOM_MAX;
        }
        if scale == self.zoom_scale {
            return;
        }
        let mut notify = true;
        if self.zoom_scale == 0.0 {
            notify = false;
        }
        self.zoom_scale = scale;
        self.change_scale(scale as i32);
        if scale == ZOOM_MIN || scale == ZOOM_MAX {
            if notify {
                out.sound(SE_TELESCOPE_STOP);
            }
        } else {
            out.sound(SE_TELESCOPE_ZOOM);
            out.telescope_pitch = Some((input.camera_zoom_focus * 32768.0 + 0.5) as i32);
        }
    }

    pub fn change_scale(&mut self, scale: i32) {
        let a = (self.panes[A_WPSC].pos_top_left_orig.y + self.panes[A_WBAP].size_orig.y / 2.0)
            - self.panes[A_WBAP].pos_center_orig.y;
        let b = ((self.panes[A_WPSC].pos_top_left_orig.y + self.panes[A_WPSC].size_orig.y)
            - self.panes[A_WBAP].size_orig.y / 2.0)
            - self.panes[A_WBAP].pos_center_orig.y;
        let index = (scale.clamp(1, 9) - 1) as usize;
        self.panes[A_WNUM].change_texture(RUPEE_NUM_TEX[index]);
        self.panes[A_WNUK].change_texture(RUPEE_NUM_TEX[index]);
        let trans_y = if self.zoom_scale < ZOOM_MID {
            (ZOOM_BASE - self.zoom_scale) / ZOOM_SPAN * b
        } else {
            (self.zoom_scale - ZOOM_BASE) / ZOOM_SPAN * a
        };
        self.panes[A_WBAP].pane_trans(0.0, trans_y);
    }

    pub fn up_down_icon_move(&mut self, input: &AlbumInput) {
        self.panes[A_CRTN].user_area += 1;
        let turn = self.panes[A_CRTN].user_area;
        if turn < ICON_TURN_PERIOD / 2 {
            let t = value_increase(20, i32::from(turn), 0);
            self.panes[A_YRTN].set_blend_ratio(0.0, 1.0, 1.0, 1.0);
            self.panes[A_YZOM].set_blend_ratio(0.0, 1.0, 1.0, 1.0);
            if input.camera_zoom != ZOOM_MIN && self.panes[A_YRTN].user_area == 0 {
                self.panes[A_CRTN].set_blend_ratio(t, 1.0 - t, 1.0, 1.0);
                self.panes[A_YRTN].pane_trans(0.0, NOTE_SLIDE_STEP);
            }
            if input.camera_zoom != ZOOM_MAX && self.panes[A_YZOM].user_area == 0 {
                self.panes[A_CZOM].set_blend_ratio(t, 1.0 - t, 1.0, 1.0);
                self.panes[A_YZOM].pane_trans(0.0, -NOTE_SLIDE_STEP);
            }
        } else {
            let t = value_increase(20, i32::from(turn - ICON_TURN_PERIOD / 2), 0);
            self.panes[A_YRTN].set_blend_ratio(1.0, 0.0, 1.0, 1.0);
            self.panes[A_YZOM].set_blend_ratio(1.0, 0.0, 1.0, 1.0);
            if input.camera_zoom == ZOOM_MIN {
                if self.panes[A_YRTN].user_area == 0 {
                    self.panes[A_CRTN].set_blend_ratio(1.0 - t, t, 1.0, 1.0);
                    self.panes[A_YRTN].pane_trans(0.0, 0.0);
                }
                if turn == ICON_TURN_PERIOD {
                    self.panes[A_YRTN].user_area = 1;
                }
            } else if self.panes[A_YRTN].user_area == 0 {
                self.panes[A_CRTN].set_blend_ratio(1.0 - t, t, 1.0, 1.0);
                self.panes[A_YRTN].pane_trans(0.0, 0.0);
            } else if turn == ICON_TURN_PERIOD {
                self.panes[A_YRTN].user_area = 0;
            }
            if input.camera_zoom == ZOOM_MAX {
                if self.panes[A_YZOM].user_area == 0 {
                    self.panes[A_CZOM].set_blend_ratio(1.0 - t, t, 1.0, 1.0);
                    self.panes[A_YZOM].pane_trans(0.0, 0.0);
                }
                if turn == ICON_TURN_PERIOD {
                    self.panes[A_YZOM].user_area = 1;
                }
            } else if self.panes[A_YZOM].user_area == 0 {
                self.panes[A_CZOM].set_blend_ratio(1.0 - t, t, 1.0, 1.0);
                self.panes[A_YZOM].pane_trans(0.0, 0.0);
            } else if turn == ICON_TURN_PERIOD {
                self.panes[A_YZOM].user_area = 0;
            }
        }
        if input.camera_zoom == ZOOM_MIN {
            self.panes[A_YRTN].hide();
            self.panes[A_YZOM].show();
        } else if input.camera_zoom == ZOOM_MAX {
            self.panes[A_YRTN].show();
            self.panes[A_YZOM].hide();
        } else {
            self.panes[A_YRTN].show();
            self.panes[A_YZOM].show();
        }
        if turn >= ICON_TURN_PERIOD {
            self.panes[A_CRTN].user_area = 0;
        }
    }

    pub fn left_right_icon_move(&mut self) {
        self.panes[A_YLIG].user_area += 1;
        let turn = self.panes[A_YLIG].user_area;
        if turn < ICON_TURN_PERIOD / 2 {
            if self.selected_slot != 2 {
                self.panes[A_YLIG].set_blend_ratio(0.0, 1.0, 1.0, 1.0);
                self.panes[A_YLIG].pane_trans(NOTE_SLIDE_STEP, 0.0);
            }
            if self.selected_slot != 1 {
                self.panes[A_YLEF].set_blend_ratio(0.0, 1.0, 1.0, 1.0);
                self.panes[A_YLEF].pane_trans(-NOTE_SLIDE_STEP, 0.0);
            }
        } else {
            if self.selected_slot != 2 {
                self.panes[A_YLIG].set_blend_ratio(1.0, 0.0, 1.0, 1.0);
                self.panes[A_YLIG].pane_trans(0.0, 0.0);
            }
            if self.selected_slot != 1 {
                self.panes[A_YLEF].set_blend_ratio(1.0, 0.0, 1.0, 1.0);
                self.panes[A_YLEF].pane_trans(0.0, 0.0);
            }
        }
        if turn >= ICON_TURN_PERIOD {
            self.panes[A_YLIG].user_area = 0;
        }
    }

    pub fn click_shutter_mode(
        &mut self,
        input: &AlbumInput,
        out: &mut AlbumOutput,
        tune: &AlbumTune,
    ) {
        let span = tune.shutter_time;
        let total = span << 1;
        let t = if self.shutter_counter < span {
            self.shutter_counter += 1;
            value_increase(i32::from(span), i32::from(self.shutter_counter), 0)
        } else if self.shutter_counter == span {
            if input.capture_step == CAPTURE_READY {
                self.shutter_counter += 1;
                out.menu_flag = Some(1);
            }
            1.0
        } else {
            self.shutter_counter += 1;
            value_increase(i32::from(span), i32::from(total - self.shutter_counter), 0)
        };

        for index in 0..SHUTTER_SEGMENT_COUNT {
            let angle = f32::from(self.panes[A_SB0 + index].user_area) + tune.shutter_angle * t;
            self.rotate_shutter(index, angle, t);
            self.shutter_line_rotate_center(angle, index);
        }

        self.shutter_line_move();

        if self.shutter_counter == total {
            self.sub_state = SUB_SHIFT_1_TO_0;
            self.shutter_hide();
            self.pending_message = Some(MSG_SAVED);
        }
    }

    fn rotate_shutter(&mut self, index: usize, angle: f32, t: f32) {
        let size = self.panes[A_SB0 + index].size_orig;
        self.panes[A_SB0 + index].rotate(size.x / 2.0, size.y / 2.0, angle);
        let lift = -(self.panes[A_ST0 + index].pos_top_left_orig.y
            - self.panes[A_SB0 + index].size_orig.y / 2.0);
        self.panes[A_ST0 + index].pane_trans(0.0, t * lift);
    }

    pub fn select_mode(&mut self, input: &AlbumInput, pad: &PadInput, out: &mut AlbumOutput) {
        let selection = self.select_check();
        if selection != self.selected_choice {
            self.selected_choice = selection;
            out.sound(SE_TALK_CURSOR);
        }

        if pad.a {
            if input.capture_step == CAPTURE_READY {
                out.capture_step = Some(CAPTURE_DONE);
                if selection == 0 {
                    let slot = usize::from(input.picture_num).min(SLOT_COUNT - 1);
                    let mut record = PhotoRecord::from_texture(&input.capture_texture);
                    record.snap_result = input.capture_result;
                    record.snap_result_detail = input.capture_result_detail;
                    record.capture_format = self.capture_format;
                    self.photos[slot] = record;
                    out.picture_num_count = 1;
                    let next = input.picture_num + 1;
                    self.occupied[slot] = true;
                    out.picture_num = Some(next);
                    out.sound(SE_SAVE_PIC);
                    self.pending_message = Some(self.remain_message_set(next));
                } else {
                    out.sound(SE_DEL_PIC);
                    self.pending_message = Some(self.remain_message_set(input.picture_num));
                }
                self.sub_state = SUB_IDLE;
                out.scope_status = Some(STATUS_SCOPE_ACTIVE);
                out.menu_flag = Some(0);
            }
        } else if pad.b && input.capture_step == CAPTURE_READY {
            out.capture_step = Some(CAPTURE_DONE);
            self.pending_message = Some(self.remain_message_set(input.picture_num));
            out.sound(SE_DEL_PIC);
            self.sub_state = SUB_IDLE;
            out.scope_status = Some(STATUS_SCOPE_ACTIVE);
            out.menu_flag = Some(0);
        }
    }

    pub fn camera_mode(&mut self, input: &AlbumInput, pad: &PadInput, out: &mut AlbumOutput) {
        if pad.a {
            if input.capture_step == 0 && input.picture_num < 3 {
                self.shutter_counter = 0;
                self.sub_state = SUB_CONFIRM;
                self.capture_format = if input.camera_slot_item == ITEM_PICTO_BOX {
                    TEXTURE_I8
                } else {
                    TEXTURE_RGB565
                };
                out.capture_format = Some(self.capture_format);
                out.capture_step = Some(1);
                // `dSnap_ReleaseShutter()` (`d_picture_box.cpp:673`): the one place the
                // picture box arms the snapshot packet, in the same A-press arm as the
                // capture step.
                out.release_shutter = true;
                out.scope_status = Some(STATUS_STOP);
                self.shutter_show();
                out.sound(SE_SHUTTER);
            }
        } else if pad.r {
            self.mode_swap = 1;
            self.shutter_counter = 0;
            self.shutter_show();
            self.selected_slot = 0;
            out.sound(SE_CHANGE_MODE);
        } else if !input.camera_attention {
            self.exec_state = STATE_CLOSE;
        } else if input.capture_step == CAPTURE_CANCEL {
            out.photo_box_cancel = true;
            self.exec_state = STATE_CLOSE;
        } else {
            self.zoom_scale(input, out);
        }
        self.up_down_icon_move(input);
    }

    pub fn picture_quads(&self, alpha: u8) -> Vec<PhotoQuad> {
        let mut quads = Vec::new();
        if self.exec_state == STATE_CLOSED {
            return quads;
        }
        match self.view_mode {
            MODE_GET => quads.push(self.quad_for(3, alpha)),
            MODE_BROWSE => {
                for index in 0..SLOT_COUNT {
                    if self.occupied[index] {
                        quads.push(self.quad_for(index, alpha));
                    }
                }
            }
            _ => {}
        }
        quads
    }

    fn quad_for(&self, slot: usize, alpha: u8) -> PhotoQuad {
        let index = if slot == 3 { 0 } else { slot };
        let pane = &self.panes[A_NO0 + index];
        let left = pane.pos_top_left.x;
        let top = pane.pos_top_left.y;
        let center_x = (left + (left + pane.size_orig.x)) / 2.0;
        let center_y = (top + (top + pane.size_orig.y)) / 2.0;
        PhotoQuad {
            left: center_x as i32 - PHOTO_QUAD_HALF_W,
            right: center_x as i32 + PHOTO_QUAD_HALF_W,
            bottom: center_y as i32 - PHOTO_QUAD_HALF_H,
            top: center_y as i32 + PHOTO_QUAD_HALF_H,
            slot,
            alpha,
        }
    }

    pub fn picture_erase_wait(
        &mut self,
        input: &AlbumInput,
        pad: &PadInput,
        out: &mut AlbumOutput,
    ) {
        let selection = self.select_check();
        if selection != self.selected_choice {
            self.selected_choice = selection;
            out.sound(SE_TALK_CURSOR);
        }
        if pad.a {
            if selection == 0 {
                self.picture_erase(input, out);
                out.sound(SE_B_DEL_PIC);
            } else {
                self.pending_message = Some(self.exist_message_set(self.picture_num));
                out.sound(SE_B_LEAVE_PIC);
            }
            self.sub_state = SUB_IDLE;
            out.scope_status = Some(STATUS_SCOPE_ACTIVE);
        } else if pad.b {
            self.pending_message = Some(self.exist_message_set(self.picture_num));
            out.sound(SE_B_LEAVE_PIC);
            self.sub_state = SUB_IDLE;
            out.scope_status = Some(STATUS_SCOPE_ACTIVE);
        }
    }

    pub fn picture_decide(&mut self, out: &mut AlbumOutput) {
        let selection = self.select_check();
        if selection != self.selected_choice {
            self.selected_choice = selection;
            out.sound(SE_TALK_CURSOR);
        }
        if selection == 0 {
            let slot = usize::from(self.selected_slot).min(PHOTO_BUFFER_COUNT - 1);
            let record = &self.photos[slot];
            out.select_picture = Some(self.selected_slot);
            out.picture_format = Some(record.capture_format);
            out.picture_status = Some(1);
            out.picture_result = Some(record.snap_result);
            out.picture_result_detail = Some(record.snap_result_detail);
            out.photo_box_cancel = true;
            self.exec_state = STATE_CLOSE;
            out.sound(SE_B_SHOW);
            self.sub_state = SUB_IDLE;
            out.scope_status = Some(STATUS_SCOPE_ACTIVE);
        } else {
            self.pending_message = Some(self.exist_message_set(self.picture_num));
            out.sound(SE_B_LEAVE_PIC);
            self.sub_state = SUB_IDLE;
            out.scope_status = Some(STATUS_SCOPE_ACTIVE);
        }
    }

    pub fn picture_erase(&mut self, input: &AlbumInput, out: &mut AlbumOutput) {
        let slot = usize::from(self.selected_slot).min(SLOT_COUNT - 1);
        if self.occupied[slot] {
            self.photo_deleted = 1;
            out.photo_deleted = true;
            self.occupied[slot] = false;
            self.panes[A_ICN0 + slot].set_now_alpha_zero();
            out.picture_num_count = -1;
            out.erase_bits = Some(PICTURE_FLAG_ALL);
        }
        self.picture_num = input.picture_num.saturating_sub(1);
        self.pending_message = Some(self.exist_message_set(self.picture_num));
    }

    pub fn picture_trans_x(&mut self, x0: f32, x1: f32, x2: f32, x3: f32, y: f32) {
        for index in 0..SLOT_COUNT {
            self.panes[A_NO0 + index].pane_trans(x0, 0.0);
            self.panes[A_NOB0 + index].pane_trans(x0, 0.0);
        }
        self.panes[A_NOK0].pane_trans(x1, -y);
        self.panes[A_NOK0 + 1].pane_trans(x2, y);
        self.panes[A_NOK0 + 2].pane_trans(x3, y);
    }

    pub fn change_picture(&mut self, tune: &AlbumTune) {
        let total = tune.shutter_time << 1;
        let gap = self.panes[A_NO0].pos_center_orig.x - self.panes[A_NO0 + 1].pos_center_orig.x;
        let rise = self.panes[A_NOK0].pos_center_orig.y - self.panes[A_NOK0 + 1].pos_center_orig.y;
        self.shutter_counter += 1;
        match self.sub_state {
            SUB_SHIFT_1_TO_0 => {
                let t =
                    value_increase(i32::from(total), i32::from(total - self.shutter_counter), 0);
                let a =
                    self.panes[A_NOK0 + 2].pos_center_orig.x - self.panes[A_NOK0].pos_center_orig.x;
                let b =
                    self.panes[A_NOK0].pos_center_orig.x - self.panes[A_NOK0 + 1].pos_center_orig.x;
                self.picture_trans_x(gap * t, a * t, b * t, a * t, rise * t);
            }
            SUB_SHIFT_0_TO_1 => {
                let t = value_increase(i32::from(total), i32::from(self.shutter_counter), 0);
                let a =
                    self.panes[A_NOK0 + 2].pos_center_orig.x - self.panes[A_NOK0].pos_center_orig.x;
                let b =
                    self.panes[A_NOK0].pos_center_orig.x - self.panes[A_NOK0 + 1].pos_center_orig.x;
                self.picture_trans_x(gap * t, a * t, b * t, a * t, rise * t);
            }
            SUB_SHIFT_0_TO_2 => {
                let t = value_increase(i32::from(total), i32::from(self.shutter_counter), 0);
                let a =
                    self.panes[A_NOK0 + 1].pos_center_orig.x - self.panes[A_NOK0].pos_center_orig.x;
                let b =
                    self.panes[A_NOK0].pos_center_orig.x - self.panes[A_NOK0 + 2].pos_center_orig.x;
                self.picture_trans_x(-gap * t, a * t, a * t, b * t, rise * t);
            }
            SUB_SHIFT_2_TO_0 => {
                let t =
                    value_increase(i32::from(total), i32::from(total - self.shutter_counter), 0);
                let a =
                    self.panes[A_NOK0 + 1].pos_center_orig.x - self.panes[A_NOK0].pos_center_orig.x;
                let b =
                    self.panes[A_NOK0].pos_center_orig.x - self.panes[A_NOK0 + 2].pos_center_orig.x;
                self.picture_trans_x(-gap * t, a * t, a * t, b * t, rise * t);
            }
            _ => {}
        }
        if self.shutter_counter == total {
            self.sub_state = SUB_IDLE;
        }
    }

    pub fn shutter_change(&mut self, tune: &AlbumTune, out: &mut AlbumOutput) {
        let span = tune.shutter_time;
        let total = span << 1;
        let counter = self.shutter_counter;
        let t = if counter < span {
            self.shutter_counter = counter + 1;
            value_increase(i32::from(span), i32::from(self.shutter_counter), 0)
        } else if counter == span {
            self.shutter_counter = counter + 1;
            if self.view_mode == MODE_CAMERA {
                self.change_camera_to_browse(out);
            } else {
                self.change_browse_to_camera(out);
            }
            1.0
        } else {
            self.shutter_counter = counter + 1;
            let t = value_increase(i32::from(span), i32::from(total - self.shutter_counter), 0);
            if self.shutter_counter == total {
                self.shutter_hide();
                self.mode_swap = 0;
            }
            t
        };

        for index in 0..SHUTTER_SEGMENT_COUNT {
            let angle = f32::from(self.panes[A_SB0 + index].user_area) + tune.shutter_angle * t;
            self.rotate_shutter(index, angle, t);
            self.shutter_line_rotate_init_pos(angle, index);
        }

        self.shutter_line_move();
    }

    pub fn move_camera(
        &mut self,
        input: &AlbumInput,
        pad: &PadInput,
        out: &mut AlbumOutput,
        tune: &AlbumTune,
    ) {
        if self.sub_state == SUB_CONFIRM {
            self.click_shutter_mode(input, out, tune);
            out.a_status_force = Some(ACT_BLANK);
            out.do_status_force = Some(ACT_BLANK);
            if input.picture_status != 2 && input.picture_status != 3 {
                out.r_status_force = Some(ACT_BLANK);
            }
            out.r_status = Some(ACT_BLANK);
        } else if self.sub_state == SUB_SHIFT_1_TO_0 {
            self.select_mode(input, pad, out);
            out.do_status_force = Some(ACT_CHOOSE);
            out.a_status_force = Some(ACT_RETURN);
        } else {
            self.camera_mode(input, pad, out);
            if input.picture_num < 3 {
                out.do_status_force = Some(ACT_TAKE_PHOTO);
            } else {
                out.do_status_force = Some(ACT_BLANK);
            }
            out.a_status_force = Some(ACT_RETURN);
            if input.picture_status != 2 && input.picture_status != 3 {
                out.r_status_force = Some(ACT_SWAP_MODES);
            }
        }
    }

    pub fn move_browse(
        &mut self,
        input: &AlbumInput,
        pad: &PadInput,
        out: &mut AlbumOutput,
        tune: &AlbumTune,
    ) {
        self.stick.check_trigger(pad.stick_value, pad.stick_angle);
        if self.sub_state == SUB_IDLE {
            if pad.r {
                self.mode_swap = 1;
                self.shutter_counter = 0;
                self.shutter_show();
                out.sound(SE_CHANGE_MODE);
            } else if pad.a {
                if self.occupied[usize::from(self.selected_slot)] {
                    self.sub_state = SUB_CONFIRM;
                    self.pending_message = Some(MSG_ERASE_ASK);
                    out.scope_status = Some(STATUS_STOP);
                    out.sound(SE_PIC_OK);
                }
            } else if self.stick.check_right_trigger() {
                self.slide_right(out);
            } else if self.stick.check_left_trigger() {
                self.slide_left(out);
            } else if self.cancel_requested(pad, input) {
                out.no_reset_flag0 = true;
                self.exec_state = STATE_CLOSE;
                out.sound(SE_SUBMENU_OUT);
            } else {
                self.left_right_icon_move();
            }

            if input.picture_num != 0 {
                out.do_status_force = Some(ACT_CHOOSE);
            } else {
                out.do_status_force = Some(ACT_BLANK);
            }
            out.a_status_force = Some(ACT_RETURN);
            if input.picture_status != 2 && input.picture_status != 3 {
                out.r_status_force = Some(ACT_SWAP_MODES);
            }
        } else if self.sub_state == SUB_CONFIRM {
            self.picture_erase_wait(input, pad, out);
            if input.picture_num != 0 {
                out.do_status_force = Some(ACT_CHOOSE);
            } else {
                out.do_status_force = Some(ACT_BLANK);
            }
            out.a_status_force = Some(ACT_RETURN);
        } else {
            self.change_picture(tune);
            out.do_status_force = Some(ACT_BLANK);
        }

        self.set_color_anime(self.selected_slot);
    }

    fn slide_right(&mut self, out: &mut AlbumOutput) {
        if self.selected_slot == 1 {
            self.set_color_init(1);
            self.selected_slot = 0;
            self.shutter_counter = 0;
            self.sub_state = SUB_SHIFT_1_TO_0;
            out.sound(SE_SWITCH_PIC);
        } else if self.selected_slot == 0 {
            self.set_color_init(0);
            self.selected_slot = 2;
            self.shutter_counter = 0;
            self.sub_state = SUB_SHIFT_0_TO_2;
            out.sound(SE_SWITCH_PIC);
        }
    }

    fn slide_left(&mut self, out: &mut AlbumOutput) {
        if self.selected_slot == 0 {
            self.set_color_init(0);
            self.selected_slot = 1;
            self.shutter_counter = 0;
            self.sub_state = SUB_SHIFT_0_TO_1;
            out.sound(SE_SWITCH_PIC);
        } else if self.selected_slot == 2 {
            self.set_color_init(2);
            self.selected_slot = 0;
            self.shutter_counter = 0;
            self.sub_state = SUB_SHIFT_2_TO_0;
            out.sound(SE_SWITCH_PIC);
        }
    }

    pub fn select_browse(
        &mut self,
        input: &AlbumInput,
        pad: &PadInput,
        out: &mut AlbumOutput,
        tune: &AlbumTune,
    ) {
        self.stick.check_trigger(pad.stick_value, pad.stick_angle);
        if self.sub_state == SUB_IDLE {
            if pad.a {
                if self.occupied[usize::from(self.selected_slot)] {
                    self.pending_message = Some(MSG_SHOW_ASK);
                    self.sub_state = SUB_CONFIRM;
                }
            } else if self.cancel_requested(pad, input) {
                out.photo_box_cancel = true;
                out.picture_status = Some(0);
                out.sound(SE_B_LEAVE_PIC);
                self.exec_state = STATE_CLOSE;
                out.sound(SE_SUBMENU_OUT);
            } else if self.stick.check_right_trigger() {
                self.slide_right(out);
            } else if self.stick.check_left_trigger() {
                self.slide_left(out);
            } else {
                self.left_right_icon_move();
            }

            if input.picture_num != 0 {
                out.do_status_force = Some(ACT_CHOOSE);
            } else {
                out.do_status_force = Some(ACT_BLANK);
            }
            out.a_status_force = Some(ACT_RETURN);
            if input.picture_status != 2 && input.picture_status != 3 {
                out.r_status_force = Some(ACT_SWAP_MODES);
            }
        } else if self.sub_state == SUB_CONFIRM {
            self.picture_decide(out);
            out.do_status_force = Some(ACT_CHOOSE);
            out.a_status_force = Some(ACT_RETURN);
        } else {
            self.change_picture(tune);
            out.do_status_force = Some(ACT_BLANK);
        }

        self.set_color_anime(self.selected_slot);
    }

    fn cancel_requested(&self, pad: &PadInput, input: &AlbumInput) -> bool {
        pad.b
            || (pad.x && input.camera_on_button(0))
            || (pad.y && input.camera_on_button(1))
            || (pad.z && input.camera_on_button(2))
    }

    pub fn get_browse(&mut self, input: &AlbumInput, pad: &PadInput, out: &mut AlbumOutput) {
        if pad.a || self.cancel_requested(pad, input) {
            out.erase_bits = Some(PICTURE_FLAG_ALL);
            let slot = usize::from(input.picture_num).min(SLOT_COUNT - 1);
            self.photos[slot] = self.photos[3].clone();
            out.picture_num_count = 1;
            self.occupied[slot] = true;
            out.photo_box_cancel = true;
            out.picture_status = Some(0);
            out.sound(SE_B_LEAVE_PIC);
            self.exec_state = STATE_CLOSE;
            out.sound(SE_SUBMENU_OUT);
        }
        out.do_status_force = Some(ACT_NEXT);
        out.a_status_force = Some(ACT_RETURN);
    }

    pub fn change_browse_to_camera(&mut self, out: &mut AlbumOutput) {
        self.view_mode = MODE_CAMERA;
        self.set_color_init(self.selected_slot);
        self.picture_trans_x(0.0, 0.0, 0.0, 0.0, 0.0);
        self.panes[A_YLEF].set_now_alpha_zero();
        self.panes[A_YLIG].set_now_alpha_zero();
        for index in 0..SLOT_COUNT {
            self.panes[A_ICN0 + index].set_now_alpha_zero();
            self.panes[A_EMP0 + index].set_now_alpha_zero();
        }
        self.panes[A_CT1].set_now_alpha_zero();
        self.panes[A_CT2].set_now_alpha_zero();
        for index in 0..8 {
            self.panes[A_SP0 + index].set_now_alpha_zero();
        }
        for index in 0..SLOT_COUNT {
            self.panes[A_NO0 + index].set_now_alpha_zero();
            self.panes[A_NOB0 + index].set_now_alpha_zero();
            self.panes[A_NOK0 + index].set_now_alpha_zero();
        }
        self.panes[A_B1].set_now_alpha_zero();
        self.panes[A_B2].set_now_alpha_zero();
        self.panes[A_SHUT].set_now_alpha_zero();
        for index in [
            A_WNUM, A_WNUK, A_WPBA, A_WBAP, A_WPSC, A_YRTN, A_YZOM, A_CRTN, A_CZOM, A_LRTN, A_RZOM,
            A_FD00,
        ] {
            self.panes[index].set_init_alpha();
        }
        self.change_data(out);
        self.pending_message = Some(self.remain_message_set(self.picture_num));
        out.menu_flag = Some(0);
    }

    pub fn change_camera_to_browse(&mut self, out: &mut AlbumOutput) {
        self.view_mode = MODE_BROWSE;
        for index in [
            A_WNUM, A_WNUK, A_WPBA, A_WBAP, A_WPSC, A_YRTN, A_YZOM, A_CRTN, A_CZOM, A_LRTN, A_RZOM,
            A_FD00,
        ] {
            self.panes[index].set_now_alpha_zero();
        }
        self.panes[A_YLEF].set_init_alpha();
        self.panes[A_YLIG].set_init_alpha();
        for index in 0..SLOT_COUNT {
            if self.occupied[index] {
                self.panes[A_ICN0 + index].set_init_alpha();
            } else {
                self.panes[A_ICN0 + index].set_now_alpha_zero();
            }
            self.panes[A_EMP0 + index].set_init_alpha();
        }
        self.panes[A_CT1].set_init_alpha();
        self.panes[A_CT2].set_init_alpha();
        for index in 0..8 {
            self.panes[A_SP0 + index].set_init_alpha();
        }
        for index in 0..SLOT_COUNT {
            self.panes[A_NO0 + index].set_init_alpha();
            self.panes[A_NOB0 + index].set_init_alpha();
            self.panes[A_NOK0 + index].set_init_alpha();
        }
        self.panes[A_B1].set_init_alpha();
        self.panes[A_B2].set_init_alpha();
        self.panes[A_SHUT].set_init_alpha();
        self.pending_message = Some(self.exist_message_set(self.picture_num));
        out.menu_flag = Some(1);
    }

    pub fn set_color_init(&mut self, slot: u8) {
        let slot = usize::from(slot).min(SLOT_COUNT - 1);
        self.panes[A_ICN0 + slot].set_white(self.icn_white);
        self.panes[A_EMP0 + slot].set_white(self.emp_white);
        self.panes[A_EMP0 + slot].set_black(self.emp_black);
        self.panes[A_ICN0 + slot].user_area = 0;
        if self.occupied[slot] {
            self.panes[A_ICN0 + slot].set_init_alpha();
        }
        self.panes[A_EMP0 + slot].set_init_alpha();
    }

    pub fn set_color_anime(&mut self, slot: u8) {
        let slot = usize::from(slot).min(SLOT_COUNT - 1);
        self.panes[A_ICN0 + slot].user_area += 1;
        if self.panes[A_ICN0 + slot].user_area >= 0x28 {
            self.panes[A_ICN0 + slot].user_area = 0;
        }
        let frame = i32::from(self.panes[A_ICN0 + slot].user_area);
        let t = if frame < 20 {
            value_increase(20, frame, 0)
        } else {
            value_increase(20, 0x28 - frame, 0)
        };

        let icn = unpack_color(self.icn_white);
        let emp = unpack_color(self.emp_white);
        let blk = unpack_color(self.emp_black);
        let icn_white = [
            (f32::from(icn[0]) - t * (f32::from(icn[0]) - 255.0)) as u8,
            (f32::from(icn[1]) - t * (f32::from(icn[1]) - 60.0)) as u8,
            (f32::from(icn[2]) - t * (f32::from(icn[2]) - 60.0)) as u8,
            0xFF,
        ];
        let emp_white = [
            (f32::from(emp[0]) - t * (f32::from(emp[0]) - 255.0)) as u8,
            (f32::from(emp[1]) - t * (f32::from(emp[1]) - 60.0)) as u8,
            (f32::from(emp[2]) - t * (f32::from(emp[2]) - 60.0)) as u8,
            0xFF,
        ];
        let emp_black = [
            (f32::from(blk[0]) - t * (f32::from(blk[0]) - 255.0)) as u8,
            (f32::from(blk[1]) - t * (f32::from(blk[1]) - 60.0)) as u8,
            (f32::from(blk[2]) - t * (f32::from(blk[2]) - 60.0)) as u8,
            0,
        ];

        self.panes[A_ICN0 + slot].set_white(pack_color(icn_white));
        self.panes[A_EMP0 + slot].set_white(pack_color(emp_white));
        self.panes[A_EMP0 + slot].set_black(pack_color(emp_black));

        if self.occupied[slot] {
            let init = f32::from(self.panes[A_ICN0 + slot].init_alpha);
            self.panes[A_ICN0 + slot].set_now_alpha((init - (init - 100.0) * t) / init);
        }
        let init = f32::from(self.panes[A_EMP0 + slot].init_alpha);
        self.panes[A_EMP0 + slot].set_now_alpha((init - (init - 100.0) * t) / init);
    }

    pub fn change_data(&mut self, out: &mut AlbumOutput) {
        let mut j = 0usize;
        for i in 0..SLOT_COUNT {
            if self.occupied[i] {
                if i != j {
                    self.occupied[i] = false;
                    self.occupied[j] = true;
                    self.photos[j] = self.photos[i].clone();
                }
                let mut record = self.photos[i].clone();
                record.set_checksum();
                self.photos[i] = record;
                out.saved[j] = Some(self.photos[i].clone());
                j += 1;
            }
        }
        out.erase_bits = Some(0);
    }

    pub fn label_sort(&mut self, input: &AlbumInput) -> bool {
        let mut count = 0usize;
        for index in 0..SLOT_COUNT {
            self.sort_map[index] = SORT_EMPTY;
            if self.get_pic_label_data(input, index as u8) == 0 {
                self.sort_map[count] = index as u8;
                count += 1;
            }
        }
        true
    }

    pub fn get_pic_label_data(&self, input: &AlbumInput, slot: u8) -> u8 {
        let reg = input.story.reg(EVENT_PICTURE_ERASE);
        if slot < 3 { reg & (1 << slot) } else { 0 }
    }

    fn shutter_line_geometry(&self, angle: f32, index: usize) -> (f32, f32, f32, f32) {
        let anchor_x = self.panes[A_ST0 + index].pos_top_left_orig.x
            + self.panes[A_SB0 + index].size_orig.x / 2.0;
        let y =
            self.panes[A_ST0 + index].pos_top_left.y - self.panes[A_SB0 + index].size_orig.y / 2.0;
        let anchor_2 = anchor_x + self.panes[A_ST0 + index].size_orig.x;
        let rad = PI * (angle / 180.0);
        let (sin1, cos1) = rad.sin_cos();
        let a = anchor_x * cos1 - y * sin1;
        let b = anchor_x * sin1 + y * cos1;
        let c = anchor_2 * cos1 - y * sin1;
        let d = anchor_2 * sin1 + y * cos1;
        (a, b, c, d)
    }

    pub fn shutter_line_rotate_center(&mut self, angle: f32, index: usize) {
        let (a, b, c, d) = self.shutter_line_geometry(angle, index);
        let center = self.panes[A_NO0].pos_center_orig;
        self.line_x1[index] = a + center.x;
        self.line_y1[index] = b + center.y;
        self.line_x2[index] = c + center.x;
        self.line_y2[index] = d + center.y;
    }

    pub fn shutter_line_rotate_init_pos(&mut self, angle: f32, index: usize) {
        let (a, b, c, d) = self.shutter_line_geometry(angle, index);
        let center = self.panes[A_SB0 + index].pos_center_orig;
        self.line_x1[index] = a + center.x;
        self.line_y1[index] = b + center.y;
        self.line_x2[index] = c + center.x;
        self.line_y2[index] = d + center.y;
    }

    pub fn shutter_line_move(&mut self) {
        let mut intersection_x = [0.0f32; SHUTTER_SEGMENT_COUNT];
        let mut intersection_y = [0.0f32; SHUTTER_SEGMENT_COUNT];

        for index in 0..SHUTTER_SEGMENT_COUNT {
            let next = if index == SHUTTER_SEGMENT_COUNT - 1 {
                0
            } else {
                index + 1
            };

            let x1 = self.line_x1[index];
            let x2 = self.line_x2[index];

            let (a1, b1, c1) = if x1 != x2 {
                let y1 = self.line_y1[index];
                let a = -(y1 - self.line_y2[index]) / (x1 - x2);
                (a, 1.0, y1 + a * x1)
            } else {
                (1.0, 0.0, x1)
            };

            let mut intercept = self.line_x1[next];
            let next_x2 = self.line_x2[next];

            let (a2, b2) = if intercept != next_x2 {
                let next_y = self.line_y1[next];
                let a = -(next_y - self.line_y2[next]) / (intercept - next_x2);
                intercept = next_y + a * intercept;
                (a, 1.0)
            } else {
                (1.0, 0.0)
            };

            let determinant = a1 * b2 - a2 * b1;
            if determinant != 0.0 {
                intersection_x[index] = (b2 * c1 - b1 * intercept) / determinant;
                intersection_y[index] = (-a2 * c1 + a1 * intercept) / determinant;
            } else {
                intersection_x[index] = x1;
                intersection_y[index] = self.line_y1[index];
            }
        }

        self.line_x1 = intersection_x;
        self.line_y1 = intersection_y;
    }

    pub fn shutter_line_draw(&self, alpha: u8) -> Vec<LineSegment> {
        (0..SHUTTER_SEGMENT_COUNT)
            .map(|index| LineSegment {
                x1: self.line_x1[index],
                y1: self.line_y1[index],
                x2: self.line_x2[index],
                y2: self.line_y2[index],
                color: pack_color([0xFF, 0xFF, 0xFF, alpha]),
            })
            .collect()
    }

    pub fn message_set(&mut self, msg_no: u32, env: &mut MessageEnv<'_>) {
        let mut get = ItemMsgGet::default();
        self.icon_main.visible = false;
        self.icon_sub.visible = false;
        self.select_arrow.visible = false;
        self.icon_main.alpha = 0;
        self.icon_sub.alpha = 0;
        self.select_arrow.alpha = 0;
        self.choice_arrow_idx = -1;
        self.icon_timer = -1;
        self.selected_choice = 0;

        let Some(head) = get.get_mesg_header(env.archive, env.clear_count, msg_no) else {
            return;
        };
        let Some(message) = get.get_message(head) else {
            return;
        };
        let message = message.to_vec();
        self.msg_entry = get.get_mesg_entry(head).unwrap_or_default();

        let color = crate::text::msglayout::fop_msgm_get_color_table(env.color_table, 0);
        let color_tag = format!("\u{1b}CC[{color:08x}]\u{1b}GM[0]");
        let white_tag = "\u{1b}CC[000000FF]\u{1b}GM[0]".to_owned();
        self.text_buffer = [color_tag.clone(), color_tag, white_tag.clone(), white_tag];

        self.msg_proc = MsgData::new_data_proc();
        self.msg_proc.set_bmg(message);
        self.msg_proc.entry = self.msg_entry;
        self.msg_proc.out = [
            Some(self.text_buffer[0].clone()),
            Some(self.text_buffer[1].clone()),
            Some(self.text_buffer[2].clone()),
            Some(self.text_buffer[3].clone()),
        ];
        self.msg_proc.char_space = self.panes[A_TX0].get_char_space() as i32;
        self.msg_proc.ruby_char_space = self.panes[A_TX0 + 1].get_char_space() as i32;
        self.msg_proc.line_space = self.panes[A_TX0].get_line_space() as i32;
        self.msg_proc.font_size = self.panes[A_TX0].font_size_x as i32;
        self.msg_proc.ruby_font_size = self.panes[A_TX0 + 1].font_size_x as i32;
        self.msg_proc.line_width = ALBUM_LINE_WIDTH;
        self.msg_proc.center_line_width = ALBUM_CENTER_LINE_WIDTH;
        self.msg_proc.send_speed = MSG_SEND_SPEED;
        self.msg_proc.space_timer = MSG_SPACE_TIMER;
        self.msg_proc.field_0x299 = 1;
        self.msg_proc.space_flag = u8::from(MSG_SPACE_FLAG != 0);
        self.msg_proc.string_length(
            env.font,
            MsgLenState {
                clear_count: env.clear_count,
                ..Default::default()
            },
        );
        self.msg_proc.string_shift();
        self.msg_proc.icon_idx_refresh();

        self.line_count = self.msg_proc.line_count as i16;
        self.msg_proc.line_count = 0;

        let half_line = self.panes[A_TX0].get_line_space() / 2.0;
        let shift_y = (ALBUM_UNUSED_LINES as f32 - self.line_count as f32) * half_line;
        for index in 0..MESSAGE_BUFFER_COUNT {
            self.panes[A_TX0 + index].shift_set(0.0, shift_y);
        }

        self.msg_proc
            .string_set(env.font, &MsgLayoutState::default());

        for index in 0..MESSAGE_BUFFER_COUNT {
            let text = self.msg_proc.out[index].clone().unwrap_or_default();
            self.text_buffer[index] = text.clone();
            self.panes[A_TX0 + index].set_string(&text);
        }

        let base_x = self.panes[A_TX0].pos_top_left.x as i32;
        let base_y = self.panes[A_TX0].pos_top_left.y as i32;
        let mut first_choice = 0u8;
        for index in 0..ICON_SHEET_LEN {
            let icon = self.msg_proc.icon_num[index];
            let font_color = self.msg_proc.string_color;
            let pos_x = self.msg_proc.icon_pos_x[index];
            let pos_y = self.msg_proc.icon_pos_y[index];
            if icon == ICON_YOKO_LEFT {
                self.choice_cursor_x0 = base_x + pos_x;
                self.choice_cursor_y =
                    base_y + (half_line * (ALBUM_UNUSED_LINES as f32 + pos_y as f32 * 2.0)) as i32;
                self.select_arrow.show();
                self.select_arrow.alpha = (0xFF);
                self.choice_arrow_idx = index as i8;
                if first_choice == 0 {
                    first_choice = 1;
                    self.selected_choice = 1;
                } else {
                    first_choice = 0;
                    self.selected_choice = 0;
                }
            } else if icon == ICON_YOKO_RIGHT {
                self.choice_cursor_x1 = base_x + pos_x;
                self.choice_cursor_y_alt = base_y
                    + (half_line * ((1.0 - f32::from(self.line_count)) + pos_y as f32 * 2.0))
                        as i32;
                self.select_arrow.show();
                self.select_arrow.set_alpha(0xFF);
                self.choice_arrow_idx = index as i8;
                if first_choice != 0 {
                    self.selected_choice = 1;
                } else {
                    self.selected_choice = 0;
                }
            } else if icon != ICON_NONE && self.icon_timer == -1 {
                fop_msgm_out_font_set(
                    &mut self.icon_main,
                    &mut self.icon_sub,
                    &mut self.icon_timer,
                    font_color,
                    icon,
                );
            }
        }

        for index in [1usize, 3usize] {
            if self.ruby_enabled {
                self.panes[A_TX0 + index].hide();
            } else if self.panes[A_TX0 + index].is_visible() {
                self.panes[A_TX0 + index].show();
            }
        }
    }

    pub fn flush_message(&mut self, env: &mut MessageEnv<'_>) {
        if let Some(msg_no) = self.pending_message.take() {
            self.message_set(msg_no, env);
        }
    }

    pub fn remain_message_set(&self, slot: u8) -> u32 {
        match slot {
            0 => MSG_REMAIN_EMPTY,
            1 => MSG_REMAIN_ONE,
            2 => MSG_REMAIN_TWO,
            _ => MSG_REMAIN_THREE,
        }
    }

    pub fn exist_message_set(&self, slot: u8) -> u32 {
        match slot {
            0 => MSG_EXIST_EMPTY,
            1 => MSG_EXIST_ONE,
            2 => MSG_EXIST_TWO,
            _ => MSG_EXIST_THREE,
        }
    }

    pub fn shutter_show(&mut self) {
        for index in 0..SHUTTER_SEGMENT_COUNT {
            self.panes[A_SB0 + index].show();
            self.panes[A_ST0 + index].show();
        }
    }

    pub fn shutter_hide(&mut self) {
        for index in 0..SHUTTER_SEGMENT_COUNT {
            self.panes[A_SB0 + index].hide();
            self.panes[A_ST0 + index].hide();
        }
    }

    pub fn create(&mut self, input: &AlbumInput, out: &mut AlbumOutput) {
        self.icon_main.blend_draw(OUT_FONT_TEX);
        self.icon_sub.blend_draw(OUT_FONT_TEX);
        self.select_arrow.blend_draw(SELECT_ARROW_TEX);
        self.icon_main.visible = false;
        self.icon_sub.visible = false;
        self.select_arrow.visible = false;
        self.icon_main.alpha = 0;
        self.icon_sub.alpha = 0;
        self.select_arrow.set_alpha(0);
        self.icon_timer = -1;

        self.ruby_enabled = input.opt_ruby;
        self.picture_num = input.picture_num;
        self.screen_set();
        self.screen_set2(input.opt_ruby);

        self.zoom_scale = 0.0;
        self.fade_timer = 0;
        self.shutter_counter = 0;
        self.mode_swap = 0;
        self.choice_arrow_idx = -1;

        if input.picture_status == 2 {
            self.exec_state = STATE_BROWSE_OPEN;
            self.view_mode = MODE_BROWSE;
        } else if input.picture_status == 3 {
            self.exec_state = STATE_GET_OPEN;
            self.view_mode = MODE_GET;
        } else {
            self.exec_state = STATE_CAMERA_OPEN;
            self.view_mode = MODE_CAMERA;
        }

        self.sub_state = SUB_IDLE;
        self.zoom_scale(input, out);
        self.selected_slot = 0;
        self.photo_deleted = 0;
        self.capture_format = TEXTURE_I8;
        self.line_count = 0;
        self.choice_cursor_y_alt = 0;
        self.choice_cursor_y = 0;
        self.choice_cursor_x1 = 0;
        self.choice_cursor_x0 = 0;
        self.label_sort(input);

        for index in 0..SLOT_COUNT {
            self.occupied[index] = false;
            if usize::from(input.picture_num) > index && self.sort_map[index] != SORT_EMPTY {
                self.occupied[index] = true;
                let source = usize::from(self.sort_map[index]).min(SLOT_COUNT - 1);
                self.photos[index] = input.saved[source].clone();
            }
        }

        for index in 0..SHUTTER_SEGMENT_COUNT {
            let angle = f32::from(self.panes[A_SB0 + index].user_area);
            self.shutter_line_rotate_init_pos(angle, index);
        }
    }

    pub fn camera_open(&mut self, input: &AlbumInput, out: &mut AlbumOutput) {
        if self.fade_timer == 0 {
            self.pending_message = Some(self.remain_message_set(input.picture_num));
        }
        self.fade_timer += 1;
        let t = value_increase(i32::from(FADE_FRAMES), i32::from(self.fade_timer), 0);
        self.camera_alpha_inc(t);
        if self.fade_timer == FADE_FRAMES {
            out.scope_status = Some(STATUS_SCOPE_ACTIVE);
            self.exec_state = STATE_CAMERA_MOVE;
            out.sound(SE_SUBMENU_IN_2);
        }
        out.do_status_force = Some(ACT_BLANK);
        out.a_status_force = Some(ACT_BLANK);
        self.up_down_icon_move(input);
    }

    pub fn browse_open(&mut self, input: &AlbumInput, out: &mut AlbumOutput) {
        if self.fade_timer == 0 {
            self.pending_message = Some(if input.picture_num == 0 {
                MSG_NO_PHOTO
            } else {
                self.exist_message_set(input.picture_num)
            });
        }
        self.fade_timer += 1;
        let t = value_increase(i32::from(FADE_FRAMES), i32::from(self.fade_timer), 0);
        self.browse_alpha_inc(t);
        if self.fade_timer == FADE_FRAMES {
            out.scope_status = Some(STATUS_SCOPE_ACTIVE);
            self.exec_state = STATE_BROWSE_MOVE;
            out.sound(SE_SUBMENU_IN_2);
        }
        out.do_status_force = Some(ACT_BLANK);
        out.a_status_force = Some(ACT_BLANK);
    }

    pub fn get_open(&mut self, input: &AlbumInput, out: &mut AlbumOutput) {
        if self.fade_timer == 0 {
            self.pending_message = Some(MSG_GET_PHOTO);
        }
        if self.fade_timer >= FADE_FRAMES {
            if input.import_ready {
                let index = usize::from(input.get_picture_num).min(PHOTO_INDEX.len() - 1);
                let mut record = PhotoRecord::from_texture(&input.import_texture);
                record.snap_result = u32::from(PHOTO_INDEX[index]);
                record.snap_result_detail = 0;
                record.capture_format = TEXTURE_CMPR;
                self.photos[3] = record;
                out.scope_status = Some(STATUS_SCOPE_ACTIVE);
                self.exec_state = STATE_GET_MOVE;
                out.sound(SE_SUBMENU_IN_2);
            }
        } else {
            self.fade_timer += 1;
            let t = value_increase(i32::from(FADE_FRAMES), i32::from(self.fade_timer), 0);
            self.get_alpha_inc(t);
        }
        out.do_status_force = Some(ACT_BLANK);
        out.a_status_force = Some(ACT_BLANK);
    }

    pub fn close(&mut self, out: &mut AlbumOutput) {
        self.fade_timer -= 1;
        let t = value_increase(i32::from(FADE_FRAMES), i32::from(self.fade_timer), 0);
        self.alpha_dec(t);
        if self.fade_timer == 0 {
            self.change_data(out);
            self.exec_state = STATE_CLOSED;
            out.menu_flag = Some(0);
        }
        out.do_status_force = Some(ACT_BLANK);
        out.a_status_force = Some(ACT_BLANK);
    }

    pub fn camera_move(
        &mut self,
        input: &AlbumInput,
        pad: &PadInput,
        out: &mut AlbumOutput,
        tune: &AlbumTune,
    ) {
        if self.mode_swap == 0 {
            if self.view_mode == MODE_CAMERA {
                self.move_camera(input, pad, out, tune);
            } else {
                self.move_browse(input, pad, out, tune);
            }
        } else {
            self.shutter_change(tune, out);
        }
    }

    pub fn browse_move(
        &mut self,
        input: &AlbumInput,
        pad: &PadInput,
        out: &mut AlbumOutput,
        tune: &AlbumTune,
    ) {
        self.select_browse(input, pad, out, tune);
    }

    pub fn get_move(&mut self, input: &AlbumInput, pad: &PadInput, out: &mut AlbumOutput) {
        self.get_browse(input, pad, out);
    }

    pub fn draw(&mut self, tune: &AlbumTune) -> AlbumDraw {
        if self.exec_state != STATE_CLOSE && self.exec_state != STATE_CLOSED {
            for index in 0..SHUTTER_SEGMENT_COUNT {
                self.panes[A_SB0 + index].set_init_alpha();
                self.panes[A_ST0 + index].set_init_alpha();
            }
            for index in 0..SLOT_COUNT {
                self.panes[A_ICN0 + index].set_init_alpha();
                self.panes[A_EMP0 + index].set_init_alpha();
            }
            for index in 0..8 {
                self.panes[A_SP0 + index].set_init_alpha();
            }
            for index in 0..SLOT_COUNT {
                self.panes[A_NO0 + index].set_init_alpha();
                self.panes[A_NOB0 + index].set_init_alpha();
                self.panes[A_NOK0 + index].set_init_alpha();
            }
            for index in [
                A_YLIG, A_YLEF, A_CT1, A_CT2, A_B1, A_B2, A_WNUM, A_WNUK, A_WPBA, A_WBAP, A_WPSC,
                A_YRTN, A_YZOM, A_CRTN, A_CZOM, A_LRTN, A_RZOM, A_SHUT, A_FD00, A_WP03, A_WP02,
                A_WP01, A_WP04,
            ] {
                self.panes[index].set_init_alpha();
            }
            for index in 0..MESSAGE_BUFFER_COUNT {
                self.panes[A_TX0 + index].set_init_alpha();
            }
        }

        let alpha = self.panes[A_TX0].now_alpha;
        let photos = self.picture_quads(alpha);
        let order: Vec<usize> = (0..ALBUM_PANE_COUNT).collect();
        let mut panes = PaneDraw::default();
        panes.emit(&self.panes, &order);
        let lines = self.shutter_line_draw(tune.shutter_line_alpha);
        let icons = self.draw_icons();
        AlbumDraw {
            panes,
            photos,
            lines,
            icons,
            draw_shutter_first: self.mode_swap == 0,
        }
    }

    fn draw_icons(&mut self) -> Vec<IconQuad> {
        let mut list = Vec::new();
        let alpha = self.panes[A_TX0].now_alpha;
        let half_line = (self.panes[A_TX0].get_line_space() / 2.0) as i32;
        let base_x = self.panes[A_TX0].pos_top_left.x as i32;
        let base_y = self.panes[A_TX0].pos_top_left.y as i32;
        for index in 0..ICON_SHEET_LEN {
            if index == self.choice_arrow_idx.max(0) as usize && self.choice_arrow_idx >= 0 {
                let x = self.choice_cursor_x0 as f32
                    + f32::from(self.selected_choice)
                        * (self.choice_cursor_x1 - self.choice_cursor_x0) as f32;
                let y = self.choice_cursor_y as f32;
                let _ = (x, y);
                self.select_arrow.rotate(SELECT_ARROW_ANGLE);
                self.select_arrow
                    .resize(MSG_FONT_SIZE as f32, MSG_FONT_SIZE as f32);
                self.arrow_anime();
                list.push(IconQuad {
                    x: self.select_arrow.pos[0],
                    y: self.select_arrow.pos[1],
                    w: self.select_arrow.size[0],
                    h: self.select_arrow.size[1],
                    rotation: self.select_arrow.rot_z,
                    white: self.select_arrow.white,
                    black: self.select_arrow.black,
                    alpha: self.select_arrow.alpha,
                    icon: index,
                });
            } else {
                let icon_no = self.msg_proc.icon_num[index];
                if icon_no == ICON_NONE || icon_no == ICON_YOKO_LEFT || icon_no == ICON_YOKO_RIGHT {
                    continue;
                }
                let pos_x = self.msg_proc.icon_pos_x[index];
                let pos_y = self.msg_proc.icon_pos_y[index];
                let scale = self.msg_proc.icon_scale[index];
                let x = (pos_x + base_x) as f32;
                let y = (half_line * (ALBUM_UNUSED_LINES + pos_y * 2) + base_y) as f32;
                fop_msgm_out_font_draw(
                    &meter_msg_tune(),
                    &mut self.icon_main,
                    &mut self.icon_sub,
                    x as i32,
                    y as i32,
                    scale,
                    &mut self.icon_timer,
                    alpha,
                    icon_no,
                );
                list.push(IconQuad {
                    x: self.icon_main.pos[0],
                    y: self.icon_main.pos[1],
                    w: self.icon_main.size[0],
                    h: self.icon_main.size[1],
                    rotation: self.icon_main.rot_z,
                    white: self.icon_main.white,
                    black: self.icon_main.black,
                    alpha: self.icon_main.alpha,
                    icon: index,
                });
            }
        }
        list
    }

    fn arrow_anime(&mut self) {
        let (white, black, alpha) = select_arrow_anime(&mut self.arrow_timer);
        self.select_arrow.set_white(white);
        self.select_arrow.set_black(black);
        self.select_arrow.set_alpha(alpha);
    }

    fn select_check(&mut self) -> u8 {
        self.msg_proc.select_check_yoko(
            MSG_FONT_SIZE as f32,
            MSG_FONT_SIZE as f32,
            MSG_FONT_SIZE as i16,
            self.choice_cursor_x0,
            self.choice_cursor_y,
            self.choice_cursor_x1 - self.choice_cursor_x0,
            0.0,
        )
    }

    pub fn delete(&mut self, out: &mut AlbumOutput) {
        out.capture_cansel = true;
        out.heap_lock = Some(0);
        out.scope_status = Some(STATUS_UNKNOWN0);
        out.do_status_force = Some(ACT_BLANK);
        out.a_status_force = Some(ACT_BLANK);
        out.r_status_force = Some(ACT_BLANK);
        out.r_status = Some(ACT_BLANK);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LineSegment {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub color: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct IconQuad {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub rotation: f32,
    pub white: [u8; 4],
    pub black: [u8; 4],
    pub alpha: u8,
    pub icon: usize,
}

#[derive(Clone, Debug, Default)]
pub struct AlbumDraw {
    pub panes: PaneDraw,
    pub photos: Vec<PhotoQuad>,
    pub lines: Vec<LineSegment>,
    pub icons: Vec<IconQuad>,
    pub draw_shutter_first: bool,
}

pub fn erase_picture(select_picture: u8, story: &mut StoryBits, out: &mut AlbumOutput) {
    let reg = story.reg(EVENT_PICTURE_ERASE);
    if select_picture < 3 {
        let next = reg | (1 << select_picture);
        story.set_reg(EVENT_PICTURE_ERASE, next);
        out.erase_bits = Some(next);
        out.picture_num_count = -1;
    }
}

pub fn select_arrow_anime(timer: &mut i16) -> ([u8; 4], [u8; 4], u8) {
    let low = ARROW_COLOR_LOW;
    let high = ARROW_COLOR_HIGH;
    let mut white = [0u8; 4];
    let mut black = [0u8; 4];
    white[3] = 0xFF;
    let alpha = if *timer < ARROW_ANIME_SPAN {
        let t = value_increase(i32::from(ARROW_ANIME_SPAN), i32::from(*timer), 2);
        for channel in 0..3 {
            let base =
                t * (f32::from(high[channel]) - f32::from(low[channel])) + f32::from(low[channel]);
            black[channel] = base as u8;
            white[channel] = base as u8;
        }
        (ARROW_ALPHA_LOW as f32 + t * (f32::from(ARROW_ALPHA_HIGH) - f32::from(ARROW_ALPHA_LOW)))
            as u8
    } else {
        let t = value_increase(
            i32::from(ARROW_ANIME_SPAN),
            i32::from(*timer - ARROW_ANIME_SPAN),
            2,
        );
        for channel in 0..3 {
            let base =
                t * (f32::from(low[channel]) - f32::from(high[channel])) + f32::from(high[channel]);
            black[channel] = base as u8;
            white[channel] = base as u8;
        }
        (ARROW_ALPHA_HIGH as f32 + t * (f32::from(ARROW_ALPHA_LOW) - f32::from(ARROW_ALPHA_HIGH)))
            as u8
    };
    *timer += 1;
    if *timer >= ARROW_ANIME_PERIOD {
        *timer = 0;
    }
    (white, black, alpha)
}

fn scale_pane(pane: &mut Pane, scale: f32) {
    pane.now_alpha = (f32::from(pane.now_alpha) * scale) as u8;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AlbumHost {
    pub heap_lock: u8,
    pub picture_num: u8,
    pub status: u8,
}

impl AlbumHost {
    pub fn create(&mut self, picture_num: &mut u8, out: &mut AlbumOutput) -> u8 {
        if self.heap_lock != 0 && self.heap_lock != 6 {
            return 0;
        }
        if *picture_num > 3 {
            *picture_num = 0;
        }
        self.picture_num = *picture_num;
        self.heap_lock = 6;
        out.heap_lock = Some(6);
        out.scope_status = Some(STATUS_UNKNOWN0);
        self.status = 1;
        1
    }

    pub fn execute(
        &mut self,
        box_: &mut AlbumBox,
        input: &AlbumInput,
        pad: &PadInput,
        out: &mut AlbumOutput,
        tune: &AlbumTune,
    ) {
        match box_.get_status() {
            STATE_CAMERA_OPEN => box_.camera_open(input, out),
            STATE_BROWSE_OPEN => box_.browse_open(input, out),
            STATE_GET_OPEN => box_.get_open(input, out),
            STATE_CAMERA_MOVE => {
                box_.camera_move(input, pad, out, tune);
                out.camera_attention = true;
            }
            STATE_BROWSE_MOVE => box_.browse_move(input, pad, out, tune),
            STATE_GET_MOVE => box_.get_move(input, pad, out),
            STATE_CLOSE => {
                box_.close(out);
                if box_.get_status() == STATE_CLOSED {
                    self.status = STATUS_BOX_CLOSED;
                }
            }
            STATE_CLOSED => {
                if self.status == STATUS_MSG_DESTROYED {
                    self.status = STATUS_MSG_DESTROYED;
                }
            }
            _ => {}
        }
    }

    pub fn destroy(&mut self, out: &mut AlbumOutput) {
        self.heap_lock = 0;
        self.status = 0;
        out.heap_lock = Some(0);
    }
}
