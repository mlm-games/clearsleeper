use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;

use crate::archive::{Archive, yaz0_decode};
use crate::assets::AssetRoot;
use crate::j2d::{Kind, Screen, ScreenPane};
use crate::sim::daynight::DayNight;
use crate::sim::flags::{Arrivals, Charts, EventReg, REG_GHOST_SHIP};
use crate::sim::salvage;

pub const SECTOR_TABLE_FILE: &str = "files/res/FmapDat/FmapDat.bin";
pub const MAP_ARCHIVE: &str = "files/res/Fmap/Fmap.arc";
pub const CHART_TABLE_ARCHIVE: &str = "files/res/Msg/fmapres.arc";
pub const CHART_TABLE_ENTRY: &str = "cmapdat.bin";
pub const MAIN_SCREEN_ENTRY: &str = "scrn/f_hikaku_1.blo";

pub const CHART_COUNT_OFFSET: usize = 0x00;
pub const CHART_LIST_OFFSET: usize = 0x04;
pub const WORD_SIZE: usize = 0x04;
pub const SALVAGE_POINTS: usize = 4;
pub const SALVAGE_SIZE: usize = 0x08;
pub const CHART_ENTRY_SIZE: usize = 0x26;
pub const CHART_EXPECTED_COUNT: i32 = 61;
pub const CHART_EXPECTED_BYTES: usize = 2322;

pub const SECTOR_COUNT_OFFSET: usize = 0x00;
pub const SECTOR_LIST_OFFSET: usize = 0x04;
pub const SECTOR_ENTRY_SIZE: usize = 0x08;
pub const SECTOR_EXPECTED_COUNT: u32 = 49;
pub const SECTOR_EXPECTED_BYTES: usize = 396;
pub const SECTOR_TOTAL: usize = 49;
pub const SECTOR_BIAS: i8 = 3;

pub const TEXTURE_SLOT_SIZE: u32 = 0x2C00;
pub const TEXTURE_PREFIX: &str = "rupy_num_";
pub const TEXTURE_TRIFORCE: &str = "f_cmap_t.bti";
pub const TEXTURE_CHART: &str = "f_cmap_normal.bti";
pub const SECTOR_MARK_PREFIX: &str = "mR";

pub const NO_MESSAGE: u16 = 0xFFFF;
pub const MSG_MAP_READY: u32 = 0x319C;
pub const MSG_TRI_FOUND: u32 = 0x319D;
pub const MSG_MAP_MISSING: u32 = 0x319E;

pub const SE_CHART_CURSOR: u32 = 0x08C3;
pub const SE_CHART_COMP_LR: u32 = 0x08C7;
pub const SE_CHART_OPEN_MAP: u32 = 0x08C8;
pub const SE_CHART_SEL_MAP: u32 = 0x08C9;
pub const SE_KAIZU_RETURN: u32 = 0x090D;
pub const SE_TC_MAP_RETURN: u32 = 0x090E;

pub const TUNE_PANE_FRAME: u8 = 8;
pub const TUNE_PANE_SLIDE: i16 = 0;
pub const TUNE_SCROLL_FRAME: u8 = 8;
pub const TUNE_OPEN_FRAME: u8 = 10;
pub const TUNE_CLOSE_FRAME: u8 = 10;
pub const TUNE_OPEN_SPEED: f32 = 0.05;
pub const TUNE_SCROLL_DX: f32 = 80.0;
pub const TUNE_SCROLL_DY: f32 = 75.0;
pub const TUNE_SCROLL_RATIO: f32 = 0.5;
pub const TUNE_REFLOW_FRAME: u8 = 4;
pub const TUNE_SETTLE_DX: f32 = 0.0;
pub const TUNE_SETTLE_DY: f32 = 0.0;
pub const TUNE_SETTLE_RATIO: f32 = 1.0;
pub const TUNE_SETTLE_FRAME: u8 = 0;
pub const TUNE_SETTLE_LIMIT: u8 = 45;
pub const TUNE_CMYU_AREA: u8 = 2;
pub const TUNE_MOON_DX: u8 = 30;
pub const TUNE_MOON_DY: u8 = 6;
pub const TUNE_MOON_SPEED: u8 = 255;
pub const TUNE_MOON_FROM: u8 = 15;
pub const TUNE_MOON_TO: u8 = 60;
pub const TUNE_ICON_DX: u8 = 30;
pub const TUNE_ICON_DY: u8 = 0;
pub const TUNE_ICON_SPEED: u8 = 255;
pub const TUNE_TRI_DX: u8 = 32;
pub const TUNE_TRI_SPEED: u8 = 15;
pub const TUNE_MOON_CLEAR_TIME: f32 = 285.0;
pub const TUNE_MOON_SET_TIME: f32 = 90.0;

pub const GHOST_SHIP_PROGRESS: u8 = 3;
pub const GS_DOOR_INDEX: usize = 7;
pub const GS_KEY_INDEX: usize = 10;
pub const GS_MOON_FIRST: usize = 11;
pub const GS_ICON_FIRST: usize = 18;
pub const TREASURE_MARKS_FIRST: usize = 9;

pub const STICK_VALUE_ON: f32 = 0.9;
pub const STICK_VALUE_OFF: f32 = 0.5;
pub const STICK_HALF_DEAD: i16 = 0x2000;

pub const AREA_GRID_STEP: f32 = 16.0;
pub const WORLD_SCALE: f32 = 700000.0;
pub const GRID_UNIT: f32 = 100000.0;
pub const SECTOR_BASE: i32 = 4;
pub const SECTOR_STRIDE: i32 = 7;
pub const SHIP_GRID_MODULO: u8 = 7;

pub const CND0_FMAP_ALPHA: f32 = 48.0;
pub const CND0_CMAP_SPAN: f32 = 122.0;
pub const CND0_CMAP_ALPHA: u8 = 170;
pub const MKM0_FMAP_ALPHA: u8 = 64;
pub const MKM0_FMAP_DIM: u8 = 128;
pub const CND0_DIM: u8 = 48;
pub const RUBY_SHIFT: f32 = 4.0;
pub const SHADE_ALPHA: u8 = 64;
pub const SHADE_BLACK: [u8; 4] = [0, 0, 0, 0];
pub const SHADE_SOLID: [u8; 4] = [0, 0, 0, 255];

pub const KIND_TINGLE_MAP: u8 = 3;
pub const KIND_GHOST_SHIP: u8 = 4;
pub const KIND_TREASURE_MAP: u8 = 7;
pub const KIND_DOCTOR_MAP: u8 = 9;
pub const KIND_DARK_PATH_MAP: u8 = 10;
pub const KIND_HEART_PIECE_MAP: u8 = 11;
pub const KIND_HEART_MAP: u8 = 12;
pub const KIND_SUBDAN_MAP: u8 = 13;
pub const KIND_MOON_MAP: u8 = 14;
pub const KIND_TOWER_MAP: u8 = 15;
pub const KIND_TERRY_MAP: u8 = 16;
pub const KIND_SUB_MARINE_MAP: u8 = 17;

pub const PROC_FMAP: u8 = 0;
pub const PROC_FMAP_CHANGE: u8 = 1;
pub const PROC_CMAP: u8 = 2;
pub const PROC_CMAP_IN: u8 = 3;
pub const PROC_CMAP_OPEN: u8 = 4;
pub const PROC_CMAP_LOAD: u8 = 5;
pub const PROC_CMAP_OPEN_SP: u8 = 6;
pub const PROC_CMAP_SCROLL: u8 = 7;
pub const PROC_CMAP_CLOSE: u8 = 8;
pub const PROC_CHANGE_MODE: u8 = 9;

pub const MSG_NAME_PANES: usize = 17;
pub const MS_PANES: usize = 5;
pub const FC_PANES: usize = 8;
pub const CC_PANES: usize = 8;
pub const CHART_PAGE_PANES: usize = 5;
pub const MARKER_PAGE_PANES: usize = 2;
pub const ZOOM_PAGE_PANES: usize = 10;
pub const ISLAND_NAME_PANES: usize = 8;
pub const KEY_PANES: usize = 2;
pub const EXTRA_PANES: usize = 3;
pub const PAGE_COUNT: usize = 2;

pub const ROLL_MAP_TEX: [&str; 18] = [
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
];

pub const NAME_TAGS_A: [&str; MSG_NAME_PANES] = [
    "ck", "ckT0", "ckT1", "cTR0", "cTR1", "ck01", "ck02", "ck03", "ck04", "ck05", "ck06", "ck07",
    "ck08", "ck09", "ck0a", "bt01", "bt02",
];

pub const NAME_TAGS_B: [&str; MSG_NAME_PANES] = [
    "ck2", "cyT1", "cyT2", "cTR2", "cTR3", "ck11", "ck12", "ck13", "ck14", "ck15", "ck16", "ck17",
    "ck18", "ck19", "ck1a", "bt11", "bt12",
];

pub const MS_TAGS: [&str; MS_PANES] = ["msw", "msT", "msT2", "msR1", "msR2"];

pub const FC_TAGS: [&str; FC_PANES] = [
    "Fc01", "Fc02", "Fc03", "Fc04", "Fc11", "Fc12", "Fc13", "Fc14",
];

pub const CC_TAGS: [&str; CC_PANES] = [
    "cc01", "cc02", "cc03", "cc04", "cc11", "cc12", "cc13", "cc14",
];

pub const MARKER_TAGS_A: [&str; MARKER_PAGE_PANES] = ["cm11", "cm12"];
pub const MARKER_TAGS_B: [&str; MARKER_PAGE_PANES] = ["cm21", "cm22"];

pub const CHART_PAGE_TAGS_A: [&str; CHART_PAGE_PANES] = ["clS1", "cm01", "cms1", "NZ01", "cck1"];
pub const CHART_PAGE_TAGS_B: [&str; CHART_PAGE_PANES] = ["clS2", "cm02", "cms2", "NZ02", "cck2"];

pub const ZOOM_PAGE_TAGS_A: [&str; ZOOM_PAGE_PANES] = [
    "cdd1", "clc1", "clyg", "clB1", "lnk4", "cbt1", "cbt2", "cmr1", "cmr3", "clc2",
];

pub const ZOOM_PAGE_TAGS_B: [&str; ZOOM_PAGE_PANES] = [
    "cdd2", "clc3", "cly2", "clB2", "lnk5", "cbt3", "cbt4", "cmr2", "cmr4", "clc4",
];

pub const ISLAND_NAME_TAGS: [&str; ISLAND_NAME_PANES] =
    ["cmt1", "cmt2", "ct3", "ct4", "cmt3", "cmt4", "cmk1", "cmk2"];

pub const KEY_TAGS: [&str; KEY_PANES] = ["fdt1", "fts1"];

pub const EXTRA_TAGS: [&str; EXTRA_PANES] = ["fmk1", "Rkjn", "clg2"];

pub const TEXT_FILL_LONG: usize = 256;
pub const TEXT_FILL_SHORT: usize = 128;

pub const TINGLE_SCREEN: &str = "t_map_01.blo";
pub const DOCTOR_SCREEN: &str = "doc_map_01.blo";
pub const DARK_PATH_SCREEN: &str = "ys_map_1.blo";
pub const HEART_PIECE_SCREEN: &str = "hrt_map_1.blo";
pub const HEART_SCREEN: &str = "hm_map_1.blo";
pub const SUBDAN_SCREEN: &str = "ir_map_1.blo";
pub const MOON_SCREEN: &str = "mn_map_1.blo";
pub const TOWER_SCREEN: &str = "yg_map_1.blo";
pub const TERRY_SCREEN: &str = "terry_map_1.blo";
pub const SUB_MARINE_SCREEN: &str = "sensui_map_1.blo";

pub const SPECIAL_ARCHIVES: [(u8, &str, &str); 12] = [
    (3, "files/res/Msg/tnmapres.arc", TINGLE_SCREEN),
    (4, "files/res/Msg/gsmapres.arc", "g_map_01.blo"),
    (7, "files/res/Msg/trmapres.arc", "tr_map_1.blo"),
    (9, "files/res/Msg/docmapres.arc", DOCTOR_SCREEN),
    (10, "files/res/Msg/ysmapres.arc", DARK_PATH_SCREEN),
    (11, "files/res/Msg/heartmapres.arc", HEART_PIECE_SCREEN),
    (12, "files/res/Msg/htmmapres.arc", HEART_SCREEN),
    (13, "files/res/Msg/irmapres.arc", SUBDAN_SCREEN),
    (14, "files/res/Msg/moonmapres.arc", MOON_SCREEN),
    (15, "files/res/Msg/ygmapres.arc", TOWER_SCREEN),
    (16, "files/res/Msg/terrymapres.arc", TERRY_SCREEN),
    (17, "files/res/Msg/submamapres.arc", SUB_MARINE_SCREEN),
];

pub const TINGLE_TAGS: [&str; 29] = [
    "HK00", "HK01", "HN19", "HK29", "gddm", "lnk", "gdgt", "TWy1", "TWy2", "TWy3", "yg22", "yg21",
    "kb11", "kb12", "kb13", "kb14", "kb15", "kb18", "kb20", "kb21", "kb22", "kb23", "ti03", "ti04",
    "ti05", "ti06", "ti07", "ys01", "ys02",
];

pub const GHOST_SHIP_TAGS: [&str; 25] = [
    "ms01", "ms02", "sd05", "hs04", "yg01", "BSdm", "tkd2", "tkd1", "S02", "kz05", "kz01", "tk01",
    "tk02", "tk03", "tk04", "tk05", "tk06", "tk07", "gsi1", "gsi2", "gsi3", "gsi4", "gsi5", "gsi6",
    "gsi7",
];

pub const TREASURE_TAGS: [&str; 33] = [
    "ms01", "ms02", "hs04", "sd05", "kz05", "gddm", "mkdm", "lnk", "gdgt", "mg1", "mg2", "mg3",
    "mg4", "mg5", "mg6", "mg7", "mg8", "tf1", "tf2", "tf3", "tf4", "tf5", "tf6", "tf7", "tf8",
    "tg1", "tg2", "tg3", "tg4", "tg5", "tg6", "tg7", "tg8",
];

pub const DOCTOR_TAGS: [&str; 8] = [
    "ms01", "ms02", "hs04", "gddm", "mkdm", "sd05", "lnk", "gdgt",
];

pub const DARK_PATH_TAGS: [&str; 7] = ["ms01", "ms02", "hs04", "gddm", "sd05", "lnk", "gdgt"];

pub const TERRY_TAGS: [&str; 7] = ["ms01", "ms02", "hs04", "gddm", "sd05", "lnk", "gdgt"];

pub const SUB_MARINE_TAGS: [&str; 7] = ["ms01", "ms02", "hs04", "gddm", "sd05", "lnk", "gdgt"];

pub const HEART_PIECE_TAGS: [&str; 12] = [
    "ms01", "ms02", "hs04", "sd05", "gddm", "mk20", "nm08", "kk12", "bh1", "bh2", "lnk", "gdgt",
];

pub const HEART_TAGS: [&str; 24] = [
    "ms01", "ms02", "hs04", "sd05", "gddm", "st09", "lnk", "gdgt", "nh1", "nh2", "nh4", "nh5",
    "nh6", "nh7", "nh9", "mkn2", "mkn4", "mk2", "mk3", "mk5", "mk6", "mk7", "mk8", "mk9",
];

pub const SUBDAN_TAGS: [&str; 10] = [
    "ms01", "ms02", "hs04", "gddm", "sd05", "IG26", "lnk", "gdgt", "IGN3", "IGN4",
];

pub const MOON_TAGS: [&str; 45] = [
    "ms01", "ms02", "hs04", "gddm", "sd05", "lnk", "gdgt", "tk00", "tk01", "tk02", "tk03", "tk04",
    "tk05", "tk06", "tk07", "tk08", "tk09", "tk10", "titl", "td1", "td2", "kr20", "kr21", "kr22",
    "kr23", "kr24", "kr01", "kr02", "kr03", "kr04", "kr05", "kr06", "kr07", "kr08", "kr09", "kr10",
    "kr11", "kr12", "kr13", "kr14", "kr15", "kr16", "kr17", "kr18", "kr19",
];

pub const TOWER_TAGS: [&str; 9] = [
    "ms01", "ms02", "hs04", "sd05", "gddm", "ttl1", "ttl2", "lnk", "gdgt",
];

pub const UNRESOLVED_STEPS: [&str; 45] = [
    "ghost_ship_moon_anime",
    "ghost_ship_ship_anime",
    "ghost_ship_icon_anime",
    "treasure_triforce_anime",
    "change_select_chart_special",
    "chart_alpha_set",
    "change_zoom_chart",
    "change_proc_mode",
    "open",
    "close",
    "draw",
    "delete",
    "field_cursor_init",
    "field_cursor_move",
    "field_cursor_anime",
    "chart_cursor_anime_init",
    "chart_cursor_anime",
    "chart_cursor_hide",
    "chart_select_cursor_init",
    "chart_select_cursor_anime_init",
    "chart_select_cursor_anime",
    "chart_select_cursor_hide",
    "player_grid_anime_init",
    "player_grid_anime",
    "change_island_name",
    "field_player_pos_disp",
    "field_player_pos_disp_check",
    "change_chart_name",
    "chart_player_pos_disp",
    "chart_salvage_pos_disp",
    "chart_player_pos_disp_check",
    "pane_trance_message",
    "pane_trans_select_chart_open",
    "pane_alpha_ghost_ship_map",
    "pane_alpha_tingle_map",
    "pane_alpha_treasure_map",
    "pane_alpha_subdan_map",
    "pane_alpha_heart_map",
    "pane_alpha_yagura_map",
    "pane_alpha_dark_path_map",
    "pane_alpha_heart_piece_map",
    "pane_alpha_terry_map",
    "pane_alpha_submarine_map",
    "pane_alpha_moon_map",
    "pane_alpha_doctor_map",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct SalvagePoint {
    pub center_x: i16,
    pub center_y: i16,
    pub x: i16,
    pub y: i16,
}

impl SalvagePoint {
    pub fn parse(data: &[u8], at: usize) -> Option<Self> {
        Some(Self {
            center_x: be_i16(data, at)?,
            center_y: be_i16(data, at + 2)?,
            x: be_i16(data, at + 4)?,
            y: be_i16(data, at + 6)?,
        })
    }

    pub fn corner(&self, side_x: i16, side_y: i16) -> (i16, i16) {
        (
            self.center_x.wrapping_add(self.x.wrapping_mul(side_x)),
            self.center_y.wrapping_add(self.y.wrapping_mul(side_y)),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ChartEntry {
    pub grid_no: i8,
    pub collect_map_no: i8,
    pub chart_no: i8,
    pub map_type: i8,
    pub group: i16,
    pub salvage: [SalvagePoint; SALVAGE_POINTS],
}

impl ChartEntry {
    pub fn byte_size() -> usize {
        6 + SALVAGE_POINTS * SALVAGE_SIZE
    }

    pub fn parse(data: &[u8], at: usize) -> Option<Self> {
        let mut salvage = [SalvagePoint::default(); SALVAGE_POINTS];
        for (index, point) in salvage.iter_mut().enumerate() {
            *point = SalvagePoint::parse(data, at + 6 + index * SALVAGE_SIZE)?;
        }
        Some(Self {
            grid_no: be_i8(data, at)?,
            collect_map_no: be_i8(data, at + 1)?,
            chart_no: be_i8(data, at + 2)?,
            map_type: be_i8(data, at + 3)?,
            group: be_i16(data, at + 4)?,
            salvage,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChartTable {
    entries: Vec<ChartEntry>,
}

impl ChartTable {
    pub fn set_data(&mut self, data: &[u8]) -> bool {
        let Some(count) = read_i32(data, CHART_COUNT_OFFSET) else {
            return false;
        };
        let Some(entries) = read_charts(data, count) else {
            return false;
        };
        self.entries = entries;
        true
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[ChartEntry] {
        &self.entries
    }

    pub fn at(&self, index: usize) -> Option<&ChartEntry> {
        self.entries.get(index)
    }

    pub fn by_grid(&self, grid_index: i32) -> Option<&ChartEntry> {
        let want = i8::try_from(grid_index.wrapping_add(1)).ok()?;
        self.entries.iter().find(|entry| entry.grid_no == want)
    }

    pub fn by_chart(&self, chart_index: i32) -> Option<&ChartEntry> {
        let want = i8::try_from(chart_index.wrapping_add(1)).ok()?;
        self.entries.iter().find(|entry| entry.chart_no == want)
    }

    pub fn collect_map_no(&self, chart_index: i32) -> Option<i8> {
        self.by_chart(chart_index).map(|entry| entry.collect_map_no)
    }

    pub fn map_type(&self, chart_index: i32) -> Option<i8> {
        self.by_chart(chart_index).map(|entry| entry.map_type)
    }

    pub fn salvage_point(&self, grid_index: i32, point: usize) -> Option<&SalvagePoint> {
        self.by_grid(grid_index)?.salvage.get(point)
    }
}

pub fn salvage_texture_name(grid_no: i8, salvage_point: u8) -> String {
    format!("R{:02X}_col_{}.bti", grid_no as u8, salvage_point)
}

pub fn world_texture_name(sector: i32) -> String {
    format!("R{:02X}_b.bti", sector as u8)
}

pub fn sector_mark_tag(sector: usize) -> String {
    format!("{}{:02X}", SECTOR_MARK_PREFIX, sector as u8)
}

pub fn ruby_digit_texture(digit: i32) -> String {
    format!("{TEXTURE_PREFIX}{:02}.bti", digit)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct SectorCheck {
    pub sector_x: i8,
    pub sector_y: i8,
    pub x: i16,
    pub y: i16,
    pub flag: u8,
}

impl SectorCheck {
    pub fn parse(data: &[u8], at: usize) -> Option<Self> {
        Some(Self {
            sector_x: be_i8(data, at)?,
            sector_y: be_i8(data, at + 1)?,
            x: be_i16(data, at + 2)?,
            y: be_i16(data, at + 4)?,
            flag: *data.get(at + 6)?,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SectorTable {
    points: Vec<SectorCheck>,
}

impl SectorTable {
    pub fn set_data(&mut self, data: &[u8]) -> bool {
        let Some(count) = read_u32(data, SECTOR_COUNT_OFFSET) else {
            return false;
        };
        let Some(points) = read_sectors(data, count) else {
            return false;
        };
        self.points = points;
        true
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    pub fn at(&self, index: usize) -> Option<&SectorCheck> {
        self.points.get(index)
    }

    pub fn use_grid(&self, x: i8, y: i8) -> i32 {
        for (index, point) in self.points.iter().enumerate() {
            if point.sector_x == x && point.sector_y == y {
                return index as i32;
            }
        }
        -1
    }
}

pub fn read_charts(data: &[u8], count: i32) -> Option<Vec<ChartEntry>> {
    let count = usize::try_from(count).ok()?;
    let end = CHART_LIST_OFFSET.checked_add(count.checked_mul(CHART_ENTRY_SIZE)?)?;
    data.get(CHART_LIST_OFFSET..end)?;
    let mut entries = Vec::with_capacity(count);
    for index in 0..count {
        entries.push(ChartEntry::parse(
            data,
            CHART_LIST_OFFSET + index * CHART_ENTRY_SIZE,
        )?);
    }
    Some(entries)
}

pub fn read_sectors(data: &[u8], count: u32) -> Option<Vec<SectorCheck>> {
    let count = usize::try_from(count).ok()?;
    let end = SECTOR_LIST_OFFSET.checked_add(count.checked_mul(SECTOR_ENTRY_SIZE)?)?;
    data.get(SECTOR_LIST_OFFSET..end)?;
    let mut points = Vec::with_capacity(count);
    for index in 0..count {
        points.push(SectorCheck::parse(
            data,
            SECTOR_LIST_OFFSET + index * SECTOR_ENTRY_SIZE,
        )?);
    }
    Some(points)
}

pub fn decode_asset(raw: &[u8], label: &str) -> Option<Vec<u8>> {
    if raw.len() >= 4 && raw[..4] == *b"Yaz0" {
        yaz0_decode(raw, label).ok()
    } else {
        Some(raw.to_vec())
    }
}

pub fn set_pane_on_off(screen: &mut Screen, tag: &str, visible: bool) {
    let Some(index) = screen.panes.iter().position(|pane| pane.name == tag) else {
        return;
    };
    let pane = &mut screen.panes[index];
    if visible {
        pane.visible = true;
    } else {
        pane.visible = false;
    }
}

pub fn read_archive_entry(root: &AssetRoot, archive_rel: &str, entry: &str) -> Option<Vec<u8>> {
    let archive = root.read_archive(archive_rel).ok()?;
    let raw = archive.read(entry).ok()?;
    decode_asset(raw, entry)
}

pub fn load_chart_table(root: &AssetRoot) -> Option<ChartTable> {
    let raw = read_archive_entry(root, CHART_TABLE_ARCHIVE, CHART_TABLE_ENTRY)?;
    let mut table = ChartTable::default();
    if !table.set_data(&raw) {
        log::warn!("ui_menu_fmap2: {CHART_TABLE_ENTRY} holds no usable chart table");
        return None;
    }
    if table.len() != usize::try_from(CHART_EXPECTED_COUNT).unwrap_or(0)
        || raw.len() != CHART_EXPECTED_BYTES
    {
        log::warn!(
            "ui_menu_fmap2: chart table is {} entries in {} bytes",
            table.len(),
            raw.len()
        );
    }
    Some(table)
}

pub fn load_sector_table(root: &AssetRoot) -> Option<SectorTable> {
    let raw = root.read(SECTOR_TABLE_FILE).ok()?;
    let raw = decode_asset(&raw, SECTOR_TABLE_FILE)?;
    let mut table = SectorTable::default();
    if !table.set_data(&raw) {
        log::warn!("ui_menu_fmap2: {SECTOR_TABLE_FILE} holds no usable sector table");
        return None;
    }
    if table.len() != SECTOR_EXPECTED_COUNT as usize || raw.len() != SECTOR_EXPECTED_BYTES {
        log::warn!(
            "ui_menu_fmap2: sector table is {} entries in {} bytes",
            table.len(),
            raw.len()
        );
    }
    Some(table)
}

pub fn read_screen(root: &AssetRoot, archive_rel: &str, entry: &str) -> Option<Screen> {
    let bytes = read_archive_entry(root, archive_rel, entry)?;
    crate::j2d::parse(&bytes)
}

pub fn texture_names(archive: &Archive) -> Vec<String> {
    let mut names: Vec<String> = archive
        .entries()
        .iter()
        .filter(|name| name.ends_with(".bti"))
        .map(|name| (*name).to_string())
        .collect();
    names.sort();
    names
}

pub fn read_map_texture_names(root: &AssetRoot) -> Vec<String> {
    match root.read_archive(MAP_ARCHIVE) {
        Ok(archive) => texture_names(&archive),
        Err(_) => Vec::new(),
    }
}

pub fn chart_texture_name(grid_no: i8, salvage: i32) -> String {
    format!("R{:02X}_col_{}.bti", grid_no as u8, salvage)
}

pub fn read_map_texture(root: &AssetRoot, name: &str) -> Option<Vec<u8>> {
    read_archive_entry(root, MAP_ARCHIVE, name)
}

pub fn lookup_texture(names: &[String], name: &str) -> Option<usize> {
    names
        .iter()
        .position(|entry| entry.eq_ignore_ascii_case(name))
}

pub fn point_to_grid(x: f32, z: f32) -> (i8, i8) {
    let gx = (x / GRID_UNIT + 0.5).floor().clamp(-3.0, 3.0) as i8;
    let gy = (z / GRID_UNIT + 0.5).floor().clamp(-3.0, 3.0) as i8;
    (gx, gy)
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
        4 => (v * 0x8000_u16 as f32 * 0.5).sin() * (v * 0x8000_u16 as f32 * 0.5).sin(),
        5 => (v * 0xFFFF_u16 as f32 * 0.5).sin() * (v * 0xFFFF_u16 as f32 * 0.5).sin(),
        _ => v,
    }
}

pub fn is_special_map_kind(kind: i32) -> bool {
    matches!(kind, 3 | 4 | 7 | 9..=16 | 17)
}

pub fn special_archive(kind: u8) -> Option<&'static str> {
    SPECIAL_ARCHIVES
        .iter()
        .find(|(id, _, _)| *id == kind)
        .map(|(_, archive, _)| *archive)
}

pub fn special_screen_entry(kind: u8) -> Option<&'static str> {
    SPECIAL_ARCHIVES
        .iter()
        .find(|(id, _, _)| *id == kind)
        .map(|(_, _, entry)| *entry)
}

pub fn special_tags(kind: u8) -> Option<&'static [&'static str]> {
    let tags: &'static [&'static str] = match kind {
        KIND_TINGLE_MAP => &TINGLE_TAGS,
        KIND_GHOST_SHIP => &GHOST_SHIP_TAGS,
        KIND_TREASURE_MAP => &TREASURE_TAGS,
        KIND_DOCTOR_MAP => &DOCTOR_TAGS,
        KIND_DARK_PATH_MAP => &DARK_PATH_TAGS,
        KIND_HEART_PIECE_MAP => &HEART_PIECE_TAGS,
        KIND_HEART_MAP => &HEART_TAGS,
        KIND_SUBDAN_MAP => &SUBDAN_TAGS,
        KIND_MOON_MAP => &MOON_TAGS,
        KIND_TOWER_MAP => &TOWER_TAGS,
        KIND_TERRY_MAP => &TERRY_TAGS,
        KIND_SUB_MARINE_MAP => &SUB_MARINE_TAGS,
        _ => return None,
    };
    Some(tags)
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shade {
    pub black: [u8; 4],
    pub white: [u8; 4],
}

#[derive(Clone, Debug)]
pub struct Pane {
    pub tag: String,
    pub kind: Kind,
    pub found: bool,
    pub visible: bool,
    pub pos_top_left_orig: Vec2,
    pub pos_top_left: Vec2,
    pub pos_center_orig: Vec2,
    pub pos_center: Vec2,
    pub size_orig: Vec2,
    pub size: Vec2,
    pub init_alpha: u8,
    pub now_alpha: u8,
    pub user_area: i16,
    pub texture: Option<String>,
    pub rotation: u16,
    pub rotate_origin: Vec2,
    pub rotate_angle: f32,
    pub shade: Shade,
    pub text: String,
    pub font_size: Vec2,
    pub line_space: f32,
}

impl Default for Pane {
    fn default() -> Self {
        Self {
            tag: String::new(),
            kind: Kind::Pane,
            found: false,
            visible: true,
            pos_top_left_orig: Vec2::default(),
            pos_top_left: Vec2::default(),
            pos_center_orig: Vec2::default(),
            pos_center: Vec2::default(),
            size_orig: Vec2::default(),
            size: Vec2::default(),
            init_alpha: 0xFF,
            now_alpha: 0,
            user_area: 0,
            texture: None,
            rotation: 0,
            rotate_origin: Vec2::default(),
            rotate_angle: 0.0,
            shade: Shade {
                black: SHADE_BLACK,
                white: [255, 255, 255, 255],
            },
            text: String::new(),
            font_size: Vec2::default(),
            line_space: 0.0,
        }
    }
}

impl Pane {
    pub fn new(tag: &str) -> Self {
        Self {
            tag: tag.to_string(),
            ..Default::default()
        }
    }

    pub fn parts_set(&mut self, layout: &ScreenPane) {
        self.kind = layout.kind;
        self.found = true;
        self.visible = layout.visible;
        self.pos_top_left_orig = Vec2 {
            x: layout.bounds.x,
            y: layout.bounds.y,
        };
        self.pos_top_left = self.pos_top_left_orig;
        self.size_orig = Vec2 {
            x: layout.bounds.w,
            y: layout.bounds.h,
        };
        self.size = self.size_orig;
        self.pos_center_orig = Vec2 {
            x: layout.bounds.x + layout.bounds.w / 2.0,
            y: layout.bounds.y + layout.bounds.h / 2.0,
        };
        self.pos_center = self.pos_center_orig;
        self.init_alpha = layout.alpha;
        self.now_alpha = 0;
        self.user_area = 0;
        self.rotation = layout.rotation;
        self.rotate_origin = Vec2 {
            x: self.size_orig.x * 0.5,
            y: self.size_orig.y * 0.5,
        };
        self.rotate_angle = rotation_to_deg(layout.rotation);
        self.texture = layout.texture.clone();
        self.text = layout.text.clone();
        self.font_size = Vec2 {
            x: layout.font_size_x as f32,
            y: layout.font_size_y as f32,
        };
        self.line_space = layout.line_space as f32;
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

    pub fn change_texture(&mut self, texture: &str) {
        self.texture = Some(texture.to_string());
    }

    pub fn set_string(&mut self, text: &str) {
        self.text.clear();
        self.text.push_str(text);
    }

    pub fn set_now_alpha(&mut self, alpha: f32) {
        self.now_alpha = (f32::from(self.init_alpha) * alpha) as u8;
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

    pub fn pane_scale_xy_child(&mut self, scale: f32) {
        self.size.x = self.size_orig.x * scale;
        self.size.y = self.size_orig.y * scale;
        self.pos_top_left.x = self.pos_top_left_orig.x * scale;
        self.pos_top_left.y = self.pos_top_left_orig.y * scale;
        self.pos_center.x = self.pos_top_left.x + self.size.x / 2.0;
        self.pos_center.y = self.pos_top_left.y + self.size.y / 2.0;
        if self.rotate_angle != 0.0 {
            self.rotate_origin.x = self.size.x * 0.5;
            self.rotate_origin.y = self.size.y * 0.5;
        }
    }

    pub fn pane_scale_xy(&mut self, scale: f32) {
        self.size.x = self.size_orig.x * scale;
        self.size.y = self.size_orig.y * scale;
        self.cpos_move();
    }

    pub fn rotate(&mut self, x: f32, y: f32, angle: f32) {
        self.rotate_origin = Vec2 { x, y };
        self.rotate_angle = angle;
        self.rotation = deg_to_rotation(angle);
    }

    pub fn scale_child(&mut self, scale: f32) {
        self.size.x = self.size_orig.x * scale;
        self.size.y = self.size_orig.y * scale;
        self.pos_top_left.x = self.pos_top_left_orig.x * scale;
        self.pos_top_left.y = self.pos_top_left_orig.y * scale;
        self.pos_center.x = self.pos_top_left.x + self.size.x / 2.0;
        self.pos_center.y = self.pos_top_left.y + self.size.y / 2.0;
        if self.rotation != 0 {
            self.rotate_origin = Vec2 {
                x: self.size.x * 0.5,
                y: self.size.y * 0.5,
            };
            self.rotate_angle = rotation_to_deg(self.rotation);
        }
    }

    pub fn set_shade(&mut self, black: [u8; 4], white: [u8; 4]) {
        self.shade = Shade { black, white };
    }
}

pub fn rotation_to_deg(rotation: u16) -> f32 {
    f32::from(rotation) / 32768.0 * 180.0
}

pub fn deg_to_rotation(angle: f32) -> u16 {
    (angle * 32768.0 / 180.0) as i16 as u16
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Unknown,
    Running,
    Done,
}

impl Default for Step {
    fn default() -> Self {
        Step::Unknown
    }
}

pub fn steps_done(steps: &[Step]) -> bool {
    steps.iter().all(|step| *step == Step::Done)
}

pub trait MsgSource {
    fn message(&self, msg_no: u32) -> String;
}

#[derive(Resource, Clone, Debug, Default)]
pub struct ChartText {
    pub table: Vec<(u32, String)>,
}

impl ChartText {
    pub fn lookup(&self, msg_no: u32) -> String {
        self.table
            .iter()
            .find(|(id, _)| *id == msg_no)
            .map(|(_, text)| text.clone())
            .unwrap_or_default()
    }
}

impl MsgSource for ChartText {
    fn message(&self, msg_no: u32) -> String {
        self.lookup(msg_no)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ChartTuning {
    pub pane_frame: u8,
    pub pane_slide: i16,
    pub scroll_frame: u8,
    pub open_frame: u8,
    pub close_frame: u8,
    pub open_speed: f32,
    pub scroll_dx: f32,
    pub scroll_dy: f32,
    pub scroll_ratio: f32,
    pub reflow_frame: u8,
    pub settle_dx: f32,
    pub settle_dy: f32,
    pub settle_ratio: f32,
    pub settle_frame: u8,
    pub settle_limit: u8,
    pub cmyu_area: u8,
    pub moon_dx: u8,
    pub moon_dy: u8,
    pub moon_speed: u8,
    pub moon_from: u8,
    pub moon_to: u8,
    pub icon_dx: u8,
    pub icon_dy: u8,
    pub icon_speed: u8,
    pub tri_dx: u8,
    pub tri_speed: u8,
    pub tune_05: u8,
    pub tune_06: u8,
    pub tune_07: u8,
    pub tune_08: u8,
    pub tune_09: u8,
    pub tune_0a: u8,
    pub tune_0b: u8,
    pub tune_0c: u8,
    pub tune_0d: u8,
    pub tune_0e: u8,
    pub tune_0f: u8,
    pub tune_10: u8,
    pub tune_11: u8,
    pub tune_12: u8,
    pub tune_13: u8,
    pub tune_14: u8,
    pub tune_16: u16,
    pub tune_1e: u8,
    pub tune_1f: u8,
    pub tune_20: u8,
    pub tune_22: u8,
    pub tune_24: f32,
    pub tune_28: u8,
    pub tune_30: u8,
    pub tune_31: u8,
    pub tune_38: u8,
    pub tune_3a: i16,
    pub tune_3c: u16,
    pub tune_44: f32,
    pub tune_48: u8,
    pub tune_49: u8,
    pub tune_4b: u8,
    pub tune_4c: u8,
    pub tune_4d: u8,
    pub tune_4e: u8,
    pub tune_50: f32,
    pub tune_54: f32,
    pub tune_7c: u8,
    pub tune_7f: u8,
    pub tune_80: u8,
    pub tune_81: u8,
    pub tune_82: u8,
    pub tune_83: u8,
    pub tune_86: u8,
    pub tune_87: u8,
    pub tune_88: u8,
    pub tune_89: u8,
    pub tune_8a: u8,
    pub tune_8b: u8,
    pub tune_8c: u8,
    pub tune_8d: u8,
    pub tune_90: u8,
    pub tune_91: u8,
    pub tune_93: u8,
    pub tune_94: u8,
}

impl ChartTuning {
    pub fn retail() -> Self {
        Self {
            pane_frame: TUNE_PANE_FRAME,
            pane_slide: TUNE_PANE_SLIDE,
            scroll_frame: TUNE_SCROLL_FRAME,
            open_frame: TUNE_OPEN_FRAME,
            close_frame: TUNE_CLOSE_FRAME,
            open_speed: TUNE_OPEN_SPEED,
            scroll_dx: TUNE_SCROLL_DX,
            scroll_dy: TUNE_SCROLL_DY,
            scroll_ratio: TUNE_SCROLL_RATIO,
            reflow_frame: TUNE_REFLOW_FRAME,
            settle_dx: TUNE_SETTLE_DX,
            settle_dy: TUNE_SETTLE_DY,
            settle_ratio: TUNE_SETTLE_RATIO,
            settle_frame: TUNE_SETTLE_FRAME,
            settle_limit: TUNE_SETTLE_LIMIT,
            cmyu_area: TUNE_CMYU_AREA,
            moon_dx: TUNE_MOON_DX,
            moon_dy: TUNE_MOON_DY,
            moon_speed: TUNE_MOON_SPEED,
            moon_from: TUNE_MOON_FROM,
            moon_to: TUNE_MOON_TO,
            icon_dx: TUNE_ICON_DX,
            icon_dy: TUNE_ICON_DY,
            icon_speed: TUNE_ICON_SPEED,
            tri_dx: TUNE_TRI_DX,
            tri_speed: TUNE_TRI_SPEED,
            tune_05: 255,
            tune_06: 255,
            tune_07: 0,
            tune_08: 255,
            tune_09: 255,
            tune_0a: 0,
            tune_0b: 0,
            tune_0c: 255,
            tune_0d: 255,
            tune_0e: 255,
            tune_0f: 130,
            tune_10: 155,
            tune_11: 255,
            tune_12: 0x8E,
            tune_13: 0,
            tune_14: 255,
            tune_16: 25,
            tune_1e: 0,
            tune_1f: 0,
            tune_20: 0,
            tune_22: 50,
            tune_24: 5.0,
            tune_28: 30,
            tune_30: 5,
            tune_31: 11,
            tune_38: 8,
            tune_3a: 0,
            tune_3c: 0,
            tune_44: 0.05,
            tune_48: 8,
            tune_49: 8,
            tune_4b: 12,
            tune_4c: 0,
            tune_4d: 0,
            tune_4e: 0,
            tune_50: 30.0,
            tune_54: 30.0,
            tune_7c: 255,
            tune_7f: 255,
            tune_80: 0,
            tune_81: 0,
            tune_82: 168,
            tune_83: 0,
            tune_86: 255,
            tune_87: 255,
            tune_88: 255,
            tune_89: 255,
            tune_8a: 0,
            tune_8b: 0,
            tune_8c: 0,
            tune_8d: 0,
            tune_90: 255,
            tune_91: 32,
            tune_93: 255,
            tune_94: 32,
        }
    }
}

impl Default for ChartTuning {
    fn default() -> Self {
        Self::retail()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ChartStick {
    pub max_value: f32,
    pub min_value: f32,
    pub bits: u8,
    pub prev_bits: u8,
    pub horizontal_wait: i16,
    pub vertical_wait: i16,
    pub horizontal_step: i16,
    pub horizontal_min: i16,
    pub horizontal_left: i16,
    pub vertical_step: i16,
    pub vertical_min: i16,
    pub vertical_left: i16,
    pub repeat_angle: i16,
    pub angle_base: i16,
    pub half_dead: i16,
    pub repeat: i16,
    pub vertical_repeat: i16,
}

impl Default for ChartStick {
    fn default() -> Self {
        Self::new(5, 2, 3, 2)
    }
}

impl ChartStick {
    pub fn new(first: i16, second: i16, step: i16, min_step: i16) -> Self {
        let mut stick = Self {
            max_value: STICK_VALUE_ON,
            min_value: STICK_VALUE_OFF,
            bits: 0,
            prev_bits: 0,
            horizontal_wait: 0,
            vertical_wait: 0,
            horizontal_step: step,
            horizontal_min: min_step,
            horizontal_left: 0,
            vertical_step: step,
            vertical_min: min_step,
            vertical_left: 0,
            repeat_angle: 0,
            angle_base: 0,
            half_dead: STICK_HALF_DEAD,
            repeat: 0,
            vertical_repeat: 0,
        };
        stick.set_wait_parm(
            first,
            second,
            step,
            min_step,
            STICK_VALUE_ON,
            STICK_VALUE_OFF,
            0,
            2048,
        );
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
        self.horizontal_wait = first;
        self.vertical_wait = first;
        self.horizontal_min = second;
        self.vertical_min = second;
        self.horizontal_step = step;
        self.vertical_step = step;
        self.max_value = max_value;
        self.min_value = min_value;
        self.repeat_angle = repeat_angle;
        self.half_dead = half_dead;
    }

    pub fn init(&mut self) {
        self.repeat = 0;
        self.vertical_repeat = 0;
        self.bits = 0;
        self.prev_bits = 0;
        self.horizontal_left = self.horizontal_min;
        self.vertical_left = self.vertical_min;
    }

    pub fn x_init(&mut self) {
        self.repeat = 0;
        self.bits &= !0x03;
        self.horizontal_wait = self.horizontal_min;
        self.horizontal_left = self.horizontal_min;
    }

    pub fn y_init(&mut self) {
        self.vertical_repeat = 0;
        self.bits &= !0x0C;
        self.vertical_wait = self.vertical_min;
        self.vertical_left = self.vertical_min;
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
                bits |= 4 | 1;
            } else if angle < base - 0x3000 + half {
                bits |= 1;
            } else if angle < base - 0x1000 - half {
                bits |= 1 | 8;
            } else if angle < base + 0x1000 + half {
                bits |= 8;
            } else if angle < base + 0x3000 - half {
                bits |= 8 | 2;
            } else if angle < base + 0x5000 + half {
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
        if self.prev_bits & self.bits & 0x03 != 0 && self.repeat > 0 {
            self.repeat -= 1;
        }
        if self.prev_bits & self.bits & 0x0C != 0 && self.vertical_repeat > 0 {
            self.vertical_repeat -= 1;
        }
        self.bits
    }

    pub fn check_left_trigger(&mut self) -> bool {
        self.horizontal_trigger(1)
    }

    pub fn check_right_trigger(&mut self) -> bool {
        self.horizontal_trigger(2)
    }

    pub fn check_up_trigger(&mut self) -> bool {
        self.vertical_trigger(4)
    }

    pub fn check_down_trigger(&mut self) -> bool {
        self.vertical_trigger(8)
    }

    fn horizontal_trigger(&mut self, bit: u8) -> bool {
        if self.repeat != 0 || self.bits & bit == 0 {
            return false;
        }
        self.repeat = self.horizontal_wait;
        if self.horizontal_left == 0 {
            self.horizontal_wait -= self.horizontal_step;
            if self.horizontal_wait < self.horizontal_min {
                self.horizontal_wait = self.horizontal_min;
            }
        } else {
            self.horizontal_left -= 1;
        }
        true
    }

    fn vertical_trigger(&mut self, bit: u8) -> bool {
        if self.vertical_repeat != 0 || self.bits & bit == 0 {
            return false;
        }
        self.vertical_repeat = self.vertical_wait;
        if self.vertical_left == 0 {
            self.vertical_wait -= self.vertical_step;
            if self.vertical_wait < self.vertical_min {
                self.vertical_wait = self.vertical_min;
            }
        } else {
            self.vertical_left -= 1;
        }
        true
    }
}

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct ChartPadInput {
    pub trig_a: bool,
    pub trig_b: bool,
    pub lock_l: bool,
    pub lock_r: bool,
    pub stick_value: f32,
    pub stick_angle: i16,
    pub opt_ruby: bool,
}

impl ChartPadInput {
    pub fn pad(&self) -> ChartPad {
        ChartPad {
            trig_a: self.trig_a,
            trig_b: self.trig_b,
            lock_l: self.lock_l,
            lock_r: self.lock_r,
            stick_value: self.stick_value,
            stick_angle: self.stick_angle,
            opt_ruby: self.opt_ruby,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ChartPad {
    pub trig_a: bool,
    pub trig_b: bool,
    pub lock_l: bool,
    pub lock_r: bool,
    pub stick_value: f32,
    pub stick_angle: i16,
    pub opt_ruby: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WorldOrigin {
    pub player_x: f32,
    pub player_z: f32,
    pub player_angle: i16,
    pub ship_room_id: u8,
    pub ocean: Option<[i8; 2]>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WorldFacts {
    pub time: f32,
    pub moon: i32,
    pub ghost_ship: u8,
    pub salvage_point: u8,
    pub arrived: u128,
}

impl WorldFacts {
    pub fn has_arrived(&self, sector: usize) -> bool {
        if sector >= SECTOR_TOTAL {
            return false;
        }
        self.arrived & (1_u128 << sector) != 0
    }
}

pub fn world_facts(
    day: &DayNight,
    regs: &EventReg,
    arrivals: &Arrivals,
    control: &salvage::Control,
) -> WorldFacts {
    let mut arrived = 0_u128;
    for (sector, byte) in arrivals.bytes().iter().enumerate() {
        if byte & 1 != 0 && sector < SECTOR_TOTAL {
            arrived |= 1_u128 << sector;
        }
    }
    WorldFacts {
        time: day.time,
        moon: day.moon_type(),
        ghost_ship: regs.reg(REG_GHOST_SHIP),
        salvage_point: control.random_salvage_point,
        arrived,
    }
}

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SharedCursor {
    pub active: u8,
    pub disp_mode: u8,
    pub cur_x: i8,
    pub cur_y: i8,
    pub chart_sel: i8,
    pub cur_hx: i8,
    pub cur_hy: i8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ListItem {
    pub screen: u8,
}

pub const LIST_WORLD: u8 = 0;
pub const LIST_SPECIAL: u8 = 1;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaneDraw {
    pub tag: String,
    pub visible: bool,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub alpha: u8,
    pub rotate: f32,
    pub texture: Option<String>,
    pub text: String,
}

#[derive(Clone, Debug, Default)]
pub struct ChartDraw {
    pub proc_index: u8,
    pub active: u8,
    pub button_mode: Option<i32>,
    pub confirm_locked: bool,
    pub world_cursor: [i8; 2],
    pub chart_cursor: i32,
    pub chart_previous: i32,
    pub scroll_dir: i8,
    pub chart_total: usize,
    pub chart_open: usize,
    pub chart_done: usize,
    pub anim_tick: i16,
    pub scroll: [f32; 3],
    pub hold: [u8; 2],
    pub player_grid: [i8; 2],
    pub player_pos: [f32; 2],
    pub player_heading: f32,
    pub player_valid: bool,
    pub world_texture: Option<usize>,
    pub chart_texture: [Option<usize>; 2],
    pub rupee_texture: [Option<String>; 4],
    pub name_page: u8,
    pub name_origin: [Vec2; 2],
    pub page_front: u8,
    pub marker_page: u8,
    pub zoom_page: u8,
    pub shade: [Option<[u8; 4]>; 2],
    pub special_kind: u8,
    pub special_archive: Option<&'static str>,
    pub special_open: bool,
    pub special_pending: bool,
    pub pane_cursor: Option<usize>,
    pub sounds: Vec<u32>,
    pub list: Vec<ListItem>,
    pub panes: Vec<PaneDraw>,
}

#[derive(Clone, Debug)]
pub struct SpecialScreen {
    pub kind: u8,
    pub entry: &'static str,
    pub link: usize,
    pub target: usize,
    pub guide: usize,
    pub moon_shade: Shade,
    pub key_shade: Shade,
    pub panes: Vec<Pane>,
}

#[derive(Resource, Clone, Debug)]
pub struct ChartLayer {
    pub tune: ChartTuning,
    pub table: ChartTable,
    pub sectors: SectorTable,
    pub stick: ChartStick,
    pub world_textures: Vec<String>,
    pub chart_textures: Vec<String>,
    pub screen: Option<Screen>,
    pub screen_loaded: bool,
    pub special: Option<SpecialScreen>,
    pub special_pending: bool,
    pub special_open: bool,
    pub proc_index: u8,
    pub button_mode: u8,
    pub lock_confirm: u8,
    pub chart_total: u8,
    pub chart_cursor: i8,
    pub chart_prev: i8,
    pub scroll_dir: i8,
    pub anim_tick: i16,
    pub salvage_point: u8,
    pub player_x: f32,
    pub player_z: f32,
    pub player_heading: f32,
    pub player_grid_x: i8,
    pub player_grid_y: i8,
    pub player_valid: u8,
    pub ghost_ship_done: u8,
    pub moon_on: u8,
    pub moon_x: u8,
    pub moon_y: u8,
    pub moon_step: u8,
    pub moon_from: u8,
    pub moon_to: u8,
    pub icon_x: u8,
    pub icon_y: u8,
    pub icon_step: u8,
    pub tri_x: u8,
    pub tri_step: u8,
    pub name_msg: [u16; 2],
    pub name_origin: [Vec2; 2],
    pub font_size_a: Vec2,
    pub font_size_b: Vec2,
    pub line_space_a: f32,
    pub line_space_b: f32,
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub scroll_ratio: f32,
    pub scroll_frame: u8,
    pub hold_left: u8,
    pub hold_right: u8,
    pub special_kind: u8,
    pub pane_front: u8,
    pub pane_marker: u8,
    pub pane_zoom: u8,
    pub pane_name: u8,
    pub shade_black: Option<[u8; 4]>,
    pub shade_white: Option<[u8; 4]>,
    pub pane_cursor: Option<usize>,
    pub world_texture: Option<usize>,
    pub chart_texture: [Option<usize>; 2],
    pub rupee_texture: [Option<String>; 4],
    pub stale_done: Vec<usize>,
    pub sounds: Vec<u32>,
    pub cl_pane: Pane,
    pub kdm_pane: Pane,
    pub fc_panes: [Pane; FC_PANES],
    pub cc_panes: [Pane; CC_PANES],
    pub ms_panes: [Pane; MS_PANES],
    pub name_panes_a: [Pane; MSG_NAME_PANES],
    pub name_panes_b: [Pane; MSG_NAME_PANES],
    pub island_name_panes: [Pane; ISLAND_NAME_PANES],
    pub key_panes: [Pane; KEY_PANES],
    pub extra_panes: [Pane; EXTRA_PANES],
    pub sector_panes: [Pane; SECTOR_TOTAL],
    pub chart_page_a: [Pane; CHART_PAGE_PANES],
    pub chart_page_b: [Pane; CHART_PAGE_PANES],
    pub zoom_page_a: [Pane; ZOOM_PAGE_PANES],
    pub zoom_page_b: [Pane; ZOOM_PAGE_PANES],
    pub marker_page_a: [Pane; MARKER_PAGE_PANES],
    pub marker_page_b: [Pane; MARKER_PAGE_PANES],
    pub cmyu_pane: Pane,
    pub cmyd_pane: Pane,
    pub cnd0_pane: Pane,
    pub nfkd_pane: Pane,
    pub mkfd_pane: Pane,
    pub mkcd_pane: Pane,
    pub ncdd_pane: Pane,
    pub mkm0_pane: Pane,
    pub lnk3_pane: Pane,
    pub r01g_pane: Pane,
    pub area_pane: Pane,
}

impl Default for ChartLayer {
    fn default() -> Self {
        Self {
            tune: ChartTuning::retail(),
            table: ChartTable::default(),
            sectors: SectorTable::default(),
            stick: ChartStick::default(),
            world_textures: Vec::new(),
            chart_textures: Vec::new(),
            screen: None,
            screen_loaded: false,
            special: None,
            special_pending: false,
            special_open: false,
            proc_index: PROC_FMAP,
            button_mode: 0,
            lock_confirm: 0,
            chart_total: 0,
            chart_cursor: 0,
            chart_prev: 0,
            scroll_dir: 0,
            anim_tick: 0,
            salvage_point: 0,
            player_x: 0.0,
            player_z: 0.0,
            player_heading: 0.0,
            player_grid_x: 0,
            player_grid_y: 0,
            player_valid: 0,
            ghost_ship_done: 0,
            moon_on: 0,
            moon_x: 0,
            moon_y: 0,
            moon_step: 1,
            moon_from: 0,
            moon_to: 0,
            icon_x: 0,
            icon_y: 0,
            icon_step: 1,
            tri_x: 0,
            tri_step: 1,
            name_msg: [NO_MESSAGE; 2],
            name_origin: [Vec2::default(); 2],
            font_size_a: Vec2::default(),
            font_size_b: Vec2::default(),
            line_space_a: 0.0,
            line_space_b: 0.0,
            scroll_x: 0.0,
            scroll_y: 0.0,
            scroll_ratio: 0.0,
            scroll_frame: 0,
            hold_left: 0,
            hold_right: 0,
            special_kind: 0,
            pane_front: 0,
            pane_marker: 0,
            pane_zoom: 0,
            pane_name: 0,
            shade_black: None,
            shade_white: None,
            pane_cursor: None,
            world_texture: None,
            chart_texture: [None; 2],
            rupee_texture: [None, None, None, None],
            stale_done: Vec::new(),
            sounds: Vec::new(),
            cl_pane: Pane::new("cl"),
            kdm_pane: Pane::new("Kdm"),
            fc_panes: std::array::from_fn(|i| Pane::new(FC_TAGS[i])),
            cc_panes: std::array::from_fn(|i| Pane::new(CC_TAGS[i])),
            ms_panes: std::array::from_fn(|i| Pane::new(MS_TAGS[i])),
            name_panes_a: std::array::from_fn(|i| Pane::new(NAME_TAGS_A[i])),
            name_panes_b: std::array::from_fn(|i| Pane::new(NAME_TAGS_B[i])),
            island_name_panes: std::array::from_fn(|i| Pane::new(ISLAND_NAME_TAGS[i])),
            key_panes: std::array::from_fn(|i| Pane::new(KEY_TAGS[i])),
            extra_panes: std::array::from_fn(|i| Pane::new(EXTRA_TAGS[i])),
            sector_panes: std::array::from_fn(|i| Pane::new(&sector_mark_tag(i))),
            chart_page_a: std::array::from_fn(|i| Pane::new(CHART_PAGE_TAGS_A[i])),
            chart_page_b: std::array::from_fn(|i| Pane::new(CHART_PAGE_TAGS_B[i])),
            zoom_page_a: std::array::from_fn(|i| Pane::new(ZOOM_PAGE_TAGS_A[i])),
            zoom_page_b: std::array::from_fn(|i| Pane::new(ZOOM_PAGE_TAGS_B[i])),
            marker_page_a: std::array::from_fn(|i| Pane::new(MARKER_TAGS_A[i])),
            marker_page_b: std::array::from_fn(|i| Pane::new(MARKER_TAGS_B[i])),
            cmyu_pane: Pane::new("cmyu"),
            cmyd_pane: Pane::new("cmyd"),
            cnd0_pane: Pane::new("cnd0"),
            nfkd_pane: Pane::new("nfkd"),
            mkfd_pane: Pane::new("mkFD"),
            mkcd_pane: Pane::new("mkCD"),
            ncdd_pane: Pane::new("ncdd"),
            mkm0_pane: Pane::new("mkm0"),
            lnk3_pane: Pane::new("lnk3"),
            r01g_pane: Pane::new("R01G"),
            area_pane: Pane::new("area"),
        }
    }
}

impl ChartLayer {
    pub fn create(&mut self, root: &AssetRoot) -> bool {
        let Some(screen) = read_screen(root, CHART_TABLE_ARCHIVE, MAIN_SCREEN_ENTRY) else {
            log::warn!("ui_menu_fmap2: {MAIN_SCREEN_ENTRY} unreadable");
            return false;
        };
        self.screen = Some(screen);
        self.screen_loaded = true;
        self.stick = ChartStick::new(5, 2, 3, 2);
        self.special_open = false;
        self.screen_set();
        true
    }

    pub fn delete(&mut self) {
        self.screen = None;
        self.screen_loaded = false;
        self.special = None;
        self.special_pending = false;
        self.special_open = false;
        self.stick = ChartStick::default();
        self.reset_panes();
    }

    pub fn bind_tables(&mut self, table: ChartTable, sectors: SectorTable) {
        self.table = table;
        self.sectors = sectors;
    }

    pub fn bind_textures(&mut self, world: Vec<String>, chart: Vec<String>) {
        self.world_textures = world;
        self.chart_textures = chart;
    }

    pub fn set_special_pending(&mut self, pending: bool) {
        self.special_pending = pending;
    }

    pub fn take_stale_done(&mut self) -> Vec<usize> {
        core::mem::take(&mut self.stale_done)
    }

    fn reset_panes(&mut self) {
        self.cl_pane = Pane::new("cl");
        self.kdm_pane = Pane::new("Kdm");
        self.fc_panes = std::array::from_fn(|i| Pane::new(FC_TAGS[i]));
        self.cc_panes = std::array::from_fn(|i| Pane::new(CC_TAGS[i]));
        self.ms_panes = std::array::from_fn(|i| Pane::new(MS_TAGS[i]));
        self.name_panes_a = std::array::from_fn(|i| Pane::new(NAME_TAGS_A[i]));
        self.name_panes_b = std::array::from_fn(|i| Pane::new(NAME_TAGS_B[i]));
        self.island_name_panes = std::array::from_fn(|i| Pane::new(ISLAND_NAME_TAGS[i]));
        self.key_panes = std::array::from_fn(|i| Pane::new(KEY_TAGS[i]));
        self.extra_panes = std::array::from_fn(|i| Pane::new(EXTRA_TAGS[i]));
        self.sector_panes = std::array::from_fn(|i| Pane::new(&sector_mark_tag(i)));
        self.chart_page_a = std::array::from_fn(|i| Pane::new(CHART_PAGE_TAGS_A[i]));
        self.chart_page_b = std::array::from_fn(|i| Pane::new(CHART_PAGE_TAGS_B[i]));
        self.zoom_page_a = std::array::from_fn(|i| Pane::new(ZOOM_PAGE_TAGS_A[i]));
        self.zoom_page_b = std::array::from_fn(|i| Pane::new(ZOOM_PAGE_TAGS_B[i]));
        self.marker_page_a = std::array::from_fn(|i| Pane::new(MARKER_TAGS_A[i]));
        self.marker_page_b = std::array::from_fn(|i| Pane::new(MARKER_TAGS_B[i]));
        self.cmyu_pane = Pane::new("cmyu");
        self.cmyd_pane = Pane::new("cmyd");
        self.cnd0_pane = Pane::new("cnd0");
        self.nfkd_pane = Pane::new("nfkd");
        self.mkfd_pane = Pane::new("mkFD");
        self.mkcd_pane = Pane::new("mkCD");
        self.ncdd_pane = Pane::new("ncdd");
        self.mkm0_pane = Pane::new("mkm0");
        self.lnk3_pane = Pane::new("lnk3");
        self.r01g_pane = Pane::new("R01G");
        self.area_pane = Pane::new("area");
    }

    fn bind(screen: &Screen, pane: &mut Pane) {
        if let Some(layout) = screen.find(&pane.tag).cloned() {
            pane.parts_set(&layout);
        }
    }

    fn bind_all<const N: usize>(screen: &Screen, panes: &mut [Pane; N]) {
        for pane in panes.iter_mut() {
            Self::bind(screen, pane);
        }
    }

    pub fn screen_set(&mut self) {
        let Some(screen) = self.screen.clone() else {
            return;
        };
        Self::bind(&screen, &mut self.cl_pane);
        Self::bind(&screen, &mut self.kdm_pane);
        Self::bind_all(&screen, &mut self.fc_panes);
        Self::bind_all(&screen, &mut self.cc_panes);
        Self::bind_all(&screen, &mut self.ms_panes);
        Self::bind_all(&screen, &mut self.name_panes_a);
        Self::bind_all(&screen, &mut self.name_panes_b);
        Self::bind_all(&screen, &mut self.island_name_panes);
        Self::bind_all(&screen, &mut self.key_panes);
        Self::bind_all(&screen, &mut self.extra_panes);
        Self::bind_all(&screen, &mut self.sector_panes);
        Self::bind_all(&screen, &mut self.chart_page_a);
        Self::bind_all(&screen, &mut self.chart_page_b);
        Self::bind_all(&screen, &mut self.zoom_page_a);
        Self::bind_all(&screen, &mut self.zoom_page_b);
        Self::bind_all(&screen, &mut self.marker_page_a);
        Self::bind_all(&screen, &mut self.marker_page_b);
        Self::bind(&screen, &mut self.cmyu_pane);
        Self::bind(&screen, &mut self.cmyd_pane);
        Self::bind(&screen, &mut self.cnd0_pane);
        Self::bind(&screen, &mut self.nfkd_pane);
        Self::bind(&screen, &mut self.mkfd_pane);
        Self::bind(&screen, &mut self.mkcd_pane);
        Self::bind(&screen, &mut self.ncdd_pane);
        Self::bind(&screen, &mut self.mkm0_pane);
        Self::bind(&screen, &mut self.lnk3_pane);
        Self::bind(&screen, &mut self.r01g_pane);
        Self::bind(&screen, &mut self.area_pane);
        self.font_size_a = self.name_panes_a[1].font_size;
        self.font_size_b = self.name_panes_a[3].font_size;
        self.line_space_a = self.name_panes_a[1].line_space;
        self.line_space_b = self.name_panes_a[3].line_space;
        let fill_long = "A".repeat(TEXT_FILL_LONG - 1);
        let fill_short = "A".repeat(TEXT_FILL_SHORT - 1);
        for index in 1..MS_PANES {
            self.ms_panes[index].set_string(&fill_long);
        }
        for page in 0..PAGE_COUNT as u8 {
            for index in 1..MS_PANES {
                self.name_page_mut(page, index).set_string("");
            }
        }
        for index in 0..4 {
            self.island_name_panes[index].set_string(&fill_short);
        }
        for index in 0..KEY_PANES {
            self.key_panes[index].set_string(&fill_short);
        }
        self.pane_name = 0;
        self.pane_marker = 0;
        self.pane_front = 0;
        self.pane_zoom = 0;
        for index in [0usize, 1, 2] {
            self.chart_page_a[index].user_area = i16::from(self.chart_page_a[index].init_alpha);
            self.chart_page_b[index].user_area = i16::from(self.chart_page_b[index].init_alpha);
        }
        for index in [0usize, 1, 2, 9] {
            self.zoom_page_a[index].user_area = i16::from(self.zoom_page_a[index].init_alpha);
            self.zoom_page_b[index].user_area = i16::from(self.zoom_page_b[index].init_alpha);
        }
        self.shade_black = Some(self.area_pane.shade.black);
        self.shade_white = Some(self.area_pane.shade.white);
    }

    pub fn initialize(
        &mut self,
        shared: &mut SharedCursor,
        start_stage: &str,
        origin: &WorldOrigin,
        charts: &mut Charts,
    ) {
        self.anim_tick = 0;
        self.hold_left = 0;
        self.hold_right = 0;
        self.name_msg = [NO_MESSAGE; 2];
        self.lock_confirm = 0;
        self.player_valid = 1;
        self.player_x = origin.player_x;
        self.player_z = origin.player_z;
        let angle = i16::wrapping_add(origin.player_angle, -0x8000) as f32;
        self.player_heading = angle * 180.0 / 32768.0;
        if start_stage == "sea" {
            let (gx, gy) = point_to_grid(self.player_x, self.player_z);
            self.player_grid_x = gx;
            self.player_grid_y = gy;
        } else {
            self.player_valid = 0;
            if matches!(start_stage, "Obshop" | "Abship" | "PShip") {
                self.player_grid_x = (origin.ship_room_id % SHIP_GRID_MODULO) as i8 - SECTOR_BIAS;
                self.player_grid_y = (origin.ship_room_id / SHIP_GRID_MODULO) as i8 - SECTOR_BIAS;
            } else {
                match origin.ocean {
                    Some([ox, oz]) => {
                        self.player_grid_x = if ox == 4 { 0 } else { ox };
                        self.player_grid_y = if oz == 4 { 0 } else { oz };
                    }
                    None => {
                        self.player_grid_x = 0;
                        self.player_grid_y = 0;
                    }
                }
            }
        }
        if shared.cur_hx < -SECTOR_BIAS || shared.cur_hy < -SECTOR_BIAS {
            shared.cur_hx = self.player_grid_x;
            shared.cur_hy = self.player_grid_y;
        } else if shared.disp_mode == 0 {
            shared.cur_hx = shared.cur_x;
            shared.cur_hy = shared.cur_y;
        }
        self.cmyu_pane.user_area = i16::from(self.tune.cmyu_area) + 1;
        self.chart_total = self.calc_get_chart(charts) as u8;
        if shared.chart_sel == -1 {
            self.chart_cursor = self.first_chart(charts) as i8;
            shared.chart_sel = self.chart_cursor;
        } else {
            self.chart_cursor = shared.chart_sel;
        }
    }

    pub fn display_init(
        &mut self,
        shared: &mut SharedCursor,
        world: &WorldFacts,
        pad: &ChartPad,
        charts: &Charts,
        texts: &dyn MsgSource,
    ) {
        self.salvage_point = world.salvage_point;
        for sector in 0..SECTOR_TOTAL {
            let visible = !world.has_arrived(sector);
            if visible {
                self.sector_panes[sector].show();
            } else {
                self.sector_panes[sector].hide();
            }
        }
        self.area_pane.pane_trans(
            f32::from(self.player_grid_x) * AREA_GRID_STEP,
            f32::from(self.player_grid_y) * AREA_GRID_STEP,
        );
        self.collect_map_tex_change(charts);
        self.fin_collect_map_tex_change(charts);
        let name = self.pane_name;
        self.name_page_mut(name, 0).hide();
        self.name_page_mut(name ^ 1, 0).hide();
        if pad.opt_ruby {
            for index in 3..MS_PANES {
                self.ms_panes[index].hide();
                self.name_page_mut(0, index).hide();
                self.name_page_mut(1, index).hide();
            }
            for index in 1..3 {
                self.shift_ruby(index);
            }
        }
        self.collect_map_check(charts, texts);
        match shared.active {
            0 => {
                self.field_pane_init();
                self.stick
                    .set_wait_parm(5, 2, 3, 2, STICK_VALUE_ON, STICK_VALUE_OFF, 0, 2048);
                self.proc_index = PROC_FMAP;
            }
            1 => {
                self.chart_pane_init();
                self.stick
                    .set_wait_parm(0, 0, 0, 0, STICK_VALUE_ON, STICK_VALUE_OFF, 0, 2048);
                self.proc_index = PROC_CMAP;
            }
            _ => {}
        }
        self.pane_trans_base(
            shared,
            self.anim_tick,
            self.tune.pane_frame,
            f32::from(self.tune.pane_slide),
            0.0,
            0,
            self.pane_zoom,
            0,
        );
    }

    fn shift_ruby(&mut self, index: usize) {
        for page in 0..PAGE_COUNT as u8 {
            let pane = self.name_page_mut(page, index);
            pane.pos_center_orig.y -= RUBY_SHIFT;
            pane.pos_center.y = pane.pos_center_orig.y;
            pane.cpos_move();
            pane.pos_top_left_orig.y = pane.pos_top_left.y;
        }
        let pane = &mut self.ms_panes[index];
        pane.pos_center_orig.y -= RUBY_SHIFT;
        pane.pos_center.y = pane.pos_center_orig.y;
        pane.cpos_move();
        pane.pos_top_left_orig.y = pane.pos_top_left.y;
    }

    pub fn field_pane_init(&mut self) {
        self.mkcd_pane.show();
        self.mkfd_pane.hide();
        if self.ms_panes[0].is_visible() {
            self.pane_alpha_message2(self.anim_tick, self.tune.scroll_frame, 0, 0);
        }
        let zoom = self.pane_zoom;
        if self.zoom_page(zoom)[0].is_visible() {
            self.mkm0_pane.now_alpha = MKM0_FMAP_ALPHA;
            self.mkm0_pane.init_alpha = MKM0_FMAP_ALPHA;
        } else {
            self.mkm0_pane.now_alpha = MKM0_FMAP_DIM;
            self.mkm0_pane.init_alpha = MKM0_FMAP_DIM;
        }
        self.cnd0_pane.now_alpha = CND0_DIM;
        self.cnd0_pane.init_alpha = CND0_DIM;
    }

    pub fn chart_pane_init(&mut self) {
        self.mkfd_pane.show();
        self.mkcd_pane.hide();
        self.cnd0_pane.now_alpha = CND0_CMAP_ALPHA;
        self.cnd0_pane.init_alpha = CND0_CMAP_ALPHA;
    }

    pub fn collect_map_check(&mut self, charts: &Charts, texts: &dyn MsgSource) {
        if self.chart_total == 0 {
            self.ms_panes[0].show();
            self.cnd0_pane.hide();
            for pane in self.island_name_panes.iter_mut() {
                pane.hide();
            }
            let front = self.pane_front;
            for index in 0..CHART_PAGE_PANES {
                self.chart_page_mut(front, index).hide();
            }
            let marker = self.pane_marker;
            for index in 0..MARKER_PAGE_PANES {
                self.marker_page_mut(marker, index).hide();
            }
            let zoom = self.pane_zoom;
            self.zoom_page_mut(zoom, 0).hide();
        } else {
            let back_zoom = self.pane_zoom ^ 1;
            let back_front = self.pane_front ^ 1;
            let back_marker = self.pane_marker ^ 1;
            let back_name = self.pane_name ^ 1;
            self.zoom_page_mut(back_zoom, 0).hide();
            self.name_page_mut(back_name, 0).hide();
            for index in 0..MARKER_PAGE_PANES {
                self.marker_page_mut(back_marker, index).hide();
            }
            for index in 0..CHART_PAGE_PANES {
                self.chart_page_mut(back_front, index).hide();
            }
            self.pane_trans_select_close(
                0,
                self.tune.reflow_frame,
                self.tune.scroll_ratio,
                1.0,
                0.0,
                0.0,
                4,
                back_marker,
                0,
            );
            self.pane_trans_select_open(
                0,
                self.tune.reflow_frame,
                self.tune.scroll_ratio,
                1.0,
                0.0,
                0.0,
                4,
                back_front,
                0,
            );
            self.pane_alpha_zoom_base(0, self.tune.reflow_frame, 0.0, 0.0, 0, back_zoom, 0);
            self.pane_alpha_chart_name(0, self.tune.reflow_frame, 0);
            self.pane_front ^= 1;
            self.pane_marker ^= 1;
            self.pane_zoom ^= 1;
            self.pane_name ^= 1;
            self.chart_move(charts, texts);
            self.pane_front ^= 1;
            self.pane_marker ^= 1;
            self.pane_zoom ^= 1;
            self.pane_name ^= 1;
            self.sync_name_origin(self.pane_name);
            self.shade_name_pane(self.pane_name);
        }
    }

    fn shade_name_pane(&mut self, page: u8) {
        let pane = self.name_page_mut(page, 16);
        pane.set_shade(SHADE_BLACK, SHADE_SOLID);
        pane.now_alpha = SHADE_ALPHA;
    }

    fn sync_name_origin(&mut self, page: u8) {
        let slot = usize::from(page) & 1;
        self.name_origin[slot] = self.name_page(page)[1].pos_top_left;
    }

    pub fn tick(
        &mut self,
        shared: &mut SharedCursor,
        pad: &ChartPad,
        charts: &mut Charts,
        world: &WorldFacts,
        texts: &dyn MsgSource,
        special_screen: Option<&Screen>,
    ) {
        self.salvage_point = world.salvage_point;
        self.sounds.clear();
        self.move_pad(shared, pad, charts, world, texts, special_screen);
    }

    pub fn move_pad(
        &mut self,
        shared: &mut SharedCursor,
        pad: &ChartPad,
        charts: &mut Charts,
        world: &WorldFacts,
        texts: &dyn MsgSource,
        special_screen: Option<&Screen>,
    ) {
        self.stick.check_trigger(pad.stick_value, pad.stick_angle);
        if self.proc_index == PROC_FMAP || self.proc_index == PROC_CMAP {
            if pad.lock_r {
                if shared.active == 0 {
                    self.sounds.push(SE_CHART_COMP_LR);
                    shared.active = 1;
                    self.mkfd_pane.show();
                    self.pane_alpha_chart_base(self.anim_tick, self.tune.scroll_frame, 0, 0);
                    self.proc_index = PROC_CHANGE_MODE;
                }
            } else if pad.lock_l {
                if shared.active == 1 {
                    self.sounds.push(SE_CHART_COMP_LR);
                    shared.active = 0;
                    let zoom = self.pane_zoom;
                    if self.zoom_page(zoom)[0].is_visible() {
                        self.mkm0_pane.now_alpha = MKM0_FMAP_ALPHA;
                        self.mkm0_pane.init_alpha = MKM0_FMAP_ALPHA;
                    } else {
                        self.mkm0_pane.now_alpha = MKM0_FMAP_DIM;
                        self.mkm0_pane.init_alpha = MKM0_FMAP_DIM;
                    }
                    self.mkcd_pane.show();
                    self.pane_alpha_field_base(self.anim_tick, self.tune.scroll_frame, 0, 0);
                    self.proc_index = PROC_CHANGE_MODE;
                }
            }
        }
        if self.proc_index >= PROC_CMAP {
            if self.stick.check_left_trigger() {
                if self.hold_left < self.tune.settle_limit {
                    self.hold_left += 1;
                }
            } else {
                self.hold_left = 0;
            }
            if self.stick.check_right_trigger() {
                if self.hold_right < self.tune.settle_limit {
                    self.hold_right += 1;
                }
            } else {
                self.hold_right = 0;
            }
        }
        match self.proc_index {
            PROC_FMAP => self.field_proc_main(shared),
            PROC_FMAP_CHANGE => self.field_change(),
            PROC_CMAP => self.chart_proc_main(pad, charts, texts),
            PROC_CMAP_IN => self.chart_proc_in(pad),
            PROC_CMAP_OPEN => self.chart_open(charts),
            PROC_CMAP_LOAD => self.special_load_wait(special_screen, charts, world),
            PROC_CMAP_OPEN_SP => self.chart_open_sp(charts),
            PROC_CMAP_SCROLL => self.chart_scroll(charts),
            PROC_CMAP_CLOSE => self.chart_close(),
            PROC_CHANGE_MODE => {}
            _ => {}
        }
        if shared.active == 1 {
            self.refresh_name_text(texts);
        }
    }

    fn refresh_name_text(&mut self, texts: &dyn MsgSource) {
        for slot in 0..PAGE_COUNT {
            let msg = self.name_msg[slot];
            if msg != NO_MESSAGE {
                let text = texts.message(u32::from(msg));
                let page = slot as u8;
                self.name_page_mut(page, 1).set_string(&text);
                self.name_page_mut(page, 2).set_string(&text);
            }
            let page = slot as u8;
            self.sync_name_origin(page);
        }
        let page = self.pane_name;
        self.shade_name_pane(page);
    }

    pub fn field_proc_main(&mut self, shared: &mut SharedCursor) {
        if self.stick.check_left_trigger() {
            if shared.cur_hx > -SECTOR_BIAS {
                shared.cur_hx -= 1;
                self.sounds.push(SE_CHART_CURSOR);
                self.r01g_pane.hide();
                self.proc_index = PROC_FMAP_CHANGE;
            }
        } else if self.stick.check_right_trigger() {
            if shared.cur_hx < SECTOR_BIAS {
                shared.cur_hx += 1;
                self.sounds.push(SE_CHART_CURSOR);
                self.r01g_pane.hide();
                self.proc_index = PROC_FMAP_CHANGE;
            }
        } else if self.stick.check_up_trigger() {
            if shared.cur_hy > -SECTOR_BIAS {
                shared.cur_hy -= 1;
                self.sounds.push(SE_CHART_CURSOR);
                self.r01g_pane.hide();
                self.proc_index = PROC_FMAP_CHANGE;
            }
        } else if self.stick.check_down_trigger() {
            if shared.cur_hy < SECTOR_BIAS {
                shared.cur_hy += 1;
                self.sounds.push(SE_CHART_CURSOR);
                self.r01g_pane.hide();
                self.proc_index = PROC_FMAP_CHANGE;
            }
        }
    }

    pub fn field_change(&mut self) {
        self.r01g_pane.show();
        self.proc_index = PROC_FMAP;
    }

    pub fn chart_proc_main(&mut self, pad: &ChartPad, charts: &mut Charts, texts: &dyn MsgSource) {
        if self.chart_total == 0 {
            return;
        }
        let mut next = i32::from(self.chart_cursor);
        if pad.trig_a {
            let kind = self.collect_map_kind(self.chart_cursor);
            self.anim_tick = 0;
            if is_special_map_kind(kind) {
                let page = self.pane_name as usize;
                self.name_msg[page] = if self.is_chart_open(self.chart_cursor, charts) {
                    MSG_MAP_READY as u16
                } else {
                    MSG_MAP_MISSING as u16
                };
                self.lock_confirm = 1;
                self.sounds.push(SE_CHART_OPEN_MAP);
                self.special_kind = kind as u8;
                self.sp_map_load_for_dvd(self.special_kind);
                self.proc_index = PROC_CMAP_LOAD;
            } else if self.chart_open_check(charts) {
                self.sounds.push(SE_CHART_OPEN_MAP);
                self.lock_confirm = 1;
                let front = self.pane_front;
                for index in 0..3 {
                    let pane = self.chart_page_mut(front, index);
                    pane.init_alpha = pane.user_area as u8;
                }
                let zoom = self.pane_zoom;
                for index in [0usize, 1, 2, 9] {
                    let pane = self.zoom_page_mut(zoom, index);
                    pane.init_alpha = pane.user_area as u8;
                }
                self.proc_index = PROC_CMAP_OPEN;
            }
        } else if self.stick.check_left_trigger() {
            self.scroll_dir = -1;
            next = self.next_chart(self.chart_cursor, -1, charts);
        } else if self.stick.check_right_trigger() {
            self.scroll_dir = 1;
            next = self.next_chart(self.chart_cursor, 1, charts);
        }
        if next != i32::from(self.chart_cursor) {
            self.sounds.push(SE_CHART_SEL_MAP);
            self.lock_confirm = 1;
            self.chart_prev = self.chart_cursor;
            self.chart_cursor = next as i8;
            if self.hold_left == self.tune.settle_limit || self.hold_right == self.tune.settle_limit
            {
                self.scroll_x = self.tune.settle_dx;
                self.scroll_y = self.tune.settle_dy;
                self.scroll_ratio = self.tune.settle_ratio;
                self.scroll_frame = self.tune.settle_frame;
            } else {
                self.scroll_x = self.tune.scroll_dx;
                self.scroll_y = self.tune.scroll_dy;
                self.scroll_ratio = self.tune.scroll_ratio;
                self.scroll_frame = self.tune.reflow_frame;
            }
            let frames = self.tune.reflow_frame;
            let front = self.pane_front ^ 1;
            let zoom = self.pane_zoom ^ 1;
            self.pane_trans_select_close(
                0,
                frames,
                self.tune.scroll_ratio,
                1.0,
                0.0,
                0.0,
                4,
                front,
                0,
            );
            self.pane_trans_select_open(
                0,
                frames,
                self.tune.scroll_ratio,
                1.0,
                0.0,
                0.0,
                4,
                front,
                0,
            );
            self.pane_alpha_zoom_base(0, frames, 0.0, 0.0, 0, zoom, 0);
            self.chart_move(charts, texts);
            self.proc_index = PROC_CMAP_SCROLL;
        }
    }

    pub fn sp_map_load_for_dvd(&mut self, kind: u8) {
        let front = self.pane_front;
        self.chart_page_mut(front, 0).hide();
        self.special_kind = kind;
        self.special_pending = special_archive(kind).is_some();
    }

    pub fn chart_open_check(&mut self, charts: &Charts) -> bool {
        if self.is_chart_open(self.chart_cursor, charts) {
            return false;
        }
        let front = self.pane_front;
        let zoom = self.pane_zoom;
        let name = self.pane_name;
        for index in 0..CHART_PAGE_PANES {
            self.chart_page_mut(front, index).show();
        }
        self.chart_page_mut(front, 4).hide();
        self.zoom_page_mut(zoom, 0).show();
        self.name_msg[usize::from(name)] = MSG_MAP_MISSING as u16;
        if self.collect_map_kind(self.chart_cursor) == 0 {
            if self.is_chart_open_triforce(self.chart_cursor, charts) {
                self.chart_page_mut(front, 3).hide();
            } else {
                self.chart_page_mut(front, 0).hide();
                self.zoom_page_mut(zoom, 0).hide();
            }
        } else {
            self.chart_page_mut(front, 3).hide();
        }
        self.pane_trans_select_open(
            self.anim_tick,
            self.tune.open_frame,
            1.0,
            1.0,
            0.0,
            0.0,
            0,
            front,
            0,
        );
        self.pane_alpha_zoom_base(self.anim_tick, self.tune.open_frame, 0.0, 0.0, 0, zoom, 0);
        true
    }

    pub fn chart_scroll(&mut self, charts: &Charts) {
        let (dx, dy) = if self.scroll_dir < 0 {
            (self.scroll_x, self.scroll_y)
        } else {
            (-self.scroll_x, -self.scroll_y)
        };
        let prev = self.chart_prev;
        let cur = self.chart_cursor;
        let frames = self.scroll_frame;
        let old_step = if self.is_chart_open(prev, charts) {
            Step::Unknown
        } else {
            self.pane_trans_select_close(
                self.anim_tick,
                frames,
                1.0,
                self.scroll_ratio,
                0.0,
                dx,
                4,
                self.pane_marker,
                1,
            )
        };
        let new_step = if self.is_chart_open(cur, charts) {
            Step::Unknown
        } else {
            self.pane_trans_select_close(
                self.anim_tick,
                frames,
                self.scroll_ratio,
                1.0,
                -dx,
                0.0,
                4,
                self.pane_marker ^ 1,
                0,
            )
        };
        let zoom = self.pane_zoom;
        let zoom_step = if self.zoom_page(zoom)[0].is_visible() {
            self.pane_alpha_zoom_base(self.anim_tick, frames, 0.0, dy, 0, zoom, 1)
        } else {
            Step::Done
        };
        let zoom_back_step = if self.zoom_page(zoom ^ 1)[0].is_visible() {
            self.pane_alpha_zoom_base(self.anim_tick, frames, -dy, 0.0, 0, zoom ^ 1, 0)
        } else {
            Step::Done
        };
        let name = self.pane_name;
        let name_step = if self.name_page(name)[0].is_visible() {
            Step::Unknown
        } else {
            Step::Done
        };
        let name_back_step = if self.name_page(name ^ 1)[0].is_visible() {
            Step::Unknown
        } else {
            Step::Done
        };
        let label_step = self.pane_alpha_chart_name(self.anim_tick, frames, 0);
        self.anim_tick += 1;
        if steps_done(&[
            old_step,
            new_step,
            zoom_step,
            zoom_back_step,
            name_step,
            name_back_step,
            label_step,
        ]) {
            self.anim_tick = 0;
            let zoom = self.pane_zoom;
            self.zoom_page_mut(zoom, 0).hide();
            let name = self.pane_name;
            self.name_page_mut(name, 0).hide();
            let marker = self.pane_marker;
            for index in 0..MARKER_PAGE_PANES {
                self.marker_page_mut(marker, index).hide();
            }
            let front = self.pane_front;
            for index in 0..CHART_PAGE_PANES {
                self.chart_page_mut(front, index).hide();
            }
            self.pane_zoom ^= 1;
            self.pane_front ^= 1;
            self.pane_marker ^= 1;
            self.pane_name ^= 1;
            self.lock_confirm = 0;
            self.proc_index = PROC_CMAP;
        }
    }

    pub fn chart_open(&mut self, charts: &mut Charts) {
        let name = self.pane_name;
        let zoom = self.pane_zoom;
        let frames = i16::from(self.tune.open_frame);
        let label_step = Step::Unknown;
        let body_step = if self.collect_map_kind(self.chart_cursor) == 0
            && !self.is_chart_open_triforce(self.chart_cursor, charts)
        {
            if self.anim_tick == frames {
                self.name_msg[usize::from(name)] = MSG_TRI_FOUND as u16;
            }
            Step::Unknown
        } else {
            self.pane_alpha_zoom_base(self.anim_tick - frames, frames as u8, 0.0, 0.0, 0, zoom, 0)
        };
        let clear_step = self.pane_trans_select_close(
            self.anim_tick - frames,
            frames as u8,
            1.0,
            1.0,
            0.0,
            0.0,
            0,
            self.pane_marker,
            1,
        );
        let open_step = Step::Unknown;
        self.anim_tick += 1;
        if steps_done(&[label_step, body_step, clear_step, open_step]) {
            self.anim_tick = 0;
            self.button_mode = 2;
            if self.collect_map_kind(self.chart_cursor) != 0
                || self.is_chart_open_triforce(self.chart_cursor, charts)
            {
                let name = self.pane_name;
                self.name_page_mut(name, 0).hide();
            }
            self.name_msg[usize::from(name)] = NO_MESSAGE;
            let marker = self.pane_marker;
            for index in 0..MARKER_PAGE_PANES {
                self.marker_page_mut(marker, index).hide();
            }
            if let Some(slot) = self.chart_slot(self.chart_cursor) {
                charts.open(slot);
            }
            self.lock_confirm = 0;
            self.proc_index = PROC_CMAP;
        }
    }

    pub fn special_load_wait(
        &mut self,
        screen: Option<&Screen>,
        charts: &Charts,
        world: &WorldFacts,
    ) {
        if !self.special_pending {
            return;
        }
        self.special_pending = false;
        self.pane_cursor = None;
        let kind = self.special_kind;
        if let Some(screen) = screen {
            self.set_special_screen(kind, screen, charts, world);
        }
        self.special_open = true;
        self.proc_index = PROC_CMAP_OPEN_SP;
    }

    pub fn chart_open_sp(&mut self, charts: &mut Charts) {
        let frames = i16::from(self.tune.close_frame);
        let label_step = Step::Unknown;
        let clear_step = self.pane_trans_select_close(
            self.anim_tick - frames,
            frames as u8,
            1.0,
            1.0,
            0.0,
            0.0,
            0,
            self.pane_marker,
            1,
        );
        let open_step = Step::Unknown;
        let body_step = Step::Unknown;
        self.anim_tick += 1;
        if steps_done(&[label_step, clear_step, open_step, body_step]) {
            self.anim_tick = 0;
            self.button_mode = 4;
            let name = self.pane_name;
            self.name_msg[usize::from(name)] = NO_MESSAGE;
            if let Some(slot) = self.chart_slot(self.chart_cursor) {
                charts.open(slot);
            }
            self.proc_index = PROC_CMAP_IN;
        }
    }

    pub fn chart_proc_in(&mut self, pad: &ChartPad) {
        if pad.trig_b {
            if self.special_kind == KIND_GHOST_SHIP {
                self.ghost_ship_moon_init();
                self.ghost_ship_icon_init();
            }
            let name = self.pane_name;
            self.name_msg[usize::from(name)] = MSG_MAP_READY as u16;
            self.proc_index = PROC_CMAP_CLOSE;
            if self.special_kind == KIND_TINGLE_MAP {
                self.sounds.push(SE_TC_MAP_RETURN);
            } else {
                self.sounds.push(SE_KAIZU_RETURN);
            }
        }
    }

    pub fn chart_close(&mut self) {
        let frames = i16::from(self.tune.close_frame);
        let label_step = Step::Unknown;
        let clear_step = self.pane_trans_select_close(
            self.anim_tick - frames,
            frames as u8,
            1.0,
            1.0,
            0.0,
            0.0,
            0,
            self.pane_marker,
            0,
        );
        let open_step = Step::Unknown;
        let body_step = Step::Unknown;
        self.anim_tick += 1;
        if steps_done(&[label_step, clear_step, open_step, body_step]) {
            self.anim_tick = 0;
            self.button_mode = 3;
            let name = self.pane_name;
            self.name_msg[usize::from(name)] = NO_MESSAGE;
            self.special = None;
            self.special_open = false;
            self.lock_confirm = 0;
            self.proc_index = PROC_CMAP;
        }
    }

    pub fn chart_move(&mut self, charts: &Charts, texts: &dyn MsgSource) {
        if !is_special_map_kind(self.collect_map_kind(self.chart_cursor)) {
            self.read_pane_chart_texture(self.pane_zoom as usize ^ 1, self.chart_cursor);
            self.change_select_chart(charts, texts);
        }
    }

    pub fn change_select_chart(&mut self, charts: &Charts, texts: &dyn MsgSource) {
        let kind = self.collect_map_kind(self.chart_cursor);
        let front = self.pane_front ^ 1;
        let zoom = self.pane_zoom ^ 1;
        let name = self.pane_name ^ 1;
        let marker = self.pane_marker ^ 1;
        if let Some(slot) = self.chart_texture[usize::from(zoom)] {
            if let Some(texture) = self.chart_textures.get(slot) {
                let texture = texture.clone();
                self.chart_page_mut(front, 0).change_texture(&texture);
            }
        }
        if self.is_chart_open(self.chart_cursor, charts) {
            self.button_mode = 2;
            self.zoom_page_mut(zoom, 0).show();
            for index in 0..MARKER_PAGE_PANES {
                self.marker_page_mut(marker, index).hide();
            }
            for index in 0..CHART_PAGE_PANES {
                self.chart_page_mut(front, index).show();
            }
            if kind == 0 {
                self.chart_page_mut(front, 1)
                    .change_texture(TEXTURE_TRIFORCE);
                self.chart_page_mut(front, 2)
                    .change_texture(TEXTURE_TRIFORCE);
                if self.is_chart_open_triforce(self.chart_cursor, charts) {
                    self.chart_page_mut(front, 3).hide();
                } else {
                    self.chart_page_mut(front, 0).hide();
                    self.zoom_page_mut(zoom, 0).hide();
                    self.name_page_mut(name, 0).show();
                    self.set_name_message(usize::from(name), MSG_TRI_FOUND, texts);
                }
            } else {
                self.chart_page_mut(front, 1).change_texture(TEXTURE_CHART);
                self.chart_page_mut(front, 2).change_texture(TEXTURE_CHART);
                self.chart_page_mut(front, 3).hide();
            }
            if !self.is_chart_complete(self.chart_cursor, charts) {
                self.chart_page_mut(front, 4).hide();
            }
        } else {
            self.button_mode = 3;
            self.zoom_page_mut(zoom, 0).hide();
            let texture = ROLL_MAP_TEX
                .get(usize::try_from(kind).unwrap_or(0))
                .copied()
                .unwrap_or(TEXTURE_CHART);
            for index in 0..MARKER_PAGE_PANES {
                self.marker_page_mut(marker, index).show();
                self.marker_page_mut(marker, index).change_texture(texture);
            }
            self.name_page_mut(name, 0).show();
            self.set_name_message(usize::from(name), MSG_MAP_MISSING, texts);
            if kind == 0 {
                self.chart_page_mut(front, 1)
                    .change_texture(TEXTURE_TRIFORCE);
                self.chart_page_mut(front, 2)
                    .change_texture(TEXTURE_TRIFORCE);
            } else {
                self.chart_page_mut(front, 1).change_texture(TEXTURE_CHART);
                self.chart_page_mut(front, 2).change_texture(TEXTURE_CHART);
            }
            for index in 0..CHART_PAGE_PANES {
                self.chart_page_mut(front, index).hide();
            }
        }
    }

    pub fn set_name_message(&mut self, slot: usize, msg_no: u32, texts: &dyn MsgSource) {
        let text = texts.message(msg_no);
        let page = slot as u8;
        self.name_page_mut(page, 1).set_string(&text);
        self.name_page_mut(page, 2).set_string(&text);
    }

    pub fn change_field_texture(&mut self, shared: &SharedCursor) {
        let sector = i32::from(shared.cur_hx)
            + (i32::from(shared.cur_hy) + SECTOR_STRIDE) * SECTOR_STRIDE
            + SECTOR_BASE;
        let name = world_texture_name(sector);
        if self.read_field_texture(&name) == 0 {
            self.r01g_pane.hide();
        } else {
            self.r01g_pane.show();
            if let Some(index) = self.world_texture {
                if let Some(texture) = self.world_textures.get(index).cloned() {
                    self.r01g_pane.change_texture(&texture);
                }
            }
        }
    }

    pub fn collect_map_tex_change(&mut self, charts: &Charts) {
        let count = self.calc_get_chart2(charts);
        let name = ruby_digit_texture(count % 10);
        self.rupee_texture[0] = Some(name);
        let name = ruby_digit_texture(count / 10);
        self.rupee_texture[1] = Some(name);
    }

    pub fn fin_collect_map_tex_change(&mut self, charts: &Charts) {
        let count = self.calc_fin_chart(charts);
        let name = ruby_digit_texture(count % 10);
        self.rupee_texture[2] = Some(name);
        let name = ruby_digit_texture(count / 10);
        self.rupee_texture[3] = Some(name);
    }

    pub fn calc_get_chart(&mut self, charts: &mut Charts) -> i32 {
        let mut result = 0;
        for chart in 0..CHART_EXPECTED_COUNT {
            let Some(no) = self.table.collect_map_no(chart) else {
                continue;
            };
            let slot = slot_of(no);
            if charts.has(slot) {
                result += 1;
                continue;
            }
            if charts.is_open(slot) {
                charts.close(slot);
            }
            if charts.is_done(slot) {
                self.stale_done.push(slot);
            }
        }
        result
    }

    pub fn calc_get_chart2(&self, charts: &Charts) -> i32 {
        let mut result = 0;
        for chart in 0..SECTOR_EXPECTED_COUNT as i32 {
            let Some(no) = self.table.collect_map_no(chart) else {
                continue;
            };
            if charts.has(slot_of(no)) {
                result += 1;
            }
        }
        result
    }

    pub fn calc_fin_chart(&self, charts: &Charts) -> i32 {
        let mut result = 0;
        for slot in 0..52usize {
            if slot != 34 && slot != 35 && charts.is_done(slot) {
                result += 1;
            }
        }
        result
    }

    pub fn first_chart(&self, charts: &Charts) -> i32 {
        if self.chart_total == 0 {
            return -1;
        }
        for chart in 0..CHART_EXPECTED_COUNT {
            let Some(no) = self.table.collect_map_no(chart) else {
                continue;
            };
            if charts.has(slot_of(no)) {
                return chart;
            }
        }
        -1
    }

    pub fn next_chart(&self, chart: i8, dir: i32, charts: &Charts) -> i32 {
        if self.chart_total == 1 {
            return i32::from(chart);
        }
        let mut index = i32::from(chart);
        let last = CHART_EXPECTED_COUNT - 1;
        loop {
            index += dir;
            if index < 0 {
                index = last;
            }
            if index > last {
                index = 0;
            }
            let Some(no) = self.table.collect_map_no(index) else {
                continue;
            };
            if charts.has(slot_of(no)) {
                return index;
            }
        }
    }

    pub fn get_now_chart_first_num(&self, charts: &Charts) -> i32 {
        if self.chart_total == 0 {
            return -1;
        }
        let mut index = 0;
        while index < CHART_EXPECTED_COUNT {
            let Some(no) = self.table.collect_map_no(index) else {
                break;
            };
            if no >= 0 && charts.has(slot_of(no)) {
                break;
            }
            index += 1;
        }
        if index > CHART_EXPECTED_COUNT {
            return -1;
        }
        index
    }

    pub fn get_now_chart_next_num(&self, start: i32, step: i32, charts: &Charts) -> i32 {
        if self.chart_total == 1 {
            return start;
        }
        let mut index = start;
        loop {
            index += step;
            if index < 0 {
                index = 60;
            }
            if index > 60 {
                index = 0;
            }
            let Some(no) = self.table.collect_map_no(index) else {
                return index;
            };
            if no >= 0 && charts.has(slot_of(no)) {
                break;
            }
        }
        index
    }

    pub fn chart_slot(&self, chart: i8) -> Option<usize> {
        self.table.collect_map_no(i32::from(chart)).map(slot_of)
    }

    pub fn collect_map_kind(&self, chart: i8) -> i32 {
        self.table.map_type(i32::from(chart)).map_or(-1, i32::from)
    }

    pub fn is_chart_got(&self, chart: i8, charts: &Charts) -> bool {
        self.chart_slot(chart).is_some_and(|slot| charts.has(slot))
    }

    pub fn is_chart_open(&self, chart: i8, charts: &Charts) -> bool {
        self.chart_slot(chart)
            .is_some_and(|slot| charts.is_open(slot))
    }

    pub fn is_chart_open_triforce(&self, chart: i8, charts: &Charts) -> bool {
        self.chart_slot(chart)
            .is_some_and(|slot| charts.triforce(slot as u8))
    }

    pub fn is_chart_complete(&self, chart: i8, charts: &Charts) -> bool {
        self.chart_slot(chart)
            .is_some_and(|slot| charts.is_done(slot))
    }

    pub fn lock_button(&self) -> bool {
        self.lock_confirm != 0
    }

    pub fn button_icon_mode(&self, shared: &SharedCursor) -> Option<i32> {
        match shared.active {
            0 => Some(0),
            1 => Some(if self.chart_total == 0 {
                1
            } else {
                i32::from(self.button_mode)
            }),
            _ => None,
        }
    }

    pub fn read_pane_chart_texture(&mut self, slot: usize, chart: i8) -> u32 {
        let Some(entry) = self.table.by_chart(i32::from(chart)) else {
            return 0;
        };
        let name = salvage_texture_name(entry.grid_no, self.salvage_point);
        self.read_chart_texture(slot, &name)
    }

    pub fn read_chart_texture(&mut self, slot: usize, name: &str) -> u32 {
        if slot >= PAGE_COUNT {
            return 0;
        }
        match lookup_texture(&self.chart_textures, name) {
            Some(index) => {
                self.chart_texture[slot] = Some(index);
                TEXTURE_SLOT_SIZE
            }
            None => 0,
        }
    }

    pub fn read_field_texture(&mut self, name: &str) -> u32 {
        match lookup_texture(&self.world_textures, name) {
            Some(index) => {
                self.world_texture = Some(index);
                TEXTURE_SLOT_SIZE
            }
            None => 0,
        }
    }

    pub fn pane_trans_base(
        &mut self,
        shared: &SharedCursor,
        anim_tick: i16,
        frames: u8,
        from: f32,
        to: f32,
        mode: u8,
        page: u8,
        variant: i32,
    ) -> bool {
        if anim_tick < 0 {
            return false;
        }
        if anim_tick > i16::from(frames) {
            return true;
        }
        let mut ratio = value_increase(i32::from(frames), i32::from(anim_tick), mode);
        self.cl_pane.pane_trans(0.0, from + ratio * (to - from));
        if variant != 2 {
            if variant == 1 {
                ratio = 1.0 - ratio;
            }
            if self.chart_total != 0
                && shared.chart_sel != -1
                && self.zoom_page(page)[0].is_visible()
            {
                for index in 0..ZOOM_PAGE_PANES {
                    let pane = self.zoom_page_mut(page, index);
                    pane.set_now_alpha(ratio);
                }
            }
            let front = self.pane_front;
            for index in 0..CHART_PAGE_PANES {
                let pane = self.chart_page_mut(front, index);
                pane.set_now_alpha(ratio);
            }
            self.nfkd_pane.set_now_alpha(ratio);
            self.mkfd_pane.set_now_alpha(ratio);
            self.ncdd_pane.set_now_alpha(ratio);
            self.mkcd_pane.set_now_alpha(ratio);
            self.kdm_pane.set_now_alpha(ratio);
            self.cl_pane.set_now_alpha(ratio);
        }
        false
    }

    pub fn pane_alpha_field_base(
        &mut self,
        anim_tick: i16,
        frames: u8,
        mode: u8,
        variant: i32,
    ) -> bool {
        if anim_tick < 0 {
            return false;
        }
        if anim_tick > i16::from(frames) {
            return true;
        }
        let mut ratio = value_increase(i32::from(frames), i32::from(anim_tick), mode);
        if variant != 2 {
            if variant == 1 {
                ratio = 1.0 - ratio;
            }
            self.mkfd_pane.set_now_alpha(ratio);
        }
        false
    }

    pub fn pane_alpha_chart_base(
        &mut self,
        anim_tick: i16,
        frames: u8,
        mode: u8,
        variant: i32,
    ) -> bool {
        if anim_tick < 0 {
            return false;
        }
        if anim_tick > i16::from(frames) {
            return true;
        }
        let mut ratio = value_increase(i32::from(frames), i32::from(anim_tick), mode);
        if variant != 2 {
            if variant == 1 {
                ratio = 1.0 - ratio;
            }
            self.cnd0_pane.now_alpha = ((1.0 - ratio) * CND0_CMAP_SPAN + CND0_FMAP_ALPHA) as u8;
            self.cnd0_pane.init_alpha = self.cnd0_pane.now_alpha;
            self.mkcd_pane.set_now_alpha(ratio);
        }
        false
    }

    pub fn pane_alpha_zoom_base(
        &mut self,
        anim_tick: i16,
        frames: u8,
        from: f32,
        to: f32,
        mode: u8,
        page: u8,
        variant: i32,
    ) -> Step {
        if anim_tick < 0 {
            return Step::Running;
        }
        if anim_tick > i16::from(frames) {
            return Step::Done;
        }
        let mut ratio = value_increase(i32::from(frames), i32::from(anim_tick), mode);
        let offset = ratio * (to - from);
        self.zoom_page_mut(page, 0).pane_trans(from + offset, 0.0);
        if variant != 2 {
            if variant == 1 {
                ratio = 1.0 - ratio;
            }
            for index in 0..ZOOM_PAGE_PANES {
                let pane = self.zoom_page_mut(page, index);
                pane.set_now_alpha(ratio);
            }
        }
        Step::Running
    }

    pub fn pane_alpha_chart_name(&mut self, anim_tick: i16, frames: u8, mode: u8) -> Step {
        if anim_tick < 0 {
            return Step::Running;
        }
        if anim_tick > i16::from(frames) {
            return Step::Done;
        }
        let ratio = value_increase(i32::from(frames), i32::from(anim_tick), mode);
        let inverse = 1.0 - ratio;
        let front = usize::from(self.pane_marker) & 1;
        let back = front ^ 1;
        self.island_name_panes[front].set_now_alpha(inverse);
        self.island_name_panes[front + 2].set_now_alpha(inverse);
        self.island_name_panes[back].set_now_alpha(ratio);
        self.island_name_panes[back + 2].set_now_alpha(ratio);
        Step::Running
    }

    pub fn pane_alpha_message2(
        &mut self,
        anim_tick: i16,
        frames: u8,
        mode: u8,
        variant: i32,
    ) -> bool {
        if anim_tick < 0 {
            return false;
        }
        if anim_tick > i16::from(frames) {
            return true;
        }
        let mut ratio = value_increase(i32::from(frames), i32::from(anim_tick), mode);
        if variant != 2 {
            if variant == 1 {
                ratio = 1.0 - ratio;
            }
            self.ms_panes[0].set_now_alpha(ratio);
        }
        false
    }

    pub fn pane_trans_select_close(
        &mut self,
        anim_tick: i16,
        frames: u8,
        from: f32,
        to: f32,
        shift_from: f32,
        shift_to: f32,
        mode: u8,
        page: u8,
        variant: i32,
    ) -> Step {
        if anim_tick < 0 {
            return Step::Running;
        }
        if anim_tick > i16::from(frames) {
            return Step::Done;
        }
        let mut ratio = value_increase(i32::from(frames), i32::from(anim_tick), mode);
        for index in 0..MARKER_PAGE_PANES {
            let pane = self.marker_page_mut(page, index);
            pane.pane_scale_xy(from + ratio * (from - to));
            pane.pane_trans(shift_from + ratio * (shift_from - shift_to), 0.0);
        }
        if variant != 2 {
            if variant == 1 {
                ratio = 1.0 - ratio;
            }
            for index in 0..MARKER_PAGE_PANES {
                let pane = self.marker_page_mut(page, index);
                pane.set_now_alpha(ratio);
            }
        }
        Step::Running
    }

    pub fn pane_trans_select_open(
        &mut self,
        anim_tick: i16,
        frames: u8,
        from: f32,
        to: f32,
        shift_from: f32,
        shift_to: f32,
        mode: u8,
        page: u8,
        variant: i32,
    ) -> Step {
        let _ = (
            anim_tick, frames, from, to, shift_from, shift_to, mode, page, variant,
        );
        Step::Unknown
    }

    pub fn ghost_ship_moon_init(&mut self) {
        self.moon_x = self.tune.moon_dx;
        self.moon_y = self.tune.moon_dy;
        self.moon_step = 1;
        self.moon_from = self.tune.moon_from;
        self.moon_to = self.tune.moon_to;
    }

    pub fn ghost_ship_icon_init(&mut self) {
        self.icon_x = self.tune.icon_dx;
        self.icon_y = self.tune.icon_dy;
        self.icon_step = 1;
    }

    pub fn treasure_triforce_init(&mut self) {
        self.tri_x = self.tune.tri_dx;
        self.tri_step = 1;
    }

    fn install_special(&mut self, kind: u8, panes: Vec<Pane>) {
        let find = |name: &str| panes.iter().position(|pane| pane.tag == name);
        let link = find("lnk").unwrap_or(usize::MAX);
        let target = find("gdgt").unwrap_or(usize::MAX);
        let guide = find("gddm").unwrap_or(usize::MAX);
        let mut moon_shade = Shade {
            black: SHADE_BLACK,
            white: [255, 255, 255, 255],
        };
        let mut key_shade = moon_shade;
        if kind == KIND_GHOST_SHIP {
            if let Some(index) = find("tk01") {
                moon_shade = panes[index].shade;
            }
            if let Some(index) = find("kz01") {
                key_shade = panes[index].shade;
            }
        }
        self.special = Some(SpecialScreen {
            kind,
            entry: special_screen_entry(kind).unwrap_or(""),
            link,
            target,
            guide,
            moon_shade,
            key_shade,
            panes,
        });
    }

    fn bind_special_panes(&self, screen: &Screen, tags: &[&'static str]) -> Vec<Pane> {
        tags.iter()
            .map(|tag| {
                let mut pane = Pane::new(tag);
                if let Some(layout) = screen.find(&pane.tag).cloned() {
                    pane.parts_set(&layout);
                }
                pane
            })
            .collect()
    }

    pub fn screen_set_tingle(&mut self, screen: &Screen) {
        let panes = self.bind_special_panes(screen, &TINGLE_TAGS);
        self.install_special(KIND_TINGLE_MAP, panes);
        self.set_player_pos();
    }

    pub fn screen_set_doctor(&mut self, screen: &Screen) {
        let panes = self.bind_special_panes(screen, &DOCTOR_TAGS);
        self.install_special(KIND_DOCTOR_MAP, panes);
        self.set_player_pos();
    }

    pub fn screen_set_dark_path(&mut self, screen: &Screen) {
        let panes = self.bind_special_panes(screen, &DARK_PATH_TAGS);
        self.install_special(KIND_DARK_PATH_MAP, panes);
        self.set_player_pos();
    }

    pub fn screen_set_heart_piece(&mut self, screen: &Screen) {
        let panes = self.bind_special_panes(screen, &HEART_PIECE_TAGS);
        self.install_special(KIND_HEART_PIECE_MAP, panes);
        self.set_player_pos();
    }

    pub fn screen_set_heart(&mut self, screen: &Screen) {
        let panes = self.bind_special_panes(screen, &HEART_TAGS);
        self.install_special(KIND_HEART_MAP, panes);
        self.set_player_pos();
    }

    pub fn screen_set_subdan(&mut self, screen: &Screen) {
        let panes = self.bind_special_panes(screen, &SUBDAN_TAGS);
        self.install_special(KIND_SUBDAN_MAP, panes);
        self.set_player_pos();
    }

    pub fn screen_set_moon(&mut self, screen: &Screen) {
        let panes = self.bind_special_panes(screen, &MOON_TAGS);
        self.install_special(KIND_MOON_MAP, panes);
        self.set_player_pos();
    }

    pub fn screen_set_tower(&mut self, screen: &Screen) {
        let panes = self.bind_special_panes(screen, &TOWER_TAGS);
        self.install_special(KIND_TOWER_MAP, panes);
        self.set_player_pos();
    }

    pub fn screen_set_terry(&mut self, screen: &Screen) {
        let panes = self.bind_special_panes(screen, &TERRY_TAGS);
        self.install_special(KIND_TERRY_MAP, panes);
        self.set_player_pos();
    }

    pub fn screen_set_sub_marine(&mut self, screen: &Screen) {
        let panes = self.bind_special_panes(screen, &SUB_MARINE_TAGS);
        self.install_special(KIND_SUB_MARINE_MAP, panes);
        self.set_player_pos();
    }

    pub fn set_special_screen(
        &mut self,
        kind: u8,
        screen: &Screen,
        charts: &Charts,
        world: &WorldFacts,
    ) {
        match kind {
            KIND_TINGLE_MAP => self.screen_set_tingle(screen),
            KIND_DOCTOR_MAP => self.screen_set_doctor(screen),
            KIND_DARK_PATH_MAP => self.screen_set_dark_path(screen),
            KIND_HEART_PIECE_MAP => self.screen_set_heart_piece(screen),
            KIND_HEART_MAP => self.screen_set_heart(screen),
            KIND_SUBDAN_MAP => self.screen_set_subdan(screen),
            KIND_MOON_MAP => self.screen_set_moon(screen),
            KIND_TOWER_MAP => self.screen_set_tower(screen),
            KIND_TERRY_MAP => self.screen_set_terry(screen),
            KIND_SUB_MARINE_MAP => self.screen_set_sub_marine(screen),
            KIND_GHOST_SHIP => {
                let Some(tags) = special_tags(kind) else {
                    return;
                };
                let panes = self.bind_special_panes(screen, tags);
                self.install_special(kind, panes);
                self.ghost_ship_screen_set(world);
            }
            KIND_TREASURE_MAP => {
                let Some(tags) = special_tags(kind) else {
                    return;
                };
                let panes = self.bind_special_panes(screen, tags);
                self.install_special(kind, panes);
                self.treasure_screen_set(charts);
                self.set_player_pos();
            }
            _ => {}
        }
    }

    pub fn ghost_ship_screen_set(&mut self, world: &WorldFacts) {
        if world.ghost_ship == GHOST_SHIP_PROGRESS {
            self.ghost_ship_done = 1;
            self.moon_on = 0;
            if let Some(special) = self.special.as_mut() {
                for index in GS_MOON_FIRST..GS_MOON_FIRST + 7 {
                    special.panes[index].hide();
                }
                special.panes[GS_KEY_INDEX].hide();
                special.panes[GS_DOOR_INDEX].now_alpha = 200;
                special.panes[GS_DOOR_INDEX].init_alpha = 200;
            }
            return;
        }
        self.ghost_ship_done = 0;
        self.moon_on = 0;
        if world.time >= TUNE_MOON_CLEAR_TIME
            || (world.time >= 0.0 && world.time <= TUNE_MOON_SET_TIME)
        {
            self.moon_on = 1;
            if let Some(special) = self.special.as_mut() {
                let slot = usize::try_from(world.moon).unwrap_or(0);
                if slot < 7 {
                    special.panes[GS_ICON_FIRST + slot].show();
                }
            }
        }
        self.ghost_ship_moon_init();
        self.ghost_ship_icon_init();
    }

    pub fn treasure_screen_set(&mut self, charts: &Charts) {
        let mut marks: [bool; 8] = [false; 8];
        let mut tri: [bool; 8] = [false; 8];
        let mut done: [bool; 8] = [false; 8];
        for slot in 0..8usize {
            let chart = slot as i8;
            if !self.is_chart_got(chart, charts) {
                continue;
            }
            marks[slot] = true;
            if !self.is_chart_open_triforce(chart, charts) {
                continue;
            }
            tri[slot] = true;
            done[slot] = self.is_chart_complete(chart, charts);
        }
        let base = TREASURE_MARKS_FIRST;
        if let Some(special) = self.special.as_mut() {
            for slot in 0..8usize {
                if !marks[slot] {
                    continue;
                }
                special.panes[base + slot].show();
                if !tri[slot] {
                    continue;
                }
                special.panes[base + 8 + slot].show();
                if done[slot] {
                    special.panes[base + 16 + slot].show();
                }
            }
        }
        self.treasure_triforce_init();
    }

    pub fn set_player_pos(&mut self) {
        let Some(mut special) = self.special.take() else {
            return;
        };
        let link = special.link;
        let target = special.target;
        let guide = special.guide;
        if self.player_valid != 0 && guide != usize::MAX && link != usize::MAX {
            let origin_x = special.panes[guide].pos_center_orig.x;
            let origin_y = special.panes[guide].pos_center_orig.y;
            let size_x = special.panes[guide].size_orig.x;
            let size_y = special.panes[guide].size_orig.y;
            special.panes[link].pos_center_orig.x = origin_x + self.player_x * size_x / WORLD_SCALE;
            special.panes[link].pos_center_orig.y = origin_y + self.player_z * size_y / WORLD_SCALE;
            let center_x = special.panes[link].pos_center_orig.x;
            let center_y = special.panes[link].pos_center_orig.y;
            special.panes[link].pos_center.x = center_x;
            special.panes[link].pos_center.y = center_y;
            special.panes[link].cpos_move();
            let origin_x = special.panes[link].size_orig.x * 0.5;
            let origin_y = special.panes[link].size_orig.y * 0.5;
            let angle = self.player_heading;
            special.panes[link].rotate(origin_x, origin_y, angle);
        } else if link != usize::MAX {
            special.panes[link].hide();
        }
        if target != usize::MAX {
            let size_x = special.panes[target].size_orig.x;
            let size_y = special.panes[target].size_orig.y;
            let x = f32::from(self.player_grid_x) * size_x;
            let y = f32::from(self.player_grid_y) * size_y;
            special.panes[target].pane_trans(x, y);
        }
        self.pane_cursor = if target == usize::MAX {
            None
        } else {
            Some(target)
        };
        self.special = Some(special);
    }

    pub fn name_page(&self, page: u8) -> &[Pane; MSG_NAME_PANES] {
        if usize::from(page) & 1 == 0 {
            &self.name_panes_a
        } else {
            &self.name_panes_b
        }
    }

    fn name_page_mut(&mut self, page: u8, index: usize) -> &mut Pane {
        if usize::from(page) & 1 == 0 {
            &mut self.name_panes_a[index]
        } else {
            &mut self.name_panes_b[index]
        }
    }

    fn chart_page_mut(&mut self, page: u8, index: usize) -> &mut Pane {
        if usize::from(page) & 1 == 0 {
            &mut self.chart_page_a[index]
        } else {
            &mut self.chart_page_b[index]
        }
    }

    fn zoom_page(&self, page: u8) -> &[Pane; ZOOM_PAGE_PANES] {
        if usize::from(page) & 1 == 0 {
            &self.zoom_page_a
        } else {
            &self.zoom_page_b
        }
    }

    fn zoom_page_mut(&mut self, page: u8, index: usize) -> &mut Pane {
        if usize::from(page) & 1 == 0 {
            &mut self.zoom_page_a[index]
        } else {
            &mut self.zoom_page_b[index]
        }
    }

    fn marker_page_mut(&mut self, page: u8, index: usize) -> &mut Pane {
        if usize::from(page) & 1 == 0 {
            &mut self.marker_page_a[index]
        } else {
            &mut self.marker_page_b[index]
        }
    }

    pub fn draw(&self, shared: &SharedCursor, charts: &Charts) -> ChartDraw {
        let mut out = ChartDraw {
            proc_index: self.proc_index,
            active: shared.active,
            button_mode: self.button_icon_mode(shared),
            confirm_locked: self.lock_button(),
            world_cursor: [shared.cur_hx, shared.cur_hy],
            chart_cursor: i32::from(self.chart_cursor),
            chart_previous: i32::from(self.chart_prev),
            scroll_dir: self.scroll_dir,
            chart_total: usize::from(self.chart_total),
            chart_open: usize::try_from(self.calc_get_chart2(charts)).unwrap_or(0),
            chart_done: usize::try_from(self.calc_fin_chart(charts)).unwrap_or(0),
            anim_tick: self.anim_tick,
            scroll: [self.scroll_x, self.scroll_y, self.scroll_ratio],
            hold: [self.hold_left, self.hold_right],
            player_grid: [self.player_grid_x, self.player_grid_y],
            player_pos: [self.player_x, self.player_z],
            player_heading: self.player_heading,
            player_valid: self.player_valid != 0,
            world_texture: self.world_texture,
            chart_texture: self.chart_texture,
            rupee_texture: self.rupee_texture.clone(),
            name_page: self.pane_name,
            name_origin: self.name_origin,
            page_front: self.pane_front,
            marker_page: self.pane_marker,
            zoom_page: self.pane_zoom,
            shade: [self.shade_black, self.shade_white],
            special_kind: self.special_kind,
            special_archive: special_archive(self.special_kind),
            special_open: self.special_open,
            special_pending: self.special_pending,
            pane_cursor: self.pane_cursor,
            sounds: self.sounds.clone(),
            list: Vec::new(),
            panes: Vec::new(),
        };
        if self.screen_loaded {
            out.list.push(ListItem { screen: LIST_WORLD });
        }
        if self.special.is_some() {
            out.list.push(ListItem {
                screen: LIST_SPECIAL,
            });
        }
        if let Some(special) = &self.special {
            for pane in special.panes.iter() {
                out.panes.push(pane_draw(pane));
            }
        }
        out.panes.push(pane_draw(&self.cl_pane));
        out.panes.push(pane_draw(&self.kdm_pane));
        for pane in self.fc_panes.iter() {
            out.panes.push(pane_draw(pane));
        }
        for pane in self.cc_panes.iter() {
            out.panes.push(pane_draw(pane));
        }
        for pane in self.ms_panes.iter() {
            out.panes.push(pane_draw(pane));
        }
        for index in 0..MSG_NAME_PANES {
            out.panes.push(pane_draw(&self.name_panes_a[index]));
            out.panes.push(pane_draw(&self.name_panes_b[index]));
        }
        for pane in self.island_name_panes.iter() {
            out.panes.push(pane_draw(pane));
        }
        for pane in self.key_panes.iter() {
            out.panes.push(pane_draw(pane));
        }
        for pane in self.extra_panes.iter() {
            out.panes.push(pane_draw(pane));
        }
        for pane in self.sector_panes.iter() {
            out.panes.push(pane_draw(pane));
        }
        for index in 0..CHART_PAGE_PANES {
            out.panes.push(pane_draw(&self.chart_page_a[index]));
            out.panes.push(pane_draw(&self.chart_page_b[index]));
        }
        for index in 0..ZOOM_PAGE_PANES {
            out.panes.push(pane_draw(&self.zoom_page_a[index]));
            out.panes.push(pane_draw(&self.zoom_page_b[index]));
        }
        for index in 0..MARKER_PAGE_PANES {
            out.panes.push(pane_draw(&self.marker_page_a[index]));
            out.panes.push(pane_draw(&self.marker_page_b[index]));
        }
        for pane in [
            &self.cmyu_pane,
            &self.cmyd_pane,
            &self.cnd0_pane,
            &self.nfkd_pane,
            &self.mkfd_pane,
            &self.mkcd_pane,
            &self.ncdd_pane,
            &self.mkm0_pane,
            &self.lnk3_pane,
            &self.r01g_pane,
            &self.area_pane,
        ] {
            out.panes.push(pane_draw(pane));
        }
        out
    }
}

fn pane_draw(pane: &Pane) -> PaneDraw {
    PaneDraw {
        tag: pane.tag.clone(),
        visible: pane.visible,
        x: pane.pos_top_left.x,
        y: pane.pos_top_left.y,
        w: pane.size.x,
        h: pane.size.y,
        alpha: pane.now_alpha,
        rotate: pane.rotate_angle,
        texture: pane.texture.clone(),
        text: pane.text.clone(),
    }
}

fn slot_of(collect_map_no: i8) -> usize {
    usize::try_from(i32::from(collect_map_no) - 1).unwrap_or(usize::MAX)
}

fn be_i8(data: &[u8], at: usize) -> Option<i8> {
    Some(*data.get(at)? as i8)
}

fn be_i16(data: &[u8], at: usize) -> Option<i16> {
    let bytes = data.get(at..at + 2)?;
    Some(i16::from_be_bytes([bytes[0], bytes[1]]))
}

fn read_u32(data: &[u8], at: usize) -> Option<u32> {
    let bytes = data.get(at..at + WORD_SIZE)?;
    Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_i32(data: &[u8], at: usize) -> Option<i32> {
    read_u32(data, at).map(|value| value as i32)
}

pub fn init(world: &mut World) {
    world.init_resource::<ChartLayer>();
    world.init_resource::<SharedCursor>();
    world.init_resource::<ChartPadInput>();
    world.init_resource::<ChartText>();
}

pub fn load_tables(layer: &mut ChartLayer, root: &AssetRoot) {
    let charts = load_chart_table(root).unwrap_or_default();
    let sectors = load_sector_table(root).unwrap_or_default();
    layer.bind_tables(charts, sectors);
    let textures = read_map_texture_names(root);
    layer.bind_textures(textures.clone(), textures);
}

#[derive(SystemParam)]
pub struct ChartEnv<'w> {
    pub layer: ResMut<'w, ChartLayer>,
    pub shared: ResMut<'w, SharedCursor>,
    pub charts: ResMut<'w, Charts>,
    pub day: Res<'w, DayNight>,
    pub regs: Res<'w, EventReg>,
    pub arrivals: Res<'w, Arrivals>,
    pub salvage: Res<'w, salvage::Control>,
    pub pad: Res<'w, ChartPadInput>,
    pub text: Res<'w, ChartText>,
}

pub fn chart_menu(env: &mut ChartEnv) {
    let facts = world_facts(&env.day, &env.regs, &env.arrivals, &env.salvage);
    let pad = env.pad.pad();
    let text = env.text.clone();
    env.layer
        .tick(&mut env.shared, &pad, &mut env.charts, &facts, &text, None);
}

pub fn chart_draw(env: &ChartEnv) -> ChartDraw {
    env.layer.draw(&env.shared, &env.charts)
}
