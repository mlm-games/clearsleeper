//! `daNpc_P2::Act_c` (`tww/src/d/actor/d_a_npc_p2.cpp`), the pirates the Outset raid brings
//! ashore: `P2a` at the telescope on the deck (`d_a_obj_pirateship.cpp:59-134`, rows 0 and 8),
//! `P2b` on Windfall's goal course, and `P2c`, the mocco-riding lookout (row 1, and the single
//! child `pirateCreate` spawns the tick the kidnapping bit rises, `:431-436`).
//!
//! Ported: `getArg` (`:1472-1484`), both action functions and the whole `m7D6` switch
//! (`:1122-1349`), `checkOrder`/`eventOrder` (`:720-755`), `talk`/`getMsg`/`next_msgStatus`/
//! `anmAtr` (`:524-719`), `setAnm`/`setTexAnm` (`:240-347`), `chkAttention`/`lookBack`
//! (`:362-472`) and `createInit` (`:1567-1665`).
//!
//! The private cut table (`d_a_npc_p2_cut.inc`) is ported below, with the two nearest-actor
//! searches (`searchNearLift`/`searchNearRope`) and the rope swing. `mAcch` is the one stand-in:
//! the port gives actors no collision stage of their own, so a cut tests the stage's ground
//! through `Collision` (`CUT_GROUND_EPS`) and keeps last tick's contact on `Cut::grounded` to tell
//! a landing from a hit, rather than reading `dBgS_Acch`'s own flags.
//!
//! Not ported: the model and its resources, the joint callback that produces the eye and
//! attention points (`:184-213`), the smoke emitter (`:495-518`), the head-joint matrix
//! `cutRopeTalkProc` reads (`:6B08`), `dKy_instant_timechg`, the `Obj_Timer` stop calls and the
//! player voice `cutSurpriseStart` starts - the port has none of those. The arc holds the BCK
//! frame counts, so `ONESHOT_FRAMES` stands in for them exactly the way `obj_ekskz` stands in for
//! its BRK.

use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::event_cut::{self, EventCut};
use super::flags::{StoryBits, Switches};
use super::intent::DoIntent;
use super::npc_p1::{ChildHio, Jnt, Msg, Text};
use super::player::Player;
use super::stream::add_calc_angle_s2;
use super::walk;
use crate::text::msgproc::{DA_EVT_MODE_NONE, FPC_M_ERROR_PROCESS_ID};
use crate::text::{
    STATUS_BOX_CLOSED, STATUS_BOX_OPENING, STATUS_MSG_CONTINUES, STATUS_MSG_DESTROYED,
    STATUS_MSG_DISPLAYED, STATUS_MSG_ENDS, STATUS_TYPING,
};

pub const NPC_P2_ACTORS: [&str; 3] = ["P2a", "P2b", "P2c"];

pub fn is_npc_p2(name: &str) -> bool {
    NPC_P2_ACTORS.contains(&name)
}

pub const TYPE_P2A: u8 = 0;
pub const TYPE_P2B: u8 = 1;
pub const TYPE_P2C: u8 = 2;
pub const TYPE_COUNT: usize = 3;

pub const PARAM_TYPE_MASK: u32 = 3;
pub const PARAM_SUB_SHIFT: u32 = 2;
pub const PARAM_BYTE_MASK: u32 = 0xFF;
pub const PARAM_SWITCH_SHIFT: u32 = 10;

/// `dSv_event_flag_c::UNK_*`, the save bits `getMsg`/`createInit`/the state machine read. As in
/// `npc_p1`, the enum name is the bit number.
pub const EV_0701: u16 = 0x0701;
pub const EV_0702: u16 = 0x0702;
pub const EV_0704: u16 = 0x0704;
pub const EV_0710: u16 = 0x0710;
pub const EV_0720: u16 = 0x0720;
pub const EV_0808: u16 = 0x0808;
pub const EV_0940: u16 = 0x0940;
pub const EV_0F02: u16 = 0x0F02;
pub const EV_1502: u16 = 0x1502;
pub const EV_1A04: u16 = 0x1A04;

pub const MSG_C96: u32 = 0xC96;
pub const MSG_C97: u32 = 0xC97;
pub const MSG_C98: u32 = 0xC98;
pub const MSG_100E: u32 = 0x100E;
pub const MSG_100F: u32 = 0x100F;
pub const MSG_1010: u32 = 0x1010;
pub const MSG_1013: u32 = 0x1013;
pub const MSG_1018: u32 = 0x1018;
pub const MSG_1019: u32 = 0x1019;
pub const MSG_1020: u32 = 0x1020;
pub const MSG_1021: u32 = 0x1021;
pub const MSG_1022: u32 = 0x1022;
pub const MSG_1023: u32 = 0x1023;
pub const MSG_1024: u32 = 0x1024;
pub const MSG_1025: u32 = 0x1025;
pub const MSG_1026: u32 = 0x1026;
pub const MSG_1027: u32 = 0x1027;
pub const MSG_1028: u32 = 0x1028;
pub const MSG_1029: u32 = 0x1029;
pub const MSG_102A: u32 = 0x102A;
pub const MSG_102C: u32 = 0x102C;
pub const MSG_102D: u32 = 0x102D;
pub const MSG_102F: u32 = 0x102F;
pub const MSG_1030: u32 = 0x1030;
pub const MSG_1031: u32 = 0x1031;
pub const MSG_1032: u32 = 0x1032;
pub const MSG_1B35: u32 = 0x1B35;
pub const MSG_1B36: u32 = 0x1B36;

/// `m7D6`, the state word the action functions switch on (`:1143-1211`, `:1266-1290`). The
/// numbers are the reference's own; `STATE_*` names them so the switch reads.
pub const STATE_WAIT: i8 = 1;
pub const STATE_TALK: i8 = 2;
pub const STATE_DEMO_WAIT: i8 = 3;
pub const STATE_DEMO_INTRO: i8 = 4;
pub const STATE_DEMO_LIFT: i8 = 5;
pub const STATE_DEMO_JUMP: i8 = 6;
pub const STATE_GOAL_TALKPOS_WAIT: i8 = 7;
pub const STATE_GOAL_TALKPOS_TALK: i8 = 8;
pub const STATE_GOALPOS_TO_TALKPOS: i8 = 9;
pub const STATE_TALKPOS_TO_GOALPOS: i8 = 0x0A;
pub const STATE_GOALPOS_WAIT: i8 = 0x0B;
pub const STATE_DEMO_GOAL: i8 = 0x0C;
pub const STATE_TREASURE_WAIT: i8 = 0x0D;
pub const STATE_TREASURE_WAIT_TALK: i8 = 0x0E;
pub const STATE_DEMO_ARRIVE: i8 = 0x0F;
pub const STATE_MOCCO_WAIT: i8 = 0x10;
pub const STATE_ZUKO_TELESCOPE: i8 = 0x11;
pub const STATE_DEMO_WAIT_2: i8 = 0x12;
pub const STATE_DEMO_INTRO_2: i8 = 0x13;
pub const STATE_GOAL_WAIT_2: i8 = 0x14;
pub const STATE_DEMO_GOAL_2: i8 = 0x15;
pub const STATE_DEMO_BOMB_GET: i8 = 0x16;

/// `m7D5`, the event-order code (`:720-742`). `1`/`2` are the talk orders, `3..=10` index
/// `DEMO_NAMES` from zero.
pub const ORDER_NONE: u8 = 0;
pub const ORDER_SPEAK: u8 = 1;
pub const ORDER_TALK: u8 = 2;
pub const ORDER_FIRST_DEMO: u8 = 3;
pub const ORDER_INTRO: u8 = 3;
pub const ORDER_TO_GOAL: u8 = 4;
pub const ORDER_GOAL_WAIT_TALK: u8 = 5;
pub const ORDER_GOAL: u8 = 6;
pub const ORDER_ARRIVE_MAJYU: u8 = 7;
pub const ORDER_INTRO_2: u8 = 8;
pub const ORDER_GOAL_2: u8 = 9;
pub const ORDER_BOMB_GET: u8 = 10;

/// `a_demo_name_tbl[]` (`:721-730`).
pub const DEMO_NAMES: [&str; 8] = [
    "P2B_INTRO",
    "P2B_TO_GOAL",
    "P2B_GOAL_WAIT_TALK",
    "P2B_GOAL",
    "P2B_ARRIVE_MAJYU",
    "P2B_INTRO_2",
    "P2B_GOAL_2",
    "P2B_BOMB_GET",
];

/// The two stages the event scripts ask the actor to end (`:1008`, `:1025`, `:1165`).
pub const EVENT_LIFT: &str = "Hlift_up";
pub const EVENT_TREASURE: &str = "DEFAULT_TREASURE";

pub const ACTION_INIT: i32 = 0;
pub const ACTION_DONE: i32 = -1;

/// `m7C1`, the wide cylinder radius in `setCollision` (`:1147-1157`).
pub const CYL_RADIUS_WIDE: f32 = 90.0;
pub const CYL_RADIUS_NARROW: f32 = 30.0;
pub const CYL_HEIGHT: f32 = 120.0;
pub const CYL_WALL: f32 = 30.0;

pub const GRAVITY: f32 = -9.0;
pub const ATTENTION_FLAGS: u32 = 0x0003;
pub const HEAD_LOCK_JNT: u8 = 4;
pub const BACKBONE_JNT: u8 = 2;

/// `a_anm_num_tbl[3][24]` (`:288-292`): `mAnmNo` (`0..=23`) to the BCK index the arc names, `-1`
/// when the child type has no clip for that slot. Type 0's `22` is `ZNOZOKU` (peer into the
/// telescope) and `23` is `ZMIAGE` (raise it); type 2's `21` is `MWAIT02`, the seated idle.
pub const ANM_TABLE: [[i8; 24]; TYPE_COUNT] = [
    [
        -1, 24, -1, -1, 1, 2, -1, -1, 3, 4, 4, 5, 6, 26, -1, -1, -1, -1, -1, -1, -1, -1, 25, 27,
    ],
    [
        -1, 7, 8, 9, 1, 2, 18, 19, 3, 4, 4, 5, 6, 12, 11, 10, 13, 14, 15, 16, 17, -1, -1, -1,
    ],
    [
        -1, 20, -1, -1, 1, 2, -1, -1, 3, 4, 4, 5, 6, 22, 23, -1, -1, -1, -1, -1, -1, 21, -1, -1,
    ],
];

/// `a_play_mode_tbl[]` (`:293-295`): `2` loops, `0` plays once and stops. Index `0` is never
/// used, because `ANM_TABLE` maps that slot to `-1`.
pub const PLAY_MODE: [i32; 24] = [
    -1, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 0, 2, 0, 0, 0, 2, 0,
];

/// `a_play_speed_tbl[]` (`:296-299`).
pub const PLAY_SPEED: [f32; 24] = [
    0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0,
];

/// `a_morf_frame_tbl[]` (`:300-303`), which `setAnm` immediately overwrites with
/// `l_HIO.mChild[mType].m74[i]` (`:305-307`); the HIO constructor (`:108-133`) leaves both the
/// same, so this is the value the blend actually uses.
pub const MORF_FRAMES: [f32; 24] = [
    0.0, 14.0, 8.0, 8.0, 4.0, 0.0, 4.0, 8.0, 6.0, 6.0, 2.0, 20.0, 20.0, 8.0, 12.0, 8.0, 4.0, 8.0,
    8.0, 8.0, 8.0, 8.0, 15.0, 8.0,
];

/// `a_tex_pattern_num_tbl[2][0x17]` (`:337-340`): only slot `1` names a pattern, and only for the
/// two child types that own a BTP.
pub const TEX_PATTERN: [[i8; 0x17]; 2] = [
    [
        -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        -1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
];

/// `head_tex_tbl[2]` (`:3B70`), the pattern the heap pass starts the type on.
pub const HEAD_TEX: [u8; 2] = [0, 1];

/// `setAnm`'s frame-1 crouch sound test (`:319-326`) and the BTP frame counts live in the arc;
/// the port keeps the state machine and stands in for the clips it cannot read.
pub const ONESHOT_FRAMES: f32 = 30.0;
pub const BTP_FRAME_MAX: u8 = 2;
pub const BTP_TIMER_MIN: i16 = 30;
pub const BTP_TIMER_RANDOM: f32 = 100.0;

/// `anm_atr[]` (`:655-658`), indexed by the message's animation attribute.
pub const ANM_ATTR: [u8; 21] = [
    0x01, 0x02, 0x03, 0x0F, 0x0E, 0x0D, 0x10, 0x11, 0x12, 0x13, 0x14, 0x01, 0x02, 0x0D, 0x0E, 0x01,
    0x16, 0x0D, 0x17, 0x07, 0x06,
];

/// `mAnmNo` values `lookBack`/`setAnm`/`anmAtr` test by number.
pub const ANM_WAIT: i32 = 1;
pub const ANM_WALK: i32 = 4;
pub const ANM_JUMP_END: i32 = 3;
pub const ANM_NICOWAIT: i32 = 7;
pub const ANM_NTALK04: i32 = 0x0D;
pub const ANM_ZMIAGE: i32 = 0x17;
pub const ANM_ZNOZOKU: i32 = 0x16;
pub const ANM_MWAIT02: i32 = 0x15;
pub const ANM_NODOROKI: i32 = 0x13;
pub const ANM_NKYORO2: i32 = 0x12;
pub const ANM_NBANZAI: i32 = 0x19;

pub const RND_SEED: i32 = 100;
pub const RND_SCALE: f32 = 100.0;
pub const RND_TIMER_MIN: f32 = 200.0;
pub const TURN_SCALE: i16 = 4;
pub const TURN_MAX_STEP: i16 = 0x800;
pub const AIM_EPS: f32 = 5.0;
pub const AIM_SCALE: f32 = 0.1;
pub const AIM_STEP: f32 = 4.0;
pub const AIM_FACING: i16 = 5376;
pub const PLAYER_EYE_OFFSET: f32 = 0.0;
pub const PLAYER_HEIGHT_GAP: f32 = 300.0;

/// `l_HIO.m280`/`m284` (`:161-162`), the walk animation speed ramp (`:1337-1346`).
pub const WALK_SPEED_SCALE: f32 = 3.0;
pub const WALK_SPEED_LIMIT: f32 = 0.9;
pub const WALK_DISTANCE_BASE: f32 = 10.0;
pub const CUT_MOVE_SLOWDOWN: f32 = 0.25;
pub const CUT_MOVE_FAST: f32 = 10.0;

/// `cutProc`'s staff name (`d_a_npc_p2_cut.inc:22`) and the 15 action names its table holds
/// (`:6-20`).
pub const STAFF_P2B: &str = "P2b";
pub const CUT_ACTIONS: [&str; 15] = [
    "TALK",
    "RIDE_SWITCH",
    "RUN_WAIT",
    "JUMP_TO_LIFT",
    "LIFT_TO_ROPE",
    "ROPE_TALK",
    "ROPE_TO_LIFT",
    "JUMP_TO_GOAL",
    "SET_ANM",
    "JUMP",
    "SW_ON",
    "SW_OFF",
    "SURPRISE",
    "OMAMORI_INIT",
    "OMAMORI_END",
];

pub const CUT_TALK: usize = 0;
pub const CUT_RIDE_SWITCH: usize = 1;
pub const CUT_RUN_WAIT: usize = 2;
pub const CUT_JUMP_TO_LIFT: usize = 3;
pub const CUT_LIFT_TO_ROPE: usize = 4;
pub const CUT_ROPE_TALK: usize = 5;
pub const CUT_ROPE_TO_LIFT: usize = 6;
pub const CUT_JUMP_TO_GOAL: usize = 7;
pub const CUT_SET_ANM: usize = 8;
pub const CUT_JUMP: usize = 9;
pub const CUT_SW_ON: usize = 10;
pub const CUT_SW_OFF: usize = 11;
pub const CUT_SURPRISE: usize = 12;
pub const CUT_OMAMORI_INIT: usize = 13;
pub const CUT_OMAMORI_END: usize = 14;

/// The cut table's tuning: `cLib_addCalc2`'s scale and `max_step` where the procs chase a point,
/// the rope grab's horizontal reach in place of `mCyl.ChkCoHit`, the landing attention offsets
/// (`m7DC.set(0, -100, 0)` / `(0, -50, 0)`, `:5D74`, `:5D8C`), and the hang cap the rope actor
/// applies through its own `m15FC` (`:6A2C`).
pub const CUT_CHASE_SCALE: f32 = 0.1;
pub const CUT_CHASE_STEP: f32 = 24.0;
pub const CUT_ROPE_GRAB: f32 = 60.0;
pub const CUT_ATTN_LAND: Vec3 = Vec3::new(0.0, -100.0, 0.0);
pub const CUT_ATTN_FALL: Vec3 = Vec3::new(0.0, -50.0, 0.0);
pub const CUT_ROPE_HANG_MAX: f32 = 200.0;
pub const CUT_GROUND_EPS: f32 = 1.0;
/// Degrees to the s16 angle the rope swing's two phase offsets use (`DEG_TO_RAD(36)`/`(63)`).
pub const CUT_ROPE_X_LEAD: f32 = 36.0;
pub const CUT_ROPE_Z_LEAD: f32 = 63.0;
pub const CUT_ROPE_CROSS_SCALE: i16 = 0x20;
pub const CUT_ROPE_LIMIT_START: i16 = 0x2EE0;

/// `createInit`'s goal-course positions for `P2b` (`:1596-1600`, `:1582-1588`).
pub const GOAL_START: Vec3 = Vec3::new(0.0, -550.0, -3020.0);
pub const BOMB_START: Vec3 = Vec3::new(0.0, -550.0, -3400.0);

/// `daNpc_P2_childHIO_c` (`:34-56`) plus the `dNpc_HIO_c` fields this actor reads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct P2Npc {
    pub child: ChildHio,
    /// `mNpc.m04`: the offset `dNpc_playerEyePos` applies to the player's eye height.
    pub eye_y: f32,
    /// `mNpc.mMaxHeadTurnVel`, the cap on the head's turn speed while the body follows.
    pub max_head_turn_vel: i16,
    /// `mNpc.mMaxAttnAngleY`.
    pub max_attn_angle_y: i16,
    /// `mNpc.mMaxAttnDistXZ`.
    pub max_attn_dist_xz: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct P2Hio {
    pub npc: P2Npc,
    /// `m30`, `m3C`: where `createInit` restarts `P2b` on each of its two errands.
    pub goal_start: Vec3,
    pub bomb_start: Vec3,
    /// `m48`: the arrival circle, with `m54`/`m58` as radius and half height.
    pub arrival: Vec3,
    pub arrival_radius: f32,
    pub arrival_height: f32,
    /// `m30`/`m5C`/`m64`: the circle that ends the goal run early.
    pub goal_circle: Vec3,
    pub goal_radius: f32,
    pub goal_height: f32,
    /// `m60`/`m68`: the second errand's circle.
    pub bomb_radius: f32,
    pub bomb_height: f32,
    /// `m70`: how close the player must be before `P2b` gives up on the goal run.
    pub player_near: f32,
}

const fn child_npc(
    eye_y: f32,
    max_head_x: i16,
    max_head_y: i16,
    max_backbone_y: i16,
    max_turn_step: i16,
    max_head_turn_vel: i16,
) -> P2Npc {
    P2Npc {
        child: ChildHio {
            attn_y_pos_offset: 50.0,
            max_head_x,
            min_head_x: -max_head_x,
            max_backbone_x: 0,
            min_backbone_x: 0,
            max_head_y,
            min_head_y: -max_head_y,
            max_backbone_y,
            min_backbone_y: -max_backbone_y,
            max_turn_step,
            look_back_target_y: 0,
            unused_20: 0,
            max_talk_dist: 200.0,
            morf_backup: 8.0,
            unused_2c: 8.0,
        },
        eye_y,
        max_head_turn_vel,
        max_attn_angle_y: 0x4000,
        max_attn_dist_xz: 200.0,
    }
}

const HIO: [P2Hio; TYPE_COUNT] = [
    P2Hio {
        npc: child_npc(-20.0, 0x834, 0x28A0, 0x1130, 0x258, 0),
        goal_start: GOAL_START,
        bomb_start: BOMB_START,
        arrival: Vec3::new(5.0, -545.0, 30.0),
        arrival_radius: 80.0,
        arrival_height: 180.0,
        goal_circle: GOAL_START,
        goal_radius: 200.0,
        goal_height: 30.0,
        bomb_radius: 300.0,
        bomb_height: 300.0,
        player_near: 5000.0,
    },
    P2Hio {
        npc: child_npc(-26.0, 0x1FFE, 0x4268, 0xFA0, 0x1000, 0x800),
        goal_start: GOAL_START,
        bomb_start: BOMB_START,
        arrival: Vec3::new(5.0, -545.0, 30.0),
        arrival_radius: 80.0,
        arrival_height: 180.0,
        goal_circle: GOAL_START,
        goal_radius: 200.0,
        goal_height: 30.0,
        bomb_radius: 300.0,
        bomb_height: 300.0,
        player_near: 5000.0,
    },
    P2Hio {
        npc: child_npc(-20.0, 0x1FFE, 0x2134, 0x8E8, 0x960, 0x7D0),
        goal_start: GOAL_START,
        bomb_start: BOMB_START,
        arrival: Vec3::new(5.0, -545.0, 30.0),
        arrival_radius: 80.0,
        arrival_height: 180.0,
        goal_circle: GOAL_START,
        goal_radius: 200.0,
        goal_height: 30.0,
        bomb_radius: 300.0,
        bomb_height: 300.0,
        player_near: 5000.0,
    },
];

pub fn hio(kind: u8) -> P2Hio {
    HIO.get(usize::from(kind)).copied().unwrap_or(HIO[0])
}

/// The goal run's absolute aim points (`:1043-1049`, `:1059-1063`).
pub const AIM_TALK: Vec3 = Vec3::new(0.0, -450.0, -2915.0);
pub const ATTN_TALK: Vec3 = Vec3::new(0.0, -450.0, -2715.0);
pub const AIM_GOAL: Vec3 = Vec3::new(0.0, -450.0, -3130.0);
pub const ATTN_GOAL: Vec3 = Vec3::new(0.0, -450.0, -3380.0);
pub const TALK_ATTN: Vec3 = Vec3::new(0.0, -450.0, -2715.0);

/// `mActionFunc`: the reference stores a member pointer and calls it every execute (`:1273`);
/// `createInit` (`:1620-1638`) picks between the two.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Intro,
    Wait,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Morph {
    /// `J3DFrameCtrl::EMode`: `0` once, `2` loop.
    pub mode: i32,
    pub frame: f32,
    pub morf: f32,
    pub rate: f32,
    pub stop: bool,
}

impl Morph {
    pub fn idle() -> Self {
        Self {
            mode: 2,
            frame: 0.0,
            morf: 1.0,
            rate: 1.0,
            stop: false,
        }
    }

    /// `dNpc_setAnm(mpMorf, mode, morfFrame, speed, bck, -1, arc)`.
    pub fn set_anm(&mut self, mode: i32, morf: f32, speed: f32) {
        self.mode = mode;
        self.frame = 0.0;
        self.morf = morf;
        self.rate = speed;
        self.stop = false;
    }

    pub fn play(&mut self) {
        self.stop = false;
        if self.rate <= 0.0 {
            // `mDoExt_McaMorf::isStop` is true the tick the frame control's rate is zero.
            self.stop = true;
            return;
        }
        self.frame += self.rate;
        if self.mode == 0 && self.frame >= ONESHOT_FRAMES {
            self.frame = ONESHOT_FRAMES;
            self.stop = true;
        }
    }

    pub fn is_stop(&self) -> bool {
        self.stop
    }
}

/// The cut table's state: `m7D7`, `m7F4`, `m7F8`, `m7FC`, `m800`, `m7E8`, `m7DC`,
/// `mSavedPos`/`mSavedAngle`, the actor's own `speed`/`speedF` that `fopAcM_posMoveF`
/// integrates, the nearest-actor search's `mClosestPos`/`mActor`, and the rope's
/// `m798..m7AC`/`mSoundPos`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cut {
    /// `m7D7`, set while a private-table act is running (`d_a_npc_p2_cut.inc:32`, `:37`).
    pub active: u8,
    /// `m7F4`, the landing pause timer.
    pub timer: i16,
    /// `m7FC`, the cut jump's horizontal speed.
    pub speed: f32,
    /// `m7F8`, the cut jump's launch speed.
    pub speed_y: f32,
    /// `m800`, the cut jump's gravity.
    pub gravity: f32,
    /// `m7E8`, the jump's goal point.
    pub goal: Vec3,
    /// `m7DC`, the offset the landing states add to the attention point.
    pub attn_offset: Vec3,
    /// `mSavedPos`/`mSavedAngle`: where the cut left the actor.
    pub saved_pos: Vec3,
    pub saved_angle: [i16; 3],
    /// `speed.y`/`speedF`, which `fopAcM_posMoveF` integrates (`:1316`).
    pub vel_y: f32,
    pub vel_f: f32,
    /// `mClosestPos`, the nearest lift/rope the search found.
    pub closest: Vec3,
    /// `mActor`, the actor the search picked.
    pub target: Option<Entity>,
    /// `m7C0`, the rope-hang latch (`:1330`).
    pub rope_hung: u8,
    /// `m798`/`m79C`, the rope's distance and the swing's angular scale.
    pub rope_dist: f32,
    pub rope_scale: f32,
    /// `m7A0`/`m7A4`, the two swing phases.
    pub rope_phase: f32,
    pub rope_swing: f32,
    /// `m7A8`/`m7AA`, the swing's amplitude and cross angle.
    pub rope_limit: i16,
    pub rope_cross: i16,
    /// `m7AC`.
    pub rope_timer: u8,
    /// `mSoundPos`, the rope's end the hang is measured from.
    pub sound_pos: Vec3,
    /// The two tilt angles the rope swing writes (`current.angle.x`/`.z`, `:6F7C`). The port's
    /// `Ctx` only carries the yaw, and the actor has no model to tilt, so they stay cut-local.
    pub tilt_x: i16,
    pub tilt_z: i16,
    /// `m72C`, the cut's attention latch (`:1282`, `:1309`).
    pub attention: bool,
    /// The port's `mAcch.ChkGroundHit()` memory: last tick's contact.
    pub grounded: bool,
}

impl Cut {
    pub const fn new() -> Self {
        Self {
            active: 0,
            timer: 0,
            speed: 0.0,
            speed_y: 0.0,
            gravity: 0.0,
            goal: Vec3::ZERO,
            attn_offset: Vec3::ZERO,
            saved_pos: Vec3::ZERO,
            saved_angle: [0; 3],
            vel_y: 0.0,
            vel_f: 0.0,
            closest: Vec3::ZERO,
            target: None,
            rope_hung: 0,
            rope_dist: 0.0,
            rope_scale: 0.0,
            rope_phase: 0.0,
            rope_swing: 0.0,
            rope_limit: 0,
            rope_cross: 0,
            rope_timer: 0,
            sound_pos: Vec3::ZERO,
            tilt_x: 0,
            tilt_z: 0,
            attention: false,
            grounded: false,
        }
    }
}

impl Default for Cut {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct NpcP2 {
    pub kind: u8,
    /// `m291`, the parameter's second byte: which of `P2b`'s two errands this child runs.
    pub sub: u8,
    /// `mSwitchNo`, read by `goal_wait_2` (`:1B04`).
    pub switch_no: u8,
    pub action: Action,
    /// `m7D6`.
    pub state: i8,
    /// `m7D5`, the pending event order.
    pub order: u8,
    /// `m808`, the action function's first-tick latch.
    pub started: i32,
    /// `m724`, `chkAttention`'s answer.
    pub attn: bool,
    /// `mHasAttention`, raised by `checkOrder` when the talk event is accepted.
    pub has_attention: bool,
    /// `m7C1`, the wide cylinder flag the goal states raise.
    pub big: bool,
    /// `m730`, the point the head tracks in the goal states.
    pub attn_pos: Vec3,
    /// `m73C`, the point `P2b` tracks after its goal conversation.
    pub talk_attn: Vec3,
    /// `m7DC`, the offset the event cut adds to its attention point.
    pub attn_offset: Vec3,
    /// `m74A`, the latch that only orders the goal conversation once.
    pub spoke: bool,
    /// `m751`.
    pub anm_latch: u8,
    /// `m74C`, the mocco idle's pause timer.
    pub mocco_timer: i16,
    /// `m748`, the child's angle relative to the ship it rides on (`:1650`, `:1657`).
    pub angle_off: i16,
    pub anm_no: i32,
    /// `m7D4`, the last `mAnmNo` `setAnm` saw.
    pub prev_anm: i32,
    /// `m7D2`, the BCK index the arc names.
    pub bone: i32,
    /// `m7D1`.
    pub tex_index: u8,
    /// `m7D0`.
    pub tex_pattern: u8,
    pub btp_frame: u8,
    pub btp_timer: i16,
    pub morph: Morph,
    pub jnt: Jnt,
    pub max_look_vel: i16,
    pub eye_pos: Vec3,
    pub home: Vec3,
    pub home_angle: i16,
    /// `mMsgNo`.
    pub msg_no: u32,
    /// `mEndMsgNo` (`d_npc_event_cut.inc:574-582`), the message that ends the cut early; `-1` in
    /// retail, so `u32::MAX` here.
    pub end_msg_no: u32,
    pub msg_id: u32,
    /// `mTalkState`.
    pub talk_state: i8,
    /// The port's create latch: the first tick serves as `_create`'s `createInit` pass.
    pub entered: bool,
    /// The port's stand-in for `fopAcM_orderSpeakEvent` having been accepted: the actor has asked
    /// for a conversation and is waiting on the player's button.
    pub talk_order_ready: bool,
    /// The button latch, so holding the action button does not re-fire a press every tick.
    pub do_held: bool,
    pub cyl_radius: f32,
    pub cyl_height: f32,
    pub seed: [i32; 3],
    pub cut: Cut,
    /// `mEventCut`, the shared `dNpc_EventCut_c` state the table in [`super::event_cut`] drives.
    pub cut_shared: EventCut,
}

impl NpcP2 {
    pub fn spawn(parameters: u32, pos: Vec3, angle: i16) -> Self {
        let mut kind = (parameters & PARAM_TYPE_MASK) as u8;
        if kind == 3 {
            kind = TYPE_P2A;
        }
        let mut sub = ((parameters >> PARAM_SUB_SHIFT) & PARAM_BYTE_MASK) as u8;
        if sub == PARAM_BYTE_MASK as u8 {
            sub = 0;
        }
        Self {
            kind,
            sub,
            switch_no: ((parameters >> PARAM_SWITCH_SHIFT) & PARAM_BYTE_MASK) as u8,
            action: Action::Wait,
            state: STATE_WAIT,
            order: ORDER_NONE,
            started: 0,
            attn: false,
            has_attention: false,
            big: false,
            attn_pos: Vec3::ZERO,
            talk_attn: Vec3::ZERO,
            attn_offset: Vec3::ZERO,
            spoke: false,
            anm_latch: 0,
            mocco_timer: 0,
            angle_off: 0,
            anm_no: 1,
            prev_anm: 0,
            bone: -1,
            tex_index: 0,
            tex_pattern: 0,
            btp_frame: 0,
            btp_timer: 0,
            morph: Morph::idle(),
            jnt: Jnt::new(),
            max_look_vel: 0,
            eye_pos: pos,
            home: pos,
            home_angle: angle,
            msg_no: 0,
            end_msg_no: u32::MAX,
            msg_id: FPC_M_ERROR_PROCESS_ID,
            talk_state: 0,
            entered: false,
            talk_order_ready: false,
            do_held: false,
            cyl_radius: CYL_RADIUS_NARROW,
            cyl_height: CYL_HEIGHT,
            seed: [RND_SEED; 3],
            cut: Cut::new(),
            cut_shared: EventCut::create(),
        }
    }

    pub fn drawn(&self) -> bool {
        true
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        let reach = CYL_WALL * 2.0;
        ([-reach, -100.0, -reach], [reach, CYL_HEIGHT + 100.0, reach])
    }

    /// `setCollision` (`:1160-1172`).
    fn set_collision(&mut self) {
        self.cyl_radius = if self.big {
            CYL_RADIUS_WIDE
        } else {
            CYL_RADIUS_NARROW
        };
        self.cyl_height = CYL_HEIGHT;
    }

    fn morf_frame(&self, anm_no: i32) -> f32 {
        usize::try_from(anm_no)
            .ok()
            .and_then(|index| MORF_FRAMES.get(index).copied())
            .unwrap_or(0.0)
    }

    fn bone_of(&self, anm_no: i32) -> i32 {
        usize::try_from(anm_no)
            .ok()
            .and_then(|index| ANM_TABLE.get(usize::from(self.kind))?.get(index).copied())
            .map(i32::from)
            .unwrap_or(-1)
    }

    fn mode_of(anm_no: i32) -> i32 {
        usize::try_from(anm_no)
            .ok()
            .and_then(|index| PLAY_MODE.get(index).copied())
            .unwrap_or(-1)
    }

    fn speed_of(anm_no: i32) -> f32 {
        usize::try_from(anm_no)
            .ok()
            .and_then(|index| PLAY_SPEED.get(index).copied())
            .unwrap_or(0.0)
    }

    /// `setAnm` (`:240-334`).
    pub fn set_anm(&mut self, cut_move_speed: f32, cut_no: i32) {
        if self.prev_anm != self.anm_no {
            let bone = self.bone_of(self.anm_no);
            if bone >= 0 {
                self.bone = bone;
                let mut speed = Self::speed_of(self.anm_no);
                if cut_move_speed != 0.0 && matches!(cut_no, 2 | 4) {
                    speed = cut_move_speed * CUT_MOVE_SLOWDOWN;
                }
                let morf = self.morf_frame(self.anm_no);
                let mode = Self::mode_of(self.anm_no);
                self.morph.set_anm(mode, morf, speed);
            }
        }
        self.prev_anm = self.anm_no;
        if self.morph.is_stop() {
            if self.anm_no == ANM_NODOROKI {
                self.anm_no = ANM_JUMP_END;
            }
            if self.anm_no == ANM_NICOWAIT {
                self.anm_no = ANM_WAIT;
            }
            if self.kind == TYPE_P2A && self.anm_no == ANM_ZMIAGE {
                self.anm_no = ANM_NTALK04;
            }
        }
        // `setAnm`'s frame-1 crouch sound (`:319-326`) needs the arc's SE id, which the actor
        // stores by resource index; the port's sound table is not indexed that way.
    }

    /// `setTexAnm` (`:335-347`). `mType == 2` has no BTP, so the table never applies to `P2c`.
    pub fn set_tex_anm(&mut self) {
        if self.kind as usize >= TEX_PATTERN.len() {
            return;
        }
        let Some(slot) = usize::try_from(self.tex_index)
            .ok()
            .and_then(|index| TEX_PATTERN[self.kind as usize].get(index).copied())
        else {
            return;
        };
        if slot >= 0 && slot as u8 != self.tex_pattern {
            self.tex_pattern = slot as u8;
            self.btp_frame = 0;
            self.btp_timer = 0;
        }
    }

    /// `playTexPatternAnm` (`:228-238`).
    pub fn play_tex_pattern_anm(&mut self) {
        if self.kind == TYPE_P2C {
            return;
        }
        if self.btp_timer != 0 {
            self.btp_timer -= 1;
        }
        if self.btp_timer == 0 {
            if self.btp_frame >= BTP_FRAME_MAX {
                self.btp_frame = self.btp_frame.wrapping_sub(BTP_FRAME_MAX);
                let rng = (super::npc_p1::rnd(&mut self.seed) * BTP_TIMER_RANDOM) as i16;
                self.btp_timer = rng.wrapping_add(BTP_TIMER_MIN);
            } else {
                self.btp_frame = self.btp_frame.wrapping_add(1);
            }
        }
    }

    /// `chkAttention` (`:362-380`). The lock-on branch compares `this` against the attention
    /// manager's target; the port's talk system owns that target, so what is left is the
    /// distance-and-facing test plus `P2b`'s two unconditional cases.
    pub fn chk_attention(&self, ctx: &Ctx) -> bool {
        let Some(player) = ctx.player else {
            return false;
        };
        let dist = abs_xz(player - ctx.pos);
        let max_angle = hio(self.kind).npc.max_attn_angle_y;
        let look = ctx.angle.wrapping_add(self.jnt.angles[0][1]);
        let look = look.wrapping_add(self.jnt.angles[1][1]);
        if self.kind == TYPE_P2B
            && (self.big || self.state == STATE_DEMO_GOAL_2)
            && dist < hio(self.kind).npc.max_attn_dist_xz
        {
            return true;
        }
        if i32::from(max_angle) > i32::from(look).abs()
            && dist < hio(self.kind).npc.max_attn_dist_xz
            && ctx.event_mode
        {
            return true;
        }
        ctx.talk_command
    }

    /// `setAttention` (`:348-361`): the port keeps the two points on the body origin, which is
    /// where `createInit` seeds them (`:1640-1641`), instead of on the head joint matrix.
    fn set_attention(&mut self, ctx: &Ctx) {
        self.attn_pos = Vec3::new(
            ctx.pos.x,
            ctx.pos.y + hio(self.kind).npc.child.attn_y_pos_offset,
            ctx.pos.z,
        );
        self.eye_pos = ctx.pos;
    }

    /// `lookBack` (`:382-471`).
    pub fn look_back(&mut self, ctx: &mut Ctx) {
        let npc = hio(self.kind).npc;
        let mut target = ctx.pos;
        let mut dst: Option<Vec3> = None;
        let desired = ctx.angle;
        let mut head_only = ctx.cut_attn_no_turn;

        if self.attn || self.cut.attention || ctx.cut_attn {
            self.jnt.set_trn();
            dst = Some(ctx.cut_attn_pos + self.attn_offset);
            target.y = self.eye_pos.y;
        } else {
            if self.state == STATE_TREASURE_WAIT_TALK {
                self.jnt.set_trn();
            }
            match self.state {
                STATE_GOALPOS_TO_TALKPOS | STATE_TALKPOS_TO_GOALPOS => {
                    self.jnt.set_trn();
                    dst = Some(self.attn_pos);
                    target.y = self.eye_pos.y;
                }
                STATE_TALK => {
                    self.jnt.set_trn();
                    dst = Some(player_eye(ctx, npc.eye_y));
                    target.y = self.eye_pos.y;
                }
                _ => {
                    if self.attn {
                        dst = Some(player_eye(ctx, npc.eye_y));
                        target.y = self.eye_pos.y;
                    }
                }
            }
        }

        if self.kind == TYPE_P2A {
            if self.anm_no == ANM_ZMIAGE {
                if let Some(zl1) = ctx.zl1 {
                    let yaw = walk::to_s16((zl1.x - ctx.pos.x).atan2(zl1.z - ctx.pos.z));
                    add_calc_angle_s2(&mut ctx.angle, yaw, TURN_SCALE, TURN_MAX_STEP);
                    let mut look = ctx.pos;
                    look.y = self.eye_pos.y;
                    self.jnt.on_head_lock();
                    self.jnt.on_back_bone_lock();
                    let mut yaw = ctx.angle;
                    self.jnt.look_at_target(
                        &mut yaw,
                        Some(zl1),
                        look,
                        desired,
                        self.max_look_vel,
                        false,
                    );
                    ctx.angle = yaw;
                    return;
                }
            } else if self.anm_no == ANM_ZNOZOKU {
                head_only = false;
                self.jnt.trn = false;
                self.jnt.on_head_lock();
                self.jnt.on_back_bone_lock();
            } else {
                self.jnt.off_head_lock();
                self.jnt.off_back_bone_lock();
            }
        }

        if self.state == STATE_MOCCO_WAIT {
            head_only = false;
            self.jnt.trn = false;
        }

        if self.kind == TYPE_P2B && self.state == STATE_TREASURE_WAIT && self.sub == 1 {
            self.jnt.set_trn();
            dst = Some(self.talk_attn);
            target.y = self.eye_pos.y;
        }

        if self.jnt.trn_chk() {
            add_calc_angle_s2(
                &mut self.max_look_vel,
                npc.max_head_turn_vel,
                TURN_SCALE,
                TURN_MAX_STEP,
            );
        } else {
            self.max_look_vel = 0;
        }

        self.jnt.set_param(npc.child);
        let mut yaw = ctx.angle;
        self.jnt
            .look_at_target(&mut yaw, dst, target, desired, self.max_look_vel, head_only);
        ctx.angle = yaw;
    }

    /// `getMsg` (`:601-638`).
    pub fn get_msg(&mut self, stories: &mut StoryBits) -> u32 {
        match self.kind {
            TYPE_P2A => {
                if !stories.test(EV_0808) {
                    set_first_msg(stories, EV_0702, MSG_100E, MSG_100F)
                } else {
                    MSG_1010
                }
            }
            TYPE_P2B => {
                if self.sub == 1 {
                    if stories.test(EV_0F02) {
                        set_first_msg(stories, EV_1502, MSG_1B35, MSG_1B36)
                    } else {
                        MSG_1028
                    }
                } else if !stories.test(EV_0720) {
                    set_first_msg(stories, EV_0940, MSG_C96, MSG_C97)
                } else if !stories.test(EV_0808) {
                    if stories.test(EV_0710) { MSG_1028 } else { 0 }
                } else {
                    set_first_msg(stories, EV_0704, MSG_1029, MSG_102A)
                }
            }
            _ => {
                if !stories.test(EV_0808) {
                    set_first_msg(stories, EV_0701, 0x1011, 0x1012)
                } else {
                    MSG_1013
                }
            }
        }
    }

    /// `next_msgStatus` (`:524-599`): advances `mMsgNo` and answers whether the box should show
    /// the next page or close.
    fn next_msg_status(&mut self) -> u8 {
        match self.msg_no {
            MSG_1018 => {
                self.msg_no = MSG_1019;
                STATUS_MSG_CONTINUES
            }
            MSG_1020 => {
                self.msg_no = MSG_1021;
                STATUS_MSG_CONTINUES
            }
            MSG_1022 => {
                self.msg_no = MSG_102C;
                STATUS_MSG_CONTINUES
            }
            MSG_1023 => {
                self.msg_no = MSG_1024;
                STATUS_MSG_CONTINUES
            }
            MSG_1025 => {
                self.msg_no = MSG_1026;
                STATUS_MSG_CONTINUES
            }
            MSG_1026 => {
                self.msg_no = MSG_1027;
                STATUS_MSG_CONTINUES
            }
            MSG_1027 => {
                self.msg_no = MSG_102D;
                STATUS_MSG_CONTINUES
            }
            MSG_C96 => {
                self.msg_no = MSG_C98;
                STATUS_MSG_CONTINUES
            }
            0x1011 => {
                self.msg_no = MSG_102F;
                STATUS_MSG_CONTINUES
            }
            MSG_102F => {
                self.msg_no = MSG_1030;
                STATUS_MSG_CONTINUES
            }
            0x1012 => {
                self.msg_no = MSG_1031;
                STATUS_MSG_CONTINUES
            }
            MSG_1031 => {
                self.msg_no = MSG_1032;
                STATUS_MSG_CONTINUES
            }
            0x1B20 => {
                self.msg_no = 0x1B21;
                STATUS_MSG_CONTINUES
            }
            0x1B21 => {
                self.msg_no = 0x1B22;
                STATUS_MSG_CONTINUES
            }
            0x1B22 => {
                self.msg_no = 0x1B23;
                STATUS_MSG_CONTINUES
            }
            0x1B23 => {
                self.msg_no = 0x1B24;
                STATUS_MSG_CONTINUES
            }
            0x1B28 => {
                self.msg_no = 0x1B29;
                STATUS_MSG_CONTINUES
            }
            0x1B29 => {
                self.msg_no = 0x1B2A;
                STATUS_MSG_CONTINUES
            }
            0x1B2A => {
                self.msg_no = 0x1B2B;
                STATUS_MSG_CONTINUES
            }
            0x1B2B => {
                self.msg_no = 0x1B2C;
                STATUS_MSG_CONTINUES
            }
            0x1B2C => {
                self.msg_no = 0x1B2D;
                STATUS_MSG_CONTINUES
            }
            0x1B2D => {
                self.msg_no = 0x1B2E;
                STATUS_MSG_CONTINUES
            }
            _ => STATUS_MSG_ENDS,
        }
    }

    /// `anmAtr` (`:645-669`).
    fn anm_attr(&mut self, status: u16, text: &mut Text<'_>) {
        if status == STATUS_MSG_DISPLAYED as u16 {
            let attr = text.mesg_anime();
            if usize::from(attr) < ANM_ATTR.len() {
                let anm = i32::from(ANM_ATTR[usize::from(attr)]);
                if self.kind == TYPE_P2A && self.anm_no == ANM_NTALK04 && anm == ANM_ZMIAGE {
                    return;
                }
                if self.kind == TYPE_P2B && anm == ANM_NICOWAIT {
                    if self.anm_latch == 0 {
                        self.anm_latch = 1;
                        self.anm_no = anm;
                    }
                    return;
                }
                if self.anm_no != anm {
                    self.anm_no = anm;
                }
            } else {
                self.anm_no = ANM_WAIT;
            }
        } else if status == STATUS_BOX_CLOSED as u16 {
            self.anm_no = ANM_WAIT;
        }
    }

    /// `talk` (`:678-718`): returns the message status the box is in, `BOX_CLOSED` once it has
    /// been dismissed.
    pub fn talk(&mut self, ctx: &mut Ctx, use_msg_no: bool) -> u16 {
        let mut mode = 0xFF;
        if self.talk_state == 0 {
            self.msg_id = FPC_M_ERROR_PROCESS_ID;
            if !use_msg_no {
                self.msg_no = self.get_msg(ctx.stories);
            }
            self.talk_state = 1;
        } else if self.talk_state == -1 {
            mode = STATUS_BOX_CLOSED as u16;
            self.anm_attr(mode, ctx.text);
        } else if self.msg_id == FPC_M_ERROR_PROCESS_ID {
            let eye = self.eye_pos;
            self.msg_id = ctx.text.message_set(self.msg_no, eye);
        } else if self.talk_state == 1 {
            if ctx.text.bound() {
                self.talk_state = 2;
            }
        } else {
            mode = ctx.text.status().map_or(0xFF, u16::from);
            if mode == STATUS_MSG_DISPLAYED as u16 {
                let next = self.next_msg_status();
                if next == STATUS_MSG_CONTINUES {
                    let eye = self.eye_pos;
                    ctx.text.message_set(self.msg_no, eye);
                }
                ctx.text.set_status(next);
            } else if mode == STATUS_BOX_CLOSED as u16 {
                ctx.text.set_status(STATUS_MSG_DESTROYED);
                self.talk_state = -1;
                self.anm_latch = 0;
            }
            self.anm_attr(mode, ctx.text);
        }
        mode
    }

    /// `eventOrder` (`:720-742`).
    fn event_order(&mut self, ctx: &mut Ctx) {
        if self.order == ORDER_SPEAK || self.order == ORDER_TALK {
            // `fopAcM_orderSpeakEvent` is the talk system's press; the port raises the actor's
            // own talk condition instead and lets `talk` drive the box.
            self.talk_order_ready = true;
        } else if self.order >= ORDER_FIRST_DEMO {
            let index = usize::from(self.order - ORDER_FIRST_DEMO);
            let Some(name) = DEMO_NAMES.get(index).copied() else {
                return;
            };
            let Some(event) = ctx.list.event_index(name) else {
                return;
            };
            if ctx.active.stage_of(event).is_none() {
                ctx.active.order(ctx.list, event, ctx.entity);
            }
        }
    }

    /// `checkOrder` (`:743-755`).
    fn check_order(&mut self, ctx: &mut Ctx) {
        if self.order >= ORDER_FIRST_DEMO {
            let index = usize::from(self.order - ORDER_FIRST_DEMO);
            let accepted = DEMO_NAMES
                .get(index)
                .and_then(|name| ctx.list.event_index(name))
                .and_then(|event| ctx.active.stage_of(event));
            if accepted.is_some() {
                self.order = ORDER_NONE;
            }
        }
        if ctx.talk_command && (self.order == ORDER_SPEAK || self.order == ORDER_TALK) {
            self.order = ORDER_NONE;
            self.talk_order_ready = false;
            self.has_attention = true;
            self.talk_state = 0;
        }
    }

    fn end_check(&self, ctx: &Ctx, name: &str) -> bool {
        ctx.list
            .event_index(name)
            .is_some_and(|index| ctx.active.end_check(index))
    }

    /// `wait01` (`:1046-1053`).
    fn wait01(&mut self) {
        if self.has_attention {
            self.state = STATE_TALK;
        } else {
            self.order = ORDER_TALK;
        }
    }

    /// `talk01` (`:1103-1120`).
    fn talk01(&mut self, ctx: &mut Ctx) {
        if self.talk(ctx, false) == STATUS_BOX_CLOSED as u16 {
            if self.kind == TYPE_P2C {
                self.anm_no = ANM_WAIT;
                self.state = STATE_MOCCO_WAIT;
            } else if self.kind == TYPE_P2A && !ctx.stories.test(EV_0808) {
                self.state = STATE_ZUKO_TELESCOPE;
                self.jnt.on_head_lock();
                self.jnt.on_back_bone_lock();
            } else {
                self.anm_no = ANM_WAIT;
                self.state = STATE_WAIT;
            }
            ctx.reset_event();
            self.has_attention = false;
        }
    }

    /// `zukotelescope` (`:1055-1070`).
    fn zukotelescope(&mut self, ctx: &mut Ctx) {
        self.anm_no = ANM_ZNOZOKU;
        self.track_parent(ctx);
        if self.has_attention {
            self.anm_no = ANM_WAIT;
            self.jnt.off_head_lock();
            self.jnt.off_back_bone_lock();
            self.state = STATE_TALK;
        } else {
            self.order = ORDER_TALK;
        }
    }

    /// `moccowait` (`:1072-1101`).
    fn moccowait(&mut self, ctx: &mut Ctx) {
        self.track_parent(ctx);
        // `mpMorf2` copies the body's frame (`:1082-1086`); the port's book morph has no model.
        if self.anm_no == ANM_WAIT {
            if self.mocco_timer != 0 {
                self.mocco_timer -= 1;
            }
            if self.mocco_timer == 0 {
                self.anm_no = ANM_MWAIT02;
                let rng = (super::npc_p1::rnd(&mut self.seed) * RND_SCALE) as i16;
                self.mocco_timer = rng.wrapping_add(RND_TIMER_MIN as i16);
            }
        }
        if self.anm_no == ANM_MWAIT02 && self.morph.is_stop() {
            self.anm_no = ANM_WAIT;
        }
        if self.has_attention && self.anm_no == ANM_WAIT {
            self.state = STATE_TALK;
        } else {
            self.order = ORDER_TALK;
        }
    }

    /// Both deck states turn the child to `m748 + parent.current.angle.y` (`:1060-1062`,
    /// `:1074-1077`). `createInit` (`:1648-1661`) only records the offset when the parent is the
    /// pirate ship.
    fn track_parent(&mut self, ctx: &mut Ctx) {
        let Some((_, ship_angle)) = ctx.ship else {
            return;
        };
        let target = self.angle_off.wrapping_add(ship_angle);
        add_calc_angle_s2(&mut ctx.angle, target, TURN_SCALE, TURN_MAX_STEP);
    }

    /// `demo_wait` (`:811-817`).
    fn demo_wait(&mut self, ctx: &mut Ctx) {
        if in_circle(
            hio(self.kind).arrival,
            hio(self.kind).arrival_radius,
            hio(self.kind).arrival_height,
            ctx.player,
        ) {
            self.order = ORDER_INTRO;
            self.state = STATE_DEMO_INTRO;
        }
    }

    /// `demo_intro` (`:819-826`).
    fn demo_intro(&mut self, ctx: &mut Ctx) {
        if self.end_check(ctx, DEMO_NAMES[0]) {
            self.state = STATE_DEMO_LIFT;
            self.order = ORDER_NONE;
            ctx.reset_event();
        }
    }

    /// `demo_lift` (`:828-835`).
    fn demo_lift(&mut self, ctx: &mut Ctx) {
        if self.end_check(ctx, EVENT_LIFT) {
            self.state = STATE_DEMO_JUMP;
            self.order = ORDER_TO_GOAL;
        }
    }

    /// `demo_jump` (`:837-844`).
    fn demo_jump(&mut self, ctx: &mut Ctx) {
        if self.end_check(ctx, DEMO_NAMES[1]) {
            ctx.stories.set(EV_0720);
            self.state = STATE_GOALPOS_TO_TALKPOS;
            ctx.reset_event();
        }
    }

    /// `goal_goalpos_to_talkpos` (`:846-880`).
    fn goal_goalpos_to_talkpos(&mut self, ctx: &mut Ctx) {
        self.big = true;
        let height_gap = ctx.pos.y - ctx.player.map_or(0.0, |link| link.y) - PLAYER_HEIGHT_GAP;
        if height_gap < 0.0 {
            self.state = STATE_TALKPOS_TO_GOALPOS;
            return;
        }
        let yaw = walk::to_s16((AIM_TALK.x - ctx.pos.x).atan2(AIM_TALK.z - ctx.pos.z));
        self.attn_pos = ATTN_TALK;
        if (i32::from(yaw.wrapping_sub(ctx.angle)).abs() as f32) < AIM_FACING as f32 {
            self.anm_no = ANM_WALK;
            add_calc2(&mut ctx.pos.x, AIM_TALK.x, AIM_SCALE, AIM_STEP);
            add_calc2(&mut ctx.pos.z, AIM_TALK.z, AIM_SCALE, AIM_STEP);
        }
        if abs_xz(AIM_TALK - ctx.pos) < AIM_EPS {
            self.anm_no = ANM_WAIT;
            self.state = STATE_GOAL_TALKPOS_WAIT;
        }
        self.goal_circle_exit(ctx);
    }

    /// `goal_talkpos_to_goalpos` (`:882-925`).
    fn goal_talkpos_to_goalpos(&mut self, ctx: &mut Ctx) {
        self.big = true;
        let dist = ctx.player.map_or(f32::MAX, |link| abs_xz(link - ctx.pos));
        let height_gap = ctx.pos.y - ctx.player.map_or(0.0, |link| link.y) - PLAYER_HEIGHT_GAP;
        if dist < hio(self.kind).player_near && height_gap > 0.0 {
            self.state = STATE_GOALPOS_TO_TALKPOS;
            return;
        }
        let remaining = abs_xz(AIM_GOAL - ctx.pos);
        self.attn_pos = if remaining < AIM_EPS {
            TALK_ATTN
        } else {
            ATTN_GOAL
        };
        let yaw = walk::to_s16((self.attn_pos.x - ctx.pos.x).atan2(self.attn_pos.z - ctx.pos.z));
        if remaining < AIM_EPS {
            if (i32::from(yaw.wrapping_sub(ctx.angle)).abs() as f32) < AIM_FACING as f32 {
                self.anm_no = ANM_WAIT;
                self.state = STATE_GOALPOS_WAIT;
            }
        } else if (i32::from(yaw.wrapping_sub(ctx.angle)).abs() as f32) < AIM_FACING as f32 {
            self.anm_no = ANM_WALK;
            add_calc2(&mut ctx.pos.x, AIM_GOAL.x, AIM_SCALE, AIM_STEP);
            add_calc2(&mut ctx.pos.z, AIM_GOAL.z, AIM_SCALE, AIM_STEP);
        }
        self.goal_circle_exit(ctx);
    }

    /// `goal_goalpos_wait` (`:927-948`).
    fn goal_goalpos_wait(&mut self, ctx: &mut Ctx) {
        self.big = true;
        let dist = ctx.player.map_or(f32::MAX, |link| abs_xz(link - ctx.pos));
        let height_gap = ctx.pos.y - ctx.player.map_or(0.0, |link| link.y) - PLAYER_HEIGHT_GAP;
        if dist < hio(self.kind).player_near && height_gap > 0.0 {
            self.state = STATE_GOALPOS_TO_TALKPOS;
            return;
        }
        self.goal_circle_exit(ctx);
    }

    /// `goal_talkpos_wait` (`:950-982`).
    fn goal_talkpos_wait(&mut self, ctx: &mut Ctx) {
        self.big = true;
        let height_gap = ctx.pos.y - ctx.player.map_or(0.0, |link| link.y) - PLAYER_HEIGHT_GAP;
        if height_gap < 0.0 {
            self.state = STATE_TALKPOS_TO_GOALPOS;
            return;
        }
        if !self.spoke {
            self.order = ORDER_SPEAK;
            self.spoke = true;
        } else {
            if self.has_attention {
                self.state = STATE_GOAL_TALKPOS_TALK;
                ctx.reset_event();
                self.order = ORDER_GOAL_WAIT_TALK;
            } else {
                // Retail also tests `fopAcM_GetSpeedF(player) < 1.0f` here; both branches raise
                // the same talk order in the port, so the speed read has nothing to choose.
                self.order = ORDER_TALK;
            }
        }
        self.goal_circle_exit(ctx);
    }

    /// The `dLib_checkPlayerInCircle(m30, m5C, m64) && !checkPlayerFly()` tail that five goal
    /// states share (`:875-879`, `:920-924`, `:945-947`, `:976-980`).
    fn goal_circle_exit(&mut self, ctx: &mut Ctx) {
        let hio = hio(self.kind);
        if in_circle(
            hio.goal_circle,
            hio.goal_radius,
            hio.goal_height,
            ctx.player,
        ) {
            self.state = STATE_DEMO_GOAL;
            // Retail stops the goal timer here (`:877-879`); the port's `Objtime` has no stop.
        }
    }

    /// `demo_goal` (`:984-997`).
    fn demo_goal(&mut self, ctx: &mut Ctx) {
        self.big = false;
        if self.end_check(ctx, DEMO_NAMES[3]) {
            ctx.stories.set(EV_0710);
            self.state = STATE_TREASURE_WAIT;
            self.order = ORDER_NONE;
            ctx.reset_event();
        } else {
            self.order = ORDER_GOAL;
        }
    }

    /// `goal_talkpos_talk` (`:999-1006`).
    fn goal_talkpos_talk(&mut self, ctx: &mut Ctx) {
        if self.end_check(ctx, DEMO_NAMES[2]) {
            self.state = STATE_GOAL_TALKPOS_WAIT;
            ctx.reset_event();
            self.has_attention = false;
        }
    }

    /// `treasure_wait` (`:1008-1023`).
    fn treasure_wait(&mut self, ctx: &mut Ctx) {
        if self.end_check(ctx, EVENT_TREASURE) {
            if self.sub == 0 {
                self.state = STATE_DEMO_ARRIVE;
                self.order = ORDER_ARRIVE_MAJYU;
            } else if self.sub == 1 {
                self.state = STATE_DEMO_BOMB_GET;
                self.order = ORDER_BOMB_GET;
            }
        } else if self.has_attention {
            self.state = STATE_TREASURE_WAIT_TALK;
        } else {
            self.order = ORDER_TALK;
        }
    }

    /// `treasure_wait_talk` (`:1025-1033`).
    fn treasure_wait_talk(&mut self, ctx: &mut Ctx) {
        if self.talk(ctx, false) == STATUS_BOX_CLOSED as u16 {
            self.state = STATE_TREASURE_WAIT;
            self.has_attention = false;
            ctx.reset_event();
        }
    }

    /// `demo_arrive` (`:1035-1044`).
    fn demo_arrive(&mut self, ctx: &mut Ctx) {
        if self.end_check(ctx, DEMO_NAMES[4]) {
            // `dKy_instant_timechg(300.0f)` jumps the sky to dusk; the port has no sky.
            ctx.stories.set(EV_0808);
            self.state = STATE_WAIT;
            self.order = ORDER_NONE;
            ctx.reset_event();
        }
    }

    /// `demo_wait_2` (`:756-762`).
    fn demo_wait_2(&mut self, ctx: &mut Ctx) {
        let hio = hio(self.kind);
        if in_circle(
            hio.arrival,
            hio.arrival_radius,
            hio.arrival_height,
            ctx.player,
        ) {
            self.order = ORDER_INTRO_2;
            self.state = STATE_DEMO_INTRO_2;
        }
    }

    /// `demo_intro_2` (`:764-772`).
    fn demo_intro_2(&mut self, ctx: &mut Ctx) {
        if self.end_check(ctx, DEMO_NAMES[5]) {
            ctx.stories.set(EV_1A04);
            self.state = STATE_GOAL_WAIT_2;
            self.order = ORDER_NONE;
            ctx.reset_event();
        }
    }

    /// `goal_wait_2` (`:774-785`).
    fn goal_wait_2(&mut self, ctx: &mut Ctx) {
        let hio = hio(self.kind);
        if ctx.switches.test(self.switch_no)
            && in_circle(hio.bomb_start, hio.bomb_radius, hio.bomb_height, ctx.player)
        {
            self.state = STATE_DEMO_GOAL_2;
        }
    }

    /// `demo_goal_2` (`:787-798`).
    fn demo_goal_2(&mut self, ctx: &mut Ctx) {
        if self.end_check(ctx, DEMO_NAMES[6]) {
            self.state = STATE_TREASURE_WAIT;
            self.order = ORDER_NONE;
            self.big = true;
            self.talk_attn = ctx.cut_attn_pos + self.attn_offset;
            ctx.reset_event();
        } else {
            self.order = ORDER_GOAL_2;
        }
    }

    /// `demo_bomb_get` (`:800-809`).
    fn demo_bomb_get(&mut self, ctx: &mut Ctx) {
        if self.end_check(ctx, DEMO_NAMES[7]) {
            ctx.stories.set(EV_0F02);
            self.state = STATE_WAIT;
            self.order = ORDER_NONE;
            self.big = false;
            ctx.reset_event();
        }
    }

    /// `intro_action` (`:1122-1208`).
    fn intro_action(&mut self, ctx: &mut Ctx) {
        if self.started == ACTION_INIT {
            if self.sub == 0 {
                self.state = if !ctx.stories.test(EV_0720) {
                    STATE_DEMO_WAIT
                } else {
                    STATE_TALKPOS_TO_GOALPOS
                };
            } else {
                self.state = if !ctx.stories.test(EV_1A04) {
                    STATE_DEMO_WAIT_2
                } else {
                    STATE_GOAL_WAIT_2
                };
            }
            self.started += 1;
            return;
        }
        if self.started == ACTION_DONE {
            return;
        }
        self.attn = self.chk_attention(ctx);
        self.order = ORDER_NONE;
        match self.state {
            STATE_WAIT => self.wait01(),
            STATE_TALK => self.talk01(ctx),
            STATE_DEMO_WAIT => self.demo_wait(ctx),
            STATE_DEMO_INTRO => self.demo_intro(ctx),
            STATE_DEMO_LIFT => self.demo_lift(ctx),
            STATE_DEMO_JUMP => self.demo_jump(ctx),
            STATE_GOAL_TALKPOS_WAIT => self.goal_talkpos_wait(ctx),
            STATE_GOAL_TALKPOS_TALK => self.goal_talkpos_talk(ctx),
            STATE_GOALPOS_TO_TALKPOS => self.goal_goalpos_to_talkpos(ctx),
            STATE_TALKPOS_TO_GOALPOS => self.goal_talkpos_to_goalpos(ctx),
            STATE_GOALPOS_WAIT => self.goal_goalpos_wait(ctx),
            STATE_DEMO_GOAL => self.demo_goal(ctx),
            STATE_TREASURE_WAIT => self.treasure_wait(ctx),
            STATE_TREASURE_WAIT_TALK => self.treasure_wait_talk(ctx),
            STATE_DEMO_ARRIVE => self.demo_arrive(ctx),
            STATE_DEMO_WAIT_2 => self.demo_wait_2(ctx),
            STATE_DEMO_INTRO_2 => self.demo_intro_2(ctx),
            STATE_GOAL_WAIT_2 => self.goal_wait_2(ctx),
            STATE_DEMO_GOAL_2 => self.demo_goal_2(ctx),
            STATE_DEMO_BOMB_GET => self.demo_bomb_get(ctx),
            _ => {}
        }
        self.look_back(ctx);
    }

    /// `wait_action` (`:1210-1242`).
    fn wait_action(&mut self, ctx: &mut Ctx) {
        if self.started == ACTION_INIT {
            self.state = if self.kind == TYPE_P2C {
                STATE_MOCCO_WAIT
            } else if self.kind == TYPE_P2A && !ctx.stories.test(EV_0808) {
                STATE_ZUKO_TELESCOPE
            } else {
                STATE_WAIT
            };
            self.started += 1;
            return;
        }
        if self.started == ACTION_DONE {
            return;
        }
        self.attn = self.chk_attention(ctx);
        self.order = ORDER_NONE;
        match self.state {
            STATE_WAIT => self.wait01(),
            STATE_MOCCO_WAIT => self.moccowait(ctx),
            STATE_ZUKO_TELESCOPE => self.zukotelescope(ctx),
            STATE_TALK => self.talk01(ctx),
            _ => {}
        }
        self.look_back(ctx);
    }

    /// `createInit` (`:1567-1664`).
    pub fn create_init(&mut self, ctx: &mut Ctx) {
        if self.kind == TYPE_P2B {
            if self.sub == 0 {
                if !ctx.stories.test(EV_0808) && ctx.stories.test(EV_0720) {
                    ctx.pos = hio(self.kind).goal_start;
                }
            } else if self.sub == 1 && !ctx.stories.test(EV_0F02) && ctx.stories.test(EV_1A04) {
                ctx.pos = hio(self.kind).bomb_start;
            }
        }
        self.home = ctx.pos;
        self.home_angle = ctx.angle;
        // Retail picks `intro_action` only when the stage the save was loaded from is `Asoko`;
        // the port's P2 children ride the pirate ship, whose stage is never `Asoko`, so every
        // child starts on `wait_action` - the same arm retail takes (`:1636-1638`).
        self.action = Action::Wait;
        self.attn_pos = ctx.pos;
        self.eye_pos = ctx.pos;
        self.tex_index = 1;
        self.anm_no = ANM_WAIT;
        self.prev_anm = 0;
        self.tex_pattern = HEAD_TEX.get(usize::from(self.kind)).copied().unwrap_or(0);
        self.set_tex_anm();
        self.set_anm(0.0, -1);
        if let Some((_, ship_angle)) = ctx.ship {
            if self.kind == TYPE_P2A && !ctx.stories.test(EV_0808) {
                self.angle_off = self.home_angle.wrapping_sub(ship_angle);
                self.jnt.on_head_lock();
                self.jnt.on_back_bone_lock();
            } else if self.kind == TYPE_P2C {
                self.angle_off = self.home_angle.wrapping_sub(ship_angle);
            }
        }
        let rng = (super::npc_p1::rnd(&mut self.seed) * RND_SCALE) as i16;
        self.mocco_timer = rng.wrapping_add(RND_TIMER_MIN as i16);
        self.attn_offset = Vec3::ZERO;
        self.set_collision();
    }

    pub fn execute(&mut self, ctx: &mut Ctx) {
        self.play_tex_pattern_anm();
        self.morph.play();
        self.check_order(ctx);
        match self.action {
            Action::Intro => self.intro_action(ctx),
            Action::Wait => self.wait_action(ctx),
        }
        if ctx.event_mode && !ctx.talk_command {
            // `_execute` (`:1285-1311`): `mEventCut.cutProc()` runs first, and only when it has no
            // act for this cut does the private table take over.
            if !self.shared_cut_proc(ctx) {
                self.cut_proc(ctx);
            }
        }
        if !ctx.event_mode {
            self.attn = false;
        }
        self.event_order(ctx);
        self.set_anm(ctx.cut_move_speed, ctx.cut_no);
        self.set_tex_anm();
        self.set_attention(ctx);
        self.set_collision();
    }
}

/// `cLib_calcTimer<s16>`: count down to zero, never past it.
fn cut_timer(timer: &mut i16) -> i16 {
    if *timer > 0 {
        *timer -= 1;
    }
    *timer
}

/// `cLib_chaseS(&value, target, step)` (`d_lib.cpp`).
fn chase_s16(value: &mut i16, target: i16, step: i16) {
    let gap = i32::from(target) - i32::from(*value);
    if gap.abs() <= i32::from(step) {
        *value = target;
    } else if gap > 0 {
        *value = value.wrapping_add(step);
    } else {
        *value = value.wrapping_sub(step);
    }
}

/// `cM_ssin`/`cM_scos`: an s16 angle in radians.
const S16_RAD: f32 = std::f32::consts::PI / 32768.0;

fn s16_sin(angle: i16) -> f32 {
    (f32::from(angle) * S16_RAD).sin()
}

fn s16_cos(angle: i16) -> f32 {
    (f32::from(angle) * S16_RAD).cos()
}

/// `cLib_addCalcAngleS(&angle, target, scale, maxStep, minStep)` (`d_lib.cpp`).
fn add_calc_angle_s16(angle: &mut i16, target: i16, scale: i16, max_step: i16, min_step: i16) {
    let gap = i32::from(target) - i32::from(*angle);
    let mut step = gap / (i32::from(scale) * 2 + 1);
    let abs_step = step.abs();
    if abs_step < i32::from(min_step) {
        step = if i32::from(*angle) > i32::from(target) {
            -i32::from(min_step)
        } else {
            i32::from(min_step)
        };
    }
    if abs_step > i32::from(max_step) {
        step = if i32::from(*angle) > i32::from(target) {
            -i32::from(max_step)
        } else {
            i32::from(max_step)
        };
    }
    *angle = (*angle as i32 + step) as i16;
}

/// `mDoMtx_stack_c::ZXYrotM(x, y, z)` applied to a vector.
fn rotate_zxy(value: Vec3, x: i16, y: i16, z: i16) -> Vec3 {
    let (sin_x, cos_x) = (s16_sin(x), s16_cos(x));
    let (sin_y, cos_y) = (s16_sin(y), s16_cos(y));
    let (sin_z, cos_z) = (s16_sin(z), s16_cos(z));
    let rx = Vec3::new(
        value.x,
        cos_x * value.y - sin_x * value.z,
        sin_x * value.y + cos_x * value.z,
    );
    let ry = Vec3::new(
        cos_y * rx.x + sin_y * rx.z,
        rx.y,
        -sin_y * rx.x + cos_y * rx.z,
    );
    Vec3::new(
        cos_z * ry.x - sin_z * ry.y,
        sin_z * ry.x + cos_z * ry.y,
        ry.z,
    )
}

impl NpcP2 {
    /// `dNpc_EventCut_c::setActorInfo`'s staff name (`d_a_npc_p2.cpp:1611`): `actor_name_table`'s
    /// entry for this actor's own type (`:1568`), which is the name the shared table searches. The
    /// private table is the one that hardcodes `"P2b"` (`d_a_npc_p2_cut.inc:22`).
    fn shared_staff(&self) -> &'static str {
        NPC_P2_ACTORS[usize::from(self.kind)]
    }

    /// `dComIfGp_evmng_cutEnd` on the shared table's staff, not the private table's.
    fn shared_cut_end(&self, ctx: &mut Ctx) {
        let staff = self.shared_staff();
        ctx.cut_end_staff(staff);
    }

    /// `dNpc_EventCut_c::cutProc` (`d_npc_event_cut.inc:10-82`), the table every event NPC shares.
    /// True when it owned this cut, which keeps the private table from running it as well.
    fn shared_cut_proc(&mut self, ctx: &mut Ctx) -> bool {
        let staff = self.shared_staff();
        let Some((cut, advance)) = ctx.active.staff_cut(ctx.list, staff) else {
            return false;
        };
        // A name the shared table does not carry belongs to the private table, which owns the
        // `cutEnd` for it; only the two move/turn/`WAIT` acts below are the shared ones.
        let Some(act) = event_cut::act_index(ctx.list, cut) else {
            return false;
        };
        ctx.cut_no = act as i32;
        if advance {
            self.cut_shared.attention = false;
            if let Some(update) = event_cut::start(&mut self.cut_shared, act, ctx.list, cut) {
                match update {
                    event_cut::TalkUpdate::Msg { curr, end } => {
                        self.msg_no = curr;
                        self.end_msg_no = end;
                    }
                    event_cut::TalkUpdate::Continue { end } => self.end_msg_no = end,
                }
            }
        }
        match act {
            event_cut::WAIT => {
                self.anm_no = ANM_WAIT;
                if event_cut::wait_proc(&mut self.cut_shared) {
                    self.shared_cut_end(ctx);
                }
            }
            event_cut::TURN_TO_ACTOR => self.shared_turn_proc(ctx, true),
            event_cut::TURN_TO_POS => self.shared_turn_proc(ctx, false),
            event_cut::MOVE_TO_ACTOR | event_cut::MOVE_TO_POS => {
                let from = ctx.pos;
                if act == event_cut::MOVE_TO_ACTOR {
                    if !self.shared_goal(ctx, from) {
                        self.shared_cut_end(ctx);
                        return true;
                    }
                }
                let goal = if act == event_cut::MOVE_TO_ACTOR {
                    event_cut::move_to_actor_goal(&self.cut_shared, from)
                } else {
                    event_cut::move_to_pos_goal(&self.cut_shared, from)
                };
                ctx.cut_move_speed = self.cut_shared.speed;
                self.anm_no = if self.cut_shared.speed == 0.0 {
                    ANM_WAIT
                } else if self.cut_shared.speed > CUT_MOVE_FAST {
                    0x05
                } else {
                    ANM_WALK
                };
                if event_cut::move_proc(&mut self.cut_shared, goal, &mut ctx.pos) {
                    self.shared_cut_end(ctx);
                }
            }
            event_cut::TALK_MSG | event_cut::CONTINUE_TALK => self.shared_talk_proc(ctx),
            _ => return false,
        }
        true
    }

    /// `dNpc_EventCut_c::cutTalkMsgProc` (`d_npc_event_cut.inc:600-614`). `mMsgNo` is the range
    /// `cutTalkMsgStart` handed over, which `talk` keeps reading for the whole conversation,
    /// exactly as the private `TALK` act does (`:168`). `mpTalkActor` is always this actor, so the
    /// reference's null test comes down to `field_0x52`.
    fn shared_talk_proc(&mut self, ctx: &mut Ctx) {
        if self.cut_shared.field_0x52 == 0 {
            self.shared_cut_end(ctx);
            return;
        }
        let status = self.talk(ctx, true);
        if status == STATUS_BOX_CLOSED as u16 {
            self.cut_shared.field_0x52 = 0;
            self.shared_cut_end(ctx);
        } else if (status == STATUS_BOX_OPENING as u16 || status == STATUS_TYPING as u16)
            && self.end_msg_no == self.msg_no
        {
            self.end_msg_no = u32::MAX;
            self.shared_cut_end(ctx);
        }
    }

    /// The `TURN_TO_ACTOR`/`TURN_TO_POS` acts (`:174-238`, `:444-476`): the goal, then the
    /// turn itself, which for `P2` is the actor's own yaw rather than a joint lock.
    fn shared_turn_proc(&mut self, ctx: &mut Ctx, to_actor: bool) {
        let from = ctx.pos;
        if to_actor && !self.shared_goal(ctx, from) {
            self.shared_cut_end(ctx);
            return;
        }
        let step = event_cut::turn_proc(&mut self.cut_shared, from, ctx.angle);
        if !self.cut_shared.no_turn {
            let yaw = event_cut::target_angle_y(from, self.cut_shared.pos);
            add_calc_angle_s2(&mut ctx.angle, yaw, TURN_SCALE, TURN_MAX_STEP);
        }
        if step.finished {
            self.shared_cut_end(ctx);
        }
    }

    /// `cutTurnToActorProc`/`cutMoveToActorProc`'s goal selection (`:205-221`).
    fn shared_goal(&mut self, ctx: &mut Ctx, from: Vec3) -> bool {
        event_cut::goal(
            &mut self.cut_shared,
            from,
            ctx.player
                .map(|eye| Vec3::new(eye.x, eye.y + PLAYER_EYE_OFFSET, eye.z)),
            ctx.player_angle,
            &mut |set_id, name, from| ctx.find_target(set_id, name, from),
        )
    }

    /// `cutProc` (`d_a_npc_p2_cut.inc:5-140`). Returns whether the private table owned this cut,
    /// which is what `_execute` tests before applying the shared table's animation override.
    pub fn cut_proc(&mut self, ctx: &mut Ctx) -> bool {
        let Some((cut, advance)) = ctx.staff_cut() else {
            self.cut.active = 0;
            return false;
        };
        let Some(act) = ctx
            .list
            .cut_name(cut)
            .and_then(|name| CUT_ACTIONS.iter().position(|entry| *entry == name))
        else {
            self.cut.active = 0;
            self.shared_cut_end(ctx);
            return false;
        };
        self.cut.active = 1;
        if advance {
            self.cut.attention = false;
            match act {
                CUT_TALK => self.cut_talk_start(cut, ctx),
                CUT_RIDE_SWITCH => self.cut_ride_switch_start(cut, ctx),
                CUT_RUN_WAIT => self.cut_run_wait_start(cut, ctx),
                CUT_JUMP_TO_LIFT => self.cut_jump_to_lift_start(cut, ctx),
                CUT_LIFT_TO_ROPE => self.cut_lift_to_rope_start(cut, ctx),
                CUT_ROPE_TALK => self.cut_rope_talk_start(cut, ctx),
                CUT_ROPE_TO_LIFT => self.cut_rope_to_lift_start(cut, ctx),
                CUT_JUMP_TO_GOAL => self.cut_jump_to_goal_start(cut, ctx),
                CUT_SET_ANM => self.cut_set_anm_start(cut, ctx),
                CUT_JUMP => self.cut_jump_start(cut, ctx),
                CUT_SW_ON => self.cut_sw_on_start(ctx),
                CUT_SW_OFF => self.cut_sw_off_start(ctx),
                _ => {}
            }
        }
        match act {
            CUT_TALK => self.cut_talk_proc(ctx),
            CUT_RIDE_SWITCH => self.cut_ride_switch_proc(ctx),
            CUT_RUN_WAIT => self.cut_run_wait_proc(ctx),
            CUT_JUMP_TO_LIFT => self.cut_jump_to_lift_proc(ctx),
            CUT_LIFT_TO_ROPE => self.cut_lift_to_rope_proc(ctx),
            CUT_ROPE_TALK => self.cut_rope_talk_proc(ctx),
            CUT_ROPE_TO_LIFT => self.cut_rope_to_lift_proc(ctx),
            CUT_JUMP_TO_GOAL => self.cut_jump_to_goal_proc(ctx),
            CUT_SET_ANM => self.cut_set_anm_proc(ctx),
            CUT_JUMP => self.cut_jump_proc(ctx),
            CUT_SW_ON => self.cut_sw_on_proc(ctx),
            CUT_SW_OFF => self.cut_sw_off_proc(ctx),
            CUT_SURPRISE => self.cut_surprise_proc(ctx),
            CUT_OMAMORI_INIT => self.cut_omamori_init_proc(ctx),
            CUT_OMAMORI_END => self.cut_omamori_end_proc(ctx),
            _ => {}
        }
        // `_execute` integrates the actor's speed after the proc and runs `mAcch.CrrPos`
        // (`:1316-1318`); the contact it lands on is what next tick's procs read.
        self.cut_move(ctx);
        self.cut.grounded = self.cut_ground_hit(ctx);
        true
    }

    /// `fopAcM_posMoveF(this, mStts.GetCCMoveP())`: `speedF` along `current.angle.y`, then
    /// `speed.y`, which gravity pulls back down.
    fn cut_move(&mut self, ctx: &mut Ctx) {
        if self.cut.vel_f != 0.0 {
            let yaw = ctx.angle;
            ctx.pos.x += s16_sin(yaw) * self.cut.vel_f;
            ctx.pos.z += s16_cos(yaw) * self.cut.vel_f;
        }
        ctx.pos.y += self.cut.vel_y;
        self.cut.vel_y += self.cut.gravity;
        if let Some(ground) = ctx.ground_y(ctx.pos) {
            if ctx.pos.y < ground {
                ctx.pos.y = ground;
                if self.cut.vel_y < 0.0 {
                    self.cut.vel_y = 0.0;
                }
            }
        }
    }

    /// `mAcch.ChkGroundHit()`: touching the ground right now.
    fn cut_ground_hit(&self, ctx: &Ctx) -> bool {
        let Some(ground) = ctx.ground_y(ctx.pos) else {
            return false;
        };
        ctx.pos.y <= ground + CUT_GROUND_EPS && self.cut.vel_y <= 0.0
    }

    /// `mAcch.ChkGroundLanding()`: contact this tick that was not there last tick.
    fn cut_ground_landing(&self, ctx: &Ctx) -> bool {
        self.cut_ground_hit(ctx) && !self.cut.grounded
    }

    /// `mAcch.m_flags & dBgS_Acch::GROUND_AWAY`: airborne, away from the ground.
    fn cut_ground_away(&self, ctx: &Ctx) -> bool {
        match ctx.ground_y(ctx.pos) {
            Some(ground) => ctx.pos.y > ground + CUT_GROUND_EPS,
            None => true,
        }
    }

    /// `searchNearLift`/`searchNearRope` (`d_a_npc_p2_cut.inc:5B0C`, `:6064`): the nearest actor
    /// under the name, kept on `mClosestPos`/`mActor`. The reference only assigns `mActor` in its
    /// closer branch; the port stores it on the first hit too, so the proc always has a target.
    fn search_near(&mut self, ctx: &Ctx, name: &str) {
        for target in ctx.targets.iter().filter(|target| target.name == name) {
            if self.cut.closest == Vec3::ZERO {
                self.cut.closest = target.pos;
                self.cut.target = Some(target.entity);
            } else if (target.pos - ctx.pos).length() < (self.cut.closest - ctx.pos).length() {
                self.cut.closest = target.pos;
                self.cut.target = Some(target.entity);
            }
        }
    }

    fn cut_target_pos(&self, ctx: &Ctx) -> Vec3 {
        self.cut
            .target
            .and_then(|entity| ctx.targets.iter().find(|target| target.entity == entity))
            .map_or(self.cut.closest, |target| target.pos)
    }

    /// `cutTalkStart` (`:142-156`).
    fn cut_talk_start(&mut self, cut: usize, ctx: &Ctx) {
        self.msg_no = ctx
            .list
            .cut_number(cut, "MsgNum")
            .and_then(|msg| u32::try_from(msg).ok())
            .unwrap_or(0);
        self.cut.attention = ctx.list.cut_number(cut, "Attention").is_some();
        // `talkInit` (`:639-642`).
        self.talk_state = 0;
    }

    /// `cutTalkProc` (`:158-162`).
    fn cut_talk_proc(&mut self, ctx: &mut Ctx) {
        if self.talk(ctx, true) == STATUS_BOX_CLOSED as u16 {
            ctx.cut_end();
        }
    }

    /// `cutRideSwitchStart` (`:164-175`).
    fn cut_ride_switch_start(&mut self, cut: usize, ctx: &Ctx) {
        self.cut.vel_y = ctx.list.cut_float(cut, "Speed_y").unwrap_or(16.0);
        self.cut.gravity = ctx.list.cut_float(cut, "Gravity").unwrap_or(-2.0);
        self.cut.attention = true;
        self.anm_no = 9;
    }

    /// `cutRideSwitchProc` (`:177-201`). The reference's `searchActorAngleY` and the distance to
    /// the switch are computed and never read, so they are left out.
    fn cut_ride_switch_proc(&mut self, ctx: &mut Ctx) {
        let Some((switch, _)) = ctx.target_named("Obj_Swpush") else {
            return;
        };
        if self.anm_no != 10 {
            add_calc2(&mut ctx.pos.x, switch.x, CUT_CHASE_SCALE, CUT_CHASE_STEP);
            add_calc2(&mut ctx.pos.z, switch.z, CUT_CHASE_SCALE, CUT_CHASE_STEP);
        }
        if self.cut_ground_landing(ctx) {
            self.cut.timer = 0x11;
            self.anm_no = 0xA;
        }
        if cut_timer(&mut self.cut.timer) == 0 && self.anm_no == 0xA {
            self.cut.vel_y = 0.0;
            self.cut.gravity = -9.0;
            self.anm_no = 1;
            ctx.cut_end();
        }
    }

    /// `cutRunWaitStart` (`:203-212`).
    fn cut_run_wait_start(&mut self, cut: usize, ctx: &Ctx) {
        self.cut.timer = ctx.list.cut_number(cut, "Timer").unwrap_or(0) as i16;
        self.anm_no = 6;
        self.cut.attention = true;
    }

    /// `cutRunWaitProc` (`:214-222`).
    fn cut_run_wait_proc(&mut self, ctx: &mut Ctx) {
        self.cut.saved_pos = ctx.pos;
        self.cut.saved_angle = [0, ctx.angle, 0];
        if self.anm_no == 6 && self.morph.is_stop() {
            ctx.cut_end();
        }
    }

    /// `cutJumpToLiftStart` (`:269-288`).
    fn cut_jump_to_lift_start(&mut self, cut: usize, ctx: &Ctx) {
        self.cut.speed = ctx.list.cut_float(cut, "Speed").unwrap_or(10.0);
        self.cut.speed_y = ctx.list.cut_float(cut, "Speed_y").unwrap_or(15.0);
        self.cut.gravity = ctx.list.cut_float(cut, "Gravity").unwrap_or(-2.5);
        self.anm_no = 5;
        self.cut.attention = true;
    }

    /// `cutJumpToLiftProc` (`:290-360`). `smoke_set()`/`mSmoke.end()` (`:495-518`) have no
    /// emitter in the port.
    fn cut_jump_to_lift_proc(&mut self, ctx: &mut Ctx) {
        self.search_near(ctx, "Obj_Hlift");
        let max_speed = self.cut.speed;
        let target = self.cut_target_pos(ctx);
        if self.cut_ground_away(ctx) {
            self.cut.vel_y = self.cut.speed_y;
            self.cut.vel_f = self.cut.speed;
            self.anm_no = 8;
        } else if self.cut_ground_hit(ctx) && matches!(self.anm_no, 9 | 10) {
            self.cut.vel_f = 0.0;
            self.cut.attn_offset = CUT_ATTN_LAND;
            self.anm_no = 10;
        } else if self.cut.vel_y <= 0.0 && self.anm_no == 8 {
            self.cut.attn_offset = CUT_ATTN_FALL;
            self.anm_no = 9;
        }
        self.cut.saved_pos = ctx.pos;
        self.cut.saved_angle = [0, ctx.angle, 0];
        if self.cut_ground_hit(ctx) && self.anm_no == 5 {
            self.cut.vel_f = max_speed * 2.0;
        } else if self.cut_ground_hit(ctx) && !matches!(self.anm_no, 10 | 1) {
            add_calc2(&mut ctx.pos.x, target.x, CUT_CHASE_SCALE, max_speed);
            add_calc2(&mut ctx.pos.z, target.z, CUT_CHASE_SCALE, max_speed);
        } else if cut_timer(&mut self.cut.timer) == 0 && self.anm_no == 10 {
            self.anm_no = 1;
            self.cut.attn_offset = Vec3::ZERO;
            self.cut.closest = Vec3::ZERO;
            self.cut.vel_y = 0.0;
            self.cut.gravity = -9.0;
            self.cut.vel_f = 0.0;
            ctx.cut_end();
        }
    }

    /// `cutLiftToRopeStart` (`:388-407`).
    fn cut_lift_to_rope_start(&mut self, cut: usize, ctx: &Ctx) {
        self.cut.speed = ctx.list.cut_float(cut, "Speed").unwrap_or(10.0);
        self.cut.speed_y = ctx.list.cut_float(cut, "Speed_y").unwrap_or(15.0);
        self.cut.gravity = ctx.list.cut_float(cut, "Gravity").unwrap_or(-2.5);
        self.anm_no = 5;
        self.cut.attention = true;
    }

    /// `cutLiftToRopeProc` (`:409-444`). `mCyl.ChkCoHit()` has no actor cylinders in the port, so
    /// the rope grab is the horizontal reach to the rope the search picked.
    fn cut_lift_to_rope_proc(&mut self, ctx: &mut Ctx) {
        self.search_near(ctx, "HIMO3");
        let speed_f = self.cut.speed;
        let rope = self.cut_target_pos(ctx);
        if self.cut_ground_away(ctx) {
            self.cut.vel_y = self.cut.speed_y;
            self.cut.vel_f = self.cut.speed;
            self.anm_no = 8;
        }
        if self.cut_ground_hit(ctx) {
            self.cut.vel_f = speed_f * 2.0;
        }
        if self.cut.target.is_some() && (rope - ctx.pos).length() < CUT_ROPE_GRAB {
            self.cut.gravity = 0.0;
            self.cut.vel_y = 0.0;
            ctx.cut_end();
        }
    }

    /// `cutRopeTalkStart` (`:446-534`).
    fn cut_rope_talk_start(&mut self, cut: usize, ctx: &mut Ctx) {
        self.msg_no = ctx
            .list
            .cut_number(cut, "MsgNum")
            .and_then(|msg| u32::try_from(msg).ok())
            .unwrap_or(0);
        self.cut.attention = ctx.list.cut_number(cut, "Attention").is_some();
        self.anm_no = 0xC;
        self.talk_state = 0;
        self.cut.rope_hung = 1;
        self.search_near(ctx, "HIMO3");
        let rope = self.cut_target_pos(ctx);
        let local = rope - ctx.pos;
        self.cut.rope_dist = local.length();
        if self.cut.rope_dist == 0.0 {
            return;
        }
        self.cut.rope_scale = (2.0 / self.cut.rope_dist).sqrt();
        let half = std::f32::consts::FRAC_PI_2 / self.cut.rope_scale;
        self.cut.rope_limit = CUT_ROPE_LIMIT_START;
        let (sin, cos) = (s16_sin(ctx.angle), s16_cos(ctx.angle));
        let x = cos * local.x - sin * local.z;
        let y = local.y;
        let z = sin * local.x + cos * local.z;
        let mut cross = walk::to_s16((-z).atan2(y));
        if i32::from(cross) > i32::from(self.cut.rope_limit) {
            cross = self.cut.rope_limit;
        } else if i32::from(cross) < -i32::from(self.cut.rope_limit) {
            cross = -self.cut.rope_limit;
        }
        let norm = f32::from(cross) / f32::from(self.cut.rope_limit);
        self.cut.rope_phase = norm.clamp(-1.0, 1.0).asin() / self.cut.rope_scale;
        self.cut.rope_cross = walk::to_s16((-x).atan2((y * y + z * z).sqrt()));
        self.cut.rope_swing = if self.cut.rope_cross > 0 { half } else { -half };
        let hang = (local.length() - 80.0).min(CUT_ROPE_HANG_MAX);
        ctx.pos = rope - local.normalize_or_zero() * hang;
        self.cut.vel_y = 0.0;
        self.cut.gravity = 0.0;
        self.cut.sound_pos = rope;
        self.cut.rope_timer = 0;
    }

    /// `cutRopeTalkProc` (`:536-660`). The head joint `dLib_getPosFromMtx` reads has no model to
    /// come from, and the reference only consumes it for the head transform the port does not
    /// draw, so the swing runs off `mSoundPos` and the two phases alone.
    fn cut_rope_talk_proc(&mut self, ctx: &mut Ctx) {
        let half = std::f32::consts::FRAC_PI_2;
        let mut angle = self.cut.rope_phase * self.cut.rope_scale;
        let mut vel = self.cut.rope_swing * self.cut.rope_scale;
        let mut delta = if (0.0..=half).contains(&angle) {
            -0.45 * angle.sin()
        } else if angle <= -half {
            0.45 * angle.sin()
        } else {
            0.0
        };
        self.cut.rope_phase += delta + 1.0;
        delta = if (0.0..=half).contains(&vel) {
            -0.45 * vel.sin()
        } else if vel <= -half {
            0.45 * vel.sin()
        } else {
            0.0
        };
        self.cut.rope_swing += delta + 1.0;
        angle = self.cut.rope_phase * self.cut.rope_scale;
        vel = self.cut.rope_swing * self.cut.rope_scale;
        if angle >= std::f32::consts::PI {
            angle -= std::f32::consts::TAU;
            self.cut.rope_phase = angle / self.cut.rope_scale;
        }
        if vel >= std::f32::consts::PI {
            vel -= std::f32::consts::TAU;
            self.cut.rope_swing = vel / self.cut.rope_scale;
        }
        chase_s16(&mut self.cut.rope_cross, 0, CUT_ROPE_CROSS_SCALE);
        let grown = f32::from(self.cut.rope_limit) + angle.abs().cos().abs() * 64.0;
        self.cut.rope_limit = grown.min(12000.0) as i16;
        let swing_tilt = (-f32::from(self.cut.rope_limit) * angle.sin()) as i16;
        let cross_tilt = (f32::from(self.cut.rope_cross) * vel.sin()) as i16;
        let x_target =
            (-f32::from(self.cut.rope_limit) * (angle - CUT_ROPE_X_LEAD.to_radians()).sin()) as i16;
        add_calc_angle_s16(&mut self.cut.tilt_x, x_target, 8, 0xC00, 0x100);
        let cos_vel = vel.cos();
        let z_target =
            (f32::from(self.cut.rope_cross) * (vel - CUT_ROPE_Z_LEAD.to_radians()).sin()) as i16;
        let z_max = (cos_vel * 2048.0 + 1024.0) as i16;
        let z_min = (cos_vel * 128.0 + 128.0) as i16;
        add_calc_angle_s16(&mut self.cut.tilt_z, z_target, 8, z_max, z_min);
        let hang = Vec3::new(0.0, -self.cut.rope_dist, 0.0);
        ctx.pos = self.cut.sound_pos + rotate_zxy(hang, swing_tilt, ctx.angle, cross_tilt);
        // `mDoAud_seStart(0x201E, &mSoundPos)` has no sound table entry in the port.
        if self.talk(ctx, true) == STATUS_BOX_CLOSED as u16
            && self.cut.rope_swing > 0.0
            && self.cut.rope_phase > 0.0
            && self.anm_no == 0xB
        {
            self.anm_no = 9;
            self.cut.rope_hung = 0;
            ctx.cut_end();
        } else if self.cut.rope_swing > 0.0 && self.cut.rope_phase > 0.0 {
            self.anm_no = 0xC;
        } else if self.cut.rope_swing > 0.0 || self.cut.rope_phase > 0.0 {
            self.anm_no = 0xB;
        } else {
            self.anm_no = 0xC;
        }
    }

    /// `cutRopeToLiftStart` (`:662-688`).
    fn cut_rope_to_lift_start(&mut self, cut: usize, ctx: &Ctx) {
        self.cut.speed = ctx.list.cut_float(cut, "Speed").unwrap_or(10.0);
        self.cut.speed_y = ctx.list.cut_float(cut, "Speed_y").unwrap_or(6.0);
        self.cut.gravity = ctx.list.cut_float(cut, "Gravity").unwrap_or(-0.5);
        self.cut.tilt_x = 0;
        self.cut.tilt_z = 0;
        self.cut.attention = true;
        self.anm_no = 9;
        self.cut.vel_y = self.cut.speed_y;
        self.cut.vel_f = self.cut.speed;
    }

    /// `cutRopeToLiftProc` (`:690-724`).
    fn cut_rope_to_lift_proc(&mut self, ctx: &mut Ctx) {
        self.search_near(ctx, "Obj_Hlift");
        self.cut.saved_pos = ctx.pos;
        self.cut.saved_angle = [0, ctx.angle, 0];
        if self.cut_ground_landing(ctx) {
            self.cut.timer = 0x11;
        }
        if self.cut_ground_hit(ctx) && matches!(self.anm_no, 9 | 10) {
            self.cut.vel_f = 0.0;
            self.cut.vel_y = 0.0;
            self.cut.gravity = -9.0;
            self.cut.attn_offset = CUT_ATTN_LAND;
            self.anm_no = 10;
        }
        if cut_timer(&mut self.cut.timer) == 0 && self.anm_no == 10 {
            self.anm_no = 1;
            self.cut.attn_offset = Vec3::ZERO;
            self.cut.closest = Vec3::ZERO;
            ctx.cut_end();
        }
    }

    /// `cutJumpToGoalStart` (`:726-762`). The `Pos` default is `l_HIO.mChild[mType].m30`, an
    /// offset the port's HIO table does not carry, so the cut's own `Pos` is required.
    fn cut_jump_to_goal_start(&mut self, cut: usize, ctx: &Ctx) {
        self.cut.goal = ctx.list.cut_vec(cut, "Pos").unwrap_or(Vec3::ZERO);
        self.cut.speed = ctx.list.cut_float(cut, "Speed").unwrap_or(10.0);
        self.cut.speed_y = ctx.list.cut_float(cut, "Speed_y").unwrap_or(15.0);
        self.cut.gravity = ctx.list.cut_float(cut, "Gravity").unwrap_or(-2.5);
        self.anm_no = 5;
        self.cut.attention = true;
    }

    /// `cutJumpToGoalProc` (`:764-812`).
    fn cut_jump_to_goal_proc(&mut self, ctx: &mut Ctx) {
        let goal = self.cut.goal;
        let max_speed = self.cut.speed;
        if self.cut_ground_away(ctx) {
            self.cut.vel_y = self.cut.speed_y;
            self.cut.vel_f = self.cut.speed;
            self.anm_no = 8;
        }
        if self.cut_ground_hit(ctx) && !matches!(self.anm_no, 10 | 1) {
            add_calc2(&mut ctx.pos.x, goal.x, CUT_CHASE_SCALE, max_speed);
            add_calc2(&mut ctx.pos.z, goal.z, CUT_CHASE_SCALE, max_speed);
        }
        if self.cut_ground_hit(ctx) && matches!(self.anm_no, 9 | 10) {
            self.cut.attn_offset = CUT_ATTN_LAND;
            self.anm_no = 10;
        } else if self.cut.vel_y <= 0.0 && self.anm_no == 8 {
            self.cut.attn_offset = CUT_ATTN_FALL;
            self.anm_no = 9;
        }
        if self.cut_ground_landing(ctx) {
            self.cut.timer = 0x11;
            self.cut.vel_f = 0.0;
        }
        self.cut.saved_pos = ctx.pos;
        self.cut.saved_angle = [0, ctx.angle, 0];
        if cut_timer(&mut self.cut.timer) == 0 && self.anm_no == 10 {
            self.cut.attn_offset = Vec3::ZERO;
            self.cut.vel_y = 0.0;
            self.cut.gravity = -9.0;
            self.cut.vel_f = 0.0;
            ctx.cut_end();
            self.anm_no = 1;
        }
    }

    /// `cutJumpStart` (`:814-847`).
    fn cut_jump_start(&mut self, cut: usize, ctx: &Ctx) {
        self.cut.goal = ctx.list.cut_vec(cut, "Pos").unwrap_or(Vec3::ZERO);
        self.cut.speed = ctx.list.cut_float(cut, "Speed").unwrap_or(10.0);
        self.cut.speed_y = ctx.list.cut_float(cut, "Speed_y").unwrap_or(15.0);
        self.cut.gravity = ctx.list.cut_float(cut, "Gravity").unwrap_or(-2.5);
        self.cut.attention = true;
        self.cut.vel_y = self.cut.speed_y;
        self.cut.vel_f = self.cut.speed;
        self.anm_no = 8;
    }

    /// `cutJumpProc` (`:849-897`).
    fn cut_jump_proc(&mut self, ctx: &mut Ctx) {
        if self.cut_ground_hit(ctx) && matches!(self.anm_no, 9 | 10) {
            self.cut.attn_offset = CUT_ATTN_LAND;
            self.anm_no = 10;
        } else if self.cut.vel_y <= 0.0 && self.anm_no == 8 {
            self.cut.attn_offset = CUT_ATTN_FALL;
            self.anm_no = 9;
        }
        if self.cut_ground_landing(ctx) {
            self.cut.timer = 0x11;
            self.cut.vel_f = 0.0;
        }
        self.cut.saved_pos = ctx.pos;
        self.cut.saved_angle = [0, ctx.angle, 0];
        if cut_timer(&mut self.cut.timer) == 0 && self.anm_no == 10 {
            self.cut.attn_offset = Vec3::ZERO;
            self.cut.vel_y = 0.0;
            self.cut.gravity = -9.0;
            self.cut.vel_f = 0.0;
            ctx.cut_end();
            self.anm_no = 1;
        }
    }

    /// `cutSetAnmStart` (`:899-918`).
    fn cut_set_anm_start(&mut self, cut: usize, ctx: &Ctx) {
        self.anm_no = match ctx.list.cut_text(cut, "Name") {
            Some("KYORO") => 0x11,
            Some("SURPRISE") => 7,
            Some("THINK") => 0x12,
            Some("NOD") => 0x13,
            _ => 1,
        };
    }

    /// `cutSetAnmProc` (`:920-924`).
    fn cut_set_anm_proc(&mut self, ctx: &mut Ctx) {
        if self.morph.is_stop() {
            ctx.cut_end();
        }
    }

    /// `cutSwOnStart` (`:926-930`): `fopAcM_onSwitch(this, mSwitchNo)`.
    fn cut_sw_on_start(&mut self, ctx: &mut Ctx) {
        if self.switch_no != 0xFF {
            ctx.switches.set(self.switch_no);
        }
    }

    /// `cutSwOnProc` (`:932-942`).
    fn cut_sw_on_proc(&mut self, ctx: &mut Ctx) {
        if self.switch_no != 0xFF {
            ctx.switches.set(self.switch_no);
        }
        ctx.cut_end();
    }

    /// `cutSwOffStart` (`:944-948`): `fopAcM_offSwitch(this, mSwitchNo)`.
    fn cut_sw_off_start(&mut self, ctx: &mut Ctx) {
        if self.switch_no != 0xFF {
            ctx.switches.clear(self.switch_no);
        }
    }

    /// `cutSwOffProc` (`:950-956`).
    fn cut_sw_off_proc(&mut self, ctx: &mut Ctx) {
        if self.switch_no != 0xFF {
            ctx.switches.clear(self.switch_no);
        }
        ctx.cut_end();
    }

    /// `cutSurpriseProc` (`:964-968`). `cutSurpriseStart` (`:958-962`) only starts two sounds
    /// (`JA_SE_ITM_OMAMORI_BLINK` and the player's voice), which the port has no table for.
    fn cut_surprise_proc(&mut self, ctx: &mut Ctx) {
        ctx.cut_end();
    }

    /// `cutOmamoriInitProc` (`:976-980`); its start only starts `JA_SE_ITM_OMAMORI_TETLA`.
    fn cut_omamori_init_proc(&mut self, ctx: &mut Ctx) {
        ctx.cut_end();
    }

    /// `cutOmamoriEndProc` (`:988-992`); its start only starts `JA_SE_ITM_OMAMORI_ED_TALK`.
    fn cut_omamori_end_proc(&mut self, ctx: &mut Ctx) {
        ctx.cut_end();
    }
}

/// `dLib_setFirstMsg` (`d_lib.cpp`): raise the bit and answer the first message, or the second
/// when the bit was already up.
fn set_first_msg(stories: &mut StoryBits, bit: u16, first: u32, second: u32) -> u32 {
    if !stories.test(bit) {
        stories.set(bit);
        first
    } else {
        second
    }
}

/// `dLib_checkPlayerInCircle(center, radius, halfHeight)` (`d_lib.cpp:269-272`) laid over
/// `dLib_checkActorInCircle`: inside the XZ radius and inside the vertical half height.
fn in_circle(center: Vec3, radius: f32, half_height: f32, point: Option<Vec3>) -> bool {
    let Some(point) = point else {
        return false;
    };
    let dx = point.x - center.x;
    let dz = point.z - center.z;
    dx * dx + dz * dz < radius * radius && (point.y - center.y).abs() < half_height
}

/// `cLib_addCalc2`: approach the target by `scale` of the gap, never more than `max_step`.
fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    let diff = (target - *value) * scale;
    if diff > max_step {
        *value += max_step;
    } else if diff < -max_step {
        *value -= max_step;
    } else {
        *value = target;
    }
}

fn abs_xz(value: Vec3) -> f32 {
    (value.x * value.x + value.z * value.z).sqrt()
}

fn player_eye(ctx: &Ctx, offset: f32) -> Vec3 {
    let mut eye = ctx.player.unwrap_or(Vec3::ZERO);
    eye.y += PLAYER_EYE_OFFSET + offset;
    eye
}

pub struct Ctx<'a> {
    pub entity: Entity,
    pub parameters: u32,
    pub pos: Vec3,
    pub angle: i16,
    pub room: i8,
    pub player: Option<Vec3>,
    /// `dNpc_EventCut_c`'s `Player` target: the player's `shape_angle.y` (`:216`), which the
    /// shared turn acts aim at.
    pub player_angle: i16,
    pub event_mode: bool,
    pub talk_command: bool,
    pub cut_attn: bool,
    pub cut_attn_pos: Vec3,
    pub cut_attn_no_turn: bool,
    pub cut_move_speed: f32,
    pub cut_no: i32,
    /// The pirate ship the child was created on, which both deck states steer against.
    pub ship: Option<(Entity, i16)>,
    /// The `Zl1` the telescope child turns to (`:425-433`).
    pub zl1: Option<Vec3>,
    pub stories: &'a mut StoryBits,
    pub switches: &'a mut Switches,
    pub list: &'a EventList,
    pub active: &'a mut ActiveEvent,
    pub text: &'a mut Text<'a>,
    /// `dEvDtFlag_c`, the store `cutEnd` raises the cut's end flag in.
    pub flags: &'a mut super::flags::EventFlags,
    /// The stage's collision, the cut table's stand-in for `mAcch` (see the module note).
    pub collision: Option<&'a super::collision::Collision>,
    /// Every other actor, snapshotted once per tick for the cut table's nearest-actor searches
    /// and `fopAcM_SearchByName`.
    pub targets: &'a [CutTarget<'a>],
}

/// One non-`NpcP2` actor, snapshotted once per tick for `searchNearLift`/`searchNearRope`
/// (`d_a_npc_p2_cut.inc:5B0C`, `:6064`) and `fopAcM_SearchByName` (`:56F4`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CutTarget<'a> {
    pub name: &'a str,
    pub entity: Entity,
    /// `fopAcM_GetSetId`, which `findActorCallBack` matches before a name (`:346-388`).
    pub set_id: u16,
    pub pos: Vec3,
    pub angle: i16,
}

impl Ctx<'_> {
    /// `dNpc_EventCut_c::findActorCallBack` (`d_npc_event_cut.inc:346-388`): a `SetId` match
    /// wins outright, otherwise the nearest actor carrying the name.
    fn find_target(&self, set_id: u16, name: Option<&str>, from: Vec3) -> Option<(Vec3, i16)> {
        let mut best: Option<(Vec3, i16)> = None;
        for target in self.targets {
            if set_id != 0 && target.set_id == set_id {
                return Some((target.pos, target.angle));
            }
            let Some(name) = name else { continue };
            if target.name != name {
                continue;
            }
            match best {
                None => best = Some((target.pos, target.angle)),
                Some((pos, _)) => {
                    if (target.pos - from).length() < (pos - from).length() {
                        best = Some((target.pos, target.angle));
                    }
                }
            }
        }
        best
    }

    fn reset_event(&mut self) {
        self.text.info.event_mode = DA_EVT_MODE_NONE;
    }

    /// `dComIfGp_evmng_getMyStaffId("P2b", NULL, 0)` plus the act index, i.e. the pair
    /// `cutProc` reads (`d_a_npc_p2_cut.inc:22-30`).
    fn staff_cut(&self) -> Option<(usize, bool)> {
        self.active.staff_cut(self.list, STAFF_P2B)
    }

    /// `dComIfGp_evmng_cutEnd(staffIdx)`.
    fn cut_end(&mut self) {
        self.active.end_staff_cut(self.list, self.flags, STAFF_P2B);
    }

    /// The same `cutEnd` for a caller-owned staff name: the shared table ends the cut under the
    /// actor's own type name (`d_a_npc_p2.cpp:1568`, `:1611`), not the private table's `"P2b"`.
    fn cut_end_staff(&mut self, staff: &str) {
        self.active.end_staff_cut(self.list, self.flags, staff);
    }

    /// `fopAcM_SearchByName` (`:56F4`), the first actor under the name.
    fn target_named(&self, name: &str) -> Option<(Vec3, i16)> {
        self.targets
            .iter()
            .find(|target| target.name == name)
            .map(|target| (target.pos, target.angle))
    }

    /// The cut table's ground query, standing in for `mAcch.ChkGroundHit()` (see the module note).
    fn ground_y(&self, pos: Vec3) -> Option<f32> {
        self.collision
            .and_then(|collision| collision.ground_at(pos.x, pos.z))
    }
}

#[derive(SystemParam)]
pub struct World<'w> {
    pub stories: ResMut<'w, StoryBits>,
    pub active: ResMut<'w, ActiveEvent>,
    pub flags: ResMut<'w, super::flags::EventFlags>,
}

pub fn control(
    do_intent: Res<DoIntent>,
    list: Res<EventList>,
    mut switches: ResMut<Switches>,
    collision: Option<Res<super::collision::Collision>>,
    mut msg: Msg,
    mut world: World<'_>,
    mut actors: Query<(Entity, &mut Actor, &mut NpcP2), (With<NpcP2>, Without<Player>)>,
    others: Query<(Entity, &Actor), (Without<NpcP2>, Without<Player>)>,
    mut commands: Commands,
) {
    let player = msg.players.iter().next().copied();
    let player_pos = player.map(|link| link.pos);
    let event_mode = msg.info.event_run_check();
    let clear_count = msg.text_flags.clear_count;
    let archive = msg
        .archive
        .as_deref()
        .map(|held| held as &dyn crate::text::msgget::MsgArchive);
    let ship = others
        .iter()
        .find(|(_, other)| super::pirateship::is_pirateship(&other.name))
        .map(|(found, other)| (found, other.angle[1]));
    let zl1 = others
        .iter()
        .find(|(_, other)| other.name == "Zl1")
        .map(|(_, other)| other.pos);
    let targets: Vec<CutTarget> = others
        .iter()
        .map(|(entity, other)| CutTarget {
            name: other.name.as_str(),
            entity,
            set_id: other.set_id,
            pos: other.pos,
            angle: other.angle[1],
        })
        .collect();
    for (entity, mut actor, mut npc) in &mut actors {
        let actor = &mut *actor;
        let npc = &mut *npc;
        let pressed = do_intent.0 && !npc.do_held;
        npc.do_held = do_intent.0;
        let mut text = Text {
            procs: &mut msg.procs,
            info: &mut msg.info,
            flags: &mut msg.flags,
            one: &mut msg.one,
            two: &mut msg.two,
            clear_count,
            archive,
        };
        let mut ctx = Ctx {
            entity,
            parameters: actor.parameters,
            pos: actor.pos,
            angle: actor.angle[1],
            room: actor.room,
            player: player_pos,
            player_angle: player.map_or(0, |link| walk::to_s16(link.yaw)),
            event_mode,
            talk_command: npc.talk_order_ready && pressed,
            cut_attn: npc.attn,
            cut_attn_pos: npc.attn_pos,
            cut_attn_no_turn: false,
            cut_move_speed: 0.0,
            cut_no: -1,
            ship,
            zl1,
            stories: &mut world.stories,
            switches: &mut switches,
            list: &list,
            active: &mut world.active,
            text: &mut text,
            flags: &mut world.flags,
            collision: collision.as_deref(),
            targets: &targets,
        };
        if !npc.entered {
            npc.entered = true;
            npc.create_init(&mut ctx);
            if !npc.drawn() {
                commands.entity(entity).despawn();
                continue;
            }
        } else {
            npc.execute(&mut ctx);
        }
        actor.pos = ctx.pos;
        actor.angle[1] = ctx.angle;
    }
}
