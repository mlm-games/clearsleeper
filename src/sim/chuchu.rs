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
use super::flags::{Switches, ZoneActors};
use super::ground;
use super::hammer;
use super::health::Health;
use super::hookshot;
use super::item::{FIRST_ARROW, Held, LAST_ARROW};
use super::pickup::{ITEM_ACTORS, Pickup};
use super::player::Player;
use super::roll;
use super::sea::Sea;
use super::stream;
use super::taglight::{self, TagLight};
use super::tsubo;
use super::walk;
use super::water;

pub const CHUCHU_ACTORS: [&str; 3] = ["c_black", "c_blue", "c_green"];
pub const CHUCHU_COLOURS: [&str; 5] = ["c_green", "c_red", "c_blue", "c_black", "c_kiiro"];

const BEHAVIOUR_SHIFT: u32 = 0x00;
const COLOUR_SHIFT: u32 = 0x08;
const RANGE_SHIFT: u32 = 0x10;
const SWITCH_SHIFT: u32 = 0x18;
const BYTE_MASK: u32 = 0xFF;
const NO_BYTE: u8 = 0xFF;
const COLOUR_BAND: u8 = 10;

pub const BEHAVIOUR_WALK: u8 = 0;
pub const BEHAVIOUR_RISE: u8 = 1;
pub const BEHAVIOUR_CLIMB: u8 = 2;
pub const BEHAVIOUR_LEASH: u8 = 3;
pub const BEHAVIOUR_POT: u8 = 4;

pub const COLOUR_GREEN: u8 = 0;
pub const COLOUR_RED: u8 = 1;
pub const COLOUR_BLUE: u8 = 2;
pub const COLOUR_BLACK: u8 = 3;
pub const COLOUR_YELLOW: u8 = 4;
pub const COLOUR_TREASURE: u8 = 5;

pub const CRITICAL_NONE: u8 = 0;
pub const CRITICAL_SWALLOW: u8 = 1;
pub const CRITICAL_DONE: u8 = 2;

const AT_BOOMERANG: u32 = 1 << 6;
const AT_WATER: u32 = 1 << 8;
const AT_FIRE_ARROW: u32 = 1 << 18;
const AT_NORMAL_ARROW: u32 = 1 << 14;
const AT_HOOKSHOT: u32 = 1 << 15;
const AT_SKULL_HAMMER: u32 = 1 << 16;
const AT_UNK20000: u32 = 1 << 17;
const AT_ICE_ARROW: u32 = 1 << 19;
const AT_LIGHT_ARROW: u32 = 1 << 20;
const AT_WIND: u32 = 1 << 21;
const AT_UNK400000: u32 = 1 << 22;
const AT_LIGHT: u32 = 1 << 23;
const AT_GRAPPLING_HOOK: u32 = 1 << 27;

pub const TG_ALL: u32 = !(AT_UNK400000 | AT_UNK20000 | AT_WATER);
pub const TG_LIE: u32 = (1 << 5)
    | AT_BOOMERANG
    | AT_NORMAL_ARROW
    | AT_HOOKSHOT
    | AT_SKULL_HAMMER
    | AT_FIRE_ARROW
    | AT_ICE_ARROW
    | AT_LIGHT_ARROW
    | AT_LIGHT
    | AT_GRAPPLING_HOOK;
pub const TG_FLOAT: u32 = AT_GRAPPLING_HOOK
    | AT_WIND
    | AT_LIGHT_ARROW
    | AT_ICE_ARROW
    | AT_FIRE_ARROW
    | AT_HOOKSHOT
    | AT_NORMAL_ARROW
    | AT_BOOMERANG;
pub const TG_POT: u32 = AT_LIGHT | (1 << 1) | (1 << 2) | (1 << 5);

const RANGE_DEFAULT: f32 = 1000.0;
const RANGE_RISE: f32 = 1200.0;
const RANGE_SCALE: f32 = 10.0;
const RANGE_FREE: f32 = 60000.0;
const LEASH_SCALE: f32 = 10.0;
const LEASH_MIN: f32 = 100.0;
const LEASH_NEAR: f32 = 100.0;
const LEASH_GATE: f32 = 0.1;
const LEASH_TURN_GATE: u16 = 0x100;
const LEASH_JITTER: f32 = 16384.0;
const TURN_NEAR: f32 = 40.0;
const WANDER_BAND: f32 = 7.0 * 0x500;

const HEALTH: [i32; 5] = [2, 1, 2, 4, 1];
const AT_POWER: [u8; 5] = [1, 1, 2, 2, 2];
const STEAL_DEFAULT: i16 = 1;
const STEAL_BLACK: i16 = 3;

pub const BCK_ATTACK01: i32 = 0x14;
pub const BCK_ATTACK02: i32 = 0x15;
pub const BCK_ATTACK03: i32 = 0x16;
pub const BCK_BETA: i32 = 0x17;
pub const BCK_PTCL: i32 = 0x18;
pub const BCK_DEKU_START: i32 = 0x19;
pub const BCK_HUKKATSU: i32 = 0x1A;
pub const BCK_HUSE2TACHI: i32 = 0x1B;
pub const BCK_HUSE_WALK: i32 = 0x1C;
pub const BCK_MAHI: i32 = 0x1D;
pub const BCK_START: i32 = 0x1E;
pub const BCK_TACHI2HUSE: i32 = 0x1F;
pub const BCK_TACHI_WALK: i32 = 0x20;
pub const BCK_TSTART01: i32 = 0x21;
pub const BCK_TSTART02: i32 = 0x22;

pub const FRAME_ATTACK01: i16 = 15;
pub const FRAME_ATTACK02: i16 = 18;
pub const FRAME_ATTACK03: i16 = 15;
pub const FRAME_BETA: i16 = 25;
pub const FRAME_PTCL: i16 = 19;
pub const FRAME_DEKU_START: i16 = 56;
pub const FRAME_HUKKATSU: i16 = 20;
pub const FRAME_HUSE2TACHI: i16 = 23;
pub const FRAME_HUSE_WALK: i16 = 44;
pub const FRAME_MAHI: i16 = 20;
pub const FRAME_START: i16 = 56;
pub const FRAME_TACHI2HUSE: i16 = 25;
pub const FRAME_TACHI_WALK: i16 = 44;
pub const FRAME_TSTART01: i16 = 24;
pub const FRAME_TSTART02: i16 = 43;

const EMODE_NONE: u8 = 0;
const EMODE_LOOP: u8 = 2;
const STATE_STOP: u8 = 1;
const STATE_LOOP: u8 = 2;

pub const NORMAL_STEP_START: u8 = 0x00;
pub const NORMAL_STEP_ALERT: u8 = 0x01;
pub const NORMAL_STEP_RISE: u8 = 0x02;
pub const NORMAL_STEP_RISE_WAIT: u8 = 0x03;
pub const NORMAL_STEP_SETTLE: u8 = 0x04;
pub const NORMAL_STEP_WALK: u8 = 0x05;
pub const NORMAL_STEP_LIE: u8 = 0x06;
pub const NORMAL_STEP_LIE_WAIT: u8 = 0x07;
pub const NORMAL_STEP_SLIDE: u8 = 0x08;
pub const NORMAL_STEP_SLIDE_WAIT: u8 = 0x09;
pub const NORMAL_STEP_STAND: u8 = 0x0A;
pub const NORMAL_STEP_UP: u8 = 0x0B;

pub const SWIM_STEP_LIE: u8 = 0x14;
pub const SWIM_STEP_DRIFT: u8 = 0x15;

pub const ATTACK_STEP_BEGIN: u8 = 0x1E;
pub const ATTACK_STEP_WINDUP: u8 = 0x1F;
pub const ATTACK_STEP_JUMP: u8 = 0x20;
pub const ATTACK_STEP_AIR: u8 = 0x21;
pub const ATTACK_STEP_LAND: u8 = 0x22;

pub const DAMAGE_STEP_BEGIN: u8 = 0x28;
pub const DAMAGE_STEP_KNOCK: u8 = 0x29;
pub const DAMAGE_STEP_WAIT: u8 = 0x2A;
pub const DAMAGE_STEP_PIYO: u8 = 0x2B;
pub const DAMAGE_STEP_GRAPPLE: u8 = 0x2C;
pub const DAMAGE_STEP_SPLIT: u8 = 0x2D;
pub const DAMAGE_STEP_SPLIT_DONE: u8 = 0x2E;

pub const DEAD_STEP_BEGIN: u8 = 50;
pub const DEAD_STEP_DROP: u8 = 51;
pub const DEAD_STEP_BETA: u8 = 52;
pub const DEAD_STEP_BETA_DONE: u8 = 53;

pub const CLIMB_STEP_BEGIN: u8 = 0x3C;
pub const CLIMB_STEP_WAIT: u8 = 0x3D;
pub const CLIMB_STEP_READY: u8 = 0x3E;
pub const CLIMB_STEP_WALK: u8 = 0x3F;
pub const CLIMB_STEP_START: u8 = 0x40;
pub const CLIMB_STEP_LEAP: u8 = 0x41;
pub const CLIMB_STEP_AIR: u8 = 0x42;
pub const CLIMB_STEP_DONE: u8 = 0x43;
pub const CLIMB_STEP_HUG: u8 = 0x44;
pub const CLIMB_STEP_FADE: u8 = 0x45;
pub const CLIMB_STEP_RETURN: u8 = 0x46;

pub const STONE_STEP_BEGIN: u8 = 0x50;
pub const STONE_STEP_POSE: u8 = 0x51;
pub const STONE_STEP_WAIT: u8 = 0x52;
pub const STONE_STEP_FREE: u8 = 0x53;
pub const STONE_STEP_THROW: u8 = 0x54;

pub const HELD_STEP_WAIT: u8 = 100;
pub const HELD_STEP_RELEASE: u8 = 101;

pub const POT_STEP_SEARCH: u8 = 0x6E;
pub const POT_STEP_FOLLOW: u8 = 0x6F;
pub const POT_STEP_LIVE: u8 = 0x70;

const REACT_NONE: u8 = 0;
const REACT_SWORD: u8 = 1;
const REACT_PIYO: u8 = 2;
const REACT_WIND: u8 = 3;
const REACT_BOOMERANG: u8 = 4;
const REACT_STONED: u8 = 5;
const REACT_HAMMER: u8 = 6;
const REACT_HOOKSHOT: u8 = 7;
const REACT_FIRE: u8 = 8;
const REACT_GRAPPLE: u8 = 9;

const TIMER_HOLD: usize = 0;
const TIMER_REST: usize = 1;
const TIMER_WOBBLE: usize = 2;
const TIMER_SPARK: usize = 3;
const TIMER_SPLIT: usize = 4;
const TIMER_DEMO: usize = 5;
const TIMER_LIE: usize = 6;
const TIMER_COUNT: usize = 7;
const SHAKE_COUNT: usize = 5;

const GRAVITY_GROUND: f32 = -3.0;
const GRAVITY_RISE: f32 = -11.0;
const MAX_FALL: f32 = -55.0;
const CLIMB_RISE: f32 = 1.0;
const RISE_FRAMES: f32 = 22.0;
const SPIN_STEP: i16 = 0x1000;
const TURN_STEP: i16 = 0x400;
const ATTACK_TURN: i16 = 0x1000;
const THROW_TURN: i16 = 0x1000;
const MOVE_TARGET: f32 = 2.0;
const MOVE_SCALE: f32 = 0.2;
const MOVE_STEP: f32 = 0.5;
const SLIDE_SPEED: f32 = 1.0;
const WALK_HOLD: i16 = 10;
const WALK_SPAN: f32 = 15.0;
const SLIDE_SPAN: f32 = 15.0;
const SLIDE_BASE: i16 = 0x46;
const LIE_SPAN: i16 = 5;
const ATTACK_RANGE: f32 = 380.0;
const ATTACK_WIND: i16 = 10;
const ATTACK_SPEED: f32 = 18.0;
const ATTACK_LIFT: f32 = 25.0;
const ATTACK_SHIELD: f32 = -0.5;
const KNOCK_DEFAULT: f32 = 7000.0;
const KNOCK_SWORD: f32 = 16000.0;
const KNOCK_SPEED_SWORD: f32 = 60.0;
const KNOCK_RATE_SWORD: f32 = 500.0;
const KNOCK_RATE_BASE: f32 = 250.0;
const KNOCK_SPEED_BASE: f32 = 20.0;
const KNOCK_SCALE: f32 = 0.8;
const KNOCK_RATE_STEP: f32 = 4.0;
const KNOCK_MIN: f32 = 1.0;
const TILT_STEP: i16 = 0x3000;
const TILT_WIND: i16 = 10000;
const BACK_TURN: i16 = -0x8000;
const PIYO_BASE: i16 = 0x2D;
const PIYO_SPAN: i16 = 0x2D;
const HUKKATSU_BASE: i16 = 0x3C;
const GRAPPLE_BASE: i16 = 0x14;
const SWAY_STEP: i16 = 0x800;
const SWAY_SCALE: f32 = 3.0;
const SWAY_BASE: f32 = -15.0;
const FLOAT_SCALE: f32 = 1.0;
const FLOAT_MAX_STEP: f32 = 10.0;
const FLOAT_DEPTH: f32 = -20.0;
const FLOAT_RATE: f32 = 6.0;
const FLOAT_EPSILON: f32 = 2.0;
const FLOAT_DRIVE: f32 = 1.0;
const FLOAT_GAIN: f32 = 0.3;
const FLOAT_FALL: f32 = 2.0;
const ABYSS_Y: f32 = -500.0;
const ABYSS_LIMIT: u8 = 0x32;
const ABYSS_DEPTH: f32 = -20000.0;
const WATER_SINK: f32 = 40.0;
const WATER_SINK_STONE: f32 = 100.0;
const SHOCK_REACH: f32 = 1000.0;
const SPARK_SPAN: f32 = 42.0;
const SPARK_BASE: f32 = 30.0;
const BODY_RADIUS: f32 = 35.0;
const BODY_HEIGHT: f32 = 100.0;
const BODY_HEIGHT_LIE: f32 = 30.0;
const STONE_LIFT: f32 = 12.0;
const STONE_SPEED: f32 = 20.0;
const STONE_BRAKE_SCALE: f32 = 0.5;
const STONE_BRAKE_MAX: f32 = 1.0;
const STONE_HOLD: i16 = 600;
const STONE_RECOVER: i16 = 0x28;
const STONE_SPAN: i16 = 20000;
const STONE_ROLL: f32 = 2000.0;
const STONE_FRAME_MAX: u8 = 0x19;
const STONE_RISE: f32 = 10.0;
const HOLD_DELAY: i16 = 0x3C;
const CLIMB_SCALE_STEP: f32 = 0.1;
const CLIMB_SCALE_MAX: f32 = 0.2;
const CLIMB_SCALE_GATE: f32 = 0.1;
const CLIMB_JUMP: f32 = 0x1000;
const CLIMB_SPAN: f32 = 43.0;
const CLIMB_RATE: f32 = 1.0;
const CLIMB_LEAP_SPEED: f32 = 15.0;
const CLIMB_LEAP_LIFT: f32 = 50.0;
const CLIMB_LEAP_DELAY: i16 = 600;
const LIFE_LOW: u8 = 4;
const ARROW_LOW: u8 = 10;
const DROP_GATE: f32 = 1.0;
const DROP_HALF: f32 = 0.5;
const DIS_IBALL: u8 = 0;
const DIS_HEART: u8 = 10;
const DIS_ARROW: u8 = 12;
const DIS_NONE13: u8 = 13;
const ITEM_HEART: u8 = 0x00;
const ITEM_ARROW_10: u8 = 0x10;
const ITEM_DROP_LIFT: f32 = 60.0;

const ICE_FREEZE: i16 = 200;
const LIGHT_START: u8 = 1;
const LIGHT_SHRINK_END: u8 = 70;
const LIGHT_GONE: u8 = 90;
const LIGHT_DEAD: i32 = -0x80;
const LIGHT_SCALE: f32 = 0.5;
const LIGHT_LIFT: f32 = 80.0;
const LIGHT_SHRINK_SCALE: f32 = 0.1;
const LIGHT_SHRINK_MAX: f32 = 0.01;
const LIGHT_RISE_TARGET: f32 = 5.0;
const LIGHT_RISE_SCALE: f32 = 0.1;
const LIGHT_RISE_MAX: f32 = 1.0;
const LIGHT_FADE_SCALE: f32 = 0.1;
const LIGHT_FADE_MAX: f32 = 0.05;
const BURN_FRAMES: i16 = 100;
const CONTACT_GAP: i16 = 4;
const PLAYER_REACH: f32 = 35.0;
const REACH_SWORD: f32 = 20.0;
const REACH_ARROW: f32 = 30.0;
const REACH_HOOK: f32 = 5.0;
const LAMP_REACH: f32 = 2000.0;

pub const SE_LK_SW_HIT_S: u32 = 0x2803;
const SE_LK_W_WEP_HIT: u32 = 0x2833;
const SE_LK_MS_WEP_HIT: u32 = 0x2834;
const SE_LK_HAMMER_HIT: u32 = 0x2855;
const SE_CV_CC_STAND_UP: u32 = 0x4860;
const SE_CV_CC_ATTACK: u32 = 0x4861;
const SE_CV_CC_DAMAGE: u32 = 0x4862;
const SE_CV_CC_DIE: u32 = 0x4863;
const SE_CV_CC_PARASITE: u32 = 0x48BB;
const SE_CM_MD_PIYO: u32 = 0x50BC;
const SE_CM_FREEZE: u32 = 0x58FA;
const SE_CM_CC_ENTER_GND: u32 = 0x585D;
const SE_CM_CC_LANDING: u32 = 0x585E;
const SE_CM_CC_STAND_TO_LIE: u32 = 0x585F;
const SE_CM_CC_LIE_TO_STAND: u32 = 0x5860;
const SE_CM_CC_MOVE_LIE: u32 = 0x5861;
const SE_CM_CC_MOVE_STAND: u32 = 0x5862;
const SE_CM_CC_ATTACK: u32 = 0x5863;
const SE_CM_CC_DMG_SWING: u32 = 0x5864;
const SE_CM_CC_DIE_SWING: u32 = 0x5865;
const SE_CM_CC_BLUE_SPARK: u32 = 0x5866;
const SE_CM_CC_STONED: u32 = 0x58FB;
const SE_CM_CC_STONED_RECOVER: u32 = 0x58FC;
const SE_CM_B_CC_RECOVER: u32 = 0x591E;
const SE_L_ARROW_SHRINK: u32 = 0x5904;
const SE_L_ARROW_PASS: u32 = 0x5905;
const SE_OBJ_FALL_WATER_M: u32 = 0x6919;
const SE_OBJ_PUT_STONE: u32 = 0x6929;
const SE_OBJ_BREAK_STONE: u32 = 0x692B;

const REG8_S2: i16 = 0;
const REG8_S4: i16 = 0;
const REG8_S5: i16 = 0;
const REG8_S6: i16 = 0;
const REG8_S8: i16 = 0;
const REG8_S9: i16 = 0;
const REG8_F17: f32 = 0.0;
const REG12_F0: f32 = 0.0;
const REG12_F19: f32 = 0.0;
const DEMO_SELECT_10_3: f32 = 3.0;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

pub const CULL_MIN: [f32; 3] = [-35.0, -20.0, -35.0];
pub const CULL_MAX: [f32; 3] = [35.0, 120.0, 35.0];

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = (seed[0]) as f32 / RND_SCALE[0]
        + (seed[1]) as f32 / RND_SCALE[1]
        + (seed[2]) as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

fn rnd_f(seed: &mut [i32; 3], range: f32) -> f32 {
    rnd(seed) * range
}

fn rnd_fx(seed: &mut [i32; 3], range: f32) -> f32 {
    range * (rnd(seed) - 0.5) * 2.0
}

fn add_calc0(value: &mut f32, scale: f32, max_step: f32) {
    let step = (*value * scale).clamp(-max_step, max_step);
    *value -= step;
}

fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value != target {
        let step = (scale * (target - *value)).clamp(-max_step, max_step);
        *value += step;
    }
}

fn frame_max(bck: i32) -> i16 {
    match bck {
        BCK_ATTACK01 => FRAME_ATTACK01,
        BCK_ATTACK02 => FRAME_ATTACK02,
        BCK_ATTACK03 => FRAME_ATTACK03,
        BCK_BETA => FRAME_BETA,
        BCK_PTCL => FRAME_PTCL,
        BCK_DEKU_START => FRAME_DEKU_START,
        BCK_HUKKATSU => FRAME_HUKKATSU,
        BCK_HUSE2TACHI => FRAME_HUSE2TACHI,
        BCK_HUSE_WALK => FRAME_HUSE_WALK,
        BCK_MAHI => FRAME_MAHI,
        BCK_START => FRAME_START,
        BCK_TACHI2HUSE => FRAME_TACHI2HUSE,
        BCK_TACHI_WALK => FRAME_TACHI_WALK,
        BCK_TSTART01 => FRAME_TSTART01,
        BCK_TSTART02 => FRAME_TSTART02,
        _ => FRAME_START,
    }
}

fn search_angle(from: Vec3, to: Vec3) -> i16 {
    stream::angle_s((to.x - from.x).atan2(to.z - from.z))
}

fn search_distance(from: Vec3, to: Vec3) -> f32 {
    (to - from).length()
}

fn distance_angle_s(x: i16, y: i16) -> u16 {
    x.wrapping_sub(y).unsigned_abs()
}

fn touches(centre: Vec3, radius: f32, at: Vec3, reach: f32) -> bool {
    walk::within(centre, at, radius + reach)
}

fn cyl_touches(centre: Vec3, radius: f32, height: f32, at: Vec3, reach: f32) -> bool {
    let dx = at.x - centre.x;
    let dz = at.z - centre.z;
    let sum = radius + reach;
    if sum * sum < dx * dx + dz * dz {
        return false;
    }
    at.y + reach >= centre.y && at.y - reach <= centre.y + height
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

    fn set_start(&mut self, start: i16) {
        self.start = start;
        self.frame = (start) as f32;
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
            walk::chase_f(&mut self.cur_morf, 1.0, self.morf_step);
        }
        self.update();
    }

    fn is_stop(&self) -> bool {
        self.state & STATE_STOP != 0 || self.rate == 0.0
    }

    fn check_pass(&self, pass: f32) -> bool {
        let start = (self.start) as f32;
        let end = (self.end) as f32;
        let cur = self.frame;
        let mut next = cur + self.rate;
        if next < start {
            next = start;
        }
        if next >= end {
            next = end - 0.001;
        }
        if cur <= next {
            cur <= pass && pass < next
        } else {
            next <= pass && pass < cur
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Normal,
    Swim,
    Attack,
    Damage,
    Dead,
    Climb,
    Stoned,
    Held,
    Pot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum At {
    Idle,
    Sword,
    Stick,
    Hook,
    FanLight,
    Arrow,
    FireArrow,
    IceArrow,
    Hammer,
    Bomb,
    Boomerang,
}

impl At {
    fn reach(self) -> f32 {
        match self {
            Self::Idle => 0.0,
            Self::FanLight => 0.0,
            Self::Hook => REACH_HOOK,
            Self::Arrow | Self::FireArrow | Self::IceArrow => REACH_ARROW,
            Self::Sword | Self::Stick => REACH_SWORD,
            Self::Hammer => hammer::AT_RADIUS,
            Self::Bomb => bombflower::BLAST_RADIUS,
            Self::Boomerang => boomerang::CAPSULE_RADIUS,
        }
    }

    fn power(self, blade: u8, soup: bool) -> i32 {
        match self {
            Self::Sword | Self::Stick => damage::sword_power(blade, soup),
            Self::Arrow | Self::FireArrow | Self::IceArrow => damage::ARROW,
            Self::Bomb => damage::BOMB,
            Self::Boomerang => damage::BOMERANG,
            Self::Hammer => hammer::AT_POWER,
            Self::Idle | Self::Hook | Self::FanLight => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Blow {
    at: At,
    blade: u8,
    soup: bool,
    turn: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Light {
    pub timer: u8,
    pub scale_xz: f32,
    pub scale_y: f32,
    pub particle_scale: f32,
    pub y_offset: f32,
}

impl Light {
    fn idle() -> Self {
        Self {
            timer: 0,
            scale_xz: 1.0,
            scale_y: 1.0,
            particle_scale: 1.0,
            y_offset: LIGHT_LIFT,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Chuchu {
    pub behaviour: u8,
    pub colour: u8,
    pub range_byte: u8,
    pub spawn_switch: u8,
    pub state: Step,
    pub step: u8,
    pub reaction: u8,
    pub spark: u8,
    pub floaty: u8,
    pub shadow: u8,
    pub abyss: u8,
    pub climb_step: u8,
    pub come_demo: u8,
    pub ret_demo: u8,
    pub stone: u8,
    pub stone_frame: u8,
    pub splashed: bool,
    pub particle_model: u8,
    pub shock_lock: u8,
    pub critical: u8,
    pub strike_frames: i16,
    pub pot: Option<Entity>,
    pub wobble: Vec3,
    pub slope: Vec3,
    pub shape: [i16; 3],
    pub anim_res: i32,
    pub morph: Frame,
    pub beta: Frame,
    pub split: Frame,
    pub light: Light,
    pub water_step: f32,
    pub hover: f32,
    pub knock: f32,
    pub knock_rate: f32,
    pub scale: f32,
    pub carry_y: f32,
    pub notice_range: f32,
    pub leash: f32,
    pub walk_frame: f32,
    pub tilt: i16,
    pub tilt_base: i16,
    pub wander: i16,
    pub timers: [i16; TIMER_COUNT],
    pub shakes: [i16; SHAKE_COUNT],
    pub home: Vec3,
    pub speed: Vec3,
    pub speed_f: f32,
    pub gravity: f32,
    pub touch: bool,
    pub contact: bool,
    pub shield: bool,
    pub at_set: bool,
    pub at_type: u32,
    pub lock: bool,
    pub body_hit: bool,
    pub entered: bool,
    pub doomed: bool,
    pub dropped: bool,
    pub shattered: bool,
    pub health: i32,
    pub max_health: i32,
    pub steal_left: i16,
    pub steal_bit: u8,
    pub freeze: i16,
    pub burn: i16,
    pub contact_gap: i16,
    pub seed: [i32; 3],
}

impl Chuchu {
    pub fn spawn(parameters: u32) -> Self {
        let mut behaviour = ((parameters >> BEHAVIOUR_SHIFT) & BYTE_MASK) as u8;
        let mut colour = ((parameters >> COLOUR_SHIFT) & BYTE_MASK) as u8;
        let mut range_byte = ((parameters >> RANGE_SHIFT) & BYTE_MASK) as u8;
        let mut spawn_switch = ((parameters >> SWITCH_SHIFT) & BYTE_MASK) as u8;
        if REG8_S9 != 0 {
            colour = (REG8_S9 - 1) as u8;
        }
        if REG8_S8 != 0 {
            behaviour = BEHAVIOUR_LEASH;
        }
        if behaviour == NO_BYTE {
            behaviour = 0;
        }
        if colour == NO_BYTE {
            colour = 0;
        }
        if range_byte == NO_BYTE {
            range_byte = 0;
        }
        let mut critical = CRITICAL_NONE;
        let notice_range = if behaviour != BEHAVIOUR_POT {
            let mut reach = if range_byte == 0 {
                if behaviour == BEHAVIOUR_RISE {
                    RANGE_RISE
                } else {
                    RANGE_DEFAULT
                }
            } else {
                (range_byte) as f32 * RANGE_SCALE
            };
            if colour >= COLOUR_BAND {
                colour -= COLOUR_BAND;
                reach = RANGE_FREE;
                if colour == COLOUR_TREASURE {
                    critical = CRITICAL_SWALLOW;
                    colour = COLOUR_RED;
                }
            }
            reach
        } else {
            (range_byte) as f32 * RANGE_SCALE
        };
        let mut seed = [RND_SEED; 3];
        let wander = (rnd_fx(&mut seed, WANDER_BAND) as i32) as i16;
        let health = *HEALTH.get(usize::from(colour)).unwrap_or(&HEALTH[0]);
        let steal_left = if colour == COLOUR_BLACK {
            STEAL_BLACK
        } else {
            STEAL_DEFAULT
        };
        let mut state = Step::Normal;
        let mut step = NORMAL_STEP_START;
        let mut gravity = 0.0;
        let mut shadow = 1;
        let mut touch = false;
        let mut contact = false;
        let mut steal_bit = NO_BYTE;
        if behaviour == BEHAVIOUR_CLIMB {
            touch = true;
            contact = true;
            spawn_switch = NO_BYTE;
            state = Step::Climb;
            step = CLIMB_STEP_BEGIN;
        } else if behaviour == BEHAVIOUR_LEASH {
            steal_bit = spawn_switch;
        } else if spawn_switch != NO_BYTE {
            if behaviour != BEHAVIOUR_RISE {
                gravity = GRAVITY_GROUND;
                shadow = 0;
            }
            state = Step::Held;
            step = HELD_STEP_WAIT;
        }
        if behaviour == BEHAVIOUR_POT {
            state = Step::Pot;
            step = POT_STEP_SEARCH;
        }
        Self {
            behaviour,
            colour,
            range_byte,
            spawn_switch,
            state,
            step,
            reaction: REACT_NONE,
            spark: 0,
            floaty: 0,
            shadow,
            abyss: 0,
            climb_step: 0,
            come_demo: 0,
            ret_demo: 0,
            stone: 0,
            stone_frame: 0,
            splashed: false,
            particle_model: 0,
            shock_lock: 0,
            critical,
            strike_frames: 0,
            pot: None,
            wobble: Vec3::ZERO,
            slope: Vec3::ZERO,
            shape: [0; 3],
            anim_res: BCK_START,
            morph: Frame::idle(),
            beta: Frame::idle(),
            split: Frame::idle(),
            light: Light::idle(),
            water_step: 0.0,
            hover: 0.0,
            knock: 0.0,
            knock_rate: 0.0,
            scale: 1.0,
            carry_y: 0.0,
            notice_range,
            leash: LEASH_MIN,
            walk_frame: 0.0,
            tilt: 0,
            tilt_base: 0,
            wander,
            timers: [0; TIMER_COUNT],
            shakes: [0; SHAKE_COUNT],
            home: Vec3::ZERO,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            gravity,
            touch,
            contact,
            shield: false,
            at_set: false,
            at_type: TG_ALL,
            lock: false,
            body_hit: false,
            entered: false,
            doomed: false,
            dropped: false,
            shattered: false,
            health,
            max_health: health,
            steal_left,
            steal_bit,
            freeze: 0,
            burn: 0,
            contact_gap: 0,
            seed,
        }
    }

    pub fn place(&mut self, actor: &mut Actor) {
        self.entered = true;
        self.home = actor.pos;
        self.shape = actor.angle;
        self.leash = (actor.angle[2]) as f32 * LEASH_SCALE;
        if self.leash < LEASH_MIN {
            self.leash = LEASH_MIN;
        }
        actor.angle[2] = 0;
        self.shape[2] = 0;
        if self.behaviour == BEHAVIOUR_RISE {
            self.shape[0] = -0x8000;
        }
        self.anim_res = if self.behaviour == BEHAVIOUR_RISE {
            BCK_TSTART01
        } else if self.behaviour == BEHAVIOUR_CLIMB {
            BCK_HUSE2TACHI
        } else {
            BCK_START
        };
        self.morph.set_start(0);
        self.morph.init(frame_max(self.anim_res));
        self.morph.attribute = EMODE_NONE;
        self.morph.rate = 0.0;
        self.morph.frame = 0.0;
        self.morph.set_loop(self.morph.frame);
        self.morph.set_morf(0.0);
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.scale > 0.0
    }

    pub fn hull(&self) -> (f32, f32) {
        (self.body_height(), BODY_RADIUS)
    }

    pub fn body_height(&self) -> f32 {
        if self.anim_res == BCK_HUSE_WALK {
            BODY_HEIGHT_LIE
        } else {
            BODY_HEIGHT
        }
    }

    pub fn attack_power(&self) -> u8 {
        *AT_POWER.get(usize::from(self.colour)).unwrap_or(&AT_POWER[0])
    }

    pub fn anm_init(&mut self, bck: i32, morf: f32, loop_mode: u8, speed: f32) {
        self.anim_res = bck;
        self.morph.set_start(0);
        self.morph.init(frame_max(bck));
        self.morph.attribute = loop_mode;
        self.morph.rate = speed;
        self.morph.frame = 0.0;
        self.morph.set_loop(self.morph.frame);
        self.morph.set_morf(morf);
    }

    fn part_init(frame: &mut Frame, bck: i32, speed: f32) {
        frame.set_start(0);
        frame.init(frame_max(bck));
        frame.attribute = EMODE_NONE;
        frame.rate = speed;
        frame.frame = 0.0;
        frame.set_loop(frame.frame);
        frame.set_morf(0.0);
    }

    pub fn tick_timers(&mut self) {
        for slot in self.timers.iter_mut() {
            if *slot != 0 {
                *slot -= 1;
            }
        }
        if self.contact_gap > 0 {
            self.contact_gap -= 1;
        }
        if self.burn > 0 {
            self.burn -= 1;
        }
    }

    pub fn clear_shakes(&mut self) {
        for slot in self.shakes.iter_mut() {
            *slot = 0;
        }
    }

    pub fn denki_start(&mut self) {
        if (self.colour != COLOUR_BLUE && self.colour != COLOUR_YELLOW) || self.spark != 0 {
            return;
        }
        self.at_set = true;
        self.spark = 1;
    }

    pub fn denki_end(&mut self) {
        if self.colour == COLOUR_BLUE || self.colour == COLOUR_YELLOW {
            self.spark = 0;
            self.at_set = false;
        }
    }

    pub fn damage_mode_move(&mut self) {
        self.reaction = REACT_PIYO;
        if self.step != DAMAGE_STEP_SPLIT && self.step != DAMAGE_STEP_SPLIT_DONE {
            self.state = Step::Damage;
            self.step = DAMAGE_STEP_BEGIN;
        }
    }

    pub fn search_angle_set(&mut self, actor: &mut Actor, player: &Player) -> bool {
        if self.behaviour != BEHAVIOUR_LEASH {
            if search_distance(actor.pos, player.pos) > TURN_NEAR {
                actor.angle[1] = search_angle(actor.pos, player.pos);
                actor.angle[1] = actor.angle[1].wrapping_add(self.wander);
                return true;
            }
            return false;
        }
        let dx = self.home.x - actor.pos.x;
        let dz = self.home.z - actor.pos.z;
        let flat = (dx * dx + dz * dz).sqrt();
        if flat > self.leash {
            actor.angle[1] = stream::angle_s(dx.atan2(dz));
        } else if flat < LEASH_NEAR
            && rnd(&mut self.seed) < LEASH_GATE
            && distance_angle_s(self.shape[1], actor.angle[1]) < LEASH_TURN_GATE
        {
            let turn = (actor.angle[1]) as f32 + rnd_fx(&mut self.seed, LEASH_JITTER);
            actor.angle[1] = (turn as i32) as i16;
        }
        false
    }

    pub fn black_light_check(&mut self, lit: bool) {
        if self.colour == COLOUR_BLACK && self.state != Step::Stoned && lit {
            self.state = Step::Stoned;
            self.step = STONE_STEP_BEGIN;
        }
    }

    pub fn shock_damage_check(&mut self, bombs: &[Vec3], actor: &Actor, player: &Player) -> bool {
        let mut seen = false;
        for origin in bombs {
            seen = true;
            let dx = origin.x - actor.pos.x;
            let dz = origin.z - actor.pos.z;
            if (dx * dx + dz * dz).sqrt() < SHOCK_REACH {
                if self.shock_lock == 0 {
                    self.damage_mode_move();
                    self.shock_lock = 1;
                    return true;
                }
            } else {
                self.shock_lock = 0;
            }
        }
        if !seen {
            self.shock_lock = 0;
        }
        let dx = player.pos.x - actor.pos.x;
        let dz = player.pos.z - actor.pos.z;
        if (dx * dx + dz * dz).sqrt() < SHOCK_REACH {
            self.damage_mode_move();
            return true;
        }
        false
    }

    fn body_atari_check(&mut self, blow: Blow, events: &mut SimEvents) -> bool {
        if blow.at == At::Idle {
            return false;
        }
        if self.critical != CRITICAL_NONE {
            self.critical = CRITICAL_DONE;
            self.health = 0;
            events.list.push(SimEvent::Se(SE_LK_MS_WEP_HIT));
            self.state = Step::Damage;
            self.step = DAMAGE_STEP_BEGIN;
            return true;
        }
        self.reaction = REACT_NONE;
        let mut guarded = false;
        if blow.at == At::FanLight {
            if self.colour == COLOUR_BLACK {
                if self.state != Step::Stoned {
                    self.state = Step::Stoned;
                    self.step = STONE_STEP_BEGIN;
                    return true;
                }
            } else {
                return false;
            }
        }
        match blow.at {
            At::Hook => {
                if self.step == SWIM_STEP_DRIFT {
                    events.list.push(SimEvent::Se(SE_LK_MS_WEP_HIT));
                    self.gravity = -(REG12_F19 + DEMO_SELECT_10_3);
                    self.knock = 0.0;
                    self.floaty = 0;
                    return false;
                }
                self.reaction = REACT_HOOKSHOT;
                guarded = true;
            }
            At::Sword => {
                if self.colour == COLOUR_BLACK {
                    events.list.push(SimEvent::Se(SE_LK_SW_HIT_S));
                    guarded = true;
                } else {
                    self.reaction = REACT_SWORD;
                }
            }
            At::Stick => {
                if self.colour == COLOUR_BLACK {
                    events.list.push(SimEvent::Se(SE_LK_W_WEP_HIT));
                    guarded = true;
                }
            }
            At::Boomerang => {
                self.reaction = REACT_BOOMERANG;
                if self.step == SWIM_STEP_DRIFT {
                    events.list.push(SimEvent::Se(SE_LK_W_WEP_HIT));
                    self.gravity = -(REG12_F19 + DEMO_SELECT_10_3);
                    self.knock = 0.0;
                    self.floaty = 0;
                    return false;
                }
                guarded = true;
            }
            At::Hammer => {
                if self.colour == COLOUR_BLACK {
                    events.list.push(SimEvent::Se(SE_LK_HAMMER_HIT));
                }
                self.reaction = REACT_SWORD;
                if blow.turn {
                    self.reaction = REACT_HAMMER;
                }
                if self.colour == COLOUR_BLACK {
                    guarded = true;
                }
            }
            At::FireArrow => {
                if self.colour == COLOUR_BLACK {
                    guarded = true;
                    events.list.push(SimEvent::Se(SE_LK_MS_WEP_HIT));
                } else {
                    self.burn = BURN_FRAMES;
                    self.reaction = REACT_FIRE;
                }
            }
            At::Light => {
                self.light.timer = LIGHT_START;
                self.light.particle_scale = LIGHT_SCALE;
                self.light.y_offset = LIGHT_LIFT;
                events.list.push(SimEvent::Se(SE_CM_CC_DIE_SWING));
                events.list.push(SimEvent::Se(SE_CV_CC_DIE));
                self.denki_end();
                self.reaction = REACT_FIRE;
            }
            At::IceArrow => {
                if self.colour == COLOUR_BLACK {
                    guarded = true;
                    events.list.push(SimEvent::Se(SE_LK_MS_WEP_HIT));
                } else {
                    self.freeze = ICE_FREEZE + REG8_S2;
                    if REG8_S2 != 0 {
                        self.health = 100;
                    }
                    events.list.push(SimEvent::Se(SE_CM_FREEZE));
                    events.list.push(SimEvent::Se(SE_CM_CC_DIE_SWING));
                    events.list.push(SimEvent::Se(SE_CV_CC_DIE));
                    self.denki_end();
                    guarded = true;
                    self.reaction = REACT_FIRE;
                }
            }
            At::Arrow | At::Bomb => {
                if self.colour == COLOUR_BLACK {
                    guarded = true;
                    events.list.push(SimEvent::Se(SE_LK_MS_WEP_HIT));
                }
            }
            At::Idle | At::FanLight => {}
        }
        if !guarded {
            let power = blow.at.power(blow.blade, blow.soup);
            if power > 0 {
                self.health -= power;
            }
        }
        if self.reaction == REACT_HOOKSHOT && self.state != Step::Damage {
            self.at_set = false;
            self.knock = 0.0;
            self.knock_rate = 0.0;
            self.tilt = 0;
            self.wobble = Vec3::ZERO;
            self.state = Step::Normal;
            self.step = NORMAL_STEP_STAND;
            return true;
        }
        if self.colour == COLOUR_BLACK
            && self.reaction != REACT_BOOMERANG
            && self.reaction != REACT_WIND
            && self.reaction != REACT_GRAPPLE
        {
            self.reaction = REACT_NONE;
        }
        if self.reaction == REACT_HAMMER {
            self.state = Step::Dead;
            self.step = DEAD_STEP_BETA;
        } else {
            self.state = Step::Damage;
            self.step = DAMAGE_STEP_BEGIN;
        }
        true
    }

    pub fn enemy_ice(&mut self, events: &mut SimEvents) -> bool {
        if self.light.timer == 0 {
            return false;
        }
        if self.light.timer == LIGHT_START {
            self.light.timer += 1;
            events.list.push(SimEvent::Se(SE_L_ARROW_SHRINK));
        } else if self.light.timer < LIGHT_SHRINK_END {
            self.light.timer += 1;
            add_calc0(
                &mut self.light.scale_xz,
                LIGHT_SHRINK_SCALE,
                LIGHT_SHRINK_MAX,
            );
            add_calc0(
                &mut self.light.scale_y,
                LIGHT_SHRINK_SCALE,
                LIGHT_SHRINK_MAX,
            );
        } else {
            self.light.timer += 1;
            if self.light.timer == LIGHT_SHRINK_END {
                events.list.push(SimEvent::Se(SE_L_ARROW_PASS));
            }
            add_calc2(
                &mut self.light.scale_y,
                LIGHT_RISE_TARGET,
                LIGHT_RISE_SCALE,
                LIGHT_RISE_MAX,
            );
            add_calc0(&mut self.light.scale_xz, LIGHT_FADE_SCALE, LIGHT_FADE_MAX);
            if self.light.timer > LIGHT_GONE {
                self.health = LIGHT_DEAD;
            }
        }
        true
    }

    fn resolve(&mut self, actor: &Actor, player_pos: Vec3, at: At, blast: bool) {
        let at = if blast && !matches!(at, At::Hook | At::FanLight | At::Light) {
            At::Bomb
        } else {
            at
        };
        self.body_hit = self.touch
            && at != At::Idle
            && at != At::FanLight
            && cyl_touches(actor.pos, BODY_RADIUS, self.body_height(), player_pos, at.reach());
    }

    pub fn touch_player(&mut self, actor: &Actor, player: &Player, health: &mut Health) {
        if !self.at_set || self.contact_gap != 0 {
            return;
        }
        if cyl_touches(actor.pos, BODY_RADIUS, self.body_height(), player.pos, PLAYER_REACH) {
            health.damage(self.attack_power());
            self.contact_gap = CONTACT_GAP;
        }
    }

    fn stall(&mut self) {
        self.speed_f = 0.0;
        self.speed = Vec3::ZERO;
        self.gravity = 0.0;
    }

    pub fn naraku_check(
        &mut self,
        actor: &mut Actor,
        solid: Option<&Collision>,
        sea: Option<&Sea>,
        events: &mut SimEvents,
    ) -> bool {
        if ground::surface(solid, actor.pos).is_none() {
            self.abyss = self.abyss.wrapping_add(1);
            if actor.pos.y < ABYSS_Y || self.abyss > ABYSS_LIMIT {
                self.stall();
                return true;
            }
        }
        if actor.pos.y < self.home.y + ABYSS_DEPTH {
            self.stall();
            return true;
        }
        let (in_water, surface) = water::probe(sea, actor.pos.x, actor.pos.z);
        if in_water {
            if self.state != Step::Swim {
                if !self.splashed {
                    events.list.push(SimEvent::Se(SE_OBJ_FALL_WATER_M));
                    self.splashed = true;
                }
                let sink = if self.stone != 0 {
                    WATER_SINK_STONE + REG12_F0
                } else {
                    WATER_SINK + REG12_F0
                };
                if actor.pos.y < surface - sink {
                    if self.stone != 0 {
                        self.stall();
                        return true;
                    }
                    self.state = Step::Swim;
                    self.step = SWIM_STEP_LIE;
                    if self.health <= 0 {
                        self.state = Step::Dead;
                        self.step = DEAD_STEP_DROP;
                    }
                }
            }
        } else if self.splashed {
            self.floaty = 0;
            self.splashed = false;
        }
        false
    }

    fn action_normal_move(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        solid: Option<&Collision>,
        blow: Blow,
        bombs: &[Vec3],
        lit: bool,
        events: &mut SimEvents,
    ) {
        let landed = ground::grounded(solid, actor.pos);
        let dist = search_distance(actor.pos, player.pos);
        let mut run = true;
        while run {
            run = false;
            match self.step {
                NORMAL_STEP_START => {
                    self.touch = false;
                    self.contact = false;
                    self.anm_init(BCK_START, 0.0, EMODE_NONE, 0.0);
                    if self.behaviour == BEHAVIOUR_WALK {
                        self.anm_init(BCK_TSTART01, 0.0, EMODE_NONE, 0.0);
                    }
                    self.step = NORMAL_STEP_ALERT;
                    run = true;
                }
                NORMAL_STEP_ALERT => {
                    if dist < self.notice_range {
                        if self.behaviour == BEHAVIOUR_RISE {
                            self.anm_init(BCK_TSTART01, 0.0, EMODE_NONE, 1.0);
                            self.step = NORMAL_STEP_RISE;
                        } else {
                            events.list.push(SimEvent::Se(SE_CM_CC_ENTER_GND));
                            self.anm_init(BCK_START, 0.0, EMODE_NONE, 1.0);
                            self.shadow = 0;
                            self.gravity = GRAVITY_GROUND;
                            self.step = NORMAL_STEP_RISE_WAIT;
                            self.touch = true;
                        }
                    }
                }
                NORMAL_STEP_RISE => {
                    if self.morph.check_pass(RISE_FRAMES) {
                        self.shadow = 0;
                        self.gravity = GRAVITY_RISE;
                    }
                    if self.morph.frame > RISE_FRAMES {
                        stream::add_calc_angle_s2(&mut self.shape[0], 0, 1, SPIN_STEP);
                    }
                    if landed {
                        events.list.push(SimEvent::Se(SE_CM_CC_LANDING));
                        self.shape[0] = 0;
                        actor.angle[0] = 0;
                        self.speed.y = 0.0;
                        self.anm_init(BCK_TSTART02, 0.0, EMODE_NONE, 1.0);
                        self.step = NORMAL_STEP_RISE_WAIT;
                        self.touch = true;
                        self.denki_start();
                    }
                }
                NORMAL_STEP_RISE_WAIT => {
                    if self.morph.is_stop() {
                        self.step = NORMAL_STEP_SETTLE;
                        run = true;
                    }
                }
                NORMAL_STEP_SETTLE => {
                    self.clear_shakes();
                    self.at_type = TG_ALL;
                    self.contact = true;
                    self.touch = true;
                    self.timers[TIMER_HOLD] = WALK_HOLD;
                    self.timers[TIMER_HOLD] = self.timers[TIMER_HOLD]
                        .wrapping_add((rnd_f(&mut self.seed, WALK_SPAN) as i32) as i16);
                    self.timers[TIMER_HOLD] =
                        self.timers[TIMER_HOLD].wrapping_add(self.timers[TIMER_REST]);
                    self.denki_start();
                    if self.anim_res != BCK_TACHI_WALK {
                        self.anm_init(BCK_TACHI_WALK, 2.0, EMODE_LOOP, 1.0);
                    }
                    events.list.push(SimEvent::Se(SE_CM_CC_MOVE_STAND));
                    self.step = NORMAL_STEP_WALK;
                }
                NORMAL_STEP_WALK => {
                    self.search_angle_set(actor, player);
                    if self.morph.check_pass(0.0) {
                        events.list.push(SimEvent::Se(SE_CM_CC_MOVE_STAND));
                    }
                    add_calc2(&mut self.speed_f, MOVE_TARGET, MOVE_SCALE, MOVE_STEP);
                    if self.timers[TIMER_HOLD] != 0 {
                        break;
                    }
                    if self.behaviour == BEHAVIOUR_CLIMB && self.timers[TIMER_DEMO] == 0 {
                        self.step = NORMAL_STEP_LIE;
                        break;
                    } else if self.colour == COLOUR_RED || self.colour == COLOUR_YELLOW {
                        if dist < ATTACK_RANGE {
                            self.state = Step::Attack;
                            self.step = ATTACK_STEP_BEGIN;
                            return;
                        }
                    } else {
                        self.step = NORMAL_STEP_LIE;
                        run = true;
                    }
                }
                NORMAL_STEP_LIE => {
                    self.denki_start();
                    events.list.push(SimEvent::Se(SE_CM_CC_STAND_TO_LIE));
                    self.at_type = TG_LIE;
                    self.contact = false;
                    self.anm_init(BCK_TACHI2HUSE, 2.0, EMODE_NONE, 1.0);
                    self.step = NORMAL_STEP_LIE_WAIT;
                }
                NORMAL_STEP_LIE_WAIT => {
                    if self.morph.is_stop() {
                        if self.behaviour == BEHAVIOUR_CLIMB && self.timers[TIMER_DEMO] == 0 {
                            self.state = Step::Climb;
                            self.step = CLIMB_STEP_HUG;
                            break;
                        }
                        self.anm_init(BCK_HUSE_WALK, 0.0, EMODE_LOOP, 1.0);
                        self.step = NORMAL_STEP_SLIDE;
                        run = true;
                    }
                }
                NORMAL_STEP_SLIDE => {
                    self.timers[TIMER_HOLD] = SLIDE_BASE;
                    self.timers[TIMER_HOLD] = self.timers[TIMER_HOLD]
                        .wrapping_add((rnd_f(&mut self.seed, SLIDE_SPAN) as i32) as i16);
                    events.list.push(SimEvent::Se(SE_CM_CC_MOVE_LIE));
                    self.speed_f = SLIDE_SPEED;
                    self.step = NORMAL_STEP_SLIDE_WAIT;
                }
                NORMAL_STEP_SLIDE_WAIT => {
                    if self.morph.check_pass(0.0) {
                        events.list.push(SimEvent::Se(SE_CM_CC_MOVE_LIE));
                    }
                    self.search_angle_set(actor, player);
                    if self.timers[TIMER_HOLD] == 0 {
                        self.step = NORMAL_STEP_STAND;
                        run = true;
                    }
                }
                NORMAL_STEP_STAND => {
                    self.at_type = TG_ALL;
                    self.anm_init(BCK_HUSE2TACHI, 0.0, EMODE_NONE, 1.0);
                    events.list.push(SimEvent::Se(SE_CM_CC_LIE_TO_STAND));
                    self.step = NORMAL_STEP_UP;
                }
                NORMAL_STEP_UP => {
                    if self.morph.is_stop() {
                        events.list.push(SimEvent::Se(SE_CV_CC_STAND_UP));
                        if self.behaviour != BEHAVIOUR_LEASH && dist < ATTACK_RANGE && landed {
                            self.state = Step::Attack;
                            self.step = ATTACK_STEP_BEGIN;
                            return;
                        }
                        self.step = NORMAL_STEP_SETTLE;
                    }
                }
                _ => {}
            }
        }
        stream::add_calc_angle_s2(&mut self.shape[1], actor.angle[1], 1, TURN_STEP);
        if (self.step < NORMAL_STEP_SETTLE
            || self.step > NORMAL_STEP_UP
            || !self.shock_damage_check(bombs, actor, player))
            && self.step >= NORMAL_STEP_RISE_WAIT
        {
            self.black_light_check(lit);
        }
    }

    pub fn action_swim(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        solid: Option<&Collision>,
        sea: Option<&Sea>,
        events: &mut SimEvents,
    ) {
        let (in_water, surface) = water::probe(sea, actor.pos.x, actor.pos.z);
        let mut run = true;
        while run {
            run = false;
            match self.step {
                SWIM_STEP_LIE => {
                    self.clear_shakes();
                    self.denki_start();
                    self.particle_model = 0;
                    events.list.push(SimEvent::Se(SE_CM_CC_STAND_TO_LIE));
                    self.wobble = Vec3::ZERO;
                    self.at_type = TG_FLOAT;
                    self.contact = false;
                    if self.colour != COLOUR_BLUE && self.colour != COLOUR_YELLOW {
                        self.at_set = false;
                        self.at_set = false;
                    }
                    events.list.push(SimEvent::Se(SE_CM_CC_STAND_TO_LIE));
                    if self.anim_res != BCK_TACHI2HUSE {
                        self.anm_init(BCK_TACHI2HUSE, 2.0, EMODE_NONE, 1.0);
                    }
                    self.speed_f = 0.0;
                    self.speed = Vec3::ZERO;
                    self.gravity = GRAVITY_GROUND;
                    self.knock = 0.0;
                    self.floaty = 0;
                    self.step = SWIM_STEP_DRIFT;
                    run = true;
                }
                SWIM_STEP_DRIFT => {
                    if self.anim_res == BCK_TACHI2HUSE && self.morph.is_stop() {
                        self.anm_init(BCK_HUSE_WALK, 0.0, EMODE_LOOP, 1.0);
                    }
                    add_calc0(&mut self.gravity, FLOAT_SCALE, FLOAT_MAX_STEP);
                    add_calc0(&mut self.speed.y, FLOAT_SCALE, FLOAT_MAX_STEP);
                    self.shakes[0] = self.shakes[0].wrapping_add(SWAY_STEP);
                    let sway = stream::sin_s(self.shakes[0]) * SWAY_SCALE + SWAY_BASE;
                    add_calc2(&mut self.hover, sway, FLOAT_SCALE, FLOAT_MAX_STEP);
                    if in_water {
                        add_calc2(
                            &mut actor.pos.y,
                            surface + FLOAT_DEPTH,
                            1.0,
                            self.water_step,
                        );
                        add_calc2(&mut self.water_step, FLOAT_RATE, 1.0, 1.0);
                        if (actor.pos.y - (surface + FLOAT_DEPTH)).abs() < FLOAT_EPSILON {
                            self.floaty = 1;
                            add_calc2(&mut self.speed_f, FLOAT_DRIVE, FLOAT_GAIN, 1.0);
                            if REG8_F17 != 0.0 || self.speed_f < FLOAT_FALL {
                                self.search_angle_set(actor, player);
                            }
                        }
                    }
                    if self.morph.check_pass(0.0) {
                        events.list.push(SimEvent::Se(SE_CM_CC_MOVE_LIE));
                    }
                }
                _ => {}
            }
        }
        if !in_water {
            self.gravity = GRAVITY_GROUND;
            if ground::grounded(solid, actor.pos)
                || (self.behaviour == BEHAVIOUR_CLIMB && self.timers[TIMER_DEMO] == 0)
            {
                self.floaty = 0;
                self.hover = 0.0;
                if self.behaviour != BEHAVIOUR_CLIMB {
                    self.state = Step::Normal;
                    self.step = NORMAL_STEP_SLIDE;
                } else {
                    self.state = Step::Climb;
                    self.step = CLIMB_STEP_HUG;
                }
            }
        }
    }

    fn action_attack_move(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        solid: Option<&Collision>,
        blow: Blow,
        bombs: &[Vec3],
        lit: bool,
        events: &mut SimEvents,
    ) {
        let mut run = true;
        while run {
            run = false;
            match self.step {
                ATTACK_STEP_BEGIN => {
                    self.clear_shakes();
                    self.speed_f = 0.0;
                    self.denki_start();
                    self.lock = true;
                    self.contact = true;
                    events.list.push(SimEvent::Se(SE_CV_CC_ATTACK));
                    self.strike_frames = 0;
                    self.anm_init(BCK_ATTACK01, 0.0, EMODE_NONE, 1.0);
                    self.step = ATTACK_STEP_WINDUP;
                    run = true;
                }
                ATTACK_STEP_WINDUP => {
                    if self.morph.is_stop() {
                        self.timers[TIMER_HOLD] = ATTACK_WIND;
                        self.step = ATTACK_STEP_JUMP;
                        run = true;
                    }
                }
                ATTACK_STEP_JUMP => {
                    if self.timers[TIMER_HOLD] == 0 {
                        actor.angle[1] = search_angle(actor.pos, player.pos);
                        self.at_set = true;
                        self.anm_init(BCK_ATTACK02, 0.0, EMODE_NONE, 1.0);
                        events.list.push(SimEvent::Se(SE_CM_CC_ATTACK));
                        self.speed_f = ATTACK_SPEED;
                        self.gravity = GRAVITY_GROUND;
                        self.speed.y = ATTACK_LIFT;
                        self.step = ATTACK_STEP_AIR;
                    }
                }
                ATTACK_STEP_AIR => {
                    if self.speed_f > 0.0 && self.shield {
                        self.speed_f *= ATTACK_SHIELD;
                        self.speed.y = 0.0;
                    }
                    if ground::grounded(solid, actor.pos) {
                        if self.colour != COLOUR_BLUE && self.colour != COLOUR_YELLOW {
                            self.at_set = false;
                            self.at_set = false;
                        }
                        events.list.push(SimEvent::Se(SE_CM_CC_LANDING));
                        self.speed_f = 0.0;
                        self.anm_init(BCK_ATTACK03, 0.0, EMODE_NONE, 1.0);
                        self.step = ATTACK_STEP_LAND;
                    }
                }
                ATTACK_STEP_LAND => {
                    if self.morph.is_stop() {
                        self.timers[TIMER_REST] = LIE_SPAN;
                        let span = (self.timers[TIMER_REST]) as f32;
                        self.timers[TIMER_REST] = self.timers[TIMER_REST]
                            .wrapping_add((rnd_f(&mut self.seed, span) as i32) as i16);
                        self.state = Step::Normal;
                        self.step = NORMAL_STEP_SETTLE;
                    }
                }
                _ => {}
            }
        }
        if (ATTACK_STEP_WINDUP..=ATTACK_STEP_LAND).contains(&self.step) {
            self.strike_frames = self.strike_frames.wrapping_add(1);
        }
        if self.step > ATTACK_STEP_JUMP || !self.shock_damage_check(bombs, actor, player) {
            stream::add_calc_angle_s2(&mut self.shape[1], actor.angle[1], 1, ATTACK_TURN);
            self.black_light_check(lit);
        }
    }

    pub fn action_damage_move(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        bombs: &[Vec3],
        lit: bool,
        events: &mut SimEvents,
    ) {
        let mut run = true;
        while run {
            run = false;
            match self.step {
                DAMAGE_STEP_BEGIN => {
                    self.clear_shakes();
                    self.at_set = false;
                    self.touch = true;
                    self.contact = true;
                    self.lock = true;
                    if self.colour == COLOUR_BLACK && self.reaction == REACT_NONE {
                        self.touch = false;
                        self.knock = 0.0;
                        self.knock_rate = 0.0;
                        self.tilt = 0;
                        self.wobble = Vec3::ZERO;
                        self.speed_f = 0.0;
                        self.step = DAMAGE_STEP_SPLIT;
                        break;
                    }
                    self.denki_end();
                    self.anm_init(BCK_TACHI2HUSE, 0.0, EMODE_NONE, 0.0);
                    self.at_type = TG_ALL;
                    self.tilt_base = search_angle(actor.pos, player.pos)
                        .wrapping_sub(BACK_TURN)
                        .wrapping_sub(self.shape[1]);
                    self.tilt = 0;
                    self.knock = KNOCK_DEFAULT;
                    if self.reaction == REACT_SWORD || self.health <= 0 {
                        self.knock = KNOCK_SWORD;
                        self.reaction = REACT_SWORD;
                    }
                    if self.health > 0 {
                        events.list.push(SimEvent::Se(SE_CM_CC_DMG_SWING));
                        events.list.push(SimEvent::Se(SE_CV_CC_DAMAGE));
                    } else {
                        events.list.push(SimEvent::Se(SE_CM_CC_DIE_SWING));
                        events.list.push(SimEvent::Se(SE_CV_CC_DIE));
                    }
                    self.speed.y = 0.0;
                    self.knock_rate = KNOCK_RATE_BASE;
                    match self.reaction {
                        REACT_SWORD => {
                            self.knock_rate = KNOCK_RATE_SWORD;
                            self.speed_f = KNOCK_SPEED_SWORD;
                        }
                        REACT_BOOMERANG | REACT_GRAPPLE => {
                            self.anm_init(BCK_MAHI, 0.0, EMODE_NONE, 1.0);
                            self.knock = 0.0;
                            self.knock_rate = 0.0;
                            self.tilt = 0;
                        }
                        REACT_PIYO => {
                            self.knock = KNOCK_SWORD;
                            self.tilt = TILT_WIND;
                        }
                        _ => {
                            self.speed_f = KNOCK_SPEED_BASE;
                        }
                    }
                    if self.behaviour == BEHAVIOUR_LEASH {
                        self.speed_f = 0.0;
                    }
                    if self.reaction != REACT_WIND {
                        actor.angle[1] = search_angle(actor.pos, player.pos).wrapping_sub(BACK_TURN);
                    }
                    self.step = DAMAGE_STEP_KNOCK;
                    run = true;
                }
                DAMAGE_STEP_KNOCK => {
                    let swing = self.knock * stream::sin_s(self.tilt);
                    self.wobble.z = -(swing * stream::sin_s(self.tilt_base));
                    self.wobble.x = swing * stream::cos_s(self.tilt_base);
                    add_calc0(&mut self.speed_f, KNOCK_SCALE, KNOCK_RATE_STEP);
                    if self.reaction != REACT_WIND {
                        self.tilt = self.tilt.wrapping_add(TILT_STEP);
                    }
                    add_calc0(&mut self.knock, 1.0, self.knock_rate);
                    if self.knock.abs() < KNOCK_MIN {
                        self.speed_f = 0.0;
                        self.wobble = Vec3::ZERO;
                        if self.reaction == REACT_BOOMERANG || self.reaction == REACT_PIYO {
                            self.timers[TIMER_SPLIT] = PIYO_BASE;
                            self.timers[TIMER_SPLIT] =
                                self.timers[TIMER_SPLIT].wrapping_add(PIYO_SPAN);
                            self.timers[TIMER_SPLIT] =
                                self.timers[TIMER_SPLIT].wrapping_add(REG8_S5);
                            events.list.push(SimEvent::Se(SE_CM_MD_PIYO));
                            self.step = DAMAGE_STEP_PIYO;
                        } else if self.reaction == REACT_GRAPPLE {
                            self.timers[TIMER_SPLIT] = GRAPPLE_BASE + REG8_S4;
                            self.step = DAMAGE_STEP_GRAPPLE;
                        } else {
                            self.step = DAMAGE_STEP_WAIT;
                        }
                    }
                }
                DAMAGE_STEP_WAIT => {
                    if self.health > 0 {
                        if self.timers[TIMER_SPLIT] == 0 {
                            self.state = Step::Normal;
                            self.step = NORMAL_STEP_LIE;
                            if self.colour == COLOUR_RED || self.colour == COLOUR_YELLOW {
                                self.step = NORMAL_STEP_SETTLE;
                            }
                        }
                    } else {
                        self.state = Step::Dead;
                        self.step = DEAD_STEP_DROP;
                    }
                }
                DAMAGE_STEP_PIYO => {
                    if self.timers[TIMER_SPLIT] <= 1 {
                        self.anm_init(BCK_HUKKATSU, 0.0, EMODE_LOOP, 1.0);
                        self.timers[TIMER_SPLIT] = HUKKATSU_BASE;
                        self.timers[TIMER_SPLIT] =
                            self.timers[TIMER_SPLIT].wrapping_add(REG8_S6);
                        self.step = DAMAGE_STEP_SPLIT;
                    }
                }
                DAMAGE_STEP_GRAPPLE => {
                    if self.timers[TIMER_SPLIT] == 0 {
                        self.state = Step::Normal;
                        self.step = NORMAL_STEP_LIE;
                        if self.colour == COLOUR_RED || self.colour == COLOUR_YELLOW {
                            self.step = NORMAL_STEP_SETTLE;
                        }
                    }
                }
                DAMAGE_STEP_SPLIT => {
                    self.particle_model = 1;
                    Self::part_init(&mut self.split, BCK_PTCL, 1.0);
                    self.step = DAMAGE_STEP_SPLIT_DONE;
                }
                DAMAGE_STEP_SPLIT_DONE => {
                    if self.split.check_pass(12.0) {
                        events.list.push(SimEvent::Se(SE_CM_B_CC_RECOVER));
                    }
                    if self.split.is_stop() {
                        self.touch = true;
                        self.particle_model = 0;
                        self.at_type = TG_ALL;
                        self.anm_init(BCK_HUSE2TACHI, 0.0, EMODE_NONE, 1.0);
                        events.list.push(SimEvent::Se(SE_CM_CC_LIE_TO_STAND));
                        self.shape[1] = search_angle(actor.pos, player.pos);
                        actor.angle[1] = self.shape[1];
                        self.state = Step::Normal;
                        self.step = NORMAL_STEP_UP;
                    }
                }
                _ => {}
            }
        }
        if self.health > 0 && !self.body_hit {
            if self.step >= DAMAGE_STEP_WAIT {
                self.shock_damage_check(bombs, actor, player);
            }
            self.black_light_check(lit);
        }
    }

    pub fn action_dead_move(&mut self, events: &mut SimEvents) {
        let mut run = true;
        while run {
            run = false;
            match self.step {
                DEAD_STEP_BEGIN => {
                    self.denki_end();
                    self.at_set = false;
                    self.touch = false;
                    self.contact = false;
                    self.step = DEAD_STEP_DROP;
                    run = true;
                }
                DEAD_STEP_DROP => {
                    self.doomed = true;
                }
                DEAD_STEP_BETA => {
                    self.denki_end();
                    events.list.push(SimEvent::Se(SE_CM_CC_DIE_SWING));
                    events.list.push(SimEvent::Se(SE_CV_CC_DIE));
                    Self::part_init(&mut self.beta, BCK_BETA, 1.0);
                    self.speed_f = 0.0;
                    self.speed = Vec3::ZERO;
                    self.at_set = false;
                    self.touch = false;
                    self.contact = false;
                    self.timers[TIMER_HOLD] = 0;
                    self.step = DEAD_STEP_BETA_DONE;
                    run = true;
                }
                DEAD_STEP_BETA_DONE => {
                    if self.beta.is_stop() {
                        self.doomed = true;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn action_climb_move(
        &mut self,
        actor: &mut Actor,
        solid: Option<&Collision>,
        rolling: bool,
        events: &mut SimEvents,
    ) {
        let mut run = true;
        while run {
            run = false;
            match self.step {
                CLIMB_STEP_BEGIN => {
                    self.climb_step = 0;
                    self.lock = false;
                    self.anm_init(BCK_DEKU_START, 0.0, EMODE_NONE, 0.0);
                    self.step = CLIMB_STEP_WAIT;
                    run = true;
                }
                CLIMB_STEP_WAIT => {
                    if self.climb_step >= 1 {
                        events.list.push(SimEvent::Se(SE_CM_CC_ENTER_GND));
                        self.anm_init(BCK_DEKU_START, 0.0, EMODE_NONE, 1.0);
                        self.step = CLIMB_STEP_READY;
                    }
                }
                CLIMB_STEP_READY => {
                    if self.morph.is_stop() {
                        self.walk_frame = rnd_f(&mut self.seed, CLIMB_SPAN);
                        self.anm_init(BCK_TACHI_WALK, 1.0, EMODE_LOOP, CLIMB_RATE);
                        self.step = CLIMB_STEP_WALK;
                        run = true;
                    }
                }
                CLIMB_STEP_WALK => {
                    if self.morph.check_pass(self.walk_frame) {
                        events.list.push(SimEvent::Se(SE_CV_CC_PARASITE));
                    }
                    if !rolling {
                        break;
                    }
                    self.anm_init(BCK_ATTACK01, 0.0, EMODE_NONE, 1.0);
                    self.step = CLIMB_STEP_START;
                    run = true;
                }
                CLIMB_STEP_START => {
                    self.step = CLIMB_STEP_LEAP;
                }
                CLIMB_STEP_LEAP => {
                    events.list.push(SimEvent::Se(SE_CV_CC_ATTACK));
                    self.anm_init(BCK_ATTACK02, 0.0, EMODE_NONE, 1.0);
                    actor.angle = [0; 3];
                    let turn = rnd_fx(&mut self.seed, CLIMB_JUMP) + 0x3000;
                    actor.angle[1] = (turn as i32) as i16;
                    self.shape = actor.angle;
                    self.climb_step = 3;
                    events.list.push(SimEvent::Se(SE_CM_CC_ATTACK));
                    self.lock = true;
                    self.at_type = TG_ALL;
                    self.touch = true;
                    self.contact = true;
                    self.at_set = false;
                    self.speed_f = CLIMB_LEAP_SPEED;
                    self.speed.y = CLIMB_LEAP_LIFT;
                    self.gravity = GRAVITY_GROUND;
                    self.shadow = 0;
                    self.step = CLIMB_STEP_AIR;
                }
                CLIMB_STEP_AIR => {
                    if ground::grounded(solid, actor.pos) {
                        self.anm_init(BCK_ATTACK03, 0.0, EMODE_NONE, 1.0);
                        events.list.push(SimEvent::Se(SE_CM_CC_LANDING));
                        self.speed.y = 0.0;
                        self.denki_start();
                        self.step = CLIMB_STEP_DONE;
                    }
                }
                CLIMB_STEP_DONE => {
                    self.timers[TIMER_DEMO] = CLIMB_LEAP_DELAY;
                    self.state = Step::Normal;
                    self.step = NORMAL_STEP_RISE_WAIT;
                }
                CLIMB_STEP_HUG => {
                    self.touch = false;
                    self.contact = false;
                    self.at_set = false;
                    self.wobble = Vec3::ZERO;
                    self.shadow = 1;
                    if self.anim_res != BCK_TACHI_WALK {
                        self.anm_init(BCK_TACHI2HUSE, 0.0, EMODE_NONE, 0.0);
                    }
                    events.list.push(SimEvent::Se(SE_CM_CC_STAND_TO_LIE));
                    if self.ret_demo == 0 {
                        self.ret_demo = 1;
                    }
                    self.speed = Vec3::ZERO;
                    self.speed_f = 0.0;
                    self.step = CLIMB_STEP_FADE;
                    run = true;
                }
                CLIMB_STEP_FADE => {
                    add_calc0(&mut self.scale, CLIMB_SCALE_STEP, CLIMB_SCALE_MAX);
                    if self.scale > CLIMB_SCALE_GATE {
                        break;
                    }
                    self.shape = [0; 3];
                    actor.angle = [0; 3];
                    self.gravity = 0.0;
                    self.speed = Vec3::ZERO;
                    self.timers[TIMER_HOLD] = 0;
                    if self.ret_demo != 0 {
                        self.timers[TIMER_HOLD] = HOLD_DELAY;
                    }
                    self.anm_init(BCK_START, 0.0, EMODE_NONE, 0.0);
                    self.climb_step = 4;
                    self.step = CLIMB_STEP_RETURN;
                    run = true;
                }
                CLIMB_STEP_RETURN => {
                    if self.timers[TIMER_HOLD] == 0 {
                        self.doomed = true;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn action_stoned_move(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        solid: Option<&Collision>,
        lit: bool,
        events: &mut SimEvents,
    ) {
        let mut run = true;
        while run {
            run = false;
            match self.step {
                STONE_STEP_BEGIN => {
                    self.reaction = REACT_STONED;
                    self.shield = true;
                    self.touch = true;
                    self.at_type = TG_ALL;
                    self.anm_init(BCK_MAHI, 0.0, EMODE_NONE, 0.0);
                    self.particle_model = 0;
                    events.list.push(SimEvent::Se(SE_CM_CC_STONED));
                    self.speed = Vec3::ZERO;
                    self.contact = true;
                    self.at_set = false;
                    self.clear_shakes();
                    self.speed_f = 0.0;
                    events.list.push(SimEvent::Se(SE_CM_CC_DMG_SWING));
                    events.list.push(SimEvent::Se(SE_CV_CC_DAMAGE));
                    self.step = STONE_STEP_POSE;
                    run = true;
                }
                STONE_STEP_POSE => {
                    self.stone = 1;
                    self.stone_frame = 0;
                    self.particle_model = 0;
                    self.timers[TIMER_SPLIT] = STONE_HOLD;
                    self.step = STONE_STEP_WAIT;
                }
                STONE_STEP_WAIT => {
                    if self.stone != 2 {
                        self.lock = true;
                    }
                    add_calc0(&mut self.speed_f, STONE_BRAKE_SCALE, STONE_BRAKE_MAX);
                    self.step = STONE_STEP_FREE;
                }
                STONE_STEP_FREE => {
                    actor.angle[1] = walk::to_s16(player.face);
                    self.lock = true;
                    self.contact = true;
                    self.gravity = GRAVITY_GROUND;
                    if self.speed_f > 0.0 {
                        self.speed.y = STONE_LIFT;
                        self.speed_f = STONE_SPEED;
                        self.step = STONE_STEP_THROW;
                    } else {
                        events.list.push(SimEvent::Se(SE_OBJ_PUT_STONE));
                        self.speed_f = 0.0;
                        self.step = STONE_STEP_WAIT;
                    }
                }
                STONE_STEP_THROW => {
                    if ground::grounded(solid, actor.pos) {
                        self.speed_f = 0.0;
                        events.list.push(SimEvent::Se(SE_OBJ_PUT_STONE));
                        self.shattered = true;
                    }
                    add_calc0(&mut self.speed_f, STONE_BRAKE_SCALE, STONE_BRAKE_MAX);
                }
                _ => {}
            }
        }
        if self.carry_y + STONE_RISE <= actor.pos.y {
            stream::add_calc_angle_s2(&mut self.shape[1], actor.angle[1], 1, THROW_TURN);
        }
        if self.step >= STONE_STEP_WAIT {
            if lit {
                self.timers[TIMER_SPLIT] = STONE_HOLD;
            }
            if self.timers[TIMER_SPLIT] == 1 {
                self.timers[TIMER_WOBBLE] = STONE_RECOVER;
                events.list.push(SimEvent::Se(SE_CM_CC_STONED_RECOVER));
            }
            if self.timers[TIMER_WOBBLE] != 0 {
                self.shakes[0] = self.shakes[0].wrapping_add(STONE_SPAN);
                self.shape[2] = (STONE_ROLL * stream::sin_s(self.shakes[0]) as i32) as i16;
                if self.timers[TIMER_WOBBLE] == 1 {
                    self.stone = 2;
                    self.stone_frame = 0;
                    self.shape[2] = 0;
                }
            }
            if self.stone == 2 && self.stone_frame >= STONE_FRAME_MAX {
                self.lock = true;
                self.shield = false;
                self.stone_frame = 0;
                self.stone = 0;
                self.shape[2] = 0;
                self.gravity = GRAVITY_GROUND;
                self.touch = true;
                actor.angle[1] = self.shape[1];
                self.state = Step::Normal;
                self.step = NORMAL_STEP_LIE;
            }
        }
        if self.shattered {
            events.list.push(SimEvent::Se(SE_OBJ_BREAK_STONE));
            self.doomed = true;
        }
    }

    pub fn action_held_move(&mut self, switches: &Switches) {
        let mut run = true;
        while run {
            run = false;
            match self.step {
                HELD_STEP_WAIT => {
                    if switches.test(self.spawn_switch) {
                        self.timers[TIMER_LIE] = 0x28;
                        self.step = HELD_STEP_RELEASE;
                    }
                }
                HELD_STEP_RELEASE => {
                    if self.timers[TIMER_LIE] == 0 {
                        self.touch = true;
                        self.state = Step::Normal;
                        self.step = NORMAL_STEP_START;
                    }
                }
                _ => {}
            }
        }
    }

    fn action_pot_search(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        pots: &Query<(Entity, &Actor), (With<tsubo::Tsubo>, Without<Chuchu>)>,
        events: &mut SimEvents,
    ) {
        let mut run = true;
        while run {
            run = false;
            match self.step {
                POT_STEP_SEARCH => {
                    let mut found = None;
                    for (entity, pot) in pots.iter() {
                        if pot.pos.x != actor.pos.x || pot.pos.z != actor.pos.z {
                            continue;
                        }
                        if (pot.pos.y - actor.pos.y).abs() < 10.0 {
                            found = Some(entity);
                        }
                    }
                    let Some(entity) = found else {
                        self.doomed = true;
                        return;
                    };
                    self.pot = Some(entity);
                    self.step = POT_STEP_FOLLOW;
                }
                POT_STEP_FOLLOW => {
                    let Some(entity) = self.pot else {
                        self.doomed = true;
                        return;
                    };
                    let Ok((_, pot)) = pots.get(entity) else {
                        self.doomed = true;
                        return;
                    };
                    actor.pos = pot.pos;
                    if self.notice_range == 0.0 {
                        return;
                    }
                    if search_distance(actor.pos, player.pos) > self.notice_range {
                        return;
                    }
                    self.touch = true;
                    self.shadow = 0;
                    actor.angle[1] = search_angle(actor.pos, player.pos);
                    self.shape[1] = actor.angle[1];
                    self.at_set = true;
                    self.at_type = TG_POT;
                    self.anm_init(BCK_HUSE2TACHI, 0.0, EMODE_NONE, 1.0);
                    events.list.push(SimEvent::Se(SE_CM_CC_LIE_TO_STAND));
                    self.denki_start();
                    self.gravity = GRAVITY_GROUND;
                    self.behaviour = BEHAVIOUR_WALK;
                    self.step = POT_STEP_LIVE;
                }
                POT_STEP_LIVE => {
                    self.at_type = TG_POT;
                    self.lock = true;
                    self.state = Step::Normal;
                    self.step = NORMAL_STEP_UP;
                }
                _ => {}
            }
        }
    }

    pub fn drop_item(
        &mut self,
        actor: &Actor,
        health: &Health,
        arrows: u8,
        commands: &mut Commands,
    ) {
        if self.dropped {
            return;
        }
        self.dropped = true;
        let kind = if self.critical == CRITICAL_NONE {
            DIS_IBALL
        } else if self.critical == CRITICAL_DONE {
            DIS_NONE13
        } else if health.max <= LIFE_LOW {
            DIS_HEART
        } else if rnd_f(&mut self.seed, DROP_GATE) < DROP_HALF {
            if arrows <= ARROW_LOW {
                DIS_ARROW
            } else {
                DIS_HEART
            }
        } else {
            DIS_NONE13
        };
        let item = match kind {
            DIS_HEART => ITEM_HEART,
            DIS_ARROW => ITEM_ARROW_10,
            _ => return,
        };
        let parameters = u32::from(item);
        commands.spawn((
            Actor {
                name: ITEM_ACTORS[0].to_owned(),
                parameters,
                pos: actor.pos + Vec3::new(0.0, ITEM_DROP_LIFT, 0.0),
                angle: actor.angle,
                set_id: 0,
                room: actor.room,
            },
            Pickup::spawn(parameters),
        ));
    }

    pub fn integrate(&mut self, actor: &mut Actor) {
        let (sin, cos) = walk::from_s16(actor.angle[1]).sin_cos();
        let pitch = walk::from_s16(actor.angle[0]).cos();
        self.speed.x = self.speed_f * sin;
        self.speed.z = self.speed_f * cos * pitch;
        self.speed.y += self.gravity;
        if self.state != Step::Climb {
            self.slope = Vec3::ZERO;
            if self.speed.y < MAX_FALL {
                self.speed.y = MAX_FALL;
            }
        } else if self.speed.y < MAX_FALL {
            self.speed.y = MAX_FALL;
        } else if self.speed.y > CLIMB_RISE {
            self.speed.y = CLIMB_RISE;
        }
        if self.step == DEAD_STEP_BETA_DONE {
            self.beta.play();
        } else if self.particle_model == 0 {
            self.morph.play();
        } else {
            self.split.play();
        }
        if self.stone != 0 {
            self.stone_frame = self.stone_frame.saturating_add(1).min(STONE_FRAME_MAX);
        }
        if self.behaviour == BEHAVIOUR_CLIMB && self.climb_step != 3 {
            actor.pos = self.home;
            self.speed = Vec3::ZERO;
        } else {
            actor.pos += self.speed;
        }
    }

    pub fn bg_check(
        &mut self,
        actor: &mut Actor,
        from: Vec3,
        solid: Option<&Collision>,
        sea: Option<&Sea>,
        events: &mut SimEvents,
    ) -> bool {
        if self.shadow != 0 {
            return false;
        }
        if let Some(solid) = solid {
            actor.pos = solid.slide(from, actor.pos);
        }
        let (in_water, surface) = water::probe(sea, actor.pos.x, actor.pos.z);
        if self.floaty != 0 && in_water {
            actor.pos.y = surface + FLOAT_DEPTH;
        }
        self.slope = ground::slope_normal(solid, actor.pos);
        self.naraku_check(actor, solid, sea, events)
    }

    fn execute(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        solid: Option<&Collision>,
        sea: Option<&Sea>,
        blow: Blow,
        bombs: &[Vec3],
        lit: bool,
        rolling: bool,
        switches: &Switches,
        pots: &Query<(Entity, &Actor), (With<tsubo::Tsubo>, Without<Chuchu>)>,
        events: &mut SimEvents,
    ) -> bool {
        if self.state != Step::Climb
            && self.state != Step::Dead
            && self.health > 0
            && self.behaviour == BEHAVIOUR_CLIMB
            && self.ret_demo != 0
        {
            self.timers[TIMER_DEMO] = 0;
            self.state = Step::Climb;
            self.step = CLIMB_STEP_HUG;
        }
        match self.state {
            Step::Normal => {
                self.action_normal_move(actor, player, solid, blow, bombs, lit, events);
            }
            Step::Swim => self.action_swim(actor, player, solid, sea, events),
            Step::Attack => self.action_attack_move(actor, player, solid, blow, bombs, lit, events),
            Step::Damage => self.action_damage_move(actor, player, bombs, lit, events),
            Step::Climb => self.action_climb_move(actor, solid, rolling, events),
            Step::Dead => self.action_dead_move(events),
            Step::Stoned => self.action_stoned_move(actor, player, solid, lit, events),
            Step::Held => self.action_held_move(switches),
            Step::Pot => {
                self.action_pot_search(actor, player, pots, events);
                return false;
            }
        }
        if self.behaviour == BEHAVIOUR_POT {
            return false;
        }
        match self.spark {
            0 => {}
            1 => self.spark = 2,
            _ => {
                if self.timers[TIMER_SPARK] == 0 {
                    events.list.push(SimEvent::Se(SE_CM_CC_BLUE_SPARK));
                    self.timers[TIMER_SPARK] =
                        (rnd_f(&mut self.seed, SPARK_SPAN) + SPARK_BASE) as i16;
                }
            }
        }
        true
    }
}

pub fn is_chuchu(name: &str) -> bool {
    CHUCHU_ACTORS.contains(&name) || CHUCHU_COLOURS.contains(&name)
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    sea: Option<Res<'w, Sea>>,
    switches: Res<'w, Switches>,
    combat: Res<'w, Combat>,
    hammer: Res<'w, hammer::Cut>,
    held: Res<'w, Held>,
    blasts: Res<'w, bombflower::Blasts>,
    roll: Res<'w, roll::Roll>,
    players: Query<'w, 's, &'static Player>,
    arrows: Query<'w, 's, &'static arrow::Arrow>,
    bolts: Query<'w, 's, &'static boomerang::Boomerang>,
    chains: Query<'w, 's, &'static hookshot::Hookshot>,
    lamps: Query<'w, 's, (&'static Actor, &'static TagLight), Without<Chuchu>>,
    pots: Query<'w, 's, (Entity, &'static Actor), (With<tsubo::Tsubo>, Without<Chuchu>)>,
}

fn active_at(watch: &Watch<'_, '_>) -> At {
    if watch
        .chains
        .iter()
        .any(|chain| chain.phase != hookshot::Phase::Wait)
    {
        return At::Hook;
    }
    for shaft in watch.arrows.iter() {
        if shaft.phase != arrow::Phase::Flying {
            continue;
        }
        return match shaft.kind {
            arrow::TYPE_LIGHT => At::Light,
            arrow::TYPE_FIRE => At::FireArrow,
            arrow::TYPE_ICE => At::IceArrow,
            _ => At::Arrow,
        };
    }
    if watch
        .bolts
        .iter()
        .any(|boom| boom.parameter == boomerang::MODE_THROW)
    {
        return At::Boomerang;
    }
    if watch.hammer.kind == hammer::CUT_HAMMER_SIDESWING
        || watch.hammer.kind == hammer::CUT_HAMMER_FRONTSWING
    {
        return At::Hammer;
    }
    if watch.combat.active && watch.combat.frame >= 1 && watch.combat.frame < 6 {
        return At::Sword;
    }
    At::Idle
}

pub fn control(
    watch: Watch<'_, '_>,
    mut health: ResMut<Health>,
    mut zones: ResMut<ZoneActors>,
    mut events: ResMut<SimEvents>,
    mut slimes: Query<(Entity, &mut Actor, &mut Chuchu), With<Chuchu>>,
    mut commands: Commands,
) {
    let solid = watch.collision.as_deref();
    let sea = watch.sea.as_deref();
    let switches = watch.switches;
    let Some(player) = watch.players.iter().next().copied() else {
        return;
    };
    let at = active_at(&watch);
    let blow = Blow {
        at: if at == At::Sword
            && damage::sword_power(watch.held.item, watch.combat.soup) == damage::BOKO_STICK
        {
            At::Stick
        } else {
            at
        },
        blade: watch.held.item,
        soup: watch.combat.soup,
        turn: watch.hammer.kind == hammer::CUT_HAMMER_FRONTSWING,
    };
    let arrows = if (FIRST_ARROW..=LAST_ARROW).contains(&watch.held.item) {
        watch.held.count
    } else {
        0
    };
    let bombs: Vec<Vec3> = watch.blasts.list.clone();
    let lamps: Vec<(Vec3, i8)> = watch
        .lamps
        .iter()
        .filter(|(_, spot)| spot.level > taglight::OFF_LEVEL)
        .map(|(actor, _)| (actor.pos, actor.room))
        .collect();
    let mut spent: Vec<(Entity, i8, u16)> = Vec::new();
    for (entity, mut actor, mut slime) in &mut slimes {
        let actor = &mut *actor;
        let slime = &mut *slime;
        if !slime.entered {
            slime.place(actor);
        }
        if slime.enemy_ice(&mut events) {
            if slime.health <= LIGHT_DEAD {
                slime.doomed = true;
                slime.drop_item(actor, &health, arrows, &mut commands);
                spent.push((entity, actor.room, actor.set_id));
            }
            continue;
        }
        slime.tick_timers();
        if slime.freeze > 0 {
            slime.freeze -= 1;
            if slime.naraku_check(actor, solid, sea, &mut events) {
                slime.doomed = true;
                spent.push((entity, actor.room, actor.set_id));
            }
            continue;
        }
        slime.resolve(actor, player.pos, blow.at, !bombs.is_empty());
        if slime.body_hit {
            slime.body_atari_check(blow, &mut events);
        }
        slime.touch_player(actor, &player, &mut health);
        let lit = lamps
            .iter()
            .any(|(pos, room)| *room == actor.room && touches(*pos, 0.0, actor.pos, LAMP_REACH));
        if slime.health <= 0 && slime.state != Step::Dead && slime.state != Step::Stoned {
            slime.state = Step::Dead;
            slime.step = DEAD_STEP_BEGIN;
        }
        let from = actor.pos;
        let advance = slime.execute(
            actor,
            &player,
            solid,
            sea,
            blow,
            &bombs,
            lit,
            watch.roll.active,
            &switches,
            &watch.pots,
            &mut events,
        );
        if advance {
            slime.integrate(actor);
            if !slime.doomed && slime.bg_check(actor, from, solid, sea, &mut events) {
                slime.doomed = true;
            }
        }
        if slime.doomed {
            if slime.state == Step::Dead {
                slime.drop_item(actor, &health, arrows, &mut commands);
            }
            spent.push((entity, actor.room, actor.set_id));
        }
    }
    for (entity, room, set_id) in spent {
        zones.mark_done(room, set_id);
        let _ = commands.entity(entity).despawn();
    }
}
