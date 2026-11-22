use bevy_ecs::prelude::*;

use super::combat::{Combat, CutKind};
use super::intent::{ItemIntent, MoveIntent};
use super::link_proc::{LinkProc, Proc};
use super::player::Player;

/// The charged spin chain `CUT_TURN_CHARGE → CUT_TURN_MOVE → CUT_ROLL`, flattened into one
/// state. Both the charged run and the roll write `daPyStts0_SPIN_ATTACK_e`
/// (`tww/src/d/actor/d_a_player_sword.inc:1720,1554`;
/// `wwhd/wwhd_src/d/actor/d_a_player_main_08.cpp:867`). The reference's magic gate drives the
/// 47-tick charge; here the charge always runs, and the roll is handed to `Combat` as a
/// `CutKind::Spin` (cut type `CUT_TYPE_CUT_ROLL`).
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SpinPhase {
    #[default]
    Idle,
    Charge,
    Move,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Spin {
    pub phase: SpinPhase,
    pub charge: u32,
    pub frame: u32,
    held: bool,
}

/// `mProcVar0.m34D0 = 47` (`tww/src/d/actor/d_a_player_sword.inc:1715`,
/// `wwhd/wwhd_src/d/actor/d_a_player_main_08.cpp:863`).
pub const CHARGE_TICKS: u32 = 47;
/// The roll's AT lives for its animation (`d_a_player_sword.inc:1512-1555`).
pub const ROLL_TICKS: u32 = 24;

pub fn control(
    item: Res<ItemIntent>,
    intent: Res<MoveIntent>,
    mut combat: ResMut<Combat>,
    mut proc: Option<ResMut<LinkProc>>,
    mut spin: ResMut<Spin>,
    players: Query<&Player>,
) {
    let held = item.0;
    let pressed = held && !spin.held;
    spin.held = held;
    let grounded = players.iter().next().is_some_and(|p| p.speed_y <= 0.0);
    match spin.phase {
        SpinPhase::Idle => {
            if pressed && !combat.active && grounded && intent.axis.length() <= 0.5 {
                spin.phase = SpinPhase::Charge;
                spin.charge = 0;
                spin.frame = 0;
                if let Some(proc) = proc.as_deref_mut() {
                    proc.enter(Proc::CutTurnCharge);
                }
            }
        }
        SpinPhase::Charge => {
            spin.frame += 1;
            if !held {
                spin.phase = SpinPhase::Idle;
                if let Some(proc) = proc.as_deref_mut() {
                    proc.enter(Proc::Wait);
                }
            } else if spin.frame >= CHARGE_TICKS {
                spin.phase = SpinPhase::Move;
                spin.charge = CHARGE_TICKS;
                spin.frame = 0;
                if let Some(proc) = proc.as_deref_mut() {
                    proc.enter(Proc::CutTurnMove);
                }
            }
        }
        SpinPhase::Move => {
            spin.frame += 1;
            spin.charge = spin.charge.saturating_sub(1);
            // The charged run ends on B release (`d_a_player_sword.inc:1805`, HD `:964`): a
            // full charge rolls, handing the spin to Combat as the cut the AT pass resolves.
            if !held {
                if spin.charge == 0 {
                    combat.active = true;
                    combat.kind = CutKind::Spin;
                    combat.frame = 0;
                    combat.combo = 0;
                    combat.queued = false;
                    if let Some(proc) = proc.as_deref_mut() {
                        proc.enter(Proc::CutRoll);
                    }
                }
                spin.phase = SpinPhase::Idle;
            }
        }
    }
}
