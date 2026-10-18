use bevy_ecs::prelude::*;

use crate::j2d::{self, Kind, Screen};
use crate::sim::flags::Collect;
use crate::text::msgproc::{MsgAppend, MsgProcs, fop_msgm_value_increase};

pub const SCREEN_MAIN_PARTS1: &str = "main_parts1.blo";
pub const SCREEN_MAIN_PARTS3: &str = "main_parts3.blo";
pub const SCREEN_TIME_SWIM1: &str = "time_swim_1.blo";
pub const SCREEN_TIME_SWIM2: &str = "time_swim2.blo";
pub const SCREEN_CHOICE_ROAD: &str = "choice_road.blo";

pub const STTS_1: u32 = 0x0000_0001;
pub const STTS_2: u32 = 0x0000_0002;
pub const STTS_4: u32 = 0x0000_0004;
pub const STTS_8: u32 = 0x0000_0008;
pub const STTS_10: u32 = 0x0000_0010;
pub const STTS_20: u32 = 0x0000_0020;
pub const STTS_40: u32 = 0x0000_0040;
pub const STTS_80: u32 = 0x0000_0080;
pub const STTS_100: u32 = 0x0000_0100;
pub const STTS_200: u32 = 0x0000_0200;
pub const STTS_400: u32 = 0x0000_0400;
pub const STTS_800: u32 = 0x0000_0800;
pub const STTS_1000: u32 = 0x0000_1000;
pub const STTS_2000: u32 = 0x0000_2000;
pub const STTS_4000: u32 = 0x0000_4000;
pub const STTS_8000: u32 = 0x0000_8000;
pub const STTS_10000: u32 = 0x0001_0000;
pub const STTS_20000: u32 = 0x0002_0000;
pub const STTS_40000: u32 = 0x0004_0000;
pub const STTS_80000: u32 = 0x0008_0000;
pub const STTS_100000: u32 = 0x0010_0000;
pub const STTS_200000: u32 = 0x0020_0000;
pub const STTS_400000: u32 = 0x0040_0000;
pub const STTS_800000: u32 = 0x0080_0000;

pub const PLAYER_STTS1_DEKU_LEAF_FLY: u32 = 0x0000_0020;
pub const PLAYER_STTS1_UNK2000: u32 = 0x0000_2000;
pub const PLAYER_STTS0_BOW_AIM: u32 = 0x0000_1000;

pub const ACT_BLANK: u8 = 0x00;
pub const ACT_LOOK: u8 = 0x01;
pub const ACT_SPEAK: u8 = 0x02;
pub const ACT_CHARTS: u8 = 0x03;
pub const ACT_LIFT: u8 = 0x04;
pub const ACT_CLIMB: u8 = 0x05;
pub const ACT_LET_GO: u8 = 0x06;
pub const ACT_RETURN: u8 = 0x07;
pub const ACT_PUT_AWAY: u8 = 0x08;
pub const ACT_DROP: u8 = 0x09;
pub const ACT_CHECK: u8 = 0x0A;
pub const ACT_OPEN: u8 = 0x0B;
pub const ACT_ATTACK: u8 = 0x0C;
pub const ACT_OSU: u8 = 0x0D;
pub const ACT_THROW: u8 = 0x0E;
pub const ACT_CROUCH: u8 = 0x0F;
pub const ACT_SIDLE: u8 = 0x10;
pub const ACT_GRAB: u8 = 0x11;
pub const ACT_JUMP: u8 = 0x12;
pub const ACT_STOP: u8 = 0x13;
pub const ACT_HARU: u8 = 0x14;
pub const ACT_TATAMU: u8 = 0x15;
pub const ACT_LET_GO_ROPE: u8 = 0x16;
pub const ACT_CHOOSE: u8 = 0x17;
pub const ACT_KOGU: u8 = 0x18;
pub const ACT_NEXT: u8 = 0x19;
pub const ACT_PARRY: u8 = 0x1A;
pub const ACT_PICK_UP: u8 = 0x1B;
pub const ACT_GET_IN_SHIP: u8 = 0x1C;
pub const ACT_GET_OUT_SHIP: u8 = 0x1D;
pub const ACT_SAVE: u8 = 0x1E;
pub const ACT_OPTION: u8 = 0x1F;
pub const ACT_TAKE_PHOTO: u8 = 0x20;
pub const ACT_INFO: u8 = 0x21;
pub const ACT_SWAP_MODES: u8 = 0x22;
pub const ACT_FLY: u8 = 0x23;
pub const ACT_CALL: u8 = 0x24;
pub const ACT_BID: u8 = 0x25;
pub const ACT_READ: u8 = 0x26;
pub const ACT_CANCEL: u8 = 0x27;
pub const ACT_KAKUDAI: u8 = 0x28;
pub const ACT_HIKAKU: u8 = 0x29;
pub const ACT_ZOOM01: u8 = 0x2A;
pub const ACT_ZOOM02: u8 = 0x2B;
pub const ACT_CRUISE: u8 = 0x2C;
pub const ACT_NAGERU_DUPE_2D: u8 = 0x2D;
pub const ACT_MOTU_DUPE_2E: u8 = 0x2E;
pub const ACT_SWING: u8 = 0x2F;
pub const ACT_SEA_CHART: u8 = 0x30;
pub const ACT_SAKE_DUPE_31: u8 = 0x31;
pub const ACT_MODORU_DUPE_32: u8 = 0x32;
pub const ACT_SAVE_DUPE_33: u8 = 0x33;
pub const ACT_SAVE_DUPE_34: u8 = 0x34;
pub const ACT_SWORD_01: u8 = 0x35;
pub const ACT_DEFEND: u8 = 0x36;
pub const ACT_CLUB_37: u8 = 0x37;
pub const ACT_HATCHET: u8 = 0x38;
pub const ACT_IRON_CLUB: u8 = 0x39;
pub const ACT_LONGSWORD: u8 = 0x3A;
pub const ACT_SPEAR: u8 = 0x3B;
pub const ACT_SPEAR_DUPE_3C: u8 = 0x3C;
pub const ACT_BOW_01: u8 = 0x3D;
pub const ACT_HIDDEN: u8 = 0x3E;

pub const MENU_STATUS_NONE: u8 = 0;
pub const MENU_STATUS_ITEM: u8 = 1;
pub const MENU_STATUS_COLLECT: u8 = 2;
pub const MENU_STATUS_MAP: u8 = 3;
pub const MENU_STATUS_NAME: u8 = 4;
pub const MENU_STATUS_SAVE: u8 = 5;

pub const GROUP_ENEMY: u8 = 0x2;

pub const SAVE_TABLE_TOTG: i32 = 0x05;

pub const PROCESS_ERROR_ID: u32 = 0xFFFF_FFFF;
pub const NAME_OPERATE_WIND: u16 = 0x01F2;

pub const SE_MP_GAUGE_INC: u32 = 0x087D;
pub const SE_MP_GAUGE_DEC: u32 = 0x087E;
pub const SE_MP_GAUGE_DEC_FIN: u32 = 0x0881;

pub const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
pub const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
pub const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
pub const RND_SEED: i32 = 100;

pub const ICON_MOVE_HIDE_Y: f32 = -200.0;
pub const ICON_MOVE_SLIDE_X: i16 = 0x21;
pub const ICON_MOVE_SLIDE_Y: i16 = -5;
pub const ICON_ALPHA_MAX_FRAME: i16 = 5;
pub const ICON_ALPHA_DIM_FRAME: i16 = 3;
pub const ICON_ALPHA_FADE_STEPS: i16 = 5;
pub const ICON_MOVE_ENTER_FRAMES: i16 = 5;
pub const ICON_MOVE_FLAG_BIT: u8 = 8;
pub const ICON_MOVE_FLAG_SETTLED: u8 = 0xF;
pub const ICON_ACTION_MODE_BIT: u8 = 0x10;

pub const ENEMY_ZAKO_FADE_FRAMES: i16 = 10;
pub const ENEMY_BOSS_FADE_FRAMES: i16 = 10;
pub const ENEMY_BOSS_OPEN_FRAMES: i16 = 0x1E;
pub const ENEMY_BOSS_RISE_FRAMES: i16 = 0xA;
pub const ENEMY_BOSS_SLIDE_FRAMES: i16 = 0x14;
pub const ENEMY_BOSS_SLIDE_DIST: f32 = 80.0;
pub const ENEMY_BOSS_BOB_STEP: f32 = 2.0;
pub const ENEMY_BOSS_BOB_BASE: i16 = 4;
pub const ENEMY_EYE_PERIOD_BASE: f32 = 40.0;
pub const ENEMY_EYE_PERIOD_SPAN: f32 = 18.0;
pub const ENEMY_EYE_PERIOD_SHIFT: i16 = 1;
pub const ENEMY_NICK_PERIOD_BASE: f32 = 300.0;
pub const ENEMY_NICK_PERIOD_SPAN: f32 = 200.0;
pub const ENEMY_SCREEN_EDGE_X: f32 = 320.0;
pub const ENEMY_SCREEN_MARGIN: f32 = 50.0;

pub const MAGIC_BARS: usize = 8;
pub const MAGIC_WIDTH_UNITS: f32 = 25.0;
pub const MAGIC_PIXEL_SCALE: f32 = 100.0;
pub const MAGIC_MAX_UNITS: f32 = 32.0;
pub const MAGIC_MAX_CLAMP: i32 = 0x20;
pub const MAGIC_MAX_SHORT: i32 = 0x10;
pub const MAGIC_DRAIN_STEP: i32 = 0x271;
pub const MAGIC_DRAIN_SOUND_STEP: i32 = 1250;
pub const MAGIC_FLY_UNITS: f32 = 25.0;
pub const MAGIC_FLY_SPAN: f32 = 8.0;
pub const MAGIC_ROW_SHIFT_Y: f32 = -22.0;
pub const MAGIC_ROW_MAX_LIFE: i32 = 0x2B;
pub const MAGIC_FRAME_SPAN: f32 = 8.0;
pub const MAGIC_BLINK_PERIOD: i16 = 0x1D;
pub const MAGIC_BLINK_RAMP: i16 = 0xF;
pub const MAGIC_BLINK_SPAN: i16 = 0x1E;
pub const MAGIC_ALPHA_FULL_FRAMES: i16 = 5;
pub const MAGIC_ALPHA_DIM_FRAMES: i16 = 3;
pub const MAGIC_FLY_FADE_FRAMES: i16 = 5;
pub const MAGIC_FLY_TRAIL_FRAMES: i16 = 0x32;
pub const MAGIC_FLY_TRAIL_SPLIT: i16 = 0x1E;
pub const MAGIC_FLY_TRAIL_RAMP: i16 = 0x14;
pub const MAGIC_FLY_TRAIL_SCALE: f32 = 0.5;
pub const MAGIC_QUAD_SLOPE_X: f32 = 1.6875;
pub const MAGIC_QUAD_SLOPE_Y: f32 = 0.8125;
pub const MAGIC_LIVE_FLAG: u8 = 0x01;
pub const MAGIC_READY_FLAG: u8 = 0x02;

pub const FLY_COLOR_A: [u8; 4] = [0x64, 0x64, 0xFF, 0xFF];
pub const FLY_COLOR_B: [u8; 4] = [0x64, 0x64, 0xFF, 0x00];
pub const FLY_COLOR_C: [u8; 4] = [0x96, 0x96, 0xE6, 0xFF];
pub const FLY_COLOR_D: [u8; 4] = [0x96, 0x96, 0xE6, 0x00];
pub const FLY_COLOR_E: [u8; 4] = [0xB4, 0xB4, 0xFF, 0xFF];
pub const FLY_COLOR_F: [u8; 4] = [0xB4, 0xB4, 0xFF, 0x00];

pub const METRONOME_OPEN: u8 = 0;
pub const METRONOME_CLOSING: u8 = 1;
pub const METRONOME_CLOSED: u8 = 2;

pub const WIND_OPERATE_ACTIVE: u8 = 2;

pub const MENU_LR_DEFAULT_SPAN: i16 = 0x14;
pub const MENU_LR_STEP_X: f32 = 20.0;
pub const MENU_LR_INIT_R: i16 = 10;
pub const MENU_LR_L_HOLD: i16 = 1;
pub const MENU_PLUS_FADE_FRAMES: i16 = 5;
pub const MENU_PLUS_SHIFT_X: f32 = 0x52 as f32;
pub const MENU_PLUS_COMPASS_OFF_X: f32 = 7.0;
pub const MENU_PLUS_COMPASS_OFF_Y: f32 = 9.0;
pub const MENU_MAP_OFF_X: f32 = 0x19 as f32;
pub const MENU_MAP_OFF_Y: f32 = 0x152 as f32;
pub const MENU_DEG_PER_TURN: f32 = 180.0 / 0x8000 as f32;
pub const MENU_DEG_BIAS: f32 = -180.0;

pub const MINIGAME_1: u8 = 1;
pub const MINIGAME_6: u8 = 6;
pub const COLLECT_MODE_COMPASS: u8 = 4;
pub const COLLECT_MODE_BUTTON: u8 = 5;

pub const ICON_ARCHIVE_ACTION: usize = 0;
pub const ICON_ARCHIVE_MENU: usize = 1;
pub const ICON_TEX_SLOT: usize = 2;
pub const ICON_TEX_BYTES: usize = 0xC00;

pub const ACTION_TEX_TABLE: [(u8, Option<&'static str>); 63] = [
    (ACT_BLANK, Some("ba_shiraberu.bti")),
    (ACT_LOOK, Some("ba_miru.bti")),
    (ACT_SPEAK, Some("ba_syaberu.bti")),
    (ACT_CHARTS, Some("ba_shiraberu.bti")),
    (ACT_LIFT, Some("ba_motu.bti")),
    (ACT_CLIMB, Some("ba_noru.bti")),
    (ACT_LET_GO, Some("ba_oriru.bti")),
    (ACT_RETURN, Some("ba_modoru.bti")),
    (ACT_PUT_AWAY, Some("ba_shimau.bti")),
    (ACT_DROP, Some("ba_oku.bti")),
    (ACT_CHECK, Some("ba_chekku.bti")),
    (ACT_OPEN, Some("ba_hiraku.bti")),
    (ACT_ATTACK, Some("ba_attack.bti")),
    (ACT_OSU, Some("ba_osu.bti")),
    (ACT_THROW, Some("ba_nageru.bti")),
    (ACT_CROUCH, Some("ba_shagamu.bti")),
    (ACT_SIDLE, Some("ba_harituku.bti")),
    (ACT_GRAB, Some("ba_tukamu.bti")),
    (ACT_JUMP, Some("ba_jump.bti")),
    (ACT_STOP, Some("ba_tomeru.bti")),
    (ACT_HARU, Some("ba_haru.bti")),
    (ACT_TATAMU, Some("ba_tatamu.bti")),
    (ACT_LET_GO_ROPE, Some("ba_hanasu.bti")),
    (ACT_CHOOSE, Some("ba_kettei.bti")),
    (ACT_KOGU, Some("ba_kogu.bti")),
    (ACT_NEXT, Some("ba_tugihe.bti")),
    (ACT_PARRY, Some("ba_sake.bti")),
    (ACT_PICK_UP, Some("ba_motu_buki.bti")),
    (ACT_GET_IN_SHIP, Some("ba_noru_hune.bti")),
    (ACT_GET_OUT_SHIP, Some("ba_oriru_hune.bti")),
    (ACT_SAVE, Some("ba_save.bti")),
    (ACT_OPTION, Some("ba_option.bti")),
    (ACT_TAKE_PHOTO, Some("ba_toru.bti")),
    (ACT_INFO, Some("ba_setumei.bti")),
    (ACT_SWAP_MODES, Some("ba_kirikae.bti")),
    (ACT_FLY, Some("ba_tobu.bti")),
    (ACT_CALL, Some("ba_yobu.bti")),
    (ACT_BID, Some("ba_hoshii.bti")),
    (ACT_READ, Some("ba_yomu.bti")),
    (ACT_CANCEL, Some("ba_yameru.bti")),
    (ACT_KAKUDAI, Some("ba_kakudai.bti")),
    (ACT_HIKAKU, Some("ba_hikaku.bti")),
    (ACT_ZOOM01, Some("ba_zoom01.bti")),
    (ACT_ZOOM02, Some("ba_zoom02.bti")),
    (ACT_CRUISE, Some("ba_susumu.bti")),
    (ACT_NAGERU_DUPE_2D, Some("ba_nageru.bti")),
    (ACT_MOTU_DUPE_2E, Some("ba_motu.bti")),
    (ACT_SWING, Some("ba_huru.bti")),
    (ACT_SEA_CHART, Some("ba_kaizuhe.bti")),
    (ACT_SAKE_DUPE_31, Some("ba_sake.bti")),
    (ACT_MODORU_DUPE_32, Some("ba_modoru.bti")),
    (ACT_SAVE_DUPE_33, Some("ba_save.bti")),
    (ACT_SAVE_DUPE_34, Some("ba_save.bti")),
    (ACT_SWORD_01, Some("sword_01.bti")),
    (ACT_DEFEND, Some("ba_husegu.bti")),
    (ACT_CLUB_37, Some("boko_stick.bti")),
    (ACT_HATCHET, Some("hatchet.bti")),
    (ACT_IRON_CLUB, Some("iron_club.bti")),
    (ACT_LONGSWORD, Some("longsword.bti")),
    (ACT_SPEAR, Some("spear.bti")),
    (ACT_SPEAR_DUPE_3C, Some("spear.bti")),
    (ACT_BOW_01, Some("bow_01.bti")),
    (ACT_HIDDEN, Some("ba_modoru.bti")),
];

pub fn action_texture_name(status: u8) -> &'static str {
    let mut index = 0usize;
    for (at, entry) in ACTION_TEX_TABLE.iter().enumerate() {
        if entry.0 == status {
            index = at;
            break;
        }
    }
    ACTION_TEX_TABLE[index]
        .1
        .or(ACTION_TEX_TABLE[0].1)
        .unwrap_or("")
}

pub fn icon_archive(status: u8) -> usize {
    if status == ACT_RETURN || status == ACT_RETURN {
        ICON_ARCHIVE_MENU
    } else {
        ICON_ARCHIVE_ACTION
    }
}

pub fn is_bit8(flag: u8, bit: u8) -> bool {
    flag & bit != 0
}

pub fn on_bit8(flag: &mut u8, bit: u8) {
    *flag |= bit;
}

pub fn off_bit8(flag: &mut u8, bit: u8) {
    *flag &= !bit;
}

pub fn linear_ramp(i_max: i32, i_value: i32) -> f32 {
    if i_max <= 0 {
        return 1.0;
    }
    let value = if i_value < 0 { 0 } else { i_value.min(i_max) };
    value as f32 / i_max as f32
}

pub fn ramp(i_max: i32, i_value: i32, i_mode: u8) -> f32 {
    fop_msgm_value_increase(i_max, i_value, i_mode).unwrap_or_else(|| linear_ramp(i_max, i_value))
}

pub fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = (seed[0] as f32 / RND_SCALE[0])
        + (seed[1] as f32 / RND_SCALE[1])
        + (seed[2] as f32 / RND_SCALE[2]);
    (sum % 1.0).abs()
}

pub fn rnd_f(seed: &mut [i32; 3], max: f32) -> f32 {
    rnd(seed) * max
}

pub fn atan2s(y: f32, x: f32) -> i16 {
    (y.atan2(x) * (32768.0 / std::f32::consts::PI)) as i16
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Picture {
    pub m124: f32,
    pub m128: f32,
    pub m12c: f32,
    pub m130: f32,
    pub m134: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GaugePane {
    pub tag: String,
    pub kind: Kind,
    pub found: bool,
    pub visible: bool,
    pub pos_top_left_orig: [f32; 2],
    pub pos_top_left: [f32; 2],
    pub pos_center_orig: [f32; 2],
    pub pos_center: [f32; 2],
    pub size_orig: [f32; 2],
    pub size: [f32; 2],
    pub rect: [f32; 4],
    pub init_alpha: u8,
    pub now_alpha: u8,
    pub alpha: u8,
    pub user_area: i16,
    pub texture: Option<String>,
    pub white: [u8; 4],
    pub black: [u8; 4],
    pub rotate_x: f32,
    pub rotate_y: f32,
    pub rotation: Option<f32>,
    pub picture: Picture,
}

impl Default for GaugePane {
    fn default() -> Self {
        Self {
            tag: String::new(),
            kind: Kind::Pane,
            found: false,
            visible: true,
            pos_top_left_orig: [0.0; 2],
            pos_top_left: [0.0; 2],
            pos_center_orig: [0.0; 2],
            pos_center: [0.0; 2],
            size_orig: [0.0; 2],
            size: [0.0; 2],
            rect: [0.0; 4],
            init_alpha: 0xFF,
            now_alpha: 0,
            alpha: 0xFF,
            user_area: 0,
            texture: None,
            white: [0xFF; 4],
            black: [0xFF; 4],
            rotate_x: 0.0,
            rotate_y: 0.0,
            rotation: None,
            picture: Picture::default(),
        }
    }
}

impl GaugePane {
    pub fn new(tag: &str) -> Self {
        Self {
            tag: tag.to_string(),
            ..Self::default()
        }
    }

    pub fn bind(&mut self, screen: &Screen) {
        if let Some(layout) = screen.find(&self.tag) {
            self.parts_set(layout);
        }
    }

    pub fn parts_set(&mut self, layout: &j2d::ScreenPane) {
        self.kind = layout.kind;
        self.found = true;
        self.visible = layout.visible;
        self.pos_top_left_orig = [layout.bounds.x, layout.bounds.y];
        self.size_orig = [layout.bounds.w, layout.bounds.h];
        self.pos_center_orig = [
            layout.bounds.x + layout.bounds.w / 2.0,
            layout.bounds.y + layout.bounds.h / 2.0,
        ];
        self.init_alpha = layout.alpha;
        self.alpha = layout.alpha;
        self.pos_top_left = self.pos_top_left_orig;
        self.pos_center = self.pos_center_orig;
        self.size = self.size_orig;
        self.rect = [
            layout.bounds.x,
            layout.bounds.y,
            layout.bounds.right(),
            layout.bounds.bottom(),
        ];
        self.now_alpha = 0;
        self.user_area = 0;
        self.texture = layout.texture.clone();
    }

    pub fn width(&self) -> f32 {
        self.rect[2] - self.rect[0]
    }

    pub fn height(&self) -> f32 {
        self.rect[3] - self.rect[1]
    }

    pub fn apply_pos_size(&mut self) {
        self.rect = [
            self.pos_top_left[0],
            self.pos_top_left[1],
            self.pos_top_left[0] + self.size[0],
            self.pos_top_left[1] + self.size[1],
        ];
    }

    pub fn move_rect(&mut self, x: f32, y: f32) {
        self.rect[0] = x;
        self.rect[1] = y;
    }

    pub fn resize_picture(&mut self, w: f32, h: f32) {
        self.rect[2] = self.rect[0] + w;
        self.rect[3] = self.rect[1] + h;
    }

    pub fn cpos_move(&mut self) {
        self.pos_top_left = [
            self.pos_center[0] - self.size[0] / 2.0,
            self.pos_center[1] - self.size[1] / 2.0,
        ];
        self.apply_pos_size();
    }

    pub fn pane_trans(&mut self, trans_x: f32, trans_y: f32) {
        self.pos_center = [
            self.pos_center_orig[0] + trans_x,
            self.pos_center_orig[1] + trans_y,
        ];
        self.cpos_move();
    }

    pub fn pane_scale_x(&mut self, scale: f32) {
        self.size[0] = self.size_orig[0] * scale;
        self.cpos_move();
    }

    pub fn pane_scale_y(&mut self, scale: f32) {
        self.size[1] = self.size_orig[1] * scale;
        self.cpos_move();
    }

    pub fn pane_scale_xy(&mut self, scale: f32) {
        self.size = [self.size_orig[0] * scale, self.size_orig[1] * scale];
        self.cpos_move();
    }

    pub fn set_init_alpha(&mut self) {
        self.now_alpha = self.init_alpha;
    }

    pub fn set_now_alpha(&mut self, value: f32) {
        self.now_alpha = (f32::from(self.init_alpha) * value) as u8;
    }

    pub fn set_now_alpha_zero(&mut self) {
        self.now_alpha = 0;
    }

    pub fn sync_alpha(&mut self) {
        self.alpha = self.now_alpha;
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn change_texture(&mut self, name: &str) {
        self.texture = Some(name.to_string());
    }

    pub fn set_white(&mut self, color: [u8; 4]) {
        self.white = color;
    }

    pub fn set_black(&mut self, color: [u8; 4]) {
        self.black = color;
    }

    pub fn rotate(&mut self, x: f32, y: f32, angle: f32) {
        self.rotate_x = x;
        self.rotate_y = y;
        self.rotation = Some(angle);
    }

    pub fn quad(&self) -> GaugeQuad {
        GaugeQuad {
            tag: self.tag.clone(),
            kind: self.kind,
            texture: self.texture.clone(),
            x: self.rect[0],
            y: self.rect[1],
            w: self.rect[2] - self.rect[0],
            h: self.rect[3] - self.rect[1],
            alpha: self.alpha,
            white: self.white,
            black: self.black,
            rotate_x: self.rotate_x,
            rotate_y: self.rotate_y,
            rotation: self.rotation,
            picture: self.picture,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GaugeQuad {
    pub tag: String,
    pub kind: Kind,
    pub texture: Option<String>,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub alpha: u8,
    pub white: [u8; 4],
    pub black: [u8; 4],
    pub rotate_x: f32,
    pub rotate_y: f32,
    pub rotation: Option<f32>,
    pub picture: Picture,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct MeterTuning {
    pub boss_gauge_fill: f32,
    pub wind_arrow_x: i16,
    pub wind_arrow_y: i16,
    pub menu_gauge_x: i16,
    pub menu_gauge_y: i16,
    pub icon_slide_x: i16,
    pub icon_slide_y: i16,
    pub icon_choose_x: i16,
    pub icon_choose_y: i16,
    pub icon_default_x: i16,
    pub icon_default_y: i16,
    pub icon_hold_x: i16,
    pub icon_hold_y: i16,
    pub menu_lr_half_width: i16,
}

impl Default for MeterTuning {
    fn default() -> Self {
        Self {
            boss_gauge_fill: 1.0,
            wind_arrow_x: 0,
            wind_arrow_y: 0,
            menu_gauge_x: 0,
            menu_gauge_y: 0,
            icon_slide_x: 0xfffa_u16 as i16,
            icon_slide_y: 0,
            icon_choose_x: 0xfffa_u16 as i16,
            icon_choose_y: 0,
            icon_default_x: 0,
            icon_default_y: 0,
            icon_hold_x: 0xff9e_u16 as i16,
            icon_hold_y: 7,
            menu_lr_half_width: 7,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnemyTarget {
    pub name: &'static str,
    pub group: u8,
    pub param: u32,
    pub health: f32,
    pub max_health: f32,
    pub screen: [f32; 2],
}

impl Default for EnemyTarget {
    fn default() -> Self {
        Self {
            name: "",
            group: GROUP_ENEMY,
            param: 0,
            health: 0.0,
            max_health: 0.0,
            screen: [0.0; 2],
        }
    }
}

pub const BOSS_SEARCH_ORDER: [&str; 6] = ["Btd", "Bmd", "Bdk", "Bst", "Bwd", "Bpw"];
pub const ZAKO_EXCLUDE_NAMES: [&str; 9] = [
    "Btd", "Bmd", "Bdk", "Bpw", "Bgn", "Bgn2", "Bgn3", "Gnd", "Bwd",
];
pub const ZAKO_PARAM_NAME: &str = "Bst";
pub const ZAKO_PARAM_MASK: u32 = 0xF;

pub const BOSS_DEMO_STAGES: [&str; 4] = ["Xboss0", "Xboss1", "Xboss2", "Xboss3"];
pub const MENU_BLINK_STAGE: &str = "Ojhous";
pub const MENU_MAJU_STAGE: &str = "MajyuE";
pub const MENU_HIDE_STAGES: [&str; 3] = ["Hyrule", "Hyroom", "kenroom"];

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MapDisplay {
    pub left_up_x: i16,
    pub left_up_y: i16,
    pub size_x: i16,
    pub size_y: i16,
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FlyGauge {
    pub flying: u8,
    pub max_timer: i16,
    pub timer: i16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Baton {
    pub alive: bool,
    pub open_return: bool,
    pub close_return: bool,
    pub frame: i32,
}

impl Baton {
    pub fn create() -> Self {
        Self {
            alive: true,
            open_return: true,
            close_return: false,
            frame: 0,
        }
    }

    pub fn open(&self) -> bool {
        self.open_return
    }

    pub fn close(&self) -> bool {
        self.close_return
    }

    pub fn step(&mut self) {
        self.frame = self.frame.wrapping_add(1);
    }

    pub fn delete(&mut self) {
        self.alive = false;
        self.open_return = false;
        self.close_return = false;
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct GaugeEnv {
    pub status_flags: u32,
    pub r_status: u8,
    pub r_status_force: u8,
    pub button_icon_mode: u8,
    pub menu_open: bool,
    pub menu_status: u8,
    pub push_menu_button: u8,
    pub collect_mode: u8,
    pub auction_flag: bool,
    pub event_hind: u16,
    pub event_running: bool,
    pub overlap_request: bool,
    pub demo_start: bool,
    pub player_status1: u32,
    pub up_button: u8,
    pub picture_status: u8,
    pub stage_boss: bool,
    pub stage_boss_demo: bool,
    pub stage_boss_enemy: bool,
    pub collect_boss_gauge: bool,
    pub attention_present: bool,
    pub lock_truth: u8,
    pub lock_target: Option<EnemyTarget>,
    pub targets: Vec<EnemyTarget>,
    pub start_stage: String,
    pub next_stage: String,
    pub event_0801: bool,
    pub event_0908: bool,
    pub mini_game_type: u8,
    pub pane_mode: u8,
    pub map: MapDisplay,
    pub operate_wind: u8,
    pub operate_wind_cancel: bool,
    pub metronome: bool,
    pub wind_arrow_active: bool,
    pub wind_vec: [f32; 2],
    pub map_alpha: f32,
    pub compass_slot: u8,
    pub player_eye_screen: [f32; 2],
    pub fly_makar: FlyGauge,
    pub fly_medli: FlyGauge,
    pub player_is_makar: bool,
    pub player_is_medli: bool,
    pub max_life: i32,
    pub heart_alpha: f32,
    pub dragon_shield: bool,
    pub cut_roll: bool,
    pub leaf_fly: bool,
    pub bow_aim: bool,
    pub arrow_keep_type: u8,
    pub item_id: i32,
    pub save_table: i32,
}

impl GaugeEnv {
    pub fn find_target(&self, name: &str) -> Option<&EnemyTarget> {
        self.targets.iter().find(|target| target.name == name)
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct GaugeOutput {
    pub sounds: Vec<u32>,
    pub spawn_process: Option<u16>,
    pub operate_wind_cancel: bool,
    pub map_alpha: f32,
    pub button_action_mode: u8,
}

impl GaugeOutput {
    pub fn reset(&mut self) {
        self.sounds.clear();
        self.spawn_process = None;
        self.operate_wind_cancel = false;
        self.map_alpha = 0.0;
        self.button_action_mode = 0;
    }
}

#[derive(Clone, Debug, Default)]
pub struct IconGauge {
    pub base: GaugePane,
    pub tag_pane: GaugePane,
    pub note: GaugePane,
    pub status_cache: u8,
    pub alpha_frame: i16,
    pub move_frame: i16,
    pub move_flag: i16,
    pub move_status: i16,
    pub now_x: f32,
    pub now_y: f32,
    pub flags: u8,
    pub action_mode: u8,
    pub texture_slot: usize,
    pub texture_archive: usize,
    pub texture_bytes: usize,
}

#[derive(Clone, Debug, Default)]
pub struct EnemyGauge {
    pub bar: [GaugePane; 4],
    pub eye_a: GaugePane,
    pub eye_b: GaugePane,
    pub nick: GaugePane,
    pub nick2: GaugePane,
    pub zako_edge: GaugePane,
    pub zako_fill: GaugePane,
    pub zako_mid: GaugePane,
    pub zako_back: GaugePane,
    pub zako_line: GaugePane,
    pub eye_flag: bool,
}

#[derive(Clone, Debug)]
pub struct MagicGauge {
    pub bars: [GaugePane; MAGIC_BARS],
    pub frame: GaugePane,
    pub shadow: GaugePane,
    pub key_back: GaugePane,
    pub front: GaugePane,
    pub key_front: GaugePane,
    pub magic: i32,
    pub max_magic: i32,
    pub item_magic_count: i32,
    pub item_max_magic_count: i32,
    pub ready_flag: u8,
    pub live_flag: u8,
    pub gauge_pixels: u16,
    pub gauge_max: u8,
    pub mode_flag: u8,
    pub now_magic: f32,
    pub alpha_value: f32,
    pub base_front_white: [u8; 4],
    pub base_front_black: [u8; 4],
    pub base_bar0_white: [u8; 4],
    pub base_bar0_black: [u8; 4],
    pub base_bar1_white: [u8; 4],
    pub base_bar1_black: [u8; 4],
    pub sound_flag: bool,
}

impl Default for MagicGauge {
    fn default() -> Self {
        Self {
            bars: std::array::from_fn(|i| GaugePane::new(MAGIC_BAR_TAGS[i])),
            frame: GaugePane::new("mbb3"),
            shadow: GaugePane::new("mbb2"),
            key_back: GaugePane::new("mbk2"),
            front: GaugePane::new("mbb1"),
            key_front: GaugePane::new("mbk1"),
            magic: 0,
            max_magic: 0,
            item_magic_count: 0,
            item_max_magic_count: 0,
            ready_flag: 0,
            live_flag: 0,
            gauge_pixels: 0,
            gauge_max: 0,
            mode_flag: 0,
            now_magic: 0.0,
            alpha_value: 0.0,
            base_front_white: [0xFF; 4],
            base_front_black: [0xFF; 4],
            base_bar0_white: [0xFF; 4],
            base_bar0_black: [0xFF; 4],
            base_bar1_white: [0xFF; 4],
            base_bar1_black: [0xFF; 4],
            sound_flag: false,
        }
    }
}

pub const MAGIC_BAR_TAGS: [&str; MAGIC_BARS] = [
    "mbr1", "mbr2", "mbr3", "mbr4", "mbr5", "mbr6", "mbr7", "mbr8",
];

#[derive(Clone, Debug, Default)]
pub struct WindGauge {
    pub process: u32,
    pub status: u8,
    pub metronome_state: u8,
    pub baton: Option<Baton>,
}

#[derive(Clone, Debug)]
pub struct MenuGauge {
    pub arrow_r: GaugePane,
    pub arrow_l: GaugePane,
    pub cross: GaugePane,
    pub markers: [GaugePane; 3],
    pub wind_arrow: GaugePane,
    pub cry1: GaugePane,
    pub cry3: GaugePane,
    pub combo: GaugePane,
    pub combo_map: GaugePane,
    pub range: GaugePane,
    pub lr_frame: Option<u8>,
    pub map_ctrl: u8,
    pub map_ctrl_aux: u8,
    pub move_flag: i16,
    pub move_status: i16,
    pub cross_timer: i16,
    pub compass_slot: i16,
    pub reset_flag: u8,
    pub trigger_flag: u8,
}

impl Default for MenuGauge {
    fn default() -> Self {
        Self {
            arrow_r: GaugePane::new("r"),
            arrow_l: GaugePane::new("l"),
            cross: GaugePane::new("crs1"),
            markers: [
                GaugePane::new("yrig"),
                GaugePane::new("yup"),
                GaugePane::new("ylef"),
            ],
            wind_arrow: GaugePane::new("wiy1"),
            cry1: GaugePane::new("cry1"),
            cry3: GaugePane::new("cry3"),
            combo: GaugePane::new("cmb1"),
            combo_map: GaugePane::new("cmtm"),
            range: GaugePane::new("rng1"),
            lr_frame: None,
            map_ctrl: 0,
            map_ctrl_aux: 0,
            move_flag: 0,
            move_status: 0,
            cross_timer: 0,
            compass_slot: 0,
            reset_flag: 0,
            trigger_flag: 0,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct MeterGauges {
    pub seed: [i32; 3],
    pub icon: IconGauge,
    pub enemy: EnemyGauge,
    pub magic: MagicGauge,
    pub wind: WindGauge,
    pub menu: MenuGauge,
    pub heart_slots: Vec<GaugePane>,
    pub heart_shadows: Vec<GaugePane>,
    pub heart_full: GaugePane,
}

impl Default for MeterGauges {
    fn default() -> Self {
        Self {
            seed: [RND_SEED; 3],
            icon: IconGauge {
                base: GaugePane::new("basv"),
                tag_pane: GaugePane::new("bar1"),
                note: GaugePane::new("ba0r"),
                ..IconGauge::default()
            },
            enemy: EnemyGauge {
                bar: std::array::from_fn(|i| GaugePane::new(ENEMY_BAR_TAGS[i])),
                eye_a: GaugePane::new("bey1"),
                eye_b: GaugePane::new("bey2"),
                nick: GaugePane::new("enem"),
                nick2: GaugePane::new("enm2"),
                zako_edge: GaugePane::new("ene3"),
                zako_fill: GaugePane::new("ene2"),
                zako_mid: GaugePane::new("ene1"),
                zako_back: GaugePane::new("enk2"),
                zako_line: GaugePane::new("enek"),
                eye_flag: false,
            },
            magic: MagicGauge::default(),
            wind: WindGauge::default(),
            menu: MenuGauge::default(),
            heart_slots: (0..20)
                .map(|i| GaugePane::new(&format!("ht{i:02}")))
                .collect(),
            heart_shadows: (0..20)
                .map(|i| GaugePane::new(&format!("hk{i:02}")))
                .collect(),
            heart_full: GaugePane::new("htfl"),
        }
    }
}

pub const ENEMY_BAR_TAGS: [&str; 4] = ["ben1", "ben2", "ben3", "ben4"];

impl MeterGauges {
    pub fn bind(&mut self, parts1: &Screen, parts3: &Screen) {
        self.icon.base.bind(parts1);
        self.icon.tag_pane.bind(parts1);
        self.icon.note.bind(parts1);
        for pane in self.enemy.bar.iter_mut() {
            pane.bind(parts1);
        }
        self.enemy.eye_a.bind(parts1);
        self.enemy.eye_b.bind(parts1);
        self.enemy.nick.bind(parts1);
        self.enemy.nick2.bind(parts1);
        self.enemy.zako_edge.bind(parts1);
        self.enemy.zako_fill.bind(parts1);
        self.enemy.zako_mid.bind(parts1);
        self.enemy.zako_back.bind(parts1);
        self.enemy.zako_line.bind(parts1);
        for pane in self.magic.bars.iter_mut() {
            pane.bind(parts1);
        }
        self.magic.frame.bind(parts1);
        self.magic.shadow.bind(parts1);
        self.magic.key_back.bind(parts1);
        self.magic.front.bind(parts1);
        self.magic.key_front.bind(parts1);
        self.menu.arrow_r.bind(parts1);
        self.menu.arrow_l.bind(parts1);
        self.menu.cross.bind(parts1);
        for pane in self.menu.markers.iter_mut() {
            pane.bind(parts1);
        }
        self.menu.wind_arrow.bind(parts1);
        self.menu.cry1.bind(parts1);
        self.menu.cry3.bind(parts1);
        self.menu.combo.bind(parts1);
        self.menu.combo_map.bind(parts3);
        self.menu.range.bind(parts3);
        for pane in self.heart_slots.iter_mut() {
            pane.bind(parts1);
        }
        for pane in self.heart_shadows.iter_mut() {
            pane.bind(parts1);
        }
        self.heart_full.bind(parts1);
    }
}

pub fn boss_gauge_collect(collect: &Collect) -> bool {
    collect.test(4, 1)
}

pub fn icon_init(g: &mut MeterGauges, env: &mut GaugeEnv) {
    g.icon.note.user_area = 0;
    g.icon.base.user_area = 0;
    g.icon.status_cache = 0;
    env.r_status = ACT_BLANK;
    g.icon.texture_archive = icon_archive(env.r_status);
    g.icon.texture_slot = ICON_TEX_SLOT;
    g.icon.texture_bytes = ICON_TEX_BYTES;
    let name = action_texture_name(ACT_BLANK);
    g.icon.base.change_texture(name);
}

pub fn icon_move(g: &mut MeterGauges, env: &mut GaugeEnv, tuning: &MeterTuning) {
    if env.r_status_force != ACT_BLANK {
        env.r_status = env.r_status_force;
    }
    if g.icon.base.user_area != 0 {
        if env.r_status != ACT_BLANK {
            g.icon.texture_archive = icon_archive(env.r_status);
            g.icon.texture_slot = ICON_TEX_SLOT;
            g.icon.texture_bytes = ICON_TEX_BYTES;
            let name = action_texture_name(env.r_status);
            g.icon.base.change_texture(name);
        }
        g.icon.status_cache = env.r_status;
        g.icon.note.user_area = 1;
        g.icon.base.user_area = 0;
    }
    if g.icon.status_cache != env.r_status {
        g.icon.base.user_area = 1;
    }
    icon_trans(g, env, tuning);
    icon_alpha(g, env);
    env.r_status_force = ACT_BLANK;
}

pub fn icon_draw(g: &mut MeterGauges) {
    g.icon.base.sync_alpha();
    g.icon.tag_pane.sync_alpha();
    g.icon.note.sync_alpha();
}

pub fn icon_alpha(g: &mut MeterGauges, env: &GaugeEnv) {
    let flags = env.status_flags;
    let hounded = env.event_hind & 8 != 0;
    let show_set = flags & STTS_4000 != 0
        || (flags & STTS_40 != 0 && hounded)
        || flags & STTS_100 != 0
        || flags & STTS_200 != 0
        || flags & STTS_8 != 0
        || flags & STTS_10 != 0
        || flags & STTS_800000 != 0
        || flags & STTS_20 != 0
        || flags & STTS_80 != 0
        || flags & STTS_100000 != 0
        || flags & STTS_200000 != 0
        || flags & STTS_40000 != 0
        || flags & STTS_80000 != 0
        || flags & STTS_400000 != 0;
    if show_set {
        if g.icon.alpha_frame < ICON_ALPHA_MAX_FRAME {
            g.icon.alpha_frame += 1;
        } else {
            g.icon.alpha_frame = ICON_ALPHA_MAX_FRAME;
        }
    } else if flags & (STTS_2000 | STTS_8000) != 0 {
        if g.icon.alpha_frame > ICON_ALPHA_DIM_FRAME {
            g.icon.alpha_frame -= 1;
        } else if g.icon.alpha_frame < ICON_ALPHA_DIM_FRAME {
            g.icon.alpha_frame += 1;
        } else {
            g.icon.alpha_frame = ICON_ALPHA_DIM_FRAME;
        }
    } else {
        let next = if g.icon.alpha_frame < ICON_ALPHA_MAX_FRAME {
            g.icon.alpha_frame + 1
        } else {
            ICON_ALPHA_MAX_FRAME
        };
        g.icon.alpha_frame = next;
        g.icon.action_mode |= ICON_ACTION_MODE_BIT;
    }
    let alpha = ramp(
        ICON_ALPHA_FADE_STEPS as i32,
        i32::from(g.icon.alpha_frame),
        0,
    );
    if env.r_status == ACT_BLANK {
        g.icon.base.set_now_alpha_zero();
    } else {
        g.icon.base.set_now_alpha(alpha);
    }
    if flags & STTS_20 != 0 && env.up_button == 0 {
        g.icon.tag_pane.set_now_alpha(1.0 - alpha);
        g.icon.note.set_now_alpha(1.0 - alpha);
    } else {
        g.icon.tag_pane.set_now_alpha(alpha);
        g.icon.note.set_now_alpha(alpha);
    }
    if is_bit8(g.icon.flags, ICON_MOVE_FLAG_BIT) && g.icon.base.user_area == 0 {
        g.icon.base.show();
    } else {
        g.icon.base.hide();
    }
}

pub fn icon_trans(g: &mut MeterGauges, env: &mut GaugeEnv, tuning: &MeterTuning) {
    let flags = env.status_flags;
    let hounded = env.event_hind & 8 != 0;
    let menu_item_push = env.menu_open
        && env.picture_status != 2
        && (env.push_menu_button == 1 || (env.menu_status == 1 && env.push_menu_button == 0));
    let menu_collect_push = env.menu_open
        && (env.push_menu_button == 2 || (env.menu_status == 2 && env.push_menu_button == 0));
    let push_set = menu_item_push
        || menu_collect_push
        || flags & STTS_800000 != 0
        || (flags & STTS_20 != 0 && env.up_button == 1)
        || flags & STTS_80 != 0
        || flags & STTS_100000 != 0
        || flags & STTS_200000 != 0
        || flags & STTS_80000 != 0
        || flags & STTS_400000 != 0;
    let next_status: i16 = if flags & STTS_4000 != 0
        || (flags & STTS_40 != 0 && hounded)
        || (flags & STTS_100 != 0 || env.r_status == ACT_HIDDEN)
    {
        1
    } else if flags & STTS_20000 != 0 {
        2
    } else if flags & STTS_40000 != 0 {
        4
    } else if push_set {
        1
    } else if flags & STTS_20 != 0 {
        match env.button_icon_mode {
            0 | 1 | 2 | 3 => 3,
            7 | 8 | 9 | 10 => 5,
            _ => 1,
        }
    } else {
        0
    };
    if g.icon.move_status != next_status {
        g.icon.move_flag = 1;
        off_bit8(&mut g.icon.flags, ICON_MOVE_FLAG_BIT);
    }
    g.icon.move_status = next_status;

    if g.icon.flags != ICON_MOVE_FLAG_SETTLED {
        if !is_bit8(g.icon.flags, ICON_MOVE_FLAG_BIT) {
            if g.icon.move_frame >= ICON_MOVE_ENTER_FRAMES {
                g.icon.move_frame = ICON_MOVE_ENTER_FRAMES;
                g.icon.move_flag = 0;
                on_bit8(&mut g.icon.flags, ICON_MOVE_FLAG_BIT);
                match g.icon.move_status {
                    1 => {
                        g.icon.now_x = 0.0;
                        g.icon.now_y = ICON_MOVE_HIDE_Y;
                    }
                    2 => {
                        g.icon.now_x = f32::from(tuning.icon_slide_x);
                        g.icon.now_y = f32::from(tuning.icon_slide_y);
                    }
                    3 => {
                        g.icon.now_x = f32::from(tuning.icon_hold_x);
                        g.icon.now_y = f32::from(tuning.icon_hold_y);
                    }
                    4 => {
                        g.icon.now_x = f32::from(tuning.icon_choose_x);
                        g.icon.now_y = f32::from(tuning.icon_choose_y);
                    }
                    5 => {
                        g.icon.now_x =
                            f32::from(tuning.icon_hold_x.wrapping_add(ICON_MOVE_SLIDE_X));
                        g.icon.now_y =
                            f32::from(tuning.icon_hold_y.wrapping_add(ICON_MOVE_SLIDE_Y));
                    }
                    _ => {
                        g.icon.now_x = f32::from(tuning.icon_default_x);
                        g.icon.now_y = f32::from(tuning.icon_default_y);
                    }
                }
            } else {
                g.icon.move_frame += 1;
            }
        }
    } else if g.icon.move_frame <= 0 {
        g.icon.move_frame = 0;
    } else {
        g.icon.move_frame -= 1;
    }

    let move_alpha = ramp(
        ICON_MOVE_ENTER_FRAMES as i32,
        i32::from(g.icon.move_frame),
        0,
    );
    let x = g.icon.now_x;
    let y = g.icon.now_y + (move_alpha * (ICON_MOVE_HIDE_Y - g.icon.now_y));
    g.icon.base.pane_trans(x, y);
    g.icon.tag_pane.pane_trans(x, y);
    g.icon.note.pane_trans(x, y);
}

pub fn enemy_init(g: &mut MeterGauges) {
    g.enemy.bar[1].user_area = 0;
    g.enemy.zako_fill.user_area = 0;
    g.enemy.nick.user_area = 0;
    let nick = rnd_f(&mut g.seed, ENEMY_NICK_PERIOD_SPAN) + ENEMY_NICK_PERIOD_BASE;
    g.enemy.nick2.user_area = nick as i16;
    g.enemy.eye_a.user_area = 0;
    let eye = rnd_f(&mut g.seed, ENEMY_EYE_PERIOD_SPAN) + ENEMY_EYE_PERIOD_BASE;
    g.enemy.eye_b.user_area = ((eye as i32) << ENEMY_EYE_PERIOD_SHIFT) as i16;
}

pub fn enemy_move(g: &mut MeterGauges, env: &GaugeEnv, tuning: &MeterTuning) {
    if env.attention_present {
        if zako_lock_ready(env) {
            zako_enemy_move(g, env);
        } else {
            zako_enemy_hide(g);
        }
    }
    if !env.menu_open
        && env.collect_boss_gauge
        && !env.event_running
        && env.status_flags & (STTS_80 | STTS_20000) == 0
        && (env.stage_boss || env.find_target("Bdk").is_some())
    {
        let demo_stage = BOSS_DEMO_STAGES.iter().any(|name| *name == env.start_stage);
        if (env.stage_boss_demo && !env.stage_boss_enemy) || demo_stage {
            for name in BOSS_SEARCH_ORDER {
                if let Some(target) = env.find_target(name) {
                    boss_enemy_move(g, target, tuning);
                    return;
                }
            }
            boss_enemy_hide(g);
            return;
        }
    }
    boss_enemy_hide(g);
}

fn zako_lock_ready(env: &GaugeEnv) -> bool {
    if env.menu_open || env.event_running {
        return false;
    }
    let Some(target) = env.lock_target else {
        return false;
    };
    if target.max_health <= 0.0 {
        return false;
    }
    if !env.collect_boss_gauge {
        return false;
    }
    if env.lock_truth & 0xFF == 0 {
        return false;
    }
    if target.group != GROUP_ENEMY {
        return false;
    }
    if ZAKO_EXCLUDE_NAMES.contains(&target.name) {
        return false;
    }
    if target.name == ZAKO_PARAM_NAME && target.param & ZAKO_PARAM_MASK == 0 {
        return false;
    }
    true
}

fn anchor_offset(pane: &GaugePane, anchor: &GaugePane) -> [f32; 2] {
    [
        pane.pos_top_left_orig[0] - anchor.pos_top_left_orig[0],
        pane.pos_top_left_orig[1] - anchor.pos_top_left_orig[1],
    ]
}

pub fn zako_enemy_move(g: &mut MeterGauges, env: &GaugeEnv) {
    let Some(target) = env.lock_target else {
        return;
    };
    let screen = target.screen;
    let ratio = target.health / target.max_health;
    if screen[0] > ENEMY_SCREEN_EDGE_X {
        g.enemy.zako_fill.pos_top_left[0] =
            (screen[0] - g.enemy.zako_fill.size_orig[0]) - ENEMY_SCREEN_MARGIN;
    } else {
        g.enemy.zako_fill.pos_top_left[0] = screen[0] + ENEMY_SCREEN_MARGIN;
    }
    g.enemy.zako_fill.pos_top_left[1] = screen[1];

    let anchor = g.enemy.zako_fill.clone();
    for pane in [
        &mut g.enemy.zako_edge,
        &mut g.enemy.zako_mid,
        &mut g.enemy.zako_line,
    ] {
        let offset = anchor_offset(pane, &anchor);
        pane.pos_top_left[0] = anchor.pos_top_left[0] + offset[0];
        pane.pos_top_left[1] = anchor.pos_top_left[1] + offset[1];
        pane.size = pane.size_orig;
        pane.apply_pos_size();
    }
    g.enemy.zako_back.pos_top_left = g.enemy.zako_back.pos_top_left_orig;
    g.enemy.zako_back.size = g.enemy.zako_back.size_orig;
    g.enemy.zako_back.apply_pos_size();

    g.enemy.zako_fill.size[0] = g.enemy.zako_fill.size_orig[0] * ratio;
    g.enemy.zako_fill.size[1] = g.enemy.zako_fill.size_orig[1];
    g.enemy.zako_fill.apply_pos_size();

    if g.enemy.zako_fill.user_area < ENEMY_ZAKO_FADE_FRAMES {
        g.enemy.zako_fill.user_area += 1;
        let alpha = ramp(
            ENEMY_ZAKO_FADE_FRAMES as i32,
            i32::from(g.enemy.zako_fill.user_area),
            0,
        );
        g.enemy.zako_edge.set_now_alpha(alpha);
        g.enemy.zako_fill.set_now_alpha(alpha);
        g.enemy.zako_mid.set_now_alpha(alpha);
        g.enemy.zako_line.set_now_alpha(alpha);
    } else {
        g.enemy.zako_edge.set_init_alpha();
        g.enemy.zako_fill.set_init_alpha();
        g.enemy.zako_mid.set_init_alpha();
        g.enemy.zako_line.set_init_alpha();
    }
}

pub fn boss_enemy_move(g: &mut MeterGauges, target: &EnemyTarget, tuning: &MeterTuning) {
    let ratio = if target.max_health != 0.0 {
        target.health / target.max_health
    } else {
        tuning.boss_gauge_fill
    };
    if g.enemy.bar[0].user_area < ENEMY_BOSS_OPEN_FRAMES {
        g.enemy.bar[0].user_area += 1;
        if g.enemy.bar[0].user_area <= ENEMY_BOSS_RISE_FRAMES {
            let rise = ramp(
                ENEMY_BOSS_BOB_BASE as i32,
                ENEMY_BOSS_RISE_FRAMES as i32 - i32::from(g.enemy.bar[0].user_area),
                0,
            ) * ENEMY_BOSS_SLIDE_DIST;
            g.enemy.bar[2].pane_scale_x(0.0);
            g.enemy.bar[3].pane_scale_x(0.0);
            g.enemy.eye_a.pane_trans(0.0, rise);
            g.enemy.eye_b.pane_trans(0.0, rise);
            g.enemy.nick.pane_trans(0.0, rise);
            g.enemy.nick2.pane_trans(0.0, rise);
            for pane in g.enemy.bar.iter_mut() {
                pane.pane_trans(0.0, rise);
            }
        } else {
            let slide = ramp(
                ENEMY_BOSS_SLIDE_FRAMES as i32,
                i32::from(g.enemy.bar[0].user_area) - ENEMY_BOSS_RISE_FRAMES as i32,
                2,
            );
            g.enemy.bar[2].pane_scale_x(ratio * slide);
            g.enemy.bar[3].pane_scale_x(ratio * slide);
        }
    } else {
        g.enemy.bar[2].pane_scale_x(ratio);
        g.enemy.bar[3].pane_scale_x(ratio);
    }
    for index in [2usize, 3usize] {
        let pane = &mut g.enemy.bar[index];
        let x = (pane.pos_top_left_orig[0] + pane.size_orig[0]) - pane.size[0];
        let y = pane.pos_top_left_orig[1];
        pane.move_rect(x, y);
    }
    if g.enemy.bar[1].user_area < ENEMY_BOSS_FADE_FRAMES {
        g.enemy.bar[1].user_area += 1;
        let alpha = ramp(
            ENEMY_BOSS_FADE_FRAMES as i32,
            i32::from(g.enemy.bar[1].user_area),
            0,
        );
        g.enemy.eye_a.set_now_alpha(alpha);
        g.enemy.eye_b.set_now_alpha(alpha);
        g.enemy.nick.set_now_alpha(alpha);
        g.enemy.nick2.set_now_alpha(alpha);
        for pane in g.enemy.bar.iter_mut() {
            pane.set_now_alpha(alpha);
        }
    } else {
        boss_anime(g);
        g.enemy.nick.set_init_alpha();
        g.enemy.nick2.set_init_alpha();
        for pane in g.enemy.bar.iter_mut() {
            pane.set_init_alpha();
        }
    }
}

pub fn zako_enemy_hide(g: &mut MeterGauges) {
    if g.enemy.zako_fill.user_area > 0 {
        g.enemy.zako_fill.user_area -= 1;
        let alpha = ramp(
            ENEMY_ZAKO_FADE_FRAMES as i32,
            i32::from(g.enemy.zako_fill.user_area),
            0,
        );
        g.enemy.zako_edge.set_now_alpha(alpha);
        g.enemy.zako_fill.set_now_alpha(alpha);
        g.enemy.zako_mid.set_now_alpha(alpha);
        g.enemy.zako_line.set_now_alpha(alpha);
    } else {
        g.enemy.zako_edge.set_now_alpha_zero();
        g.enemy.zako_fill.set_now_alpha_zero();
        g.enemy.zako_mid.set_now_alpha_zero();
        g.enemy.zako_line.set_now_alpha_zero();
    }
}

pub fn boss_enemy_hide(g: &mut MeterGauges) {
    if g.enemy.bar[1].user_area > 0 {
        g.enemy.bar[1].user_area -= 1;
        let alpha = ramp(
            ENEMY_BOSS_FADE_FRAMES as i32,
            i32::from(g.enemy.bar[1].user_area),
            0,
        );
        g.enemy.eye_a.set_now_alpha(alpha);
        g.enemy.eye_b.set_now_alpha(alpha);
        g.enemy.nick.set_now_alpha(alpha);
        g.enemy.nick2.set_now_alpha(alpha);
        for pane in g.enemy.bar.iter_mut() {
            pane.set_now_alpha(alpha);
        }
    } else {
        g.enemy.eye_a.set_now_alpha_zero();
        g.enemy.eye_b.set_now_alpha_zero();
        g.enemy.nick.set_now_alpha_zero();
        g.enemy.nick2.set_now_alpha_zero();
        for pane in g.enemy.bar.iter_mut() {
            pane.set_now_alpha_zero();
        }
    }
}

pub const ENEMY_BOB_TABLE: [i16; 15] = [
    0x04, 0x0A, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02,
];

pub fn boss_bob_limits() -> [i16; 15] {
    let mut limits = [0i16; 15];
    limits[0] = ENEMY_BOB_TABLE[0];
    for index in 1..15 {
        limits[index] = limits[index - 1].wrapping_add(ENEMY_BOB_TABLE[index]);
    }
    limits
}

pub fn boss_anime(g: &mut MeterGauges) {
    let mut bob = 0.0f32;
    let limits = boss_bob_limits();
    g.enemy.nick.user_area = g.enemy.nick.user_area.wrapping_add(1);
    if g.enemy.nick.user_area >= g.enemy.nick2.user_area {
        g.enemy.nick.user_area = 0;
        let period = rnd_f(&mut g.seed, ENEMY_NICK_PERIOD_SPAN) + ENEMY_NICK_PERIOD_BASE;
        g.enemy.nick2.user_area = period as i16;
    } else if g.enemy.nick.user_area < limits[10] {
        let frame = g.enemy.nick.user_area;
        if frame < limits[0] {
            bob = ramp(ENEMY_BOB_TABLE[0] as i32, i32::from(frame), 0) * ENEMY_BOSS_BOB_STEP;
        } else if frame < limits[1] {
            bob = ENEMY_BOSS_BOB_STEP;
        } else if frame < limits[2] {
            bob = ramp(
                ENEMY_BOB_TABLE[2] as i32,
                limits[2] as i32 - i32::from(frame),
                0,
            ) * ENEMY_BOSS_BOB_STEP;
        } else if frame < limits[3] {
            bob = 0.0;
        } else if frame < limits[4] {
            bob = ramp(
                ENEMY_BOB_TABLE[4] as i32,
                limits[4] as i32 - i32::from(frame),
                0,
            ) * ENEMY_BOSS_BOB_STEP;
        } else if frame < limits[5] {
            bob = ENEMY_BOSS_BOB_STEP;
        } else if frame < limits[6] {
            bob = ramp(
                ENEMY_BOB_TABLE[6] as i32,
                limits[6] as i32 - i32::from(frame),
                0,
            ) * ENEMY_BOSS_BOB_STEP;
        } else if frame < limits[7] {
            bob = 0.0;
        } else if frame < limits[8] {
            bob = ramp(
                ENEMY_BOB_TABLE[8] as i32,
                limits[8] as i32 - i32::from(frame),
                0,
            ) * ENEMY_BOSS_BOB_STEP;
        } else if frame < limits[9] {
            bob = ENEMY_BOSS_BOB_STEP;
        } else if frame < limits[10] {
            bob = ramp(
                ENEMY_BOB_TABLE[10] as i32,
                limits[10] as i32 - i32::from(frame),
                0,
            ) * ENEMY_BOSS_BOB_STEP;
        } else if frame < limits[11] {
            bob = 0.0;
        } else if frame < limits[12] {
            bob = ramp(
                ENEMY_BOB_TABLE[12] as i32,
                limits[12] as i32 - i32::from(frame),
                0,
            ) * ENEMY_BOSS_BOB_STEP;
        } else if frame < limits[13] {
            bob = ENEMY_BOSS_BOB_STEP;
        } else if frame < limits[14] {
            bob = ramp(
                ENEMY_BOB_TABLE[14] as i32,
                limits[14] as i32 - i32::from(frame),
                0,
            ) * ENEMY_BOSS_BOB_STEP;
        }
    }
    g.enemy.nick.pane_trans(0.0, -bob);
    g.enemy.eye_a.pane_trans(0.0, -bob);
    g.enemy.eye_b.pane_trans(0.0, -bob);
    g.enemy.nick2.pane_trans(0.0, bob);
    boss_eye_anime(g);
}

pub fn boss_eye_anime(g: &mut MeterGauges) {
    let half = g.enemy.eye_b.user_area / ENEMY_EYE_PERIOD_SHIFT;
    g.enemy.eye_a.user_area = g.enemy.eye_a.user_area.wrapping_add(1);
    let alpha = if g.enemy.eye_flag {
        if g.enemy.eye_a.user_area < half {
            ramp(i32::from(half), i32::from(g.enemy.eye_a.user_area), 2)
        } else {
            ramp(
                i32::from(half),
                i32::from(g.enemy.eye_b.user_area) - i32::from(g.enemy.eye_a.user_area),
                2,
            )
        }
    } else {
        0.0
    };
    if g.enemy.eye_a.user_area == g.enemy.eye_b.user_area {
        g.enemy.eye_a.user_area = 0;
        let period = rnd_f(&mut g.seed, ENEMY_EYE_PERIOD_SPAN) + ENEMY_EYE_PERIOD_BASE;
        g.enemy.eye_b.user_area = ((period as i32) << ENEMY_EYE_PERIOD_SHIFT) as i16;
        g.enemy.eye_flag = !g.enemy.eye_flag;
    }
    g.enemy.eye_a.set_now_alpha(alpha);
    g.enemy.eye_b.set_now_alpha(alpha);
}

pub fn magic_init(g: &mut MeterGauges, env: &GaugeEnv) {
    if g.magic.ready_flag >> 1 & 1 != 0 {
        g.magic.gauge_max = g.magic.max_magic as u8;
        g.magic.gauge_pixels =
            (g.magic.bars[0].size_orig[0] * g.magic.magic as f32 * MAGIC_WIDTH_UNITS) as u16;
    } else {
        g.magic.gauge_max = 0;
        g.magic.gauge_pixels = 0;
        g.magic.ready_flag |= MAGIC_READY_FLAG;
    }
    g.magic.mode_flag = 0;
    for index in 0..5 {
        g.magic.bars[index].user_area = 0;
    }
    g.magic.bars[5].user_area = 1;
    g.magic.frame.user_area = 0;
    g.magic.front.user_area = 0;
    g.magic.shadow.user_area = 0;
    g.magic.key_front.user_area = 0;
    g.magic.alpha_value = 0.0;
    let max_life = env.max_life;
    magic_length(g, f32::from(g.magic.gauge_max) / MAGIC_MAX_UNITS);
    let pixels = g.magic.gauge_pixels as f32 / MAGIC_PIXEL_SCALE;
    magic_change(g, pixels, max_life);
    g.magic.base_front_white = g.magic.front.white;
    g.magic.base_front_black = g.magic.front.black;
    g.magic.base_bar0_white = g.magic.bars[0].white;
    g.magic.base_bar0_black = g.magic.bars[0].black;
    g.magic.base_bar1_white = g.magic.bars[1].white;
    g.magic.base_bar1_black = g.magic.bars[1].black;
    magic_trans_now_init(g, max_life);
    g.magic.frame.set_init_alpha();
    g.magic.front.set_init_alpha();
    g.magic.key_front.set_init_alpha();
}

pub fn magic_move(g: &mut MeterGauges, env: &mut GaugeEnv, out: &mut GaugeOutput) {
    if env.status_flags & STTS_40000 != 0 {
        let fly = if env.player_is_makar {
            env.fly_makar
        } else if env.player_is_medli {
            env.fly_medli
        } else {
            FlyGauge {
                flying: 0,
                max_timer: 1,
                timer: 1,
            }
        };
        fly_gauge_move(g, env, fly);
        if g.magic.bars[2].user_area == 1 {
            g.magic.bars[2].user_area = 0;
        }
        g.magic.bars[5].user_area = 1;
    } else {
        magic_gauge_move(g, env, out);
        if g.magic.key_front.user_area != 0 {
            g.magic.key_front.user_area = 0;
        }
    }
}

pub fn magic_gauge_move(g: &mut MeterGauges, env: &mut GaugeEnv, out: &mut GaugeOutput) {
    let max_life = env.max_life;
    if g.magic.bars[0].user_area == 1 {
        g.magic.front.set_white(g.magic.base_front_white);
        g.magic.front.set_black(g.magic.base_front_black);
        g.magic.shadow.set_white(g.magic.base_front_white);
        g.magic.shadow.set_black(g.magic.base_front_black);
        for index in 0..4 {
            g.magic.bars[index * 2].set_white(g.magic.base_bar0_white);
            g.magic.bars[index * 2].set_black(g.magic.base_bar0_black);
            g.magic.bars[index * 2 + 1].set_white(g.magic.base_bar1_white);
            g.magic.bars[index * 2 + 1].set_black(g.magic.base_bar1_black);
        }
        g.magic.bars[0].user_area = 0;
        magic_trans_scale(g, 0.0, 0.0, 1.0, max_life);
        magic_init_trans(g, max_life);
        let pixels = g.magic.gauge_pixels as f32 / MAGIC_PIXEL_SCALE;
        magic_change(g, pixels, max_life);
    }
    if g.magic.item_max_magic_count != 0 {
        let mut total = g.magic.max_magic + g.magic.item_max_magic_count;
        if total > MAGIC_MAX_CLAMP {
            total = MAGIC_MAX_CLAMP;
        }
        g.magic.max_magic = total;
        g.magic.item_max_magic_count = 0;
    }
    if g.magic.gauge_max as i32 != g.magic.max_magic || g.magic.bars[2].user_area == 0 {
        if g.magic.gauge_max < g.magic.max_magic as u8 {
            g.magic.gauge_max = g.magic.gauge_max.wrapping_add(1);
        } else if g.magic.gauge_max > g.magic.max_magic as u8 {
            g.magic.gauge_max = g.magic.gauge_max.wrapping_sub(1);
        }
        let span = f32::from(g.magic.gauge_max) / MAGIC_MAX_UNITS;
        magic_length(g, span);
        if g.magic.gauge_max as i32 == g.magic.max_magic {
            let span = if g.magic.max_magic <= MAGIC_MAX_SHORT {
                0.5
            } else {
                1.0
            };
            magic_length(g, span);
            g.magic.bars[2].user_area = 1;
        }
    }
    if g.magic.max_magic != 0 && g.magic.item_magic_count != 0 {
        let mut total = g.magic.magic + g.magic.item_magic_count;
        if total < 0 {
            total = 0;
        }
        g.magic.magic = total;
        g.magic.item_magic_count = 0;
    }
    if g.magic.magic > g.magic.max_magic {
        g.magic.magic = g.magic.max_magic;
    }
    let allowed = !env.overlap_request
        && (env.status_flags & STTS_40 == 0
            || env.event_hind & 0x20 == 0
            || !env.demo_start
            || env.player_status1 & PLAYER_STTS1_UNK2000 == 0);
    if allowed {
        let target =
            (g.magic.bars[0].size_orig[0] * g.magic.magic as f32 * MAGIC_WIDTH_UNITS) as u16;
        let now = g.magic.gauge_pixels;
        if now < target {
            if i32::from(target - now) < MAGIC_DRAIN_STEP {
                g.magic.gauge_pixels = target;
            } else {
                g.magic.gauge_pixels = now.wrapping_add(MAGIC_DRAIN_STEP as u16);
            }
            if g.magic.gauge_pixels as i32 % MAGIC_DRAIN_SOUND_STEP == 0 {
                out.sounds.push(SE_MP_GAUGE_INC);
            }
            let pixels = g.magic.gauge_pixels as f32 / MAGIC_PIXEL_SCALE;
            magic_change(g, pixels, max_life);
            g.magic.bars[5].user_area = 1;
        } else if now > target {
            if i32::from(now - target) < MAGIC_DRAIN_STEP {
                g.magic.gauge_pixels = 0;
            } else {
                g.magic.gauge_pixels = now.wrapping_sub(MAGIC_DRAIN_STEP as u16);
            }
            if g.magic.gauge_pixels as i32 % MAGIC_DRAIN_SOUND_STEP == 0 {
                out.sounds.push(SE_MP_GAUGE_DEC);
            }
            if g.magic.gauge_pixels == 0 {
                out.sounds.push(SE_MP_GAUGE_DEC_FIN);
            }
            let pixels = g.magic.gauge_pixels as f32 / MAGIC_PIXEL_SCALE;
            magic_change(g, pixels, max_life);
            g.magic.bars[5].user_area = 1;
        } else if g.magic.bars[5].user_area != 0 {
            let pixels = now as f32 / MAGIC_PIXEL_SCALE;
            magic_change(g, pixels, max_life);
            g.magic.bars[5].user_area = 0;
        }
    } else {
        g.magic.bars[5].user_area = 1;
    }
    magic_color(g, env);
    magic_alpha(g, env);
}

pub fn fly_gauge_move(g: &mut MeterGauges, env: &GaugeEnv, fly: FlyGauge) {
    let fade;
    if fly.flying != 0 {
        let span = fly.timer as f32 / fly.max_timer as f32 * MAGIC_FLY_UNITS * MAGIC_FLY_SPAN;
        magic_init_trans(g, env.max_life);
        magic_change(g, span, env.max_life);
        if g.magic.bars[0].user_area == 0 {
            g.magic.front.set_white(FLY_COLOR_A);
            g.magic.front.set_black(FLY_COLOR_B);
            g.magic.shadow.set_white(FLY_COLOR_A);
            g.magic.shadow.set_black(FLY_COLOR_B);
            for index in 0..4 {
                g.magic.bars[index * 2].set_white(FLY_COLOR_C);
                g.magic.bars[index * 2].set_black(FLY_COLOR_D);
                g.magic.bars[index * 2 + 1].set_white(FLY_COLOR_E);
                g.magic.bars[index * 2 + 1].set_black(FLY_COLOR_F);
            }
            g.magic.bars[0].user_area = 1;
            g.magic.bars[1].user_area = 0x14;
            g.magic.shadow.user_area = 0;
        }
        if g.magic.front.user_area < MAGIC_FLY_FADE_FRAMES {
            g.magic.front.user_area += 1;
            fade = ramp(
                MAGIC_FLY_FADE_FRAMES as i32,
                i32::from(g.magic.front.user_area),
                0,
            );
        } else {
            fade = 1.0;
        }
        if g.magic.shadow.user_area < MAGIC_FLY_TRAIL_FRAMES {
            g.magic.shadow.user_area += 1;
            let eye = env.player_eye_screen;
            let center = g.magic.front.pos_center;
            if g.magic.shadow.user_area > MAGIC_FLY_TRAIL_SPLIT {
                let head = ramp(
                    MAGIC_FLY_TRAIL_RAMP as i32,
                    MAGIC_FLY_TRAIL_FRAMES as i32 - i32::from(g.magic.shadow.user_area),
                    0,
                );
                let tail = ramp(
                    MAGIC_FLY_TRAIL_RAMP as i32,
                    i32::from(g.magic.shadow.user_area) - MAGIC_FLY_TRAIL_SPLIT as i32,
                    0,
                );
                magic_trans_scale(
                    g,
                    head * (eye[0] - center[0]),
                    head * (eye[1] - center[1]),
                    (tail * MAGIC_FLY_TRAIL_SCALE) + MAGIC_FLY_TRAIL_SCALE,
                    env.max_life,
                );
            } else {
                magic_trans_scale(
                    g,
                    eye[0] - center[0],
                    eye[1] - center[1],
                    MAGIC_FLY_TRAIL_SCALE,
                    env.max_life,
                );
            }
        }
    } else {
        if g.magic.front.user_area > 0 {
            g.magic.front.user_area -= 1;
            fade = ramp(
                MAGIC_FLY_FADE_FRAMES as i32,
                i32::from(g.magic.front.user_area),
                0,
            );
        } else {
            fade = 0.0;
            g.magic.front.user_area = 0;
            magic_trans_now_init(g, env.max_life);
        }
        if g.magic.shadow.user_area != 0 {
            g.magic.shadow.user_area = 0;
        }
    }
    g.magic.alpha_value = fade;
}

pub fn magic_length(g: &mut MeterGauges, span: f32) {
    let full = g.magic.bars[0].size_orig[0] * MAGIC_FRAME_SPAN;
    g.magic.frame.size[0] = (full * span) + (g.magic.frame.size_orig[0] - full);
    g.magic.frame.pos_center[0] = g.magic.frame.pos_top_left_orig[0] + g.magic.frame.size[0] / 2.0;
    g.magic.frame.cpos_move();
    set_magic_picture(&mut g.magic.front);
    g.magic.front.size[0] =
        g.magic.frame.size[0] + (g.magic.front.size_orig[0] - g.magic.frame.size_orig[0]);
    g.magic.front.pos_center[0] = g.magic.front.pos_top_left_orig[0] + g.magic.front.size[0] / 2.0;
    g.magic.front.cpos_move();
    set_magic_picture(&mut g.magic.shadow);
    g.magic.shadow.size[0] =
        g.magic.frame.size[0] + (g.magic.shadow.size_orig[0] - g.magic.frame.size_orig[0]);
    g.magic.shadow.pos_center[0] =
        g.magic.shadow.pos_top_left_orig[0] + g.magic.shadow.size[0] / 2.0;
    g.magic.shadow.cpos_move();
}

fn set_magic_picture(pane: &mut GaugePane) {
    pane.picture.m134 = 1;
    pane.picture.m124 = 0.0;
    pane.picture.m128 = 0.0;
    pane.picture.m12c = MAGIC_QUAD_SLOPE_X;
    pane.picture.m130 = MAGIC_QUAD_SLOPE_Y;
}

pub fn magic_change(g: &mut MeterGauges, value: f32, max_life: i32) {
    g.magic.now_magic = (value / g.magic.bars[0].size_orig[0]) * 4.0;
    magic_trans_now_init(g, max_life);
    let mut rest = value;
    for index in 0..MAGIC_BARS {
        if rest >= g.magic.bars[index].size_orig[0] {
            g.magic.bars[index].pane_scale_xy(1.0);
            g.magic.bars[index].show();
            rest -= g.magic.bars[index].size_orig[0];
        } else if rest <= 0.0 {
            g.magic.bars[index].pane_scale_xy(1.0);
            g.magic.bars[index].hide();
        } else {
            let w = rest;
            let h = g.magic.bars[index].size_orig[1];
            g.magic.bars[index].resize_picture(w, h);
            g.magic.bars[index].show();
            rest = 0.0;
        }
    }
}

pub fn magic_trans_now_init(g: &mut MeterGauges, max_life: i32) {
    let y = if max_life <= MAGIC_ROW_MAX_LIFE {
        MAGIC_ROW_SHIFT_Y
    } else {
        0.0
    };
    g.magic.frame.pane_trans(
        g.magic.frame.pos_center[0] - g.magic.frame.pos_center_orig[0],
        y,
    );
    g.magic.front.pane_trans(
        g.magic.front.pos_center[0] - g.magic.front.pos_center_orig[0],
        y,
    );
    g.magic.key_front.pane_trans(
        g.magic.key_front.pos_center[0] - g.magic.key_front.pos_center_orig[0],
        y,
    );
    for pane in g.magic.bars.iter_mut() {
        pane.pane_trans(pane.pos_center[0] - pane.pos_center_orig[0], y);
    }
}

pub fn magic_init_trans(g: &mut MeterGauges, max_life: i32) {
    let y = if max_life <= MAGIC_ROW_MAX_LIFE {
        MAGIC_ROW_SHIFT_Y
    } else {
        0.0
    };
    g.magic.frame.pane_trans(0.0, y);
    g.magic.front.pane_trans(0.0, y);
    g.magic.key_front.pane_trans(0.0, y);
    for pane in g.magic.bars.iter_mut() {
        pane.pane_trans(0.0, y);
    }
}

pub fn magic_trans_scale(g: &mut MeterGauges, x: f32, y: f32, scale: f32, max_life: i32) {
    let y = if max_life <= MAGIC_ROW_MAX_LIFE {
        y + MAGIC_ROW_SHIFT_Y
    } else {
        y
    };
    g.magic.front.pane_scale_xy(scale);
    g.magic.front.pane_trans(x, y);
    g.magic.shadow.pos_center[0] = g.magic.shadow.pos_center_orig[0] * scale;
    g.magic.shadow.pos_center[1] = g.magic.shadow.pos_center_orig[1] * scale;
    g.magic.shadow.pane_scale_xy(scale);
    let anchor_x = g.magic.front.pos_center_orig[0];
    g.magic.frame.pos_center[0] =
        x + (anchor_x + (scale * (g.magic.frame.pos_center_orig[0] - anchor_x)));
    let anchor_y = g.magic.front.pos_center_orig[1];
    g.magic.frame.pos_center[1] =
        y + (anchor_y + (scale * (g.magic.frame.pos_center_orig[1] - anchor_y)));
    g.magic.frame.pane_scale_xy(scale);
    g.magic.key_front.pos_center[0] =
        x + (anchor_x + (scale * (g.magic.key_front.pos_center_orig[0] - anchor_x)));
    g.magic.key_front.pos_center[1] =
        y + (anchor_y + (scale * (g.magic.key_front.pos_center_orig[1] - anchor_y)));
    g.magic.key_front.pane_scale_xy(scale);
    g.magic.key_back.pos_center[0] = g.magic.key_back.pos_center_orig[0] * scale;
    g.magic.key_back.pos_center[1] = g.magic.key_back.pos_center_orig[1] * scale;
    g.magic.key_back.pane_scale_xy(scale);
    for pane in g.magic.bars.iter_mut() {
        pane.pos_center[0] = x + (anchor_x + (scale * (pane.pos_center_orig[0] - anchor_x)));
        pane.pos_center[1] = y + (anchor_y + (scale * (pane.pos_center_orig[1] - anchor_y)));
        pane.pos_center[0] -= (pane.size_orig[0] * scale) / 2.0;
        pane.size[0] = scale * pane.width();
        pane.size[1] = scale * pane.height();
        pane.pos_center[0] += pane.size[0] / 2.0;
        pane.cpos_move();
        pane.size = pane.size_orig;
    }
}

pub fn magic_color(g: &mut MeterGauges, env: &mut GaugeEnv) {
    let flags = env.status_flags;
    let blink = env.dragon_shield
        || g.magic.live_flag & MAGIC_LIVE_FLAG != 0
        || env.cut_roll
        || env.leaf_fly
        || (env.bow_aim && env.arrow_keep_type != 0 && env.item_id != -1);
    if flags & (STTS_8 | STTS_10 | STTS_800000 | STTS_100) == 0 && blink {
        let mut white = [0u8; 4];
        let mut black = [0u8; 4];
        if g.magic.bars[4].user_area == 0 {
            g.magic.bars[4].user_area = 1;
        }
        if g.magic.bars[3].user_area < MAGIC_BLINK_PERIOD {
            g.magic.bars[3].user_area += 1;
        } else {
            g.magic.bars[3].user_area = 0;
        }
        let ramp_value = if g.magic.bars[3].user_area < MAGIC_BLINK_RAMP {
            ramp(
                MAGIC_BLINK_RAMP as i32,
                i32::from(g.magic.bars[3].user_area),
                0,
            )
        } else {
            ramp(
                MAGIC_BLINK_RAMP as i32,
                MAGIC_BLINK_SPAN as i32 - i32::from(g.magic.bars[3].user_area),
                0,
            )
        };
        for index in 0..3 {
            let base = g.magic.base_front_white[index] as f32;
            white[index] = (base - ((base - 255.0) * ramp_value)) as u8;
            let base = g.magic.base_front_black[index] as f32;
            black[index] = (base - ((base - 255.0) * ramp_value)) as u8;
        }
        white[3] = 0xFF;
        black[3] = 0x00;
        g.magic.front.set_white(white);
        g.magic.front.set_black(black);
        g.magic.shadow.set_white(white);
        g.magic.shadow.set_black(black);
        g.magic.live_flag &= !MAGIC_LIVE_FLAG;
        return;
    }
    if g.magic.bars[4].user_area == 1 {
        g.magic.front.set_white(g.magic.base_front_white);
        g.magic.front.set_black(g.magic.base_front_black);
        g.magic.shadow.set_white(g.magic.base_front_white);
        g.magic.shadow.set_black(g.magic.base_front_black);
        g.magic.bars[4].user_area = 0;
    }
}

pub fn magic_alpha(g: &mut MeterGauges, env: &mut GaugeEnv) {
    let flags = env.status_flags;
    let hidden = g.magic.max_magic == 0
        || flags & STTS_4000 != 0
        || (flags & STTS_40 != 0
            && env.event_hind & 0x20 != 0
            && env.player_status1 & PLAYER_STTS1_UNK2000 == 0)
        || env.collect_mode == COLLECT_MODE_COMPASS
        || flags & STTS_200000 != 0
        || flags & STTS_80 != 0
        || flags & STTS_100000 != 0
        || flags & STTS_20000 != 0
        || flags & STTS_20 != 0
        || flags & STTS_80000 != 0
        || env.auction_flag
        || flags & STTS_400000 != 0
        || flags & STTS_800000 != 0
        || (env.start_stage == MENU_BLINK_STAGE && env.mini_game_type == MINIGAME_6);
    if hidden {
        if g.magic.frame.user_area > 0 {
            g.magic.frame.user_area -= 1;
        } else {
            g.magic.frame.user_area = 0;
        }
    } else if flags & (STTS_8 | STTS_10) != 0 {
        if g.magic.frame.user_area < MAGIC_ALPHA_DIM_FRAMES {
            g.magic.frame.user_area += 1;
        } else if g.magic.frame.user_area > MAGIC_ALPHA_DIM_FRAMES {
            g.magic.frame.user_area -= 1;
        } else {
            g.magic.frame.user_area = MAGIC_ALPHA_DIM_FRAMES;
        }
    } else if g.magic.frame.user_area < MAGIC_ALPHA_FULL_FRAMES {
        g.magic.frame.user_area += 1;
    } else {
        g.magic.frame.user_area = MAGIC_ALPHA_FULL_FRAMES;
    }
    g.magic.alpha_value = ramp(
        MAGIC_ALPHA_FULL_FRAMES as i32,
        i32::from(g.magic.frame.user_area),
        0,
    );
}

pub fn gauge_alpha(g: &mut MeterGauges, env: &GaugeEnv) {
    let heart = env.heart_alpha;
    let gauge = g.magic.alpha_value;
    for pane in g.heart_slots.iter_mut() {
        pane.set_now_alpha(heart);
    }
    for pane in g.heart_shadows.iter_mut() {
        pane.set_now_alpha(heart);
    }
    let scaled = g.heart_full.now_alpha as f32 * heart;
    g.heart_full.now_alpha = scaled as u8;
    g.heart_full.alpha = (scaled * heart) as u8;
    for pane in g.magic.bars.iter_mut() {
        pane.set_now_alpha(gauge);
    }
    g.magic.frame.set_now_alpha(gauge);
    g.magic.front.set_now_alpha(gauge);
    g.magic.key_front.set_now_alpha(gauge);
}

pub fn menu_init(g: &mut MeterGauges, env: &GaugeEnv) {
    g.menu.arrow_r.user_area = MENU_LR_INIT_R;
    g.menu.arrow_l.user_area = 0;
    g.menu.cross.user_area = 0;
    g.menu.compass_slot = env.compass_slot as i16;
    g.menu.markers[0].user_area = env.compass_slot as i16;
}

pub fn menu_move(g: &mut MeterGauges, env: &mut GaugeEnv, tuning: &MeterTuning) {
    menu_lr_move(g, env, tuning);
    menu_plus_move(g, env, tuning);
}

pub fn menu_lr_move(g: &mut MeterGauges, env: &mut GaugeEnv, tuning: &MeterTuning) {
    if g.menu.lr_frame.is_none() {
        g.menu.lr_frame = Some(env.push_menu_button);
    }
    let frame = g.menu.lr_frame.unwrap_or(0);
    if env.status_flags & STTS_200000 != 0 || env.collect_mode == COLLECT_MODE_BUTTON {
        g.menu.arrow_r.set_now_alpha_zero();
        g.menu.arrow_l.set_now_alpha_zero();
        return;
    }
    let mut span = MENU_LR_DEFAULT_SPAN;
    if frame != env.push_menu_button {
        if frame != 0 && env.push_menu_button != 0 {
            span = tuning.menu_lr_half_width * 2;
            g.menu.arrow_r.user_area = span;
            g.menu.arrow_l.user_area = MENU_LR_L_HOLD;
        } else {
            g.menu.arrow_l.user_area = 0;
        }
        g.menu.lr_frame = Some(env.push_menu_button);
    }
    let half = span / 2;
    if env.push_menu_button != 0 {
        if g.menu.arrow_r.user_area != 0 {
            g.menu.arrow_r.user_area -= 1;
        }
    } else if g.menu.arrow_r.user_area < half {
        g.menu.arrow_r.user_area += 1;
    } else if g.menu.arrow_r.user_area > half {
        g.menu.arrow_r.user_area -= 1;
    }
    let now = i32::from(g.menu.arrow_r.user_area);
    let half_now = i32::from(half);
    let (slide, alpha) = if now > half_now {
        let slide = ramp(half_now, span as i32 - now, 0);
        let alpha = if g.menu.arrow_l.user_area != 0 {
            1.0
        } else {
            ramp(half_now, i32::from(g.menu.arrow_r.user_area) - half_now, 0)
        };
        (slide, alpha)
    } else {
        let slide = ramp(half_now, now, 0);
        let alpha = if g.menu.arrow_l.user_area != 0 {
            1.0
        } else {
            ramp(half_now, half_now - i32::from(g.menu.arrow_r.user_area), 0)
        };
        (slide, alpha)
    };
    g.menu.arrow_r.pane_trans(slide * MENU_LR_STEP_X, 0.0);
    g.menu.arrow_l.pane_trans(slide * -MENU_LR_STEP_X, 0.0);
    g.menu.arrow_r.set_now_alpha(alpha);
    g.menu.arrow_l.set_now_alpha(alpha);
}

pub fn menu_plus_move(g: &mut MeterGauges, env: &mut GaugeEnv, tuning: &MeterTuning) {
    if g.menu.compass_slot != env.compass_slot as i16 {
        if g.menu.map_ctrl == 0 {
            if env.map.enabled {
                g.menu.map_ctrl = 1;
                g.menu.map_ctrl_aux = 0;
            }
            g.menu.trigger_flag = 1;
        }
        g.menu.reset_flag = 0;
        g.menu.compass_slot = env.compass_slot as i16;
    }
    let flags = env.status_flags;
    let hounded = env.event_hind & 0x100 != 0;
    let maju_stage = env.next_stage == MENU_MAJU_STAGE && !env.event_0801;
    let hide_stage = MENU_HIDE_STAGES.iter().any(|name| *name == env.next_stage);
    let hide_set = flags & STTS_4000 != 0
        || env.overlap_request
        || (flags & STTS_40 != 0 && hounded)
        || flags & STTS_200000 != 0
        || flags & STTS_100 != 0
        || flags & STTS_80 != 0
        || flags & STTS_100000 != 0
        || flags & STTS_20000 != 0
        || flags & STTS_8 != 0
        || flags & STTS_10 != 0
        || flags & STTS_800000 != 0
        || flags & STTS_20 != 0
        || flags & STTS_80000 != 0
        || flags & STTS_800 != 0
        || flags & STTS_1000 != 0
        || (flags & STTS_400 == 0 && maju_stage)
        || (flags & STTS_400 == 0 && hide_stage)
        || env.mini_game_type == MINIGAME_1
        || env.mini_game_type == MINIGAME_6;
    let next_status: i16 = if hide_set {
        1
    } else if env.pane_mode == 1 || env.pane_mode == 2 {
        if g.menu.map_ctrl != 0 && env.map.enabled {
            2
        } else if env.map.enabled {
            3
        } else if flags & STTS_400 != 0 && env.save_table != SAVE_TABLE_TOTG {
            7
        } else {
            6
        }
    } else if env.pane_mode == 5 || env.pane_mode == 9 || env.pane_mode == 3 {
        4
    } else {
        5
    };
    if g.menu.move_status != next_status {
        g.menu.move_flag = 1;
    }
    g.menu.move_status = next_status;

    let choose_visible =
        (env.event_0908 && env.up_button == 0) || (env.up_button == 1 && flags & STTS_400000 == 0);
    let (x, y);
    if g.menu.move_flag != 0 {
        if g.menu.cross.user_area >= MENU_PLUS_FADE_FRAMES {
            g.menu.cross.user_area = MENU_PLUS_FADE_FRAMES;
        } else {
            g.menu.cross.user_area += 1;
        }
        match g.menu.move_status {
            2 => {
                x = f32::from(tuning.menu_gauge_x);
                y = f32::from(tuning.menu_gauge_y);
                g.menu.cross.hide();
                g.menu.cry1.hide();
                g.menu.cry3.hide();
                g.menu.markers[0].show();
                if choose_visible {
                    g.menu.markers[1].show();
                } else {
                    g.menu.markers[1].hide();
                }
                g.menu.markers[2].show();
                g.menu.move_flag = 0;
            }
            3 => {
                x = f32::from(tuning.menu_gauge_x);
                y = f32::from(tuning.menu_gauge_y);
                g.menu.cross.show();
                g.menu.cry1.show();
                g.menu.markers[0].hide();
                if choose_visible {
                    g.menu.markers[1].show();
                    g.menu.cry3.show();
                } else {
                    g.menu.markers[1].hide();
                    g.menu.cry3.hide();
                }
                g.menu.markers[2].hide();
                g.menu.move_flag = 0;
            }
            4 => {
                x = f32::from(tuning.menu_gauge_x) + MENU_PLUS_SHIFT_X;
                y = f32::from(tuning.menu_gauge_y);
                g.menu.cross.show();
                g.menu.cry1.show();
                if choose_visible {
                    g.menu.cry3.show();
                } else {
                    g.menu.cry3.hide();
                }
                g.menu.markers[0].hide();
                g.menu.markers[1].hide();
                g.menu.markers[2].hide();
                g.menu.move_flag = 0;
            }
            5 => {
                x = f32::from(tuning.menu_gauge_x);
                y = f32::from(tuning.menu_gauge_y);
                g.menu.cross.hide();
                g.menu.cry1.hide();
                g.menu.cry3.hide();
                g.menu.markers[0].hide();
                if choose_visible {
                    g.menu.markers[1].show();
                } else {
                    g.menu.markers[1].hide();
                }
                g.menu.markers[2].show();
                g.menu.move_flag = 0;
            }
            6 => {
                x = f32::from(tuning.menu_gauge_x);
                y = f32::from(tuning.menu_gauge_y);
                if choose_visible {
                    g.menu.cross.show();
                    g.menu.cry1.hide();
                    g.menu.cry3.show();
                } else {
                    g.menu.cross.hide();
                    g.menu.cry1.hide();
                    g.menu.cry3.hide();
                }
                g.menu.markers[0].hide();
                g.menu.markers[1].hide();
                g.menu.markers[2].hide();
                g.menu.move_flag = 0;
            }
            7 => {
                x = f32::from(tuning.menu_gauge_x);
                y = f32::from(tuning.menu_gauge_y);
                if choose_visible {
                    g.menu.cross.show();
                    g.menu.cry1.show();
                    g.menu.cry3.show();
                } else {
                    g.menu.cross.hide();
                    g.menu.cry1.show();
                    g.menu.cry3.hide();
                }
                g.menu.markers[0].hide();
                g.menu.markers[1].hide();
                g.menu.markers[2].hide();
                g.menu.move_flag = 0;
            }
            _ => {
                x = f32::from(tuning.menu_gauge_x);
                y = f32::from(tuning.menu_gauge_y);
            }
        }
    } else {
        if g.menu.cross.user_area <= 0 {
            g.menu.cross.user_area = 0;
        } else {
            g.menu.cross.user_area -= 1;
        }
        match g.menu.move_status {
            4 => {
                x = f32::from(tuning.menu_gauge_x) + MENU_PLUS_SHIFT_X;
                y = f32::from(tuning.menu_gauge_y);
            }
            _ => {
                x = f32::from(tuning.menu_gauge_x);
                y = f32::from(tuning.menu_gauge_y);
            }
        }
    }

    let mut offset = [[0.0f32; 2]; 3];
    if g.menu.move_status == 4 || g.menu.move_status == 5 {
        let combo = g.menu.combo.clone();
        let combo_map = g.menu.combo_map.clone();
        let range = g.menu.range.clone();
        let combo_center = combo.pos_center;
        offset[0][0] = ((combo_center[0] + combo.size[0] / 2.0)
            - g.menu.markers[0].pos_center_orig[0])
            - g.menu.markers[0].size_orig[0] / 2.0;
        offset[0][1] = combo_center[1] - g.menu.markers[0].pos_center_orig[1];
        offset[1][0] = combo_center[0] - g.menu.markers[1].pos_center_orig[0];
        offset[1][1] = ((combo_center[1] - combo.size[1] / 2.0)
            - g.menu.markers[1].pos_center_orig[1])
            - g.menu.markers[1].size_orig[1] / 2.0;
        offset[2][0] = g.menu.markers[2].size_orig[0] / 2.0
            + ((combo_map.pos_center[0] + range.size[0] / 2.0)
                - g.menu.markers[2].pos_center_orig[0]);
        offset[2][1] = combo_map.pos_center[1] - g.menu.markers[2].pos_center_orig[1];
    } else {
        let half_x = f32::from(env.map.left_up_x) + (f32::from(env.map.size_x) / 2.0);
        let half_y = f32::from(env.map.left_up_y) + (f32::from(env.map.size_y) / 2.0);
        for index in 0..3 {
            offset[index][0] = half_x - g.menu.markers[1].pos_center_orig[0];
            offset[index][1] = half_y - g.menu.markers[0].pos_center_orig[1];
        }
    }

    let arrow_x = env.map.left_up_x as f32 - MENU_MAP_OFF_X;
    let arrow_y = env.map.left_up_y as f32 - MENU_MAP_OFF_Y;
    let angle = atan2s(env.wind_vec[0], env.wind_vec[1]);
    g.menu.wind_arrow.rotate(
        g.menu.wind_arrow.size_orig[0],
        g.menu.wind_arrow.size_orig[1] / 2.0,
        f32::from(angle) * MENU_DEG_PER_TURN + MENU_DEG_BIAS,
    );
    g.menu.cross.pane_trans(x, y);
    g.menu.wind_arrow.pane_trans(
        (arrow_x - MENU_PLUS_COMPASS_OFF_X) + f32::from(tuning.wind_arrow_x),
        (arrow_y + MENU_PLUS_COMPASS_OFF_Y) + f32::from(tuning.wind_arrow_y),
    );
    for index in 0..3 {
        g.menu.markers[index].pane_trans(x + offset[index][0], y + offset[index][1]);
    }
    let fade = ramp(
        MENU_PLUS_FADE_FRAMES as i32,
        MENU_PLUS_FADE_FRAMES as i32 - i32::from(g.menu.cross.user_area),
        0,
    );
    g.menu.cross.set_now_alpha(fade);
    env.map_alpha = fade * 255.0;
    if !env.wind_arrow_active || env.menu_open {
        g.menu.wind_arrow.set_now_alpha_zero();
    } else {
        g.menu.wind_arrow.set_now_alpha(fade);
    }
    for index in 0..3 {
        if !env.menu_open {
            g.menu.markers[index].set_now_alpha(fade);
        } else {
            g.menu.markers[index].set_now_alpha_zero();
        }
    }
}

pub fn wind_init(g: &mut MeterGauges, env: &mut GaugeEnv, out: &mut GaugeOutput) {
    g.wind.process = PROCESS_ERROR_ID;
    g.wind.status = 0;
    env.operate_wind_cancel = true;
    out.operate_wind_cancel = true;
}

pub fn metronome_init(g: &mut MeterGauges) {
    g.wind.metronome_state = METRONOME_OPEN;
    if let Some(baton) = g.wind.baton.as_mut() {
        baton.delete();
    }
    g.wind.baton = None;
}

pub fn wind_move(g: &mut MeterGauges, env: &GaugeEnv, out: &mut GaugeOutput, procs: &mut MsgProcs) {
    if g.wind.status != env.operate_wind {
        if env.operate_wind == WIND_OPERATE_ACTIVE {
            out.spawn_process = Some(NAME_OPERATE_WIND);
            g.wind.process = procs.spawn(NAME_OPERATE_WIND, MsgAppend::None);
        }
        g.wind.status = env.operate_wind;
    } else if env.operate_wind != WIND_OPERATE_ACTIVE
        && g.wind.process != PROCESS_ERROR_ID
        && procs.search_by_id(g.wind.process).is_none()
    {
        g.wind.process = PROCESS_ERROR_ID;
    }
}

pub fn metronome_move(g: &mut MeterGauges, env: &GaugeEnv) {
    let open_set =
        env.metronome && env.status_flags & (STTS_8 | STTS_10 | STTS_800000 | STTS_20) == 0;
    if open_set {
        if g.wind.metronome_state == METRONOME_OPEN && g.wind.baton.is_none() {
            g.wind.baton = Some(Baton::create());
        }
        if let Some(baton) = g.wind.baton.as_mut() {
            if baton.open() {
                baton.step();
            }
        }
        g.wind.metronome_state = METRONOME_CLOSING;
    } else if g.wind.metronome_state == METRONOME_CLOSING {
        if let Some(baton) = g.wind.baton.as_mut() {
            if baton.close() {
                g.wind.metronome_state = METRONOME_CLOSED;
            } else {
                baton.step();
            }
        }
    } else if g.wind.metronome_state == METRONOME_CLOSED && g.wind.baton.is_some() {
        g.wind.metronome_state = METRONOME_OPEN;
        if let Some(baton) = g.wind.baton.as_mut() {
            baton.delete();
        }
        g.wind.baton = None;
    }
}

pub fn magic_fill_ratio(g: &MeterGauges) -> f32 {
    if g.magic.bars[0].size_orig[0] == 0.0 {
        return 0.0;
    }
    g.magic.gauge_pixels as f32 / (g.magic.bars[0].size_orig[0] * MAGIC_WIDTH_UNITS)
}

pub fn magic_ratio(g: &MeterGauges) -> f32 {
    if g.magic.max_magic == 0 {
        return 0.0;
    }
    g.magic.magic as f32 / g.magic.max_magic as f32
}

pub fn draw(g: &mut MeterGauges) -> Vec<GaugeQuad> {
    icon_draw(g);
    let mut quads = Vec::new();
    let mut push = |pane: &GaugePane| {
        if pane.found && pane.visible {
            quads.push(pane.quad());
        }
    };
    push(&g.menu.wind_arrow);
    for pane in g.menu.markers.iter() {
        push(pane);
    }
    push(&g.menu.arrow_r);
    push(&g.menu.arrow_l);
    push(&g.menu.cross);
    push(&g.enemy.eye_a);
    push(&g.enemy.eye_b);
    push(&g.enemy.nick);
    push(&g.enemy.nick2);
    for pane in g.enemy.bar.iter() {
        push(pane);
    }
    push(&g.enemy.zako_edge);
    push(&g.enemy.zako_fill);
    push(&g.enemy.zako_mid);
    push(&g.enemy.zako_back);
    push(&g.enemy.zako_line);
    for pane in g.magic.bars.iter() {
        push(pane);
    }
    push(&g.magic.frame);
    push(&g.magic.front);
    push(&g.magic.key_front);
    push(&g.icon.base);
    push(&g.icon.tag_pane);
    push(&g.icon.note);
    quads
}

pub fn create(g: &mut MeterGauges, env: &mut GaugeEnv, out: &mut GaugeOutput) {
    out.reset();
    icon_init(g, env);
    enemy_init(g);
    magic_init(g, env);
    menu_init(g, env);
    wind_init(g, env, out);
    metronome_init(g);
}

pub fn init(world: &mut World) {
    world.init_resource::<MeterGauges>();
    world.init_resource::<GaugeEnv>();
    world.init_resource::<GaugeOutput>();
    world.init_resource::<MeterTuning>();
}

pub fn execute(
    mut g: ResMut<MeterGauges>,
    mut env: ResMut<GaugeEnv>,
    tuning: Res<MeterTuning>,
    mut out: ResMut<GaugeOutput>,
    mut procs: ResMut<MsgProcs>,
) {
    out.reset();
    icon_move(&mut g, &mut env, &tuning);
    enemy_move(&mut g, &env, &tuning);
    magic_move(&mut g, &mut env, &mut out);
    wind_move(&mut g, &env, &mut out, &mut procs);
    metronome_move(&mut g, &env);
    menu_move(&mut g, &mut env, &tuning);
    gauge_alpha(&mut g, &env);
    out.map_alpha = env.map_alpha;
    out.button_action_mode = g.icon.action_mode;
}

pub fn setup(world: &mut World) {
    init(world);
}
