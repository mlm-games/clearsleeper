use bevy_ecs::prelude::*;

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
}

pub fn spawn(parameters: u32, switches: &Switches) -> Cleared {
    let switch = ((parameters >> SWITCH_SHIFT) & 0xFF) as u8;
    Cleared {
        switch,
        countdown: COUNTDOWN,
        done: switches.0.test(switch),
    }
}

pub fn control(
    mut switches: ResMut<Switches>,
    mut markers: Query<&mut Cleared>,
    foes: Query<(), With<Enemy>>,
) {
    let living = foes.iter().count();
    for mut marker in &mut markers {
        if marker.done {
            continue;
        }
        if living > 0 {
            marker.countdown = COUNTDOWN;
            continue;
        }
        marker.countdown = marker.countdown.saturating_sub(1);
        if marker.countdown == 0 {
            switches.0.set(marker.switch);
            marker.done = true;
        }
    }
}
