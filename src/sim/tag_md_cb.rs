use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::carry::Carry;
use super::event::{ActiveEvent, EventList, Stage};
use super::event_manager::EventManager;
use super::events::{SimEvent, SimEvents};
use super::flags::{Collect, EventFlags, Switches};
use super::intent::DoIntent;
use super::inventory::{self, Equipped};
use super::item;
use super::makar::Makar;
use super::medli::Medli;
use super::obj_doguu::Text;
use super::player::Player;
use super::registry;
use super::vibration::Vibration;
use crate::assets::MsgArchives;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgget::MsgArchive;
use crate::text::msgproc::{FPC_M_ERROR_PROCESS_ID, MsgGameInfo, MsgProcs};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_INPUT, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED,
    STATUS_MSG_DISPLAYED, STATUS_MSG_ENDS, STATUS_TYPING,
};

pub const TAG_MD_CB_ACTORS: [&str; 3] = ["TagCb1", "TagCb11", "TagCb12"];

pub fn is_tag_md_cb(name: &str) -> bool {
    TAG_MD_CB_ACTORS.contains(&name)
}

pub const PRM_MSG_SHIFT: u32 = 0;
pub const PRM_MSG_MASK: u32 = 0xFFFF;
pub const PRM_SW_SHIFT: u32 = 0x10;
pub const PRM_SW_MASK: u32 = 0xFF;
pub const PRM_SW_COUNT_SHIFT: u32 = 0x18;
pub const ANGLE_SW_MASK: u16 = 0xFF;

pub const ARG_MD1: i32 = 0;
pub const ARG_CB1: i32 = 1;

pub const EVENT_BIT_TABLE: [u32; 2] = [0x2920, 0x2910];

pub const EVENT_NAME_TABLE: [&str; 7] = [
    "md_tag_message",
    "md_tag_message_carry",
    "md_tag_message_fm",
    "cb_tag_message",
    "cb_tag_message_carry",
    "cb_tag_message_fm",
    "cb_tag_message_sekizo",
];

pub const STAFF_NAME_TABLE: [&str; 12] = [
    "TagMd1",
    "TagCb1",
    "MD_MESSAGE_TAG",
    "MD_MESSAGE_TAG",
    "MD_MESSAGE_TAG",
    "MD_MESSAGE_TAG",
    "MD_MESSAGE_TAG",
    "MD_MESSAGE_TAG",
    "MD_MESSAGE_TAG",
    "MD_MESSAGE_TAG",
    "MD_MESSAGE_TAG",
    "MD_MESSAGE_TAG",
];

pub const CUT_NAME_TABLE: [&str; 6] = [
    "wait",
    "init",
    "msg_init",
    "message",
    "player_off",
    "player_on",
];

pub const CUT_INIT: i32 = 1;
pub const CUT_MSG_INIT: i32 = 2;
pub const CUT_MESSAGE: i32 = 3;
pub const CUT_PLAYER_OFF: i32 = 4;
pub const CUT_PLAYER_ON: i32 = 5;

pub const HIO_M00: f32 = 200.0;
pub const HIO_M04: f32 = 200.0;
pub const HIO_M08: i16 = 900;
pub const HIO_M0A: i16 = 150;
pub const HIO_NO_NONE: i8 = -1;

pub const SCALE_MESSAGE: f32 = 50.0;
pub const SCALE_ARG3_X: f32 = 2250.0;
pub const TIMER_ARG3: i16 = 60;

pub const EVENT_UNK_3301: u32 = 0x3301;
pub const EVENT_UNK_3302: u32 = 0x3302;
pub const EVENT_UNK_3304: u32 = 0x3304;
pub const EVENT_UNK_3308: u32 = 0x3308;
pub const EVENT_UNK_3310: u32 = 0x3310;
pub const EVENT_UNK_3320: u32 = 0x3320;
pub const EVENT_UNK_3404: u32 = 0x3404;
pub const EVENT_UNK_3408: u32 = 0x3408;
pub const EVENT_UNK_3410: u32 = 0x3410;
pub const EVENT_UNK_3420: u32 = 0x3420;
pub const EVENT_UNK_3440: u32 = 0x3440;
pub const EVENT_UNK_3480: u32 = 0x3480;
pub const EVENT_UNK_4001: u32 = 0x4001;
pub const EVENT_UNK_4180: u32 = 0x4180;

pub const EVENT_INFO_NONE: u8 = 0xFF;
pub const STAFF_IDX_NONE: i32 = -1;
pub const ACT_IDX_NONE: i32 = -1;
pub const ORDER_NONE: i8 = -1;
pub const CUT_NONE: i8 = -1;

pub const ROOM_NONE: i8 = -1;
pub const COND_ARG2_DIST2: f32 = 250000.0;
pub const AREA_MIN_DY: f32 = -10.0;

pub const COLLECT_TACT: u8 = 1;
pub const COLLECT_TACT_ITEM: u8 = 1;

pub const MSG_MD_HERE: u32 = 0x19F0;
pub const MSG_MD_HERE_END: u32 = 0x19F1;
pub const MSG_MD_HERE_2: u32 = 0x19F2;
pub const MSG_MD_AFTER: u32 = 0x19F3;
pub const MSG_MD_FINISH: u32 = 0x19F4;
pub const MSG_MD_FINISH_2: u32 = 0x19F5;
pub const MSG_CB_HERE: u32 = 0x1530;
pub const MSG_CB_AFTER: u32 = 0x1531;
pub const MSG_CB_LAST: u32 = 0x1533;
pub const MSG_CB_FINISH: u32 = 0x1534;
pub const MSG_CB_TAIL: u32 = 0x1535;
pub const MSG_KEY: &str = "msg_number";

pub const SE_CV_MD_I_AM_HERE: u32 = 0x4921;
pub const SE_CV_CB_I_AM_HERE: u32 = 0x491B;

pub const SHOCK_PATTERN: i32 = 4;
pub const SHOCK_FLAGS: i32 = -0x21;
pub const SHOCK_COORD: Vec3 = Vec3::new(0.0, 1.0, 0.0);

pub const CULL_MIN: [f32; 3] = [0.0, 0.0, 0.0];
pub const CULL_MAX: [f32; 3] = [0.0, 0.0, 0.0];

pub const EVENT_ACCEPT_BIT: u32 = 1;

pub const ORDER_MD_DEFAULT: i8 = 0;
pub const ORDER_MD_CARRY: i8 = 1;
pub const ORDER_MD_ARG5: i8 = 2;
pub const ORDER_CB_DEFAULT: i8 = 3;
pub const ORDER_CB_CARRY: i8 = 4;
pub const ORDER_CB_ARG9: i8 = 5;
pub const ORDER_CB_ARG11: i8 = 6;

pub const M2BC_OFFSET: i32 = 0x2BC;

#[derive(Resource, Debug, Clone, Copy)]
pub struct TagMdCbHio {
    pub m_no: i8,
    pub m00: f32,
    pub m04: f32,
    pub m08: i16,
    pub m0a: i16,
    pub counter: i32,
}

impl Default for TagMdCbHio {
    fn default() -> Self {
        Self {
            m_no: HIO_NO_NONE,
            m00: HIO_M00,
            m04: HIO_M04,
            m08: HIO_M08,
            m0a: HIO_M0A,
            counter: 0,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct TagMdCbGlobal {
    pub talk_partner: Option<Entity>,
    pub npc_message_animation: u8,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct TagMdCbMsg {
    pub pid: u32,
}

impl Default for TagMdCbMsg {
    fn default() -> Self {
        Self {
            pid: FPC_M_ERROR_PROCESS_ID,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagMdCbAction {
    Warp,
    Message,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct TagMdCb {
    pub health: i32,
    pub max_health: i32,
    pub argument: i32,
    pub m29c: i32,
    pub m2a0: i32,
    pub m2a4: u32,
    pub m2a8: u32,
    pub m2ac: u32,
    pub m2b0: u16,
    pub m2b2: i16,
    pub m2b4: i8,
    pub m2b5: i8,
    pub m2b6: i8,
    pub m2b7: u8,
    pub m2b8: u8,
    pub m2b9: u8,
    pub m2ba: u8,
    pub m2bc: [i16; 7],
    pub m2ca: i16,
    pub m2cc: i16,
    pub m2ce: i16,
    pub m2d0: i16,
    pub m2d8: f32,
    pub action: Option<TagMdCbAction>,
    pub entered: bool,
    pub failed: bool,
}

impl TagMdCb {
    pub fn spawn(parameters: u32) -> Self {
        let _ = parameters;
        Self {
            health: 0,
            max_health: 0,
            argument: ARG_MD1,
            m29c: 0,
            m2a0: 0,
            m2a4: 0,
            m2a8: 0,
            m2ac: 0,
            m2b0: 0,
            m2b2: 0,
            m2b4: ORDER_NONE,
            m2b5: CUT_NONE,
            m2b6: -1,
            m2b7: 0,
            m2b8: 0,
            m2b9: 0,
            m2ba: 0,
            m2bc: [0; 7],
            m2ca: 0,
            m2cc: 0,
            m2ce: 0,
            m2d0: 0,
            m2d8: 0.0,
            action: None,
            entered: false,
            failed: false,
        }
    }

    pub fn argument_of(name: &str) -> i32 {
        match name {
            "TagCb1" => 1,
            "TagCb11" => 8,
            "TagCb12" => 9,
            _ => 0,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    pub fn off_event_accept(&mut self) {
        self.m2ac &= !EVENT_ACCEPT_BIT;
    }

    pub fn on_event_accept(&mut self) {
        self.m2ac |= EVENT_ACCEPT_BIT;
    }

    pub fn set_timer(&mut self, value: i16) {
        self.m2b2 = value;
    }

    pub fn calc_timer(&mut self) -> bool {
        if self.m2b2 != 0 {
            self.m2b2 -= 1;
        }
        self.m2b2 == 0
    }

    pub fn check_timer(&mut self) -> bool {
        self.calc_timer()
    }

    fn byte_at(&self, offset: i32) -> Option<u8> {
        match offset {
            0x2B0 => Some(self.m2b0 as u8),
            0x2B1 => Some((self.m2b0 >> 8) as u8),
            0x2B2 => Some(self.m2b2 as u8),
            0x2B3 => Some((self.m2b2 >> 8) as u8),
            0x2B4 => Some(self.m2b4 as u8),
            0x2B5 => Some(self.m2b5 as u8),
            0x2B6 => Some(self.m2b6 as u8),
            0x2B7 => Some(self.m2b7),
            0x2B8 => Some(self.m2b8),
            0x2B9 => Some(self.m2b9),
            0x2BA => Some(self.m2ba),
            0x2BB => Some(0),
            0x2BC..=0x2C9 => {
                let half = usize::try_from((offset - M2BC_OFFSET) / 2).unwrap_or(0);
                let high = (offset - M2BC_OFFSET) % 2 == 1;
                let value = self.m2bc.get(half).copied().unwrap_or(0) as u16;
                Some(if high { (value >> 8) as u8 } else { value as u8 })
            }
            0x2CA => Some(self.m2ca as u8),
            0x2CB => Some((self.m2ca >> 8) as u8),
            0x2CC => Some(self.m2cc as u8),
            0x2CD => Some((self.m2cc >> 8) as u8),
            0x2CE => Some(self.m2ce as u8),
            0x2CF => Some((self.m2ce >> 8) as u8),
            0x2D0 => Some(self.m2d0 as u8),
            0x2D1 => Some((self.m2d0 >> 8) as u8),
            _ => None,
        }
    }

    pub fn event_slot(&self, index: i32) -> i32 {
        let base = M2BC_OFFSET + index * 2;
        match (self.byte_at(base), self.byte_at(base + 1)) {
            (Some(low), Some(high)) => i32::from(i16::from_le_bytes([low, high])),
            _ => 0,
        }
    }

    pub fn check_event_finish(&self, flags: &EventFlags) -> bool {
        match self.argument {
            2 => flags.test(EVENT_UNK_3310),
            3 => flags.test(EVENT_UNK_4180),
            4 => flags.test(EVENT_UNK_3308),
            5 => flags.test(EVENT_UNK_3304),
            6 => flags.test(EVENT_UNK_3302),
            7 => flags.test(EVENT_UNK_3301),
            8 => flags.test(EVENT_UNK_3480),
            9 => flags.test(EVENT_UNK_3440),
            10 => flags.test(EVENT_UNK_3420),
            11 => flags.test(EVENT_UNK_3410),
            _ => false,
        }
    }

    pub fn create(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if self.argument <= 1 {
            let bit = EVENT_BIT_TABLE.get(self.argument as usize).copied().unwrap_or(0);
            if ctx.flags.test(bit) {
                self.failed = true;
                return false;
            }
        } else {
            if self.argument == 11 {
                ctx.flags.clear(EVENT_UNK_3410);
            }
            if self.check_event_finish(ctx.flags) {
                self.failed = true;
                return false;
            }
        }
        if ctx.hio.m_no < 0 {
            ctx.hio.m_no = 0;
            ctx.hio.counter = 1;
        } else {
            ctx.hio.counter += 1;
        }
        if self.argument > 1 {
            ctx.actor.scale *= SCALE_MESSAGE;
        }
        self.init(ctx);
        self.entered = true;
        true
    }

    pub fn init(&mut self, ctx: &mut Ctx<'_>) {
        let parameters = ctx.actor.parameters;
        self.m29c = ((parameters >> PRM_MSG_SHIFT) & PRM_MSG_MASK) as i32;
        self.m2a0 = ((parameters >> PRM_SW_SHIFT) & PRM_SW_MASK) as i32;
        self.m2a4 = (parameters >> PRM_SW_COUNT_SHIFT) & 0xFF;
        self.m2b0 = (ctx.actor.angle[0] as u16) & ANGLE_SW_MASK;
        for (slot, name) in self.m2bc.iter_mut().zip(EVENT_NAME_TABLE.iter()) {
            *slot = event_index(ctx.manager, name) as i16;
        }
        if self.argument > 1 {
            self.set_action(TagMdCbAction::Message, ctx);
        } else {
            self.set_action(TagMdCbAction::Warp, ctx);
        }
        self.m2b4 = ORDER_NONE;
        self.m2b7 = 0;
        self.m2ba = 0;
        if self.argument == 8 || self.argument == 11 {
            self.set_timer(0);
        }
        if self.argument == 3 {
            self.set_timer(TIMER_ARG3);
            ctx.actor.scale.x = SCALE_ARG3_X;
        } else {
            self.set_timer(ctx.hio.m0a);
        }
    }

    pub fn set_action(&mut self, func: TagMdCbAction, ctx: &mut Ctx<'_>) -> bool {
        if self.action != Some(func) {
            if let Some(previous) = self.action {
                self.m2b6 = -1;
                self.run_action(previous, ctx);
            }
            self.action = Some(func);
            self.m2b6 = 0;
            self.m2ca = 0;
            self.m2cc = 0;
            self.m2ce = 0;
            self.m2d0 = 0;
            self.m2d8 = 0.0;
            self.run_action(func, ctx);
        }
        true
    }

    pub fn action(&mut self, ctx: &mut Ctx<'_>) {
        if self.action.is_none() {
            self.set_action(TagMdCbAction::Warp, ctx);
        }
        if let Some(func) = self.action {
            self.run_action(func, ctx);
        }
    }

    pub fn run_action(&mut self, func: TagMdCbAction, ctx: &mut Ctx<'_>) {
        match func {
            TagMdCbAction::Warp => {
                self.warp_action(ctx);
            }
            TagMdCbAction::Message => {
                self.message_action(ctx);
            }
        }
    }

    pub fn warp_action(&mut self, ctx: &mut Ctx<'_>) {
        if self.m2b6 == 0 {
            self.m2b6 += 1;
        } else if self.m2b6 != -1 && self.argument <= 1 {
            let bit = EVENT_BIT_TABLE.get(self.argument as usize).copied().unwrap_or(0);
            ctx.flags.set(bit);
        }
    }

    pub fn message_action(&mut self, ctx: &mut Ctx<'_>) {
        if self.m2b6 == 0 {
            self.m2b6 += 1;
        } else if self.m2b6 != -1 && self.argument > 1 && !self.check_event_finish(ctx.flags) {
            if ctx.npc.present && ctx.npc.tag_check_ok {
                if ctx.npc.is_md {
                    if self.argument == 5 {
                        self.m2b4 = ORDER_MD_ARG5;
                    } else if ctx.npc.carry_now {
                        if !ctx.npc.no_carry_action {
                            self.m2b4 = ORDER_MD_CARRY;
                        }
                    } else {
                        self.m2b4 = ORDER_MD_DEFAULT;
                    }
                } else if self.argument == 9 {
                    self.m2b4 = ORDER_CB_ARG9;
                } else if self.argument == 11 {
                    self.m2b4 = ORDER_CB_ARG11;
                } else if ctx.npc.carry_now {
                    if !ctx.npc.no_carry_action {
                        self.m2b4 = ORDER_CB_CARRY;
                    }
                } else {
                    self.m2b4 = ORDER_CB_DEFAULT;
                }
            }
        }
    }

    pub fn talk_init(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if ctx.msg.pid == FPC_M_ERROR_PROCESS_ID {
            ctx.msg.pid = message_set_actor(ctx, self.m2a8);
        } else if ctx
            .text
            .alive(ctx.msg.pid)
        {
            return true;
        }
        false
    }

    pub fn talk(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let status = ctx.text.status(ctx.msg.pid);
        if status == STATUS_MSG_DISPLAYED {
            let next = self.next_msg_status(ctx);
            ctx.text.set_status(ctx.msg.pid, next);
            if next == STATUS_MSG_CONTINUES {
                message_set(ctx, self.m2a8);
            }
        } else if status != STATUS_INPUT {
            if status == STATUS_TYPING {
                if matches!(self.m2b5, 0 | 2 | 3 | 5 | 6) {
                    ctx.npc_wants_message_animation = Some(ctx.text.info.mesg_anime);
                }
            } else if status == STATUS_BOX_CLOSED {
                ctx.text.set_status(ctx.msg.pid, STATUS_MSG_DESTROYED);
                return true;
            }
        }
        false
    }

    pub fn next_msg_status(&mut self, ctx: &mut Ctx<'_>) -> u8 {
        let mut status = STATUS_MSG_ENDS;
        match self.m2a8 {
            MSG_MD_HERE => {
                ctx.flags.set(EVENT_UNK_3320);
                self.set_timer(ctx.hio.m08);
            }
            MSG_MD_HERE_END => {
                ctx.flags.set(EVENT_UNK_3310);
                ctx.flags.set(EVENT_UNK_4001);
                self.set_timer(ctx.hio.m08);
            }
            MSG_MD_HERE_2 => {
                self.set_timer(ctx.hio.m08);
            }
            MSG_MD_AFTER => {
                ctx.flags.set(EVENT_UNK_3304);
            }
            MSG_MD_FINISH => {
                self.set_timer(ctx.hio.m08);
            }
            MSG_MD_FINISH_2 => {
                ctx.flags.set(EVENT_UNK_3301);
            }
            MSG_CB_HERE => {
                ctx.flags.set(EVENT_UNK_3480);
            }
            MSG_CB_AFTER => {
                ctx.flags.set(EVENT_UNK_3440);
            }
            MSG_CB_LAST => {
                self.m2a8 = MSG_CB_TAIL;
                status = STATUS_MSG_CONTINUES;
            }
            MSG_CB_FINISH => {
                ctx.flags.set(EVENT_UNK_3420);
            }
            MSG_CB_TAIL => {
                ctx.flags.set(EVENT_UNK_3410);
            }
            _ => {}
        }
        status
    }

    pub fn get_my_staff_id(&self, ctx: &Ctx<'_>) -> i32 {
        let name = STAFF_NAME_TABLE
            .get(self.argument.max(0) as usize)
            .copied()
            .unwrap_or("MD_MESSAGE_TAG");
        ctx.manager.get_my_staff_id(name, None, 0)
    }

    pub fn event_proc(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if !self.check_command_demo_accrpt(ctx) {
            return false;
        }
        if self.m2b4 != ORDER_NONE {
            self.on_event_accept();
            self.m2b4 = ORDER_NONE;
        }
        let staff_idx = self.get_my_staff_id(ctx);
        if ctx.text.info.event_run_check() && !self.check_command_talk(ctx) {
            if staff_idx != STAFF_IDX_NONE {
                let act = ctx
                    .manager
                    .get_my_act_idx(staff_idx, &CUT_NAME_TABLE, true, 0);
                if act == ACT_IDX_NONE {
                    ctx.manager.cut_end(staff_idx);
                } else {
                    if ctx.manager.get_is_advance(staff_idx) {
                        self.initial(staff_idx, act, ctx);
                    }
                    if self.action_of(act, ctx) {
                        ctx.manager.cut_end(staff_idx);
                    }
                }
            }
            let running = self.event_slot(i32::from(self.m2b5));
            if let Ok(slot) = usize::try_from(running) {
                if ctx.active.end_check(slot) {
                    self.event_end(ctx);
                }
            }
            return true;
        }
        false
    }

    pub fn check_command_demo_accrpt(&self, ctx: &Ctx<'_>) -> bool {
        for index in 0..self.m2bc.len() {
            if matches!(
                ctx.active.stage_of(index),
                Some(Stage::Play) | Some(Stage::Close)
            ) {
                return true;
            }
        }
        false
    }

    pub fn event_end(&mut self, ctx: &mut Ctx<'_>) {
        ctx.manager.exception.init();
        self.off_event_accept();
        self.m2b5 = CUT_NONE;
    }

    pub fn initial(&mut self, staff_idx: i32, act: i32, ctx: &mut Ctx<'_>) {
        match act {
            CUT_INIT => self.initial_init_event(ctx),
            CUT_MSG_INIT => self.initial_msg_set_event(staff_idx, ctx),
            CUT_PLAYER_OFF => self.initial_player_off_draw(ctx),
            CUT_PLAYER_ON => self.initial_player_on_draw(ctx),
            _ => self.initial_default(),
        }
    }

    pub fn initial_default(&mut self) {}

    pub fn initial_init_event(&mut self, ctx: &mut Ctx<'_>) {
        ctx.talk_partner = ctx.player_entity;
    }

    pub fn initial_msg_set_event(&mut self, staff_idx: i32, ctx: &mut Ctx<'_>) {
        ctx.msg.pid = FPC_M_ERROR_PROCESS_ID;
        match ctx.manager.get_my_int(staff_idx, MSG_KEY) {
            Some(value) => self.m2a8 = value as u32,
            None => self.m2a8 = self.m29c as u32,
        }
        if self.m2a8 == MSG_MD_AFTER || self.m2a8 == MSG_CB_AFTER {
            if ctx.player_pos.is_some() {
                let sound = if self.m2a8 == MSG_MD_AFTER {
                    SE_CV_MD_I_AM_HERE
                } else {
                    SE_CV_CB_I_AM_HERE
                };
                ctx.vibration
                    .start_shock(SHOCK_PATTERN, SHOCK_FLAGS, SHOCK_COORD);
                ctx.events.list.push(SimEvent::Se(sound));
            }
        }
    }

    pub fn initial_player_off_draw(&mut self, ctx: &mut Ctx<'_>) {
        ctx.player.no_draw = true;
    }

    pub fn initial_player_on_draw(&mut self, ctx: &mut Ctx<'_>) {
        ctx.player.no_draw = false;
    }

    pub fn action_of(&mut self, act: i32, ctx: &mut Ctx<'_>) -> bool {
        match act {
            CUT_MSG_INIT => self.talk_init(ctx),
            CUT_MESSAGE => self.talk(ctx),
            _ => self.action_default(),
        }
    }

    pub fn action_default(&mut self) -> bool {
        true
    }

    pub fn event_order(&mut self, ctx: &mut Ctx<'_>) {
        if self.check_event_finish(ctx.flags) {
            self.m2b4 = ORDER_NONE;
        }
        if self.m2b4 != ORDER_NONE && self.m2b4 < self.m2bc.len() as i8 {
            self.m2b5 = self.m2b4;
            let slot = usize::try_from(self.m2b4).unwrap_or(0);
            ctx.active.order(ctx.list, slot, ctx.entity);
        }
    }

    pub fn check_command_talk(&self, ctx: &Ctx<'_>) -> bool {
        ctx.talk_command
    }

    pub fn check_area_in(&mut self, ctx: &Ctx<'_>, other: Option<Vec3>) -> bool {
        let Some(point) = other else {
            return false;
        };
        let distance = registry::actor_distance_xz2(ctx.actor.pos, point);
        let dy = point.y - ctx.actor.pos.y;
        let limit = ctx.actor.scale.x;
        if distance < limit * limit && dy >= AREA_MIN_DY && dy < ctx.actor.scale.y {
            self.m2b9 = 1;
            return true;
        }
        false
    }

    pub fn check_condition(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let Some(player_pos) = ctx.player_pos else {
            return false;
        };
        let room = ctx.actor.room;
        let player_room = ctx.player_room.unwrap_or(ROOM_NONE);
        if player_room == ROOM_NONE || player_room != room {
            return false;
        }
        let mut result = false;
        let argument = self.argument;
        if argument <= 1 {
            if self.check_area_in(ctx, Some(player_pos)) {
                result = true;
            }
        } else if argument == 2 {
            if !ctx.flags.test(EVENT_UNK_3310)
                && ctx.switches.test(self.m2b0 as u8)
                && ctx
                    .link_pos
                    .map_or(false, |link| {
                        registry::actor_distance_xz2(player_pos, link) < COND_ARG2_DIST2
                    })
            {
                result = self.check_timer();
            }
        } else if argument == 3 {
            if ctx.switches.test(self.m2b0 as u8) {
                ctx.flags.set(EVENT_UNK_4180);
                ctx.flags.set(EVENT_UNK_3310);
            }
            if !ctx.flags.test(EVENT_UNK_4180) && ctx.flags.test(EVENT_UNK_3310) {
                if ctx.flags.test(EVENT_UNK_4001) {
                    result = self.check_timer();
                } else if self.check_area_in(ctx, ctx.link_pos)
                    || self.check_area_in(ctx, Some(player_pos))
                {
                    result = self.check_timer();
                }
            }
        } else if argument == 4 {
            if ctx.switches.test(self.m2b0 as u8) {
                ctx.flags.set(EVENT_UNK_3308);
            }
            if ctx.switches.test(self.m2a0 as u8)
                && !ctx.flags.test(EVENT_UNK_3308)
                && self.check_area_in(ctx, Some(player_pos))
            {
                result = self.check_timer();
            }
        } else if argument == 5 {
            if ctx.flags.test(EVENT_UNK_3404) && self.check_area_in(ctx, ctx.link_pos) {
                result = true;
            }
        } else if argument == 6 {
            if ctx.collect.test(COLLECT_TACT, COLLECT_TACT_ITEM) {
                if ctx.switches.test(self.m2b0 as u8) {
                    ctx.flags.set(EVENT_UNK_3302);
                }
                if !ctx.flags.test(EVENT_UNK_3302) {
                    let mut index = 0u32;
                    let mut switch = self.m2a0 as u32;
                    while index < self.m2a4 {
                        if ctx.switches.test(switch as u8) {
                            self.m2ba = 1;
                            break;
                        }
                        index += 1;
                        switch += 1;
                    }
                    if self.check_area_in(ctx, Some(player_pos)) && self.m2ba != 0 {
                        result = self.check_timer();
                    }
                }
            }
        } else if argument == 7 {
            if ctx.switches.test(self.m2b0 as u8) {
                ctx.flags.set(EVENT_UNK_3301);
            }
            let mut index = 0u32;
            let mut switch = self.m2a0 as u32;
            while index < self.m2a4 {
                if !ctx.switches.test(switch as u8) {
                    break;
                }
                index += 1;
                switch += 1;
            }
            if !ctx.flags.test(EVENT_UNK_3301)
                && index == self.m2a4
                && self.check_area_in(ctx, Some(player_pos))
            {
                result = self.check_timer();
            }
        } else if argument == 8 {
            if ctx.switches.test(self.m2b0 as u8) {
                ctx.flags.set(EVENT_UNK_3480);
            }
            if !ctx.flags.test(EVENT_UNK_3480) && self.check_area_in(ctx, Some(player_pos)) {
                result = self.check_timer();
            }
        } else if argument == 9 {
            if ctx.flags.test(EVENT_UNK_3408) && self.check_area_in(ctx, ctx.link_pos) {
                result = true;
            }
        } else if argument == 10 {
            if inventory::check_get_item(ctx.equipped, ctx.collect, item::HOOKSHOT) {
                let mut index = 0u32;
                let mut switch = self.m2a0 as u32;
                let mut found = false;
                while index < self.m2a4 {
                    if ctx.switches.test(switch as u8) {
                        found = true;
                        break;
                    }
                    index += 1;
                    switch += 1;
                }
                if found {
                    result = self.check_timer();
                }
            }
        } else if argument == 11 {
            if ctx.switches.test(self.m2b0 as u8) {
                ctx.flags.set(EVENT_UNK_3410);
            }
            if !ctx.flags.test(EVENT_UNK_3410)
                && ctx.flags.test(EVENT_UNK_3440)
                && self.check_area_in(ctx, ctx.link_pos)
            {
                result = true;
            }
        }
        result
    }

    pub fn execute(&mut self, ctx: &mut Ctx<'_>) {
        self.m2b8 = self.m2b9;
        self.m2b9 = 0;
        if self.argument <= 1 {
            ctx.actor.scale.x = ctx.hio.m04;
            ctx.actor.scale.y = ctx.hio.m00;
        }
        if !self.event_proc(ctx) {
            for (slot, name) in self.m2bc.iter_mut().zip(EVENT_NAME_TABLE.iter()) {
                *slot = event_index(ctx.manager, name) as i16;
            }
            if self.check_condition(ctx) {
                self.action(ctx);
            }
            self.event_order(ctx);
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NpcTalk {
    pub present: bool,
    pub is_md: bool,
    pub tag_check_ok: bool,
    pub carry_now: bool,
    pub no_carry_action: bool,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct TagMdCbPlayer {
    pub no_draw: bool,
}

pub struct Ctx<'a> {
    pub entity: Entity,
    pub actor: &'a mut Actor,
    pub player_entity: Option<Entity>,
    pub player_pos: Option<Vec3>,
    pub player_room: Option<i8>,
    pub link_pos: Option<Vec3>,
    pub flags: &'a mut EventFlags,
    pub switches: &'a Switches,
    pub collect: &'a Collect,
    pub equipped: &'a Equipped,
    pub events: &'a mut SimEvents,
    pub list: &'a EventList,
    pub active: &'a mut ActiveEvent,
    pub manager: &'a mut EventManager,
    pub vibration: &'a mut Vibration,
    pub hio: &'a mut TagMdCbHio,
    pub msg: &'a mut TagMdCbMsg,
    pub npc: NpcTalk,
    pub player: &'a mut TagMdCbPlayer,
    pub talk_command: bool,
    pub talk_partner: Option<Entity>,
    pub npc_wants_message_animation: Option<u8>,
    pub text: Text<'a>,
}

fn event_index(manager: &EventManager, name: &str) -> i32 {
    manager.get_event_idx(Some(name), EVENT_INFO_NONE)
}

fn message_set(ctx: &mut Ctx<'_>, msg_no: u32) {
    let archive = ctx.text.archive;
    let Some(archive) = archive else {
        return;
    };
    crate::text::msgproc::fop_msgm_message_set(
        ctx.text.procs,
        ctx.text.info,
        archive,
        0,
        msg_no,
        None,
        Vec3::ZERO,
    );
}

fn message_set_actor(ctx: &mut Ctx<'_>, msg_no: u32) -> u32 {
    let archive = ctx.text.archive;
    let Some(archive) = archive else {
        return FPC_M_ERROR_PROCESS_ID;
    };
    let pos = ctx.actor.pos;
    crate::text::msgproc::fop_msgm_message_set_actor(
        ctx.text.procs,
        ctx.text.info,
        archive,
        0,
        msg_no,
        None,
        pos,
    )
}

#[derive(SystemParam)]
pub struct World<'w, 's> {
    pub tags: Query<
        'w,
        's,
        (Entity, &'static mut Actor, &'static mut TagMdCb),
        Without<Player>,
    >,
    pub players: Query<'w, 's, (Entity, &'static Actor, &'static Player), Without<TagMdCb>>,
    pub npcs: Query<'w, 's, (Entity, Option<&'static Medli>, Option<&'static Makar>)>,
    pub list: Res<'w, EventList>,
    pub active: ResMut<'w, ActiveEvent>,
    pub manager: ResMut<'w, EventManager>,
    pub flags: ResMut<'w, EventFlags>,
    pub switches: Res<'w, Switches>,
    pub collect: Res<'w, Collect>,
    pub equipped: Res<'w, Equipped>,
    pub events: ResMut<'w, SimEvents>,
    pub vibration: ResMut<'w, Vibration>,
    pub carry: Res<'w, Carry>,
    pub do_intent: Res<'w, DoIntent>,
    pub hio: ResMut<'w, TagMdCbHio>,
    pub msg: ResMut<'w, TagMdCbMsg>,
    pub shared: ResMut<'w, TagMdCbGlobal>,
    pub player: ResMut<'w, TagMdCbPlayer>,
    pub procs: ResMut<'w, MsgProcs>,
    pub info: ResMut<'w, MsgGameInfo>,
    pub one: ResMut<'w, Msg1Box>,
    pub two: ResMut<'w, Msg2Box>,
    pub archive: Option<Res<'w, MsgArchives>>,
}

pub fn control(mut world: World<'_, '_>, mut commands: Commands<'_, '_>) {
    let archive = world.archive.as_deref().map(|held| held as &dyn MsgArchive);
    let carried = world.carry.held;
    let talk_command = world.do_intent.0;
    let mut npc = NpcTalk::default();
    for (entity, medli, makar) in &world.npcs {
        if medli.is_some() || makar.is_some() {
            npc.present = true;
            npc.is_md = medli.is_some();
            npc.carry_now = carried == Some(entity);
            if let Some(medli) = medli {
                npc.tag_check_ok = medli.is_tag_check_ok(npc.carry_now);
                npc.no_carry_action = medli.is_no_carry();
            } else if let Some(makar) = makar {
                npc.tag_check_ok = makar.is_tag_check_ok();
                npc.no_carry_action = makar.is_no_carry_action();
            }
            break;
        }
    }
    for (entity, mut actor, mut tag) in &mut world.tags {
        if tag.failed {
            continue;
        }
        let (player_entity, player_pos, player_room) = match world.players.iter().next() {
            Some((id, player_actor, _)) => (Some(id), Some(player_actor.pos), Some(player_actor.room)),
            None => (None, None, None),
        };
        let link_pos = player_pos;
        let text = Text {
            procs: &mut world.procs,
            info: &mut world.info,
            one: &mut world.one,
            two: &mut world.two,
            archive,
        };
        let mut ctx = Ctx {
            entity,
            actor: &mut *actor,
            player_entity,
            player_pos,
            player_room,
            link_pos,
            flags: &mut world.flags,
            switches: &world.switches,
            collect: &world.collect,
            equipped: &world.equipped,
            events: &mut world.events,
            list: &world.list,
            active: &mut world.active,
            manager: &mut world.manager,
            vibration: &mut world.vibration,
            hio: &mut world.hio,
            msg: &mut world.msg,
            npc,
            player: &mut world.player,
            talk_command,
            talk_partner: None,
            npc_wants_message_animation: None,
            text,
        };
        if !tag.entered {
            tag.argument = TagMdCb::argument_of(&ctx.actor.name);
            if !tag.create(&mut ctx) {
                drop(ctx);
                commands.entity(entity).despawn();
                continue;
            }
        }
        tag.execute(&mut ctx);
        let partner = ctx.talk_partner;
        let animation = ctx.npc_wants_message_animation;
        if let Some(entity) = partner {
            world.shared.talk_partner = Some(entity);
        }
        if let Some(value) = animation {
            world.shared.npc_message_animation = value;
        }
    }
}