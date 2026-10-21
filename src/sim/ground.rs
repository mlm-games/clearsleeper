use bevy_ecs::prelude::*;
use glam::Vec3;

use super::collision::Collision;
use super::player::Player;

/// How far above the actor's feet the ground check looks. `dBgS_Acch::GroundCheck`
/// (`d_bg_s_acch.cpp:157`) probes at `m_ground_check_offset - m_ground_up_h` above the position it
/// was given; an accessor that never changes either carries `60` (`d_bg_s_acch.cpp:93`).
pub const GROUND_PROBE: f32 = 20.0;

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
