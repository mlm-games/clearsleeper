use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::event::{ActiveEvent, EventList, PresentItem};
use super::events::{SimEvent, SimEvents};
use super::flags::{Collect, EventFlags, StoryBits};
use super::ground;
use super::inventory::{Equipped, check_get_item as item_obtained};
use super::player::Player;
use super::stream;
use super::walk;
use crate::assets::MsgArchives;
use crate::text::driver::TextFlags;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgFlags, MsgGameInfo, MsgProcs,
    fop_msgm_message_set,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED,
    STATUS_MSG_ENDS,
};

pub const IVAN_ACTORS: [&str; 2] = ["Mk", "Mk2"];

pub fn is_ivan(name: &str) -> bool {
    IVAN_ACTORS.contains(&name)
}

pub const STAFF_NAMES: [&str; 2] = ["Mk", "Mk2"];
pub const UM2_ACTOR: &str = "Um2";

pub const CULL_MIN: [f32; 3] = [-35.0, -10.0, -35.0];
pub const CULL_MAX: [f32; 3] = [35.0, 100.0, 35.0];

pub const PRM_TYPE_SHIFT: u32 = 0;
pub const PRM_TYPE_MASK: u32 = 0xFF;
pub const PRM_PATH_SHIFT: u32 = 8;
pub const PRM_PATH_MASK: u32 = 0xFF;
pub const PRM_UM2_HIDE: u32 = 0x8000_0000;

pub const CYL_RADIUS_SRC: f32 = 40.0;
pub const CYL_HEIGHT_SRC: f32 = 80.0;
pub const CYL_RADIUS_RUN: f32 = 40.0;
pub const CYL_HEIGHT_RUN: f32 = 80.0;
pub const LINK_HIT_PAD: f32 = 20.0;

pub const ATTN_SPEAK_TALK: u32 = 0x0000_0003;
pub const ATTN_DIST_TALK: i32 = 0xA9;
pub const ATTN_DIST_SPEAK: i32 = 0xA9;
pub const ATTN_Y_LIFT: f32 = 45.0;

pub const STTS_SHOWMAP: u32 = 0x0000_0001;
pub const STTS_UNK40: u32 = 0x0000_0020;
pub const STTS_UNK4000: u32 = 0x0001_0000;
pub const STTS_INIT_A0: u16 = 0x00A0;
pub const STTS_WEIGHT_FE: u16 = 0x00FE;
pub const MAP_SET_ID: i32 = 0x27;

pub const GRAVITY: f32 = -30.0;
pub const GRAVITY_RUN: f32 = -3.3;
pub const GRAVITY_JUMP2: f32 = -2.1;
pub const MAX_FALL: f32 = -90.0;

pub const LOOKBACK_PLAYER_EYE: f32 = -20.0;
pub const HEAD_TURN_MAX: i16 = 1500;
pub const HEAD_TURN_SCALE: i16 = 4;
pub const HEAD_TURN_STEP: i16 = 0x800;

pub const JNT_MAX_BACKBONE_X: i16 = 0;
pub const JNT_MAX_BACKBONE_Y: i16 = 7000;
pub const JNT_MIN_BACKBONE_X: i16 = 0;
pub const JNT_MIN_BACKBONE_Y: i16 = -7000;
pub const JNT_MAX_HEAD_X: i16 = 8000;
pub const JNT_MAX_HEAD_Y: i16 = 9000;
pub const JNT_MIN_HEAD_X: i16 = -2000;
pub const JNT_MIN_HEAD_Y: i16 = -9000;
pub const JNT_MAX_TURN_STEP: i16 = 1000;

pub const BLINK_BASE: f32 = 30.0;
pub const BLINK_RANGE: f32 = 100.0;
pub const BLINK_FRAME_MAX: i16 = 30;

pub const MORF_RATE: f32 = 8.0;
pub const ANM_MAX: [f32; 12] = [
    60.0, 40.0, 40.0, 40.0, 40.0, 40.0, 40.0, 30.0, 30.0, 30.0, 40.0, 20.0,
];
pub const JUMP_ANM_CHANCE: f32 = 0.4;

pub const RUN_SPEED: f32 = 8.0;
pub const RUN_SPEED_WALK: f32 = 12.0;
pub const RUN_SPEED_LAND: f32 = 6.0;
pub const JUMP_SPEED_F: f32 = 8.0;
pub const JUMP_Y: f32 = 20.0;
pub const JUMP2_Y: f32 = 30.0;
pub const JUMP2_SPEED_F: f32 = 10.0;
pub const RUN_JUMP_Y: f32 = 25.0;
pub const VISIT_SPEED_WALK: f32 = 2.5;
pub const VISIT_SPEED_LINK: f32 = 12.0;
pub const VISIT_SPEED_AROUND: f32 = 9.0;
pub const CHASE_FAST: f32 = 2.8;
pub const CHASE_SLOW: f32 = 1.1;
pub const RUN_CHASE: f32 = 2.8;
pub const DEMO_CHASE: f32 = 1.1;
pub const DEMO_TARGET_SPEED: f32 = 12.0;
pub const RUNAWAY_MIN_SPEED: f32 = 15.0;
pub const RUNAWAY_MAX_SPEED: f32 = 18.0;

pub const TURN_SCALE: i16 = 8;
pub const TURN_STEP: i16 = 0x1000;
pub const TURN_MIN: i16 = 0x400;
pub const TURN_LINK_SCALE: i16 = 8;
pub const TURN_LINK_STEP: i16 = 0x800;
pub const AROUND_TURN_SCALE: i16 = 2;
pub const AROUND_TURN_STEP: i16 = 0x800;
pub const AROUND_TURN_MIN: i16 = 0x400;
pub const AROUND_TURN_MIN2: i16 = 0x200;
pub const AROUND_ADD1: i16 = 0x3000;
pub const AROUND_ADD2: i16 = 0x3400;
pub const AROUND_ADD3: i16 = 0x4000;
pub const AROUND_ADD4: i16 = 0x3a00;
pub const PATH_TURN_SCALE: i16 = 8;
pub const PATH_TURN_STEP: i16 = 0x800;
pub const PATH_TURN_MIN: i16 = 0x200;
pub const PATH_TURN_FAST_SCALE: i16 = 2;
pub const PATH_TURN_FAST_STEP: i16 = 0x2000;
pub const PATH_TURN_FAST_MIN: i16 = 0x400;

pub const F_INF: f32 = 1.0e9;
pub const NEAR_PATH_RANGE: f32 = 100.0;
pub const REACHED_LINK_TIMER: u8 = 55;
pub const WAIT_TIMER_NOTICE: u8 = 0xF;
pub const WAIT_TIMER_LEFT: u8 = 0x19;
pub const FAST_WALK_LIMIT: u16 = 0x64;
pub const IGNORE_LINK_LIMIT: u16 = 0x258;
pub const LOOK_ANGLE: i16 = 0x1800;

pub const SQUARE_400: f32 = 400.0 * 400.0;
pub const SQUARE_150: f32 = 150.0 * 150.0;
pub const SQUARE_180: f32 = 180.0 * 180.0;
pub const CLIMB_DIST: f32 = 200.0;

pub const MK_INIT_P0: u8 = 0x50;
pub const MK_INIT_P1: u16 = 0x12C;
pub const RUN_JUMP_COS: f32 = 0.0;
pub const FAR_LINK_ANGLE: i32 = 0x1800;
pub const FAR_LINK_LEN: f32 = 10000.0;
pub const NEAR_PATH_50: f32 = 50.0;
pub const NEAR_PATH_20: f32 = 20.0;
pub const NEAR_PATH_800: f32 = 800.0;
pub const NEAR_PATH_900: f32 = 900.0;
pub const NEAR_PATH_400: f32 = 400.0;
pub const NEAR_PATH_200: f32 = 200.0;
pub const NEAR_PATH_10: u8 = 10;
pub const MK2_RANGE: u8 = 5;

pub const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
pub const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
pub const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
pub const RND_SEED: i32 = 100;
pub const RND_RANGE: f32 = 1.0;
pub const RND_CHANCE: f32 = 0.5;

pub const SHOCK_FRAME: i32 = 4;
pub const SHOCK_MAGNITUDE: i32 = -0x21;
pub const SHOCK_DIR: [f32; 3] = [0.0, 1.0, 0.0];

pub const PATH_POINTS: usize = 16;
pub const PATH_INF_NONE: u8 = 0xFF;
pub const NO_EVENT_IDX: i32 = -1;
pub const NO_STAFF: i32 = -1;
pub const NO_ACTION: i32 = -1;

const TYPE_NORMAL: u8 = 1;
const TYPE_MINIGAME: u8 = 2;

const STATE_WAIT: i8 = 0;
const STATE_TALK01: i8 = 1;
const STATE_TALK02: i8 = 2;
const STATE_VISIT: i8 = 3;
const STATE_CLIMB01: i8 = 4;
const STATE_DROP01: i8 = 5;
const STATE_RUNAWAY: i8 = 6;
const STATE_JUMP: i8 = 7;
const STATE_JITANDA01: i8 = 8;
const STATE_JITANDA02: i8 = 9;
const STATE_DEMO01: i8 = 10;
const STATE_DEMO02: i8 = 11;
const STATE_DEMO03: i8 = 12;
const STATE_13: i8 = 13;

const JNT_PARAM: [i16; 9] = [
    JNT_MAX_BACKBONE_X,
    JNT_MAX_BACKBONE_Y,
    JNT_MIN_BACKBONE_X,
    JNT_MIN_BACKBONE_Y,
    JNT_MAX_HEAD_X,
    JNT_MAX_HEAD_Y,
    JNT_MIN_HEAD_X,
    JNT_MIN_HEAD_Y,
    JNT_MAX_TURN_STEP,
];

const ACTION_STARTING: i8 = 0;
const ACTION_ONGOING: i8 = 1;
const ACTION_ENDING: i8 = -1;

const TALK_FINISHED: i8 = -1;
const TALK_INIT: i8 = 0;
const TALK_MSG_CREATE: i8 = 1;
const TALK_ACTIVE: i8 = 2;

const VISIT_START: u8 = 0;
const VISIT_TALK: u8 = 1;
const VISIT_WALK_PATH: u8 = 2;
const VISIT_RUN_LINK: u8 = 3;
const VISIT_WALK_AROUND_LINK: u8 = 4;
const VISIT_REACHED_LINK: u8 = 5;
const VISIT_NOTICE_LINK: u8 = 6;
const VISIT_LEFT_PATH: u8 = 7;
const VISIT_WALK_PATH_FAST: u8 = 8;
const VISIT_WALK_PATH_IGNORE_LINK: u8 = 9;

const ORDER_TALK: u8 = 0x1;
const ORDER_SPEAK: u8 = 0x2;
const ORDER_LOCK: u8 = 0x3;
const ORDER_CANTALK: u8 = 0x3;

const FLAG_TALK: u16 = 0x1;
const FLAG_JUMP: u16 = 0x2;
const FLAG_ATTENTION: u16 = 0x4;
const FLAG_SELECT: u16 = 0x8;
const FLAG_NO_MOVE: u16 = 0x10;
const FLAG_STAND: u16 = 0x20;
const FLAG_AIR: u16 = 0x40;
const FLAG_TURN_PATH: u16 = 0x80;
const FLAG_NO_MSG_ANM: u16 = 0x100;
const FLAG_TURN_STOP: u16 = 0x200;
const FLAG_GAMESET: u16 = 0x400;

const TMP_0002: u32 = 0x0002;
const TMP_0004: u32 = 0x0004;
const TMP_0008: u32 = 0x0008;
const TMP_0010: u32 = 0x0010;
const TMP_0020: u32 = 0x0020;
const TMP_0040: u32 = 0x0040;
const TMP_0120: u32 = 0x0120;
const TMP_0580: u32 = 0x0580;

const BIT_1208: u16 = 0x1208;
const BIT_1210: u16 = 0x1210;
const BIT_1340: u16 = 0x1340;
const BIT_1380: u16 = 0x1380;
const BIT_1602: u16 = 0x1602;
const BIT_1E02: u16 = 0x1E02;
const BIT_1E04: u16 = 0x1E04;
const BIT_1F80: u16 = 0x1F80;
const BIT_2201: u16 = 0x2201;

const ITEM_HEART_PIECE: u8 = 0x07;
const ITEM_PICTO_BOX: u8 = 0x23;
const ITEM_DELUXE_PICTO_BOX: u8 = 0x26;

const MSG_SELECT_BOX: u32 = 0x26CA;
const MSG_TALK_DEFAULT: u32 = 0x26BD;
const MSG_TALK_PLAY: u32 = 0x26AD;
const MSG_TALK_HUNTED: u32 = 0x26C5;
const MSG_TALK_NOTICE: u32 = 0x26B9;
const MSG_TALK_HOLD: u32 = 0x26B5;
const MSG_TALK_MINIGAME: u32 = 0x26B1;
const MSG_CHAIN: [u32; 13] = [
    0x26BD, 0x26C5, 0x26C6, 0x26C7, 0x26C8, 0x26C9, 0x26CB, 0x26CD, 0x26D0, 0x26D1, 0x26D3,
    0x26D4, 0x26D8,
];

const SE_START_WHISTLE: u32 = 0x08F0;
const SE_END_WHISTLE: u32 = 0x08F1;
const SE_CV_TR_KO_A_RUN_AWAY: u32 = 0x4904;
const SE_CV_TR_KO_A_FOUND: u32 = 0x4905;
const SE_CV_TR_KO_A_TURN: u32 = 0x4906;
const SE_CV_TR_KO_A_CAUGHT: u32 = 0x4907;

const EVENT_MK_TALK: &str = "MK_TALK";
const EVENT_MK_TALK2: &str = "MK_TALK2";
const EVENT_MK_TALK3: &str = "MK_TALK3";
const EVENT_MK_TALK4: &str = "MK_TALK4";
const EVENT_MK_GAMESTART: &str = "MK_GAMESTART";
const EVENT_MK_GAMESET: &str = "MK_GAMESET";
const EVENT_MK_DROP: &str = "MK_DROP";

const STAFF_ACTION_NAMES: [&str; 17] = [
    "WAIT", "TALK", "HOME", "RUN", "RUN3", "SPEAK", "PARTNER", "JUMP", "JUMP2", "JITABATA",
    "LOOK_P", "HIND", "DISP", "MAKEITEM", "GAMESET", "WARP", "TURN",
];

const SOUND_KEY: &str = "SOUND";
const POS_KEY: &str = "Pos";
const TIMER_KEY: &str = "Timer";
const MSGNO_KEY: &str = "MsgNo";
const ANGLE_KEY: &str = "Angle";
const RM_UM2_KEY: &str = "RM_Um2";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    None,
    Wait,
    Seek,
    Hind,
    Visit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaffAction {
    Wait,
    Talk,
    Home,
    Run,
    Run3,
    Speak,
    Partner,
    Jump,
    Jump2,
    Jitabata,
    LookP,
    Hind,
    Disp,
    MakeItem,
    GameSet,
    Warp,
    Turn,
}

impl StaffAction {
    pub fn of_name(name: &str) -> Option<Self> {
        let index = STAFF_ACTION_NAMES.iter().position(|cut| *cut == name)?;
        Some(match index {
            0 => Self::Wait,
            1 => Self::Talk,
            2 => Self::Home,
            3 => Self::Run,
            4 => Self::Run3,
            5 => Self::Speak,
            6 => Self::Partner,
            7 => Self::Jump,
            8 => Self::Jump2,
            9 => Self::Jitabata,
            10 => Self::LookP,
            11 => Self::Hind,
            12 => Self::Disp,
            13 => Self::MakeItem,
            14 => Self::GameSet,
            15 => Self::Warp,
            16 => Self::Turn,
            _ => return None,
        })
    }

    pub fn index(self) -> i32 {
        STAFF_ACTION_NAMES
            .iter()
            .position(|name| *name == self.name())
            .map_or(NO_ACTION, |index| index as i32)
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Wait => "WAIT",
            Self::Talk => "TALK",
            Self::Home => "HOME",
            Self::Run => "RUN",
            Self::Run3 => "RUN3",
            Self::Speak => "SPEAK",
            Self::Partner => "PARTNER",
            Self::Jump => "JUMP",
            Self::Jump2 => "JUMP2",
            Self::Jitabata => "JITABATA",
            Self::LookP => "LOOK_P",
            Self::Hind => "HIND",
            Self::Disp => "DISP",
            Self::MakeItem => "MAKEITEM",
            Self::GameSet => "GAMESET",
            Self::Warp => "WARP",
            Self::Turn => "TURN",
        }
    }
}

fn target_angle_y(from: Vec3, to: Vec3) -> i16 {
    walk::to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn dis_xz_ang_y(from: Vec3, to: Vec3) -> (f32, i16) {
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    ((dx * dx + dz * dz).sqrt(), walk::to_s16(dx.atan2(dz)))
}

fn abs2_xz(v: Vec3) -> f32 {
    v.x * v.x + v.z * v.z
}

fn abs_xz(v: Vec3) -> f32 {
    (v.x * v.x + v.z * v.z).sqrt()
}

fn len_2d_sq_pnt_and_seg_line(
    px: f32,
    pz: f32,
    ax: f32,
    az: f32,
    bx: f32,
    bz: f32,
) -> Option<f32> {
    let vx = bx - ax;
    let vz = bz - az;
    let wx = px - ax;
    let wz = pz - az;
    let len = vx * vx + vz * vz;
    if len == 0.0 {
        return None;
    }
    let t = (wx * vx + wz * vz) / len;
    if t < 0.0 || t > 1.0 {
        return None;
    }
    let cx = ax + vx * t;
    let cz = az + vz * t;
    let dx = px - cx;
    let dz = pz - cz;
    Some(dx * dx + dz * dz)
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

fn event_index(ctx: &Ctx, name: &str) -> i32 {
    ctx.list
        .event_index(name)
        .map_or(NO_EVENT_IDX, |index| index as i32)
}

fn event_slot(index: i32) -> Option<usize> {
    usize::try_from(index).ok()
}

fn event_end(index: i32, ctx: &Ctx) -> bool {
    usize::try_from(index).is_ok_and(|index| ctx.active.end_check(index))
}

fn calc_timer(value: &mut i16) -> bool {
    if *value != 0 {
        *value -= 1;
    }
    *value == 0
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathRun {
    pub num: usize,
    pub idx: usize,
    pub dir: bool,
    pub points: [Vec3; PATH_POINTS],
}

impl PathRun {
    pub fn new() -> Self {
        Self {
            num: 0,
            idx: 0,
            dir: false,
            points: [Vec3::ZERO; PATH_POINTS],
        }
    }

    pub fn is_path(&self) -> bool {
        self.num != 0
    }

    pub fn get_path(&self) -> bool {
        self.num != 0
    }

    pub fn set_inf(&mut self, path_idx: u8, room: i8, forwards: u8) -> bool {
        let _ = room;
        self.num = 0;
        self.idx = 0;
        self.dir = forwards != 0;
        if path_idx != PATH_INF_NONE {
            self.num = 1;
            self.idx = 0;
            return true;
        }
        false
    }

    pub fn set_inf_drct(&mut self, from: &PathRun) -> bool {
        self.num = from.num;
        self.idx = 0;
        true
    }

    pub fn next_path(&self, room: i8) -> &PathRun {
        let _ = room;
        self
    }

    pub fn get_idx(&self) -> usize {
        self.idx
    }

    pub fn set_idx(&mut self, idx: usize) {
        self.idx = if self.num == 0 { 0 } else { idx.min(self.num - 1) };
    }

    pub fn clr_idx(&mut self) {
        self.idx = 0;
    }

    pub fn get_dir(&self) -> bool {
        self.dir
    }

    pub fn turn_dir(&mut self) {
        self.dir = !self.dir;
    }

    pub fn max_point(&self) -> usize {
        self.num
    }

    pub fn get_point(&self, point_idx: usize) -> Vec3 {
        if self.is_path() && point_idx < self.num {
            self.points[point_idx.min(PATH_POINTS - 1)]
        } else {
            Vec3::ZERO
        }
    }

    pub fn set_point(&mut self, point_idx: usize, pos: Vec3) {
        if let Some(point) = self.points.get_mut(point_idx) {
            *point = pos;
        }
    }

    pub fn inc_idx(&mut self) -> bool {
        let mut ret = true;
        if self.is_path() {
            self.idx += 1;
            if self.idx >= self.num {
                self.idx = self.num - 1;
                ret = false;
            }
        }
        ret
    }

    pub fn inc_idx_loop(&mut self) -> bool {
        let mut ret = true;
        if self.is_path() {
            self.idx += 1;
            if self.idx >= self.num {
                self.idx = 0;
                ret = false;
            }
        }
        ret
    }

    pub fn dec_idx_loop(&mut self) -> bool {
        let mut ret = true;
        if self.is_path() {
            self.idx = self.idx.wrapping_sub(1);
            if self.idx >= self.num {
                self.idx = self.num - 1;
                ret = false;
            }
        }
        ret
    }

    pub fn chk_point_pass(&self, curr: Vec3, going_forwards: bool) -> bool {
        if !self.is_path() {
            return false;
        }
        let target = self.get_point(self.idx);
        let delta_x;
        let delta_z;
        if self.idx == 0 {
            delta_x = self.get_point(1).x - self.get_point(0).x;
            delta_z = self.get_point(1).z - self.get_point(0).z;
        } else if self.idx == self.num - 1 {
            delta_x = self.get_point(self.num - 1).x - self.get_point(self.num - 2).x;
            delta_z = self.get_point(self.num - 1).z - self.get_point(self.num - 2).z;
        } else {
            delta_x = self.get_point(self.idx + 1).x - self.get_point(self.idx - 1).x;
            delta_z = self.get_point(self.idx + 1).z - self.get_point(self.idx - 1).z;
        }
        let heading = walk::to_s16(delta_x.atan2(delta_z));
        let f29 = walk::from_s16(heading).sin();
        let f2 = walk::from_s16(heading).cos();
        let f3 = -(f29 * target.x + f2 * target.z);
        let side = f29 * curr.x + f2 * curr.z + f3;
        if going_forwards { side > 0.0 } else { side <= 0.0 }
    }

    pub fn set_near_path_indx(&mut self, pos: Vec3, range: f32) -> bool {
        if !self.is_path() {
            return false;
        }
        let mut max_dist = F_INF;
        let mut point_idx = 0;
        for i in 0..self.max_point() {
            let diff = pos - self.get_point(i);
            let dist = (range * (diff.y * diff.y) + abs2_xz(diff)).sqrt();
            if max_dist > dist {
                max_dist = dist;
                point_idx = i;
            }
        }
        self.idx = point_idx;
        true
    }

    pub fn set_near_path_indx_mk(&mut self, pos: Vec3) -> f32 {
        if !self.is_path() {
            return F_INF;
        }
        let mut max_dist = F_INF;
        let mut point_idx = 0;
        for i in 0..self.max_point() {
            let dist = (pos - self.get_point(i)).length();
            if max_dist > dist {
                max_dist = dist;
                point_idx = i;
            }
        }
        self.idx = point_idx;
        max_dist
    }

    pub fn abs_idx(&self, start: usize, last: usize) -> i32 {
        if !self.is_path() {
            return 0;
        }
        let max = self.max_point() as i32;
        let mut diff_1 = start as i32 - last as i32;
        let mut diff_2 = last as i32 - start as i32;
        if diff_1 < 0 {
            diff_1 += max;
        }
        if diff_2 < 0 {
            diff_2 += max;
        }
        if diff_1 > diff_2 {
            return diff_2;
        }
        diff_1
    }

    pub fn set_near_path_indx_mk2(&mut self, pos: Vec3, from: usize, range: u8) -> bool {
        if !self.is_path() {
            return false;
        }
        let mut max_dist = F_INF;
        let mut point_idx = from;
        let mut set = false;
        for i in 0..self.max_point() {
            let dist = (pos - self.get_point(i)).length();
            let idx = self.abs_idx(from, i);
            if idx <= i32::from(range) && from != i && max_dist > dist {
                max_dist = dist;
                point_idx = i;
                set = true;
            }
        }
        self.idx = point_idx;
        set
    }

    pub fn chk_inside(&mut self, pos: &Vec3) -> bool {
        self.set_near_path_indx(*pos, 0.0);
        let point2 = self.get_point(self.idx);
        self.dec_idx_loop();
        let point = self.get_point(self.idx);
        self.inc_idx_loop();
        self.inc_idx_loop();
        let point3 = self.get_point(self.idx);
        let angle1 = target_angle_y(point2, point);
        let angle2 = target_angle_y(point2, *pos);
        let angle3 = target_angle_y(point2, point3);
        let diff1 = i32::from(angle2.wrapping_sub(angle3));
        let diff2 = i32::from(angle1.wrapping_sub(angle3));
        if diff1 > 0 {
            if diff2 < 0 || diff1 < diff2 {
                return true;
            }
        } else if diff2 < 0 && diff1 < diff2 {
            return true;
        }
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MkStatic {
    pub m0: u8,
    pub m4: f32,
    pub m8: u16,
    pub ma: u16,
    pub point_index1: u8,
    pub md: bool,
    pub me: u8,
    pub point_index2: u8,
}

impl MkStatic {
    pub fn new() -> Self {
        Self {
            m0: 0,
            m4: 1.0,
            m8: 0,
            ma: 0,
            point_index1: 0,
            md: false,
            me: 0,
            point_index2: 0,
        }
    }

    pub fn init(&mut self, p0: u8, p1: u16) {
        self.m0 = 0;
        self.me = p0;
        self.m8 = p1;
        self.ma = p1;
        self.md = false;
        self.m4 = 1.0;
    }

    pub fn turn_path(&mut self, yaw: &mut i16, pos: Vec3, path: &mut PathRun, options: u8) -> u32 {
        let point = path.get_point(path.get_idx());
        let angle = dis_xz_ang_y(pos, point).1;
        if options & 2 != 0 {
            *yaw = walk::add_calc_angle_s(
                *yaw,
                angle,
                PATH_TURN_FAST_SCALE,
                PATH_TURN_FAST_STEP,
                PATH_TURN_FAST_MIN,
            );
        } else {
            *yaw = walk::add_calc_angle_s(*yaw, angle, PATH_TURN_SCALE, PATH_TURN_STEP, PATH_TURN_MIN);
        }
        let cosine = (yaw.wrapping_sub(angle) as f32).cos();
        if cosine < RUN_JUMP_COS {
            self.m4 = cosine + 1.0;
        }
        0
    }

    pub fn chk_path(&mut self, pos: Vec3, path: &mut PathRun, options: u8) -> bool {
        let passed = path.chk_point_pass(pos, options & 1 != 1);
        if passed {
            if options & 1 != 0 {
                path.dec_idx_loop();
            } else {
                path.inc_idx_loop();
            }
            self.point_index2 = self.point_index1;
            self.point_index1 = path.get_idx() as u8;
            true
        } else {
            false
        }
    }

    pub fn walk_path(&mut self, yaw: &mut i16, pos: Vec3, path: &mut PathRun, options: u8) -> bool {
        let result = self.chk_path(pos, path, options);
        self.turn_path(yaw, pos, path, options);
        result
    }

    pub fn get_speed_f(&mut self, p1: f32, p2: f32) -> f32 {
        let speed = if !self.md {
            p1
        } else if self.m8 >= (self.ma >> 1) {
            p2
        } else {
            let f4 = 2.0 / self.ma as f32;
            self.m8 as f32 * (f4 * (p2 - p1)) + p1
        };
        let result = speed * self.m4;
        self.m4 = 1.0;
        result
    }

    pub fn around_walk(&mut self, yaw: &mut i16, self_pos: Vec3, link: Vec3, timer: u8) {
        let mut angle = dis_xz_ang_y(self_pos, link).1;
        if timer != 0 {
            angle = angle.wrapping_add(AROUND_ADD1);
            *yaw = walk::add_calc_angle_s(
                *yaw,
                angle,
                AROUND_TURN_SCALE,
                AROUND_TURN_STEP,
                AROUND_TURN_MIN,
            );
        } else {
            let mag = abs_xz(self_pos - link);
            if mag > 150.0 {
                angle = angle.wrapping_add(AROUND_ADD2);
            } else if mag < 120.0 {
                angle = angle.wrapping_add(AROUND_ADD3);
            } else {
                angle = angle.wrapping_add(AROUND_ADD4);
            }
            *yaw = walk::add_calc_angle_s(
                *yaw,
                angle,
                PATH_TURN_SCALE,
                PATH_TURN_STEP,
                AROUND_TURN_MIN2,
            );
        }
    }

    pub fn go_far_link_3(&mut self, self_pos: Vec3, link: Vec3, path: &PathRun) -> u8 {
        let mut other = PathRun::new();
        other.set_inf_drct(path);
        if self.point_index2 == self.point_index1 {
            return self.m0;
        }
        let local_60 = other.get_point(usize::from(self.point_index2));
        let local_6c = other.get_point(usize::from(self.point_index1));
        let s_var4 = target_angle_y(self_pos, link);
        let s_var5 = target_angle_y(local_60, local_6c);
        if (self.m0 == 1 || self.m0 == 2)
            && (i32::from(s_var5.wrapping_sub(s_var4))).unsigned_abs() < FAR_LINK_ANGLE as u32
            && len_2d_sq_pnt_and_seg_line(link.x, link.z, local_60.x, local_60.z, local_6c.x, local_6c.z)
                .is_some_and(|output2| output2 < FAR_LINK_LEN)
        {
            if self.m0 == 1 { 2 } else { 1 }
        } else {
            self.m0
        }
    }

    pub fn go_far_link_2(&mut self, link: Vec3, path: &PathRun) -> u8 {
        let mut stack = PathRun::new();
        stack.set_inf_drct(path);
        stack.set_idx(usize::from(self.point_index2));
        let local_8c = stack.get_point(stack.get_idx());
        stack.inc_idx_loop();
        let local_80 = stack.get_point(stack.get_idx());
        stack.dec_idx_loop();
        stack.dec_idx_loop();
        let local_74 = stack.get_point(stack.get_idx());
        let s_var5 = target_angle_y(local_8c, link);
        let s_var6 = target_angle_y(local_8c, local_80);
        let s_var7 = target_angle_y(local_8c, local_74);
        let i_var1 = i32::from(s_var6.wrapping_sub(s_var5));
        if i_var1.unsigned_abs() < FAR_LINK_ANGLE as u32
            && len_2d_sq_pnt_and_seg_line(
                link.x,
                link.z,
                local_8c.x,
                local_8c.z,
                local_80.x,
                local_80.z,
            )
            .is_some_and(|output2| output2 < FAR_LINK_LEN)
        {
            return 2;
        }
        let i_var2 = i32::from(s_var7.wrapping_sub(s_var5));
        if i_var2.unsigned_abs() < FAR_LINK_ANGLE as u32
            && len_2d_sq_pnt_and_seg_line(
                link.x,
                link.z,
                local_8c.x,
                local_8c.z,
                local_74.x,
                local_74.z,
            )
            .is_some_and(|output2| output2 < FAR_LINK_LEN)
        {
            return 1;
        }
        if stack.set_near_path_indx_mk2(link, usize::from(self.point_index2), MK2_RANGE) {
            let index_diff = stack.get_idx() as i32 - i32::from(self.point_index2);
            if (index_diff > 0) == (index_diff.unsigned_abs() <= u32::from(MK2_RANGE)) {
                return 2;
            }
            return 1;
        }
        if i_var1.unsigned_abs() >= i_var2.unsigned_abs() {
            1
        } else {
            2
        }
    }

    pub fn runaway_com2(&mut self, path: &mut PathRun, p2: u8) {
        path.set_idx(usize::from(self.point_index2));
        if p2 == 1 {
            path.inc_idx_loop();
        } else {
            path.dec_idx_loop();
        }
        self.point_index1 = path.get_idx() as u8;
    }

    pub fn run_away_proc(
        &mut self,
        yaw: &mut i16,
        link: Option<Vec3>,
        self_pos: Vec3,
        path: &mut PathRun,
        co_hit: bool,
        angle: &mut i16,
    ) -> u8 {
        if co_hit {
            *angle = link.map_or(*angle, |pos| target_angle_y(pos, self_pos));
            return 4;
        }
        if self.me != 0 {
            self.me -= 1;
        }
        match self.m0 {
            0 => {
                let dist = path.set_near_path_indx_mk(self_pos);
                self.point_index1 = path.get_idx() as u8;
                self.point_index2 = self.point_index1;
                if dist >= NEAR_PATH_50 { 5 } else { 3 }
            }
            5 => {
                let point = path.get_point(usize::from(self.point_index1));
                if (point - self_pos).length() < NEAR_PATH_20 {
                    3
                } else {
                    self.turn_path(yaw, self_pos, path, 2);
                    5
                }
            }
            3 => {
                let dist = link.map_or(F_INF, |pos| abs_xz(pos - self_pos));
                if dist < NEAR_PATH_800 {
                    self.me = NEAR_PATH_10;
                    let u_var6 = self.go_far_link_2(link.unwrap_or(Vec3::ZERO), path);
                    self.runaway_com2(path, u_var6);
                    return u_var6;
                }
                if self.m8 + 3 < self.ma {
                    self.m8 += 3;
                }
                3
            }
            1 | 2 => {
                let dist = link.map_or(F_INF, |pos| abs_xz(pos - self_pos));
                let options = if self.m0 == 1 { 2 } else { 3 };
                if self.walk_path(yaw, self_pos, path, options) {
                    if dist > NEAR_PATH_900 {
                        return 3;
                    }
                    let u_var6 = self.go_far_link_2(link.unwrap_or(Vec3::ZERO), path);
                    if u_var6 != self.m0 {
                        self.runaway_com2(path, u_var6);
                    }
                    return u_var6;
                }
                if self.me == 0 {
                    self.me = NEAR_PATH_10;
                    let u_var6 = self.go_far_link_3(self_pos, link.unwrap_or(Vec3::ZERO), path);
                    if u_var6 != self.m0 {
                        self.point_index2 = self.point_index1;
                        self.runaway_com2(path, u_var6);
                    }
                    return u_var6;
                } else if self.md {
                    if self.m8 >= 1 {
                        self.m8 -= 1;
                    }
                    if dist > NEAR_PATH_400 {
                        self.md = false;
                    }
                } else if dist < NEAR_PATH_200 {
                    self.md = true;
                }
                self.m0
            }
            _ => self.m0,
        }
    }

    pub fn set_rnd_path_pos(&mut self, roll: usize, path: &mut PathRun) {
        if path.is_path() {
            let index = roll % path.max_point().max(1);
            path.set_idx(index);
        }
    }

    pub fn chk_game_set(&self, ev_flags: &EventFlags) -> bool {
        ev_flags.test(TMP_0020)
            && ev_flags.test(TMP_0010)
            && ev_flags.test(TMP_0008)
            && ev_flags.test(TMP_0004)
    }

    pub fn chk_point_pass(&self, p1: Vec3, p2: Vec3, p3: Vec3) -> bool {
        if p1.x == p2.x && p1.z == p2.z {
            return true;
        }
        if p3.x == p2.x && p3.z == p2.z {
            return true;
        }
        (target_angle_y(p1, p2).wrapping_sub(target_angle_y(p3, p2)) as f32).cos() < 0.0
    }
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
    pub fn proc_name(&self) -> Option<u16> {
        if self.procs.search_by_name(FPC_NM_MSG).is_some() {
            Some(FPC_NM_MSG)
        } else if self.procs.search_by_name(FPC_NM_MSG2).is_some() {
            Some(FPC_NM_MSG2)
        } else {
            None
        }
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

    pub fn select_num(&self) -> u8 {
        match self.proc_name() {
            Some(FPC_NM_MSG) => self.one.select_num,
            _ => 0,
        }
    }

    pub fn message_set(&mut self, msg_no: u32, look_at: Vec3) -> u32 {
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

    pub fn reset_event(&mut self) {
        self.info.event_mode = crate::text::msgproc::DA_EVT_MODE_NONE;
    }
}

impl Ctx<'_> {
    pub fn reset_event(&mut self) {
        self.text.reset_event();
    }
}

pub struct Ctx<'a> {
    pub entity: Entity,
    pub player: Option<Player>,
    pub link: Option<Vec3>,
    pub link_shape_yaw: i16,
    pub lock_on: bool,
    pub lock_target: bool,
    pub action_target: bool,
    pub talk_command: bool,
    pub demo_accepted: bool,
    pub front_roll_crash: bool,
    pub ground_h: Option<f32>,
    pub event_idx: usize,
    pub cut: Option<(usize, bool)>,
    pub event_mode: bool,
    pub stories: &'a mut StoryBits,
    pub ev_flags: &'a mut EventFlags,
    pub list: &'a EventList,
    pub active: &'a mut ActiveEvent,
    pub events: &'a mut SimEvents,
    pub equipped: &'a Equipped,
    pub collect: &'a Collect,
    pub text: &'a mut Text<'a>,
    pub order_name: Option<&'static str>,
    pub present_item: Option<(Vec3, u8)>,
    pub shock: Option<(i32, i32, [f32; 3])>,
    pub delete_um2: bool,
    pub talk_partner_reset: bool,
    pub end_cut: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ivan {
    pub entered: bool,
    pub prm: u32,
    pub staffer: u8,
    pub ty: u8,
    pub flags: u16,
    pub order_flags: u8,
    pub state: i8,
    pub prev_state: i8,
    pub action: Action,
    pub action_status: i8,
    pub talk_state: i8,
    pub visit_mode: u8,
    pub wait_timer: u8,
    pub run_around_timer: u8,
    pub timer_to_reach_link: u8,
    pub flag_6a4: u16,
    pub anm_idx: i8,
    pub anm_timer: f32,
    pub anm_frame: f32,
    pub anm_ended: bool,
    pub morf_timer: f32,
    pub blink_frame: u8,
    pub blink_timer: i16,
    pub blink_max: i16,
    pub attn_set_count: u8,
    pub attention_timer: u8,
    pub attn_flags: u32,
    pub attn_dist_talk: i32,
    pub attn_dist_speak: i32,
    pub attn_pos: Vec3,
    pub attn_eye: Vec3,
    pub curr_msg_no: u32,
    pub msg_no: u32,
    pub msg_select_num: u8,
    pub msg_anm_idx: u8,
    pub msg_id: u32,
    pub staff_idx: i32,
    pub event_action: i32,
    pub event_idx: i32,
    pub event_order: Option<usize>,
    pub mtrl_snd_id: u32,
    pub reverb: i8,
    pub max_head_turn_vel: i16,
    pub jnt_param: [i16; 9],
    pub trn: bool,
    pub speak_request: bool,
    pub talk_partner_request: bool,
    pub eye_pos: Vec3,
    pub attn_base_pos: Vec3,
    pub speed: Vec3,
    pub speed_f: f32,
    pub gravity: f32,
    pub max_fall: f32,
    pub pos: Vec3,
    pub old: Vec3,
    pub yaw: i16,
    pub shape_yaw: i16,
    pub home: Vec3,
    pub home_yaw: i16,
    pub demo_from: Vec3,
    pub demo_to: Vec3,
    pub demo_timer: i16,
    pub run_angle: i16,
    pub mk: MkStatic,
    pub path_a: PathRun,
    pub path_b: PathRun,
    pub path_c: PathRun,
    pub cyl_radius: f32,
    pub cyl_height: f32,
    pub cyl_on: bool,
    pub stts_weight: u16,
    pub status: u32,
    pub grounded: bool,
    pub link_hit: bool,
    pub map_set: i32,
    pub seed: [i32; 3],
}

impl Ivan {
    pub fn spawn(parameters: u32) -> Self {
        let ty = ((parameters >> PRM_TYPE_SHIFT) & PRM_TYPE_MASK) as u8;
        let ty = match ty {
            TYPE_NORMAL => TYPE_NORMAL,
            TYPE_MINIGAME => TYPE_MINIGAME,
            _ => 0,
        };
        let staffer = if ty == TYPE_MINIGAME { 1 } else { 0 };
        Self {
            entered: false,
            prm: parameters,
            staffer,
            ty,
            flags: 0,
            order_flags: 0,
            state: STATE_WAIT,
            prev_state: STATE_WAIT,
            action: Action::None,
            action_status: ACTION_STARTING,
            talk_state: TALK_INIT,
            visit_mode: VISIT_START,
            wait_timer: 0,
            run_around_timer: 0,
            timer_to_reach_link: 0,
            flag_6a4: 0,
            anm_idx: 0,
            anm_timer: 0.0,
            anm_frame: 0.0,
            anm_ended: false,
            morf_timer: 0.0,
            blink_frame: 0,
            blink_timer: 0,
            blink_max: BLINK_FRAME_MAX,
            attn_set_count: 0,
            attention_timer: 0,
            attn_flags: 0,
            attn_dist_talk: 0,
            attn_dist_speak: 0,
            attn_pos: Vec3::ZERO,
            attn_eye: Vec3::ZERO,
            curr_msg_no: 0,
            msg_no: 0,
            msg_select_num: 0,
            msg_anm_idx: 0xFF,
            msg_id: FPC_M_ERROR_PROCESS_ID,
            staff_idx: NO_STAFF,
            event_action: NO_ACTION,
            event_idx: NO_EVENT_IDX,
            event_order: None,
            mtrl_snd_id: 0,
            reverb: 0,
            max_head_turn_vel: 0,
            jnt_param: JNT_PARAM,
            trn: false,
            speak_request: false,
            talk_partner_request: false,
            eye_pos: Vec3::ZERO,
            attn_base_pos: Vec3::ZERO,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            gravity: GRAVITY,
            max_fall: MAX_FALL,
            pos: Vec3::ZERO,
            old: Vec3::ZERO,
            yaw: 0,
            shape_yaw: 0,
            home: Vec3::ZERO,
            home_yaw: 0,
            demo_from: Vec3::ZERO,
            demo_to: Vec3::ZERO,
            demo_timer: 0,
            run_angle: 0,
            mk: MkStatic::new(),
            path_a: PathRun::new(),
            path_b: PathRun::new(),
            path_c: PathRun::new(),
            cyl_radius: CYL_RADIUS_SRC,
            cyl_height: CYL_HEIGHT_SRC,
            cyl_on: false,
            stts_weight: STTS_INIT_A0,
            status: STTS_SHOWMAP,
            grounded: false,
            link_hit: false,
            map_set: 0,
            seed: [RND_SEED; 3],
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        !self.chk_flag(FLAG_NO_MOVE)
    }

    pub fn get_type(&self) -> u8 {
        (self.prm & PRM_TYPE_MASK) as u8
    }

    pub fn get_path(&self) -> u8 {
        ((self.prm >> PRM_PATH_SHIFT) & PRM_PATH_MASK) as u8
    }

    pub fn chk_flag(&self, flag: u16) -> bool {
        self.flags & flag == flag
    }

    pub fn set_flag(&mut self, flag: u16) {
        self.flags |= flag;
    }

    pub fn clr_flag(&mut self, flag: u16) {
        self.flags &= !flag;
    }

    pub fn chk_order(&self, flag: u8) -> bool {
        self.order_flags & flag != 0
    }

    pub fn set_order(&mut self, flag: u8) {
        self.order_flags |= flag;
    }

    pub fn clr_order(&mut self, flag: u8) {
        self.order_flags &= !flag;
    }

    pub fn inc_attn_set_count(&mut self) {
        if self.attn_set_count != 0xFF {
            self.attn_set_count += 1;
        }
    }

    pub fn rnd(&mut self) -> f32 {
        rnd(&mut self.seed)
    }

    pub fn rnd_f(&mut self, range: f32) -> f32 {
        rnd(&mut self.seed) * range
    }

    pub fn staff_name(&self) -> &'static str {
        STAFF_NAMES[usize::from(self.staffer).min(STAFF_NAMES.len() - 1)]
    }

    pub fn set_anm(&mut self, new_anm_idx: i8) {
        let mut morf = MORF_RATE;
        if new_anm_idx == 5 && self.anm_idx == 11 {
            morf = 0.0;
        }
        if new_anm_idx != self.anm_idx && new_anm_idx != -1 {
            self.anm_idx = new_anm_idx;
            self.anm_timer = 0.0;
            self.morf_timer = morf;
            self.anm_frame = 0.0;
        }
    }

    pub fn anm_max(&self) -> f32 {
        let index = usize::from(self.anm_idx.max(0) as u8).min(ANM_MAX.len() - 1);
        ANM_MAX[index]
    }

    pub fn is_morf(&self) -> bool {
        self.morf_timer > 0.0
    }

    pub fn play_morf(&mut self) {
        let max = self.anm_max();
        let next = self.anm_frame + 1.0;
        self.anm_frame = if next > max { 0.0 } else { next };
        let mut ended = self.anm_frame >= max;
        if self.anm_frame < self.anm_timer {
            ended = true;
        }
        self.anm_ended = ended;
        self.anm_timer = self.anm_frame;
        if self.morf_timer > 0.0 {
            self.morf_timer -= 1.0;
        }
    }

    pub fn set_anm_status(&mut self) {
        match self.state {
            STATE_DEMO01 => self.set_anm(2),
            STATE_CLIMB01 | STATE_RUNAWAY => self.set_anm(6),
            STATE_JITANDA02 => {}
            _ => self.set_anm(0),
        }
    }

    pub fn init_tex_pattern_anm(&mut self) -> bool {
        self.blink_frame = 0;
        self.blink_timer = 0;
        true
    }

    pub fn play_tex_pattern_anm(&mut self) {
        if calc_timer(&mut self.blink_timer) {
            if self.blink_frame >= self.blink_max as u8 {
                self.blink_frame = self.blink_frame.wrapping_sub(self.blink_max as u8);
                self.blink_timer = self.rnd_f(BLINK_RANGE) as i16 + BLINK_BASE as i16;
            } else {
                self.blink_frame += 1;
            }
        }
    }

    pub fn chk_attention_local(&mut self, ctx: &Ctx) -> bool {
        if self.chk_flag(FLAG_TALK) {
            return true;
        }
        if self.attention_timer != 0 {
            self.attention_timer -= 1;
            return true;
        }
        if ctx.lock_on {
            return ctx.lock_target;
        }
        ctx.action_target
    }

    pub fn chk_attention(&mut self, ctx: &Ctx) {
        let temp = self.chk_flag(FLAG_ATTENTION);
        if self.chk_attention_local(ctx) {
            self.set_flag(FLAG_ATTENTION);
        } else {
            self.clr_flag(FLAG_ATTENTION);
        }
        if temp != self.chk_flag(FLAG_ATTENTION) && temp {
            self.trn = true;
        }
    }

    pub fn event_order(&mut self, _ctx: &mut Ctx) {
        if self.chk_order(ORDER_CANTALK) && self.chk_order(ORDER_SPEAK) {
            self.speak_request = true;
        }
    }

    pub fn check_order(&mut self, ctx: &mut Ctx) {
        if ctx.talk_command && self.chk_order(ORDER_LOCK) {
            self.set_flag(FLAG_TALK);
            self.talk_init();
        }
        self.clr_order(0xFF);
    }

    pub fn talk_init(&mut self) {
        self.talk_state = TALK_INIT;
        self.msg_anm_idx = 0xFF;
    }

    pub fn next_msg_status(&mut self, msg_no: &mut u32) -> u8 {
        let mut msg_status = STATUS_MSG_CONTINUES;
        if MSG_CHAIN.contains(msg_no) {
            *msg_no += 1;
        } else if *msg_no == MSG_SELECT_BOX {
            if self.msg_select_num == 0 {
                self.set_flag(FLAG_SELECT);
                msg_status = STATUS_MSG_ENDS;
            } else {
                self.flag_6a4 = 1;
                *msg_no = 0x26CB;
            }
        } else {
            msg_status = STATUS_MSG_ENDS;
        }
        msg_status
    }

    pub fn get_msg(&mut self, ctx: &mut Ctx) -> u32 {
        let mut msg_no = 0;
        self.clr_flag(FLAG_SELECT);
        match self.ty {
            TYPE_NORMAL => {
                if self.state == STATE_DEMO01
                    || self.state == STATE_DEMO02
                    || self.state == STATE_DEMO03
                {
                    msg_no = self.msg_no;
                } else if ctx.ev_flags.test(TMP_0040) {
                    msg_no = MSG_TALK_HOLD;
                } else if ctx.stories.test(BIT_1340) {
                    msg_no = MSG_TALK_PLAY;
                } else if ctx.stories.test(BIT_1380) {
                    msg_no = MSG_TALK_HUNTED;
                } else if ctx.stories.test(BIT_1210) {
                    msg_no = MSG_TALK_NOTICE;
                } else {
                    msg_no = MSG_TALK_DEFAULT;
                    ctx.stories.set(BIT_1210);
                }
            }
            TYPE_MINIGAME => {
                if ctx.ev_flags.test(TMP_0020) {
                    msg_no = MSG_TALK_HOLD;
                } else {
                    msg_no = MSG_TALK_MINIGAME;
                    ctx.ev_flags.set(TMP_0020);
                }
            }
            _ => {}
        }
        msg_no
    }

    pub fn set_collision(&mut self) {
        self.cyl_radius = CYL_RADIUS_RUN;
        self.cyl_height = CYL_HEIGHT_RUN;
    }

    pub fn msg_anm(&mut self, param_1: u8, ctx: &mut Ctx) {
        if self.msg_anm_idx != param_1 {
            self.msg_anm_idx = param_1;
            match self.msg_anm_idx {
                0 => self.set_anm(0),
                1 => self.set_anm(1),
                2 => self.set_anm(2),
                3 => {
                    ctx.events.list.push(SimEvent::Se(SE_START_WHISTLE));
                    self.set_anm(3);
                }
                4 => self.set_anm(7),
                5 => self.set_anm(4),
                6 => self.set_anm(5),
                7 => self.set_anm(6),
                8 => self.set_anm(9),
                _ => {}
            }
        }
    }

    pub fn msg_push_button(&mut self, text: &Text) {
        if self.curr_msg_no != MSG_SELECT_BOX {
            return;
        }
        self.msg_select_num = text.select_num();
    }

    pub fn talk(&mut self, ctx: &mut Ctx) -> u8 {
        let mut msg_status = 0xFF;
        if self.talk_state == TALK_INIT {
            self.msg_id = FPC_M_ERROR_PROCESS_ID;
            self.curr_msg_no = self.get_msg(ctx);
            self.talk_state = TALK_MSG_CREATE;
        } else if self.talk_state == TALK_FINISHED {
            msg_status = 0xFE;
        } else if self.msg_id == FPC_M_ERROR_PROCESS_ID {
            let msg_no = self.curr_msg_no;
            self.msg_id = ctx.text.message_set(msg_no, self.eye_pos);
        } else {
            if !self.chk_flag(FLAG_NO_MSG_ANM) {
                let attr = ctx.text.info.mesg_anime;
                self.msg_anm(attr, ctx);
            }
            match self.talk_state {
                TALK_MSG_CREATE => {
                    if ctx.text.procs.search_by_id(self.msg_id).is_some() {
                        self.talk_state = TALK_ACTIVE;
                    }
                }
                TALK_ACTIVE => {
                    msg_status = ctx.text.status().unwrap_or(0);
                    if msg_status == STATUS_MSG_DISPLAYED {
                        self.msg_push_button(ctx.text);
                        let mut curr = self.curr_msg_no;
                    let next = self.next_msg_status(&mut curr);
                    self.curr_msg_no = curr;
                        ctx.text.set_status(next);
                        if next == STATUS_MSG_CONTINUES {
                            let msg_no = self.curr_msg_no;
                            let _ = ctx.text.message_set(msg_no, Vec3::ZERO);
                        }
                    } else if msg_status == STATUS_BOX_CLOSED {
                        ctx.text.set_status(STATUS_MSG_DESTROYED);
                        self.talk_state = TALK_FINISHED;
                    }
                }
                _ => {}
            }
        }
        msg_status
    }

    pub fn check_demo_start(&mut self, ctx: &Ctx) -> bool {
        self.staff_idx = match ctx.list.staff_of(self.staff_name()) {
            Some(slot) => slot as i32,
            None => NO_STAFF,
        };
        if self.staff_idx != -1 {
            self.event_action = self.get_now_event_action(ctx);
            return true;
        }
        false
    }

    pub fn get_now_event_action(&self, ctx: &Ctx) -> i32 {
        let Some((cut, _)) = ctx.cut else {
            return NO_ACTION;
        };
        let Some(name) = ctx.list.cut_name(cut) else {
            return NO_ACTION;
        };
        match StaffAction::of_name(name) {
            Some(action) => action.index(),
            None => NO_ACTION,
        }
    }

    pub fn cut_number(&self, ctx: &Ctx, key: &str) -> Option<i32> {
        let (cut, _) = ctx.cut?;
        ctx.list.cut_number(cut, key)
    }

    pub fn cut_vec(&self, ctx: &Ctx, key: &str) -> Option<Vec3> {
        let (cut, _) = ctx.cut?;
        ctx.list.cut_vec(cut, key)
    }

    pub fn remove_um2(&mut self, ctx: &mut Ctx) {
        if self.cut_number(ctx, RM_UM2_KEY).is_some() {
            ctx.delete_um2 = true;
        }
    }

    pub fn demo_init_com(&mut self, ctx: &mut Ctx) {
        self.remove_um2(ctx);
        match self.cut_number(ctx, SOUND_KEY) {
            Some(1) => ctx.events.list.push(SimEvent::Se(SE_CV_TR_KO_A_RUN_AWAY)),
            Some(2) => ctx.events.list.push(SimEvent::Se(SE_CV_TR_KO_A_FOUND)),
            _ => {}
        }
    }

    pub fn demo_proc(&mut self, ctx: &mut Ctx) -> bool {
        let mut temp = false;
        if ctx.cut.is_some_and(|(_, advance)| advance) {
            self.demo_init_com(ctx);
            match self.event_action {
                0 => {
                    self.speed_f = 0.0;
                    self.set_anm(0);
                }
                1 | 10 => {
                    self.speed_f = 0.0;
                    self.set_anm(0);
                }
                2 => {
                    self.speed_f = 0.0;
                    self.old = self.home;
                    self.pos = self.home;
                    self.yaw = self.home_yaw;
                }
                3 | 4 => {
                    if self.anm_idx == 5 {
                        self.speed_f = RUN_SPEED_WALK;
                    } else if self.anm_idx == 11 {
                        self.speed_f = RUN_SPEED_LAND;
                        self.set_anm(5);
                    } else {
                        self.set_anm(5);
                    }
                    if let Some(pos) = self.cut_vec(ctx, POS_KEY) {
                        self.demo_from = self.pos;
                        self.demo_to = pos;
                    }
                    self.demo_timer = self.cut_number(ctx, TIMER_KEY).unwrap_or(-1) as i16;
                }
                5 => {
                    self.talk_init();
                    self.talk_partner_request = true;
                    self.msg_no = self.cut_number(ctx, MSGNO_KEY).unwrap_or(0) as u32;
                    temp = true;
                }
                6 => {
                    self.talk_partner_request = true;
                }
                8 => {
                    self.set_flag(FLAG_JUMP);
                    self.gravity = GRAVITY_RUN;
                    self.speed.y = JUMP_Y;
                    self.speed_f = JUMP_SPEED_F;
                    self.set_flag(FLAG_AIR);
                    self.set_anm(10);
                }
                7 => {
                    self.set_flag(FLAG_JUMP);
                    self.gravity = GRAVITY_JUMP2;
                    self.speed.y = JUMP2_Y;
                    self.speed_f = JUMP2_SPEED_F;
                    self.set_flag(FLAG_AIR);
                    self.set_anm(10);
                }
                9 => {
                    self.set_flag(FLAG_JUMP);
                    self.set_anm(7);
                }
                11 => {
                    self.speed_f = 0.0;
                    self.set_flag(FLAG_NO_MOVE);
                    self.prev_state = STATE_13;
                }
                12 => {
                    self.action = Action::None;
                    self.action = Action::Visit;
                    self.action_status = ACTION_STARTING;
                    self.run_action(Action::Visit, ctx);

                    self.prev_state = self.state;
                    self.state = STATE_DEMO01;
                    self.clr_flag(FLAG_NO_MOVE);
                    self.status |= STTS_SHOWMAP;
                }
                13 => {
                    ctx.present_item = Some((self.pos, ITEM_HEART_PIECE));
                    ctx.stories.set(BIT_1340);
                }
                14 => {
                    ctx.ev_flags.clear(TMP_0040);
                    ctx.ev_flags.set(TMP_0120);
                }
                15 => {
                    self.speed_f = 0.0;
                    if let Some(pos) = self.cut_vec(ctx, POS_KEY) {
                        self.pos = pos;
                        self.old = pos;
                    }
                    if let Some(angle) = self.cut_number(ctx, ANGLE_KEY) {
                        self.yaw = angle as i16;
                        self.shape_yaw = angle as i16;
                    }
                }
                16 => {
                    self.run_angle = self.cut_number(ctx, ANGLE_KEY).unwrap_or(0) as i16;
                }
                _ => {}
            }
        }
        match self.event_action {
            3 | 4 => {
                let (dist, angle) = dis_xz_ang_y(self.pos, self.demo_to);
                if dist < self.speed_f
                    || self
                        .mk
                        .chk_point_pass(self.demo_from, self.demo_to, self.pos)
                {
                    if self.event_action == 4 {
                        self.pos.x = self.demo_to.x;
                        self.pos.z = self.demo_to.z;
                        self.speed_f = 0.0;
                    }
                    ctx.end_cut = true;
                } else {
                    self.yaw = walk::add_calc_angle_s(
                        self.yaw,
                        angle,
                        TURN_SCALE,
                        TURN_STEP,
                        TURN_MIN,
                    );
                    walk::chase_f(&mut self.speed_f, DEMO_TARGET_SPEED, DEMO_CHASE);
                    if self.demo_timer > 0 {
                        self.demo_timer -= 1;
                    }
                    if self.demo_timer == 0 {
                        ctx.end_cut = true;
                    }
                }
                true
            }
            5 => {
                let status = self.talk(ctx);
                if status == STATUS_BOX_CLOSED || status == 0xFE {
                    ctx.end_cut = true;
                }
                self.set_flag(FLAG_STAND);
                if temp {
                    return true;
                }
                self.is_morf()
            }
            2 | 6 | 15 => {
                ctx.end_cut = true;
                true
            }
            1 => {
                self.set_flag(FLAG_STAND);
                ctx.end_cut = true;
                if temp {
                    return true;
                }
                self.is_morf()
            }
            10 => {
                self.set_flag(FLAG_TURN_STOP);
                ctx.end_cut = true;
                if temp {
                    return true;
                }
                self.is_morf()
            }
            7 | 8 => {
                if !self.chk_flag(FLAG_AIR) {
                    self.set_anm(11);
                    self.gravity = GRAVITY;
                    self.speed_f = 0.0;
                    if self.chk_flag(FLAG_JUMP) {
                        self.clr_flag(FLAG_JUMP);
                        if self.event_action == 7 {
                            ctx.shock = Some((SHOCK_FRAME, SHOCK_MAGNITUDE, SHOCK_DIR));
                        }
                    } else if self.anm_ended {
                        ctx.end_cut = true;
                    }
                }
                true
            }
            9 => {
                if self.chk_flag(FLAG_JUMP) {
                    if self.anm_ended {
                        ctx.end_cut = true;
                        self.clr_flag(FLAG_JUMP);
                    }
                } else {
                    ctx.end_cut = true;
                }
                if temp {
                    return true;
                }
                self.is_morf()
            }
            16 => {
                if walk::add_calc_angle_s(
                    self.yaw,
                    self.run_angle,
                    TURN_SCALE,
                    TURN_STEP,
                    TURN_MIN,
                ) == 0
                {
                    ctx.end_cut = true;
                }
                if temp {
                    return true;
                }
                self.is_morf()
            }
            _ => {
                ctx.end_cut = true;
                if temp {
                    return true;
                }
                self.is_morf()
            }
        }
    }

    pub fn set_attention(&mut self, param_1: bool) {
        if !param_1 && self.attn_set_count >= 2 {
            return;
        }
        self.attn_eye = self.eye_pos;
        self.attn_pos = Vec3::new(
            self.attn_base_pos.x,
            self.attn_base_pos.y + ATTN_Y_LIFT,
            self.attn_base_pos.z,
        );
    }

    pub fn get_look_back_mode(&self) -> u8 {
        if self.chk_flag(FLAG_TURN_STOP) {
            return 3;
        }
        if self.chk_flag(FLAG_STAND) {
            return 0;
        }
        if self.flag_6a4 != 0 {
            return 2;
        }
        if self.chk_flag(FLAG_GAMESET) || self.anm_idx == 7 || self.anm_idx == 8 {
            return 2;
        }
        if self.chk_flag(FLAG_ATTENTION) {
            return 1;
        }
        if self.state == STATE_VISIT
            && (self.visit_mode == VISIT_WALK_AROUND_LINK
                || self.visit_mode == VISIT_REACHED_LINK)
        {
            return 1;
        }
        2
    }

    pub fn look_back(&mut self, ctx: &Ctx) {
        let mut desired_y_rot = self.yaw;
        let mut head_only_follow = false;
        let mut dst_pos = None;
        let mut temp = Vec3::ZERO;
        match self.get_look_back_mode() {
            0 => {
                self.trn = true;
                dst_pos = ctx.link.map(|pos| Vec3::new(pos.x, pos.y + LOOKBACK_PLAYER_EYE, pos.z));
                temp = self.pos;
                temp.y = self.attn_eye.y;
            }
            1 => {
                dst_pos = ctx.link.map(|pos| Vec3::new(pos.x, pos.y + LOOKBACK_PLAYER_EYE, pos.z));
                temp = self.pos;
                temp.y = self.attn_eye.y;
                head_only_follow = true;
            }
            2 => {
                desired_y_rot = self.yaw;
                head_only_follow = true;
            }
            _ => {
                self.trn = true;
                dst_pos = ctx.link.map(|pos| pos);
                temp = self.pos;
                temp.y = self.attn_eye.y;
            }
        }
        if self.trn {
            stream::add_calc_angle_s2(
                &mut self.max_head_turn_vel,
                HEAD_TURN_MAX,
                HEAD_TURN_SCALE,
                HEAD_TURN_STEP,
            );
        } else {
            self.max_head_turn_vel = 0;
        }
        let _ = (dst_pos, temp, desired_y_rot, head_only_follow);
    }

    pub fn wait01(&mut self) -> bool {
        if self.chk_flag(FLAG_TALK) {
            self.prev_state = self.state;
            self.state = STATE_TALK01;
            self.set_anm_status();
        } else {
            self.set_order(ORDER_TALK);
        }
        self.is_morf()
    }

    pub fn talk01(&mut self, ctx: &mut Ctx) -> bool {
        if self.talk(ctx) == STATUS_BOX_CLOSED {
            if self.chk_flag(FLAG_SELECT) {
                self.event_idx = event_index(ctx, EVENT_MK_GAMESTART);
                self.order_change_event(ctx);
                self.state = STATE_DEMO02;
                self.set_flag(FLAG_GAMESET);
                ctx.ev_flags.set(TMP_0040);
                ctx.stories.set(BIT_2201);
            } else {
                self.state = self.prev_state;
                self.set_anm_status();
                ctx.talk_partner_reset = true;
                self.clr_flag(FLAG_TALK);
                self.attention_timer = 5;
            }
        }
        self.set_flag(FLAG_STAND);
        self.is_morf()
    }

    pub fn talk02(&mut self, ctx: &mut Ctx) -> bool {
        if self.talk(ctx) == STATUS_BOX_CLOSED {
            if !self.mk.chk_game_set(ctx.ev_flags) {
                self.state = self.prev_state;
                self.set_anm_status();
                ctx.talk_partner_reset = true;
            } else {
                self.event_idx = event_index(ctx, EVENT_MK_GAMESET);
                self.order_change_event(ctx);
                ctx.events.list.push(SimEvent::Se(SE_END_WHISTLE));
                self.state = STATE_DEMO02;
                self.set_flag(FLAG_GAMESET);
            }
            self.clr_flag(FLAG_TALK);
            self.attention_timer = 5;
        }
        self.set_flag(FLAG_STAND);
        self.is_morf()
    }

    pub fn visit_talk_init(&mut self, ctx: &mut Ctx) -> u8 {
        if ctx.stories.test(BIT_1F80) && !ctx.stories.test(BIT_1E02) {
            self.event_idx = ctx.event_idx as i32;
            self.state = STATE_DEMO03;
            self.demo03(ctx);
            ctx.stories.set(BIT_1E04);
            return 12;
        }
        if ctx.stories.test(BIT_1208)
            && !item_obtained(ctx.equipped, ctx.collect, ITEM_PICTO_BOX)
            && !item_obtained(ctx.equipped, ctx.collect, ITEM_DELUXE_PICTO_BOX)
        {
            self.event_idx = ctx.event_idx as i32;
            self.state = STATE_DEMO03;
            self.demo03(ctx);
            ctx.stories.set(BIT_1602);
            return 12;
        }
        1
    }

    pub fn visit_set_event(&self, ctx: &Ctx) -> &'static str {
        if ctx.stories.test(BIT_1F80) && !ctx.stories.test(BIT_1E02) {
            return EVENT_MK_TALK3;
        }
        if ctx.stories.test(BIT_1208)
            && !item_obtained(ctx.equipped, ctx.collect, ITEM_PICTO_BOX)
            && !item_obtained(ctx.equipped, ctx.collect, ITEM_DELUXE_PICTO_BOX)
        {
            return EVENT_MK_TALK2;
        }
        EVENT_MK_TALK
    }

    pub fn visit01(&mut self, ctx: &mut Ctx) -> bool {
        if self.chk_flag(FLAG_TALK) {
            self.prev_state = self.state;
            self.state = self.visit_talk_init(ctx) as i8;
            self.set_anm_status();
            self.speed_f = 0.0;
            self.visit_mode = VISIT_TALK;
            if ctx.ev_flags.test(TMP_0580) {
                ctx.ev_flags.clear(TMP_0580);
            }
        } else if self.check_demo_start(ctx) {
            self.prev_state = self.state;
            self.state = STATE_DEMO01;
            self.set_anm_status();
            self.speed_f = 0.0;
            self.visit_mode = VISIT_TALK;
            self.demo01(ctx);
        } else {
            let next_visit_mode = self.next_visit_mode(ctx);
            if next_visit_mode != self.visit_mode {
                self.visit_init(next_visit_mode);
            }
            self.visit_proc(ctx);
            let mut temp = 0.0;
            if self.anm_idx == 4 {
                temp = VISIT_SPEED_WALK;
            } else if self.anm_idx == 5 {
                if self.visit_mode == VISIT_REACHED_LINK {
                    temp = VISIT_SPEED_AROUND;
                } else {
                    temp = VISIT_SPEED_LINK;
                }
            }
            if self.speed_f > temp {
                walk::chase_f(&mut self.speed_f, temp, CHASE_FAST);
            } else {
                walk::chase_f(&mut self.speed_f, temp, CHASE_SLOW);
            }
            self.set_order(ORDER_TALK);
            if self.visit_mode == VISIT_WALK_AROUND_LINK || self.visit_mode == VISIT_REACHED_LINK {
                if let Some(link) = ctx.link {
                    let angle = target_angle_y(link, self.pos)
                        .wrapping_sub(ctx.link_shape_yaw);
                    let angle = if angle < 0 { -angle } else { angle };
                    if angle > LOOK_ANGLE {
                        self.order_flags &= !0x01;
                    }
                }
            }
            if self.flag_6a4 != 0 {
                self.order_flags &= !0x01;
            }
            if ctx.stories.test(BIT_1F80)
                && !ctx.stories.test(BIT_1E02)
                && ctx.ev_flags.test(TMP_0580)
            {
                self.set_order(ORDER_SPEAK);
                ctx.order_name = Some(EVENT_MK_TALK4);
            } else if self.chk_order(ORDER_TALK) {
                ctx.order_name = Some(self.visit_set_event(ctx));
            }
        }
        true
    }

    pub fn next_visit_mode(&mut self, ctx: &Ctx) -> u8 {
        match self.visit_mode {
            VISIT_START => VISIT_WALK_PATH,
            VISIT_TALK => {
                if self.flag_6a4 != 0 {
                    VISIT_WALK_PATH_FAST
                } else {
                    VISIT_RUN_LINK
                }
            }
            VISIT_WALK_PATH => {
                let Some(link) = ctx.link else {
                    return VISIT_WALK_PATH;
                };
                if abs2_xz(link - self.pos) < SQUARE_400 && self.path_c.chk_inside(&link) {
                    return VISIT_NOTICE_LINK;
                }
                VISIT_WALK_PATH
            }
            VISIT_RUN_LINK => {
                let Some(link) = ctx.link else {
                    return VISIT_RUN_LINK;
                };
                if !self.path_b.chk_inside(&link) {
                    return VISIT_LEFT_PATH;
                }
                if abs2_xz(link - self.pos) < SQUARE_150 {
                    return VISIT_REACHED_LINK;
                }
                VISIT_RUN_LINK
            }
            VISIT_WALK_AROUND_LINK | VISIT_REACHED_LINK => {
                let Some(link) = ctx.link else {
                    return self.visit_mode;
                };
                if !self.path_b.chk_inside(&link) {
                    return VISIT_LEFT_PATH;
                }
                if abs2_xz(link - self.pos) > SQUARE_180 {
                    return VISIT_RUN_LINK;
                }
                if self.visit_mode == VISIT_REACHED_LINK {
                    if self.run_around_timer != 0 {
                        self.run_around_timer -= 1;
                    } else {
                        return VISIT_WALK_AROUND_LINK;
                    }
                }
                self.visit_mode
            }
            VISIT_NOTICE_LINK | VISIT_LEFT_PATH => {
                if self.wait_timer != 0 {
                    self.wait_timer -= 1;
                    return self.visit_mode;
                }
                let Some(link) = ctx.link else {
                    return VISIT_WALK_PATH;
                };
                if abs2_xz(link - self.pos) < SQUARE_400 && self.path_c.chk_inside(&link) {
                    return VISIT_RUN_LINK;
                }
                VISIT_WALK_PATH
            }
            VISIT_WALK_PATH_FAST => {
                if self.flag_6a4 != 0 {
                    self.flag_6a4 -= 1;
                    return self.visit_mode;
                }
                VISIT_WALK_PATH_IGNORE_LINK
            }
            VISIT_WALK_PATH_IGNORE_LINK => {
                if self.flag_6a4 != 0 {
                    self.flag_6a4 -= 1;
                    return self.visit_mode;
                }
                VISIT_WALK_PATH
            }
            _ => self.visit_mode,
        }
    }

    pub fn visit_init(&mut self, next_visit_mode: u8) {
        match next_visit_mode {
            VISIT_WALK_PATH => {
                self.path_a.set_near_path_indx(self.pos, NEAR_PATH_RANGE);
                self.set_anm(4);
            }
            VISIT_RUN_LINK => {
                self.set_anm(5);
                if self.visit_mode == VISIT_NOTICE_LINK {
                    self.timer_to_reach_link = REACHED_LINK_TIMER;
                } else {
                    self.timer_to_reach_link = 0;
                }
            }
            VISIT_WALK_AROUND_LINK => {
                self.set_anm(4);
            }
            VISIT_REACHED_LINK => {
                self.run_around_timer = self.timer_to_reach_link;
                self.set_anm(5);
            }
            VISIT_NOTICE_LINK => {
                self.set_anm(0);
                self.speed_f = 0.0;
                self.wait_timer = WAIT_TIMER_NOTICE;
            }
            VISIT_LEFT_PATH => {
                self.set_anm(0);
                self.speed_f = 0.0;
                self.wait_timer = WAIT_TIMER_LEFT;
            }
            VISIT_WALK_PATH_FAST => {
                self.path_a.set_near_path_indx(self.pos, NEAR_PATH_RANGE);
                self.set_anm(5);
                self.flag_6a4 = FAST_WALK_LIMIT;
            }
            VISIT_WALK_PATH_IGNORE_LINK => {
                self.set_anm(4);
                self.flag_6a4 = IGNORE_LINK_LIMIT;
            }
            _ => {}
        }
        self.visit_mode = next_visit_mode;
    }

    pub fn visit_proc(&mut self, ctx: &mut Ctx) {
        match self.visit_mode {
            VISIT_WALK_PATH | VISIT_WALK_PATH_IGNORE_LINK => {
                self.walk_path(0);
            }
            VISIT_WALK_PATH_FAST => {
                self.walk_path(2);
            }
            VISIT_RUN_LINK => {
                self.run_link(ctx);
                if self.timer_to_reach_link < REACHED_LINK_TIMER {
                    self.timer_to_reach_link += 1;
                }
            }
            VISIT_WALK_AROUND_LINK | VISIT_REACHED_LINK => {
                self.around_link(ctx);
            }
            VISIT_NOTICE_LINK | VISIT_LEFT_PATH => {
                self.set_flag(FLAG_STAND);
            }
            _ => {}
        }
    }

    pub fn runaway_init(&mut self, ctx: &mut Ctx) {
        match self.mk.m0 {
            1 | 2 | 5 => {
                if self.anm_idx == 6 {
                    ctx.events.list.push(SimEvent::Se(SE_CV_TR_KO_A_FOUND));
                }
                self.speed_f = 0.0;
                self.set_anm(5);
            }
            3 => {
                self.speed_f = 0.0;
                self.set_anm(6);
            }
            4 => {
                self.speed.y = RUN_JUMP_Y;
                self.gravity = GRAVITY_RUN;
                self.speed_f = RUN_SPEED;
                self.set_flag(FLAG_AIR);
                self.set_anm(9);
                self.state = STATE_JUMP;
                self.yaw = self.run_angle;
                ctx.events.list.push(SimEvent::Se(SE_CV_TR_KO_A_CAUGHT));
            }
            _ => {}
        }
    }

    pub fn walk_path(&mut self, param_1: u8) {
        let mut yaw = self.yaw;
        let mut path = self.path_a;
        if self.mk.walk_path(&mut yaw, self.pos, &mut path, param_1) {
            self.set_flag(FLAG_TURN_PATH);
        }
        self.path_a = path;
        self.yaw = yaw;
    }

    pub fn run_link(&mut self, ctx: &Ctx) {
        let Some(link) = ctx.link else {
            return;
        };
        let angle = dis_xz_ang_y(self.pos, link).1;
        stream::add_calc_angle_s2(&mut self.yaw, angle, TURN_LINK_SCALE, TURN_LINK_STEP);
    }

    pub fn around_link(&mut self, ctx: &Ctx) {
        let Some(link) = ctx.link else {
            return;
        };
        let mut yaw = self.yaw;
        self.mk
            .around_walk(&mut yaw, self.pos, link, self.run_around_timer);
        self.yaw = yaw;
    }

    pub fn runaway(&mut self, ctx: &mut Ctx) -> bool {
        let mut path = self.path_a;
        let mut yaw = self.yaw;
        let mut angle = self.run_angle;
        let temp = self.mk.run_away_proc(
            &mut yaw,
            ctx.link,
            self.pos,
            &mut path,
            self.link_hit,
            &mut angle,
        );
        self.path_a = path;
        self.yaw = yaw;
        self.run_angle = angle;
        if temp != self.mk.m0 {
            if (temp == 1 && self.mk.m0 == 2) || (temp == 2 && self.mk.m0 == 1) {
                if self.rnd_f(RND_RANGE) < RND_CHANCE {
                    ctx.events.list.push(SimEvent::Se(SE_CV_TR_KO_A_TURN));
                }
            }
            self.mk.m0 = temp;
            self.runaway_init(ctx);
        }
        if temp == 3 {
            self.set_flag(FLAG_STAND);
        }
        if self.anm_idx == 5 {
            let speed = self.mk.get_speed_f(RUNAWAY_MIN_SPEED, RUNAWAY_MAX_SPEED);
            walk::chase_f(&mut self.speed_f, speed, RUN_CHASE);
        }
        true
    }

    pub fn jump(&mut self) -> bool {
        if !self.chk_flag(FLAG_AIR) {
            self.set_anm(0);
            self.gravity = GRAVITY;
            self.speed_f = 0.0;
            if self.is_morf() {
                self.state = STATE_JITANDA01;
                self.set_anm(7);
                self.set_order(ORDER_SPEAK);
            }
        }
        true
    }

    pub fn jitanda01(&mut self) -> bool {
        if self.chk_flag(FLAG_TALK) {
            self.set_flag(FLAG_NO_MSG_ANM);
            self.prev_state = STATE_JITANDA02;
            self.state = STATE_TALK02;
        } else {
            self.set_order(ORDER_SPEAK);
        }
        true
    }

    pub fn jitanda02(&mut self, ctx: &mut Ctx) -> bool {
        if self.chk_flag(FLAG_TALK) {
            self.prev_state = STATE_JITANDA02;
            self.state = STATE_TALK02;
        } else if self.check_demo_start(ctx) {
            self.prev_state = self.state;
            self.state = STATE_DEMO01;
            self.speed_f = 0.0;
            self.demo01(ctx);
        } else {
            self.set_order(ORDER_TALK);
        }
        true
    }

    pub fn demo01(&mut self, ctx: &mut Ctx) -> bool {
        if !self.check_demo_start(ctx) {
            self.state = self.prev_state;
            self.set_anm_status();
            return true;
        }
        self.demo_proc(ctx)
    }

    pub fn demo02(&mut self, ctx: &mut Ctx) -> bool {
        if event_end(self.event_idx, ctx) {
            ctx.text.reset_event();
            let _ = self.set_action(Action::Hind, ctx);
            self.set_flag(FLAG_NO_MOVE);
            self.status &= !STTS_SHOWMAP;
            self.clr_flag(FLAG_TALK);
            self.clr_flag(FLAG_GAMESET);
            return true;
        }
        if !self.check_demo_start(ctx) {
            return true;
        }
        self.demo_proc(ctx)
    }

    pub fn demo03(&mut self, ctx: &mut Ctx) -> bool {
        if event_end(self.event_idx, ctx) {
            self.state = self.prev_state;
            self.set_anm_status();
            ctx.text.reset_event();
            self.clr_flag(FLAG_TALK);
            self.attention_timer = 5;
            return true;
        }
        if !self.check_demo_start(ctx) {
            return true;
        }
        self.demo_proc(ctx)
    }

    pub fn climb01(&mut self, ctx: &mut Ctx) -> bool {
        if self.chk_flag(FLAG_AIR) {
            return false;
        }
        if ctx.front_roll_crash {
            let Some(link) = ctx.link else {
                return self.is_morf();
            };
            let temp = dis_xz_ang_y(self.pos, link).0;
            if temp < CLIMB_DIST {
                self.state = STATE_DROP01;
                self.event_idx = event_index(ctx, EVENT_MK_DROP);
                self.order_other_event(ctx);
                ctx.ev_flags.set(TMP_0002);
            }
        }
        self.is_morf()
    }

    pub fn drop01(&mut self, ctx: &mut Ctx) -> bool {
        if ctx.demo_accepted {
            self.prev_state = STATE_RUNAWAY;
            self.state = STATE_DEMO03;
            self.mk.init(MK_INIT_P0, MK_INIT_P1);
            return self.demo03(ctx);
        }
        self.order_other_event(ctx);
        self.is_morf()
    }

    pub fn order_change_event(&mut self, ctx: &Ctx) {
        self.event_order = event_slot(self.event_idx);
        let _ = ctx;
    }

    pub fn order_other_event(&mut self, ctx: &Ctx) {
        self.event_order = event_slot(self.event_idx);
        let _ = ctx;
    }

    pub fn set_action(&mut self, action: Action, ctx: &mut Ctx) -> bool {
        if self.action != action {
            if self.action != Action::None {
                self.action_status = ACTION_ENDING;
                let current = self.action;
                self.run_action(current, ctx);
            }
            self.action = action;
            self.action_status = ACTION_STARTING;
            self.run_action(action, ctx);
        }
        true
    }

    pub fn run_action(&mut self, action: Action, ctx: &mut Ctx) -> bool {
        match action {
            Action::Wait => self.wait_action(ctx),
            Action::Seek => self.seek_action(ctx),
            Action::Hind => self.hind_action(ctx),
            Action::Visit => self.visit_action(ctx),
            Action::None => true,
        }
    }

    pub fn wait_action(&mut self, ctx: &mut Ctx) -> bool {
        if self.action_status == ACTION_STARTING {
            self.state = STATE_WAIT;
            self.set_anm_status();
            self.action_status = ACTION_ONGOING;
        } else if self.action_status != ACTION_ENDING {
            self.chk_attention(ctx);
            self.clr_flag(FLAG_TURN_STOP | FLAG_STAND);
            let temp = match self.state {
                STATE_WAIT => self.wait01(),
                STATE_TALK01 => self.talk01(ctx),
                _ => false,
            };
            self.look_back(ctx);
            self.set_attention(temp);
        }
        true
    }

    pub fn seek_action(&mut self, ctx: &mut Ctx) -> bool {
        if self.action_status == ACTION_STARTING {
            if ctx.ev_flags.test(TMP_0002) {
                self.state = STATE_RUNAWAY;
                self.mk.init(MK_INIT_P0, MK_INIT_P1);
                let count = self.path_a.max_point();
                let roll = if count == 0 {
                    0
                } else {
                    (self.rnd_f(count as f32) as usize) % count
                };
                self.mk.set_rnd_path_pos(roll, &mut self.path_a);
            } else {
                self.state = STATE_CLIMB01;
            }
            self.set_anm_status();
            self.action_status = ACTION_ONGOING;
        } else if self.action_status != ACTION_ENDING {
            self.chk_attention(ctx);
            self.clr_flag(FLAG_TURN_STOP | FLAG_STAND);
            let mut temp = match self.state {
                STATE_CLIMB01 => self.climb01(ctx),
                STATE_DROP01 => self.drop01(ctx),
                STATE_RUNAWAY => self.runaway(ctx),
                STATE_JUMP => self.jump(),
                STATE_JITANDA01 => self.jitanda01(),
                STATE_JITANDA02 => self.jitanda02(ctx),
                STATE_DEMO01 => self.demo01(ctx),
                STATE_DEMO02 => self.demo02(ctx),
                STATE_DEMO03 => self.demo03(ctx),
                STATE_TALK02 => self.talk02(ctx),
                STATE_13 => {
                    let _ = self.set_action(Action::Hind, ctx);
                    false
                }
                _ => false,
            };
            self.look_back(ctx);
            if self.chk_flag(FLAG_AIR) {
                temp = true;
            }
            self.set_attention(temp);
        }
        true
    }

    pub fn hind_action(&mut self, ctx: &mut Ctx) -> bool {
        if self.action_status == ACTION_STARTING {
            self.action_status = ACTION_ONGOING;
        } else if self.action_status != ACTION_ENDING {
            self.clr_flag(FLAG_TURN_STOP | FLAG_STAND);
            if self.ty == TYPE_MINIGAME
                && ctx.ev_flags.test(TMP_0040)
                && !ctx.ev_flags.test(TMP_0020)
            {
                if self.set_action(Action::Seek, ctx) {
                    self.clr_flag(FLAG_NO_MOVE);
                    self.status |= STTS_SHOWMAP;
                }
            }
            if (self.ty == TYPE_NORMAL || self.ty == TYPE_MINIGAME) && self.check_demo_start(ctx) {
                let temp = self.demo_proc(ctx);
                self.look_back(ctx);
                self.set_attention(temp);
            }
        }
        true
    }

    pub fn visit_action(&mut self, ctx: &mut Ctx) -> bool {
        if self.action_status == ACTION_STARTING {
            if ctx.ev_flags.test(TMP_0040) {
                self.state = STATE_JITANDA02;
                self.set_anm(7);
                self.stts_weight = STTS_WEIGHT_FE;
            } else {
                self.state = STATE_VISIT;
                self.set_anm_status();
                self.stts_weight = STTS_INIT_A0;
            }
            self.action_status = ACTION_ONGOING;
            self.visit_mode = VISIT_START;
        } else if self.action_status != ACTION_ENDING {
            self.chk_attention(ctx);
            self.clr_flag(FLAG_TURN_STOP | FLAG_STAND);
            let temp = match self.state {
                STATE_VISIT => self.visit01(ctx),
                STATE_JITANDA02 => self.jitanda02(ctx),
                STATE_TALK01 => self.talk01(ctx),
                STATE_TALK02 => self.talk02(ctx),
                STATE_DEMO01 => self.demo01(ctx),
                STATE_DEMO02 => self.demo02(ctx),
                STATE_DEMO03 => self.demo03(ctx),
                _ => false,
            };
            self.look_back(ctx);
            self.set_attention(temp);
        }
        true
    }

    pub fn init(&mut self, actor: &mut Actor, ctx: &mut Ctx) -> bool {
        self.attn_flags = ATTN_SPEAK_TALK;
        self.attn_dist_talk = ATTN_DIST_TALK;
        self.attn_dist_speak = ATTN_DIST_SPEAK;
        self.gravity = GRAVITY;
        self.stts_weight = STTS_INIT_A0;
        self.cyl_radius = CYL_RADIUS_SRC;
        self.cyl_height = CYL_HEIGHT_SRC;
        self.map_set = MAP_SET_ID;
        self.max_fall = MAX_FALL;
        self.status |= STTS_SHOWMAP;
        match self.ty {
            TYPE_NORMAL => {
                if ctx.ev_flags.test(TMP_0040) && !ctx.ev_flags.test(TMP_0020) {
                    if self.set_action(Action::Hind, ctx) {
                        self.set_flag(FLAG_NO_MOVE);
                        self.status &= !STTS_SHOWMAP;
                    }
                } else if self.set_action(Action::Visit, ctx) && ctx.ev_flags.test(TMP_0580) {
                    self.status |= STTS_UNK4000;
                }
                self.path_a.set_inf(self.get_path(), actor.room, 1);
                if !self.path_a.get_path() {
                    return false;
                }
                let next = self.path_a.next_path(actor.room);
                self.path_b.set_inf_drct(next);
                if !self.path_b.get_path() {
                    return false;
                }
                let next = self.path_b.next_path(actor.room);
                self.path_c.set_inf_drct(next);
                if !self.path_c.get_path() {
                    return false;
                }
                for path in [
                    &mut self.path_a,
                    &mut self.path_b,
                    &mut self.path_c,
                ] {
                    path.set_point(0, self.pos);
                }
            }
            TYPE_MINIGAME => {
                self.stts_weight = STTS_WEIGHT_FE;
                self.path_a.set_inf(self.get_path(), actor.room, 1);
                if !self.path_a.get_path() {
                    return false;
                }
                if ctx.ev_flags.test(TMP_0040) && !ctx.ev_flags.test(TMP_0020) {
                    let _ = self.set_action(Action::Seek, ctx);
                } else {
                    if self.set_action(Action::Hind, ctx) {
                        self.set_flag(FLAG_NO_MOVE);
                    }
                    self.status &= !STTS_SHOWMAP;
                }
                self.status |= STTS_UNK40;
                self.path_a.set_point(0, self.pos);
            }
            _ => {
                let _ = self.set_action(Action::Wait, ctx);
            }
        }
        self.attn_base_pos = self.pos;
        self.eye_pos = self.pos;
        self.mtrl_snd_id = 0;
        self.reverb = 0;
        true
    }

    pub fn execute(&mut self, ctx: &mut Ctx) {
        self.jnt_param = JNT_PARAM;
        self.play_tex_pattern_anm();
        self.play_morf();
        if self.anm_ended {
            match self.anm_idx {
                7 => {
                    if self.rnd() < JUMP_ANM_CHANCE {
                        self.set_anm(8);
                    }
                }
                8 => self.set_anm(7),
                _ => {}
            }
        }
        self.check_order(ctx);
        let action = self.action;
        self.run_action(action, ctx);
        self.event_order(ctx);
        if !self.chk_flag(FLAG_TURN_PATH) {
            self.shape_yaw = self.yaw;
        }
        self.clr_flag(FLAG_TURN_PATH);
        self.mtrl_snd_id = 0;
        if !self.chk_flag(FLAG_NO_MOVE) {
            let (next_speed, speed_f) =
                stream::calc_speed(self.speed_f, self.yaw, self.speed.y, self.gravity, self.max_fall);
            self.speed = next_speed;
            self.speed_f = speed_f;
            self.old = self.pos;
            self.pos += self.speed;
            match ctx.ground_h {
                Some(height) if self.pos.y <= height => {
                    self.pos.y = height;
                    self.speed.y = 0.0;
                    self.grounded = true;
                    self.clr_flag(FLAG_AIR);
                }
                _ => {
                    self.grounded = false;
                    self.set_flag(FLAG_AIR);
                }
            }
        } else {
            self.set_flag(FLAG_AIR);
        }
        if self.chk_flag(FLAG_NO_MOVE) {
            self.cyl_on = false;
        } else {
            self.cyl_on = true;
            self.set_collision();
        }
    }
}

#[derive(SystemParam)]
pub struct World<'w> {
    pub stories: ResMut<'w, StoryBits>,
    pub ev_flags: ResMut<'w, EventFlags>,
    pub events: ResMut<'w, SimEvents>,
    pub list: Res<'w, EventList>,
    pub active: ResMut<'w, ActiveEvent>,
    pub equipped: Res<'w, Equipped>,
    pub collect: Res<'w, Collect>,
    pub collision: Option<Res<'w, Collision>>,
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
    pub do_intent: Res<'w, super::intent::DoIntent>,
    pub lock: Res<'w, super::lock::Lock>,
}

pub fn control(
    mut world: World,
    mut msg: Msg,
    mut actors: Query<(Entity, &mut Actor, &mut Ivan)>,
    mut others: Query<(Entity, &mut Actor), Without<Ivan>>,
    mut commands: Commands,
) {
    let player = msg.players.iter().next().copied();
    let clear_count = msg.text_flags.clear_count;
    let event_mode = msg.info.event_run_check();
    let press = msg.do_intent.0;
    let lock_on = msg.lock.target.is_some();
    let lock_target = msg.lock.target;
    let archive = msg
        .archive
        .as_deref()
        .map(|held| held as &dyn MsgArchive);
    for (entity, mut actor, mut ivan) in &mut actors {
        let actor = &mut *actor;
        let ivan = &mut *ivan;
        if !ivan.entered {
            ivan.entered = true;
            ivan.pos = actor.pos;
            ivan.old = actor.pos;
            ivan.yaw = actor.angle[1];
            ivan.shape_yaw = actor.angle[1];
            ivan.home = actor.pos;
            ivan.home_yaw = actor.angle[1];
        }
        let link = player.map(|link| link.pos);
        let link_shape_yaw = player.map_or(0, |link| walk::to_s16(link.face));
        let mut text = Text {
            procs: &mut msg.procs,
            info: &mut msg.info,
            flags: &mut msg.flags,
            one: &mut msg.one,
            two: &mut msg.two,
            clear_count,
            archive,
        };
        let cut = ActiveEvent::staff_cut(&world.active, &world.list, ivan.staff_name());
        let ground_h = ground::surface(world.collision.as_deref(), ivan.pos);
        let link_hit = match link {
            Some(pos) => abs_xz(pos - ivan.pos) <= ivan.cyl_radius + LINK_HIT_PAD,
            None => false,
        };
        ivan.link_hit = link_hit;
        let mut ctx = Ctx {
            entity,
            player,
            link,
            link_shape_yaw,
            lock_on,
            lock_target: lock_target == Some(entity),
            action_target: lock_target == Some(entity) && lock_on,
            talk_command: press && event_mode,
            demo_accepted: false,
            front_roll_crash: false,
            ground_h,
            event_idx: 0,
            cut,
            event_mode,
            stories: &mut world.stories,
            ev_flags: &mut world.ev_flags,
            list: &world.list,
            active: &mut world.active,
            events: &mut world.events,
            equipped: &world.equipped,
            collect: &world.collect,
            text: &mut text,
            order_name: None,
            present_item: None,
            shock: None,
            delete_um2: false,
            talk_partner_reset: false,
            end_cut: false,
        };
        if !ivan.entered {
            ivan.entered = true;
            ivan.init(actor, &mut ctx);
        } else {
            ivan.execute(&mut ctx);
        }
        let end_cut = ctx.end_cut;
        let talk_partner_reset = ctx.talk_partner_reset;
        let order_name = ctx.order_name;
        let present_item = ctx.present_item;
        let delete_um2 = ctx.delete_um2;
        if talk_partner_reset {
            ctx.reset_event();
        }
        let order_index = ivan.event_order.take();
        let staff = ivan.staff_name();
        drop(ctx);
        if end_cut {
            world.active.end_staff_cut(&world.list, &mut world.ev_flags, staff);
        }
        if let Some((pos, item)) = present_item {
            let room = actor.room;
            let _ = commands.spawn((
                Actor {
                    name: super::event::PRESENT_ACTOR.to_owned(),
                    parameters: u32::from(item),
                    pos,
                    angle: [0; 3],
                    set_id: u16::MAX,
                    room,
                    ..Default::default()
                },
                PresentItem { item, hidden: false },
            ));
        }
        if delete_um2 {
            for (_, mut other) in &mut others {
                if other.name == UM2_ACTOR {
                    other.parameters |= PRM_UM2_HIDE;
                }
            }
        }
        let order_index = order_index.or_else(|| {
            order_name.and_then(|name| world.list.event_index(name))
        });
        if let Some(index) = order_index
            && world.active.stage_of(index).is_none()
        {
            let _ = world.active.order(&world.list, index, entity);
        }
        actor.pos = ivan.pos;
        actor.angle = [actor.angle[0], ivan.yaw, actor.angle[2]];
    }
}
