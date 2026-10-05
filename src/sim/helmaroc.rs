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
use super::walk::{chase_f, from_s16, to_s16};

pub const HELMAROC_ACTOR: &str = "Bdk";
pub const BDK_OBJ_ACTOR: &str = "Bdkobj";

pub fn is_helmaroc(name: &str) -> bool {
    name == HELMAROC_ACTOR
}

const ANGLE_X: usize = 0;
const ANGLE_Y: usize = 1;
const ANGLE_Z: usize = 2;

const PRM_BYTE_MASK: u32 = 0xF;
const PRM_MASK_SET: u32 = 0xF;
const PRM_NIBBLE_TWO: u32 = 2;
const SWITCH_TOGE: u8 = 0x80;
const EVENT_BIT_LAST_DAMAGE: u16 = 0x3C01;
const TMP_BIT_SHUTTER: u32 = 0x0401;

const HIO_M005: u8 = 0;
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
const REG0_F12: f32 = 0.0;
const REG0_F15: f32 = 0.0;
const REG0_F5: f32 = 0.0;
const REG0_F6: f32 = 0.0;
const REG0_F13: f32 = 0.0;
const REG0_F16: f32 = 0.0;
const REG0_S0: i16 = 0;
const REG0_S1: i16 = 0;
const REG0_S3: i16 = 0;
const REG0_S4: i16 = 0;
const REG0_S5: i16 = 0;
const REG0_S6: i16 = 0;
const REG0_S7: i16 = 0;
const REG0_S8: i16 = 0;

const REG6_F2: f32 = 0.0;
const REG6_F11: f32 = 0.0;
const REG6_F12: f32 = 0.0;
const REG6_F13: f32 = 0.0;
const REG6_F16: f32 = 0.0;
const REG6_F17: f32 = 0.0;
const REG6_F18: f32 = 0.0;
const REG6_F19: f32 = 0.0;
const REG6_S1: i16 = 0;
const REG6_S2: i16 = 0;
const REG6_S3: i16 = 0;
const REG6_S4: i16 = 0;
const REG6_S7: i16 = 0;

const REG8_F2: f32 = 0.0;
const REG8_F6: f32 = 0.0;
const REG8_F7: f32 = 0.0;
const REG8_F8: f32 = 0.0;
const REG8_F9: f32 = 0.0;
const REG8_F10: f32 = 0.0;
const REG8_F11: f32 = 0.0;
const REG8_F14: f32 = 0.0;
const REG8_F15: f32 = 0.0;
const REG8_F16: f32 = 0.0;
const REG8_F18: f32 = 0.0;
const REG8_F19: f32 = 0.0;
const REG8_S4: i16 = 0;
const REG8_S5: i16 = 0;
const REG8_S8: i16 = 0;

const REG12_F4: f32 = 0.0;
const REG12_F5: f32 = 0.0;
const REG12_F6: f32 = 0.0;
const REG12_F7: f32 = 0.0;
const REG12_S7: i16 = 0;

const REG13_F0: f32 = 0.0;
const REG13_F1: f32 = 0.0;
const REG13_F2: f32 = 0.0;
const REG13_F3: f32 = 0.0;
const REG13_F4: f32 = 0.0;
const REG13_F5: f32 = 0.0;
const REG13_F11: f32 = 0.0;
const REG13_F17: f32 = 0.0;
const REG13_F18: f32 = 0.0;
const REG13_S1: i16 = 0;
const REG13_S2: i16 = 0;
const REG13_S6: i16 = 0;
const REG13_S7: i16 = 0;
const REG13_S8: i16 = 0;

const REG14_F2: f32 = 0.0;
const REG14_F4: f32 = 0.0;
const REG14_S9: i16 = 0;


const AREA_X: f32 = 3600.0;
const AREA_Y: f32 = 9800.0;
const AREA_Z: f32 = -3800.0;

pub const CULL_MIN: [f32; 3] = [-650.0, -20.0, -400.0];
pub const CULL_MAX: [f32; 3] = [650.0, 790.0, 320.0];

const GRAVITY: f32 = -5.0;
const GRAVITY_FALL: f32 = -5.0;
const MAX_HEALTH: i32 = 20;
const HAMMER_HEALTH: i32 = 0x14;
const AT_POWER_HEAD: u8 = 2;
const AT_POWER_FOOT: u8 = 2;

const HEAD_OFFSET: [f32; 3] = [40.0, 0.0, -70.0];
const BEAK_OFFSET: [f32; 3] = [120.0, 0.0, 0.0];
const BEAK_TIP_OFFSET: [f32; 3] = [0.0, 0.0, 0.0];
const EYE_OFFSET: [f32; 3] = [120.0, 0.0, -120.0];

const BODY_LIFT: f32 = 380.0;
const HEAD_RADIUS_MASKED: f32 = 110.0;
const HEAD_RADIUS_BARE: f32 = 80.0;
const BEAK_RADIUS: f32 = 70.0;
const BODY_RADIUS: f32 = 250.0;
const FOOT_RADIUS: f32 = 60.0;
const FOOT_X: f32 = 50.0;
const FOOT_TALL_X: f32 = 100.0;
const FOOT_TALL_Y: f32 = 200.0;
const FOOT_RADIUS_T_FLY: f32 = 300.0;
const TOSAKA_RADIUS: f32 = 80.0;
const WIND_RADIUS: f32 = 300.0;
const WIND_RADIUS_BATTLE: f32 = 600.0;
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
const AT_LANCE: u8 = 7;
const AT_WIND: u8 = 8;
const AT_HAMMER: u8 = 9;
const AT_BOOMERANG: u8 = 10;
const AT_HELMAROC: u8 = 11;
const AT_OTHER: u8 = 12;
const LANCE_POWER_MAX: i32 = 2;
const REACH_SWORD: f32 = 20.0;
const REACH_ARROW: f32 = 35.0;
const REACH_BOOMERANG: f32 = 90.0;
const REACH_HAMMER: f32 = 50.0;
const REACH_HOOKSHOT: f32 = 5.0;
const REACH_BOMB: f32 = 150.0;
const REACH_TSUBO: f32 = 30.0;
const PLAYER_BODY_RADIUS: f32 = 20.0;

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
const SE_LK_SW_CRT_HIT: u32 = 0x2806;
const SE_LK_LAST_HIT: u32 = 0x2828;
const SE_LK_W_WEP_CRT_HIT: u32 = 0x2835;
const SE_LK_MS_WEP_CRT_HIT: u32 = 0x2836;
const SE_LK_HS_SPIKE: u32 = 0x286F;
const SE_LK_ARROW_HIT: u32 = 0x2879;
const SE_ATM_MJT_JINARI: u32 = 0x7020;
const SE_ATM_MJT_SHUTTER: u32 = 0x7021;
const SE_ATM_MJT_SHUTTER_END: u32 = 0x7822;
const SE_OBJ_TOGE_IN: u32 = 0x6977;

const BGM_DK_BATTLE: u32 = 0x8000_0014;
const BGM_MJ_TOWER_BATTLE: u32 = 0x8000_0015;
const BGM_UNK_110: u32 = 0x8000_0110;
const STRM_DK_START: u32 = 0xC000_0008;
const STRM_DK_CLEAR: u32 = 0xC000_001A;

const KAMEN_BREAK_OFF_X: [f32; 4] = [50.0, 30.0, -50.0, -30.0];
const KAMEN_BREAK_OFF_Y: [f32; 4] = [0.0, -120.0, 20.0, 0.0];
const KAMEN_BREAK_OFF_Z: [f32; 4] = [50.0, 20.0, 50.0, 0.0];
const KAMEN_BREAK_SD_X: [f32; 4] = [-5.0, 0.0, 5.0, 0.0];
const KAMEN_BREAK_SD_Z: [f32; 4] = [0.0, 5.0, 0.0, 0.0];
const KAMEN_BREAK_TIME: [i8; 4] = [0x1E, 0x2D, 0x28, 0x23];
const KAMEN_BREAK_Z1: [i16; 4] = [0x2000, 0x2000, -0x2000, -0x2000];
const KAMEN_BREAK_Z2: [i16; 4] = [-0x1000, -0x1000, 0x1000, 0x1000];

const EFF_COUNT: usize = 40;
const EFF_ROCK: usize = 0x27;
const WIND_COUNT: usize = 10;
const PIECE_COUNT: usize = 4;
const PIECE_FLOOR: f32 = 8000.0;
const PIECE_GROUND_LIFT: f32 = 15.0;
const PIECE_SPIN: f32 = 3000.0;
const PIECE_SPIN_STEP: i32 = 0x2000;
const PIECE_SPIN_SPREAD: i32 = 0x3000;
const PIECE_BOUNCE_CAP: f32 = -30.0;
const PIECE_PUSH_STEP: f32 = 0.2;
const SHUTTER_BATTLE_Y: f32 = -500.0;
const SHUTTER_X: f32 = 4600.0 + 155.0;
const SHUTTER_Z: f32 = 4700.0;
const GROCK_DROP: f32 = -10.0;
const EFF_FALL: f32 = -130.0;
const EFF_RISE_CAP: f32 = 2.0;
const EFF_RISE: f32 = 10.0;
const EFF_UP: f32 = 15000.0;
const EFF_CAP: f32 = 1000.0;
const EFF_UP_CALM: f32 = 8000.0;
const EFF_CAP_CALM: f32 = 200.0;
const EFF_CEIL: f32 = 9930.0;
const EFF_FLOOR: f32 = 9700.0;
const EFF_AREA: f32 = 3000.0;
const EFF_GROUND_LIFT: f32 = 20.0;
const EFF_RISE_STEP: f32 = 0.1;
const EFF_FLOOR_CLAMP: f32 = -5.0;
const TIMER_COUNT: usize = 5;
const SMOKE_COUNT: usize = 4;
const FOOT_COUNT: usize = 2;
const EFFECT_LIFE: i16 = 2000;
const EFFECT_LIFE_LIMIT: u8 = 0x26;
const LANDING_AREA: f32 = 1400.0;
const LANDING_LIFT: f32 = 600.0;
const APPROACH: f32 = 1300.0;
const APPROACH_STOP: f32 = 300.0;
const VIEW_LIMIT: u16 = 0x6000;
const WALK_LIMIT: i16 = 0x3000;
const SENKAI_LIMIT: i16 = 0x800;
const TRACK_LIMIT: i16 = 7000;
const TURN_DIV_WALK: i16 = 2;
const TURN_DIV_AIM: i16 = 8;
const TURN_DIV_FACE: i16 = 4;
const TURN_DIV_CAM: i16 = 0x10;
const TURN_DIV_HEAD: i16 = 4;
const TURN_DIV_NECK: i16 = 0x10;
const BEAK_SWING: f32 = -200.0;
const BEAK_WOBBLE: f32 = -2000.0;
const CAM_NEAR_X: f32 = -50.0;
const TURN_CAP: f32 = 0x800;
const TURN_BANK_I32: i32 = 5500;
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
    pub rise: f32,
    pub sway_rate: f32,
    pub spin: [i16; 3],
    pub tumble: [i16; 3],
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
            rise: 0.0,
            sway_rate: 0.0,
            spin: [0; 3],
            tumble: [0; 3],
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
        let start = (self.start) as f32;
        let end = (self.end) as f32;
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
                    self.frame += (low) as f32;
                }
                while self.frame >= end {
                    self.state |= STATE_LOOP;
                    if high <= 0 {
                        break;
                    }
                    self.frame -= (high) as f32;
                }
            }
            _ => {}
        }
    }

    fn play(&mut self) {
        if self.cur_morf < 1.0 {
            self.prev_morf = self.cur_morf;
            chase_f(&mut self.cur_morf, 1.0, self.morf_step);
        }
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

    fn reach(self) -> f32 {
        match self {
            Self::Sword => REACH_SWORD,
            Self::Arrow | Self::FireArrow | Self::IceArrow | Self::LightArrow => REACH_ARROW,
            Self::Boomerang => REACH_BOOMERANG,
            Self::Hammer => REACH_HAMMER,
            Self::Hookshot => REACH_HOOKSHOT,
            Self::Bomb => REACH_BOMB,
            Self::Tsubo | Self::TsuboJug => REACH_TSUBO,
            _ => 0.0,
        }
    }

    fn at_se(self) -> u32 {
        match self {
            Self::Hammer => SE_LK_HAMMER_HIT,
            Self::Arrow | Self::FireArrow | Self::IceArrow | Self::LightArrow => SE_LK_ARROW_HIT,
            Self::Hookshot => SE_LK_HS_SPIKE,
            Self::Bomb => SE_LK_MS_WEP_CRT_HIT,
            Self::Boomerang => SE_LK_W_WEP_CRT_HIT,
            _ => SE_LK_SW_CRT_HIT,
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
pub struct Env<'a> {
    pub player_pos: Vec3,
    pub player_angle: i16,
    pub player_hover: bool,
    pub solid: Option<&'a Collision>,
    pub blow: Blow,
    pub peck: Option<(Entity, Vec3)>,
}

impl Env<'_> {
    fn ground_h(&self, pos: Vec3) -> f32 {
        match self.solid {
            Some(solid) => solid.ground_at(pos.x, pos.z).unwrap_or(0.0),
            None => 0.0,
        }
    }

    fn player_bearing(&self, pos: Vec3) -> i16 {
        bearing(pos, self.player_pos)
    }

    fn player_distance(&self, pos: Vec3) -> f32 {
        (self.player_pos - pos).length()
    }

    fn corrected(&self, from: Vec3, to: Vec3) -> (Vec3, bool) {
        match self.solid {
            Some(solid) => {
                let next = solid.slide(from, to);
                let hit = next.x != to.x || next.z != to.z;
                (next, hit)
            }
            None => (to, false),
        }
    }

    fn landed(&self, pos: Vec3) -> bool {
        pos.y <= self.ground_h(pos) + 1.0
    }

    fn crossed(&self, from: Vec3, to: Vec3) -> bool {
        match self.solid {
            Some(solid) => solid.slide(from, to) != to,
            None => false,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Orders {
    pub purge_mobs: bool,
    pub purge_shards: bool,
}

pub struct Sink<'a> {
    pub events: &'a mut SimEvents,
    pub switches: &'a mut Switches,
    pub stories: &'a mut StoryBits,
    pub flags: &'a mut EventFlags,
    pub zones: &'a mut ZoneActors,
    pub health: &'a mut Health,
    pub orders: &'a mut Orders,
    pub drop: &'a mut Vec<(Entity, i8, u16)>,
}

impl Sink<'_> {
    fn se(&mut self, id: u32) {
        self.events.list.push(SimEvent::Se(id));
    }

    fn bgm(&mut self, id: u32) {
        self.events.list.push(SimEvent::Bgm(id));
    }

    fn stop_bgm(&mut self) {
        self.events.list.push(SimEvent::Bgm(0));
    }

    fn clear_mobs(&mut self) {
        self.orders.purge_mobs = true;
    }

    fn purge_shards(&mut self) {
        self.orders.purge_shards = true;
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Helmaroc {
    pub battle_mode: u8,
    pub morph: Frame,
    pub frame_count: i32,
    pub action: Action,
    pub state: i16,
    pub fly_ca: i16,
    pub loop_kind: u8,
    pub target: Vec3,
    pub target_yaw: i16,
    pub turn_scale: f32,
    pub turn_step: f32,
    pub speed_f: f32,
    pub speed_step: f32,
    pub timers: [i16; TIMER_COUNT],
    pub tail_hold: i16,
    pub hit_stop: u8,
    pub mask_timer: u8,
    pub mask_damage: u8,
    pub at_mode: u8,
    pub foot_at_live: u8,
    pub ground_hit: i8,
    pub resume_bck: i32,
    pub resume_bas: i32,
    pub resume_loop: u8,
    pub resume_pending: u8,
    pub sit_count: i8,
    pub swoop_count: i8,
    pub beat_count: i8,
    pub peck_obj: Option<Entity>,
    pub blure_rate: i16,
    pub demo_state: i16,
    pub flown_high: i8,
    pub demo_count: i16,
    pub demo_sub: i16,
    pub cam_look: Vec3,
    pub cam_eye: Vec3,
    pub cam_yaw: i16,
    pub fov: f32,
    pub demo_param: f32,
    pub demo_radius: f32,
    pub shake: f32,
    pub demo_flag: i8,
    pub demo_ordered: bool,
    pub wind_power: f32,
    pub wind_scale: Vec3,
    pub wind_dir: Vec3,
    pub rise_smoke: u8,
    pub nail_live: u8,
    pub smoke_pos: Vec3,
    pub effects: [Eff; EFF_COUNT],
    pub effect_cursor: u8,
    pub smoke_timer: [i16; SMOKE_COUNT],
    pub shutter_y: f32,
    pub shutter_yaw: f32,
    pub shutter_roll: f32,
    pub shutter_ring: f32,
    pub shutter_shift: f32,
    pub track_player: bool,
    pub head_pitch: i16,
    pub head_yaw: i16,
    pub swoop_offset: f32,
    pub tosaka_y1: i16,
    pub tosaka_z1: i16,
    pub tosaka_y2: i16,
    pub tosaka_z2: i16,
    pub tosaka_y3: i16,
    pub tosaka_z3: i16,
    pub tosaka_y4: i16,
    pub tosaka_z4: i16,
    pub beak_tick: i16,
    pub beak_sway: i16,
    pub beak_amp: i16,
    pub beak_aim: i16,
    pub beak_timer: i16,
    pub beak_free: i8,
    pub beak_calc: f32,
    pub beak_sway_calc: f32,
    pub head_prev: Vec3,
    pub head: Vec3,
    pub head_radius: f32,
    pub beak: Vec3,
    pub beak_radius: f32,
    pub tosaka: Vec3,
    pub tosaka_radius: f32,
    pub body: Vec3,
    pub body_radius: f32,
    pub foot: [Vec3; FOOT_COUNT],
    pub foot_radius: [f32; FOOT_COUNT],
    pub foot_at: [bool; FOOT_COUNT],
    pub foot_tg: [bool; FOOT_COUNT],
    pub foot_split: [bool; FOOT_COUNT],
    pub wind_active: [u8; WIND_COUNT],
    pub wind_pos: [Vec3; WIND_COUNT],
    pub wind_move: [Vec3; WIND_COUNT],
    pub wind_radius: [f32; WIND_COUNT],
    pub wind_anchor: Vec3,
    pub piece_live: [i8; PIECE_COUNT],
    pub piece_pos: [Vec3; PIECE_COUNT],
    pub piece_prev: [Vec3; PIECE_COUNT],
    pub piece_move: [Vec3; PIECE_COUNT],
    pub piece_rot: [[i16; 3]; PIECE_COUNT],
    pub piece_sway: [[i16; 3]; PIECE_COUNT],
    pub piece_push: [[i16; 3]; PIECE_COUNT],
    pub piece_hold: [i8; PIECE_COUNT],
    pub piece_roll: [f32; PIECE_COUNT],
    pub piece_speed: [f32; PIECE_COUNT],
    pub piece_angle: [i16; PIECE_COUNT],
    pub piece_sway_speed: [f32; PIECE_COUNT],
    pub health: i32,
    pub max_health: i32,
    pub stage_boss_enemy: bool,
    pub stage_boss_demo: bool,
    pub speed: Vec3,
    pub speed_f_walk: f32,
    pub gravity: f32,
    pub eye: Vec3,
    pub shape_angle: [i16; 3],
    pub seed: [i32; 3],
}

impl Helmaroc {
    pub fn spawn(parameters: u32) -> Self {
        let _param_byte = parameters & 0xFF;
        let mut morph = Frame::idle();
        morph.init(frame_max(BCK_FLY1));
        morph.attribute = EMODE_LOOP;
        morph.set_loop(0.0);
        morph.set_morf(0.0);
        Self {
            battle_mode: 1,
            morph,
            frame_count: 0,
            action: Action::TFly,
            state: 0,
            fly_ca: -1,
            loop_kind: 0,
            target: Vec3::ZERO,
            target_yaw: 0,
            turn_scale: 0.0,
            turn_step: 0.0,
            speed_f: 0.0,
            speed_step: 0.0,
            timers: [0; TIMER_COUNT],
            tail_hold: 0,
            hit_stop: 0,
            mask_timer: 0,
            mask_damage: 0,
            at_mode: 0,
            foot_at_live: 0,
            ground_hit: 0,
            resume_bck: 0,
            resume_bas: 0,
            resume_loop: 0,
            resume_pending: 0,
            sit_count: 0,
            swoop_count: 0,
            beat_count: 0,
            peck_obj: None,
            blure_rate: 0,
            demo_state: 0,
            flown_high: 0,
            demo_count: 0,
            demo_sub: 0,
            cam_look: Vec3::ZERO,
            cam_eye: Vec3::ZERO,
            cam_yaw: 0,
            fov: 45.0,
            demo_param: 0.0,
            demo_radius: 0.0,
            shake: 0.0,
            demo_flag: 0,
            demo_ordered: false,
            wind_power: 0.0,
            wind_scale: Vec3::ZERO,
            wind_dir: Vec3::ZERO,
            rise_smoke: 0,
            nail_live: 0,
            smoke_pos: Vec3::ZERO,
            effects: [Eff::idle(); EFF_COUNT],
            effect_cursor: 0,
            smoke_timer: [0; SMOKE_COUNT],
            shutter_y: 0.0,
            shutter_yaw: 0.0,
            shutter_roll: 0.0,
            shutter_ring: -630.0,
            shutter_shift: -70.0,
            track_player: false,
            head_pitch: 0,
            head_yaw: 0,
            swoop_offset: 0.0,
            tosaka_y1: -2000,
            tosaka_z1: 0,
            tosaka_y2: 0,
            tosaka_z2: 0,
            tosaka_y3: 0,
            tosaka_z3: 0,
            tosaka_y4: 0,
            tosaka_z4: 0,
            beak_tick: 0,
            beak_sway: 0,
            beak_amp: 0,
            beak_aim: 0,
            beak_timer: 0,
            beak_free: 0,
            beak_calc: 0.0,
            beak_sway_calc: 0.0,
            head_prev: Vec3::ZERO,
            head: Vec3::ZERO,
            head_radius: HEAD_RADIUS_MASKED,
            beak: Vec3::ZERO,
            beak_radius: BEAK_RADIUS,
            tosaka: Vec3::ZERO,
            tosaka_radius: TOSAKA_RADIUS,
            body: Vec3::ZERO,
            body_radius: BODY_RADIUS,
            foot: [Vec3::ZERO; FOOT_COUNT],
            foot_radius: [FOOT_RADIUS; FOOT_COUNT],
            foot_at: [false; FOOT_COUNT],
            foot_tg: [true; FOOT_COUNT],
            foot_split: [false; FOOT_COUNT],
            wind_active: [0; WIND_COUNT],
            wind_pos: [Vec3::ZERO; WIND_COUNT],
            wind_move: [Vec3::ZERO; WIND_COUNT],
            wind_radius: [WIND_RADIUS; WIND_COUNT],
            wind_anchor: Vec3::ZERO,
            piece_live: [0; PIECE_COUNT],
            piece_pos: [Vec3::ZERO; PIECE_COUNT],
            piece_prev: [Vec3::ZERO; PIECE_COUNT],
            piece_move: [Vec3::ZERO; PIECE_COUNT],
            piece_rot: [[0; 3]; PIECE_COUNT],
            piece_sway: [[0; 3]; PIECE_COUNT],
            piece_push: [[0; 3]; PIECE_COUNT],
            piece_hold: [0; PIECE_COUNT],
            piece_roll: [0.0; PIECE_COUNT],
            piece_speed: [0.0; PIECE_COUNT],
            piece_angle: [0; PIECE_COUNT],
            piece_sway_speed: [0.0; PIECE_COUNT],
            health: MAX_HEALTH,
            max_health: MAX_HEALTH,
            stage_boss_enemy: false,
            stage_boss_demo: false,
            speed: Vec3::ZERO,
            speed_f_walk: 0.0,
            gravity: 0.0,
            eye: Vec3::ZERO,
            shape_angle: [0; 3],
            seed: [RND_SEED; 3],
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.action != Action::AfterFight
    }

    pub fn reset_health(&mut self) {
        self.health = MAX_HEALTH;
        self.max_health = MAX_HEALTH;
    }

    fn rnd_f(&mut self, range: f32) -> f32 {
        rnd(&mut self.seed) * range
    }

    fn rnd_fx(&mut self, range: f32) -> f32 {
        range * (rnd(&mut self.seed) - 0.5) * 2.0
    }

    fn anm_init(&mut self, bck: i32, morf: f32, loop_mode: u8, speed: f32, bas: i32, param7: u8) {
        if param7 == 0 {
            self.resume_bck = bck;
            self.resume_bas = bas;
            self.resume_loop = loop_mode;
            self.resume_pending = 0;
        } else {
            self.resume_pending = 1;
        }
        self.morph.init(frame_max(bck));
        self.morph.attribute = loop_mode;
        self.morph.rate = speed;
        self.morph.set_loop(0.0);
        self.morph.set_morf(morf);
    }

    fn frame(&self) -> f32 {
        self.morph.frame
    }

    fn frame_i(&self) -> i32 {
        self.morph.frame as i32
    }

    fn pos_move(&mut self, actor: &mut Actor) {
        let diff = self.target - actor.pos;
        let target_xz = bearing(actor.pos, self.target);
        let target_pitch = -to_s16(diff.y.atan2(flat(actor.pos, self.target)));
        let before = actor.angle[ANGLE_Y];
        add_calc_angle_s2(
            &mut actor.angle[ANGLE_Y],
            target_xz,
            REG0_S3 + 10,
            (self.turn_scale * self.turn_step) as i16,
        );
        let mut bank = (i32::from(before) - i32::from(actor.angle[ANGLE_Y])) as f32 * 0x20;
        let cap = i32::from(REG0_S1) + TURN_BANK_I32;
        if bank > cap {
            bank = cap;
        } else if bank < -cap {
            bank = -cap;
        }
        add_calc_angle_s2(
            &mut actor.angle[ANGLE_Z],
            bank as i16,
            REG0_S3 + 10,
            (self.turn_scale * self.turn_step * 0.5) as i16,
        );
        add_calc_angle_s2(
            &mut actor.angle[ANGLE_X],
            target_pitch,
            REG0_S3 + 10,
            (self.turn_scale * self.turn_step) as i16,
        );
        add_calc2(&mut self.turn_step, SPEED_STEP, SPEED_STEP, TURN_STEP);
        add_calc2(
            &mut actor.speed_f_walk,
            self.speed_f,
            SPEED_STEP,
            self.speed_step,
        );
        let forward = turned(actor.angle[ANGLE_Y], [0.0, 0.0, actor.speed_f_walk]);
        if self.resume_pending == 0 {
            self.speed = forward;
        } else {
            let out = turned(actor.angle[ANGLE_X], forward);
            self.speed.x = out.x;
            self.speed.z = out.z;
            if self.morph.frame < RISE_FRAMES {
                self.speed.y -= REG0_F9 + RISE_SPEED_UP;
            } else {
                self.speed.y += REG0_F10 + RISE_SPEED_DOWN;
            }
        }
        actor.pos += self.speed;
    }

    fn ground_move(&mut self, actor: &mut Actor) {
        add_calc_angle_s2(&mut actor.angle[ANGLE_Z], 0, TURN_DIV_WALK, TURN_CAP as i16);
        add_calc_angle_s2(&mut actor.angle[ANGLE_X], 0, TURN_DIV_WALK, TURN_CAP as i16);
        add_calc2(
            &mut actor.speed_f_walk,
            self.speed_f,
            SPEED_STEP,
            self.speed_step,
        );
        let forward = turned(actor.angle[ANGLE_Y], [0.0, 0.0, actor.speed_f_walk]);
        let out = turned(actor.angle[ANGLE_X], forward);
        self.speed.x = out.x;
        self.speed.z = out.z;
        actor.pos += self.speed;
        self.speed.y += self.gravity;
        add_calc_angle_s2(
            &mut actor.angle[ANGLE_Y],
            bearing(actor.pos, self.target),
            TURN_DIV_WALK,
            (self.turn_scale * self.turn_step) as i16,
        );
        add_calc2(&mut self.turn_step, SPEED_STEP, SPEED_STEP, TURN_STEP_GROUND);
    }

    fn pl_view_check(&self, actor: &Actor, env: &Env<'_>) -> bool {
        let angle = actor.angle[ANGLE_Y].wrapping_sub(env.player_bearing(actor.pos));
        angle.unsigned_abs() < VIEW_LIMIT
    }

    fn eff_hane_set(&mut self, offset: Vec3, count: i32, mode: i8) {
        let mut made = 0;
        let mut index = 0;
        while index < EFF_COUNT - 1 && made < count {
            let slot = usize::from(self.effect_cursor);
            self.effect_cursor = self.effect_cursor.wrapping_add(1);
            if self.effect_cursor > EFFECT_LIFE_LIMIT {
                self.effect_cursor = 0;
            }
            let roll = self.rnd_f(0x10000 as f32) as i32 as i16;
            let z_spin = self.rnd_fx(1200.0 + REG8_F10) as i16;
            let x_spin = self.rnd_fx(800.0 + REG8_F16) as i16;
            let base_yaw = self.shape_angle[ANGLE_Y];
            let (spin_y, speed, sway, chase) = if mode > 0 {
                let spin_y = base_yaw.wrapping_add(self.rnd_fx(5000.0) as i16);
                let speed = REG8_F8 + (self.rnd_f(10.0) + 20.0);
                let sway = REG8_F9 + (self.rnd_fx(5.0) - 10.0);
                let chase = self.rnd_f(20.0) as u8 + 20;
                (spin_y, speed, sway, chase)
            } else {
                let speed = if mode < 0 {
                    self.rnd_f(10.0)
                } else {
                    REG8_F8 + (self.rnd_f(5.0) + 5.0)
                };
                let sway = REG8_F9 + (self.rnd_f(5.0) + 5.0);
                let spin_y = self.rnd_f(0x10000 as f32) as i32 as i16;
                (spin_y, speed, sway, 0)
            };
            let eff = &mut self.effects[slot];
            eff.kind = 1;
            eff.life = EFFECT_LIFE;
            eff.done = 0;
            eff.chase = 0;
            eff.scale = 1.0;
            eff.pos = offset;
            eff.fall = 0.0;
            eff.count = roll;
            eff.tumble[2] = z_spin;
            eff.tumble[0] = x_spin;
            made += 1;
            if mode > 0 {
                eff.spin[1] = spin_y;
                eff.speed = speed;
                eff.rise = sway;
                eff.chase = chase;
            } else {
                eff.speed = speed;
                eff.rise = sway;
                eff.spin[1] = spin_y;
            }
            index += 1;
        }
    }

    fn up_fly(&mut self, actor: &mut Actor) {
        let mut move_pos = false;
        let mut arm = self.state;
        loop {
            match arm {
            0 => {
                self.anm_init(BCK_TOBITATU1, 5.0, EMODE_NONE, 1.0, 0x16, 0);
                self.rise_smoke = 0x19;
                let out = turned(actor.angle[ANGLE_Y], [0.0, 2000.0, 5000.0]);
                self.target = actor.pos + out;
                self.state += 1;
                self.speed_f = 0.0;
                self.speed_step = 1.0;
                actor.speed_f_walk = 0.0;
                break;
            }
            1 => {
                if self.morph.frame == f32::from(REG0_S7 + 0x17) {
                    self.state = 2;
                    actor.speed_f_walk = REG0_F8 + 50.0;
                    self.speed_f = 40.0;
                    self.speed_step = REG0_F9 + 1.0;
                    actor.angle[ANGLE_X] = -0x3000;
                    arm = 2;
                } else {
                    move_pos = true;
                    break;
                }
            }
            2 => {
                self.turn_step = 0.0;
                if self.morph.is_stop() {
                    self.action = Action::Fly;
                    self.fly_ca = -1;
                    self.state = 0;
                }
                break;
            }
            _ => break,
            }
        }
        if !move_pos {
            self.pos_move(actor);
        }
    }

    fn fly(&mut self, actor: &mut Actor) {
        self.track_player = true;
        match self.fly_ca {
            0 => {}
            -1 => {
                self.anm_init(BCK_FLY1, 10.0, EMODE_LOOP, 1.0, 0xB, 0);
                self.fly_ca = 0;
            }
            _ => {}
        }
        self.speed_f = HIO_M020;
        self.speed_step = REG0_F9 + 1.0;
        self.turn_scale = 600.0;
        match self.state {
            0 => {
                if self.loop_kind == 0 {
                    self.loop_kind = self.rnd_f(2.990) as u8;
                } else if self.rnd_f(1.0) < 0.8 {
                    if self.loop_kind == 1 {
                        self.loop_kind = 2;
                    } else if self.loop_kind == 2 {
                        self.loop_kind = 1;
                    }
                } else {
                    self.loop_kind = 0;
                }
                if self.loop_kind == 0 {
                    self.target.x = REG0_F0 + 6500.0;
                    self.target.y = REG0_F1 + 11000.0;
                    self.target.z = REG0_F2 + 2600.0;
                    self.target.x += self.rnd_fx(REG0_F3 + 3000.0);
                    self.target.y = self.target.y + 1000.0;
                    self.target.z += self.rnd_fx(REG0_F3 + 3000.0);
                } else if self.loop_kind == 1 {
                    self.target.x = REG0_F0 + 9800.0;
                    self.target.y = REG0_F1 + 12000.0;
                    self.target.z = REG0_F2 - 1500.0;
                    self.target.x += self.rnd_fx(REG0_F3 + 1000.0);
                    self.target.y += self.rnd_f(1000.0);
                    self.target.z += self.rnd_fx(REG0_F3 + 1000.0);
                } else if self.loop_kind == 2 {
                    self.target.x = REG0_F0 - 1000.0;
                    self.target.y = REG0_F1 + 12000.0;
                    self.target.z = REG0_F2 - 2900.0;
                    self.target.x += self.rnd_fx(REG0_F3 + 1000.0);
                    self.target.y += self.rnd_f(1000.0);
                    self.target.z += self.rnd_fx(REG0_F3 + 1000.0);
                }
                self.turn_step = 0.0;
                if (self.target - actor.pos).length() > 1000.0 {
                    self.state += 1;
                    self.timers[0] = 0;
                }
            }
            1 => {
                if (self.target - actor.pos).length() < 1500.0 {
                    if self.loop_kind == 0
                        || self.rnd_f(1.0) < 0.3
                        || self.swoop_count >= 4
                        || self.mask_damage < 2
                    {
                        self.action = Action::Landing;
                        self.swoop_count = 0;
                        self.state = 0;
                    } else {
                        self.action = Action::FlyAttack;
                        self.state = 0;
                        self.swoop_count += 1;
                    }
                }
            }
            _ => {}
        }
        self.pos_move(actor);
    }

    fn landing(&mut self, actor: &mut Actor, env: &Env<'_>) {
        let mut move_pos = false;
        let mut feathers = 0;
        self.track_player = true;
        let mut arm = self.state;
        loop {
            match arm {
                -2 | 0 => {
                    let angle = env.player_bearing(actor.pos);
                    let back = if self.state == 0 { -800.0 } else { 0.0 };
                    let out = turned(angle, [0.0, 0.0, back]);
                    self.target = env.player_pos + out;
                    if land_area_check(self.target, LANDING_AREA) {
                        self.target.y += REG0_F2 + LANDING_LIFT;
                        self.state = 1;
                        self.turn_scale = 1000.0;
                        self.turn_step = 0.0;
                        self.timers[0] = 50;
                        arm = 1;
                    } else {
                        self.action = Action::Fly;
                        self.state = 0;
                        break;
                    }
                }
                1 => {
                    if (self.target - actor.pos).length() < APPROACH {
                        self.state = 2;
                        self.anm_init(BCK_TYAKUTI1, 15.0, EMODE_NONE, 0.001, 0x17, 0);
                        self.turn_step = 1.0;
                        move_pos = true;
                    } else if self.timers[0] == 1 {
                        self.anm_init(BCK_KAKKU1, 30.0, EMODE_LOOP, 1.0, BAS_NONE, 0);
                    }
                    break;
                }
                2 => {
                    move_pos = true;
                    add_calc2(
                        &mut actor.pos.x,
                        self.target.x,
                        1.0,
                        self.speed.x.abs() * self.turn_step,
                    );
                    add_calc2(
                        &mut actor.pos.y,
                        self.target.y,
                        1.0,
                        self.speed.y.abs() * self.turn_step,
                    );
                    add_calc2(
                        &mut actor.pos.z,
                        self.target.z,
                        1.0,
                        self.speed.z.abs() * self.turn_step,
                    );
                    add_calc_angle_s2(
                        &mut actor.angle[ANGLE_Z],
                        0,
                        TURN_DIV_WALK,
                        TURN_CAP as i16,
                    );
                    add_calc_angle_s2(
                        &mut actor.angle[ANGLE_X],
                        0,
                        TURN_DIV_WALK,
                        TURN_CAP as i16,
                    );
                    if (self.target - actor.pos).length() < REG0_F3 + APPROACH_STOP {
                        self.state = 3;
                    }
                    break;
                }
                3 | 4 => {
                    move_pos = true;
                    feathers = 3;
                    actor.pos.y += self.speed.y;
                    self.speed.y += self.gravity;
                    add_calc2(
                        &mut actor.pos.x,
                        self.target.x,
                        0.2,
                        self.speed.x.abs() * self.turn_step,
                    );
                    add_calc2(
                        &mut actor.pos.z,
                        self.target.z,
                        0.2,
                        self.speed.z.abs() * self.turn_step,
                    );
                    add_calc_angle_s2(
                        &mut actor.angle[ANGLE_Z],
                        0,
                        TURN_DIV_WALK,
                        TURN_CAP as i16,
                    );
                    add_calc_angle_s2(
                        &mut actor.angle[ANGLE_X],
                        0,
                        TURN_DIV_WALK,
                        TURN_CAP as i16,
                    );
                    let ground = env.ground_h(actor.pos);
                    if self.ground_hit != 0 || actor.pos.y <= ground + 1.0 {
                        actor.speed_f_walk = 0.0;
                        self.speed.z = 0.0;
                        self.speed.x = 0.0;
                        self.speed.y = -10.0;
                        if self.state == 3 {
                            self.anm_init(BCK_TYAKUTI1, 5.0, EMODE_NONE, 1.0, 0x17, 0);
                            self.state = 4;
                            feathers = 10;
                        }
                    }
                    if self.morph.is_stop() {
                        self.action = Action::Wait;
                        self.state = 0;
                        self.timers[3] = self.rnd_f(600.0) as i16 + 600;
                    }
                    self.at_mode = 2;
                    break;
                }
                _ => break,
            }
        }
        if !move_pos {
            self.pos_move(actor);
        }
        if feathers != 0 {
            let mask = self.frame_count;
            if feathers <= 7 {
                if mask & feathers == 0 {
                    self.eff_hane_set(self.body, 1, 0);
                }
            } else {
                self.eff_hane_set(self.body, feathers, 0);
            }
        }
    }

    fn wait(&mut self, actor: &mut Actor, env: &Env<'_>) {
        let dist = env.player_distance(actor.pos);
        self.track_player = true;
        self.target = env.player_pos;
        let angle = actor
            .angle[ANGLE_Y]
            .wrapping_sub(env.player_bearing(actor.pos));
        self.speed_f = 0.0;
        self.speed_step = 5.0;
        self.turn_scale = 500.0;
        let view = self.pl_view_check(actor, env);
        let mut arm = self.state;
        loop {
            match arm {
                0 => {
                    if i32::from(self.beat_count) >= 5
                        || (i32::from(self.sit_count) >= 2 && self.mask_damage < 4)
                        || !view
                        || self.timers[3] == 0
                    {
                        self.action = Action::UpFly;
                        self.beat_count = 0;
                        self.sit_count = 0;
                        break;
                    }
                    self.anm_init(BCK_WAIT1, 10.0, EMODE_LOOP, 1.0, BAS_NONE, 0);
                    self.state = 1;
                    self.timers[0] = self.rnd_f(30.0) as i16 + 40;
                    self.fly_ca = 0;
                    actor.speed_f_walk = 0.0;
                    arm = 1;
                }
                1 => {
                    if angle > WALK_LIMIT || angle < -WALK_LIMIT {
                        self.state = 5;
                        break;
                    }
                    if self.timers[1] == 0 {
                        if !view {
                            self.state = 0;
                            break;
                        }
                        if angle > SENKAI_LIMIT {
                            if self.fly_ca != 1 {
                                self.fly_ca = 1;
                                self.anm_init(BCK_SENKAI_L1, 10.0, EMODE_LOOP, 1.0, 0x13, 0);
                                self.timers[1] = 0x14;
                            }
                        } else if angle < -SENKAI_LIMIT {
                            if self.fly_ca != 2 {
                                self.fly_ca = 2;
                                self.anm_init(BCK_SENKAI_R1, 10.0, EMODE_LOOP, 1.0, 0x14, 0);
                                self.timers[1] = 0x14;
                            }
                        } else if self.fly_ca != 3 {
                            self.fly_ca = 3;
                            self.anm_init(BCK_WAIT1, 10.0, EMODE_LOOP, 1.0, BAS_NONE, 0);
                            self.timers[1] = 0x14;
                        }
                    }
                    if self.fly_ca != 3 {
                        self.turn_scale = REG0_F0 + 800.0;
                    }
                    if self.timers[0] == 0 && self.fly_ca == 3 {
                        if HIO_M014 != 0 {
                            if HIO_M014 == 1 {
                                self.action = Action::KutiAttack;
                            } else if HIO_M014 == 2 {
                                self.action = Action::WindAttack;
                            }
                            self.state = 0;
                        } else {
                            self.sit_count += 1;
                            if dist < REG0_F11 + 450.0 {
                                self.action = Action::JidaAttack;
                                self.state = 0;
                            } else if dist < REG0_F11 + 550.0 {
                                if self.mask_damage < 2 {
                                    self.action = Action::KutiAttack;
                                } else if self.rnd_f(1.0) < 0.25 {
                                    self.action = Action::WindAttack;
                                } else {
                                    self.action = Action::KutiAttack;
                                }
                                self.state = 0;
                            } else if dist < REG0_F15 + 1300.0 {
                                self.state = 2;
                                self.anm_init(BCK_WALK1, 3.0, EMODE_LOOP, HIO_M01C, 0x19, 0);
                                self.turn_step = 0.0;
                                actor.speed_f_walk = 0.0;
                            } else if self.rnd_f(1.0) < 0.5 {
                                self.action = Action::Jump;
                                self.state = 0;
                            } else {
                                self.action = Action::WindAttack;
                                self.state = 0;
                            }
                        }
                    }
                    break;
                }
                2 => {
                    self.speed_f = HIO_M018;
                    self.speed_step = 1.0;
                    if dist < REG0_F12 + 530.0 || dist > 1350.0 {
                        self.state = 1;
                    }
                    break;
                }
                5 => {
                    self.anm_init(BCK_JUMP1, 5.0, EMODE_NONE, 0.0, 0xF, 0);
                    self.state = 6;
                    self.speed.y = REG8_F9 + 50.0;
                    actor.speed_f_walk = 0.0;
                    arm = 6;
                }
                6 => {
                    self.turn_scale = 700.0;
                    if self.speed.y < 0.0 && actor.pos.y < env.ground_h(actor.pos) + 5.0 {
                        self.anm_init(BCK_JUMP1, 2.0, EMODE_NONE, 1.0, 0xF, 0);
                        self.state = 7;
                    }
                    break;
                }
                7 => {
                    self.turn_scale = 0.0;
                    if self.morph.is_stop() {
                        self.state = 1;
                    }
                    break;
                }
                _ => break,
            }
        }
        self.ground_move(actor);
    }

    fn jump(&mut self, actor: &mut Actor, env: &Env<'_>) {
        self.track_player = true;
        self.speed_f = 0.0;
        self.speed_step = 5.0;
        self.turn_scale = 500.0;
        let mut arm = self.state;
        loop {
            match arm {
                0 => {
                    self.anm_init(BCK_TOBITATU1, 5.0, EMODE_NONE, 1.0, 0x16, 0);
                    self.rise_smoke = 0x19;
                    self.state = 1;
                    self.target = env.player_pos;
                    break;
                }
                1 => {
                    if self.morph.frame != f32::from(REG0_S7 + 0x17) {
                        break;
                    }
                    self.speed.y = REG8_F9 + 50.0;
                    self.state = 2;
                    self.ground_hit = 0;
                    let out = turned(actor.angle[ANGLE_Y], [0.0, 0.0, 10000.0]);
                    self.target = out + actor.pos;
                    self.timers[0] = REG8_S5 + 0x1E;
                    arm = 2;
                }
                2 => {
                    actor.speed_f_walk = REG8_F10 + 30.0;
                    self.speed_f = 30.0;
                    self.speed_step = 30.0;
                    if self.speed.y < 0.0 {
                        self.speed.y = 0.0;
                    }
                    if self.timers[0] == 0 {
                        self.anm_init(BCK_TYAKUTI1, 15.0, EMODE_NONE, 0.001, 0x17, 0);
                        self.action = Action::Landing;
                        self.state = 3;
                    }
                    break;
                }
                _ => break,
            }
        }
        self.ground_move(actor);
    }

    fn jida_attack(&mut self, actor: &mut Actor, env: &Env<'_>) {
        let dist = env.player_distance(actor.pos);
        self.track_player = true;
        let mut arm = self.state;
        loop {
            match arm {
                0 => {
                    self.anm_init(BCK_JIDANDA1, 10.0, EMODE_LOOP, 1.0, 0xE, 0);
                    self.state = 1;
                    self.timers[0] = (self.rnd_f(5.0) as i32 * 0x18) as i16;
                    arm = 1;
                }
                1 => {
                    if self.frame_count & 7 == 0 {
                        self.eff_hane_set(self.body, 1, 0);
                    }
                    self.at_mode = 2;
                    let frame = self.frame_i();
                    if (frame >= 4 && frame <= 0xB) || (frame >= 0x10 && frame <= 0x17) {
                        add_calc_angle_s2(
                            &mut actor.angle[ANGLE_Y],
                            env.player_bearing(actor.pos),
                            TURN_DIV_AIM,
                            0x200,
                        );
                    }
                    let push = turned(
                        env.player_bearing(actor.pos),
                        [
                            0.0,
                            -10.0,
                            if (frame >= 4 && frame <= 0xB) || (frame >= 0x10 && frame <= 0x17) {
                                REG0_F1 + 10.0
                            } else {
                                0.0
                            },
                        ],
                    );
                    actor.pos += push;
                    if self.timers[0] == 0 || dist > 600.0 {
                        self.action = Action::Wait;
                        self.state = 0;
                    }
                    break;
                }
                _ => break,
            }
        }
    }

    fn kuti_attack(&mut self, actor: &mut Actor, out: &mut Sink<'_>) {
        let mut rock = false;
        let mut grab = false;
        self.beak_amp = 2500;
        let mut arm = self.state;
        loop {
            match arm {
                0 => {
                    self.anm_init(BCK_ATTACK1, 5.0, EMODE_NONE, 1.0, 0x7, 0);
                    self.state = 2;
                    out.se(SE_CV_DK_ATTACK);
                    break;
                }
                2 => {
                    if self.morph.frame < 36.0 {
                        self.at_mode = 1;
                    }
                    if i32::from(REG0_S5 + 0x1E) == self.frame_i() {
                        self.beak_timer = REG0_S6 + 0x1E;
                        self.beak_aim = 0;
                        out.se(SE_CM_DK_ATTACK);
                        let eff = &mut self.effects[EFF_ROCK];
                        eff.kind = 2;
                        eff.count = 0;
                        let rock_pos = turned(actor.shape_angle[ANGLE_Y], [0.0, 0.0, 500.0]);
                        eff.pos = rock_pos + actor.pos;
                        self.eff_hane_set(self.body, 5, 0);
                        rock = true;
                    }
                    if self.morph.is_stop() {
                        self.anm_init(BCK_NUKENAI1, 2.0, EMODE_NONE, 1.0, BAS_NONE, 0);
                        self.state = 5;
                        self.timers[0] = self.rnd_f(60.0) as i16 + 90;
                        out.se(SE_CM_DK_JITABATA);
                        out.se(SE_CV_DK_NUKEZU);
                    }
                    break;
                }
                3 => {
                    if self.morph.is_stop() {
                        self.action = Action::Wait;
                        self.state = 0;
                    }
                    break;
                }
                5 => {
                    if self.morph.is_stop() {
                        if self.timers[0] == 0 {
                            grab = true;
                        } else {
                            self.morph.rate = -1.0;
                            self.state = 6;
                        }
                    }
                    break;
                }
                6 => {
                    if self.morph.is_stop() {
                        self.morph.rate = 1.0;
                        self.state = 5;
                        out.se(SE_CM_DK_JITABATA);
                        out.se(SE_CV_DK_NUKEZU);
                        self.eff_hane_set(self.body, 2, 0);
                    }
                    break;
                }
                10 => {
                    if self.morph.is_stop() {
                        if self.timers[0] == 0 {
                            grab = true;
                        } else {
                            self.anm_init(BCK_NUKENAI1, 2.0, EMODE_NONE, 1.0, BAS_NONE, 0);
                            self.state = 5;
                            out.se(SE_CM_DK_JITABATA);
                            out.se(SE_CV_DK_NUKEZU);
                        }
                        self.eff_hane_set(self.body, 2, 0);
                    }
                    break;
                }
                _ => break,
            }
        }
        if self.timers[2] == 1 || grab {
            self.anm_init(BCK_NUKU1, 1.0, EMODE_NONE, 1.0, BAS_NONE, 0);
            self.state = 3;
            out.se(SE_CM_DK_PULL_UP_HEAD);
            out.se(SE_CV_DK_NUKERU);
            self.effects[EFF_ROCK].kind = 0;
            rock = true;
        }
        if rock {
            if self.smoke_timer[2] == 0 {
                self.smoke_timer[2] = 0x3C;
            }
        }
        actor.pos.y += self.speed.y;
    }

    fn wind_set(&mut self, actor: &Actor, env: &Env<'_>, out: &mut Sink<'_>, at: Vec3) {
        if self.frame_count & 1 == 0 {
            for index in 0..WIND_COUNT {
                if self.wind_active[index] == 0 {
                    self.wind_active[index] = 1;
                    let yaw = actor.angle[ANGLE_Y].wrapping_add(self.rnd_fx(6000.0) as i16);
                    self.wind_move[index] = turned(yaw, [0.0, 0.0, REG0_F4 + 100.0]);
                    self.wind_pos[index] = at;
                    self.wind_pos[index].y = env.player_pos.y;
                    break;
                }
            }
        }
        out.se(SE_CM_DK_WIND);
    }

    fn fly_attack(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        let mut glide = false;
        self.track_player = true;
        let mut arm = self.state;
        loop {
            match arm {
                0 => {
                    self.state = 1;
                    self.timers[0] = 0x1E;
                    self.turn_step = 0.0;
                    self.timers[1] = 200;
                    self.swoop_offset = self.rnd_fx(100.0);
                    arm = 1;
                }
                1 => {
                    glide = true;
                    self.target = env.player_pos;
                    self.target.x += self.swoop_offset;
                    self.target.z += self.swoop_offset;
                    if self.timers[0] == 0 {
                        self.turn_scale = 400.0;
                    } else {
                        self.target.x = 3273.0;
                        self.target.z = -3108.0;
                        self.turn_scale = 1000.0;
                    }
                    self.speed_f = HIO_M024;
                    self.speed_step = 1.0;
                    let offset = self.target - actor.pos;
                    if offset.length() < REG8_F11 + 1000.0 || self.timers[1] == 0 {
                        self.state = 2;
                        let dest = turned(actor.angle[ANGLE_Y], [0.0, 500.0, 4000.0]);
                        self.target = actor.pos + dest;
                        self.turn_step = 0.0;
                        self.timers[0] = 0x28;
                        self.eff_hane_set(self.body, 10, 0);
                    } else if self.timers[0] == 1 {
                        self.anm_init(BCK_KAKKU1, 30.0, EMODE_LOOP, 1.0, BAS_NONE, 0);
                    }
                    break;
                }
                2 => {
                    glide = true;
                    let dest = turned(actor.angle[ANGLE_Y], [0.0, 0.0, REG0_F11 - 1000.0]);
                    self.wind_set(actor, env, out, dest + actor.pos);
                    if self.timers[0] == 0 {
                        self.action = Action::Fly;
                        self.fly_ca = -1;
                        self.state = 0;
                    }
                    break;
                }
                10 => {
                    self.anm_init(BCK_DAMAGE4, 2.0, EMODE_NONE, 1.0, BAS_NONE, 1);
                    self.state = 0xB;
                    self.speed.y = 0.0;
                    self.timers[0] = REG0_S5 + 0x28;
                    self.timers[1] = REG0_S6 + 0x50;
                    out.se(SE_CM_DK_CRASH_WALL);
                    out.se(SE_CV_DK_KABE_DAMAGE);
                    arm = 0xB;
                }
                0xB => {
                    actor.pos.y += self.speed.y;
                    if self.morph.is_stop() {
                        self.anm_init(BCK_PIKUPIKU1, 5.0, EMODE_LOOP, 1.0, BAS_NONE, 0);
                    }
                    if self.timers[0] == 0 {
                        self.speed.y -= 0.5 + REG0_F7;
                        actor.angle[ANGLE_Z] =
                            actor.angle[ANGLE_Z].wrapping_add(REG0_S7 + 0x50);
                        if self.timers[1] == 0 {
                            self.state = 0xC;
                            self.target_yaw = actor.angle[ANGLE_Y].wrapping_add(0x8000);
                            self.timers[0] = REG0_S8 + 0x28;
                            self.anm_init(BCK_FLY2, 5.0, EMODE_LOOP, 2.0, 0xC, 0);
                        }
                    }
                    break;
                }
                0xC => {
                    self.speed.y += REG0_F8 + 2.0;
                    actor.pos.y += self.speed.y;
                    add_calc_angle_s2(
                        &mut actor.angle[ANGLE_Y],
                        self.target_yaw,
                        TURN_DIV_AIM,
                        0x400,
                    );
                    if self.timers[0] == 0 {
                        self.action = Action::Fly;
                        self.fly_ca = -1;
                        self.state = 0;
                        actor.speed_f_walk = 0.0;
                    }
                    break;
                }
                _ => break,
            }
        }
        if self.action == Action::FlyAttack && self.state < 10 {
            self.at_mode = 2;
            let from = actor.pos;
            self.pos_move(actor);
            let (next, wall) = env.corrected(from, actor.pos);
            actor.pos = next;
            if env.landed(actor.pos) && self.nail_live == 0 {
                self.nail_live += 1;
            }
            if wall {
                self.state = 10;
            }
        }
        if glide {
            out.se(SE_CM_DK_GLIDING);
        }
    }

    fn wind_attack(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        let mut arm = self.state;
        let mut done = false;
        loop {
            match arm {
                0 => {
                    self.anm_init(BCK_TOBITATU1, 5.0, EMODE_NONE, 1.0, 0x16, 0);
                    self.rise_smoke = 0x19;
                    self.state += 1;
                    self.target = actor.pos;
                    self.turn_step = 0.0;
                    arm = 1;
                }
                1 => {
                    if self.morph.frame == f32::from(23) {
                        self.state = 2;
                        self.target.y += 500.0;
                    }
                    break;
                }
                2 => {
                    if self.morph.is_stop() {
                        self.anm_init(BCK_FLY2, 5.0, EMODE_LOOP, 2.0, 0xC, 0);
                        self.state = 3;
                        self.timers[0] = 200;
                    }
                    break;
                }
                3 => {
                    self.wind_set(actor, env, out, actor.pos);
                    if self.frame_count & 1 == 0 {
                        let mut offset = self.body;
                        offset.x += self.rnd_fx(300.0);
                        offset.y -= self.rnd_f(200.0);
                        offset.z += self.rnd_fx(300.0);
                        self.eff_hane_set(offset, 1, 1);
                    }
                    add_calc2(&mut self.wind_power, 1.0, 1.0, 0.05);
                    add_calc2(&mut self.turn_step, 1.0, 1.0, 0.05);
                    if self.timers[0] == 0 || self.hit_stop != 0 {
                        self.action = Action::Landing;
                        self.state = 3;
                        self.target = env.player_pos;
                        self.speed.y = -20.0;
                        done = true;
                        break;
                    }
                    break;
                }
                _ => break,
            }
        }
        if done {
            return;
        }
        add_calc_angle_s2(
            &mut actor.angle[ANGLE_Y],
            env.player_bearing(actor.pos),
            TURN_DIV_FACE,
            TURN_CAP as i16,
        );
        let sway_y = sin_s(self.frame_count.wrapping_mul(1500) as i16);
        let sway_x = sin_s(self.frame_count.wrapping_mul(700) as i16);
        let sway_z = sin_s(self.frame_count.wrapping_mul(500) as i16);
        add_calc2(
            &mut actor.pos.y,
            self.target.y + sway_y * self.turn_step * 100.0,
            0.1,
            60.0,
        );
        add_calc2(
            &mut actor.pos.x,
            self.target.x + sway_x * self.turn_step * 200.0,
            0.1,
            40.0,
        );
        add_calc2(
            &mut actor.pos.z,
            self.target.z + sway_z * self.turn_step * 200.0,
            0.1,
            40.0,
        );
    }

    fn end_set(&mut self, actor: &Actor, out: &mut Sink<'_>) {
        self.action = Action::End;
        self.effects[EFF_ROCK].kind = 0;
        if actor.pos.y < 9810.0 {
            if self.body.y < 10000.0 {
                self.state = 0;
            } else {
                self.state = 1;
            }
        } else {
            self.state = 2;
            self.flown_high = 1;
        }
        self.demo_count = 100;
        out.stop_bgm();
    }

    fn damage_check(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        if self.hit_stop != 0 || !self.at_pending(env) {
            return;
        }
        let reach = env.blow.at.reach();
        let mut hit: u8 = HIT_NONE;
        if self.mask_damage < 4 {
            if touches(self.head.pos, self.head_radius, env.player_pos, reach) {
                hit = HIT_HEAD;
            }
        } else if touches(self.tosaka, self.tosaka_radius, env.player_pos, reach) {
            hit = HIT_TOSAKA;
        }
        if hit == HIT_NONE {
            return;
        }
        let result = env.blow.at.result();
        if hit == HIT_HEAD {
            self.hit_stop = 0xC;
            if result == AT_HAMMER && self.head.pos.y < 10000.0 {
                self.health = HAMMER_HEALTH;
                out.se(SE_LK_HAMMER_HIT);
                if self.battle_mode == 1 {
                    self.action = Action::TDown;
                    self.state = 0;
                    self.mask_timer = REG6_S3 + 10;
                    self.hit_stop = 0x1E;
                    out.se(SE_CV_DK_LAST_DAMAGE);
                    out.stop_bgm();
                } else {
                    out.stories.set(EVENT_BIT_LAST_DAMAGE);
                    self.mask_damage = self.mask_damage.wrapping_add(1);
                    if self.mask_damage == 4 {
                        self.mask_damage = 3;
                        self.action = Action::KamenDemo;
                        self.state = 0;
                        out.se(SE_CM_DK_BREAK_MASK);
                    } else {
                        self.mask_timer = REG6_S3 + 10;
                        self.timers[2] = 0xF;
                        self.hit_stop = 0x1E;
                    }
                }
                self.beak_timer = 0;
            } else {
                out.se(env.blow.at.at_se());
            }
        } else {
            self.hit_stop = 6;
            out.se(SE_CV_DK_DAMAGE);
            self.beak_timer = REG0_S5 + 0x1E;
            self.beak_aim = bearing(env.player_pos, actor.eye);
            let mut flinched = false;
            if self.action == Action::KutiAttack && self.body.y < 10000.0 {
                self.anm_init(BCK_DAMAGE1, 2.0, EMODE_NONE, 1.0, BAS_NONE, 0);
                self.state = 10;
                self.beat_count = self.beat_count.wrapping_add(1);
                if i32::from(self.beat_count) >= 6 {
                    self.timers[2] = 10;
                    self.hit_stop = 0x14;
                }
                flinched = true;
            } else if self.ground_hit != 0 {
                if self.action == Action::KutiAttack {
                    self.action = Action::Wait;
                    self.anm_init(BCK_WAIT1, 10.0, EMODE_LOOP, 1.0, BAS_NONE, 0);
                    self.state = 1;
                    self.timers[0] = self.rnd_f(30.0) as i16 + 30;
                    self.timers[1] = self.timers[0];
                    self.fly_ca = 0;
                }
                self.anm_init(BCK_DAMAGE3, 2.0, EMODE_NONE, 1.0, BAS_NONE, 1);
                flinched = true;
            } else {
                self.anm_init(BCK_DAMAGE2, 2.0, EMODE_NONE, 1.0, 0x9, 1);
                flinched = true;
            }
            let power = env.blow.at.power(env.blow.blade, env.blow.soup);
            self.health -= power;
            if self.health <= 0 {
                out.se(SE_LK_LAST_HIT);
                out.se(SE_CV_DK_LAST_DAMAGE);
                self.end_set(actor, out);
            } else {
                out.se(env.blow.at.at_se());
                let _ = flinched;
            }
        }
    }

    fn at_pending(&self, env: &Env<'_>) -> bool {
        !matches!(env.blow.at, At::Idle)
    }

    fn kamen_demo(&mut self, actor: &Actor, out: &mut Sink<'_>) {
        self.beak_amp = 0;
        match self.state {
            0 => {
                self.anm_init(BCK_NUKENAI1, 0.0, EMODE_NONE, 0.0, BAS_NONE, 0);
                self.state = 1;
                self.demo_count = 1;
            }
            1 => {
                self.beak_sway = 0;
                self.beak_aim = 0;
                self.fov = 45.0;
                for index in 0..PIECE_COUNT {
                    self.piece_live[index] = 1;
                    self.piece_pos[index] =
                        turned(actor.shape_angle[ANGLE_Y], JOINT_ATAMA1);
                    self.piece_rot[index][1] = actor.shape_angle[ANGLE_Y];
                    self.piece_rot[index][0] = REG0_S6 + 0x3625;
                    self.piece_rot[index][2] = 0;
                    let scale = 1.0 + REG0_F7;
                    let away = turned(
                        self.piece_rot[index][1],
                        [
                            -KAMEN_BREAK_OFF_X[index],
                            0.01 * REG0_F15 + (55.0 - KAMEN_BREAK_OFF_Y[index]),
                            0.01 * REG0_F16 + (110.0 - KAMEN_BREAK_OFF_Z[index]),
                        ],
                    );
                    self.piece_pos[index] += away;
                    self.piece_prev[index] = self.piece_pos[index];
                    self.piece_move[index] = turned(
                        self.piece_rot[index][1],
                        [
                            scale * KAMEN_BREAK_SD_X[index],
                            0.0,
                            scale * KAMEN_BREAK_SD_Z[index],
                        ],
                    );
                    self.piece_hold[index] = KAMEN_BREAK_TIME[index];
                }
                self.state = 2;
            }
            2 => {
                if self.demo_sub > REG0_S8 + 0x2D {
                    self.beak_amp = 500;
                    self.beak_free = 1;
                }
                if self.demo_sub >= 0x6E && REG8_S8 == 0 {
                    self.demo_count = 150;
                    self.action = Action::KutiAttack;
                    self.anm_init(BCK_NUKENAI1, 2.0, EMODE_NONE, 1.0, BAS_NONE, 0);
                    self.state = 5;
                    self.timers[0] = self.rnd_f(120.0) as i16 + 30;
                    out.bgm(BGM_UNK_110);
                }
            }
            _ => {}
        }
    }

    fn start(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        match self.state {
            0 => {
                if env.player_pos.y >= 9800.0 && !env.player_hover {
                    if self.timers[0] == 0 {
                        self.state = 1;
                        self.demo_state = 10;
                        self.demo_count = 0;
                        self.demo_sub = 0;
                        out.clear_mobs();
                    }
                } else {
                    self.timers[0] = 0x14;
                }
                self.timers[0] = 0x1E;
            }
            1 => {
                if self.demo_flag != 0 {
                    self.demo_flag = 0;
                    self.state = 2;
                    self.tail_hold = REG0_S6 + 0x19;
                    actor.pos.y = REG0_F10 + 8500.0;
                    self.speed.y = REG0_F12 + 80.0;
                    self.anm_init(BCK_S_DEMO1, 5.0, EMODE_LOOP, 1.0, BAS_NONE, 0);
                    out.se(SE_CM_DK_ROUND_UP);
                }
            }
            2 => {
                actor.pos.y += self.speed.y;
                self.speed.y -= REG0_F13 + 1.0;
                actor.angle[ANGLE_Y] = actor.angle[ANGLE_Y].wrapping_add(0x1300);
                if self.demo_flag != 0 {
                    self.demo_flag = 0;
                    self.state = 3;
                    self.anm_init(BCK_S_DEMO2, 5.0, EMODE_NONE, 1.0, 0x15, 0);
                    self.timers[0] = 0x2D;
                }
            }
            3 => {
                if self.timers[0] != 0 {
                    actor.pos.y += self.speed.y;
                    self.speed.y -= REG0_F13 + 1.0;
                }
                if self.timers[0] == REG8_S8 + 5 {
                    out.se(SE_CV_DK_ENTER);
                }
                add_calc_angle_s2(
                    &mut actor.angle[ANGLE_Y],
                    env.player_bearing(actor.pos),
                    10,
                    0x1300,
                );
            }
            _ => {}
        }
    }

    fn end(&mut self, actor: &mut Actor, out: &mut Sink<'_>) {
        self.beak_amp = 4000;
        self.hit_stop = 10;
        let mut groan = false;
        let mut arm = self.state;
        loop {
            match arm {
                0 => {
                    self.anm_init(BCK_LAST_DAMAGE1, 2.0, EMODE_NONE, 1.0, 0x10, 0);
                    self.state = 5;
                    self.timers[0] = 0xB3;
                    break;
                }
                1 => {
                    self.anm_init(BCK_LAST_DAMAGE3, 2.0, EMODE_NONE, 1.0, 0x12, 0);
                    self.state = 5;
                    self.timers[0] = 0xB3;
                    break;
                }
                2 => {
                    self.anm_init(BCK_LAST_DAMAGE2, 2.0, EMODE_NONE, 1.0, 0x11, 0);
                    self.state = 5;
                    self.timers[0] = 0;
                    break;
                }
                5 => {
                    if self.morph.is_stop() {
                        self.state = 6;
                        self.anm_init(BCK_DEATH1, 2.0, EMODE_LOOP, 1.0, 0xA, 0);
                        self.timers[0] = REG6_S3 + 300;
                        groan = true;
                        arm = 6;
                    } else {
                        if self.timers[0] == 0 {
                            add_calc2(&mut actor.pos.y, 10800.0, 0.1, 50.0);
                            groan = true;
                        }
                        break;
                    }
                }
                6 => {
                    groan = true;
                    if self.timers[0] == 0xAA {
                        self.demo_count = 0x6E;
                    }
                    add_calc2(&mut actor.pos.x, AREA_X, 0.05, 50.0);
                    add_calc2(&mut actor.pos.y, REG8_F18 + 15800.0, 0.1, REG8_F19 + 20.0);
                    add_calc2(&mut actor.pos.z, AREA_Z, 0.05, 50.0);
                    add_calc_angle_s2(&mut actor.angle[ANGLE_Y], -8000, TURN_DIV_CAM, 0x80);
                    if self.timers[0] == 0x3C {
                        out.se(SE_CV_DK_DIE);
                        self.anm_init(BCK_DEATH2, 5.0, EMODE_NONE, 1.0, BAS_NONE, 0);
                    }
                    if self.timers[0] == 0 && REG6_S4 == 0 {
                        self.state = 7;
                        for index in 0..PIECE_COUNT {
                            self.piece_live[index] = 0;
                        }
                        out.purge_shards();
                        self.target = actor.pos;
                        self.target.y += 200.0;
                        actor.angle[ANGLE_Y] = 0x4000;
                        out.se(SE_CM_DK_DIE_EXPLODE);
                        self.timers[0] = 0x1E;
                    }
                    break;
                }
                7 => {
                    actor.pos.x = 100000.0;
                    actor.pos.y = 100000.0;
                    if self.timers[0] == 0 {
                        self.demo_count += 1;
                        self.state = 8;
                    }
                    break;
                }
                8 => break,
                _ => break,
            }
        }
        if groan && self.timers[4] == 0 {
            self.timers[4] = self.rnd_f(30.0) as i16 + 25;
            out.se(SE_CV_DK_MODAE);
        }
        add_calc_angle_s2(&mut actor.angle[ANGLE_Z], 0, TURN_DIV_WALK, TURN_CAP as i16);
        add_calc_angle_s2(&mut actor.angle[ANGLE_X], 0, TURN_DIV_WALK, TURN_CAP as i16);
    }

    fn t_fly(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        self.track_player = true;
        let mut dy = 0.0;
        let mut dz = -500.0;
        let mut step = 20.0;
        if self.frame_count == 100 {
            out.switches.set(SWITCH_TOGE);
        }
        match self.state {
            -10 => {
                self.anm_init(BCK_TOBITATU1, 5.0, EMODE_NONE, 1.0, 0x16, 0);
                self.state += 1;
            }
            -9 => {
                if self.morph.is_stop() {
                    self.state = 0;
                }
            }
            0 => {
                self.anm_init(BCK_FLY2, 5.0, EMODE_LOOP, 1.0, 0xC, 0);
                self.state += 1;
            }
            1 => {
                if let Some((entity, peck)) = env.peck {
                    self.target = peck;
                    self.anm_init(BCK_ATTACK_T1, 5.0, EMODE_NONE, 1.0, BAS_NONE, 0);
                    self.state += 1;
                    out.se(SE_CV_DK_ATTACK);
                    self.peck_obj = Some(entity);
                } else {
                    self.target = env.player_pos;
                    if self.timers[0] == 0 {
                        self.anm_init(BCK_ATTACK_T2, 5.0, EMODE_NONE, 1.0, BAS_NONE, 0);
                        self.state = 10;
                        out.se(SE_CV_DK_ATTACK);
                        out.se(SE_CM_DK_FOOT_ATTACK);
                        self.timers[0] = HIO_M028;
                    }
                }
            }
            2 => {
                let frame = self.frame_i();
                if i32::from(REG0_S3 + 0x2B) <= frame && frame <= i32::from(REG0_S4 + 0x32) {
                    self.wind_set(actor, env, out, actor.pos);
                    self.at_mode = 1;
                    dz = REG8_F8 + 150.0;
                    step = 300.0;
                    if self.peck_obj.is_some() && self.frame_i() == 0x30 {
                        self.peck_obj = None;
                        out.break_peck();
                        out.se(SE_CM_DK_PECK);
                    }
                }
                if self.morph.is_stop() {
                    self.state = 0;
                    self.turn_step = 0.0;
                }
            }
            10 => {
                let frame = self.frame_i();
                if i32::from(REG0_S3 + 0x17) <= frame && frame <= i32::from(REG0_S4 + 0x32) {
                    self.wind_set(actor, env, out, actor.pos);
                    dy = REG8_F9 - 50.0;
                    dz = REG8_F10 + 250.0;
                    step = 300.0;
                    if frame <= i32::from(REG0_S5 + 0x23) {
                        self.at_mode = 2;
                    }
                }
                if touches(self.foot[0], self.foot_radius[0], env.player_pos, PLAYER_BODY_RADIUS)
                    || touches(self.foot[1], self.foot_radius[1], env.player_pos, PLAYER_BODY_RADIUS)
                {
                    out.se(SE_CM_DK_FOOT_ATK_COL);
                }
                if self.morph.is_stop() {
                    self.state = 0;
                    self.turn_step = 0.0;
                }
            }
            _ => {}
        }
        let pos = self.target - Vec3::new(AREA_X, AREA_Y, AREA_Z);
        let angle = bearing(Vec3::ZERO, pos);
        add_calc_angle_s2(&mut actor.angle[ANGLE_Y], angle, 10, 0x800);
        let dist = turned(angle, [0.0, dy, dz - 450.0]);
        add_calc2(
            &mut actor.pos.x,
            self.target.x + dist.x,
            0.2,
            step * self.turn_step,
        );
        if self.speed.y > 0.0 {
            actor.pos.y += self.speed.y;
            self.speed.y -= 5.0;
        } else {
            add_calc2(
                &mut actor.pos.y,
                self.target.y + dist.y,
                0.2,
                step * self.turn_step,
            );
        }
        add_calc2(
            &mut actor.pos.z,
            self.target.z + dist.z,
            0.2,
            step * self.turn_step,
        );
        add_calc2(&mut self.turn_step, 1.0, 1.0, 0.02);
        if env.player_pos.y > REG0_F3 + 9300.0 - 50.0 {
            self.action = Action::TLastAttack;
            self.state = 0;
        }
    }

    fn t_landing(&mut self, actor: &mut Actor) {
        self.track_player = true;
        if self.state == 0 {
            self.anm_init(BCK_TOBITATU_T1, 5.0, EMODE_LOOP, 1.0, BAS_NONE, 0);
            self.state += 1;
        }
        self.pos_move(actor);
    }

    fn t_lastattack(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        let goal_z = REG0_F9 * 0.01 + (-3872.0);
        let goal_y = REG0_F8 * 0.01 + 9579.0;
        self.target.x = REG0_F7 * 0.01 + 2825.0 - 40.0;
        self.target.y = goal_y;
        self.target.z = goal_z;
        add_calc2(&mut actor.pos.x, self.target.x, 0.5, 30.0);
        add_calc2(&mut actor.pos.z, self.target.z, 0.5, 30.0);
        actor.pos.y += self.speed.y;
        self.speed.y -= 5.0;
        let mut falling = false;
        if actor.pos.y <= self.target.y {
            actor.pos.y = self.target.y;
            self.speed.y = -1.0;
            falling = true;
        }
        let mut angle = env.player_bearing(actor.pos).wrapping_sub(0x8000);
        if angle < -0xEB8 {
            angle = -0xEB8;
        } else if 0xE8 < angle {
            angle = 0xE8;
        }
        add_calc_angle_s2(
            &mut actor.angle[ANGLE_Y],
            angle.wrapping_add(0x8000),
            TURN_DIV_WALK,
            TURN_CAP as i16,
        );
        match self.state {
            0 => {
                self.speed.y = REG0_F6 + 80.0;
                self.state += 1;
                self.anm_init(BCK_TYAKUTI_T1, 10.0, EMODE_NONE, 0.001, 0x18, 0);
            }
            1 => {
                if falling {
                    self.anm_init(BCK_TYAKUTI_T1, 1.0, EMODE_NONE, 1.0, 0x18, 0);
                    self.state += 1;
                    out.se(SE_CM_DK_PECK);
                    self.eff_hane_set(self.body, 5, 0);
                }
            }
            2 => {
                if self.morph.is_stop() {
                    self.anm_init(BCK_WAIT_T1, 5.0, EMODE_LOOP, 1.0, BAS_NONE, 0);
                    self.state += 1;
                }
            }
            3 => {
                if self.timers[0] == 0
                    && env.player_distance(actor.pos) < REG0_F13 + 700.0
                {
                    self.anm_init(BCK_ATTACK_T3, 5.0, EMODE_NONE, 1.0, BAS_NONE, 0);
                    out.se(SE_CV_DK_ATTACK);
                    self.state = 4;
                }
            }
            4 => {
                let frame = self.frame();
                if frame > 10.0 && frame < 15.0 {
                    self.at_mode = 1;
                }
                if i32::from(REG0_S5 + 0xD) == self.frame_i() {
                    out.se(SE_CM_DK_PECK);
                    self.eff_hane_set(self.body, 5, 0);
                }
                if self.morph.is_stop() {
                    self.anm_init(BCK_WAIT_T1, 5.0, EMODE_LOOP, 1.0, BAS_NONE, 0);
                    self.state = 3;
                    self.timers[0] = self.rnd_f(100.0) as i16 + 100;
                }
            }
            _ => {}
        }
        if env.player_pos.y < REG0_F3 + 9290.0 - 50.0 {
            self.action = Action::TFly;
            self.state = -10;
            self.speed.y = REG0_F7 + 70.0;
        }
    }

    fn t_down(&mut self, actor: &mut Actor, out: &mut Sink<'_>) {
        let mut arm = self.state;
        loop {
            match arm {
                0 => {
                    self.anm_init(BCK_TO_DAMAGE_T1, 2.0, EMODE_NONE, 1.0, BAS_NONE, 0);
                    self.state += 1;
                    self.timers[0] = REG0_S6 + 0x50;
                    self.timers[1] = REG0_S7 + 0x50;
                    self.demo_count = 0x32;
                    self.blure_rate = 0xB4;
                    self.eff_hane_set(self.body, 5, 0);
                    out.switches.clear(SWITCH_TOGE);
                    arm = 1;
                }
                1 | 2 => {
                    if self.morph.is_stop() {
                        self.anm_init(BCK_DAMAGE_T1, 2.0, EMODE_LOOP, 1.0, BAS_NONE, 0);
                    }
                    add_calc2(&mut actor.pos.x, AREA_X, 0.2, REG0_F4 + 10.0);
                    add_calc2(&mut actor.pos.z, AREA_Z, 0.2, REG0_F4 + 10.0);
                    add_calc_angle_s2(&mut self.blure_rate, 1, 1, 2);
                    if self.timers[1] == 1 {
                        self.anm_init(BCK_RAKKA_T1, REG0_F6 + 20.0, EMODE_LOOP, 1.0, BAS_NONE, 0);
                        self.blure_rate = 1;
                        out.se(SE_CM_DK_FALL_WATER);
                    }
                    if self.timers[0] != 0 {
                        break;
                    }
                    actor.pos.y += self.speed.y;
                    self.speed.y -= GRAVITY_FALL;
                    if actor.pos.y < 6900.0 && self.state == 1 {
                        self.state = 2;
                        out.se(SE_CM_DK_WATER_COLUMN);
                    }
                    if actor.pos.y < 5500.0 {
                        self.action = Action::Start;
                        self.state = 0;
                        actor.pos.y = -25000.0;
                        self.reset_health();
                    }
                    break;
                }
                _ => break,
            }
        }
    }

    fn after_fight(&mut self) {
        self.hit_stop = 10;
    }

    fn move_actor(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        match self.action {
            Action::Fly => self.fly(actor),
            Action::UpFly => self.up_fly(actor),
            Action::Landing => self.landing(actor, env),
            Action::Wait => self.wait(actor, env),
            Action::Jump => self.jump(actor, env),
            Action::KutiAttack => self.kuti_attack(actor, out),
            Action::JidaAttack => self.jida_attack(actor, env),
            Action::FlyAttack => self.fly_attack(actor, env, out),
            Action::WindAttack => self.wind_attack(actor, env, out),
            Action::KamenDemo => self.kamen_demo(actor, out),
            Action::End => self.end(actor, out),
            Action::Start => self.start(actor, env, out),
            Action::TFly => self.t_fly(actor, env, out),
            Action::TLanding => self.t_landing(actor),
            Action::TLastAttack => self.t_lastattack(actor, env, out),
            Action::TDown => self.t_down(actor, out),
            Action::AfterFight => self.after_fight(),
        }
        if self.resume_pending != 0 && self.morph.is_stop() {
            self.anm_init(self.resume_bck, 20.0, self.resume_loop, 1.0, self.resume_bas, 0);
            self.resume_pending = 0;
        }
    }

    fn at_pos(yaw: i16, joint: [f32; 3]) -> Vec3 {
        turned(yaw, joint)
    }

    fn col_set(&mut self, actor: &Actor) {
        let yaw = actor.shape_angle[ANGLE_Y];
        self.body = actor.pos;
        self.body.y += BODY_LIFT;
        self.head_prev = self.head.pos;
        let mut offset = HEAD_OFFSET;
        if self.mask_damage >= 4 {
            offset[2] = 0.0;
            self.head_radius = HEAD_RADIUS_BARE;
        } else {
            offset[2] += REG0_F2;
            self.head_radius = HEAD_RADIUS_MASKED;
        }
        self.head.pos = Self::at_pos(yaw, JOINT_ATAMA1) + turned(yaw, offset);
        if self.at_mode == 1 {
            self.beak = Self::at_pos(yaw, JOINT_AGO) + turned(yaw, BEAK_OFFSET);
        } else {
            self.beak.y = HEAD_OFF_Y;
        }
        self.beak_radius = REG8_F6 + BEAK_RADIUS;
        self.body_radius = REG8_F8 + BODY_RADIUS;
        if self.mask_damage >= 4 {
            self.tosaka = Self::at_pos(yaw, JOINT_TOSAKA_A3) + turned(yaw, BEAK_TIP_OFFSET);
            self.tosaka_radius = REG8_F11 + TOSAKA_RADIUS;
        }
        for index in 0..FOOT_COUNT {
            let base = if index == 0 {
                JOINT_ASHI_L3
            } else {
                JOINT_ASHI_R3
            };
            let step = if self.action == Action::TLastAttack {
                self.foot_radius[index] = REG0_F8 + FOOT_RADIUS_T_FLY;
                self.foot_tg[index] = false;
                [REG0_F3 + FOOT_TALL_X, REG0_F4 + FOOT_TALL_Y, REG0_F5]
            } else {
                self.foot_radius[index] = REG0_F9 + FOOT_RADIUS;
                self.foot_tg[index] = true;
                [REG0_F3 + FOOT_X, REG0_F4, REG0_F5]
            };
            self.foot[index] = Self::at_pos(yaw, base) + turned(yaw, step);
            if self.at_mode == 2 {
                self.foot_at[index] = true;
                self.foot_split[index] = matches!(
                    self.action,
                    Action::FlyAttack | Action::TFly | Action::JidaAttack
                );
                if self.foot_at_live == 0 {
                    self.foot_at_live = 1;
                }
            } else {
                self.foot_at[index] = false;
                self.foot_at_live = 0;
            }
        }
    }

    fn kankyo_cont(&mut self, actor: &Actor) {
        self.wind_anchor = actor.pos;
        self.wind_dir = turned(actor.angle[ANGLE_Y], [0.0, 0.0, 1.0]);
        self.wind_scale.x = self.wind_power * 2000.0;
        self.wind_scale.z = 0.0;
        add_calc0(&mut self.wind_power, 1.0, 0.02);
        self.wind_scale.y = if self.wind_power > 1.0 { 1.0 } else { 0.0 };
        for index in 0..WIND_COUNT {
            if self.wind_active[index] != 0 {
                self.wind_pos[index] += self.wind_move[index];
                self.wind_radius[index] = if self.battle_mode == 1 {
                    REG8_F7 + WIND_RADIUS_BATTLE
                } else {
                    REG8_F6 + WIND_RADIUS
                };
                self.wind_active[index] += 1;
                if self.wind_active[index] > 25 {
                    self.wind_active[index] = 0;
                }
            }
        }
    }

    fn kamen_break_move(&mut self, env: &Env<'_>, out: &mut Sink<'_>) {
        for index in 0..PIECE_COUNT {
            if self.piece_live[index] == 0 {
                continue;
            }
            let mut rot_x = 0;
            let mut rot_z = 0;
            if self.piece_hold[index] == 0 {
                self.piece_prev[index] = self.piece_pos[index];
                self.piece_pos[index] += self.piece_move[index];
                self.piece_move[index].y -= GRAVITY_FALL;
                if self.piece_pos[index].y < PIECE_FLOOR {
                    self.piece_live[index] = 0;
                }
                let probe = Vec3::new(
                    self.piece_pos[index].x,
                    self.piece_pos[index].y + 200.0,
                    self.piece_pos[index].z,
                );
                let ground = env.ground_h(probe) + PIECE_GROUND_LIFT + REG0_F11;
                if self.piece_pos[index].y <= ground {
                    self.piece_pos[index].y = ground;
                    self.piece_push[index][0] = 0;
                    self.piece_push[index][1] = 0;
                    if self.piece_move[index].y < REG0_F2 + PIECE_BOUNCE_CAP {
                        self.piece_move[index].y = 4.0 + (10.0 + REG0_F3);
                        self.piece_roll[index] = REG0_F4 + PIECE_SPIN;
                        self.piece_rot[index][2] = KAMEN_BREAK_Z2[index];
                        out.se(SE_CM_MAGBALL_BOUND);
                    } else {
                        self.piece_move[index].y = 0.0;
                        let swing = self.piece_roll[index]
                            * sin_s(
                                self.frame_count
                                    .wrapping_add(PIECE_SPIN_STEP)
                                    .wrapping_add(index as i32 * PIECE_SPIN_SPREAD) as i16,
                            );
                        add_calc0(&mut self.piece_roll[index], 1.0, 200.0 + REG0_F5);
                        add_calc_angle_s2(
                            &mut self.piece_rot[index][2],
                            KAMEN_BREAK_Z1[index].wrapping_add(swing as i16),
                            1,
                            0x1000,
                        );
                    }
                    let push = turned(
                        self.piece_angle[index],
                        [0.0, 0.0, self.piece_speed[index]],
                    );
                    self.piece_move[index].x = push.x;
                    self.piece_move[index].z = push.z;
                    self.piece_rot[index][1] = self.piece_rot[index][1].wrapping_add(
                        (self.piece_sway_speed[index] * self.piece_speed[index]) as i16,
                    );
                    add_calc0(&mut self.piece_speed[index], 1.0, REG0_F5 + PIECE_PUSH_STEP);
                    add_calc_angle_s2(&mut self.piece_rot[index][0], 0x4000, 1, 0x2000);
                }
                let travel = self.piece_pos[index] - self.piece_prev[index];
                if travel.length() > 0.0 {
                    let step = turned(bearing(Vec3::ZERO, travel), [0.0, 30.0, 20.0]);
                    let from = Vec3::new(
                        self.piece_pos[index].x,
                        self.piece_pos[index].y + 30.0,
                        self.piece_pos[index].z,
                    );
                    if env.crossed(from, self.piece_pos[index] + step) {
                        self.piece_pos[index].x = self.piece_prev[index].x;
                        self.piece_pos[index].z = self.piece_prev[index].z;
                        self.piece_push[index] = [0, 0, 0];
                        self.piece_speed[index] = 0.0;
                    }
                }
            } else {
                self.piece_hold[index] -= 1;
                if self.piece_hold[index] < 0x14 {
                    rot_x = (sin_s(self.piece_hold[index].wrapping_mul(3300)) * 200.0) as i16;
                    rot_z = (cos_s(self.piece_hold[index].wrapping_mul(4800)) * 200.0) as i16;
                }
            }
            self.piece_rot[index][0] = self.piece_rot[index][0].wrapping_add(rot_x);
            self.piece_rot[index][2] = self.piece_rot[index][2].wrapping_add(rot_z);
            for axis in 0..3 {
                self.piece_sway[index][axis] =
                    self.piece_sway[index][axis].wrapping_add(self.piece_push[index][axis]);
            }
        }
    }

    fn obj_move(&mut self) {
        if self.stage_boss_enemy {
            add_calc2(&mut self.shutter_y, SHUTTER_BATTLE_Y, 1.0, 20.0);
        }
        self.shutter_ring = REG0_F10 + SHUTTER_X;
        self.shutter_shift = AREA_Y + self.shutter_y + REG8_F2;
        self.shutter_yaw = REG0_F12 - SHUTTER_Z;
    }

    fn eff_hane_move(&mut self, index: usize, env: &Env<'_>) {
        let mut accel = EFF_FALL;
        let mut rise_cap = EFF_RISE_CAP;
        let eff = &mut self.effects[index];
        if eff.chase != 0 {
            eff.chase -= 1;
        }
        let step = turned(eff.spin[1], [0.0, 0.0, eff.speed]);
        eff.prev = eff.pos;
        eff.pos += Vec3::new(step.x, eff.rise, step.z);
        let travel = eff.pos - eff.prev;
        if travel.length() > 0.0 {
            let probe = turned(bearing(Vec3::ZERO, travel), [0.0, 30.0, 20.0]);
            let from = Vec3::new(eff.pos.x, eff.pos.y + 30.0, eff.pos.z);
            if env.crossed(from, eff.pos + probe) {
                eff.pos.x = eff.prev.x;
                eff.pos.z = eff.prev.z;
                eff.speed = 0.0;
            }
        }
        let height =
            env.ground_h(Vec3::new(eff.pos.x, eff.pos.y + 200.0, eff.pos.z)) + EFF_GROUND_LIFT;
        eff.rise -= EFF_RISE_STEP + REG8_F14;
        if eff.rise < REG8_F15 + EFF_FLOOR_CLAMP {
            eff.rise = REG8_F15 + EFF_FLOOR_CLAMP;
        }
        if eff.pos.y <= height {
            eff.pos.y = height;
            add_calc0(&mut eff.speed, 1.0, 0.1);
            eff.spin[1] = eff.spin[1].wrapping_add(eff.tumble[0]);
            add_calc_angle_s2(&mut eff.tumble[0], 0, 10, 10);
            add_calc_angle_s2(&mut eff.tumble[2], 0, 10, 10);
            add_calc_angle_s2(&mut eff.spin[0], 0, 10, 200);
            add_calc_angle_s2(&mut eff.spin[2], 0, 10, 200);
        } else {
            eff.spin[0] = (eff.sway_rate * sin_s(eff.count.wrapping_mul(0x500))) as i16;
            eff.spin[2] = (eff.sway_rate * cos_s(eff.count.wrapping_mul(0x400))) as i16;
            let (up, cap) = if eff.chase != 0 {
                accel = REG8_F8 + EFF_FALL;
                rise_cap = EFF_RISE;
                (REG8_F7 + EFF_UP, EFF_CAP)
            } else {
                (EFF_UP_CALM, EFF_CAP_CALM)
            };
            add_calc2(&mut eff.sway_rate, up, 1.0, cap);
        }
        eff.tumble[1] = eff.tumble[1].wrapping_add(eff.tumble[2]);
        if eff.pos.y > REG8_F11 + EFF_CEIL {
            add_calc2(&mut eff.fall, accel, 0.1, rise_cap);
        } else {
            add_calc0(&mut eff.fall, 0.1, 1.3);
        }
        if eff.life != 0 {
            eff.life -= 1;
        }
        if eff.pos.y < EFF_FLOOR || eff.life == 0 || !land_area_check(eff.pos, EFF_AREA) {
            eff.done = 1;
        }
        if eff.done != 0 {
            add_calc0(&mut eff.scale, 1.0, 0.05);
            if eff.scale < 0.05 {
                eff.kind = 0;
            }
        }
    }

    fn eff_grock_move(&mut self, index: usize) {
        let drop = if self.effects[index].count == 1 {
            GROCK_DROP
        } else {
            0.0
        };
        self.effects[index].prev.y = drop;
    }

    fn my_effect_move(&mut self, env: &Env<'_>) {
        for index in 0..EFF_COUNT {
            if self.effects[index].kind != 0 {
                self.effects[index].count += 1;
                if self.effects[index].kind == 1 {
                    self.eff_hane_move(index, env);
                } else if self.effects[index].kind == 2 {
                    self.eff_grock_move(index);
                }
            }
        }
    }

    fn demo_gate(&mut self) -> bool {
        if !self.demo_ordered {
            self.demo_ordered = true;
            return false;
        }
        true
    }

    fn demo_camera(&mut self, actor: &Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        let mut run = true;
        let mut rumble = false;
        match self.demo_state {
            0 => {}
            1 => {
                if !self.demo_gate() {
                    run = false;
                } else {
                    self.demo_state += 1;
                    self.demo_param = 0.0;
                    self.demo_count = 0;
                    self.demo_sub = 0;
                    self.fov = 45.0;
                    self.mask_damage = 4;
                }
            }
            2 => {
                self.cam_eye =
                    turned(actor.shape_angle[ANGLE_Y], [REG0_F0 + CAM_NEAR_X - 30.0, REG0_F1 - 30.0, REG0_F2 + 50.0])
                        + self.head.pos;
                self.cam_look =
                    turned(actor.shape_angle[ANGLE_Y], [REG0_F5 + 400.0, REG0_F6 - 100.0, REG0_F7 + 400.0])
                        + self.head.pos;
                if self.demo_sub > REG6_S7 + 0x3C {
                    add_calc2(&mut self.fov, 20.0, 0.5, 1.0);
                } else if self.demo_sub > 0 {
                    add_calc2(&mut self.fov, 35.0, 0.35, 1.0);
                } else {
                    add_calc2(&mut self.fov, 45.0, 0.05, 1.0);
                }
            }
            0xA => {
                if !self.demo_gate() {
                    run = false;
                } else {
                    self.demo_state += 1;
                    self.demo_param = 0.0;
                    self.demo_count = 0;
                    self.demo_sub = 0;
                    self.fov = 55.0;
                    self.cam_eye = Vec3::new(2598.0, 9902.0, -3498.0);
                    self.cam_look = Vec3::new(2228.0, 9961.0, -3403.0);
                }
            }
            0xB => {
                add_calc2(&mut self.cam_eye.x, 164.0, 0.05, self.demo_param * 24.34);
                add_calc2(&mut self.cam_eye.y, 11617.0, 0.05, self.demo_param * 17.15);
                add_calc2(&mut self.cam_eye.z, -2802.0, 0.05, self.demo_param * 6.96);
                add_calc2(&mut self.cam_look.x, -167.0, 0.05, self.demo_param * 23.949999);
                add_calc2(&mut self.cam_look.y, 11791.0, 0.05, self.demo_param * 18.3);
                add_calc2(&mut self.cam_look.z, -2706.0, 0.05, self.demo_param * 6.97);
                if self.demo_sub > REG13_S1 + 0x1E {
                    add_calc2(&mut self.demo_param, REG13_F1 + 1.0, 1.0, REG13_F2 + 0.015);
                }
                if self.demo_sub >= REG13_S6 + 0xBE {
                    if self.demo_sub == REG13_S6 + 0xD2 {
                        out.bgm(STRM_DK_START);
                    }
                    if self.demo_sub == REG13_S6 + 0xBE {
                        out.shutter_smoke();
                    }
                    self.shake = REG13_F18 + 1.5;
                    out.se(SE_ATM_MJT_JINARI);
                    add_calc0(&mut self.shutter_ring, 1.0, REG13_F0 + 1.0);
                    self.shutter_roll = self.shutter_roll + 0.003 + REG0_F8;
                    rumble = true;
                }
                if self.demo_sub > REG13_S2 + 0xFA {
                    self.demo_state = 0xC;
                    self.cam_eye = Vec3::new(2195.0, 10007.0, -4079.0);
                    self.cam_look = Vec3::new(1816.0, 10051.0, -4137.0);
                    self.demo_param = 0.0;
                    self.demo_sub = 0;
                }
            }
            0xC => {
                add_calc0(&mut self.shutter_ring, 1.0, REG13_F0 + 1.0);
                rumble = true;
                self.shutter_roll = self.shutter_roll + REG0_F8 + 0.003;
                if self.demo_sub < REG13_S6 + 0x9B {
                    self.shake = REG13_F5 + 5.0;
                    out.se(SE_ATM_MJT_JINARI);
                }
                if self.demo_sub == REG13_S6 + 0x96 {
                    self.demo_flag = 1;
                }
                if self.demo_sub == REG13_S7 + 0xA0 {
                    self.demo_sub = 0;
                    self.demo_state = 0xD;
                }
                add_calc2(&mut self.cam_eye.x, 2682.0, 0.05, self.demo_param * 4.87);
                add_calc2(&mut self.cam_eye.y, 9904.0, 0.05, self.demo_param * 1.03);
                add_calc2(&mut self.cam_eye.z, -3600.0, 0.05, self.demo_param * 4.79);
                add_calc2(&mut self.cam_look.x, 2308.0, 0.05, self.demo_param * 4.92);
                add_calc2(&mut self.cam_look.y, 9961.0, 0.05, self.demo_param * 0.9);
                add_calc2(&mut self.cam_look.z, -3524.0, 0.05, self.demo_param * 6.1299998);
                add_calc2(&mut self.demo_param, REG13_F3 + 1.5, 1.0, REG13_F4 + 0.02);
            }
            0xD => {
                if self.demo_sub < REG0_S3 + 0x64 {
                    add_calc2(&mut self.fov, 25.0, 0.1, 0.5);
                } else {
                    add_calc2(&mut self.fov, 55.0, 0.2, 3.0);
                }
                if self.demo_sub == REG13_S8 + 0x37 {
                    self.demo_flag = 1;
                }
                add_calc2(&mut self.cam_eye.x, self.head.pos.x, 0.1, 50.0);
                add_calc2(
                    &mut self.cam_eye.y,
                    (self.head.pos.y - 200.0) + REG13_F11,
                    0.1,
                    50.0,
                );
                add_calc2(&mut self.cam_eye.z, self.head.pos.z, 0.1, 50.0);
                if self.morph.is_stop() {
                    self.anm_init(BCK_FLY3, 1.0, EMODE_LOOP, 1.0, 0xD, 0);
                }
                if self.demo_sub == REG13_S8 + 0xF0 {
                    self.demo_sub = 0;
                    self.demo_state = 0xE;
                }
                rumble = true;
            }
            0xE => {
                self.cam_eye = Vec3::new(3304.0, 10983.0, -3644.0);
                self.cam_look = Vec3::new(2841.0, 13058.0, -3272.0);
                self.demo_state = 0xF;
                self.demo_sub = 0;
                self.demo_param = 0.0;
                self.shutter_ring = REG13_F17 - 100.0;
                self.shutter_roll = 0.0;
                rumble = true;
            }
            0xF | 0x10 => {
                add_calc0(&mut self.shutter_ring, 1.0, REG6_F11 + 1.5);
                if self.shutter_ring.abs() < 1.0 {
                    add_calc0(&mut self.shutter_shift, 1.0, REG6_F12 + 1.0);
                    if self.demo_state == 0xF {
                        self.flags_on(out);
                        self.stage_boss_demo = true;
                        self.demo_state = 0x10;
                    }
                }
                add_calc2(&mut self.cam_look.x, 3152.0, 0.05, self.demo_param * 3.11);
                add_calc2(&mut self.cam_look.z, -3070.0, 0.05, self.demo_param * 2.02);
                add_calc2(&mut self.demo_param, REG13_F3 + 1.2, 1.0, REG13_F4 + 0.015);
                if self.demo_sub == 0xA0 {
                    self.demo_state = 0x96;
                    self.action = Action::Fly;
                    self.fly_ca = -1;
                    self.state = 0;
                    self.gravity = REG0_F4 + GRAVITY;
                    self.battle_mode = 0;
                    out.bgm(BGM_DK_BATTLE);
                }
            }
            0x32 => {
                if !self.demo_gate() {
                    run = false;
                } else {
                    self.demo_state += 1;
                    self.demo_param = 0.0;
                    self.demo_count = 0;
                    self.demo_sub = 0;
                    self.fov = 60.0;
                    self.cam_look = Vec3::new(3267.0, 9561.0, -4562.0);
                    self.cam_eye = actor.eye;
                    self.cam_eye.y -= REG0_F7 + 100.0;
                }
            }
            0x33 => {
                add_calc2(&mut self.cam_eye.x, actor.eye.x, 0.1, self.demo_param * 100.0);
                add_calc2(
                    &mut self.cam_eye.y,
                    actor.eye.y - (REG0_F7 + 100.0),
                    0.3,
                    self.demo_param * 200.0,
                );
                add_calc2(&mut self.cam_eye.z, actor.eye.z, 0.1, self.demo_param * 100.0);
                if self.action == Action::Start {
                    self.demo_state += 1;
                    self.demo_sub = 0;
                }
                add_calc2(&mut self.demo_param, 1.0, 1.0, REG0_F4 + 0.02);
            }
            0x34 => {
                if self.demo_sub >= 0x64 {
                    self.demo_state = 0x96;
                }
            }
            0x64 => {
                if !self.demo_gate() {
                    run = false;
                } else {
                    self.demo_state += 1;
                    self.demo_param = 0.0;
                    self.demo_count = 0;
                    self.demo_sub = 0;
                    self.fov = 45.0;
                    self.cam_look = Vec3::new(
                        env.player_pos.x,
                        env.player_pos.y,
                        env.player_pos.z,
                    );
                    self.cam_yaw = actor.angle[ANGLE_Y].wrapping_add(self.rnd_fx(4000.0) as i16);
                    self.demo_radius = 1500.0;
                }
            }
            0x65 => {
                if self.demo_sub == 0x3C {
                    out.bgm(STRM_DK_CLEAR);
                }
                self.cam_eye = self.head.pos;
                self.cam_eye.y += REG6_F16 - 100.0;
                let look = turned(
                    self.cam_pitch as i16,
                    [0.0, 0.0, self.demo_radius + REG6_F17],
                );
                let look = turned(self.cam_yaw, [look.x, look.y, look.z]);
                let look = look + self.cam_eye;
                add_calc2(&mut self.cam_look.x, look.x, 0.1, 100.0);
                if self.flown_high == 0 {
                    self.cam_look.y = REG6_F18 + 9950.0;
                } else {
                    add_calc2(&mut self.cam_look.y, look.y + REG6_F13, 0.1, 100.0);
                }
                add_calc2(&mut self.cam_look.z, look.z, 0.1, 100.0);
            }
            0x6E => {
                self.fov = 40.0;
                self.cam_look = Vec3::new(-25267.0, -948.0, 22916.0);
                self.demo_state = 0x6F;
            }
            0x6F => {
                self.cam_eye.x = AREA_X;
                if actor.pos.y < 30000.0 {
                    self.cam_eye.y = (actor.pos.y - 6000.0) + REG6_F19;
                }
                self.cam_eye.z = AREA_Z;
                add_calc2(&mut self.cam_look.z, 30449.0, 0.02, self.demo_param * 75.33);
                if self.demo_sub > REG6_S2 {
                    add_calc2(&mut self.demo_param, REG6_S1 + 1.5, 1.0, REG6_F2 + 0.02);
                }
            }
            0x70 => {
                self.cam_eye = self.target;
                self.demo_state += 1;
                self.demo_sub = 0;
                self.fov = 45.0;
            }
            0x71 => {
                if self.demo_sub == 0x78 {
                    let mut at = Vec3::new(AREA_X, 10300.0, AREA_Z);
                    at.y = 10300.0;
                    self.eff_hane_set(at, 0x27, -1);
                }
                if self.demo_sub > REG12_S7 + 0x43 {
                    add_calc2(&mut self.cam_eye.y, REG12_F4 + 9900.0, 0.5, REG12_F6 + 200.0);
                }
                self.cam_look = self.target;
                self.cam_look.y = REG12_F5 + 10000.0;
                self.cam_look.x += REG12_F7 + 500.0;
            }
            0x72 => {
                self.cam_eye = Vec3::new(4386.0, 10020.0, -4368.0);
                self.cam_look = Vec3::new(4144.0, 10061.0, -4131.0);
                if self.demo_sub == 10 {
                    self.stage_boss_enemy = true;
                    out.se(SE_OBJ_TOGE_IN);
                }
                if self.demo_sub == 0x28 {
                    self.demo_state = 0x96;
                }
            }
            0x96 => {
                self.demo_state = 0;
            }
            _ => {}
        }
        if self.demo_state != 0 && run {
            let fx = self.shake * sin_s(self.demo_count.wrapping_mul(0x3300));
            let fy = self.shake * cos_s(self.demo_count.wrapping_mul(0x3000));
            let fz = self.shake * sin_s(self.demo_count.wrapping_mul(0x3700));
            self.cam_look += Vec3::new(fx, fy, fz);
            self.cam_eye += Vec3::new(fx, fy, fz);
            add_calc0(&mut self.shake, 1.0, REG0_F16 + 1.0);
            self.demo_sub = self.demo_sub.wrapping_add(1);
            self.demo_count = self.demo_count.wrapping_add(1);
        }
        if rumble {
            out.se(SE_ATM_MJT_SHUTTER);
        }
    }

    fn flags_on(&mut self, out: &mut Sink<'_>) {
        out.flags.set(TMP_BIT_SHUTTER);
        out.se(SE_ATM_MJT_SHUTTER_END);
    }

    fn beak_step(&mut self, actor: &Actor, env: &Env<'_>) {
        let yaw = -actor.shape_angle[ANGLE_Y];
        let swing = self.head.pos - self.head_prev;
        let rot = turned(yaw, swing);
        let mut angle = (rot.z * (REG8_F15 + BEAK_SWING)) as i16;
        if angle > 8000 {
            angle = 8000;
        } else if angle < -8000 {
            angle = -8000;
        }
        let wobble = ((REG0_F5 + BEAK_WOBBLE) * self.beak_sway_calc
            * sin_s(self.beak_tick.wrapping_mul(REG8_S4 + 2500)))
            as i16;
        let rate = (self.beak_calc * 4000.0) as i16;
        add_calc2(&mut self.beak_calc, 1.0, 1.0, REG14_F4 + 0.01);
        if self.demo_sub > REG14_S9 + 0x41 {
            add_calc2(&mut self.beak_sway_calc, 1.0, 1.0, REG14_F2 + 0.4);
        }
        add_calc_angle_s2(
            &mut self.tosaka_y1,
            angle.wrapping_add(wobble).wrapping_add(4500),
            TURN_DIV_HEAD,
            rate,
        );
        add_calc_angle_s2(&mut self.tosaka_z1, 4, 10, rate);
        self.beak_tick = self.beak_tick.wrapping_add(1);
        if self.beak_timer != 0 {
            self.beak_amp = 0;
            self.beak_sway = 0;
            let reach = turned(self.beak_aim, [0.0, 0.0, self.beak_timer as f32 * (REG0_F7 + 500.0)]);
            let wave = sin_s(self.frame_count.wrapping_mul(REG0_S4 + 7000));
            self.tosaka_y4 = (reach.z * wave) as i16;
            self.tosaka_z4 = (reach.x * wave) as i16;
        }
        if 3000 < self.beak_amp {
            self.beak_tick = self.beak_tick.wrapping_add(1);
        }
        let sway = self.beak_sway;
        self.tosaka_y2 = (sway as f32 * sin_s(self.beak_tick.wrapping_mul(1500))) as i16;
        self.tosaka_z2 = (sway as f32 * cos_s(self.beak_tick.wrapping_mul(1200))) as i16;
        self.tosaka_y3 =
            (sway as f32 * sin_s(self.beak_tick.wrapping_mul(1500).wrapping_add(0x4000))) as i16;
        self.tosaka_z3 =
            (sway as f32 * cos_s(self.beak_tick.wrapping_mul(1200).wrapping_add(0x4000))) as i16;
        add_calc_angle_s2(&mut self.beak_sway, self.beak_amp, 2, 500);
        self.beak_amp = 1500;
        let _ = env;
    }

    fn attention_step(&mut self, actor: &mut Actor, env: &Env<'_>) {
        self.eye = Self::at_pos(actor.shape_angle[ANGLE_Y], JOINT_ATAMA1)
            + turned(
                actor.shape_angle[ANGLE_Y],
                [REG0_F10 + 120.0, 0.0, REG0_F12 + EYE_OFFSET[2]],
            );
        let angle = if self.track_player {
            let delta = env.player_pos - actor.pos;
            let mut turn = bearing(Vec3::ZERO, delta).wrapping_sub(actor.shape_angle[ANGLE_Y]);
            if turn > TRACK_LIMIT {
                turn = TRACK_LIMIT;
            } else if turn < -TRACK_LIMIT {
                turn = -TRACK_LIMIT;
            }
            turn
        } else {
            0
        };
        self.track_player = false;
        add_calc_angle_s2(&mut self.head_pitch, angle, TURN_DIV_HEAD, 0x800);
        add_calc_angle_s2(&mut self.head_yaw, 0, TURN_DIV_NECK, 0x200);
    }

    fn hurt_player(&self, env: &Env<'_>, out: &mut Sink<'_>) {
        if self.at_mode == 1
            && touches(self.beak, self.beak_radius, env.player_pos, PLAYER_BODY_RADIUS)
        {
            out.health.damage(AT_POWER_HEAD);
        }
        for index in 0..FOOT_COUNT {
            if self.foot_at[index]
                && touches(self.foot[index], self.foot_radius[index], env.player_pos, PLAYER_BODY_RADIUS)
            {
                out.health.damage(AT_POWER_FOOT);
            }
        }
    }

    pub fn execute(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        self.smoke_pos = actor.pos;
        self.smoke_pos.y = AREA_Y;
        self.frame_count = self.frame_count.wrapping_add(1);
        for slot in 0..TIMER_COUNT {
            timer(&mut self.timers[slot]);
        }
        if self.hit_stop != 0 {
            self.hit_stop -= 1;
        }
        if self.mask_timer != 0 {
            self.mask_timer -= 1;
        }
        if self.beak_timer != 0 {
            self.beak_timer -= 1;
        }
        if !HIO_M005 {
            self.morph.play();
            self.move_actor(actor, env, out);
            if self.action != Action::Start && self.action.code() < ACTION_LIMIT {
                let from = actor.pos;
                let (next, _wall) = env.corrected(from, actor.pos);
                actor.pos = next;
                if env.landed(actor.pos) {
                    self.speed.y = -10.0;
                    self.ground_hit = 3;
                }
                if self.ground_hit != 0 {
                    self.ground_hit -= 1;
                }
            }
            self.damage_check(actor, env, out);
        }
        self.shape_angle[ANGLE_X] = 0;
        self.col_set(actor);
        if self.beak_free != 0 {
            self.beak_step(actor, env);
        } else {
            self.beak_timer = 0;
            self.beak_amp = 0;
            self.beak_sway = 0;
        }
        self.shape_angle[ANGLE_Y] = actor.angle[ANGLE_Y];
        self.shape_angle[ANGLE_Z] = actor.angle[ANGLE_Z];
        self.attention_step(actor, env);
        self.hurt_player(env, out);
        self.kamen_break_move(env, out);
        self.my_effect_move(env);
        self.obj_move();
        self.demo_camera(actor, env, out);
        self.kankyo_cont(actor);
        for slot in 0..SMOKE_COUNT {
            timer(&mut self.smoke_timer[slot]);
        }
        if self.rise_smoke != 0 {
            self.rise_smoke -= 1;
        }
        if self.nail_live != 0 {
            out.se(SE_CM_DK_NAIL);
            if self.foot[0].y <= 9810.0 || self.at_mode == 0 {
                self.nail_live = 0;
            }
        }
        self.at_mode = 0;
    }

    pub fn begin(&mut self, actor: &mut Actor, out: &mut Sink<'_>) {
        self.entered = true;
        if self.stage_boss_enemy || REG0_S0 != 0 {
            self.stage_boss_enemy = true;
            self.action = Action::AfterFight;
            actor.pos = Vec3::splat(300000.0);
            self.shutter_ring = SHUTTER_BATTLE;
            self.shutter_shift = SHUTTER_SHIFT;
            self.shutter_y = SHUTTER_BATTLE_Y;
        } else if self.battle_mode == 0x1 {
            self.action = Action::TFly;
            actor.pos = Vec3::new(AREA_X, SPAWN_HEIGHT, AREA_Z);
            self.shutter_ring = SHUTTER_BATTLE;
            self.shutter_shift = SHUTTER_SHIFT;
            out.bgm(BGM_MJ_TOWER_BATTLE);
            self.timers[0] = SPAWN_TIMER;
        } else {
            self.gravity = REG0_F4 + GRAVITY;
            actor.speed_f_walk = HIO_M020;
            self.fly_ca = -1;
            self.tosaka_y1 = -2000;
            self.health = MAX_HEALTH;
            self.max_health = MAX_HEALTH;
            actor.pos.y = 11000.0;
        }
    }
}

const SHUTTER_BATTLE: f32 = -630.0;
const SHUTTER_SHIFT: f32 = -70.0;
const SPAWN_HEIGHT: f32 = 5000.0;
const SPAWN_TIMER: i16 = 0x168;
const PURGE_NAMES: [&str; 6] = ["Kamome", "Dk", "Kui", "Bk", "Boko", "bonbori"];
const TIDE_NAMES: [&str; 6] = ["Humi0z", "Humi4z", "Humi5z", "Tide3", "Humi2z", "Humi3z"];

fn blow_of(watch: &Watch<'_, '_>) -> Blow {
    let mut at = At::Idle;
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
    } else if watch
        .bolts
        .iter()
        .any(|bolt| bolt.parameter == boomerang::MODE_THROW)
    {
        at = At::Boomerang;
    } else if watch.hammer.kind == hammer::CUT_HAMMER_SIDESWING
        || watch.hammer.kind == hammer::CUT_HAMMER_FRONTSWING
    {
        at = At::Hammer;
    } else if watch.combat.active && watch.combat.frame >= 1 && watch.combat.frame < 6 {
        at = At::Sword;
    } else if !watch.blasts.list.is_empty() {
        at = At::Bomb;
    }
    Blow {
        at,
        blade: watch.held.item,
        soup: watch.combat.soup,
    }
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    combat: Res<'w, Combat>,
    hammer: Res<'w, hammer::Cut>,
    held: Res<'w, Held>,
    blasts: Res<'w, bombflower::Blasts>,
    players: Query<'w, 's, (Entity, &'static mut Player)>,
    shafts: Query<'w, 's, &'static arrow::Arrow>,
    bolts: Query<'w, 's, &'static boomerang::Boomerang>,
    others: Query<'w, 's, (Entity, &'static Actor), Without<Helmaroc>>,
}

pub fn control(
    watch: Watch<'_, '_>,
    mut switches: ResMut<Switches>,
    mut stories: ResMut<StoryBits>,
    mut flags: ResMut<EventFlags>,
    mut zones: ResMut<ZoneActors>,
    mut health: ResMut<Health>,
    mut events: ResMut<SimEvents>,
    mut commands: Commands,
    mut bosses: Query<
        (Entity, &mut Actor, &mut Helmaroc),
        (With<Helmaroc>, Without<Player>),
    >,
) {
    let solid = watch.collision.as_deref();
    let blow = blow_of(&watch);
    let mut players = watch.players;
    let Some(entry) = players.iter_mut().next() else {
        return;
    };
    let mut player = entry.1;
    let mut orders = Orders::default();
    let mut drop: Vec<(Entity, i8, u16)> = Vec::new();
    for (_, actor, boss) in &mut bosses {
        let actor = &mut *actor;
        let boss = &mut *boss;
        let peck = watch
            .others
            .iter()
            .find(|(id, other)| {
                other.name == BDK_OBJ_ACTOR
                    && (other.parameters & PRM_BYTE_MASK) != PRM_MASK_SET
                    && Some(*id) != boss.peck_done
            })
            .map(|(id, other)| (id, other.pos));
        let env = Env {
            player_pos: player.pos,
            player_angle: to_s16(player.face),
            player_hover: false,
            solid,
            blow,
            peck,
        };
        let mut sink = Sink {
            events: &mut events,
            switches: &mut switches,
            stories: &mut stories,
            flags: &mut flags,
            zones: &mut zones,
            health: &mut health,
            orders: &mut orders,
            drop: &mut drop,
        };
        if !boss.entered {
            boss.begin(actor, &mut sink);
        }
        boss.execute(actor, &env, &mut sink);
    }
    if orders.purge_mobs || orders.purge_shards {
        for (id, other) in watch.others.iter() {
            let band = other.parameters & PRM_BYTE_MASK;
            let mob = PURGE_NAMES.contains(&other.name.as_str())
                || TIDE_NAMES.contains(&other.name.as_str())
                || (other.name == BDK_OBJ_ACTOR && band >= PRM_NIBBLE_TWO);
            let shard = other.name == BDK_OBJ_ACTOR && band != PRM_MASK_SET;
            if (orders.purge_mobs && mob) || (orders.purge_shards && shard) {
                drop.push((id, other.room, other.set_id));
            }
        }
    }
    for (id, room, set_id) in drop {
        zones.mark_done(room, set_id);
        let _ = commands.entity(id).despawn();
    }
}
