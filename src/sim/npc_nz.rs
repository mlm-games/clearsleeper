use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::flags::{Collect, EventFlags};
use super::intent::DoIntent;
use super::inventory::{Equipped, NO_ITEM, check_get_item};
use super::pickup::Rupees;
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
    STATUS_BOX_CLOSED, STATUS_BOX_CLOSING, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED,
    STATUS_MSG_DISPLAYED, STATUS_MSG_ENDS,
};

pub const NPC_NZ_ACTORS: [&str; 1] = ["NpcNz"];

pub fn is_npc_nz(name: &str) -> bool {
    NPC_NZ_ACTORS.contains(&name)
}

const STAFF_NZ: &str = "NpcNz"; // <- mEventCut.setActorInfo2("NpcNz", this), :900
const EVENT_ESA: &str = "DEFAULT_NPC_NZ_ESA"; // <- XyEventCB, :260
const EVENT_GETITEM: &str = "DEFAULT_NPC_NZ_GETITEM"; // <- next_msgStatus, :709

// daNpc_Nz_HIO_c ctor - d_a_npc_nz.cpp:214-230
const HIO_NO: i8 = -1; // <- mNo = -1, :215
const HIO_MAX_HEAD_X: i16 = 0x1F40; // <- field_0x06, :216
const HIO_MAX_HEAD_Y: i16 = 0x1F40; // <- field_0x08, :217
const HIO_MAX_BACKBONE_X: i16 = 0x0000; // <- field_0x0A, :218
const HIO_MAX_BACKBONE_Y: i16 = 0x1F40; // <- field_0x0C, :219
const HIO_MIN_HEAD_X: i16 = -0x0BB8; // <- field_0x0E, :220
const HIO_MIN_HEAD_Y: i16 = -0x1F40; // <- field_0x10, :221
const HIO_MIN_BACKBONE_X: i16 = 0x0000; // <- field_0x12, :222
const HIO_MIN_BACKBONE_Y: i16 = -0x1F40; // <- field_0x14, :223
const HIO_MAX_TURN_STEP: i16 = 0x07D0; // <- field_0x16, :224
const HIO_MAX_HEAD_TURN_VEL: i16 = 0x0640; // <- field_0x18, :225


// a_anm_idx_tbl - d_a_npc_nz.cpp:315-331 (mBckIdx, mSndIdx)
const ANM_IDX_TBL: [(i8, i8); 15] = [
    (0x23, 0x12), // <- :316
    (0x15, 0x07), // <- :317
    (0x16, -1),   // <- :318
    (0x17, 0x08), // <- :319
    (0x18, 0x09), // <- :320
    (0x19, 0x0A), // <- :321
    (0x1A, 0x0B), // <- :322
    (0x1B, 0x0C), // <- :323
    (0x1C, 0x0D), // <- :324
    (0x1D, 0x0E), // <- :325
    (0x1F, 0x10), // <- :326
    (0x20, 0x11), // <- :327
    (0x1E, 0x11), // <- :328
    (0x21, -1),   // <- :329
    (0x22, -1),   // <- :330
]; // <- :331
// a_anm_prm_tbl - d_a_npc_nz.cpp:333-348 (mAnmIdx, mNextPrmIdx, field_0x02, mMorf,
// mPlaySpeed, mLoopMode)
const ANM_PRM_TBL: [(i8, i8, i16, f32, f32, i32); 15] = [
    (0x00, -1, 0x00, 8.0, 1.0, 2), // <- :333
    (0x01, -1, 0x00, 8.0, 1.0, 0), // <- :334
    (0x02, -1, 0x00, 8.0, 1.0, 0), // <- :335
    (0x03, -1, 0x00, 8.0, 1.0, 0), // <- :336
    (0x04, -1, 0x00, 8.0, 1.0, 0), // <- :337
    (0x05, -1, 0x00, 8.0, 1.0, 0), // <- :338
    (0x06, -1, 0x00, 8.0, 1.0, 2), // <- :339
    (0x07, -1, 0x00, 8.0, 1.0, 2), // <- :340
    (0x08, -1, 0x00, 8.0, 1.0, 0), // <- :341
    (0x09, -1, 0x00, 8.0, 1.0, 0), // <- :342
    (0x0A, -1, 0x00, 8.0, 1.0, 2), // <- :343
    (0x0B, -1, 0x00, 8.0, 1.0, 2), // <- :344
    (0x0C, -1, 0x00, 8.0, 1.0, 0), // <- :345
    (0x0D, -1, 0x00, 8.0, 1.0, 0), // <- :346
    (0x0E, -1, 0x00, 8.0, 1.0, 0), // <- :347
]; // <- :348
const ANM_KEEP: i8 = 0xF; // <- setAnm param_1 != 0xF, :350
const ANM_WAIT: i8 = 0; // <- createInit setAnm(0, false), :877
const ANM_EAT: i8 = 6; // <- cutEatesaStart, d_a_npc_nz_cut.inc:185
const ANM_EAT_FIRST: i8 = 0xA; // <- cutEatesaFirstStart, :214
const ANM_TALK: i8 = 0xC; // <- anmAtr, :783
const ANM_TALK_BACK: i8 = 0xB; // <- anmAtr, :741
const ANM_TALK_PRE: i8 = 9; // <- anmAtr field_0x6D5 == 9, :737

// mode_tbl - d_a_npc_nz.cpp:502-513
const MODE_WAIT: i32 = 0; // <- :503
const MODE_EVENT_ESA: i32 = 1; // <- :508
const PROC_INIT: i32 = 0; // <- daNpc_Nz_c::Proc_e PROC_INIT_e, d_a_npc_nz.h:17
const PROC_EXEC: i32 = 1; // <- PROC_EXEC_e, d_a_npc_nz.h:18

// order field_0x8F8 - eventOrder / checkOrder :268, :279
const ORDER_NONE: u8 = 0; // <- :284
const ORDER_SPEAK: u8 = 1; // <- :271
const ORDER_TALK: u8 = 2; // <- :268
const ORDER_GETITEM: u8 = 3; // <- next_msgStatus, :708

// createInit - d_a_npc_nz.cpp:876-916
const HOME_OFFSET: f32 = 50.0; // <- home.pos -= sin/cos * 50.0f, :906-907

// _create fopAcM_setCullSizeBox - :893
const CULL_MIN: [f32; 3] = [-450.0, -50.0, -450.0]; // <- fopAcM_setCullSizeBox, d_a_npc_nz.cpp:893
const CULL_MAX: [f32; 3] = [450.0, 750.0, 450.0]; // <- fopAcM_setCullSizeBox, d_a_npc_nz.cpp:893

// next_msgStatus shop_next_msg_tbl - :608-625
const SHOP_NEXT_MSG: [[u32; 2]; 4] = [
    [0x33FB, 0x33FC], // <- :610-611
    [0x33FD, 0x33FE], // <- :613-614
    [0x33FF, 0x3400], // <- :617-618
    [0x3401, 0x3402], // <- :621-622
]; // <- :625
// itemArr1 - :627-644
const SHOP_ITEM_ARR: [[u8; 2]; 4] = [
    [0x82, 0x83], // <- dItemNo_BIRD_BAIT_5_e, dItemNo_HYOI_PEAR_e, :629-630
    [0x51, 0x53], // <- dItemNo_RED_POTION_e, dItemNo_BLUE_POTION_e, :633-634
    [0x0C, 0x0E], // <- dItemNo_BOMB_10_e, dItemNo_BOMB_30_e, :637-638
    [0x10, 0x12], // <- dItemNo_ARROW_10_e, dItemNo_ARROW_30_e, :641-642
]; // <- :644
const MSG_BUY_OK: u32 = 0x3403; // <- getShopBoughtMsg success, :551
const MSG_NO_RUPEE: u32 = 0x3405; // <- :528
const MSG_NO_BOTTLE: u32 = 0x3406; // <- :532
const MSG_BUY_BASE: u32 = 0x33F7; // <- :663
const SHOP_ROWS: i32 = 4; // <- temp = 1 + three checks, :646-660
const ITEM_RED_POTION: u8 = 0x51; // <- dItemNo_RED_POTION_e, :531
const ITEM_BLUE_POTION: u8 = 0x53; // <- dItemNo_BLUE_POTION_e, :531

// dInvSlot - d_com_inf_game.h:1073-1085
const SLOT_BOW: usize = 12; // <- dInvSlot_BOW_e, :658
const SLOT_BOTTLE0: usize = 14; // <- dInvSlot_BOTTLE0_e, d_com_inf_game.h:1077
const SLOT_BOTTLE_COUNT: usize = 4; // <- dInvSlot_BOTTLE_COUNT_e, d_com_inf_game.h:1081
const ITEM_EMPTY_BOTTLE: u8 = 0x50; // <- dItemNo_EMPTY_BOTTLE_e, d_item_data.h:96
const ITEM_BOMB_BAG: u8 = 0x31; // <- dItemNo_BOMB_BAG_e, d_item_data.h:56

// dNpc_EventCut_c action_table - d_npc_event_cut.inc:11-19
const GEN_WAIT: usize = 0; // <- :13
const GEN_TURN_TO_ACTOR: usize = 1; // <- :14
const GEN_MOVE_TO_ACTOR: usize = 2; // <- :15
const GEN_TURN_TO_POS: usize = 3; // <- :16
const GEN_MOVE_TO_POS: usize = 4; // <- :17
const GEN_TALK_MSG: usize = 5; // <- :18
const GEN_CONTINUE_TALK: usize = 6; // <- :19
const GEN_ACTIONS: [&str; 7] = [
    "WAIT",
    "TURN_TO_ACTOR",
    "MOVE_TO_ACTOR",
    "TURN_TO_POS",
    "MOVE_TO_POS",
    "TALK_MSG",
    "CONTINUE_TALK",
]; // <- :19
const GEN_TURN_TYPE_DEFAULT: i32 = 1; // <- cutTurnToPosStart, :402
const GEN_TIMER_DEFAULT: i32 = 1; // <- cutTurnToPosStart, :411
const GEN_TIMER_WAIT_DEFAULT: i32 = 0; // <- cutWaitStart, :91
const GEN_SPEED_DEFAULT: f32 = 2.0; // <- cutMoveToPosStart, :510
const GEN_STABLE_ANGLE: f32 = 5376.0; // <- std::fabsf(temp2) < 5376.0f, :466
const GEN_MOVE_SCALE: f32 = 0.1; // <- cLib_addCalc2 scale, :548
const GEN_MOVE_DONE: f32 = 5.0; // <- dist < 5.0f, :546
const GEN_TURN_BACKBONE_LOCK: u32 = 0; // <- mTurnType case 0, :457
const GEN_TURN_HEAD_LOCK: u32 = 2; // <- mTurnType case 2, :452

// cutProc action_table - d_a_npc_nz_cut.inc:120-128
const CUT_EATESA: usize = 0; // <- :121
const CUT_EATESA1ST: usize = 1; // <- :122
const CUT_KILLALL: usize = 2; // <- :123
const CUT_SHOW: usize = 3; // <- :124
const CUT_HIDE: usize = 4; // <- :125
const CUT_SETANM: usize = 5; // <- :126
const CUT_GOHOME: usize = 6; // <- :127
const CUT_CHKGETITEM: usize = 7; // <- :128
const CUT_ACTIONS: [&str; 8] = [
    "EATESA",
    "EATESA1ST",
    "KILLALL",
    "SHOW",
    "HIDE",
    "SETANM",
    "GOHOME",
    "CHKGETITEM",
]; // <- :128

// cutEatesaFirstProc - d_a_npc_nz_cut.inc:219-249
const EAT_OFFSET: f32 = 100.0; // <- target.x += sin(home.angle.y) * 100.0f, :220-221
const EAT_MOVE_SCALE: f32 = 0.25; // <- cLib_addCalcPosXZ scale, :224
const EAT_MOVE_MAX: f32 = 10.0; // <- maxStep, :224
const EAT_MOVE_MIN: f32 = 2.0; // <- minStep, :224
const EAT_TURN_SCALE: i16 = 8; // <- cLib_addCalcAngleS2 scale, :226
const EAT_TURN_MAX: i16 = 0x1000; // <- maxStep, :226
const EAT_TIMER_SET: i32 = 2; // <- field_0x906 = 2, :230
const IS_ZERO: f32 = 8e-11; // <- cLib_IsZero, c_lib.h:8-10
// cutKillAllStart - d_a_npc_nz_cut.inc:257-265
const KILL_OFFSET: f32 = 250.0; // <- pos.x += sin(home.angle.y) * 250.0f, :261-262
const KILL_ANGLE: i16 = 0x8000; // <- home.angle.y + 0x8000, :260
// cutGoHomeProc - d_a_npc_nz_cut.inc:360-375
const HOME_TURN_SCALE: i16 = 4; // <- cLib_addCalcAngleS2 scale, :362
const HOME_TURN_MAX: i16 = 0x1000; // <- maxStep, :362
const HOME_ANGLE_GATE: i32 = 0x4000; // <- cLib_distanceAngleS < 0x4000, :365
const HOME_MOVE_SCALE: f32 = 0.5; // <- cLib_addCalcPosXZ scale, :371
const HOME_MOVE_MAX: f32 = 10.0; // <- maxStep, :371
const HOME_MOVE_MIN: f32 = 1.0; // <- minStep, :371
const HOME_MOVE_DONE: f32 = 1.0; // <- < 1.0f, :371

const JNT_SCALE: i16 = 4; // <- d_npc.cpp follow angCalcS(..., 4, maxVel), d_npc.cpp:47
const JNT_ANG_MIN_STEP: i16 = 0x60; // <- cLib_addCalcAngleS min_step, d_npc.cpp:23
const JNT_MIN_STEP: i32 = 1; // <- cLib_addCalcAngleL min_step, d_npc.cpp:78
const PLAYER_HEAD_TOP: f32 = 40.0; // <- d_a_player_main.cpp:11575

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA]; // <- cM_rnd, c_math.cpp:178-180
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673]; // <- cM_rnd, c_math.cpp:178-180
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0]; // <- cM_rnd, c_math.cpp:181
const RND_SEED: i32 = 100; // <- cM_initRnd actor seeds, c_math.cpp:170-175

const NO_EVENT: usize = usize::MAX; // <- dComIfGp_evmng_getEventIdx miss
const NO_STATUS: u8 = 0xFF; // <- fopNpc_npc_c::talk u16 status = 0xFF, d_npc.cpp:665

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = seed[0] as f32 / RND_SCALE[0]
        + seed[1] as f32 / RND_SCALE[1]
        + seed[2] as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

fn calc_timer(value: &mut i32) -> i32 {
    let before = *value;
    if before != 0 {
        *value -= 1;
    }
    before
}

fn abs_xz(v: Vec3) -> f32 {
    (v.x * v.x + v.z * v.z).sqrt()
}

fn target_angle_y(from: Vec3, to: Vec3) -> i16 {
    walk::to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn distance_angle_s(x: i16, y: i16) -> i32 {
    i32::from(x.wrapping_sub(y)).abs()
}

/// `cLib_addCalcAngleS2` (`c_lib.cpp:192-202`).
fn add_calc_angle_s2(value: &mut i16, target: i16, scale: i16, max_step: i16) {
    super::stream::add_calc_angle_s2(value, target, scale, max_step);
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

/// `cLib_addCalcPosXZ` (`c_lib.cpp:105-131`), the XZ pair.
fn add_calc_pos_xz(value: &mut Vec3, target: Vec3, scale: f32, max_step: f32, min_step: f32) -> f32 {
    if value.x != target.x || value.z != target.z {
        let mut diff = *value - target;
        let mut step = abs_xz(diff);
        if step < min_step {
            value.x = target.x;
            value.z = target.z;
        } else {
            step *= scale;
            diff *= scale;
            if step.abs() >= IS_ZERO {
                if step > max_step {
                    diff *= max_step / step;
                } else if step < min_step {
                    diff *= min_step / step;
                }
                value.x -= diff.x;
                value.z -= diff.z;
            } else {
                value.x = target.x;
                value.z = target.z;
            }
        }
    }
    abs_xz(*value - target)
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

#[derive(Debug, Clone, Copy, PartialEq)]
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

    fn ang_calc_s(out: &mut i16, target: i16, speed: i16, max_vel: i16) -> bool {
        let orig = out.wrapping_sub(target);
        let diff = walk::add_calc_angle_s(orig, 0, speed, max_vel, JNT_ANG_MIN_STEP);
        if i32::from(diff).abs() > i32::from(speed) {
            *out = out.wrapping_add(diff.wrapping_sub(orig));
        } else {
            *out = target;
        }
        target.wrapping_sub(*out) == 0
    }

    fn follow(&mut self, out: &mut i16, target: i16, max_vel: i16, axis: usize) -> bool {
        let orig = *out;
        Self::ang_calc_s(out, target, JNT_SCALE, max_vel);
        let delta = i32::from(out.wrapping_sub(orig));
        let mut prev = 0i16;
        let mut acc = 0i16;
        let want = target.wrapping_sub(*out);
        for row in (0..2).rev() {
            acc = acc.wrapping_add(self.angles[row][axis]);
            if (delta > 0 && i32::from(acc) > i32::from(want))
                || (delta < 0 && i32::from(acc) < i32::from(want))
            {
                self.angles[row][axis] = want.wrapping_sub(prev);
                limitter(
                    &mut self.angles[row][axis],
                    self.min_angles[row][axis],
                    self.max_angles[row][axis],
                );
            }
            prev = prev.wrapping_add(self.angles[row][axis]);
        }
        target != *out
    }

    fn r#move(&mut self, target: i16, axis: usize) -> bool {
        let mut rest = target;
        let mut reached = [0i16; 2];
        for row in 0..2 {
            let locked = (row == 0 && self.head_lock) || (row == 1 && self.backbone_lock);
            let mut angle = if locked { 0 } else { rest };
            limitter(
                &mut angle,
                self.min_angles[row][axis],
                self.max_angles[row][axis],
            );
            rest = rest.wrapping_sub(angle);
            reached[row] = angle;
        }
        for row in 0..2 {
            let current = i32::from(self.angles[row][axis]);
            let next = add_calc_angle_l(
                current,
                i32::from(reached[row]),
                JNT_SCALE as i32,
                i32::from(self.max_turn_step[row][axis]),
                JNT_MIN_STEP,
            );
            self.angles[row][axis] = next as i16;
        }
        let mut done = 0;
        for row in 0..2 {
            if i32::from(reached[row].wrapping_sub(self.angles[row][axis])).abs()
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
        free: bool,
    ) {
        let (target_y, pitch) = match dst {
            Some(point) => {
                let delta = point - src;
                let flat = (delta.x * delta.x + delta.z * delta.z).sqrt();
                (walk::to_s16(delta.x.atan2(delta.z)), walk::to_s16(delta.y.atan2(flat)))
            }
            None => (default_y, 0),
        };
        let delta_y = i32::from(target_y) - i32::from(*out_y);
        let max_y = i32::from(self.max_angles[1][1]) + i32::from(self.max_angles[0][1]);
        let min_y = i32::from(self.min_angles[1][1]) + i32::from(self.min_angles[0][1]);
        let settled = self.r#move(delta_y as i16, 1);
        if (self.trn || (delta_y >= max_y || delta_y <= min_y) && settled) && !free {
            self.trn = self.follow(out_y, target_y, max_vel, 1);
        }
        self.r#move(pitch, 0);
    }
}

fn limitter(value: &mut i16, min_diff: i16, max_diff: i16) {
    if *value > max_diff {
        *value = max_diff;
    }
    if *value < min_diff {
        *value = min_diff;
    }
}

/// `dNpc_EventCut_c` state - d_npc.h:120-137.
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

/// One non-`NpcNz` actor, snapshotted once per tick for
/// `dNpc_EventCut_c::findActorCallBack` (`d_npc_event_cut.inc:346-388`).
#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    pub name: String,
    pub set_id: u16,
    pub pos: Vec3,
    pub angle: i16,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct NpcNz {
    pub entered: bool,
    pub prev_do: bool,
    pub mode: i32,
    pub order: u8,
    pub esa_event: usize,
    pub getitem_event: usize,
    pub shop_visible: bool,
    pub shop_msg_base: u32,
    pub shop_slot: i32,
    pub timer: i32,
    pub shop_item: u8,
    pub anm_idx: i8,
    pub prm_idx: i8,
    pub old_prm_idx: i8,
    pub base_pos: Vec3,
    pub home_pos: Vec3,
    pub home_angle: i16,
    pub shape_angle: i16,
    pub eye_pos: Vec3,
    pub attn_pos: Vec3,
    pub curr_msg_no: u32,
    pub end_msg_no: u32,
    pub msg_id: u32,
    pub msg_bound: bool,
    pub seed: [i32; 3],
    pub cut: EventCut,
    pub jnt: JntCtrl,
}

impl NpcNz {
    pub fn spawn(parameters: u32) -> Self {
        let _ = parameters;
        Self {
            entered: false,
            prev_do: false,
            mode: MODE_WAIT,
            order: ORDER_NONE,
            esa_event: NO_EVENT,
            getitem_event: NO_EVENT,
            shop_visible: false,
            shop_msg_base: 0,
            shop_slot: 0,
            timer: 0,
            shop_item: 0,
            anm_idx: 0,
            prm_idx: 0,
            old_prm_idx: HIO_NO,
            base_pos: Vec3::ZERO,
            home_pos: Vec3::ZERO,
            home_angle: 0,
            shape_angle: 0,
            eye_pos: Vec3::ZERO,
            attn_pos: Vec3::ZERO,
            curr_msg_no: 0,
            end_msg_no: u32::MAX,
            msg_id: FPC_M_ERROR_PROCESS_ID,
            msg_bound: false,
            seed: [RND_SEED; 3],
            cut: EventCut::create(),
            jnt: JntCtrl::create(),
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.shop_visible
    }

    /// `daNpc_Nz_c::setAnm` (`:314-364`) plus `dLib_setAnm` (`d_lib.cpp:137-156`),
    /// minus the `morf->isStop()` auto-advance (BCK frame counts are unavailable).
    fn set_anm(&mut self, param_1: i8, force: bool) {
        if param_1 != ANM_KEEP {
            self.prm_idx = param_1;
        }
        if self.prm_idx == 0 {
            self.jnt.off_head_lock();
            self.jnt.off_backbone_lock();
        } else {
            self.jnt.on_head_lock();
            self.jnt.on_backbone_lock();
        }
        let row = self.prm_idx.max(0) as usize;
        if let Some(prm) = ANM_PRM_TBL.get(row) {
            if (self.old_prm_idx != self.prm_idx && prm.0 != -1) || force {
                self.anm_idx = prm.0;
                let _ = ANM_IDX_TBL
                    .get(self.anm_idx.max(0) as usize)
                    .copied()
                    .unwrap_or((-1, -1));
            }
        }
        self.old_prm_idx = self.prm_idx;
    }

    /// `daNpc_Nz_c::setAttention` (`:289-292`); `REG10_F(9)` is a proven-zero HIO
    /// float (RULE 2b), so the y offset is exactly 0.0.
    fn set_attention(&mut self, pos: Vec3) {
        self.attn_pos = Vec3::new(pos.x, self.eye_pos.y, pos.z);
    }

    /// `daNpc_Nz_c::LookBack` (`:295-311`).
    fn look_back(&mut self, actor: &mut Actor, player_eye: Option<Vec3>) {
        let target_y = actor.angle[1];
        let max_follow = if self.jnt.trn_chk() {
            HIO_MAX_HEAD_TURN_VEL
        } else {
            0
        };
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
        self.jnt.look_at_target(
            &mut actor.angle[1],
            player_eye,
            self.eye_pos,
            target_y,
            max_follow,
            true,
        );
    }

    /// `daNpc_Nz_c::eventOrder` (`:267-275`). `onCondition` only raises the
    /// CANTALK / CANTALKITEM conditions the sim's `DoIntent` press stands in for.
    fn event_order(&mut self) {}

    /// `daNpc_Nz_c::checkOrder` (`:278-286`).
    fn check_order(&mut self, talk_command: bool) {
        if talk_command && (self.order == ORDER_SPEAK || self.order == ORDER_TALK) {
            if self.order == ORDER_TALK {
                self.mode_proc_init(MODE_EVENT_ESA);
            }
            self.order = ORDER_NONE;
        }
    }

    /// `daNpc_Nz_c::modeWaitInit` (`:457-459`).
    fn mode_wait_init(&mut self) {}

    /// `daNpc_Nz_c::modeWait` (`:462-464`).
    fn mode_wait(&mut self) {
        self.order = ORDER_TALK;
    }

    /// `daNpc_Nz_c::modeEventEsaInit` (`:467-469`).
    fn mode_event_esa_init(&mut self) {}

    /// `daNpc_Nz_c::modeEventEsa` (`:472-491`), minus `deleteShopItem` and
    /// `dComIfGp_event_reset`.
    fn mode_event_esa(&mut self, ctx: &mut Ctx<'_>) {
        if !self.generic_cut_proc(ctx) {
            self.cut_proc(ctx);
        }
        if ctx.end_check(self.esa_event) {
            self.esa_event = NO_EVENT;
            self.shop_visible = false;
            self.base_pos = self.home_pos;
            ctx.actor_pos = self.base_pos;
            self.shape_angle = self.home_angle;
            ctx.actor_angle[1] = self.shape_angle;
            self.mode_proc_init(MODE_WAIT);
        }
    }

    /// `daNpc_Nz_c::modeProc` (`:494-522`).
    fn mode_proc(&mut self, proc: i32, ctx: &mut Ctx<'_>) {
        if proc == PROC_INIT {
            match ctx.new_mode {
                MODE_EVENT_ESA => self.mode_event_esa_init(),
                _ => self.mode_wait_init(),
            }
            self.mode = ctx.new_mode;
        } else if proc == PROC_EXEC {
            match self.mode {
                MODE_EVENT_ESA => self.mode_event_esa(ctx),
                _ => self.mode_wait(),
            }
        }
    }

    /// `daNpc_Nz_c::modeProcInit` (`d_a_npc_nz.h:23`).
    fn mode_proc_init(&mut self, new_mode: i32) {
        self.mode = new_mode;
    }

    /// `daNpc_Nz_c::createInit` (`:876-916`), minus the `mAcch` collider setup,
    /// `mAcch.CrrPos`, `setMtx`, `mpMorf->calc` and the `eventInfo` XY callbacks.
    pub fn create_init(&mut self, actor: &mut Actor, list: &EventList) {
        self.set_anm(ANM_WAIT, false);
        self.mode_proc_init(MODE_WAIT);
        self.esa_event = list.event_index(EVENT_ESA).unwrap_or(NO_EVENT);
        self.getitem_event = list.event_index(EVENT_GETITEM).unwrap_or(NO_EVENT);
        self.order = ORDER_NONE;
        self.shop_visible = false;
        self.timer = 0;
        self.home_angle = actor.angle[1];
        self.shape_angle = actor.angle[1];
        let mut home = actor.pos;
        let rad = walk::from_s16(actor.angle[1]);
        home.x -= rad.sin() * HOME_OFFSET;
        home.z -= rad.cos() * HOME_OFFSET;
        self.home_pos = home;
        self.base_pos = self.home_pos;
        actor.pos = self.home_pos;
        self.entered = true;
    }

    /// `fopNpc_npc_c::getMsg` (`d_npc.h:262`) returns 0.
    fn get_msg(&self) -> u32 {
        0
    }

    /// `fopNpc_npc_c::talk` (`d_npc.cpp:664-697`).
    fn talk(&mut self, ctx: &mut Ctx<'_>) -> u8 {
        let mut status = NO_STATUS;
        if self.msg_id == FPC_M_ERROR_PROCESS_ID {
            self.curr_msg_no = self.get_msg();
            let no = self.curr_msg_no;
            self.msg_id = ctx.text.message_set_actor(no, self.eye_pos);
            self.msg_bound = false;
        } else if self.msg_bound {
            status = ctx.text.status().unwrap_or(NO_STATUS);
            if status == STATUS_MSG_DISPLAYED {
                let select = ctx.text.select_num();
                let next = self.next_msg_status(select, ctx);
                ctx.text.set_status(next);
                if next == STATUS_MSG_CONTINUES {
                    let no = self.curr_msg_no;
                    let _ = ctx.text.message_set_actor(no, self.eye_pos);
                }
            } else if status == STATUS_BOX_CLOSED {
                ctx.text.set_status(STATUS_MSG_DESTROYED);
                self.msg_id = FPC_M_ERROR_PROCESS_ID;
                self.msg_bound = false;
            }
            self.anm_atr(status, ctx);
        } else {
            self.msg_bound = ctx.text.procs.search_by_id(self.msg_id).is_some();
        }
        status
    }

    /// `daNpcNz_getShopBoughtMsg` (`:525-555`), minus the bomb/arrow capacity
    /// checks (`:535`, `:539`) and the bait-capacity check (`:543`): the sim has
    /// no bomb/arrow count or bait-bag resource. `execItemGet` is limited to the
    /// rupee payout the sim models.
    fn shop_bought_msg(&mut self, item: u8, ctx: &mut Ctx<'_>) -> u32 {
        let message_rupee = i32::from(ctx.message_rupee);
        if i32::try_from(ctx.rupees.count).unwrap_or(0) < message_rupee {
            return MSG_NO_RUPEE;
        }
        if (item == ITEM_RED_POTION || item == ITEM_BLUE_POTION)
            && check_empty_bottle(ctx.equipped) == 0
        {
            return MSG_NO_BOTTLE;
        }
        if super::inventory::check_get_item(ctx.equipped, ctx.collect, item) {
            ctx.rupees.count = ctx.rupees.count.saturating_sub(message_rupee as u32);
            return MSG_BUY_OK;
        }
        0
    }

    /// `daNpc_Nz_c::next_msgStatus` (`:605-733`).
    fn next_msg_status(&mut self, select_num: u8, ctx: &mut Ctx<'_>) -> u8 {
        let mut msg_status = STATUS_MSG_CONTINUES;
        match self.curr_msg_no {
            0x33F5 => {
                self.curr_msg_no = 0x33F6;
            }
            0x33F6 => {
                let mut temp = 1;
                if check_get_bottle(ctx.equipped) {
                    temp += 1;
                }
                if check_get_item(&ctx.equipped, &ctx.collect, ITEM_BOMB_BAG) {
                    temp += 1;
                }
                if ctx.equipped.item(SLOT_BOW) != NO_ITEM {
                    temp += 1;
                }
                let slot = (rnd(&mut self.seed) * temp as f32) as i32;
                self.shop_slot = slot.clamp(0, SHOP_ROWS - 1);
                self.curr_msg_no = (MSG_BUY_BASE as i32 + self.shop_slot) as u32;
                self.shop_msg_base = self.curr_msg_no;
            }
            0x33F7..=0x33FA => match select_num {
                0 | 1 => {
                    let row = self.shop_slot.clamp(0, SHOP_ROWS - 1) as usize;
                    let pair = SHOP_ITEM_ARR[row];
                    self.shop_item = pair[usize::from(select_num)];
                    self.curr_msg_no = SHOP_NEXT_MSG[row][usize::from(select_num)];
                }
                2 => {
                    self.curr_msg_no = 0x3404;
                }
                _ => {}
            },
            0x33FB..=0x3402 => match select_num {
                0 => {
                    let item = self.shop_item;
                    let msg = self.shop_bought_msg(item, ctx);
                    if msg != 0 {
                        self.curr_msg_no = msg;
                    } else {
                        self.order = ORDER_GETITEM;
                        self.esa_event = self.getitem_event;
                        msg_status = STATUS_MSG_ENDS;
                    }
                }
                1 => {
                    self.curr_msg_no = self.shop_msg_base;
                }
                _ => {}
            },
            0x3404..=0x3407 => {
                self.curr_msg_no = MSG_BUY_OK;
            }
            _ => {
                msg_status = STATUS_MSG_ENDS;
            }
        }
        msg_status
    }

    /// `daNpc_Nz_c::anmAtr` (`:736-813`), minus the `mpMorf->checkFrame` end-frame
    /// tests (`:737`, `:740`) and every `getShopItem(...)->show()/hide()`.
    fn anm_atr(&mut self, status: u8, ctx: &mut Ctx<'_>) {
        if self.prm_idx == ANM_TALK_PRE {
            self.set_anm(ANM_WAIT, false);
        } else if self.prm_idx == ANM_TALK {
            self.set_anm(ANM_TALK_BACK, false);
        }
        if ctx.text.send_button() {
            if status == STATUS_BOX_CLOSING
                || status == STATUS_BOX_CLOSED
                || status == STATUS_MSG_ENDS
            {
                return;
            }
            match self.curr_msg_no {
                0x33F5 | 0x33F6 => {}
                0x33F7..=0x33FA => {
                    self.set_anm(ANM_WAIT, false);
                }
                0x33FB..=0x3402 => {
                    self.set_anm(ANM_TALK, false);
                }
                0x3405..=0x3407 => {
                    self.set_anm(ANM_WAIT, false);
                }
                0x3404 | MSG_BUY_OK => {
                    self.set_anm(ANM_WAIT, false);
                }
                _ => {}
            }
        }
    }

    /// `dNpc_EventCut_c::cutProc` (`d_npc_event_cut.inc:10-82`).
    fn generic_cut_proc(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let Some((cut, advance)) = ctx.active.staff_cut(ctx.list, STAFF_NZ) else {
            return false;
        };
        let Some(act) = GEN_ACTIONS
            .iter()
            .position(|name| ctx.list.cut_name(cut) == Some(*name))
        else {
            ctx.end_staff_cut();
            return false;
        };
        if advance {
            self.cut.attention = false;
            match act {
                GEN_WAIT => self.gen_wait_start(cut, ctx),
                GEN_TURN_TO_ACTOR => self.gen_turn_to_actor_start(cut, ctx),
                GEN_MOVE_TO_ACTOR => self.gen_move_to_actor_start(cut, ctx),
                GEN_TURN_TO_POS => self.gen_turn_to_pos_start(cut, ctx),
                GEN_MOVE_TO_POS => self.gen_move_to_pos_start(cut, ctx),
                GEN_TALK_MSG => self.gen_talk_msg_start(cut, ctx),
                GEN_CONTINUE_TALK => self.gen_continue_talk_start(cut, ctx),
                _ => {}
            }
        }
        match act {
            GEN_WAIT => self.gen_wait_proc(ctx),
            GEN_TURN_TO_ACTOR | GEN_TURN_TO_POS => self.gen_turn_proc(ctx),
            GEN_MOVE_TO_ACTOR | GEN_MOVE_TO_POS => self.gen_move_proc(ctx),
            GEN_TALK_MSG | GEN_CONTINUE_TALK => self.gen_talk_msg_proc(ctx),
            _ => {}
        }
        true
    }

    /// `dNpc_EventCut_c::cutWaitStart` (`d_npc_event_cut.inc:85-93`).
    fn gen_wait_start(&mut self, cut: usize, ctx: &Ctx<'_>) {
        self.cut.timer = ctx
            .list
            .cut_number(cut, "Timer")
            .unwrap_or(GEN_TIMER_WAIT_DEFAULT);
    }

    /// `dNpc_EventCut_c::cutWaitProc` (`d_npc_event_cut.inc:96-100`).
    fn gen_wait_proc(&mut self, ctx: &mut Ctx<'_>) {
        if calc_timer(&mut self.cut.timer) == 0 {
            ctx.end_staff_cut();
        }
    }

    /// `dNpc_EventCut_c::cutTurnToActorStart` (`d_npc_event_cut.inc:103-171`).
    fn gen_turn_to_actor_start(&mut self, cut: usize, ctx: &Ctx<'_>) {
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
            .unwrap_or(GEN_TURN_TYPE_DEFAULT)
            .max(0) as u32;
        self.cut.timer = ctx
            .list
            .cut_number(cut, "Timer")
            .unwrap_or(GEN_TIMER_DEFAULT);
        self.cut.turn_speed = ctx.list.cut_number(cut, "TurnSpeed").unwrap_or(0) as i16;
        self.cut.add_angle = ctx.list.cut_number(cut, "AddAngle").unwrap_or(0) as i16;
        self.cut.del_distance = ctx.list.cut_float(cut, "DelDistance").unwrap_or(0.0);
        self.cut.no_turn = ctx.list.cut_number(cut, "NoTurn").is_some();
        self.cut.field_0x44 = 0.0;
    }

    /// `dNpc_EventCut_c::cutMoveToActorStart` (`d_npc_event_cut.inc:241-299`).
    fn gen_move_to_actor_start(&mut self, cut: usize, ctx: &Ctx<'_>) {
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
            .unwrap_or(GEN_SPEED_DEFAULT);
        self.cut.del_distance = ctx.list.cut_float(cut, "DelDistance").unwrap_or(0.0);
        self.cut.add_angle = ctx.list.cut_number(cut, "AddAngle").unwrap_or(0) as i16;
    }

    /// `dNpc_EventCut_c::cutTurnToPosStart` (`d_npc_event_cut.inc:391-441`).
    fn gen_turn_to_pos_start(&mut self, cut: usize, ctx: &Ctx<'_>) {
        self.cut.pos = ctx.list.cut_vec(cut, "Pos").unwrap_or(Vec3::ZERO);
        self.cut.turn_type = ctx
            .list
            .cut_number(cut, "TurnType")
            .unwrap_or(GEN_TURN_TYPE_DEFAULT)
            .max(0) as u32;
        self.cut.timer = ctx
            .list
            .cut_number(cut, "Timer")
            .unwrap_or(GEN_TIMER_DEFAULT);
        self.cut.turn_speed = ctx.list.cut_number(cut, "TurnSpeed").unwrap_or(0) as i16;
        self.cut.no_turn = ctx.list.cut_number(cut, "NoTurn").is_some();
        self.cut.add_angle = ctx.list.cut_number(cut, "AddAngle").unwrap_or(0) as i16;
        self.cut.field_0x44 = 0.0;
    }

    /// `dNpc_EventCut_c::cutMoveToPosStart` (`d_npc_event_cut.inc:479-528`).
    fn gen_move_to_pos_start(&mut self, cut: usize, ctx: &Ctx<'_>) {
        self.cut.no_turn = ctx.list.cut_number(cut, "NoTurn").is_some();
        self.cut.pos = ctx.list.cut_vec(cut, "Pos").unwrap_or(Vec3::ZERO);
        self.cut.attention = ctx.list.cut_number(cut, "Attention").is_some();
        self.cut.speed = ctx
            .list
            .cut_float(cut, "Speed")
            .unwrap_or(GEN_SPEED_DEFAULT);
        self.cut.del_distance = ctx.list.cut_float(cut, "DelDistance").unwrap_or(0.0);
        self.cut.add_angle = ctx.list.cut_number(cut, "AddAngle").unwrap_or(0) as i16;
    }

    /// `dNpc_EventCut_c::cutTurnToActorProc` / `cutTurnToPosProc`
    /// (`d_npc_event_cut.inc:174-238` / `:444-476`).
    fn gen_turn_proc(&mut self, ctx: &mut Ctx<'_>) {
        self.cut_goal(ctx);
        let pos = ctx.actor_pos;
        let t_angle = target_angle_y(pos, self.cut.pos);
        match self.cut.turn_type {
            GEN_TURN_HEAD_LOCK => {
                self.jnt.set_trn();
                self.jnt.off_backbone_lock();
            }
            GEN_TURN_BACKBONE_LOCK => self.jnt.on_backbone_lock(),
            _ => self.jnt.off_backbone_lock(),
        }
        let temp2 = (t_angle.wrapping_sub(ctx.actor_angle[1]) as f32).abs();
        if self.cut.field_0x44 == temp2 && temp2 < GEN_STABLE_ANGLE {
            calc_timer(&mut self.cut.timer);
        }
        if self.cut.timer == 0 {
            self.cut.turn_speed = 0;
            ctx.end_staff_cut();
        }
        self.cut.field_0x44 = (t_angle.wrapping_sub(ctx.actor_angle[1]) as f32).abs();
    }

    /// `dNpc_EventCut_c::cutMoveToActorProc` / `cutMoveToPosProc`
    /// (`d_npc_event_cut.inc:302-343` / `:531-554`).
    fn gen_move_proc(&mut self, ctx: &mut Ctx<'_>) {
        self.cut_goal(ctx);
        let goal = self.cut.pos;
        let angle = if self.cut.add_angle != 0 {
            self.cut.target_angle.wrapping_add(self.cut.add_angle)
        } else {
            target_angle_y(ctx.actor_pos, goal)
        };
        let rad = walk::from_s16(angle);
        let temp = Vec3::new(
            goal.x - self.cut.del_distance * rad.sin(),
            goal.y,
            goal.z - self.cut.del_distance * rad.cos(),
        );
        if self.cut.speed == 0.0 {
            ctx.actor_pos = temp;
            ctx.end_staff_cut();
        }
        let dist = abs_xz(temp - ctx.actor_pos);
        add_calc2(&mut ctx.actor_pos.x, temp.x, GEN_MOVE_SCALE, self.cut.speed);
        add_calc2(&mut ctx.actor_pos.z, temp.z, GEN_MOVE_SCALE, self.cut.speed);
        if dist < GEN_MOVE_DONE {
            self.cut.speed = 0.0;
            ctx.end_staff_cut();
        }
    }

    /// `dNpc_EventCut_c::cutTalkMsgStart` (`d_npc_event_cut.inc:557-584`).
    fn gen_talk_msg_start(&mut self, cut: usize, ctx: &Ctx<'_>) {
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
    fn gen_continue_talk_start(&mut self, cut: usize, ctx: &Ctx<'_>) {
        self.end_msg_no = ctx
            .list
            .cut_number(cut, "end_num")
            .map(|no| no as u32)
            .unwrap_or(u32::MAX);
    }

    /// `dNpc_EventCut_c::cutTalkMsgProc` (`d_npc_event_cut.inc:600-614`).
    fn gen_talk_msg_proc(&mut self, ctx: &mut Ctx<'_>) {
        if self.cut.field_0x52 == 0 {
            ctx.end_staff_cut();
            return;
        }
        let status = self.talk(ctx);
        if status == STATUS_BOX_CLOSED {
            self.cut.field_0x52 = 0;
            ctx.end_staff_cut();
        } else if (status == crate::text::STATUS_BOX_OPENING || status == crate::text::STATUS_TYPING)
            && self.end_msg_no == self.curr_msg_no
        {
            self.end_msg_no = u32::MAX;
            ctx.end_staff_cut();
        }
    }

    /// `dNpc_EventCut_c::cutTurnToActorProc`'s goal selection (`:205-221`).
    fn cut_goal(&mut self, ctx: &mut Ctx<'_>) {
        if self.cut.actor_name.is_none() {
            self.cut.target_angle = ctx.player_angle;
            self.cut.pos = ctx.player_eye.unwrap_or(Vec3::ZERO) + self.cut.offset_pos;
            return;
        }
        let name = self.cut.actor_name.as_ref();
        let from = ctx.actor_pos;
        let Some((pos, angle)) = ctx.find_target(self.cut.set_id, name, from) else {
            ctx.end_staff_cut();
            return;
        };
        self.cut.target_angle = angle;
        if self.cut.add_angle != 0 && self.cut.del_distance != 0.0 {
            let yaw = angle.wrapping_add(self.cut.add_angle);
            let rad = walk::from_s16(yaw);
            self.cut.pos = Vec3::new(
                pos.x - self.cut.del_distance * rad.sin(),
                pos.y,
                pos.z - self.cut.del_distance * rad.cos(),
            );
        } else {
            self.cut.pos = pos + self.cut.offset_pos;
        }
    }

    /// `daNpc_Nz_c::cutProc` (`d_a_npc_nz_cut.inc:108-181`).
    fn cut_proc(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let Some((cut, advance)) = ctx.active.staff_cut(ctx.list, STAFF_NZ) else {
            return false;
        };
        let Some(act) = CUT_ACTIONS
            .iter()
            .position(|name| ctx.list.cut_name(cut) == Some(*name))
        else {
            ctx.end_staff_cut();
            return false;
        };
        if advance {
            match act {
                CUT_EATESA => self.set_anm(ANM_EAT, false),
                CUT_EATESA1ST => {
                    self.shape_angle = self.home_angle;
                    ctx.actor_angle[1] = self.home_angle;
                    self.set_anm(ANM_EAT_FIRST, false);
                }
                CUT_KILLALL => self.cut_kill_all_start(ctx),
                CUT_SHOW => self.shop_visible = true,
                CUT_HIDE => self.shop_visible = false,
                CUT_SETANM => self.cut_set_anm_start(cut, ctx),
                CUT_GOHOME => {}
                CUT_CHKGETITEM => self.cut_chk_get_item_start(ctx),
                _ => {}
            }
        }
        match act {
            CUT_EATESA => ctx.end_staff_cut(),
            CUT_EATESA1ST => self.cut_eat_first_proc(ctx),
            CUT_KILLALL => ctx.end_staff_cut(),
            CUT_SHOW => ctx.end_staff_cut(),
            CUT_HIDE => ctx.end_staff_cut(),
            CUT_SETANM => self.cut_set_anm_proc(ctx),
            CUT_GOHOME => self.cut_go_home_proc(ctx),
            CUT_CHKGETITEM => ctx.end_staff_cut(),
            _ => {}
        }
        true
    }

    /// `daNpc_Nz_c::cutEatesaFirstProc` (`d_a_npc_nz_cut.inc:218-249`), minus the
    /// bait search and the bait delete.
    fn cut_eat_first_proc(&mut self, ctx: &mut Ctx<'_>) {
        let mut target = self.base_pos;
        let rad = walk::from_s16(self.home_angle);
        target.x += rad.sin() * EAT_OFFSET;
        target.z += rad.cos() * EAT_OFFSET;
        if self.prm_idx == 10 {
            let pos = ctx.actor_pos;
            let rest = add_calc_pos_xz(
                &mut { pos },
                target,
                EAT_MOVE_SCALE,
                EAT_MOVE_MAX,
                EAT_MOVE_MIN,
            );
            ctx.actor_pos = pos;
            let want = target_angle_y(ctx.actor_pos, target);
            add_calc_angle_s2(&mut self.shape_angle, want, EAT_TURN_SCALE, EAT_TURN_MAX);
            if rest < IS_ZERO {
                self.set_anm(ANM_EAT, false);
                self.timer = EAT_TIMER_SET;
            } else {
                return;
            }
        }
        ctx.end_staff_cut();
    }

    /// `daNpc_Nz_c::cutKillAllStart` (`d_a_npc_nz_cut.inc:252-265`), minus the
    /// Nz / Bomb / Tsubo / Barrel searches and the Esa randomiser.
    fn cut_kill_all_start(&mut self, ctx: &mut Ctx<'_>) {
        let mut pos = self.base_pos;
        let rad = walk::from_s16(self.home_angle);
        pos.x += rad.sin() * KILL_OFFSET;
        pos.z += rad.cos() * KILL_OFFSET;
        ctx.teleport = Some((pos, self.home_angle.wrapping_add(KILL_ANGLE)));
    }

    /// `daNpc_Nz_c::cutSetAnmStart` (`d_a_npc_nz_cut.inc:293-313`).
    fn cut_set_anm_start(&mut self, cut: usize, ctx: &Ctx<'_>) {
        let anm_no = ctx.list.cut_number(cut, "anm_no").unwrap_or(0);
        self.timer = ctx.list.cut_number(cut, "loop_count").unwrap_or(0);
        self.set_anm(anm_no.clamp(0, 0xFF) as i8, false);
    }

    /// `daNpc_Nz_c::cutSetAnmProc` (`d_a_npc_nz_cut.inc:316-352`), minus the
    /// `mpMorf->checkFrame` gates and the particle emitters.
    fn cut_set_anm_proc(&mut self, ctx: &mut Ctx<'_>) {
        if self.timer > 0 {
            self.timer -= 1;
        }
        if self.timer <= 0 {
            ctx.end_staff_cut();
        }
    }

    /// `daNpc_Nz_c::cutGoHomeProc` (`d_a_npc_nz_cut.inc:360-375`).
    fn cut_go_home_proc(&mut self, ctx: &mut Ctx<'_>) {
        let target = target_angle_y(ctx.actor_pos, self.home_pos);
        let mut angle = ctx.actor_angle[1];
        add_calc_angle_s2(&mut angle, target, HOME_TURN_SCALE, HOME_TURN_MAX);
        ctx.actor_angle[1] = angle;
        self.shape_angle = angle;
        if distance_angle_s(target, angle) < HOME_ANGLE_GATE {
            self.set_anm(ANM_EAT_FIRST, false);
            let rest = add_calc_pos_xz(
                &mut ctx.actor_pos,
                self.home_pos,
                HOME_MOVE_SCALE,
                HOME_MOVE_MAX,
                HOME_MOVE_MIN,
            );
            if rest < HOME_MOVE_DONE {
                ctx.end_staff_cut();
            }
        }
    }

    /// `daNpc_Nz_c::cutChkGetItemStart` (`d_a_npc_nz_cut.inc:378-382`).
    fn cut_chk_get_item_start(&mut self, ctx: &mut Ctx<'_>) {
        if self.order >= ORDER_GETITEM {
            let slot = self.esa_event;
            ctx.order(slot);
        }
    }

    /// `daNpc_Nz_c::_execute` (`:816-832`), minus the renderer, `setMtx`,
    /// `TailControl` and `mAcch.CrrPos`.
    pub fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.check_order(ctx.talk_command);
        ctx.actor_pos = actor.pos;
        ctx.actor_angle = actor.angle;
        self.mode_proc(PROC_EXEC, ctx);
        actor.pos = ctx.actor_pos;
        actor.angle = ctx.actor_angle;
        self.look_back(actor, ctx.player_eye);
        self.event_order();
        self.set_attention(actor.pos);
        self.shape_angle = actor.angle[1];
    }
}

/// `dComIfGs_checkGetBottle` (`d_com_inf_game.h:1962-1970`).
fn check_get_bottle(equipped: &Equipped) -> bool {
    (SLOT_BOTTLE0..SLOT_BOTTLE0 + SLOT_BOTTLE_COUNT).any(|slot| equipped.item(slot) != NO_ITEM)
}

/// `dSv_player_item_c::checkEmptyBottle` (`d_save.cpp:184-194`).
fn check_empty_bottle(equipped: &Equipped) -> u32 {
    (SLOT_BOTTLE0..SLOT_BOTTLE0 + SLOT_BOTTLE_COUNT)
        .filter(|slot| equipped.item(*slot) == ITEM_EMPTY_BOTTLE)
        .count() as u32
}

pub struct Text<'a> {
    pub procs: &'a mut MsgProcs,
    pub info: &'a mut MsgGameInfo,
    pub clear_count: u8,
    pub one: &'a mut Msg1Box,
    pub two: &'a mut Msg2Box,
    pub send: u8,
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

    /// `mpCurrMsg->mSelectNum`.
    pub fn select_num(&self) -> u8 {
        self.one.select_num
    }

    /// `dComIfGp_checkMesgSendButton`.
    pub fn send_button(&self) -> bool {
        self.send != 0
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
    pub player_eye: Option<Vec3>,
    pub player_pos: Option<Vec3>,
    pub player_angle: i16,
    pub actor_pos: Vec3,
    pub actor_angle: [i16; 3],
    pub talk_command: bool,
    pub new_mode: i32,
    pub teleport: Option<(Vec3, i16)>,
    pub message_rupee: i16,
    pub rupees: &'a mut Rupees,
    pub equipped: &'a Equipped,
    pub collect: &'a Collect,
    pub active: &'a mut ActiveEvent,
    pub list: &'a EventList,
    pub flags: &'a mut EventFlags,
    pub targets: &'a [Target],
    pub entity: Entity,
    pub text: &'a mut Text<'a>,
}

impl Ctx<'_> {
    fn end_check(&self, index: usize) -> bool {
        index != NO_EVENT && self.active.end_check(index)
    }

    fn end_staff_cut(&mut self) {
        self.active.end_staff_cut(self.list, self.flags, STAFF_NZ);
    }

    fn order(&mut self, index: usize) {
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
    pub rupees: ResMut<'w, Rupees>,
    pub equipped: Res<'w, Equipped>,
    pub collect: Res<'w, Collect>,
    pub list: Res<'w, EventList>,
    pub active: ResMut<'w, ActiveEvent>,
    pub flags: ResMut<'w, EventFlags>,
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
    others: Query<(Entity, &'static Actor), Without<NpcNz>>,
    mut actors: Query<(Entity, &mut Actor, &mut NpcNz)>,
) {
    let clear_count = world.text_flags.clear_count;
    let message_rupee = msg.info.message_rupee;
    let send = msg.one.m116a;
    let do_now = do_intent.0;
    let archive = world
        .archive
        .as_deref()
        .map(|held| held as &dyn MsgArchive);
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
    for (entity, mut actor, mut nz) in &mut actors {
        let tap = do_now && !nz.prev_do;
        nz.prev_do = do_now;
        let mut text = Text {
            procs: &mut msg.procs,
            info: &mut msg.info,
            clear_count,
            one: &mut msg.one,
            two: &mut msg.two,
            send,
            archive,
        };
        let mut ctx = Ctx {
            player_eye: player
                .map(|link| Vec3::new(link.pos.x, link.pos.y + PLAYER_HEAD_TOP, link.pos.z)),
            player_pos: player.map(|link| link.pos),
            player_angle: player.map_or(0, |link| walk::to_s16(link.yaw)),
            actor_pos: actor.pos,
            actor_angle: actor.angle,
            talk_command: tap,
            new_mode: MODE_WAIT,
            teleport: None,
            message_rupee,
            rupees: &mut world.rupees,
            equipped: &world.equipped,
            collect: &world.collect,
            active: &mut world.active,
            list: &world.list,
            flags: &mut world.flags,
            targets: &targets,
            entity,
            text: &mut text,
        };
        if !nz.entered {
            nz.create_init(&mut actor, &world.list);
        }
        nz.execute(&mut actor, &mut ctx);
        if let Some((pos, yaw)) = ctx.teleport
            && let Some((_, mut link)) = players.iter_mut().next()
        {
            link.pos = pos;
            link.yaw = walk::from_s16(yaw);
        }
    }
}