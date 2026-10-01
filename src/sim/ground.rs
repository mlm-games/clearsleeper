use bevy_ecs::prelude::*;

use super::player::Player;

pub fn ground_check(mut players: Query<&mut Player>) {
    for mut player in &mut players {
        if player.pos.y <= 0.0 {
            player.pos.y = 0.0;
            player.speed_y = 0.0;
        }
    }
}
