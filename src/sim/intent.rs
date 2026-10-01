use bevy_ecs::prelude::*;
use glam::Vec2;

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct MoveIntent {
    pub axis: Vec2,
}

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct CameraYaw(pub f32);
