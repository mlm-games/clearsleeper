use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::enemy::Enemy;
use super::flags::Switches;

pub const CLEARED_ACTOR: &str = "ALLdie";

pub const SWITCH_SHIFT: u32 = 8;
const COUNTDOWN: u32 = 65;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cleared {
    pub switch: u8,
    pub countdown: u32,
    pub done: bool,
    pub room: i32,
}

pub fn spawn(parameters: u32, switches: &Switches, room: i32) -> Cleared {
    let switch = ((parameters >> SWITCH_SHIFT) & 0xFF) as u8;
    Cleared {
        switch,
        countdown: COUNTDOWN,
        done: switches.test(switch),
        room,
    }
}

/// `fopAcM_myRoomSearchEnemy` (`f_op_actor_mng.cpp:1283`) judges each proc in the
/// room's layer with `enemySearchJugge`, which tests `i_ac->group == fopAc_ENEMY_e`
/// (`:1275`) - the actor PROFILE GROUP, not the presence of hit points. Several
/// enemies (`gohdan`, `phantom_ganon`, `bwds`, `bwd`) carry `Enemy` without
/// `combat::HitPoints`, so gating on hit points made those rooms read as cleared.
/// `roomNo` is `fopAcM_GetRoomNo(this)` (`:1334`), re-read every tick, not frozen
/// at spawn.
pub fn control(
    mut switches: ResMut<Switches>,
    mut markers: Query<(&Actor, &mut Cleared)>,
    foes: Query<&Actor, With<Enemy>>,
) {
    for (actor, mut marker) in &mut markers {
        if marker.done {
            continue;
        }
        let room = marker.room.max(i32::from(actor.room));
        let living = room >= 0 && foes.iter().any(|other| i32::from(other.room) == room);
        if living {
            marker.countdown = COUNTDOWN;
            continue;
        }
        marker.countdown = marker.countdown.saturating_sub(1);
        if marker.countdown == 0 {
            switches.set(marker.switch);
            marker.done = true;
        }
    }
}
