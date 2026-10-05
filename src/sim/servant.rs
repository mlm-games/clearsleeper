use bevy_ecs::prelude::*;
use glam::{Vec2, Vec3};

use super::actor::Actor;
use super::carry::Carry;
use super::collision::Collision;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{StoryBits, Switches};
use super::ground;
use super::intent::{CameraYaw, MoveIntent};
use super::player::Player;
use super::stream::add_calc_angle_s2;
use super::walk;
use crate::assets::MsgArchives;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgproc::fop_msgm_message_set;
use crate::text::msgproc::{MsgGameInfo, MsgProcs, DA_EVT_MODE_NONE, FPC_M_ERROR_PROCESS_ID};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_INPUT, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED,
    STATUS_MSG_DISPLAYED, STATUS_MSG_ENDS, STATUS_TYPING,
};

pub const SERVANT_ACTORS: [&str; 3] = ["Os", "Os1", "Os2"];

const SWITCH_SHIFT: u32 = 0x10;
const SWITCH_MASK: u32 = 0xFF;

const GOAL_ROOM: i8 = 7;

const MSG_CLEAR_COUNT: u8 = 0;
const MSG_DEFAULT: u32 = 0;
const MSG_GIVE_UP: u32 = 0xEF4;

const BIT_WAKEUP: [u16; 3] = [0x1780, 0x1740, 0x1720];
const BIT_FINISH: [u16; 3] = [0x1710, 0x1704, 0x1B01];
const BIT_MSG0: u16 = 0x2510;
const BIT_MSG1: u16 = 0x2608;
const BIT_MSG2: u16 = 0x2604;

const STAFF_NAMES: [&str; 3] = ["Os", "Os1", "Os2"];
const DAIZA_NAMES: [&str; 3] = ["Hdai1", "Hdai2", "Hdai3"];

const EVENT_NAMES: [&str; EVENT_SLOTS] = [
    "OPTION_CHAR_END",
    "Os_Wakeup",
    "Os_Finish",
    "Os1_Wakeup",
    "Os1_Finish",
    "Os2_Wakeup",
    "Os2_Finish",
    "Os_Message",
    "Os1_Message",
    "Os2_Message",
    "Os_Finit0",
    "Os_Finit1",
    "Os1_Finit0",
    "Os1_Finit1",
    "Os2_Finit0",
    "Os2_Finit1",
];

const RESTART_NO: [i8; 3] = [3, 4, 5];
const WAKEUP_ORDER: [i8; 3] = [1, 3, 5];
const FINISH_ORDER_FREE: [i8; 3] = [0xA, 0xC, 0xE];
const FINISH_ORDER_HELD: [i8; 3] = [0xB, 0xD, 0xF];
const MESSAGE_ORDER: [i8; 3] = [7, 8, 9];
const ORDER_TALK_LOW: i8 = 0x11;
const ORDER_TALK_HIGH: i8 = 0x12;
const ORDER_SPOKEN: i8 = 0xFF;
const ORDER_LIMIT: i8 = 0x10;

const FINISH_HOME_POS: [Vec3; 3] = [
    Vec3::new(683.0, 340.0, -8947.0),
    Vec3::new(-683.0, 340.0, -8947.0),
    Vec3::new(0.0, 340.0, -9630.0),
];
const FINISH_HOME_LIFT: f32 = 240.0;

const HEIGHT: f32 = 120.0;
const MAX_FALL: f32 = -100.0;

const HIO_M04: f32 = -20.0;
const HIO_MAX_ATTN_DIST_XZ: f32 = 200.0;
const HIO_MAX_ATTN_ANGLE_Y: i32 = 0x4000;
const HIO_ATTN_Y_OFFSET: f32 = 130.0;
const HIO_MAX_HEAD_X: i16 = 0;
const HIO_MAX_HEAD_Y: i16 = 0x1388;
const HIO_MIN_HEAD_X: i16 = 0;
const HIO_MIN_HEAD_Y: i16 = -0x1388;
const HIO_MAX_BACKBONE_X: i16 = 0;
const HIO_MAX_BACKBONE_Y: i16 = 0;
const HIO_MIN_BACKBONE_X: i16 = 0;
const HIO_MIN_BACKBONE_Y: i16 = 0;
const HIO_MAX_TURN_STEP: i16 = 0x1000;
const HIO_MAX_HEAD_TURN_VEL: i16 = 0x800;

const HIO_WALK_SPEED: f32 = 17.0;
const HIO_WALK_SCALE: i16 = 5;
const HIO_WALK_MAX: i16 = 3000;
const HIO_WALK_MIN: i16 = 1000;
const HIO_WALK_STICK: f32 = 1.0;

const HIO_CALL_RANGE: f32 = 80.0;
const HIO_GRAVITY: f32 = 4.0;

const HIO_THROW_SPEED: f32 = 15.0;
const HIO_THROW_LIFT: f32 = 22.0;

const HIO_CARRY_PITCH: f32 = -1.51367;
const HIO_CARRY_YAW: f32 = -1.09863;
const HIO_CARRY_ROLL: f32 = -0.56152;
const HIO_CHASE: f32 = 1.0;

const HIO_STICK_FACE: f32 = 0.1;
const HIO_STICK_WALK: f32 = 0.2;
const HIO_SEARCH_SPEED: f32 = 0.75;

const HIO_MORF_LO: f32 = 8.0;
const HIO_MORF_HI: f32 = 18.0;
const HIO_TENMETU_LO: f32 = 5.0;
const HIO_TENMETU_HI: f32 = 50.0;

const THROW_HAND_SPEED: f32 = 8.0;
const JUMP_SPEED: f32 = 50.0;
const GROUND_SNAP: f32 = 30.1;
const ROUTE_PROBE: f32 = 80.0;
const ROUTE_DROP: f32 = 80.0;
const ROUTE_LIMIT: f32 = 360000.0;
const ROUTE_JUMP_MIN: f32 = 0.0;
const ROUTE_JUMP_MAX: f32 = 80.0;
const ROUTE_JUMP_SCALE: f32 = 3.6;
const ROUTE_JUMP_NONE: f32 = -1.0;
const ROUTE_CROSS_MIN: f32 = 1.0;
const ROUTE_ANGLE: u16 = 0x4000;
const ROUTE_TURN: u16 = 0x800;
const ROUTE_FACE: u16 = 0x2000;
const ROUTE_FLIP: i16 = 0x8000;
const STOP_SPEED: f32 = 0.001;
const NO_GROUND_Y: f32 = -1.0e7;

const PLAY_SCALE: i16 = 0x19;
const PLAY_MAX: i16 = 0x7FFF;
const PLAY_MIN: i16 = 1;
const WALK_MIN_STICK: f32 = 0.05;

const ANM_TABLE: [(i8, f32, f32, i32); ANM_ROWS] = [
    (0, 8.0, 0.0, 0),
    (1, 8.0, 2.0, 0),
    (2, 0.0, 1.0, 0),
    (2, 0.0, -1.0, 0),
    (2, 0.0, 0.0, -1),
];

const BRK_TABLE: [(f32, i32); BRK_ROWS] = [
    (1.0, 0),
    (1.0, -1),
    (1.0, 0),
    (-1.0, 0),
    (1.0, 0),
    (0.0, 0),
    (1.0, 0),
    (0.0, -1),
    (1.0, 0),
];

const BRK_PAIR: [(i8, i8); BRK_ROWS] = [
    (0, 0),
    (0, 1),
    (2, 2),
    (3, 3),
    (0, 4),
    (0, 5),
    (0, 6),
    (4, 7),
];

const BRK_INIT: i8 = 9;
const BRK_TENMETU: i8 = 6;
const BRK_WAKEUP: i8 = 4;
const BRK_SETTLE: i8 = 6;
const BRK_LINK: i8 = 8;

const SE_LIGHT_START: u32 = 0x5900;
const SE_PUT: u32 = 0x58FE;
const SE_BLINK: u32 = 0x50FF;
const SE_LIGHT_SUSTAIN: u32 = 0x7046;

const STICK_REACH: f32 = 100.0;
const STICK_HALF: i16 = 0x8000;
const ATTN_DIST_BONUS: f32 = 40.0;
const ATTN_ANGLE_BONUS: i32 = 0x71C;

const OFF_COUNT_MAX: u8 = 30;
const NO_STEP: i8 = -1;
const STEP_INIT: i8 = 0;
const STEP_EXIT: i8 = -1;
const STEP_FIRST: i8 = 1;
const STEP_TALK_RUN: i8 = 2;

const RND_MIN: f32 = 8.0;
const RND_RANGE: f32 = 20.0;
const RND_MUL: i32 = 0xAB;
const RND_MOD: i32 = 0x763D;
const RND_SCALE: f32 = 30269.0;
const RND_SEED: i32 = 100;

const JOINT_SCALE: i16 = 4;
const JOINT_SCALE_I: i32 = 4;
const JOINT_MIN_ANGLE: i16 = 0x60;
const JOINT_MIN_STEP: i32 = 1;
const SPINE: usize = 1;
const HEAD: usize = 0;
const PITCH: usize = 0;
const YAW: usize = 1;

const POSITION_BIT: u32 = 0x10;
const EVENT_SLOTS: usize = 16;
const ANM_ROWS: usize = 5;
const BRK_ROWS: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    None,
    Wait,
    Finish01,
    Finish02,
    Talk,
    Carry,
    Throw,
    Jump,
    Search,
    WaitPlayer,
    WalkPlayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Param {
    Null,
    Marked,
    Angle(i16),
    Speed(f32),
}

pub struct Ctx<'a> {
    pub player: Option<&'a Player>,
    pub camera: &'a CameraYaw,
    pub collision: Option<&'a Collision>,
    pub pedestal: Option<&'a Actor>,
    pub list: &'a EventList,
    pub archive: Option<&'a MsgArchives>,
    pub active: &'a mut ActiveEvent,
    pub stories: &'a mut StoryBits,
    pub switches: &'a mut Switches,
    pub events: &'a mut SimEvents,
    pub procs: &'a mut MsgProcs,
    pub info: &'a mut MsgGameInfo,
    pub msg: &'a mut Msg1Box,
    pub stick: f32,
    pub stick_angle: i16,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Servant {
    pub argument: u8,
    pub switch_no: u8,
    pub state: Step,
    pub player_state: Step,
    pub act_step: i8,
    pub order: i8,
    pub accepted: i8,
    pub msg: u32,
    pub msg_id: u32,
    pub flags: u32,
    pub height: f32,
    pub anm: i32,
    pub anm_done: i8,
    pub brk: i8,
    pub anm_tbl: i8,
    pub attn_count: u8,
    pub attn_seen: u8,
    pub step_flip: i8,
    pub rnd_timer: u8,
    pub grounded: u8,
    pub call_command: bool,
    pub turn: i16,
    pub morph_frame: f32,
    pub brk_frame: f32,
    pub morph_speed: f32,
    pub brk_speed: f32,
    pub morph_pinned: bool,
    pub reached_end: bool,
    pub eye: Vec3,
    pub base: Vec3,
    pub offset: Vec3,
    pub normal: Vec3,
    pub event_id: [i16; EVENT_SLOTS],
    pub angles: [[i16; 2]; 2],
    pub max_angles: [[i16; 2]; 2],
    pub min_angles: [[i16; 2]; 2],
    pub max_step: [[i16; 2]; 2],
    pub turning: bool,
    pub beam: bool,
    pub player_room: bool,
    pub room: i8,
    pub pos: Vec3,
    pub old: Vec3,
    pub yaw: i16,
    pub shape_yaw: i16,
    pub speed: Vec3,
    pub speed_f: f32,
    pub gravity: f32,
    pub max_fall: f32,
    pub home: Vec3,
    pub home_yaw: i16,
    pub carried: bool,
    pub linked: bool,
    pub cb1_player: bool,
    pub off_count: u8,
    pub ground_h: f32,
    pub seed: i32,
    pub entered: bool,
}

fn bit(table: &[u16; 3], slot: usize) -> u16 {
    table.get(slot).copied().unwrap_or(0)
}

fn chase_to(value: f32, target: f32, step: f32) -> f32 {
    let mut held = value;
    let _ = walk::chase_f(&mut held, target, step);
    held
}

fn add_calc_angle_l(value: i32, target: i32, scale: i32, max_step: i32, min_step: i32) -> i32 {
    let diff = target - value;
    let mut value = value;
    if value != target {
        let mut step = diff / scale;
        if step > min_step || step < -min_step {
            step = step.min(max_step).max(-max_step);
            value += step;
        } else if diff >= 0 {
            value += min_step;
            if target - value <= 0 {
                value = target;
            }
        } else {
            value -= min_step;
            if target - value >= 0 {
                value = target;
            }
        }
    }
    value
}

impl Servant {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            argument: 0,
            switch_no: ((parameters >> SWITCH_SHIFT) & SWITCH_MASK) as u8,
            state: Step::None,
            player_state: Step::None,
            act_step: STEP_INIT,
            order: NO_STEP,
            accepted: NO_STEP,
            msg: MSG_DEFAULT,
            msg_id: FPC_M_ERROR_PROCESS_ID,
            flags: 0,
            height: HEIGHT,
            anm: 0,
            anm_done: 0,
            brk: BRK_INIT,
            anm_tbl: 3,
            attn_count: 0,
            attn_seen: 0,
            step_flip: 0,
            rnd_timer: 0,
            grounded: 0,
            call_command: false,
            turn: 0,
            morph_frame: 0.0,
            brk_frame: 0.0,
            morph_speed: 0.0,
            brk_speed: 0.0,
            morph_pinned: false,
            reached_end: false,
            eye: Vec3::ZERO,
            base: Vec3::ZERO,
            offset: Vec3::ZERO,
            normal: Vec3::ZERO,
            event_id: [NO_STEP; EVENT_SLOTS],
            angles: [[0, 0], [0, 0]],
            max_angles: [
                [HIO_MAX_HEAD_X, HIO_MAX_HEAD_Y],
                [HIO_MAX_BACKBONE_X, HIO_MAX_BACKBONE_Y],
            ],
            min_angles: [
                [HIO_MIN_HEAD_X, HIO_MIN_HEAD_Y],
                [HIO_MIN_BACKBONE_X, HIO_MIN_BACKBONE_Y],
            ],
            max_step: [[HIO_MAX_TURN_STEP; 2]; 2],
            turning: false,
            beam: false,
            player_room: false,
            room: 0,
            pos: Vec3::ZERO,
            old: Vec3::ZERO,
            yaw: 0,
            shape_yaw: 0,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            gravity: HIO_GRAVITY,
            max_fall: MAX_FALL,
            home: Vec3::ZERO,
            home_yaw: 0,
            carried: false,
            linked: false,
            cb1_player: false,
            off_count: 0,
            ground_h: NO_GROUND_Y,
            seed: RND_SEED,
            entered: false,
        }
    }

    pub fn slot(&self) -> usize {
        usize::from(self.argument)
    }

    pub fn argument_of(name: &str) -> u8 {
        match name {
            "Os1" => 1,
            "Os2" => 2,
            _ => 0,
        }
    }

    pub fn staff_name(&self) -> &'static str {
        STAFF_NAMES.get(self.slot()).copied().unwrap_or(STAFF_NAMES[0])
    }

    pub fn daiza_name(&self) -> &'static str {
        DAIZA_NAMES.get(self.slot()).copied().unwrap_or(DAIZA_NAMES[0])
    }

    pub fn head_x(&self) -> i16 {
        self.angles[HEAD][PITCH]
    }

    pub fn head_y(&self) -> i16 {
        self.angles[HEAD][YAW]
    }

    pub fn backbone_x(&self) -> i16 {
        self.angles[SPINE][PITCH]
    }

    pub fn backbone_y(&self) -> i16 {
        self.angles[SPINE][YAW]
    }

    pub fn wakeup_check(&self, stories: &StoryBits) -> bool {
        match self.argument {
            0 => stories.test(bit(&BIT_WAKEUP, 0)),
            1 => stories.test(bit(&BIT_WAKEUP, 1)),
            2 => stories.test(bit(&BIT_WAKEUP, 2)),
            _ => false,
        }
    }

    pub fn set_wakeup(&mut self, stories: &mut StoryBits) {
        let value = match self.argument {
            0 => bit(&BIT_WAKEUP, 0),
            1 => bit(&BIT_WAKEUP, 1),
            2 => bit(&BIT_WAKEUP, 2),
            _ => return,
        };
        stories.set(value);
    }

    pub fn finish_check(&self, stories: &StoryBits) -> bool {
        match self.argument {
            0 => stories.test(bit(&BIT_FINISH, 0)),
            1 => stories.test(bit(&BIT_FINISH, 1)),
            2 => stories.test(bit(&BIT_FINISH, 2)),
            _ => false,
        }
    }

    pub fn set_finish(&mut self, stories: &mut StoryBits) {
        let value = match self.argument {
            0 => bit(&BIT_FINISH, 0),
            1 => bit(&BIT_FINISH, 1),
            2 => bit(&BIT_FINISH, 2),
            _ => return,
        };
        stories.set(value);
    }

    pub fn wakeup_order_event_num(&self) -> i8 {
        match self.argument {
            0 => WAKEUP_ORDER[0],
            1 => WAKEUP_ORDER[1],
            2 => WAKEUP_ORDER[2],
            _ => NO_STEP,
        }
    }

    pub fn finish_order_event_num(&self) -> i8 {
        if self.carried {
            match self.argument {
                0 => FINISH_ORDER_HELD[0],
                1 => FINISH_ORDER_HELD[1],
                2 => FINISH_ORDER_HELD[2],
                _ => NO_STEP,
            }
        } else {
            match self.argument {
                0 => FINISH_ORDER_FREE[0],
                1 => FINISH_ORDER_FREE[1],
                2 => FINISH_ORDER_FREE[2],
                _ => NO_STEP,
            }
        }
    }

    pub fn restart_number(&self) -> i8 {
        RESTART_NO.get(self.slot()).copied().unwrap_or(0)
    }

    pub fn my_staff_id(&self, list: &EventList) -> Option<usize> {
        if self.argument < 3 {
            list.staff_of(self.staff_name())
        } else {
            None
        }
    }

    pub fn joint_check(&self, joint: i8) -> bool {
        match self.argument {
            0 => joint == 3,
            1 => joint == 2,
            2 => joint == 1,
            _ => false,
        }
    }

    pub fn is_finish(&self) -> bool {
        self.flags & 0x1 != 0
    }

    pub fn on_finish(&mut self) {
        self.flags |= 0x1;
    }

    pub fn is_set_home_pos(&self) -> bool {
        self.flags & 0x2 != 0
    }

    pub fn on_set_home_pos(&mut self) {
        self.flags |= 0x2;
    }

    pub fn is_water_hit(&self) -> bool {
        self.flags & 0x4 != 0
    }

    pub fn on_water_hit(&mut self) {
        self.flags |= 0x4;
    }

    pub fn is_gravity(&self) -> bool {
        self.flags & 0x8 != 0
    }

    pub fn on_gravity(&mut self) {
        self.flags |= 0x8;
    }

    pub fn off_gravity(&mut self) {
        self.flags &= !0x8;
    }

    pub fn clear_status(&mut self) {
        self.flags = 0;
    }

    pub fn inc_attn_set_count(&mut self) {
        if self.attn_count != 0xFF {
            self.attn_count = self.attn_count.wrapping_add(1);
        }
    }

    pub fn grounded(&self) -> bool {
        self.grounded != 0
    }

    pub fn set_grounded(&mut self, on: bool) {
        self.grounded = u8::from(on);
    }

    pub fn wall_hit(&self) -> bool {
        false
    }

    pub fn make_beam(&mut self, sound: bool, events: &mut SimEvents) {
        if !self.beam && sound {
            events.list.push(SimEvent::Se(SE_LIGHT_START));
        }
        self.beam = true;
    }

    pub fn end_beam(&mut self) {
        self.beam = false;
    }

    pub fn get_msg(&self) -> u32 {
        MSG_DEFAULT
    }

    pub fn next_msg_status(&mut self, stories: &mut StoryBits) -> u8 {
        if self.msg == MSG_DEFAULT {
            STATUS_MSG_ENDS
        } else if self.msg == MSG_GIVE_UP {
            stories.set(BIT_MSG0);
            STATUS_MSG_ENDS
        } else {
            STATUS_MSG_CONTINUES
        }
    }

    pub fn message_set(
        &self,
        procs: &mut MsgProcs,
        info: &mut MsgGameInfo,
        archive: Option<&MsgArchives>,
    ) -> u32 {
        let Some(archive) = archive else {
            return FPC_M_ERROR_PROCESS_ID;
        };
        fop_msgm_message_set(
            procs,
            info,
            archive,
            MSG_CLEAR_COUNT,
            self.msg,
            None,
            self.eye,
        )
    }

    pub fn talk_init(
        &mut self,
        procs: &mut MsgProcs,
        info: &mut MsgGameInfo,
        archive: Option<&MsgArchives>,
    ) -> bool {
        if self.msg_id == FPC_M_ERROR_PROCESS_ID {
            self.msg_id = self.message_set(procs, info, archive);
            false
        } else {
            procs.search_by_id(self.msg_id).is_some()
        }
    }

    pub fn talk(
        &mut self,
        msg: &mut Msg1Box,
        stories: &mut StoryBits,
        procs: &mut MsgProcs,
        info: &mut MsgGameInfo,
        archive: Option<&MsgArchives>,
    ) -> bool {
        if msg.status == STATUS_MSG_DISPLAYED {
            msg.status = self.next_msg_status(stories);
            if msg.status == STATUS_MSG_CONTINUES {
                self.message_set(procs, info, archive);
            }
        } else if msg.status != STATUS_INPUT
            && msg.status != STATUS_TYPING
            && msg.status == STATUS_BOX_CLOSED
        {
            msg.status = STATUS_MSG_DESTROYED;
            return true;
        }
        false
    }

    pub fn set_anm(&mut self, anm: i32) {
        self.anm = anm;
        let index = anm.clamp(0, ANM_ROWS as i32 - 1) as usize;
        let (tbl, frame, speed, at_end) = ANM_TABLE.get(index).copied().unwrap_or(ANM_TABLE[0]);
        if self.anm_tbl != tbl || self.morph_speed != speed {
            self.anm_tbl = tbl;
            self.reached_end = false;
            self.morph_pinned = at_end < 0;
            self.morph_frame = frame;
            self.morph_speed = speed;
        }
    }

    pub fn set_anm_brk_anm(&mut self, anm: i32) {
        let index = anm.clamp(0, BRK_ROWS as i32 - 1) as usize;
        let (base, brk) = BRK_PAIR.get(index).copied().unwrap_or(BRK_PAIR[0]);
        self.set_anm(i32::from(base));
        self.init_brk_anm(brk);
        self.anm_done = 0;
    }

    pub fn init_brk_anm(&mut self, brk: i8) -> bool {
        if self.brk == brk {
            return true;
        }
        let index = brk.clamp(0, BRK_ROWS as i32 - 1) as usize;
        let (speed, at_end) = BRK_TABLE.get(index).copied().unwrap_or(BRK_TABLE[0]);
        self.brk = brk;
        self.brk_speed = speed;
        if at_end >= 0 {
            self.brk_frame = 0.0;
        }
        true
    }

    pub fn play_brk_anm(&mut self, events: &mut SimEvents) {
        let previous = self.brk_frame;
        self.brk_frame += self.brk_speed;
        self.anm_done = 0;
        if self.brk_speed < 0.0 {
            if self.brk_frame > previous {
                self.anm_done = 1;
            }
        } else if self.brk_frame < previous {
            self.anm_done = 1;
        }
        if self.brk == BRK_TENMETU
            && self.brk_frame > HIO_TENMETU_LO
            && self.brk_frame < HIO_TENMETU_HI
        {
            events.list.push(SimEvent::Se(SE_BLINK));
        }
    }

    pub fn animation_play(&mut self, events: &mut SimEvents) {
        let previous = self.morph_frame;
        if !self.morph_pinned {
            self.morph_frame += self.morph_speed;
        }
        self.reached_end = false;
        if self.morph_speed < 0.0 {
            if self.morph_frame > previous {
                self.reached_end = true;
            }
        } else if self.morph_frame < previous {
            self.reached_end = true;
        }
        self.play_brk_anm(events);
    }

    pub fn walk_proc(&mut self, scale: f32, target: i16) {
        if self.morph_frame > HIO_MORF_LO && self.morph_frame < HIO_MORF_HI {
            self.speed_f = HIO_WALK_SPEED * scale;
        } else {
            self.speed_f = 0.0;
        }
        self.yaw = walk::add_calc_angle_s(
            self.yaw,
            target,
            HIO_WALK_SCALE,
            HIO_WALK_MAX,
            HIO_WALK_MIN,
        );
    }

    pub fn chase_shape_yaw(&mut self) {
        self.shape_yaw = walk::add_calc_angle_s(
            self.shape_yaw,
            self.yaw,
            HIO_WALK_SCALE,
            HIO_WALK_MAX * 2,
            HIO_WALK_MIN * 2,
        );
    }

    pub fn distance_sq(&self, player: &Player) -> f32 {
        let dx = player.pos.x - self.pos.x;
        let dz = player.pos.z - self.pos.z;
        dx * dx + dz * dz
    }

    pub fn chk_attention(&mut self, player: &Player, facing: i16) -> bool {
        let mut max_dist = HIO_MAX_ATTN_DIST_XZ;
        let mut max_angle = HIO_MAX_ATTN_ANGLE_Y;
        let dx = player.pos.x - self.pos.x;
        let dz = player.pos.z - self.pos.z;
        let dist = (dx * dx + dz * dz).sqrt();
        let angle = walk::to_s16(dx.atan2(dz)).wrapping_sub(facing);
        if self.attn_seen != 0 {
            max_dist += ATTN_DIST_BONUS;
            max_angle += ATTN_ANGLE_BONUS;
        }
        max_angle > i32::from(angle.unsigned_abs()) && max_dist > dist
    }

    pub fn facing_angle(&self) -> i16 {
        self.shape_yaw
            .wrapping_add(self.head_y())
            .wrapping_add(self.backbone_y())
    }

    pub fn chk_area(&self, player: &Player) -> bool {
        let dx = player.pos.x - self.pos.x;
        let dz = player.pos.z - self.pos.z;
        let dist = (dx * dx + dz * dz).sqrt();
        HIO_MAX_ATTN_DIST_XZ + ATTN_DIST_BONUS > dist
    }

    pub fn player_eye_y(&self, player: &Player) -> f32 {
        player.pos.y + HIO_M04
    }

    pub fn set_attention(&mut self, active: bool) {
        if !active && self.attn_count >= 2 {
            return;
        }
        self.eye = self.base;
    }

    pub fn attention_point(&self) -> Vec3 {
        Vec3::new(self.pos.x, self.pos.y + HIO_ATTN_Y_OFFSET, self.pos.z)
    }

    pub fn get_stick_ang_y(&self, camera: &CameraYaw, stick_angle: i16) -> i16 {
        walk::to_s16(camera.0)
            .wrapping_add(stick_angle)
            .wrapping_add(STICK_HALF)
    }

    pub fn calc_stick_pos(&self, mut angle: i16, target: Option<&Player>) -> (i32, Vec3) {
        let lockon = self.attn_seen != 0;
        let ret: i32 = if !lockon {
            0
        } else if self.attn_count != 0 {
            1
        } else {
            -1
        };
        if let Some(player) = target {
            let mut pos = player.pos;
            pos.y = self.player_eye_y(player);
            return (ret, pos);
        }
        if lockon {
            angle = self.shape_yaw;
        }
        let (sin, cos) = walk::from_s16(angle).sin_cos();
        (
            ret,
            Vec3::new(
                self.pos.x + sin * STICK_REACH,
                self.pos.y,
                self.pos.z + cos * STICK_REACH,
            ),
        )
    }

    pub fn set_npc_action(&mut self, action: Step, param: Param, stories: &StoryBits) {
        self.player_state = Step::None;
        self.swap(true, action, param, stories);
    }

    pub fn set_player_action(&mut self, action: Step, stories: &StoryBits) {
        self.state = Step::None;
        self.swap(false, action, Param::Null, stories);
    }

    fn swap(&mut self, npc: bool, action: Step, param: Param, stories: &StoryBits) {
        let current = if npc { self.state } else { self.player_state };
        if current == action {
            return;
        }
        if current != Step::None {
            self.act_step = STEP_EXIT;
            self.exit_action(current);
        }
        if npc {
            self.state = action;
        } else {
            self.player_state = action;
        }
        self.act_step = STEP_INIT;
        self.enter_action(action, param, stories);
    }

    fn exit_action(&mut self, action: Step) {
        if action == Step::Carry {
            self.offset.y = 0.0;
            self.offset.z = 0.0;
        }
    }

    fn enter_action(&mut self, action: Step, param: Param, stories: &StoryBits) {
        match action {
            Step::Wait => {
                self.max_fall = MAX_FALL;
                self.gravity = HIO_GRAVITY;
                self.speed_f = 0.0;
                self.height = HEIGHT;
            }
            Step::Finish01 => {
                self.linked = false;
                self.set_anm(0);
                self.init_brk_anm(0);
                self.anm_done = 0;
                self.max_fall = MAX_FALL;
                self.gravity = HIO_GRAVITY;
                self.speed_f = 0.0;
                self.height = HEIGHT;
            }
            Step::Finish02 => {
                if !matches!(param, Param::Marked) {
                    let anm = if stories.test(bit(&BIT_FINISH, 2)) {
                        7
                    } else {
                        1
                    };
                    self.set_anm_brk_anm(anm);
                }
                self.on_finish();
                self.max_fall = MAX_FALL;
                self.gravity = HIO_GRAVITY;
                self.speed_f = 0.0;
                self.height = HEIGHT;
            }
            Step::Talk => {
                self.msg_id = FPC_M_ERROR_PROCESS_ID;
                self.msg = self.get_msg();
                self.attn_count = 0;
            }
            Step::Carry => {
                self.set_anm(0);
                self.speed_f = 0.0;
                self.height = HEIGHT;
            }
            Step::Throw => match param {
                Param::Angle(angle) => {
                    self.speed_f = THROW_HAND_SPEED;
                    self.speed.y = 0.0;
                    self.yaw = angle;
                }
                _ => {
                    self.speed_f = HIO_THROW_SPEED;
                    self.speed.y = HIO_THROW_LIFT;
                    self.yaw = self.shape_yaw;
                }
            },
            Step::Jump => {
                self.set_anm(0);
                if let Param::Speed(speed) = param {
                    self.speed.y = speed;
                }
                self.speed_f = JUMP_SPEED;
                self.max_fall = MAX_FALL;
                self.gravity = HIO_GRAVITY;
            }
            Step::Search => {
                self.set_anm(1);
            }
            Step::WaitPlayer => {
                self.speed_f = 0.0;
                self.set_anm(0);
                self.init_brk_anm(BRK_LINK);
            }
            Step::WalkPlayer => {
                self.speed_f = 0.0;
                self.set_anm(1);
                self.init_brk_anm(BRK_LINK);
            }
            Step::None => {}
        }
        self.act_step += 1;
    }

    pub fn route_ang_check(
        &mut self,
        plane: Vec3,
        angle: &mut i16,
        player: Option<&Player>,
    ) -> f32 {
        let cross = self.normal.cross(plane);
        let mut next = walk::to_s16(cross.x.atan2(cross.z));
        let far = next.wrapping_sub(*angle).unsigned_abs() > ROUTE_ANGLE;
        let height = player.map_or(0.0, |player| player.pos.y - self.pos.y);
        if (self.normal.y >= ROUTE_CROSS_MIN && far) || height * cross.y < 0.0 {
            next = next.wrapping_add(ROUTE_FLIP);
        }
        *angle = next;
        height
    }

    pub fn check_forward_ground_y(&self) -> f32 {
        NO_GROUND_Y
    }

    pub fn check_wall_jump(&self) -> f32 {
        let mut delta = self.check_forward_ground_y() - self.pos.y;
        if ROUTE_JUMP_MIN < delta && delta < ROUTE_JUMP_MAX {
            delta = delta.sqrt() * ROUTE_JUMP_SCALE;
        } else {
            delta = ROUTE_JUMP_NONE;
        }
        delta
    }

    pub fn route_check(
        &mut self,
        distance_sq: f32,
        angle: &mut i16,
        collision: Option<&Collision>,
        stories: &StoryBits,
    ) -> bool {
        if !self.grounded() {
            self.pos = self.old;
            self.speed_f = 0.0;
            self.grounded = 1;
            let probe = Vec3::new(
                self.pos.x + walk::from_s16(*angle).sin() * ROUTE_PROBE,
                self.pos.y + ROUTE_PROBE,
                self.pos.z + walk::from_s16(*angle).cos() * ROUTE_PROBE,
            );
            if ground::surface(collision, probe).unwrap_or(NO_GROUND_Y) - self.pos.y > -ROUTE_DROP {
                if self.yaw.wrapping_sub(*angle).unsigned_abs() < ROUTE_TURN {
                    self.set_npc_action(Step::Jump, Param::Null, stories);
                }
                return true;
            }
            if distance_sq > ROUTE_LIMIT {
                return false;
            }
        } else {
            if self.wall_hit() {
                let jump = self.check_wall_jump();
                if jump >= 0.0 {
                    self.set_npc_action(Step::Jump, Param::Speed(jump), stories);
                    return true;
                }
                if distance_sq > ROUTE_LIMIT {
                    return false;
                }
            }
        }
        true
    }

    pub fn settle(&mut self, collision: Option<&Collision>) {
        self.ground_h = ground::surface(collision, self.pos).unwrap_or(NO_GROUND_Y);
        self.set_grounded(self.ground_h != NO_GROUND_Y && self.pos.y <= self.ground_h);
        self.normal = ground::slope_normal(collision, self.pos);
    }

    pub fn move_step(&mut self) {
        if self.is_gravity() {
            if self.max_fall < self.speed.y {
                self.speed.y -= self.gravity;
                if self.speed.y < self.max_fall {
                    self.speed.y = self.max_fall;
                }
            } else if self.max_fall > self.speed.y {
                self.speed.y += self.gravity;
                if self.speed.y > self.max_fall {
                    self.speed.y = self.max_fall;
                }
            }
        }
        let (sin, cos) = walk::from_s16(self.yaw).sin_cos();
        self.speed.x = self.speed_f * sin;
        self.speed.z = self.speed_f * cos;
        self.pos += self.speed;
    }

    pub fn snap_to_ground(&mut self) {
        if !self.grounded() {
            let delta = self.ground_h - self.pos.y;
            if delta < 0.0 && delta >= -GROUND_SNAP {
                self.pos.y = self.ground_h;
                self.set_grounded(true);
                self.speed.y = 0.0;
            }
        }
    }

    pub fn rnd(&mut self) -> f32 {
        self.seed = (self.seed * RND_MUL) % RND_MOD;
        ((self.seed) as f32 / RND_SCALE) % 1.0
    }

    pub fn chase_carry(&mut self) {
        self.offset.x = chase_to(self.offset.x, HIO_CARRY_PITCH, HIO_CHASE);
        self.offset.y = chase_to(self.offset.y, HIO_CARRY_YAW, HIO_CHASE);
        self.offset.z = chase_to(self.offset.z, HIO_CARRY_ROLL, HIO_CHASE);
    }

    pub fn return_link_player(&mut self) {
        self.linked = true;
        self.init_brk_anm(BRK_SETTLE);
    }

    pub fn check_goal_room(&mut self, stories: &StoryBits) -> bool {
        if self.room == GOAL_ROOM && self.wakeup_check(stories) && !self.finish_check(stories) {
            self.order = self.finish_order_event_num();
            true
        } else {
            false
        }
    }

    pub fn event_order_check(&mut self, stories: &StoryBits, switches: &Switches) {
        if self.order != NO_STEP || self.room != GOAL_ROOM {
            return;
        }
        if switches.test(self.switch_no) {
            return;
        }
        let already = match self.argument {
            0 => stories.test(BIT_MSG0),
            1 => stories.test(BIT_MSG1),
            2 => stories.test(BIT_MSG2),
            _ => return,
        };
        if !already {
            self.order = MESSAGE_ORDER
                .get(self.slot())
                .copied()
                .unwrap_or(NO_STEP);
        }
    }

    pub fn check_order(&mut self, stories: &StoryBits) {
        if self.order == ORDER_TALK_HIGH || self.order == ORDER_TALK_LOW {
            self.order = ORDER_SPOKEN;
            self.set_npc_action(Step::Talk, Param::Null, stories);
            self.carried = false;
        }
    }

    pub fn event_order(&mut self, list: &EventList, active: &mut ActiveEvent, entity: Entity) {
        if self.accepted != NO_STEP {
            return;
        }
        if self.order == ORDER_TALK_HIGH || self.order == ORDER_TALK_LOW {
            return;
        }
        if self.order == NO_STEP || i32::from(self.order) >= i32::from(ORDER_LIMIT) {
            return;
        }
        self.accepted = self.order;
        let slot = self.order.clamp(0, (EVENT_SLOTS - 1) as i8) as usize;
        let Some(index) = self
            .event_id
            .get(slot)
            .copied()
            .filter(|value| *value >= 0)
            .map(|value| usize::from(value as u16))
        else {
            return;
        };
        active.order(list, index, entity);
    }

    pub fn init(&mut self, list: &EventList, stories: &StoryBits, pos: Vec3, yaw: i16, room: i8) {
        self.room = room;
        self.pos = pos;
        self.old = pos;
        self.home = pos;
        self.home_yaw = yaw;
        self.yaw = yaw;
        self.shape_yaw = yaw;
        self.eye = pos;
        self.base = pos;
        self.clear_status();
        self.gravity = HIO_GRAVITY;
        self.anm_tbl = 3;
        self.anm = 5;
        if self.wakeup_check(stories) {
            self.on_gravity();
        } else {
            self.off_gravity();
        }
        self.init_brk_anm(self.opening_brk(stories));
        if self.finish_check(stories) {
            self.set_npc_action(Step::Finish02, Param::Null, stories);
        } else {
            if self.argument < 3 {
                self.home = FINISH_HOME_POS
                    .get(self.slot())
                    .copied()
                    .unwrap_or(FINISH_HOME_POS[0]);
                self.pos = self.home;
            }
            self.set_anm(0);
            self.set_npc_action(Step::Wait, Param::Null, stories);
        }
        self.accepted = NO_STEP;
        self.order = NO_STEP;
        self.height = HEIGHT;
        self.eye = self.pos;
        self.base = self.pos;
        for (slot, name) in EVENT_NAMES.iter().enumerate() {
            self.event_id[slot] = list
                .event_index(name)
                .and_then(|index| i16::try_from(index).ok())
                .unwrap_or(NO_STEP);
        }
    }

    fn opening_brk(&self, stories: &StoryBits) -> i8 {
        if self.wakeup_check(stories) {
            if self.finish_check(stories) {
                1
            } else {
                6
            }
        } else {
            5
        }
    }

    fn wait_npc_action(&mut self, ctx: &mut Ctx) {
        if self.anm != 0 && self.reached_end {
            self.set_anm(0);
        }
        if self.act_step == STEP_FIRST {
            if self.wakeup_check(ctx.stories) && self.linked {
                self.init_brk_anm(BRK_WAKEUP);
                self.anm_done = 0;
                self.act_step += 1;
            }
        } else if let Some(player) = ctx.player {
            let angle = self.facing_angle();
            self.attn_seen = u8::from(self.chk_attention(player, angle));
            if self.chk_area(player) {
                self.look_back(self.attn_seen != 0, 0, 1, Some(player));
            }
            self.yaw = self.shape_yaw;
            if self.finish_check(ctx.stories) {
                self.set_npc_action(Step::Finish01, Param::Null, ctx.stories);
            }
        }
        self.turning = false;
        let Some(player) = ctx.player else {
            self.set_attention(true);
            return;
        };
        let dist_sq = self.distance_sq(player);
        if self.call_command {
            if self.wakeup_check(ctx.stories) {
                if dist_sq >= HIO_CALL_RANGE * HIO_CALL_RANGE {
                    self.set_npc_action(Step::Search, Param::Null, ctx.stories);
                }
            } else {
                self.order = self.wakeup_order_event_num();
            }
        }
        self.set_attention(true);
    }

    fn finish01_npc_action(&mut self, ctx: &mut Ctx) {
        let default_y = self.shape_yaw;
        self.look_at_target(true, 0, None, Vec3::ZERO, default_y, self.turn, true);
        if self.anm_done != 0 {
            self.set_npc_action(Step::Finish02, Param::Null, ctx.stories);
        }
        self.set_attention(true);
    }

    fn finish02_npc_action(&mut self) {
        let default_y = self.shape_yaw;
        self.look_at_target(true, 0, None, Vec3::ZERO, default_y, self.turn, true);
    }

    fn talk_npc_action(&mut self, ctx: &mut Ctx) {
        if let Some(player) = ctx.player {
            let angle = self.facing_angle();
            self.attn_seen = u8::from(self.chk_attention(player, angle));
        }
        let archive = ctx.archive;
        match self.act_step {
            STEP_FIRST => {
                if self.talk_init(ctx.procs, ctx.info, archive) {
                    self.act_step = STEP_TALK_RUN;
                }
            }
            STEP_TALK_RUN => {
                let done = self.talk(ctx.msg, ctx.stories, ctx.procs, ctx.info, archive);
                if done {
                    self.set_npc_action(Step::Wait, Param::Null, ctx.stories);
                    ctx.info.event_mode = DA_EVT_MODE_NONE;
                }
            }
            _ => {
                self.set_npc_action(Step::Wait, Param::Null, ctx.stories);
            }
        }
        self.turning = true;
        self.look_back(self.attn_seen != 0, 0, 0, ctx.player);
        self.yaw = self.shape_yaw;
        self.set_attention(true);
    }

    fn carry_npc_action(&mut self, ctx: &mut Ctx) {
        if self.act_step == STEP_EXIT {
            self.offset.y = 0.0;
            self.offset.z = 0.0;
            return;
        }
        self.chase_carry();
        if self.wall_hit() {
            self.set_npc_action(Step::Throw, Param::Angle(self.shape_yaw), ctx.stories);
            return;
        }
        if !self.carried {
            if self.speed_f > 0.0 {
                self.set_npc_action(Step::Throw, Param::Null, ctx.stories);
            } else {
                ctx.events.list.push(SimEvent::Se(SE_PUT));
                self.set_npc_action(Step::Wait, Param::Null, ctx.stories);
            }
            return;
        }
        let default_y = self.shape_yaw;
        self.look_at_target(true, 0, None, Vec3::ZERO, default_y, 0, false);
        self.yaw = self.shape_yaw;
    }

    fn throw_npc_action(&mut self, ctx: &mut Ctx) {
        if self.grounded() {
            ctx.events.list.push(SimEvent::Se(SE_PUT));
            self.set_npc_action(Step::Wait, Param::Null, ctx.stories);
        }
        self.set_attention(true);
    }

    fn jump_npc_action(&mut self, ctx: &mut Ctx) {
        if self.grounded() {
            self.set_npc_action(Step::Wait, Param::Null, ctx.stories);
        }
        self.set_attention(true);
    }

    fn search_npc_action(&mut self, ctx: &mut Ctx) {
        self.attn_seen = 1;
        let Some(player) = ctx.player else {
            return;
        };
        let dist_sq = self.distance_sq(player);
        let mut scale = if dist_sq < HIO_CALL_RANGE * HIO_CALL_RANGE {
            0.0
        } else {
            HIO_SEARCH_SPEED
        };
        let mut angle = walk::to_s16((player.pos.x - self.pos.x).atan2(player.pos.z - self.pos.z));
        let facing = angle;
        let routed = self.route_check(dist_sq, &mut angle, ctx.collision, ctx.stories);
        let aligned = facing.wrapping_sub(angle).unsigned_abs() <= ROUTE_FACE;
        if !routed || !aligned {
            scale = 0.0;
        } else {
            if self.rnd_timer == 0 {
                self.step_flip ^= 1;
                self.rnd_timer = (RND_MIN + self.rnd() * RND_RANGE) as u8;
            } else {
                self.rnd_timer = self.rnd_timer.saturating_sub(1);
            }
            self.morph_speed = 2.0;
        }
        self.walk_proc(scale, angle);
        self.chase_shape_yaw();
        let saved = self.shape_yaw;
        self.look_back(true, 0, 0, Some(player));
        if scale < STOP_SPEED {
            self.set_npc_action(Step::Wait, Param::Null, ctx.stories);
        } else {
            self.shape_yaw = saved;
        }
        self.set_attention(true);
    }

    fn wait_player_action(&mut self, ctx: &mut Ctx) {
        let Some(player) = ctx.player else {
            return;
        };
        let stick = ctx.stick;
        if stick >= HIO_STICK_FACE || self.attn_seen != 0 {
            let target = self.get_stick_ang_y(ctx.camera, ctx.stick_angle);
            self.yaw = walk::add_calc_angle_s(self.yaw, target, PLAY_SCALE, PLAY_MAX, PLAY_MIN);
            let (stick_pos, aim) = self.calc_stick_pos(target, Some(player));
            if stick_pos == 0 || stick >= HIO_STICK_WALK {
                self.shape_yaw = self.yaw;
            }
            let local = self.shape_yaw;
            let out = self.look_at_target(
                false,
                local,
                Some(aim),
                self.pos,
                self.shape_yaw,
                HIO_MAX_TURN_STEP,
                false,
            );
            if stick_pos > 0 {
                self.shape_yaw = out;
            }
            self.yaw = self.shape_yaw;
            if stick >= HIO_STICK_WALK && stick_pos == 0 {
                self.yaw = target;
                self.set_player_action(Step::WalkPlayer, ctx.stories);
            }
        } else {
            self.look_at_target(true, 0, None, Vec3::ZERO, self.shape_yaw, 0, false);
            self.yaw = self.shape_yaw;
        }
        self.set_attention(true);
    }

    fn walk_player_action(&mut self, ctx: &mut Ctx) {
        let Some(player) = ctx.player else {
            return;
        };
        let stick = ctx.stick;
        let target = self.get_stick_ang_y(ctx.camera, ctx.stick_angle);
        if stick > WALK_MIN_STICK {
            if self.morph_frame > HIO_MORF_LO && self.morph_frame < HIO_MORF_HI {
                self.speed_f = HIO_WALK_SPEED * stick * HIO_WALK_STICK;
            } else {
                self.speed_f = 0.0;
                self.yaw = walk::add_calc_angle_s(
                    self.yaw,
                    target,
                    HIO_WALK_SCALE,
                    HIO_WALK_MAX,
                    HIO_WALK_MIN,
                );
            }
            let (stick_pos, aim) = self.calc_stick_pos(target, Some(player));
            if stick_pos == 0 {
                self.chase_shape_yaw();
            }
            let local = self.shape_yaw;
            let out = self.look_at_target(
                false,
                local,
                Some(aim),
                self.pos,
                self.shape_yaw,
                HIO_MAX_TURN_STEP,
                false,
            );
            if stick_pos > 0 {
                self.shape_yaw = out;
            }
            if stick_pos != 0 {
                self.set_player_action(Step::WaitPlayer, ctx.stories);
            }
        } else {
            self.set_player_action(Step::WaitPlayer, ctx.stories);
        }
        self.set_attention(true);
    }

    pub fn npc_action(&mut self, ctx: &mut Ctx) {
        if self.state == Step::None {
            self.speed_f = 0.0;
            if self.brk == BRK_LINK {
                self.init_brk_anm(BRK_SETTLE);
            }
            self.set_npc_action(Step::Wait, Param::Null, ctx.stories);
        }
        match self.state {
            Step::Wait => self.wait_npc_action(ctx),
            Step::Finish01 => self.finish01_npc_action(ctx),
            Step::Finish02 => self.finish02_npc_action(),
            Step::Talk => self.talk_npc_action(ctx),
            Step::Carry => self.carry_npc_action(ctx),
            Step::Throw => self.throw_npc_action(ctx),
            Step::Jump => self.jump_npc_action(ctx),
            Step::Search => self.search_npc_action(ctx),
            _ => {}
        }
    }

    pub fn player_action(&mut self, ctx: &mut Ctx) {
        if self.player_state == Step::None {
            self.speed_f = 0.0;
            self.set_player_action(Step::WaitPlayer, ctx.stories);
        }
        match self.player_state {
            Step::WaitPlayer => self.wait_player_action(ctx),
            Step::WalkPlayer => self.walk_player_action(ctx),
            _ => {}
        }
    }

    pub fn carry_check(&mut self, stories: &StoryBits) {
        if self.carried {
            self.set_npc_action(Step::Carry, Param::Null, stories);
        }
    }

    pub fn initial_end_event(&mut self, stories: &mut StoryBits) {
        self.set_finish(stories);
        self.set_npc_action(Step::Finish02, Param::Marked, stories);
    }

    pub fn event_proc(&mut self, ctx: &mut Ctx) -> bool {
        let _ = self.my_staff_id(ctx.list);
        if !ctx.info.event_run_check() || self.accepted == NO_STEP {
            return false;
        }
        let slot = self.accepted.clamp(0, (EVENT_SLOTS - 1) as i8) as usize;
        let Some(index) = self
            .event_id
            .get(slot)
            .copied()
            .filter(|value| *value >= 0)
            .map(|value| usize::from(value as u16))
        else {
            self.accepted = NO_STEP;
            return true;
        };
        let index = usize::from(index as u16);
        if ctx.active.end_check(index) {
            ctx.info.event_mode = DA_EVT_MODE_NONE;
            if slot == 0 {
                self.return_link_player();
            }
            self.accepted = NO_STEP;
        }
        true
    }

    pub fn look_back(&mut self, watch: bool, facing: i32, follow: i32, player: Option<&Player>) {
        if self.turning {
            add_calc_angle_s2(
                &mut self.turn,
                HIO_MAX_HEAD_TURN_VEL,
                JOINT_SCALE,
                HIO_MAX_HEAD_TURN_VEL,
            );
        } else {
            self.turn = 0;
        }
        let mut aim = None;
        let mut from = Vec3::ZERO;
        if watch {
            if let Some(player) = player {
                let mut point = player.pos;
                point.y = self.player_eye_y(player);
                aim = Some(point);
            }
            from = self.pos;
            from.y = self.eye.y;
        }
        self.look_at_target(
            true,
            0,
            aim,
            from,
            self.shape_yaw,
            self.turn,
            follow != 0,
        );
        let _ = facing;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn look_at_target(
        &mut self,
        shape: bool,
        local: i16,
        dst: Option<Vec3>,
        src: Vec3,
        default_y: i16,
        max_vel: i16,
        param_6: bool,
    ) -> i16 {
        let (target_y, target_pitch) = match dst {
            Some(point) => {
                let delta = point - src;
                let flat = (delta.x * delta.x + delta.z * delta.z).sqrt();
                (
                    walk::to_s16(delta.x.atan2(delta.z)),
                    walk::to_s16(delta.y.atan2(flat)),
                )
            }
            None => (default_y, 0),
        };
        let mut out = if shape { self.shape_yaw } else { local };
        let delta_y = target_y.wrapping_sub(out);
        let max_y = i32::from(self.max_angles[SPINE][YAW]) + i32::from(self.max_angles[HEAD][YAW]);
        let min_y = i32::from(self.min_angles[SPINE][YAW]) + i32::from(self.min_angles[HEAD][YAW]);
        let moved = self.joint_move(delta_y, YAW);
        if (self.turning || ((i32::from(delta_y) >= max_y || i32::from(delta_y) <= min_y) && moved))
            && !param_6
        {
            self.turning = self.joint_follow(&mut out, target_y, max_vel);
        }
        self.joint_move(target_pitch, PITCH);
        if shape {
            self.shape_yaw = out;
        }
        out
    }

    fn joint_limitter(&self, value: i16, row: usize, axis: usize) -> i16 {
        value.min(self.max_angles[row][axis]).max(self.min_angles[row][axis])
    }

    fn joint_move(&mut self, input: i16, axis: usize) -> bool {
        let mut rest = input;
        let mut parts = [0_i16; 2];
        for row in 0..2 {
            let angle = self.joint_limitter(rest, row, axis);
            rest = rest.wrapping_sub(angle);
            parts[row] = angle;
            self.angles[row][axis] = add_calc_angle_l(
                i32::from(self.angles[row][axis]),
                i32::from(angle),
                JOINT_SCALE_I,
                i32::from(self.max_step[row][axis]),
                JOINT_MIN_STEP,
            ) as i16;
        }
        let mut hits = 0;
        for row in 0..2 {
            let gap = parts[row]
                .wrapping_sub(self.angles[row][axis])
                .unsigned_abs();
            if gap <= (self.max_step[row][axis] / 2) as u16 {
                hits += 1;
            }
        }
        hits == 2
    }

    fn joint_ang_calc(&self, out: &mut i16, target: i16, max_vel: i16) {
        let before = *out;
        let orig_diff = before.wrapping_sub(target);
        let diff = walk::add_calc_angle_s(
            orig_diff,
            0,
            JOINT_SCALE,
            max_vel,
            JOINT_MIN_ANGLE,
        );
        if diff.unsigned_abs() > JOINT_SCALE as u16 {
            *out = before.wrapping_add(diff.wrapping_sub(orig_diff));
        } else {
            *out = target;
        }
    }

    fn joint_follow(&mut self, out: &mut i16, target: i16, max_vel: i16) -> bool {
        let orig = *out;
        self.joint_ang_calc(out, target, max_vel);
        let delta = out.wrapping_sub(orig);
        let rest = target.wrapping_sub(*out);
        let mut acc_a = 0_i16;
        let mut acc_b = 0_i16;
        for row in [HEAD, SPINE] {
            acc_b = acc_b.wrapping_add(self.angles[row][YAW]);
            if (delta > 0 && acc_b > rest) || (delta < 0 && acc_b < rest) {
                let want = rest.wrapping_sub(acc_a);
                self.angles[row][YAW] = self.joint_limitter(want, row, YAW);
            }
            acc_a = acc_a.wrapping_add(self.angles[row][YAW]);
        }
        target != *out
    }

    fn settle_beam(&mut self, events: &mut SimEvents) {
        if self.anm == 2 || self.anm == 4 {
            self.make_beam(false, events);
        } else {
            self.end_beam();
        }
    }

    fn count_off(&mut self) {
        if self.carried {
            return;
        }
        if self.ground_h == NO_GROUND_Y {
            if self.off_count < OFF_COUNT_MAX {
                self.off_count += 1;
            } else {
                self.pos = self.home;
                self.yaw = self.home_yaw;
                self.shape_yaw = self.home_yaw;
                self.speed_f = 0.0;
                self.off_count = 0;
            }
        } else {
            self.off_count = 0;
        }
    }

    fn check_player_room(&mut self, ctx: &Ctx) {
        self.player_room = false;
        if !self.wakeup_check(ctx.stories) || self.finish_check(ctx.stories) {
            return;
        }
        self.player_room = ctx.player.is_some();
    }

    pub fn execute(&mut self, ctx: &mut Ctx, entity: Entity) -> bool {
        self.old = self.pos;
        self.flags &= !POSITION_BIT;
        self.check_player_room(ctx);
        let finished = self.finish_check(ctx.stories);
        if let Some(base) = ctx.pedestal {
            let lifted = Vec3::new(
                base.pos.x,
                base.pos.y + FINISH_HOME_LIFT,
                base.pos.z,
            );
            if !self.is_finish() && finished {
                self.home = lifted;
            } else if self.is_finish() && !self.is_set_home_pos() {
                self.home = lifted;
                self.on_set_home_pos();
            }
        }
        if finished {
            if self.is_finish() {
                self.pos = self.home;
            }
            if self.room < 0 || ctx.pedestal.is_none() {
                self.end_beam();
                return true;
            }
        }
        if !self.is_finish() {
            if !self.carried && self.is_gravity() {
                self.move_step();
            }
            self.settle(ctx.collision);
            self.flags |= POSITION_BIT;
            if self.player_state == Step::WalkPlayer || self.state == Step::Search {
                self.snap_to_ground();
            }
            self.count_off();
        }
        self.animation_play(ctx.events);
        if !self.event_proc(ctx) {
            if !finished {
                if self.cb1_player {
                    self.player_action(ctx);
                } else {
                    self.carry_check(ctx.stories);
                    self.check_order(ctx.stories);
                    self.npc_action(ctx);
                    if self.state != Step::Throw {
                        self.yaw = self.shape_yaw;
                    }
                }
                self.check_goal_room(ctx.stories);
            }
            self.settle_beam(ctx.events);
            self.event_order_check(ctx.stories, ctx.switches);
        }
        self.event_order(ctx.list, ctx.active, entity);
        if self.beam {
            ctx.events.list.push(SimEvent::Se(SE_LIGHT_SUSTAIN));
        }
        true
    }
}

pub fn is_servant(name: &str) -> bool {
    SERVANT_ACTORS.contains(&name)
}

#[allow(clippy::too_many_arguments)]
pub fn control(
    mut servants: Query<(Entity, &mut Actor, &mut Servant)>,
    players: Query<&Player>,
    pedestals: Query<&Actor, Without<Servant>>,
    carry: Res<Carry>,
    intent: Res<MoveIntent>,
    collision: Option<Res<Collision>>,
    scene: (Res<CameraYaw>, Res<EventList>),
    mut active: ResMut<ActiveEvent>,
    mut stories: ResMut<StoryBits>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut procs: ResMut<MsgProcs>,
    mut info: ResMut<MsgGameInfo>,
    mut msg: ResMut<Msg1Box>,
    archive: Option<Res<MsgArchives>>,
) {
    let (camera, list) = (&*scene.0, &*scene.1);
    let player = players.iter().next().copied();
    let stick = intent.axis.length();
    let stick_angle = if intent.axis == Vec2::ZERO {
        0
    } else {
        walk::to_s16(intent.axis.x.atan2(intent.axis.y))
    };
    for (entity, mut actor, mut servant) in &mut servants {
        if !servant.entered {
            servant.entered = true;
            servant.argument = Servant::argument_of(&actor.name);
            servant.init(&list, &stories, actor.pos, actor.angle[1], actor.room);
        }
        servant.carried = carry.held == Some(entity);
        let pedestal = pedestals
            .iter()
            .find(|base| base.name == servant.daiza_name());
        let mut ctx = Ctx {
            player: player.as_ref(),
            camera,
            collision: collision.as_deref(),
            pedestal,
            list,
            archive: archive.as_deref(),
            stick,
            stick_angle,
            active: &mut active,
            stories: &mut stories,
            switches: &mut switches,
            events: &mut events,
            procs: &mut procs,
            info: &mut info,
            msg: &mut msg,
        };
        servant.execute(&mut ctx, entity);
        actor.pos = servant.pos;
        actor.angle[1] = if servant.state == Step::Throw {
            servant.yaw
        } else {
            servant.shape_yaw
        };
    }
}