use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::{Mat4, Vec3, Vec4};

use super::actor::Actor;
use super::arrow;
use super::collision::Collision;
use super::combat::{Combat, CutKind};
use super::damage;
use super::damage_resolve::{
    AT_SE_ARROW, AT_SE_NONE, AT_SPL_UNK0, AT_SPL_UNK1, AT_TYPE_ALL, AT_TYPE_BOOMERANG,
    AT_TYPE_FIRE_ARROW, AT_TYPE_ICE_ARROW, AT_TYPE_LIGHT, AT_TYPE_LIGHT_ARROW,
    AT_TYPE_NORMAL_ARROW, AT_TYPE_SWORD, AT_TYPE_UNK20000, AT_TYPE_UNK400000, AT_TYPE_WATER,
    AT_TYPE_WIND, Atk, AtkType, Attacker, CcAtInfo, HitAudio, HitStop, NAME_ARROW, NAME_PLAYER,
    PlayerCut, RESULT_BOMB, RESULT_SKULL_HAMMER, RESULT_SWORD, at_power_check, def_se_set,
};
use super::esa::Esa;
use super::events::{SimEvent, SimEvents};
use super::health::Health;
use super::item::Held;
use super::player::Player;
use super::puppet_ganon::PuppetGanon;
use super::vibration::Vibration;
use super::walk;

pub const BGN3_ACTORS: [&str; 0] = []; // <- d_a_bgn.cpp:3674

pub fn is_bgn3(name: &str) -> bool {
    BGN3_ACTORS.contains(&name)
}

pub fn is_ks(name: &str) -> bool {
    super::ks::KS_ACTORS.contains(&name)
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Bgn3Hio {
    pub no: i8,
    pub m05: u8,
    pub m06: u8,
    pub m08: f32,
    pub m0c: i16,
    pub m0e: i16,
    pub m10: f32,
    pub m14: i16,
    pub m16: i16,
    pub m18: f32,
    pub m1c: i16,
    pub m1e: i16,
    pub m20: i16,
    pub m22: i16,
    pub m24: i16,
    pub m26: i16,
    pub m28: i16,
    pub m2a: i16,
    pub m2c: i16,
    pub m2e: i16,
    pub m30: i16,
    pub m34: f32,
    pub m38: f32,
    pub m3c: i16,
    pub m3e: u8,
    pub m40: f32,
    pub m44: f32,
    pub m48: f32,
    pub m4c: f32,
    pub m50: u8,
    pub m52: u8,
    pub m54: u8,
}

pub const HIO_NO: i8 = -1; // <- d_a_bgn3.cpp:73
pub const HIO_M05: u8 = 0; // <- d_a_bgn3.cpp:74
pub const HIO_M06: u8 = 0; // <- d_a_bgn3.cpp:75
pub const HIO_M08: f32 = 50.0; // <- d_a_bgn3.cpp:76
pub const HIO_M0C: i16 = 800; // <- d_a_bgn3.cpp:77
pub const HIO_M0E: i16 = 3; // <- d_a_bgn3.cpp:78
pub const HIO_M10: f32 = 60.0; // <- d_a_bgn3.cpp:80
pub const HIO_M14: i16 = 1000; // <- d_a_bgn3.cpp:81
pub const HIO_M16: i16 = 3; // <- d_a_bgn3.cpp:82
pub const HIO_M18: f32 = 80.0; // <- d_a_bgn3.cpp:84
pub const HIO_M1C: i16 = 0x4b0; // <- d_a_bgn3.cpp:85
pub const HIO_M1E: i16 = 3; // <- d_a_bgn3.cpp:86
pub const HIO_M20: i16 = 0x3c; // <- d_a_bgn3.cpp:79
pub const HIO_M22: i16 = 0x28; // <- d_a_bgn3.cpp:83
pub const HIO_M24: i16 = 0x1e; // <- d_a_bgn3.cpp:87
pub const HIO_M26: i16 = 10; // <- d_a_bgn3.cpp:88
pub const HIO_M28: i16 = 0x14; // <- d_a_bgn3.cpp:89
pub const HIO_M2A: i16 = 0x2d; // <- d_a_bgn3.cpp:90
pub const HIO_M2C: i16 = 0x78; // <- d_a_bgn3.cpp:91
pub const HIO_M2E: i16 = 200; // <- d_a_bgn3.cpp:92
pub const HIO_M30: i16 = 0xfa; // <- d_a_bgn3.cpp:93
pub const HIO_M34: f32 = 5.0; // <- d_a_bgn3.cpp:94
pub const HIO_M38: f32 = 0.5; // <- d_a_bgn3.cpp:95
pub const HIO_M3C: i16 = 3; // <- d_a_bgn3.cpp:96
pub const HIO_M3E: u8 = 1; // <- d_a_bgn3.cpp:97
pub const HIO_M40: f32 = 150.0; // <- d_a_bgn3.cpp:98
pub const HIO_M44: f32 = 45.0; // <- d_a_bgn3.cpp:99
pub const HIO_M48: f32 = 8.0; // <- d_a_bgn3.cpp:100
pub const HIO_M4C: f32 = -8.0; // <- d_a_bgn3.cpp:101
pub const HIO_M50: u8 = 0xbc; // <- d_a_bgn3.cpp:102
pub const HIO_M52: u8 = 0x98; // <- d_a_bgn3.cpp:103
pub const HIO_M54: u8 = 0x42; // <- d_a_bgn3.cpp:104

impl Default for Bgn3Hio {
    fn default() -> Self {
        Self {
            no: HIO_NO,
            m05: HIO_M05,
            m06: HIO_M06,
            m08: HIO_M08,
            m0c: HIO_M0C,
            m0e: HIO_M0E,
            m10: HIO_M10,
            m14: HIO_M14,
            m16: HIO_M16,
            m18: HIO_M18,
            m1c: HIO_M1C,
            m1e: HIO_M1E,
            m20: HIO_M20,
            m22: HIO_M22,
            m24: HIO_M24,
            m26: HIO_M26,
            m28: HIO_M28,
            m2a: HIO_M2A,
            m2c: HIO_M2C,
            m2e: HIO_M2E,
            m30: HIO_M30,
            m34: HIO_M34,
            m38: HIO_M38,
            m3c: HIO_M3C,
            m3e: HIO_M3E,
            m40: HIO_M40,
            m44: HIO_M44,
            m48: HIO_M48,
            m4c: HIO_M4C,
            m50: HIO_M50,
            m52: HIO_M52,
            m54: HIO_M54,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Bgn3Static {
    pub hio: Bgn3Hio,
    pub hio_set: bool,
    pub ke_color: [u8; 4],
}

impl Default for Bgn3Static {
    fn default() -> Self {
        let hio = Bgn3Hio::default();
        Self {
            hio,
            hio_set: false,
            ke_color: [hio.m50, hio.m52, hio.m54, 0xFF],
        }
    }
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Bgn3Siblings {
    pub bgn_found: bool,
    pub bgn_phase: i8,
    pub bgn_pos: Vec3,
    pub esa_found: bool,
    pub esa_pos: Vec3,
    pub ki_count: i32,
}

pub const KE_MAX: [u16; 7] = [0x0028, 0x0023, 0x001E, 0x0019, 0x0014, 0x000F, 0x000F]; // <- d_a_bgn3.cpp:67

pub const SIZE_D: [f32; 9] = [1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.6]; // <- d_a_bgn3.cpp:69

pub const KE_STEPS: usize = 5; // <- d_a_bgn3.h:10
pub const KE_HAIRS: usize = 40; // <- d_a_bgn3.h:49
pub const PART_KE_MAX: usize = 40; // <- d_a_bgn3.h:38

pub const PART_CHAIN: usize = 10; // <- d_a_bgn3.cpp:253
pub const PARTS: usize = 9; // <- d_a_bgn3.cpp:931
pub const PART_CORE: usize = 8; // <- d_a_bgn3.cpp:939
pub const PART_KE: usize = 7; // <- d_a_bgn3.cpp:963
pub const PART_TG: usize = 8; // <- d_a_bgn3.cpp:703

pub const COOL_SLOTS: usize = 5; // <- d_a_bgn3.cpp:1099

pub const GRAVITY: f32 = -5.0; // <- d_a_bgn3.cpp:333
pub const HIDDEN_Y: f32 = 30000.0; // <- d_a_bgn3.cpp:1083
pub const GROUND_Y: f32 = 150.0; // <- d_a_bgn3.cpp:311
pub const WALL_REACH: f32 = 150.0; // <- d_a_bgn3.cpp:293
pub const WALL_RISE: f32 = 100.0; // <- d_a_bgn3.cpp:292
pub const NECK_BACK: f32 = -300.0; // <- d_a_bgn3.cpp:832
pub const NECK_FWD: f32 = 250.0; // <- d_a_bgn3.cpp:902
pub const SEG_REACH: f32 = 70.0; // <- d_a_bgn3.cpp:254
pub const SEG_SINK: f32 = -5.0; // <- d_a_bgn3.cpp:245
pub const SEG_STEP: f32 = 150.0; // <- d_a_bgn3.cpp:244
pub const SEG_SCALE: f32 = 1.7; // <- d_a_bgn3.cpp:836
pub const SEG_RADIUS: f32 = 100.0; // <- d_a_bgn3.cpp:955
pub const SEG_KE_Y: f32 = 60.0; // <- d_a_bgn3.cpp:969
pub const KE_ROOT_Y: f32 = 100.0; // <- d_a_bgn3.cpp:894
pub const LAG_STEP: f32 = 0.002; // <- d_a_bgn3.cpp:274
pub const TAIL_STEP: f32 = 0.5; // <- d_a_bgn3.cpp:275
pub const TURN_SCALE: i16 = 4; // <- d_a_bgn3.cpp:368
pub const SHAPE_Y_SCALE: i16 = 4; // <- d_a_bgn3.cpp:828
pub const SHAPE_Y_STEP: i16 = 0x1000; // <- d_a_bgn3.cpp:828
pub const SHAPE_X_SCALE: i16 = 8; // <- d_a_bgn3.cpp:829
pub const SHAPE_X_STEP: i16 = 0x400; // <- d_a_bgn3.cpp:829
pub const PITCH_CAP: i16 = 0x3000; // <- d_a_bgn3.cpp:491
pub const HALF_TURN: i16 = -0x8000; // <- d_a_bgn3.cpp:385
pub const LAG_RESET: f32 = 0.9; // <- d_a_bgn3.cpp:386
pub const LAG_RESET_FULL: f32 = 1.0; // <- d_a_bgn3.cpp:468
pub const CRAWL_GATE: f32 = 10.0; // <- d_a_bgn3.cpp:808
pub const SE_ROTATE_SCALE: f32 = 3.5; // <- d_a_bgn3.cpp:772
pub const SE_ROPE_SCALE: f32 = 2.0; // <- d_a_bgn3.cpp:780
pub const SE_PITCH_MAX: u32 = 100; // <- d_a_bgn3.cpp:774
pub const FLASH_AMPLITUDE: f32 = 500.0; // <- d_a_bgn3.cpp:906
pub const FLASH_PHASE_Y: i16 = 0x2100; // <- d_a_bgn3.cpp:907
pub const FLASH_PHASE_X: i16 = 0x2300; // <- d_a_bgn3.cpp:908
pub const SWAY_PHASE: i16 = 0x900; // <- d_a_bgn3.cpp:364
pub const SWAY_REACH: f32 = 4000.0; // <- d_a_bgn3.cpp:366
pub const TURN_ROLL_RANGE: f32 = 50.0; // <- d_a_bgn3.cpp:412
pub const TURN_ROLL_LOW: f32 = 30.0; // <- d_a_bgn3.cpp:412
pub const TURN_FLIP_RANGE: f32 = 1.0; // <- d_a_bgn3.cpp:423
pub const TURN_FLIP_CHANCE: f32 = 0.5; // <- d_a_bgn3.cpp:423
pub const WALL_COOLDOWN: i16 = 0x14; // <- d_a_bgn3.cpp:388
pub const HIT_COOLDOWN: i16 = 0x14; // <- d_a_bgn3.cpp:635
pub const ARMOUR_COOLDOWN: i16 = 10; // <- d_a_bgn3.cpp:710
pub const BROKEN_COOLDOWN: i16 = 10000; // <- d_a_bgn3.cpp:676
pub const SHOCK_PATTERN: i32 = 5; // <- d_a_bgn3.cpp:389
pub const SHOCK_FLAGS: i32 = -0x21; // <- d_a_bgn3.cpp:389
pub const SHOCK_DIR: Vec3 = Vec3::new(0.0, 1.0, 0.0); // <- d_a_bgn3.cpp:389
pub const SHOCK_FACE_PATTERN: i32 = 3; // <- d_a_bgn3.cpp:600

pub const FLASH_FRAME_HAMMER: i16 = 0x1e; // <- d_a_bgn3.cpp:602
pub const FLASH_FRAME_BOMB: i16 = 0x19; // <- d_a_bgn3.cpp:606
pub const FLASH_FRAME_CRITICAL: i16 = 0x14; // <- d_a_bgn3.cpp:612
pub const FLASH_FRAME_SWORD: i16 = 0xf; // <- d_a_bgn3.cpp:617
pub const FLASH_FRAME_ARROW: i16 = 10; // <- d_a_bgn3.cpp:622
pub const FLASH_FRAME_OTHER: i16 = 7; // <- d_a_bgn3.cpp:626

pub const PAUSE_HAMMER: i8 = 5; // <- d_a_bgn3.cpp:601
pub const PAUSE_CRITICAL: i8 = 3; // <- d_a_bgn3.cpp:611
pub const PAUSE_SWORD: i8 = 2; // <- d_a_bgn3.cpp:616

pub const SE_SELECT_HEAD: u32 = 0x40; // <- d_a_bgn3.cpp:645
pub const SE_SELECT_BODY: u32 = 0x44; // <- d_a_bgn3.cpp:711

pub const LAUNCH_Y: f32 = 150.0; // <- d_a_bgn3.cpp:465
pub const DAMAGE_SPEED_F: f32 = 30.0; // <- d_a_bgn3.cpp:466
pub const DAMAGE_HOVER: f32 = 350.0; // <- d_a_bgn3.cpp:471
pub const DAMAGE_HOVER_STEP: f32 = 50.0; // <- d_a_bgn3.cpp:471
pub const BOUNCE_Y: f32 = 50.0; // <- d_a_bgn3.cpp:473
pub const END_HOVER_OFFSET: f32 = 150.0; // <- d_a_bgn3.cpp:524
pub const END_HOVER_MAX: f32 = 350.0; // <- d_a_bgn3.cpp:525
pub const END_BOUNCE_Y: f32 = 130.0; // <- d_a_bgn3.cpp:534
pub const END_TRIGGER_FALL: f32 = -30.0; // <- d_a_bgn3.cpp:539
pub const END_GRAVITY: f32 = 5.5; // <- d_a_bgn3.cpp:545
pub const END_TURN_TARGET: i16 = 5000; // <- d_a_bgn3.cpp:553
pub const END_TURN_SCALE: i16 = 4; // <- d_a_bgn3.cpp:553
pub const END_TURN_RATE: i16 = 100; // <- d_a_bgn3.cpp:553
pub const END_TAIL: f32 = 50.0; // <- d_a_bgn3.cpp:554
pub const END_HOVER_STEP: f32 = 50.0; // <- d_a_bgn3.cpp:555
pub const END_SLIDE_X: f32 = -15.0; // <- d_a_bgn3.cpp:573
pub const END_SLIDE_Z: f32 = 375.17; // <- d_a_bgn3.cpp:574
pub const END_SLIDE_SCALE: f32 = 0.05; // <- d_a_bgn3.cpp:573
pub const END_SLIDE_STEP: f32 = 30.0; // <- d_a_bgn3.cpp:573

pub const HEALTH_MAX: i16 = 3; // <- d_a_bgn3.cpp:1318

pub const ACT_CRAWL: i16 = 0; // <- d_a_bgn3.cpp:806
pub const ACT_STUN: i16 = 1; // <- d_a_bgn3.cpp:815
pub const ACT_DAMAGE: i16 = 2; // <- d_a_bgn3.cpp:818
pub const ACT_END: i16 = 3; // <- d_a_bgn3.cpp:821
pub const ACT_HIDDEN: i16 = 10; // <- d_a_bgn3.cpp:804

pub const ENTER_TURN_TIMER: i16 = 180; // <- d_a_bgn3.cpp:1092
pub const ENTER_AIM_TIMER: i16 = 180; // <- d_a_bgn3.cpp:1093
pub const ESA_TURN_TIMER: i16 = 2; // <- d_a_bgn3.cpp:349
pub const PHASE_ACTIVE: i8 = 2; // <- d_a_bgn3.cpp:1080
pub const KI_CAP: i32 = 0x14; // <- d_a_bgn3.cpp:1121
pub const KI_PARAM: u32 = 3; // <- d_a_bgn3.cpp:1125
pub const KI_SPAWN_RANGE: f32 = 500.0; // <- d_a_bgn3.cpp:1123
pub const KI_SPAWN_LOW: f32 = 3500.0; // <- d_a_bgn3.cpp:1123
pub const KI_SPAWN_SPREAD: f32 = 2500.0; // <- d_a_bgn3.cpp:1122

pub const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA]; // <- c_math.cpp:179
pub const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673]; // <- c_math.cpp:179
pub const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0]; // <- c_math.cpp:183
pub const RND_SEED: i32 = 100; // <- m_Do_machine.cpp:617

pub const OBJ_ATP: u8 = 2; // <- d_a_bgn3.cpp:1237
pub const CORE_TG_TYPE: AtkType = AT_TYPE_LIGHT_ARROW; // <- d_a_bgn3.cpp:1267
pub const OBJ_TG_TYPE: AtkType = AtkType(
    AT_TYPE_ALL.0
        & !AT_TYPE_BOOMERANG.0
        & !AT_TYPE_WATER.0
        & !AT_TYPE_UNK20000.0
        & !AT_TYPE_WIND.0
        & !AT_TYPE_UNK400000.0
        & !AT_TYPE_LIGHT.0,
); // <- d_a_bgn3.cpp:1239
pub const RADIUS_BODY: f32 = 200.0; // <- d_a_bgn3.cpp:853
pub const RADIUS_HEAD: f32 = 150.0; // <- d_a_bgn3.cpp:916
pub const MODEL_SCALE: f32 = 2.0; // <- d_a_bgn3.cpp:848

pub const SE_LK_ARROW_HIT: u32 = 0x2879; // <- d_a_bgn3.cpp:659
pub const SE_CV_BK_SURPRISE: u32 = 0x4830; // <- d_a_bgn3.cpp:518
pub const SE_CV_BGN_HIT_1: u32 = 0x496E; // <- d_a_bgn3.cpp:537
pub const SE_CV_BGN_HIT_LAST: u32 = 0x4970; // <- d_a_bgn3.cpp:681
pub const SE_CV_BGN_M_HIT_FACE: u32 = 0x4973; // <- d_a_bgn3.cpp:596
pub const SE_CM_BGN_M_HIT_WALL: u32 = 0x5981; // <- d_a_bgn3.cpp:390
pub const SE_CM_BGN_M_FACE_HIT: u32 = 0x5982; // <- d_a_bgn3.cpp:597
pub const SE_CM_BGN_M_BRK_ORB: u32 = 0x5983; // <- d_a_bgn3.cpp:682
pub const SE_CM_BGN_MECHA_ROTATE: u32 = 0x705A; // <- d_a_bgn3.cpp:776
pub const SE_CM_BGN_MECHA_ROPE: u32 = 0x705B; // <- d_a_bgn3.cpp:784
pub const SE_CM_BGN_M_CRAWL: u32 = 0x705D; // <- d_a_bgn3.cpp:811

pub const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:557
pub const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:557

pub const KE_LEAN_STEP_X: i32 = 1000; // <- d_a_bgn3.cpp:892
pub const KE_LEAN_BASE_X: i32 = 0x2000; // <- d_a_bgn3.cpp:892
pub const KE_LEAN_MASK_X: i32 = 0x3FFF; // <- d_a_bgn3.cpp:892
pub const KE_LEAN_STEP_Z: i32 = 5000; // <- d_a_bgn3.cpp:893
pub const KE_LEAN_BASE_Z: i32 = 0x800; // <- d_a_bgn3.cpp:893
pub const KE_LEAN_MASK_Z: i32 = 0xFFF; // <- d_a_bgn3.cpp:893

fn seg_lean_x(index: i32) -> i16 {
    ((KE_LEAN_MASK_X & index.wrapping_mul(KE_LEAN_STEP_X)) - KE_LEAN_BASE_X) as i16
}

fn seg_lean_z(index: i32) -> i16 {
    ((KE_LEAN_MASK_Z & index.wrapping_mul(KE_LEAN_STEP_Z)) - KE_LEAN_BASE_Z) as i16
}

fn ke_lean_x(index: i32) -> i16 {
    seg_lean_x(index).wrapping_neg()
}

fn ke_lean_z(index: i32) -> i16 {
    seg_lean_z(index).wrapping_neg()
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ke {
    pub head: Vec3,
    pub pos: [Vec3; KE_STEPS],
}

impl Ke {
    pub fn new() -> Self {
        Self {
            head: Vec3::ZERO,
            pos: [Vec3::ZERO; KE_STEPS],
        }
    }
}

impl Default for Ke {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartSph {
    pub centre: Vec3,
    pub radius: f32,
    pub at_set: bool,
}

impl PartSph {
    pub fn new() -> Self {
        Self {
            centre: Vec3::ZERO,
            radius: 0.0,
            at_set: false,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Part {
    pub pos: Vec3,
    pub vel: Vec3,
    pub angle_y: i16,
    pub angle_x: i16,
    pub scale: f32,
    pub sph: PartSph,
    pub hair: [Ke; PART_KE_MAX],
}

impl Part {
    pub fn new() -> Self {
        Self {
            pos: Vec3::ZERO,
            vel: Vec3::ZERO,
            angle_y: 0,
            angle_x: 0,
            scale: 0.0,
            sph: PartSph::new(),
            hair: [Ke::new(); PART_KE_MAX],
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bgn3Sph {
    pub centre: Vec3,
    pub radius: f32,
    pub at_set: bool,
}

impl Bgn3Sph {
    pub fn new() -> Self {
        Self {
            centre: Vec3::ZERO,
            radius: 0.0,
            at_set: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct At {
    pub obj: Atk,
    pub pos: Vec3,
    pub radius: f32,
}

impl At {
    pub fn hits(&self, centre: Vec3, radius: f32) -> bool {
        walk::within(self.pos, centre, self.radius + radius)
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bgn3 {
    pub health: i16,
    pub frame: i16,
    pub act: i16,
    pub sub: i16,
    pub turn: i16,
    pub wall_count: i16,
    pub lag: f32,
    pub tail: f32,
    pub hover: f32,
    pub neck: Vec3,
    pub flash: i16,
    pub ks_timer: i32,
    pub cool_hit: i16,
    pub cool_armour: i16,
    pub cool: [i16; COOL_SLOTS],
    pub parts: [Part; PART_CHAIN],
    pub hairs: [Ke; KE_HAIRS],
    pub head: Bgn3Sph,
    pub body: Bgn3Sph,
    pub seed: [i32; 3],
    pub home: Vec3,
    pub home_set: bool,
}

impl Bgn3 {
    pub fn spawn(parameters: u32) -> Self {
        let _ = parameters;
        Self {
            health: HEALTH_MAX,
            frame: 0,
            act: ACT_HIDDEN,
            sub: 0,
            turn: 0,
            wall_count: 0,
            lag: 0.0,
            tail: 0.0,
            hover: 0.0,
            neck: Vec3::ZERO,
            flash: 0,
            ks_timer: 0,
            cool_hit: 0,
            cool_armour: 0,
            cool: [0; COOL_SLOTS],
            parts: [Part::new(); PART_CHAIN],
            hairs: [Ke::new(); KE_HAIRS],
            head: Bgn3Sph::new(),
            body: Bgn3Sph::new(),
            seed: [RND_SEED; 3],
            home: Vec3::ZERO,
            home_set: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.act != ACT_HIDDEN
    }

    pub fn rnd(&mut self) -> f32 {
        for lane in 0..3 {
            self.seed[lane] = (self.seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
        }
        let sum = (self.seed[0]) as f32 / RND_SCALE[0]
            + (self.seed[1]) as f32 / RND_SCALE[1]
            + (self.seed[2]) as f32 / RND_SCALE[2];
        (sum % 1.0).abs()
    }

    pub fn rnd_f(&mut self, range: f32) -> f32 {
        self.rnd() * range
    }

    pub fn rnd_fx(&mut self, range: f32) -> f32 {
        range * (self.rnd() - 0.5) * 2.0
    }

    pub fn touches(&self, at: &At) -> bool {
        if self.body.at_set && at.hits(self.body.centre, self.body.radius) {
            return true;
        }
        for part in self.parts.iter().take(PART_TG) {
            if part.sph.at_set && at.hits(part.sph.centre, part.sph.radius) {
                return true;
            }
        }
        false
    }
}

fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value != target {
        let mut step = scale * (target - *value);
        if step > max_step {
            step = max_step;
        } else if step < -max_step {
            step = -max_step;
        }
        *value += step;
    }
}

fn add_calc0(value: &mut f32, scale: f32, max_step: f32) {
    let mut step = *value * scale;
    if step > max_step {
        step = max_step;
    } else if step < -max_step {
        step = -max_step;
    }
    *value -= step;
}

fn ssin(angle: i16) -> f32 {
    walk::from_s16(angle).sin()
}

fn scos(angle: i16) -> f32 {
    walk::from_s16(angle).cos()
}

fn xrot(angle: i16) -> Mat4 {
    let a = walk::from_s16(angle);
    Mat4::from_cols(
        Vec4::X,
        Vec4::new(0.0, a.cos(), a.sin(), 0.0),
        Vec4::new(0.0, -a.sin(), a.cos(), 0.0),
        Vec4::W,
    )
}

fn yrot(angle: i16) -> Mat4 {
    let a = walk::from_s16(angle);
    Mat4::from_cols(
        Vec4::new(a.cos(), 0.0, -a.sin(), 0.0),
        Vec4::Y,
        Vec4::new(a.sin(), 0.0, a.cos(), 0.0),
        Vec4::W,
    )
}

fn zrot(angle: i16) -> Mat4 {
    let a = walk::from_s16(angle);
    Mat4::from_cols(
        Vec4::new(a.cos(), a.sin(), 0.0, 0.0),
        Vec4::new(-a.sin(), a.cos(), 0.0, 0.0),
        Vec4::Z,
        Vec4::W,
    )
}

fn point(m: &Mat4, v: Vec3) -> Vec3 {
    (m * Vec4::new(v.x, v.y, v.z, 1.0)).truncate()
}

impl Bgn3 {
    pub fn check_ground(&self, actor: &mut Actor) -> bool {
        if actor.pos.y <= GROUND_Y {
            actor.pos.y = GROUND_Y;
            actor.speed.y = 0.0;
            true
        } else {
            false
        }
    }

    pub fn check_wall(&self, actor: &mut Actor, solid: Option<&Collision>) -> bool {
        let delta = actor.pos - actor.old.pos;
        let flat = Vec3::new(delta.x, 0.0, delta.z);
        if flat.length() <= 0.0 {
            return false;
        }
        let Some(solid) = solid else {
            return false;
        };
        let m = yrot(walk::to_s16(flat.x.atan2(flat.z)));
        let ahead = point(&m, Vec3::new(0.0, WALL_RISE, WALL_REACH)) + actor.pos;
        let from = Vec3::new(actor.pos.x, actor.pos.y + WALL_RISE, actor.pos.z);
        if solid.slide(from, ahead) != ahead {
            actor.pos.x = actor.old.pos.x;
            actor.pos.z = actor.old.pos.z;
            return true;
        }
        false
    }

    pub fn pos_move(&self, actor: &mut Actor) {
        let m = yrot(actor.angle[1]);
        let v = point(&m, Vec3::new(0.0, 0.0, actor.speed_f));
        actor.speed.x = v.x;
        actor.speed.z = v.z;
        actor.pos += actor.speed;
        actor.speed.y += GRAVITY;
    }

    pub fn pitch_from_move(&self, actor: &mut Actor) {
        let delta = actor.pos - actor.old.pos;
        let flat = (delta.x * delta.x + delta.z * delta.z).sqrt();
        let pitch = -walk::to_s16(delta.y.atan2(flat));
        actor.angle[0] = pitch.clamp(-PITCH_CAP, PITCH_CAP);
    }

    pub fn part_control(&mut self, actor: &Actor, hio: &Bgn3Hio) {
        let lag = self.lag;
        let base = yrot(actor.shape_angle[1]) * xrot(actor.shape_angle[0]);
        let root = point(&base, Vec3::new(0.0, 0.0, -self.tail));
        self.parts[0].pos = self.neck;
        for i in 1..PART_CHAIN {
            let prev = self.parts[i - 1];
            let next = self.parts[i];
            let reach = SEG_REACH * prev.scale;
            let mut high = next.vel.y + (next.pos.y + SEG_SINK);
            if high <= reach {
                high = reach;
            }
            let flat_x = root.x + (next.vel.x + (next.pos.x - prev.pos.x));
            let rise = high - prev.pos.y;
            let flat_z = root.z + (next.vel.z + (next.pos.z - prev.pos.z));
            let span = (flat_x * flat_x + flat_z * flat_z).sqrt();
            self.parts[i - 1].angle_y = walk::to_s16(flat_x.atan2(flat_z));
            self.parts[i - 1].angle_x = -walk::to_s16(rise.atan2(span));
            let m = yrot(self.parts[i - 1].angle_y) * xrot(self.parts[i - 1].angle_x);
            let step = point(&m, Vec3::new(0.0, 0.0, SEG_STEP * prev.scale));
            self.parts[i].pos = prev.pos + step;
            self.parts[i].vel = lag * (self.parts[i].pos - next.pos);
        }
        add_calc2(&mut self.lag, hio.m38, 1.0, LAG_STEP);
        add_calc2(&mut self.tail, hio.m34, 1.0, TAIL_STEP);
    }

    pub fn move0(
        &mut self,
        actor: &mut Actor,
        hio: &Bgn3Hio,
        esa: Option<Vec3>,
        player: Vec3,
        solid: Option<&Collision>,
        shake: &mut Vibration,
        events: &mut SimEvents,
    ) {
        if esa.is_some() {
            self.cool[0] = ESA_TURN_TIMER;
        }
        if self.cool[0] != 0 {
            self.cool[0] -= 1;
            let mut max_step = if self.turn < 0 {
                self.turn.wrapping_neg()
            } else {
                self.turn
            };
            let aim = if self.cool[3] != 0 {
                walk::to_s16((-actor.pos.x).atan2(-actor.pos.z))
            } else if let Some(esa) = esa {
                let d = esa - actor.pos;
                max_step = max_step.wrapping_mul(2);
                walk::to_s16(d.x.atan2(d.z))
            } else {
                let sway = ssin(self.frame.wrapping_mul(SWAY_PHASE));
                let face = walk::to_s16((player.x - actor.pos.x).atan2(player.z - actor.pos.z));
                face.wrapping_add((SWAY_REACH * sway) as i32 as i16)
            };
            super::stream::add_calc_angle_s2(&mut actor.angle[1], aim, TURN_SCALE, max_step);
        } else {
            actor.angle[1] = actor.angle[1].wrapping_add(self.turn);
        }
        let want = if self.health == 3 {
            hio.m08
        } else if self.health == 2 {
            hio.m10
        } else {
            hio.m18
        };
        add_calc2(&mut actor.speed_f, want, 1.0, 3.0);
        self.pos_move(actor);
        self.check_ground(actor);
        let mut reroll = false;
        if self.check_wall(actor, solid) && self.cool[1] == 0 {
            self.cool[0] = 0;
            actor.angle[1] = actor.angle[1].wrapping_add(HALF_TURN);
            self.lag = LAG_RESET;
            reroll = true;
            self.cool[1] = WALL_COOLDOWN;
            events.list.push(SimEvent::Se(SE_CM_BGN_M_HIT_WALL));
            shake.start_shock(SHOCK_PATTERN, SHOCK_FLAGS, SHOCK_DIR);
            self.ks_timer = i32::from(hio.m3c);
            let cap = if self.health == 3 {
                hio.m0e
            } else if self.health == 2 {
                hio.m16
            } else {
                hio.m1e
            };
            self.wall_count += 1;
            if self.wall_count >= cap {
                self.wall_count = 0;
                self.cool[0] = if self.health == 3 {
                    hio.m20
                } else if self.health == 2 {
                    hio.m22
                } else {
                    hio.m24
                };
            }
        }
        if self.cool[2] == 0 {
            let roll = self.rnd_f(TURN_ROLL_RANGE) + TURN_ROLL_LOW;
            self.cool[2] = roll as i16;
            reroll = true;
        }
        if reroll {
            self.turn = if self.health == 3 {
                hio.m0c
            } else if self.health == 2 {
                hio.m14
            } else {
                hio.m1c
            };
            if self.rnd_f(TURN_FLIP_RANGE) < TURN_FLIP_CHANCE {
                self.turn = self.turn.wrapping_neg();
            }
        }
        self.body.at_set = true;
        self.head.at_set = true;
        actor.angle[0] = 0;
    }

    pub fn mahi(&mut self, actor: &mut Actor, solid: Option<&Collision>) {
        match self.sub {
            0 => {
                self.sub = 1;
                self.mahi_1();
            }
            1 => self.mahi_1(),
            _ => {}
        }
        add_calc0(&mut actor.speed_f, 1.0, 3.0);
        self.pos_move(actor);
        actor.angle[0] = 0;
        self.check_ground(actor);
        if self.check_wall(actor, solid) && self.cool[1] == 0 {
            self.cool[0] = 0;
            actor.angle[1] = actor.angle[1].wrapping_add(HALF_TURN);
            self.lag = LAG_RESET;
        }
    }

    fn mahi_1(&mut self) {
        if self.cool[0] == 0 {
            self.act = ACT_CRAWL;
            self.sub = 0;
        }
    }

    pub fn damage(&mut self, actor: &mut Actor, solid: Option<&Collision>, shake: &mut Vibration) {
        match self.sub {
            0 => {
                actor.speed.y = LAUNCH_Y;
                actor.speed_f = DAMAGE_SPEED_F;
                self.sub = 1;
                self.lag = LAG_RESET_FULL;
            }
            1 => {
                add_calc2(&mut self.hover, DAMAGE_HOVER, 1.0, DAMAGE_HOVER_STEP);
                if self.check_ground(actor) {
                    actor.speed.y = BOUNCE_Y;
                    self.sub = 2;
                    actor.angle[1] = (self.rnd_f(65536.0) as i32) as i16;
                    self.lag = LAG_RESET_FULL;
                    shake.start_shock(SHOCK_PATTERN, SHOCK_FLAGS, SHOCK_DIR);
                }
            }
            2 => {
                if self.check_ground(actor) {
                    self.act = ACT_CRAWL;
                    self.sub = 0;
                }
            }
            _ => {}
        }
        self.pos_move(actor);
        self.pitch_from_move(actor);
        self.check_ground(actor);
        if self.check_wall(actor, solid) && self.cool[1] == 0 {
            actor.angle[1] = actor.angle[1].wrapping_add(HALF_TURN);
            self.cool[1] = WALL_COOLDOWN;
        }
        self.body.at_set = false;
        self.head.at_set = false;
    }

    pub fn end(&mut self, actor: &mut Actor, shake: &mut Vibration, events: &mut SimEvents) {
        match self.sub {
            0 => {
                actor.speed.y = LAUNCH_Y;
                actor.speed_f = 0.0;
                self.sub = 1;
                self.lag = LAG_RESET_FULL;
                events.list.push(SimEvent::Se(SE_CV_BK_SURPRISE));
            }
            1..=4 => {
                self.hover = actor.pos.y - END_HOVER_OFFSET;
                if self.hover > END_HOVER_MAX {
                    self.hover = END_HOVER_MAX;
                }
                if self.check_ground(actor) {
                    self.sub += 1;
                    actor.angle[1] = (self.rnd_f(65536.0) as i32) as i16;
                    self.lag = LAG_RESET_FULL;
                    shake.start_shock(SHOCK_PATTERN, SHOCK_FLAGS, SHOCK_DIR);
                    actor.speed.y = END_BOUNCE_Y;
                    actor.speed_f = 0.0;
                    actor.angle[1] = (self.rnd_f(65536.0) as i32) as i16;
                    events.list.push(SimEvent::Se(SE_CV_BGN_HIT_1));
                }
                if self.sub == 4 && actor.speed.y <= END_TRIGGER_FALL {
                    self.sub = 5;
                    self.turn = 0;
                }
            }
            5 => {
                actor.speed.y += END_GRAVITY;
                actor.angle[1] = actor.angle[1].wrapping_add(self.turn);
                actor.shape_angle[1] = actor.shape_angle[1].wrapping_add(self.turn);
                super::stream::add_calc_angle_s2(
                    &mut self.turn,
                    END_TURN_TARGET,
                    END_TURN_SCALE,
                    END_TURN_RATE,
                );
                add_calc2(&mut self.tail, END_TAIL, 1.0, 1.0);
                add_calc0(&mut self.hover, 1.0, END_HOVER_STEP);
            }
            _ => {}
        }
        self.pos_move(actor);
        self.pitch_from_move(actor);
        self.check_ground(actor);
        self.body.at_set = false;
        self.head.at_set = false;
        add_calc2(
            &mut actor.pos.x,
            END_SLIDE_X,
            END_SLIDE_SCALE,
            END_SLIDE_STEP,
        );
        add_calc2(
            &mut actor.pos.z,
            END_SLIDE_Z,
            END_SLIDE_SCALE,
            END_SLIDE_STEP,
        );
    }

    pub fn ke_control(ke: &mut Ke, scale: f32, hio: &Bgn3Hio) {
        let step = Vec3::new(0.0, 0.0, scale);
        for i in 1..KE_STEPS {
            let taper = 0.1 * (KE_STEPS - i) as f32;
            let push = ke.head * taper;
            let flat_x = push.x + (ke.pos[i].x - ke.pos[i - 1].x);
            let flat_z = push.z + (ke.pos[i].z - ke.pos[i - 1].z);
            let rise = hio.m4c + (ke.pos[i].y + push.y) - ke.pos[i - 1].y;
            let pitch = -walk::to_s16(rise.atan2(flat_z));
            let yaw = walk::to_s16(flat_x.atan2((flat_x * flat_x + flat_z * flat_z).sqrt()));
            let m = xrot(pitch) * yrot(yaw);
            ke.pos[i] = ke.pos[i - 1] + point(&m, step);
        }
    }

    pub fn ke_move(ke: &mut Ke, scale: f32, hio: &Bgn3Hio) {
        if hio.m3e == 0 {
            return;
        }
        Self::ke_control(ke, scale, hio);
    }

    pub fn move_se_set(&self, actor: &Actor, events: &mut SimEvents) {
        let mut delta = actor.pos - actor.old.pos;
        delta.y = 0.0;
        let rotate = (delta.length() * SE_ROTATE_SCALE) as u32;
        events.list.push(SimEvent::Se(SE_CM_BGN_MECHA_ROTATE));
        let _ = rotate.min(SE_PITCH_MAX);
        delta = actor.pos - actor.old.pos;
        delta.z = 0.0;
        delta.x = 0.0;
        let rope = (delta.length() * SE_ROPE_SCALE) as u32;
        events.list.push(SimEvent::Se(SE_CM_BGN_MECHA_ROPE));
        let _ = rope.min(SE_PITCH_MAX);
    }

    pub fn damage_check(
        &mut self,
        hits: &[At],
        hio: &Bgn3Hio,
        shake: &mut Vibration,
        stop: &mut HitStop,
        audio: &mut HitAudio,
        events: &mut SimEvents,
    ) {
        if self.cool_hit != 0 {
            return;
        }
        if let Some(hit) = hits.iter().find(|at| {
            at.obj.chk_at_type(OBJ_TG_TYPE) && at.hits(self.head.centre, self.head.radius)
        }) {
            let mut info = CcAtInfo {
                mp_obj: Some(hit.obj.clone()),
                p_particle_pos: None,
                ..Default::default()
            };
            at_power_check(&mut info, &PlayerCut::default());
            events.list.push(SimEvent::Se(SE_CV_BGN_M_HIT_FACE));
            events.list.push(SimEvent::Se(SE_CM_BGN_M_FACE_HIT));
            let result = info.m_resulting_attack_type;
            let arrow = info
                .mp_actor
                .as_ref()
                .is_some_and(|attacker| attacker.name == NAME_ARROW);
            let (frame, stun) = if result == RESULT_SKULL_HAMMER {
                shake.start_shock(SHOCK_FACE_PATTERN, SHOCK_FLAGS, SHOCK_DIR);
                stop.set_pause_timer(PAUSE_HAMMER);
                (FLASH_FRAME_HAMMER, hio.m30)
            } else if result == RESULT_BOMB {
                (FLASH_FRAME_BOMB, hio.m2e)
            } else if result == RESULT_SWORD && hit.obj.at_spl == AT_SPL_UNK1 {
                stop.set_pause_timer(PAUSE_CRITICAL);
                (FLASH_FRAME_CRITICAL, hio.m2c)
            } else if result == RESULT_SWORD {
                stop.set_pause_timer(PAUSE_SWORD);
                (FLASH_FRAME_SWORD, hio.m2a)
            } else if arrow {
                (FLASH_FRAME_ARROW, hio.m28)
            } else {
                (FLASH_FRAME_OTHER, hio.m26)
            };
            if self.act != ACT_STUN {
                self.cool[0] = stun;
                self.act = ACT_STUN;
            }
            self.sub = 0;
            self.cool_hit = HIT_COOLDOWN;
            self.flash = frame;
            def_se_set(&hit.obj, SE_SELECT_HEAD, audio, events);
            return;
        }
        let core = self.parts[PART_CORE].sph;
        let core_hit = hits
            .iter()
            .any(|at| at.obj.chk_at_type(CORE_TG_TYPE) && at.hits(core.centre, core.radius));
        if core_hit {
            self.cool_hit = HIT_COOLDOWN;
            self.act = ACT_DAMAGE;
            self.sub = 0;
            if self.health != 0 {
                self.health -= 1;
                events.list.push(SimEvent::Se(SE_LK_ARROW_HIT));
                if self.health == 0 {
                    events.list.push(SimEvent::Se(SE_CV_BGN_HIT_LAST));
                    events.list.push(SimEvent::Se(SE_CM_BGN_M_BRK_ORB));
                    self.act = ACT_END;
                    self.sub = 0;
                    self.cool_hit = BROKEN_COOLDOWN;
                } else {
                    events.list.push(SimEvent::Se(SE_CV_BGN_HIT_1));
                }
            }
        }
        let mut struck: Option<&At> = None;
        for at in hits.iter() {
            if !at.obj.chk_at_type(OBJ_TG_TYPE) {
                continue;
            }
            if at.hits(self.body.centre, self.body.radius) {
                struck = Some(at);
            }
        }
        for i in 0..PART_TG {
            let sphere = self.parts[i].sph;
            for at in hits.iter() {
                if at.obj.chk_at_type(OBJ_TG_TYPE) && at.hits(sphere.centre, sphere.radius) {
                    struck = Some(at);
                }
            }
        }
        if struck.is_some() && self.cool_armour == 0 {
            self.cool_armour = ARMOUR_COOLDOWN;
            if let Some(hit) = struck {
                def_se_set(&hit.obj, SE_SELECT_BODY, audio, events);
            }
        }
    }

    pub fn move_state(
        &mut self,
        actor: &mut Actor,
        hio: &Bgn3Hio,
        esa: Option<Vec3>,
        player: Vec3,
        solid: Option<&Collision>,
        shake: &mut Vibration,
        stop: &mut HitStop,
        audio: &mut HitAudio,
        hits: &[At],
        events: &mut SimEvents,
    ) {
        let mut crawling = false;
        if hio.m05 == 0 {
            match self.act {
                ACT_HIDDEN => {}
                ACT_CRAWL => {
                    self.move0(actor, hio, esa, player, solid, shake, events);
                    if actor.speed_f > CRAWL_GATE {
                        crawling = true;
                        events.list.push(SimEvent::Se(SE_CM_BGN_M_CRAWL));
                        self.move_se_set(actor, events);
                    }
                }
                ACT_STUN => self.mahi(actor, solid),
                ACT_DAMAGE => self.damage(actor, solid, shake),
                ACT_END => self.end(actor, shake, events),
                _ => {}
            }
        }
        self.damage_check(hits, hio, shake, stop, audio, events);
        super::stream::add_calc_angle_s2(
            &mut actor.shape_angle[1],
            actor.angle[1],
            SHAPE_Y_SCALE,
            SHAPE_Y_STEP,
        );
        super::stream::add_calc_angle_s2(
            &mut actor.shape_angle[0],
            actor.angle[0],
            SHAPE_X_SCALE,
            SHAPE_X_STEP,
        );
        let base = yrot(actor.shape_angle[1]) * xrot(actor.shape_angle[0]);
        self.neck = actor.pos + point(&base, Vec3::new(0.0, 0.0, NECK_BACK));
        for i in 0..PARTS {
            self.parts[i].scale = SEG_SCALE * SIZE_D[i];
        }
        self.part_control(actor, hio);
        let model = base * zrot(actor.shape_angle[2]) * Mat4::from_scale(Vec3::splat(MODEL_SCALE));
        self.body.centre = point(&model, Vec3::ZERO);
        self.body.radius = RADIUS_BODY;
        for i in 0..KE_HAIRS {
            let m = model * xrot(ke_lean_x(i as i32)) * zrot(ke_lean_z(i as i32));
            let mut ke = self.hairs[i];
            ke.pos[0] = point(&m, Vec3::new(0.0, KE_ROOT_Y, 0.0));
            ke.head = point(&m, Vec3::new(0.0, hio.m40, 0.0)) - self.body.centre;
            Self::ke_move(&mut ke, 2.0 * hio.m44, hio);
            self.hairs[i] = ke;
        }
        if self.flash != 0 {
            self.flash -= 1;
        }
        let sway = f32::from(self.flash) * FLASH_AMPLITUDE;
        let head = base
            * zrot(actor.shape_angle[2])
            * Mat4::from_translation(Vec3::new(0.0, 0.0, NECK_FWD))
            * yrot((sway * ssin(self.frame.wrapping_mul(FLASH_PHASE_Y))) as i16)
            * xrot((sway * scos(self.frame.wrapping_mul(FLASH_PHASE_X))) as i16);
        self.head.centre = point(&head, Vec3::ZERO);
        self.head.radius = RADIUS_HEAD;
        for i in 0..PARTS {
            let part = self.parts[i];
            let m = Mat4::from_translation(part.pos)
                * yrot(part.angle_y)
                * xrot(part.angle_x)
                * Mat4::from_scale(Vec3::splat(part.scale));
            let centre = point(&m, Vec3::ZERO);
            self.parts[i].sph.centre = centre;
            self.parts[i].sph.radius = part.scale * SEG_RADIUS;
            self.parts[i].sph.at_set = crawling;
            if i < PART_KE && hio.m3e != 0 {
                for j in 0..usize::from(KE_MAX[i]) {
                    let km = m
                        * xrot(seg_lean_x(j as i32))
                        * zrot(seg_lean_z(j as i32))
                        * Mat4::from_translation(Vec3::new(0.0, part.scale * SEG_KE_Y, 0.0));
                    let mut ke = part.hair[j];
                    ke.pos[0] = point(&km, Vec3::ZERO);
                    ke.head = point(&km, Vec3::new(0.0, hio.m40, 0.0)) - centre;
                    Self::ke_move(&mut ke, part.scale * hio.m44, hio);
                    self.parts[i].hair[j] = ke;
                }
            }
        }
        self.head.at_set = crawling;
        self.body.at_set = crawling;
    }

    pub fn execute(
        &mut self,
        actor: &mut Actor,
        stat: &Bgn3Static,
        sib: &Bgn3Siblings,
        player: Vec3,
        solid: Option<&Collision>,
        shake: &mut Vibration,
        stop: &mut HitStop,
        audio: &mut HitAudio,
        hits: &[At],
        events: &mut SimEvents,
    ) -> Option<(u32, Vec3, i8)> {
        if !self.home_set {
            self.home_set = true;
            self.home = actor.pos;
        }
        if !sib.bgn_found {
            return None;
        }
        let hio = stat.hio;
        if hio.m06 != 0 {
            self.health = i16::from(hio.m06);
        }
        let esa = if sib.esa_found {
            Some(sib.esa_pos)
        } else {
            None
        };
        if sib.bgn_phase != PHASE_ACTIVE {
            self.act = ACT_HIDDEN;
            actor.pos = Vec3::new(0.0, HIDDEN_Y, 0.0);
            return None;
        }
        if self.act == ACT_HIDDEN {
            actor.pos = self.home;
            self.act = ACT_CRAWL;
            self.cool[0] = ENTER_TURN_TIMER;
            self.cool[3] = ENTER_AIM_TIMER;
        }
        self.frame = self.frame.wrapping_add(1);
        for slot in self.cool.iter_mut() {
            if *slot != 0 {
                *slot -= 1;
            }
        }
        if self.cool_hit != 0 {
            self.cool_hit -= 1;
        }
        if self.cool_armour != 0 {
            self.cool_armour -= 1;
        }
        self.move_state(
            actor, &hio, esa, player, solid, shake, stop, audio, hits, events,
        );
        if self.ks_timer != 0 {
            self.ks_timer -= 1;
            if sib.ki_count < KI_CAP {
                let x = self.rnd_fx(KI_SPAWN_SPREAD);
                let y = self.rnd_f(KI_SPAWN_RANGE) + KI_SPAWN_LOW;
                let z = self.rnd_fx(KI_SPAWN_SPREAD);
                return Some((KI_PARAM, Vec3::new(x, y, z), actor.room));
            }
        }
        None
    }
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    combat: Res<'w, Combat>,
    held: Res<'w, Held>,
    players: Query<'w, 's, (Entity, &'static Player), Without<Bgn3>>,
    arrows: Query<'w, 's, (&'static Actor, &'static arrow::Arrow), Without<Bgn3>>,
    bgns: Query<'w, 's, (&'static Actor, &'static PuppetGanon), Without<Bgn3>>,
    esas: Query<'w, 's, (&'static Actor, &'static Esa), Without<Bgn3>>,
    kis: Query<'w, 's, &'static Actor, Without<Bgn3>>,
}

pub fn control(
    watch: Watch<'_, '_>,
    mut stat: ResMut<Bgn3Static>,
    mut sib: ResMut<Bgn3Siblings>,
    mut events: ResMut<SimEvents>,
    mut audio: ResMut<HitAudio>,
    mut stop: ResMut<HitStop>,
    mut shake: ResMut<Vibration>,
    mut health: ResMut<Health>,
    mut bosses: Query<(Entity, &mut Actor, &mut Bgn3), (With<Bgn3>, Without<Player>)>,
) {
    let solid = watch.collision.as_deref();
    let mut bgn: Option<(Vec3, i8)> = None;
    for (actor, boss) in watch.bgns.iter() {
        if bgn.is_none() {
            bgn = Some((actor.pos, boss.hio_phase as i8));
        }
    }
    sib.bgn_found = bgn.is_some();
    sib.bgn_phase = bgn.map_or(0, |(_, phase)| phase);
    if let Some((pos, _)) = bgn {
        sib.bgn_pos = pos;
    }
    let esa = watch.esas.iter().next().map(|(actor, _esa)| actor.pos);
    sib.esa_found = esa.is_some();
    if let Some(esa) = esa {
        sib.esa_pos = esa;
    }
    sib.ki_count = watch.kis.iter().filter(|actor| is_ks(&actor.name)).count() as i32;
    if !stat.hio_set {
        stat.hio_set = true;
        stat.ke_color = [stat.hio.m50, stat.hio.m52, stat.hio.m54, 0xFF];
    }

    let mut hits: Vec<At> = Vec::new();
    for (_, body) in watch.players.iter() {
        if watch.combat.active && watch.combat.frame >= 1 && watch.combat.frame < 6 {
            hits.push(At {
                obj: Atk {
                    at_type: AT_TYPE_SWORD,
                    at_se: AT_SE_NONE,
                    at_spl: cut_spl(watch.combat.kind),
                    at_atp: damage::sword_power(watch.held.item, watch.combat.soup) as u8,
                    ac: Some(Attacker {
                        name: NAME_PLAYER.to_owned(),
                        pos: body.pos,
                        speed: Vec3::ZERO,
                        bomb_state: 0,
                        tsubo_type: 0,
                    }),
                },
                pos: body.pos + Vec3::new(0.0, 0.0, SWORD_REACH),
                radius: SWORD_RADIUS,
            });
        }
    }
    for (_, shaft) in watch.arrows.iter() {
        if shaft.phase != arrow::Phase::Flying {
            continue;
        }
        let at_type = match shaft.kind {
            arrow::TYPE_LIGHT => AT_TYPE_LIGHT_ARROW,
            arrow::TYPE_FIRE => AT_TYPE_FIRE_ARROW,
            arrow::TYPE_ICE => AT_TYPE_ICE_ARROW,
            _ => AT_TYPE_NORMAL_ARROW,
        };
        let tip = shaft.pos + shaft.speed * ARROW_STRETCH;
        let centre = (shaft.pos + tip) * 0.5;
        let reach = ARROW_RADIUS + (tip - shaft.pos).length() * 0.5;
        hits.push(At {
            obj: Atk {
                at_type,
                at_se: AT_SE_ARROW,
                at_spl: AT_SPL_UNK0,
                at_atp: damage::ARROW as u8,
                ac: Some(Attacker {
                    name: NAME_ARROW.to_owned(),
                    pos: shaft.pos,
                    speed: shaft.speed,
                    bomb_state: 0,
                    tsubo_type: 0,
                }),
            },
            pos: centre,
            radius: reach,
        });
    }
    let player = watch
        .players
        .iter()
        .next()
        .map_or(Vec3::ZERO, |(_, body)| body.pos);
    for (_, mut actor, mut boss) in bosses.iter_mut() {
        let actor = &mut *actor;
        let boss = &mut *boss;
        let spawned = boss.execute(
            actor,
            &stat,
            &sib,
            player,
            solid,
            &mut shake,
            &mut stop,
            &mut audio,
            &hits,
            &mut events,
        );
        let _ = spawned;
        for at in hits.iter() {
            if boss.touches(at) {
                health.damage(OBJ_ATP);
            }
        }
    }
}

fn cut_spl(kind: CutKind) -> u8 {
    match kind {
        CutKind::EA | CutKind::EB => AT_SPL_UNK1,
        _ => AT_SPL_UNK0,
    }
}

const SWORD_REACH: f32 = 20.0; // <- d_a_player_main_data.inc:159
const SWORD_RADIUS: f32 = 20.0; // <- d_a_player_main_data.inc:159
const ARROW_RADIUS: f32 = 5.0; // <- d_a_arrow.cpp:295
const ARROW_STRETCH: f32 = 1.25; // <- d_a_arrow.cpp:293
