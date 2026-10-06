use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;

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

const EVENT_TALK: usize = 0;
const EVENT_GAME: usize = 1;
const EVENT_RUPEE: usize = 2;

const ACT_WAIT: i8 = 0;
const ACT_TALK: i8 = 1;
const ACT_GAME: i8 = 2;
const ACT_GETITEM: i8 = 3;
const ACT_STOP: i8 = 4;
const ACT_SOUND: i8 = 5;
const ACT_PUSH_A: i8 = 6;
const ACT_EVENT_END: i8 = 7;

const MOVE_DIR_UP: u8 = 0;
const MOVE_DIR_LEFT: u8 = 1;
const MOVE_DIR_DOWN: u8 = 2;

const CLEAR_MOVE_TIMER: u8 = 40;
const SLIDE_MOVE_TIMER: u8 = 5;
const REWARD_TIMER: i32 = 150;
const REWARD_TIMER_STEP: i32 = 60;
const RUPEE_MAX: usize = 30;
const PUZZLE_MAX: u8 = 0x10;

const SHUFFLE_ROUNDS: i32 = 10000;
const SHUFFLE_SOLVED_MAX: u8 = 3;
const SHUFFLE_FIX_PASSES: i32 = 3;

const MSG_TALK: u32 = 0x1BC6;
const MSG_GAME_END: u32 = 0x1BCD;
const MSG_GAME_CLEARED: u32 = 0x1BCF;
const MSG_REWARD: u32 = 0x1BD0;
const MSG_QUIT: u32 = 0x1BC5;

const CULL_MIN: [f32; 3] = [-600.0, -600.0, -600.0];
const CULL_MAX: [f32; 3] = [600.0, 600.0, 600.0];

const CELL_COUNT: usize = 16;
const ARROW_COUNT: usize = 4;

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

const CUT_NAME_NONE: &str = "Apzl";
const EVENT_NAME_TALK: &str = "PUZZLE_TALK";
const EVENT_NAME_GAME: &str = "PUZZLE_GAME";
const EVENT_NAME_RUPEE: &str = "PUZZLE_RUPEE";

const STICK_LIMIT_X: i16 = 0x3C;
const STICK_LIMIT_Y: i16 = 0x1E;
const STICK_ALIVE_LIMIT: f32 = 0.9;
const STICK_DEAD_LIMIT: f32 = 0.5;

const BOARD_TILE_STEP: f32 = 40.0;
const BOARD_TILE_ORIGIN: f32 = 60.0;
const BOARD_TILE_LIFT: f32 = 60.0;
const BOARD_ARROW_LIFT: f32 = 15.0;
const BOARD_SCORE_STEP: f32 = 28.62;
const BOARD_SCORE_ORIGIN: f32 = 10.0;
const BOARD_SLIDE_RATE: f32 = 8.0;
const ARROW_SPIN: i16 = 0x4000;

const RUPEE_SPAWN_X: f32 = 0.0;
const RUPEE_SPAWN_Y: f32 = 100.0;
const RUPEE_SPAWN_Z: f32 = 735.0;
const RUPEE_SPREAD_X: f32 = 4000.0;
const RUPEE_SPREAD_Y: f32 = 6000.0;
const RUPEE_SPIN_BASE: i16 = -0x4000;
const RUPEE_SPAWN_Z_ACCEL: f32 = -2.1;

const ITEM_GREEN_RUPEE: u32 = 0;
const ITEM_BLUE_RUPEE: u32 = 1;
const ITEM_YELLOW_RUPEE: u32 = 2;
const ITEM_RED_RUPEE: u32 = 3;
const ITEM_PURPLE_RUPEE: u32 = 4;

const ITEM_STATUS_UNK4000: u32 = 0x4000;
const ITEM_LIFETIME: i32 = 450;

const ACT_STTS_BLANK: u8 = 0;
const ACT_STTS_NEXT: u8 = 1;
const ACT_STTS_CANCEL: u8 = 2;

const STAFF_IDX_NONE: i32 = -1;
const EVENT_IDX_NONE: i16 = -1;

const EPM_ERROR_PROCESS_ID: u32 = 0xFFFF_FFFF;
const EPM_CONTINUES: u16 = 0;
const EPM_ENDS: u16 = 1;
const EPM_DESTROYED: u16 = 0x12;

const SHOCK_POWER: u32 = 4;
const SHOCK_TICKS: u32 = 1;

const STTS_NOCULLEXEC: u32 = 0x0000_0008;
const STTS_CULL: u32 = 0x0000_0100;
const STTS_UNK40000: u32 = 0x0004_0000;

#[derive(Resource, Debug, Clone, Copy)]
pub struct Statics {
    pub puzzle_info: u8,
    pub puzzle_data: [u8; CELL_COUNT],
    pub event_reset: bool,
}

impl Default for Statics {
    fn default() -> Self {
        Self {
            puzzle_info: 0,
            puzzle_data: [0; CELL_COUNT],
            event_reset: false,
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
    pub event_idx: [i16; 3],
    pub act_idx: i8,
    pub quit_game: bool,
    pub game_started: bool,
    pub game_cleared: bool,
    pub gave_reward: bool,
    pub shown_reward_message: bool,
    pub played_start_sound: bool,
    pub rupee_ids: [u32; RUPEE_MAX],
    pub given_rupee_count: i32,
    pub msg_no: u32,
    pub curr_msg_id: i32,
    pub reward_timer: i32,
    pub scoreboard_frame: [f32; CELL_COUNT],
    pub seed: [i32; 3],
    pub entered: bool,
    pub doomed: bool,
}

impl Apzl {
    pub fn spawn(parameters: u32) -> Self {
        let kind = ((parameters >> PARAM_TYPE_SHIFT) & PARAM_TYPE_MASK) as u8;
        Self {
            puzzle_no: 0,
            kind,
            blank_idx: 3,
            swapped_piece_idx: 0,
            move_timer: 0,
            move_direction: 0,
            show_blank_piece: false,
            piece_pos: [0; CELL_COUNT],
            state: STATE_IDLE,
            draw_arrow: [false; ARROW_COUNT],
            event_idx: [EVENT_IDX_NONE; 3],
            act_idx: -1,
            quit_game: false,
            game_started: false,
            game_cleared: false,
            gave_reward: false,
            shown_reward_message: false,
            played_start_sound: false,
            rupee_ids: [0; RUPEE_MAX],
            given_rupee_count: 0,
            msg_no: 0,
            curr_msg_id: -1,
            reward_timer: 0,
            scoreboard_frame: [0.0; CELL_COUNT],
            seed: [100; 3],
            entered: false,
            doomed: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }
}