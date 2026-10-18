use bevy_ecs::prelude::*;
use std::path::PathBuf;

use crate::j2d::{Kind, Screen, ScreenPane};
use crate::save::{DEFAULT_NAME, SAVE_VERSION, SLOTS, SaveGame};
use crate::sim::health::Health;
use crate::sim::vibration::Vibration;
use crate::text::msgget::{MsgArchive, MsgGet};

pub const SCREEN_LABEL: &str = "menu_save.blo";
pub const SCREEN_ARCHIVE: &str = "files/res/Msg/saveres.arc";
pub const ERROR_ARCHIVE: &str = "files/res/Msg/errorres.arc";
pub const DATA_BUFFER_SIZE: usize = 5712;
pub const WIPE_PANES: usize = 8;
pub const BASE_PANES: usize = 5;
pub const MENU_PANES: usize = 3;
pub const TEXT_BUFFER_A: usize = 256;
pub const TEXT_BUFFER_B: usize = 0x80;
pub const ERROR_PANES: usize = 13;
pub const ERROR_TEXT_BUFFER: usize = 512;
pub const ERROR_LINES_BIG: i32 = 12;
pub const ERROR_LINES_SMALL: i32 = 9;
pub const ERROR_LINE_BUMP: i32 = 7;
pub const ERROR_LINE_BUMP_BIG: i32 = 10;
pub const ERROR_FRAME: [i32; 6] = [5, 0xA0, 0x50, 0xE, -5, 0];
pub const ERROR_TALL: f32 = 1.4;
pub const ERROR_SHORT: f32 = 1.1;
pub const ERROR_CURSOR_X: [f32; 2] = [265.0, 345.0];
pub const ERROR_CURSOR_Y: f32 = 0.0;
pub const ERROR_SEPARATOR: &str = "  /  ";
pub const ERROR_CODE_SKIP: [usize; 5] = [3, 2, 3, 2, 2];
pub const ERROR_BORDER: f32 = 32.0;
pub const ERROR_HEAD_H: f32 = 12.0;

pub const SE_SAVE_PANEL_IN: u32 = 0x08B9;
pub const SE_SAVE_PANEL_OUT: u32 = 0x08BA;
pub const SE_SAVE_START: u32 = 0x08BB;
pub const SE_SAVE_CANCEL: u32 = 0x08BC;
pub const SE_SAVE_FINISH: u32 = 0x08BD;
pub const SE_ALERT_DATA: u32 = 0x08BE;
pub const SE_CONTINUE_GAME: u32 = 0x08C0;
pub const SE_QUIT_GAME: u32 = 0x08C1;
pub const SE_TALK_CURSOR: u32 = 0x083D;
pub const SE_ALERT_PANEL_IN: u32 = 0x0897;
pub const SE_ALERT_PANEL_OUT: u32 = 0x0898;

pub const CARD_STAT_WAIT: u8 = 0;
pub const CARD_STAT_RESTORE: u8 = 1;
pub const CARD_STAT_CREATE: u8 = 2;
pub const CARD_STAT_FORMAT: u8 = 3;
pub const CARD_STAT_READY: u8 = 4;
pub const CARD_STAT_DETACH: u8 = 5;
pub const CARD_STAT_ENCODING: u8 = 6;
pub const CARD_STAT_ERROR: u8 = 7;
pub const CARD_STAT_WRONG_DEVICE: u8 = 10;
pub const CARD_STAT_IOERROR: u8 = 12;

pub const MSG_RECOVER_YES: u32 = 99;
pub const MSG_RESTART: u32 = 100;
pub const MSG_NO_SPACE: u32 = 0x30;
pub const MSG_NO_FILE: u32 = 0x31;
pub const MSG_FORMAT_DONE: u32 = 0x32;
pub const MSG_FORMAT_FAIL: u32 = 0x33;
pub const MSG_GAME_FILE_ASK: u32 = 0x34;
pub const MSG_GAME_FILE_NO: u32 = 0x35;
pub const MSG_GAME_FILE_WRITING: u32 = 0x36;
pub const MSG_GAME_FILE_FAIL: u32 = 0x37;
pub const MSG_GAME_FILE_NEW: u32 = 0x38;
pub const MSG_BAD_CHECKSUM: u32 = 0x3A;
pub const MSG_BAD_CHECKSUM2: u32 = 0x3B;
pub const MSG_SAVING: u32 = 0x3C;
pub const MSG_SAVE_FAILED: u32 = 0x3D;
pub const MSG_SAVED: u32 = 0x3E;
pub const MSG_TITLE: u32 = 0x39;
pub const MSG_CONTINUE: u32 = 0x40;
pub const MSG_IO_ERROR: u32 = 0x62;
pub const MSG_CARD_UNAVAILABLE: u32 = 0x65;
pub const MSG_ENDING_TITLE: u32 = 0x4C;
pub const MSG_ENDING_CONTINUE: u32 = 0x4D;

pub const PUSH_MENU_BUTTON_NONE: u8 = 0;
pub const PUSH_MENU_BUTTON_CLOSE: u8 = 2;
pub const MENU_STATUS_NONE: u8 = 0;
pub const MENU_STATUS_ITEM: u8 = 1;
pub const MENU_STATUS_COLLECT: u8 = 2;
pub const MENU_STATUS_MAP: u8 = 3;
pub const MENU_STATUS_NAME: u8 = 4;
pub const MENU_STATUS_SAVE: u8 = 5;

pub use crate::j2d::{STICK_HALF_DEAD, STICK_MAX_VALUE, STICK_MIN_VALUE};

pub const SAVE_HIO_SCALE: f32 = 1.25;
pub const SAVE_HIO_DY: f32 = 12.0;

pub const GAME_OVER_HOLD: i16 = 90;
pub const GAME_OVER_TAIL: i16 = 0x46;

pub const CONTINUE_DELAY: i16 = 15;
pub const QUIT_DELAY: i16 = 30;
pub const ENDING_TICKER_DELAY: i16 = 30;

pub const MESSAGE_TABLE: [u32; 5] = [0x40, 0x3A, 0x3C, 0x3E, 0x4D];

pub const WIPE_TAGS: [&str; WIPE_PANES] = [
    "wp00", "wp01", "wp02", "wp03", "wp04", "wp05", "wp06", "wp07",
];
pub const MASK_TAG: &str = "msk0";
pub const BASE_TAGS: [&str; BASE_PANES] = ["cc01", "cc12", "cc23", "st00", "st01"];
pub const TITLE_TAG: &str = "tk00";
pub const TITLE_LABEL_TAG: &str = "tl00";
pub const CURSOR_TAG: &str = "cur1";
pub const YES_KEY_TAGS: [&str; MENU_PANES] = ["ysk0", "ys00", "yes"];
pub const NO_KEY_TAGS: [&str; MENU_PANES] = ["nok0", "no00", "no"];
pub const OPEN_FOR_GAME_OVER_X: [f32; 4] = [-180.0, -70.0, 70.0, 180.0];
pub const OPEN_FOR_GAME_OVER_Y: f32 = 170.0;
pub const PARTICLE_OPEN_MENU: u16 = 0x2E;

pub const IDX_05: u8 = 5;
pub const IDX_06: u8 = 0;
pub const IDX_07: u8 = 0;
pub const IDX_08: u8 = 5;
pub const IDX_09: u8 = 0;
pub const IDX_0a: u8 = 0;
pub const IDX_0b: u8 = 3;
pub const IDX_0c: i16 = -500;
pub const IDX_0e: i16 = 200;
pub const IDX_10: u8 = 5;
pub const IDX_11: u8 = 8;
pub const IDX_12: u8 = 5;
pub const IDX_13: u8 = 0;
pub const IDX_14: i16 = -100;
pub const IDX_16: i16 = 150;
pub const IDX_18: u8 = 5;
pub const IDX_19: u8 = 8;
pub const IDX_1a: u8 = 5;
pub const IDX_1b: u8 = 6;
pub const IDX_1c: u8 = 2;
pub const IDX_1e: i16 = 200;
pub const IDX_20: u8 = 12;
pub const IDX_21: u8 = 140;
pub const IDX_22: u8 = 70;
pub const IDX_23: u8 = 6;
pub const IDX_24: u8 = 8;
pub const IDX_25: u8 = 0;
pub const IDX_26: u8 = 3;
pub const IDX_28: u8 = 90;
pub const IDX_29: u8 = 10;
pub const IDX_2a: u8 = 0;
pub const IDX_2c: i16 = -60;
pub const IDX_2e: u8 = 0;

pub const proc_save_question: u8 = 0;
pub const proc_mem_card_check: u8 = 1;
pub const proc_open_save_menu: u8 = 2;
pub const proc_close_save_menu: u8 = 3;
pub const proc_mem_card_err_msg_wait_key: u8 = 4;
pub const proc_mem_card_err_msg_wait_key2: u8 = 5;
pub const proc_mem_card_err_msg_wait_key3: u8 = 6;
pub const proc_mem_card_err_go_ipl_sel: u8 = 7;
pub const proc_mem_card_err_go_ipl_sel2: u8 = 8;
pub const proc_mem_card_format: u8 = 9;
pub const proc_mem_card_err_msg_wait_format_sel: u8 = 10;
pub const proc_mem_card_err_msg_wait_format_sel2: u8 = 11;
pub const proc_mem_card_format_check: u8 = 12;
pub const proc_mem_card_make_game_file_sel: u8 = 13;
pub const proc_mem_card_make_game_file: u8 = 14;
pub const proc_mem_card_make_game_file_check: u8 = 15;
pub const proc_mem_card_data_load_wait: u8 = 16;
pub const proc_mem_card_data_load_wait2: u8 = 17;
pub const proc_mem_card_data_save: u8 = 18;
pub const proc_mem_card_data_save_up_menu: u8 = 19;
pub const proc_mem_card_data_save_sel: u8 = 20;
pub const proc_mem_card_data_save_down_menu: u8 = 21;
pub const proc_mem_card_data_save_wait: u8 = 22;
pub const proc_msg_wait: u8 = 23;
pub const proc_msg_wait2: u8 = 24;
pub const proc_save_end_wait: u8 = 25;
pub const proc_save_wait: u8 = 26;
pub const proc_can_not_save: u8 = 27;
pub const proc_can_not_save2: u8 = 28;
pub const proc_game_continue: u8 = 29;
pub const proc_game_continue2: u8 = 30;
pub const proc_game_continue3: u8 = 31;
pub const proc_game_continue4: u8 = 32;
pub const proc_ending_no_save: u8 = 33;
pub const proc_ending_no_save2: u8 = 34;
pub const proc_ending_no_save3: u8 = 35;
pub const proc_ending_data_check: u8 = 36;
pub const proc_no_save: u8 = 37;
pub const proc_no_save2: u8 = 38;

pub const PROC_COUNT: usize = 39;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Proc {
    SaveQuestion,
    MemCardCheck,
    OpenSaveMenu,
    CloseSaveMenu,
    MemCardErrMsgWaitKey,
    MemCardErrMsgWaitKey2,
    MemCardErrMsgWaitKey3,
    MemCardErrGoIplSel,
    MemCardErrGoIplSel2,
    MemCardErrMsgWaitFormatSel,
    MemCardErrMsgWaitFormatSel2,
    MemCardFormat,
    MemCardFormatCheck,
    MemCardMakeGameFileSel,
    MemCardMakeGameFile,
    MemCardMakeGameFileCheck,
    MemCardDataLoadWait,
    MemCardDataLoadWait2,
    MemCardDataSave,
    MemCardDataSaveUpMenu,
    MemCardDataSaveSel,
    MemCardDataSaveDownMenu,
    MemCardDataSaveWait,
    MsgWait,
    MsgWait2,
    SaveEndWait,
    SaveWait,
    CanNotSave,
    CanNotSave2,
    GameContinue,
    GameContinue2,
    GameContinue3,
    GameContinue4,
    EndingNoSave,
    EndingNoSave2,
    EndingNoSave3,
    EndingDataCheck,
    NoSave,
    NoSave2,
}

pub fn dispatch(proc: u8) -> Proc {
    match proc {
        0 => Proc::SaveQuestion,
        1 => Proc::MemCardCheck,
        2 => Proc::OpenSaveMenu,
        3 => Proc::CloseSaveMenu,
        4 => Proc::MemCardErrMsgWaitKey,
        5 => Proc::MemCardErrMsgWaitKey2,
        6 => Proc::MemCardErrMsgWaitKey3,
        7 => Proc::MemCardErrGoIplSel,
        8 => Proc::MemCardErrGoIplSel2,
        9 => Proc::MemCardErrMsgWaitFormatSel,
        10 => Proc::MemCardErrMsgWaitFormatSel2,
        11 => Proc::MemCardFormat,
        12 => Proc::MemCardFormatCheck,
        proc_mem_card_make_game_file_sel => Proc::MemCardMakeGameFileSel,
        proc_mem_card_make_game_file => Proc::MemCardMakeGameFile,
        proc_mem_card_make_game_file_check => Proc::MemCardMakeGameFileCheck,
        proc_mem_card_data_load_wait => Proc::MemCardDataLoadWait,
        proc_mem_card_data_load_wait2 => Proc::MemCardDataLoadWait2,
        proc_mem_card_data_save => Proc::MemCardDataSave,
        proc_mem_card_data_save_up_menu => Proc::MemCardDataSaveUpMenu,
        proc_mem_card_data_save_sel => Proc::MemCardDataSaveSel,
        proc_mem_card_data_save_down_menu => Proc::MemCardDataSaveDownMenu,
        proc_mem_card_data_save_wait => Proc::MemCardDataSaveWait,
        proc_msg_wait => Proc::MsgWait,
        proc_msg_wait2 => Proc::MsgWait2,
        proc_save_end_wait => Proc::SaveEndWait,
        proc_save_wait => Proc::SaveWait,
        proc_can_not_save => Proc::CanNotSave,
        proc_can_not_save2 => Proc::CanNotSave2,
        proc_game_continue => Proc::GameContinue,
        proc_game_continue2 => Proc::GameContinue2,
        proc_game_continue3 => Proc::GameContinue3,
        proc_game_continue4 => Proc::GameContinue4,
        proc_ending_no_save => Proc::EndingNoSave,
        proc_ending_no_save2 => Proc::EndingNoSave2,
        proc_ending_no_save3 => Proc::EndingNoSave3,
        proc_ending_data_check => Proc::EndingDataCheck,
        proc_no_save => Proc::NoSave,
        proc_no_save2 => Proc::NoSave2,
        _ => Proc::SaveWait,
    }
}

pub fn proc_name(proc: u8) -> &'static str {
    match dispatch(proc) {
        Proc::SaveQuestion => "save_question",
        Proc::MemCardCheck => "mem_card_check",
        Proc::OpenSaveMenu => "open_save_menu",
        Proc::CloseSaveMenu => "close_save_menu",
        Proc::MemCardErrMsgWaitKey => "mem_card_err_msg_wait_key",
        Proc::MemCardErrMsgWaitKey2 => "mem_card_err_msg_wait_key2",
        Proc::MemCardErrMsgWaitKey3 => "mem_card_err_msg_wait_key3",
        Proc::MemCardErrGoIplSel => "mem_card_err_go_ipl_sel",
        Proc::MemCardErrGoIplSel2 => "mem_card_err_go_ipl_sel2",
        Proc::MemCardFormat => "mem_card_format",
        Proc::MemCardErrMsgWaitFormatSel => "mem_card_err_msg_wait_format_sel",
        Proc::MemCardErrMsgWaitFormatSel2 => "mem_card_err_msg_wait_format_sel2",
        Proc::MemCardFormatCheck => "mem_card_format_check",
        Proc::MemCardMakeGameFileSel => "mem_card_make_game_file_sel",
        Proc::MemCardMakeGameFile => "mem_card_make_game_file",
        Proc::MemCardMakeGameFileCheck => "mem_card_make_game_file_check",
        Proc::MemCardDataLoadWait => "mem_card_data_load_wait",
        Proc::MemCardDataLoadWait2 => "mem_card_data_load_wait2",
        Proc::MemCardDataSave => "mem_card_data_save",
        Proc::MemCardDataSaveUpMenu => "mem_card_data_save_up_menu",
        Proc::MemCardDataSaveSel => "mem_card_data_save_sel",
        Proc::MemCardDataSaveDownMenu => "mem_card_data_save_down_menu",
        Proc::MemCardDataSaveWait => "mem_card_data_save_wait",
        Proc::MsgWait => "msg_wait",
        Proc::MsgWait2 => "msg_wait2",
        Proc::SaveEndWait => "save_end_wait",
        Proc::SaveWait => "save_wait",
        Proc::CanNotSave => "can_not_save",
        Proc::CanNotSave2 => "can_not_save2",
        Proc::GameContinue => "game_continue",
        Proc::GameContinue2 => "game_continue2",
        Proc::GameContinue3 => "game_continue3",
        Proc::GameContinue4 => "game_continue4",
        Proc::EndingNoSave => "ending_no_save",
        Proc::EndingNoSave2 => "ending_no_save2",
        Proc::EndingNoSave3 => "ending_no_save3",
        Proc::EndingDataCheck => "ending_data_check",
        Proc::NoSave => "no_save",
        Proc::NoSave2 => "no_save2",
    }
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
        4 => (v * (0x8000u32 as f32) * 0.5).sin() * (v * (0x8000u32 as f32) * 0.5).sin(),
        5 => (v * (0xFFFFu32 as f32) * 0.5).sin() * (v * (0xFFFFu32 as f32) * 0.5).sin(),
        _ => v,
    }
}

fn deg2s(deg: f32) -> i16 {
    (deg * 32768.0 / 180.0) as i16
}

pub use crate::j2d::F2;

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
    pub black: Color,
    pub white: Color,
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
            black: Color::default(),
            white: Color::new(0x8C, 0xB4, 0x50, 0xFF),
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
        self.black = Color::new(0x28, 0x6E, 0x1E, 0);
        self.white = Color::new(0x8C, 0xB4, 0x50, 0xFF);
        self.text = layout.text.clone();
        self.font_size_x = layout.font_size_x as f32;
        self.font_size_y = layout.font_size_y as f32;
        self.char_space = layout.char_space as f32;
        self.line_space = layout.line_space as f32;
        self.shift_x = 0.0;
        self.shift_y = 0.0;
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

    pub fn resize(&mut self, w: f32, h: f32) {
        self.size.x = w;
        self.size.y = h;
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.pos_top_left.x = x;
        self.pos_top_left.y = y;
        self.pos_center.x = x + self.size.x / 2.0;
        self.pos_center.y = y + self.size.y / 2.0;
    }

    pub fn rotate(&mut self, x: f32, y: f32, angle: f32) {
        self.rotate_x = x;
        self.rotate_y = y;
        self.rotation = (deg2s(angle) as u16) & 0xFFFF;
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

    pub fn parts_set_file(&mut self, layout: &ScreenPane) {
        self.parts_set(layout);
        self.black = Color::new(0x28, 0x6E, 0x1E, 0);
        self.white = Color::new(0x8C, 0xB4, 0x50, 0xFF);
    }

    pub fn set_black_white(&mut self, black: Color, white: Color) {
        self.black = black;
        self.white = white;
    }
}

pub fn pane_trans_group(panes: &mut [Pane], x: f32, y: f32) {
    for pane in panes.iter_mut() {
        pane.pane_trans(x, y);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl From<[u8; 4]> for Color {
    fn from(value: [u8; 4]) -> Self {
        Self::new(value[0], value[1], value[2], value[3])
    }
}

pub use crate::j2d::StickControl;

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
    pub stick_value: f32,
    pub stick_angle: i16,
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

pub trait MsgSource {
    fn message(&self, msg_no: u32) -> String;
}

pub struct ArchiveMsgSource<'a> {
    pub archive: &'a dyn MsgArchive,
    pub clear_count: u8,
}

impl MsgSource for ArchiveMsgSource<'_> {
    fn message(&self, msg_no: u32) -> String {
        message_get(self.archive, self.clear_count, msg_no)
    }
}

pub fn message_get(archive: &dyn MsgArchive, clear_count: u8, msg_no: u32) -> String {
    let mut get = MsgGet::default();
    let mut out = String::new();
    let Some(head) = get.get_mesg_header(archive, clear_count, msg_no) else {
        return out;
    };
    let Some(bytes) = get.get_message(head) else {
        return out;
    };
    let mut at = 0usize;
    while bytes.get(at).copied().unwrap_or(0) != 0 {
        let byte = bytes[at];
        if byte == 0x1A {
            let len = bytes.get(at + 1).copied().unwrap_or(0);
            at += usize::from(len).max(2);
        } else {
            out.push(byte as char);
            at += 1;
        }
    }
    out
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Emitter {
    pub id: u16,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub alive: bool,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct Particles {
    pub layer: Vec<Emitter>,
}

impl Particles {
    pub fn reset(&mut self) {
        self.layer.clear();
    }

    pub fn set_2d_fore(&mut self, id: u16, x: f32, y: f32, z: f32) -> usize {
        self.push(Emitter {
            id,
            x,
            y,
            z,
            alive: true,
        })
    }

    pub fn set_2d_menu_fore(&mut self, id: u16, x: f32, y: f32, z: f32) -> usize {
        self.push(Emitter {
            id,
            x,
            y,
            z,
            alive: true,
        })
    }

    fn push(&mut self, emitter: Emitter) -> usize {
        self.layer.push(emitter);
        self.layer.len() - 1
    }

    pub fn stop(&mut self, index: usize) {
        if let Some(emitter) = self.layer.get_mut(index) {
            emitter.alive = false;
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct MemCard {
    pub dir: PathBuf,
    pub slots: [Option<SaveGame>; SLOTS],
    pub clear_count: [u8; SLOTS],
    pub owl_link: [u8; SLOTS],
    pub status: u8,
    pub probe_stat: u8,
    pub serial: u64,
    pub copy_to_pos: u8,
    pub picture_data: Option<usize>,
    pub picture_write: Option<usize>,
    pub data_version: u32,
    pub save_table: u32,
    pub staged_save: bool,
    pub game_start_stage_set: bool,
    pub data_num: u8,
    pub new_file: u8,
    pub no_file: u8,
    pub mem_card_check_id: u64,
    pub checksum: [u64; SLOTS],
    pub load_pending: bool,
    pub load_frames: u8,
    pub load_result: i32,
    pub save_pending: bool,
    pub save_frames: u8,
    pub save_result: i32,
    pub format_pending: bool,
    pub format_frames: u8,
    pub format_result: i32,
    pub written: Vec<usize>,
    /// What each slot's file held when the card was read, which is what separates a slot the card
    /// has never been written to from one whose file will not parse. The card's `status` names one
    /// state for the whole card, `CARD_STAT_CREATE`, and retail reaches it from
    /// `mDoMemCd_Ctrl_c::LoadSync`'s "not state three" arm, `m_Do_MemCard.cpp:198-200`, which is a
    /// per-load failure rather than a per-card truth. Retail's own reader
    /// `mDoMemCdRWm_Restore` is what tells the two apart, and it is in
    /// `m_Do_MemCardRWmng.cpp`.
    pub slot_state: [SlotState; SLOTS],
}

/// One slot's file as [`crate::save::read_slot_in`] found it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SlotState {
    /// No file, or one that is not a save at all: the card offers to create here.
    #[default]
    Absent,
    /// A file that would not parse. This is the slot the erase prompt is for, and it is what the
    /// file select's `data_broken` reports, so it has to survive the read rather than collapse
    /// into [`SlotState::Absent`].
    Corrupt,
    /// A save, kept so a copy can rewrite it without a second parse.
    Loaded,
}

impl Default for MemCard {
    fn default() -> Self {
        Self {
            dir: crate::save::slots_dir(),
            slots: [None, None, None],
            clear_count: [0; SLOTS],
            owl_link: [0; SLOTS],
            status: CARD_STAT_WAIT,
            probe_stat: 2,
            serial: 0,
            copy_to_pos: 0xFF,
            picture_data: None,
            picture_write: None,
            data_version: SAVE_VERSION,
            save_table: 0,
            staged_save: false,
            game_start_stage_set: false,
            data_num: 0,
            new_file: 1,
            no_file: 0,
            mem_card_check_id: 0,
            checksum: [0; SLOTS],
            load_pending: false,
            load_frames: 0,
            load_result: 1,
            save_pending: false,
            save_frames: 0,
            save_result: 1,
            format_pending: false,
            format_frames: 0,
            format_result: 1,
            written: Vec::new(),
            slot_state: [SlotState::Absent; SLOTS],
        }
    }
}

impl MemCard {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            ..Default::default()
        }
    }

    pub fn get_status(&self) -> u8 {
        self.status
    }

    pub fn get_probe_stat(&self) -> u8 {
        self.probe_stat
    }

    pub fn clear_probe_stat(&mut self) {
        self.probe_stat = 2;
    }

    pub fn get_card_serial_no(&self) -> u64 {
        self.serial
    }

    pub fn set_card_serial_no(&mut self, serial: u64) {
        self.serial = serial;
    }

    pub fn get_copy_to_pos(&self) -> u8 {
        self.copy_to_pos
    }

    pub fn set_copy_to_pos(&mut self, pos: u8) {
        self.copy_to_pos = pos;
    }

    pub fn set_pict_data_ptr(&mut self, ptr: Option<usize>) {
        self.picture_data = ptr;
    }

    pub fn set_pict_write_data_ptr(&mut self, ptr: Option<usize>) {
        self.picture_write = ptr;
    }

    pub fn set_data_version(&mut self, version: u32) {
        self.data_version = version;
    }

    pub fn get_data_num(&self) -> u8 {
        self.data_num
    }

    pub fn set_data_num(&mut self, num: u8) {
        self.data_num = num;
    }

    pub fn exchange_player_recollection_data(&mut self) {}

    pub fn get_save_table(&self) -> u32 {
        self.save_table
    }

    pub fn set_save_table(&mut self, table: u32) {
        self.save_table = table;
    }

    pub fn put_save(&mut self, save_table: u32) {
        self.save_table = save_table;
        self.staged_save = true;
    }

    pub fn set_game_start_stage(&mut self) {
        self.game_start_stage_set = true;
    }

    pub fn get_new_file(&self) -> u8 {
        self.new_file
    }

    pub fn set_new_file(&mut self, file: u8) {
        self.new_file = file;
    }

    pub fn get_no_file(&self) -> u8 {
        self.no_file
    }

    pub fn set_no_file(&mut self, num: u8) {
        self.no_file = num;
    }

    pub fn get_mem_card_check_id(&self) -> u64 {
        self.mem_card_check_id
    }

    pub fn set_mem_card_check_id(&mut self, id: u64) {
        self.mem_card_check_id = id;
    }

    pub fn load(&mut self) {
        self.load_pending = true;
        self.load_frames = 2;
    }

    pub fn load_sync(&mut self) -> i32 {
        if !self.load_pending {
            return 1;
        }
        if self.load_frames > 0 {
            self.load_frames -= 1;
            return 0;
        }
        self.load_pending = false;
        if self.load_result == 2 {
            return 2;
        }
        self.restore();
        1
    }

    /// `mDoMemCd_Ctrl_c::restore`, `m_Do_MemCard.cpp`, which reads the card's file and leaves
    /// `field_0x1660` at three, the loaded state `LoadSync` then answers from.
    ///
    /// The port reads the slots rather than the card's one `gczelda` file, so this is also where
    /// the card's `status` comes from: `status_after_read` is the `CARDOpen` of `:143-144`. Retail
    /// reaches three only through `mDoMemCdRWm_Restore`, which is in `m_Do_MemCardRWmng.cpp` and
    /// which the port's per-slot read stands in for.
    pub fn restore(&mut self) {
        self.slots = std::array::from_fn(|_| None);
        self.checksum = [0; SLOTS];
        self.slot_state = [SlotState::Absent; SLOTS];
        for slot in 0..SLOTS {
            self.read_slot(slot);
        }
        self.status = self.status_after_read();
    }

    /// The read behind [`MemCard::load_sync`], which is also what a card that was never loaded
    /// through the two-frame handshake needs: `mDoMemCd_Ctrl_c::LoadSync` hands back two when the
    /// card is not in its loaded state, `m_Do_MemCard.cpp:203`, and the port's two arms are the
    /// same wait and the same answer.
    ///
    /// `read_slot_in` answers one `Option`, which cannot say whether the file was absent or
    /// unreadable, and the two mean opposite things to the file select: absent offers a new file,
    /// unreadable offers an erase. So the file is read here rather than through
    /// `read_slot_in`, and the two failures are kept apart.
    pub fn read_slot(&mut self, slot: usize) {
        let Some(slot) = usize::try_from(slot).ok().filter(|&slot| slot < SLOTS) else {
            return;
        };
        let path = crate::save::slot_path_in(&self.dir, slot);
        let state = match std::fs::read_to_string(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => SlotState::Absent,
            Err(_) => SlotState::Corrupt,
            Ok(text) => match serde_json::from_str::<SaveGame>(&text) {
                Ok(save) => {
                    self.slots[slot] = Some(save);
                    SlotState::Loaded
                }
                Err(_) => SlotState::Corrupt,
            },
        };
        if state == SlotState::Absent {
            self.slots[slot] = None;
        }
        self.slot_state[slot] = state;
        if state == SlotState::Loaded {
            self.set_checksum_game_data(slot);
        }
    }

    /// The status the card reports once every slot has been read.
    ///
    /// `CARD_STAT_CREATE` is a per-card truth, not a per-slot one, and retail reaches it from one
    /// `CARDOpen` of `gczelda`, `m_Do_MemCard.cpp:143-144`. It means the card holds no game file,
    /// and the name scene's answer to it is not an error: it sets the picture pointer and loads,
    /// `d_s_name.cpp:748-751`, so the screen proceeds to offer a new game. So a card with no
    /// readable save in it is in the create state and one holding any save is not; a card whose
    /// slots are all unreadable is still a card with no readable save in it.
    pub fn status_after_read(&self) -> u8 {
        if self
            .slot_state
            .iter()
            .all(|state| *state != SlotState::Loaded)
        {
            CARD_STAT_CREATE
        } else {
            CARD_STAT_WAIT
        }
    }

    pub fn save(&mut self) {
        self.save_pending = true;
        self.save_frames = 2;
    }

    pub fn save_sync(&mut self) -> i32 {
        if !self.save_pending {
            return 1;
        }
        if self.save_frames > 0 {
            self.save_frames -= 1;
            return 0;
        }
        self.save_pending = false;
        if self.save_result == 2 {
            return 2;
        }
        self.flush();
        1
    }

    pub fn format(&mut self) {
        self.format_pending = true;
        self.format_frames = 2;
    }

    pub fn format_sync(&mut self) -> i32 {
        if !self.format_pending {
            return 1;
        }
        if self.format_frames > 0 {
            self.format_frames -= 1;
            return 0;
        }
        self.format_pending = false;
        if self.format_result == 2 {
            return 2;
        }
        for slot in self.slots.iter_mut() {
            *slot = None;
        }
        self.checksum = [0; SLOTS];
        self.clear_count = [0; SLOTS];
        self.owl_link = [0; SLOTS];
        1
    }

    pub fn flush(&mut self) {
        for slot in 0..SLOTS {
            let Some(save) = self.slots[slot].clone() else {
                continue;
            };
            if crate::save::write_slot_in(&self.dir, slot, &save).is_ok() {
                self.written.push(slot);
            }
        }
    }

    pub fn set_init_data_to_card(&mut self, slot: usize) {
        if slot < SLOTS {
            self.slots[slot] = None;
        }
    }

    pub fn set_memory_to_card(&mut self, slot: usize, save: SaveGame) {
        if slot < SLOTS {
            self.slots[slot] = Some(save);
        }
    }

    pub fn set_checksum_game_data(&mut self, slot: usize) {
        if slot >= SLOTS {
            return;
        }
        let hash = self.slots[slot]
            .as_ref()
            .and_then(|save| serde_json::to_vec(save).ok())
            .map(|bytes| checksum_of(&bytes))
            .unwrap_or(0);
        self.checksum[slot] = hash;
    }

    pub fn test_checksum_game_data(&self, slot: usize) -> bool {
        if slot >= SLOTS {
            return false;
        }
        let Some(save) = self.slots[slot].as_ref() else {
            return false;
        };
        if save.version != SAVE_VERSION {
            return false;
        }
        if self.checksum[slot] == 0 {
            return true;
        }
        serde_json::to_vec(save)
            .map(|bytes| checksum_of(&bytes) == self.checksum[slot])
            .unwrap_or(false)
    }

    pub fn erase_slot(&mut self, slot: usize) {
        if slot < SLOTS {
            self.slots[slot] = None;
            self.checksum[slot] = 0;
            self.clear_count[slot] = 0;
            self.owl_link[slot] = 0;
        }
    }

    pub fn set_clear_count(&mut self, slot: usize, count: u8) {
        if slot < SLOTS {
            self.clear_count[slot] = count;
        }
    }

    pub fn set_owl_link(&mut self, slot: usize, link: u8) {
        if slot < SLOTS {
            self.owl_link[slot] = link;
        }
    }
}

pub fn checksum_of(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xCBF2_9CE4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100_0000_01B3);
    }
    hash
}

pub fn blank_save() -> SaveGame {
    crate::save::SaveGame {
        version: SAVE_VERSION,
        name: String::new(),
        tick: 0,
        camera_yaw: 0.0,
        player: crate::save::SavedPlayer {
            pos: [0.0; 3],
            yaw: 0.0,
            speed: 0.0,
            speed_y: 0.0,
        },
        area: crate::save::SavedArea {
            bounds_min: [0.0; 2],
            bounds_max: [0.0; 2],
            spawn: [0.0; 3],
            yaw: 0.0,
        },
        health: crate::save::SavedHealth::default(),
        stage: String::new(),
        rooms: Vec::new(),
        rupees: 0,
        events: Vec::new(),
        switches: [0; 4],
        arrivals: Vec::new(),
        story: Vec::new(),
        held_item: 0,
        held_count: 0,
        time: 0.0,
        date: 0,
        get_bag: 0,
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct MenuFlags {
    pub menu_open: bool,
    pub menu_status: u8,
    pub push_menu_button: Option<u8>,
    pub heap_lock_flag: u8,
    pub message_status: u8,
}

impl MenuFlags {
    pub fn flag(&self) -> bool {
        self.menu_open
    }

    pub fn flag_set(&mut self, value: bool) {
        self.menu_open = value;
    }

    pub fn set_push_menu_button(&mut self, button: u8) {
        self.push_menu_button = Some(button);
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct SystemRequest {
    pub reset: bool,
    pub os_reset: bool,
}

impl SystemRequest {
    pub fn on_reset(&mut self) {
        self.reset = true;
    }

    pub fn is_reset(&self) -> bool {
        self.reset
    }

    pub fn clear(&mut self) {
        self.reset = false;
        self.os_reset = false;
    }
}

#[derive(Clone, Debug)]
pub struct ErrorPanel {
    pub panes: [Pane; ERROR_PANES],
    pub message: String,
    pub state: u8,
    pub status: u8,
    pub yes_no: u8,
    pub flag_2f7: u8,
    pub flag_2f8: u8,
    pub flag_2f9: u8,
    pub flag_2fa: u8,
    pub flag_2fb: u8,
    pub flag_2fc: u8,
    pub time_count_down: u8,
    pub timer: i16,
    pub board_y: f32,
    pub pane0_rotation: f32,
    pub pane1_rotation: f32,
    pub stick: StickControl,
    pub pal_language: u8,
    pub tick_rate: u8,
    pub sounds: Vec<u32>,
}

impl Default for ErrorPanel {
    fn default() -> Self {
        let tags = [
            "cur1", "cur2", "mes", "cc00", "cc01", "b000", "cc02", "b001", "cc03", "b002", "cc04",
            "b003", "b004",
        ];
        Self {
            panes: std::array::from_fn(|i| Pane::new(tags[i])),
            message: String::new(),
            state: 0,
            status: 0,
            sounds: Vec::new(),
            yes_no: 1,
            flag_2f7: 0,
            flag_2f8: 0,
            flag_2f9: 0,
            flag_2fa: 0,
            flag_2fb: 0x5A,
            flag_2fc: 0,
            time_count_down: 0,
            timer: 0,
            board_y: 0.0,
            pane0_rotation: 0.0,
            pane1_rotation: 0.0,
            stick: StickControl::new(5, 2, 3, 2),
            pal_language: 0,
            tick_rate: 0,
        }
    }
}

impl ErrorPanel {
    pub fn get_status(&self) -> u8 {
        self.status
    }

    pub fn get_yes_no(&self) -> u8 {
        self.yes_no ^ 1
    }

    pub fn set_time_count_down_mode(&mut self) {
        self.time_count_down = 1;
    }

    pub fn cursor_scale(&self) -> f32 {
        if self.yes_no == 0 {
            ERROR_TALL
        } else {
            ERROR_SHORT
        }
    }

    pub fn cursor_x(&self) -> f32 {
        ERROR_CURSOR_X[usize::from(self.yes_no & 1)]
    }

    pub fn create(&mut self, screen: &Screen) {
        self.panes[0].rotation = screen.find("cur1").map_or(0, |pane| pane.rotation);
        self.panes[1].rotation = screen.find("cur2").map_or(0, |pane| pane.rotation);
        self.pane0_rotation = f32::from(deg2s(f32::from(self.panes[0].rotation))) / 32768.0 * 180.0;
        self.pane1_rotation = f32::from(deg2s(f32::from(self.panes[1].rotation))) / 32768.0 * 180.0;
        for pane in &mut self.panes {
            if let Some(layout) = screen.find(pane.tag) {
                pane.parts_set(layout);
            }
        }
        if let Some(pane) = self.panes.get_mut(2) {
            pane.set_string(&"A".repeat(ERROR_TEXT_BUFFER - 1));
        }
        self.message.clear();
        self.display_init();
        self.pane_trans_init();
    }

    pub fn initial(&mut self) {
        self.display_init();
        self.pane_trans_init();
    }

    pub fn delete(&mut self) {
        for pane in &mut self.panes {
            pane.found = false;
            pane.text.clear();
            pane.texture = None;
        }
        self.message.clear();
    }

    pub fn display_init(&mut self) {
        self.time_count_down = 0;
        self.flag_2fb = 0x5A;
        self.status = 0;
        self.state = 0;
    }

    pub fn pane_trans_init(&mut self) {
        self.timer = 0;
        for index in 3..8 {
            let pane = &mut self.panes[index];
            pane.pos_center = pane.pos_center_orig;
            pane.size = pane.size_orig;
            pane.cpos_move();
        }
        for index in 8..13 {
            let pane = &mut self.panes[index];
            pane.pos_center = pane.pos_center_orig;
            pane.size = pane.size_orig;
            pane.cpos_move();
        }
        for index in 0..3 {
            let pane = &mut self.panes[index];
            pane.pos_center = pane.pos_center_orig;
            pane.size = pane.size_orig;
            pane.cpos_move();
        }
        self.panes[0].now_alpha = ERROR_FRAME[1] as u8;
        self.panes[1].now_alpha = ERROR_FRAME[2] as u8;
        self.panes[0].user_area = ERROR_FRAME[3] as i16;
        self.panes[1].user_area = ERROR_FRAME[4] as i16;
        self.flag_2f8 = 0;
        self.flag_2f9 = 0;
        self.yes_no = 1;
        self.pane_trance_base(
            self.timer,
            ERROR_FRAME[0] as u8,
            ERROR_FRAME[5] as f32,
            0.0,
            0.0,
            0.0,
            1,
            1,
        );
        self.panes[0].hide();
        self.panes[1].hide();
    }

    pub fn close_message(&mut self) {
        self.timer = 0;
        self.state = 2;
        self.sounds.push(SE_ALERT_PANEL_OUT);
    }

    pub fn set_err_message(&mut self, msg_no: u32, mode: i32) {
        self.flag_2fa = mode as u8;
        self.sounds.push(SE_ALERT_PANEL_IN);
        self.initial();
        let buffer = self.pending_message(msg_no);
        let line_count = self.line_count(&buffer);
        if line_count + 2 < ERROR_LINES_BIG {
            self.resize_msg_board(line_count + 2);
        }
        let line_space = self.panes[2].get_line_space();
        let bottom_y = self.panes[2].pos_top_left.y + self.panes[2].size.y;
        self.board_y = bottom_y - line_space * 2.0;
        self.board_y -= self.panes[0].pos_top_left.y;
        if line_count > ERROR_LINE_BUMP_BIG {
            self.board_y += 20.0;
        }
        self.yn_cursor_init();
        self.set_message(&buffer);
        self.timer = 0;
        self.status = 1;
        self.state = 1;
    }

    pub fn pending_message(&self, msg_no: u32) -> String {
        let mut buffer = String::new();
        if msg_no == 0 {
            return buffer;
        }
        buffer.push('\u{1}');
        buffer.push_str(&format!("{msg_no}"));
        buffer
    }

    pub fn line_count(&self, text: &str) -> i32 {
        let bytes: Vec<char> = text.chars().collect();
        let mut count = 0;
        let mut at = 0usize;
        while at < bytes.len() {
            if bytes[at] == '\n' && bytes.get(at + 1).is_some_and(|c| *c != '\0') {
                count += 1;
            }
            at += 1;
        }
        count + 1
    }

    pub fn resize_msg_board(&mut self, lines: i32) {
        let pane2_height = self.panes[2].height();
        let scale = (pane2_height / ERROR_HEAD_H) * lines as f32 / pane2_height;
        self.panes[2].pane_scale_y(scale);
        self.panes[3].pane_scale_y(scale);
        let height3 = self.panes[3].size.y + ERROR_BORDER;
        self.panes[5].resize(self.panes[5].size.x, height3);
        let height7 = self.panes[7].size.y + ERROR_BORDER;
        self.panes[7].resize(self.panes[7].size.x, height7);
        self.panes[8].pane_scale_y(scale);
        let height8 = self.panes[8].size.y;
        self.panes[9].resize(self.panes[9].size.x, height8);
        self.panes[12].resize(self.panes[12].size.x, height8);
        let (x4, y3) = (self.panes[4].pos_top_left.x, self.panes[3].size.y);
        self.panes[4].move_to(x4, y3);
        for index in [5usize, 7, 9, 12] {
            let origin = self.panes[index].pos_top_left_orig;
            self.panes[index].move_to(origin.x, origin.y);
        }
        let x10 = self.panes[10].pos_top_left.x;
        let y10 = self.panes[8].size.y;
        self.panes[10].move_to(x10, y10);
    }

    pub fn set_message(&mut self, text: &str) {
        let skip = ERROR_CODE_SKIP[usize::from(self.pal_language) % ERROR_CODE_SKIP.len()];
        let mut own = String::new();
        self.flag_2f7 = 0;
        let mut at = 0usize;
        let raw: Vec<char> = text.chars().collect();
        while at < raw.len() && raw[at] != '\0' {
            if raw[at] == '\u{1}' {
                self.flag_2f7 = 1;
                self.panes[0].show();
                self.panes[1].show();
                at += 1;
                let mut taken = 0;
                while taken < skip && at < raw.len() {
                    own.push(raw[at]);
                    at += 1;
                    taken += 1;
                }
                own.push_str(ERROR_SEPARATOR);
            } else {
                own.push(raw[at]);
                at += 1;
            }
        }
        self.message = own;
    }

    pub fn move_step(&mut self, pad: &PadInput) {
        match self.state {
            0 => {}
            1 => self.show_msg_board(),
            2 => self.hide_msg_board(),
            3 => self.msg_disp_wait(pad),
            _ => self.yes_no_select_wait(pad),
        }
    }

    pub fn show_msg_board(&mut self) {
        let cond1 = self.pane_trance_base(
            self.timer,
            ERROR_FRAME[0] as u8,
            ERROR_FRAME[5] as f32,
            40.0,
            self.cursor_x() - self.panes[0].pos_top_left_orig.x,
            self.board_y + ERROR_CURSOR_Y,
            1,
            0,
        );
        let mut cond2 = 0;
        if cond1 == 1 {
            cond2 = self.pane_trance_base(
                self.timer - ERROR_FRAME[0] as i16,
                ERROR_FRAME[0] as u8,
                40.0,
                0.0,
                self.cursor_x() - self.panes[0].pos_top_left_orig.x,
                self.board_y + ERROR_CURSOR_Y,
                4,
                2,
            );
        }
        self.timer += 1;
        if cond1 == 1 && cond2 == 1 {
            if self.flag_2fa != 0 {
                self.state = 0;
            } else if self.flag_2f7 != 0 {
                self.state = 4;
            } else {
                self.state = 3;
            }
        }
    }

    pub fn hide_msg_board(&mut self) {
        let cond1 = self.pane_trance_base(
            self.timer,
            ERROR_FRAME[1] as u8,
            0.0,
            40.0,
            self.cursor_x() - self.panes[0].pos_top_left_orig.x,
            self.board_y + ERROR_CURSOR_Y,
            4,
            2,
        );
        let mut cond2 = 0;
        if cond1 == 1 {
            cond2 = self.pane_trance_base(
                self.timer - ERROR_FRAME[1] as i16,
                ERROR_FRAME[1] as u8,
                40.0,
                ERROR_FRAME[5] as f32,
                self.cursor_x() - self.panes[0].pos_top_left_orig.x,
                self.board_y + ERROR_CURSOR_Y,
                1,
                1,
            );
        }
        self.timer += 1;
        if cond1 == 1 && cond2 == 1 {
            self.time_count_down = 0;
            self.flag_2fb = 0x5A;
            self.status = 2;
            self.state = 0;
        }
    }

    pub fn msg_disp_wait(&mut self, pad: &PadInput) {
        if self.time_count_down == 1 && self.flag_2fb != 0 {
            self.flag_2fb -= 1;
        }
        if !pad.any_confirm() && self.flag_2fb != 0 {
            return;
        }
        self.timer = 0;
        self.state = 2;
    }

    pub fn yes_no_select_wait(&mut self, pad: &PadInput) {
        self.stick.check_trigger(pad.stick_value, pad.stick_angle);
        if pad.trig_a {
            self.state = 2;
        } else if pad.trig_b {
            self.yes_no = 1;
            self.yn_cursor_move();
            self.timer = 0;
            self.state = 2;
        } else if self.stick.check_left_trigger() {
            if self.yes_no != 0 {
                self.yes_no = 0;
                self.yn_cursor_move();
            }
        } else if self.stick.check_right_trigger() {
            if self.yes_no != 1 {
                self.yes_no = 1;
                self.yn_cursor_move();
            }
        }
        self.yn_cursor_anime();
    }

    pub fn yn_cursor_init(&mut self) {
        let scale = self.cursor_scale();
        self.panes[0].pane_scale_x(scale);
        self.panes[1].pane_scale_x(scale);
        let base0 = self.pane0_rotation - ERROR_FRAME[4] as f32;
        let base1 = self.pane1_rotation + ERROR_FRAME[4] as f32;
        let (x0, y0) = (self.panes[0].size.x / 2.0, self.panes[0].size.y / 2.0);
        self.panes[0].rotate(x0, y0, base0);
        let (x1, y1) = (self.panes[1].size.x / 2.0, self.panes[1].size.y / 2.0);
        self.panes[1].rotate(x1, y1, base1);
    }

    pub fn yn_cursor_move(&mut self) {
        self.panes[0].now_alpha = ERROR_FRAME[1] as u8;
        self.panes[1].now_alpha = ERROR_FRAME[2] as u8;
        self.panes[0].user_area = ERROR_FRAME[3] as i16;
        self.panes[1].user_area = ERROR_FRAME[4] as i16;
        self.flag_2f8 = 0;
        self.flag_2f9 = 0;
        let scale = self.cursor_scale();
        for index in 0..2 {
            let cursor_x = self.cursor_x();
            let origin_x = self.panes[index].pos_top_left_orig.x;
            let board_y = self.board_y + ERROR_CURSOR_Y;
            self.panes[index].pane_scale_x(scale);
            self.panes[index].pane_trans(cursor_x - origin_x, board_y);
        }
        let (x0, y0) = (self.panes[0].size.x / 2.0, self.panes[0].size.y / 2.0);
        self.panes[0].rotate(x0, y0, self.pane0_rotation - ERROR_FRAME[4] as f32);
        let (x1, y1) = (self.panes[1].size.x / 2.0, self.panes[1].size.y / 2.0);
        self.panes[1].rotate(x1, y1, self.pane1_rotation + ERROR_FRAME[4] as f32);
    }

    pub fn yn_cursor_anime(&mut self) {
        const XP1: [f32; 3] = [0.0, 1.0, 1.0];
        const RT1: [f32; 3] = [0.0, -3.0, 4.0];
        const XP2: [f32; 3] = [0.0, -1.0, -1.0];
        const RT2: [f32; 3] = [0.0, 4.0, -3.0];

        if self.panes[0].user_area == 0 {
            self.panes[0].user_area = ERROR_FRAME[3] as i16;
            let idx = self.flag_2f9 as usize;
            let cursor_x = self.cursor_x();
            let board_y = self.board_y + ERROR_CURSOR_Y;
            let origin0 = self.panes[0].pos_top_left_orig.x;
            let origin1 = self.panes[1].pos_top_left_orig.x;
            self.panes[0].pane_trans(cursor_x - origin0 + XP1[idx], board_y);
            self.panes[1].pane_trans(cursor_x - origin1 + XP2[idx], board_y);
            let (x0, y0) = (self.panes[0].size.x / 2.0, self.panes[0].size.y / 2.0);
            self.panes[0].rotate(
                x0,
                y0,
                self.pane0_rotation - ERROR_FRAME[4] as f32 + RT1[idx],
            );
            let (x1, y1) = (self.panes[1].size.x / 2.0, self.panes[1].size.y / 2.0);
            self.panes[1].rotate(
                x1,
                y1,
                self.pane1_rotation + ERROR_FRAME[4] as f32 + RT2[idx],
            );
            self.flag_2f9 = self.flag_2f9.wrapping_add(1);
            if self.flag_2f9 > 2 {
                self.flag_2f9 = 0;
            }
        } else {
            self.panes[0].user_area -= 1;
        }

        let mut value = value_increase(ERROR_FRAME[4], i32::from(self.panes[1].user_area), 0);
        if self.flag_2f8 != 0 {
            value = 1.0 - value;
        }
        let span = ERROR_FRAME[1] as f32 - ERROR_FRAME[2] as f32;
        let alpha = ERROR_FRAME[2] as f32 + span * value;
        self.panes[0].now_alpha = alpha as u8;
        self.panes[1].now_alpha = alpha as u8;

        if self.panes[1].user_area == 0 {
            self.panes[1].user_area = ERROR_FRAME[4] as i16;
            self.flag_2f8 ^= 1;
        } else {
            self.panes[1].user_area -= 1;
        }
    }

    pub fn pane_trance_base(
        &mut self,
        param_0: i16,
        param_1: u8,
        param_2: f32,
        param_3: f32,
        param_4: f32,
        offset_y: f32,
        param_6: u8,
        param_7: i32,
    ) -> i32 {
        if param_0 < 0 {
            return 0;
        }
        if param_0 > i16::from(param_1) {
            return 1;
        }
        let mut alpha = value_increase(i32::from(param_1), i32::from(param_0), param_6);
        let y = param_2 + alpha * (param_3 - param_2);
        self.panes[2].pane_trans(0.0, y);
        self.panes[3].pane_trans(0.0, y);
        self.panes[8].pane_trans(0.0, y);
        self.panes[0].pane_trans(param_4, offset_y + y);
        self.panes[1].pane_trans(param_4, offset_y + y);
        if param_7 != 2 {
            if param_7 == 1 {
                alpha = 1.0 - alpha;
            }
            for index in [2usize, 3, 8, 0, 1] {
                self.panes[index].set_now_alpha(alpha);
            }
        }
        0
    }
}

#[derive(Resource, Clone, Debug)]
pub struct SavePanes {
    pub wipe: [Pane; WIPE_PANES],
    pub mask: Pane,
    pub base: [Pane; BASE_PANES],
    pub title: Pane,
    pub title_label: Pane,
    pub cursor: Pane,
    pub yes: [Pane; MENU_PANES],
    pub no: [Pane; MENU_PANES],
}

impl Default for SavePanes {
    fn default() -> Self {
        Self {
            wipe: std::array::from_fn(|i| Pane::new(WIPE_TAGS[i])),
            mask: Pane::new(MASK_TAG),
            base: std::array::from_fn(|i| Pane::new(BASE_TAGS[i])),
            title: Pane::new(TITLE_TAG),
            title_label: Pane::new(TITLE_LABEL_TAG),
            cursor: Pane::new(CURSOR_TAG),
            yes: std::array::from_fn(|i| Pane::new(YES_KEY_TAGS[i])),
            no: std::array::from_fn(|i| Pane::new(NO_KEY_TAGS[i])),
        }
    }
}

impl SavePanes {
    pub fn bind(&mut self, screen: &Screen) {
        for pane in &mut self.wipe {
            if let Some(layout) = screen.find(pane.tag) {
                pane.parts_set(layout);
            }
        }
        if let Some(layout) = screen.find(MASK_TAG) {
            self.mask.parts_set(layout);
        }
        for pane in &mut self.base {
            if let Some(layout) = screen.find(pane.tag) {
                pane.parts_set(layout);
            }
        }
        for tag in [TITLE_TAG, TITLE_LABEL_TAG, CURSOR_TAG] {
            let layout = screen.find(tag).cloned();
            if let Some(layout) = layout {
                match tag {
                    TITLE_TAG => self.title.parts_set(&layout),
                    TITLE_LABEL_TAG => self.title_label.parts_set(&layout),
                    _ => self.cursor.parts_set(&layout),
                }
            }
        }
        for index in 0..MENU_PANES {
            let yes = screen.find(YES_KEY_TAGS[index]).cloned();
            if let Some(layout) = yes {
                self.yes[index].parts_set(&layout);
            }
            let no = screen.find(NO_KEY_TAGS[index]).cloned();
            if let Some(layout) = no {
                self.no[index].parts_set(&layout);
            }
        }
    }

    pub fn text_pane(&mut self, index: usize) -> &mut Pane {
        &mut self.base[index + 3]
    }
}

#[derive(Resource, Clone, Debug)]
pub struct SaveMenu {
    pub screen: Option<Screen>,
    pub error_screen: Option<Screen>,
    pub panes: SavePanes,
    pub error: ErrorPanel,
    pub stick: StickControl,
    pub proc: u8,
    pub flag_52d: u8,
    pub flag_52e: u8,
    pub flag_52f: u8,
    pub flag_530: u8,
    pub save_status: u8,
    pub flag_532: u8,
    pub flag_533: u8,
    pub flag_534: u8,
    pub flag_535: u8,
    pub flag_536: u8,
    pub use_type: u8,
    pub end_status: u8,
    pub flag_539: u8,
    pub flag_53a: u8,
    pub flag_53b: u8,
    pub error_flag: u8,
    pub error_type: u8,
    pub op_result: i32,
    pub timer: i16,
    pub flag_546: [u8; 14],
    pub text_a: String,
    pub text_b: String,
    pub sounds: Vec<u32>,
    pub title_label_rotation: f32,
}

impl Default for SaveMenu {
    fn default() -> Self {
        Self {
            screen: None,
            error_screen: None,
            panes: SavePanes::default(),
            error: ErrorPanel::default(),
            stick: StickControl::new(5, 2, 3, 2),
            proc: proc_save_wait,
            flag_52d: 0,
            flag_52e: 0,
            flag_52f: 0,
            flag_530: 0,
            save_status: 0,
            flag_532: 0,
            flag_533: 0,
            flag_534: 0,
            flag_535: 0,
            flag_536: 0,
            use_type: 0,
            end_status: 0,
            flag_539: 0,
            flag_53a: 0,
            flag_53b: 0,
            error_flag: 0,
            error_type: 0,
            op_result: 0,
            timer: 0,
            flag_546: [0; 14],
            text_a: String::new(),
            text_b: String::new(),
            sounds: Vec::new(),
            title_label_rotation: 0.0,
        }
    }
}

impl SaveMenu {
    pub fn get_data_buf_ptr(&self) -> String {
        self.text_a.clone()
    }

    pub fn get_end_status(&self) -> u8 {
        self.end_status
    }

    pub fn get_save_status(&self) -> u8 {
        self.save_status
    }

    pub fn set_error_flag(&mut self, error_flag: u8) {
        self.error_flag = error_flag;
    }

    pub fn set_error_type(&mut self, error_type: u8) {
        self.error_type = error_type;
    }

    pub fn set_use_type(&mut self, use_type: u8) {
        self.use_type = use_type;
    }

    pub fn sound(&mut self, se: u32) {
        self.sounds.push(se);
    }

    pub fn create(&mut self, screen: Screen, error_screen: Screen) {
        self.screen = Some(screen);
        self.error_screen = Some(error_screen);
        self.error
            .create(self.error_screen.as_ref().expect("error screen bound"));
        self.screen_set();
        self.display_init();
        self.pane_trans_init();
    }

    pub fn delete(&mut self) {
        self.screen = None;
        self.error_screen = None;
        self.panes = SavePanes::default();
        self.error.delete();
        self.stick = StickControl::new(5, 2, 3, 2);
    }

    pub fn initialize(&mut self) {
        self.display_init();
        self.pane_trans_init();
    }

    pub fn initialize_ex(&mut self, msgs: &dyn MsgSource) {
        let msg = msgs.message(0x3A);
        self.text_a = msg.clone();
        self.text_b = msg;
    }

    pub fn screen_set(&mut self) {
        let Some(screen) = self.screen.clone() else {
            return;
        };
        self.panes.bind(&screen);
        let fill = "A".repeat(TEXT_BUFFER_A - 1);
        for index in [3usize, 4] {
            self.panes.base[index].set_string(&fill);
            self.panes.base[index].set_font_size(
                self.panes.base[index].font_size_x,
                self.panes.base[index].font_size_y,
            );
        }
        self.text_a.clear();
        self.text_b.clear();
        self.panes.base[3].set_string(&self.text_a);
        self.panes.base[4].set_string(&self.text_b);
        self.title_label_rotation =
            f32::from(deg2s(f32::from(self.panes.title_label.rotation))) / 32768.0 * 180.0;
    }

    pub fn pane_trans_init(&mut self) {
        self.timer = 0;
        self.pane_scale_alpha_wipe(self.timer, IDX_23, f32::from(IDX_26), 1, 0);
        self.pane_alpha_mask(self.timer, IDX_05, 1, 0);
        self.pane_trance_base(self.timer, IDX_08, IDX_0c as f32, 0.0, 1, 0);
        self.pane_trance_title(self.timer, IDX_10, IDX_14 as f32, 0.0, 1, 0);
        let yes_y = IDX_1e as f32;
        let no_y = IDX_1e as f32;
        self.pane_trance_menu(self.timer, IDX_18, MenuSlot::Yes, yes_y, 0.0, 1, 0);
        self.pane_trance_menu(self.timer, IDX_18, MenuSlot::No, no_y, 0.0, 1, 0);
        self.panes.cursor.hide();
    }

    pub fn display_init(&mut self) {
        self.flag_536 = 0;
        self.error_flag = 0;
        self.error_type = 0;
        self.flag_53b = 0;
        self.proc = proc_save_wait;
        self.save_status = 1;
        self.flag_53a = 1;
        self.end_status = 2;
        self.flag_535 = 0;
        self.cursor_alpha_init();
        self.flag_52f = 0;
        self.cursor_move();

        self.panes.base[3].set_now_alpha(1.0);
        self.panes.base[4].set_now_alpha(0.0);

        self.flag_539 = 1;

        let msg = if self.use_type == 3 {
            MSG_ENDING_TITLE
        } else {
            MSG_TITLE
        };
        self.text_a = format!("\u{1}{msg}");

        self.text_b.clear();
    }

    pub fn open(&mut self, env: &mut SaveEnv<'_>) -> bool {
        self.open_normal(env)
    }

    pub fn close(&mut self, env: &mut SaveEnv<'_>) -> bool {
        self.close_normal(env)
    }

    pub fn open_normal(&mut self, env: &mut SaveEnv<'_>) -> bool {
        let mut rt = false;
        match self.use_type {
            1 => rt = self.open_for_collect(env),
            0 => rt = self.open_for_item(env),
            2 => rt = self.open_for_game_over(env),
            3 => {
                rt = if self.error_flag != 0 {
                    if self.error_type == 2 {
                        self.open_for_ending2()
                    } else {
                        true
                    }
                } else {
                    self.open_for_ending()
                };
            }
            _ => {}
        }
        self.timer += 1;
        if rt {
            self.timer = 0;
            self.panes.cursor.show();
            if self.use_type == 3 {
                if self.error_flag != 0 {
                    if self.error_type == 2 {
                        self.flag_53a = 0;
                        self.flag_535 = 1;
                        self.proc = proc_can_not_save;
                    } else {
                        self.flag_53a = 1;
                        self.proc = proc_mem_card_check;
                    }
                } else {
                    self.flag_53a = 0;
                    self.proc = proc_save_question;
                }
            } else {
                self.flag_53a = 0;
                self.proc = proc_save_question;
            }
            self.save_status = 2;
            return true;
        }
        false
    }

    pub fn close_normal(&mut self, env: &mut SaveEnv<'_>) -> bool {
        let mut rt = false;
        match self.use_type {
            1 => rt = self.close_for_collect(env),
            0 => rt = self.close_for_item(env),
            2 => rt = self.close_for_game_over(),
            3 => rt = self.close_for_ending(),
            _ => {}
        }
        self.timer += 1;
        if rt {
            self.proc = proc_save_wait;
            self.save_status = 0;
            self.timer = 0;
            self.flag_53a = 1;
            return true;
        }
        false
    }

    pub fn open_for_collect(&mut self, env: &mut SaveEnv<'_>) -> bool {
        if self.timer == 0 {
            self.sound(SE_SAVE_PANEL_IN);
        }
        env.menu.set_push_menu_button(PUSH_MENU_BUTTON_NONE);
        let mut second = false;
        let alpha_mask = self.pane_alpha_mask(self.timer - IDX_06 as i16, IDX_05, 1, 0);
        let base = self.pane_trance_base(
            self.timer - IDX_09 as i16,
            IDX_08,
            IDX_0c as f32,
            40.0,
            1,
            0,
        );
        if base {
            second = self.pane_trance_base(
                self.timer - (IDX_09 as i16 + IDX_08 as i16),
                IDX_08,
                40.0,
                0.0,
                4,
                2,
            );
        }
        let alpha_wipe =
            self.pane_scale_alpha_wipe(self.timer - IDX_24 as i16, IDX_23, f32::from(IDX_26), 1, 0);
        let title =
            self.pane_trance_title(self.timer - IDX_11 as i16, IDX_10, IDX_14 as f32, 0.0, 1, 0);
        let menu = self.menu_up();
        alpha_mask && base && second && alpha_wipe && title && menu
    }

    pub fn open_for_item(&mut self, env: &mut SaveEnv<'_>) -> bool {
        if self.timer == 0 {
            self.sound(SE_SAVE_PANEL_IN);
        }
        env.menu.set_push_menu_button(PUSH_MENU_BUTTON_NONE);
        let mut second = false;
        let alpha_mask = self.pane_alpha_mask(self.timer - IDX_06 as i16, IDX_05, 1, 0);
        let base = self.pane_trance_base(
            self.timer - IDX_09 as i16,
            IDX_08,
            IDX_0c as f32,
            40.0,
            1,
            0,
        );
        if base {
            second = self.pane_trance_base(
                self.timer - (IDX_09 as i16 + IDX_08 as i16),
                IDX_08,
                40.0,
                0.0,
                4,
                2,
            );
        }
        let alpha_wipe =
            self.pane_scale_alpha_wipe(self.timer - IDX_24 as i16, IDX_23, f32::from(IDX_26), 1, 0);
        let title =
            self.pane_trance_title(self.timer - IDX_11 as i16, IDX_10, IDX_14 as f32, 0.0, 1, 0);
        let menu = self.menu_up();
        alpha_mask && base && second && alpha_wipe && title && menu
    }

    pub fn close_for_collect(&mut self, env: &mut SaveEnv<'_>) -> bool {
        if self.timer == 0 {
            self.sound(SE_SAVE_PANEL_OUT);
        }
        let mut second = false;
        env.menu.set_push_menu_button(PUSH_MENU_BUTTON_CLOSE);
        let alpha_mask = self.pane_alpha_mask(self.timer - IDX_07 as i16, IDX_05, 0, 1);
        let base = self.pane_trance_base(self.timer - IDX_0a as i16, IDX_08, 0.0, 40.0, 4, 2);
        if base {
            second = self.pane_trance_base(
                self.timer - (IDX_0a as i16 + IDX_08 as i16),
                IDX_08,
                0.0,
                IDX_0c as f32,
                0,
                1,
            );
        }
        let alpha_wipe = self.pane_scale_alpha_wipe(self.timer - IDX_25 as i16, IDX_23, 1.0, 0, 1);
        let title =
            self.pane_trance_title(self.timer - IDX_12 as i16, IDX_10, 0.0, IDX_14 as f32, 0, 1);
        let menu = if self.flag_535 != 0 {
            true
        } else {
            self.menu_down()
        };
        alpha_mask && base && second && alpha_wipe && title && menu
    }

    pub fn close_for_item(&mut self, env: &mut SaveEnv<'_>) -> bool {
        if self.timer == 0 {
            self.sound(SE_SAVE_PANEL_OUT);
        }
        let mut second = false;
        env.menu.set_push_menu_button(1);
        let alpha_mask = self.pane_alpha_mask(self.timer - IDX_07 as i16, IDX_05, 0, 1);
        let base = self.pane_trance_base(self.timer - IDX_0a as i16, IDX_08, 0.0, 40.0, 4, 2);
        if base {
            second = self.pane_trance_base(
                self.timer - (IDX_0a as i16 + IDX_08 as i16),
                IDX_08,
                0.0,
                IDX_0c as f32,
                0,
                1,
            );
        }
        let alpha_wipe = self.pane_scale_alpha_wipe(self.timer - IDX_25 as i16, IDX_23, 1.0, 0, 1);
        let title =
            self.pane_trance_title(self.timer - IDX_12 as i16, IDX_10, 0.0, IDX_14 as f32, 0, 1);
        let menu = if self.flag_535 != 0 {
            true
        } else {
            self.menu_down()
        };
        alpha_mask && base && second && alpha_wipe && title && menu
    }

    pub fn open_for_game_over(&mut self, env: &mut SaveEnv<'_>) -> bool {
        if self.timer == 0 {
            self.sound(SE_SAVE_PANEL_IN);
        }
        let mut second = false;
        let base =
            self.pane_trance_base(self.timer - IDX_09 as i16, IDX_08, IDX_0c as f32, 0.0, 1, 0);
        let temp = self.timer - (IDX_09 as i16 + IDX_08 as i16 - 2);
        if temp == 0 && self.flag_539 != 0 {
            for x in OPEN_FOR_GAME_OVER_X {
                env.particles
                    .set_2d_menu_fore(PARTICLE_OPEN_MENU, x, OPEN_FOR_GAME_OVER_Y, 0.0);
            }
            self.flag_539 = 0;
        }
        if base {
            second = self.pane_trance_base(
                self.timer - (IDX_09 as i16 + IDX_08 as i16),
                IDX_08,
                0.0,
                -20.0,
                5,
                2,
            );
        }
        let title =
            self.pane_trance_title(self.timer - IDX_11 as i16, IDX_10, IDX_14 as f32, 0.0, 1, 0);
        let menu = self.menu_up();
        base && second && title && menu
    }

    pub fn close_for_game_over(&mut self) -> bool {
        match self.end_status {
            1 => self.close_for_game_over_1(),
            0 => self.close_for_game_over_2(),
            _ => false,
        }
    }

    pub fn close_for_game_over_1(&mut self) -> bool {
        if self.timer == 0 {
            self.sound(SE_SAVE_PANEL_OUT);
        }
        let mut second = false;
        let base = self.pane_trance_base(self.timer - IDX_0a as i16, IDX_08, 0.0, 40.0, 4, 2);
        if base {
            second = self.pane_trance_base(
                self.timer - (IDX_0a as i16 + IDX_08 as i16),
                IDX_08,
                0.0,
                IDX_0c as f32,
                0,
                1,
            );
        }
        let title =
            self.pane_trance_title(self.timer - IDX_12 as i16, IDX_10, 0.0, IDX_14 as f32, 0, 1);
        let menu = if self.flag_535 != 0 {
            true
        } else {
            self.menu_down()
        };
        base && second && title && menu
    }
    pub fn close_for_game_over_2(&mut self) -> bool {
        let mut title_rt = false;
        let mut base_rt = false;
        let mut yes_rt = false;
        let mut no_rt = false;

        let first =
            self.pane_trance_title(self.timer - IDX_13 as i16, IDX_10, 0.0, IDX_2c as f32, 1, 2);
        self.pane_rotate(
            self.timer - IDX_13 as i16,
            IDX_10,
            TitleSlot::Title,
            73.5,
            self.panes.title.size_orig.y / 2.0,
            7.0,
            0,
        );
        if first {
            title_rt = self.pane_trance_title(
                self.timer - (IDX_13 as i16 + IDX_10 as i16),
                IDX_10,
                IDX_2c as f32,
                IDX_16 as f32,
                0,
                1,
            );
        }

        let first =
            self.pane_trance_base(self.timer - IDX_0b as i16, IDX_08, 0.0, IDX_2c as f32, 1, 2);
        self.pane_rotate(
            self.timer - IDX_0b as i16,
            IDX_10,
            TitleSlot::Base,
            257.0,
            0.0,
            -7.0,
            0,
        );
        if first {
            base_rt = self.pane_trance_base(
                self.timer - (IDX_0b as i16 + IDX_08 as i16),
                IDX_08,
                IDX_2c as f32,
                IDX_0e as f32,
                0,
                1,
            );
        }

        let first = self.pane_trance_menu(
            self.timer - IDX_1b as i16,
            IDX_18,
            MenuSlot::Yes,
            0.0,
            IDX_2c as f32,
            1,
            2,
        );
        self.pane_rotate(
            self.timer - IDX_1b as i16,
            IDX_10,
            TitleSlot::Yes,
            30.0,
            32.0,
            -7.0,
            0,
        );
        if first {
            yes_rt = self.pane_trance_menu(
                self.timer - (IDX_1b as i16 + IDX_18 as i16),
                IDX_18,
                MenuSlot::Yes,
                IDX_2c as f32,
                IDX_1e as f32,
                0,
                1,
            );
        }

        let first = self.pane_trance_menu(
            self.timer - (IDX_1b as i16 + IDX_1c as i16),
            IDX_18,
            MenuSlot::No,
            0.0,
            IDX_2c as f32,
            1,
            2,
        );
        self.pane_rotate(
            self.timer - (IDX_1b as i16 + IDX_1c as i16),
            IDX_10,
            TitleSlot::No,
            30.0,
            32.0,
            7.0,
            0,
        );
        if first {
            no_rt = self.pane_trance_menu(
                self.timer - (IDX_1b as i16 + IDX_1c as i16 + IDX_18 as i16),
                IDX_18,
                MenuSlot::No,
                IDX_2c as f32,
                IDX_1e as f32,
                0,
                1,
            );
        }

        if title_rt && base_rt && yes_rt && no_rt {
            self.pane_rotate(
                0,
                IDX_10,
                TitleSlot::Title,
                73.5,
                self.panes.title.size_orig.y / 2.0,
                0.0,
                0,
            );
            self.pane_rotate(0, IDX_10, TitleSlot::Base, 250.5, 0.0, 0.0, 0);
            self.pane_rotate(0, IDX_10, TitleSlot::Yes, 30.0, 32.0, 0.0, 0);
            self.pane_rotate(0, IDX_10, TitleSlot::No, 30.0, 32.0, 0.0, 0);
            return true;
        }
        false
    }
    pub fn open_for_ending(&mut self) -> bool {
        if self.timer == 0 {
            self.sound(SE_SAVE_PANEL_IN);
        }
        let mut second = false;
        let base =
            self.pane_trance_base(self.timer - IDX_09 as i16, IDX_08, IDX_0c as f32, 0.0, 1, 0);
        if base {
            second = self.pane_trance_base(
                self.timer - (IDX_09 as i16 + IDX_08 as i16),
                IDX_08,
                0.0,
                -20.0,
                5,
                2,
            );
        }
        let title =
            self.pane_trance_title(self.timer - IDX_11 as i16, IDX_10, IDX_14 as f32, 0.0, 1, 0);
        let menu = self.menu_up();
        base && second && title && menu
    }
    pub fn open_for_ending2(&mut self) -> bool {
        if self.timer == 0 {
            self.sound(SE_SAVE_PANEL_IN);
        }
        let mut second = false;
        let base =
            self.pane_trance_base(self.timer - IDX_09 as i16, IDX_08, IDX_0c as f32, 0.0, 1, 0);
        if base {
            second = self.pane_trance_base(
                self.timer - (IDX_09 as i16 + IDX_08 as i16),
                IDX_08,
                0.0,
                -20.0,
                5,
                2,
            );
        }
        let title =
            self.pane_trance_title(self.timer - IDX_11 as i16, IDX_10, IDX_14 as f32, 0.0, 1, 0);
        base && second && title
    }
    pub fn close_for_ending(&mut self) -> bool {
        if self.timer == 0 {
            self.sound(SE_SAVE_PANEL_OUT);
        }
        let mut second = false;
        let base = self.pane_trance_base(self.timer - IDX_0a as i16, IDX_08, 0.0, 40.0, 4, 2);
        if base {
            second = self.pane_trance_base(
                self.timer - (IDX_0a as i16 + IDX_08 as i16),
                IDX_08,
                0.0,
                IDX_0c as f32,
                0,
                1,
            );
        }
        let title =
            self.pane_trance_title(self.timer - IDX_12 as i16, IDX_10, 0.0, IDX_14 as f32, 0, 1);
        let menu = if self.flag_535 != 0 {
            true
        } else {
            self.menu_down()
        };
        base && second && title && menu
    }
    pub fn move_step(&mut self, env: &mut SaveEnv<'_>) {
        if env.system.is_reset() {
            return;
        }
        let probe = env.card.get_probe_stat();
        if (probe == 0 || probe == 1) && !env.system.is_reset() {
            if self.flag_53b == 0 && self.error.get_status() == 1 {
                self.error.close_message();
                self.flag_533 = 0xFF;
                self.flag_52d = proc_mem_card_check;
                self.proc = proc_mem_card_err_msg_wait_key3;
            }
            env.card.clear_probe_stat();
        }
        self.error.move_step(&env.pad);
        match dispatch(self.proc) {
            Proc::NoSave => self.no_save(),
            Proc::NoSave2 => self.no_save2(),
            Proc::SaveQuestion => self.save_question(env),
            Proc::MemCardCheck => self.mem_card_check(env),
            Proc::OpenSaveMenu => self.open_save_menu(),
            Proc::CloseSaveMenu => self.close_save_menu(),
            Proc::MemCardErrMsgWaitKey => self.mem_card_err_msg_wait_key(),
            Proc::MemCardErrMsgWaitKey2 => self.mem_card_err_msg_wait_key2(),
            Proc::MemCardErrMsgWaitKey3 => self.mem_card_err_msg_wait_key3(),
            Proc::MemCardErrGoIplSel => self.mem_card_err_go_ipl_sel(),
            Proc::MemCardErrGoIplSel2 => self.mem_card_err_go_ipl_sel2(env),
            Proc::MemCardErrMsgWaitFormatSel => self.mem_card_err_msg_wait_format_sel(),
            Proc::MemCardErrMsgWaitFormatSel2 => self.mem_card_err_msg_wait_format_sel2(env),
            Proc::MemCardFormat => self.mem_card_format(env),
            Proc::MemCardFormatCheck => self.mem_card_format_check(),
            Proc::MemCardMakeGameFileSel => self.mem_card_make_game_file_sel(env),
            Proc::MemCardMakeGameFile => self.mem_card_make_game_file(env),
            Proc::MemCardMakeGameFileCheck => self.mem_card_make_game_file_check(env),
            Proc::MemCardDataLoadWait => self.mem_card_data_load_wait(env),
            Proc::MemCardDataLoadWait2 => self.mem_card_data_load_wait2(),
            Proc::MemCardDataSave => self.mem_card_data_save(env),
            Proc::MemCardDataSaveUpMenu => self.mem_card_data_save_up_menu(),
            Proc::MemCardDataSaveSel => self.mem_card_data_save_sel(env),
            Proc::MemCardDataSaveDownMenu => self.mem_card_data_save_down_menu(env),
            Proc::MemCardDataSaveWait => self.mem_card_data_save_wait(env),
            Proc::MsgWait => self.msg_wait(),
            Proc::MsgWait2 => self.msg_wait2(env),
            Proc::SaveEndWait => self.save_end_wait(env),
            Proc::CanNotSave => self.can_not_save(env),
            Proc::CanNotSave2 => self.can_not_save2(),
            Proc::GameContinue => self.game_continue(),
            Proc::GameContinue2 => self.game_continue2(env),
            Proc::GameContinue3 => self.game_continue3(env),
            Proc::GameContinue4 => self.game_continue4(),
            Proc::EndingNoSave => self.ending_no_save(),
            Proc::EndingNoSave2 => self.ending_no_save2(env),
            Proc::EndingNoSave3 => self.ending_no_save3(),
            Proc::EndingDataCheck => self.ending_data_check(env),
            Proc::SaveWait => {}
        }
    }

    pub fn no_save(&mut self) {
        self.flag_533 = 0xFF;
        self.error.set_err_message(MSG_CARD_UNAVAILABLE, 0);
        self.proc = proc_mem_card_err_msg_wait_key3;
        self.flag_52d = proc_no_save2;
    }
    pub fn no_save2(&mut self) {
        self.proc = proc_save_wait;
        self.save_status = 3;
    }

    pub fn save_question(&mut self, env: &mut SaveEnv<'_>) {
        let answered = if self.use_type == 2 || self.use_type == 3 {
            self.yes_no_select2(env, 0)
        } else {
            self.yes_no_select(env, 0)
        };
        if !answered {
            return;
        }
        if self.flag_52f == 0 {
            self.text_b.clear();
            self.proc = proc_mem_card_check;
        } else if self.flag_52f == 2 {
            self.proc = proc_save_wait;
            self.save_status = 3;
        } else {
            self.timer = 0;
            match self.use_type {
                0 | 1 | 2 => {
                    self.text_b = format!("\u{1}{}", MESSAGE_TABLE[0]);
                    self.proc = proc_game_continue;
                }
                3 => {
                    self.text_b = format!("\u{1}{MSG_ENDING_CONTINUE}");
                    self.proc = proc_ending_no_save;
                }
                _ => {}
            }
        }
    }

    pub fn mem_card_check(&mut self, env: &mut SaveEnv<'_>) {
        match env.card.get_status() {
            CARD_STAT_WAIT => {
                self.flag_532 = 0x29;
                self.proc = proc_close_save_menu;
                self.flag_52d = proc_mem_card_err_msg_wait_key;
            }
            8 => {
                self.flag_532 = 0x2A;
                self.proc = proc_close_save_menu;
                self.flag_52d = proc_mem_card_err_msg_wait_key;
            }
            9 => {
                self.flag_532 = 0x2B;
                self.proc = proc_close_save_menu;
                self.flag_52d = proc_mem_card_err_msg_wait_key;
            }
            CARD_STAT_WRONG_DEVICE => {
                self.flag_532 = 0x2C;
                self.proc = proc_close_save_menu;
                self.flag_52d = proc_mem_card_err_msg_wait_key;
            }
            CARD_STAT_ENCODING | CARD_STAT_ERROR => {
                self.flag_532 = 0x2d;
                self.proc = proc_close_save_menu;
                self.flag_52d = proc_mem_card_format;
            }
            11 | CARD_STAT_IOERROR => {
                self.flag_532 = 0x2E;
                self.proc = proc_close_save_menu;
                self.flag_52d = proc_mem_card_err_msg_wait_key2;
                self.flag_533 = 0x62;
                self.flag_52e = 7;
            }
            CARD_STAT_CREATE => {
                env.card.set_pict_data_ptr(None);
                env.card.load();
                self.proc = proc_mem_card_data_load_wait;
            }
            CARD_STAT_RESTORE => {
                self.flag_532 = 0x34;
                self.proc = proc_close_save_menu;
                self.flag_52d = proc_mem_card_make_game_file_sel;
            }
            _ => {}
        }
    }

    pub fn open_save_menu(&mut self) {
        if self.open_save() {
            self.flag_53a = 0;
            self.panes.cursor.show();
            self.cursor_alpha_init();
            self.proc = proc_save_question;
        }
    }

    pub fn close_save_menu(&mut self) {
        let rt = if self.flag_53a == 1 {
            true
        } else {
            self.close_save()
        };
        if rt {
            self.flag_53a = 1;
            let msg = if self.use_type == 3 {
                MSG_ENDING_TITLE
            } else {
                MSG_TITLE
            };
            self.text_a = format!("\u{1}{msg}");
            self.error.set_err_message(u32::from(self.flag_532), 0);
            self.proc = self.flag_52d;
        }
    }
    pub fn mem_card_err_msg_wait_key(&mut self) {
        if self.error.get_status() == 2 {
            self.proc = proc_open_save_menu;
        }
    }

    pub fn mem_card_err_msg_wait_key2(&mut self) {
        if self.error.get_status() == 2 {
            if self.flag_533 != 0xFF {
                self.error.set_err_message(u32::from(self.flag_533), 0);
                self.flag_533 = 0xFF;
                self.proc = proc_mem_card_err_msg_wait_key2;
            } else {
                self.error.set_err_message(MSG_RECOVER_YES, 0);
                self.proc = self.flag_52e;
            }
        }
    }
    pub fn mem_card_err_msg_wait_key3(&mut self) {
        if self.error.get_status() == 2 {
            if self.flag_533 != 0xFF {
                let msg_no = u32::from(self.flag_533);
                self.error.set_err_message(msg_no, 0);
                self.flag_533 = 0xFF;
            }
            self.proc = self.flag_52d;
        }
    }
    pub fn mem_card_err_go_ipl_sel(&mut self) {
        if self.error.get_status() == 2 {
            if self.error.get_yes_no() != 0 {
                self.error.set_err_message(MSG_RESTART, 0);
                self.proc = proc_mem_card_err_go_ipl_sel2;
            } else {
                self.proc = proc_open_save_menu;
            }
        }
    }

    pub fn mem_card_err_go_ipl_sel2(&mut self, env: &mut SaveEnv<'_>) {
        if self.error.get_status() == 2 {
            if self.error.get_yes_no() != 0 {
                env.system.os_reset = true;
            } else {
                self.proc = proc_open_save_menu;
            }
        }
    }

    pub fn mem_card_err_msg_wait_format_sel(&mut self) {
        if self.error.get_status() == 2 {
            if self.error.get_yes_no() != 0 {
                self.error.set_err_message(MSG_NO_SPACE, 0);
                self.proc = proc_mem_card_err_msg_wait_format_sel;
            } else {
                self.error.set_err_message(0x2F, 0);
                self.proc = proc_mem_card_err_msg_wait_key;
            }
        }
    }

    pub fn mem_card_err_msg_wait_format_sel2(&mut self, env: &mut SaveEnv<'_>) {
        if self.error.get_status() == 2 {
            if self.error.get_yes_no() != 0 {
                env.card.format();
                self.error.set_err_message(MSG_NO_FILE, 1);
                self.proc = proc_mem_card_err_msg_wait_format_sel2;
                self.flag_53b = 1;
            } else {
                self.error.set_err_message(0x2F, 0);
                self.proc = proc_mem_card_err_msg_wait_key;
            }
        }
    }

    pub fn mem_card_format(&mut self, env: &mut SaveEnv<'_>) {
        self.op_result = env.card.format_sync();
        if self.op_result != 0 {
            self.error.close_message();
            self.proc = proc_mem_card_format_check;
        }
    }

    pub fn mem_card_format_check(&mut self) {
        if self.error.get_status() == 2 {
            if self.op_result == 2 {
                self.error.set_err_message(MSG_FORMAT_FAIL, 0);
                self.proc = proc_mem_card_err_msg_wait_key3;
                self.flag_52d = proc_mem_card_check;
            } else if self.op_result == 1 {
                self.error.set_err_message(MSG_FORMAT_DONE, 0);
                self.error.set_time_count_down_mode();
                self.proc = proc_mem_card_err_msg_wait_key3;
                self.flag_52d = proc_mem_card_check;
            }
            self.flag_53b = 0;
        }
    }
    pub fn mem_card_make_game_file_sel(&mut self, env: &mut SaveEnv<'_>) {
        if self.error.get_status() == 2 {
            if self.error.get_yes_no() != 0 {
                for slot in 0..SLOTS {
                    env.card.set_init_data_to_card(slot);
                    env.card.set_checksum_game_data(slot);
                }
                env.card.save();
                self.error.set_err_message(MSG_GAME_FILE_WRITING, 1);
                self.proc = proc_mem_card_make_game_file;
                self.flag_53b = 1;
            } else {
                self.error.set_err_message(MSG_GAME_FILE_NO, 0);
                self.proc = proc_mem_card_err_msg_wait_key;
            }
        }
    }

    pub fn mem_card_make_game_file(&mut self, env: &mut SaveEnv<'_>) {
        self.op_result = env.card.save_sync();
        if self.op_result != 0 {
            self.error.close_message();
            self.proc = proc_mem_card_make_game_file_check;
        }
    }

    pub fn mem_card_make_game_file_check(&mut self, env: &mut SaveEnv<'_>) {
        if self.error.get_status() == 2 {
            if self.op_result == 2 {
                self.error.set_err_message(MSG_GAME_FILE_FAIL, 0);
                self.proc = proc_mem_card_err_msg_wait_key3;
                self.flag_52d = proc_mem_card_check;
            } else if self.op_result == 1 {
                env.card.set_new_file(1);
                self.error.set_err_message(MSG_GAME_FILE_NEW, 0);
                self.proc = proc_mem_card_err_msg_wait_key;
            }
            self.flag_53b = 0;
        }
    }

    pub fn mem_card_data_load_wait(&mut self, env: &mut SaveEnv<'_>) {
        let rt = env.card.load_sync();
        if rt == 0 {
            return;
        }
        if rt == 2 {
            self.proc = proc_mem_card_check;
        } else if rt == 1 {
            if self.flag_53a == 1 {
                self.proc = proc_open_save_menu;
            } else if self.use_type == 3 {
                self.proc = proc_ending_data_check;
            } else {
                self.proc = proc_mem_card_data_load_wait2;
            }
        }
    }

    pub fn mem_card_data_load_wait2(&mut self) {
        let menu_rt = self.menu_down();
        let alpha_msg_rt = self.pane_alpha_msg_txt(self.timer, IDX_29);

        self.timer += 1;
        if menu_rt && alpha_msg_rt {
            self.flag_536 ^= 1;
            self.timer = 0;
            self.flag_535 = 1;
            self.proc = proc_mem_card_data_save;
        }
    }

    pub fn mem_card_data_save(&mut self, env: &mut SaveEnv<'_>) {
        let can_save = if env.card.get_new_file() == 1 {
            true
        } else if env.card.get_no_file() != 0 {
            false
        } else {
            env.card.get_mem_card_check_id() == env.card.get_card_serial_no()
        };

        if can_save {
            env.card.exchange_player_recollection_data();
            env.card.put_save(env.card.get_save_table());
            env.card.set_game_start_stage();
            let slot = usize::from(env.card.get_data_num());
            let test = env.card.test_checksum_game_data(slot);
            if test {
                if let Some(save) = env.pending_save.clone() {
                    env.card.set_memory_to_card(slot, save);
                }
                self.text_b = format!("\u{1}{MSG_SAVING}");
                self.data_write(env);
                self.proc = proc_mem_card_data_save_wait;
            } else {
                self.sound(SE_ALERT_DATA);
                self.text_b = format!("\u{1}{MSG_BAD_CHECKSUM2}");
                self.proc = proc_mem_card_data_save_up_menu;
            }
        } else {
            self.sound(SE_ALERT_DATA);
            self.text_b = format!("\u{1}{MSG_BAD_CHECKSUM}");
            self.proc = proc_can_not_save;
        }
    }

    pub fn mem_card_data_save_up_menu(&mut self) {
        let menu_rt = self.menu_up();
        let alpha_msg_rt = self.pane_alpha_msg_txt(self.timer, IDX_29);

        self.timer += 1;
        if menu_rt && alpha_msg_rt {
            self.flag_536 ^= 1;
            self.timer = 0;
            self.flag_535 = 0;
            self.cursor_alpha_init();
            self.flag_52f = 1;
            self.cursor_move();
            self.panes.cursor.show();
            self.proc = proc_mem_card_data_save_sel;
        }
    }

    pub fn mem_card_data_save_sel(&mut self, env: &mut SaveEnv<'_>) {
        if self.yes_no_select(env, 0) {
            if self.flag_52f == 0 {
                self.proc = proc_mem_card_data_save_down_menu;
            } else {
                self.cursor_alpha_init();
                self.flag_52f = 0;
                self.cursor_move();
                self.panes.cursor.show();
                self.text_b = format!("\u{1}{MSG_TITLE}");
                self.proc = proc_msg_wait;
                self.flag_52d = proc_save_question;
            }
        }
    }

    pub fn mem_card_data_save_down_menu(&mut self, env: &mut SaveEnv<'_>) {
        let menu_rt = self.menu_down();
        let temp_r5 = true;

        self.timer += 1;
        if menu_rt && temp_r5 {
            self.timer = 0;
            self.flag_535 = 1;
            self.text_b = format!("\u{1}{MSG_SAVING}");
            self.data_write(env);
            self.proc = proc_mem_card_data_save_wait;
        }
    }

    pub fn data_write(&mut self, env: &mut SaveEnv<'_>) {
        let slot = usize::from(env.card.get_data_num());
        env.card.set_checksum_game_data(slot);
        env.card.save();
        self.op_result = env.card.save_sync();
    }

    pub fn mem_card_data_save_wait(&mut self, env: &mut SaveEnv<'_>) {
        let alpha_msg_rt = self.pane_alpha_msg_txt(self.timer, IDX_29);
        self.timer += 1;

        if self.op_result == 0 {
            self.op_result = env.card.save_sync();
        }

        if self.op_result != 0 && alpha_msg_rt {
            self.flag_536 ^= 1;
            self.timer = 0;
            self.flag_534 = IDX_28;

            if self.op_result == 1 {
                self.sound(SE_SAVE_FINISH);
                env.card
                    .set_mem_card_check_id(env.card.get_card_serial_no());
                env.card.set_new_file(0);
                env.card.set_no_file(0);
                self.text_b = format!("\u{1}{MSG_SAVED}");
                self.proc = proc_msg_wait;
                self.flag_52d = proc_save_end_wait;
            } else if self.op_result == 2 {
                self.sound(SE_ALERT_DATA);
                self.text_b = format!("\u{1}{MSG_SAVE_FAILED}");
                self.proc = proc_msg_wait2;
                self.flag_52d = proc_mem_card_check;
            }
        }
    }

    pub fn msg_wait(&mut self) {
        let alpha_msg_rt = self.pane_alpha_msg_txt(self.timer, IDX_29);

        self.timer += 1;
        if alpha_msg_rt {
            self.flag_536 ^= 1;
            self.timer = 0;
            self.proc = self.flag_52d;
        }
    }

    pub fn msg_wait2(&mut self, env: &mut SaveEnv<'_>) {
        if self.flag_534 != 0 {
            self.flag_534 -= 1;
        }

        let alpha_msg_rt = self.pane_alpha_msg_txt(self.timer, IDX_29);

        if alpha_msg_rt {
            if env.pad.any_confirm() || self.flag_534 == 0 {
                self.flag_536 ^= 1;
                self.timer = 0;
                self.proc = self.flag_52d;
            }
        } else {
            self.timer += 1;
        }
    }

    pub fn save_end_wait(&mut self, env: &mut SaveEnv<'_>) {
        if self.flag_534 != 0 {
            self.flag_534 -= 1;
        }

        if env.pad.any_confirm() || self.flag_534 == 0 {
            self.timer = 0;
            self.text_b = format!("\u{1}{MSG_CONTINUE}");
            self.proc = proc_game_continue;
        }
    }

    pub fn can_not_save(&mut self, env: &mut SaveEnv<'_>) {
        let alpha_msg_rt = self.pane_alpha_msg_txt(self.timer, IDX_29);
        if !alpha_msg_rt {
            self.timer += 1;
        }

        if alpha_msg_rt && env.pad.any_confirm() {
            self.flag_536 ^= 1;
            self.timer = 0;

            let msg = if self.use_type == 3 {
                MSG_ENDING_TITLE
            } else {
                MSG_TITLE
            };
            self.text_b = format!("\u{1}{msg}");

            self.proc = proc_can_not_save2;
        }
    }

    pub fn can_not_save2(&mut self) {
        let menu_rt = if self.flag_535 != 0 {
            self.menu_up()
        } else {
            true
        };

        let alpha_msg_rt = self.pane_alpha_msg_txt(self.timer, IDX_29);

        self.timer += 1;
        if menu_rt && alpha_msg_rt {
            self.flag_536 ^= 1;
            self.timer = 0;
            self.flag_535 = 0;
            self.cursor_alpha_init();
            self.flag_52f = 0;
            self.cursor_move();
            self.panes.cursor.show();
            self.proc = proc_save_question;
        }
    }

    pub fn game_continue(&mut self) {
        let alpha_msg_rt = self.pane_alpha_msg_txt(self.timer, IDX_29);

        let menu_rt = if self.flag_535 != 0 {
            self.menu_up()
        } else {
            true
        };

        self.timer += 1;
        if alpha_msg_rt && menu_rt {
            self.flag_536 ^= 1;
            self.timer = 0;
            self.flag_535 = 0;
            self.cursor_alpha_init();
            self.flag_52f = 0;
            self.cursor_move();
            self.panes.cursor.show();
            self.proc = proc_game_continue2;
        }
    }

    pub fn game_continue2(&mut self, env: &mut SaveEnv<'_>) {
        let rt = if self.use_type == 2 || self.use_type == 3 {
            self.yes_no_select2(env, 1)
        } else {
            self.yes_no_select(env, 2)
        };

        if rt {
            if self.flag_52f == 0 {
                match self.use_type {
                    0 | 1 => {}
                    2 => self.end_status = 1,
                    _ => {}
                }

                self.proc = proc_save_wait;
                self.save_status = 3;
            } else if self.use_type == 2 {
                self.flag_534 = CONTINUE_DELAY as u8;
                self.proc = proc_game_continue4;
            } else if self.flag_52f == 2 {
                self.proc = proc_save_wait;
                self.save_status = 3;
            } else {
                self.timer = 0;
                self.flag_534 = QUIT_DELAY as u8;
                self.proc = proc_game_continue3;
            }
        }
    }

    pub fn game_continue3(&mut self, env: &mut SaveEnv<'_>) {
        let alpha_wipe_rt = self.pane_scale_alpha_wipe(self.timer, IDX_23, 1.0, 0, 1);
        let close_rt = self.close_for_game_over_2();

        self.timer += 1;
        if alpha_wipe_rt && close_rt {
            if self.flag_534 != 0 {
                self.flag_534 -= 1;
            } else {
                env.system.on_reset();
            }
        }
    }

    pub fn game_continue4(&mut self) {
        if self.flag_534 != 0 {
            self.flag_534 -= 1;
        } else {
            self.proc = proc_save_wait;
            self.save_status = 3;
            self.end_status = 0;
        }
    }

    pub fn ending_no_save(&mut self) {
        let alpha_msg_rt = self.pane_alpha_msg_txt(self.timer, IDX_29);

        self.timer += 1;
        if alpha_msg_rt {
            self.flag_536 ^= 1;
            self.timer = 0;
            self.flag_535 = 0;
            self.cursor_alpha_init();
            self.flag_52f = 1;
            self.cursor_move();
            self.panes.cursor.show();
            self.proc = proc_ending_no_save2;
        }
    }

    pub fn ending_no_save2(&mut self, env: &mut SaveEnv<'_>) {
        if self.yes_no_select2(env, 1) {
            if self.flag_52f == 0 {
                self.end_status = 0;
                self.proc = proc_save_wait;
                self.save_status = 3;
            } else {
                self.text_b = format!("\u{1}{MSG_ENDING_TITLE}");
                self.proc = proc_ending_no_save3;
            }
        }
    }

    pub fn ending_no_save3(&mut self) {
        let alpha_msg_rt = self.pane_alpha_msg_txt(self.timer, IDX_29);

        self.timer += 1;
        if alpha_msg_rt {
            self.flag_536 ^= 1;
            self.timer = 0;
            self.flag_535 = 0;
            self.cursor_alpha_init();
            self.flag_52f = 0;
            self.cursor_move();
            self.panes.cursor.show();
            self.proc = proc_save_question;
        }
    }

    pub fn ending_data_check(&mut self, env: &mut SaveEnv<'_>) {
        let can_save = if env.card.get_new_file() == 1 {
            true
        } else if env.card.get_no_file() != 0 {
            false
        } else {
            env.card.get_mem_card_check_id() == env.card.get_card_serial_no()
        };

        if can_save {
            self.proc = proc_save_wait;
            self.save_status = 3;
        } else {
            self.sound(SE_ALERT_DATA);
            self.text_b = format!("\u{1}{MSG_BAD_CHECKSUM}");
            self.proc = proc_can_not_save;
        }
    }

    pub fn yes_no_select(&mut self, env: &mut SaveEnv<'_>, param_0: i32) -> bool {
        if !self.panes.cursor.is_visible() {
            return true;
        }

        self.stick
            .check_trigger(env.pad.stick_value, env.pad.stick_angle);

        if env.pad.trig_a {
            self.panes.cursor.hide();
            if self.flag_52f == 0 {
                if param_0 != 0 {
                    self.sound(SE_CONTINUE_GAME);
                } else {
                    self.sound(SE_SAVE_START);
                }
            } else if param_0 != 0 {
                self.sound(SE_QUIT_GAME);
            } else {
                self.sound(SE_SAVE_CANCEL);
            }
            return true;
        } else if env.pad.trig_b {
            if param_0 != 0 {
                self.sound(SE_QUIT_GAME);
            } else {
                self.sound(SE_CONTINUE_GAME);
            }

            self.panes.cursor.hide();
            self.flag_52f = 2;
            return true;
        } else if self.stick.check_right_trigger() {
            if self.flag_52f != 1 {
                self.sound(SE_TALK_CURSOR);
                self.flag_52f = 1;
                self.cursor_move();
            }
        } else if self.stick.check_left_trigger() {
            if self.flag_52f != 0 {
                self.sound(SE_TALK_CURSOR);
                self.flag_52f = 0;
                self.cursor_move();
            }
        }

        self.cursor_anime();
        false
    }

    pub fn yes_no_select2(&mut self, env: &mut SaveEnv<'_>, param_0: i32) -> bool {
        if !self.panes.cursor.is_visible() {
            return true;
        }

        self.stick
            .check_trigger(env.pad.stick_value, env.pad.stick_angle);

        if env.pad.trig_a {
            self.panes.cursor.hide();
            if self.flag_52f == 0 {
                if param_0 != 0 {
                    self.sound(SE_CONTINUE_GAME);
                } else {
                    self.sound(SE_SAVE_START);
                }
            } else if param_0 != 0 {
                self.sound(SE_QUIT_GAME);
            } else {
                self.sound(SE_SAVE_CANCEL);
            }
            return true;
        } else if self.stick.check_right_trigger() {
            if self.flag_52f != 1 {
                self.sound(SE_TALK_CURSOR);
                self.flag_52f = 1;
                self.cursor_move();
            }
        } else if self.stick.check_left_trigger() {
            if self.flag_52f != 0 {
                self.sound(SE_TALK_CURSOR);
                self.flag_52f = 0;
                self.cursor_move();
            }
        }

        self.cursor_anime();
        false
    }

    pub fn cursor_alpha_init(&mut self) {
        self.panes.cursor.now_alpha = 0;
        self.panes.cursor.user_area = 0;
        self.flag_530 = 0;
    }

    pub fn cursor_move(&mut self) {
        let x = f32::from(self.flag_52f) * 139.0;
        self.panes.cursor.pane_trans(x, 0.0);
    }

    pub fn cursor_anime(&mut self) {
        let mut value =
            value_increase(i32::from(IDX_20), i32::from(self.panes.cursor.user_area), 0);
        if self.flag_530 != 0 {
            value = 1.0 - value;
        }

        let span = IDX_21 as f32 - IDX_22 as f32;
        self.panes.cursor.now_alpha = IDX_22 + (span * value) as u8;

        if self.panes.cursor.user_area == 0 {
            self.panes.cursor.user_area = IDX_20 as i16;
            self.flag_530 ^= 1;
        } else {
            self.panes.cursor.user_area -= 1;
        }
    }

    pub fn open_save(&mut self) -> bool {
        let mut second = false;
        let base = self.pane_trance_base(
            self.timer - IDX_09 as i16,
            IDX_08,
            IDX_0c as f32,
            40.0,
            1,
            0,
        );
        if base {
            second = self.pane_trance_base(
                (self.timer - IDX_09 as i16) - IDX_08 as i16,
                IDX_08,
                40.0,
                0.0,
                4,
                2,
            );
        }
        let title =
            self.pane_trance_title(self.timer - IDX_11 as i16, IDX_10, IDX_14 as f32, 0.0, 1, 0);
        let menu = self.menu_up();

        self.timer += 1;
        if base && second && title && menu {
            self.timer = 0;
            return true;
        }
        false
    }

    pub fn close_save(&mut self) -> bool {
        let mut second = false;
        let base = self.pane_trance_base(self.timer - IDX_0a as i16, IDX_08, 0.0, 40.0, 4, 2);
        if base {
            second = self.pane_trance_base(
                (self.timer - IDX_0a as i16) - IDX_08 as i16,
                IDX_08,
                0.0,
                IDX_0c as f32,
                0,
                1,
            );
        }
        let title =
            self.pane_trance_title(self.timer - IDX_12 as i16, IDX_10, 0.0, IDX_14 as f32, 0, 1);
        let menu = if self.flag_535 != 0 {
            true
        } else {
            self.menu_down()
        };

        self.timer += 1;
        if base && second && title && menu {
            self.flag_535 = 0;
            self.timer = 0;
            return true;
        }
        false
    }

    pub fn menu_up(&mut self) -> bool {
        let yes =
            self.pane_trance_menu(self.timer, IDX_18, MenuSlot::Yes, IDX_1e as f32, 0.0, 1, 0);
        let no = self.pane_trance_menu(
            self.timer - IDX_1c as i16,
            IDX_18,
            MenuSlot::No,
            IDX_1e as f32,
            0.0,
            1,
            0,
        );
        yes && no
    }

    pub fn menu_down(&mut self) -> bool {
        let yes =
            self.pane_trance_menu(self.timer, IDX_18, MenuSlot::Yes, 0.0, IDX_1e as f32, 0, 1);
        let no = self.pane_trance_menu(
            self.timer - IDX_1c as i16,
            IDX_18,
            MenuSlot::No,
            0.0,
            IDX_1e as f32,
            0,
            1,
        );
        yes && no
    }

    pub fn pane_alpha_msg_txt(&mut self, param_0: i16, param_1: u8) -> bool {
        if param_0 < 0 {
            return false;
        }
        if param_0 > i16::from(param_1) {
            return true;
        }

        let value = value_increase(i32::from(param_1), i32::from(param_0), 0);
        let fade = 1.0 - value;
        let front = usize::from(self.flag_536) + 3;
        let back = usize::from(self.flag_536 ^ 1) + 3;
        self.panes.base[front].set_now_alpha(fade);
        self.panes.base[back].set_now_alpha(value);
        false
    }

    pub fn pane_trance_base(
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
        self.panes.base[0].pane_trans(0.0, param_2 + value * span);

        if param_5 != 2 {
            if param_5 == 1 {
                value = 1.0 - value;
            }

            for index in 0..3 {
                self.panes.base[index].set_now_alpha(value);
            }
        }

        false
    }

    pub fn pane_scale_alpha_wipe(
        &mut self,
        param_0: i16,
        param_1: u8,
        param_2: f32,
        param_3: u8,
        param_4: i32,
    ) -> bool {
        if param_0 < 0 {
            return false;
        }
        if param_0 > i16::from(param_1) {
            return true;
        }

        let mut value = value_increase(i32::from(param_1), i32::from(param_0), param_3);

        let grow = param_2 - 1.0;
        if grow >= 0.0 {
            let factor = (1.0 - value) * grow;
            for index in 0..WIPE_PANES {
                let size = &mut self.panes.wipe[index].size;
                size.x = self.panes.wipe[index].size_orig.x * (factor + 1.0);
                size.y = self.panes.wipe[index].size_orig.y * (factor + 1.0);
            }

            let origin0 = self.panes.wipe[0].pos_top_left_orig;
            let size0 = self.panes.wipe[0].size;
            let pos0 = (
                origin0.x - (size0.x - self.panes.wipe[0].size_orig.x),
                origin0.y,
            );
            self.panes.wipe[0].move_to(pos0.0, pos0.1);

            let origin2 = self.panes.wipe[2].pos_top_left_orig;
            let size2 = self.panes.wipe[2].size;
            self.panes.wipe[2].move_to(
                origin2.x,
                origin2.y - (size2.y - self.panes.wipe[2].size_orig.y),
            );

            let origin1 = self.panes.wipe[1].pos_top_left_orig;
            let size1 = self.panes.wipe[1].size;
            self.panes.wipe[1].move_to(
                size2.x,
                origin1.y - (size1.y - self.panes.wipe[1].size_orig.y),
            );

            let origin3 = self.panes.wipe[3].pos_top_left_orig;
            self.panes.wipe[3].move_to(size0.x, origin3.y);

            let x4 = size2.x + size1.x;
            let y4 = -size1.y;
            self.panes.wipe[4].move_to(x4, y4);

            let size5 = self.panes.wipe[5].size;
            self.panes.wipe[5].move_to(-size5.x, -size2.y);

            let size6 = self.panes.wipe[6].size;
            let size7 = self.panes.wipe[7].size;
            self.panes.wipe[6].move_to(-(size0.x + size5.x), -(size2.y + size6.y));
            self.panes.wipe[7].move_to(-(size0.x + size5.x), size0.y + size7.y);
        }

        if param_4 != 2 {
            if param_4 == 1 {
                value = 1.0 - value;
            }

            for index in 0..WIPE_PANES {
                self.panes.wipe[index].set_now_alpha(value);
            }
        }

        false
    }

    pub fn pane_alpha_mask(
        &mut self,
        param_0: i16,
        param_1: u8,
        param_2: u8,
        param_3: i32,
    ) -> bool {
        if param_0 < 0 {
            return false;
        }
        if param_0 > i16::from(param_1) {
            return true;
        }

        let mut value = value_increase(i32::from(param_1), i32::from(param_0), param_2);

        if param_3 != 2 {
            if param_3 == 1 {
                value = 1.0 - value;
            }

            self.panes.mask.set_now_alpha(value);
        }

        false
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
        self.panes.title.pane_trans(0.0, param_2 + value * span);

        if param_5 != 2 {
            if param_5 == 1 {
                value = 1.0 - value;
            }

            self.panes.title.set_now_alpha(value);
            self.panes.title_label.set_now_alpha(value);
        }

        false
    }

    pub fn pane_rotate(
        &mut self,
        param_0: i16,
        param_1: u8,
        slot: TitleSlot,
        angle_x: f32,
        angle_y: f32,
        angle_z: f32,
        param_6: u8,
    ) -> bool {
        if param_0 < 0 {
            return false;
        }
        if param_0 > i16::from(param_1) {
            return true;
        }

        let value = value_increase(i32::from(param_1), i32::from(param_0), param_6);
        match slot {
            TitleSlot::Title => self.panes.title.rotate(angle_x, angle_y, angle_z * value),
            TitleSlot::Base => self.panes.base[0].rotate(angle_x, angle_y, angle_z * value),
            TitleSlot::Label => self
                .panes
                .title_label
                .rotate(angle_x, angle_y, angle_z * value),
            TitleSlot::Yes => self.panes.yes[0].rotate(angle_x, angle_y, angle_z * value),
            TitleSlot::No => self.panes.no[0].rotate(angle_x, angle_y, angle_z * value),
        }
        false
    }

    pub fn pane_trance_menu(
        &mut self,
        param_0: i16,
        param_1: u8,
        slot: MenuSlot,
        param_3: f32,
        param_4: f32,
        param_5: u8,
        param_6: i32,
    ) -> bool {
        if param_0 < 0 {
            return false;
        }
        if param_0 > i16::from(param_1) {
            return true;
        }

        let mut value = value_increase(i32::from(param_1), i32::from(param_0), param_5);
        let span = param_4 - param_3;
        let group: &mut [Pane; MENU_PANES] = match slot {
            MenuSlot::Yes => &mut self.panes.yes,
            MenuSlot::No => &mut self.panes.no,
        };
        group[0].pane_trans(0.0, param_3 + value * span);

        if param_6 != 2 {
            if param_6 == 1 {
                value = 1.0 - value;
            }

            for pane in group.iter_mut() {
                pane.set_now_alpha(value);
            }
        }

        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleSlot {
    Title,
    Base,
    Label,
    Yes,
    No,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuSlot {
    Yes,
    No,
}

pub struct SaveEnv<'a> {
    pub pad: PadInput,
    pub card: &'a mut MemCard,
    pub menu: &'a mut MenuFlags,
    pub system: &'a mut SystemRequest,
    pub particles: &'a mut Particles,
    pub pending_save: Option<SaveGame>,
    pub vibration: &'a mut Vibration,
    pub health: &'a mut Health,
    pub life_on_open: u16,
}

pub fn default_name() -> &'static str {
    DEFAULT_NAME
}

pub fn save_title(use_type: u8) -> &'static str {
    if use_type == 3 { "ending" } else { "save" }
}

pub fn status_name(save_status: u8) -> &'static str {
    match save_status {
        0 => "closed",
        1 => "idle",
        2 => "open",
        3 => "finished",
        _ => "unknown",
    }
}
