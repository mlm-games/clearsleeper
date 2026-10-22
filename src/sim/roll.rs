use bevy_ecs::prelude::*;

use super::collision::Collision;
use super::combat::Combat;
use super::ground::grounded;
use super::health::Health;
use super::intent::{CameraYaw, DoIntent, MoveIntent, PadButtons};
use super::jump::{Direction, direction};
use super::link_move::REST_SPEED;
use super::link_proc::{Proc, ROLL_AIR, ROLL_GROUND, allows};
use super::lock::Lock;
use super::player::Player;
use super::swim::Swim;
use super::talk::Dialogue;
use super::walk::{from_s16, target_yaw, to_s16};

const ROLL_STICK: f32 = 0.75;
const ROLL_IDLE: f32 = 0.05;
const ROLL_SPEED_SCALE: f32 = 1.5;
const ROLL_SPEED_BASE: f32 = 0.5;
const ROLL_SPEED_MIN: f32 = 5.0;
const ROLL_SPEED_MAX: f32 = 26.0;
const ROLL_RATE: f32 = 1.1;
const ROLL_END: f32 = 19.0;
/// `m_HIO->mRoll.m.field_0x10`, `main.cpp:6853`: the frame past which the roll may be cut short.
const ROLL_CUT: f32 = 17.0;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Roll {
    pub active: bool,
    pub frame: f32,
    held: bool,
}

/// `checkNextMode`, `main.cpp:4461-4467`: with a lock-on the roll leaves through the attention
/// states, which are the ones that hold the model on its target, and without one it leaves
/// through the ordinary move. The split is on the speed the roll is carrying, the same
/// `std::abs(mNormalSpeed) <= 0.001f` the reference asks at `main.cpp:4462`.
fn leave(locked: bool, speed: f32) -> Proc {
    if locked {
        if speed.abs() <= REST_SPEED {
            Proc::AtnActorWait
        } else {
            Proc::AtnActorMove
        }
    } else {
        Proc::Move
    }
}

/// `spActionButton`, `d_a_player_main.h:1898`: the action button is the one the reference reads as a level
/// rather than an edge, and it is the button that lets the roll be cut short.
fn action_down(pad: Option<&PadButtons>) -> bool {
    pad.is_some_and(|pad| pad.hold.r)
}

pub fn control(
    do_intent: Res<DoIntent>,
    intent: Res<MoveIntent>,
    camera: Res<CameraYaw>,
    swim: Res<Swim>,
    health: Res<Health>,
    lock: Res<Lock>,
    combat: Res<Combat>,
    dialogue: Res<Dialogue>,
    pad: Option<Res<PadButtons>>,
    collision: Option<Res<Collision>>,
    mut proc: Option<ResMut<super::link_proc::LinkProc>>,
    mut roll: ResMut<Roll>,
    mut players: Query<&mut Player>,
) {
    for mut player in &mut players {
        let player = &mut *player;
        let pressed = do_intent.0 && !roll.held;
        roll.held = do_intent.0;
        if roll.active {
            if !allows(proc.as_deref(), ROLL_AIR) {
                continue;
            }
            roll.frame += ROLL_RATE;
            let held = intent.axis.length();
            // `procFrontRoll`, `main.cpp:6852-6854`: the roll may be cut short once its own frame
            // is past the roll table's mark, but only while the stick is held or the action button
            // is down. `checkNextMode(1)`, `main.cpp:4446-4448`, refuses the change when neither
            // is, so the roll runs on to the end of its own animation.
            let cut = roll.frame > ROLL_CUT && (held > ROLL_IDLE || action_down(pad.as_deref()));
            if cut || roll.frame >= ROLL_END {
                if !cut && held <= ROLL_IDLE {
                    // `procFrontRoll`, `main.cpp:6848-6850`: the roll gives its floor speed back
                    // when the stick is centred as the animation runs out.
                    player.speed -= ROLL_SPEED_MIN;
                }
                roll.active = false;
                if let Some(state) = proc.as_deref_mut() {
                    state.enter(leave(lock.target.is_some(), player.speed));
                }
                continue;
            }
        }
        if roll.active
            || !pressed
            || dialogue.open
            || swim.active
            || health.units == 0
            || combat.active
            || !allows(proc.as_deref(), ROLL_GROUND)
        {
            continue;
        }
        let locked = lock.target.is_some();
        let held = intent.axis.length();
        // `setDoStatusBasic`, `main.cpp:2189-2194`: with a lock-on the stick's own direction
        // picks between the jump and the roll, and a stick held forward is the roll. Without a
        // lock-on the roll is the only action the button gives, and only once the stick is held
        // past the button's own mark, `main.cpp:2216-2221`.
        let wanted = if locked {
            let forward = direction(target_yaw(&intent, &camera), to_s16(player.face));
            held > ROLL_IDLE && forward == Direction::Forward
        } else {
            held > ROLL_STICK
        };
        if !wanted || !grounded(collision.as_deref(), player.pos) {
            continue;
        }
        // `checkNextActionFromButton`, `main.cpp:4307-4309`: the model turns onto the stick's own
        // heading only when the player is not locked on, and `procFrontRoll_init`,
        // `main.cpp:6824`, gives the body the model's facing. A roll under a lock-on therefore
        // goes the way the model is already facing, which is the target's.
        if !locked {
            player.face = from_s16(target_yaw(&intent, &camera));
        }
        player.yaw = player.face;
        player.speed = (player.speed * ROLL_SPEED_SCALE + ROLL_SPEED_BASE)
            .clamp(ROLL_SPEED_MIN, ROLL_SPEED_MAX);
        roll.active = true;
        roll.frame = 0.0;
        if let Some(state) = proc.as_deref_mut() {
            state.enter(Proc::FrontRoll);
        }
    }
}
