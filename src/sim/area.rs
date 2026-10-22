use bevy_ecs::prelude::*;
use glam::{Vec2, Vec3};

use super::collision::{Collision, MoveFrom, WALL_HEIGHTS, WALL_RADIUS};
use super::ground::GROUND_PROBE;
use super::player::Player;

#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct StageName(pub String);

#[derive(Resource, Debug, Clone, Copy)]
pub struct Area {
    pub bounds_min: Vec2,
    pub bounds_max: Vec2,
    pub spawn: Vec3,
    pub yaw: f32,
}

impl Default for Area {
    fn default() -> Self {
        Self {
            bounds_min: Vec2::new(-1000.0, -1000.0),
            bounds_max: Vec2::new(1000.0, 1000.0),
            spawn: Vec3::ZERO,
            yaw: 0.0,
        }
    }
}

pub fn clamp_to_area(
    collision: Option<Res<Collision>>,
    from: Res<MoveFrom>,
    area: Res<Area>,
    mut players: Query<&mut Player>,
) {
    for mut player in &mut players {
        match &collision {
            Some(collision) => {
                // `dBgS_Acch::CrrPos`, `d_bg_s_acch.cpp:238-277`: the line check runs first and
                // leaves the player at the wall it ran into, and the wall correction then pushes
                // him out of the wall's own radius.
                player.pos = collision.slide(from.0, player.pos);
                // `GetWallAddY`, `d_bg_s_acch.cpp:467-480`, reads the plane the ground check
                // found, and the reference only has one while the player stands on the ground
                // (`ChkGroundFind`, `d_bg_s_acch.cpp:171`).
                let ground = collision.ground_cross_below(player.pos + Vec3::Y * GROUND_PROBE);
                let standing = ground.is_some_and(|hit| player.pos.y <= hit.height);
                let slope = if standing {
                    ground.map(|hit| hit.normal)
                } else {
                    None
                };
                let correction = collision.wall_correct(
                    player.pos,
                    WALL_RADIUS,
                    &WALL_HEIGHTS,
                    player.speed_y,
                    slope,
                );
                player.pos = correction.pos;
            }
            None => {
                player.pos.x = player.pos.x.clamp(area.bounds_min.x, area.bounds_max.x);
                player.pos.z = player.pos.z.clamp(area.bounds_min.y, area.bounds_max.y);
            }
        }
    }
}
