use bevy_ecs::prelude::*;
use glam::Vec3;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Actor {
    pub name: String,
    pub parameters: u32,
    pub pos: Vec3,
    pub angle: [i16; 3],
    pub set_id: u16,
    pub room: i8,
}
