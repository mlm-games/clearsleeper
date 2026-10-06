use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;

use crate::j2d::{Screen, ScreenPane, Screens};
use crate::sim::flags::Collect;
use crate::sim::inventory::{Equipped, check_get_item};

pub const ROTATE_Z: u8 = b'z';
pub const HEART_COUNT: usize = 20;
pub const ITEM_BUTTON_COUNT: usize = 3;
pub const BEAST_COUNT: usize = 8;
pub const BAIT_COUNT: usize = 8;
pub const ITEM_SLOT_COUNT: usize = 21;
pub const BAG_COUNT: usize = 8;
pub const RESERVE_COUNT: usize = 8;
pub const TEX_SLOT_BYTES: usize = 0xc00;
pub const PARTS1: &str = "main_parts1.blo";
pub const PARTS2: &str = "main_parts2.blo";
pub const PARTS3: &str = "main_parts3.blo";
pub const SWIM1: &str = "time_swim_1.blo";
pub const SWIM2: &str = "time_swim_2.blo";
pub const CHOICE_ROAD: &str = "choice_road.blo";

pub mod ms {
    pub const UNK1: u32 = 0x0000_0001;
    pub const UNK2: u32 = 0x0000_0002;
    pub const UNK4: u32 = 0x0000_0004;
    pub const UNK8: u32 = 0x0000_0008;
    pub const UNK10: u32 = 0x0000_0010;
    pub const UNK20: u32 = 0x0000_0020;
    pub const UNK40: u32 = 0x0000_0040;
    pub const UNK80: u32 = 0x0000_0080;
    pub const UNK100: u32 = 0x0000_0100;
    pub const UNK200: u32 = 0x0000_0200;
    pub const UNK400: u32 = 0x0000_0400;
    pub const UNK800: u32 = 0x0000_0800;
    pub const UNK1000: u32 = 0x0000_1000;
    pub const UNK2000: u32 = 0x0000_2000;
    pub const UNK4000: u32 = 0x0000_4000;
    pub const UNK8000: u32 = 0x0000_8000;
    pub const UNK10000: u32 = 0x0001_0000;
    pub const UNK20000: u32 = 0x0002_0000;
    pub const UNK40000: u32 = 0x0004_0000;
    pub const UNK80000: u32 = 0x0008_0000;
    pub const UNK100000: u32 = 0x0010_0000;
    pub const UNK200000: u32 = 0x0020_0000;
    pub const UNK400000: u32 = 0x0040_0000;
    pub const UNK800000: u32 = 0x0080_0000;
}

pub mod act {
    pub const BLANK: u8 = 0x00;
    pub const LOOK: u8 = 0x01;
    pub const SPEAK: u8 = 0x02;
    pub const CHARTS: u8 = 0x03;
    pub const LIFT: u8 = 0x04;
    pub const CLIMB: u8 = 0x05;
    pub const LET_GO: u8 = 0x06;
    pub const RETURN: u8 = 0x07;
    pub const PUT_AWAY: u8 = 0x08;
    pub const DROP: u8 = 0x09;
    pub const CHECK: u8 = 0x0A;
    pub const OPEN: u8 = 0x0B;
    pub const ATTACK: u8 = 0x0C;
    pub const PRESS: u8 = 0x0D;
    pub const THROW: u8 = 0x0E;
    pub const CROUCH: u8 = 0x0F;
    pub const SIDLE: u8 = 0x10;
    pub const GRAB: u8 = 0x11;
    pub const JUMP: u8 = 0x12;
    pub const STOP: u8 = 0x13;
    pub const HANG: u8 = 0x14;
    pub const FOLD: u8 = 0x15;
    pub const LET_GO_ROPE: u8 = 0x16;
    pub const CHOOSE: u8 = 0x17;
    pub const ROW: u8 = 0x18;
    pub const NEXT: u8 = 0x19;
    pub const PARRY: u8 = 0x1A;
    pub const PICK_UP: u8 = 0x1B;
    pub const GET_IN_SHIP: u8 = 0x1C;
    pub const GET_OUT_SHIP: u8 = 0x1D;
    pub const SAVE: u8 = 0x1E;
    pub const OPTION: u8 = 0x1F;
    pub const TAKE_PHOTO: u8 = 0x20;
    pub const INFO: u8 = 0x21;
    pub const SWAP_MODES: u8 = 0x22;
    pub const FLY: u8 = 0x23;
    pub const CALL: u8 = 0x24;
    pub const BID: u8 = 0x25;
    pub const READ: u8 = 0x26;
    pub const CANCEL: u8 = 0x27;
    pub const ZOOM_IN: u8 = 0x28;
    pub const COMPARE: u8 = 0x29;
    pub const ZOOM01: u8 = 0x2A;
    pub const ZOOM02: u8 = 0x2B;
    pub const CRUISE: u8 = 0x2C;
    pub const THROW_2D: u8 = 0x2D;
    pub const CARRY_2E: u8 = 0x2E;
    pub const SWING: u8 = 0x2F;
    pub const SEA_CHART: u8 = 0x30;
    pub const PARRY_31: u8 = 0x31;
    pub const RETURN_32: u8 = 0x32;
    pub const SAVE_33: u8 = 0x33;
    pub const SAVE_34: u8 = 0x34;
    pub const SWORD: u8 = 0x35;
    pub const DEFEND: u8 = 0x36;
    pub const STICK: u8 = 0x37;
    pub const HATCHET: u8 = 0x38;
    pub const IRON_CLUB: u8 = 0x39;
    pub const LONGSWORD: u8 = 0x3A;
    pub const SPEAR: u8 = 0x3B;
    pub const SPEAR_DUPE: u8 = 0x3C;
    pub const BOW: u8 = 0x3D;
    pub const HIDDEN: u8 = 0x3E;
}

pub mod py {
    pub mod st0 {
        pub const UNK1: u32 = 0x0000_0001;
        pub const UNK2: u32 = 0x0000_0002;
        pub const UNK4: u32 = 0x0000_0004;
        pub const UNK10: u32 = 0x0000_0010;
        pub const UNK20: u32 = 0x0000_0020;
        pub const UNK40: u32 = 0x0000_0040;
        pub const UNK80: u32 = 0x0000_0080;
        pub const HANG: u32 = 0x0000_0100;
        pub const UNK200: u32 = 0x0000_0200;
        pub const UNK400: u32 = 0x0000_0400;
        pub const UNK800: u32 = 0x0000_0800;
        pub const BOW_AIM: u32 = 0x0000_1000;
        pub const SUBJECT: u32 = 0x0000_2000;
        pub const HOOKSHOT_AIM: u32 = 0x0000_4000;
        pub const SWORD_SWING: u32 = 0x0000_8000;
        pub const SHIP_RIDE: u32 = 0x0001_0000;
        pub const ROPE_AIM: u32 = 0x0002_0000;
        pub const UNK40000: u32 = 0x0004_0000;
        pub const BOOMERANG_AIM: u32 = 0x0008_0000;
        pub const SWIM: u32 = 0x0010_0000;
        pub const TELESCOPE_LOOK: u32 = 0x0020_0000;
        pub const BOOMERANG_WAIT: u32 = 0x0040_0000;
        pub const UNK800000: u32 = 0x0080_0000;
        pub const UNK1000000: u32 = 0x0100_0000;
        pub const UNK2000000: u32 = 0x0200_0000;
        pub const UNK4000000: u32 = 0x0400_0000;
        pub const CRAWL: u32 = 0x0800_0000;
        pub const UNK10000000: u32 = 0x1000_0000;
        pub const UNK20000000: u32 = 0x2000_0000;
        pub const SPIN_ATTACK: u32 = 0x4000_0000;
        pub const UNK80000000: u32 = 0x8000_0000;
    }
    pub mod st1 {
        pub const WIND_WAKER_CONDUCT: u32 = 0x0000_0001;
        pub const UNK2: u32 = 0x0000_0002;
        pub const UNK4: u32 = 0x0000_0004;
        pub const PICTO_BOX_AIM: u32 = 0x0000_0008;
        pub const UNK10: u32 = 0x0000_0010;
        pub const DEKU_LEAF_FLY: u32 = 0x0000_0020;
        pub const DEKU_LEAF_FAN: u32 = 0x0000_0040;
        pub const UNK80: u32 = 0x0000_0080;
        pub const UNK100: u32 = 0x0000_0100;
        pub const UNK200: u32 = 0x0000_0200;
        pub const SAIL: u32 = 0x0000_0400;
        pub const UNK800: u32 = 0x0000_0800;
        pub const UNK1000: u32 = 0x0000_1000;
        pub const UNK2000: u32 = 0x0000_2000;
        pub const UNK4000: u32 = 0x0000_4000;
        pub const UNK8000: u32 = 0x0000_8000;
        pub const UNK10000: u32 = 0x0001_0000;
        pub const UNK20000: u32 = 0x0002_0000;
        pub const UNK40000: u32 = 0x0004_0000;
        pub const UNK80000: u32 = 0x0008_0000;
    }
}

pub mod cam {
    pub const ATTENTION_00000100: u16 = 0x0100;
    pub const BODY_FLAG_02000000: u32 = 0x0200_0000;
}

pub mod it {
    pub const NONE: u8 = 0xFF;
    pub const JOY_PENDANT: u8 = 0x1F;
    pub const TELESCOPE: u8 = 0x20;
    pub const TINGLE_TUNER: u8 = 0x21;
    pub const WIND_WAKER: u8 = 0x22;
    pub const PICTO_BOX: u8 = 0x23;
    pub const SPOILS_BAG: u8 = 0x24;
    pub const GRAPPLING_HOOK: u8 = 0x25;
    pub const DELUXE_PICTO_BOX: u8 = 0x26;
    pub const BOW: u8 = 0x27;
    pub const IRON_BOOTS: u8 = 0x29;
    pub const MAGIC_ARMOR: u8 = 0x2A;
    pub const BAIT_BAG: u8 = 0x2C;
    pub const BOOMERANG: u8 = 0x2D;
    pub const DEKU_LEAF: u8 = 0x34;
    pub const HOOKSHOT: u8 = 0x2F;
    pub const DELIVERY_BAG: u8 = 0x30;
    pub const BOMB_BAG: u8 = 0x31;
    pub const SKULL_HAMMER: u8 = 0x33;
    pub const MAGIC_ARROW: u8 = 0x35;
    pub const LIGHT_ARROW: u8 = 0x36;
    pub const SKULL_NECKLACE: u8 = 0x45;
    pub const BOKOBABA_SEED: u8 = 0x46;
    pub const GOLDEN_FEATHER: u8 = 0x47;
    pub const KNIGHTS_CREST: u8 = 0x48;
    pub const RED_JELLY: u8 = 0x49;
    pub const GREEN_JELLY: u8 = 0x4A;
    pub const BLUE_JELLY: u8 = 0x4B;
    pub const EMPTY_BOTTLE: u8 = 0x50;
    pub const RED_POTION: u8 = 0x51;
    pub const GREEN_POTION: u8 = 0x52;
    pub const BLUE_POTION: u8 = 0x53;
    pub const HALF_SOUP_BOTTLE: u8 = 0x54;
    pub const SOUP_BOTTLE: u8 = 0x55;
    pub const WATER_BOTTLE: u8 = 0x56;
    pub const FAIRY_BOTTLE: u8 = 0x57;
    pub const FIREFLY_BOTTLE: u8 = 0x58;
    pub const FOREST_WATER: u8 = 0x59;
    pub const SAIL: u8 = 0x78;
    pub const BIRD_BAIT_5: u8 = 0x82;
    pub const HYOI_PEAR: u8 = 0x83;
    pub const TOWN_FLOWER: u8 = 0x8C;
    pub const SEA_FLOWER: u8 = 0x8D;
    pub const EXOTIC_FLOWER: u8 = 0x8E;
    pub const HEROS_FLAG: u8 = 0x8F;
    pub const BIG_CATCH_FLAG: u8 = 0x90;
    pub const BIG_SALE_FLAG: u8 = 0x91;
    pub const PINWHEEL: u8 = 0x92;
    pub const SICKLE_MOON_FLAG: u8 = 0x93;
    pub const SKULL_TOWER_IDOL: u8 = 0x94;
    pub const FOUNTAIN_IDOL: u8 = 0x95;
    pub const POSTMAN_STATUE: u8 = 0x96;
    pub const SHOP_GURU_STATUE: u8 = 0x97;
    pub const FATHER_LETTER: u8 = 0x98;
    pub const NOTE_TO_MOM: u8 = 0x99;
    pub const MAGGIES_LETTER: u8 = 0x9A;
    pub const MOBLINS_LETTER: u8 = 0x9B;
    pub const CABANA_DEED: u8 = 0x9C;
    pub const COMPLIMENTARY_ID: u8 = 0x9D;
    pub const FILL_UP_COUPON: u8 = 0x9E;
}

pub mod slot {
    pub const FIRST: u8 = 0;
    pub const LAST: u8 = 21;
    pub const BEAST_FIRST: u8 = 24;
    pub const BEAST_LAST: u8 = 32;
    pub const BAIT_FIRST: u8 = 36;
    pub const BAIT_LAST: u8 = 44;
    pub const RESERVE_FIRST: u8 = 48;
    pub const RESERVE_LAST: u8 = 56;
    pub const NONE: u8 = 0xFF;
}

pub mod beast {
    pub const SKULL_NECKLACE: u8 = 0;
    pub const BOKOBABA_SEED: u8 = 1;
    pub const GOLDEN_FEATHER: u8 = 2;
    pub const KNIGHTS_CREST: u8 = 3;
    pub const RED_JELLY: u8 = 4;
    pub const GREEN_JELLY: u8 = 5;
    pub const BLUE_JELLY: u8 = 6;
    pub const JOY_PENDANT: u8 = 7;
    pub const COUNT: u8 = 8;
}

pub mod stage {
    pub const TYPE_NONE: u8 = 0;
    pub const TYPE_DUNGEON: u8 = 1;
    pub const TYPE_MISC: u8 = 2;
    pub const TYPE_BOSS: u8 = 3;
    pub const SAVE_DRC: u8 = 3;
    pub const SAVE_FW: u8 = 4;
    pub const SAVE_ET: u8 = 6;
    pub const SAVE_WT: u8 = 7;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl TColor {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn all(value: u8) -> Self {
        Self {
            r: value,
            g: value,
            b: value,
            a: value,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct F2 {
    pub x: f32,
    pub y: f32,
}

impl F2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaneNode {
    pub name: String,
    pub bound: bool,
    pub visible: bool,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub base_position: F2,
    pub rotation_axis: u8,
    pub rotation: f32,
    pub alpha: u8,
    pub white: TColor,
    pub black: TColor,
    pub texture: String,
}

impl PaneNode {
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
        self.x = x;
        self.y = y;
    }

    pub fn resize(&mut self, w: f32, h: f32) {
        self.w = w;
        self.h = h;
    }

    pub fn set_alpha(&mut self, alpha: u8) {
        self.alpha = alpha;
    }

    pub fn set_white(&mut self, color: TColor) {
        self.white = color;
    }

    pub fn set_black(&mut self, color: TColor) {
        self.black = color;
    }

    pub fn change_texture(&mut self, name: &str) {
        self.texture.clear();
        self.texture.push_str(name);
    }

    pub fn rotate(&mut self, offset_x: f32, offset_y: f32, axis: u8, angle: f32) {
        self.base_position = F2::new(offset_x, offset_y);
        self.rotation_axis = axis;
        self.rotation = angle;
    }

    pub fn get_rotate(&self) -> f32 {
        self.rotation
    }

    pub fn from_screen(pane: &ScreenPane) -> Self {
        Self {
            name: pane.name.clone(),
            bound: true,
            visible: pane.visible,
            x: pane.bounds.x,
            y: pane.bounds.y,
            w: pane.bounds.w,
            h: pane.bounds.h,
            rotation: pane.rotation as f32,
            white: TColor::all(0xFF),
            black: TColor::all(0x00),
            texture: pane.texture.clone().unwrap_or_default(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MsgPane {
    pub node: PaneNode,
    pub pos_top_left_orig: F2,
    pub pos_top_left: F2,
    pub pos_center_orig: F2,
    pub pos_center: F2,
    pub size_orig: F2,
    pub size: F2,
    pub init_alpha: u8,
    pub now_alpha: u8,
    pub user_area: i16,
}

impl MsgPane {
    pub fn quad(&self) -> Quad {
        Quad {
            name: self.node.name.clone(),
            texture: self.node.texture.clone(),
            x: self.node.x,
            y: self.node.y,
            w: self.node.w,
            h: self.node.h,
            alpha: self.node.alpha,
            white: self.node.white,
            black: self.node.black,
            rotation: self.node.rotation,
            visible: self.node.visible,
            bound: self.node.bound,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Quad {
    pub name: String,
    pub texture: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub alpha: u8,
    pub white: TColor,
    pub black: TColor,
    pub rotation: f32,
    pub visible: bool,
    pub bound: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Archive {
    Menu,
    ActionIcon,
    ItemIcon,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TexSlot {
    pub bytes: usize,
    pub kind: String,
    pub name: String,
    pub archive: Option<Archive>,
}

impl TexSlot {
    pub fn new() -> Self {
        Self {
            bytes: TEX_SLOT_BYTES,
            kind: String::from("TIMG"),
            name: String::new(),
            archive: None,
        }
    }

    pub fn read(&mut self, name: &str, archive: Archive) {
        self.kind.clear();
        self.kind.push_str("TIMG");
        self.name.clear();
        self.name.push_str(name);
        self.archive = Some(archive);
    }
}

impl Default for TexSlot {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RupyEmitter {
    pub live: bool,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub alpha: f32,
}

impl RupyEmitter {
    pub fn spawn(x: f32, y: f32, z: f32) -> Self {
        Self {
            live: true,
            x,
            y,
            z,
            alpha: 0.0,
        }
    }

    pub fn set_global_translation(&mut self, x: f32, y: f32, z: f32) {
        self.x = x;
        self.y = y;
        self.z = z;
    }

    pub fn play_draw_particle(&mut self) {
        self.live = true;
    }

    pub fn set_global_alpha(&mut self, alpha: f32) {
        self.alpha = alpha;
    }

    pub fn become_invalid_emitter(&mut self) {
        self.live = false;
    }

    pub fn quit_immortal_emitter(&mut self) {
        self.live = false;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundCue {
    HpGaugeInc,
    HeartGauge,
    SeedOkBlink,
    HarpIndicator,
    OmamoriBlink,
    SpecialAttackOn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecollectBoss {
    pub copy: bool,
    pub event_bit: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenLayer {
    pub label: String,
    pub scissor: bool,
}

pub fn value_increase(max: i32, value: i32, mode: u8) -> f32 {
    if max <= 0 {
        return 1.0;
    }
    let clamped = if value < 0 {
        0
    } else if value > max {
        max
    } else {
        value
    };
    let v = clamped as f32 / max as f32;
    match mode {
        0 => v * v,
        1 => v.sqrt(),
        3 => (v * 2.0 - 1.0) * 2.0 - 1.0,
        4 => (v * 0x8000 as f32 * 0.5).sin() * (v * 0x8000 as f32 * 0.5).sin(),
        5 => (v * 0xFFFF as f32 * 0.5).sin() * (v * 0xFFFF as f32 * 0.5).sin(),
        _ => v,
    }
}

pub fn center_pos_calc(top_left: F2, size: F2) -> F2 {
    F2::new(top_left.x + size.x / 2.0, top_left.y + size.y / 2.0)
}

pub fn pane_parts_set(pane: &mut MsgPane, screen_pane: &ScreenPane) {
    pane.pos_top_left_orig = F2::new(screen_pane.bounds.x, screen_pane.bounds.y);
    pane.size_orig = F2::new(screen_pane.bounds.w, screen_pane.bounds.h);
    pane.pos_center_orig = center_pos_calc(pane.pos_top_left_orig, pane.size_orig);
    pane.init_alpha = screen_pane.alpha;
    pane.pos_top_left = pane.pos_top_left_orig;
    pane.pos_center = pane.pos_center_orig;
    pane.size = pane.size_orig;
    pane.now_alpha = 0;
    pane.user_area = 0;
    pane.node = PaneNode::from_screen(screen_pane);
}

pub fn set_pane_data(pane: &mut MsgPane, screen: &Screen, tag: &str) {
    if let Some(found) = screen.find(tag) {
        pane_parts_set(pane, found);
    }
}

pub fn cpos_move(pane: &mut MsgPane) {
    pane.pos_top_left.x = pane.pos_center.x - pane.size.x / 2.0;
    pane.pos_top_left.y = pane.pos_center.y - pane.size.y / 2.0;
    pane.node.move_to(pane.pos_top_left.x, pane.pos_top_left.y);
    pane.node.resize(pane.size.x, pane.size.y);
}

pub fn pane_trans(pane: &mut MsgPane, trans_x: f32, trans_y: f32) {
    pane.pos_center.x = pane.pos_center_orig.x + trans_x;
    pane.pos_center.y = pane.pos_center_orig.y + trans_y;
    cpos_move(pane);
}

pub fn pane_scale_xy(pane: &mut MsgPane, scale: f32) {
    pane.size.x = pane.size_orig.x * scale;
    pane.size.y = pane.size_orig.y * scale;
    cpos_move(pane);
}

pub fn pane_bottom_scale_y(pane: &mut MsgPane, scale: f32) {
    let old_bottom = pane.pos_top_left.y + pane.size.y;
    pane.size.y = pane.size_orig.y * scale;
    pane.pos_center.y = old_bottom - (pane.size.y / 2.0);
    cpos_move(pane);
}

pub fn parent_pane_trans(pane: &mut MsgPane, scale: f32, offset_x: f32, offset_y: f32) {
    pane.pos_center.x += offset_x;
    pane.pos_center.y += offset_y;
    pane_scale_xy(pane, scale);
    let angle = pane.node.get_rotate();
    pane.node
        .rotate(pane.size.x / 2.0, pane.size.y / 2.0, ROTATE_Z, angle);
}

pub fn child_pane_trans(
    pane: &mut MsgPane,
    parent: &MsgPane,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
) {
    let x = pane.pos_center_orig.x - parent.pos_center_orig.x;
    let y = pane.pos_center_orig.y - parent.pos_center_orig.y;
    pane.pos_center.x += x * (scale - 1.0);
    pane.pos_center.y += y * (scale - 1.0);
    pane.pos_center.x += offset_x;
    pane.pos_center.y += offset_y;
    pane_scale_xy(pane, scale);
    let angle = pane.node.get_rotate();
    pane.node
        .rotate(pane.size.x / 2.0, pane.size.y / 2.0, ROTATE_Z, angle);
}

pub fn child_pane_trans_only(
    pane: &mut MsgPane,
    parent: &MsgPane,
    trans_scale: f32,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
) {
    let x = pane.pos_center_orig.x - parent.pos_center_orig.x;
    let y = pane.pos_center_orig.y - parent.pos_center_orig.y;
    pane.pos_center.x += x * (trans_scale - 1.0);
    pane.pos_center.y += y * (trans_scale - 1.0);
    pane.pos_center.x += offset_x;
    pane.pos_center.y += offset_y;
    pane_scale_xy(pane, scale);
    let angle = pane.node.get_rotate();
    pane.node
        .rotate(pane.size.x / 2.0, pane.size.y / 2.0, ROTATE_Z, angle);
}

pub fn child_pane_trans_child_trans(
    pane: &mut MsgPane,
    parent: &MsgPane,
    child: &MsgPane,
    trans_scale: f32,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
) {
    let mut x = pane.pos_center_orig.x - parent.pos_center_orig.x;
    let mut y = pane.pos_center_orig.y - parent.pos_center_orig.y;
    pane.pos_center.x += x * (trans_scale - 1.0);
    pane.pos_center.y += y * (trans_scale - 1.0);
    x = pane.pos_center.x - child.pos_center.x;
    y = pane.pos_center.y - child.pos_center.y;
    pane.pos_center.x += x * (scale - 1.0);
    pane.pos_center.y += y * (scale - 1.0);
    pane.pos_center.x += offset_x;
    pane.pos_center.y += offset_y;
    pane_scale_xy(pane, scale);
    let angle = pane.node.get_rotate();
    pane.node
        .rotate(pane.size.x / 2.0, pane.size.y / 2.0, ROTATE_Z, angle);
}

pub fn child_pane_trans_child_trans_only(
    pane: &mut MsgPane,
    parent: &MsgPane,
    _child: &MsgPane,
    trans_scale: f32,
    _scale: f32,
    offset_x: f32,
    offset_y: f32,
) {
    let x = pane.pos_center_orig.x - parent.pos_center_orig.x;
    let y = pane.pos_center_orig.y - parent.pos_center_orig.y;
    pane.pos_center.x += x * (trans_scale - 1.0);
    pane.pos_center.y += y * (trans_scale - 1.0);
    pane.pos_center.x += offset_x;
    pane.pos_center.y += offset_y;
    cpos_move(pane);
    let angle = pane.node.get_rotate();
    pane.node
        .rotate(pane.size.x / 2.0, pane.size.y / 2.0, ROTATE_Z, angle);
}

pub fn set_now_alpha(pane: &mut MsgPane, alpha: f32) {
    pane.now_alpha = (pane.init_alpha as f32 * alpha) as i64 as u8;
}

pub fn set_now_alpha_zero(pane: &mut MsgPane) {
    pane.now_alpha = 0;
}

pub fn set_alpha(pane: &mut MsgPane) {
    pane.node.set_alpha(pane.now_alpha);
}

pub fn pane_hide(pane: &mut MsgPane) {
    pane.node.hide();
}

pub fn pane_show(pane: &mut MsgPane) {
    pane.node.show();
}

pub fn is_bit8(flag: u8, bit: u8) -> bool {
    (flag & bit) != 0
}

pub fn on_bit8(flag: &mut u8, bit: u8) {
    *flag |= bit & 0xFF;
}

pub fn off_bit8(flag: &mut u8, bit: u8) {
    *flag &= !(bit & 0xFF);
}

pub fn alpha_close(timer: &mut i16, value: &mut i16) -> f32 {
    let r5: i16 = 6;
    if *timer == 8 {
        *timer = 9;
        *value = 1;
        1.0
    } else if *timer == 9 {
        if *value < r5 {
            let ret = 1.0 - value_increase(6, *value as i32, 1);
            *value += 1;
            ret
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
        if *value < 6 {
            let ret = value_increase(6, *value as i32, 1);
            *value += 1;
            ret
        } else {
            *timer = 8;
            1.0
        }
    } else {
        1.0
    }
}

pub fn rupy_num(num: i16) -> String {
    format!("rupy_num_{num:02}.bti")
}

const ACTION_TEX: [&str; 63] = [
    "ba_shiraberu.bti",
    "ba_miru.bti",
    "ba_syaberu.bti",
    "ba_shiraberu.bti",
    "ba_motu.bti",
    "ba_noru.bti",
    "ba_oriru.bti",
    "ba_modoru.bti",
    "ba_shimau.bti",
    "ba_oku.bti",
    "ba_chekku.bti",
    "ba_hiraku.bti",
    "ba_attack.bti",
    "ba_osu.bti",
    "ba_nageru.bti",
    "ba_shagamu.bti",
    "ba_harituku.bti",
    "ba_tukamu.bti",
    "ba_jump.bti",
    "ba_tomeru.bti",
    "ba_haru.bti",
    "ba_tatamu.bti",
    "ba_hanasu.bti",
    "ba_kettei.bti",
    "ba_kogu.bti",
    "ba_tugihe.bti",
    "ba_sake.bti",
    "ba_motu_buki.bti",
    "ba_noru_hune.bti",
    "ba_oriru_hune.bti",
    "ba_save.bti",
    "ba_option.bti",
    "ba_toru.bti",
    "ba_setumei.bti",
    "ba_kirikae.bti",
    "ba_tobu.bti",
    "ba_yobu.bti",
    "ba_hoshii.bti",
    "ba_yomu.bti",
    "ba_yameru.bti",
    "ba_kakudai.bti",
    "ba_hikaku.bti",
    "ba_zoom01.bti",
    "ba_zoom02.bti",
    "ba_susumu.bti",
    "ba_nageru.bti",
    "ba_motu.bti",
    "ba_huru.bti",
    "ba_kaizuhe.bti",
    "ba_sake.bti",
    "ba_modoru.bti",
    "ba_save.bti",
    "ba_save.bti",
    "sword_01.bti",
    "ba_husegu.bti",
    concat!("bo", "ko_stick.bti"),
    "hatchet.bti",
    "iron_club.bti",
    "longsword.bti",
    "spear.bti",
    "spear.bti",
    "bow_01.bti",
    "ba_modoru.bti",
];

pub fn action_tex(id: u8) -> &'static str {
    let mut found = 0;
    let mut index = 0;
    while index < ACTION_TEX.len() {
        if id as usize == index {
            found = index;
            break;
        }
        index += 1;
    }
    ACTION_TEX[found]
}

const WEPON_TEX: [&str; 4] = [
    "sword_00.bti",
    "sword_01.bti",
    "sword_02.bti",
    "sword_03.bti",
];

pub fn wepon_tex(
    collect_slot0_bit1: bool,
    collect_slot0_bit2: bool,
    collect_slot0_bit3: bool,
) -> &'static str {
    let index = if collect_slot0_bit3 {
        3
    } else if collect_slot0_bit2 {
        2
    } else if collect_slot0_bit1 {
        1
    } else {
        0
    };
    WEPON_TEX[index]
}

const HEART_TEX: [&str; 5] = [
    "heart.bti",
    "heart_03.bti",
    "heart_02.bti",
    "heart_01.bti",
    "heart_04.bti",
];

pub fn heart_tex(index: i16) -> &'static str {
    HEART_TEX[(index.max(0) as usize).min(HEART_TEX.len() - 1)]
}

const NUMBER_TEX: [&str; 10] = [
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

pub fn number_set(pane: &mut MsgPane, digit: u8) {
    let digit = if digit >= 10 { 0 } else { digit };
    pane.node.change_texture(NUMBER_TEX[digit as usize]);
}

pub fn number_color(pane: &mut MsgPane, value: u8, max: u8) {
    if value == 0 {
        pane.node.set_white(TColor::new(0xb4, 0xb4, 0xb4, 0xff));
        pane.node.set_black(TColor::new(0x1e, 0x1e, 0x1e, 0x00));
    } else if value == max {
        pane.node.set_white(TColor::new(0xff, 0xc8, 0x32, 0xff));
        pane.node.set_black(TColor::new(0x1e, 0x1e, 0x1e, 0x00));
    } else {
        pane.node.set_white(TColor::all(0xff));
        pane.node.set_black(TColor::all(0x00));
    }
}

pub fn item_texture(no: u8) -> &'static str {
    ITEM_TEX[(no as usize).min(ITEM_TEX.len() - 1)]
}

pub fn item_num_idx(no: u8) -> u8 {
    ITEM_ICON[(no as usize).min(ITEM_ICON.len() - 1)]
}

const ITEM_ICON: [u8; 60] = [
    it::TELESCOPE,
    it::SAIL,
    it::WIND_WAKER,
    it::GRAPPLING_HOOK,
    it::SPOILS_BAG,
    it::BOOMERANG,
    it::DEKU_LEAF,
    it::TINGLE_TUNER,
    it::PICTO_BOX,
    it::IRON_BOOTS,
    it::MAGIC_ARMOR,
    it::MAGIC_ARMOR,
    it::BOW,
    it::BOMB_BAG,
    it::EMPTY_BOTTLE,
    it::RED_POTION,
    it::GREEN_POTION,
    it::BLUE_POTION,
    it::DELIVERY_BAG,
    it::HOOKSHOT,
    it::SKULL_HAMMER,
    it::MAGIC_ARMOR,
    it::MAGIC_ARMOR,
    it::MAGIC_ARMOR,
    it::SKULL_NECKLACE,
    it::BOKOBABA_SEED,
    it::GOLDEN_FEATHER,
    it::KNIGHTS_CREST,
    it::RED_JELLY,
    it::GREEN_JELLY,
    it::BLUE_JELLY,
    it::JOY_PENDANT,
    it::NONE,
    it::NONE,
    it::NONE,
    it::NONE,
    it::HYOI_PEAR,
    it::BIRD_BAIT_5,
    it::NONE,
    it::NONE,
    it::NONE,
    it::NONE,
    it::NONE,
    it::NONE,
    it::NONE,
    it::NONE,
    it::NONE,
    it::NONE,
    it::TOWN_FLOWER,
    it::SEA_FLOWER,
    it::EXOTIC_FLOWER,
    it::HEROS_FLAG,
    it::BIG_CATCH_FLAG,
    it::BIG_SALE_FLAG,
    it::PINWHEEL,
    it::SICKLE_MOON_FLAG,
    it::SKULL_TOWER_IDOL,
    it::FOUNTAIN_IDOL,
    it::POSTMAN_STATUE,
    it::SHOP_GURU_STATUE,
];

const ITEM_TEX: [&str; 256] = [
    "bottle_00.bti",
    "get_rupy.bti",
    "get_rupy.bti",
    "get_rupy.bti",
    "get_rupy.bti",
    "get_rupy.bti",
    "get_rupy.bti",
    "heart_up_02.bti",
    "heart_up_01.bti",
    "bottle_00.bti",
    "bottle_00.bti",
    "bomb_00.bti",
    "bomb_00.bti",
    "bomb_00.bti",
    "bomb_00.bti",
    "get_rupy.bti",
    "bow_01.bti",
    "bow_01.bti",
    "bow_01.bti",
    "",
    "",
    "get_key.bti",
    "",
    "",
    "heart_up_02.bti",
    "",
    "heart_up_02.bti",
    "",
    "heart_up_02.bti",
    "get_rupy.bti",
    "",
    "",
    "",
    "bottle_00.bti",
    "beast_08.bti",
    "telescope.bti",
    "whistle.bti",
    "baton.bti",
    "camera.bti",
    "coverofbeast.bti",
    "rope.bti",
    "camera_2.bti",
    "bow_01.bti",
    "gloves_00.bti",
    "boots_00.bti",
    "shield_02.bti",
    "boots_01.bti",
    "coverofbait.bti",
    "boomerang.bti",
    "gloves_00.bti",
    "hookshot.bti",
    "delivery.bti",
    "bomb_00.bti",
    "clothes.bti",
    "hammer_01.bti",
    "fan.bti",
    "arrow_power_01.bti",
    "arrow_power_02.bti",
    "clothes.bti",
    "sword_00.bti",
    "sword_01.bti",
    "sword_02.bti",
    "shield_00.bti",
    "shield_01.bti",
    "sword_00.bti",
    "sword_03.bti",
    "heart_up_02.bti",
    "",
    "",
    "amulet_00.bti",
    "amulet_01.bti",
    "bottle_00.bti",
    "beast_01.bti",
    "beast_02.bti",
    "beast_03.bti",
    "beast_04.bti",
    "beast_05.bti",
    "beast_06.bti",
    "beast_07.bti",
    "dungeon_map.bti",
    "compass.bti",
    "boss_key.bti",
    "bottle_00.bti",
    "bottle_00.bti",
    "bottle_01.bti",
    "bottle_02.bti",
    "bottle_03.bti",
    "bottle_09.bti",
    "bottle_04.bti",
    "bottle_05.bti",
    "bottle_06.bti",
    "bottle_07.bti",
    "bottle_08.bti",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "triforce_00.bti",
    "triforce_01.bti",
    "triforce_02.bti",
    "triforce_03.bti",
    "triforce_04.bti",
    "triforce_05.bti",
    "triforce_06.bti",
    "triforce_07.bti",
    "god_symbol_02.bti",
    "god_symbol_00.bti",
    "god_symbol_01.bti",
    "amulet_00.bti",
    "baton.bti",
    "baton.bti",
    "baton.bti",
    "baton.bti",
    "baton.bti",
    "baton.bti",
    "",
    "",
    "",
    "",
    "",
    "sail_00.bti",
    "cmap_tri2.bti",
    "cmap_tri2.bti",
    "cmap_tri2.bti",
    "cmap_tri2.bti",
    "cmap_tri2.bti",
    "cmap_tri2.bti",
    "cmap_tri2.bti",
    "cmap_tri2.bti",
    "",
    "bait_01.bti",
    "bait_02.bti",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
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
    "delivery_19.bti",
    "camera.bti",
    "amulet_00.bti",
    "amulet_00.bti",
    "amulet_00.bti",
    "tingle_figure.bti",
    "tingle_figure.bti",
    "tingle_figure.bti",
    "tingle_figure.bti",
    "tingle_figure.bti",
    "",
    "",
    "",
    "big_purse.bti",
    "max_purse.bti",
    "bombpouch_1.bti",
    "bombpouch_2.bti",
    "arrowcase_1.bti",
    "arrowcase_2.bti",
    "",
    "",
    "get_rupy.bti",
    "get_rupy.bti",
    "get_rupy.bti",
    "get_rupy.bti",
    "get_rupy.bti",
    "get_rupy.bti",
    "get_rupy.bti",
    "get_rupy.bti",
    "get_rupy.bti",
    "get_rupy.bti",
    "get_rupy.bti",
    "get_rupy.bti",
    "get_rupy.bti",
    "get_rupy.bti",
    "get_rupy.bti",
    "cmap_hint2.bti",
    "cmap_hint2.bti",
    "cmap_hint2.bti",
    "cmap_hint2.bti",
    "cmap_hint2.bti",
    "cmap_hint2.bti",
    "cmap_hint2.bti",
    "cmap_hint2.bti",
    "cmap_hint2.bti",
    "cmap_tingle2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_phantomship2.bti",
    "cmap_tingle2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_treasure2.bti",
    "cmap_tri2.bti",
    "cmap_tri2.bti",
    "cmap_tri2.bti",
    "cmap_tri2.bti",
    "cmap_tri2.bti",
    "cmap_tri2.bti",
];

#[derive(Clone, Debug, Default)]
pub struct MeterInfo {
    pub mode: u8,
    pub button_icon_mode: u8,
}

#[derive(Clone, Debug)]
pub struct MeterTuning {
    pub m006: i16,
    pub m00c: f32,
    pub m010: f32,
    pub m014: f32,
    pub m018: f32,
    pub m01c: f32,
    pub m020: f32,
    pub m024: f32,
    pub m028: f32,
    pub m02c: u8,
    pub m02d: u8,
    pub m02e: i16,
    pub m030: i16,
    pub m032: i16,
    pub m034: i16,
    pub m036: i16,
    pub m038: i16,
    pub m03c: f32,
    pub m040: i16,
    pub m042: i16,
    pub m044: i16,
    pub m046: i16,
    pub m048: f32,
    pub m04c: f32,
    pub m050: i16,
    pub m052: i16,
    pub m054: i16,
    pub m056: i16,
    pub m058: f32,
    pub m05c: f32,
    pub m060: u8,
    pub m061: u8,
    pub m062: u8,
    pub m063: u8,
    pub m064: i16,
    pub m066: i16,
    pub m068: i16,
    pub m06a: i16,
    pub m06c: [i16; 3],
    pub m072: [i16; 3],
    pub m078: [i16; 3],
    pub m07e: i16,
    pub m088: i16,
    pub m08a: i16,
    pub m08c: i16,
    pub m08e: i16,
    pub m090: i16,
    pub m092: i16,
    pub m094: i16,
    pub m096: i16,
    pub m098: i16,
    pub m09a: i16,
    pub m09c: i16,
    pub m09e: i16,
    pub m0a0: i16,
    pub m0a2: i16,
    pub m0a4: i16,
    pub m0a6: i16,
    pub m0a8: i16,
    pub m0aa: i16,
    pub m0ac: i16,
    pub m0ae: i16,
    pub m0b0: i16,
    pub m0b2: i16,
    pub m0b4: i16,
    pub m0b6: i16,
    pub m0b8: i16,
    pub m0ba: i16,
    pub m0bc: i16,
    pub m0be: i16,
    pub m0c0: i16,
    pub m0c2: i16,
    pub m0c4: i16,
    pub m0c6: i16,
    pub m0c8: i16,
    pub m0ca: i16,
    pub m0cc: i16,
    pub m0ce: i16,
    pub m0d0: i16,
    pub m0d2: i16,
    pub m0d4: i16,
    pub m0d6: i16,
    pub m0d8: i16,
    pub m0da: i16,
    pub m0dc: i16,
    pub m0de: i16,
    pub m0e0: i16,
    pub m0e2: i16,
    pub m0e4: i16,
    pub m0e6: i16,
    pub m0e8: i16,
    pub m0ea: i16,
    pub m0ec: i16,
    pub m0ee: i16,
    pub m0f0: i16,
    pub m0f2: i16,
    pub m0f4: i16,
    pub m0f6: i16,
    pub m0f8: i16,
    pub m0fa: i16,
    pub m0fc: i16,
    pub m0fe: i16,
    pub m100: i16,
    pub m102: i16,
    pub m104: i16,
    pub m106: i16,
    pub m108: i16,
    pub m10a: i16,
    pub m10c: i16,
    pub m10e: i16,
    pub m110: i16,
    pub m112: i16,
    pub m114: i16,
    pub m116: i16,
    pub m118: i16,
    pub m11a: i16,
    pub m11c: i16,
    pub m11e: i16,
    pub m120: u8,
    pub m121: u8,
    pub m122: u8,
    pub m123: u8,
    pub m124: i16,
    pub m126: i16,
    pub m128: i16,
    pub m12a: i16,
    pub m12c: i16,
    pub m12e: i16,
    pub m130: i16,
    pub m132: i16,
    pub m134: i16,
    pub m136: i16,
    pub m138: i16,
    pub m13a: i16,
    pub m13c: i16,
    pub m13e: i16,
    pub m140: i16,
    pub m142: u8,
    pub m143: u8,
    pub m144: u8,
    pub m145: u8,
    pub m146: u8,
    pub m147: u8,
    pub m148: u8,
    pub m149: u8,
    pub m14a: u8,
    pub m14b: u8,
    pub m14c: u8,
    pub m14e: i16,
    pub m150: i16,
    pub m152: i16,
    pub m154: i16,
}

impl Default for MeterTuning {
    fn default() -> Self {
        Self {
            m006: 10,
            m00c: 1.0,
            m010: 1.4,
            m014: 0.0,
            m018: -29.0,
            m01c: 1.0,
            m020: 1.5,
            m024: 150.0,
            m028: 2.1,
            m02c: 0xff,
            m02d: 0x8c,
            m02e: 0,
            m030: 0,
            m032: 0,
            m034: 0,
            m036: 0x16,
            m038: 0,
            m03c: 0.65,
            m040: 0x5a,
            m042: 100,
            m044: 100,
            m046: 0x5a,
            m048: 1.8,
            m04c: 12.0,
            m050: 7,
            m052: -12,
            m054: 0,
            m056: -88,
            m058: 2.3,
            m05c: 12.0,
            m060: 0x96,
            m061: 100,
            m062: 0,
            m063: 0,
            m064: 0,
            m066: 0,
            m068: 0,
            m06a: 0,
            m06c: [-22, -35, -6],
            m072: [-43, -3, -13],
            m078: [0x8c, 0x82, 0x8c],
            m07e: 0x14,
            m088: 0xf,
            m08a: 5,
            m08c: 5,
            m08e: 5,
            m090: 0x14,
            m092: 0xc,
            m094: 3,
            m096: 3,
            m098: 3,
            m09a: 0,
            m09c: 0,
            m09e: 0,
            m0a0: 0,
            m0a2: 0x3c,
            m0a4: 0,
            m0a6: 0,
            m0a8: 0,
            m0aa: -60,
            m0ac: 0,
            m0ae: 0,
            m0b0: 0x19,
            m0b2: 0x23,
            m0b4: -33,
            m0b6: -6,
            m0b8: -30,
            m0ba: -22,
            m0bc: -5,
            m0be: 0,
            m0c0: 0,
            m0c2: 0,
            m0c4: 0,
            m0c6: 0,
            m0c8: -20,
            m0ca: -20,
            m0cc: -20,
            m0ce: 0,
            m0d0: 0x28,
            m0d2: 0x28,
            m0d4: 0,
            m0d6: 0,
            m0d8: -60,
            m0da: -60,
            m0dc: 0,
            m0de: 0,
            m0e0: 0x19,
            m0e2: 0x23,
            m0e4: 0,
            m0e6: -6,
            m0e8: -30,
            m0ea: -22,
            m0ec: -60,
            m0ee: 0,
            m0f0: 0,
            m0f2: 0,
            m0f4: 0,
            m0f6: 0,
            m0f8: 0,
            m0fa: 0,
            m0fc: 0,
            m0fe: 0,
            m100: 0,
            m102: 0,
            m104: 0x32,
            m106: 0x32,
            m108: -60,
            m10a: -60,
            m10c: -72,
            m10e: -81,
            m110: 0,
            m112: 0x11,
            m114: -30,
            m116: -98,
            m118: -21,
            m11a: -12,
            m11c: -11,
            m11e: 7,
            m120: 0x8c,
            m121: 200,
            m122: 0x96,
            m123: 1,
            m124: 5,
            m126: 10,
            m128: 6,
            m12a: 3,
            m12c: 3,
            m12e: 2,
            m130: 0x14,
            m132: 0x27,
            m134: 3,
            m136: 4,
            m138: 10,
            m13a: 0,
            m13c: 10,
            m13e: 0,
            m140: 0,
            m142: 0xb4,
            m143: 0,
            m144: 0,
            m145: 0xff,
            m146: 0,
            m147: 0,
            m148: 0,
            m149: 0,
            m14a: 0xff,
            m14b: 0x32,
            m14c: 0x32,
            m14e: 10,
            m150: 0x14,
            m152: 4,
            m154: 0x32,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MenuTuning {
    pub m008: f32,
    pub m00c: f32,
    pub m010: f32,
    pub m014: f32,
    pub m018: f32,
    pub m01c: f32,
    pub m020: f32,
    pub m024: f32,
    pub m028: i16,
    pub m02a: i16,
    pub m02c: i16,
    pub m02e: i16,
    pub m030: i16,
    pub m032: i16,
    pub m034: i16,
    pub m036: i16,
    pub m038: i16,
    pub m03a: i16,
    pub m03c: f32,
    pub m040: f32,
    pub m044: f32,
    pub m048: f32,
    pub m04c: f32,
    pub m050: f32,
    pub m054: f32,
    pub m058: f32,
    pub m05c: f32,
    pub m060: f32,
    pub m064: f32,
    pub m068: f32,
    pub m06c: u8,
    pub m06d: u8,
    pub m06e: u8,
    pub m06f: u8,
    pub m070: u8,
    pub m071: u8,
    pub m072: u8,
    pub m073: u8,
    pub m074: u8,
    pub m075: u8,
    pub m076: u8,
    pub m077: u8,
    pub m078: u8,
    pub m079: u8,
    pub m07a: u8,
    pub m07b: u8,
    pub m07c: i16,
    pub m07e: i16,
    pub m080: i16,
    pub m082: i16,
    pub m084: i16,
    pub m088: i16,
    pub m08a: i16,
    pub m08c: i16,
    pub m08e: i16,
    pub m090: i16,
    pub m092: i16,
    pub m094: f32,
    pub m098: u8,
    pub m099: u8,
    pub m09a: i16,
    pub m0a0: f32,
    pub m0a4: i16,
    pub m0a6: i16,
    pub m0a8: i16,
    pub m0aa: i16,
    pub m0ac: i16,
    pub m0ae: i16,
    pub m0b0: i16,
    pub m0b2: i16,
    pub m0b4: i16,
    pub m0b6: i16,
    pub m0b8: u8,
    pub m0ba: i16,
    pub m0bc: i16,
    pub m0be: u8,
    pub m0bf: u8,
    pub m0c0: u8,
    pub m0c1: u8,
    pub m0c2: u8,
    pub m0c3: u8,
    pub m0c4: u8,
    pub m0c5: u8,
    pub m0c6: u8,
    pub m0c7: u8,
    pub m0c8: u8,
    pub m0c9: u8,
    pub m0ca: u8,
}

impl Default for MenuTuning {
    fn default() -> Self {
        Self {
            m008: 1.4,
            m00c: 18.0,
            m010: 28.0,
            m014: 1.3,
            m018: 0.25,
            m01c: 2.5,
            m020: 0.5,
            m024: 1.4,
            m028: -20,
            m02a: 0x10,
            m02c: 0x18,
            m02e: 0x17,
            m030: 0,
            m032: 8,
            m034: 5,
            m036: 7,
            m038: 0x1a,
            m03a: -3,
            m03c: 0.0,
            m040: -50.0,
            m044: 1100.0,
            m048: 0.0,
            m04c: 0.0,
            m050: -1200.0,
            m054: -1500.0,
            m058: 1000.0,
            m05c: 500.0,
            m060: 1.0,
            m064: -1.0,
            m068: -1.0,
            m06c: 0x32,
            m06d: 0x32,
            m06e: 0x32,
            m06f: 0xff,
            m070: 0xff,
            m071: 0xff,
            m072: 0xff,
            m073: 0xff,
            m074: 0xff,
            m075: 0xff,
            m076: 0xff,
            m077: 0xff,
            m078: 200,
            m079: 200,
            m07a: 200,
            m07b: 0xff,
            m07c: 0,
            m07e: 3,
            m080: 3,
            m082: 8,
            m084: 5,
            m088: 3,
            m08a: 0,
            m08c: -200,
            m08e: 0x15e,
            m090: 3,
            m092: 7,
            m094: 1.0,
            m098: 0xff,
            m099: 0xff,
            m09a: 0,
            m0a0: 1.0,
            m0a4: 6,
            m0a6: 2,
            m0a8: 2,
            m0aa: 2,
            m0ac: 2,
            m0ae: 0x1e,
            m0b0: 0x3c,
            m0b2: 0,
            m0b4: 6,
            m0b6: 0x46,
            m0b8: 0x96,
            m0ba: 5,
            m0bc: 3,
            m0be: 0,
            m0bf: 0,
            m0c0: 1,
            m0c1: 0,
            m0c2: 10,
            m0c3: 4,
            m0c4: 0,
            m0c5: 2,
            m0c6: 0,
            m0c7: 1,
            m0c8: 9,
            m0c9: 0,
            m0ca: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MsgTuning {
    pub m005: TColor,
    pub m009: TColor,
    pub m00d: u8,
    pub m00e: u8,
    pub m00f: u8,
    pub m010: u8,
    pub m011: u8,
    pub m012: u8,
    pub m013: u8,
    pub m014: u8,
    pub m015: u8,
    pub m016: u8,
    pub m017: u8,
    pub m018: u8,
    pub m019: u8,
    pub m01a: u8,
    pub m01b: u8,
    pub m01c: u8,
    pub m01d: u8,
    pub m01e: u8,
    pub m01f: u8,
    pub m020: u8,
    pub m021: u8,
    pub m022: u8,
    pub m023: u8,
    pub m024: u8,
    pub m025: u8,
    pub m026: u8,
    pub m027: u8,
    pub m028: u8,
    pub m029: u8,
    pub m02a: u8,
    pub m02b: u8,
    pub m02c: u8,
    pub m02d: u8,
    pub m02e: u8,
    pub m02f: u8,
    pub m030: u8,
    pub m038: f32,
    pub m03c: f32,
    pub m040: f32,
    pub m044: i16,
    pub m046: i16,
    pub m048: i16,
    pub m04a: i16,
    pub m04c: i16,
    pub m050: f32,
    pub m054: f32,
    pub m058: u8,
    pub m05a: i16,
    pub m05c: i16,
    pub m05e: i16,
    pub m060: i16,
    pub m062: i16,
    pub m064: u8,
    pub m066: i16,
    pub m068: i16,
    pub m06a: i16,
    pub m06c: i16,
    pub m06e: i16,
    pub m070: i16,
    pub m072: u8,
    pub m074: i16,
    pub m076: i16,
    pub m078: i16,
    pub m07a: i16,
    pub m07c: i16,
    pub m07e: u8,
    pub m07f: u8,
    pub m080: u8,
    pub m081: u8,
    pub m082: u8,
    pub m083: u8,
    pub m084: u8,
    pub m085: u8,
    pub m086: u8,
    pub m087: u8,
    pub m088: u8,
    pub m089: u8,
    pub m08a: u8,
}

impl Default for MsgTuning {
    fn default() -> Self {
        Self {
            m005: TColor::new(0x1e, 0x1e, 0x1e, 0xd7),
            m009: TColor::new(0x1e, 0x1e, 0x4b, 0x00),
            m00d: 0x1e,
            m00e: 0x1e,
            m00f: 0x1e,
            m010: 0xdc,
            m011: 0x1e,
            m012: 0x1e,
            m013: 0xb4,
            m014: 0,
            m015: 0,
            m016: 0xff,
            m017: 0,
            m018: 0xff,
            m019: 0x78,
            m01a: 0x78,
            m01b: 0xff,
            m01c: 0xff,
            m01d: 0xff,
            m01e: 0xff,
            m01f: 0x3c,
            m020: 0xff,
            m021: 0,
            m022: 0xff,
            m023: 0xff,
            m024: 0xff,
            m025: 0xff,
            m026: 0,
            m027: 0xff,
            m028: 0xff,
            m029: 0xff,
            m02a: 0x50,
            m02b: 0x50,
            m02c: 0x96,
            m02d: 0xff,
            m02e: 0x96,
            m02f: 0x96,
            m030: 0xff,
            m038: -40.0,
            m03c: 0.96,
            m040: 0.5,
            m044: 5,
            m046: 3,
            m048: 4,
            m04a: 2,
            m04c: 4,
            m050: 0.6,
            m054: 1.1,
            m058: 0x1d,
            m05a: 0,
            m05c: 0,
            m05e: 0x1e,
            m060: 0xd,
            m062: 0,
            m064: 0,
            m066: 0,
            m068: 0xf,
            m06a: 0,
            m06c: 0,
            m06e: 0xf0,
            m070: 0x17,
            m072: 0,
            m074: 0x80,
            m076: 10,
            m078: 0,
            m07a: 0,
            m07c: 0x46,
            m07e: 10,
            m07f: 0xaa,
            m080: 0xb4,
            m081: 0x46,
            m082: 2,
            m083: 0,
            m084: 1,
            m085: 0,
            m086: 0,
            m087: 10,
            m088: 0,
            m089: 0,
            m08a: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MessageTuning {
    pub m005: TColor,
    pub m009: TColor,
    pub m00d: TColor,
    pub m011: TColor,
    pub m015: TColor,
    pub m019: TColor,
    pub m01d: TColor,
    pub m021: TColor,
    pub m025: TColor,
    pub m029: u8,
    pub m02a: u8,
    pub m02b: u8,
    pub m02c: u8,
    pub m02d: u8,
    pub m02e: u8,
    pub m02f: u8,
    pub m030: u8,
    pub m031: u8,
    pub m032: u8,
    pub m034: i16,
    pub m036: i16,
    pub m038: i16,
    pub m03a: u8,
    pub m03b: u8,
    pub m03c: i16,
    pub m040: f32,
    pub m044: u8,
    pub m046: i16,
    pub m048: i16,
    pub m04a: u8,
    pub m04b: u8,
    pub m04c: u8,
    pub m04d: u8,
    pub m04e: u8,
    pub m04f: u8,
    pub m050: u8,
    pub m051: u8,
    pub m052: u8,
    pub m053: u8,
    pub m054: i16,
    pub m056: i16,
    pub m058: i16,
    pub m05a: u8,
    pub m05b: u8,
    pub m05c: u8,
    pub m05d: u8,
    pub m05e: u8,
    pub m060: i16,
    pub m062: i16,
    pub m064: i16,
    pub m066: i16,
    pub m068: i16,
    pub m06a: i16,
    pub m06c: i16,
    pub m06e: i16,
    pub m070: i16,
    pub m072: i16,
    pub m074: i16,
    pub m076: i16,
    pub m078: u8,
    pub m07a: i16,
    pub m07c: i16,
    pub m07e: i16,
    pub m080: i16,
    pub m082: i16,
}

impl Default for MessageTuning {
    fn default() -> Self {
        Self {
            m005: TColor::new(0, 0, 0, 0xff),
            m009: TColor::new(0xff, 0xff, 0xff, 0xff),
            m00d: TColor::new(0xff, 0xff, 0xff, 0x00),
            m011: TColor::new(0xff, 0xff, 0xff, 0xff),
            m015: TColor::new(0xff, 0xff, 0xff, 0xff),
            m019: TColor::new(0xff, 0xff, 0xff, 0x00),
            m01d: TColor::new(0, 0, 0, 0xff),
            m021: TColor::new(0xff, 0xff, 0xff, 0xff),
            m025: TColor::new(0xff, 0xff, 0xff, 0x00),
            m029: 0x80,
            m02a: 0xff,
            m02b: 0xff,
            m02c: 0xff,
            m02d: 0xff,
            m02e: 0xb4,
            m02f: 0xa0,
            m030: 0x4b,
            m031: 0,
            m032: 0x1b,
            m034: -2,
            m036: 0x2a,
            m038: 0xf,
            m03a: 1,
            m03b: 10,
            m03c: 0x16,
            m040: 0.7,
            m044: 0,
            m046: 1,
            m048: 0,
            m04a: 0,
            m04b: 1,
            m04c: 0xff,
            m04d: 0xff,
            m04e: 0xff,
            m04f: 0xff,
            m050: 0,
            m051: 0,
            m052: 0,
            m053: 0,
            m054: 0,
            m056: 0,
            m058: 0,
            m05a: 0,
            m05b: 0xff,
            m05c: 0,
            m05d: 0,
            m05e: 0,
            m060: 0,
            m062: 0,
            m064: 0,
            m066: 0,
            m068: 0,
            m06a: 0,
            m06c: 1,
            m06e: 1,
            m070: 1,
            m072: 1,
            m074: 1,
            m076: 1,
            m078: 0,
            m07a: 4,
            m07c: 4,
            m07e: 2,
            m080: 600,
            m082: 0x18c,
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct MeterGlobals {
    pub map_alpha: u8,
    pub menu_status: u8,
    pub menu_status_old: u8,
    pub pause_flag: u8,
    pub frame_timer: i32,
    pub menu_button: u8,
    pub sub_win_flag: bool,
    pub auction_flag: bool,
    pub item_move_flag: u8,
    pub button_check: u8,
    pub item_number: u8,
    pub item_timer: i16,
    pub tex_rate_x: f32,
    pub tex_rate_y: f32,
    pub tex_scale_x: f32,
    pub tex_scale_y: f32,
    pub menu_flag: u8,
    pub bait_num: [u8; BAIT_COUNT],
    pub place_name: u8,
    pub item_mode: u8,
    pub collect_mode: u8,
    pub wind_id: i32,
    pub wind_status: u8,
    pub rupy_sound_set_flag: u8,
    pub rupy_sound_on_flag: u8,
    pub item_parts: MsgPane,
    pub info: MeterInfo,
    pub tuning: MeterTuning,
    pub menu_tuning: MenuTuning,
    pub msg_tuning: MsgTuning,
    pub message_tuning: MessageTuning,
    pub button_action_mode: u8,
    pub sounds: Vec<SoundCue>,
}

impl Default for MeterGlobals {
    fn default() -> Self {
        Self {
            map_alpha: 0,
            menu_status: 0,
            menu_status_old: 0,
            pause_flag: 0,
            frame_timer: 0,
            menu_button: 0,
            sub_win_flag: false,
            auction_flag: false,
            item_move_flag: 0,
            button_check: 0,
            item_number: 0,
            item_timer: 0,
            tex_rate_x: 0.0,
            tex_rate_y: 0.0,
            tex_scale_x: 0.0,
            tex_scale_y: 0.0,
            menu_flag: 0,
            bait_num: [0; BAIT_COUNT],
            place_name: 0,
            item_mode: 0,
            collect_mode: 0,
            wind_id: -1,
            wind_status: 0,
            rupy_sound_set_flag: 0,
            rupy_sound_on_flag: 0,
            item_parts: MsgPane::default(),
            info: MeterInfo::default(),
            tuning: MeterTuning::default(),
            menu_tuning: MenuTuning::default(),
            msg_tuning: MsgTuning::default(),
            message_tuning: MessageTuning::default(),
            button_action_mode: 0,
            sounds: Vec::new(),
        }
    }
}

impl MeterGlobals {
    pub fn set_menu_status(&mut self, value: u8) {
        self.menu_status = value;
    }

    pub fn set_menu_status_old(&mut self, value: u8) {
        self.menu_status_old = value;
    }

    pub fn menu_status(&self) -> u8 {
        self.menu_status
    }

    pub fn collect_mode(&self) -> u8 {
        self.collect_mode
    }

    pub fn set_collect_mode(&mut self, value: u8) {
        self.collect_mode = value;
    }

    pub fn item_mode(&self) -> u8 {
        self.item_mode
    }

    pub fn set_item_mode(&mut self, value: u8) {
        self.item_mode = value;
    }

    pub fn sub_win_flag(&self) -> bool {
        self.sub_win_flag
    }

    pub fn sub_win_flag_on(&mut self) {
        self.sub_win_flag = true;
    }

    pub fn sub_win_flag_off(&mut self) {
        self.sub_win_flag = false;
    }

    pub fn is_auction_flag(&self) -> bool {
        self.auction_flag
    }

    pub fn on_auction_flag(&mut self) {
        self.auction_flag = true;
    }

    pub fn off_auction_flag(&mut self) {
        self.auction_flag = false;
    }

    pub fn item_move_set(&mut self, pane: &MsgPane, btn: u8, item: u8) {
        self.item_move_flag = 1;
        self.button_check = btn;
        self.item_number = item;
        self.item_parts.pos_top_left_orig = pane.pos_top_left;
        self.item_parts.pos_top_left = self.item_parts.pos_top_left_orig;
        self.item_parts.pos_center_orig = pane.pos_center;
        self.item_parts.pos_center = self.item_parts.pos_center_orig;
        self.item_parts.size_orig = pane.size_orig;
        self.item_parts.size = pane.size;
        self.item_timer = 0;
    }

    pub fn item_move_flag_check(&self) -> u8 {
        self.item_move_flag
    }

    pub fn flag(&self) -> u8 {
        self.pause_flag
    }

    pub fn flag_set(&mut self, value: u8) {
        self.pause_flag = value;
    }

    pub fn timer(&self) -> i32 {
        self.frame_timer
    }

    pub fn push_menu_button(&self) -> u8 {
        self.menu_button
    }

    pub fn set_push_menu_button(&mut self, value: u8) {
        self.menu_button = value;
    }
}

#[derive(Resource, Clone, Debug)]
pub struct VitalsEnv {
    pub show_2d: bool,
    pub camera_flags: u32,
    pub camera_attention: u16,
    pub message_status: u8,
    pub scope_message_status: u8,
    pub tmp_bit_0408: bool,
    pub event_start_name: Option<String>,
    pub event_running: bool,
    pub event_hind: u16,
    pub operate_wind: u8,
    pub mini_game_type: u8,
    pub demo_mode: u8,
    pub metronome: bool,
    pub picture_status: u8,
    pub heap_lock_flag: u8,
    pub player_status0: u32,
    pub player_status1: u32,
    pub c_b1_is_player: bool,
    pub hyoi_singer_present: bool,
    pub rope_tag: bool,
    pub soup_power_up: bool,
    pub stage_type: u8,
    pub stage_boss_enemy: bool,
    pub stage_save_tbl: u8,
    pub boss_recollect_done: bool,
    pub start_stage_name: String,
    pub up_button: i8,
    pub battle_life_point: u8,
    pub mini_game_rest_arrow: Option<i32>,
    pub a_status: u8,
    pub a_status_force: u8,
    pub do_status: u8,
    pub do_status_force: u8,
    pub r_status: u8,
    pub r_status_force: u8,
    pub life: u8,
    pub max_life: u8,
    pub item_now_life: u16,
    pub item_max_life_count: i16,
    pub item_life_count: f32,
    pub item_picture_num_count: i16,
    pub item_arrow_num_count: i32,
    pub item_bomb_num_count: i32,
    pub item_beast_num_count: [i16; BEAST_COUNT],
    pub arrow_num: u8,
    pub arrow_max: u8,
    pub bomb_num: u8,
    pub bomb_max: u8,
    pub picture_num: u8,
    pub beast_num: [u8; BEAST_COUNT],
    pub bait_num: [u8; BAIT_COUNT],
    pub select_item: [u8; ITEM_BUTTON_COUNT],
    pub select_slot: [u8; ITEM_BUTTON_COUNT],
    pub equipped: Equipped,
    pub collect: Collect,
    pub beast_bag: [u8; BAG_COUNT],
    pub bait_bag: [u8; BAG_COUNT],
    pub reserve_bag: [u8; RESERVE_COUNT],
    pub item_slot: u8,
    pub item_no: u8,
}

impl Default for VitalsEnv {
    fn default() -> Self {
        Self {
            show_2d: false,
            camera_flags: 0,
            camera_attention: 0,
            message_status: 0,
            scope_message_status: 0,
            tmp_bit_0408: false,
            event_start_name: None,
            event_running: false,
            event_hind: 0,
            operate_wind: 0,
            mini_game_type: 0,
            demo_mode: 0,
            metronome: false,
            picture_status: 0,
            heap_lock_flag: 0,
            player_status0: 0,
            player_status1: 0,
            c_b1_is_player: true,
            hyoi_singer_present: false,
            rope_tag: false,
            soup_power_up: false,
            stage_type: stage::TYPE_NONE,
            stage_boss_enemy: false,
            stage_save_tbl: 0,
            boss_recollect_done: false,
            start_stage_name: String::new(),
            up_button: 0,
            battle_life_point: 0,
            mini_game_rest_arrow: None,
            a_status: act::BLANK,
            a_status_force: act::BLANK,
            do_status: act::BLANK,
            do_status_force: act::BLANK,
            r_status: act::BLANK,
            r_status_force: act::BLANK,
            life: 0,
            max_life: 0,
            item_now_life: 0,
            item_max_life_count: 0,
            item_life_count: 0.0,
            item_picture_num_count: 0,
            item_arrow_num_count: 0,
            item_bomb_num_count: 0,
            item_beast_num_count: [0; BEAST_COUNT],
            arrow_num: 0,
            arrow_max: 0,
            bomb_num: 0,
            bomb_max: 0,
            picture_num: 0,
            beast_num: [0; BEAST_COUNT],
            bait_num: [0; BAIT_COUNT],
            select_item: [it::NONE; ITEM_BUTTON_COUNT],
            select_slot: [slot::NONE; ITEM_BUTTON_COUNT],
            equipped: Equipped::default(),
            collect: Collect::default(),
            beast_bag: [it::NONE; BAG_COUNT],
            bait_bag: [it::NONE; BAG_COUNT],
            reserve_bag: [it::NONE; RESERVE_COUNT],
            item_slot: 0,
            item_no: 0,
        }
    }
}

impl VitalsEnv {
    pub fn show_2d_on(&mut self) {
        self.show_2d = true;
    }

    pub fn show_2d_off(&mut self) {
        self.show_2d = false;
    }

    pub fn set_a_status(&mut self, status: u8) {
        self.a_status = status;
    }

    pub fn set_a_status_force(&mut self, status: u8) {
        self.a_status_force = status;
    }

    pub fn set_do_status(&mut self, status: u8) {
        self.do_status = status;
    }

    pub fn set_do_status_force(&mut self, status: u8) {
        self.do_status_force = status;
    }

    pub fn set_r_status(&mut self, status: u8) {
        self.r_status = status;
    }

    pub fn set_r_status_force(&mut self, status: u8) {
        self.r_status_force = status;
    }

    pub fn check_player_status0(&self, flag: u32) -> bool {
        self.player_status0 & flag != 0
    }

    pub fn check_player_status1(&self, flag: u32) -> bool {
        self.player_status1 & flag != 0
    }

    pub fn check_hind(&self, flag: u16) -> bool {
        if !self.event_running {
            return false;
        }
        self.event_hind & flag != 0
    }

    pub fn check_camera_flag(&self, flag: u32) -> bool {
        self.camera_flags & flag != 0
    }

    pub fn is_collect(&self, idx: u8, item: u8) -> bool {
        self.collect.test(idx, item)
    }

    pub fn item(&self, slot_index: u8) -> u8 {
        match slot_index {
            0..=20 => self.equipped.item(usize::from(slot_index)),
            24..=31 => self.beast_bag[usize::from(slot_index - slot::BEAST_FIRST)],
            36..=43 => self.bait_bag[usize::from(slot_index - slot::BAIT_FIRST)],
            48..=55 => self.reserve_bag[usize::from(slot_index - slot::RESERVE_FIRST)],
            _ => 0xFF,
        }
    }

    pub fn set_item(&mut self, slot_index: u8, item_no: u8) {
        match slot_index {
            0..=20 => self.equipped.set(usize::from(slot_index), item_no),
            24..=31 => self.beast_bag[usize::from(slot_index - slot::BEAST_FIRST)] = item_no,
            36..=43 => self.bait_bag[usize::from(slot_index - slot::BAIT_FIRST)] = item_no,
            48..=55 => self.reserve_bag[usize::from(slot_index - slot::RESERVE_FIRST)] = item_no,
            _ => {}
        }
    }

    pub fn check_beast_item(&self, item_no: u8) -> bool {
        self.beast_bag.contains(&item_no)
    }

    pub fn set_beast_item_empty(&mut self, item_no: u8) {
        if !self.check_beast_item(item_no) {
            return;
        }
        for index in 0..BAG_COUNT {
            if self.beast_bag[index] == item_no {
                self.beast_bag[index] = it::NONE;
                for btn in 0..ITEM_BUTTON_COUNT {
                    if self.select_item[btn] == item_no {
                        let slot_index = self.select_slot[btn];
                        self.set_item(slot_index, it::NONE);
                        self.item_slot = slot_index;
                        self.item_no = it::NONE;
                        self.set_select_item(btn);
                    }
                }
                return;
            }
        }
    }

    pub fn set_select_item(&mut self, btn: usize) {
        if self.select_slot[btn] != slot::NONE {
            let inv = self.select_slot[btn];
            let item_no = self.item(inv);
            self.select_item[btn] = item_no;
            let inv = self.select_slot[btn];
            let item_no = self.item(inv);
            if item_no == it::NONE {
                self.select_slot[btn] = slot::NONE;
            }
        } else {
            self.select_item[btn] = it::NONE;
        }
    }

    pub fn has_item(&self, item_no: u8) -> bool {
        check_get_item(&self.equipped, &self.collect, item_no)
    }
}

pub fn meter_show(env: &mut VitalsEnv) {
    env.show_2d_on();
}

pub fn meter_hide(env: &mut VitalsEnv) {
    env.show_2d_off();
}

pub fn recollect_boss_data(env: &VitalsEnv) -> Option<RecollectBoss> {
    if env.stage_type == stage::TYPE_BOSS && env.stage_boss_enemy {
        let bit = match env.stage_save_tbl {
            stage::SAVE_DRC => 0x3D80,
            stage::SAVE_FW => 0x3D40,
            stage::SAVE_ET => 0x3D20,
            stage::SAVE_WT => 0x3D10,
            _ => return None,
        };
        if !env.boss_recollect_done {
            return Some(RecollectBoss {
                copy: true,
                event_bit: bit,
            });
        }
    }
    None
}

pub fn loaded_screen<'a>(screens: &'a Screens, label: &str) -> Option<&'a Screen> {
    screens
        .cache
        .iter()
        .find(|(name, _)| name == label)
        .map(|(_, screen)| screen)
}

pub fn meter1_draw(screens: &Screens) -> Vec<ScreenLayer> {
    let mut list = vec![ScreenLayer {
        label: String::from(PARTS3),
        scissor: false,
    }];
    list.push(ScreenLayer {
        label: String::from(PARTS1),
        scissor: false,
    });
    if loaded_screen(screens, SWIM1).is_some() {
        list.push(ScreenLayer {
            label: String::from(SWIM1),
            scissor: true,
        });
    }
    if loaded_screen(screens, SWIM2).is_some() {
        list.push(ScreenLayer {
            label: String::from(SWIM2),
            scissor: false,
        });
    }
    list.push(ScreenLayer {
        label: String::from(CHOICE_ROAD),
        scissor: false,
    });
    list
}

pub fn meter2_draw(screens: &Screens) -> Vec<ScreenLayer> {
    if loaded_screen(screens, PARTS2).is_none() {
        return Vec::new();
    }
    vec![ScreenLayer {
        label: String::from(PARTS2),
        scissor: false,
    }]
}

fn blend(base: f32, target: f32, progress: f32) -> u8 {
    (base - ((base - target) * progress)) as i64 as u8
}

pub fn set_heart_scale_xy(pane: &mut MsgPane, scale: i16, rate_x: f32, rate_y: f32) {
    let (scale_x, scale_y) = if scale != 4 {
        (1.0, 1.0)
    } else {
        (rate_x, rate_y)
    };
    pane.size.x = pane.size_orig.x * scale_x;
    pane.size.y = pane.size_orig.y * scale_y;
    cpos_move(pane);
}

pub fn set_heart_scale_xy2(pane: &mut MsgPane, scale: i16, rate_x: f32, rate_y: f32, factor: f32) {
    let (scale_x, scale_y) = if scale != 4 {
        (1.0, 1.0)
    } else {
        (rate_x, rate_y)
    };
    pane.size.x = pane.size_orig.x * scale_x * factor;
    pane.size.y = pane.size_orig.y * scale_y * factor;
    cpos_move(pane);
}

pub fn set_now_heart_scale_xy(pane: &mut MsgPane, factor: f32) {
    pane.size.x = pane.size_orig.x * factor;
    pane.size.y = pane.size_orig.y * factor;
    cpos_move(pane);
}

pub fn set_now_heart_scale_xy2(pane: &mut MsgPane, first: f32, second: f32) {
    pane.size.x = pane.size_orig.x * first * second;
    pane.size.y = pane.size_orig.y * first * second;
    cpos_move(pane);
}

pub fn heart_data_set(flash: &mut MsgPane, heart: &mut MsgPane, shadow: &mut MsgPane, factor: f32) {
    set_now_heart_scale_xy(heart, factor);
    set_now_heart_scale_xy(shadow, factor);
    flash.pos_top_left = heart.pos_top_left;
    flash.size = heart.size;
    flash.pos_center = heart.pos_center;
    flash
        .node
        .move_to(flash.pos_top_left.x, flash.pos_top_left.y);
    flash.node.resize(flash.size.x, flash.size.y);
}

pub fn heart_change_texture(heart: &mut MsgPane, shadow: &mut MsgPane, texture_name: &str) {
    heart.node.change_texture(texture_name);
    shadow.node.change_texture(texture_name);
    pane_show(heart);
    pane_show(shadow);
}

pub fn xy_rotate_z(pane: &mut MsgPane, first: f32, second: f32) {
    let x = pane.size.x * 0.5;
    let y = pane.size.y * 0.5;
    pane.node.rotate(x, y, ROTATE_Z, first + second);
}

#[derive(Resource, Clone, Debug)]
pub struct Vitals {
    pub globals: MeterGlobals,
    pub heart: [MsgPane; HEART_COUNT],
    pub heart_shadow: [MsgPane; HEART_COUNT],
    pub heart_flash: MsgPane,
    pub magic_bar: [MsgPane; 8],
    pub wepon_blink: MsgPane,
    pub action_blink: MsgPane,
    pub action_text_shadow: [MsgPane; 2],
    pub action_text: [MsgPane; 2],
    pub count_tens: MsgPane,
    pub count_ones: MsgPane,
    pub action_text_main: MsgPane,
    pub action_text_main_key: MsgPane,
    pub wepon_text: MsgPane,
    pub wepon_text_key: MsgPane,
    pub wepon_back: MsgPane,
    pub wepon_add: MsgPane,
    pub center: MsgPane,
    pub xy_count_tens: [MsgPane; ITEM_BUTTON_COUNT],
    pub xy_count_ones: [MsgPane; ITEM_BUTTON_COUNT],
    pub xy_item: [MsgPane; ITEM_BUTTON_COUNT],
    pub xy_item_key: [MsgPane; ITEM_BUTTON_COUNT],
    pub xy_bow_light: [MsgPane; ITEM_BUTTON_COUNT],
    pub xy_bow_light_2: [MsgPane; ITEM_BUTTON_COUNT],
    pub xy_bow_light_3: [MsgPane; ITEM_BUTTON_COUNT],
    pub xy_arrow_back: [MsgPane; 2],
    pub xy_arrow_00: [MsgPane; 2],
    pub xy_arrow_02: [MsgPane; 2],
    pub xy_arrow: MsgPane,
    pub status_flags: u32,
    pub fade_bits: u8,
    pub hide_bits: u8,
    pub last_a_status: u8,
    pub last_do_status: u8,
    pub select_cache: [u8; ITEM_BUTTON_COUNT],
    pub screen_alpha: f32,
    pub screen_alpha_frame: i16,
    pub heart_scale: f32,
    pub heart_alpha: f32,
    pub heart_alpha_frame: i16,
    pub max_hp: u8,
    pub curr_hp: u8,
    pub adjust_hp: i16,
    pub max_hp_adjust: i16,
    pub action_tex: [TexSlot; 3],
    pub xy_icon_tex: [TexSlot; ITEM_BUTTON_COUNT],
    pub rupy_light: [Option<RupyEmitter>; 2],
    pub battle_mode: bool,
    pub battle_point_cached: u8,
    pub life_sound: bool,
    pub wepon_move_frame: i16,
    pub wepon_move_flag: i16,
    pub wepon_move_status: i16,
    pub wepon_now_x: f32,
    pub wepon_now_y: f32,
    pub wepon_exp_x: f32,
    pub wepon_exp_y: f32,
    pub wepon_alpha_frame: i16,
    pub action_move_frame: i16,
    pub action_move_flag: i16,
    pub action_move_status: i16,
    pub action_now_x: f32,
    pub action_now_y: f32,
    pub action_exp_x: f32,
    pub action_exp_y: f32,
    pub action_alpha_frame: i16,
    pub xy_move_frame: i16,
    pub xy_move_flag: i16,
    pub xy_move_status: i16,
    pub xy_now_x: f32,
    pub xy_now_y: f32,
    pub xy_alpha_frame: [i16; ITEM_BUTTON_COUNT],
    pub arrow_max_cached: Option<u8>,
    pub bomb_max_cached: Option<u8>,
    pub bind_pending: bool,
    pub bound: bool,
}

impl Default for Vitals {
    fn default() -> Self {
        Self {
            globals: MeterGlobals::default(),
            heart: Default::default(),
            heart_shadow: Default::default(),
            heart_flash: MsgPane::default(),
            magic_bar: Default::default(),
            wepon_blink: MsgPane::default(),
            action_blink: MsgPane::default(),
            action_text_shadow: Default::default(),
            action_text: Default::default(),
            count_tens: MsgPane::default(),
            count_ones: MsgPane::default(),
            action_text_main: MsgPane::default(),
            action_text_main_key: MsgPane::default(),
            wepon_text: MsgPane::default(),
            wepon_text_key: MsgPane::default(),
            wepon_back: MsgPane::default(),
            wepon_add: MsgPane::default(),
            center: MsgPane::default(),
            xy_count_tens: Default::default(),
            xy_count_ones: Default::default(),
            xy_item: Default::default(),
            xy_item_key: Default::default(),
            xy_bow_light: Default::default(),
            xy_bow_light_2: Default::default(),
            xy_bow_light_3: Default::default(),
            xy_arrow_back: Default::default(),
            xy_arrow_00: Default::default(),
            xy_arrow_02: Default::default(),
            xy_arrow: MsgPane::default(),
            status_flags: 0,
            fade_bits: 0,
            hide_bits: 0,
            last_a_status: 0,
            last_do_status: 0,
            select_cache: [0; ITEM_BUTTON_COUNT],
            screen_alpha: 0.0,
            screen_alpha_frame: 0,
            heart_scale: 0.0,
            heart_alpha: 0.0,
            heart_alpha_frame: 0,
            max_hp: 0,
            curr_hp: 0,
            adjust_hp: 0,
            max_hp_adjust: 0,
            action_tex: [TexSlot::new(), TexSlot::new(), TexSlot::new()],
            xy_icon_tex: [TexSlot::new(), TexSlot::new(), TexSlot::new()],
            rupy_light: [None, None],
            battle_mode: false,
            battle_point_cached: 0,
            life_sound: false,
            wepon_move_frame: 0,
            wepon_move_flag: 0,
            wepon_move_status: 0,
            wepon_now_x: 0.0,
            wepon_now_y: 0.0,
            wepon_exp_x: 0.0,
            wepon_exp_y: 0.0,
            wepon_alpha_frame: 0,
            action_move_frame: 0,
            action_move_flag: 0,
            action_move_status: 0,
            action_now_x: 0.0,
            action_now_y: 0.0,
            action_exp_x: 0.0,
            action_exp_y: 0.0,
            action_alpha_frame: 0,
            xy_move_frame: 0,
            xy_move_flag: 0,
            xy_move_status: 0,
            xy_now_x: 0.0,
            xy_now_y: 0.0,
            xy_alpha_frame: [0; ITEM_BUTTON_COUNT],
            arrow_max_cached: None,
            bomb_max_cached: None,
            bind_pending: true,
            bound: false,
        }
    }
}

impl Vitals {
    pub fn bind_screens(&mut self, screens: &Screens) -> bool {
        let Some(parts1) = loaded_screen(screens, PARTS1) else {
            return false;
        };
        for index in 0..HEART_COUNT {
            set_pane_data(&mut self.heart[index], parts1, &format!("ht{index:02}"));
            set_pane_data(
                &mut self.heart_shadow[index],
                parts1,
                &format!("hk{index:02}"),
            );
        }
        set_pane_data(&mut self.heart_flash, parts1, "htfl");
        for index in 0..8 {
            set_pane_data(
                &mut self.magic_bar[index],
                parts1,
                &format!("mbr{}", index + 1),
            );
        }
        set_pane_data(&mut self.wepon_blink, parts1, "bawd");
        set_pane_data(&mut self.action_blink, parts1, "bawp");
        set_pane_data(&mut self.action_text_shadow[0], parts1, "baa");
        set_pane_data(&mut self.action_text_shadow[1], parts1, "baa2");
        set_pane_data(&mut self.action_text[0], parts1, "bawe");
        set_pane_data(&mut self.action_text[1], parts1, "bat2");
        set_pane_data(&mut self.count_tens, parts1, "a10");
        set_pane_data(&mut self.count_ones, parts1, "a01");
        set_pane_data(&mut self.action_text_main, parts1, "yum");
        set_pane_data(&mut self.action_text_main_key, parts1, "yumk");
        set_pane_data(&mut self.wepon_text, parts1, "weit");
        set_pane_data(&mut self.wepon_text_key, parts1, "witk");
        set_pane_data(&mut self.wepon_back, parts1, "bab");
        set_pane_data(&mut self.wepon_add, parts1, "baat");
        set_pane_data(&mut self.center, parts1, "cent");
        set_pane_data(&mut self.xy_arrow, parts1, "baz");
        set_pane_data(&mut self.xy_arrow_back[0], parts1, "bax1");
        set_pane_data(&mut self.xy_arrow_back[1], parts1, "bay");
        set_pane_data(&mut self.xy_arrow_00[0], parts1, "b001");
        set_pane_data(&mut self.xy_arrow_00[1], parts1, "b002");
        set_pane_data(&mut self.xy_arrow_02[0], parts1, "ba01");
        set_pane_data(&mut self.xy_arrow_02[1], parts1, "ba02");
        let tags = ["x", "y", "z"];
        for btn in 0..ITEM_BUTTON_COUNT {
            let tag = tags[btn];
            set_pane_data(&mut self.xy_count_ones[btn], parts1, &format!("{tag}01"));
            set_pane_data(&mut self.xy_count_tens[btn], parts1, &format!("{tag}10"));
            set_pane_data(&mut self.xy_item[btn], parts1, &format!("{tag}itm"));
            set_pane_data(&mut self.xy_item_key[btn], parts1, &format!("{tag}itk"));
            set_pane_data(&mut self.xy_bow_light[btn], parts1, &format!("bl{tag}1"));
            set_pane_data(&mut self.xy_bow_light_2[btn], parts1, &format!("bl{tag}2"));
            set_pane_data(&mut self.xy_bow_light_3[btn], parts1, &format!("bl{tag}3"));
        }
        self.bound = true;
        self.bind_pending = false;
        true
    }

    pub fn create(&mut self, env: &mut VitalsEnv) {
        env.set_do_status_force(act::BLANK);
        env.set_r_status_force(act::BLANK);
        env.set_a_status_force(act::BLANK);
        env.show_2d_on();
        self.screen_alpha = 1.0;
        self.screen_alpha_frame = self.globals.tuning.m006;
        self.heart_init(env);
        self.wepon_init(env);
        self.action_init(env);
        self.xy_init(env);
    }

    pub fn tick(&mut self, env: &mut VitalsEnv) {
        self.fade_bits = 0;
        self.globals.sounds.clear();
        self.status_check(env);
        self.alpha_control(env);
        self.heart_move(env);
        self.action_move(env);
        self.wepon_move(env);
        self.xy_move(env);
        self.pane_scale_transform();
        self.globals.button_action_mode = self.fade_bits;
    }

    pub fn status_check(&mut self, env: &mut VitalsEnv) {
        self.status_flags = 0;
        if !env.show_2d || env.check_camera_flag(cam::BODY_FLAG_02000000) {
            self.status_flags |= ms::UNK4000;
        } else if (self.globals.is_auction_flag() && env.message_status == 0)
            || (env.tmp_bit_0408 && env.message_status == 0)
            || env.event_start_name.as_deref() == Some("PUZZLE_GAME")
            || env.event_start_name.as_deref() == Some("PUZZLE_RUPEE")
        {
            self.status_flags |= ms::UNK80000;
            env.set_a_status(act::BLANK);
            env.set_do_status(act::BLANK);
        } else if env.operate_wind == 2 {
            env.set_do_status_force(act::CHOOSE);
            self.status_flags |= ms::UNK400000;
        } else if env.mini_game_type == 8 && !env.check_player_status0(py::st0::UNK10) {
            env.set_do_status_force(act::BOW);
            env.set_a_status_force(act::HIDDEN);
            self.status_flags |= ms::UNK80000;
        } else if env.check_player_status1(py::st1::WIND_WAKER_CONDUCT)
            && env.a_status == act::RETURN
        {
            self.status_flags |= ms::UNK200000;
        } else if env.event_running && self.globals.menu_status() != 4 {
            if env.demo_mode != 1 && env.message_status != 0 && !env.metronome {
                self.status_flags |= ms::UNK100;
            } else if env.demo_mode != 1 && env.metronome {
                self.status_flags |= ms::UNK200000;
                env.set_a_status_force(act::HIDDEN);
            } else {
                self.status_flags |= ms::UNK40;
            }
        }
        if self.globals.flag() != 0 {
            if env.check_player_status1(py::st1::PICTO_BOX_AIM)
                || env.picture_status == 2
                || env.picture_status == 3
                || env.heap_lock_flag == 6
            {
                self.status_flags |= ms::UNK20000;
                env.set_a_status(act::BLANK);
                env.set_do_status(act::BLANK);
                env.set_r_status(act::BLANK);
            } else if self.globals.menu_status() == 1 {
                self.status_flags |= ms::UNK8;
            } else if self.globals.menu_status() == 2 {
                self.status_flags |= ms::UNK10;
            } else if self.globals.menu_status() == 4 {
                self.status_flags |= ms::UNK800000;
            } else {
                self.status_flags |= ms::UNK20;
            }
        } else {
            if env.c_b1_is_player || env.hyoi_singer_present {
                self.status_flags |= ms::UNK40000;
                if env.a_status == act::SWORD {
                    env.set_a_status(act::BLANK);
                }
            } else if env.check_player_status0(py::st0::SUBJECT)
                || (env.camera_attention & cam::ATTENTION_00000100 != 0
                    && env.check_player_status0(py::st0::CRAWL))
            {
                self.status_flags |= ms::UNK100000;
            }
            if env.check_player_status0(py::st0::UNK10) {
                self.status_flags |= ms::UNK100;
            } else if env.check_player_status0(py::st0::TELESCOPE_LOOK) {
                self.status_flags |= ms::UNK80;
            } else if env.check_player_status1(py::st1::PICTO_BOX_AIM)
                || env.picture_status == 2
                || env.picture_status == 3
                || env.heap_lock_flag == 6
            {
                self.status_flags |= ms::UNK20000;
                env.set_a_status(act::BLANK);
                env.set_r_status(act::BLANK);
                env.set_do_status(act::BLANK);
            } else if env.check_player_status0(py::st0::SHIP_RIDE) {
                self.status_flags |= ms::UNK400;
            } else if env.check_player_status0(py::st0::CRAWL) {
                self.status_flags |= ms::UNK200;
            } else if env.check_player_status0(py::st0::UNK800000) {
                if env.rope_tag {
                    self.status_flags |= ms::UNK800;
                } else {
                    self.status_flags |= ms::UNK1000;
                }
            } else if env.check_player_status0(py::st0::SWIM) {
                self.status_flags |= ms::UNK2000;
            } else if env.check_player_status0(py::st0::HANG | py::st0::UNK2000000) {
                self.status_flags |= ms::UNK8000;
            } else if env.check_player_status0(py::st0::UNK4000000) {
                self.status_flags |= ms::UNK10000;
            }
        }
        if env.stage_type == stage::TYPE_DUNGEON {
            self.status_flags |= ms::UNK4;
        } else if env.stage_type == stage::TYPE_MISC {
            self.status_flags |= ms::UNK2;
        } else {
            self.status_flags |= ms::UNK1;
        }
    }

    pub fn alpha_control(&mut self, env: &VitalsEnv) {
        if !env.show_2d {
            if self.screen_alpha > 0.0 {
                self.screen_alpha = value_increase(
                    self.globals.tuning.m006 as i32,
                    self.screen_alpha_frame as i32,
                    0,
                );
                self.screen_alpha_frame -= 1;
            } else {
                self.screen_alpha = 0.0;
                self.screen_alpha_frame = 0;
            }
        } else if self.globals.flag() != 0 {
            if self.globals.timer() <= 5 {
                let mut dvar = value_increase(5, self.globals.timer(), 0);
                if self.globals.menu_status() != 3 {
                    dvar = (1.0 - dvar) * 0.5 + 0.5;
                }
                if self.screen_alpha > dvar {
                    self.screen_alpha = dvar;
                }
            } else if self.globals.menu_status() == 3 {
                self.screen_alpha = 0.0;
            } else {
                self.screen_alpha = 0.5;
            }
        } else if self.screen_alpha < 1.0 {
            self.screen_alpha = value_increase(
                self.globals.tuning.m006 as i32,
                self.screen_alpha_frame as i32,
                0,
            );
            self.screen_alpha_frame += 1;
        } else {
            self.screen_alpha = 1.0;
            self.screen_alpha_frame = self.globals.tuning.m006;
        }
    }

    pub fn heart_scale_init(&mut self) {
        let containers = self.max_hp as i32 / 4;
        let quarter = (self.curr_hp as i32 - 1) / 4;
        let rate_x = self.globals.tex_rate_x;
        let rate_y = self.globals.tex_rate_y;
        for index in 0..HEART_COUNT {
            let area: i16 = if containers == 0 {
                5
            } else if self.curr_hp == 0 {
                if (index as i32) < containers { 4 } else { 5 }
            } else if index as i32 == quarter {
                (self.curr_hp % 4) as i16
            } else if (index as i32) < quarter {
                0
            } else if (index as i32) < containers {
                4
            } else {
                5
            };
            self.heart[index].user_area = area;
            set_heart_scale_xy(&mut self.heart[index], area, rate_x, rate_y);
            set_heart_scale_xy(&mut self.heart_shadow[index], area, rate_x, rate_y);
            if area != 5 {
                let texture = heart_tex(area);
                heart_change_texture(
                    &mut self.heart[index],
                    &mut self.heart_shadow[index],
                    texture,
                );
            } else {
                pane_hide(&mut self.heart[index]);
                pane_hide(&mut self.heart_shadow[index]);
            }
            pane_trans(&mut self.heart[index], 0.0, 0.0);
            pane_trans(&mut self.heart_shadow[index], 0.0, 0.0);
            let uvar2 = self.curr_hp as i32 - 1;
            if index as i32 == uvar2 / 4 && self.curr_hp != 0 {
                let texture = heart_tex(area);
                self.heart_flash.node.change_texture(texture);
                let factor = self.globals.tuning.m010;
                heart_data_set(
                    &mut self.heart_flash,
                    &mut self.heart[index],
                    &mut self.heart_shadow[index],
                    factor,
                );
            }
        }
    }

    pub fn max_life_change(&mut self) {
        if self.max_hp_adjust > 0 {
            self.max_hp = self.max_hp.wrapping_add(1);
            self.max_hp_adjust -= 1;
            return;
        }
        if self.max_hp_adjust >= 0 {
            return;
        }
        self.max_hp = self.max_hp.wrapping_sub(1);
        self.max_hp_adjust += 1;
    }

    pub fn life_change(&mut self, env: &mut VitalsEnv, sound_on: &mut bool) {
        if self.adjust_hp > 0 {
            self.curr_hp = self.curr_hp.wrapping_add(1);
            self.adjust_hp -= 1;
            if *sound_on != false {
                if (self.curr_hp % 4) == 0 {
                    self.globals.sounds.push(SoundCue::HpGaugeInc);
                }
                if self.adjust_hp == 0 {
                    *sound_on = false;
                }
            }
        } else if self.adjust_hp < 0 {
            self.curr_hp = self.curr_hp.wrapping_sub(1);
            self.adjust_hp += 1;
        }
        self.heart_flash.user_area = 0;
        env.item_now_life = self.curr_hp as u16;
    }

    pub fn heart_light_move(&mut self, env: &VitalsEnv) {
        let hp = env.life as i32 - 1;
        let frames: i16 = if hp / 4 != 0 { 22 } else { 12 };
        self.heart_flash.user_area += 1;
        if frames < self.heart_flash.user_area {
            self.heart_flash.user_area = 0;
        }
        let progress = value_increase(frames as i32, self.heart_flash.user_area as i32, 0);
        let grey = (progress * 255.0f32) as i64 as u8;
        let grow = (self.globals.tuning.m03c * progress) + 1.0;
        set_now_alpha(&mut self.heart_flash, 1.0 - progress);
        self.heart_flash
            .node
            .set_black(TColor::new(grey, grey, grey, 0));
        let width = self.heart_flash.size.x * grow;
        let height = self.heart_flash.size.y * grow;
        self.heart_flash.pos_top_left.x = self.heart_flash.pos_center.x - (width / 2.0);
        self.heart_flash.pos_top_left.y = self.heart_flash.pos_center.y - (height / 2.0);
        self.heart_flash.node.move_to(
            self.heart_flash.pos_top_left.x,
            self.heart_flash.pos_top_left.y,
        );
        self.heart_flash.node.resize(width, height);
    }

    pub fn heart_alpha(&mut self, env: &VitalsEnv) {
        if self.status_flags & ms::UNK4000 != 0
            || (self.status_flags & ms::UNK40 != 0
                && env.check_hind(0x10)
                && !env.check_player_status1(py::st1::UNK2000))
            || self.globals.collect_mode() == 4
            || self.status_flags & ms::UNK200000 != 0
            || self.status_flags & ms::UNK80 != 0
            || self.status_flags & ms::UNK100000 != 0
            || self.status_flags & ms::UNK20000 != 0
            || self.status_flags & ms::UNK20 != 0
            || self.status_flags & ms::UNK80000 != 0
            || self.globals.is_auction_flag()
            || self.status_flags & ms::UNK400000 != 0
            || self.status_flags & ms::UNK800000 != 0
        {
            if self.heart_alpha_frame > 0 {
                self.heart_alpha_frame -= 1;
            } else {
                self.heart_alpha_frame = 0;
            }
        } else if self.status_flags & ms::UNK8 != 0 || self.status_flags & ms::UNK10 != 0 {
            if self.heart_alpha_frame < 3 {
                self.heart_alpha_frame += 1;
            } else if self.heart_alpha_frame > 3 {
                self.heart_alpha_frame -= 1;
            } else {
                self.heart_alpha_frame = 3;
            }
        } else {
            if self.heart_alpha_frame < 5 {
                self.heart_alpha_frame += 1;
            } else {
                self.heart_alpha_frame = 5;
            }
            if env.mini_game_type != 6 {
                self.globals.sounds.push(SoundCue::HeartGauge);
            }
        }
        self.heart_alpha = value_increase(5, self.heart_alpha_frame as i32, 0);
    }

    pub fn heart_init(&mut self, env: &mut VitalsEnv) {
        self.globals.tex_scale_x = self.heart[0].size_orig.x / 40.0;
        self.globals.tex_scale_y = self.heart[0].size_orig.y / 32.0;
        self.globals.tex_rate_x = 0.4;
        self.globals.tex_rate_y = 0.5;
        self.heart_scale = 1.0;
        self.heart_alpha = 0.0;
        self.max_hp = env.max_life;
        self.curr_hp = env.life;
        self.heart_alpha_frame = 5;
        self.heart_flash.user_area = 0;
        self.heart_flash.user_area = 0;
        set_now_alpha_zero(&mut self.heart_flash);
        self.heart_scale_init();
        env.item_now_life = self.curr_hp as u16;
    }

    pub fn life_move(&mut self, env: &mut VitalsEnv, force: bool) {
        let mut changed = false;
        if env.item_max_life_count != 0 || (env.item_life_count as i16) != 0 || force {
            let mut max_hp = env.max_life as i16 + env.item_max_life_count;
            let mut new_hp: i16 = if env.item_max_life_count > 0 {
                (((max_hp as i32) / 4) * 4) as i16
            } else {
                (env.life as i32 + env.item_life_count as i32) as i16
            };
            if env.item_max_life_count != 0 {
                if (max_hp as i32) > 0x50 {
                    max_hp = 0x50;
                } else if max_hp < 0 {
                    max_hp = 0;
                }
                self.max_hp_adjust = max_hp - self.max_hp as i16;
                env.max_life = max_hp as u8;
                env.item_max_life_count = 0;
                self.magic_bar[5].user_area = 1;
            }
            if new_hp as i32 > ((max_hp as i32) / 4) * 4 {
                new_hp = (((max_hp as i32) / 4) * 4) as i16;
            } else if new_hp < 0 {
                new_hp = 0;
            }
            self.adjust_hp = new_hp - self.curr_hp as i16;
            env.life = new_hp as u8;
            env.item_life_count = 0.0;
            if self.adjust_hp >= 4 {
                self.life_sound = true;
            }
        }
        if self.max_hp_adjust != 0 || force {
            self.max_life_change();
            changed = true;
        }
        if self.adjust_hp != 0 || force {
            let mut sound = self.life_sound;
            self.life_change(env, &mut sound);
            self.life_sound = sound;
            changed = true;
        }
        if changed {
            self.heart_scale_init();
        }
        if env.life != 0 {
            self.heart_light_move(env);
        } else {
            set_now_alpha_zero(&mut self.heart_flash);
        }
    }

    pub fn battle_life_move(&mut self, env: &VitalsEnv, force: bool) {
        let point = env.battle_life_point;
        let gap = self.heart[1].pos_center_orig.x - self.heart[0].pos_center_orig.x;
        let rate_x = self.globals.tex_rate_x;
        let rate_y = self.globals.tex_rate_y;
        let factor = self.globals.tuning.m048;
        let now_factor = self.globals.tuning.m010;
        let spread = self.globals.tuning.m058;
        let nudge_x = self.globals.tuning.m04c;
        let nudge_y = self.globals.tuning.m05c;
        let base_x = self.heart[0].pos_center_orig.x;
        let base_y = self.heart[0].pos_center_orig.y;
        let shadow_base_x = self.heart_shadow[0].pos_center_orig.x;
        let shadow_base_y = self.heart_shadow[0].pos_center_orig.y;
        if self.battle_point_cached != point || !force {
            for index in 0..HEART_COUNT {
                self.battle_point_cached = point;
                if index == 1 || index == 2 {
                    self.heart[index].pos_center.x =
                        (spread * gap * index as f32) + base_x + nudge_x;
                    self.heart_shadow[index].pos_center.x =
                        (spread * gap * index as f32) + shadow_base_x + nudge_x;
                    self.heart[index].pos_center.y = base_y + nudge_y;
                    self.heart_shadow[index].pos_center.y = shadow_base_y + nudge_y;
                } else if index == 0 {
                    self.heart[index].pos_center.x = base_x + nudge_x;
                    self.heart_shadow[index].pos_center.x = shadow_base_x + nudge_x;
                    self.heart[index].pos_center.y = base_y + nudge_y;
                    self.heart_shadow[index].pos_center.y = shadow_base_y + nudge_y;
                }
                if index < point as usize {
                    heart_change_texture(
                        &mut self.heart[index],
                        &mut self.heart_shadow[index],
                        heart_tex(0),
                    );
                    if index as u8 == point - 1 {
                        set_now_heart_scale_xy2(&mut self.heart[index], now_factor, factor);
                        set_now_heart_scale_xy2(&mut self.heart_shadow[index], now_factor, factor);
                    } else {
                        set_heart_scale_xy2(&mut self.heart[index], 0, rate_x, rate_y, factor);
                        set_heart_scale_xy2(
                            &mut self.heart_shadow[index],
                            0,
                            rate_x,
                            rate_y,
                            factor,
                        );
                    }
                } else if index < 3 {
                    heart_change_texture(
                        &mut self.heart[index],
                        &mut self.heart_shadow[index],
                        heart_tex(4),
                    );
                    set_heart_scale_xy2(&mut self.heart[index], 4, rate_x, rate_y, factor);
                    set_heart_scale_xy2(&mut self.heart_shadow[index], 4, rate_x, rate_y, factor);
                } else {
                    pane_hide(&mut self.heart[index]);
                    pane_hide(&mut self.heart_shadow[index]);
                }
            }
        }
    }

    pub fn heart_color(&mut self, env: &VitalsEnv) {
        let white = TColor::new(0xFF, 0xFF, 0xFF, 0xFF);
        let black = TColor::new(0x00, 0x00, 0x00, 0x00);
        if env.soup_power_up && env.mini_game_type != 6 {
            if self.heart_shadow[0].user_area == 0 {
                self.heart_shadow[0].user_area = 1;
            }
            if self.heart_shadow[1].user_area < 0x1d {
                self.heart_shadow[1].user_area += 1;
            } else {
                self.heart_shadow[1].user_area = 0;
            }
            let progress = if self.heart_shadow[1].user_area < 0xf {
                value_increase(0xf, self.heart_shadow[1].user_area as i32, 0)
            } else {
                value_increase(0xf, 0x1e - self.heart_shadow[1].user_area as i32, 0)
            };
            let white2 = TColor::new(
                blend(white.r as f32, 255.0, progress),
                blend(white.g as f32, 255.0, progress),
                blend(white.b as f32, 255.0, progress),
                0xFF,
            );
            let black2 = TColor::new(
                blend(black.r as f32, 255.0, progress),
                blend(black.g as f32, 255.0, progress),
                blend(black.b as f32, 255.0, progress),
                0x00,
            );
            for index in 0..HEART_COUNT {
                self.heart[index].node.set_white(white2);
                self.heart[index].node.set_black(black2);
            }
        } else if self.heart_shadow[0].user_area == 1 {
            for index in 0..HEART_COUNT {
                self.heart[index].node.set_white(white);
                self.heart[index].node.set_black(black);
            }
            self.heart_shadow[0].user_area = 0;
        }
    }

    pub fn heart_move(&mut self, env: &mut VitalsEnv) {
        if env.mini_game_type == 6 {
            let flag = self.battle_mode;
            self.battle_life_move(env, flag);
            self.battle_mode = true;
        } else if self.battle_mode {
            self.heart_init(env);
            self.battle_mode = false;
        } else {
            let flag = self.battle_mode;
            self.life_move(env, flag);
        }
        self.heart_color(env);
        self.heart_alpha(env);
    }

    pub fn heart_draw(&mut self) {
        for index in 0..HEART_COUNT {
            set_alpha(&mut self.heart[index]);
            set_alpha(&mut self.heart_shadow[index]);
        }
    }

    pub fn pane_scale_transform(&mut self) {
        let f31 = self.globals.tuning.m040 as f32 / 100.0;
        let f30 = self.globals.tuning.m044 as f32 / 100.0;
        let f29 = self.globals.tuning.m046 as f32 / 100.0;
        let offset_x = self.globals.tuning.m050 as f32;
        let offset_y = self.globals.tuning.m052 as f32;
        let center = self.center.clone();
        parent_pane_trans(&mut self.center, f31, offset_x, offset_y);
        child_pane_trans(&mut self.wepon_back, &center, f31, offset_x, offset_y);
        child_pane_trans(&mut self.wepon_add, &center, f31, offset_x, offset_y);
        child_pane_trans_only(&mut self.wepon_blink, &center, f31, f30, offset_x, offset_y);
        child_pane_trans_only(&mut self.wepon_text, &center, f31, f29, offset_x, offset_y);
        let wepon_text = self.wepon_text.clone();
        child_pane_trans_child_trans(
            &mut self.wepon_text_key,
            &center,
            &wepon_text,
            f31,
            f29,
            offset_x,
            offset_y,
        );
        child_pane_trans_only(
            &mut self.action_blink,
            &center,
            f31,
            f30,
            offset_x,
            offset_y,
        );
        for index in 0..2 {
            child_pane_trans(
                &mut self.action_text[index],
                &center,
                f31,
                offset_x,
                offset_y,
            );
            child_pane_trans(
                &mut self.action_text_shadow[index],
                &center,
                f31,
                offset_x,
                offset_y,
            );
        }
        child_pane_trans(&mut self.count_tens, &center, f31, offset_x, offset_y);
        child_pane_trans(&mut self.count_ones, &center, f31, offset_x, offset_y);
        child_pane_trans(&mut self.action_text_main, &center, f31, offset_x, offset_y);
        child_pane_trans(
            &mut self.action_text_main_key,
            &center,
            f31,
            offset_x,
            offset_y,
        );
        for index in 0..ITEM_BUTTON_COUNT {
            child_pane_trans_only(
                &mut self.xy_item[index],
                &center,
                f31,
                f29,
                offset_x,
                offset_y,
            );
            let item = self.xy_item[index].clone();
            child_pane_trans_child_trans_only(
                &mut self.xy_count_ones[index],
                &center,
                &item,
                f31,
                f29,
                offset_x,
                offset_y,
            );
            child_pane_trans_child_trans_only(
                &mut self.xy_count_tens[index],
                &center,
                &item,
                f31,
                f29,
                offset_x,
                offset_y,
            );
            let item = self.xy_item[index].clone();
            child_pane_trans_child_trans(
                &mut self.xy_item_key[index],
                &center,
                &item,
                f31,
                f29,
                offset_x,
                offset_y,
            );
            let item = self.xy_item[index].clone();
            child_pane_trans_child_trans(
                &mut self.xy_bow_light[index],
                &center,
                &item,
                f31,
                f29,
                offset_x,
                offset_y,
            );
            let item = self.xy_item[index].clone();
            child_pane_trans_child_trans(
                &mut self.xy_bow_light_2[index],
                &center,
                &item,
                f31,
                f29,
                offset_x,
                offset_y,
            );
            let item = self.xy_item[index].clone();
            child_pane_trans_child_trans(
                &mut self.xy_bow_light_3[index],
                &center,
                &item,
                f31,
                f29,
                offset_x,
                offset_y,
            );
        }
        child_pane_trans(&mut self.xy_arrow, &center, f31, offset_x, offset_y);
        for index in 0..2 {
            child_pane_trans(
                &mut self.xy_arrow_back[index],
                &center,
                f31,
                offset_x,
                offset_y,
            );
            child_pane_trans(
                &mut self.xy_arrow_00[index],
                &center,
                f31,
                offset_x,
                offset_y,
            );
            child_pane_trans(
                &mut self.xy_arrow_02[index],
                &center,
                f31,
                offset_x,
                offset_y,
            );
        }
    }

    pub fn wepon_init(&mut self, env: &mut VitalsEnv) {
        self.wepon_add.user_area = 0;
        self.last_a_status = 0;
        env.set_a_status(act::BLANK);
        let archive = if env.a_status == act::RETURN || env.a_status == act::RETURN {
            Archive::Menu
        } else {
            Archive::ActionIcon
        };
        let filename = action_tex(act::BLANK);
        self.action_tex[0].read(filename, archive);
        let name = self.action_tex[0].name.clone();
        self.wepon_text.node.change_texture(&name);
        self.wepon_text_key.node.change_texture(&name);
        self.wepon_blink.node.change_texture(&name);
        self.wepon_text.user_area = 0;
    }

    pub fn wepon_move(&mut self, env: &mut VitalsEnv) {
        if env.a_status_force != act::BLANK {
            env.set_a_status(env.a_status_force);
        }
        self.wepon_change(env);
        if env.do_status == act::BID {
            let mut white = TColor::all(0xFF);
            let black = TColor::new(0xFF, 0xFF, 0xFF, 0xFF);
            white.a = 0x00;
            self.action_blink.user_area += 1;
            if self.action_blink.user_area >= 0x14 {
                self.action_blink.user_area = 0;
            }
            let frame = self.action_blink.user_area as i32;
            let progress = if frame < 10 {
                value_increase(10, frame, 0)
            } else {
                value_increase(10, 0x14 - frame, 0)
            };
            let grey = (-(progress * -255.0f32)) as i64 as u8;
            white.g = grey;
            white.r = grey;
            self.action_blink.node.set_white(black);
            self.action_blink.node.set_black(white);
            let init = self.wepon_blink.init_alpha as f32;
            self.action_blink.now_alpha = (init - (init - (0x78 as f32)) * progress) as i64 as u8;
        } else if self.action_blink.user_area != 0 {
            self.action_blink
                .node
                .set_white(TColor::new(0xFF, 0xFF, 0xFF, 0xFF));
            self.action_blink
                .node
                .set_black(TColor::new(0x00, 0x00, 0xFF, 0x00));
            self.action_blink.now_alpha = self.wepon_blink.init_alpha;
            self.action_blink.user_area = 0;
        }
        if env.do_status == act::PARRY_31
            || env.do_status == act::THROW_2D
            || env.do_status == act::CARRY_2E
        {
            let mut white = TColor::all(0xFF);
            white.a = 0x00;
            self.action_text[1].user_area += 1;
            if self.action_text[1].user_area >= 0x14 {
                self.action_text[1].user_area = 0;
                if env.do_status == act::THROW_2D {
                    self.globals.sounds.push(SoundCue::SeedOkBlink);
                } else if env.do_status == act::CARRY_2E {
                    self.globals.sounds.push(SoundCue::HarpIndicator);
                } else {
                    self.globals.sounds.push(SoundCue::OmamoriBlink);
                }
            }
            let frame = self.action_text[1].user_area as i32;
            let progress = if frame < 10 {
                value_increase(10, frame, 0)
            } else {
                value_increase(10, 0x14 - frame, 0)
            };
            let grey = (progress * 255.0f32) as i64 as u8;
            white.b = grey;
            white.g = grey;
            white.r = grey;
            self.action_text[1].node.set_black(white);
        } else if env.do_status == act::PARRY {
            self.action_text[1].node.set_black(TColor::all(0x00));
        }
        let frame = self.wepon_add.user_area;
        if frame != 0 && frame <= 0xc {
            self.wepon_anime();
            self.wepon_add.user_area += 1;
            if self.wepon_add.user_area > 0xc {
                self.wepon_add.user_area = 0;
            }
        }
        self.wepon_trans(env);
        self.wepon_alpha(env);
        env.set_a_status_force(act::BLANK);
    }

    pub fn wepon_draw(&mut self) {
        set_alpha(&mut self.wepon_blink);
        set_alpha(&mut self.wepon_text);
        set_alpha(&mut self.wepon_text_key);
        set_alpha(&mut self.wepon_back);
        set_alpha(&mut self.wepon_add);
    }

    pub fn wepon_change(&mut self, env: &mut VitalsEnv) {
        if self.wepon_text.user_area != 0 {
            if self.last_a_status == act::BID {
                self.wepon_blink.user_area = 0;
                self.wepon_blink
                    .node
                    .set_white(TColor::new(0xFF, 0xFF, 0xFF, 0xFF));
                self.wepon_blink
                    .node
                    .set_black(TColor::new(0x00, 0x00, 0xFF, 0x00));
            }
            if env.a_status != act::BLANK {
                if env.a_status == act::SWORD {
                    let filename = wepon_tex(
                        env.is_collect(0, 1),
                        env.is_collect(0, 2),
                        env.is_collect(0, 3),
                    );
                    self.action_tex[0].read(filename, Archive::ItemIcon);
                    let name = self.action_tex[0].name.clone();
                    self.wepon_text.node.change_texture(&name);
                    self.wepon_text_key.node.change_texture(&name);
                    self.wepon_blink.node.change_texture(&name);
                } else if env.a_status == act::STICK
                    || env.a_status == act::HATCHET
                    || env.a_status == act::IRON_CLUB
                    || env.a_status == act::LONGSWORD
                    || env.a_status == act::SPEAR
                    || env.a_status == act::SPEAR_DUPE
                {
                    let filename = action_tex(env.a_status);
                    self.action_tex[0].read(filename, Archive::ItemIcon);
                    let name = self.action_tex[0].name.clone();
                    self.wepon_text.node.change_texture(&name);
                    self.wepon_text_key.node.change_texture(&name);
                    self.wepon_blink.node.change_texture(&name);
                } else {
                    let archive = if env.a_status == act::RETURN || env.a_status == act::RETURN {
                        Archive::Menu
                    } else {
                        Archive::ActionIcon
                    };
                    let filename = action_tex(env.a_status);
                    self.action_tex[0].read(filename, archive);
                    let name = self.action_tex[0].name.clone();
                    self.wepon_text.node.change_texture(&name);
                    self.wepon_text_key.node.change_texture(&name);
                    self.wepon_blink.node.change_texture(&name);
                }
            }
            self.wepon_text.user_area = 0;
        }
        if self.last_a_status != env.a_status {
            self.wepon_text.user_area = 1;
            self.wepon_add.user_area = 1;
            self.last_a_status = env.a_status;
        }
    }

    pub fn wepon_anime(&mut self) {
        let frame = self.wepon_add.user_area as i32;
        let offset = if frame <= 7 {
            (frame as f32 * 5.0) / 7.0
        } else if frame <= 10 {
            5.0 - ((frame - 7) as f32 * 13.0) / 3.0
        } else {
            -8.0 + (8.0 * (frame - 10) as f32 / 2.0)
        };
        let tops: [f32; 5] = [
            self.wepon_blink.pos_top_left.y,
            self.wepon_text.pos_top_left.y,
            self.wepon_text_key.pos_top_left.y,
            self.wepon_back.pos_top_left.y,
            self.wepon_add.pos_top_left.y,
        ];
        let xs: [f32; 5] = [
            self.wepon_blink.pos_top_left.x,
            self.wepon_text.pos_top_left.x,
            self.wepon_text_key.pos_top_left.x,
            self.wepon_back.pos_top_left.x,
            self.wepon_add.pos_top_left.x,
        ];
        let panes: [&mut MsgPane; 5] = [
            &mut self.wepon_blink,
            &mut self.wepon_text,
            &mut self.wepon_text_key,
            &mut self.wepon_back,
            &mut self.wepon_add,
        ];
        for index in 0..5 {
            panes[index].node.move_to(xs[index], tops[index] + offset);
        }
    }

    pub fn wepon_trans(&mut self, env: &VitalsEnv) {
        let tuning = self.globals.tuning.clone();
        let icon = self.globals.info.button_icon_mode;
        if self.status_flags & ms::UNK4000 != 0
            || (self.status_flags & ms::UNK40 != 0
                && env.check_hind(4)
                && self.status_flags & ms::UNK20 == 0)
            || (self.status_flags & ms::UNK80 != 0 && env.scope_message_status != 0x0B)
            || ((self.status_flags & ms::UNK100 != 0 && env.a_status != act::CANCEL)
                || env.a_status == act::HIDDEN)
        {
            if self.wepon_move_status != 1 {
                self.wepon_move_flag = 1;
                off_bit8(&mut self.hide_bits, 2);
            }
            self.wepon_move_status = 1;
        } else if self.status_flags & ms::UNK80000 != 0 {
            if self.wepon_move_status != 0xb {
                self.wepon_move_flag = 1;
                off_bit8(&mut self.hide_bits, 2);
            }
            self.wepon_move_status = 0xb;
        } else if self.status_flags & ms::UNK20 != 0 {
            if env.up_button == 1 {
                if self.wepon_move_status != 9 {
                    self.wepon_move_flag = 1;
                    off_bit8(&mut self.hide_bits, 2);
                }
                self.wepon_move_status = 9;
            } else if env.up_button == 0 {
                if icon == 0 || icon == 1 || icon == 2 || icon == 3 {
                    if self.wepon_move_status != 8 {
                        self.wepon_move_flag = 1;
                        off_bit8(&mut self.hide_bits, 2);
                    }
                    self.wepon_move_status = 8;
                } else if icon == 7 || icon == 8 || icon == 9 || icon == 10 {
                    if self.wepon_move_status != 10 {
                        self.wepon_move_flag = 1;
                        off_bit8(&mut self.hide_bits, 2);
                    }
                    self.wepon_move_status = 10;
                } else if icon == 6 {
                    if self.wepon_move_status != 1 {
                        self.wepon_move_flag = 1;
                        off_bit8(&mut self.hide_bits, 2);
                    }
                    self.wepon_move_status = 1;
                } else if self.wepon_move_status != 9 {
                    self.wepon_move_flag = 1;
                    off_bit8(&mut self.hide_bits, 2);
                    self.wepon_move_status = 9;
                }
            }
        } else if self.status_flags & (ms::UNK80 | ms::UNK100000) != 0
            || (self.status_flags & ms::UNK200000 != 0 && self.status_flags & ms::UNK10 == 0)
        {
            if self.wepon_move_status != 2 {
                self.wepon_move_flag = 1;
                off_bit8(&mut self.hide_bits, 2);
            }
            self.wepon_move_status = 2;
        } else if self.status_flags & ms::UNK20000 != 0 {
            if self.wepon_move_status != 3 {
                self.wepon_move_flag = 1;
                off_bit8(&mut self.hide_bits, 2);
            }
            self.wepon_move_status = 3;
        } else if self.globals.flag() != 0
            && (self.globals.push_menu_button() == 1
                || (self.globals.menu_status() == 1 && self.globals.push_menu_button() == 0))
        {
            if self.globals.item_mode() == 0 {
                if self.wepon_move_status != 4 {
                    self.wepon_move_flag = 1;
                    off_bit8(&mut self.hide_bits, 2);
                }
                self.wepon_move_status = 4;
            } else if self.wepon_move_status != 5 {
                self.wepon_move_flag = 1;
                off_bit8(&mut self.hide_bits, 2);
                self.wepon_move_status = 5;
            }
        } else if self.globals.flag() != 0
            && (self.globals.push_menu_button() == 2
                || (self.globals.menu_status() == 2 && self.globals.push_menu_button() == 0))
        {
            if self.wepon_move_status != 5 {
                self.wepon_move_flag = 1;
                off_bit8(&mut self.hide_bits, 2);
            }
            self.wepon_move_status = 5;
        } else if self.status_flags & ms::UNK800000 != 0 {
            if self.wepon_move_status != 5 {
                self.wepon_move_flag = 1;
                off_bit8(&mut self.hide_bits, 2);
            }
            self.wepon_move_status = 5;
        } else if self.status_flags & ms::UNK40000 != 0 {
            if self.wepon_move_status != 6 {
                self.wepon_move_flag = 1;
                off_bit8(&mut self.hide_bits, 2);
            }
            self.wepon_move_status = 6;
        } else if self.status_flags & ms::UNK400000 != 0 {
            if self.wepon_move_status != 5 {
                self.wepon_move_flag = 1;
                off_bit8(&mut self.hide_bits, 2);
            }
            self.wepon_move_status = 5;
        } else if self.status_flags & ms::UNK100 != 0 && env.a_status == act::CANCEL {
            if self.wepon_move_status != 7 {
                self.wepon_move_flag = 1;
                off_bit8(&mut self.hide_bits, 2);
            }
            self.wepon_move_status = 7;
        } else if self.wepon_move_status != 0 {
            self.wepon_move_flag = 1;
            off_bit8(&mut self.hide_bits, 2);
            self.wepon_move_status = 0;
        }
        if self.hide_bits != 0xf {
            if !is_bit8(self.hide_bits, 2) {
                if self.wepon_move_frame >= 5 {
                    self.wepon_move_frame = 5;
                    self.wepon_move_flag = 0;
                    on_bit8(&mut self.hide_bits, 2);
                    match self.wepon_move_status {
                        1 => {
                            self.wepon_now_x = 0.0;
                            self.wepon_now_y = -200.0;
                        }
                        2 => {
                            self.wepon_now_x = tuning.m0a2 as f32;
                            self.wepon_now_y = tuning.m0aa as f32;
                        }
                        3 => {
                            self.wepon_now_x = tuning.m0b2 as f32;
                            self.wepon_now_y = tuning.m0ba as f32;
                        }
                        4 => {
                            self.wepon_now_x = tuning.m0c2 as f32;
                            self.wepon_now_y = tuning.m0ca as f32;
                        }
                        5 => {
                            self.wepon_now_x = tuning.m0d2 as f32;
                            self.wepon_now_y = tuning.m0da as f32;
                        }
                        6 => {
                            self.wepon_now_x = tuning.m0e2 as f32;
                            self.wepon_now_y = tuning.m0ea as f32;
                        }
                        7 => {
                            self.wepon_now_x = tuning.m102 as f32;
                            self.wepon_now_y = tuning.m10a as f32;
                        }
                        8 => {
                            self.wepon_now_x = tuning.m112 as f32;
                            self.wepon_now_y = tuning.m11a as f32;
                        }
                        9 => {
                            if env.up_button == 1 {
                                self.wepon_now_x = tuning.m106 as f32;
                                self.wepon_now_y = tuning.m10e as f32;
                            } else if env.up_button == 0 {
                                self.wepon_now_x = tuning.m106 as f32 - (0x21 as f32);
                                self.wepon_now_y = tuning.m10e as f32 - (4 as f32);
                            }
                        }
                        10 => {
                            self.wepon_now_x = tuning.m112 as f32 + (0x12 as f32);
                            self.wepon_now_y = tuning.m11a as f32 + (3 as f32);
                        }
                        0xb => {
                            self.wepon_now_x = tuning.m0f2 as f32;
                            self.wepon_now_y = tuning.m0fa as f32 - (0x3c as f32);
                        }
                        _ => {
                            self.wepon_now_x = tuning.m0f2 as f32;
                            self.wepon_now_y = tuning.m0fa as f32;
                        }
                    }
                } else {
                    self.wepon_move_frame += 1;
                }
            }
        } else if (self.wepon_move_frame as i32) <= 0 {
            self.wepon_move_frame = 0;
        } else {
            self.wepon_move_frame -= 1;
        }
        let progress = value_increase(5, self.wepon_move_frame as i32, 0);
        let x = self.wepon_now_x;
        let y = self.wepon_now_y + (progress * (-200.0 - self.wepon_now_y));
        if self.wepon_move_status == 10 {
            pane_trans(
                &mut self.wepon_blink,
                (x + self.wepon_exp_x) + 57.0,
                (y + self.wepon_exp_y) + 9.0,
            );
        } else {
            pane_trans(
                &mut self.wepon_blink,
                x + self.wepon_exp_x,
                y + self.wepon_exp_y,
            );
        }
        pane_trans(&mut self.wepon_text, x, y);
        pane_trans(&mut self.wepon_text_key, x, y);
        pane_trans(&mut self.wepon_back, x, y);
        pane_trans(&mut self.wepon_add, x, y);
    }

    pub fn wepon_alpha(&mut self, env: &VitalsEnv) {
        let weapon_status = [
            act::SWORD,
            act::STICK,
            act::HATCHET,
            act::IRON_CLUB,
            act::LONGSWORD,
            act::SPEAR,
            act::SPEAR_DUPE,
        ];
        if (self.status_flags & ms::UNK400 != 0 && env.a_status == act::SWORD)
            || (self.status_flags & ms::UNK2000 != 0 && env.a_status != act::SPEAK)
            || (self.status_flags & ms::UNK2 != 0
                && env.a_status == act::SWORD
                && env.start_stage_name != "Ojhous"
                && env.start_stage_name != "Orichh")
            || self.status_flags & (ms::UNK8000 | ms::UNK10000) != 0
        {
            if self.wepon_alpha_frame > 3 {
                self.wepon_alpha_frame -= 1;
            } else if self.wepon_alpha_frame < 3 {
                self.wepon_alpha_frame += 1;
            } else {
                self.wepon_alpha_frame = 3;
            }
        } else {
            if self.status_flags & ms::UNK2 == 0
                || (env.a_status != act::SWORD
                    && env.a_status != act::BLANK
                    && self.globals.flag() == 0)
                || env.start_stage_name == "Ojhous"
                || env.start_stage_name == "Orichh"
            {
                on_bit8(&mut self.fade_bits, 2);
            }
            if self.wepon_alpha_frame < 5 {
                self.wepon_alpha_frame += 1;
            } else {
                self.wepon_alpha_frame = 5;
            }
        }
        let progress = value_increase(5, self.wepon_alpha_frame as i32, 0);
        if env.a_status == act::BLANK {
            set_now_alpha_zero(&mut self.wepon_blink);
            set_now_alpha_zero(&mut self.wepon_text);
            set_now_alpha_zero(&mut self.wepon_text_key);
            set_now_alpha(&mut self.wepon_back, progress);
            set_now_alpha(&mut self.wepon_add, progress);
        } else if weapon_status.contains(&env.a_status) {
            set_now_alpha_zero(&mut self.wepon_blink);
            set_now_alpha(&mut self.wepon_text, progress);
            set_now_alpha(&mut self.wepon_text_key, progress);
            set_now_alpha(&mut self.wepon_back, progress);
            set_now_alpha(&mut self.wepon_add, progress);
            set_now_alpha(&mut self.center, progress);
        } else {
            set_now_alpha(&mut self.wepon_blink, progress);
            set_now_alpha_zero(&mut self.wepon_text);
            set_now_alpha_zero(&mut self.wepon_text_key);
            set_now_alpha(&mut self.wepon_back, progress);
            set_now_alpha(&mut self.wepon_add, progress);
            set_now_alpha(&mut self.center, progress);
        }
        if is_bit8(self.hide_bits, 2) && self.wepon_text.user_area == 0 {
            self.wepon_blink.node.show();
            self.wepon_text.node.show();
            self.wepon_text_key.node.show();
        } else {
            self.wepon_blink.node.hide();
            self.wepon_text.node.hide();
            self.wepon_text_key.node.hide();
        }
    }

    pub fn action_init(&mut self, env: &mut VitalsEnv) {
        self.action_text[0].user_area = 0;
        self.action_text_main.user_area = 0;
        self.last_do_status = 0;
        env.set_do_status(act::BLANK);
        let filename = action_tex(act::BLANK);
        self.action_tex[1].read(filename, Archive::ActionIcon);
        let name = self.action_tex[1].name.clone();
        self.action_text_main.node.change_texture(&name);
        self.action_text_main_key.node.change_texture(&name);
        self.action_blink.node.change_texture(&name);
    }

    pub fn action_move(&mut self, env: &mut VitalsEnv) {
        if env.do_status_force != act::BLANK {
            env.set_do_status(env.do_status_force);
        } else {
            self.action_force(env);
        }
        self.action_change(env);
        self.action_trans(env);
        self.action_alpha(env);
        env.set_do_status_force(act::BLANK);
    }

    pub fn action_draw(&mut self) {
        set_alpha(&mut self.action_blink);
        for index in 0..2 {
            set_alpha(&mut self.action_text[index]);
            set_alpha(&mut self.action_text_shadow[index]);
        }
        set_alpha(&mut self.count_tens);
        set_alpha(&mut self.count_ones);
        set_alpha(&mut self.action_text_main);
        set_alpha(&mut self.action_text_main_key);
        set_alpha(&mut self.center);
    }

    pub fn action_force(&mut self, env: &mut VitalsEnv) {
        if self.status_flags & ms::UNK4000 != 0
            || (self.status_flags & ms::UNK40 != 0
                && env.check_hind(4)
                && self.status_flags & ms::UNK20 == 0)
        {
            env.set_do_status(act::BLANK);
            return;
        }
        if (self.status_flags & (ms::UNK8 | ms::UNK10 | ms::UNK800000) != 0)
            || self.status_flags & ms::UNK20 != 0
        {
            if self.globals.collect_mode() != 5 {
                env.set_a_status(act::RETURN);
            }
            if self.status_flags & ms::UNK20 != 0 && env.up_button == 0 {
                let icon = self.globals.info.button_icon_mode;
                if icon == 0 || icon == 1 || icon == 2 || icon == 3 {
                    if icon == 0 {
                        env.set_do_status(act::ZOOM01);
                    } else if icon == 1 {
                        env.set_do_status(act::ZOOM02);
                    } else if icon == 2 || icon == 3 {
                        env.set_do_status(act::BLANK);
                    }
                    env.set_r_status(act::CHARTS);
                    return;
                }
                if icon == 7 || icon == 8 || icon == 9 || icon == 10 {
                    if icon == 10 {
                        env.set_do_status(act::OPEN);
                    } else {
                        env.set_do_status(act::BLANK);
                    }
                    env.set_r_status(act::SEA_CHART);
                    return;
                }
                if icon == 4 {
                    env.set_do_status(act::CHOOSE);
                    env.set_r_status(act::BLANK);
                    return;
                }
                env.set_do_status(act::BLANK);
                env.set_r_status(act::BLANK);
                return;
            }
            if self.status_flags & ms::UNK800000 != 0 {
                env.set_do_status(act::CHOOSE);
            }
            return;
        }
        if self.status_flags & ms::UNK100 == 0 {
            return;
        }
        match self.globals.info.mode {
            1 => env.set_do_status(act::NEXT),
            3 => env.set_do_status(act::CHOOSE),
            4 => env.set_do_status(act::RETURN),
            _ => env.set_do_status(act::BLANK),
        }
        if env.do_status_force != act::BLANK {
            env.set_do_status_force(act::BLANK);
        }
        if env.a_status_force != act::BLANK {
            let forced = env.do_status_force;
            env.set_a_status(forced);
            env.set_a_status_force(act::BLANK);
        }
    }

    pub fn action_change(&mut self, env: &mut VitalsEnv) {
        if self.action_text_main.user_area != 0 {
            if env.do_status == act::BLANK {
                self.action_text[0].user_area = 1;
            } else if env.do_status == act::PARRY {
                self.globals.sounds.push(SoundCue::SpecialAttackOn);
            } else if env.do_status != act::PARRY_31
                && env.do_status != act::THROW_2D
                && env.do_status != act::CARRY_2E
            {
                if env.do_status == act::BOW {
                    let filename = action_tex(env.do_status);
                    self.action_tex[1].read(filename, Archive::ItemIcon);
                    let name = self.action_tex[1].name.clone();
                    self.action_text_main.node.change_texture(&name);
                    self.action_text_main_key.node.change_texture(&name);
                } else {
                    let archive = if env.do_status == act::RETURN || env.do_status == act::RETURN {
                        Archive::Menu
                    } else {
                        Archive::ActionIcon
                    };
                    let filename = action_tex(env.do_status);
                    self.action_tex[1].read(filename, archive);
                    let name = self.action_tex[1].name.clone();
                    self.action_blink.node.change_texture(&name);
                    self.action_text[0].user_area = 1;
                }
            }
            self.last_do_status = env.do_status;
            self.action_text_main.user_area = 0;
        }
        if self.last_do_status != env.do_status {
            self.action_text_main.user_area = 1;
        }
    }

    pub fn action_trans(&mut self, env: &VitalsEnv) {
        let tuning = self.globals.tuning.clone();
        let icon = self.globals.info.button_icon_mode;
        if self.status_flags & ms::UNK4000 != 0
            || (self.status_flags & ms::UNK40 != 0
                && env.check_hind(2)
                && self.status_flags & ms::UNK20 == 0)
            || self.status_flags & (ms::UNK80 | ms::UNK100000) != 0
            || (self.status_flags & ms::UNK200000 != 0
                && self.status_flags & ms::UNK20 == 0
                && self.status_flags & ms::UNK10 == 0)
            || env.do_status == act::HIDDEN
        {
            if self.action_move_status != 1 {
                self.action_move_flag = 1;
                off_bit8(&mut self.hide_bits, 1);
            }
            self.action_move_status = 1;
        } else if self.status_flags & ms::UNK20 != 0 {
            if env.up_button == 1 {
                if self.action_move_status != 9 {
                    self.action_move_flag = 1;
                    off_bit8(&mut self.hide_bits, 1);
                }
                self.action_move_status = 9;
            } else if env.up_button == 0 {
                if icon == 0 || icon == 1 || icon == 2 || icon == 3 {
                    if self.action_move_status != 8 {
                        self.action_move_flag = 1;
                        off_bit8(&mut self.hide_bits, 1);
                    }
                    self.action_move_status = 8;
                } else if icon == 7 || icon == 8 || icon == 9 || icon == 10 {
                    if self.action_move_status != 0xb {
                        self.action_move_flag = 1;
                        off_bit8(&mut self.hide_bits, 1);
                    }
                    self.action_move_status = 0xb;
                } else if icon == 0xb || icon == 6 {
                    if self.action_move_status != 1 {
                        self.action_move_flag = 1;
                        off_bit8(&mut self.hide_bits, 1);
                    }
                    self.action_move_status = 1;
                } else if self.action_move_status != 9 {
                    self.action_move_flag = 1;
                    off_bit8(&mut self.hide_bits, 1);
                    self.action_move_status = 9;
                }
            }
        } else if self.status_flags & ms::UNK20000 != 0 {
            if self.action_move_status != 2 {
                self.action_move_flag = 1;
                off_bit8(&mut self.hide_bits, 1);
            }
            self.action_move_status = 2;
        } else if self.globals.flag() != 0
            && (self.globals.push_menu_button() == 1
                || (self.globals.menu_status() == 1 && self.globals.push_menu_button() == 0))
        {
            if self.globals.item_mode() == 0 {
                if self.action_move_status != 3 {
                    self.action_move_flag = 1;
                    off_bit8(&mut self.hide_bits, 1);
                }
                self.action_move_status = 3;
            } else if self.action_move_status != 4 {
                self.action_move_flag = 1;
                off_bit8(&mut self.hide_bits, 1);
                self.action_move_status = 4;
            }
        } else if self.globals.flag() != 0
            && (self.globals.push_menu_button() == 2
                || (self.globals.menu_status() == 2 && self.globals.push_menu_button() == 0))
        {
            if self.action_move_status != 4 {
                self.action_move_flag = 1;
                off_bit8(&mut self.hide_bits, 1);
            }
            self.action_move_status = 4;
        } else if self.status_flags & ms::UNK800000 != 0 {
            if self.action_move_status != 4 {
                self.action_move_flag = 1;
                off_bit8(&mut self.hide_bits, 1);
            }
            self.action_move_status = 4;
        } else if self.status_flags & ms::UNK40000 != 0 {
            if self.action_move_status != 5 {
                self.action_move_flag = 1;
                off_bit8(&mut self.hide_bits, 1);
            }
            self.action_move_status = 5;
        } else if self.status_flags & ms::UNK100 != 0 {
            if self.action_move_status != 7 {
                self.action_move_flag = 1;
                off_bit8(&mut self.hide_bits, 1);
            }
            self.action_move_status = 7;
        } else if self.status_flags & ms::UNK80000 != 0 {
            if self.action_move_status != 6 {
                self.action_move_flag = 1;
                off_bit8(&mut self.hide_bits, 1);
            }
            self.action_move_status = 6;
        } else if self.status_flags & ms::UNK400000 != 0 {
            if self.action_move_status != 4 {
                self.action_move_flag = 1;
                off_bit8(&mut self.hide_bits, 1);
            }
            self.action_move_status = 4;
        } else if self.action_move_status != 0 {
            self.action_move_flag = 1;
            off_bit8(&mut self.hide_bits, 1);
            self.action_move_status = 0;
        }
        if self.hide_bits != 0xf {
            if !is_bit8(self.hide_bits, 1) {
                if self.action_move_frame >= 5 {
                    self.action_move_frame = 5;
                    self.action_move_flag = 0;
                    on_bit8(&mut self.hide_bits, 1);
                    match self.action_move_status {
                        1 => {
                            self.action_now_x = 0.0;
                            self.action_now_y = -200.0;
                        }
                        2 => {
                            self.action_now_x = tuning.m0b0 as f32;
                            self.action_now_y = tuning.m0b8 as f32;
                        }
                        3 => {
                            self.action_now_x = tuning.m0c0 as f32;
                            self.action_now_y = tuning.m0c8 as f32;
                        }
                        4 => {
                            self.action_now_x = tuning.m0d0 as f32;
                            self.action_now_y = tuning.m0d8 as f32;
                        }
                        5 => {
                            self.action_now_x = tuning.m0e0 as f32;
                            self.action_now_y = tuning.m0e8 as f32;
                        }
                        6 => {
                            self.action_now_x = tuning.m0e4 as f32;
                            self.action_now_y = tuning.m0ec as f32;
                        }
                        7 => {
                            self.action_now_x = tuning.m100 as f32;
                            self.action_now_y = tuning.m108 as f32;
                        }
                        8 => {
                            self.action_now_x = tuning.m110 as f32;
                            self.action_now_y = tuning.m118 as f32;
                        }
                        9 => {
                            if env.up_button == 1 {
                                self.action_now_x = tuning.m104 as f32;
                                self.action_now_y = tuning.m10c as f32;
                            } else if env.up_button == 0 {
                                self.action_now_x = tuning.m104 as f32 - (10 as f32);
                                self.action_now_y = tuning.m10c as f32 - (3 as f32);
                            }
                        }
                        10 => {
                            self.action_now_x = tuning.m110 as f32 + (0x2e as f32);
                            self.action_now_y = tuning.m118 as f32 - (0x17 as f32);
                        }
                        0xb => {
                            self.action_now_x = tuning.m110 as f32 + (0x2a as f32);
                            self.action_now_y = tuning.m118 as f32 - (3 as f32);
                        }
                        _ => {
                            self.action_now_x = tuning.m0f0 as f32;
                            self.action_now_y = tuning.m0f8 as f32;
                        }
                    }
                    if self.action_move_status == 2 {
                        self.action_exp_x = tuning.m0b4 as f32;
                        self.action_exp_y = tuning.m0bc as f32;
                    } else {
                        self.action_exp_x = 0.0;
                        self.action_exp_y = 0.0;
                    }
                } else {
                    self.action_move_frame += 1;
                }
            }
        } else if (self.action_move_frame as i32) <= 0 {
            self.action_move_frame = 0;
        } else {
            self.action_move_frame -= 1;
        }
        let progress = value_increase(5, self.action_move_frame as i32, 0);
        let x = self.action_now_x;
        let y = self.action_now_y + (progress * (-200.0 - self.action_now_y));
        if self.action_move_status == 10 {
            pane_trans(&mut self.action_blink, x + 27.0, y + 14.0);
        } else {
            pane_trans(&mut self.action_blink, x, y);
        }
        for index in 0..2 {
            pane_trans(&mut self.action_text[index], x, y);
            pane_trans(&mut self.action_text_shadow[index], x, y);
        }
        pane_trans(&mut self.count_tens, x, y);
        pane_trans(&mut self.count_ones, x, y);
        pane_trans(&mut self.action_text_main, x, y);
        pane_trans(&mut self.action_text_main_key, x, y);
        pane_trans(&mut self.center, x, y);
    }

    pub fn action_alpha(&mut self, env: &VitalsEnv) {
        let icon = self.globals.info.button_icon_mode;
        if self.status_flags & ms::UNK20 != 0
            && env.up_button == 0
            && (icon == 2 || icon == 3 || icon == 7 || icon == 8 || icon == 9)
        {
            if self.action_alpha_frame > 3 {
                self.action_alpha_frame -= 1;
            } else if self.action_alpha_frame < 3 {
                self.action_alpha_frame += 1;
            } else {
                self.action_alpha_frame = 3;
            }
        } else {
            on_bit8(&mut self.fade_bits, 1);
            if self.action_alpha_frame < 5 {
                self.action_alpha_frame += 1;
            } else {
                self.action_alpha_frame = 5;
            }
        }
        let progress = value_increase(5, self.action_alpha_frame as i32, 0);
        if env.mini_game_type == 8 {
            if let Some(rest) = env.mini_game_rest_arrow {
                if rest < 10 {
                    pane_hide(&mut self.count_tens);
                } else {
                    pane_show(&mut self.count_tens);
                    number_set(&mut self.count_tens, (rest / 10 & 0xff) as u8);
                }
                pane_show(&mut self.count_ones);
                number_set(&mut self.count_ones, (rest % 10 & 0xff) as u8);
            }
        } else {
            self.count_tens.node.hide();
            self.count_ones.node.hide();
        }
        if env.do_status == act::BLANK {
            set_now_alpha_zero(&mut self.action_blink);
            set_now_alpha_zero(&mut self.action_text[1]);
            set_now_alpha_zero(&mut self.action_text_shadow[1]);
            set_now_alpha_zero(&mut self.count_tens);
            set_now_alpha_zero(&mut self.count_ones);
            set_now_alpha_zero(&mut self.action_text_main);
            set_now_alpha_zero(&mut self.action_text_main_key);
            set_now_alpha(&mut self.action_text[0], progress);
            set_now_alpha(&mut self.action_text_shadow[0], progress);
        } else if env.do_status == act::PARRY
            || env.do_status == act::PARRY_31
            || env.do_status == act::THROW_2D
            || env.do_status == act::CARRY_2E
        {
            set_now_alpha_zero(&mut self.action_blink);
            set_now_alpha(&mut self.action_text[1], progress);
            set_now_alpha(&mut self.action_text_shadow[1], progress);
            set_now_alpha_zero(&mut self.action_text[0]);
            set_now_alpha_zero(&mut self.action_text_shadow[0]);
            set_now_alpha_zero(&mut self.count_tens);
            set_now_alpha_zero(&mut self.count_ones);
            set_now_alpha_zero(&mut self.action_text_main);
            set_now_alpha_zero(&mut self.action_text_main_key);
        } else if env.do_status == act::BOW {
            set_now_alpha_zero(&mut self.action_blink);
            set_now_alpha_zero(&mut self.action_text[1]);
            set_now_alpha_zero(&mut self.action_text_shadow[1]);
            set_now_alpha(&mut self.action_text[0], progress);
            set_now_alpha(&mut self.action_text_shadow[0], progress);
            set_now_alpha(&mut self.count_tens, progress);
            set_now_alpha(&mut self.count_ones, progress);
            set_now_alpha(&mut self.action_text_main, progress);
            set_now_alpha(&mut self.action_text_main_key, progress);
        } else {
            set_now_alpha(&mut self.action_blink, progress);
            set_now_alpha_zero(&mut self.action_text[1]);
            set_now_alpha_zero(&mut self.action_text_shadow[1]);
            set_now_alpha(&mut self.action_text[0], progress);
            set_now_alpha(&mut self.action_text_shadow[0], progress);
            set_now_alpha_zero(&mut self.count_tens);
            set_now_alpha_zero(&mut self.count_ones);
            set_now_alpha_zero(&mut self.action_text_main);
            set_now_alpha_zero(&mut self.action_text_main_key);
        }
        set_now_alpha(&mut self.center, progress);
        if is_bit8(self.hide_bits, 1) && self.action_text_main.user_area == 0 {
            self.action_blink.node.show();
            self.action_text_main.node.show();
            self.action_text_main_key.node.show();
        } else {
            self.action_blink.node.hide();
            self.count_tens.node.hide();
            self.count_ones.node.hide();
            self.action_text_main.node.hide();
            self.action_text_main_key.node.hide();
        }
    }

    pub fn xy_init(&mut self, env: &mut VitalsEnv) {
        self.xy_arrow_back[0].user_area = 0;
        self.rupy_light = [None, None];
        for index in 0..BAIT_COUNT {
            self.globals.bait_num[index] = env.bait_num[index];
        }
        for btn in 0..ITEM_BUTTON_COUNT {
            env.set_select_item(btn);
            self.select_cache[btn] = env.select_item[btn];
            self.xy_item_change(env, btn);
        }
    }

    pub fn xy_move(&mut self, env: &mut VitalsEnv) {
        let mut changed = true;
        for btn in 0..ITEM_BUTTON_COUNT {
            if self.select_cache[btn] != env.select_item[btn] {
                self.xy_item_change(env, btn);
                changed = false;
                if env.select_item[btn] != it::MAGIC_ARROW
                    && env.select_item[btn] != it::LIGHT_ARROW
                {
                    self.xy_item[btn].node.hide();
                    self.xy_item_key[btn].node.hide();
                    self.xy_bow_light[btn].node.hide();
                    self.xy_item[btn].user_area = 0;
                }
                self.select_cache[btn] = env.select_item[btn];
            }
        }
        if changed {
            self.xy_item_count_up(env);
        }
        self.xy_trans(env);
        let mut water = false;
        for btn in 0..ITEM_BUTTON_COUNT {
            if env.select_item[btn] == it::MAGIC_ARROW || env.select_item[btn] == it::LIGHT_ARROW {
                self.xy_bow_light_anime(btn);
            } else if env.select_item[btn] == it::FOREST_WATER {
                water = true;
                let x = self.xy_item[btn].pos_center.x - 320.0;
                let y = self.xy_item[btn].pos_center.y - 240.0;
                let alpha = self.xy_item[btn].now_alpha as f32;
                for index in 0..2 {
                    match self.rupy_light[index] {
                        Some(ref mut emitter) => {
                            emitter.set_global_translation(x, y, 0.0);
                            emitter.play_draw_particle();
                            emitter.set_global_alpha(alpha);
                        }
                        None => {
                            self.rupy_light[index] = Some(RupyEmitter::spawn(x, y, 0.0));
                        }
                    }
                }
            }
        }
        if !water {
            for index in 0..2 {
                if let Some(ref mut emitter) = self.rupy_light[index] {
                    emitter.become_invalid_emitter();
                    emitter.quit_immortal_emitter();
                }
                self.rupy_light[index] = None;
            }
        }
        self.xy_alpha(env);
    }

    pub fn xy_draw(&mut self) {
        for index in 0..3 {
            set_alpha(&mut self.xy_count_ones[index]);
            set_alpha(&mut self.xy_count_tens[index]);
            set_alpha(&mut self.xy_item[index]);
            set_alpha(&mut self.xy_item_key[index]);
            set_alpha(&mut self.xy_bow_light[index]);
            set_alpha(&mut self.xy_bow_light_2[index]);
            set_alpha(&mut self.xy_bow_light_3[index]);
        }
        set_alpha(&mut self.xy_arrow);
        for index in 0..2 {
            set_alpha(&mut self.xy_arrow_back[index]);
            set_alpha(&mut self.xy_arrow_00[index]);
            set_alpha(&mut self.xy_arrow_02[index]);
        }
    }

    pub fn xy_equip_item(&self, env: &VitalsEnv, btn: usize) -> u8 {
        let inv = env.select_slot[btn];
        let item_no = env.item(inv);
        match item_no {
            it::SKULL_NECKLACE => beast::SKULL_NECKLACE,
            it::BOKOBABA_SEED => beast::BOKOBABA_SEED,
            it::GOLDEN_FEATHER => beast::GOLDEN_FEATHER,
            it::KNIGHTS_CREST => beast::KNIGHTS_CREST,
            it::RED_JELLY => beast::RED_JELLY,
            it::GREEN_JELLY => beast::GREEN_JELLY,
            it::BLUE_JELLY => beast::BLUE_JELLY,
            it::JOY_PENDANT => beast::JOY_PENDANT,
            _ => 0,
        }
    }

    pub fn xy_item_number_set(&mut self, env: &VitalsEnv, btn: usize) {
        let selected = env.select_item[btn];
        let slot_index = env.select_slot[btn];
        if selected == it::PICTO_BOX || selected == it::DELUXE_PICTO_BOX {
            pane_hide(&mut self.xy_count_tens[btn]);
            pane_show(&mut self.xy_count_ones[btn]);
            number_set(&mut self.xy_count_ones[btn], env.picture_num);
            number_color(&mut self.xy_count_ones[btn], env.picture_num, 3);
        } else if selected == it::BOW || selected == it::MAGIC_ARROW || selected == it::LIGHT_ARROW
        {
            pane_show(&mut self.xy_count_tens[btn]);
            pane_show(&mut self.xy_count_ones[btn]);
            if env.arrow_num < 10 {
                pane_hide(&mut self.xy_count_tens[btn]);
            } else {
                number_set(&mut self.xy_count_tens[btn], env.arrow_num / 10);
                number_color(&mut self.xy_count_tens[btn], env.arrow_num, env.arrow_max);
            }
            number_set(&mut self.xy_count_ones[btn], env.arrow_num % 10);
            number_color(&mut self.xy_count_ones[btn], env.arrow_num, env.arrow_max);
        } else if selected == it::BOMB_BAG {
            pane_show(&mut self.xy_count_tens[btn]);
            pane_show(&mut self.xy_count_ones[btn]);
            if env.bomb_num < 10 {
                pane_hide(&mut self.xy_count_tens[btn]);
            } else {
                number_set(&mut self.xy_count_tens[btn], env.bomb_num / 10);
                number_color(&mut self.xy_count_tens[btn], env.bomb_num, env.bomb_max);
            }
            number_set(&mut self.xy_count_ones[btn], env.bomb_num % 10);
            number_color(&mut self.xy_count_ones[btn], env.bomb_num, env.bomb_max);
        } else if slot_index >= slot::BEAST_FIRST && slot_index <= slot::BEAST_LAST - 1 {
            pane_show(&mut self.xy_count_tens[btn]);
            pane_show(&mut self.xy_count_ones[btn]);
            let beast_idx = self.xy_equip_item(env, btn);
            let count = env.beast_num[beast_idx as usize];
            if count < 10 {
                pane_hide(&mut self.xy_count_tens[btn]);
            } else {
                number_set(&mut self.xy_count_tens[btn], count / 10);
                number_color(&mut self.xy_count_tens[btn], count, 99);
            }
            number_set(&mut self.xy_count_ones[btn], count % 10);
            number_color(&mut self.xy_count_ones[btn], count, 99);
        } else if slot_index >= slot::BAIT_FIRST && slot_index <= slot::BAIT_LAST - 1 {
            if selected == it::HYOI_PEAR {
                pane_hide(&mut self.xy_count_tens[btn]);
                pane_hide(&mut self.xy_count_ones[btn]);
            } else {
                pane_hide(&mut self.xy_count_tens[btn]);
                pane_show(&mut self.xy_count_ones[btn]);
                let count = env.bait_num[usize::from(slot_index - slot::BAIT_FIRST)];
                number_set(&mut self.xy_count_ones[btn], count);
                number_color(&mut self.xy_count_ones[btn], count, 3);
            }
        } else {
            pane_hide(&mut self.xy_count_tens[btn]);
            pane_hide(&mut self.xy_count_ones[btn]);
        }
    }

    pub fn xy_bow_light_anime(&mut self, btn: usize) {
        self.xy_bow_light[0].user_area += 1;
        if self.xy_bow_light[btn].user_area >= 0x78 {
            self.xy_bow_light[btn].user_area = 0;
        }
        let frame = self.xy_bow_light[btn].user_area as i32;
        let progress = if frame < 0x3c {
            value_increase(0x3c, frame, 0)
        } else {
            value_increase(0x3c, 0x78 - frame, 0)
        };
        let scale = 1.0 - (progress * 0.39999998);
        let init = self.xy_bow_light[0].init_alpha as f32;
        let alpha = init - ((init - 50.0) * progress);
        pane_scale_xy(&mut self.xy_bow_light[btn], scale);
        pane_scale_xy(&mut self.xy_bow_light_2[btn], scale);
        pane_scale_xy(&mut self.xy_bow_light_3[btn], scale);
        let byte = alpha as i64 as u8;
        let pane = &mut self.xy_bow_light[btn];
        for slot in 0..3 {
            match slot {
                0 => pane.now_alpha = byte,
                1 => pane.user_area = (pane.user_area & -256) | ((byte as i16) << 8),
                _ => pane.user_area = (pane.user_area & !0x00FF) | byte as i16,
            }
        }
    }

    pub fn xy_item_count_up(&mut self, env: &mut VitalsEnv) {
        let mut changed = false;
        if self.arrow_max_cached.is_none() {
            self.arrow_max_cached = Some(env.arrow_max);
        }
        if self.bomb_max_cached.is_none() {
            self.bomb_max_cached = Some(env.bomb_max);
        }
        let arrow_cap = self.arrow_max_cached.unwrap_or(0);
        let bomb_cap = self.bomb_max_cached.unwrap_or(0);
        if env.item_picture_num_count != 0 {
            let mut count = env.picture_num as i16 + env.item_picture_num_count;
            env.item_picture_num_count = 0;
            if count < 0 {
                count = 0;
            }
            if count > 3 {
                count = 3;
            }
            env.picture_num = count as u8;
            changed = true;
        }
        if (env.has_item(it::BOW) || env.has_item(it::MAGIC_ARROW) || env.has_item(it::LIGHT_ARROW))
            && env.item_arrow_num_count != 0
            || arrow_cap != env.arrow_max
        {
            if self.globals.tuning.m062 != 0 {
                env.item_arrow_num_count = 0;
            }
            let mut count = env.arrow_num as i16 + env.item_arrow_num_count as i16;
            env.item_arrow_num_count = 0;
            if count < 0 {
                count = 0;
            }
            if count > env.arrow_max as i16 {
                count = env.arrow_max as i16;
            }
            env.arrow_num = count as u8;
            self.arrow_max_cached = Some(env.arrow_max);
            changed = true;
        }
        if (env.has_item(it::BOMB_BAG) && env.item_bomb_num_count != 0) || bomb_cap != env.bomb_max
        {
            if self.globals.tuning.m063 != 0 {
                env.item_bomb_num_count = 0;
            }
            let mut count = env.bomb_num as i16 + env.item_bomb_num_count as i16;
            env.item_bomb_num_count = 0;
            if count < 0 {
                count = 0;
            }
            if count > env.bomb_max as i16 {
                count = env.bomb_max as i16;
            }
            env.bomb_num = count as u8;
            self.bomb_max_cached = Some(env.bomb_max);
            changed = true;
        }
        for beast_idx in 0..beast::COUNT {
            let index = beast_idx as usize;
            if env.item_beast_num_count[index] != 0 {
                let mut count = env.beast_num[index] as i16 + env.item_beast_num_count[index];
                env.item_beast_num_count[index] = 0;
                if count <= 0 {
                    env.beast_num[index] = 0;
                    for btn in 0..ITEM_BUTTON_COUNT {
                        if env.select_item[btn] == item_num_idx(beast_idx + slot::BEAST_FIRST) {
                            pane_hide(&mut self.xy_item[btn]);
                            pane_hide(&mut self.xy_item_key[btn]);
                            pane_hide(&mut self.xy_count_tens[btn]);
                            pane_hide(&mut self.xy_count_ones[btn]);
                        }
                    }
                    let item = item_num_idx(beast_idx + slot::BEAST_FIRST);
                    env.set_beast_item_empty(item);
                    changed = true;
                } else {
                    if count > 99 {
                        count = 99;
                    }
                    env.beast_num[index] = count as u8;
                    changed = true;
                }
            }
            if self.globals.bait_num[index] != env.bait_num[index] {
                self.globals.bait_num[index] = env.bait_num[index];
                changed = true;
            }
        }
        for btn in 0..ITEM_BUTTON_COUNT {
            if env.select_item[btn] == it::BIRD_BAIT_5 {
                changed = true;
            }
        }
        if changed {
            for btn in 0..ITEM_BUTTON_COUNT {
                self.xy_item_number_set(env, btn);
            }
        }
    }

    pub fn xy_item_change(&mut self, env: &mut VitalsEnv, btn: usize) {
        let selected = env.select_item[btn];
        let slot_index = env.select_slot[btn];
        if selected != it::NONE {
            if slot_index >= slot::RESERVE_FIRST {
                pane_show(&mut self.xy_item[btn]);
                pane_show(&mut self.xy_item_key[btn]);
                let filename = item_texture(selected);
                self.xy_icon_tex[btn].read(filename, Archive::ItemIcon);
                let name = self.xy_icon_tex[btn].name.clone();
                self.xy_item[btn].node.change_texture(&name);
                self.xy_item_key[btn].node.change_texture(&name);
            } else if slot_index >= slot::BAIT_FIRST {
                pane_show(&mut self.xy_item[btn]);
                pane_show(&mut self.xy_item_key[btn]);
                let filename = item_texture(selected);
                self.xy_icon_tex[btn].read(filename, Archive::ItemIcon);
                let name = self.xy_icon_tex[btn].name.clone();
                self.xy_item[btn].node.change_texture(&name);
                self.xy_item_key[btn].node.change_texture(&name);
            } else if slot_index >= slot::BEAST_FIRST {
                pane_show(&mut self.xy_item[btn]);
                pane_show(&mut self.xy_item_key[btn]);
                let filename = item_texture(selected);
                self.xy_icon_tex[btn].read(filename, Archive::ItemIcon);
                let name = self.xy_icon_tex[btn].name.clone();
                self.xy_item[btn].node.change_texture(&name);
                self.xy_item_key[btn].node.change_texture(&name);
            } else {
                pane_show(&mut self.xy_item[btn]);
                pane_show(&mut self.xy_item_key[btn]);
                let mut item_no = selected;
                if item_no == it::MAGIC_ARROW {
                    self.xy_bow_light[btn].node.show();
                    self.xy_bow_light_2[btn].node.show();
                    self.xy_bow_light_3[btn].node.hide();
                    item_no = it::BOW;
                } else if item_no == it::LIGHT_ARROW {
                    self.xy_bow_light[btn].node.show();
                    self.xy_bow_light_2[btn].node.show();
                    self.xy_bow_light_3[btn].node.show();
                    item_no = it::BOW;
                }
                if item_no == it::MAGIC_ARROW || item_no == it::LIGHT_ARROW {
                    item_no = it::BOW;
                }
                let filename = item_texture(item_no);
                self.xy_icon_tex[btn].read(filename, Archive::ItemIcon);
                let name = self.xy_icon_tex[btn].name.clone();
                self.xy_item[btn].node.change_texture(&name);
                self.xy_item_key[btn].node.change_texture(&name);
            }
        } else {
            pane_hide(&mut self.xy_item[btn]);
            pane_hide(&mut self.xy_item_key[btn]);
        }
        self.xy_item_number_set(env, btn);
    }

    pub fn xy_alpha(&mut self, env: &VitalsEnv) {
        if self.status_flags & ms::UNK4000 == 0
            && (self.status_flags & ms::UNK40 == 0 || !env.check_hind(1))
            && self.status_flags & ms::UNK100 == 0
            && self.status_flags & ms::UNK10 == 0
            && self.status_flags & ms::UNK800000 == 0
            && (self.status_flags & ms::UNK20 == 0 || env.up_button != 1)
            && self.status_flags & ms::UNK80 == 0
            && self.status_flags & ms::UNK100000 == 0
            && self.status_flags & ms::UNK200000 == 0
            && self.status_flags & ms::UNK20000 == 0
            && self.status_flags & ms::UNK40000 == 0
            && self.status_flags & ms::UNK80000 == 0
            && self.status_flags & ms::UNK400000 == 0
        {
            if self.status_flags & ms::UNK8 != 0 {
                for frame in self.xy_alpha_frame.iter_mut() {
                    if *frame < 5 {
                        *frame += 1;
                    } else {
                        *frame = 5;
                    }
                }
            } else if self.status_flags & ms::UNK8 == 0
                && (self.status_flags & (ms::UNK800 | ms::UNK2000) != 0
                    || self.status_flags & (ms::UNK8000 | ms::UNK200 | ms::UNK10000) != 0)
            {
                for frame in self.xy_alpha_frame.iter_mut() {
                    if *frame > 3 {
                        *frame -= 1;
                    } else if *frame < 3 {
                        *frame += 1;
                    } else {
                        *frame = 3;
                    }
                }
            } else if self.status_flags & ms::UNK20 != 0 && env.up_button == 0 {
                for index in 0..3 {
                    if index == 1 {
                        if self.xy_alpha_frame[index] < 5 {
                            self.xy_alpha_frame[index] += 1;
                        } else {
                            self.xy_alpha_frame[index] = 5;
                        }
                    } else if self.xy_alpha_frame[index] as i32 > 0 {
                        self.xy_alpha_frame[index] -= 1;
                    } else {
                        self.xy_alpha_frame[index] = 0;
                    }
                }
            } else {
                for btn in 0..ITEM_BUTTON_COUNT {
                    let selected = env.select_item[btn];
                    let ship_allowed = self.status_flags & ms::UNK400 != 0
                        && env.mini_game_type != 1
                        && (selected == it::SAIL
                            || selected == it::TELESCOPE
                            || selected == it::WIND_WAKER
                            || selected == it::GRAPPLING_HOOK
                            || selected == it::BOOMERANG
                            || selected == it::TINGLE_TUNER
                            || selected == it::PICTO_BOX
                            || selected == it::DELUXE_PICTO_BOX
                            || selected == it::MAGIC_ARMOR
                            || selected == it::BAIT_BAG
                            || selected == it::HYOI_PEAR
                            || selected == it::BIRD_BAIT_5
                            || selected == it::BOW
                            || selected == it::MAGIC_ARROW
                            || selected == it::LIGHT_ARROW
                            || selected == it::BOMB_BAG
                            || selected == it::HOOKSHOT
                            || selected == it::EMPTY_BOTTLE
                            || selected == it::RED_POTION
                            || selected == it::GREEN_POTION
                            || selected == it::BLUE_POTION
                            || selected == it::HALF_SOUP_BOTTLE
                            || selected == it::SOUP_BOTTLE
                            || selected == it::WATER_BOTTLE
                            || selected == it::FAIRY_BOTTLE
                            || selected == it::FIREFLY_BOTTLE
                            || selected == it::FOREST_WATER);
                    let minigame_blocked = env.mini_game_type == 1
                        && selected != it::SAIL
                        && selected != it::TELESCOPE;
                    let boss_blocked = env.stage_type == stage::TYPE_BOSS
                        && (env.start_stage_name == "Xboss0"
                            || env.start_stage_name == "Xboss1"
                            || env.start_stage_name == "Xboss2"
                            || env.start_stage_name == "Xboss3")
                        && (selected == it::WATER_BOTTLE
                            || selected == it::FIREFLY_BOTTLE
                            || selected == it::FOREST_WATER);
                    let misc_blocked = self.status_flags & ms::UNK2 != 0
                        && selected != it::TINGLE_TUNER
                        && (selected != it::WIND_WAKER || env.start_stage_name != "Otkura")
                        && (selected != it::GRAPPLING_HOOK || env.start_stage_name != "Abesso")
                        && selected != it::PICTO_BOX
                        && selected != it::DELUXE_PICTO_BOX
                        && selected != it::SPOILS_BAG
                        && selected != it::SKULL_NECKLACE
                        && selected != it::BOKOBABA_SEED
                        && selected != it::GOLDEN_FEATHER
                        && selected != it::KNIGHTS_CREST
                        && selected != it::RED_JELLY
                        && selected != it::GREEN_JELLY
                        && selected != it::BLUE_JELLY
                        && selected != it::JOY_PENDANT
                        && selected != it::BAIT_BAG
                        && selected != it::BIRD_BAIT_5
                        && selected != it::HYOI_PEAR
                        && selected != it::EMPTY_BOTTLE
                        && selected != it::RED_POTION
                        && selected != it::GREEN_POTION
                        && selected != it::BLUE_POTION
                        && selected != it::HALF_SOUP_BOTTLE
                        && selected != it::SOUP_BOTTLE
                        && selected != it::WATER_BOTTLE
                        && selected != it::FAIRY_BOTTLE
                        && selected != it::FIREFLY_BOTTLE
                        && selected != it::FOREST_WATER
                        && selected != it::DELIVERY_BAG
                        && selected != it::TOWN_FLOWER
                        && selected != it::SEA_FLOWER
                        && selected != it::EXOTIC_FLOWER
                        && selected != it::HEROS_FLAG
                        && selected != it::BIG_CATCH_FLAG
                        && selected != it::BIG_SALE_FLAG
                        && selected != it::PINWHEEL
                        && selected != it::SICKLE_MOON_FLAG
                        && selected != it::SKULL_TOWER_IDOL
                        && selected != it::FOUNTAIN_IDOL
                        && selected != it::POSTMAN_STATUE
                        && selected != it::SHOP_GURU_STATUE
                        && selected != it::FATHER_LETTER
                        && selected != it::NOTE_TO_MOM
                        && selected != it::MAGGIES_LETTER
                        && selected != it::MOBLINS_LETTER
                        && selected != it::CABANA_DEED
                        && selected != it::COMPLIMENTARY_ID
                        && selected != it::FILL_UP_COUPON;
                    let blocked = (self.status_flags & ms::UNK1000 != 0
                        && selected != it::GRAPPLING_HOOK)
                        || (self.status_flags & ms::UNK80 != 0 && selected != it::TELESCOPE)
                        || (self.status_flags & ms::UNK20000 != 0
                            && (selected != it::PICTO_BOX || selected != it::DELUXE_PICTO_BOX))
                        || !ship_allowed
                        || minigame_blocked
                        || boss_blocked
                        || misc_blocked;
                    if blocked {
                        if self.xy_alpha_frame[btn] > 3 {
                            self.xy_alpha_frame[btn] -= 1;
                        } else if self.xy_alpha_frame[btn] < 3 {
                            self.xy_alpha_frame[btn] += 1;
                        } else {
                            self.xy_alpha_frame[btn] = 3;
                        }
                    } else {
                        if self.xy_alpha_frame[btn] < 5 {
                            self.xy_alpha_frame[btn] += 1;
                        } else {
                            self.xy_alpha_frame[btn] = 5;
                        }
                        match btn {
                            0 => on_bit8(&mut self.fade_bits, 4),
                            1 => on_bit8(&mut self.fade_bits, 8),
                            2 => on_bit8(&mut self.fade_bits, 0x20),
                            _ => {}
                        }
                    }
                }
            }
        }
        let alpha0 = value_increase(5, self.xy_alpha_frame[0] as i32, 0);
        let alpha1 = value_increase(5, self.xy_alpha_frame[1] as i32, 0);
        let alpha2 = value_increase(5, self.xy_alpha_frame[2] as i32, 0);
        set_now_alpha(&mut self.xy_count_ones[0], alpha0);
        set_now_alpha(&mut self.xy_count_tens[0], alpha0);
        set_now_alpha(&mut self.xy_item[0], alpha0);
        set_now_alpha(&mut self.xy_item_key[0], alpha0);
        set_now_alpha(&mut self.xy_arrow_back[0], alpha0);
        set_now_alpha(&mut self.xy_arrow_00[0], alpha0);
        set_now_alpha(&mut self.xy_arrow_02[0], alpha0);
        set_now_alpha(&mut self.xy_arrow_back[1], alpha1);
        set_now_alpha(&mut self.xy_arrow_00[1], alpha1);
        set_now_alpha(&mut self.xy_arrow_02[1], alpha1);
        set_now_alpha(&mut self.xy_count_ones[2], alpha2);
        set_now_alpha(&mut self.xy_count_tens[2], alpha2);
        set_now_alpha(&mut self.xy_item[2], alpha2);
        set_now_alpha(&mut self.xy_item_key[2], alpha2);
        set_now_alpha(&mut self.xy_arrow, alpha2);
        if self.status_flags & ms::UNK20 != 0 && env.up_button == 0 {
            set_now_alpha(&mut self.xy_count_ones[1], alpha0);
            set_now_alpha(&mut self.xy_count_tens[1], alpha0);
            set_now_alpha(&mut self.xy_item[1], alpha0);
            set_now_alpha(&mut self.xy_item_key[1], alpha0);
            for index in 0..3 {
                set_now_alpha_zero(&mut self.xy_bow_light[index]);
                set_now_alpha_zero(&mut self.xy_bow_light_2[index]);
                set_now_alpha_zero(&mut self.xy_bow_light_3[index]);
            }
        } else {
            set_now_alpha(&mut self.xy_count_ones[1], alpha1);
            set_now_alpha(&mut self.xy_count_tens[1], alpha1);
            set_now_alpha(&mut self.xy_item[1], alpha1);
            set_now_alpha(&mut self.xy_item_key[1], alpha1);
        }
    }

    pub fn xy_trans(&mut self, env: &VitalsEnv) {
        let tuning = self.globals.tuning.clone();
        let icon = self.globals.info.button_icon_mode;
        if self.status_flags & ms::UNK4000 != 0
            || (self.status_flags & ms::UNK40 != 0 && env.check_hind(1))
            || self.status_flags & ms::UNK100 != 0
            || (self.globals.flag() != 0
                && (self.globals.push_menu_button() == 2
                    || (self.globals.menu_status() == 2 && self.globals.push_menu_button() == 0)))
            || self.status_flags & ms::UNK800000 != 0
            || (self.status_flags & ms::UNK20 != 0 && env.up_button == 1)
            || self.status_flags
                & (ms::UNK80
                    | ms::UNK100000
                    | ms::UNK200000
                    | ms::UNK20000
                    | ms::UNK40000
                    | ms::UNK80000
                    | ms::UNK400000)
                != 0
        {
            if self.xy_move_status != 1 {
                self.xy_move_flag = 1;
                off_bit8(&mut self.hide_bits, 4);
            }
            self.xy_move_status = 1;
        } else if self.globals.flag() != 0
            && (self.globals.push_menu_button() == 1
                || (self.globals.menu_status() == 1 && self.globals.push_menu_button() == 0))
        {
            if self.globals.item_mode() == 0 {
                if self.xy_move_status != 2 {
                    self.xy_move_flag = 1;
                    off_bit8(&mut self.hide_bits, 4);
                }
                self.xy_move_status = 2;
            } else if self.xy_move_status != 1 {
                self.xy_move_flag = 1;
                off_bit8(&mut self.hide_bits, 4);
                self.xy_move_status = 1;
            }
        } else if self.status_flags & ms::UNK20 != 0 {
            if icon == 0 || icon == 1 || icon == 2 || icon == 3 {
                if self.xy_move_status != 3 {
                    self.xy_move_flag = 1;
                    off_bit8(&mut self.hide_bits, 4);
                }
                self.xy_move_status = 3;
            } else if icon == 7 || icon == 8 || icon == 9 || icon == 10 {
                if self.xy_move_status != 4 {
                    self.xy_move_flag = 1;
                    off_bit8(&mut self.hide_bits, 4);
                }
                self.xy_move_status = 4;
            } else if self.xy_move_status != 1 {
                self.xy_move_flag = 1;
                off_bit8(&mut self.hide_bits, 4);
                self.xy_move_status = 1;
            }
        } else if self.xy_move_status != 0 {
            self.xy_move_flag = 1;
            off_bit8(&mut self.hide_bits, 4);
            self.xy_move_status = 0;
        }
        if self.hide_bits != 0xf {
            if !is_bit8(self.hide_bits, 4) {
                if self.xy_move_frame >= 5 {
                    self.xy_move_frame = 5;
                    self.xy_move_flag = 0;
                    on_bit8(&mut self.hide_bits, 4);
                    match self.xy_move_status {
                        1 => {
                            self.xy_now_x = 0.0;
                            self.xy_now_y = -200.0;
                        }
                        2 => {
                            self.xy_now_x = tuning.m0c4 as f32;
                            self.xy_now_y = tuning.m0cc as f32;
                        }
                        3 => {
                            self.xy_now_x = tuning.m114 as f32;
                            self.xy_now_y = tuning.m11c as f32;
                        }
                        4 => {
                            self.xy_now_x = (tuning.m114 + 0x28) as f32;
                            self.xy_now_y = tuning.m11c as f32;
                        }
                        _ => {
                            self.xy_now_x = tuning.m0f4 as f32;
                            self.xy_now_y = tuning.m0fc as f32;
                        }
                    }
                } else {
                    self.xy_move_frame += 1;
                }
            }
        } else if (self.xy_move_frame as i32) <= 0 {
            self.xy_move_frame = 0;
        } else {
            self.xy_move_frame -= 1;
        }
        let progress = value_increase(5, self.xy_move_frame as i32, 0);
        let x = self.xy_now_x;
        let y = self.xy_now_y + (progress * (-200.0 - self.xy_now_y));
        for index in 0..3 {
            pane_trans(&mut self.xy_count_ones[index], x, y);
            pane_trans(&mut self.xy_count_tens[index], x, y);
            pane_trans(&mut self.xy_item[index], x, y);
            pane_trans(&mut self.xy_item_key[index], x, y);
            pane_trans(&mut self.xy_bow_light[index], x, y);
            pane_trans(&mut self.xy_bow_light_2[index], x, y);
            pane_trans(&mut self.xy_bow_light_3[index], x, y);
        }
        pane_trans(&mut self.xy_arrow, x, y);
        for index in 0..2 {
            pane_trans(&mut self.xy_arrow_back[index], x, y);
            pane_trans(&mut self.xy_arrow_00[index], x, y);
            pane_trans(&mut self.xy_arrow_02[index], x, y);
        }
    }

    pub fn quads(&self) -> Vec<Quad> {
        let mut out = Vec::new();
        for index in 0..HEART_COUNT {
            out.push(self.heart[index].quad());
            out.push(self.heart_shadow[index].quad());
        }
        out.push(self.wepon_blink.quad());
        out.push(self.wepon_text.quad());
        out.push(self.wepon_text_key.quad());
        out.push(self.wepon_back.quad());
        out.push(self.wepon_add.quad());
        out.push(self.action_blink.quad());
        for index in 0..2 {
            out.push(self.action_text[index].quad());
            out.push(self.action_text_shadow[index].quad());
        }
        out.push(self.count_tens.quad());
        out.push(self.count_ones.quad());
        out.push(self.action_text_main.quad());
        out.push(self.action_text_main_key.quad());
        out.push(self.center.quad());
        for index in 0..ITEM_BUTTON_COUNT {
            out.push(self.xy_count_ones[index].quad());
            out.push(self.xy_count_tens[index].quad());
            out.push(self.xy_item[index].quad());
            out.push(self.xy_item_key[index].quad());
            out.push(self.xy_bow_light[index].quad());
            out.push(self.xy_bow_light_2[index].quad());
            out.push(self.xy_bow_light_3[index].quad());
        }
        out.push(self.xy_arrow.quad());
        for index in 0..2 {
            out.push(self.xy_arrow_back[index].quad());
            out.push(self.xy_arrow_00[index].quad());
            out.push(self.xy_arrow_02[index].quad());
        }
        out
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct VitalsDraw {
    pub layers: Vec<ScreenLayer>,
    pub quads: Vec<Quad>,
}

#[derive(SystemParam)]
pub struct VitalsWorld<'w> {
    pub vitals: Option<ResMut<'w, Vitals>>,
    pub env: Option<ResMut<'w, VitalsEnv>>,
    pub screens: Option<Res<'w, Screens>>,
    pub draw: Option<ResMut<'w, VitalsDraw>>,
}

pub fn hud_vitals(mut world: VitalsWorld<'_>) {
    let Some(screens) = world.screens.as_ref() else {
        return;
    };
    let (Some(vitals), Some(env)) = (world.vitals.as_mut(), world.env.as_mut()) else {
        return;
    };
    if vitals.bind_pending && vitals.bind_screens(screens) {
        vitals.create(env);
    }
    vitals.tick(env);
    let layers = meter1_draw(screens);
    let quads = vitals.quads();
    if let Some(mut draw) = world.draw {
        draw.layers = layers;
        draw.quads = quads;
    }
}

pub fn hud_vitals_draw(mut world: VitalsWorld<'_>) {
    let Some(vitals) = world.vitals.as_mut() else {
        return;
    };
    vitals.heart_draw();
    vitals.wepon_draw();
    vitals.action_draw();
    vitals.xy_draw();
    let quads = vitals.quads();
    if let Some(mut draw) = world.draw {
        draw.quads = quads;
    }
}
