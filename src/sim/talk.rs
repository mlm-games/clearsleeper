use bevy_ecs::prelude::*;
use glam::Vec2;

use super::actor::Actor;
use super::health::Health;
use super::intent::{DoIntent, MoveIntent};
use super::player::Player;

pub const TALK_ACTOR: &str = "NpcSo";

const TALK_RANGE: f32 = 400.0;
const DEFAULT_MESSAGE: u16 = 1;
const BLANK_MESSAGE: i16 = 0;
const MISSING_MESSAGE: i16 = -1;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Talk {
    pub message: u16,
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Dialogue {
    pub open: bool,
    pub message: u16,
    held: bool,
}

pub fn message(angle_x: i16) -> u16 {
    if angle_x == BLANK_MESSAGE || angle_x == MISSING_MESSAGE {
        DEFAULT_MESSAGE
    } else {
        angle_x as u16
    }
}

fn talking(player: &Player, actor: &Actor) -> bool {
    let dx = player.pos.x - actor.pos.x;
    let dz = player.pos.z - actor.pos.z;
    dx * dx + dz * dz <= TALK_RANGE * TALK_RANGE
}

pub fn control(
    do_intent: Res<DoIntent>,
    health: Res<Health>,
    mut intent: ResMut<MoveIntent>,
    mut dialogue: ResMut<Dialogue>,
    talks: Query<(&Talk, &Actor)>,
    mut players: Query<&mut Player>,
) {
    let pressed = do_intent.0 && !dialogue.held;
    dialogue.held = do_intent.0;
    if dialogue.open {
        if pressed {
            dialogue.open = false;
        }
    } else if pressed && health.units > 0 {
        let Some(player) = players.iter().next() else {
            return;
        };
        for (talk, actor) in &talks {
            if talking(player, actor) {
                dialogue.open = true;
                dialogue.message = talk.message;
                break;
            }
        }
    }
    if !dialogue.open {
        return;
    }
    intent.axis = Vec2::ZERO;
    for mut player in &mut players {
        player.speed = 0.0;
    }
}
