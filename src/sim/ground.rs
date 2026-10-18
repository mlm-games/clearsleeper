use bevy_ecs::prelude::*;
use glam::Vec3;

use super::collision::Collision;
use super::player::Player;

pub fn ground_check(collision: Option<Res<Collision>>, mut players: Query<&mut Player>) {
    for mut player in &mut players {
        let surface = match &collision {
            Some(collision) => collision.ground_at(player.pos.x, player.pos.z),
            None => Some(0.0),
        };
        if let Some(surface) = surface {
            if player.pos.y <= surface {
                player.pos.y = surface;
                player.speed_y = 0.0;
            }
        }
    }
}

pub fn grounded(collision: Option<&Collision>, pos: Vec3) -> bool {
    let surface = match collision {
        Some(collision) => collision.ground_at(pos.x, pos.z),
        None => Some(0.0),
    };
    matches!(surface, Some(ground) if pos.y <= ground)
}

pub fn surface(collision: Option<&Collision>, pos: Vec3) -> Option<f32> {
    match collision {
        Some(collision) => collision.ground_at(pos.x, pos.z),
        None => Some(0.0),
    }
}

pub fn slope_normal(collision: Option<&Collision>, pos: Vec3) -> Vec3 {
    match collision {
        Some(collision) => collision.ground_normal(pos.x, pos.z).unwrap_or(Vec3::Y),
        None => Vec3::Y,
    }
}
