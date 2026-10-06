use core::f32::consts::PI;

use bevy_ecs::prelude::*;

use crate::j2d::{self, Kind, Screen};
use crate::sim::inventory::Equipped;
use crate::text::font::FontBank;
use crate::text::msgdata::MsgLenState;
use crate::text::msgget::{ItemMsgGet, MsgArchive};
use crate::text::msglayout::MsgLayoutState;
use crate::text::{MesgEntry, MsgData};

pub const SCREEN_LABEL: &str = "menu_item_02.blo";
pub const ICON_ARCHIVE: &str = "Item_res.arc";
pub const CLOTH_ARCHIVE: &str = "Menu_res.arc";
pub const CLOTH_TEXTURE: &str = "cloth_piece01.bti";
pub const CURSOR_TEXTURE: &str = "cursor_00_02.bti";
pub const RETURN_TEXTURE: &str = "cover_return.bti";
pub const BLEND_SHADOW_TEXTURE: &str = "font_00.bti";
pub const OUT_FONT_TEXTURE: &str = "font_07_02.bti";
pub const TILE_SIZE: usize = 0xC00;
pub const TEXT_BUFFER_SIZE: usize = 0x200;
pub const NAME_BUFFER_SIZE: usize = 0x20;
pub const TEXT_BUFFER_COUNT: usize = 33;
pub const NAME_BUFFER_COUNT: usize = 2;
pub const ITEM_BUFFER_COUNT: usize = 21;
pub const SUB_BUFFER_COUNT: usize = 9;
pub const SUB_ITEM_COUNT: usize = 8;
pub const CHILD_HEAP_SIZE: usize = 0x506A1;

pub const SLOT_COUNT: usize = 21;
pub const SLOT_FIRST: u8 = 0;
pub const SLOT_TELESCOPE: u8 = 0;
pub const SLOT_SAIL: u8 = 1;
pub const SLOT_WIND_WAKER: u8 = 2;
pub const SLOT_GRAPPLING_HOOK: u8 = 3;
pub const SLOT_SPOILS_BAG: u8 = 4;
pub const SLOT_BOOMERANG: u8 = 5;
pub const SLOT_DEKU_LEAF: u8 = 6;
pub const SLOT_TINGLE_TUNER: u8 = 7;
pub const SLOT_CAMERA: u8 = 8;
pub const SLOT_IRON_BOOTS: u8 = 9;
pub const SLOT_MAGIC_ARMOR: u8 = 10;
pub const SLOT_BAIT_BAG: u8 = 11;
pub const SLOT_BOW: u8 = 12;
pub const SLOT_BOMB: u8 = 13;
pub const SLOT_BOTTLE0: u8 = 14;
pub const SLOT_BOTTLE1: u8 = 15;
pub const SLOT_BOTTLE2: u8 = 16;
pub const SLOT_BOTTLE3: u8 = 17;
pub const SLOT_DELIVERY_BAG: u8 = 18;
pub const SLOT_HOOKSHOT: u8 = 19;
pub const SLOT_SKULL_HAMMER: u8 = 20;
pub const SLOT_LAST: u8 = 21;
pub const SLOT_BEAST_FIRST: u8 = 24;
pub const SLOT_BEAST_LAST: u8 = 32;
pub const SLOT_BAIT_FIRST: u8 = 36;
pub const SLOT_BAIT_LAST: u8 = 44;
pub const SLOT_RESERVE_FIRST: u8 = 48;
pub const SLOT_RESERVE_LAST: u8 = 56;
pub const SLOT_NONE: u8 = 0xFF;
pub const GRID_COLUMNS: u8 = 7;

pub const BTN_X: u8 = 0;
pub const BTN_Y: u8 = 1;
pub const BTN_Z: u8 = 2;
pub const BTN_NONE: u8 = 3;
pub const BTN_COUNT: usize = 3;
pub const SELECT_SLOTS: usize = 5;

pub const ITEM_NONE: u8 = 0xFF;
pub const ITEM_TELESCOPE: u8 = 0x20;
pub const ITEM_TINGLE_TUNER: u8 = 0x21;
pub const ITEM_WIND_WAKER: u8 = 0x22;
pub const ITEM_PICTO_BOX: u8 = 0x23;
pub const ITEM_SPOILS_BAG: u8 = 0x24;
pub const ITEM_GRAPPLING_HOOK: u8 = 0x25;
pub const ITEM_DELUXE_PICTO_BOX: u8 = 0x26;
pub const ITEM_BOW: u8 = 0x27;
pub const ITEM_IRON_BOOTS: u8 = 0x29;
pub const ITEM_MAGIC_ARMOR: u8 = 0x2A;
pub const ITEM_BAIT_BAG: u8 = 0x2C;
pub const ITEM_BOOMERANG: u8 = 0x2D;
pub const ITEM_HOOKSHOT: u8 = 0x2F;
pub const ITEM_DELIVERY_BAG: u8 = 0x30;
pub const ITEM_BOMB_BAG: u8 = 0x31;
pub const ITEM_SKULL_HAMMER: u8 = 0x33;
pub const ITEM_DEKU_LEAF: u8 = 0x34;
pub const ITEM_MAGIC_ARROW: u8 = 0x35;
pub const ITEM_LIGHT_ARROW: u8 = 0x36;
pub const ITEM_JOY_PENDANT: u8 = 0x1F;
pub const ITEM_EMPTY_BOTTLE: u8 = 0x50;
pub const ITEM_RED_POTION: u8 = 0x51;
pub const ITEM_GREEN_POTION: u8 = 0x52;
pub const ITEM_BLUE_POTION: u8 = 0x53;
pub const ITEM_WATER_BOTTLE: u8 = 0x56;
pub const ITEM_FIREFLY_BOTTLE: u8 = 0x58;
pub const ITEM_FOREST_WATER: u8 = 0x59;
pub const ITEM_SAIL: u8 = 0x78;
pub const ITEM_HYOI_PEAR: u8 = 0x83;
pub const ITEM_BIRD_BAIT_5: u8 = 0x82;
pub const ITEM_SKULL_NECKLACE: u8 = 0x45;
pub const ITEM_BOBOBA_SEED: u8 = 0x46;
pub const ITEM_GOLDEN_FEATHER: u8 = 0x47;
pub const ITEM_KNIGHTS_CREST: u8 = 0x48;
pub const ITEM_RED_JELLY: u8 = 0x49;
pub const ITEM_GREEN_JELLY: u8 = 0x4A;
pub const ITEM_BLUE_JELLY: u8 = 0x4B;
pub const ITEM_TOWN_FLOWER: u8 = 0x8C;
pub const ITEM_SEA_FLOWER: u8 = 0x8D;
pub const ITEM_EXOTIC_FLOWER: u8 = 0x8E;
pub const ITEM_HEROS_FLAG: u8 = 0x8F;
pub const ITEM_BIG_CATCH_FLAG: u8 = 0x90;
pub const ITEM_BIG_SALE_FLAG: u8 = 0x91;
pub const ITEM_PINWHEEL: u8 = 0x92;
pub const ITEM_SICKLE_MOON_FLAG: u8 = 0x93;
pub const ITEM_SKULL_TOWER_IDOL: u8 = 0x94;
pub const ITEM_FOUNTAIN_IDOL: u8 = 0x95;
pub const ITEM_POSTMAN_STATUE: u8 = 0x96;
pub const ITEM_SHOP_GURU_STATUE: u8 = 0x97;

pub const RECOLLECT_STAGE_TYPE: u8 = 4;
pub const RECOLLECT_STAGES: [&str; 4] = ["Xboss0", "Xboss1", "Xboss2", "Xboss3"];
pub const FOREST_WATER_RESOURCE: u32 = 0x2D;
pub const FOREST_WATER_SCREEN_X: f32 = 320.0;
pub const FOREST_WATER_SCREEN_Y: f32 = 240.0;

pub const SE_MENU_CURSOR: u32 = 0x080E;
pub const SE_MENU_IN: u32 = 0x080F;
pub const SE_MENU_OUT: u32 = 0x0810;
pub const SE_MENU_PANEL_IN: u32 = 0x0811;
pub const SE_MENU_ITEMS_IN: u32 = 0x0813;
pub const SE_MENU_EXP_IN: u32 = 0x0814;
pub const SE_MENU_PAGE: u32 = 0x0815;
pub const SE_MENU_EXP_OUT: u32 = 0x0816;
pub const SE_MENU_SET: u32 = 0x0817;
pub const SE_MENU_DECIDE: u32 = 0x084A;
pub const SE_MENU_CANCEL: u32 = 0x084B;
pub const SE_MENU_OPT_IN: u32 = 0x084C;
pub const SE_MENU_MAP_IN: u32 = 0x0850;
pub const SE_ITEM_COL_SW: u32 = 0x08B5;
pub const SE_ITEM_EXP_OPEN: u32 = 0x08B6;
pub const SE_SHIPPU_CHART_OPEN: u32 = 0x08CB;

pub const DO_STATUS_FORCE_ITEM: u8 = 0x21;
pub const DO_STATUS_FORCE_LAST: u8 = 0x17;
pub const A_STATUS_FORCE_SAVE: u8 = 7;

pub const MSG_ITEM_LAST_SLOT: u32 = 0x1F8;
pub const MSG_BAG_RUBY: u32 = 0x1F5;
pub const MSG_ITEM_RUBY: u32 = 0x1F9;
pub const MSG_LAST_SLOT_RUBY: u32 = 0x1FC;
pub const MSG_NO_ITEM_NAME: u32 = 0x19C;
pub const MSG_NO_ITEM_NOTE: u32 = 0x264;
pub const NOTE_MESSAGE_BASE: u32 = 200;
pub const NOTE_UNUSED_LINES_USA: i32 = 3;
pub const NOTE_LINE_WIDTH: i32 = 510;
pub const NOTE_CENTER_LINE_WIDTH: i32 = 486;
pub const NOTE_SEND_SPEED: i32 = 2;
pub const NOTE_SPACE_TIMER: i32 = 0;
pub const NAME_CELL_SCALE: f32 = 29.0;
pub const RUBY_OPTION_DY: f32 = -4.0;
pub const RUBY_OPTION_PANE: usize = P_ST00;

pub const ICON_A_BUTTON: u8 = 0x00;
pub const ICON_B_BUTTON: u8 = 0x01;
pub const ICON_C_STICK: u8 = 0x02;
pub const ICON_L_BUTTON: u8 = 0x03;
pub const ICON_R_BUTTON: u8 = 0x04;
pub const ICON_X_BUTTON: u8 = 0x05;
pub const ICON_Y_BUTTON: u8 = 0x06;
pub const ICON_Z_BUTTON: u8 = 0x07;
pub const ICON_DPAD: u8 = 0x08;
pub const ICON_MAIN_STICK: u8 = 0x09;
pub const ICON_ARROW_LEFT: u8 = 0x0A;
pub const ICON_ARROW_RIGHT: u8 = 0x0B;
pub const ICON_ARROW_UP: u8 = 0x0C;
pub const ICON_ARROW_DOWN: u8 = 0x0D;
pub const ICON_MAIN_STICK_UP: u8 = 0x0E;
pub const ICON_MAIN_STICK_DOWN: u8 = 0x0F;
pub const ICON_MAIN_STICK_LEFT: u8 = 0x10;
pub const ICON_MAIN_STICK_RIGHT: u8 = 0x11;
pub const ICON_MAIN_STICK_UP_DOWN: u8 = 0x12;
pub const ICON_MAIN_STICK_LEFT_RIGHT: u8 = 0x13;
pub const ICON_FLASHING_A_BUTTON: u8 = 0x14;
pub const ICON_YOKO_LEFT: u8 = 0x14;
pub const ICON_HEART: u8 = 0x15;
pub const ICON_YOKO_RIGHT: u8 = 0x15;
pub const ICON_MUSIC_NOTE: u8 = 0x16;
pub const ICON_INPUT: u8 = 0x16;
pub const ICON_STARBURST: u8 = 0x17;
pub const ICON_NONE: u8 = 0xFF;
pub const ICON_FLASHING_A_BUTTON_2: u8 = 0x17;
pub const ICON_HEART_2: u8 = 0x18;
pub const ICON_MUSIC_NOTE_2: u8 = 0x19;
pub const ICON_STARBURST_2: u8 = 0x1A;

pub const CTRL_A_BUTTON: u32 = 0x0A;
pub const CTRL_B_BUTTON: u32 = 0x0B;
pub const CTRL_C_STICK: u32 = 0x0C;
pub const CTRL_L_BUTTON: u32 = 0x0D;
pub const CTRL_R_BUTTON: u32 = 0x0E;
pub const CTRL_X_BUTTON: u32 = 0x0F;
pub const CTRL_Y_BUTTON: u32 = 0x10;
pub const CTRL_Z_BUTTON: u32 = 0x11;
pub const CTRL_DPAD: u32 = 0x12;
pub const CTRL_MAIN_STICK: u32 = 0x13;
pub const CTRL_ARROW_LEFT: u32 = 0x14;
pub const CTRL_ARROW_RIGHT: u32 = 0x15;
pub const CTRL_ARROW_UP: u32 = 0x16;
pub const CTRL_ARROW_DOWN: u32 = 0x17;
pub const CTRL_MAIN_STICK_UP: u32 = 0x18;
pub const CTRL_MAIN_STICK_DOWN: u32 = 0x19;
pub const CTRL_MAIN_STICK_LEFT: u32 = 0x1A;
pub const CTRL_MAIN_STICK_RIGHT: u32 = 0x1B;
pub const CTRL_MAIN_STICK_UP_DOWN: u32 = 0x1C;
pub const CTRL_MAIN_STICK_LEFT_RIGHT: u32 = 0x1D;
pub const CTRL_FLASHING_A_BUTTON: u32 = 0x27;
pub const CTRL_HEART: u32 = 0x39;
pub const CTRL_MUSIC_NOTE: u32 = 0x3A;
pub const SPCL_COLOR: u32 = 0;
pub const SPCL_RUBY: u32 = 1;

pub const MSG_FONT_SIZE_USA: i32 = 0x17;
pub const MSG_LINE_SPACE_USA: i32 = 0x1E;
pub const MSG_STICK_A: i16 = 10;
pub const MSG_STICK_B: i16 = 0;
pub const MSG_STICK_C: i16 = 0;
pub const MSG_ICON_SCALE: i32 = 23;

pub const NUMBER_TEX: [&str; 10] = [
    "count_num_0.bti",
    "count_num_1.bti",
    "count_num_2.bti",
    "count_num_3.bti",
    "count_num_4.bti",
    "count_num_5.bti",
    "count_num_6.bti",
    "count_num_7.bti",
    "count_num_8.bti",
    "count_num_9.bti",
];

pub const BUTTON_TEX: [&str; 24] = [
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

pub const BUTTON_COLOR: [[u8; 4]; 24] = [
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

pub const ITEM_ICON_NO: [u8; 60] = [
    ITEM_TELESCOPE, ITEM_SAIL, ITEM_WIND_WAKER, ITEM_GRAPPLING_HOOK, ITEM_SPOILS_BAG, ITEM_BOOMERANG,
    ITEM_DEKU_LEAF, ITEM_TINGLE_TUNER, ITEM_PICTO_BOX, ITEM_IRON_BOOTS, ITEM_MAGIC_ARMOR, ITEM_BAIT_BAG,
    ITEM_BOW, ITEM_BOMB_BAG, ITEM_EMPTY_BOTTLE, ITEM_RED_POTION, ITEM_GREEN_POTION, ITEM_BLUE_POTION,
    ITEM_DELIVERY_BAG, ITEM_HOOKSHOT, ITEM_SKULL_HAMMER, ITEM_MAGIC_ARMOR, ITEM_MAGIC_ARMOR, ITEM_MAGIC_ARMOR,
    ITEM_SKULL_NECKLACE, ITEM_BOBOBA_SEED, ITEM_GOLDEN_FEATHER, ITEM_KNIGHTS_CREST, ITEM_RED_JELLY, ITEM_GREEN_JELLY,
    ITEM_BLUE_JELLY, ITEM_JOY_PENDANT, ITEM_NONE, ITEM_NONE, ITEM_NONE, ITEM_NONE,
    ITEM_HYOI_PEAR, ITEM_BIRD_BAIT_5, ITEM_NONE, ITEM_NONE, ITEM_NONE, ITEM_NONE,
    ITEM_NONE, ITEM_NONE, ITEM_NONE, ITEM_NONE, ITEM_NONE, ITEM_NONE,
    ITEM_TOWN_FLOWER, ITEM_SEA_FLOWER, ITEM_EXOTIC_FLOWER, ITEM_HEROS_FLAG, ITEM_BIG_CATCH_FLAG, ITEM_BIG_SALE_FLAG,
    ITEM_PINWHEEL, ITEM_SICKLE_MOON_FLAG, ITEM_SKULL_TOWER_IDOL, ITEM_FOUNTAIN_IDOL, ITEM_POSTMAN_STATUE, ITEM_SHOP_GURU_STATUE,
];

pub const ITEM_TEXTURE: [&str; 0x74] = [
    "get_rupy.bti",
    "get_key.bti",
    "heart_up_01.bti",
    "heart_up_02.bti",
    "telescope.bti",
    "whistle.bti",
    "baton.bti",
    "camera.bti",
    "coverofbeast.bti",
    "boomerang.bti",
    "rope.bti",
    "sail_00.bti",
    "fan.bti",
    "hammer_01.bti",
    "boots_00.bti",
    "boots_01.bti",
    "coverofbait.bti",
    "bow_01.bti",
    "bomb_00.bti",
    "sail_01.bti",
    "bottle_00.bti",
    "bottle_01.bti",
    "bottle_02.bti",
    "bottle_03.bti",
    "delivery.bti",
    "hookshot.bti",
    "shield_02.bti",
    "sail_02.bti",
    "beast_01.bti",
    "beast_02.bti",
    "beast_03.bti",
    "beast_04.bti",
    "beast_05.bti",
    "beast_06.bti",
    "beast_07.bti",
    "beast_08.bti",
    "bait_01.bti",
    "bait_02.bti",
    "bait_03.bti",
    "bait_04.bti",
    "bait_05.bti",
    "bait_06.bti",
    "bait_07.bti",
    "bait_08.bti",
    "baton_stone.bti",
    "god_symbol_00.bti",
    "god_symbol_01.bti",
    "god_symbol_02.bti",
    "sword_00.bti",
    "sword_01.bti",
    "shield_00.bti",
    "shield_01.bti",
    "gloves_00.bti",
    "gloves_00.bti",
    "amulet_00.bti",
    "amulet_01.bti",
    "delivery_01.bti",
    "delivery_02.bti",
    "delivery_03.bti",
    "delivery_04.bti",
    "delivery_05.bti",
    "delivery_06.bti",
    "delivery_07.bti",
    "delivery_08.bti",
    "delivery_09.bti",
    "delivery_10.bti",
    "delivery_11.bti",
    "delivery_12.bti",
    "delivery_13.bti",
    "delivery_14.bti",
    "delivery_15.bti",
    "delivery_16.bti",
    "delivery_17.bti",
    "delivery_18.bti",
    "dungeon_map.bti",
    "compass.bti",
    "boss_key.bti",
    "bottle_04.bti",
    "bottle_05.bti",
    "bottle_06.bti",
    "bottle_07.bti",
    "bottle_08.bti",
    "baton_stone.bti",
    "triforce_00.bti",
    "triforce_01.bti",
    "triforce_02.bti",
    "triforce_03.bti",
    "triforce_04.bti",
    "triforce_05.bti",
    "triforce_06.bti",
    "triforce_07.bti",
    "arrow_00.bti",
    "arrow_01.bti",
    "arrow_02.bti",
    "arrow_03.bti",
    "big_purse.bti",
    "max_purse.bti",
    "sword_02.bti",
    "sword_03.bti",
    "clothes.bti",
    "bombpouch_1.bti",
    "bombpouch_2.bti",
    "arrowcase_1.bti",
    "arrowcase_2.bti",
    "cmap_tri2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_phantomship2.bti",
    "cmap_tingle2.bti",
    "arrow_power_01.bti",
    "arrow_power_02.bti",
    "camera_2.bti",
    "tingle_figure.bti",
    "bottle_09.bti",
    "cmap_hint2.bti",
    "delivery_19.bti",
];

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
pub const P_SIN01: usize = 50;
pub const P_SIN10: usize = 58;
pub const P_SIT: usize = 66;
pub const P_SIK: usize = 74;
pub const P_IT29: usize = 82;
pub const P_IK29: usize = 83;
pub const P_SB0: usize = 84;
pub const P_BLAK: usize = 93;
pub const P_PANE_BG: usize = 94;
pub const P_WK01: usize = 95;
pub const P_WK00: usize = 96;
pub const P_0301: usize = 97;
pub const P_1301: usize = 98;
pub const P_1310: usize = 99;
pub const P_1401: usize = 100;
pub const P_1410: usize = 101;
pub const P_IT0: usize = 102;
pub const P_IK0: usize = 123;
pub const P_BWL1: usize = 144;
pub const P_CHK1: usize = 147;
pub const P_IP0: usize = 150;
pub const P_CC35: usize = 156;
pub const PANE_COUNT: usize = 160;

pub const TAG_TABLE: [&str; PANE_COUNT] = [
    "ft15", "ft16", "ft17", "ft00", "ft01", "ft02", "ft03", "ft04", "ft05", "ft06", "ft07", "ft08",
    "ft09", "ft10", "ft11", "ft12", "ft13", "ft14", "fd00", "fd01", "fd02", "fd03", "fd04", "fd05",
    "fd06", "fd07", "fd08", "fd09", "fd10", "fd11", "fd12", "fd13", "fd14", "str0", "st00", "nt00",
    "nk00", "no11", "wd", "nm00", "nm01", "itnm", "itnk", "msk0", "tl00", "tk00", "car1", "car2",
    "car3", "car4", "2401", "2501", "2801", "2601", "2701", "2901", "3001", "3101", "2410", "2510",
    "2810", "2610", "2710", "2910", "3010", "3110", "it21", "it22", "it25", "it23", "it24", "it26",
    "it27", "it28", "it29", "ik21", "ik22", "ik25", "ik23", "ik24", "ik26", "ik27", "ik28", "ik29",
    "sb00", "sb01", "sb02", "sb03", "sb04", "sb05", "sb06", "sb07", "sb08", "blak", "wdsv", "wk01",
    "wk00", "0301", "1301", "1310", "1401", "1410", "it00", "it01", "it02", "it03", "it04", "it05",
    "it06", "it07", "it08", "it09", "it10", "it11", "it12", "it13", "it14", "it15", "it16", "it17",
    "it18", "it19", "it20", "ik00", "ik01", "ik02", "ik03", "ik04", "ik05", "ik06", "ik07", "ik08",
    "ik09", "ik10", "ik11", "ik12", "ik13", "ik14", "ik15", "ik16", "ik17", "ik18", "ik19", "ik20",
    "bwl1", "bwl2", "bwl3", "chk1", "chk2", "chk3", "ip01", "ip03", "ip05", "ip07", "ip09", "ip11",
    "cc35", "cc22", "cc11", "cc00",
];

pub const FT_TAGS: [&str; 15] = [
    "ft00", "ft01", "ft02", "ft03", "ft04", "ft05", "ft06", "ft07", "ft08", "ft09", "ft10", "ft11",
    "ft12", "ft13", "ft14",
];

pub const FD_TAGS: [&str; 15] = [
    "fd00", "fd01", "fd02", "fd03", "fd04", "fd05", "fd06", "fd07", "fd08", "fd09", "fd10", "fd11",
    "fd12", "fd13", "fd14",
];

pub const NO_TAGS: [&str; 6] = ["no23", "no13", "no22", "no12", "no21", "no11"];

pub const CAR_TAGS: [&str; 4] = ["car1", "car2", "car3", "car4"];

pub const SIN01_TAGS: [&str; 8] = [
    "2401", "2501", "2801", "2601", "2701", "2901", "3001", "3101",
];

pub const SIN10_TAGS: [&str; 8] = [
    "2410", "2510", "2810", "2610", "2710", "2910", "3010", "3110",
];

pub const SIT_TAGS: [&str; 9] = [
    "it21", "it22", "it25", "it23", "it24", "it26", "it27", "it28", "it29",
];

pub const SIK_TAGS: [&str; 9] = [
    "ik21", "ik22", "ik25", "ik23", "ik24", "ik26", "ik27", "ik28", "ik29",
];

pub const SB_TAGS: [&str; SUB_BUFFER_COUNT] = [
    "sb00", "sb01", "sb02", "sb03", "sb04", "sb05", "sb06", "sb07", "sb08",
];

pub const IT_TAGS: [&str; ITEM_BUFFER_COUNT] = [
    "it00", "it01", "it02", "it03", "it04", "it05", "it06", "it07", "it08", "it09", "it10", "it11",
    "it12", "it13", "it14", "it15", "it16", "it17", "it18", "it19", "it20",
];

pub const IK_TAGS: [&str; ITEM_BUFFER_COUNT] = [
    "ik00", "ik01", "ik02", "ik03", "ik04", "ik05", "ik06", "ik07", "ik08", "ik09", "ik10", "ik11",
    "ik12", "ik13", "ik14", "ik15", "ik16", "ik17", "ik18", "ik19", "ik20",
];

pub const IP_TAGS: [&str; 6] = ["ip01", "ip03", "ip05", "ip07", "ip09", "ip11"];

pub const NOTE_SLOTS: usize = 15;
pub const CURSOR_PARTS: usize = 4;
pub const PLACE_GROUPS: usize = 6;
pub const MAIN_PANES: [usize; 8] = [P_PANE_BG, P_WK01, P_WK00, P_0301, P_1301, P_1310, P_1401, P_1410];
pub const FADE_EXEMPT: [usize; 12] = [
    P_STR0,
    P_ST00,
    P_NT00,
    P_NK00,
    P_NO11,
    P_WD,
    P_NM00,
    P_NM01,
    P_ITNM,
    P_ITNK,
    P_TL00,
    P_TK00,
];

pub const TEXT_PANES: [usize; 13] = [
    P_STR0,
    P_ST00,
    P_NT00,
    P_NK00,
    P_NO11,
    P_WD,
    P_NM00,
    P_NM01,
    P_ITNM,
    P_ITNK,
    P_MSK0,
    P_TL00,
    P_TK00,
];

pub const CLOTH_TYPE_DEFAULT: u8 = 0;
pub const CLOTH_TYPE_FILE_SELECT: u8 = 1;
pub const CLOTH_TYPE_CLOTH_ONLY: u8 = 2;
pub const CLOTH_TYPE_COUNT: usize = 3;
pub const CLOTH_INNER: usize = 11;
pub const CLOTH_VERTS: usize = 441;
pub const CLOTH_SHADOW_Z: f32 = -3300.0;
pub const CLOTH_FADE_STEP: u8 = 25;

pub const STATE_NO_MENU: u8 = 0;
pub const STATE_ITEM_OPEN: u8 = 1;
pub const STATE_ITEM_OPEN_FROM_COLLECT_LEFT: u8 = 2;
pub const STATE_ITEM_OPEN_FROM_COLLECT_RIGHT: u8 = 3;
pub const STATE_ITEM_MOVE: u8 = 4;
pub const STATE_ITEM_CLOSE: u8 = 5;
pub const STATE_ITEM_TO_COLLECT_LEFT: u8 = 6;
pub const STATE_ITEM_TO_COLLECT_RIGHT: u8 = 7;
pub const STATE_COLLECT_OPEN: u8 = 8;
pub const STATE_COLLECT_OPEN_LEFT: u8 = 9;
pub const STATE_COLLECT_OPEN_RIGHT: u8 = 10;
pub const STATE_COLLECT_OPEN_TACT: u8 = 11;
pub const STATE_COLLECT_OPEN_ALT: u8 = 12;
pub const STATE_COLLECT_MOVE: u8 = 13;
pub const STATE_COLLECT_MOVE_TACT: u8 = 14;
pub const STATE_COLLECT_MOVE_ALT: u8 = 15;
pub const STATE_COLLECT_CLOSE: u8 = 16;
pub const STATE_COLLECT_TO_ITEM_LEFT: u8 = 17;
pub const STATE_COLLECT_TO_ITEM_RIGHT: u8 = 18;
pub const STATE_COLLECT_CLOSE_TACT: u8 = 19;
pub const STATE_COLLECT_CLOSE_ALT: u8 = 20;
pub const STATE_FMAP_OPEN: u8 = 21;
pub const STATE_FMAP_OPEN_WARP_MODE: u8 = 22;
pub const STATE_FMAP_OPEN_WALLPAPER: u8 = 23;
pub const STATE_FMAP_OPEN_FISHMAN_MODE: u8 = 24;
pub const STATE_FMAP_MOVE: u8 = 25;
pub const STATE_FMAP_MOVE_WARP_MODE: u8 = 26;
pub const STATE_FMAP_MOVE_WALLPAPER: u8 = 27;
pub const STATE_FMAP_MOVE_FISHMAN_MODE: u8 = 28;
pub const STATE_FMAP_CLOSE: u8 = 29;
pub const STATE_DMAP_OPEN: u8 = 30;
pub const STATE_DMAP_MOVE: u8 = 31;
pub const STATE_DMAP_CLOSE: u8 = 32;
pub const STATE_NAME_OPEN: u8 = 33;
pub const STATE_NAME_MOVE: u8 = 34;
pub const STATE_NAME_CLOSE: u8 = 35;
pub const STATE_SAVE_OPEN: u8 = 36;
pub const STATE_SAVE_MOVE: u8 = 37;
pub const STATE_SAVE_CLOSE: u8 = 38;
pub const STATE_COUNT: usize = 39;

pub const MENU_STATUS_COLLECT: u8 = 1;
pub const MENU_STATUS_ITEM: u8 = 2;
pub const MENU_STATUS_MAP: u8 = 3;
pub const MENU_STATUS_NAME: u8 = 4;
pub const MENU_STATUS_SAVE: u8 = 5;

pub const ITEM_TUNING_ARRAYS: usize = 5;
pub const ITEM_TUNING_SCALE: f32 = 0.8;
pub const ITEM_TUNING_MASK_ALPHA: u8 = 130;
pub const ITEM_TUNING_CLOTH: [u8; 4] = [0xE3, 0xFF, 0xB3, 0xDC];
pub const ITEM_TUNING_SHADOW: [u8; 4] = [0x00, 0x00, 0x00, 0x00];
pub const ITEM_TUNING_BLACK: [u8; 4] = [0x00, 0x00, 0x00, 0xFF];
pub const ITEM_TUNING_WHITE: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
pub const ITEM_TUNING_CURSOR_HALF: i16 = 10;
pub const ITEM_TUNING_CURSOR_STEP: i16 = 7;
pub const ITEM_TUNING_CURSOR_STEP_LARGE: i16 = 14;
pub const ITEM_TUNING_NOTE_OPEN_LEN: i16 = 5;
pub const ITEM_TUNING_NOTE_CLOSE_LEN: i16 = 12;
pub const ITEM_TUNING_NOTE_CLOSE_FRAMES: i16 = 10;
pub const ITEM_TUNING_NOTE_OPEN_ROT: i16 = 200;
pub const ITEM_TUNING_NOTE_ROTATE_FROM: i16 = -3;
pub const ITEM_TUNING_NOTE_CLOSE_ANGLE: i16 = -120;
pub const ITEM_TUNING_SLIDE_X: i16 = -100;
pub const ITEM_TUNING_TITLE_Y: i16 = -100;
pub const ITEM_TUNING_NOTE_ANGLE: i16 = -45;
pub const ITEM_TUNING_NAME_Y: i16 = 50;
pub const ITEM_TUNING_UNUSED_2E: i16 = 30;
pub const ITEM_TUNING_SCREEN_X: i16 = 320;
pub const ITEM_TUNING_SCREEN_Y: i16 = 240;
pub const ITEM_TUNING_UNUSED_46: f32 = 0.0;
pub const ITEM_TUNING_UNUSED_48: f32 = 20.0;
pub const ITEM_TUNING_FADE_FRAMES: i16 = 10;
pub const ITEM_TUNING_ARROW_PERIOD: i16 = 120;
pub const ITEM_TUNING_ARROW_HALF: i16 = 60;
pub const ITEM_TUNING_ARROW_MIN_ALPHA: f32 = 50.0;
pub const ITEM_TUNING_ARROW_SCALE: f32 = 0.39999998;
pub const ITEM_TUNING_CORNER_PERIOD: i16 = 240;
pub const ITEM_TUNING_CORNER_HALF: i16 = 60;
pub const ITEM_TUNING_SHEEN_PERIOD: i16 = 40;
pub const ITEM_TUNING_SHEEN_HALF: i16 = 20;
pub const ITEM_TUNING_SHEEN_BASE: f32 = 128.0;
pub const ITEM_TUNING_SHEEN_RATE: f32 = -42.0;
pub const ITEM_TUNING_NAME_PERIOD: i16 = 130;
pub const ITEM_TUNING_NAME_RESET: i16 = 11;
pub const ITEM_TUNING_NAME_HOLD: i16 = 60;
pub const ITEM_TUNING_NAME_OUT: i16 = 120;
pub const ITEM_TUNING_NAME_MID: i16 = 70;
pub const ITEM_TUNING_NAME_FADE: i32 = 10;

pub const MENU_TUNING_ITEM_SCALE: f32 = 1.4;
pub const MENU_TUNING_SUB_ROWS: i16 = 8;
pub const MENU_TUNING_OPEN_WAIT: i16 = 3;
pub const MENU_TUNING_PAGE_FRAMES: i16 = 7;

pub const WINDOW_BIT_ITEMS: usize = 21;
pub const WINDOW_BIT_PAIRS: usize = 8;
pub const WINDOW_BIT_TENS: usize = 6;
pub const WINDOW_BIT_TRIPLES: usize = 3;
pub const WINDOW_BIT_EIGHTHS: usize = 8;
pub const WINDOW_RUBY_SOURCE: u8 = 2;
pub const WINDOW_EVENT_WAIT: i8 = 5;
pub const WINDOW_BUTTON_L: u8 = 1;
pub const WINDOW_BUTTON_R: u8 = 2;
pub const WINDOW_DRAW_OFF: u8 = 0;
pub const WINDOW_DRAW_ON: u8 = 1;
pub const WINDOW_DRAW_CAPTURE: u8 = 2;
pub const WINDOW_COLLECT_MODE_CLOSED: u8 = 5;
pub const WINDOW_ICON_MODE_DEFAULT: u8 = 3;
pub const WINDOW_CLOSE_TIMER: i16 = 10;
pub const SAVE_OPEN_FRAMES: i16 = 10;
pub const SAVE_MOVE_FRAMES: i16 = 60;
pub const SAVE_CLOSE_FRAMES: i16 = 10;

pub const CLOTH_SCALE_DEFAULT: f32 = 1.78;
pub const CLOTH_SCALE_FILE_SELECT: [f32; 3] = [1.27, 0.93, 1.0];
pub const CLOTH_SCALE_CLOTH_ONLY: [f32; 3] = [1.7, 1.7, 1.0];
pub const CLOTH_ROT_CLOTH_ONLY: [i16; 3] = [0, 0, 0x4000];
pub const CLOTH_POS_DEFAULT: [f32; 3] = [360.0, 40.0, -2400.0];
pub const CLOTH_POS_FILE_SELECT: [f32; 3] = [350.0, -15.0, -2400.0];
pub const CLOTH_COLOR_DEFAULT: [u8; 4] = [0x78, 0xA5, 0x37, 0xDC];
pub const CLOTH_SHADOW_DEFAULT: [u8; 4] = [0x23, 0x5F, 0x19, 0x20];
pub const CLOTH_COLOR_BEIGE: [u8; 4] = [0xBE, 0xB4, 0x64, 0xAA];
pub const CLOTH_SHADOW_BROWN: [u8; 4] = [0x8C, 0x50, 0x14, 0x20];
pub const CLOTH_COLOR_BEIGE_FULL: [u8; 4] = [0xBE, 0xB4, 0x64, 0xDC];
pub const CLOTH_START_ALPHA: i16 = 0;
pub const CLOTH_FADE_IN_LENGTH: i16 = 7;
pub const CLOTH_WAVE_PROGRESS_STEP: i16 = -1000;
pub const CLOTH_WAVE_FREQ_X: i16 = 9000;
pub const CLOTH_WAVE_FREQ_Y: i16 = 0x800;
pub const CLOTH_WAVE_AMP_X: f32 = 5.0;
pub const CLOTH_WAVE_AMP_Y: f32 = 5.0;
pub const CLOTH_WAVE_AMP_Z: f32 = 250.0;
pub const CLOTH_LIGHT_DISTANCE: f32 = 1000.0;
pub const CLOTH_LIGHT_PITCH: i16 = 0;
pub const CLOTH_LIGHT_YAW: i16 = 0;
pub const CLOTH_WAVE_PRE_STEPS: i16 = 2;
pub const CLOTH_MAX_STEP: f32 = 290.0;
pub const CLOTH_RANDOM_Z_DEG: f32 = 45.0;
pub const CLOTH_RANDOM_Z_BASE: f32 = 90.0;
pub const CLOTH_ONLY_RANDOM_DEG: f32 = 20.0;
pub const CLOTH_WAVE_MOVE_SCALE: f32 = 0.5;
pub const CLOTH_NORM_YAW_STEP: i32 = -800;
pub const CLOTH_NORM_YAW_AMP: f32 = 900.0;
pub const CLOTH_SIN_AMP: f32 = 954.9299;
pub const CLOTH_SIN_AMP_Z: f32 = 716.1974;
pub const CLOTH_OFF_AMP: f32 = -320.0;
pub const CLOTH_OFF_AMP_Z: f32 = -270.0;
pub const CLOTH_OFF_PHASE: f32 = -1000.0;
pub const CLOTH_ROW_ANGLE: f32 = 3500.0;
pub const CLOTH_COL_ANGLE: f32 = 3276.8;
pub const CLOTH_BASE_X: f32 = -1500.0;
pub const CLOTH_BASE_Y: f32 = -1500.0;
pub const CLOTH_BASE_Z: f32 = -3400.0;
pub const CLOTH_INIT_X_STEP: f32 = 10.0;
pub const CLOTH_INIT_Z_STEP: f32 = -5.0;
pub const CLOTH_ROW_STEP_POS: f32 = 300.0;
pub const CLOTH_WAVE_X_STEP: f32 = 305.0;
pub const CLOTH_NORMAL_FALLBACK: [f32; 3] = [0.0, 0.0, 1.0];
pub const CLOTH_EPSILON: f32 = 8e-11;
pub const CLOTH_MODEL_DX: f32 = 350.0;
pub const CLOTH_MODEL_DZ: f32 = 3800.0;
pub const CLOTH_SHADOW_DX: f32 = 275.0;
pub const CLOTH_SHADOW_DY: f32 = 75.0;

pub const STICK_MAX_VALUE: f32 = 0.9;
pub const STICK_MIN_VALUE: f32 = 0.5;
pub const STICK_HALF_DEAD: i16 = 0x800;

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

pub fn min_max_limit(value: i16, low: i16, high: i16) -> i16 {
    value.max(low).min(high)
}

pub fn max_limit(value: i32, high: i32) -> i32 {
    value.min(high)
}

pub fn deg2s(deg: f32) -> i16 {
    (deg * 32768.0 / 180.0) as i16
}

pub fn ssin(angle: i16) -> f32 {
    (angle as f32 * (PI / 2.0 / 32768.0)).sin()
}

pub fn scos(angle: i16) -> f32 {
    (angle as f32 * (PI / 2.0 / 32768.0)).cos()
}

pub fn pack_color(c: [u8; 4]) -> u32 {
    ((c[0] as u32) << 24) | ((c[1] as u32) << 16) | ((c[2] as u32) << 8) | c[3] as u32
}

pub fn unpack_color(c: u32) -> [u8; 4] {
    [
        (c >> 24) as u8,
        ((c >> 16) & 0xFF) as u8,
        ((c >> 8) & 0xFF) as u8,
        (c & 0xFF) as u8,
    ]
}

pub fn item_num(item: u8) -> u8 {
    ITEM_ICON_NO
        .iter()
        .position(|entry| *entry == item)
        .unwrap_or(0) as u8
}

pub fn item_texture(item: u8) -> &'static str {
    ITEM_TEXTURE
        .get(usize::from(item))
        .copied()
        .unwrap_or("")
}

pub fn item_message(item: u8) -> i16 {
    ITEM_MESSAGE
        .get(usize::from(item))
        .copied()
        .unwrap_or(0)
}

pub fn is_recollect_disabled(item: u8, boss_rematch: bool) -> bool {
    boss_rematch
        && (item == ITEM_WATER_BOTTLE
            || item == ITEM_FIREFLY_BOTTLE
            || item == ITEM_FOREST_WATER)
}

pub fn boss_rematch(stage_boss: bool, start_stage: &str) -> bool {
    stage_boss && RECOLLECT_STAGES.contains(&start_stage)
}

fn rand_fx(max: f32, salt: u32) -> f32 {
    let mut seed = salt.wrapping_mul(1664525).wrapping_add(1013904223);
    seed ^= seed >> 15;
    seed = seed.wrapping_mul(2246822519);
    seed ^= seed >> 13;
    seed = seed.wrapping_mul(3266489917);
    seed ^= seed >> 16;
    max * 2.0 * (seed as f32 / u32::MAX as f32) - max
}

fn near_zero(v: [f32; 3]) -> bool {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2] < CLOTH_EPSILON
}

fn norm_zp(v: [f32; 3]) -> [f32; 3] {
    if near_zero(v) {
        return [0.0, 0.0, 0.0];
    }
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / len, v[1] / len, v[2] / len]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn rot_y(v: [f32; 3], angle: f32) -> [f32; 3] {
    let (s, c) = angle.sin_cos();
    [v[0] * c + v[2] * s, v[1], -v[0] * s + v[2] * c]
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct F2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContentsColor {
    pub tl: [u8; 4],
    pub tr: [u8; 4],
    pub bl: [u8; 4],
    pub br: [u8; 4],
}

impl Default for ContentsColor {
    fn default() -> Self {
        Self {
            tl: [0, 0, 0, 0],
            tr: [0, 0, 0, 0],
            bl: [0, 0, 0, 0],
            br: [0, 0, 0, 0],
        }
    }
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
    }

    pub fn blend_draw(&mut self, texture: &'static str) {
        self.visible = true;
        self.remove();
        self.append(texture);
    }

    pub fn set_blend(&mut self, blend: [f32; 4]) {
        self.blend = blend;
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

    pub fn rotate(&mut self, x: f32, y: f32, angle: f32) {
        self.rotate_x = x;
        self.rotate_y = y;
        self.rotation = angle as i16 as u16;
    }

    pub fn rotate_deg(&mut self, x: f32, y: f32, deg: f32) {
        self.rotate(x, y, f32::from(deg2s(deg)));
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

    pub fn pane_scale_x(&mut self, scale: f32) {
        self.size.x = self.size_orig.x * scale;
        self.cpos_move();
    }

    pub fn pane_scale_y(&mut self, scale: f32) {
        self.size.y = self.size_orig.y * scale;
        self.cpos_move();
    }

    pub fn pane_scale_xy(&mut self, scale: f32) {
        self.size.x = self.size_orig.x * scale;
        self.size.y = self.size_orig.y * scale;
        self.cpos_move();
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
#[derive(Clone, Debug)]
pub struct ItemPanes {
    pub panes: [Pane; PANE_COUNT],
    pub order: Vec<usize>,
}

impl Default for ItemPanes {
    fn default() -> Self {
        Self {
            panes: std::array::from_fn(|i| Pane::new(TAG_TABLE[i])),
            order: (0..PANE_COUNT).collect(),
        }
    }
}

impl ItemPanes {
    pub fn bind(&mut self, screen: &Screen) {
        for pane in &mut self.panes {
            if let Some(layout) = screen.find(pane.tag) {
                pane.parts_set(layout);
            }
        }
    }

    pub fn pane(&self, index: usize) -> &Pane {
        &self.panes[index]
    }

    pub fn pane_mut(&mut self, index: usize) -> &mut Pane {
        &mut self.panes[index]
    }

    pub fn note_icon(&self, i: usize) -> &Pane {
        &self.panes[P_FT0 + i]
    }

    pub fn note_icon_mut(&mut self, i: usize) -> &mut Pane {
        &mut self.panes[P_FT0 + i]
    }

    pub fn note_shadow(&self, i: usize) -> &Pane {
        &self.panes[P_FD0 + i]
    }

    pub fn note_shadow_mut(&mut self, i: usize) -> &mut Pane {
        &mut self.panes[P_FD0 + i]
    }

    pub fn cursor(&self, i: usize) -> &Pane {
        &self.panes[P_CAR0 + i]
    }

    pub fn cursor_mut(&mut self, i: usize) -> &mut Pane {
        &mut self.panes[P_CAR0 + i]
    }

    pub fn sub_ones(&self, i: usize) -> &Pane {
        &self.panes[P_SIN01 + i]
    }

    pub fn sub_ones_mut(&mut self, i: usize) -> &mut Pane {
        &mut self.panes[P_SIN01 + i]
    }

    pub fn sub_tens(&self, i: usize) -> &Pane {
        &self.panes[P_SIN10 + i]
    }

    pub fn sub_tens_mut(&mut self, i: usize) -> &mut Pane {
        &mut self.panes[P_SIN10 + i]
    }

    pub fn sub_item_index(&self, i: usize) -> usize {
        if i < SUB_ITEM_COUNT {
            P_SIT + i
        } else {
            P_IT29
        }
    }

    pub fn sub_shadow_index(&self, i: usize) -> usize {
        if i < SUB_ITEM_COUNT {
            P_SIK + i
        } else {
            P_IK29
        }
    }

    pub fn sub_item(&self, i: usize) -> &Pane {
        &self.panes[self.sub_item_index(i)]
    }

    pub fn sub_item_mut(&mut self, i: usize) -> &mut Pane {
        let index = self.sub_item_index(i);
        &mut self.panes[index]
    }

    pub fn sub_shadow(&self, i: usize) -> &Pane {
        &self.panes[self.sub_shadow_index(i)]
    }

    pub fn sub_shadow_mut(&mut self, i: usize) -> &mut Pane {
        let index = self.sub_shadow_index(i);
        &mut self.panes[index]
    }

    pub fn sub_frame(&self, i: usize) -> &Pane {
        &self.panes[P_SB0 + i]
    }

    pub fn sub_frame_mut(&mut self, i: usize) -> &mut Pane {
        &mut self.panes[P_SB0 + i]
    }

    pub fn item_icon(&self, i: usize) -> &Pane {
        &self.panes[P_IT0 + i]
    }

    pub fn item_icon_mut(&mut self, i: usize) -> &mut Pane {
        &mut self.panes[P_IT0 + i]
    }

    pub fn item_shadow(&self, i: usize) -> &Pane {
        &self.panes[P_IK0 + i]
    }

    pub fn item_shadow_mut(&mut self, i: usize) -> &mut Pane {
        &mut self.panes[P_IK0 + i]
    }

    pub fn arrow_tint(&self, i: usize) -> &Pane {
        &self.panes[P_BWL1 + i]
    }

    pub fn arrow_tint_mut(&mut self, i: usize) -> &mut Pane {
        &mut self.panes[P_BWL1 + i]
    }

    pub fn button_check(&self, i: usize) -> &Pane {
        &self.panes[P_CHK1 + i]
    }

    pub fn button_check_mut(&mut self, i: usize) -> &mut Pane {
        &mut self.panes[P_CHK1 + i]
    }

    pub fn place(&self, i: usize) -> &Pane {
        &self.panes[P_IP0 + i]
    }

    pub fn place_mut(&mut self, i: usize) -> &mut Pane {
        &mut self.panes[P_IP0 + i]
    }

    pub fn corner(&self, i: usize) -> &Pane {
        &self.panes[P_CC35 + i]
    }

    pub fn corner_mut(&mut self, i: usize) -> &mut Pane {
        &mut self.panes[P_CC35 + i]
    }

    pub fn insert_child(&mut self, after: usize, child: usize) {
        let pos = self
            .order
            .iter()
            .position(|entry| *entry == after)
            .map(|at| at + 1)
            .unwrap_or(self.order.len());
        self.order.retain(|entry| *entry != child);
        self.order.insert(pos.min(self.order.len()), child);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct EmitterState {
    pub spawned: bool,
    pub x: f32,
    pub y: f32,
    pub alpha: f32,
    pub resource: u32,
    pub playing: bool,
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct ItemMoveBridge {
    pub flag: u8,
    pub button: u8,
    pub slot: u8,
    pub from: F2,
    pub from_center: F2,
    pub from_size: F2,
    pub from_size_orig: F2,
    pub timer: i16,
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct ItemRecord {
    pub picture_num: u8,
    pub arrow_num: u8,
    pub arrow_max: u8,
    pub bomb_num: u8,
    pub bomb_max: u8,
    pub beast: [u8; SUB_ITEM_COUNT],
    pub beast_num: [u8; SUB_ITEM_COUNT],
    pub bait: [u8; SUB_ITEM_COUNT],
    pub bait_num: [u8; SUB_ITEM_COUNT],
    pub reserve: [u8; SUB_ITEM_COUNT],
    pub select_item: [u8; SELECT_SLOTS],
    pub item_timer: u8,
}

impl ItemRecord {
    pub fn select(&self, button: u8) -> u8 {
        self.select_item
            .get(usize::from(button))
            .copied()
            .unwrap_or(ITEM_NONE)
    }

    pub fn set_select(&mut self, button: u8, slot: u8) {
        if let Some(entry) = self.select_item.get_mut(usize::from(button)) {
            *entry = slot;
        }
    }
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct PadState {
    pub a: bool,
    pub b: bool,
    pub z: bool,
    pub x: bool,
    pub y: bool,
    pub start: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub lock_l: bool,
    pub lock_r: bool,
    pub stick_value: f32,
    pub stick_angle: i16,
}

#[derive(Resource, Clone, Debug)]
pub struct MenuEnv {
    pub event_running: bool,
    pub heap_lock_flag: bool,
    pub heap_lock_level: u8,
    pub overlap_request: bool,
    pub enable_next_stage: bool,
    pub message_status: u8,
    pub scope_message_status: u8,
    pub fmap_open_check: u8,
    pub password_open_check: u8,
    pub menu_collect: bool,
    pub operate_wind: u8,
    pub up_button: u8,
    pub mini_game_type: u8,
    pub stage_boss: bool,
    pub start_stage: String,
    pub button_info: [u8; 2],
    pub opt_ruby: bool,
    pub camera_telescope: bool,
    pub camera_pictobox: bool,
    pub player_status_800000: bool,
    pub picture_status: u8,
    pub scope_type: u8,
    pub tact_normal_wait: bool,
    pub link_actor: bool,
    pub demo_mode: u8,
    pub event_bit_0908: bool,
    pub event_start_check: Option<&'static str>,
    pub demo_frame_no_msg: u32,
}

impl Default for MenuEnv {
    fn default() -> Self {
        Self {
            event_running: false,
            heap_lock_flag: false,
            heap_lock_level: 0,
            overlap_request: false,
            enable_next_stage: false,
            message_status: 0,
            scope_message_status: 0,
            fmap_open_check: 0,
            password_open_check: 0,
            menu_collect: false,
            operate_wind: 0,
            up_button: 0,
            mini_game_type: 0,
            stage_boss: false,
            start_stage: String::new(),
            button_info: [0, 0],
            opt_ruby: false,
            camera_telescope: false,
            camera_pictobox: false,
            player_status_800000: false,
            picture_status: 0,
            scope_type: 0,
            tact_normal_wait: false,
            link_actor: true,
            demo_mode: 0,
            event_bit_0908: false,
            event_start_check: None,
            demo_frame_no_msg: 0,
        }
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct MenuOutput {
    pub sounds: Vec<u32>,
    pub do_status_force: Option<u8>,
    pub do_status: Option<u8>,
    pub a_status_force: Option<u8>,
    pub push_menu_button: Option<u8>,
    pub select_item_writes: Vec<(u8, u8)>,
    pub item_timer_dec: u32,
    pub record_timer_dec: bool,
    pub message_push: Option<u32>,
    pub message_status: u8,
    pub stage_name_on: Option<u8>,
    pub capture_draw: u8,
    pub picture_status_set: Option<u8>,
    pub fwater_timer_start: bool,
}

impl MenuOutput {
    pub fn reset(&mut self) {
        self.sounds.clear();
        self.do_status_force = None;
        self.do_status = None;
        self.a_status_force = None;
        self.push_menu_button = None;
        self.select_item_writes.clear();
        self.item_timer_dec = 0;
        self.record_timer_dec = false;
        self.message_push = None;
        self.message_status = 0;
        self.stage_name_on = None;
        self.capture_draw = WINDOW_DRAW_OFF;
        self.picture_status_set = None;
        self.fwater_timer_start = false;
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct ItemTuning {
    pub scale: f32,
    pub note_scale_frames: i16,
    pub arr_0e: [i16; ITEM_TUNING_ARRAYS],
    pub arr_1a: [i16; ITEM_TUNING_ARRAYS],
    pub enable: i16,
    pub slide_x: i16,
    pub title_y: i16,
    pub note_angle: i16,
    pub name_y: i16,
    pub unused_2e: i16,
    pub note_rotate_from: i16,
    pub cursor_half: i16,
    pub cursor_step: i16,
    pub cursor_step_large: i16,
    pub note_open_len: i16,
    pub note_close_len: i16,
    pub note_close_frames: i16,
    pub note_open_rot: i16,
    pub screen_x: i16,
    pub screen_y: i16,
    pub note_close_angle: i16,
    pub unused_46: f32,
    pub unused_48: f32,
    pub cloth_color: [u8; 4],
    pub shadow_color: [u8; 4],
    pub black: [u8; 4],
    pub white: [u8; 4],
    pub mask_alpha: u8,
}

impl Default for ItemTuning {
    fn default() -> Self {
        Self {
            scale: ITEM_TUNING_SCALE,
            note_scale_frames: 15,
            arr_0e: [0; ITEM_TUNING_ARRAYS],
            arr_1a: [10; ITEM_TUNING_ARRAYS],
            enable: 1,
            slide_x: ITEM_TUNING_SLIDE_X,
            title_y: ITEM_TUNING_TITLE_Y,
            note_angle: ITEM_TUNING_NOTE_ANGLE,
            name_y: ITEM_TUNING_NAME_Y,
            unused_2e: ITEM_TUNING_UNUSED_2E,
            note_rotate_from: ITEM_TUNING_NOTE_ROTATE_FROM,
            cursor_half: ITEM_TUNING_CURSOR_HALF,
            cursor_step: ITEM_TUNING_CURSOR_STEP,
            cursor_step_large: ITEM_TUNING_CURSOR_STEP_LARGE,
            note_open_len: ITEM_TUNING_NOTE_OPEN_LEN,
            note_close_len: ITEM_TUNING_NOTE_CLOSE_LEN,
            note_close_frames: ITEM_TUNING_NOTE_CLOSE_FRAMES,
            note_open_rot: ITEM_TUNING_NOTE_OPEN_ROT,
            screen_x: ITEM_TUNING_SCREEN_X,
            screen_y: ITEM_TUNING_SCREEN_Y,
            note_close_angle: ITEM_TUNING_NOTE_CLOSE_ANGLE,
            unused_46: ITEM_TUNING_UNUSED_46,
            unused_48: ITEM_TUNING_UNUSED_48,
            cloth_color: ITEM_TUNING_CLOTH,
            shadow_color: ITEM_TUNING_SHADOW,
            black: ITEM_TUNING_BLACK,
            white: ITEM_TUNING_WHITE,
            mask_alpha: ITEM_TUNING_MASK_ALPHA,
        }
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct MenuTuning {
    pub item_scale: f32,
    pub sub_rows: i16,
    pub open_wait: i16,
    pub page_frames: i16,
}

impl Default for MenuTuning {
    fn default() -> Self {
        Self {
            item_scale: MENU_TUNING_ITEM_SCALE,
            sub_rows: MENU_TUNING_SUB_ROWS,
            open_wait: MENU_TUNING_OPEN_WAIT,
            page_frames: MENU_TUNING_PAGE_FRAMES,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ClothTuning {
    pub scale: [f32; 3],
    pub pos: [f32; 3],
    pub rot: [i16; 3],
    pub cloth_color: [u8; 4],
    pub shadow_color: [u8; 4],
    pub start_alpha: i16,
    pub fade_in_length: i16,
    pub wave_progress_step: i16,
    pub wave_freq_x: i16,
    pub wave_freq_y: i16,
    pub wave_amp_x: f32,
    pub wave_amp_y: f32,
    pub wave_amp_z: f32,
    pub light_pitch: i16,
    pub light_yaw: i16,
    pub light_distance: f32,
    pub wave_pre_steps: i16,
    pub max_step: f32,
}

pub const CLOTH_TUNING: [ClothTuning; CLOTH_TYPE_COUNT] = [
    ClothTuning {
        scale: [CLOTH_SCALE_DEFAULT, CLOTH_SCALE_DEFAULT, 1.0],
        pos: CLOTH_POS_DEFAULT,
        rot: [0, 0, 0],
        cloth_color: CLOTH_COLOR_DEFAULT,
        shadow_color: CLOTH_SHADOW_DEFAULT,
        start_alpha: CLOTH_START_ALPHA,
        fade_in_length: CLOTH_FADE_IN_LENGTH,
        wave_progress_step: CLOTH_WAVE_PROGRESS_STEP,
        wave_freq_x: CLOTH_WAVE_FREQ_X,
        wave_freq_y: CLOTH_WAVE_FREQ_Y,
        wave_amp_x: CLOTH_WAVE_AMP_X,
        wave_amp_y: CLOTH_WAVE_AMP_Y,
        wave_amp_z: CLOTH_WAVE_AMP_Z,
        light_pitch: CLOTH_LIGHT_PITCH,
        light_yaw: CLOTH_LIGHT_YAW,
        light_distance: CLOTH_LIGHT_DISTANCE,
        wave_pre_steps: CLOTH_WAVE_PRE_STEPS,
        max_step: CLOTH_MAX_STEP,
    },
    ClothTuning {
        scale: CLOTH_SCALE_FILE_SELECT,
        pos: CLOTH_POS_FILE_SELECT,
        rot: [0, 0, 0],
        cloth_color: CLOTH_COLOR_BEIGE,
        shadow_color: CLOTH_SHADOW_BROWN,
        start_alpha: CLOTH_START_ALPHA,
        fade_in_length: CLOTH_FADE_IN_LENGTH,
        wave_progress_step: CLOTH_WAVE_PROGRESS_STEP,
        wave_freq_x: CLOTH_WAVE_FREQ_X,
        wave_freq_y: CLOTH_WAVE_FREQ_Y,
        wave_amp_x: CLOTH_WAVE_AMP_X,
        wave_amp_y: CLOTH_WAVE_AMP_Y,
        wave_amp_z: CLOTH_WAVE_AMP_Z,
        light_pitch: CLOTH_LIGHT_PITCH,
        light_yaw: CLOTH_LIGHT_YAW,
        light_distance: CLOTH_LIGHT_DISTANCE,
        wave_pre_steps: CLOTH_WAVE_PRE_STEPS,
        max_step: CLOTH_MAX_STEP,
    },
    ClothTuning {
        scale: CLOTH_SCALE_CLOTH_ONLY,
        pos: CLOTH_POS_DEFAULT,
        rot: CLOTH_ROT_CLOTH_ONLY,
        cloth_color: CLOTH_COLOR_BEIGE_FULL,
        shadow_color: CLOTH_SHADOW_BROWN,
        start_alpha: CLOTH_START_ALPHA,
        fade_in_length: CLOTH_FADE_IN_LENGTH,
        wave_progress_step: CLOTH_WAVE_PROGRESS_STEP,
        wave_freq_x: CLOTH_WAVE_FREQ_X,
        wave_freq_y: CLOTH_WAVE_FREQ_Y,
        wave_amp_x: CLOTH_WAVE_AMP_X,
        wave_amp_y: CLOTH_WAVE_AMP_Y,
        wave_amp_z: CLOTH_WAVE_AMP_Z,
        light_pitch: CLOTH_LIGHT_PITCH,
        light_yaw: CLOTH_LIGHT_YAW,
        light_distance: CLOTH_LIGHT_DISTANCE,
        wave_pre_steps: CLOTH_WAVE_PRE_STEPS,
        max_step: CLOTH_MAX_STEP,
    },
];

#[derive(Clone, Copy, Debug, Default)]
pub struct ClothVertex {
    pub pos: [f32; 3],
    pub nrm: [f32; 3],
    pub back_nrm: [f32; 3],
    pub offset: [f32; 3],
    pub shadow: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct ClothMesh {
    pub vertices: [[f32; 3]; CLOTH_VERTS],
    pub normals: [[f32; 3]; CLOTH_VERTS],
    pub back_normals: [[f32; 3]; CLOTH_VERTS],
    pub shadows: [[f32; 3]; CLOTH_VERTS],
    pub scale: [f32; 3],
    pub rotation: [i16; 3],
    pub color: [u8; 4],
    pub shadow_color: [u8; 4],
    pub light: [f32; 3],
    pub model_offset: [f32; 3],
    pub shadow_offset: [f32; 3],
    pub alpha_out: u8,
    pub fade: f32,
    pub inner: usize,
    pub vertex_alpha: Option<Vec<[u8; 4]>>,
}

#[derive(Clone, Debug)]
pub struct MenuCloth {
    pub kind: u8,
    pub wave_progress: i16,
    pub fade_in_counter: i16,
    pub alpha_out: u8,
    pub current_alpha: u8,
    pub cur_arr: u8,
    pub scale: [f32; 3],
    pub rot: [i16; 3],
    pub cloth_color: [u8; 4],
    pub shadow_color: [u8; 4],
    pub init_angle_z: i16,
    pub salt: u32,
    pub verts: [ClothVertex; CLOTH_VERTS],
}

impl Default for MenuCloth {
    fn default() -> Self {
        Self::new(CLOTH_TYPE_DEFAULT)
    }
}

impl MenuCloth {
    pub fn new(kind: u8) -> Self {
        let mut cloth = Self {
            kind: kind.min(CLOTH_TYPE_COUNT as u8 - 1),
            wave_progress: 0,
            fade_in_counter: 0,
            alpha_out: 0,
            current_alpha: 0,
            cur_arr: 0,
            scale: CLOTH_SCALE_FILE_SELECT,
            rot: [0; 3],
            cloth_color: [0; 4],
            shadow_color: [0; 4],
            init_angle_z: 0,
            salt: 0x1234_5678,
            verts: [ClothVertex::default(); CLOTH_VERTS],
        };
        cloth.cloth_init();
        cloth
    }

    pub fn tuning(&self) -> &'static ClothTuning {
        &CLOTH_TUNING[usize::from(self.kind)]
    }

    pub fn set_cloth_type(&mut self, kind: u8) {
        self.kind = kind.min(CLOTH_TYPE_COUNT as u8 - 1);
    }

    pub fn set_nrm_vtx(&mut self, x: usize, y: usize) {
        let index = x + y * CLOTH_INNER;
        let pos = self.verts[index].pos;
        let mut total = [0.0f32; 3];
        if x != 0 {
            let x_diff = sub3(self.verts[x - 1 + y * CLOTH_INNER].pos, pos);
            if y != 0 {
                let y_diff = sub3(self.verts[x + (y - 1) * CLOTH_INNER].pos, pos);
                let n = norm_zp(cross(y_diff, x_diff));
                total = [total[0] + n[0], total[1] + n[1], total[2] + n[2]];
            }
            if y != CLOTH_INNER - 1 {
                let y_diff = sub3(self.verts[x + (y + 1) * CLOTH_INNER].pos, pos);
                let n = norm_zp(cross(x_diff, y_diff));
                total = [total[0] + n[0], total[1] + n[1], total[2] + n[2]];
            }
        }
        if x != CLOTH_INNER - 1 {
            let x_diff = sub3(self.verts[x + 1 + y * CLOTH_INNER].pos, pos);
            if y != 0 {
                let y_diff = sub3(self.verts[x + (y - 1) * CLOTH_INNER].pos, pos);
                let n = norm_zp(cross(y_diff, x_diff));
                total = [total[0] + n[0], total[1] + n[1], total[2] + n[2]];
            }
            if y != CLOTH_INNER - 1 {
                let y_diff = sub3(self.verts[x + (y + 1) * CLOTH_INNER].pos, pos);
                let n = norm_zp(cross(x_diff, y_diff));
                total = [total[0] + n[0], total[1] + n[1], total[2] + n[2]];
            }
        }
        let yaw = ssin((CLOTH_NORM_YAW_STEP * x as i32) as i16) * CLOTH_NORM_YAW_AMP;
        let mut dst = rot_y(total, yaw);
        if near_zero(dst) {
            dst = CLOTH_NORMAL_FALLBACK;
        } else {
            let len = (dst[0] * dst[0] + dst[1] * dst[1] + dst[2] * dst[2]).sqrt();
            dst = [dst[0] / len, dst[1] / len, dst[2] / len];
        }
        self.verts[index].nrm = dst;
    }

    pub fn set_back_nrm(&mut self) {
        for vert in &mut self.verts {
            vert.back_nrm = [-vert.nrm[0], -vert.nrm[1], -vert.nrm[2]];
        }
    }

    pub fn cloth_init(&mut self) {
        let tune = self.tuning();
        for y in 0..CLOTH_INNER {
            let y_angle = (-(y as i32) * CLOTH_ROW_ANGLE as i32) as i16;
            for x in 0..CLOTH_INNER {
                let index = x + y * CLOTH_INNER;
                let phase = (CLOTH_COL_ANGLE * x as f32) as i16;
                self.verts[index].pos = [
                    ssin(phase) * CLOTH_SIN_AMP
                        + CLOTH_BASE_X
                        + x as f32 * CLOTH_INIT_X_STEP * ssin(y_angle),
                    y as f32 * CLOTH_ROW_STEP_POS + CLOTH_BASE_Y,
                    (1.0 - scos(phase)) * CLOTH_SIN_AMP_Z
                        + CLOTH_BASE_Z
                        + x as f32 * CLOTH_INIT_Z_STEP * scos(y_angle),
                ];
                let off_phase = (phase as f32 + CLOTH_OFF_PHASE) as i16;
                self.verts[index].offset = [
                    scos(off_phase) * CLOTH_OFF_AMP,
                    0.0,
                    ssin(off_phase) * CLOTH_OFF_AMP_Z,
                ];
            }
        }
        for y in 0..CLOTH_INNER {
            for x in 0..CLOTH_INNER {
                self.set_nrm_vtx(x, y);
            }
        }
        self.set_back_nrm();
        self.wave_progress = 0;
        self.fade_in_counter = 0;
        self.cloth_move();
        self.alpha_out = 0;
        self.current_alpha = tune.cloth_color[3];
        self.wave_progress = self.wave_progress.wrapping_add(tune.wave_progress_step);
        self.cloth_color = tune.cloth_color;
        self.shadow_color = tune.shadow_color;
        self.salt = self.salt.wrapping_add(0x9E37_79B9);
        match self.kind {
            CLOTH_TYPE_DEFAULT => {
                self.init_angle_z = self
                    .init_angle_z
                    .wrapping_add(deg2s(CLOTH_RANDOM_Z_BASE + rand_fx(CLOTH_RANDOM_Z_DEG, self.salt)));
                self.scale = tune.scale;
                self.rot = [0, 0, self.init_angle_z];
                let mut n = tune.wave_pre_steps;
                while n != 0 {
                    n -= 1;
                    self.cloth_move_sin();
                }
            }
            CLOTH_TYPE_FILE_SELECT => {
                self.scale = CLOTH_SCALE_FILE_SELECT;
                self.rot = [0; 3];
            }
            _ => {
                self.scale = tune.scale;
                self.rot = tune.rot;
                self.rot[2] = self
                    .rot[2]
                    .wrapping_add(deg2s(rand_fx(CLOTH_ONLY_RANDOM_DEG, self.salt)));
                let mut n = tune.wave_pre_steps;
                while n != 0 {
                    n -= 1;
                    self.cloth_move_sin();
                }
            }
        }
    }

    pub fn cloth_move_sin(&mut self) {
        let tune = self.tuning();
        self.wave_progress = self.wave_progress.wrapping_add(tune.wave_progress_step);
        for y in 0..CLOTH_INNER {
            for x in 0..CLOTH_INNER {
                let (fx, fy) = cloth_factors(x, y);
                let mut pos = [
                    CLOTH_BASE_X + CLOTH_WAVE_X_STEP * x as f32,
                    CLOTH_BASE_Y + CLOTH_ROW_STEP_POS * y as f32,
                    CLOTH_BASE_Z,
                ];
                let wave = self
                    .wave_progress
                    .wrapping_add((x as i16).wrapping_mul(tune.wave_freq_x))
                    .wrapping_add((y as i16).wrapping_mul(tune.wave_freq_y));
                pos[0] += tune.wave_amp_x * fx * ssin(wave);
                pos[1] += tune.wave_amp_y * fx * scos(wave);
                pos[2] += tune.wave_amp_z
                    * fx
                    * ssin(
                        self.wave_progress
                            .wrapping_add((x as i16).wrapping_mul(tune.wave_freq_x)),
                    );
                let index = x + y * CLOTH_INNER;
                add_calc_pos(&mut self.verts[index].pos, pos, CLOTH_WAVE_MOVE_SCALE, tune.max_step);
            }
        }
        self.refresh_normals();
    }

    pub fn cloth_move_simple(&mut self) {
        let tune = self.tuning();
        for y in 0..CLOTH_INNER {
            for x in 0..CLOTH_INNER {
                let (fx, fy) = cloth_factors(x, y);
                let mut pos = [
                    CLOTH_BASE_X + CLOTH_WAVE_X_STEP * x as f32,
                    CLOTH_BASE_Y + CLOTH_ROW_STEP_POS * y as f32,
                    CLOTH_BASE_Z,
                ];
                let wave = self
                    .wave_progress
                    .wrapping_add((x as i16).wrapping_mul(tune.wave_freq_x))
                    .wrapping_add((y as i16).wrapping_mul(tune.wave_freq_y));
                pos[0] += tune.wave_amp_x * fx * ssin(wave);
                pos[1] += tune.wave_amp_y * fx * scos(wave);
                pos[2] += tune.wave_amp_z
                    * fx
                    * ssin(
                        self.wave_progress
                            .wrapping_add((y as i16).wrapping_mul(tune.wave_freq_y)),
                    );
                let index = x + y * CLOTH_INNER;
                add_calc_pos(&mut self.verts[index].pos, pos, CLOTH_WAVE_MOVE_SCALE, tune.max_step);
            }
        }
        self.refresh_normals();
    }

    fn refresh_normals(&mut self) {
        for y in 0..CLOTH_INNER {
            for x in 0..CLOTH_INNER {
                self.set_nrm_vtx(x, y);
            }
        }
        self.set_back_nrm();
    }

    pub fn cloth_move(&mut self) {
        let tune = self.tuning();
        if self.kind == CLOTH_TYPE_FILE_SELECT {
            if self.fade_in_counter < tune.fade_in_length {
                self.fade_in_counter += 1;
            }
            self.cloth_move_sin();
        } else if self.fade_in_counter < tune.fade_in_length {
            self.cloth_move_sin();
            self.fade_in_counter += 1;
        } else {
            self.cloth_move_simple();
        }
    }

    pub fn alpha_out(&mut self) {
        self.alpha_out = 1;
    }

    pub fn cloth_color_now(&mut self) -> [u8; 4] {
        let tune = self.tuning();
        let mut color = self.cloth_color;
        if self.alpha_out == 1 {
            color[3] = self.current_alpha;
            self.current_alpha = if self.current_alpha > CLOTH_FADE_STEP {
                self.current_alpha - CLOTH_FADE_STEP
            } else {
                0
            };
        } else {
            let f = self.fade_in_counter as f32 / tune.fade_in_length as f32;
            let cloth = min_max_limit(tune.cloth_color[3] as i16, 0, 0xFF) as f32;
            color[3] = (cloth * f + (1.0 - f) * tune.start_alpha as f32) as u8;
        }
        color
    }

    pub fn shadow_color_now(&self) -> [u8; 4] {
        let tune = self.tuning();
        let mut color = self.shadow_color;
        if self.alpha_out == 1 {
            return color;
        }
        let f = self.fade_in_counter as f32 / tune.fade_in_length as f32;
        let shadow = min_max_limit(tune.shadow_color[3] as i16, 0, 0xFF) as f32;
        color[3] = (shadow * f + (1.0 - f) * tune.start_alpha as f32) as u8;
        color
    }

    pub fn shadow_positions(&mut self) {
        for vert in &mut self.verts {
            vert.shadow = [vert.pos[0], vert.pos[1], CLOTH_SHADOW_Z];
        }
    }

    pub fn light_position(&self) -> [f32; 3] {
        let tune = self.tuning();
        [
            -(tune.light_distance * ssin(tune.light_yaw) * scos(tune.light_pitch)),
            -(tune.light_distance * ssin(tune.light_pitch)),
            -(tune.light_distance * scos(tune.light_yaw) * scos(tune.light_pitch)),
        ]
    }

    pub fn mesh(&mut self) -> ClothMesh {
        self.shadow_positions();
        let color = self.cloth_color_now();
        let vertex_alpha = if self.kind == CLOTH_TYPE_FILE_SELECT {
            Some(self.file_select_alpha(color))
        } else {
            None
        };
        ClothMesh {
            vertices: self.verts.map(|vert| vert.pos),
            normals: self.verts.map(|vert| vert.nrm),
            back_normals: self.verts.map(|vert| vert.back_nrm),
            shadows: self.verts.map(|vert| vert.shadow),
            scale: self.scale,
            rotation: self.rot,
            color,
            shadow_color: self.shadow_color_now(),
            light: self.light_position(),
            model_offset: [
                self.tuning().pos[0] - CLOTH_MODEL_DX,
                self.tuning().pos[1],
                self.tuning().pos[2] - CLOTH_MODEL_DZ,
            ],
            shadow_offset: [
                self.tuning().pos[0] - CLOTH_SHADOW_DX,
                self.tuning().pos[1] - CLOTH_SHADOW_DY,
                self.tuning().pos[2] - CLOTH_MODEL_DZ,
            ],
            alpha_out: self.alpha_out,
            fade: self.fade_in_counter as f32 / self.tuning().fade_in_length as f32,
            inner: CLOTH_INNER,
            vertex_alpha,
        }
    }

    fn file_select_alpha(&self, base: [u8; 4]) -> Vec<[u8; 4]> {
        let mut out = Vec::with_capacity(CLOTH_VERTS);
        for y in 0..CLOTH_INNER {
            for x in 0..CLOTH_INNER {
                let from_right = 10 - x as i32;
                let mut color = base;
                color[3] = max_limit(from_right * 25 + y as i32 * 25, 0xFF) as u8;
                out.push(color);
            }
        }
        out
    }
}

fn cloth_factors(x: usize, y: usize) -> (f32, f32) {
    let mut fx = x as f32;
    let mut fy = y as f32;
    if x > CLOTH_INNER / 2 {
        fx = (CLOTH_INNER as f32 - fx) - 1.0;
    }
    if y > CLOTH_INNER / 2 {
        fy = (CLOTH_INNER as f32 - fy) - 1.0;
    }
    let fx = if fx + fy > 4.0 { 1.0 } else { (fx + fy) * 0.2 };
    (fx, fy)
}

fn add_calc_pos(value: &mut [f32; 3], target: [f32; 3], scale: f32, max_step: f32) {
    if *value == target {
        return;
    }
    let mut diff = [
        (value[0] - target[0]) * scale,
        (value[1] - target[1]) * scale,
        (value[2] - target[2]) * scale,
    ];
    let len = (diff[0] * diff[0] + diff[1] * diff[1] + diff[2] * diff[2]).sqrt();
    if len > max_step {
        diff = norm_zp(diff);
        diff = [diff[0] * max_step, diff[1] * max_step, diff[2] * max_step];
    }
    value[0] -= diff[0];
    value[1] -= diff[1];
    value[2] -= diff[2];
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
        Self::new(
            5,
            2,
            3,
            2,
            STICK_MAX_VALUE,
            STICK_MIN_VALUE,
            0,
            STICK_HALF_DEAD,
        )
    }
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

pub fn stick_period() -> i16 {
    MSG_STICK_B + MSG_STICK_A + MSG_STICK_C + MSG_STICK_A
}

pub fn icon_texture(icon: u8) -> &'static str {
    BUTTON_TEX.get(usize::from(icon)).copied().unwrap_or("")
}

pub fn icon_color(icon: u8) -> u32 {
    BUTTON_COLOR
        .get(usize::from(icon))
        .map(|color| pack_color(*color))
        .unwrap_or(0xFFFF_FFFF)
}

pub fn icon_palette(icon: u8) -> u8 {
    match icon {
        ICON_FLASHING_A_BUTTON_2 => ICON_FLASHING_A_BUTTON,
        ICON_HEART_2 => ICON_HEART,
        ICON_MUSIC_NOTE_2 => ICON_MUSIC_NOTE,
        ICON_STARBURST_2 => ICON_STARBURST,
        other => other,
    }
}

fn stick_blend(timer: i16, piece: i16) -> [f32; 4] {
    let period = stick_period();
    let start = MSG_STICK_B + period * piece;
    let fade_in = MSG_STICK_A + start;
    let solid = MSG_STICK_C + fade_in;
    let fade_out = MSG_STICK_A + solid;
    if timer < start {
        [0.0, 1.0, 1.0, 1.0]
    } else if timer < fade_in {
        let v = value_increase(i32::from(period), i32::from(timer - start), 0);
        [v, 1.0 - v, 1.0, 1.0]
    } else if timer < solid {
        [1.0, 0.0, 1.0, 1.0]
    } else if timer < fade_out {
        let v = value_increase(i32::from(period), i32::from(timer - solid), 0);
        [1.0 - v, v, 1.0, 1.0]
    } else {
        [0.0, 1.0, 1.0, 1.0]
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OutFontIcon {
    pub x: f32,
    pub y: f32,
    pub offset_x: f32,
    pub offset_y: f32,
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
    pub pic: [usize; 3],
    pub ruby_pane: Option<usize>,
    pub ruby_msg: u32,
    pub cell: f32,
    pub char_space: f32,
    pub line_width: f32,
    pub line_space: f32,
    pub left_up_x: f32,
    pub left_up_y: f32,
    pub icons: [OutFontIcon; 3],
    pub icon_count: u8,
    pub line: i16,
    pub advance: f32,
    pub text: String,
    pub ruby_text: String,
    pub alpha: u8,
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
            left_up_x: 0.0,
            left_up_y: 0.0,
            icons: [OutFontIcon::default(); 3],
            icon_count: 0,
            line: 0,
            advance: 0.0,
            text: String::new(),
            ruby_text: String::new(),
            alpha: 0xFF,
        }
    }
}

impl OutFont {
    pub fn set_pane(
        &mut self,
        panes: &mut ItemPanes,
        text_pane: usize,
        pic1: usize,
        pic2: usize,
        pic3: usize,
    ) {
        self.box_pane = text_pane;
        self.pic = [pic1, pic2, pic3];
        self.ruby_pane = None;
        self.initial(panes);
        for index in [pic1, pic2, pic3] {
            if let Some(pane) = panes.panes.get_mut(index) {
                pane.change_texture(OUT_FONT_TEXTURE);
                pane.append(OUT_FONT_TEXTURE);
            }
        }
    }

    pub fn set_ruby(&mut self, ruby_pane: usize) {
        self.ruby_pane = Some(ruby_pane);
    }

    pub fn set_left_up_pos(&mut self, x: f32, y: f32) {
        self.left_up_x = x;
        self.left_up_y = y;
    }

    pub fn initial(&mut self, panes: &mut ItemPanes) {
        self.advance = 0.0;
        self.icon_count = 0;
        self.left_up_y = 0.0;
        self.left_up_x = 0.0;
        self.line = 0;
        self.text.clear();
        self.ruby_text.clear();
        if let Some(pane) = panes.panes.get_mut(self.box_pane) {
            pane.show();
            self.cell = pane.font_size_x;
            self.char_space = pane.char_space;
            self.line_width = pane.width();
            self.line_space = pane.line_space;
        }
        for index in 0..3 {
            self.icons[index] = OutFontIcon {
                x: 0.0,
                y: 0.0,
                offset_x: 0.0,
                offset_y: 0.0,
                scale_x: self.cell,
                scale_y: self.cell,
                timer: 0,
                icon_no: ICON_A_BUTTON,
                line: -1,
                texture: icon_texture(ICON_A_BUTTON),
                blend: [0.0, 1.0, 1.0, 1.0],
                rotation: 0,
                visible: false,
            };
            if let Some(pane) = panes.panes.get_mut(self.pic[index]) {
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

    fn icon_set(&mut self, panes: &mut ItemPanes, icon_no: u8, out: &mut String) -> f32 {
        let mut advance = 0.0;
        if self.icon_count >= 3 {
            return advance;
        }
        let index = usize::from(self.icon_count);
        self.icons[index].x = self.advance;
        self.icons[index].y = 0.0;
        self.icons[index].icon_no = icon_no;
        self.icons[index].line = self.line;
        self.icons[index].texture = icon_texture(icon_no);
        self.icons[index].blend = [0.0, 1.0, 1.0, 1.0];
        self.icons[index].visible = true;
        let pane_index = self.pic[index];
        if let Some(pane) = panes.panes.get_mut(pane_index) {
            pane.show();
            pane.remove();
            pane.append(icon_texture(icon_no));
            pane.set_blend([0.0, 1.0, 1.0, 1.0]);
            pane.set_white(icon_color(icon_no));
            pane.set_black(0);
            match icon_no {
                ICON_ARROW_LEFT => {
                    self.icons[index].x += self.cell;
                    pane.rotate_deg(0.0, 0.0, 270.0);
                    self.icons[index].rotation = pane.rotation;
                }
                ICON_ARROW_RIGHT => {
                    self.icons[index].y += self.cell;
                    pane.rotate_deg(0.0, 0.0, 90.0);
                    self.icons[index].rotation = pane.rotation;
                }
                ICON_ARROW_UP => {
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
        panes: &mut ItemPanes,
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
                let icon_no = match code {
                    _ if code == (0xFF0000 | SPCL_COLOR) => {
                        line_text.push_str("\u{1b}CC[00000000]\u{1b}GM[0]");
                        None
                    }
                    _ if code == (0xFF0000 | SPCL_RUBY) => None,
                    CTRL_A_BUTTON => Some(ICON_A_BUTTON),
                    CTRL_B_BUTTON => Some(ICON_B_BUTTON),
                    CTRL_C_STICK => Some(ICON_C_STICK),
                    CTRL_L_BUTTON => Some(ICON_L_BUTTON),
                    CTRL_R_BUTTON => Some(ICON_R_BUTTON),
                    CTRL_X_BUTTON => Some(ICON_X_BUTTON),
                    CTRL_Y_BUTTON => Some(ICON_Y_BUTTON),
                    CTRL_Z_BUTTON => Some(ICON_Z_BUTTON),
                    CTRL_DPAD => Some(ICON_DPAD),
                    CTRL_MAIN_STICK => Some(ICON_MAIN_STICK),
                    CTRL_ARROW_LEFT => Some(ICON_ARROW_LEFT),
                    CTRL_ARROW_RIGHT => Some(ICON_ARROW_RIGHT),
                    CTRL_ARROW_UP => Some(ICON_ARROW_UP),
                    CTRL_ARROW_DOWN => Some(ICON_ARROW_DOWN),
                    CTRL_MAIN_STICK_UP => Some(ICON_MAIN_STICK_UP),
                    CTRL_MAIN_STICK_DOWN => Some(ICON_MAIN_STICK_DOWN),
                    CTRL_MAIN_STICK_LEFT => Some(ICON_MAIN_STICK_LEFT),
                    CTRL_MAIN_STICK_RIGHT => Some(ICON_MAIN_STICK_RIGHT),
                    CTRL_MAIN_STICK_UP_DOWN => Some(ICON_MAIN_STICK_UP_DOWN),
                    CTRL_MAIN_STICK_LEFT_RIGHT => Some(ICON_MAIN_STICK_LEFT_RIGHT),
                    CTRL_FLASHING_A_BUTTON => Some(ICON_FLASHING_A_BUTTON),
                    CTRL_HEART => Some(ICON_HEART),
                    CTRL_MUSIC_NOTE => Some(ICON_MUSIC_NOTE),
                    _ => None,
                };
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
                            for icon in 0..3 {
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

        let shift_y =
            ((self.box_height(panes) - self.cell) - line_no as f32 * self.line_space) / 2.0;
        let shift = ((self.line_width - self.advance) / 2.0) as i32;
        let tail = if shift > 0 {
            format!("\u{1b}CR[{shift}]")
        } else {
            String::new()
        };
        if shift > 0 {
            for icon in 0..3 {
                if line_no == self.icons[icon].line {
                    self.icons[icon].x += shift as f32;
                }
                if self.icons[icon].line != -1 {
                    self.icons[icon].y += shift_y + self.icons[icon].line as f32 * self.line_space;
                }
            }
        }
        self.text.push_str(&tail);
        self.text.push_str(&line_text);
        if let Some(pane) = panes.panes.get_mut(self.box_pane) {
            pane.shift_set(0.0, shift_y);
            pane.set_string(&self.text);
        }
        if let Some(ruby) = self.ruby_pane
            && let Some(pane) = panes.panes.get_mut(ruby)
        {
            pane.shift_set(0.0, shift_y);
            pane.set_string(&self.ruby_text);
        }
    }

    fn box_height(&self, panes: &ItemPanes) -> f32 {
        panes
            .panes
            .get(self.box_pane)
            .map(|pane| pane.height())
            .unwrap_or(0.0)
    }

    fn out_font_stick_anime(&mut self, panes: &mut ItemPanes, index: usize, which: u8) {
        let period = stick_period();
        if self.icons[index].timer < period {
            self.icons[index].blend = stick_blend(self.icons[index].timer, 0);
            let cell = self.cell;
            let (ox, oy) = match which {
                0 => (0.0, 0.0),
                1 => (cell, 0.0),
                2 => (cell, cell),
                _ => (0.0, cell),
            };
            self.icons[index].offset_x = ox;
            self.icons[index].offset_y = oy;
        }
        let angle = f32::from(which) * -90.0;
        self.icons[index].rotation = deg2s(angle) as u16;
        if let Some(pane) = panes.panes.get_mut(self.pic[index]) {
            pane.rotate_deg(0.0, 0.0, angle);
        }
        self.icons[index].timer += 1;
        if self.icons[index].timer >= period {
            self.icons[index].timer = 0;
        }
    }

    fn out_font_stick_anime_cycle(&mut self, panes: &mut ItemPanes, index: usize, which: u8) {
        let period = stick_period();
        let cell = self.cell;
        let mut facing = 0u8;
        if which == 0 {
            if self.icons[index].timer < period {
                self.icons[index].blend = stick_blend(self.icons[index].timer, 0);
                self.icons[index].offset_x = cell;
                self.icons[index].offset_y = 0.0;
                facing = 1;
            } else if self.icons[index].timer < period * 2 {
                self.icons[index].blend = stick_blend(self.icons[index].timer, 1);
                self.icons[index].offset_x = 0.0;
                self.icons[index].offset_y = cell;
                facing = 3;
            }
        } else if self.icons[index].timer < period {
            self.icons[index].blend = stick_blend(self.icons[index].timer, 0);
            self.icons[index].offset_x = 0.0;
            self.icons[index].offset_y = 0.0;
            facing = 0;
        } else if self.icons[index].timer < period * 2 {
            self.icons[index].blend = stick_blend(self.icons[index].timer, 1);
            self.icons[index].offset_x = cell;
            self.icons[index].offset_y = cell;
            facing = 2;
        }
        let angle = f32::from(facing) * -90.0;
        self.icons[index].rotation = deg2s(angle) as u16;
        if let Some(pane) = panes.panes.get_mut(self.pic[index]) {
            pane.rotate_deg(0.0, 0.0, angle);
        }
        self.icons[index].timer += 1;
        if self.icons[index].timer >= period * 2 {
            self.icons[index].timer = 0;
        }
    }

    fn out_font_stick_anime_wheel(&mut self, panes: &mut ItemPanes, index: usize) {
        let period = stick_period();
        let timer = self.icons[index].timer;
        if timer < period {
            self.icons[index].blend = stick_blend(timer, 0);
            self.icons[index].offset_x = 0.0;
            self.icons[index].offset_y = 0.0;
        } else if timer < period * 2 {
            self.icons[index].blend = stick_blend(timer, 1);
            self.icons[index].offset_x = self.cell;
            self.icons[index].offset_y = 0.0;
        } else if timer < period * 3 {
            self.icons[index].blend = stick_blend(timer, 2);
            self.icons[index].offset_x = self.cell;
            self.icons[index].offset_y = self.cell;
        } else if timer < period * 4 {
            self.icons[index].blend = stick_blend(timer, 3);
            self.icons[index].offset_x = 0.0;
            self.icons[index].offset_y = self.cell;
        }
        if timer % period == 0 {
            let angle = (timer / period) as f32 * -90.0;
            self.icons[index].rotation = deg2s(angle) as u16;
            if let Some(pane) = panes.panes.get_mut(self.pic[index]) {
                pane.rotate_deg(0.0, 0.0, angle);
            }
        }
        self.icons[index].timer += 1;
        if self.icons[index].timer >= period * 4 {
            self.icons[index].timer = 0;
        }
    }

    pub fn move_out_font(&mut self, panes: &mut ItemPanes) {
        for index in 0..usize::from(self.icon_count) {
            match self.icons[index].icon_no {
                ICON_MAIN_STICK => self.out_font_stick_anime_wheel(panes, index),
                ICON_MAIN_STICK_UP => self.out_font_stick_anime(panes, index, 1),
                ICON_MAIN_STICK_DOWN => self.out_font_stick_anime(panes, index, 3),
                ICON_MAIN_STICK_LEFT => self.out_font_stick_anime(panes, index, 0),
                ICON_MAIN_STICK_RIGHT => self.out_font_stick_anime(panes, index, 2),
                ICON_MAIN_STICK_UP_DOWN => self.out_font_stick_anime_cycle(panes, index, 0),
                ICON_MAIN_STICK_LEFT_RIGHT => self.out_font_stick_anime_cycle(panes, index, 1),
                _ => {}
            }
            let blend = self.icons[index].blend;
            if let Some(pane) = panes.panes.get_mut(self.pic[index]) {
                pane.set_blend(blend);
            }
        }
        for index in 0..3 {
            if !self.icons[index].visible {
                continue;
            }
            let x = self.left_up_x + self.icons[index].x + self.icons[index].offset_x;
            let y = self.left_up_y + self.icons[index].y + self.icons[index].offset_y - 1.0;
            if let Some(pane) = panes.panes.get_mut(self.pic[index]) {
                pane.pos_top_left.x = x;
                pane.pos_top_left.y = y;
            }
        }
        if let Some(pane) = panes.panes.get_mut(self.box_pane) {
            pane.pos_top_left.x = self.left_up_x;
            pane.pos_top_left.y = self.left_up_y;
        }
    }

    pub fn set_alpha(&mut self, panes: &mut ItemPanes, alpha: u8) {
        self.alpha = alpha;
        for index in 0..3 {
            if let Some(pane) = panes.panes.get_mut(self.pic[index]) {
                pane.now_alpha = alpha;
            }
        }
        if let Some(pane) = panes.panes.get_mut(self.box_pane) {
            pane.now_alpha = alpha;
        }
    }
}

#[derive(Clone, Debug)]
pub struct ItemMenu {
    pub panes: ItemPanes,
    pub stick: StickControl,
    pub out_font: OutFont,
    pub equip: Equipped,
    pub record: ItemRecord,
    pub contents_color: ContentsColor,
    pub name_buf: [String; NAME_BUFFER_COUNT],
    pub note_buf: [String; NAME_BUFFER_COUNT],
    pub dummy_buf: [String; NAME_BUFFER_COUNT],
    pub timer: i16,
    pub frame_timer: i16,
    pub sheen_timer: i16,
    pub item_mode: u8,
    pub now_item: u8,
    pub sub_menu_item_base: u8,
    pub select_item_slot: u8,
    pub swap_slot: u8,
    pub select_item_btn: u8,
    pub swap_btn: u8,
    pub sub_menu_kind: u8,
    pub items: [u8; SLOT_COUNT],
    pub picture_num: u8,
    pub arrow_num: u8,
    pub bomb_num: u8,
    pub trigger_info: u8,
    pub fading: u8,
    pub sub_win: bool,
    pub now_item_button: u8,
    pub cursor_off: [f32; CURSOR_PARTS],
    pub cursor_dy: [f32; CURSOR_PARTS],
    pub emitters: [EmitterState; 2],
    pub note: MsgData,
    pub tuning: ItemTuning,
    pub msg_font_size: i32,
    pub msg_line_space: i32,
    pub opt_ruby: bool,
    pub boss_rematch: bool,
    pub item_icons: [Option<&'static str>; SLOT_COUNT],
    pub sub_icons: [Option<&'static str>; SUB_ITEM_COUNT],
    pub play_select: [u8; SELECT_SLOTS],
    pub save_open: bool,
    pub save_status: u8,
    pub save_timer: i16,
    pub name_pending: bool,
}

impl Default for ItemMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemMenu {
    pub fn empty() -> Self {
        Self {
            panes: ItemPanes::default(),
            stick: StickControl::default(),
            out_font: OutFont::default(),
            equip: Equipped::default(),
            record: ItemRecord::default(),
            contents_color: ContentsColor::default(),
            name_buf: [String::new(), String::new()],
            note_buf: [String::new(), String::new()],
            dummy_buf: [String::new(), String::new()],
            timer: 0,
            frame_timer: 0,
            sheen_timer: 0,
            item_mode: 0,
            now_item: 0,
            sub_menu_item_base: 0,
            select_item_slot: 0,
            swap_slot: 0,
            select_item_btn: BTN_NONE,
            swap_btn: BTN_NONE,
            sub_menu_kind: 3,
            items: [ITEM_NONE; SLOT_COUNT],
            picture_num: 0,
            arrow_num: 0,
            bomb_num: 0,
            trigger_info: 0,
            fading: 0,
            sub_win: false,
            now_item_button: 0,
            cursor_off: [0.0; CURSOR_PARTS],
            cursor_dy: [0.0; CURSOR_PARTS],
            emitters: [EmitterState::default(); 2],
            note: MsgData::new_data_proc(),
            tuning: ItemTuning::default(),
            msg_font_size: MSG_FONT_SIZE_USA,
            msg_line_space: MSG_LINE_SPACE_USA,
            opt_ruby: false,
            boss_rematch: false,
            item_icons: [None; SLOT_COUNT],
            sub_icons: [None; SUB_ITEM_COUNT],
            play_select: [ITEM_NONE; SELECT_SLOTS],
            save_open: false,
            save_status: 0,
            save_timer: 0,
            name_pending: false,
        }
    }

    pub fn new() -> Self {
        let mut menu = Self::empty();
        menu.screen_set();
        menu.initialize();
        menu.cursor_anime();
        menu
    }

    pub fn bind(&mut self, screen: &Screen) {
        self.panes.bind(screen);
        self.screen_set();
        self.initialize();
        self.cursor_anime();
    }

    pub fn sync(&mut self, equipped: &Equipped, record: &ItemRecord) {
        self.equip = *equipped;
        self.record = *record;
        self.play_select = *record.select_item;
    }

    pub fn slot_item(&self, slot: u8) -> u8 {
        if slot < SLOT_LAST {
            self.equip.item(usize::from(slot))
        } else if slot < SLOT_BEAST_FIRST {
            ITEM_NONE
        } else if slot < SLOT_BEAST_LAST {
            self.record.beast[usize::from(slot - SLOT_BEAST_FIRST)]
        } else if slot < SLOT_BAIT_FIRST {
            ITEM_NONE
        } else if slot < SLOT_BAIT_LAST {
            self.record.bait[usize::from(slot - SLOT_BAIT_FIRST)]
        } else if slot < SLOT_RESERVE_FIRST {
            ITEM_NONE
        } else if slot < SLOT_RESERVE_LAST {
            self.record.reserve[usize::from(slot - SLOT_RESERVE_FIRST)]
        } else {
            ITEM_NONE
        }
    }

    pub fn disabled(&self, item: u8) -> bool {
        is_recollect_disabled(item, self.boss_rematch)
    }

    pub fn screen_set(&mut self) {
        for index in 0..NOTE_SLOTS {
            self.panes.note_icon_mut(index).user_area = -1;
        }
        if self.opt_ruby {
            self.panes.pane_mut(RUBY_OPTION_PANE).pane_trans(0.0, RUBY_OPTION_DY);
        }
        let anchor_angle = self.panes.pane(P_NO11).rotation as i16;
        self.panes.pane_mut(P_NO11).user_area = anchor_angle;
        self.panes.pane_mut(P_STR0).set_white(0xFFFF_FFFF);
        self.panes.pane_mut(P_STR0).set_black(0x0000_00FF);
        self.panes.pane_mut(P_STR0).set_char_color(0x0000_00FF);
        self.panes.pane_mut(P_STR0).set_grad_color(0x0000_00FF);
        self.panes.pane_mut(P_ST00).set_white(0xFFFF_FFFF);
        self.panes.pane_mut(P_ST00).set_black(0x0000_00FF);
        self.panes.pane_mut(P_ST00).set_char_color(0x0000_00FF);
        self.panes.pane_mut(P_ST00).set_grad_color(0x0000_00FF);
        self.panes.out_box_mut(0).show();
        self.out_font
            .set_pane(&mut self.panes, P_WD, P_FT15, P_FT16, P_FT17);
        for index in 0..NAME_BUFFER_COUNT {
            self.panes.name_box_mut(index).set_font_size(
                self.panes.name_box(0).font_size_x,
                self.panes.name_box(0).font_size_y,
            );
        }
        self.panes
            .name_box_mut(1)
            .set_char_space(self.panes.name_box(0).get_char_space());
        for index in 0..SUB_ITEM_COUNT {
            self.panes.sub_item_mut(index).hide();
            self.panes.sub_shadow_mut(index).hide();
        }
        for index in 0..3 {
            self.panes.arrow_tint_mut(index).hide();
        }
    }

    pub fn initialize(&mut self) {
        self.item_bit_check(true);
        self.timer = 0;
        self.panes.out_box_mut(0).user_area = 10;
        self.panes.cursor_mut(0).user_area = 0;
        self.fading = 0;
        self.note_init();
        self.out_font_init();
        let select = [
            self.record.select(BTN_X),
            self.record.select(BTN_Y),
            self.record.select(BTN_Z),
        ];
        self.check_move(&select);
        self.item_number_set();
        self.panes.pane_mut(P_MSK0).init_alpha = self.tuning.mask_alpha;
        self.frame_timer = 0;
        self.sheen_timer = 0;
        self.contents_color = self.panes.pane(P_WK01).contents_color;
        for index in 0..CURSOR_PARTS {
            let x = self.panes.cursor(index).pos_center_orig.x
                - self.panes.item_icon(0).pos_center_orig.x;
            let y = self.panes.cursor(index).pos_center_orig.y
                - self.panes.item_icon(0).pos_center_orig.y;
            self.cursor_off[index] = x;
            self.cursor_dy[index] = y;
            self.panes.cursor_mut(index).append(CURSOR_TEXTURE);
        }
        for index in 0..NOTE_SLOTS {
            self.panes.note_icon_mut(index).change_texture(OUT_FONT_TEXTURE);
            self.panes.note_icon_mut(index).blend_init(BLEND_SHADOW_TEXTURE);
        }
        self.sub_menu_item_base = 0;
        self.select_item_slot = 0;
        self.swap_slot = 0;
        self.bomb_num = 0;
        self.arrow_num = 0;
        self.picture_num = 0;
        self.select_item_btn = BTN_NONE;
        self.swap_btn = BTN_NONE;
        self.sub_menu_kind = 3;
        self.panes.pane_mut(P_IT29).change_texture(RETURN_TEXTURE);
        self.panes.pane_mut(P_IK29).change_texture(RETURN_TEXTURE);
        self.stack_cursor(P_BLAK);
    }

    fn stack_cursor(&mut self, after: usize) {
        self.panes.insert_child(after, P_CAR0 + 3);
        self.panes.insert_child(P_CAR0 + 3, P_CAR0 + 2);
        self.panes.insert_child(P_CAR0 + 2, P_CAR0 + 1);
        self.panes.insert_child(P_CAR0 + 1, P_CAR0);
    }

    fn cursor_anime(&mut self) {
        let mut slot = self.now_item;
        if slot == SLOT_LAST {
            let pane = self.panes.pane(P_PANE_BG);
            let cx = pane.pos_center.x;
            let cy = pane.pos_center.y;
            let hw = pane.size.x / 2.0;
            let hh = pane.size.y / 2.0;
            let corners = [
                (cx - hw, cy + hh),
                (cx + hw, cy + hh),
                (cx - hw, cy - hh),
                (cx + hw, cy - hh),
            ];
            for index in 0..CURSOR_PARTS {
                self.panes.cursor_mut(index).pos_center_orig = F2 {
                    x: corners[index].0,
                    y: corners[index].1,
                };
            }
        } else if slot >= SLOT_BEAST_FIRST {
            if slot == SLOT_NONE {
                slot = SLOT_CAMERA;
            } else {
                slot = slot.wrapping_sub(self.sub_menu_item_base);
            }
            let pane = self.panes.sub_item(usize::from(slot)).clone();
            for index in 0..CURSOR_PARTS {
                self.panes.cursor_mut(index).pos_center_orig = F2 {
                    x: pane.pos_center.x + self.cursor_off[index],
                    y: pane.pos_center.y + self.cursor_dy[index],
                };
            }
        } else {
            let pane = self.panes.item_icon(usize::from(slot)).clone();
            for index in 0..CURSOR_PARTS {
                self.panes.cursor_mut(index).pos_center_orig = F2 {
                    x: pane.pos_center.x + self.cursor_off[index],
                    y: pane.pos_center.y + self.cursor_dy[index],
                };
            }
        }

        let half = self.tuning.cursor_half;
        let mut step = f32::from(self.tuning.cursor_step);
        if self.panes.cursor(0).user_area < half {
            for index in 0..CURSOR_PARTS {
                self.panes.cursor_mut(index).set_blend([1.0, 0.0, 1.0, 1.0]);
            }
        } else if self.panes.cursor(0).user_area < half * 2 {
            for index in 0..CURSOR_PARTS {
                self.panes.cursor_mut(index).set_blend([0.0, 1.0, 1.0, 1.0]);
            }
            step = f32::from(self.tuning.cursor_step_large);
        }

        self.panes.cursor_mut(0).pane_trans(-step, step);
        self.panes.cursor_mut(1).pane_trans(step, step);
        self.panes.cursor_mut(2).pane_trans(-step, -step);
        self.panes.cursor_mut(3).pane_trans(step, -step);

        let user = self.panes.cursor(0).user_area.wrapping_add(1);
        self.panes.cursor_mut(0).user_area = if user >= half * 2 { 0 } else { user };
    }

    fn cursor_main_move(&mut self, pad: &PadState, out: &mut MenuOutput) {
        self.stick.check_trigger(pad.stick_value, pad.stick_angle);
        let prev = self.now_item;
        if self.stick.check_right_trigger() {
            if self.now_item != SLOT_LAST {
                if self.now_item % GRID_COLUMNS == 6 {
                    self.now_item = self.now_item.wrapping_sub(6);
                } else {
                    self.now_item = self.now_item.wrapping_add(1);
                }
            }
        } else if self.stick.check_left_trigger() {
            if self.now_item != SLOT_LAST {
                if self.now_item % GRID_COLUMNS == 0 {
                    self.now_item = self.now_item.wrapping_add(6);
                } else {
                    self.now_item = self.now_item.wrapping_sub(1);
                }
            }
        }

        if self.stick.check_up_trigger() {
            if self.now_item == SLOT_LAST {
                self.now_item = SLOT_BOTTLE0;
            } else if self.now_item > SLOT_DEKU_LEAF {
                self.now_item = self.now_item.wrapping_sub(7);
            }
        } else if self.stick.check_down_trigger() {
            if self.now_item < SLOT_BOTTLE0 {
                self.now_item = self.now_item.wrapping_add(7);
            } else if self.now_item >= SLOT_BOTTLE0 && self.now_item <= SLOT_SKULL_HAMMER {
                self.now_item = SLOT_LAST;
            }
        }

        if self.now_item != prev {
            self.panes.out_box_mut(0).user_area = 0;
            self.name_pending = true;
            out.sounds.push(SE_MENU_CURSOR);
        }
    }

    fn cursor_sub_move(&mut self, pad: &PadState, tuning: &MenuTuning, out: &mut MenuOutput) {
        let rows = tuning.sub_rows;
        let mut slot = if self.now_item == SLOT_NONE {
            SLOT_CAMERA
        } else {
            self.now_item.wrapping_sub(self.sub_menu_item_base)
        };
        self.stick.check_trigger(pad.stick_value, pad.stick_angle);
        if self.stick.check_right_trigger() {
            if slot % 3 != 2 && i32::from(slot) <= i32::from(rows) - 1 {
                slot += 1;
            }
        } else if self.stick.check_left_trigger() {
            if slot % 3 != 0 {
                slot -= 1;
            }
        }
        if self.stick.check_up_trigger() {
            if slot >= 3 {
                slot -= 3;
            }
        } else if self.stick.check_down_trigger() {
            if u16::from(slot) + 3 <= rows as u16 {
                slot += 3;
            }
        }
        if slot != self.now_item.wrapping_sub(self.sub_menu_item_base) {
            if slot == SLOT_CAMERA {
                if self.now_item != SLOT_NONE {
                    self.now_item = SLOT_NONE;
                    out.sounds.push(SE_MENU_CURSOR);
                }
            } else {
                self.now_item = slot.wrapping_add(self.sub_menu_item_base);
                out.sounds.push(SE_MENU_CURSOR);
            }
            self.panes.out_box_mut(0).user_area = 0;
            self.name_pending = true;
        }
    }

    fn check_move(&mut self, select_item: &[u8; BTN_COUNT]) {
        for i in 0..BTN_COUNT {
            let mut item = select_item[i];
            if item == SLOT_NONE {
                self.panes.button_check_mut(i).hide();
                continue;
            }
            self.panes.button_check_mut(i).show();
            if self.sub_win {
                if item >= SLOT_RESERVE_FIRST {
                    if self.sub_menu_kind == 2 {
                        item = item.wrapping_sub(SLOT_RESERVE_FIRST);
                        self.panes.button_check_mut(i).pos_center =
                            self.panes.sub_item(usize::from(item)).pos_center;
                        self.panes.insert_child(P_IK29, P_CHK1 + i);
                    } else {
                        item = item_num(ITEM_DELIVERY_BAG);
                        self.panes.button_check_mut(i).pos_center =
                            self.panes.item_icon(usize::from(item)).pos_center_orig;
                        self.panes.insert_child(P_IK0 + 20, P_CHK1 + i);
                    }
                } else if item >= SLOT_BAIT_FIRST {
                    if self.sub_menu_kind == 1 {
                        item = item.wrapping_sub(SLOT_BAIT_FIRST);
                        self.panes.button_check_mut(i).pos_center =
                            self.panes.sub_item(usize::from(item)).pos_center;
                        self.panes.insert_child(P_IK29, P_CHK1 + i);
                    } else {
                        item = item_num(ITEM_BAIT_BAG);
                        self.panes.button_check_mut(i).pos_center =
                            self.panes.item_icon(usize::from(item)).pos_center_orig;
                        self.panes.insert_child(P_IK0 + 20, P_CHK1 + i);
                    }
                } else if item >= SLOT_BEAST_FIRST {
                    if self.sub_menu_kind == 0 {
                        item = item.wrapping_sub(SLOT_BEAST_FIRST);
                        self.panes.button_check_mut(i).pos_center =
                            self.panes.sub_item(usize::from(item)).pos_center;
                        self.panes.insert_child(P_IK29, P_CHK1 + i);
                    } else {
                        item = item_num(ITEM_SPOILS_BAG);
                        self.panes.button_check_mut(i).pos_center =
                            self.panes.item_icon(usize::from(item)).pos_center_orig;
                        self.panes.insert_child(P_IK0 + 20, P_CHK1 + i);
                    }
                } else {
                    self.panes.button_check_mut(i).pos_center =
                        self.panes.item_icon(usize::from(item)).pos_center_orig;
                    self.panes.insert_child(P_IK0 + 20, P_CHK1 + i);
                }
            } else {
                if item >= SLOT_RESERVE_FIRST {
                    item = item_num(ITEM_DELIVERY_BAG);
                } else if item >= SLOT_BAIT_FIRST {
                    item = item_num(ITEM_BAIT_BAG);
                } else if item >= SLOT_BEAST_FIRST {
                    item = item_num(ITEM_SPOILS_BAG);
                }
                self.panes.button_check_mut(i).pos_center =
                    self.panes.item_icon(usize::from(item)).pos_center_orig;
                self.panes.insert_child(P_IK0 + 20, P_CHK1 + i);
            }
            let center = self.panes.button_check(i).pos_center;
            self.panes.button_check_mut(i).pos_center_orig = center;
            self.panes.button_check_mut(i).cpos_move();
        }
    }

    pub fn itemplace_check(&self, kind: usize) -> bool {
        const GROUPS: [&[u8]; PLACE_GROUPS] = [
            &[
                SLOT_TELESCOPE,
                SLOT_SAIL,
                SLOT_WIND_WAKER,
                SLOT_GRAPPLING_HOOK,
            ],
            &[
                SLOT_TINGLE_TUNER,
                SLOT_CAMERA,
                SLOT_IRON_BOOTS,
                SLOT_MAGIC_ARMOR,
            ],
            &[SLOT_BOTTLE0, SLOT_BOTTLE1, SLOT_BOTTLE2, SLOT_BOTTLE3],
            &[SLOT_SPOILS_BAG, SLOT_BAIT_BAG, SLOT_DELIVERY_BAG],
            &[SLOT_BOOMERANG, SLOT_BOW, SLOT_HOOKSHOT],
            &[SLOT_DEKU_LEAF, SLOT_BOMB, SLOT_SKULL_HAMMER],
        ];
        match GROUPS.get(kind) {
            Some(slots) => slots.iter().any(|slot| self.slot_item(*slot) != ITEM_NONE),
            None => false,
        }
    }

    fn flush_selection(&mut self, out: &mut MenuOutput) {
        if self.select_item_btn == BTN_NONE {
            return;
        }
        let button = self.select_item_btn;
        let slot = self.select_item_slot;
        out.select_item_writes.push((button, slot));
        self.record.set_select(button, slot);
        self.play_select = self.record.select_item;
        self.select_item_btn = BTN_NONE;
        if self.swap_btn != BTN_NONE {
            let swap = self.swap_btn;
            let target = self.swap_slot;
            out.select_item_writes.push((swap, target));
            self.record.set_select(swap, target);
            self.play_select = self.record.select_item;
            self.swap_btn = BTN_NONE;
        }
        out.sounds.push(SE_MENU_SET);
    }

    fn other_buttons(&self) -> (u8, u8) {
        match self.select_item_btn {
            BTN_X => (BTN_Y, BTN_Z),
            BTN_Y => (BTN_Z, BTN_X),
            BTN_Z => (BTN_X, BTN_Y),
            _ => (BTN_NONE, BTN_NONE),
        }
    }

    fn read_button(&mut self, pad: &PadState) {
        if pad.x {
            self.select_item_btn = BTN_X;
        } else if pad.y {
            self.select_item_btn = BTN_Y;
        } else if pad.z {
            self.select_item_btn = BTN_Z;
        }
    }

    fn item_decide(&mut self, pad: &PadState, bridge: &mut ItemMoveBridge, out: &mut MenuOutput) {
        self.flush_selection(out);
        self.read_button(pad);
        let item = self.slot_item(self.now_item);
        if item == ITEM_SPOILS_BAG || item == ITEM_BAIT_BAG || item == ITEM_DELIVERY_BAG {
            self.sub_menu_kind = match item {
                ITEM_SPOILS_BAG => 0,
                ITEM_BAIT_BAG => 1,
                _ => 2,
            };
            self.sub_window_init();
            self.select_item_btn = BTN_NONE;
            self.swap_btn = BTN_NONE;
            out.sounds.push(SE_MENU_DECIDE);
            return;
        }
        let (first, second) = self.other_buttons();
        if self.now_item == self.record.select(first) {
            self.swap_btn = first;
            self.swap_slot = self.record.select(self.select_item_btn);
            if self.sub_win {
                self.sub_window_delete();
            }
        } else if self.now_item == self.record.select(second) {
            self.swap_btn = second;
            self.swap_slot = self.record.select(self.select_item_btn);
            if self.sub_win {
                self.sub_window_delete();
            }
        }
        self.select_item_slot = self.now_item;
        let pane = self.panes.item_icon(usize::from(self.select_item_slot)).clone();
        *bridge = ItemMoveBridge {
            flag: 1,
            button: self.select_item_btn,
            slot: self.select_item_slot,
            from: pane.pos_top_left,
            from_center: pane.pos_center,
            from_size: pane.size,
            from_size_orig: pane.size_orig,
            timer: 0,
        };
        out.sounds.push(SE_MENU_DECIDE);
    }

    fn sub_item_decide(
        &mut self,
        pad: &PadState,
        bridge: &mut ItemMoveBridge,
        out: &mut MenuOutput,
    ) {
        self.flush_selection(out);
        self.read_button(pad);
        let (first, second) = self.other_buttons();
        if self.now_item == self.record.select(first) {
            self.swap_btn = first;
            self.swap_slot = self.record.select(self.select_item_btn);
        } else if self.now_item == self.record.select(second) {
            self.swap_btn = second;
            self.swap_slot = self.record.select(self.select_item_btn);
        }
        self.select_item_slot = self.now_item;
        let index = usize::from(self.select_item_slot.wrapping_sub(self.sub_menu_item_base));
        let pane = self.panes.sub_item(index).clone();
        *bridge = ItemMoveBridge {
            flag: 1,
            button: self.select_item_btn,
            slot: self.select_item_slot,
            from: pane.pos_top_left,
            from_center: pane.pos_center,
            from_size: pane.size,
            from_size_orig: pane.size_orig,
            timer: 0,
        };
        out.sounds.push(SE_MENU_DECIDE);
        if self.sub_win {
            self.sub_window_delete();
        }
    }

    fn item_move(&mut self, bridge: &ItemMoveBridge, out: &mut MenuOutput) {
        if bridge.flag == 2 {
            self.flush_selection(out);
        }
    }

    fn item_scale(&mut self, tuning: &MenuTuning) {
        if self.now_item == SLOT_LAST {
            for index in 0..SLOT_COUNT {
                self.panes.item_icon_mut(index).pane_scale_xy(1.0);
                self.panes.item_shadow_mut(index).pane_scale_xy(1.0);
            }
            for index in 0..SUB_ITEM_COUNT {
                self.panes.sub_item_mut(index).pane_scale_xy(1.0);
                self.panes.sub_shadow_mut(index).pane_scale_xy(1.0);
            }
            return;
        }
        let focus = if self.now_item >= SLOT_RESERVE_FIRST {
            SLOT_RESERVE_FIRST
        } else if self.now_item >= SLOT_BAIT_FIRST {
            SLOT_BAIT_LAST
        } else if self.now_item >= SLOT_BEAST_FIRST {
            SLOT_BAIT_FIRST
        } else {
            self.now_item
        };
        let scale = tuning.item_scale;
        for index in 0..SLOT_COUNT {
            if index as u8 == focus && self.select_item_btn == BTN_NONE {
                self.panes.item_icon_mut(index).pane_scale_xy(scale);
                self.panes.item_shadow_mut(index).pane_scale_xy(scale);
                if focus == SLOT_BOW {
                    let icon = self.panes.item_icon(index).clone();
                    for tint in 0..3 {
                        let tint_pane = self.panes.arrow_tint(tint).clone();
                        self.panes.arrow_tint_mut(tint).pos_center.x = icon.pos_center.x
                            + scale * (tint_pane.pos_center_orig.x - icon.pos_center_orig.x);
                        self.panes.arrow_tint_mut(tint).pos_center.y = icon.pos_center.y
                            + scale * (tint_pane.pos_center_orig.y - icon.pos_center_orig.y);
                    }
                }
            } else {
                self.panes.item_icon_mut(index).pane_scale_xy(1.0);
                self.panes.item_shadow_mut(index).pane_scale_xy(1.0);
            }
        }
        if self.now_item == SLOT_NONE {
            for index in 0..SUB_ITEM_COUNT {
                self.panes.sub_item_mut(index).pane_scale_xy(1.0);
                self.panes.sub_shadow_mut(index).pane_scale_xy(1.0);
            }
        } else if self.now_item >= SLOT_LAST {
            let focus = self.now_item.wrapping_sub(self.sub_menu_item_base);
            for index in 0..SUB_ITEM_COUNT {
                if index as u8 == focus && self.select_item_btn == BTN_NONE {
                    self.panes.sub_item_mut(index).pane_scale_xy(scale);
                    self.panes.sub_shadow_mut(index).pane_scale_xy(scale);
                } else {
                    self.panes.sub_item_mut(index).pane_scale_xy(1.0);
                    self.panes.sub_shadow_mut(index).pane_scale_xy(1.0);
                }
            }
        }
    }

    fn sub_window_init(&mut self) {
        self.sub_win = true;
        self.panes.out_box_mut(0).user_area = 0;
        match self.sub_menu_kind {
            0 => {
                self.sub_menu_item_base = SLOT_BEAST_FIRST;
                self.now_item = self.sub_menu_item_base;
                for index in 0..SUB_ITEM_COUNT {
                    if self.record.beast[index] != ITEM_NONE {
                        self.panes.sub_item_mut(index).user_area = 1;
                        let texture = item_texture(self.record.beast[index]);
                        self.sub_icons[index] = Some(texture);
                        self.panes.sub_item_mut(index).change_texture(texture);
                        self.panes.sub_shadow_mut(index).change_texture(texture);
                        self.panes.sub_item_mut(index).show();
                        self.panes.sub_shadow_mut(index).show();
                        self.panes.sub_tens_mut(index).show();
                        self.panes.sub_ones_mut(index).show();
                    } else {
                        self.panes.sub_item_mut(index).user_area = 0;
                        self.panes.sub_item_mut(index).hide();
                        self.panes.sub_shadow_mut(index).hide();
                        self.panes.sub_tens_mut(index).hide();
                        self.panes.sub_ones_mut(index).hide();
                    }
                    let num = self.record.beast_num[beast_index(index)];
                    let color = if num == 0 {
                        2
                    } else if num == 99 {
                        1
                    } else {
                        0
                    };
                    self.number_color(P_SIN10 + index, color);
                    self.number_color(P_SIN01 + index, color);
                    if num < 10 {
                        self.panes.sub_tens_mut(index).hide();
                    } else {
                        self.number_set(P_SIN10 + index, num / 10);
                    }
                    self.number_set(P_SIN01 + index, num % 10);
                }
            }
            1 => {
                self.sub_menu_item_base = SLOT_BAIT_FIRST;
                self.now_item = self.sub_menu_item_base.wrapping_add(3);
                for index in 0..SUB_ITEM_COUNT {
                    self.panes.sub_tens_mut(index).hide();
                    if self.record.bait[index] != ITEM_NONE {
                        self.panes.sub_item_mut(index).user_area = 1;
                        let texture = item_texture(self.record.bait[index]);
                        self.sub_icons[index] = Some(texture);
                        self.panes.sub_item_mut(index).change_texture(texture);
                        self.panes.sub_shadow_mut(index).change_texture(texture);
                        self.panes.sub_item_mut(index).show();
                        self.panes.sub_shadow_mut(index).show();
                        if self.record.bait[index] == ITEM_BIRD_BAIT_5 {
                            self.panes.sub_ones_mut(index).show();
                        } else {
                            self.panes.sub_ones_mut(index).hide();
                        }
                    } else {
                        self.panes.sub_item_mut(index).user_area = 0;
                        self.panes.sub_item_mut(index).hide();
                        self.panes.sub_shadow_mut(index).hide();
                        self.panes.sub_ones_mut(index).hide();
                    }
                    let num = self.record.bait_num[index];
                    let color = if num == 0 {
                        2
                    } else if num == 3 {
                        1
                    } else {
                        0
                    };
                    self.number_color(P_SIN10 + index, color);
                    self.number_color(P_SIN01 + index, color);
                    self.number_set(P_SIN01 + index, num % 10);
                }
            }
            _ => {
                self.sub_menu_item_base = SLOT_RESERVE_FIRST;
                self.now_item = self.sub_menu_item_base.wrapping_add(6);
                for index in 0..SUB_ITEM_COUNT {
                    self.panes.sub_tens_mut(index).hide();
                    self.panes.sub_ones_mut(index).hide();
                    if self.record.reserve[index] != ITEM_NONE {
                        self.panes.sub_item_mut(index).user_area = 1;
                        let texture = item_texture(self.record.reserve[index]);
                        self.sub_icons[index] = Some(texture);
                        self.panes.sub_item_mut(index).change_texture(texture);
                        self.panes.sub_shadow_mut(index).change_texture(texture);
                        self.panes.sub_item_mut(index).show();
                        self.panes.sub_shadow_mut(index).show();
                    } else {
                        self.panes.sub_item_mut(index).user_area = 0;
                        self.panes.sub_item_mut(index).hide();
                        self.panes.sub_shadow_mut(index).hide();
                    }
                }
            }
        }
        self.name_pending = true;
        for index in 0..SUB_ITEM_COUNT {
            self.panes.sub_item_mut(index).pane_scale_xy(1.0);
            self.panes.sub_shadow_mut(index).pane_scale_xy(1.0);
        }
        for index in 0..SUB_ITEM_COUNT {
            self.panes.sub_ones_mut(index).set_init_alpha();
            self.panes.sub_tens_mut(index).set_init_alpha();
            self.panes.sub_item_mut(index).set_init_alpha();
            self.panes.sub_shadow_mut(index).set_init_alpha();
        }
        self.panes.pane_mut(P_IT29).set_init_alpha();
        self.panes.pane_mut(P_IK29).set_init_alpha();
        for index in 0..SUB_BUFFER_COUNT {
            self.panes.sub_frame_mut(index).set_init_alpha();
        }
        self.panes.pane_mut(P_BLAK).set_init_alpha();
        self.stack_cursor(P_SIN10 + 7);
    }

    fn sub_window_delete(&mut self) {
        self.panes.out_box_mut(0).user_area = 0;
        self.now_item = match self.sub_menu_kind {
            0 => item_num(ITEM_SPOILS_BAG),
            1 => item_num(ITEM_BAIT_BAG),
            _ => item_num(ITEM_DELIVERY_BAG),
        };
        self.name_pending = true;
        self.sub_win = false;
        for index in 0..SUB_ITEM_COUNT {
            self.panes.sub_ones_mut(index).set_now_alpha_zero();
            self.panes.sub_tens_mut(index).set_now_alpha_zero();
            self.panes.sub_item_mut(index).set_now_alpha_zero();
            self.panes.sub_shadow_mut(index).set_now_alpha_zero();
        }
        self.panes.pane_mut(P_IT29).set_now_alpha_zero();
        self.panes.pane_mut(P_IK29).set_now_alpha_zero();
        for index in 0..SUB_BUFFER_COUNT {
            self.panes.sub_frame_mut(index).set_now_alpha_zero();
        }
        self.panes.pane_mut(P_BLAK).set_now_alpha_zero();
        self.stack_cursor(P_BLAK);
    }

    fn item_name_move(&mut self) {
        let mut user = self.panes.out_box(0).user_area.wrapping_add(1);
        if user > ITEM_TUNING_NAME_PERIOD {
            user = ITEM_TUNING_NAME_RESET;
        }
        self.panes.out_box_mut(0).user_area = user;
        if user <= 10 {
            let alpha = value_increase(ITEM_TUNING_NAME_FADE, i32::from(user), 0);
            self.panes.name_box_mut(0).set_now_alpha(alpha);
            self.panes.name_box_mut(1).set_now_alpha(1.0 - alpha);
            self.panes.out_box_mut(0).set_now_alpha_zero();
        } else if user <= ITEM_TUNING_NAME_HOLD {
            self.panes.name_box_mut(0).set_init_alpha();
            self.panes.out_box_mut(0).set_now_alpha_zero();
        } else if user <= ITEM_TUNING_NAME_MID {
            let alpha = value_increase(
                ITEM_TUNING_NAME_FADE,
                i32::from(user - ITEM_TUNING_NAME_HOLD),
                0,
            );
            self.panes.name_box_mut(0).set_now_alpha(1.0 - alpha);
            self.panes.out_box_mut(0).set_now_alpha(alpha);
        } else if user <= ITEM_TUNING_NAME_OUT {
            self.panes.name_box_mut(0).set_now_alpha_zero();
            self.panes.out_box_mut(0).set_init_alpha();
        } else if user <= ITEM_TUNING_NAME_PERIOD {
            let alpha = value_increase(
                ITEM_TUNING_NAME_FADE,
                i32::from(user - ITEM_TUNING_NAME_OUT),
                0,
            );
            self.panes.name_box_mut(0).set_now_alpha(alpha);
            self.panes.out_box_mut(0).set_now_alpha(1.0 - alpha);
        }
    }

    fn item_name_set(
        &mut self,
        font: &mut FontBank,
        archive: &dyn MsgArchive,
        clear_count: u8,
    ) {
        let mut get = ItemMsgGet::default();
        let mut msg_no = 0u32;
        self.panes.out_box_mut(0).set_font_size(NAME_CELL_SCALE, NAME_CELL_SCALE);
        let size = self.panes.name_box(0).get_font_size();
        self.panes.name_box_mut(1).set_font_size(size.0, size.1);
        self.panes
            .name_box_mut(1)
            .set_char_space(self.panes.name_box(0).get_char_space());

        self.name_buf[1] = self.name_buf[0].clone();
        let carried = self.name_buf[0].clone();
        self.panes.name_box_mut(1).set_string(&carried);
        self.name_buf[0].clear();

        if self.now_item == SLOT_LAST {
            msg_no = MSG_ITEM_LAST_SLOT;
        } else {
            let item = self.slot_item(self.now_item);
            if item == ITEM_NONE {
                return;
            }
            msg_no = item_message(item) as u32;
            if msg_no == 0 {
                msg_no = MSG_NO_ITEM_NAME;
            }
        }

        let Some(head) = get.get_mesg_header(archive, clear_count, msg_no) else {
            return;
        };
        let Some(message) = get.get_message(head) else {
            return;
        };
        let message = message.to_vec();

        let (mut size_x, size_y) = self.panes.name_box(0).get_font_size();
        size_x = size_y;
        let Some(font_ref) = font.get(0) else {
            return;
        };
        let scale = size_y / font_ref.cell_width() as f32;
        let mut str_width = 0.0f32;
        let mut first = false;
        let mut index = 0usize;
        let mut raw = Vec::new();
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
        let box_width = self.panes.name_box(0).width();
        if box_width < str_width {
            size_x = size_x * (box_width / str_width);
        }
        self.panes.name_box_mut(0).set_font_size(size_x, size_y);
        self.panes.name_box_mut(0).set_char_space(0.0);
        let name = self.name_buf[0].clone();
        self.panes.name_box_mut(0).set_string(&name);

        self.out_font.ruby_msg = if self.now_item == SLOT_LAST {
            MSG_LAST_SLOT_RUBY
        } else if self.now_item == SLOT_SPOILS_BAG
            || self.now_item == SLOT_BAIT_BAG
            || self.now_item == SLOT_DELIVERY_BAG
        {
            MSG_BAG_RUBY
        } else {
            MSG_ITEM_RUBY
        };
        let msg = self.out_font.ruby_msg;
        let mut panes = std::mem::take(&mut self.panes);
        self.out_font.message_set(&mut panes, font, archive, clear_count, msg);
        self.panes = panes;
        let (x, y) = (
            self.panes.out_box(0).pos_top_left.x,
            self.panes.out_box(0).pos_top_left.y,
        );
        self.out_font.set_left_up_pos(x, y);
    }

    fn item_note_set(
        &mut self,
        font: &mut FontBank,
        archive: &dyn MsgArchive,
        clear_count: u8,
    ) {
        let mut get = ItemMsgGet::default();
        self.note_buf = [String::new(), String::new()];
        self.dummy_buf = [String::new(), String::new()];
        self.out_font_init();

        if self.opt_ruby {
            self.panes.pane_mut(P_STR0).hide();
        } else {
            self.panes.pane_mut(P_STR0).show();
        }
        let ruby_size = self.panes.pane(P_STR0).font_size_x;
        self.panes
            .pane_mut(P_ST00)
            .set_font_size(self.msg_font_size as f32, self.msg_font_size as f32);
        self.panes
            .pane_mut(P_ST00)
            .set_line_space(self.msg_line_space as f32);

        let item = self.slot_item(self.now_item);
        if item == ITEM_NONE {
            return;
        }
        let base = item_message(item) as u32;
        let msg_no = if base == 0 {
            MSG_NO_ITEM_NOTE
        } else {
            base + NOTE_MESSAGE_BASE
        };

        let Some(head) = get.get_mesg_header(archive, clear_count, msg_no) else {
            return;
        };
        let Some(message) = get.get_message(head) else {
            return;
        };
        let message = message.to_vec();
        let entry: MesgEntry = get.get_mesg_entry(head).unwrap_or_default();

        self.note = MsgData::new_data_proc();
        self.note.set_bmg(message);
        self.note.entry = entry;
        self.note.out = [
            Some(String::new()),
            Some(String::new()),
            Some(String::new()),
            Some(String::new()),
        ];
        self.note.font = 0;
        self.note.ruby_font = 1;
        self.note.char_space = self.panes.pane(P_ST00).get_char_space() as i32;
        self.note.ruby_char_space = self.panes.pane(P_STR0).get_char_space() as i32;
        self.note.line_space = self.panes.pane(P_ST00).get_line_space() as i32;
        self.note.font_size = self.msg_font_size;
        self.note.ruby_font_size = ruby_size as i32;
        self.note.line_width = NOTE_LINE_WIDTH;
        self.note.center_line_width = NOTE_CENTER_LINE_WIDTH;
        self.note.send_speed = NOTE_SEND_SPEED;
        self.note.space_timer = NOTE_SPACE_TIMER;
        self.note.short_cut();
        self.note.space_flag = 0;
        let state = MsgLenState {
            bomb_max: i32::from(self.record.bomb_max),
            arrow_max: i32::from(self.record.arrow_max),
            item_timer: i32::from(self.record.item_timer),
            clear_count,
            ..Default::default()
        };
        self.note.string_length(font, state);
        self.note.string_shift();
        self.note.icon_idx_refresh();

        let line_count = self.note.line_count;
        self.note.line_count = 0;
        let line_space = self.panes.pane(P_ST00).get_line_space();
        let unused = NOTE_UNUSED_LINES_USA - line_count;
        let shift_y = unused as f32 * (line_space / 2.0);
        self.panes.pane_mut(P_STR0).shift_set(0.0, shift_y);
        self.panes.pane_mut(P_ST00).shift_set(0.0, shift_y);
        self.note.string_set(font, &MsgLayoutState::default());
        let note = self.note.out[0].clone().unwrap_or_default();
        let ruby = self.note.out[1].clone().unwrap_or_default();
        self.note_buf = [note.clone(), note];
        self.panes.pane_mut(P_ST00).set_string(&self.note_buf[0]);
        self.panes.pane_mut(P_STR0).set_string(&self.note_buf[1]);

        let half_space = self.panes.pane(P_ST00).get_line_space() / 2.0;
        for index in 0..NOTE_SLOTS {
            let icon_no = self.note.icon_num[index];
            let mut color = self.note.icon_color[index];
            if color == 0xFFFF_FFFF {
                color = 0xFF;
            }
            if icon_no == ICON_NONE {
                continue;
            }
            if self.panes.note_icon(index).user_area != -1 {
                continue;
            }
            if icon_no == ICON_INPUT {
                continue;
            }
            self.panes.note_icon_mut(index).pos_top_left.x = self.note.icon_pos_x[index] as f32;
            self.panes.note_icon_mut(index).pos_top_left.y =
                half_space * (unused as f32 + self.note.icon_pos_y[index] as f32 * 2.0);
            self.panes.note_icon_mut(index).pos_top_left_orig.y = f32::from(icon_no);
            let icon = icon_palette(icon_no);
            let texture = icon_texture(icon);
            self.panes.note_icon_mut(index).show();
            self.panes.note_icon_mut(index).blend_draw(texture);
            self.panes.note_shadow_mut(index).show();
            self.panes.note_shadow_mut(index).blend_draw(texture);
            self.panes.note_shadow_mut(index).set_white(0x0000_00FF);
            self.panes.note_shadow_mut(index).set_black(0x0000_0000);
            if matches!(
                icon,
                ICON_ARROW_LEFT | ICON_ARROW_RIGHT | ICON_ARROW_UP | ICON_ARROW_DOWN | 0x15 | 0x17
            ) {
                let raw = unpack_color(color);
                self.panes
                    .note_icon_mut(index)
                    .set_white(pack_color([raw[0], raw[1], raw[2], 0xFF]));
                self.panes.note_icon_mut(index).set_black(0x0000_0000);
            } else {
                self.panes.note_icon_mut(index).set_white(icon_color(icon));
                self.panes.note_icon_mut(index).set_black(0x0000_0000);
            }
            self.panes.note_icon_mut(index).set_blend([0.0, 1.0, 1.0, 1.0]);
            self.panes
                .note_shadow_mut(index)
                .set_blend([0.0, 1.0, 1.0, 1.0]);
            self.panes.note_icon_mut(index).user_area = 0;
        }
    }

    fn out_font_init(&mut self) {
        for index in 0..NOTE_SLOTS {
            self.panes.note_icon_mut(index).user_area = -1;
            self.panes.note_icon_mut(index).rotate_deg(0.0, 0.0, 0.0);
            self.panes.note_icon_mut(index).set_now_alpha_zero();
        }
    }

    fn out_font_move(&mut self) {
        for index in 0..NOTE_SLOTS {
            if self.panes.note_icon(index).user_area == -1 {
                self.panes.note_icon_mut(index).set_now_alpha_zero();
                continue;
            }
            let note = self.panes.note_icon(index).pos_top_left;
            let box_pos = self.panes.pane(P_ST00).pos_top_left;
            let anchor = self.panes.pane(P_NO11).pos_top_left;
            let frame = self.panes.pane(P_NT00).pos_top_left;
            let x = note.x + (box_pos.x + (anchor.x + frame.x));
            let y = note.y + (box_pos.y + (anchor.y + frame.y));
            self.panes.note_shadow_mut(index).pos_top_left.x = x;
            self.panes.note_shadow_mut(index).pos_top_left.y = y;
            let cx = self.panes.pane(P_NO11).pos_center.x;
            let cy = self.panes.pane(P_NO11).pos_center.y;
            let angle = f32::from(self.panes.pane(P_NO11).rotation as i16);
            self.panes
                .note_shadow_mut(index)
                .rotate(cx - x, cy - y, angle);
        }
    }

    fn out_font_draw(&mut self) {
        for index in 0..NOTE_SLOTS {
            if self.panes.note_icon(index).user_area == -1 {
                continue;
            }
            let alpha = self.panes.pane(P_ST00).now_alpha;
            self.panes.note_icon_mut(index).now_alpha = alpha;
            let mut size_x = MSG_ICON_SCALE;
            let mut size_y = MSG_ICON_SCALE;
            let mut x = self.panes.note_shadow(index).pos_top_left.x as i32;
            let mut y = self.panes.note_shadow(index).pos_top_left.y as i32;
            let mut timer = self.panes.note_icon(index).user_area;
            let icon_no = self.panes.note_icon(index).pos_top_left_orig.y as u8;
            match icon_palette(icon_no) {
                ICON_MAIN_STICK => out_font_stick_draw(
                    self.panes.note_icon_mut(index),
                    &mut x,
                    &mut y,
                    &mut size_x,
                    &mut size_y,
                    &mut timer,
                ),
                ICON_MAIN_STICK_UP | ICON_MAIN_STICK_DOWN | ICON_MAIN_STICK_LEFT
                | ICON_MAIN_STICK_RIGHT => out_font_stick_dir_draw(
                    self.panes.note_icon_mut(index),
                    &mut x,
                    &mut y,
                    &mut size_x,
                    &mut size_y,
                    &mut timer,
                    icon_no,
                ),
                ICON_MAIN_STICK_UP_DOWN | ICON_MAIN_STICK_LEFT_RIGHT => out_font_stick_cycle_draw(
                    self.panes.note_icon_mut(index),
                    &mut x,
                    &mut y,
                    &mut size_x,
                    &mut size_y,
                    &mut timer,
                ),
                ICON_ARROW_LEFT | ICON_ARROW_RIGHT | ICON_ARROW_UP | ICON_ARROW_DOWN => {
                    out_font_arrow_draw(
                        self.panes.note_icon_mut(index),
                        &mut x,
                        &mut y,
                        &mut size_x,
                        &mut size_y,
                        icon_no,
                    );
                }
                _ => {}
            }
            self.panes.note_icon_mut(index).user_area = timer;
            self.panes.note_shadow_mut(index).pos_top_left.x = x as f32 + 2.0;
            self.panes.note_shadow_mut(index).pos_top_left.y = y as f32 + 1.0;
            self.panes.note_icon_mut(index).pos_top_left.x = x as f32;
            self.panes.note_icon_mut(index).pos_top_left.y = y as f32;
            self.panes.note_icon_mut(index).size_orig = F2 {
                x: size_x as f32,
                y: size_y as f32,
            };
            self.panes.note_icon_mut(index).size = self.panes.note_icon(index).size_orig;
            self.panes.note_icon_mut(index).cpos_move();
            self.panes.note_icon_mut(index).now_alpha = alpha;
        }
    }

    pub fn note_check(&self) -> bool {
        self.panes.pane(P_NK00).user_area == 1
    }

    fn note_init(&mut self) {
        self.panes.pane_mut(P_NK00).user_area = 0;
        self.panes.pane_mut(P_NT00).user_area = 0;
        let size = self.panes.pane(P_NO11).size_orig;
        let angle = f32::from(self.panes.pane(P_NO11).user_area);
        self.panes
            .pane_mut(P_NO11)
            .rotate(size.x / 2.0, size.y / 2.0, angle);
        self.panes.pane_mut(P_NO11).pane_trans(0.0, 0.0);
    }

    fn note_appear(&mut self, pad: &PadState, out: &mut MenuOutput) {
        let span = self.tuning.note_open_len + self.tuning.note_close_len;
        if self.panes.pane(P_NK00).user_area != 1 {
            return;
        }
        let user = self.panes.pane(P_NT00).user_area;
        if user <= span {
            self.note_open(out);
        } else if user > span + 1 {
            self.note_close();
        } else {
            let alpha = self.tuning.mask_alpha;
            self.panes.pane_mut(P_MSK0).init_alpha = alpha;
            self.panes.pane_mut(P_MSK0).now_alpha = alpha;
            if pad.a || pad.b {
                self.panes.pane_mut(P_NT00).user_area += 1;
                out.sounds.push(SE_MENU_EXP_OUT);
            }
        }
    }

    fn note_open(&mut self, out: &mut MenuOutput) {
        let from = self.tuning.note_rotate_from;
        let span = self.tuning.note_open_len + self.tuning.note_close_len;
        let dx = f32::from(self.tuning.screen_x) - self.panes.pane(P_NO11).pos_center_orig.x;
        let dy = f32::from(self.tuning.screen_y) - self.panes.pane(P_NO11).pos_center_orig.y;
        let user = self.panes.pane(P_NT00).user_area;
        let alpha = 1.0 - value_increase(i32::from(span), i32::from(span - user), 0);
        if user >= span {
            self.panes.pane_mut(P_MSK0).set_init_alpha();
            self.panes.pane_mut(P_STR0).set_init_alpha();
            self.panes.pane_mut(P_ST00).set_init_alpha();
            out.sounds.push(SE_MENU_EXP_IN);
        } else {
            self.panes.pane_mut(P_MSK0).set_now_alpha(alpha);
            self.panes.pane_mut(P_STR0).set_now_alpha(alpha);
            self.panes.pane_mut(P_ST00).set_now_alpha(alpha);
        }
        let base = f32::from(self.panes.pane(P_NO11).user_area);
        let size = self.panes.pane(P_NO11).size;
        let angle = base + alpha * (f32::from(from) - base);
        self.panes
            .pane_mut(P_NO11)
            .rotate(size.x / 2.0, size.y / 2.0, angle);
        let grow = if user < self.tuning.note_open_len {
            user as f32 * self.tuning.scale / self.tuning.note_open_len as f32
        } else {
            let rest = value_increase(
                i32::from(self.tuning.note_close_len),
                i32::from(span - from),
                0,
            );
            (1.0 - rest) * (1.0 - self.tuning.scale) + self.tuning.scale
        };
        self.panes.pane_mut(P_NO11).pane_trans(dx * grow, dy * grow);
        self.panes.pane_mut(P_NT00).user_area += 1;
    }

    fn note_close(&mut self) {
        let from = self.tuning.note_rotate_from;
        let end = self.tuning.note_close_angle;
        let span = self.tuning.note_open_len + self.tuning.note_close_len;
        let frames = self.tuning.note_close_frames;
        let screen_x = f32::from(self.tuning.screen_x);
        let screen_y = f32::from(self.tuning.screen_y);
        let center = self.panes.pane(P_NO11).pos_center_orig;
        let out_x = (screen_x * 2.0 - center.x) - screen_x;
        let out_y = (screen_y * 2.0 - center.y) - screen_y;
        let home_x = screen_x - center.x;
        let home_y = screen_y - center.y;
        let user = self.panes.pane(P_NT00).user_area;
        let alpha = value_increase(i32::from(frames), i32::from(user - (span + 1)), 0);
        let angle = alpha * (f32::from(end) - f32::from(from)) + f32::from(from);
        self.panes
            .pane_mut(P_NO11)
            .pane_trans(out_x * alpha + home_x, out_y * alpha + home_y);
        let size = self.panes.pane(P_NO11).size;
        self.panes
            .pane_mut(P_NO11)
            .rotate(size.x / 2.0, size.y / 2.0, angle);
        self.panes.pane_mut(P_NT00).user_area += 1;
        let limit = span + frames + 1;
        if self.panes.pane(P_NT00).user_area > limit {
            self.panes.pane_mut(P_NT00).set_init_alpha();
            self.panes.pane_mut(P_NK00).set_init_alpha();
            self.panes.pane_mut(P_NO11).set_init_alpha();
            self.panes.pane_mut(P_MSK0).set_now_alpha_zero();
            self.panes.pane_mut(P_STR0).set_now_alpha_zero();
            self.panes.pane_mut(P_ST00).set_now_alpha_zero();
            self.panes.pane_mut(P_NK00).user_area = 0;
            self.panes.pane_mut(P_NT00).user_area = 0;
            let size = self.panes.pane(P_NO11).size;
            let angle = f32::from(self.panes.pane(P_NO11).user_area);
            self.panes
                .pane_mut(P_NO11)
                .rotate(size.x / 2.0, size.y / 2.0, angle);
            self.panes.pane_mut(P_NO11).pane_trans(0.0, 0.0);
        } else {
            for index in [P_NT00, P_NK00, P_NO11, P_MSK0, P_STR0, P_ST00] {
                self.panes.pane_mut(index).set_now_alpha(1.0 - alpha);
            }
        }
    }

    fn main_trans(&mut self, x: f32, y: f32) {
        for index in MAIN_PANES {
            self.panes.pane_mut(index).pane_trans(x, y);
        }
        for index in 0..SLOT_COUNT {
            self.panes.item_icon_mut(index).pane_trans(x, y);
            self.panes.item_shadow_mut(index).pane_trans(x, y);
        }
        for index in 0..3 {
            self.panes.arrow_tint_mut(index).pane_trans(x, y);
        }
        for index in 0..PLACE_GROUPS {
            self.panes.place_mut(index).pane_trans(x, y);
        }
        for index in 0..CURSOR_PARTS {
            self.panes.corner_mut(index).pane_trans(x, y);
        }
        for index in 0..CURSOR_PARTS {
            self.panes.cursor_mut(index).pane_trans(x, y);
        }
        for index in 0..BTN_COUNT {
            self.panes.button_check_mut(index).pane_trans(x, y);
        }
    }

    fn title_trans(&mut self, x: f32, y: f32) {
        self.panes.pane_mut(P_TK00).pane_trans(x, y);
    }

    fn note_rotate(&mut self, y: f32, angle: f32) {
        self.panes.pane_mut(P_NO11).pane_trans(0.0, y);
        let size = self.panes.pane(P_NO11).size;
        self.panes
            .pane_mut(P_NO11)
            .rotate(size.x / 2.0, size.y / 2.0, angle);
    }

    fn name_trans(&mut self, x: f32, y: f32) {
        self.panes.pane_mut(P_WD).pane_trans(x, y);
        let (bx, by) = (
            self.panes.pane(P_WD).pos_top_left.x,
            self.panes.pane(P_WD).pos_top_left.y,
        );
        self.out_font.set_left_up_pos(bx, by);
        let mut panes = std::mem::take(&mut self.panes);
        self.out_font.move_out_font(&mut panes);
        self.panes = panes;
        self.panes.pane_mut(P_NM00).pane_trans(x, y);
        self.panes.pane_mut(P_NM01).pane_trans(x, y);
        self.panes.pane_mut(P_ITNM).pane_trans(x, y);
        self.panes.pane_mut(P_ITNK).pane_trans(x, y);
    }

    fn main_open_proc(&mut self, now: i16, end: i16, distance: f32) {
        if end < now {
            return;
        }
        let alpha = value_increase(i32::from(end), i32::from(now), 0);
        let slide = value_increase(i32::from(end), i32::from(end - now), 0);
        self.main_trans(distance * slide, 0.0);
        for index in MAIN_PANES {
            self.panes.pane_mut(index).set_now_alpha(alpha);
        }
        for index in 0..SLOT_COUNT {
            self.panes.item_icon_mut(index).set_now_alpha(alpha);
            self.panes.item_shadow_mut(index).set_now_alpha(alpha);
        }
        for index in 0..PLACE_GROUPS {
            if self.itemplace_check(index) {
                self.panes.place_mut(index).set_now_alpha(alpha);
            }
        }
        for index in 0..CURSOR_PARTS {
            self.panes.corner_mut(index).set_now_alpha(alpha);
            self.panes.cursor_mut(index).set_now_alpha(alpha);
        }
        for index in 0..3 {
            self.panes.arrow_tint_mut(index).set_now_alpha(alpha);
            self.panes.button_check_mut(index).set_now_alpha(alpha);
            let show = self.play_select[[BTN_X, BTN_Y, BTN_Z][index] as usize] != ITEM_NONE;
            if show {
                self.panes.button_check_mut(index).show();
            } else {
                self.panes.button_check_mut(index).hide();
            }
        }
    }

    fn title_open_proc(&mut self, now: i16, end: i16, out: &mut MenuOutput) {
        if end < now {
            return;
        }
        value_increase(i32::from(end), i32::from(now), 0);
        let y = f32::from(self.tuning.title_y)
            * value_increase(i32::from(end), i32::from(end - now), 0);
        self.title_trans(0.0, y);
        self.panes.pane_mut(P_TL00).set_init_alpha();
        self.panes.pane_mut(P_TK00).set_init_alpha();
        if now == 1 {
            out.sounds.push(SE_MENU_PANEL_IN);
        }
    }

    fn note_open_proc(&mut self, now: i16, end: i16) {
        if end < now {
            return;
        }
        value_increase(i32::from(end), i32::from(now), 0);
        let slide = value_increase(i32::from(end), i32::from(end - now), 0);
        let base = f32::from(self.panes.pane(P_NO11).user_area);
        let angle = base + (f32::from(self.tuning.note_angle) - base) * slide;
        self.note_rotate(f32::from(self.tuning.note_open_rot) * slide, angle);
        for _ in 0..2 {
            self.panes.pane_mut(P_NT00).set_init_alpha();
            self.panes.pane_mut(P_NK00).set_init_alpha();
        }
    }

    fn name_open_proc(&mut self, now: i16, end: i16) {
        if end < now {
            return;
        }
        value_increase(i32::from(end), i32::from(now), 0);
        let y = f32::from(self.tuning.name_y)
            * value_increase(i32::from(end), i32::from(end - now), 0);
        self.name_trans(0.0, y);
        self.panes.pane_mut(P_NM00).set_init_alpha();
        self.panes.pane_mut(P_ITNM).set_init_alpha();
        self.panes.pane_mut(P_ITNK).set_init_alpha();
    }

    fn number_set(&mut self, index: usize, mut value: u8) {
        if value >= 10 {
            value = 0;
        }
        let texture = NUMBER_TEX
            .get(usize::from(value))
            .copied()
            .unwrap_or("");
        self.panes.pane_mut(index).change_texture(texture);
    }

    fn number_color(&mut self, index: usize, mode: u8) {
        let pane = self.panes.pane_mut(index);
        match mode {
            1 => {
                pane.set_white(0xFFC8_32FF);
                pane.set_black(0x1E1E_1E00);
            }
            2 => {
                pane.set_white(0xB4B4_B4FF);
                pane.set_black(0x1E1E_1E00);
            }
            _ => {
                pane.set_white(0xFFFF_FFFF);
                pane.set_black(0x0000_0000);
            }
        }
    }

    fn item_number_set(&mut self) {
        let pictures = self.record.picture_num;
        self.number_set(P_0301, pictures);
        let color = if pictures == 0 {
            2
        } else if pictures == 3 {
            1
        } else {
            0
        };
        self.number_color(P_0301, color);
        self.picture_num = pictures;

        let arrows = self.record.arrow_num;
        let arrow_color = if arrows == 0 {
            2
        } else if arrows == self.record.arrow_max {
            1
        } else {
            0
        };
        self.number_color(P_1301, arrow_color);
        self.number_color(P_1310, arrow_color);
        if arrows < 10 {
            self.panes.pane_mut(P_1310).hide();
        } else {
            self.number_set(P_1310, arrows / 10);
        }
        self.number_set(P_1301, arrows % 10);
        self.arrow_num = arrows;

        let bombs = self.record.bomb_num;
        let bomb_color = if bombs == 0 {
            2
        } else if bombs == self.record.bomb_max {
            1
        } else {
            0
        };
        self.number_color(P_1401, bomb_color);
        self.number_color(P_1410, bomb_color);
        if bombs < 10 {
            self.panes.pane_mut(P_1410).hide();
        } else {
            self.number_set(P_1410, bombs / 10);
        }
        self.number_set(P_1401, bombs % 10);
        self.bomb_num = bombs;
    }

    fn item_check(&mut self, slot: usize) {
        let mut item = self.slot_item(slot as u8);
        if item != ITEM_NONE {
            if self.disabled(item) {
                self.panes.item_icon_mut(slot).hide();
                self.panes.item_shadow_mut(slot).hide();
            }
            if item == ITEM_MAGIC_ARROW {
                self.panes.arrow_tint_mut(0).show();
                self.panes.arrow_tint_mut(1).show();
                self.panes.arrow_tint_mut(2).hide();
                item = ITEM_BOW;
            } else if item == ITEM_LIGHT_ARROW {
                self.panes.arrow_tint_mut(0).show();
                self.panes.arrow_tint_mut(1).show();
                self.panes.arrow_tint_mut(2).show();
                item = ITEM_BOW;
            } else if item == ITEM_FOREST_WATER && !self.disabled(item) {
                let center = self.panes.item_icon(slot).pos_center;
                for emitter in self.emitters.iter_mut() {
                    if !emitter.spawned {
                        emitter.spawned = true;
                        emitter.resource = FOREST_WATER_RESOURCE;
                    }
                    emitter.x = center.x - FOREST_WATER_SCREEN_X;
                    emitter.y = center.y - FOREST_WATER_SCREEN_Y;
                }
            }
            let texture = item_texture(item);
            self.item_icons[slot] = Some(texture);
            self.panes.item_icon_mut(slot).change_texture(texture);
            self.panes.item_shadow_mut(slot).change_texture(texture);
            if slot == usize::from(SLOT_CAMERA) {
                let real = self.slot_item(SLOT_CAMERA);
                let show = real == ITEM_PICTO_BOX || real == ITEM_DELUXE_PICTO_BOX;
                if show {
                    self.panes.item_icon_mut(slot).show();
                    self.panes.item_shadow_mut(slot).show();
                    self.panes.pane_mut(P_0301).show();
                } else {
                    self.panes.item_icon_mut(slot).hide();
                    self.panes.item_shadow_mut(slot).hide();
                    self.panes.pane_mut(P_0301).hide();
                }
            }
            self.items[slot] = self.slot_item(slot as u8);
        } else {
            self.panes.item_icon_mut(slot).hide();
            self.panes.item_shadow_mut(slot).hide();
            if slot == usize::from(SLOT_CAMERA) {
                self.panes.pane_mut(P_0301).hide();
            } else if slot == usize::from(SLOT_BOW) {
                self.panes.pane_mut(P_1301).hide();
                self.panes.pane_mut(P_1310).hide();
            } else if slot == usize::from(SLOT_BOMB) {
                self.panes.pane_mut(P_1401).hide();
                self.panes.pane_mut(P_1410).hide();
            }
        }
    }

    fn item_bit_check(&mut self, forced: bool) {
        for index in 0..SLOT_COUNT {
            if self.items[index] != self.slot_item(index as u8) || forced {
                self.item_check(index);
            }
        }
    }

    fn arrow_light_anime(&mut self) {
        let mut user = self.panes.arrow_tint(0).user_area.wrapping_add(1);
        if user >= ITEM_TUNING_ARROW_PERIOD {
            user = 0;
        }
        self.panes.arrow_tint_mut(0).user_area = user;
        let phase = if user < ITEM_TUNING_ARROW_HALF {
            value_increase(i32::from(ITEM_TUNING_ARROW_HALF), i32::from(user), 0)
        } else {
            value_increase(
                i32::from(ITEM_TUNING_ARROW_HALF),
                i32::from(ITEM_TUNING_ARROW_PERIOD - user),
                0,
            )
        };
        let scale = 1.0 - phase * ITEM_TUNING_ARROW_SCALE;
        let init = f32::from(self.panes.arrow_tint(0).init_alpha);
        let alpha = init - (init - ITEM_TUNING_ARROW_MIN_ALPHA) * phase;
        for index in 0..3 {
            self.panes.arrow_tint_mut(index).pane_scale_xy(scale);
            self.panes.arrow_tint_mut(index).now_alpha = alpha as u8;
        }
    }

    pub fn bottle_fwater_check(&self) -> usize {
        for index in 0..4u8 {
            if self.slot_item(SLOT_BOTTLE0.wrapping_add(index)) == ITEM_FOREST_WATER {
                return usize::from(index);
            }
        }
        0
    }

    fn corner_move(&mut self) {
        let base = self.contents_color;
        let mut frame = self.frame_timer.wrapping_add(1);
        if frame >= ITEM_TUNING_CORNER_PERIOD {
            frame = 0;
        }
        self.frame_timer = frame;
        let blend = value_increase(
            i32::from(ITEM_TUNING_CORNER_HALF),
            i32::from(frame % ITEM_TUNING_CORNER_HALF),
            2,
        );
        let color = if frame < ITEM_TUNING_CORNER_HALF {
            ContentsColor {
                tl: mix(base.tl, base.bl, blend),
                tr: mix(base.tr, base.tl, blend),
                bl: mix(base.bl, base.br, blend),
                br: mix(base.br, base.tr, blend),
            }
        } else if frame < ITEM_TUNING_CORNER_HALF * 2 {
            ContentsColor {
                tr: mix(base.tl, base.bl, blend),
                br: mix(base.tr, base.tl, blend),
                tl: mix(base.bl, base.br, blend),
                bl: mix(base.br, base.tr, blend),
            }
        } else if frame < ITEM_TUNING_CORNER_HALF * 3 {
            ContentsColor {
                br: mix(base.tl, base.bl, blend),
                bl: mix(base.tr, base.tl, blend),
                tr: mix(base.bl, base.br, blend),
                tl: mix(base.br, base.tr, blend),
            }
        } else {
            ContentsColor {
                bl: mix(base.tl, base.bl, blend),
                tl: mix(base.tr, base.tl, blend),
                br: mix(base.bl, base.br, blend),
                tr: mix(base.br, base.tr, blend),
            }
        };
        self.panes.pane_mut(P_WK01).contents_color = color;

        let mut sheen = self.sheen_timer.wrapping_add(1);
        if sheen >= ITEM_TUNING_SHEEN_PERIOD {
            sheen = 0;
        }
        self.sheen_timer = sheen;
        let wave = if sheen < ITEM_TUNING_SHEEN_HALF {
            value_increase(i32::from(ITEM_TUNING_SHEEN_HALF), i32::from(sheen), 0)
        } else {
            value_increase(
                i32::from(ITEM_TUNING_SHEEN_HALF),
                i32::from(ITEM_TUNING_SHEEN_PERIOD - sheen),
                0,
            )
        };
        let green = (ITEM_TUNING_SHEEN_BASE - wave * ITEM_TUNING_SHEEN_RATE) as u8;
        let raw = pack_color([0xFF, green, 0x00, 0xFF]);
        self.panes.pane_mut(P_WK01).set_white(raw);
    }
}

pub const BEAST_INDEX: [usize; SUB_ITEM_COUNT] = [0, 1, 2, 3, 4, 5, 6, 7];

fn beast_index(slot: usize) -> usize {
    BEAST_INDEX.get(slot).copied().unwrap_or(0)
}

fn mix(a: [u8; 4], b: [u8; 4], t: f32) -> [u8; 4] {
    let mut out = [0u8; 4];
    for index in 0..4 {
        let from = f32::from(a[index]);
        let to = f32::from(b[index]);
        out[index] = (from + t * (to - from)) as u8;
    }
    out
}

fn out_font_stick_draw(
    pane: &mut Pane,
    x: &mut i32,
    y: &mut i32,
    width: &mut i32,
    height: &mut i32,
    timer: &mut i16,
) {
    let period = stick_period();
    if *timer < period {
        pane.set_blend(stick_blend(*timer, 0));
    } else if *timer < period * 2 {
        pane.set_blend(stick_blend(*timer, 1));
        *x += *width;
    } else if *timer < period * 3 {
        pane.set_blend(stick_blend(*timer, 2));
        *x += *width;
        *y += *height;
    } else if *timer < period * 4 {
        pane.set_blend(stick_blend(*timer, 3));
        *y += *width;
    }
    if *timer % period == 0 {
        pane.rotate_deg(0.0, 0.0, (*timer / period) as f32 * -90.0);
    }
    *timer += 1;
    if *timer >= period * 4 {
        *timer = 0;
    }
}

fn out_font_stick_dir_draw(
    pane: &mut Pane,
    x: &mut i32,
    y: &mut i32,
    width: &mut i32,
    height: &mut i32,
    timer: &mut i16,
    icon_no: u8,
) {
    let period = stick_period();
    if *timer < period {
        pane.set_blend(stick_blend(*timer, 0));
        match icon_no {
            ICON_MAIN_STICK_UP => {
                let swap = *width;
                *width = *height;
                *height = swap;
                *x += *height;
            }
            ICON_MAIN_STICK_DOWN => {
                let swap = *width;
                *width = *height;
                *height = swap;
                *y += *width;
            }
            _ => {}
        }
    }
    pane.rotate_deg(0.0, 0.0, icon_direction(icon_no) * -90.0);
    *timer += 1;
    if *timer >= period {
        *timer = 0;
    }
}

fn icon_direction(icon_no: u8) -> f32 {
    match icon_no {
        ICON_MAIN_STICK_UP => 1.0,
        ICON_MAIN_STICK_DOWN => 3.0,
        ICON_MAIN_STICK_LEFT => 0.0,
        _ => 2.0,
    }
}

fn out_font_stick_cycle_draw(
    pane: &mut Pane,
    x: &mut i32,
    y: &mut i32,
    width: &mut i32,
    height: &mut i32,
    timer: &mut i16,
) {
    let period = stick_period();
    let mut facing = 0i32;
    if *timer < period {
        pane.set_blend(stick_blend(*timer, 0));
        let swap = *width;
        *width = *height;
        *height = swap;
        *x += *height;
        facing = 1;
    } else if *timer < period * 2 {
        pane.set_blend(stick_blend(*timer, 1));
        let swap = *width;
        *width = *height;
        *height = swap;
        *y += *width;
        facing = 3;
    }
    pane.rotate_deg(0.0, 0.0, facing as f32 * -90.0);
    *timer += 1;
    if *timer >= period * 2 {
        *timer = 0;
    }
}

fn out_font_arrow_draw(
    pane: &mut Pane,
    x: &mut i32,
    y: &mut i32,
    width: &mut i32,
    height: &mut i32,
    icon_no: u8,
) {
    match icon_no {
        ICON_ARROW_LEFT => {
            let swap = *width;
            *width = *height;
            *height = swap;
            *x += *height;
            pane.rotate_deg(0.0, 0.0, 270.0);
        }
        ICON_ARROW_RIGHT => {
            let swap = *width;
            *width = *height;
            *height = swap;
            *y += *width;
            pane.rotate_deg(0.0, 0.0, 90.0);
        }
        ICON_ARROW_UP => {
            *x += *width;
            *y += *height;
            pane.rotate_deg(0.0, 0.0, 180.0);
        }
        _ => {}
    }
}

pub struct ItemQuad {
    pub tag: &'static str,
    pub kind: Kind,
    pub bounds: [f32; 4],
    pub alpha: u8,
    pub texture: Option<&'static str>,
    pub appended: Vec<&'static str>,
    pub blend: [f32; 4],
    pub white: u32,
    pub black: u32,
    pub char_color: u32,
    pub grad_color: u32,
    pub rotation: u16,
    pub visible: bool,
}

pub struct ItemText {
    pub tag: &'static str,
    pub text: String,
    pub bounds: [f32; 4],
    pub font_size: (f32, f32),
    pub char_space: f32,
    pub line_space: f32,
    pub alpha: u8,
}

pub struct ItemDraw {
    pub quads: Vec<ItemQuad>,
    pub texts: Vec<ItemText>,
    pub emitters: [EmitterState; 2],
    pub contents_color: ContentsColor,
    pub order: Vec<usize>,
}

impl ItemMenu {
    pub fn set_timer(&mut self, value: i16) {
        self.timer = value;
    }

    pub fn set_trigger_info(&mut self, info: u8) {
        self.trigger_info = info;
    }

    pub fn set_now_item(&mut self, item: u8) {
        self.now_item = item;
    }

    pub fn get_now_item(&self) -> u8 {
        self.now_item
    }

    pub fn get_item_mode(&self) -> u8 {
        self.item_mode
    }

    pub fn note_reset(&mut self) {
        for index in 0..NAME_BUFFER_COUNT {
            self.name_buf[index].clear();
            self.note_buf[index].clear();
            self.panes.name_box_mut(index).set_string("");
        }
        self.panes.pane_mut(P_ST00).set_string("");
        self.panes.pane_mut(P_STR0).set_string("");
    }

    pub fn open(
        &mut self,
        font: &mut FontBank,
        archive: &dyn MsgArchive,
        clear_count: u8,
        out: &mut MenuOutput,
    ) -> bool {
        self.open_common(font, archive, clear_count, ITEM_TUNING_FADE_FRAMES, out, true)
    }

    pub fn open_page(
        &mut self,
        tuning: &MenuTuning,
        font: &mut FontBank,
        archive: &dyn MsgArchive,
        clear_count: u8,
        out: &mut MenuOutput,
    ) -> bool {
        self.open_common(
            font,
            archive,
            clear_count,
            tuning.page_frames,
            out,
            false,
        )
    }

    fn open_common(
        &mut self,
        font: &mut FontBank,
        archive: &dyn MsgArchive,
        clear_count: u8,
        span: i16,
        out: &mut MenuOutput,
        chime: bool,
    ) -> bool {
        let distance = self.tuning.slide_x;
        if self.timer == 0 {
            self.note_reset();
            self.item_name_set(font, archive, clear_count);
            let item = self.slot_item(self.now_item);
            if self.now_item != SLOT_LAST && !self.disabled(item) {
                self.item_note_set(font, archive, clear_count);
                if item != ITEM_NONE {
                    out.do_status_force = Some(DO_STATUS_FORCE_ITEM);
                } else {
                    out.do_status_force = Some(0);
                    out.do_status = Some(0);
                }
            } else {
                out.do_status_force = Some(DO_STATUS_FORCE_LAST);
            }
        }
        self.timer += 1;
        if self.timer > 0 && self.timer <= span {
            let signed = if self.trigger_info == 2 {
                f32::from(distance)
            } else {
                -f32::from(distance)
            };
            self.main_open_proc(self.timer, span, signed);
            self.title_open_proc(self.timer, span, out);
            self.note_open_proc(self.timer, span);
            self.name_open_proc(self.timer, span);
        }
        let mut done = false;
        if self.timer >= span {
            done = true;
            if chime {
                out.sounds.push(SE_MENU_ITEMS_IN);
            }
        }
        done
    }

    pub fn close(&mut self, span: i16, out: &mut MenuOutput) -> bool {
        self.close_common(span, true, out)
    }

    pub fn close_page(&mut self, span: i16, out: &mut MenuOutput) -> bool {
        self.close_common(span, false, out)
    }

    fn close_common(&mut self, span: i16, fade_tints: bool, out: &mut MenuOutput) -> bool {
        self.timer -= 1;
        let slide = value_increase(i32::from(span), i32::from(span - self.timer), 0);
        let fade = value_increase(i32::from(span), i32::from(self.timer), 0);
        let main_x = f32::from(self.tuning.slide_x) * slide;
        let title_y = f32::from(self.tuning.title_y) * slide;
        let name_y = f32::from(self.tuning.name_y) * slide;
        let base = f32::from(self.panes.pane(P_NO11).user_area);
        let angle = base + (f32::from(self.tuning.note_angle) - base) * slide;
        let rotation = f32::from(self.tuning.note_open_rot) * slide;
        for emitter in self.emitters.iter_mut() {
            if emitter.spawned {
                emitter.alpha = 255.0 * fade;
            }
        }
        if self.trigger_info == 2 {
            self.main_trans(-main_x, 0.0);
            self.title_trans(0.0, title_y);
            self.note_rotate(rotation, angle);
            self.name_trans(0.0, name_y);
        } else if self.trigger_info == 1 {
            self.main_trans(main_x, 0.0);
            self.title_trans(0.0, title_y);
            self.note_rotate(rotation, angle);
            self.name_trans(0.0, name_y);
        }
        let mut done = false;
        if self.timer > 0 {
            self.fading = 1;
        } else {
            done = true;
            self.fading = 0;
            self.zero_alpha(fade_tints);
            self.flush_selection(out);
            if self.sub_win {
                self.sub_window_delete();
            }
        }
        done
    }

    fn zero_alpha(&mut self, fade_tints: bool) {
        for index in 0..NOTE_SLOTS {
            self.panes.note_icon_mut(index).set_now_alpha_zero();
        }
        for index in TEXT_PANES {
            self.panes.pane_mut(index).set_now_alpha_zero();
        }
        for index in 0..CURSOR_PARTS {
            self.panes.cursor_mut(index).set_now_alpha_zero();
        }
        for index in 0..SUB_ITEM_COUNT {
            self.panes.sub_ones_mut(index).set_now_alpha_zero();
            self.panes.sub_tens_mut(index).set_now_alpha_zero();
            self.panes.sub_item_mut(index).set_now_alpha_zero();
            self.panes.sub_shadow_mut(index).set_now_alpha_zero();
        }
        self.panes.pane_mut(P_IT29).set_now_alpha_zero();
        self.panes.pane_mut(P_IK29).set_now_alpha_zero();
        for index in 0..SUB_BUFFER_COUNT {
            self.panes.sub_frame_mut(index).set_now_alpha_zero();
        }
        for index in [
            P_BLAK,
            P_PANE_BG,
            P_WK01,
            P_WK00,
            P_0301,
            P_1301,
            P_1310,
            P_1401,
            P_1410,
        ] {
            self.panes.pane_mut(index).set_now_alpha_zero();
        }
        for index in 0..SLOT_COUNT {
            self.panes.item_icon_mut(index).set_now_alpha_zero();
            self.panes.item_shadow_mut(index).set_now_alpha_zero();
        }
        if fade_tints {
            for index in 0..3 {
                self.panes.arrow_tint_mut(index).set_now_alpha_zero();
            }
        }
        for index in 0..BTN_COUNT {
            self.panes.button_check_mut(index).set_now_alpha_zero();
        }
        for index in 0..PLACE_GROUPS {
            self.panes.place_mut(index).set_now_alpha_zero();
        }
        for index in 0..CURSOR_PARTS {
            self.panes.corner_mut(index).set_now_alpha_zero();
        }
    }

    pub fn step(
        &mut self,
        pad: &PadState,
        tuning: &MenuTuning,
        bridge: &mut ItemMoveBridge,
        font: &mut FontBank,
        archive: &dyn MsgArchive,
        clear_count: u8,
        out: &mut MenuOutput,
    ) {
        let previous = self.now_item;
        self.item_bit_check(false);
        if self.item_mode == 0 {
            if !self.note_check() {
                if !pad.start && !pad.lock_r && !pad.lock_l {
                    self.input(pad, tuning, bridge, previous, font, archive, clear_count, out);
                }
            } else {
                self.note_appear(pad, out);
            }
            if self.select_item_btn != BTN_NONE {
                self.item_move(bridge, out);
            }
            let item = self.slot_item(self.now_item);
            if self.now_item == SLOT_LAST || (item != ITEM_NONE && !self.disabled(item)) {
                self.item_name_move();
            } else {
                self.panes.pane_mut(P_NM00).set_now_alpha_zero();
                self.panes.pane_mut(P_NM01).set_now_alpha_zero();
                self.panes.pane_mut(P_WD).set_now_alpha_zero();
            }
            self.out_font_move();
            self.cursor_anime();
            self.corner_move();
            let bow = self.slot_item(SLOT_BOW);
            if bow == ITEM_MAGIC_ARROW || bow == ITEM_LIGHT_ARROW {
                self.arrow_light_anime();
            }
            let select = [
                self.record.select(BTN_X),
                self.record.select(BTN_Y),
                self.record.select(BTN_Z),
            ];
            self.check_move(&select);
            let bottle = self.bottle_fwater_check();
            if bottle != 0 {
                let center = self.panes.item_icon(bottle).pos_center_orig;
                for emitter in self.emitters.iter_mut() {
                    if emitter.spawned {
                        emitter.x = center.x - FOREST_WATER_SCREEN_X;
                        emitter.y = center.y - FOREST_WATER_SCREEN_Y;
                        emitter.playing = true;
                        emitter.alpha = 255.0;
                    }
                }
            }
        } else if self.item_mode == 1 {
            self.save_timer += 1;
            let span = match self.save_status {
                1 => SAVE_OPEN_FRAMES,
                2 => SAVE_MOVE_FRAMES,
                _ => SAVE_CLOSE_FRAMES,
            };
            if self.save_timer >= span {
                self.save_timer = 0;
                if self.save_status >= 3 {
                    self.save_status = 0;
                    self.item_mode = 0;
                    self.save_open = false;
                } else {
                    self.save_status += 1;
                }
            }
            if self.bottle_fwater_check() != 0 {
                for emitter in self.emitters.iter_mut() {
                    if emitter.spawned {
                        emitter.alpha = 0.0;
                    }
                }
            }
            out.a_status_force = Some(A_STATUS_FORCE_SAVE);
        }
        if self.name_pending {
            self.name_pending = false;
            self.item_name_set(font, archive, clear_count);
        }
        if self.now_item != SLOT_LAST {
            if self.slot_item(self.now_item) != ITEM_NONE {
                out.do_status_force = Some(DO_STATUS_FORCE_ITEM);
            } else {
                out.do_status_force = Some(0);
                out.do_status = Some(0);
            }
        } else {
            out.do_status_force = Some(DO_STATUS_FORCE_LAST);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn input(
        &mut self,
        pad: &PadState,
        tuning: &MenuTuning,
        bridge: &mut ItemMoveBridge,
        previous: u8,
        font: &mut FontBank,
        archive: &dyn MsgArchive,
        clear_count: u8,
        out: &mut MenuOutput,
    ) {
        if pad.a {
            if self.sub_win && self.now_item == SLOT_NONE {
                self.sub_window_delete();
                out.sounds.push(SE_MENU_CANCEL);
            } else if self.now_item == SLOT_LAST {
                self.item_mode = 1;
                self.save_open = true;
                self.save_status = 1;
                self.save_timer = 0;
                out.sounds.push(SE_MENU_OPT_IN);
            } else {
                let item = self.slot_item(self.now_item);
                if item != ITEM_NONE && !self.disabled(item) {
                    for index in [P_STR0, P_ST00, P_NT00, P_NK00, P_NO11] {
                        self.panes.pane_mut(index).set_init_alpha();
                    }
                    self.panes.pane_mut(P_NK00).user_area = 1;
                    self.panes.pane_mut(P_NT00).user_area = 1;
                    self.item_note_set(font, archive, clear_count);
                    out.sounds.push(SE_ITEM_EXP_OPEN);
                }
            }
        } else if pad.b {
            if self.sub_win {
                self.sub_window_delete();
                out.sounds.push(SE_MENU_CANCEL);
            }
        } else if pad.x || pad.y || pad.z {
            if self.sub_win && self.now_item == SLOT_NONE {
                self.sub_window_delete();
                out.sounds.push(SE_MENU_CANCEL);
            } else if self.now_item != SLOT_LAST {
                let item = self.slot_item(self.now_item);
                if item != ITEM_NONE {
                    if self.sub_win {
                        if self.now_item == SLOT_NONE {
                            self.sub_window_delete();
                            out.sounds.push(SE_MENU_CANCEL);
                        } else {
                            self.sub_item_decide(pad, bridge, out);
                        }
                    } else if !self.disabled(item) {
                        self.item_decide(pad, bridge, out);
                    }
                }
            }
        } else {
            if self.sub_win {
                self.cursor_sub_move(pad, tuning, out);
            } else {
                self.cursor_main_move(pad, out);
            }
            self.item_scale(tuning);
            if self.now_item != previous && previous == SLOT_BOW {
                for index in 0..3 {
                    self.panes.arrow_tint_mut(index).pane_trans(0.0, 0.0);
                }
            }
        }
    }

    pub fn draw_state(&mut self) -> ItemDraw {
        self.out_font_draw();
        let fade = if self.fading == 0 {
            1.0
        } else {
            value_increase(10, i32::from(self.timer), 0)
        };
        let plain = self.trigger_info != 1 && self.trigger_info != 2;
        let mut quads = Vec::with_capacity(PANE_COUNT);
        let order = self.panes.order.clone();
        for index in order {
            let pane = self.panes.panes[index].clone();
            if !pane.visible {
                continue;
            }
            let mut alpha = pane.now_alpha;
            if self.fading != 0 && !(plain && FADE_EXEMPT.contains(&index)) {
                alpha = (f32::from(alpha) * fade) as u8;
            }
            quads.push(ItemQuad {
                tag: pane.tag,
                kind: pane.kind,
                bounds: [
                    pane.pos_top_left.x,
                    pane.pos_top_left.y,
                    pane.size.x,
                    pane.size.y,
                ],
                alpha,
                texture: pane.texture,
                appended: pane.appended,
                blend: pane.blend,
                white: pane.white,
                black: pane.black,
                char_color: pane.char_color,
                grad_color: pane.grad_color,
                rotation: pane.rotation,
                visible: pane.visible,
            });
        }
        let mut texts = Vec::new();
        for index in [P_STR0, P_ST00, P_WD, P_NM00, P_NM01, P_ITNM, P_ITNK] {
            let pane = self.panes.panes[index].clone();
            texts.push(ItemText {
                tag: pane.tag,
                text: pane.text.clone(),
                bounds: [
                    pane.pos_top_left.x + pane.shift_x,
                    pane.pos_top_left.y + pane.shift_y,
                    pane.size.x,
                    pane.size.y,
                ],
                font_size: pane.get_font_size(),
                char_space: pane.char_space,
                line_space: pane.line_space,
                alpha: pane.now_alpha,
            });
        }
        let box_pane = self.panes.panes[P_WD].clone();
        texts.push(ItemText {
            tag: "wd",
            text: self.out_font.text.clone(),
            bounds: [
                self.out_font.left_up_x,
                self.out_font.left_up_y,
                box_pane.width(),
                box_pane.height(),
            ],
            font_size: (self.out_font.cell, self.out_font.cell),
            char_space: self.out_font.char_space,
            line_space: self.out_font.line_space,
            alpha: self.out_font.alpha,
        });
        ItemDraw {
            quads,
            texts,
            emitters: self.emitters,
            contents_color: self.panes.pane(P_WK01).contents_color,
            order: self.panes.order.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CollectPage {
    pub present: bool,
    pub note_open: bool,
    pub mode: u8,
    pub trigger_info: u8,
    pub timer: i16,
    pub open_done: bool,
    pub close_done: bool,
    pub slide_done: bool,
    pub alt_done: bool,
    pub button_icon_mode: u8,
}

impl Default for CollectPage {
    fn default() -> Self {
        Self {
            present: false,
            note_open: false,
            mode: 0,
            trigger_info: 0,
            timer: 0,
            open_done: false,
            close_done: false,
            slide_done: false,
            alt_done: false,
            button_icon_mode: 0,
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct MenuWindow {
    pub state: u8,
    pub buttons_pressed: u8,
    pub event_wait_frame: i8,
    pub pause: u8,
    pub menu_status: u8,
    pub menu_status_old: u8,
    pub push_menu_button: u8,
    pub item_mode: u8,
    pub collect_mode: u8,
    pub button_icon_mode: u8,
    pub picture_status: u8,
    pub heap_lock_level: u8,
    pub heap_lock_flag: bool,
    pub fwater_timer: u8,
    pub proc_msg_id: u32,
    pub proc_msg_kind: u8,
    pub capture_draw: u8,
    pub open_timer: i16,
    pub now_item_button: u8,
    pub now_collect_button: u8,
    pub name_buffers: [String; NAME_BUFFER_COUNT],
    pub note_buffers: [String; NAME_BUFFER_COUNT],
    pub dummy_buffers: [String; NAME_BUFFER_COUNT],
    pub texture_buffers: usize,
    pub child_heap: Option<usize>,
    pub cloth: Option<MenuCloth>,
    pub item: Option<ItemMenu>,
    pub collect: CollectPage,
    pub dungeon_bits: [u8; 0xC0],
    pub item_bits: [u8; WINDOW_BIT_ITEMS],
    pub ruby_source: u8,
    pub collect_slide: i16,
    pub attention: u8,
    pub message_open: bool,
}

impl Default for MenuWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuWindow {
    pub fn new() -> Self {
        Self {
            state: STATE_NO_MENU,
            buttons_pressed: 0,
            event_wait_frame: 0,
            pause: WINDOW_DRAW_OFF,
            menu_status: MENU_STATUS_ITEM,
            menu_status_old: MENU_STATUS_ITEM,
            push_menu_button: 0,
            item_mode: 0,
            collect_mode: 0,
            button_icon_mode: WINDOW_ICON_MODE_DEFAULT,
            picture_status: 0,
            heap_lock_level: 0,
            heap_lock_flag: false,
            fwater_timer: 0,
            proc_msg_id: u32::MAX,
            proc_msg_kind: 0,
            capture_draw: WINDOW_DRAW_OFF,
            open_timer: 0,
            now_item_button: 0,
            now_collect_button: 0,
            name_buffers: [String::new(), String::new()],
            note_buffers: [String::new(), String::new()],
            dummy_buffers: [String::new(), String::new()],
            texture_buffers: 0,
            child_heap: None,
            cloth: None,
            item: None,
            collect: CollectPage::default(),
            dungeon_bits: [0; 0xC0],
            item_bits: [0xFF; WINDOW_BIT_ITEMS],
            ruby_source: WINDOW_RUBY_SOURCE,
            collect_slide: 0,
            attention: 0,
            message_open: false,
        }
    }

    pub fn flag(&self) -> u8 {
        self.pause
    }

    pub fn flag_set(&mut self, value: u8) {
        self.pause = value;
    }

    pub fn on_button_bit(&mut self, bit: u8) {
        self.buttons_pressed |= bit;
    }

    pub fn off_button_bit(&mut self, bit: u8) {
        self.buttons_pressed &= !bit;
    }

    pub fn is_button_bit(&self, bit: u8) -> bool {
        self.buttons_pressed & bit != 0
    }

    pub fn is_push_l_button(&mut self, pad: &PadState) -> bool {
        if pad.lock_l && !self.is_button_bit(WINDOW_BUTTON_L) {
            self.on_button_bit(WINDOW_BUTTON_L);
            true
        } else {
            false
        }
    }

    pub fn is_push_r_button(&mut self, pad: &PadState) -> bool {
        if pad.lock_r && !self.is_button_bit(WINDOW_BUTTON_R) {
            self.on_button_bit(WINDOW_BUTTON_R);
            true
        } else {
            false
        }
    }

    pub fn child_heap_free_all(&mut self) {
        self.name_buffers = [String::new(), String::new()];
        self.note_buffers = [String::new(), String::new()];
        self.dummy_buffers = [String::new(), String::new()];
        self.texture_buffers = 0;
    }

    pub fn item_create(
        &mut self,
        screen: Option<&Screen>,
        equipped: &Equipped,
        record: &ItemRecord,
        env: &MenuEnv,
        out: &mut MenuOutput,
    ) {
        self.texture_buffers = ITEM_BUFFER_COUNT;
        for index in 0..NAME_BUFFER_COUNT {
            self.name_buffers[index].clear();
            self.note_buffers[index].clear();
            self.dummy_buffers[index].clear();
        }
        self.ruby_source = WINDOW_RUBY_SOURCE;
        let mut item = ItemMenu::empty();
        if let Some(screen) = screen {
            item.bind(screen);
        }
        item.now_item_button = self.now_item_button;
        item.opt_ruby = env.opt_ruby;
        item.boss_rematch = boss_rematch(env.stage_boss, &env.start_stage);
        item.sync(equipped, record);
        item.item_bit_check(true);
        self.item = Some(item);
        if env.opt_ruby {
            self.ruby_source = WINDOW_RUBY_SOURCE;
        }
        let _ = out;
    }

    pub fn item_delete(&mut self, out: &mut MenuOutput) {
        if let Some(item) = self.item.take() {
            self.now_item_button = item.now_item_button;
        }
        self.texture_buffers = 0;
        let _ = out;
    }

    pub fn cloth_create(&mut self, kind: u8) {
        self.heap_lock_level = 1;
        self.heap_lock_flag = true;
        self.cloth = Some(MenuCloth::new(kind));
        self.child_heap = Some(CHILD_HEAP_SIZE);
    }

    pub fn cloth_only_create(&mut self) {
        self.cloth = Some(MenuCloth::new(CLOTH_TYPE_CLOTH_ONLY));
    }

    pub fn cloth_delete(&mut self) {
        self.child_heap = None;
        self.cloth = None;
    }

    pub fn cloth_only_delete(&mut self) {
        self.cloth = None;
    }

    pub fn cloth_move(&mut self) {
        if let Some(cloth) = self.cloth.as_mut() {
            cloth.cloth_move();
        }
    }

    pub fn cloth_alpha_out(&mut self) {
        if let Some(cloth) = self.cloth.as_mut() {
            cloth.alpha_out();
        }
    }
}

pub struct WindowContext<'a> {
    pub pad: &'a PadState,
    pub equipped: &'a mut Equipped,
    pub record: &'a mut ItemRecord,
    pub env: &'a mut MenuEnv,
    pub tuning: &'a MenuTuning,
    pub bridge: &'a mut ItemMoveBridge,
    pub font: &'a mut FontBank,
    pub archive: &'a dyn MsgArchive,
    pub clear_count: u8,
    pub screen: Option<&'a Screen>,
    pub out: &'a mut MenuOutput,
}

impl MenuWindow {
    fn can_proceed(&self, env: &MenuEnv) -> bool {
        self.pause == WINDOW_DRAW_OFF && !env.enable_next_stage && !env.overlap_request
    }

    pub fn execute(&mut self, ctx: &mut WindowContext<'_>) {
        self.execute_common(ctx);
        if !ctx.pad.lock_l && self.is_button_bit(WINDOW_BUTTON_L) {
            self.off_button_bit(WINDOW_BUTTON_L);
        }
        if !ctx.pad.lock_r && self.is_button_bit(WINDOW_BUTTON_R) {
            self.off_button_bit(WINDOW_BUTTON_R);
        }
        self.item_mode = self.item.as_ref().map_or(0, |item| item.get_item_mode());
        self.collect_mode = if self.collect.present {
            self.collect.mode
        } else {
            0
        };
        if self.pause == WINDOW_DRAW_OFF && !ctx.env.event_running {
            ctx.out.item_timer_dec += 1;
        }
    }

    fn execute_common(&mut self, ctx: &mut WindowContext<'_>) {
        let env_flag = ctx.env.event_running;
        if env_flag {
            self.event_wait_frame = WINDOW_EVENT_WAIT;
        } else if self.event_wait_frame > 0 {
            self.event_wait_frame -= 1;
        } else {
            self.event_wait_frame = 0;
        }
        self.state = self.dispatch(ctx);
    }

    fn dispatch(&mut self, ctx: &mut WindowContext<'_>) -> u8 {
        match self.state {
            STATE_NO_MENU => {
                if self.heap_lock_flag || ctx.env.message_status != 0 {
                    return STATE_NO_MENU;
                }
                self.open_from_idle(ctx)
            }
            STATE_ITEM_OPEN => {
                self.open_timer = self.open_timer.wrapping_add(1);
                self.cloth_move();
                if ctx.tuning.open_wait_frames_reached(self.open_timer) {
                    let done = self
                        .item
                        .as_mut()
                        .map(|item| {
                            item.open(ctx.font, ctx.archive, ctx.clear_count, ctx.out)
                        })
                        .unwrap_or(false);
                    if done {
                        return STATE_ITEM_MOVE;
                    }
                }
                STATE_ITEM_OPEN
            }
            STATE_ITEM_OPEN_FROM_COLLECT_LEFT | STATE_ITEM_OPEN_FROM_COLLECT_RIGHT => {
                self.cloth_move();
                let done = self
                    .item
                    .as_mut()
                    .map(|item| {
                        item.open_page(ctx.tuning, ctx.font, ctx.archive, ctx.clear_count, ctx.out)
                    })
                    .unwrap_or(false);
                if done {
                    return STATE_ITEM_MOVE;
                }
                self.state
            }
            STATE_ITEM_MOVE => {
                self.cloth_move();
                let note_open = self.item.as_ref().is_some_and(|item| item.note_check());
                let mode = self.item.as_ref().map_or(0, |item| item.get_item_mode());
                let sub_win = self.item.as_ref().is_some_and(|item| item.sub_win);
                if (ctx.pad.start || ctx.pad.b) && !note_open && mode == 0 && !sub_win {
                    self.cloth_alpha_out();
                    if let Some(item) = self.item.as_mut() {
                        item.set_trigger_info(0);
                        item.set_timer(WINDOW_CLOSE_TIMER);
                    }
                    self.push_menu_button = 0;
                    ctx.out.push_menu_button = Some(0);
                    ctx.out.sounds.push(SE_MENU_OUT);
                    return STATE_ITEM_CLOSE;
                }
                let right = self.is_push_r_button(ctx.pad);
                if right && !note_open && mode == 0 {
                    if let Some(item) = self.item.as_mut() {
                        item.set_trigger_info(2);
                        item.set_timer(ctx.tuning.page_frames);
                    }
                    self.push_menu_button = 2;
                    ctx.out.push_menu_button = Some(2);
                    ctx.out.sounds.push(SE_ITEM_COL_SW);
                    self.collect_slide = ctx.tuning.page_frames;
                    return STATE_ITEM_TO_COLLECT_RIGHT;
                }
                let left = self.is_push_l_button(ctx.pad);
                if left && !note_open && mode == 0 {
                    if let Some(item) = self.item.as_mut() {
                        item.set_trigger_info(1);
                        item.set_timer(ctx.tuning.page_frames);
                    }
                    self.push_menu_button = 2;
                    ctx.out.push_menu_button = Some(2);
                    ctx.out.sounds.push(SE_ITEM_COL_SW);
                    self.collect_slide = ctx.tuning.page_frames;
                    return STATE_ITEM_TO_COLLECT_LEFT;
                }
                self.item_step(ctx);
                STATE_ITEM_MOVE
            }
            STATE_ITEM_CLOSE => {
                self.cloth_move();
                let done = self
                    .item
                    .as_mut()
                    .map(|item| item.close(WINDOW_CLOSE_TIMER, ctx.out))
                    .unwrap_or(true);
                if done {
                    self.item_delete(ctx.out);
                    self.child_heap_free_all();
                    self.cloth_delete();
                    self.flag_set(WINDOW_DRAW_OFF);
                    self.heap_lock_flag = false;
                    return STATE_NO_MENU;
                }
                STATE_ITEM_CLOSE
            }
            STATE_ITEM_TO_COLLECT_LEFT | STATE_ITEM_TO_COLLECT_RIGHT => {
                self.cloth_move();
                let done = self
                    .item
                    .as_mut()
                    .map(|item| item.close_page(ctx.tuning.page_frames, ctx.out))
                    .unwrap_or(true);
                if done {
                    let right = self.state == STATE_ITEM_TO_COLLECT_RIGHT;
                    self.item_delete(ctx.out);
                    self.child_heap_free_all();
                    self.collect.present = true;
                    self.collect.trigger_info = if right { 2 } else { 1 };
                    self.collect.timer = ctx.tuning.page_frames;
                    self.collect_slide = ctx.tuning.page_frames;
                    return if right {
                        STATE_COLLECT_OPEN_RIGHT
                    } else {
                        STATE_COLLECT_OPEN_LEFT
                    };
                }
                self.state
            }
            STATE_COLLECT_OPEN_LEFT | STATE_COLLECT_OPEN_RIGHT => {
                self.cloth_move();
                self.collect_slide -= 1;
                if self.collect_slide <= 0 {
                    self.collect.slide_done = false;
                    self.collect.mode = 0;
                    return STATE_COLLECT_MOVE;
                }
                self.state
            }
            STATE_COLLECT_TO_ITEM_LEFT | STATE_COLLECT_TO_ITEM_RIGHT => {
                self.cloth_move();
                self.collect_slide -= 1;
                if self.collect_slide <= 0 {
                    self.collect.close_done = false;
                    self.collect.present = false;
                    self.menu_status_old = self.menu_status;
                    self.menu_status = MENU_STATUS_ITEM;
                    self.item_create(ctx.screen, ctx.equipped, ctx.record, ctx.env, ctx.out);
                    if let Some(item) = self.item.as_mut() {
                        let info = if self.collect.trigger_info == 2 { 2 } else { 1 };
                        item.set_trigger_info(info);
                    }
                    return if self.collect.trigger_info == 2 {
                        STATE_ITEM_OPEN_FROM_COLLECT_RIGHT
                    } else {
                        STATE_ITEM_OPEN_FROM_COLLECT_LEFT
                    };
                }
                self.state
            }
            STATE_COLLECT_MOVE => {
                self.cloth_move();
                let note_open = self.collect.note_open;
                if (ctx.pad.start || ctx.pad.b) && !note_open && (self.collect.mode == 0 || self.collect.mode == 2)
                {
                    self.cloth_alpha_out();
                    self.collect.trigger_info = 0;
                    self.collect.timer = WINDOW_CLOSE_TIMER;
                    self.push_menu_button = 0;
                    ctx.out.push_menu_button = Some(0);
                    ctx.out.sounds.push(SE_MENU_OUT);
                    return STATE_COLLECT_CLOSE;
                }
                let right = self.is_push_r_button(ctx.pad);
                if right && !note_open && (self.collect.mode == 0 || self.collect.mode == 2) {
                    self.collect.trigger_info = 2;
                    self.collect.timer = ctx.tuning.page_frames;
                    self.push_menu_button = 1;
                    ctx.out.push_menu_button = Some(1);
                    ctx.out.sounds.push(SE_ITEM_COL_SW);
                    return STATE_COLLECT_TO_ITEM_RIGHT;
                }
                let left = self.is_push_l_button(ctx.pad);
                if left && !note_open && (self.collect.mode == 0 || self.collect.mode == 2) {
                    self.collect.trigger_info = 1;
                    self.collect.timer = ctx.tuning.page_frames;
                    self.push_menu_button = 1;
                    ctx.out.push_menu_button = Some(1);
                    ctx.out.sounds.push(SE_ITEM_COL_SW);
                    return STATE_COLLECT_TO_ITEM_LEFT;
                }
                STATE_COLLECT_MOVE
            }
            STATE_COLLECT_CLOSE => {
                self.cloth_move();
                if self.collect.close_done {
                    self.collect.present = false;
                    self.child_heap_free_all();
                    self.cloth_delete();
                    self.flag_set(WINDOW_DRAW_OFF);
                    self.heap_lock_flag = false;
                    return STATE_NO_MENU;
                }
                STATE_COLLECT_CLOSE
            }
            STATE_COLLECT_OPEN_ALT => {
                self.cloth_move();
                if self.collect.open_done {
                    return STATE_COLLECT_MOVE_ALT;
                }
                STATE_COLLECT_OPEN_ALT
            }
            STATE_COLLECT_MOVE_ALT => {
                self.cloth_move();
                if self.collect.mode != WINDOW_COLLECT_MODE_CLOSED {
                    self.cloth_alpha_out();
                    self.collect.trigger_info = 0;
                    self.collect.timer = WINDOW_CLOSE_TIMER;
                    self.push_menu_button = 0;
                    ctx.out.push_menu_button = Some(0);
                    ctx.out.sounds.push(SE_MENU_OUT);
                    return STATE_COLLECT_CLOSE_ALT;
                }
                STATE_COLLECT_MOVE_ALT
            }
            STATE_COLLECT_CLOSE_ALT => {
                self.cloth_move();
                if self.collect.close_done {
                    self.collect.present = false;
                    self.child_heap_free_all();
                    self.cloth_delete();
                    self.flag_set(WINDOW_DRAW_OFF);
                    self.heap_lock_flag = false;
                    return STATE_NO_MENU;
                }
                STATE_COLLECT_CLOSE_ALT
            }
            STATE_COLLECT_OPEN | STATE_COLLECT_OPEN_TACT => {
                self.cloth_move();
                if self.collect.open_done {
                    self.collect.open_done = false;
                    return if self.state == STATE_COLLECT_OPEN_TACT {
                        STATE_COLLECT_MOVE_TACT
                    } else {
                        STATE_COLLECT_MOVE
                    };
                }
                self.state
            }
            STATE_COLLECT_MOVE_TACT => {
                self.cloth_move();
                let idle = self.collect.mode == 0 || self.collect.mode == 2;
                if (ctx.pad.start || ctx.pad.b) && !self.collect.note_open && idle {
                    self.cloth_alpha_out();
                    self.collect.trigger_info = 0;
                    self.collect.timer = WINDOW_CLOSE_TIMER;
                    self.push_menu_button = 0;
                    ctx.out.push_menu_button = Some(0);
                    ctx.out.sounds.push(SE_MENU_OUT);
                    return STATE_COLLECT_CLOSE_TACT;
                }
                STATE_COLLECT_MOVE_TACT
            }
            STATE_COLLECT_CLOSE_TACT => {
                self.cloth_move();
                if self.collect.close_done {
                    self.collect.present = false;
                    self.child_heap_free_all();
                    self.cloth_delete();
                    self.flag_set(WINDOW_DRAW_OFF);
                    self.heap_lock_flag = false;
                    return STATE_NO_MENU;
                }
                STATE_COLLECT_CLOSE_TACT
            }
            _ => self.state,
        }
    }

    fn item_step(&mut self, ctx: &mut WindowContext<'_>) {
        let equipped = *ctx.equipped;
        let record = *ctx.record;
        if let Some(item) = self.item.as_mut() {
            item.sync(&equipped, &record);
            item.step(
                ctx.pad,
                ctx.tuning,
                ctx.bridge,
                ctx.font,
                ctx.archive,
                ctx.clear_count,
                ctx.out,
            );
        }
    }

    fn open_from_idle(&mut self, ctx: &mut WindowContext<'_>) -> u8 {
        if self.can_proceed(ctx.env) && ctx.env.fmap_open_check == 1 {
            self.open_timer = 0;
            self.flag_set(WINDOW_DRAW_ON);
            self.menu_status_old = self.menu_status;
            self.menu_status = MENU_STATUS_MAP;
            ctx.out.sounds.push(SE_SHIPPU_CHART_OPEN);
            return STATE_FMAP_OPEN_WARP_MODE;
        }
        if self.can_proceed(ctx.env) && ctx.env.password_open_check == 2 {
            self.flag_set(WINDOW_DRAW_ON);
            self.menu_status_old = self.menu_status;
            self.menu_status = MENU_STATUS_NAME;
            return STATE_NAME_OPEN;
        }
        if self.can_proceed(ctx.env) && ctx.env.fmap_open_check == 2 {
            self.open_timer = 0;
            self.flag_set(WINDOW_DRAW_ON);
            self.menu_status_old = self.menu_status;
            self.menu_status = MENU_STATUS_MAP;
            ctx.out.sounds.push(SE_MENU_IN);
            ctx.out.sounds.push(SE_MENU_MAP_IN);
            return STATE_FMAP_OPEN_FISHMAN_MODE;
        }
        if self.can_proceed(ctx.env) && ctx.env.menu_collect {
            self.cloth_create(CLOTH_TYPE_DEFAULT);
            self.open_timer = 0;
            self.flag_set(WINDOW_DRAW_ON);
            self.collect.present = true;
            self.collect.mode = 0;
            self.menu_status_old = self.menu_status;
            self.menu_status = MENU_STATUS_COLLECT;
            ctx.out.sounds.push(SE_MENU_IN);
            ctx.out.sounds.push(SE_MENU_PAGE);
            ctx.env.menu_collect = false;
            return STATE_COLLECT_OPEN_ALT;
        }
        let blocked = self.pause != WINDOW_DRAW_OFF
            || ctx.env.overlap_request
            || ctx.pad.a
            || ctx.pad.b
            || ctx.pad.z;
        if blocked {
            return STATE_NO_MENU;
        }
        let camera_busy = ctx.env.camera_telescope
            || ctx.env.camera_pictobox
            || ctx.env.player_status_800000;
        if camera_busy {
            return STATE_NO_MENU;
        }
        if ctx.env.message_status != 0 || ctx.env.scope_message_status != 0 {
            return STATE_NO_MENU;
        }
        let wait_open = self.event_wait_frame == 0
            || (ctx.env.tact_normal_wait && ctx.pad.start)
            || (ctx.env.operate_wind == 2
                && ctx.pad.up
                && ctx.env.up_button == 0
                && ctx.env.event_bit_0908);
        if !wait_open {
            return STATE_NO_MENU;
        }
        let wind_gate = ctx.env.operate_wind != 2
            || (ctx.pad.up && ctx.env.up_button == 0 && ctx.env.event_bit_0908);
        if !wind_gate {
            return STATE_NO_MENU;
        }
        if ctx.pad.start && !ctx.env.enable_next_stage && ctx.env.link_actor {
            self.cloth_create(CLOTH_TYPE_DEFAULT);
            self.open_timer = 0;
            self.flag_set(WINDOW_DRAW_ON);
            let item_first = self.menu_status == MENU_STATUS_ITEM || ctx.env.tact_normal_wait;
            if item_first {
                self.collect.present = true;
                self.collect.trigger_info = 2;
                self.menu_status_old = self.menu_status;
                self.menu_status = MENU_STATUS_COLLECT;
                self.push_menu_button = 1;
                ctx.out.push_menu_button = Some(1);
                ctx.out.sounds.push(SE_MENU_IN);
                ctx.out.sounds.push(SE_MENU_PAGE);
                if ctx.env.tact_normal_wait {
                    return STATE_COLLECT_OPEN_TACT;
                }
                return STATE_COLLECT_OPEN;
            }
            self.item_create(ctx.screen, ctx.equipped, ctx.record, ctx.env, ctx.out);
            if let Some(item) = self.item.as_mut() {
                item.set_trigger_info(2);
            }
            self.menu_status_old = self.menu_status;
            self.menu_status = MENU_STATUS_ITEM;
            self.push_menu_button = 1;
            ctx.out.push_menu_button = Some(1);
            ctx.out.sounds.push(SE_MENU_IN);
            ctx.out.sounds.push(SE_MENU_PAGE);
            return STATE_ITEM_OPEN;
        }
        if self.can_proceed(ctx.env) && ctx.pad.up {
            self.open_timer = 0;
            if ctx.env.up_button == 1 {
                self.flag_set(WINDOW_DRAW_ON);
                self.menu_status_old = self.menu_status;
                self.menu_status = MENU_STATUS_MAP;
                ctx.out.sounds.push(SE_MENU_IN);
                ctx.out.sounds.push(SE_MENU_MAP_IN);
                return STATE_DMAP_OPEN;
            }
            let sea = ctx.env.up_button == 0
                && ctx.env.event_bit_0908
                && ctx.env.mini_game_type != 1
                && ctx.env.mini_game_type != 6;
            if sea {
                if ctx.env.operate_wind == 2 {
                    return STATE_FMAP_OPEN_WALLPAPER;
                }
                self.cloth_only_create();
                self.flag_set(WINDOW_DRAW_ON);
                self.menu_status_old = self.menu_status;
                self.menu_status = MENU_STATUS_MAP;
                ctx.out.sounds.push(SE_MENU_IN);
                ctx.out.sounds.push(SE_MENU_MAP_IN);
                return STATE_FMAP_OPEN;
            }
        }
        STATE_NO_MENU
    }

    pub fn draw(&mut self) -> WindowDraw {
        let mut draw = WindowDraw {
            visible: false,
            capture: false,
            item: None,
            cloth: None,
        };
        if self.pause == WINDOW_DRAW_ON {
            draw.visible = true;
            if self.capture_draw != WINDOW_DRAW_OFF {
                draw.capture = true;
                if self.capture_draw == WINDOW_DRAW_CAPTURE {
                    self.capture_draw = WINDOW_DRAW_OFF;
                    self.flag_set(WINDOW_DRAW_ON);
                }
            }
            if self.menu_status == MENU_STATUS_ITEM {
                draw.cloth = self.cloth.as_mut().map(MenuCloth::mesh);
                draw.item = self.item.as_mut().map(ItemMenu::draw_state);
            }
        } else if self.pause == WINDOW_DRAW_CAPTURE {
            draw.visible = true;
            draw.capture = true;
        }
        draw
    }
}

impl MenuTuning {
    pub fn open_wait_frames_reached(&self, timer: i16) -> bool {
        timer > self.open_wait
    }
}

impl ItemMoveBridge {
    pub fn pending(&self) -> bool {
        self.flag != 2
    }
}

pub struct WindowDraw {
    pub visible: bool,
    pub capture: bool,
    pub item: Option<ItemDraw>,
    pub cloth: Option<ClothMesh>,
}

pub fn menu_window_step(
    mut window: ResMut<MenuWindow>,
    pad: Res<PadState>,
    mut equipped: ResMut<Equipped>,
    mut record: ResMut<ItemRecord>,
    mut env: ResMut<MenuEnv>,
    tuning: Res<MenuTuning>,
    mut bridge: ResMut<ItemMoveBridge>,
    mut out: ResMut<MenuOutput>,
    screens: Option<Res<j2d::Screens>>,
    mut fonts: Option<ResMut<FontBank>>,
    archive: Option<Res<crate::assets::MsgArchives>>,
) {
    out.reset();
    let (Some(archive), Some(mut fonts)) = (archive, fonts) else {
        return;
    };
    let pad_state = *pad;
    let mut equip_copy = *equipped;
    let mut record_copy = *record;
    let mut env_copy = (*env).clone();
    let screen = screens.as_ref().and_then(|screens| {
        screens
            .cache
            .iter()
            .find(|(label, _)| label == SCREEN_LABEL)
            .map(|(_, screen)| screen)
    });
    let archive_ref: &dyn MsgArchive = &*archive;
    {
        let mut ctx = WindowContext {
            pad: &pad_state,
            equipped: &mut equip_copy,
            record: &mut record_copy,
            env: &mut env_copy,
            tuning: &*tuning,
            bridge: &mut *bridge,
            font: &mut *fonts,
            archive: archive_ref,
            clear_count: 0,
            screen,
            out: &mut *out,
        };
        window.execute(&mut ctx);
    }
    if let Some(item) = window.item.as_mut() {
        item.sync(&equip_copy, &record_copy);
    }
    *equipped = equip_copy;
    *record = record_copy;
    *env = env_copy;
}
