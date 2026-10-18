use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::flags::OrderFlags;
use super::intent::DoIntent;
use super::player::Player;
use super::walk;
use crate::assets::MsgArchives;
use crate::text::driver::TextFlags;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgGameInfo, MsgProcs,
    fop_msgm_message_set_actor,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_BOX_OPENING, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED,
    STATUS_TYPING,
};

pub const NPC_HI1_ACTORS: [&str; 1] = ["Hi1"];

pub fn is_npc_hi1(name: &str) -> bool {
    NPC_HI1_ACTORS.contains(&name)
}

const STAFF_HI1: &str = "Hi1"; // <- mEventCut.setActorInfo2("Hi1", this), :134
const EVENT_DUMMY: &str = "dummy"; // <- l_evn_tbl[0], :67

// daNpc_Hi1_HIO_c::hio_prm_c a_prm_tbl - d_a_npc_hi1.cpp:47-60
const HIO_MAX_HEAD_X: i16 = 0x0000; // <- :48
const HIO_MAX_HEAD_Y: i16 = 0x1FFE; // <- :49
const HIO_MIN_HEAD_X: i16 = -0x0A00; // <- 0xF600, :50
const HIO_MIN_HEAD_Y: i16 = -0x1FFE; // <- 0xE002, :51
const HIO_MAX_BACKBONE_X: i16 = 0x0000; // <- :52
const HIO_MAX_BACKBONE_Y: i16 = 0x0744; // <- :53
const HIO_MIN_BACKBONE_X: i16 = -0x0626; // <- 0xF9DA, :54
const HIO_MIN_BACKBONE_Y: i16 = -0x0744; // <- 0xF8BC, :55
const HIO_MAX_TURN_STEP: i16 = 0x0258; // <- :56
const HIO_FIELD_12: i16 = 0x0000; // <- lookBack maxVel, :520
const HIO_ATTENTION_Y_OFFSET: f32 = 270.0; // <- :58, setAttention :536

// daNpc_Hi1_c::anm_prm_c - d_a_npc_hi1.h:13-19
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnmPrm {
    pub index: i8,
    pub btp: i8,
    pub morf: f32,
    pub speed: f32,
    pub loop_mode: i32,
}

const ANM_NONE: i8 = -1; // <- setAnm_anm temp < 0, :296
const ANM_TABLE_SET: [AnmPrm; 3] = [
    AnmPrm {
        index: -1,
        btp: -1,
        morf: 0.0,
        speed: 0.0,
        loop_mode: -1,
    }, // <- :329
    AnmPrm {
        index: 0,
        btp: 0,
        morf: 8.0,
        speed: 1.0,
        loop_mode: 2,
    }, // <- :330
    AnmPrm {
        index: -1,
        btp: -1,
        morf: 0.0,
        speed: 0.0,
        loop_mode: -1,
    }, // <- :331
]; // <- setAnm, :328-332
const ANM_TABLE_NUM: [AnmPrm; 2] = [
    AnmPrm {
        index: 0,
        btp: 0,
        morf: 8.0,
        speed: 1.0,
        loop_mode: 2,
    }, // <- :316
    AnmPrm {
        index: 1,
        btp: 0,
        morf: 8.0,
        speed: 1.0,
        loop_mode: 2,
    }, // <- :317
]; // <- setAnm_NUM, :315-318
const ANM_TABLE_ATR: [AnmPrm; 2] = [
    AnmPrm {
        index: 0,
        btp: 0,
        morf: 8.0,
        speed: 1.0,
        loop_mode: 2,
    }, // <- :364
    AnmPrm {
        index: 1,
        btp: 0,
        morf: 8.0,
        speed: 1.0,
        loop_mode: 2,
    }, // <- :365
]; // <- setAnm_ATR, :363-366
const ANM_NUM_WAIT: usize = 0; // <- setAnm_NUM(0, 1), :720
const ANM_ATR_MAX: u8 = 2; // <- chngAnmAtr param_1 > 2, :347
const ANM_TAG_NONE: u8 = 0xFF; // <- :440, :441
const ANM_ATR_NONE: u8 = 0xFF; // <- :440, :608
const BTP_FRAME_MAX: u8 = 0xFF; // <- getFrameMax() & 0xff stand-in, :270, :286
// play_btp_anm - :277
const BTP_TIMER_LO: i16 = 0x3C; // <- cLib_getRndValue(0x3c, 0x5a)
const BTP_TIMER_RANGE: i16 = 0x1E; // <- cM_rndF(0x5a)

// daNpc_Hi1_c state field_0x7C2 - setStt / wait_action1 :646, :740
const STT_ANM_NONE: i8 = 0; // <- :647
const STT_WAIT: i8 = 1; // <- :649
const STT_TALK: i8 = 2; // <- :653
// order field_0x7C1 - eventOrder / checkOrder :423, :438
const ORDER_NONE: i8 = 0; // <- :439, :444
const ORDER_SPEAK: i8 = 1; // <- :426
const ORDER_TALK: i8 = 2; // <- :429
const ORDER_EVENT: i8 = 3; // <- :430
const EVT_CND_CANTALK: u16 = 0x0001; // <- dEvtCnd_CANTALK_e, :424
const EVENT_IDX_SPEAK: usize = 0x1EA; // <- fopAcM_orderSpeakEvent msgNo, f_op_actor_mng.cpp:677
// look state field_0x7C4 - lookBack :496
const LOOK_NONE: i8 = 0; // <- :497 absent case
const LOOK_PLAYER: i8 = 1; // <- :497
const LOOK_ATT_POS: i8 = 2; // <- :502
const LOOK_ANGLE: i8 = 3; // <- :506
const LOOK_PID: i8 = 4; // <- :509

// wait_1 - :665-703
const HOME_TURN_SCALE: i16 = 4; // <- cLib_addCalcAngleS scale, :668
const HOME_TURN_MAX: i16 = 0x800; // <- maxStep, :668
const HOME_TURN_MIN: i16 = 0x80; // <- minStep, :668
const TIMER2_LO: i16 = 0xF; // <- cLib_getRndValue(0xf, 0x1e), :688, :721
const TIMER2_RANGE: i16 = 0x1E; // <- cM_rndF(0x1e)
const ATTN_DIST: f32 = 200.0; // <- dNpc_chkAttn param_3, :696
const ATTN_DIST_EXT: f32 = 50.0; // <- param_4, :696
const ATTN_DEG: f32 = 55.0; // <- param_5, :696
const ATTN_DEG_MIN: f32 = 0.0; // <- cLib_minLimit, d_npc.cpp:936
const ATTN_DEG_MAX: f32 = 359.0; // <- cLib_maxLimit, d_npc.cpp:936
const DEG2S: f32 = 0x8000 as f32 / 180.0; // <- cM_deg2s, SSystem/SComponent/c_angle.h:26

const NO_ITEM: u8 = 0xFF; // <- mItemNo = 0xff, :459, :717
const TYPE_NORMAL: i8 = 0; // <- decideType field_0x7C6 = 0, :550

// dNpc_EventCut_c action_table - d_npc_event_cut.inc:11-19
const CUT_WAIT: usize = 0; // <- :13
const CUT_TURN_TO_ACTOR: usize = 1; // <- :14
const CUT_MOVE_TO_ACTOR: usize = 2; // <- :15
const CUT_TURN_TO_POS: usize = 3; // <- :16
const CUT_MOVE_TO_POS: usize = 4; // <- :17
const CUT_TALK_MSG: usize = 5; // <- :18
const CUT_CONTINUE_TALK: usize = 6; // <- :19
const CUT_ACTIONS: [&str; 7] = [
    "WAIT",
    "TURN_TO_ACTOR",
    "MOVE_TO_ACTOR",
    "TURN_TO_POS",
    "MOVE_TO_POS",
    "TALK_MSG",
    "CONTINUE_TALK",
]; // <- :19

const CUT_NO_TARGET: i32 = -1; // <- dComIfGp_evmng_getMyStaffId == -1, :27
const CUT_TURN_TYPE_DEFAULT: i32 = 1; // <- cutTurnToPosStart :402
const CUT_TIMER_DEFAULT: i32 = 1; // <- cutTurnToPosStart :411
const CUT_TIMER_WAIT_DEFAULT: i32 = 0; // <- cutWaitStart :91
const CUT_SPEED_DEFAULT: f32 = 2.0; // <- cutMoveToPosStart :510
const CUT_STABLE_ANGLE: f32 = 5376.0; // <- std::fabsf(temp2) < 5376.0f, :466
const CUT_TURN_SETTLE_ADD: i32 = 4; // <- cLib_addCalcAngleL min_step, d_npc.cpp:934-937
const CUT_MOVE_SCALE: f32 = 0.1; // <- cLib_addCalc2 scale, :548
const CUT_MOVE_DONE: f32 = 5.0; // <- dist < 5.0f, :546
const CUT_TURN_BACKBONE_LOCK: u32 = 0; // <- mTurnType case 0, :457
const CUT_TURN_HEAD_LOCK: u32 = 2; // <- mTurnType case 2, :452
const CUT_TURN_DEFAULT: u32 = 3; // <- mTurnType default, :459

// _create fopAcM_setCullSizeBox - :947
const CULL_MIN: [f32; 3] = [-110.0, -20.0, -100.0]; // <- fopAcM_setCullSizeBox, d_a_npc_hi1.cpp:947
const CULL_MAX: [f32; 3] = [110.0, 280.0, 100.0]; // <- fopAcM_setCullSizeBox, d_a_npc_hi1.cpp:947

const JNT_SCALE: i16 = 4; // <- cLib_addCalcAngleL scale, d_npc.cpp:934, :940, :934
const JNT_ANG_MIN_STEP: i16 = 0x100; // <- cLib_addCalcAngleS min_step, d_npc.cpp:910
const PLAYER_EYE_OFFSET_Y: f32 = -20.0; // <- dNpc_playerEyePos(-20.0f), :498
const PLAYER_HEAD_TOP: f32 = 40.0; // <- d_a_player_main.cpp:11575

const NO_STATUS: u8 = 0xFF; // <- fopNpc_npc_c::talk u16 status = 0xFF, d_npc.cpp:665
const NO_EVENT: usize = usize::MAX; // <- dComIfGp_evmng_getEventIdx miss

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA]; // <- cM_rnd, c_math.cpp:178-180
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673]; // <- cM_rnd, c_math.cpp:178-180
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0]; // <- cM_rnd, c_math.cpp:181
const RND_SEED: i32 = 100; // <- cM_initRnd actor seeds, c_math.cpp:170-175

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = seed[0] as f32 / RND_SCALE[0]
        + seed[1] as f32 / RND_SCALE[1]
        + seed[2] as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

/// `cLib_getRndValue` (`c_lib.h:80-82`).
fn rnd_value(seed: &mut [i32; 3], min: i16, range: i16) -> i16 {
    min.wrapping_add((rnd(seed) * range as f32) as i16)
}

/// `cLib_calcTimer` (`c_lib.h:85-89`).
fn calc_timer<T: Copy + PartialEq + Default>(value: &mut T) -> T {
    let cur = *value;
    if cur != T::default() {
        *value = T::default();
    }
    cur
}

fn abs_xz(v: Vec3) -> f32 {
    (v.x * v.x + v.z * v.z).sqrt()
}

/// `cLib_targetAngleY` (`c_lib.cpp`).
fn target_angle_y(from: Vec3, to: Vec3) -> i16 {
    walk::to_s16((to.x - from.x).atan2(to.z - from.z))
}

/// `cLib_targetAngleX` (`c_lib.cpp:348-352`).
fn target_angle_x(from: Vec3, to: Vec3) -> i16 {
    let delta = to - from;
    walk::to_s16(delta.y.atan2(abs_xz(delta)))
}

/// `cM_atan2s`.
fn atan2s(y: f32, x: f32) -> i16 {
    walk::to_s16(y.atan2(x))
}

/// `cLib_addCalc2` (`c_lib.cpp:56-66`).
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

/// `cLib_addCalcAngleL` (`c_lib.cpp:205-231`).
fn add_calc_angle_l(value: i32, target: i32, scale: i32, max_step: i32, min_step: i32) -> i32 {
    let diff = target - value;
    if value == target {
        return value;
    }
    let mut step = diff / scale;
    if step > min_step || step < -min_step {
        if step > max_step {
            step = max_step;
        }
        if step < -max_step {
            step = -max_step;
        }
        value + step
    } else if diff >= 0 {
        let next = value + min_step;
        if target - next <= 0 { target } else { next }
    } else {
        let next = value - min_step;
        if target - next >= 0 { target } else { next }
    }
}

/// `cLib_addCalcAngleS` (`c_lib.cpp:190-203`).
fn add_calc_angle_s(value: i16, target: i16, scale: i16, max_step: i16, min_step: i16) -> i16 {
    walk::add_calc_angle_s(value, target, scale, max_step, min_step)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JntCtrl {
    pub angles: [[i16; 2]; 2],
    pub min_angles: [[i16; 2]; 2],
    pub max_angles: [[i16; 2]; 2],
    pub max_turn_step: [[i16; 2]; 2],
    pub trn: bool,
    pub head_lock: bool,
    pub backbone_lock: bool,
    pub field_0x2c: i32,
    pub field_0x2e: i32,
    pub field_0x30: i32,
    pub field_0x32: i32,
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
            field_0x2c: 0,
            field_0x2e: 0,
            field_0x30: 0,
            field_0x32: 0,
        }
    }

    pub fn set_trn(&mut self) {
        self.trn = true;
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

    /// `dNpc_JntCtrl_c::chkLim` (`d_npc.cpp:788-794`).
    fn chk_lim(&self, angle: i32, row: usize, axis: usize) -> i32 {
        let max = i32::from(self.max_angles[row][axis]);
        let min = i32::from(self.min_angles[row][axis]);
        angle.min(max).max(min)
    }

    /// `dNpc_JntCtrl_c::turn_fromBackbone2Head` (`d_npc.cpp:796-812`).
    fn turn_from_backbone2_head(&self, target: i16, head: &mut i16, backbone: &mut i16) {
        *backbone = 0;
        if !self.backbone_lock {
            *backbone = self.chk_lim(i32::from(target), 1, 1) as i16;
            if self.field_0x32 != 0 && *backbone < 0 {
                *backbone = 0;
            }
        }
        *head = 0;
        if !self.head_lock {
            *head = self.chk_lim(i32::from(target - *backbone), 0, 1) as i16;
        }
    }

    /// `dNpc_JntCtrl_c::turn_fromHead2Backbone` (`d_npc.cpp:814-827`).
    fn turn_from_head2_backbone(&self, target: i16, head: &mut i16, backbone: &mut i16) {
        *head = 0;
        if !self.head_lock {
            *head = self.chk_lim(i32::from(target) - self.field_0x32, 0, 1) as i16;
        }
        *backbone = 0;
        if !self.backbone_lock {
            *backbone = self.chk_lim(i32::from(target - *head), 1, 1) as i16;
        }
    }

    /// `dNpc_JntCtrl_c::follow_current` (`d_npc.cpp:829-840`).
    fn follow_current(angle: &mut i16, diff: i16) -> i16 {
        let before = *angle;
        *angle = angle.wrapping_sub(diff);
        let mut ret = 0;
        if (before > 0 && *angle < 0) || (before < 0 && *angle > 0) {
            ret = -*angle;
            *angle = 0;
        }
        ret
    }

    /// `dNpc_JntCtrl_c::lookAtTarget_2` (`d_npc.cpp:841-928`).
    pub fn look_at_target_2(
        &mut self,
        out_y: &mut i16,
        dst: Option<Vec3>,
        src: Vec3,
        default_y: i16,
        max_vel: i16,
        free: bool,
    ) {
        let (target_y, mut target_x) = match dst {
            Some(point) => (target_angle_y(src, point), target_angle_x(src, point)),
            None => (default_y, 0),
        };
        let delta_y = i32::from(target_y) - i32::from(*out_y);
        let mut head = 0i16;
        let mut backbone = 0i16;
        if self.field_0x32 >= 0 {
            if delta_y >= self.field_0x32 || self.field_0x32 == 0 {
                self.turn_from_head2_backbone(delta_y as i16, &mut head, &mut backbone);
            } else {
                self.turn_from_backbone2_head(delta_y as i16, &mut backbone, &mut head);
            }
        } else if delta_y <= self.field_0x32 || self.field_0x32 == 0 {
            self.turn_from_head2_backbone(delta_y as i16, &mut head, &mut backbone);
        } else {
            self.turn_from_backbone2_head(delta_y as i16, &mut backbone, &mut head);
        }
        self.field_0x2e = i32::from(backbone);
        self.field_0x32 = i32::from(head);
        let step = i32::from(self.max_turn_step[0][0]);
        self.angles[0][1] = add_calc_angle_l(
            i32::from(self.angles[0][1]),
            self.field_0x2e,
            JNT_SCALE as i32,
            step,
            CUT_TURN_SETTLE_ADD,
        ) as i16;
        self.angles[1][1] = add_calc_angle_l(
            i32::from(self.angles[1][1]),
            self.field_0x32,
            JNT_SCALE as i32,
            step,
            CUT_TURN_SETTLE_ADD,
        ) as i16;
        if self.trn && !free {
            let before = *out_y;
            *out_y = add_calc_angle_s(*out_y, target_y, JNT_SCALE, max_vel, JNT_ANG_MIN_STEP);
            let moved = out_y.wrapping_sub(before);
            self.trn = moved != 0;
            if !self.backbone_lock {
                let rest = Self::follow_current(&mut self.angles[1][1], moved);
                let rest = rest;
                if !self.head_lock {
                    Self::follow_current(&mut self.angles[0][1], rest);
                }
            } else if !self.head_lock {
                Self::follow_current(&mut self.angles[0][1], moved);
            }
        } else {
            let total = self.field_0x2e + self.field_0x32;
            self.trn = if delta_y >= 0 {
                delta_y > total
            } else {
                delta_y < total
            };
        }
        let raw_x = target_x;
        target_x -= self.chk_lim(i32::from(raw_x), 0, 0) as i16;
        self.field_0x2c = self.chk_lim(i32::from(raw_x), 0, 0);
        self.field_0x30 = self.chk_lim(i32::from(target_x), 1, 0);
        let pitch_head = self.field_0x2c;
        let pitch_backbone = self.field_0x30;
        self.angles[0][0] = add_calc_angle_l(
            i32::from(self.angles[0][0]),
            pitch_head,
            JNT_SCALE as i32,
            step,
            4,
        ) as i16;
        self.angles[1][0] = add_calc_angle_l(
            i32::from(self.angles[1][0]),
            pitch_backbone,
            JNT_SCALE as i32,
            step,
            4,
        ) as i16;
    }
}

/// `dNpc_EventCut_c` state - d_npc.h:120-137, d_npc_event_cut.inc.
#[derive(Debug, Clone, PartialEq)]
pub struct EventCut {
    pub timer: i32,
    pub turn_speed: i16,
    pub add_angle: i16,
    pub del_distance: f32,
    pub attention: bool,
    pub no_turn: bool,
    pub turn_type: u32,
    pub set_id: u16,
    pub offset_pos: Vec3,
    pub speed: f32,
    pub pos: Vec3,
    pub target_angle: i16,
    pub field_0x44: f32,
    pub actor_name: Option<String>,
    pub field_0x52: u8,
}

impl EventCut {
    pub fn create() -> Self {
        Self {
            timer: 0,
            turn_speed: 0,
            add_angle: 0,
            del_distance: 0.0,
            attention: false,
            no_turn: false,
            turn_type: 0,
            set_id: 0,
            offset_pos: Vec3::ZERO,
            speed: 0.0,
            pos: Vec3::ZERO,
            target_angle: 0,
            field_0x44: 0.0,
            actor_name: None,
            field_0x52: 0,
        }
    }
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct NpcHi1 {
    pub entered: bool,
    pub prev_do: bool,
    pub anm_no: i8,
    pub order: i8,
    pub state: i8,
    pub return_state: i8,
    pub look_state: i8,
    pub action_status: i8,
    pub anm_atr: u8,
    pub anm_tag: u8,
    pub anm_atr_count: i8,
    pub attn_ended: bool,
    pub return_home: bool,
    pub talk_order: bool,
    pub head_only_follow: bool,
    pub has_attention: bool,
    pub init_done: bool,
    pub anm_type: i8,
    pub item_no: u8,
    pub timer2: i32,
    pub target_y: i16,
    pub event_idx: [usize; 1],
    pub other_slot: i16,
    pub curr_msg_no: u32,
    pub end_msg_no: u32,
    pub msg_id: u32,
    pub msg_bound: bool,
    pub eye_pos: Vec3,
    pub attn_base_pos: Vec3,
    pub attn_pos: Vec3,
    pub attn_look: Vec3,
    pub base_pos: Vec3,
    pub home_angle: [i16; 3],
    pub prev_angle: [i16; 3],
    pub btp_idx: i8,
    pub btp_frame: u8,
    pub btk_frame: u8,
    pub timer1: i16,
    pub seed: [i32; 3],
    pub cut: EventCut,
    pub jnt: JntCtrl,
}

impl NpcHi1 {
    pub fn spawn(parameters: u32) -> Self {
        let param = (parameters & 0xFF) as i8;
        Self {
            entered: false,
            prev_do: false,
            anm_no: 2,
            order: ORDER_NONE,
            state: STT_ANM_NONE,
            return_state: STT_ANM_NONE,
            look_state: LOOK_NONE,
            action_status: 0,
            anm_atr: ANM_ATR_NONE,
            anm_tag: ANM_TAG_NONE,
            anm_atr_count: 0,
            attn_ended: false,
            return_home: false,
            talk_order: false,
            head_only_follow: false,
            has_attention: false,
            init_done: false,
            anm_type: if param == 0 { TYPE_NORMAL } else { param },
            item_no: NO_ITEM,
            timer2: 0,
            target_y: 0,
            event_idx: [NO_EVENT],
            other_slot: 0,
            curr_msg_no: 0,
            end_msg_no: u32::MAX,
            msg_id: FPC_M_ERROR_PROCESS_ID,
            msg_bound: false,
            eye_pos: Vec3::ZERO,
            attn_base_pos: Vec3::ZERO,
            attn_pos: Vec3::ZERO,
            attn_look: Vec3::ZERO,
            base_pos: Vec3::ZERO,
            home_angle: [0; 3],
            prev_angle: [0; 3],
            btp_idx: 0,
            btp_frame: 0,
            btk_frame: 0,
            timer1: 0,
            seed: [RND_SEED; 3],
            cut: EventCut::create(),
            jnt: JntCtrl::create(),
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    /// `daNpc_Hi1_c::decideType` (`:544-558`): only type 0 exists in retail, so
    /// `_create`'s `if(!decideType(...)) return cPhs_ERROR_e` never refuses.
    fn create_refused(&self) -> bool {
        self.anm_type != TYPE_NORMAL
    }

    /// `daNpc_Hi1_c::setBtp` (`:225-238`).
    fn set_btp(&mut self, idx: i8, _modify: bool) -> bool {
        if idx < 0 {
            return false;
        }
        self.btp_idx = idx;
        self.btp_frame = 0;
        self.timer1 = 0;
        true
    }

    /// `daNpc_Hi1_c::setBtk` (`:241-253`).
    fn set_btk(&mut self, idx: i8, _modify: bool) -> bool {
        if idx < 0 {
            return false;
        }
        self.btk_frame = 0;
        true
    }

    /// `daNpc_Hi1_c::init_texPttrnAnm` (`:256-266`).
    fn init_tex_pttrn_anm(&mut self, idx: i8, modify: bool) -> bool {
        if !self.set_btp(idx, modify) {
            return false;
        }
        self.set_btk(0, modify)
    }

    /// `daNpc_Hi1_c::play_btp_anm` (`:269-282`).
    fn play_btp_anm(&mut self) {
        let frame = BTP_FRAME_MAX;
        if self.btp_idx != 0 || calc_timer(&mut self.timer1) == 0 {
            self.btp_frame = self.btp_frame.wrapping_add(1);
            if self.btp_frame >= frame {
                if self.btp_idx != 0 {
                    self.btp_frame = frame;
                } else {
                    self.timer1 = rnd_value(&mut self.seed, BTP_TIMER_LO, BTP_TIMER_RANGE);
                    self.btp_frame = 0;
                }
            }
        }
    }

    /// `daNpc_Hi1_c::play_btk_anm` (`:285-291`).
    fn play_btk_anm(&mut self) {
        let frame = BTP_FRAME_MAX;
        self.btk_frame = self.btk_frame.wrapping_add(1);
        if self.btk_frame >= frame {
            self.btk_frame = frame;
        }
    }

    /// `daNpc_Hi1_c::play_animation` (`:174-187`), minus the `mpMorf->getFrame()`
    /// end-frame test (BCK frame counts are unavailable).
    fn play_animation(&mut self) {
        self.play_btp_anm();
        self.play_btk_anm();
    }

    /// `daNpc_Hi1_c::setAnm_anm` (`:294-311`).
    fn set_anm_anm(&mut self, prm: &AnmPrm) {
        if prm.index < 0 || self.anm_no == prm.index {
            return;
        }
        self.anm_no = prm.index;
    }

    /// `daNpc_Hi1_c::setAnm_NUM` (`:314-324`).
    fn set_anm_num(&mut self, idx: usize, modify: i32) {
        let prm = ANM_TABLE_NUM[idx.min(ANM_TABLE_NUM.len() - 1)];
        if modify != 0 {
            self.init_tex_pttrn_anm(prm.btp, true);
        }
        self.set_anm_anm(&prm);
    }

    /// `daNpc_Hi1_c::setAnm` (`:327-335`).
    fn set_anm(&mut self) {
        let row = usize::try_from(self.state)
            .unwrap_or(0)
            .min(ANM_TABLE_SET.len() - 1);
        let prm = ANM_TABLE_SET[row];
        self.init_tex_pttrn_anm(prm.btp, true);
        self.set_anm_anm(&prm);
    }

    /// `daNpc_Hi1_c::setAnm_ATR` (`:362-369`).
    fn set_anm_atr(&mut self) {
        let row = usize::from(self.anm_atr).min(ANM_TABLE_ATR.len() - 1);
        let prm = ANM_TABLE_ATR[row];
        self.init_tex_pttrn_anm(prm.btp, true);
        self.set_anm_anm(&prm);
    }

    /// `daNpc_Hi1_c::chngAnmTag` (`:338-339`).
    fn chng_anm_tag(&mut self) {}

    /// `daNpc_Hi1_c::ctrlAnmTag` (`:342-343`).
    fn ctrl_anm_tag(&mut self) {}

    /// `daNpc_Hi1_c::chngAnmAtr` (`:346-352`).
    fn chng_anm_atr(&mut self, attr: u8) {
        if attr == self.anm_atr || attr > ANM_ATR_MAX {
            return;
        }
        self.anm_atr = attr;
        self.set_anm_atr();
    }

    /// `daNpc_Hi1_c::ctrlAnmAtr` (`:355-359`).
    fn ctrl_anm_atr(&mut self) {
        if self.anm_atr == 2 {
            return;
        }
    }

    /// `daNpc_Hi1_c::anmAtr` (`:372-395`).
    fn anm_atr(&mut self, status: u16, ctx: &mut Ctx<'_>) {
        match status {
            s if s == u16::from(STATUS_TYPING) => {
                if self.anm_atr_count == 0 {
                    let attr = ctx.text.mesg_anime();
                    self.chng_anm_atr(attr);
                    self.anm_atr_count += 1;
                }
                let tag = ctx.text.mesg_anime_tag();
                if tag != 0xFF && tag != self.anm_tag {
                    ctx.text.clear_mesg_anime_tag();
                    self.anm_tag = tag;
                    self.chng_anm_tag();
                }
            }
            s if s == u16::from(STATUS_MSG_DISPLAYED) => {
                self.anm_atr_count = 0;
            }
            _ => {}
        }
        self.ctrl_anm_atr();
        self.ctrl_anm_tag();
    }

    /// `daNpc_Hi1_c::next_msgStatus` (`:398-400`).
    fn next_msg_status(&self) -> u8 {
        crate::text::STATUS_MSG_ENDS
    }

    /// `daNpc_Hi1_c::getMsg_HI1_0` (`:403-405`) and `getMsg` (`:408-419`).
    fn get_msg(&self) -> u32 {
        if self.anm_type == TYPE_NORMAL { 0 } else { 0 }
    }

    /// `fopNpc_npc_c::talk` (`d_npc.cpp:664-697`).
    fn talk(&mut self, from_get: bool, ctx: &mut Ctx<'_>) -> u8 {
        let mut status = NO_STATUS;
        if self.msg_id == FPC_M_ERROR_PROCESS_ID {
            if from_get {
                self.curr_msg_no = self.get_msg();
            }
            let no = self.curr_msg_no;
            let eye = self.eye_pos;
            self.msg_id = ctx.text.message_set_actor(no, eye);
            self.msg_bound = false;
        } else if self.msg_bound {
            status = ctx.text.status().unwrap_or(NO_STATUS);
            if status == STATUS_MSG_DISPLAYED {
                ctx.text.set_status(self.next_msg_status());
            } else if status == STATUS_BOX_CLOSED {
                ctx.text.set_status(STATUS_MSG_DESTROYED);
                self.msg_id = FPC_M_ERROR_PROCESS_ID;
            }
            self.anm_atr(u16::from(status), ctx);
        } else {
            self.msg_bound = ctx.text.procs.search_by_id(self.msg_id).is_some();
        }
        status
    }

    /// `daNpc_Hi1_c::chk_parts_notMov` (`:464-468`).
    fn chk_parts_not_mov(&self, angle: i16) -> bool {
        self.prev_angle[1] != self.jnt.angles[0][1]
            || self.prev_angle[2] != self.jnt.angles[1][1]
            || self.prev_angle[0] != angle
    }

    /// `daNpc_Hi1_c::chk_talk` (`:450-461`), minus the
    /// `dComIfGp_event_getPreItemNo()` read at `:453`.
    fn chk_talk(&mut self, ctx: &Ctx<'_>) -> bool {
        if ctx.talk_xy {
            return ctx.present_done;
        }
        self.item_no = NO_ITEM;
        true
    }

    /// `daNpc_Hi1_c::chkAttention` (`:524-532`), minus the `dAttention_c` queries.
    fn chk_attention(&self, attention: bool) -> bool {
        attention
    }

    /// `daNpc_Hi1_c::setAttention` (`:535-541`).
    fn set_attention(&mut self, param_1: bool, actor_pos: Vec3) {
        self.attn_pos = Vec3::new(
            actor_pos.x,
            actor_pos.y + HIO_ATTENTION_Y_OFFSET,
            actor_pos.z,
        );
        if !self.attn_ended && !param_1 {
            return;
        }
        self.eye_pos = Vec3::new(self.attn_look.x, self.attn_look.y, self.attn_look.z);
    }

    /// `daNpc_Hi1_c::lookBack` (`:483-521`).
    fn look_back(&mut self, actor: &mut Actor, ctx: &Ctx<'_>) {
        self.prev_angle[1] = self.jnt.angles[0][1];
        self.prev_angle[2] = self.jnt.angles[1][1];
        self.prev_angle[0] = actor.angle[1];
        let src = Vec3::new(actor.pos.x, self.eye_pos.y, actor.pos.z);
        let mut dst = None;
        let mut target_y = actor.angle[1];
        match self.look_state {
            LOOK_PLAYER => {
                self.attn_pos = ctx.player_eye.unwrap_or(Vec3::ZERO);
                dst = ctx.player_eye;
            }
            LOOK_ATT_POS => dst = Some(self.attn_pos),
            LOOK_ANGLE => target_y = self.target_y,
            LOOK_PID => {
                // mPId is never assigned (`:510`), so fopAcM_SearchByID(0) never
                // matches and dstPos stays NULL.
            }
            _ => {}
        }
        let free = self.head_only_follow;
        self.jnt
            .look_at_target_2(&mut actor.angle[1], dst, src, target_y, HIO_FIELD_12, free);
    }

    /// `daNpc_Hi1_c::setStt` (`:642-662`).
    fn set_stt(&mut self, stt: i8) {
        let temp = self.state;
        self.state = stt;
        match stt {
            STT_WAIT => self.order = ORDER_NONE,
            STT_TALK => {
                self.order = ORDER_NONE;
                self.anm_atr = ANM_ATR_NONE;
                self.anm_tag = ANM_TAG_NONE;
                self.anm_atr_count = 0;
                self.return_state = temp;
            }
            _ => {}
        }
        self.set_anm();
    }

    /// `mEventIdx[field_0x796]` (`:430`, `:438`, `:618`); `mEventIdx` holds one
    /// entry, so any other `field_0x796` reads out of range in retail.
    fn event_slot(&self) -> usize {
        self.event_idx
            .get(self.other_slot.max(0) as usize)
            .copied()
            .unwrap_or(NO_EVENT)
    }

    /// `daNpc_Hi1_c::eventOrder` (`:422-433`).
    fn event_order(&mut self, ctx: &mut Ctx<'_>) {
        if self.order == ORDER_SPEAK || self.order == ORDER_TALK {
            ctx.order_flags.set(EVT_CND_CANTALK);
            if self.order == ORDER_SPEAK {
                ctx.order_other(EVENT_IDX_SPEAK);
            }
        } else if self.order >= ORDER_EVENT {
            self.other_slot = i16::from(self.order) - i16::from(ORDER_EVENT);
            let slot = self.event_slot();
            ctx.order_other(slot);
        }
    }

    /// `daNpc_Hi1_c::checkOrder` (`:436-447`).
    fn check_order(&mut self, ctx: &Ctx<'_>) {
        if ctx.demo_command {
            let slot = self.event_slot();
            if ctx.start_check(slot) && self.order >= 3 {
                self.order = ORDER_NONE;
                self.anm_atr = ANM_ATR_NONE;
                self.anm_tag = ANM_TAG_NONE;
            }
        } else if ctx.talk_command && (self.order == ORDER_SPEAK || self.order == ORDER_TALK) {
            self.order = ORDER_NONE;
            self.talk_order = true;
        }
    }

    /// `daNpc_Hi1_c::wait_1` (`:665-703`).
    fn wait_1(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) -> bool {
        let mut timer = 0i32;
        if self.return_home {
            let want = self.home_angle[1];
            actor.angle[1] = add_calc_angle_s(
                actor.angle[1],
                want,
                HOME_TURN_SCALE,
                HOME_TURN_MAX,
                HOME_TURN_MIN,
            );
            timer = i32::from(want.wrapping_sub(actor.angle[1]));
        }
        if self.talk_order {
            if self.chk_talk(ctx) {
                self.set_stt(STT_TALK);
                self.look_state = LOOK_PLAYER;
                self.head_only_follow = false;
                self.return_home = false;
                self.jnt.set_trn();
            }
            return true;
        }
        self.order = ORDER_TALK;
        self.look_state = LOOK_NONE;
        self.head_only_follow = true;
        if timer == 0 {
            if self.has_attention {
                self.timer2 = rnd_value(&mut self.seed, TIMER2_LO, TIMER2_RANGE) as i32;
            }
            timer = calc_timer(&mut self.timer2);
            if timer != 0 {
                self.look_state = LOOK_PLAYER;
            }
            if self.chk_attn(actor, ctx) {
                return true;
            }
            self.look_state = LOOK_NONE;
            self.return_home = true;
        }
        true
    }

    /// `dNpc_chkAttn` (`d_npc.cpp:930-949`).
    fn chk_attn(&self, actor: &Actor, ctx: &Ctx<'_>) -> bool {
        let Some(player) = ctx.player_pos else {
            return false;
        };
        let angle = target_angle_y(actor.pos, player);
        let dist = abs_xz(actor.pos - player);
        let mut limit = ATTN_DIST;
        let angle_diff = angle.wrapping_sub(actor.angle[1]);
        let deg = ATTN_DEG.max(ATTN_DEG_MAX).min(ATTN_DEG_MIN);
        if self.look_state == LOOK_PLAYER {
            limit += ATTN_DIST_EXT;
        }
        dist <= limit && (i32::from(angle_diff).abs() as f32) <= deg * DEG2S
    }

    /// `daNpc_Hi1_c::talk_1` (`:706-726`).
    fn talk_1(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let ret = self.chk_parts_not_mov(ctx.angle[1]);
        self.talk(true, ctx);
        if !self.msg_bound {
            return true;
        }
        match ctx.text.status().unwrap_or(NO_STATUS) {
            s if s == STATUS_TYPING || s == STATUS_BOX_OPENING => {}
            STATUS_MSG_DESTROYED => {
                self.item_no = NO_ITEM;
                self.talk_order = false;
                let back = self.return_state;
                self.set_stt(back);
                self.set_anm_num(ANM_NUM_WAIT, 1);
                self.timer2 = rnd_value(&mut self.seed, TIMER2_LO, TIMER2_RANGE) as i32;
                self.end_event();
            }
            _ => {}
        }
        ret
    }

    /// `daNpc_Hi1_c::endEvent` (`:605-609`), minus `dComIfGp_event_reset`.
    fn end_event(&mut self) {
        self.anm_atr = ANM_ATR_NONE;
        self.anm_tag = ANM_TAG_NONE;
    }

    /// `daNpc_Hi1_c::isEventEntry` (`:612-614`) - `dComIfGp_evmng_getMyStaffId("Hi1")`.
    fn is_event_entry(&self, ctx: &Ctx<'_>) -> i32 {
        match ctx.active.staff_cut(ctx.list, STAFF_HI1) {
            Some(_) => 0,
            None => CUT_NO_TARGET,
        }
    }

    /// `daNpc_Hi1_c::event_proc` (`:617-625`).
    fn event_proc(&mut self, staff: i32, ctx: &mut Ctx<'_>) {
        let slot = self.event_slot();
        if ctx.end_check(slot) {
            self.end_event();
        } else if !self.cut_proc(ctx) {
            self.private_cut(staff, ctx);
        }
    }

    /// `daNpc_Hi1_c::privateCut` (`:561-602`): retail cuts the "DUMMY" staff
    /// immediately, both in the DEMO and the retail arm (`:597-598`).
    fn private_cut(&mut self, staff: i32, ctx: &mut Ctx<'_>) {
        if staff != CUT_NO_TARGET {
            ctx.end_staff_cut(STAFF_HI1);
        }
    }

    /// `daNpc_Hi1_c::wait_action1` (`:729-752`).
    fn wait_action1(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        match self.action_status {
            0 => {
                self.set_stt(STT_WAIT);
                self.return_home = true;
                self.action_status += 1;
            }
            1..=3 => {
                self.has_attention = self.chk_attention(ctx.attention);
                self.attn_ended = match self.state {
                    STT_WAIT => self.wait_1(actor, ctx),
                    STT_TALK => self.talk_1(ctx),
                    _ => false,
                };
            }
            _ => {}
        }
    }

    /// `daNpc_Hi1_c::createInit` (`:129-171`), minus `mStts`/`mCyl`, the
    /// `mObjAcch.CrrPos`/`tevStr` fetches, `mpMorf->setMorf` and `setMtx`.
    /// `init_HI1_0` (`:123-126`) runs `wait_action1` once with
    /// `field_0x7C7 == 0` through `set_action` (`:628-639`).
    pub fn create_init(&mut self, actor: &mut Actor, list: &EventList) {
        self.event_idx[0] = list.event_index(EVENT_DUMMY).unwrap_or(NO_EVENT);
        self.anm_no = 2;
        self.set_stt(STT_WAIT);
        self.return_home = true;
        self.action_status += 1;
        self.home_angle = actor.angle;
        self.base_pos = actor.pos;
        self.entered = true;
    }

    /// `dNpc_EventCut_c::cutProc` (`d_npc_event_cut.inc:10-82`).
    fn cut_proc(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let Some((cut, advance)) = ctx.active.staff_cut(ctx.list, STAFF_HI1) else {
            return false;
        };
        let Some(act) = CUT_ACTIONS
            .iter()
            .position(|name| ctx.list.cut_name(cut) == Some(*name))
        else {
            ctx.end_staff_cut(STAFF_HI1);
            return false;
        };
        if advance {
            self.cut.attention = false;
            match act {
                CUT_WAIT => self.cut_wait_start(cut, ctx),
                CUT_TURN_TO_ACTOR => self.cut_turn_to_actor_start(cut, ctx),
                CUT_MOVE_TO_ACTOR => self.cut_move_to_actor_start(cut, ctx),
                CUT_TURN_TO_POS => self.cut_turn_to_pos_start(cut, ctx),
                CUT_MOVE_TO_POS => self.cut_move_to_pos_start(cut, ctx),
                CUT_TALK_MSG => self.cut_talk_msg_start(cut, ctx),
                CUT_CONTINUE_TALK => self.cut_continue_talk_start(cut, ctx),
                _ => {}
            }
        }
        match act {
            CUT_WAIT => self.cut_wait_proc(ctx),
            CUT_TURN_TO_ACTOR => self.cut_turn_to_proc(actor_pos_of(ctx), ctx),
            CUT_MOVE_TO_ACTOR => self.cut_move_to_actor_proc(actor_pos_of(ctx), ctx),
            CUT_TURN_TO_POS => self.cut_turn_to_proc(actor_pos_of(ctx), ctx),
            CUT_MOVE_TO_POS => self.cut_move_to_pos_proc(actor_pos_of(ctx), ctx),
            CUT_TALK_MSG | CUT_CONTINUE_TALK => self.cut_talk_msg_proc(ctx),
            _ => {}
        }
        true
    }

    /// `dNpc_EventCut_c::cutWaitStart` (`d_npc_event_cut.inc:85-93`).
    fn cut_wait_start(&mut self, cut: usize, ctx: &Ctx<'_>) {
        self.cut.timer = ctx
            .list
            .cut_number(cut, "Timer")
            .unwrap_or(CUT_TIMER_WAIT_DEFAULT);
    }

    /// `dNpc_EventCut_c::cutWaitProc` (`d_npc_event_cut.inc:96-100`).
    fn cut_wait_proc(&mut self, ctx: &mut Ctx<'_>) {
        if calc_timer(&mut self.cut.timer) == 0 {
            ctx.end_staff_cut(STAFF_HI1);
        }
    }

    /// `dNpc_EventCut_c::cutTurnToActorStart` (`d_npc_event_cut.inc:103-171`).
    fn cut_turn_to_actor_start(&mut self, cut: usize, ctx: &Ctx<'_>) {
        self.cut.actor_name = ctx
            .list
            .cut_text(cut, "ActorName")
            .filter(|name| *name != "Player")
            .map(str::to_owned);
        self.cut.set_id = ctx.list.cut_number(cut, "SetId").unwrap_or(0).max(0) as u16;
        self.cut.offset_pos = ctx.list.cut_vec(cut, "OffsetPos").unwrap_or(Vec3::ZERO);
        self.cut.turn_type = ctx
            .list
            .cut_number(cut, "TurnType")
            .unwrap_or(CUT_TURN_TYPE_DEFAULT)
            .max(0) as u32;
        self.cut.timer = ctx
            .list
            .cut_number(cut, "Timer")
            .unwrap_or(CUT_TIMER_DEFAULT);
        self.cut.turn_speed = ctx.list.cut_number(cut, "TurnSpeed").unwrap_or(0) as i16;
        self.cut.add_angle = ctx.list.cut_number(cut, "AddAngle").unwrap_or(0) as i16;
        self.cut.del_distance = ctx.list.cut_float(cut, "DelDistance").unwrap_or(0.0);
        self.cut.no_turn = ctx.list.cut_number(cut, "NoTurn").is_some();
        self.cut.field_0x44 = 0.0;
    }

    /// `dNpc_EventCut_c::cutTurnToActorProc` / `cutTurnToPosProc`
    /// (`d_npc_event_cut.inc:174-238` / `:444-476`).
    fn cut_turn_to_proc(&mut self, pos: Vec3, ctx: &mut Ctx<'_>) {
        self.cut_goal(pos, ctx);
        let t_angle = target_angle_y(pos, self.cut.pos);
        match self.cut.turn_type {
            CUT_TURN_HEAD_LOCK => {
                self.jnt.set_trn();
                self.jnt.off_backbone_lock();
            }
            CUT_TURN_BACKBONE_LOCK => self.jnt.on_backbone_lock(),
            _ => self.jnt.off_backbone_lock(),
        }
        let angle = pos_angle(ctx);
        let temp2 = (t_angle.wrapping_sub(angle) as f32).abs();
        if self.cut.field_0x44 == temp2 && temp2 < CUT_STABLE_ANGLE {
            calc_timer(&mut self.cut.timer);
        }
        if self.cut.timer == 0 {
            self.cut.turn_speed = 0;
            ctx.end_staff_cut(STAFF_HI1);
        }
        self.cut.field_0x44 = (t_angle.wrapping_sub(pos_angle(ctx)) as f32).abs();
    }

    /// `dNpc_EventCut_c::cutMoveToActorStart` (`d_npc_event_cut.inc:241-299`).
    fn cut_move_to_actor_start(&mut self, cut: usize, ctx: &Ctx<'_>) {
        self.cut.actor_name = ctx
            .list
            .cut_text(cut, "ActorName")
            .filter(|name| *name != "Player")
            .map(str::to_owned);
        self.cut.no_turn = ctx.list.cut_number(cut, "NoTurn").is_some();
        self.cut.attention = ctx.list.cut_number(cut, "Attention").is_some();
        self.cut.offset_pos = ctx.list.cut_vec(cut, "OffsetPos").unwrap_or(Vec3::ZERO);
        self.cut.set_id = ctx.list.cut_number(cut, "SetId").unwrap_or(0).max(0) as u16;
        self.cut.speed = ctx
            .list
            .cut_float(cut, "Speed")
            .unwrap_or(CUT_SPEED_DEFAULT);
        self.cut.del_distance = ctx.list.cut_float(cut, "DelDistance").unwrap_or(0.0);
        self.cut.add_angle = ctx.list.cut_number(cut, "AddAngle").unwrap_or(0) as i16;
    }

    /// `dNpc_EventCut_c::cutMoveToActorProc` (`d_npc_event_cut.inc:302-343`).
    fn cut_move_to_actor_proc(&mut self, pos: Vec3, ctx: &mut Ctx<'_>) {
        self.cut_goal(pos, ctx);
        let goal = self.cut.pos;
        let mut temp = goal;
        let angle = if self.cut.add_angle != 0 {
            self.cut.target_angle.wrapping_add(self.cut.add_angle)
        } else {
            atan2s(goal.x - pos.x, goal.z - pos.z)
        };
        temp.x -= self.cut.del_distance * walk::from_s16(angle).sin();
        temp.z -= self.cut.del_distance * walk::from_s16(angle).cos();
        if self.cut.speed == 0.0 {
            ctx.actor_pos = temp;
            ctx.end_staff_cut(STAFF_HI1);
        }
        let dist = abs_xz(temp - ctx.actor_pos);
        add_calc2(&mut ctx.actor_pos.x, temp.x, CUT_MOVE_SCALE, self.cut.speed);
        add_calc2(&mut ctx.actor_pos.z, temp.z, CUT_MOVE_SCALE, self.cut.speed);
        if dist < CUT_MOVE_DONE {
            self.cut.speed = 0.0;
            ctx.end_staff_cut(STAFF_HI1);
        }
    }

    /// `dNpc_EventCut_c::cutTurnToPosStart` (`d_npc_event_cut.inc:391-441`).
    fn cut_turn_to_pos_start(&mut self, cut: usize, ctx: &Ctx<'_>) {
        self.cut.pos = ctx.list.cut_vec(cut, "Pos").unwrap_or(Vec3::ZERO);
        self.cut.turn_type = ctx
            .list
            .cut_number(cut, "TurnType")
            .unwrap_or(CUT_TURN_TYPE_DEFAULT)
            .max(0) as u32;
        self.cut.timer = ctx
            .list
            .cut_number(cut, "Timer")
            .unwrap_or(CUT_TIMER_DEFAULT);
        self.cut.turn_speed = ctx.list.cut_number(cut, "TurnSpeed").unwrap_or(0) as i16;
        self.cut.no_turn = ctx.list.cut_number(cut, "NoTurn").is_some();
        self.cut.add_angle = ctx.list.cut_number(cut, "AddAngle").unwrap_or(0) as i16;
        self.cut.field_0x44 = 0.0;
    }

    /// `dNpc_EventCut_c::cutMoveToPosStart` (`d_npc_event_cut.inc:479-528`).
    fn cut_move_to_pos_start(&mut self, cut: usize, ctx: &Ctx<'_>) {
        self.cut.no_turn = ctx.list.cut_number(cut, "NoTurn").is_some();
        self.cut.pos = ctx.list.cut_vec(cut, "Pos").unwrap_or(Vec3::ZERO);
        self.cut.attention = ctx.list.cut_number(cut, "Attention").is_some();
        self.cut.speed = ctx
            .list
            .cut_float(cut, "Speed")
            .unwrap_or(CUT_SPEED_DEFAULT);
        self.cut.del_distance = ctx.list.cut_float(cut, "DelDistance").unwrap_or(0.0);
        self.cut.add_angle = ctx.list.cut_number(cut, "AddAngle").unwrap_or(0) as i16;
    }

    /// `dNpc_EventCut_c::cutMoveToPosProc` (`d_npc_event_cut.inc:531-554`).
    fn cut_move_to_pos_proc(&mut self, _pos: Vec3, ctx: &mut Ctx<'_>) {
        let angle = target_angle_y(ctx.actor_pos, self.cut.pos);
        let mut temp = self.cut.pos;
        temp.x -= self.cut.del_distance * walk::from_s16(angle).sin();
        temp.z -= self.cut.del_distance * walk::from_s16(angle).cos();
        if self.cut.speed == 0.0 {
            ctx.actor_pos = temp;
            ctx.end_staff_cut(STAFF_HI1);
        }
        let dist = abs_xz(temp - ctx.actor_pos);
        add_calc2(&mut ctx.actor_pos.x, temp.x, CUT_MOVE_SCALE, self.cut.speed);
        add_calc2(&mut ctx.actor_pos.z, temp.z, CUT_MOVE_SCALE, self.cut.speed);
        if dist < CUT_MOVE_DONE {
            self.cut.speed = 0.0;
            ctx.end_staff_cut(STAFF_HI1);
        }
    }

    /// `dNpc_EventCut_c::cutTalkMsgStart` (`d_npc_event_cut.inc:557-584`).
    fn cut_talk_msg_start(&mut self, cut: usize, ctx: &Ctx<'_>) {
        self.cut.attention = ctx.list.cut_number(cut, "Attention").is_some();
        self.curr_msg_no = ctx
            .list
            .cut_number(cut, "msg_num")
            .map(|no| no as u32)
            .unwrap_or(0);
        self.end_msg_no = ctx
            .list
            .cut_number(cut, "end_msg")
            .map(|no| no as u32)
            .unwrap_or(u32::MAX);
        self.cut.field_0x52 = 1;
    }

    /// `dNpc_EventCut_c::cutContinueTalkStart` (`d_npc_event_cut.inc:587-597`).
    fn cut_continue_talk_start(&mut self, cut: usize, ctx: &Ctx<'_>) {
        self.end_msg_no = ctx
            .list
            .cut_number(cut, "end_num")
            .map(|no| no as u32)
            .unwrap_or(u32::MAX);
    }

    /// `dNpc_EventCut_c::cutTalkMsgProc` (`d_npc_event_cut.inc:600-614`).
    fn cut_talk_msg_proc(&mut self, ctx: &mut Ctx<'_>) {
        if self.cut.field_0x52 == 0 {
            ctx.end_staff_cut(STAFF_HI1);
            return;
        }
        let status = self.talk(false, ctx);
        if status == STATUS_BOX_CLOSED {
            self.cut.field_0x52 = 0;
            ctx.end_staff_cut(STAFF_HI1);
        } else if (status == STATUS_BOX_OPENING || status == STATUS_TYPING)
            && self.end_msg_no == self.curr_msg_no
        {
            self.end_msg_no = u32::MAX;
            ctx.end_staff_cut(STAFF_HI1);
        }
    }

    /// `dNpc_EventCut_c::cutTurnToActorProc`'s goal selection (`:205-221`), storing
    /// into `mPos` exactly as retail does. `:216` reads the target's
    /// `shape_angle.y`; the sim only carries `current.angle.y`.
    fn cut_goal(&mut self, from: Vec3, ctx: &mut Ctx<'_>) {
        let want_player = match self.cut.actor_name {
            None => true,
            Some(_) => false,
        };
        if want_player {
            self.cut.target_angle = ctx.player_angle;
            self.cut.pos = ctx.player_eye.unwrap_or(Vec3::ZERO) + self.cut.offset_pos;
            return;
        }
        let name = self.cut.actor_name.as_ref();
        let Some((pos, angle)) = ctx.find_target(self.cut.set_id, name, from) else {
            ctx.end_staff_cut(STAFF_HI1);
            self.cut.pos = Vec3::ZERO;
            return;
        };
        self.cut.target_angle = angle;
        self.cut.pos = if self.cut.add_angle != 0 && self.cut.del_distance != 0.0 {
            let angle = angle.wrapping_add(self.cut.add_angle);
            Vec3::new(
                pos.x - self.cut.del_distance * walk::from_s16(angle).sin(),
                pos.y,
                pos.z - self.cut.del_distance * walk::from_s16(angle).cos(),
            )
        } else {
            pos + self.cut.offset_pos
        };
    }

    /// `daNpc_Hi1_c::_execute` (`:846-899`), minus the renderer, `demo()`,
    /// `fopAcM_posMoveF`, `mObjAcch.CrrPos`, `tevStr`, `setMtx` and
    /// `setCollision`.
    pub fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        if !self.init_done {
            self.base_pos = actor.pos;
            self.home_angle = actor.angle;
            self.init_done = true;
        }
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
        self.check_order(ctx);
        ctx.actor_pos = actor.pos;
        ctx.angle = actor.angle;
        let staff = if ctx.event_run && !ctx.talk_command {
            self.is_event_entry(ctx)
        } else {
            CUT_NO_TARGET
        };
        if staff >= 0 {
            self.event_proc(staff, ctx);
        } else {
            self.wait_action1(actor, ctx);
        }
        actor.pos = ctx.actor_pos;
        self.look_back(actor, ctx);
        self.play_animation();
        self.event_order(ctx);
        self.set_attention(self.attn_ended, actor.pos);
    }
}

fn actor_pos_of(ctx: &Ctx<'_>) -> Vec3 {
    ctx.actor_pos
}

fn pos_angle(ctx: &Ctx<'_>) -> i16 {
    ctx.angle[1]
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

    pub fn mesg_anime_tag(&self) -> u8 {
        self.info.mesg_anime_tag
    }

    pub fn clear_mesg_anime_tag(&mut self) {
        self.info.clear_mesg_anime_tag_info();
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
    pub player_pos: Option<Vec3>,
    pub player_eye: Option<Vec3>,
    pub player_angle: i16,
    pub actor_pos: Vec3,
    pub angle: [i16; 3],
    pub attention: bool,
    pub talk_xy: bool,
    pub present_done: bool,
    pub talk_command: bool,
    pub demo_command: bool,
    pub event_run: bool,
    pub entity: Entity,
    pub active: &'a mut ActiveEvent,
    pub list: &'a EventList,
    pub flags: &'a mut super::flags::EventFlags,
    pub order_flags: &'a mut OrderFlags,
    pub targets: &'a [Target],
    pub text: &'a mut Text<'a>,
}

/// One non-`NpcHi1` actor, snapshotted once per tick for
/// `dNpc_EventCut_c::findActorCallBack` (`d_npc_event_cut.inc:346-388`).
#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    pub name: String,
    pub set_id: u16,
    pub pos: Vec3,
    pub angle: i16,
}

impl Ctx<'_> {
    fn start_check(&self, index: usize) -> bool {
        index != NO_EVENT && self.active.start_check(index)
    }

    fn end_check(&self, index: usize) -> bool {
        index != NO_EVENT && self.active.end_check(index)
    }

    fn end_staff_cut(&mut self, name: &str) {
        self.active.end_staff_cut(self.list, self.flags, name);
    }

    /// `fopAcM_orderOtherEventId` (`f_op_actor_mng.cpp:729-740`), guarded so a
    /// re-order every tick cannot reset `Stage::Order`.
    fn order_other(&mut self, index: usize) {
        if index == NO_EVENT {
            return;
        }
        if self.active.stage_of(index).is_none() {
            self.active.order(self.list, index, self.entity);
        }
    }

    /// `dNpc_EventCut_c::findActorCallBack` (`d_npc_event_cut.inc:346-388`).
    fn find_target(&self, set_id: u16, name: Option<&String>, from: Vec3) -> Option<(Vec3, i16)> {
        let mut best: Option<(Vec3, i16)> = None;
        for actor in self.targets {
            if set_id != 0 && actor.set_id == set_id {
                return Some((actor.pos, actor.angle));
            }
            let Some(name) = name else { continue };
            if actor.name != name.as_str() {
                continue;
            }
            match best {
                None => best = Some((actor.pos, actor.angle)),
                Some((pos, _)) => {
                    if (actor.pos - from).length() < (pos - from).length() {
                        best = Some((actor.pos, actor.angle));
                    }
                }
            }
        }
        best
    }
}

#[derive(SystemParam)]
pub struct Env<'w> {
    pub text_flags: Res<'w, TextFlags>,
    pub flags: ResMut<'w, super::flags::EventFlags>,
    pub order_flags: ResMut<'w, OrderFlags>,
    pub list: Res<'w, EventList>,
    pub active: ResMut<'w, ActiveEvent>,
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
    mut world: Env,
    mut msg: Msg,
    do_intent: Res<DoIntent>,
    others: Query<(Entity, &'static Actor), Without<NpcHi1>>,
    mut actors: Query<(Entity, &mut Actor, &mut NpcHi1)>,
    mut commands: Commands,
) {
    let player = msg.players.iter().next().copied();
    let targets: Vec<Target> = others
        .iter()
        .map(|(_, actor)| Target {
            name: actor.name.clone(),
            set_id: actor.set_id,
            pos: actor.pos,
            angle: actor.angle[1],
        })
        .collect();
    let clear_count = world.text_flags.clear_count;
    let event_run = msg.info.event_run_check();
    let do_now = do_intent.0;
    let archive = world.archive.as_deref().map(|held| held as &dyn MsgArchive);
    for (entity, mut actor, mut hi1) in &mut actors {
        if !hi1.entered && hi1.create_refused() {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        let tap = do_now && !hi1.prev_do;
        hi1.prev_do = do_now;
        let mut text = Text {
            procs: &mut msg.procs,
            info: &mut msg.info,
            clear_count,
            one: &mut msg.one,
            two: &mut msg.two,
            archive,
        };
        let mut ctx = Ctx {
            player_pos: player.map(|link| link.pos),
            player_eye: player.map(|link| {
                Vec3::new(
                    link.pos.x,
                    link.pos.y + PLAYER_HEAD_TOP + PLAYER_EYE_OFFSET_Y,
                    link.pos.z,
                )
            }),
            player_angle: player.map_or(0, |link| walk::to_s16(link.yaw)),
            actor_pos: actor.pos,
            angle: actor.angle,
            attention: false,
            talk_xy: false,
            present_done: false,
            talk_command: tap,
            demo_command: false,
            event_run,
            entity,
            active: &mut world.active,
            list: &world.list,
            flags: &mut world.flags,
            order_flags: &mut world.order_flags,
            targets: &targets,
            text: &mut text,
        };
        if !hi1.entered {
            hi1.create_init(&mut actor, &world.list);
        }
        hi1.execute(&mut actor, &mut ctx);
    }
}
