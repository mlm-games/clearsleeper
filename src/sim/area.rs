use bevy_ecs::prelude::*;
use glam::{Vec2, Vec3};

/// First walkable yard: XZ bounds plus the spawn point. Placeholder extents
/// until island layout arrives with the asset slice.
#[derive(Resource, Debug, Clone, Copy)]
pub struct Area {
    pub bounds_min: Vec2,
    pub bounds_max: Vec2,
    pub spawn: Vec3,
}

impl Default for Area {
    fn default() -> Self {
        Self {
            bounds_min: Vec2::new(-50.0, -50.0),
            bounds_max: Vec2::new(50.0, 50.0),
            spawn: Vec3::ZERO,
        }
    }
}
