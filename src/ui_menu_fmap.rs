use std::rc::Rc;

use bevy_ecs::prelude::*;

use crate::j2d::{Kind, Screen, ScreenPane};
use crate::sim::flags::{Arrivals, Charts, Collect, EventFlags, MemBits, StoryBits};

pub const SCREEN_LABEL: &str = "f_map.blo";
pub const SCREEN_ARCHIVE: &str = "files/res/Msg/fmapres.arc";
pub const TEXTURE_ARCHIVE: &str = "files/res/Fmap/Fmap.arc";
pub const GRID_MARK_FILE: &str = "files/res/FmapDat/FmapDat.bin";
pub const CHART_DATA_GROUP: &str = "DATA";
pub const CHART_DATA_ENTRY: &str = "CmapDat.bin";
pub const CHECK_MARK_GROUP: &str = "FM  ";
pub const CHECK_MARK_BUFFER_SIZE: usize = 0x2C00;
pub const TEXT_BUFFER_LEN: usize = 0x80;
pub const TUNING_NAME: &str = "フィールドマップ画面";

pub const DISP_NORMAL: u8 = 0;
pub const DISP_COMPARE: u8 = 1;

pub const MODE_NORMAL: u8 = 0;
pub const MODE_WARP: u8 = 1;
pub const MODE_WALLPAPER: u8 = 2;
pub const MODE_FISHMAN: u8 = 3;

pub const ZOOM_WORLD: u8 = 0;
pub const ZOOM_SECTOR: u8 = 1;
pub const ZOOM_DETAIL: u8 = 2;

pub const BTN_ICON_WORLD: u8 = 0;
pub const BTN_ICON_SECTOR: u8 = 1;
pub const BTN_ICON_DETAIL: u8 = 2;
pub const BTN_ICON_LOCKED: u8 = 3;
pub const BTN_ICON_WARP: u8 = 4;
pub const BTN_ICON_WALLPAPER: u8 = 5;
pub const BTN_ICON_FISHMAN: u8 = 6;

pub const WARP_SEL_YES: u8 = 0;
pub const WARP_SEL_NO: u8 = 1;

pub const WARP_SPOT_COUNT: usize = 9;

pub const ZOOM_PROC_SELECT_GRID: u8 = 0;
pub const ZOOM_PROC_LV1_IN: u8 = 1;
pub const ZOOM_PROC_LV1: u8 = 2;
pub const ZOOM_PROC_LV1_OUT: u8 = 3;
pub const ZOOM_PROC_LV2_IN: u8 = 4;
pub const ZOOM_PROC_LV2: u8 = 5;
pub const ZOOM_PROC_LV2_OUT: u8 = 6;

pub const MAIN_PROC_FMAP: u8 = 0;
pub const MAIN_PROC_HIKAKU: u8 = 1;

pub const HIKAKU_PROC_OPEN: u8 = 0;
pub const HIKAKU_PROC_MOVE: u8 = 1;
pub const HIKAKU_PROC_CLOSE: u8 = 2;

pub const WARP_PROC_MOVE: u8 = 0;
pub const WARP_PROC_SEL_WIN_FADE_IN: u8 = 1;
pub const WARP_PROC_SELECT: u8 = 2;
pub const WARP_PROC_SEL_WIN_FADE_OUT: u8 = 3;
pub const WARP_PROC_SEL_WARP: u8 = 4;

pub const FISHMAN_PROC_DISP_AREA: u8 = 0;
pub const FISHMAN_PROC_ZOOM_LV1_IN: u8 = 1;
pub const FISHMAN_PROC_ZOOM_LV2_IN: u8 = 2;
pub const FISHMAN_PROC_MAP_WRITE: u8 = 3;
pub const FISHMAN_PROC_MAP_WAIT: u8 = 4;
pub const FISHMAN_PROC_ZOOM_LV2_OUT: u8 = 5;
pub const FISHMAN_PROC_ZOOM_LV1_OUT: u8 = 6;
pub const FISHMAN_PROC_END_WAIT: u8 = 7;

pub const GRID_WIDTH: i32 = 7;
pub const GRID_COUNT: i32 = 49;
pub const GRID_MIN: i8 = -3;
pub const GRID_MAX: i8 = 3;
pub const GRID_STEP: f32 = 56.0;
pub const WORLD_UNIT: f32 = 100000.0;
pub const ISLAND_MARK_RANGE: f32 = 10000.0;
pub const F2D_MARGIN: f32 = 50.0;

pub const EVENT_MET_FOREST_HUMAN: u32 = 0x0F80;
pub const EVENT_PLACED_DINS_PEARL: u32 = 0x1480;
pub const EVENT_PLACED_FARORES_PEARL: u32 = 0x1440;
pub const EVENT_PLACED_NAYRUS_PEARL: u32 = 0x1410;
pub const EVENT_UNKNOWN_1E80: u32 = 0x1E80;
pub const EVENT_JAB_IRIS_RETRIEVED: u32 = 0x3920;
pub const EVENT_UNKNOWN_1820: u32 = 0x1820;
pub const EVENT_UNKNOWN_0102: u32 = 0x0102;
pub const EVENT_UNKNOWN_3940: u32 = 0x3940;
pub const EVENT_UNKNOWN_2D80: u32 = 0x2D80;
pub const EVENT_UNKNOWN_9EFF: u32 = 0x9EFF;
pub const REG_GHOST_SHIP: u16 = 0x8803;

pub const SYMBOL_NAYRU: u8 = crate::sim::flags::SYMBOL_NAYRU;
pub const SYMBOL_FARORE: u8 = crate::sim::flags::SYMBOL_FARORE;

pub const MSG_GRID_NAME_BASE: u32 = 0x31A6;
pub const MSG_PRIVATE_ISLAND: u32 = 0x31D7;
pub const MSG_UNKNOWN_NAME: u32 = 0x319B;
pub const MSG_UNKNOWN_AREA: u32 = 0x5F;

pub const SE_YUUREISEN_SONG: u32 = 0x107A;
pub const SE_TALK_CURSOR: u32 = 0x083D;
pub const SE_CHART_CURSOR: u32 = 0x08C3;
pub const SE_CHART_ZOOM_IN: u32 = 0x08C4;
pub const SE_CHART_ZOOM_OUT: u32 = 0x08C5;
pub const SE_CHART_TO_COMPARE: u32 = 0x08C6;
pub const SE_CHART_TO_NORMAL: u32 = 0x08CA;
pub const SE_SHIPPU_CHART_OPEN: u32 = 0x08CB;
pub const SE_SHIPPU_CHART_CLOSE: u32 = 0x08CC;
pub const SE_SHIPPU_DIST_SEL: u32 = 0x08CD;
pub const SE_SHIPPU_WARP_OK: u32 = 0x08CE;
pub const SE_SHIPPU_WARP_CANCEL: u32 = 0x08CF;
pub const SE_CM_SO_DRAW_MAP: u32 = 0x593D;
pub const BGM_BGN_GET_BOX: u32 = 0x8000005D;

pub const PARTICLE_WARP_SPOT: u16 = 0x0357;
pub const PARTICLE_WARP_BACK: u16 = 0x0358;

pub const ITEM_FOREST_WATER: u8 = 0x59;

pub const ISLE_IDX_FORSAKEN_FORTRESS: i32 = 0;
pub const ISLE_IDX_GALE_ISLE: i32 = 3;
pub const ISLE_IDX_WINDFALL: i32 = 10;
pub const ISLE_IDX_DRAGON_ROOST: i32 = 12;
pub const ISLE_IDX_FIRE_MOUNTAIN: i32 = 19;
pub const ISLE_IDX_GREATFISH_ISLE: i32 = 22;
pub const ISLE_IDX_TOWER_OF_THE_GODS: i32 = 25;
pub const ISLE_IDX_ICE_RING_ISLE: i32 = 39;
pub const ISLE_IDX_FOREST_HAVEN: i32 = 40;
pub const ISLE_IDX_OUTSET_ISLAND: i32 = 43;
pub const ISLE_IDX_HEADSTONE_ISLAND: i32 = 44;
pub const ISLE_IDX_PRIVATE_OASIS: i32 = 32;

pub const ISLE_ROOM_FORSAKEN_FORTRESS: i32 = 1;
pub const ISLE_ROOM_GALE_ISLE: i32 = 4;
pub const ISLE_ROOM_WINDFALL: i32 = 11;
pub const ISLE_ROOM_DRAGON_ROOST: i32 = 13;
pub const ISLE_ROOM_FIRE_MOUNTAIN: i32 = 20;
pub const ISLE_ROOM_GREATFISH_ISLE: i32 = 23;
pub const ISLE_ROOM_TOWER_OF_THE_GODS: i32 = 26;
pub const ISLE_ROOM_ICE_RING_ISLE: i32 = 40;
pub const ISLE_ROOM_FOREST_HAVEN: i32 = 41;
pub const ISLE_ROOM_OUTSET_ISLAND: i32 = 44;
pub const ISLE_ROOM_HEADSTONE_ISLAND: i32 = 45;

pub const NO_COLOR: u32 = 0x0000_0000;
pub const FULL_COLOR: u32 = 0xFFFF_FFFF;
pub const WARP_NO_SHADOW: u32 = 0x3315_0000;
pub const WARP_NO_BODY: u32 = 0x7654_2FFF;
pub const WARP_YES_SHADOW: u32 = 0xFFFF_FF00;

pub const KTX1_FONT_SIZE_X: f32 = 27.0;
pub const KTX1_FONT_SIZE_Y: f32 = 29.0;
pub const KTX1_LINE_SPACE: f32 = 47.0;
pub const KTX2_FONT_SIZE: f32 = 16.0;

pub const STICK_HALF_DEAD: i16 = 0x2000;
pub const STICK_FIRST_WAIT: i16 = 5;
pub const STICK_SECOND_WAIT: i16 = 2;
pub const STICK_STEP: i16 = 3;
pub const STICK_MIN_STEP: i16 = 2;
pub const STICK_MAX_VALUE: f32 = 0.9;
pub const STICK_MIN_VALUE: f32 = 0.5;

pub const KOROK_STATS: [&str; 2] = ["f_korog_kare.bti", "f_korog_saki.bti"];

pub const ROLL_MAP_TEX: [&str; 19] = [
    "cmap_tri.bti",
    "cmap_treasure.bti",
    "cmap_treasure.bti",
    "cmap_tingle.bti",
    "cmap_phantomship.bti",
    "cmap_treasure.bti",
    "cmap_treasure.bti",
    "cmap_tingle.bti",
    "cmap_treasure.bti",
    "cmap_hint.bti",
    "cmap_hint.bti",
    "cmap_hint.bti",
    "cmap_hint.bti",
    "cmap_hint.bti",
    "cmap_hint.bti",
    "cmap_hint.bti",
    "cmap_hint.bti",
    "cmap_hint.bti",
    "cmap_hint.bti",
];

pub const SALVAGE_ITEM_TEX: [&str; 19] = [
    "f_item_tri.bti",
    "f_get_rupy.bti",
    "f_heart_up_02.bti",
    "cmap_tingle.bti",
    "cmap_phantomship.bti",
    "cmap_treasure.bti",
    "cmap_hint.bti",
    "cmap_hint.bti",
    "cmap_tri.bti",
    "cmap_hint.bti",
    "cmap_hint.bti",
    "cmap_hint.bti",
    "cmap_hint.bti",
    "cmap_hint.bti",
    "cmap_hint.bti",
    "cmap_hint.bti",
    "cmap_hint.bti",
    "cmap_hint.bti",
    "cmap_hint.bti",
];

pub const AREA_TAGS: [&str; 49] = [
    "mR01", "mR02", "mR03", "mR04", "mR05", "mR06", "mR07", "mR08", "mR09", "mR0A", "mR0B", "mR0C",
    "mR0D", "mR0E", "mR0F", "mR10", "mR11", "mR12", "mR13", "mR14", "mR15", "mR16", "mR17", "mR18",
    "mR19", "mR1A", "mR1B", "mR1C", "mR1D", "mR1E", "mR1F", "mR20", "mR21", "mR22", "mR23", "mR24",
    "mR25", "mR26", "mR27", "mR28", "mR29", "mR2A", "mR2B", "mR2C", "mR2D", "mR2E", "mR2F", "mR30",
    "mR31",
];

pub const SALVAGE_MARK_TAGS: [&str; 49] = [
    "ST01", "ST02", "ST03", "ST04", "ST05", "ST06", "ST07", "ST08", "ST09", "ST10", "ST11", "ST12",
    "ST13", "ST14", "ST15", "ST16", "ST17", "ST18", "ST19", "ST20", "ST21", "ST22", "ST23", "ST24",
    "ST25", "ST26", "ST27", "ST28", "ST29", "ST30", "ST31", "ST32", "ST33", "ST34", "ST35", "ST36",
    "ST37", "ST38", "ST39", "ST40", "ST41", "ST42", "ST43", "ST44", "ST45", "ST46", "ST47", "ST48",
    "ST49",
];

pub const CURSOR_TABLE: [CursorSlot; WARP_SPOT_COUNT] = [
    CursorSlot {
        grid_x: -2,
        grid_y: -2,
        no: 0,
        left: -1,
        right: 1,
        up: -1,
        down: 7,
    },
    CursorSlot {
        grid_x: 0,
        grid_y: -2,
        no: 1,
        left: 0,
        right: 2,
        up: -1,
        down: 7,
    },
    CursorSlot {
        grid_x: 2,
        grid_y: -2,
        no: 2,
        left: 1,
        right: -1,
        up: 0,
        down: 7,
    },
    CursorSlot {
        grid_x: -2,
        grid_y: 0,
        no: 3,
        left: -1,
        right: 4,
        up: 7,
        down: 8,
    },
    CursorSlot {
        grid_x: 1,
        grid_y: 0,
        no: 4,
        left: 3,
        right: -1,
        up: 7,
        down: 8,
    },
    CursorSlot {
        grid_x: 2,
        grid_y: 2,
        no: 5,
        left: 8,
        right: -1,
        up: 4,
        down: 6,
    },
    CursorSlot {
        grid_x: -2,
        grid_y: 3,
        no: 6,
        left: 5,
        right: -1,
        up: 8,
        down: -1,
    },
    CursorSlot {
        grid_x: -1,
        grid_y: -1,
        no: 7,
        left: -1,
        right: -1,
        up: 0,
        down: 3,
    },
    CursorSlot {
        grid_x: 0,
        grid_y: 2,
        no: 8,
        left: -1,
        right: 5,
        up: 3,
        down: 6,
    },
];

pub const ISLAND_ROOM_TABLE: [i32; 11] = [
    ISLE_ROOM_FORSAKEN_FORTRESS,
    ISLE_ROOM_GALE_ISLE,
    ISLE_ROOM_WINDFALL,
    ISLE_ROOM_DRAGON_ROOST,
    ISLE_ROOM_FIRE_MOUNTAIN,
    ISLE_ROOM_GREATFISH_ISLE,
    ISLE_ROOM_TOWER_OF_THE_GODS,
    ISLE_ROOM_ICE_RING_ISLE,
    ISLE_ROOM_FOREST_HAVEN,
    ISLE_ROOM_OUTSET_ISLAND,
    ISLE_ROOM_HEADSTONE_ISLAND,
];

pub const ISLAND_PANE_TABLE: [&str; 11] = [
    "R01", "R04", "R0B", "R0D", "R14", "R17", "R1A", "R28", "R29", "R2C", "R2D",
];

pub const SELECT_CURSOR_TABLE: [&str; 8] = [
    "kk10", "kk11", "kk12", "kk13", "kk20", "kk21", "kk22", "kk23",
];

pub const ZOOM_CURSOR_TABLE: [&str; 8] = [
    "kk30", "kk31", "kk32", "kk33", "kk40", "kk41", "kk42", "kk43",
];

pub const WORLD_MARK_TABLE: [&str; 9] = [
    "sc11", "sc12", "sc13", "sc14", "sc15", "sc16", "sc17", "sc18", "sc19",
];

pub const DETAIL_MARK_TABLE: [&str; 7] = ["sc21", "sc22", "sc23", "sc24", "sc25", "sc26", "sc27"];

pub const DETAIL_FRAME_TABLE: [&str; 5] = ["fmum", "fmw1", "fmw2", "fmw3", "fmw4"];

pub const DETAIL_AREA_TABLE: [&str; 15] = [
    "fmn1", "fmn2", "fmn3", "fmn4", "fmn5", "fmn6", "fmn7", "fmn8", "fmna", "fmnb", "fmnc", "fmnd",
    "fmne", "fmnf", "fmnh",
];

pub const PEARL_TABLE_A: [&str; 3] = ["CK11", "CK12", "CK13"];
pub const PEARL_TABLE_B: [&str; 3] = ["CK21", "CK22", "CK23"];
pub const PEARL_TABLE_C: [&str; 2] = ["CK31", "CK32"];

pub const SECRET_TABLE: [&str; 8] = [
    "KR03", "KR05", "KR01", "KR08", "KR06", "KR02", "KR07", "KR04",
];

pub const AREA_TEXT_TABLE: [&str; 3] = ["txt1", "txt3", "txt2"];

pub const GHOST_SHIP_TABLE: [&str; 7] = ["gs07", "gs01", "gs02", "gs03", "gs04", "gs05", "gs06"];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CursorSlot {
    pub grid_x: i8,
    pub grid_y: i8,
    pub no: i8,
    pub left: i8,
    pub right: i8,
    pub up: i8,
    pub down: i8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tuning {
    pub no: i8,
    pub field_0x05: u8,
    pub field_0x06: u8,
    pub field_0x07: u8,
    pub field_0x08: u8,
    pub wind_arrow_color1: u32,
    pub wind_arrow_color2: u32,
    pub wind_arrow_pivot_x: i16,
    pub wind_arrow_pivot_y: i16,
    pub player_point_white1: u32,
    pub player_point_white2: u32,
    pub player_point_black1: u32,
    pub player_point_black2: u32,
    pub wind_arrow_period: i16,
    pub player_point_period: i16,
    pub wind_arrow_hold1: i16,
    pub wind_arrow_hold2: i16,
    pub zoom_anim_frame: u8,
    pub zoom2_anim_frame: u8,
    pub zoom_out_delay: u8,
    pub area_text_change_frame: u8,
    pub area_text_alpha_frame: u8,
    pub open_delay: u8,
    pub base_anim_frame: u8,
    pub open_pos_y: f32,
    pub close_pos_y: f32,
    pub cursor_flash_frame: u8,
    pub select_cursor_alpha_frame: u8,
    pub zoom_map_delay: u8,
    pub field_0x3d: u8,
    pub warp_msg_back_alpha_frame: u8,
    pub warp_msg_back_in_delay: u8,
    pub warp_msg_back_out_delay: u8,
    pub warp_msg_anim_frame: u8,
    pub warp_msg_pos_y: f32,
    pub warp_msg_delay: u8,
    pub field_0x43: u8,
    pub warp_yes_no_anim_frame: u8,
    pub field_0x47: u8,
    pub field_0x48: u8,
    pub warp_no_delay: u8,
    pub warp_yes_no_pos_y: f32,
    pub warp_cursor_blink_frame: u8,
    pub warp_cursor_alpha_min: u8,
    pub warp_cursor_alpha_max: u8,
    pub warp_spot_pos_x: [f32; WARP_SPOT_COUNT],
    pub warp_spot_pos_y: [f32; WARP_SPOT_COUNT],
    pub warp_spot_scale_sel: f32,
    pub warp_spot_scale: f32,
    pub field_0xa0: f32,
    pub field_0xa4: f32,
    pub field_0xa8: f32,
    pub field_0xac: f32,
    pub field_0xb0: f32,
    pub field_0xb4: f32,
    pub field_0xb8: f32,
    pub field_0xbc: f32,
    pub field_0xc0: f32,
    pub field_0xc4: f32,
    pub field_0xc8: f32,
    pub field_0xcc: f32,
    pub field_0xd0: f32,
    pub field_0xd4: f32,
    pub field_0xd8: f32,
    pub field_0xdc: f32,
    pub check_mark_anim_frame: u8,
    pub check_mark_white: u32,
    pub check_mark_black: u32,
    pub korok_mark_anim_frame: u8,
    pub korok_mark_anim_white: u32,
    pub korok_mark_anim_black: u32,
    pub field_0xf2: u8,
    pub korok_mark_white: u32,
    pub korok_mark_black: u32,
    pub field_0xfb: u8,
    pub area_text_color: u32,
    pub area_text_color_main: u32,
    pub field_0x104: u8,
    pub field_0x105: u8,
    pub field_0x106: u8,
    pub warp_back_color_cycle: u8,
    pub warp_back_color1: u32,
    pub warp_back_color2: u32,
    pub fishman_open_frame: u8,
    pub fishman_zoom1_frame: u8,
    pub field_0x112: u8,
    pub fishman_zoom1_out_frame: u8,
    pub fishman_zoom2_frame: u8,
    pub fishman_zoom2_alpha_frame: u8,
    pub fishman_map_write_wait: u8,
    pub fishman_map_write_frame: u8,
    pub fishman_blink_frame: u8,
    pub fishman_disp_wait: u8,
    pub fishman_close_wait: u8,
    pub fishman_close_frame: u8,
    pub moon_alpha_frame: u8,
    pub moon_alpha_max: u8,
    pub moon_alpha_min: u8,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            no: 0,
            field_0x05: 0,
            field_0x06: 0,
            field_0x07: 0,
            field_0x08: 0,
            wind_arrow_color1: 0xFFFF_FFFF,
            wind_arrow_color2: 0x8080_FFFF,
            wind_arrow_pivot_x: 0,
            wind_arrow_pivot_y: 2,
            player_point_white1: 0xFFFF_00FF,
            player_point_white2: 0xFF00_00FF,
            player_point_black1: 0xFFFF_829B,
            player_point_black2: 0xFF8E_00FF,
            wind_arrow_period: 15,
            player_point_period: 25,
            wind_arrow_hold1: 15,
            wind_arrow_hold2: 5,
            zoom_anim_frame: 7,
            zoom2_anim_frame: 7,
            zoom_out_delay: 5,
            area_text_change_frame: 50,
            area_text_alpha_frame: 10,
            open_delay: 4,
            base_anim_frame: 6,
            open_pos_y: 20.0,
            close_pos_y: 60.0,
            cursor_flash_frame: 8,
            select_cursor_alpha_frame: 3,
            zoom_map_delay: 3,
            field_0x3d: 3,
            warp_msg_back_alpha_frame: 6,
            warp_msg_back_in_delay: 0,
            warp_msg_back_out_delay: 0,
            warp_msg_anim_frame: 5,
            warp_msg_pos_y: -500.0,
            warp_msg_delay: 0,
            field_0x43: 0,
            warp_yes_no_anim_frame: 5,
            field_0x47: 8,
            field_0x48: 5,
            warp_no_delay: 2,
            warp_yes_no_pos_y: 200.0,
            warp_cursor_blink_frame: 12,
            warp_cursor_alpha_min: 70,
            warp_cursor_alpha_max: 140,
            warp_spot_pos_x: [
                -193.0, -82.0, 30.0, -193.0, -26.0, 30.0, -193.0, -137.0, -83.0,
            ],
            warp_spot_pos_y: [
                -137.0, -137.0, -137.0, -25.0, -25.0, 86.0, 145.0, -80.0, 86.0,
            ],
            warp_spot_scale_sel: 1.6,
            warp_spot_scale: 0.75,
            field_0xa0: 30.0,
            field_0xa4: -195.0,
            field_0xa8: 30.0,
            field_0xac: -135.0,
            field_0xb0: -25.0,
            field_0xb4: 85.0,
            field_0xb8: -82.0,
            field_0xbc: -82.0,
            field_0xc0: 30.0,
            field_0xc4: -80.0,
            field_0xc8: 34.0,
            field_0xcc: -25.0,
            field_0xd0: -80.0,
            field_0xd4: -137.0,
            field_0xd8: -195.0,
            field_0xdc: 145.0,
            check_mark_anim_frame: 50,
            check_mark_white: 0xFFFF_FFFF,
            check_mark_black: 0xFFFF_FFFF,
            korok_mark_anim_frame: 50,
            korok_mark_anim_white: 0x0000_00FF,
            korok_mark_anim_black: 0xFF40_0000,
            field_0xf2: 192,
            korok_mark_white: 0x0081_00FF,
            korok_mark_black: 0x0040_0000,
            field_0xfb: 128,
            area_text_color: 0x8832_00FF,
            area_text_color_main: 0xF000_00FF,
            field_0x104: 7,
            field_0x105: 2,
            field_0x106: 5,
            warp_back_color_cycle: 20,
            warp_back_color1: 0xFFFF_00FF,
            warp_back_color2: 0x8080_80FF,
            fishman_open_frame: 10,
            fishman_zoom1_frame: 7,
            field_0x112: 7,
            fishman_zoom1_out_frame: 7,
            fishman_zoom2_frame: 7,
            fishman_zoom2_alpha_frame: 30,
            fishman_map_write_wait: 30,
            fishman_map_write_frame: 10,
            fishman_blink_frame: 2,
            fishman_disp_wait: 30,
            fishman_close_wait: 0,
            fishman_close_frame: 10,
            moon_alpha_frame: 20,
            moon_alpha_max: 255,
            moon_alpha_min: 32,
        }
    }
}

impl Tuning {
    pub fn name(&self) -> &'static str {
        TUNING_NAME
    }

    pub fn no(&self) -> i8 {
        self.no
    }
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SaveState {
    pub disp_mode: u8,
    pub zoom: u8,
    pub zoom_grid_x: i8,
    pub zoom_grid_y: i8,
    pub cur_x: i8,
    pub cur_y: i8,
    pub cur_wx: i8,
    pub cur_wy: i8,
    pub active: u8,
    pub cmap_sel_no: i8,
    pub cur_hx: i8,
    pub cur_hy: i8,
}

impl Default for SaveState {
    fn default() -> Self {
        Self {
            disp_mode: 0,
            zoom: 0,
            zoom_grid_x: -10,
            zoom_grid_y: -10,
            cur_x: -10,
            cur_y: -10,
            cur_wx: -10,
            cur_wy: -10,
            active: 0,
            cmap_sel_no: -1,
            cur_hx: -10,
            cur_hy: -10,
        }
    }
}

impl SaveState {
    pub fn get_disp_mode(&self) -> u8 {
        self.disp_mode
    }

    pub fn set_disp_mode(&mut self, val: u8) {
        self.disp_mode = val;
    }

    pub fn get_zoom(&self) -> u8 {
        self.zoom
    }

    pub fn set_zoom(&mut self, val: u8) {
        self.zoom = val;
    }

    pub fn get_zoom_grid_x(&self) -> i8 {
        self.zoom_grid_x
    }

    pub fn set_zoom_grid_x(&mut self, val: i8) {
        self.zoom_grid_x = val;
    }

    pub fn get_zoom_grid_y(&self) -> i8 {
        self.zoom_grid_y
    }

    pub fn set_zoom_grid_y(&mut self, val: i8) {
        self.zoom_grid_y = val;
    }

    pub fn get_cur_x(&self) -> i8 {
        self.cur_x
    }

    pub fn set_cur_x(&mut self, val: i8) {
        self.cur_x = val;
    }

    pub fn get_cur_y(&self) -> i8 {
        self.cur_y
    }

    pub fn set_cur_y(&mut self, val: i8) {
        self.cur_y = val;
    }

    pub fn get_cur_wx(&self) -> i8 {
        self.cur_wx
    }

    pub fn set_cur_wx(&mut self, val: i8) {
        self.cur_wx = val;
    }

    pub fn get_cur_wy(&self) -> i8 {
        self.cur_wy
    }

    pub fn set_cur_wy(&mut self, val: i8) {
        self.cur_wy = val;
    }

    pub fn get_cur_hx(&self) -> i8 {
        self.cur_hx
    }

    pub fn set_cur_hx(&mut self, val: i8) {
        self.cur_hx = val;
    }

    pub fn get_cur_hy(&self) -> i8 {
        self.cur_hy
    }

    pub fn set_cur_hy(&mut self, val: i8) {
        self.cur_hy = val;
    }

    pub fn get_active(&self) -> u8 {
        self.active
    }

    pub fn set_active(&mut self, val: u8) {
        self.active = val;
    }

    pub fn get_cmap_sel_no(&self) -> i8 {
        self.cmap_sel_no
    }

    pub fn set_cmap_sel_no(&mut self, val: i8) {
        self.cmap_sel_no = val;
    }
}

pub use crate::j2d::F2;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PadInput {
    pub trig_a: bool,
    pub trig_b: bool,
    pub trig_x: bool,
    pub trig_y: bool,
    pub trig_z: bool,
    pub trig_l: bool,
    pub trig_r: bool,
    pub trig_start: bool,
    pub trig_left: bool,
    pub trig_down: bool,
    pub stick_value: f32,
    pub stick_angle: i16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MapEnv {
    pub player_x: f32,
    pub player_y: f32,
    pub player_angle_y: i16,
    pub has_ship: bool,
    pub ship_x: f32,
    pub ship_y: f32,
    pub ship_angle_y: i16,
    pub ship_ride: bool,
    pub start_stage: &'static str,
    pub ikada_ship_before_room_id: u8,
    pub has_stage_map_info: bool,
    pub stage_ocean_x: i8,
    pub stage_ocean_z: i8,
    pub wind_x: f32,
    pub wind_y: f32,
    pub wind_power: f32,
    pub opt_ruby: bool,
    pub ghost_ship_alpha: bool,
    pub ghost_ship_reg: u8,
    pub game_time: f32,
    pub moon_type: u8,
    pub random_salvage_point: u8,
}

#[derive(Clone, Debug, Default)]
pub struct MapOutput {
    pub sounds: Vec<u32>,
    pub bgms: Vec<u32>,
    pub arrive_grids: Vec<i32>,
    pub ship_warp_pos: Vec<i32>,
}

impl MapOutput {
    pub fn reset(&mut self) {
        self.sounds.clear();
        self.bgms.clear();
        self.arrive_grids.clear();
        self.ship_warp_pos.clear();
    }

    pub fn sound(&mut self, id: u32) {
        self.sounds.push(id);
    }

    pub fn bgm(&mut self, id: u32) {
        self.bgms.push(id);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SalvagePoint {
    pub field_0x0: i16,
    pub field_0x2: i16,
    pub x: i16,
    pub y: i16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChartPoint {
    pub grid_no: i8,
    pub collect_map_no: i8,
    pub cmap_no: i8,
    pub map_type: i8,
    pub field_0x4: i16,
    pub salvage_pnt: [SalvagePoint; 4],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IslandMark {
    pub sector_x: i8,
    pub sector_y: i8,
    pub field_0x2: i16,
    pub field_0x4: i16,
    pub field_0x6: u8,
}

pub trait ChartTable {
    fn num(&self) -> usize;
    fn at(&self, index: usize) -> ChartPoint;
}

pub trait IslandMarkTable {
    fn count(&self) -> usize;
    fn at(&self, index: usize) -> IslandMark;
}

pub trait TextureSource {
    fn has(&self, name: &str) -> bool;
}

pub trait TextResolver {
    fn message(&self, msg_no: u32) -> String;
    fn select_message(&self, msg_no: u32) -> (String, String, String, String);
}

pub trait CompareScreen {
    fn set_save(&mut self, save: &SaveState);
    fn create(&mut self, chart: &dyn ChartTable);
    fn initialize(&mut self, marks: &dyn IslandMarkTable);
    fn display_init(&mut self);
    fn open(&mut self) -> bool;
    fn move_step(&mut self);
    fn close(&mut self) -> bool;
    fn draw(&mut self) -> Vec<MapPane>;
    fn delete(&mut self);
    fn lock_b_button(&self) -> bool;
    fn button_icon_mode(&self) -> i32;
}

pub fn chart_index_by_grid<T: ChartTable + ?Sized>(table: &T, grid_idx: i32) -> Option<usize> {
    let num = table.num();
    let mut index = 0usize;
    while index < num {
        if i32::from(table.at(index).grid_no) == grid_idx + 1 {
            return Some(index);
        }
        index += 1;
    }
    None
}

pub fn chart_point_by_grid<T: ChartTable + ?Sized>(table: &T, grid_idx: i32) -> Option<ChartPoint> {
    chart_index_by_grid(table, grid_idx).map(|index| table.at(index))
}

pub fn check_point_use_grid<T: IslandMarkTable + ?Sized>(table: &T, x: i8, y: i8) -> i32 {
    let count = table.count();
    let mut index = 0usize;
    while index < count {
        let mark = table.at(index);
        if mark.sector_x == x && mark.sector_y == y {
            return index as i32;
        }
        index += 1;
    }
    -1
}

pub fn mark_prm<T: IslandMarkTable + ?Sized>(
    table: &T,
    index: i32,
) -> Option<(i8, i8, i16, i16, u8)> {
    if index < 0 {
        return None;
    }
    let mark = table.at(index as usize);
    Some((
        mark.sector_x,
        mark.sector_y,
        mark.field_0x2,
        mark.field_0x4,
        mark.field_0x6,
    ))
}

pub fn point_to_grid(x: f32, z: f32) -> (i8, i8) {
    let gx = (x / WORLD_UNIT + 0.5).floor() as i32;
    let gy = (z / WORLD_UNIT + 0.5).floor() as i32;
    (
        gx.clamp(i32::from(GRID_MIN), i32::from(GRID_MAX)) as i8,
        gy.clamp(i32::from(GRID_MIN), i32::from(GRID_MAX)) as i8,
    )
}

pub fn value_increase(max: i32, value: i32, mode: u8) -> f32 {
    if max <= 0 {
        return 1.0;
    }
    let v = value.clamp(0, max) as f32 / max as f32;
    match mode {
        0 => v * v,
        1 => v.sqrt(),
        3 => (v * 2.0 - 1.0) * 2.0 - 1.0,
        4 => {
            let s = (v * 0x8000_u16 as f32 * 0.5).sin();
            s * s
        }
        5 => {
            let s = (v * 0xFFFF_u16 as f32 * 0.5).sin();
            s * s
        }
        _ => v,
    }
}

pub fn line_inter(a: u8, b: u8, t: f32) -> u8 {
    (f32::from(a) + (f32::from(b) - f32::from(a)) * t) as u8
}

pub fn line_inter_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn mix_color(from: u32, to: u32, t: f32) -> u32 {
    let r = line_inter((from >> 24) as u8, (to >> 24) as u8, t);
    let g = line_inter((from >> 16) as u8, (to >> 16) as u8, t);
    let b = line_inter((from >> 8) as u8, (to >> 8) as u8, t);
    let a = line_inter(from as u8, to as u8, t);
    (u32::from(r) << 24) | (u32::from(g) << 16) | (u32::from(b) << 8) | u32::from(a)
}

pub fn grid_index(x: i8, y: i8) -> i32 {
    i32::from(x) + (i32::from(y) + 3) * GRID_WIDTH + 3
}

pub fn grid_x_of(grid: i32) -> i8 {
    ((grid - 3) % GRID_WIDTH) as i8
}

pub fn grid_y_of(grid: i32) -> i8 {
    (((grid - 3) / GRID_WIDTH) - 3) as i8
}

pub fn rupy_num_texture(value: i32) -> String {
    format!("rupy_num_{value:02}.bti")
}

pub fn island_texture(grid: i32) -> String {
    format!("R{:02X}_b.bti", grid + 1)
}

pub fn salvage_item_texture(map_type: i8) -> Option<&'static str> {
    if map_type < 0 {
        return None;
    }
    SALVAGE_ITEM_TEX.get(map_type as usize).copied()
}

pub fn child_pane_move_sp(pane: &mut MapPane, parent: &MapPane, scale: f32, x: f32, y: f32) {
    let x_size = parent.size.x / 2.0;
    let y_size = parent.size.y / 2.0;
    pane.pos_center.x = x_size + scale * (pane.pos_center_orig.x - parent.size_orig.x / 2.0);
    pane.pos_center.y = y_size + scale * (pane.pos_center_orig.y - parent.size_orig.y / 2.0);
    pane.pane_scale(x, y);
}

pub fn set_island_pos(base: &MapPane, pane: &mut MapPane, x: f32, y: f32) {
    pane.pos_center_orig.x = base.size_orig.x / 2.0 + x;
    pane.pos_center_orig.y = base.size_orig.y / 2.0 + y;
    pane.pos_center = pane.pos_center_orig;
    pane.cpos_move();
}

pub fn zoom_map_alpha_set(grid_x: i8, grid_y: i8, pane: &mut MapPane, alpha: u8) {
    if (i32::from(grid_x) + 3) + (i32::from(grid_y) + 3) * GRID_WIDTH == ISLE_IDX_TOWER_OF_THE_GODS
    {
        pane.init_alpha = alpha;
    } else {
        pane.init_alpha = 0xFF;
    }
    pane.init_alpha = 0xFF;
    pane.set_alpha();
}

pub fn pane_alpha_fmap_cursor(
    panes: &mut [MapPane],
    frame: i16,
    max: u8,
    mask: u8,
    flag: i32,
) -> bool {
    if frame < 0 {
        return false;
    }
    if frame > i16::from(max) {
        return true;
    }
    let mut alpha = value_increase(i32::from(max), i32::from(frame), mask);
    if flag != 2 {
        if flag == 1 {
            alpha = 1.0 - alpha;
        }
        for pane in panes.iter_mut() {
            pane.set_now_alpha(alpha);
            pane.set_alpha();
        }
    }
    false
}

pub fn pane_trance_warp_msg(
    pane: &mut MapPane,
    frame: i16,
    max: u8,
    start_y: f32,
    end_y: f32,
    mode: u8,
    flag: i32,
) -> bool {
    if frame < 0 {
        return false;
    }
    if frame > i16::from(max) {
        return true;
    }
    let mut alpha = value_increase(i32::from(max), i32::from(frame), mode);
    let y_adj = alpha * (end_y - start_y);
    pane.pane_trans(0.0, start_y + y_adj);
    if flag != 2 {
        if flag == 1 {
            alpha = 1.0 - alpha;
        }
        pane.set_now_alpha(alpha);
        pane.set_alpha();
    }
    false
}

pub fn mix_rgb(from: u32, to: u32, t: f32) -> u32 {
    (mix_color(from, to, t) & 0xFFFF_FF00) | (to & 0xFF)
}

pub fn is_main_island(grid: i32) -> bool {
    grid == ISLE_IDX_FORSAKEN_FORTRESS
        || grid == ISLE_IDX_GALE_ISLE
        || grid == ISLE_IDX_WINDFALL
        || grid == ISLE_IDX_DRAGON_ROOST
        || grid == ISLE_IDX_FIRE_MOUNTAIN
        || grid == ISLE_IDX_GREATFISH_ISLE
        || grid == ISLE_IDX_TOWER_OF_THE_GODS
        || grid == ISLE_IDX_ICE_RING_ISLE
        || grid == ISLE_IDX_FOREST_HAVEN
        || grid == ISLE_IDX_OUTSET_ISLAND
        || grid == ISLE_IDX_HEADSTONE_ISLAND
}

pub fn warp_area_grid_x(no: i32) -> i8 {
    if no < 0 {
        return 0;
    }
    CURSOR_TABLE
        .get(no as usize)
        .map(|slot| slot.grid_x)
        .unwrap_or(0)
}

pub fn warp_area_grid_y(no: i32) -> i8 {
    if no < 0 {
        return 0;
    }
    CURSOR_TABLE
        .get(no as usize)
        .map(|slot| slot.grid_y)
        .unwrap_or(0)
}

pub fn warp_area_no(table: Option<usize>) -> i32 {
    match table {
        Some(index) => i32::from(CURSOR_TABLE[index].no),
        None => -1,
    }
}

pub fn warp_area_no_up(table: Option<usize>) -> i32 {
    match table {
        Some(index) => i32::from(CURSOR_TABLE[index].up),
        None => -1,
    }
}

pub fn warp_area_no_down(table: Option<usize>) -> i32 {
    match table {
        Some(index) => i32::from(CURSOR_TABLE[index].down),
        None => -1,
    }
}

pub fn warp_area_no_left(table: Option<usize>) -> i32 {
    match table {
        Some(index) => i32::from(CURSOR_TABLE[index].left),
        None => -1,
    }
}

pub fn warp_area_no_right(table: Option<usize>) -> i32 {
    match table {
        Some(index) => i32::from(CURSOR_TABLE[index].right),
        None => -1,
    }
}

pub fn atan2s_deg(y: f32, x: f32) -> i16 {
    (y.atan2(x) * 32768.0 / std::f32::consts::PI) as i16
}

#[derive(Clone, Debug)]
pub struct MapPane {
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
    pub influenced_alpha: bool,
    pub texture: Option<String>,
    pub white: u32,
    pub black: u32,
    pub corner_tl: u32,
    pub corner_tr: u32,
    pub corner_bl: u32,
    pub corner_br: u32,
    pub rotation: f32,
    pub rotate_x: f32,
    pub rotate_y: f32,
    pub text: String,
    pub msg_no: Option<u32>,
    pub font_size_x: f32,
    pub font_size_y: f32,
    pub char_space: f32,
    pub line_space: f32,
    pub char_color: u32,
    pub font: Option<&'static str>,
    pub ruby_font: Option<&'static str>,
}

impl Default for MapPane {
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
            alpha: 0xFF,
            user_area: 0,
            influenced_alpha: true,
            texture: None,
            white: FULL_COLOR,
            black: NO_COLOR,
            corner_tl: FULL_COLOR,
            corner_tr: FULL_COLOR,
            corner_bl: FULL_COLOR,
            corner_br: FULL_COLOR,
            rotation: 0.0,
            rotate_x: 0.0,
            rotate_y: 0.0,
            text: String::new(),
            msg_no: None,
            font_size_x: 0.0,
            font_size_y: 0.0,
            char_space: 0.0,
            line_space: 0.0,
            char_color: FULL_COLOR,
            font: None,
            ruby_font: None,
        }
    }
}

impl MapPane {
    pub fn new(tag: &'static str) -> Self {
        Self {
            tag,
            ..Default::default()
        }
    }

    pub fn bind(&mut self, layout: &ScreenPane) {
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
        self.alpha = layout.alpha;
        self.pos_top_left = self.pos_top_left_orig;
        self.pos_center = self.pos_center_orig;
        self.size = self.size_orig;
        self.now_alpha = 0;
        self.user_area = 0;
        self.rotation = layout.rotation as f32;
        self.texture = None;
        self.text = layout.text.clone();
        self.msg_no = None;
        self.font_size_x = f32::from(layout.font_size_x);
        self.font_size_y = f32::from(layout.font_size_y);
        self.char_space = f32::from(layout.char_space);
        self.line_space = f32::from(layout.line_space);
    }

    pub fn lookup(&mut self, screen: &Screen) {
        if let Some(layout) = screen.find(self.tag) {
            self.bind(layout);
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

    pub fn set_influenced_alpha(&mut self, on: bool) {
        self.influenced_alpha = on;
    }

    pub fn change_texture(&mut self, texture: &str) {
        self.texture = Some(texture.to_owned());
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

    pub fn pane_scale(&mut self, scale_x: f32, scale_y: f32) {
        self.size.x = self.size_orig.x * scale_x;
        self.size.y = self.size_orig.y * scale_y;
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

    pub fn set_alpha(&mut self) {
        self.alpha = self.now_alpha;
    }

    pub fn set_white(&mut self, color: u32) {
        self.white = color;
    }

    pub fn set_black(&mut self, color: u32) {
        self.black = color;
    }

    pub fn set_black_white(&mut self, black: u32, white: u32) {
        self.black = black;
        self.white = white;
    }

    pub fn set_corner_color(&mut self, tl: u32, tr: u32, bl: u32, br: u32) {
        self.corner_tl = tl;
        self.corner_tr = tr;
        self.corner_bl = bl;
        self.corner_br = br;
    }

    pub fn rotate(&mut self, x: f32, y: f32, angle: f32) {
        self.rotate_x = x;
        self.rotate_y = y;
        self.rotation = angle;
    }

    pub fn set_string(&mut self, text: &str) {
        self.text = text.to_owned();
        self.msg_no = None;
    }

    pub fn set_message(&mut self, msg_no: u32, text: Option<&str>) {
        self.msg_no = Some(msg_no);
        self.text = text.unwrap_or_default().to_owned();
    }

    pub fn clear_text(&mut self) {
        self.text.clear();
        self.msg_no = None;
    }

    pub fn set_char_color(&mut self, color: u32) {
        self.char_color = color;
    }

    pub fn set_font(&mut self, font: &'static str) {
        self.font = Some(font);
    }

    pub fn set_font_size(&mut self, x: f32, y: f32) {
        self.font_size_x = x;
        self.font_size_y = y;
    }

    pub fn set_line_space(&mut self, space: f32) {
        self.line_space = space;
    }
}

#[derive(Clone, Debug, Default)]
pub struct OutFontBox {
    pub box_pane: &'static str,
    pub pic_panes: [&'static str; 3],
    pub ruby_pane: Option<&'static str>,
    pub ruby_font: Option<&'static str>,
    pub font: Option<&'static str>,
    pub message_no: u32,
    pub left_up_x: f32,
    pub left_up_y: f32,
    pub alpha: u8,
    pub icon_num: u8,
    pub icon_no: [u8; 3],
    pub tick: i32,
}

impl OutFontBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_pane(
        &mut self,
        font: &'static str,
        box_pane: &'static str,
        pic1: &'static str,
        pic2: &'static str,
        pic3: &'static str,
    ) {
        self.font = Some(font);
        self.box_pane = box_pane;
        self.pic_panes = [pic1, pic2, pic3];
        self.ruby_pane = None;
    }

    pub fn set_pane_ex(
        &mut self,
        font: &'static str,
        box_pane: &'static str,
        pic1: Option<&'static str>,
        pic2: Option<&'static str>,
        pic3: Option<&'static str>,
    ) {
        self.font = Some(font);
        self.box_pane = box_pane;
        self.pic_panes = [pic1.unwrap_or(""), pic2.unwrap_or(""), pic3.unwrap_or("")];
        self.ruby_pane = None;
    }

    pub fn set_ruby_ex(&mut self, ruby_font: &'static str, ruby_pane: &'static str) {
        self.ruby_font = Some(ruby_font);
        self.ruby_pane = Some(ruby_pane);
    }

    pub fn message_set(&mut self, msg_no: u32) {
        self.message_no = msg_no;
    }

    pub fn set_left_up_pos(&mut self, x: f32, y: f32) {
        self.left_up_x = x;
        self.left_up_y = y;
    }

    pub fn set_alpha(&mut self, alpha: u8) {
        self.alpha = alpha;
    }

    pub fn move_step(&mut self) {
        self.tick += 1;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WarpEmitter {
    pub id: u16,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub scale_z: f32,
    pub alive: bool,
    pub drawing: bool,
}

macro_rules! push_mut {
    ($base:tt, $list:ident, $($field:ident),* $(,)?) => {
        $( $list.push(&mut $base.$field); )*
    };
}

macro_rules! extend_mut {
    ($base:tt, $list:ident, $($field:ident),* $(,)?) => {
        $( $list.extend($base.$field.iter_mut()); )*
    };
}

macro_rules! push_ref {
    ($base:tt, $list:ident, $($field:ident),* $(,)?) => {
        $( $list.push(&$base.$field); )*
    };
}

macro_rules! extend_ref {
    ($base:tt, $list:ident, $($field:ident),* $(,)?) => {
        $( $list.extend($base.$field.iter()); )*
    };
}

#[derive(Clone, Debug)]
pub struct MapPanes {
    pub fddm: MapPane,
    pub cl: MapPane,
    pub area: MapPane,
    pub ci22: MapPane,
    pub ci21: MapPane,
    pub ci32: MapPane,
    pub ci31: MapPane,
    pub gti: [MapPane; 2],
    pub kk1: [MapPane; 8],
    pub kk3: [MapPane; 8],
    pub kkdm: MapPane,
    pub smsk: MapPane,
    pub lnk1: MapPane,
    pub spi1: MapPane,
    pub lnk2: MapPane,
    pub spi2: MapPane,
    pub lnk3: MapPane,
    pub spi3: MapPane,
    pub sc1: [MapPane; 9],
    pub sc2: [MapPane; 7],
    pub fmx: [MapPane; 5],
    pub fmn: [MapPane; 15],
    pub tsw1: MapPane,
    pub r01b: MapPane,
    pub r01g: MapPane,
    pub stm1: MapPane,
    pub stl1: MapPane,
    pub stx: [MapPane; 49],
    pub mr01: MapPane,
    pub ktx1: MapPane,
    pub ktx2: MapPane,
    pub clb: MapPane,
    pub clb2: MapPane,
    pub clg: MapPane,
    pub ck1: [MapPane; 3],
    pub ck2: [MapPane; 3],
    pub ck3: [MapPane; 2],
    pub kr0: [MapPane; 8],
    pub r0x: [MapPane; 11],
    pub wnd1: MapPane,
    pub wnd2: MapPane,
    pub area_txt: [MapPane; 3],
    pub bt00: MapPane,
    pub bt01: MapPane,
    pub bt02: MapPane,
    pub wts1: MapPane,
    pub cc01: MapPane,
    pub ys01: MapPane,
    pub ys00: MapPane,
    pub ysk0: MapPane,
    pub yes: MapPane,
    pub no01: MapPane,
    pub no00: MapPane,
    pub nok0: MapPane,
    pub no: MapPane,
    pub cur1: MapPane,
    pub wt0: MapPane,
    pub wt1: MapPane,
    pub gs: [MapPane; 7],
}

impl Default for MapPanes {
    fn default() -> Self {
        Self {
            fddm: MapPane::default(),
            cl: MapPane::default(),
            area: MapPane::default(),
            ci22: MapPane::default(),
            ci21: MapPane::default(),
            ci32: MapPane::default(),
            ci31: MapPane::default(),
            gti: std::array::from_fn(|_| MapPane::default()),
            kk1: std::array::from_fn(|_| MapPane::default()),
            kk3: std::array::from_fn(|_| MapPane::default()),
            kkdm: MapPane::default(),
            smsk: MapPane::default(),
            lnk1: MapPane::default(),
            spi1: MapPane::default(),
            lnk2: MapPane::default(),
            spi2: MapPane::default(),
            lnk3: MapPane::default(),
            spi3: MapPane::default(),
            sc1: std::array::from_fn(|_| MapPane::default()),
            sc2: std::array::from_fn(|_| MapPane::default()),
            fmx: std::array::from_fn(|_| MapPane::default()),
            fmn: std::array::from_fn(|_| MapPane::default()),
            tsw1: MapPane::default(),
            r01b: MapPane::default(),
            r01g: MapPane::default(),
            stm1: MapPane::default(),
            stl1: MapPane::default(),
            stx: std::array::from_fn(|_| MapPane::default()),
            mr01: MapPane::default(),
            ktx1: MapPane::default(),
            ktx2: MapPane::default(),
            clb: MapPane::default(),
            clb2: MapPane::default(),
            clg: MapPane::default(),
            ck1: std::array::from_fn(|_| MapPane::default()),
            ck2: std::array::from_fn(|_| MapPane::default()),
            ck3: std::array::from_fn(|_| MapPane::default()),
            kr0: std::array::from_fn(|_| MapPane::default()),
            r0x: std::array::from_fn(|_| MapPane::default()),
            wnd1: MapPane::default(),
            wnd2: MapPane::default(),
            area_txt: std::array::from_fn(|_| MapPane::default()),
            bt00: MapPane::default(),
            bt01: MapPane::default(),
            bt02: MapPane::default(),
            wts1: MapPane::default(),
            cc01: MapPane::default(),
            ys01: MapPane::default(),
            ys00: MapPane::default(),
            ysk0: MapPane::default(),
            yes: MapPane::default(),
            no01: MapPane::default(),
            no00: MapPane::default(),
            nok0: MapPane::default(),
            no: MapPane::default(),
            cur1: MapPane::default(),
            wt0: MapPane::default(),
            wt1: MapPane::default(),
            gs: std::array::from_fn(|_| MapPane::default()),
        }
    }
}

impl MapPanes {
    pub fn new() -> Self {
        let mut panes = Self {
            fddm: MapPane::new("fddm"),
            cl: MapPane::new("cl"),
            area: MapPane::new("area"),
            ci22: MapPane::new("ci22"),
            ci21: MapPane::new("ci21"),
            ci32: MapPane::new("ci32"),
            ci31: MapPane::new("ci31"),
            gti: [MapPane::new("GTI1"), MapPane::new("GTI2")],
            kkdm: MapPane::new("kkdm"),
            smsk: MapPane::new("smsk"),
            lnk1: MapPane::new("lnk1"),
            spi1: MapPane::new("spi1"),
            lnk2: MapPane::new("lnk2"),
            spi2: MapPane::new("spi2"),
            lnk3: MapPane::new("lnk3"),
            spi3: MapPane::new("spi3"),
            tsw1: MapPane::new("tsw1"),
            r01b: MapPane::new("R01B"),
            r01g: MapPane::new("R01G"),
            stm1: MapPane::new("STM1"),
            stl1: MapPane::new("STL1"),
            mr01: MapPane::new(AREA_TAGS[0]),
            ktx1: MapPane::new("ktx1"),
            ktx2: MapPane::new("ktx2"),
            clb: MapPane::new("clb"),
            clb2: MapPane::new("clb2"),
            clg: MapPane::new("clg"),
            wnd1: MapPane::new("wnd1"),
            wnd2: MapPane::new("wnd2"),
            bt00: MapPane::new("bt00"),
            bt01: MapPane::new("bt01"),
            bt02: MapPane::new("bt02"),
            wts1: MapPane::new("wts1"),
            cc01: MapPane::new("cc01"),
            ys01: MapPane::new("ys01"),
            ys00: MapPane::new("ys00"),
            ysk0: MapPane::new("ysk0"),
            yes: MapPane::new("yes"),
            no01: MapPane::new("no01"),
            no00: MapPane::new("no00"),
            nok0: MapPane::new("nok0"),
            no: MapPane::new("no"),
            cur1: MapPane::new("cur1"),
            wt0: MapPane::new("wt0"),
            wt1: MapPane::new("wt1"),
            ..Default::default()
        };
        for (index, tag) in SELECT_CURSOR_TABLE.iter().enumerate() {
            panes.kk1[index] = MapPane::new(tag);
        }
        for (index, tag) in ZOOM_CURSOR_TABLE.iter().enumerate() {
            panes.kk3[index] = MapPane::new(tag);
        }
        for (index, tag) in WORLD_MARK_TABLE.iter().enumerate() {
            panes.sc1[index] = MapPane::new(tag);
        }
        for (index, tag) in DETAIL_MARK_TABLE.iter().enumerate() {
            panes.sc2[index] = MapPane::new(tag);
        }
        for (index, tag) in DETAIL_FRAME_TABLE.iter().enumerate() {
            panes.fmx[index] = MapPane::new(tag);
        }
        for (index, tag) in DETAIL_AREA_TABLE.iter().enumerate() {
            panes.fmn[index] = MapPane::new(tag);
        }
        for (index, tag) in PEARL_TABLE_A.iter().enumerate() {
            panes.ck1[index] = MapPane::new(tag);
        }
        for (index, tag) in PEARL_TABLE_B.iter().enumerate() {
            panes.ck2[index] = MapPane::new(tag);
        }
        for (index, tag) in PEARL_TABLE_C.iter().enumerate() {
            panes.ck3[index] = MapPane::new(tag);
        }
        for (index, tag) in SECRET_TABLE.iter().enumerate() {
            panes.kr0[index] = MapPane::new(tag);
        }
        for (index, tag) in AREA_TEXT_TABLE.iter().enumerate() {
            panes.area_txt[index] = MapPane::new(tag);
        }
        for (index, tag) in GHOST_SHIP_TABLE.iter().enumerate() {
            panes.gs[index] = MapPane::new(tag);
        }
        for (index, tag) in SALVAGE_MARK_TAGS.iter().enumerate() {
            panes.stx[index] = MapPane::new(tag);
        }
        for (index, tag) in ISLAND_PANE_TABLE.iter().enumerate() {
            panes.r0x[index] = MapPane::new(tag);
        }
        panes
    }

    pub fn bind(&mut self, screen: &Screen) {
        for pane in self.all_mut() {
            pane.lookup(screen);
        }
    }

    pub fn all_mut(&mut self) -> Vec<&mut MapPane> {
        let mut list: Vec<&mut MapPane> = Vec::new();
        push_mut!(
            self, list, fddm, cl, area, ci22, ci21, ci32, ci31, kkdm, smsk, lnk1, spi1, lnk2, spi2,
            lnk3, spi3, tsw1, r01b, r01g, stm1, stl1, mr01, ktx1, ktx2, clb, clb2, clg, wnd1, wnd2,
            bt00, bt01, bt02, wts1, cc01, ys01, ys00, ysk0, yes, no01, no00, nok0, no, cur1, wt0,
            wt1,
        );
        extend_mut!(
            self, list, gti, kk1, kk3, sc1, sc2, fmx, fmn, stx, ck1, ck2, ck3, kr0, r0x
        );
        extend_mut!(self, list, area_txt, gs);
        list
    }

    pub fn all(&self) -> Vec<&MapPane> {
        let mut list: Vec<&MapPane> = Vec::new();
        push_ref!(
            self, list, fddm, cl, area, ci22, ci21, ci32, ci31, kkdm, smsk, lnk1, spi1, lnk2, spi2,
            lnk3, spi3, tsw1, r01b, r01g, stm1, stl1, mr01, ktx1, ktx2, clb, clb2, clg, wnd1, wnd2,
            bt00, bt01, bt02, wts1, cc01, ys01, ys00, ysk0, yes, no01, no00, nok0, no, cur1, wt0,
            wt1,
        );
        extend_ref!(
            self, list, gti, kk1, kk3, sc1, sc2, fmx, fmn, stx, ck1, ck2, ck3, kr0, r0x
        );
        extend_ref!(self, list, area_txt, gs);
        list
    }

    pub fn by_tag_mut(&mut self, tag: &str) -> Option<&mut MapPane> {
        self.all_mut().into_iter().find(|pane| pane.tag == tag)
    }

    pub fn by_tag(&self, tag: &str) -> Option<&MapPane> {
        self.all().into_iter().find(|pane| pane.tag == tag)
    }
}

pub struct SaveView<'a> {
    pub arrivals: &'a Arrivals,
    pub charts: &'a Charts,
    pub events: &'a EventFlags,
    pub regs: &'a StoryBits,
    pub collect: &'a Collect,
    pub mem: &'a MemBits,
    pub stage_boss_enemy_et: bool,
    pub stage_boss_enemy_wt: bool,
    pub has_bottle_forest_water: bool,
}

impl<'a> SaveView<'a> {
    pub fn new(
        arrivals: &'a Arrivals,
        charts: &'a Charts,
        events: &'a EventFlags,
        regs: &'a StoryBits,
        collect: &'a Collect,
        mem: &'a MemBits,
    ) -> Self {
        Self {
            arrivals,
            charts,
            events,
            regs,
            collect,
            mem,
            stage_boss_enemy_et: false,
            stage_boss_enemy_wt: false,
            has_bottle_forest_water: false,
        }
    }

    pub fn is_save_arrive_grid(&self, no: i32) -> bool {
        if no < 0 {
            return false;
        }
        self.arrivals.is_arrived(no as usize)
    }

    pub fn is_get_collect_map(&self, no: i32) -> bool {
        no >= 1 && self.charts.has((no - 1) as usize)
    }

    pub fn is_open_collect_map(&self, no: i32) -> bool {
        no >= 1 && self.charts.is_open((no - 1) as usize)
    }

    pub fn is_complete_collect_map(&self, no: i32) -> bool {
        no >= 1 && self.charts.is_done((no - 1) as usize)
    }

    pub fn is_event_bit(&self, flag: u32) -> bool {
        self.events.test(flag)
    }

    pub fn event_reg(&self, id: u16) -> u8 {
        self.regs.reg(id)
    }

    pub fn is_symbol(&self, no: u8) -> bool {
        self.collect.is_symbol(no)
    }

    pub fn is_stage_boss_et(&self) -> bool {
        self.stage_boss_enemy_et
    }

    pub fn is_stage_boss_wt(&self) -> bool {
        self.stage_boss_enemy_wt
    }

    pub fn has_bottle(&self, item: u8) -> bool {
        item == ITEM_FOREST_WATER && self.has_bottle_forest_water
    }

    pub fn is_stage_boss_enemy(&self) -> bool {
        self.mem.is_stage_boss_enemy()
    }

    pub fn collect_counts(&self) -> (i32, i32) {
        let mut num = 0;
        let mut complete = 0;
        let mut i = 1;
        while i <= 51 {
            if i != 35 && i != 36 {
                if self.is_get_collect_map(i) {
                    num += 1;
                }
                if self.is_complete_collect_map(i) {
                    complete += 1;
                }
            }
            i += 1;
        }
        (num, complete)
    }
}

pub struct MapContext<'a> {
    pub flags: SaveView<'a>,
    pub text: &'a dyn TextResolver,
    pub marks: &'a dyn IslandMarkTable,
    pub textures: &'a dyn TextureSource,
    pub env: MapEnv,
    pub pad: PadInput,
    pub out: &'a mut MapOutput,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StickControl {
    pub max_value: f32,
    pub min_value: f32,
    pub bits: u8,
    pub prev_bits: u8,
    pub repeat_count: i16,
    pub vertical_wait: i16,
    pub first_wait: i16,
    pub second_wait: i16,
    pub step: i16,
    pub horizontal_wait: i16,
    pub vertical_rest: i16,
    pub min_step: i16,
    pub repeat_left: i16,
    pub repeat_left_rest: i16,
    pub angle_base: i16,
    pub angle_limit: i16,
    pub half_dead: i16,
}

impl Default for StickControl {
    fn default() -> Self {
        Self::new()
    }
}

impl StickControl {
    pub fn new() -> Self {
        Self {
            max_value: STICK_MAX_VALUE,
            min_value: STICK_MIN_VALUE,
            bits: 0,
            prev_bits: 0,
            repeat_count: 0,
            vertical_wait: 0,
            first_wait: STICK_FIRST_WAIT,
            second_wait: STICK_SECOND_WAIT,
            step: STICK_STEP,
            horizontal_wait: STICK_FIRST_WAIT,
            vertical_rest: STICK_FIRST_WAIT,
            min_step: STICK_MIN_STEP,
            repeat_left: STICK_MIN_STEP,
            repeat_left_rest: STICK_MIN_STEP,
            angle_base: 0,
            angle_limit: 0,
            half_dead: STICK_HALF_DEAD,
        }
    }

    pub fn set_wait_parm(
        &mut self,
        first: i16,
        second: i16,
        step: i16,
        min_step: i16,
        max_value: f32,
        min_value: f32,
        angle_limit: i16,
        half_dead: i16,
    ) {
        self.first_wait = first;
        self.second_wait = second;
        self.step = step;
        self.min_step = min_step;
        self.max_value = max_value;
        self.min_value = min_value;
        self.angle_limit = angle_limit;
        self.half_dead = half_dead;
    }

    pub fn init(&mut self) {
        self.repeat_count = 0;
        self.vertical_wait = 0;
        self.prev_bits = 0;
        self.bits = 0;
        self.repeat_left_rest = 0;
        self.horizontal_wait = self.first_wait;
        self.vertical_rest = self.first_wait;
        self.repeat_left = self.min_step;
        self.repeat_left_rest = self.min_step;
    }

    pub fn x_init(&mut self) {
        self.repeat_count = 0;
        self.bits &= 0xFC;
        self.horizontal_wait = self.first_wait;
        self.repeat_left = self.min_step;
    }

    pub fn y_init(&mut self) {
        self.vertical_wait = 0;
        self.bits &= !0x0C;
        self.vertical_rest = self.first_wait;
        self.repeat_left_rest = self.min_step;
    }

    pub fn check_trigger(&mut self, value: f32, angle: i16) -> u8 {
        self.prev_bits = self.bits;
        let mut bits = 0u8;
        if value != 0.0 {
            let half = (0x2000 - self.half_dead) >> 1;
            let base = self.angle_base;
            if angle < base - 0x7000 + half {
                bits |= 4;
            } else if angle < base - 0x5000 - half {
                bits |= 4 | 1;
            } else if angle < base - 0x3000 + half {
                bits |= 1;
            } else if angle < base - 0x1000 - half {
                bits |= 1 | 8;
            } else if angle < base + 0x1000 + half {
                bits |= 8;
            } else if angle < base + 0x3000 - half {
                bits |= 8 | 2;
            } else if angle < base + 0x5000 - half {
                bits |= 2;
            } else if angle < base + 0x7000 - half {
                bits |= 2 | 4;
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
                    self.angle_base = self.angle_limit;
                } else {
                    self.angle_base = -self.angle_limit;
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

    pub fn check_left_trigger(&mut self) -> bool {
        if self.repeat_count == 0 && self.bits & 1 != 0 {
            self.repeat_count = self.horizontal_wait;
            if self.repeat_left == 0 {
                self.horizontal_wait -= self.step;
                if self.horizontal_wait < self.second_wait {
                    self.horizontal_wait = self.second_wait;
                }
            } else {
                self.repeat_left -= 1;
            }
            return true;
        }
        false
    }

    pub fn check_right_trigger(&mut self) -> bool {
        if self.repeat_count == 0 && self.bits & 2 != 0 {
            self.repeat_count = self.horizontal_wait;
            if self.repeat_left == 0 {
                self.horizontal_wait -= self.step;
                if self.horizontal_wait < self.second_wait {
                    self.horizontal_wait = self.second_wait;
                }
            } else {
                self.repeat_left -= 1;
            }
            return true;
        }
        false
    }

    pub fn check_up_trigger(&mut self) -> bool {
        if self.vertical_wait == 0 && self.bits & 4 != 0 {
            self.vertical_wait = self.vertical_rest;
            if self.repeat_left_rest == 0 {
                self.vertical_rest -= self.step;
                if self.vertical_rest < self.second_wait {
                    self.vertical_rest = self.second_wait;
                }
            } else {
                self.repeat_left_rest -= 1;
            }
            return true;
        }
        false
    }

    pub fn check_down_trigger(&mut self) -> bool {
        if self.vertical_wait == 0 && self.bits & 8 != 0 {
            self.vertical_wait = self.vertical_rest;
            if self.repeat_left_rest == 0 {
                self.vertical_rest -= self.step;
                if self.vertical_rest < self.second_wait {
                    self.vertical_rest = self.second_wait;
                }
            } else {
                self.repeat_left_rest -= 1;
            }
            return true;
        }
        false
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayerPos {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Debug, Default)]
pub struct MapDraw {
    pub visible: bool,
    pub compare_visible: bool,
    pub panes: Vec<MapPane>,
    pub compare_panes: Vec<MapPane>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapTearDown {
    pub release_screen_archive: bool,
    pub release_texture_archive: bool,
    pub release_tuning: bool,
}

pub struct SeaMapScreen {
    pub tuning: Tuning,
    pub save: SaveState,
    pub panes: MapPanes,
    pub area_panes: [MapPane; 49],
    pub chart: Option<Rc<dyn ChartTable>>,
    pub compare: Option<Box<dyn CompareScreen>>,
    pub out_font: OutFontBox,
    pub out_font2: OutFontBox,
    pub stick: StickControl,
    pub fonts: [&'static str; 2],
    pub warp_back_emitter: WarpEmitter,
    pub warp_spot_emitters: [WarpEmitter; WARP_SPOT_COUNT],
    pub screen_loaded: bool,
    pub check_mark_texture: Option<String>,
    pub ruby_offset_applied: bool,
    pub map_close: bool,
    pub select_cursor_buf_idx: u8,
    pub cursor_buf_idx: u8,
    pub player_point_timer: u8,
    pub player_point_toggle: u8,
    pub cur_chk_pnt_no: i8,
    pub player_chk_pnt_no: i8,
    pub main_proc_idx: u8,
    pub fmap_proc_idx: u8,
    pub compare_proc_idx: u8,
    pub frame_timer: i16,
    pub grid_x: i8,
    pub grid_y: i8,
    pub target_grid_x: i8,
    pub target_grid_y: i8,
    pub player_pos: PlayerPos,
    pub ship_pos: PlayerPos,
    pub disp_island_pos_x: f32,
    pub disp_island_pos_y: f32,
    pub full_map_mode: bool,
    pub salv_itm_buf_idx: u8,
    pub salv_itm_timer: u8,
    pub salv_itm_changing: bool,
    pub area_txt_buf_idx: u8,
    pub area_txt_timer: u8,
    pub area_txt_changing: bool,
    pub warp_anim_timer: u8,
    pub warp_blink_toggle: u8,
    pub txt_name: [String; 3],
    pub txt_name_msg: [Option<u32>; 3],
    pub txt_note: [String; 2],
    pub txt_note_msg: [Option<u32>; 2],
    pub txt_dummy: [String; 2],
    pub warp_scroll_guard: bool,
    pub warp_anim_active: bool,
    pub black_alpha: u8,
    pub white_alpha: u8,
    pub map_select_active: bool,
    pub zoom_locked: bool,
    pub input_disabled: bool,
    pub fmap_mode: u8,
    pub warp_proc_idx: u8,
    pub warp_sub_state: u8,
    pub frame_counter: u8,
    pub fishman_proc_idx: u8,
    pub fishman_timer1: u8,
    pub fishman_timer2: u8,
    pub fishman_timer3: u8,
    pub check_mark_timer: u8,
    pub check_mark_toggle: u8,
    pub fishman_active: bool,
    pub salvage_pnt_idx: u8,
    pub button_icon_mode: u8,
    pub dsp_large_map_link: bool,
    pub dsp_huge_map_link: bool,
    pub dsp_large_map_ship: bool,
    pub dsp_huge_map_ship: bool,
    pub ship_open_water: bool,
    pub ck1_color1: u32,
    pub ck1_color2: u32,
    pub kr0_color1: u32,
    pub kr0_color2: u32,
    pub ys01_color1: u32,
    pub ys01_color2: u32,
    pub moon_alpha_active: bool,
    pub moon_alpha_state1: bool,
    pub moon_alpha_state2: bool,
    pub moon_alpha_timer: u8,
    pub moon_alpha_toggle: u8,
    pub moon_type: u8,
}

impl Default for SeaMapScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl SeaMapScreen {
    pub fn new() -> Self {
        Self {
            tuning: Tuning::default(),
            save: SaveState::default(),
            panes: MapPanes::new(),
            area_panes: std::array::from_fn(|index| MapPane::new(AREA_TAGS[index])),
            chart: None,
            compare: None,
            out_font: OutFontBox::new(),
            out_font2: OutFontBox::new(),
            stick: StickControl::new(),
            fonts: ["", ""],
            warp_back_emitter: WarpEmitter::default(),
            warp_spot_emitters: [WarpEmitter::default(); WARP_SPOT_COUNT],
            screen_loaded: false,
            check_mark_texture: None,
            ruby_offset_applied: false,
            map_close: false,
            select_cursor_buf_idx: 0,
            cursor_buf_idx: 0,
            player_point_timer: 0,
            player_point_toggle: 0,
            cur_chk_pnt_no: 0,
            player_chk_pnt_no: 0,
            main_proc_idx: MAIN_PROC_FMAP,
            fmap_proc_idx: ZOOM_PROC_SELECT_GRID,
            compare_proc_idx: HIKAKU_PROC_OPEN,
            frame_timer: 0,
            grid_x: 0,
            grid_y: 0,
            target_grid_x: 0,
            target_grid_y: 0,
            player_pos: PlayerPos::default(),
            ship_pos: PlayerPos::default(),
            disp_island_pos_x: 0.0,
            disp_island_pos_y: 0.0,
            full_map_mode: false,
            salv_itm_buf_idx: 0,
            salv_itm_timer: 0,
            salv_itm_changing: false,
            area_txt_buf_idx: 0,
            area_txt_timer: 0,
            area_txt_changing: false,
            warp_anim_timer: 0,
            warp_blink_toggle: 0,
            txt_name: [String::new(), String::new(), String::new()],
            txt_name_msg: [None, None, None],
            txt_note: [String::new(), String::new()],
            txt_note_msg: [None, None],
            txt_dummy: [String::new(), String::new()],
            warp_scroll_guard: false,
            warp_anim_active: false,
            black_alpha: 0,
            white_alpha: 0,
            map_select_active: false,
            zoom_locked: false,
            input_disabled: false,
            fmap_mode: MODE_NORMAL,
            warp_proc_idx: WARP_PROC_MOVE,
            warp_sub_state: WARP_SEL_YES,
            frame_counter: 0,
            fishman_proc_idx: FISHMAN_PROC_DISP_AREA,
            fishman_timer1: 0,
            fishman_timer2: 0,
            fishman_timer3: 0,
            check_mark_timer: 0,
            check_mark_toggle: 0,
            fishman_active: false,
            salvage_pnt_idx: 0,
            button_icon_mode: BTN_ICON_WORLD,
            dsp_large_map_link: false,
            dsp_huge_map_link: false,
            dsp_large_map_ship: false,
            dsp_huge_map_ship: false,
            ship_open_water: false,
            ck1_color1: NO_COLOR,
            ck1_color2: FULL_COLOR,
            kr0_color1: NO_COLOR,
            kr0_color2: FULL_COLOR,
            ys01_color1: NO_COLOR,
            ys01_color2: FULL_COLOR,
            moon_alpha_active: false,
            moon_alpha_state1: false,
            moon_alpha_state2: false,
            moon_alpha_timer: 0,
            moon_alpha_toggle: 0,
            moon_type: 0,
        }
    }

    pub fn set_chart(&mut self, chart: Rc<dyn ChartTable>) {
        self.chart = Some(chart);
    }

    pub fn set_compare(&mut self, compare: Box<dyn CompareScreen>) {
        self.compare = Some(compare);
    }

    pub fn set_font(&mut self, font: &'static str, ruby_font: &'static str) {
        self.fonts = [font, ruby_font];
    }

    pub fn set_text_area(&mut self, name0: &'static str, name1: &'static str) {
        self.txt_name[0] = name0.to_owned();
        self.txt_name[1] = name1.to_owned();
        self.txt_name[2] = name0.to_owned();
        self.txt_name[0].clear();
        self.txt_name[1].clear();
        self.txt_name[2].clear();
    }

    pub fn set_note_area(&mut self, note0: &'static str, note1: &'static str) {
        self.txt_note[0] = note0.to_owned();
        self.txt_note[1] = note1.to_owned();
        self.txt_note[0].clear();
        self.txt_note[1].clear();
    }

    pub fn set_dummy_area(&mut self, dummy0: &'static str, dummy1: &'static str) {
        self.txt_dummy[0] = dummy0.to_owned();
        self.txt_dummy[1] = dummy1.to_owned();
        self.txt_dummy[0].clear();
        self.txt_dummy[1].clear();
    }

    pub fn stop_wrap_back_emitter(&mut self) {
        self.warp_back_emitter.alive = false;
    }

    pub fn stop_wrap_spot_emitter(&mut self, index: usize) {
        if let Some(emitter) = self.warp_spot_emitters.get_mut(index) {
            emitter.alive = false;
        }
    }

    pub fn chart_point(&self, grid: i32) -> Option<ChartPoint> {
        let chart = self.chart.clone()?;
        chart_point_by_grid(chart.as_ref(), grid)
    }

    pub fn create(&mut self, screen: &Screen, ctx: &mut MapContext<'_>) {
        self.salvage_pnt_idx = ctx.env.random_salvage_point;
        self.screen_loaded = !screen.panes.is_empty();
        self.screen_set(screen, ctx.env.opt_ruby);
        self.stick = StickControl::new();
        self.stick.set_wait_parm(
            STICK_FIRST_WAIT,
            STICK_SECOND_WAIT,
            STICK_STEP,
            STICK_MIN_STEP,
            STICK_MAX_VALUE,
            STICK_MIN_VALUE,
            0,
            0x800,
        );
        self.initialize(ctx);
        self.display_init(ctx);
        self.phantom_ship_check(ctx);
        self.map_close = false;
        self.fmap_mode = MODE_NORMAL;
    }

    pub fn phantom_ship_check(&self, ctx: &mut MapContext<'_>) {
        if ctx.env.ghost_ship_alpha {
            ctx.out.sound(SE_YUUREISEN_SONG);
        }
    }

    pub fn screen_set(&mut self, screen: &Screen, opt_ruby: bool) {
        self.panes.bind(screen);
        for index in 0..AREA_TAGS.len() {
            self.area_panes[index].lookup(screen);
        }

        self.black_alpha = self.panes.area.black as u8;
        self.white_alpha = self.panes.area.white as u8;

        self.ck1_color1 = self.panes.ck1[0].black;
        self.ck1_color2 = self.panes.ck1[0].white;
        self.kr0_color1 = self.panes.kr0[0].black;
        self.kr0_color2 = self.panes.kr0[0].white;

        let blank: String = "A".repeat(TEXT_BUFFER_LEN - 1);

        self.panes.ktx1.set_font(self.fonts[0]);
        self.panes.ktx2.set_font(self.fonts[1]);
        self.panes.ktx1.set_string(&blank);
        self.panes.ktx2.set_string(&blank);

        self.out_font2
            .set_pane_ex(self.fonts[0], "ktx1", None, None, None);
        self.out_font2.set_ruby_ex(self.fonts[1], "ktx2");
        self.out_font2.message_set(MSG_UNKNOWN_NAME);
        let x = self.panes.ktx1.pos_top_left.x;
        let y = self.panes.ktx1.pos_top_left.y;
        self.out_font2.set_left_up_pos(x, y);

        if opt_ruby {
            self.panes.ktx1.pos_center_orig.y -= 4.0;
            self.panes.ktx1.pos_center.y = self.panes.ktx1.pos_center_orig.y;
            self.panes.ktx1.cpos_move();
            self.panes.ktx1.pos_top_left_orig.y = self.panes.ktx1.pos_top_left.y;
            self.ruby_offset_applied = true;
        }

        for index in 0..3usize {
            self.panes.area_txt[index].set_font(self.fonts[0]);
            self.panes.area_txt[index].set_string(&blank);
            self.panes.area_txt[index].clear_text();
            self.panes.area_txt[index].show();
        }

        self.out_font
            .set_pane(self.fonts[0], "txt2", "bt00", "bt01", "bt02");

        self.panes.yes.set_font(self.fonts[0]);
        self.panes.no.set_font(self.fonts[0]);
        self.panes.wt0.set_font(self.fonts[0]);
        self.panes.wt1.set_font(self.fonts[1]);

        self.ys01_color2 = self.panes.ys01.white;
        self.ys01_color1 = self.panes.ys01.black;

        self.panes.wt0.set_black(WARP_YES_SHADOW);
        self.panes.wt0.set_white(FULL_COLOR);
    }

    pub fn initialize(&mut self, ctx: &mut MapContext<'_>) {
        self.frame_timer = 0;
        self.input_disabled = false;
        self.player_pos.x = ctx.env.player_x;
        self.player_pos.y = ctx.env.player_y;
        self.player_pos.z =
            f32::from(ctx.env.player_angle_y.wrapping_add(i16::MIN)) * 180.0 / 32768.0;

        if ctx.env.has_ship {
            self.ship_pos.x = ctx.env.ship_x;
            self.ship_pos.y = ctx.env.ship_y;
            self.ship_pos.z =
                f32::from(ctx.env.ship_angle_y.wrapping_add(i16::MIN)) * 180.0 / 32768.0;
        } else {
            self.ship_pos.x = 0.0;
            self.ship_pos.y = 0.0;
        }

        if ctx.env.start_stage == "sea" {
            self.fishman_active = true;
            let (x, y) = point_to_grid(self.player_pos.x, self.player_pos.y);
            self.grid_x = x;
            self.grid_y = y;
            let (sx, sy) = point_to_grid(self.ship_pos.x, self.ship_pos.y);
            self.target_grid_x = sx;
            self.target_grid_y = sy;
        } else {
            self.fishman_active = false;
            if ctx.env.start_stage == "Obshop"
                || ctx.env.start_stage == "Abship"
                || ctx.env.start_stage == "PShip"
            {
                let room = ctx.env.ikada_ship_before_room_id;
                self.grid_x = (i32::from(room) % 7 - 3) as i8;
                self.grid_y = (i32::from(room) / 7 - 3) as i8;
            } else if ctx.env.has_stage_map_info {
                self.grid_x = ctx.env.stage_ocean_x;
                if self.grid_x == 4 {
                    self.grid_x = 0;
                }
                self.grid_y = ctx.env.stage_ocean_z;
                if self.grid_y == 4 {
                    self.grid_y = 0;
                }
            } else {
                self.grid_x = 0;
                self.grid_y = 0;
            }
            self.target_grid_x = self.grid_x;
            self.target_grid_y = self.grid_y;
        }

        self.player_chk_pnt_no = check_point_use_grid(ctx.marks, self.grid_x, self.grid_y) as i8;
        self.panes.wnd1.user_area = 0;
        self.check_mark_anime_init();
        self.area_txt_buf_idx = 0;
        self.area_txt_timer = 0;
        self.area_txt_changing = false;
        self.salv_itm_buf_idx = 0;
        self.salv_itm_timer = 0;
        self.salv_itm_changing = false;

        if self.save.get_cur_x() < -3 || self.save.get_cur_y() < -3 {
            self.save.set_cur_x(self.grid_x);
            self.save.set_cur_y(self.grid_y);
        }

        if self.save.get_zoom_grid_x() < -3 || self.save.get_zoom_grid_y() < -3 {
            self.save.set_zoom_grid_x(self.save.get_cur_x());
            self.save.set_zoom_grid_y(self.save.get_cur_y());
        }

        match self.save.get_disp_mode() {
            DISP_NORMAL => {
                self.save.set_cur_hx(self.save.get_cur_x());
                self.save.set_cur_hy(self.save.get_cur_y());
                self.main_proc_idx = MAIN_PROC_FMAP;
                self.zoom_locked = false;
            }
            _ => {
                self.main_proc_idx = MAIN_PROC_HIKAKU;
                self.zoom_locked = true;
            }
        }

        self.compare_proc_idx = HIKAKU_PROC_OPEN;
        self.pane_trans_base(
            0,
            self.tuning.base_anim_frame,
            self.tuning.open_pos_y,
            0.0,
            0,
            0,
        );
        self.warp_scroll_guard = false;
        self.warp_anim_active = false;
        self.save.set_zoom(ZOOM_WORLD);
        self.sel_grid_mask_alpha_ctrl(0, self.tuning.zoom_anim_frame, 0, 0);
        self.fmap_mask_alpha_ctrl(0, self.tuning.zoom_anim_frame, 0, 0);
        self.pane_trance_zoom_map(
            0,
            self.tuning.zoom_anim_frame,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            1.0,
            0,
            0,
        );
        self.pane_trance_zoom2_map(
            0,
            self.tuning.zoom_anim_frame,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            1.0,
            0,
            0,
        );
        pane_alpha_fmap_cursor(
            &mut self.panes.kk3,
            0,
            self.tuning.select_cursor_alpha_frame,
            0,
            0,
        );
        self.fmap_proc_idx = ZOOM_PROC_SELECT_GRID;
    }

    pub fn display_init(&mut self, ctx: &mut MapContext<'_>) {
        for index in 0..GRID_COUNT as usize {
            self.set_area_on_off(index, !ctx.flags.is_save_arrive_grid(index as i32));
        }

        for index in 0..ISLAND_ROOM_TABLE.len() {
            if ctx.flags.is_save_arrive_grid(ISLAND_ROOM_TABLE[index] - 1) {
                self.panes.r0x[index].show();
            } else {
                self.panes.r0x[index].hide();
            }
        }

        self.disp_end_salvage_mark(ctx);
        let size_x = self.panes.area.size_orig.x;
        let size_y = self.panes.area.size_orig.y;
        self.panes.area.pane_trans(
            f32::from(self.grid_x) * size_x,
            f32::from(self.grid_y) * size_y,
        );
        self.set_dsp_normal_map_link(ctx);
        self.set_dsp_large_map_link(ctx);
        self.sel_cursor_init();
        self.sel_cursor_move();
        self.panes.area_txt[1].set_now_alpha(0.0);
        self.panes.area_txt[1].set_alpha();
        self.change_island_name(self.area_txt_buf_idx, ctx);
        self.panes.gti[0].set_now_alpha(1.0);
        self.panes.gti[0].set_alpha();
        self.panes.gti[1].set_now_alpha(0.0);
        self.panes.gti[1].set_alpha();
        self.change_salvage_get_item(self.salv_itm_buf_idx, ctx);
        self.player_point_grid_anime_init();
        self.check_mark_check1(ctx);
        self.check_mark_check2(ctx);
        self.check_mark_check3(ctx);
        self.set_dsp_wind_angle(ctx);
        self.calc_get_map_count(ctx);
        self.g_ship_mark_anime_init(ctx);
        let chart = self.chart.clone();
        if let Some(compare) = self.compare.as_mut() {
            if let Some(chart) = chart {
                compare.create(chart.as_ref());
            }
            compare.set_save(&self.save);
            compare.initialize(ctx.marks);
            compare.display_init();
        }
    }

    pub fn back_cloth_disp_init(&mut self) {
        self.panes.cl.hide();
    }

    pub fn calc_get_map_count(&mut self, ctx: &mut MapContext<'_>) {
        let (num_map, complete_map) = ctx.flags.collect_counts();
        let name = rupy_num_texture(complete_map / 10);
        self.panes.ci31.change_texture(&name);
        let name = rupy_num_texture(complete_map % 10);
        self.panes.ci32.change_texture(&name);
        let name = rupy_num_texture(num_map / 10);
        self.panes.ci21.change_texture(&name);
        let name = rupy_num_texture(num_map % 10);
        self.panes.ci22.change_texture(&name);
    }

    pub fn disp_end_salvage_mark(&mut self, ctx: &mut MapContext<'_>) {
        for index in 0..GRID_COUNT as usize {
            let Some(pnt) = self.chart_point(index as i32) else {
                self.panes.stx[index].hide();
                continue;
            };
            if !ctx
                .flags
                .is_complete_collect_map(i32::from(pnt.collect_map_no))
            {
                self.panes.stx[index].hide();
            } else {
                let base_x = self.area_panes[index].pos_center_orig.x;
                let base_y = self.area_panes[index].pos_center_orig.y;
                let salvage = pnt.salvage_pnt[self.salvage_pnt_idx as usize % 4];
                let x = f32::from(salvage.x);
                let y = f32::from(salvage.y);
                let pane = &mut self.panes.stx[index];
                pane.pos_center_orig.x = base_x + x * GRID_STEP / WORLD_UNIT;
                pane.pos_center_orig.y = base_y + y * GRID_STEP / WORLD_UNIT;
                pane.pos_center.x = pane.pos_center_orig.x;
                pane.pos_center.y = pane.pos_center_orig.y;
                pane.cpos_move();
                pane.show();
            }
        }
    }

    pub fn check_mark_check1(&mut self, ctx: &mut MapContext<'_>) {
        if !ctx.flags.is_event_bit(EVENT_PLACED_DINS_PEARL)
            || !ctx.flags.is_event_bit(EVENT_PLACED_FARORES_PEARL)
            || !ctx.flags.is_event_bit(EVENT_PLACED_NAYRUS_PEARL)
        {
            if ctx.flags.is_event_bit(EVENT_MET_FOREST_HUMAN) {
                self.panes.ck1[0].show();
            }
            if ctx.flags.is_symbol(crate::sim::flags::SYMBOL_DIN) {
                self.panes.ck1[0].hide();
                self.panes.ck1[2].show();
            }
            if ctx.flags.is_symbol(SYMBOL_FARORE) {
                self.panes.ck1[2].hide();
                self.panes.ck1[1].show();
            }
            if ctx.flags.is_event_bit(EVENT_UNKNOWN_1E80) {
                self.panes.ck1[1].hide();
            }

            if ctx.flags.is_event_bit(EVENT_JAB_IRIS_RETRIEVED) {
                if ctx.flags.is_event_bit(EVENT_PLACED_DINS_PEARL) {
                    self.panes.ck2[0].hide();
                } else {
                    self.panes.ck2[0].show();
                }
                if ctx.flags.is_event_bit(EVENT_PLACED_FARORES_PEARL) {
                    self.panes.ck2[2].hide();
                } else {
                    self.panes.ck2[2].show();
                }
                if ctx.flags.is_event_bit(EVENT_PLACED_NAYRUS_PEARL) {
                    self.panes.ck2[1].hide();
                } else {
                    self.panes.ck2[1].show();
                }
            }
        }
    }

    pub fn check_mark_check2(&mut self, ctx: &mut MapContext<'_>) {
        if ctx.flags.is_event_bit(EVENT_UNKNOWN_1820) {
            self.panes.ck3[0].show();
            self.panes.ck3[1].show();
            if ctx.flags.is_stage_boss_et() {
                self.panes.ck3[1].hide();
            }
            if ctx.flags.is_stage_boss_wt() {
                self.panes.ck3[0].hide();
            }
        }
    }

    pub fn check_mark_check3(&mut self, ctx: &mut MapContext<'_>) {
        if !ctx.flags.is_event_bit(EVENT_UNKNOWN_0102) && ctx.flags.is_event_bit(EVENT_UNKNOWN_3940)
        {
            for index in 0..8usize {
                let bit = index & 7;
                let stat = ctx.flags.is_event_bit(EVENT_UNKNOWN_0102)
                    || ((u32::from(ctx.flags.event_reg(EVENT_UNKNOWN_9EFF as u16)) & 0xFF) >> bit
                        & 1)
                        != 0;
                if stat {
                    self.panes.kr0[index].set_black_white(
                        self.tuning.korok_mark_black,
                        self.tuning.korok_mark_white,
                    );
                }
                self.panes.kr0[index].change_texture(KOROK_STATS[usize::from(stat)]);
                self.panes.kr0[index].show();
                if ctx.flags.has_bottle(ITEM_FOREST_WATER) {
                    self.panes.kr0[index].init_alpha = 0xC0;
                    self.panes.kr0[index].now_alpha = 0xC0;
                    self.panes.kr0[index].pane_scale_xy(1.2);
                } else {
                    self.panes.kr0[index].init_alpha = 0x30;
                    self.panes.kr0[index].now_alpha = 0x30;
                    self.panes.kr0[index].pane_scale_xy(1.0);
                }
                self.panes.kr0[index].set_alpha();
            }
        }
    }

    pub fn is_map_close(&self) -> bool {
        self.map_close
    }

    pub fn set_pane_on_off(&self, panes: &mut MapPanes, tag: &str, on: bool) {
        if let Some(pane) = panes.by_tag_mut(tag) {
            if on {
                pane.show();
            } else {
                pane.hide();
            }
        }
    }

    pub fn set_area_on_off(&mut self, index: usize, on: bool) {
        if let Some(pane) = self.area_panes.get_mut(index) {
            if on {
                pane.show();
            } else {
                pane.hide();
            }
        }
    }

    pub fn sel_grid_mask_alpha_ctrl(&mut self, value: i16, max: u8, mode: u8, flag: i32) -> bool {
        if value < 0 {
            return false;
        }
        if value > i16::from(max) {
            return true;
        }
        let mut alpha = value_increase(i32::from(max), i32::from(value), mode);
        if flag != 2 {
            if flag == 1 {
                alpha = 1.0 - alpha;
            }
            self.panes.kkdm.set_now_alpha(alpha);
            self.panes.kkdm.set_alpha();
        }
        false
    }

    pub fn fmap_mask_alpha_ctrl(&mut self, value: i16, max: u8, mode: u8, flag: i32) -> bool {
        if value < 0 {
            return false;
        }
        if value > i16::from(max) {
            return true;
        }
        let mut alpha = value_increase(i32::from(max), i32::from(value), mode);
        if flag != 2 {
            if flag == 1 {
                alpha = 1.0 - alpha;
            }
            self.panes.smsk.set_now_alpha(alpha);
            self.panes.smsk.set_alpha();
        }
        false
    }

    pub fn sel_cursor_init(&mut self) {
        self.select_cursor_buf_idx = 0;
        for index in 0..4usize {
            self.panes.kk1[index].show();
            self.panes.kk1[index + 4].hide();
        }
        self.panes.kk1[0].user_area = i16::from(self.tuning.cursor_flash_frame);
    }

    pub fn sel_cursor_hide(&mut self) {
        for index in 0..8usize {
            self.panes.kk1[index].hide();
        }
    }

    pub fn sel_cursor_move(&mut self) {
        let x = f32::from(self.save.get_cur_x()) * GRID_STEP;
        let y = f32::from(self.save.get_cur_y()) * GRID_STEP;
        for index in 0..8usize {
            self.panes.kk1[index].pane_trans(x, y);
        }
        self.panes.kkdm.pane_trans(x, y);
    }

    pub fn island_name_change(&mut self, ctx: &mut MapContext<'_>) {
        if self.area_txt_changing {
            self.area_txt_chg_fast();
        } else {
            self.area_txt_changing = true;
        }
        self.change_island_name(self.area_txt_buf_idx ^ 1, ctx);
    }

    pub fn change_island_name(&mut self, no: u8, ctx: &mut MapContext<'_>) {
        let wy = self.save.get_cur_y();
        let wx = self.save.get_cur_x();
        let grid = grid_index(wx, wy);
        let index = no as usize % 3;

        if !ctx.flags.is_save_arrive_grid(grid) {
            self.panes.area_txt[index].clear_text();
            self.txt_name[index].clear();
            self.txt_name_msg[index] = None;
        } else {
            let msg_no =
                if grid == ISLE_IDX_PRIVATE_OASIS && ctx.flags.is_event_bit(EVENT_UNKNOWN_2D80) {
                    MSG_PRIVATE_ISLAND
                } else {
                    grid as u32 + MSG_GRID_NAME_BASE
                };
            let color = if is_main_island(grid) {
                self.tuning.area_text_color_main
            } else {
                self.tuning.area_text_color
            };
            self.panes.area_txt[index].set_char_color(color);
            let text = ctx.text.message(msg_no);
            self.panes.area_txt[index].set_message(msg_no, Some(&text));
            self.txt_name[index] = text;
            self.txt_name_msg[index] = Some(msg_no);
        }
    }

    pub fn area_txt_chg(&mut self) {
        if self.area_txt_changing {
            let alpha = self.pane_alpha_area_txt(
                i32::from(self.area_txt_timer),
                self.tuning.area_text_alpha_frame,
                0,
            );
            self.area_txt_timer = self.area_txt_timer.wrapping_add(1);
            if alpha {
                self.area_txt_buf_idx ^= 1;
                self.area_txt_timer = 0;
                self.area_txt_changing = false;
            }
        }
    }

    pub fn area_txt_chg_fast(&mut self) {
        self.area_txt_timer = 0;
        let cur = self.area_txt_buf_idx as usize;
        self.panes.area_txt[cur ^ 1].set_now_alpha(1.0);
        self.panes.area_txt[cur ^ 1].set_alpha();
        self.panes.area_txt[cur].set_now_alpha(0.0);
        self.panes.area_txt[cur].set_alpha();
        self.area_txt_buf_idx ^= 1;
    }

    pub fn salvage_get_item_chg(&mut self) {
        if self.salv_itm_changing {
            let alpha = self.pane_alpha_selvage_item(
                i16::from(self.salv_itm_timer),
                self.tuning.area_text_alpha_frame,
            );
            self.salv_itm_timer = self.salv_itm_timer.wrapping_add(1);
            if alpha {
                self.salv_itm_buf_idx ^= 1;
                self.salv_itm_timer = 0;
                self.salv_itm_changing = false;
            }
        }
    }

    pub fn salvage_get_item_change(&mut self, ctx: &mut MapContext<'_>) {
        if self.salv_itm_changing {
            self.salv_itm_disp_chg_fast();
        } else {
            self.salv_itm_changing = true;
        }
        self.change_salvage_get_item(self.salv_itm_buf_idx ^ 1, ctx);
    }

    pub fn salv_itm_disp_chg_fast(&mut self) {
        self.salv_itm_timer = 0;
        let cur = self.salv_itm_buf_idx as usize;
        if self.full_map_mode {
            self.panes.gti[cur ^ 1].set_now_alpha(1.0);
        } else {
            self.panes.gti[cur ^ 1].set_now_alpha(0.0);
        }
        self.panes.gti[cur ^ 1].set_alpha();
        self.panes.gti[cur].set_now_alpha(0.0);
        self.panes.gti[cur].set_alpha();
        self.salv_itm_buf_idx ^= 1;
    }

    pub fn change_salvage_get_item(&mut self, no: u8, ctx: &mut MapContext<'_>) {
        let wy = self.save.get_cur_y();
        let wx = self.save.get_cur_x();
        let grid = grid_index(wx, wy);
        let index = no as usize % 2;
        let Some(pnt) = self.chart_point(grid) else {
            self.full_map_mode = false;
            self.panes.gti[index].set_now_alpha(0.0);
            self.panes.gti[index].set_alpha();
            return;
        };
        if ctx
            .flags
            .is_complete_collect_map(i32::from(pnt.collect_map_no))
        {
            self.full_map_mode = true;
            if let Some(texture) = salvage_item_texture(pnt.map_type) {
                self.panes.gti[index].change_texture(texture);
            }
        } else {
            self.full_map_mode = false;
            self.panes.gti[index].set_now_alpha(0.0);
            self.panes.gti[index].set_alpha();
        }
    }

    pub fn sel_cursor_anime(&mut self) {
        if self.panes.kk1[0].user_area == 0 {
            for index in 0..4usize {
                self.panes.kk1[index + (self.select_cursor_buf_idx ^ 1) as usize * 4].show();
                self.panes.kk1[index + self.select_cursor_buf_idx as usize * 4].hide();
            }
            self.select_cursor_buf_idx ^= 1;
            self.panes.kk1[0].user_area = i16::from(self.tuning.cursor_flash_frame);
        } else {
            self.panes.kk1[0].user_area -= 1;
        }
    }

    pub fn zoom_cursor_init(&mut self) {
        self.cursor_buf_idx = 0;
        for index in 0..4usize {
            self.panes.kk3[index].show();
            self.panes.kk3[index + 4].hide();
        }
        self.panes.kk3[0].user_area = i16::from(self.tuning.cursor_flash_frame);
    }

    pub fn zoom_cursor_anime(&mut self) {
        if self.panes.kk3[0].user_area == 0 {
            for index in 0..4usize {
                self.panes.kk3[index + (self.cursor_buf_idx ^ 1) as usize * 4].show();
                self.panes.kk3[index + self.cursor_buf_idx as usize * 4].hide();
            }
            self.cursor_buf_idx ^= 1;
            self.panes.kk3[0].user_area = i16::from(self.tuning.cursor_flash_frame);
        } else {
            self.panes.kk3[0].user_area -= 1;
        }
    }

    pub fn player_point_grid_anime_init(&mut self) {
        self.player_point_timer = self.tuning.player_point_period as u8;
        self.player_point_toggle = 0;
    }

    pub fn player_point_grid_anime(&mut self) {
        let mut alpha = value_increase(
            i32::from(self.tuning.player_point_period),
            i32::from(self.player_point_timer),
            2,
        );
        if self.player_point_toggle == 0 {
            alpha = 1.0 - alpha;
        }
        let mut black = mix_color(
            self.tuning.player_point_white2,
            self.tuning.player_point_black2,
            alpha,
        );
        black = (black & 0xFFFF_FF00) | u32::from(self.black_alpha);
        let mut white = mix_color(
            self.tuning.player_point_white1,
            self.tuning.player_point_black1,
            alpha,
        );
        white = (white & 0xFFFF_FF00) | u32::from(self.white_alpha);
        self.panes.area.set_white(white);
        self.panes.area.set_black(black);
        if self.player_point_timer == 0 {
            self.player_point_timer = self.tuning.player_point_period as u8;
            self.player_point_toggle ^= 1;
        } else {
            self.player_point_timer -= 1;
        }
    }

    pub fn set_dsp_wind_angle(&mut self, ctx: &mut MapContext<'_>) {
        let adj = atan2s_deg(ctx.env.wind_x, ctx.env.wind_y).wrapping_add(i16::MIN);
        let angle = f32::from(adj) * 180.0 / 32768.0 % 360.0;

        let x =
            (self.panes.wnd1.size_orig.x + f32::from(self.tuning.wind_arrow_pivot_x)) as i32 as f32;
        let y = (self.panes.wnd1.size_orig.y * 0.5 + f32::from(self.tuning.wind_arrow_pivot_y))
            as i32 as f32;
        self.panes.wnd1.rotate(x, y, angle);
        let y = (self.panes.wnd2.size_orig.y * 0.5 + f32::from(self.tuning.wind_arrow_pivot_y))
            as i32 as f32;
        self.panes
            .wnd2
            .rotate(f32::from(self.tuning.wind_arrow_pivot_x), y, angle);
    }

    pub fn wind_arrow_color_anime(&mut self, ctx: &mut MapContext<'_>) {
        let period =
            (f32::from(self.tuning.wind_arrow_period) - (ctx.env.wind_power * 10.0 + 0.5)) as i16;
        let hold_a = self.tuning.wind_arrow_hold1;
        let hold_b = self.tuning.wind_arrow_hold2;

        let cycle = hold_a + period * 2 + hold_b;
        self.panes.wnd1.user_area += 1;
        if self.panes.wnd1.user_area > cycle {
            self.panes.wnd1.user_area = 0;
        }

        let val = self.panes.wnd1.user_area;
        let color_a = if val < period {
            mix_color(
                self.tuning.wind_arrow_color2,
                self.tuning.wind_arrow_color1,
                value_increase(i32::from(period), i32::from(val), 0),
            )
        } else if val < period + hold_a {
            self.tuning.wind_arrow_color1
        } else if val < hold_a + period * 2 {
            mix_color(
                self.tuning.wind_arrow_color2,
                self.tuning.wind_arrow_color1,
                value_increase(i32::from(period), i32::from(hold_a + period * 2 - val), 0),
            )
        } else {
            self.tuning.wind_arrow_color2
        };

        let area = self.panes.wnd1.user_area;
        let color_b = if area < hold_a {
            self.tuning.wind_arrow_color1
        } else if area < period + hold_a {
            mix_color(
                self.tuning.wind_arrow_color2,
                self.tuning.wind_arrow_color1,
                value_increase(i32::from(period), i32::from(period + hold_a - area), 0),
            )
        } else if area < period + hold_a + hold_b {
            self.tuning.wind_arrow_color2
        } else {
            mix_color(
                self.tuning.wind_arrow_color2,
                self.tuning.wind_arrow_color1,
                value_increase(
                    i32::from(period),
                    i32::from(area - (period + hold_a + hold_b)),
                    0,
                ),
            )
        };

        self.panes
            .wnd1
            .set_corner_color(color_a, color_a, color_b, color_b);
        self.panes
            .wnd2
            .set_corner_color(color_a, color_a, color_b, color_b);
    }

    pub fn check_mark_anime_init(&mut self) {
        self.check_mark_timer = self.tuning.check_mark_anim_frame;
        self.check_mark_toggle = 1;
    }

    pub fn check_mark_anime(&mut self) {
        let mut alpha = value_increase(
            i32::from(self.tuning.check_mark_anim_frame),
            i32::from(self.check_mark_timer),
            2,
        );
        if self.check_mark_toggle == 0 {
            alpha = 1.0 - alpha;
        }

        let white = mix_rgb(self.tuning.check_mark_white, self.ck1_color2, alpha);
        let black = mix_rgb(self.tuning.check_mark_black, self.ck1_color1, alpha);

        if self.check_mark_timer == 0 {
            self.check_mark_timer = self.tuning.check_mark_anim_frame;
            self.check_mark_toggle ^= 1;
        } else {
            self.check_mark_timer -= 1;
        }

        for index in 0..3usize {
            if self.panes.ck1[index].is_visible() {
                self.panes.ck1[index].set_black_white(black, white);
            }
            if self.panes.ck2[index].is_visible() {
                self.panes.ck2[index].set_black_white(black, white);
            }
        }

        for index in 0..2usize {
            if self.panes.ck3[index].is_visible() {
                self.panes.ck3[index].set_black_white(black, white);
            }
        }
    }

    pub fn read_fmap_texture(&mut self, filename: &str, ctx: &mut MapContext<'_>) -> bool {
        if ctx.textures.has(filename) {
            self.check_mark_texture = Some(filename.to_owned());
            true
        } else {
            self.check_mark_texture = None;
            false
        }
    }

    pub fn set_disp_island_pos(&mut self, x: i8, y: i8, ctx: &mut MapContext<'_>) {
        let index = check_point_use_grid(ctx.marks, x, y);
        if index != -1 {
            let Some((gx, gy, px, py, prm)) = mark_prm(ctx.marks, index) else {
                self.panes.tsw1.hide();
                self.panes.r01b.hide();
                return;
            };
            let _ = (gx, gy, prm);
            let offset_x = f32::from(px) * ((self.panes.clb.size_orig.x - F2D_MARGIN) / WORLD_UNIT);
            let offset_y = f32::from(py) * ((self.panes.clb.size_orig.y - F2D_MARGIN) / WORLD_UNIT);
            self.disp_island_pos_x = f32::from(px);
            self.disp_island_pos_y = f32::from(py);
            let clb = self.panes.clb.clone();
            set_island_pos(&clb, &mut self.panes.r01b, offset_x, offset_y);
            set_island_pos(&clb, &mut self.panes.tsw1, offset_x, offset_y);
            self.change_fmap_texture(x, y, ctx);
            self.panes.tsw1.show();
            self.panes.r01b.show();
        } else {
            self.panes.tsw1.hide();
            self.panes.r01b.hide();
        }
    }

    pub fn change_fmap_texture(&mut self, x: i8, y: i8, ctx: &mut MapContext<'_>) {
        let grid = grid_index(x, y);
        let filename = island_texture(grid);
        if !self.read_fmap_texture(&filename, ctx) {
            self.panes.r01b.hide();
            self.panes.r01g.hide();
        } else if let Some(texture) = self.check_mark_texture.clone() {
            self.panes.r01b.change_texture(&texture);
            self.panes.r01g.change_texture(&texture);
        }
    }

    pub fn set_dsp_normal_map_link(&mut self, ctx: &mut MapContext<'_>) {
        if !self.fishman_active {
            self.panes.lnk1.hide();
            self.panes.spi1.hide();
        } else {
            let area_x = self.panes.area.pos_center_orig.x;
            let area_y = self.panes.area.pos_center_orig.y;
            self.panes.lnk1.pos_center_orig.x =
                area_x + (self.player_pos.x * GRID_STEP) / WORLD_UNIT;
            self.panes.lnk1.pos_center_orig.y =
                area_y + (self.player_pos.y * GRID_STEP) / WORLD_UNIT;
            self.panes.lnk1.pos_center.x = self.panes.lnk1.pos_center_orig.x;
            self.panes.lnk1.pos_center.y = self.panes.lnk1.pos_center_orig.y;
            self.panes.lnk1.cpos_move();
            let sx = (self.panes.lnk1.size.x * 0.5) as i32 as f32;
            let sy = (self.panes.lnk1.size.y * 0.5) as i32 as f32;
            let angle = self.player_pos.z;
            self.panes.lnk1.rotate(sx, sy, angle);

            if !ctx.env.ship_ride {
                self.panes.spi1.pos_center_orig.x =
                    area_x + (self.ship_pos.x * GRID_STEP) / WORLD_UNIT;
                self.panes.spi1.pos_center_orig.y =
                    area_y + (self.ship_pos.y * GRID_STEP) / WORLD_UNIT;
                self.panes.spi1.pos_center.x = self.panes.spi1.pos_center_orig.x;
                self.panes.spi1.pos_center.y = self.panes.spi1.pos_center_orig.y;
                self.panes.spi1.cpos_move();
                let hx = (self.panes.spi1.size.x * 0.5) as i32 as f32;
                let hy = (self.panes.spi1.size.y * 0.5) as i32 as f32;
                let angle = self.ship_pos.z;
                self.panes.spi1.rotate(hx, hy, angle);
            } else {
                self.panes.spi1.hide();
            }
        }
    }

    pub fn set_dsp_large_map_link(&mut self, ctx: &mut MapContext<'_>) {
        if self.fishman_active {
            let x = self.player_pos.x - WORLD_UNIT * f32::from(self.grid_x);
            let y = self.player_pos.y - WORLD_UNIT * f32::from(self.grid_y);

            let size_x = self.panes.clb.size_orig.x;
            self.panes.lnk2.pos_center_orig.x =
                size_x / 2.0 + x * ((size_x - F2D_MARGIN) / WORLD_UNIT);
            let size_y = self.panes.clb.size_orig.y;
            self.panes.lnk2.pos_center_orig.y =
                size_y / 2.0 + y * ((size_y - F2D_MARGIN) / WORLD_UNIT);
            self.panes.lnk2.pos_center.x = self.panes.lnk2.pos_center_orig.x;
            self.panes.lnk2.pos_center.y = self.panes.lnk2.pos_center_orig.y;
            self.panes.lnk2.cpos_move();

            if !ctx.env.ship_ride {
                let x = self.ship_pos.x - WORLD_UNIT * f32::from(self.target_grid_x);
                let y = self.ship_pos.y - WORLD_UNIT * f32::from(self.target_grid_y);
                let size_x = self.panes.clb.size_orig.x;
                self.panes.spi2.pos_center_orig.x =
                    size_x / 2.0 + x * ((size_x - F2D_MARGIN) / WORLD_UNIT);
                let size_y = self.panes.clb.size_orig.y;
                self.panes.spi2.pos_center_orig.y =
                    size_y / 2.0 + y * ((size_y - F2D_MARGIN) / WORLD_UNIT);
                self.panes.spi2.pos_center.x = self.panes.spi2.pos_center_orig.x;
                self.panes.spi2.pos_center.y = self.panes.spi2.pos_center_orig.y;
                self.panes.spi2.cpos_move();
            }
        }
    }

    pub fn check_dsp_large_map_link(&mut self, ctx: &mut MapContext<'_>) {
        if !self.fishman_active {
            self.panes.lnk2.hide();
            self.dsp_large_map_link = false;
        } else {
            self.cur_chk_pnt_no =
                check_point_use_grid(ctx.marks, self.save.get_cur_x(), self.save.get_cur_y()) as i8;
            if self.player_chk_pnt_no == self.cur_chk_pnt_no {
                self.dsp_large_map_link = true;
            } else {
                self.panes.lnk2.hide();
                self.dsp_large_map_link = false;
            }
        }
        self.disp_end_salvage_large_mark(ctx);
    }

    pub fn check_dsp_large_map_ship(&mut self, ctx: &mut MapContext<'_>) {
        if !self.fishman_active {
            self.panes.spi2.hide();
            self.dsp_large_map_ship = false;
        } else if !ctx.env.ship_ride
            && self.target_grid_x == self.save.get_cur_x()
            && self.target_grid_y == self.save.get_cur_y()
        {
            self.dsp_large_map_ship = true;
        } else {
            self.panes.spi2.hide();
            self.dsp_large_map_ship = false;
        }
    }

    pub fn disp_end_salvage_large_mark(&mut self, ctx: &mut MapContext<'_>) {
        let grid = grid_index(self.save.get_cur_x(), self.save.get_cur_y());
        let Some(pnt) = self.chart_point(grid) else {
            self.panes.stm1.hide();
            return;
        };

        if !ctx
            .flags
            .is_complete_collect_map(i32::from(pnt.collect_map_no))
        {
            self.panes.stm1.hide();
        } else {
            let salvage = pnt.salvage_pnt[self.salvage_pnt_idx as usize % 4];
            let x = f32::from(salvage.x);
            let y = f32::from(salvage.y);
            self.panes.stm1.pos_center_orig.x = self.panes.clb.size_orig.x / 2.0
                + x * ((self.panes.clb.size_orig.x - F2D_MARGIN) / WORLD_UNIT);
            self.panes.stm1.pos_center_orig.y = self.panes.clb.size_orig.y / 2.0
                + y * ((self.panes.clb.size_orig.y - F2D_MARGIN) / WORLD_UNIT);
            self.panes.stm1.pos_center.x = self.panes.stm1.pos_center_orig.x;
            self.panes.stm1.pos_center.y = self.panes.stm1.pos_center_orig.y;
            self.panes.stm1.cpos_move();
            self.panes.stm1.show();
        }
    }

    pub fn set_dsp_huge_map_link(&mut self, ctx: &mut MapContext<'_>) {
        let index = check_point_use_grid(
            ctx.marks,
            self.save.get_zoom_grid_x(),
            self.save.get_zoom_grid_y(),
        );
        if index == -1 {
            return;
        }
        let Some((gx, gy, a, b, prm)) = mark_prm(ctx.marks, index) else {
            return;
        };
        let _ = prm;
        if !self.fishman_active {
            self.panes.lnk3.hide();
            self.dsp_huge_map_link = false;
            self.disp_end_salvage_huge_mark(f32::from(a), f32::from(b), ctx);
        } else {
            let island_x = f32::from(a) + WORLD_UNIT * f32::from(gx);
            let island_y = f32::from(b) + WORLD_UNIT * f32::from(gy);
            let dx = self.player_pos.x - island_x;
            let dy = self.player_pos.y - island_y;

            self.map_select_active = dx <= ISLAND_MARK_RANGE
                && dx >= -ISLAND_MARK_RANGE
                && dy <= ISLAND_MARK_RANGE
                && dy >= -ISLAND_MARK_RANGE;

            let base_x = self.panes.r01g.pos_top_left_orig.x + self.panes.r01g.size_orig.x / 2.0;
            let base_y = self.panes.r01g.pos_top_left_orig.y + self.panes.r01g.size_orig.y / 2.0;
            self.panes.lnk3.pos_center_orig.x =
                base_x + dx * (self.panes.r01g.size_orig.x / 20000.0);
            self.panes.lnk3.pos_center_orig.y =
                base_y + dy * (self.panes.r01g.size_orig.y / 20000.0);
            self.panes.lnk3.pos_center.x = self.panes.lnk3.pos_center_orig.x;
            self.panes.lnk3.pos_center.y = self.panes.lnk3.pos_center_orig.y;
            self.panes.lnk3.cpos_move();

            if !ctx.env.ship_ride {
                let ship_dx = self.ship_pos.x - island_x;
                let ship_dy = self.ship_pos.y - island_y;

                if ship_dx <= ISLAND_MARK_RANGE
                    && ship_dx >= -ISLAND_MARK_RANGE
                    && ship_dy <= ISLAND_MARK_RANGE
                    && ship_dy >= -ISLAND_MARK_RANGE
                {
                    self.dsp_huge_map_ship = true;
                    self.ship_open_water = false;
                } else {
                    self.dsp_huge_map_ship = false;
                    self.ship_open_water = true;
                    self.panes.spi3.hide();
                }

                self.panes.spi3.pos_center_orig.x =
                    base_x + ship_dx * (self.panes.r01g.size_orig.x / 20000.0);
                self.panes.spi3.pos_center_orig.y =
                    base_y + ship_dy * (self.panes.r01g.size_orig.y / 20000.0);
                self.panes.spi3.pos_center.x = self.panes.spi3.pos_center_orig.x;
                self.panes.spi3.pos_center.y = self.panes.spi3.pos_center_orig.y;
                self.panes.spi3.cpos_move();
            }

            self.disp_end_salvage_huge_mark(f32::from(a), f32::from(b), ctx);
        }
    }

    pub fn disp_end_salvage_huge_mark(&mut self, x: f32, y: f32, ctx: &mut MapContext<'_>) {
        let grid = grid_index(self.save.get_cur_x(), self.save.get_cur_y());
        let Some(pnt) = self.chart_point(grid) else {
            self.panes.stl1.hide();
            return;
        };

        if !ctx
            .flags
            .is_complete_collect_map(i32::from(pnt.collect_map_no))
        {
            self.panes.stl1.hide();
        } else {
            let salvage = pnt.salvage_pnt[self.salvage_pnt_idx as usize % 4];
            let dx = f32::from(salvage.x) - x;
            let dy = f32::from(salvage.y) - y;
            if dx <= ISLAND_MARK_RANGE
                && dx >= -ISLAND_MARK_RANGE
                && dy <= ISLAND_MARK_RANGE
                && dy >= -ISLAND_MARK_RANGE
            {
                self.panes.stl1.pos_center_orig.x = self.panes.r01g.pos_top_left_orig.x
                    + self.panes.r01g.size_orig.x / 2.0
                    + dx * (self.panes.r01g.size_orig.x / 20000.0);
                self.panes.stl1.pos_center_orig.y = self.panes.r01g.pos_top_left_orig.y
                    + self.panes.r01g.size_orig.y / 2.0
                    + dy * (self.panes.r01g.size_orig.y / 20000.0);
                self.panes.stl1.pos_center.x = self.panes.stl1.pos_center_orig.x;
                self.panes.stl1.pos_center.y = self.panes.stl1.pos_center_orig.y;
                self.panes.stl1.cpos_move();
                self.panes.stl1.show();
            } else {
                self.panes.stl1.hide();
            }
        }
    }

    pub fn check_dsp_huge_map_link(&mut self, ctx: &mut MapContext<'_>) {
        if !self.fishman_active {
            self.panes.lnk3.hide();
        } else {
            self.cur_chk_pnt_no =
                check_point_use_grid(ctx.marks, self.save.get_cur_x(), self.save.get_cur_y()) as i8;
            if self.player_chk_pnt_no == self.cur_chk_pnt_no && self.map_select_active {
                self.dsp_huge_map_link = true;
            } else {
                self.panes.lnk3.hide();
                self.dsp_huge_map_link = false;
            }
        }
    }

    pub fn check_dsp_huge_map_ship(&mut self, ctx: &mut MapContext<'_>) {
        if !self.fishman_active {
            self.panes.spi3.hide();
            self.dsp_huge_map_ship = false;
        } else if !ctx.env.ship_ride
            && self.target_grid_x == self.save.get_cur_x()
            && self.target_grid_y == self.save.get_cur_y()
        {
            if self.ship_open_water {
                self.dsp_huge_map_ship = false;
                self.panes.spi3.hide();
            } else {
                self.dsp_huge_map_ship = true;
            }
        } else {
            self.panes.spi3.hide();
            self.dsp_huge_map_ship = false;
        }
    }

    pub fn open(&mut self) -> bool {
        if self.frame_timer == 0 {
            self.button_icon_mode = BTN_ICON_WORLD;
        }
        let ret = if self.zoom_locked {
            true
        } else {
            let result = self.pane_trans_base(
                self.frame_timer
                    .wrapping_sub(i16::from(self.tuning.open_delay)),
                self.tuning.base_anim_frame,
                self.tuning.open_pos_y,
                0.0,
                0,
                0,
            );
            self.frame_timer = self.frame_timer.wrapping_add(1);
            result
        };
        if ret {
            self.frame_timer = 0;
            return true;
        }
        false
    }

    pub fn close(&mut self) -> bool {
        match self.fmap_mode {
            MODE_WARP => self.close_warp_mode(),
            MODE_FISHMAN => self.close_fishman_mode(),
            _ => self.close_normal_mode(),
        }
    }

    pub fn close_normal_mode(&mut self) -> bool {
        let result = if self.input_disabled {
            true
        } else {
            self.pane_trans_base(
                self.frame_timer,
                self.tuning.base_anim_frame,
                0.0,
                self.tuning.close_pos_y,
                0,
                1,
            )
        };
        self.frame_timer = self.frame_timer.wrapping_add(1);
        if result {
            if !self.input_disabled {
                self.save.set_disp_mode(DISP_NORMAL);
            }
            self.frame_timer = 0;
            return true;
        }
        false
    }

    pub fn close_warp_mode(&mut self) -> bool {
        let result = self.pane_trans_base(
            self.frame_timer,
            self.tuning.base_anim_frame,
            0.0,
            0.0,
            0,
            1,
        );
        self.frame_timer = self.frame_timer.wrapping_add(1);
        if result {
            self.frame_timer = 0;
            self.fmap_mode = MODE_NORMAL;
            return true;
        }
        false
    }

    pub fn close_fishman_mode(&mut self) -> bool {
        let result = self.pane_trans_base(
            self.frame_timer,
            self.tuning.fishman_close_frame,
            0.0,
            0.0,
            0,
            1,
        );
        self.frame_timer = self.frame_timer.wrapping_add(1);
        if result {
            self.frame_timer = 0;
            self.fmap_mode = MODE_NORMAL;
            return true;
        }
        false
    }

    pub fn move_step(&mut self, ctx: &mut MapContext<'_>) {
        match self.fmap_mode {
            MODE_WARP => self.move_main_warp_mode(ctx),
            MODE_WALLPAPER => {
                if ctx.pad.trig_left || ctx.pad.trig_down || ctx.pad.trig_b {
                    self.map_close = true;
                } else {
                    self.map_close = false;
                }
            }
            MODE_FISHMAN => self.move_fishman_mode(ctx),
            _ => self.move_normal(ctx),
        }
    }

    pub fn draw(&mut self) -> MapDraw {
        let compare_visible = self.main_proc_idx == MAIN_PROC_HIKAKU;
        let mut compare_panes = Vec::new();
        if compare_visible {
            if let Some(compare) = self.compare.as_mut() {
                compare_panes = compare.draw();
            }
        }
        MapDraw {
            visible: self.screen_loaded,
            compare_visible,
            panes: self.panes.all().into_iter().cloned().collect(),
            compare_panes,
        }
    }

    pub fn delete(&mut self) -> MapTearDown {
        if let Some(compare) = self.compare.as_mut() {
            compare.delete();
        }
        self.compare = None;
        self.chart = None;
        self.screen_loaded = false;
        MapTearDown {
            release_screen_archive: true,
            release_texture_archive: true,
            release_tuning: true,
        }
    }

    pub fn fmap_proc_main(&mut self, ctx: &mut MapContext<'_>) {
        if ctx.pad.trig_left
            || ctx.pad.trig_down
            || self.input_disabled
            || (ctx.pad.trig_b && self.save.get_zoom() == ZOOM_WORLD)
        {
            self.map_close = true;
            return;
        }
        match self.fmap_proc_idx {
            ZOOM_PROC_SELECT_GRID => self.select_grid(ctx),
            ZOOM_PROC_LV1_IN => self.zoom_grid_lv1_in(ctx),
            ZOOM_PROC_LV1 => self.zoom_grid_lv1_proc(ctx),
            ZOOM_PROC_LV1_OUT => self.zoom_grid_lv1_out(),
            ZOOM_PROC_LV2_IN => self.zoom_grid_lv2_in(),
            ZOOM_PROC_LV2 => self.zoom_grid_lv2_proc(ctx),
            _ => self.zoom_grid_lv2_out(ctx),
        }
        self.wind_arrow_color_anime(ctx);
        self.check_mark_anime();
        self.g_ship_mark_anime();
    }

    pub fn select_grid(&mut self, ctx: &mut MapContext<'_>) {
        self.stick
            .check_trigger(ctx.pad.stick_value, ctx.pad.stick_angle);
        if ctx.pad.trig_a {
            ctx.out.sound(SE_CHART_ZOOM_IN);
            if ctx
                .flags
                .is_save_arrive_grid(grid_index(self.save.get_cur_x(), self.save.get_cur_y()))
            {
                self.button_icon_mode = BTN_ICON_SECTOR;
            } else {
                self.button_icon_mode = BTN_ICON_LOCKED;
            }
            self.save.set_zoom_grid_x(self.save.get_cur_x());
            self.save.set_zoom_grid_y(self.save.get_cur_y());
            self.save.set_zoom(ZOOM_SECTOR);
            self.zoom_1000x1000_init(ctx);
            self.warp_scroll_guard = false;
            if self.area_txt_changing {
                self.area_txt_chg_fast();
                self.area_txt_changing = false;
            }
            self.frame_timer = 0;
            self.fmap_proc_idx = ZOOM_PROC_LV1_IN;
        } else {
            let mut moved = false;
            if self.stick.check_right_trigger() {
                if self.save.get_cur_x() < 3 {
                    self.save.set_cur_x(self.save.get_cur_x() + 1);
                    moved = true;
                }
            } else if self.stick.check_left_trigger() {
                if self.save.get_cur_x() > -3 {
                    self.save.set_cur_x(self.save.get_cur_x() - 1);
                    moved = true;
                }
            } else if self.stick.check_up_trigger() {
                if self.save.get_cur_y() > -3 {
                    self.save.set_cur_y(self.save.get_cur_y() - 1);
                    moved = true;
                }
            } else if self.stick.check_down_trigger() {
                if self.save.get_cur_y() < 3 {
                    self.save.set_cur_y(self.save.get_cur_y() + 1);
                    moved = true;
                }
            }
            if moved {
                ctx.out.sound(SE_CHART_CURSOR);
                self.sel_cursor_move();
                self.island_name_change(ctx);
                self.salvage_get_item_change(ctx);
            }
        }
        self.player_point_grid_anime();
        self.sel_cursor_anime();
        self.area_txt_chg();
        self.salvage_get_item_chg();
    }

    pub fn zoom_1000x1000_init(&mut self, ctx: &mut MapContext<'_>) {
        let wy = self.save.get_cur_y();
        let wx = self.save.get_cur_x();
        let grid = grid_index(wx, wy);

        zoom_map_alpha_set(wx, wy, &mut self.panes.r01b, 0x14);
        self.check_dsp_large_map_link(ctx);
        self.check_dsp_large_map_ship(ctx);

        self.set_disp_island_pos(self.save.get_cur_x(), self.save.get_cur_y(), ctx);
        self.sel_cursor_init();
        self.zoom_cursor_init();
        self.panes.kkdm.show();
        self.panes.smsk.show();
        self.panes.clb.show();

        if !ctx.flags.is_save_arrive_grid(grid) {
            self.panes.tsw1.hide();
            self.panes.r01b.hide();
            self.panes.ktx1.show();
            if !ctx.env.opt_ruby {
                self.panes.ktx2.show();
            }
            self.panes.clb2.show();
        } else {
            self.panes.ktx1.hide();
            self.panes.ktx2.hide();
            self.panes.clb2.hide();
        }
    }

    pub fn zoom_grid_lv1_in(&mut self, ctx: &mut MapContext<'_>) {
        let sel_cursor = pane_alpha_fmap_cursor(
            &mut self.panes.kk1,
            self.frame_timer,
            self.tuning.select_cursor_alpha_frame,
            0,
            1,
        );
        let sel_grid_mask =
            self.sel_grid_mask_alpha_ctrl(self.frame_timer, self.tuning.zoom_anim_frame, 0, 0);
        let fmap_mask =
            self.fmap_mask_alpha_ctrl(self.frame_timer, self.tuning.zoom_anim_frame, 0, 0);
        let (tx, ty, ts) = {
            let x = self.panes.kkdm.pos_center.x - self.panes.clb.pos_center_orig.x;
            let y = self.panes.kkdm.pos_center.y - self.panes.clb.pos_center_orig.y;
            let scale = self.panes.kkdm.size_orig.x / self.panes.clb.size_orig.x;
            (x, y, scale)
        };
        let zoom_map = self.pane_trance_zoom_map(
            self.frame_timer
                .wrapping_sub(i16::from(self.tuning.zoom_map_delay)),
            self.tuning.zoom_anim_frame,
            tx,
            ty,
            0.0,
            0.0,
            ts,
            1.0,
            0x02,
            0,
        );
        let zoom_cursor = pane_alpha_fmap_cursor(
            &mut self.panes.kk3,
            self.frame_timer
                .wrapping_sub(i16::from(self.tuning.zoom_map_delay)),
            self.tuning.zoom_anim_frame,
            0,
            0,
        );
        self.frame_timer = self.frame_timer.wrapping_add(1);
        if ctx.pad.trig_a && !self.warp_scroll_guard {
            self.warp_scroll_guard = true;
        }
        if sel_cursor && sel_grid_mask && fmap_mask && zoom_map && zoom_cursor {
            self.frame_timer = 0;
            if self.dsp_large_map_link {
                self.panes.lnk2.show();
            }
            if self.dsp_large_map_ship {
                self.panes.spi2.show();
            }
            self.fmap_proc_idx = ZOOM_PROC_LV1;
        }
    }

    pub fn zoom_grid_lv1_proc(&mut self, ctx: &mut MapContext<'_>) {
        if ctx.pad.trig_a || self.warp_scroll_guard {
            if self.button_icon_mode == BTN_ICON_SECTOR {
                ctx.out.sound(SE_CHART_ZOOM_IN);
                self.button_icon_mode = BTN_ICON_DETAIL;
                self.save.set_zoom(ZOOM_DETAIL);
                self.zoom_200x200_init(ctx);
                let clg_x = self.panes.clg.size_orig.x;
                let px = self.disp_island_pos_x * (clg_x / WORLD_UNIT);
                let py = self.disp_island_pos_y * (self.panes.clg.size_orig.y / WORLD_UNIT);
                let sx = self.panes.tsw1.size_orig.x / clg_x;
                self.pane_trance_zoom2_map(
                    0,
                    self.tuning.zoom2_anim_frame,
                    px,
                    py,
                    0.0,
                    0.0,
                    sx,
                    1.0,
                    0x2,
                    0,
                );
                for index in 0..8usize {
                    self.panes.kk3[index].set_influenced_alpha(false);
                }
                self.panes.r01b.set_influenced_alpha(false);
                self.frame_timer = 0;
                self.fmap_proc_idx = ZOOM_PROC_LV2_IN;
            }
            self.warp_scroll_guard = false;
        } else if ctx.pad.trig_b || self.warp_anim_active {
            ctx.out.sound(SE_CHART_ZOOM_OUT);
            if self.dsp_large_map_link {
                self.panes.lnk2.hide();
            }
            if self.dsp_large_map_ship {
                self.panes.spi2.hide();
            }
            self.warp_anim_active = false;
            self.zoom_cursor_init();
            self.fmap_proc_idx = ZOOM_PROC_LV1_OUT;
        }
        self.zoom_cursor_anime();
    }

    pub fn zoom_200x200_init(&mut self, ctx: &mut MapContext<'_>) {
        let cx = self.save.get_cur_x();
        let cy = self.save.get_cur_y();
        zoom_map_alpha_set(cx, cy, &mut self.panes.r01g, 0x1E);
        self.zoom_cursor_init();
        self.set_dsp_huge_map_link(ctx);
        self.check_dsp_huge_map_link(ctx);
        self.check_dsp_huge_map_ship(ctx);
        self.panes.clg.show();
    }

    pub fn zoom_grid_lv1_out(&mut self) {
        let sel_grid_mask = self.sel_grid_mask_alpha_ctrl(
            self.frame_timer
                .wrapping_sub(i16::from(self.tuning.zoom_out_delay)),
            self.tuning.zoom_anim_frame,
            0,
            1,
        );
        let fmap_mask = self.fmap_mask_alpha_ctrl(
            self.frame_timer
                .wrapping_sub(i16::from(self.tuning.zoom_out_delay)),
            self.tuning.zoom_anim_frame,
            0,
            1,
        );
        let (tx, ty, ts) = {
            let x = self.panes.kkdm.pos_center.x - self.panes.clb.pos_center_orig.x;
            let y = self.panes.kkdm.pos_center.y - self.panes.clb.pos_center_orig.y;
            let scale = self.panes.kkdm.size_orig.x / self.panes.clb.size_orig.x;
            (x, y, scale)
        };
        let zoom_map = self.pane_trance_zoom_map(
            self.frame_timer
                .wrapping_sub(i16::from(self.tuning.zoom_map_delay)),
            self.tuning.zoom_anim_frame,
            0.0,
            0.0,
            tx,
            ty,
            1.0,
            ts,
            0x02,
            1,
        );
        let zoom_cursor = pane_alpha_fmap_cursor(
            &mut self.panes.kk3,
            self.frame_timer
                .wrapping_sub(i16::from(self.tuning.zoom_map_delay)),
            self.tuning.zoom_anim_frame,
            0,
            1,
        );
        let sel_cursor = pane_alpha_fmap_cursor(
            &mut self.panes.kk1,
            self.frame_timer
                .wrapping_sub(i16::from(self.tuning.zoom_out_delay)),
            self.tuning.select_cursor_alpha_frame,
            0,
            0,
        );
        self.frame_timer = self.frame_timer.wrapping_add(1);
        if sel_grid_mask && fmap_mask && zoom_map && zoom_cursor && sel_cursor {
            self.frame_timer = 0;
            self.panes.kkdm.hide();
            self.panes.smsk.hide();
            self.panes.clb.hide();
            self.button_icon_mode = BTN_ICON_WORLD;
            self.save.set_zoom(ZOOM_WORLD);
            self.fmap_proc_idx = ZOOM_PROC_SELECT_GRID;
        }
    }

    pub fn zoom_grid_lv2_in(&mut self) {
        let orig_x = self.panes.clg.size_orig.x;
        let px = self.disp_island_pos_x * (orig_x / WORLD_UNIT);
        let py = self.disp_island_pos_y * (self.panes.clg.size_orig.y / WORLD_UNIT);
        let sx = self.panes.tsw1.size_orig.x / orig_x;
        let zoom2 = self.pane_trance_zoom2_map(
            self.frame_timer,
            self.tuning.zoom2_anim_frame,
            px,
            py,
            0.0,
            0.0,
            sx,
            1.0,
            0x02,
            0,
        );
        let alpha = self.pane_trance_zoom_map_alphah(
            self.frame_timer,
            self.tuning.zoom2_anim_frame,
            0x02,
            1,
        );
        self.frame_timer = self.frame_timer.wrapping_add(1);
        if zoom2 && alpha {
            self.frame_timer = 0;
            if self.dsp_huge_map_link {
                self.panes.lnk3.show();
            } else {
                self.panes.lnk3.hide();
            }
            if self.dsp_huge_map_ship {
                self.panes.spi3.show();
            } else {
                self.panes.spi3.hide();
            }
            self.fmap_proc_idx = ZOOM_PROC_LV2;
        }
    }

    pub fn zoom_grid_lv2_proc(&mut self, ctx: &mut MapContext<'_>) {
        if ctx.pad.trig_b {
            ctx.out.sound(SE_CHART_ZOOM_OUT);
            if self.dsp_huge_map_link {
                self.panes.lnk3.hide();
            }
            if self.dsp_huge_map_ship {
                self.panes.spi3.hide();
            }
            self.warp_anim_active = false;
            self.zoom_cursor_init();
            self.fmap_proc_idx = ZOOM_PROC_LV2_OUT;
        }
        self.zoom_cursor_anime();
    }

    pub fn zoom_grid_lv2_out(&mut self, ctx: &mut MapContext<'_>) {
        let orig_x = self.panes.clg.size_orig.x;
        let px = self.disp_island_pos_x * (orig_x / WORLD_UNIT);
        let py = self.disp_island_pos_y * (self.panes.clg.size_orig.y / WORLD_UNIT);
        let sx = self.panes.tsw1.size_orig.x / orig_x;
        let zoom2 = self.pane_trance_zoom2_map(
            self.frame_timer,
            self.tuning.zoom2_anim_frame,
            0.0,
            0.0,
            px,
            py,
            1.0,
            sx,
            0x02,
            1,
        );
        let alpha = self.pane_trance_zoom_map_alphah(
            self.frame_timer,
            self.tuning.zoom2_anim_frame,
            0x02,
            0,
        );
        if ctx.pad.trig_b && !self.warp_anim_active {
            self.warp_anim_active = true;
        }
        self.frame_timer = self.frame_timer.wrapping_add(1);

        if zoom2 && alpha {
            self.button_icon_mode = BTN_ICON_SECTOR;
            self.save.set_zoom(ZOOM_SECTOR);
            if self.dsp_large_map_link {
                self.panes.lnk2.show();
            }
            self.panes.clg.hide();
            for index in 0..8usize {
                self.panes.kk3[index].set_influenced_alpha(true);
            }
            self.panes.r01b.set_influenced_alpha(true);
            self.frame_timer = 0;
            self.fmap_proc_idx = ZOOM_PROC_LV1;
        }
    }

    pub fn move_normal(&mut self, ctx: &mut MapContext<'_>) {
        match self.main_proc_idx {
            MAIN_PROC_HIKAKU => self.compare_proc(ctx),
            _ => self.fmap_proc(ctx),
        }
    }

    pub fn fmap_proc(&mut self, ctx: &mut MapContext<'_>) {
        if ctx.pad.trig_y {
            ctx.out.sound(SE_CHART_TO_COMPARE);
            self.save.set_disp_mode(DISP_COMPARE);
            self.main_proc_idx = MAIN_PROC_HIKAKU;
            self.compare_proc_idx = HIKAKU_PROC_OPEN;
        } else {
            self.fmap_proc_main(ctx);
        }
    }

    pub fn compare_proc(&mut self, ctx: &mut MapContext<'_>) {
        match self.compare_proc_idx {
            HIKAKU_PROC_OPEN => self.compare_open(),
            HIKAKU_PROC_CLOSE => self.compare_close(),
            _ => self.compare_move(ctx),
        }
    }

    pub fn compare_open(&mut self) {
        let mut opened = false;
        if let Some(compare) = self.compare.as_mut() {
            opened = compare.open();
        }
        if opened {
            if self.zoom_locked {
                self.pane_trans_base(0, self.tuning.base_anim_frame, 0.0, 0.0, 0, 1);
            }
            self.compare_proc_idx = HIKAKU_PROC_MOVE;
        }
    }

    pub fn compare_move(&mut self, ctx: &mut MapContext<'_>) {
        if let Some(compare) = self.compare.as_mut() {
            compare.move_step();
        }
        let locked = self
            .compare
            .as_ref()
            .is_some_and(|compare| compare.lock_b_button());
        if ctx.pad.trig_y && !locked {
            ctx.out.sound(SE_CHART_TO_NORMAL);
            self.compare_proc_idx = HIKAKU_PROC_CLOSE;
            self.input_disabled = false;
            self.g_ship_mark_anime_init(ctx);
            return;
        }

        if !ctx.pad.trig_b && !ctx.pad.trig_left && !ctx.pad.trig_down {
            return;
        } else if !locked {
            self.pane_trans_base(0, self.tuning.base_anim_frame, 480.0, 0.0, 0, 0);
            self.input_disabled = true;
            self.compare_proc_idx = HIKAKU_PROC_CLOSE;
        }
    }

    pub fn compare_close(&mut self) {
        let mut closed = false;
        if let Some(compare) = self.compare.as_mut() {
            closed = compare.close();
        }
        if closed {
            self.main_proc_idx = MAIN_PROC_FMAP;
            self.compare_proc_idx = HIKAKU_PROC_MOVE;
        }
    }

    pub fn pane_trans_base(
        &mut self,
        frame: i16,
        max: u8,
        x: f32,
        y: f32,
        mode: u8,
        flag: i32,
    ) -> bool {
        if frame < 0 {
            return false;
        }
        if frame > i16::from(max) {
            return true;
        }
        let mut alpha = value_increase(i32::from(max), i32::from(frame), mode);
        let trans_y = alpha * (y - x);
        self.panes.cl.pane_trans(0.0, x + trans_y);
        self.panes.fddm.pane_trans(0.0, x + trans_y);

        if flag != 2 {
            if flag == 1 {
                alpha = 1.0 - alpha;
            }
            for index in 0..3usize {
                self.panes.ck1[index].set_now_alpha(alpha);
                self.panes.ck2[index].set_now_alpha(alpha);
                self.panes.ck1[index].set_alpha();
                self.panes.ck2[index].set_alpha();
            }
            for index in 0..2usize {
                self.panes.ck3[index].set_now_alpha(alpha);
                self.panes.ck3[index].set_alpha();
            }
            self.panes.fddm.set_now_alpha(alpha);
            self.panes.fddm.set_alpha();
            self.panes.cl.set_now_alpha(alpha);
            self.panes.cl.set_alpha();
        }
        false
    }

    pub fn pane_trance_zoom_map(
        &mut self,
        frame: i16,
        max: u8,
        trans_start_x: f32,
        trans_start_y: f32,
        trans_end_x: f32,
        trans_end_y: f32,
        scale_start: f32,
        scale_end: f32,
        mode: u8,
        flag: i32,
    ) -> bool {
        if frame < 0 {
            return false;
        }
        if frame > i16::from(max) {
            return true;
        }

        let mut alpha = value_increase(i32::from(max), i32::from(frame), mode);

        let f0 = trans_end_x - trans_start_x;
        let f1 = trans_end_y - trans_start_y;
        let f2 = scale_end - scale_start;
        let f29 = alpha * f0;
        let f28 = alpha * f1;
        let scale_adj = alpha * f2;
        self.panes.clb.pane_scale_xy(scale_start + scale_adj);
        let scale = scale_start + scale_adj;
        let start_x = trans_start_x + f29;
        let start_y = trans_start_y + f28;
        self.panes.clb.pane_trans(start_x, start_y);
        if flag == 1 {
            alpha = 1.0 - alpha;
        }

        if self.panes.lnk2.is_visible() {
            let angle = self.player_pos.z;
            let base = self.panes.clb.clone();
            child_pane_move_sp(&mut self.panes.lnk2, &base, alpha, scale, scale);
            let cx = (self.panes.lnk2.size.x * 0.5) as i32 as f32;
            let cy = (self.panes.lnk2.size.y * 0.5) as i32 as f32;
            self.panes.lnk2.rotate(cx, cy, angle);
        }

        if self.panes.spi2.is_visible() {
            let angle = self.ship_pos.z;
            let base = self.panes.clb.clone();
            child_pane_move_sp(&mut self.panes.spi2, &base, alpha, scale, scale);
            let cx = (self.panes.spi2.size.x * 0.5) as i32 as f32;
            let cy = (self.panes.spi2.size.y * 0.5) as i32 as f32;
            self.panes.spi2.rotate(cx, cy, angle);
        }

        if self.panes.ktx2.is_visible() {
            let size = scale * KTX2_FONT_SIZE;
            self.panes.ktx2.set_font_size(size, size);
            let base = self.panes.clb.clone();
            child_pane_move_sp(&mut self.panes.ktx2, &base, alpha, scale, scale);
        }

        if self.panes.ktx1.is_visible() {
            self.panes
                .ktx1
                .set_font_size(scale * KTX1_FONT_SIZE_X, scale * KTX1_FONT_SIZE_Y);
            self.panes.ktx1.set_line_space(scale * KTX1_LINE_SPACE);
            let base = self.panes.clb.clone();
            child_pane_move_sp(&mut self.panes.ktx1, &base, alpha, scale, scale);
            self.out_font2.message_set(MSG_UNKNOWN_NAME);
            let left = self.panes.ktx1.pos_top_left.x;
            let top = self.panes.ktx1.pos_top_left.y;
            self.out_font2.set_left_up_pos(left, top);
            self.out_font2.move_step();
        }

        if self.panes.clb2.is_visible() {
            let base = self.panes.clb.clone();
            child_pane_move_sp(&mut self.panes.clb2, &base, alpha, scale, scale);
        }
        for index in 0..9usize {
            let base = self.panes.clb.clone();
            child_pane_move_sp(&mut self.panes.sc1[index], &base, alpha, scale, scale);
        }

        let base = self.panes.clb.clone();
        child_pane_move_sp(&mut self.panes.r01b, &base, alpha, scale, scale);
        child_pane_move_sp(&mut self.panes.stm1, &base, alpha, scale, scale);
        child_pane_move_sp(&mut self.panes.tsw1, &base, alpha, scale, scale);

        for index in 0..8usize {
            child_pane_move_sp(&mut self.panes.kk3[index], &base, alpha, 1.0, 1.0);
        }

        if flag != 2 {
            self.panes.r01b.set_now_alpha(alpha);
            self.panes.r01b.set_alpha();
            self.panes.clb.set_now_alpha(alpha);
            self.panes.clb.set_alpha();
        }
        false
    }

    pub fn pane_trance_zoom_map_alphah(
        &mut self,
        frame: i16,
        max: u8,
        mode: u8,
        flag: i32,
    ) -> bool {
        if frame < 0 {
            return false;
        }
        if frame > i16::from(max) {
            return true;
        }
        let mut alpha = value_increase(i32::from(max), i32::from(frame), mode);
        if flag != 2 {
            if flag == 1 {
                alpha = 1.0 - alpha;
            }
            self.panes.clb.set_now_alpha(alpha);
            self.panes.clb.set_alpha();
        }
        false
    }

    pub fn pane_trance_zoom2_map(
        &mut self,
        frame: i16,
        max: u8,
        trans_start_x: f32,
        trans_start_y: f32,
        trans_end_x: f32,
        trans_end_y: f32,
        scale_start: f32,
        scale_end: f32,
        mode: u8,
        flag: i32,
    ) -> bool {
        if frame < 0 {
            return false;
        }
        if frame > i16::from(max) {
            return true;
        }

        let mut alpha = value_increase(i32::from(max), i32::from(frame), mode);
        let trans_x_range = trans_end_x - trans_start_x;
        let trans_y_range = trans_end_y - trans_start_y;
        let scale_range = scale_end - scale_start;
        let trans_x_adj = alpha * trans_x_range;
        let trans_y_adj = alpha * trans_y_range;
        let scale_adj = alpha * scale_range;
        let start_x = trans_start_x + trans_x_adj;
        let start_y = trans_start_y + trans_y_adj;
        self.panes.clg.pane_trans(start_x, start_y);
        self.panes.clg.pane_scale_xy(scale_start + scale_adj);
        let scale = scale_start + scale_adj;

        if flag == 1 {
            alpha = 1.0 - alpha;
        }

        if self.panes.lnk3.is_visible() {
            let angle = self.player_pos.z;
            let base = self.panes.clg.clone();
            child_pane_move_sp(&mut self.panes.lnk3, &base, alpha, scale, scale);
            let cx = (self.panes.lnk3.size.x * 0.5) as i32 as f32;
            let cy = (self.panes.lnk3.size.y * 0.5) as i32 as f32;
            self.panes.lnk3.rotate(cx, cy, angle);
        }

        if self.panes.spi3.is_visible() {
            let angle = self.ship_pos.z;
            let base = self.panes.clg.clone();
            child_pane_move_sp(&mut self.panes.spi3, &base, alpha, scale, scale);
            let cx = (self.panes.spi3.size.x * 0.5) as i32 as f32;
            let cy = (self.panes.spi3.size.y * 0.5) as i32 as f32;
            self.panes.spi3.rotate(cx, cy, angle);
        }

        let base = self.panes.clg.clone();
        child_pane_move_sp(&mut self.panes.r01g, &base, alpha, scale, scale);
        child_pane_move_sp(&mut self.panes.stl1, &base, alpha, scale, scale);

        for index in 0..7usize {
            child_pane_move_sp(&mut self.panes.sc2[index], &base, alpha, scale, scale);
        }
        for index in 0..5usize {
            child_pane_move_sp(&mut self.panes.fmx[index], &base, alpha, scale, scale);
        }

        self.panes.fmx[1].rotate(
            (self.panes.fmx[1].size.x * 0.5) as i32 as f32,
            (self.panes.fmx[1].size.y * 0.5) as i32 as f32,
            90.0,
        );
        self.panes.fmx[4].rotate(
            (self.panes.fmx[4].size.x * 0.5) as i32 as f32,
            (self.panes.fmx[4].size.y * 0.5) as i32 as f32,
            90.0,
        );

        for index in 0..15usize {
            child_pane_move_sp(&mut self.panes.fmn[index], &base, alpha, scale, scale);
        }

        if flag != 2 {
            for _ in 0..4 {
                self.panes.r01g.set_now_alpha(alpha);
                self.panes.r01g.set_alpha();
                self.panes.clg.set_now_alpha(alpha);
                self.panes.clg.set_alpha();
            }
        }
        false
    }

    pub fn pane_alpha_selvage_item(&mut self, frame: i16, max: u8) -> bool {
        if frame < 0 {
            return false;
        }
        if frame > i16::from(max) {
            return true;
        }
        let in_alpha = value_increase(i32::from(max), i32::from(frame), 0);
        let out_alpha = 1.0 - in_alpha;
        let cur = self.salv_itm_buf_idx as usize % 2;
        if self.panes.gti[cur].now_alpha != 0 {
            self.panes.gti[cur].set_now_alpha(out_alpha);
            self.panes.gti[cur].set_alpha();
        }
        if self.full_map_mode {
            self.panes.gti[cur ^ 1].set_now_alpha(in_alpha);
            self.panes.gti[cur ^ 1].set_alpha();
        }
        false
    }

    pub fn g_ship_mark_anime_init(&mut self, ctx: &mut MapContext<'_>) {
        self.moon_type = ctx.env.moon_type;
        if ctx.flags.is_open_collect_map(0x24) {
            self.moon_alpha_active = true;
            if i32::from(ctx.flags.event_reg(REG_GHOST_SHIP)) == 3 {
                self.moon_alpha_state1 = true;
            } else {
                self.moon_alpha_state1 = false;
                self.moon_alpha_state2 = false;
                let time = ctx.env.game_time;
                if time >= 285.0 || (time >= 0.0 && time <= 90.0) {
                    self.moon_alpha_state2 = true;
                    let index = self.moon_type as usize % 7;
                    self.panes.gs[index].show();
                }
            }
            self.moon_alpha_timer = self.tuning.moon_alpha_frame;
            self.moon_alpha_toggle = 1;
        } else {
            self.moon_alpha_active = false;
        }
    }

    pub fn g_ship_mark_anime(&mut self) {
        if self.moon_alpha_active && !self.moon_alpha_state1 && self.moon_alpha_state2 {
            let mut alpha = value_increase(
                i32::from(self.tuning.moon_alpha_frame),
                i32::from(self.moon_alpha_timer),
                2,
            );
            if self.moon_alpha_toggle == 0 {
                alpha = 1.0 - alpha;
            }
            let range =
                f32::from(self.tuning.moon_alpha_max) - f32::from(self.tuning.moon_alpha_min);
            let index = self.moon_type as usize % 7;
            self.panes.gs[index].now_alpha =
                (f32::from(self.tuning.moon_alpha_min) + (range * alpha) as u32 as f32) as u8;
            self.panes.gs[index].set_alpha();
            if self.moon_alpha_timer == 0 {
                self.moon_alpha_timer = self.tuning.moon_alpha_frame;
                self.moon_alpha_toggle ^= 1;
            } else {
                self.moon_alpha_timer -= 1;
            }
        }
    }

    pub fn open_warp_mode(&mut self, ctx: &mut MapContext<'_>) -> bool {
        if self.frame_timer == 0 {
            self.init_warp_mode(ctx);
        }
        let ret = self.pane_trans_base(
            self.frame_timer,
            self.tuning.base_anim_frame,
            0.0,
            0.0,
            0,
            0,
        );
        self.frame_timer = self.frame_timer.wrapping_add(1);
        let ratio = f32::from(self.frame_counter) / f32::from(self.tuning.warp_back_color_cycle);
        self.frame_counter = self.frame_counter.wrapping_add(1);
        let counter = self.frame_counter;
        let span = u32::from(self.tuning.warp_back_color_cycle) * 4;
        self.frame_counter = (u32::from(counter) - (u32::from(counter) / span) * span) as u8;
        self.set_dsp_warp_back_corner_color(ratio);

        if ret {
            self.frame_timer = 0;
            for index in 0..WARP_SPOT_COUNT {
                self.warp_spot_emitters[index].drawing = true;
            }
            self.warp_back_emitter.drawing = false;
            self.warp_proc_idx = WARP_PROC_MOVE;
            return true;
        }
        false
    }

    pub fn init_warp_mode(&mut self, ctx: &mut MapContext<'_>) {
        ctx.out.sound(SE_SHIPPU_CHART_OPEN);
        self.fmap_mode = MODE_WARP;
        self.button_icon_mode = BTN_ICON_WARP;

        if ctx.env.opt_ruby {
            self.panes.wt1.hide();
            self.panes.wt0.pos_center_orig.y -= 4.0;
            self.panes.wt0.pos_center.y = self.panes.wt0.pos_center_orig.y;
            self.panes.wt0.cpos_move();
        }

        if self.save.get_cur_wx() < -3 || self.save.get_cur_wy() < -3 {
            self.save.set_cur_wx(-2);
            self.save.set_cur_wy(-2);
        }
        self.sel_cursor_init();
        self.sel_cursor_move_warp();
        self.warp_sub_state = WARP_SEL_YES;

        self.set_wrap_back_emitter(0.0, 0.0, 0.0);
        self.warp_back_emitter.drawing = false;

        for index in 0..WARP_SPOT_COUNT {
            let x = self.tuning.warp_spot_pos_x[index];
            let y = self.tuning.warp_spot_pos_y[index];
            self.set_wrap_spot_emitter(index, x, y, 0.0);
            self.warp_spot_emitters[index].drawing = false;
        }

        self.area_text_change_anime_init(ctx);
        self.frame_counter = 0;
        self.set_dsp_warp_back_corner_color(0.0);
        self.warp_area_anime0();
    }

    pub fn sel_cursor_move_warp(&mut self) {
        let trans_x = f32::from(self.save.get_cur_wx()) * GRID_STEP;
        let trans_y = f32::from(self.save.get_cur_wy()) * GRID_STEP;
        for index in 0..8usize {
            self.panes.kk1[index].pane_trans(trans_x, trans_y);
        }
    }

    pub fn move_main_warp_mode(&mut self, ctx: &mut MapContext<'_>) {
        self.stick
            .check_trigger(ctx.pad.stick_value, ctx.pad.stick_angle);
        match self.warp_proc_idx {
            WARP_PROC_MOVE => self.wrap_move(ctx),
            WARP_PROC_SEL_WIN_FADE_IN => self.wrap_sel_win_fade_in(),
            WARP_PROC_SELECT => self.wrap_select(ctx),
            WARP_PROC_SEL_WIN_FADE_OUT => self.wrap_sel_win_fade_out(),
            _ => self.wrap_sel_warp(ctx),
        }
    }

    pub fn wrap_move(&mut self, ctx: &mut MapContext<'_>) {
        let table = self.warp_area_table(self.save.get_cur_wx(), self.save.get_cur_wy());
        let mut msg_no = warp_area_no(table);

        if ctx.pad.trig_a {
            ctx.out.sound(SE_SHIPPU_DIST_SEL);
            if ctx
                .flags
                .is_save_arrive_grid(grid_index(self.save.get_cur_wx(), self.save.get_cur_wy()))
            {
                if msg_no == 7 {
                    msg_no = 0x43;
                } else if msg_no == 8 {
                    msg_no = 0x42;
                } else {
                    msg_no += 0x45;
                }
            } else {
                msg_no = 0x44;
            }

            let msg = msg_no as u32;
            let (note0, note1, dummy0, dummy1) = ctx.text.select_message(msg);
            self.txt_note[0] = note0.clone();
            self.txt_note[1] = note1.clone();
            self.txt_dummy[0] = dummy0;
            self.txt_dummy[1] = dummy1;
            self.txt_note_msg[0] = Some(msg);
            self.txt_note_msg[1] = Some(msg);
            self.panes.wt0.set_message(msg, Some(&note0));
            self.panes.wt1.set_message(msg, Some(&note1));
            self.panes.cur1.show();
            self.warp_sub_state = WARP_SEL_YES;

            self.panes
                .no01
                .set_black_white(WARP_NO_SHADOW, WARP_NO_BODY);
            self.panes
                .no00
                .set_black_white(WARP_NO_SHADOW, WARP_NO_BODY);
            self.panes
                .ys01
                .set_black_white(self.ys01_color1, self.ys01_color2);
            self.panes
                .ys00
                .set_black_white(self.ys01_color1, self.ys01_color2);
            self.warp_sel_cursor_move();
            self.warp_sel_cursor_anime_init();

            self.panes.wts1.show();
            self.panes.cc01.show();
            self.panes.ysk0.show();
            self.panes.nok0.show();
            self.panes.cur1.hide();
            self.frame_timer = 0;

            self.pane_alpha_warp_msg_back(self.frame_timer, self.tuning.warp_msg_anim_frame, 0, 0);
            pane_trance_warp_msg(
                &mut self.panes.cc01,
                self.frame_timer,
                self.tuning.warp_msg_anim_frame,
                self.tuning.warp_msg_pos_y,
                0.0,
                0,
                0,
            );
            pane_trance_warp_msg(
                &mut self.panes.ysk0,
                self.frame_timer,
                self.tuning.warp_msg_anim_frame,
                self.tuning.warp_yes_no_pos_y,
                0.0,
                0,
                0,
            );
            pane_trance_warp_msg(
                &mut self.panes.nok0,
                self.frame_timer,
                self.tuning.warp_msg_anim_frame,
                self.tuning.warp_yes_no_pos_y,
                0.0,
                0,
                0,
            );
            self.area_text_change_anime_init(ctx);
            for index in 0..WARP_SPOT_COUNT {
                self.warp_spot_emitters[index].drawing = false;
            }
            self.warp_proc_idx = WARP_PROC_SEL_WIN_FADE_IN;
        } else if ctx.pad.trig_b || ctx.pad.trig_down {
            ctx.out.sound(SE_SHIPPU_CHART_CLOSE);
            if ctx.env.has_ship {
                ctx.out.ship_warp_pos.push(-1);
            }
            self.map_close = true;
            self.stop_wrap_back_emitter();
            for index in 0..WARP_SPOT_COUNT {
                self.warp_spot_emitters[index].drawing = false;
                self.stop_wrap_spot_emitter(index);
            }
        } else {
            let mut cursor_area_no = -1i32;
            if self.stick.check_right_trigger() {
                cursor_area_no = warp_area_no_right(table);
            } else if self.stick.check_left_trigger() {
                cursor_area_no = warp_area_no_left(table);
            } else if self.stick.check_up_trigger() {
                cursor_area_no = warp_area_no_up(table);
            } else if self.stick.check_down_trigger() {
                cursor_area_no = warp_area_no_down(table);
            }
            if cursor_area_no >= 0 {
                ctx.out.sound(SE_CHART_CURSOR);
                self.save.set_cur_wx(self.warp_area_grid_x(cursor_area_no));
                self.save.set_cur_wy(self.warp_area_grid_y(cursor_area_no));
                self.area_text_change_anime_init(ctx);
                self.sel_cursor_move_warp();
                self.warp_area_anime0();
            }
        }
        self.area_text_change_anime();
        let x = self.panes.area_txt[2].pos_top_left_orig.x;
        let y = self.panes.area_txt[2].pos_top_left_orig.y;
        self.out_font.set_left_up_pos(x, y);
        self.out_font.move_step();
        self.sel_cursor_anime();
        let ratio = f32::from(self.frame_counter) / f32::from(self.tuning.warp_back_color_cycle);
        self.frame_counter = self.frame_counter.wrapping_add(1);
        let counter = self.frame_counter;
        let span = u32::from(self.tuning.warp_back_color_cycle) * 4;
        self.frame_counter = (u32::from(counter) - (u32::from(counter) / span) * span) as u8;
        self.set_dsp_warp_back_corner_color(ratio);
    }

    pub fn wrap_sel_win_fade_in(&mut self) {
        let back_alpha = self.pane_alpha_warp_msg_back(
            self.frame_timer
                .wrapping_sub(i16::from(self.tuning.warp_msg_back_in_delay)),
            self.tuning.warp_msg_back_alpha_frame,
            0,
            0,
        );
        let msg = pane_trance_warp_msg(
            &mut self.panes.cc01,
            self.frame_timer
                .wrapping_sub(i16::from(self.tuning.warp_msg_delay)),
            self.tuning.warp_msg_anim_frame,
            self.tuning.warp_msg_pos_y,
            40.0,
            0,
            0,
        );
        let mut msg2 = false;
        if msg {
            let frame = self.frame_timer.wrapping_sub(
                i16::from(self.tuning.warp_msg_delay)
                    .wrapping_add(i16::from(self.tuning.warp_msg_anim_frame)),
            );
            msg2 = pane_trance_warp_msg(
                &mut self.panes.cc01,
                frame,
                self.tuning.warp_msg_anim_frame,
                40.0,
                0.0,
                0,
                2,
            );
        }
        let yes_move = pane_trance_warp_msg(
            &mut self.panes.ysk0,
            self.frame_timer,
            self.tuning.warp_yes_no_anim_frame,
            self.tuning.warp_yes_no_pos_y,
            0.0,
            1,
            0,
        );
        let no_move = pane_trance_warp_msg(
            &mut self.panes.nok0,
            self.frame_timer
                .wrapping_sub(i16::from(self.tuning.warp_no_delay)),
            self.tuning.warp_yes_no_anim_frame,
            self.tuning.warp_yes_no_pos_y,
            0.0,
            1,
            0,
        );
        self.frame_timer = self.frame_timer.wrapping_add(1);
        if back_alpha && msg && msg2 && yes_move && no_move {
            self.frame_timer = 0;
            self.panes.cur1.show();
            self.warp_proc_idx = WARP_PROC_SELECT;
        }
    }

    pub fn wrap_select(&mut self, ctx: &mut MapContext<'_>) {
        if ctx.pad.trig_a {
            if self.warp_sub_state == WARP_SEL_YES {
                ctx.out.sound(SE_SHIPPU_WARP_OK);
                self.warp_proc_idx = WARP_PROC_SEL_WARP;
            } else {
                ctx.out.sound(SE_SHIPPU_WARP_CANCEL);
                self.warp_proc_idx = WARP_PROC_SEL_WIN_FADE_OUT;
            }
            self.panes.cur1.hide();
        } else if ctx.pad.trig_b {
            ctx.out.sound(SE_SHIPPU_WARP_CANCEL);
            self.warp_proc_idx = WARP_PROC_SEL_WIN_FADE_OUT;
            self.panes.cur1.hide();
        } else if self.stick.check_right_trigger() {
            if self.warp_sub_state != WARP_SEL_NO {
                self.warp_sub_state = WARP_SEL_NO;
                ctx.out.sound(SE_TALK_CURSOR);
                self.warp_sel_cursor_move();
                self.panes
                    .ys01
                    .set_black_white(WARP_NO_SHADOW, WARP_NO_BODY);
                self.panes
                    .ys00
                    .set_black_white(WARP_NO_SHADOW, WARP_NO_BODY);
                self.panes
                    .no01
                    .set_black_white(self.ys01_color1, self.ys01_color2);
                self.panes
                    .no00
                    .set_black_white(self.ys01_color1, self.ys01_color2);
            }
        } else if self.stick.check_left_trigger() {
            if self.warp_sub_state != WARP_SEL_YES {
                ctx.out.sound(SE_TALK_CURSOR);
                self.warp_sub_state = WARP_SEL_YES;
                self.warp_sel_cursor_move();
                self.panes
                    .no01
                    .set_black_white(WARP_NO_SHADOW, WARP_NO_BODY);
                self.panes
                    .no00
                    .set_black_white(WARP_NO_SHADOW, WARP_NO_BODY);
                self.panes
                    .ys01
                    .set_black_white(self.ys01_color1, self.ys01_color2);
                self.panes
                    .ys00
                    .set_black_white(self.ys01_color1, self.ys01_color2);
            }
        }
        self.warp_sel_cursor_anime();
    }

    pub fn wrap_sel_win_fade_out(&mut self) {
        let back_alpha = self.pane_alpha_warp_msg_back(
            self.frame_timer
                .wrapping_sub(i16::from(self.tuning.warp_msg_back_out_delay)),
            self.tuning.warp_msg_back_alpha_frame,
            0,
            1,
        );
        let msg = pane_trance_warp_msg(
            &mut self.panes.cc01,
            self.frame_timer
                .wrapping_sub(i16::from(self.tuning.warp_msg_delay)),
            self.tuning.warp_msg_anim_frame,
            0.0,
            40.0,
            0,
            2,
        );
        let mut msg2 = false;
        if msg {
            let frame = self.frame_timer.wrapping_sub(
                i16::from(self.tuning.warp_msg_delay)
                    .wrapping_add(i16::from(self.tuning.warp_msg_anim_frame)),
            );
            msg2 = pane_trance_warp_msg(
                &mut self.panes.cc01,
                frame,
                self.tuning.warp_msg_anim_frame,
                0.0,
                self.tuning.warp_msg_pos_y,
                0,
                1,
            );
        }
        let yes_move = pane_trance_warp_msg(
            &mut self.panes.ysk0,
            self.frame_timer,
            self.tuning.warp_yes_no_anim_frame,
            0.0,
            self.tuning.warp_yes_no_pos_y,
            0,
            1,
        );
        let no_move = pane_trance_warp_msg(
            &mut self.panes.nok0,
            self.frame_timer
                .wrapping_sub(i16::from(self.tuning.warp_no_delay)),
            self.tuning.warp_yes_no_anim_frame,
            0.0,
            self.tuning.warp_yes_no_pos_y,
            0,
            1,
        );
        self.frame_timer = self.frame_timer.wrapping_add(1);
        if back_alpha && msg && msg2 && yes_move && no_move {
            self.frame_timer = 0;
            for index in 0..WARP_SPOT_COUNT {
                self.warp_spot_emitters[index].drawing = true;
            }
            self.warp_proc_idx = WARP_PROC_MOVE;
        }
    }

    pub fn wrap_sel_warp(&mut self, ctx: &mut MapContext<'_>) {
        let ysk_warp = pane_trance_warp_msg(
            &mut self.panes.ysk0,
            self.frame_timer,
            self.tuning.warp_yes_no_anim_frame,
            0.0,
            self.tuning.warp_yes_no_pos_y,
            0,
            1,
        );
        let nok_warp = pane_trance_warp_msg(
            &mut self.panes.nok0,
            self.frame_timer
                .wrapping_sub(i16::from(self.tuning.warp_no_delay)),
            self.tuning.warp_yes_no_anim_frame,
            0.0,
            self.tuning.warp_yes_no_pos_y,
            0,
            1,
        );
        self.frame_timer = self.frame_timer.wrapping_add(1);

        if ysk_warp && nok_warp {
            self.frame_timer = 0;
            let table = self.warp_area_table(self.save.get_cur_wx(), self.save.get_cur_wy());
            let area_no = warp_area_no(table);

            if ctx.env.has_ship {
                ctx.out.ship_warp_pos.push(area_no);
            }

            self.map_close = true;
            self.stop_wrap_back_emitter();
            for index in 0..WARP_SPOT_COUNT {
                self.warp_spot_emitters[index].drawing = false;
                self.stop_wrap_spot_emitter(index);
            }
        }
    }

    pub fn warp_area_anime0(&mut self) {
        let table = self.warp_area_table(self.save.get_cur_wx(), self.save.get_cur_wy());
        let area_no = warp_area_no(table);

        for index in 0..WARP_SPOT_COUNT {
            let scale = if index as i32 == area_no {
                self.tuning.warp_spot_scale_sel
            } else {
                self.tuning.warp_spot_scale
            };
            let emitter = &mut self.warp_spot_emitters[index];
            emitter.x = self.tuning.warp_spot_pos_x[index];
            emitter.y = self.tuning.warp_spot_pos_y[index];
            emitter.scale_x = scale;
            emitter.scale_y = scale;
            emitter.scale_z = 0.0;
        }
    }

    pub fn pane_alpha_warp_msg_back(&mut self, frame: i16, max: u8, mode: u8, flag: i32) -> bool {
        if frame < 0 {
            return false;
        }
        if frame > i16::from(max) {
            return true;
        }
        let mut alpha = value_increase(i32::from(max), i32::from(frame), mode);
        if flag != 2 {
            if flag == 1 {
                alpha = 1.0 - alpha;
            }
            self.panes.wts1.set_now_alpha(alpha);
            self.panes.wts1.set_alpha();
        }
        false
    }

    pub fn warp_sel_cursor_move(&mut self) {
        let x = f32::from(self.warp_sub_state) * 139.0;
        self.panes.cur1.pane_trans(x, 0.0);
    }

    pub fn warp_sel_cursor_anime_init(&mut self) {
        self.panes.cur1.now_alpha = 0;
        self.panes.cur1.user_area = 0;
        self.warp_blink_toggle = 0;
        self.panes.cur1.set_alpha();
    }

    pub fn warp_sel_cursor_anime(&mut self) {
        let mut alpha = value_increase(
            i32::from(self.tuning.warp_cursor_blink_frame),
            i32::from(self.panes.cur1.user_area),
            0,
        );
        if self.warp_blink_toggle != 0 {
            alpha = 1.0 - alpha;
        }
        let range = f32::from(self.tuning.warp_cursor_alpha_max)
            - f32::from(self.tuning.warp_cursor_alpha_min);
        self.panes.cur1.now_alpha =
            (f32::from(self.tuning.warp_cursor_alpha_min) + (range * alpha) as u32 as f32) as u8;
        self.panes.cur1.set_alpha();

        if self.panes.cur1.user_area == 0 {
            self.panes.cur1.user_area = i16::from(self.tuning.warp_cursor_blink_frame);
            self.warp_blink_toggle ^= 1;
        } else {
            self.panes.cur1.user_area -= 1;
        }
    }

    pub fn warp_area_grid_x(&self, no: i32) -> i8 {
        warp_area_grid_x(no)
    }

    pub fn warp_area_grid_y(&self, no: i32) -> i8 {
        warp_area_grid_y(no)
    }

    pub fn warp_area_no(&self, table: Option<usize>) -> i32 {
        warp_area_no(table)
    }

    pub fn warp_area_no_up(&self, table: Option<usize>) -> i32 {
        warp_area_no_up(table)
    }

    pub fn warp_area_no_down(&self, table: Option<usize>) -> i32 {
        warp_area_no_down(table)
    }

    pub fn warp_area_no_left(&self, table: Option<usize>) -> i32 {
        warp_area_no_left(table)
    }

    pub fn warp_area_no_right(&self, table: Option<usize>) -> i32 {
        warp_area_no_right(table)
    }

    pub fn warp_area_table(&self, grid_x: i8, grid_y: i8) -> Option<usize> {
        let mut index = 0usize;
        while index < WARP_SPOT_COUNT {
            if CURSOR_TABLE[index].grid_x == grid_x && CURSOR_TABLE[index].grid_y == grid_y {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    pub fn area_text_change_anime_init(&mut self, ctx: &mut MapContext<'_>) {
        self.warp_anim_timer = self.tuning.area_text_change_frame;
        self.area_txt_buf_idx = 0;
        self.panes.area_txt[0].clear_text();
        self.txt_name[0].clear();
        self.txt_name_msg[0] = None;
        let grid = grid_index(self.save.get_cur_wx(), self.save.get_cur_wy());

        if ctx.flags.is_save_arrive_grid(grid) {
            if is_main_island(grid) {
                self.panes.area_txt[1].set_char_color(self.tuning.area_text_color_main);
            } else {
                self.panes.area_txt[1].set_char_color(self.tuning.area_text_color);
            }
            let msg_no = grid as u32 + MSG_GRID_NAME_BASE;
            let text = ctx.text.message(msg_no);
            self.panes.area_txt[1].set_message(msg_no, Some(&text));
            self.txt_name[1] = text;
            self.txt_name_msg[1] = Some(msg_no);
        } else {
            self.panes.area_txt[1].clear_text();
            self.txt_name[1].clear();
            self.txt_name_msg[1] = None;
        }
        self.out_font.message_set(MSG_UNKNOWN_AREA);
        self.panes.area_txt[1].set_now_alpha(1.0);
        self.panes.area_txt[1].set_alpha();
        self.panes.area_txt[0].set_now_alpha(0.0);
        self.panes.area_txt[0].set_alpha();
        self.out_font.set_alpha(0);
    }

    pub fn area_text_change_anime(&mut self) {
        if self.warp_anim_timer == 0 {
            let alpha = self.pane_alpha_area_txt(
                i32::from(self.area_txt_timer),
                self.tuning.area_text_alpha_frame,
                1,
            );
            self.area_txt_timer = self.area_txt_timer.wrapping_add(1);
            if alpha {
                self.area_txt_buf_idx ^= 1;
                self.area_txt_timer = 0;
                self.warp_anim_timer = self.tuning.area_text_change_frame;
            }
        } else {
            self.warp_anim_timer -= 1;
        }
    }

    pub fn pane_alpha_area_txt(&mut self, value: i32, max: u8, flag: i32) -> bool {
        if value < 0 {
            return false;
        }
        if value > i32::from(max) {
            return true;
        }
        let in_alpha = value_increase(i32::from(max), value, 0);
        let out_alpha = 1.0 - in_alpha;
        let cur = self.area_txt_buf_idx as i32;
        let first = (cur + flag) as usize % 3;
        let second = ((cur ^ 1) + flag) as usize % 3;
        self.panes.area_txt[first].set_now_alpha(out_alpha);
        if first == 2 {
            self.out_font.set_alpha((out_alpha * 255.0) as u8);
        }
        self.panes.area_txt[second].set_now_alpha(in_alpha);
        if second == 2 {
            self.out_font.set_alpha((in_alpha * 255.0) as u8);
        }
        let base = flag as usize % 3;
        self.panes.area_txt[base].set_alpha();
        self.panes.area_txt[(base + 1) % 3].set_alpha();
        false
    }

    pub fn set_dsp_warp_back_corner_color(&mut self, ratio: f32) {
        let color1 = self.tuning.warp_back_color1;
        let color2 = self.tuning.warp_back_color2;
        let rgb: [[u8; 3]; 4] = [
            [
                (color1 >> 24) as u8,
                (color1 >> 16) as u8,
                (color1 >> 8) as u8,
            ],
            [
                (color2 >> 24) as u8,
                (color2 >> 16) as u8,
                (color2 >> 8) as u8,
            ],
            [
                (color2 >> 24) as u8,
                (color2 >> 16) as u8,
                (color2 >> 8) as u8,
            ],
            [
                (color2 >> 24) as u8,
                (color2 >> 16) as u8,
                (color2 >> 8) as u8,
            ],
        ];

        let phase = ratio % 4.0;
        let idx = phase as i32;
        let alpha = phase - idx as f32;

        let mut color = [0u32; 4];
        for index in 0..4usize {
            let next = if index + 1 >= 4 { 0 } else { index + 1 };
            let r = line_inter(rgb[index][0], rgb[next][0], alpha);
            let g = line_inter(rgb[index][1], rgb[next][1], alpha);
            let b = line_inter(rgb[index][2], rgb[next][2], alpha);
            color[index] = (u32::from(r) << 24) | (u32::from(g) << 16) | (u32::from(b) << 8) | 255;
        }

        let idx0 = idx.rem_euclid(4) as usize;
        let idx1 = (idx + 1).rem_euclid(4) as usize;
        let idx2 = (idx + 2).rem_euclid(4) as usize;
        let idx3 = (idx + 3).rem_euclid(4) as usize;

        self.panes
            .cl
            .set_corner_color(color[idx0], color[idx1], color[idx3], color[idx2]);
    }

    pub fn set_wrap_back_emitter(&mut self, x: f32, y: f32, z: f32) {
        self.warp_back_emitter = WarpEmitter {
            id: PARTICLE_WARP_BACK,
            x,
            y,
            z,
            scale_x: 1.0,
            scale_y: 1.0,
            scale_z: 0.0,
            alive: true,
            drawing: true,
        };
    }

    pub fn set_wrap_spot_emitter(&mut self, index: usize, x: f32, y: f32, z: f32) {
        if let Some(emitter) = self.warp_spot_emitters.get_mut(index) {
            *emitter = WarpEmitter {
                id: PARTICLE_WARP_SPOT,
                x,
                y,
                z,
                scale_x: 1.0,
                scale_y: 1.0,
                scale_z: 0.0,
                alive: true,
                drawing: true,
            };
        }
    }

    pub fn open_fishman_mode(&mut self, ctx: &mut MapContext<'_>) -> bool {
        if self.frame_timer == 0 {
            self.init_fishman_mode(ctx);
        }
        let ret = self.pane_trans_base(
            self.frame_timer,
            self.tuning.fishman_open_frame,
            0.0,
            0.0,
            0,
            0,
        );
        self.frame_timer = self.frame_timer.wrapping_add(1);
        if ret {
            self.frame_timer = 0;
            self.fishman_proc_idx = FISHMAN_PROC_DISP_AREA;
            return true;
        }
        false
    }

    pub fn init_fishman_mode(&mut self, ctx: &mut MapContext<'_>) {
        self.fmap_mode = MODE_FISHMAN;
        self.button_icon_mode = BTN_ICON_FISHMAN;
        self.sel_cursor_hide();
        for index in 0..3usize {
            self.panes.ck1[index].hide();
            self.panes.ck2[index].hide();
        }
        for index in 0..2usize {
            self.panes.ck1[index].hide();
        }
        for index in 0..8usize {
            self.panes.kr0[index].hide();
        }
        for index in 0..49usize {
            self.panes.stx[index].hide();
        }
        self.panes.lnk2.hide();
        self.panes.spi2.hide();
        self.panes.stm1.hide();
        self.panes.lnk3.hide();
        self.panes.spi3.hide();
        self.panes.stl1.hide();
        for index in 0..8usize {
            self.panes.kk3[index].hide();
        }

        self.player_point_grid_anime_init();
        self.player_point_grid_anime();
        let x = f32::from(self.grid_x) * GRID_STEP;
        let y = f32::from(self.grid_y) * GRID_STEP;
        self.panes.kkdm.pane_trans(x, y);
        self.set_disp_island_pos(self.grid_x, self.grid_y, ctx);
        let gx = self.grid_x;
        let gy = self.grid_y;
        zoom_map_alpha_set(gx, gy, &mut self.panes.r01b, 0x14);
        zoom_map_alpha_set(gx, gy, &mut self.panes.r01g, 0x1E);
        self.panes.tsw1.hide();
        self.panes.r01b.hide();
        self.panes.r01g.hide();

        for index in 0..3usize {
            self.panes.area_txt[index].set_now_alpha(0.0);
            self.panes.area_txt[index].set_alpha();
        }
        let grid = grid_index(self.grid_x, self.grid_y);
        if let Some(source) = self.area_panes.get(grid.max(0) as usize) {
            self.panes.mr01 = source.clone();
        }
        self.fishman_timer2 = self.tuning.fishman_blink_frame;
        self.fishman_timer3 = self.tuning.fishman_disp_wait;
        self.fishman_proc_idx = FISHMAN_PROC_DISP_AREA;
        ctx.out.sound(SE_CM_SO_DRAW_MAP);
    }

    pub fn move_fishman_mode(&mut self, ctx: &mut MapContext<'_>) {
        match self.fishman_proc_idx {
            FISHMAN_PROC_DISP_AREA => self.fm_disp_area(ctx),
            FISHMAN_PROC_ZOOM_LV1_IN => self.fm_zoom_grid_lv1_in(ctx),
            FISHMAN_PROC_ZOOM_LV2_IN => self.fm_zoom_grid_lv2_in(ctx),
            FISHMAN_PROC_MAP_WRITE => self.fm_map_write(ctx),
            FISHMAN_PROC_MAP_WAIT => self.fm_map_wait(ctx),
            FISHMAN_PROC_ZOOM_LV2_OUT => self.fm_zoom_grid_lv2_out(ctx),
            FISHMAN_PROC_ZOOM_LV1_OUT => self.fm_zoom_grid_lv1_out(),
            _ => self.fm_end_wait(),
        }
    }

    pub fn fm_disp_area(&mut self, ctx: &mut MapContext<'_>) {
        if self.fishman_timer2 == 0 {
            if self.panes.mr01.is_visible() {
                self.panes.mr01.hide();
            } else {
                self.panes.mr01.show();
            }
            self.fishman_timer2 = self.tuning.fishman_blink_frame;
        } else {
            self.fishman_timer2 -= 1;
        }

        if self.fishman_timer3 == 0 {
            self.panes.mr01.hide();
            ctx.out
                .arrive_grids
                .push(grid_index(self.grid_x, self.grid_y));
            self.panes.kkdm.show();
            self.panes.smsk.show();
            self.panes.clb.show();
            self.panes.ktx1.hide();
            self.panes.ktx2.hide();
            self.panes.clb2.hide();
            ctx.out.sound(SE_CHART_ZOOM_IN);
            self.fishman_proc_idx = FISHMAN_PROC_ZOOM_LV1_IN;
        } else {
            self.fishman_timer3 -= 1;
        }
    }

    pub fn fm_zoom_grid_lv1_in(&mut self, ctx: &mut MapContext<'_>) {
        let sel_grid_mask =
            self.sel_grid_mask_alpha_ctrl(self.frame_timer, self.tuning.fishman_zoom1_frame, 0, 0);
        let fmap_mask =
            self.fmap_mask_alpha_ctrl(self.frame_timer, self.tuning.fishman_zoom1_frame, 0, 0);
        let (tx, ty, ts) = {
            let x = self.panes.kkdm.pos_center.x - self.panes.clb.pos_center_orig.x;
            let y = self.panes.kkdm.pos_center.y - self.panes.clb.pos_center_orig.y;
            let scale = self.panes.kkdm.size_orig.x / self.panes.clb.size_orig.x;
            (x, y, scale)
        };
        let zoom_map = self.pane_trance_zoom_map(
            self.frame_timer
                .wrapping_sub(i16::from(self.tuning.zoom_map_delay)),
            self.tuning.fishman_zoom1_frame,
            tx,
            ty,
            0.0,
            0.0,
            ts,
            1.0,
            2,
            0,
        );
        self.frame_timer = self.frame_timer.wrapping_add(1);
        if sel_grid_mask && fmap_mask && zoom_map {
            ctx.out.sound(SE_CHART_ZOOM_IN);
            self.frame_timer = 0;
            self.panes.clg.show();
            for index in 0..8usize {
                self.panes.kk3[index].set_influenced_alpha(false);
            }
            self.panes.r01b.set_influenced_alpha(false);
            self.fishman_proc_idx = FISHMAN_PROC_ZOOM_LV2_IN;
        }
    }

    pub fn fm_zoom_grid_lv2_in(&mut self, ctx: &mut MapContext<'_>) {
        let px = self.disp_island_pos_x * (self.panes.clg.size_orig.x / WORLD_UNIT);
        let py = self.disp_island_pos_y * (self.panes.clg.size_orig.y / WORLD_UNIT);
        let sx = self.panes.tsw1.size_orig.x / self.panes.clg.size_orig.x;
        let zoom_map = self.pane_trance_zoom2_map(
            self.frame_timer,
            self.tuning.fishman_zoom2_frame,
            px,
            py,
            0.0,
            0.0,
            sx,
            1.0,
            2,
            0,
        );
        let zoom_map_alpha = self.pane_trance_zoom_map_alphah(
            self.frame_timer,
            self.tuning.fishman_zoom2_frame,
            0x02,
            1,
        );
        self.frame_timer = self.frame_timer.wrapping_add(1);
        if zoom_map && zoom_map_alpha {
            self.panes.tsw1.show();
            self.panes.r01b.show();
            self.panes.r01g.show();
            self.island_name_set(0, ctx);
            self.pane_alpha_zoom2_map(0, self.tuning.fishman_zoom2_alpha_frame, 2, 0);
            self.fishman_timer1 = self.tuning.fishman_map_write_wait;
            self.fishman_proc_idx = FISHMAN_PROC_MAP_WRITE;
        }
    }

    pub fn island_name_set(&mut self, no: u8, ctx: &mut MapContext<'_>) {
        let grid = grid_index(self.grid_x, self.grid_y);
        let index = no as usize % 3;
        let msg_no = if grid == ISLE_IDX_PRIVATE_OASIS && ctx.flags.is_event_bit(EVENT_UNKNOWN_2D80)
        {
            MSG_PRIVATE_ISLAND
        } else {
            grid as u32 + MSG_GRID_NAME_BASE
        };
        let color = if is_main_island(grid) {
            self.tuning.area_text_color_main
        } else {
            self.tuning.area_text_color
        };
        self.panes.area_txt[index].set_char_color(color);
        let text = ctx.text.message(msg_no);
        self.panes.area_txt[index].set_message(msg_no, Some(&text));
        self.txt_name[index] = text;
        self.txt_name_msg[index] = Some(msg_no);
    }

    pub fn fm_map_write(&mut self, ctx: &mut MapContext<'_>) {
        if self.fishman_timer1 != 0 {
            self.fishman_timer1 -= 1;
        } else {
            if self.frame_timer == 0 {
                ctx.out.bgm(BGM_BGN_GET_BOX);
            }
            let zoom_map = self.pane_alpha_zoom2_map(
                self.frame_timer,
                self.tuning.fishman_zoom2_alpha_frame,
                0x02,
                0,
            );
            self.frame_timer = self.frame_timer.wrapping_add(1);
            if zoom_map {
                self.frame_timer = 0;
                self.fishman_timer1 = self.tuning.fishman_map_write_frame;
                self.fishman_proc_idx = FISHMAN_PROC_MAP_WAIT;
            }
        }
    }

    pub fn fm_map_wait(&mut self, ctx: &mut MapContext<'_>) {
        if self.fishman_timer1 != 0 {
            self.fishman_timer1 -= 1;
        }
        if ctx.pad.trig_a
            || ctx.pad.trig_b
            || ctx.pad.trig_x
            || ctx.pad.trig_y
            || ctx.pad.trig_l
            || ctx.pad.trig_r
            || ctx.pad.trig_z
            || ctx.pad.trig_start
            || self.fishman_timer1 == 0
        {
            ctx.out.sound(SE_CHART_ZOOM_OUT);
            self.fishman_proc_idx = FISHMAN_PROC_ZOOM_LV2_OUT;
        }
    }

    pub fn pane_alpha_zoom2_map(&mut self, value: i16, max: u8, mode: u8, flag: i32) -> bool {
        if value < 0 {
            return false;
        }
        if value > i16::from(max) {
            return true;
        }
        let mut alpha = value_increase(i32::from(max), i32::from(value), mode);
        if flag != 2 {
            if flag == 1 {
                alpha = 1.0 - alpha;
            }
            self.panes.r01g.set_now_alpha(alpha);
            self.panes.r01g.set_alpha();
            for index in 0..3usize {
                self.panes.area_txt[index].set_now_alpha(alpha);
                self.panes.area_txt[index].set_alpha();
            }
        }
        false
    }

    pub fn fm_zoom_grid_lv2_out(&mut self, ctx: &mut MapContext<'_>) {
        let px = self.disp_island_pos_x * (self.panes.clg.size_orig.x / WORLD_UNIT);
        let py = self.disp_island_pos_y * (self.panes.clg.size_orig.y / WORLD_UNIT);
        let sx = self.panes.tsw1.size_orig.x / self.panes.clg.size_orig.x;
        let zoom_map = self.pane_trance_zoom2_map(
            self.frame_timer,
            self.tuning.fishman_zoom2_frame,
            0.0,
            0.0,
            px,
            py,
            1.0,
            sx,
            2,
            1,
        );
        let zoom_alpha = self.pane_trance_zoom_map_alphah(
            self.frame_timer,
            self.tuning.fishman_zoom2_frame,
            0x02,
            0,
        );
        self.frame_timer = self.frame_timer.wrapping_add(1);
        if zoom_map && zoom_alpha {
            ctx.out.sound(SE_CHART_ZOOM_OUT);
            self.panes.clg.hide();
            for index in 0..8usize {
                self.panes.kk3[index].set_influenced_alpha(true);
            }
            self.panes.r01b.set_influenced_alpha(true);
            self.frame_timer = 0;
            self.fishman_proc_idx = FISHMAN_PROC_ZOOM_LV1_OUT;
        }
    }

    pub fn fm_zoom_grid_lv1_out(&mut self) {
        let sel_grid_mask = self.sel_grid_mask_alpha_ctrl(
            self.frame_timer
                .wrapping_sub(i16::from(self.tuning.zoom_out_delay)),
            self.tuning.zoom_anim_frame,
            0,
            1,
        );
        let fmap_mask = self.fmap_mask_alpha_ctrl(
            self.frame_timer
                .wrapping_sub(i16::from(self.tuning.zoom_out_delay)),
            self.tuning.zoom_anim_frame,
            0,
            1,
        );
        let (tx, ty, ts) = {
            let x = self.panes.kkdm.pos_center.x - self.panes.clb.pos_center_orig.x;
            let y = self.panes.kkdm.pos_center.y - self.panes.clb.pos_center_orig.y;
            let scale = self.panes.kkdm.size_orig.x / self.panes.clb.size_orig.x;
            (x, y, scale)
        };
        let zoom_map = self.pane_trance_zoom_map(
            self.frame_timer
                .wrapping_sub(i16::from(self.tuning.zoom_map_delay)),
            self.tuning.fishman_zoom1_out_frame,
            0.0,
            0.0,
            tx,
            ty,
            1.0,
            ts,
            2,
            1,
        );
        self.frame_timer = self.frame_timer.wrapping_add(1);
        if sel_grid_mask && fmap_mask && zoom_map {
            self.frame_timer = 0;
            self.panes.kkdm.hide();
            self.panes.smsk.hide();
            self.panes.clb.hide();
            self.fishman_timer1 = self.tuning.fishman_close_wait;
            self.fishman_proc_idx = FISHMAN_PROC_END_WAIT;
        }
    }

    pub fn fm_end_wait(&mut self) {
        if self.fishman_timer1 == 0 {
            self.map_close = true;
        } else {
            self.fishman_timer1 -= 1;
        }
    }

    pub fn open_wall_paper(&mut self) -> bool {
        if self.frame_timer == 0 {
            self.button_icon_mode = BTN_ICON_WALLPAPER;
            self.sel_cursor_hide();
        }
        let ret = self.pane_trans_base(
            self.frame_timer,
            self.tuning.base_anim_frame,
            self.tuning.open_pos_y,
            0.0,
            0,
            0,
        );
        self.frame_timer = self.frame_timer.wrapping_add(1);
        if ret {
            self.frame_timer = 0;
            self.fmap_mode = MODE_WALLPAPER;
            return true;
        }
        false
    }

    pub fn get_button_icon_mode(&self) -> i32 {
        let compare_mode = self
            .compare
            .as_ref()
            .map(|compare| compare.button_icon_mode())
            .unwrap_or(0);
        if self.main_proc_idx == MAIN_PROC_HIKAKU {
            return compare_mode + 7;
        }
        if self.input_disabled {
            return compare_mode + 7;
        }
        i32::from(self.button_icon_mode)
    }

    pub fn cursor_position(&self) -> (i8, i8) {
        (self.save.get_cur_x(), self.save.get_cur_y())
    }

    pub fn warp_cursor_position(&self) -> (i8, i8) {
        (self.save.get_cur_wx(), self.save.get_cur_wy())
    }

    pub fn zoom_level(&self) -> u8 {
        self.save.get_zoom()
    }

    pub fn zoom_grid(&self) -> (i8, i8) {
        (self.save.get_zoom_grid_x(), self.save.get_zoom_grid_y())
    }

    pub fn player_grid(&self) -> (i8, i8) {
        (self.grid_x, self.grid_y)
    }

    pub fn ship_grid(&self) -> (i8, i8) {
        (self.target_grid_x, self.target_grid_y)
    }

    pub fn player_marker(&self) -> PlayerPos {
        self.player_pos
    }

    pub fn ship_marker(&self) -> PlayerPos {
        self.ship_pos
    }

    pub fn area_name(&self, index: usize) -> &str {
        self.txt_name.get(index).map(String::as_str).unwrap_or("")
    }
}
