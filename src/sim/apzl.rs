use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::{Vec2, Vec3};

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, OrderFlags};
use super::intent::{DoIntent, MoveIntent, TargetIntent};
use super::npc_btsw::StControl;
use super::pickup::{ITEM_ACTORS, Pickup};
use super::player::Player;
use super::walk::{from_s16, to_s16};
use crate::assets::MsgArchives;
use crate::text::driver::TextFlags;
use crate::text::msgbox::Msg2Box;
use crate::text::msgbox1::Msg1Box;
use crate::text::msgproc::{
    FPC_M_ERROR_PROCESS_ID, FPC_NM_MSG, FPC_NM_MSG2, MsgGameInfo, MsgProcs,
    fop_msgm_message_set_actor, fop_msgm_message_set_look_at,
};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED, STATUS_MSG_DISPLAYED,
    STATUS_MSG_ENDS,
};

pub const APZL_ACTORS: [&str; 1] = ["Apzl"];

pub fn is_apzl(name: &str) -> bool {
    APZL_ACTORS.contains(&name)
}

const PARAM_TYPE_SHIFT: u32 = 0x8;
const PARAM_TYPE_MASK: u32 = 0xFF;

const STATE_IDLE: u8 = 0;
const STATE_TALK: u8 = 1;
const STATE_START: u8 = 2;
const STATE_PLAYING: u8 = 3;
const STATE_END: u8 = 4;
const STATE_RUPEE: u8 = 5;

const EVENT_IDX_COUNT: usize = 3;
const EVENT_TALK: usize = 0;
const EVENT_GAME: usize = 1;
const EVENT_RUPEE: usize = 2;
const EVENT_IDX_NONE: i16 = -1;

const ACT_WAIT: i8 = 0;
const ACT_TALK: i8 = 1;
const ACT_GAME: i8 = 2;
const ACT_GETITEM: i8 = 3;
const ACT_STOP: i8 = 4;
const ACT_SOUND: i8 = 5;
const ACT_PUSH_A: i8 = 6;
const ACT_EVENT_END: i8 = 7;

const CUT_NAMES: [&str; 8] = [
    "WAIT",
    "TALK",
    "GAME",
    "GETITEM",
    "STOP",
    "SOUND",
    "PUSH_A",
    "EVENT_END",
];

const STAFF_NAME: &str = "Apzl";
const EVENT_NAME_TALK: &str = "PUZZLE_TALK";
const EVENT_NAME_GAME: &str = "PUZZLE_GAME";
const EVENT_NAME_RUPEE: &str = "PUZZLE_RUPEE";

const MOVE_DIR_UP: u8 = 0;
const MOVE_DIR_LEFT: u8 = 1;
const MOVE_DIR_DOWN: u8 = 2;
const MOVE_DIR_RIGHT: u8 = 3;

const CELL_COUNT: usize = 16;
const ARROW_COUNT: usize = 4;
const ARROW_UP: usize = 0;
const ARROW_DOWN: usize = 2;
const ARROW_LEFT: usize = 1;
const ARROW_RIGHT: usize = 3;

const COL_MASK: u8 = 3;
const ROW_MASK: u8 = 0xC;
const COL_LAST: u8 = 3;
const ROW_LAST: u8 = 0xC;

const BLANK_SLOT: u8 = 3;

const CLEAR_MOVE_TIMER: u8 = 40;
const SLIDE_MOVE_TIMER: u8 = 5;

const REWARD_TIMER_FULL: i32 = 150;
const REWARD_TIMER_STEP: i32 = 60;
const REWARD_TIMER_GATE: i32 = 2;
const REWARD_GATE_MASK: i32 = 0;
const REWARD_BEAM: i32 = 6;
const RUPEE_MAX: usize = 30;
const PUZZLE_MAX: u8 = 0x10;

const SHUFFLE_ROUNDS: i32 = 10000;
const SHUFFLE_DIRECTIONS: f32 = 4.0;
const SHUFFLE_SOLVED_MAX: u8 = 3;
const SHUFFLE_FIX_PASSES: i32 = 3;

const STICK_WAIT_H: i16 = 0x3C;
const STICK_WAIT_V: i16 = 0x1E;
const STICK_STEP_H: i16 = 0;
const STICK_STEP_V: i16 = 0;
const STICK_ALIVE_ZONE: f32 = 0.9;
const STICK_DEAD_ZONE: f32 = 0.5;
const STICK_FIXED: i16 = 0;
const STICK_EPSILON: f32 = 3.8146973e-6;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const BOARD_TILE_STEP: f32 = 40.0;
const BOARD_TILE_ORIGIN: f32 = 60.0;
const BOARD_ARROW_LIFT: f32 = 15.0;
const BOARD_SCORE_STEP: f32 = 28.62;
const BOARD_SCORE_ORIGIN: f32 = 10.0;
const BOARD_SLIDE_RATE: f32 = 8.0;
const ARROW_SPIN: i16 = 0x4000;

const CULL_MIN: [f32; 3] = [-600.0, -600.0, -600.0];
const CULL_MAX: [f32; 3] = [600.0, 600.0, 600.0];

const MSG_NO_NONE: u32 = 0;
const MSG_QUIT: u32 = 0x1BC5;
const MSG_TALK: u32 = 0x1BC6;
const MSG_TALK_YES: u32 = 0x1BC7;
const MSG_TALK_QUIT: u32 = 0x1BC8;
const MSG_GAME_END: u32 = 0x1BC9;
const MSG_REWARD_YES: u32 = 0x1BCA;
const MSG_REWARD_NO: u32 = 0x1BCB;
const MSG_REWARD_AGAIN: u32 = 0x1BCC;
const MSG_GAME: u32 = 0x1BCD;
const MSG_GAME_QUIT: u32 = 0x1BCE;
const MSG_GAME_CLEARED: u32 = 0x1BCF;
const MSG_REWARD: u32 = 0x1BD0;

const SELECT_YES: u8 = 0;

const NO_MSG_STATUS: u8 = 0xFF;

const RUPEE_TABLE: [u8; RUPEE_MAX] = [
    0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x02, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x01, 0x00,
    0x01, 0x02, 0x01, 0x00, 0x01, 0x01, 0x01, 0x00, 0x01, 0x03, 0x01, 0x01, 0x01, 0x04,
];

const RUPEE_TYPE_GREEN: u8 = 0;
const RUPEE_TYPE_BLUE: u8 = 1;
const RUPEE_TYPE_YELLOW: u8 = 2;
const RUPEE_TYPE_RED: u8 = 3;

const ITEM_GREEN_RUPEE: u8 = 0x01;
const ITEM_BLUE_RUPEE: u8 = 0x02;
const ITEM_YELLOW_RUPEE: u8 = 0x03;
const ITEM_RED_RUPEE: u8 = 0x04;
const ITEM_PURPLE_RUPEE: u8 = 0x05;

const ITEM_BIT_NO: u8 = 0xFF;
const ITEM_SWITCH_NO: u8 = 0xFF;
const ITEM_TYPE_0: u32 = 0;
const ITEM_TYPE_MASK: u32 = 0x3;
const ITEM_ACT_A: u32 = 10;
const ITEM_ACT_MASK: u32 = 0x3F;
const ITEM_PRM_ACT_SHIFT: u32 = 26;

const RUPEE_SPAWN_POS: [f32; 3] = [0.0, 100.0, 735.0];
const RUPEE_ANGLE_SPREAD_X: f32 = 4000.0;
const RUPEE_ANGLE_SPREAD_Y: f32 = 6000.0;
const RUPEE_SPIN_OFFSET: i16 = 0x4000;
const RUPEE_FLY_MIN: f32 = 5.0;
const RUPEE_FLY_SPAN: f32 = 15.0;
const RUPEE_FALL: f32 = -2.1;
const RUPEE_REWARD_ROW: u8 = 15;

const DO_STATUS_BLANK: u8 = 0x00;
const DO_STATUS_NEXT: u8 = 0x19;
const DO_STATUS_CANCEL: u8 = 0x27;

const EVT_CND_CANTALK: u16 = 0x0001;

const ATTENTION_TALK: u8 = 0x28;
const ATTENTION_ACTION_SPEAK: u32 = 0x0000_0008;
const ATTENTION_TALKFLAG_CHECK: u32 = 0x2000_0000;

const SE_TALK_CURSOR: u32 = 0x083D;
const SE_PUZZLE_START: u32 = 0x08F2;
const SE_PUZZLE_EXIT: u32 = 0x08F3;
const SE_PUZZLE_COMPLETE: u32 = 0x08F4;

const SOUND_WAIT_FRAMES: i32 = 150;

const SHOCK_PATTERN: i32 = 4;
const SHOCK_FLAGS: i32 = 1;
const SHOCK_DIR: [f32; 3] = [0.0, 1.0, 0.0];

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Statics {
    pub puzzle_info: u8,
    pub puzzle_data: [u8; CELL_COUNT],
}

impl Default for Statics {
    fn default() -> Self {
        Self {
            puzzle_info: 0,
            puzzle_data: [0; CELL_COUNT],
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Apzl {
    pub puzzle_no: u8,
    pub kind: u8,
    pub blank_idx: u8,
    pub swapped_piece_idx: u8,
    pub move_timer: u8,
    pub move_direction: u8,
    pub show_blank_piece: bool,
    pub piece_pos: [u8; CELL_COUNT],
    pub state: u8,
    pub draw_arrow: [bool; ARROW_COUNT],
    pub event_idx: [i16; EVENT_IDX_COUNT],
    pub act_idx: i8,
    pub quit_game: bool,
    pub game_started: bool,
    pub game_cleared: bool,
    pub gave_reward: bool,
    pub shown_reward_message: bool,
    pub played_start_sound: bool,
    pub rupee_ids: [Entity; RUPEE_MAX],
    pub given_rupee_count: i32,
    pub msg_no: u32,
    pub curr_msg_id: u32,
    pub msg_bound: bool,
    pub reward_timer: i32,
    pub scoreboard_frame: [f32; CELL_COUNT],
    pub piece_local: [[f32; 3]; CELL_COUNT],
    pub arrow_local: [[f32; 3]; ARROW_COUNT],
    pub arrow_spin: [i16; ARROW_COUNT],
    pub scoreboard_world: [[f32; 3]; CELL_COUNT],
    pub attention_flags: u32,
    pub attention_pos: Vec3,
    pub talk_distance: f32,
    pub eye_pos: Vec3,
    pub do_status: u8,
    pub a_status: u8,
    pub shock: Option<(i32, i32, [f32; 3])>,
    pub sound_playing: bool,
    pub sound_timer: i32,
    pub prev_do: bool,
    pub stick: StControl,
    pub seed: [i32; 3],
    pub entered: bool,
}

impl Apzl {
    pub fn spawn(parameters: u32) -> Self {
        let kind = ((parameters >> PARAM_TYPE_SHIFT) & PARAM_TYPE_MASK) as u8;
        let mut stick = StControl {
            field_04: STICK_ALIVE_ZONE,
            field_08: STICK_DEAD_ZONE,
            field_0c: 0,
            field_0d: 0,
            field_0e: 0,
            field_10: 0,
            field_12: 0,
            field_14: 0,
            field_16: 0,
            field_18: 0,
            field_1a: 0,
            field_1c: 0,
            field_1e: 0,
            field_20: 0,
            field_22: 0,
            field_24: 0,
            field_26: 0,
        };
        stick.set_wait_parm(
            STICK_WAIT_H,
            STICK_WAIT_V,
            STICK_STEP_H,
            STICK_STEP_V,
            STICK_ALIVE_ZONE,
            STICK_DEAD_ZONE,
            STICK_FIXED,
            STICK_FIXED,
        );
        stick.init();
        Self {
            puzzle_no: 0,
            kind,
            blank_idx: BLANK_SLOT,
            swapped_piece_idx: 0,
            move_timer: 0,
            move_direction: 0,
            show_blank_piece: false,
            piece_pos: [0; CELL_COUNT],
            state: STATE_IDLE,
            draw_arrow: [false; ARROW_COUNT],
            event_idx: [EVENT_IDX_NONE; EVENT_IDX_COUNT],
            act_idx: -1,
            quit_game: false,
            game_started: false,
            game_cleared: false,
            gave_reward: false,
            shown_reward_message: false,
            played_start_sound: false,
            rupee_ids: [Entity::PLACEHOLDER; RUPEE_MAX],
            given_rupee_count: 0,
            msg_no: MSG_NO_NONE,
            curr_msg_id: FPC_M_ERROR_PROCESS_ID,
            msg_bound: false,
            reward_timer: 0,
            scoreboard_frame: [0.0; CELL_COUNT],
            piece_local: [[0.0; 3]; CELL_COUNT],
            arrow_local: [[0.0; 3]; ARROW_COUNT],
            arrow_spin: [0; ARROW_COUNT],
            scoreboard_world: [[0.0; 3]; CELL_COUNT],
            attention_flags: 0,
            attention_pos: Vec3::ZERO,
            talk_distance: f32::from(ATTENTION_TALK),
            eye_pos: Vec3::ZERO,
            do_status: 0,
            a_status: 0,
            shock: None,
            sound_playing: false,
            sound_timer: 0,
            prev_do: false,
            stick,
            seed: [RND_SEED; 3],
            entered: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    pub fn blank(&self) -> u8 {
        self.piece_pos
            .get(usize::from(self.blank_idx))
            .copied()
            .unwrap_or(0)
    }

    pub fn piece_transforms(&self) -> &[[f32; 3]; CELL_COUNT] {
        &self.piece_local
    }

    pub fn arrow_transforms(&self) -> &[[f32; 3]; ARROW_COUNT] {
        &self.arrow_local
    }

    pub fn arrow_angles(&self) -> &[i16; ARROW_COUNT] {
        &self.arrow_spin
    }

    pub fn scoreboard_transforms(&self) -> &[[f32; 3]; CELL_COUNT] {
        &self.scoreboard_world
    }

    fn rnd(&mut self) -> f32 {
        for lane in 0..3 {
            self.seed[lane] = (self.seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
        }
        let sum = (self.seed[0]) as f32 / RND_SCALE[0]
            + (self.seed[1]) as f32 / RND_SCALE[1]
            + (self.seed[2]) as f32 / RND_SCALE[2];
        (sum % 1.0).abs()
    }

    fn rnd_f(&mut self, range: f32) -> f32 {
        self.rnd() * range
    }

    fn rnd_fx(&mut self, range: f32) -> f32 {
        range * (self.rnd() - 0.5) * 2.0
    }

    pub fn search_piece(&self, piece: u8) -> u8 {
        for index in 0..CELL_COUNT {
            if self.piece_pos[index] == piece {
                return index as u8;
            }
        }
        0
    }

    pub fn swap_piece(&mut self, first: u8, second: u8) {
        let a = usize::from(first);
        let b = usize::from(second);
        let left = self.piece_pos.get(a).copied().unwrap_or(0);
        let right = self.piece_pos.get(b).copied().unwrap_or(0);
        if a < CELL_COUNT {
            self.piece_pos[a] = right;
        }
        if b < CELL_COUNT {
            self.piece_pos[b] = left;
        }
    }

    pub fn check_clear(&self) -> bool {
        for index in 0..CELL_COUNT {
            if u32::from(self.piece_pos[index]) != (index as u32 & 0xFF) {
                return false;
            }
        }
        true
    }

    pub fn check_arrow_draw(&mut self) {
        for index in 0..ARROW_COUNT {
            self.draw_arrow[index] = false;
        }
        if self.state != STATE_PLAYING {
            return;
        }
        if self.move_timer != 0 {
            return;
        }
        let blank = self.blank();
        if blank & COL_MASK != COL_LAST {
            self.draw_arrow[ARROW_LEFT] = true;
        }
        if blank & COL_MASK != 0 {
            self.draw_arrow[ARROW_RIGHT] = true;
        }
        if blank & ROW_MASK != ROW_LAST {
            self.draw_arrow[ARROW_UP] = true;
        }
        if blank & ROW_MASK != 0 {
            self.draw_arrow[ARROW_DOWN] = true;
        }
    }

    pub fn randamize_piece(&mut self) {
        for index in 0..CELL_COUNT {
            self.piece_pos[index] = index as u8;
        }
        loop {
            let mut unscrambled_count: u8 = 0;
            for _ in 0..SHUFFLE_ROUNDS {
                let temp = (self.rnd_f(SHUFFLE_DIRECTIONS) as i32 & 3) as u8;
                if temp == 1 {
                    if self.blank() & COL_MASK != COL_LAST {
                        let target = self.search_piece(self.blank().wrapping_add(1));
                        self.swap_piece(self.blank_idx, target);
                    }
                } else if temp == 3 {
                    if self.blank() & COL_MASK != 0 {
                        let target = self.search_piece(self.blank().wrapping_sub(1));
                        self.swap_piece(self.blank_idx, target);
                    }
                } else if temp == 0 {
                    if self.blank() & ROW_MASK != ROW_LAST {
                        let target = self.search_piece(self.blank().wrapping_add(4));
                        self.swap_piece(self.blank_idx, target);
                    }
                } else if self.blank() & ROW_MASK != 0 {
                    let target = self.search_piece(self.blank().wrapping_sub(4));
                    self.swap_piece(self.blank_idx, target);
                }
            }
            for index in 0..CELL_COUNT {
                if u32::from(self.piece_pos[index]) == (index as u32 & 0xFF) {
                    unscrambled_count += 1;
                }
            }
            if unscrambled_count < SHUFFLE_SOLVED_MAX {
                break;
            }
        }
        for _ in 0..SHUFFLE_FIX_PASSES {
            if self.blank() & COL_MASK != COL_LAST {
                let target = self.search_piece(self.blank().wrapping_add(1));
                self.swap_piece(self.blank_idx, target);
            }
            if self.blank() & ROW_MASK != 0 {
                let target = self.search_piece(self.blank().wrapping_sub(4));
                self.swap_piece(self.blank_idx, target);
            }
        }
    }

    pub fn save_piece(&self, statics: &mut Statics) {
        for index in 0..CELL_COUNT {
            statics.puzzle_data[index] = self.piece_pos[index];
        }
    }

    fn get_msg(&self) -> u32 {
        if self.shown_reward_message {
            return MSG_QUIT;
        }
        if self.gave_reward {
            return MSG_REWARD;
        }
        if self.game_cleared {
            return MSG_GAME_CLEARED;
        }
        if self.game_started {
            return MSG_GAME;
        }
        MSG_TALK
    }

    fn next_msg_status(&mut self, select: u8) -> u8 {
        let mut status = STATUS_MSG_CONTINUES;
        match self.msg_no {
            MSG_QUIT => {
                self.quit_game = true;
                status = STATUS_MSG_ENDS;
            }
            MSG_TALK => {
                self.msg_no = if select == SELECT_YES {
                    MSG_TALK_YES
                } else {
                    MSG_TALK_QUIT
                };
            }
            MSG_TALK_YES => {
                self.msg_no = if select == SELECT_YES {
                    MSG_REWARD_YES
                } else {
                    MSG_GAME_END
                };
            }
            MSG_TALK_QUIT => {
                self.quit_game = true;
                status = STATUS_MSG_ENDS;
            }
            MSG_GAME_END => {
                status = STATUS_MSG_ENDS;
            }
            MSG_REWARD_YES => {
                self.msg_no = MSG_REWARD_NO;
            }
            MSG_REWARD_NO => {
                self.msg_no = if select == SELECT_YES {
                    MSG_GAME_END
                } else {
                    MSG_REWARD_AGAIN
                };
            }
            MSG_REWARD_AGAIN => {
                self.msg_no = MSG_REWARD_YES;
            }
            MSG_GAME => {
                if select == SELECT_YES {
                    status = STATUS_MSG_ENDS;
                } else {
                    self.msg_no = MSG_GAME_QUIT;
                }
            }
            MSG_GAME_QUIT => {
                self.quit_game = true;
                status = STATUS_MSG_ENDS;
            }
            MSG_GAME_CLEARED => {
                status = STATUS_MSG_ENDS;
            }
            MSG_REWARD => {
                self.shown_reward_message = true;
                status = STATUS_MSG_ENDS;
            }
            _ => {
                self.quit_game = true;
                status = STATUS_MSG_ENDS;
            }
        }
        status
    }

    pub fn set_mtx(&mut self) {
        for index in 0..CELL_COUNT {
            let piece = i32::from(self.piece_pos[index]);
            let mut local = [
                (piece % 4) as f32 * BOARD_TILE_STEP - BOARD_TILE_ORIGIN,
                BOARD_TILE_ORIGIN - (piece / 4) as f32 * BOARD_TILE_STEP,
                0.0,
            ];
            if index as u8 == self.swapped_piece_idx && self.move_timer != 0 {
                let move_timer = if !self.check_clear() {
                    f32::from(self.move_timer) * BOARD_SLIDE_RATE
                } else {
                    f32::from(self.move_timer)
                };
                let direction = self.move_direction;
                if direction == MOVE_DIR_UP {
                    local[1] -= move_timer;
                } else if direction == MOVE_DIR_LEFT {
                    local[0] += move_timer;
                } else if direction == MOVE_DIR_DOWN {
                    local[1] += move_timer;
                } else {
                    local[0] -= move_timer;
                }
            }
            self.piece_local[index] = local;
        }

        let blank = i32::from(self.blank());
        for index in 0..ARROW_COUNT {
            let spin = (index as i16).wrapping_mul(ARROW_SPIN);
            let (sin, cos) = from_s16(spin).sin_cos();
            self.arrow_local[index] = [
                (blank % 4) as f32 * BOARD_TILE_STEP - BOARD_TILE_ORIGIN
                    + BOARD_ARROW_LIFT * sin,
                BOARD_TILE_ORIGIN - (blank / 4) as f32 * BOARD_TILE_STEP
                    - BOARD_ARROW_LIFT * cos,
                0.0,
            ];
            self.arrow_spin[index] = spin;
        }

        for index in 0..CELL_COUNT {
            self.scoreboard_world[index] = [
                0.0,
                -((index % 8) as f32 + 1.0) * BOARD_SCORE_STEP + BOARD_SCORE_ORIGIN,
                (index as f32 / 8.0 - 2.0) * BOARD_SCORE_STEP,
            ];
        }
    }

    pub fn create_init(&mut self, actor: &Actor, statics: &mut Statics, list: &EventList) {
        self.kind = ((actor.parameters >> PARAM_TYPE_SHIFT) & PARAM_TYPE_MASK) as u8;
        self.puzzle_no = statics.puzzle_info;
        if self.puzzle_no >= PUZZLE_MAX {
            self.puzzle_no = 0;
        }
        for index in 0..CELL_COUNT {
            if index as u8 >= self.puzzle_no {
                self.scoreboard_frame[index] = 0.0;
            } else {
                self.scoreboard_frame[index] = 1.0;
            }
        }

        self.blank_idx = BLANK_SLOT;
        self.show_blank_piece = false;

        if self.kind == 0 {
            self.randamize_piece();
        } else {
            for index in 0..CELL_COUNT {
                self.piece_pos[index] = index as u8;
            }
        }

        self.event_idx[EVENT_TALK] = event_index(list, EVENT_NAME_TALK);
        self.event_idx[EVENT_GAME] = event_index(list, EVENT_NAME_GAME);
        self.event_idx[EVENT_RUPEE] = event_index(list, EVENT_NAME_RUPEE);

        if self.kind == 0 {
            self.attention_flags |= ATTENTION_TALKFLAG_CHECK | ATTENTION_ACTION_SPEAK;
        }

        self.talk_distance = f32::from(ATTENTION_TALK);
        self.attention_pos = actor.pos;
        self.eye_pos = actor.pos;

        self.msg_bound = false;
        self.curr_msg_id = FPC_M_ERROR_PROCESS_ID;

        self.quit_game = false;
        self.game_started = false;
        self.game_cleared = false;
        self.gave_reward = false;
        self.shown_reward_message = false;

        self.swapped_piece_idx = 0;
        self.move_timer = 0;
        self.move_direction = 0;
        self.played_start_sound = false;
        self.state = STATE_IDLE;

        for index in 0..ARROW_COUNT {
            self.draw_arrow[index] = false;
        }

        self.given_rupee_count = 0;
        self.set_mtx();
    }

    fn stick_update(&mut self, axis: Vec2) {
        let value = axis.length().min(1.0);
        let angle = if cm3d_is_zero(axis.x) && cm3d_is_zero(axis.y) {
            0
        } else {
            to_s16(axis.x.atan2(axis.y))
        };
        self.stick.check_trigger(value, angle);
    }

    fn move_piece(&mut self) -> bool {
        let mut swapped = false;
        if self.move_timer != 0 {
            self.move_timer -= 1;
            return false;
        }

        if self.blank() & COL_MASK != COL_LAST && self.stick.check_left_trigger() {
            self.move_direction = MOVE_DIR_LEFT;
            self.swapped_piece_idx = self.search_piece(self.blank().wrapping_add(1));
            swapped = true;
        }

        if self.blank() & COL_MASK != 0 && self.stick.check_right_trigger() {
            self.move_direction = MOVE_DIR_RIGHT;
            self.swapped_piece_idx = self.search_piece(self.blank().wrapping_sub(1));
            swapped = true;
        }

        if self.blank() & ROW_MASK != ROW_LAST && self.stick.check_up_trigger() {
            self.move_direction = MOVE_DIR_UP;
            self.swapped_piece_idx = self.search_piece(self.blank().wrapping_add(4));
            swapped = true;
        }

        if self.blank() & ROW_MASK != 0 && self.stick.check_down_trigger() {
            self.move_direction = MOVE_DIR_DOWN;
            self.swapped_piece_idx = self.search_piece(self.blank().wrapping_sub(4));
            swapped = true;
        }

        if swapped {
            let blank = self.blank_idx;
            let target = self.swapped_piece_idx;
            self.swap_piece(blank, target);
            if self.check_clear() {
                self.move_timer = CLEAR_MOVE_TIMER;
            } else {
                self.move_timer = SLIDE_MOVE_TIMER;
            }
        }
        swapped
    }

    fn sound_tick(&mut self) {
        if self.sound_playing {
            self.sound_timer -= 1;
            if self.sound_timer <= 0 {
                self.sound_playing = false;
                self.sound_timer = 0;
            }
        }
    }

    fn rupee_item(&self) -> u8 {
        if self.puzzle_no == RUPEE_REWARD_ROW {
            let index = usize::try_from(self.given_rupee_count).unwrap_or(RUPEE_MAX);
            let rupee_type = RUPEE_TABLE.get(index).copied().unwrap_or(0);
            if rupee_type == RUPEE_TYPE_GREEN {
                ITEM_GREEN_RUPEE
            } else if rupee_type == RUPEE_TYPE_BLUE {
                ITEM_BLUE_RUPEE
            } else if rupee_type == RUPEE_TYPE_YELLOW {
                ITEM_YELLOW_RUPEE
            } else if rupee_type == RUPEE_TYPE_RED {
                ITEM_RED_RUPEE
            } else {
                ITEM_PURPLE_RUPEE
            }
        } else if self.given_rupee_count.rem_euclid(REWARD_BEAM) == 0 {
            ITEM_BLUE_RUPEE
        } else {
            ITEM_GREEN_RUPEE
        }
    }
}

fn item_param(item: u8) -> u32 {
    u32::from(item)
        | (u32::from(ITEM_BIT_NO) << 8)
        | (u32::from(ITEM_SWITCH_NO) << 16)
        | ((ITEM_TYPE_0 & ITEM_TYPE_MASK) << 24)
        | ((ITEM_ACT_A & ITEM_ACT_MASK) << ITEM_PRM_ACT_SHIFT)
}

fn event_index(list: &EventList, name: &str) -> i16 {
    match list.event_index(name) {
        Some(slot) => i16::try_from(slot).unwrap_or(EVENT_IDX_NONE),
        None => EVENT_IDX_NONE,
    }
}

fn act_index(name: &str) -> i8 {
    match CUT_NAMES.iter().position(|entry| *entry == name) {
        Some(index) => index as i8,
        None => -1,
    }
}

fn in_talk_range(distance: f32, from: Vec3, to: Vec3) -> bool {
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    dx * dx + dz * dz < distance * distance
}

fn cm3d_is_zero(value: f32) -> bool {
    value.abs() < STICK_EPSILON
}

struct Rupee {
    owner: Entity,
    slot: usize,
    parameters: u32,
    pos: [f32; 3],
    angle: [i16; 3],
    room: i8,
    set_id: u16,
}

struct Env<'a> {
    entity: Entity,
    actor_angle: [i16; 3],
    room: i8,
    set_id: u16,
    stick: Vec2,
    hold_a: bool,
    hold_b: bool,
    talk_command: bool,
    clear_count: u8,
    list: &'a EventList,
    active: &'a mut ActiveEvent,
    ev_flags: &'a mut EventFlags,
    events: &'a mut SimEvents,
    procs: &'a mut MsgProcs,
    info: &'a mut MsgGameInfo,
    one: &'a mut Msg1Box,
    two: &'a mut Msg2Box,
    archive: Option<&'a MsgArchives>,
}

impl Env<'_> {
    fn box_live(&self) -> bool {
        self.procs.search_by_name(FPC_NM_MSG).is_some()
            || self.procs.search_by_name(FPC_NM_MSG2).is_some()
    }

    fn box_status(&self) -> u8 {
        if self.procs.search_by_name(FPC_NM_MSG).is_some() {
            self.one.status
        } else if self.procs.search_by_name(FPC_NM_MSG2).is_some() {
            self.two.status
        } else {
            NO_MSG_STATUS
        }
    }

    fn set_box_status(&mut self, status: u8) {
        if self.procs.search_by_name(FPC_NM_MSG).is_some() {
            self.one.status = status;
        } else if self.procs.search_by_name(FPC_NM_MSG2).is_some() {
            self.two.status = status;
        }
    }

    fn select_num(&self) -> u8 {
        self.one.select_num
    }

    fn se(&mut self, id: u32) {
        self.events.list.push(SimEvent::Se(id));
    }
}

impl Apzl {
    fn talk(&mut self, env: &mut Env<'_>, param: i32) -> u8 {
        let mut status = NO_MSG_STATUS;
        if self.curr_msg_id == FPC_M_ERROR_PROCESS_ID {
            if param == 1 {
                self.msg_no = self.get_msg();
            }
            self.curr_msg_id = match env.archive {
                Some(archive) => fop_msgm_message_set_actor(
                    env.procs,
                    env.info,
                    archive,
                    env.clear_count,
                    self.msg_no,
                    None,
                    self.eye_pos,
                ),
                None => FPC_M_ERROR_PROCESS_ID,
            };
            self.msg_bound = false;
        } else if self.msg_bound {
            status = env.box_status();
            if status == STATUS_MSG_DISPLAYED {
                let select = env.select_num();
                let next = self.next_msg_status(select);
                env.set_box_status(next);
                if next == STATUS_MSG_CONTINUES {
                    if let Some(archive) = env.archive {
                        let _ = fop_msgm_message_set_look_at(
                            env.procs,
                            env.info,
                            archive,
                            env.clear_count,
                            self.msg_no,
                            Vec3::ZERO,
                        );
                    }
                }
            } else if status == STATUS_BOX_CLOSED {
                env.set_box_status(STATUS_MSG_DESTROYED);
                self.curr_msg_id = FPC_M_ERROR_PROCESS_ID;
            }
        } else {
            self.msg_bound = env.box_live();
        }
        status
    }

    fn private_cut(&mut self, env: &mut Env<'_>, stops: &mut Vec<Entity>, spawns: &mut Vec<Rupee>) {
        let list = env.list;
        let Some((cut, advance)) = env.active.staff_cut(list, STAFF_NAME) else {
            return;
        };
        let name = list.cut_name(cut).unwrap_or("");
        self.act_idx = act_index(name);
        if self.act_idx < 0 {
            env.active.end_staff_cut(list, env.ev_flags, STAFF_NAME);
            return;
        }
        if advance {
            match self.act_idx {
                ACT_WAIT | ACT_TALK | ACT_STOP | ACT_PUSH_A => {}
                ACT_GAME => self.game_started = true,
                ACT_GETITEM => {
                    self.gave_reward = true;
                    self.reward_timer = REWARD_TIMER_FULL;
                }
                ACT_SOUND => {
                    env.se(SE_PUZZLE_COMPLETE);
                    self.sound_playing = true;
                    self.sound_timer = SOUND_WAIT_FRAMES;
                }
                ACT_EVENT_END => {
                    let count =
                        usize::try_from(self.given_rupee_count).unwrap_or(RUPEE_MAX).min(RUPEE_MAX);
                    for slot in 0..count {
                        let entity = self
                            .rupee_ids
                            .get(slot)
                            .copied()
                            .unwrap_or(Entity::PLACEHOLDER);
                        if entity != Entity::PLACEHOLDER {
                            stops.push(entity);
                        }
                    }
                }
                _ => {}
            }
        }
        let mut temp = false;
        match self.act_idx {
            ACT_WAIT => temp = true,
            ACT_TALK => {
                if self.talk(env, 1) == STATUS_BOX_CLOSED {
                    temp = true;
                }
            }
            ACT_GAME => temp = true,
            ACT_GETITEM => self.cut_getitem(env, spawns),
            ACT_STOP => {}
            ACT_SOUND => {
                self.do_status = DO_STATUS_BLANK;
                self.a_status = DO_STATUS_BLANK;
                if !self.sound_playing {
                    temp = true;
                }
            }
            ACT_PUSH_A => {
                self.do_status = DO_STATUS_NEXT;
                self.a_status = DO_STATUS_BLANK;
                if env.hold_a {
                    temp = true;
                }
            }
            ACT_EVENT_END => temp = true,
            _ => temp = true,
        }
        if temp {
            env.active.end_staff_cut(list, env.ev_flags, STAFF_NAME);
        }
    }

    fn cut_getitem(&mut self, env: &Env<'_>, spawns: &mut Vec<Rupee>) {
        self.do_status = DO_STATUS_BLANK;
        self.a_status = DO_STATUS_BLANK;
        if self.reward_timer == 0 {
            return;
        }
        self.reward_timer -= 1;
        if (self.reward_timer & REWARD_TIMER_GATE) != REWARD_GATE_MASK {
            return;
        }
        if self.given_rupee_count >= RUPEE_MAX as i32 {
            return;
        }

        let item = self.rupee_item();
        let mut angle = [0i16; 3];
        angle[0] = env.actor_angle[0].wrapping_add(self.rnd_fx(RUPEE_ANGLE_SPREAD_X) as i16);
        angle[1] = (self.rnd_fx(RUPEE_ANGLE_SPREAD_Y) as i16)
            .wrapping_add(env.actor_angle[1])
            .wrapping_sub(RUPEE_SPIN_OFFSET);
        angle[2] = ITEM_SWITCH_NO as i16;

        let _ = (
            self.rnd_f(RUPEE_FLY_SPAN) + RUPEE_FLY_MIN,
            self.rnd_f(RUPEE_FLY_SPAN) + RUPEE_FLY_MIN,
            RUPEE_FALL,
        );

        let slot = usize::try_from(self.given_rupee_count).unwrap_or(RUPEE_MAX);
        spawns.push(Rupee {
            owner: env.entity,
            slot,
            parameters: item_param(item),
            pos: RUPEE_SPAWN_POS,
            angle,
            room: env.room,
            set_id: env.set_id,
        });

        self.given_rupee_count += 1;
        self.gave_reward = true;
        self.reward_timer = REWARD_TIMER_STEP;
    }

    fn order_slot(&self, env: &mut Env<'_>, which: usize) {
        let list = env.list;
        let index = self.event_idx.get(which).copied().unwrap_or(EVENT_IDX_NONE);
        if index >= 0 {
            let slot = usize::from(index as u16);
            let _ = env.active.order(list, slot, env.entity);
        }
    }

    fn end_slot(&self, env: &Env<'_>, which: usize) -> bool {
        let index = self.event_idx.get(which).copied().unwrap_or(EVENT_IDX_NONE);
        index >= 0 && env.active.end_check(usize::from(index as u16))
    }

    fn end_all(&self, env: &mut Env<'_>) {
        for which in 0..EVENT_IDX_COUNT {
            let index = self.event_idx[which];
            if index >= 0 {
                let _ = env.active.end(usize::from(index as u16));
            }
        }
        *env.ev_flags = EventFlags::default();
    }

    fn execute(
        &mut self,
        env: &mut Env<'_>,
        statics: &mut Statics,
        order_flags: &mut OrderFlags,
        stops: &mut Vec<Entity>,
        spawns: &mut Vec<Rupee>,
    ) {
        self.sound_tick();
        self.stick_update(env.stick);

        if self.kind == 0 {
            let mut state = self.state;
            let mut fall = true;
            while fall {
                fall = false;
                match state {
                    STATE_IDLE => {
                        order_flags.set(EVT_CND_CANTALK);
                        if env.talk_command {
                            self.state = STATE_TALK;
                        }
                    }
                    STATE_TALK => {
                        self.private_cut(env, stops, spawns);
                        if self.end_slot(env, EVENT_TALK) {
                            if self.quit_game {
                                self.end_all(env);
                                self.state = STATE_IDLE;
                                self.quit_game = false;
                                self.game_started = false;
                                self.save_piece(statics);
                            } else {
                                self.state = STATE_START;
                            }
                        }
                    }
                    STATE_START => {
                        if !self.played_start_sound {
                            env.se(SE_PUZZLE_START);
                            self.played_start_sound = true;
                        }
                        self.order_slot(env, EVENT_GAME);
                        self.state = STATE_PLAYING;
                    }
                    STATE_PLAYING => {
                        self.private_cut(env, stops, spawns);
                        if self.move_piece() {
                            env.se(SE_TALK_CURSOR);
                        }
                        self.do_status = DO_STATUS_BLANK;
                        self.a_status = DO_STATUS_CANCEL;

                        if self.check_clear() && self.move_timer == 0 {
                            self.show_blank_piece = true;
                            self.game_cleared = true;
                            self.shock = Some((SHOCK_PATTERN, SHOCK_FLAGS, SHOCK_DIR));
                            self.state = STATE_END;
                        } else if env.hold_b {
                            env.se(SE_PUZZLE_EXIT);
                            self.state = STATE_END;
                        }
                    }
                    STATE_END => {
                        self.quit_game = false;
                        if self.game_cleared {
                            let mut clear_count = statics.puzzle_info;
                            clear_count += 1;
                            if clear_count >= PUZZLE_MAX {
                                clear_count = 0;
                            }
                            let slot = usize::from(self.puzzle_no);
                            if slot < CELL_COUNT {
                                self.scoreboard_frame[slot] = 1.0;
                            }
                            statics.puzzle_info = clear_count;

                            self.order_slot(env, EVENT_RUPEE);
                            self.state = STATE_RUPEE;
                        } else {
                            self.order_slot(env, EVENT_TALK);
                            self.state = STATE_TALK;
                        }
                        state = STATE_RUPEE;
                        fall = true;
                    }
                    STATE_RUPEE => {
                        self.private_cut(env, stops, spawns);
                        if self.end_slot(env, EVENT_RUPEE) {
                            self.end_all(env);
                            self.state = STATE_IDLE;
                            self.quit_game = false;
                            self.game_started = false;
                        }
                    }
                    _ => {}
                }
            }
            self.check_arrow_draw();
        }

        self.set_mtx();
    }
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    list: Res<'w, EventList>,
    flags: Res<'w, TextFlags>,
    intent: Res<'w, MoveIntent>,
    do_intent: Res<'w, DoIntent>,
    target: Res<'w, TargetIntent>,
    archives: Option<Res<'w, MsgArchives>>,
    players: Query<'w, 's, &'static Player>,
}

pub fn control(
    watch: Watch<'_, '_>,
    mut statics: ResMut<Statics>,
    mut active: ResMut<ActiveEvent>,
    mut ev_flags: ResMut<EventFlags>,
    mut order_flags: ResMut<OrderFlags>,
    mut events: ResMut<SimEvents>,
    mut procs: ResMut<MsgProcs>,
    mut info: ResMut<MsgGameInfo>,
    mut one: ResMut<Msg1Box>,
    mut two: ResMut<Msg2Box>,
    mut commands: Commands,
    mut boards: Query<(Entity, &Actor, &mut Apzl), (With<Apzl>, Without<Player>)>,
    mut rupees: Query<&mut Actor, (With<Pickup>, Without<Apzl>)>,
) {
    let player = watch.players.iter().next();
    let archive = watch.archives.as_deref();
    let clear_count = watch.flags.clear_count;
    let axis = watch.intent.axis;
    let hold_a = watch.do_intent.0;
    let hold_b = watch.target.0;

    let mut stops: Vec<Entity> = Vec::new();
    let mut spawns: Vec<Rupee> = Vec::new();

    for (_, actor, mut board) in &mut boards {
        if board.entered {
            continue;
        }
        let mut fresh = Apzl::spawn(actor.parameters);
        fresh.create_init(actor, &mut statics, &watch.list);
        fresh.entered = true;
        *board = fresh;
    }

    for (entity, actor, board) in &mut boards {
        let angle = actor.angle;
        let room = actor.room;
        let set_id = actor.set_id;
        let pos = board.attention_pos;
        let range = board.talk_distance;
        let tap = hold_a && !board.prev_do;
        board.prev_do = hold_a;
        let mut env = Env {
            entity,
            actor_angle: angle,
            room,
            set_id,
            stick: axis,
            hold_a,
            hold_b,
            talk_command: tap && player.is_some_and(|link| in_talk_range(range, pos, link.pos)),
            clear_count,
            list: &watch.list,
            active: &mut active,
            ev_flags: &mut ev_flags,
            events: &mut events,
            procs: &mut procs,
            info: &mut info,
            one: &mut one,
            two: &mut two,
            archive,
        };
        board.execute(
            &mut env,
            &mut statics,
            &mut order_flags,
            &mut stops,
            &mut spawns,
        );
    }

    let mut born: Vec<(Entity, usize, Entity)> = Vec::new();
    for rupee in spawns {
        let parameters = rupee.parameters;
        let item = commands
            .spawn((
                Actor {
                    name: ITEM_ACTORS[0].to_owned(),
                    parameters,
                    pos: Vec3::from_array(rupee.pos),
                    angle: rupee.angle,
                    set_id: rupee.set_id,
                    room: rupee.room,
                    ..Default::default()
                },
                Pickup::spawn(parameters),
            ))
            .id();
        born.push((rupee.owner, rupee.slot, item));
    }

    for (owner, slot, item) in born {
        if let Some(mut entry) = boards.iter_mut().find(|(entity, _, _)| *entity == owner) {
            if slot < RUPEE_MAX {
                entry.2.rupee_ids[slot] = item;
            }
        }
    }

    for entity in stops {
        if let Ok(mut actor) = rupees.get_mut(entity) {
            actor.angle = [0; 3];
        }
    }
}