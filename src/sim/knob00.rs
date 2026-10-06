use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::{Mat4, Vec3};

use crate::assets::MsgArchives;
use crate::text::driver::TextFlags;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgGameInfo, MsgProcs,
    fop_msgm_message_set,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED,
    STATUS_MSG_ENDS, STATUS_TYPING,
};

use super::actor::Actor;
use super::daynight::{DayNight, TIME_NIGHT};
use super::door::StayRoom;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{Collect, EventFlags, EventReg, SYMBOL_DIN, SYMBOL_NAYRU};
use super::inventory::{Equipped, check_get_item};
use super::player::Player;
use super::stream::{angle_s, cos_s, sin_s};
use super::walk;

pub const KNOB00_ACTORS: [&str; 8] = [
    "KNOB00",
    "KNOB01",
    "KNOB02",
    "KNOB03",
    "KNOB00D",
    "KNOB01D",
    "KNOB02D",
    "KNOB03D",
];

pub fn is_knob00(name: &str) -> bool {
    KNOB00_ACTORS.contains(&name)
}

pub const STAFF_SHUTTER_DOOR: &str = "SHUTTER_DOOR";

const SWBIT_MASK: u32 = 0xFF;
const TYPE_SHIFT: u32 = 0x08;
const TYPE_MASK: u32 = 0x0F;
const EVENT_NO_SHIFT: u32 = 0x0C;
const SHAPE_SHIFT: u32 = 0x14;
const TYPE2_SHIFT: u32 = 0x1C;
const ROOM_MASK: u32 = 0x3F;
const ROOM_SHIFT: u32 = 0x06;
const ARG1_SHIFT: u32 = 0x08;
const ROOM_NONE: u8 = 0x3F;

const A_INIT: u8 = 0;
const A_WAIT: u8 = 1;
const A_DEMO: u8 = 2;
const A_TALK: u8 = 3;
const A_TALK_WAIT: u8 = 4;
const A_PASSWORD: u8 = 5;
const A_PASSWORD2: u8 = 6;
const A_VILLA_TALK: u8 = 7;
const A_VILLA_WAIT: u8 = 8;
const A_FIGURE_TALK: u8 = 9;
const A_FIGURE_WAIT: u8 = 10;
const A_ITEM: u8 = 11;

const DOOR_MSG_ALWAYS: u8 = 1;
const DOOR_MSG_NIGHT: u8 = 2;
const DOOR_MSG_DAY: u8 = 3;
const DOOR_PASSWORD: u8 = 4;
const DOOR_VILLA: u8 = 5;
const DOOR_FIGURE: u8 = 6;

const SHAPE_MARKET: u8 = 4;
const SHAPE_FIGURE: u8 = 6;
const FIGURE_EXCEPTION: i16 = 0x366A;

const EXC_NAYRU: i16 = 0x6A6;
const EXC_UNK_0520: i16 = 0x6A7;
const EXC_ENDLESS_NIGHT: i16 = 0x6A8;
const EXC_UNK_1701: i16 = 0x6A9;
const EXC_DIN: i16 = 0x6AA;

const EVENT_UNK_2110: u32 = 0x2110;
const EVENT_UNK_0520: u32 = 0x0520;
const EVENT_ENDLESS_NIGHT: u32 = 0x0A02;
const EVENT_UNK_1701: u32 = 0x1701;
const EVENT_UNK_1A80: u32 = 0x1A80;
const EVENT_UNK_3401: u32 = 0x3401;
const EVENT_UNK_2D80: u32 = 0x2D80;
const EVENT_UNK_1910: u32 = 0x1910;
const EVENT_UNK_3B20: u32 = 0x3B20;
const EVENT_REG_BA0F: u16 = 0xBA0F;

const ITEM_BOMB_BAG: u8 = 0x31;

const FLAG_OPEN: u16 = 0x0001;
const FLAG_EVENT_OFF: u16 = 0x0004;

const FRONT_NONE: u8 = 2;
const FRONT_HALF_TURN: i16 = 0x4000;
const CHECK_ANGLE_GATE: i16 = 0x5000;
const HALF_TURN: i16 = 0x7FFF;
const START_DEMO_GATE: i16 = 0x1000;
const START_DEMO_GATE2: i16 = -1000;

const AREA_FRONT_SQ: f32 = 6400.0;
const AREA_SIDE_SQ: f32 = 12100.0;
const AREA_DIST_SQ: f32 = 62500.0;
const ATTENTION_LIFT: f32 = 150.0;
const MARKET_LIFT: f32 = 60.0;

const ADJUST_PULL: f32 = -70.0;
const START_PULL_Z: f32 = -70.0;
const START_PUSH_X: f32 = -43.0;
const START_PUSH_Z: f32 = -14.8;
const START_PULL_X: f32 = 46.0;
const START_PULL2_Z: f32 = -86.0;
const ADJUST_WEIGHT: f32 = 0.8;
const ADJUST_LERP: f32 = 0.2;
const STOP_STEPS: i16 = 10;
const RESTART_RAD: f32 = 180.0;
const HALF_TURN_S: i16 = -32768;

const MSG_ID_OK: i16 = 0x1B18;
const MSG_ID_NG: i16 = 0x1B19;
const MSG_ID_HINT: i16 = 0x1B1A;
const MSG_ID_PASSWORD_BASE: i32 = 0x1B37;
const MSG_CONTINUES_1: i16 = 0x1BBD;
const MSG_CONTINUES_2: i16 = 0x1BC0;
const MSG_CONTINUES_3: i16 = 0x1BC1;
const MSG_CONTINUES_4: i16 = 0x1BC2;
const MSG_VIBRATE: i16 = 0x06A8;

const PASS_STEP_0: u8 = 0;
const PASS_STEP_1: u8 = 1;
const PASS_STEP_2: u8 = 2;
const PASS_STEP_3: u8 = 3;
const PASS_STEP_10: u8 = 10;

const NAME_CHECK_CANCEL: u8 = 0;
const NAME_CHECK_OK: u8 = 1;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_RANGE: f32 = 6.0;

const BCK_ADOOR: usize = 0;
const BCK_BDOOR: usize = 1;
const BCK_TABLE: [usize; 4] = [BCK_ADOOR, BCK_BDOOR, BCK_ADOOR, BCK_BDOOR];
const BCK_FRAME_MAX: f32 = 79.0;
const OPEN_SET_FRAME: f32 = 34.0;

const SE_DOOR_N_OPEN: u32 = 0x696E;
const SE_DOOR_N_CLOSE_1: u32 = 0x696F;
const SE_DOOR_N_CLOSE_2: u32 = 0x6970;

const DEMO_SETSTART: i32 = 9;
const DEMO_SETANGLE: i32 = 10;
const DEMO_ADJUSTMENT: i32 = 11;
const DEMO_OPEN_PUSH: i32 = 12;
const DEMO_OPEN_PULL: i32 = 13;
const DEMO_OPEN_PUSH2: i32 = 14;
const DEMO_OPEN_PULL2: i32 = 15;
const DEMO_TALK: i32 = 16;
const DEMO_SETSTART_PUSH: i32 = 17;
const DEMO_SETSTART_PULL: i32 = 18;
const DEMO_NONE: i32 = -1;
const ACTION_TABLE: [&str; 22] = [
    "WAIT",
    "STOP_OPEN",
    "STOP_CLOSE",
    "OPEN",
    "CLOSE",
    "SMOKE",
    "SMOKE_END",
    "SETGOAL",
    "UNLOCK",
    "SETSTART",
    "SETANGLE",
    "ADJUSTMENT",
    "OPEN_PUSH",
    "OPEN_PULL",
    "OPEN_PUSH2",
    "OPEN_PULL2",
    "TALK",
    "SETSTART_PUSH",
    "SETSTART_PULL",
    "END_CHECK",
    "DROP_BF",
    "DROP_AF",
];

const EVT_CMD_NONE: u8 = 0x00;
const EVT_CMD_INTALK: u8 = 0x01;
const EVT_CMD_INDEMO: u8 = 0x02;
const EVT_CMD_INDOOR: u8 = 0x03;
const EVT_CMD_INGETITEM: u8 = 0x04;
const EVT_CMD_INCATCH: u8 = 0x06;

const CULL_MIN: [f32; 3] = [-150.0, 0.0, -150.0];
const CULL_MAX: [f32; 3] = [150.0, 300.0, 150.0];

const NAME_MAX: usize = 20;
const NAME_CHECK_IDLE: u8 = 0xFF;
const STATUS_NONE: u8 = 0xFF;
const OPEN_FADE_RATE: f32 = 0.05;

const EVENT_IDX_NONE: i16 = -1;

const M2C6_FRONT: u8 = 7;
const M2C6_BACK: u8 = 8;
const M2C6_TALK: u8 = 9;

const EVENT_IDX_TABLE: [i16; 12] = [
    EVENT_IDX_NONE,
    EVENT_IDX_NONE,
    EVENT_IDX_NONE,
    EVENT_IDX_NONE,
    EVENT_IDX_NONE,
    EVENT_IDX_NONE,
    EVENT_IDX_NONE,
    EVENT_IDX_NONE,
    EVENT_IDX_NONE,
    EVENT_IDX_NONE,
    EVENT_IDX_NONE,
    EVENT_IDX_NONE,
];

const OPEN_FADE_FRAME: f32 = 15.0;
const OPEN_SOUND_FRAME_0: f32 = 25.0;
const OPEN_SOUND_FRAME_1: f32 = 28.0;
const OPEN_SOUND_FRAME_2: f32 = 49.0;
const OPEN_SOUND_FRAME_3: f32 = 60.0;
const OPEN_SOUND_FRAME_4: f32 = 64.0;
const OPEN_SOUND_FRAME_5: f32 = 67.0;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NameEntry {
    pub open: bool,
    pub check: u8,
    pub input: [u8; NAME_MAX],
    pub input_len: usize,
    pub answer: [u8; NAME_MAX],
    pub answer_len: usize,
}

impl Default for NameEntry {
    fn default() -> Self {
        Self {
            open: false,
            check: NAME_CHECK_IDLE,
            input: [0; NAME_MAX],
            input_len: 0,
            answer: [0; NAME_MAX],
            answer_len: 0,
        }
    }
}

impl NameEntry {
    pub fn set_input(&mut self, text: &str) {
        let take = text.len().min(NAME_MAX);
        self.input = [0; NAME_MAX];
        if let Some(slot) = self.input.get_mut(..take) {
            slot.copy_from_slice(text.as_bytes().get(..take).unwrap_or(&[]));
        }
        self.input_len = take;
    }

    pub fn set_answer(&mut self, text: &str) {
        let take = text.len().min(NAME_MAX);
        self.answer = [0; NAME_MAX];
        if let Some(slot) = self.answer.get_mut(..take) {
            slot.copy_from_slice(text.as_bytes().get(..take).unwrap_or(&[]));
        }
        self.answer_len = take;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BckAnm {
    frame: f32,
    end: f32,
}

impl BckAnm {
    const fn new() -> Self {
        Self {
            frame: 0.0,
            end: BCK_FRAME_MAX,
        }
    }

    fn init(&mut self) {
        self.frame = 0.0;
        self.end = BCK_FRAME_MAX;
    }

    fn set_frame(&mut self, frame: f32) {
        self.frame = frame;
    }

    fn play(&mut self) -> bool {
        self.frame = (self.frame + 1.0).min(self.end);
        self.frame < self.end
    }

    fn check_pass(&mut self, frame: f32) -> bool {
        if self.frame >= frame {
            self.frame = frame - 1.0;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoorMsg {
    pid: u32,
    state: u8,
    msg_id: i16,
}

impl DoorMsg {
    const fn new() -> Self {
        Self {
            pid: FPC_M_ERROR_PROCESS_ID,
            state: 0,
            msg_id: 0,
        }
    }

    fn init(&mut self, msg_id: i16) {
        self.msg_id = msg_id;
        self.pid = FPC_M_ERROR_PROCESS_ID;
        self.state = 0;
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Knob00 {
    prm: u32,
    door_type: u8,
    exception: i16,
    home_yaw: i16,
    shape_angle: i16,
    attention: Vec3,
    angle_vec: Vec3,
    front_room: u8,
    back_room: u8,
    ship_id: u8,
    arg1: u8,
    event_no: u8,
    view_room: u8,
    room_no: i8,
    room_no2: i8,
    from_room: u8,
    to_room: u8,
    front_check: u8,
    action: u8,
    command: u8,
    flags: u16,
    stop: i16,
    pass_state: u8,
    prm2c6: u8,
    event_idx: i16,
    bgw: bool,
    bgw_moved: bool,
    bck: usize,
    fade_rate: f32,
    fade_dur: f32,
    fade_white: bool,
    fade_out: bool,
    anim: BckAnm,
    msg: DoorMsg,
    seed: [i32; 3],
    event_reset: bool,
    talk_partner_set: bool,
    talk_partner_off: bool,
    talk_partner_found: bool,
    room_flag_cleared: bool,
    vibrate: bool,
    restart_pos: Vec3,
    restart_angle: i16,
    restart_room: i8,
}

impl Knob00 {
    pub fn spawn(parameters: u32, angle: [i16; 3]) -> Self {
        let shape = ((parameters >> SHAPE_SHIFT) & SWBIT_MASK) as u8;
        let mut door_type = ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8;
        let mut exception = angle[2];
        let home_yaw = angle[1];
        let mut attention = Vec3::ZERO;
        attention.y = ATTENTION_LIFT;
        if shape == SHAPE_MARKET && door_type == DOOR_MSG_ALWAYS {
            door_type = DOOR_VILLA;
            attention.y += MARKET_LIFT;
        }
        if shape == SHAPE_FIGURE {
            door_type = DOOR_FIGURE;
            exception = FIGURE_EXCEPTION;
        }
        let packed_x = u32::from(angle[0] as u16);
        let packed_z = u32::from(angle[2] as u16);
        Self {
            prm: parameters,
            door_type,
            exception,
            home_yaw,
            shape_angle: home_yaw,
            attention,
            angle_vec: Vec3::new(sin_s(home_yaw), 0.0, cos_s(home_yaw)),
            front_room: (packed_x & ROOM_MASK) as u8,
            back_room: ((packed_x >> ROOM_SHIFT) & ROOM_MASK) as u8,
            ship_id: (packed_z & ROOM_MASK) as u8,
            arg1: ((packed_z >> ARG1_SHIFT) & SWBIT_MASK) as u8,
            event_no: ((parameters >> EVENT_NO_SHIFT) & SWBIT_MASK) as u8,
            view_room: 1,
            room_no: -1,
            room_no2: -1,
            from_room: ROOM_NONE,
            to_room: ROOM_NONE,
            front_check: FRONT_NONE,
            action: A_INIT,
            command: EVT_CMD_NONE,
            flags: 0,
            stop: 0,
            pass_state: PASS_STEP_0,
            prm2c6: M2C6_FRONT,
            event_idx: EVENT_IDX_NONE,
            bgw: false,
            bgw_moved: true,
            bck: BCK_ADOOR,
            fade_rate: 1.0,
            fade_dur: 1.0,
            fade_white: false,
            fade_out: false,
            anim: BckAnm::new(),
            msg: DoorMsg::new(),
            seed: [0; 3],
            event_reset: false,
            talk_partner_set: false,
            talk_partner_off: false,
            talk_partner_found: false,
            room_flag_cleared: false,
            vibrate: false,
            restart_pos: Vec3::ZERO,
            restart_angle: 0,
            restart_room: -1,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.adjoin_player() && self.view_room != 0
    }

    pub fn shape_type(&self) -> u8 {
        ((self.prm >> SHAPE_SHIFT) & SWBIT_MASK) as u8
    }

    pub fn type2(&self) -> u8 {
        ((self.prm >> TYPE2_SHIFT) & SWBIT_MASK) as u8
    }

    pub fn swbit(&self) -> u8 {
        (self.prm & SWBIT_MASK) as u8
    }

    pub fn front_room(&self) -> u8 {
        self.front_room
    }

    pub fn back_room(&self) -> u8 {
        self.back_room
    }

    pub fn ship_id(&self) -> u8 {
        self.ship_id
    }

    pub fn arg1(&self) -> u8 {
        self.arg1
    }

    pub fn event_no(&self) -> u8 {
        self.event_no
    }

    pub fn exception(&self) -> i16 {
        self.exception
    }

    pub fn door_type(&self) -> u8 {
        self.door_type
    }

    pub fn action(&self) -> u8 {
        self.action
    }

    pub fn bck(&self) -> usize {
        self.bck
    }

    pub fn msg_id(&self) -> i16 {
        self.msg.msg_id
    }

    pub fn pass_state(&self) -> u8 {
        self.pass_state
    }

    pub fn restart(&self) -> (Vec3, i16, i8) {
        (self.restart_pos, self.restart_angle, self.restart_room)
    }

    pub fn bgw_is_moving(&self) -> bool {
        self.bgw_moved
    }

    pub fn bgw_is_used(&self) -> bool {
        self.bgw
    }

    pub fn fade(&self) -> Option<(f32, f32, bool)> {
        if self.fade_out {
            Some((self.fade_rate, self.fade_dur, self.fade_white))
        } else {
            None
        }
    }

    pub fn event_reset_pending(&self) -> bool {
        self.event_reset
    }

    pub fn talk_partner_off_pending(&self) -> bool {
        self.talk_partner_off
    }

    pub fn talk_partner_set_pending(&self) -> bool {
        self.talk_partner_set
    }

    pub fn talk_partner_found(&self) -> bool {
        self.talk_partner_found
    }

    pub fn room_no(&self) -> i8 {
        self.room_no
    }

    pub fn room_no2(&self) -> i8 {
        self.room_no2
    }

    pub fn from_room(&self) -> u8 {
        self.from_room
    }

    pub fn to_room(&self) -> u8 {
        self.to_room
    }

    pub fn prm2c6(&self) -> u8 {
        self.prm2c6
    }

    pub fn event_idx(&self) -> i16 {
        self.event_idx
    }

    pub fn front_check_of(&self) -> u8 {
        self.front_check
    }

    pub fn anim_frame(&self) -> f32 {
        self.anim.frame
    }

    pub fn room_flag_cleared(&self) -> bool {
        self.room_flag_cleared
    }

    pub fn vibrate_pending(&self) -> bool {
        self.vibrate
    }

    pub fn demo_action_of(&self, list: &EventList, cut: usize) -> i32 {
        let Some(name) = list.cut_name(cut) else {
            return DEMO_NONE;
        };
        Self::action_index(name)
    }

    pub fn password_msg_no(&self, base: u8) -> u32 {
        (i32::from(base) + MSG_ID_PASSWORD_BASE) as u32
    }

    pub fn set_command(&mut self, command: u8) {
        self.command = command;
    }

    pub fn command_none(&self) -> bool {
        self.command == EVT_CMD_NONE
    }

    pub fn command_talk(&self) -> bool {
        self.command == EVT_CMD_INTALK
    }

    pub fn command_demo_accrpt(&self) -> bool {
        self.command == EVT_CMD_INDEMO
    }

    pub fn command_door(&self) -> bool {
        self.command == EVT_CMD_INDOOR
    }

    pub fn command_item(&self) -> bool {
        self.command == EVT_CMD_INGETITEM
    }

    pub fn command_catch(&self) -> bool {
        self.command == EVT_CMD_INCATCH
    }

    pub fn check_flag(&self, bit: u16) -> bool {
        self.flags & bit != 0
    }

    pub fn val_ship_id(&self) -> bool {
        self.ship_id != ROOM_NONE
    }

    pub fn adjoin_player(&self) -> bool {
        self.front_room == ROOM_NONE || self.back_room == ROOM_NONE
    }

    pub fn base_mtx(&self, pos: Vec3, yaw: i16) -> Mat4 {
        Mat4::from_translation(pos) * Mat4::from_rotation_y(walk::from_s16(yaw))
    }

    pub fn action_index(name: &str) -> i32 {
        match ACTION_TABLE.iter().position(|entry| *entry == name) {
            Some(index) => i32::try_from(index).unwrap_or(DEMO_NONE),
            None => DEMO_NONE,
        }
    }

    pub fn event_idx_at(index: u8) -> i16 {
        EVENT_IDX_TABLE
            .get(usize::from(index))
            .copied()
            .unwrap_or(EVENT_IDX_NONE)
    }

    pub fn bck_of(arg1: i32) -> usize {
        let index = usize::try_from(arg1).unwrap_or(0);
        BCK_TABLE.get(index).copied().unwrap_or(BCK_ADOOR)
    }

    pub fn character_exchange(bytes: &mut [u8]) {
        for byte in bytes.iter_mut() {
            if *byte == 0 {
                break;
            }
            if byte.is_ascii_uppercase() {
                byte.make_ascii_lowercase();
            }
        }
    }

    fn set_action(&mut self, action: u8) {
        self.action = action;
    }

    fn next_action(&mut self) {
        self.action = self.action.wrapping_add(1);
    }

    fn on_flag(&mut self, bit: u16) {
        self.flags |= bit;
    }

    fn off_flag(&mut self, bit: u16) {
        self.flags &= !bit;
    }

    fn rnd(&mut self) -> f32 {
        for lane in 0..3 {
            self.seed[lane] = (self.seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
        }
        let sum = self.seed[0] as f32 / RND_SCALE[0]
            + self.seed[1] as f32 / RND_SCALE[1]
            + self.seed[2] as f32 / RND_SCALE[2];
        (sum % 1.0).abs()
    }

    fn rnd_f(&mut self, range: f32) -> f32 {
        self.rnd() * range
    }

    fn bgw_move(&mut self) {
        self.bgw_moved = true;
    }

    fn front_check_old(&self, here: Vec3, yaw: i16, player: Vec3) -> u8 {
        let delta = player - here;
        let bearing = angle_s(delta.x.atan2(delta.z));
        let angle = bearing.wrapping_sub(yaw).unsigned_abs();
        if angle < FRONT_HALF_TURN as u16 {
            0
        } else {
            1
        }
    }

    fn check_face(&self, here: Vec3, yaw: i16, stay: i8, player: Vec3) -> u8 {
        if self.front_room == self.back_room {
            return self.front_check_old(here, yaw, player);
        }
        match stay {
            room if room == i16::from(self.front_room) as i8 => 0,
            room if room == i16::from(self.back_room) as i8 => 1,
            _ => FRONT_NONE,
        }
    }

    fn view_room_no(&self, eye: Vec3, here: Vec3) -> u8 {
        let delta = eye - here;
        let dot = delta.x * self.angle_vec.x + delta.z * self.angle_vec.z;
        if dot < 0.0 {
            self.back_room
        } else {
            self.front_room
        }
    }

    fn check_area(
        &self,
        f1: f32,
        f2: f32,
        dist_xz_sq_max: f32,
        here: Vec3,
        yaw: i16,
        player: Vec3,
        player_yaw: i16,
    ) -> bool {
        let delta = player - here;
        let dist_xz_sq = delta.x * delta.x + delta.z * delta.z;
        if dist_xz_sq > dist_xz_sq_max {
            return false;
        }
        let unit = delta.normalize();
        let dot = unit.x * self.angle_vec.x + unit.z * self.angle_vec.z;
        let d3 = dist_xz_sq * dot * dot;
        if d3 > f2 {
            return false;
        }
        if dist_xz_sq - d3 > f1 {
            return false;
        }
        let mut check_angle = yaw;
        if self.front_check == 1 {
            check_angle = check_angle.wrapping_add(HALF_TURN);
        }
        check_angle.wrapping_sub(player_yaw).unsigned_abs() < CHECK_ANGLE_GATE as u16
    }

    fn msg_door(&self, night: bool) -> bool {
        if self.door_type == DOOR_MSG_ALWAYS {
            return true;
        }
        if night {
            if self.door_type == DOOR_MSG_NIGHT {
                return true;
            }
        } else if self.door_type == DOOR_MSG_DAY {
            return true;
        }
        false
    }

    fn chk_exception(&self, collect: &Collect, flags: &EventFlags) -> bool {
        match self.exception {
            EXC_NAYRU => {
                if collect.is_symbol(SYMBOL_NAYRU) {
                    return true;
                }
                !flags.test(EVENT_UNK_2110)
            }
            EXC_UNK_0520 => !flags.test(EVENT_UNK_0520),
            EXC_ENDLESS_NIGHT => !flags.test(EVENT_ENDLESS_NIGHT) || flags.test(EVENT_UNK_2110),
            EXC_UNK_1701 => flags.test(EVENT_UNK_1701),
            EXC_DIN => !collect.is_symbol(SYMBOL_DIN) || flags.test(EVENT_UNK_1A80),
            _ => false,
        }
    }

    fn demo_action(&self, env: &Env<'_>) -> i32 {
        let Some((cut, _)) = env.staff_cut else {
            return DEMO_NONE;
        };
        let Some(name) = env.list.cut_name(cut) else {
            return DEMO_NONE;
        };
        Self::action_index(name)
    }

    fn open_init_com(&mut self) {
        if self.front_check == 0 {
            self.from_room = self.front_room;
            self.to_room = self.back_room;
        } else {
            self.from_room = self.back_room;
            self.to_room = self.front_room;
        }
    }

    fn close_end_com(&mut self, env: &Env<'_>) {
        if self.from_room != self.to_room
            && self.from_room != ROOM_NONE
            && self.to_room != ROOM_NONE
        {
            self.room_flag_cleared = true;
        }
        let delta = env.ppos - env.here;
        let dot = delta.x * self.angle_vec.x + delta.z * self.angle_vec.z;
        let rad = if dot < 0.0 { RESTART_RAD } else { -RESTART_RAD };
        self.restart_pos = Vec3::new(
            env.here.x - rad * self.angle_vec.x,
            env.here.y,
            env.here.z - rad * self.angle_vec.z,
        );
        self.restart_angle = if dot > 0.0 {
            env.yaw
        } else {
            env.yaw.wrapping_add(HALF_TURN_S)
        };
        self.restart_room = env.proom;
    }

    fn start_demo_proc(&mut self, player_yaw: i16) {
        self.shape_angle = self.home_yaw;
        let mut delta = player_yaw.wrapping_sub(self.home_yaw);
        if delta < 0 {
            delta = delta.wrapping_neg();
        }
        if delta < START_DEMO_GATE && delta > START_DEMO_GATE2 {
            self.shape_angle = self.shape_angle.wrapping_add(HALF_TURN);
        }
    }

    fn init_open_demo(&mut self) {
        self.shape_angle = self.home_yaw;
        if self.front_check == 1 {
            self.shape_angle = self.shape_angle.wrapping_add(HALF_TURN);
        }
    }

    fn set_event_prm(&mut self, env: &Env<'_>) {
        if self.front_check == 0 {
            self.prm2c6 = M2C6_FRONT;
        } else {
            self.prm2c6 = M2C6_BACK;
        }
        if self.action == A_TALK_WAIT
            || self.action == A_PASSWORD
            || self.action == A_VILLA_WAIT
            || self.action == A_FIGURE_WAIT
        {
            self.prm2c6 = M2C6_TALK;
        }
        if !self.check_area(
            AREA_FRONT_SQ,
            AREA_SIDE_SQ,
            AREA_DIST_SQ,
            env.here,
            env.yaw,
            env.ppos,
            env.pyaw,
        ) {
            self.off_flag(FLAG_EVENT_OFF);
        } else {
            self.event_idx = Self::event_idx_at(self.prm2c6);
        }
    }

    fn open_init(&mut self, arg1: i32) {
        self.bck = Self::bck_of(arg1);
        self.anim.init();
        self.open_init_com();
        self.on_flag(FLAG_OPEN);
        if arg1 >= 2 {
            self.anim.set_frame(OPEN_SET_FRAME);
        } else {
            self.bgw = false;
        }
    }

    fn open_proc(&mut self, arg1: i32, env: &mut Env<'_>) -> bool {
        if self.anim.play() {
            return true;
        }
        if arg1 == DEMO_OPEN_PUSH || arg1 == DEMO_OPEN_PULL {
            let b_var3 = env.night;
            let b_var4 = arg1 == DEMO_OPEN_PULL;
            let tmp = b_var4 == b_var3;
            if self.anim.check_pass(OPEN_FADE_FRAME) {
                self.fade_rate = 0.0;
                self.fade_dur = OPEN_FADE_RATE;
                self.fade_white = tmp;
                self.fade_out = true;
            }
        }
        if arg1 == DEMO_OPEN_PUSH || arg1 == DEMO_OPEN_PUSH2 {
            if self.anim.check_pass(OPEN_SOUND_FRAME_1) {
                env.se(SE_DOOR_N_OPEN);
            } else if self.anim.check_pass(OPEN_SOUND_FRAME_3) {
                env.se(SE_DOOR_N_CLOSE_1);
            } else if self.anim.check_pass(OPEN_SOUND_FRAME_5) && arg1 == DEMO_OPEN_PUSH2 {
                env.se(SE_DOOR_N_CLOSE_2);
            }
        } else if self.anim.check_pass(OPEN_SOUND_FRAME_0) {
            env.se(SE_DOOR_N_OPEN);
        } else if self.anim.check_pass(OPEN_SOUND_FRAME_2) {
            env.se(SE_DOOR_N_CLOSE_1);
        } else if self.anim.check_pass(OPEN_SOUND_FRAME_4) && arg1 == DEMO_OPEN_PULL2 {
            env.se(SE_DOOR_N_CLOSE_2);
        }
        false
    }

    fn open_end(&mut self, env: &Env<'_>) {
        self.off_flag(FLAG_OPEN);
        self.bgw = true;
        self.close_end_com(env);
    }

    fn chk_password(&self, env: &Env<'_>) -> i32 {
        if !env.flags.test(EVENT_UNK_2110) {
            return -1;
        }
        let base = i32::from(env.regs.reg(EVENT_REG_BA0F));
        let alen = env.names.answer_len.min(NAME_MAX);
        let ilen = env.names.input_len.min(NAME_MAX);
        let mut answer = env.names.answer;
        let mut input = env.names.input;
        Self::character_exchange(&mut answer[..alen]);
        Self::character_exchange(&mut input[..ilen]);
        if answer[..alen] != input[..ilen] {
            return -1;
        }
        base
    }

    fn set_start(&self, arg1: f32, arg2: f32, here: Vec3, player: &mut Player) {
        let mut sp14 = here;
        let angle = self.shape_angle.wrapping_add(HALF_TURN);
        sp14.x += sin_s(angle) * arg2 - cos_s(angle) * arg1;
        sp14.z += cos_s(angle) * arg2 + sin_s(angle) * arg1;
        player.pos = sp14;
    }

    fn set_angle(&self, player: &mut Player) {
        player.yaw = walk::from_s16(self.shape_angle.wrapping_add(HALF_TURN));
    }

    fn adjustment_proc(&mut self, here: Vec3, player: &mut Player) -> bool {
        let angle = self.shape_angle.wrapping_add(HALF_TURN);
        let mut sp08 = player.pos;
        let mut sp14 = here;
        sp14.x += sin_s(angle) * ADJUST_PULL;
        sp14.z += cos_s(angle) * ADJUST_PULL;
        if self.stop > 0 {
            sp08.x = sp08.x * ADJUST_WEIGHT + sp14.x * ADJUST_LERP;
            sp08.z = sp08.z * ADJUST_WEIGHT + sp14.z * ADJUST_LERP;
            player.pos = sp08;
            self.stop -= 1;
            false
        } else {
            player.pos = sp14;
            true
        }
    }

    fn demo_proc(&mut self, env: &mut Env<'_>) -> bool {
        let mut ret = false;
        let demo_action = self.demo_action(env);
        if env.edit_advance() {
            match demo_action {
                DEMO_SETSTART => {
                    if let Some(player) = env.player.as_deref_mut() {
                        self.set_start(0.0, START_PULL_Z, env.here, player);
                    }
                }
                DEMO_SETSTART_PUSH => {
                    if let Some(player) = env.player.as_deref_mut() {
                        self.set_start(START_PUSH_X, START_PUSH_Z, env.here, player);
                    }
                    if self.bgw {
                        self.bgw = false;
                    }
                }
                DEMO_SETSTART_PULL => {
                    if let Some(player) = env.player.as_deref_mut() {
                        self.set_start(START_PULL_X, START_PULL2_Z, env.here, player);
                    }
                    if self.bgw {
                        self.bgw = false;
                    }
                }
                DEMO_OPEN_PUSH => self.open_init(0),
                DEMO_OPEN_PULL => self.open_init(1),
                DEMO_OPEN_PUSH2 => self.open_init(2),
                DEMO_OPEN_PULL2 => self.open_init(3),
                DEMO_SETANGLE => {
                    if let Some(player) = env.player.as_deref_mut() {
                        self.set_angle(player);
                    }
                }
                DEMO_ADJUSTMENT => self.stop = STOP_STEPS,
                DEMO_TALK => self.msg.init(self.exception),
                _ => {}
            }
        }
        match demo_action {
            DEMO_OPEN_PUSH | DEMO_OPEN_PULL | DEMO_OPEN_PUSH2 | DEMO_OPEN_PULL2 => {
                if self.check_flag(FLAG_OPEN) {
                    if self.open_proc(demo_action, env) {
                        self.open_end(env);
                        env.cut_end();
                    }
                } else {
                    env.cut_end();
                }
            }
            DEMO_ADJUSTMENT => {
                let done = match env.player.as_deref_mut() {
                    Some(player) => self.adjustment_proc(env.here, player),
                    None => false,
                };
                if done {
                    env.cut_end();
                }
            }
            DEMO_TALK => {
                if self.demo_proc_talk(env) {
                    ret = true;
                }
            }
            _ => env.cut_end(),
        }
        ret
    }

    fn demo_proc_talk(&mut self, env: &mut Env<'_>) -> bool {
        if self.action == A_FIGURE_TALK {
            self.talk_partner_found = true;
        }
        self.msg_proc(env)
    }

    fn demo_proc2(&mut self, env: &mut Env<'_>) -> bool {
        match self.demo_action(env) {
            DEMO_TALK => true,
            _ => {
                env.cut_end();
                false
            }
        }
    }

    fn msg_proc(&mut self, env: &mut Env<'_>) -> bool {
        match self.msg.state {
            0 => {
                self.msg.pid = env.message_set(self.msg.msg_id, self.attention);
                if self.msg.pid != FPC_M_ERROR_PROCESS_ID {
                    self.msg.state += 1;
                }
            }
            1 => {
                if env.msg_live() {
                    self.msg.state += 1;
                }
            }
            2 => {
                if env.msg_live() && env.box_status() == STATUS_TYPING {
                    if self.msg.msg_id == MSG_CONTINUES_1 || self.msg.msg_id == MSG_VIBRATE {
                        self.vibrate = true;
                    }
                    self.msg.state += 1;
                }
            }
            3 => {
                if env.msg_live() && env.box_status() == STATUS_MSG_DISPLAYED {
                    match self.msg.msg_id {
                        MSG_CONTINUES_1 | MSG_CONTINUES_2 | MSG_CONTINUES_3 | MSG_CONTINUES_4 => {
                            self.msg.msg_id = self.msg.msg_id.wrapping_add(1);
                            env.set_box_status(STATUS_MSG_CONTINUES);
                            env.message_set(self.msg.msg_id, self.attention);
                        }
                        _ => {
                            self.msg.state += 1;
                            env.set_box_status(STATUS_MSG_ENDS);
                        }
                    }
                }
            }
            _ => {}
        }
        if env.msg_live() && env.box_status() == STATUS_BOX_CLOSED {
            env.set_box_status(STATUS_MSG_DESTROYED);
            true
        } else {
            false
        }
    }

    fn action_wait(&mut self, env: &mut Env<'_>) -> bool {
        if self.command_door() {
            self.init_open_demo();
            self.set_action(A_DEMO);
            self.demo_proc(env);
        } else {
            self.set_event_prm(env);
        }
        true
    }

    fn action_demo(&mut self, env: &mut Env<'_>) -> bool {
        if self.event_idx >= 0 {
            if self.prm2c6 != M2C6_FRONT && self.prm2c6 != M2C6_BACK {
                self.set_action(A_WAIT);
                self.event_reset = true;
                self.shape_angle = self.home_yaw;
            }
        } else {
            self.demo_proc(env);
        }
        true
    }

    fn action_talk(&mut self, env: &mut Env<'_>) -> bool {
        if self.demo_proc(env) {
            if self.action == A_FIGURE_TALK {
                self.talk_partner_off = true;
            }
            self.next_action();
            self.event_reset = true;
            self.shape_angle = self.home_yaw;
        }
        true
    }

    fn action_talk_wait(&mut self, env: &mut Env<'_>) -> bool {
        if self.command_door() {
            self.init_open_demo();
            self.set_action(A_TALK);
            self.demo_proc(env);
        } else {
            self.set_event_prm(env);
        }
        true
    }

    fn action_password2(&mut self, env: &mut Env<'_>) -> bool {
        match self.pass_state {
            PASS_STEP_0 => {
                if self.demo_proc2(env) {
                    self.pass_state = self.pass_state.wrapping_add(1);
                    if !env.flags.test(EVENT_UNK_3B20) {
                        env.flags.set(EVENT_UNK_3B20);
                        let roll = self.rnd_f(RND_RANGE) as i32 as i8;
                        env.regs.set_reg(EVENT_REG_BA0F, roll as u8);
                    }
                    let tmp = i32::from(env.regs.reg(EVENT_REG_BA0F));
                    self.msg.init((tmp + i32::from(MSG_ID_HINT)) as i16);
                }
            }
            PASS_STEP_1 => {
                if self.msg_proc(env) {
                    self.pass_state = self.pass_state.wrapping_add(1);
                    env.names.open = true;
                }
            }
            PASS_STEP_2 => match env.names.check {
                NAME_CHECK_OK => {
                    if self.chk_password(env) == -1 {
                        self.pass_state = PASS_STEP_10;
                        self.msg.init(MSG_ID_NG);
                    } else {
                        env.flags.set(EVENT_UNK_1910);
                        self.pass_state = self.pass_state.wrapping_add(1);
                        self.msg.init(MSG_ID_OK);
                    }
                }
                NAME_CHECK_CANCEL => {
                    self.pass_state = PASS_STEP_10;
                    self.msg.init(MSG_ID_NG);
                }
                _ => {}
            },
            PASS_STEP_3 => {
                if self.msg_proc(env) {
                    self.set_action(A_WAIT);
                    self.event_reset = true;
                    self.shape_angle = self.home_yaw;
                }
            }
            PASS_STEP_10 => {
                if self.msg_proc(env) {
                    self.set_action(A_PASSWORD);
                    self.event_reset = true;
                    self.shape_angle = self.home_yaw;
                }
            }
            _ => {}
        }
        true
    }

    fn action_villa(&mut self, env: &mut Env<'_>) -> bool {
        if self.command_door() {
            self.init_open_demo();
            self.set_action(A_VILLA_TALK);
            self.demo_proc(env);
        } else if env.flags.test(EVENT_UNK_2D80) {
            self.set_action(A_WAIT);
        } else {
            self.set_event_prm(env);
        }
        true
    }

    fn action_password(&mut self, env: &mut Env<'_>) -> bool {
        if self.command_door() {
            self.init_open_demo();
            self.set_action(A_PASSWORD2);
            self.action_password2(env);
            self.pass_state = PASS_STEP_0;
        } else {
            self.set_event_prm(env);
        }
        true
    }

    fn action_figure(&mut self, env: &mut Env<'_>) -> bool {
        if self.command_door() {
            self.talk_partner_set = true;
            self.init_open_demo();
            self.set_action(A_FIGURE_TALK);
            self.demo_proc(env);
        } else if env.flags.test(EVENT_UNK_3401) {
            self.set_action(A_WAIT);
        } else {
            self.set_event_prm(env);
        }
        true
    }

    fn action_init(&mut self, env: &mut Env<'_>) -> bool {
        if !self.bgw {
            self.bgw = true;
        }
        if self.door_type == DOOR_FIGURE && !env.flags.test(EVENT_UNK_3401) {
            self.action_figure(env);
            self.set_action(A_FIGURE_WAIT);
            return true;
        }
        if self.door_type == DOOR_VILLA && !env.flags.test(EVENT_UNK_2D80) {
            self.action_villa(env);
            self.set_action(A_VILLA_WAIT);
            return true;
        }
        if self.door_type == DOOR_PASSWORD {
            if !env.flags.test(EVENT_UNK_1910) {
                self.set_action(A_PASSWORD);
                self.action_password(env);
                return true;
            }
            if check_get_item(env.equipped, env.collect, ITEM_BOMB_BAG) {
                self.set_action(A_ITEM);
                return true;
            }
        }
        if self.msg_door(env.night) && !self.chk_exception(env.collect, env.flags) {
            self.action_talk_wait(env);
            self.set_action(A_TALK_WAIT);
            return true;
        }
        self.action_wait(env);
        self.set_action(A_WAIT);
        true
    }

    fn action_dead(&mut self) -> bool {
        true
    }

    fn check_execute(&mut self, env: &Env<'_>) -> i32 {
        self.front_check = self.check_face(env.here, env.yaw, env.stay, env.ppos);
        if self.command_demo_accrpt() || self.command_door() {
            return 2;
        }
        if self.room_no2 != env.stay {
            return 0;
        }
        if self.front_check != FRONT_NONE && self.adjoin_player() {
            return 2;
        }
        0
    }

    fn draw_check_local(&mut self, env: &Env<'_>) {
        if !self.adjoin_player() {
            self.view_room = 0;
            return;
        }
        self.view_room = if self.front_room == ROOM_NONE || self.back_room == ROOM_NONE {
            env.stay as u8
        } else {
            self.view_room_no(env.ppos, env.here)
        };
        self.room_no = if self.front_check == FRONT_NONE {
            -1
        } else {
            env.stay
        };
    }

    fn execute(&mut self, env: &mut Env<'_>) {
        match self.check_execute(env) {
            0 => {
                self.set_action(A_INIT);
            }
            2 => {
                match self.action {
                    A_INIT => self.action_init(env),
                    A_WAIT => self.action_wait(env),
                    A_DEMO => self.action_demo(env),
                    A_TALK_WAIT => self.action_talk_wait(env),
                    A_TALK => self.action_talk(env),
                    A_PASSWORD => self.action_password(env),
                    A_PASSWORD2 => self.action_password2(env),
                    A_VILLA_WAIT => self.action_villa(env),
                    A_VILLA_TALK => self.action_talk(env),
                    A_FIGURE_WAIT => self.action_figure(env),
                    A_FIGURE_TALK => self.action_talk(env),
                    _ => self.action_dead(),
                };
            }
            1 => {
                self.start_demo_proc(env.pyaw);
                self.demo_proc(env);
            }
            _ => {}
        }
        self.room_no2 = env.stay;
        if self.type2() == 1 && self.bgw {
            self.bgw_move();
        }
        self.draw_check_local(env);
    }
}

struct Env<'a> {
    here: Vec3,
    yaw: i16,
    stay: i8,
    night: bool,
    ppos: Vec3,
    pyaw: i16,
    proom: i8,
    collect: &'a Collect,
    equipped: &'a Equipped,
    flags: &'a mut EventFlags,
    regs: &'a mut EventReg,
    names: &'a mut NameEntry,
    events: &'a mut SimEvents,
    list: &'a EventList,
    active: &'a mut ActiveEvent,
    procs: &'a mut MsgProcs,
    info: &'a mut MsgGameInfo,
    one: &'a mut Msg1Box,
    two: &'a mut Msg2Box,
    archive: Option<&'a MsgArchives>,
    clear_count: u8,
    staff_cut: Option<(usize, bool)>,
    player: Option<&'a mut Player>,
}

impl Env<'_> {
    fn edit_advance(&self) -> bool {
        self.active
            .staff_cut(self.list, STAFF_SHUTTER_DOOR)
            .is_some_and(|(_, advance)| advance)
    }

    fn cut_end(&mut self) {
        self.active
            .end_staff_cut(self.list, self.flags, STAFF_SHUTTER_DOOR);
    }

    fn se(&mut self, id: u32) {
        self.events.list.push(SimEvent::Se(id));
    }

    fn msg_live(&self) -> bool {
        self.procs.search_by_name(FPC_NM_MSG).is_some()
            || self.procs.search_by_name(FPC_NM_MSG2).is_some()
    }

    fn box_status(&self) -> u8 {
        if self.procs.search_by_name(FPC_NM_MSG).is_some() {
            self.one.status
        } else if self.procs.search_by_name(FPC_NM_MSG2).is_some() {
            self.two.status
        } else {
            STATUS_NONE
        }
    }

    fn set_box_status(&mut self, status: u8) {
        if self.procs.search_by_name(FPC_NM_MSG).is_some() {
            self.one.status = status;
        } else if self.procs.search_by_name(FPC_NM_MSG2).is_some() {
            self.two.status = status;
        }
    }

    fn message_set(&mut self, msg_id: i16, pos: Vec3) -> u32 {
        let msg_no = u32::from(msg_id as u16);
        match self.archive {
            Some(archive) => fop_msgm_message_set(
                self.procs,
                self.info,
                archive,
                self.clear_count,
                msg_no,
                None,
                pos,
            ),
            None => FPC_M_ERROR_PROCESS_ID,
        }
    }
}

#[derive(SystemParam)]
pub struct Read<'w> {
    stay: Res<'w, StayRoom>,
    clock: Res<'w, DayNight>,
    collect: Res<'w, Collect>,
    equipped: Res<'w, Equipped>,
    text: Res<'w, TextFlags>,
    list: Res<'w, EventList>,
    archives: Option<Res<'w, MsgArchives>>,
}

#[derive(SystemParam)]
pub struct MsgIo<'w> {
    procs: ResMut<'w, MsgProcs>,
    info: ResMut<'w, MsgGameInfo>,
    one: ResMut<'w, Msg1Box>,
    two: ResMut<'w, Msg2Box>,
}

pub fn control(
    read: Read,
    mut io: MsgIo,
    mut active: ResMut<ActiveEvent>,
    mut flags: ResMut<EventFlags>,
    mut regs: ResMut<EventReg>,
    mut names: ResMut<NameEntry>,
    mut events: ResMut<SimEvents>,
    mut players: Query<&mut Player>,
    prooms: Query<&Actor, (With<Player>, Without<Knob00>)>,
    mut doors: Query<(&mut Actor, &mut Knob00)>,
) {
    let night = read.clock.daynight() == TIME_NIGHT;
    let stay = read.stay.0;
    let clear_count = read.text.clear_count;
    let archive = read.archives.as_deref();
    for (actor, mut door) in &mut doors {
        let staff_cut = active.staff_cut(&read.list, STAFF_SHUTTER_DOOR);
        let mut player_handle = players.iter_mut().next();
        let ppos = player_handle.as_mut().map(|link| link.pos).unwrap_or(Vec3::ZERO);
        let pyaw = player_handle
            .as_mut()
            .map_or(0, |link| walk::to_s16(link.yaw));
        let mut env = Env {
            here: actor.pos,
            yaw: actor.angle[1],
            stay,
            night,
            ppos,
            pyaw,
            proom: prooms.iter().next().map_or(-1, |link| link.room),
            collect: &read.collect,
            equipped: &read.equipped,
            flags: &mut flags,
            regs: &mut regs,
            names: &mut names,
            events: &mut events,
            list: &read.list,
            active: &mut active,
            procs: &mut io.procs,
            info: &mut io.info,
            one: &mut io.one,
            two: &mut io.two,
            archive,
            clear_count,
            staff_cut,
            player: player_handle.as_mut().map(|link| &mut **link),
        };
        door.execute(&mut env);
    }
}