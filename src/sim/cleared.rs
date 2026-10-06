use bevy_ecs::prelude::*;

use super::actor::Actor;
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

pub fn control(
    mut switches: ResMut<Switches>,
    mut markers: Query<&mut Cleared>,
    foes: Query<(&Actor, Option<&super::combat::HitPoints>)>,
) {
    for mut marker in &mut markers {
        if marker.done {
            continue;
        }
        let room = marker.room;
        let living = room >= 0
            && foes
                .iter()
                .any(|(other, hp)| hp.is_some() && i32::from(other.room) == room);
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
