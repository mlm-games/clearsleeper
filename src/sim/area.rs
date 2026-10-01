use bevy_ecs::prelude::*;
use glam::{Vec2, Vec3};

use super::collision::{Collision, MoveFrom};
use super::player::Player;

#[derive(Resource, Debug, Clone, Copy)]
pub struct Area {
    pub bounds_min: Vec2,
    pub bounds_max: Vec2,
    pub spawn: Vec3,
}

impl Default for Area {
    fn default() -> Self {
        Self {
            bounds_min: Vec2::new(-1000.0, -1000.0),
            bounds_max: Vec2::new(1000.0, 1000.0),
            spawn: Vec3::ZERO,
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
            Some(collision) => player.pos = collision.slide(from.0, player.pos),
            None => {
                player.pos.x = player.pos.x.clamp(area.bounds_min.x, area.bounds_max.x);
                player.pos.z = player.pos.z.clamp(area.bounds_min.y, area.bounds_max.y);
            }
        }
    }
}
