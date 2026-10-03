use bevy_ecs::prelude::*;

use super::collision::{Collision, NO_EXIT};
use super::ground::grounded;
use super::health::Health;
use super::player::Player;

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NextStage(pub Option<u8>);

pub fn check(
    collision: Option<Res<Collision>>,
    health: Res<Health>,
    mut next: ResMut<NextStage>,
    players: Query<&Player>,
) {
    for player in &players {
        if health.units == 0 {
            next.0 = None;
            continue;
        }
        let Some(collision) = collision.as_deref() else {
            next.0 = None;
            continue;
        };
        if !grounded(Some(collision), player.pos) {
            next.0 = None;
            continue;
        }
        next.0 = match collision.exit_at(player.pos.x, player.pos.z) {
            Some(exit) if exit != NO_EXIT => Some(exit),
            _ => None,
        };
    }
}
