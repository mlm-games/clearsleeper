use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::player::Player;

pub const GOHMA_ACTOR: &str = "Btd";

pub fn is_gohma(name: &str) -> bool {
    name == GOHMA_ACTOR
}

pub type Xyz = [f32; 3];

const ZERO_XYZ: Xyz = [0.0, 0.0, 0.0];

const HIO_M05: u8 = 0;
const HIO_M06: u8 = 0;
const HIO_M07: u8 = 0;
const HIO_M08: f32 = 1.0;
const HIO_M0C: i16 = 0x96;
const HIO_M0E: i16 = 5;
const HIO_M10: u8 = 0;
const HIO_M14: f32 = 100.0;
const HIO_M18: f32 = 0.5;
const HIO_M1C: f32 = 0.5;
const HIO_M20: f32 = 0.2;
const HIO_M24: f32 = 0.2;
const HIO_M2C: i16 = 10;
const HIO_M28: f32 = 150.0;
const HIO_M3C: i16 = 0xb6;
const HIO_M3E: i16 = 0x9c;
const HIO_M40: i16 = 0x56;
const HIO_M42: i16 = 0xff;
const HIO_M44: i16 = 0x8e;
const HIO_M46: i16 = 0x24;
const HIO_M48: f32 = 12.0;
const HIO_M4C: f32 = 35.0;
const HIO_M50: f32 = 1.0;
const HIO_M54: u8 = 0;

pub const JOINT_CHECK: [i8; 73] = [
    -1, 0x05, -1, -1, 0x11, -1, -1, -1, -1, -1, 0x12, -1, -1, -1, 0x06, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0x04, -1, -1, 0x0D, -1, -1, 0x0F, -1,
    -1, 0x0E, -1, -1, 0x11, 0x03, -1, -1, 0x0B, -1, -1, 0x0C, -1, -1, -1, 0x02, -1, -1, -1, 0x64,
    -1, 0x00, 0x01, 0x65, 0x66, -1, -1, 0x07, 0x08, -1, -1, 0x09, 0x0A,
];

const SE_BTD_MAGMA_MOVE: u32 = 0x583E;
const SE_BTD_ARM_STRIKE: u32 = 0x581B;
const SE_BTD_WIND_JAB: u32 = 0x5818;
const SE_BTD_WIND_ATTACK: u32 = 0x5819;
const SE_BTD_WIND_BEF_FIRE: u32 = 0x581A;
const SE_BTD_ARM_PULLOUT_1: u32 = 0x581C;
const SE_BTD_ARM_PULLOUT_2: u32 = 0x581D;
const SE_BTD_HIFUKI_BEF: u32 = 0x501E;
const SE_BTD_HIFUKI: u32 = 0x501F;
const SE_BTD_BRK_SHIELD_S: u32 = 0x582C;
const SE_BTD_INTO_MAGMA_N: u32 = 0x582D;
const SE_BTD_ENT_BLAST: u32 = 0x5826;
const SE_BTD_LIFT_UP: u32 = 0x582E;
const SE_BTD_JUMP_DOWN: u32 = 0x5830;
const SE_BTD_LANDING: u32 = 0x5831;
const SE_BTD_FALL_FWD: u32 = 0x5832;
const SE_BTD_GET_UP: u32 = 0x5833;
const SE_BTD_BRK_SHIELD_L: u32 = 0x5834;
const SE_BTD_BECOME_BLACK: u32 = 0x5837;
const SE_BTD_ENDURE: u32 = 0x5838;
const SE_BTD_BECOME_RED: u32 = 0x5839;
const SE_BTD_LAST_EXPLODE: u32 = 0x583A;
const SE_BTD_DROP_HEAD: u32 = 0x583B;
const SE_MAGTAIL_ATTACK: u32 = 0x5810;
const SE_BOSS_EXPLODE: u32 = 0x583C;
const SE_ATM_JINARI: u32 = 0x105E;
const SE_OBJ_BTD_FIRE_DROP: u32 = 0x6942;
const SE_LK_LAST_HIT: u32 = 0x2828;
const SE_LK_B_BOSS_WARP: u32 = 0x2888;
const SE_CV_BTD_ENTER: u32 = 0x483D;
const SE_CV_BTD_AFT_DAMAGE: u32 = 0x4837;
const SE_CV_BTD_DMG_ROCK: u32 = 0x483B;
const SE_CV_BTD_DAMAGE: u32 = 0x4836;
const SE_CV_BTD_WAIT: u32 = 0x483E;
const SE_CV_BTD_JAB: u32 = 0x4834;
const SE_CV_BTD_ATTACK: u32 = 0x4835;
const SE_CV_BTD_BEF_FIRE: u32 = 0x483A;
const SE_CV_BTD_JITABATA: u32 = 0x4838;
const SE_CV_BTD_AFT_JITABATA: u32 = 0x4839;
const SE_CV_BTD_DIE: u32 = 0x483C;
const SE_LK_SW_HIT_S: u32 = 0x2803;
const SE_LK_ARROW_HIT: u32 = 0x2879;
const SE_LK_HAMMER_HIT: u32 = 0x2855;
const SE_LK_HS_SPIKE: u32 = 0x286F;

const BGM_GOMA: u32 = 0x80000003;
const BGM_GOMA_2: u32 = 0x80000010;
const BGM_PAST_GOMA: u32 = 0x8000004B;
const SE_BTD_ENT_BUBBLE: u32 = 0x5825;
const BGM_PAST_GOMA_2: u32 = 0x80000054;

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
const REG0_S2: i16 = 0;
const REG0_S3: i16 = 0;
const REG0_S4: i16 = 0;
const REG0_S5: i16 = 0;
const REG0_S6: i16 = 0;
const REG0_S7: i16 = 0;

const RND_SEED: i32 = 0x3a5f_1b2c;
const RND_MUL: [i32; 3] = [1103515245, 1103515245, 1103515245];
const RND_MOD: [i32; 3] = [2147483647, 2147483647, 2147483647];
const RND_SCALE: [f32; 3] = [2147483647.0, 2147483647.0, 2147483647.0];

const EMODE_NONE: u8 = 0;
const EMODE_LOOP: u8 = 1;
const STATE_STOP: u8 = 2;

const BCK_BTD_POSE: i32 = 0x08;
const BCK_DAMAGE1: i32 = 0x09;
const BCK_DAMAGE2: i32 = 0x0A;
const BCK_DAMAGE3: i32 = 0x0B;
const BCK_DEAD: i32 = 0x0C;
const BCK_FATTACK1: i32 = 0x0D;
const BCK_FATTACK2: i32 = 0x0E;
const BCK_FATTACK3: i32 = 0x0F;
const BCK_FATTACK4: i32 = 0x10;
const BCK_HASAMI_ATTACK: i32 = 0x11;
const BCK_HIT_L: i32 = 0x12;
const BCK_HIT_M: i32 = 0x13;
const BCK_HIT_R: i32 = 0x14;
const BCK_KYORO: i32 = 0x15;
const BCK_LATTACK1: i32 = 0x16;
const BCK_LATTACK2: i32 = 0x17;
const BCK_LATTACK3: i32 = 0x18;
const BCK_LATTACK4: i32 = 0x19;
const BCK_LATTACK5: i32 = 0x1A;
const BCK_LJAB1: i32 = 0x1B;
const BCK_LJAB2: i32 = 0x1C;
const BCK_LJAB3: i32 = 0x1D;
const BCK_MOTIAGE1: i32 = 0x1E;
const BCK_MOTIAGE2: i32 = 0x1F;
const BCK_MOTIAGE3: i32 = 0x20;
const BCK_RATTACK1: i32 = 0x21;
const BCK_RATTACK2: i32 = 0x22;
const BCK_RATTACK3: i32 = 0x23;
const BCK_RATTACK4: i32 = 0x24;
const BCK_RATTACK5: i32 = 0x25;
const BCK_RJAB1: i32 = 0x26;
const BCK_RJAB2: i32 = 0x27;
const BCK_RJAB3: i32 = 0x28;
const BCK_SIZUMU1: i32 = 0x2A;
const BCK_SIZUMU2: i32 = 0x2B;
const BCK_SIZUMU3: i32 = 0x2C;
const BCK_SSTART: i32 = 0x2D;
const BCK_UEMUKI_WAIT: i32 = 0x2F;
const BCK_UE_KAEN1: i32 = 0x30;
const BCK_UE_KAEN2: i32 = 0x31;
const BCK_UE_KAEN3: i32 = 0x32;
const BCK_WAIT1: i32 = 0x33;
const BCK_WARERU1: i32 = 0x34;
const BCK_WARERU2: i32 = 0x35;
const BCK_YOKO_KAEN1: i32 = 0x36;
const BCK_YOKO_KAEN2: i32 = 0x37;
const BCK_YOKO_KAEN3: i32 = 0x38;
const BCK_YOKO_WAIT: i32 = 0x39;

const FRAME_BTD_POSE: i16 = 30;
const FRAME_DAMAGE1: i16 = 24;
const FRAME_DAMAGE2: i16 = 30;
const FRAME_DAMAGE3: i16 = 114;
const FRAME_DEAD: i16 = 349;
const FRAME_FATTACK1: i16 = 34;
const FRAME_FATTACK2: i16 = 35;
const FRAME_FATTACK3: i16 = 9;
const FRAME_FATTACK4: i16 = 89;
const FRAME_HASAMI_ATTACK: i16 = 19;
const FRAME_HIT_L: i16 = 9;
const FRAME_HIT_M: i16 = 9;
const FRAME_HIT_R: i16 = 9;
const FRAME_KYORO: i16 = 75;
const FRAME_LATTACK1: i16 = 39;
const FRAME_LATTACK2: i16 = 5;
const FRAME_LATTACK3: i16 = 13;
const FRAME_LATTACK4: i16 = 40;
const FRAME_LATTACK5: i16 = 19;
const FRAME_LJAB1: i16 = 18;
const FRAME_LJAB2: i16 = 5;
const FRAME_LJAB3: i16 = 14;
const FRAME_MOTIAGE1: i16 = 110;
const FRAME_MOTIAGE2: i16 = 29;
const FRAME_MOTIAGE3: i16 = 59;
const FRAME_RATTACK1: i16 = 39;
const FRAME_RATTACK2: i16 = 5;
const FRAME_RATTACK3: i16 = 13;
const FRAME_RATTACK4: i16 = 40;
const FRAME_RATTACK5: i16 = 19;
const FRAME_RJAB1: i16 = 18;
const FRAME_RJAB2: i16 = 5;
const FRAME_RJAB3: i16 = 14;
const FRAME_SIZUMU1: i16 = 9;
const FRAME_SIZUMU2: i16 = 5;
const FRAME_SIZUMU3: i16 = 149;
const FRAME_SSTART: i16 = 319;
const FRAME_UEMUKI_WAIT: i16 = 50;
const FRAME_UE_KAEN1: i16 = 59;
const FRAME_UE_KAEN2: i16 = 20;
const FRAME_UE_KAEN3: i16 = 69;
const FRAME_WAIT1: i16 = 50;
const FRAME_WARERU1: i16 = 116;
const FRAME_WARERU2: i16 = 98;
const FRAME_YOKO_KAEN1: i16 = 91;
const FRAME_YOKO_KAEN2: i16 = 7;
const FRAME_YOKO_KAEN3: i16 = 100;
const FRAME_YOKO_WAIT: i16 = 50;

const JAB_BCK: [i32; 8] = [
    BCK_RJAB1,
    BCK_RJAB2,
    BCK_RJAB3,
    0,
    0,
    BCK_LJAB1,
    BCK_LJAB2,
    BCK_LJAB3,
];

const PUNCH_BCK: [i32; 10] = [
    BCK_RATTACK1,
    BCK_RATTACK2,
    BCK_RATTACK3,
    BCK_RATTACK4,
    BCK_RATTACK5,
    BCK_LATTACK1,
    BCK_LATTACK2,
    BCK_LATTACK3,
    BCK_LATTACK4,
    BCK_LATTACK5,
];

const AT_SIZE: [f32; 19] = [
    100.0, 100.0, 250.0, 300.0, 300.0, 300.0, 300.0, 100.0, 250.0, 100.0, 250.0, 150.0, 150.0, 150.0, 150.0, 150.0,
    150.0, 150.0, 150.0,
];

const AT_SPH_SRC_RADIUS: f32 = 200.0;
const AT_SPH_COUNT: usize = 19;
const HAND_CYL_COUNT: usize = 6;
const REACH_SWORD: f32 = 20.0;
const REACH_ARROW: f32 = 35.0;
const REACH_HAMMER: f32 = 50.0;
const REACH_HOOKSHOT: f32 = 5.0;
const REACH_BOOMERANG: f32 = 90.0;
const DEAD_HEAD_CENTRE: Xyz = [0.0, -20000.0, 0.0];
const PARKED_CENTRE: Xyz = [0.0, -10000.0, 0.0];

const HAND_CO_POS_X: [f32; 6] = [630.0, 750.0, 700.0, -630.0, -750.0, -700.0];
const HAND_CO_POS_Y: [f32; 6] = [0.0, 100.0, 0.0, 0.0, 100.0, 0.0];
const HAND_CO_POS_Z: [f32; 6] = [1680.0, 1600.0, 1320.0, 1680.0, 1600.0, 1320.0];
const HAND_CO_SIZE_H: [f32; 6] = [250.0, 200.0, 200.0, 250.0, 200.0, 200.0];
const HAND_CO_SIZE_R: [f32; 6] = [30.0, 170.0, 180.0, 30.0, 170.0, 180.0];

const ANGLE_X: usize = 0;
const ANGLE_Y: usize = 1;
const ANGLE_Z: usize = 2;

const MAX_HEALTH: i32 = 10;
const EYE_RADIUS: f32 = 80.0;
const FIRE_RADIUS: f32 = 100.0;
const EYE_REACH: f32 = 400.0;
const HAND_AT_FRAME: i16 = 0x14;
const HAND_ANGLE_TARGET: i16 = 13000;
const HAND_ANGLE_STEP: i16 = 0x1000;
const HAND_ANGLE_REL: i16 = 0x400;
const HAND_IDLE_ROLL: f32 = 150.0;
const HAND_IDLE_RANGE: f32 = 100.0;
const CULL_MIN: Xyz = [-2000.0, -2000.0, -2000.0];
const CULL_MAX: Xyz = [2000.0, 2000.0, 2000.0];

const DEMO_PLAYER_OFFSET: Xyz = [-1534.0, 0.0, 909.0];
const DEMO_MAGMA_FLOOR: f32 = -50.0;
const DEMO_MAGMA_FLOOR_ALT: f32 = -30.0;
const DEMO_MAGMA_RISE: f32 = 2300.0;
const DEMO_STEP_UP: f32 = 5.0;
const DEMO_MIN_DIST: f32 = 200.0;
const HOHMA_ROCK_DROP: f32 = 100.0;

fn frame_max(bck: i32) -> i16 {
    match bck {
        BCK_BTD_POSE => FRAME_BTD_POSE,
        BCK_DAMAGE1 => FRAME_DAMAGE1,
        BCK_DAMAGE2 => FRAME_DAMAGE2,
        BCK_DAMAGE3 => FRAME_DAMAGE3,
        BCK_DEAD => FRAME_DEAD,
        BCK_FATTACK1 => FRAME_FATTACK1,
        BCK_FATTACK2 => FRAME_FATTACK2,
        BCK_FATTACK3 => FRAME_FATTACK3,
        BCK_FATTACK4 => FRAME_FATTACK4,
        BCK_HASAMI_ATTACK => FRAME_HASAMI_ATTACK,
        BCK_HIT_L => FRAME_HIT_L,
        BCK_HIT_M => FRAME_HIT_M,
        BCK_HIT_R => FRAME_HIT_R,
        BCK_KYORO => FRAME_KYORO,
        BCK_LATTACK1 => FRAME_LATTACK1,
        BCK_LATTACK2 => FRAME_LATTACK2,
        BCK_LATTACK3 => FRAME_LATTACK3,
        BCK_LATTACK4 => FRAME_LATTACK4,
        BCK_LATTACK5 => FRAME_LATTACK5,
        BCK_LJAB1 => FRAME_LJAB1,
        BCK_LJAB2 => FRAME_LJAB2,
        BCK_LJAB3 => FRAME_LJAB3,
        BCK_MOTIAGE1 => FRAME_MOTIAGE1,
        BCK_MOTIAGE2 => FRAME_MOTIAGE2,
        BCK_MOTIAGE3 => FRAME_MOTIAGE3,
        BCK_RATTACK1 => FRAME_RATTACK1,
        BCK_RATTACK2 => FRAME_RATTACK2,
        BCK_RATTACK3 => FRAME_RATTACK3,
        BCK_RATTACK4 => FRAME_RATTACK4,
        BCK_RATTACK5 => FRAME_RATTACK5,
        BCK_RJAB1 => FRAME_RJAB1,
        BCK_RJAB2 => FRAME_RJAB2,
        BCK_RJAB3 => FRAME_RJAB3,
        BCK_SIZUMU1 => FRAME_SIZUMU1,
        BCK_SIZUMU2 => FRAME_SIZUMU2,
        BCK_SIZUMU3 => FRAME_SIZUMU3,
        BCK_SSTART => FRAME_SSTART,
        BCK_UEMUKI_WAIT => FRAME_UEMUKI_WAIT,
        BCK_UE_KAEN1 => FRAME_UE_KAEN1,
        BCK_UE_KAEN2 => FRAME_UE_KAEN2,
        BCK_UE_KAEN3 => FRAME_UE_KAEN3,
        BCK_WAIT1 => FRAME_WAIT1,
        BCK_WARERU1 => FRAME_WARERU1,
        BCK_WARERU2 => FRAME_WARERU2,
        BCK_YOKO_KAEN1 => FRAME_YOKO_KAEN1,
        BCK_YOKO_KAEN2 => FRAME_YOKO_KAEN2,
        BCK_YOKO_KAEN3 => FRAME_YOKO_KAEN3,
        BCK_YOKO_WAIT => FRAME_YOKO_WAIT,
        _ => FRAME_WAIT1,
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
                    self.state |= EMODE_LOOP;
                    if low <= 0 {
                        break;
                    }
                    self.frame += (low) as f32;
                }
                while self.frame >= end {
                    self.state |= EMODE_LOOP;
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
            super::walk::chase_f(&mut self.cur_morf, 1.0, self.morf_step);
        }
        self.update();
    }

    fn is_stop(&self) -> bool {
        self.state & STATE_STOP != 0 || self.rate == 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hahen {
    pub m04: u8,
    pub m05: u8,
    pub m08: Xyz,
    pub m14: Xyz,
    pub m20: i16,
    pub m22: i16,
    pub m24: u8,
}

impl Hahen {
    const ZERO: Self = Hahen {
        m04: 0,
        m05: 0,
        m08: ZERO_XYZ,
        m14: ZERO_XYZ,
        m20: 0,
        m22: 0,
        m24: 0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sibuki {
    pub m00: i8,
    pub m04: Xyz,
    pub m10: Xyz,
    pub m1c: u8,
    pub m20: u8,
    pub m24: u8,
}

impl Sibuki {
    const ZERO: Self = Sibuki {
        m00: 0,
        m04: ZERO_XYZ,
        m10: ZERO_XYZ,
        m1c: 0,
        m20: 0,
        m24: 0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtSph {
    pub centre: Xyz,
    pub radius: f32,
    pub at_set: bool,
    pub at_atp: u8,
    pub co_set: bool,
    pub tg_set: bool,
    pub tg_shield: bool,
}

impl AtSph {
    const SRC: Self = AtSph {
        centre: ZERO_XYZ,
        radius: AT_SPH_SRC_RADIUS,
        at_set: true,
        at_atp: 1,
        co_set: true,
        tg_set: true,
        tg_shield: true,
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HandCyl {
    pub centre: Xyz,
    pub radius: f32,
    pub height: f32,
    pub at_set: bool,
    pub at_atp: u8,
    pub co_set: bool,
    pub tg_set: bool,
    pub tg_shield: bool,
}

impl HandCyl {
    const SRC: Self = HandCyl {
        centre: ZERO_XYZ,
        radius: 0.0,
        height: 0.0,
        at_set: false,
        at_atp: 0,
        co_set: true,
        tg_set: true,
        tg_shield: true,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum At {
    Idle,
    Sword,
    Arrow,
    FireArrow,
    IceArrow,
    LightArrow,
    Hammer,
    Bomb,
    Boomerang,
    Hookshot,
}

impl At {
    fn reach(self) -> f32 {
        match self {
            At::Idle => 0.0,
            At::Sword => REACH_SWORD,
            At::Hookshot => REACH_HOOKSHOT,
            At::Arrow | At::FireArrow | At::IceArrow | At::LightArrow => REACH_ARROW,
            At::Hammer => REACH_HAMMER,
            At::Bomb => super::bombflower::BLAST_RADIUS,
            At::Boomerang => REACH_BOOMERANG,
        }
    }

    fn power(self, blade: u8, soup: bool) -> i32 {
        match self {
            At::Sword => super::damage::sword_power(blade, soup),
            At::Arrow | At::FireArrow | At::IceArrow | At::LightArrow => super::damage::ARROW,
            At::Bomb => super::damage::BOMB,
            At::Boomerang => super::damage::BOMERANG,
            At::Hammer => super::hammer::AT_POWER,
            At::Idle | At::Hookshot => 0,
        }
    }

    fn resulting_type(self) -> u8 {
        match self {
            At::Sword => 1,
            At::Bomb | At::Boomerang => 2,
            At::Hookshot => 0,
            At::Idle => 0xC,
            At::Arrow
            | At::FireArrow
            | At::IceArrow
            | At::LightArrow
            | At::Hammer => 5,
        }
    }

    fn se(self) -> u32 {
        match self {
            At::Arrow | At::FireArrow | At::IceArrow | At::LightArrow => SE_LK_ARROW_HIT,
            At::Hammer => SE_LK_HAMMER_HIT,
            At::Hookshot => SE_LK_HS_SPIKE,
            At::Idle | At::Sword | At::Bomb | At::Boomerang => SE_LK_SW_HIT_S,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blow {
    pub at: At,
    pub blade: u8,
    pub soup: bool,
}

impl Blow {
    fn reach(&self) -> f32 {
        self.at.reach()
    }

    fn power(&self) -> i32 {
        self.at.power(self.blade, self.soup)
    }

    fn resulting_type(&self) -> u8 {
        self.at.resulting_type()
    }

    fn se(&self) -> u32 {
        self.at.se()
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Gohma {
    pub health: i32,
    pub max_health: i32,
    pub phase2: u8,
    pub m_at_sphs: [AtSph; AT_SPH_COUNT],
    pub m_hand_cyls: [HandCyl; HAND_CYL_COUNT],
    pub m02e1: i8,
    pub m02e2: i16,
    pub m02e4: i16,
    pub gohma_state: i16,
    pub attack_type: i8,
    pub m02e9: i8,
    pub m02ea: i8,
    pub m02ec: [i16; 2],
    pub m02f0: i16,
    pub stun_timer: i16,
    pub action_state: i16,
    pub m02f6: u8,
    pub m02f8: i16,
    pub m02fa: i16,
    pub m02fc: [Xyz; 19],
    pub m03e0: Xyz,
    pub m5e7c: u8,
    pub m5e80: f32,
    pub m5e84: u8,
    pub total_damage: i8,
    pub m5e86: u8,
    pub m5e88: i16,
    pub m5e8a: i16,
    pub m5e8c: i16,
    pub m5e8e: i16,
    pub m5e90: i16,
    pub m5e92: i16,
    pub m5e94: i16,
    pub m5e96: i16,
    pub m5e98: [Xyz; 25],
    pub m5fc4: [i16; AT_SPH_COUNT],
    pub m5fea: [i16; HAND_CYL_COUNT],
    pub m6028: [i16; 7],
    pub m602c: i16,
    pub m602e: i16,
    pub m6030: i16,
    pub m6032: i16,
    pub m6034: i16,
    pub m6038: u32,
    pub m603c: Xyz,
    pub m6048: [i16; 3],
    pub m60d0: f32,
    pub m60d4: f32,
    pub m60d8: u8,
    pub m60d9: u8,
    pub m60da: u8,
    pub kankyo_state: u8,
    pub kankyo_timer: i16,
    pub kankyo_colset: u8,
    pub m6190: u8,
    pub morph: Frame,
    pub hahen: [Hahen; 35],
    pub sibuki: [Sibuki; 50],
    pub m6e14: i8,
    pub m6e15: u8,
    pub m6e16: i16,
    pub m6e18: i16,
    pub m6e1a: i16,
    pub m6e1c: Xyz,
    pub m6e28: Xyz,
    pub m6e34: i16,
    pub m6e36: i16,
    pub m6e3c: f32,
    pub m6e40: f32,
    pub m6e44: f32,
    pub m6e48: f32,
    pub m6e4c: u8,
    pub m6e50: Xyz,
    pub m6e5c: Xyz,
    pub m6e68: Xyz,
    pub m6e74: [i16; 3],
    pub m6e7c: f32,
    pub m6e80: f32,
    pub m6e84: f32,
    pub m6e88: u8,
    pub m6e90: i8,
    pub static_center_pos: Xyz,
    pub kui: u32,
    pub dr2: u32,
    pub kui_health: i32,
    pub dr2_43c: u8,
    pub dr2_4ba: i16,
    pub dr2_4ca: i16,
    pub eye_pos: Xyz,
    pub eye_radius: f32,
    pub eye_shield: bool,
    pub fire_pos: Xyz,
    pub fire_radius: f32,
    pub speed_f: f32,
    pub scale: f32,
    pub morph_sound_idx: i32,
    pub morph_bck: i32,
    pub demo_mode: bool,
    pub entered: bool,
    pub home: Vec3,
    pub seed: [i32; 3],
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    players: Query<'w, 's, (Entity, &'static Player)>,
    others: Query<'w, 's, (Entity, &'static Actor), Without<Gohma>>,
    combat: Res<'w, super::combat::Combat>,
    held: Res<'w, super::item::Held>,
    hammer: Res<'w, super::hammer::Cut>,
    blasts: Res<'w, super::bombflower::Blasts>,
    shafts: Query<'w, 's, &'static super::arrow::Arrow>,
    bolts: Query<'w, 's, &'static super::boomerang::Boomerang>,
    chains: Query<'w, 's, &'static super::hookshot::Hookshot>,
}



pub struct Sink<'a> {
    pub events: &'a mut SimEvents,
}

impl Sink<'_> {
    fn se(&mut self, id: u32) {
        self.events.list.push(SimEvent::Se(id));
    }

    fn se_at(&mut self, id: u32, _pos: Xyz) {
        self.events.list.push(SimEvent::Se(id));
    }

    fn bgm(&mut self, id: u32) {
        self.events.list.push(SimEvent::Bgm(id));
    }

    fn stop_bgm(&mut self, id: u32) {
        self.events.list.push(SimEvent::Bgm(id));
    }
}

pub struct Env {
    pub player_pos: Vec3,
    pub player_eye: Vec3,
    pub player_angle: [i16; 3],
    pub player_cut: bool,
    pub blow: Blow,
    pub demo_stage: bool,
    pub kui: u32,
    pub kui_health: i32,
    pub dr2: u32,
    pub dr2_43c: u8,
    pub dr2_4ba: i16,
    pub dr2_4ca: i16,
    pub item_pos: Option<Vec3>,
    pub item_speed: f32,
    pub item_name: &'static str,
}

impl Gohma {
    pub fn spawn(_parameters: u32) -> Self {
        Gohma {
            health: MAX_HEALTH,
            max_health: MAX_HEALTH,
            phase2: 0,
            m_at_sphs: [AtSph::SRC; AT_SPH_COUNT],
            m_hand_cyls: [HandCyl::SRC; HAND_CYL_COUNT],
            m02e1: 0,
            m02e2: 0,
            m02e4: 0,
            gohma_state: 0,
            attack_type: 0,
            m02e9: 0,
            m02ea: 0,
            m02ec: [0, 0],
            m02f0: 0,
            stun_timer: 0,
            action_state: 0,
            m02f6: 0,
            m02f8: 0,
            m02fa: 0,
            m02fc: [ZERO_XYZ; 19],
            m03e0: ZERO_XYZ,
            m5e7c: 0,
            m5e80: 0.0,
            m5e84: 0,
            total_damage: 0,
            m5e86: 0,
            m5e88: 0,
            m5e8a: 0,
            m5e8c: 0,
            m5e8e: 0,
            m5e90: 0,
            m5e92: 10,
            m5e94: 0,
            m5e96: 0,
            m5e98: [ZERO_XYZ; 25],
            m5fc4: [0; AT_SPH_COUNT],
            m5fea: [0; HAND_CYL_COUNT],
            m6028: [0, 0, 0, 0, 0, 0, 0],
            m602c: 0,
            m602e: 0,
            m6030: 0,
            m6032: 0,
            m6034: 0,
            m6038: 0,
            m603c: ZERO_XYZ,
            m6048: [0, 0, 0],
            m60d0: 0.0,
            m60d4: 0.0,
            m60d8: 0,
            m60d9: 0,
            m60da: 0,
            kankyo_state: 0,
            kankyo_timer: 0,
            kankyo_colset: 0,
            m6190: 0,
            morph: Frame::idle(),
            hahen: [Hahen::ZERO; 35],
            sibuki: [Sibuki::ZERO; 50],
            m6e14: 0,
            m6e15: 0,
            m6e16: 0,
            m6e18: 0,
            m6e1a: 0,
            m6e1c: ZERO_XYZ,
            m6e28: ZERO_XYZ,
            m6e34: 0,
            m6e36: 0,
            m6e3c: 0.0,
            m6e40: 0.0,
            m6e44: 0.0,
            m6e48: 0.0,
            m6e4c: 0,
            m6e50: ZERO_XYZ,
            m6e5c: ZERO_XYZ,
            m6e68: ZERO_XYZ,
            m6e74: [0, 0, 0],
            m6e7c: 0.0,
            m6e80: 0.0,
            m6e84: 0.0,
            m6e88: 0,
            m6e90: 1,
            static_center_pos: ZERO_XYZ,
            kui: 0,
            dr2: 0,
            kui_health: 0,
            dr2_43c: 0,
            dr2_4ba: 0,
            dr2_4ca: 0,
            eye_pos: ZERO_XYZ,
            eye_radius: 50.0,
            eye_shield: true,
            fire_pos: ZERO_XYZ,
            fire_radius: 100.0,
            speed_f: 0.0,
            scale: HIO_M08,
            morph_sound_idx: -1,
            morph_bck: BCK_WAIT1,
            demo_mode: false,
            entered: false,
            home: Vec3::ZERO,
            seed: [RND_SEED; 3],
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    fn rnd_f(&mut self, range: f32) -> f32 {
        rnd(&mut self.seed) * range
    }

    fn rnd_fx(&mut self, range: f32) -> f32 {
        range * (rnd(&mut self.seed) - 0.5) * 2.0
    }

    fn rnd_u16(&mut self) -> i16 {
        self.rnd_f(65536.0) as i16
    }

    fn anm_init(&mut self, bck: i32, morf: f32, loop_mode: u8, speed: f32, sound_idx: i32) {
        self.morph_sound_idx = sound_idx;
        self.morph_bck = bck;
        self.morph.set_morf(morf);
        self.morph.start = 0;
        self.morph.end = frame_max(bck);
        self.morph.loop_point = 0;
        self.morph.attribute = loop_mode;
        self.morph.rate = speed;
        self.morph.frame = 0.0;
        self.morph.state = 0;
    }

    fn is_stop(&self) -> bool {
        self.morph.is_stop()
    }
}

pub fn control(
    watch: Watch<'_, '_>,
    mut events: ResMut<SimEvents>,
    mut bosses: Query<(Entity, &mut Actor, &mut Gohma), (With<Gohma>, Without<Player>)>,
) {
    let players = watch.players;
    let Some((_, player)) = players.iter().next() else {
        return;
    };
    for (_, mut actor, mut boss) in &mut bosses {
        let at = {
            let mut found = At::Idle;
            if watch
                .chains
                .iter()
                .any(|chain| chain.phase != super::hookshot::Phase::Wait)
            {
                found = At::Hookshot;
            } else {
                for shaft in watch.shafts.iter() {
                    if shaft.phase != super::arrow::Phase::Flying {
                        continue;
                    }
                    found = match shaft.kind {
                        super::arrow::TYPE_LIGHT => At::LightArrow,
                        super::arrow::TYPE_FIRE => At::FireArrow,
                        super::arrow::TYPE_ICE => At::IceArrow,
                        _ => At::Arrow,
                    };
                    break;
                }
                if found == At::Idle
                    && watch
                        .bolts
                        .iter()
                        .any(|bolt| bolt.parameter == super::boomerang::MODE_THROW)
                {
                    found = At::Boomerang;
                }
                if found == At::Idle
                    && (watch.hammer.kind == super::hammer::CUT_HAMMER_SIDESWING
                        || watch.hammer.kind == super::hammer::CUT_HAMMER_FRONTSWING)
                {
                    found = At::Hammer;
                }
                if found == At::Idle && watch.combat.active && watch.combat.frame >= 1 && watch.combat.frame < 6 {
                    found = At::Sword;
                }
                if found == At::Idle && !watch.blasts.list.is_empty() {
                    found = At::Bomb;
                }
            }
            found
        };
        let kui = watch
            .others
            .iter()
            .find(|(_, other)| Gohma::k_a_d_check(&other.name, other.parameters));
        let dr2 = watch
            .others
            .iter()
            .find(|(_, other)| Gohma::dr2_a_d_check(&other.name));
        let env = Env {
            player_pos: player.pos,
            player_eye: player.pos,
            player_angle: [0, 0, 0],
            player_cut: false,
            blow: Blow {
                at,
                blade: watch.held.item,
                soup: watch.combat.soup,
            },
            demo_stage: false,
            kui: if kui.is_some() { 1 } else { 0 },
            kui_health: boss.kui_health,
            dr2: if dr2.is_some() { 1 } else { 0 },
            dr2_43c: boss.dr2_43c,
            dr2_4ba: boss.dr2_4ba,
            dr2_4ca: boss.dr2_4ca,
            item_pos: watch
                .others
                .iter()
                .find(|(_, other)| Gohma::wepon_s_check(&other.name))
                .map(|(_, other)| other.pos),
            item_speed: 0.0,
            item_name: "",
        };
        let actor = &mut *actor;
        let boss = &mut *boss;
        let mut sink = Sink {
            events: &mut events,
        };
        boss.execute(actor, &env, &mut sink);
    }
}

impl Gohma {
    fn execute(&mut self, actor: &mut Actor, env: &Env, out: &mut Sink<'_>) {
        if !self.entered {
            self.begin(actor, env, out);
        }
        self.tick(actor, env, out);
    }

    fn begin(&mut self, actor: &mut Actor, env: &Env, out: &mut Sink<'_>) {
        self.entered = true;
        self.home = actor.pos;
        self.health = MAX_HEALTH;
        self.max_health = MAX_HEALTH;
        self.eye_radius = EYE_RADIUS;
        self.fire_radius = FIRE_RADIUS;
        if env.demo_stage {
            self.m6e80 = DEMO_MAGMA_FLOOR;
            self.m6e84 = 0.0;
            self.m02ec[0] = ((self.rnd_f(50.0) + 70.0) + 100.0) as i16;
            out.bgm(BGM_GOMA);
        } else {
            self.m02e4 = 10;
            self.m6e80 = DEMO_MAGMA_FLOOR;
            self.m6e84 = DEMO_MAGMA_FLOOR_ALT;
            self.m02ec[0] = REG0_S4.wrapping_add(0x96);
            self.m6e7c = REG0_F18 + DEMO_MAGMA_RISE;
        }
        self.m6e90 = 1;
        actor.pos.y = if env.demo_stage {
            -10000.0
        } else {
            actor.pos.y
        };
    }

    fn tick(&mut self, actor: &mut Actor, env: &Env, out: &mut Sink<'_>) {
        self.kui = env.kui;
        self.kui_health = env.kui_health;
        self.dr2 = env.dr2;
        self.dr2_43c = env.dr2_43c;
        self.dr2_4ba = env.dr2_4ba;
        self.dr2_4ca = env.dr2_4ca;
        self.m6e88 = 1;
        self.m6e7c += REG0_F15 + 30.0;
        if self.m6e7c > REG0_F18 + DEMO_MAGMA_RISE {
            self.m6e7c = REG0_F18 + DEMO_MAGMA_RISE;
        }
        self.m02e2 = self.m02e2.wrapping_add(1);
        for lane in 0..2 {
            if self.m02ec[lane] != 0 {
                self.m02ec[lane] -= 1;
            }
        }
        if self.action_state != 0 {
            self.action_state -= 1;
        }
        self.move_step(actor, env, out);
        self.scale = HIO_M08;
        if HIO_M05 == 0 {
            self.morph.play();
        }
        self.damage_check(actor, env, out);
        self.face_step(actor, env);
        self.hahen_move();
        self.sibuki_move(env, out);
        self.demo_camera(actor, env, out);
        self.btd_effect();
        self.btd_kankyo();
        self.cc_at_sph_set();
        self.cc_hand_cyl_set(actor);
        self.eye_sph_set();
        self.collide(actor, env);
    }

    fn cc_at_sph_set(&mut self) {
        let mut scale = 1.0f32;
        if self.phase2 != 0 {
            scale = 0.6;
        }
        for i in 0..AT_SPH_COUNT {
            if self.phase2 != 0 && i != 8 && i != 10 {
                self.m_at_sphs[i].tg_shield = false;
            }
            self.m_at_sphs[i].radius = scale * AT_SIZE[i];
            self.m_at_sphs[i].centre = if self.m6e4c != 0 {
                DEAD_HEAD_CENTRE
            } else {
                self.m02fc[i]
            };
            if i == 8 || i == 10 {
                if self.m5e84 == 1 || self.m5e84 == 2 {
                    self.m_at_sphs[i].at_set = true;
                    if self.m5e84 == 2 {
                        self.m_at_sphs[i].at_atp = 2;
                    } else {
                        self.m_at_sphs[i].at_atp = 1;
                    }
                } else {
                    self.m_at_sphs[i].at_set = false;
                }
                self.m_at_sphs[i].co_set = false;
                self.m_at_sphs[i].tg_set = false;
            } else {
                if i < 2 && self.m5e84 == 3 {
                    self.m_at_sphs[i].at_set = true;
                } else {
                    self.m_at_sphs[i].at_set = false;
                }
                if i < 2 && self.phase2 == 0 {
                    self.m_at_sphs[i].co_set = false;
                    self.m_at_sphs[i].tg_set = false;
                }
            }
        }
    }

    fn cc_hand_cyl_set(&mut self, actor: &Actor) {
        let mut turn = 0i16;
        let mut scale = 1.0f32;
        if self.m5e7c == 2 {
            turn = -(REG0_S5.wrapping_add(0x1068));
            scale = REG0_F10 + 0.95;
        } else if self.m5e7c == 3 {
            turn = REG0_S5.wrapping_add(0x1068);
            scale = REG0_F10 + 0.95;
        }
        for cyl in self.m_hand_cyls.iter_mut() {
            cyl.centre = PARKED_CENTRE;
        }
        for i in 0..HAND_CYL_COUNT {
            if self.m5e7c != 0
                && ((self.m5e7c == 1 || (self.m5e7c == 2 && i <= 2))
                    || (self.m5e7c == 3 && i >= 3))
            {
                let local = [
                    HAND_CO_POS_X[i],
                    HAND_CO_POS_Y[i],
                    scale * HAND_CO_POS_Z[i],
                ];
                self.m5e98[i + 19] = mtx_position_y(actor.angle[ANGLE_Y].wrapping_add(turn), local);
                self.m5e98[i + 19][0] += actor.pos.x;
                self.m5e98[i + 19][1] += actor.pos.y;
                self.m5e98[i + 19][2] += actor.pos.z;
                self.m_hand_cyls[i].centre = self.m5e98[i + 19];
                self.m_hand_cyls[i].height = HAND_CO_SIZE_H[i];
                self.m_hand_cyls[i].radius = HAND_CO_SIZE_R[i];
            }
        }
        self.m5e7c = 0;
    }

    fn eye_sph_set(&mut self) {
        if self.phase2 != 0 {
            self.eye_shield = false;
        }
        self.eye_radius = REG0_F5 + EYE_RADIUS;
        self.eye_pos = self.m03e0;
    }

    fn collide(&mut self, _actor: &Actor, env: &Env) {
        if let Some(item) = env.item_pos {
            if env.item_speed > 10.0 {
                let delta = item - self.eye3();
                if delta.length() < EYE_REACH {
                    self.m02f8 = HAND_AT_FRAME;
                }
            }
        }
        let _ = env.item_name;
        let eye_delta = env.player_eye - self.eye3();
        if env.player_cut && abs3(eye_delta.x, eye_delta.y, eye_delta.z) < EYE_REACH {
            self.m02f8 = HAND_AT_FRAME;
        }
        if (self.m02f8 != 0) && self.phase2 == 0 {
            self.m02f8 -= 1;
            super::stream::add_calc_angle_s2(
                &mut self.m02fa,
                HAND_ANGLE_TARGET,
                1,
                HAND_ANGLE_STEP,
            );
        } else {
            super::stream::add_calc_angle_s2(&mut self.m02fa, 0, 4, HAND_ANGLE_REL);
        }
        if self.m02f0 == 0 {
            self.m02f0 = (self.rnd_f(HAND_IDLE_ROLL) + HAND_IDLE_RANGE) as i16;
            self.m02f8 = HAND_AT_FRAME;
        }
        self.m6e15 = 0;
    }

    fn move_step(&mut self, actor: &mut Actor, env: &Env, out: &mut Sink<'_>) {
        self.m5e84 = 0;
        let mut busy = false;
        match self.m02e4 {
            0 => self.wait(actor, env, out),
            1 => self.attack(actor, env, out),
            2 => {
                busy = self.damage(actor, env, out);
            }
            0xA => {
                self.startdemo(actor, env, out);
                busy = true;
            }
            0xB => {
                self.end(actor, env, out);
                busy = true;
            }
            _ => {}
        }
        if busy {
            self.m5e86 = 0;
        }
    }

    fn face_step(&mut self, actor: &mut Actor, env: &Env) {
        if self.m5e94 != 0 {
            if self.dr2 != 0
                && (self.dr2_4ba == 0 || self.dr2_4ba >= 0xA)
                && (self.dr2_4ca == 0 || self.dr2_4ca >= 0x62)
            {
                let goal = if self.m5e90 != 0 {
                    self.m5e90
                } else {
                    self.search_player_angle_y(actor, env)
                };
                super::stream::add_calc_angle_s2(
                    &mut actor.angle[ANGLE_Y],
                    goal,
                    self.m5e92,
                    self.m5e94,
                );
            }
            self.m5e90 = 0;
            self.m5e92 = 10;
            self.m5e94 = 0;
        }
        let mut target = 0i16;
        let mut yaw = 0i16;
        if self.m5e86 != 0 {
            let flat = env.player_pos.x * env.player_pos.x + env.player_pos.z * env.player_pos.z;
            if flat < 40000.0 {
                target = 0;
                yaw = 0;
            } else {
                yaw = self
                    .search_player_angle_y(actor, env)
                    .wrapping_sub(actor.angle[ANGLE_Y]);
                let limit = 5000i16;
                if yaw > limit {
                    yaw = limit;
                } else if yaw < -limit {
                    yaw = -limit;
                }
                let diff = env.player_eye - self.eye3();
                let pitch = super::walk::to_s16(diff.x.atan2(diff.z));
                target = wrapping_div(
                    pitch.wrapping_sub(actor.angle[ANGLE_Y]),
                    2,
                );
                if target > 0x2000 {
                    target = 0x2000;
                } else if target < -0x2000 {
                    target = -0x2000;
                }
            }
        }
        super::stream::add_calc_angle_s2(&mut self.m5e88, yaw, 4, 0x200);
        super::stream::add_calc_angle_s2(&mut self.m5e8e, target, 4, 0x200);
        super::stream::add_calc_angle_s2(&mut self.m5e8a, self.m5e8c, 4, 0x100);
        self.m5e86 = 0;
        self.m5e8c = 0;
    }

    fn eye3(&self) -> Vec3 {
        Vec3::new(self.eye_pos[0], self.eye_pos[1], self.eye_pos[2])
    }

    fn hahen_move(&mut self) {
        for piece in self.hahen.iter_mut() {
            if piece.m04 != 0 {
                piece.m08[0] += piece.m14[0];
                piece.m08[1] += piece.m14[1];
                piece.m08[2] += piece.m14[2];
                piece.m20 = piece.m20.wrapping_add(0x900);
                piece.m22 = piece.m22.wrapping_add(0x600);
                piece.m14[1] -= 3.0;
                if piece.m08[1] < -100.0 {
                    piece.m04 = 0;
                }
            }
        }
    }

    fn damage_check(&mut self, actor: &mut Actor, env: &Env, out: &mut Sink<'_>) {
        if self.m02f6 != 0 && self.m02f6 == 1 {
            self.m02f6 = 2;
            self.m02e4 = 2;
            self.gohma_state = 0x14;
            out.se(SE_CV_BTD_DMG_ROCK);
            self.eff_off();
            return;
        }
        if self.phase2 == 0 && self.action_state == 0 && self.eye_hit(env) {
            self.action_state = 10;
            out.se(env.blow.se());
        }
        if self.phase2 == 1 && self.action_state == 0 && self.eye_hit(env) {
            self.action_state = 7;
            self.total_damage = self.total_damage.wrapping_add(env.blow.power() as i8);
            self.health -= env.blow.power();
            if self.health <= 0 || HIO_M06 != 0 {
                self.m02e4 = 0xb;
                self.gohma_state = 0x32;
                out.se(SE_LK_LAST_HIT);
                return;
            }
            if self.m02e4 == 2 {
                if i32::from(self.total_damage) >= HIO_M0E as i32 {
                    self.gohma_state = 10;
                    self.total_damage = 0;
                } else {
                    let anm = if env.blow.resulting_type() == 1 {
                        self.hit_anm(actor, env)
                    } else {
                        BCK_HIT_M
                    };
                    self.anm_init(anm, 1.0, EMODE_NONE, 1.0, -1);
                    self.gohma_state = 3;
                }
            } else {
                self.m02e4 = 2;
                self.gohma_state = 0;
            }
            out.se(SE_CV_BTD_DAMAGE);
            self.eff_off();
        }
        for i in 0..AT_SPH_COUNT {
            if self.m5fc4[i] == 0 {
                if self.at_sph_hit(i, env) {
                    self.m5fc4[i] = 10;
                    self.m5e98[i] = self.m02fc[i];
                    out.se_at(env.blow.se(), self.m5e98[i]);
                    break;
                }
            } else {
                self.m5fc4[i] -= 1;
            }
        }
        for i in 0..HAND_CYL_COUNT {
            if self.m5fea[i] == 0 {
                if self.hand_cyl_hit(i, env) {
                    self.m5fea[i] = 10;
                    out.se_at(env.blow.se(), self.m5e98[i + 19]);
                    return;
                }
            } else {
                self.m5fea[i] -= 1;
            }
        }
    }

    fn eye_hit(&self, env: &Env) -> bool {
        let reach = env.blow.reach();
        if reach <= 0.0 {
            return false;
        }
        abs3(
            env.player_eye.x - self.eye_pos[0],
            env.player_eye.y - self.eye_pos[1],
            env.player_eye.z - self.eye_pos[2],
        ) <= self.eye_radius + reach
    }

    fn at_sph_hit(&self, i: usize, env: &Env) -> bool {
        if !self.m_at_sphs[i].tg_set {
            return false;
        }
        if env.blow.reach() <= 0.0 {
            return false;
        }
        let centre = self.m_at_sphs[i].centre;
        abs3(
            env.player_eye.x - centre[0],
            env.player_eye.y - centre[1],
            env.player_eye.z - centre[2],
        ) <= self.m_at_sphs[i].radius + env.blow.reach()
    }

    fn hand_cyl_hit(&self, i: usize, env: &Env) -> bool {
        if !self.m_hand_cyls[i].tg_set {
            return false;
        }
        if env.blow.reach() <= 0.0 {
            return false;
        }
        let centre = self.m_hand_cyls[i].centre;
        let half = self.m_hand_cyls[i].height * 0.5;
        let below = env.player_eye.y - (centre[1] - half);
        let above = (centre[1] + half) - env.player_eye.y;
        let dy = if below < above { below } else { above };
        let dy = if dy < 0.0 { 0.0 } else { dy };
        abs3(
            env.player_eye.x - centre[0],
            dy,
            env.player_eye.z - centre[2],
        ) <= self.m_hand_cyls[i].radius + env.blow.reach()
    }

    fn hit_anm(&self, actor: &Actor, env: &Env) -> i32 {
        let to_player = self.search_player_angle_y(actor, env);
        let rel = to_player.wrapping_sub(actor.angle[ANGLE_Y]);
        if rel > -0x100 && rel < 0x100 {
            BCK_HIT_M
        } else if rel > 0 {
            BCK_HIT_R
        } else {
            BCK_HIT_L
        }
    }

    fn k_a_d_check(name: &str, parameters: u32) -> bool {
        name == "Kui" && parameters == 0x511
    }

    fn dr2_a_d_check(name: &str) -> bool {
        name == "Dr2"
    }

    fn wepon_s_check(name: &str) -> bool {
        name == "Him2" || name == "Bomerang"
    }

    fn sibuki_move(&mut self, env: &Env, out: &mut Sink<'_>) {
        for idx in 0..self.sibuki.len() {
            if self.sibuki[idx].m00 == 0 {
                continue;
            }
            if self.sibuki[idx].m00 <= 2 {
                let mut landed = false;
                if idx == 0 && HIO_M54 != 0 {
                    add_calc2(
                        &mut self.sibuki[idx].m04[0],
                        env.player_pos.x,
                        1.0,
                        30.0,
                    );
                    add_calc2(
                        &mut self.sibuki[idx].m04[1],
                        env.player_pos.y + 70.0,
                        1.0,
                        30.0,
                    );
                    add_calc2(
                        &mut self.sibuki[idx].m04[2],
                        env.player_pos.z,
                        1.0,
                        30.0,
                    );
                    self.sibuki[idx].m10[1] = -10.0;
                } else {
                    self.sibuki[idx].m10[1] -= HIO_M50;
                    self.sibuki[idx].m04[0] += self.sibuki[idx].m10[0];
                    self.sibuki[idx].m04[1] += self.sibuki[idx].m10[1];
                    self.sibuki[idx].m04[2] += self.sibuki[idx].m10[2];
                }
                if self.sibuki[idx].m10[1] < 0.0 {
                    if self.sibuki[idx].m00 == 1 {
                        self.sibuki[idx].m00 += 1;
                    }
                    if self.sibuki[idx].m04[1] < 0.0 {
                        self.sibuki[idx].m04[1] = 0.0;
                        landed = true;
                    }
                }
                if landed {
                    let delta = self.sibuki[idx].m04;
                    let reach = abs3(delta[0], delta[1], delta[2]);
                    self.sibuki[idx].m00 = 0;
                    if reach < 1500.0 && self.m6e14 < 20 {
                        self.m6e14 += 1;
                        out.se(SE_OBJ_BTD_FIRE_DROP);
                        self.sibuki[idx].m00 = 0;
                    }
                }
            } else {
                self.sibuki[idx].m00 += 1;
                if self.sibuki[idx].m00 == 0x1e {
                    self.sibuki[idx].m00 = 0;
                }
            }
        }
    }

    fn btd_effect(&mut self) {
        for lane in 0..7 {
            if self.m6028[lane] != 0 {
                self.m6028[lane] -= 1;
            }
        }
    }

    fn btd_kankyo(&mut self) {
        let mut colset = 0u8;
        if self.kankyo_timer != 0 {
            self.kankyo_timer -= 1;
            self.kankyo_state = 2;
        }
        match self.kankyo_state {
            0 => add_calc0(&mut self.m60d0, 1.0, 0.04),
            1 => {
                add_calc2(&mut self.m60d4, 1.0, 1.0, 0.065);
                self.m60d8 = HIO_M3C as u8;
                self.m60d9 = HIO_M3E as u8;
                self.m60da = HIO_M40 as u8;
            }
            2 => {
                add_calc2(&mut self.m60d0, 1.0, 1.0, 0.2);
                add_calc2(&mut self.m60d4, 1.0, 1.0, 0.22500001);
                self.m60d8 = HIO_M42 as u8;
                self.m60d9 = HIO_M44 as u8;
                self.m60da = HIO_M46 as u8;
            }
            3 => add_calc2(&mut self.m60d0, 0.6, 1.0, 0.01),
            4 => add_calc2(&mut self.m60d0, 1.0, 1.0, 0.2),
            5 => {
                add_calc2(&mut self.m60d0, 1.0, 1.0, 0.005);
                colset = 1;
            }
            6 => {
                self.m60d0 = 0.0;
                colset = 2;
            }
            7 => {
                add_calc2(&mut self.m60d0, 1.0, 1.0, 0.03333);
                colset = 2;
            }
            8 => {
                add_calc0(&mut self.m60d0, 1.0, 0.06666);
                colset = 3;
            }
            _ => {}
        }
        self.kankyo_colset = colset;
        self.kankyo_state = 0;
        add_calc0(&mut self.m60d4, 1.0, 0.025);
    }
}

fn wrapping_div(value: i16, scale: i16) -> i16 {
    value / scale
}

fn abs3(x: f32, y: f32, z: f32) -> f32 {
    (x * x + y * y + z * z).sqrt()
}

fn bearing(from: Vec3, to: Vec3) -> i16 {
    super::walk::to_s16((to.x - from.x).atan2(to.z - from.z))
}

impl Gohma {
    fn wait(&mut self, actor: &mut Actor, env: &Env, out: &mut Sink<'_>) {
        let view = self.player_view_check(actor, env);
        let player_high = env.player_pos.y > REG0_F5 + 400.0;
        match self.gohma_state {
            0 => {
                self.anm_init(BCK_WAIT1, 20.0, EMODE_LOOP, 1.0, -1);
                self.gohma_state += 1;
                self.wait_engage(view, player_high, env);
            }
            1 => {
                if view < 0x1800 && self.m02ec[0] == 0 {
                    self.m02e4 = 1;
                    self.gohma_state = 0;
                    if self.rnd_f(1.0) < 0.5 {
                        self.m02e9 = 0;
                        self.m02ea = 1;
                    } else {
                        self.m02e9 = 5;
                        self.m02ea = 0;
                    }
                    if HIO_M10 == 0 {
                        if view > 0x400 && self.rnd_f(1.0) < 0.5 {
                            self.attack_type = 5;
                        } else if view > 0x800 {
                            self.attack_type = 5;
                        } else if self.phase2 == 0 {
                            if self.rnd_f(1.0) < HIO_M18 {
                                self.attack_type = 0;
                            } else if self.rnd_f(1.0) < HIO_M1C {
                                self.attack_type = 1;
                            } else {
                                self.attack_type = 2;
                            }
                        } else if self.rnd_f(1.0) < HIO_M20 {
                            self.attack_type = 0;
                        } else if self.rnd_f(1.0) < HIO_M24 {
                            self.attack_type = 1;
                        } else {
                            self.attack_type = 2;
                        }
                    } else if HIO_M10 == 1 {
                        self.attack_type = 0;
                    } else if HIO_M10 == 2 {
                        self.attack_type = 1;
                    } else if HIO_M10 == 3 {
                        self.attack_type = 2;
                    } else if HIO_M10 == 4 {
                        self.attack_type = 5;
                        self.m02e9 = 5;
                        self.m02ea = 0;
                    }
                } else {
                    self.m5e94 = 0x100;
                    if view >= 0x6000 && self.m02ec[1] == 0 {
                        self.gohma_state = 2;
                        self.anm_init(BCK_KYORO, 10.0, EMODE_LOOP, 1.0, -1);
                        self.m02ec[0] = 0x96;
                    } else if player_high {
                        if self.kui != 0 && self.kui_health == 3 {
                            self.gohma_state = 10;
                        } else {
                            self.gohma_state = 0x14;
                        }
                    }
                    self.m5e86 = 1;
                }
            }
            2 => {
                if view < 0x6000 || self.m02ec[0] == 0 {
                    self.gohma_state = 0;
                    self.m02ec[1] = 0x28;
                }
            }
            10 => {
                self.anm_init(BCK_UEMUKI_WAIT, 20.0, EMODE_LOOP, 1.0, -1);
                self.gohma_state += 1;
                self.m02ec[0] = 300;
                self.m02ec[1] = (self.rnd_f(80.0) + 90.0) as i16;
                self.wait_engage(view, player_high, env);
            }
            0xb => {
                if self.m02ec[1] == 0 && self.m02ec[0] > 0x96 {
                    out.se(SE_CV_BTD_WAIT);
                    self.m02ec[1] = (self.rnd_f(80.0) + 90.0) as i16;
                }
                self.m5e86 = 3;
                self.m5e94 = 0x100;
                self.m5e90 = env.player_angle[ANGLE_Y].wrapping_add(0x4000);
                if !player_high || (self.kui != 0 && self.kui_health != 3) {
                    self.gohma_state = 0;
                }
                if self.m02ec[0] == 0 {
                    self.m02e4 = 1;
                    self.attack_type = 3;
                    self.gohma_state = 0;
                }
            }
            0x14 => {
                self.anm_init(BCK_YOKO_WAIT, 20.0, EMODE_LOOP, 1.0, -1);
                self.gohma_state += 1;
                self.m02ec[0] = 0x96;
                self.wait_engage(view, player_high, env);
            }
            0x15 => {
                self.m5e86 = 1;
                self.m5e94 = 0x100;
                if !player_high || (self.kui != 0 && self.kui_health == 3) {
                    self.gohma_state = 0;
                }
                if self.m02ec[0] == 0 {
                    self.m02e4 = 1;
                    self.attack_type = 4;
                    self.gohma_state = 0;
                }
            }
            _ => {}
        }
    }

    fn wait_engage(&mut self, _view: u16, _player_high: bool, _env: &Env) {}

    fn attack(&mut self, actor: &mut Actor, env: &Env, out: &mut Sink<'_>) {
        self.m02f0 = 5;
        match self.attack_type {
            0 => self.jab_attack(out),
            1 => self.punch_attack(actor, out),
            2 => self.fire_attack(out),
            3 => self.up_fire_attack(actor, env, out),
            4 => self.yoko_fire_attack(actor, env, out),
            5 => self.punch2_attack(actor, out),
            _ => {}
        }
    }

    fn search_player_angle_y(&self, actor: &Actor, env: &Env) -> i16 {
        bearing(actor.pos, env.player_pos)
    }

    fn player_view_check(&self, actor: &Actor, env: &Env) -> u16 {
        let to_player = self.search_player_angle_y(actor, env);
        let diff = actor.angle[ANGLE_Y].wrapping_sub(to_player);
        if diff < 0 {
            (-diff) as u16
        } else {
            diff as u16
        }
    }

    fn eff_off(&mut self) {
        for lane in 0..7 {
            if self.m6028[lane] > 2 {
                self.m6028[lane] = 2;
            }
        }
    }

    fn wave_set(&mut self, out: &mut Sink<'_>) {
        out.se(SE_BTD_MAGMA_MOVE);
        self.m6e7c = 0.0;
    }

    fn hahen_set(&mut self, actor: &Actor) {
        self.m6048 = actor.angle;
        let idx;
        if self.m02ea != 0 {
            idx = 10usize;
            self.m6048[1] = self.m6048[1].wrapping_add(0x4000);
        } else {
            idx = 8usize;
            self.m6048[1] = self.m6048[1].wrapping_sub(0x4000);
        }
        self.m603c = self.m02fc.get(idx).copied().unwrap_or(ZERO_XYZ);
        self.m603c[1] -= REG0_F14 + HOHMA_ROCK_DROP;
    }

    fn hahen_set2(&mut self, out: &mut Sink<'_>) {
        out.se(SE_BTD_ARM_STRIKE);
        out.se(SE_BTD_ARM_STRIKE);
    }

    fn jab_attack(&mut self, out: &mut Sink<'_>) {
        match self.gohma_state {
            0 => {
                let bck = JAB_BCK.get(self.m02e9 as usize).copied().unwrap_or(0);
                self.anm_init(bck, 5.0, EMODE_NONE, 1.0, -1);
                out.se(SE_CV_BTD_JAB);
                self.gohma_state += 1;
            }
            1 => {
                if self.is_stop() {
                    self.m02e9 += 1;
                    let bck = JAB_BCK.get(self.m02e9 as usize).copied().unwrap_or(0);
                    self.anm_init(bck, 0.0, EMODE_NONE, 1.0, -1);
                    self.gohma_state += 1;
                    let idx = if self.m02ea != 0 { 10 } else { 8 };
                    let pos = self.m02fc.get(idx).copied().unwrap_or(ZERO_XYZ);
                    out.se_at(SE_BTD_WIND_JAB, pos);
                }
            }
            2 => {
                self.m5e84 = 1;
                if self.is_stop() {
                    self.m02e9 += 1;
                    let bck = JAB_BCK.get(self.m02e9 as usize).copied().unwrap_or(0);
                    self.anm_init(bck, 0.0, EMODE_NONE, 1.0, -1);
                    self.gohma_state += 1;
                }
            }
            3 => {
                if self.is_stop() {
                    self.m02e4 = 0;
                    self.gohma_state = 0;
                    self.m02ec[0] = (HIO_M14 + self.rnd_f(50.0)) as i16;
                }
            }
            _ => {}
        }
    }

    fn punch_attack(&mut self, actor: &Actor, out: &mut Sink<'_>) {
        match self.gohma_state {
            0 => {
                let bck = PUNCH_BCK.get(self.m02e9 as usize).copied().unwrap_or(0);
                self.anm_init(bck, 5.0, EMODE_NONE, 1.0, -1);
                self.gohma_state += 1;
                out.se(SE_CV_BTD_ATTACK);
            }
            1 => {
                if self.is_stop() {
                    self.m02e9 += 1;
                    let bck = PUNCH_BCK.get(self.m02e9 as usize).copied().unwrap_or(0);
                    self.anm_init(bck, 0.0, EMODE_NONE, 1.0, -1);
                    self.gohma_state += 1;
                    self.punch_wind_se(out, SE_BTD_WIND_ATTACK);
                }
            }
            2 => {
                self.m5e84 = 2;
                if self.is_stop() {
                    self.m02e9 += 1;
                    let bck = PUNCH_BCK.get(self.m02e9 as usize).copied().unwrap_or(0);
                    self.anm_init(bck, 0.0, EMODE_NONE, 1.0, -1);
                    self.gohma_state += 1;
                    self.punch_wind_se(out, SE_BTD_ARM_STRIKE);
                    self.hahen_set(actor);
                }
            }
            3 => {
                if self.is_stop() {
                    self.m02e9 += 1;
                    let bck = PUNCH_BCK.get(self.m02e9 as usize).copied().unwrap_or(0);
                    self.anm_init(bck, 0.0, EMODE_LOOP, 1.0, -1);
                    self.m02ec[0] = (HIO_M28 + self.rnd_f(100.0)) as i16;
                    self.gohma_state += 1;
                }
            }
            4 => {
                if (self.morph.frame as i32) == 2 {
                    self.punch_wind_se(out, SE_BTD_ARM_PULLOUT_1);
                    out.se(SE_CV_BTD_JITABATA);
                }
                if self.m02ec[0] == 0 {
                    self.m02e9 += 1;
                    let bck = PUNCH_BCK.get(self.m02e9 as usize).copied().unwrap_or(0);
                    self.anm_init(bck, 5.0, EMODE_NONE, 1.0, -1);
                    self.gohma_state += 1;
                    out.se(SE_CV_BTD_AFT_JITABATA);
                    self.punch_wind_se(out, SE_BTD_ARM_PULLOUT_2);
                    self.m6032 = 0x17;
                    self.m6030 = 0x17;
                    self.m6034 = 3;
                }
            }
            5 => {
                if self.is_stop() {
                    self.m02e4 = 0;
                    self.gohma_state = 0;
                    self.m02ec[0] = (HIO_M14 + self.rnd_f(50.0)) as i16;
                }
            }
            _ => {}
        }
        if self.gohma_state == 3 || self.gohma_state == 4 {
            self.m5e7c = (self.m02ea + 2) as u8;
        }
    }

    fn punch_wind_se(&mut self, out: &mut Sink<'_>, id: u32) {
        let idx = if self.m02ea != 0 { 10 } else { 8 };
        let pos = self.m02fc.get(idx).copied().unwrap_or(ZERO_XYZ);
        out.se_at(id, pos);
    }

    fn punch2_attack(&mut self, actor: &Actor, out: &mut Sink<'_>) {
        match self.gohma_state {
            0 => {
                let bck = PUNCH_BCK.get(self.m02e9 as usize).copied().unwrap_or(0);
                self.anm_init(bck, 5.0, EMODE_NONE, 1.0, -1);
                self.gohma_state += 1;
                out.se(SE_CV_BTD_ATTACK);
            }
            1 => {
                if self.is_stop() {
                    self.m02e9 += 1;
                    let bck = PUNCH_BCK.get(self.m02e9 as usize).copied().unwrap_or(0);
                    self.anm_init(bck, 0.0, EMODE_NONE, 1.0, -1);
                    self.gohma_state += 1;
                    self.punch_wind_se(out, SE_BTD_WIND_ATTACK);
                }
                self.m5e92 = REG0_S2.wrapping_add(0xf);
                self.m5e94 = REG0_S3.wrapping_add(0xc00);
            }
            2 => {
                self.m5e84 = 2;
                if self.is_stop() {
                    self.m02e9 += 1;
                    let bck = PUNCH_BCK.get(self.m02e9 as usize).copied().unwrap_or(0);
                    self.anm_init(bck, 0.0, EMODE_NONE, 1.0, -1);
                    self.gohma_state += 1;
                    self.punch_wind_se(out, SE_BTD_ARM_STRIKE);
                    self.hahen_set(actor);
                }
            }
            3 => {
                if self.is_stop() {
                    self.m02e9 += 1;
                    let bck = PUNCH_BCK.get(self.m02e9 as usize).copied().unwrap_or(0);
                    self.anm_init(bck, 0.0, EMODE_NONE, 1.0, -1);
                    self.m02ec[0] = (self.rnd_f(80.0) + 80.0) as i16;
                    self.gohma_state += 1;
                }
            }
            4 => {
                if self.is_stop() {
                    self.m02e9 += 1;
                    let bck = PUNCH_BCK.get(self.m02e9 as usize).copied().unwrap_or(0);
                    self.anm_init(bck, 5.0, EMODE_NONE, 1.0, -1);
                    self.gohma_state += 1;
                    out.se(SE_CV_BTD_AFT_JITABATA);
                    self.punch_wind_se(out, SE_BTD_ARM_PULLOUT_2);
                    self.m6032 = 0x17;
                    self.m6030 = 0x17;
                    self.m6034 = 3;
                }
            }
            5 => {
                if self.is_stop() {
                    if HIO_M10 != 4 {
                        self.m02e4 = 0;
                        self.gohma_state = 0;
                        self.m02ec[0] = 0;
                    } else {
                        self.gohma_state = 0;
                        self.m02e9 = 5;
                    }
                }
            }
            _ => {}
        }
    }

    fn fire_attack(&mut self, out: &mut Sink<'_>) {
        self.m5e86 = 2;
        match self.gohma_state {
            0 => {
                self.anm_init(BCK_FATTACK1, 5.0, EMODE_NONE, 1.0, -1);
                self.gohma_state += 1;
                self.m02ec[0] = 0x28;
                self.m6e7c = 0.0;
            }
            1 => {
                if self.m02ec[0] == REG0_S6.wrapping_add(0x10) {
                    self.hahen_set2(out);
                }
                if self.m02ec[0] == 0x1f {
                    out.se(SE_CV_BTD_BEF_FIRE);
                }
                if self.is_stop() {
                    self.anm_init(BCK_FATTACK2, 0.0, EMODE_NONE, 1.0, -1);
                    self.gohma_state += 1;
                }
            }
            2 => {
                if self.is_stop() {
                    if self.phase2 != 0 {
                        self.gohma_state = 6;
                        self.anm_init(BCK_HASAMI_ATTACK, 0.0, EMODE_NONE, 1.0, -1);
                        out.se(SE_MAGTAIL_ATTACK);
                    } else {
                        self.anm_init(BCK_FATTACK3, 0.0, EMODE_LOOP, 1.0, -1);
                        self.m02ec[0] = (HIO_M2C as i32 * 9) as i16;
                        self.gohma_state = 3;
                    }
                }
            }
            3 => {
                self.kankyo_state = 1;
                out.se_at(SE_BTD_HIFUKI_BEF, self.eye_pos);
                if self.m02ec[0] > 9 {
                    self.m6028[1] = 1;
                }
                self.m6028[0] = REG0_S6.wrapping_add(0x14);
                if self.m02ec[0] == 0 {
                    if self.phase2 != 0 {
                        self.anm_init(BCK_HASAMI_ATTACK, 0.0, EMODE_NONE, 1.0, -1);
                        self.gohma_state = 5;
                        out.se(SE_MAGTAIL_ATTACK);
                    } else {
                        self.anm_init(BCK_FATTACK4, 0.0, EMODE_NONE, 1.0, -1);
                        self.gohma_state += 1;
                        self.m02ec[0] = REG0_S3.wrapping_add(0x19);
                        out.se_at(SE_BTD_WIND_BEF_FIRE, self.eye_pos);
                    }
                }
            }
            4 => {
                self.m602c = 0xc;
                self.m602e = 3;
                if self.is_stop() {
                    self.m02e4 = 0;
                    self.gohma_state = 0;
                    self.m02ec[0] = (HIO_M14 + self.rnd_f(50.0)) as i16;
                }
                if self.m02ec[0] <= 1 {
                    self.kankyo_state = 2;
                    out.se_at(SE_BTD_HIFUKI, self.eye_pos);
                    self.fire_pos = [0.0, self.m5e80 * -200.0, self.m5e80 * 1000.0];
                    self.fire_radius = self.m5e80 * 200.0;
                    self.m5e80 += 0.1;
                    if self.m5e80 > 1.0 {
                        self.m5e80 = 0.0;
                    }
                } else {
                    self.m5e80 = 0.0;
                }
            }
            5 => {
                self.m5e84 = 3;
                if self.is_stop() {
                    self.anm_init(BCK_FATTACK4, 0.0, EMODE_NONE, 1.0, -1);
                    self.gohma_state = 4;
                    self.m02ec[0] = REG0_S3.wrapping_add(0x19);
                    out.se_at(SE_BTD_WIND_BEF_FIRE, self.eye_pos);
                }
            }
            6 => {
                self.m5e84 = 3;
                if self.is_stop() {
                    self.anm_init(BCK_FATTACK3, 15.0, EMODE_LOOP, 1.0, -1);
                    self.m02ec[0] = ((HIO_M2C as i32) << 2) as i16;
                    self.gohma_state = 3;
                }
            }
            _ => {}
        }
        if self.gohma_state >= 2 || self.m02ec[0] < 0x10 {
            self.m5e7c = 1;
        }
    }

    fn up_fire_attack(&mut self, _actor: &mut Actor, env: &Env, out: &mut Sink<'_>) {
        self.m5e86 = 3;
        match self.gohma_state {
            0 => {
                self.anm_init(BCK_UE_KAEN1, 5.0, EMODE_NONE, 1.0, -1);
                self.gohma_state += 1;
                self.m02ec[0] = 0x28;
            }
            1 => {
                if self.is_stop() {
                    self.anm_init(BCK_UE_KAEN2, 1.0, EMODE_LOOP, 1.0, -1);
                    self.m02ec[0] = (HIO_M2C as i32 * 9) as i16;
                    self.gohma_state += 1;
                }
            }
            2 => {
                self.kankyo_state = 1;
                out.se_at(SE_BTD_HIFUKI_BEF, self.eye_pos);
                self.m6028[1] = 1;
                self.m6028[0] = REG0_S6.wrapping_add(0x14);
                if self.m02ec[0] == 0 {
                    self.anm_init(BCK_UE_KAEN3, 5.0, EMODE_NONE, 1.0, -1);
                    self.gohma_state += 1;
                    self.m02ec[0] = REG0_S3.wrapping_add(0x19);
                    out.se_at(SE_BTD_WIND_BEF_FIRE, self.eye_pos);
                }
            }
            3 => {
                self.m602c = 0xc;
                self.m602e = 3;
                if self.m02ec[0] == 0 {
                    self.kankyo_state = 2;
                    out.se_at(SE_BTD_HIFUKI, self.eye_pos);
                    self.fire_pos = [0.0, self.m5e80 * -200.0, self.m5e80 * 1000.0];
                    self.fire_radius = self.m5e80 * 200.0;
                    self.m5e80 += 0.1;
                    if self.m5e80 > 1.0 {
                        self.m5e80 = 0.0;
                    }
                } else {
                    self.m5e80 = 0.0;
                }
                if self.is_stop() {
                    self.m02e4 = 0;
                    self.gohma_state = 0;
                    self.m02ec[0] = (HIO_M14 + self.rnd_f(50.0)) as i16;
                }
            }
            _ => {}
        }
        if env.player_pos.y < REG0_F5 + 400.0 || (self.kui != 0 && self.kui_health != 3) {
            self.m02e4 = 0;
            self.gohma_state = 0;
            self.m02ec[0] = (HIO_M14 + self.rnd_f(50.0)) as i16;
        }
    }

    fn yoko_fire_attack(&mut self, actor: &Actor, env: &Env, out: &mut Sink<'_>) {
        self.m5e86 = 1;
        match self.gohma_state {
            0 => {
                self.anm_init(BCK_YOKO_KAEN1, 5.0, EMODE_NONE, 1.0, -1);
                self.gohma_state += 1;
                self.m02ec[0] = 0x28;
            }
            1 => {
                if self.is_stop() {
                    self.anm_init(BCK_YOKO_KAEN2, 1.0, EMODE_LOOP, 1.0, -1);
                    self.m02ec[0] = (HIO_M2C as i32 * 7) as i16;
                    self.gohma_state += 1;
                }
            }
            2 => {
                self.kankyo_state = 1;
                out.se_at(SE_BTD_HIFUKI_BEF, self.eye_pos);
                self.m6028[1] = 1;
                self.m6028[0] = REG0_S6.wrapping_add(0x14);
                if self.m02ec[0] == 0 {
                    self.anm_init(BCK_YOKO_KAEN3, 5.0, EMODE_NONE, 1.0, -1);
                    self.gohma_state += 1;
                    self.m02ec[0] = REG0_S3.wrapping_add(0x19);
                    out.se_at(SE_BTD_WIND_BEF_FIRE, self.eye_pos);
                }
            }
            3 => {
                let diff = env.player_eye - self.eye3();
                let flat = (diff.x * diff.x + diff.z * diff.z).sqrt();
                self.m5e8c = super::walk::to_s16(diff.y.atan2(flat))
                    .wrapping_neg()
                    .wrapping_sub(0x9c4);
                self.m602c = 0xc;
                self.m602e = 3;
                if self.m02ec[0] == 0 {
                    self.kankyo_state = 2;
                    out.se_at(SE_BTD_HIFUKI, self.eye_pos);
                    self.fire_pos = [0.0, self.m5e80 * -200.0, self.m5e80 * 1000.0];
                    self.fire_radius = self.m5e80 * 200.0;
                    self.m5e80 += 0.1;
                    if self.m5e80 > REG0_F9 + 1.0 {
                        self.m5e80 = 0.0;
                    }
                } else {
                    self.m5e80 = 0.0;
                }
                if self.is_stop() {
                    self.m02e4 = 0;
                    self.gohma_state = 0;
                    self.m02ec[0] = (HIO_M14 + self.rnd_f(50.0)) as i16;
                }
                let _ = actor;
            }
            _ => {}
        }
        if env.player_pos.y < REG0_F5 + 400.0 || (self.kui != 0 && self.kui_health == 3) {
            self.m02e4 = 0;
            self.gohma_state = 0;
            self.m02ec[0] = (HIO_M14 + self.rnd_f(50.0)) as i16;
        }
        self.m5e94 = 0x200;
    }

    fn startdemo(&mut self, actor: &mut Actor, env: &Env, out: &mut Sink<'_>) {
        self.m02f0 = 30;
        actor.angle[ANGLE_Y] = self.search_player_angle_y(actor, env);
        let local = [
            env.player_pos.x - DEMO_PLAYER_OFFSET[0],
            env.player_pos.y - DEMO_PLAYER_OFFSET[1],
            env.player_pos.z - DEMO_PLAYER_OFFSET[2],
        ];
        match self.gohma_state {
            0 => {
                if self.m6e16 == 0 {
                    self.kankyo_state = 6;
                    self.m6e88 = 0;
                    actor.pos.y = -10000.0;
                    if self.demo_mode {
                        self.m6e16 = 1;
                    } else if abs3(local[0], local[1], local[2]) > REG0_F13 + DEMO_MIN_DIST {
                        self.m6e16 = 0;
                    }
                }
            }
            1 => {
                self.kankyo_state = 8;
                self.anm_init(BCK_SSTART, 1.0, EMODE_NONE, 1.0, -1);
                actor.pos.y = self.home.y;
                self.gohma_state = 2;
                out.se(SE_CV_BTD_ENTER);
            }
            2 => {
                if self.m6e1a <= 0x46 {
                    self.kankyo_state = 8;
                }
                if self.m6e1a == 0x28 || self.m6e1a == 0xDC {
                    out.se(SE_CV_BTD_AFT_DAMAGE);
                }
                if self.m6e1a == 0xD7 {
                    self.wave_set(out);
                    self.m6e48 = 40.0;
                    self.hahen_set2(out);
                }
                if self.m6e1a >= 0x46 && self.m6e1a <= 0xC3 {
                    self.m602e = 3;
                    if self.m6e1a >= 0x5A {
                        out.se(SE_BTD_HIFUKI);
                        self.kankyo_state = 2;
                    }
                }
                if self.m6e1a >= 0x46 {
                    self.m602c = 3;
                }
                if self.is_stop() && REG0_F7 == 0.0 {
                    self.m6e16 = 0x96;
                    self.m02e4 = 0;
                    self.gohma_state = 0;
                    self.m02ec[0] = (self.rnd_f(50.0) + 70.0) as i16;
                    if env.demo_stage {
                        out.bgm(BGM_PAST_GOMA);
                    } else {
                        out.bgm(BGM_GOMA);
                    }
                    self.demo_mode = true;
                }
            }
            _ => {}
        }
    }

    fn damage(&mut self, _actor: &mut Actor, _env: &Env, out: &mut Sink<'_>) -> bool {
        let mut state = 1;
        self.m02f0 = 5;
        if self.gohma_state >= 10 {
            self.action_state = 10;
        }
        if self.gohma_state <= 11 {
            state = 0;
        }
        let mut fall = true;
        let mut steps = 0;
        while fall && steps < 4 {
            fall = false;
            steps += 1;
            self.damage_step(out, &mut fall);
        }
        state != 0
    }

    

    fn damage_step(&mut self, out: &mut Sink<'_>, fall: &mut bool) {
        match self.gohma_state {
            0 => {
                self.anm_init(BCK_DAMAGE1, 5.0, EMODE_NONE, 1.0, -1);
                self.gohma_state += 1;
                *fall = true;
            }
            1 => {
                if self.is_stop() {
                    self.anm_init(BCK_DAMAGE2, 1.0, EMODE_LOOP, 1.0, -1);
                    self.stun_timer = HIO_M0C;
                    self.gohma_state += 1;
                }
            }
            2 => {
                if self.stun_timer == 0 {
                    self.damage_release();
                }
            }
            3 => {
                if self.is_stop() {
                    self.anm_init(BCK_DAMAGE2, 1.0, EMODE_LOOP, 1.0, -1);
                    self.gohma_state = 2;
                }
            }
            10 => {
                self.anm_init(BCK_DAMAGE3, 5.0, EMODE_NONE, 1.0, -1);
                self.gohma_state += 1;
                out.se(SE_CV_BTD_AFT_DAMAGE);
                *fall = true;
            }
            0xb => {
                if self.is_stop() {
                    self.damage_release();
                }
            }
            0x14 => {
                self.anm_init(BCK_SIZUMU1, 3.0, EMODE_NONE, 1.0, -1);
                self.gohma_state += 1;
                self.m6e7c = 0.0;
            }
            0x15 => {
                if self.is_stop() {
                    self.anm_init(BCK_SIZUMU2, 1.0, EMODE_LOOP, 1.0, -1);
                    self.m02ec[0] = REG0_S7.wrapping_add(0x14).wrapping_mul(5);
                    self.gohma_state += 1;
                    *fall = true;
                }
            }
            0x16 => {
                if self.m02ec[0] == REG0_S5.wrapping_add(0x28) {
                    self.m6190 += 1;
                    out.se(SE_BTD_BRK_SHIELD_S);
                    if HIO_M06 != 0 {
                        self.m6190 = 3;
                    }
                    if HIO_M07 != 0 && self.m6190 >= 3 {
                        self.m6190 = 2;
                    }
                }
                if self.m02ec[0] == 0 {
                    if self.m6190 >= 3 {
                        self.gohma_state = 0x28;
                    } else {
                        self.anm_init(BCK_SIZUMU3, 15.0, EMODE_NONE, 1.0, -1);
                        self.gohma_state += 1;
                        self.static_center_pos = [0.0, 0.0, 0.0];
                        out.se_at(SE_BTD_INTO_MAGMA_N, self.static_center_pos);
                        out.se(SE_CV_BTD_WAIT);
                        self.m6e7c = 0.0;
                        self.m02ec[0] = 0x96;
                    }
                }
            }
            0x17 => {
                if self.m02ec[0] == 1 || self.m02ec[0] == 0x46 {
                    self.m6e7c = 0.0;
                }
                if self.is_stop() {
                    self.gohma_state = 0x1e;
                    self.m02ec[0] = 0;
                }
            }
            0x1e => {
                self.m6e88 = 0;
                if self.m02ec[0] == 0 {
                    self.anm_init(BCK_MOTIAGE1, 1.0, EMODE_NONE, 1.0, -1);
                    self.gohma_state += 1;
                    self.m02f6 = 0;
                    out.se_at(SE_BTD_ENT_BLAST, self.static_center_pos);
                    out.se(SE_CV_BTD_ENTER);
                    self.m02ec[0] = 0x3c;
                    self.m6e7c = 0.0;
                    self.kankyo_timer = 0x14;
                }
            }
            0x1f => {
                if self.m02ec[0] == 0x37 {
                    self.sibuki_set();
                }
                if self.m02ec[0] == 1 {
                    self.m6e7c = 0.0;
                }
                if self.is_stop() {
                    self.anm_init(BCK_MOTIAGE2, 1.0, EMODE_NONE, 1.0, -1);
                    self.gohma_state += 1;
                    out.se_at(SE_BTD_LIFT_UP, self.static_center_pos);
                    self.m6e7c = 0.0;
                }
            }
            0x20 => {
                self.m6e88 = 0;
                if self.is_stop() {
                    self.anm_init(BCK_MOTIAGE3, 3.0, EMODE_NONE, 1.0, -1);
                    out.se_at(SE_BTD_JUMP_DOWN, ZERO_XYZ);
                    self.gohma_state += 1;
                    self.m02ec[0] = 0x1e;
                }
            }
            0x21 => {
                if self.m02ec[0] == 0x14 {
                    out.se_at(SE_BTD_LANDING, self.static_center_pos);
                    self.m6e7c = 0.0;
                    self.sibuki_set();
                }
                if self.is_stop() {
                    self.damage_release();
                }
            }
            0x28 => {
                self.anm_init(BCK_WARERU1, 3.0, EMODE_NONE, 1.0, -1);
                self.gohma_state += 1;
                out.se(SE_BTD_FALL_FWD);
                out.stop_bgm(30);
                self.m02ec[0] = 0x96;
                self.m6e7c = 0.0;
                *fall = true;
            }
            0x29 => {
                self.damage_fall(out);
                if self.m02ec[0] == 0x65 {
                    out.se(SE_BTD_GET_UP);
                    out.se(SE_CV_BTD_ENTER);
                }
                if self.is_stop() {
                    out.se(SE_BTD_BRK_SHIELD_L);
                    out.se(SE_CV_BTD_AFT_DAMAGE);
                    self.phase2 = 1;
                    self.anm_init(BCK_WARERU2, 0.0, EMODE_NONE, 1.0, -1);
                    self.gohma_state += 1;
                }
            }
            0x2a => {
                if self.is_stop() {
                    self.damage_release();
                }
            }
            _ => {}
        }
    }

    fn damage_release(&mut self) {
        self.m02e4 = 0;
        self.gohma_state = 0;
        self.m02ec[0] = (self.rnd_f(50.0) + 50.0) as i16;
    }

    fn damage_fall(&mut self, out: &mut Sink<'_>) {
        if self.m02ec[0] == REG0_S5.wrapping_add(0x26) {
            self.dr2_4ba = 10;
            if self.demo_stage_bgm() {
                out.bgm(BGM_PAST_GOMA_2);
            } else {
                out.bgm(BGM_GOMA_2);
            }
            self.spawn_hahen();
        }
    }

    fn demo_stage_bgm(&self) -> bool {
        false
    }

    fn spawn_hahen(&mut self) {
        for idx in 0..self.hahen.len() {
            if self.hahen[idx].m04 == 0 {
                let joint = if idx < 5 { 2 } else { self.rnd_f(18.999) as usize };
                self.hahen[idx].m04 = 1;
                let src = self.m02fc.get(joint).copied().unwrap_or(ZERO_XYZ);
                self.hahen[idx].m08 = [
                    src[0] + self.rnd_fx(REG0_F2 + 50.0),
                    src[1] + self.rnd_fx(REG0_F2 + 50.0),
                    src[2] + self.rnd_fx(REG0_F2 + 50.0),
                ];
                let yaw = self.rnd_fx(2000.0) as i16;
                self.hahen[idx].m22 = self.rnd_u16().wrapping_add(yaw);
                let local = [
                    REG0_F0 + (self.rnd_f(5.0) + 20.0),
                    REG0_F1 + (self.rnd_f(5.0) + 20.0),
                    0.0,
                ];
                self.hahen[idx].m14 = mtx_position_y(self.rnd_u16(), local);
                self.hahen[idx].m22 = self.rnd_u16();
                self.hahen[idx].m20 = self.rnd_u16();
            }
        }
    }

    fn demo_camera(&mut self, actor: &mut Actor, env: &Env, out: &mut Sink<'_>) {
        self.m6e1a = self.m6e1a.wrapping_add(1);
        self.m6e18 = self.m6e18.wrapping_add(1);
        match self.m6e16 {
            0 => {}
            1 => {
                self.kankyo_state = 6;
                self.m6e16 += 1;
                self.m6e3c = REG0_F12 + 50.0;
                self.m6e1c = [282.0, 2775.0, -582.0];
                self.m6e28 = [-1200.0, 0.0, 660.0];
                self.m6e1a = 0;
                self.m6e18 = 0;
                self.m6e40 = 0.0;
                self.static_center_pos = [0.0, 0.0, 0.0];
                self.demo_camera_step2(out);
            }
            2 => self.demo_camera_step2(out),
            3 => self.demo_camera_step3(out),
            4 => self.demo_camera_step4(out),
            5 => self.demo_camera_step5(out),
            100 => {
                self.m6e3c = REG0_F12 + 50.0;
                self.m6e1c = self.eye_pos;
            }
            101 => {
                add_calc2(&mut self.m6e3c, REG0_F4 + 50.0, 0.5, 3.0);
                add_calc2(&mut self.m6e28[0], self.eye_pos[0] * 0.3, 0.2, 200.0);
                let mut target_y = (self.eye_pos[1] + REG0_F5 * 0.1) - 500.0;
                if target_y < REG0_F6 + 500.0 {
                    target_y = REG0_F6 + 500.0;
                }
                add_calc2(&mut self.m6e28[1], target_y, 0.2, 200.0);
                add_calc2(&mut self.m6e28[2], self.eye_pos[2] * 0.3, 0.2, 200.0);
                let local = [
                    REG0_F7 * 0.1 + -500.0,
                    REG0_F8 * 0.1 + 70.0,
                    REG0_F9 * 0.1 + 2000.0,
                ];
                let want = mtx_position_y(actor.angle[ANGLE_Y], local);
                add_calc2(&mut self.m6e1c[0], want[0], 0.1, 50.0);
                add_calc2(&mut self.m6e1c[1], want[1], 0.1, 50.0);
                add_calc2(&mut self.m6e1c[2], want[2], 0.1, 50.0);
                if self.m6e1a >= 0x140 {
                    self.kankyo_state = 3;
                }
            }
            102 => {
                if self.m6e1a < 0x3c {
                    self.kankyo_state = 4;
                }
                if self.m6e1a == 0x5a {
                    self.m6e15 = 1;
                }
                if self.m6e1a > REG0_S2.wrapping_add(0x5a) {
                    add_calc2(
                        &mut self.m6e28[0],
                        self.eye_pos[0],
                        self.m6e40 * 0.2,
                        self.m6e40 * 200.0,
                    );
                    add_calc2(
                        &mut self.m6e28[1],
                        self.eye_pos[1] + REG0_F5 * 0.1,
                        self.m6e40 * 0.5,
                        self.m6e40 * 500.0,
                    );
                    add_calc2(
                        &mut self.m6e28[2],
                        self.eye_pos[2],
                        self.m6e40 * 0.2,
                        self.m6e40 * 200.0,
                    );
                    add_calc2(&mut self.m6e40, 1.0, 1.0, 0.1);
                }
                let local = [
                    REG0_F7 * 0.1 + -500.0,
                    REG0_F8 * 0.1 + 70.0,
                    REG0_F9 * 0.1 + 2000.0,
                ];
                let want = mtx_position_y(actor.angle[ANGLE_Y], local);
                add_calc2(&mut self.m6e1c[0], want[0], 0.1, 50.0);
                add_calc2(&mut self.m6e1c[1], want[1], 0.1, 50.0);
                add_calc2(&mut self.m6e1c[2], want[2], 0.1, 50.0);
                let head = [
                    REG0_F10 * 0.1 + -250.0,
                    REG0_F11 * 0.1 + 100.0,
                    REG0_F12 * 0.1 + 1600.0,
                ];
                self.m6e5c = mtx_position_y(actor.angle[ANGLE_Y], head);
            }
            103 => {
                self.m6e16 += 1;
                self.m6e1a = 0;
                self.m6e40 = 0.0;
                self.m6e28 = [0.0, 0.0, 0.0];
                self.m6e3c = 55.0;
                self.m6e36 = -6000;
                self.m6e34 = -11000;
                self.m6e44 = 2800.0;
                self.demo_camera_step4(out);
            }
            104 => self.demo_camera_step4(out),
            150 | 151 => {
                self.m6e16 = 0;
            }
            _ => {}
        }
        if self.m6e16 != 0 {
            let fx = self.m6e48 * super::stream::sin_s(self.m6e1a.wrapping_mul(0x3300));
            let fy = self.m6e48 * super::stream::cos_s(self.m6e1a.wrapping_mul(0x3000));
            self.m6e1c[0] += fx;
            self.m6e1c[1] += fy;
            self.m6e28[0] += fx;
            self.m6e28[1] += fy;
            add_calc0(&mut self.m6e48, 1.0, REG0_F16 + 2.0);
            self.demo_face(env);
        }
    }

    fn demo_face(&mut self, env: &Env) {
        if self.m6e16 < 100 {
            return;
        }
        let _ = env;
    }

    fn demo_camera_step2(&mut self, out: &mut Sink<'_>) {
        self.kankyo_state = 6;
        if self.m6e1a > 0x32 {
            self.m6e1c[1] -= REG0_F6 + DEMO_STEP_UP;
        }
        if self.m6e1a != 0x6F {
            return;
        }
        self.m6e1a = 0;
        self.m6e16 = 3;
        self.m6e1c = [-1728.0, 111.0, 844.0];
        self.m6e28 = [-1397.0, 110.0, 685.0];
        self.demo_camera_step3(out);
    }

    fn demo_camera_step3(&mut self, out: &mut Sink<'_>) {
        self.kankyo_state = 6;
        if self.m6e1a > 0x32 {
            add_calc2(&mut self.m6e28[0], -1536.0, 0.1, self.m6e40 * 139.0);
            add_calc2(&mut self.m6e28[1], 411.0, 0.1, self.m6e40 * 301.0);
            add_calc2(&mut self.m6e28[2], 752.0, 0.1, self.m6e40 * 67.0);
            add_calc2(&mut self.m6e40, 1.0, 0.1, 0.005);
        }
        if self.m6e1a >= 0xb4 && REG0_S6 == 0 {
            self.m6e1a = 0;
            self.m6e16 = 4;
            self.m6e40 = 0.0;
            self.m6e1c = [-2501.0, 418.0, -32.0];
            self.m6e28 = [-2139.0, 373.0, 17.0];
            self.m6e3c = 60.0;
            self.demo_camera_step4(out);
        } else if self.m6e1a >= 0x8C {
            self.kankyo_state = 7;
            self.m6e48 = 20.0;
            out.se(SE_ATM_JINARI);
        }
    }

    fn demo_camera_step4(&mut self, out: &mut Sink<'_>) {
        self.kankyo_state = 7;
        self.m6e48 = 15.0;
        out.se(SE_ATM_JINARI);
        add_calc2(&mut self.m6e1c[0], -1763.0, 0.1, self.m6e40 * 738.0);
        add_calc2(&mut self.m6e1c[1], 167.0, 0.1, self.m6e40 * 251.0);
        add_calc2(&mut self.m6e1c[2], 812.0, 0.1, self.m6e40 * 844.0);
        add_calc2(&mut self.m6e28[0], -1430.0, 0.1, self.m6e40 * 709.0);
        add_calc2(&mut self.m6e28[1], 123.0, 0.1, self.m6e40 * 250.0);
        add_calc2(&mut self.m6e28[2], 663.0, 0.1, self.m6e40 * 646.0);
        add_calc2(&mut self.m6e40, REG0_F10 + 0.01, 0.1, 0.001);
        if self.m6e1a > 0x28 {
            add_calc2(&mut self.m6e84, 0.0, 0.1, 1.0);
        }
        if self.m6e1a == 0x1e {
            out.se_at(SE_BTD_ENT_BUBBLE, self.static_center_pos);
        }
        if self.m6e1a == 0x50 {
            self.wave_set(out);
        }
        if self.m6e1a == 0x91 {
            self.wave_set(out);
        }
        if self.m6e1a == 0x96 {
            self.gohma_state = 1;
            self.m6e1a = 0;
            self.m6e16 = 5;
            self.m6e40 = 0.0;
            self.m6e84 = 0.0;
            out.se_at(SE_BTD_ENT_BLAST, self.static_center_pos);
            self.kankyo_state = 8;
        }
    }

    fn demo_camera_step5(&mut self, out: &mut Sink<'_>) {
        if self.m6e1a == 0x32 {
            self.wave_set(out);
        }
        if self.m6e1a >= 0xA {
            if self.m6e1a < 0xcd {
                add_calc2(
                    &mut self.m6e28[0],
                    self.eye_pos[0] * 0.3,
                    0.2,
                    self.m6e40 * 200.0,
                );
                let mut target_y = (self.eye_pos[1] + REG0_F5 * 0.1) - 100.0;
                if target_y < REG0_F6 + 300.0 {
                    target_y = REG0_F6 + 300.0;
                }
                add_calc2(&mut self.m6e28[1], target_y, 0.2, self.m6e40 * 200.0);
                add_calc2(
                    &mut self.m6e28[2],
                    self.eye_pos[2] * 0.3,
                    0.2,
                    self.m6e40 * 200.0,
                );
            } else {
                if self.m6e1a == 0xcd {
                    self.m6e40 = 0.0;
                }
                add_calc2(&mut self.m6e1c[0], -1750.0, 0.2, self.m6e40 * 2000.0);
                add_calc2(&mut self.m6e1c[1], 50.0, 0.2, self.m6e40 * 2000.0);
                add_calc2(&mut self.m6e1c[2], 851.0, 0.2, self.m6e40 * 2000.0);
                add_calc2(&mut self.m6e28[0], -299.0, 0.2, self.m6e40 * 2000.0);
                add_calc2(&mut self.m6e28[1], 319.0, 0.2, self.m6e40 * 2000.0);
                add_calc2(&mut self.m6e28[2], 248.0, 0.2, self.m6e40 * 2000.0);
                if self.m6e1a >= 0xFF {
                    add_calc2(&mut self.m6e3c, 45.0, 0.2, 1.5);
                }
            }
            add_calc2(&mut self.m6e40, 1.0, 0.1, 0.01);
        }
    }

    fn end(&mut self, _actor: &mut Actor, env: &Env, out: &mut Sink<'_>) {
        self.action_state = 10;
        match self.gohma_state {
            0x32 => {
                if env.demo_stage {
                    out.se(SE_LK_B_BOSS_WARP);
                    out.stop_bgm(30);
                } else {
                    self.gohma_state = 0x33;
                    self.anm_init(BCK_DEAD, 0.0, EMODE_NONE, 1.0, -1);
                    self.m6e16 = 100;
                    self.m6e1a = 0;
                    self.m6e18 = 0;
                    self.m02ec[0] = REG0_S3.wrapping_add(0x21c);
                    out.stop_bgm(20);
                    self.wave_set(out);
                }
            }
            0x33 => self.end_death(out),
            0x34 => self.end_flight(),
            0x35 => self.end_drop(out),
            0x36 => self.end_rest(out),
            0x37 => {
                if self.m02ec[0] == 2 {
                    self.dr2_43c = 1;
                    self.m6e84 = -200.0;
                }
                if self.m02ec[0] == 1 {
                    self.m6e16 += 1;
                    self.gohma_state = 0x38;
                }
            }
            0x39 => {
                self.kankyo_state = 6;
            }
            _ => {}
        }
    }

    fn end_death(&mut self, out: &mut Sink<'_>) {
        if self.m6e1a == 3 {
            out.se(SE_CV_BTD_AFT_DAMAGE);
        }
        if self.m6e1a == 0x40 {
            out.se(SE_CV_BTD_DMG_ROCK);
            self.wave_set(out);
        }
        if self.m6e1a == 0x78 {
            out.se(SE_BTD_BECOME_BLACK);
            self.wave_set(out);
        }
        if self.m6e1a == 0xa0 {
            out.se(SE_CV_BTD_AFT_JITABATA);
        }
        if self.m6e1a == 200 {
            self.static_center_pos = [0.0, 300.0, 0.0];
            out.se_at(SE_BTD_ENDURE, self.static_center_pos);
            out.se(SE_CV_BTD_DIE);
        }
        if self.m6e1a == 0x136 {
            out.se_at(SE_BTD_BECOME_RED, self.static_center_pos);
        }
        if self.m02ec[0] == 0x65 {
            self.m6e4c = 1;
            self.m6e50 = self.eye_pos;
            self.m6e74 = [0, 0, 0];
            self.m6e16 += 1;
            self.m6e1a = 0;
            self.m6e40 = 0.0;
            self.gohma_state += 1;
            self.speed_f = 30.0;
            self.m02ec[0] = 0x46;
            out.se_at(SE_BTD_LAST_EXPLODE, self.static_center_pos);
            self.m6e48 = 30.0;
        } else if self.m02ec[0] == 0x1cc {
            self.phase2 = 2;
        }
    }

    fn end_flight(&mut self) {
        self.m6e88 = 0;
        let mut delta = [
            self.m6e5c[0] - self.m6e50[0],
            self.m6e5c[1] - self.m6e50[1],
            self.m6e5c[2] - self.m6e50[2],
        ];
        let radius = (delta[0] * delta[0] + delta[2] * delta[2]).sqrt();
        delta[1] += radius * (REG0_F14 + 3.5);
        self.m6e74[ANGLE_Y] = super::walk::to_s16(delta[0].atan2(delta[2]));
        let pitch = super::walk::to_s16(delta[1].atan2(radius));
        self.m6e74[ANGLE_X] = pitch;
        self.m6e68 = [0.0, 0.0, self.speed_f];
        self.m6e50[0] += self.m6e68[0];
        self.m6e50[1] += self.m6e68[1];
        self.m6e50[2] += self.m6e68[2];
        if self.m6e68[1] < 0.0 {
            add_calc2(&mut self.speed_f, 300.0, 1.0, 2.0);
        }
        self.eye_pos = self.m6e50;
        let gap = [
            self.m6e5c[0] - self.m6e50[0],
            self.m6e5c[1] - self.m6e50[1],
            self.m6e5c[2] - self.m6e50[2],
        ];
        if abs3(gap[0], gap[1], gap[2]) < self.speed_f * 2.0 {
            self.gohma_state = 0x35;
        }
        self.end_flight_common();
    }

    fn end_flight_common(&mut self) {
        self.m6e74[ANGLE_X] = REG0_S3.wrapping_add(1000);
        self.m6e74[ANGLE_Z] = REG0_S4.wrapping_add(0x5dc);
        self.kubi_calc();
    }

    fn end_drop(&mut self, out: &mut Sink<'_>) {
        self.m6e88 = 0;
        add_calc2(
            &mut self.m6e50[0],
            self.m6e5c[0],
            1.0,
            self.m6e68[0].abs(),
        );
        self.m6e50[1] += self.m6e68[1];
        self.m6e68[1] -= REG0_F3 + 5.0;
        if self.m6e50[1] <= self.m6e5c[1] {
            self.m6e50[1] = self.m6e5c[1];
            if self.m6e68[1] < -50.0 {
                self.m6e68[1] *= REG0_F4 + -0.3;
                self.m6e48 = 40.0;
                out.se(SE_BTD_DROP_HEAD);
                self.gohma_state = 0x36;
                self.m02ec[0] = 0x32;
                self.m6038 = 0;
            }
        }
        add_calc2(
            &mut self.m6e50[2],
            self.m6e5c[2],
            1.0,
            self.m6e68[2].abs(),
        );
        self.eye_pos = self.m6e50;
        self.end_flight_common();
    }

    fn end_rest(&mut self, out: &mut Sink<'_>) {
        self.m6e88 = 0;
        super::stream::add_calc_angle_s2(&mut self.m6e74[ANGLE_X], 0x2000, 2, 0x200);
        super::stream::add_calc_angle_s2(&mut self.m6e74[ANGLE_Z], 0x2000, 2, 0x200);
        self.m6e50[1] += self.m6e68[1];
        self.m6e68[1] -= REG0_F3 + 5.0;
        if self.m6e50[1] <= self.m6e5c[1] {
            self.m6e50[1] = self.m6e5c[1];
            self.m6e68[1] = 0.0;
        }
        self.eye_pos = self.m6e50;
        if self.m02ec[0] == 1 {
            self.m6e50[1] -= 100.0;
            out.se(SE_BOSS_EXPLODE);
            self.gohma_state = 0x37;
            self.m6e50[1] = -10000.0;
            self.m02ec[0] = 0x96;
        }
        self.kubi_calc();
    }

    fn kubi_calc(&mut self) {}

    fn sibuki_set(&mut self) {
        self.m6e14 = 0;
        for lane in 0..self.sibuki.len() {
            if self.sibuki[lane].m00 == 0 {
                self.sibuki[lane].m00 = 1;
                let yaw = self.rnd_fx(32768.0) as i16;
                let local = [0.0, self.rnd_f(50.0), REG0_F13 + (self.rnd_fx(200.0) + 300.0)];
                self.sibuki[lane].m04 = mtx_position_y(yaw, local);
                let scatter = [
                    0.0,
                    HIO_M4C + self.rnd_f(HIO_M4C * 0.5),
                    HIO_M48 + self.rnd_f(HIO_M48 * 0.5),
                ];
                let yaw2 = self.rnd_fx(32768.0) as i16;
                self.sibuki[lane].m10 = mtx_position_y(yaw2, scatter);
            }
        }
    }
}

fn mtx_position_y(yaw: i16, local: Xyz) -> Xyz {
    let (sin, cos) = super::walk::from_s16(yaw).sin_cos();
    [
        local[0] * cos + local[2] * sin,
        local[1],
        local[2] * cos - local[0] * sin,
    ]
}