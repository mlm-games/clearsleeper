use bevy_ecs::prelude::*;
use glam::Vec3;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Player {
    pub pos: Vec3,
    pub yaw: f32,
    pub face: f32,
    pub speed: f32,
    pub speed_y: f32,
}
