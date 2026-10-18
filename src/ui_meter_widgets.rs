use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;

use crate::j2d::{self, Screen};
use crate::sim::actor::Actor;
use crate::sim::daynight::DayNight;
use crate::sim::flags::EventFlags;
use crate::sim::pickup::Rupees;
use crate::sim::player_status::{PlayerStatus, STTS0_BOW_AIM};
use crate::text::msgproc::MsgProcs;
use crate::ui_meter_gauges as gauges;
use crate::ui_meter_gauges::{
    self, GaugeEnv, GaugeOutput, MeterGauges, MeterTuning, boss_gauge_collect,
};
use crate::ui_meter_vitals as vitals;
use crate::ui_meter_vitals::{self, Vitals, VitalsEnv};

pub const SCREEN_PARTS1: &str = "main_parts1.blo";
pub const SCREEN_PARTS2: &str = "main_parts2.blo";
pub const SCREEN_PARTS3: &str = "main_parts3.blo";
pub const SCREEN_TIMER1: &str = "time_swim_1.blo";
pub const SCREEN_TIMER2: &str = "time_swim_2.blo";
pub const SCREEN_CHOICE_ROAD: &str = "choice_road.blo";

pub const TEXTURE_MASK: &str = "swimtime_meter_mask.bti";
pub const TEXTURE_SCREW: &str = "screw_01.bti";
pub const TEXTURE_TEKARI: &str = "tekari.bti";
pub const TEXTURE_BOW: &str = "bow_01.bti";

pub const ARROW_TEXTURES: [&str; 4] = [
    "arrow_00.bti",
    "arrow_01.bti",
    "arrow_02.bti",
    "arrow_03.bti",
];

pub const AMMO_GLOW_TEXTURES: [&str; 3] = [
    "HM_J2_ARWG_FLAME00",
    "HM_J2_ARWI_COLD00",
    "HM_J2_ARWF_LIGHT00",
];

pub const RUPEE_GLOW_TEXTURE: &str = "HM_J2_RUPYLIGHT";

pub const SE_MAP_SHOW: u32 = 0x0818;
pub const SE_MAP_SHOW_END: u32 = 0x0819;
pub const SE_MAP_CANCEL: u32 = 0x081A;
pub const SE_COMPASS_MOVE: u32 = 0x081F;
pub const SE_RUPEE_UP_1: u32 = 0x0839;
pub const SE_RUPEE_UP_2: u32 = 0x083A;
pub const SE_RUPEE_DOWN_1: u32 = 0x083B;
pub const SE_RUPEE_DOWN_2: u32 = 0x083C;
pub const SE_SWIM_TIMER_ON: u32 = 0x083F;
pub const SE_SWIM_TIMER_BLINK: u32 = 0x0840;
pub const SE_MAP_ZOOM: u32 = 0x08C2;

pub const ST_SCREEN_OFF: u32 = 0x0000_4000;
pub const ST_EVENT: u32 = 0x0000_0040;
pub const ST_SCOPE: u32 = 0x0000_0080;
pub const ST_MESSAGE: u32 = 0x0000_0100;
pub const ST_SHIP_RIDE: u32 = 0x0000_0400;
pub const ST_MENU_ITEM: u32 = 0x0000_0008;
pub const ST_MENU_COLLECT: u32 = 0x0000_0010;
pub const ST_MENU_MAP: u32 = 0x0000_0020;
pub const ST_PICTURE: u32 = 0x0002_0000;
pub const ST_CONDUCT: u32 = 0x0020_0000;
pub const ST_PLAYER_HELD: u32 = 0x0010_0000;
pub const ST_WIND_PICK: u32 = 0x0040_0000;
pub const ST_MENU_NAME: u32 = 0x0080_0000;

pub const SAVE_TABLE_DRC: i32 = 0x03;
pub const SAVE_TABLE_FW: i32 = 0x04;
pub const SAVE_TABLE_TOTG: i32 = 0x05;
pub const SAVE_TABLE_ET: i32 = 0x06;
pub const SAVE_TABLE_WT: i32 = 0x07;

pub const STAGE_TYPE_BOSS: u32 = 3;

pub const EVENT_FLAG_MAJYU_DOOR: u32 = 0x0801;
pub const EVENT_FLAG_RECOLLECT_WT: u32 = 0x3D10;
pub const EVENT_FLAG_RECOLLECT_ET: u32 = 0x3D20;
pub const EVENT_FLAG_RECOLLECT_FW: u32 = 0x3D40;
pub const EVENT_FLAG_RECOLLECT_DRC: u32 = 0x3D80;

pub const RECOLLECT_TABLE: [(i32, u32); 4] = [
    (SAVE_TABLE_DRC, EVENT_FLAG_RECOLLECT_DRC),
    (SAVE_TABLE_FW, EVENT_FLAG_RECOLLECT_FW),
    (SAVE_TABLE_ET, EVENT_FLAG_RECOLLECT_ET),
    (SAVE_TABLE_WT, EVENT_FLAG_RECOLLECT_WT),
];

pub const SLOT_BEAST_FIRST: u8 = 24;
pub const SLOT_BAIT_FIRST: u8 = 36;
pub const SLOT_RESERVE_FIRST: u8 = 48;

pub const ITEM_BOW: u8 = 0x27;
pub const ITEM_MAGIC_ARROW: u8 = 0x35;
pub const ITEM_LIGHT_ARROW: u8 = 0x36;

pub const MINI_GAME_BOW: i32 = 8;

pub const NEXT_STAGE_MAJYU_E: &str = "MajyuE";
pub const START_STAGE_SEA_T: &str = "sea_T";

pub const MAP_ICON_NONE: u8 = 0;
pub const MAP_ICON_SELF: u8 = 2;
pub const MAP_ICON_FREE: u8 = 3;

pub const HEAP_SIZE: usize = 0x2a819;
pub const TILE_SIZE: usize = 0xc00;
pub const TILE_ALIGN: usize = 0x20;

pub const COMPASS_CLOSED: u8 = 1;
pub const COMPASS_DIR_OPENING: u8 = 2;
pub const COMPASS_DIR_OPEN: u8 = 3;
pub const COMPASS_DIR_OPENED: u8 = 5;
pub const COMPASS_WIND_OPEN: u8 = 6;
pub const COMPASS_WIND_OPENING: u8 = 7;
pub const COMPASS_WIND_CLOSING: u8 = 8;
pub const COMPASS_CLOSING: u8 = 9;

pub const DIR_SHOW: i16 = 1;
pub const DIR_GROW: i16 = 2;
pub const DIR_HALF: i16 = 0x14;
pub const DIR_PEAK: i16 = 10;
pub const DIR_LOOP: i16 = 30;
pub const DIR_SCALE_TABLE: [f32; 4] = [1.0, 1.9, 1.0, 1.9];
pub const DIR_TRANS_TABLE: [i32; 4] = [0, 30, 0, 30];
pub const DIR_WHITE_TABLE: [u8; 16] = [
    0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0xFF, 0xFF,
];
pub const DIR_BLACK_TABLE: [u8; 16] = [
    0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0x00, 0xFF, 0x00,
];
pub const DIR_ALPHA: f32 = 0.8627451;

pub const SCREEN_CENTER_X: f32 = 320.0;
pub const SCREEN_CENTER_Y: f32 = 240.0;

pub const KEY_LIGHT_FRAMES: [i16; 5] = [0x1e, 0x28, 0x2d, 0x32, 0x3a];
pub const CLOCK_INIT_X: [i16; 3] = [0x0071, 0x0062, 0x009A];
pub const CLOCK_INIT_Y: [i16; 3] = [0x01B3, 0x018C, 0x0194];
pub const CLOCK_SCALE_X: [f32; 3] = [0.813, 0.75, 0.5];
pub const CLOCK_SCALE_Y: [f32; 3] = [0.667, 0.583, 0.604];

pub const DAY_WINDOW_FROM: f32 = 90.0;
pub const DAY_WINDOW_TO: f32 = 270.0;

pub const WALLET_SMALL: u8 = 0;
pub const WALLET_MEDIUM: u8 = 1;
pub const RUPEE_MAX_SMALL: i32 = 200;
pub const RUPEE_MAX_MEDIUM: i32 = 1000;
pub const RUPEE_MAX_LARGE: i32 = 5000;
pub const RUPEE_SOUND_STEP: i32 = 5;
pub const KEY_MAX: i32 = 99;

pub const ALPHA_SQUARE: u8 = 0;
pub const ALPHA_ROOT: u8 = 1;
pub const ALPHA_LINEAR: u8 = 2;
pub const ALPHA_OPEN_STEPS: i16 = 6;

pub const SWIM_BUBBLE_COUNT: usize = 9;

pub const PLACE_NAME_NONE: u8 = 0;
pub const PLACE_NAME_ON: u8 = 2;

pub const NO_PROCESS: u32 = u32::MAX;

pub const SHIP_ACTOR: &str = "Ship";

pub const DIGIT_COUNT: usize = 4;
pub const SLOT_COUNT_2: usize = 2;
pub const SLOT_COUNT_3: usize = 3;
pub const SLOT_COUNT_4: usize = 4;

pub const HEX_8000: f32 = 0x8000u32 as f32;
pub const DEG_180: f32 = 180.0;

pub const SCROLL_SLOT0: usize = 0;
pub const SCROLL_SLOT1: usize = 1;

pub const MAP_CTRL_BLINK_LIMIT: u8 = 9;
pub const MAP_CTRL_BLINK_IN: u8 = 6;
pub const MAP_CTRL_BLINK_HOLD: u8 = 3;
pub const MAP_CTRL_REST: u8 = 5;
pub const MAP_ANGLE_SCALE: i16 = 2;
pub const MAP_ANGLE_MAX: i16 = 0x3c;
pub const MAP_ANGLE_MIN: i16 = 10;
pub const MAP_SCISSOR_BIAS: f32 = 452.0;
pub const MAP_SCROLL_FRAMES: i32 = 6;
pub const MAP_FADE_FRAMES: i32 = 3;
pub const MAP_BLINK_IN: f32 = 1.2;
pub const MAP_BLINK_OUT: f32 = 0.1;
pub const MAP_BLINK_HOLD: f32 = 1.1;
pub const MAP_ICON_SELF_ALPHA: u8 = 0xCC;
pub const MAP_ICON_SCALE: f32 = 0.8;
pub const MAP_LEFT_UP_X: i16 = 0x23;
pub const MAP_LEFT_UP_Y: i16 = 0x152;
pub const MAP_SLIDE_HOME: i16 = -0xb4;
pub const MAP_ICON_FREE_X: i16 = 0x24e;
pub const MAP_ICON_FREE_Y: i16 = 0x96;
pub const MAP_ALPHA_BASE: f32 = 200.0;

pub fn item_texture(item: u8) -> &'static str {
    crate::ui_meter_vitals::item_texture(item)
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct F2 {
    pub x: f32,
    pub y: f32,
}

impl F2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct F2i {
    pub x: i16,
    pub y: i16,
}

impl F2i {
    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
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
    pub const CLEAR: Self = Self::rgba(0, 0, 0, 0);
    pub const OPAQUE: Self = Self::rgba(0xFF, 0xFF, 0xFF, 0xFF);
    pub const BLACK: Self = Self::rgba(0, 0, 0, 0xFF);
    pub const RED: Self = Self::rgba(0xFF, 0, 0, 0xFF);

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn packed(self) -> u32 {
        u32::from(self.r)
            | (u32::from(self.g) << 8)
            | (u32::from(self.b) << 16)
            | (u32::from(self.a) << 24)
    }

    pub fn unpack(packed: u32) -> Self {
        Self {
            r: (packed & 0xFF) as u8,
            g: ((packed >> 8) & 0xFF) as u8,
            b: ((packed >> 16) & 0xFF) as u8,
            a: ((packed >> 24) & 0xFF) as u8,
        }
    }

    pub fn mix(from: Self, to: Self, ratio: f32) -> Self {
        Self {
            r: (f32::from(from.r) + ratio * (f32::from(to.r) - f32::from(from.r))) as u8,
            g: (f32::from(from.g) + ratio * (f32::from(to.g) - f32::from(from.g))) as u8,
            b: (f32::from(from.b) + ratio * (f32::from(to.b) - f32::from(from.b))) as u8,
            a: from.a,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rotation {
    pub offset: F2,
    pub angle: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Quad {
    pub tag: String,
    pub texture: Option<String>,
    pub appended: Vec<(String, f32)>,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub alpha: u8,
    pub flip: bool,
    pub mirror_offset: F2,
    pub mirror_scale: F2,
    pub black: Color,
    pub white: Color,
    pub corner: [Color; 4],
    pub blend_color: [f32; 4],
    pub blend_alpha: [f32; 4],
    pub rotation: Option<Rotation>,
    pub visible: bool,
}

impl Default for Quad {
    fn default() -> Self {
        Self {
            tag: String::new(),
            texture: None,
            appended: Vec::new(),
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
            alpha: 0xFF,
            flip: false,
            mirror_offset: F2::default(),
            mirror_scale: F2::new(1.0, 1.0),
            black: Color::OPAQUE,
            white: Color::OPAQUE,
            corner: [Color::OPAQUE; 4],
            blend_color: [0.0, 1.0, 1.0, 1.0],
            blend_alpha: [0.0, 1.0, 1.0, 1.0],
            rotation: None,
            visible: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pane {
    pub tag: String,
    pub kind: j2d::Kind,
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
    pub alpha: u8,
    pub flip: bool,
    pub mirror_offset: F2,
    pub mirror_scale: F2,
    pub black: Color,
    pub white: Color,
    pub corner: [Color; 4],
    pub texture: Option<String>,
    pub appended: Vec<(String, f32)>,
    pub blend_color: [f32; 4],
    pub blend_alpha: [f32; 4],
    pub rotation: Option<Rotation>,
}

impl Default for Pane {
    fn default() -> Self {
        Self {
            tag: String::new(),
            kind: j2d::Kind::Pane,
            found: false,
            visible: false,
            pos_top_left_orig: F2::default(),
            pos_top_left: F2::default(),
            pos_center_orig: F2::default(),
            pos_center: F2::default(),
            size_orig: F2::default(),
            size: F2::default(),
            init_alpha: 0,
            now_alpha: 0,
            user_area: 0,
            alpha: 0,
            flip: false,
            mirror_offset: F2::default(),
            mirror_scale: F2::default(),
            black: Color::default(),
            white: Color::default(),
            corner: [Color::default(); 4],
            texture: None,
            appended: Vec::new(),
            blend_color: [0.0; 4],
            blend_alpha: [0.0; 4],
            rotation: None,
        }
    }
}

impl Pane {
    pub fn parts_set(&mut self, layout: &j2d::ScreenPane) {
        self.kind = layout.kind;
        self.found = true;
        self.visible = layout.visible;
        self.pos_top_left_orig = F2::new(layout.bounds.x, layout.bounds.y);
        self.size_orig = F2::new(layout.bounds.w, layout.bounds.h);
        self.pos_center_orig = F2::new(
            layout.bounds.x + layout.bounds.w / 2.0,
            layout.bounds.y + layout.bounds.h / 2.0,
        );
        self.pos_top_left = self.pos_top_left_orig;
        self.pos_center = self.pos_center_orig;
        self.size = self.size_orig;
        self.init_alpha = layout.alpha;
        self.now_alpha = 0;
        self.user_area = 0;
        self.alpha = layout.alpha;
    }

    pub fn bind(&mut self, screen: &Screen, tag: &str) {
        if let Some(layout) = screen.find(tag) {
            self.parts_set(layout);
            self.tag = tag.to_string();
        }
    }

    pub fn width(&self) -> f32 {
        self.size.x
    }

    pub fn height(&self) -> f32 {
        self.size.y
    }

    pub fn angle(&self) -> f32 {
        self.rotation.map_or(0.0, |rot| rot.angle)
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
    }

    pub fn resize(&mut self, w: f32, h: f32) {
        self.size.x = w;
        self.size.y = h;
    }

    pub fn rotate(&mut self, offset_x: f32, offset_y: f32, angle: f32) {
        self.rotation = Some(Rotation {
            offset: F2::new(offset_x, offset_y),
            angle,
        });
    }

    pub fn set_mirror(&mut self, flip: bool, offset: F2, scale: F2) {
        self.flip = flip;
        self.mirror_offset = offset;
        self.mirror_scale = scale;
    }

    pub fn cpos_move(&mut self) {
        self.pos_top_left.x = self.pos_center.x - self.size.x / 2.0;
        self.pos_top_left.y = self.pos_center.y - self.size.y / 2.0;
    }

    pub fn pane_trans(&mut self, trans_x: f32, trans_y: f32) {
        self.pos_center.x = self.pos_center_orig.x + trans_x;
        self.pos_center.y = self.pos_center_orig.y + trans_y;
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

    pub fn set_alpha_value(&mut self, alpha: u8) {
        self.alpha = alpha;
    }

    pub fn set_black(&mut self, black: Color) {
        self.black = black;
    }

    pub fn set_white(&mut self, white: Color) {
        self.white = white;
    }

    pub fn set_corner_color(&mut self, one: Color, two: Color, three: Color, four: Color) {
        self.corner = [one, two, three, four];
    }

    pub fn change_texture(&mut self, name: &str) {
        self.texture = Some(name.to_string());
    }

    pub fn append(&mut self, name: &str, ratio: f32) {
        self.appended.push((name.to_string(), ratio));
    }

    pub fn remove(&mut self) {
        self.appended.clear();
    }

    pub fn blend_init(&mut self, name: &str) {
        self.append(name, 1.0);
        self.blend_color = [0.0, 1.0, 1.0, 1.0];
        self.blend_alpha = [0.0, 1.0, 1.0, 1.0];
    }

    pub fn blend_draw(&mut self, name: &str) {
        self.visible = true;
        self.remove();
        self.append(name, 1.0);
    }

    pub fn quad(&self) -> Quad {
        Quad {
            tag: self.tag.clone(),
            texture: self.texture.clone(),
            appended: self.appended.clone(),
            x: self.pos_top_left.x,
            y: self.pos_top_left.y,
            w: self.size.x,
            h: self.size.y,
            alpha: self.alpha,
            flip: self.flip,
            mirror_offset: self.mirror_offset,
            mirror_scale: self.mirror_scale,
            black: self.black,
            white: self.white,
            corner: self.corner,
            blend_color: self.blend_color,
            blend_alpha: self.blend_alpha,
            rotation: self.rotation,
            visible: self.visible,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Emitter {
    pub alive: bool,
    pub playing: bool,
    pub texture: &'static str,
    pub pos: F2,
    pub alpha: f32,
}

impl Emitter {
    pub fn spawn(texture: &'static str, x: f32, y: f32) -> Self {
        Self {
            alive: true,
            playing: false,
            texture,
            pos: F2::new(x, y),
            alpha: 0.0,
        }
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn kill(&mut self) {
        self.alive = false;
        self.playing = false;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Quad2D {
    pub ready: bool,
    pub mask_texture: &'static str,
    pub scroll_texture: &'static str,
    pub x0: i16,
    pub y0: i16,
    pub x1: i16,
    pub y1: i16,
    pub scale: F2,
    pub black: Color,
    pub white: Color,
    pub scroll: [(i16, i16); 2],
}

impl Quad2D {
    pub fn init(
        mask_texture: &'static str,
        scroll_texture: &'static str,
        scale_x: f32,
        scale_y: f32,
    ) -> Self {
        Self {
            ready: true,
            mask_texture,
            scroll_texture,
            scale: F2::new(scale_x, scale_y),
            black: Color::OPAQUE,
            white: Color::CLEAR,
            ..Self::default()
        }
    }

    pub fn set_pos(&mut self, x0: i16, y0: i16, x1: i16, y1: i16) {
        self.x0 = x0;
        self.y0 = y0;
        self.x1 = x1;
        self.y1 = y1;
    }

    pub fn set_scale(&mut self, scale_x: f32, scale_y: f32) {
        self.scale = F2::new(scale_x, scale_y);
    }

    pub fn set_scroll(&mut self, slot: usize, x: i16, y: i16) {
        self.scroll[slot] = (x, y);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapCtrl {
    pub shown: u8,
    pub flags: u8,
    pub blend_step: u8,
    pub slide: i16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapStat {
    pub timer: i16,
    pub stage_flag: u8,
    pub stay_room: i32,
    pub stay_valid: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MapView {
    pub alpha: u8,
    pub mode: u8,
    pub left_up: F2i,
    pub icon_free_pos: F2i,
    pub icon_free_alpha: u8,
    pub icon_free_scale: f32,
    pub icon_self_alpha: u8,
    pub icon_self_scale: f32,
    pub icon_mode: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AirMeter {
    pub count: i32,
    pub max: i32,
    pub running: bool,
}

impl Default for AirMeter {
    fn default() -> Self {
        Self {
            count: 0,
            max: 900,
            running: false,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AirTimer {
    pub air: AirMeter,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tuning {
    pub wake_steps: i16,
    pub row_scale_x: i16,
    pub row_scale_y: i16,
    pub item_scale_y: i16,
    pub row_offset_x: i16,
    pub row_offset_y: i16,
    pub star_base: i16,
    pub star_span: [i16; 4],
    pub star_jitter: [i16; 5],
    pub clock_scroll_x: [i16; 3],
    pub clock_scroll_y: [i16; 3],
    pub clock_steps: [i16; 3],
    pub bubble_size: f32,
    pub blink_steps: i16,
    pub spin_frames: i16,
    pub blink_short: i16,
    pub bubble_fade: i16,
    pub bubble_birth: i16,
    pub swim_offset: f32,
    pub bubble_top: f32,
    pub low_r: u8,
    pub low_g: u8,
    pub low_b: u8,
    pub tekari_ramp: i16,
    pub tekari_hold_a: i16,
    pub tekari_ramp_b: i16,
    pub tekari_hold_b: i16,
    pub icon_scale: f32,
    pub item_move_frames: i16,
    pub item_move_mode: i32,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            wake_steps: 10,
            row_scale_x: 0x5a,
            row_scale_y: 100,
            item_scale_y: 0x5a,
            row_offset_x: 7,
            row_offset_y: 0xfff4u16 as i16,
            star_base: 0x14,
            star_span: [0xf, 5, 5, 5],
            star_jitter: [0, 0xc, 3, 3, 3],
            clock_scroll_x: [0xffeau16 as i16, 0xffddu16 as i16, 0xfffau16 as i16],
            clock_scroll_y: [0xffd5u16 as i16, 0xfffdu16 as i16, 0xfff3u16 as i16],
            clock_steps: [0x8c, 0x82, 0x8c],
            bubble_size: 0x14 as f32,
            blink_steps: 0x14,
            spin_frames: 0x27,
            blink_short: 3,
            bubble_fade: 4,
            bubble_birth: 10,
            swim_offset: 0.0,
            bubble_top: 10.0,
            low_r: 0xff,
            low_g: 0x32,
            low_b: 0x32,
            tekari_ramp: 10,
            tekari_hold_a: 0x14,
            tekari_ramp_b: 4,
            tekari_hold_b: 0x32,
            icon_scale: 1.4,
            item_move_frames: 9,
            item_move_mode: 0,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Panes {
    pub anchor: Pane,
    pub dungeon_item: [Pane; 3],
    pub boss_health: Pane,
    pub boss_health_bar: Pane,
    pub boss_health_back: Pane,
    pub boss_frame_1: Pane,
    pub boss_frame_2: Pane,
    pub boss_mark: Pane,
    pub boss_mark_2: Pane,
    pub boss_name: [Pane; 4],
    pub boss_name_2: Pane,
    pub boss_name_3: Pane,
    pub boss_name_4: Pane,
    pub boss_name_5: Pane,
    pub boss_name_6: Pane,
    pub heart: [Pane; 20],
    pub heart_shadow: [Pane; 20],
    pub heart_flush: Pane,
    pub magic_bar: [Pane; 8],
    pub magic_bar_back_3: Pane,
    pub magic_bar_back_2: Pane,
    pub magic_bar_back_1: Pane,
    pub magic_key_2: Pane,
    pub magic_key_1: Pane,
    pub compass_face: Pane,
    pub compass_needle_mag: Pane,
    pub compass_needle: Pane,
    pub compass_ring: Pane,
    pub compass_base: Pane,
    pub clock_bar: Pane,
    pub clock_ring_2: Pane,
    pub clock_horizon_1: Pane,
    pub clock_horizon_2: Pane,
    pub clock_star_day: [Pane; 3],
    pub clock_moon_2: Pane,
    pub clock_sun_2: Pane,
    pub clock_circle: [Pane; 4],
    pub clock_needle: Pane,
    pub clock_needle_back: Pane,
    pub clock_ring_1: Pane,
    pub clock_star_night: [Pane; 4],
    pub clock_moon_1: Pane,
    pub clock_clip: Pane,
    pub clock_cloud: [Pane; 3],
    pub clock_sun_1: Pane,
    pub compass_frame: Pane,
    pub key_back_1: Pane,
    pub key_back_2: Pane,
    pub rupee_digit_1: [Pane; DIGIT_COUNT],
    pub key_digit_1: [Pane; SLOT_COUNT_2],
    pub rupee_digit_2: [Pane; DIGIT_COUNT],
    pub key_digit_2: [Pane; SLOT_COUNT_2],
    pub key_frame: Pane,
    pub magic_gauge_base: Pane,
    pub item_back_dark: Pane,
    pub item_back_pale: Pane,
    pub item_back: [Pane; SLOT_COUNT_3],
    pub item_white: [Pane; SLOT_COUNT_3],
    pub item_label_1: [Pane; SLOT_COUNT_3],
    pub item_label_2: [Pane; SLOT_COUNT_3],
    pub item_name: [Pane; SLOT_COUNT_3],
    pub item_icon: [Pane; SLOT_COUNT_3],
    pub item_blast_1: [Pane; SLOT_COUNT_3],
    pub item_blast_2: [Pane; SLOT_COUNT_3],
    pub item_blast_3: [Pane; SLOT_COUNT_3],
    pub magic_bar_base: Pane,
    pub magic_name_1: [Pane; SLOT_COUNT_2],
    pub magic_name_2: [Pane; SLOT_COUNT_2],
    pub magic_icon: [Pane; SLOT_COUNT_2],
    pub magic_name: Pane,
    pub magic_name_right: Pane,
    pub heart_count_a: Pane,
    pub heart_count_b: Pane,
    pub item_count_a: Pane,
    pub item_count_b: Pane,
    pub weight_frame: Pane,
    pub weight_icon: Pane,
    pub bomb_frame: Pane,
    pub bomb_icon: Pane,
    pub meter_center: Pane,
    pub ammo_frame: [Pane; SLOT_COUNT_2],
    pub ammo_glow: [Pane; SLOT_COUNT_2],
    pub rupee_back: Pane,
    pub rupee_light: Pane,
    pub bubble: [Pane; SWIM_BUBBLE_COUNT],
    pub gauge_frame: Pane,
    pub gauge_mask: Pane,
    pub gauge_soul: Pane,
    pub gauge_blink: Pane,
    pub gauge_main: Pane,
    pub gauge_kilari: Pane,
    pub gauge_cover: Pane,
    pub direction_arrow: [Pane; 4],
}

#[derive(Resource, Debug, Clone)]
pub struct Widget {
    pub tuning: Tuning,
    pub pane: Panes,
    pub timer_screen: [bool; 2],
    pub timer_screen_data: [Option<Screen>; 2],
    pub status_flags: u32,
    pub wake_alpha: f32,
    pub wake_step: i16,
    pub rupee_count: u16,
    pub rupee_adjust: i32,
    pub rupee_sound_set: u8,
    pub rupee_sound_on: u8,
    pub key_shown: u8,
    pub key_adjust: i16,
    pub key_light_frame: [i16; 5],
    pub star_frame: [i16; 8],
    pub compass_state: u8,
    pub compass_map_flag: u8,
    pub compass_sound_flag: u8,
    pub compass_time_pass: f32,
    pub compass_day_light: i16,
    pub map_zoom: u8,
    pub map_control: MapCtrl,
    pub map_stat: MapStat,
    pub map_view: MapView,
    pub ammo_type: u8,
    pub ammo_old_type: u8,
    pub ammo_glow: [Emitter; 2],
    pub rupee_glow: Emitter,
    pub dir_timer: i16,
    pub item_move_flag: u8,
    pub item_move_button: u8,
    pub item_move_slot: u8,
    pub item_move_timer: i16,
    pub item_parts: Pane,
    pub item_icon: Pane,
    pub swim_state: i16,
    pub swim_tekari_frame: i16,
    pub swim_bubble_wait: i16,
    pub swim_base_black: Color,
    pub swim_base_white: Color,
    pub swim_full_black: Color,
    pub swim_full_white: Color,
    pub swim_mask_black: Color,
    pub swim_low: Color,
    pub swim_blink_steps: i32,
    pub swim_bubble_rise: [f32; SWIM_BUBBLE_COUNT],
    pub swim_bubble_sway: [f32; SWIM_BUBBLE_COUNT],
    pub kilari: Quad2D,
    pub clock: [Quad2D; 3],
    pub place_name: bool,
    pub button_icon_mode: u8,
    pub metronome_armed: u8,
    pub wind_id: u32,
    pub wind_status: u8,
    pub auction_flag: u8,
    pub recollection_copied: bool,
    pub sounds: Vec<u32>,
    pub sound_enabled: bool,
}

impl Default for Widget {
    fn default() -> Self {
        Self {
            tuning: Tuning::default(),
            pane: Panes::default(),
            timer_screen: [false; 2],
            timer_screen_data: [None, None],
            status_flags: 0,
            wake_alpha: 0.0,
            wake_step: 0,
            rupee_count: 0,
            rupee_adjust: 0,
            rupee_sound_set: 0,
            rupee_sound_on: 0,
            key_shown: 0,
            key_adjust: 0,
            key_light_frame: [0; 5],
            star_frame: [0; 8],
            compass_state: COMPASS_CLOSED,
            compass_map_flag: 0,
            compass_sound_flag: 0,
            compass_time_pass: 0.0,
            compass_day_light: 0,
            map_zoom: 0xFF,
            map_control: MapCtrl::default(),
            map_stat: MapStat::default(),
            map_view: MapView::default(),
            ammo_type: 0,
            ammo_old_type: 0,
            ammo_glow: [Emitter::default(); 2],
            rupee_glow: Emitter::default(),
            dir_timer: 0,
            item_move_flag: 0,
            item_move_button: 0,
            item_move_slot: 0,
            item_move_timer: 0,
            item_parts: Pane::default(),
            item_icon: Pane::default(),
            swim_state: 0,
            swim_tekari_frame: 0,
            swim_bubble_wait: 0,
            swim_base_black: Color::CLEAR,
            swim_base_white: Color::OPAQUE,
            swim_full_black: Color::CLEAR,
            swim_full_white: Color::OPAQUE,
            swim_mask_black: Color::CLEAR,
            swim_low: Color::CLEAR,
            swim_blink_steps: 0,
            swim_bubble_rise: [0.0; SWIM_BUBBLE_COUNT],
            swim_bubble_sway: [0.0; SWIM_BUBBLE_COUNT],
            kilari: Quad2D::default(),
            clock: [Quad2D::default(); 3],
            place_name: false,
            button_icon_mode: 0,
            metronome_armed: 0,
            wind_id: NO_PROCESS,
            wind_status: 0,
            auction_flag: 0,
            recollection_copied: false,
            sounds: Vec::new(),
            sound_enabled: true,
        }
    }
}

impl Widget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn play(&mut self, sound: u32) {
        if self.sound_enabled {
            self.sounds.push(sound);
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Input {
    pub time: f32,
    pub time_pass: bool,
    pub wallet: u8,
    pub rupee: i32,
    pub key_num: i32,
    pub pending_rupee: i32,
    pub pending_key: i32,
    pub item_now_rupee: i32,
    pub air: AirMeter,
    pub keep_type: u8,
    pub status_flags: u32,
    pub event_hind: u32,
    pub events: EventFlags,
    pub mini_game: i32,
    pub menu_open: bool,
    pub menu_timer: i32,
    pub menu_status: u8,
    pub item_mode: u8,
    pub collect_mode: u8,
    pub auction_flag: u8,
    pub sub_window_flag: u8,
    pub menu_button: u8,
    pub texture_rate_x: f32,
    pub texture_rate_y: f32,
    pub texture_scale_x: f32,
    pub texture_scale_y: f32,
    pub key_display: bool,
    pub save_table: i32,
    pub stage_type: u32,
    pub dungeon_item_compass: bool,
    pub stage_boss_enemy: bool,
    pub map_display_allowed: bool,
    pub map_alpha: u8,
    pub map_view: MapView,
    pub overlay_running: bool,
    pub stay_room: i32,
    pub camera_manual: bool,
    pub camera_attention: bool,
    pub camera_body_mode: i32,
    pub subject_accept: bool,
    pub event_running: bool,
    pub metronome: bool,
    pub bow_aim: bool,
    pub ship_angle: i16,
    pub trig_left: bool,
    pub trig_right: bool,
    pub trig_up: bool,
    pub trig_down: bool,
    pub advance_direction: u8,
    pub scissor_y: f32,
    pub scissor_height: f32,
    pub place_name_state: u8,
    pub start_stage: String,
    pub next_stage: String,
    pub equip_slot: Vec<u8>,
    pub equip_reserve: Vec<u8>,
    pub equip_bait: Vec<u8>,
    pub equip_beast: Vec<u8>,
    pub seed: [i32; 3],
    pub day_hour: i32,
    pub day_minute: i32,
    pub day_night: i32,
    pub day_of_week: i32,
    pub moon: i32,
    pub button_icon_mode: u8,
    pub request_place_name: bool,
    pub request_menu_window: bool,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            time: 0.0,
            time_pass: false,
            wallet: 0,
            rupee: 0,
            key_num: 0,
            pending_rupee: 0,
            pending_key: 0,
            item_now_rupee: 0,
            air: AirMeter::default(),
            keep_type: 0,
            status_flags: 0,
            event_hind: 0,
            events: EventFlags::default(),
            mini_game: -1,
            menu_open: false,
            menu_timer: 0,
            menu_status: 0,
            item_mode: 0,
            collect_mode: 0,
            auction_flag: 0,
            sub_window_flag: 0,
            menu_button: 0,
            texture_rate_x: 0.0,
            texture_rate_y: 0.0,
            texture_scale_x: 0.0,
            texture_scale_y: 0.0,
            key_display: false,
            save_table: 0,
            stage_type: 0,
            dungeon_item_compass: false,
            stage_boss_enemy: false,
            map_display_allowed: true,
            map_alpha: 0,
            map_view: MapView::default(),
            overlay_running: false,
            stay_room: 0,
            camera_manual: false,
            camera_attention: false,
            camera_body_mode: 0,
            subject_accept: false,
            event_running: false,
            metronome: false,
            bow_aim: false,
            ship_angle: 0,
            trig_left: false,
            trig_right: false,
            trig_up: false,
            trig_down: false,
            advance_direction: 0,
            scissor_y: 0.0,
            scissor_height: 480.0,
            place_name_state: 0,
            start_stage: String::new(),
            next_stage: String::new(),
            equip_slot: Vec::new(),
            equip_reserve: Vec::new(),
            equip_bait: Vec::new(),
            equip_beast: Vec::new(),
            seed: [100; 3],
            day_hour: 0,
            day_minute: 0,
            day_night: 0,
            day_of_week: 0,
            moon: 0,
            button_icon_mode: 0,
            request_place_name: false,
            request_menu_window: false,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawOut {
    pub quads: Vec<Quad>,
    pub clock: [Quad2D; 3],
    pub kilari: Quad2D,
    pub glow: Emitter,
    pub ammo_glow: [Emitter; 2],
    pub item_quad: Option<Quad>,
    pub layers: Vec<Layer>,
    pub sounds: Vec<u32>,
    pub result: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    MeterBase,
    Clock(usize),
    Kilari,
    MeterTop,
    Metronome,
}

pub trait Peers {
    fn status_check(&mut self, widget: &mut Widget, input: &mut Input);
    fn alpha_control(&mut self, widget: &mut Widget, input: &mut Input);
    fn heart_move(&mut self, widget: &mut Widget, input: &mut Input);
    fn action_move(&mut self, widget: &mut Widget, input: &mut Input);
    fn wepen_move(&mut self, widget: &mut Widget, input: &mut Input);
    fn item_row_move(&mut self, widget: &mut Widget, input: &mut Input);
    fn gauge_move(&mut self, widget: &mut Widget, input: &mut Input);
    fn enemy_move(&mut self, widget: &mut Widget, input: &mut Input);
    fn magic_move(&mut self, widget: &mut Widget, input: &mut Input);
    fn wind_move(&mut self, widget: &mut Widget, input: &mut Input);
    fn metronome_move(&mut self, widget: &mut Widget, input: &mut Input);
    fn menu_move(&mut self, widget: &mut Widget, input: &mut Input);
    fn gauge_alpha(&mut self, widget: &mut Widget, input: &mut Input);
    fn heart_draw(&mut self, widget: &mut Widget);
    fn wepen_draw(&mut self, widget: &mut Widget);
    fn action_draw(&mut self, widget: &mut Widget);
    fn item_row_draw(&mut self, widget: &mut Widget);
    fn gauge_draw(&mut self, widget: &mut Widget);
}

pub struct MeterPeers<'a> {
    pub vitals: &'a mut Vitals,
    pub vitals_env: &'a mut VitalsEnv,
    pub gauges: &'a mut MeterGauges,
    pub gauge_env: &'a mut GaugeEnv,
    pub gauge_out: &'a mut GaugeOutput,
    pub gauge_tuning: &'a MeterTuning,
    pub procs: &'a mut MsgProcs,
}

impl MeterPeers<'_> {
    fn share(&mut self, widget: &Widget, input: &Input) {
        let MeterPeers {
            vitals,
            vitals_env,
            gauge_env,
            ..
        } = self;
        gauge_env.status_flags = widget.status_flags;
        gauge_env.button_icon_mode = widget.button_icon_mode;
        gauge_env.menu_open = input.menu_open;
        gauge_env.menu_status = input.menu_status;
        gauge_env.push_menu_button = input.menu_button;
        gauge_env.collect_mode = input.collect_mode;
        gauge_env.auction_flag = input.auction_flag != 0;
        gauge_env.event_running = input.event_running;
        gauge_env.event_hind = input.event_hind as u16;
        gauge_env.overlap_request = input.overlay_running;
        gauge_env.metronome = input.metronome;
        gauge_env.bow_aim = input.bow_aim;
        gauge_env.start_stage.clear();
        gauge_env.start_stage.push_str(&input.start_stage);
        gauge_env.next_stage.clear();
        gauge_env.next_stage.push_str(&input.next_stage);
        gauge_env.save_table = input.save_table;
        gauge_env.stage_boss = vitals_env.stage_type == vitals::stage::TYPE_BOSS;
        gauge_env.stage_boss_enemy = input.stage_boss_enemy;
        gauge_env.collect_boss_gauge = boss_gauge_collect(&vitals_env.collect);
        gauge_env.up_button = vitals_env.up_button as u8;
        gauge_env.picture_status = vitals_env.picture_status;
        gauge_env.mini_game_type = vitals_env.mini_game_type;
        gauge_env.operate_wind = vitals_env.operate_wind;
        gauge_env.player_status1 = vitals_env.player_status1;
        gauge_env.r_status = vitals_env.r_status;
        gauge_env.r_status_force = vitals_env.r_status_force;
        gauge_env.max_life = i32::from(vitals_env.max_life);
        gauge_env.heart_alpha = vitals.heart_alpha;
    }

    fn play(&mut self, widget: &mut Widget) {
        for sound in self.gauge_out.sounds.drain(..) {
            widget.play(sound);
        }
    }
}

impl Peers for MeterPeers<'_> {
    fn status_check(&mut self, widget: &mut Widget, _input: &mut Input) {
        self.vitals.fade_bits = 0;
        self.vitals.globals.sounds.clear();
        self.gauges.icon.action_mode = 0;
        self.gauge_out.reset();
        self.vitals.status_check(self.vitals_env);
        widget.status_flags = self.vitals.status_flags;
    }

    fn alpha_control(&mut self, _widget: &mut Widget, _input: &mut Input) {
        self.vitals.alpha_control(self.vitals_env);
    }

    fn heart_move(&mut self, _widget: &mut Widget, _input: &mut Input) {
        self.vitals.heart_move(self.vitals_env);
    }

    fn action_move(&mut self, widget: &mut Widget, _input: &mut Input) {
        self.vitals.action_move(self.vitals_env);
        widget.button_icon_mode |= self.vitals.fade_bits;
        self.vitals.fade_bits = 0;
    }

    fn wepen_move(&mut self, widget: &mut Widget, _input: &mut Input) {
        self.vitals.wepon_move(self.vitals_env);
        widget.button_icon_mode |= self.vitals.fade_bits;
        self.vitals.fade_bits = 0;
    }

    fn item_row_move(&mut self, widget: &mut Widget, _input: &mut Input) {
        self.vitals.xy_move(self.vitals_env);
        widget.button_icon_mode |= self.vitals.fade_bits;
        self.vitals.fade_bits = 0;
    }

    fn gauge_move(&mut self, widget: &mut Widget, input: &mut Input) {
        self.share(widget, input);
        gauges::icon_move(self.gauges, self.gauge_env, self.gauge_tuning);
        widget.button_icon_mode |= self.gauges.icon.action_mode;
        self.gauges.icon.action_mode = 0;
    }

    fn enemy_move(&mut self, widget: &mut Widget, input: &mut Input) {
        self.vitals.pane_scale_transform();
        self.share(widget, input);
        gauges::enemy_move(self.gauges, self.gauge_env, self.gauge_tuning);
    }

    fn magic_move(&mut self, widget: &mut Widget, input: &mut Input) {
        self.share(widget, input);
        gauges::magic_move(self.gauges, self.gauge_env, self.gauge_out);
        self.play(widget);
    }

    fn wind_move(&mut self, widget: &mut Widget, input: &mut Input) {
        self.share(widget, input);
        gauges::wind_move(self.gauges, self.gauge_env, self.gauge_out, self.procs);
        self.play(widget);
    }

    fn metronome_move(&mut self, widget: &mut Widget, input: &mut Input) {
        self.share(widget, input);
        gauges::metronome_move(self.gauges, self.gauge_env);
    }

    fn menu_move(&mut self, widget: &mut Widget, input: &mut Input) {
        self.share(widget, input);
        gauges::menu_move(self.gauges, self.gauge_env, self.gauge_tuning);
        input.map_alpha = self.gauge_env.map_alpha as u8;
        self.gauge_out.map_alpha = self.gauge_env.map_alpha;
        self.gauge_out.button_action_mode = widget.button_icon_mode;
    }

    fn gauge_alpha(&mut self, widget: &mut Widget, input: &mut Input) {
        self.share(widget, input);
        gauges::gauge_alpha(self.gauges, self.gauge_env);
    }

    fn heart_draw(&mut self, _widget: &mut Widget) {
        self.vitals.heart_draw();
    }

    fn wepen_draw(&mut self, _widget: &mut Widget) {
        self.vitals.wepon_draw();
    }

    fn action_draw(&mut self, _widget: &mut Widget) {
        self.vitals.action_draw();
    }

    fn item_row_draw(&mut self, _widget: &mut Widget) {
        self.vitals.xy_draw();
    }

    fn gauge_draw(&mut self, _widget: &mut Widget) {
        gauges::icon_draw(self.gauges);
    }
}

pub fn value_increase(max: i32, value: i32, mode: u8) -> f32 {
    if max <= 0 {
        return 1.0;
    }
    let value = value.clamp(0, max);
    let ratio = value as f32 / max as f32;
    match mode {
        0 => ratio * ratio,
        1 => ratio.sqrt(),
        3 => (ratio * 2.0 - 1.0) * 2.0 - 1.0,
        4 => {
            let wave = (ratio * 0x8000u32 as f32 * 0.5).sin();
            wave * wave
        }
        5 => {
            let wave = (ratio * 0xFFFFu32 as f32 * 0.5).sin();
            wave * wave
        }
        _ => ratio,
    }
}

pub fn alpha_close(timer: &mut i16, value: &mut i16) -> f32 {
    if *timer == 8 {
        *timer = 9;
        *value = 1;
        1.0
    } else if *timer == 9 {
        if *value < ALPHA_OPEN_STEPS {
            let alpha =
                1.0 - value_increase(ALPHA_OPEN_STEPS as i32, i32::from(*value), ALPHA_ROOT);
            *value += 1;
            alpha
        } else {
            *timer = 0;
            0.0
        }
    } else {
        0.0
    }
}

pub fn alpha_open(timer: &mut i16, value: &mut i16) -> f32 {
    if *timer == 0 {
        *timer = 7;
        *value = 0;
        0.0
    } else if *timer == 7 {
        if *value < ALPHA_OPEN_STEPS {
            let alpha = value_increase(ALPHA_OPEN_STEPS as i32, i32::from(*value), ALPHA_ROOT);
            *value += 1;
            alpha
        } else {
            *timer = 8;
            1.0
        }
    } else {
        1.0
    }
}

pub fn add_calc_angle_s(
    value: &mut i16,
    target: i16,
    scale: i16,
    max_step: i16,
    min_step: i16,
) -> i16 {
    let mut diff = target.wrapping_sub(*value);
    if *value != target {
        let mut step = diff / scale;
        if step > min_step || step < -min_step {
            step = step.clamp(-max_step, max_step);
            *value = value.wrapping_add(step);
        } else if diff >= 0 {
            *value = value.wrapping_add(min_step);
            diff = target.wrapping_sub(*value);
            if diff <= 0 {
                *value = target;
            }
        } else {
            *value = value.wrapping_sub(min_step);
            diff = target.wrapping_sub(*value);
            if diff >= 0 {
                *value = target;
            }
        }
    }
    target.wrapping_sub(*value)
}

pub fn rand(seed: &mut [i32; 3]) -> f32 {
    const MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
    const MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
    const SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
    for lane in 0..3 {
        seed[lane] = (seed[lane] * MUL[lane]) % MOD[lane];
    }
    let sum =
        (seed[0] as f32) / SCALE[0] + (seed[1] as f32) / SCALE[1] + (seed[2] as f32) / SCALE[2];
    (sum % 1.0).abs()
}

pub fn rand_f(seed: &mut [i32; 3], range: f32) -> f32 {
    rand(seed) * range
}

pub fn rand_fx(seed: &mut [i32; 3], range: f32) -> f32 {
    range * (rand(seed) - 0.5) * 2.0
}

pub fn digit_texture(digit: i32) -> String {
    format!("rupy_num_{digit:02}.bti")
}

pub fn parent_pane_trans(pane: &mut Pane, tuning: &Tuning, scale: f32) {
    pane.pos_center.x += f32::from(tuning.row_offset_x);
    pane.pos_center.y += f32::from(tuning.row_offset_y);
    pane.pane_scale_xy(scale);
    let angle = pane.angle();
    pane.rotate(pane.size.x / 2.0, pane.size.y / 2.0, angle);
}

pub fn child_pane_trans(pane: &mut Pane, tuning: &Tuning, parent: &Pane, scale: f32) {
    let offset_x = pane.pos_center_orig.x - parent.pos_center_orig.x;
    let offset_y = pane.pos_center_orig.y - parent.pos_center_orig.y;
    pane.pos_center.x += offset_x * (scale - 1.0);
    pane.pos_center.y += offset_y * (scale - 1.0);
    pane.pos_center.x += f32::from(tuning.row_offset_x);
    pane.pos_center.y += f32::from(tuning.row_offset_y);
    pane.pane_scale_xy(scale);
    let angle = pane.angle();
    pane.rotate(pane.size.x / 2.0, pane.size.y / 2.0, angle);
}

pub fn child_pane_trans_only(
    pane: &mut Pane,
    tuning: &Tuning,
    parent: &Pane,
    trans_scale: f32,
    scale: f32,
) {
    let offset_x = pane.pos_center_orig.x - parent.pos_center_orig.x;
    let offset_y = pane.pos_center_orig.y - parent.pos_center_orig.y;
    pane.pos_center.x += offset_x * (trans_scale - 1.0);
    pane.pos_center.y += offset_y * (trans_scale - 1.0);
    pane.pos_center.x += f32::from(tuning.row_offset_x);
    pane.pos_center.y += f32::from(tuning.row_offset_y);
    pane.pane_scale_xy(scale);
    let angle = pane.angle();
    pane.rotate(pane.size.x / 2.0, pane.size.y / 2.0, angle);
}

pub fn child_pane_trans_child_trans(
    pane: &mut Pane,
    tuning: &Tuning,
    parent: &Pane,
    child: &Pane,
    trans_scale: f32,
    scale: f32,
) {
    let offset_x = pane.pos_center_orig.x - parent.pos_center_orig.x;
    let offset_y = pane.pos_center_orig.y - parent.pos_center_orig.y;
    pane.pos_center.x += offset_x * (trans_scale - 1.0);
    pane.pos_center.y += offset_y * (trans_scale - 1.0);
    let offset_x = pane.pos_center.x - child.pos_center.x;
    let offset_y = pane.pos_center.y - child.pos_center.y;
    pane.pos_center.x += offset_x * (scale - 1.0);
    pane.pos_center.y += offset_y * (scale - 1.0);
    pane.pos_center.x += f32::from(tuning.row_offset_x);
    pane.pos_center.y += f32::from(tuning.row_offset_y);
    pane.pane_scale_xy(scale);
    let angle = pane.angle();
    pane.rotate(pane.size.x / 2.0, pane.size.y / 2.0, angle);
}

pub fn child_pane_trans_child_trans_only(
    pane: &mut Pane,
    tuning: &Tuning,
    parent: &Pane,
    child: &Pane,
    trans_scale: f32,
    scale: f32,
) {
    let offset_x = pane.pos_center_orig.x - parent.pos_center_orig.x;
    let offset_y = pane.pos_center_orig.y - parent.pos_center_orig.y;
    pane.pos_center.x += offset_x * (trans_scale - 1.0);
    pane.pos_center.y += offset_y * (trans_scale - 1.0);
    pane.pos_center.x += f32::from(tuning.row_offset_x);
    pane.pos_center.y += f32::from(tuning.row_offset_y);
    pane.cpos_move();
    let angle = pane.angle();
    pane.rotate(pane.size.x / 2.0, pane.size.y / 2.0, angle);
}

pub fn pane_bottom_scale_y(pane: &mut Pane, scale: f32) {
    let bottom = pane.pos_top_left.y + pane.size.y;
    pane.size.y = pane.size_orig.y * scale;
    pane.pos_center.y = bottom - (pane.size.y / 2.0);
    pane.cpos_move();
}

pub fn rotate_z(pane: &mut Pane, angle_a: f32, angle_b: f32) {
    let x = pane.size.x * 0.5;
    let y = pane.size.y * 0.5;
    let angle = angle_a + angle_b;
    pane.rotate(x, y, angle);
}

pub fn rupee_alpha(widget: &mut Widget, input: &mut Input) {
    let flags = input.status_flags;
    let hide = flags & ST_SCREEN_OFF != 0
        || (flags & ST_EVENT != 0 && input.event_hind & 0x80 != 0)
        || flags & ST_CONDUCT != 0
        || flags & ST_SCOPE != 0
        || flags & ST_PLAYER_HELD != 0
        || flags & ST_WIND_PICK != 0
        || flags & ST_PICTURE != 0
        || flags & ST_MENU_ITEM != 0
        || flags & ST_MENU_COLLECT != 0
        || flags & ST_MENU_NAME != 0
        || flags & ST_MENU_MAP != 0
        || input.mini_game == 1
        || input.mini_game == 6;
    let alpha = if hide {
        let alpha = alpha_close(
            &mut widget.pane.rupee_back.user_area,
            &mut widget.pane.rupee_digit_1[0].user_area,
        );
        if widget.rupee_glow.alive {
            widget.rupee_glow.stop();
        } else {
            let x = widget.pane.rupee_back.pos_center.x - SCREEN_CENTER_X;
            let y = widget.pane.rupee_back.pos_center.y - SCREEN_CENTER_Y;
            widget.rupee_glow = Emitter::spawn(RUPEE_GLOW_TEXTURE, x, y);
        }
        alpha
    } else {
        let alpha = alpha_open(
            &mut widget.pane.rupee_back.user_area,
            &mut widget.pane.rupee_digit_1[0].user_area,
        );
        if widget.rupee_glow.alive {
            widget.rupee_glow.play();
        } else {
            let x = widget.pane.rupee_back.pos_center.x - SCREEN_CENTER_X;
            let y = widget.pane.rupee_back.pos_center.y - SCREEN_CENTER_Y;
            widget.rupee_glow = Emitter::spawn(RUPEE_GLOW_TEXTURE, x, y);
        }
        alpha
    };
    for index in 0..DIGIT_COUNT {
        widget.pane.rupee_digit_1[index].set_now_alpha(alpha);
        widget.pane.rupee_digit_2[index].set_now_alpha(alpha);
    }
    widget.pane.rupee_back.set_now_alpha(alpha);
    widget.pane.rupee_light.set_now_alpha(alpha);
}

pub fn rupee_init(widget: &mut Widget, input: &mut Input) {
    widget.pane.rupee_back.user_area = 0;
    widget.pane.rupee_digit_1[0].user_area = 0;
    widget.pane.rupee_digit_1[0].user_area = 1;
    widget.rupee_count = input.rupee as u16;
    let count = i32::from(widget.rupee_count);
    let digits = [
        count / 1000,
        (count % 1000) / 100,
        (count % 100) / 10,
        count % 10,
    ];
    for index in 0..DIGIT_COUNT {
        let name = digit_texture(digits[index]);
        widget.pane.rupee_digit_1[index].change_texture(&name);
        widget.pane.rupee_digit_2[index].change_texture(&name);
    }
    input.item_now_rupee = count;
    let x = widget.pane.rupee_back.pos_center.x - SCREEN_CENTER_X;
    let y = widget.pane.rupee_back.pos_center.y - SCREEN_CENTER_Y;
    widget.rupee_glow = Emitter::spawn(RUPEE_GLOW_TEXTURE, x, y);
}

pub fn rupee_move(widget: &mut Widget, input: &mut Input) {
    if widget.pane.rupee_digit_1[1].user_area != i16::from(input.wallet) {
        wallet_change(widget, input);
        widget.pane.rupee_digit_1[1].user_area = i16::from(input.wallet);
    }
    if input.pending_rupee != 0 {
        let mut target = input.rupee + input.pending_rupee;
        let max = match input.wallet {
            WALLET_SMALL => RUPEE_MAX_SMALL,
            WALLET_MEDIUM => RUPEE_MAX_MEDIUM,
            _ => RUPEE_MAX_LARGE,
        };
        if target > max {
            target = max;
        } else if target < 0 {
            target = 0;
        }
        widget.rupee_adjust = target - i32::from(widget.rupee_count);
        input.rupee = target;
        input.pending_rupee = 0;
        if widget.rupee_adjust.abs() >= RUPEE_SOUND_STEP {
            widget.rupee_sound_on = 1;
        }
    }
    if widget.rupee_adjust != 0 {
        if widget.rupee_adjust > 0 {
            widget.rupee_count = widget.rupee_count.wrapping_add(1);
            widget.rupee_adjust -= 1;
            if widget.rupee_sound_on != 0 {
                if widget.rupee_adjust != 0 {
                    if widget.rupee_sound_set == 0 {
                        widget.play(SE_RUPEE_UP_1);
                        widget.rupee_sound_set = 1;
                    } else {
                        widget.rupee_sound_set = 0;
                    }
                } else {
                    widget.play(SE_RUPEE_UP_2);
                    widget.rupee_sound_set = 0;
                    widget.rupee_sound_on = 0;
                }
            }
        } else {
            widget.rupee_count = widget.rupee_count.wrapping_sub(1);
            widget.rupee_adjust += 1;
            if widget.rupee_sound_on != 0 {
                if widget.rupee_adjust != 0 {
                    if widget.rupee_sound_set == 0 {
                        widget.play(SE_RUPEE_DOWN_1);
                        widget.rupee_sound_set = 1;
                    } else {
                        widget.rupee_sound_set = 0;
                    }
                } else {
                    widget.play(SE_RUPEE_DOWN_2);
                    widget.rupee_sound_set = 0;
                    widget.rupee_sound_on = 0;
                }
            }
        }
        let count = i32::from(widget.rupee_count);
        let digits = [
            count / 1000,
            (count % 1000) / 100,
            (count % 100) / 10,
            count % 10,
        ];
        for index in 0..DIGIT_COUNT {
            let name = digit_texture(digits[index]);
            widget.pane.rupee_digit_1[index].change_texture(&name);
            widget.pane.rupee_digit_2[index].change_texture(&name);
        }
        input.item_now_rupee = count;
    }
    rupee_alpha(widget, input);
}

pub fn wallet_change(widget: &mut Widget, input: &Input) {
    if input.wallet == WALLET_SMALL {
        let shift = widget.pane.rupee_digit_1[1].pos_top_left_orig.x
            - widget.pane.rupee_digit_1[0].pos_top_left_orig.x;
        widget.pane.rupee_digit_1[0].hide();
        widget.pane.rupee_digit_2[0].hide();
        widget.pane.rupee_back.set_black(Color::unpack(0x0A28_0A00));
        widget.pane.rupee_back.set_white(Color::unpack(0x28FF_28FF));
        widget.pane.rupee_back.pane_trans(shift, 0.0);
        widget.pane.rupee_light.pane_trans(shift, 0.0);
    } else if input.wallet == WALLET_MEDIUM {
        widget.pane.rupee_digit_1[0].show();
        widget.pane.rupee_digit_2[0].show();
        widget.pane.rupee_back.set_black(Color::unpack(0x1414_A000));
        widget.pane.rupee_back.set_white(Color::unpack(0x32C8_FFFF));
        widget.pane.rupee_back.pane_trans(0.0, 0.0);
        widget.pane.rupee_light.pane_trans(0.0, 0.0);
    } else {
        widget.pane.rupee_digit_1[0].show();
        widget.pane.rupee_digit_2[0].show();
        widget.pane.rupee_back.set_black(Color::unpack(0x5028_0000));
        widget.pane.rupee_back.set_white(Color::unpack(0xFF8C_00FF));
        widget.pane.rupee_back.pane_trans(0.0, 0.0);
        widget.pane.rupee_light.pane_trans(0.0, 0.0);
    }
}

pub fn key_light(pane: &mut Pane, frames: &[i16; 5], master: f32) {
    let step = pane.user_area;
    let (scale, light) = if step < frames[0] {
        (0.0, 0.0)
    } else if step < frames[1] {
        let ratio = value_increase(
            (frames[1] - frames[0]) as i32,
            (step - frames[0]) as i32,
            ALPHA_SQUARE,
        );
        ((ratio * 0.8) + 0.2, ratio * 170.0)
    } else if step < frames[2] {
        let ratio = value_increase(
            (frames[2] - frames[1]) as i32,
            (step - frames[1]) as i32,
            ALPHA_SQUARE,
        );
        (1.0 - (ratio * 0.2), 170.0 - (ratio * 90.0))
    } else if step < frames[3] {
        let ratio = value_increase(
            (frames[3] - frames[2]) as i32,
            (step - frames[2]) as i32,
            ALPHA_SQUARE,
        );
        ((ratio * 0.2) + 0.8, (ratio * 90.0) + 80.0)
    } else {
        let ratio = value_increase(
            (frames[4] - frames[3]) as i32,
            (step - frames[3]) as i32,
            ALPHA_SQUARE,
        );
        (1.0, 170.0 - (ratio * 170.0))
    };
    if step < frames[4] {
        pane.user_area = step + 1;
    } else {
        pane.user_area = 1;
    }
    let width = pane.size_orig.x * scale;
    let height = pane.size_orig.y * scale;
    let x = pane.pos_top_left_orig.x - (width - pane.size_orig.x) / 2.0;
    let y = pane.pos_top_left_orig.y - (height - pane.size_orig.y) / 2.0;
    pane.move_to(x, y);
    pane.resize(width, height);
    let angle = pane.angle();
    pane.rotate(width / 2.0, height / 2.0, angle);
    pane.now_alpha = (light * master) as u8;
}

pub fn key_rnd_light_frame(widget: &mut Widget, input: &mut Input) {
    let base = widget.tuning.star_base;
    let span = widget.tuning.star_span;
    let jitter = widget.tuning.star_jitter;
    widget.key_light_frame[0] = base + rand_fx(&mut input.seed, f32::from(jitter[0])) as i16;
    widget.key_light_frame[1] = (widget.key_light_frame[0] + span[0])
        + rand_fx(&mut input.seed, f32::from(jitter[1])) as i16;
    widget.key_light_frame[2] = (widget.key_light_frame[1] + span[1])
        + rand_fx(&mut input.seed, f32::from(jitter[2])) as i16;
    widget.key_light_frame[3] = (widget.key_light_frame[2] + span[2])
        + rand_fx(&mut input.seed, f32::from(jitter[3])) as i16;
    widget.key_light_frame[4] = (widget.key_light_frame[3] + span[3])
        + rand_fx(&mut input.seed, f32::from(jitter[4])) as i16;
}

pub fn key_alpha(widget: &mut Widget, input: &mut Input) {
    let flags = input.status_flags;
    let hide = flags & ST_SCREEN_OFF != 0
        || (flags & ST_EVENT != 0 && input.event_hind & 0x40 != 0)
        || flags & ST_CONDUCT != 0
        || flags & ST_SCOPE != 0
        || flags & ST_PLAYER_HELD != 0
        || flags & ST_WIND_PICK != 0
        || flags & ST_PICTURE != 0
        || flags & ST_SHIP_RIDE != 0
        || flags & ST_MENU_ITEM != 0
        || flags & ST_MENU_COLLECT != 0
        || flags & ST_MENU_NAME != 0
        || flags & ST_MENU_MAP != 0;
    let alpha = if hide {
        alpha_close(
            &mut widget.pane.key_frame.user_area,
            &mut widget.pane.key_digit_1[0].user_area,
        )
    } else if input.key_display {
        alpha_open(
            &mut widget.pane.key_frame.user_area,
            &mut widget.pane.key_digit_1[0].user_area,
        )
    } else {
        alpha_close(
            &mut widget.pane.key_frame.user_area,
            &mut widget.pane.key_digit_1[0].user_area,
        )
    };
    for index in 0..SLOT_COUNT_2 {
        widget.pane.key_digit_1[index].set_now_alpha(alpha);
        widget.pane.key_digit_2[index].set_now_alpha(alpha);
    }
    widget.pane.key_frame.set_now_alpha(alpha);
    if widget.pane.key_back_2.user_area == 1 {
        key_rnd_light_frame(widget, input);
    }
    key_light(&mut widget.pane.key_back_1, &KEY_LIGHT_FRAMES, alpha);
    let frames = widget.key_light_frame;
    key_light(&mut widget.pane.key_back_2, &frames, alpha);
}

pub fn key_init(widget: &mut Widget, input: &Input) {
    widget.pane.key_back_2.user_area = 1;
    widget.pane.key_back_1.user_area = 1;
    widget.pane.key_frame.user_area = 0;
    widget.pane.key_digit_1[0].user_area = 0;
    widget.key_shown = input.key_num as u8;
    let digits = [
        i32::from(widget.key_shown) / 10,
        i32::from(widget.key_shown) % 10,
    ];
    for index in 0..SLOT_COUNT_2 {
        let name = digit_texture(digits[index]);
        widget.pane.key_digit_1[index].blend_init(&name);
        widget.pane.key_digit_2[index].blend_init(&name);
    }
}

pub fn key_move(widget: &mut Widget, input: &mut Input) {
    if input.key_display {
        if input.pending_key != 0 {
            let mut target = input.key_num + input.pending_key;
            if target > KEY_MAX {
                target = KEY_MAX;
            } else if target < 0 {
                target = 0;
            }
            widget.key_adjust = (target - i32::from(widget.key_shown)) as i16;
            input.key_num = target;
            input.pending_key = 0;
        }
        let adjust = widget.key_adjust;
        if adjust != 0 {
            if adjust > 0 {
                widget.key_shown = widget.key_shown.wrapping_add(1);
                widget.key_adjust -= 1;
            } else if adjust < 0 {
                widget.key_shown = widget.key_shown.wrapping_sub(1);
                widget.key_adjust += 1;
            }
            let digits = [
                i32::from(widget.key_shown) / 10,
                i32::from(widget.key_shown) % 10,
            ];
            for index in 0..SLOT_COUNT_2 {
                let name = digit_texture(digits[index]);
                widget.pane.key_digit_1[index].blend_draw(&name);
                widget.pane.key_digit_2[index].blend_draw(&name);
            }
        }
    }
    key_alpha(widget, input);
}

pub fn compass_rotate(pane: &mut Pane, _second: &Pane, angle: f32) {
    let offset_x = pane.width() / 2.0;
    let offset_y = pane.height() / 2.0;
    pane.rotate(offset_x, offset_y, angle);
}

pub fn compass_get_on_proc(widget: &mut Widget, input: &mut Input) {
    match widget.compass_state {
        COMPASS_CLOSED => {
            if widget.map_control.shown != 0
                && (input.status_flags & ST_SHIP_RIDE == 0 || input.save_table != SAVE_TABLE_TOTG)
            {
                widget.compass_state = COMPASS_DIR_OPENING;
                widget.compass_map_flag = 1;
            } else {
                widget.compass_state = COMPASS_DIR_OPENED;
                widget.pane.compass_base.show();
                clock_show(widget, input);
                compass_dir_open(widget);
                widget.compass_map_flag = 0;
            }
        }
        COMPASS_DIR_OPENING => {
            if widget.map_control.shown == 0 {
                widget.pane.compass_base.show();
                clock_show(widget, input);
                widget.compass_state = COMPASS_DIR_OPENED;
                compass_dir_open(widget);
                widget.compass_sound_flag = 1;
                widget.play(SE_MAP_SHOW);
            }
        }
        COMPASS_DIR_OPENED => {
            compass_dir_open(widget);
        }
        COMPASS_DIR_OPEN => {
            if (input.trig_right && !input.trig_up && !input.trig_down)
                || widget.pane.compass_frame.user_area != 0
            {
                widget.compass_state = COMPASS_WIND_OPEN;
                widget.pane.compass_frame.user_area = 1;
                compass_wind_open(widget);
                widget.compass_sound_flag = 1;
                widget.play(SE_MAP_SHOW);
            }
        }
        COMPASS_WIND_OPEN => {
            if input.trig_left && !input.trig_up && !input.trig_down {
                widget.compass_state = COMPASS_WIND_CLOSING;
                widget.pane.compass_frame.user_area = 0;
                compass_wind_close(widget, input);
            } else {
                compass_wind_open(widget);
            }
        }
        COMPASS_WIND_CLOSING => {
            if input.trig_right && !input.trig_up && !input.trig_down {
                widget.compass_state = COMPASS_WIND_OPEN;
                widget.pane.compass_frame.user_area = 1;
                compass_wind_open(widget);
            } else {
                compass_wind_close(widget, input);
            }
        }
        COMPASS_CLOSING => {
            compass_dir_close(widget, input);
        }
        COMPASS_WIND_OPENING => {
            compass_anime_move(widget, input);
        }
        _ => {}
    }
    let heading = (i32::from(input.ship_angle) + 0x8000) as i16;
    let heading = -(heading as f32) * DEG_180 / HEX_8000;
    compass_rotate(
        &mut widget.pane.clock_needle,
        &widget.pane.clock_needle_back,
        -input.time - 90.0,
    );
    compass_rotate(
        &mut widget.pane.compass_needle_mag,
        &widget.pane.compass_base,
        heading,
    );
    compass_rotate(
        &mut widget.pane.compass_needle,
        &widget.pane.compass_base,
        heading,
    );
    clock_move(widget, input);
}

pub fn compass_get_off_proc(widget: &mut Widget, input: &mut Input) {
    match widget.compass_state {
        COMPASS_DIR_OPEN | COMPASS_DIR_OPENED => {
            widget.compass_state = COMPASS_CLOSING;
            compass_dir_close(widget, input);
        }
        COMPASS_CLOSING => {
            compass_dir_close(widget, input);
        }
        COMPASS_WIND_OPEN | COMPASS_WIND_OPENING => {
            widget.compass_state = COMPASS_WIND_CLOSING;
            widget.pane.compass_frame.user_area = 0;
            compass_wind_close(widget, input);
        }
        COMPASS_WIND_CLOSING => {
            compass_wind_close(widget, input);
        }
        _ => {
            compass_value_init(widget, input);
        }
    }
}

pub fn compass_dir_open(widget: &mut Widget) {
    let half = widget.pane.clock_ring_2.width() / 2.0;
    let needle_offset_x = widget.pane.compass_frame.width() / 2.0 - half;
    let needle_offset_y = widget.pane.compass_frame.height() / 2.0;
    let back_offset_x = widget.pane.clock_needle_back.width() / 2.0 - half;
    let back_offset_y = widget.pane.clock_needle_back.height() / 2.0;
    widget.pane.compass_base.user_area += 1;
    let shift;
    if widget.pane.compass_base.user_area >= 5 {
        widget.pane.compass_base.user_area = 5;
        widget.compass_state = COMPASS_DIR_OPEN;
        shift = 0.0;
        if widget.compass_sound_flag != 0 {
            widget.compass_sound_flag = 0;
            widget.play(SE_MAP_SHOW_END);
        }
    } else {
        let ratio = value_increase(
            5,
            5 - widget.pane.compass_base.user_area as i32,
            ALPHA_SQUARE,
        );
        shift = ratio
            * -(widget.pane.compass_frame.pos_top_left_orig.x
                + widget.pane.compass_frame.size_orig.x);
        if widget.compass_sound_flag == 0 {
            widget.compass_sound_flag = 1;
            widget.play(SE_COMPASS_MOVE);
        }
    }
    widget.pane.compass_base.pane_trans(shift, 0.0);
    widget.pane.clock_needle_back.pane_trans(shift, 0.0);
    widget.pane.compass_frame.pane_trans(shift, 0.0);
    widget
        .pane
        .clock_needle_back
        .rotate(back_offset_x, back_offset_y, 180.0);
    widget
        .pane
        .compass_frame
        .rotate(needle_offset_x, needle_offset_y, 180.0);
}

pub fn compass_wind_open(widget: &mut Widget) {
    let half = widget.pane.clock_ring_2.width() / 2.0;
    let needle_offset_x = widget.pane.compass_frame.width() / 2.0 - half;
    let needle_offset_y = widget.pane.compass_frame.height() / 2.0;
    let back_offset_x = widget.pane.clock_needle_back.width() / 2.0 - half;
    let back_offset_y = widget.pane.clock_needle_back.height() / 2.0;
    widget.pane.compass_base.user_area += 1;
    let angle;
    if widget.pane.compass_base.user_area >= 10 {
        widget.pane.compass_base.user_area = 10;
        widget.compass_state = COMPASS_WIND_OPENING;
        angle = 0.0;
        if widget.compass_sound_flag != 0 {
            widget.compass_sound_flag = 0;
            widget.play(SE_MAP_SHOW_END);
        }
    } else {
        let ratio = value_increase(
            5,
            10 - widget.pane.compass_base.user_area as i32,
            ALPHA_SQUARE,
        );
        angle = ratio * -180.0;
        if widget.compass_sound_flag == 0 {
            widget.compass_sound_flag = 1;
            widget.play(SE_COMPASS_MOVE);
        }
    }
    widget
        .pane
        .clock_needle_back
        .rotate(back_offset_x, back_offset_y, angle);
    widget
        .pane
        .compass_frame
        .rotate(needle_offset_x, needle_offset_y, angle);
}

pub fn compass_wind_close(widget: &mut Widget, input: &mut Input) {
    let half = widget.pane.clock_ring_2.width() / 2.0;
    let needle_offset_x = widget.pane.compass_frame.width() / 2.0 - half;
    let needle_offset_y = widget.pane.compass_frame.height() / 2.0;
    let back_offset_x = widget.pane.clock_needle_back.width() / 2.0 - half;
    let back_offset_y = widget.pane.clock_needle_back.height() / 2.0;
    widget.pane.compass_base.user_area -= 1;
    let day = if input.time >= DAY_WINDOW_FROM && input.time < DAY_WINDOW_TO {
        0
    } else {
        1
    };
    let angle;
    if widget.pane.compass_base.user_area <= 5 {
        widget.pane.compass_base.user_area = 5;
        if (input.time_pass && widget.compass_time_pass == 0.0)
            || (!input.time_pass && widget.compass_time_pass != 0.0)
            || (!input.time_pass && widget.compass_day_light != day)
        {
            clock_show(widget, input);
            widget.compass_state = COMPASS_WIND_OPEN;
            widget.pane.compass_frame.user_area = 1;
        } else {
            widget.compass_state = COMPASS_DIR_OPEN;
        }
        angle = -180.0;
        if widget.compass_sound_flag != 0 {
            widget.compass_sound_flag = 0;
            widget.play(SE_MAP_SHOW_END);
        }
    } else {
        let ratio = value_increase(
            5,
            10 - widget.pane.compass_base.user_area as i32,
            ALPHA_SQUARE,
        );
        angle = ratio * -180.0;
        if widget.compass_sound_flag == 0 {
            widget.compass_sound_flag = 1;
            widget.play(SE_COMPASS_MOVE);
        }
    }
    widget
        .pane
        .clock_needle_back
        .rotate(back_offset_x, back_offset_y, angle);
    widget
        .pane
        .compass_frame
        .rotate(needle_offset_x, needle_offset_y, angle);
}

pub fn compass_dir_close(widget: &mut Widget, input: &mut Input) {
    let half = widget.pane.clock_ring_2.width() / 2.0;
    let needle_offset_x = widget.pane.compass_frame.width() / 2.0 - half;
    let needle_offset_y = widget.pane.compass_frame.height() / 2.0;
    let back_offset_x = widget.pane.clock_needle_back.width() / 2.0 - half;
    let back_offset_y = widget.pane.clock_needle_back.height() / 2.0;
    widget.pane.compass_base.user_area -= 1;
    let shift;
    if widget.pane.compass_base.user_area <= 0 {
        compass_value_init(widget, input);
        widget.pane.compass_base.user_area = 0;
        shift = -(widget.pane.compass_frame.pos_top_left_orig.x
            + widget.pane.compass_frame.size_orig.x);
        if widget.compass_sound_flag != 0 {
            widget.compass_sound_flag = 0;
            widget.play(SE_MAP_SHOW_END);
        }
    } else {
        let ratio = value_increase(
            5,
            5 - widget.pane.compass_base.user_area as i32,
            ALPHA_SQUARE,
        );
        shift = ratio
            * -(widget.pane.compass_frame.pos_top_left_orig.x
                + widget.pane.compass_frame.size_orig.x);
        if widget.compass_sound_flag == 0 {
            widget.compass_sound_flag = 1;
            widget.play(SE_COMPASS_MOVE);
        }
    }
    widget.pane.compass_base.pane_trans(shift, 0.0);
    widget.pane.clock_needle_back.pane_trans(shift, 0.0);
    widget.pane.compass_frame.pane_trans(shift, 0.0);
    widget
        .pane
        .clock_needle_back
        .rotate(back_offset_x, back_offset_y, 180.0);
    widget
        .pane
        .compass_frame
        .rotate(needle_offset_x, needle_offset_y, 180.0);
}

pub fn compass_anime_move(widget: &mut Widget, input: &mut Input) {
    let day = if input.time >= DAY_WINDOW_FROM && input.time < DAY_WINDOW_TO {
        0
    } else {
        1
    };
    if (!input.trig_left || input.trig_up || input.trig_down)
        && (!input.time_pass || widget.compass_time_pass == 0.0)
        && (input.time_pass || widget.compass_time_pass == 0.0)
    {
        if input.time_pass {
            return;
        }
        if widget.compass_day_light == day {
            return;
        }
    }
    widget.compass_state = COMPASS_WIND_CLOSING;
    widget.pane.compass_frame.user_area = 0;
    widget.compass_sound_flag = 1;
    widget.play(SE_MAP_CANCEL);
}

pub fn compass_value_init(widget: &mut Widget, input: &mut Input) {
    widget.compass_time_pass = if input.time_pass { 1.0 } else { 0.0 };
    widget.compass_day_light = if input.time >= DAY_WINDOW_FROM && input.time < DAY_WINDOW_TO {
        0
    } else {
        1
    };
    widget.pane.compass_base.user_area = 0;
    widget.compass_sound_flag = 0;
    let half = widget.pane.clock_ring_2.width() / 2.0;
    let needle_offset_x = widget.pane.compass_frame.width() / 2.0 - half;
    let needle_offset_y = widget.pane.compass_frame.height() / 2.0;
    let back_offset_x = widget.pane.clock_needle_back.width() / 2.0;
    let back_offset_y = widget.pane.clock_needle_back.height() / 2.0;
    let shift =
        -(widget.pane.compass_frame.pos_top_left_orig.x + widget.pane.compass_frame.size_orig.x);
    widget.pane.compass_base.pane_trans(shift, 0.0);
    widget.pane.compass_frame.pane_trans(shift, 0.0);
    widget
        .pane
        .clock_needle_back
        .rotate(back_offset_x, back_offset_y, 180.0);
    widget
        .pane
        .compass_frame
        .rotate(needle_offset_x, needle_offset_y, 180.0);
    widget.pane.compass_face.hide();
    widget.pane.compass_base.hide();
    clock_hide(widget);
    widget.compass_state = COMPASS_CLOSED;
}

pub fn compass_alpha(widget: &mut Widget) {
    let alpha = widget.wake_alpha;
    widget.pane.compass_face.set_now_alpha(alpha);
    widget.pane.compass_base.set_now_alpha(alpha);
    widget.pane.clock_bar.set_now_alpha(alpha);
    widget.pane.clock_ring_2.set_now_alpha(alpha);
    widget.pane.clock_horizon_1.set_now_alpha(alpha);
    widget.pane.clock_horizon_2.set_now_alpha(alpha);
    for index in 0..3 {
        widget.pane.clock_star_day[index].set_now_alpha(alpha);
        widget.pane.clock_cloud[0].set_now_alpha(alpha);
    }
    widget.pane.clock_moon_2.set_now_alpha(alpha);
    widget.pane.clock_sun_2.set_now_alpha(alpha);
    for index in 0..4 {
        widget.pane.clock_circle[index].set_now_alpha(alpha);
        widget.pane.clock_star_night[0].set_now_alpha(alpha);
    }
    widget.pane.clock_needle.set_now_alpha(alpha);
    widget.pane.clock_needle_back.set_now_alpha(alpha);
    widget.pane.clock_ring_1.set_now_alpha(alpha);
    widget.pane.clock_moon_1.set_now_alpha(alpha);
    widget.pane.clock_clip.set_now_alpha(alpha);
    widget.pane.clock_sun_1.set_now_alpha(alpha);
    widget.pane.compass_frame.set_now_alpha(alpha);
}

pub fn compass_init(widget: &mut Widget, input: &mut Input) {
    widget
        .pane
        .compass_needle_mag
        .set_mirror(true, F2::new(-1.0, 0.0), F2::new(1.0, 1.0));
    widget
        .pane
        .compass_needle
        .set_mirror(true, F2::new(-1.0, 0.0), F2::new(1.0, 1.0));
    widget.compass_map_flag = u8::from(widget.map_control.shown != 0);
    widget.pane.compass_frame.user_area = 0;
    compass_value_init(widget, input);
    clock_multi_init(widget);
}

pub fn compass_move(widget: &mut Widget, input: &mut Input) {
    let flags = input.status_flags;
    if flags & ST_SCREEN_OFF != 0
        || (flags & ST_EVENT != 0 && input.event_hind & 0x100 != 0)
        || flags & ST_CONDUCT != 0
    {
        compass_value_init(widget, input);
    } else if flags & ST_MESSAGE != 0
        || flags & ST_SCOPE != 0
        || flags & ST_PLAYER_HELD != 0
        || flags & ST_PICTURE != 0
        || flags & ST_MENU_ITEM != 0
        || flags & ST_MENU_COLLECT != 0
        || flags & (ST_MENU_NAME | ST_MENU_MAP) != 0
        || input.mini_game == 1
        || input.mini_game == MINI_GAME_BOW
    {
        compass_value_init(widget, input);
    } else if flags & ST_SHIP_RIDE != 0 && input.save_table != SAVE_TABLE_TOTG {
        compass_get_on_proc(widget, input);
        compass_alpha(widget);
        widget.map_view.mode = u8::from(widget.map_zoom != 0);
        input.map_view = widget.map_view;
    } else {
        compass_get_off_proc(widget, input);
        compass_alpha(widget);
    }
    clock_multi_move(widget);
}

pub fn clock_show(widget: &mut Widget, input: &mut Input) {
    widget.compass_time_pass = if input.time_pass { 1.0 } else { 0.0 };
    widget.compass_day_light = if input.time >= DAY_WINDOW_FROM && input.time < DAY_WINDOW_TO {
        0
    } else {
        1
    };
    clock_init(widget);
    if input.time_pass {
        widget.pane.clock_bar.show();
        widget.pane.clock_ring_2.show();
        widget.pane.clock_horizon_1.show();
        widget.pane.clock_horizon_2.show();
        for index in 0..3 {
            widget.pane.clock_star_day[index].show();
        }
        widget.pane.clock_moon_2.show();
        widget.pane.clock_sun_2.show();
        for index in 0..4 {
            widget.pane.clock_circle[index].show();
        }
        widget.pane.clock_needle.show();
        widget.pane.clock_needle_back.show();
        widget.pane.clock_ring_1.hide();
        widget.pane.clock_moon_1.hide();
        widget.pane.clock_clip.hide();
        widget.pane.clock_sun_1.hide();
        widget.pane.compass_frame.hide();
        for index in 0..3 {
            widget.pane.clock_cloud[index].hide();
        }
        for index in 0..4 {
            widget.pane.clock_star_night[index].hide();
        }
    } else {
        widget.pane.clock_bar.hide();
        widget.pane.clock_ring_2.hide();
        widget.pane.clock_horizon_1.hide();
        widget.pane.clock_horizon_2.hide();
        for index in 0..3 {
            widget.pane.clock_star_day[index].hide();
        }
        widget.pane.clock_moon_2.hide();
        widget.pane.clock_sun_2.hide();
        for index in 0..4 {
            widget.pane.clock_circle[index].hide();
        }
        widget.pane.clock_needle.hide();
        widget.pane.clock_needle_back.hide();
        if input.time >= DAY_WINDOW_FROM && input.time < DAY_WINDOW_TO {
            widget.pane.clock_ring_1.show();
            widget.pane.clock_moon_1.hide();
            widget.pane.clock_clip.show();
            widget.pane.clock_sun_1.show();
            widget.pane.compass_frame.show();
            for index in 0..3 {
                widget.pane.clock_cloud[index].hide();
            }
            for index in 0..4 {
                widget.pane.clock_star_night[index].hide();
            }
            widget.pane.compass_frame.set_corner_color(
                Color::unpack(0xFFFF_00FF),
                Color::unpack(0xFFFF_00FF),
                Color::unpack(0xFFFF_00FF),
                Color::unpack(0x0000_FFFF),
            );
        } else {
            widget.pane.clock_ring_1.show();
            widget.pane.clock_moon_1.show();
            widget.pane.clock_clip.hide();
            widget.pane.clock_sun_1.hide();
            widget.pane.compass_frame.show();
            for index in 0..3 {
                widget.pane.clock_cloud[index].hide();
            }
            for index in 0..4 {
                widget.pane.clock_star_night[index].show();
            }
            widget.pane.compass_frame.set_corner_color(
                Color::unpack(0x0000_FFFF),
                Color::unpack(0x0000_00FF),
                Color::unpack(0x0000_00FF),
                Color::unpack(0x0000_00FF),
            );
        }
    }
}

pub fn clock_hide(widget: &mut Widget) {
    widget.pane.clock_bar.hide();
    widget.pane.clock_ring_2.hide();
    widget.pane.clock_horizon_1.hide();
    widget.pane.clock_horizon_2.hide();
    for index in 0..3 {
        widget.pane.clock_star_day[index].hide();
        widget.pane.clock_cloud[index].hide();
    }
    widget.pane.clock_moon_2.hide();
    widget.pane.clock_sun_2.hide();
    for index in 0..4 {
        widget.pane.clock_circle[index].hide();
        widget.pane.clock_star_night[index].hide();
    }
    widget.pane.clock_needle.hide();
    widget.pane.clock_needle_back.hide();
    widget.pane.clock_ring_1.hide();
    widget.pane.clock_moon_1.hide();
    widget.pane.clock_clip.hide();
    widget.pane.clock_sun_1.hide();
    widget.pane.compass_frame.hide();
}

pub fn clock_init(widget: &mut Widget) {
    let x =
        widget.pane.compass_frame.pos_top_left_orig.x + widget.pane.clock_clip.pos_top_left_orig.x;
    let y =
        widget.pane.compass_frame.pos_top_left_orig.y + widget.pane.clock_clip.pos_top_left_orig.y;
    for index in 0..3 {
        widget.pane.clock_star_night[index].user_area = 1;
        widget.pane.clock_star_day[index].user_area = 1;
        widget.pane.clock_cloud[index].pos_center.x = x - f32::from(CLOCK_INIT_X[index]);
        widget.pane.clock_cloud[index].pos_center.y = y - f32::from(CLOCK_INIT_Y[index]);
    }
    widget.pane.clock_star_night[3].user_area = 1;
    widget.pane.clock_sun_1.user_area = 0;
}

pub fn clock_move(widget: &mut Widget, input: &mut Input) {
    let step = widget.pane.clock_sun_1.user_area;
    if step <= 0x3c {
        value_increase(0x3c, step as i32, ALPHA_SQUARE);
    } else {
        value_increase(0x3c, 0x78 - step as i32, ALPHA_SQUARE);
    }
    if widget.pane.clock_sun_1.user_area < 0x78 {
        widget.pane.clock_sun_1.user_area += 1;
    } else {
        widget.pane.clock_sun_1.user_area = 0;
    }
    for index in 0..3 {
        widget.pane.clock_cloud[index].pos_center.x =
            f32::from(widget.tuning.clock_scroll_x[index]);
        widget.pane.clock_cloud[index].pos_center.y =
            f32::from(widget.tuning.clock_scroll_y[index]);
    }
    if input.time_pass {
        if widget.pane.clock_star_day[0].user_area == 1 {
            star_light_frame(widget, input, 0);
        }
        if widget.pane.clock_star_day[1].user_area == 1 {
            star_light_frame(widget, input, 1);
        }
        if widget.pane.clock_star_day[2].user_area == 1 {
            star_light_frame(widget, input, 2);
        }
        clock_star_light(
            &mut widget.pane.clock_star_day[0],
            widget.star_frame[0],
            widget.star_frame[1],
        );
        clock_star_light(
            &mut widget.pane.clock_star_day[1],
            widget.star_frame[1],
            widget.star_frame[2],
        );
        clock_star_light(
            &mut widget.pane.clock_star_day[2],
            widget.star_frame[2],
            widget.star_frame[3],
        );
    } else {
        if widget.pane.clock_star_night[0].user_area == 1 {
            star_light_frame(widget, input, 0);
        }
        if widget.pane.clock_star_night[1].user_area == 1 {
            star_light_frame(widget, input, 1);
        }
        if widget.pane.clock_star_night[2].user_area == 1 {
            star_light_frame(widget, input, 2);
        }
        if widget.pane.clock_star_night[3].user_area == 1 {
            star_light_frame(widget, input, 3);
        }
        clock_star_light(
            &mut widget.pane.clock_star_night[0],
            widget.star_frame[0],
            widget.star_frame[1],
        );
        clock_star_light(
            &mut widget.pane.clock_star_night[1],
            widget.star_frame[1],
            widget.star_frame[2],
        );
        clock_star_light(
            &mut widget.pane.clock_star_night[2],
            widget.star_frame[2],
            widget.star_frame[3],
        );
        clock_star_light(
            &mut widget.pane.clock_star_night[3],
            widget.star_frame[3],
            widget.star_frame[4],
        );
    }
}

pub fn clock_star_light(pane: &mut Pane, frame_from: i16, frame_to: i16) {
    let step = pane.user_area;
    let ratio = if step < frame_from {
        value_increase(frame_from as i32, step as i32, ALPHA_SQUARE)
    } else {
        value_increase(
            (frame_to - frame_from) as i32,
            (frame_to - step) as i32,
            ALPHA_SQUARE,
        )
    };
    let level = (ratio * 255.0) as u8;
    pane.set_black(Color::rgba(level, level, level, 0x00));
    if step < frame_to {
        pane.user_area = step + 1;
    } else {
        pane.user_area = 1;
    }
}

pub fn star_light_frame(widget: &mut Widget, input: &mut Input, slot: usize) {
    let base = widget.tuning.star_base;
    let span = widget.tuning.star_span;
    let jitter = widget.tuning.star_jitter;
    widget.star_frame[slot] = base + rand_fx(&mut input.seed, f32::from(jitter[0])) as i16;
    widget.star_frame[slot + 1] =
        (widget.star_frame[slot] + span[0]) + rand_fx(&mut input.seed, f32::from(jitter[1])) as i16;
}

pub fn clock_multi_init(widget: &mut Widget) {
    for index in 0..3 {
        widget.clock[index] = Quad2D::init(
            TEXTURE_MASK,
            TEXTURE_SCREW,
            1.0 / CLOCK_SCALE_X[index],
            1.0 / CLOCK_SCALE_Y[index],
        );
    }
}

pub fn clock_multi_move(widget: &mut Widget) {
    for index in 0..3 {
        let x0 = (widget.pane.compass_frame.pos_top_left_orig.x
            + widget.pane.clock_clip.pos_top_left_orig.x) as i16;
        let y0 = (widget.pane.compass_frame.pos_top_left_orig.y
            + widget.pane.clock_clip.pos_top_left_orig.y) as i16;
        let scroll_x = widget.pane.clock_cloud[index].pos_center.x as i16;
        let scroll_y = widget.pane.clock_cloud[index].pos_center.y as i16;
        let x1 = f32::from(x0) + widget.pane.clock_clip.size_orig.x;
        let y1 = f32::from(y0) + widget.pane.clock_clip.size_orig.y;
        widget.clock[index].set_pos(x0, y0, x1 as i16, y1 as i16);
        widget.clock[index].black = Color::rgba(0xFF, 0xFF, 0xFF, 0xFF);
        widget.clock[index].white = Color::rgba(0x00, 0x00, 0xFF, 0x00);
        widget.clock[index].set_scroll(SCROLL_SLOT0, 0, 0);
        widget.clock[index].set_scroll(SCROLL_SLOT1, scroll_x, scroll_y);
        let steps = f32::from(widget.tuning.clock_steps[index]);
        widget.clock[index].set_scale(
            (100.0 * (1.0 / CLOCK_SCALE_X[index])) / steps,
            (100.0 * (1.0 / CLOCK_SCALE_Y[index])) / steps,
        );
    }
}

pub fn map_init(widget: &mut Widget, input: &mut Input) {
    widget.map_zoom = 0xFF;
    widget.pane.dungeon_item[0].user_area = i16::from(input.dungeon_item_compass);
    widget.map_control.shown = 0;
    widget.map_control.slide = MAP_SLIDE_HOME.wrapping_sub(MAP_LEFT_UP_X);
    map_ctrl_init(widget, input);
}

pub fn map_move(widget: &mut Widget, input: &mut Input) {
    if !widget.map_stat.stay_valid {
        widget.map_stat.stay_room = input.stay_room;
        widget.map_stat.stay_valid = true;
    }
    if !input.overlay_running {
        if widget.map_zoom == 0xFF {
            widget.map_stat.stage_flag = 1;
        } else if widget.map_stat.stay_room != input.stay_room {
            widget.map_stat.stage_flag = 2;
            widget.map_stat.stay_room = input.stay_room;
        }
    }
    let flags = input.status_flags;
    if flags & ST_SCREEN_OFF == 0
        && (flags & ST_EVENT == 0 || input.event_hind & 0x100 == 0)
        && flags & ST_CONDUCT == 0
        && flags & ST_MESSAGE == 0
        && flags & ST_SCOPE == 0
        && flags & ST_PLAYER_HELD == 0
        && flags & ST_PICTURE == 0
        && flags & ST_MENU_ITEM == 0
        && flags & ST_MENU_COLLECT == 0
        && flags & ST_MENU_MAP == 0
        && input.map_display_allowed
        && (input.next_stage != NEXT_STAGE_MAJYU_E || input.map_alpha == 0)
    {
        if flags & ST_SHIP_RIDE != 0 && input.save_table != SAVE_TABLE_TOTG {
            if widget.map_control.shown != 0 && input.map_display_allowed {
                widget.map_control.shown = 0;
                widget.map_control.flags = 7;
            }
        } else {
            if !input.trig_up
                && !input.trig_down
                && input.mini_game != 1
                && (input.trig_left || input.trig_right)
            {
                if input.trig_left {
                    if input.map_alpha != 0 && widget.map_control.shown != 0 {
                        if input.map_display_allowed {
                            widget.map_control.shown = 0;
                            widget.map_control.flags = 0;
                        }
                        widget.compass_map_flag = 0;
                        widget.play(SE_MAP_CANCEL);
                    }
                } else if input.trig_right && input.map_alpha != 0 {
                    if widget.map_control.shown == 0 {
                        if input.map_display_allowed {
                            widget.map_control.shown = 1;
                            widget.map_control.flags = 0;
                        }
                        widget.compass_map_flag = 1;
                        widget.play(SE_MAP_SHOW);
                    } else if widget.map_zoom != 0 {
                        widget.map_zoom = 0;
                        widget.play(SE_MAP_ZOOM);
                    } else {
                        widget.map_zoom = 1;
                        widget.play(SE_MAP_ZOOM);
                    }
                }
            } else if widget.compass_map_flag != 0
                && widget.compass_state == COMPASS_CLOSED
                && widget.map_control.shown == 0
                && input.map_display_allowed
            {
                widget.map_control.shown = 1;
                widget.map_control.flags = 7;
            }
            if widget.map_stat.stage_flag == 1 {
                widget.map_stat.stage_flag = 0;
                if widget.map_control.shown == 0 {
                    if input.map_display_allowed {
                        widget.map_control.shown = 1;
                        widget.map_control.flags = 0;
                    }
                    widget.compass_map_flag = 1;
                    widget.map_zoom = 0;
                }
            } else if widget.map_stat.stage_flag == 2 {
                widget.map_stat.stage_flag = 0;
                widget.map_zoom = 0;
            }
            widget.map_view.mode = u8::from(widget.map_zoom != 0);
            if widget.map_control.shown != 0 && widget.map_stat.timer != 6 {
                widget.map_stat.timer = 6;
            }
        }
    }
    map_ctrl_move(widget, input);
    recollect_boss_data(widget, input);
}

pub fn recollect_boss_data(widget: &mut Widget, input: &mut Input) {
    if input.stage_type != STAGE_TYPE_BOSS || !input.stage_boss_enemy {
        return;
    }
    for (table, bit) in RECOLLECT_TABLE {
        if input.save_table == table && !input.events.test(bit) {
            widget.recollection_copied = true;
            input.events.set(bit);
            return;
        }
    }
}

pub fn ammo_init(widget: &mut Widget, input: &Input) {
    let kind = input.keep_type;
    let index = if kind > 3 { 0 } else { kind as usize };
    for slot in 0..SLOT_COUNT_2 {
        widget.pane.ammo_frame[slot].change_texture(ARROW_TEXTURES[index]);
        widget.pane.ammo_frame[slot].set_now_alpha_zero();
        widget.pane.ammo_glow[slot].set_now_alpha_zero();
        widget.pane.ammo_frame[slot].set_alpha();
        widget.pane.ammo_glow[slot].set_alpha();
        widget.ammo_glow[slot] = Emitter::default();
    }
    widget.ammo_type = kind;
    widget.ammo_old_type = kind;
}

pub fn ammo_move(widget: &mut Widget, input: &mut Input) {
    let shift =
        widget.pane.ammo_glow[0].pos_center_orig.x - widget.pane.ammo_glow[1].pos_center_orig.x;
    let flags = input.status_flags;
    let aiming = input.bow_aim
        && flags & ST_SCREEN_OFF == 0
        && flags & ST_EVENT == 0
        && flags & ST_CONDUCT == 0
        && flags & ST_SCOPE == 0
        && flags & ST_PLAYER_HELD == 0
        && flags & ST_PICTURE == 0
        && flags & ST_MENU_ITEM == 0
        && flags & ST_MENU_COLLECT == 0
        && flags & ST_MENU_NAME == 0
        && flags & ST_MENU_MAP == 0
        && flags & ST_MESSAGE == 0
        && input.mini_game != MINI_GAME_BOW;
    if aiming {
        if widget.ammo_type != input.keep_type {
            widget.pane.ammo_frame[0].change_texture(ARROW_TEXTURES[widget.ammo_type as usize]);
            widget.ammo_old_type = widget.ammo_type;
            widget.ammo_type = input.keep_type;
            widget.pane.ammo_frame[1].change_texture(ARROW_TEXTURES[widget.ammo_type as usize]);
            widget.pane.ammo_glow[0].user_area = 0;
            if widget.pane.ammo_glow[1].user_area < 2 {
                widget.pane.ammo_glow[1].user_area += 1;
            }
            for slot in 0..SLOT_COUNT_2 {
                widget.ammo_glow[slot].kill();
            }
        }
        if widget.ammo_old_type != 0 {
            if !widget.ammo_glow[0].alive {
                let x = widget.pane.ammo_glow[0].pos_center.x - SCREEN_CENTER_X;
                let y = widget.pane.ammo_glow[0].pos_center.y - SCREEN_CENTER_Y;
                let name = AMMO_GLOW_TEXTURES[(widget.ammo_old_type - 1) as usize];
                widget.ammo_glow[0] = Emitter::spawn(name, x, y);
            }
        } else if widget.ammo_glow[0].alive {
            widget.ammo_glow[0].kill();
        }
        if widget.ammo_type != 0 {
            if !widget.ammo_glow[1].alive {
                let x = widget.pane.ammo_glow[0].pos_center.x - SCREEN_CENTER_X;
                let y = widget.pane.ammo_glow[1].pos_center.y - SCREEN_CENTER_Y;
                let name = AMMO_GLOW_TEXTURES[(widget.ammo_type - 1) as usize];
                widget.ammo_glow[1] = Emitter::spawn(name, x, y);
            }
        } else if widget.ammo_glow[1].alive {
            widget.ammo_glow[1].kill();
        }
        if widget.pane.ammo_glow[1].user_area == 0 {
            widget.pane.ammo_glow[1].user_area = 1;
        }
        if widget.pane.ammo_glow[0].user_area < 10 {
            widget.pane.ammo_glow[0].user_area += 1;
            if widget.pane.ammo_glow[0].user_area == 10 {
                widget.pane.ammo_frame[0].set_now_alpha_zero();
                widget.pane.ammo_glow[0].set_now_alpha_zero();
                widget.pane.ammo_frame[1].set_init_alpha();
                widget.pane.ammo_glow[1].set_init_alpha();
                for slot in 0..SLOT_COUNT_2 {
                    widget.pane.ammo_frame[slot].pane_trans(shift, 0.0);
                    widget.pane.ammo_glow[slot].pane_trans(shift, 0.0);
                }
            } else {
                let ratio =
                    value_increase(10, widget.pane.ammo_glow[0].user_area as i32, ALPHA_SQUARE);
                let offset = shift * ratio;
                for slot in 0..SLOT_COUNT_2 {
                    widget.pane.ammo_frame[slot].pane_trans(offset, 0.0);
                    widget.pane.ammo_glow[slot].pane_trans(offset, 0.0);
                }
                widget.pane.ammo_frame[1].set_now_alpha(ratio);
                widget.pane.ammo_glow[1].set_now_alpha(ratio);
                if widget.pane.ammo_glow[1].user_area == 2 {
                    let fade = value_increase(
                        10,
                        (10 - widget.pane.ammo_glow[0].user_area) as i32,
                        ALPHA_SQUARE,
                    );
                    widget.pane.ammo_frame[0].set_now_alpha(fade);
                    widget.pane.ammo_glow[0].set_now_alpha(fade);
                } else {
                    widget.pane.ammo_frame[0].set_now_alpha_zero();
                    widget.pane.ammo_glow[0].set_now_alpha_zero();
                }
            }
        }
        for slot in 0..SLOT_COUNT_2 {
            widget.pane.ammo_frame[slot].set_alpha();
            widget.pane.ammo_glow[slot].set_alpha();
        }
    } else {
        if widget.pane.ammo_glow[1].user_area != 0 {
            widget.pane.ammo_glow[1].user_area = 0;
        }
        if widget.pane.ammo_glow[0].user_area > 0 {
            widget.pane.ammo_glow[0].user_area -= 1;
            if widget.pane.ammo_glow[0].user_area == 0 {
                for slot in 0..SLOT_COUNT_2 {
                    widget.pane.ammo_frame[slot].set_now_alpha_zero();
                    widget.pane.ammo_glow[slot].set_now_alpha_zero();
                    widget.pane.ammo_frame[slot].set_alpha();
                    widget.pane.ammo_glow[slot].set_alpha();
                    widget.ammo_glow[slot].kill();
                }
            } else {
                let ratio =
                    value_increase(10, widget.pane.ammo_glow[0].user_area as i32, ALPHA_SQUARE);
                for slot in 0..SLOT_COUNT_2 {
                    let alpha = (f32::from(widget.pane.ammo_frame[slot].now_alpha) * ratio) as u8;
                    widget.pane.ammo_frame[slot].set_alpha_value(alpha);
                    let alpha = (f32::from(widget.pane.ammo_glow[slot].now_alpha) * ratio) as u8;
                    widget.pane.ammo_glow[slot].set_alpha_value(alpha);
                }
            }
        }
    }
    for slot in 0..SLOT_COUNT_2 {
        if widget.ammo_glow[slot].alive {
            widget.ammo_glow[slot].play();
            widget.ammo_glow[slot].pos = F2::new(
                widget.pane.ammo_glow[1].pos_center.x - SCREEN_CENTER_X,
                widget.pane.ammo_glow[1].pos_center.y - SCREEN_CENTER_Y,
            );
            widget.ammo_glow[slot].alpha = f32::from(widget.pane.ammo_frame[slot].now_alpha);
        }
    }
}

pub fn direction_init(widget: &mut Widget) {
    for index in 0..4 {
        widget.pane.direction_arrow[index].user_area = 0;
    }
    widget.dir_timer = 0;
}

pub fn direction_check_status(widget: &mut Widget, input: &Input) {
    for index in 0..4 {
        if input.advance_direction & (1 << index) != 0 {
            if widget.pane.direction_arrow[index].user_area == 0
                && (widget.dir_timer == 0 || widget.dir_timer == DIR_HALF)
            {
                widget.pane.direction_arrow[index].user_area = DIR_SHOW;
            }
        } else {
            direction_erase(&mut widget.pane.direction_arrow[index]);
        }
    }
}

pub fn direction_trans_scale(pane: &mut Pane, shift: f32, scale: f32, index: usize) {
    match index {
        0 => {
            let right = pane.pos_top_left_orig.x + pane.size_orig.x;
            let top = pane.pos_top_left_orig.y;
            pane.size.x = pane.size_orig.x * scale;
            pane.size.y = pane.size_orig.y;
            pane.pos_top_left.x = right - shift - pane.size.x;
            pane.pos_top_left.y = top;
        }
        1 => {
            let left = pane.pos_top_left_orig.x;
            let top = pane.pos_top_left_orig.y;
            pane.size.x = pane.size_orig.x;
            pane.size.y = pane.size_orig.y * scale;
            pane.pos_top_left.x = left;
            pane.pos_top_left.y = top + shift;
        }
        2 => {
            let left = pane.pos_top_left_orig.x;
            let top = pane.pos_top_left_orig.y;
            pane.size.x = pane.size_orig.x * scale;
            pane.size.y = pane.size_orig.y;
            pane.pos_top_left.x = left + shift;
            pane.pos_top_left.y = top;
        }
        3 => {
            let left = pane.pos_top_left_orig.x;
            let bottom = pane.pos_top_left_orig.y + pane.size_orig.y;
            pane.size.x = pane.size_orig.x;
            pane.size.y = pane.size_orig.y * scale;
            pane.pos_top_left.x = left;
            pane.pos_top_left.y = bottom - shift - pane.size.y;
        }
        _ => {}
    }
    pane.move_to(pane.pos_top_left.x, pane.pos_top_left.y);
}

fn dir_channel(table: &[u8; 16], from: usize, channel: usize, ratio: f32) -> u8 {
    let from_value = f32::from(table[from + channel]);
    let to_value = f32::from(table[from + 4 + channel]);
    ((ratio * (to_value - from_value)) + from_value) as u8
}

fn dir_paint(pane: &mut Pane, from: usize, ratio: f32) {
    let black = Color::rgba(
        dir_channel(&DIR_BLACK_TABLE, from, 0, ratio),
        dir_channel(&DIR_BLACK_TABLE, from, 1, ratio),
        dir_channel(&DIR_BLACK_TABLE, from, 2, ratio),
        dir_channel(&DIR_BLACK_TABLE, from, 3, ratio),
    );
    let white = Color::rgba(
        dir_channel(&DIR_WHITE_TABLE, from, 0, ratio),
        dir_channel(&DIR_WHITE_TABLE, from, 1, ratio),
        dir_channel(&DIR_WHITE_TABLE, from, 2, ratio),
        dir_channel(&DIR_WHITE_TABLE, from, 3, ratio),
    );
    pane.set_black(black);
    pane.set_white(white);
}

pub fn direction_anime(widget: &mut Widget) {
    for index in 0..4 {
        if widget.pane.direction_arrow[index].user_area == DIR_SHOW {
            let mut step = widget.dir_timer;
            if step > DIR_HALF {
                step -= DIR_HALF;
            }
            let ratio = value_increase(10, step as i32, ALPHA_SQUARE);
            let shift = (ratio * (DIR_TRANS_TABLE[1] - DIR_TRANS_TABLE[0]) as f32)
                + DIR_TRANS_TABLE[0] as f32;
            let scale = (ratio * (DIR_SCALE_TABLE[1] - DIR_SCALE_TABLE[0])) + DIR_SCALE_TABLE[0];
            direction_trans_scale(&mut widget.pane.direction_arrow[index], shift, scale, index);
            widget.pane.direction_arrow[index].set_now_alpha(ratio * DIR_ALPHA);
            dir_paint(&mut widget.pane.direction_arrow[index], 0, ratio);
            if step == DIR_PEAK {
                widget.pane.direction_arrow[index].user_area = DIR_GROW;
            }
        } else if widget.pane.direction_arrow[index].user_area == DIR_GROW {
            let early = widget.dir_timer <= DIR_HALF;
            let ratio = if early {
                value_increase(10, (widget.dir_timer - DIR_PEAK) as i32, ALPHA_SQUARE)
            } else {
                value_increase(10, (widget.dir_timer - DIR_HALF) as i32, ALPHA_SQUARE)
            };
            let from = if early { 1 } else { 2 };
            let shift = (ratio * (DIR_TRANS_TABLE[from + 1] - DIR_TRANS_TABLE[from]) as f32)
                + DIR_TRANS_TABLE[from] as f32;
            let scale = (ratio * (DIR_SCALE_TABLE[from + 1] - DIR_SCALE_TABLE[from]))
                + DIR_SCALE_TABLE[from];
            direction_trans_scale(&mut widget.pane.direction_arrow[index], shift, scale, index);
            dir_paint(&mut widget.pane.direction_arrow[index], from * 4, ratio);
        }
    }
}

pub fn direction_erase(pane: &mut Pane) {
    if pane.user_area != 0 {
        pane.set_now_alpha_zero();
        pane.pane_trans(0.0, 0.0);
        pane.pane_scale_xy(1.0);
        pane.set_black(Color::unpack(0xFF00_0000));
        pane.set_white(Color::unpack(0xFF00_00FF));
        pane.user_area = 0;
    }
}

pub fn direction_move(widget: &mut Widget, input: &Input) {
    if input.advance_direction != 0 && !input.menu_open {
        direction_check_status(widget, input);
        widget.dir_timer += 1;
        direction_anime(widget);
        if widget.dir_timer < DIR_LOOP {
            return;
        }
        widget.dir_timer = DIR_PEAK;
        return;
    }
    if widget.dir_timer != 0 {
        widget.dir_timer = 0;
        for index in 0..4 {
            direction_erase(&mut widget.pane.direction_arrow[index]);
        }
    }
}

pub fn direction_draw(widget: &mut Widget) {
    for index in 0..4 {
        widget.pane.direction_arrow[index].set_alpha();
    }
}

pub fn item_move_init(widget: &mut Widget) {
    widget.item_icon = Pane::default();
    widget.item_icon.found = true;
    widget.item_icon.change_texture(TEXTURE_BOW);
}

pub fn item_move_set(widget: &mut Widget, pane: &Pane, button: u8, slot: u8) {
    widget.item_move_flag = 1;
    widget.item_move_button = button;
    widget.item_move_slot = slot;
    widget.item_parts.pos_top_left_orig = pane.pos_top_left;
    widget.item_parts.pos_top_left = widget.item_parts.pos_top_left_orig;
    widget.item_parts.pos_center_orig = pane.pos_center;
    widget.item_parts.pos_center = widget.item_parts.pos_center_orig;
    widget.item_parts.size_orig = pane.size_orig;
    widget.item_parts.size = pane.size;
    widget.item_move_timer = 0;
}

pub fn item_move_flag(widget: &Widget) -> u8 {
    widget.item_move_flag
}

pub fn item_move_move(widget: &mut Widget, input: &mut Input) {
    if widget.item_move_timer == 0 {
        let slot = widget.item_move_slot;
        let name = if slot >= SLOT_RESERVE_FIRST {
            input
                .equip_reserve
                .get((slot - SLOT_RESERVE_FIRST) as usize)
                .map_or(TEXTURE_BOW, |item| item_texture(*item))
        } else if slot >= SLOT_BAIT_FIRST {
            input
                .equip_bait
                .get((slot - SLOT_BAIT_FIRST) as usize)
                .map_or(TEXTURE_BOW, |item| item_texture(*item))
        } else if slot >= SLOT_BEAST_FIRST {
            input
                .equip_beast
                .get((slot - SLOT_BEAST_FIRST) as usize)
                .map_or(TEXTURE_BOW, |item| item_texture(*item))
        } else {
            let mut item = input
                .equip_slot
                .get(slot as usize)
                .copied()
                .unwrap_or(ITEM_BOW);
            if item == ITEM_MAGIC_ARROW || item == ITEM_LIGHT_ARROW {
                item = ITEM_BOW;
            }
            item_texture(item)
        };
        widget.item_icon.change_texture(name);
    }
    let origin = widget.item_parts.pos_center_orig;
    let mut dx = 0.0;
    let mut dy = 0.0;
    if widget.item_move_button == 0 {
        dx = widget.pane.item_name[0].pos_center.x - origin.x;
        dy = widget.pane.item_name[0].pos_center.y - origin.y;
    } else if widget.item_move_button == 1 {
        dx = widget.pane.item_name[1].pos_center.x - origin.x;
        dy = widget.pane.item_name[1].pos_center.y - origin.y;
    } else if widget.item_move_button == 2 {
        dx = widget.pane.item_name[2].pos_center.x - origin.x;
        dy = widget.pane.item_name[2].pos_center.y - origin.y;
    }
    widget.item_move_timer += 1;
    let limit = widget.tuning.item_move_frames;
    if widget.item_move_timer <= limit {
        let ratio = value_increase(limit as i32, widget.item_move_timer as i32, ALPHA_LINEAR);
        widget.item_parts.pos_center.x = origin.x + (dx * ratio);
        widget.item_parts.pos_center.y = origin.y + (dy * ratio);
        let scale = widget.tuning.icon_scale;
        let width = widget.item_parts.size_orig.x * scale - widget.pane.item_name[0].size_orig.x;
        let height = widget.item_parts.size_orig.y * scale - widget.pane.item_name[0].size_orig.y;
        match widget.tuning.item_move_mode {
            1 => {
                widget.item_parts.size.x = widget.item_parts.size_orig.x * scale;
                widget.item_parts.size.y = widget.item_parts.size_orig.y * scale;
            }
            2 => {
                widget.item_parts.size.x = widget.item_parts.size_orig.x * scale - (width * ratio);
                widget.item_parts.size.y = widget.item_parts.size_orig.y * scale - (height * ratio);
            }
            _ => {
                widget.item_parts.size.x = widget.item_parts.size_orig.x;
                widget.item_parts.size.y = widget.item_parts.size_orig.y;
            }
        }
        widget.item_parts.pos_top_left.x =
            widget.item_parts.pos_center.x - widget.item_parts.size.x / 2.0;
        widget.item_parts.pos_top_left.y =
            widget.item_parts.pos_center.y - widget.item_parts.size.y / 2.0;
        if widget.item_move_timer == limit - 1 {
            widget.item_move_flag = 2;
        }
    }
}

pub fn item_move_draw(widget: &mut Widget) -> Option<Quad> {
    let limit = widget.tuning.item_move_frames;
    if widget.item_move_flag != 0 && widget.item_move_timer != 0 && widget.item_icon.found {
        widget.item_icon.move_to(
            widget.item_parts.pos_top_left.x,
            widget.item_parts.pos_top_left.y,
        );
        widget
            .item_icon
            .resize(widget.item_parts.size.x, widget.item_parts.size.y);
        let quad = widget.item_icon.quad();
        if widget.item_move_timer >= limit {
            widget.item_move_flag = 0;
        }
        return Some(quad);
    }
    None
}

pub fn swim_trans_y(widget: &mut Widget, offset: f32) {
    for index in 0..SWIM_BUBBLE_COUNT {
        widget.pane.bubble[index].pos_center_orig.y = offset
            + (widget.pane.bubble[index].pos_top_left_orig.y
                + widget.pane.bubble[index].size_orig.y / 2.0);
    }
    widget.pane.gauge_frame.pos_center_orig.y = offset
        + (widget.pane.gauge_frame.pos_top_left_orig.y + widget.pane.gauge_frame.size_orig.y / 2.0);
    widget.pane.gauge_frame.pane_trans(0.0, 0.0);
    widget.pane.gauge_mask.pos_center_orig.y = offset
        + (widget.pane.gauge_mask.pos_top_left_orig.y + widget.pane.gauge_mask.size_orig.y / 2.0);
    widget.pane.gauge_mask.pane_trans(0.0, 0.0);
    widget.pane.gauge_blink.pos_center_orig.y = offset
        + (widget.pane.gauge_blink.pos_top_left_orig.y + widget.pane.gauge_blink.size_orig.y / 2.0);
    widget.pane.gauge_blink.pane_trans(0.0, 0.0);
    widget.pane.gauge_kilari.pos_center_orig.y = offset
        + (widget.pane.gauge_kilari.pos_top_left_orig.y
            + widget.pane.gauge_kilari.size_orig.y / 2.0);
    widget.pane.gauge_kilari.pane_trans(0.0, 0.0);
}

pub fn swim_pane_show(widget: &mut Widget) {
    widget.pane.gauge_soul.set_init_alpha();
    widget.pane.gauge_main.set_init_alpha();
    widget.pane.gauge_blink.set_init_alpha();
    widget.pane.gauge_frame.set_init_alpha();
    widget.pane.gauge_kilari.set_init_alpha();
}

pub fn swim_pane_hide(widget: &mut Widget) {
    for index in 0..SWIM_BUBBLE_COUNT {
        widget.pane.bubble[index].set_now_alpha_zero();
    }
    widget.pane.gauge_frame.set_now_alpha_zero();
    widget.pane.gauge_mask.set_now_alpha_zero();
    widget.pane.gauge_soul.set_now_alpha_zero();
    widget.pane.gauge_blink.set_now_alpha_zero();
    widget.pane.gauge_main.set_now_alpha_zero();
    widget.pane.gauge_kilari.set_now_alpha_zero();
    widget.kilari.black.a = 0;
}

pub fn swim_pane_trans_y(widget: &mut Widget, offset: f32) {
    widget.pane.gauge_blink.pane_trans(0.0, offset);
    widget.pane.gauge_frame.pane_trans(0.0, offset);
    widget.pane.gauge_soul.pane_trans(0.0, 0.0);
    widget.pane.gauge_main.pane_trans(0.0, 0.0);
}

pub fn swim_pane_alpha(widget: &mut Widget, alpha: f32) {
    widget.pane.gauge_soul.set_now_alpha(alpha);
    widget.pane.gauge_main.set_now_alpha(alpha);
    widget.pane.gauge_blink.set_now_alpha(alpha);
    widget.pane.gauge_frame.set_now_alpha(alpha);
}

pub fn swim_init(widget: &mut Widget, input: &Input) {
    let one = widget.timer_screen_data[0].take();
    let two = widget.timer_screen_data[1].take();
    if let (Some(one), Some(two)) = (one.as_ref(), two.as_ref()) {
        screen_data_time_set(widget, one, two);
    }
    widget.timer_screen_data[0] = one;
    widget.timer_screen_data[1] = two;
    swim_pane_hide(widget);
    if !input.key_display {
        swim_trans_y(widget, widget.tuning.swim_offset + 32.0);
    }
    widget.swim_base_black = widget.pane.gauge_frame.black;
    widget.swim_base_white = widget.pane.gauge_frame.white;
    widget.swim_mask_black = widget.pane.gauge_main.black;
    widget.pane.gauge_main.user_area = 0;
    widget.pane.gauge_frame.user_area = 0;
    widget.pane.gauge_kilari.user_area = 0;
    widget.kilari = Quad2D::init(TEXTURE_MASK, TEXTURE_TEKARI, 1.0, 1.0);
}

pub fn swim_move(widget: &mut Widget, input: &mut Input) {
    if input.air.count >= input.air.max / 2 {
        widget.swim_low.r = 200;
        widget.swim_low.g = 200;
        widget.swim_low.b = 0xff;
        widget.swim_full_black = widget.swim_base_black;
        widget.swim_full_white = widget.swim_base_white;
        widget.swim_blink_steps = i32::from(widget.tuning.blink_steps);
    }
    match widget.swim_state {
        0 => {
            if input.air.count != 0 {
                if !widget.timer_screen[0] && !widget.timer_screen[1] {
                    swim_init(widget, input);
                }
                widget.swim_state = 1;
                widget.swim_bubble_wait = 0;
                widget.pane.gauge_frame.user_area = 0;
                widget.pane.gauge_kilari.user_area = 0;
                widget.pane.gauge_soul.user_area = 0;
                widget.pane.gauge_blink.user_area = input.air.count as i16;
                rotate_z(&mut widget.pane.gauge_soul, 0.0, 0.0);
                widget.pane.gauge_frame.set_black(widget.swim_base_black);
                widget.pane.gauge_frame.set_white(widget.swim_base_white);
                widget.pane.gauge_soul.set_black(widget.swim_mask_black);
                widget.pane.gauge_main.set_black(widget.swim_mask_black);
                widget.swim_tekari_frame = 0;
                widget.swim_low.r = 200;
                widget.swim_low.g = 200;
                widget.swim_low.b = 0xff;
                widget.swim_full_black = widget.swim_base_black;
                widget.swim_full_white = widget.swim_base_white;
                widget.swim_blink_steps = i32::from(widget.tuning.blink_steps);
                for index in 0..SWIM_BUBBLE_COUNT {
                    widget.pane.bubble[index].user_area = 0;
                }
                widget.play(SE_SWIM_TIMER_ON);
            }
        }
        1 => {
            if input.air.count != 0 {
                if !input.menu_open && !input.event_running && input.air.running {
                    swim_open_proc(widget);
                } else {
                    swim_pane_hide(widget);
                }
            } else {
                swim_pane_hide(widget);
                widget.swim_state = 0;
                if widget.timer_screen[0] && widget.timer_screen[1] {
                    widget.timer_screen = [false; 2];
                }
            }
        }
        2 => {
            if input.air.count != 0 {
                if !input.menu_open && !input.event_running && input.air.running {
                    swim_move_proc(widget, input);
                } else {
                    swim_pane_hide(widget);
                }
            } else {
                swim_pane_hide(widget);
                widget.swim_state = 0;
                if widget.timer_screen[0] && widget.timer_screen[1] {
                    widget.timer_screen = [false; 2];
                }
            }
        }
        3 => {
            if !input.menu_open {
                swim_move_proc(widget, input);
            } else {
                swim_pane_hide(widget);
            }
        }
        _ => {}
    }
}

pub fn swim_draw(widget: &mut Widget) {
    if widget.timer_screen[1] {
        for index in 0..SWIM_BUBBLE_COUNT {
            widget.pane.bubble[index].set_alpha();
        }
        widget.pane.gauge_frame.set_alpha();
        widget.pane.gauge_mask.set_alpha();
    }
    if widget.timer_screen[0] {
        widget.pane.gauge_soul.set_alpha();
        widget.pane.gauge_main.set_alpha();
        widget.pane.gauge_blink.set_alpha();
        widget.pane.gauge_kilari.set_alpha();
    }
}

pub fn swim_open_proc(widget: &mut Widget) {
    widget.pane.gauge_frame.user_area += 1;
    let step = widget.pane.gauge_frame.user_area;
    if step == 1 {
        widget.pane.gauge_kilari.pane_scale_xy(0.1);
        widget.pane.gauge_kilari.set_now_alpha_zero();
    } else if step < 5 {
        let alpha = value_increase(4, (step - 1) as i32, ALPHA_SQUARE);
        widget.pane.gauge_kilari.pane_scale_xy((alpha * 0.9) + 0.1);
        widget.pane.gauge_kilari.set_now_alpha(alpha);
    } else if step < 8 {
        let alpha = value_increase(3, (step - 5) as i32, ALPHA_SQUARE);
        widget.pane.gauge_kilari.pane_scale_xy(1.0 - (alpha * 0.3));
    } else if step < 10 {
        let alpha = value_increase(2, (step - 8) as i32, ALPHA_SQUARE);
        widget.pane.gauge_kilari.pane_scale_xy((alpha * 0.3) + 0.7);
    } else if step < 12 {
        let alpha = value_increase(2, (step - 10) as i32, ALPHA_SQUARE);
        widget
            .pane
            .gauge_kilari
            .pane_scale_xy(1.0 - (alpha * 0.100000024));
    } else if step < 14 {
        let alpha = value_increase(2, (step - 12) as i32, ALPHA_SQUARE);
        widget
            .pane
            .gauge_kilari
            .pane_scale_xy((alpha * 0.100000024) + 0.9);
    } else if step == 14 {
        widget.pane.gauge_kilari.pane_scale_xy(1.0);
    }
    if step == 1 {
        swim_pane_trans_y(widget, -130.0);
        widget.pane.gauge_soul.pane_scale_y(1.0);
        widget.pane.gauge_main.pane_scale_y(1.0);
        widget.pane.gauge_blink.pane_scale_y(1.0);
        widget.pane.gauge_frame.pane_scale_y(1.0);
        widget.pane.gauge_soul.set_now_alpha_zero();
        widget.pane.gauge_main.set_now_alpha_zero();
        widget.pane.gauge_blink.set_now_alpha_zero();
        widget.pane.gauge_frame.set_now_alpha_zero();
    } else if step <= 5 {
        let alpha = value_increase(4, (step - 1) as i32, ALPHA_SQUARE);
        let offset = -((1.0 - alpha) * 130.0);
        swim_pane_trans_y(widget, offset);
        swim_pane_alpha(widget, alpha);
    } else if step <= 8 {
        let alpha = value_increase(3, (step - 5) as i32, ALPHA_SQUARE);
        let offset = -25.0 - (1.0 - alpha) * -25.0;
        swim_pane_trans_y(widget, offset);
        swim_pane_show(widget);
    } else if step <= 0xa {
        let alpha = value_increase(2, (step - 8) as i32, ALPHA_SQUARE);
        let offset = -((1.0 - alpha) * 25.0);
        swim_pane_trans_y(widget, offset);
        swim_pane_show(widget);
    } else if step <= 12 {
        let alpha = value_increase(2, (step - 10) as i32, ALPHA_SQUARE);
        let offset = -6.0 - (1.0 - alpha) * -6.0;
        swim_pane_trans_y(widget, offset);
        swim_pane_show(widget);
    } else if step < 14 {
        let alpha = value_increase(2, (step - 12) as i32, ALPHA_SQUARE);
        let offset = -((1.0 - alpha) * 6.0);
        swim_pane_trans_y(widget, offset);
        swim_pane_show(widget);
    } else if step == 14 {
        swim_pane_trans_y(widget, 0.0);
        widget.pane.gauge_soul.pane_scale_y(1.0);
        widget.pane.gauge_main.pane_scale_y(1.0);
        widget.pane.gauge_blink.pane_scale_y(1.0);
        widget.pane.gauge_frame.pane_scale_y(1.0);
        swim_pane_show(widget);
        widget.pane.gauge_frame.user_area = 0;
        widget.swim_state = 2;
    }
}

pub fn swim_move_proc(widget: &mut Widget, input: &mut Input) {
    swim_main_rotate(widget);
    swim_main_blink(widget, input);
    swim_main_down(widget, input);
    swim_tekari_scroll(widget);
    swim_light_move(widget, input);
    swim_pane_show(widget);
}

pub fn swim_main_rotate(widget: &mut Widget) {
    let spin = widget.tuning.spin_frames;
    widget.pane.gauge_frame.user_area += 1;
    let step = widget.pane.gauge_frame.user_area;
    let mut angle = 0.0;
    if step < spin {
        angle = (f32::from(step) * 360.0) / f32::from(spin);
    } else if step == spin {
        widget.pane.gauge_frame.user_area = 0;
        angle = 0.0;
    }
    rotate_z(&mut widget.pane.gauge_soul, angle, 0.0);
}

pub fn swim_main_blink(widget: &mut Widget, input: &mut Input) {
    let mut mask = widget.swim_mask_black;
    let mut base_black = widget.swim_base_black;
    let mut base_white = widget.swim_base_white;
    widget.pane.gauge_kilari.user_area += 1;
    let step = widget.pane.gauge_kilari.user_area;
    let total = widget.swim_blink_steps;
    let limit = (total * 2) as i16;
    if step < total as i16 {
        let ratio = f32::from(step) / total as f32;
        mask.r = (75.0 + (ratio * ((i32::from(widget.swim_low.r) - 0x4B) as f32))) as u8;
        mask.g = (75.0 + (ratio * ((i32::from(widget.swim_low.g) - 0x4B) as f32))) as u8;
        mask.b = (255.0 + (ratio * ((i32::from(widget.swim_low.b) - 0xFF) as f32))) as u8;
        base_black = Color::mix(widget.swim_base_black, widget.swim_full_black, ratio);
        base_white = Color::mix(widget.swim_base_white, widget.swim_full_white, ratio);
    } else if step < limit {
        let ratio = (f32::from(step) - total as f32) / total as f32;
        mask.r = (f32::from(widget.swim_low.r)
            + (ratio * ((0x4B - i32::from(widget.swim_low.r)) as f32))) as u8;
        mask.g = (f32::from(widget.swim_low.g)
            + (ratio * ((0x4B - i32::from(widget.swim_low.g)) as f32))) as u8;
        mask.b = (f32::from(widget.swim_low.b)
            + (ratio * ((0xFF - i32::from(widget.swim_low.b)) as f32))) as u8;
        base_black = Color::mix(widget.swim_full_black, widget.swim_base_black, ratio);
        base_white = Color::mix(widget.swim_full_white, widget.swim_base_white, ratio);
    } else if step >= limit {
        widget.pane.gauge_kilari.user_area = 0;
        mask.r = 0x4b;
        mask.g = 0x4b;
        mask.b = 0xff;
        base_black = widget.swim_base_black;
        base_white = widget.swim_base_white;
        if input.air.count <= 0 {
            widget.swim_blink_steps = 0;
        } else if input.air.count < input.air.max / 2 {
            let ratio = value_increase(
                input.air.max / 2,
                input.air.max / 2 - input.air.count,
                ALPHA_ROOT,
            );
            widget.swim_blink_steps = (widget.tuning.blink_steps as f32
                + ratio * (widget.tuning.blink_short as f32 - widget.tuning.blink_steps as f32))
                as i32;
            let low = Color::rgba(
                widget.tuning.low_r,
                widget.tuning.low_g,
                widget.tuning.low_b,
                mask.a,
            );
            widget.swim_low = low;
            widget.swim_full_black = low;
            widget.swim_full_white = low;
            if widget.pane.gauge_soul.user_area <= 0 {
                widget.pane.gauge_soul.user_area = 2;
                if !input.event_running {
                    widget.play(SE_SWIM_TIMER_BLINK);
                }
            } else {
                widget.pane.gauge_soul.user_area -= 1;
            }
        } else if input.air.count < 0 {
            widget.swim_blink_steps = i32::from(widget.tuning.blink_short);
            let low = Color::rgba(
                widget.tuning.low_r,
                widget.tuning.low_g,
                widget.tuning.low_b,
                mask.a,
            );
            widget.swim_low = low;
            widget.swim_full_black = low;
            widget.swim_full_white = low;
        }
    }
    if widget.swim_blink_steps != 0 {
        widget.pane.gauge_frame.set_black(base_black);
        widget.pane.gauge_frame.set_white(base_white);
        widget.pane.gauge_soul.set_black(mask);
        widget.pane.gauge_main.set_black(mask);
    } else {
        widget.pane.gauge_frame.set_black(widget.swim_full_black);
        widget.pane.gauge_frame.set_white(widget.swim_full_white);
        widget.pane.gauge_soul.set_black(widget.swim_low);
        widget.pane.gauge_main.set_black(widget.swim_low);
    }
}

pub fn swim_main_down(widget: &mut Widget, input: &mut Input) {
    let count = input.air.count;
    let area = f32::from(widget.pane.gauge_blink.user_area);
    pane_bottom_scale_y(&mut widget.pane.gauge_blink, count as f32 / area);
    widget.pane.gauge_main.pos_top_left.y = (widget.pane.gauge_blink.pos_center_orig.y
        - widget.pane.gauge_blink.size_orig.y / 2.0)
        - widget.pane.gauge_main.pos_top_left.y;
    widget.pane.gauge_main.pos_center.y =
        widget.pane.gauge_main.pos_top_left.y + widget.pane.gauge_main.size.y / 2.0;
    widget.pane.gauge_main.cpos_move();
    widget.pane.gauge_soul.pos_center.y = widget.pane.gauge_main.pos_center.y;
    widget.pane.gauge_soul.cpos_move();
    if count > 0 {
        if !input.event_running && input.air.running {
            input.air.count -= 1;
        }
        if input.air.count == 0 {
            widget.swim_state = 3;
            input.air.running = false;
        }
    }
}

pub fn swim_tekari_scroll(widget: &mut Widget) {
    let ramp = widget.tuning.tekari_ramp;
    let end_a = ramp + widget.tuning.tekari_hold_a;
    let end_b = end_a + ramp;
    let end_c = end_b + widget.tuning.tekari_ramp_b;
    let end_d = end_c + widget.tuning.tekari_hold_b;
    let left = (0.0
        + (widget.pane.gauge_frame.pos_center_orig.x - widget.pane.gauge_frame.size_orig.x / 2.0))
        as i16;
    let top = (1.0
        + (widget.pane.gauge_frame.pos_center_orig.y - widget.pane.gauge_frame.size_orig.y / 2.0))
        as i16;
    let right = (0.0
        + (widget.pane.gauge_frame.pos_center_orig.x + widget.pane.gauge_frame.size_orig.x / 2.0))
        as i16;
    let bottom = (1.0
        + (widget.pane.gauge_frame.pos_center_orig.y + widget.pane.gauge_frame.size_orig.y / 2.0))
        as i16;
    let frame = widget.swim_tekari_frame;
    let scroll = if frame < ramp {
        let ratio = value_increase(ramp as i32, frame as i32, ALPHA_SQUARE);
        (0.5 + (28.0 - (64.0 * ratio))) as i16
    } else if frame < end_a {
        -0x24
    } else if frame < end_b {
        let ratio = value_increase(ramp as i32, (frame - end_a) as i32, ALPHA_SQUARE);
        (0.5 + (28.0 - (64.0 * ratio))) as i16
    } else if frame < end_c {
        -0x24
    } else if frame < end_d {
        let ratio = value_increase(ramp as i32, (frame - end_b) as i32, ALPHA_SQUARE);
        (0.5 + (28.0 - (64.0 * ratio))) as i16
    } else {
        -0x24
    };
    widget.swim_tekari_frame += 1;
    if widget.swim_tekari_frame >= end_d {
        widget.swim_tekari_frame = 0;
    }
    widget.kilari.set_pos(left, top, right, bottom);
    widget.kilari.black = Color::rgba(0xFF, 0xFF, 0xFF, 0xFF);
    widget.kilari.white = Color::rgba(0xFF, 0xFF, 0xFF, 0x00);
    widget.kilari.set_scroll(SCROLL_SLOT0, 0, 0);
    widget.kilari.black.a = 0xFF;
    widget.kilari.set_scroll(SCROLL_SLOT1, scroll, scroll);
}

pub fn swim_light_move(widget: &mut Widget, input: &mut Input) {
    if input.air.count != 0 {
        if widget.swim_bubble_wait == 0 {
            swim_light_birth(widget, input);
        } else {
            widget.swim_bubble_wait -= 1;
        }
    }
    for index in 0..SWIM_BUBBLE_COUNT {
        if widget.pane.bubble[index].user_area != 0 {
            swim_light_anime(widget, input, index);
        }
    }
}

pub fn swim_light_birth(widget: &mut Widget, input: &mut Input) {
    let mut free = 0;
    for index in 0..SWIM_BUBBLE_COUNT {
        if widget.pane.bubble[index].user_area == 0 {
            free += 1;
        }
    }
    let slot = (rand_f(&mut input.seed, free as f32) as usize).min(SWIM_BUBBLE_COUNT - 1);
    widget.swim_bubble_rise[slot] = rand_f(&mut input.seed, 3.0) + 5.0;
    widget.swim_bubble_sway[slot] = rand_fx(&mut input.seed, 5.0);
    widget.pane.bubble[slot].user_area = 1;
    widget.pane.bubble[slot].pos_center.y =
        widget.pane.gauge_blink.pos_top_left.y + widget.tuning.bubble_top;
    widget.pane.bubble[slot].cpos_move();
    widget.swim_bubble_wait = rand_f(&mut input.seed, f32::from(widget.tuning.bubble_birth)) as i16;
}

pub fn swim_light_anime(widget: &mut Widget, input: &mut Input, index: usize) {
    let fade = widget.tuning.bubble_fade;
    let birth = widget.tuning.bubble_birth;
    let end = fade + birth;
    let step = widget.pane.bubble[index].user_area;
    let wave = (f32::from(step) * 0.5235988).sin();
    widget.pane.bubble[index].pos_center.x =
        widget.pane.bubble[index].pos_center_orig.x + (widget.swim_bubble_sway[index] * wave);
    widget.pane.bubble[index].pos_center.y =
        widget.pane.bubble[index].pos_center.y - widget.swim_bubble_rise[index];
    widget.pane.bubble[index].size.x = widget.tuning.bubble_size;
    widget.pane.bubble[index].size.y = widget.tuning.bubble_size;
    widget.pane.bubble[index].cpos_move();
    widget.swim_bubble_rise[index] *= rand_f(&mut input.seed, 0.2) + 0.8;
    widget.swim_bubble_sway[index] *= rand_f(&mut input.seed, 0.1) + 0.9;
    if step == 1 {
        widget.pane.bubble[index].set_now_alpha_zero();
    } else if step < fade {
        let alpha = value_increase((fade - 1) as i32, (step - 1) as i32, ALPHA_ROOT);
        widget.pane.bubble[index].set_now_alpha(alpha);
    } else if step < end {
        let ratio = value_increase((end - fade) as i32, (step - fade) as i32, ALPHA_SQUARE);
        widget.pane.bubble[index].set_now_alpha(1.0 - ratio);
    } else if step == end {
        widget.pane.bubble[index].set_now_alpha_zero();
    }
    widget.pane.bubble[index].user_area = step + 1;
    if widget.pane.bubble[index].user_area > end {
        widget.pane.bubble[index].user_area = 0;
    }
}

pub fn place_name_move(widget: &mut Widget, input: &mut Input) {
    if !widget.place_name {
        if input.place_name_state == PLACE_NAME_ON {
            input.request_place_name = true;
            widget.place_name = true;
        }
    } else if input.place_name_state == PLACE_NAME_NONE {
        widget.place_name = false;
    }
}

pub fn screen_data_set(
    widget: &mut Widget,
    parts1: &Screen,
    parts2: &Screen,
    parts3: &Screen,
    input: &Input,
) {
    widget.pane.anchor.bind(parts1, "wiy1");
    widget.pane.dungeon_item[0].bind(parts1, "yrig");
    widget.pane.dungeon_item[1].bind(parts1, "yup");
    widget.pane.dungeon_item[2].bind(parts1, "ylef");
    widget.pane.boss_health.bind(parts1, "r");
    widget.pane.boss_health_bar.bind(parts1, "l");
    widget.pane.boss_health_back.bind(parts1, "crs1");
    widget.pane.dungeon_item[0].pos_center_orig.x -= 2.0;
    widget.pane.dungeon_item[1].pos_center_orig.y -= 2.0;
    widget.pane.dungeon_item[2].pos_center_orig.x -= 1.0;
    widget.pane.boss_frame_1.bind(parts1, "bey1");
    widget.pane.boss_frame_2.bind(parts1, "bey2");
    widget.pane.boss_mark.bind(parts1, "enem");
    widget.pane.boss_mark_2.bind(parts1, "enm2");
    widget.pane.boss_name[0].bind(parts1, "ben1");
    widget.pane.boss_name[1].bind(parts1, "ben2");
    widget.pane.boss_name[2].bind(parts1, "ben3");
    widget.pane.boss_name[3].bind(parts1, "ben4");
    widget.pane.boss_name_2.bind(parts1, "ene3");
    widget.pane.boss_name_3.bind(parts1, "ene2");
    widget.pane.boss_name_4.bind(parts1, "ene1");
    widget.pane.boss_name_5.bind(parts1, "enk2");
    widget.pane.boss_name_6.bind(parts1, "enek");
    for index in 0..20 {
        widget.pane.heart[index].bind(parts1, &format!("ht{index:02}"));
        widget.pane.heart_shadow[index].bind(parts1, &format!("hk{index:02}"));
    }
    widget.pane.heart_flush.bind(parts1, "htfl");
    widget.pane.magic_bar[0].bind(parts1, "mbr1");
    widget.pane.magic_bar[1].bind(parts1, "mbr2");
    widget.pane.magic_bar[2].bind(parts1, "mbr3");
    widget.pane.magic_bar[3].bind(parts1, "mbr4");
    widget.pane.magic_bar[4].bind(parts1, "mbr5");
    widget.pane.magic_bar[5].bind(parts1, "mbr6");
    widget.pane.magic_bar[6].bind(parts1, "mbr7");
    widget.pane.magic_bar[7].bind(parts1, "mbr8");
    widget.pane.magic_bar_back_3.bind(parts1, "mbb3");
    widget.pane.magic_bar_back_2.bind(parts1, "mbb2");
    widget.pane.magic_bar_back_1.bind(parts1, "mbb1");
    widget.pane.magic_key_2.bind(parts1, "mbk2");
    widget.pane.magic_key_1.bind(parts1, "mbk1");
    widget.pane.compass_face.bind(parts1, "cmcv");
    widget.pane.compass_needle_mag.bind(parts1, "cmmg");
    widget.pane.compass_needle.bind(parts1, "cmmk");
    widget.pane.compass_ring.bind(parts1, "cmrg");
    widget.pane.compass_base.bind(parts1, "cmb1");
    widget.pane.clock_bar.bind(parts3, "bar");
    widget.pane.clock_ring_2.bind(parts3, "rng2");
    widget.pane.clock_horizon_1.bind(parts3, "hrz1");
    widget.pane.clock_horizon_2.bind(parts3, "hrz2");
    widget.pane.clock_star_day[0].bind(parts3, "str5");
    widget.pane.clock_star_day[1].bind(parts3, "str6");
    widget.pane.clock_star_day[2].bind(parts3, "str7");
    widget.pane.clock_moon_2.bind(parts3, "mon2");
    widget.pane.clock_sun_2.bind(parts3, "sun2");
    widget.pane.clock_circle[0].bind(parts3, "cir1");
    widget.pane.clock_circle[1].bind(parts3, "cir2");
    widget.pane.clock_circle[2].bind(parts3, "cir3");
    widget.pane.clock_circle[3].bind(parts3, "cir4");
    widget.pane.clock_needle.bind(parts3, "cnt2");
    widget.pane.clock_needle_back.bind(parts3, "cnt1");
    widget
        .pane
        .clock_needle_back
        .set_white(Color::unpack(0x0000_0000));
    widget.pane.clock_ring_1.bind(parts3, "rng1");
    widget.pane.clock_star_night[0].bind(parts3, "str1");
    widget.pane.clock_star_night[1].bind(parts3, "str2");
    widget.pane.clock_star_night[2].bind(parts3, "str3");
    widget.pane.clock_star_night[3].bind(parts3, "str4");
    widget.pane.clock_moon_1.bind(parts3, "mon1");
    widget.pane.clock_clip.bind(parts3, "clp");
    widget.pane.clock_cloud[0].bind(parts3, "cld1");
    widget.pane.clock_cloud[1].bind(parts3, "cld2");
    widget.pane.clock_cloud[2].bind(parts3, "cld3");
    widget.pane.clock_sun_1.bind(parts3, "sun1");
    widget.pane.compass_frame.bind(parts3, "cmtm");
    widget.pane.clock_clip.set_white(Color::unpack(0x0000_0000));
    widget.pane.key_back_1.bind(parts1, "kyl1");
    widget.pane.key_back_2.bind(parts1, "kyl2");
    widget.pane.key_digit_1[0].bind(parts1, "nm03");
    widget.pane.key_digit_1[1].bind(parts1, "nm04");
    widget.pane.key_digit_2[0].bind(parts1, "nm08");
    widget.pane.key_digit_2[1].bind(parts1, "nm09");
    widget.pane.key_frame.bind(parts1, "key0");
    widget.pane.magic_gauge_base.bind(parts1, "basv");
    widget.pane.item_back_dark.bind(parts1, "bawd");
    widget.pane.item_back_pale.bind(parts1, "bawp");
    widget.pane.item_back[1].bind(parts1, "baa2");
    widget.pane.item_white[1].bind(parts1, "bat2");
    widget.pane.item_label_1[1].bind(parts1, "y01");
    widget.pane.item_label_2[1].bind(parts1, "y10");
    widget.pane.item_name[1].bind(parts1, "yitm");
    widget.pane.item_icon[1].bind(parts1, "yitk");
    widget.pane.item_blast_1[1].bind(parts1, "bly1");
    widget.pane.item_blast_2[1].bind(parts1, "bly2");
    widget.pane.item_blast_3[1].bind(parts1, "bly3");
    widget.pane.item_blast_1[1].hide();
    widget.pane.item_blast_2[1].hide();
    widget.pane.item_blast_3[1].hide();
    widget.pane.item_label_1[0].bind(parts1, "x01");
    widget.pane.item_label_2[0].bind(parts1, "x10");
    widget.pane.item_name[0].bind(parts1, "xitm");
    widget.pane.item_icon[0].bind(parts1, "xitk");
    widget.pane.item_blast_1[0].bind(parts1, "blx1");
    widget.pane.item_blast_2[0].bind(parts1, "blx2");
    widget.pane.item_blast_3[0].bind(parts1, "blx3");
    widget.pane.item_blast_1[0].hide();
    widget.pane.item_blast_2[0].hide();
    widget.pane.item_blast_3[0].hide();
    widget.pane.item_label_1[2].bind(parts1, "z01");
    widget.pane.item_label_2[2].bind(parts1, "z10");
    widget.pane.item_name[2].bind(parts1, "zitm");
    widget.pane.item_icon[2].bind(parts1, "zitk");
    widget.pane.item_blast_1[2].bind(parts1, "blz1");
    widget.pane.item_blast_2[2].bind(parts1, "blz2");
    widget.pane.item_blast_3[2].bind(parts1, "blz3");
    widget.pane.item_blast_1[2].hide();
    widget.pane.item_blast_2[2].hide();
    widget.pane.item_blast_3[2].hide();
    widget.pane.magic_bar_base.bind(parts1, "baz");
    widget.pane.magic_name_1[1].bind(parts1, "bay");
    widget.pane.magic_name_2[1].bind(parts1, "b002");
    widget.pane.magic_icon[1].bind(parts1, "ba02");
    widget.pane.magic_name_1[0].bind(parts1, "bax1");
    widget.pane.magic_name_2[0].bind(parts1, "b001");
    widget.pane.magic_icon[0].bind(parts1, "ba01");
    widget.pane.item_back[0].bind(parts1, "baa");
    widget.pane.item_white[0].bind(parts1, "bawe");
    widget.pane.magic_name.bind(parts1, "bar1");
    widget.pane.magic_name_right.bind(parts1, "ba0r");
    widget.pane.heart_count_a.bind(parts1, "a10");
    widget.pane.heart_count_b.bind(parts1, "a01");
    widget.pane.item_count_a.bind(parts1, "yum");
    widget.pane.item_count_b.bind(parts1, "yumk");
    widget.pane.weight_frame.bind(parts1, "weit");
    widget.pane.weight_icon.bind(parts1, "witk");
    widget.pane.bomb_frame.bind(parts1, "bab");
    widget.pane.bomb_icon.bind(parts1, "baat");
    widget.pane.meter_center.bind(parts1, "cent");
    widget.pane.ammo_frame[0].bind(parts1, "arw1");
    widget.pane.ammo_glow[0].bind(parts1, "ip00");
    widget.pane.ammo_frame[1].bind(parts1, "arw2");
    widget.pane.ammo_glow[1].bind(parts1, "ip02");
    widget.pane.rupee_back.bind(parts2, "rup1");
    widget.pane.rupee_light.bind(parts2, "rup2");
    widget.pane.rupee_digit_1[0].bind(parts2, "nm00");
    widget.pane.rupee_digit_1[1].bind(parts2, "nm01");
    widget.pane.rupee_digit_1[2].bind(parts2, "nm02");
    widget.pane.rupee_digit_1[3].bind(parts2, "nm03");
    widget.pane.rupee_digit_2[0].bind(parts2, "nm04");
    widget.pane.rupee_digit_2[1].bind(parts2, "nm05");
    widget.pane.rupee_digit_2[2].bind(parts2, "nm06");
    widget.pane.rupee_digit_2[3].bind(parts2, "nm07");
    widget.pane.rupee_digit_1[1].user_area = i16::from(input.wallet);
    wallet_change(widget, input);
}

pub fn screen_data_time_set(widget: &mut Widget, timer1: &Screen, timer2: &Screen) {
    widget.timer_screen = [true; 2];
    widget.pane.bubble[8].bind(timer2, "hk33");
    widget.pane.bubble[7].bind(timer2, "hk32");
    widget.pane.bubble[6].bind(timer2, "hk31");
    widget.pane.bubble[5].bind(timer2, "hk23");
    widget.pane.bubble[4].bind(timer2, "hk22");
    widget.pane.bubble[3].bind(timer2, "hk21");
    widget.pane.bubble[2].bind(timer2, "hk13");
    widget.pane.bubble[1].bind(timer2, "hk12");
    widget.pane.bubble[0].bind(timer2, "hk11");
    widget.pane.gauge_frame.bind(timer2, "wk00");
    widget.pane.gauge_mask.bind(timer2, "msk2");
    widget.pane.gauge_blink.bind(timer1, "msk1");
    widget.pane.gauge_soul.bind(timer1, "soul");
    widget.pane.gauge_main.bind(timer1, "main");
    widget.pane.gauge_kilari.bind(timer1, "kg00");
    widget.pane.gauge_cover.bind(timer2, "kra");
    widget.pane.gauge_cover.hide();
}

pub fn screen_data_arrow_set(widget: &mut Widget, choice_road: &Screen) {
    widget.pane.direction_arrow[0].bind(choice_road, "yaz4");
    widget.pane.direction_arrow[1].bind(choice_road, "yaz3");
    widget.pane.direction_arrow[2].bind(choice_road, "yaz2");
    widget.pane.direction_arrow[3].bind(choice_road, "yaz1");
}

pub fn draw(widget: &mut Widget, input: &Input, peers: &mut dyn Peers) -> DrawOut {
    widget.pane.anchor.set_alpha();
    for index in 0..3 {
        widget.pane.dungeon_item[index].set_alpha();
    }
    widget.pane.boss_health.set_alpha();
    widget.pane.boss_health_bar.set_alpha();
    widget.pane.boss_health_back.set_alpha();
    widget.pane.boss_frame_1.set_alpha();
    widget.pane.boss_frame_2.set_alpha();
    widget.pane.boss_mark.set_alpha();
    widget.pane.boss_mark_2.set_alpha();
    for index in 0..4 {
        widget.pane.boss_name[index].set_alpha();
    }
    widget.pane.boss_name_2.set_alpha();
    widget.pane.boss_name_3.set_alpha();
    widget.pane.boss_name_4.set_alpha();
    widget.pane.boss_name_5.set_alpha();
    widget.pane.boss_name_6.set_alpha();
    peers.heart_draw(widget);
    for index in 0..8 {
        widget.pane.magic_bar[index].set_alpha();
    }
    widget.pane.magic_bar_back_3.set_alpha();
    widget.pane.magic_bar_back_1.set_alpha();
    widget.pane.magic_key_1.set_alpha();
    widget.pane.compass_face.set_alpha();
    widget.pane.compass_base.set_alpha();
    widget.pane.clock_bar.set_alpha();
    widget.pane.clock_ring_2.set_alpha();
    widget.pane.clock_horizon_1.set_alpha();
    widget.pane.clock_horizon_2.set_alpha();
    for index in 0..3 {
        widget.pane.clock_star_day[index].set_alpha();
        widget.pane.clock_cloud[index].set_alpha();
    }
    widget.pane.clock_moon_2.set_alpha();
    widget.pane.clock_sun_2.set_alpha();
    for index in 0..4 {
        widget.pane.clock_circle[index].set_alpha();
        widget.pane.clock_star_night[index].set_alpha();
    }
    widget.pane.clock_needle.set_alpha();
    widget.pane.clock_needle_back.set_alpha();
    widget.pane.clock_ring_1.set_alpha();
    widget.pane.clock_moon_1.set_alpha();
    widget.pane.clock_clip.set_alpha();
    widget.pane.clock_sun_1.set_alpha();
    widget.pane.compass_frame.set_alpha();
    if widget.pane.clock_clip.is_visible() && widget.compass_state == COMPASS_WIND_OPENING {
        for index in 0..3 {
            widget.clock[index].black.a = widget.pane.clock_clip.now_alpha;
        }
    } else {
        for index in 0..3 {
            widget.clock[index].black.a = 0;
        }
    }
    widget.pane.key_back_1.set_alpha();
    widget.pane.key_back_2.set_alpha();
    for index in 0..SLOT_COUNT_2 {
        widget.pane.key_digit_1[index].set_alpha();
        widget.pane.key_digit_2[index].set_alpha();
    }
    widget.pane.key_frame.set_alpha();
    peers.wepen_draw(widget);
    peers.action_draw(widget);
    peers.item_row_draw(widget);
    peers.gauge_draw(widget);
    for index in 0..DIGIT_COUNT {
        widget.pane.rupee_digit_1[index].set_alpha();
        widget.pane.rupee_digit_2[index].set_alpha();
    }
    widget.pane.rupee_back.set_alpha();
    widget.pane.rupee_light.set_alpha();
    swim_draw(widget);
    direction_draw(widget);

    let mut out = DrawOut {
        result: true,
        ..DrawOut::default()
    };
    out.quads.push(widget.pane.anchor.quad());
    out.quads
        .extend(widget.pane.dungeon_item.iter().map(Pane::quad));
    out.quads.push(widget.pane.boss_health.quad());
    out.quads.push(widget.pane.boss_health_bar.quad());
    out.quads.push(widget.pane.boss_health_back.quad());
    out.quads.push(widget.pane.boss_frame_1.quad());
    out.quads.push(widget.pane.boss_frame_2.quad());
    out.quads.push(widget.pane.boss_mark.quad());
    out.quads.push(widget.pane.boss_mark_2.quad());
    out.quads
        .extend(widget.pane.boss_name.iter().map(Pane::quad));
    out.quads.push(widget.pane.boss_name_2.quad());
    out.quads.push(widget.pane.boss_name_3.quad());
    out.quads.push(widget.pane.boss_name_4.quad());
    out.quads.push(widget.pane.boss_name_5.quad());
    out.quads.push(widget.pane.boss_name_6.quad());
    out.quads
        .extend(widget.pane.magic_bar.iter().map(Pane::quad));
    out.quads.push(widget.pane.magic_bar_back_3.quad());
    out.quads.push(widget.pane.magic_bar_back_1.quad());
    out.quads.push(widget.pane.magic_key_1.quad());
    out.quads.push(widget.pane.compass_face.quad());
    out.quads.push(widget.pane.compass_base.quad());
    out.quads
        .extend(widget.pane.clock_star_day.iter().map(Pane::quad));
    out.quads
        .extend(widget.pane.clock_circle.iter().map(Pane::quad));
    out.quads
        .extend(widget.pane.clock_star_night.iter().map(Pane::quad));
    out.quads
        .extend(widget.pane.clock_cloud.iter().map(Pane::quad));
    out.quads.push(widget.pane.clock_needle.quad());
    out.quads.push(widget.pane.clock_needle_back.quad());
    out.quads.push(widget.pane.clock_ring_1.quad());
    out.quads.push(widget.pane.clock_moon_1.quad());
    out.quads.push(widget.pane.clock_clip.quad());
    out.quads.push(widget.pane.clock_sun_1.quad());
    out.quads.push(widget.pane.compass_frame.quad());
    out.quads.push(widget.pane.key_back_1.quad());
    out.quads.push(widget.pane.key_back_2.quad());
    out.quads
        .extend(widget.pane.key_digit_1.iter().map(Pane::quad));
    out.quads
        .extend(widget.pane.key_digit_2.iter().map(Pane::quad));
    out.quads.push(widget.pane.key_frame.quad());
    out.quads
        .extend(widget.pane.rupee_digit_1.iter().map(Pane::quad));
    out.quads
        .extend(widget.pane.rupee_digit_2.iter().map(Pane::quad));
    out.quads.push(widget.pane.rupee_back.quad());
    out.quads.push(widget.pane.rupee_light.quad());
    out.quads.extend(widget.pane.bubble.iter().map(Pane::quad));
    out.quads.push(widget.pane.gauge_frame.quad());
    out.quads.push(widget.pane.gauge_mask.quad());
    out.quads.push(widget.pane.gauge_blink.quad());
    out.quads.push(widget.pane.gauge_soul.quad());
    out.quads.push(widget.pane.gauge_main.quad());
    out.quads.push(widget.pane.gauge_kilari.quad());
    out.quads
        .extend(widget.pane.ammo_frame.iter().map(Pane::quad));
    out.quads
        .extend(widget.pane.ammo_glow.iter().map(Pane::quad));
    out.quads
        .extend(widget.pane.direction_arrow.iter().map(Pane::quad));
    out.clock = widget.clock;
    out.kilari = widget.kilari;
    out.glow = widget.rupee_glow;
    out.ammo_glow = widget.ammo_glow;
    if !input.overlay_running {
        out.layers.push(Layer::MeterBase);
        for index in 0..3 {
            out.layers.push(Layer::Clock(index));
        }
        if widget.timer_screen[0] && widget.timer_screen[1] {
            out.layers.push(Layer::Kilari);
        }
        out.layers.push(Layer::MeterTop);
        out.item_quad = item_move_draw(widget);
        if widget.button_icon_mode == 1
            && input.metronome
            && widget.status_flags & ST_MENU_ITEM == 0
            && widget.status_flags & ST_MENU_COLLECT == 0
            && widget.status_flags & ST_MENU_NAME == 0
            && widget.status_flags & ST_MENU_MAP == 0
        {
            out.layers.push(Layer::Metronome);
        }
    }
    out.sounds = std::mem::take(&mut widget.sounds);
    out
}

pub fn execute(widget: &mut Widget, input: &mut Input, peers: &mut dyn Peers) -> bool {
    widget.button_icon_mode = 0;
    peers.status_check(widget, input);
    input.status_flags = widget.status_flags;
    peers.alpha_control(widget, input);
    peers.heart_move(widget, input);
    peers.action_move(widget, input);
    peers.wepen_move(widget, input);
    peers.item_row_move(widget, input);
    peers.gauge_move(widget, input);

    let tuning = widget.tuning;
    let scale_x = f32::from(tuning.row_scale_x) / 100.0;
    let scale_y = f32::from(tuning.row_scale_y) / 100.0;
    let scale_item = f32::from(tuning.item_scale_y) / 100.0;

    parent_pane_trans(&mut widget.pane.meter_center, &tuning, scale_x);
    let center = widget.pane.meter_center.clone();
    child_pane_trans(&mut widget.pane.bomb_frame, &tuning, &center, scale_x);
    child_pane_trans(&mut widget.pane.bomb_icon, &tuning, &center, scale_x);
    child_pane_trans_only(
        &mut widget.pane.item_back_dark,
        &tuning,
        &center,
        scale_x,
        scale_y,
    );
    child_pane_trans_only(
        &mut widget.pane.weight_frame,
        &tuning,
        &center,
        scale_x,
        scale_item,
    );
    let weight = widget.pane.weight_frame.clone();
    child_pane_trans_child_trans(
        &mut widget.pane.weight_icon,
        &tuning,
        &center,
        &weight,
        scale_x,
        scale_item,
    );
    child_pane_trans_only(
        &mut widget.pane.item_back_pale,
        &tuning,
        &center,
        scale_x,
        scale_y,
    );
    for index in 0..2 {
        child_pane_trans(
            &mut widget.pane.item_white[index],
            &tuning,
            &center,
            scale_x,
        );
        child_pane_trans(&mut widget.pane.item_back[index], &tuning, &center, scale_x);
    }
    child_pane_trans(&mut widget.pane.heart_count_a, &tuning, &center, scale_x);
    child_pane_trans(&mut widget.pane.heart_count_b, &tuning, &center, scale_x);
    child_pane_trans(&mut widget.pane.item_count_a, &tuning, &center, scale_x);
    child_pane_trans(&mut widget.pane.item_count_b, &tuning, &center, scale_x);
    for index in 0..3 {
        child_pane_trans_only(
            &mut widget.pane.item_name[index],
            &tuning,
            &center,
            scale_x,
            scale_item,
        );
        let name = widget.pane.item_name[index].clone();
        child_pane_trans_child_trans_only(
            &mut widget.pane.item_label_1[index],
            &tuning,
            &center,
            &name,
            scale_x,
            scale_item,
        );
        child_pane_trans_child_trans_only(
            &mut widget.pane.item_label_2[index],
            &tuning,
            &center,
            &name,
            scale_x,
            scale_item,
        );
        child_pane_trans_child_trans(
            &mut widget.pane.item_icon[index],
            &tuning,
            &center,
            &name,
            scale_x,
            scale_item,
        );
        child_pane_trans_child_trans(
            &mut widget.pane.item_blast_1[index],
            &tuning,
            &center,
            &name,
            scale_x,
            scale_item,
        );
        child_pane_trans_child_trans(
            &mut widget.pane.item_blast_2[index],
            &tuning,
            &center,
            &name,
            scale_x,
            scale_item,
        );
        child_pane_trans_child_trans(
            &mut widget.pane.item_blast_3[index],
            &tuning,
            &center,
            &name,
            scale_x,
            scale_item,
        );
    }
    child_pane_trans(&mut widget.pane.magic_bar_base, &tuning, &center, scale_x);
    for index in 0..2 {
        child_pane_trans(
            &mut widget.pane.magic_name_1[index],
            &tuning,
            &center,
            scale_x,
        );
        child_pane_trans(
            &mut widget.pane.magic_name_2[index],
            &tuning,
            &center,
            scale_x,
        );
        child_pane_trans(
            &mut widget.pane.magic_icon[index],
            &tuning,
            &center,
            scale_x,
        );
    }
    child_pane_trans_only(
        &mut widget.pane.magic_gauge_base,
        &tuning,
        &center,
        scale_x,
        scale_y,
    );
    child_pane_trans(&mut widget.pane.magic_name_right, &tuning, &center, scale_x);
    child_pane_trans(&mut widget.pane.magic_name, &tuning, &center, scale_x);

    peers.enemy_move(widget, input);
    peers.magic_move(widget, input);
    peers.wind_move(widget, input);
    peers.metronome_move(widget, input);
    rupee_move(widget, input);
    key_move(widget, input);
    compass_move(widget, input);
    map_move(widget, input);
    swim_move(widget, input);
    direction_move(widget, input);
    if widget.item_move_flag != 0 {
        item_move_move(widget, input);
    }
    ammo_move(widget, input);
    place_name_move(widget, input);
    peers.menu_move(widget, input);
    peers.gauge_alpha(widget, input);
    input.button_icon_mode = widget.button_icon_mode;
    true
}

pub fn is_delete(_widget: &Widget) -> bool {
    true
}

pub fn delete(widget: &mut Widget, input: &mut Input) -> bool {
    input.map_alpha = u8::from(widget.map_control.shown == 0);
    if widget.rupee_sound_on != 0 {
        widget.play(SE_RUPEE_UP_2);
        widget.rupee_sound_set = 0;
        widget.rupee_sound_on = 0;
    }
    if widget.wind_id != NO_PROCESS {
        widget.wind_id = NO_PROCESS;
        widget.wind_status = 0;
    }
    if widget.rupee_glow.alive {
        widget.rupee_glow.kill();
    }
    for index in 0..SLOT_COUNT_2 {
        widget.ammo_glow[index].kill();
    }
    widget.timer_screen = [false; 2];
    widget.item_icon = Pane::default();
    true
}

pub fn create(
    parts1: &Screen,
    parts2: &Screen,
    parts3: &Screen,
    timer1: &Screen,
    timer2: &Screen,
    choice_road: &Screen,
    input: &mut Input,
) -> Widget {
    input.map_alpha = 0;
    input.menu_status = 0;
    input.menu_timer = 0;
    input.menu_open = false;
    input.menu_button = 0;
    input.sub_window_flag = 0;
    input.auction_flag = 0;
    input.texture_rate_x = 0.0;
    input.texture_rate_y = 0.0;
    input.texture_scale_x = 0.0;
    input.texture_scale_y = 0.0;
    input.item_mode = 0;
    input.place_name_state = PLACE_NAME_NONE;
    input.collect_mode = 0;
    input.button_icon_mode = 0;

    let mut widget = Widget::default();
    widget.auction_flag = input.auction_flag;
    if input.start_stage != START_STAGE_SEA_T {
        input.request_menu_window = true;
    }
    input.request_place_name = false;
    widget.timer_screen_data = [Some(timer1.clone()), Some(timer2.clone())];
    screen_data_set(&mut widget, parts1, parts2, parts3, input);
    screen_data_arrow_set(&mut widget, choice_road);
    widget.wake_alpha = 1.0;
    widget.wake_step = widget.tuning.wake_steps;
    map_init(&mut widget, input);
    compass_init(&mut widget, input);
    direction_init(&mut widget);
    rupee_init(&mut widget, input);
    key_init(&mut widget, input);
    item_move_init(&mut widget);
    ammo_init(&mut widget, input);
    widget
}

pub fn map_ctrl_init(widget: &mut Widget, input: &mut Input) {
    widget.map_view.alpha = 0;
    widget.map_view.icon_free_alpha = 0;
    widget.map_view.mode = 1;
    widget.map_view.left_up = F2i::new(MAP_LEFT_UP_X, MAP_LEFT_UP_Y);
    widget.map_view.icon_free_pos = F2i::new(MAP_ICON_FREE_X, MAP_ICON_FREE_Y);
    widget.map_view.icon_free_scale = 0.0;
    widget.map_view.icon_self_alpha = 0;
    widget.map_view.icon_self_scale = 0.0;
    widget.map_view.icon_mode = MAP_ICON_NONE;
    widget.map_control.shown = 0;
    widget.map_control.flags = 0;
    widget.map_control.blend_step = 0;
    widget.map_control.slide = 0;
    input.map_view = widget.map_view;
    input.map_alpha = widget.map_view.alpha;
}

pub fn map_ctrl_move(widget: &mut Widget, input: &mut Input) {
    let alpha;
    let scale;
    if input.camera_manual {
        if widget.map_control.blend_step < MAP_CTRL_BLINK_LIMIT {
            widget.map_control.blend_step += 1;
            if widget.map_control.blend_step < MAP_CTRL_BLINK_IN {
                let ratio = value_increase(
                    MAP_SCROLL_FRAMES,
                    widget.map_control.blend_step as i32,
                    ALPHA_SQUARE,
                );
                alpha = ratio;
                scale = ratio * MAP_BLINK_IN;
            } else {
                let ratio = value_increase(
                    MAP_FADE_FRAMES,
                    widget.map_control.blend_step as i32 - 6,
                    ALPHA_LINEAR,
                );
                alpha = 1.0;
                scale = MAP_BLINK_IN - (ratio * MAP_BLINK_OUT);
            }
        } else {
            alpha = 1.0;
            scale = MAP_BLINK_HOLD;
        }
    } else {
        if widget.map_control.blend_step > MAP_CTRL_REST {
            widget.map_control.blend_step = MAP_CTRL_REST;
        }
        if widget.map_control.blend_step != 0 {
            widget.map_control.blend_step -= 1;
        }
        alpha = value_increase(
            MAP_FADE_FRAMES,
            widget.map_control.blend_step as i32,
            ALPHA_LINEAR,
        );
        scale = MAP_BLINK_HOLD;
    }
    widget.map_view.icon_mode = MAP_ICON_NONE;
    let icon_alpha = ((alpha * 255.0) * (MAP_ALPHA_BASE / 255.0)) as u8;
    widget.map_view.icon_free_alpha = icon_alpha;
    widget.map_view.icon_free_scale = scale * MAP_ICON_SCALE;
    if icon_alpha != 0 {
        widget.map_view.icon_mode = MAP_ICON_FREE;
    }
    if input.camera_attention && input.subject_accept && !input.event_running {
        widget.map_view.icon_self_alpha = MAP_ICON_SELF_ALPHA;
        widget.map_view.icon_mode = MAP_ICON_SELF;
    }
    if input.camera_body_mode == 4 {
        widget.map_view.icon_mode = MAP_ICON_NONE;
    }
    if widget.map_control.shown != 0 {
        if widget.map_control.slide != 0
            && add_calc_angle_s(
                &mut widget.map_control.slide,
                0,
                MAP_ANGLE_SCALE,
                MAP_ANGLE_MAX,
                MAP_ANGLE_MIN,
            ) == 0
            && widget.map_control.flags & 4 == 0
        {
            widget.play(SE_MAP_SHOW_END);
        }
    } else {
        add_calc_angle_s(
            &mut widget.map_control.slide,
            MAP_SLIDE_HOME,
            MAP_ANGLE_SCALE,
            MAP_ANGLE_MAX,
            MAP_ANGLE_MIN,
        );
    }
    let scissor = (input.scissor_y + input.scissor_height) - MAP_SCISSOR_BIAS;
    widget.map_view.left_up.x =
        (i32::from(MAP_LEFT_UP_X) + i32::from(widget.map_control.slide)) as i16;
    widget.map_view.left_up.y = MAP_LEFT_UP_Y;
    if (scissor as i32) < 0 {
        widget.map_view.left_up.y = (scissor as i32 + i32::from(MAP_LEFT_UP_Y)) as i16;
    }
    input.map_view = widget.map_view;
}

#[derive(SystemParam)]
pub struct Env<'w, 's> {
    pub day: Res<'w, DayNight>,
    pub rupees: Res<'w, Rupees>,
    pub status: Res<'w, PlayerStatus>,
    pub ship: Query<'w, 's, &'static Actor>,
    pub air: Res<'w, AirTimer>,
    pub widget: ResMut<'w, Widget>,
    pub input: ResMut<'w, Input>,
}

pub fn gather(env: &mut Env<'_, '_>) {
    let time = env.day.time;
    let time_pass = env.day.time_pass;
    let hour = env.day.hour();
    let minute = env.day.minute();
    let daynight = env.day.daynight();
    let weekday = env.day.weekday();
    let moon = env.day.moon_type();
    let rupee = env.rupees.count as i32;
    let bow_aim = env.status.check0(STTS0_BOW_AIM);
    let ship_angle = env
        .ship
        .iter()
        .find(|actor| actor.name == SHIP_ACTOR)
        .map_or(0, |actor| actor.angle[1]);
    let air = env.air.air;
    let input = env.input.bypass_change_detection();
    input.time = time;
    input.time_pass = time_pass;
    input.rupee = rupee;
    input.bow_aim = bow_aim;
    input.ship_angle = ship_angle;
    input.air = air;
    input.day_hour = hour;
    input.day_minute = minute;
    input.day_night = daynight;
    input.day_of_week = weekday;
    input.moon = moon;
}
