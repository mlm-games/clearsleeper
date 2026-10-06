use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::daynight::{DayNight, TIME_DAY};
use super::event::{ActiveEvent, EventList};
use super::flags::{Collect, StoryBits, SYMBOL_DIN};
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
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED, STATUS_MSG_ENDS,
};

pub const NPC_BTSW2_ACTORS: [&str; 1] = ["Btsw2"];

pub fn is_npc_btsw2(name: &str) -> bool {
    NPC_BTSW2_ACTORS.contains(&name)
}

pub const STAFF_BTSW2: &str = "Btsw2";

// daNpc_Btsw2_HIO_c::daNpc_Btsw2_HIO_c() - d_a_npc_btsw2.cpp:66-89
const HIO_M04: f32 = 0.0; // <- :67
const HIO_MAX_HEAD_X: i16 = 0x1F40; // <- :68
const HIO_MAX_HEAD_Y: i16 = 0x1F40; // <- :69
const HIO_MAX_BACKBONE_X: i16 = 0x0; // <- :70
const HIO_MAX_BACKBONE_Y: i16 = 0x1F40; // <- :71
const HIO_MIN_HEAD_X: i16 = -0xBB8; // <- :72
const HIO_MIN_HEAD_Y: i16 = -0x1F40; // <- :73
const HIO_MIN_BACKBONE_X: i16 = 0x0; // <- :74
const HIO_MIN_BACKBONE_Y: i16 = -0x1F40; // <- :75
const HIO_MAX_TURN_STEP: i16 = 0x3E8; // <- :76
const HIO_MAX_HEAD_TURN_VEL: i16 = 0x640; // <- :77
const HIO_ATTN_Y_OFFSET: f32 = 40.0; // <- :78
const HIO_MAX_ATTN_ANGLE_Y: i16 = 0x2000; // <- :79
const HIO_MAX_ATTN_DIST_XZ: f32 = 200.0; // <- :81

const ATTN_EXTEND_DIST: f32 = 40.0; // <- :227
const ATTN_EXTEND_DEG: f32 = 10.0; // <- :228
const DEG2S: f32 = 0x8000 as f32 / 180.0; // <- SSystem/SComponent/c_angle.h:26

const TALK_TURN_SCALE: i16 = 4; // <- :334
const TALK_TURN_MAX: i16 = 0x1800; // <- :334

const JNT_SCALE: i16 = 4; // <- d_npc.cpp dNpc_JntCtrl_c::follow
const JNT_MIN_STEP: i32 = 1; // <- d_npc.cpp dNpc_JntCtrl_c::move

const PLAYER_HEAD_TOP: f32 = 40.0; // <- d_a_player_main.cpp:11575
const STOP_TIMER_BASE: f32 = 90.0; // <- :416
const STOP_TIMER_RANGE: f32 = 300.0; // <- :417

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const CULL_MIN: [f32; 3] = [-70.0, -100.0, -80.0]; // <- f_op_actor_mng.cpp:570 (fopAc_CULLBOX_12_e)
const CULL_MAX: [f32; 3] = [70.0, 240.0, 100.0]; // <- f_op_actor_mng.cpp:570

const ACTION_STARTING: i8 = 0; // <- d_a_npc_btsw2.h:12
const ACTION_ONGOING: i8 = 1; // <- d_a_npc_btsw2.h:13
const ACTION_ENDING: i8 = -1; // <- d_a_npc_btsw2.h:14

const MODE_WAIT: i8 = 1; // <- :517
const MODE_TALK: i8 = 2; // <- :529

const ORDER_NONE: i8 = 0; // <- :522
const ORDER_SPEAK: i8 = 1; // <- :238
const ORDER_ATTENTION: i8 = 2; // <- :236

const ANM_WAIT01: i8 = 0; // <- l_bck_ix_tbl index 0, :94
const ANM_WAIT02: i8 = 1; // <- :95
const ANM_TALK01: i8 = 2; // <- :96
const ANM_TALK02: i8 = 3; // <- :97
const ANM_ONEGAI: i8 = 4; // <- :98
const ANM_SUGOI: i8 = 5; // <- :99
const ANM_SUGOI_ALT: i8 = 6; // <- :100
const ANM_WALK: i8 = 7; // <- :101
const ANM_KASIGE: i8 = 8; // <- :102
const ANM_NONE: i8 = -1; // <- :195

const MSG_1AB0: u32 = 0x1AB0; // <- :297
const MSG_1AB1: u32 = 0x1AB1; // <- :301
const MSG_1AB2: u32 = 0x1AB2; // <- :299

const FLAG_3102: u16 = 0x3102; // <- dSv_event_flag_c::UNK_3102, :295
const REG_C203: u16 = 0xC203; // <- dSv_event_flag_c::UNK_C203, :545
const REG_C203_DONE: u8 = 3; // <- :545

const NO_STATUS: u8 = 0xFF; // <- :283 (status == 0xFF before the message is bound)

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = seed[0] as f32 / RND_SCALE[0]
        + seed[1] as f32 / RND_SCALE[1]
        + seed[2] as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

fn target_angle_y(lhs: Vec3, rhs: Vec3) -> i16 {
    to_s16((rhs.x - lhs.x).atan2(rhs.z - lhs.z))
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

    pub fn head_y(&self) -> i16 {
        self.angles[0][1]
    }

    pub fn backbone_y(&self) -> i16 {
        self.angles[1][1]
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
pub struct NpcBtsw2 {
    pub entered: bool,
    pub prev_do: bool,
    pub anm: i8,
    pub order: i8,
    pub mode: i8,
    pub action_status: i8,
    pub has_attention: bool,
    pub talk_order: bool,
    pub stop_timer: i16,
    pub att_pos: Vec3,
    pub attention: Vec3,
    pub eye_pos: Vec3,
    pub curr_msg_no: u32,
    pub curr_msg_id: u32,
    pub seed: [i32; 3],
    pub jnt: JntCtrl,
}

impl NpcBtsw2 {
    pub fn spawn(parameters: u32) -> Self {
        let _ = parameters;
        Self {
            entered: false,
            prev_do: false,
            anm: ANM_WAIT01,
            order: ORDER_NONE,
            mode: MODE_WAIT,
            action_status: ACTION_STARTING,
            has_attention: false,
            talk_order: false,
            stop_timer: 0,
            att_pos: Vec3::ZERO,
            attention: Vec3::ZERO,
            eye_pos: Vec3::ZERO,
            curr_msg_no: 0,
            curr_msg_id: FPC_M_ERROR_PROCESS_ID,
            seed: [RND_SEED; 3],
            jnt: JntCtrl::create(),
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    /// `daNpc_Btsw2_c::CreateInit` (`:402-425`), minus `gravity`, the `mCyl`/`mStts`
    /// registration and the whole `dPath_GetRoomPath` block. `setAction(&wait_action)`
    /// runs `wait_action` once with `ACTION_STARTING` (d_a_npc_btsw2.h:18-28).
    pub fn create_init(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.att_pos = actor.pos;
        self.stop_timer = (STOP_TIMER_BASE + rnd(&mut self.seed) * STOP_TIMER_RANGE) as i16;
        self.curr_msg_no = 0;
        self.curr_msg_id = FPC_M_ERROR_PROCESS_ID;
        self.wait_action(actor, ctx);
        self.entered = true;
    }

    /// `_create` (`:545-547`) refuses creation outright, so the actor never executes.
    pub fn create_refused(&self, stories: &StoryBits, collect: &Collect) -> bool {
        stories.reg(REG_C203) == REG_C203_DONE || !collect.is_symbol(SYMBOL_DIN)
    }

    fn next_msg_status(&self) -> u8 {
        STATUS_MSG_ENDS
    }

    /// `daNpc_Btsw2_c::setAnm` (`:171-213`).
    pub fn set_anm(&mut self, idx: i8) {
        if idx == self.anm {
            return;
        }
        if self.anm == ANM_NONE {
            return;
        }
        self.anm = idx;
        if self.anm == ANM_WALK || self.anm == ANM_KASIGE {
            self.jnt.on_head_lock();
        } else {
            self.jnt.off_head_lock();
        }
    }

    /// `daNpc_Btsw2_c::anmAtr` (`:258-290`), minus the `mpMorf->checkFrame` tail
    /// (BCK frame counts are unavailable).
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
    }

    /// `daNpc_Btsw2_c::chkAttention` (`:216-232`).
    pub fn chk_attention(&self, pos: Vec3, angle: i16, player: Option<Vec3>) -> bool {
        let mut max_dist = HIO_MAX_ATTN_DIST_XZ;
        let mut max_angle = i32::from(HIO_MAX_ATTN_ANGLE_Y);
        let Some(player) = player else {
            return false;
        };
        let delta = Vec3::new(player.x - pos.x, 0.0, player.z - pos.z);
        let dist_xz = (delta.x * delta.x + delta.z * delta.z).sqrt();
        let mut target_angle = to_s16(delta.x.atan2(delta.z));
        if self.has_attention {
            max_dist += ATTN_EXTEND_DIST;
            max_angle += (ATTN_EXTEND_DEG * DEG2S) as i32;
        }
        target_angle = target_angle.wrapping_sub(angle);
        max_angle > i32::from(target_angle).abs() && max_dist > dist_xz
    }

    /// `daNpc_Btsw2_c::getMsg` (`:293-304`).
    pub fn get_msg(&mut self, stories: &mut StoryBits, day_night: i32) -> u32 {
        if !stories.test(FLAG_3102) {
            stories.set(FLAG_3102);
            MSG_1AB0
        } else if day_night != TIME_DAY {
            MSG_1AB2
        } else {
            MSG_1AB1
        }
    }

    /// `daNpc_Btsw2_c::setAttention` (`:312-314`).
    pub fn set_attention(&mut self) {
        self.attention = Vec3::new(
            self.att_pos.x,
            self.att_pos.y + HIO_ATTN_Y_OFFSET,
            self.att_pos.z,
        );
    }

    /// `daNpc_Btsw2_c::lookBack` (`:317-347`).
    pub fn look_back(&mut self, actor: &mut Actor, player_eye: Option<Vec3>) {
        let default_y = actor.angle[1];
        let mut dst = None;
        let mut src = Vec3::ZERO;
        let mut free = true;
        if self.mode == MODE_WAIT || self.mode == MODE_TALK {
            if self.mode == MODE_TALK {
                free = false;
                self.jnt.set_trn();
                if !self.has_attention {
                    if let Some(eye) = player_eye {
                        let want = target_angle_y(actor.pos, eye);
                        add_calc_angle_s2(
                            &mut actor.angle[1],
                            want,
                            TALK_TURN_SCALE,
                            TALK_TURN_MAX,
                        );
                    }
                }
            }
            if self.has_attention {
                dst = player_eye;
                src = Vec3::new(actor.pos.x, self.eye_pos.y, actor.pos.z);
            }
        }
        self.jnt.look_at_target(
            &mut actor.angle[1],
            dst,
            src,
            default_y,
            HIO_MAX_HEAD_TURN_VEL,
            free,
        );
    }

    /// `fopNpc_npc_c::talk` (`d_npc.cpp:664-697`) specialised to this actor.
    fn talk(&mut self, ctx: &mut Ctx<'_>) -> u8 {
        let mut status = NO_STATUS;
        if self.curr_msg_id == FPC_M_ERROR_PROCESS_ID {
            self.curr_msg_no = self.get_msg(ctx.stories, ctx.day_night);
            let eye = self.eye_pos;
            self.curr_msg_id = ctx.text.message_set_actor(self.curr_msg_no, eye);
        } else if ctx.text.msg_live(self.curr_msg_id) {
            status = ctx.text.status().unwrap_or(NO_STATUS);
            if status == STATUS_MSG_DISPLAYED {
                ctx.text.set_status(self.next_msg_status());
            } else if status == STATUS_BOX_CLOSED {
                ctx.text.set_status(STATUS_MSG_DESTROYED);
                self.curr_msg_id = FPC_M_ERROR_PROCESS_ID;
            }
            self.anm_atr(ctx.text.mesg_anime());
            ctx.text.set_mesg_anime(0xFF);
        }
        status
    }

    /// `daNpc_Btsw2_c::talk01` (`:437-449`).
    fn talk01(&mut self, ctx: &mut Ctx<'_>) {
        if self.talk(ctx) == STATUS_BOX_CLOSED {
            self.mode = MODE_WAIT;
            self.talk_order = false;
            if self.stop_timer == 0 {
                self.set_anm(ANM_WAIT01);
            } else {
                self.set_anm(ANM_WAIT01);
            }
        }
    }

    /// `daNpc_Btsw2_c::wait01` (`:428-434`), minus `pathMove`.
    fn wait01(&mut self) {
        if self.talk_order {
            self.mode = MODE_TALK;
        } else {
            self.order = ORDER_ATTENTION;
        }
    }

    /// `daNpc_Btsw2_c::checkOrder` (`:245-255`).
    fn check_order(&mut self, demo_accept: bool, talk_command: bool) {
        if demo_accept {
            return;
        }
        if !talk_command {
            return;
        }
        if self.order == ORDER_SPEAK || self.order == ORDER_ATTENTION {
            self.order = ORDER_NONE;
            self.talk_order = true;
        }
    }

    /// `daNpc_Btsw2_c::wait_action` (`:515-539`).
    fn wait_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        if self.action_status == ACTION_STARTING {
            self.mode = MODE_WAIT;
            self.action_status += 1;
        } else if self.action_status != ACTION_ENDING {
            let facing = actor.angle[1]
                .wrapping_add(self.jnt.head_y())
                .wrapping_add(self.jnt.backbone_y());
            self.has_attention = self.chk_attention(actor.pos, facing, ctx.player_pos);
            self.order = ORDER_NONE;
            if self.mode == MODE_WAIT {
                self.wait01();
            } else if self.mode == MODE_TALK {
                self.talk01(ctx);
            }
            self.look_back(actor, ctx.player_eye);
            self.set_attention();
        }
    }

    /// `daNpc_Btsw2_c::_execute` (`:581-606`).
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
        let demo_accept = ctx.demo_accept;
        let talk_command = ctx.talk_command;
        self.check_order(demo_accept, talk_command);
        if !ctx.cut_proc {
            self.wait_action(actor, ctx);
        }
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
    pub talk_command: bool,
    pub demo_accept: bool,
    pub cut_proc: bool,
    pub day_night: i32,
    pub stories: &'a mut StoryBits,
    pub text: &'a mut Text<'a>,
}

#[derive(SystemParam)]
pub struct Env<'w> {
    pub text_flags: Res<'w, TextFlags>,
    pub daynight: Res<'w, DayNight>,
    pub collect: Res<'w, Collect>,
    pub stories: ResMut<'w, StoryBits>,
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
    list: Res<EventList>,
    active: Res<ActiveEvent>,
    do_intent: Res<DoIntent>,
    mut commands: Commands,
    mut actors: Query<(Entity, &mut Actor, &mut NpcBtsw2)>,
) {
    let player = msg.players.iter().next().copied();
    let clear_count = world.text_flags.clear_count;
    let day_night = world.daynight.daynight();
    let demo_accept = msg.info.event_run_check();
    let do_now = do_intent.0;
    let archive = world
        .archive
        .as_deref()
        .map(|held| held as &dyn MsgArchive);
    for (entity, mut actor, mut btsw2) in &mut actors {
        let mut refused = false;
        if !btsw2.entered {
            refused = btsw2.create_refused(&world.stories, &world.collect);
        }
        if refused {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        let tap = do_now && !btsw2.prev_do;
        btsw2.prev_do = do_now;
        let cut_proc = ActiveEvent::staff_cut(&active, &list, STAFF_BTSW2).is_some();
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
                    link.pos.y + PLAYER_HEAD_TOP + HIO_M04,
                    link.pos.z,
                )
            }),
            talk_command: tap,
            demo_accept,
            cut_proc,
            day_night,
            stories: &mut world.stories,
            text: &mut text,
        };
        if !btsw2.entered {
            btsw2.create_init(&mut actor, &mut ctx);
        }
        btsw2.execute(&mut actor, &mut ctx);
    }
}