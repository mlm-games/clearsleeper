use bevy_ecs::prelude::*;

use crate::j2d::Screen;
use crate::save::{SLOTS, SaveGame};
use crate::sim::daynight::{DayNight, TIME_WRAP};
use crate::ui_menu_option::value_increase;
use crate::ui_menu_save::{
    blank_save, Color, MemCard, Pane, Particles, PadInput as SavePadInput, SlotState, StickControl,
};

pub const SCREEN_LABEL: &str = "file_select.blo";
pub const SCREEN_ARCHIVE: &str = "files/res/Stage/Name/Stage.arc";
pub const GAME_OVER_LABEL: &str = "gameover.blo";
pub const GAME_OVER_ARCHIVE: &str = "files/res/Object/Gover.arc";
pub const TEXT_BUFFER_NAME: usize = 0x30;
pub const TEXT_BUFFER_INFO: usize = 0x80;
pub const SIM_HZ: u64 = 30;
pub const WEEK_DAYS: i32 = 7;
pub const TIME_PATTERN_COUNT: i32 = 0x1d;

pub const IDX_07: u8 = 1;
pub const IDX_0b: u8 = 0xA;
pub const IDX_08: u8 = 6;
pub const IDX_09: u8 = 0xA;
pub const IDX_48: i16 = -0x96;
pub const IDX_0c: u8 = 6;
pub const IDX_0d: u8 = 6;
pub const IDX_0e: u8 = 6;
pub const IDX_0f: u8 = 0;
pub const IDX_10: u8 = 6;
pub const IDX_18: i16 = -0xFA;
pub const IDX_14: u8 = 0xA;
pub const IDX_11: u8 = 6;
pub const IDX_1a: i16 = 0xFA;
pub const IDX_15: u8 = 5;
pub const IDX_12: u8 = 6;
pub const IDX_1c: i16 = 0xFA;
pub const IDX_16: u8 = 0;
pub const IDX_13: u8 = 6;
pub const IDX_1e: i16 = 0xFA;
pub const IDX_21: u8 = 0;
pub const IDX_20: u8 = 0;
pub const IDX_22: u8 = 0xC;
pub const IDX_23: u8 = 0x8C;
pub const IDX_24: u8 = 0x46;
pub const IDX_25: u8 = 0xC;
pub const IDX_26: u8 = 0x8C;
pub const IDX_27: u8 = 0x46;
pub const IDX_28: u8 = 6;
pub const IDX_40: i32 = 0x190;
pub const IDX_29: u8 = 0;
pub const IDX_2a: u8 = 5;
pub const IDX_2b: u8 = 5;
pub const IDX_2c: u8 = 0xA;
pub const IDX_2d: u8 = 8;
pub const IDX_2e: u8 = 5;
pub const IDX_2f: u8 = 6;
pub const IDX_30: u8 = 5;
pub const IDX_31: u8 = 0;
pub const IDX_32: u8 = 6;
pub const IDX_33: u8 = 0;
pub const IDX_44: i32 = 0x1F4;
pub const IDX_0a: u8 = 5;
pub const IDX_34: u8 = 6;
pub const IDX_35: u8 = 5;
pub const IDX_36: u8 = 0;
pub const IDX_37: u8 = 2;
pub const IDX_38: u8 = 8;
pub const IDX_39: u8 = 0;
pub const IDX_42: i32 = 0x64;
pub const IDX_46: i32 = -0x12C;
pub const IDX_3a: u8 = 0;
pub const IDX_3b: u8 = 0;
pub const IDX_3c: i32 = 0x190;
pub const IDX_3e: i32 = 0xC8;
pub const IDX_4f: u8 = 5;
pub const IDX_4e: i32 = 0x14;
pub const IDX_50: u8 = 0;
pub const IDX_51: u8 = 0;
pub const IDX_52: u8 = 0;
pub const IDX_05: u8 = 0;
pub const IDX_06: u8 = 0;
pub const IDX_4a: i16 = 0x5A;
pub const IDX_4c: i32 = 0x64;
pub const IDX_54: f32 = 1.25;
pub const IDX_58: f32 = 12.0;

pub const COLLEC_TAGS: [&str; 12] = [
    "ci00", "ci01", "ci02", "ci03", "ci04", "ci05", "ci06", "ci07", "ci08", "ci09", "ci10",
    "ci11",
];
pub const HEART_TAGS: [&str; 0x14] = [
    "ht00", "ht01", "ht02", "ht03", "ht04", "ht05", "ht06", "ht07", "ht08", "ht09", "ht10",
    "ht11", "ht12", "ht13", "ht14", "ht15", "ht16", "ht17", "ht18", "ht19",
];
pub const HEART_SHADOW_TAGS: [&str; 0x14] = [
    "hk00", "hk01", "hk02", "hk03", "hk04", "hk05", "hk06", "hk07", "hk08", "hk09", "hk10",
    "hk11", "hk12", "hk13", "hk14", "hk15", "hk16", "hk17", "hk18", "hk19",
];
pub const REC_INFO_1_TAGS: [&str; 0x12] = [
    "1000", "1001", "1002", "1003", "1004", "1005", "1006", "1007", "1008", "1009", "1010",
    "1011", "1012", "1013", "1014", "1015", "1016", "1017",
];
pub const REC_INFO_2_TAGS: [&str; 0x12] = [
    "2000", "2001", "2002", "2003", "2004", "2005", "2006", "2007", "2008", "2009", "2010",
    "2011", "2012", "2013", "2014", "2015", "2016", "2017",
];
pub const REC_INFO_3_TAGS: [&str; 0x12] = [
    "3000", "3001", "3002", "3003", "3004", "3005", "3006", "3007", "3008", "3009", "3010",
    "3011", "3012", "3013", "3014", "3015", "3016", "3017",
];
pub const TRIFORCE_TAGS: [&str; 9] = [
    "tri0", "tri1", "tri2", "tri3", "tri4", "tri5", "tri6", "tri7", "tri8",
];
pub const PEARL_TAGS: [&str; 3] = ["it03", "it01", "it02"];
pub const PEARL_SHADOW_TAGS: [&str; 3] = ["ik03", "ik01", "ik02"];

pub const SLOT_KEY_TAGS: [&str; 16] = [
    "ak00", "1dat", "a000", "a001", "a002", "a003", "a004", "a005", "a006", "a007",
    "a008", "a009", "ak01", "ak02", "ak03", "ak04",
];
pub const SLOT_KEY_EXTRA_TAGS: [&str; 4] = ["ak01", "ak02", "ak03", "ak04"];
pub const SLOT_KEY2_TAGS: [&str; 16] = [
    "bk00", "2dat", "b000", "b001", "b002", "b003", "b004", "b005", "b006", "b007",
    "b008", "b009", "bk01", "bk02", "bk03", "bk04",
];
pub const SLOT_KEY2_EXTRA_TAGS: [&str; 4] = ["bk01", "bk02", "bk03", "bk04"];
pub const SLOT_KEY3_TAGS: [&str; 16] = [
    "ck00", "3dat", "c000", "c001", "c002", "c003", "c004", "c005", "c006", "c007",
    "c008", "c009", "ck01", "ck02", "ck03", "ck04",
];
pub const SLOT_KEY3_EXTRA_TAGS: [&str; 4] = ["ck01", "ck02", "ck03", "ck04"];

pub const NAME_TAGS: [&str; 3] = ["nam1", "nam2", "nam3"];
pub const DAY_TAGS: [&str; 3] = ["day1", "day2", "day3"];

pub const SLOT_COUNT: usize = SLOTS;
pub const SLOT_ROW_DY: f32 = 85.0;
pub const SLOT_COPY_DY: f32 = 55.0;
pub const SLOT_COPY_DY_ALT: f32 = 140.0;
pub const SLOT_TITLE_SCALE: f32 = 1.25;
pub const SLOT_TITLE_DY: f32 = 12.0;
pub const SLOT_BROKEN_NAME: &str = "This data is corrupted.";
pub const SLOT_NEW_NAME: &str = "New Game";

pub const MENU_X: [f32; 4] = [0.0, 131.0, 262.0, 394.0];
pub const YES_NO_X: [f32; 2] = [0.0, 139.0];

pub const MENU_POS_START: u8 = 0;
pub const MENU_POS_COPY: u8 = 1;
pub const MENU_POS_ERASE: u8 = 2;
pub const MENU_POS_RETURN: u8 = 3;

pub const ICON_MODE_DEFAULT: u8 = 3;
pub const ICON_MODE_SELECT: u8 = 4;
pub const ICON_MODE_BACK: u8 = 5;
pub const ICON_MODE_WAIT: u8 = 6;

pub const SELECT_END_NONE: u8 = 0;
pub const SELECT_END_START: u8 = 1;
pub const SELECT_END_CANCEL: u8 = 2;
pub const SELECT_END_RETURN: u8 = 3;

pub const MODE_NORMAL: u8 = 0;
pub const MODE_EXTRA: u8 = 1;

pub const OP_NONE: u8 = 0;
pub const OP_SELECT: u8 = 1;
pub const OP_COPY: u8 = 3;
pub const OP_ERASE: u8 = 4;
pub const OP_ERASE_EXTRA: u8 = 5;
pub const OP_CARD_ERR: u8 = 6;
pub const OP_CARD_REMOVED: u8 = 7;
pub const OP_BACK: u8 = 8;

pub const ERR_WAIT_FRAMES: u8 = 0x5A;

pub const ERASE_COLOR_A: [u8; 4] = [0x28, 0x6E, 0x1E, 0];
pub const ERASE_COLOR_B: [u8; 4] = [0x8C, 0xB4, 0x50, 0xFF];
pub const ERASE_COLOR_C: [u8; 4] = [0xBE, 0xDC, 0x8C, 0];
pub const ERASE_COLOR_D: [u8; 4] = [0xD2, 0xFF, 0x73, 0xFF];
pub const EXTRA_COLOR_A: [u8; 4] = [0x6E, 0x50, 0, 0];
pub const EXTRA_COLOR_B: [u8; 4] = [0x96, 0x78, 0, 0xFF];
pub const EXTRA_COLOR_C: [u8; 4] = [0xDC, 0xA0, 0, 0];
pub const EXTRA_COLOR_D: [u8; 4] = [0xFF, 0xB4, 0x0F, 0xFF];
pub const EXTRA_INFO_A: [u8; 4] = [0x6E, 0x50, 0, 0];
pub const EXTRA_INFO_B: [u8; 4] = [0x96, 0x78, 0, 0xFF];
pub const EXTRA_INFO_C: [u8; 4] = [0xDC, 0xB4, 0x5A, 0];
pub const EXTRA_INFO_D: [u8; 4] = [0xFF, 0xC8, 0x5F, 0xFF];
pub const BROKEN_INFO_A: [u8; 4] = [0x50, 0x50, 0x50, 0];
pub const BROKEN_INFO_B: [u8; 4] = [0x78, 0x78, 0x78, 0xFF];
pub const BROKEN_INFO_C: [u8; 4] = [0x96, 0x96, 0x96, 0];
pub const BROKEN_INFO_D: [u8; 4] = [0xC8, 0xC8, 0xC8, 0xFF];
pub const PLAIN_INFO_A: [u8; 4] = [0xFF, 0xF0, 0xBE, 0];
pub const PLAIN_INFO_B: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
pub const PLAIN_INFO_C: [u8; 4] = [0x64, 0x96, 0x32, 0];
pub const PLAIN_INFO_D: [u8; 4] = [0xB4, 0xDC, 0x64, 0xFF];
pub const TINT_KEY: f32 = 80.0;
pub const TINT_WHITE_KEY: f32 = 128.0;
pub const TINT_KEY_ALT: f32 = 130.0;
pub const TINT_INFO: f32 = 180.0;

pub const HEART_FULL: &str = "heart.bti";
pub const HEART_EMPTY: &str = "heart_04_2.bti";
pub const HEART_PARTIAL: [&str; 3] = ["heart_03.bti", "heart_02.bti", "heart_01.bti"];

pub const TRIFORCE_FULL: u8 = 0xFF;
pub const TRIFORCE_COUNT: usize = 8;
pub const PEARL_COUNT: usize = 3;

pub const CARD_GAME_DATA: usize = 0x770;
pub const CARD_PICT_DATA: usize = 0x2000;
pub const CARD_SLOT_SIZE: usize = CARD_GAME_DATA;
pub const CARD_TOTAL_SIZE: usize = CARD_GAME_DATA * SLOT_COUNT;

pub const MSG_TITLE: u32 = 0x1B;
pub const MSG_RETURN: u32 = 0x4E;
pub const MSG_RETURN_TITLE: u32 = 0x4F;
pub const MSG_BROKEN: u32 = 0x1C;
pub const MSG_CORRUPT_SUMMARY: u32 = 0x23;
pub const MSG_COPY_ASK: u32 = 0x1F;
pub const MSG_CANCEL: u32 = 0x1B;
pub const MSG_COPY_TO: u32 = 0x20;
pub const MSG_COPY_OK: u32 = 0x21;
pub const MSG_COPY_DONE: u32 = 0x22;
pub const MSG_ERASE_OK: u32 = 0x1D;
pub const MSG_ERASE_DONE: u32 = 0x1E;
pub const MSG_ERASE_FAIL: u32 = 0x26;
pub const MSG_COPY_FAIL: u32 = 0x27;
pub const MSG_CONFIRM_DONE: u32 = 0x25;
pub const MSG_COPY_DEST_BUSY: u32 = 0x24;
pub const MSG_EXTRA_CONFIRM: u32 = 0x50;
pub const MSG_EXTRA_BACK: u32 = 0x4E;
pub const MSG_EXTRA_DONE: u32 = 0x51;
pub const MSG_EXTRA_FAIL: u32 = 0x3D;

pub const SE_MENU_OK: u32 = 0x0831;
pub const SE_MENU_CANCEL: u32 = 0x0832;
pub const SE_MSEL_CURSOR: u32 = 0x0830;
pub const SE_MSEL_ENTER: u32 = 0x0833;
pub const SE_MENU_DATA_START: u32 = 0x089A;
pub const SE_MENU_DATA_END: u32 = 0x089B;
pub const SE_ALERT_DATA: u32 = 0x08BE;
pub const SE_SAVE_FINISH: u32 = 0x08BD;

pub const proc_data_select: u8 = 0;
pub const proc_select_title: u8 = 1;
pub const proc_data_select_pane_move: u8 = 2;
pub const proc_menu_select: u8 = 3;
pub const proc_to_copy: u8 = 4;
pub const proc_to_erase: u8 = 5;
pub const proc_to_back: u8 = 6;
pub const proc_copy_data_to_select: u8 = 7;
pub const proc_copy_data_sel_anime: u8 = 8;
pub const proc_copy_to_sel_back: u8 = 9;
pub const proc_copy_to_sel_pane_move: u8 = 0xA;
pub const proc_yes_no_select: u8 = 0xB;
pub const proc_cmd_exec_pane_move0: u8 = 0xC;
pub const proc_cmd_exec_pane_move2: u8 = 0xD;
pub const proc_command_exec: u8 = 0xE;
pub const proc_data_erase_wait: u8 = 0xF;
pub const proc_data_copy_wait: u8 = 0x10;
pub const proc_copy_pane_move_ok: u8 = 0x11;
pub const proc_copy_pane_move_ok2: u8 = 0x12;
pub const proc_erase_pane_move_ok: u8 = 0x13;
pub const proc_erase_pane_move_ok2: u8 = 0x14;
pub const proc_error_msg_pane_move: u8 = 0x15;
pub const proc_error_msg_pane_move2: u8 = 0x16;
pub const proc_back_dat_sel_wait: u8 = 0x17;
pub const proc_back_dat_sel_wait2: u8 = 0x18;
pub const proc_back_dat_sel_pane_move: u8 = 0x19;
pub const proc_data_select_ex: u8 = 0x1A;
pub const proc_ex_save_pane_move: u8 = 0x1B;
pub const proc_yes_no_select_ex: u8 = 0x1C;
pub const proc_ex_save_pane_move0: u8 = 0x1D;
pub const proc_ex_save_pane_move1: u8 = 0x1E;
pub const proc_ex_data_save: u8 = 0x1F;
pub const proc_ex_data_save_wait: u8 = 0x20;
pub const proc_ex_back_dat_sel_pane_move: u8 = 0x21;
pub const proc_ex_data_save_pane_move_ok: u8 = 0x22;
pub const proc_ex_data_save_pane_move_ok2: u8 = 0x23;
pub const proc_ex_error_msg_pane_move: u8 = 0x24;
pub const proc_ex_error_msg_pane_move2: u8 = 0x25;
pub const proc_next_mode_wait: u8 = 0x26;

pub const PROC_COUNT: usize = 0x27;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Proc {
    DataSelect,
    SelectTitle,
    DataSelectPaneMove,
    MenuSelect,
    ToCopy,
    ToErase,
    ToBack,
    CopyDataToSelect,
    CopyDataSelAnime,
    CopyToSelBack,
    CopyToSelPaneMove,
    YesNoSelect,
    CmdExecPaneMove0,
    CmdExecPaneMove2,
    CommandExec,
    DataEraseWait,
    DataCopyWait,
    CopyPaneMoveOk,
    CopyPaneMoveOk2,
    ErasePaneMoveOk,
    ErasePaneMoveOk2,
    ErrorMsgPaneMove,
    ErrorMsgPaneMove2,
    BackDatSelWait,
    BackDatSelWait2,
    BackDatSelPaneMove,
    DataSelectEx,
    ExSavePaneMove,
    YesNoSelectEx,
    ExSavePaneMove0,
    ExSavePaneMove1,
    ExDataSave,
    ExDataSaveWait,
    ExBackDatSelPaneMove,
    ExDataSavePaneMoveOk,
    ExDataSavePaneMoveOk2,
    ExErrorMsgPaneMove,
    ExErrorMsgPaneMove2,
    NextModeWait,
}

pub fn dispatch(proc: u8) -> Proc {
    match proc {
        0 => Proc::DataSelect,
        1 => Proc::SelectTitle,
        2 => Proc::DataSelectPaneMove,
        3 => Proc::MenuSelect,
        4 => Proc::ToCopy,
        5 => Proc::ToErase,
        6 => Proc::ToBack,
        7 => Proc::CopyDataToSelect,
        8 => Proc::CopyDataSelAnime,
        9 => Proc::CopyToSelBack,
        10 => Proc::CopyToSelPaneMove,
        11 => Proc::YesNoSelect,
        12 => Proc::CmdExecPaneMove0,
        13 => Proc::CmdExecPaneMove2,
        14 => Proc::CommandExec,
        15 => Proc::DataEraseWait,
        16 => Proc::DataCopyWait,
        17 => Proc::CopyPaneMoveOk,
        18 => Proc::CopyPaneMoveOk2,
        19 => Proc::ErasePaneMoveOk,
        20 => Proc::ErasePaneMoveOk2,
        21 => Proc::ErrorMsgPaneMove,
        22 => Proc::ErrorMsgPaneMove2,
        23 => Proc::BackDatSelWait,
        24 => Proc::BackDatSelWait2,
        25 => Proc::BackDatSelPaneMove,
        26 => Proc::DataSelectEx,
        27 => Proc::ExSavePaneMove,
        28 => Proc::YesNoSelectEx,
        29 => Proc::ExSavePaneMove0,
        30 => Proc::ExSavePaneMove1,
        31 => Proc::ExDataSave,
        32 => Proc::ExDataSaveWait,
        33 => Proc::ExBackDatSelPaneMove,
        34 => Proc::ExDataSavePaneMoveOk,
        35 => Proc::ExDataSavePaneMoveOk2,
        36 => Proc::ExErrorMsgPaneMove,
        37 => Proc::ExErrorMsgPaneMove2,
        _ => Proc::NextModeWait,
    }
}

pub fn proc_name(proc: u8) -> &'static str {
    match dispatch(proc) {
        Proc::DataSelect => "data_select",
        Proc::SelectTitle => "select_title_anime",
        Proc::DataSelectPaneMove => "data_select_pane_move",
        Proc::MenuSelect => "menu_select",
        Proc::ToCopy => "to_copy_pane_move",
        Proc::ToErase => "to_erase_pane_move",
        Proc::ToBack => "to_back_pane_move",
        Proc::CopyDataToSelect => "copy_data_to_select",
        Proc::CopyDataSelAnime => "copy_data_sel_anime",
        Proc::CopyToSelBack => "copy_to_sel_back",
        Proc::CopyToSelPaneMove => "copy_to_sel_pane_move",
        Proc::YesNoSelect => "yes_no_select",
        Proc::CmdExecPaneMove0 => "cmd_exec_pane_move0",
        Proc::CmdExecPaneMove2 => "cmd_exec_pane_move2",
        Proc::CommandExec => "command_exec",
        Proc::DataEraseWait => "data_erase_wait",
        Proc::DataCopyWait => "data_copy_wait",
        Proc::CopyPaneMoveOk => "copy_pane_move_ok",
        Proc::CopyPaneMoveOk2 => "copy_pane_move_ok2",
        Proc::ErasePaneMoveOk => "erase_pane_move_ok",
        Proc::ErasePaneMoveOk2 => "erase_pane_move_ok2",
        Proc::ErrorMsgPaneMove => "error_msg_pane_move",
        Proc::ErrorMsgPaneMove2 => "error_msg_pane_move2",
        Proc::BackDatSelWait => "back_dat_sel_wait",
        Proc::BackDatSelWait2 => "back_dat_sel_wait2",
        Proc::BackDatSelPaneMove => "back_dat_sel_pane_move",
        Proc::DataSelectEx => "data_select_ex",
        Proc::ExSavePaneMove => "ex_save_pane_move",
        Proc::YesNoSelectEx => "yes_no_select_ex",
        Proc::ExSavePaneMove0 => "ex_save_pane_move0",
        Proc::ExSavePaneMove1 => "ex_save_pane_move1",
        Proc::ExDataSave => "ex_data_save",
        Proc::ExDataSaveWait => "ex_data_save_wait",
        Proc::ExBackDatSelPaneMove => "ex_back_dat_sel_pane_move",
        Proc::ExDataSavePaneMoveOk => "ex_data_save_pane_move_ok",
        Proc::ExDataSavePaneMoveOk2 => "ex_data_save_pane_move_ok2",
        Proc::ExErrorMsgPaneMove => "ex_error_msg_pane_move",
        Proc::ExErrorMsgPaneMove2 => "ex_error_msg_pane_move2",
        Proc::NextModeWait => "next_mode_wait",
    }
}

pub fn half_life(health: u16) -> (u16, u16) {
    let hearts = health / 4;
    let rem = health % 4;
    if rem != 0 { (hearts + 1, rem) } else { (hearts, 0) }
}

pub fn max_hearts(health: u16) -> u16 {
    health / 4
}

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SlotProgress {
    pub triforce: u8,
    pub symbol: u8,
}

#[derive(Clone, Debug)]
pub struct SlotPanes {
    pub key: [Vec<Pane>; 3],
    pub info: [Vec<Pane>; 3],
    pub record: [Pane; 2],
    pub collect: [Pane; 12],
    pub pearl: [Pane; 3],
    pub pearl_shadow: [Pane; 3],
    pub triforce_bar: Pane,
    pub shard: [Pane; 9],
    pub heart: [Pane; 0x14],
    pub heart_shadow: [Pane; 0x14],
    pub slot_mark: [Pane; 3],
    pub slot_mark_key: [Pane; 3],
}

impl Default for SlotPanes {
    fn default() -> Self {
        Self {
            key: std::array::from_fn(|i| match i {
                0 => SLOT_KEY_TAGS.iter().map(|t| Pane::new(leak(t))).collect(),
                1 => SLOT_KEY2_TAGS.iter().map(|t| Pane::new(leak(t))).collect(),
                _ => SLOT_KEY3_TAGS.iter().map(|t| Pane::new(leak(t))).collect(),
            }),
            info: std::array::from_fn(|i| match i {
                0 => info_tags(REC_INFO_1_TAGS, NAME_TAGS[0], DAY_TAGS[0], "1k00", "1k01"),
                1 => info_tags(REC_INFO_2_TAGS, NAME_TAGS[1], DAY_TAGS[1], "2k00", "2k01"),
                _ => info_tags(REC_INFO_3_TAGS, NAME_TAGS[2], DAY_TAGS[2], "3k00", "3k01"),
            }),
            record: [Pane::new("rc00"), Pane::new("rck0")],
            collect: std::array::from_fn(|i| Pane::new(COLLEC_TAGS[i])),
            pearl: std::array::from_fn(|i| Pane::new(PEARL_TAGS[i])),
            pearl_shadow: std::array::from_fn(|i| Pane::new(PEARL_SHADOW_TAGS[i])),
            triforce_bar: Pane::new("trib"),
            shard: std::array::from_fn(|i| Pane::new(TRIFORCE_TAGS[i])),
            heart: std::array::from_fn(|i| Pane::new(HEART_TAGS[i])),
            heart_shadow: std::array::from_fn(|i| Pane::new(HEART_SHADOW_TAGS[i])),
            slot_mark: [
                Pane::new("e1"),
                Pane::new("e2"),
                Pane::new("e3"),
            ],
            slot_mark_key: [Pane::new("ek1"), Pane::new("ek2"), Pane::new("ek3")],
        }
    }
}

fn leak(text: &str) -> &'static str {
    Box::leak(text.to_string().into_boxed_str())
}

fn info_tags(
    rec: [&str; 0x12],
    name: &'static str,
    day: &'static str,
    head: &'static str,
    tail: &'static str,
) -> Vec<Pane> {
    let mut out: Vec<Pane> = Vec::with_capacity(0x1D);
    out.push(Pane::new(head));
    out.push(Pane::new(name));
    out.push(Pane::new(day));
    for index in 0..0x12 {
        out.push(Pane::new(leak(rec[index])));
    }
    for offset in 0..8 {
        let mut tag = String::from(tail);
        tag.push((b'0' + offset) as char);
        out.push(Pane::new(leak(&tag)));
    }
    out
}

#[derive(Clone, Debug)]
pub struct PaneGroups {
    pub title_key: Pane,
    pub title_label: Pane,
    pub title_text: [Pane; 2],
    pub message_key: [Pane; 2],
    pub message_text: Pane,
    pub record_cursor: [Pane; 5],
    pub menu_cursor: Pane,
    pub yes_key: [Pane; 2],
    pub yes_label: Pane,
    pub yes_text: Pane,
    pub no_key: [Pane; 2],
    pub no_label: Pane,
    pub no_text: Pane,
    pub start_key: [Pane; 2],
    pub start_label: Pane,
    pub copy_key: [Pane; 2],
    pub copy_pic: [Pane; 2],
    pub copy_label: Pane,
    pub erase_key: [Pane; 2],
    pub erase_pic: [Pane; 2],
    pub erase_label: Pane,
    pub back_key: [Pane; 2],
    pub back_pic: Pane,
    pub back_label: Pane,
    pub back_text: Pane,
}

impl Default for PaneGroups {
    fn default() -> Self {
        Self {
            title_key: Pane::new("exk0"),
            title_label: Pane::new("ex00"),
            title_text: [Pane::new("expl"), Pane::new("exp2")],
            message_key: [Pane::new("ntk0"), Pane::new("nt00")],
            message_text: Pane::new("mes1"),
            record_cursor: [
                Pane::new("2cu1"),
                Pane::new("2cu2"),
                Pane::new("2cu3"),
                Pane::new("2cu4"),
                Pane::new("2cu5"),
            ],
            menu_cursor: Pane::new("1cu1"),
            yes_key: [Pane::new("ysk0"), Pane::new("ysk1")],
            yes_label: Pane::new("ys00"),
            yes_text: Pane::new("yes"),
            no_key: [Pane::new("nok0"), Pane::new("nok1")],
            no_label: Pane::new("no00"),
            no_text: Pane::new("no"),
            start_key: [Pane::new("sk00"), Pane::new("sk01")],
            start_label: Pane::new("s000"),
            copy_key: [Pane::new("cpk0"), Pane::new("cpk1")],
            copy_pic: [Pane::new("cp00"), Pane::new("cp01")],
            copy_label: Pane::new("copy"),
            erase_key: [Pane::new("dlk0"), Pane::new("dlk1")],
            erase_pic: [Pane::new("dl00"), Pane::new("dl01")],
            erase_label: Pane::new("dlle"),
            back_key: [Pane::new("rek0"), Pane::new("rek1")],
            back_pic: Pane::new("re00"),
            back_label: Pane::new("retu"),
            back_text: Pane::new("stat"),
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct FileSelectPanes {
    pub slots: SlotPanes,
    pub groups: PaneGroups,
}

impl Default for FileSelectPanes {
    fn default() -> Self {
        Self {
            slots: SlotPanes::default(),
            groups: PaneGroups::default(),
        }
    }
}

impl FileSelectPanes {
    pub fn bind(&mut self, screen: &Screen) {
        for pane in &mut self.slots.key[0][..2] {
            bind(pane, screen);
        }
        for index in 2..12 {
            let mut tag = String::from("a00");
            tag.push((b'0' + (index as u8 - 2)) as char);
            bind_fixed(&mut self.slots.key[0][index], screen, &leak(&tag));
        }
        for index in 12..16 {
            let mut tag = String::from("ak0");
            tag.push((b'0' + (index as u8 - 12)) as char);
            bind_fixed(&mut self.slots.key[0][index], screen, &leak(&tag));
        }
        for index in 2..12 {
            let mut tag = String::from("b00");
            tag.push((b'0' + (index as u8 - 2)) as char);
            bind_fixed(&mut self.slots.key[1][index], screen, &leak(&tag));
        }
        for index in 12..16 {
            let mut tag = String::from("bk0");
            tag.push((b'0' + (index as u8 - 12)) as char);
            bind_fixed(&mut self.slots.key[1][index], screen, &leak(&tag));
        }
        for index in 2..12 {
            let mut tag = String::from("c00");
            tag.push((b'0' + (index as u8 - 2)) as char);
            bind_fixed(&mut self.slots.key[2][index], screen, &leak(&tag));
        }
        for index in 12..16 {
            let mut tag = String::from("ck0");
            tag.push((b'0' + (index as u8 - 12)) as char);
            bind_fixed(&mut self.slots.key[2][index], screen, &leak(&tag));
        }
        for index in 0..2 {
            bind(&mut self.slots.key[0][index], screen);
            bind(&mut self.slots.key[1][index], screen);
            bind(&mut self.slots.key[2][index], screen);
        }
        for slot in 0..3 {
            for pane in self.slots.info[slot].iter_mut() {
                bind(pane, screen);
            }
            bind(&mut self.slots.slot_mark[slot], screen);
            bind(&mut self.slots.slot_mark_key[slot], screen);
        }
        for pane in &mut self.slots.record {
            bind(pane, screen);
        }
        for pane in &mut self.slots.collect {
            bind(pane, screen);
        }
        for pane in &mut self.slots.pearl {
            bind(pane, screen);
        }
        for pane in &mut self.slots.pearl_shadow {
            bind(pane, screen);
        }
        bind(&mut self.slots.triforce_bar, screen);
        for pane in &mut self.slots.shard {
            bind(pane, screen);
        }
        for pane in &mut self.slots.heart {
            bind(pane, screen);
        }
        for pane in &mut self.slots.heart_shadow {
            bind(pane, screen);
        }
        bind(&mut self.groups.title_key, screen);
        bind(&mut self.groups.title_label, screen);
        bind(&mut self.groups.title_text[0], screen);
        bind(&mut self.groups.title_text[1], screen);
        bind(&mut self.groups.message_key[0], screen);
        bind(&mut self.groups.message_key[1], screen);
        bind(&mut self.groups.message_text, screen);
        for pane in &mut self.groups.record_cursor {
            bind(pane, screen);
        }
        bind(&mut self.groups.menu_cursor, screen);
        for pane in &mut self.groups.yes_key {
            bind(pane, screen);
        }
        bind(&mut self.groups.yes_label, screen);
        bind(&mut self.groups.yes_text, screen);
        for pane in &mut self.groups.no_key {
            bind(pane, screen);
        }
        bind(&mut self.groups.no_label, screen);
        bind(&mut self.groups.no_text, screen);
        for pane in &mut self.groups.start_key {
            bind(pane, screen);
        }
        bind(&mut self.groups.start_label, screen);
        for pane in &mut self.groups.copy_key {
            bind(pane, screen);
        }
        for pane in &mut self.groups.copy_pic {
            bind(pane, screen);
        }
        bind(&mut self.groups.copy_label, screen);
        for pane in &mut self.groups.erase_key {
            bind(pane, screen);
        }
        for pane in &mut self.groups.erase_pic {
            bind(pane, screen);
        }
        bind(&mut self.groups.erase_label, screen);
        for pane in &mut self.groups.back_key {
            bind(pane, screen);
        }
        bind(&mut self.groups.back_pic, screen);
        bind(&mut self.groups.back_label, screen);
        bind(&mut self.groups.back_text, screen);
    }
}

fn bind(pane: &mut Pane, screen: &Screen) {
    if let Some(layout) = screen.find(pane.tag) {
        pane.parts_set_file(layout);
    }
}

fn bind_fixed(pane: &mut Pane, screen: &Screen, tag: &str) {
    if let Some(layout) = screen.find(tag) {
        pane.parts_set_file(layout);
    }
}

#[derive(Clone, Debug)]
pub struct SlotSummary {
    pub name: String,
    pub date: String,
    pub hearts: u16,
    pub heart_rem: u16,
    pub max_hearts: u16,
    pub triforce: u8,
    pub symbol: u8,
    pub new_game: bool,
    pub broken: bool,
    pub extra: bool,
    pub extra_link: bool,
    pub heart_shown: [bool; 0x14],
    pub heart_texture: [Option<&'static str>; 0x14],
    pub pearl_shown: [bool; PEARL_COUNT],
    pub pearl_shadow_shown: [bool; PEARL_COUNT],
    pub shard_shown: [bool; TRIFORCE_COUNT + 1],
    pub collect_shown: [bool; 12],
    pub triforce_bar: bool,
}

impl Default for SlotSummary {
    fn default() -> Self {
        Self {
            name: String::new(),
            date: String::new(),
            hearts: 0,
            heart_rem: 0,
            max_hearts: 0,
            triforce: 0,
            symbol: 0,
            new_game: false,
            broken: false,
            extra: false,
            extra_link: false,
            heart_shown: [false; 0x14],
            heart_texture: [None; 0x14],
            pearl_shown: [false; PEARL_COUNT],
            pearl_shadow_shown: [false; PEARL_COUNT],
            shard_shown: [false; TRIFORCE_COUNT + 1],
            collect_shown: [false; 12],
            triforce_bar: false,
        }
    }
}

impl SlotSummary {
    pub fn from_save(save: &SaveGame) -> Self {
        Self::from_parts(save, 0, 0, 0)
    }

    pub fn from_parts(save: &SaveGame, triforce: u8, symbol: u8, extra_flags: u8) -> Self {
        let mut out = Self {
            triforce,
            symbol,
            extra_link: ((extra_flags / 16) & 1) == 1,
            ..Default::default()
        };
        let life = u16::from(save.health.units);
        let max = u16::from(save.health.max);
        let (hearts, rem) = half_life(life);
        out.hearts = hearts;
        out.heart_rem = rem;
        out.max_hearts = max_hearts(max);
        for index in 0..0x14usize {
            if index < out.max_hearts as usize {
                out.heart_shown[index] = true;
                if index < hearts as usize {
                    if rem != 0 && index == hearts as usize - 1 {
                        out.heart_texture[index] = Some(HEART_PARTIAL[(rem - 1) as usize]);
                    } else {
                        out.heart_texture[index] = Some(HEART_FULL);
                    }
                } else {
                    out.heart_texture[index] = Some(HEART_EMPTY);
                }
            } else {
                out.heart_shown[index] = false;
            }
        }
        let mut pearl_bit = 1u8;
        for index in 0..PEARL_COUNT {
            let on = symbol & pearl_bit != 0;
            out.pearl_shown[index] = on;
            out.pearl_shadow_shown[index] = on;
            pearl_bit <<= 1;
        }
        out
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PadInput {
    pub trig_a: bool,
    pub trig_b: bool,
    pub trig_start: bool,
    pub trig_x: bool,
    pub trig_y: bool,
    pub trig_l: bool,
    pub trig_r: bool,
    pub trig_z: bool,
    pub trig_z3: bool,
    pub trig_a2: bool,
    pub trig_up: bool,
    pub trig_down: bool,
    pub trig_left: bool,
    pub trig_right: bool,
    pub hold_up: bool,
    pub hold_down: bool,
    pub hold_b: bool,
    pub hold_a: bool,
    pub stick_value: f32,
    pub stick_angle: i16,
}

impl From<&SavePadInput> for PadInput {
    fn from(value: &SavePadInput) -> Self {
        Self {
            trig_a: value.trig_a,
            trig_b: value.trig_b,
            trig_start: value.trig_start,
            trig_x: value.trig_x,
            trig_y: value.trig_y,
            trig_l: value.trig_l,
            trig_r: value.trig_r,
            trig_z: value.trig_z,
            trig_z3: false,
            trig_a2: false,
            trig_up: false,
            trig_down: false,
            trig_left: false,
            trig_right: false,
            hold_up: false,
            hold_down: false,
            hold_b: false,
            hold_a: false,
            stick_value: value.stick_value,
            stick_angle: value.stick_angle,
        }
    }
}

impl PadInput {
    pub fn any_confirm(&self) -> bool {
        self.trig_a
            || self.trig_b
            || self.trig_x
            || self.trig_y
            || self.trig_l
            || self.trig_r
            || self.trig_z
            || self.trig_start
    }
}

#[derive(Resource, Clone, Debug)]
pub struct FileSelect {
    pub screen: Option<Screen>,
    pub panes: FileSelectPanes,
    pub stick: StickControl,
    pub stick2: StickControl,
    pub data_new: [u8; SLOT_COUNT],
    pub data_broken: [u8; SLOT_COUNT],
    pub data_extra: [u8; SLOT_COUNT],
    pub data_extra_link: [u8; SLOT_COUNT],
    pub flag_3920: u8,
    pub flag_3921: u8,
    pub select_num: u8,
    pub title_dir: i8,
    pub flag_3924: u8,
    pub flag_3925: u8,
    pub flag_3926: u8,
    pub menu_pos: u8,
    pub op: u8,
    pub flag_3929: u8,
    pub select_end: u8,
    pub flag_392d: u8,
    pub flag_392e: u8,
    pub err_type: u8,
    pub proc: u8,
    pub timer: i16,
    pub timer2: i16,
    pub flag_3934: [u8; 2],
    pub icon_mode: u8,
    pub use_type: u8,
    pub flag_3941: u8,
    pub flag_3942: [u8; 2],
    pub slide: f32,
    pub title_dy: f32,
    pub progress: [SlotProgress; SLOT_COUNT],
    pub summary: [SlotSummary; SLOT_COUNT],
    pub text_name: [String; SLOT_COUNT],
    pub text_date: [String; SLOT_COUNT],
    pub message_a: String,
    pub message_b: String,
    pub message_c: String,
    pub extra_text_y: f32,
    pub colors: Colors,
    pub sounds: Vec<u32>,
    pub slot_picked: Option<usize>,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct Colors {
    pub slot_black: [Color; 11],
    pub slot_white: [Color; 11],
    pub info_black: [Color; 18],
    pub info_white: [Color; 18],
    pub copy_black: [Color; 2],
    pub copy_white: [Color; 2],
    pub erase_black: [Color; 2],
    pub erase_white: [Color; 2],
}

impl Default for FileSelect {
    fn default() -> Self {
        Self {
            screen: None,
            panes: FileSelectPanes::default(),
            stick: StickControl::new(5, 2, 3, 2),
            stick2: StickControl::new(3, 1, 3, 2),
            data_new: [0; SLOT_COUNT],
            data_broken: [0; SLOT_COUNT],
            data_extra: [0; SLOT_COUNT],
            data_extra_link: [0; SLOT_COUNT],
            flag_3920: 0,
            flag_3921: 0,
            select_num: 0,
            title_dir: 0,
            flag_3924: 0,
            flag_3925: 0,
            flag_3926: 0,
            menu_pos: MENU_POS_START,
            op: OP_NONE,
            flag_3929: 0,
            select_end: SELECT_END_NONE,
            flag_392d: 0,
            flag_392e: 0,
            err_type: 0,
            proc: proc_data_select,
            timer: 0,
            timer2: 0,
            flag_3934: [0; 2],
            icon_mode: ICON_MODE_DEFAULT,
            use_type: MODE_NORMAL,
            flag_3941: 0,
            flag_3942: [0; 2],
            slide: 0.0,
            title_dy: 0.0,
            progress: Default::default(),
            summary: Default::default(),
            text_name: Default::default(),
            text_date: Default::default(),
            message_a: String::new(),
            message_b: String::new(),
            message_c: String::new(),
            extra_text_y: 0.0,
            colors: Colors::default(),
            sounds: Vec::new(),
            slot_picked: None,
        }
    }
}

impl FileSelect {
    pub fn message_text_of(&self, index: usize) -> &str {
        match index {
            0 => &self.message_a,
            1 => &self.message_b,
            _ => &self.message_c,
        }
    }

    pub fn message_set(&mut self, index: usize, msg_no: u32) {
        let text = format!("\u{1}{msg_no}");
        match index {
            0 => {
                self.message_a = text.clone();
                self.panes.groups.title_text[0].set_string(&text);
            }
            1 => {
                self.message_b = text.clone();
                self.panes.groups.title_text[1].set_string(&text);
            }
            _ => {
                self.message_c = text.clone();
                self.panes.groups.message_text.set_string(&text);
            }
        }
    }

    pub fn message_clear(&mut self, index: usize) {
        match index {
            0 => {
                self.message_a.clear();
                self.panes.groups.title_text[0].set_string("");
            }
            1 => {
                self.message_b.clear();
                self.panes.groups.title_text[1].set_string("");
            }
            _ => {
                self.message_c.clear();
                self.panes.groups.message_text.set_string("");
            }
        }
    }

    pub fn get_icon_mode(&self) -> u8 {
        self.icon_mode
    }

    pub fn set_icon_mode(&mut self, mode: u8) {
        self.icon_mode = mode;
    }

    pub fn set_use_type(&mut self, use_type: u8) {
        self.use_type = use_type;
    }

    pub fn get_err_type(&self) -> u8 {
        self.err_type
    }

    pub fn set_slot_progress(&mut self, index: usize, progress: SlotProgress) {
        if index < SLOT_COUNT {
            self.progress[index] = progress;
        }
    }

    pub fn get_select_num(&self) -> u8 {
        self.select_num
    }

    pub fn is_data_extra(&self, idx: usize) -> bool {
        self.data_extra.get(idx).copied().unwrap_or(0) != 0
    }

    pub fn is_data_new(&self, idx: usize) -> bool {
        self.data_new.get(idx).copied().unwrap_or(0) != 0
    }

    pub fn is_data_broken(&self, idx: usize) -> bool {
        self.data_broken.get(idx).copied().unwrap_or(0) != 0
    }

    pub fn is_select_end(&self) -> u8 {
        self.select_end
    }

    pub fn sound(&mut self, se: u32) {
        self.sounds.push(se);
    }

    pub fn create(&mut self, screen: Screen) {
        self.screen = Some(screen);
        for index in 0..SLOT_COUNT {
            self.data_new[index] = 0;
            self.data_broken[index] = 0;
            self.data_extra[index] = 0;
            self.data_extra_link[index] = 0;
        }
        self.select_num = 0;
        self.screen_set();
        self.display_init();
        self.pane_trans_init();
    }

    pub fn delete(&mut self) {
        self.screen = None;
        self.panes = FileSelectPanes::default();
        self.stick = StickControl::new(5, 2, 3, 2);
        self.stick2 = StickControl::new(3, 1, 3, 2);
    }

    pub fn initial(&mut self) {
        self.display_init();
        self.pane_trans_init();
    }

    pub fn screen_set(&mut self) {
        let Some(screen) = self.screen.clone() else {
            return;
        };
        self.panes.bind(&screen);

        let fill_name = "A".repeat(TEXT_BUFFER_NAME - 1);
        for index in 0..2 {
            self.panes.groups.title_text[index].set_string(&fill_name);
            self.message_clear(index);
        }

        let fill_info = "A".repeat(TEXT_BUFFER_INFO - 1);
        self.panes.groups.message_text.set_string(&fill_info);
        self.message_clear(2);

        let fill_slot = "A".repeat(crate::ui_menu_save::TEXT_BUFFER_A - 1);
        for slot in 0..3 {
            self.panes.slots.info[slot][1].set_string(&fill_slot);
            self.text_name[slot].clear();
            self.text_date[slot].clear();
        }

        for index in 0..2 {
            self.colors.copy_black[index] = self.panes.groups.copy_pic[index].black;
            self.colors.copy_white[index] = self.panes.groups.copy_pic[index].white;
            self.colors.erase_black[index] = self.panes.groups.erase_pic[index].black;
            self.colors.erase_white[index] = self.panes.groups.erase_pic[index].white;
        }
    }

    pub fn pane_trans_init(&mut self) {
        self.timer = 0;
        self.pane_trance_title(
            self.timer,
            IDX_08,
            IDX_48 as f32,
            0.0,
            IDX_07,
            0,
        );
        self.pane_trance_rec_tlt(0, self.timer, IDX_0c, IDX_18 as f32, 0.0, 0.0, 0.0, 1, 0);
        self.pane_trance_rec_tlt(1, self.timer, IDX_0d, IDX_18 as f32, 0.0, 0.0, 0.0, 1, 0);
        self.pane_trance_rec_tlt(2, self.timer, IDX_0e, IDX_18 as f32, 0.0, 0.0, 0.0, 1, 0);
        self.pane_trance_rec_info(0, self.timer, IDX_11, 0.0, IDX_1a as f32, 0.0, 0.0, 1, 0);
        self.pane_trance_rec_info(1, self.timer, IDX_12, 0.0, IDX_1c as f32, 0.0, 0.0, 1, 0);
        self.pane_trance_rec_info(2, self.timer, IDX_13, 0.0, IDX_1c as f32, 0.0, 0.0, 1, 0);
        self.pane_trance_message_base(
            self.timer,
            IDX_32,
            IDX_44 as f32,
            0.0,
            0.0,
            0.0,
            1,
            0,
        );
        self.pane_trance_rec_base(
            self.timer,
            IDX_2f,
            IDX_44 as f32,
            0.0,
            0.0,
            0.0,
            1,
            0,
        );
        self.menu_pane_move_set();
        self.yes_no_pane_move_set();
    }

    pub fn menu_pane_move_set(&mut self) {
        self.pane_trance_menu_item(self.timer, IDX_34, IDX_42 as f32, 0.0, 1, 0, MenuItem::Start);
        self.pane_trance_menu_item(self.timer, IDX_34, IDX_42 as f32, 0.0, 1, 0, MenuItem::Copy);
        self.pane_trance_menu_item(self.timer, IDX_34, IDX_42 as f32, 0.0, 1, 0, MenuItem::Erase);
        self.pane_trance_menu_item(self.timer, IDX_34, IDX_42 as f32, 0.0, 1, 0, MenuItem::Back);
    }

    pub fn yes_no_pane_move_set(&mut self) {
        self.pane_trance_menu_item(self.timer, IDX_34, IDX_42 as f32, 0.0, 1, 0, MenuItem::Yes);
        self.pane_trance_menu_item(self.timer, IDX_34, IDX_42 as f32, 0.0, 1, 0, MenuItem::No);
    }

    pub fn display_init(&mut self) {
        self.flag_3941 = 0;
        self.icon_mode = ICON_MODE_WAIT;
        self.flag_392d = 0;
        self.select_end = SELECT_END_NONE;
        self.flag_392e = 0;
        self.message_clear(2);
        self.timer2 = IDX_4a;
        self.slide = 0.0;
        self.rec_cursor_alpha_init();
        self.rec_cursor_move();
        self.panes.groups.record_cursor[0].show();
        self.menu_pos = MENU_POS_START;
        self.menu_cursor_alpha_init();

        let title = if self.use_type == MODE_NORMAL {
            MSG_TITLE
        } else {
            MSG_RETURN
        };
        self.message_set(self.flag_392d as usize, title);
        self.panes.groups.title_text[1].set_now_alpha(0.0);

        self.op = OP_NONE;
        if self.use_type == MODE_NORMAL {
            self.proc = proc_data_select;
        } else {
            self.proc = proc_data_select_ex;
        }

        self.panes.slots.shard[8].hide();
    }

    pub fn set_save_data(&mut self, card: &MemCard) {
        for index in 0..SLOT_COUNT {
            let name_pane = &mut self.panes.slots.info[index][1];
            name_pane.pane_scale_xy(1.0);
            let base = name_pane.get_font_size();
            name_pane.set_font_size(base.0, base.1);
            name_pane.pane_trans(0.0, 0.0);

            let save = card.slots[index].clone();
            let broken = card.slot_state[index] == SlotState::Corrupt;
            match save {
                _ if broken => {
                    self.change_broken_color(index);
                    self.panes.slots.info[index][1].pane_scale_xy(IDX_54);
                    self.panes.slots.info[index][1]
                        .set_font_size(base.0 * 0.9, base.1 * IDX_54);
                    let trans_x = self.panes.slots.info[index][1].pos_top_left_orig.x
                        - self.panes.slots.info[index][1].pos_top_left.x;
                    let trans_y = self.panes.slots.info[index][1].pos_top_left_orig.y
                        - self.panes.slots.info[index][1].pos_top_left.y
                        + IDX_58;
                    self.panes.slots.info[index][1].pane_trans(trans_x, trans_y);
                    self.text_name[index] = SLOT_BROKEN_NAME.to_string();
                    self.text_date[index].clear();
                    self.data_new[index] = 0;
                    self.data_broken[index] = 1;
                    self.summary[index] = SlotSummary {
                        name: SLOT_BROKEN_NAME.to_string(),
                        broken: true,
                        ..Default::default()
                    };
                }
                None => {
                    self.panes.slots.info[index][1].pane_scale_xy(IDX_54);
                    self.panes.slots.info[index][1].set_font_size(
                        base.0 * IDX_54,
                        base.1 * IDX_54,
                    );
                    let trans_x = self.panes.slots.info[index][1].pos_top_left_orig.x
                        - self.panes.slots.info[index][1].pos_top_left.x;
                    let trans_y = self.panes.slots.info[index][1].pos_top_left_orig.y
                        - self.panes.slots.info[index][1].pos_top_left.y
                        + IDX_58;
                    self.panes.slots.info[index][1].pane_trans(trans_x, trans_y);
                    self.text_name[index] = SLOT_NEW_NAME.to_string();
                    self.text_date[index].clear();
                    self.data_new[index] = 1;
                    self.data_broken[index] = 0;
                    self.data_extra[index] = 0;
                    self.data_extra_link[index] = 0;
                    self.summary[index] = SlotSummary {
                        name: SLOT_NEW_NAME.to_string(),
                        new_game: true,
                        ..Default::default()
                    };
                }
                Some(ref save) if card.test_checksum_game_data(index) => {
                    if card.clear_count[index] != 0 {
                        self.data_extra[index] = 1;
                        self.data_extra_link[index] = (card.owl_link[index] / 16) & 1 ^ 1;
                    } else {
                        self.data_extra[index] = 0;
                        self.data_extra_link[index] = 0;
                    }
                    self.change_extra_color(index);

                    let progress = self.progress[index];
                    let mut summary = SlotSummary::from_parts(save, progress.triforce, progress.symbol, card.owl_link[index]);
                    summary.extra = self.data_extra[index] == 1;
                    summary.name = save.name.clone();
                    summary.date = ticks_to_date(save.tick);
                    summary.new_game = save.name.is_empty();
                    summary.broken = false;
                    self.summary[index] = summary;

                    if save.name.is_empty() {
                        self.panes.slots.info[index][1].pane_scale_xy(IDX_54);
                        self.panes.slots.info[index][1].set_font_size(
                            base.0 * IDX_54,
                            base.1 * IDX_54,
                        );
                        let trans_x = self.panes.slots.info[index][1].pos_top_left_orig.x
                            - self.panes.slots.info[index][1].pos_top_left.x;
                        let trans_y = self.panes.slots.info[index][1].pos_top_left_orig.y
                            - self.panes.slots.info[index][1].pos_top_left.y
                            + IDX_58;
                        self.panes.slots.info[index][1].pane_trans(trans_x, trans_y);
                        self.text_name[index] = SLOT_NEW_NAME.to_string();
                        self.text_date[index].clear();
                        self.data_new[index] = 1;
                    } else {
                        self.text_name[index] = save.name.clone();
                        self.text_date[index] = ticks_to_date(save.tick);
                        self.data_new[index] = 0;
                    }
                    self.data_broken[index] = 0;
                }
                _ => {
                    self.change_broken_color(index);
                    self.panes.slots.info[index][1].pane_scale_xy(IDX_54);
                    self.panes.slots.info[index][1]
                        .set_font_size(base.0 * 0.9, base.1 * IDX_54);
                    let trans_x = self.panes.slots.info[index][1].pos_top_left_orig.x
                        - self.panes.slots.info[index][1].pos_top_left.x;
                    let trans_y = self.panes.slots.info[index][1].pos_top_left_orig.y
                        - self.panes.slots.info[index][1].pos_top_left.y
                        + IDX_58;
                    self.panes.slots.info[index][1].pane_trans(trans_x, trans_y);
                    self.text_name[index] = SLOT_BROKEN_NAME.to_string();
                    self.text_date[index].clear();
                    self.data_new[index] = 0;
                    self.data_broken[index] = 1;
                    self.summary[index] = SlotSummary {
                        name: SLOT_BROKEN_NAME.to_string(),
                        broken: true,
                        ..Default::default()
                    };
                }
            }
            self.panes.slots.info[index][1].set_string(&self.text_name[index].clone());
            self.panes.slots.info[index][2].set_string(&self.text_date[index].clone());
        }
    }

    pub fn open(&mut self) -> bool {
        let temp1 = self.pane_trance_title(
            self.timer - IDX_0b as i16,
            IDX_08,
            IDX_48 as f32,
            0.0,
            IDX_07,
            0,
        );
        let temp2 = self.pane_trance_rec_tlt(0,
            self.timer - IDX_0f as i16,
            IDX_0c,
            IDX_18 as f32,
            0.0,
            0.0,
            0.0,
            1,
            0,
        );
        let temp3 = self.pane_trance_rec_tlt(1,
            self.timer - (IDX_0f as i16 + IDX_10 as i16),
            IDX_0d,
            IDX_18 as f32,
            0.0,
            0.0,
            0.0,
            1,
            0,
        );
        let temp4 = self.pane_trance_rec_tlt(2,
            self.timer - (IDX_0f as i16 + IDX_10 as i16 * 2),
            IDX_0e,
            IDX_18 as f32,
            0.0,
            0.0,
            0.0,
            1,
            0,
        );
        let temp5 = self.pane_trance_rec_info(0,
            self.timer - IDX_14 as i16,
            IDX_11,
            0.0,
            IDX_1a as f32,
            0.0,
            0.0,
            1,
            0,
        );
        let temp6 = self.pane_trance_rec_info(1,
            self.timer - IDX_15 as i16,
            IDX_12,
            0.0,
            IDX_1c as f32,
            0.0,
            0.0,
            1,
            0,
        );
        let temp7 = self.pane_trance_rec_info(2,
            self.timer - IDX_16 as i16,
            IDX_13,
            0.0,
            IDX_1e as f32,
            0.0,
            0.0,
            1,
            0,
        );

        self.timer += 1;

        temp1 && temp2 && temp3 && temp4 && temp5 && temp6 && temp7
    }

    pub fn close(&mut self) -> bool {
        let result = if self.flag_392e != 0 {
            match self.op {
                OP_ERASE => self.close_err_erase(),
                OP_COPY => self.close_err_copy(),
                OP_ERASE_EXTRA => self.close_err_erase(),
                OP_CARD_ERR => self.close_card_err(),
                _ => true,
            }
        } else if self.select_end == SELECT_END_RETURN {
            self.close_back()
        } else {
            self.close_end()
        };

        self.timer += 1;

        result
    }

    pub fn close_end(&mut self) -> bool {
        let temp1 = self.pane_trance_title(self.timer, IDX_08, 0.0, IDX_48 as f32, 0, 1);
        let (temp2, temp3) = self.rec_row_trance_out();
        let temp4 = self.rec_info_pane_move(0.0, 0.0, IDX_44 as f32, 0.0, 0, 0, 1);
        let temp5 = self.menu_pane_move(0.0, IDX_42 as f32, 0, 0, 1);
        temp1 && temp2 && temp3 && temp4 && temp5
    }

    fn rec_row_trance_out(&mut self) -> (bool, bool) {
        match self.select_num {
            0 => {
                let temp2 = self.pane_trance_rec_tlt(0,
                    self.timer,
                    IDX_0c,
                    0.0,
                    0.0,
                    -(IDX_40 as f32),
                    0.0,
                    0,
                    1,
                );
                let temp3 = self.pane_trance_rec_info(0,
                    self.timer,
                    IDX_11,
                    0.0,
                    0.0,
                    -(IDX_40 as f32),
                    0.0,
                    0,
                    1,
                );
                (temp2, temp3)
            }
            1 => {
                let temp2 = self.pane_trance_rec_tlt(1,
                    self.timer,
                    IDX_0d,
                    0.0,
                    -85.0,
                    -(IDX_40 as f32),
                    -85.0,
                    0,
                    1,
                );
                let temp3 = self.pane_trance_rec_info(1,
                    self.timer,
                    IDX_12,
                    0.0,
                    -85.0,
                    -(IDX_40 as f32),
                    -85.0,
                    0,
                    1,
                );
                (temp2, temp3)
            }
            _ => {
                let temp2 = self.pane_trance_rec_tlt(2,
                    self.timer,
                    IDX_0e,
                    0.0,
                    -170.0,
                    -(IDX_40 as f32),
                    -170.0,
                    0,
                    1,
                );
                let temp3 = self.pane_trance_rec_info(1,
                    self.timer,
                    IDX_12,
                    0.0,
                    -170.0,
                    -(IDX_40 as f32),
                    -170.0,
                    0,
                    1,
                );
                (temp2, temp3)
            }
        }
    }

    pub fn close_card_err(&mut self) -> bool {
        let temp1 = self.pane_trance_title(self.timer, IDX_08, 0.0, IDX_48 as f32, 0, 1);
        let (temp2, temp3) = self.rec_row_trance_out();
        let temp4 = self.rec_info_pane_move(0.0, 0.0, IDX_44 as f32, 0.0, 0, 0, 1);
        let temp5 = self.yes_no_pane_move(0.0, IDX_42 as f32, 0, 0, 1);
        temp1 && temp2 && temp3 && temp4 && temp5
    }

    pub fn close_back(&mut self) -> bool {
        let temp1 = self.pane_trance_title(self.timer, IDX_08, 0.0, IDX_48 as f32, 0, 1);
        let temp2 = self.pane_trance_rec_tlt(0,
            self.timer,
            IDX_0c,
            0.0,
            0.0,
            -(IDX_40 as f32),
            0.0,
            0,
            1,
        );
        let temp3 = self.pane_trance_rec_info(0,
            self.timer,
            IDX_11,
            0.0,
            0.0,
            -(IDX_40 as f32),
            0.0,
            0,
            1,
        );
        let temp4 = self.pane_trance_rec_tlt(1,
            self.timer - IDX_10 as i16,
            IDX_0d,
            0.0,
            0.0,
            -(IDX_40 as f32),
            0.0,
            0,
            1,
        );
        let temp5 = self.pane_trance_rec_info(1,
            self.timer - IDX_10 as i16,
            IDX_12,
            0.0,
            0.0,
            -(IDX_40 as f32),
            0.0,
            0,
            1,
        );
        let temp6 = self.pane_trance_rec_tlt(2,
            self.timer - IDX_10 as i16 * 2,
            IDX_0e,
            0.0,
            0.0,
            -(IDX_40 as f32),
            0.0,
            0,
            1,
        );
        let temp7 = self.pane_trance_rec_info(1,
            self.timer - IDX_10 as i16 * 2,
            IDX_12,
            0.0,
            0.0,
            -(IDX_40 as f32),
            0.0,
            0,
            1,
        );
        temp1 && temp2 && temp3 && temp4 && temp5 && temp6 && temp7
    }

    pub fn close_err_erase(&mut self) -> bool {
        let temp1 = self.pane_trance_title(self.timer, IDX_08, 0.0, IDX_48 as f32, 0, 1);
        let temp4 = self.message_pane_move(0.0, 0.0, IDX_44 as f32, 0.0, IDX_31 as i16, 0, 1);
        let (temp2, temp3) = self.rec_row_trance_out();
        temp1 && temp4 && temp2 && temp3
    }

    pub fn close_err_copy(&mut self) -> bool {
        let temp1 = self.pane_trance_title(self.timer, IDX_08, 0.0, IDX_48 as f32, 0, 1);
        let temp2 = self.message_pane_move(0.0, 85.0, IDX_44 as f32, 85.0, IDX_31 as i16, 0, 1);
        let (temp3, temp4, temp5, temp6) = self.copy_pair_trance_out();
        temp1 && temp2 && temp3 && temp4 && temp5 && temp6
    }

    fn copy_pair_trance_out(&mut self) -> (bool, bool, bool, bool) {
        match self.flag_3924 {
            0 => {
                let temp3 = self.pane_trance_rec_tlt(0,
                    self.timer,
                    IDX_0c,
                    0.0,
                    0.0,
                    -(IDX_40 as f32),
                    0.0,
                    0,
                    1,
                );
                let temp4 = self.pane_trance_rec_info(0,
                    self.timer,
                    IDX_11,
                    0.0,
                    0.0,
                    -(IDX_40 as f32),
                    0.0,
                    0,
                    1,
                );
                if self.flag_3925 == 0 {
                    let temp5 = self.pane_trance_rec_tlt(1,
                        self.timer - IDX_37 as i16,
                        IDX_0d,
                        0.0,
                        0.0,
                        -(IDX_40 as f32),
                        0.0,
                        0,
                        1,
                    );
                    let temp6 = self.pane_trance_rec_info(1,
                        self.timer - IDX_37 as i16,
                        IDX_12,
                        0.0,
                        0.0,
                        -(IDX_40 as f32),
                        0.0,
                        0,
                        1,
                    );
                    (temp3, temp4, temp5, temp6)
                } else {
                    let temp5 = self.pane_trance_rec_tlt(2,
                        self.timer - IDX_37 as i16,
                        IDX_0e,
                        0.0,
                        -85.0,
                        -(IDX_40 as f32),
                        -85.0,
                        0,
                        1,
                    );
                    let temp6 = self.pane_trance_rec_info(1,
                        self.timer - IDX_37 as i16,
                        IDX_12,
                        0.0,
                        -85.0,
                        -(IDX_40 as f32),
                        -85.0,
                        0,
                        1,
                    );
                    (temp3, temp4, temp5, temp6)
                }
            }
            1 => {
                let temp3 = self.pane_trance_rec_tlt(1,
                    self.timer,
                    IDX_0d,
                    0.0,
                    -85.0,
                    -(IDX_40 as f32),
                    -85.0,
                    0,
                    1,
                );
                let temp4 = self.pane_trance_rec_info(1,
                    self.timer,
                    IDX_12,
                    0.0,
                    -85.0,
                    -(IDX_40 as f32),
                    -85.0,
                    0,
                    1,
                );
                if self.flag_3925 == 0 {
                    let temp5 = self.pane_trance_rec_tlt(0,
                        self.timer - IDX_37 as i16,
                        IDX_0c,
                        0.0,
                        85.0,
                        -(IDX_40 as f32),
                        85.0,
                        0,
                        1,
                    );
                    let temp6 = self.pane_trance_rec_info(0,
                        self.timer - IDX_37 as i16,
                        IDX_11,
                        0.0,
                        85.0,
                        -(IDX_40 as f32),
                        85.0,
                        0,
                        1,
                    );
                    (temp3, temp4, temp5, temp6)
                } else {
                    let temp5 = self.pane_trance_rec_tlt(2,
                        self.timer - IDX_37 as i16,
                        IDX_0e,
                        0.0,
                        -85.0,
                        -(IDX_40 as f32),
                        -85.0,
                        0,
                        1,
                    );
                    let temp6 = self.pane_trance_rec_info(1,
                        self.timer - IDX_37 as i16,
                        IDX_12,
                        0.0,
                        -85.0,
                        -(IDX_40 as f32),
                        -85.0,
                        0,
                        1,
                    );
                    (temp3, temp4, temp5, temp6)
                }
            }
            _ => {
                let temp3 = self.pane_trance_rec_tlt(2,
                    self.timer,
                    IDX_0e,
                    0.0,
                    -170.0,
                    -(IDX_40 as f32),
                    -170.0,
                    0,
                    1,
                );
                let temp4 = self.pane_trance_rec_info(1,
                    self.timer,
                    IDX_12,
                    0.0,
                    -170.0,
                    -(IDX_40 as f32),
                    -170.0,
                    0,
                    1,
                );
                if self.flag_3925 == 0 {
                    let temp5 = self.pane_trance_rec_tlt(0,
                        self.timer - IDX_37 as i16,
                        IDX_0c,
                        0.0,
                        85.0,
                        -(IDX_40 as f32),
                        85.0,
                        0,
                        1,
                    );
                    let temp6 = self.pane_trance_rec_info(0,
                        self.timer - IDX_37 as i16,
                        IDX_11,
                        0.0,
                        85.0,
                        -(IDX_40 as f32),
                        85.0,
                        0,
                        1,
                    );
                    (temp3, temp4, temp5, temp6)
                } else {
                    let temp5 = self.pane_trance_rec_tlt(1,
                        self.timer - IDX_37 as i16,
                        IDX_0d,
                        0.0,
                        0.0,
                        -(IDX_40 as f32),
                        0.0,
                        0,
                        1,
                    );
                    let temp6 = self.pane_trance_rec_info(1,
                        self.timer - IDX_37 as i16,
                        IDX_12,
                        0.0,
                        0.0,
                        -(IDX_40 as f32),
                        0.0,
                        0,
                        1,
                    );
                    (temp3, temp4, temp5, temp6)
                }
            }
        }
    }

    pub fn move_step(&mut self, pad: &PadInput, card: &mut MemCard, current: Option<SaveGame>) {
        let probe = card.get_probe_stat();
        if (probe == 0 || probe == 1) && self.flag_3941 == 0 {
            self.flag_392e = 1;
            self.timer = 0;
            self.select_end = SELECT_END_CANCEL;
            self.icon_mode = ICON_MODE_DEFAULT;
            self.op = OP_CARD_REMOVED;
            self.proc = proc_next_mode_wait;
        } else {
            match dispatch(self.proc) {
                Proc::DataSelect => self.data_select(pad),
                Proc::SelectTitle => self.select_title_anime(),
                Proc::DataSelectPaneMove => self.data_select_pane_move(),
                Proc::MenuSelect => self.menu_select(pad),
                Proc::ToCopy => self.to_copy_pane_move(),
                Proc::ToErase => self.to_erase_pane_move(),
                Proc::ToBack => self.to_back_pane_move(),
                Proc::CopyDataToSelect => self.copy_data_to_select(pad),
                Proc::CopyDataSelAnime => self.copy_data_sel_anime(),
                Proc::CopyToSelBack => self.copy_to_sel_back(),
                Proc::CopyToSelPaneMove => self.copy_to_sel_pane_move(),
                Proc::YesNoSelect => self.yes_no_select(pad),
                Proc::CmdExecPaneMove0 => self.cmd_exec_pane_move0(),
                Proc::CmdExecPaneMove2 => self.cmd_exec_pane_move2(),
                Proc::CommandExec => self.command_exec(card),
                Proc::DataEraseWait => self.data_erase_wait(card),
                Proc::DataCopyWait => self.data_copy_wait(card),
                Proc::CopyPaneMoveOk => self.copy_pane_move_ok(card),
                Proc::CopyPaneMoveOk2 => self.copy_pane_move_ok2(),
                Proc::ErasePaneMoveOk => self.erase_pane_move_ok(card),
                Proc::ErasePaneMoveOk2 => self.erase_pane_move_ok2(),
                Proc::ErrorMsgPaneMove => self.error_msg_pane_move(),
                Proc::ErrorMsgPaneMove2 => self.error_msg_pane_move2(),
                Proc::BackDatSelWait => self.back_dat_sel_wait(pad),
                Proc::BackDatSelWait2 => self.back_dat_sel_wait2(pad),
                Proc::BackDatSelPaneMove => self.back_dat_sel_pane_move(),
                Proc::DataSelectEx => self.data_select_ex(pad),
                Proc::ExSavePaneMove => self.ex_save_pane_move(),
                Proc::YesNoSelectEx => self.yes_no_select_ex(pad, card),
                Proc::ExSavePaneMove0 => self.ex_save_pane_move0(),
                Proc::ExSavePaneMove1 => self.ex_save_pane_move1(),
                Proc::ExDataSave => self.ex_data_save(card, current),
                Proc::ExDataSaveWait => self.ex_data_save_wait(card),
                Proc::ExBackDatSelPaneMove => self.ex_back_dat_sel_pane_move(),
                Proc::ExDataSavePaneMoveOk => self.ex_data_save_pane_move_ok(card),
                Proc::ExDataSavePaneMoveOk2 => self.ex_data_save_pane_move_ok2(),
                Proc::ExErrorMsgPaneMove => self.ex_error_msg_pane_move(),
                Proc::ExErrorMsgPaneMove2 => self.ex_error_msg_pane_move2(),
                Proc::NextModeWait => {}
            }
        }

        card.clear_probe_stat();
    }

    pub fn data_select(&mut self, pad: &PadInput) {
        self.stick2
            .check_trigger(pad.stick_value, pad.stick_angle);
        if pad.trig_a || pad.trig_start {
            self.sound(SE_MENU_OK);
            self.icon_mode = ICON_MODE_SELECT;
            self.panes.groups.record_cursor[0].hide();
            if self.data_broken[self.select_num as usize] != 0 {
                self.message_set(self.flag_392d as usize ^ 1, MSG_BROKEN);
            }
            self.menu_color_change();
            self.data_sel_move_set();
            self.proc = proc_data_select_pane_move;
        } else if self.stick2.check_up_trigger() {
            if self.select_num != 0 {
                self.sound(SE_MSEL_CURSOR);
                self.select_num -= 1;
                self.slide = 0.0;
                self.title_dir = -1;
                self.timer = 0;
                self.proc = proc_select_title;
                self.rec_cursor_move();
            }
        } else if self.stick2.check_down_trigger() {
            if self.select_num != 2 {
                self.sound(SE_MSEL_CURSOR);
                self.select_num += 1;
                self.slide = 0.0;
                self.title_dir = 1;
                self.timer = 0;
                self.proc = proc_select_title;
                self.rec_cursor_move();
            }
        }

        self.rec_cursor_anime();
    }

    pub fn menu_color_change(&mut self) {
        let grey = |a: u8| Color::new(0x5A, 0x5A, 0x5A, a);
        let pale = |a: u8| Color::new(0xC8, 0xC8, 0xC8, a);
        let (c0, c1, d0, d1) = if self.data_new[self.select_num as usize] != 0 {
            (
                (grey(self.colors.copy_black[0].a), pale(self.colors.copy_white[0].a)),
                (grey(self.colors.copy_black[1].a), pale(self.colors.copy_white[1].a)),
                (
                    grey(self.colors.erase_black[0].a),
                    pale(self.colors.erase_white[1].a),
                ),
                (
                    grey(self.colors.erase_black[0].a),
                    pale(self.colors.erase_white[1].a),
                ),
            )
        } else {
            (
                (
                    self.colors.copy_black[0],
                    self.colors.copy_white[0],
                ),
                (
                    self.colors.copy_black[1],
                    self.colors.copy_white[1],
                ),
                (
                    self.colors.erase_black[0],
                    self.colors.erase_white[0],
                ),
                (
                    self.colors.erase_black[1],
                    self.colors.erase_white[1],
                ),
            )
        };
        self.panes.groups.copy_pic[0].set_black_white(c0.0, c0.1);
        self.panes.groups.copy_pic[1].set_black_white(c1.0, c1.1);
        self.panes.groups.erase_pic[0].set_black_white(d0.0, d0.1);
        self.panes.groups.erase_pic[1].set_black_white(d1.0, d1.1);
    }

    pub fn select_title_anime(&mut self) {
        let temp1 = value_increase(
            i32::from(IDX_4f),
            i32::from(self.timer),
            5,
        );
        self.slide = IDX_4e as f32 * temp1 * f32::from(self.title_dir);

        match self.select_num {
            0 => {
                self.panes.slots.key[0][0].pane_trans(0.0, self.slide);
                if self.panes.slots.slot_mark[0].is_visible() {
                    self.panes.slots.slot_mark[0].pane_trans(0.0, self.slide);
                    self.panes.slots.slot_mark_key[0].pane_trans(0.0, self.slide);
                }
            }
            1 => {
                self.panes.slots.key[1][0].pane_trans(0.0, self.slide);
                if self.panes.slots.slot_mark[1].is_visible() {
                    self.panes.slots.slot_mark[1].pane_trans(0.0, self.slide);
                    self.panes.slots.slot_mark_key[1].pane_trans(0.0, self.slide);
                }
            }
            _ => {
                self.panes.slots.key[2][0].pane_trans(0.0, self.slide);
                if self.panes.slots.slot_mark[0].is_visible() {
                    self.panes.slots.slot_mark[2].pane_trans(0.0, self.slide);
                    self.panes.slots.slot_mark_key[2].pane_trans(0.0, self.slide);
                }
            }
        }

        self.panes.groups.record_cursor[0].pane_trans(
            0.0,
            f32::from(self.select_num) * SLOT_ROW_DY + self.slide,
        );
        self.timer += 1;
        if self.timer > IDX_4f as i16 {
            self.timer = 0;
            if self.use_type == MODE_NORMAL {
                self.proc = proc_data_select;
            } else {
                self.proc = proc_data_select_ex;
            }
        }
    }

    pub fn rec_cursor_move(&mut self) {
        self.panes.groups.record_cursor[0]
            .pane_trans(0.0, f32::from(self.select_num) * SLOT_ROW_DY);
    }

    pub fn rec_cursor_anime(&mut self) {
        let mut temp = value_increase(
            i32::from(IDX_22),
            i32::from(self.panes.groups.record_cursor[0].user_area),
            0,
        );
        if self.flag_3921 != 0 {
            temp = 1.0 - temp;
        }

        let span = IDX_23 as f32 - IDX_24 as f32;
        self.panes.groups.record_cursor[0].now_alpha = IDX_24 + (span * temp) as u8;

        if self.panes.groups.record_cursor[0].user_area == 0 {
            self.panes.groups.record_cursor[0].user_area = IDX_22 as i16;
            self.flag_3921 ^= 1;
        } else {
            self.panes.groups.record_cursor[0].user_area -= 1;
        }
    }

    pub fn rec_cursor_alpha_init(&mut self) {
        self.panes.groups.record_cursor[0].now_alpha = 0;
        self.panes.groups.record_cursor[0].user_area = 0;
        self.flag_3921 = 0;
    }

    pub fn rec_data_pane_move(&mut self) -> bool {
        let (temp1, temp2, temp3, temp4, temp5, temp6) = match self.select_num {
            0 => {
                let temp1 = self.pane_trance_rec_tlt(1,
                    self.timer - IDX_29 as i16,
                    IDX_28,
                    0.0,
                    0.0,
                    IDX_40 as f32,
                    0.0,
                    0,
                    1,
                );
                let temp2 = self.pane_trance_rec_info(1,
                    self.timer - IDX_29 as i16,
                    IDX_28,
                    0.0,
                    0.0,
                    IDX_40 as f32,
                    0.0,
                    0,
                    1,
                );
                let temp3 = self.pane_trance_rec_tlt(2,
                    self.timer - IDX_2a as i16,
                    IDX_28,
                    0.0,
                    0.0,
                    IDX_40 as f32,
                    0.0,
                    0,
                    1,
                );
                let temp4 = self.pane_trance_rec_info(2,
                    self.timer - IDX_2a as i16,
                    IDX_28,
                    0.0,
                    0.0,
                    IDX_40 as f32,
                    0.0,
                    0,
                    1,
                );
                self.flag_3920 = 0;
                (temp1, temp2, temp3, temp4, true, true)
            }
            1 => {
                let temp1 = self.pane_trance_rec_tlt(0,
                    self.timer - IDX_29 as i16,
                    IDX_28,
                    0.0,
                    0.0,
                    IDX_40 as f32,
                    0.0,
                    0,
                    1,
                );
                let temp2 = self.pane_trance_rec_info(0,
                    self.timer - IDX_29 as i16,
                    IDX_28,
                    0.0,
                    0.0,
                    IDX_40 as f32,
                    0.0,
                    0,
                    1,
                );
                let temp3 = self.pane_trance_rec_tlt(2,
                    self.timer - IDX_2a as i16,
                    IDX_28,
                    0.0,
                    0.0,
                    IDX_40 as f32,
                    0.0,
                    0,
                    1,
                );
                let temp4 = self.pane_trance_rec_info(2,
                    self.timer - IDX_2a as i16,
                    IDX_28,
                    0.0,
                    0.0,
                    IDX_40 as f32,
                    0.0,
                    0,
                    1,
                );
                let temp5 = self.pane_trance_rec_tlt(1,
                    self.timer - IDX_2d as i16,
                    IDX_28,
                    0.0,
                    0.0,
                    0.0,
                    -85.0,
                    IDX_07,
                    2,
                );
                let temp6 = self.pane_trance_rec_info(1,
                    self.timer - IDX_2d as i16,
                    IDX_28,
                    0.0,
                    0.0,
                    0.0,
                    -85.0,
                    IDX_07,
                    2,
                );
                self.flag_3920 = IDX_28 / 2;
                (temp1, temp2, temp3, temp4, temp5, temp6)
            }
            _ => {
                let temp1 = self.pane_trance_rec_tlt(0,
                    self.timer - IDX_29 as i16,
                    IDX_28,
                    0.0,
                    0.0,
                    IDX_40 as f32,
                    0.0,
                    0,
                    1,
                );
                let temp2 = self.pane_trance_rec_info(0,
                    self.timer - IDX_29 as i16,
                    IDX_28,
                    0.0,
                    0.0,
                    IDX_40 as f32,
                    0.0,
                    0,
                    1,
                );
                let temp3 = self.pane_trance_rec_tlt(1,
                    self.timer - IDX_2a as i16,
                    IDX_28,
                    0.0,
                    0.0,
                    IDX_40 as f32,
                    0.0,
                    0,
                    1,
                );
                let temp4 = self.pane_trance_rec_info(1,
                    self.timer - IDX_2a as i16,
                    IDX_28,
                    0.0,
                    0.0,
                    IDX_40 as f32,
                    0.0,
                    0,
                    1,
                );
                let temp5 = self.pane_trance_rec_tlt(2,
                    self.timer - IDX_2d as i16,
                    IDX_28,
                    0.0,
                    0.0,
                    0.0,
                    -170.0,
                    IDX_07,
                    2,
                );
                let temp6 = self.pane_trance_rec_info(2,
                    self.timer - IDX_2d as i16,
                    IDX_28,
                    0.0,
                    0.0,
                    0.0,
                    -170.0,
                    IDX_07,
                    2,
                );
                self.flag_3920 = IDX_28 / 2;
                (temp1, temp2, temp3, temp4, temp5, temp6)
            }
        };
        temp1 && temp2 && temp3 && temp4 && temp5 && temp6
    }

    pub fn data_sel_move_set(&mut self) {
        self.timer = 0;
        if self.data_broken[self.select_num as usize] != 0 {
            self.message_set(2, MSG_CORRUPT_SUMMARY);
        } else {
            self.make_rec_info(self.select_num);
        }
    }

    pub fn make_rec_info(&mut self, data_no: u8) {
        let index = usize::from(data_no).min(SLOT_COUNT - 1);
        let summary = self.summary[index].clone();

        for i in 0..0x14usize {
            if summary.heart_shown[i] {
                self.panes.slots.heart[i].show();
                self.panes.slots.heart_shadow[i].show();
                if let Some(texture) = summary.heart_texture[i] {
                    self.panes.slots.heart[i].change_texture(texture);
                    self.panes.slots.heart_shadow[i].change_texture(texture);
                }
            } else {
                self.panes.slots.heart[i].hide();
                self.panes.slots.heart_shadow[i].hide();
            }
        }

        if summary.triforce != 0 {
            self.panes.slots.collect[2].hide();
            self.panes.slots.triforce_bar.show();

            if summary.triforce == TRIFORCE_FULL {
                for pane in &mut self.panes.slots.shard[..TRIFORCE_COUNT] {
                    pane.hide();
                }
                self.panes.slots.shard[8].show();
            } else {
                self.panes.slots.shard[8].hide();
                let mut bit = 1u8;
                for i in 0..TRIFORCE_COUNT {
                    if summary.triforce & bit != 0 {
                        self.panes.slots.shard[i].show();
                    } else {
                        self.panes.slots.shard[i].hide();
                    }
                    bit <<= 1;
                }
            }
        } else {
            self.panes.slots.collect[2].show();
            self.panes.slots.triforce_bar.hide();
            for pane in &mut self.panes.slots.shard {
                pane.hide();
            }
        }

        let mut bit = 1u8;
        for i in 0..PEARL_COUNT {
            if summary.symbol & bit != 0 {
                self.panes.slots.pearl[i].show();
                self.panes.slots.pearl_shadow[i].show();
            } else {
                self.panes.slots.pearl[i].hide();
                self.panes.slots.pearl_shadow[i].hide();
            }
            bit <<= 1;
        }
    }

    pub fn data_select_pane_move(&mut self) {
        let temp1 = self.rec_data_pane_move();
        let mut temp4 = true;
        let mut temp5 = false;
        let (temp2, temp3);
        if self.data_broken[self.select_num as usize] != 0 {
            temp2 = self.message_pane_move(IDX_44 as f32, 0.0, 0.0, 0.0, (IDX_30 as i16) + i16::from(self.flag_3920),
                1,
                0,
            );
            temp3 = self.yes_no_pane_move(
                IDX_42 as f32,
                0.0,
                (IDX_38 as i16) + i16::from(self.flag_3920),
                1,
                0,
            );
            temp4 = self.pane_alpha_title_txt(self.timer - IDX_29 as i16, IDX_09);
            temp5 = true;
        } else {
            temp2 = self.rec_info_pane_move(IDX_44 as f32, 0.0, 0.0, 0.0, (IDX_30 as i16) + i16::from(self.flag_3920),
                1,
                0,
            );
            temp3 = self.menu_pane_move(
                IDX_42 as f32,
                0.0,
                (IDX_35 as i16) + i16::from(self.flag_3920),
                1,
                0,
            );
        }

        self.timer += 1;
        if temp1 && temp2 && temp3 && temp4 {
            if temp5 {
                self.flag_392d ^= 1;
            }

            self.menu_cursor_alpha_init();
            self.panes.groups.menu_cursor.show();

            if self.data_broken[self.select_num as usize] != 0 {
                self.menu_pos = 1;
                self.yes_no_cursor_move();
                self.timer = 0;
                self.extra_text_y = 0.0;
                self.proc = proc_yes_no_select;
                self.op = OP_ERASE;
            } else {
                self.menu_pos = 0;
                self.menu_cursor_move();
                self.proc = proc_menu_select;
                self.op = OP_SELECT;
            }
        }
    }

    pub fn message_pane_move_set(
        &mut self,
        x: f32,
        y: f32,
        x2: f32,
        y2: f32,
        mode: u8,
        flip: i32,
    ) {
        self.pane_trance_message_base(self.timer, IDX_32, x, y, x2, y2, mode, flip);

        if !self.panes.groups.message_key[0].is_visible() {
            self.panes.groups.message_key[0].show();
            self.panes.groups.message_key[1].show();
        }
    }

    pub fn menu_select(&mut self, pad: &PadInput) {
        self.stick
            .check_trigger(pad.stick_value, pad.stick_angle);
        if pad.trig_a {
            match self.menu_pos {
                MENU_POS_START => {
                    let slot = self.select_num as usize;
                    if self.data_new[slot] != 0 {
                        self.sound(SE_MENU_OK);
                    } else if self.data_extra[slot] != 0 && self.data_extra_link[slot] != 0 {
                        self.sound(SE_MENU_OK);
                    } else {
                        self.sound(SE_MSEL_ENTER);
                    }
                    self.timer = 0;
                    self.panes.groups.menu_cursor.hide();
                    self.select_end = SELECT_END_START;
                    self.slot_picked = Some(slot);
                    self.proc = proc_next_mode_wait;
                }
                MENU_POS_COPY => {
                    self.sound(SE_MENU_OK);
                    self.timer = 0;
                    self.panes.groups.menu_cursor.hide();
                    self.flag_3924 = self.select_num;
                    self.save_copy_bw_color();
                    self.extra_text_y = SLOT_ROW_DY;
                    self.message_set(self.flag_392d as usize ^ 1, MSG_COPY_ASK);
                    self.proc = proc_to_copy;
                    self.op = OP_COPY;
                }
                MENU_POS_ERASE => {
                    self.sound(SE_MENU_OK);
                    self.timer = 0;
                    self.panes.groups.menu_cursor.hide();
                    self.extra_text_y = 0.0;
                    self.message_set(self.flag_392d as usize ^ 1, MSG_BROKEN);
                    self.proc = proc_to_erase;
                    self.op = OP_ERASE;
                }
                _ => {
                    self.sound(SE_MENU_CANCEL);
                    self.timer = 0;
                    self.panes.groups.menu_cursor.hide();
                    self.extra_text_y = 0.0;
                    self.icon_mode = ICON_MODE_BACK;
                    self.proc = proc_to_back;
                    self.op = OP_BACK;
                }
            }
        } else if pad.trig_b {
            self.sound(SE_MENU_CANCEL);
            self.timer = 0;
            self.panes.groups.menu_cursor.hide();
            self.extra_text_y = 0.0;
            self.icon_mode = ICON_MODE_BACK;
            self.proc = proc_to_back;
            self.op = OP_BACK;
        } else if self.stick.check_right_trigger() {
            if self.menu_pos != MENU_POS_RETURN {
                self.sound(SE_MSEL_CURSOR);
                self.menu_pos += 1;
                if self.data_new[self.select_num as usize] != 0
                    && (self.menu_pos == MENU_POS_COPY || self.menu_pos == MENU_POS_ERASE)
                {
                    self.menu_pos = MENU_POS_RETURN;
                }
            }

            self.timer = 0;
            self.menu_cursor_move();
        } else if self.stick.check_left_trigger() {
            if self.menu_pos != MENU_POS_START {
                self.sound(SE_MSEL_CURSOR);
                self.menu_pos -= 1;
                if self.data_new[self.select_num as usize] != 0
                    && (self.menu_pos == MENU_POS_COPY || self.menu_pos == MENU_POS_ERASE)
                {
                    self.menu_pos = MENU_POS_START;
                }
            }

            self.timer = 0;
            self.menu_cursor_move();
        }

        self.menu_cursor_anime();
    }

    pub fn menu_cursor_alpha_init(&mut self) {
        self.panes.groups.menu_cursor.now_alpha = 0;
        self.panes.groups.menu_cursor.user_area = 0;
        self.flag_3934[0] = 0;
    }

    pub fn menu_cursor_move(&mut self) {
        let x = MENU_X[usize::from(self.menu_pos)];
        self.panes.groups.menu_cursor.pane_trans(x, 0.0);
    }

    pub fn menu_cursor_anime(&mut self) {
        let mut temp = value_increase(
            i32::from(IDX_25),
            i32::from(self.panes.groups.menu_cursor.user_area),
            0,
        );
        if self.flag_3934[0] != 0 {
            temp = 1.0 - temp;
        }

        let span = IDX_26 as f32 - IDX_27 as f32;
        self.panes.groups.menu_cursor.now_alpha = IDX_27 + (span * temp) as u8;

        if self.panes.groups.menu_cursor.user_area == 0 {
            self.panes.groups.menu_cursor.user_area = IDX_25 as i16;
            self.flag_3934[0] ^= 1;
        } else {
            self.panes.groups.menu_cursor.user_area -= 1;
        }
    }

    pub fn to_copy_pane_move(&mut self) {
        let temp1 = self.data_select_pane_copy_move(IDX_2b as i16, IDX_2c as i16);
        let temp2 = self.data_select_pane_copy_color_move(self.timer, IDX_0a, 1);
        let temp3 = self.rec_info_pane_move(0.0, 0.0, IDX_44 as f32, 0.0, IDX_31 as i16, 0, 1);
        let temp4 = self.menu_pane_move(0.0, IDX_42 as f32, IDX_36 as i16, 0, 1);
        let temp5 = self.pane_alpha_title_txt(self.timer - IDX_29 as i16, IDX_09);

        self.timer += 1;

        if temp1 && temp2 && temp3 && temp4 && temp5 {
            self.flag_392d ^= 1;
            self.timer = 0;
            self.rec_cursor_alpha_init();
            self.panes.groups.record_cursor[0].show();
            self.flag_3925 = 0;
            self.copy_cursor_move();
            self.proc = proc_copy_data_to_select;
        }
    }

    pub fn to_erase_pane_move(&mut self) {
        let temp1 = self.menu_pane_move(0.0, IDX_42 as f32, IDX_36 as i16, 0, 1);
        let temp2 = self.yes_no_pane_move(IDX_42 as f32, 0.0, IDX_38 as i16, 1, 0);
        let temp3 = self.pane_alpha_title_txt(self.timer - IDX_29 as i16, IDX_09);

        self.timer += 1;

        if temp1 && temp2 && temp3 {
            self.flag_392d ^= 1;
            self.timer = 0;
            self.menu_cursor_alpha_init();
            self.panes.groups.menu_cursor.show();
            self.menu_pos = MENU_POS_RETURN;
            self.yes_no_cursor_move();
            self.proc = proc_yes_no_select;
        }
    }

    pub fn to_back_pane_move(&mut self) {
        let temp1 = self.data_select_pane_back_move(IDX_2b as i16, IDX_2c as i16, IDX_2e as i16);
        let temp2 = self.rec_info_pane_move(0.0, 0.0, IDX_44 as f32, 0.0, IDX_31 as i16, 0, 1);
        let temp3 = self.menu_pane_move(0.0, IDX_42 as f32, IDX_36 as i16, 0, 1);

        self.timer += 1;

        if temp1 && temp2 && temp3 {
            self.rec_cursor_alpha_init();
            self.panes.groups.record_cursor[0].show();
            self.proc = proc_data_select;
        }
    }

    pub fn copy_data_to_select(&mut self, pad: &PadInput) {
        self.stick
            .check_trigger(pad.stick_value, pad.stick_angle);
        if pad.trig_a {
            self.sound(SE_MENU_OK);

            self.flag_3926 = match self.flag_3924 {
                0 => self.flag_3925 + 1,
                1 => self.flag_3925 * 2,
                _ => self.flag_3925,
            };

            if self.data_broken[self.flag_3926 as usize] != 0 {
                self.message_set(2, MSG_COPY_DEST_BUSY);
            } else {
                self.make_rec_info(self.flag_3926);
            }

            self.message_set(self.flag_392d as usize ^ 1, MSG_COPY_TO);
            self.panes.groups.record_cursor[0].hide();
            self.timer = 0;
            self.proc = proc_copy_to_sel_pane_move;
        } else if pad.trig_b {
            self.sound(SE_MENU_CANCEL);
            self.panes.groups.record_cursor[0].hide();
            self.message_set(self.flag_392d as usize ^ 1, MSG_CANCEL);
            self.icon_mode = ICON_MODE_BACK;
            self.proc = proc_copy_to_sel_back;
        } else if self.stick.check_up_trigger() {
            if self.flag_3925 != 0 {
                self.sound(SE_MSEL_CURSOR);
                self.slide = 0.0;
                self.title_dir = -1;
                self.timer = 0;
                self.flag_3925 = 0;
                self.copy_cursor_move();
                self.proc = proc_copy_data_sel_anime;
            }
        } else if self.stick.check_down_trigger() {
            if self.flag_3925 != 1 {
                self.sound(SE_MSEL_CURSOR);
                self.slide = 0.0;
                self.title_dir = 1;
                self.timer = 0;
                self.flag_3925 = 1;
                self.copy_cursor_move();
                self.proc = proc_copy_data_sel_anime;
            }
        }

        self.rec_cursor_anime();
    }

    pub fn copy_data_sel_anime(&mut self) {
        let temp = value_increase(i32::from(IDX_4f), i32::from(self.timer), 5);
        self.slide = IDX_4e as f32 * temp * f32::from(self.title_dir);

        match self.flag_3924 {
            0 => {
                if self.flag_3925 == 0 {
                    self.panes.slots.key[1][0].pane_trans(0.0, self.slide + SLOT_COPY_DY);
                    if self.panes.slots.slot_mark[1].is_visible() {
                        self.panes.slots.slot_mark[1]
                            .pane_trans(0.0, self.slide + SLOT_COPY_DY);
                        self.panes.slots.slot_mark_key[1]
                            .pane_trans(0.0, self.slide + SLOT_COPY_DY);
                    }
                } else {
                    self.panes.slots.key[2][0].pane_trans(0.0, self.slide + SLOT_COPY_DY);
                    if self.panes.slots.slot_mark[2].is_visible() {
                        self.panes.slots.slot_mark[2]
                            .pane_trans(0.0, self.slide + SLOT_COPY_DY);
                        self.panes.slots.slot_mark_key[2]
                            .pane_trans(0.0, self.slide + SLOT_COPY_DY);
                    }
                }
            }
            1 => {
                if self.flag_3925 == 0 {
                    self.panes.slots.key[0][0]
                        .pane_trans(0.0, self.slide + SLOT_COPY_DY_ALT);
                    if self.panes.slots.slot_mark[0].is_visible() {
                        self.panes.slots.slot_mark[0]
                            .pane_trans(0.0, self.slide + SLOT_COPY_DY_ALT);
                        self.panes.slots.slot_mark_key[0]
                            .pane_trans(0.0, self.slide + SLOT_COPY_DY_ALT);
                    }
                } else {
                    self.panes.slots.key[2][0].pane_trans(0.0, self.slide + SLOT_COPY_DY);
                    if self.panes.slots.slot_mark[2].is_visible() {
                        self.panes.slots.slot_mark[2]
                            .pane_trans(0.0, self.slide + SLOT_COPY_DY);
                        self.panes.slots.slot_mark_key[2]
                            .pane_trans(0.0, self.slide + SLOT_COPY_DY);
                    }
                }
            }
            _ => {
                if self.flag_3925 == 0 {
                    self.panes.slots.key[0][0]
                        .pane_trans(0.0, self.slide + SLOT_COPY_DY_ALT);
                    if self.panes.slots.slot_mark[0].is_visible() {
                        self.panes.slots.slot_mark[0]
                            .pane_trans(0.0, self.slide + SLOT_COPY_DY_ALT);
                        self.panes.slots.slot_mark_key[0]
                            .pane_trans(0.0, self.slide + SLOT_COPY_DY_ALT);
                    }
                } else {
                    self.panes.slots.key[1][0]
                        .pane_trans(0.0, self.slide + SLOT_COPY_DY_ALT);
                    if self.panes.slots.slot_mark[1].is_visible() {
                        self.panes.slots.slot_mark[1]
                            .pane_trans(0.0, self.slide + SLOT_COPY_DY_ALT);
                        self.panes.slots.slot_mark_key[1]
                            .pane_trans(0.0, self.slide + SLOT_COPY_DY_ALT);
                    }
                }
            }
        }

        self.panes.groups.record_cursor[0].pane_trans(
            0.0,
            (f32::from(self.flag_3925) + 1.0) * SLOT_ROW_DY + SLOT_COPY_DY + self.slide,
        );
        self.timer += 1;
        if self.timer > IDX_4f as i16 {
            self.timer = 0;
            self.proc = proc_copy_data_to_select;
        }
    }

    pub fn copy_cursor_move(&mut self) {
        self.panes.groups.record_cursor[0].pane_trans(
            0.0,
            (f32::from(self.flag_3925) + 1.0) * SLOT_ROW_DY + SLOT_COPY_DY,
        );
    }

    pub fn copy_to_sel_back(&mut self) {
        let temp1 =
            self.data_select_pane_back_from_copy_move(IDX_20, IDX_21);
        let temp2 = self.data_select_pane_copy_color_move(self.timer, IDX_0a, 0);
        let temp3 = self.pane_alpha_title_txt(self.timer - IDX_29 as i16, IDX_09);

        self.timer += 1;

        if temp1 && temp2 && temp3 {
            self.flag_392d ^= 1;
            self.panes.groups.record_cursor[0].show();
            self.rec_cursor_move();
            self.proc = proc_data_select;
        }
    }

    pub fn copy_to_sel_pane_move(&mut self) {
        let temp1 = self.data_select_pane_copy_move2(IDX_29 as i16, IDX_2a as i16);
        let temp2 = if self.data_broken[self.flag_3926 as usize] != 0 {
            self.message_pane_move(IDX_44 as f32, 85.0, 0.0, 85.0, IDX_30 as i16, 1, 0, )
        } else {
            self.rec_info_pane_move(IDX_44 as f32, 85.0, 0.0, 85.0, IDX_30 as i16, 1, 0, )
        };
        let temp3 = self.yes_no_pane_move(IDX_42 as f32, 0.0, IDX_38 as i16, 1, 0);
        let temp4 = self.pane_alpha_title_txt(self.timer - IDX_29 as i16, IDX_09);

        self.timer += 1;

        if temp1 && temp2 && temp3 && temp4 {
            self.flag_392d ^= 1;
            self.panes.groups.menu_cursor.show();
            self.menu_pos = 1;
            self.yes_no_cursor_move();
            self.timer = 0;
            self.proc = proc_yes_no_select;
        }
    }

    pub fn copy_pane_move_ok(&mut self, card: &mut MemCard) {
        let temp = match self.flag_3924 {
            0 => {
                if self.flag_3925 == 0 {
                    self.pane_trance_rec_info(1,
                        self.timer - IDX_3a as i16,
                        IDX_12,
                        0.0,
                        0.0,
                        0.0,
                        IDX_3e as f32,
                        IDX_07,
                        1,
                    )
                } else {
                    self.pane_trance_rec_info(2,
                        self.timer - IDX_3a as i16,
                        IDX_13,
                        0.0,
                        -85.0,
                        0.0,
                        IDX_3e as f32 - 85.0,
                        IDX_07,
                        1,
                    )
                }
            }
            1 => {
                if self.flag_3925 == 0 {
                    self.pane_trance_rec_info(0,
                        self.timer - IDX_3a as i16,
                        IDX_11,
                        0.0,
                        85.0,
                        0.0,
                        IDX_3e as f32 + 85.0,
                        IDX_07,
                        1,
                    )
                } else {
                    self.pane_trance_rec_info(2,
                        self.timer - IDX_3a as i16,
                        IDX_13,
                        0.0,
                        -85.0,
                        0.0,
                        IDX_3e as f32 - 85.0,
                        IDX_07,
                        1,
                    )
                }
            }
            _ => {
                if self.flag_3925 == 0 {
                    self.pane_trance_rec_info(0,
                        self.timer - IDX_3a as i16,
                        IDX_11,
                        0.0,
                        85.0,
                        0.0,
                        IDX_3e as f32 + 85.0,
                        IDX_07,
                        1,
                    )
                } else {
                    self.pane_trance_rec_info(1,
                        self.timer - IDX_3a as i16,
                        IDX_12,
                        0.0,
                        0.0,
                        0.0,
                        IDX_3e as f32,
                        IDX_07,
                        1,
                    )
                }
            }
        };

        self.message_pane_move(0.0, 85.0, IDX_46 as f32, 85.0, IDX_3b as i16, 0, 1);
        self.timer += 1;

        if temp {
            self.set_save_data(card);
            self.make_rec_info(self.flag_3926);
            self.timer = 0;
            self.proc = proc_copy_pane_move_ok2;
        }
    }

    pub fn copy_pane_move_ok2(&mut self) {
        let temp = match self.flag_3924 {
            0 => {
                if self.flag_3925 == 0 {
                    self.pane_trance_rec_info(1,
                        self.timer - IDX_3a as i16,
                        IDX_12,
                        0.0,
                        -85.0,
                        0.0,
                        0.0,
                        1,
                        0,
                    )
                } else {
                    self.pane_trance_rec_info(2,
                        self.timer - IDX_3a as i16,
                        IDX_13,
                        0.0,
                        -170.0,
                        0.0,
                        -85.0,
                        1,
                        0,
                    )
                }
            }
            1 => {
                if self.flag_3925 == 0 {
                    self.pane_trance_rec_info(0,
                        self.timer - IDX_3a as i16,
                        IDX_11,
                        0.0,
                        0.0,
                        0.0,
                        85.0,
                        1,
                        0,
                    )
                } else {
                    self.pane_trance_rec_info(2,
                        self.timer - IDX_3a as i16,
                        IDX_13,
                        0.0,
                        -170.0,
                        0.0,
                        -85.0,
                        1,
                        0,
                    )
                }
            }
            _ => {
                if self.flag_3925 == 0 {
                    self.pane_trance_rec_info(0,
                        self.timer - IDX_3a as i16,
                        IDX_11,
                        0.0,
                        0.0,
                        0.0,
                        85.0,
                        1,
                        0,
                    )
                } else {
                    self.pane_trance_rec_info(1,
                        self.timer - IDX_3a as i16,
                        IDX_12,
                        0.0,
                        -85.0,
                        0.0,
                        0.0,
                        1,
                        0,
                    )
                }
            }
        };

        let temp4 = self.rec_info_pane_move(0.0, 0.0, 0.0, 85.0, IDX_3a as i16, 1, 0);
        let temp5 = self.pane_alpha_title_txt(self.timer - IDX_29 as i16, IDX_09);
        self.timer += 1;

        if temp && temp4 && temp5 {
            self.flag_392d ^= 1;
            self.timer = 0;
            self.proc = proc_back_dat_sel_wait;
        }
    }

    pub fn save_copy_bw_color(&mut self) {
        let slot = usize::from(self.flag_3924).min(SLOT_COUNT - 1);
        for index in 2..12usize {
            self.colors.slot_black[index - 2] = self.panes.slots.key[slot][index].black;
            self.colors.slot_white[index - 2] = self.panes.slots.key[slot][index].white;
        }
        self.colors.slot_black[10] = self.panes.slots.key[slot][1].black;
        self.colors.slot_white[10] = self.panes.slots.key[slot][1].white;
        for index in 3..21usize {
            self.colors.info_black[index - 3] = self.panes.slots.info[slot][index].black;
            self.colors.info_white[index - 3] = self.panes.slots.info[slot][index].white;
        }
    }

    pub fn data_select_pane_copy_move(&mut self, param_1: i16, param_2: i16) -> bool {
        let span = IDX_40 as f32;
        let (temp1, temp2, temp3, temp4) = match self.flag_3924 {
            0 => {
                let temp1 = self.pane_trance_rec_tlt(1,
                    self.timer - param_1,
                    IDX_28,
                    span,
                    SLOT_COPY_DY,
                    0.0,
                    SLOT_COPY_DY,
                    1,
                    0,
                );
                let temp2 = self.pane_trance_rec_info(1,
                    self.timer - param_1,
                    IDX_28,
                    span,
                    SLOT_COPY_DY,
                    0.0,
                    SLOT_COPY_DY,
                    1,
                    0,
                );
                let temp3 = self.pane_trance_rec_tlt(2,
                    self.timer - param_2,
                    IDX_28,
                    span,
                    SLOT_COPY_DY,
                    0.0,
                    SLOT_COPY_DY,
                    1,
                    0,
                );
                let temp4 = self.pane_trance_rec_info(2,
                    self.timer - param_2,
                    IDX_28,
                    span,
                    SLOT_COPY_DY,
                    0.0,
                    SLOT_COPY_DY,
                    1,
                    0,
                );
                (temp1, temp2, temp3, temp4)
            }
            1 => {
                let temp1 = self.pane_trance_rec_tlt(0,
                    self.timer - param_1,
                    IDX_28,
                    span,
                    SLOT_COPY_DY_ALT,
                    0.0,
                    SLOT_COPY_DY_ALT,
                    1,
                    0,
                );
                let temp2 = self.pane_trance_rec_info(0,
                    self.timer - param_1,
                    IDX_28,
                    span,
                    SLOT_COPY_DY_ALT,
                    0.0,
                    SLOT_COPY_DY_ALT,
                    1,
                    0,
                );
                let temp3 = self.pane_trance_rec_tlt(2,
                    self.timer - param_2,
                    IDX_28,
                    span,
                    SLOT_COPY_DY,
                    0.0,
                    SLOT_COPY_DY,
                    1,
                    0,
                );
                let temp4 = self.pane_trance_rec_info(2,
                    self.timer - param_2,
                    IDX_28,
                    span,
                    SLOT_COPY_DY,
                    0.0,
                    SLOT_COPY_DY,
                    1,
                    0,
                );
                (temp1, temp2, temp3, temp4)
            }
            _ => {
                let temp1 = self.pane_trance_rec_tlt(0,
                    self.timer - param_1,
                    IDX_28,
                    span,
                    SLOT_COPY_DY_ALT,
                    0.0,
                    SLOT_COPY_DY_ALT,
                    1,
                    0,
                );
                let temp2 = self.pane_trance_rec_info(0,
                    self.timer - param_1,
                    IDX_28,
                    span,
                    SLOT_COPY_DY_ALT,
                    0.0,
                    SLOT_COPY_DY_ALT,
                    1,
                    0,
                );
                let temp3 = self.pane_trance_rec_tlt(1,
                    self.timer - param_2,
                    IDX_28,
                    span,
                    SLOT_COPY_DY_ALT,
                    0.0,
                    SLOT_COPY_DY_ALT,
                    1,
                    0,
                );
                let temp4 = self.pane_trance_rec_info(1,
                    self.timer - param_2,
                    IDX_28,
                    span,
                    SLOT_COPY_DY_ALT,
                    0.0,
                    SLOT_COPY_DY_ALT,
                    1,
                    0,
                );
                (temp1, temp2, temp3, temp4)
            }
        };
        temp1 && temp2 && temp3 && temp4
    }

    pub fn data_select_pane_copy_back_move(&mut self, param_1: i16, param_2: i16) -> bool {
        let span = IDX_40 as f32;
        match self.flag_3924 {
            0 => {
                if self.flag_3925 == 0 {
                    let temp1 = self.pane_trance_rec_tlt(2,
                        self.timer - param_1,
                        IDX_28,
                        span,
                        0.0,
                        0.0,
                        0.0,
                        1,
                        0,
                    );
                    let temp2 = self.pane_trance_rec_info(2,
                        self.timer - param_1,
                        IDX_28,
                        span,
                        0.0,
                        0.0,
                        0.0,
                        1,
                        0,
                    );
                    temp1 && temp2
                } else {
                    let temp1 = self.pane_trance_rec_tlt(2,
                        self.timer - param_1,
                        IDX_28,
                        0.0,
                        -85.0,
                        0.0,
                        0.0,
                        IDX_07,
                        2,
                    );
                    let temp2 = self.pane_trance_rec_info(2,
                        self.timer - param_1,
                        IDX_28,
                        0.0,
                        -85.0,
                        0.0,
                        0.0,
                        IDX_07,
                        2,
                    );
                    let temp3 = self.pane_trance_rec_tlt(1,
                        self.timer - param_2,
                        IDX_28,
                        span,
                        0.0,
                        0.0,
                        0.0,
                        1,
                        0,
                    );
                    let temp4 = self.pane_trance_rec_info(1,
                        self.timer - param_2,
                        IDX_28,
                        span,
                        0.0,
                        0.0,
                        0.0,
                        1,
                        0,
                    );
                    temp1 && temp2 && temp3 && temp4
                }
            }
            1 => {
                let temp5 = self.pane_trance_rec_tlt(1,
                    self.timer - param_1,
                    IDX_28,
                    0.0,
                    -85.0,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                let temp6 = self.pane_trance_rec_info(1,
                    self.timer - param_1,
                    IDX_28,
                    0.0,
                    -85.0,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                if self.flag_3925 == 0 {
                    let temp1 = self.pane_trance_rec_tlt(0,
                        self.timer - param_1,
                        IDX_28,
                        0.0,
                        85.0,
                        0.0,
                        0.0,
                        IDX_07,
                        2,
                    );
                    let temp2 = self.pane_trance_rec_info(0,
                        self.timer - param_1,
                        IDX_28,
                        0.0,
                        85.0,
                        0.0,
                        0.0,
                        IDX_07,
                        2,
                    );
                    let temp3 = self.pane_trance_rec_tlt(2,
                        self.timer - param_2,
                        IDX_28,
                        span,
                        0.0,
                        0.0,
                        0.0,
                        1,
                        0,
                    );
                    let temp4 = self.pane_trance_rec_info(2,
                        self.timer - param_2,
                        IDX_28,
                        span,
                        0.0,
                        0.0,
                        0.0,
                        1,
                        0,
                    );
                    temp1 && temp2 && temp3 && temp4 && temp5 && temp6
                } else {
                    let temp1 = self.pane_trance_rec_tlt(2,
                        self.timer - param_1,
                        IDX_28,
                        0.0,
                        -85.0,
                        0.0,
                        0.0,
                        IDX_07,
                        2,
                    );
                    let temp2 = self.pane_trance_rec_info(2,
                        self.timer - param_1,
                        IDX_28,
                        0.0,
                        -85.0,
                        0.0,
                        0.0,
                        IDX_07,
                        2,
                    );
                    let temp3 = self.pane_trance_rec_tlt(0,
                        self.timer - param_2,
                        IDX_28,
                        span,
                        0.0,
                        0.0,
                        0.0,
                        1,
                        0,
                    );
                    let temp4 = self.pane_trance_rec_info(0,
                        self.timer - param_2,
                        IDX_28,
                        span,
                        0.0,
                        0.0,
                        0.0,
                        1,
                        0,
                    );
                    temp1 && temp2 && temp3 && temp4 && temp5 && temp6
                }
            }
            _ => {
                let temp5 = self.pane_trance_rec_tlt(2,
                    self.timer - param_1,
                    IDX_28,
                    0.0,
                    -170.0,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                let temp6 = self.pane_trance_rec_info(2,
                    self.timer - param_1,
                    IDX_28,
                    0.0,
                    -170.0,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                if self.flag_3925 == 0 {
                    let temp1 = self.pane_trance_rec_tlt(0,
                        self.timer - param_1,
                        IDX_28,
                        0.0,
                        85.0,
                        0.0,
                        0.0,
                        IDX_07,
                        2,
                    );
                    let temp2 = self.pane_trance_rec_info(0,
                        self.timer - param_1,
                        IDX_28,
                        0.0,
                        85.0,
                        0.0,
                        0.0,
                        IDX_07,
                        2,
                    );
                    let temp3 = self.pane_trance_rec_tlt(1,
                        self.timer - param_2,
                        IDX_28,
                        span,
                        0.0,
                        0.0,
                        0.0,
                        1,
                        0,
                    );
                    let temp4 = self.pane_trance_rec_info(1,
                        self.timer - param_2,
                        IDX_28,
                        span,
                        0.0,
                        0.0,
                        0.0,
                        1,
                        0,
                    );
                    temp1 && temp2 && temp3 && temp4 && temp5 && temp6
                } else {
                    let temp3 = self.pane_trance_rec_tlt(0,
                        self.timer - param_2,
                        IDX_28,
                        span,
                        0.0,
                        0.0,
                        0.0,
                        1,
                        0,
                    );
                    let temp4 = self.pane_trance_rec_info(0,
                        self.timer - param_2,
                        IDX_28,
                        span,
                        0.0,
                        0.0,
                        0.0,
                        1,
                        0,
                    );
                    temp3 && temp4 && temp5 && temp6
                }
            }
        }
    }
    pub fn data_select_pane_copy_move2(&mut self, param_1: i16, param_2: i16) -> bool {
        let span = IDX_40 as f32;
        match self.flag_3924 {
            0 => {
                if self.flag_3925 == 0 {
                    let temp1 = self.pane_trance_rec_tlt(1,
                        self.timer - param_2,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY,
                        0.0,
                        0.0,
                        0,
                        2,
                    );
                    let temp2 = self.pane_trance_rec_info(1,
                        self.timer - param_2,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY,
                        0.0,
                        0.0,
                        0,
                        2,
                    );
                    let temp3 = self.pane_trance_rec_tlt(2,
                        self.timer - param_1,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY,
                        span,
                        SLOT_COPY_DY,
                        0,
                        1,
                    );
                    let temp4 = self.pane_trance_rec_info(2,
                        self.timer - param_1,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY,
                        span,
                        SLOT_COPY_DY,
                        0,
                        1,
                    );
                    temp1 && temp2 && temp3 && temp4
                } else {
                    let temp1 = self.pane_trance_rec_tlt(1,
                        self.timer - param_1,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY,
                        span,
                        SLOT_COPY_DY,
                        0,
                        1,
                    );
                    let temp2 = self.pane_trance_rec_info(1,
                        self.timer - param_1,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY,
                        span,
                        SLOT_COPY_DY,
                        0,
                        1,
                    );
                    let temp3 = self.pane_trance_rec_tlt(2,
                        self.timer - param_2,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY,
                        0.0,
                        -85.0,
                        IDX_07,
                        2,
                    );
                    let temp4 = self.pane_trance_rec_info(2,
                        self.timer - param_2,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY,
                        0.0,
                        -85.0,
                        IDX_07,
                        2,
                    );
                    temp1 && temp2 && temp3 && temp4
                }
            }
            1 => {
                if self.flag_3925 == 0 {
                    let temp1 = self.pane_trance_rec_tlt(2,
                        self.timer - param_1,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY,
                        span,
                        SLOT_COPY_DY,
                        0,
                        1,
                    );
                    let temp2 = self.pane_trance_rec_info(2,
                        self.timer - param_1,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY,
                        span,
                        SLOT_COPY_DY,
                        0,
                        1,
                    );
                    let temp3 = self.pane_trance_rec_tlt(0,
                        self.timer - param_2,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY_ALT,
                        0.0,
                        85.0,
                        0,
                        2,
                    );
                    let temp4 = self.pane_trance_rec_info(0,
                        self.timer - param_2,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY_ALT,
                        0.0,
                        85.0,
                        0,
                        2,
                    );
                    temp1 && temp2 && temp3 && temp4
                } else {
                    let temp1 = self.pane_trance_rec_tlt(0,
                        self.timer - param_1,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY_ALT,
                        span,
                        SLOT_COPY_DY_ALT,
                        0,
                        1,
                    );
                    let temp2 = self.pane_trance_rec_info(0,
                        self.timer - param_1,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY_ALT,
                        span,
                        SLOT_COPY_DY_ALT,
                        0,
                        1,
                    );
                    let temp3 = self.pane_trance_rec_tlt(2,
                        self.timer - param_2,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY,
                        0.0,
                        -85.0,
                        IDX_07,
                        2,
                    );
                    let temp4 = self.pane_trance_rec_info(2,
                        self.timer - param_2,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY,
                        0.0,
                        -85.0,
                        IDX_07,
                        2,
                    );
                    temp1 && temp2 && temp3 && temp4
                }
            }
            _ => {
                if self.flag_3925 == 0 {
                    let temp1 = self.pane_trance_rec_tlt(1,
                        self.timer - param_1,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY_ALT,
                        span,
                        SLOT_COPY_DY_ALT,
                        0,
                        1,
                    );
                    let temp2 = self.pane_trance_rec_info(1,
                        self.timer - param_1,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY_ALT,
                        span,
                        SLOT_COPY_DY_ALT,
                        0,
                        1,
                    );
                    let temp3 = self.pane_trance_rec_tlt(0,
                        self.timer - param_2,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY_ALT,
                        0.0,
                        85.0,
                        0,
                        2,
                    );
                    let temp4 = self.pane_trance_rec_info(0,
                        self.timer - param_2,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY_ALT,
                        0.0,
                        85.0,
                        0,
                        2,
                    );
                    temp1 && temp2 && temp3 && temp4
                } else {
                    let temp1 = self.pane_trance_rec_tlt(0,
                        self.timer - param_1,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY_ALT,
                        span,
                        SLOT_COPY_DY_ALT,
                        0,
                        1,
                    );
                    let temp2 = self.pane_trance_rec_info(0,
                        self.timer - param_1,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY_ALT,
                        span,
                        SLOT_COPY_DY_ALT,
                        0,
                        1,
                    );
                    let temp3 = self.pane_trance_rec_tlt(1,
                        self.timer - param_2,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY_ALT,
                        0.0,
                        0.0,
                        IDX_07,
                        2,
                    );
                    let temp4 = self.pane_trance_rec_info(1,
                        self.timer - param_2,
                        IDX_28,
                        0.0,
                        SLOT_COPY_DY_ALT,
                        0.0,
                        0.0,
                        IDX_07,
                        2,
                    );
                    temp1 && temp2 && temp3 && temp4
                }
            }
        }
    }

    pub fn data_select_pane_copy_color_move(&mut self, param_1: i16, param_2: u8, param_3: i32) -> bool {
        if param_1 < 0 {
            return false;
        }

        if param_1 > i16::from(param_2) {
            return true;
        }

        let tempf = value_increase(i32::from(param_2), i32::from(param_1), 0);

        let mut temp5 = [Color::default(); 5];
        let mut temp7 = [Color::default(); 5];
        let mut temp6 = [Color::default(); 5];
        let mut temp8 = [Color::default(); 5];
        let mut temp = [Color::default(); 9];
        let mut temp2 = [Color::default(); 9];
        let mut temp3 = [Color::default(); 9];
        let mut temp4 = [Color::default(); 9];

        if param_3 == 1 {
            for index in 0..5usize {
                temp5[index] = mix(self.colors.slot_black[index], TINT_KEY, tempf);
                temp7[index] = mix(self.colors.slot_white[index], TINT_WHITE_KEY, tempf);
                temp6[index] = mix(self.colors.slot_black[index + 5], TINT_KEY_ALT, tempf);
                temp8[index] = mix(self.colors.slot_white[index + 5], TINT_INFO, tempf);
            }

            let temp9 = zero(self.colors.slot_black[10], tempf);
            let temp10 = mix(self.colors.slot_white[10], TINT_WHITE_KEY, tempf);

            for index in 0..9usize {
                temp[index] = mix(self.colors.info_black[index], TINT_INFO, tempf);
                temp3[index] = mix(self.colors.info_white[index], TINT_INFO, tempf);
                temp2[index] = mix(self.colors.info_black[index + 9], TINT_KEY_ALT, tempf);
                temp4[index] = mix(self.colors.info_white[index + 9], TINT_INFO, tempf);
            }

            self.apply_copy_colors(temp9, temp10, &temp5, &temp7, &temp6, &temp8, &temp, &temp3, &temp2, &temp4);
        } else {
            for index in 0..5usize {
                temp5[index] = mix_back(TINT_KEY, self.colors.slot_black[index], tempf);
                temp7[index] = mix_back(TINT_WHITE_KEY, self.colors.slot_white[index], tempf);
                temp6[index] = mix_back(TINT_KEY_ALT, self.colors.slot_black[index + 5], tempf);
                temp8[index] = mix_back(TINT_INFO, self.colors.slot_white[index + 5], tempf);
            }

            let temp9 = zero(self.colors.slot_black[10], tempf);
            let temp10 = mix_back(TINT_WHITE_KEY, self.colors.slot_white[10], tempf);

            for index in 0..9usize {
                temp[index] = mix_back(TINT_INFO, self.colors.info_black[index], tempf);
                temp3[index] = mix_back(TINT_INFO, self.colors.info_white[index], tempf);
                temp2[index] = mix_back(TINT_KEY_ALT, self.colors.info_black[index + 9], tempf);
                temp4[index] = mix_back(TINT_INFO, self.colors.info_white[index + 9], tempf);
            }

            self.apply_copy_colors(temp9, temp10, &temp5, &temp7, &temp6, &temp8, &temp, &temp3, &temp2, &temp4);
        }

        false
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_copy_colors(
        &mut self,
        temp9: Color,
        temp10: Color,
        temp5: &[Color; 5],
        temp7: &[Color; 5],
        temp6: &[Color; 5],
        temp8: &[Color; 5],
        temp: &[Color; 9],
        temp3: &[Color; 9],
        temp2: &[Color; 9],
        temp4: &[Color; 9],
    ) {
        let slot = usize::from(self.flag_3924).min(SLOT_COUNT - 1);
        self.panes.slots.key[slot][1].set_black_white(temp9, temp10);
        for index in 2..7usize {
            self.panes.slots.key[slot][index].set_black_white(temp5[index - 2], temp7[index - 2]);
            self.panes.slots.key[slot][index + 5]
                .set_black_white(temp6[index - 2], temp8[index - 2]);
        }
        for index in 3..0xCusize {
            self.panes.slots.info[slot][index].set_black_white(temp[index - 3], temp3[index - 3]);
            self.panes.slots.info[slot][index + 9]
                .set_black_white(temp2[index - 3], temp4[index - 3]);
        }
    }

    pub fn data_select_pane_back_from_copy_move(&mut self, param_1: u8, param_2: u8) -> bool {
        let (temp1, temp2, temp3, temp4, temp5, temp6) = match self.flag_3924 {
            0 => {
                let temp1 = self.pane_trance_rec_tlt(1,
                    self.timer - i16::from(param_1),
                    IDX_28,
                    0.0,
                    SLOT_COPY_DY,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                let temp2 = self.pane_trance_rec_info(1,
                    self.timer - i16::from(param_1),
                    IDX_28,
                    0.0,
                    SLOT_COPY_DY,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                let temp3 = self.pane_trance_rec_tlt(2,
                    self.timer - (i16::from(param_1) + i16::from(param_2)),
                    IDX_28,
                    0.0,
                    SLOT_COPY_DY,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                let temp4 = self.pane_trance_rec_info(2,
                    self.timer - (i16::from(param_1) + i16::from(param_2)),
                    IDX_28,
                    0.0,
                    SLOT_COPY_DY,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                (temp1, temp2, temp3, temp4, true, true)
            }
            1 => {
                let temp1 = self.pane_trance_rec_tlt(0,
                    self.timer - i16::from(param_1),
                    IDX_28,
                    0.0,
                    SLOT_COPY_DY_ALT,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                let temp2 = self.pane_trance_rec_info(0,
                    self.timer - i16::from(param_1),
                    IDX_28,
                    0.0,
                    SLOT_COPY_DY_ALT,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                let temp3 = self.pane_trance_rec_tlt(1,
                    self.timer - (i16::from(param_1) + i16::from(param_2)),
                    IDX_28,
                    0.0,
                    -85.0,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                let temp4 = self.pane_trance_rec_info(1,
                    self.timer - (i16::from(param_1) + i16::from(param_2)),
                    IDX_28,
                    0.0,
                    -85.0,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                let temp5 = self.pane_trance_rec_tlt(2,
                    self.timer - (i16::from(param_1) + i16::from(param_2) * 2),
                    IDX_28,
                    0.0,
                    SLOT_COPY_DY,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                let temp6 = self.pane_trance_rec_info(2,
                    self.timer - (i16::from(param_1) + i16::from(param_2) * 2),
                    IDX_28,
                    0.0,
                    SLOT_COPY_DY,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                (temp1, temp2, temp3, temp4, temp5, temp6)
            }
            _ => {
                let temp1 = self.pane_trance_rec_tlt(0,
                    self.timer - i16::from(param_1),
                    IDX_28,
                    0.0,
                    SLOT_COPY_DY_ALT,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                let temp2 = self.pane_trance_rec_info(0,
                    self.timer - i16::from(param_1),
                    IDX_28,
                    0.0,
                    SLOT_COPY_DY_ALT,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                let temp3 = self.pane_trance_rec_tlt(1,
                    self.timer - (i16::from(param_1) + i16::from(param_2)),
                    IDX_28,
                    0.0,
                    SLOT_COPY_DY_ALT,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                let temp4 = self.pane_trance_rec_info(1,
                    self.timer - (i16::from(param_1) + i16::from(param_2)),
                    IDX_28,
                    0.0,
                    SLOT_COPY_DY_ALT,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                let temp5 = self.pane_trance_rec_tlt(2,
                    self.timer - (i16::from(param_1) + i16::from(param_2) * 2),
                    IDX_28,
                    0.0,
                    -170.0,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                let temp6 = self.pane_trance_rec_info(2,
                    self.timer - (i16::from(param_1) + i16::from(param_2) * 2),
                    IDX_28,
                    0.0,
                    -170.0,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                (temp1, temp2, temp3, temp4, temp5, temp6)
            }
        };
        temp1 && temp2 && temp3 && temp4 && temp5 && temp6
    }

    pub fn yes_no_select(&mut self, pad: &PadInput) {
        self.stick
            .check_trigger(pad.stick_value, pad.stick_angle);
        if pad.trig_a {
            if self.menu_pos == 0 {
                self.sound(SE_MENU_DATA_START);
                self.flag_3941 = 1;
            } else {
                self.sound(SE_MENU_CANCEL);
                self.icon_mode = ICON_MODE_BACK;
            }

            self.panes.groups.menu_cursor.hide();
            self.message_clear(self.flag_392d as usize ^ 1);
            self.proc = proc_cmd_exec_pane_move0;
        } else if pad.trig_b {
            self.sound(SE_MENU_CANCEL);
            self.icon_mode = ICON_MODE_BACK;
            self.panes.groups.menu_cursor.hide();
            self.menu_pos = 1;
            self.proc = proc_cmd_exec_pane_move0;
            self.message_clear(self.flag_392d as usize ^ 1);
        } else if self.stick.check_right_trigger() {
            if self.menu_pos != 1 {
                self.sound(SE_MSEL_CURSOR);
                self.menu_pos = 1;
                self.timer = 0;
                self.yes_no_cursor_move();
            }
        } else if self.stick.check_left_trigger() {
            if self.menu_pos != 0 {
                self.sound(SE_MSEL_CURSOR);
                self.menu_pos = 0;
                self.timer = 0;
                self.yes_no_cursor_move();
            }
        }

        self.menu_cursor_anime();
    }

    pub fn yes_no_cursor_move(&mut self) {
        let x = YES_NO_X[usize::from(self.menu_pos & 1)];
        self.panes.groups.menu_cursor.pane_trans(x, 0.0);
    }

    pub fn cmd_exec_pane_move0(&mut self) {
        let target = if self.op == OP_ERASE {
            self.select_num
        } else {
            self.flag_3926
        };
        let temp2 = if self.data_broken[usize::from(target)] != 0 {
            self.message_pane_move(0.0, self.extra_text_y, IDX_46 as f32, self.extra_text_y, IDX_31 as i16, 0, 1)
        } else {
            self.rec_info_pane_move(0.0, self.extra_text_y, IDX_44 as f32, self.extra_text_y, IDX_31 as i16, 0, 1, )
        };

        let temp3 = self.yes_no_pane_move(0.0, IDX_42 as f32, IDX_36 as i16, 0, 1);
        let temp4 = self.pane_alpha_title_txt(self.timer - IDX_29 as i16, IDX_09);

        if self.op == OP_COPY {
            self.data_select_pane_copy_color_move(self.timer, IDX_0a, 0);
        }

        self.timer += 1;

        if temp2 && temp3 && temp4 {
            self.flag_392d ^= 1;
            self.timer = 0;
            if self.menu_pos == 0 {
                if self.op == OP_ERASE {
                    self.message_set(self.flag_392d as usize ^ 1, MSG_ERASE_OK);
                } else if self.op == OP_COPY {
                    self.message_set(self.flag_392d as usize ^ 1, MSG_COPY_OK);
                }

                self.message_set(2, MSG_CONFIRM_DONE);
                self.message_pane_move_set(IDX_44 as f32, self.extra_text_y, 0.0, self.extra_text_y, 1, 0);
                self.proc = proc_cmd_exec_pane_move2;
            } else {
                self.flag_3929 = 0xFF;
                self.message_set(self.flag_392d as usize ^ 1, MSG_CANCEL);
                self.proc = proc_back_dat_sel_pane_move;
            }
        }
    }

    pub fn cmd_exec_pane_move2(&mut self) {
        let temp1 = self.message_pane_move(IDX_44 as f32, self.extra_text_y, 0.0, self.extra_text_y, IDX_31 as i16, 1, 0, );
        let temp2 = self.pane_alpha_title_txt(self.timer - IDX_29 as i16, IDX_09);
        if temp1 && temp2 {
            self.flag_392d ^= 1;
            self.proc = proc_command_exec;
        } else {
            self.timer += 1;
        }
    }

    pub fn command_exec(&mut self, card: &mut MemCard) {
        if self.op == OP_ERASE {
            let slot = usize::from(self.select_num);
            card.erase_slot(slot);
            card.set_checksum_game_data(slot);
            card.save();
            self.proc = proc_data_erase_wait;
        } else if self.op == OP_COPY {
            let from = usize::from(self.flag_3924);
            let to = usize::from(self.flag_3926);
            if let Some(save) = card.slots[from].clone() {
                card.slots[to] = Some(save);
                card.clear_count[to] = card.clear_count[from];
                card.owl_link[to] = card.owl_link[from];
            }
            card.set_checksum_game_data(to);
            card.set_copy_to_pos(self.flag_3926);
            card.save();
            self.proc = proc_data_copy_wait;
        }
    }

    pub fn data_erase_wait(&mut self, card: &mut MemCard) {
        let temp = card.save_sync();
        if temp != 0 {
            self.sound(SE_MENU_DATA_END);

            if temp == 2 {
                self.timer = 0;
                self.flag_3929 = 1;
                self.extra_text_y = 0.0;
                self.message_clear(self.flag_392d as usize ^ 1);
                self.proc = proc_error_msg_pane_move;
            } else if temp == 1 {
                self.flag_3941 = 0;
                self.timer = 0;
                self.flag_3929 = 0;
                card.set_pict_write_data_ptr(None);
                self.message_set(self.flag_392d as usize ^ 1, MSG_ERASE_DONE);
                self.proc = proc_erase_pane_move_ok;
            }
        }
    }

    pub fn data_copy_wait(&mut self, card: &mut MemCard) {
        let temp = card.save_sync();
        if temp != 0 {
            self.sound(SE_MENU_DATA_END);

            if temp == 2 {
                self.flag_3929 = 1;
                self.timer = 0;
                self.extra_text_y = SLOT_ROW_DY;
                self.message_clear(self.flag_392d as usize ^ 1);
                self.proc = proc_error_msg_pane_move;
            } else if temp == 1 {
                self.flag_3941 = 0;
                self.flag_3929 = 0;
                self.timer = 0;
                card.set_pict_write_data_ptr(None);
                self.message_set(self.flag_392d as usize ^ 1, MSG_COPY_DONE);
                self.proc = proc_copy_pane_move_ok;
            }
        }
    }

    pub fn erase_pane_move_ok(&mut self, card: &mut MemCard) {
        let temp1 = match self.select_num {
            0 => self.pane_trance_rec_info(0,
                self.timer - IDX_3a as i16,
                IDX_11,
                0.0,
                0.0,
                0.0,
                IDX_3e as f32,
                IDX_07,
                1,
            ),
            1 => self.pane_trance_rec_info(1,
                self.timer - IDX_3a as i16,
                IDX_12,
                0.0,
                -85.0,
                0.0,
                IDX_3e as f32 - 85.0,
                IDX_07,
                1,
            ),
            _ => self.pane_trance_rec_info(2,
                self.timer - IDX_3a as i16,
                IDX_13,
                0.0,
                -170.0,
                0.0,
                IDX_3e as f32 - 170.0,
                IDX_07,
                1,
            ),
        };
        let temp2 = self.pane_alpha_title_txt(self.timer - IDX_29 as i16, IDX_09);
        self.timer += 1;

        if temp1 && temp2 {
            self.set_save_data(card);
            self.flag_392d ^= 1;
            self.timer = 0;
            self.proc = proc_erase_pane_move_ok2;
        }
    }

    pub fn erase_pane_move_ok2(&mut self) {
        let temp1 = match self.select_num {
            0 => self.pane_trance_rec_info(0,
                self.timer - IDX_3b as i16,
                IDX_11,
                IDX_3c as f32,
                0.0,
                0.0,
                0.0,
                IDX_07,
                0,
            ),
            1 => self.pane_trance_rec_info(1,
                self.timer - IDX_3b as i16,
                IDX_12,
                IDX_3c as f32,
                -85.0,
                0.0,
                -85.0,
                IDX_07,
                0,
            ),
            _ => self.pane_trance_rec_info(2,
                self.timer - IDX_3b as i16,
                IDX_13,
                IDX_3c as f32,
                -170.0,
                0.0,
                -170.0,
                IDX_07,
                0,
            ),
        };
        let temp2 = self.message_pane_move(0.0, 0.0, IDX_46 as f32, 0.0, IDX_3b as i16, 1, 1, );
        self.timer += 1;

        if temp1 && temp2 {
            self.timer = 0;
            self.proc = proc_back_dat_sel_wait;
        }
    }

    pub fn error_msg_pane_move(&mut self) {
        let temp1 = self.message_pane_move(0.0, self.extra_text_y, IDX_46 as f32, self.extra_text_y, IDX_31 as i16, 0, 1, );
        let temp2 = self.pane_alpha_title_txt(self.timer - IDX_29 as i16, IDX_09);
        self.timer += 1;
        if temp1 && temp2 {
            self.flag_392d ^= 1;

            let msg_no = if self.op == OP_ERASE {
                MSG_ERASE_FAIL
            } else {
                MSG_COPY_FAIL
            };

            self.message_set(2, msg_no);
            self.message_pane_move_set(IDX_44 as f32, self.extra_text_y, 0.0, self.extra_text_y, 1, 0);
            self.timer = 0;
            self.proc = proc_error_msg_pane_move2;
        }
    }

    pub fn error_msg_pane_move2(&mut self) {
        let temp = self.message_pane_move(IDX_44 as f32, self.extra_text_y, 0.0, self.extra_text_y, IDX_31 as i16, 1, 0, );
        self.timer += 1;
        if temp {
            self.timer = 0;
            self.proc = proc_back_dat_sel_wait2;
        }
    }

    pub fn back_dat_sel_pane_move(&mut self) {
        let mut temp1 = true;
        let mut temp2 = true;
        let mut temp3 = true;

        if self.op == OP_ERASE {
            temp1 = self.data_select_pane_back_move(IDX_29 as i16, IDX_2a as i16, IDX_29 as i16);

            if self.flag_3929 == 1 {
                temp2 = self.message_pane_move(0.0, 0.0, IDX_46 as f32, 0.0, IDX_31 as i16, 0, 1);
            }
        } else if self.op == OP_COPY {
            temp1 = self.data_select_pane_copy_back_move(IDX_29 as i16, IDX_2a as i16);

            if self.flag_3929 != 0xFF {
                if self.flag_3929 != 0 {
                    temp2 =
                        self.message_pane_move(0.0, 85.0, IDX_46 as f32, 85.0, IDX_31 as i16, 0, 1);
                } else {
                    temp3 = self.rec_info_pane_move(0.0, 85.0, IDX_44 as f32, 85.0, IDX_31 as i16, 0, 1);
                }
            }
        }

        let temp4 = self.pane_alpha_title_txt(self.timer - IDX_29 as i16, IDX_09);
        self.timer += 1;

        if temp1 && temp2 && temp3 && temp4 {
            self.flag_392d ^= 1;
            self.timer2 = IDX_4a;
            self.panes.groups.record_cursor[0].show();
            self.rec_cursor_move();
            self.proc = proc_data_select;
            self.op = OP_NONE;
        }
    }

    pub fn back_dat_sel_wait(&mut self, pad: &PadInput) {
        if self.timer2 != 0 {
            self.timer2 -= 1;
        }

        if pad.any_confirm() || self.timer2 == 0 {
            self.timer = 0;
            self.message_set(self.flag_392d as usize ^ 1, MSG_CANCEL);
            self.icon_mode = ICON_MODE_BACK;
            self.proc = proc_back_dat_sel_pane_move;
        }
    }

    pub fn back_dat_sel_wait2(&mut self, pad: &PadInput) {
        if self.timer2 != 0 {
            self.timer2 -= 1;
        }

        if pad.any_confirm() || self.timer2 == 0 {
            self.flag_392e = 1;
            self.timer = 0;
            self.select_end = SELECT_END_CANCEL;
            self.icon_mode = ICON_MODE_DEFAULT;
            self.proc = proc_next_mode_wait;
        }
    }

    pub fn data_select_pane_back_move(&mut self, param_1: i16, param_2: i16, param_3: i16) -> bool {
        let span = IDX_40 as f32;
        match self.select_num {
            0 => {
                let temp3 = self.pane_trance_rec_tlt(1,
                    self.timer - param_1,
                    IDX_28,
                    span,
                    0.0,
                    0.0,
                    0.0,
                    1,
                    0,
                );
                let temp4 = self.pane_trance_rec_info(1,
                    self.timer - param_1,
                    IDX_28,
                    span,
                    0.0,
                    0.0,
                    0.0,
                    1,
                    0,
                );
                let temp5 = self.pane_trance_rec_tlt(2,
                    self.timer - param_2,
                    IDX_28,
                    span,
                    0.0,
                    0.0,
                    0.0,
                    1,
                    0,
                );
                let temp6 = self.pane_trance_rec_info(2,
                    self.timer - param_2,
                    IDX_28,
                    span,
                    0.0,
                    0.0,
                    0.0,
                    1,
                    0,
                );
                temp3 && temp4 && temp5 && temp6
            }
            1 => {
                let temp1 = self.pane_trance_rec_tlt(1,
                    self.timer - param_3,
                    IDX_28,
                    0.0,
                    -85.0,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                let temp2 = self.pane_trance_rec_info(1,
                    self.timer - param_3,
                    IDX_28,
                    0.0,
                    -85.0,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                let temp3 = self.pane_trance_rec_tlt(2,
                    self.timer - param_2,
                    IDX_28,
                    span,
                    0.0,
                    0.0,
                    0.0,
                    1,
                    0,
                );
                let temp4 = self.pane_trance_rec_info(2,
                    self.timer - param_2,
                    IDX_28,
                    span,
                    0.0,
                    0.0,
                    0.0,
                    1,
                    0,
                );
                let temp5 = self.pane_trance_rec_tlt(0,
                    self.timer - param_1,
                    IDX_28,
                    span,
                    0.0,
                    0.0,
                    0.0,
                    1,
                    0,
                );
                let temp6 = self.pane_trance_rec_info(0,
                    self.timer - param_1,
                    IDX_28,
                    span,
                    0.0,
                    0.0,
                    0.0,
                    1,
                    0,
                );
                temp1 && temp2 && temp3 && temp4 && temp5 && temp6
            }
            _ => {
                let temp1 = self.pane_trance_rec_tlt(2,
                    self.timer - param_3,
                    IDX_28,
                    0.0,
                    -170.0,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                let temp2 = self.pane_trance_rec_info(2,
                    self.timer - param_3,
                    IDX_28,
                    0.0,
                    -170.0,
                    0.0,
                    0.0,
                    IDX_07,
                    2,
                );
                let temp3 = self.pane_trance_rec_tlt(0,
                    self.timer - param_1,
                    IDX_28,
                    span,
                    0.0,
                    0.0,
                    0.0,
                    1,
                    0,
                );
                let temp4 = self.pane_trance_rec_info(0,
                    self.timer - param_1,
                    IDX_28,
                    span,
                    0.0,
                    0.0,
                    0.0,
                    1,
                    0,
                );
                let temp5 = self.pane_trance_rec_tlt(1,
                    self.timer - param_2,
                    IDX_28,
                    span,
                    0.0,
                    0.0,
                    0.0,
                    1,
                    0,
                );
                let temp6 = self.pane_trance_rec_info(1,
                    self.timer - param_2,
                    IDX_28,
                    span,
                    0.0,
                    0.0,
                    0.0,
                    1,
                    0,
                );
                temp1 && temp2 && temp3 && temp4 && temp5 && temp6
            }
        }
    }

    pub fn data_select_ex(&mut self, pad: &PadInput) {
        self.stick2
            .check_trigger(pad.stick_value, pad.stick_angle);
        if pad.trig_a || pad.trig_start {
            self.sound(SE_MENU_OK);
            self.icon_mode = ICON_MODE_SELECT;
            self.panes.groups.record_cursor[0].hide();
            self.message_set(self.flag_392d as usize ^ 1, MSG_RETURN_TITLE);
            self.data_sel_move_set();
            self.proc = proc_ex_save_pane_move;
        } else if pad.trig_b {
            self.sound(SE_MENU_CANCEL);
            self.panes.groups.record_cursor[0].hide();
            self.timer = 0;
            self.flag_392e = 0;
            self.select_end = SELECT_END_RETURN;
            self.icon_mode = ICON_MODE_DEFAULT;
            self.proc = proc_next_mode_wait;
        } else if self.stick2.check_up_trigger() {
            if self.select_num != 0 {
                self.sound(SE_MSEL_CURSOR);
                self.select_num -= 1;
                self.slide = 0.0;
                self.title_dir = -1;
                self.timer = 0;
                self.proc = proc_select_title;
                self.rec_cursor_move();
            }
        } else if self.stick2.check_down_trigger() {
            if self.select_num != 2 {
                self.sound(SE_MSEL_CURSOR);
                self.select_num += 1;
                self.slide = 0.0;
                self.title_dir = 1;
                self.timer = 0;
                self.proc = proc_select_title;
                self.rec_cursor_move();
            }
        }

        self.rec_cursor_anime();
    }

    pub fn ex_save_pane_move(&mut self) {
        let temp1 = self.rec_data_pane_move();
        let temp2 = if self.data_broken[self.select_num as usize] != 0 {
            self.message_pane_move(IDX_44 as f32, 0.0, 0.0, 0.0, (IDX_30 as i16) + i16::from(self.flag_3920),
                1,
                0,
            )
        } else {
            self.rec_info_pane_move(IDX_44 as f32, 0.0, 0.0, 0.0, (IDX_30 as i16) + i16::from(self.flag_3920),
                1,
                0,
            )
        };
        let temp3 = self.yes_no_pane_move(IDX_42 as f32, 0.0, IDX_38 as i16, 1, 0);
        let temp4 = self.pane_alpha_title_txt(self.timer - IDX_29 as i16, IDX_09);
        self.timer += 1;

        if temp1 && temp2 && temp3 && temp4 {
            self.flag_392d ^= 1;
            self.timer = 0;
            self.menu_cursor_alpha_init();
            self.panes.groups.menu_cursor.show();
            self.menu_pos = 1;
            self.yes_no_cursor_move();
            self.proc = proc_yes_no_select_ex;
        }
    }

    pub fn yes_no_select_ex(&mut self, pad: &PadInput, card: &MemCard) {
        self.stick
            .check_trigger(pad.stick_value, pad.stick_angle);
        if pad.trig_a {
            if self.menu_pos == 0 {
                self.sound(SE_MENU_DATA_START);
                let result = self.ex_card_check(card);
                if result != 0 {
                    self.flag_392e = 1;
                    self.err_type = result;
                    self.op = OP_CARD_ERR;
                    self.select_end = SELECT_END_CANCEL;
                    self.icon_mode = ICON_MODE_DEFAULT;
                    self.proc = proc_next_mode_wait;
                } else {
                    self.proc = proc_ex_save_pane_move0;
                }
            } else {
                self.sound(SE_MENU_CANCEL);
                self.icon_mode = ICON_MODE_BACK;
                self.proc = proc_ex_save_pane_move0;
            }

            self.panes.groups.menu_cursor.hide();
            self.message_clear(self.flag_392d as usize ^ 1);
        } else if pad.trig_b {
            self.sound(SE_MENU_CANCEL);
            self.icon_mode = ICON_MODE_SELECT;
            self.panes.groups.menu_cursor.hide();
            self.menu_pos = 1;
            self.proc = proc_ex_save_pane_move0;
            self.message_clear(self.flag_392d as usize ^ 1);
        } else if self.stick.check_right_trigger() {
            if self.menu_pos != 1 {
                self.sound(SE_MSEL_CURSOR);
                self.menu_pos = 1;
                self.timer = 0;
                self.yes_no_cursor_move();
            }
        } else if self.stick.check_left_trigger() {
            if self.menu_pos != 0 {
                self.sound(SE_MSEL_CURSOR);
                self.menu_pos = 0;
                self.timer = 0;
                self.yes_no_cursor_move();
            }
        }

        self.menu_cursor_anime();
    }

    pub fn ex_save_pane_move0(&mut self) {
        let temp1 = if self.data_broken[self.select_num as usize] != 0 {
            self.message_pane_move(0.0, self.extra_text_y, IDX_46 as f32, self.extra_text_y, IDX_31 as i16, 0, 1)
        } else {
            self.rec_info_pane_move(0.0, self.extra_text_y, IDX_44 as f32, self.extra_text_y, IDX_31 as i16, 0, 1, )
        };
        let temp2 = self.yes_no_pane_move(0.0, IDX_42 as f32, IDX_36 as i16, 0, 1);
        let temp3 = self.pane_alpha_title_txt(self.timer - IDX_29 as i16, IDX_09);
        self.timer += 1;

        if temp1 && temp2 && temp3 {
            self.flag_392d ^= 1;
            self.timer = 0;
            if self.menu_pos == 0 {
                self.message_set(self.flag_392d as usize ^ 1, MSG_EXTRA_CONFIRM);
                self.message_set(2, MSG_CONFIRM_DONE);
                self.message_pane_move_set(IDX_44 as f32, self.extra_text_y, 0.0, self.extra_text_y, 1, 0);
                self.proc = proc_ex_save_pane_move1;
            } else {
                self.flag_3929 = 0xFF;
                self.message_set(self.flag_392d as usize ^ 1, MSG_EXTRA_BACK);
                self.proc = proc_ex_back_dat_sel_pane_move;
            }
        }
    }

    pub fn ex_card_check(&self, card: &MemCard) -> u8 {
        if card.get_status() != crate::ui_menu_save::CARD_STAT_CREATE {
            return 1;
        }

        if card.get_new_file() == 1 {
            return 0;
        }

        if card.get_mem_card_check_id() != card.get_card_serial_no() {
            return 2;
        }

        if card.get_no_file() != 0 {
            return 2;
        }

        0
    }

    pub fn ex_back_dat_sel_pane_move(&mut self) {
        let mut temp2 = true;
        let temp1 = self.data_select_pane_back_move(IDX_29 as i16, IDX_2a as i16, IDX_29 as i16);
        if self.flag_3929 == 1 {
            temp2 = self.message_pane_move(0.0, 0.0, IDX_46 as f32, 0.0, IDX_31 as i16, 0, 1);
        }
        let temp3 = self.pane_alpha_title_txt(self.timer - IDX_29 as i16, IDX_09);
        self.timer += 1;

        if temp1 && temp2 && temp3 {
            self.flag_392d ^= 1;
            self.timer2 = IDX_4a;
            self.panes.groups.record_cursor[0].show();
            self.rec_cursor_move();
            self.proc = proc_data_select_ex;
        }
    }

    pub fn ex_save_pane_move1(&mut self) {
        let temp1 = self.message_pane_move(IDX_44 as f32, self.extra_text_y, 0.0, self.extra_text_y, IDX_31 as i16, 1, 0, );
        let temp2 = self.pane_alpha_title_txt(self.timer - IDX_29 as i16, IDX_09);

        if temp1 && temp2 {
            self.flag_392d ^= 1;
            self.proc = proc_ex_data_save;
        } else {
            self.timer += 1;
        }
    }

    pub fn ex_data_save(&mut self, card: &mut MemCard, current: Option<SaveGame>) {
        let slot = usize::from(self.select_num);
        card.set_memory_to_card(slot, current.unwrap_or_else(blank_save));
        card.set_checksum_game_data(slot);
        card.set_data_num(self.select_num);
        card.set_pict_write_data_ptr(None);
        card.save();
        self.proc = proc_ex_data_save_wait;
    }

    pub fn ex_data_save_wait(&mut self, card: &mut MemCard) {
        let temp = card.save_sync();
        if temp != 0 {
            if temp == 2 {
                self.sound(SE_ALERT_DATA);

                self.timer = 0;
                self.flag_3929 = 1;
                self.extra_text_y = 0.0;
                self.message_clear(self.flag_392d as usize ^ 1);
                self.proc = proc_ex_error_msg_pane_move;
            } else if temp == 1 {
                self.sound(SE_SAVE_FINISH);

                self.timer = 0;
                self.flag_3929 = 0;
                card.set_pict_write_data_ptr(None);
                self.message_set(self.flag_392d as usize ^ 1, MSG_EXTRA_DONE);
                self.proc = proc_ex_data_save_pane_move_ok;
            }
        }
    }

    pub fn ex_data_save_pane_move_ok(&mut self, card: &mut MemCard) {
        let temp1 = match self.select_num {
            0 => self.pane_trance_rec_info(0,
                self.timer - IDX_3a as i16,
                IDX_11,
                0.0,
                0.0,
                0.0,
                IDX_3e as f32,
                IDX_07,
                1,
            ),
            1 => self.pane_trance_rec_info(1,
                self.timer - IDX_3a as i16,
                IDX_12,
                0.0,
                -85.0,
                0.0,
                IDX_3e as f32 - 85.0,
                IDX_07,
                1,
            ),
            _ => self.pane_trance_rec_info(2,
                self.timer - IDX_3a as i16,
                IDX_13,
                0.0,
                -170.0,
                0.0,
                IDX_3e as f32 - 170.0,
                IDX_07,
                1,
            ),
        };

        let temp2 = self.pane_alpha_title_txt(self.timer - IDX_29 as i16, IDX_09);
        let temp3 = self.message_pane_move(0.0, 0.0, IDX_46 as f32, 0.0, IDX_3b as i16, 1, 1);
        self.timer += 1;

        if temp1 && temp2 && temp3 {
            self.set_save_data(card);
            self.flag_392d ^= 1;
            self.timer = 0;
            self.proc = proc_ex_data_save_pane_move_ok2;
        }
    }

    pub fn ex_data_save_pane_move_ok2(&mut self) {
        let temp1 = match self.select_num {
            0 => self.pane_trance_rec_info(0,
                self.timer - IDX_3b as i16,
                IDX_11,
                IDX_3c as f32,
                0.0,
                0.0,
                0.0,
                IDX_07,
                0,
            ),
            1 => self.pane_trance_rec_info(1,
                self.timer - IDX_3b as i16,
                IDX_12,
                IDX_3c as f32,
                -85.0,
                0.0,
                -85.0,
                IDX_07,
                0,
            ),
            _ => self.pane_trance_rec_info(2,
                self.timer - IDX_3b as i16,
                IDX_13,
                IDX_3c as f32,
                -170.0,
                0.0,
                -170.0,
                IDX_07,
                0,
            ),
        };

        let temp2 = self.rec_info_pane_move(IDX_44 as f32, 0.0, 0.0, 0.0, IDX_30 as i16, 1, 0, );
        self.timer += 1;

        if temp1 && temp2 {
            self.select_end = SELECT_END_START;
            self.slot_picked = Some(usize::from(self.select_num));
        }
    }

    pub fn ex_error_msg_pane_move(&mut self) {
        let temp1 = self.message_pane_move(0.0, self.extra_text_y, IDX_46 as f32, self.extra_text_y, IDX_31 as i16, 0, 1, );
        let temp2 = self.pane_alpha_title_txt(self.timer - IDX_29 as i16, IDX_09);
        self.timer += 1;

        if temp1 && temp2 {
            self.flag_392d ^= 1;
            self.message_set(2, MSG_EXTRA_FAIL);
            self.message_pane_move_set(IDX_44 as f32, self.extra_text_y, 0.0, self.extra_text_y, 1, 0);
            self.timer = 0;
            self.proc = proc_ex_error_msg_pane_move2;
        }
    }

    pub fn ex_error_msg_pane_move2(&mut self) {
        let result = self.message_pane_move(IDX_44 as f32, self.extra_text_y, 0.0, self.extra_text_y, IDX_31 as i16, 1, 0, );

        self.timer += 1;
        if result {
            self.timer = 0;
            self.op = OP_ERASE_EXTRA;
            self.proc = proc_back_dat_sel_wait2;
        }
    }

    pub fn change_extra_color(&mut self, index: usize) {
        if index >= SLOT_COUNT {
            return;
        }
        self.panes.slots.key[index][1].set_black_white(
            Color::new(0, 0, 0, 0),
            Color::new(0xDC, 0xFF, 0x64, 0xFF),
        );
        if self.data_extra[index] == 1 {
            for i in 2..7usize {
                self.panes.slots.key[index][i].set_black_white(
                    Color::from(EXTRA_COLOR_A),
                    Color::from(EXTRA_COLOR_B),
                );
                self.panes.slots.key[index][i + 5].set_black_white(
                    Color::from(EXTRA_COLOR_C),
                    Color::from(EXTRA_COLOR_D),
                );
            }
            for i in 3..12usize {
                self.panes.slots.info[index][i].set_black_white(
                    Color::from(EXTRA_INFO_A),
                    Color::from(EXTRA_INFO_B),
                );
                self.panes.slots.info[index][i + 9].set_black_white(
                    Color::from(EXTRA_INFO_C),
                    Color::from(EXTRA_INFO_D),
                );
            }
            self.panes.slots.slot_mark[index].show();
            self.panes.slots.slot_mark_key[index].show();
        } else {
            for i in 2..7usize {
                self.panes.slots.key[index][i].set_black_white(
                    Color::from(ERASE_COLOR_A),
                    Color::from(ERASE_COLOR_B),
                );
                self.panes.slots.key[index][i + 5].set_black_white(
                    Color::from(ERASE_COLOR_C),
                    Color::from(ERASE_COLOR_D),
                );
            }
            for i in 3..12usize {
                self.panes.slots.info[index][i].set_black_white(
                    Color::from(PLAIN_INFO_A),
                    Color::from(PLAIN_INFO_B),
                );
                self.panes.slots.info[index][i + 9].set_black_white(
                    Color::from(PLAIN_INFO_C),
                    Color::from(PLAIN_INFO_D),
                );
            }
            if self.panes.slots.slot_mark[index].is_visible() {
                self.panes.slots.slot_mark[index].hide();
                self.panes.slots.slot_mark_key[index].hide();
            }
        }
    }

    pub fn change_broken_color(&mut self, index: usize) {
        if index >= SLOT_COUNT {
            return;
        }
        self.panes.slots.key[index][1].set_black_white(
            Color::new(0, 0, 0, 0),
            Color::new(0xDC, 0xFF, 0x64, 0xFF),
        );
        for i in 2..7usize {
            self.panes.slots.key[index][i].set_black_white(
                Color::from(ERASE_COLOR_A),
                Color::from(ERASE_COLOR_B),
            );
            self.panes.slots.key[index][i + 5].set_black_white(
                Color::from(ERASE_COLOR_C),
                Color::from(ERASE_COLOR_D),
            );
        }
        for i in 3..12usize {
            self.panes.slots.info[index][i].set_black_white(
                Color::from(BROKEN_INFO_A),
                Color::from(BROKEN_INFO_B),
            );
            self.panes.slots.info[index][i + 9].set_black_white(
                Color::from(BROKEN_INFO_C),
                Color::from(BROKEN_INFO_D),
            );
        }
        if self.panes.slots.slot_mark[index].is_visible() {
            self.panes.slots.slot_mark[index].hide();
            self.panes.slots.slot_mark_key[index].hide();
        }
    }

    pub fn message_pane_move(
        &mut self,
        x: f32,
        y: f32,
        x2: f32,
        y2: f32,
        delay: i16,
        mode: u8,
        flip: i32,
    ) -> bool {
        self.pane_trance_message_base(
            self.timer - delay,
            IDX_32,
            x,
            y,
            x2,
            y2,
            mode,
            flip,
        )
    }

    pub fn rec_info_pane_move(
        &mut self,
        x: f32,
        y: f32,
        x2: f32,
        y2: f32,
        delay: i16,
        mode: u8,
        flip: i32,
    ) -> bool {
        self.pane_trance_rec_base(
            self.timer - delay,
            IDX_2f,
            x,
            y,
            x2,
            y2,
            mode,
            flip,
        )
    }

    pub fn menu_pane_move(
        &mut self,
        x: f32,
        y: f32,
        delay: i16,
        mode: u8,
        flip: i32,
    ) -> bool {
        let temp1 = self.pane_trance_menu_item(self.timer - delay, IDX_34, x, y, mode, flip, MenuItem::Start);
        let temp2 = self.pane_trance_menu_item(
            self.timer - (delay + IDX_37 as i16),
            IDX_34,
            x,
            y,
            mode,
            flip,
            MenuItem::Copy,
        );
        let temp3 = self.pane_trance_menu_item(
            self.timer - (delay + IDX_37 as i16 * 2),
            IDX_34,
            x,
            y,
            mode,
            flip,
            MenuItem::Erase,
        );
        let temp4 = self.pane_trance_menu_item(
            self.timer - (delay + IDX_37 as i16 * 3),
            IDX_34,
            x,
            y,
            mode,
            flip,
            MenuItem::Back,
        );
        temp1 && temp2 && temp3 && temp4
    }

    pub fn yes_no_pane_move(
        &mut self,
        x: f32,
        y: f32,
        delay: i16,
        mode: u8,
        flip: i32,
    ) -> bool {
        let temp1 =
            self.pane_trance_menu_item(self.timer - delay, IDX_34, x, y, mode, flip, MenuItem::Yes);
        let temp2 = self.pane_trance_menu_item(
            self.timer - (delay + IDX_37 as i16),
            IDX_34,
            x,
            y,
            mode,
            flip,
            MenuItem::No,
        );
        temp1 && temp2
    }

    pub fn pane_trance_title(
        &mut self,
        param_0: i16,
        param_1: u8,
        param_2: f32,
        param_3: f32,
        param_4: u8,
        param_5: i32,
    ) -> bool {
        if param_0 < 0 {
            return false;
        }

        if param_0 > i16::from(param_1) {
            return true;
        }

        let mut value = value_increase(i32::from(param_1), i32::from(param_0), param_4);
        let span = param_3 - param_2;
        self.panes.groups.title_key.pane_trans(0.0, param_2 + value * span);

        if param_5 != 2 {
            if param_5 == 1 {
                value = 1.0 - value;
            }

            let index = usize::from(self.flag_392d);
            self.panes.groups.title_key.set_now_alpha(value);
            self.panes.groups.title_label.set_now_alpha(value);
            self.panes.groups.title_text[index.min(1)].set_now_alpha(value);
        }

        false
    }

    pub fn pane_alpha_title_txt(&mut self, param_0: i16, param_1: u8) -> bool {
        if param_0 < 0 {
            return false;
        }

        if param_0 > i16::from(param_1) {
            return true;
        }

        let value = value_increase(i32::from(param_1), i32::from(param_0), 0);
        let fade = 1.0 - value;
        let index = usize::from(self.flag_392d);
        self.panes.groups.title_text[index.min(1)].set_now_alpha(fade);
        self.panes.groups.title_text[(index ^ 1).min(1)].set_now_alpha(value);
        false
    }

    pub fn pane_trance_rec_tlt(
        &mut self,
        slot: usize,
        param_0: i16,
        param_1: u8,
        param_2: f32,
        param_3: f32,
        param_4: f32,
        param_5: f32,
        param_6: u8,
        param_7: i32,
    ) -> bool {
        if param_0 < 0 {
            return false;
        }

        if param_0 > i16::from(param_1) {
            return true;
        }

        let mut value = value_increase(i32::from(param_1), i32::from(param_0), param_6);
        let dx = param_4 - param_2;
        let dy = param_5 - param_3;
        let slot = slot.min(SLOT_COUNT - 1);
        self.panes.slots.key[slot][0].pane_trans(param_2 + value * dx, param_3 + value * dy);
        self.panes.slots.slot_mark[slot].pane_trans(param_2 + value * dx, param_3 + value * dy);
        self.panes.slots.slot_mark_key[slot].pane_trans(param_2 + value * dx, param_3 + value * dy);

        if param_7 != 2 {
            if param_7 == 1 {
                value = 1.0 - value;
            }

            self.panes.slots.slot_mark[slot].set_now_alpha(value);
            self.panes.slots.slot_mark_key[slot].set_now_alpha(value);
            self.panes.slots.key[slot][0].set_now_alpha(value);
            self.panes.slots.key[slot][1].set_now_alpha(value);
            self.panes.slots.key[slot][2].set_now_alpha(value);
        }

        false
    }

    pub fn pane_trance_rec_info(
        &mut self,
        slot: usize,
        param_0: i16,
        param_1: u8,
        param_2: f32,
        param_3: f32,
        param_4: f32,
        param_5: f32,
        param_6: u8,
        param_7: i32,
    ) -> bool {
        if param_0 < 0 {
            return false;
        }

        if param_0 > i16::from(param_1) {
            return true;
        }

        let mut value = value_increase(i32::from(param_1), i32::from(param_0), param_6);
        let dx = param_4 - param_2;
        let dy = param_5 - param_3;
        let slot = slot.min(SLOT_COUNT - 1);
        self.panes.slots.info[slot][0]
            .pane_trans(param_2 + value * dx, param_3 + value * dy);

        if param_7 != 2 {
            if param_7 == 1 {
                value = 1.0 - value;
            }

            for index in 0..4usize {
                self.panes.slots.info[slot][index].set_now_alpha(value);
            }
        }

        false
    }

    pub fn pane_trance_rec_base(
        &mut self,
        param_0: i16,
        param_1: u8,
        param_2: f32,
        param_3: f32,
        param_4: f32,
        param_5: f32,
        param_6: u8,
        param_7: i32,
    ) -> bool {
        if param_0 < 0 {
            return false;
        }

        if param_0 > i16::from(param_1) {
            return true;
        }

        let mut value = value_increase(i32::from(param_1), i32::from(param_0), param_6);
        let dx = param_4 - param_2;
        let dy = param_5 - param_3;

        for pane in &mut self.panes.slots.record {
            pane.pane_trans(param_2 + value * dx, param_3 + value * dy);
        }
        for pane in &mut self.panes.slots.collect {
            pane.pane_trans(param_2 + value * dx, param_3 + value * dy);
        }
        for index in 0..PEARL_COUNT {
            self.panes.slots.pearl[index].pane_trans(param_2 + value * dx, param_3 + value * dy);
            self.panes.slots.pearl_shadow[index]
                .pane_trans(param_2 + value * dx, param_3 + value * dy);
        }
        self.panes.slots.triforce_bar
            .pane_trans(param_2 + value * dx, param_3 + value * dy);
        for pane in &mut self.panes.slots.shard {
            pane.pane_trans(param_2 + value * dx, param_3 + value * dy);
        }
        for index in 0..0x14usize {
            self.panes.slots.heart[index].pane_trans(param_2 + value * dx, param_3 + value * dy);
            self.panes.slots.heart_shadow[index]
                .pane_trans(param_2 + value * dx, param_3 + value * dy);
        }

        if param_7 == 1 {
            value = 1.0 - value;
        }

        for pane in &mut self.panes.slots.record {
            pane.set_now_alpha(value);
        }
        for pane in &mut self.panes.slots.collect {
            pane.set_now_alpha(value);
        }
        for index in 0..PEARL_COUNT {
            self.panes.slots.pearl[index].set_now_alpha(value);
            self.panes.slots.pearl_shadow[index].set_now_alpha(value);
        }
        self.panes.slots.triforce_bar.set_now_alpha(value);
        for pane in &mut self.panes.slots.shard {
            pane.set_now_alpha(value);
        }
        for index in 0..0x14usize {
            self.panes.slots.heart[index].set_now_alpha(value);
            self.panes.slots.heart_shadow[index].set_now_alpha(value);
        }

        false
    }

    pub fn pane_trance_message_base(
        &mut self,
        param_0: i16,
        param_1: u8,
        param_2: f32,
        param_3: f32,
        param_4: f32,
        param_5: f32,
        param_6: u8,
        param_7: i32,
    ) -> bool {
        if param_0 < 0 {
            return false;
        }

        if param_0 > i16::from(param_1) {
            return true;
        }

        let mut value = value_increase(i32::from(param_1), i32::from(param_0), param_6);
        let dx = param_4 - param_2;
        let dy = param_5 - param_3;
        self.panes.groups.message_key[0]
            .pane_trans(param_2 + value * dx, param_3 + value * dy);

        if param_7 == 1 {
            value = 1.0 - value;
        }

        self.panes.groups.message_text.set_now_alpha(value);
        self.panes.groups.message_key[0].set_now_alpha(value);
        self.panes.groups.message_key[1].set_now_alpha(value);

        false
    }

    pub fn pane_trance_menu_item(
        &mut self,
        param_0: i16,
        param_1: u8,
        param_2: f32,
        param_3: f32,
        param_4: u8,
        param_5: i32,
        item: MenuItem,
    ) -> bool {
        if param_0 < 0 {
            return false;
        }

        if param_0 > i16::from(param_1) {
            return true;
        }

        let mut value = value_increase(i32::from(param_1), i32::from(param_0), param_4);
        let span = param_3 - param_2;
        let group: &mut [Pane; 2] = match item {
            MenuItem::Yes => &mut self.panes.groups.yes_key,
            MenuItem::No => &mut self.panes.groups.no_key,
            MenuItem::Start => &mut self.panes.groups.start_key,
            MenuItem::Copy => &mut self.panes.groups.copy_key,
            MenuItem::Erase => &mut self.panes.groups.erase_key,
            MenuItem::Back => &mut self.panes.groups.back_key,
        };
        let label = match item {
            MenuItem::Yes => &mut self.panes.groups.yes_label,
            MenuItem::No => &mut self.panes.groups.no_label,
            MenuItem::Start => &mut self.panes.groups.start_label,
            MenuItem::Copy => &mut self.panes.groups.copy_label,
            MenuItem::Erase => &mut self.panes.groups.erase_label,
            MenuItem::Back => &mut self.panes.groups.back_label,
        };
        let text = match item {
            MenuItem::Yes => Some(&mut self.panes.groups.yes_text),
            MenuItem::No => Some(&mut self.panes.groups.no_text),
            MenuItem::Start => None,
            MenuItem::Copy => None,
            MenuItem::Erase => None,
            MenuItem::Back => Some(&mut self.panes.groups.back_text),
        };
        group[0].pane_trans(0.0, param_2 + value * span);
        if param_5 == 1 {
            value = 1.0 - value;
        }
        group[0].set_now_alpha(value);
        label.set_now_alpha(value);
        if let Some(pane) = text {
            pane.set_now_alpha(value);
        }
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuItem {
    Yes,
    No,
    Start,
    Copy,
    Erase,
    Back,
}

fn mix(base: Color, target: f32, value: f32) -> Color {
    Color {
        r: (f32::from(base.r) - (f32::from(base.r) - target) * value) as u8,
        g: (f32::from(base.g) - (f32::from(base.g) - target) * value) as u8,
        b: (f32::from(base.b) - (f32::from(base.b) - target) * value) as u8,
        a: base.a,
    }
}

fn mix_back(target: f32, base: Color, value: f32) -> Color {
    Color {
        r: (target - (target - f32::from(base.r)) * value) as u8,
        g: (target - (target - f32::from(base.g)) * value) as u8,
        b: (target - (target - f32::from(base.b)) * value) as u8,
        a: base.a,
    }
}

fn zero(base: Color, value: f32) -> Color {
    Color {
        r: (f32::from(base.r) - f32::from(base.r) * value) as u8,
        g: (f32::from(base.g) - f32::from(base.g) * value) as u8,
        b: (f32::from(base.b) - f32::from(base.b) * value) as u8,
        a: base.a,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Calendar {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

pub fn ticks_to_calendar(tick: u64) -> Calendar {
    let seconds = (tick / SIM_HZ) as i64;
    let days = seconds.div_euclid(86_400);
    let rest = seconds.rem_euclid(86_400);
    let mut year = 2000;
    let mut remaining = days;
    loop {
        let leap = is_leap(year);
        let len = if leap { 366 } else { 365 };
        if remaining < len {
            break;
        }
        remaining -= len;
        year += 1;
    }
    let month_lengths = month_table(is_leap(year));
    let mut month = 0usize;
    while month < 12 && remaining >= month_lengths[month] as i64 {
        remaining -= month_lengths[month] as i64;
        month += 1;
    }
    Calendar {
        year,
        month: month as u32 + 1,
        day: remaining as u32 + 1,
        hour: (rest / 3600) as u32,
        minute: ((rest % 3600) / 60) as u32,
        second: (rest % 60) as u32,
    }
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn month_table(leap: bool) -> [u32; 12] {
    if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    }
}

pub fn ticks_to_date(tick: u64) -> String {
    let time = ticks_to_calendar(tick);
    format!(
        "{:02}/{:02}/{} {:02}:{:02}:{:02}",
        time.month, time.day, time.year, time.hour, time.minute, time.second
    )
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneName {
    Title,
    Ending,
    Opening,
    NameSelect,
    Play,
    None,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct SceneRequest {
    pub change_to: Option<SceneName>,
    pub overlap: u8,
    pub transition: u8,
    pub movie_volume: f32,
    pub menu_open: bool,
    pub stop_status: bool,
    pub enable_next_stage: bool,
    pub restart_option: bool,
}

#[derive(Resource, Clone, Debug)]
pub struct TitleScene {
    pub movie_id: u32,
    pub movie_rest_frame: i32,
    pub movie_volume: f32,
    pub scene: SceneName,
    pub clear_count: u8,
    pub requests: SceneRequest,
}

impl Default for TitleScene {
    fn default() -> Self {
        Self {
            movie_id: 0,
            movie_rest_frame: 0,
            movie_volume: 1.0,
            scene: SceneName::Title,
            clear_count: 0,
            requests: SceneRequest::default(),
        }
    }
}

impl TitleScene {
    pub fn create(&mut self, scene: SceneName) {
        self.scene = scene;
        self.requests.menu_open = false;
        self.requests.stop_status = false;
        self.requests.enable_next_stage = true;
        self.movie_id = u32::from(scene != SceneName::Title);
    }

    pub fn execute(
        &mut self,
        pad: &PadInput,
        overlap_peek: bool,
        reset_to_opening: bool,
        movie_ready: bool,
    ) -> bool {
        if overlap_peek || reset_to_opening || !movie_ready {
            return true;
        }

        let leave = self.scene == SceneName::Ending
            && self.movie_rest_frame == 0
            || self.scene != SceneName::Ending
                && (pad.trig_a || pad.trig_b || pad.trig_start || self.movie_rest_frame == 0);
        if !leave {
            return true;
        }

        if self.scene == SceneName::Ending && self.clear_count == 0 {
            self.requests.change_to = Some(SceneName::NameSelect);
        } else {
            self.requests.change_to = Some(SceneName::Opening);
        }
        self.requests.overlap = 0;
        self.requests.transition = 5;
        self.requests.movie_volume = 0.0;
        self.movie_volume = 0.0;
        true
    }

    pub fn is_delete(&self) -> bool {
        true
    }

    pub fn delete(&mut self) -> SceneRequest {
        std::mem::take(&mut self.requests)
    }
}

#[derive(Clone, Debug)]
pub struct RoomInfo {
    pub name: String,
    pub stage_name: String,
    pub room_no: i8,
    pub start_code: u8,
    pub layer_no: i8,
}

#[derive(Clone, Debug)]
pub struct StageInfo {
    pub name: String,
    pub rooms: Vec<RoomInfo>,
}

#[derive(Clone, Debug, Default)]
pub struct MenuSceneData {
    pub num: u8,
    pub stage: Vec<StageInfo>,
}

#[derive(Resource, Clone, Debug)]
pub struct MenuScene {
    pub start_id: i32,
    pub cursor_id: i32,
    pub time_pattern: i32,
    pub week_pattern: i16,
    pub demo23: i16,
    pub group_point: Vec<i8>,
    pub language: u8,
    pub info: MenuSceneData,
    pub start_code: u8,
    pub repeat_up: u8,
    pub repeat_group: u8,
    pub display_flag: u8,
    pub pal_language: u8,
    pub next_stage: String,
    pub next_room: i8,
    pub next_layer: i8,
    pub next_start_code: u8,
    pub requests: SceneRequest,
    pub day: DayNight,
    pub time_advance: f32,
}

impl Default for MenuScene {
    fn default() -> Self {
        Self {
            start_id: 0,
            cursor_id: 0,
            time_pattern: 0,
            week_pattern: 0,
            demo23: 0,
            group_point: Vec::new(),
            language: 0,
            info: MenuSceneData::default(),
            start_code: 0,
            repeat_up: 0,
            repeat_group: 0,
            display_flag: 0,
            pal_language: 0,
            next_stage: String::new(),
            next_room: 0,
            next_layer: 0,
            next_start_code: 0,
            requests: SceneRequest::default(),
            day: DayNight::default(),
            time_advance: 0.0,
        }
    }
}

pub const TIME_PATTERN_NAMES: [&str; 7] = [
    "normal",
    "fast forward",
    "fixed morning",
    "fixed noon",
    "fixed evening",
    "fixed night",
    "fixed time",
];
pub const WEEK_PATTERN_NAMES: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
pub const LANGUAGE_NAMES: [&str; 5] = [
    "ENGLISH",
    "GERMAN",
    "FRENCH",
    "SPANISH",
    "ITALIAN",
];
pub const LANGUAGE_TABLE: [u8; 5] = [0, 1, 2, 3, 4];

impl MenuScene {
    pub fn phase_1(&mut self) -> bool {
        self.info = MenuSceneData::default();
        self.group_point = vec![0; usize::from(self.info.num)];
        true
    }

    pub fn phase_2(&mut self) -> bool {
        self.display_flag |= 2;
        true
    }

    pub fn execute(&mut self, pad: &PadInput) -> bool {
        if pad.hold_up || pad.hold_down {
            let mut trig = true;
            if pad.trig_up || pad.trig_down {
                self.repeat_up = 20;
            } else if self.repeat_up == 0 {
                self.repeat_up = 4;
            } else {
                trig = false;
            }

            if trig {
                if pad.hold_up {
                    self.cursor_id -= 1;
                    if self.cursor_id < 0 {
                        self.cursor_id = i32::from(self.info.num) - 1;
                    }
                } else {
                    self.cursor_id += 1;
                    if self.cursor_id >= i32::from(self.info.num) {
                        self.cursor_id = 0;
                    }
                }
            }
        }

        if pad.hold_b || pad.hold_a {
            let mut trig = true;
            if pad.trig_b || pad.trig_a {
                self.repeat_group = 20;
            } else if self.repeat_group == 0 {
                self.repeat_group = 4;
            } else {
                trig = false;
            }

            if trig {
                let slot = self
                    .cursor_id
                    .clamp(0, i32::from(self.info.num).saturating_sub(1))
                    as usize;
                let Some(info) = self.info.stage.get(slot) else {
                    return true;
                };
                let Some(point) = self.group_point.get_mut(slot) else {
                    return true;
                };
                if pad.hold_b {
                    *point -= 1;
                    if *point < 0 {
                        *point = info.rooms.len() as i8 - 1;
                    }
                } else {
                    *point += 1;
                    if *point >= info.rooms.len() as i8 {
                        *point = 0;
                    }
                }
            }
        }

        if pad.trig_r {
            self.start_code = self.start_code.wrapping_add(1);
            if self.start_code == 0 {
                self.start_code = 1;
            }
        } else if pad.trig_l {
            self.start_code = self.start_code.wrapping_sub(1);
            if self.start_code == 0 {
                self.start_code = u8::MAX;
            }
        }

        if pad.trig_start {
            self.requests.enable_next_stage = false;
            let index = self
                .cursor_id
                .clamp(0, i32::from(self.info.num).saturating_sub(1))
                as usize;
            let stage = self.info.stage.get(index);
            let point = self.group_point.get(index).copied().unwrap_or(0) as usize;
            let room = stage
                .and_then(|entry| entry.rooms.get(point))
                .cloned()
                .unwrap_or(RoomInfo {
                    name: String::new(),
                    stage_name: String::new(),
                    room_no: 0,
                    start_code: 0,
                    layer_no: 0,
                });
            let start_code = if self.start_code != 0 {
                self.start_code - 1
            } else {
                room.start_code
            };
            self.next_stage = room.stage_name.clone();
            self.next_room = room.room_no;
            self.next_layer = room.layer_no;
            self.next_start_code = start_code;

            if self.next_stage == "ENDING" {
                self.requests.change_to = Some(SceneName::Ending);
                self.requests.overlap = 0;
                self.requests.transition = 5;
            } else {
                self.requests.change_to = Some(SceneName::Play);
                self.requests.overlap = 0;
                self.requests.transition = 5;
            }
        }

        if pad.trig_y {
            self.time_pattern -= 1;
            if self.time_pattern < 0 {
                self.time_pattern = TIME_PATTERN_COUNT;
            }
        } else if pad.trig_x {
            self.time_pattern += 1;
            if self.time_pattern > TIME_PATTERN_COUNT {
                self.time_pattern = 0;
            }
        }

        if pad.trig_left {
            self.week_pattern -= 1;
            if self.week_pattern < 0 {
                self.week_pattern = 6;
            }
        } else if pad.trig_right {
            self.week_pattern += 1;
            if self.week_pattern >= 7 {
                self.week_pattern = 0;
            }
        }
        self.day.date = self.week_pattern as u8;

        if pad.trig_z3 {
            self.language += 1;
            if self.language > 4 {
                self.language = 0;
            }
            self.pal_language = LANGUAGE_TABLE[usize::from(self.language)];
        }

        if pad.trig_a2 {
            self.demo23 ^= 1;
        }

        self.time_advance = 0.0;
        if self.time_pattern >= 6 {
            self.day.time = (self.time_pattern - 6) as f32 * 15.0;
            self.time_advance = 1000.0;
        } else {
            match self.time_pattern {
                0 => {
                    self.day.time = 120.0;
                    self.time_advance = 0.02;
                }
                1 => {
                    self.day.time = 120.0;
                    self.time_advance = 0.25;
                }
                2 => {
                    self.day.time = 75.0;
                    self.time_advance = 1000.0;
                }
                3 => {
                    self.day.time = 195.0;
                    self.time_advance = 1001.0;
                }
                4 => {
                    self.day.time = 255.0;
                    self.time_advance = 1002.0;
                }
                5 => {
                    self.day.time = 315.0;
                    self.time_advance = 1003.0;
                }
                _ => {}
            }
        }

        true
    }

    pub fn draw(&mut self) -> Vec<String> {
        let mut out = Vec::new();
        out.push("menu".to_string());
        if self.start_code != 0 {
            out.push(format!("<{}>", i32::from(self.start_code) - 1));
        }
        let num = usize::from(self.info.num);
        let line_num = num.min(20);
        let scroll = self.cursor_id - self.start_id;
        if scroll < 5 {
            self.start_id += scroll - 5;
            if self.start_id < 0 {
                self.start_id = 0;
            }
        } else if scroll > 15 {
            let mut limit = i32::try_from(num).unwrap_or(0) - 20;
            if limit < 0 {
                limit = 0;
            }
            self.start_id += scroll - 15;
            if self.start_id > limit {
                self.start_id = limit;
            }
        }
        for id in self.start_id..self.start_id + line_num as i32 {
            let marker = if self.cursor_id == id { 'O' } else { ' ' };
            let index = id.max(0) as usize;
            let stage = self.info.stage.get(index);
            let name = stage.map_or("", |entry| entry.name.as_str());
            let room = stage
                .and_then(|entry| {
                    let point = self.group_point.get(index).copied().unwrap_or(0);
                    usize::try_from(point)
                        .ok()
                        .and_then(|at| entry.rooms.get(at))
                })
                .map_or("", |entry| entry.name.as_str());
            out.push(format!("{} {:2} {} <{}>", marker, id, name, room));
        }
        out.push("X: advance  Y: back".to_string());
        if self.time_pattern >= 6 {
            out.push(format!(
                "time: {}{}",
                self.time_pattern - 6,
                TIME_PATTERN_NAMES[6]
            ));
        } else {
            out.push(format!(
                "time: {}",
                TIME_PATTERN_NAMES[self.time_pattern.clamp(0, 6) as usize]
            ));
        }
        out.push("dpad right: advance  dpad left: back".to_string());
        out.push(format!(
            "weekday: {}",
            WEEK_PATTERN_NAMES[self.week_pattern.clamp(0, 6) as usize]
        ));
        out.push(LANGUAGE_NAMES[usize::from(self.pal_language).min(4)].to_string());
        out.push(format!(
            "pad3 (A): demo23 {}",
            if self.demo23 != 0 { "ON" } else { "OFF" }
        ));
        out
    }

    pub fn delete(&mut self) -> bool {
        self.display_flag &= !2;
        self.display_flag &= !2;
        self.requests.restart_option = false;
        true
    }
}

pub const GOVER_LETTER_COUNT: usize = 8;
pub const GOVER_LETTER_TAGS: [&str; GOVER_LETTER_COUNT] = [
    "txg", "txa", "txm", "txe1", "txo", "txv", "txe2", "txr",
];
pub const GOVER_ALPHA_MAX: f32 = 244.0;
pub const GOVER_IN_ALPHA: f32 = -288.0;
pub const GOVER_STEP_A: f32 = -9.0;
pub const GOVER_STEP_B: f32 = -9.0;
pub const GOVER_PARTICLE_X: [f32; GOVER_LETTER_COUNT] =
    [-175.0, -130.0, -80.0, -30.0, 32.0, 75.0, 120.0, 160.0];
pub const GOVER_PARTICLE_Y: f32 = 50.0;
pub const GOVER_PARTICLE_SMOKE: u16 = 0x002F;
pub const GOVER_PARTICLE_RING: u16 = 0x002E;
pub const GOVER_PARTICLE_HA: u16 = 0x0030;
pub const GOVER_ROT_TABLE: [i16; 6] = [40, 30, 20, -20, -30, -40];
pub const GOVER_HOLD: i16 = 90;
pub const GOVER_TAIL: i16 = 0x46;
pub const GOVER_LIFE_REFILL: u16 = 12;
pub const GOVER_SE_EXIT: u32 = 0x085A;

pub fn acc(param_0: i16, param_1: i16, param_2: i16) -> f32 {
    let span = f32::from(param_0 - param_2);
    if span == 0.0 {
        return 0.0;
    }
    let value = f32::from(param_1 - param_2);
    value * value / (span * span)
}

#[derive(Resource, Clone, Debug)]
pub struct GameOver {
    pub screen: Option<Screen>,
    pub letter: [Pane; GOVER_LETTER_COUNT],
    pub backdrop: Pane,
    pub count: i32,
    pub delay: i32,
    pub alpha: f32,
    pub rotate: [i16; GOVER_LETTER_COUNT],
    pub emitters: [Option<usize>; 2],
    pub hold: i16,
    pub tail: i16,
    pub state: u8,
    pub anime_start: bool,
    pub capture: bool,
    pub menu_open: bool,
    pub gameover_status: u8,
    pub reset: bool,
    pub add_death_count: bool,
    pub rng_seed: [i32; 3],
    pub sounds: Vec<u32>,
    pub life: u16,
}

impl Default for GameOver {
    fn default() -> Self {
        Self {
            screen: None,
            letter: std::array::from_fn(|i| Pane::new(GOVER_LETTER_TAGS[i])),
            backdrop: Pane::new("blak"),
            count: GOVER_LETTER_COUNT as i32,
            delay: 0,
            alpha: 0.0,
            rotate: [0; GOVER_LETTER_COUNT],
            emitters: [None, None],
            hold: GOVER_HOLD,
            tail: GOVER_TAIL,
            state: 0,
            anime_start: false,
            capture: false,
            menu_open: false,
            gameover_status: 0,
            reset: false,
            add_death_count: false,
            rng_seed: [100; 3],
            sounds: Vec::new(),
            life: GOVER_LIFE_REFILL,
        }
    }
}

impl GameOver {
    pub fn create(&mut self, screen: Screen) {
        self.screen = Some(screen);
        let bound = self.screen.as_ref();
        for index in 0..GOVER_LETTER_COUNT {
            if let Some(layout) = bound.and_then(|s| s.find(GOVER_LETTER_TAGS[index])) {
                self.letter[index].parts_set_file(layout);
            }
        }
        if let Some(layout) = bound.and_then(|s| s.find("blak")) {
            self.backdrop.parts_set_file(layout);
            self.backdrop.init_alpha = 0xFF;
        }
        self.add_death_count = true;
        self.hold = GOVER_HOLD;
        self.tail = GOVER_TAIL;
        self.anime_start = false;
    }

    pub fn delete(&mut self) {
        self.screen = None;
        for pane in &mut self.letter {
            pane.found = false;
        }
        self.backdrop.found = false;
    }

    pub fn anime_start(&mut self) {
        self.anime_start = true;
    }

    pub fn set_back_alpha(&mut self, alpha: f32) {
        self.alpha = alpha;
        self.backdrop.set_now_alpha(alpha);
    }

    pub fn value_init(&mut self) {
        self.count = GOVER_LETTER_COUNT as i32;
        self.delay = 0;
        for index in 0..GOVER_LETTER_COUNT {
            self.letter[index].user_area = 0;
        }
    }

    pub fn rnd(&mut self) -> f32 {
        for lane in 0..3 {
            self.rng_seed[lane] = (self.rng_seed[lane] * 0xAB) % 0x763D;
        }
        let sum = self.rng_seed[0] as f32 / 30269.0
            + self.rng_seed[1] as f32 / 30307.0
            + self.rng_seed[2] as f32 / 30323.0;
        (sum % 1.0).abs()
    }

    pub fn rnd_f(&mut self, range: f32) -> f32 {
        (self.rnd() * range) as i32 as f32
    }

    pub fn set_emitter0(&mut self, particles: &mut Particles, x: f32, y: f32, z: f32) {
        self.emitters[0] = Some(particles.set_2d_fore(GOVER_PARTICLE_SMOKE, x, y, z));
    }

    pub fn set_emitter1(&mut self, particles: &mut Particles, x: f32, y: f32, z: f32) {
        self.emitters[1] = Some(particles.set_2d_menu_fore(GOVER_PARTICLE_HA, x, y, z));
    }

    pub fn stop_emitter0(&mut self, particles: &mut Particles) {
        if let Some(index) = self.emitters[0] {
            particles.stop(index);
        }
    }

    pub fn stop_emitter1(&mut self, particles: &mut Particles) {
        if let Some(index) = self.emitters[1] {
            particles.stop(index);
        }
    }

    pub fn anime_open(&mut self, particles: &mut Particles) -> bool {
        let mut active = false;
        let mut started: i16 = -1;
        if self.count >= 0 {
            if self.delay != 0 {
                self.delay -= 1;
            } else {
                let mut rand = self.rnd_f(self.count as f32) as i32;
                if rand >= self.count - 1 {
                    rand = self.count - 1;
                }

                for index in 0..GOVER_LETTER_COUNT {
                    if self.letter[index].user_area == 0 {
                        if rand == 0 {
                            self.anime1(index);
                            started = index as i16;
                            break;
                        }
                        rand -= 1;
                    }
                }

                self.count -= 1;
                self.delay = (self.count * self.count) / 5;
            }
        }

        for index in 0..GOVER_LETTER_COUNT {
            if self.letter[index].user_area >= 0 && self.letter[index].user_area < 9 {
                if self.letter[index].user_area >= 1 && started != index as i16 {
                    self.anime1(index);
                    if self.letter[index].user_area == 5 {
                        particles.set_2d_fore(
                            GOVER_PARTICLE_RING,
                            GOVER_PARTICLE_X[index],
                            GOVER_PARTICLE_Y,
                            0.0,
                        );
                    }
                }

                active = true;
            }
        }

        !active
    }

    pub fn anime_close(&mut self) -> bool {
        let mut active = false;
        let mut started: i16 = -1;

        if self.count >= 0 {
            if self.delay != 0 {
                self.delay = 0;
            } else {
                let mut rand = self.rnd_f(self.count as f32) as i32;
                if rand >= self.count - 1 {
                    rand = self.count - 1;
                }

                for index in 0..GOVER_LETTER_COUNT {
                    if self.letter[index].user_area == 0 {
                        if rand == 0 {
                            self.anime2(index);
                            started = index as i16;
                            break;
                        }
                        rand -= 1;
                    }
                }

                self.delay = self.rnd_f(3.0) as i32 + 1;

                if self.delay >= 3 {
                    self.delay = 3;
                }

                self.count -= 1;
            }
        }

        for index in 0..GOVER_LETTER_COUNT {
            if self.letter[index].user_area >= 0 && self.letter[index].user_area < 7 {
                if self.letter[index].user_area >= 1 && started != index as i16 {
                    self.anime2(index);
                }

                active = true;
            }
        }

        !active
    }

    pub fn anime1(&mut self, index: usize) -> bool {
        let mut ret = false;

        if self.letter[index].user_area < 5 {
            if self.letter[index].user_area == 0 {
                self.letter[index].set_init_alpha();
            }

            self.letter[index].user_area += 1;
            let value = 1.0 - acc(5, self.letter[index].user_area, 0);
            self.letter[index]
                .pane_trans(0.0, value * GOVER_IN_ALPHA);
        } else if self.letter[index].user_area < 7 {
            self.letter[index].user_area += 1;
            let value = acc(2, self.letter[index].user_area - 5, 0);
            self.letter[index].pane_trans(0.0, value * GOVER_STEP_A);
        } else if self.letter[index].user_area < 9 {
            self.letter[index].user_area += 1;
            let value = 1.0 - acc(2, self.letter[index].user_area - 7, 0);
            self.letter[index].pane_trans(0.0, value * GOVER_STEP_B);
            if self.letter[index].user_area == 9 {
                self.sounds.push(GOVER_SE_EXIT);
            }
        }

        if self.letter[index].user_area == 9 {
            ret = true;
        }

        ret
    }

    pub fn anime2(&mut self, index: usize) -> bool {
        if self.letter[index].user_area < 7 {
            if self.letter[index].user_area == 0 {
                let rand = self.rnd_f(6.0) as i32;
                let rand = if rand > 5 { 5 } else { rand };
                self.rotate[index] = GOVER_ROT_TABLE[rand.max(0) as usize];
            }

            self.letter[index].user_area += 1;

            let step = self.letter[index].user_area;
            let y2 = f32::from(step) * f32::from(step) / 49.0;
            let angle = f32::from(self.rotate[index]) * (1.0 - f32::from(step) / 7.0);
            let (rx, ry) = (self.letter[index].size.x, self.letter[index].size.y);
            self.letter[index].rotate(rx / 2.0, ry / 2.0, angle);
            self.letter[index].pane_trans(0.0, y2 * GOVER_ALPHA_MAX);
        }

        if self.letter[index].user_area == 7 {
            self.letter[index].set_now_alpha_zero();
            true
        } else {
            false
        }
    }

    pub fn execute(
        &mut self,
        particles: &mut Particles,
        save_open: &mut dyn FnMut() -> bool,
        save_move: &mut dyn FnMut(),
        save_close: &mut dyn FnMut() -> bool,
        save_status: &mut dyn FnMut() -> u8,
        save_end_status: &mut dyn FnMut() -> u8,
    ) -> bool {
        if self.state == 3 {
            if save_open() {
                self.state = 4;
                self.life = GOVER_LIFE_REFILL;
            }
        } else if self.state == 4 {
            save_move();
            if save_status() == 3 {
                self.state = 5;
            }
        } else if self.state == 5 {
            if save_close() {
                match save_end_status() {
                    0 => {
                        self.gameover_status = 3;
                        self.menu_open = false;
                        self.reset = true;
                    }
                    1 => {
                        self.gameover_status = 2;
                        self.menu_open = false;
                    }
                    _ => {}
                }

                self.stop_emitter1(particles);
                self.state = 6;
            }
        } else if self.state != 6 && self.anime_start {
            if self.hold == GOVER_HOLD {
                if self.anime_open(particles) {
                    self.set_emitter0(particles, 0.0, 0.0, 0.0);
                    self.set_emitter1(particles, 0.0, 0.0, 0.0);
                    self.hold -= 1;
                }
            } else if self.hold > 0 {
                self.hold -= 1;
                if self.hold == 0 {
                    self.value_init();
                }
            } else if self.anime_close() {
                self.stop_emitter0(particles);
                self.tail -= 1;
                if self.tail <= 0 {
                    self.state = 3;
                    self.menu_open = true;
                }
            }
        }

        true
    }

    pub fn delete_check(&self) -> bool {
        self.state == 6
    }
}

pub fn time_of_day(time: f32) -> f32 {
    if time < 0.0 { TIME_WRAP + time } else { time % TIME_WRAP }
}
