use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::flags::EventFlags;
use super::intent::DoIntent;
use super::player::Player;
use super::stream::add_calc_angle_s2;
use super::walk::{chase_f, to_s16};
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

pub const NPC_TT_ACTORS: [&str; 1] = ["Tt"];

pub fn is_npc_tt(name: &str) -> bool {
    NPC_TT_ACTORS.contains(&name)
}

const STATE_NONE: i8 = 0; // <- zeroed mState before wait_action sets 1, :667
const STATE_WAIT: i8 = 1; // <- wait_action ACTION_STARTING, :667
const STATE_TALK: i8 = 2; // <- wait01 chkFlag(0x2), :621
const STATE_TACT: i8 = 3; // <- wait01 chkFlag(0x4), :612

const TALK_FINISHED: i8 = -1; // <- daNpc_Tt_c::TalkStates::TALK_FINISHED, d_a_npc_tt.h:29
const TALK_INIT: i8 = 0; // <- :30
const TALK_MSG_CREATE: i8 = 1; // <- :31
const TALK_ACTIVE: i8 = 2; // <- :32

const ACTION_STARTING: i8 = 0; // <- daNpc_Tt_c::ActionStatus::ACTION_STARTING, :25
const ACTION_ENDING: i8 = -1; // <- :23

const ORDER_TALK_DEMO: u8 = 0x7; // <- checkOrder ChkOrder(7), :114
const ORDER_TALK: u8 = 0x2; // <- eventOrder ChkOrder(2) / wait01 SetOrder(0x2), :103 / :623
const ORDER_TALK_ITEM: u8 = 0x4; // <- eventOrder ChkOrder(4) / wait01 SetOrder(0x4), :106 / :625

const FLAG_HOME_FOLLOW: u16 = 0x20; // <- getMsg clrFlag(0x20) / lookBack chkFlag(0x20), :157 / :584
const FLAG_TALK: u16 = 0x2; // <- checkOrder setFlag(0x2), :118
const FLAG_TALK_ITEM: u16 = 0x4; // <- checkOrder setFlag(0x4), :116
const FLAG_HIDE: u16 = 0x8; // <- wait_action clrFlag(0x8), :671
const FLAG_DANCE: u16 = 0x40; // <- setAnm clrFlag(0x40), d_a_npc_tt_anm.inc:93
const FLAG_DANCE_SOUND: u16 = 0x100; // <- danceNext chkFlag(0x100), :178

const FLAG_0C40: u32 = 0x0C40; // <- dSv_event_flag_c::UNK_0C40, :73
const FLAG_0B08: u32 = 0x0B08; // <- dSv_event_flag_c::UNK_0B08, :163

const MSG_22C5: u32 = 0x22C5; // <- getMsg, :168
const MSG_22CA: u32 = 0x22CA; // <- :164
const MSG_22CB: u32 = 0x22CB; // <- :161
const MSG_22C6: u32 = 0x22C6; // <- next_msgStatus, :139
const MSG_22C7: u32 = 0x22C7; // <- :133
const MSG_22C8: u32 = 0x22C8; // <- :133
const MSG_22D5: u32 = 0x22D5; // <- :135
const MSG_22D6: u32 = 0x22D6; // <- :135
const MSG_22D7: u32 = 0x22D7; // <- :142
const MSG_5B3: u32 = 0x5B3; // <- getMsg, :176
const MELODY_5: u8 = 5; // <- dComIfGp_setMelodyNum(5), :177

const NOTICE_LINK_TIMER: i16 = 0xF; // <- msgPushButton, :234
const TACT_TIMER: i16 = 0x14; // <- wait01 mTimer = 0x14, :613

const ATTN_Y_OFFSET: f32 = 65.0; // <- setAttention, :562
const PLAYER_HEAD_TOP: f32 = 40.0; // <- d_a_player_main.cpp:11575
const PLAYER_EYE_OFFSET_Y: f32 = -20.0; // <- dNpc_playerEyePos(-20.0f), :589

const JNT_MAX_HEAD_X: i16 = 0x1FFE; // <- _execute m_jnt.setParam, :799
const JNT_MAX_HEAD_Y: i16 = 9000; // <- :800
const JNT_MIN_HEAD_X: i16 = -0x1FFE; // <- :801
const JNT_MIN_HEAD_Y: i16 = -9000; // <- :802
const JNT_MAX_BACKBONE_Y: i16 = 0x1C70; // <- :796
const JNT_MIN_BACKBONE_Y: i16 = -0x1C70; // <- :798
const JNT_MAX_TURN_STEP: i16 = 0x1000; // <- :803
const JNT_SCALE: i16 = 4; // <- lookBack addCalcAngleS2 scale, :598
const JNT_MAX_VEL: i16 = 0x800; // <- :598
const JNT_MIN_STEP: i32 = 1; // <- d_npc.cpp dNpc_JntCtrl_c::move

const CULL_MIN: [f32; 3] = [-70.0, -100.0, -80.0]; // <- f_op_actor_mng.cpp:570 (CULLBOX_12)
const CULL_MAX: [f32; 3] = [70.0, 240.0, 100.0]; // <- :570

const ANM_MORF: f32 = 8.0; // <- d_a_npc_tt_anm.inc:84
const ANM_MORF_POSE: f32 = 7.0; // <- :87
const ANM_POSE_LO: i32 = 7; // <- newAnmIdx >= 7, :87
const ANM_POSE_HI: i32 = 31; // <- :87
const ANM_LOOP_HI: i32 = 5; // <- newAnmIdx < 5, :93

const DANCE_STEP_LOOP: u8 = 6; // <- danceNext mDanceStep < 6, :159
const DANCE_STEP_DEF: u8 = 8; // <- danceInit mDanceStepTimer = 8, :154
const DANCE_STEP_SLOW: u8 = 58; // <- danceNext case 4, :172
const DANCE_STEP_POSE: u8 = 53; // <- danceNext case 5, :176

const NO_STATUS: u8 = 0xFF; // <- talk, :252
const NO_ANM: i8 = -1; // <- setAnm anmIdx != -1, :97

fn target_angle_y(lhs: Vec3, rhs: Vec3) -> i16 {
    to_s16((rhs.x - lhs.x).atan2(rhs.z - lhs.z))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Morf {
    pub cur: f32,
    pub prev: f32,
    pub step: f32,
}

impl Morf {
    /// `mDoExt_McaMorf` ctor: `setAnm(..., 0.0f, ...)` then `mPrevMorf = -1.0f`
    /// (`m_Do_ext.cpp:1311-1312`).
    pub fn create() -> Self {
        Self {
            cur: 1.0,
            prev: -1.0,
            step: 0.0,
        }
    }

    /// `mDoExt_McaMorf::setMorf` (`m_Do_ext.cpp:1452-1460`).
    fn set(&mut self, morf: f32) {
        if self.prev < 0.0 || morf <= 0.0 {
            self.cur = 1.0;
        } else {
            self.cur = 0.0;
            self.step = 1.0 / morf;
        }
        self.prev = self.cur;
    }

    /// `mDoExt_McaMorf::play` (`m_Do_ext.cpp:1463-1473`), minus `mFrameCtrl.update()`.
    fn play(&mut self) {
        if self.cur < 1.0 {
            self.prev = self.cur;
            chase_f(&mut self.cur, 1.0, self.step);
        }
    }

    /// `mDoExt_McaMorf::isMorf` (`m_Do_ext.h:366`).
    pub fn is_morf(&self) -> bool {
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

    pub fn clr_trn(&mut self) {
        self.trn = false;
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
        let mut diff = orig;
        add_calc_angle_s2(&mut diff, 0, speed, max_vel);
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
                (target_angle_y(src, point), to_s16(delta.y.atan2(flat)))
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
pub struct NpcTt {
    pub entered: bool,
    pub prev_do: bool,
    pub action_status: i8,
    pub state: i8,
    pub return_to_state: i8,
    pub talk_state: i8,
    pub order_flags: u8,
    pub msg_anm_idx: u8,
    pub dance_step: u8,
    pub dance_step_timer: u8,
    pub notice_link_timer: i16,
    pub flags: u16,
    pub anm_idx: i8,
    pub attn_set_count: u8,
    pub max_head_turn_velocity: i16,
    pub home_angle: [i16; 3],
    pub eye_pos: Vec3,
    pub attn_base_pos: Vec3,
    pub curr_msg_no: u32,
    pub msg_no: u32,
    pub msg_id: u32,
    pub msg_bound: bool,
    pub melody_num: u8,
    pub morf: Morf,
    pub jnt: JntCtrl,
}

impl NpcTt {
    pub fn spawn(parameters: u32) -> Self {
        let _ = parameters;
        Self {
            entered: false,
            prev_do: false,
            action_status: ACTION_STARTING,
            state: STATE_NONE,
            return_to_state: STATE_NONE,
            talk_state: TALK_FINISHED,
            order_flags: 0,
            msg_anm_idx: 0xFF,
            dance_step: 0,
            dance_step_timer: 0,
            notice_link_timer: 0,
            flags: 0,
            anm_idx: NO_ANM,
            attn_set_count: 0,
            max_head_turn_velocity: 0,
            home_angle: [0; 3],
            eye_pos: Vec3::ZERO,
            attn_base_pos: Vec3::ZERO,
            curr_msg_no: 0,
            msg_no: 0,
            msg_id: FPC_M_ERROR_PROCESS_ID,
            msg_bound: false,
            melody_num: 0,
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

    /// `daNpc_Tt_c::init` (`:523-553`), minus `ke_execute`, the collider registration and
    /// the two `eventInfo.setXy*CB` callbacks.
    pub fn create_init(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.attn_base_pos = actor.pos;
        self.eye_pos = actor.pos;
        self.home_angle = actor.angle;
        self.action_status = ACTION_STARTING;
        self.wait_action(actor, ctx);
        self.entered = true;
    }

    /// `daNpc_Tt_c::setAnm` (`d_a_npc_tt_anm.inc:78-100`), minus the play-speed and
    /// loop-mode values, which only reach `mFrameCtrl` via `mDoExt_McaMorf::setAnm`.
    pub fn set_anm(&mut self, new_anm_idx: i32) {
        let idx = new_anm_idx as i8;
        let mut morf = ANM_MORF;
        if (ANM_POSE_LO..=ANM_POSE_HI).contains(&new_anm_idx) {
            morf = ANM_MORF_POSE;
        }
        if new_anm_idx < ANM_LOOP_HI {
            self.flags &= !FLAG_DANCE;
        }
        if idx != self.anm_idx && idx != NO_ANM {
            self.anm_idx = idx;
            self.morf.set(morf);
        }
    }

    /// `daNpc_Tt_c::danceInit` (`d_a_npc_tt_anm.inc:103-110`).
    fn dance_init(&mut self, param: i32) {
        if param != 0 || self.flags & FLAG_DANCE == 0 {
            self.dance_step = 0;
            self.dance_step_timer = DANCE_STEP_DEF;
            self.set_anm(5);
            self.flags |= FLAG_DANCE;
        }
    }

    /// `daNpc_Tt_c::danceProc` (`d_a_npc_tt_anm.inc:113-128`), minus the
    /// `mpMorf->isMorf()` early-out and the `mHeadAnm.swing_vertical_init` call, both of
    /// which feed only the joint callback.
    fn dance_proc(&mut self) -> bool {
        if self.flags & FLAG_DANCE == 0 {
            return false;
        }
        if self.dance_step_timer != 0 {
            self.dance_step_timer = self.dance_step_timer.wrapping_sub(1);
            return false;
        }
        self.dance_next()
    }

    /// `daNpc_Tt_c::danceNext` (`d_a_npc_tt_anm.inc:131-182`), minus the
    /// `fopAcM_seStart(JA_SE_CV_TT_DANCE_FIN)` call.
    fn dance_next(&mut self) -> bool {
        if self.dance_step < DANCE_STEP_LOOP {
            self.dance_step = self.dance_step.wrapping_add(1);
        } else {
            self.dance_step = 0;
        }
        match self.dance_step {
            0 | 2 => {
                self.set_anm(5);
                self.dance_step_timer = DANCE_STEP_DEF;
            }
            1 => {
                self.set_anm(10);
                self.dance_step_timer = DANCE_STEP_DEF;
            }
            3 => {
                self.set_anm(9);
                self.dance_step_timer = DANCE_STEP_DEF;
            }
            4 => {
                self.set_anm(6);
                self.dance_step_timer = DANCE_STEP_SLOW;
            }
            5 => {
                self.flags &= !FLAG_DANCE_SOUND;
                self.set_anm(8);
                self.dance_step_timer = DANCE_STEP_POSE;
            }
            _ => {}
        }
        self.dance_step == 0
    }

    /// `daNpc_Tt_c::setAnmStatus` (`:69-85`).
    fn set_anm_status(&mut self, flags: &EventFlags) {
        match self.state {
            STATE_WAIT | STATE_TALK => {
                if flags.test(FLAG_0C40) {
                    self.dance_init(0);
                } else {
                    self.set_anm(4);
                }
            }
            STATE_TACT => {}
            _ => self.set_anm(0),
        }
    }

    /// `daNpc_Tt_c::msgAnm` (`:203-227`).
    fn msg_anm(&mut self, attr: u8) {
        if self.msg_anm_idx != attr {
            self.msg_anm_idx = attr;
            match self.msg_anm_idx {
                0 => self.set_anm(0),
                1 => self.set_anm(1),
                2 => self.set_anm(3),
                3 => self.set_anm(4),
                4 => self.dance_init(0),
                5 => self.set_anm(2),
                _ => {}
            }
        }
    }

    /// `daNpc_Tt_c::msgPushButton` (`:230-235`).
    fn msg_push_button(&mut self) {
        if self.curr_msg_no != MSG_22C6 {
            return;
        }
        self.notice_link_timer = NOTICE_LINK_TIMER;
    }

    /// `daNpc_Tt_c::msgContinue` (`:238-248`).
    fn msg_continue(&mut self, ctx: &mut Ctx<'_>) {
        if self.curr_msg_no == MSG_22D5 {
            if self.notice_link_timer > 0 {
                self.notice_link_timer -= 1;
            } else {
                ctx.text.message_send_on();
            }
        }
    }

    /// `daNpc_Tt_c::next_msgStatus` (`:127-151`).
    pub fn next_msg_status(&self, msg_no: &mut u32) -> u8 {
        match *msg_no {
            MSG_22C5 | MSG_22C7 | MSG_22C8 | MSG_22D5 | MSG_22D6 => {
                *msg_no += 1;
                STATUS_MSG_CONTINUES
            }
            MSG_22C6 => {
                *msg_no = MSG_22D5;
                STATUS_MSG_CONTINUES
            }
            MSG_22D7 => {
                *msg_no = MSG_22C7;
                STATUS_MSG_CONTINUES
            }
            _ => STATUS_MSG_ENDS,
        }
    }

    /// `daNpc_Tt_c::getMsg` (`:155-182`).
    fn get_msg(&mut self, ctx: &mut Ctx<'_>) -> u32 {
        self.flags &= !FLAG_HOME_FOLLOW;
        if self.state == STATE_TALK {
            if ctx.flags.test(FLAG_0C40) {
                MSG_22CB
            } else if ctx.flags.test(FLAG_0B08) {
                self.flags |= FLAG_HOME_FOLLOW;
                MSG_22CA
            } else {
                ctx.flags.set(FLAG_0B08);
                self.flags |= FLAG_HOME_FOLLOW;
                MSG_22C5
            }
        } else {
            let msg_no = self.msg_no;
            if msg_no == MSG_5B3 {
                self.melody_num = MELODY_5;
            }
            msg_no
        }
    }

    /// `daNpc_Tt_c::talkInit` (`:197-200`).
    fn talk_init(&mut self) {
        self.talk_state = TALK_INIT;
        self.msg_anm_idx = 0xFF;
    }

    /// `daNpc_Tt_c::talk` (`:251-292`).
    fn talk(&mut self, ctx: &mut Ctx<'_>) -> u8 {
        let mut msg_status = NO_STATUS;
        if self.talk_state == TALK_INIT {
            self.msg_id = FPC_M_ERROR_PROCESS_ID;
            self.curr_msg_no = self.get_msg(ctx);
            self.talk_state = TALK_MSG_CREATE;
        } else if self.talk_state != TALK_FINISHED {
            if self.msg_id == FPC_M_ERROR_PROCESS_ID {
                let no = self.curr_msg_no;
                self.msg_id = ctx.text.message_set(no);
            } else {
                let attr = ctx.text.mesg_anime();
                self.msg_anm(attr);
                match self.talk_state {
                    TALK_MSG_CREATE => {
                        self.msg_bound = ctx.text.procs.search_by_id(self.msg_id).is_some();
                        if self.msg_bound {
                            self.talk_state = TALK_ACTIVE;
                        }
                    }
                    TALK_ACTIVE => {
                        if self.msg_bound {
                            msg_status = ctx.text.status().unwrap_or(NO_STATUS);
                            if msg_status == STATUS_MSG_CONTINUES {
                                self.msg_continue(ctx);
                            } else if msg_status == STATUS_MSG_DISPLAYED {
                                self.msg_push_button();
                                let mut no = self.curr_msg_no;
                                let next = self.next_msg_status(&mut no);
                                self.curr_msg_no = no;
                                ctx.text.set_status(next);
                                if next == STATUS_MSG_CONTINUES {
                                    let want = self.curr_msg_no;
                                    ctx.text.message_set(want);
                                }
                            } else if msg_status == STATUS_BOX_CLOSED {
                                ctx.text.set_status(STATUS_MSG_DESTROYED);
                                self.msg_id = FPC_M_ERROR_PROCESS_ID;
                                self.msg_bound = false;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        msg_status
    }

    /// `daNpc_Tt_c::lookBack` (`:566-605`).
    fn look_back(&mut self, actor: &mut Actor, ctx: &Ctx<'_>) {
        let mut dst = None;
        let mut src = Vec3::ZERO;
        let mut desired_y_rot = actor.angle[1];
        let head_only_follow = false;
        if self.state == STATE_WAIT {
            desired_y_rot = self.home_angle[1];
            if desired_y_rot != actor.angle[1] {
                self.jnt.set_trn();
            } else {
                self.jnt.clr_trn();
            }
        } else if self.flags & FLAG_HOME_FOLLOW != 0 {
            desired_y_rot = self.home_angle[1];
            self.jnt.clr_trn();
        } else {
            self.jnt.set_trn();
            dst = ctx.player_eye;
            src = Vec3::new(actor.pos.x, self.eye_pos.y, actor.pos.z);
        }
        if self.jnt.trn_chk() {
            add_calc_angle_s2(
                &mut self.max_head_turn_velocity,
                JNT_MAX_VEL,
                JNT_SCALE,
                JNT_MAX_VEL,
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
    }

    /// `daNpc_Tt_c::setAttention` (`:556-563`).
    pub fn set_attention(&mut self, set_eye_pos: bool) {
        if !set_eye_pos && self.attn_set_count >= 2 {
            return;
        }
        self.eye_pos = Vec3::new(self.eye_pos.x, self.eye_pos.y, self.eye_pos.z);
        self.attn_base_pos = Vec3::new(
            self.attn_base_pos.x,
            self.attn_base_pos.y + ATTN_Y_OFFSET,
            self.attn_base_pos.z,
        );
    }

    /// `daNpc_Tt_c::wait01` (`:608-630`), minus the `chkFlag(0x4)` branch's
    /// `demoProc()` return.
    fn wait01(&mut self, ctx: &Ctx<'_>) -> bool {
        if self.flags & FLAG_TALK_ITEM != 0 {
            self.return_to_state = self.state;
            self.state = STATE_TACT;
            self.set_anm_status(ctx.flags);
        } else if self.flags & FLAG_TALK != 0 {
            self.return_to_state = self.state;
            self.state = STATE_TALK;
            self.set_anm_status(ctx.flags);
        } else {
            self.order_flags |= ORDER_TALK;
            if !ctx.flags.test(FLAG_0C40) && ctx.flags.test(FLAG_0B08) {
                self.order_flags |= ORDER_TALK_ITEM;
            }
        }
        self.dance_proc();
        self.morf.is_morf()
    }

    /// `daNpc_Tt_c::talk01` (`:633-648`).
    fn talk01(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let temp = self.talk(ctx);
        if self.curr_msg_no == MSG_22C6 && ctx.mesg_send_button {
            self.flags &= !FLAG_HOME_FOLLOW;
        }
        if temp == STATUS_BOX_CLOSED {
            self.state = self.return_to_state;
            self.set_anm_status(ctx.flags);
            self.flags &= !FLAG_TALK;
            self.flags &= !FLAG_TALK_ITEM;
        }
        self.dance_proc();
        self.morf.is_morf()
    }

    /// `daNpc_Tt_c::wait_action` (`:664-690`), minus the `case 3: tact00()` arm, whose
    /// `mState == 3` is only reachable from the omitted `demoProc` path.
    fn wait_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        if self.action_status == ACTION_STARTING {
            self.state = STATE_WAIT;
            self.set_anm_status(ctx.flags);
            self.action_status += 1;
        } else if self.action_status != ACTION_ENDING {
            self.flags &= !FLAG_HIDE;
            let temp = match self.state {
                STATE_WAIT => self.wait01(ctx),
                STATE_TALK => self.talk01(ctx),
                _ => false,
            };
            self.look_back(actor, ctx);
            self.set_attention(temp);
        }
    }

    /// `daNpc_Tt_c::checkOrder` (`:112-124`).
    fn check_order(&mut self, demo_accept: bool, talk_command: bool, talk_xy: bool) {
        if !demo_accept
            && talk_command
            && self.order_flags & ORDER_TALK_DEMO != 0
        {
            if talk_xy {
                self.flags |= FLAG_TALK_ITEM;
            } else {
                self.flags |= FLAG_TALK;
            }
            self.talk_init();
        }
        self.order_flags = 0;
    }

    /// `daNpc_Tt_c::_execute` (`:793-847`), minus `playTexPatternAnm`, the
    /// `mpMorf->play` frame check, `mHeadAnm.move`, `eventOrder`,
    /// `fopAcM_posMoveF`, `mObjAcch.CrrPos`, `setCollision` and `ke_execute`.
    pub fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.jnt.set_param(
            0,
            JNT_MAX_BACKBONE_Y,
            0,
            JNT_MIN_BACKBONE_Y,
            JNT_MAX_HEAD_X,
            JNT_MAX_HEAD_Y,
            JNT_MIN_HEAD_X,
            JNT_MIN_HEAD_Y,
            JNT_MAX_TURN_STEP,
        );
        self.morf.play();
        let demo_accept = ctx.demo_accept;
        let talk_command = ctx.talk_command;
        let talk_xy = ctx.talk_xy;
        self.check_order(demo_accept, talk_command, talk_xy);
        self.wait_action(actor, ctx);
    }
}

pub struct Text<'a> {
    pub procs: &'a mut MsgProcs,
    pub info: &'a mut MsgGameInfo,
    pub flags: &'a mut MsgFlags,
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

    /// `fopMsgM_messageSendOn`.
    pub fn message_send_on(&mut self) {
        self.flags.message_send_on();
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
}

pub struct Ctx<'a> {
    pub player_eye: Option<Vec3>,
    pub flags: &'a mut EventFlags,
    pub talk_xy: bool,
    pub talk_command: bool,
    pub demo_accept: bool,
    pub mesg_send_button: bool,
    pub text: &'a mut Text<'a>,
}

#[derive(SystemParam)]
pub struct Env<'w> {
    pub text_flags: Res<'w, TextFlags>,
    pub flags: ResMut<'w, EventFlags>,
    pub archive: Option<Res<'w, MsgArchives>>,
}

#[derive(SystemParam)]
pub struct Msg<'w, 's> {
    pub procs: ResMut<'w, MsgProcs>,
    pub info: ResMut<'w, MsgGameInfo>,
    pub flags: ResMut<'w, MsgFlags>,
    pub one: ResMut<'w, Msg1Box>,
    pub two: ResMut<'w, Msg2Box>,
    pub players: Query<'w, 's, &'static Player>,
}

pub fn control(
    mut world: Env,
    mut msg: Msg,
    do_intent: Res<DoIntent>,
    mut actors: Query<(&mut Actor, &mut NpcTt)>,
) {
    let player = msg.players.iter().next().copied();
    let clear_count = world.text_flags.clear_count;
    let demo_accept = msg.info.event_run_check();
    let send_button = msg.one.m116a != 0;
    let do_now = do_intent.0;
    let archive = world
        .archive
        .as_deref()
        .map(|held| held as &dyn MsgArchive);
    for (mut actor, mut tt) in &mut actors {
        let tap = do_now && !tt.prev_do;
        tt.prev_do = do_now;
        let mut text = Text {
            procs: &mut msg.procs,
            info: &mut msg.info,
            flags: &mut msg.flags,
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
            flags: &mut world.flags,
            talk_xy: false,
            talk_command: tap,
            demo_accept,
            mesg_send_button: send_button,
            text: &mut text,
        };
        if !tt.entered {
            tt.create_init(&mut actor, &mut ctx);
        }
        tt.execute(&mut actor, &mut ctx);
    }
}