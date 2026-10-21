use bevy_ecs::prelude::*;
use glam::Vec3;

use super::collision::Collision;
use super::player::Player;

const GROUND_PROBE: f32 = 20.0;

fn floor_below(collision: &Collision, pos: Vec3) -> Option<f32> {
    collision
        .ground_cross_below(pos + Vec3::Y * GROUND_PROBE)
        .map(|hit| hit.height)
}

pub fn ground_check(collision: Option<Res<Collision>>, mut players: Query<&mut Player>) {
    for mut player in &mut players {
        let surface = match &collision {
            Some(collision) => floor_below(collision, player.pos),
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
        Some(collision) => floor_below(collision, pos),
        None => Some(0.0),
    };
    matches!(surface, Some(ground) if pos.y <= ground)
}

pub fn surface(collision: Option<&Collision>, pos: Vec3) -> Option<f32> {
    match collision {
        Some(collision) => floor_below(collision, pos),
        None => Some(0.0),
    }
}

pub fn slope_normal(collision: Option<&Collision>, pos: Vec3) -> Vec3 {
    match collision {
        Some(collision) => collision
            .ground_cross_below(pos + Vec3::Y * GROUND_PROBE)
            .map(|hit| hit.normal)
            .unwrap_or(Vec3::Y),
        None => Vec3::Y,
    }
}
