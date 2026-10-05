use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{Collect, EventFlags, StoryBits, SYMBOL_DIN};
use super::intent::{DoIntent, MoveIntent};
use super::player::Player;
use super::stream::{add_calc_angle_s2, cos_s};
use super::walk::{add_calc_angle_s, from_s16, to_s16};
use crate::assets::MsgArchives;
use crate::text::driver::{Counters, TextFlags};
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgGameInfo, MsgProcs, fop_msgm_message_set,
    fop_msgm_message_set_actor,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED,
    STATUS_MSG_ENDS,
};

pub const NPC_BTSW_ACTOR: &str = "Btsw";

pub fn is_npc_btsw(name: &str) -> bool {
    name == NPC_BTSW_ACTOR
}

pub const STAFF_BTSW: &str = "Btsw";
pub const EVENT_GETMOTHERLETTER: &str = "GETMOTHERLETTER";
pub const EVENT_SHIWAKEGAME: &str = "SHIWAKEGAME";
pub const EVENT_BTSW_TALK_SUGOI: &str = "BTSW_TALK_SUGOI";

const HIO_M04: f32 = 0.0;
const HIO_MAX_HEAD_X: i16 = 8000;
const HIO_MAX_HEAD_Y: i16 = 2000;
const HIO_MAX_BACKBONE_X: i16 = 0;
const HIO_MAX_BACKBONE_Y: i16 = 8000;
const HIO_MIN_HEAD_X: i16 = -3000;
const HIO_MIN_HEAD_Y: i16 = -2000;
const HIO_MIN_BACKBONE_X: i16 = 0;
const HIO_MIN_BACKBONE_Y: i16 = -8000;
const HIO_MAX_TURN_STEP: i16 = 2000;
const HIO_MAX_HEAD_TURN_VEL: i16 = 0x640;
const HIO_ATTN_Y_OFFSET: f32 = 40.0;
const HIO_MAX_ATTN_ANGLE_Y: i32 = 0x2000;
const HIO_MAX_ATTN_DIST_XZ: f32 = 300.0;
const HIO_TIMER_START: i32 = 0xF;
const HIO_TIMER_END: i32 = 0x3C;
const HIO_MAIL_STEP: f32 = 40.0;
const HIO_STICK_HI: f32 = 0.8;
const HIO_STICK_LO: f32 = 0.75;
const HIO_SUGOI_MORF: f32 = 10.0;

const ACTION_STARTING: i8 = 0;
const ACTION_ONGOING: i8 = 1;
const ACTION_UNK_2: i8 = 2;
const ACTION_ENDING: i8 = -1;

const MODE_WAIT: i8 = 1;
const MODE_TALK: i8 = 2;
const MODE_SHIWAKE: i8 = 3;

const ORDER_NONE: i8 = 0;
const ORDER_SPEAK: i8 = 1;
const ORDER_ATTENTION: i8 = 2;
const ORDER_GETMOTHER: i8 = 3;
const ORDER_EVENT: i8 = 4;

const ANM_WAIT01: i8 = 0;
const ANM_WAIT02: i8 = 1;
const ANM_TALK01: i8 = 2;
const ANM_TALK02: i8 = 3;
const ANM_ONEGAI: i8 = 4;
const ANM_SUGOI: i8 = 5;
const ANM_SUGOI_ALT: i8 = 6;
const ANM_SIWAKE01: i8 = 9;
const ANM_SIWAKE02: i8 = 10;
const ANM_COUNT: usize = 11;

const EMODE_NONE: u8 = 0;
const EMODE_LOOP: u8 = 2;

const STATE_STOP: u8 = 1;
const STATE_LOOP: u8 = 2;

const ANM_PLAY_MODE: [u8; ANM_COUNT] = [
    EMODE_LOOP, EMODE_LOOP, EMODE_LOOP, EMODE_LOOP, EMODE_NONE, EMODE_LOOP, EMODE_LOOP, EMODE_LOOP,
    EMODE_NONE, EMODE_LOOP, EMODE_NONE,
];
const ANM_PLAY_SPEED: [f32; ANM_COUNT] = [
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
];
const ANM_MORF_FRAME: f32 = 8.0;
const ANM_FRAME_MAX: [f32; ANM_COUNT] = [
    40.0, 40.0, 40.0, 40.0, 34.0, 45.0, 45.0, 40.0, 74.0, 40.0, 49.0,
];

const BTP_FRAME_MAX: i32 = 4;
const BTP_TIMER_MIN: f32 = 30.0;
const BTP_TIMER_RANGE: f32 = 100.0;

const PLAYER_HEAD_TOP: f32 = 40.0;
const TALK_RANGE: f32 = 100.0;

const GRAVITY: f32 = -30.0;

const ATTN_EXTRA_DIST: f32 = 40.0;
const ATTN_EXTRA_DEG: f32 = 10.0;
const ATTN_DEG2S: f32 = 0x8000 as f32 / 180.0;

const NO_EVENT: i16 = -1;
const NO_STATUS: u8 = 0xFF;

const MINI_GAME_LETTER: u8 = 7;
const MINI_RUPEE_LIMIT: i16 = 25;
const SHIWAKE_REST_START: i32 = 20000;
const TIMER_GATE: i32 = 10000;
const TIMER_BEAT: i32 = 1000;
const TIMER_BEAT_BIG: i32 = 10000;
const MS_PER_FRAME: i32 = 1000 / 60;
const TIMER_DELETE_FRAMES: i32 = 2;

const STICK_A: i16 = -0x7000;
const STICK_B: i16 = -0x5000;
const STICK_C: i16 = -0x3000;
const STICK_D: i16 = -0x1000;
const STICK_E: i16 = 0x1000;
const STICK_F: i16 = 0x3000;
const STICK_G: i16 = 0x5000;
const STICK_H: i16 = 0x7000;
const STICK_X_MASK: u8 = 0x03;
const STICK_Y_MASK: u8 = 0x0C;
const F_ABS_MIN: f32 = 3.8146973e-6;
const LIB_ZERO: f32 = 8e-11;

const JNT_SCALE: i32 = 4;
const JNT_MIN_STEP: i32 = 1;
const JNT_ANG_MIN: i16 = 0x60;

const CURSOR_ROWS: usize = 2;
const CURSOR_COLS: usize = 3;
const CURSOR_SCALE_X: f32 = 0.65;
const CURSOR_SCALE_Y: f32 = 0.9;
const CURSOR_SCALE_Z: f32 = 0.5;
const CURSOR_SCALE_W: f32 = 27.0;
const CURSOR_SCALE_H: f32 = 20.0;

const AIM_POS: [[Vec3; CURSOR_COLS]; CURSOR_ROWS] = [
    [
        Vec3::new(-40.0, 772.0, 1035.0),
        Vec3::new(-139.0, 772.0, 1035.0),
        Vec3::new(-240.0, 772.0, 1035.0),
    ],
    [
        Vec3::new(-40.0, 855.0, 1035.0),
        Vec3::new(-139.0, 855.0, 1035.0),
        Vec3::new(-240.0, 855.0, 1035.0),
    ],
];
const CURSOR_POS: [[Vec3; CURSOR_COLS]; CURSOR_ROWS] = [
    [
        Vec3::new(-52.0, 804.0, 996.0),
        Vec3::new(-140.0, 804.0, 996.0),
        Vec3::new(-227.0, 804.0, 996.0),
    ],
    [
        Vec3::new(-52.0, 889.0, 996.0),
        Vec3::new(-140.0, 889.0, 996.0),
        Vec3::new(-227.0, 889.0, 996.0),
    ],
];

const CAM_CENTRE: [[Vec3; CURSOR_COLS]; CURSOR_ROWS] = [
    [
        Vec3::new(-104.0, 805.0, 875.0),
        Vec3::new(-139.0, 805.0, 882.0),
        Vec3::new(-173.0, 805.0, 875.0),
    ],
    [
        Vec3::new(-104.0, 825.0, 868.0),
        Vec3::new(-139.0, 825.0, 875.0),
        Vec3::new(-173.0, 825.0, 875.0),
    ],
];
const CAM_EYE: Vec3 = Vec3::new(-139.0, 815.0, 662.0);
const CAM_FOV: f32 = 58.0;
const CAM_SCALE: f32 = 0.25;
const CAM_STEP: f32 = 10.0;
const CAM_MIN: f32 = 1.0;

const MAIL_COUNT: usize = 3;
const MAIL_LAST: u8 = 2;
const MAIL_SPAN: i32 = 3;
const MAIL_DROP_Y: f32 = -200.0;
const MAIL_SIDE_DROP_X: f32 = -150.0;
const MAIL_YAW: i16 = -0x4000;
const MAIL_TILT: i16 = -0x4000;
const MAIL_APPEAR_SCALE: f32 = 0.25;
const MAIL_APPEAR_STEP: f32 = 30.0;
const MAIL_APPEAR_MIN: f32 = 2.5;
const MAIL_REACH: f32 = 120.0;
const MAIL_REACH_Y: f32 = 40.0;
const MAIL_APPEAR_SPEED: i16 = 4;
const MAIL_APPEAR_MAX: i16 = 0x1000;
const MAIL_THROW_SCALE: f32 = 0.5;
const MAIL_THROW_STEP: f32 = 30.0;
const MAIL_THROW_MIN: f32 = 2.0;
const MAIL_THROW_DONE: f32 = 1.0;
const MAIL_THROW_VEC_X: f32 = -40.0;
const MAIL_THROW_VEC_Y: f32 = -50.0;
const MAIL_THROW_VEC_Z: f32 = 100.0;
const MAIL_LAND_SCALE: f32 = 0.5;
const MAIL_LAND_MIN: f32 = 1.0;
const MAIL_SPIN_STEP: f32 = 0x80 as f32;
const MAIL_SPIN_BASE: f32 = 0x1000 as f32;
const MAIL_SPIN_ADD: i32 = 4000;
const MAIL_SPIN_AMP: f32 = 20.0;
const MAIL_END_SCALE: f32 = 0.25;
const MAIL_END_STEP: f32 = 5.0;
const MAIL_END_Z: f32 = -10.0;
const MAIL_FALL_GATE_Z: f32 = 995.0;
const MAIL_TILT_PIVOT_Y: f32 = 720.0;
const MAIL_CLAMP_Y: f32 = 700.0;
const MAIL_FALL_STEP: f32 = 3.0;
const MAIL_POP_Y: f32 = 0.6;
const MAIL_POP_XZ: f32 = 0.9;
const MAIL_END_TURN_SCALE: i16 = 2;
const MAIL_END_TURN_MAX: i16 = 0x800;
const MAIL_END_YAW: i16 = -0x8000;
const MAIL_FALL_RATE: f32 = 0.9;
const MAIL_SAME_SPAN: f32 = 1.25;
const MAIL_RANDOM_BASE: f32 = 2.5;
const MAIL_RANDOM_SPAN: f32 = 2.5;
const MAIL_RANDOM_THIRD: f32 = 0.3333;
const MAIL_RANDOM_TWO_THIRD: f32 = 0.6666;
const MAIL_RANDOM_HALF: f32 = 0.5;

const REG10_F0: f32 = 0.0;
const REG10_F1: f32 = 0.0;
const REG10_F2: f32 = 0.0;
const REG10_F3: f32 = 0.0;
const REG10_F4: f32 = 0.0;
const REG10_F10: f32 = 0.0;
const REG10_F11: f32 = 0.0;
const REG10_S0: i16 = 0;
const REG10_S1: i16 = 0;
const REG10_S2: i16 = 0;
const REG10_S3: i16 = 0;

const SE_MINIGAME_TIMER_30: u32 = 0x08AD;
const SE_MINIGAME_TIMER_10: u32 = 0x08AE;
const SE_LETTER_GAME_TIMER_0: u32 = 0x08AF;
const SE_LETTER_GAME_CURSOR: u32 = 0x08B0;
const SE_LETTER_GAME_OK: u32 = 0x08B1;
const SE_LETTER_GAME_NEW: u32 = 0x69A5;
const SE_LETTER_IN_BOX: u32 = 0x69A6;
const SE_LK_SW_KAZEKIRI_S: u32 = 0x2800;
const SE_MINIGAME_RIGHT: u32 = 0x08A5;
const SE_MINIGAME_WRONG: u32 = 0x08A6;

const MSG_1A91: u32 = 0x1A91;
const MSG_1A93: u32 = 0x1A93;
const MSG_1A94: u32 = 0x1A94;
const MSG_1A96: u32 = 0x1A96;
const MSG_1A97: u32 = 0x1A97;
const MSG_1A98: u32 = 0x1A98;
const MSG_1A9B: u32 = 0x1A9B;
const MSG_1A9C: u32 = 0x1A9C;
const MSG_1A9D: u32 = 0x1A9D;
const MSG_1A9E: u32 = 0x1A9E;
const MSG_1A9F: u32 = 0x1A9F;
const MSG_1AA0: u32 = 0x1AA0;
const MSG_1AA1: u32 = 0x1AA1;
const MSG_1AA2: u32 = 0x1AA2;
const MSG_1AA3: u32 = 0x1AA3;
const MSG_1AA4: u32 = 0x1AA4;
const MSG_1AA5: u32 = 0x1AA5;
const MSG_1AA7: u32 = 0x1AA7;
const MSG_1AA8: u32 = 0x1AA8;
const MSG_1AA9: u32 = 0x1AA9;
const MSG_1AAA: u32 = 0x1AAA;
const MSG_1AAB: u32 = 0x1AAB;
const MSG_1AAC: u32 = 0x1AAC;
const MSG_1AAD: u32 = 0x1AAD;
const MSG_1AB3: u32 = 0x1AB3;
const MSG_1AB4: u32 = 0x1AB4;
const MSG_1AB5: u32 = 0x1AB5;

const FLAG_GOT_LETTER: u16 = 0x2701;
const FLAG_TRIED_LETTER: u16 = 0x2702;
const FLAG_TALKED: u16 = 0x2704;
const FLAG_SENT_LETTER: u16 = 0x3104;
const REG_TRIES: u16 = 0xAB03;
const REG_8AFF: u16 = 0x8AFF;
const LETTER_BAITOS_MOM: u16 = 0xAC03;
const LETTER_NOSEND: u8 = 0;
const LETTER_READ: u8 = 3;
const REG_GOT_LETTER: u8 = 25;

const CULL_MIN: [f32; 3] = [-30.0, -30.0, -30.0];
const CULL_MAX: [f32; 3] = [30.0, 180.0, 30.0];

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = seed[0] as f32 / RND_SCALE[0]
        + seed[1] as f32 / RND_SCALE[1]
        + seed[2] as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

fn is_zero(value: f32) -> bool {
    value.abs() < LIB_ZERO
}

fn cm3d_is_zero(value: f32) -> bool {
    value.abs() < F_ABS_MIN
}

fn near_zero_square(value: Vec3) -> bool {
    value.length_squared() < 8e-11
}

fn normalize_rs(value: &mut Vec3) -> bool {
    if near_zero_square(*value) {
        return false;
    }
    *value = value.normalize();
    true
}

fn norm_zp(value: Vec3) -> Vec3 {
    if near_zero_square(value) {
        Vec3::ZERO
    } else {
        value.normalize()
    }
}

fn add_calc_pos(value: &mut Vec3, target: Vec3, scale: f32, max_step: f32, min_step: f32) -> f32 {
    if *value != target {
        let mut diff = *value - target;
        let mut step = diff.length();
        if step < min_step {
            *value = target;
        } else {
            step *= scale;
            diff *= scale;
            if !is_zero(step) {
                if step > max_step {
                    diff *= max_step / step;
                } else if step < min_step {
                    diff *= min_step / step;
                }
                *value -= diff;
            } else {
                *value = target;
            }
        }
    }
    (*value - target).length()
}

fn add_calc_pos2(value: &mut Vec3, target: Vec3, scale: f32, max_step: f32) {
    if *value != target {
        let mut diff = (*value - target) * scale;
        if diff.length() > max_step {
            diff = norm_zp(diff) * max_step;
        }
        *value -= diff;
    }
}

fn add_calc_angle_l(value: &mut i32, target: i32, scale: i32, max_step: i32, min_step: i32) {
    let diff = target.wrapping_sub(*value);
    if *value != target {
        let mut step = diff / scale;
        if step > min_step || step < -min_step {
            if step > max_step {
                step = max_step;
            }
            if step < -max_step {
                step = -max_step;
            }
            *value = value.wrapping_add(step);
        } else if diff >= 0 {
            *value = value.wrapping_add(min_step);
            if target.wrapping_sub(*value) <= 0 {
                *value = target;
            }
        } else {
            *value = value.wrapping_sub(min_step);
            if target.wrapping_sub(*value) >= 0 {
                *value = target;
            }
        }
    }
}

fn target_angle_y(lhs: Vec3, rhs: Vec3) -> i16 {
    to_s16((rhs.x - lhs.x).atan2(rhs.z - lhs.z))
}

fn target_angle_x(lhs: Vec3, rhs: Vec3) -> i16 {
    let delta = rhs - lhs;
    to_s16(delta.y.atan2((delta.x * delta.x + delta.z * delta.z).sqrt()))
}

fn distance_angle_s(x: i16, y: i16) -> i32 {
    i32::from(x.wrapping_sub(y)).abs()
}

fn calc_timer(value: &mut i16) -> i16 {
    if *value != 0 {
        *value -= 1;
    }
    *value
}

fn rot_y(angle: i16, vec: Vec3) -> Vec3 {
    let rad = from_s16(angle);
    Vec3::new(
        vec.x * rad.cos() + vec.z * rad.sin(),
        vec.y,
        vec.z * rad.cos() - vec.x * rad.sin(),
    )
}

fn rot_x(angle: i16, vec: Vec3) -> Vec3 {
    let rad = from_s16(angle);
    let cos = rad.cos();
    let sin = rad.sin();
    Vec3::new(vec.x, vec.y * cos - vec.z * sin, vec.y * sin + vec.z * cos)
}

fn aim_pos(aim_y: i8, row: i32) -> Vec3 {
    AIM_POS
        .get(usize::from(aim_y.max(0) as u8))
        .and_then(|line| line.get(usize::try_from(row).unwrap_or(1)))
        .copied()
        .unwrap_or(AIM_POS[0][1])
}

fn cursor_pos(aim_y: i8, aim_x: i8) -> Vec3 {
    CURSOR_POS
        .get(usize::from(aim_y.max(0) as u8))
        .and_then(|line| line.get(usize::from(aim_x.wrapping_add(1).max(0) as u8)))
        .copied()
        .unwrap_or(CURSOR_POS[0][1])
}

fn cam_centre(col: i8, row: i8) -> Vec3 {
    CAM_CENTRE
        .get(usize::from(col.max(0) as u8))
        .and_then(|line| line.get(usize::from(row.wrapping_add(1).max(0) as u8)))
        .copied()
        .unwrap_or(CAM_CENTRE[0][1])
}

fn event_index(list: &EventList, name: &str) -> i16 {
    list.event_index(name)
        .and_then(|index| i16::try_from(index).ok())
        .unwrap_or(NO_EVENT)
}

fn get_game_end_msg(rupees: i16) -> u32 {
    if rupees == 0 {
        MSG_1A9D
    } else if rupees == 1 {
        MSG_1A9E
    } else if rupees < MINI_RUPEE_LIMIT {
        MSG_1A9F
    } else {
        MSG_1A98
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub attribute: u8,
    pub state: u8,
    pub start: f32,
    pub end: f32,
    pub loop_point: f32,
    pub rate: f32,
    pub frame: f32,
    pub cur_morf: f32,
    pub prev_morf: f32,
    pub morf_step: f32,
}

impl Frame {
    pub fn create(end: f32) -> Self {
        Self {
            attribute: EMODE_LOOP,
            state: 0,
            start: 0.0,
            end,
            loop_point: 0.0,
            rate: 1.0,
            frame: 0.0,
            cur_morf: 1.0,
            prev_morf: 1.0,
            morf_step: 0.0,
        }
    }

    pub fn set_anm(&mut self, end: f32, mode: u8, rate: f32) {
        self.start = 0.0;
        self.end = end;
        self.loop_point = 0.0;
        self.attribute = mode;
        self.state = 0;
        self.rate = rate;
        self.frame = 0.0;
    }

    pub fn end_frame(&self) -> f32 {
        self.end
    }

    pub fn is_stop(&self) -> bool {
        self.state & STATE_STOP != 0 || self.rate == 0.0
    }

    pub fn update(&mut self) {
        if self.cur_morf < 1.0 {
            self.prev_morf = self.cur_morf;
            self.cur_morf = (self.cur_morf + self.morf_step).min(1.0);
        }
        self.state = 0;
        self.frame += self.rate;
        if self.attribute == EMODE_NONE {
            if self.frame < self.start {
                self.frame = self.start;
                self.rate = 0.0;
                self.state |= STATE_STOP;
            }
            if self.frame >= self.end {
                self.frame = self.end - 0.001;
                self.rate = 0.0;
                self.state |= STATE_STOP;
            }
        } else if self.attribute == EMODE_LOOP {
            while self.frame < self.start {
                self.state |= STATE_LOOP;
                if self.loop_point - self.start <= 0.0 {
                    break;
                }
                self.frame += self.loop_point - self.start;
            }
            while self.frame >= self.end {
                self.state |= STATE_LOOP;
                if self.end - self.loop_point <= 0.0 {
                    break;
                }
                self.frame -= self.end - self.loop_point;
            }
        }
    }

    pub fn check_pass(&self, pass: f32) -> bool {
        let cur = self.frame;
        let mut next = cur + self.rate;
        if self.attribute == EMODE_NONE {
            if next < self.start {
                next = self.start;
            }
            if next >= self.end {
                next = self.end - 0.001;
            }
            if cur <= next {
                cur <= pass && pass < next
            } else {
                next <= pass && pass < cur
            }
        } else if cur < self.start {
            while next < self.start {
                if self.loop_point - self.start <= 0.0 {
                    break;
                }
                next += self.loop_point - self.start;
            }
            next <= pass && pass < self.loop_point
        } else if self.end <= cur {
            while next >= self.end {
                if self.end - self.loop_point <= 0.0 {
                    break;
                }
                next -= self.end - self.loop_point;
            }
            self.loop_point <= pass && pass < next
        } else if next < self.start {
            while next < self.start {
                if self.loop_point - self.start <= 0.0 {
                    break;
                }
                next += self.loop_point - self.start;
            }
            (self.start <= pass && pass < cur) || (next <= pass && pass < self.loop_point)
        } else if self.end <= next {
            while next >= self.end {
                if self.end - self.loop_point <= 0.0 {
                    break;
                }
                next -= self.end - self.loop_point;
            }
            (cur <= pass && pass < self.end) || (self.loop_point <= pass && pass < next)
        } else if cur <= next {
            cur <= pass && pass < next
        } else {
            next <= pass && pass < cur
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Wait,
    DummyEvent,
    ShiwakeGame,
    GetDemo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MailProc {
    Dummy,
    Appear,
    Wait,
    Throw,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MailState {
    pub seed: [i32; 3],
    pub same_count: u8,
    pub no_buff: u8,
}

impl MailState {
    pub fn create() -> Self {
        Self {
            seed: [RND_SEED; 3],
            same_count: 0,
            no_buff: 0,
        }
    }

    pub fn rnd_f(&mut self, range: f32) -> f32 {
        rnd(&mut self.seed) * range
    }
}

fn get_next_no(state: &mut MailState, previous: u8) -> u8 {
    let random = state.rnd_f(10.0);
    let mut fvar = MAIL_RANDOM_BASE - f32::from(state.same_count) * MAIL_SAME_SPAN;
    let mut box_y = i32::from(previous) % MAIL_SPAN;
    let mut box_x = i32::from(previous) / MAIL_SPAN;

    if random < fvar {
        state.same_count = state.same_count.wrapping_add(1);
    } else {
        let new_threshold = fvar + MAIL_RANDOM_SPAN;
        if random < new_threshold {
            if state.rnd_f(1.0) > MAIL_RANDOM_HALF {
                if box_y == 0 || box_y == 2 {
                    box_y = 1;
                } else if state.rnd_f(1.0) > MAIL_RANDOM_HALF {
                    box_y = 0;
                } else {
                    box_y = 2;
                }
            } else {
                box_x = i32::from(box_x == 0);
            }
            state.same_count = 0;
        } else if random >= new_threshold {
            fvar = state.rnd_f(1.0);
            if fvar < MAIL_RANDOM_THIRD {
                if box_y == 0 || box_y == 2 {
                    box_y = 1;
                } else if state.rnd_f(1.0) > MAIL_RANDOM_HALF {
                    box_y = 0;
                } else {
                    box_y = 2;
                }
                box_x = i32::from(box_x == 0);
            } else if fvar < MAIL_RANDOM_TWO_THIRD {
                if box_y == 0 {
                    box_y = 2;
                } else if box_y == 2 {
                    box_y = 0;
                } else if state.rnd_f(1.0) > MAIL_RANDOM_HALF {
                    box_y = 0;
                } else {
                    box_y = 2;
                }
            } else if box_y == 0 {
                box_y = 2;
            } else if box_y == 2 {
                box_y = 0;
            } else if state.rnd_f(1.0) > MAIL_RANDOM_HALF {
                box_y = 0;
            } else {
                box_y = 2;
            }
            if fvar >= MAIL_RANDOM_TWO_THIRD {
                box_x = i32::from(box_x == 0);
            }
            state.same_count = 0;
        }
    }

    let res = (box_y + box_x * MAIL_SPAN) as u8;
    state.no_buff = res;
    res
}

pub struct MailCtx<'a> {
    pub p58: Vec3,
    pub p5c: Vec3,
    pub events: &'a mut SimEvents,
    pub mini_rupee: &'a mut i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mail {
    pub func: MailProc,
    pub field_24: Vec3,
    pub field_30: Vec3,
    pub field_3c: Vec3,
    pub field_48: [i16; 3],
    pub field_4e: [i16; 3],
    pub field_54: u8,
    pub field_55: u8,
    pub field_56: u8,
}

impl Mail {
    pub fn create() -> Self {
        Self {
            func: MailProc::Dummy,
            field_24: Vec3::ZERO,
            field_30: Vec3::ZERO,
            field_3c: Vec3::ZERO,
            field_48: [0; 3],
            field_4e: [0; 3],
            field_54: 0,
            field_55: 0,
            field_56: 0,
        }
    }

    pub fn check_proc(&self, func: MailProc) -> bool {
        self.func == func
    }

    pub fn set_proc(&mut self, func: MailProc) {
        self.func = func;
    }

    pub fn init(&mut self, p5c: Vec3) {
        self.field_54 = 0;
        self.field_56 = 0;
        self.field_30 = Vec3::ZERO;
        self.field_3c = Vec3::ZERO;
        self.field_55 = 0;
        self.field_24 = Vec3::new(p5c.x, p5c.y + MAIL_DROP_Y, p5c.z);
    }

    pub fn dummy_init(&mut self, ctx: &MailCtx<'_>, state: &mut MailState) {
        target_angle_y(ctx.p58, ctx.p5c);
        self.field_48 = [0, MAIL_YAW, 0];
        self.field_24 = Vec3::new(
            ctx.p5c.x + MAIL_SIDE_DROP_X,
            ctx.p5c.y + MAIL_DROP_Y,
            ctx.p5c.z,
        );
        self.field_54 = get_next_no(state, self.field_54);
        self.field_4e = [0, MAIL_YAW, MAIL_TILT];
        self.field_3c = Vec3::ZERO;
        self.set_proc(MailProc::Dummy);
    }

    pub fn appear_init(&mut self, ctx: &mut MailCtx<'_>, state: &mut MailState) {
        target_angle_y(ctx.p58, ctx.p5c);
        self.field_48 = [0, MAIL_YAW, 0];
        self.field_24 = Vec3::new(ctx.p5c.x, ctx.p5c.y + MAIL_DROP_Y, ctx.p5c.z);
        self.field_54 = get_next_no(state, self.field_54);
        self.field_4e = [0, MAIL_YAW, MAIL_TILT];
        self.field_3c = Vec3::ZERO;
        self.field_56 = 0;
        ctx.events.list.push(SimEvent::Se(SE_LETTER_GAME_NEW));
        self.set_proc(MailProc::Appear);
    }

    fn appear(&mut self, ctx: &MailCtx<'_>) {
        let new_y = target_angle_y(ctx.p58, ctx.p5c);
        let mut diff = ctx.p58 - ctx.p5c;
        if !normalize_rs(&mut diff) {
            diff = Vec3::new(0.0, 0.0, 1.0);
        }
        let mut vec = ctx.p5c;
        diff *= MAIL_REACH + REG10_F0;
        vec += diff;
        vec.y -= MAIL_REACH_Y + REG10_F1;
        self.field_48[0] = self.field_4e[0];
        self.field_48[1] = self.field_4e[1].wrapping_add(new_y).wrapping_add(REG10_S1);
        add_calc_angle_s2(
            &mut self.field_48[2],
            self.field_4e[2],
            MAIL_APPEAR_SPEED,
            MAIL_APPEAR_MAX,
        );
        if add_calc_pos(
            &mut self.field_24,
            vec,
            MAIL_APPEAR_SCALE,
            MAIL_APPEAR_STEP,
            MAIL_APPEAR_MIN,
        )
        .abs()
            < MAIL_APPEAR_MIN
        {
            self.wait_init();
        }
    }

    pub fn wait_init(&mut self) {
        self.set_proc(MailProc::Wait);
    }

    fn wait(&mut self, ctx: &MailCtx<'_>) {
        let new_y = target_angle_y(ctx.p58, ctx.p5c);
        let mut diff = ctx.p58 - ctx.p5c;
        if !normalize_rs(&mut diff) {
            diff = Vec3::new(0.0, 0.0, 1.0);
        }
        self.field_24 = ctx.p5c;
        diff *= MAIL_REACH + REG10_F0;
        self.field_24 += diff;
        self.field_24.y -= MAIL_REACH_Y + REG10_F1;
        self.field_48 = [
            self.field_4e[0],
            self.field_4e[1].wrapping_add(new_y).wrapping_add(REG10_S1),
            self.field_4e[2].wrapping_add(REG10_S2),
        ];
    }

    pub fn throw_init(&mut self, target: Vec3, want_no: u8) {
        self.field_30 = target;
        self.field_56 = 0;
        self.field_55 = want_no;
        self.set_proc(MailProc::Throw);
    }

    fn r#throw(&mut self, ctx: &mut MailCtx<'_>) {
        if self.field_56 == 0 {
            let x_angle = target_angle_x(self.field_30, ctx.p5c);
            let y_angle = target_angle_y(ctx.p58, ctx.p5c);
            let vec = Vec3::new(
                REG10_F2 + MAIL_THROW_VEC_X,
                REG10_F3 + MAIL_THROW_VEC_Y,
                MAIL_THROW_VEC_Z * REG10_F4,
            );
            let mult = rot_x(x_angle, rot_y(y_angle, vec)) + ctx.p5c;
            if add_calc_pos(
                &mut self.field_24,
                mult,
                MAIL_THROW_SCALE,
                MAIL_THROW_STEP,
                MAIL_THROW_MIN,
            ) < MAIL_THROW_DONE
            {
                self.field_48[1] = 0;
                self.field_4e[0] = 0;
                ctx.events.list.push(SimEvent::Se(SE_LK_SW_KAZEKIRI_S));
                self.field_56 = self.field_56.wrapping_add(1);
            }
        } else {
            let x_angle = target_angle_x(self.field_30, ctx.p5c);
            self.field_48[0] = x_angle.wrapping_neg();
            let spin = f32::from(self.field_54) * MAIL_SPIN_STEP + MAIL_SPIN_BASE;
            self.field_48[1] = self.field_48[1].wrapping_add(spin as i16);
            self.field_48[2] = 0;
            let pos_step = add_calc_pos(
                &mut self.field_24,
                self.field_30,
                MAIL_LAND_SCALE,
                HIO_MAIL_STEP,
                MAIL_LAND_MIN,
            );
            self.field_4e[0] = self.field_4e[0].wrapping_add(MAIL_SPIN_ADD as i16 + REG10_S0);
            self.field_3c.y = (1.0 - cos_s(self.field_4e[0])) * (MAIL_SPIN_AMP + REG10_F10);
            if pos_step < MAIL_LAND_MIN {
                ctx.events.list.push(SimEvent::Se(SE_LETTER_IN_BOX));
                if self.field_55 == self.field_54 {
                    *ctx.mini_rupee += 1;
                    ctx.events.list.push(SimEvent::Se(SE_MINIGAME_RIGHT));
                } else {
                    ctx.events.list.push(SimEvent::Se(SE_MINIGAME_WRONG));
                }
                self.end_init();
            }
        }
    }

    fn end_init(&mut self) {
        self.field_30 = Vec3::new(0.0, 0.0, MAIL_END_Z);
        self.set_proc(MailProc::End);
    }

    fn end(&mut self) {
        if self.field_55 == self.field_54 {
            add_calc_angle_s2(&mut self.field_48[1], 0, MAIL_APPEAR_SPEED, MAIL_APPEAR_MAX);
            add_calc_pos2(&mut self.field_3c, Vec3::ZERO, MAIL_END_SCALE, MAIL_END_STEP);
        } else {
            add_calc_angle_s2(
                &mut self.field_48[1],
                MAIL_END_YAW,
                MAIL_APPEAR_SPEED,
                MAIL_APPEAR_MAX,
            );
            add_calc_pos2(&mut self.field_3c, Vec3::ZERO, MAIL_END_SCALE, MAIL_END_STEP);
            if self.field_24.z < MAIL_FALL_GATE_Z {
                let target;
                if self.field_24.y > MAIL_TILT_PIVOT_Y {
                    target = MAIL_YAW;
                    self.field_30.y -= MAIL_FALL_STEP + REG10_F11;
                } else {
                    target = 0;
                    self.field_30.y *= MAIL_POP_Y;
                    self.field_30.x *= MAIL_POP_XZ;
                    self.field_30.z *= MAIL_POP_XZ;
                }
                if self.field_24.y <= MAIL_CLAMP_Y {
                    self.field_30.y = 0.0;
                    self.field_24.y = MAIL_CLAMP_Y;
                }
                add_calc_angle_s2(
                    &mut self.field_48[0],
                    target,
                    MAIL_END_TURN_SCALE,
                    MAIL_END_TURN_MAX.wrapping_add(REG10_S3),
                );
            }
            self.field_30 *= MAIL_FALL_RATE;
            self.field_24 += self.field_30;
        }
    }

    pub fn r#move(&mut self, ctx: &mut MailCtx<'_>) {
        match self.func {
            MailProc::Dummy => {}
            MailProc::Appear => self.appear(ctx),
            MailProc::Wait => self.wait(ctx),
            MailProc::Throw => self.r#throw(ctx),
            MailProc::End => self.end(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwCam {
    pub field_00: Vec3,
    pub field_0c: Vec3,
    pub field_18: f32,
    pub field_1c: i8,
    pub field_1d: i8,
    pub field_1e: u8,
}

impl SwCam {
    pub fn create() -> Self {
        Self {
            field_00: CAM_CENTRE[0][1],
            field_0c: CAM_EYE,
            field_18: CAM_FOV,
            field_1c: 0,
            field_1d: 0,
            field_1e: 0,
        }
    }

    pub fn active_off(&mut self) {
        self.field_1e = 0;
    }

    pub fn active_on(&mut self) {
        self.field_1e = 1;
        self.field_00 = CAM_CENTRE[0][1];
        self.field_0c = CAM_EYE;
        self.field_18 = CAM_FOV;
        self.set_aim_idx(0, 0);
    }

    pub fn set_aim_idx(&mut self, row: i8, col: i8) {
        self.field_1c = row;
        self.field_1d = col;
    }

    pub fn r#move(&mut self) {
        if self.field_1e != 0 {
            let centre = cam_centre(self.field_1d, self.field_1c);
            add_calc_pos(
                &mut self.field_00,
                centre,
                CAM_SCALE,
                CAM_STEP,
                CAM_MIN,
            );
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StControl {
    pub field_04: f32,
    pub field_08: f32,
    pub field_0c: u8,
    pub field_0d: u8,
    pub field_0e: i16,
    pub field_10: i16,
    pub field_12: i16,
    pub field_14: i16,
    pub field_16: i16,
    pub field_18: i16,
    pub field_1a: i16,
    pub field_1c: i16,
    pub field_1e: i16,
    pub field_20: i16,
    pub field_22: i16,
    pub field_24: i16,
    pub field_26: i16,
}

impl StControl {
    pub fn create() -> Self {
        let mut next = Self {
            field_04: 0.9,
            field_08: 0.5,
            field_0c: 0,
            field_0d: 0,
            field_0e: 0,
            field_10: 0,
            field_12: 0,
            field_14: 0,
            field_16: 0,
            field_18: 0,
            field_1a: 0,
            field_1c: 0,
            field_1e: 0,
            field_20: 0,
            field_22: 0,
            field_24: 0,
            field_26: 0,
        };
        next.set_wait_parm(0xF, 0xF, 0, 0, 0.9, 0.5, 0, 0x2000);
        next.init();
        next
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_wait_parm(
        &mut self,
        a: i16,
        b: i16,
        c: i16,
        d: i16,
        e: f32,
        f: f32,
        g: i16,
        h: i16,
    ) {
        self.field_12 = a;
        self.field_14 = b;
        self.field_16 = c;
        self.field_1c = d;
        self.field_04 = e;
        self.field_08 = f;
        self.field_24 = g;
        self.field_26 = h;
    }

    pub fn init(&mut self) {
        self.field_0e = 0;
        self.field_10 = 0;
        self.field_0d = 0;
        self.field_0c = 0;
        self.field_22 = 0;
        self.field_18 = self.field_12;
        self.field_1a = self.field_12;
        self.field_1e = self.field_1c;
        self.field_20 = self.field_1c;
    }

    fn xinit(&mut self) {
        self.field_0e = 0;
        self.field_0c &= 0xFC;
        self.field_18 = self.field_12;
        self.field_1e = self.field_1c;
    }

    fn yinit(&mut self) {
        self.field_10 = 0;
        self.field_0c &= !0x0C;
        self.field_1a = self.field_12;
        self.field_20 = self.field_1c;
    }

    pub fn check_trigger(&mut self, value: f32, angle: i16) -> u8 {
        self.field_0d = self.field_0c;
        let mut bits = 0_u8;
        let half = (0x2000 - self.field_26) >> 1;
        if !cm3d_is_zero(value) {
            if angle < self.field_22.wrapping_add(STICK_A).wrapping_add(half) {
                bits |= 4;
            } else if angle < self.field_22.wrapping_add(STICK_B).wrapping_sub(half) {
                bits |= 4 | 1;
            } else if angle < self.field_22.wrapping_add(STICK_C).wrapping_add(half) {
                bits |= 1;
            } else if angle < self.field_22.wrapping_add(STICK_D).wrapping_sub(half) {
                bits |= 1 | 8;
            } else if angle < self.field_22.wrapping_add(STICK_E).wrapping_add(half) {
                bits |= 8;
            } else if angle < self.field_22.wrapping_add(STICK_F).wrapping_sub(half) {
                bits |= 8 | 2;
            } else if angle < self.field_22.wrapping_add(STICK_G).wrapping_add(half) {
                bits |= 2;
            } else if angle < self.field_22.wrapping_add(STICK_H).wrapping_sub(half) {
                bits |= 2 | 4;
            } else {
                bits |= 4;
            }

            if value >= self.field_04 {
                self.field_0c = bits;
            } else if value < self.field_08 {
                self.field_0c = 0;
            } else {
                self.field_0c &= !bits;
            }

            if self.field_0c != self.field_0d {
                if self.field_0c == 0 {
                    self.field_22 = 0;
                } else if (angle & 0x1FFF) > 0x1000 {
                    self.field_22 = self.field_24;
                } else {
                    self.field_22 = self.field_24.wrapping_neg();
                }
            }

            if self.field_0c & STICK_X_MASK == 0 {
                self.xinit();
            }
            if self.field_0c & STICK_Y_MASK == 0 {
                self.yinit();
            }
        } else {
            self.field_0c = 0;
            self.xinit();
            self.yinit();
        }

        if self.field_0d & self.field_0c & STICK_X_MASK != 0 && self.field_0e > 0 {
            self.field_0e -= 1;
        }
        if self.field_0d & self.field_0c & STICK_Y_MASK != 0 && self.field_10 > 0 {
            self.field_10 -= 1;
        }
        self.field_0c
    }

    fn horizontal(&mut self, bit: u8) -> bool {
        if self.field_0e == 0 && self.field_0c & bit != 0 {
            self.field_0e = self.field_18;
            if self.field_1e == 0 {
                self.field_18 = self.field_18.wrapping_sub(self.field_16);
                if self.field_18 < self.field_14 {
                    self.field_18 = self.field_14;
                }
            } else {
                self.field_1e -= 1;
            }
            return true;
        }
        false
    }

    fn vertical(&mut self, bit: u8) -> bool {
        if self.field_10 == 0 && self.field_0c & bit != 0 {
            self.field_10 = self.field_1a;
            if self.field_20 == 0 {
                self.field_1a = self.field_1a.wrapping_sub(self.field_16);
                if self.field_1a < self.field_14 {
                    self.field_1a = self.field_14;
                }
            } else {
                self.field_20 -= 1;
            }
            return true;
        }
        false
    }

    pub fn check_left_trigger(&mut self) -> bool {
        self.horizontal(1)
    }

    pub fn check_right_trigger(&mut self) -> bool {
        self.horizontal(2)
    }

    pub fn check_up_trigger(&mut self) -> bool {
        self.vertical(4)
    }

    pub fn check_down_trigger(&mut self) -> bool {
        self.vertical(8)
    }
}

fn limitter(value: &mut i16, max_diff: i16, min_diff: i16) {
    if *value > max_diff {
        *value = max_diff;
    }
    if *value < min_diff {
        *value = min_diff;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JntCtrl {
    pub angles: [[i16; 2]; 2],
    pub min_angles: [[i16; 2]; 2],
    pub max_angles: [[i16; 2]; 2],
    pub max_turn_step: [[i16; 2]; 2],
    pub trn: bool,
    pub head_lock: bool,
    pub backbone_lock: bool,
}

impl JntCtrl {
    pub fn create() -> Self {
        Self {
            angles: [[0; 2]; 2],
            min_angles: [[0; 2]; 2],
            max_angles: [[0; 2]; 2],
            max_turn_step: [[0; 2]; 2],
            trn: false,
            head_lock: false,
            backbone_lock: false,
        }
    }

    pub fn head_y(&self) -> i16 {
        self.angles[0][1]
    }

    pub fn backbone_y(&self) -> i16 {
        self.angles[1][1]
    }

    pub fn trn_chk(&self) -> bool {
        self.trn
    }

    pub fn set_trn(&mut self) {
        self.trn = true;
    }

    pub fn on_head_lock(&mut self) {
        self.head_lock = true;
    }

    pub fn off_head_lock(&mut self) {
        self.head_lock = false;
    }

    pub fn on_backbone_lock(&mut self) {
        self.backbone_lock = true;
    }

    pub fn off_backbone_lock(&mut self) {
        self.backbone_lock = false;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_param(
        &mut self,
        max_backbone_x: i16,
        max_backbone_y: i16,
        min_backbone_x: i16,
        min_backbone_y: i16,
        max_head_x: i16,
        max_head_y: i16,
        min_head_x: i16,
        min_head_y: i16,
        max_turn_step: i16,
    ) {
        self.max_angles[1][0] = max_backbone_x;
        self.max_angles[1][1] = max_backbone_y;
        self.min_angles[1][0] = min_backbone_x;
        self.min_angles[1][1] = min_backbone_y;
        self.max_angles[0][0] = max_head_x;
        self.max_angles[0][1] = max_head_y;
        self.min_angles[0][0] = min_head_x;
        self.min_angles[0][1] = min_head_y;
        self.max_turn_step[1][0] = max_turn_step;
        self.max_turn_step[1][1] = max_turn_step;
        self.max_turn_step[0][0] = max_turn_step;
        self.max_turn_step[0][1] = max_turn_step;
    }

    fn ang_calc_s(out: &mut i16, target_y: i16, speed: i16, max_vel: i16) -> bool {
        let orig_diff = out.wrapping_sub(target_y);
        let diff = add_calc_angle_s(orig_diff, 0, speed, max_vel, JNT_ANG_MIN);
        if i32::from(diff).abs() > i32::from(speed) {
            *out = out.wrapping_add(diff.wrapping_sub(orig_diff));
        } else {
            *out = target_y;
        }
        target_y.wrapping_sub(*out) == 0
    }

    fn follow(&mut self, out_y: &mut i16, target_y: i16, max_vel: i16, axis: usize) -> bool {
        let orig_y = *out_y;
        Self::ang_calc_s(out_y, target_y, JNT_SCALE as i16, max_vel);
        let delta_y = out_y.wrapping_sub(orig_y);
        let mut acc = 0_i16;
        let mut prev = 0_i16;
        let rest = target_y.wrapping_sub(*out_y);
        for row in (0..2).rev() {
            acc = acc.wrapping_add(self.angles[row][axis]);
            if (delta_y > 0 && acc > rest) || (delta_y < 0 && acc < rest) {
                self.angles[row][axis] = rest.wrapping_sub(prev);
                limitter(
                    &mut self.angles[row][axis],
                    self.max_angles[row][axis],
                    self.min_angles[row][axis],
                );
            }
            prev = prev.wrapping_add(self.angles[row][axis]);
        }
        target_y != *out_y
    }

    fn r#move(&mut self, target: i16, axis: usize) -> bool {
        let mut rest = target;
        let mut want = [0_i16; 2];
        for row in 0..2 {
            let locked = (row == 0 && self.head_lock) || (row == 1 && self.backbone_lock);
            let mut angle = if locked { 0 } else { rest };
            limitter(
                &mut angle,
                self.max_angles[row][axis],
                self.min_angles[row][axis],
            );
            rest = rest.wrapping_sub(angle);
            want[row] = angle;
            let mut current = i32::from(self.angles[row][axis]);
            add_calc_angle_l(
                &mut current,
                i32::from(angle),
                JNT_SCALE,
                i32::from(self.max_turn_step[row][axis]),
                JNT_MIN_STEP,
            );
            self.angles[row][axis] = current as i16;
        }
        let mut done = 0;
        for row in 0..2 {
            if distance_angle_s(want[row], self.angles[row][axis])
                <= i32::from(self.max_turn_step[row][axis] / 2)
            {
                done += 1;
            }
        }
        done == 2
    }

    pub fn look_at_target(
        &mut self,
        out_y: &mut i16,
        dst: Option<Vec3>,
        src: Vec3,
        default_y: i16,
        max_vel: i16,
        lock: bool,
    ) {
        let (target_y, pitch) = match dst {
            Some(point) => {
                let delta = point - src;
                let dist_xz = (delta.x * delta.x + delta.z * delta.z).sqrt();
                (
                    to_s16(delta.x.atan2(delta.z)),
                    to_s16(delta.y.atan2(dist_xz)),
                )
            }
            None => (default_y, 0),
        };
        let delta_y = i32::from(target_y) - i32::from(*out_y);
        let max_y = i32::from(self.max_angles[1][1]) + i32::from(self.max_angles[0][1]);
        let min_y = i32::from(self.min_angles[1][1]) + i32::from(self.min_angles[0][1]);
        let settled = self.r#move(delta_y as i16, 1);
        if (self.trn || (delta_y >= max_y || delta_y <= min_y) && settled) && !lock {
            self.trn = self.follow(out_y, target_y, max_vel, 1);
        }
        self.r#move(pitch, 0);
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct NpcBtsw {
    pub action: Option<Action>,
    pub action_status: i8,
    pub field_9c4: i8,
    pub field_9c5: i8,
    pub field_9c6: i8,
    pub order: i8,
    pub field_9c8: i16,
    pub mode: i8,
    pub aim_x: i8,
    pub aim_y: i8,
    pub field_9a4: i32,
    pub has_attention: u8,
    pub field_9a9: u8,
    pub field_9b0: u32,
    pub field_92c: u8,
    pub field_952: u8,
    pub field_953: u8,
    pub field_7fc: u8,
    pub field_954: i16,
    pub field_96a: i16,
    pub field_96e: [i16; 3],
    pub curr_msg_no: u32,
    pub curr_msg_id: u32,
    pub att_pos: Vec3,
    pub attention: Vec3,
    pub eye_pos: Vec3,
    pub gravity: f32,
    pub collision_radius: f32,
    pub collision_height: f32,
    pub timer_rest: i32,
    pub timer_live: bool,
    pub timer_delete: i32,
    pub mini_game_type: u8,
    pub prev_do: bool,
    pub entered: bool,
    pub morf: Frame,
    pub jnt: JntCtrl,
    pub st: StControl,
    pub swcam: SwCam,
    pub mail_state: MailState,
    pub mails: [Mail; MAIL_COUNT],
}

impl NpcBtsw {
    pub fn spawn(parameters: u32) -> Self {
        let _ = parameters;
        Self {
            action: None,
            action_status: ACTION_STARTING,
            field_9c4: 0,
            field_9c5: ANM_WAIT01,
            field_9c6: 0,
            order: ORDER_NONE,
            field_9c8: NO_EVENT,
            mode: 0,
            aim_x: 0,
            aim_y: 0,
            field_9a4: 0,
            has_attention: 0,
            field_9a9: 0,
            field_9b0: FPC_M_ERROR_PROCESS_ID,
            field_92c: 0,
            field_952: 0,
            field_953: 0,
            field_7fc: 0,
            field_954: 0,
            field_96a: 0,
            field_96e: [0; 3],
            curr_msg_no: 0,
            curr_msg_id: FPC_M_ERROR_PROCESS_ID,
            att_pos: Vec3::ZERO,
            attention: Vec3::ZERO,
            eye_pos: Vec3::ZERO,
            gravity: GRAVITY,
            collision_radius: 60.0,
            collision_height: 150.0,
            timer_rest: 0,
            timer_live: false,
            timer_delete: 0,
            mini_game_type: 0,
            prev_do: false,
            entered: false,
            morf: Frame::create(ANM_FRAME_MAX[0]),
            jnt: JntCtrl::create(),
            st: StControl::create(),
            swcam: SwCam::create(),
            mail_state: MailState::create(),
            mails: [Mail::create(); MAIL_COUNT],
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    fn check_frame(&self) -> bool {
        self.morf.check_pass(self.morf.end_frame() - 1.0)
    }

    fn mail_at(&self, index: u8) -> Mail {
        self.mails
            .get(usize::from(index))
            .copied()
            .unwrap_or_else(Mail::create)
    }

    fn set_mail(&mut self, index: u8, mail: Mail) {
        if let Some(slot) = self.mails.get_mut(usize::from(index)) {
            *slot = mail;
        }
    }

    fn swcam_00(&self) -> Vec3 {
        self.swcam.field_00
    }

    fn swcam_0c(&self) -> Vec3 {
        self.swcam.field_0c
    }

    pub fn set_anm(&mut self, idx: i8) {
        let mut morf = ANM_MORF_FRAME;
        if self.field_9c5 == ANM_SUGOI_ALT || self.field_9c5 == ANM_SUGOI {
            morf = HIO_SUGOI_MORF;
        }
        if idx != self.field_9c5 {
            self.field_9c5 = idx;
            let at = usize::try_from(idx).unwrap_or(ANM_COUNT - 1);
            let mode = ANM_PLAY_MODE.get(at).copied().unwrap_or(EMODE_NONE);
            let speed = ANM_PLAY_SPEED.get(at).copied().unwrap_or(1.0);
            let end = ANM_FRAME_MAX.get(at).copied().unwrap_or(0.0);
            self.morf.set_anm(end, mode, speed);
            self.morf.morf_step = morf;
            if self.field_9c5 == ANM_SIWAKE01
                || self.field_9c5 == ANM_SIWAKE02
                || self.field_9c5 == ANM_SUGOI_ALT
                || self.field_9c5 == ANM_SUGOI
            {
                self.jnt.on_head_lock();
                self.jnt.on_backbone_lock();
            } else {
                self.jnt.off_head_lock();
                self.jnt.off_backbone_lock();
            }
        }
    }

    pub fn play_tex_pattern_anm(&mut self) {
        if calc_timer(&mut self.field_954) == 0 {
            if i32::from(self.field_7fc) >= BTP_FRAME_MAX {
                self.field_7fc = self.field_7fc.wrapping_sub(BTP_FRAME_MAX as u8);
                let wait = self.mail_state.rnd_f(BTP_TIMER_RANGE) + BTP_TIMER_MIN;
                self.field_954 = wait as i16;
            } else {
                self.field_7fc += 1;
            }
        }
    }

    pub fn chk_attention(&self, pos: Vec3, angle: i16, player: Option<Vec3>) -> bool {
        let mut max_dist = HIO_MAX_ATTN_DIST_XZ;
        let mut max_angle = HIO_MAX_ATTN_ANGLE_Y;
        let Some(player) = player else {
            return false;
        };
        let dx = player.x - pos.x;
        let dz = player.z - pos.z;
        let dist_xz = (dx * dx + dz * dz).sqrt();
        let mut target = to_s16(dx.atan2(dz));
        if self.has_attention != 0 {
            max_dist += ATTN_EXTRA_DIST;
            max_angle += (ATTN_EXTRA_DEG * ATTN_DEG2S) as i32;
        }
        target = target.wrapping_sub(angle);
        max_angle > i32::from(target).abs() && max_dist > dist_xz
    }

    pub fn get_msg(&mut self, stories: &mut StoryBits, clear_count: u8, talk_xy: bool) -> u32 {
        if self.field_9b0 != FPC_M_ERROR_PROCESS_ID {
            let msg = self.field_9b0;
            self.field_9b0 = FPC_M_ERROR_PROCESS_ID;
            return msg;
        }
        if talk_xy {
            return MSG_1AA3;
        }
        let msg = if !stories.test(FLAG_TALKED) {
            stories.set(FLAG_TALKED);
            if clear_count == 0 { MSG_1A91 } else { MSG_1AB5 }
        } else if !stories.test(FLAG_TRIED_LETTER) {
            MSG_1AA1
        } else if stories.reg(REG_TRIES) < 3 {
            MSG_1AA5
        } else {
            MSG_1AA4
        };
        stories.set_reg(REG_TRIES, 0);
        msg
    }

    pub fn next_msg_status(
        &self,
        curr: &mut u32,
        select: u8,
        stories: &mut StoryBits,
        collect: &Collect,
    ) -> u8 {
        let mut ret = STATUS_MSG_CONTINUES;
        match *curr {
            0x1A91 | 0x1A92 | 0x1A94 | 0x1A95 | 0x1A96 | 0x1A98 | 0x1A99 | 0x1A9A | 0x1AA7
            | 0x1AAD => *curr += 1,
            0x1AB5 => *curr = MSG_1A93,
            MSG_1A93 => {
                if select == 0 {
                    stories.set(FLAG_TRIED_LETTER);
                    *curr = MSG_1A94;
                } else {
                    *curr = MSG_1AA0;
                }
            }
            MSG_1AA1 => {
                if select == 0 {
                    stories.set(FLAG_TRIED_LETTER);
                    *curr = MSG_1A94;
                } else {
                    *curr = MSG_1AA2;
                }
            }
            MSG_1AA8 => {
                if select == 0 {
                    *curr = MSG_1AA9;
                } else {
                    *curr = MSG_1AAA;
                }
            }
            0x1AA4 | 0x1AA5 | 0x1AA6 => {
                if !stories.test(FLAG_GOT_LETTER) {
                    *curr = MSG_1AA7;
                } else if stories.reg(LETTER_BAITOS_MOM) == LETTER_NOSEND {
                    *curr = MSG_1AAB;
                } else if !collect.is_symbol(SYMBOL_DIN) {
                    *curr = MSG_1AB4;
                } else if stories.reg(LETTER_BAITOS_MOM) != LETTER_READ {
                    *curr = MSG_1AAD;
                } else if !stories.test(FLAG_SENT_LETTER) {
                    *curr = MSG_1AAC;
                    stories.set(FLAG_SENT_LETTER);
                } else {
                    *curr = MSG_1AB3;
                }
            }
            MSG_1AA9 => *curr = MSG_1A97,
            _ => ret = STATUS_MSG_ENDS,
        }
        ret
    }

    pub fn anm_atr(&mut self, attr: u8) {
        match attr {
            0 => self.set_anm(ANM_WAIT01),
            1 => self.set_anm(ANM_WAIT02),
            2 => self.set_anm(ANM_TALK01),
            3 => self.set_anm(ANM_TALK02),
            4 => self.set_anm(ANM_ONEGAI),
            5 => self.set_anm(ANM_SUGOI),
            6 => self.set_anm(ANM_SUGOI_ALT),
            _ => {}
        }
        if self.check_frame() && (self.field_9c5 == ANM_ONEGAI || self.field_9c5 == ANM_SUGOI_ALT)
        {
            self.set_anm(ANM_WAIT02);
        }
    }

    pub fn set_attention(&mut self) {
        self.attention = Vec3::new(
            self.att_pos.x,
            self.att_pos.y + HIO_ATTN_Y_OFFSET,
            self.att_pos.z,
        );
    }

    pub fn look_back(&mut self, actor: &mut Actor, ctx: &Ctx<'_>) {
        let desired = actor.angle[1];
        let mut dst = None;
        let mut src = Vec3::ZERO;
        if self.mode == MODE_WAIT || self.mode == MODE_TALK {
            if self.mode == MODE_TALK {
                self.jnt.set_trn();
            }
            if self.has_attention != 0 {
                dst = ctx.player_eye;
                src = Vec3::new(actor.pos.x, self.eye_pos.y, actor.pos.z);
            }
        }
        if self.jnt.trn_chk() {
            add_calc_angle_s2(&mut self.field_96a, HIO_MAX_HEAD_TURN_VEL, 4, 0x800);
        } else {
            self.field_96a = 0;
        }
        let turn = self.field_96a;
        self.jnt
            .look_at_target(&mut actor.angle[1], dst, src, desired, turn, true);
    }

    pub fn wait01(&mut self) {
        if self.field_9a9 != 0 {
            self.mode = MODE_TALK;
            return;
        }
        if self.check_frame() {
            self.field_9c6 -= 1;
            if self.field_9c6 <= 0 {
                if self.field_9c5 == ANM_SIWAKE01 {
                    self.field_9c6 = 1;
                    self.set_anm(ANM_SIWAKE02);
                } else {
                    let count = self.mail_state.rnd_f(4.0) + 2.0;
                    self.field_9c6 = count as i8;
                    self.set_anm(ANM_SIWAKE01);
                }
            }
        }
        if self.has_attention != 0 {
            self.order = ORDER_ATTENTION;
        }
    }

    fn talk(&mut self, ctx: &mut Ctx<'_>) -> u8 {
        let mut status = NO_STATUS;
        if self.curr_msg_id == FPC_M_ERROR_PROCESS_ID {
            self.curr_msg_no = self.get_msg(ctx.stories, ctx.text.clear_count, ctx.talk_xy);
            self.curr_msg_id = ctx.text.message_set_actor(self.curr_msg_no, self.eye_pos);
        } else if ctx.text.msg_live(self.curr_msg_id) {
            status = ctx.text.status().unwrap_or(NO_STATUS);
            if status == STATUS_MSG_DISPLAYED {
                let select = ctx.select;
                let mut curr = self.curr_msg_no;
                let next = self.next_msg_status(&mut curr, select, ctx.stories, ctx.collect);
                self.curr_msg_no = curr;
                ctx.text.set_status(next);
                if next == STATUS_MSG_CONTINUES {
                    let msg = self.curr_msg_no;
                    ctx.text.message_set(msg);
                }
            } else if status == STATUS_BOX_CLOSED {
                ctx.text.set_status(STATUS_MSG_DESTROYED);
                self.curr_msg_id = FPC_M_ERROR_PROCESS_ID;
            }
            self.anm_atr(ctx.text.mesg_anime());
            ctx.text.set_mesg_anime(0xFF);
        }
        status
    }

    fn talk01(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) -> u8 {
        if self.curr_msg_id == FPC_M_ERROR_PROCESS_ID && ctx.talk_xy && !ctx.present_end {
            return NO_STATUS;
        }
        let talk_res = self.talk(ctx);
        if ctx.mesg_send_button == 1 && (self.curr_msg_no == MSG_1A96 || self.curr_msg_no == MSG_1AA9)
        {
            self.swcam.active_on();
        }
        if talk_res == STATUS_BOX_CLOSED {
            self.mode = MODE_WAIT;
            ctx.event_reset(self.field_9c8);
            self.set_anm(ANM_WAIT02);
            self.field_9c6 = 1;
            self.field_9a9 = 0;
            if self.curr_msg_no == MSG_1A97 {
                self.set_action(Action::ShiwakeGame, actor, ctx);
            } else if self.curr_msg_no == MSG_1A9B {
                self.order = ORDER_GETMOTHER;
            }
        }
        talk_res
    }

    fn run_action(&mut self, which: Action, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        match which {
            Action::Wait => self.wait_action(actor, ctx),
            Action::DummyEvent => self.dummy_event_action(actor, ctx),
            Action::ShiwakeGame => self.shiwake_game_action(actor, ctx),
            Action::GetDemo => self.getdemo_action(actor, ctx),
        }
    }

    pub fn set_action(&mut self, action: Action, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        if self.action != Some(action) {
            if let Some(previous) = self.action {
                self.action_status = ACTION_ENDING;
                self.run_action(previous, actor, ctx);
            }
            self.action = Some(action);
            self.action_status = ACTION_STARTING;
            self.run_action(action, actor, ctx);
        }
    }

    fn wait_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        if self.action_status == ACTION_STARTING {
            self.mode = MODE_WAIT;
            self.field_9c6 = 1;
            self.set_anm(ANM_WAIT02);
            self.action_status += 1;
        } else if self.action_status != ACTION_ENDING {
            let angle = actor.angle[1]
                .wrapping_add(self.jnt.head_y())
                .wrapping_add(self.jnt.backbone_y());
            self.has_attention = u8::from(self.chk_attention(actor.pos, angle, ctx.player_pos));
            self.order = ORDER_NONE;
            if self.mode == MODE_WAIT {
                self.wait01();
            } else if self.mode == MODE_TALK {
                self.talk01(actor, ctx);
            }
            self.look_back(actor, ctx);
            self.set_attention();
        }
    }

    fn dummy_event_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        if self.action_status == ACTION_STARTING {
            self.mode = MODE_TALK;
            self.action_status += 1;
        } else if self.action_status != ACTION_ENDING {
            if !ctx.cut_proc {
                ctx.end_staff_cut(STAFF_BTSW);
            }
            let angle = actor.angle[1]
                .wrapping_add(self.jnt.head_y())
                .wrapping_add(self.jnt.backbone_y());
            self.has_attention = u8::from(self.chk_attention(actor.pos, angle, ctx.player_pos));
            if ctx.event_ended(self.field_9c8) {
                self.field_9c8 = NO_EVENT;
                self.order = ORDER_GETMOTHER;
                ctx.event_reset(self.field_9c8);
                self.set_action(Action::Wait, actor, ctx);
            }
            self.look_back(actor, ctx);
            self.set_attention();
        }
    }

    fn check_next_mail_throw_ok(&self) -> bool {
        let index = if self.field_92c < MAIL_LAST {
            self.field_92c + 1
        } else {
            0
        };
        let mail = self.mail_at(index);
        mail.check_proc(MailProc::Dummy) || mail.check_proc(MailProc::End)
    }

    fn timer_count_down(&mut self, events: &mut SimEvents) {
        if self.field_9a4 > self.timer_rest {
            if self.field_9a4 <= TIMER_GATE {
                events.list.push(SimEvent::Se(SE_MINIGAME_TIMER_30));
                self.field_9a4 -= TIMER_BEAT;
            } else {
                events.list.push(SimEvent::Se(SE_MINIGAME_TIMER_10));
                self.field_9a4 -= TIMER_BEAT_BIG;
            }
        }
    }

    fn mail_start(&mut self, ctx: &mut Ctx<'_>) {
        let p58 = self.swcam_00();
        let p5c = self.swcam_0c();
        let mut state = self.mail_state;
        {
            let mut mail_ctx = ctx.mail_ctx(p58, p5c);
            for index in 0..MAIL_COUNT {
                self.mails[index].init(p5c);
            }
            self.mails[0].appear_init(&mut mail_ctx, &mut state);
            self.mails[1].dummy_init(&mail_ctx, &mut state);
            self.mails[2].dummy_init(&mail_ctx, &mut state);
        }
        self.mail_state = state;
    }

    fn appear_mail(&mut self, index: u8, ctx: &mut Ctx<'_>) {
        let p58 = self.swcam_00();
        let p5c = self.swcam_0c();
        let mut state = self.mail_state;
        {
            let mut mail_ctx = ctx.mail_ctx(p58, p5c);
            if let Some(mail) = self.mails.get_mut(usize::from(index)) {
                mail.appear_init(&mut mail_ctx, &mut state);
            }
        }
        self.mail_state = state;
    }

    fn mail_move(&mut self, ctx: &mut Ctx<'_>) {
        let p58 = self.swcam_00();
        let p5c = self.swcam_0c();
        let mut mail_ctx = ctx.mail_ctx(p58, p5c);
        for index in 0..MAIL_COUNT {
            self.mails[index].r#move(&mut mail_ctx);
        }
    }

    fn shiwake_game_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        if self.action_status == ACTION_STARTING {
            self.mode = MODE_SHIWAKE;
            self.aim_x = 0;
            self.aim_y = 0;
            if !ctx.demo_accept {
                ctx.order_event_name(EVENT_SHIWAKEGAME);
                return;
            }
            if self.field_952 == 0 {
                self.field_952 = 1;
                ctx.mini_rupee = 0;
                ctx.mini_game_type = MINI_GAME_LETTER;
                self.timer_rest = HIO_TIMER_START * 1000;
                self.timer_live = true;
                self.timer_delete = 0;
            }
            if !self.timer_live {
                return;
            }
            self.field_953 = 1;
            self.mail_start(ctx);
            self.field_92c = 0;
            self.action_status += 1;
            ctx.cursor_pos = cursor_pos(self.aim_y, self.aim_x);
            ctx.cursor_shown = true;
            self.field_9a4 = SHIWAKE_REST_START;
        } else if self.action_status == ACTION_ENDING {
            self.field_952 = 0;
            self.field_953 = 0;
        } else if self.timer_rest > 0 {
            self.timer_rest = (self.timer_rest - MS_PER_FRAME).max(0);
            self.timer_count_down(ctx.events);
            self.st.check_trigger(ctx.stick_value, ctx.stick_angle);
            if self.aim_x > -1 && self.st.check_left_trigger() {
                ctx.events.list.push(SimEvent::Se(SE_LETTER_GAME_CURSOR));
                self.aim_x -= 1;
            } else if self.aim_x < 1 && self.st.check_right_trigger() {
                ctx.events.list.push(SimEvent::Se(SE_LETTER_GAME_CURSOR));
                self.aim_x += 1;
            }
            if self.aim_y < 1 && self.st.check_up_trigger() {
                ctx.events.list.push(SimEvent::Se(SE_LETTER_GAME_CURSOR));
                self.aim_y += 1;
            } else if self.aim_y > 0 && self.st.check_down_trigger() {
                ctx.events.list.push(SimEvent::Se(SE_LETTER_GAME_CURSOR));
                self.aim_y -= 1;
            }
            let ready = self.mail_at(self.field_92c).check_proc(MailProc::Wait);
            if ready && ctx.timer_status != 1 && ctx.trig_a && self.check_next_mail_throw_ok() {
                let row = i32::from(self.aim_x) + 1;
                let col = i32::from(self.aim_y);
                let want = (row + (1 - col) * MAIL_SPAN) as u8;
                let target = aim_pos(self.aim_y, row);
                let mut mail = self.mail_at(self.field_92c);
                mail.throw_init(target, want);
                self.set_mail(self.field_92c, mail);
                self.field_92c = if self.field_92c < MAIL_LAST {
                    self.field_92c + 1
                } else {
                    0
                };
                ctx.events.list.push(SimEvent::Se(SE_LETTER_GAME_OK));
                self.appear_mail(self.field_92c, ctx);
            }
            self.swcam.set_aim_idx(self.aim_x, self.aim_y);
            ctx.cursor_pos = cursor_pos(self.aim_y, self.aim_x);
            ctx.cursor_scale = [
                CURSOR_SCALE_X,
                CURSOR_SCALE_Y,
                CURSOR_SCALE_Z,
                CURSOR_SCALE_W,
                CURSOR_SCALE_H,
            ];
        } else {
            if self.action_status == ACTION_ONGOING {
                ctx.events.list.push(SimEvent::Se(SE_LETTER_GAME_TIMER_0));
                ctx.timer_end = HIO_TIMER_END as i16;
                ctx.cursor_hidden = true;
                self.action_status += 1;
            } else if self.action_status == ACTION_UNK_2 {
                self.timer_delete += 1;
                if self.timer_delete >= TIMER_DELETE_FRAMES {
                    self.timer_live = false;
                    self.action_status += 1;
                }
            } else if !self.timer_live {
                ctx.end_staff_cut(STAFF_BTSW);
                ctx.event_reset(self.field_9c8);
                let rupees = ctx.mini_rupee as i16;
                self.field_9b0 = get_game_end_msg(rupees);
                if rupees < MINI_RUPEE_LIMIT {
                    self.order = ORDER_SPEAK;
                    self.set_action(Action::Wait, actor, ctx);
                    self.field_9c8 = NO_EVENT;
                } else {
                    self.order = ORDER_EVENT;
                    self.field_9c8 = event_index(ctx.list, EVENT_BTSW_TALK_SUGOI);
                    self.set_action(Action::DummyEvent, actor, ctx);
                }
                self.swcam.active_off();
                if ctx.mini_game_type == MINI_GAME_LETTER {
                    ctx.end_mini_game = true;
                }
            }
            self.mail_move(ctx);
        }
    }

    fn getdemo_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        if self.action_status == ACTION_STARTING {
            self.action_status += 1;
        } else if self.action_status != ACTION_ENDING {
            let index = event_index(ctx.list, EVENT_GETMOTHERLETTER);
            if ctx.event_ended(index) {
                ctx.event_reset(index);
                self.order = ORDER_SPEAK;
                self.field_9b0 = MSG_1A9C;
                ctx.stories.set(FLAG_GOT_LETTER);
                ctx.stories.set_reg(REG_8AFF, REG_GOT_LETTER);
                self.set_action(Action::Wait, actor, ctx);
            }
        }
    }

    fn check_order(
        &mut self,
        demo_accept: bool,
        talk_command: bool,
        actor: &mut Actor,
        ctx: &mut Ctx<'_>,
    ) {
        if demo_accept {
            if self.order == ORDER_GETMOTHER {
                self.order = ORDER_NONE;
                self.set_action(Action::GetDemo, actor, ctx);
            } else if self.order == ORDER_EVENT {
                self.order = ORDER_NONE;
                self.set_action(Action::DummyEvent, actor, ctx);
            }
            return;
        }
        if !talk_command {
            return;
        }
        if self.order != ORDER_SPEAK && self.order != ORDER_ATTENTION {
            return;
        }
        self.order = ORDER_NONE;
        self.field_9a9 = 1;
    }

    fn event_order(&mut self, ctx: &mut Ctx<'_>) {
        if self.order == ORDER_GETMOTHER {
            ctx.order_event_name(EVENT_GETMOTHERLETTER);
        } else if self.order == ORDER_EVENT {
            ctx.order_event(self.field_9c8);
        }
    }

    pub fn create_init(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.field_96e = actor.angle;
        self.attention = Vec3::ZERO;
        self.gravity = GRAVITY;
        self.set_action(Action::Wait, actor, ctx);
        self.att_pos = actor.pos;
        self.collision_radius = 60.0;
        self.collision_height = 150.0;
        self.field_9c8 = NO_EVENT;
        self.field_952 = 0;
        self.field_953 = 0;
        self.field_9b0 = FPC_M_ERROR_PROCESS_ID;
        self.field_9c4 = 0;
        self.field_7fc = 0;
        self.field_954 = 0;
        self.field_92c = 0;
        self.curr_msg_no = 0;
        self.curr_msg_id = FPC_M_ERROR_PROCESS_ID;
        self.swcam = SwCam::create();
        self.mail_state = MailState::create();
        let p58 = self.swcam_00();
        let p5c = self.swcam_0c();
        let mut state = self.mail_state;
        {
            let mail_ctx = ctx.mail_ctx(p58, p5c);
            for index in 0..MAIL_COUNT {
                self.mails[index] = Mail::create();
                self.mails[index].init(p5c);
                self.mails[index].dummy_init(&mail_ctx, &mut state);
            }
        }
        self.mail_state = state;
        self.st.set_wait_parm(5, 2, 3, 2, 1.0, 0.9, 0, 0x2000);
    }

    pub fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.jnt.set_param(
            HIO_MAX_BACKBONE_X,
            HIO_MAX_BACKBONE_Y,
            HIO_MIN_BACKBONE_X,
            HIO_MIN_BACKBONE_Y,
            HIO_MAX_HEAD_X,
            HIO_MAX_HEAD_Y,
            HIO_MIN_HEAD_X,
            HIO_MIN_HEAD_Y,
            HIO_MAX_TURN_STEP,
        );
        self.st
            .set_wait_parm(5, 2, 3, 2, HIO_STICK_HI, HIO_STICK_LO, 0, 0x2000);
        self.play_tex_pattern_anm();
        self.morf.update();
        let demo_accept = ctx.demo_accept;
        let talk_command = ctx.talk_command;
        self.check_order(demo_accept, talk_command, actor, ctx);
        if let Some(which) = self.action {
            self.run_action(which, actor, ctx);
        }
        self.swcam.r#move();
        self.event_order(ctx);
    }
}

pub struct Text<'a> {
    pub procs: &'a mut MsgProcs,
    pub info: &'a mut MsgGameInfo,
    pub clear_count: u8,
    pub one: &'a mut Msg1Box,
    pub two: &'a mut Msg2Box,
    pub archive: Option<&'a dyn MsgArchive>,
}

impl Text<'_> {
    fn proc_name(&self) -> Option<u16> {
        if self.procs.search_by_name(FPC_NM_MSG).is_some() {
            Some(FPC_NM_MSG)
        } else if self.procs.search_by_name(FPC_NM_MSG2).is_some() {
            Some(FPC_NM_MSG2)
        } else {
            None
        }
    }

    pub fn msg_live(&self, id: u32) -> bool {
        id != FPC_M_ERROR_PROCESS_ID && self.procs.search_by_id(id).is_some()
    }

    pub fn status(&self) -> Option<u8> {
        match self.proc_name() {
            Some(FPC_NM_MSG) => Some(self.one.status),
            Some(FPC_NM_MSG2) => Some(self.two.status),
            _ => None,
        }
    }

    pub fn set_status(&mut self, status: u8) {
        match self.proc_name() {
            Some(FPC_NM_MSG) => self.one.status = status,
            Some(FPC_NM_MSG2) => self.two.status = status,
            _ => {}
        }
    }

    pub fn mesg_anime(&self) -> u8 {
        self.info.mesg_anime
    }

    pub fn set_mesg_anime(&mut self, id: u8) {
        self.info.set_mesg_anime_attr_info(id);
    }

    pub fn message_set(&mut self, msg_no: u32) -> u32 {
        let Some(archive) = self.archive else {
            return FPC_M_ERROR_PROCESS_ID;
        };
        fop_msgm_message_set(
            self.procs,
            self.info,
            archive,
            self.clear_count,
            msg_no,
            None,
            Vec3::ZERO,
        )
    }

    pub fn message_set_actor(&mut self, msg_no: u32, eye: Vec3) -> u32 {
        let Some(archive) = self.archive else {
            return FPC_M_ERROR_PROCESS_ID;
        };
        fop_msgm_message_set_actor(
            self.procs,
            self.info,
            archive,
            self.clear_count,
            msg_no,
            None,
            eye,
        )
    }
}

pub struct Ctx<'a> {
    pub entity: Entity,
    pub player_pos: Option<Vec3>,
    pub player_eye: Option<Vec3>,
    pub select: u8,
    pub talk_xy: bool,
    pub present_end: bool,
    pub talk_command: bool,
    pub demo_accept: bool,
    pub cut_proc: bool,
    pub trig_a: bool,
    pub stick_value: f32,
    pub stick_angle: i16,
    pub mesg_send_button: u8,
    pub timer_status: i32,
    pub timer_end: i16,
    pub cursor_hidden: bool,
    pub cursor_shown: bool,
    pub cursor_pos: Vec3,
    pub cursor_scale: [f32; 5],
    pub mini_rupee: i32,
    pub mini_game_type: u8,
    pub end_mini_game: bool,
    pub stories: &'a mut StoryBits,
    pub collect: &'a Collect,
    pub ev_flags: &'a mut EventFlags,
    pub list: &'a EventList,
    pub active: &'a mut ActiveEvent,
    pub events: &'a mut SimEvents,
    pub text: &'a mut Text<'a>,
}

impl Ctx<'_> {
    pub fn mail_ctx<'b>(&'b mut self, p58: Vec3, p5c: Vec3) -> MailCtx<'b> {
        MailCtx {
            p58,
            p5c,
            events: &mut *self.events,
            mini_rupee: &mut self.mini_rupee,
        }
    }

    pub fn event_reset(&mut self, index: i16) {
        if let Ok(slot) = usize::try_from(index) {
            let _ = self.active.end(slot);
        }
    }

    pub fn event_ended(&self, index: i16) -> bool {
        usize::try_from(index).is_ok_and(|slot| self.active.end_check(slot))
    }

    pub fn order_event(&mut self, index: i16) {
        if let Ok(slot) = usize::try_from(index) {
            let entity = self.entity;
            let _ = self.active.order(self.list, slot, entity);
        }
    }

    pub fn order_event_name(&mut self, name: &str) {
        if let Some(slot) = self.list.event_index(name) {
            let entity = self.entity;
            let _ = self.active.order(self.list, slot, entity);
        }
    }

    pub fn end_staff_cut(&mut self, name: &str) {
        let flags = &mut *self.ev_flags;
        self.active.end_staff_cut(self.list, flags, name);
    }
}

#[derive(SystemParam)]
pub struct World<'w> {
    pub text_flags: Res<'w, TextFlags>,
    pub intent: Res<'w, MoveIntent>,
    pub do_intent: Res<'w, DoIntent>,
    pub list: Res<'w, EventList>,
    pub stories: ResMut<'w, StoryBits>,
    pub collect: Res<'w, Collect>,
    pub ev_flags: ResMut<'w, EventFlags>,
    pub active: ResMut<'w, ActiveEvent>,
    pub events: ResMut<'w, SimEvents>,
    pub counters: ResMut<'w, Counters>,
    pub archive: Option<Res<'w, MsgArchives>>,
}

#[derive(SystemParam)]
pub struct Msg<'w, 's> {
    pub procs: ResMut<'w, MsgProcs>,
    pub info: ResMut<'w, MsgGameInfo>,
    pub one: ResMut<'w, Msg1Box>,
    pub two: ResMut<'w, Msg2Box>,
    pub players: Query<'w, 's, &'static Player>,
}

pub fn control(
    mut world: World,
    mut msg: Msg,
    mut actors: Query<(Entity, &mut Actor, &mut NpcBtsw)>,
) {
    let player = msg.players.iter().next().copied();
    let clear_count = world.text_flags.clear_count;
    let select = msg.one.select_num;
    let send_button = msg.one.m116a;
    let demo_accept = msg.info.event_run_check();
    let axis = world.intent.axis;
    let stick_value = axis.length().min(1.0);
    let stick_angle = if cm3d_is_zero(axis.x) && cm3d_is_zero(axis.y) {
        0
    } else {
        to_s16(axis.x.atan2(axis.y))
    };
    let trig_a = world.do_intent.0;
    let archive = world
        .archive
        .as_deref()
        .map(|held| held as &dyn MsgArchive);
    for (entity, mut actor, mut btsw) in &mut actors {
        let actor = &mut *actor;
        let btsw = &mut *btsw;
        let do_now = trig_a;
        let tap = do_now && !btsw.prev_do;
        btsw.prev_do = do_now;
        let talk_command = tap
            && player.is_some_and(|link| {
                let gap = link.pos - btsw.att_pos;
                gap.x * gap.x + gap.z * gap.z < TALK_RANGE * TALK_RANGE
            });
        let cut_proc = usize::try_from(btsw.field_9c8)
            .ok()
            .is_some_and(|slot| world.active.stage_of(slot).is_some());
        let mut text = Text {
            procs: &mut msg.procs,
            info: &mut msg.info,
            clear_count,
            one: &mut msg.one,
            two: &mut msg.two,
            archive,
        };
        let mut ctx = Ctx {
            entity,
            player_pos: player.map(|link| link.pos),
            player_eye: player.map(|link| {
                Vec3::new(
                    link.pos.x,
                    link.pos.y + PLAYER_HEAD_TOP + HIO_M04,
                    link.pos.z,
                )
            }),
            select,
            talk_xy: false,
            present_end: false,
            talk_command,
            demo_accept,
            cut_proc,
            trig_a: do_now,
            stick_value,
            stick_angle,
            mesg_send_button: send_button,
            timer_status: 0,
            timer_end: 0,
            cursor_hidden: false,
            cursor_shown: false,
            cursor_pos: CURSOR_POS[0][1],
            cursor_scale: [
                CURSOR_SCALE_X,
                CURSOR_SCALE_Y,
                CURSOR_SCALE_Z,
                CURSOR_SCALE_W,
                CURSOR_SCALE_H,
            ],
            mini_rupee: world.counters.mini_game_rupee,
            mini_game_type: btsw.mini_game_type,
            end_mini_game: false,
            stories: &mut world.stories,
            collect: &world.collect,
            ev_flags: &mut world.ev_flags,
            list: &world.list,
            active: &mut world.active,
            events: &mut world.events,
            text: &mut text,
        };
        if !btsw.entered {
            btsw.entered = true;
            btsw.create_init(actor, &mut ctx);
        }
        btsw.execute(actor, &mut ctx);
        world.counters.mini_game_rupee = ctx.mini_rupee;
        btsw.mini_game_type = ctx.mini_game_type;
    }
}
