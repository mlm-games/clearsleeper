use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::daynight::{DayNight, TIME_DAY};
use super::event::{ActiveEvent, EventList};
use super::flags::{EventFlags, StoryBits};
use super::intent::DoIntent;
use super::player::Player;
use super::stream::add_calc_angle_s2;
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
    STATUS_BOX_CLOSED, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED,
    STATUS_MSG_ENDS,
};

pub const NPC_HO_ACTORS: [&str; 1] = ["Ho"];

pub fn is_npc_ho(name: &str) -> bool {
    NPC_HO_ACTORS.contains(&name)
}

// daNpc_Ho_c::HoFlags - d_a_npc_ho.h:12-18
const HO_FLAG_00000001: u16 = 0x0000_0001; // <- :13
const HO_FLAG_00000004: u16 = 0x0000_0004; // <- :14
const HO_FLAG_00000008: u16 = 0x0000_0008; // <- :15
const HO_FLAG_00000010: u16 = 0x0000_0010; // <- :16
const HO_FLAG_00000020: u16 = 0x0000_0020; // <- :17

// daNpc_Ho_c::HoStates - d_a_npc_ho.h:20-30
const HO_STATE_WAIT_01: i8 = 0; // <- :21
const HO_STATE_TALK_01: i8 = 1; // <- :22
const HO_STATE_TALK_02: i8 = 2; // <- :23
const HO_STATE_TALK_03: i8 = 3; // <- :24
const HO_STATE_TALK_03_CONTINUE: i8 = 4; // <- :25
const HO_STATE_GIVE_01: i8 = 5; // <- :26
const HO_STATE_GIVE_02: i8 = 6; // <- :27
const HO_STATE_PREACH: i8 = 7; // <- :28

// daNpc_Ho_c::ActionStatus - d_a_npc_ho.h:32-36
const ACTION_STARTING: i8 = 0; // <- :33
const ACTION_ENDING: i8 = -1; // <- :35

// daNpc_Ho_c::TalkStates - d_a_npc_ho.h:38-43
const TALK_INIT: i8 = 0; // <- :39
const TALK_MSG_CREATE: i8 = 1; // <- :40
const TALK_ACTIVE: i8 = 2; // <- :41
const TALK_FINISHED: i8 = -1; // <- :42

// l_bck_ix_tbl - :15-21, indexed by mCurrAnmIdx
const BCK_HO_WAIT01: u8 = 0; // <- dRes_INDEX_HO_BCK_HO_WAIT01_e, :16
const BCK_HO_TALK01: u8 = 1; // <- dRes_INDEX_HO_BCK_HO_TALK01_e, :17
const BCK_HO_TALK02: u8 = 2; // <- dRes_INDEX_HO_BCK_HO_TALK02_e, :18
const BCK_HO_GLAD: u8 = 3; // <- dRes_INDEX_HO_BCK_HO_GLAD_e, :19
const BCK_TABLE: [u8; 5] = [
    BCK_HO_WAIT01,
    BCK_HO_TALK01,
    BCK_HO_TALK02,
    BCK_HO_GLAD,
    BCK_HO_TALK02,
]; // <- :21

// daNpc_Ho_c::msgAnm msg_anm_table - :458-460
const MSG_ANM_TABLE: [i8; 5] = [0, 1, 2, 3, 4]; // <- :459

const NO_ANM: i8 = -1; // <- setAnm newAnmIdx != -1, :115
const NO_ITEM: u8 = 0xFF; // <- dItemNo_NONE_e, d_item_data.h:262
const ITEM_RED_RUPEE: u8 = 0x04; // <- dItemNo_RED_RUPEE_e, d_item_data.h:11
const ITEM_PURPLE_RUPEE: u8 = 0x05; // <- dItemNo_PURPLE_RUPEE_e, d_item_data.h:12
const ITEM_JOY_PENDANT: u8 = 0x1F; // <- dItemNo_JOY_PENDANT_e, d_item_data.h:38
const ITEM_HEROS_CHARM: u8 = 0x43; // <- dItemNo_HEROS_CHARM_e, d_item_data.h:74
const ITEM_CABANA_DEED: u8 = 0x9C; // <- dItemNo_CABANA_DEED_e, d_item_data.h:163

// receivePendant - :75
const PENDANT_TOTAL_MAX: i32 = 99; // <- totalGiven > 99, :75
// receivePendant(0x14) - :312
const PENDANT_BULK: i32 = 0x14; // <- :312

// dSv_event_flag_c ids - tww/include/d/d_save_event_flag.inc
const FLAG_1340: u16 = 0x1340; // <- UNK_1340, :195
const FLAG_1380: u16 = 0x1380; // <- UNK_1380, :232
const FLAG_1C04: u16 = 0x1C04; // <- UNK_1C04, :262, :329, :353
const FLAG_1C08: u16 = 0x1C08; // <- UNK_1C08, :264, :318, :409
const FLAG_1E01: u16 = 0x1E01; // <- UNK_1E01, :393
const FLAG_1E04: u16 = 0x1E04; // <- DEMO_SELECT(UNK_1E02, UNK_1E04) retail, :401, :620
const FLAG_1F80: u16 = 0x1F80; // <- UNK_1F80, :196
const FLAG_2201: u16 = 0x2201; // <- UNK_2201, :194
// dSv_event_tmp_flag_c ids - tww/include/d/d_save_event_tmp_flag.inc
const TMP_0104: u32 = 0x0104; // <- UNK_0104, :319, :414
const TMP_0580: u32 = 0x0580; // <- UNK_0580, :203
// dComIfGs_getEventReg(dSv_event_flag_c::UNK_C0FF) - :71, :329, :398, :405
const REG_C0FF: u16 = 0xC0FF; // <- :71
const REG_C0FF_TRADE: u8 = 0x28; // <- getEventReg(UNK_C0FF) >= 0x28, :329

// init - :529-556
const ATTN_BASE_Y: f32 = 100.0; // <- mAttnBasePos.y += 100.0f, :544
const ATTN_POS_Y: f32 = 50.0; // <- mAttnBasePos.y + 50.0f, :547
const CYL_RADIUS_INIT: f32 = 40.0; // <- mCylCollisionRadius = 40.0f, :552, :641, :670
// setAttention - :559-566
const ATTN_POS_Y_SET: f32 = 90.0; // <- mAttnBasePos.y + 90.0f, :565
// talk02 - :648-656
const CYL_RADIUS_MAX: f32 = 90.0; // <- mCylCollisionRadius < 90.0f, :652
const CYL_RADIUS_STEP: f32 = 5.0; // <- mCylCollisionRadius += 5.0f, :653
// talk01 / talk03 - :640, :669
const ATTENTION_TIMER_SET: u8 = 5; // <- mAttentionTimer = 5, :640

// _execute m_jnt.setParam - :786-796
const JNT_MAX_BACKBONE_X: i16 = 200; // <- :787
const JNT_MAX_BACKBONE_Y: i16 = 0x9C4; // <- :788
const JNT_MIN_BACKBONE_X: i16 = -500; // <- :789
const JNT_MIN_BACKBONE_Y: i16 = -0x9C4; // <- :790
const JNT_MAX_HEAD_X: i16 = 8000; // <- :791
const JNT_MAX_HEAD_Y: i16 = 8000; // <- :792
const JNT_MIN_HEAD_X: i16 = -0x9C4; // <- :793
const JNT_MIN_HEAD_Y: i16 = -8000; // <- :794
const JNT_MAX_TURN_STEP: i16 = 0x3E8; // <- :795

// lookBack - :598
const LOOK_VEL_TARGET: i16 = 0x5DC; // <- :598
const LOOK_VEL_SCALE: i16 = 4; // <- :598
const LOOK_VEL_MAX: i16 = 0x800; // <- :598

const ANM_LOOP_LIMIT: u8 = 2; // <- mAnmLoopCount < 2, :808, :816
const ANM_MORF: f32 = 8.0; // <- setAnm f32 morf = 8.0f, :114

// playTexPatternAnm - :105
const BTP_TIMER_MIN: f32 = 30.0; // <- :105
const BTP_TIMER_RANGE: f32 = 100.0; // <- cM_rndF(100.0f), :105
const BTP_FRAME_MAX: u8 = 0xFF; // <- mBlinkFrame >= getFrameMax() stand-in, :103

const JNT_SCALE: i16 = 4; // <- d_npc.cpp follow angCalcS(..., 4, maxVel), d_npc.cpp:47
const JNT_ANG_MIN_STEP: i16 = 0x60; // <- cLib_addCalcAngleS min_step, d_npc.cpp:23
const JNT_MIN_STEP: i32 = 1; // <- cLib_addCalcAngleL min_step, d_npc.cpp:78
const PLAYER_EYE_OFFSET_Y: f32 = -20.0; // <- dNpc_playerEyePos(-20.0f), :579, :590
const PLAYER_HEAD_TOP: f32 = 40.0; // <- d_a_player_main.cpp:11575

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

// fopAc_CULLBOX_12_e - f_op_actor_mng.cpp:570
const CULL_MIN: [f32; 3] = [-70.0, -100.0, -80.0];
const CULL_MAX: [f32; 3] = [70.0, 240.0, 100.0];

const NO_STATUS: u8 = 0xFF; // <- talk u16 msgStatus = 0xFF, :479
const NO_EVENT: usize = usize::MAX; // <- dComIfGp_evmng_getEventIdx miss
const ANM_TAG_NONE: u8 = 0xFF; // <- talkInit mMsgAnmIdx = 0xff, :474

const EVENT_PREACH: &str = "HO_PREACH"; // <- fopAcM_orderChangeEvent name, :632, :702
const EVENT_GIVEITEM: &str = "DEFAULT_GIVEITEM"; // <- :663, :689
const EVENT_TALK: &str = "DEFAULT_TALK"; // <- :690, :703
const STAFF_GIVEMAN: &str = "GIVEMAN"; // <- dEvt_control_c::giveItemCut, d_event.cpp:777

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = seed[0] as f32 / RND_SCALE[0]
        + seed[1] as f32 / RND_SCALE[1]
        + seed[2] as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

fn calc_timer(value: &mut i16) -> i16 {
    if *value != 0 {
        *value -= 1;
    }
    *value
}

/// `mDoExt_McaMorf::setMorf` (`m_Do_ext.cpp:1452-1460`) and `::isMorf`
/// (`m_Do_ext.h:418`) - the only parts of the morph blend `daNpc_Ho_c` observes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Morf {
    pub cur: f32,
    pub prev: f32,
    pub step: f32,
}

impl Morf {
    /// `mDoExt_McaMorf` ctor: `setAnm(..., 0.0f, ...)` then `mPrevMorf = -1.0f`
    /// (`m_Do_ext.cpp:1437`, `:1311-1312`).
    pub fn create() -> Self {
        Self {
            cur: 1.0,
            prev: -1.0,
            step: 0.0,
        }
    }

    fn set(&mut self, morf: f32) {
        if self.prev < 0.0 || morf <= 0.0 {
            self.cur = 1.0;
        } else {
            self.cur = 0.0;
            self.step = 1.0 / morf;
        }
        self.prev = self.cur;
    }

    /// `mDoExt_McaMorf::play` (`m_Do_ext.cpp:1463-1467`), minus `mFrameCtrl.update()`.
    fn play(&mut self) {
        if self.cur < 1.0 {
            self.prev = self.cur;
            walk::chase_f(&mut self.cur, 1.0, self.step);
        }
    }

    fn is_morf(&self) -> bool {
        self.cur < 1.0
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

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct NpcHo {
    pub entered: bool,
    pub prev_do: bool,
    pub action_status: i8,
    pub state: i8,
    pub prev_state: i8,
    pub talk_state: i8,
    pub flags: u16,
    pub order_flags: u8,
    pub attention_timer: u8,
    pub msg_select_num: u8,
    pub msg_anm_idx: u8,
    pub anm_idx: i8,
    pub anm_loop_count: u8,
    pub bck_idx: u8,
    pub blink_frame: u8,
    pub blink_timer: i16,
    pub attn_set_count: u8,
    pub item_num: u8,
    pub next_message_id: i32,
    pub curr_msg_no: u32,
    pub msg_id: u32,
    pub msg_bound: bool,
    pub get_item_no: u8,
    pub max_head_turn_velocity: i16,
    pub home_angle: i16,
    pub shape_angle: i16,
    pub cyl_radius: f32,
    pub eye_pos: Vec3,
    pub attn_base_pos: Vec3,
    pub attn_pos: Vec3,
    pub seed: [i32; 3],
    pub give_event: usize,
    pub preach_event: usize,
    pub talk_event: usize,
    pub morf: Morf,
    pub jnt: JntCtrl,
}

impl NpcHo {
    pub fn spawn(parameters: u32) -> Self {
        let _ = parameters;
        Self {
            entered: false,
            prev_do: false,
            action_status: ACTION_STARTING,
            state: HO_STATE_WAIT_01,
            prev_state: HO_STATE_WAIT_01,
            talk_state: TALK_INIT,
            flags: 0,
            order_flags: 0,
            attention_timer: 0,
            msg_select_num: 0,
            msg_anm_idx: ANM_TAG_NONE,
            anm_idx: NO_ANM,
            anm_loop_count: 0,
            bck_idx: BCK_HO_WAIT01,
            blink_frame: 0,
            blink_timer: 0,
            attn_set_count: 0,
            item_num: NO_ITEM,
            next_message_id: 0,
            curr_msg_no: 0,
            msg_id: FPC_M_ERROR_PROCESS_ID,
            msg_bound: false,
            get_item_no: 0,
            max_head_turn_velocity: 0,
            home_angle: 0,
            shape_angle: 0,
            cyl_radius: 0.0,
            eye_pos: Vec3::ZERO,
            attn_base_pos: Vec3::ZERO,
            attn_pos: Vec3::ZERO,
            seed: [RND_SEED; 3],
            give_event: NO_EVENT,
            preach_event: NO_EVENT,
            talk_event: NO_EVENT,
            morf: Morf::create(),
            jnt: JntCtrl::create(),
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    fn chk_flag(&self, flag: u16) -> bool {
        self.flags & flag == flag
    }

    fn set_flag(&mut self, flag: u16) {
        self.flags |= flag;
    }

    fn clr_flag(&mut self, flag: u16) {
        self.flags &= !flag;
    }

    fn chk_order(&self, flag: u8) -> bool {
        self.order_flags & flag != 0
    }

    fn set_order(&mut self, flag: u8) {
        self.order_flags |= flag;
    }

    fn clr_order(&mut self) {
        self.order_flags = 0;
    }

    /// `daNpc_Ho_c::init` (`:529-556`), minus the `mStts`/`mCyl` registration, the
    /// `eventInfo.setXyCheckCB` callback (an event-order hook, not sim state) and
    /// `dComIfGp_getReverb`. `setAction(&wait_action)` (`:537`) runs the
    /// `ACTION_STARTING` arm of `wait_action` once (`:713-716`).
    pub fn create_init(&mut self, actor: &mut Actor, list: &EventList) {
        self.home_angle = actor.angle[1];
        self.shape_angle = actor.angle[1];
        self.attn_base_pos = actor.pos;
        self.attn_base_pos.y += ATTN_BASE_Y;
        self.eye_pos = self.attn_base_pos;
        self.attn_pos = Vec3::new(
            self.attn_base_pos.x,
            self.attn_base_pos.y + ATTN_POS_Y,
            self.attn_base_pos.z,
        );
        self.cyl_radius = CYL_RADIUS_INIT;
        self.give_event = list.event_index(EVENT_GIVEITEM).unwrap_or(NO_EVENT);
        self.preach_event = list.event_index(EVENT_PREACH).unwrap_or(NO_EVENT);
        self.talk_event = list.event_index(EVENT_TALK).unwrap_or(NO_EVENT);
        self.state = HO_STATE_WAIT_01;
        self.set_anm_status();
        self.action_status += 1;
        self.entered = true;
    }

    /// `daNpc_Ho_c::setAnm` (`:113-120`), minus `dNpc_setAnm`'s frame-count
    /// dependency; the morf blend it installs is kept.
    fn set_anm(&mut self, new_anm_idx: i8) {
        if new_anm_idx == self.anm_idx || new_anm_idx == NO_ANM {
            return;
        }
        self.anm_idx = new_anm_idx;
        let slot = new_anm_idx.max(0) as usize;
        self.bck_idx = BCK_TABLE.get(slot).copied().unwrap_or(BCK_HO_WAIT01);
        self.morf.set(ANM_MORF);
    }

    /// `daNpc_Ho_c::setAnmStatus` (`:123-125`).
    fn set_anm_status(&mut self) {
        self.set_anm(0);
    }

    /// `daNpc_Ho_c::playTexPatternAnm` (`:101-110`), minus the `getFrameMax()`
    /// bound (the BTP archive does not ship).
    fn play_tex_pattern_anm(&mut self) {
        if calc_timer(&mut self.blink_timer) == 0 {
            if self.blink_frame >= BTP_FRAME_MAX {
                self.blink_frame = self.blink_frame.wrapping_sub(BTP_FRAME_MAX);
                self.blink_timer = (rnd(&mut self.seed) * BTP_TIMER_RANGE + BTP_TIMER_MIN) as i16;
            } else {
                self.blink_frame = self.blink_frame.wrapping_add(1);
            }
        }
    }

    /// `daNpc_Ho_c::chkAttentionLocal` (`:128-145`), minus the `dAttention_c`
    /// lock-on / action-target queries.
    fn chk_attention_local(&mut self, attention: bool) -> bool {
        if self.chk_flag(HO_FLAG_00000001 | HO_FLAG_00000010) {
            return true;
        }
        if self.attention_timer != 0 {
            self.attention_timer -= 1;
            return true;
        }
        attention
    }

    /// `daNpc_Ho_c::chkAttention` (`:148-160`).
    fn chk_attention(&mut self, attention: bool) {
        let temp = self.chk_flag(HO_FLAG_00000004);
        if self.chk_attention_local(attention) {
            self.set_flag(HO_FLAG_00000004);
        } else {
            self.clr_flag(HO_FLAG_00000004);
        }
        if temp != self.chk_flag(HO_FLAG_00000004) && temp {
            self.jnt.set_trn();
        }
    }

    /// `daNpc_Ho_c::eventOrder` (`:163-174`). `eventInfo.onCondition` and
    /// `fopAcM_orderSpeakEvent` only raise the CANTALK condition the sim's
    /// `DoIntent` press already stands in for, so nothing changes here.
    fn event_order(&mut self) {}

    /// `daNpc_Ho_c::checkOrder` (`:177-187`).
    fn check_order(&mut self, demo_accept: bool, talk_command: bool, talk_xy: bool) {
        if !demo_accept && talk_command && self.chk_order(7) {
            if talk_xy {
                self.set_flag(HO_FLAG_00000010);
            } else {
                self.set_flag(HO_FLAG_00000001);
            }
            self.talk_init();
        }
        self.clr_order();
    }

    /// `daNpc_Ho_c::talkInit` (`:472-475`).
    fn talk_init(&mut self) {
        self.talk_state = TALK_INIT;
        self.msg_anm_idx = ANM_TAG_NONE;
    }

    /// `daNpc_Ho_c::msgPushButton` (`:445-454`).
    fn msg_push_button(&mut self, select_num: u8) {
        match self.curr_msg_no {
            0x2712 | 0x2713 | 0x2718 | 0x272B => self.msg_select_num = select_num,
            _ => {}
        }
    }

    /// `daNpc_Ho_c::msgAnm` (`:457-469`).
    fn msg_anm(&mut self, attr: u8) {
        if self.msg_anm_idx != attr {
            self.msg_anm_idx = attr;
            if self.msg_anm_idx < 5 {
                let idx = MSG_ANM_TABLE
                    .get(usize::from(self.msg_anm_idx))
                    .copied()
                    .unwrap_or(0);
                self.set_anm(idx);
            }
            self.anm_loop_count = 0;
        }
    }

    /// `daNpc_Ho_c::next_msg_sub0` (`:190-218`).
    fn next_msg_sub0(&mut self, ctx: &mut Ctx<'_>) -> u32 {
        match self.msg_select_num {
            0 => {
                if ctx.stories.test(FLAG_2201) {
                    if ctx.stories.test(FLAG_1340) {
                        if ctx.stories.test(FLAG_1F80) {
                            return 0x2720;
                        }
                        ctx.stories.set(FLAG_1F80);
                        if ctx.day_night != TIME_DAY {
                            ctx.tmp.set(TMP_0580);
                        }
                        0x271E
                    } else {
                        0x271D
                    }
                } else {
                    0x2716
                }
            }
            _ => 0x2714,
        }
    }

    /// `daNpc_Ho_c::next_msgStatus` (`:221-383`).
    fn next_msg_status(&mut self, msg_no: &mut u32, ctx: &mut Ctx<'_>) -> u8 {
        let mut msg_status = STATUS_MSG_CONTINUES;
        match *msg_no {
            0x2712 | 0x2713 => {
                *msg_no = self.next_msg_sub0(ctx);
            }
            0x2718 => match self.msg_select_num {
                0 => {
                    *msg_no = 0x271A;
                    ctx.stories.set(FLAG_1380);
                }
                _ => {
                    *msg_no = 0x2719;
                }
            },
            0x271E => {
                self.next_message_id = 0x2748;
                self.item_num = ITEM_PURPLE_RUPEE;
                *msg_no = msg_no.wrapping_add(1);
                self.state = HO_STATE_TALK_03;
            }
            0x272B => match self.msg_select_num {
                0 => {
                    *msg_no = 0x272C;
                }
                1 => {
                    *msg_no = 0x272E;
                }
                _ => {
                    *msg_no = 0x2732;
                }
            },
            0x2732 => {
                *msg_no = 0x2736;
            }
            0x272C => {
                if ctx.stories.test(FLAG_1C04) {
                    *msg_no = 0x2758;
                } else if ctx.stories.test(FLAG_1C08) {
                    *msg_no = 0x2759;
                } else {
                    *msg_no = 0x272D;
                }
            }
            0x2725 => {
                self.set_flag(HO_FLAG_00000020);
                self.next_message_id = 0x2726;
                msg_status = STATUS_MSG_ENDS;
            }
            0x2716 | 0x2717 | 0x271A | 0x271B | 0x2720 | 0x2724 | 0x2726 | 0x2727 | 0x2728
            | 0x272E | 0x272F | 0x2730 | 0x2736 | 0x2737 => {
                *msg_no = msg_no.wrapping_add(1);
            }
            0x273D => {
                *msg_no = 0x275B;
                self.receive_pendant(1, ctx);
            }
            0x2749 => {
                *msg_no = 0x273E;
            }
            0x275C => {
                self.next_message_id = 0x275D;
                self.item_num = ITEM_RED_RUPEE;
                msg_status = STATUS_MSG_ENDS;
            }
            0x2742 => {
                *msg_no = 0x2743;
                self.receive_pendant(PENDANT_BULK, ctx);
            }
            0x2743 => {
                self.next_message_id = 0x2744;
                self.item_num = ITEM_CABANA_DEED;
                msg_status = STATUS_MSG_ENDS;
                ctx.stories.set(FLAG_1C08);
                ctx.tmp.set(TMP_0104);
            }
            0x274B => {
                *msg_no = 0x274D;
            }
            0x274E => {
                if ctx.stories.reg(REG_C0FF) >= REG_C0FF_TRADE && !ctx.stories.test(FLAG_1C04)
                {
                    *msg_no = 0x274F;
                } else {
                    msg_status = STATUS_MSG_ENDS;
                }
            }
            0x2751 => {
                self.next_message_id = 0x2752;
                self.item_num = ITEM_HEROS_CHARM;
                msg_status = STATUS_MSG_ENDS;
                ctx.stories.set(FLAG_1C04);
            }
            0x2756 => {
                msg_status = STATUS_MSG_ENDS;
            }
            0x273A | 0x274C => {
                *msg_no = msg_no.wrapping_add(1);
            }
            0x273B | 0x273C | 0x273F | 0x2741 | 0x2744 | 0x2745 | 0x2746 | 0x2747 | 0x274A
            | 0x274F | 0x2750 | 0x2752 | 0x2755 | 0x275B => {
                *msg_no = msg_no.wrapping_add(1);
            }
            _ => {
                msg_status = STATUS_MSG_ENDS;
            }
        }
        msg_status
    }

    /// `daNpc_Ho_c::receivePendant` (`:70-82`), minus the two
    /// `dComIfGp_setItemBeastNumCount(7, ...)` calls (`:72`, `:77`).
    fn receive_pendant(&mut self, given: i32, ctx: &mut Ctx<'_>) {
        let mut total = i32::from(ctx.stories.reg(REG_C0FF)) + given;
        if total > PENDANT_TOTAL_MAX {
            total = PENDANT_TOTAL_MAX;
        }
        ctx.stories.set_reg(REG_C0FF, total as u8);
    }

    /// `daNpc_Ho_c::getMsg` (`:386-429`).
    fn get_msg(&mut self, ctx: &mut Ctx<'_>) -> u32 {
        match self.state {
            HO_STATE_TALK_01 => {
                if ctx.night_stop {
                    return 0x2722;
                }
                if !ctx.stories.test(FLAG_1E01) {
                    ctx.stories.set(FLAG_1E01);
                    return 0x2711;
                }
                if ctx.stories.reg(REG_C0FF) == 0 {
                    0x2712
                } else {
                    0x2713
                }
            }
            HO_STATE_TALK_03_CONTINUE => self.next_message_id as u32,
            _ => 0,
        }
    }

    /// `daNpc_Ho_c::talk` (`:478-526`).
    fn talk(&mut self, ctx: &mut Ctx<'_>) -> u8 {
        let mut status = NO_STATUS;
        if self.talk_state == TALK_INIT {
            self.msg_id = FPC_M_ERROR_PROCESS_ID;
            self.curr_msg_no = self.get_msg(ctx);
            self.talk_state = TALK_MSG_CREATE;
        } else if self.talk_state != TALK_FINISHED {
            if self.msg_id == FPC_M_ERROR_PROCESS_ID {
                let no = self.curr_msg_no;
                let eye = self.eye_pos;
                self.msg_id = ctx.text.message_set_actor(no, eye);
            } else {
                if !self.chk_flag(HO_FLAG_00000008) {
                    let attr = ctx.text.mesg_anime();
                    self.msg_anm(attr);
                }
                match self.talk_state {
                    TALK_MSG_CREATE => {
                        self.msg_bound = ctx.text.procs.search_by_id(self.msg_id).is_some();
                        if self.msg_bound {
                            self.talk_state = TALK_ACTIVE;
                        }
                    }
                    TALK_ACTIVE => {
                        if self.msg_bound {
                            status = ctx.text.status().unwrap_or(NO_STATUS);
                            if status == STATUS_MSG_DISPLAYED {
                                let select = ctx.text.select_num();
                                self.msg_push_button(select);
                                let mut no = self.curr_msg_no;
                                let next = self.next_msg_status(&mut no, ctx);
                                self.curr_msg_no = no;
                                ctx.text.set_status(next);
                                if next == STATUS_MSG_CONTINUES {
                                    let no = self.curr_msg_no;
                                    ctx.text.message_set_actor(no, self.eye_pos);
                                }
                            } else if status == STATUS_BOX_CLOSED {
                                ctx.text.set_status(STATUS_MSG_DESTROYED);
                                self.talk_state = TALK_FINISHED;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        status
    }

    /// `daNpc_Ho_c::setAttention` (`:559-566`).
    fn set_attention(&mut self, param_1: bool) {
        if !param_1 && self.attn_set_count >= 2 {
            return;
        }
        self.eye_pos = Vec3::new(self.eye_pos.x, self.eye_pos.y, self.eye_pos.z);
        self.attn_pos = Vec3::new(
            self.attn_base_pos.x,
            self.attn_base_pos.y + ATTN_POS_Y_SET,
            self.attn_base_pos.z,
        );
    }

    /// `daNpc_Ho_c::lookBack` (`:569-605`).
    fn look_back(&mut self, actor: &mut Actor, ctx: &Ctx<'_>) {
        let mut dst = None;
        let mut desired_y_rot = actor.angle[1];
        let mut head_only_follow = false;
        let mut src = Vec3::new(actor.pos.x, self.eye_pos.y, actor.pos.z);
        if self.state == HO_STATE_WAIT_01 {
            if self.chk_flag(HO_FLAG_00000004) {
                dst = ctx.player_eye;
                src = Vec3::new(actor.pos.x, self.eye_pos.y, actor.pos.z);
                head_only_follow = true;
            } else {
                desired_y_rot = self.home_angle;
            }
        } else {
            self.jnt.set_trn();
            dst = ctx.player_eye;
            src = Vec3::new(actor.pos.x, self.eye_pos.y, actor.pos.z);
        }
        if self.jnt.trn_chk() {
            add_calc_angle_s2(
                &mut self.max_head_turn_velocity,
                LOOK_VEL_TARGET,
                LOOK_VEL_SCALE,
                LOOK_VEL_MAX,
            );
        } else {
            self.max_head_turn_velocity = 0;
        }
        let vel = self.max_head_turn_velocity;
        self.jnt.look_at_target(
            &mut actor.angle[1],
            dst,
            src,
            desired_y_rot,
            vel,
            head_only_follow,
        );
        self.shape_angle = actor.angle[1];
    }

    /// `daNpc_Ho_c::wait01` (`:608-625`).
    fn wait01(&mut self, ctx: &Ctx<'_>) -> bool {
        if self.chk_flag(HO_FLAG_00000010) {
            self.prev_state = self.state;
            self.state = HO_STATE_TALK_02;
            self.set_anm_status();
            self.item_num = NO_ITEM;
        } else if self.chk_flag(HO_FLAG_00000001) {
            self.prev_state = self.state;
            self.state = HO_STATE_TALK_01;
            self.set_anm_status();
        } else {
            self.set_order(HO_FLAG_00000001 as u8);
            if ctx.stories.test(FLAG_1E04) && !ctx.night_stop {
                self.set_order(HO_FLAG_00000004 as u8);
            }
        }
        self.morf.is_morf()
    }

    /// `daNpc_Ho_c::talk01` (`:628-645`).
    fn talk01(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.talk(ctx) == STATUS_BOX_CLOSED {
            if self.chk_flag(HO_FLAG_00000020) {
                self.state = HO_STATE_PREACH;
                let slot = self.preach_event;
                ctx.order(slot);
                self.clr_flag(HO_FLAG_00000020);
            } else {
                self.state = self.prev_state;
                self.set_anm_status();
                self.clr_flag(HO_FLAG_00000001 | HO_FLAG_00000010);
                self.attention_timer = ATTENTION_TIMER_SET;
                self.cyl_radius = CYL_RADIUS_INIT;
            }
        }
        self.morf.is_morf()
    }

    /// `daNpc_Ho_c::talk02` (`:648-656`).
    fn talk02(&mut self, ctx: &Ctx<'_>) -> bool {
        if ctx.present_done {
            self.state = HO_STATE_TALK_03;
        }
        if self.cyl_radius < CYL_RADIUS_MAX {
            self.cyl_radius += CYL_RADIUS_STEP;
        }
        false
    }

    /// `daNpc_Ho_c::talk03` (`:659-675`).
    fn talk03(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.talk(ctx) == STATUS_BOX_CLOSED {
            if self.item_num != NO_ITEM {
                self.state = HO_STATE_GIVE_01;
                let slot = self.give_event;
                ctx.order(slot);
            } else {
                self.state = self.prev_state;
                self.set_anm_status();
                self.clr_flag(HO_FLAG_00000001 | HO_FLAG_00000010);
                self.attention_timer = ATTENTION_TIMER_SET;
                self.cyl_radius = CYL_RADIUS_INIT;
            }
        }
        self.morf.is_morf()
    }

    /// `daNpc_Ho_c::give01` (`:678-685`).
    fn give01(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let item = self.item_num;
        if ctx.give_item_cut(item) {
            self.get_item_no = item;
            self.state = HO_STATE_GIVE_02;
        }
        self.morf.is_morf()
    }

    /// `daNpc_Ho_c::give02` (`:688-696`).
    fn give02(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let slot = self.give_event;
        if ctx.end_check(slot) {
            let talk = self.talk_event;
            ctx.order(talk);
            self.state = HO_STATE_TALK_03_CONTINUE;
            self.item_num = NO_ITEM;
            self.talk_init();
        }
        self.morf.is_morf()
    }

    /// `daNpc_Ho_c::preach` (`:699-709`).
    fn preach(&mut self, ctx: &mut Ctx<'_>) -> bool {
        self.set_flag(HO_FLAG_00000004);
        let slot = self.preach_event;
        if ctx.end_check(slot) {
            let talk = self.talk_event;
            ctx.order(talk);
            self.item_num = NO_ITEM;
            self.state = HO_STATE_TALK_03_CONTINUE;
            self.talk_init();
        }
        self.morf.is_morf()
    }

    /// `daNpc_Ho_c::wait_action` (`:712-754`).
    fn wait_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        if self.action_status == ACTION_STARTING {
            self.state = HO_STATE_WAIT_01;
            self.set_anm_status();
            self.action_status += 1;
        } else if self.action_status != ACTION_ENDING {
            self.chk_attention(ctx.attention);
            let temp = match self.state {
                HO_STATE_WAIT_01 => self.wait01(ctx),
                HO_STATE_TALK_01 => self.talk01(ctx),
                HO_STATE_TALK_02 => self.talk02(ctx),
                HO_STATE_TALK_03 | HO_STATE_TALK_03_CONTINUE => self.talk03(ctx),
                HO_STATE_GIVE_01 => self.give01(ctx),
                HO_STATE_GIVE_02 => self.give02(ctx),
                HO_STATE_PREACH => self.preach(ctx),
                _ => false,
            };
            self.look_back(actor, ctx);
            self.set_attention(temp);
        }
    }

    /// `daNpc_Ho_c::_execute` (`:785-847`), minus the renderer, `fopAcM_posMoveF`,
    /// `mObjAcch.CrrPos`, the `mtrlSndId`/`tevStr` fetches, `setCollision` and the
    /// `mpMorf->getFrame()` end-frame test that needs BCK frame counts.
    pub fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.jnt.set_param(
            JNT_MAX_BACKBONE_X,
            JNT_MAX_BACKBONE_Y,
            JNT_MIN_BACKBONE_X,
            JNT_MIN_BACKBONE_Y,
            JNT_MAX_HEAD_X,
            JNT_MAX_HEAD_Y,
            JNT_MIN_HEAD_X,
            JNT_MIN_HEAD_Y,
            JNT_MAX_TURN_STEP,
        );
        self.play_tex_pattern_anm();
        self.morf.play();
        let demo_accept = ctx.demo_accept;
        let talk_command = ctx.talk_command;
        self.check_order(demo_accept, talk_command, ctx.talk_xy);
        self.wait_action(actor, ctx);
        self.event_order();
        self.shape_angle = actor.angle[1];
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

    /// `l_msg->mSelectNum`.
    pub fn select_num(&self) -> u8 {
        self.one.select_num
    }

    pub fn mesg_anime(&self) -> u8 {
        self.info.mesg_anime
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
    pub attention: bool,
    pub night_stop: bool,
    pub present_done: bool,
    pub talk_xy: bool,
    pub talk_command: bool,
    pub demo_accept: bool,
    pub day_night: i32,
    pub stories: &'a mut StoryBits,
    pub tmp: &'a mut EventFlags,
    pub active: &'a mut ActiveEvent,
    pub list: &'a EventList,
    pub entity: Entity,
    pub text: &'a mut Text<'a>,
}

impl Ctx<'_> {
    /// `fopAcM_orderChangeEvent` (`f_op_actor_mng.cpp:705-708`), guarded so a
    /// re-order every tick cannot reset `Stage::Order`.
    fn order(&mut self, index: usize) {
        if index == NO_EVENT {
            return;
        }
        if self.active.stage_of(index).is_none() {
            self.active.order(self.list, index, self.entity);
        }
    }

    /// `dComIfGp_evmng_endCheck(name)` (`d_com_inf_game.h:3663-3665`).
    fn end_check(&self, index: usize) -> bool {
        index != NO_EVENT && self.active.end_check(index)
    }

    /// `dEvt_control_c::giveItemCut` (`d_event.cpp:776-785`).
    fn give_item_cut(&mut self, item: u8) -> bool {
        if self.active.staff_cut(self.list, STAFF_GIVEMAN).is_none() {
            return false;
        }
        self.active.end_staff_cut(self.list, self.tmp, STAFF_GIVEMAN);
        let _ = item;
        true
    }
}

#[derive(SystemParam)]
pub struct Env<'w> {
    pub text_flags: Res<'w, TextFlags>,
    pub stories: ResMut<'w, StoryBits>,
    pub tmp: ResMut<'w, EventFlags>,
    pub daynight: Res<'w, DayNight>,
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
    mut actors: Query<(Entity, &mut Actor, &mut NpcHo)>,
) {
    let player = msg.players.iter().next().copied();
    let clear_count = world.text_flags.clear_count;
    let demo_accept = msg.info.event_run_check();
    let night_stop = world.daynight.night_stop;
    let day_night = world.daynight.daynight();
    let do_now = do_intent.0;
    let archive = world
        .archive
        .as_deref()
        .map(|held| held as &dyn MsgArchive);
    for (entity, mut actor, mut ho) in &mut actors {
        let tap = do_now && !ho.prev_do;
        ho.prev_do = do_now;
        let mut text = Text {
            procs: &mut msg.procs,
            info: &mut msg.info,
            clear_count,
            one: &mut msg.one,
            two: &mut msg.two,
            archive,
        };
        let mut ctx = Ctx {
            player_eye: player.map(|link| {
                Vec3::new(
                    link.pos.x,
                    link.pos.y + PLAYER_HEAD_TOP + PLAYER_EYE_OFFSET_Y,
                    link.pos.z,
                )
            }),
            attention: false,
            night_stop,
            present_done: false,
            talk_xy: false,
            talk_command: tap,
            demo_accept,
            day_night,
            stories: &mut world.stories,
            tmp: &mut world.tmp,
            active: &mut world.active,
            list: &world.list,
            entity,
            text: &mut text,
        };
        if !ho.entered {
            ho.create_init(&mut actor, &world.list);
        }
        ho.execute(&mut actor, &mut ctx);
    }
}