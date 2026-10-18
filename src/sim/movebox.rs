use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::pickup;
use super::player::Player;
use super::pushable::{
    Face, PPLABEL_HEAVY, PPLABEL_PULL, PPLABEL_PUSH_PULL, PPLABEL_UNK4, PushPulls,
};
use super::sea::Sea;
use super::stream;
use super::walk::from_s16;
use super::water;

pub const MOVEBOX_ACTORS: [&str; 15] = [
    "osiBLK0", "osiBLK1", "Kkiba", "Hseki2", "Hseki7", "Mmrr", "MkieBB", "Ecube", "Hjump1",
    "Hbox1", "MpwrB", "DBLK0", "DBLK1", "DKkiba", "Hbox2",
];

pub fn is_movebox(name: &str) -> bool {
    MOVEBOX_ACTORS.contains(&name)
}

const PRM_TYPE_W: u32 = 0x04;
const PRM_TYPE_S: u32 = 0x18;
const PRM_SWSAVE_W: u32 = 0x08;
const PRM_SWSAVE_S: u32 = 0x08;
const PRM_ITEMNO_W: u32 = 0x06;
const PRM_ITEMNO_S: u32 = 0x00;
const PRM_ITEMSAVE_W: u32 = 0x07;
const PRM_ITEMSAVE_S: u32 = 0x10;
const PRM_DMY_W: u32 = 0x01;
const PRM_DMY_S: u32 = 0x1E;
const PRM_BUOY_W: u32 = 0x01;
const PRM_BUOY_S: u32 = 0x1F;

pub const TYPE_BREAKABLE_WOODEN_CRATE: i32 = 0x0;
pub const TYPE_BLACK_BOX: i32 = 0x1;
pub const TYPE_BLACK_BOX_WITH_STATUE: i32 = 0x2;
pub const TYPE_BIG_BLACK_BOX: i32 = 0x3;
pub const TYPE_WOODEN_CRATE: i32 = 0x4;
pub const TYPE_GOLDEN_CRATE: i32 = 0x5;
pub const TYPE_METAL_BOX: i32 = 0x6;
pub const TYPE_METAL_BOX_WITH_SPRING: i32 = 0x7;
pub const TYPE_WOODEN_CRATE_2: i32 = 0x8;
pub const TYPE_WOODEN_CRATE_3: i32 = 0x9;
pub const TYPE_MIRROR: i32 = 0xA;
pub const TYPE_BLACK_BOX_WITH_MKIE: i32 = 0xB;
pub const TYPE_MOSSY_BLACK_BOX: i32 = 0xC;
pub const TYPE_COUNT: usize = 13;

const SE_LK_MOVE_ROCK: u32 = 0x2022;
const SE_LK_MOVE_WBLOCK: u32 = 0x2026;
const SE_LK_MOVE_ROCK_LIMIT: u32 = 0x2823;
const SE_LK_MOVE_WBLOCK_LIMIT: u32 = 0x2827;
const SE_OBJ_COL_SWC_WODCRL: u32 = 0x6805;
const SE_OBJ_FALL_WATER_M: u32 = 0x6919;
const SE_OBJ_FALL_WATER_L: u32 = 0x691A;
const SE_OBJ_FALL_MAGMA_M: u32 = 0x691C;
const SE_OBJ_FALL_MAGMA_L: u32 = 0x691D;
const SE_OBJ_BLOCK_FALL_NORMAL: u32 = 0x691F;
const SE_OBJ_WBOX_FALL_NORMAL: u32 = 0x6920;

const BGC_ON_GROUND: u32 = 0x01;
const BGC_JUST_LEFT_GROUND: u32 = 0x02;
const BGC_JUST_HIT_GROUND: u32 = 0x04;
const BGC_ABOVE_WATER: u32 = 0x08;
const BGC_IN_WATER: u32 = 0x10;
const BGC_JUST_HIT_WATER: u32 = 0x20;

const SLOT_COUNT: usize = 23;
const SIDE_SLOTS: usize = 4;
const SMOKE_CBS: usize = 2;

const GROUND_SLOT_Y: f32 = 100.0;
const WALL_PRE_LEN: f32 = 74.0;
const WALL_TOUCH_LEN: f32 = 10.0;
const WALL_TILT: i16 = 0x4000;
const HALF: f32 = 0.5;
const HALF_TURN: f32 = 32768.0;
const SWAY_TENSION: f32 = 0.9;
const SWAY_BIAS: f32 = 0.1;
const SMOKE_LIFT: f32 = 10.0;
const AFLOAT_LOCK: i16 = 20;
const LAND_DELAY: i16 = 0x14;
const NO_SWITCH: i32 = 0xFF;
const NO_PATH_ID: i32 = 0xFF;
const NO_ITEM_SAVE: i32 = 0x7F;
const ITEM_MAX_BIT: i32 = 79;
const G_CM3D_F_INF: f32 = -3.4028235e38;
const FLOAT_MAX: f32 = 3.4028235e38;
const MAX_FALL: f32 = -100.0;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BgcSrc {
    pub m00: f32,
    pub m04: f32,
    pub m08: f32,
    pub m0c: f32,
}

pub const M_LIN5: [BgcSrc; 5] = [
    BgcSrc {
        m00: 0.0,
        m04: 0.0,
        m08: 0.0,
        m0c: 0.0,
    },
    BgcSrc {
        m00: -0.5,
        m04: -0.5,
        m08: 1.0,
        m0c: 1.0,
    },
    BgcSrc {
        m00: 0.5,
        m04: -0.5,
        m08: -1.0,
        m0c: 1.0,
    },
    BgcSrc {
        m00: 0.5,
        m04: 0.5,
        m08: -1.0,
        m0c: -1.0,
    },
    BgcSrc {
        m00: -0.5,
        m04: 0.5,
        m08: 1.0,
        m0c: -1.0,
    },
];

pub const M_LIN20: [BgcSrc; 23] = [
    BgcSrc {
        m00: 0.0,
        m04: 0.0,
        m08: 0.0,
        m0c: 0.0,
    },
    BgcSrc {
        m00: -0.25,
        m04: -0.25,
        m08: 0.0,
        m0c: 0.0,
    },
    BgcSrc {
        m00: -0.5,
        m04: -0.5,
        m08: 1.0,
        m0c: 1.0,
    },
    BgcSrc {
        m00: 0.0,
        m04: -0.5,
        m08: -1.0,
        m0c: 1.0,
    },
    BgcSrc {
        m00: 0.0,
        m04: 0.0,
        m08: -1.0,
        m0c: -1.0,
    },
    BgcSrc {
        m00: -0.5,
        m04: 0.0,
        m08: 1.0,
        m0c: -1.0,
    },
    BgcSrc {
        m00: 0.25,
        m04: -0.25,
        m08: 0.0,
        m0c: 0.0,
    },
    BgcSrc {
        m00: 0.0,
        m04: -0.5,
        m08: 1.0,
        m0c: 1.0,
    },
    BgcSrc {
        m00: 0.5,
        m04: -0.5,
        m08: -1.0,
        m0c: 1.0,
    },
    BgcSrc {
        m00: 0.5,
        m04: 0.0,
        m08: -1.0,
        m0c: -1.0,
    },
    BgcSrc {
        m00: 0.0,
        m04: 0.0,
        m08: 1.0,
        m0c: -1.0,
    },
    BgcSrc {
        m00: 0.25,
        m04: 0.25,
        m08: 0.0,
        m0c: 0.0,
    },
    BgcSrc {
        m00: 0.0,
        m04: 0.0,
        m08: 1.0,
        m0c: 1.0,
    },
    BgcSrc {
        m00: 0.5,
        m04: 0.0,
        m08: -1.0,
        m0c: 1.0,
    },
    BgcSrc {
        m00: 0.5,
        m04: 0.5,
        m08: -1.0,
        m0c: -1.0,
    },
    BgcSrc {
        m00: 0.0,
        m04: 0.5,
        m08: 1.0,
        m0c: -1.0,
    },
    BgcSrc {
        m00: -0.25,
        m04: 0.25,
        m08: 0.0,
        m0c: 0.0,
    },
    BgcSrc {
        m00: -0.5,
        m04: 0.0,
        m08: 1.0,
        m0c: 1.0,
    },
    BgcSrc {
        m00: 0.0,
        m04: 0.0,
        m08: -1.0,
        m0c: 1.0,
    },
    BgcSrc {
        m00: 0.0,
        m04: 0.5,
        m08: -1.0,
        m0c: -1.0,
    },
    BgcSrc {
        m00: -0.5,
        m04: 0.5,
        m08: 1.0,
        m0c: -1.0,
    },
    BgcSrc {
        m00: -0.75,
        m04: 0.25,
        m08: 0.0,
        m0c: 0.0,
    },
    BgcSrc {
        m00: -0.75,
        m04: -0.25,
        m08: 0.0,
        m0c: 0.0,
    },
];

pub const M_DIR_BASE: [i16; SIDE_SLOTS] = [0x0000, 0x4000, i16::MIN, -0x4000];

const DIR_VEC: [Vec3; SIDE_SLOTS] = [
    Vec3::new(0.0, 0.0, 1.0),
    Vec3::new(1.0, 0.0, 0.0),
    Vec3::new(0.0, 0.0, -1.0),
    Vec3::new(-1.0, 0.0, 0.0),
];

const SMOKE_BASE_POS: [Vec3; SMOKE_CBS] = [Vec3::new(-0.5, 0.0, -0.5), Vec3::new(0.5, 0.0, -0.5)];

const WALL_PROBE_DIR: Vec3 = Vec3::new(0.0, 0.0, -1.0);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Attr {
    pub m00: i16,
    pub m02: i16,
    pub m04: i16,
    pub m06: i16,
    pub m08: i16,
    pub m0a: i16,
    pub m0c: f32,
    pub m10: f32,
    pub m14: f32,
    pub m18: f32,
    pub m1c: f32,
    pub m20: f32,
    pub land_smoke_scale: f32,
    pub m28: f32,
    pub m2c: f32,
    pub m30: f32,
    pub m34: f32,
    pub m38: i16,
    pub m3c: f32,
    pub m40: f32,
    pub m44: f32,
    pub m48: f32,
    pub m4c: f32,
    pub m50: f32,
    pub m54: f32,
    pub scale_y: f32,
    pub m68: f32,
    pub m6c: f32,
    pub scale_xz: f32,
    pub m74: f32,
    pub move_se: u32,
    pub cant_move_se: u32,
    pub normal_fall_se: u32,
    pub water_fall_se: u32,
    pub magma_fall_se: u32,
    pub cull_min: [f32; 3],
    pub cull_max: [f32; 3],
    pub use_bg_tev: bool,
    pub casts_shadow: bool,
    pub m9a: bool,
}

const STD_MIN: [f32; 3] = [-90.0, -1.0, -90.0];
const STD_MAX: [f32; 3] = [90.0, 151.0, 90.0];
const BIG_MIN: [f32; 3] = [-180.0, -1.0, -180.0];
const BIG_MAX: [f32; 3] = [180.0, 301.0, 180.0];
const MIRROR_MIN: [f32; 3] = [-150.0, -1.0, -150.0];
const MIRROR_MAX: [f32; 3] = [150.0, 681.0, 150.0];
const STATUE_MAX: [f32; 3] = [90.0, 295.0, 90.0];
const SPRING_MAX: [f32; 3] = [90.0, 200.0, 90.0];

pub const M_ATTR: [Attr; TYPE_COUNT] = [
    Attr {
        m00: 4,
        m02: 4,
        m04: 20,
        m06: 4,
        m08: 4,
        m0a: 20,
        m0c: 75.0,
        m10: 90.0,
        m14: -3.0,
        m18: 0.005,
        m1c: 0.001,
        m20: 0.0,
        land_smoke_scale: 1.8,
        m28: 3.9,
        m2c: -0.39,
        m30: -0.2,
        m34: 0.02,
        m38: 1000,
        m3c: 0.04,
        m40: 0.013,
        m44: 0.15,
        m48: 0.1,
        m4c: 0.1,
        m50: 0.06,
        m54: 0.075,
        scale_y: 150.0,
        m68: 150.0,
        m6c: 1.0 / 150.0,
        scale_xz: 150.0,
        m74: 1.0 / 150.0,
        move_se: SE_LK_MOVE_WBLOCK,
        cant_move_se: SE_LK_MOVE_WBLOCK_LIMIT,
        normal_fall_se: SE_OBJ_WBOX_FALL_NORMAL,
        water_fall_se: SE_OBJ_FALL_WATER_M,
        magma_fall_se: SE_OBJ_FALL_MAGMA_M,
        cull_min: STD_MIN,
        cull_max: STD_MAX,
        use_bg_tev: false,
        casts_shadow: false,
        m9a: false,
    },
    Attr {
        m00: 4,
        m02: 4,
        m04: 20,
        m06: 4,
        m08: 4,
        m0a: 20,
        m0c: 75.0,
        m10: 90.0,
        m14: -5.0,
        m18: 0.005,
        m1c: 0.001,
        m20: 0.0,
        land_smoke_scale: 1.8,
        m28: 4.5,
        m2c: -0.5,
        m30: -0.2,
        m34: 0.02,
        m38: 1000,
        m3c: 0.04,
        m40: 0.013,
        m44: 0.15,
        m48: 0.1,
        m4c: 0.1,
        m50: 0.06,
        m54: 0.075,
        scale_y: 150.0,
        m68: 150.0,
        m6c: 1.0 / 150.0,
        scale_xz: 150.0,
        m74: 1.0 / 150.0,
        move_se: SE_LK_MOVE_ROCK,
        cant_move_se: SE_LK_MOVE_ROCK_LIMIT,
        normal_fall_se: SE_OBJ_BLOCK_FALL_NORMAL,
        water_fall_se: SE_OBJ_FALL_WATER_L,
        magma_fall_se: SE_OBJ_FALL_MAGMA_L,
        cull_min: STD_MIN,
        cull_max: STD_MAX,
        use_bg_tev: false,
        casts_shadow: false,
        m9a: false,
    },
    Attr {
        m00: 4,
        m02: 4,
        m04: 20,
        m06: 4,
        m08: 4,
        m0a: 20,
        m0c: 75.0,
        m10: 90.0,
        m14: -5.0,
        m18: 0.005,
        m1c: 0.001,
        m20: 0.0,
        land_smoke_scale: 1.8,
        m28: 4.5,
        m2c: -0.5,
        m30: -0.2,
        m34: 0.02,
        m38: 1000,
        m3c: 0.04,
        m40: 0.013,
        m44: 0.15,
        m48: 0.1,
        m4c: 0.1,
        m50: 0.06,
        m54: 0.075,
        scale_y: 150.0,
        m68: 300.0,
        m6c: 1.0 / 300.0,
        scale_xz: 150.0,
        m74: 1.0 / 150.0,
        move_se: SE_LK_MOVE_ROCK,
        cant_move_se: SE_LK_MOVE_ROCK_LIMIT,
        normal_fall_se: SE_OBJ_BLOCK_FALL_NORMAL,
        water_fall_se: SE_OBJ_FALL_WATER_L,
        magma_fall_se: SE_OBJ_FALL_MAGMA_L,
        cull_min: STD_MIN,
        cull_max: STATUE_MAX,
        use_bg_tev: false,
        casts_shadow: false,
        m9a: false,
    },
    Attr {
        m00: 4,
        m02: 4,
        m04: 20,
        m06: 4,
        m08: 4,
        m0a: 20,
        m0c: 75.0,
        m10: 170.0,
        m14: -5.0,
        m18: 0.005,
        m1c: 0.001,
        m20: 0.0,
        land_smoke_scale: 3.6,
        m28: 4.5,
        m2c: -0.5,
        m30: -0.2,
        m34: 0.02,
        m38: 1000,
        m3c: 0.04,
        m40: 0.013,
        m44: 0.15,
        m48: 0.1,
        m4c: 0.1,
        m50: 0.06,
        m54: 0.075,
        scale_y: 300.0,
        m68: 300.0,
        m6c: 1.0 / 300.0,
        scale_xz: 300.0,
        m74: 1.0 / 300.0,
        move_se: SE_LK_MOVE_ROCK,
        cant_move_se: SE_LK_MOVE_ROCK_LIMIT,
        normal_fall_se: SE_OBJ_BLOCK_FALL_NORMAL,
        water_fall_se: SE_OBJ_FALL_WATER_L,
        magma_fall_se: SE_OBJ_FALL_MAGMA_L,
        cull_min: BIG_MIN,
        cull_max: BIG_MAX,
        use_bg_tev: false,
        casts_shadow: false,
        m9a: true,
    },
    Attr {
        m00: 4,
        m02: 4,
        m04: 20,
        m06: 4,
        m08: 4,
        m0a: 20,
        m0c: 75.0,
        m10: 90.0,
        m14: -3.0,
        m18: 0.005,
        m1c: 0.001,
        m20: 0.0,
        land_smoke_scale: 1.8,
        m28: 3.9,
        m2c: -0.39,
        m30: -0.2,
        m34: 0.02,
        m38: 1000,
        m3c: 0.04,
        m40: 0.013,
        m44: 0.15,
        m48: 0.1,
        m4c: 0.1,
        m50: 0.06,
        m54: 0.075,
        scale_y: 150.0,
        m68: 150.0,
        m6c: 1.0 / 150.0,
        scale_xz: 150.0,
        m74: 1.0 / 150.0,
        move_se: SE_LK_MOVE_WBLOCK,
        cant_move_se: SE_LK_MOVE_WBLOCK_LIMIT,
        normal_fall_se: SE_OBJ_WBOX_FALL_NORMAL,
        water_fall_se: SE_OBJ_FALL_WATER_M,
        magma_fall_se: SE_OBJ_FALL_MAGMA_M,
        cull_min: STD_MIN,
        cull_max: STD_MAX,
        use_bg_tev: false,
        casts_shadow: false,
        m9a: false,
    },
    Attr {
        m00: 4,
        m02: 4,
        m04: 20,
        m06: 4,
        m08: 4,
        m0a: 20,
        m0c: 75.0,
        m10: 90.0,
        m14: -3.0,
        m18: 0.005,
        m1c: 0.001,
        m20: 0.0,
        land_smoke_scale: 1.8,
        m28: 3.9,
        m2c: -0.39,
        m30: -0.2,
        m34: 0.02,
        m38: 1000,
        m3c: 0.04,
        m40: 0.013,
        m44: 0.15,
        m48: 0.1,
        m4c: 0.1,
        m50: 0.06,
        m54: 0.075,
        scale_y: 150.0,
        m68: 150.0,
        m6c: 1.0 / 150.0,
        scale_xz: 150.0,
        m74: 1.0 / 150.0,
        move_se: SE_LK_MOVE_WBLOCK,
        cant_move_se: SE_LK_MOVE_WBLOCK_LIMIT,
        normal_fall_se: SE_OBJ_WBOX_FALL_NORMAL,
        water_fall_se: SE_OBJ_FALL_WATER_M,
        magma_fall_se: SE_OBJ_FALL_MAGMA_M,
        cull_min: STD_MIN,
        cull_max: STD_MAX,
        use_bg_tev: false,
        casts_shadow: false,
        m9a: false,
    },
    Attr {
        m00: 4,
        m02: 4,
        m04: 20,
        m06: 4,
        m08: 4,
        m0a: 20,
        m0c: 75.0,
        m10: 90.0,
        m14: -3.0,
        m18: 0.005,
        m1c: 0.001,
        m20: 0.0,
        land_smoke_scale: 1.8,
        m28: 3.9,
        m2c: -0.39,
        m30: -0.2,
        m34: 0.02,
        m38: 1000,
        m3c: 0.04,
        m40: 0.013,
        m44: 0.15,
        m48: 0.1,
        m4c: 0.1,
        m50: 0.06,
        m54: 0.075,
        scale_y: 150.0,
        m68: 150.0,
        m6c: 1.0 / 150.0,
        scale_xz: 150.0,
        m74: 1.0 / 150.0,
        move_se: SE_LK_MOVE_ROCK,
        cant_move_se: SE_LK_MOVE_ROCK_LIMIT,
        normal_fall_se: SE_OBJ_BLOCK_FALL_NORMAL,
        water_fall_se: SE_OBJ_FALL_WATER_L,
        magma_fall_se: SE_OBJ_FALL_MAGMA_L,
        cull_min: STD_MIN,
        cull_max: STD_MAX,
        use_bg_tev: false,
        casts_shadow: false,
        m9a: false,
    },
    Attr {
        m00: 4,
        m02: 4,
        m04: 20,
        m06: 4,
        m08: 4,
        m0a: 20,
        m0c: 75.0,
        m10: 90.0,
        m14: -3.0,
        m18: 0.005,
        m1c: 0.001,
        m20: 0.0,
        land_smoke_scale: 1.8,
        m28: 3.9,
        m2c: -0.39,
        m30: -0.2,
        m34: 0.02,
        m38: 1000,
        m3c: 0.04,
        m40: 0.013,
        m44: 0.15,
        m48: 0.1,
        m4c: 0.1,
        m50: 0.06,
        m54: 0.075,
        scale_y: 150.0,
        m68: 300.0,
        m6c: 1.0 / 300.0,
        scale_xz: 150.0,
        m74: 1.0 / 150.0,
        move_se: SE_LK_MOVE_ROCK,
        cant_move_se: SE_LK_MOVE_ROCK_LIMIT,
        normal_fall_se: SE_OBJ_BLOCK_FALL_NORMAL,
        water_fall_se: SE_OBJ_FALL_WATER_L,
        magma_fall_se: SE_OBJ_FALL_MAGMA_L,
        cull_min: STD_MIN,
        cull_max: SPRING_MAX,
        use_bg_tev: false,
        casts_shadow: false,
        m9a: false,
    },
    Attr {
        m00: 4,
        m02: 4,
        m04: 20,
        m06: 4,
        m08: 4,
        m0a: 20,
        m0c: 75.0,
        m10: 90.0,
        m14: -3.0,
        m18: 0.005,
        m1c: 0.001,
        m20: 0.0,
        land_smoke_scale: 1.8,
        m28: 3.9,
        m2c: -0.39,
        m30: -0.2,
        m34: 0.02,
        m38: 1000,
        m3c: 0.04,
        m40: 0.013,
        m44: 0.15,
        m48: 0.1,
        m4c: 0.1,
        m50: 0.06,
        m54: 0.075,
        scale_y: 150.0,
        m68: 150.0,
        m6c: 1.0 / 150.0,
        scale_xz: 150.0,
        m74: 1.0 / 150.0,
        move_se: SE_LK_MOVE_WBLOCK,
        cant_move_se: SE_LK_MOVE_WBLOCK_LIMIT,
        normal_fall_se: SE_OBJ_WBOX_FALL_NORMAL,
        water_fall_se: SE_OBJ_FALL_WATER_M,
        magma_fall_se: SE_OBJ_FALL_MAGMA_M,
        cull_min: STD_MIN,
        cull_max: STD_MAX,
        use_bg_tev: false,
        casts_shadow: false,
        m9a: false,
    },
    Attr {
        m00: 4,
        m02: 4,
        m04: 20,
        m06: 4,
        m08: 4,
        m0a: 20,
        m0c: 75.0,
        m10: 90.0,
        m14: -3.0,
        m18: 0.005,
        m1c: 0.001,
        m20: 0.0,
        land_smoke_scale: 1.8,
        m28: 3.9,
        m2c: -0.39,
        m30: -0.2,
        m34: 0.02,
        m38: 1000,
        m3c: 0.04,
        m40: 0.013,
        m44: 0.15,
        m48: 0.1,
        m4c: 0.1,
        m50: 0.06,
        m54: 0.075,
        scale_y: 150.0,
        m68: 150.0,
        m6c: 1.0 / 150.0,
        scale_xz: 150.0,
        m74: 1.0 / 150.0,
        move_se: SE_LK_MOVE_WBLOCK,
        cant_move_se: SE_LK_MOVE_WBLOCK_LIMIT,
        normal_fall_se: SE_OBJ_WBOX_FALL_NORMAL,
        water_fall_se: SE_OBJ_FALL_WATER_M,
        magma_fall_se: SE_OBJ_FALL_MAGMA_M,
        cull_min: STD_MIN,
        cull_max: STD_MAX,
        use_bg_tev: false,
        casts_shadow: false,
        m9a: false,
    },
    Attr {
        m00: 4,
        m02: 4,
        m04: 20,
        m06: 4,
        m08: 4,
        m0a: 20,
        m0c: 75.0,
        m10: 170.0,
        m14: -5.0,
        m18: 0.005,
        m1c: 0.001,
        m20: 0.0,
        land_smoke_scale: 1.8,
        m28: 4.5,
        m2c: -0.5,
        m30: -0.2,
        m34: 0.02,
        m38: 1000,
        m3c: 0.04,
        m40: 0.013,
        m44: 0.15,
        m48: 0.1,
        m4c: 0.1,
        m50: 0.06,
        m54: 0.075,
        scale_y: 300.0,
        m68: 450.0,
        m6c: 1.0 / 450.0,
        scale_xz: 300.0,
        m74: 1.0 / 300.0,
        move_se: SE_LK_MOVE_ROCK,
        cant_move_se: SE_LK_MOVE_ROCK_LIMIT,
        normal_fall_se: SE_OBJ_BLOCK_FALL_NORMAL,
        water_fall_se: SE_OBJ_FALL_WATER_L,
        magma_fall_se: SE_OBJ_FALL_MAGMA_L,
        cull_min: MIRROR_MIN,
        cull_max: MIRROR_MAX,
        use_bg_tev: false,
        casts_shadow: false,
        m9a: true,
    },
    Attr {
        m00: 4,
        m02: 4,
        m04: 20,
        m06: 4,
        m08: 4,
        m0a: 20,
        m0c: 75.0,
        m10: 90.0,
        m14: -5.0,
        m18: 0.005,
        m1c: 0.001,
        m20: 0.0,
        land_smoke_scale: 1.8,
        m28: 4.5,
        m2c: -0.5,
        m30: -0.2,
        m34: 0.02,
        m38: 1000,
        m3c: 0.04,
        m40: 0.013,
        m44: 0.15,
        m48: 0.1,
        m4c: 0.1,
        m50: 0.06,
        m54: 0.075,
        scale_y: 150.0,
        m68: 350.0,
        m6c: 1.0 / 350.0,
        scale_xz: 150.0,
        m74: 1.0 / 150.0,
        move_se: SE_LK_MOVE_ROCK,
        cant_move_se: SE_LK_MOVE_ROCK_LIMIT,
        normal_fall_se: SE_OBJ_BLOCK_FALL_NORMAL,
        water_fall_se: SE_OBJ_FALL_WATER_L,
        magma_fall_se: SE_OBJ_FALL_MAGMA_L,
        cull_min: STD_MIN,
        cull_max: STD_MAX,
        use_bg_tev: false,
        casts_shadow: false,
        m9a: false,
    },
    Attr {
        m00: 4,
        m02: 4,
        m04: 20,
        m06: 4,
        m08: 4,
        m0a: 20,
        m0c: 75.0,
        m10: 90.0,
        m14: -5.0,
        m18: 0.005,
        m1c: 0.001,
        m20: 0.0,
        land_smoke_scale: 1.8,
        m28: 4.5,
        m2c: -0.5,
        m30: -0.2,
        m34: 0.02,
        m38: 1000,
        m3c: 0.04,
        m40: 0.013,
        m44: 0.15,
        m48: 0.1,
        m4c: 0.1,
        m50: 0.06,
        m54: 0.075,
        scale_y: 150.0,
        m68: 150.0,
        m6c: 1.0 / 150.0,
        scale_xz: 150.0,
        m74: 1.0 / 150.0,
        move_se: SE_LK_MOVE_ROCK,
        cant_move_se: SE_LK_MOVE_ROCK_LIMIT,
        normal_fall_se: SE_OBJ_BLOCK_FALL_NORMAL,
        water_fall_se: SE_OBJ_FALL_WATER_L,
        magma_fall_se: SE_OBJ_FALL_MAGMA_L,
        cull_min: STD_MIN,
        cull_max: STD_MAX,
        use_bg_tev: true,
        casts_shadow: true,
        m9a: false,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Wait,
    Walk,
    Afloat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildKind {
    None,
    BuoyFlag,
    Jump,
    Mirror,
    Mkie,
}

fn prm(parameters: u32, width: u32, shift: u32) -> i32 {
    let mask = (1u32 << width) - 1;
    ((parameters >> shift) & mask) as i32
}

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = (seed[0]) as f32 / RND_SCALE[0]
        + (seed[1]) as f32 / RND_SCALE[1]
        + (seed[2]) as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

fn yrot(angle: i16, v: Vec3) -> Vec3 {
    let (s, c) = from_s16(angle).sin_cos();
    Vec3::new(v.x * c + v.z * s, v.y, -v.x * s + v.z * c)
}

fn xrot(angle: i16, v: Vec3) -> Vec3 {
    let (s, c) = from_s16(angle).sin_cos();
    Vec3::new(v.x, v.y * c - v.z * s, v.y * s + v.z * c)
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Movebox {
    pub kind: i32,
    pub parameters: u32,
    pub mode: Mode,
    pub pos: Vec3,
    pub home_pos: Vec3,
    pub angle: [i16; 3],
    pub home_angle: [i16; 3],
    pub shape_angle: [i16; 3],
    pub eye_pos: Vec3,
    pub speed: Vec3,
    pub speed_f: f32,
    pub gravity: f32,
    pub room: i8,
    pub home_room: i8,
    pub prm_z: u16,
    pub prm_x: u16,
    pub prm_z_ready: bool,
    pub prm_x_ready: bool,
    pub should_appear: bool,
    pub sway_clock: i16,
    pub sway_drive_x: f32,
    pub sway_drive_z: f32,
    pub wave_x: f32,
    pub wave_z: f32,
    pub wave_vel_x: f32,
    pub wave_vel_z: f32,
    pub buoy_sink: f32,
    pub submerge: f32,
    pub step_x: i32,
    pub step_z: i32,
    pub ease_rate: f32,
    pub push_dir: i32,
    pub pp_label: u32,
    pub moment_cnt: [i16; SIDE_SLOTS],
    pub ease_count: i16,
    pub float_lock: i16,
    pub land_delay: i16,
    pub pushed: bool,
    pub settle_done: bool,
    pub pull_keep: bool,
    pub reverb: i8,
    pub child: Option<Entity>,
    pub roll_crash: bool,
    pub ground_y: [f32; SLOT_COUNT],
    pub max_ground_idx: i32,
    pub water_y: f32,
    pub wall_pos: [Vec3; SLOT_COUNT],
    pub wall_idx: i32,
    pub nearest_wall_dist: f32,
    pub state_flags: u32,
    pub smoke_pos: [Vec3; SMOKE_CBS],
    pub smoke_live: bool,
    pub placed: bool,
    pub broken: bool,
    pub struck: bool,
    pub path_pos: [Vec3; 4],
    pub path_num: i32,
    pub seed: [i32; 3],
}

impl Movebox {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: prm(parameters, PRM_TYPE_W, PRM_TYPE_S),
            parameters,
            mode: Mode::Wait,
            pos: Vec3::ZERO,
            home_pos: Vec3::ZERO,
            angle: [0; 3],
            home_angle: [0; 3],
            shape_angle: [0; 3],
            eye_pos: Vec3::ZERO,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            gravity: 0.0,
            room: 0,
            home_room: 0,
            prm_z: 0,
            prm_x: 0,
            prm_z_ready: false,
            prm_x_ready: false,
            should_appear: true,
            sway_clock: 0,
            sway_drive_x: 0.0,
            sway_drive_z: 0.0,
            wave_x: 0.0,
            wave_z: 0.0,
            wave_vel_x: 0.0,
            wave_vel_z: 0.0,
            buoy_sink: 0.0,
            submerge: 0.0,
            step_x: 0,
            step_z: 0,
            ease_rate: 0.0,
            push_dir: -1,
            pp_label: 0,
            moment_cnt: [0; SIDE_SLOTS],
            ease_count: 0,
            float_lock: 0,
            land_delay: LAND_DELAY,
            pushed: false,
            settle_done: false,
            pull_keep: false,
            reverb: 0,
            child: None,
            roll_crash: false,
            ground_y: [0.0; SLOT_COUNT],
            max_ground_idx: -1,
            water_y: 0.0,
            wall_pos: [Vec3::ZERO; SLOT_COUNT],
            wall_idx: -1,
            nearest_wall_dist: FLOAT_MAX,
            state_flags: 0,
            smoke_pos: [Vec3::ZERO; SMOKE_CBS],
            smoke_live: false,
            placed: false,
            broken: false,
            struck: false,
            path_pos: [Vec3::ZERO; 4],
            path_num: 0,
            seed: [RND_SEED; 3],
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        let a = self.attr();
        (a.cull_min, a.cull_max)
    }

    pub fn drawn(&self) -> bool {
        !self.broken
    }

    pub fn attr(&self) -> Attr {
        M_ATTR
            .get(self.kind as usize)
            .copied()
            .unwrap_or(M_ATTR[TYPE_BREAKABLE_WOODEN_CRATE as usize])
    }

    pub fn attr_of(kind: i32) -> Attr {
        M_ATTR
            .get(kind as usize)
            .copied()
            .unwrap_or(M_ATTR[TYPE_BREAKABLE_WOODEN_CRATE as usize])
    }

    pub fn height(&self) -> f32 {
        self.attr().m68
    }

    pub fn radius(&self) -> f32 {
        self.attr().m0c
    }

    pub fn set_path(&mut self, points: [Vec3; 4], count: i32) {
        self.path_pos = points;
        self.path_num = count;
    }

    pub fn prm_get_sw_save(&self) -> i32 {
        prm(self.parameters, PRM_SWSAVE_W, PRM_SWSAVE_S)
    }

    fn prm_get_item_no(&self) -> i32 {
        prm(self.parameters, PRM_ITEMNO_W, PRM_ITEMNO_S)
    }

    fn prm_get_item_save(&self) -> i32 {
        prm(self.parameters, PRM_ITEMSAVE_W, PRM_ITEMSAVE_S)
    }

    pub fn prm_get_dmy(&self) -> i32 {
        prm(self.parameters, PRM_DMY_W, PRM_DMY_S)
    }

    pub fn prm_get_buoy(&self) -> i32 {
        prm(self.parameters, PRM_BUOY_W, PRM_BUOY_S)
    }

    fn prm_z_path_id(&self) -> i32 {
        if self.kind == TYPE_BLACK_BOX_WITH_MKIE {
            return NO_PATH_ID;
        }
        i32::from(self.prm_z & 0xFF)
    }

    fn prm_z_sw_save2(&self) -> i32 {
        if self.kind == TYPE_BLACK_BOX_WITH_MKIE {
            return NO_SWITCH;
        }
        i32::from((self.prm_z >> 8) & 0xFF)
    }

    pub fn prm_z_sw_save2_mkie_b(&self) -> i32 {
        if self.kind == TYPE_BLACK_BOX_WITH_MKIE {
            return i32::from((self.prm_z >> 8) & 0xFF);
        }
        NO_SWITCH
    }

    pub fn prm_x_ev_id(&self) -> i32 {
        i32::from(self.prm_x & 0x00FF)
    }

    pub fn prm_z_init(&mut self) {
        if self.prm_z_ready {
            return;
        }
        self.prm_z_ready = true;
        if self.prm_get_dmy() != 0 {
            self.prm_z = 0xFFFF;
        } else {
            self.prm_z = self.home_angle[2] as u16;
        }
        self.home_angle[2] = 0;
        self.angle[2] = 0;
        self.shape_angle[2] = 0;
    }

    pub fn prm_x_init(&mut self) {
        if !self.prm_x_ready {
            self.prm_x = self.home_angle[0] as u16;
            self.prm_x_ready = true;
        }
        self.home_angle[0] = 0;
        self.angle[0] = 0;
        self.shape_angle[0] = 0;
    }

    pub fn chk_appear(&self, switches: &Switches) -> bool {
        let path_id = self.prm_z_path_id();
        let sw = self.prm_get_sw_save();
        if path_id != NO_PATH_ID || sw == NO_SWITCH {
            return true;
        }
        let switch_is_set = switches.test(sw as u8);
        let switch_enables_spawn = self.prm_get_dmy() != 0;
        (!switch_is_set && !switch_enables_spawn) || (switch_is_set && switch_enables_spawn)
    }

    pub fn set_roll_crash(&mut self) {
        if self.kind == TYPE_BREAKABLE_WOODEN_CRATE || self.kind == TYPE_GOLDEN_CRATE {
            self.roll_crash = true;
        }
    }

    pub fn path_init(&mut self, switches: &Switches) {
        let path_id = self.prm_z_path_id();
        let sw_save1 = self.prm_get_sw_save();
        if path_id == NO_PATH_ID || sw_save1 == NO_SWITCH || self.path_num == 0 {
            return;
        }
        let sw_save2 = self.prm_z_sw_save2();
        let is_moved1 = switches.test(sw_save1 as u8);
        let is_moved2 = sw_save2 != NO_SWITCH && switches.test(sw_save2 as u8);
        let mut pnt_idx = 0;
        if is_moved1 {
            pnt_idx += 1;
        }
        if is_moved2 {
            pnt_idx += 2;
        }
        if let Some(pnt) = self.path_pos.get(pnt_idx) {
            self.home_pos = *pnt;
            self.pos = *pnt;
        }
    }

    pub fn path_save(&mut self, switches: &mut Switches) {
        let path_id = self.prm_z_path_id();
        let sw_save1 = self.prm_get_sw_save();
        if path_id == NO_PATH_ID || sw_save1 == NO_SWITCH {
            return;
        }
        let sw_save2 = self.prm_z_sw_save2();
        let mut num_pnts = if sw_save2 == NO_SWITCH { 2 } else { 4 };
        if num_pnts > self.path_num {
            num_pnts = self.path_num;
        }
        let mut pnt_idx = 0usize;
        while (pnt_idx as i32) < num_pnts {
            let pnt_pos = self.path_pos.get(pnt_idx).copied().unwrap_or(Vec3::ZERO);
            if (self.pos - pnt_pos).length_squared() < 3.0 * 3.0 {
                break;
            }
            pnt_idx += 1;
        }
        if (pnt_idx as i32) >= num_pnts {
            return;
        }
        if pnt_idx & 1 != 0 {
            switches.set(sw_save1 as u8);
        } else {
            switches.clear(sw_save1 as u8);
        }
        if sw_save2 != NO_SWITCH {
            if pnt_idx & 2 != 0 {
                switches.set(sw_save2 as u8);
            } else {
                switches.clear(sw_save2 as u8);
            }
        }
    }

    fn gnd_sources(&self) -> (&'static [BgcSrc], usize) {
        if self.attr().m9a {
            (&M_LIN20, M_LIN20.len() - 2)
        } else {
            (&M_LIN5, M_LIN5.len())
        }
    }

    fn wall_sources(&self) -> (&'static [BgcSrc], usize) {
        if self.attr().m9a {
            if self.kind == TYPE_MIRROR {
                (&M_LIN20, M_LIN20.len())
            } else {
                (&M_LIN20, M_LIN20.len() - 2)
            }
        } else {
            (&M_LIN5, M_LIN5.len())
        }
    }

    fn sway_sources(&self) -> (&'static [BgcSrc], usize) {
        if self.attr().m9a {
            (&M_LIN20, M_LIN20.len() - 2)
        } else {
            (&M_LIN5, M_LIN5.len())
        }
    }

    fn gnd_pos(&mut self, solid: Option<&Collision>, platforms: &[Platform]) {
        let a = self.attr();
        let (srcs, count) = self.gnd_sources();
        let mut max_ground_y = G_CM3D_F_INF;
        self.max_ground_idx = -1;
        for index in 0..count {
            let src = srcs.get(index).copied().unwrap_or(M_LIN5[0]);
            let offset = Vec3::new(
                src.m0c + src.m04 * a.scale_xz,
                GROUND_SLOT_Y,
                src.m08 + src.m00 * a.scale_xz,
            );
            let pos = self.pos + yrot(self.home_angle[1], offset);
            let ground = match solid {
                Some(solid) => solid.ground_at(pos.x, pos.z).unwrap_or(G_CM3D_F_INF),
                None => G_CM3D_F_INF,
            };
            if let Some(slot) = self.ground_y.get_mut(index) {
                *slot = ground;
            }
            if ground > max_ground_y {
                if platforms
                    .iter()
                    .any(|entry| entry.is_afloat() && entry.covers(pos.x, pos.z))
                {
                    continue;
                }
                max_ground_y = ground;
                self.max_ground_idx = index as i32;
            }
        }
    }

    pub fn ground_rider(&self, platforms: &[Platform]) -> Option<usize> {
        if self.max_ground_idx < 0 {
            return None;
        }
        let (srcs, _) = self.gnd_sources();
        let a = self.attr();
        let slot = self.max_ground_idx as usize;
        let src = srcs.get(slot).copied().unwrap_or(M_LIN5[0]);
        let offset = Vec3::new(
            src.m0c + src.m04 * a.scale_xz,
            GROUND_SLOT_Y,
            src.m08 + src.m00 * a.scale_xz,
        );
        let probe = self.pos + yrot(self.home_angle[1], offset);
        let ground = self.ground_y.get(slot).copied().unwrap_or(G_CM3D_F_INF);
        platforms.iter().position(|entry| {
            entry.is_afloat() && entry.covers(probe.x, probe.z) && ground <= entry.pos.y
        })
    }

    fn wrt_pos(&mut self, sea: Option<&Sea>) {
        let (in_area, surface) = water::probe(sea, self.pos.x, self.pos.z);
        if in_area {
            self.water_y = surface;
            self.state_flags |= BGC_ABOVE_WATER;
            if self.water_y > self.pos.y {
                self.state_flags |= BGC_IN_WATER;
            }
        }
    }

    fn wall_probe_origin(&self, src: BgcSrc, angle: i16) -> Vec3 {
        let a = self.attr();
        let temp_2c = xrot(WALL_TILT, Vec3::new(src.m0c, 0.0, src.m08));
        let local = Vec3::new(
            src.m04 * a.scale_xz + temp_2c.x,
            (src.m00 + HALF) * a.scale_y + temp_2c.y,
            0.0,
        );
        xrot(WALL_TILT, yrot(angle, local)) + self.pos
    }

    fn wall_touch_origin(&self, src: BgcSrc, angle: i16) -> Vec3 {
        let a = self.attr();
        let temp_20 = xrot(WALL_TILT, xrot(WALL_TILT, Vec3::new(src.m0c, 0.0, src.m08)));
        let local = Vec3::new(
            src.m04 * a.scale_xz + temp_20.x,
            (src.m00 + HALF) * a.scale_y + temp_20.y,
            0.0,
        );
        xrot(WALL_TILT, yrot(angle, local)) + self.pos
    }

    fn wall_probe(
        &mut self,
        solid: Option<&Collision>,
        platforms: &[Platform],
        reach: f32,
        dir_angle: i16,
        count: usize,
    ) {
        let (srcs, _) = self.wall_sources();
        let angle = self.home_angle[1].wrapping_add(dir_angle);
        self.wall_idx = -1;
        self.nearest_wall_dist = FLOAT_MAX;
        let ray = WALL_PROBE_DIR * reach;
        for index in 0..count {
            let src = srcs.get(index).copied().unwrap_or(M_LIN5[0]);
            let start = self.wall_probe_origin(src, angle);
            let end = start + ray;
            let cross = crossed(solid, platforms, start, end);
            if let Some(slot) = self.wall_pos.get_mut(index) {
                *slot = cross.unwrap_or(Vec3::ZERO);
            }
            if let Some(point) = cross {
                let dist_sq = (start - point).length_squared();
                if dist_sq < self.nearest_wall_dist {
                    self.nearest_wall_dist = dist_sq;
                    self.wall_idx = index as i32;
                }
            }
        }
    }

    fn ride_call_back(platform: &mut Platform, a: Attr, delta_x: f32, delta_z: f32, player: bool) {
        let f0 = a.m2c + a.m30;
        let (mut f3, f4) = if player {
            (a.m2c, a.m74 * a.m44)
        } else {
            (a.m30, a.m74 * a.m48)
        };
        let mut f6 = (delta_x * delta_x + delta_z * delta_z).sqrt();
        f6 = 1.0 - f6 * a.m74;
        f6 = f6 * SWAY_TENSION + SWAY_BIAS;
        if f6 < 0.0 {
            f6 = 0.0;
        }
        f3 *= f6;
        platform.sink += f3;
        if platform.sink < f0 {
            platform.sink = f0;
        }
        platform.drive_x += f4 * delta_x;
        platform.drive_z += f4 * delta_z;
    }

    fn proc_vertical(
        &mut self,
        solid: Option<&Collision>,
        sea: Option<&Sea>,
        platforms: &[Platform],
    ) {
        let prev = self.state_flags;
        self.state_flags = 0;
        self.gnd_pos(solid, platforms);
        if self.max_ground_idx >= 0 {
            let ground = self
                .ground_y
                .get(self.max_ground_idx as usize)
                .copied()
                .unwrap_or(G_CM3D_F_INF);
            if ground > self.pos.y {
                self.pos.y = ground;
                self.state_flags |= BGC_ON_GROUND;
                self.speed.y = 0.0;
            }
        }
        self.wrt_pos(sea);
        if self.state_flags & BGC_ABOVE_WATER != 0 && self.water_y > self.pos.y {
            self.state_flags |= BGC_IN_WATER;
        }
        if self.state_flags & BGC_ON_GROUND == 0 && prev & BGC_ON_GROUND != 0 {
            self.state_flags |= BGC_JUST_LEFT_GROUND;
            self.speed.y = self.attr().m20;
        }
        if self.state_flags & BGC_ON_GROUND != 0 && prev & BGC_ON_GROUND == 0 {
            self.state_flags |= BGC_JUST_HIT_GROUND;
        }
        if self.state_flags & BGC_IN_WATER != 0 && prev & BGC_IN_WATER == 0 {
            self.state_flags |= BGC_JUST_HIT_WATER;
        }
    }

    fn chk_wall_pre(
        &mut self,
        solid: Option<&Collision>,
        platforms: &[Platform],
        dir_angle: i16,
    ) -> bool {
        let (_, count) = self.wall_sources();
        self.wall_probe(solid, platforms, WALL_PRE_LEN, dir_angle, count);
        self.wall_idx >= 0
    }

    fn chk_wall_touch(
        &self,
        solid: Option<&Collision>,
        platforms: &[Platform],
        src: BgcSrc,
        dir_angle: i16,
    ) -> bool {
        let a = self.attr();
        let angle = self.home_angle[1].wrapping_add(dir_angle);
        let direction = WALL_PROBE_DIR * (WALL_TOUCH_LEN + a.scale_xz * HALF);
        let start = self.wall_touch_origin(src, angle);
        crossed(solid, platforms, start, start + direction).is_some()
    }

    fn chk_wall_touch2(
        &self,
        solid: Option<&Collision>,
        platforms: &[Platform],
        dir_angle: i16,
    ) -> bool {
        let (srcs, count) = self.sway_sources();
        for index in 0..count {
            let src = srcs.get(index).copied().unwrap_or(M_LIN5[0]);
            if self.chk_wall_touch(solid, platforms, src, dir_angle) {
                return true;
            }
        }
        false
    }

    pub fn pp_call_back(&mut self, angle: i16, label: u32) {
        let pp_label = label & PPLABEL_PUSH_PULL;
        let can_push = if self.attr().m9a {
            label & PPLABEL_HEAVY != 0
        } else {
            true
        };
        if pp_label == 0 || !can_push {
            return;
        }
        let move_ang = if pp_label & PPLABEL_PULL != 0 {
            angle.wrapping_sub(HALF_TURN as i16)
        } else {
            angle
        };
        let angle_diff = move_ang.wrapping_sub(self.home_angle[1]);
        let which_side = if angle_diff >= -0x2000 && angle_diff < 0x2000 {
            0
        } else if angle_diff >= 0x2000 && angle_diff < 0x6000 {
            1
        } else if angle_diff >= 0x6000 || angle_diff < -0x6000 {
            2
        } else {
            3
        };
        for slot in 0..SIDE_SLOTS {
            if let Some(cnt) = self.moment_cnt.get_mut(slot) {
                if slot == which_side {
                    *cnt = cnt.wrapping_add(1);
                } else {
                    *cnt = 0;
                }
            }
        }
        self.pp_label = label;
        self.pushed = true;
    }

    pub fn afl_sway(&mut self, solid: Option<&Collision>, platforms: &[Platform]) {
        let a = self.attr();
        let f31 = self.sway_drive_x * self.sway_drive_x + self.sway_drive_z * self.sway_drive_z;
        let f30 = a.m4c * a.m4c;
        let touched_front_back = self.chk_wall_touch2(solid, platforms, M_DIR_BASE[0])
            || self.chk_wall_touch2(solid, platforms, M_DIR_BASE[2]);
        let touched_left_right = self.chk_wall_touch2(solid, platforms, M_DIR_BASE[1])
            || self.chk_wall_touch2(solid, platforms, M_DIR_BASE[3]);
        if f31 > f30 {
            let temp = a.m4c / f31.sqrt();
            self.sway_drive_x *= temp;
            self.sway_drive_z *= temp;
        }
        let temp2 = self.wave_x - self.sway_drive_x;
        let temp1 = self.wave_z - self.sway_drive_z;
        let f1 = -temp2 * a.m50;
        let f5 = -temp1 * a.m50;
        let f0 = -self.wave_vel_x * a.m54;
        let f6 = -self.wave_vel_z * a.m54;
        self.wave_vel_x += f1 + f0;
        self.wave_vel_z += f5 + f6;
        self.wave_x += self.wave_vel_x;
        self.wave_z += self.wave_vel_z;
        if touched_left_right {
            self.wave_x = 0.0;
        }
        if touched_front_back {
            self.wave_z = 0.0;
        }
        self.sway_drive_x = 0.0;
        self.sway_drive_z = 0.0;
    }

    fn clr_moment_cnt(&mut self) {
        for slot in 0..SIDE_SLOTS {
            if let Some(cnt) = self.moment_cnt.get_mut(slot) {
                *cnt = 0;
            }
        }
    }

    pub fn check_to_walk(&mut self, solid: Option<&Collision>, platforms: &[Platform]) -> i32 {
        let mut r30 = -1;
        let mut r29 = true;
        if self.pushed
            && self.state_flags & BGC_ON_GROUND != 0
            && (self.kind != TYPE_BLACK_BOX_WITH_MKIE || self.child.is_none())
        {
            let pull = self.pp_label & PPLABEL_PULL != 0;
            let unk4 = self.pp_label & PPLABEL_UNK4 != 0;
            let a = self.attr();
            let r0 = if pull {
                if unk4 { a.m06 } else { a.m08 }
            } else if unk4 {
                a.m00
            } else {
                a.m02
            };
            for index in 0..SIDE_SLOTS {
                let cnt = self.moment_cnt.get(index).copied().unwrap_or(0);
                if cnt >= r0 {
                    if !self.chk_wall_pre(solid, platforms, M_DIR_BASE[index]) {
                        r30 = index as i32;
                    }
                } else {
                    r29 = false;
                }
            }
        }
        if r29 {
            self.clr_moment_cnt();
        }
        self.pushed = false;
        r30
    }

    fn eff_set_slip_smoke_pos(&mut self) {
        let scale_mag = self.attr().scale_xz;
        let dir = self.push_dir.max(0) as usize;
        let base_angle = M_DIR_BASE.get(dir).copied().unwrap_or(0);
        let angle = self.home_angle[1].wrapping_add(base_angle);
        let lift = Vec3::new(0.0, 0.0, SMOKE_LIFT);
        for index in 0..SMOKE_CBS {
            let base = SMOKE_BASE_POS.get(index).copied().unwrap_or(Vec3::ZERO);
            let local = self.pos + yrot(angle, base * scale_mag) + yrot(angle, lift * scale_mag);
            if let Some(slot) = self.smoke_pos.get_mut(index) {
                *slot = local;
            }
        }
    }

    fn eff_smoke_slip_start(&mut self) {
        self.eff_set_slip_smoke_pos();
        self.smoke_live = true;
    }

    fn eff_smoke_slip_end(&mut self) {
        self.smoke_live = false;
    }

    pub fn eff_smoke_slip_remove(&mut self) {
        self.smoke_live = false;
    }

    fn mode_wait_init(&mut self) {
        self.speed_f = 0.0;
        self.gravity = self.attr().m14;
        self.clr_moment_cnt();
        self.push_dir = -1;
        self.mode = Mode::Wait;
    }

    fn mode_walk_init(&mut self) {
        self.speed_f = 0.0;
        self.settle_done = false;
        self.mode = Mode::Walk;
    }

    fn mode_afl_init(&mut self) {
        self.speed_f = 0.0;
        self.mode = Mode::Afloat;
    }

    fn pos_move_f_stream(&mut self, damp: f32, drag: f32) {
        let (next, speed_f, facing) = stream::move_stream(
            self.speed,
            Vec3::ZERO,
            self.angle[1],
            self.speed_f,
            self.gravity,
            MAX_FALL,
            damp,
            drag,
        );
        self.speed = next;
        self.speed_f = speed_f;
        self.angle[1] = facing;
        self.pos += next;
    }

    fn side_step(&self, out: &mut Vec3) {
        let a = self.attr();
        let local = Vec3::new(
            (self.step_x as f32) * a.m0c,
            0.0,
            (self.step_z as f32) * a.m0c,
        );
        let point = self.home_pos + yrot(self.home_angle[1], local);
        out.x = point.x;
        out.z = point.z;
    }

    fn walk_anchor(&self, ease: f32, out: &mut Vec3) {
        let a = self.attr();
        let dir = DIR_VEC
            .get(self.push_dir.max(0) as usize)
            .copied()
            .unwrap_or(Vec3::ZERO);
        let local = Vec3::new(
            ((self.step_x as f32) + ease * dir.x) * a.m0c,
            0.0,
            ((self.step_z as f32) + ease * dir.z) * a.m0c,
        );
        let point = self.home_pos + yrot(self.home_angle[1], local);
        out.x = point.x;
        out.z = point.z;
    }

    fn mode_wait(
        &mut self,
        solid: Option<&Collision>,
        platforms: &[Platform],
        sink: &mut Sink<'_>,
    ) {
        if self.float_lock > 0 {
            self.float_lock -= 1;
        }
        let r31 = self.check_to_walk(solid, platforms);
        if !self.settle_done && self.state_flags & BGC_ON_GROUND != 0 {
            self.settle_done = true;
            self.path_save(&mut *sink.switches);
        }
        let a = self.attr();
        self.pos_move_f_stream(a.m18, a.m1c);
        let mut anchor = Vec3::ZERO;
        self.side_step(&mut anchor);
        self.pos.x = anchor.x;
        self.pos.z = anchor.z;
        if r31 != -1 {
            self.push_dir = r31;
            self.eff_smoke_slip_start();
            self.pull_keep = true;
            self.mode_walk_init();
            if self.pp_label & PPLABEL_PULL != 0 {
                self.ease_count = a.m0a;
                self.ease_rate = HALF_TURN / a.m0a as f32;
            } else {
                self.ease_count = a.m04;
                self.ease_rate = HALF_TURN / a.m04 as f32;
            }
        }
    }

    fn mode_walk(
        &mut self,
        solid: Option<&Collision>,
        platforms: &[Platform],
        sink: &mut Sink<'_>,
    ) {
        let a = self.attr();
        self.ease_count = self.ease_count.wrapping_sub(1);
        let r28 = self.ease_count <= 0;
        let ease =
            (stream::cos_s(((self.ease_count as f32) * self.ease_rate) as i32 as i16) + 1.0) * HALF;
        let mut anchor = Vec3::ZERO;
        self.walk_anchor(ease, &mut anchor);
        self.pos.x = anchor.x;
        self.pos.z = anchor.z;
        self.eye_pos = self.pos;
        self.sound_slip(sink);
        if r28 {
            let dir_angle = M_DIR_BASE
                .get(self.push_dir.max(0) as usize)
                .copied()
                .unwrap_or(0);
            if self.chk_wall_pre(solid, platforms, dir_angle) {
                self.sound_limit(sink);
            }
            self.eff_smoke_slip_end();
        } else {
            self.eff_set_slip_smoke_pos();
        }
        self.pos_move_f_stream(a.m18, a.m1c);
        self.pos.x = anchor.x;
        self.pos.z = anchor.z;
        if r28 {
            match self.push_dir {
                0 => self.step_z += 1,
                1 => self.step_x += 1,
                2 => self.step_z -= 1,
                3 => self.step_x -= 1,
                _ => {}
            }
            self.pull_keep = false;
            self.mode_wait_init();
        }
    }

    fn mode_afl(&mut self, solid: Option<&Collision>, platforms: &[Platform]) {
        let a = self.attr();
        let mut f1 = self.pos.y - self.water_y;
        let f0 = if f1 >= 0.0 {
            0.0
        } else if f1 <= -a.m68 {
            1.0
        } else {
            f1 = -f1;
            f1 * a.m6c
        };
        let r3 = ((a.m38 as f32) * (1.0 + rnd(&mut self.seed))) as i32 as i16;
        self.sway_clock = self.sway_clock.wrapping_add(r3);
        self.gravity =
            (f0 * a.m28) + a.m14 + a.m34 * stream::sin_s(self.sway_clock) + self.buoy_sink;
        self.buoy_sink = 0.0;
        self.afl_sway(solid, platforms);
        let temp3 = 1.0 - f0;
        let temp1 = f0 * a.m3c + temp3 * a.m18;
        let temp2 = f0 * a.m40 + temp3 * a.m1c;
        self.submerge = self.water_y - self.pos.y;
        if self.submerge < 0.0 {
            self.submerge = 0.0;
        } else if self.submerge > a.m68 {
            self.submerge = a.m68;
        }
        self.pos_move_f_stream(temp1, temp2);
    }

    fn make_item(&mut self, sink: &mut Sink<'_>) {
        let item_no = self.prm_get_item_no();
        let item_bit_no = self.prm_get_item_save();
        if item_bit_no != NO_ITEM_SAVE && (0..=ITEM_MAX_BIT).contains(&item_bit_no) {
            sink.switches.set(item_bit_no as u8);
        }
        sink.spawns.push(Spawn {
            parameters: item_no as u32,
            pos: self.pos,
            angle: self.angle,
            room: self.home_room,
        });
    }

    fn sound_break(&mut self, sink: &mut Sink<'_>) {
        sink.se(SE_OBJ_COL_SWC_WODCRL);
    }

    fn sound_slip(&mut self, sink: &mut Sink<'_>) {
        let id = self.attr().move_se;
        sink.se(id);
    }

    fn sound_limit(&mut self, sink: &mut Sink<'_>) {
        let id = self.attr().cant_move_se;
        sink.se(id);
    }

    fn sound_land(&mut self, sink: &mut Sink<'_>) {
        let id = self.attr().normal_fall_se;
        sink.se(id);
    }

    pub fn hit(&mut self) {
        self.struck = true;
    }

    pub fn child_kind(&self) -> ChildKind {
        if self.prm_get_buoy() == 0 {
            ChildKind::BuoyFlag
        } else if self.kind == TYPE_METAL_BOX_WITH_SPRING {
            ChildKind::Jump
        } else if self.kind == TYPE_MIRROR {
            ChildKind::Mirror
        } else if self.kind == TYPE_BLACK_BOX_WITH_MKIE {
            ChildKind::Mkie
        } else {
            ChildKind::None
        }
    }

    pub fn child_pos(&self) -> Vec3 {
        match self.child_kind() {
            ChildKind::Mkie => Vec3::new(self.pos.x, self.pos.y + 150.0, self.pos.z),
            ChildKind::BuoyFlag => {
                Vec3::new(self.pos.x, self.pos.y + self.attr().m68 - 5.0, self.pos.z)
            }
            _ => self.pos,
        }
    }

    pub fn execute(
        &mut self,
        solid: Option<&Collision>,
        sea: Option<&Sea>,
        platforms: &[Platform],
        sink: &mut Sink<'_>,
    ) {
        if !self.placed {
            self.placed = true;
            self.prm_x_init();
            self.prm_z_init();
            self.should_appear = self.chk_appear(&*sink.switches);
            if self.should_appear {
                self.path_init(&*sink.switches);
            }
            self.create(solid, sea, platforms);
        }
        if self.struck || self.roll_crash {
            self.make_item(sink);
            self.sound_break(sink);
            self.broken = true;
            return;
        }
        if self.state_flags & BGC_JUST_HIT_WATER != 0 {
            let id = self.attr().water_fall_se;
            sink.se(id);
        }
        let a = self.attr();
        if self.mode == Mode::Wait {
            if self.state_flags & BGC_IN_WATER != 0 && self.float_lock == 0 {
                if self.state_flags & BGC_ON_GROUND != 0 {
                    if a.m28 + a.m14 > 0.0 {
                        self.mode_afl_init();
                    }
                } else {
                    self.mode_afl_init();
                }
            }
        } else if self.mode == Mode::Afloat && self.state_flags & BGC_ON_GROUND != 0 {
            self.mode_wait_init();
            self.float_lock = AFLOAT_LOCK;
        }
        match self.mode {
            Mode::Wait => self.mode_wait(solid, platforms, sink),
            Mode::Walk => self.mode_walk(solid, platforms, sink),
            Mode::Afloat => self.mode_afl(solid, platforms),
        }
        self.proc_vertical(solid, sea, platforms);
        if self.land_delay > 0 {
            self.land_delay -= 1;
        } else if self.state_flags & BGC_JUST_HIT_GROUND != 0 {
            self.sound_land(sink);
        }
    }

    fn create(&mut self, solid: Option<&Collision>, sea: Option<&Sea>, platforms: &[Platform]) {
        let a = self.attr();
        self.sway_clock = 0;
        self.sway_drive_x = 0.0;
        self.sway_drive_z = 0.0;
        self.wave_x = 0.0;
        self.wave_z = 0.0;
        self.wave_vel_x = 0.0;
        self.wave_vel_z = 0.0;
        self.buoy_sink = 0.0;
        self.submerge = a.m68 * HALF;
        self.step_x = 0;
        self.step_z = 0;
        self.ease_rate = 0.0;
        self.pp_label = 0;
        self.clr_moment_cnt();
        self.ease_count = 0;
        self.float_lock = 0;
        self.land_delay = LAND_DELAY;
        self.pushed = false;
        self.pull_keep = false;
        self.speed_f = 0.0;
        self.gravity = a.m14;
        let (speed, _) =
            stream::calc_speed(0.0, self.angle[1], self.speed.y, self.gravity, MAX_FALL);
        self.speed = speed;
        self.pos += self.speed;
        self.proc_vertical(solid, sea, platforms);
        self.state_flags &= !(BGC_JUST_LEFT_GROUND | BGC_JUST_HIT_GROUND | BGC_JUST_HIT_WATER);
        self.settle_done = true;
        self.roll_crash = false;
        self.mode_wait_init();
    }
}

#[derive(Clone, Copy)]
pub struct Platform {
    pub pos: Vec3,
    pub afloat: bool,
    pub sink: f32,
    pub drive_x: f32,
    pub drive_z: f32,
}

impl Platform {
    fn covers(&self, x: f32, z: f32) -> bool {
        (x - self.pos.x).abs() < 150.0 && (z - self.pos.z).abs() < 150.0
    }

    fn is_afloat(&self) -> bool {
        self.afloat
    }
}

fn crossed(
    solid: Option<&Collision>,
    platforms: &[Platform],
    from: Vec3,
    to: Vec3,
) -> Option<Vec3> {
    if platforms
        .iter()
        .any(|entry| entry.is_afloat() && entry.covers(to.x, to.z) && entry.pos.y <= from.y)
    {
        return Some(to);
    }
    let solid = solid?;
    let next = solid.slide(from, to);
    if next.x == to.x && next.z == to.z {
        return None;
    }
    Some(next)
}

pub struct Spawn {
    pub parameters: u32,
    pub pos: Vec3,
    pub angle: [i16; 3],
    pub room: i8,
}

pub struct Sink<'a> {
    pub events: &'a mut SimEvents,
    pub switches: &'a mut Switches,
    pub spawns: &'a mut Vec<Spawn>,
}

impl Sink<'_> {
    fn se(&mut self, id: u32) {
        self.events.list.push(SimEvent::Se(id));
    }
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    sea: Option<Res<'w, Sea>>,
    players: Query<'w, 's, &'static mut Player>,
    /// `cBgS::m_chk_element`, which this actor's own background and its two callbacks are
    /// registered in, `movebox.cpp:1321-1322` and `:1882-1883`. It is absent until this
    /// actor's own first tick has put it there, which is the table `cBgS::Ct` leaves empty,
    /// `c_bg_s.cpp:94-98`.
    pushpull: Option<ResMut<'w, PushPulls>>,
}

#[derive(Clone, Copy)]
struct Slot {
    entity: Entity,
    state: Movebox,
    room: i8,
}

/// `Act_c::Create`, `movebox.cpp:1321-1322`: the background this actor built for itself
/// publishes the ride and the push and pull callbacks, and `Act_c::Delete` at `:1882-1883` takes
/// both back, so every slot goes first and the actors still alive register against it again.
fn register(pushpull: &mut PushPulls, slots: &[Slot]) {
    pushpull.reset();
    for slot in slots {
        if slot.state.broken || !slot.state.should_appear {
            continue;
        }
        pushpull.claim(slot.entity, Some(Face::of(&slot.state)), true, true);
    }
}

/// `dBgS_Acch::GroundCheck`, `d_bg_s_acch.cpp:144`, called on the rider's own ground polygon: the
/// background the rider stands on is the one whose ride callback answers, and a box standing on an
/// afloat box is answered through that box's own ground, `movebox.cpp:236`.
fn ride_player((actor, _): (Entity, Face), slots: &[Slot], platforms: &mut [Platform], feet: Vec3) {
    let Some(slot) = slots
        .iter()
        .find(|slot| slot.entity == actor)
        .filter(|slot| slot.state.state_flags & BGC_ON_GROUND != 0)
    else {
        return;
    };
    let Some(rider) = slot.state.ground_rider(platforms) else {
        return;
    };
    let Some(ground) = platforms.get_mut(rider) else {
        return;
    };
    let before = ground.pos;
    Movebox::ride_call_back(
        ground,
        slot.state.attr(),
        feet.x - before.x,
        feet.z - before.z,
        true,
    );
}

pub fn control(
    mut watch: Watch<'_, '_>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut boxes: Query<(Entity, &mut Actor, &mut Movebox), Without<Player>>,
    mut commands: Commands,
) {
    let solid = watch.collision.as_deref();
    let sea = watch.sea.as_deref();
    let players = watch.players;
    let pending = watch.pushpull.is_none();
    let mut slots: Vec<Slot> = Vec::new();
    for (entity, actor, mut box_) in &mut boxes {
        if !box_.placed {
            box_.pos = actor.pos;
            box_.home_pos = actor.pos;
            box_.angle = actor.angle;
            box_.home_angle = actor.angle;
            box_.shape_angle = actor.angle;
            box_.room = actor.room;
            box_.home_room = actor.room;
            box_.placed = true;
        }
        slots.push(Slot {
            entity,
            state: *box_,
            room: actor.room,
        });
    }
    let player_pos = players.iter().next().map_or(Vec3::ZERO, |entry| entry.pos);
    let mut platforms: Vec<Platform> = slots
        .iter()
        .map(|slot| Platform {
            pos: slot.state.pos,
            afloat: slot.state.mode == Mode::Afloat,
            sink: slot.state.buoy_sink,
            drive_x: slot.state.sway_drive_x,
            drive_z: slot.state.sway_drive_z,
        })
        .collect();
    let mut spawns: Vec<Spawn> = Vec::new();
    let mut broken: Vec<Entity> = Vec::new();
    let mut vanished: Vec<Entity> = Vec::new();
    for index in 0..slots.len() {
        let Some(mut slot) = slots.get(index).copied() else {
            continue;
        };
        let mut sink = Sink {
            events: &mut events,
            switches: &mut switches,
            spawns: &mut spawns,
        };
        slot.state.execute(solid, sea, &platforms, &mut sink);
        if let Some(target) = slots.get_mut(index) {
            target.state = slot.state;
        }
        if let Some(own) = platforms.get_mut(index) {
            own.pos = slot.state.pos;
            own.afloat = slot.state.mode == Mode::Afloat;
        }
        if let Some(rider) = slot.state.ground_rider(&platforms) {
            if let Some(ground) = platforms.get_mut(rider) {
                let before = ground.pos;
                Movebox::ride_call_back(
                    ground,
                    slot.state.attr(),
                    slot.state.pos.x - before.x,
                    slot.state.pos.z - before.z,
                    false,
                );
            }
        }
        if slot.state.broken {
            broken.push(slot.entity);
            continue;
        }
        if !slot.state.should_appear {
            vanished.push(slot.entity);
        }
    }
    for (index, platform) in platforms.iter().enumerate() {
        if let Some(target) = slots.get_mut(index) {
            target.state.buoy_sink = platform.sink;
            target.state.sway_drive_x = platform.drive_x;
            target.state.sway_drive_z = platform.drive_z;
        }
    }
    if !pending && let Some(pushpull) = watch.pushpull.as_deref_mut() {
        register(pushpull, &slots);
    }
    for slot in &slots {
        if let Ok((_, mut actor, mut box_)) = boxes.get_mut(slot.entity) {
            *box_ = slot.state;
            actor.pos = box_.pos;
            actor.angle = box_.angle;
            actor.room = slot.room;
        }
    }
    if let Some(pushpull) = watch.pushpull.as_deref()
        && let Some(rider) = pushpull.standing(player_pos)
    {
        ride_player(rider, &slots, &mut platforms, player_pos);
    }
    if pending {
        commands.insert_resource(PushPulls::new());
    }
    for entity in broken {
        let _ = commands.entity(entity).despawn();
    }
    for entity in vanished {
        let _ = commands.entity(entity).despawn();
    }
    for spawn in spawns {
        let actor = Actor {
            name: super::pickup::ITEM_ACTORS[0].to_owned(),
            parameters: spawn.parameters,
            pos: spawn.pos,
            angle: spawn.angle,
            set_id: 0xFFFF,
            room: spawn.room,
            ..Default::default()
        };
        commands.spawn((actor, pickup::Pickup::spawn(spawn.parameters)));
    }
}
