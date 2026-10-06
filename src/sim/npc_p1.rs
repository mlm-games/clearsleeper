use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, StoryBits};
use super::intent::DoIntent;
use super::player::Player;
use super::stream::{add_calc_angle_s2, cos_s, sin_s};
use super::transition::{NextStage, Target};
use super::walk;
use crate::assets::MsgArchives;
use crate::text::driver::TextFlags;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    DA_EVT_MODE_NONE, FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgFlags, MsgGameInfo,
    MsgProcs, fop_msgm_message_set,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED,
    STATUS_MSG_ENDS,
};

pub const NPC_P1_ACTORS: [&str; 3] = ["P1a", "P1b", "P1c"];

pub fn is_npc_p1(name: &str) -> bool {
    NPC_P1_ACTORS.contains(&name)
}

pub const ARC_NAME: &str = "P1";
pub const STAGE_SEA: &str = "sea";
pub const NEXT_STAGE: &str = "Ocean";
pub const NEXT_STAGE_POINT: u8 = 1;
pub const ROOM_OUTSET_ISLAND: i8 = 44;
pub const START_LAYER_DORA: i32 = 0xA;
pub const SEA_EXP_CAM: &str = "sea_exp_cam";

pub const PARAM_TYPE_SHIFT: u32 = 0x14;
pub const PARAM_TYPE_WIDTH: u32 = 4;
pub const PARAM_TYPE_MASK: u32 = (1 << PARAM_TYPE_WIDTH) - 1;
pub const PARAM_BYTE_MASK: u32 = 0xFF;

pub const TYPE_P1A: u8 = 0;
pub const TYPE_P1B: u8 = 1;
pub const TYPE_P1C: u8 = 2;
pub const TYPE_COUNT: usize = 3;

pub const ACTION_STARTING: i8 = 0;
pub const ACTION_ONGOING: i8 = 1;
pub const ACTION_ENDING: i8 = -1;

pub const NO_INDEX: i32 = -1;
pub const NO_STAFF: usize = usize::MAX;
pub const NO_KAJI: u32 = FPC_M_ERROR_PROCESS_ID;
pub const NO_MSG: u32 = 0xFFFF_FFFF;
pub const NO_ANM: i32 = -1;
pub const NO_MSG_NO: u32 = 0;

pub const EV_0310: u16 = 0x0310;
pub const EV_0808: u16 = 0x0808;
pub const EV_0820: u16 = 0x0820;
pub const EV_0840: u16 = 0x0840;
pub const EV_0880: u16 = 0x0880;
pub const EV_0910: u16 = 0x0910;

pub const MSG_C94: u32 = 0xC94;
pub const MSG_C95: u32 = 0xC95;
pub const MSG_C99: u32 = 0xC99;
pub const MSG_FA1: u32 = 0xFA1;
pub const MSG_FA2: u32 = 0xFA2;
pub const MSG_FA3: u32 = 0xFA3;
pub const MSG_FA4: u32 = 0xFA4;
pub const MSG_FA5: u32 = 0xFA5;
pub const MSG_FA6: u32 = 0xFA6;
pub const MSG_FA7: u32 = 0xFA7;
pub const MSG_FA8: u32 = 0xFA8;
pub const MSG_FAA: u32 = 0xFAA;
pub const MSG_FAB: u32 = 0xFAB;
pub const MSG_FAC: u32 = 0xFAC;
pub const MSG_FAD: u32 = 0xFAD;
pub const MSG_FAE: u32 = 0xFAE;
pub const MSG_FB0: u32 = 0xFB0;
pub const MSG_1007: u32 = 0x1007;
pub const MSG_1009: u32 = 0x1009;
pub const MSG_100A: u32 = 0x100A;
pub const MSG_100B: u32 = 0x100B;
pub const MSG_100C: u32 = 0x100C;
pub const MSG_100D: u32 = 0x100D;
pub const MSG_1014: u32 = 0x1014;
pub const MSG_1015: u32 = 0x1015;
pub const MSG_1016: u32 = 0x1016;
pub const MSG_1017: u32 = 0x1017;
pub const MSG_102E: u32 = 0x102E;
pub const MSG_1033: u32 = 0x1033;
pub const MSG_1034: u32 = 0x1034;

pub const ANM_WAIT: i32 = 0;
pub const ANM_TALK: i32 = 1;
pub const ANM_TALK02: i32 = 2;
pub const ANM_TALK03: i32 = 3;
pub const ANM_C_WAIT: i32 = 4;
pub const ANM_C_TALK01: i32 = 5;
pub const ANM_C_TALK02: i32 = 6;
pub const ANM_C_STOP: i32 = 7;
pub const ANM_C_TALK01_AGAIN: i32 = 8;
pub const ANM_WAIT02: i32 = 9;
pub const ANM_OMOKJ: i32 = 10;
pub const ANM_TORIKJ: i32 = 11;
pub const ANM_ANGRY: i32 = 12;
pub const ANM_TALK04: i32 = 13;
pub const ANM_CHECK01: i32 = 14;
pub const ANM_CHECK02: i32 = 15;
pub const ANM_LOOK: i32 = 16;

pub const EMODE_NONE: u8 = 0;
pub const EMODE_LOOP: u8 = 2;
pub const MORF_PLAY_SPEED: f32 = 1.0;
pub const MORF_START_FRAME: f32 = 0.0;

pub const FRAME_WAIT: i16 = 39;
pub const FRAME_WAIT02: i16 = 49;
pub const FRAME_TALK: i16 = 78;
pub const FRAME_TALK02: i16 = 39;
pub const FRAME_TALK03: i16 = 59;
pub const FRAME_TALK04: i16 = 40;
pub const FRAME_OMOKJ: i16 = 98;
pub const FRAME_TORIKJ: i16 = 98;
pub const FRAME_ANGRY: i16 = 49;
pub const FRAME_CHECK01: i16 = 100;
pub const FRAME_CHECK02: i16 = 67;
pub const FRAME_LOOK: i16 = 60;
pub const FRAME_C_WAIT: i16 = 39;
pub const FRAME_C_TALK01: i16 = 49;
pub const FRAME_C_TALK02: i16 = 59;
pub const FRAME_C_STOP: i16 = 39;

pub const BTP_FRAME_A: i16 = 4;
pub const BTP_FRAME_B: i16 = 4;
pub const BTP_FRAME_C: i16 = 5;

pub const GRAVITY: f32 = -9.0;
pub const CYL_RADIUS: f32 = 40.0;
pub const CYL_HEIGHT: f32 = 160.0;
pub const CYL_RADIUS_P1A_REG: f32 = 90.0;
pub const CYL_RADIUS_P1B_PARAM2: f32 = 90.0;
pub const CYL_RADIUS_P1C: f32 = 100.0;
pub const STTS_ATP: u8 = 0xFF;
pub const STTS_TP: u8 = 0xFF;
pub const ATTN_TYPE_TALK: u32 = 0xAA;
pub const ATTN_TYPE_TALK_P1A: u32 = 0xAB;
pub const ATTN_FLAG_LOCKON_TALK: u32 = 0x0001;
pub const ATTN_FLAG_ACTION_SPEAK: u32 = 0x0002;
pub const ATTN_FLAGS: u32 = ATTN_FLAG_LOCKON_TALK | ATTN_FLAG_ACTION_SPEAK;
pub const CAM_ATTENTION_FLAG: u32 = 4;
pub const REG10_F5: f32 = 0.0;

pub const TURN_SCALE: i16 = 8;
pub const TURN_MAX_STEP: i16 = 0x400;
pub const LOOK_SCALE: i16 = 4;
pub const LOOK_MAX_STEP: i16 = 0x800;
pub const LOOK_SRC_LIFT: f32 = 190.0;
pub const PLAYER_EYE_OFFSET_Y: f32 = 0.0;
pub const RANGE_SPEAK: f32 = 400.0;
pub const RANGE_P1C: f32 = 200.0;
pub const MORF_TALK_SPEED: f32 = 2.0;
pub const MORF_P1C_CLOSE: f32 = 8.0;
pub const MORF_ANGRY_TO_WAIT02: f32 = 8.0;
pub const MORF_AGAIN_TO_WAIT: f32 = 15.0;

pub const KAJI_TIMER: i16 = 300;
pub const KAJI_WHEEL_FRAME: f32 = 30.0;
pub const KAJI_ANM_BIAS: i32 = 9;
pub const BLINK_TIMER_MIN: i16 = 0x1E;
pub const BLINK_RANDOM: f32 = 100.0;
pub const HEAD_SWING_COUNT: i16 = 2;
pub const HEAD_SWING_RATE: i16 = 0x1000;
pub const HEAD_SWING_AMPLITUDE: i16 = 0x800;
pub const HEAD_SWING_AGAIN: i32 = 1;
pub const HEAD_CHASE_SCALE: i16 = 4;
pub const HEAD_CHASE_MAX: i16 = 0x1000;
pub const HEAD_CHASE_MIN: i16 = 0x100;
pub const HEAD_CHASE_STEP: i16 = 0x60;
pub const HEAD_FOLLOW_SCALE: i16 = 4;

pub const P1C_HOME_SCALE: f32 = 0.75;
pub const P1C_HOME_MAX_STEP: f32 = 5.0;
pub const P1C_HOME_EPS: f32 = 1.0;

pub const DEMO_POS_LIFT: f32 = 700.0;
pub const DEMO_POS_SPREAD: f32 = 950.0;
pub const DEMO_POS_SIDE: f32 = 150.0;
pub const DEMO_ANGLE: i16 = 0x4000;

pub const SE_PIRATE_WHEEL: u32 = 0x697E;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageStart {
    pub point: i16,
    pub name: &'static str,
    pub layer: i32,
    pub room: i8,
    pub kaji: Option<u32>,
}

impl Default for StageStart {
    fn default() -> Self {
        Self {
            point: 0,
            name: "",
            layer: 0,
            room: ROOM_OUTSET_ISLAND,
            kaji: None,
        }
    }
}

pub const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0];
pub const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0];

pub const MINIGAME_ACTION_NAMES: [&str; 2] = ["4013_msg", "4014_msg"];
pub const PRIVATE_CUT_NAMES: [&str; 5] = [
    "4013_msg",
    "4014_msg",
    "SETANM",
    "HEADSWING",
    "TALKMSG",
];
pub const PRIVATE_ACT_MINIGAME_A: i32 = 0;
pub const PRIVATE_ACT_MINIGAME_B: i32 = 1;
pub const PRIVATE_ACT_SETANM: i32 = 2;
pub const PRIVATE_ACT_HEADSWING: i32 = 3;
pub const PRIVATE_ACT_TALKMSG: i32 = 4;
pub const MINIGAME_ACT_SECOND: i32 = 1;

pub const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
pub const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
pub const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
pub const RND_SEED: i32 = 100;

pub fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = (seed[0]) as f32 / RND_SCALE[0]
        + (seed[1]) as f32 / RND_SCALE[1]
        + (seed[2]) as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

fn abs_xz(value: Vec3) -> f32 {
    (value.x * value.x + value.z * value.z).sqrt()
}

fn abs_xyz(value: Vec3) -> f32 {
    value.x.abs() + value.y.abs() + value.z.abs()
}

fn distance_angle_s(x: i16, y: i16) -> i32 {
    i32::from(x.wrapping_sub(y)).abs()
}

fn add_calc_pos2(value: &mut Vec3, target: Vec3, scale: f32, max_step: f32) {
    if *value != target {
        let mut diff = (*value - target) * scale;
        if abs_xyz(diff) > max_step {
            diff = Vec3::new(diff.x, diff.y, 0.0).normalize_or_zero() * max_step;
        }
        *value -= diff;
    }
}

fn add_calc_angle_l(value: &mut i32, target: i32, scale: i32, max_step: i32, min_step: i32) {
    let diff = target.wrapping_sub(*value);
    if *value != target {
        let mut step = diff / scale;
        if step > min_step || step < -min_step {
            step = step.clamp(-max_step, max_step);
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChildHio {
    pub attn_y_pos_offset: f32,
    pub max_head_x: i16,
    pub min_head_x: i16,
    pub max_backbone_x: i16,
    pub min_backbone_x: i16,
    pub max_head_y: i16,
    pub min_head_y: i16,
    pub max_backbone_y: i16,
    pub min_backbone_y: i16,
    pub max_turn_step: i16,
    pub look_back_target_y: i16,
    pub unused_20: i8,
    pub max_talk_dist: f32,
    pub morf_backup: f32,
    pub unused_2c: f32,
}

const HIO_P1A: ChildHio = ChildHio {
    attn_y_pos_offset: 50.0,
    max_head_x: 0x9C4,
    min_head_x: 0xf63cu16 as i16,
    max_backbone_x: 2000,
    min_backbone_x: 0xf830u16 as i16,
    max_head_y: 5000,
    min_head_y: 0xec78u16 as i16,
    max_backbone_y: 8000,
    min_backbone_y: 0xe0c0u16 as i16,
    max_turn_step: 1000,
    look_back_target_y: 0x708,
    unused_20: 0,
    max_talk_dist: 300.0,
    morf_backup: 8.0,
    unused_2c: 8.0,
};

const HIO_P1B: ChildHio = ChildHio {
    attn_y_pos_offset: 45.0,
    max_head_x: 0x9C4,
    min_head_x: 0xf63cu16 as i16,
    max_backbone_x: 2000,
    min_backbone_x: 0xf830u16 as i16,
    max_head_y: 10000,
    min_head_y: 0xd8f0u16 as i16,
    max_backbone_y: 8000,
    min_backbone_y: 0xe0c0u16 as i16,
    max_turn_step: 1000,
    look_back_target_y: 0x708,
    unused_20: 0,
    max_talk_dist: 250.0,
    morf_backup: 8.0,
    unused_2c: 8.0,
};

const HIO_P1C: ChildHio = ChildHio {
    attn_y_pos_offset: 45.0,
    max_head_x: 0x9C4,
    min_head_x: 0xf63cu16 as i16,
    max_backbone_x: 2000,
    min_backbone_x: 0xf830u16 as i16,
    max_head_y: 5000,
    min_head_y: 0xec78u16 as i16,
    max_backbone_y: 8000,
    min_backbone_y: 0xe0c0u16 as i16,
    max_turn_step: 1000,
    look_back_target_y: 0x708,
    unused_20: 0,
    max_talk_dist: 250.0,
    morf_backup: 7.0,
    unused_2c: 16.0,
};

pub const HIO_CHILDREN: [ChildHio; TYPE_COUNT] = [HIO_P1A, HIO_P1B, HIO_P1C];

pub fn hio(kind: u8) -> ChildHio {
    HIO_CHILDREN
        .get(usize::from(kind))
        .copied()
        .unwrap_or(HIO_P1A)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    None,
    Normal,
    Confuse,
    Talk,
    P1cSpeak,
    Speak,
    Explain,
}

pub const STATE_STOP: u8 = 1;
pub const STATE_LOOP: u8 = 2;

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
    pub fn idle() -> Self {
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

    fn set_anm(&mut self, end: i16, attribute: u8, morf: f32) {
        self.start = 0;
        self.end = end;
        self.loop_point = 0;
        self.rate = MORF_PLAY_SPEED;
        self.frame = MORF_START_FRAME;
        self.attribute = attribute;
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
        if self.attribute == EMODE_NONE {
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
        } else if self.attribute == EMODE_LOOP {
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
    }

    fn play(&mut self) {
        if self.cur_morf < 1.0 {
            self.prev_morf = self.cur_morf;
            walk::chase_f(&mut self.cur_morf, 1.0, self.morf_step);
        }
        self.update();
    }

    pub fn get_frame(&self) -> f32 {
        self.frame
    }

    pub fn get_end_frame(&self) -> f32 {
        f32::from(self.end)
    }

    pub fn check_frame(&self, frame: f32) -> bool {
        (self.frame - frame).abs() < 1.0
    }
}

pub const JNT_HEAD: usize = 0;
pub const JNT_BACKBONE: usize = 1;
pub const JNT_AXIS_X: usize = 0;
pub const JNT_AXIS_Y: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jnt {
    pub angles: [[i16; 2]; 2],
    pub min_angles: [[i16; 2]; 2],
    pub max_angles: [[i16; 2]; 2],
    pub max_turn_step: [[i16; 2]; 2],
    pub trn: bool,
    pub head_lock: bool,
    pub back_bone_lock: bool,
}

impl Jnt {
    pub fn new() -> Self {
        Self {
            angles: [[0; 2]; 2],
            min_angles: [[0; 2]; 2],
            max_angles: [[0; 2]; 2],
            max_turn_step: [[0; 2]; 2],
            trn: false,
            head_lock: false,
            back_bone_lock: false,
        }
    }

    pub fn on_head_lock(&mut self) {
        self.head_lock = true;
    }

    pub fn off_head_lock(&mut self) {
        self.head_lock = false;
    }

    pub fn on_back_bone_lock(&mut self) {
        self.back_bone_lock = true;
    }

    pub fn off_back_bone_lock(&mut self) {
        self.back_bone_lock = false;
    }

    pub fn trn_chk(&self) -> bool {
        self.trn
    }

    pub fn set_trn(&mut self) {
        self.trn = true;
    }

    pub fn set_param(&mut self, child: ChildHio) {
        self.max_angles[JNT_BACKBONE][JNT_AXIS_X] = child.max_backbone_x;
        self.max_angles[JNT_BACKBONE][JNT_AXIS_Y] = child.max_backbone_y;
        self.min_angles[JNT_BACKBONE][JNT_AXIS_X] = child.min_backbone_x;
        self.min_angles[JNT_BACKBONE][JNT_AXIS_Y] = child.min_backbone_y;
        self.max_angles[JNT_HEAD][JNT_AXIS_X] = child.max_head_x;
        self.max_angles[JNT_HEAD][JNT_AXIS_Y] = child.max_head_y;
        self.min_angles[JNT_HEAD][JNT_AXIS_X] = child.min_head_x;
        self.min_angles[JNT_HEAD][JNT_AXIS_Y] = child.min_head_y;
        self.max_turn_step[JNT_BACKBONE][JNT_AXIS_X] = child.max_turn_step;
        self.max_turn_step[JNT_BACKBONE][JNT_AXIS_Y] = child.max_turn_step;
        self.max_turn_step[JNT_HEAD][JNT_AXIS_X] = child.max_turn_step;
        self.max_turn_step[JNT_HEAD][JNT_AXIS_Y] = child.max_turn_step;
    }

    fn limitter(&self, value: &mut i16, joint: usize, axis: usize) {
        if *value > self.max_angles[joint][axis] {
            *value = self.max_angles[joint][axis];
        }
        if *value < self.min_angles[joint][axis] {
            *value = self.min_angles[joint][axis];
        }
    }

    fn ang_calc_s(&mut self, out: &mut i16, target: i16, speed: i16, max_vel: i16) -> bool {
        let mut diff = out.wrapping_sub(target);
        let orig_diff = diff;
        diff = walk::add_calc_angle_s(diff, 0, speed, max_vel, HEAD_CHASE_STEP);
        if i32::from(diff).abs() > i32::from(speed) {
            *out = out.wrapping_add(diff.wrapping_sub(orig_diff));
        } else {
            *out = target;
        }
        target.wrapping_sub(*out) == 0
    }

    fn follow(&mut self, out: &mut i16, target: i16, max_vel: i16, axis: usize) -> bool {
        let orig = *out;
        self.ang_calc_s(out, target, HEAD_FOLLOW_SCALE, max_vel);
        let delta = out.wrapping_sub(orig);
        let left = target.wrapping_sub(*out);
        let mut sum_prev = 0_i16;
        let mut sum = 0_i16;
        for joint in (0..2).rev() {
            sum = sum.wrapping_add(self.angles[joint][axis]);
            if (delta > 0 && sum > left) || (delta < 0 && sum < left) {
                let mut value = left.wrapping_sub(sum_prev);
                self.limitter(&mut value, joint, axis);
                self.angles[joint][axis] = value;
            }
            sum_prev = sum_prev.wrapping_add(self.angles[joint][axis]);
        }
        target != *out
    }

    fn jnt_move(&mut self, mut delta: i16, axis: usize) -> bool {
        let mut assigned = [0_i16; 2];
        for joint in 0..2 {
            let mut angle = if (joint == JNT_HEAD && self.head_lock)
                || (joint == JNT_BACKBONE && self.back_bone_lock)
            {
                0
            } else {
                delta
            };
            self.limitter(&mut angle, joint, axis);
            delta = delta.wrapping_sub(angle);
            assigned[joint] = angle;
            let mut current = i32::from(self.angles[joint][axis]);
            let max_step = i32::from(self.max_turn_step[joint][axis]);
            add_calc_angle_l(&mut current, i32::from(angle), 4, max_step, 1);
            self.angles[joint][axis] = current as i16;
        }
        let mut settled = 0_i32;
        for joint in 0..2 {
            if distance_angle_s(assigned[joint], self.angles[joint][axis])
                <= i32::from(self.max_turn_step[joint][axis]) / 2
            {
                settled += 1;
            }
        }
        settled == 2
    }

    fn look_at_target(
        &mut self,
        out_y: &mut i16,
        dst: Option<Vec3>,
        src: Vec3,
        default_y: i16,
        max_vel: i16,
        hold_body: bool,
    ) {
        let (target_y, target_x) = match dst {
            Some(point) => {
                let delta = point - src;
                let dist_xz = (delta.x * delta.x + delta.z * delta.z).sqrt();
                (
                    walk::to_s16(delta.x.atan2(delta.z)),
                    walk::to_s16(delta.y.atan2(dist_xz)),
                )
            }
            None => (default_y, 0),
        };
        let delta_y = target_y.wrapping_sub(*out_y);
        let max_y = i32::from(self.max_angles[JNT_BACKBONE][JNT_AXIS_Y])
            .wrapping_add(i32::from(self.max_angles[JNT_HEAD][JNT_AXIS_Y]));
        let min_y = i32::from(self.min_angles[JNT_BACKBONE][JNT_AXIS_Y])
            .wrapping_add(i32::from(self.min_angles[JNT_HEAD][JNT_AXIS_Y]));
        let wide = i32::from(delta_y);
        let settled = self.jnt_move(delta_y, JNT_AXIS_Y);
        if (self.trn || (wide >= max_y || wide <= min_y) && settled) && !hold_body {
            self.trn = self.follow(out_y, target_y, max_vel, JNT_AXIS_Y);
        }
        self.jnt_move(target_x, JNT_AXIS_X);
    }
}

impl Default for Jnt {
    fn default() -> Self {
        Self::new()
    }
}

pub const SWING_NONE: u8 = 0;
pub const SWING_VERTICAL: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeadAnm {
    pub swing: [i16; 2],
    pub proc_state: u8,
    pub amplitude: f32,
    pub rate: i16,
    pub count: i16,
    pub phase: i16,
}

impl HeadAnm {
    pub fn new() -> Self {
        Self {
            swing: [0; 2],
            proc_state: SWING_NONE,
            amplitude: 0.0,
            rate: 0,
            count: 0,
            phase: 0,
        }
    }

    pub fn swing_vertical_init(&mut self, count: i16, rate: i16, amplitude: i16, again: i32) {
        if again == 0 || self.proc_state != SWING_VERTICAL {
            self.phase = 0;
            self.count = count;
            self.rate = rate;
            self.amplitude = f32::from(amplitude);
            self.proc_state = SWING_VERTICAL;
        }
    }

    fn swing_vertical(&mut self) {
        let prev = self.phase;
        self.phase = self.phase.wrapping_add(self.rate);
        let temp = (sin_s(self.phase) * self.amplitude) as i16;
        self.swing[JNT_AXIS_X] =
            walk::add_calc_angle_s(self.swing[JNT_AXIS_X], temp, HEAD_CHASE_SCALE, HEAD_CHASE_MAX, HEAD_CHASE_MIN);
        self.swing[JNT_AXIS_Y] =
            walk::add_calc_angle_s(self.swing[JNT_AXIS_Y], 0, HEAD_CHASE_SCALE, HEAD_CHASE_MAX, HEAD_CHASE_MIN);
        if prev < 0 && self.phase >= 0 {
            self.count = self.count.wrapping_sub(1);
            if self.count <= 0 {
                self.proc_state = SWING_NONE;
            }
        }
    }

    pub fn head_move(&mut self) {
        if self.proc_state == SWING_VERTICAL {
            self.swing_vertical();
        } else {
            self.swing[JNT_AXIS_X] =
                walk::add_calc_angle_s(self.swing[JNT_AXIS_X], 0, HEAD_CHASE_SCALE, HEAD_CHASE_MAX, HEAD_CHASE_MIN);
            self.swing[JNT_AXIS_Y] =
                walk::add_calc_angle_s(self.swing[JNT_AXIS_Y], 0, HEAD_CHASE_SCALE, HEAD_CHASE_MAX, HEAD_CHASE_MIN);
        }
    }
}

impl Default for HeadAnm {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DemoActor {
    pub btp_frame_max: u8,
}

pub struct Text<'a> {
    pub procs: &'a mut MsgProcs,
    pub info: &'a mut MsgGameInfo,
    pub flags: &'a mut MsgFlags,
    pub one: &'a mut Msg1Box,
    pub two: &'a mut Msg2Box,
    pub clear_count: u8,
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

    fn status(&self) -> Option<u8> {
        match self.proc_name() {
            Some(FPC_NM_MSG) => Some(self.one.status),
            Some(FPC_NM_MSG2) => Some(self.two.status),
            _ => None,
        }
    }

    fn set_status(&mut self, status: u8) {
        match self.proc_name() {
            Some(FPC_NM_MSG) => self.one.status = status,
            Some(FPC_NM_MSG2) => self.two.status = status,
            _ => {}
        }
    }

    fn select_num(&self) -> u8 {
        match self.proc_name() {
            Some(FPC_NM_MSG) => self.one.select_num,
            _ => 0,
        }
    }

    fn mesg_anime(&self) -> u8 {
        self.info.mesg_anime
    }

    fn clear_mesg_anime(&mut self) {
        self.info.set_mesg_anime_attr_info(0xFF);
    }

    fn message_set(&mut self, msg_no: u32, look_at: Vec3) -> u32 {
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
            look_at,
        )
    }

    fn bound(&self) -> bool {
        self.proc_name().is_some()
    }
}

pub struct Ctx<'a> {
    pub entity: Entity,
    pub parameters: u32,
    pub player: Option<Vec3>,
    pub player_eye: Vec3,
    pub pos: Vec3,
    pub angle: i16,
    pub room: i8,
    pub start_point: i16,
    pub event_mode: bool,
    pub talk_command: bool,
    pub send_button: bool,
    pub staff_cut: Option<(usize, bool)>,
    pub sea_exp_cam: Option<usize>,
    pub sea_exp_end: bool,
    pub cut_attn: bool,
    pub cam_attention: bool,
    pub pirate: Option<Entity>,
    pub pirate_pos: Vec3,
    pub pirate_angle: i16,
    pub pirate_kaji: Option<u32>,
    pub pirate_moved: Option<(Entity, Vec3, i16)>,
    pub demo: Option<DemoActor>,
    pub stories: &'a mut StoryBits,
    pub events: &'a mut SimEvents,
    pub list: &'a EventList,
    pub active: &'a mut ActiveEvent,
    pub next: &'a mut NextStage,
    pub text: &'a mut Text<'a>,
}

impl Ctx<'_> {
    fn command_talk(&self) -> bool {
        self.talk_command
    }

    fn reset_event(&mut self) {
        self.text.info.event_mode = DA_EVT_MODE_NONE;
    }

    fn order_sea_exp_cam(&mut self) {
        let Some(index) = self.sea_exp_cam else {
            return;
        };
        if self.active.stage_of(index).is_none() {
            self.active.order(self.list, index, self.entity);
        }
    }

    fn set_next_stage(&mut self) {
        self.next.target = Some(Target {
            stage: NEXT_STAGE.to_owned(),
            room: u8::try_from(self.room).unwrap_or(0),
            start: NEXT_STAGE_POINT,
        });
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct NpcP1 {
    pub act: Act,
    pub prev_act: Act,
    pub act_status: i8,
    pub cut_state: i8,
    pub kind: u8,
    pub param: u8,
    pub curr_mesg: u32,
    pub prev_mesg: u32,
    pub anm_num: i32,
    pub counter_66c: i32,
    pub counter_670: u8,
    pub counter_671: u8,
    pub kaji_id: u32,
    pub kaji_timer: i16,
    pub jnt: Jnt,
    pub head: HeadAnm,
    pub morph: Frame,
    pub blink_frame: u8,
    pub blink_timer: i16,
    pub attn_flag: bool,
    pub visible: bool,
    pub max_look_vel: i16,
    pub eye_pos: Vec3,
    pub home: Vec3,
    pub home_angle: i16,
    pub attn_pos: Vec3,
    pub attn_flags: u32,
    pub attn_dist: u32,
    pub cyl_radius: f32,
    pub msg_id: u32,
    pub msg_bound: bool,
    pub talk_condition: bool,
    pub talk_command: bool,
    pub speak_ordered: bool,
    pub skip_smoother: bool,
    pub set_kaji: Option<(i32, f32)>,
    pub kaji_anm: i32,
    pub kaji_frame: f32,
    pub staff_slot: usize,
    pub end_staff: bool,
    pub cam_attention: bool,
    pub cut_proc: bool,
    pub cut_attn: bool,
    pub cut_attn_pos: Vec3,
    pub seed: [i32; 3],
    pub staff_name: &'static str,
    pub start_point: i16,
    pub entered: bool,
    pub do_held: bool,
}

impl NpcP1 {
    pub fn spawn(parameters: u32) -> Self {
        let kind = match (parameters >> PARAM_TYPE_SHIFT) & PARAM_TYPE_MASK {
            0 => TYPE_P1A,
            1 => TYPE_P1B,
            _ => TYPE_P1C,
        };
        let staff_name = staff_name_of(kind);
        Self {
            act: Act::None,
            prev_act: Act::None,
            act_status: ACTION_STARTING,
            cut_state: 0,
            kind,
            param: (parameters & PARAM_BYTE_MASK) as u8,
            curr_mesg: NO_MSG_NO,
            prev_mesg: NO_MSG_NO,
            anm_num: NO_ANM,
            counter_66c: 0,
            counter_670: 0,
            counter_671: 0,
            kaji_id: NO_KAJI,
            kaji_timer: KAJI_TIMER,
            jnt: Jnt::new(),
            head: HeadAnm::new(),
            morph: Frame::idle(),
            blink_frame: 0,
            blink_timer: 0,
            attn_flag: false,
            visible: true,
            max_look_vel: 0,
            eye_pos: Vec3::ZERO,
            home: Vec3::ZERO,
            home_angle: 0,
            attn_pos: Vec3::ZERO,
            attn_flags: ATTN_FLAGS,
            attn_dist: ATTN_TYPE_TALK,
            cyl_radius: CYL_RADIUS,
            msg_id: FPC_M_ERROR_PROCESS_ID,
            msg_bound: false,
            talk_condition: false,
            talk_command: false,
            speak_ordered: false,
            skip_smoother: false,
            set_kaji: None,
            kaji_anm: NO_ANM,
            kaji_frame: 0.0,
            staff_slot: NO_STAFF,
            end_staff: false,
            cam_attention: false,
            cut_proc: false,
            cut_attn: false,
            cut_attn_pos: Vec3::ZERO,
            seed: [RND_SEED; 3],
            staff_name,
            start_point: 0,
            entered: false,
            do_held: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.visible
    }

    fn refresh_visible(&mut self, stories: &mut StoryBits) {
        self.visible = !(self.kind == TYPE_P1A && !stories.test(EV_0310));
    }

    fn btp_frame_max(&self) -> i16 {
        match self.kind {
            TYPE_P1A => BTP_FRAME_A,
            TYPE_P1B => BTP_FRAME_B,
            _ => BTP_FRAME_C,
        }
    }

    fn anm_table(&self, anm: i32) -> Option<(i16, u8)> {
        match anm {
            ANM_WAIT => Some((FRAME_WAIT, EMODE_LOOP)),
            ANM_TALK => Some((FRAME_TALK, EMODE_LOOP)),
            ANM_TALK02 => Some((FRAME_TALK02, EMODE_LOOP)),
            ANM_TALK03 => Some((FRAME_TALK03, EMODE_LOOP)),
            ANM_C_WAIT => Some((FRAME_C_WAIT, EMODE_LOOP)),
            ANM_C_TALK01 => Some((FRAME_C_TALK01, EMODE_LOOP)),
            ANM_C_TALK02 => Some((FRAME_C_TALK02, EMODE_NONE)),
            ANM_C_STOP => Some((FRAME_C_STOP, EMODE_LOOP)),
            ANM_C_TALK01_AGAIN => Some((FRAME_C_TALK01, EMODE_LOOP)),
            ANM_WAIT02 => Some((FRAME_WAIT02, EMODE_LOOP)),
            ANM_OMOKJ => Some((FRAME_OMOKJ, EMODE_NONE)),
            ANM_TORIKJ => Some((FRAME_TORIKJ, EMODE_NONE)),
            ANM_ANGRY => Some((FRAME_ANGRY, EMODE_LOOP)),
            ANM_TALK04 => Some((FRAME_TALK04, EMODE_LOOP)),
            ANM_CHECK01 => Some((FRAME_CHECK01, EMODE_LOOP)),
            ANM_CHECK02 => Some((FRAME_CHECK02, EMODE_LOOP)),
            ANM_LOOK => Some((FRAME_LOOK, EMODE_LOOP)),
            _ => None,
        }
    }

    pub fn set_anm(&mut self, anm: i32, morf: f32) -> bool {
        if self.anm_num == anm {
            return false;
        }
        self.anm_num = anm;
        let morf = if morf < 0.0 {
            hio(self.kind).morf_backup
        } else {
            morf
        };
        if let Some((end, attr)) = self.anm_table(anm) {
            self.morph.set_anm(end, attr, morf);
        }
        true
    }

    pub fn check_act(&self, act: Act) -> bool {
        self.act == act
    }

    pub fn set_act(&mut self, act: Act, ctx: &mut Ctx) {
        if self.act != act {
            if self.act != Act::None {
                self.act_status = ACTION_ENDING;
                self.run_act(ctx);
            }
            self.prev_act = self.act;
            self.act = act;
            self.act_status = ACTION_STARTING;
            self.run_act(ctx);
        }
    }

    fn run_act(&mut self, ctx: &mut Ctx) -> bool {
        match self.act {
            Act::Normal => self.normal_action(ctx),
            Act::Confuse => self.confuse_action(ctx),
            Act::Talk => self.talk_action(ctx),
            Act::P1cSpeak => self.p1c_speak_action(ctx),
            Act::Speak => self.speak_action(ctx),
            Act::Explain => self.explain_action(ctx),
            Act::None => false,
        }
    }

    pub fn set_anim_from_msg(&mut self, ctx: &mut Ctx) {
        let anime = ctx.text.mesg_anime();
        if (self.anm_num == ANM_OMOKJ || self.anm_num == ANM_TORIKJ)
            && !self.morph.check_frame(self.morph.get_end_frame() - 1.0)
        {
            return;
        }
        if self.anm_num == ANM_C_TALK01_AGAIN
            && self.morph.check_frame(self.morph.get_end_frame() - 1.0)
        {
            self.counter_66c -= 1;
            if self.counter_66c <= 0 {
                self.set_anm(ANM_C_WAIT, MORF_AGAIN_TO_WAIT);
            }
        } else if self.anm_num == ANM_ANGRY
            && self.morph.check_frame(self.morph.get_end_frame() - 1.0)
        {
            self.counter_66c -= 1;
            if self.counter_66c <= 0 {
                self.set_anm(ANM_WAIT02, MORF_ANGRY_TO_WAIT02);
            }
        }
        let anm = match anime {
            0 => {
                if self.kind != TYPE_P1C {
                    ANM_WAIT
                } else {
                    ANM_C_WAIT
                }
            }
            1 => ANM_TALK,
            2 => ANM_TALK02,
            3 => ANM_TALK03,
            4 => {
                if self.kind != TYPE_P1C {
                    ANM_WAIT02
                } else {
                    ANM_C_WAIT
                }
            }
            5 => ANM_OMOKJ,
            6 => ANM_TORIKJ,
            7 => {
                self.counter_66c = 4;
                ANM_ANGRY
            }
            8 => ANM_TALK04,
            9 => ANM_CHECK01,
            10 => ANM_CHECK02,
            11 => ANM_LOOK,
            12 => ANM_C_WAIT,
            13 => ANM_C_TALK01,
            14 => ANM_C_TALK02,
            15 => ANM_C_STOP,
            16 => {
                self.counter_66c = 6;
                ANM_C_TALK01_AGAIN
            }
            _ => return,
        };
        ctx.text.clear_mesg_anime();
        self.set_anm(anm, -1.0);
    }

    fn normal_action(&mut self, ctx: &mut Ctx) -> bool {
        let dist = ctx
            .player
            .map_or(0.0, |link| abs_xz(link - ctx.pos));
        if self.act_status == ACTION_STARTING {
            if self.kind == TYPE_P1A {
                if self.counter_671 == 1 {
                    if self.morph.check_frame(self.morph.get_end_frame() - 1.0) {
                        self.set_anm(ANM_WAIT02, -1.0);
                    } else {
                        return false;
                    }
                } else {
                    self.set_anm(ANM_WAIT, -1.0);
                }
            } else if self.kind == TYPE_P1B {
                if self.param == 2 {
                    self.set_anm(ANM_CHECK01, -1.0);
                } else if self.param == 3 {
                    self.set_anm(ANM_LOOK, -1.0);
                } else {
                    self.set_anm(ANM_WAIT, -1.0);
                }
            } else {
                self.set_anm(ANM_C_WAIT, -1.0);
            }
            self.act_status += 1;
        } else if self.act_status != ACTION_ENDING {
            if ctx.command_talk() {
                self.set_act(Act::Talk, ctx);
            }
            if dist < hio(self.kind).max_talk_dist {
                self.talk_condition = true;
            }
            if self.kind == TYPE_P1C {
                add_calc_angle_s2(&mut ctx.angle, self.home_angle, TURN_SCALE, TURN_MAX_STEP);
            }
        }
        true
    }

    fn confuse_action(&mut self, _ctx: &mut Ctx) -> bool {
        if self.act_status == ACTION_STARTING {
            if self.kind == TYPE_P1A {
                self.set_anm(ANM_TALK, -1.0);
                self.morph.rate = MORF_TALK_SPEED;
            } else {
                self.set_anm(ANM_TALK02, -1.0);
                self.morph.rate = MORF_TALK_SPEED;
            }
            self.act_status += 1;
        } else if self.morph.check_frame(self.morph.get_end_frame() - 2.0) {
            self.counter_66c += 1;
            if self.counter_66c > 3 {
                if self.anm_num == ANM_TALK {
                    self.set_anm(ANM_TALK02, -1.0);
                    self.morph.rate = MORF_TALK_SPEED;
                } else {
                    self.set_anm(ANM_TALK, -1.0);
                    self.morph.rate = MORF_TALK_SPEED;
                }
                self.counter_66c = 0;
            }
        }
        true
    }

    fn talk_action(&mut self, ctx: &mut Ctx) -> bool {
        if self.act_status == ACTION_STARTING {
            if self.anm_num == ANM_OMOKJ || self.anm_num == ANM_TORIKJ {
                return false;
            }
            self.prev_mesg = self.curr_mesg;
            if self.kind == TYPE_P1A {
                if self.param == 1 {
                    if ctx.stories.test(EV_0910) {
                        self.curr_mesg = MSG_C95;
                    } else {
                        ctx.stories.set(EV_0910);
                        self.curr_mesg = MSG_C94;
                    }
                } else if self.param == 3 {
                    self.curr_mesg = MSG_100A;
                } else if !ctx.stories.test(EV_0880) {
                    self.curr_mesg = MSG_1007;
                    ctx.stories.set(EV_0880);
                } else {
                    self.curr_mesg = MSG_1009;
                }
            } else if self.kind == TYPE_P1B {
                if self.param == 1 {
                    self.curr_mesg = MSG_FA1;
                } else if self.param == 3 {
                    self.curr_mesg = MSG_100D;
                } else if !ctx.stories.test(EV_0840) {
                    self.curr_mesg = MSG_100B;
                    ctx.stories.set(EV_0840);
                } else if ctx.text.clear_count == 0 {
                    self.curr_mesg = MSG_100C;
                } else {
                    self.curr_mesg = MSG_1034;
                }
                if self.param == 2 {
                    self.jnt.off_head_lock();
                }
            } else if ctx.stories.test(EV_0808) {
                self.curr_mesg = MSG_1017;
            } else if !ctx.stories.test(EV_0820) {
                self.curr_mesg = MSG_1014;
                ctx.stories.set(EV_0820);
            } else {
                self.curr_mesg = MSG_1015;
            }
            self.act_status += 1;
            self.msg_id = FPC_M_ERROR_PROCESS_ID;
        } else if self.act_status != ACTION_ENDING {
            if self.msg_id == FPC_M_ERROR_PROCESS_ID {
                let eye = self.eye_pos;
                self.msg_id = ctx.text.message_set(self.curr_mesg, eye);
            } else {
                if self.kind == TYPE_P1B {
                    if self.param != 2 && self.param != 3 {
                        self.jnt.set_trn();
                    }
                } else if self.kind == TYPE_P1A {
                    if self.param != 2 {
                        self.jnt.set_trn();
                    }
                } else {
                    self.jnt.set_trn();
                }
                self.set_anim_from_msg(ctx);
                if self.act_status == ACTION_ONGOING {
                    self.msg_bound = ctx.text.bound();
                    if self.msg_bound {
                        self.act_status += 1;
                    }
                } else if ctx.text.status() == Some(STATUS_MSG_DISPLAYED) {
                    if self.get_next_msg_no(ctx, 1) != NO_MSG {
                        ctx.text.set_status(STATUS_MSG_CONTINUES);
                        let eye = self.eye_pos;
                        ctx.text.message_set(self.curr_mesg, eye);
                    } else {
                        ctx.text.set_status(STATUS_MSG_ENDS);
                    }
                } else if ctx.text.status() == Some(STATUS_BOX_CLOSED) {
                    if self.param == 2 {
                        self.jnt.on_head_lock();
                    }
                    ctx.text.set_status(STATUS_MSG_DESTROYED);
                    let prev = self.prev_act;
                    self.set_act(prev, ctx);
                    ctx.reset_event();
                }
            }
        }
        true
    }

    fn p1c_speak_action(&mut self, ctx: &mut Ctx) -> bool {
        if self.act_status == ACTION_STARTING {
            if !ctx.command_talk() {
                let near = ctx
                    .player
                    .is_some_and(|link| abs_xz(link - self.home) < RANGE_P1C);
                if near {
                    self.speak_ordered = true;
                    self.talk_condition = true;
                }
                return false;
            }
            self.curr_mesg = MSG_1014;
            self.prev_mesg = 0;
            self.act_status += 1;
            self.msg_bound = false;
            self.msg_id = NO_MSG;
        } else if self.act_status != ACTION_ENDING {
            if self.msg_id == FPC_M_ERROR_PROCESS_ID {
                let eye = self.eye_pos;
                self.msg_id = ctx.text.message_set(self.curr_mesg, eye);
            } else {
                self.set_anim_from_msg(ctx);
                if !self.msg_bound {
                    self.msg_bound = ctx.text.bound();
                } else if ctx.text.status() == Some(STATUS_MSG_DISPLAYED) {
                    if self.get_next_msg_no(ctx, 1) != NO_MSG {
                        ctx.text.set_status(STATUS_MSG_CONTINUES);
                        let eye = self.eye_pos;
                        ctx.text.message_set(self.curr_mesg, eye);
                    } else {
                        ctx.text.set_status(STATUS_MSG_ENDS);
                    }
                } else if ctx.text.status() == Some(STATUS_BOX_CLOSED) {
                    self.set_anm(ANM_C_WAIT, MORF_P1C_CLOSE);
                    let home = self.home;
                    add_calc_pos2(&mut ctx.pos, home, P1C_HOME_SCALE, P1C_HOME_MAX_STEP);
                    if abs_xz(home - ctx.pos) < P1C_HOME_EPS {
                        ctx.stories.set(EV_0820);
                        self.cyl_radius = CYL_RADIUS_P1C;
                        ctx.text.set_status(STATUS_MSG_DESTROYED);
                        self.set_act(Act::Normal, ctx);
                        ctx.reset_event();
                    }
                }
            }
        }
        true
    }

    fn speak_action(&mut self, ctx: &mut Ctx) -> bool {
        if self.act_status == ACTION_STARTING {
            if self.prev_mesg == MSG_FAE {
                self.skip_smoother = true;
            }
            if !ctx.command_talk() {
                let near = ctx
                    .player
                    .is_some_and(|link| abs_xz(link - ctx.pos) < RANGE_SPEAK);
                if near {
                    self.speak_ordered = true;
                    self.talk_condition = true;
                }
                return false;
            }
            if self.prev_mesg == MSG_FAE {
                self.prev_mesg = self.curr_mesg;
                self.curr_mesg = MSG_FA5;
            } else if self.start_point == 2 {
                self.prev_mesg = self.curr_mesg;
                self.curr_mesg = MSG_FAA;
            } else {
                self.prev_mesg = self.curr_mesg;
                self.curr_mesg = MSG_FA1;
            }
            self.act_status += 1;
            self.msg_id = FPC_M_ERROR_PROCESS_ID;
        } else if self.act_status != ACTION_ENDING {
            if self.msg_id == FPC_M_ERROR_PROCESS_ID {
                let eye = self.eye_pos;
                self.msg_id = ctx.text.message_set(self.curr_mesg, eye);
            } else {
                self.jnt.set_trn();
                self.set_anim_from_msg(ctx);
                if self.act_status == ACTION_ONGOING {
                    if self.curr_mesg == MSG_FA5 {
                        self.skip_smoother = true;
                    }
                    self.msg_bound = ctx.text.bound();
                    if self.msg_bound {
                        self.act_status += 1;
                    }
                } else if ctx.text.status() == Some(STATUS_MSG_DISPLAYED) {
                    if self.get_next_msg_no(ctx, 1) != NO_MSG {
                        ctx.text.set_status(STATUS_MSG_CONTINUES);
                        let eye = self.eye_pos;
                        ctx.text.message_set(self.curr_mesg, eye);
                    } else {
                        ctx.text.set_status(STATUS_MSG_ENDS);
                    }
                } else if ctx.text.status() == Some(STATUS_BOX_CLOSED) {
                    ctx.text.set_status(STATUS_MSG_DESTROYED);
                    self.set_act(Act::Normal, ctx);
                    if self.prev_mesg == MSG_FA4 {
                        ctx.reset_event();
                        ctx.order_sea_exp_cam();
                        self.set_act(Act::Explain, ctx);
                    } else {
                        ctx.set_next_stage();
                    }
                }
            }
        }
        true
    }

    fn explain_action(&mut self, ctx: &mut Ctx) -> bool {
        if self.act_status == ACTION_STARTING {
            self.act_status += 1;
        } else if self.act_status == ACTION_ENDING && !ctx.command_talk() {
            self.speak_ordered = true;
            self.talk_condition = true;
            return false;
        }
        true
    }

    fn get_next_msg_no(&mut self, ctx: &mut Ctx, advance: i32) -> u32 {
        let next = match self.curr_mesg {
            MSG_C94 => MSG_C99,
            MSG_FA1 | MSG_FA2 | MSG_FA5 | MSG_FA6 | MSG_FA7 | MSG_FAA | MSG_FAB
            | MSG_1015 | MSG_1007 => self.curr_mesg.wrapping_add(1),
            MSG_1009 => MSG_102E,
            MSG_1016 => MSG_1033,
            MSG_FAE => MSG_FA5,
            MSG_FA8 | MSG_FAC => MSG_FA2,
            MSG_FA3 => match ctx.text.select_num() {
                0 => MSG_FA4,
                1 => MSG_FB0,
                _ => NO_MSG,
            },
            _ => NO_MSG,
        };
        if advance == 1 {
            self.prev_mesg = self.curr_mesg;
            self.curr_mesg = next;
        }
        next
    }

    fn play_tex_pattern_anm(&mut self) -> bool {
        if self.blink_timer != 0 {
            self.blink_timer -= 1;
        }
        if self.blink_timer == 0 {
            self.blink_frame = self.blink_frame.wrapping_add(1);
            let max = self.btp_frame_max();
            if i32::from(self.blink_frame) >= i32::from(max) {
                self.blink_frame = self.blink_frame.wrapping_sub(max as u8);
                let rng = (rnd(&mut self.seed) * BLINK_RANDOM) as i16;
                self.blink_timer = rng.wrapping_add(BLINK_TIMER_MIN);
            }
        }
        true
    }

    fn demo_end_init(&mut self, ctx: &mut Ctx) {
        if self.param == 0 {
            self.anm_num = NO_ANM;
            let local = Act::Confuse;
            if self.act != Act::None {
                self.act_status = ACTION_ENDING;
                self.run_act(ctx);
            }
            self.prev_act = self.act;
            self.act = local;
            self.act_status = ACTION_STARTING;
            self.run_act(ctx);
            if let Some(pirate) = ctx.pirate {
                let sin = sin_s(ctx.pirate_angle);
                let cos = cos_s(ctx.pirate_angle);
                let mut pos = ctx.pirate_pos;
                pos.y += DEMO_POS_LIFT;
                pos.x += sin * DEMO_POS_SPREAD;
                pos.z += cos * DEMO_POS_SPREAD;
                pos.x += cos * DEMO_POS_SIDE;
                pos.z -= sin * DEMO_POS_SIDE;
                let angle = ctx.pirate_angle.wrapping_sub(DEMO_ANGLE);
                ctx.pirate_moved = Some((pirate, pos, angle));
                ctx.pos = pos;
                ctx.angle = angle;
            }
        }
        self.jnt.off_head_lock();
        self.jnt.off_back_bone_lock();
        self.counter_670 = 0;
    }

    fn demo_move(&mut self, ctx: &mut Ctx) -> bool {
        let Some(demo) = ctx.demo else {
            if self.counter_670 == 1 {
                self.demo_end_init(ctx);
            }
            return false;
        };
        self.jnt.on_head_lock();
        self.jnt.on_back_bone_lock();
        self.counter_670 = 1;
        self.morph.set_anm(FRAME_WAIT, EMODE_LOOP, 1.0);
        self.blink_frame = 0;
        if demo.btp_frame_max != 0 {
            self.blink_frame = self.blink_frame.wrapping_add(1);
            if self.blink_frame >= demo.btp_frame_max {
                self.blink_frame = demo.btp_frame_max;
            }
        }
        true
    }

    fn event_move(&mut self, ctx: &mut Ctx) -> bool {
        if self.cut_proc {
            self.attn_flag = self.cut_attn;
            if !self.cut_attn {
                self.cut_attn = self.attn_flag;
            }
            true
        } else {
            self.private_cut(ctx)
        }
    }

    fn evn_set_anm_init(&mut self, ctx: &Ctx, cut: usize) -> bool {
        match ctx.list.cut_number(cut, "idx") {
            Some(value) => {
                self.set_anm(value, -1.0);
            }
            None => {
                if self.kind == TYPE_P1C {
                    self.set_anm(ANM_C_WAIT, -1.0);
                } else {
                    self.set_anm(ANM_WAIT, -1.0);
                }
            }
        }
        true
    }

    fn evn_talk_init(&mut self, ctx: &Ctx, cut: usize) -> bool {
        let msg = ctx.list.cut_number(cut, "MsgNo");
        self.msg_id = FPC_M_ERROR_PROCESS_ID;
        self.msg_bound = false;
        self.curr_mesg = u32::try_from(msg.unwrap_or(0)).unwrap_or(0);
        true
    }

    fn evn_talk(&mut self, ctx: &mut Ctx) -> bool {
        if self.msg_id == FPC_M_ERROR_PROCESS_ID {
            let eye = self.eye_pos;
            self.msg_id = ctx.text.message_set(self.curr_mesg, eye);
        } else if !self.msg_bound {
            self.msg_bound = ctx.text.bound();
        } else if ctx.text.status() == Some(STATUS_MSG_DISPLAYED) {
            if self.get_next_msg_no(ctx, 1) != NO_MSG {
                ctx.text.set_status(STATUS_MSG_CONTINUES);
                let eye = self.eye_pos;
                ctx.text.message_set(self.curr_mesg, eye);
            } else {
                ctx.text.set_status(STATUS_MSG_ENDS);
            }
        } else if ctx.text.status() == Some(STATUS_BOX_CLOSED) {
            ctx.text.set_status(STATUS_MSG_DESTROYED);
            self.msg_bound = false;
            self.msg_id = FPC_M_ERROR_PROCESS_ID;
            return true;
        }
        false
    }

    fn minigame_explain_cut(&mut self, ctx: &mut Ctx) -> bool {
        let Some((cut, _)) = ctx.staff_cut else {
            self.attn_flag = false;
            return false;
        };
        let act_idx = act_index_of(ctx.list, cut, &MINIGAME_ACTION_NAMES);
        if self.cut_state == 0 {
            self.msg_id = FPC_M_ERROR_PROCESS_ID;
            self.cam_attention = true;
            ctx.cam_attention = true;
            self.prev_mesg = self.curr_mesg;
            self.curr_mesg = MSG_FAD;
            self.cut_state += 1;
        } else if self.cut_state != -1 {
            if self.msg_id == FPC_M_ERROR_PROCESS_ID {
                let eye = self.eye_pos;
                self.msg_id = ctx.text.message_set(self.curr_mesg, eye);
            } else {
                if self.cut_state == 1 {
                    self.msg_bound = ctx.text.bound();
                    if self.msg_bound {
                        self.cut_state += 1;
                    }
                } else if ctx.text.status() == Some(STATUS_MSG_DISPLAYED) {
                    if self.curr_mesg == MSG_FAD {
                        ctx.text.set_status(STATUS_MSG_CONTINUES);
                        self.prev_mesg = self.curr_mesg;
                        self.curr_mesg = MSG_FAE;
                        let eye = self.eye_pos;
                        ctx.text.message_set(self.curr_mesg, eye);
                    } else {
                        self.prev_mesg = self.curr_mesg;
                        self.curr_mesg = NO_MSG;
                        ctx.text.set_status(STATUS_MSG_ENDS);
                    }
                } else if ctx.text.status() == Some(STATUS_BOX_CLOSED) {
                    if act_idx == MINIGAME_ACT_SECOND {
                        self.end_staff = true;
                    }
                    ctx.text.set_status(STATUS_MSG_DESTROYED);
                    self.cam_attention = false;
                    ctx.cam_attention = false;
                } else if ctx.send_button && act_idx == PRIVATE_ACT_MINIGAME_A {
                    self.end_staff = true;
                }
                if self.check_act(Act::Explain) && ctx.sea_exp_end {
                    self.set_act(Act::Speak, ctx);
                    ctx.reset_event();
                    self.skip_smoother = true;
                    self.speak_ordered = true;
                    self.talk_condition = true;
                }
            }
        }
        true
    }

    fn private_cut(&mut self, ctx: &mut Ctx) -> bool {
        let Some((cut, advance)) = ctx.staff_cut else {
            return false;
        };
        let act_idx = act_index_of(ctx.list, cut, &PRIVATE_CUT_NAMES);
        if act_idx == NO_INDEX {
            self.end_staff = true;
        } else {
            let mut ret = 0;
            if advance {
                match act_idx {
                    PRIVATE_ACT_SETANM => {
                        self.evn_set_anm_init(ctx, cut);
                    }
                    PRIVATE_ACT_HEADSWING => {
                        self.head.swing_vertical_init(
                            HEAD_SWING_COUNT,
                            HEAD_SWING_RATE,
                            HEAD_SWING_AMPLITUDE,
                            HEAD_SWING_AGAIN,
                        );
                    }
                    PRIVATE_ACT_TALKMSG => {
                        self.evn_talk_init(ctx, cut);
                    }
                    _ => {}
                }
            }
            match act_idx {
                PRIVATE_ACT_MINIGAME_A | PRIVATE_ACT_MINIGAME_B => {
                    self.minigame_explain_cut(ctx);
                }
                PRIVATE_ACT_TALKMSG => {
                    ret = i32::from(self.evn_talk(ctx));
                    self.set_anim_from_msg(ctx);
                }
                _ => {
                    ret = 1;
                }
            }
            if ret != 0 {
                self.end_staff = true;
            }
        }
        true
    }

    pub fn set_attention_pos(&mut self) {
        self.attn_pos = self.eye_pos;
        self.attn_pos.y += hio(self.kind).attn_y_pos_offset;
    }

    pub fn create_init(&mut self, ctx: &mut Ctx) -> bool {
        let bit = (ctx.parameters >> PARAM_TYPE_SHIFT) & PARAM_TYPE_MASK;
        self.param = (ctx.parameters & PARAM_BYTE_MASK) as u8;
        if bit == 0 {
            self.kind = TYPE_P1A;
            if self.param == 2 {
                self.counter_671 = 1;
                self.jnt.on_back_bone_lock();
                self.kaji_id = self.get_kaji_id(ctx);
            } else {
                self.counter_671 = 0;
            }
        } else if bit == 1 {
            self.kind = TYPE_P1B;
            self.counter_671 = 0;
        } else {
            self.kind = TYPE_P1C;
            self.counter_671 = 0;
        }
        self.staff_name = staff_name_of(self.kind);
        self.staff_slot = ctx.list.staff_of(self.staff_name).unwrap_or(NO_STAFF);
        self.anm_num = NO_ANM;
        self.counter_66c = 0;
        self.counter_670 = 0;
        self.kaji_timer = KAJI_TIMER;
        self.attn_flags = ATTN_FLAGS;
        self.home = ctx.pos;
        self.home_angle = ctx.angle;
        self.start_point = ctx.start_point;
        if self.kind == TYPE_P1B {
            self.attn_dist = ATTN_TYPE_TALK;
            if self.param == 1 && (ctx.start_point == 0 || ctx.start_point == 2) {
                self.set_act(Act::Speak, ctx);
            } else if self.param == 0 {
                self.set_act(Act::Confuse, ctx);
            } else {
                if self.param == 2 {
                    self.jnt.on_back_bone_lock();
                    self.jnt.on_head_lock();
                    self.cyl_radius = CYL_RADIUS_P1B_PARAM2;
                } else if self.param == 3 {
                    self.jnt.on_back_bone_lock();
                    self.jnt.on_head_lock();
                }
                self.set_act(Act::Normal, ctx);
            }
        } else if self.kind == TYPE_P1C {
            self.attn_dist = ATTN_TYPE_TALK;
            if !ctx.stories.test(EV_0820) && !ctx.stories.test(EV_0808) {
                let sin = sin_s(ctx.angle);
                let cos = cos_s(ctx.angle);
                ctx.pos.x -= cos * CYL_RADIUS;
                ctx.pos.z += sin * CYL_RADIUS;
                self.set_anm(ANM_C_WAIT, -1.0);
                self.set_act(Act::P1cSpeak, ctx);
            } else {
                self.set_act(Act::Normal, ctx);
            }
            self.cyl_radius = CYL_RADIUS_P1C;
        } else {
            self.attn_dist = ATTN_TYPE_TALK_P1A;
            if self.counter_671 != 0 {
                self.set_anm(ANM_WAIT02, 0.0);
                self.cyl_radius = REG10_F5 + CYL_RADIUS_P1A_REG;
            }
            self.set_act(Act::Normal, ctx);
        }
        self.refresh_visible(ctx.stories);
        self.kind == TYPE_P1B && self.param == 1 && ctx.start_point == 1
    }

    fn get_kaji_id(&self, ctx: &Ctx) -> u32 {
        ctx.pirate_kaji.unwrap_or(NO_KAJI)
    }

    fn kaji_anm(&mut self, ctx: &mut Ctx) -> bool {
        if self.counter_671 != 0 {
            if self.anm_num == ANM_OMOKJ || self.anm_num == ANM_TORIKJ {
                if self.morph.check_frame(self.morph.get_end_frame() - 1.0) {
                    self.set_anm(ANM_WAIT02, -1.0);
                    self.kaji_timer = KAJI_TIMER;
                } else if self.morph.check_frame(KAJI_WHEEL_FRAME) {
                    ctx.events.list.push(SimEvent::Se(SE_PIRATE_WHEEL));
                }
            } else if self.kaji_timer < 0 {
                self.kaji_timer = self.kaji_timer.wrapping_sub(1);
                if self.morph.check_frame(self.morph.get_end_frame() - 1.0)
                    && !self.check_act(Act::Talk)
                {
                    if rnd(&mut self.seed) > 0.5 {
                        self.set_anm(ANM_OMOKJ, -1.0);
                    } else {
                        self.set_anm(ANM_TORIKJ, -1.0);
                    }
                }
            }
            if self.kaji_id != NO_KAJI {
                self.set_kaji =
                    Some((self.anm_num - KAJI_ANM_BIAS, self.morph.get_frame()));
            }
            return true;
        }
        if self.kind == TYPE_P1B && self.param == 2 && !self.check_act(Act::Talk) {
            if self.anm_num == ANM_CHECK01
                && self.morph.check_frame(self.morph.get_end_frame() - 1.0)
            {
                self.counter_66c -= 1;
                if self.counter_66c <= 0 {
                    self.set_anm(ANM_CHECK02, -1.0);
                    self.counter_66c = if rnd(&mut self.seed) > 0.5 { 1 } else { 2 };
                }
            } else if self.anm_num == ANM_CHECK02
                && self.morph.check_frame(self.morph.get_end_frame() - 1.0)
            {
                self.counter_66c -= 1;
                if self.counter_66c <= 0 {
                    self.set_anm(ANM_CHECK01, -1.0);
                    self.counter_66c = 4;
                }
            }
        }
        false
    }

    pub fn execute(&mut self, ctx: &mut Ctx) {
        self.refresh_visible(ctx.stories);
        self.head.head_move();
        if !self.demo_move(ctx) {
            self.play_tex_pattern_anm();
            self.morph.play();
            if !ctx.event_mode || ctx.command_talk() {
                self.run_act(ctx);
                self.cut_state = 0;
                self.attn_flag = false;
            } else {
                self.event_move(ctx);
            }
        }
        self.kaji_anm(ctx);
        self.look_back(ctx);
        self.set_attention_pos();
    }

    fn look_back(&mut self, ctx: &mut Ctx) -> bool {
        let dist = ctx.player.map_or(0.0, |link| abs_xz(link - ctx.pos));
        let dst: Option<Vec3> = if self.counter_671 == 1 {
            if self.anm_num == ANM_WAIT02 && self.check_act(Act::Talk) {
                Some(ctx.player_eye)
            } else {
                None
            }
        } else if self.attn_flag {
            Some(self.cut_attn_pos)
        } else if dist < hio(self.kind).max_talk_dist
            || self.check_act(Act::Talk)
            || self.check_act(Act::Explain)
            || self.check_act(Act::Speak)
            || self.check_act(Act::P1cSpeak)
        {
            Some(ctx.player_eye)
        } else {
            None
        };
        let child = hio(self.kind);
        self.jnt.set_param(child);
        let mut hold_body = true;
        if self.jnt.trn_chk() {
            add_calc_angle_s2(
                &mut self.max_look_vel,
                child.look_back_target_y,
                LOOK_SCALE,
                LOOK_MAX_STEP,
            );
            hold_body = false;
        } else {
            self.max_look_vel = 0;
        }
        let mut src = ctx.pos;
        src.y += LOOK_SRC_LIFT;
        let mut yaw = ctx.angle;
        self.jnt.look_at_target(
            &mut yaw,
            dst,
            src,
            ctx.angle,
            self.max_look_vel,
            hold_body,
        );
        ctx.angle = yaw;
        !hold_body
    }
}

pub fn staff_name_of(kind: u8) -> &'static str {
    NPC_P1_ACTORS
        .get(usize::from(kind))
        .copied()
        .unwrap_or(NPC_P1_ACTORS[0])
}

fn act_index_of(list: &EventList, cut: usize, names: &[&str]) -> i32 {
    let Some(name) = list.cut_name(cut) else {
        return NO_INDEX;
    };
    names
        .iter()
        .position(|entry| *entry == name)
        .and_then(|index| i32::try_from(index).ok())
        .unwrap_or(NO_INDEX)
}

#[derive(SystemParam)]
pub struct World<'w> {
    pub stories: ResMut<'w, StoryBits>,
    pub ev_flags: ResMut<'w, EventFlags>,
    pub events: ResMut<'w, SimEvents>,
    pub active: ResMut<'w, ActiveEvent>,
    pub next: ResMut<'w, NextStage>,
}

#[derive(SystemParam)]
pub struct Msg<'w, 's> {
    pub procs: ResMut<'w, MsgProcs>,
    pub info: ResMut<'w, MsgGameInfo>,
    pub flags: ResMut<'w, MsgFlags>,
    pub one: ResMut<'w, Msg1Box>,
    pub two: ResMut<'w, Msg2Box>,
    pub text_flags: Res<'w, TextFlags>,
    pub archive: Option<Res<'w, MsgArchives>>,
    pub players: Query<'w, 's, &'static Player>,
}

pub fn control(
    do_intent: Res<DoIntent>,
    start_point: Res<StageStart>,
    list: Res<EventList>,
    mut world: World<'_>,
    mut msg: Msg,
    mut actors: Query<(Entity, &mut Actor, &mut NpcP1), (With<NpcP1>, Without<Player>)>,
    mut others: Query<(Entity, &mut Actor), (Without<NpcP1>, Without<Player>)>,
    mut commands: Commands,
) {
    let player = msg.players.iter().next().copied();
    let player_eye = player.map_or(Vec3::ZERO, |link| Vec3::new(
        link.pos.x,
        link.pos.y + PLAYER_EYE_OFFSET_Y,
        link.pos.z,
    ));
    let clear_count = msg.text_flags.clear_count;
    let event_mode = msg.info.event_run_check();
    let archive = msg.archive.as_deref().map(|held| held as &dyn MsgArchive);
    let sea_exp_cam = list.event_index(SEA_EXP_CAM);
    let kaji = start_point.kaji;
    let pirate = others
        .iter_mut()
        .find(|(_, other)| super::pirateship::is_pirateship(&other.name))
        .map(|(found, other)| (found, other.pos, other.angle[1]));
    let mut moved = Vec::new();
    let mut cuts = Vec::new();
    for (entity, mut actor, mut npc) in &mut actors {
        let actor = &mut *actor;
        let npc = &mut *npc;
        let pressed = do_intent.0 && !npc.do_held;
        npc.do_held = do_intent.0;
        let staff_cut = ActiveEvent::staff_cut(&world.active, &list, npc.staff_name);
        let sea_exp_end = sea_exp_cam.is_some_and(|index| world.active.end_check(index));
        let mut text = Text {
            procs: &mut msg.procs,
            info: &mut msg.info,
            flags: &mut msg.flags,
            one: &mut msg.one,
            two: &mut msg.two,
            clear_count,
            archive,
        };
        let mut ctx = Ctx {
            entity,
            parameters: actor.parameters,
            player: player.map(|link| link.pos),
            player_eye,
            pos: actor.pos,
            angle: actor.angle[1],
            room: actor.room,
            start_point: start_point.point,
            event_mode,
            talk_command: npc.talk_command || pressed && event_mode,
            send_button: pressed,
            staff_cut,
            sea_exp_cam,
            sea_exp_end,
            cut_attn: npc.cut_attn,
            cam_attention: npc.cam_attention,
            pirate: pirate.map(|(found, _, _)| found),
            pirate_pos: pirate.map_or(Vec3::ZERO, |(_, pos, _)| pos),
            pirate_angle: pirate.map_or(0, |(_, _, angle)| angle),
            pirate_kaji: kaji,
            pirate_moved: None,
            demo: None,
            stories: &mut world.stories,
            events: &mut world.events,
            list: &list,
            active: &mut world.active,
            next: &mut world.next,
            text: &mut text,
        };
        if !npc.entered {
            npc.entered = true;
            let created = npc.create_init(&mut ctx);
            if !created {
                actor.pos = ctx.pos;
                actor.angle[1] = ctx.angle;
                commands.entity(entity).despawn();
                continue;
            }
        } else {
            npc.execute(&mut ctx);
        }
        actor.pos = ctx.pos;
        actor.angle[1] = ctx.angle;
        npc.cut_attn = ctx.cut_attn;
        npc.cam_attention = ctx.cam_attention;
        let end_staff = npc.end_staff;
        let staff = npc.staff_name;
        if let Some(place) = ctx.pirate_moved {
            moved.push(place);
        }
        if end_staff {
            cuts.push(staff);
        }
        npc.end_staff = false;
        npc.talk_command = npc.speak_ordered;
        npc.speak_ordered = false;
        if let Some((anm, frame)) = npc.set_kaji.take()
            && npc.kaji_id != NO_KAJI
        {
            npc.kaji_anm = anm;
            npc.kaji_frame = frame;
        }
    }
    for (entity, pos, angle) in moved {
        if let Ok((_, mut actor)) = others.get_mut(entity) {
            actor.pos = pos;
            actor.angle[1] = angle;
        }
    }
    for staff in cuts {
        world
            .active
            .end_staff_cut(&list, &mut world.ev_flags, staff);
    }
}
