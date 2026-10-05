use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::arrow;
use super::collision::Collision;
use super::flags::{EventFlags, StoryBits};
use super::health::Health;
use super::item::Held;
use super::player::Player;
use super::stream::{add_calc_angle_s2, cos_s, sin_s};
use super::transition::{NextStage, Target};
use super::walk::{from_s16, to_s16};
use super::water::Water;
use super::events::SimEvents;
use super::flags::Switches;
use super::flags::ZoneActors;

pub const PUPPET_GANON_ACTOR: &str = "Bgn";

pub fn is_puppet_ganon(name: &str) -> bool {
    name == PUPPET_GANON_ACTOR
}

const PRM_BYTE_MASK: u32 = 0xFF;
const NO_BYTE: u8 = 0xFF;

const HAND_MAX: i32 = 5;
const TAIL_MAX: i32 = 10;
const HEAD_PARTS: usize = 2;
const PELVIS_PARTS: usize = 2;
const ARM_PARTS: usize = 21;
const LEG_PARTS: usize = 4;
const TAIL_PARTS: usize = 21;
const MOVE_COUNT: usize = 8;
const TIMER_COUNT: usize = 5;
const ROPE_POINTS: usize = 60;

const CHAIN_HEAD: u8 = 0;
const CHAIN_PELVIS: u8 = 2;
const CHAIN_LEFT_ARM: u8 = 3;
const CHAIN_RIGHT_ARM: u8 = 4;
const CHAIN_LEFT_LEG: u8 = 5;
const CHAIN_RIGHT_LEG: u8 = 6;
const CHAIN_TAIL: u8 = 7;

const CHAIN_TAIL_ALIAS: usize = 9;
const CHAIN_TIP: usize = 5;

const HIO_M004: i16 = 0;
const HIO_M0240: u8 = 0;
const HIO_M025: u8 = 0;
const HIO_M027: u8 = 0;
const HIO_M028: u8 = 1;
const HIO_M029: u8 = 0;
const HIO_M02A: u8 = 0;
const HIO_M02B: u8 = 0;
const HIO_M02C: u8 = 0;
const HIO_M02D: u8 = 0;
const HIO_M02E: u8 = 0;
const HIO_M02F: u8 = 0;
const HIO_M030: u8 = 0;
const HIO_M034: f32 = -20.0;
const HIO_M038: f32 = -30.0;
const HIO_M03C: i16 = 0;
const HIO_M03E: i16 = 22000;
const HIO_M040: i16 = 0;
const HIO_M042: i16 = 0;
const HIO_M04C: f32 = 950.0;
const HIO_M054: f32 = 0.0;
const HIO_M058: f32 = 2000.0;
const HIO_M05C: f32 = 800.0;
const HIO_M060: f32 = 0.0;
const HIO_M064: f32 = -800.0;
const HIO_M068: f32 = -800.0;
const HIO_M06C: f32 = 700.0;
const HIO_M070: f32 = -900.0;
const HIO_M074: f32 = 300.0;
const HIO_M078: f32 = -700.0;
const HIO_M07C: f32 = -900.0;
const HIO_M080: f32 = 300.0;
const HIO_M084: f32 = 500.0;
const HIO_M088: f32 = -1300.0;
const HIO_M08C: f32 = -100.0;
const HIO_M090: f32 = -500.0;
const HIO_M094: f32 = -1300.0;
const HIO_M098: f32 = -100.0;
const HIO_M09C: f32 = 0.0;
const HIO_M0A0: f32 = -700.0;
const HIO_M0A4: f32 = -1200.0;
const HIO_M0A8: f32 = 0.0;
const HIO_M0AA: f32 = 0.0;
const HIO_M0AC: f32 = 0.0;
const HIO_M0AE: f32 = 0.0;
const HIO_M0B0: f32 = 0.0;
const HIO_M0B2: f32 = 0.0;
const HIO_M0B4: f32 = 0.0;
const HIO_M0B6: f32 = 0.0;
const HIO_M0B8: f32 = 0.0;
const HIO_M0BA: f32 = 0.0;
const HIO_M0BC: f32 = 0.0;
const HIO_M0BE: f32 = 0.0;
const HIO_M0C0: f32 = 0.0;
const HIO_M0C2: f32 = 0.0;
const HIO_M0C4: f32 = 0.0;
const HIO_M0C6: f32 = 0.0;
const HIO_M0C8: f32 = 0.0;
const HIO_M0CA: f32 = 0.0;
const HIO_M0CC: f32 = 0.0;
const HIO_M0CE: f32 = 0.0;
const HIO_M0D0: f32 = 0.0;
const HIO_M0D2: f32 = 0.0;
const HIO_M0D4: f32 = 40.0;
const HIO_M0D8: i16 = 120;
const HIO_M0DA: i16 = 20;
const HIO_M0DC: i16 = 1200;
const HIO_M0DE: i16 = 1500;
const HIO_M0E0: i16 = 2000;
const HIO_M0E2: i16 = 300;
const HIO_M0E4: i16 = 200;
const HIO_M0E6: i16 = 150;
const HIO_M0E8: i16 = 15;
const HIO_M0EA: i16 = 15;
const HIO_M0EC: i16 = 15;
const HIO_M0EE: i16 = 500;
const HIO_M0F4: f32 = 1.3;
const HIO_M0F8: f32 = 1.6;
const HIO_M100: f32 = 1.8;
const HIO_M104: f32 = 1.2;
const HIO_M108: f32 = 0.3;
const HIO_M10C: f32 = 1.2;
const HIO_M110: f32 = 0.4;
const HIO_M114: f32 = 1.2;
const HIO_M118: f32 = 0.1;
const HIO_M11C: f32 = 185.0;
const HIO_M120: f32 = 185.0;
const HIO_M124: f32 = 185.0;
const HIO_M128: f32 = 185.0;
const HIO_M168: f32 = 100.0;
const HIO_M12C: f32 = 0.0;
const HIO_M130: f32 = -10.0;
const HIO_M134: f32 = 120.0;
const HIO_M138: f32 = 100.0;
const HIO_M13C: f32 = 25.0;
const HIO_M140: f32 = 20.0;
const HIO_M144: f32 = 0.0;
const HIO_M148: f32 = -115.0;
const HIO_M14C: f32 = 0.0;
const HIO_M150: f32 = 80.0;
const HIO_M154: f32 = -20.0;
const HIO_M158: f32 = 100.0;
const HIO_M15C: f32 = 0.0;
const HIO_M160: f32 = -60.0;
const HIO_M164: f32 = 150.0;
const HIO_M16C: f32 = 0.0;
const HIO_M170: f32 = -100.0;
const HIO_KEESE_NUM_3HP: i8 = 3;
const HIO_KEESE_NUM_2HP: i8 = 4;
const HIO_KEESE_NUM_1HP: i8 = 5;
const HIO_KEESE_NUM_MAX: i32 = 5;

const REG0_F0: f32 = 0.0;
const REG0_F1: f32 = 0.0;
const REG0_F2: f32 = 0.0;
const REG0_F3: f32 = 0.0;
const REG0_F4: f32 = 0.0;
const REG0_F5: f32 = 0.0;
const REG0_F6: f32 = 0.0;
const REG0_F7: f32 = 0.0;
const REG0_F8: f32 = 0.0;
const REG0_F9: f32 = 0.0;
const REG0_F10: f32 = 0.0;
const REG0_F11: f32 = 0.0;
const REG0_F12: f32 = 0.0;
const REG0_F13: f32 = 0.0;
const REG0_F14: f32 = 0.0;
const REG0_F15: f32 = 0.0;
const REG0_F16: f32 = 0.0;
const REG0_F18: f32 = 0.0;
const REG0_F19: f32 = 0.0;
const REG0_F20: f32 = 0.0;
const REG0_F21: f32 = 0.0;
const REG0_F25: f32 = 0.0;
const REG0_S0: i16 = 0;
const REG0_S1: i16 = 0;
const REG0_S2: i16 = 0;
const REG0_S3: i16 = 0;
const REG0_S4: i16 = 0;
const REG0_S5: i16 = 0;
const REG0_S6: i16 = 0;
const REG0_S7: i16 = 0;
const REG0_S8: i16 = 0;
const REG0_S9: i16 = 0;
const REG0_F17: f32 = 0.0;
const REG8_F3: f32 = 0.0;
const REG8_F4: f32 = 0.0;
const REG8_F16: f32 = 0.0;
const REG8_S7: i16 = 0;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const SE_CM_BGN_MECHA_ROTATE: u32 = 0x705A;
const SE_CM_BGN_MECHA_ROPE: u32 = 0x705B;
const SE_CM_BGN_D_HIT_PUNCH: u32 = 0x6A40;
const SE_CM_BGN_D_HIT_PUNCH_W: u32 = 0x6A41;
const SE_CM_BGN_STRING_RIPPLE: u32 = 0x6A42;
const SE_CM_BGN_BODY_RIPPLE: u32 = 0x6A43;
const SE_CM_BGN_M_BRK_ORB: u32 = 0x5983;
const SE_CM_BGN_LAST_EXPLODE: u32 = 0x5984;
const SE_CM_BGN_METAM_EXPLODE: u32 = 0x5980;
const SE_CM_BGN_D_DANCE: u32 = 0x516F;
const SE_CM_BGN_D_PUNCH_FIRE: u32 = 0x5971;
const SE_CM_BGN_D_ROPE_RESET: u32 = 0x5972;
const SE_CM_BGN_D_KAZEKIRI: u32 = 0x5973;
const SE_CM_BGN_D_FALL_WATER: u32 = 0x5974;
const SE_CM_BGN_D_FALL: u32 = 0x5975;
const SE_CM_BGN_D_TO_T_1: u32 = 0x5976;
const SE_CM_BGN_D_TO_T_2: u32 = 0x5977;
const SE_CM_BGN_T_MOUSE_OPEN: u32 = 0x5978;
const SE_CM_BGN_T_MOUSE_CLOSE: u32 = 0x5979;
const SE_CV_BGN_HIT_1: u32 = 0x496E;
const SE_CV_BGN_HIT_2: u32 = 0x496F;
const SE_CV_BGN_D_ATTACK: u32 = 0x4971;
const SE_CV_BGN_DIE: u32 = 0x4972;
const SE_LK_ARROW_HIT: u32 = 0x2879;
const SE_LK_LAST_HIT: u32 = 0x2828;
const SE_LK_HAMMER_HIT: u32 = 0x2855;
const SE_LK_HS_SPIKE: u32 = 0x286F;
const SE_LK_W_WEP_HIT: u32 = 0x2833;
const SE_LK_SW_HIT_S: u32 = 0x2803;
const SE_LK_MS_WEP_HIT: u32 = 0x2834;

const BGM_BGN_KUGUTSU: u32 = 0x8000_0059;
const BGM_BGN_TARABA: u32 = 0x8000_005A;
const BGM_BGN_TARABA_IN: u32 = 0x8000_005C;

const EVENT_BIT_3F10: u16 = 0x3F10;
const STAGE_GANONK: &str = "GanonK";
const STAGE_GANONK_ROOM: u8 = 0;
const STAGE_GANONK_POINT: u8 = 4;

const ACTION_NONE: i16 = 0;
const ACTION_PUNCH: i16 = 1;
const ACTION_BODY: i16 = 2;
const ACTION_TAIL: i16 = 3;
const ACTION_HEAD_RECOVER: i16 = 4;
const ACTION_DAMAGE: i16 = 5;
const ACTION_HENSIN: i16 = 6;
const ACTION_START: i16 = 7;
const ACTION_DONE: i16 = 10;

const ATTN_LOCKON_BATTLE: i32 = 1;
const ATTN_DIST_BATTLE: i32 = 0x22;

const MAX_HEALTH: i32 = 3;
const CORE_LOCK: i16 = 100;
const CORE_TIMER: i16 = 50;
const CORE_EFFECT: i16 = 100;
const CORE_RADIUS: f32 = 150.0;
const CORE_TAG_ARROW: i16 = 0xd;
const BODY_RADIUS_BASE: f32 = 120.0;
const PART_RADIUS_BASE: f32 = 120.0;

const EMODE_NONE: u8 = 0;
const EMODE_LOOP: u8 = 2;
const STATE_STOP: u8 = 1;
const STATE_LOOP: u8 = 2;
const BCK_HEAD1_FRAMES: i16 = 48;
const MORF_FRAME_GATE: i32 = 0x18;

const GR_LINE_TOP: f32 = 200.0;
const GR_LINE_DOWN: f32 = 1000.0;
const GR_WATER_Y: f32 = -2.0;
const WATER_Y: f32 = 90.0;
const BODY_LIFT: f32 = 100.0;
const SHOULDER_Y: f32 = 50.0;

const DANCE_A_STEP: i16 = 0x10;
const DANCE_B_STEP: i16 = 8;
const DANCE_B_TARGET_A: i16 = 0x1555;
const DANCE_B_TARGET_B: i16 = -0x1555;

const CUT_STATE_150: i8 = -106;
const CAM_FOV: f32 = 55.0;
const CAM_BANK_SCALE: f32 = 7.5;

const WIND_STEP_F: f32 = 0.053247336;
const WIND_TAPER: f32 = 0.01666667;

pub const CULL_MIN: [f32; 3] = [-400.0, -1000.0, -400.0];
pub const CULL_MAX: [f32; 3] = [400.0, 1200.0, 400.0];

const DANCE_PAUSE_1: [[i16; 3]; MOVE_COUNT] = [
    [0, 0, -0x4b0],
    [0, 0, 0],
    [0, 0, 0],
    [0x1f4, 0x3e8, 0x5dc],
    [-0x1f4, 0x3e8, 0x5dc],
    [0x4b, 0x384, -0x2bc],
    [0x1f4, -0x514, -0x64],
    [-0xc8, 0x258, -0x190],
];

const DANCE_PAUSE_2: [[i16; 3]; MOVE_COUNT] = [
    [0, 0, -0x4b0],
    [0, 0, 0],
    [0, 0x12c, 0],
    [0x1f4, -0x1f4, 0x5dc],
    [-0x1f4, -0x1f4, 0x5dc],
    [0x4b, 0x190, -0x2bc],
    [0x1f4, -0x44c, -0x320],
    [-0xc8, 0xc8, -0x1f4],
];

const DANCE_PAUSE_3: [[i16; 3]; MOVE_COUNT] = [
    [0, 0, -0x3e8],
    [0, 0, 0],
    [0x1f4, 0, -0xc8],
    [-0xc8, 0x3e8, 0x15e],
    [0xc8, 0x578, 0x15e],
    [-0x190, 0x1f4, 0x1f4],
    [0x190, -0xc8, -0x12c],
    [0x320, 0x190, -0xc8],
];

const DANCE_PAUSE_4: [[i16; 3]; MOVE_COUNT] = [
    [0, 0, -0x3e8],
    [0, 0, 0],
    [-0x1f4, 0, -0xc8],
    [-0xc8, 0x578, 0x15e],
    [0xc8, 0x3e8, 0x15e],
    [-0x190, -0xc8, -0x12c],
    [0x190, 0x1f4, 0x1f4],
    [-0x320, 0x190, -0xc8],
];

const PUNCH_LR1_D: [[i16; 3]; MOVE_COUNT] = [
    [0, 0, -0x6a4],
    [0, 0, 0],
    [-0x96, 0, 0x28a],
    [0x64, 0x578, -0x1f4],
    [-0x64, 0x4b0, 0x1f4],
    [0xc8, 0x12c, 0x384],
    [-0x64, 0x1f4, 0x384],
    [-0x258, 0x1f4, 0x12c],
];

const PUNCH_LR12_D: [[i16; 3]; MOVE_COUNT] = [
    [0, 0, -0x6a4],
    [0, 0, 0],
    [0x96, 0, 0x28a],
    [0x64, 0x4b0, -0x1f4],
    [-0x64, 0x578, 0x1f4],
    [0xc8, 0x1f4, 0x384],
    [-0x64, 0x12c, 0x384],
    [0x258, 0x1f4, 0x12c],
];

const PUNCH_LR2_D: [[i16; 3]; MOVE_COUNT] = [
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0x1f4],
    [-0x1f4, 0x96, 0x3e8],
    [0x1f4, 0x96, 0x3e8],
    [0x96, 0x258, 0x3e8],
    [-0x96, 0x258, 0x384],
    [0, 0x3b6, 0x64],
];

const PUNCH_R1_D: [[i16; 3]; MOVE_COUNT] = [
    [-0x96, 0, 0],
    [0, 0, 0],
    [-0xc8, 0, 0x190],
    [0, 0x190, 0x258],
    [-0x96, 0x640, -0x190],
    [-0x258, 0x12c, 0x28a],
    [-0x4b0, 0, 0xc8],
    [0x2bc, 0x2bc, 0xc8],
];

const PUNCH_R2_D: [[i16; 3]; MOVE_COUNT] = [
    [0x96, 0, -0x1f4],
    [0, 0, 0],
    [0x258, 0, 0],
    [0x28a, 0x3e8, -0x1f4],
    [-0x258, 0, 0x4b0],
    [0x3e8, 0xfa, 0x7d0],
    [0x384, 0xc8, -0x12c],
    [0, 0x320, -0x15e],
];

const PUNCH_L1_D: [[i16; 3]; MOVE_COUNT] = [
    [0x96, 0, 0],
    [0, 0, 0],
    [0xc8, 0, 0x190],
    [0x96, 0x640, -0x190],
    [0, 0x190, 0x258],
    [0x4b0, 0, 0x7d0],
    [0x258, 0x12c, 0x28a],
    [-0x2bc, 0x2bc, 0xc8],
];

const PUNCH_L2_D: [[i16; 3]; MOVE_COUNT] = [
    [-0x96, 0, -0x1f4],
    [0, 0, 0],
    [-0x258, 0, 0],
    [0x258, 0, 0x4b0],
    [-0x28a, 0x3e8, -0x1f4],
    [-0x384, 0xc8, -0x12c],
    [-0x3e8, 0xfa, 0x7d0],
    [0, 0x320, -0x15e],
];

const START_PAUSE: [[i16; 3]; MOVE_COUNT] = [
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [-0x64, 0xfa, 0],
    [0x190, 0xfa, 0],
    [-0xc8, 0, 0],
    [0xc8, 0, 0],
    [0, 0, 0],
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat {
    pub x: Vec3,
    pub y: Vec3,
    pub z: Vec3,
    pub t: Vec3,
}

impl Mat {
    const fn none() -> Self {
        Self {
            x: Vec3::ZERO,
            y: Vec3::ZERO,
            z: Vec3::ZERO,
            t: Vec3::ZERO,
        }
    }

    fn pos(&self, src: Vec3) -> Vec3 {
        self.t + self.x * src.x + self.y * src.y + self.z * src.z
    }

    fn concat(&mut self, a: Vec3, b: Vec3, c: Vec3) {
        let x = self.x * a.x + self.y * a.y + self.z * a.z;
        let y = self.x * b.x + self.y * b.y + self.z * b.z;
        let z = self.x * c.x + self.y * c.y + self.z * c.z;
        self.x = x;
        self.y = y;
        self.z = z;
    }

    fn trans(&mut self, at: Vec3, concat: bool) {
        if concat {
            self.t += at;
        } else {
            self.x = Vec3::X;
            self.y = Vec3::Y;
            self.z = Vec3::Z;
            self.t = at;
        }
    }

    fn scale(&mut self, at: Vec3) {
        self.x *= at.x;
        self.y *= at.y;
        self.z *= at.z;
    }

    fn xrot_s(&mut self, angle: i16) {
        let (sin, cos) = from_s16(angle).sin_cos();
        self.x = Vec3::X;
        self.y = Vec3::new(0.0, cos, sin);
        self.z = Vec3::new(0.0, -sin, cos);
        self.t = Vec3::ZERO;
    }

    fn xrot_m(&mut self, angle: i16) {
        let (sin, cos) = from_s16(angle).sin_cos();
        self.concat(
            Vec3::X,
            Vec3::new(0.0, cos, sin),
            Vec3::new(0.0, -sin, cos),
        );
    }

    fn yrot_s(&mut self, angle: i16) {
        let (sin, cos) = from_s16(angle).sin_cos();
        self.x = Vec3::new(cos, 0.0, -sin);
        self.y = Vec3::Y;
        self.z = Vec3::new(sin, 0.0, cos);
        self.t = Vec3::ZERO;
    }

    fn yrot_m(&mut self, angle: i16) {
        let (sin, cos) = from_s16(angle).sin_cos();
        self.concat(
            Vec3::new(cos, 0.0, -sin),
            Vec3::Y,
            Vec3::new(sin, 0.0, cos),
        );
    }

    

    fn zrot_m(&mut self, angle: i16) {
        let (sin, cos) = from_s16(angle).sin_cos();
        self.concat(
            Vec3::new(cos, sin, 0.0),
            Vec3::new(-sin, cos, 0.0),
            Vec3::Z,
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub attribute: u8,
    pub state: u8,
    pub start: i16,
    pub end: i16,
    pub loop_point: i16,
    pub rate: f32,
    pub frame: f32,
}

impl Frame {
    fn head() -> Self {
        Self {
            attribute: EMODE_LOOP,
            state: 0,
            start: 0,
            end: BCK_HEAD1_FRAMES,
            loop_point: 0,
            rate: 1.0,
            frame: 0.0,
        }
    }

    fn update(&mut self) {
        self.state = 0;
        self.frame += self.rate;
        match self.attribute {
            EMODE_NONE => {
                if self.frame < self.start as f32 {
                    self.frame = self.start as f32;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
                if self.frame >= self.end as f32 {
                    self.frame = self.end as f32 - 0.001;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
            }
            EMODE_LOOP => {
                let low = f32::from(self.loop_point) - self.start as f32;
                let high = self.end as f32 - self.loop_point as f32;
                while self.frame < self.start as f32 {
                    self.state |= STATE_LOOP;
                    if low <= 0.0 {
                        break;
                    }
                    self.frame += low;
                }
                while self.frame >= self.end as f32 {
                    self.state |= STATE_LOOP;
                    if high <= 0.0 {
                        break;
                    }
                    self.frame -= high;
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Part {
    pub rot: [i16; 3],
    pub pos: Vec3,
    pub world: Vec3,
    pub scale: f32,
    pub centre: Vec3,
    pub radius: f32,
    pub tag: i8,
    pub flash: i16,
    pub effect: i16,
    pub flash_angle: i16,
    pub flash_rate: f32,
    pub at: bool,
    pub weak: u8,
}

impl Part {
    const fn idle() -> Self {
        Self {
            rot: [0; 3],
            pos: Vec3::ZERO,
            world: Vec3::ZERO,
            scale: 0.0,
            centre: Vec3::ZERO,
            radius: 0.0,
            tag: 0,
            flash: 0,
            effect: 0,
            flash_angle: 0,
            flash_rate: 0.0,
            at: false,
            weak: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Anchor {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: i16,
}

impl Anchor {
    const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        yaw: 0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Move {
    pub rope: bool,
    pub flag: u8,
    pub pos: Vec3,
    pub target: [i16; 3],
    pub slack: f32,
    pub wave: f32,
    pub sway: f32,
    pub tension: i16,
    pub phase_x: i16,
    pub phase_z: i16,
    pub timer: i16,
    pub ripple: i16,
    pub rope_sway: f32,
    pub join: i8,
    pub himo: [Vec3; ROPE_POINTS],
}

impl Move {
    const fn idle() -> Self {
        Self {
            rope: false,
            flag: 0,
            pos: Vec3::ZERO,
            target: [0; 3],
            slack: 0.0,
            wave: 0.0,
            sway: 0.0,
            tension: 0,
            phase_x: 0,
            phase_z: 0,
            timer: 0,
            ripple: 0,
            rope_sway: 0.0,
            join: 0,
            himo: [Vec3::ZERO; ROPE_POINTS],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum At {
    Idle,
    Sword,
    Boomerang,
    Hookshot,
    Wind,
    Arrow,
    FireArrow,
    IceArrow,
    LightArrow,
    Hammer,
    Bomb,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blow {
    pub at: At,
    pub blade: u8,
    pub soup: bool,
    pub pos: Option<Vec3>,
}

impl At {
    fn reach(self) -> f32 {
        match self {
            Self::Idle => 0.0,
            Self::Sword => 20.0,
            Self::Arrow | Self::FireArrow | Self::IceArrow => 35.0,
            Self::Hammer => 50.0,
            Self::Hookshot => 5.0,
            Self::Boomerang => 90.0,
            Self::Bomb => 150.0,
            Self::Wind | Self::LightArrow => 0.0,
        }
    }

    fn at_se(self) -> u32 {
        match self {
            Self::Hammer => SE_LK_HAMMER_HIT,
            Self::Arrow | Self::FireArrow | Self::IceArrow | Self::LightArrow => SE_LK_ARROW_HIT,
            Self::Hookshot => SE_LK_HS_SPIKE,
            Self::Boomerang => SE_LK_W_WEP_HIT,
            Self::Bomb => SE_LK_MS_WEP_HIT,
            Self::Sword | Self::Idle | Self::Wind => SE_LK_SW_HIT_S,
        }
    }
}

pub struct Env<'a> {
    pub player_pos: Vec3,
    pub solid: Option<&'a Collision>,
    pub water: Option<&'a Water>,
    pub blow: Blow,
    pub keese: i32,
    pub stories_done: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct KeeseSpawn {
    pub pos: Vec3,
    pub room: i8,
}

pub struct Sink<'a> {
    pub events: &'a mut SimEvents,
    pub switches: &'a mut Switches,
    pub stories: &'a mut StoryBits,
    pub flags: &'a mut EventFlags,
    pub health: &'a mut Health,
    pub next: &'a mut NextStage,
    pub drop: &'a mut Vec<(Entity, i8, u16)>,
    pub keese: &'a mut Vec<KeeseSpawn>,
}

impl Sink<'_> {
    fn se(&mut self, id: u32) {
        self.events.list.push(super::events::SimEvent::Se(id));
    }

    fn bgm(&mut self, id: u32) {
        self.events.list.push(super::events::SimEvent::Bgm(id));
    }

    fn stop_bgm(&mut self) {
        self.events.list.push(super::events::SimEvent::Bgm(0));
    }
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

fn atan2s(f0: f32, f1: f32) -> i16 {
    to_s16(f0.atan2(f1))
}

fn spread(y: f32, z: f32) -> f32 {
    (y * y + z * z).sqrt()
}

fn reaches(centre: Vec3, radius: f32, at: Vec3, reach: f32) -> bool {
    (at - centre).length_squared() <= (radius + reach) * (radius + reach)
}

fn length_of(v: Vec3) -> f32 {
    (v.x * v.x + v.y * v.y + v.z * v.z).sqrt()
}

fn timer(value: &mut i16) -> bool {
    if *value != 0 {
        *value -= 1;
    }
    *value == 0
}

fn anchor_vec(anchor: Anchor) -> Vec3 {
    Vec3::new(anchor.x, anchor.y, anchor.z)
}

fn gr_probe(
    solid: &Option<&Collision>,
    water: Option<&Water>,
    at: &mut Vec3,
) -> bool {
    let top = Vec3::new(at.x, at.y + GR_LINE_TOP, at.z);
    let bottom = Vec3::new(at.x, at.y - GR_LINE_DOWN, at.z);
    let hit = match solid {
        Some(solid) => solid.ground_at(at.x, at.z),
        None => None,
    };
    if let Some(y) = hit
        && y <= top.y
        && y >= bottom.y
    {
        at.y = GR_WATER_Y;
        return match water {
            Some(water) => !water.in_area,
            None => true,
        };
    }
    true
}

fn roll_value(seed: &mut [i32; 3]) -> f32 {
    rnd(seed) * 20.0
}

fn to_i16x3(v: Vec3) -> [i16; 3] {
    [v.x as i16, v.y as i16, v.z as i16]
}

const HIO_M0FC: f32 = 3.25;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct PuppetGanon {
    pub morph: Frame,
    pub prm: u8,
    pub body_draw: i8,
    pub hio_phase: u8,
    pub hio_owned: u8,
    pub health: i32,
    pub max_health: i32,
    pub home: Vec3,
    pub speed: Vec3,
    pub shape_angle: [i16; 3],
    pub eye: Vec3,
    pub attention: Vec3,
    pub attn_flags: i32,
    pub attn_dist: i32,
    pub mtx: Mat,
    pub head: [Part; HEAD_PARTS],
    pub pelvis: [Part; PELVIS_PARTS],
    pub left_arm: [Part; ARM_PARTS],
    pub right_arm: [Part; ARM_PARTS],
    pub left_leg: [Part; LEG_PARTS],
    pub right_leg: [Part; LEG_PARTS],
    pub tail: [Part; TAIL_PARTS],
    pub moves: [Move; MOVE_COUNT],
    pub blue_pos: [[Vec3; ROPE_POINTS]; MOVE_COUNT],
    pub blue_size: [[u8; ROPE_POINTS]; MOVE_COUNT],
    pub red_pos: [Vec3; ROPE_POINTS],
    pub red_size: [u8; ROPE_POINTS],
    pub cs_pos: [Vec3; ROPE_POINTS],
    pub cs_size: [u8; ROPE_POINTS],
    pub cs_himo: [Vec3; ROPE_POINTS],
    pub rope_anchor: [Vec3; MOVE_COUNT],
    pub core_pos: Vec3,
    pub core_centre: Vec3,
    pub core_radius: f32,
    pub body_centre: Vec3,
    pub body_radius: f32,
    pub body_at: bool,
    pub moved_by: Vec3,
    pub anchor_angle: [i16; 3],
    pub body_angle: [i16; 3],
    pub yaw_tick: i16,
    pub hand_scale: [f32; 2],
    pub hand_dilate: [f32; 2],
    pub pos_speed: f32,
    pub pos_target: f32,
    pub body: Vec3,
    pub body_prev: Vec3,
    pub sway: i16,
    pub frame: i16,
    pub action: i16,
    pub sub: i16,
    pub dance: i16,
    pub dance_sub: i16,
    pub yaw_bob: i16,
    pub yaw_step: i16,
    pub move_count: i16,
    pub dest: Vec3,
    pub dest_yaw: i16,
    pub dest_speed: f32,
    pub shake_timer: i16,
    pub chain_speed: f32,
    pub left_held: bool,
    pub right_held: bool,
    pub left_arm_tip: Vec3,
    pub right_arm_tip: Vec3,
    pub left_arm_step: Vec3,
    pub right_arm_step: Vec3,
    pub timers: [i16; TIMER_COUNT],
    pub core_lock: i16,
    pub hit_se: i16,
    pub water_y: f32,
    pub arrow_flash: i16,
    pub arrow_effect: i16,
    pub flash_angle: i16,
    pub flash_rate: f32,
    pub hit_tag: i16,
    pub ripple_timer: i16,
    pub ripple_pos: Vec3,
    pub blure: i16,
    pub cut_state: i8,
    pub cut_count: i16,
    pub cut_ordered: bool,
    pub cam_eye: Vec3,
    pub cam_rise: Vec3,
    pub cam_center: Vec3,
    pub cam_anchor: Anchor,
    pub cam_follow: Vec3,
    pub cut_param: f32,
    pub shake: f32,
    pub fov: f32,
    pub cam_bank: i16,
    pub cut_player: bool,
    pub cut_player_pos: Vec3,
    pub cut_player_yaw: i16,
    pub player_demo: i32,
    pub player_voice: i32,
    pub chest_pot: bool,
    pub defeat_state: i8,
    pub defeat_sway: f32,
    pub reflection: i8,
    pub keese_spawn: i8,
    pub scale: f32,
    pub squash: f32,
    pub fog: f32,
    pub exit_mode: i8,
    pub entered: bool,
    pub seed: [i32; 3],
}

impl PuppetGanon {
    pub fn spawn(parameters: u32) -> Self {
        let prm = (parameters & PRM_BYTE_MASK) as u8;
        Self {
            morph: Frame::head(),
            prm,
            body_draw: 0,
            hio_phase: HIO_M0240,
            hio_owned: 0,
            health: MAX_HEALTH,
            max_health: MAX_HEALTH,
            home: Vec3::ZERO,
            speed: Vec3::ZERO,
            shape_angle: [0; 3],
            eye: Vec3::ZERO,
            attention: Vec3::ZERO,
            attn_flags: 0,
            attn_dist: 0,
            mtx: Mat::none(),
            head: [Part::idle(); HEAD_PARTS],
            pelvis: [Part::idle(); PELVIS_PARTS],
            left_arm: [Part::idle(); ARM_PARTS],
            right_arm: [Part::idle(); ARM_PARTS],
            left_leg: [Part::idle(); LEG_PARTS],
            right_leg: [Part::idle(); LEG_PARTS],
            tail: [Part::idle(); TAIL_PARTS],
            moves: [Move::idle(); MOVE_COUNT],
            blue_pos: [[Vec3::ZERO; ROPE_POINTS]; MOVE_COUNT],
            blue_size: [[0; ROPE_POINTS]; MOVE_COUNT],
            red_pos: [Vec3::ZERO; ROPE_POINTS],
            red_size: [0; ROPE_POINTS],
            cs_pos: [Vec3::ZERO; ROPE_POINTS],
            cs_size: [0; ROPE_POINTS],
            cs_himo: [Vec3::ZERO; ROPE_POINTS],
            rope_anchor: [Vec3::ZERO; MOVE_COUNT],
            core_pos: Vec3::ZERO,
            core_centre: Vec3::ZERO,
            core_radius: CORE_RADIUS,
            body_centre: Vec3::ZERO,
            body_radius: 0.0,
            body_at: false,
            moved_by: Vec3::ZERO,
            anchor_angle: [0; 3],
            body_angle: [0; 3],
            yaw_tick: 0,
            hand_scale: [0.0; 2],
            hand_dilate: [0.0; 2],
            pos_speed: 0.0,
            pos_target: 0.0,
            body: Vec3::ZERO,
            body_prev: Vec3::ZERO,
            sway: 0,
            frame: 0,
            action: ACTION_NONE,
            sub: 0,
            dance: 0,
            dance_sub: 0,
            yaw_bob: 0,
            yaw_step: 0,
            move_count: 0,
            dest: Vec3::ZERO,
            dest_yaw: 0,
            dest_speed: 0.0,
            shake_timer: 0,
            chain_speed: 0.0,
            left_held: false,
            right_held: false,
            left_arm_tip: Vec3::ZERO,
            right_arm_tip: Vec3::ZERO,
            left_arm_step: Vec3::ZERO,
            right_arm_step: Vec3::ZERO,
            timers: [0; TIMER_COUNT],
            core_lock: 0,
            hit_se: 0,
            water_y: WATER_Y,
            arrow_flash: 0,
            arrow_effect: 0,
            flash_angle: 0,
            flash_rate: 0.0,
            hit_tag: 0,
            ripple_timer: 0,
            ripple_pos: Vec3::ZERO,
            blure: 0,
            cut_state: 0,
            cut_count: 0,
            cut_ordered: false,
            cam_eye: Vec3::ZERO,
            cam_rise: Vec3::ZERO,
            cam_center: Vec3::ZERO,
            cam_anchor: Anchor::ZERO,
            cam_follow: Vec3::ZERO,
            cut_param: 0.0,
            shake: 0.0,
            fov: 0.0,
            cam_bank: 0,
            cut_player: false,
            cut_player_pos: Vec3::ZERO,
            cut_player_yaw: 0,
            player_demo: -1,
            player_voice: -1,
            chest_pot: false,
            defeat_state: 0,
            defeat_sway: 0.0,
            reflection: 0,
            keese_spawn: 0,
            scale: 1.0,
            squash: 0.0,
            fog: 0.0,
            exit_mode: 0,
            entered: false,
            seed: [RND_SEED; 3],
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.prm != NO_BYTE
    }

    fn rnd_f(&mut self, range: f32) -> f32 {
        rnd(&mut self.seed) * range
    }

    fn rnd_fx(&mut self, range: f32) -> f32 {
        range * (rnd(&mut self.seed) - 0.5) * 2.0
    }

    fn parts(&self, chain: u8) -> &[Part] {
        match chain {
            CHAIN_HEAD => &self.head,
            CHAIN_PELVIS => &self.pelvis,
            CHAIN_LEFT_ARM => &self.left_arm,
            CHAIN_RIGHT_ARM => &self.right_arm,
            CHAIN_LEFT_LEG => &self.left_leg,
            CHAIN_RIGHT_LEG => &self.right_leg,
            _ => &self.tail,
        }
    }

    fn parts_mut(&mut self, chain: u8) -> &mut [Part] {
        match chain {
            CHAIN_HEAD => &mut self.head,
            CHAIN_PELVIS => &mut self.pelvis,
            CHAIN_LEFT_ARM => &mut self.left_arm,
            CHAIN_RIGHT_ARM => &mut self.right_arm,
            CHAIN_LEFT_LEG => &mut self.left_leg,
            CHAIN_RIGHT_LEG => &mut self.right_leg,
            _ => &mut self.tail,
        }
    }

    fn part(&self, chain: u8, index: usize) -> Part {
        self.parts(chain)
            .get(index)
            .copied()
            .unwrap_or(Part::idle())
    }

    fn weak_model(&self) -> u8 {
        match self.health {
            3 => 1,
            2 => 2,
            1 => 3,
            _ => 0,
        }
    }

    fn pelvis_probe(&self) -> f32 {
        self.part(CHAIN_PELVIS, 0).pos.y
    }

    fn grab_left(&self) -> Vec3 {
        self.part(CHAIN_PELVIS, 0).pos
    }

    fn grab_right(&self) -> Vec3 {
        self.part(CHAIN_RIGHT_ARM, CHAIN_TIP).pos
    }

    fn player_yaw(env: &Env<'_>, from: Vec3) -> i16 {
        atan2s(env.player_pos.x - from.x, env.player_pos.z - from.z)
    }

    fn gr_check(&self, at: &mut Vec3, env: &Env<'_>) -> bool {
        let top = Vec3::new(at.x, at.y + GR_LINE_TOP, at.z);
        let bottom = Vec3::new(at.x, at.y - GR_LINE_DOWN, at.z);
        let hit = match env.solid {
            Some(solid) => solid.ground_at(at.x, at.z),
            None => None,
        };
        if let Some(y) = hit
            && y <= top.y
            && y >= bottom.y
        {
            at.y = GR_WATER_Y;
            return match env.water {
                Some(water) => !water.in_area,
                None => true,
            };
        }
        true
    }

    fn move_se_set(&self, out: &mut Sink<'_>) {
        let mut local_28 = self.moved_by;
        local_28.y = 0.0;
        let span = length_of(local_28) * 3.5;
        let mut uvar2 = span as u32;
        if uvar2 > 100 {
            uvar2 = 100;
        }
        let _ = uvar2;
        out.se(SE_CM_BGN_MECHA_ROTATE);
        let mut local_28 = self.body - self.body_prev;
        local_28.z = 0.0;
        local_28.x = 0.0;
        let span = length_of(local_28);
        let mut uvar2 = span as u32;
        if uvar2 > 100 {
            uvar2 = 100;
        }
        let _ = uvar2;
        out.se(SE_CM_BGN_MECHA_ROPE);
    }

    fn attack_eff_set(&mut self, at: Vec3, param_3: i32, env: &Env<'_>, out: &mut Sink<'_>) {
        let mut local = at;
        if !self.gr_check(&mut local, env) {
            if param_3 <= 1 {
                out.se(SE_CM_BGN_D_HIT_PUNCH_W);
            } else {
                out.se(SE_CM_BGN_D_FALL_WATER);
            }
        } else {
            if param_3 <= 1 {
                out.se(SE_CM_BGN_D_HIT_PUNCH);
            } else {
                out.se(SE_CM_BGN_D_FALL);
            }
            let mut param_3 = param_3;
            if param_3 > 1 {
                param_3 = 1;
            }
            let _ = param_3;
        }
    }

    fn size_set(&mut self) {
        self.head[0].scale = HIO_M0F8;
        self.pelvis[0].scale = HIO_M100;
        self.pelvis[1].scale = HIO_M100;
        let mut fvar2 =
            (1.0 / (HAND_MAX - 2) as f32) * (HIO_M104 - HIO_M108);
        self.left_arm[0].scale = HIO_M104 - fvar2;
        self.right_arm[0].scale = HIO_M104 - fvar2;
        for i in 1..HAND_MAX {
            let i = i as usize;
            self.left_arm[i].scale = HIO_M104 - fvar2 * (i as f32 - 1.0);
            self.right_arm[i].scale = HIO_M104 - fvar2 * (i as f32 - 1.0);
        }
        self.left_arm[HAND_MAX as usize].scale = HIO_M108;
        self.right_arm[HAND_MAX as usize].scale = HIO_M108;
        fvar2 = 0.5 * (HIO_M104 - HIO_M108);
        fvar2 += HIO_M108;
        for i in 0..HAND_MAX {
            let i = i as usize;
            self.left_arm[i].scale +=
                2.0 * (self.hand_dilate[0] * (fvar2 - self.left_arm[i].scale));
            self.right_arm[i].scale +=
                2.0 * (self.hand_dilate[1] * (fvar2 - self.right_arm[i].scale));
        }
        fvar2 = 0.5 * (HIO_M10C - HIO_M110);
        for i in 0..3 {
            self.left_leg[i].scale = HIO_M10C - fvar2 * i as f32;
            self.right_leg[i].scale = HIO_M10C - fvar2 * i as f32;
        }
        self.left_leg[3].scale = HIO_M110;
        self.right_leg[3].scale = HIO_M110;
        fvar2 = (1.0 / (TAIL_MAX - 1) as f32) * (HIO_M114 - HIO_M118);
        for i in 0..TAIL_MAX {
            self.tail[i as usize].scale = HIO_M114 - (fvar2 * i as f32);
        }
        self.right_leg[CHAIN_TAIL_ALIAS].scale = HIO_M114 * 0.7;
        self.tail[TAIL_MAX as usize].scale = HIO_M114 * 0.7;
    }

    fn part_control_0(
        &mut self,
        chain: u8,
        count: i32,
        move_index: usize,
        param_5: f32,
    ) {
        let mut mtx = self.mtx;
        let mut local_e8 = Vec3::ZERO;
        if chain == CHAIN_HEAD && (self.moves[0].rope || self.moves[1].rope) {
            mtx.yrot_s(self.body_angle[1]);
            local_e8 = mtx.pos(Vec3::new(0.0, 0.0, REG0_F6 + 200.0));
        } else if chain == CHAIN_LEFT_ARM && self.moves[3].rope {
            mtx.yrot_s(self.body_angle[1]);
            local_e8 = mtx.pos(Vec3::new(REG0_F8 + 20.0, 0.0, 0.0));
        } else if chain == CHAIN_RIGHT_ARM && self.moves[4].rope {
            mtx.yrot_s(self.body_angle[1]);
            local_e8 = mtx.pos(Vec3::new(-(REG0_F8 + 20.0), 0.0, 0.0));
        }
        let dvar11 = self.water_y - 50.0 + REG0_F6;
        let dvar10 = if !self.moves[move_index].rope {
            HIO_M16C
        } else if self.cut_state != 0 {
            -40.0
        } else {
            HIO_M170
        };
        let sway = self.moves[move_index].sway;
        let phase_x = self.moves[move_index].phase_x;
        let phase_z = self.moves[move_index].phase_z;
        let body_scale = self.scale;
        {
            let parts = self.parts_mut(chain);
            for i in 1..count + 1 {
                let i = i as usize;
                let mut y2 = parts[i].pos.y + dvar10;
                let fvar3 = dvar11 * parts[i - 1].scale;
                if y2 <= fvar3 {
                    y2 = fvar3;
                }
                let temp_f30 = local_e8.x + (parts[i].pos.x - parts[i - 1].pos.x);
                let temp_f26 = y2 - parts[i - 1].pos.y;
                let temp_f25 = local_e8.z + (parts[i].pos.z - parts[i - 1].pos.z);
                let ivar5 =
                    (sway * sin_s(phase_x.wrapping_add((i as i16).wrapping_mul(REG0_S3 + 8000)))) as i32;
                let ivar2 =
                    (sway * cos_s(phase_z.wrapping_add((i as i16).wrapping_mul(REG0_S4 + 9000)))) as i32;
                parts[i - 1].rot[0] =
                    (ivar5 as i16).wrapping_sub(atan2s(temp_f26, temp_f25));
                parts[i - 1].rot[1] = (ivar2 as i16)
                    .wrapping_add(atan2s(temp_f30, spread(temp_f26, temp_f25)));
                mtx.xrot_s(parts[i - 1].rot[0]);
                mtx.yrot_m(parts[i - 1].rot[1]);
                let cstack = mtx.pos(Vec3::new(0.0, 0.0, body_scale * (param_5 * parts[i - 1].scale)));
                parts[i].pos = parts[i - 1].pos + cstack;
            }
        }
        self.mtx = mtx;
    }

    fn part_control_0z(
        &mut self,
        chain: u8,
        count: i32,
        move_index: usize,
        param_5: f32,
        env: &Env<'_>,
        out: &mut Sink<'_>,
    ) {
        let mut mtx = self.mtx;
        mtx.yrot_s(self.body_angle[1]);
        let mut local_128 = Vec3::ZERO;
        if chain == CHAIN_LEFT_ARM {
            local_128 = mtx.pos(Vec3::new(REG0_F8 + 20.0, 0.0, 0.0));
        } else if chain == CHAIN_RIGHT_ARM {
            local_128 = mtx.pos(Vec3::new(-(REG0_F8 + 20.0), 0.0, 0.0));
        } else if chain == CHAIN_TAIL {
            local_128 = mtx.pos(Vec3::new(0.0, 0.0, REG0_F6 + -5.0));
        }
        let dvar10 = self.water_y;
        let dvar9 = HIO_M170;
        let sway = self.moves[move_index].sway;
        let phase_x = self.moves[move_index].phase_x;
        let phase_z = self.moves[move_index].phase_z;
        let body_scale = self.scale;
        let frame_mask = self.frame as i32;
        let mut ripple = self.ripple_timer;
        let mut ripple_pos = self.ripple_pos;
        let mut seed = self.seed;
        let mut splash = false;
        {
            let parts = self.parts_mut(chain);
            for i in 1..count + 1 {
                let i = i as usize;
                let mut dvar11 = parts[i].pos.y + dvar9;
                if dvar11 <= dvar10 {
                    dvar11 = dvar10;
                    if i as i32 == count && (frame_mask & 7) == 0 {
                        let mut local_134 = parts[i].pos;
                        if ripple == 0 && !gr_probe(&env.solid, env.water, &mut local_134) {
                            ripple = 20;
                            ripple_pos = local_134;
                            splash = true;
                        }
                    }
                }
                let f1 = (count - i as i32) as f32;
                let local_11c_x = local_128.x * f1;
                let local_11c_z = local_128.z * f1;
                let x = local_11c_x + (parts[i].pos.x - parts[i - 1].pos.x);
                let y = dvar11 - parts[i - 1].pos.y;
                let z = local_11c_z + (parts[i].pos.z - parts[i - 1].pos.z);
                let ivar2 = (sway
                    * sin_s(phase_x.wrapping_add((i as i16).wrapping_mul(REG0_S3 + 8000))))
                    as i32;
                let ivar4 = (sway
                    * cos_s(phase_z.wrapping_add((i as i16).wrapping_mul(REG0_S4 + 9000))))
                    as i32;
                parts[i - 1].rot[1] = (ivar4 as i16).wrapping_add(atan2s(x, z));
                parts[i - 1].rot[0] = (ivar2 as i16).wrapping_sub(atan2s(y, spread(y, z)));
                mtx.yrot_s(parts[i - 1].rot[1]);
                mtx.xrot_m(parts[i - 1].rot[0]);
                let cstack =
                    mtx.pos(Vec3::new(0.0, 0.0, body_scale * (param_5 * parts[i - 1].scale)));
                parts[i].pos = parts[i - 1].pos + cstack;
            }
        }
        self.mtx = mtx;
        if splash {
            ripple = roll_value(&mut seed) as i16 + 20;
            out.se(SE_CM_BGN_BODY_RIPPLE);
        }
        self.seed = seed;
        self.ripple_timer = ripple;
        self.ripple_pos = ripple_pos;
    }

    fn part_mtx_set(
        &mut self,
        count: i32,
        chain: u8,
        chain_id: i32,
        param_5: i32,
    ) {
        let mut mtx = self.mtx;
        let mut core_centre = self.core_centre;
        let mut core_pos = self.core_pos;
        let mut core_radius = self.core_radius;
        let mut body_radius = self.body_radius;
        let mut body_centre = self.body_centre;
        let mut morph = self.morph;
        let weak_model = self.weak_model();
        let head_loose = self.moves[0].rope || self.moves[1].rope;
        let tail_rope = self.moves[7].rope;
        let hand_scale = self.hand_scale;
        let part_radius = HIO_M0F4;
        let ripple0 = self.moves[0].ripple as i32;
        let ripple1 = self.moves[1].ripple as i32;
        let sway = self.sway;
        for i in 0..count {
            let iu = i as usize;
            let part = self.part(chain, iu);
            let mut flash = part.flash;
            let mut effect = part.effect;
            let scale = part.scale;
            let mut weak = part.weak;
            if self.hit_tag == i16::from(part.tag) {
                flash = CORE_TIMER;
                effect = CORE_EFFECT;
            }
            if flash != 0 {
                flash -= 1;
            }
            let mut rot = part.rot;
            add_calc_angle_s2(&mut rot[2], self.body_angle[1], 4, 0x2000);
            let mut pos = part.pos;
            if pos.y <= self.water_y {
                pos.y = self.water_y;
            }
            mtx.trans(pos, false);
            if chain_id == CHAIN_TAIL as i32 && tail_rope {
                mtx.yrot_m(rot[1]);
                mtx.xrot_m(rot[0]);
                mtx.zrot_m(rot[2].wrapping_neg());
            } else {
                mtx.xrot_m(rot[0]);
                mtx.yrot_m(rot[1]);
                if rot[0] < 0 {
                    mtx.zrot_m(rot[2]);
                } else {
                    mtx.zrot_m(rot[2].wrapping_neg());
                }
            }
            let fvar4 = scale * part_radius;
            let dvar10 = fvar4;
            let fvar5 = if i != count - 1 && (chain_id == 3 || chain_id == 4) {
                (REG0_F0 + 0.005) * (hand_scale[(chain_id - 3) as usize] - HIO_M124) + 1.0
            } else {
                1.0
            };
            mtx.scale(Vec3::new(fvar4, fvar4, dvar10 * fvar5));
            if chain_id == 0 {
                let uvar7 = ((ripple0 + ripple1) as i32) as i16;
                let fvar4 = uvar7 as f32 * (REG0_F14 + 170.0);
                let svar8 = (fvar4 * sin_s((uvar7 as i32 * 0x2100) as i16)) as i16;
                let ivar1 = (fvar4 * sin_s((svar8 as i32 * 0x2300) as i16)) as i32;
                mtx.yrot_m(sway.wrapping_add(HIO_M03C).wrapping_add(ivar1 as i16));
                mtx.zrot_m(HIO_M040.wrapping_add(svar8));
            }
            mtx.trans(Vec3::new(0.0, 0.0, HIO_M168), true);
            if chain_id == 0 {
                mtx.trans(Vec3::new(0.0, HIO_M034, HIO_M038), true);
                mtx.xrot_m(HIO_M03E);
            }
            if chain_id == CHAIN_TAIL as i32 && i == count - 1 {
                weak = weak_model;
            }
            if chain_id == 0 {
                if head_loose || morph.frame as i32 != MORF_FRAME_GATE {
                    morph.update();
                }
            }
            let local_98 = mtx.pos(Vec3::ZERO);
            if chain_id == CHAIN_TAIL as i32 && i == count - 1 {
                core_radius = REG0_F4 + CORE_RADIUS;
                core_centre = local_98;
                core_pos = local_98;
            }
            body_radius = dvar10 * (BODY_RADIUS_BASE + REG0_F0);
            body_centre = local_98;
            if i == param_5 {
                if chain_id == 0 {
                    self.rope_anchor[0] = mtx.pos(Vec3::new(90.0, 180.0, -40.0));
                    self.rope_anchor[1] = mtx.pos(Vec3::new(-90.0, 180.0, -40.0));
                } else {
                    self.rope_anchor[chain_id as usize] =
                        mtx.pos(Vec3::new(0.0, 0.0, HIO_M168));
                }
            }
            if effect != 0 {
                effect -= 1;
            }
            if let Some(slot) = self.parts_mut(chain).get_mut(iu) {
                slot.rot = rot;
                slot.pos = pos;
                if chain_id != CHAIN_TAIL as i32 || i != count - 1 {
                    slot.weak = weak;
                }
                slot.world = local_98;
                slot.centre = local_98;
                slot.radius = dvar10 * (PART_RADIUS_BASE + REG0_F0);
                slot.flash = flash;
                slot.effect = effect;
            }
        }
        self.mtx = mtx;
        self.core_centre = core_centre;
        self.core_pos = core_pos;
        self.core_radius = core_radius;
        self.body_radius = body_radius;
        self.body_centre = body_centre;
        self.morph = morph;
    }

    fn part_control_2(&mut self, chain: u8, count: i32, param_4: f32) {
        let mut mtx = self.mtx;
        let local_9c = self.part(chain, 0).pos;
        {
            let parts = self.parts_mut(chain);
            let mut i = count - 1;
            while i >= 0 {
                let iu = i as usize;
                let x = parts[iu].pos.x - parts[iu + 1].pos.x;
                let y = parts[iu].pos.y - parts[iu + 1].pos.y;
                let z = parts[iu].pos.z - parts[iu + 1].pos.z;
                let ivar4 = atan2s(y, z).wrapping_neg();
                let ivar5 = atan2s(x, spread(y, z));
                mtx.xrot_s(ivar4);
                mtx.yrot_m(ivar5);
                let cstack = mtx.pos(Vec3::new(0.0, 0.0, param_4 * parts[iu].scale));
                parts[iu].pos = parts[iu + 1].pos + cstack;
                i -= 1;
            }
            let local_84 = parts[0].pos - local_9c;
            for i in 0..count as usize {
                parts[i].pos -= local_84;
            }
        }
        self.mtx = mtx;
    }

    fn damage_check(&mut self, env: &Env<'_>, out: &mut Sink<'_>) {
        if self.core_lock != 0 {
            return;
        }
        let Some(shot) = env.blow.pos else {
            return;
        };
        let reach = env.blow.at.reach();
        if env.blow.at == At::LightArrow && reaches(self.core_centre, self.core_radius, shot, reach)
        {
            self.hit_tag = 1;
            self.core_lock = CORE_LOCK;
            self.action = ACTION_DAMAGE;
            self.sub = 0;
            self.left_held = false;
            self.right_held = false;
            if self.health != 0 {
                self.health -= 1;
                out.se(SE_LK_ARROW_HIT);
                if self.health == 0 {
                    out.stop_bgm();
                    self.action = ACTION_HENSIN;
                    self.sub = 0;
                    out.se(SE_CV_BGN_HIT_2);
                    out.se(SE_LK_LAST_HIT);
                    out.se(SE_CM_BGN_M_BRK_ORB);
                } else {
                    out.se(SE_CV_BGN_HIT_1);
                }
            }
        }
        let mut tag: u8 = 0;
        if self.body_at && reaches(self.body_centre, self.body_radius, shot, 0.0) {
            tag = 1;
        }
        if reaches(self.head[0].centre, self.head[0].radius, shot, 0.0) {
            tag = 2;
        }
        for i in 0..PELVIS_PARTS {
            if reaches(
                self.pelvis[i].centre,
                self.pelvis[i].radius,
                shot,
                0.0,
            ) {
                tag = 3;
            }
        }
        for i in 0..HAND_MAX as usize {
            if reaches(
                self.left_arm[i].centre,
                self.left_arm[i].radius,
                shot,
                0.0,
            ) {
                tag = 4;
            }
            if reaches(
                self.right_arm[i].centre,
                self.right_arm[i].radius,
                shot,
                0.0,
            ) {
                tag = 5;
            }
        }
        for i in 0..3 {
            if reaches(
                self.left_leg[i].centre,
                self.left_leg[i].radius,
                shot,
                0.0,
            ) {
                tag = 6;
            }
            if reaches(
                self.right_leg[i].centre,
                self.right_leg[i].radius,
                shot,
                0.0,
            ) {
                tag = 7;
            }
        }
        for i in 0..HAND_MAX as usize {
            if reaches(
                self.tail[i].centre,
                self.tail[i].radius,
                shot,
                0.0,
            ) {
                tag = 8;
            }
        }
        if tag != 0 && self.hit_se == 0 {
            self.hit_se = 10;
            out.se(env.blow.at.at_se());
        }
    }

    fn shape_calc(&mut self, env: &Env<'_>, out: &mut Sink<'_>) {
        let mut mtx = self.mtx;
        let mut local_b0 = self.body_centre;
        local_b0.y = self.body_centre.y + HIO_M0A8;
        let mut fvar3 = self.water_y + BODY_LIFT + REG0_F0;
        if local_b0.y <= fvar3 {
            local_b0.y = fvar3;
        }
        if self.hit_tag == CORE_TAG_ARROW {
            self.arrow_flash = CORE_TIMER;
            self.arrow_effect = CORE_EFFECT;
        }
        mtx.trans(local_b0, false);
        let mut svar8: i16 = 0;
        let mut svar5: i16 = 0;
        if self.shake_timer != 0 {
            self.shake_timer -= 1;
            fvar3 = f32::from(self.shake_timer) * (REG0_F14 + 300.0);
            svar8 = (fvar3 * sin_s((self.frame as i32 * 0x1c00) as i16)) as i16;
            svar5 = (fvar3 * cos_s((self.frame as i32 * 0x1900) as i16)) as i16;
        }
        mtx.yrot_m(self.body_angle[1]);
        mtx.xrot_m(self.body_angle[0].wrapping_add(svar8));
        mtx.zrot_m(self.body_angle[2].wrapping_add(svar5));
        fvar3 = HIO_M0FC * HIO_M0F4;
        let dvar12 = fvar3;
        mtx.scale(Vec3::splat(fvar3));
        mtx.xrot_m(HIO_M004);
        if self.squash > 0.0 {
            svar5 = (self.frame as i32 * (REG0_S4 as i32 + 0x900)) as i16;
            mtx.zrot_m(svar5);
            mtx.scale(Vec3::new(
                self.squash + 1.0,
                1.0 - self.squash,
                self.squash + 1.0,
            ));
            mtx.zrot_m(svar5.wrapping_neg());
        }
        self.mtx = mtx;
        self.body_radius = dvar12 * (BODY_RADIUS_BASE + REG0_F1);
        if self.arrow_effect != 0 {
            self.arrow_effect -= 1;
        }

        let dvar12 = HIO_M11C * HIO_M0F4;
        let dvar11 = HIO_M120 * HIO_M0F4;
        let dvar10 = self.hand_scale[0] * HIO_M0F4;
        let dvar9 = self.hand_scale[1] * HIO_M0F4;
        let dvar13 = HIO_M128 * HIO_M0F4;

        self.head[0].pos = self.mtx.pos(Vec3::new(HIO_M12C, HIO_M130, HIO_M134));
        self.part_control_0(CHAIN_HEAD, 1, 0, dvar12);
        if !self.moves[0].rope || !self.moves[1].rope {
            let mut mtx = self.mtx;
            mtx.yrot_s(self.body_angle[1]);
            let local_8c = Vec3::new(HIO_M054, HIO_M058, HIO_M05C) + self.moves[0].pos;
            self.head[1].pos = mtx.pos(local_8c);
            self.mtx = mtx;
            self.head[1].pos += self.body_centre;
            self.part_control_2(CHAIN_HEAD, 1, dvar12);
        }
        self.part_mtx_set(1, CHAIN_HEAD, 0, 0);
        self.eye = self.head[0].pos;
        self.attention = self.eye;
        self.attention.y += SHOULDER_Y;

        let local_8c = Vec3::new(HIO_M144, HIO_M148 + 20.0, HIO_M14C) * self.scale;
        self.pelvis[0].pos = self.mtx.pos(local_8c);
        self.part_control_0(CHAIN_PELVIS, 1, 2, dvar12);
        if !self.moves[2].rope {
            let mut mtx = self.mtx;
            mtx.yrot_s(self.body_angle[1]);
            let local_8c = Vec3::new(HIO_M060, HIO_M064, HIO_M068) + self.moves[2].pos;
            self.pelvis[1].pos = mtx.pos(local_8c);
            self.mtx = mtx;
            self.pelvis[1].pos += self.body_centre;
            self.part_control_2(CHAIN_PELVIS, 1, dvar12);
        }
        self.part_mtx_set(1, CHAIN_PELVIS, CHAIN_PELVIS as i32, 0);

        let local_8c = Vec3::new(HIO_M138, HIO_M13C, HIO_M140) * self.scale;
        self.left_arm[0].pos = self.mtx.pos(local_8c);
        if !self.moves[3].rope {
            self.part_control_0(CHAIN_LEFT_ARM, HAND_MAX, 3, dvar10);
            if self.left_held {
                self.left_arm[CHAIN_TIP].pos = self.left_arm_tip;
            } else {
                let mut mtx = self.mtx;
                mtx.yrot_s(self.body_angle[1]);
                let local_8c = Vec3::new(HIO_M06C, HIO_M070, HIO_M074) + self.moves[3].pos;
                self.left_arm[CHAIN_TIP].pos = mtx.pos(local_8c);
                self.mtx = mtx;
                self.left_arm[CHAIN_TIP].pos += self.body_centre;
            }
            self.part_control_2(CHAIN_LEFT_ARM, HAND_MAX, dvar10);
        } else {
            self.part_control_0(CHAIN_LEFT_ARM, HAND_MAX, 3, dvar10);
        }
        self.part_mtx_set(
            HAND_MAX,
            CHAIN_LEFT_ARM,
            CHAIN_LEFT_ARM as i32,
            HAND_MAX - 1,
        );

        let local_8c = Vec3::new(-HIO_M138, HIO_M13C, HIO_M140) * self.scale;
        self.right_arm[0].pos = self.mtx.pos(local_8c);
        if !self.moves[4].rope {
            self.part_control_0(CHAIN_RIGHT_ARM, HAND_MAX, 4, dvar9);
            if self.right_held {
                self.right_arm[CHAIN_TIP].pos = self.right_arm_tip;
            } else {
                let mut mtx = self.mtx;
                mtx.yrot_s(self.body_angle[1]);
                let local_8c = Vec3::new(HIO_M078, HIO_M07C, HIO_M080) + self.moves[4].pos;
                self.right_arm[CHAIN_TIP].pos = mtx.pos(local_8c);
                self.mtx = mtx;
                self.right_arm[CHAIN_TIP].pos += self.body_centre;
            }
            self.part_control_2(CHAIN_RIGHT_ARM, HAND_MAX, dvar9);
        } else {
            self.part_control_0(CHAIN_RIGHT_ARM, HAND_MAX, 4, dvar9);
        }
        self.part_mtx_set(
            HAND_MAX,
            CHAIN_RIGHT_ARM,
            CHAIN_RIGHT_ARM as i32,
            HAND_MAX - 1,
        );
        let mut mtx = self.mtx;
        mtx.xrot_s(self.pelvis[0].rot[0]);
        mtx.yrot_m(self.pelvis[0].rot[1]);
        mtx.zrot_m(self.pelvis[0].rot[2].wrapping_neg());
        fvar3 = HIO_M100 * HIO_M0F4;
        mtx.scale(Vec3::splat(fvar3));
        let local_8c = Vec3::new(HIO_M150, HIO_M154, HIO_M158) * self.scale;
        let cstack = mtx.pos(local_8c);
        self.mtx = mtx;
        self.left_leg[0].pos = self.pelvis[0].pos + cstack;
        self.part_control_0(CHAIN_LEFT_LEG, 3, 5, dvar11);
        if !self.moves[5].rope {
            let mut mtx = self.mtx;
            mtx.yrot_s(self.body_angle[1]);
            let local_8c = Vec3::new(HIO_M084, HIO_M088, HIO_M08C) + self.moves[5].pos;
            self.left_leg[3].pos = mtx.pos(local_8c);
            self.mtx = mtx;
            self.left_leg[3].pos += self.body_centre;
            self.part_control_2(CHAIN_LEFT_LEG, 3, dvar11);
        }
        self.part_mtx_set(3, CHAIN_LEFT_LEG, CHAIN_LEFT_LEG as i32, 2);

        let local_8c = Vec3::new(-HIO_M150, HIO_M154, HIO_M158) * self.scale;
        let cstack = self.mtx.pos(local_8c);
        self.right_leg[0].pos = self.pelvis[0].pos + cstack;
        self.part_control_0(CHAIN_RIGHT_LEG, 3, 6, dvar11);
        if !self.moves[6].rope {
            let mut mtx = self.mtx;
            mtx.yrot_s(self.body_angle[1]);
            let local_8c = Vec3::new(HIO_M090, HIO_M094, HIO_M098) + self.moves[6].pos;
            self.right_leg[3].pos = mtx.pos(local_8c);
            self.mtx = mtx;
            self.right_leg[3].pos += self.body_centre;
            self.part_control_2(CHAIN_RIGHT_LEG, 3, dvar11);
        }
        self.part_mtx_set(3, CHAIN_RIGHT_LEG, CHAIN_RIGHT_LEG as i32, 2);

        let local_8c = Vec3::new(HIO_M15C, HIO_M160, HIO_M164) * self.scale;
        let cstack = self.mtx.pos(local_8c);
        self.tail[0].pos = self.pelvis[0].pos + cstack;
        if !self.moves[7].rope {
            self.part_control_0(CHAIN_TAIL, TAIL_MAX, 7, dvar13);
            let mut mtx = self.mtx;
            mtx.yrot_s(self.body_angle[1]);
            let local_8c = Vec3::new(HIO_M09C, HIO_M0A0, HIO_M0A4) + self.moves[7].pos;
            self.tail[TAIL_MAX as usize].pos = mtx.pos(local_8c);
            self.mtx = mtx;
            self.tail[TAIL_MAX as usize].pos += self.body_centre;
            if self.tail[TAIL_MAX as usize].pos.y <= self.water_y {
                self.tail[TAIL_MAX as usize].pos.y = self.water_y;
            }
            self.part_control_2(CHAIN_TAIL, TAIL_MAX, dvar13);
        } else {
            self.part_control_0z(CHAIN_TAIL, TAIL_MAX, 7, dvar13, env, out);
        }
        self.part_mtx_set(TAIL_MAX, CHAIN_TAIL, CHAIN_TAIL as i32, TAIL_MAX - 1);
    }

    fn dance_a(&mut self, out: &mut Sink<'_>) {
        for i in 0..MOVE_COUNT {
            match self.dance_sub {
                0 => {
                    self.dance_sub += 1;
                    self.yaw_step = 0;
                    self.moves[i].target = DANCE_PAUSE_1[i];
                    if self.timers[3] == 0 {
                        self.dance_sub += 1;
                        self.timers[3] = HIO_M0DA;
                    }
                }
                1 => {
                    self.moves[i].target = DANCE_PAUSE_1[i];
                    if self.timers[3] == 0 {
                        self.dance_sub += 1;
                        self.timers[3] = HIO_M0DA;
                    }
                }
                2 => {
                    self.moves[i].target = DANCE_PAUSE_2[i];
                    if self.timers[3] == 0 {
                        self.dance_sub = 1;
                        self.timers[3] = HIO_M0DA;
                    }
                }
                _ => {}
            }
        }
        let uvar2 = self.yaw_bob;
        self.yaw_bob = self.yaw_bob.wrapping_add(self.yaw_step);
        if (uvar2 > 0 && self.yaw_bob <= self.yaw_step)
            || ((uvar2 as u16) > 0x8000
                && (self.yaw_bob as u16) <= (self.yaw_step as i32 + 0x8000) as u16)
        {
            out.se(SE_CM_BGN_D_KAZEKIRI);
        }
        let target = match self.health {
            3 => HIO_M0DC,
            2 => HIO_M0DE,
            _ => HIO_M0E0,
        };
        add_calc_angle_s2(&mut self.yaw_step, target, 1, DANCE_A_STEP);
    }

    fn punch_lr(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        let mut bvar2 = false;
        add_calc0(&mut self.body.y, 0.5, 20.0);
        let mut bvar3 = false;
        let mut uvar11: i32 = 0;
        let mut local_90 = env.player_pos;
        if self.moves[0].rope && self.moves[1].rope {
            let mut mtx = self.mtx;
            mtx.yrot_s(self.shape_angle[1]);
            let local_6c = Vec3::new(
                self.rnd_fx(1000.0),
                0.0,
                self.rnd_f(1000.0) + 1000.0,
            );
            local_90 = mtx.pos(local_6c);
            local_90 += actor.pos;
            local_90.y = 0.0;
        }
        loop {
            match self.sub {
                1 => {
                    bvar3 = true;
                    uvar11 = 2;
                    self.pos_target = 130.0;
                    if self.timers[0] == 0x14 {
                        self.right_arm_tip = self.right_arm[CHAIN_TIP].pos;
                        let local_84 = local_90 - self.right_arm_tip;
                        let mut mtx = self.mtx;
                        mtx.yrot_s(atan2s(local_84.x, local_84.z));
                        mtx.xrot_m(
                            atan2s(local_84.y, spread(local_84.x, local_84.z)).wrapping_neg(),
                        );
                        self.right_arm_step = mtx.pos(Vec3::new(0.0, 0.0, REG0_F7 + 200.0));
                        self.mtx = mtx;
                    }
                    if self.timers[0] != 0 {
                        break;
                    }
                    self.right_held = true;
                    self.sub += 1;
                    self.timers[0] = REG0_S1 + 0x32;
                    out.se(SE_CM_BGN_D_PUNCH_FIRE);
                    self.sub = 2;
                }
                2 => {
                    uvar11 = 2;
                    self.pos_target = 200.0;
                    self.pos_speed = 200.0;
                    self.right_arm[HAND_MAX as usize - 1].at = true;
                    self.right_arm_tip += self.right_arm_step;
                    self.moves[4].slack = 100.0;
                    if self.timers[0] == 0
                        || self.pelvis_probe() < self.water_y + 30.0
                    {
                        self.right_held = false;
                        self.moves[4].sway = REG0_F15 + 5000.0;
                        bvar2 = true;
                        self.moves[4].target =
                            to_i16x3(self.grab_left());
                        self.attack_eff_set(
                            self.right_arm[HAND_MAX as usize - 1].pos,
                            1,
                            env,
                            out,
                        );
                    }
                    break;
                }
                3 => {
                    bvar3 = true;
                    uvar11 = 1;
                    self.pos_target = 200.0;
                    self.pos_speed = 200.0;
                    if self.timers[0] == 0x14 {
                        self.left_arm_tip = self.left_arm[HAND_MAX as usize - 1].pos;
                        let local_84 = local_90 - self.left_arm_tip;
                        let mut mtx = self.mtx;
                        mtx.yrot_s(atan2s(local_84.x, local_84.z));
                        mtx.xrot_m(
                            atan2s(local_84.y, spread(local_84.x, local_84.z)).wrapping_neg(),
                        );
                        self.left_arm_step = mtx.pos(Vec3::new(0.0, 0.0, REG0_F7 + 200.0));
                        self.mtx = mtx;
                    }
                    if self.timers[0] != 0 {
                        break;
                    }
                    self.sub += 1;
                    self.timers[0] = REG0_S1 + 0x32;
                    self.left_held = true;
                    out.se(SE_CM_BGN_D_PUNCH_FIRE);
                    self.sub = 4;
                }
                4 => {
                    uvar11 = 1;
                    self.left_arm[HAND_MAX as usize - 1].at = true;
                    self.left_arm_tip += self.left_arm_step;
                    self.pos_target = 200.0;
                    if self.timers[0] == 0 || self.pelvis_probe() < self.water_y + 30.0 {
                        self.moves[3].sway = REG0_F15 + 5000.0;
                        self.left_held = false;
                        bvar2 = true;
                        self.moves[3].target = to_i16x3(self.grab_right());
                        self.attack_eff_set(
                            self.left_arm[HAND_MAX as usize - 1].pos,
                            0,
                            env,
                            out,
                        );
                    }
                    break;
                }
                5 => {
                    bvar3 = true;
                    uvar11 = 3;
                    self.pos_target = 50.0;
                    if self.timers[0] == 0x14 {
                        self.left_arm_tip = self.left_arm[HAND_MAX as usize - 1].pos;
                        self.right_arm_tip = self.right_arm[HAND_MAX as usize - 1].pos;
                        let local_84 = local_90 - self.left_arm_tip;
                        let mut mtx = self.mtx;
                        mtx.yrot_s(atan2s(local_84.x, local_84.z));
                        mtx.xrot_m(
                            atan2s(local_84.y, spread(local_84.x, local_84.z)).wrapping_neg(),
                        );
                        self.left_arm_step = mtx.pos(Vec3::new(0.0, 0.0, REG0_F7 + 200.0));
                        let local_84 = local_90 - self.right_arm_tip;
                        mtx.yrot_s(atan2s(local_84.x, local_84.z));
                        mtx.xrot_m(
                            atan2s(local_84.y, spread(local_84.x, local_84.z)).wrapping_neg(),
                        );
                        self.right_arm_step = mtx.pos(Vec3::new(0.0, 0.0, REG0_F7 + 200.0));
                        self.mtx = mtx;
                    }
                    if self.timers[0] != 0 {
                        break;
                    }
                    self.left_held = true;
                    self.right_held = true;
                    self.sub += 1;
                    self.timers[0] = REG0_S1 + 0x32;
                    out.se(SE_CM_BGN_D_PUNCH_FIRE);
                    self.sub = 6;
                }
                6 => {
                    uvar11 = 3;
                    self.right_arm[HAND_MAX as usize - 1].at = true;
                    self.left_arm[HAND_MAX as usize - 1].at = true;
                    self.left_arm_tip += self.left_arm_step;
                    self.right_arm_tip += self.right_arm_step;
                    self.pos_target = 200.0;
                    self.pos_speed = 200.0;
                    if self.timers[0] == 0 || self.pelvis_probe() < self.water_y + 30.0 {
                        self.moves[3].sway = REG0_F15 + 5000.0;
                        self.moves[4].sway = REG0_F15 + 5000.0;
                        self.left_held = false;
                        self.right_held = false;
                        self.moves[3].target = to_i16x3(self.grab_left());
                        self.moves[4].target = to_i16x3(self.grab_right());
                        self.attack_eff_set(
                            self.right_arm[HAND_MAX as usize - 1].pos,
                            1,
                            env,
                            out,
                        );
                        self.attack_eff_set(
                            self.left_arm[HAND_MAX as usize - 1].pos,
                            0,
                            env,
                            out,
                        );
                        bvar2 = true;
                    }
                    break;
                }
                _ => break,
            }
        }
        if bvar2 {
            self.action = ACTION_NONE;
            self.sub = 0;
        }
        if uvar11 != 0 {
            if uvar11 <= 2 {
                let slot = (uvar11 - 1) as usize;
                add_calc2(
                    &mut self.hand_dilate[slot],
                    REG0_F14 + 1.0,
                    0.2,
                    0.05,
                );
            } else {
                for i in 0..2 {
                    add_calc2(&mut self.hand_dilate[i], 1.0 + REG0_F14, 0.2, 0.05);
                }
            }
        }
        if bvar3 && (!self.moves[0].rope || !self.moves[1].rope) {
            let svar7 = Self::player_yaw(env, actor.pos);
            add_calc_angle_s2(&mut self.shape_angle[1], svar7, 10, 0x400);
        }
        for i in 0..MOVE_COUNT {
            let table = match self.sub {
                1 => PUNCH_R1_D[i],
                2 => PUNCH_R2_D[i],
                3 => PUNCH_L1_D[i],
                4 => PUNCH_L2_D[i],
                5 => {
                    if (self.timers[0] & 0x10) != 0 {
                        PUNCH_LR1_D[i]
                    } else {
                        PUNCH_LR12_D[i]
                    }
                }
                6 => PUNCH_LR2_D[i],
                _ => continue,
            };
            self.moves[i].target = table;
        }
    }

    fn start(&mut self, actor: &mut Actor, env: &Env<'_>) {
        let target = Self::player_yaw(env, actor.pos);
        add_calc_angle_s2(&mut self.shape_angle[1], target, 10, 0x400);
        for i in 0..MOVE_COUNT {
            self.moves[i].target = START_PAUSE[i];
        }
        if self.timers[0] == 0 {
            self.dance = 0;
            self.move_count = 1;
            self.action = ACTION_NONE;
        }
    }

    fn body_attack(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        let mut bvar2 = false;
        let mut local_50 = self.dest - actor.pos;
        local_50.y = 0.0;
        let fvar10 = length_of(local_50);
        match self.sub {
            0 => {
                self.dest_speed = 0.0;
                self.dest = env.player_pos;
                self.dest.y = 0.0;
                if !self.moves[0].rope || !self.moves[1].rope {
                    self.dest_yaw = Self::player_yaw(env, actor.pos);
                }
                self.sub = 1;
            }
            1 => {
                bvar2 = true;
                if fvar10 < 200.0 {
                    self.sub = 2;
                    self.timers[0] = 0x1e;
                }
            }
            2 => {
                bvar2 = true;
                self.dest.y = 500.0;
                if self.timers[0] == 0 {
                    self.sub = 3;
                    self.speed.y = 0.0;
                    self.shake_timer = REG0_S0 + 0x14;
                    for i in 0..MOVE_COUNT {
                        self.moves[i].tension = REG0_S7 + 0x1e;
                        self.moves[i].sway = REG0_F15 + 5000.0;
                    }
                }
            }
            3 => {
                self.body.y += self.speed.y;
                self.speed.y -= REG0_F4 + 10.0;
                self.body_at = true;
                self.pelvis[0].at = true;
                if self.body.y <= -1000.0 {
                    self.body.y = -1000.0;
                    self.speed.y *= -(REG0_F19 + 0.4);
                    for i in 0..MOVE_COUNT {
                        self.moves[i].tension = REG0_S7 + 0x28;
                        self.moves[i].sway = REG0_F15 + 5000.0;
                    }
                    self.shake_timer = REG0_S0 + 0x14;
                    self.timers[0] = 0x46;
                    self.sub = 4;
                    self.attack_eff_set(self.body_centre, 2, env, out);
                }
            }
            4 => {
                self.body.y += self.speed.y;
                self.speed.y -= REG0_F4 + 10.0;
                if self.body.y <= -1000.0 {
                    self.body.y = -1000.0;
                    self.speed.y = 0.0;
                }
                if self.timers[0] > 10 {
                    add_calc2(&mut self.chain_speed, REG0_F9 + 250.0, 1.0, REG0_F10 + 50.0);
                    for i in 0..MOVE_COUNT {
                        add_calc2(
                            &mut self.moves[i].wave,
                            250.0 + REG0_F9,
                            1.0,
                            50.0 + REG0_F10,
                        );
                    }
                }
                if self.timers[0] == 0 {
                    self.action = ACTION_NONE;
                    self.sub = 0;
                    for i in 0..MOVE_COUNT {
                        self.moves[i].tension = REG0_S7 + 0x32;
                        self.moves[i].sway = REG0_F15 + 5000.0;
                    }
                }
            }
            _ => {}
        }
        if bvar2 {
            add_calc2(&mut actor.pos.x, self.dest.x, 0.1, self.dest_speed);
            add_calc2(&mut actor.pos.z, self.dest.z, 0.1, self.dest_speed);
            add_calc2(&mut self.dest_speed, REG0_F19 + 30.0, 1.0, 0.5);
            add_calc2(&mut self.body.y, self.dest.y, 0.1, 50.0);
            add_calc_angle_s2(&mut self.shape_angle[1], self.dest_yaw, 10, 0x400);
        }
    }

    fn tail_attack(&mut self, actor: &mut Actor, env: &Env<'_>) {
        match self.sub {
            0 => {
                self.timers[0] = 100;
                self.sub = 1;
                self.dest_yaw = 0;
                self.dest_speed = 0.0;
            }
            1 => {
                self.shape_angle[1] = self.shape_angle[1].wrapping_add(self.dest_yaw);
                if self.timers[0] > 0x28 {
                    add_calc_angle_s2(&mut self.dest_yaw, 0x800, 1, 0x80);
                    add_calc2(&mut actor.pos.x, env.player_pos.x, 0.1, self.dest_speed);
                    add_calc2(&mut actor.pos.z, env.player_pos.z, 0.1, self.dest_speed);
                    add_calc2(&mut self.dest_speed, REG0_F19 + 15.0, 1.0, 0.2);
                    for i in 0..TAIL_MAX as usize {
                        self.tail[i].at = true;
                    }
                    self.moves[7].target[2] = (REG0_F3 + -600.0) as i16;
                    add_calc2(&mut self.body.y, -550.0, 0.1, 20.0);
                } else {
                    add_calc_angle_s2(&mut self.dest_yaw, 0, 1, 0x20);
                    add_calc2(&mut self.body.y, 0.0, 0.1, 5.0);
                    self.moves[7].target[2] = 0;
                }
                if self.timers[0] == 0 {
                    self.action = ACTION_NONE;
                    self.sub = 0;
                }
            }
            _ => {}
        }
    }

    fn damage(&mut self, actor: &mut Actor) {
        self.morph.update();
        self.morph.update();
        match self.sub {
            0 => {
                self.timers[0] = 0;
                self.sub = 1;
                self.speed.y = REG0_F8 + 100.0;
                self.shake_timer = REG0_S0 + 0x32;
                for i in 0..MOVE_COUNT {
                    self.moves[i].tension = REG0_S7 + 0x46;
                    self.moves[i].sway = REG0_F15 + 8000.0;
                }
            }
            1 => {
                if self.body.y <= 0.0 {
                    self.speed.y = REG0_F9 + 30.0;
                    self.timers[0] = REG0_S5 + 0x28;
                    self.sub = 2;
                }
            }
            2 => {
                if self.timers[0] == 0 {
                    if self.moves[7].rope {
                        self.action = ACTION_HEAD_RECOVER;
                    } else {
                        self.action = ACTION_NONE;
                    }
                    self.sub = 0;
                }
            }
            _ => {}
        }
        self.body.y += self.speed.y;
        self.speed.y -= REG0_F4 + 10.0;
        if self.body.y <= 0.0 {
            self.body.y = 0.0;
            self.speed.y = 0.0;
        }
        if self.body.y > 100.0 {
            add_calc2(&mut self.chain_speed, REG0_F9 + 250.0, 1.0, REG0_F10 + 50.0);
            for i in 0..MOVE_COUNT {
                add_calc2(
                    &mut self.moves[i].wave,
                    250.0 + REG0_F9,
                    1.0,
                    50.0 + REG0_F10,
                );
            }
        }
        let _ = actor;
    }

    fn head_recover(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        match self.sub {
            0 => {
                add_calc2(
                    &mut self.body.y,
                    4000.0,
                    0.2,
                    (REG0_F10 + 100.0) * self.cut_param,
                );
                add_calc2(&mut self.cut_param, 1.0, 1.0, 0.02);
                self.shape_angle[1] = self.shape_angle[1].wrapping_add(0x200);
                if self.body.y >= 3990.0 {
                    self.sub = 1;
                    self.timers[0] = REG0_S5 + 0x28;
                    self.moves[1].rope = false;
                    self.moves[0].rope = false;
                    self.moves[1].join = 2;
                    self.moves[0].join = 2;
                    self.moves[4].rope = false;
                    self.moves[3].rope = false;
                    self.moves[4].join = 2;
                    self.moves[3].join = 2;
                    self.moves[7].rope = false;
                    self.moves[7].join = 2;
                    out.se(SE_CM_BGN_D_ROPE_RESET);
                }
            }
            1 => {
                if self.timers[0] == 0 {
                    self.shape_angle[1] = self.shape_angle[1].wrapping_add(-0x400);
                    add_calc2(&mut self.body.y, 0.0, 0.05, REG0_F10 + 200.0);
                    if self.body.y < 5.0 {
                        self.action = ACTION_NONE;
                        self.sub = 0;
                    }
                }
            }
            _ => {}
        }
        let _ = (actor, env);
    }

    fn hensin(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        add_calc2(&mut actor.pos.x, 0.0, 0.05, 50.0);
        add_calc2(&mut actor.pos.z, 0.0, 0.05, 50.0);
        match self.sub {
            0 => {
                self.sub = 1;
                for i in 0..MOVE_COUNT {
                    self.moves[i].rope = true;
                }
                self.cut_state = 1;
                self.sub = 1;
            }
            1 | 2 | 3 | 4 => {
                self.body.y += self.speed.y;
                self.speed.y -= REG0_F4 + 10.0;
                if self.body.y <= REG0_F2 + -1000.0 {
                    self.body.y = REG0_F2 + -1000.0;
                    self.shake_timer = REG0_S0 + 0x14;
                    for i in 0..MOVE_COUNT {
                        self.moves[i].tension = REG0_S7 + 60;
                        self.moves[i].sway = REG0_F15 + 6000.0;
                    }
                    if self.sub == 1 {
                        self.speed.y = REG0_F3 + 220.0;
                        self.shake = REG0_F18 + 20.0;
                        self.attack_eff_set(self.head[0].pos, 0, env, out);
                    } else if self.sub == 2 {
                        self.speed.y = REG0_F3 + 190.0;
                        self.shake = REG0_F18 + 15.0;
                        self.attack_eff_set(self.head[0].pos, 1, env, out);
                    } else if self.sub == 3 {
                        self.speed.y = REG0_F3 + 130.0;
                        self.shake = REG0_F18 + 10.0;
                    } else {
                        self.speed.y = 0.0;
                        out.bgm(BGM_BGN_TARABA_IN);
                    }
                    self.sub += 1;
                }
            }
            5 => {
                if self.cut_count >= (REG0_S3 + 150) as i16 {
                    add_calc2(&mut self.fog, 1.0, 1.0, REG0_F15 + 0.005);
                }
                add_calc2(&mut self.chain_speed, REG0_F9 + 250.0, 1.0, REG0_F10 + 50.0);
                if self.cut_count == 280 {
                    self.sub = 10;
                    self.timers[0] = REG0_S6 + 300;
                    self.timers[1] = REG0_S7 + 0x96;
                    self.chest_pot = true;
                }
            }
            10 => {
                add_calc2(&mut self.fog, 1.0, 1.0, REG0_F15 + 0.005);
                if self.chest_pot {
                    if self.timers[1] != 0 {
                        self.chest_pot = true;
                    } else {
                        self.chest_pot = false;
                    }
                }
                add_calc2(&mut self.body.y, REG0_F9 + 1000.0, 0.05, self.speed.y);
                add_calc2(&mut self.speed.y, 50.0, 1.0, 0.2);
                if self.timers[0] <= 100 {
                    add_calc0(&mut self.scale, 1.0, 0.02);
                    if self.timers[0] == 100 {
                        out.se(SE_CM_BGN_D_TO_T_1);
                    }
                    if self.timers[0] <= 1 {
                        add_calc2(&mut self.squash, 0.22, 0.1, 0.0055);
                        if self.timers[0] == 1 {
                            out.se(SE_CM_BGN_D_TO_T_2);
                        }
                        if self.squash > 0.21 {
                            self.hio_phase += 1;
                            self.cut_state += 1;
                            self.cut_count = 0;
                            out.se(SE_CM_BGN_METAM_EXPLODE);
                        } else {
                            add_calc_angle_s2(&mut self.blure, 0xb4, 1, 4);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn himo_control(&mut self, line: usize, env: &Env<'_>, out: &mut Sink<'_>) {
        let mut mtx = self.mtx;
        let local_ac = Vec3::new(0.0, 0.0, (REG0_F12 + 15.0) * self.scale);
        let move_index = line;
        let sway = self.moves[move_index].rope_sway;
        let phase_x = self.moves[move_index].phase_x;
        let phase_z = self.moves[move_index].phase_z;
        let mut j: i16 = 0;
        let mut k: i16 = 0;
        let frame_mask = self.frame as i32;
        let mut ripple = self.ripple_timer;
        let mut ripple_pos = self.ripple_pos;
        let mut seed = self.seed;
        let mut himo = self.moves[move_index].himo;
        himo[0] = self.rope_anchor[line];
        self.blue_pos[line][0] = himo[0];
        for i in 1..ROPE_POINTS {
            let x = himo[i].x - himo[i - 1].x;
            let z = himo[i].z - himo[i - 1].z;
            let mut y2 = himo[i].y - 40.0 + REG0_F13;
            let mut bvar9 = false;
            if y2 < 0.0 {
                y2 = 0.0;
                bvar9 = true;
            }
            let y = y2 - himo[i - 1].y;
            if sway > 0.01 {
                j = (sway * sin_s(phase_x.wrapping_add((i as i16).wrapping_mul(REG0_S3 + 2000))))
                    as i16;
                k = (sway * cos_s(phase_z.wrapping_add((i as i16).wrapping_mul(REG0_S4 + 0x9c4))))
                    as i16;
            }
            mtx.xrot_s(j.wrapping_sub(atan2s(y, z)));
            mtx.yrot_m(k.wrapping_add(atan2s(x, spread(y, z))));
            let local_b8 = mtx.pos(local_ac);
            himo[i] = himo[i - 1] + local_b8;
            self.blue_pos[line][i] = himo[i];
            if bvar9 && (frame_mask & 7) == 0 && ((frame_mask >> 3 & 0xf) == (i as i32 & 0xf)) {
                let mut local_c4 = himo[i];
                if ripple == 0 && !gr_probe(&env.solid, env.water, &mut local_c4) {
                    ripple = roll_value(&mut seed) as i16 + 20;
                    ripple_pos = local_c4;
                    out.se(SE_CM_BGN_STRING_RIPPLE);
                }
            }
        }
        self.moves[move_index].himo = himo;
        self.mtx = mtx;
        self.seed = seed;
        self.ripple_timer = ripple;
        self.ripple_pos = ripple_pos;
    }

    fn last_himo_control(&mut self) {
        let mut mtx = self.mtx;
        self.cs_himo[0] = Vec3::new(
            -15.0,
            REG8_F4 + 4441.46,
            REG8_F3 + 375.17,
        );
        self.cs_pos[0] = self.cs_himo[0];
        self.cs_size[0] = 10;
        if self.defeat_state == 1 {
            self.defeat_state += 1;
            let head = self.cs_himo[0];
            for i in 1..ROPE_POINTS {
                self.cs_himo[i] = head;
            }
            self.defeat_sway = REG0_F19 + 5000.0;
        } else {
            add_calc0(&mut self.defeat_sway, 1.0, REG0_F18 + 50.0);
        }
        let local_a8 = Vec3::new(0.0, 0.0, REG0_F12 + 45.0);
        let mut k: i16 = 0;
        let mut j: i16 = 0;
        let sway = self.defeat_sway;
        let frame = self.frame as i32;
        for i in 1..ROPE_POINTS {
            let x = self.cs_himo[i].x - self.cs_himo[i - 1].x;
            let y = self.cs_himo[i].y - self.cs_himo[i - 1].y - 35.0 + REG0_F13;
            let z = self.cs_himo[i].z - self.cs_himo[i - 1].z;
            if sway > 0.01 {
                k = (sway * sin_s(((frame * 2000) as i16).wrapping_add((i as i16).wrapping_mul(REG0_S3 + 2000)))) as i16;
                j = (sway * cos_s(((frame * 0x8fc) as i16).wrapping_add((i as i16).wrapping_mul(REG0_S4 + 0x9c4)))) as i16;
            }
            mtx.xrot_s(k.wrapping_sub(atan2s(y, z)));
            mtx.yrot_m(j.wrapping_add(atan2s(x, spread(y, z))));
            let local_b4 = mtx.pos(local_a8);
            self.cs_himo[i] = self.cs_himo[i - 1] + local_b4;
            self.cs_pos[i] = self.cs_himo[i];
            self.cs_size[i] = 10;
        }
        self.mtx = mtx;
    }

    fn action_s(&mut self, line: usize, env: &Env<'_>, out: &mut Sink<'_>) {
        let move_index = line;
        let param_3 = line as i32;
        if self.moves[move_index].tension != 0 {
            self.moves[move_index].tension -= 1;
        }
        let mut target = 0.0;
        if HIO_M025 == 0 && self.body.y > REG0_F12 + -900.0 {
            target = REG0_F13
                + (f32::from(self.moves[move_index].tension) * (REG0_F2 + 20.0) + 300.0);
        }
        add_calc2(
            &mut self.moves[move_index].sway,
            target,
            0.5,
            REG0_F8 + 200.0,
        );
        self.moves[move_index].phase_x = self.moves[move_index].phase_x.wrapping_add(
            ((param_3 * 3 + 2600) + REG0_S5 as i32) as i16,
        );
        self.moves[move_index].phase_z = self.moves[move_index].phase_z.wrapping_add(
            ((param_3 * 3 + 2300) + REG0_S6 as i32) as i16,
        );
        if self.moves[move_index].timer != 0 {
            self.moves[move_index].timer -= 1;
        }
        if HIO_M025 != 0 {
            self.moves[move_index].rope = false;
            let table: [f32; 3] = match param_3 {
                0 => [HIO_M0AA, HIO_M0AC, HIO_M0AE],
                2 => [HIO_M0B0, HIO_M0B2, HIO_M0B4],
                3 => [HIO_M0B6, HIO_M0B8, HIO_M0BA],
                4 => [HIO_M0BC, HIO_M0BE, HIO_M0C0],
                5 => [HIO_M0C2, HIO_M0C4, HIO_M0C6],
                6 => [HIO_M0C8, HIO_M0CA, HIO_M0CC],
                7 => [HIO_M0CE, HIO_M0D0, HIO_M0D2],
                _ => [0.0, 0.0, 0.0],
            };
            self.moves[move_index].target = [table[0] as i16, table[1] as i16, table[2] as i16];
        }
        if !self.moves[move_index].rope {
            add_calc2(
                &mut self.moves[move_index].pos.x,
                f32::from(self.moves[move_index].target[0]),
                0.5,
                self.pos_speed,
            );
            add_calc2(
                &mut self.moves[move_index].pos.y,
                f32::from(self.moves[move_index].target[1]),
                0.5,
                self.pos_speed,
            );
            add_calc2(
                &mut self.moves[move_index].pos.z,
                f32::from(self.moves[move_index].target[2]),
                0.5,
                self.pos_speed,
            );
        }
        let mut local_64 = self.rope_anchor[line];
        if self.moves[move_index].rope {
            self.moves[move_index].himo[0] = local_64;
            self.blue_pos[line][0] = local_64;
            self.himo_control(line, env, out);
            add_calc0(
                &mut self.moves[move_index].rope_sway,
                1.0,
                REG0_F5 + 100.0,
            );
        } else {
            let mut local_70 = Vec3::new(0.0, 0.0, 0.0);
            add_calc0(&mut self.moves[move_index].wave, 1.0, 25.0);
            if self.moves[move_index].ripple != 0 {
                self.moves[move_index].ripple -= 1;
            }
            let mut mtx = self.mtx;
            mtx.yrot_s((param_3 * (REG0_S2 as i32 + 13000)) as i16);
            for i in 0..ROPE_POINTS {
                if self.moves[move_index].ripple != 0 {
                    let fvar3 = (REG0_F12 + 10.0)
                        * (f32::from(self.moves[move_index].ripple)
                            * (WIND_STEP_F * i as f32).sin());
                    local_70.x = fvar3 * cos_s((self.frame as i32 * (REG0_S3 as i32 + 0x5800)) as i16);
                    local_70.z = fvar3 * cos_s((self.frame as i32 * (REG0_S5 as i32 + 0x5200)) as i16);
                } else {
                    let mut fvar3 =
                        self.moves[move_index].wave * (WIND_STEP_F * i as f32).sin();
                    fvar3 *= WIND_TAPER * (59 - i) as f32;
                    local_70.x = fvar3
                        * cos_s(
                            (self.frame as i32 * (REG0_S3 as i32 + 200)
                                + i as i32 * (REG0_S4 as i32 + 2000)) as i16,
                        );
                    local_70.z = fvar3
                        * cos_s(
                            (self.frame as i32 * (REG0_S5 as i32 + 0xfa)
                                + i as i32 * (REG0_S6 as i32 + 2000)) as i16,
                        );
                }
                let cstack = mtx.pos(local_70);
                self.moves[move_index].himo[i] = local_64;
                self.blue_pos[line][i] = local_64 + cstack;
                self.blue_size[line][i] = (REG0_S2 + 6) as u8;
                local_64.y += 50.0;
            }
            self.mtx = mtx;
        }
    }

    fn action_main(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        let mut local_9c = self.body_centre;
        add_calc0(&mut self.chain_speed, 1.0, 25.0);
        for i in 0..ROPE_POINTS {
            let mut dvar9 = self.chain_speed * (WIND_STEP_F * i as f32).sin();
            dvar9 *= WIND_TAPER * (59 - i) as f32;
            let local_90 = Vec3::new(
                dvar9 * sin_s((self.frame as i32 * (REG0_S3 as i32 + 300)
                    + i as i32 * (REG0_S4 as i32 + 2000)) as i16),
                0.0,
                dvar9 * sin_s((self.frame as i32 * (REG0_S5 as i32 + 0xfa)
                    + i as i32 * (REG0_S6 as i32 + 2000)) as i16),
            );
            self.red_pos[i] = local_9c + local_90;
            self.red_size[i] = (REG0_S3 + 10) as u8;
            local_9c.y += 50.0;
        }
        if HIO_M025 == 0 {
            match self.action {
                0 => {
                    self.dance_0(actor, env, out);
                    self.move_se_set(out);
                }
                1 => {
                    self.punch_lr(actor, env, out);
                    self.move_se_set(out);
                }
                2 => {
                    self.body_attack(actor, env, out);
                    self.move_se_set(out);
                }
                3 => {
                    self.tail_attack(actor, env);
                    self.move_se_set(out);
                }
                4 => {
                    self.head_recover(actor, env, out);
                    self.move_se_set(out);
                }
                5 => self.damage(actor),
                6 => self.hensin(actor, env, out),
                7 => self.start(actor, env),
                10 => {
                    self.move_count = 1;
                    self.scale = 1.0;
                }
                _ => {}
            }
            if self.action > 0 {
                add_calc_angle_s2(&mut self.yaw_bob, 0, 10, 0x200);
            }
        }
        for i in 0..MOVE_COUNT {
            self.action_s(i, env, out);
        }
        add_calc2(&mut self.pos_speed, self.pos_target, 1.0, 10.0);
        self.pos_target = HIO_M0D4;
    }

    fn move_step(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        self.body_prev = self.body;
        self.moved_by = actor.pos - self.home;
        if self.body_draw != 0 {
            self.action = ACTION_DONE;
            actor.pos = Vec3::new(0.0, 30000.0, 0.0);
            self.attn_flags = 0;
            self.scale = REG0_F25 + 1.0;
            self.squash = REG0_F20;
            self.fog = REG0_F21;
        } else {
            if self.action == ACTION_DONE {
                actor.pos = self.home;
                self.action = ACTION_NONE;
                self.sub = 0;
                self.health = MAX_HEALTH;
            }
            self.attn_flags = ATTN_LOCKON_BATTLE;
            self.attn_dist = ATTN_DIST_BATTLE;
            self.body_centre = actor.pos + self.body;
            self.body_centre.y += HIO_M04C;
            let floor = self.water_y + BODY_LIFT + REG0_F0;
            if self.body_centre.y <= floor {
                self.body_centre.y = floor;
            }
            let mut local_110 = self.shape_angle;
            if HIO_M025 != 0 {
                local_110[1] = HIO_M042;
            }
            self.water_y = REG0_F16 + -40.0 + 30.0 + 100.0;
            let mut svar1: i16 = 0;
            if self.moves[0].rope {
                svar1 = 8000;
            }
            if self.moves[1].rope {
                svar1 -= 8000;
            }
            add_calc_angle_s2(&mut self.sway, svar1, 8, 0x400);
            if !self.moves[2].rope {
                local_110[0] = local_110[0]
                    .wrapping_add((self.moves[2].pos.y * (REG0_F11 + 8.0)) as i32 as i16);
            }
            if self.moves[3].rope {
                local_110[2] = local_110[2].wrapping_sub((REG0_S0 + 3000) as i16);
            } else {
                local_110[2] = local_110[2]
                    .wrapping_add((self.moves[3].pos.y * (REG0_F12 + 5.0)) as i32 as i16);
                local_110[1] = local_110[1].wrapping_add(
                    (self.moves[3].pos.z * -(REG0_F13 + 5.0)) as i32 as i16,
                );
            }
            if self.moves[4].rope {
                local_110[2] = local_110[2].wrapping_add((REG0_S0 + 3000) as i16);
            } else {
                local_110[2] = local_110[2]
                    .wrapping_add((self.moves[4].pos.y * -(REG0_F12 + 5.0)) as i32 as i16);
                local_110[1] = local_110[1].wrapping_add(
                    (self.moves[4].pos.z * (REG0_F13 + 5.0)) as i32 as i16,
                );
            }
            add_calc_angle_s2(&mut self.body_angle[0], local_110[0], 4, 0x200);
            add_calc_angle_s2(&mut self.yaw_tick, local_110[1], 4, 0x1000);
            add_calc_angle_s2(&mut self.body_angle[2], local_110[2], 4, 0x200);
            self.body_angle[1] = self.yaw_tick.wrapping_add(self.yaw_bob);
            if HIO_M029 != 0 {
                self.moves[0].rope = true;
            }
            if HIO_M02A != 0 {
                self.moves[1].rope = true;
            }
            if HIO_M02B != 0 {
                self.moves[2].rope = true;
            }
            if HIO_M02C != 0 {
                self.moves[3].rope = true;
            }
            if HIO_M02D != 0 {
                self.moves[4].rope = true;
            }
            if HIO_M02E != 0 {
                self.moves[5].rope = true;
            }
            if HIO_M02F != 0 {
                self.moves[6].rope = true;
            }
            if HIO_M030 != 0 {
                self.moves[7].rope = true;
            }
            for i in 0..2 {
                let mut fvar9 = 0.0;
                if i == 0 && self.left_held {
                    fvar9 = length_of(
                        self.left_arm[CHAIN_TIP].pos - self.left_arm[HAND_MAX as usize - 1].pos,
                    );
                } else if i == 1 && self.right_held {
                    fvar9 = length_of(
                        self.right_arm[CHAIN_TIP].pos
                            - self.right_arm[HAND_MAX as usize - 1].pos,
                    );
                }
                let temp_f0_8 = 200.0 + REG0_F5;
                if fvar9 > temp_f0_8 {
                    fvar9 -= temp_f0_8;
                    fvar9 *= REG0_F6 + 0.2;
                } else {
                    fvar9 = 0.0;
                }
                add_calc2(&mut self.hand_scale[i], HIO_M124 + fvar9, 1.0, 20.0 + REG0_F4);
                add_calc0(&mut self.hand_dilate[i], 0.05, 0.02);
            }
            self.size_set();
            self.body_at = false;
            self.pelvis[0].at = false;
            self.left_arm[HAND_MAX as usize - 1].at = false;
            self.right_arm[HAND_MAX as usize - 1].at = false;
            for i in 0..TAIL_MAX as usize {
                self.tail[i].at = false;
            }
            self.action_main(actor, env, out);
            self.damage_check(env, out);
            self.shape_calc(env, out);
        }
    }

    fn demo_camera(&mut self, actor: &Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        let _ = actor;
        match self.cut_state {
            0 => {}
            1 => {
                if !self.cut_ordered {
                    self.cut_ordered = true;
                    return;
                }
                self.cut_state += 1;
                self.cut_count = 0;
                self.fov = CAM_FOV;
                self.cut_param = 0.0;
                self.cam_center = self.body_centre;
                self.cut_player = true;
                self.keese_spawn = 0;
                self.cut_state = 2;
            }
            2 => {
                let mut mtx = self.mtx;
                mtx.yrot_s(self.body_angle[1]);
                let local_44 = Vec3::new(REG0_F0 + 200.0, REG0_F1 + 100.0, REG0_F2 + 2000.0);
                self.cam_eye = mtx.pos(local_44);
                self.mtx = mtx;
                let fvar2 = if self.cut_count > 300 { 0.1 } else { 0.5 };
                add_calc2(&mut self.cam_center.x, self.body_centre.x, 0.1, 300.0);
                add_calc2(
                    &mut self.cam_center.y,
                    self.body_centre.y + 100.0 + REG0_F3,
                    fvar2,
                    400.0,
                );
                add_calc2(&mut self.cam_center.z, self.body_centre.z, 0.1, 300.0);
                let local_44 = Vec3::new(REG0_F4, env.player_pos.y, REG0_F5 + 1500.0);
                self.cut_player_pos = local_44;
                self.cut_player_yaw = self.body_angle[1].wrapping_add(i16::MIN);
                if self.cut_count == (REG0_S5 + 220) as i16 {
                    self.player_demo = 0x1d;
                }
                if self.cut_count == (REG0_S6 + 320) as i16 {
                    self.player_demo = 4;
                }
            }
            3 => {
                add_calc2(&mut self.cam_center.x, self.cam_anchor.x, 0.1, 100.0);
                add_calc2(&mut self.cam_center.y, self.cam_anchor.y + REG0_F13, 0.1, 200.0);
                add_calc2(&mut self.cam_center.z, self.cam_anchor.z, 0.1, 100.0);
                if self.cut_count == (REG0_S7 + 90) as i16 {
                    self.shake = REG0_F18 + 30.0;
                }
                if self.cut_count == (REG0_S7 + 100) as i16 {
                    self.player_demo = 5;
                    self.player_voice = 29;
                }
                if self.cut_count > (REG0_S8 + 110) as i16 {
                    add_calc2(&mut self.fov, REG0_F11 + 35.0, 0.1, REG0_F12 + 0.5);
                    if self.cut_count == 130 || self.cut_count == 180 {
                        out.se(SE_CM_BGN_T_MOUSE_OPEN);
                    }
                    if self.cut_count == 152 {
                        out.se(SE_CM_BGN_T_MOUSE_CLOSE);
                    }
                }
                add_calc_angle_s2(&mut self.blure, 1, 1, 1);
                if self.cut_count <= (REG0_S8 + 200) as i16 {
                    return;
                }
                out.bgm(BGM_BGN_TARABA);
                self.cut_release();
                return;
            }
            10 => {
                if !self.cut_ordered {
                    self.cut_ordered = true;
                    return;
                }
                self.cut_state += 1;
                self.cut_count = 0;
                self.fov = CAM_FOV;
                self.cut_param = 0.0;
                self.cam_center = anchor_vec(self.cam_anchor);
                self.cut_player = true;
                self.keese_spawn = 0;
                self.cut_state = 11;
            }
            11 | 12 => {
                add_calc2(&mut self.cam_rise.x, -1800.0, 0.05, self.cut_param * 2000.0);
                add_calc2(&mut self.cam_rise.y, 1100.0, 0.05, self.cut_param * 1000.0);
                add_calc2(&mut self.cam_rise.z, 1000.0, 0.05, self.cut_param * 1500.0);
                add_calc2(&mut self.cut_param, 0.01, 1.0, 0.0001);
                let mut mtx = self.mtx;
                mtx.yrot_s(self.cam_anchor.yaw);
                self.cam_eye = mtx.pos(self.cam_rise);
                self.mtx = mtx;
                if self.cut_state == 11 {
                    add_calc2(&mut self.cam_center.x, self.cam_anchor.x, 0.1, 300.0);
                    add_calc2(&mut self.cam_center.y, self.cam_anchor.y + REG0_F13, 0.1, 300.0);
                    add_calc2(&mut self.cam_center.z, self.cam_anchor.z, 0.1, 300.0);
                    self.cut_player_pos = Vec3::new(
                        REG0_F4,
                        env.player_pos.y,
                        REG0_F5 + -2000.0,
                    );
                    self.cut_player_yaw = self.cam_anchor.yaw;
                } else {
                    add_calc2(&mut self.cam_center.x, self.cam_follow.x, 0.1, 300.0);
                    add_calc2(
                        &mut self.cam_center.y,
                        self.cam_follow.y + 50.0 + REG8_F16,
                        0.1,
                        300.0,
                    );
                    add_calc2(&mut self.cam_center.z, self.cam_follow.z, 0.1, 300.0);
                    add_calc2(
                        &mut self.fov,
                        REG0_F17 + 30.0,
                        REG0_F18 + 0.05,
                        REG0_F19 + 0.2,
                    );
                    add_calc_angle_s2(&mut self.blure, 1, 1, 2);
                }
                if self.cut_count > (REG0_S9 + 370) as i16 {
                    self.cut_release();
                    return;
                }
            }
            20 => {
                if !self.cut_ordered {
                    self.cut_ordered = true;
                    return;
                }
                self.cut_state += 1;
                self.cut_count = 0;
                self.fov = CAM_FOV;
                self.cut_param = 0.0;
                self.cam_center = self.cam_follow;
                self.cut_player = true;
                self.blure = 0xb4;
                self.keese_spawn = 0;
                self.cut_state = 21;
            }
            21 => {
                add_calc_angle_s2(&mut self.blure, 1, 1, 1);
                self.cam_eye = Vec3::new(500.0, REG0_F3 + 500.0, REG0_F4 + 2000.0);
                add_calc2(&mut self.cam_center.x, self.cam_follow.x, 0.1, 300.0);
                add_calc2(&mut self.cam_center.y, self.cam_follow.y + REG0_F13, 0.1, 300.0);
                add_calc2(&mut self.cam_center.z, self.cam_follow.z, 0.1, 300.0);
                self.cut_player_pos = Vec3::new(
                    REG0_F4,
                    env.player_pos.y,
                    REG0_F5 + -1000.0,
                );
                self.cut_player_yaw = self.cam_anchor.yaw;
                if self.cut_count == 260 {
                    out.se(SE_CV_BGN_DIE);
                }
                if self.cut_count >= 370 {
                    out.se(SE_CM_BGN_LAST_EXPLODE);
                    self.body_draw = 3;
                    self.hio_phase = 3;
                    self.cut_state += 1;
                    self.defeat_state = 1;
                    self.blure = 0xb4;
                }
            }
            22 => {
                if self.cut_count >= 390 {
                    self.last_himo_control();
                }
                if self.cut_count >= (REG8_S7 + 500) as i16 {
                    self.exit_mode = 2;
                }
            }
            CUT_STATE_150 => {
                self.cut_release();
                return;
            }
            _ => {}
        }
        if self.cut_state != 0 {
            let fx = self.shake * sin_s((self.cut_count as i32 * 0x3300) as i16);
            let fy = self.shake * cos_s((self.cut_count as i32 * 0x3000) as i16);
            self.cam_eye.x += fx;
            self.cam_eye.y += fy;
            self.cam_center.x += fx;
            self.cam_center.y += fy;
            self.cam_bank = (self.shake * cos_s((self.frame as i32 * 0x1c00) as i16) * CAM_BANK_SCALE)
                as i16;
            add_calc0(&mut self.shake, 1.0, REG0_F16 + 1.0);
            self.cut_count += 1;
        }
    }

    fn cut_release(&mut self) {
        self.cut_state = 0;
        self.blure = 1;
    }

    pub fn execute(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        if self.prm == NO_BYTE {
            return;
        }
        if HIO_M027 != 0 {
            self.health = HIO_M027 as i32;
        }
        if self.exit_mode == 1 {
            self.exit_mode = 0;
            self.body_draw = 2;
            self.hio_phase = 2;
        }
        self.frame = self.frame.wrapping_add(1);
        for slot in self.timers.iter_mut() {
            timer(slot);
        }
        if self.core_lock != 0 {
            self.core_lock -= 1;
        }
        if self.hit_se != 0 {
            self.hit_se -= 1;
        }
        if self.arrow_flash != 0 {
            self.arrow_flash -= 1;
        }
        if self.ripple_timer != 0 {
            self.ripple_timer -= 1;
        }
        if self.hit_tag != 0 {
            self.hit_tag += 1;
            if self.hit_tag > 100 {
                self.hit_tag = 0;
            }
        }
        self.move_step(actor, env, out);
        self.demo_camera(actor, env, out);
        if self.keese_spawn != 0 {
            self.keese_spawn -= 1;
            if env.keese < HIO_KEESE_NUM_MAX {
                out.keese.push(KeeseSpawn {
                    pos: Vec3::new(
                        self.rnd_fx(2500.0),
                        self.rnd_f(500.0) + 3500.0,
                        self.rnd_fx(2500.0),
                    ),
                    room: actor.room,
                });
            }
        }
        self.body_draw = self.hio_phase as i8;
        if self.exit_mode == 2 {
            self.exit_mode = 0;
            out.next.exit = None;
            out.next.target = Some(Target {
                stage: STAGE_GANONK.to_owned(),
                room: STAGE_GANONK_ROOM,
                start: STAGE_GANONK_POINT,
            });
            out.stories.set(EVENT_BIT_3F10);
        }
    }

    pub fn begin(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        self.entered = true;
        self.home = actor.pos;
        if env.stories_done {
            self.prm = NO_BYTE;
        } else {
            for i in 0..MOVE_COUNT {
                self.moves[i].join = 3;
            }
            out.bgm(BGM_BGN_KUGUTSU);
            self.health = MAX_HEALTH;
            self.max_health = MAX_HEALTH;
        }
        self.scale = 1.0;
        self.action = ACTION_START;
        self.timers[0] = 60;
        self.execute(actor, env, out);
    }

    pub fn is_puppet_ganon(name: &str) -> bool {
        is_puppet_ganon(name)
    }
    fn dance_b(&mut self, out: &mut Sink<'_>) {
        out.se(SE_CM_BGN_D_DANCE);
        for i in 0..MOVE_COUNT {
            match self.dance_sub {
                0 => {
                    self.moves[i].target = DANCE_PAUSE_3[i];
                    if self.timers[3] == 0 {
                        self.dance_sub += 1;
                        self.timers[3] = match self.health {
                            3 => HIO_M0E8,
                            2 => HIO_M0EA,
                            _ => HIO_M0EC,
                        };
                    }
                }
                1 => {
                    self.moves[i].target = DANCE_PAUSE_4[i];
                    if self.timers[3] == 0 {
                        self.dance_sub = 0;
                        self.timers[3] = match self.health {
                            3 => HIO_M0E8,
                            2 => HIO_M0EA,
                            _ => HIO_M0EC,
                        };
                    }
                }
                _ => {}
            }
        }
        let svar4 = if self.dance_sub == 1 {
            DANCE_B_TARGET_A
        } else {
            DANCE_B_TARGET_B
        };
        add_calc_angle_s2(&mut self.yaw_bob, svar4, DANCE_B_STEP, HIO_M0EE);
    }

    fn ki_set(&mut self) {
        self.keese_spawn = match self.health {
            3 => HIO_KEESE_NUM_3HP,
            2 => HIO_KEESE_NUM_2HP,
            _ => HIO_KEESE_NUM_1HP,
        };
    }

    fn dance_0(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        if self.timers[1] == 0 {
            self.timers[1] = self.rnd_f(150.0) as i16 + 50;
            self.dest.x = self.rnd_fx(1500.0);
            self.dest.z = self.rnd_fx(1500.0);
            self.dest_speed = 0.0;
        }
        add_calc2(&mut actor.pos.x, self.dest.x, 0.05, self.dest_speed);
        add_calc2(&mut actor.pos.z, self.dest.z, 0.05, self.dest_speed);
        add_calc2(&mut self.dest_speed, REG0_F19 + 20.0, 1.0, 0.5);
        let bob = (REG0_F9 + 50.0) * sin_s((self.frame as i32 * (REG0_S9 as i32 + 500)) as i16);
        add_calc2(&mut self.body.y, bob, 0.5, REG0_F8 + 20.0);
        if !self.moves[0].rope || !self.moves[1].rope {
            let target = Self::player_yaw(env, actor.pos);
            add_calc_angle_s2(&mut self.shape_angle[1], target, 10, 0x400);
        }
        let mut early = false;
        match self.sub {
            0 => {
                if self.move_count == 0 {
                    self.dance = 0;
                    self.timers[0] = HIO_M0D8;
                    self.ki_set();
                } else {
                    self.dance = 1;
                    self.timers[0] = match self.health {
                        3 => HIO_M0E2,
                        2 => HIO_M0E4,
                        _ => HIO_M0E6,
                    };
                }
                self.dance_sub = 0;
                self.sub += 1;
            }
            1 => {
                if self.timers[0] != 0 || env.keese != 0 || HIO_M028 == 0 {
                    early = true;
                } else if self.move_count == 1
                    || self.move_count == 2
                    || self.move_count == 3
                {
                    if self.moves[0].rope && self.moves[1].rope && self.timers[2] == 0 {
                        self.action = ACTION_HEAD_RECOVER;
                        self.sub = 0;
                        self.cut_param = 0.0;
                        early = true;
                    } else {
                        self.action = ACTION_PUNCH;
                        self.timers[0] = REG0_S0 + 0x32;
                        let left = self.moves[3].rope;
                        if !left && !self.moves[4].rope {
                            self.sub = 5;
                            self.timers[0] = REG0_S6 + 0x40;
                            out.se(SE_CV_BGN_D_ATTACK);
                        } else if !left {
                            self.sub = 3;
                            out.se(SE_CV_BGN_D_ATTACK);
                        } else if !self.moves[4].rope {
                            self.sub = 1;
                            out.se(SE_CV_BGN_D_ATTACK);
                        } else if !self.moves[7].rope {
                            self.action = ACTION_TAIL;
                            self.sub = 0;
                        } else {
                            self.action = ACTION_BODY;
                            self.sub = 0;
                        }
                    }
                } else if self.move_count == 0 || self.move_count == 5 {
                    self.sub = 0;
                }
                if !early {
                    self.move_count += 1;
                    if self.move_count > 5 {
                        self.move_count = 0;
                    }
                }
            }
            _ => {}
        }
        if early {
            return;
        }
        match self.dance {
            0 => self.dance_a(out),
            1 => self.dance_b(out),
            _ => {}
        }
        for i in 0..MOVE_COUNT {
            if !self.moves[i].rope {
                let local_34 = Vec3::new(
                    f32::from(self.moves[i].target[0]) - self.moves[i].pos.x,
                    f32::from(self.moves[i].target[1]) - self.moves[i].pos.y,
                    f32::from(self.moves[i].target[2]) - self.moves[i].pos.z,
                );
                let fvar9 = length_of(local_34);
                let temp_f0_4 = REG0_F0 + 100.0;
                if fvar9 > temp_f0_4 {
                    let f4 = fvar9 - temp_f0_4;
                    let want = ((REG0_F2 + 50.0) + (f4 * (REG0_F1 + 0.06))) as i32 as i16;
                    if want > self.moves[i].tension {
                        self.moves[i].tension = want;
                    }
                }
            }
        }
    }

}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    water: Option<Res<'w, Water>>,
    combat: Res<'w, super::combat::Combat>,
    held: Res<'w, Held>,
    players: Query<'w, 's, &'static Player>,
    shafts: Query<'w, 's, &'static arrow::Arrow>,
    others: Query<'w, 's, (Entity, &'static Actor), Without<PuppetGanon>>,
}

pub const KEESE_ACTORS: [&str; 2] = ["keeth", "Fkeeth"];
pub const KEESE_SPAWN_NAME: &str = "keeth";
pub const KEESE_SPAWN_PARAMETERS: u32 = 0xFFFF_0003;
pub const BODY_2_ACTOR: &str = "Bgn2";
pub const BODY_3_ACTOR: &str = "Bgn3";

fn blow_of(watch: &Watch<'_, '_>) -> Blow {
    let mut at = At::Idle;
    let mut pos = None;
    if let Some(shaft) = watch
        .shafts
        .iter()
        .find(|shaft| shaft.phase == arrow::Phase::Flying)
    {
        at = match shaft.kind {
            arrow::TYPE_LIGHT => At::LightArrow,
            arrow::TYPE_FIRE => At::FireArrow,
            arrow::TYPE_ICE => At::IceArrow,
            _ => At::Arrow,
        };
        pos = Some(shaft.pos);
    }
    if at == At::Idle {
        return Blow {
            at,
            blade: watch.held.item,
            soup: watch.combat.soup,
            pos,
        };
    }
    Blow {
        at,
        blade: watch.held.item,
        soup: watch.combat.soup,
        pos,
    }
}

pub fn control(
    watch: Watch<'_, '_>,
    mut events: ResMut<SimEvents>,
    mut switches: ResMut<Switches>,
    mut stories: ResMut<StoryBits>,
    mut flags: ResMut<EventFlags>,
    mut health: ResMut<Health>,
    mut next: ResMut<NextStage>,
    mut zones: ResMut<ZoneActors>,
    mut commands: Commands,
    mut bosses: Query<
        (Entity, &mut Actor, &mut PuppetGanon),
        (With<PuppetGanon>, Without<Player>),
    >,
) {
    let solid = watch.collision.as_deref();
    let water = watch.water.as_deref();
    let blow = blow_of(&watch);
    let stories_done = stories.test(EVENT_BIT_3F10);
    let mut live = 0i32;
    let mut anchor = Anchor::ZERO;
    let mut follow = Vec3::ZERO;
    for (_, other) in watch.others.iter() {
        if KEESE_ACTORS.contains(&other.name.as_str()) {
            live += 1;
        } else if other.name == BODY_2_ACTOR {
            anchor.x = other.pos.x;
            anchor.y = other.pos.y;
            anchor.z = other.pos.z;
            anchor.yaw = other.angle[1];
        } else if other.name == BODY_3_ACTOR {
            follow = other.pos;
        }
    }
    let Some(player) = watch.players.iter().next() else {
        return;
    };
    let mut drop: Vec<(Entity, i8, u16)> = Vec::new();
    let mut spawns: Vec<KeeseSpawn> = Vec::new();
    let mut pending: Vec<Entity> = Vec::new();
    for (entity, mut actor, mut boss) in &mut bosses {
        let actor = &mut *actor;
        let boss = &mut *boss;
        boss.cam_anchor = anchor;
        boss.cam_follow = follow;
        let env = Env {
            player_pos: player.pos,
            solid,
            water,
            blow,
            keese: live,
            stories_done,
        };
        let mut sink = Sink {
            events: &mut events,
            switches: &mut switches,
            stories: &mut stories,
            flags: &mut flags,
            health: &mut health,
            next: &mut next,
            drop: &mut drop,
            keese: &mut spawns,
        };
        if !boss.entered {
            boss.begin(actor, &env, &mut sink);
        } else {
            boss.execute(actor, &env, &mut sink);
        }
        if boss.exit_mode == 2 {
            drop.push((entity, actor.room, actor.set_id));
        }
    }
    for spawn in spawns {
        let child = commands.spawn((
            Actor {
                name: KEESE_SPAWN_NAME.to_owned(),
                parameters: KEESE_SPAWN_PARAMETERS,
                pos: spawn.pos,
                angle: [0; 3],
                set_id: 0,
                room: spawn.room,
            },
            Keese,
        ));
        pending.push(child.id());
    }
    for (entity, room, set_id) in drop {
        zones.mark_done(room, set_id);
        pending.push(entity);
    }
    for entity in pending {
        let _ = commands.entity(entity).despawn();
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Keese;
