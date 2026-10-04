use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{StoryBits, Switches};
use super::intent::DoIntent;
use super::inventory::{Equipped, ITEM_WIND_WAKER, WAKER_SLOT};
use super::player::Player;
use super::walk::{from_s16, to_s16};
use crate::assets::MsgArchives;
use crate::text::driver::TextFlags;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgGameInfo, MsgProcs, fop_msgm_message_set,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED,
    STATUS_MSG_ENDS,
};

pub const SAGE_ACTORS: [&str; 2] = ["Hr", "Hr2"];

const TORNADO_ACTOR: &str = "Ytrnd00";
const STAFF_HR: &str = "Hr";
const STAFF_HR2: &str = "Hr2";
const EVENT_TACT_HT: &str = "TACT_HT";
const EVENT_INTRO_RT: &str = "INTRO_RT";
const EVENT_INTRO_RT_F: &str = "INTRO_RT_F";
const EVENT_TACT0_RT: &str = "TACT0_RT";

const TYPE_HT: i8 = 0;
const TYPE_RT: i8 = 1;

const HR_FLAG_00000001: u16 = 0x0000_0001;
const HR_FLAG_00000008: u16 = 0x0000_0008;
const HR_FLAG_00000010: u16 = 0x0000_0010;
const HR_FLAG_00000020: u16 = 0x0000_0020;
const HR_FLAG_00000040: u16 = 0x0000_0040;
const HR_FLAG_00000080: u16 = 0x0000_0080;
const HR_FLAG_00000100: u16 = 0x0000_0100;
const HR_FLAG_00000400: u16 = 0x0000_0400;

const EVFLAG_TORNADO_ACTIVE: u16 = 0x0000_0008;

const ORDER_TALK: u8 = 1;
const ORDER_TALK_ITEM: u8 = 4;
const ORDER_ALL: u8 = 7;

const EVENT_TACT_DONE: u16 = 0x2708;
const EVENT_TACT_START: u16 = 0x2710;
const EVENT_INTRO_SEEN: u16 = 0x3D01;

const MSG_PLAIN_TALK: u32 = 0x1901;
const MSG_LINK: u32 = 0x5B3;
const MSG_CHAIN: [u32; 4] = [0x1906, 0x1933, 0x1934, 0x1938];

const MSG_ANM: [i8; 9] = [0, 1, 3, 4, 7, 5, 9, 8, 6];

const ITEM_NONE: u8 = 0xFF;

const BYTE_MASK: u32 = 0xFF;
const SWBIT_SHIFT: u32 = 8;

const NO_STATUS: u8 = 0xFF;
const NO_INDEX: i16 = -1;
const NO_STAFF: usize = usize::MAX;

const EYE_LIFT: f32 = 100.0;
const ATTENTION_LIFT: f32 = 150.0;
const SET_ATTENTION_LIFT: f32 = 50.0;

const TALK_CONE: i32 = 0x2AAA;
const RT_TIMER: i16 = 0x14;
const HIT_DELAY: u8 = 0x52;
const HIT_COUNT_MAX: u8 = 3;
const SMALL_SCALE: f32 = 11.0;
const ANGLE_STEP: i16 = 0x100;
const SHOT_Y: f32 = 6000.0;
const SHOT_Y_ARC: f32 = 1000.0;
const TORNADO_DROP: f32 = 1500.0;
const TORNADO_DAMP: f32 = 0.95;
const BODY_RADIUS: f32 = 45.0;
const BODY_HEIGHT: f32 = 100.0;

const ANM_ANGRY: i8 = 2;
const ANM_DAMAGE: i8 = 11;
const ANM_TALK_03: i8 = 12;
const TEX_DAMAGE: i8 = 1;
const TEX_ANGRY: i8 = 2;

const SE_PRE_TAKT: u32 = 0x08A7;
const SE_CV_RC_DAMAGE: u32 = 0x48EA;
const SE_CV_RC_MODAE_1: u32 = 0x48EB;
const SE_CV_RC_MODAE_2: u32 = 0x48EC;
const SE_CV_RC_ENTER: u32 = 0x48EE;
const BGM_DIOCTA_BATTLE: u32 = 0x8000_002B;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Wait01 = 0,
    Wait02 = 1,
    Talk = 2,
    HtTact = 3,
    RtSearch = 5,
    RtHide = 6,
    RtIntro = 7,
    RtAngry = 8,
    RtHit0 = 9,
    RtHit1 = 10,
    HtHide = 11,
    RtWin = 12,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Starting = 0,
    Ongoing = 1,
    Ending = -1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TalkPhase {
    Init = 0,
    Create = 1,
    Active = 2,
    Finished = -1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookBack {
    Full = 0,
    Head = 1,
    None = 0xFF,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Sage {
    pub shape_type: u8,
    pub swbit: u8,
    pub sage_type: i8,
    pub state: Step,
    pub return_state: Step,
    pub flags: u16,
    pub ev_flags: u8,
    pub order_flags: u8,
    pub action: Action,
    pub talk_phase: TalkPhase,
    pub talk_release: u8,
    pub hit_count: u8,
    pub hit_delay_timer: u8,
    pub attn_set_count: u8,
    pub tex_pattern_idx: i8,
    pub msg_anm_idx: u8,
    pub anm_idx: i8,
    pub anm_loop_count: u8,
    pub anm_timer: f32,
    pub curr_msg_no: u32,
    pub msg_no: u32,
    pub msg_id: u32,
    pub staff_idx: usize,
    pub event_idx: i16,
    pub move_timer: i16,
    pub speed: Vec3,
    pub speed_f: f32,
    pub scale: f32,
    pub eye_pos: Vec3,
    pub eye_target: Vec3,
    pub attn_base_pos: Vec3,
    pub attn_target: Vec3,
    pub cloud_pos: Vec3,
    pub home: Vec3,
    pub home_angle: i16,
    pub tornado: Option<Entity>,
    pub entered: bool,
    pub do_held: bool,
}

impl Sage {
    pub fn spawn(parameters: u32) -> Self {
        let shape_type = (parameters & BYTE_MASK) as u8;
        let sage_type = if shape_type == 1 { TYPE_RT } else { TYPE_HT };
        Self {
            shape_type,
            swbit: ((parameters >> SWBIT_SHIFT) & BYTE_MASK) as u8,
            sage_type,
            state: Step::Wait01,
            return_state: Step::Wait01,
            flags: 0,
            ev_flags: 0,
            order_flags: 0,
            action: Action::Starting,
            talk_phase: TalkPhase::Init,
            talk_release: 0,
            hit_count: 0,
            hit_delay_timer: 0,
            attn_set_count: 0,
            tex_pattern_idx: if shape_type == 1 { 2 } else { 0 },
            msg_anm_idx: 0,
            anm_idx: 0,
            anm_loop_count: 0,
            anm_timer: 0.0,
            curr_msg_no: 0,
            msg_no: 0,
            msg_id: FPC_M_ERROR_PROCESS_ID,
            staff_idx: NO_STAFF,
            event_idx: NO_INDEX,
            move_timer: 0,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            scale: 1.0,
            eye_pos: Vec3::ZERO,
            eye_target: Vec3::ZERO,
            attn_base_pos: Vec3::ZERO,
            attn_target: Vec3::ZERO,
            cloud_pos: Vec3::ZERO,
            home: Vec3::ZERO,
            home_angle: 0,
            tornado: None,
            entered: false,
            do_held: false,
        }
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

    pub fn chk_ev_flag(&self, flag: u16) -> bool {
        u16::from(self.ev_flags) & flag == flag
    }

    pub fn set_ev_flag(&mut self, flag: u16) {
        self.ev_flags |= flag as u8;
    }

    pub fn clr_ev_flag(&mut self, flag: u16) {
        self.ev_flags &= !(flag as u8);
    }

    pub fn chk_order(&self, order: u8) -> bool {
        self.order_flags & order != 0
    }

    pub fn set_order(&mut self, order: u8) {
        self.order_flags |= order;
    }

    pub fn clr_order(&mut self) {
        self.order_flags = 0;
    }

    pub fn xy_check_cb(&self, item_btn: u8) -> bool {
        item_btn == ITEM_WIND_WAKER
    }

    pub fn on_hide(&mut self, param: i32) {
        if !self.chk_flag(HR_FLAG_00000010) || param != 0 {
            self.set_flag(HR_FLAG_00000010);
        }
    }

    pub fn off_hide(&mut self, param: i32) {
        if self.chk_flag(HR_FLAG_00000010) || param != 0 {
            self.clr_flag(HR_FLAG_00000010);
        }
    }

    pub fn default_set_pos(&mut self, pos: Vec3) {
        self.attn_base_pos = pos + Vec3::new(0.0, ATTENTION_LIFT, 0.0);
        self.eye_pos = pos + Vec3::new(0.0, EYE_LIFT, 0.0);
    }

    pub fn set_tex_ptn(&mut self, index: i8) {
        if index != self.tex_pattern_idx {
            self.tex_pattern_idx = index;
        }
    }

    pub fn set_anm(&mut self, anm_idx: i8) {
        if anm_idx != self.anm_idx && anm_idx != -1 {
            self.anm_idx = anm_idx;
            self.anm_timer = 0.0;
            self.anm_loop_count = 0;
        }
    }

    pub fn set_anm_status(&mut self) {
        match self.state {
            Step::RtAngry => self.set_anm(ANM_ANGRY),
            _ => {
                if self.shape_type == 1 {
                    self.set_anm(1);
                } else {
                    self.set_anm(0);
                }
            }
        }
    }

    pub fn next_anm(&mut self, next: i8, loop_count: i32) {
        if i32::from(self.anm_loop_count) < loop_count {
            self.anm_loop_count = self.anm_loop_count.wrapping_add(1);
        } else {
            self.set_anm(next);
        }
    }

    pub fn msg_anm(&mut self, attr: u8) {
        if self.msg_anm_idx != attr {
            self.msg_anm_idx = attr;
            if self.msg_anm_idx < 9 {
                let next = MSG_ANM
                    .get(usize::from(self.msg_anm_idx))
                    .copied()
                    .unwrap_or(MSG_ANM[0]);
                self.set_anm(next);
            }
        }
    }

    pub fn check_order(&mut self, ctx: &mut Ctx<'_>) {
        if ctx.ordered && self.chk_order(ORDER_ALL) {
            if self.chk_order(ORDER_TALK_ITEM) && self.xy_check_cb(ctx.shown_item) {
                self.event_idx = index_of(ctx.list, EVENT_TACT_HT);
                self.set_flag(HR_FLAG_00000008);
            } else {
                self.set_flag(HR_FLAG_00000001);
            }
            self.talk_init();
        }
        self.clr_order();
    }

    pub fn next_msg_status(&mut self) -> u8 {
        if MSG_CHAIN.contains(&self.curr_msg_no) {
            self.curr_msg_no = self.curr_msg_no.wrapping_add(1);
            STATUS_MSG_CONTINUES
        } else {
            STATUS_MSG_ENDS
        }
    }

    pub fn get_msg(&mut self, switches: &mut Switches) -> u32 {
        let mut msg_no = 0;
        match self.state {
            Step::RtIntro | Step::RtHit1 | Step::HtTact => {
                msg_no = self.msg_no;
                if msg_no == MSG_LINK && self.sage_type == TYPE_HT {
                    switches.set(self.swbit);
                }
            }
            Step::Talk => msg_no = MSG_PLAIN_TALK,
            _ => {}
        }
        msg_no
    }

    pub fn talk_init(&mut self) {
        self.talk_phase = TalkPhase::Init;
        self.msg_anm_idx = 0xFF;
    }

    pub fn talk(&mut self, pos: Vec3, switches: &mut Switches, text: &mut Text<'_>) -> u8 {
        let mut msg_status = NO_STATUS;
        if self.talk_phase == TalkPhase::Init {
            self.msg_id = FPC_M_ERROR_PROCESS_ID;
            let msg = self.get_msg(switches);
            self.curr_msg_no = msg;
            self.talk_phase = TalkPhase::Create;
        } else if self.talk_phase != TalkPhase::Finished {
            if self.msg_id == FPC_M_ERROR_PROCESS_ID {
                let look_at = pos + Vec3::new(0.0, EYE_LIFT, 0.0);
                self.msg_id = fop_msgm_message_set(
                    text.procs,
                    text.info,
                    text.archive,
                    text.clear_count,
                    self.curr_msg_no,
                    None,
                    look_at,
                );
            } else {
                let attr = text.info.mesg_anime;
                self.msg_anm(attr);
                match self.talk_phase {
                    TalkPhase::Create => {
                        if text.procs.search_by_id(self.msg_id).is_some() {
                            self.talk_phase = TalkPhase::Active;
                        }
                    }
                    TalkPhase::Active => {
                        msg_status = text.status().unwrap_or(NO_STATUS);
                        if msg_status == STATUS_MSG_DISPLAYED {
                            let next = self.next_msg_status();
                            text.set_status(next);
                            if next == STATUS_MSG_CONTINUES {
                                fop_msgm_message_set(
                                    text.procs,
                                    text.info,
                                    text.archive,
                                    text.clear_count,
                                    self.curr_msg_no,
                                    None,
                                    Vec3::ZERO,
                                );
                            }
                        } else if msg_status == STATUS_BOX_CLOSED {
                            text.set_status(STATUS_MSG_DESTROYED);
                            self.talk_phase = TalkPhase::Finished;
                        }
                    }
                    _ => {}
                }
            }
        }
        msg_status
    }

    pub fn init(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.wait_action(actor, ctx);
        self.cloud_pos = actor.pos;
        if self.sage_type == TYPE_RT {
            if ctx.stories.test(EVENT_TACT_START) {
                self.on_hide(1);
            } else {
                self.off_hide(1);
            }
        } else {
            self.on_hide(1);
        }
        self.default_set_pos(actor.pos);
    }

    pub fn inc_attn_set_count(&mut self) {
        if self.attn_set_count != 0xFF {
            self.attn_set_count = self.attn_set_count.wrapping_add(1);
        }
    }

    pub fn set_attention(&mut self, param: bool) {
        if self.chk_flag(HR_FLAG_00000010) || self.chk_flag(HR_FLAG_00000100) {
            return;
        }
        if !param && self.attn_set_count >= 2 {
            return;
        }
        self.eye_target = self.eye_pos;
        self.attn_target = self.attn_base_pos + Vec3::new(0.0, SET_ATTENTION_LIFT, 0.0);
    }

    pub fn ride_tornado(&mut self) -> bool {
        self.speed.x *= TORNADO_DAMP;
        self.speed.y = 0.0;
        self.speed.z *= TORNADO_DAMP;
        false
    }

    pub fn get_look_back_mode(&self) -> LookBack {
        if self.chk_flag(HR_FLAG_00000080) {
            return LookBack::Full;
        }
        if self.chk_flag(HR_FLAG_00000040) {
            return LookBack::Head;
        }
        LookBack::None
    }

    pub fn rt_search(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) -> bool {
        if ctx.stories.test(EVENT_TACT_START) {
            return true;
        }
        let Some(tornado) = ctx.tornado else {
            return false;
        };
        self.tornado = Some(tornado);
        self.state = Step::RtHide;
        self.hit_count = 0;
        actor.pos.y = ctx.home.y - TORNADO_DROP;
        self.scale = SMALL_SCALE;
        self.speed = Vec3::ZERO;
        let name = if ctx.stories.test(EVENT_INTRO_SEEN) {
            EVENT_INTRO_RT
        } else {
            EVENT_INTRO_RT_F
        };
        self.event_idx = index_of(ctx.list, name);
        self.move_timer = RT_TIMER;
        false
    }

    pub fn rt_hide(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) -> bool {
        if let Some(player) = ctx.player {
            actor.angle[1] = target_angle_y(actor.pos, player);
        }
        actor.pos.y -= TORNADO_DROP;
        if ctx.demo {
            self.return_state = Step::RtAngry;
            self.state = Step::RtIntro;
            self.staff_idx = ctx.list.staff_of(STAFF_HR2).unwrap_or(NO_STAFF);
            self.set_ev_flag(EVFLAG_TORNADO_ACTIVE);
            ctx.events.list.push(SimEvent::Se(SE_CV_RC_ENTER));
            ctx.stories.set(EVENT_INTRO_SEEN);
        } else if ctx.tornado_flg {
            if self.move_timer > 0 {
                self.move_timer -= 1;
            } else {
                self.order_event(ctx);
            }
        }
        true
    }

    pub fn rt_intro(&mut self, ctx: &mut Ctx<'_>) -> bool {
        self.rt_hit1(ctx)
    }

    pub fn to_rt_hit(&mut self, ctx: &mut Ctx<'_>) {
        ctx.events.list.push(SimEvent::Se(SE_CV_RC_DAMAGE));
        ctx.events.list.push(SimEvent::Se(SE_CV_RC_MODAE_1));
        self.set_anm(ANM_DAMAGE);
        self.set_tex_ptn(TEX_DAMAGE);
        self.set_flag(HR_FLAG_00000400);
        self.hit_delay_timer = HIT_DELAY;
    }

    pub fn to_rt_tact(&mut self, ctx: &mut Ctx<'_>) {
        ctx.events.list.push(SimEvent::Se(SE_CV_RC_DAMAGE));
        ctx.events.list.push(SimEvent::Se(SE_CV_RC_MODAE_2));
        self.event_idx = index_of(ctx.list, EVENT_TACT0_RT);
        self.return_state = self.state;
        self.state = Step::RtHit0;
        self.order_event(ctx);
        self.hit_delay_timer = HIT_DELAY;
    }

    pub fn rt_angry(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) -> bool {
        actor.angle[0] = actor.angle[0].wrapping_add(ANGLE_STEP);
        let current_y = actor.pos.y;
        let pitch = actor.angle[0];
        self.speed.y = ctx.home.y + SHOT_Y + (1.0 - from_s16(pitch).cos()) * SHOT_Y_ARC - current_y;
        self.scale = SMALL_SCALE;
        if let Some(player) = ctx.player {
            actor.angle[1] = target_angle_y(actor.pos, player);
        }
        self.ride_tornado();
        if ctx.tornado_up {
            self.state = Step::RtWin;
            self.set_anm(ANM_TALK_03);
            ctx.events.list.push(SimEvent::Se(SE_CV_RC_ENTER));
            return true;
        } else if self.hit_delay_timer != 0 {
            return true;
        } else {
            if self.chk_flag(HR_FLAG_00000400) {
                self.clr_flag(HR_FLAG_00000400);
                self.set_anm(ANM_ANGRY);
                self.set_tex_ptn(TEX_ANGRY);
            }
            if ctx.hit {
                self.hit_count = self.hit_count.wrapping_add(1);
                if self.hit_count >= HIT_COUNT_MAX {
                    self.to_rt_tact(ctx);
                } else {
                    self.to_rt_hit(ctx);
                }
            }
        }
        true
    }

    pub fn rt_win(&mut self) -> bool {
        self.speed.y = 0.0;
        self.ride_tornado();
        true
    }

    pub fn rt_hit0(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if ctx.demo {
            self.state = Step::RtHit1;
            self.staff_idx = ctx.list.staff_of(STAFF_HR2).unwrap_or(NO_STAFF);
            self.set_ev_flag(EVFLAG_TORNADO_ACTIVE);
        } else if ctx.tornado_up {
            self.state = Step::RtWin;
            self.set_anm(ANM_TALK_03);
            ctx.events.list.push(SimEvent::Se(SE_CV_RC_ENTER));
        } else {
            self.order_event(ctx);
        }
        self.speed.y = 0.0;
        self.ride_tornado();
        true
    }

    pub fn rt_hit1(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.chk_ev_flag(EVFLAG_TORNADO_ACTIVE) {
            self.ride_tornado();
        }
        if event_ended(self.event_idx, ctx.active) {
            if self.return_state == Step::Wait02 {
                self.on_hide(0);
            }
            if self.state == Step::RtIntro {
                ctx.events.list.push(SimEvent::Bgm(BGM_DIOCTA_BATTLE));
            }
            self.end_talk();
            return false;
        }
        true
    }

    pub fn ht_hide(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) -> bool {
        if self.chk_flag(HR_FLAG_00000008) {
            ctx.events.list.push(SimEvent::Se(SE_PRE_TAKT));
            self.return_state = self.state;
            self.state = Step::HtTact;
            self.set_anm_status();
            self.staff_idx = ctx.list.staff_of(STAFF_HR).unwrap_or(NO_STAFF);
            ctx.switches.set(self.swbit);
        } else if self.chk_flag(HR_FLAG_00000001) {
            self.return_state = self.state;
            self.state = Step::Talk;
            self.set_anm_status();
        } else if let Some(player) = ctx.player
            && seen_actor_angle_y(player, ctx.player_face, actor.pos) < TALK_CONE
        {
            self.set_order(ORDER_TALK);
            if !ctx.stories.test(EVENT_TACT_DONE) {
                self.set_order(ORDER_TALK_ITEM);
            }
        }
        true
    }

    pub fn wait01(&mut self) -> bool {
        if self.chk_flag(HR_FLAG_00000001) {
            self.return_state = self.state;
            self.state = Step::Talk;
            self.set_anm_status();
        } else {
            self.set_order(ORDER_TALK);
        }
        true
    }

    pub fn wait02(&mut self) -> bool {
        true
    }

    pub fn end_talk(&mut self) {
        self.state = self.return_state;
        self.set_anm_status();
        self.clr_flag(HR_FLAG_00000008 | HR_FLAG_00000001);
        self.talk_release = 5;
    }

    pub fn end_tact(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.on_hide(0);
        self.default_set_pos(ctx.home);
        actor.angle[1] = ctx.home_angle;
        self.speed = Vec3::ZERO;
        self.speed_f = 0.0;
        if self.sage_type == TYPE_HT {
            ctx.switches.clear(self.swbit);
        }
    }

    pub fn drawn(&self) -> bool {
        !self.chk_flag(HR_FLAG_00000010)
    }

    pub fn cull(&self) -> (f32, f32) {
        (self.scale * BODY_RADIUS, self.scale * BODY_HEIGHT)
    }

    pub fn smoke_proc(&mut self) {
        let hidden = self.chk_flag(HR_FLAG_00000010);
        let active = !hidden && self.sage_type == TYPE_RT && self.hit_delay_timer != 0;
        if active {
            self.hit_delay_timer -= 1;
        }
    }

    pub fn talk01(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) -> bool {
        let status = self.talk(actor.pos, ctx.switches, ctx.text);
        if status == STATUS_BOX_CLOSED {
            self.end_talk();
        }
        true
    }

    pub fn ht_tact01(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) -> bool {
        if event_ended(self.event_idx, ctx.active) {
            self.end_talk();
            self.end_tact(actor, ctx);
            return false;
        }
        true
    }

    pub fn wait_action(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) -> bool {
        if self.action == Action::Starting {
            self.state = match self.sage_type {
                TYPE_HT => Step::HtHide,
                TYPE_RT => {
                    if ctx.stories.test(EVENT_TACT_START) {
                        Step::Wait02
                    } else {
                        Step::RtSearch
                    }
                }
                _ => Step::Wait01,
            };
            self.set_anm_status();
            self.action = Action::Ongoing;
        } else if self.action != Action::Ending {
            self.clr_flag(HR_FLAG_00000100 | HR_FLAG_00000080 | HR_FLAG_00000040);
            let temp = match self.state {
                Step::Wait01 => self.wait01(),
                Step::Wait02 => self.wait02(),
                Step::Talk => self.talk01(actor, ctx),
                Step::HtTact => self.ht_tact01(actor, ctx),
                Step::RtSearch => self.rt_search(actor, ctx),
                Step::RtHide => self.rt_hide(actor, ctx),
                Step::RtIntro => self.rt_intro(ctx),
                Step::RtAngry => self.rt_angry(actor, ctx),
                Step::RtWin => self.rt_win(),
                Step::RtHit0 => self.rt_hit0(ctx),
                Step::RtHit1 => self.rt_hit1(ctx),
                Step::HtHide => self.ht_hide(actor, ctx),
                _ => false,
            };
            self.set_attention(temp);
            self.smoke_proc();
        }
        true
    }

    pub fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.check_order(ctx);
        self.wait_action(actor, ctx);
        if !self.chk_flag(HR_FLAG_00000020) {
            actor.pos += self.speed;
        }
        self.clr_flag(HR_FLAG_00000020);
    }

    fn order_event(&mut self, ctx: &mut Ctx<'_>) {
        if self.event_idx < 0 {
            return;
        }
        let index = usize::from(self.event_idx as u16);
        if ctx.active.stage_of(index).is_none() {
            ctx.active.order(ctx.list, index, ctx.entity);
        }
    }
}

struct Text<'a> {
    procs: &'a mut MsgProcs,
    info: &'a mut MsgGameInfo,
    archive: &'a dyn MsgArchive,
    clear_count: u8,
    one: &'a mut Msg1Box,
    two: &'a mut Msg2Box,
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
}

struct Ctx<'a> {
    entity: Entity,
    player: Option<Vec3>,
    player_face: i16,
    home: Vec3,
    home_angle: i16,
    stories: &'a mut StoryBits,
    switches: &'a mut Switches,
    events: &'a mut SimEvents,
    list: &'a EventList,
    active: &'a mut ActiveEvent,
    text: Text<'a>,
    tornado: Option<Entity>,
    tornado_flg: bool,
    tornado_up: bool,
    demo: bool,
    hit: bool,
    ordered: bool,
    shown_item: u8,
}

fn target_angle_y(from: Vec3, to: Vec3) -> i16 {
    to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn seen_actor_angle_y(from: Vec3, facing: i16, to: Vec3) -> i32 {
    let angle = target_angle_y(from, to);
    i32::from(angle.wrapping_sub(facing).unsigned_abs())
}

fn index_of(list: &EventList, name: &str) -> i16 {
    list.event_index(name)
        .map_or(NO_INDEX, |index| i16::try_from(index).unwrap_or(NO_INDEX))
}

fn event_ended(index: i16, active: &ActiveEvent) -> bool {
    index >= 0 && active.end_check(usize::from(index as u16))
}

pub fn is_sage(name: &str) -> bool {
    SAGE_ACTORS.contains(&name)
}

pub fn control(
    do_intent: Res<DoIntent>,
    equipped: Res<Equipped>,
    flags: Res<TextFlags>,
    archive: Option<Res<MsgArchives>>,
    list: Res<EventList>,
    mut stories: ResMut<StoryBits>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut active: ResMut<ActiveEvent>,
    mut procs: ResMut<MsgProcs>,
    mut info: ResMut<MsgGameInfo>,
    mut one: ResMut<Msg1Box>,
    mut two: ResMut<Msg2Box>,
    players: Query<&Player>,
    mut actors: Query<(Entity, &mut Actor, &mut Sage)>,
) {
    let Some(archive) = archive.as_deref() else {
        return;
    };
    let clear_count = flags.clear_count;
    let player = players.iter().next().copied();
    let player_face = player.map_or(0, |link| to_s16(link.face));
    let tornado = actors
        .iter()
        .find(|(_, actor, _)| actor.name == TORNADO_ACTOR)
        .map(|(found, _, _)| *found);
    let shown_item = if equipped.item(WAKER_SLOT) == ITEM_WIND_WAKER {
        ITEM_WIND_WAKER
    } else {
        ITEM_NONE
    };
    for (entity, actor, sage) in &mut actors {
        let actor = &mut *actor;
        let sage = &mut *sage;
        let ordered = do_intent.0 && !sage.do_held;
        sage.do_held = do_intent.0;
        let fresh = !sage.entered;
        if fresh {
            sage.entered = true;
            sage.home = actor.pos;
            sage.home_angle = actor.angle[1];
        }
        let mut ctx = Ctx {
            entity,
            player: player.map(|link| link.pos),
            player_face,
            home: sage.home,
            home_angle: sage.home_angle,
            stories: &mut *stories,
            switches: &mut *switches,
            events: &mut *events,
            list: &*list,
            active: &mut *active,
            text: Text {
                procs: &mut *procs,
                info: &mut *info,
                archive: archive as &dyn MsgArchive,
                clear_count,
                one: &mut *one,
                two: &mut *two,
            },
            tornado,
            tornado_flg: false,
            tornado_up: false,
            demo: false,
            hit: false,
            ordered,
            shown_item,
        };
        if fresh {
            sage.init(actor, &mut ctx);
        } else {
            sage.execute(actor, &mut ctx);
        }
    }
}
