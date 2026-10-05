use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::arrow;
use super::boomerang;
use super::bombflower;
use super::collision::Collision;
use super::combat::Combat;
use super::damage;
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, StoryBits, Switches, ZoneActors};
use super::hammer;
use super::health::Health;
use super::item::Held;
use super::player::Player;
use super::stream::{add_calc_angle_s2, cos_s, sin_s};
use super::walk::{from_s16, to_s16};

pub const HELMAROC_ACTOR: &str = "Bdk";
pub const BDK_OBJ_ACTOR: &str = "Bdkobj";

pub fn is_helmaroc(name: &str) -> bool {
    name == HELMAROC_ACTOR
}

const ANGLE_X: usize = 0;
const ANGLE_Y: usize = 1;
const ANGLE_Z: usize = 2;

const PRM_BYTE_MASK: u32 = 0xF;
const SWITCH_TOGE: u8 = 0x80;
const EVENT_BIT_LAST_DAMAGE: u16 = 0x3C01;
const TMP_BIT_SHUTTER: u32 = 0x0401;

const HIO_NO: i8 = -1;
const HIO_M005: u8 = 0;
const HIO_M006: i16 = 0;
const HIO_M008: f32 = 1.0;
const HIO_M00C: f32 = 4.0;
const HIO_M010: f32 = 1.5;
const HIO_M014: u8 = 0;
const HIO_M018: f32 = 20.0;
const HIO_M01C: f32 = 2.0;
const HIO_M020: f32 = 60.0;
const HIO_M024: f32 = 100.0;
const HIO_M028: i16 = 0x2D0;

const REG0_F0: f32 = 0.0;
const REG0_F1: f32 = 0.0;
const REG0_F2: f32 = 0.0;
const REG0_F3: f32 = 0.0;
const REG0_F4: f32 = 0.0;
const REG0_F7: f32 = 0.0;
const REG0_F8: f32 = 0.0;
const REG0_F9: f32 = 0.0;
const REG0_F10: f32 = 0.0;
const REG0_F11: f32 = 0.0;
const REG0_F13: f32 = 0.0;
const REG0_F14: f32 = 0.0;
const REG0_F15: f32 = 0.0;
const REG0_F16: f32 = 0.0;
const REG0_F18: f32 = 0.0;
const REG0_S0: i16 = 0;
const REG0_S1: i16 = 0;
const REG0_S3: i16 = 0;
const REG0_S4: i16 = 0;
const REG0_S5: i16 = 0;
const REG0_S6: i16 = 0;
const REG0_S7: i16 = 0;
const REG0_S8: i16 = 0;

const REG6_F0: f32 = 0.0;
const REG6_F2: f32 = 0.0;
const REG6_F4: f32 = 0.0;
const REG6_F9: f32 = 0.0;
const REG6_F10: f32 = 0.0;
const REG6_F11: f32 = 0.0;
const REG6_F12: f32 = 0.0;
const REG6_F13: f32 = 0.0;
const REG6_F16: f32 = 0.0;
const REG6_F17: f32 = 0.0;
const REG6_F18: f32 = 0.0;
const REG6_F19: f32 = 0.0;
const REG6_S0: i16 = 0;
const REG6_S2: i16 = 0;
const REG6_S3: i16 = 0;
const REG6_S4: i16 = 0;
const REG6_S6: i16 = 0;
const REG6_S7: i16 = 0;

const REG8_F0: f32 = 0.0;
const REG8_F1: f32 = 0.0;
const REG8_F2: f32 = 0.0;
const REG8_F3: f32 = 0.0;
const REG8_F4: f32 = 0.0;
const REG8_F5: f32 = 0.0;
const REG8_F6: f32 = 0.0;
const REG8_F7: f32 = 0.0;
const REG8_F8: f32 = 0.0;
const REG8_F9: f32 = 0.0;
const REG8_F10: f32 = 0.0;
const REG8_F11: f32 = 0.0;
const REG8_F15: f32 = 0.0;
const REG8_F18: f32 = 0.0;
const REG8_F19: f32 = 0.0;
const REG8_S2: i16 = 0;
const REG8_S4: i16 = 0;
const REG8_S5: i16 = 0;
const REG8_S8: i16 = 0;

const REG12_F2: f32 = 0.0;
const REG12_F3: f32 = 0.0;
const REG12_F4: f32 = 0.0;
const REG12_F5: f32 = 0.0;
const REG12_F6: f32 = 0.0;
const REG12_F7: f32 = 0.0;
const REG12_F8: f32 = 0.0;
const REG12_F9: f32 = 0.0;
const REG12_F11: f32 = 0.0;
const REG12_S7: i16 = 0;
const REG12_S8: i16 = 0;

const REG13_F0: f32 = 0.0;
const REG13_F1: f32 = 0.0;
const REG13_F2: f32 = 0.0;
const REG13_F3: f32 = 0.0;
const REG13_F4: f32 = 0.0;
const REG13_F5: f32 = 0.0;
const REG13_F6: f32 = 0.0;
const REG13_F7: f32 = 0.0;
const REG13_F8: f32 = 0.0;
const REG13_F9: f32 = 0.0;
const REG13_F10: f32 = 0.0;
const REG13_F11: f32 = 0.0;
const REG13_F12: f32 = 0.0;
const REG13_F13: f32 = 0.0;
const REG13_F17: f32 = 0.0;
const REG13_F18: f32 = 0.0;
const REG13_S1: i16 = 0;
const REG13_S2: i16 = 0;
const REG13_S6: i16 = 0;
const REG13_S8: i16 = 0;

const REG14_F0: f32 = 0.0;
const REG14_F1: f32 = 0.0;
const REG14_F2: f32 = 0.0;
const REG14_F3: f32 = 0.0;
const REG14_F4: f32 = 0.0;
const REG14_F5: f32 = 0.0;
const REG14_F7: f32 = 0.0;
const REG14_F15: f32 = 0.0;
const REG14_S8: i16 = 0;
const REG14_S9: i16 = 0;

const REG17_S0: i16 = 0;

const AREA_X: f32 = 3600.0;
const AREA_Y: f32 = 9800.0;
const AREA_Z: f32 = -3800.0;
const AREA2_Y: f32 = 9720.0;

pub const CULL_MIN: [f32; 3] = [-650.0, -20.0, -400.0];
pub const CULL_MAX: [f32; 3] = [650.0, 790.0, 320.0];

const GRAVITY: f32 = -5.0;
const GRAVITY_FALL: f32 = -2.5;
const MAX_HEALTH: i32 = 20;
const HAMMER_HEALTH: i32 = 0x14;
const AT_POWER_HEAD: u8 = 2;
const AT_POWER_FOOT: u8 = 2;

const HEAD_OFFSET: [f32; 3] = [40.0, 0.0, -70.0];
const BEAK_OFFSET: [f32; 3] = [120.0, 0.0, 0.0];
const BEAK_TIP_OFFSET: [f32; 3] = [0.0, 0.0, 0.0];
const EYE_OFFSET: [f32; 3] = [120.0, 0.0, -120.0];
const FOOT_OFFSET: [f32; 3] = [50.0, 0.0, 0.0];
const FOOT_OFFSET_TALL: [f32; 3] = [100.0, 200.0, 0.0];
const BODY_LIFT: f32 = 380.0;
const HEAD_RADIUS_MASKED: f32 = 110.0;
const HEAD_RADIUS_BARE: f32 = 80.0;
const BEAK_RADIUS: f32 = 70.0;
const BODY_RADIUS: f32 = 250.0;
const FOOT_RADIUS: f32 = 60.0;
const FOOT_RADIUS_T_FLY: f32 = 300.0;
const TOSAKA_RADIUS: f32 = 80.0;
const WIND_RADIUS: f32 = 300.0;
const WIND_RADIUS_BATTLE: f32 = 600.0;
const MODEL_SCALE: f32 = 1.0;
const MODEL_SCALE_BATTLE: f32 = 0.85;
const HEAD_OFF_Y: f32 = -10000.0;

const JOINT_ATAMA1: [f32; 3] = [0.0, 592.90, 135.00];
const JOINT_AGO: [f32; 3] = [0.0, 577.44, 172.50];
const JOINT_TOSAKA_A3: [f32; 3] = [0.0, 754.15, 67.96];
const JOINT_ASHI_L3: [f32; 3] = [75.02, 129.52, 185.61];
const JOINT_ASHI_R3: [f32; 3] = [-74.98, 129.51, 185.61];

const EMODE_NONE: u8 = 0;
const EMODE_LOOP: u8 = 2;
const STATE_STOP: u8 = 1;
const STATE_LOOP: u8 = 2;

const BCK_ATTACK1: i32 = 0x1C;
const BCK_ATTACK_T1: i32 = 0x1D;
const BCK_ATTACK_T2: i32 = 0x1E;
const BCK_ATTACK_T3: i32 = 0x1F;
const BCK_DAMAGE1: i32 = 0x21;
const BCK_DAMAGE2: i32 = 0x22;
const BCK_DAMAGE3: i32 = 0x23;
const BCK_DAMAGE4: i32 = 0x24;
const BCK_DAMAGE_T1: i32 = 0x25;
const BCK_DEATH1: i32 = 0x26;
const BCK_DEATH2: i32 = 0x27;
const BCK_FLY1: i32 = 0x2A;
const BCK_FLY2: i32 = 0x2B;
const BCK_FLY3: i32 = 0x2C;
const BCK_JIDANDA1: i32 = 0x2D;
const BCK_JUMP1: i32 = 0x2E;
const BCK_KAKKU1: i32 = 0x2F;
const BCK_LAST_DAMAGE1: i32 = 0x30;
const BCK_LAST_DAMAGE2: i32 = 0x31;
const BCK_LAST_DAMAGE3: i32 = 0x32;
const BCK_NUKENAI1: i32 = 0x33;
const BCK_NUKU1: i32 = 0x34;
const BCK_PIKUPIKU1: i32 = 0x35;
const BCK_RAKKA_T1: i32 = 0x36;
const BCK_SENKAI_L1: i32 = 0x37;
const BCK_SENKAI_R1: i32 = 0x38;
const BCK_S_DEMO1: i32 = 0x39;
const BCK_S_DEMO2: i32 = 0x3A;
const BCK_TOBITATU1: i32 = 0x3B;
const BCK_TOBITATU_T1: i32 = 0x3C;
const BCK_TO_DAMAGE_T1: i32 = 0x3D;
const BCK_TYAKUTI1: i32 = 0x3E;
const BCK_TYAKUTI_T1: i32 = 0x3F;
const BCK_WAIT1: i32 = 0x40;
const BCK_WAIT_T1: i32 = 0x41;
const BCK_WALK1: i32 = 0x42;

const BCK_ATTACK1_FRAMES: i16 = 50;
const BCK_ATTACK_T1_FRAMES: i16 = 75;
const BCK_ATTACK_T2_FRAMES: i16 = 47;
const BCK_ATTACK_T3_FRAMES: i16 = 29;
const BCK_DAMAGE1_FRAMES: i16 = 16;
const BCK_DAMAGE2_FRAMES: i16 = 50;
const BCK_DAMAGE3_FRAMES: i16 = 30;
const BCK_DAMAGE4_FRAMES: i16 = 20;
const BCK_DAMAGE_T1_FRAMES: i16 = 16;
const BCK_DEATH1_FRAMES: i16 = 94;
const BCK_DEATH2_FRAMES: i16 = 0;
const BCK_FLY1_FRAMES: i16 = 28;
const BCK_FLY2_FRAMES: i16 = 28;
const BCK_FLY3_FRAMES: i16 = 28;
const BCK_JIDANDA1_FRAMES: i16 = 24;
const BCK_JUMP1_FRAMES: i16 = 11;
const BCK_KAKKU1_FRAMES: i16 = 24;
const BCK_LAST_DAMAGE_FRAMES: i16 = 209;
const BCK_NUKENAI1_FRAMES: i16 = 20;
const BCK_NUKU1_FRAMES: i16 = 43;
const BCK_PIKUPIKU1_FRAMES: i16 = 24;
const BCK_RAKKA_T1_FRAMES: i16 = 16;
const BCK_SENKAI_L1_FRAMES: i16 = 23;
const BCK_SENKAI_R1_FRAMES: i16 = 23;
const BCK_S_DEMO1_FRAMES: i16 = 1;
const BCK_S_DEMO2_FRAMES: i16 = 98;
const BCK_TOBITATU1_FRAMES: i16 = 52;
const BCK_TOBITATU_T1_FRAMES: i16 = 52;
const BCK_TO_DAMAGE_T1_FRAMES: i16 = 27;
const BCK_TYAKUTI1_FRAMES: i16 = 27;
const BCK_TYAKUTI_T1_FRAMES: i16 = 27;
const BCK_WAIT1_FRAMES: i16 = 60;
const BCK_WAIT_T1_FRAMES: i16 = 120;
const BCK_WALK1_FRAMES: i16 = 48;

const BAS_NONE: i32 = -1;

const HIT_NONE: u8 = 0;
const HIT_HEAD: u8 = 1;
const HIT_TOSAKA: u8 = 2;
const AT_NONE: u8 = 0;
const AT_SWORD: u8 = 1;
const AT_BOMB: u8 = 2;
const AT_TSUBO: u8 = 3;
const AT_JUG: u8 = 4;
const AT_HOOKSHOT: u8 = 5;
const AT_MAGBALL: u8 = 6;
const AT_LANCE: u8 = 7;
const AT_WIND: u8 = 8;
const AT_HAMMER: u8 = 9;
const AT_BOOMERANG: u8 = 10;
const AT_HELMAROC: u8 = 11;
const AT_OTHER: u8 = 12;
const LANCE_POWER_MAX: i32 = 2;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const SE_CM_DK_ROUND_UP: u32 = 0x586B;
const SE_CM_DK_PULL_UP_HEAD: u32 = 0x5874;
const SE_CM_MAGBALL_BOUND: u32 = 0x5815;
const SE_CM_DK_GLIDING: u32 = 0x506F;
const SE_CM_DK_WIND: u32 = 0x5076;
const SE_CM_DK_NAIL: u32 = 0x5079;
const SE_CM_DK_ATTACK: u32 = 0x5872;
const SE_CM_DK_JITABATA: u32 = 0x5873;
const SE_CM_DK_BREAK_MASK: u32 = 0x5875;
const SE_CM_DK_CRASH_WALL: u32 = 0x5877;
const SE_CM_DK_DIE_EXPLODE: u32 = 0x5878;
const SE_CM_DK_FOOT_ATTACK: u32 = 0x589D;
const SE_CM_DK_FOOT_ATK_COL: u32 = 0x589E;
const SE_CM_DK_FALL_WATER: u32 = 0x589F;
const SE_CM_DK_PECK: u32 = 0x589C;
const SE_CM_DK_WATER_COLUMN: u32 = 0x782E;
const SE_CV_DK_ENTER: u32 = 0x4864;
const SE_CV_DK_ATTACK: u32 = 0x4865;
const SE_CV_DK_NUKEZU: u32 = 0x4866;
const SE_CV_DK_NUKERU: u32 = 0x4867;
const SE_CV_DK_DAMAGE: u32 = 0x4868;
const SE_CV_DK_KABE_DAMAGE: u32 = 0x4869;
const SE_CV_DK_LAST_DAMAGE: u32 = 0x486A;
const SE_CV_DK_MODAE: u32 = 0x486B;
const SE_CV_DK_DIE: u32 = 0x486C;
const SE_LK_HAMMER_HIT: u32 = 0x2855;
const SE_ATM_MJT_JINARI: u32 = 0x7020;
const SE_ATM_MJT_SHUTTER: u32 = 0x7021;
const SE_ATM_MJT_SHUTTER_END: u32 = 0x7822;
const SE_OBJ_TOGE_IN: u32 = 0x6977;

const BGM_DK_BATTLE: u32 = 0x8000_0014;
const BGM_MJ_TOWER_BATTLE: u32 = 0x8000_0015;
const BGM_UNK_110: u32 = 0x8000_0110;

const VIBE_Y: f32 = 1.0;
const VIBE_ATTACK: i16 = -0x21;
const VIBE_LAND: i16 = 5;
const VIBE_JAB: i16 = 4;
const VIBE_KUTI: i16 = 6;
const VIBE_PECK: i16 = 3;

const KAMEN_BREAK_OFF_X: [f32; 4] = [50.0, 30.0, -50.0, -30.0];
const KAMEN_BREAK_OFF_Y: [f32; 4] = [0.0, -120.0, 20.0, 0.0];
const KAMEN_BREAK_OFF_Z: [f32; 4] = [50.0, 20.0, 50.0, 0.0];
const KAMEN_BREAK_SD_X: [f32; 4] = [-5.0, 0.0, 5.0, 0.0];
const KAMEN_BREAK_SD_Z: [f32; 4] = [0.0, 5.0, 0.0, 0.0];
const KAMEN_BREAK_TIME: [i8; 4] = [0x1E, 0x2D, 0x28, 0x23];
const KAMEN_BREAK_G: [f32; 4] = [9820.0, 9820.0, 9820.0, 9820.0];
const KAMEN_BREAK_Z1: [i16; 4] = [0x2000, 0x2000, -0x2000, -0x2000];
const KAMEN_BREAK_Z2: [i16; 4] = [-0x1000, -0x1000, 0x1000, 0x1000];

const EFF_COUNT: usize = 40;
const EFF_ROCK: usize = 0x27;
const WIND_COUNT: usize = 10;
const PIECE_COUNT: usize = 4;
const TAIL_COUNT: usize = 4;
const TIMER_COUNT: usize = 5;
const SMOKE_COUNT: usize = 4;
const FOOT_COUNT: usize = 2;
const WALL_INNER: f32 = 50.0;
const WALL_OUTER: f32 = 300.0;
const WALL_TOGE_INNER: f32 = 500.0;
const WALL_TOGE_OUTER: f32 = 500.0;
const GROUND_UP_Y: f32 = 0.0;
const EFFECT_LIFE: i16 = 2000;
const EFFECT_LIFE_LIMIT: i16 = 0x26;
const LANDING_AREA: f32 = 1400.0;
const LANDING_LIFT: f32 = 600.0;
const APPROACH: f32 = 1300.0;
const APPROACH_STOP: f32 = 300.0;
const VIEW_LIMIT: u16 = 0x6000;
const WALK_LIMIT: i16 = 0x3000;
const SENKAI_LIMIT: i16 = 0x800;
const TRACK_LIMIT: i16 = 7000;
const TURN_DIV: i16 = 10;
const TURN_DIV_WALK: i16 = 2;
const TURN_DIV_AIM: i16 = 8;
const TURN_DIV_FACE: i16 = 4;
const TURN_DIV_CAM: i16 = 0x10;
const TURN_DIV_HEAD: i16 = 4;
const TURN_DIV_NECK: i16 = 0x10;
const TURN_CAP: f32 = 0x800;
const TURN_CAP_TILT: f32 = 0.5;
const TURN_BANK: f32 = 5500.0;
const TURN_STEP: f32 = 0.05;
const TURN_STEP_GROUND: f32 = 0.1;
const SPEED_STEP: f32 = 1.0;
const RISE_SPEED_UP: f32 = 1.5;
const RISE_SPEED_DOWN: f32 = -1.5;
const RISE_FRAMES: f32 = 20.0;

const ACTION_FLY: i16 = 0;
const ACTION_UP_FLY: i16 = 1;
const ACTION_LANDING: i16 = 2;
const ACTION_WAIT: i16 = 3;
const ACTION_JUMP: i16 = 4;
const ACTION_KUTI_ATTACK: i16 = 5;
const ACTION_JIDA_ATTACK: i16 = 6;
const ACTION_FLY_ATTACK: i16 = 7;
const ACTION_WIND_ATTACK: i16 = 8;
const ACTION_KAMEN_DEMO: i16 = 9;
const ACTION_END: i16 = 0xA;
const ACTION_START: i16 = 0xF;
const ACTION_T_FLY: i16 = 0x64;
const ACTION_T_LANDING: i16 = 0x65;
const ACTION_T_LASTATTACK: i16 = 0x66;
const ACTION_T_DOWN: i16 = 0x67;
const ACTION_AFTER_FIGHT: i16 = 0x6E;
const ACTION_LIMIT: i16 = 0x64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Fly,
    UpFly,
    Landing,
    Wait,
    Jump,
    KutiAttack,
    JidaAttack,
    FlyAttack,
    WindAttack,
    KamenDemo,
    End,
    Start,
    TFly,
    TLanding,
    TLastAttack,
    TDown,
    AfterFight,
}

impl Action {
    pub fn code(self) -> i16 {
        match self {
            Self::Fly => ACTION_FLY,
            Self::UpFly => ACTION_UP_FLY,
            Self::Landing => ACTION_LANDING,
            Self::Wait => ACTION_WAIT,
            Self::Jump => ACTION_JUMP,
            Self::KutiAttack => ACTION_KUTI_ATTACK,
            Self::JidaAttack => ACTION_JIDA_ATTACK,
            Self::FlyAttack => ACTION_FLY_ATTACK,
            Self::WindAttack => ACTION_WIND_ATTACK,
            Self::KamenDemo => ACTION_KAMEN_DEMO,
            Self::End => ACTION_END,
            Self::Start => ACTION_START,
            Self::TFly => ACTION_T_FLY,
            Self::TLanding => ACTION_T_LANDING,
            Self::TLastAttack => ACTION_T_LASTATTACK,
            Self::TDown => ACTION_T_DOWN,
            Self::AfterFight => ACTION_AFTER_FIGHT,
        }
    }

    fn by_code(code: i16) -> Action {
        match code {
            ACTION_UP_FLY => Action::UpFly,
            ACTION_LANDING => Action::Landing,
            ACTION_WAIT => Action::Wait,
            ACTION_JUMP => Action::Jump,
            ACTION_KUTI_ATTACK => Action::KutiAttack,
            ACTION_JIDA_ATTACK => Action::JidaAttack,
            ACTION_FLY_ATTACK => Action::FlyAttack,
            ACTION_WIND_ATTACK => Action::WindAttack,
            ACTION_KAMEN_DEMO => Action::KamenDemo,
            ACTION_END => Action::End,
            ACTION_START => Action::Start,
            ACTION_T_FLY => Action::TFly,
            ACTION_T_LANDING => Action::TLanding,
            ACTION_T_LASTATTACK => Action::TLastAttack,
            ACTION_T_DOWN => Action::TDown,
            ACTION_AFTER_FIGHT => Action::AfterFight,
            _ => Action::Fly,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Eff {
    pub kind: i8,
    pub done: i8,
    pub prev: Vec3,
    pub pos: Vec3,
    pub speed: f32,
    pub fall: f32,
    pub scale: f32,
    pub sway: f32,
    pub spin: [f32; 3],
    pub tumble: [f32; 3],
    pub count: i16,
    pub life: i16,
    pub chase: u8,
}

impl Eff {
    fn idle() -> Self {
        Self {
            kind: 0,
            done: 0,
            prev: Vec3::ZERO,
            pos: Vec3::ZERO,
            speed: 0.0,
            fall: 0.0,
            scale: 0.0,
            sway: 0.0,
            spin: [0.0; 3],
            tumble: [0.0; 3],
            count: 0,
            life: 0,
            chase: 0,
        }
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
    pub cur_morf: f32,
    pub prev_morf: f32,
    pub morf_step: f32,
}

impl Frame {
    fn idle() -> Self {
        Self {
            attribute: EMODE_LOOP,
            state: 0,
            start: 0,
            end: 0,
            loop_point: 0,
            rate: 1.0,
            frame: 0.0,
            cur_morf: 1.0,
            prev_morf: 1.0,
            morf_step: 0.0,
        }
    }

    fn init(&mut self, end: i16) {
        self.attribute = EMODE_LOOP;
        self.state = 0;
        self.start = 0;
        self.end = end;
        self.loop_point = 0;
        self.rate = 1.0;
        self.frame = 0.0;
    }

    fn set_loop(&mut self, frame: f32) {
        self.loop_point = frame as i16;
    }

    fn set_morf(&mut self, morf: f32) {
        if self.prev_morf < 0.0 || morf <= 0.0 {
            self.cur_morf = 1.0;
        } else {
            self.cur_morf = 0.0;
            self.morf_step = 1.0 / morf;
        }
        self.prev_morf = self.cur_morf;
    }

    fn update(&mut self) {
        self.state = 0;
        self.frame += self.rate;
        let start = f32::from(self.start);
        let end = f32::from(self.end);
        match self.attribute {
            EMODE_NONE => {
                if self.frame < start {
                    self.frame = start;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
                if self.frame >= end {
                    self.frame = end - 0.001;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
            }
            EMODE_LOOP => {
                let low = i32::from(self.loop_point) - i32::from(self.start);
                let high = i32::from(self.end) - i32::from(self.loop_point);
                while self.frame < start {
                    self.state |= STATE_LOOP;
                    if low <= 0 {
                        break;
                    }
                    self.frame += f32::from(low);
                }
                while self.frame >= end {
                    self.state |= STATE_LOOP;
                    if high <= 0 {
                        break;
                    }
                    self.frame -= f32::from(high);
                }
            }
            _ => {}
        }
    }

    fn play(&mut self) {
        self.update();
    }

    fn is_stop(&self) -> bool {
        self.state & STATE_STOP != 0 || self.rate == 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum At {
    Idle,
    Sword,
    Boomerang,
    Hookshot,
    Arrow,
    FireArrow,
    IceArrow,
    LightArrow,
    Hammer,
    Bomb,
    Tsubo,
    TsuboJug,
    Lance,
    Wind,
    Water,
    Helmaroc,
    Other,
}

impl At {
    fn result(self) -> u8 {
        match self {
            Self::Idle => AT_NONE,
            Self::Sword => AT_SWORD,
            Self::Bomb => AT_BOMB,
            Self::Tsubo => AT_TSUBO,
            Self::TsuboJug => AT_JUG,
            Self::Hookshot | Self::FireArrow => AT_HOOKSHOT,
            Self::Lance => AT_LANCE,
            Self::Wind => AT_WIND,
            Self::Hammer => AT_HAMMER,
            Self::Boomerang => AT_BOOMERANG,
            Self::Helmaroc => AT_HELMAROC,
            Self::Arrow | Self::IceArrow | Self::LightArrow | Self::Other => AT_OTHER,
            Self::Water => AT_JUG,
        }
    }

    fn power(self, blade: u8, soup: bool) -> i32 {
        match self {
            Self::Sword => damage::sword_power(blade, soup),
            Self::Arrow | Self::FireArrow | Self::IceArrow | Self::LightArrow => damage::ARROW,
            Self::Bomb => damage::BOMB,
            Self::Boomerang => damage::BOMERANG,
            Self::Tsubo => damage::TSUBO,
            Self::TsuboJug => damage::WATER_JUG,
            Self::Lance => {
                if damage::ARROW > LANCE_POWER_MAX {
                    LANCE_POWER_MAX
                } else {
                    damage::ARROW
                }
            }
            Self::Hammer => hammer::AT_POWER,
            Self::Idle
            | Self::Hookshot
            | Self::Wind
            | Self::Water
            | Self::Helmaroc
            | Self::Other => 0,
        }
    }

    fn spl(self) -> u8 {
        match self {
            Self::Arrow | Self::FireArrow | Self::IceArrow | Self::LightArrow => 0xB,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blow {
    pub at: At,
    pub blade: u8,
    pub soup: bool,
}

fn frame_max(bck: i32) -> i16 {
    match bck {
        BCK_ATTACK1 => BCK_ATTACK1_FRAMES,
        BCK_ATTACK_T1 => BCK_ATTACK_T1_FRAMES,
        BCK_ATTACK_T2 => BCK_ATTACK_T2_FRAMES,
        BCK_ATTACK_T3 => BCK_ATTACK_T3_FRAMES,
        BCK_DAMAGE1 => BCK_DAMAGE1_FRAMES,
        BCK_DAMAGE2 => BCK_DAMAGE2_FRAMES,
        BCK_DAMAGE3 => BCK_DAMAGE3_FRAMES,
        BCK_DAMAGE4 => BCK_DAMAGE4_FRAMES,
        BCK_DAMAGE_T1 => BCK_DAMAGE_T1_FRAMES,
        BCK_DEATH1 => BCK_DEATH1_FRAMES,
        BCK_DEATH2 => BCK_DEATH2_FRAMES,
        BCK_FLY1 => BCK_FLY1_FRAMES,
        BCK_FLY2 => BCK_FLY2_FRAMES,
        BCK_FLY3 => BCK_FLY3_FRAMES,
        BCK_JIDANDA1 => BCK_JIDANDA1_FRAMES,
        BCK_JUMP1 => BCK_JUMP1_FRAMES,
        BCK_KAKKU1 => BCK_KAKKU1_FRAMES,
        BCK_LAST_DAMAGE1 | BCK_LAST_DAMAGE2 | BCK_LAST_DAMAGE3 => BCK_LAST_DAMAGE_FRAMES,
        BCK_NUKENAI1 => BCK_NUKENAI1_FRAMES,
        BCK_NUKU1 => BCK_NUKU1_FRAMES,
        BCK_PIKUPIKU1 => BCK_PIKUPIKU1_FRAMES,
        BCK_RAKKA_T1 => BCK_RAKKA_T1_FRAMES,
        BCK_SENKAI_L1 => BCK_SENKAI_L1_FRAMES,
        BCK_SENKAI_R1 => BCK_SENKAI_R1_FRAMES,
        BCK_S_DEMO1 => BCK_S_DEMO1_FRAMES,
        BCK_S_DEMO2 => BCK_S_DEMO2_FRAMES,
        BCK_TOBITATU1 => BCK_TOBITATU1_FRAMES,
        BCK_TOBITATU_T1 => BCK_TOBITATU_T1_FRAMES,
        BCK_TO_DAMAGE_T1 => BCK_TO_DAMAGE_T1_FRAMES,
        BCK_TYAKUTI1 => BCK_TYAKUTI1_FRAMES,
        BCK_TYAKUTI_T1 => BCK_TYAKUTI_T1_FRAMES,
        BCK_WAIT1 => BCK_WAIT1_FRAMES,
        BCK_WAIT_T1 => BCK_WAIT_T1_FRAMES,
        BCK_WALK1 => BCK_WALK1_FRAMES,
        _ => BCK_WAIT1_FRAMES,
    }
}

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = f32::from(seed[0]) / RND_SCALE[0]
        + f32::from(seed[1]) / RND_SCALE[1]
        + f32::from(seed[2]) / RND_SCALE[2];
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

fn turned(yaw: i16, offset: [f32; 3]) -> Vec3 {
    let (sin, cos) = from_s16(yaw).sin_cos();
    Vec3::new(
        offset[0] * cos + offset[2] * sin,
        offset[1],
        offset[2] * cos - offset[0] * sin,
    )
}

fn bearing(from: Vec3, to: Vec3) -> i16 {
    to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn flat(from: Vec3, to: Vec3) -> f32 {
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    (dx * dx + dz * dz).sqrt()
}

fn land_area_check(pos: Vec3, radius: f32) -> bool {
    let x = pos.x - AREA_X;
    let z = pos.z - AREA_Z;
    (x * x + z * z).sqrt() < radius
}

fn touches(centre: Vec3, radius: f32, at: Vec3, reach: f32) -> bool {
    (at - centre).length_squared() <= (radius + reach) * (radius + reach)
}

fn timer(value: &mut i16) -> bool {
    if *value != 0 {
        *value -= 1;
    }
    *value == 0
}

fn timer8(value: &mut u8) -> bool {
    if *value != 0 {
        *value -= 1;
    }
    *value == 0
}