use bevy_ecs::prelude::*;
use glam::Vec2;

use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::intent::MoveIntent;
use super::player::Player;

pub const COMPLETE_SWITCH: u8 = 0;
const CREDITS_TICKS: u32 = 600;

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Ending {
    pub running: bool,
    pub finished: bool,
    pub ticks: u32,
}

pub fn control(
    switches: Res<Switches>,
    mut ending: ResMut<Ending>,
    mut events: ResMut<SimEvents>,
    mut intent: ResMut<MoveIntent>,
    mut players: Query<&mut Player>,
) {
    if !ending.running {
        if !switches.0.test(COMPLETE_SWITCH) {
            return;
        }
        ending.running = true;
        ending.finished = false;
        ending.ticks = 0;
        events.list.push(SimEvent::Credits);
    }
    ending.ticks += 1;
    if ending.ticks >= CREDITS_TICKS {
        ending.running = false;
        ending.finished = true;
        events.list.push(SimEvent::CreditsDone);
        return;
    }
    intent.axis = Vec2::ZERO;
    for mut player in &mut players {
        player.speed = 0.0;
    }
}
