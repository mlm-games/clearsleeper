use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::intent::DoIntent;
use super::player::Player;
use super::stream::add_calc_angle_s2;
use super::walk::to_s16;
use crate::assets::MsgArchives;
use crate::text::driver::TextFlags;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgGameInfo, MsgProcs,
    fop_msgm_message_set_actor,
};
use crate::text::{STATUS_BOX_CLOSED, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED, STATUS_MSG_ENDS};

pub const NPC_KM1_ACTORS: [&str; 1] = ["Km1"];

pub fn is_npc_km1(name: &str) -> bool {
    NPC_KM1_ACTORS.contains(&name)
}

pub const STAFF_KM1: &str = "Km1"; // <- mEventCut.setActorInfo2("Km1", this), :95

// daNpc_Km1_HIO_c::a_prm_tbl - d_a_npc_km1.cpp:48-61
const HIO_FIELD_0: i16 = 0x1FFE; // <- mPrmTbl.field_0, :49
const HIO_FIELD_2: i16 = 0x38E0; // <- :50
const HIO_FIELD_4: i16 = -0x1FFE; // <- :51
const HIO_FIELD_6: i16 = -0x38E0; // <- :52
const HIO_FIELD_8: i16 = 0x0000; // <- :53
const HIO_FIELD_A: i16 = 0x0000; // <- :54
const HIO_FIELD_C: i16 = 0x0000; // <- :55
const HIO_FIELD_E: i16 = 0x0000; // <- :56
const HIO_FIELD_10: i16 = 0x0800; // <- :57
const HIO_FIELD_12: i16 = 0x0800; // <- :58
const HIO_ATTENTION_ARROW_Y_OFFSET: f32 = 150.0; // <- :59

const PLAYER_EYE_OFFSET_Y: f32 = -20.0; // <- dNpc_playerEyePos(-20.0f), :383
const PLAYER_HEAD_TOP: f32 = 40.0; // <- d_a_player_main.cpp:11575
const TRN_SCALE: i16 = 4; // <- :399
const TRN_MAX: i16 = 0x800; // <- :399
const JNT_SCALE: i16 = 4; // <- d_npc.cpp dNpc_JntCtrl_c::follow
const JNT_MIN_STEP: i32 = 1; // <- d_npc.cpp dNpc_JntCtrl_c::move

const CULL_MIN: [f32; 3] = [-60.0, -20.0, -80.0]; // <- fopAcM_setCullSizeBox, :727
const CULL_MAX: [f32; 3] = [60.0, 160.0, 60.0]; // <- :727

const ACTION_STARTING: i8 = 0; // <- set_action, :518
const ACTION_ONGOING: i8 = 1; // <- wait_action1, :576
const ACTION_ENDING: i8 = -1; // <- set_action, :514

const STT_WAIT: i8 = 1; // <- a_anm_prm_tbl[1], :245
const STT_TALK: i8 = 2; // <- setStt case 2, :317

const LOOK_NONE: i8 = 0; // <- lookBack case 0, :380
const LOOK_PLAYER: i8 = 1; // <- :382
const LOOK_ATT_POS: i8 = 2; // <- :388
const LOOK_ANGLE: i8 = 3; // <- :394

const ORDER_NONE: i8 = 0; // <- checkOrder, :361
const ORDER_SPEAK: i8 = 1; // <- :341
const ORDER_ATTENTION: i8 = 2; // <- :341

const ANM_WAIT: i8 = 0; // <- daNpc_Km1_c::Animation::ANM_WAIT, d_a_npc_km1.h:21
const ANM_NONE: i8 = -1; // <- a_anm_prm_tbl[0].field_0x0, :244
const ANM_ATR_NONE: u8 = 0xFF; // <- wait01 / endEvent, :528 / :498

const NO_STATUS: u8 = 0xFF; // <- fopNpc_npc_c::talk, d_npc.cpp:665
const MSG_STATUS_BOX_OPENING: u8 = 0x02; // <- talk01 case 2, :558
const MSG_STATUS_TYPING: u8 = 0x06; // <- talk01 case 6, :557

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
        let mut diff = orig;
        add_calc_angle_s2(&mut diff, 0, speed, max_vel);
        if i32::from(diff).unsigned_abs() > i32::from(speed) {
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
            if i32::from(reached[row].wrapping_sub(self.angles[row][axis])).unsigned_abs()
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
                (to_s16(delta.x.atan2(delta.z)), to_s16(delta.y.atan2(flat)))
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
pub struct NpcKm1 {
    pub entered: bool,
    pub prev_do: bool,
    pub action_status: i8,
    pub state: i8,
    pub prev_state: i8,
    pub look_state: i8,
    pub order: i8,
    pub talk_order: bool,
    pub anm: i8,
    pub anm_atr: u8,
    pub anm_tag: u8,
    pub anm_typing_count: i8,
    pub has_attention: bool,
    pub head_only_follow: bool,
    pub eye_set: bool,
    pub act_result: bool,
    pub head_turn_vel: i16,
    pub target_y: i16,
    pub start_angle: [i16; 3],
    pub att_pos: Vec3,
    pub eye_pos: Vec3,
    pub attention: Vec3,
    pub curr_msg_no: u32,
    pub curr_msg_id: u32,
    pub msg_bound: bool,
    pub jnt: JntCtrl,
}

impl NpcKm1 {
    pub fn spawn(parameters: u32) -> Self {
        let _ = parameters;
        Self {
            entered: false,
            prev_do: false,
            action_status: ACTION_STARTING,
            state: STT_WAIT,
            prev_state: STT_WAIT,
            look_state: LOOK_NONE,
            order: ORDER_NONE,
            talk_order: false,
            anm: ANM_NONE,
            anm_atr: ANM_ATR_NONE,
            anm_tag: 0,
            anm_typing_count: 0,
            has_attention: false,
            head_only_follow: false,
            eye_set: false,
            act_result: false,
            head_turn_vel: 0,
            target_y: 0,
            start_angle: [0; 3],
            att_pos: Vec3::ZERO,
            eye_pos: Vec3::ZERO,
            attention: Vec3::ZERO,
            curr_msg_no: 0,
            curr_msg_id: FPC_M_ERROR_PROCESS_ID,
            msg_bound: false,
            jnt: JntCtrl::create(),
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    /// `daNpc_Km1_c::createInit` (`:94-110`), minus `gravity`, the `mCyl`/`mStts`
    /// registration, `mpMorf->setMorf` and the `setMtx` matrix work. `set_action` runs
    /// `wait_action1` once with `ACTION_STARTING` (`:511-522`).
    pub fn create_init(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.att_pos = actor.pos;
        self.start_angle = actor.angle;
        self.eye_set = true;
        self.action_status = ACTION_STARTING;
        self.wait_action1(actor, ctx);
        self.entered = true;
    }

    /// `daNpc_Km1_c::setAnm_anm` (`:219-239`), minus `anmResID`/`dNpc_setAnmIDRes`.
    fn set_anm_anm(&mut self, anm: i8) -> bool {
        if anm >= 0 && self.anm != anm {
            self.anm = anm;
            return true;
        }
        false
    }

    /// `daNpc_Km1_c::setAnm` (`:242-250`): `a_anm_prm_tbl[field_0x7D0].field_0x0`.
    fn set_anm(&mut self) {
        let anm = match self.state {
            STT_WAIT => ANM_WAIT,
            _ => ANM_NONE,
        };
        self.set_anm_anm(anm);
    }

    /// `daNpc_Km1_c::setAnm_ATR` (`:281-289`).
    fn set_anm_atr(&mut self) {
        self.set_anm_anm(ANM_WAIT);
    }

    /// `daNpc_Km1_c::chngAnmAtr` (`:269-274`).
    fn chng_anm_atr(&mut self, atr: u8) {
        if atr < 1 && atr != self.anm_atr {
            self.anm_atr = atr;
            self.set_anm_atr();
        }
    }

    fn chng_anm_tag(&mut self) {}

    fn ctrl_anm_atr(&mut self) {}

    /// `daNpc_Km1_c::ctrlAnmTag` (`:261-265`).
    fn ctrl_anm_tag(&mut self) {
        let _ = self.anm_atr;
    }

    /// `daNpc_Km1_c::anmAtr` (`:292-310`).
    fn anm_atr(&mut self, status: u16, ctx: &mut Ctx<'_>) {
        if status == MSG_STATUS_TYPING as u16 {
            if self.anm_typing_count == 0 {
                self.anm_tag = 0xFF;
                let attr = ctx.text.mesg_anime();
                self.chng_anm_atr(attr);
                self.anm_typing_count += 1;
            }
            let tag = ctx.text.mesg_anime_tag();
            if tag != 0xFF && tag != self.anm_tag {
                ctx.text.clear_mesg_anime_tag();
                self.anm_tag = tag;
                self.chng_anm_tag();
            }
        } else if status == STATUS_MSG_DISPLAYED as u16 {
            self.anm_typing_count = 0;
        }
        self.ctrl_anm_atr();
        self.ctrl_anm_tag();
    }

    /// `daNpc_Km1_c::setStt` (`:313-327`).
    fn set_stt(&mut self, stt: i8) {
        let prev = self.state;
        self.state = stt;
        if stt == STT_TALK {
            self.look_state = LOOK_PLAYER;
            self.anm_atr = ANM_ATR_NONE;
            self.prev_state = prev;
        } else {
            self.set_anm();
        }
    }

    /// `daNpc_Km1_c::getMsg` (`:335-337`).
    fn get_msg(&self) -> u32 {
        0
    }

    /// `daNpc_Km1_c::next_msgStatus` (`:330-332`).
    fn next_msg_status(&self, curr: &mut u32) -> u8 {
        let _ = curr;
        STATUS_MSG_ENDS
    }

    /// `fopNpc_npc_c::talk` (`d_npc.cpp:664-697`) with `param_1 == 1`.
    fn talk(&mut self, ctx: &mut Ctx<'_>) {
        if self.curr_msg_id == FPC_M_ERROR_PROCESS_ID {
            self.curr_msg_no = self.get_msg();
            let eye = self.eye_pos;
            self.curr_msg_id = ctx.text.message_set_actor(self.curr_msg_no, eye);
            self.msg_bound = false;
        } else if self.msg_bound {
            let status = ctx.text.status().unwrap_or(NO_STATUS);
            if status == STATUS_MSG_DISPLAYED {
                let mut curr = self.curr_msg_no;
                let next = self.next_msg_status(&mut curr);
                self.curr_msg_no = curr;
                ctx.text.set_status(next);
            } else if status == STATUS_BOX_CLOSED {
                ctx.text.set_status(STATUS_MSG_DESTROYED);
                self.curr_msg_id = FPC_M_ERROR_PROCESS_ID;
            }
            self.anm_atr(status as u16, ctx);
        } else {
            self.msg_bound = ctx.text.procs.search_by_id(self.curr_msg_id).is_some();
        }
    }

    /// `daNpc_Km1_c::endEvent` (`:496-499`), minus `dComIfGp_event_reset`.
    fn end_event(&mut self) {
        self.anm_atr = ANM_ATR_NONE;
    }

    /// `daNpc_Km1_c::talk01` (`:553-570`).
    fn talk01(&mut self, ctx: &mut Ctx<'_>) -> bool {
        self.talk(ctx);
        if self.msg_bound {
            match ctx.text.status().unwrap_or(NO_STATUS) {
                MSG_STATUS_TYPING | MSG_STATUS_BOX_OPENING => {}
                STATUS_MSG_DESTROYED => {
                    self.set_stt(self.prev_state);
                    self.talk_order = false;
                    self.end_event();
                }
                _ => {}
            }
        }
        true
    }

    /// `daNpc_Km1_c::wait01` (`:525-550`).
    fn wait01(&mut self, ctx: &Ctx<'_>) -> bool {
        if self.talk_order {
            let mut set_stt = true;
            if ctx.talk_xy && !ctx.present_done {
                set_stt = false;
            }
            if set_stt {
                self.set_stt(STT_TALK);
            }
        } else {
            self.order = ORDER_ATTENTION;
            if self.has_attention {
                self.look_state = LOOK_PLAYER;
            } else {
                self.look_state = LOOK_ANGLE;
                self.target_y = self.start_angle[1];
                self.jnt.set_trn();
            }
        }
        true
    }

    /// `daNpc_Km1_c::wait_action1` (`:573-594`).
    fn wait_action1(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        if self.action_status == ACTION_STARTING {
            self.set_stt(STT_WAIT);
            self.action_status += 1;
        } else if self.action_status != ACTION_ENDING {
            self.has_attention = ctx.attention;
            self.act_result = match self.state {
                STT_TALK => self.talk01(ctx),
                STT_WAIT => self.wait01(ctx),
                _ => false,
            };
            self.look_back(actor, ctx.player_eye);
        }
    }

    /// `daNpc_Km1_c::lookBack` (`:367-404`).
    fn look_back(&mut self, actor: &mut Actor, player_eye: Option<Vec3>) {
        let mut dst = None;
        let mut src = Vec3::new(actor.pos.x, self.eye_pos.y, actor.pos.z);
        let mut target_y = actor.angle[1];
        match self.look_state {
            LOOK_NONE => {}
            LOOK_PLAYER => {
                dst = player_eye;
                src = Vec3::new(actor.pos.x, self.eye_pos.y, actor.pos.z);
            }
            LOOK_ATT_POS => {
                dst = Some(self.att_pos);
                src = Vec3::new(actor.pos.x, self.eye_pos.y, actor.pos.z);
            }
            LOOK_ANGLE => target_y = self.target_y,
            _ => {}
        }
        if self.jnt.trn_chk() {
            add_calc_angle_s2(&mut self.head_turn_vel, HIO_FIELD_12, TRN_SCALE, TRN_MAX);
        } else {
            self.head_turn_vel = 0;
        }
        let turn = self.head_turn_vel;
        let follow = self.head_only_follow;
        self.jnt.look_at_target(&mut actor.angle[1], dst, src, target_y, turn, follow);
    }

    /// `daNpc_Km1_c::setAttention` (`:417-427`).
    pub fn set_attention(&mut self, actor: &Actor) {
        self.attention = Vec3::new(
            actor.pos.x,
            actor.pos.y + HIO_ATTENTION_ARROW_Y_OFFSET,
            actor.pos.z,
        );
        if !self.eye_set && !self.act_result {
            return;
        }
        self.eye_set = false;
    }

    /// `daNpc_Km1_c::checkOrder` (`:351-364`).
    fn check_order(&mut self, demo_accept: bool, talk_command: bool) {
        if demo_accept {
            return;
        }
        if !talk_command {
            return;
        }
        if self.order != ORDER_SPEAK && self.order != ORDER_ATTENTION {
            return;
        }
        self.order = ORDER_NONE;
        self.talk_order = true;
    }

    /// `daNpc_Km1_c::event_proc` (`:502-508`), minus `privateCut`.
    fn event_proc(&mut self, cut_proc: bool, actor: &mut Actor, ctx: &Ctx<'_>) {
        if !cut_proc {
            self.private_cut();
        }
        self.look_back(actor, ctx.player_eye);
    }

    fn private_cut(&mut self) {}

    /// `daNpc_Km1_c::_execute` (`:646-673`).
    pub fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.jnt.set_param(
            HIO_FIELD_8,
            HIO_FIELD_A,
            HIO_FIELD_C,
            HIO_FIELD_E,
            HIO_FIELD_0,
            HIO_FIELD_2,
            HIO_FIELD_4,
            HIO_FIELD_6,
            HIO_FIELD_10,
        );
        let demo_accept = ctx.demo_accept;
        let talk_command = ctx.talk_command;
        self.check_order(demo_accept, talk_command);
        if ctx.event_run && !talk_command {
            self.event_proc(ctx.cut_proc, actor, ctx);
        } else {
            self.wait_action1(actor, ctx);
        }
        self.set_attention(actor);
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
    pub player_eye: Option<Vec3>,
    pub talk_command: bool,
    pub demo_accept: bool,
    pub event_run: bool,
    pub cut_proc: bool,
    pub talk_xy: bool,
    pub present_done: bool,
    pub attention: bool,
    pub text: &'a mut Text<'a>,
}

#[derive(SystemParam)]
pub struct Env<'w> {
    pub text_flags: Res<'w, TextFlags>,
    pub active: Res<'w, ActiveEvent>,
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
    world: Env,
    mut msg: Msg,
    list: Res<EventList>,
    do_intent: Res<DoIntent>,
    mut actors: Query<(&mut Actor, &mut NpcKm1)>,
) {
    let player = msg.players.iter().next().copied();
    let clear_count = world.text_flags.clear_count;
    let event_run = msg.info.event_run_check();
    let do_now = do_intent.0;
    let archive = world
        .archive
        .as_deref()
        .map(|held| held as &dyn MsgArchive);
    for (mut actor, mut km1) in &mut actors {
        let tap = do_now && !km1.prev_do;
        km1.prev_do = do_now;
        let cut_proc = ActiveEvent::staff_cut(&world.active, &list, STAFF_KM1).is_some();
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
            talk_command: tap,
            demo_accept: event_run,
            event_run,
            cut_proc,
            talk_xy: false,
            present_done: false,
            attention: false,
            text: &mut text,
        };
        if !km1.entered {
            km1.create_init(&mut actor, &mut ctx);
        }
        km1.execute(&mut actor, &mut ctx);
    }
}