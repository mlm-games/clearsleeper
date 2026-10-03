use bevy_ecs::prelude::*;

use super::collision::{Collision, NO_EXIT};
use super::ground::grounded;
use super::health::Health;
use super::player::Player;

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct NextStage {
    pub exit: Option<u8>,
    pub target: Option<Target>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub stage: String,
    pub room: u8,
    pub start: u8,
}

pub fn check(
    collision: Option<Res<Collision>>,
    health: Res<Health>,
    mut next: ResMut<NextStage>,
    players: Query<&Player>,
) {
    for player in &players {
        if health.units == 0 {
            next.exit = None;
            continue;
        }
        let Some(collision) = collision.as_deref() else {
            next.exit = None;
            continue;
        };
        if !grounded(Some(collision), player.pos) {
            next.exit = None;
            continue;
        }
        next.exit = match collision.exit_at(player.pos.x, player.pos.z) {
            Some(exit) if exit != NO_EXIT => Some(exit),
            _ => None,
        };
    }
}
