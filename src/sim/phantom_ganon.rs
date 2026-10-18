use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::arrow;
use super::boko;
use super::bombflower;
use super::boomerang;
use super::collision::Collision;
use super::combat::Combat;
use super::damage;
use super::events::{SimEvent, SimEvents};
use super::fgmahou;
use super::flags::{Collect, StoryBits, Switches, ZoneActors};
use super::ground;
use super::hammer;
use super::health::Health;
use super::hookshot;
use super::inventory;
use super::item::Held;
use super::player::Player;
use super::ship::Ship;
use super::stream::add_calc_angle_s2;
use super::swim::Swim;
use super::walk::{from_s16, to_s16};

pub const PHANTOM_GANON_ACTOR: &str = "Fganon";
pub const ORB_ACTOR: &str = "Fgmahou";
pub const BOKO_SWORD: u32 = 5;

pub fn is_phantom_ganon(name: &str) -> bool {
    name == PHANTOM_GANON_ACTOR
}

const BYTE_MASK: u32 = 0xFF;
const NIBBLE_MASK: u32 = 0xF;
const BAND_SHIFT: u32 = 0x08;
const SWITCH_SHIFT: u32 = 0x10;
const OFF_SWITCH_SHIFT: u32 = 0x18;
const CLONE_SHIFT: u32 = 0x04;

const NO_SWITCH: u8 = 0xFF;
const ITEM_LIGHT_ARROW: u8 = 0x36;
const STORY_FINAL_FORM: u16 = 0x3A08;
const STORY_DEFEATED: u16 = 0x3F20;

const HIO_M05: u8 = 0;
const HIO_M06: u8 = 0;
const HIO_M08: i16 = 60;
const HIO_M0A: i16 = 500;
const HIO_M0C: f32 = 1500.0;
const HIO_M10: f32 = 400.0;
const HIO_M14: f32 = 1500.0;
const HIO_M18: f32 = 400.0;
const HIO_M1C: f32 = 45.0;
const HIO_M20: f32 = 45.0;
const HIO_M24: f32 = 2.0;
const HIO_M28: f32 = 2.0;
const HIO_M2C: f32 = 30.0;
const HIO_M30: f32 = -270.0;
const HIO_M34: f32 = -600.0;
const HIO_M38: f32 = 270.0;
const HIO_M3C: i16 = 50;
const HIO_M3E: i16 = 1;
const HIO_M40: i16 = 32;
const HIO_M48: i16 = 150;

const REG0_F0: f32 = 0.0;
const REG0_F6: f32 = 0.0;
const REG0_F1: f32 = 0.0;
const REG0_F2: f32 = 0.0;
const REG0_F3: f32 = 0.0;
const REG0_F4: f32 = 0.0;
const REG0_F5: f32 = 0.0;
const REG0_F7: f32 = 0.0;
const REG0_F9: f32 = 0.0;
const REG0_F10: f32 = 0.0;
const REG0_F11: f32 = 0.0;
const REG0_F12: f32 = 0.0;
const REG0_F13: f32 = 0.0;
const REG0_F14: f32 = 0.0;
const REG0_F15: f32 = 0.0;
const REG0_F17: f32 = 0.0;
const REG0_F18: f32 = 0.0;
const REG0_S0: i16 = 0;
const REG0_S1: i16 = 0;
const REG0_S3: i16 = 0;
const REG0_S4: i16 = 0;
const REG0_S5: i16 = 0;
const REG0_S6: i16 = 0;
const REG0_S8: i16 = 0;

const REG6_F0: f32 = 0.0;
const REG6_F1: f32 = 0.0;
const REG6_F2: f32 = 0.0;
const REG6_F4: f32 = 0.0;
const REG6_F5: f32 = 0.0;
const REG6_F6: f32 = 0.0;
const REG6_S0: i16 = 0;

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
const REG8_F11: f32 = 0.0;
const REG8_F18: f32 = 0.0;
const REG8_S2: i16 = 0;
const REG8_S5: i16 = 0;
const REG8_S6: i16 = 0;
const REG8_S7: i16 = 0;

const REG12_F1: f32 = 0.0;
const REG12_F2: f32 = 0.0;
const REG12_F3: f32 = 0.0;
const REG12_F10: f32 = 0.0;

const REG17_S0: i16 = 0;
const REG17_S2: i16 = 0;
const REG17_S3: i16 = 0;

const EMODE_NONE: u8 = 0;
const EMODE_LOOP: u8 = 2;
const STATE_STOP: u8 = 1;
const STATE_LOOP: u8 = 2;

const BCK_DAMAGE1: i32 = 0x6;
const BCK_DAMAGE2: i32 = 0x7;
const BCK_DOWN1: i32 = 0x8;
const BCK_KAITEN1: i32 = 0x9;
const BCK_KAITEN_S1: i32 = 0xA;
const BCK_LAST_DAMAGE1: i32 = 0xB;
const BCK_MOGAKU1: i32 = 0xC;
const BCK_NAGERU1: i32 = 0xD;
const BCK_NAGERU_S1: i32 = 0xE;
const BCK_SARU1: i32 = 0xF;
const BCK_TAME1: i32 = 0x10;
const BCK_TAME_S1: i32 = 0x11;
const BCK_TAME_S2: i32 = 0x12;
const BCK_TENNISLR1: i32 = 0x13;
const BCK_TENNISRL1: i32 = 0x14;
const BCK_TYAKUTI1: i32 = 0x15;
const BCK_WAIT1: i32 = 0x16;
const BCK_WARAU1: i32 = 0x17;

const BCK_DAMAGE1_FRAMES: i16 = 0x1F;
const BCK_DAMAGE2_FRAMES: i16 = 0x15;
const BCK_DOWN1_FRAMES: i16 = 0x18;
const BCK_KAITEN1_FRAMES: i16 = 0x2F;
const BCK_KAITEN_S1_FRAMES: i16 = 0x20;
const BCK_LAST_DAMAGE1_FRAMES: i16 = 0x77;
const BCK_MOGAKU1_FRAMES: i16 = 0x78;
const BCK_NAGERU1_FRAMES: i16 = 0x1F;
const BCK_NAGERU_S1_FRAMES: i16 = 0x28;
const BCK_SARU1_FRAMES: i16 = 0x3C;
const BCK_TAME1_FRAMES: i16 = 0x20;
const BCK_TAME_S1_FRAMES: i16 = 0x23;
const BCK_TAME_S2_FRAMES: i16 = 0x30;
const BCK_TENNISLR1_FRAMES: i16 = 0x1F;
const BCK_TENNISRL1_FRAMES: i16 = 0x1F;
const BCK_TYAKUTI1_FRAMES: i16 = 0x13;
const BCK_WAIT1_FRAMES: i16 = 0x30;
const BCK_WARAU1_FRAMES: i16 = 0x0C;

const TIMER_COUNT: usize = 5;

pub const CULL_MIN: [f32; 3] = [-150.0, -50.0, -150.0];
pub const CULL_MAX: [f32; 3] = [150.0, 300.0, 150.0];

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const SE_LK_PG_BOMB_STRIKE: u32 = 0x28A5;
const SE_CM_PG_FLYING: u32 = 0x303E;
const SE_CV_PG_LAUGH_1: u32 = 0x4938;
const SE_CV_PG_LAUGH_2: u32 = 0x4939;
const SE_CV_PG_EBALL_MAKE_S: u32 = 0x493A;
const SE_CV_PG_EBALL_MAKE_L: u32 = 0x493B;
const SE_CV_PG_EBALL_FIRE_S: u32 = 0x493C;
const SE_CV_PG_EBALL_FIRE_L: u32 = 0x493D;
const SE_CV_PG_REFLECT_EBALL: u32 = 0x493E;
const SE_CV_PG_HIT_EBALL: u32 = 0x493F;
const SE_CV_PG_TIRED: u32 = 0x4940;
const SE_CV_PG_LAUGH_3: u32 = 0x4941;
const SE_CV_PG_DAMAGE_S: u32 = 0x4942;
const SE_CV_PG_DAMAGE_L: u32 = 0x4943;
const SE_CV_PG_DIE: u32 = 0x4945;
const SE_CM_PG_EBALL_MAKING_S: u32 = 0x5152;
const SE_CM_PG_EBALL_MAKING_L: u32 = 0x5153;
const SE_CM_L_ARROW_SHRINK: u32 = 0x5904;
const SE_CM_L_ARROW_PASS_AWAY: u32 = 0x5905;
const SE_CM_PG_APPEAR: u32 = 0x594E;
const SE_CM_PG_DISAPPEAR: u32 = 0x594F;
const SE_CM_PG_SWING_S: u32 = 0x5950;
const SE_CM_PG_SWING_L: u32 = 0x5951;
const SE_CM_PG_EBALL_FIRE_S: u32 = 0x5954;
const SE_CM_PG_EBALL_FIRE_L: u32 = 0x5955;
const SE_CM_PG_END_1: u32 = 0x5956;
const SE_CM_PG_END_2: u32 = 0x5957;
const SE_OBJ_PG_EBALL_FLY_S: u32 = 0x6237;
const SE_OBJ_TN_SWORD_FALL: u32 = 0x6998;
const SE_OBJ_PG_EBALL_EXP_S: u32 = 0x6A35;

const BGM_P_GANON_1: u32 = 0x8000_0041;
const BGM_P_GANON_2: u32 = 0x8000_0047;

const ACTION_STANDBY: i16 = 0;
const ACTION_DERU: i16 = 1;
const ACTION_KIERU: i16 = 2;
const ACTION_FAIL: i16 = 3;
const ACTION_FLY: i16 = 5;
const ACTION_SHOT: i16 = 6;
const ACTION_SPIN: i16 = 7;
const ACTION_DOWN: i16 = 8;
const ACTION_FLY2: i16 = 9;
const ACTION_SPIN2: i16 = 10;
const ACTION_SHOT2: i16 = 11;
const ACTION_START: i16 = 20;
const ACTION_END: i16 = 21;
const ACTION_LAST_END: i16 = 22;

const MODE_ENTRY: i16 = -10;
const MODE_HIDE: i16 = -1;
const MODE_DAMAGE: i16 = 10;

const MASTER_HEALTH: i32 = 30;
const STAGE_HEALTH: i32 = 100;
const FALL_HEALTH: i32 = 100;
const COMBO_LEFT: i8 = 10;

const ACTIVATION_SCALE: f32 = 10.0;
const PLAYER_GATE: f32 = 710.0;
const SPAWN_LIFT: f32 = 10000.0;
const GRAVITY_DOWN: f32 = 0.5;
const MAX_FALL_DOWN: f32 = -15.0;
const RISE_GRAVITY: f32 = 0.2;
const MAX_RISE: f32 = 20.0;
const GROUND_PROBE: f32 = 300.0;
const MOVE_SCALE: f32 = 0.05;
const FLY_DISTANCE: f32 = 300.0;
const FLY_ACCEL: f32 = 30.0;
const FLY_DECEL: f32 = 2.0;
const FLY_BRAKE: f32 = 1.0;
const FLY_SCALE: f32 = 1500.0;
const FACE_DIV: i16 = 10;
const FACE_CAP: i16 = 0x400;
const FLY_SE_SCALE: f32 = 3.5;
const WAIT_SPAN: f32 = 50.0;
const WAIT_BASE: f32 = 50.0;
const SPIN_DIV: i16 = 5;
const SPIN_ACCEL: f32 = 20.0;
const SPIN_STEP: f32 = 4.0;
const SWING_FRAME: i32 = 14;
const SWING_FIRST: i32 = 5;
const SWING_LAST: i32 = 15;
const SWING_EXIT: f32 = 20.0;
const SWING_GATE: i8 = 7;
const SWING_LIMIT: i8 = 4;
const SWING_CHANCE: f32 = 0.3;
const SWING_BASE: f32 = 400.0;
const SWING_SPAN: f32 = 10.0;
const KABE_RAYS: usize = 8;
const KABE_STEP: i16 = 0x2000;
const ORB_COUNT: usize = 8;
const ORB_ALPHA_START: u8 = 250;
const ORB_ALPHA_STEP: u8 = 50;
const ORB_LIFT_BASE: f32 = 400.0;
const ORB_LIFT_SPEED: f32 = 250.0;
const ORB_LIFT_STEP: f32 = 50.0;
const ORB_HIT_LIMIT: i8 = 5;
const SWORD_TILT_MAX: f32 = 15884.0;
const SWORD_SPIN_CAP: f32 = 700.0;
const SWORD_SPIN_GAIN: f32 = 0.4;
const SWORD_ACCEL_BASE: f32 = 3.0;
const SWORD_ACCEL_CAP: f32 = 1000.0;
const SPIN_SEAT_Y: f32 = 100.0;
const SWORD_TURN_DIV: i16 = 1;
const SWORD_YAW_OFFSET: i16 = 0x7058;
const SWORD_SPEED_SPAN: u32 = 10_000;
const SWORD_TURN_STEP: i16 = 0x80;
const SWORD_CAP_BASE: i16 = 0x800;
const SPIN_OFFSET: i16 = 0x3333;
const SPIN_SPEED_BASE: f32 = 5.0;
const SPIN_SPEED_STEP: f32 = 0.5;
const SPIN_TARGET: f32 = 20.0;
const CLONE_COUNT: usize = 4;
const BANISH_FRAMES: i16 = 30;
const DROP_FRAMES: i16 = 30;
const DROP_HOLD: i16 = 10;
const LAST_HOLD_A: i16 = 25;
const LAST_HOLD_B: i16 = 150;
const LAST_HOLD_C: i16 = 20;
const LAST_HOLD_D: i16 = 30;
const LAST_HOLD_E: i16 = 30;
const LAST_FOG_START: f32 = 10_000.0;
const LAST_FOG_END: f32 = 5100.0;
const LAST_FOG_SPEED: f32 = 100.0;
const LAST_FOG_SCALE: f32 = 63.5;
const LAST_SCALE: f32 = 0.1;
const LAST_RISE: f32 = 10.0;
const TURN_PITCH_DIV: i16 = 4;
const TURN_PITCH_CAP: i16 = 0x800;
const BALL_TG_RADIUS: f32 = 100.0;
const BALL_AT_RADIUS: f32 = 25.0;
const BALL_PROBE_AIM: f32 = 50.0;
const BALL_PROBE_SWING: f32 = 30.0;
const BALL_GAP: f32 = 200.0;
const BALL_RANGE_BASE: f32 = 100.0;
const BALL_PAUSE_BREAK: i8 = 50;
const BALL_PAUSE_FIRE: i8 = 5;
const FAR_OFF: f32 = -20_000.0;
const FAR_OFF_LOW: f32 = -10_000.0;
const FAR_OFF_UP: f32 = 20_000.0;
const BALL_LIMIT: f32 = 10_000.0;
const BALL_PREV_SCALE: f32 = 1.05;
const BODY_RADIUS: f32 = 70.0;
const BODY_HEIGHT: f32 = 250.0;
const BODY_LIFT: f32 = -150.0;
const PLAYER_BODY: f32 = 20.0;
const REACH_SWORD: f32 = 20.0;
const SWAY_YAW: i16 = 700;
const SWAY_PITCH: i16 = 750;
const SWAY_ROLL: i16 = 720;
const SWAY_SPAN: f32 = 50.0;
const SWAY_STEP: f32 = 2.0;
const PUSH_GATE: f32 = 0.01;
const PUSH_STEP: f32 = 7.0;
const PUSH_RATE: f32 = 80.0;
const PUSH_FRAMES: i16 = 7;
const BALL_POWER: u8 = 1;

const JNT_AGO1: [f32; 3] = [-0.00072, 258.75162, 2.51625];
const JNT_MUNE2: [f32; 3] = [-0.0005, 179.77229, 4.77766];
const JNT_ITEM1: [f32; 3] = [-192.50333, 205.01847, -9.97463];
const JNT_ITEM1_TIP: [f32; 3] = [-192.49533, 205.0, 90.02537];
const JNT_UDE_L3: [f32; 3] = [174.9995, 205.01682, -10.00957];
const BALL_SPAWN_X: f32 = 30.0;
const BALL_SPAWN_Y: f32 = 30.0;
const MUNE2_DROP: f32 = 150.0;
const PLAYER_EYE_Y: f32 = 60.0;
const CYL_HEIGHT: f32 = 250.0;
const CYL_RADIUS: f32 = 70.0;
const KEN_RADIUS: f32 = 60.0;
const KEN_ATP: i32 = 8;

const DEMO_INTRO: i16 = 47;
const DEMO_INTRO_HOLD: i16 = 60;
const DEMO_FOV: f32 = 60.0;
const DEMO_INTRO_LAUGH: i16 = 2;
const DEMO_INTRO_ENTER: i16 = 45;
const DEMO_INTRO_END: i16 = 280;
const DEMO_WAIT_MODE: i16 = 110;
const DEMO_TAME_MODE: i16 = 180;
const DEMO_BALL_FRAME: i16 = 29;
const DEMO_STAND_HOLD: i16 = 0x96;
const DEMO_STAND_MODE: i16 = 10;
const DEMO_STAND_END: i16 = 40;
const DEMO_CAPE_DROP: i16 = 30;
const DEMO_CAPE_LIFT: f32 = 30.0;
const DEMO_CAPE_END: i16 = 130;
const DEMO_CAP_SWING: i16 = 100;
const DEMO_CAP_LAUGH: i16 = 20;
const DEMO_STAND_FOV: f32 = 55.0;

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        if let Some(slot) = seed.get_mut(lane) {
            *slot = (*slot * RND_MUL[lane]) % RND_MOD[lane];
        }
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

fn sin_s(units: i16) -> f32 {
    from_s16(units).sin()
}

fn bearing(from: Vec3, to: Vec3) -> i16 {
    to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn flat(from: Vec3, to: Vec3) -> f32 {
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    (dx * dx + dz * dz).sqrt()
}

fn turned(yaw: i16, offset: [f32; 3]) -> Vec3 {
    let (sin, cos) = from_s16(yaw).sin_cos();
    Vec3::new(
        offset[0] * cos + offset[2] * sin,
        offset[1],
        offset[2] * cos - offset[0] * sin,
    )
}

fn pitched(pitch: i16, offset: Vec3) -> Vec3 {
    let (sin, cos) = from_s16(pitch).sin_cos();
    Vec3::new(
        offset.x,
        offset.y * cos - offset.z * sin,
        offset.y * sin + offset.z * cos,
    )
}

fn aimed(yaw: i16, pitch: i16, distance: f32) -> Vec3 {
    pitched(pitch, turned(yaw, [0.0, 0.0, distance]))
}

fn within(from: Vec3, to: Vec3, radius: f32) -> bool {
    (to - from).length_squared() <= radius * radius
}

fn bind(ofs: [f32; 3], shape_yaw: i16) -> Vec3 {
    turned(shape_yaw, ofs)
}

fn timer(value: &mut i16) {
    if *value != 0 {
        *value -= 1;
    }
}

fn timer8(value: &mut i8) {
    if *value != 0 {
        *value -= 1;
    }
}

fn frame_max(bck: i32) -> i16 {
    match bck {
        BCK_DAMAGE1 => BCK_DAMAGE1_FRAMES,
        BCK_DAMAGE2 => BCK_DAMAGE2_FRAMES,
        BCK_DOWN1 => BCK_DOWN1_FRAMES,
        BCK_KAITEN1 => BCK_KAITEN1_FRAMES,
        BCK_KAITEN_S1 => BCK_KAITEN_S1_FRAMES,
        BCK_LAST_DAMAGE1 => BCK_LAST_DAMAGE1_FRAMES,
        BCK_MOGAKU1 => BCK_MOGAKU1_FRAMES,
        BCK_NAGERU1 => BCK_NAGERU1_FRAMES,
        BCK_NAGERU_S1 => BCK_NAGERU_S1_FRAMES,
        BCK_SARU1 => BCK_SARU1_FRAMES,
        BCK_TAME1 => BCK_TAME1_FRAMES,
        BCK_TAME_S1 => BCK_TAME_S1_FRAMES,
        BCK_TAME_S2 => BCK_TAME_S2_FRAMES,
        BCK_TENNISLR1 => BCK_TENNISLR1_FRAMES,
        BCK_TENNISRL1 => BCK_TENNISRL1_FRAMES,
        BCK_TYAKUTI1 => BCK_TYAKUTI1_FRAMES,
        BCK_WAIT1 => BCK_WAIT1_FRAMES,
        BCK_WARAU1 => BCK_WARAU1_FRAMES,
        _ => BCK_WAIT1_FRAMES,
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
        let start = self.start as f32;
        let end = self.end as f32;
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
                    self.frame += low as f32;
                }
                while self.frame >= end {
                    self.state |= STATE_LOOP;
                    if high <= 0 {
                        break;
                    }
                    self.frame -= high as f32;
                }
            }
            _ => {}
        }
    }

    fn play(&mut self) {
        if self.cur_morf < 1.0 {
            self.prev_morf = self.cur_morf;
            self.cur_morf = if self.cur_morf + self.morf_step > 1.0 {
                1.0
            } else {
                self.cur_morf + self.morf_step
            };
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
}

impl At {
    fn power(self, blade: u8, soup: bool) -> i32 {
        match self {
            Self::Sword => damage::sword_power(blade, soup),
            Self::Arrow | Self::FireArrow | Self::IceArrow | Self::LightArrow => damage::ARROW,
            Self::Bomb => damage::BOMB,
            Self::Boomerang => damage::BOMERANG,
            Self::Hammer => hammer::AT_POWER,
            Self::Idle | Self::Hookshot => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blow {
    pub at: At,
    pub blade: u8,
    pub soup: bool,
}

pub struct Env<'a> {
    pub player_pos: Vec3,
    pub player_eye: Vec3,
    pub player_face: i16,
    pub player_ride: bool,
    pub player_swim: bool,
    pub player_bottle: bool,
    pub solid: Option<&'a Collision>,
    pub blow: Blow,
    pub has_light_arrow: bool,
    pub light_touch: bool,
    pub orb_touch: bool,
    pub arrow_pos: Option<Vec3>,
    pub master_action: i16,
}

impl Env<'_> {
    fn crossed(&self, from: Vec3, to: Vec3) -> (bool, Vec3) {
        match self.solid {
            Some(solid) => {
                let hit = solid.slide(from, to);
                if hit != to { (true, hit) } else { (false, to) }
            }
            None => (false, to),
        }
    }

    fn strike_pos(&self) -> Option<Vec3> {
        match self.blow.at {
            At::Sword | At::LightArrow => Some(self.player_eye),
            At::Arrow | At::FireArrow | At::IceArrow => self.arrow_pos,
            _ => None,
        }
    }

    fn blocked(&self, from: Vec3, to: Vec3) -> bool {
        self.crossed(from, to).0
    }

    fn landing(&self, pos: Vec3) -> f32 {
        let probe = Vec3::new(pos.x, pos.y + GROUND_PROBE, pos.z);
        match ground::surface(self.solid, probe) {
            Some(height) => height,
            None => f32::NEG_INFINITY,
        }
    }
}

pub struct Sink<'a> {
    pub events: &'a mut SimEvents,
    pub stories: &'a mut StoryBits,
    pub switches: &'a mut Switches,
    pub health: &'a mut Health,
    pub delete: bool,
    pub orbs: bool,
    pub clones: Vec<u32>,
    pub sword: Option<bool>,
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

    fn story(&mut self, id: u16) {
        self.stories.set(id);
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct PhantomGanon {
    pub kind: u8,
    pub band: u8,
    pub switch_no: u8,
    pub off_switch: u8,
    pub clone: u8,
    pub action: i16,
    pub mode: i16,
    pub morph: Frame,
    pub home_pos: Vec3,
    pub home_angle: i16,
    pub old_pos: Vec3,
    pub speed: Vec3,
    pub speed_f: f32,
    pub shape_angle: [i16; 3],
    pub scale: Vec3,
    pub eye: Vec3,
    pub model_offset: Vec3,
    pub body: Vec3,
    pub body_live: bool,
    pub body_tg: bool,
    pub cyl_radius: f32,
    pub cyl_height: f32,
    pub ken: Vec3,
    pub ken_radius: f32,
    pub ken_atp: i32,
    pub ken_at_live: i8,
    pub ken_held: i8,
    pub slot_pos: Vec3,
    pub orbit_yaw: i16,
    pub turn_scale: f32,
    pub move_scale: f32,
    pub timers: [i16; TIMER_COUNT],
    pub invuln: i16,
    pub push_xz: f32,
    pub push_yaw: i16,
    pub push_pitch: i16,
    pub shake: i16,
    pub sway: f32,
    pub fog_end: f32,
    pub ball_pos: Vec3,
    pub ball_prev: Vec3,
    pub ball_step: Vec3,
    pub ball_speed: f32,
    pub ball_mode: i8,
    pub ball_fire: i8,
    pub ball_pause: i8,
    pub ball_live: bool,
    pub ball_tg_set: bool,
    pub ball_at_radius: f32,
    pub ball_tg_radius: f32,
    pub ball_hit_player: bool,
    pub orb_home: Vec3,
    pub orb_hits: i8,
    pub orb_stage: i8,
    pub orb_alpha: u8,
    pub orb_lift: f32,
    pub ken_active: i8,
    pub swing_hit: i8,
    pub return_count: i8,
    pub tension: i8,
    pub no_reflect: i8,
    pub laugh_timer: i8,
    pub hurt_gauge: i8,
    pub ball_landed: i8,
    pub materialized: bool,
    pub light_shot: i8,
    pub combo_left: i8,
    pub sword_tilt: f32,
    pub sword_spin: f32,
    pub sword_accel: f32,
    pub sword_yaw: i16,
    pub sword_roll: i16,
    pub sword_step: i16,
    pub sword_dangle: i16,
    pub sword_timer: u32,
    pub sword_live: i8,
    pub demo_state: i16,
    pub demo_count: i16,
    pub cam_eye: Vec3,
    pub cam_look: Vec3,
    pub cam_swing: i16,
    pub cam_rate: f32,
    pub cam_fov: f32,
    pub bgm_delay: i8,
    pub health: i32,
    pub max_health: i32,
    pub lock_on: bool,
    pub grounded: bool,
    pub entered: bool,
    pub tick: i16,
    pub seed: [i32; 3],
}

impl PhantomGanon {
    pub fn spawn(parameters: u32) -> Self {
        let kind = (parameters & NIBBLE_MASK) as u8;
        let clone = if kind == 3 {
            ((parameters >> CLONE_SHIFT) & NIBBLE_MASK) as u8
        } else {
            0
        };
        let mut morph = Frame::idle();
        morph.init(frame_max(BCK_WAIT1));
        morph.attribute = EMODE_LOOP;
        morph.rate = 1.0;
        morph.set_morf(0.0);
        Self {
            kind,
            band: ((parameters >> BAND_SHIFT) & BYTE_MASK) as u8,
            switch_no: ((parameters >> SWITCH_SHIFT) & BYTE_MASK) as u8,
            off_switch: ((parameters >> OFF_SWITCH_SHIFT) & BYTE_MASK) as u8,
            clone,
            action: ACTION_STANDBY,
            mode: 0,
            morph,
            home_pos: Vec3::ZERO,
            home_angle: 0,
            old_pos: Vec3::ZERO,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            shape_angle: [0; 3],
            scale: Vec3::ONE,
            eye: Vec3::ZERO,
            model_offset: Vec3::ZERO,
            body: Vec3::ZERO,
            body_live: false,
            body_tg: true,
            cyl_radius: CYL_RADIUS,
            cyl_height: CYL_HEIGHT,
            ken: Vec3::new(FAR_OFF_LOW, FAR_OFF_LOW, FAR_OFF_LOW),
            ken_radius: KEN_RADIUS,
            ken_atp: 0,
            ken_at_live: 0,
            ken_held: 0,
            slot_pos: Vec3::ZERO,
            orbit_yaw: 0,
            turn_scale: 0.0,
            move_scale: 0.0,
            timers: [0; TIMER_COUNT],
            invuln: 0,
            push_xz: 0.0,
            push_yaw: 0,
            push_pitch: 0,
            shake: 0,
            sway: 0.0,
            fog_end: 0.0,
            ball_pos: Vec3::ZERO,
            ball_prev: Vec3::ZERO,
            ball_step: Vec3::ZERO,
            ball_speed: 0.0,
            ball_mode: 0,
            ball_fire: 0,
            ball_pause: 0,
            ball_live: false,
            ball_tg_set: false,
            ball_at_radius: BALL_AT_RADIUS,
            ball_tg_radius: BALL_TG_RADIUS,
            ball_hit_player: false,
            orb_home: Vec3::ZERO,
            orb_hits: 0,
            orb_stage: 0,
            orb_alpha: 0,
            orb_lift: 0.0,
            ken_active: 0,
            swing_hit: 0,
            return_count: 0,
            tension: 0,
            no_reflect: 0,
            laugh_timer: 0,
            hurt_gauge: 0,
            ball_landed: 0,
            materialized: false,
            light_shot: 0,
            combo_left: 0,
            sword_tilt: 0.0,
            sword_spin: 0.0,
            sword_accel: 0.0,
            sword_yaw: 0,
            sword_roll: 0,
            sword_step: 0,
            sword_dangle: 0,
            sword_timer: 0,
            sword_live: 0,
            demo_state: 0,
            demo_count: 0,
            cam_eye: Vec3::ZERO,
            cam_look: Vec3::ZERO,
            cam_swing: 0,
            cam_rate: 0.0,
            cam_fov: 0.0,
            bgm_delay: 0,
            health: MASTER_HEALTH,
            max_health: MASTER_HEALTH,
            lock_on: false,
            grounded: false,
            entered: false,
            tick: 0,
            seed: [RND_SEED; 3],
        }
    }

    pub fn blocked_by_switch(parameters: u32, switches: &Switches) -> bool {
        let switch_no = ((parameters >> SWITCH_SHIFT) & BYTE_MASK) as u8;
        switch_no != NO_SWITCH && switches.test(switch_no)
    }

    pub fn drop_sword(parameters: u32, switches: &Switches, stories: &StoryBits) -> bool {
        PhantomGanon::blocked_by_switch(parameters, switches)
            && (parameters & NIBBLE_MASK) == 2
            && !stories.test(STORY_FINAL_FORM)
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.materialized
    }

    pub fn hull(&self) -> (f32, f32) {
        (BODY_HEIGHT, BODY_RADIUS)
    }

    fn rnd_f(&mut self, range: f32) -> f32 {
        rnd_f(&mut self.seed, range)
    }

    fn rnd_fx(&mut self, range: f32) -> f32 {
        rnd_fx(&mut self.seed, range)
    }

    fn tick_timers(&mut self) {
        for slot in self.timers.iter_mut() {
            if *slot != 0 {
                *slot -= 1;
            }
        }
    }

    fn tv(&self, index: usize) -> i16 {
        self.timers.get(index).copied().unwrap_or(0)
    }

    fn set_t(&mut self, index: usize, value: i16) {
        if let Some(slot) = self.timers.get_mut(index) {
            *slot = value;
        }
    }

    fn frame_i(&self) -> i32 {
        self.morph.frame as i32
    }

    fn frame_is(&self, at: i32) -> bool {
        (self.morph.frame as i32) == at
    }

    fn anm_init(&mut self, bck: i32, morf: f32, loop_mode: u8, speed: f32) {
        self.morph.init(frame_max(bck));
        self.morph.attribute = loop_mode;
        self.morph.rate = speed;
        self.morph.set_morf(morf);
    }

    fn deru_brk(&mut self, out: &mut Sink<'_>) {
        out.se(SE_CM_PG_APPEAR);
        self.materialized = true;
    }

    fn kieru_brk(&mut self, part: u8, out: &mut Sink<'_>) {
        if part == 0 || part == 1 {
            out.se(SE_CM_PG_DISAPPEAR);
        }
        self.materialized = false;
    }

    fn deru_brk2(&mut self, out: &mut Sink<'_>) {
        out.se(SE_CM_PG_APPEAR);
        self.materialized = true;
    }

    fn kieru_brk2(&mut self, out: &mut Sink<'_>) {
        out.se(SE_CM_PG_DISAPPEAR);
        self.materialized = false;
    }

    fn pos_move(&mut self, actor: &mut Actor, param_2: u8) {
        if param_2 == 0 {
            let delta = self.slot_pos - actor.pos;
            let y_target = to_s16(delta.x.atan2(delta.z));
            let span = (delta.x * delta.x + delta.z * delta.z).sqrt();
            let x_target = to_s16(delta.y.atan2(span)).wrapping_neg();
            let scale = SPIN_DIV;
            let step = (self.turn_scale * self.move_scale) as i16;
            add_calc_angle_s2(&mut actor.angle[1], y_target, scale + REG0_S3, step);
            add_calc_angle_s2(&mut actor.angle[0], x_target, scale + REG0_S3, step);
        }
        add_calc2(&mut self.move_scale, 1.0, 1.0, MOVE_SCALE);
        let step = turned(actor.angle[1], [0.0, 0.0, self.speed_f]);
        self.speed = pitched(actor.angle[0], step);
        actor.pos += self.speed;
    }

    fn fly_se_set(&mut self, actor: &Actor, out: &mut Sink<'_>) {
        let _span = ((actor.pos - self.old_pos).length() * FLY_SE_SCALE) as u32;
        out.se(SE_CM_PG_FLYING);
    }

    fn fly(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>, big: bool) {
        if ((self.tick & 0xF) == 0 && self.rnd_f(1.0) < 0.5) || self.mode == MODE_ENTRY {
            self.orbit_yaw = env.player_face;
        }
        match self.mode {
            MODE_ENTRY => {
                self.deru_brk(out);
                actor.pos = self.home_pos;
                actor.pos.y = env.player_pos.y + if big { HIO_M18 } else { HIO_M10 };
                self.shape_angle[1] = bearing(actor.pos, env.player_eye);
                self.mode = 1;
                self.anm_init(BCK_WAIT1, 20.0, EMODE_LOOP, 1.0);
                self.set_t(1, if big { 40 } else { 60 });
            }
            0 => {
                self.anm_init(BCK_WAIT1, 20.0, EMODE_LOOP, 1.0);
                self.mode += 1;
                self.move_scale = 0.0;
                let span = (self.rnd_f(WAIT_SPAN) + WAIT_BASE) as i16;
                self.set_t(1, span);
                self.fly_hold(actor, env, big);
            }
            1 => self.fly_hold(actor, env, big),
            _ => {}
        }
        self.pos_move(actor, 0);
        self.fly_se_set(actor, out);
        if self.tv(1) == 0 {
            self.action = if big { ACTION_SHOT2 } else { ACTION_SHOT };
            self.mode = 0;
        }
        if self.tv(2) == 0 {
            self.action = ACTION_KIERU;
            self.mode = 0;
        }
    }

    fn fly_hold(&mut self, actor: &mut Actor, env: &Env<'_>, big: bool) {
        if self.laugh_timer == 1 {
            self.anm_init(BCK_WAIT1, 10.0, EMODE_LOOP, 1.0);
        }
        let forward = if big { HIO_M14 } else { HIO_M0C };
        let lift = if big { HIO_M18 } else { HIO_M10 };
        let seat = env.player_pos + turned(self.orbit_yaw, [0.0, lift, forward]);
        self.slot_pos = seat;
        let delta = seat - actor.pos;
        if delta.length() > REG0_F12 + FLY_DISTANCE {
            add_calc2(
                &mut self.speed_f,
                REG0_F15 + FLY_ACCEL,
                1.0,
                REG0_F13 + FLY_DECEL,
            );
        } else {
            add_calc0(&mut self.speed_f, 1.0, REG0_F14 + FLY_BRAKE);
        }
        self.turn_scale = REG0_F5 + FLY_SCALE;
        add_calc_angle_s2(
            &mut self.shape_angle[1],
            bearing(actor.pos, env.player_eye),
            FACE_DIV,
            FACE_CAP,
        );
        add_calc2(&mut self.sway, REG0_F11 + SWAY_SPAN, 1.0, SWAY_STEP);
    }

    fn shot(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        add_calc_angle_s2(
            &mut self.shape_angle[1],
            bearing(actor.pos, env.player_eye),
            FACE_DIV,
            FACE_CAP,
        );
        match self.mode {
            0 => {
                if self.ball_mode != 0 {
                    self.shot_tail(actor, out);
                    return;
                }
                self.anm_init(BCK_TAME1, 10.0, EMODE_LOOP, 1.0);
                self.mode += 1;
                self.set_t(0, 40);
                out.se(SE_CV_PG_EBALL_MAKE_S);
                self.shot_make(out);
            }
            1 => self.shot_make(out),
            2 => {
                if self.frame_is(i32::from(REG0_S0) + SWING_FRAME) {
                    self.ball_fire = 1;
                    self.swing_hit = 0;
                    self.return_count = 0;
                    self.no_reflect = 0;
                    self.tension = 0;
                }
                if self.morph.is_stop() {
                    self.anm_init(BCK_WAIT1, 3.0, EMODE_LOOP, 1.0);
                    self.mode += 1;
                }
                if self.shot_catch(out) {
                    return;
                }
            }
            3 => {
                if self.shot_catch(out) {
                    return;
                }
            }
            _ => self.shot_swing(),
        }
        self.shot_tail(actor, out);
    }

    fn shot_make(&mut self, out: &mut Sink<'_>) {
        if self.tv(0) == 30 {
            self.ball_mode = 1;
        }
        if self.tv(0) < 30 {
            out.se(SE_CM_PG_EBALL_MAKING_S);
        }
        if self.tv(0) == 0 {
            self.anm_init(BCK_NAGERU1, 3.0, EMODE_NONE, 1.0);
            self.mode += 1;
            out.se(SE_CV_PG_EBALL_FIRE_S);
            out.se(SE_CM_PG_EBALL_FIRE_S);
        }
    }

    fn shot_catch(&mut self, out: &mut Sink<'_>) -> bool {
        if self.ball_mode != 5 {
            return false;
        }
        let delta = self.ball_pos - self.eye;
        let reach = REG0_F1 + (self.ball_speed * (REG0_F0 + SWING_SPAN) + SWING_BASE);
        if !(delta.length() < reach) {
            return false;
        }
        if self.no_reflect == 0 {
            if self.rnd_f(1.0) < 0.5 {
                self.anm_init(BCK_TENNISLR1, 0.0, EMODE_NONE, 1.0);
            } else {
                self.anm_init(BCK_TENNISRL1, 0.0, EMODE_NONE, 1.0);
            }
            self.mode += 1;
            out.se(SE_CV_PG_REFLECT_EBALL);
            out.se(SE_CM_PG_SWING_S);
            self.return_count += 1;
            if self.tension >= SWING_GATE
                || (self.return_count >= SWING_LIMIT && self.rnd_f(1.0) < SWING_CHANCE)
            {
                self.no_reflect = 1;
            }
            return false;
        }
        if self.kind == 2 {
            self.action = ACTION_KIERU;
            self.mode = 0;
            return true;
        }
        false
    }

    fn shot_swing(&mut self) {
        if self.frame_i() >= SWING_FIRST && self.frame_i() <= SWING_LAST {
            self.swing_hit = 1;
        }
        if self.morph.is_stop() || (self.kind >= 1 && self.morph.frame >= SWING_EXIT) {
            self.anm_init(BCK_WAIT1, 10.0, EMODE_LOOP, 1.0);
            self.mode = 3;
        }
    }

    fn shot_tail(&mut self, actor: &mut Actor, out: &mut Sink<'_>) {
        add_calc0(&mut self.speed_f, 1.0, REG0_F14 + FLY_BRAKE);
        self.pos_move(actor, 0);
        self.fly_se_set(actor, out);
        if self.mode >= 2 && self.ball_mode == 0 {
            self.action = ACTION_FLY;
            self.mode = 0;
        }
        if self.mode == 3 || self.mode == 4 {
            out.se(SE_OBJ_PG_EBALL_FLY_S);
        }
    }

    fn spin_attack(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        let frame = self.frame_i();
        if self.mode == 0 {
            self.shape_angle[1] = env.player_face;
            self.mode = 1;
        }
        if self.mode == 1 {
            let seat =
                env.player_pos + turned(self.shape_angle[1], [0.0, HIO_M2C + SPIN_SEAT_Y, HIO_M30]);
            actor.pos = seat;
            if !env.blocked(env.player_pos, actor.pos) {
                actor.pos.y -= 100.0;
                self.anm_init(BCK_WAIT1, 1.0, EMODE_LOOP, 1.0);
                self.mode = 2;
                self.set_t(0, 30);
                self.deru_brk(out);
            } else {
                self.shape_angle[1] = to_s16(self.rnd_fx(32768.0));
                return;
            }
        }
        if self.mode == 2 {
            if self.tv(0) == 0 {
                self.anm_init(BCK_KAITEN1, 2.0, EMODE_NONE, 1.0);
                self.mode = 3;
                self.speed_f = 0.0;
                out.se(SE_CV_PG_EBALL_FIRE_S);
            }
            return;
        }
        if self.mode == 3 {
            if frame == SWING_FRAME {
                out.se(SE_CM_PG_SWING_L);
            }
            self.ken_active = 1;
            self.ken_atp = KEN_ATP;
            if frame == i32::from(HIO_M3E) {
                self.sword_live = (HIO_M40 - HIO_M3E) as i8;
            }
            let (target, step) = if (SWING_FRAME - 1..=24).contains(&frame) {
                (SPIN_ACCEL, SPIN_STEP)
            } else {
                (0.0, SPIN_STEP)
            };
            add_calc2(&mut self.speed_f, target, 1.0, step);
            actor.angle[1] = self.shape_angle[1];
            actor.angle[0] = 0;
            self.pos_move(actor, 1);
            if self.morph.is_stop() {
                self.action = ACTION_KIERU;
                self.mode = 0;
            }
        }
    }

    fn spin_attack2(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        let frame = self.frame_i();
        if self.mode == 0 {
            self.shape_angle[1] = env
                .player_face
                .wrapping_add(SPIN_OFFSET.wrapping_mul(i16::from(self.clone)));
            self.mode = 1;
            if self.clone == 0 {
                for index in 1..=CLONE_COUNT {
                    out.clones.push((index as u32 * 16) | 3);
                }
            }
        }
        if self.mode == 1 {
            let seat = env.player_pos + turned(self.shape_angle[1], [0.0, HIO_M2C, HIO_M34]);
            actor.pos = seat;
            self.anm_init(BCK_KAITEN_S1, 1.0, EMODE_LOOP, 1.0);
            self.mode = 2;
            self.deru_brk(out);
            self.slot_pos = env.player_pos;
        }
        if self.mode == 2 {
            actor.angle[1] = self.shape_angle[1];
            self.speed_f = REG8_F4 + SPIN_SPEED_BASE;
            actor.angle[0] = 0;
            self.pos_move(actor, 1);
            if (self.slot_pos - actor.pos).length() < HIO_M38 {
                self.anm_init(BCK_KAITEN1, 2.0, EMODE_NONE, 1.0);
                self.mode = 3;
                self.speed_f = 0.0;
                out.se(SE_CV_PG_EBALL_FIRE_S);
            }
            return;
        }
        if self.mode == 3 {
            if frame == SWING_FRAME {
                out.se(SE_CM_PG_SWING_L);
            }
            self.ken_active = 1;
            self.ken_atp = KEN_ATP;
            if frame == i32::from(HIO_M3E) {
                self.sword_live = (HIO_M40 - HIO_M3E) as i8;
            }
            if self.morph.is_stop() {
                if self.clone == 0 {
                    self.mode = 4;
                    self.set_t(0, 40);
                    self.anm_init(BCK_WAIT1, 10.0, EMODE_LOOP, 1.0);
                } else {
                    self.action = ACTION_KIERU;
                    self.mode = 0;
                }
            }
            return;
        }
        if self.tv(0) == 0 {
            self.action = ACTION_KIERU;
            self.mode = 0;
        }
    }

    fn tama_set(&mut self, actor: &Actor) {
        if self.orb_stage == 0 {
            return;
        }
        let seat = turned(self.shape_angle[1], [0.0, -self.orb_lift, self.orb_lift]);
        let temp = Vec3::new(
            actor.pos.x + seat.x,
            seat.y + (ORB_LIFT_BASE + actor.pos.y + REG8_F4),
            actor.pos.z + seat.z,
        );
        self.orb_home = temp;
        match self.orb_stage {
            1 => {
                self.orb_alpha = ORB_ALPHA_START;
                self.orb_stage = 2;
            }
            2 => {}
            3 => self.orb_stage = 4,
            4 => {
                self.orb_alpha = self.orb_alpha.wrapping_sub(ORB_ALPHA_STEP);
                if self.orb_alpha == 0 {
                    self.orb_stage = 5;
                }
            }
            5..=8 => self.orb_stage += 1,
            9 => self.orb_stage = 0,
            _ => {}
        }
    }

    fn mahou_set(&mut self, out: &mut Sink<'_>) {
        out.orbs = true;
        self.orb_hits = 0;
    }

    fn shot2(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        add_calc_angle_s2(
            &mut self.shape_angle[1],
            bearing(actor.pos, env.player_eye),
            FACE_DIV,
            FACE_CAP,
        );
        let frame = self.frame_i();
        if self.mode == 0 {
            self.anm_init(BCK_TAME_S1, 5.0, EMODE_NONE, 1.0);
            out.se(SE_CV_PG_EBALL_MAKE_L);
            self.mode += 1;
        }
        if self.mode == 1 {
            out.se(SE_CM_PG_EBALL_MAKING_L);
            if self.morph.is_stop() {
                self.anm_init(BCK_TAME_S2, 1.0, EMODE_LOOP, 1.0);
                self.set_t(0, HIO_M3C);
                self.mode += 1;
            }
            if frame == 28 {
                self.orb_stage = 1;
                self.orb_lift = 0.0;
            }
        } else if self.mode == 2 {
            out.se(SE_CM_PG_EBALL_MAKING_L);
            if self.tv(0) != 0 {
                self.shot2_tail(actor, out);
                return;
            }
            self.anm_init(BCK_NAGERU_S1, 3.0, EMODE_NONE, 1.0);
            self.mode += 1;
        }
        let throwing = self.mode == 3;
        self.shot2_throw(frame, out);
        if !throwing && self.mode == 4 {
            if self.laugh_timer == 1 {
                self.anm_init(BCK_WAIT1, 10.0, EMODE_LOOP, 1.0);
            }
            if self.tv(0) == 0 {
                self.action = ACTION_FLY2;
                self.mode = 0;
            }
        }
        self.shot2_tail(actor, out);
    }

    fn shot2_throw(&mut self, frame: i32, out: &mut Sink<'_>) {
        if self.mode != 3 {
            return;
        }
        if frame == i32::from(REG8_S5) + SWING_FRAME {
            out.se(SE_CV_PG_EBALL_FIRE_L);
            out.se(SE_CM_PG_EBALL_FIRE_L);
        }
        if frame >= i32::from(REG8_S5) + SWING_FRAME {
            add_calc2(
                &mut self.orb_lift,
                REG8_F6 + ORB_LIFT_SPEED,
                1.0,
                REG8_F7 + ORB_LIFT_STEP,
            );
            if frame == i32::from(REG8_S6) + 16 {
                self.orb_stage = 3;
            }
            if frame == i32::from(REG8_S7) + 16 {
                self.mahou_set(out);
            }
        }
        if self.morph.is_stop() {
            self.set_t(0, 60);
            self.mode = 4;
        }
    }

    fn shot2_tail(&mut self, actor: &mut Actor, out: &mut Sink<'_>) {
        add_calc0(&mut self.speed_f, 1.0, REG0_F14 + FLY_BRAKE);
        self.pos_move(actor, 0);
        self.fly_se_set(actor, out);
        self.tama_set(actor);
    }

    fn down(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        if self.mode == 0 {
            self.anm_init(BCK_DAMAGE1, 2.0, EMODE_NONE, 1.0);
            self.mode += 1;
            self.speed.y = 0.0;
        }
        if self.mode == 1 {
            self.invuln = 5;
            if ground::grounded(env.solid, actor.pos) && self.morph.is_stop() {
                self.mode += 1;
                self.anm_init(BCK_TYAKUTI1, 2.0, EMODE_NONE, 1.0);
            }
            self.set_t(0, HIO_M48);
        } else if self.mode == 2 {
            if self.morph.is_stop() {
                self.mode += 1;
                self.anm_init(BCK_DOWN1, 2.0, EMODE_LOOP, 1.0);
            }
        } else if self.mode == 3 {
            if self.frame_i() == 2 {
                out.se(SE_CV_PG_TIRED);
            }
        } else if self.mode == MODE_DAMAGE {
            self.anm_init(BCK_DAMAGE2, 2.0, EMODE_NONE, 1.0);
            self.mode = 2;
        }
        actor.pos.y += self.speed.y;
        self.speed.y -= GRAVITY_DOWN;
        if self.speed.y < MAX_FALL_DOWN {
            self.speed.y = MAX_FALL_DOWN;
        }
        let landed = actor.pos.y <= env.landing(actor.pos);
        if self.tv(0) == 0 || landed {
            self.action = ACTION_KIERU;
            self.mode = 0;
            if self.kind != 0 {
                self.health = FALL_HEALTH;
            }
        }
    }

    fn kabe_check(&self, env: &Env<'_>) -> bool {
        let offset = [0.0, 0.0, HIO_M34 + 100.0];
        for index in 0..KABE_RAYS {
            let step = KABE_STEP.wrapping_mul(index as i16);
            let seat = env.player_pos + turned(step, offset);
            if env.blocked(env.player_pos, seat) {
                return true;
            }
        }
        false
    }

    fn deru(&mut self, env: &Env<'_>) {
        self.invuln = 3;
        if self.mode != 0 {
            return;
        }
        if HIO_M06 != 0 {
            match HIO_M06 {
                1 => {
                    self.action = ACTION_SHOT;
                    self.mode = MODE_ENTRY;
                    self.set_t(2, HIO_M0A);
                }
                2 => {
                    self.action = ACTION_SPIN;
                    self.mode = 0;
                }
                3 => {
                    self.action = ACTION_SHOT2;
                    self.mode = MODE_ENTRY;
                    self.set_t(2, HIO_M0A);
                }
                _ => {
                    self.action = ACTION_SPIN2;
                    self.mode = 0;
                }
            }
            return;
        }
        if self.kind == 0 {
            if self.rnd_f(1.0) < 0.5 {
                self.action = ACTION_SHOT;
                self.mode = MODE_ENTRY;
                self.set_t(2, HIO_M0A);
            } else {
                self.action = ACTION_SPIN;
                self.mode = 0;
            }
            return;
        }
        if self.rnd_f(1.0) < 0.4 && !self.kabe_check(env) {
            self.action = ACTION_SPIN2;
            self.mode = 0;
            return;
        }
        if self.rnd_f(1.0) < 0.5 {
            self.action = ACTION_SHOT2;
            self.mode = MODE_ENTRY;
            self.set_t(2, HIO_M0A);
        } else {
            self.action = ACTION_SHOT;
            self.mode = MODE_ENTRY;
            self.set_t(2, HIO_M0A);
        }
    }

    fn kieru(&mut self, out: &mut Sink<'_>) {
        self.invuln = 3;
        if self.mode == 0 {
            if self.ball_mode == 1 || self.ball_mode == 2 {
                self.ball_mode = 35;
            }
            self.kieru_brk(0, out);
            if self.clone != 0 {
                self.mode = 1;
                self.set_t(0, BANISH_FRAMES);
            } else {
                self.action = ACTION_STANDBY;
                self.mode = 0;
            }
            return;
        }
        if self.tv(0) == 0 {
            out.delete = true;
        }
    }

    fn fail(&mut self, env: &Env<'_>, out: &mut Sink<'_>) {
        self.invuln = 3;
        if self.mode == 0 {
            self.kieru_brk(1, out);
            self.mode = 1;
            self.anm_init(BCK_DAMAGE2, 2.0, EMODE_NONE, 1.0);
            return;
        }
        if self.mode == 1 {
            if !self.morph.is_stop() {
                return;
            }
            self.mode = 2;
            self.set_t(0, DROP_FRAMES);
            let anchor = Vec3::new(self.home_pos.x, self.home_pos.y + 100.0, self.home_pos.z);
            let far = anchor + turned(self.home_angle, [0.0, 0.0, SWORD_SPEED_SPAN as f32]);
            let (hit, cross) = env.crossed(anchor, far);
            let base = if hit { cross } else { anchor };
            let tip = self.home_pos + bind(JNT_ITEM1, self.home_angle);
            self.sword_yaw =
                self.shape_angle[1].wrapping_sub(to_s16((tip.x - base.x).atan2(tip.z - base.z)));
            self.sword_yaw = self.sword_yaw.wrapping_add(SWORD_YAW_OFFSET + REG0_S8);
            self.sword_step = REG0_S5 + SWORD_TURN_STEP;
            self.sword_timer = 100;
            return;
        }
        if self.mode == 2 {
            if self.tv(0) != 0 {
                return;
            }
            let step = self.sword_dangle.wrapping_add(self.sword_step);
            add_calc_angle_s2(&mut self.sword_roll, self.sword_yaw, SWORD_TURN_DIV, step);
            self.sword_tilt += self.sword_spin;
            self.sword_spin += self.sword_accel;
            self.sword_accel += REG12_F10 + SWORD_ACCEL_BASE;
            if self.sword_accel > SWORD_ACCEL_CAP + REG12_F3 {
                self.sword_accel = SWORD_ACCEL_CAP + REG12_F3;
            }
            if self.sword_tilt < SWORD_TILT_MAX {
                return;
            }
            self.sword_tilt = SWORD_TILT_MAX;
            self.sword_step = REG0_S6 + SWORD_CAP_BASE;
            if self.sword_spin > REG12_F1 + SWORD_SPIN_CAP {
                self.sword_spin *= -(REG12_F2 + SWORD_SPIN_GAIN);
                out.se(SE_OBJ_TN_SWORD_FALL);
                if self.sword_timer >= 30 {
                    self.sword_timer -= 20;
                }
                if self.sword_dangle == 0 {
                    self.sword_dangle = REG0_S1 + 30;
                }
            } else {
                self.sword_spin = 0.0;
                self.set_t(0, 0);
                self.mode = 3;
            }
            return;
        }
        if self.mode == 3 {
            if self.tv(0) == 0 {
                self.kieru_brk(2, out);
                self.set_t(0, DROP_HOLD);
                self.mode = 4;
                out.sword = Some(false);
            }
            return;
        }
        if self.tv(0) == 0 {
            out.delete = true;
            out.sword = Some(true);
        }
    }

    fn standby(&mut self, actor: &mut Actor, env: &Env<'_>) {
        self.invuln = 3;
        if self.mode == MODE_HIDE {
            if self.kind == 2 {
                if !env.has_light_arrow {
                    actor.pos.y = FAR_OFF;
                    return;
                }
            } else if self.kind == 1 && env.has_light_arrow {
                actor.pos.y = FAR_OFF;
                return;
            }
            let reach = f32::from(self.band) * ACTIVATION_SCALE;
            if flat(env.player_pos, self.home_pos) < reach {
                self.set_t(0, HIO_M08);
                self.mode = 1;
                self.bgm_delay = 22;
            }
            return;
        }
        if self.mode == 0 {
            self.set_t(0, HIO_M08);
            self.mode += 1;
        }
        if self.tv(0) == 0 {
            self.action = ACTION_DERU;
            self.mode = 0;
            self.sword_accel = 0.0;
            self.sword_spin = 0.0;
            self.sword_tilt = 0.0;
        }
    }

    fn start(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        self.invuln = 3;
        if self.mode == 0 {
            let reach = f32::from(self.band) * ACTIVATION_SCALE;
            if flat(env.player_pos, self.home_pos) < reach {
                self.demo_state = 1;
                self.mode = 1;
                self.anm_init(BCK_WAIT1, 0.0, EMODE_LOOP, 1.0);
            }
            return;
        }
        if self.mode == 1 {
            self.shape_angle[1] = bearing(actor.pos, env.player_pos);
            return;
        }
        if self.mode == 2 {
            self.anm_init(BCK_WARAU1, 5.0, EMODE_LOOP, 1.0);
            out.se(SE_CV_PG_LAUGH_2);
            self.mode = 3;
            return;
        }
        if self.mode != 4 && self.mode != 5 {
            return;
        }
        if self.mode == 4 {
            self.mode = 5;
            self.set_t(0, REG17_S0.wrapping_add(0x50));
            self.anm_init(BCK_TAME1, 10.0, EMODE_LOOP, 1.0);
        }
        if self.tv(0) == REG17_S0.wrapping_add(0x4B) {
            self.ball_mode = 1;
            out.bgm(BGM_P_GANON_1);
        }
        if self.tv(0) <= REG17_S0.wrapping_add(0x4B) {
            out.se(SE_CM_PG_EBALL_MAKING_S);
        }
        let step = turned(
            self.shape_angle[1],
            [0.0, self.speed_f * 0.5, -self.speed_f],
        );
        self.speed = step;
        actor.pos += self.speed;
        let target = if self.tv(0) == 0 {
            0.0
        } else {
            REG0_F7 + SPIN_TARGET
        };
        add_calc2(&mut self.speed_f, target, 1.0, REG0_F5 + SPIN_SPEED_STEP);
    }

    fn end(&mut self, actor: &mut Actor, out: &mut Sink<'_>) {
        self.invuln = 3;
        let frame = self.frame_i();
        match self.mode {
            0 => {
                self.mode = 1;
                self.anm_init(BCK_LAST_DAMAGE1, 2.0, EMODE_NONE, 1.0);
                self.demo_state = 50;
                out.story(STORY_DEFEATED);
            }
            1 => {
                if frame == 104 {
                    self.kieru_brk(0, out);
                }
            }
            2 => {
                self.deru_brk(out);
                self.anm_init(BCK_WARAU1, 1.0, EMODE_LOOP, 1.0);
                out.se(SE_CV_PG_LAUGH_2);
                self.mode = 3;
            }
            4 => {
                self.deru_brk(out);
                self.anm_init(BCK_SARU1, 1.0, EMODE_NONE, 1.0);
                self.mode = 5;
                self.speed.y = 0.0;
                out.se(SE_CM_PG_END_1);
            }
            _ => {}
        }
        if self.mode == 5 {
            actor.pos.y += self.speed.y;
            self.speed.y += REG8_F8 + RISE_GRAVITY;
            if self.speed.y > REG8_F9 + MAX_RISE {
                self.speed.y = REG8_F9 + MAX_RISE;
            }
            if frame == 28 {
                self.kieru_brk2(out);
            }
            if frame == 48 {
                self.mode = 6;
            }
        }
    }

    fn last_end(&mut self, actor: &mut Actor, out: &mut Sink<'_>) {
        self.invuln = 3;
        if self.mode == 0 {
            self.mode = 1;
            self.anm_init(BCK_MOGAKU1, 2.0, EMODE_NONE, 1.0);
            self.demo_state = 100;
            self.fog_end = LAST_FOG_START;
            self.set_t(0, REG0_S4 + LAST_HOLD_A);
            self.set_t(1, LAST_HOLD_B);
        }
        if self.mode == 1 {
            if self.tv(1) == 0 {
                self.mode = 2;
                self.set_t(1, LAST_HOLD_C);
                self.ken_held = 1;
                self.demo_state += 1;
            }
        } else if self.mode == 2 {
            add_calc2(&mut self.scale.x, LAST_SCALE, 1.0, MOVE_SCALE);
            add_calc2(&mut self.scale.z, LAST_SCALE, 1.0, MOVE_SCALE);
            add_calc2(&mut self.scale.y, LAST_SCALE, 1.0, MOVE_SCALE);
            if self.tv(1) == 0 {
                self.mode = 3;
                self.set_t(1, LAST_HOLD_D);
                out.se(SE_CM_L_ARROW_PASS_AWAY);
            }
        } else if self.mode == 3 {
            add_calc0(&mut self.scale.x, LAST_SCALE, MOVE_SCALE);
            add_calc0(&mut self.scale.z, LAST_SCALE, MOVE_SCALE);
            add_calc2(&mut self.scale.y, LAST_RISE, 0.1, 1.0);
            if self.tv(1) == 0 {
                self.scale = Vec3::ZERO;
                self.mode = 4;
                self.set_t(1, LAST_HOLD_E);
            }
        } else if self.mode == 4 {
            if self.tv(1) <= 4 {
                if self.tv(1) == 4 {
                    out.sword = Some(false);
                }
                self.ken_held = 2;
                if self.tv(1) == 1 {
                    self.mode = 5;
                    self.demo_state += 1;
                    self.demo_count = 0;
                }
            }
        }
        if self.tv(0) <= 1 {
            if self.tv(0) == 1 {
                self.light_shot = 1;
                self.fog_end = LAST_FOG_END;
            }
            add_calc2(
                &mut self.fog_end,
                LAST_FOG_SPEED,
                1.0,
                REG0_F4 + LAST_FOG_SCALE,
            );
        }
        add_calc_angle_s2(&mut actor.angle[0], 0, TURN_PITCH_DIV, TURN_PITCH_CAP);
    }

    fn cyl_hits(&self, from: Vec3) -> bool {
        let dx = from.x - self.body.x;
        let dz = from.z - self.body.z;
        let dy = from.y - self.body.y;
        dx * dx + dz * dz <= self.cyl_radius * self.cyl_radius && dy.abs() <= self.cyl_height * 0.5
    }

    fn damage_check(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        if self.invuln != 0 {
            return;
        }
        let strike = env.strike_pos();
        let tg_hit = self.body_tg && strike.is_some_and(|at| self.cyl_hits(at));
        if tg_hit {
            if self.kind == 2 {
                if env.light_touch {
                    self.light_arrow_hit(out);
                } else {
                    self.action = ACTION_KIERU;
                    self.mode = 0;
                    if self.ball_mode == 1 || self.ball_mode == 2 {
                        self.ball_mode = 35;
                    }
                }
                self.orb_stage = 0;
                return;
            }
            if env.orb_touch {
                self.orb_hit(actor, env, out);
                return;
            }
        }
        if self.clone != 0 {
            if tg_hit || env.master_action == ACTION_DOWN {
                out.delete = true;
            }
            return;
        }
        if tg_hit || self.ball_landed != 0 {
            self.invuln = 6;
            if tg_hit {
                if self.action == ACTION_DOWN
                    || self.action == ACTION_SPIN
                    || self.action == ACTION_SPIN2
                {
                    if self.sword_touch(env) {
                        self.sword_branch(env, out);
                    }
                }
            } else {
                out.se(SE_CV_PG_HIT_EBALL);
                self.knock_back(actor, env);
            }
        }
    }

    fn sword_touch(&self, env: &Env<'_>) -> bool {
        env.blow.at == At::Sword
            && damage::is_master_sword(env.blow.blade)
            && within(self.body, env.player_eye, self.cyl_radius + REACH_SWORD)
    }

    fn light_arrow_hit(&mut self, out: &mut Sink<'_>) {
        out.se(SE_CV_PG_DIE);
        out.se(SE_CM_PG_END_2);
        self.action = ACTION_LAST_END;
        self.mode = 0;
        self.invuln = 1000;
        out.stop_bgm();
        if self.ball_mode != 0 {
            self.ball_mode = 35;
        }
        out.se(SE_CM_L_ARROW_SHRINK);
        self.orb_stage = 0;
        self.speed_f = 0.0;
    }

    fn orb_hit(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        self.orb_hits += 1;
        self.anm_init(BCK_DAMAGE1, 2.0, EMODE_NONE, 1.0);
        self.set_t(0, 60);
        out.se(SE_CV_PG_HIT_EBALL);
        if self.orb_hits < ORB_HIT_LIMIT {
            return;
        }
        self.knock_back(actor, env);
        self.speed_f = 0.0;
    }

    fn knock_back(&mut self, actor: &Actor, env: &Env<'_>) {
        self.action = ACTION_DOWN;
        self.mode = 0;
        self.push_xz = REG8_F11 + PUSH_RATE;
        self.set_t(2, REG8_S2 + PUSH_FRAMES);
        let delta = actor.pos - env.player_pos;
        self.push_yaw = to_s16(delta.x.atan2(delta.z));
        let span = (delta.x * delta.x + delta.z * delta.z).sqrt();
        self.push_pitch = to_s16(delta.y.atan2(span)).wrapping_neg();
    }

    fn sword_branch(&mut self, env: &Env<'_>, out: &mut Sink<'_>) {
        let swung =
            self.action == ACTION_DOWN || self.action == ACTION_SPIN || self.action == ACTION_SPIN2;
        if !swung {
            return;
        }
        let power = env.blow.at.power(env.blow.blade, env.blow.soup);
        if power > 0 {
            self.health -= power;
        }
        if self.health <= 0 {
            out.se(SE_CV_PG_DAMAGE_L);
        } else {
            out.se(SE_CV_PG_DAMAGE_S);
        }
        if self.action == ACTION_SPIN || self.action == ACTION_SPIN2 {
            self.action = ACTION_DOWN;
            self.mode = 0;
        } else if self.kind == 0 {
            if self.health <= 0 {
                self.action = ACTION_END;
                self.mode = 0;
                self.invuln = 1000;
                out.stop_bgm();
            } else {
                self.combo_left = self.combo_left.wrapping_sub(power as i8);
                if self.combo_left <= 0 {
                    self.combo_left = COMBO_LEFT;
                    self.action = ACTION_KIERU;
                    self.mode = 0;
                } else {
                    self.mode = MODE_DAMAGE;
                }
            }
        } else if self.kind == 1 {
            self.action = ACTION_FAIL;
            self.mode = 0;
            self.health = 0;
            out.stop_bgm();
        }
    }

    fn ball_bg_check(&mut self, env: &Env<'_>) -> bool {
        let probe = self.ball_pos + (self.ball_pos - self.ball_prev) * BALL_PREV_SCALE;
        let (hit, cross) = env.crossed(self.ball_pos, probe);
        if hit {
            self.ball_pos = cross;
        }
        hit
    }

    fn energy_ball_move(&mut self, actor: &Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        self.ball_tg_set = false;
        self.ball_hit_player = false;
        if self.ball_mode == 0 {
            return;
        }
        if self.ball_mode == 35 {
            if self.tension != 0 {
                self.tension = 0;
            }
            self.ball_mode = 0;
            return;
        }
        if self.ball_mode == 1 {
            self.ball_mode = 2;
        }
        if self.ball_mode == 2 {
            self.ball_pos = actor.pos
                + self.model_offset
                + bind(JNT_UDE_L3, self.shape_angle[1])
                + turned(self.shape_angle[1], [BALL_SPAWN_X, BALL_SPAWN_Y, 0.0]);
            if self.ball_fire != 0 {
                self.ball_fire = 0;
                self.ball_mode = 3;
                let mut delta = env.player_eye - self.ball_pos;
                delta.y -= REG0_F18 + BALL_PROBE_AIM;
                let (yaw, pitch) = aim_of(delta);
                let reach = if self.kind == 0 { HIO_M1C } else { HIO_M20 };
                self.ball_speed = reach;
                self.ball_step = aimed(yaw, pitch, reach);
                self.ball_pause = BALL_PAUSE_FIRE;
                self.ball_live = false;
            }
        }
        if self.ball_mode >= 3 {
            self.ball_prev = self.ball_pos;
            self.ball_pos += self.ball_step;
            let mut landed = false;
            let mut delta = self.eye - self.ball_pos;
            delta.y -= BALL_PROBE_AIM;
            if delta.length() < self.ball_speed + BALL_GAP + REG0_F3 {
                if self.swing_hit != 0 {
                    landed = true;
                    self.ball_landed = 1;
                } else if self.ball_mode == 5 {
                    if self.kind != 2 {
                        self.ball_landed = 1;
                    } else {
                        if self.ball_pause != 0 {
                            return;
                        }
                        self.action = ACTION_KIERU;
                        self.mode = 0;
                        self.ball_pause = BALL_PAUSE_BREAK;
                        return;
                    }
                }
            }
            let mut bottle = false;
            if self.ball_mode == 4 && env.player_bottle {
                let mut probe = env.player_eye - self.ball_pos;
                probe.y -= BALL_PROBE_SWING;
                if probe.length() < REG0_F2 + BALL_RANGE_BASE {
                    bottle = true;
                }
            }
            if (landed || bottle) && self.ball_pause == 0 {
                let sword = env.blow.at == At::Sword
                    && damage::is_master_sword(env.blow.blade)
                    && within(
                        self.ball_pos,
                        env.player_eye,
                        self.ball_tg_radius + REACH_SWORD,
                    );
                let mut bounced = false;
                if sword || bottle {
                    let mut aim = self.eye - self.ball_pos;
                    aim.y -= REG0_F17 + BALL_PROBE_SWING;
                    let (yaw, pitch) = aim_of(aim);
                    self.ball_step = aimed(yaw, pitch, self.ball_speed);
                    self.ball_mode = 5;
                    self.tension += 1;
                    out.se(SE_LK_PG_BOMB_STRIKE);
                    out.story(STORY_DEFEATED);
                    bounced = true;
                } else if landed {
                    let mut aim = env.player_eye - self.ball_pos;
                    aim.y -= REG0_F18 + BALL_PROBE_AIM;
                    let (yaw, pitch) = aim_of(aim);
                    self.ball_step = aimed(yaw, pitch, self.ball_speed);
                    self.ball_mode = 4;
                    bounced = true;
                }
                if bounced {
                    if self.kind == 0 {
                        self.ball_speed += HIO_M24;
                    } else {
                        self.ball_speed += HIO_M28;
                    }
                    self.ball_pos += self.ball_step;
                    self.ball_pause = BALL_PAUSE_FIRE;
                }
            }
            let mut live = self.ball_pos;
            if self.ball_pause != 0 {
                self.ball_pause -= 1;
                live = Vec3::new(FAR_OFF, FAR_OFF, FAR_OFF_UP);
            }
            if self.ball_mode == 3 {
                self.ball_live = true;
                self.ball_mode = 4;
            }
            self.ball_at_radius = REG0_F6 + BALL_AT_RADIUS;
            self.ball_tg_radius = REG0_F7 + BALL_TG_RADIUS;
            self.ball_hit_player = within(live, env.player_eye, self.ball_at_radius + PLAYER_BODY);
            if self.ball_landed != 0 || self.ball_bg_check(env) {
                if self.tension != 0 {
                    self.tension = 0;
                }
                self.ball_mode = 0;
                out.se(SE_OBJ_PG_EBALL_EXP_S);
                if self.ball_hit_player && self.action != ACTION_LAST_END {
                    self.ball_struck_player(out);
                }
                self.ball_tg_set = false;
                return;
            }
            self.ball_tg_set = true;
        }
        if (self.home_pos - self.ball_pos).length() > BALL_LIMIT {
            self.ball_mode = 35;
        }
    }

    fn ball_struck_player(&mut self, out: &mut Sink<'_>) {
        out.health.damage(BALL_POWER);
        self.hurt_gauge = 1;
        self.action = ACTION_FLY;
        self.mode = 1;
        let span = (self.rnd_f(30.0) + 70.0) as i16;
        self.set_t(1, span);
    }

    fn demo_camera(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        'phase: {
            if self.demo_state == 1 {
                self.demo_state += 1;
                self.demo_count = 0;
                self.cam_fov = DEMO_FOV;
                self.cam_rate = 0.0;
            }
            if self.demo_state == 2 {
                if self.demo_count == DEMO_INTRO_LAUGH {
                    out.se(SE_CV_PG_LAUGH_1);
                }
                if self.demo_count == DEMO_INTRO_ENTER {
                    self.deru_brk2(out);
                }
                if self.demo_count != DEMO_INTRO {
                    break 'phase;
                }
                self.demo_state = 3;
                self.demo_count = 0;
                actor.pos = Vec3::new(REG8_F0 - 300294.0, REG8_F1 + 745.0, REG8_F2 - 303109.0);
                self.cam_eye = Vec3::new(REG8_F3 - 300169.0, REG8_F4 + 770.0, REG8_F5 - 303635.0);
                self.cam_look =
                    Vec3::new(actor.pos.x, actor.pos.y + (REG0_F0 + 160.0), actor.pos.z);
            }
            if self.demo_state == 3 {
                if self.demo_count > DEMO_INTRO_HOLD {
                    add_calc2(&mut self.cam_eye.x, -300269.0, 0.1, self.cam_rate * 100.0);
                    add_calc2(&mut self.cam_eye.y, 870.0, 0.1, self.cam_rate * 100.0);
                    add_calc2(&mut self.cam_eye.z, -303335.0, 0.1, self.cam_rate * 300.0);
                    self.cam_look.x = actor.pos.x;
                    self.cam_look.z = actor.pos.z;
                    add_calc2(
                        &mut self.cam_look.y,
                        ((actor.pos.y + 230.0) - 70.0) + 55.0,
                        0.1,
                        self.cam_rate * 55.0,
                    );
                    add_calc2(&mut self.cam_rate, 0.1, 1.0, REG8_F7 + 0.01);
                }
                if self.demo_count == DEMO_WAIT_MODE {
                    self.mode = 2;
                }
                if self.demo_count == DEMO_TAME_MODE {
                    self.mode = 4;
                }
                if self.demo_count == DEMO_INTRO_END {
                    self.action = ACTION_SHOT;
                    self.mode = 1;
                    self.set_t(0, DEMO_BALL_FRAME);
                    self.demo_state = 150;
                    if self.off_switch != NO_SWITCH {
                        out.switches.set(self.off_switch);
                    }
                }
                break 'phase;
            }
            if self.demo_state == 50 {
                self.demo_state += 1;
                self.demo_count = 0;
                self.cam_fov = DEMO_STAND_FOV;
                self.cam_rate = 0.0;
                self.shape_angle[1] = 0;
            }
            if self.demo_state == 51 {
                actor.pos = Vec3::new(-300202.0, 715.0, -301859.0);
                self.cam_eye = actor.pos
                    + turned(
                        self.shape_angle[1],
                        [REG6_F0 - 150.0, REG6_F1 + 20.0, REG6_F2 + 500.0],
                    );
                self.cam_look = self.eye;
                self.cam_look.y += REG6_F6 - 40.0;
                if self.demo_count > DEMO_STAND_HOLD {
                    self.demo_state += 1;
                    self.demo_count = 0;
                    self.cam_look = Vec3::new(-299898.0, 1188.0, -301158.0);
                    self.cam_eye = Vec3::new(-299703.0, 921.0, -300851.0);
                }
                break 'phase;
            }
            if self.demo_state == 52 {
                if self.demo_count == DEMO_STAND_MODE {
                    self.mode += 1;
                }
                actor.pos = Vec3::new(
                    (self.cam_look.x - 50.0) + REG6_F4,
                    self.cam_look.y + REG6_F5,
                    (self.cam_look.z - 100.0) + REG6_F6,
                );
                self.shape_angle[1] = REG6_S0 + 2000;
                if self.demo_count == DEMO_STAND_END {
                    self.demo_state += 1;
                    self.demo_count = 0;
                }
                break 'phase;
            }
            if self.demo_state == 53 {
                self.cam_look = Vec3::new(-300098.0, 580.0, -301997.0);
                self.cam_eye = Vec3::new(-300274.0, 929.0, -301770.0);
                if self.demo_count != DEMO_CAPE_DROP {
                    break 'phase;
                }
                self.demo_state += 1;
                self.demo_count = 0;
                self.cam_eye = Vec3::new(-299703.0, 921.0, -300851.0);
                self.speed.y = 0.0;
                self.cam_look = self.eye;
                self.cam_look.y -= REG0_F11 + 30.0;
                self.mode += 1;
            }
            if self.demo_state == 54 {
                add_calc2(
                    &mut self.cam_look.y,
                    (self.eye.y - DEMO_CAPE_LIFT) + REG0_F11,
                    0.1,
                    20.0,
                );
                if self.demo_count == DEMO_CAP_SWING {
                    self.demo_state = 55;
                    self.demo_count = 0;
                }
            }
            if self.demo_state == 55 {
                if self.demo_count == DEMO_CAP_LAUGH {
                    out.se(SE_CV_PG_LAUGH_1);
                }
                self.cam_look = env.player_pos;
                self.cam_look.y += REG8_F3;
                let yaw = env
                    .player_face
                    .wrapping_add(self.cam_swing)
                    .wrapping_add(REG8_S5);
                self.cam_swing += REG8_S6 + 30;
                self.cam_eye =
                    env.player_pos + turned(yaw, [0.0, REG8_F4 + 1500.0, REG8_F5 + 2000.0]);
                if self.demo_count != DEMO_CAPE_END {
                    break 'phase;
                }
                if self.switch_no != NO_SWITCH {
                    out.switches.set(self.switch_no);
                }
                if self.off_switch != NO_SWITCH {
                    out.switches.clear(self.off_switch);
                }
                self.demo_state = 150;
                out.delete = true;
            }
            if self.demo_state == 100 {
                self.demo_state += 1;
                self.demo_count = 0;
                self.cam_fov = DEMO_FOV;
                self.cam_rate = 0.0;
                self.shape_angle[1] = bearing(actor.pos, env.player_pos);
            }
            if self.demo_state == 101 {
                add_calc2(&mut self.cam_look.x, actor.pos.x, 0.1, 200.0);
                add_calc2(&mut self.cam_look.y, self.eye.y - 50.0, 0.1, 200.0);
                add_calc2(&mut self.cam_look.z, actor.pos.z, 0.1, 200.0);
                let seat = actor.pos
                    + turned(self.shape_angle[1], [0.0, REG8_F0 - 100.0, REG8_F1 + 400.0]);
                let (hit, mut look) = env.crossed(self.cam_look, seat);
                if hit {
                    look.y += REG8_F18 + 20.0;
                }
                add_calc2(&mut self.cam_eye.x, look.x, 0.1, self.cam_rate * 50.0);
                add_calc2(&mut self.cam_eye.y, look.y, 0.1, self.cam_rate * 50.0);
                add_calc2(&mut self.cam_eye.z, look.z, 0.1, self.cam_rate * 50.0);
                add_calc2(&mut self.cam_rate, 1.0, 1.0, REG8_F7 + 0.1);
                break 'phase;
            }
            if self.demo_state == 103 {
                if self.demo_count > REG17_S2.wrapping_add(8) {
                    add_calc2(
                        &mut self.cam_look.y,
                        env.player_pos.y + REG0_F9,
                        0.8,
                        REG0_F10 + 30.0,
                    );
                }
                if self.demo_count <= REG17_S3.wrapping_add(80) {
                    break 'phase;
                }
                self.demo_state = 150;
                out.delete = true;
                if REG17_S3 == 0 && self.switch_no != NO_SWITCH {
                    out.switches.set(self.switch_no);
                }
            }
            if self.demo_state == 150 {
                self.demo_state = 0;
            }
        }
        if self.demo_state != 0 {
            self.demo_count += 1;
        }
    }

    fn sway_step(&mut self) -> Vec3 {
        let x = self.sway * sin_s(self.tick.wrapping_mul(SWAY_YAW));
        let y = self.sway * sin_s(self.tick.wrapping_mul(SWAY_PITCH));
        let z = self.sway * from_s16(self.tick.wrapping_mul(SWAY_ROLL)).cos();
        add_calc0(&mut self.sway, 1.0, 1.0);
        Vec3::new(x, y, z)
    }

    fn model_pose(&mut self, actor: &Actor) {
        self.eye = actor.pos + self.model_offset + bind(JNT_AGO1, self.shape_angle[1]);
        self.body = actor.pos + self.model_offset + bind(JNT_MUNE2, self.shape_angle[1])
            - turned(self.shape_angle[1], [0.0, MUNE2_DROP, 0.0]);
        self.cyl_radius = REG8_F4 + CYL_RADIUS;
        self.cyl_height = REG8_F3 + CYL_HEIGHT;
        self.ken_radius = REG0_F13 + KEN_RADIUS;
        if self.ken_active != 0 {
            self.ken = actor.pos + self.model_offset + bind(JNT_ITEM1_TIP, self.shape_angle[1]);
            self.ken_active = 0;
            if self.ken_at_live == 0 {
                self.ken_at_live = 1;
            }
        } else {
            self.ken_at_live = 0;
            self.ken = Vec3::new(FAR_OFF_LOW, FAR_OFF_LOW, FAR_OFF_LOW);
        }
    }

    fn execute(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        self.old_pos = actor.pos;
        if self.bgm_delay >= 2 {
            self.bgm_delay -= 1;
            if self.bgm_delay == 1 {
                out.bgm(BGM_P_GANON_2);
            }
        }
        let laughable = self.action == ACTION_FLY
            || self.action == ACTION_FLY2
            || self.action == ACTION_SPIN
            || self.action == ACTION_SPIN2;
        if laughable && self.hurt_gauge != 0 && self.laugh_timer == 0 {
            self.hurt_gauge = 0;
            self.anm_init(BCK_WARAU1, 6.0, EMODE_LOOP, 1.0);
            self.laugh_timer = 60;
            out.se(SE_CV_PG_LAUGH_3);
        }
        self.tick = self.tick.wrapping_add(1);
        self.tick_timers();
        timer8(&mut self.laugh_timer);
        timer(&mut self.invuln);
        if self.shake != 0 {
            self.shake -= 1;
        }
        self.lock_on = HIO_M05 == 0 && self.materialized;
        let moved = self.move_state(actor, env, out);
        if moved {
            self.correct_pos(actor, env);
        }
        self.morph.play();
        self.model_offset = self.sway_step();
        self.model_pose(actor);
        self.demo_camera(actor, env, out);
        self.energy_ball_move(actor, env, out);
        self.body_live = self.materialized && self.clone == 0;
        self.body_tg = self.materialized;
        self.swing_hit = 0;
        timer8(&mut self.sword_live);
    }

    fn correct_pos(&mut self, actor: &Actor, env: &Env<'_>) {
        self.grounded = ground::grounded(env.solid, actor.pos);
    }

    fn move_state(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) -> bool {
        let mut res = false;
        if self.kind == 0 && self.action < 20 {
            if self.action == ACTION_STANDBY && env.player_pos.y < PLAYER_GATE {
                return false;
            }
            if self.action != ACTION_KIERU && env.player_pos.y < PLAYER_GATE {
                self.action = ACTION_KIERU;
                self.mode = 0;
            }
        }
        match self.action {
            ACTION_STANDBY => self.standby(actor, env),
            ACTION_DERU => self.deru(env),
            ACTION_KIERU => self.kieru(out),
            ACTION_FAIL => {
                self.fail(env, out);
                res = true;
            }
            ACTION_FLY => {
                self.fly(actor, env, out, false);
                res = true;
            }
            ACTION_SHOT => {
                self.shot(actor, env, out);
                res = true;
            }
            ACTION_SPIN => {
                self.spin_attack(actor, env, out);
                res = true;
            }
            ACTION_FLY2 => {
                self.fly(actor, env, out, true);
                res = true;
            }
            ACTION_SHOT2 => {
                self.shot2(actor, env, out);
                res = true;
            }
            ACTION_SPIN2 => self.spin_attack2(actor, env, out),
            ACTION_DOWN => {
                self.down(actor, env, out);
                res = true;
            }
            ACTION_START => self.start(actor, env, out),
            ACTION_END => self.end(actor, out),
            ACTION_LAST_END => self.last_end(actor, out),
            _ => {}
        }
        self.damage_check(actor, env, out);
        self.ball_landed = 0;
        if self.push_xz > PUSH_GATE {
            let step = turned(self.push_yaw, [0.0, 0.0, self.push_xz]);
            let step = pitched(self.push_pitch, step);
            actor.pos += step;
            add_calc0(&mut self.push_xz, 1.0, PUSH_STEP);
        }
        res
    }

    fn begin(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        self.entered = true;
        self.home_pos = actor.pos;
        self.home_angle = actor.angle[1];
        self.old_pos = actor.pos;
        self.eye = actor.pos;
        self.body = actor.pos + Vec3::new(0.0, BODY_LIFT, 0.0);
        self.shape_angle = actor.angle;
        if self.kind == 3 && self.clone != 0 {
            self.action = ACTION_SPIN2;
            self.mode = 0;
            self.deru_brk(out);
            return;
        }
        if self.clone == 0 {
            if self.kind == 0 {
                self.action = ACTION_START;
                self.mode = 0;
                self.kieru_brk(0, out);
                self.max_health = MASTER_HEALTH;
                self.health = MASTER_HEALTH;
                self.combo_left = COMBO_LEFT;
            } else {
                self.max_health = STAGE_HEALTH;
                self.health = STAGE_HEALTH;
                self.action = ACTION_STANDBY;
                self.mode = MODE_HIDE;
            }
            actor.pos.y += SPAWN_LIFT;
        } else {
            self.action = ACTION_SPIN2;
            self.mode = 0;
            self.deru_brk(out);
        }
        self.execute(actor, env, out);
    }
}

fn aim_of(delta: Vec3) -> (i16, i16) {
    let yaw = to_s16(delta.x.atan2(delta.z));
    let pitch = to_s16(
        delta
            .y
            .atan2((delta.x * delta.x + delta.z * delta.z).sqrt()),
    )
    .wrapping_neg();
    (yaw, pitch)
}

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
    } else if watch
        .chains
        .iter()
        .any(|chain| chain.phase != hookshot::Phase::Wait)
    {
        at = At::Hookshot;
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
    equipped: Res<'w, inventory::Equipped>,
    collect: Res<'w, Collect>,
    swim: Res<'w, Swim>,
    ships: Query<'w, 's, &'static Ship>,
    players: Query<'w, 's, (Entity, &'static Player)>,
    shafts: Query<'w, 's, &'static arrow::Arrow>,
    bolts: Query<'w, 's, &'static boomerang::Boomerang>,
    chains: Query<'w, 's, &'static hookshot::Hookshot>,
    others: Query<'w, 's, (Entity, &'static Actor), Without<PhantomGanon>>,
}

pub fn control(
    watch: Watch<'_, '_>,
    mut switches: ResMut<Switches>,
    mut stories: ResMut<StoryBits>,
    mut zones: ResMut<ZoneActors>,
    mut health: ResMut<Health>,
    mut events: ResMut<SimEvents>,
    mut commands: Commands<'_, '_>,
    mut bosses: Query<
        (Entity, &mut Actor, &mut PhantomGanon),
        (With<PhantomGanon>, Without<Player>),
    >,
) {
    let solid = watch.collision.as_deref();
    let blow = blow_of(&watch);
    let has_light_arrow =
        inventory::check_get_item(&watch.equipped, &watch.collect, ITEM_LIGHT_ARROW);
    let riding = watch.ships.iter().any(|ship| ship.riding);
    let swimming = watch.swim.active;
    let Some(player) = watch.players.iter().next().map(|(_, player)| *player) else {
        return;
    };
    let master_action = bosses
        .iter()
        .find(|(_, _, boss)| boss.clone == 0)
        .map(|(_, _, boss)| boss.action)
        .unwrap_or(ACTION_STANDBY);
    let orbs: Vec<Vec3> = watch
        .others
        .iter()
        .filter(|(_, other)| other.name == ORB_ACTOR)
        .map(|(_, other)| other.pos)
        .collect();
    let shafts: Vec<(u8, arrow::Phase, Vec3)> = watch
        .shafts
        .iter()
        .map(|shaft| (shaft.kind, shaft.phase, shaft.pos))
        .collect();
    let mut orbs_made: Vec<(Vec3, i8, u16)> = Vec::new();
    let mut clones: Vec<(Vec3, i16, i8, u32)> = Vec::new();
    let mut swords: Vec<(Vec3, bool, i8, u16)> = Vec::new();
    let mut gone: Vec<(Entity, i8, u16)> = Vec::new();
    let mut sword_cut: Vec<(u8, Vec3, f32)> = Vec::new();
    let player_eye = player.pos + Vec3::new(0.0, PLAYER_EYE_Y, 0.0);
    for (entity, mut actor, mut boss) in &mut bosses {
        let actor = &mut *actor;
        let boss = &mut *boss;
        let arrow_pos = shafts
            .iter()
            .find(|(_, phase, _)| *phase == arrow::Phase::Flying)
            .map(|(_, _, pos)| *pos);
        let light_touch = shafts.iter().any(|(kind, phase, pos)| {
            *kind == arrow::TYPE_LIGHT && *phase == arrow::Phase::Flying && boss.cyl_hits(*pos)
        });
        let orb_touch = orbs.iter().any(|pos| boss.cyl_hits(*pos));
        let env = Env {
            player_pos: player.pos,
            player_eye,
            player_face: to_s16(player.face),
            player_ride: riding,
            player_swim: swimming,
            player_bottle: false,
            solid,
            blow,
            has_light_arrow,
            light_touch,
            orb_touch,
            arrow_pos,
            master_action,
        };
        let room = actor.room;
        let set_id = actor.set_id;
        let mut sink = Sink {
            events: &mut events,
            stories: &mut stories,
            switches: &mut switches,
            health: &mut health,
            delete: false,
            orbs: false,
            clones: Vec::new(),
            sword: None,
        };
        if !boss.entered {
            boss.begin(actor, &env, &mut sink);
        }
        boss.execute(actor, &env, &mut sink);
        if boss.ken_at_live != 0 && boss.ken_atp != 0 {
            sword_cut.push((boss.ken_atp as u8, boss.ken, boss.ken_radius));
        }
        if sink.orbs {
            orbs_made.push((boss.orb_home, room, set_id));
        }
        for parameters in sink.clones {
            clones.push((actor.pos, actor.angle[1], room, parameters));
        }
        if let Some(dropped) = sink.sword {
            swords.push((actor.pos, dropped, room, set_id));
        }
        if sink.delete {
            if boss.bgm_delay != 0 {
                events.list.push(SimEvent::Bgm(0));
            }
            gone.push((entity, room, set_id));
        }
    }
    for (power, centre, radius) in sword_cut {
        if within(centre, player_eye, radius + PLAYER_BODY) {
            health.damage(power);
        }
    }
    for (pos, room, set_id) in orbs_made {
        for orb in 0..ORB_COUNT {
            let actor = Actor {
                name: ORB_ACTOR.to_owned(),
                parameters: orb as u32,
                pos,
                angle: [0; 3],
                set_id,
                room,
                ..Default::default()
            };
            commands.spawn((actor, fgmahou::FgMahou::spawn(orb as u32)));
        }
    }
    for (pos, yaw, room, parameters) in clones {
        let actor = Actor {
            name: PHANTOM_GANON_ACTOR.to_owned(),
            parameters,
            pos,
            angle: [0, yaw, 0],
            set_id: 0,
            room,
            ..Default::default()
        };
        commands.spawn((actor, PhantomGanon::spawn(parameters)));
    }
    for (pos, dropped, room, set_id) in swords {
        let actor = Actor {
            name: boko::BOKO_ACTORS[0].to_owned(),
            parameters: BOKO_SWORD,
            pos,
            angle: [0; 3],
            set_id,
            room,
            ..Default::default()
        };
        let Some(mut stick) = boko::Boko::spawn(BOKO_SWORD) else {
            continue;
        };
        stick.floor_flag = dropped;
        commands.spawn((actor, stick));
    }
    for (entity, room, set_id) in gone {
        zones.mark_done(room, set_id);
        let _ = commands.entity(entity).despawn();
    }
}
