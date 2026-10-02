use bevy_ecs::prelude::*;
use glam::Vec2;

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct MoveIntent {
    pub axis: Vec2,
}

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct CameraYaw(pub f32);

/// Quarter-heart damage staged per frame (0 clears), like [`MoveIntent`].
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct DamageIntent(pub u8);

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct DoIntent(pub bool);
