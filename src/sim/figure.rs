use bevy_ecs::prelude::*;
use glam::{Quat, Vec2, Vec3};

use super::actor::Actor;
use super::event::{ActiveEvent, EventList, Stage};
use super::flags::{EventFlags, EventReg, StoryBits};
use super::msg::{
    MsgBox, MsgTable, MSG_BOX_CLOSED, MSG_CONTINUES, MSG_DESTROYED, MSG_DISPLAYED, MSG_ENDS,
};
use super::player::Player;
use super::walk;

pub const FIGURE_ACTOR: &str = "Figure";

pub const TOTAL_FIGURE_COUNT: u8 = 0x86;
pub const FIGURE_NONE: u8 = 0xFF;

pub const FIGURE_NO_WIDTH: u32 = 8;
pub const FIGURE_NO_SHIFT: u32 = 0;
pub const FIGURE_NO_MASK: u32 = (1u32 << FIGURE_NO_WIDTH) - 1;

pub const FIGURE_COMP: [u16; 17] = [
    0x95FF, 0x94FF, 0x93FF, 0x92FF, 0x91FF, 0x90FF, 0x8FFF, 0x8EFF, 0x8DFF, 0x8CFF, 0xB1FF, 0x9CFF,
    0x84FF, 0x83FF, 0x82FF, 0x81FF, 0x80FF,
];
pub const FIGURE_COMP_GROUPS: u8 = 0x11;

pub const FIGURE_FATHER_BIT: u16 = 0x2D01;
pub const FIGURE_MILA_FATHER: u8 = 0x10;
pub const FIGURE_MAGGIE_FATHER: u8 = 0x12;
pub const FIGURE_BRKPANEL: u8 = 0x3D;
pub const FIGURE_LINK: u8 = 0x40;
pub const FIGURE_UNLIT: u8 = 0x32;

pub const BMD_FIGURE1_VF_016B: u32 = 0x3;
pub const BMD_FIGURE1_VF_018B: u32 = 0x6;

pub const MODEL_ATTR_FULL: u32 = 0x3744_1422;
pub const MODEL_ATTR_ALT: u32 = 0x1100_1222;
pub const PEDESTAL_MODEL_ATTR: u32 = 0x1102_0022;
pub const PEDESTAL_MORF_ATTR: u32 = 0x1100_1222;

pub const FIGURE_DATA: [(u32, u32, i32); 134] = [
    (0x00, 0x37441422, -1),
    (0x01, 0x37441422, -1),
    (0x02, 0x37441422, -1),
    (0x03, 0x37441422, -1),
    (0x0A, 0x37441422, -1),
    (0x04, 0x37441422, -1),
    (0x05, 0x37441422, -1),
    (0x06, 0x37441422, -1),
    (0x07, 0x37441422, -1),
    (0x08, 0x37441422, -1),
    (0x09, 0x11001222, -1),
    (0x0B, 0x37441422, -1),
    (0x0C, 0x37441422, -1),
    (0x0D, 0x37441422, -1),
    (0x00, 0x37441422, -1),
    (0x01, 0x37441422, -1),
    (0x02, 0x37441422, -1),
    (0x04, 0x37441422, -1),
    (0x05, 0x37441422, -1),
    (0x07, 0x37441422, -1),
    (0x08, 0x37441422, -1),
    (0x09, 0x37441422, -1),
    (0x0A, 0x37441422, -1),
    (0x0B, 0x37441422, -1),
    (0x0C, 0x37441422, -1),
    (0x0D, 0x37441422, -1),
    (0x0E, 0x37441422, -1),
    (0x0F, 0x37441422, -1),
    (0x10, 0x37441422, -1),
    (0x11, 0x37441422, -1),
    (0x12, 0x37441422, -1),
    (0x13, 0x37441422, -1),
    (0x14, 0x37441422, -1),
    (0x15, 0x37441422, -1),
    (0x16, 0x37441422, -1),
    (0x17, 0x37441422, -1),
    (0x18, 0x37441422, -1),
    (0x19, 0x37441422, -1),
    (0x1A, 0x37441422, -1),
    (0x1B, 0x37441422, -1),
    (0x1C, 0x37441422, -1),
    (0x00, 0x37441422, 8),
    (0x01, 0x37441422, 8),
    (0x02, 0x37441422, 8),
    (0x03, 0x37441422, 8),
    (0x04, 0x37441422, 8),
    (0x05, 0x37441422, 8),
    (0x06, 0x37441422, 8),
    (0x07, 0x37441422, 8),
    (0x08, 0x37441422, 8),
    (0x09, 0x37441422, 8),
    (0x0A, 0x37441422, 8),
    (0x0B, 0x37441422, 8),
    (0x0C, 0x37441422, 8),
    (0x0D, 0x37441422, 8),
    (0x0E, 0x37441422, 8),
    (0x0F, 0x37441422, 8),
    (0x10, 0x37441422, 8),
    (0x11, 0x37441422, 8),
    (0x12, 0x37441422, 8),
    (0x13, 0x37441422, 8),
    (0x06, 0x37441422, 9),
    (0x01, 0x37441422, 9),
    (0x02, 0x37441422, 9),
    (0x05, 0x37441422, 9),
    (0x00, 0x11001222, -1),
    (0x01, 0x37441422, -1),
    (0x02, 0x37441422, -1),
    (0x03, 0x37441422, -1),
    (0x04, 0x37441422, -1),
    (0x05, 0x37441422, -1),
    (0x06, 0x37441422, -1),
    (0x07, 0x37441422, -1),
    (0x08, 0x37441422, -1),
    (0x09, 0x37441422, -1),
    (0x0A, 0x37441422, -1),
    (0x0B, 0x37441422, -1),
    (0x0C, 0x37441422, -1),
    (0x0D, 0x37441422, -1),
    (0x0E, 0x37441422, -1),
    (0x0F, 0x37441422, -1),
    (0x10, 0x37441422, -1),
    (0x11, 0x11001222, -1),
    (0x00, 0x37441422, -1),
    (0x05, 0x37441422, -1),
    (0x06, 0x37441422, -1),
    (0x07, 0x37441422, -1),
    (0x08, 0x37441422, -1),
    (0x09, 0x37441422, -1),
    (0x0A, 0x37441422, -1),
    (0x0B, 0x37441422, -1),
    (0x0C, 0x37441422, -1),
    (0x0D, 0x37441422, -1),
    (0x01, 0x37441422, -1),
    (0x02, 0x37441422, -1),
    (0x03, 0x37441422, -1),
    (0x04, 0x37441422, -1),
    (0x00, 0x37441422, -1),
    (0x01, 0x37441422, -1),
    (0x02, 0x37441422, -1),
    (0x03, 0x37441422, -1),
    (0x04, 0x37441422, -1),
    (0x05, 0x37441422, -1),
    (0x06, 0x37441422, -1),
    (0x07, 0x37441422, -1),
    (0x08, 0x37441422, -1),
    (0x09, 0x37441422, -1),
    (0x12, 0x37441422, -1),
    (0x0A, 0x11001222, -1),
    (0x0B, 0x11001222, -1),
    (0x0C, 0x11001222, -1),
    (0x0D, 0x37441422, -1),
    (0x0E, 0x37441422, -1),
    (0x0F, 0x37441422, -1),
    (0x10, 0x37441422, -1),
    (0x11, 0x37441422, -1),
    (0x00, 0x37441422, 0xA),
    (0x01, 0x37441422, 0xA),
    (0x02, 0x11001222, 0xA),
    (0x03, 0x11001222, 0xA),
    (0x04, 0x11001222, 0xA),
    (0x05, 0x37441422, 0xA),
    (0x06, 0x37441422, 0xA),
    (0x07, 0x37441422, 0xA),
    (0x00, 0x37441422, 0xB),
    (0x01, 0x11001222, 0xB),
    (0x02, 0x11001222, 0xB),
    (0x03, 0x37441422, 0xB),
    (0x04, 0x37441422, 0xB),
    (0x00, 0x11001222, 0xC),
    (0x01, 0x11001222, 0xC),
    (0x02, 0x11001222, 0xC),
    (0x03, 0x11001222, 0xC),
    (0x04, 0x11001222, 0xC),
];

pub const CHECK_TARGET_DIST: f32 = 200.0;
pub const CHECK_HEIGHT_STEP: f32 = 5.0;
pub const CHECK_HEIGHT_INIT: f32 = 118.0;
pub const CHECK_HEIGHT_MIN: f32 = 80.0;
pub const CHECK_HEIGHT_MAX: f32 = 150.0;
pub const CHECK_EYE_LIFT: f32 = 30.0;
pub const CHECK_FOVY_INIT: f32 = 2.0;
pub const CHECK_FOVY_STEP: f32 = 50.0;
pub const CHECK_FOVY_MIN: f32 = 20.0;
pub const CHECK_FOVY_MAX: f32 = 60.0;
pub const CHECK_SPIN_STEP: i16 = 0x0000;
pub const CHECK_TURN_UNITS: i16 = 0x0200;
pub const CHECK_TURN_STEP: f32 = 512.0;
pub const CHECK_AUTO_FRAMES: i16 = 0x0000;
pub const CHECK_OPEN_FRAMES: i16 = 10;

pub const CULL_MIN: [f32; 3] = [-50.0, 0.0, -50.0];
pub const CULL_MAX: [f32; 3] = [50.0, 200.0, 50.0];

pub const CYL_AT_TYPE_ALL: u32 = !0u32;
pub const CYL_TG_SPRM: u32 = 0x01 | 0x02;
pub const CYL_CO_SPRM: u32 = 0x01 | 0x08 | 0x70;
pub const CYL_GOBJ_TG_SPRM: u32 = 1 << 2;
pub const CYL_RADIUS: f32 = 60.0;
pub const CYL_HEIGHT: f32 = 150.0;
pub const CYL2_CO_SPRM: u32 = 0x129;
pub const CYL2_RADIUS: f32 = 200.0;
pub const CYL2_HEIGHT: f32 = 100.0;

pub const MODEL_LIFT: f32 = 100.0;
pub const ATTENTION_LIFT: f32 = 200.0;
pub const EYE_LIFT: f32 = 150.0;

pub const TALK_RANGE: f32 = 150.0;
pub const TALK_ANGLE: i32 = 13000;
pub const TALK_DISTANCE: u16 = 0xA8;
pub const SPEAK_DISTANCE: u16 = 0xA8;
pub const ATTN_LOCKON_TALK: u32 = 0x0000_0002;
pub const ATTN_ACTION_SPEAK: u32 = 0x0000_0008;
pub const ATTN_TALKFLAG_CHECK: u32 = 0x2000_0000;
pub const ATTN_FLAGS: u32 = ATTN_LOCKON_TALK | ATTN_ACTION_SPEAK | ATTN_TALKFLAG_CHECK;

pub const MSG_BASE: u32 = 0x37DD;
pub const MSG_STATUS_NONE: u8 = 0xFF;

pub const FIGURE_CHECK_EVENT: &str = "FIGURE_CHECK";
pub const FIGURE_EVENT_STAFF: &str = "Figure";
pub const CUT_NAMES: [&str; 3] = ["MES_SET", "ON_PLR", "OFF_PLR"];
pub const CUT_MES_SET: i8 = 0;
pub const CUT_ON_PLR: i8 = 1;
pub const CUT_OFF_PLR: i8 = 2;
pub const CUT_NONE: i8 = -1;

pub const TMP_PLAYER_HIDDEN: u32 = 0x0408;
pub const DO_STATUS_INFO: u8 = 0x21;
pub const A_STATUS_CANCEL: u8 = 0x27;

pub const NO_EVENT: i16 = -1;

pub const HEAP_PEDESTAL: u32 = 0xCD90;
pub const HEAP_LINK: u32 = 0x25000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proc {
    Normal,
}

const MOVE_PROC: [Proc; 1] = [Proc::Normal];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Move,
    Talk,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CutInput {
    pub action: i8,
    pub advance: bool,
    pub cut_proc: bool,
    pub axis: Vec2,
    pub c_stick_y: f32,
    pub trig_a: bool,
    pub trig_b: bool,
    pub msg_no: Option<i32>,
}

impl CutInput {
    pub fn idle() -> Self {
        Self {
            action: CUT_NONE,
            advance: false,
            cut_proc: false,
            axis: Vec2::ZERO,
            c_stick_y: 0.0,
            trig_a: false,
            trig_b: false,
            msg_no: None,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Figure {
    pub figure: u8,
    pub display: bool,
    pub proc: u8,
    pub step: Step,
    pub entered: bool,
    pub open_frames: i16,
    pub auto_spin: i16,
    pub cam_spin: i16,
    pub model_turn: i16,
    pub cam_height: f32,
    pub cam_fovy: f32,
    pub cam_center: Vec3,
    pub cam_eye: Vec3,
    pub msg_live: bool,
    pub msg_bound: bool,
    pub msg_no: u32,
    pub msg_seq: Option<u32>,
    pub event_idx: i16,
    pub cut: i8,
    pub attention: bool,
    pub can_talk: bool,
    pub player_hidden: bool,
    pub do_status: u8,
    pub a_status: u8,
}

impl Figure {
    pub fn spawn(parameters: u32) -> Self {
        let mut figure = ((parameters >> FIGURE_NO_SHIFT) & FIGURE_NO_MASK) as u8;
        if figure == FIGURE_NONE {
            figure = 0;
        }
        if figure >= TOTAL_FIGURE_COUNT {
            figure = 0;
        }
        Self {
            figure,
            display: false,
            proc: 0,
            step: Step::Move,
            entered: false,
            open_frames: 0,
            auto_spin: 0,
            cam_spin: 0,
            model_turn: 0,
            cam_height: 0.0,
            cam_fovy: 0.0,
            cam_center: Vec3::ZERO,
            cam_eye: Vec3::ZERO,
            msg_live: false,
            msg_bound: false,
            msg_no: 0,
            msg_seq: None,
            event_idx: NO_EVENT,
            cut: 0,
            attention: false,
            can_talk: false,
            player_hidden: false,
            do_status: 0,
            a_status: 0,
        }
    }

    pub fn ctor(&mut self, actor: &Actor, regs: &EventReg) {
        self.cam_spin = actor.angle[1];
        self.display = self.is_figure_get(regs);
        self.cam_spin = 0;
        self.model_turn = actor.angle[1];
        self.proc = 0;
    }

    pub fn create_init(&mut self, list: &EventList) {
        self.event_idx = list
            .event_index(FIGURE_CHECK_EVENT)
            .and_then(|index| i16::try_from(index).ok())
            .unwrap_or(NO_EVENT);
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.display || !self.unlit()
    }

    pub fn collides(&self) -> bool {
        self.drawn()
    }

    pub fn unlit(&self) -> bool {
        self.figure == FIGURE_LINK || self.figure == FIGURE_UNLIT
    }

    pub fn figure_data(&self) -> (u32, u32, i32) {
        FIGURE_DATA
            .get(usize::from(self.figure))
            .copied()
            .unwrap_or(FIGURE_DATA[0])
    }

    pub fn heap_size(&self) -> u32 {
        if self.figure == FIGURE_LINK {
            HEAP_LINK
        } else {
            HEAP_PEDESTAL
        }
    }

    pub fn brk_panel(&self) -> bool {
        self.figure == FIGURE_BRKPANEL
    }

    pub fn figure_bmd(&self, stories: &StoryBits) -> u32 {
        let mut bmd = self.figure_data().0;
        match self.figure {
            FIGURE_MILA_FATHER if stories.test(FIGURE_FATHER_BIT) => bmd = BMD_FIGURE1_VF_016B,
            FIGURE_MAGGIE_FATHER if stories.test(FIGURE_FATHER_BIT) => bmd = BMD_FIGURE1_VF_018B,
            _ => {}
        }
        bmd
    }

    pub fn is_figure_get(&self, regs: &EventReg) -> bool {
        let group = self.figure / 8;
        if group >= FIGURE_COMP_GROUPS {
            return false;
        }
        let bit = self.figure % 8;
        let reg = regs.reg(FIGURE_COMP.get(usize::from(group)).copied().unwrap_or(0));
        reg & (1 << bit) != 0
    }

    pub fn cantalk(&self, pos: Vec3, yaw: i16, player: &Player) -> bool {
        let dx = player.pos.x - pos.x;
        let dz = player.pos.z - pos.z;
        let temp1 = (dx * dx + dz * dz).sqrt();
        let temp2 = walk::to_s16(dx.atan2(dz)).wrapping_sub(yaw);
        TALK_RANGE > temp1 && TALK_ANGLE > i32::from(temp2.unsigned_abs())
    }

    pub fn attention_pos(&self, pos: Vec3) -> Vec3 {
        Vec3::new(pos.x, pos.y + ATTENTION_LIFT, pos.z)
    }

    pub fn eye_pos(&self, pos: Vec3) -> Vec3 {
        Vec3::new(pos.x, pos.y + EYE_LIFT, pos.z)
    }

    pub fn event_running(&self, active: &ActiveEvent) -> bool {
        self.event_idx >= 0 && active.stage_of(self.event_slot()) == Some(Stage::Play)
    }

    pub fn event_end_check(&self, active: &ActiveEvent) -> bool {
        self.event_idx >= 0 && active.end_check(self.event_slot())
    }

    pub fn event_reset(&self, active: &mut ActiveEvent) {
        if self.event_idx >= 0 {
            let _ = active.end(self.event_slot());
        }
    }

    pub fn event_slot(&self) -> usize {
        usize::from(self.event_idx.max(0) as u16)
    }

    pub fn event_move(
        &mut self,
        active: &mut ActiveEvent,
        input: CutInput,
        actor: &Actor,
        msg: &mut MsgBox,
        table: &MsgTable,
        flags: &mut EventFlags,
    ) -> bool {
        if self.event_end_check(active) {
            self.event_reset(active);
            return false;
        }
        let attn = self.attention;
        if input.cut_proc {
            if !self.attention {
                self.attention = attn;
            }
            return false;
        }
        self.private_cut(input, actor, msg, table, flags)
    }

    pub fn private_cut(
        &mut self,
        input: CutInput,
        actor: &Actor,
        msg: &mut MsgBox,
        table: &MsgTable,
        flags: &mut EventFlags,
    ) -> bool {
        self.cut = input.action;
        if self.cut == CUT_NONE {
            return true;
        }
        if input.advance {
            match self.cut {
                CUT_MES_SET => self.event_mes_set_init(input.msg_no),
                CUT_ON_PLR => self.event_on_plr_init(flags),
                CUT_OFF_PLR => self.event_off_plr_init(flags),
                _ => {}
            }
        }
        match self.cut {
            CUT_MES_SET => self.event_mes_set(actor, input, msg, table),
            _ => true,
        }
    }

    pub fn event_mes_set_init(&mut self, msg_no: Option<i32>) {
        if let Some(value) = msg_no {
            self.msg_seq = None;
            match value {
                0 => {
                    let msg = self.get_msg();
                    self.set_message(msg);
                }
                1 => {}
                _ => self.set_message(value as u32),
            }
            if let Some(seq) = self.msg_seq {
                self.set_message(seq);
            }
        } else {
            self.bump_seq();
            let seq = self.msg_seq.unwrap_or(0);
            self.set_message(seq);
        }
        self.cam_fovy = CHECK_FOVY_INIT;
        self.cam_height = CHECK_HEIGHT_INIT;
        self.auto_spin = 0;
        self.step = Step::Move;
        self.open_frames = CHECK_OPEN_FRAMES;
    }

    pub fn event_mes_set(
        &mut self,
        actor: &Actor,
        input: CutInput,
        msg: &mut MsgBox,
        table: &MsgTable,
    ) -> bool {
        match self.step {
            Step::Move => {
                if self.open_frames != 0 {
                    self.open_frames -= 1;
                } else {
                    self.force_status();
                    let stick_x = input.axis.x;
                    let stick_y = input.axis.y;
                    let sub_y = input.c_stick_y;
                    if stick_x != 0.0 || stick_y != 0.0 || sub_y != 0.0 {
                        self.auto_spin = CHECK_AUTO_FRAMES;
                        self.cam_spin =
                            self.cam_spin.wrapping_add((stick_x * CHECK_TURN_STEP) as i16);
                        self.cam_height += f32::from((stick_y * CHECK_HEIGHT_STEP) as i16);
                        if self.cam_height < CHECK_HEIGHT_MIN {
                            self.cam_height = CHECK_HEIGHT_MIN;
                        } else if self.cam_height > CHECK_HEIGHT_MAX {
                            self.cam_height = CHECK_HEIGHT_MAX;
                        }
                        self.cam_fovy -= sub_y * CHECK_FOVY_STEP;
                        if self.cam_fovy < CHECK_FOVY_MIN {
                            self.cam_fovy = CHECK_FOVY_MIN;
                        } else if self.cam_fovy > CHECK_FOVY_MAX {
                            self.cam_fovy = CHECK_FOVY_MAX;
                        }
                    } else if self.auto_spin != 0 {
                        self.auto_spin -= 1;
                    } else {
                        self.model_turn = self.model_turn.wrapping_add(CHECK_SPIN_STEP);
                    }
                    self.force_status();
                    if input.trig_b {
                        self.cam_spin = 0;
                        self.model_turn = actor.angle[1];
                        return true;
                    }
                    if input.trig_a {
                        self.step = Step::Talk;
                    }
                }
                self.update_camera(actor);
            }
            Step::Talk => {
                self.talk(0, msg, table);
                if self.msg_live {
                    self.step = Step::Close;
                }
            }
            Step::Close => {
                if self.talk(0, msg, table) == MSG_BOX_CLOSED {
                    self.step = Step::Move;
                }
            }
        }
        false
    }

    pub fn event_on_plr_init(&mut self, flags: &mut EventFlags) {
        self.player_hidden = true;
        flags.clear(TMP_PLAYER_HIDDEN);
    }

    pub fn event_off_plr_init(&mut self, flags: &mut EventFlags) {
        self.player_hidden = false;
        flags.set(TMP_PLAYER_HIDDEN);
    }

    pub fn talk(&mut self, param_1: i32, msg: &mut MsgBox, table: &MsgTable) -> u8 {
        let mut status = MSG_STATUS_NONE;
        if !self.msg_live {
            if param_1 == 1 {
                let msg_no = self.get_msg();
                self.msg_no = msg_no;
            }
            msg.message_set(self.msg_no, table);
            self.msg_live = msg.active();
            self.msg_bound = false;
        } else if self.msg_bound {
            status = msg.status();
            if status == MSG_DISPLAYED {
                let next = self.next_msg_status();
                msg.set_status(next);
                if next == MSG_CONTINUES {
                    msg.message_set(self.msg_no, table);
                }
            } else if status == MSG_BOX_CLOSED {
                msg.set_status(MSG_DESTROYED);
                self.msg_live = false;
            }
        } else {
            self.msg_bound = msg.active();
        }
        status
    }

    pub fn next_msg_status(&mut self) -> u8 {
        let mut status = MSG_CONTINUES;
        if self.msg_seq.is_some() {
            self.bump_seq();
            match self.msg_seq {
                Some(0) => {
                    self.msg_seq = None;
                    status = MSG_ENDS;
                }
                Some(value) => self.msg_no = value,
                None => {}
            }
        } else {
            status = MSG_ENDS;
        }
        status
    }

    pub fn get_msg(&mut self) -> u32 {
        self.msg_seq = None;
        MSG_BASE + u32::from(self.figure)
    }

    pub fn set_message(&mut self, msg_no: u32) {
        self.msg_live = false;
        self.msg_no = msg_no;
    }

    pub fn execute_normal(&mut self, pos: Vec3, yaw: i16, player: Option<&Player>) {
        if !self.display {
            return;
        }
        let talkable = player.map_or(false, |link| self.cantalk(pos, yaw, link));
        if talkable {
            self.can_talk = true;
        }
    }

    pub fn force_status(&mut self) {
        self.do_status = DO_STATUS_INFO;
        self.a_status = A_STATUS_CANCEL;
    }

    pub fn camera(&self) -> (Vec3, Vec3, f32, i16) {
        (self.cam_center, self.cam_eye, self.cam_fovy, 0)
    }

    fn update_camera(&mut self, actor: &Actor) {
        let temp = self.cam_height + CHECK_EYE_LIFT;
        self.cam_center = Vec3::new(0.0, temp, 0.0);
        let turn = Quat::from_rotation_y(walk::from_s16(actor.angle[1].wrapping_add(self.cam_spin)));
        self.cam_eye = turn * Vec3::new(0.0, temp, CHECK_TARGET_DIST) + actor.pos;
        self.cam_center += actor.pos;
    }

    fn bump_seq(&mut self) {
        self.msg_seq = Some(self.msg_seq.unwrap_or(0).wrapping_add(1));
    }
}

pub fn is_figure(name: &str) -> bool {
    name == FIGURE_ACTOR
}

pub fn control(
    list: Res<EventList>,
    active: Res<ActiveEvent>,
    regs: Res<EventReg>,
    players: Query<&Player>,
    mut figures: Query<(&Actor, &mut Figure), Without<Player>>,
) {
    let player = players.iter().next().copied();
    for (actor, figure) in &mut figures {
        if !figure.entered {
            figure.entered = true;
            figure.ctor(actor, &regs);
            figure.create_init(&list);
        }
        if figure.event_running(&active) {
            continue;
        }
        match MOVE_PROC
            .get(usize::from(figure.proc))
            .copied()
            .unwrap_or(MOVE_PROC[0])
        {
            Proc::Normal => figure.execute_normal(actor.pos, actor.angle[1], player.as_ref()),
        }
    }
}
