use std::f32::consts::FRAC_PI_2;
use std::f32::consts::PI;

use bevy_ecs::prelude::*;

use super::anim_clock::{FrameClocks, FrameCtrl, Mode, Under};
use super::collision::Collision;
use super::combat::Combat;
use super::ground::grounded;
use super::health::Health;
use super::intent::{CameraYaw, DoIntent, MoveIntent};
use super::link_proc::{BACK_AIR, BACK_GROUND, Proc, allows};
use super::lock::Lock;
use super::player::Player;
use super::roll::Roll;
use super::swim::Swim;
use super::walk::{RAD_PER_UNIT, target_yaw, to_s16};

const STICK_GATE: f32 = 0.05;
pub(crate) const BACK_SPEED: f32 = 22.5;
pub(crate) const BACK_RISE: f32 = 19.0;
pub(crate) const BACK_GRAVITY: f32 = -3.0;
const SIDE_PITCH: f32 = 6200.0 * RAD_PER_UNIT;
const SIDE_SPEED: f32 = 30.0;
pub(crate) const SIDE_GRAVITY: f32 = -2.4;
pub(crate) const SIDE_OFFSET: f32 = FRAC_PI_2;
/// `m_HIO->mBackJump.m.field_0x1C`, `main.cpp:7021`: the drop past which a back jump has
/// become a fall, and `m_HIO->mSideStep.m.field_0x30`, `main.cpp:6344`: the same for a side step.
const BACK_FALL_DROP: f32 = 30.0;
const SIDE_FALL_DROP: f32 = 50.0;
/// `m_HIO->mBackJump.m.field_0x24`, `main.cpp:7035`: the rate the back jump's landing animation
/// runs at, `m_HIO->mBackJump.m.field_0x28` the frame it starts from and
/// `m_HIO->mBackJump.m.field_0x2` the frame it ends at.
const BACK_LAND_RATE: f32 = 0.8;
const BACK_LAND_START: i16 = 0;
const BACK_LAND_END: i16 = 5;
/// `m_HIO->mSideStep.m.field_0x1C`, `main.cpp:6362`: the rate the side step's landing animation
/// runs at, `m_HIO->mSideStep.m.field_0x20` the frame it starts from and
/// `m_HIO->mSideStep.m.field_0x6` the frame it ends at.
const SIDE_LAND_RATE: f32 = 0.85;
const SIDE_LAND_START: i16 = 1;
const SIDE_LAND_END: i16 = 5;
/// `m_HIO->mBackJump.m.field_0x30`, `main.cpp:7069`: the frame the back jump's landing state
/// has to pass before the player goes back to the dispatcher.
pub(crate) const BACK_LAND_FRAME: f32 = 2.0;
/// `m_HIO->mSideStep.m.field_0x28`, `main.cpp:6379`: the same for the side step.
pub(crate) const SIDE_LAND_FRAME: f32 = 1.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JumpKind {
    #[default]
    Back,
    Left,
    Right,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Jump {
    pub active: bool,
    pub kind: JumpKind,
    held: bool,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub(super) enum Direction {
    Forward,
    Left,
    Right,
    Back,
}

pub(super) fn direction(desired: i16, face: i16) -> Direction {
    let diff = desired.wrapping_sub(face);
    if diff.unsigned_abs() > 0x6000 {
        Direction::Back
    } else if diff >= 0x2000 {
        Direction::Left
    } else if diff <= -0x2000 {
        Direction::Right
    } else {
        Direction::Forward
    }
}

/// The drop past which the jump has become a fall, `procBackJump`, `main.cpp:7021-7022`, and
/// `procSideStep`, `main.cpp:6344-6345`: the feet have fallen further below the height the jump
/// left the ground at than the jump's own table allows, so the jump gives up and the fall takes
/// over, and the fall is what lands.
fn fall_drop(kind: JumpKind) -> f32 {
    match kind {
        JumpKind::Back => BACK_FALL_DROP,
        JumpKind::Left | JumpKind::Right => SIDE_FALL_DROP,
    }
}

/// The landing state the jump leaves into, `procBackJumpLand_init`, `main.cpp:7015-7016`, and
/// `procSideStepLand_init`, `main.cpp:6335-6336`.
fn land_proc(kind: JumpKind) -> Proc {
    match kind {
        JumpKind::Back => Proc::BackJumpLand,
        JumpKind::Left | JumpKind::Right => Proc::SideStepLand,
    }
}

/// The landing animation the state plays, `procBackJumpLand_init`, `main.cpp:7033-7038`, and
/// `procSideStepLand_init`, `main.cpp:6362`.
fn land_anime(kind: JumpKind, clock: &mut FrameCtrl) {
    match kind {
        JumpKind::Back => clock.set_frame_ctrl(
            Mode::None,
            BACK_LAND_START,
            BACK_LAND_END,
            BACK_LAND_RATE,
            f32::from(BACK_LAND_START),
        ),
        JumpKind::Left | JumpKind::Right => clock.set_frame_ctrl(
            Mode::None,
            SIDE_LAND_START,
            SIDE_LAND_END,
            SIDE_LAND_RATE,
            f32::from(SIDE_LAND_START),
        ),
    }
}

pub fn control(
    collision: Option<Res<Collision>>,
    do_intent: Res<DoIntent>,
    intent: Res<MoveIntent>,
    camera: Res<CameraYaw>,
    health: Res<Health>,
    swim: Res<Swim>,
    lock: Res<Lock>,
    combat: Res<Combat>,
    mut clocks: ResMut<FrameClocks>,
    mut proc: Option<ResMut<super::link_proc::LinkProc>>,
    mut roll: ResMut<Roll>,
    mut jump: ResMut<Jump>,
    mut players: Query<&mut Player>,
) {
    let pressed = do_intent.0 && !jump.held;
    jump.held = do_intent.0;
    for mut player in &mut players {
        let player = &mut *player;
        if swim.active || health.units == 0 || combat.active {
            jump.active = false;
            continue;
        }
        if jump.active {
            if !allows(proc.as_deref(), BACK_AIR) {
                continue;
            }
            if grounded(collision.as_deref(), player.pos) && player.speed_y <= 0.0 {
                jump.active = false;
                player.speed = 0.0;
                // `procBackJumpLand_init`, `main.cpp:7043`, takes the body's facing from the
                // model's, while `procSideStepLand_init`, `main.cpp:6366`, turns the body the
                // other way instead.
                player.yaw = match jump.kind {
                    JumpKind::Back => player.face,
                    JumpKind::Left => player.face + SIDE_OFFSET + PI,
                    JumpKind::Right => player.face - SIDE_OFFSET + PI,
                };
                let drop = proc
                    .as_deref()
                    .map_or(0.0, |state| state.takeoff_y - player.pos.y);
                if drop <= fall_drop(jump.kind) {
                    land_anime(jump.kind, clocks.under_mut(Under::Move0));
                    if let Some(state) = proc.as_deref_mut() {
                        state.enter(land_proc(jump.kind));
                    }
                }
            }
            continue;
        }
        if !allows(proc.as_deref(), BACK_GROUND) {
            continue;
        }
        if !pressed
            || lock.target.is_none()
            || !grounded(collision.as_deref(), player.pos)
            || intent.axis.length().min(1.0) <= STICK_GATE
        {
            continue;
        }
        let desired = target_yaw(&intent, &camera);
        match direction(desired, to_s16(player.face)) {
            Direction::Forward => continue,
            Direction::Back => {
                player.yaw = player.face + PI;
                player.speed = BACK_SPEED;
                player.speed_y = BACK_RISE;
                jump.kind = JumpKind::Back;
            }
            Direction::Left => {
                player.yaw = player.face + SIDE_OFFSET;
                player.speed = SIDE_PITCH.cos() * SIDE_SPEED;
                player.speed_y = SIDE_PITCH.sin() * SIDE_SPEED;
                jump.kind = JumpKind::Left;
            }
            Direction::Right => {
                player.yaw = player.face - SIDE_OFFSET;
                player.speed = SIDE_PITCH.cos() * SIDE_SPEED;
                player.speed_y = SIDE_PITCH.sin() * SIDE_SPEED;
                jump.kind = JumpKind::Right;
            }
        }
        jump.active = true;
        roll.active = false;
        // `checkNextActionFromButton`, `main.cpp:4300-4312`: the button's own dispatcher takes the
        // jump into the process that carries it, the back jump at `main.cpp:4305` and the side step
        // at `main.cpp:4302`.
        if let Some(state) = proc.as_deref_mut() {
            state.enter(super::link_move::jump_proc(jump.kind));
        }
    }
}
