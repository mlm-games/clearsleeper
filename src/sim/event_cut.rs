//! `dNpc_EventCut_c`, the cut table every event-driven NPC shares
//! (`tww/src/d/d_npc_event_cut.inc`).
//!
//! The seven acts were first ported inside `npc_hi1`, the one actor that exercises all of them.
//! They live here now so `P2` and the rest of the NPCs run the same act code - the same
//! `getParam` parsing, the same `cLib_targetAngleY` goal selection and the same
//! `cLib_addCalc2` movement - instead of carrying private copies of it.

use glam::Vec3;

use super::event::EventList;
use super::walk;

/// The `action_table` order (`d_npc_event_cut.inc:11-19`).
pub const ACTIONS: [&str; 7] = [
    "WAIT",
    "TURN_TO_ACTOR",
    "MOVE_TO_ACTOR",
    "TURN_TO_POS",
    "MOVE_TO_POS",
    "TALK_MSG",
    "CONTINUE_TALK",
];

pub const WAIT: usize = 0; // <- :13
pub const TURN_TO_ACTOR: usize = 1; // <- :14
pub const MOVE_TO_ACTOR: usize = 2; // <- :15
pub const TURN_TO_POS: usize = 3; // <- :16
pub const MOVE_TO_POS: usize = 4; // <- :17
pub const TALK_MSG: usize = 5; // <- :18
pub const CONTINUE_TALK: usize = 6; // <- :19

/// `cutProc`'s miss on the action table.
pub const NO_ACT: usize = usize::MAX;

/// `dComIfGp_evmng_getMyStaffId` miss (`:27`).
pub const NO_TARGET: i32 = -1;

const TURN_TYPE_DEFAULT: i32 = 1; // <- cutTurnToActorStart :127
const TIMER_DEFAULT: i32 = 1; // <- cutTurnToActorStart :135
const TIMER_WAIT_DEFAULT: i32 = 0; // <- cutWaitStart :91
const SPEED_DEFAULT: f32 = 2.0; // <- cutMoveToPosStart :509
const STABLE_ANGLE: f32 = 5376.0; // <- std::fabsf(temp2) < 5376.0f, :466
const MOVE_SCALE: f32 = 0.1; // <- cLib_addCalc2 scale, :548
const MOVE_DONE: f32 = 5.0; // <- dist < 5.0f, :546
const END_MSG_NONE: u32 = u32::MAX; // <- mEndMsgNo = -1, :582

/// `mTurnType`'s joint handling (`:444-476`). The caller owns the joints, so the act only says
/// which lock the cut asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnLock {
    /// `case 2` (`:452`): the head tracks the target, the backbone is released.
    Head,
    /// `case 0` (`:457`): the backbone is locked.
    Backbone,
    /// The default arm (`:459`): both are released.
    Off,
}

/// One tick of `cutTurnToActorProc`/`cutTurnToPosProc` (`:174-238`, `:444-476`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurnStep {
    pub finished: bool,
    pub lock: TurnLock,
}

/// What a talk act's start hands back for the actor's own message state: `TALK_MSG` names both
/// ends of the range, `CONTINUE_TALK` only moves the end (`:557-597`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TalkUpdate {
    Msg { curr: u32, end: u32 },
    Continue { end: u32 },
}

/// `dNpc_EventCut_c` state (`d_npc.h:120-137`).
#[derive(Debug, Clone, PartialEq)]
pub struct EventCut {
    pub timer: i32,
    pub turn_speed: i16,
    pub add_angle: i16,
    pub del_distance: f32,
    pub attention: bool,
    pub no_turn: bool,
    pub turn_type: u32,
    pub set_id: u16,
    pub offset_pos: Vec3,
    pub speed: f32,
    pub pos: Vec3,
    pub target_angle: i16,
    pub field_0x44: f32,
    pub actor_name: Option<String>,
    pub field_0x52: u8,
}

impl EventCut {
    /// `dNpc_EventCut_c`'s constructor (`d_npc.h:139-160`).
    pub fn create() -> Self {
        Self {
            timer: 0,
            turn_speed: 0,
            add_angle: 0,
            del_distance: 0.0,
            attention: false,
            no_turn: false,
            turn_type: 0,
            set_id: 0,
            offset_pos: Vec3::ZERO,
            speed: 0.0,
            pos: Vec3::ZERO,
            target_angle: 0,
            field_0x44: 0.0,
            actor_name: None,
            field_0x52: 0,
        }
    }
}

impl Default for EventCut {
    fn default() -> Self {
        Self::create()
    }
}

/// `cutProc`'s lookup of the action table (`:22-30`): the cut's own name picks the act.
pub fn act_index(list: &EventList, cut: usize) -> Option<usize> {
    let name = list.cut_name(cut)?;
    ACTIONS.iter().position(|entry| *entry == name)
}

/// Every act's start (`d_npc_event_cut.inc:85-93`, `:103-171`, `:241-299`, `:391-441`,
/// `:479-528`, `:557-584`, `:587-597`). `advance` gated the caller, so this runs once.
pub fn start(cut: &mut EventCut, act: usize, list: &EventList, index: usize) -> Option<TalkUpdate> {
    match act {
        WAIT => {
            cut.timer = list
                .cut_number(index, "Timer")
                .unwrap_or(TIMER_WAIT_DEFAULT)
        }
        TURN_TO_ACTOR => turn_to_actor_start(cut, list, index),
        MOVE_TO_ACTOR => move_to_actor_start(cut, list, index),
        TURN_TO_POS => turn_to_pos_start(cut, list, index),
        MOVE_TO_POS => move_to_pos_start(cut, list, index),
        TALK_MSG => return talk_msg_start(cut, list, index),
        CONTINUE_TALK => return continue_talk_start(list, index),
        _ => {}
    }
    None
}

/// `cutTurnToActorStart` (`:103-171`).
fn turn_to_actor_start(cut: &mut EventCut, list: &EventList, index: usize) {
    cut.actor_name = list
        .cut_text(index, "ActorName")
        .filter(|name| *name != "Player")
        .map(str::to_owned);
    cut.set_id = list.cut_number(index, "SetId").unwrap_or(0).max(0) as u16;
    cut.offset_pos = list.cut_vec(index, "OffsetPos").unwrap_or(Vec3::ZERO);
    cut.turn_type = list
        .cut_number(index, "TurnType")
        .unwrap_or(TURN_TYPE_DEFAULT)
        .max(0) as u32;
    cut.timer = list.cut_number(index, "Timer").unwrap_or(TIMER_DEFAULT);
    cut.turn_speed = list.cut_number(index, "TurnSpeed").unwrap_or(0) as i16;
    cut.add_angle = list.cut_number(index, "AddAngle").unwrap_or(0) as i16;
    cut.del_distance = list.cut_float(index, "DelDistance").unwrap_or(0.0);
    cut.no_turn = list.cut_number(index, "NoTurn").is_some();
    cut.field_0x44 = 0.0;
}

/// `cutMoveToActorStart` (`:241-299`).
fn move_to_actor_start(cut: &mut EventCut, list: &EventList, index: usize) {
    cut.actor_name = list
        .cut_text(index, "ActorName")
        .filter(|name| *name != "Player")
        .map(str::to_owned);
    cut.no_turn = list.cut_number(index, "NoTurn").is_some();
    cut.attention = list.cut_number(index, "Attention").is_some();
    cut.offset_pos = list.cut_vec(index, "OffsetPos").unwrap_or(Vec3::ZERO);
    cut.set_id = list.cut_number(index, "SetId").unwrap_or(0).max(0) as u16;
    cut.speed = list.cut_float(index, "Speed").unwrap_or(SPEED_DEFAULT);
    cut.del_distance = list.cut_float(index, "DelDistance").unwrap_or(0.0);
    // the decomp names this parameter `ShapeAngle` (`:249`) and still stores it in `mAddAngle`
    cut.add_angle = list.cut_number(index, "ShapeAngle").unwrap_or(0) as i16;
}

/// `cutTurnToPosStart` (`:391-441`).
fn turn_to_pos_start(cut: &mut EventCut, list: &EventList, index: usize) {
    cut.pos = list.cut_vec(index, "Pos").unwrap_or(Vec3::ZERO);
    cut.turn_type = list
        .cut_number(index, "TurnType")
        .unwrap_or(TURN_TYPE_DEFAULT)
        .max(0) as u32;
    cut.timer = list.cut_number(index, "Timer").unwrap_or(TIMER_DEFAULT);
    cut.turn_speed = list.cut_number(index, "TurnSpeed").unwrap_or(0) as i16;
    cut.no_turn = list.cut_number(index, "NoTurn").is_some();
    cut.add_angle = list.cut_number(index, "AddAngle").unwrap_or(0) as i16;
    cut.field_0x44 = 0.0;
}

/// `cutMoveToPosStart` (`:479-528`).
fn move_to_pos_start(cut: &mut EventCut, list: &EventList, index: usize) {
    cut.no_turn = list.cut_number(index, "NoTurn").is_some();
    cut.pos = list.cut_vec(index, "Pos").unwrap_or(Vec3::ZERO);
    cut.attention = list.cut_number(index, "Attention").is_some();
    cut.speed = list.cut_float(index, "Speed").unwrap_or(SPEED_DEFAULT);
    cut.del_distance = list.cut_float(index, "DelDistance").unwrap_or(0.0);
    cut.add_angle = list.cut_number(index, "AddAngle").unwrap_or(0) as i16;
}

/// `cutTalkMsgStart` (`:557-584`). Both talk acts are wrapped in `if(mpTalkActor != 0)` in the
/// decomp; every actor that runs these acts is its own talk actor here (`npc_hi1`, `npc_nz`,
/// `npc_p2`), so the guard has no condition to fail and is not modelled.
fn talk_msg_start(cut: &mut EventCut, list: &EventList, index: usize) -> Option<TalkUpdate> {
    cut.attention = list.cut_number(index, "Attention").is_some();
    let curr = list
        .cut_number(index, "msg_num")
        .map(|no| no as u32)
        .unwrap_or(0);
    let end = list
        .cut_number(index, "end_msg")
        .map(|no| no as u32)
        .unwrap_or(END_MSG_NONE);
    cut.field_0x52 = 1;
    Some(TalkUpdate::Msg { curr, end })
}

/// `cutContinueTalkStart` (`:587-597`). `field_0x52` is `cutTalkMsgStart`'s write alone, so a
/// continue-talk that was not preceded by a talk is `cutTalkMsgProc`'s immediate `cutEnd` (`:601`).
fn continue_talk_start(list: &EventList, index: usize) -> Option<TalkUpdate> {
    let end = list
        .cut_number(index, "end_num")
        .map(|no| no as u32)
        .unwrap_or(END_MSG_NONE);
    Some(TalkUpdate::Continue { end })
}

/// `cutWaitProc` (`:96-100`). True once the timer has run out.
pub fn wait_proc(cut: &mut EventCut) -> bool {
    if cut.timer > 0 {
        cut.timer -= 1;
    }
    cut.timer == 0
}

/// The goal selection `cutTurnToActorProc` and `cutMoveToActorProc` share (`:205-221`, `:302`).
/// `:216` reads the target's `shape_angle.y`; the sim only carries the current angle.
/// False means the target was not in the stage, which is `cutEnd`.
pub fn goal(
    cut: &mut EventCut,
    from: Vec3,
    player_eye: Option<Vec3>,
    player_angle: i16,
    finder: &mut dyn FnMut(u16, Option<&str>, Vec3) -> Option<(Vec3, i16)>,
) -> bool {
    if cut.actor_name.is_none() {
        cut.target_angle = player_angle;
        cut.pos = player_eye.unwrap_or(Vec3::ZERO) + cut.offset_pos;
        return true;
    }
    let name = cut.actor_name.clone();
    let Some((pos, angle)) = finder(cut.set_id, name.as_deref(), from) else {
        cut.pos = Vec3::ZERO;
        return false;
    };
    cut.target_angle = angle;
    cut.pos = if cut.add_angle != 0 && cut.del_distance != 0.0 {
        let angle = angle.wrapping_add(cut.add_angle);
        Vec3::new(
            pos.x - cut.del_distance * walk::from_s16(angle).sin(),
            pos.y,
            pos.z - cut.del_distance * walk::from_s16(angle).cos(),
        )
    } else {
        pos + cut.offset_pos
    };
    true
}

/// `cutTurnToActorProc` / `cutTurnToPosProc` (`:174-238` / `:444-476`), after `goal` has set
/// `cut.pos`. The caller owns the joints and the `cutEnd`.
pub fn turn_proc(cut: &mut EventCut, from: Vec3, angle: i16) -> TurnStep {
    let t_angle = target_angle_y(from, cut.pos);
    let lock = match cut.turn_type {
        2 => TurnLock::Head,
        0 => TurnLock::Backbone,
        _ => TurnLock::Off,
    };
    let temp2 = (t_angle.wrapping_sub(angle) as f32).abs();
    if cut.field_0x44 == temp2 && temp2 < STABLE_ANGLE && cut.timer > 0 {
        cut.timer -= 1;
    }
    let finished = cut.timer == 0;
    if finished {
        cut.turn_speed = 0;
    }
    cut.field_0x44 = (t_angle.wrapping_sub(angle) as f32).abs();
    TurnStep { finished, lock }
}

/// `cutMoveToActorProc`'s goal (`:302-321`): the target pulled back by `DelDistance` along
/// `AddAngle`, or along the direction of travel when the cut names no angle.
pub fn move_to_actor_goal(cut: &EventCut, from: Vec3) -> Vec3 {
    let angle = if cut.add_angle != 0 {
        cut.target_angle.wrapping_add(cut.add_angle)
    } else {
        atan2s(cut.pos.x - from.x, cut.pos.z - from.z)
    };
    cut.pos
        - Vec3::new(
            cut.del_distance * walk::from_s16(angle).sin(),
            0.0,
            cut.del_distance * walk::from_s16(angle).cos(),
        )
}

/// `cutMoveToPosProc`'s goal (`:531-543`).
pub fn move_to_pos_goal(cut: &EventCut, from: Vec3) -> Vec3 {
    let angle = target_angle_y(from, cut.pos);
    cut.pos
        - Vec3::new(
            cut.del_distance * walk::from_s16(angle).sin(),
            0.0,
            cut.del_distance * walk::from_s16(angle).cos(),
        )
}

/// The step both move acts run (`:322-343`, `:543-554`). True once the actor has arrived, which
/// is the caller's `cutEnd`.
pub fn move_proc(cut: &mut EventCut, goal: Vec3, pos: &mut Vec3) -> bool {
    if cut.speed == 0.0 {
        *pos = goal;
        return true;
    }
    let dist = abs_xz(goal - *pos);
    add_calc2(&mut pos.x, goal.x, MOVE_SCALE, cut.speed);
    add_calc2(&mut pos.z, goal.z, MOVE_SCALE, cut.speed);
    if dist < MOVE_DONE {
        cut.speed = 0.0;
        return true;
    }
    false
}

/// `cLib_calcTimer<i32>`: count down to zero, never past it.
pub fn calc_timer(timer: &mut i32) -> i32 {
    if *timer > 0 {
        *timer -= 1;
    }
    *timer
}

fn abs_xz(value: Vec3) -> f32 {
    (value.x * value.x + value.z * value.z).sqrt()
}

/// `cLib_targetAngleY` (`c_lib.cpp`).
pub fn target_angle_y(from: Vec3, to: Vec3) -> i16 {
    walk::to_s16((to.x - from.x).atan2(to.z - from.z))
}

/// `cM_atan2s`.
fn atan2s(y: f32, x: f32) -> i16 {
    walk::to_s16(y.atan2(x))
}

/// `cLib_addCalc2` (`d_lib.cpp`): `value += clamp((target - value) * scale, +-maxStep)`.
fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    let step = (target - *value) * scale;
    let step = step.clamp(-max_step, max_step);
    *value += step;
}
