use bevy_ecs::prelude::*;
use glam::Vec3;

pub const TAG_ACTOR: &str = "TagSo";
pub const MAP_TAG_ACTOR: &str = "TagMSo";

const TYPE_SHIFT: u32 = 16;
const TYPE_MASK: u32 = 0xFF;
const MINIGAME_TYPE: u32 = 1;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Tag {
    pub minigame: bool,
    pub pos: Vec3,
    pub yaw: i16,
}

impl Tag {
    pub fn spawn(parameters: u32, pos: Vec3, yaw: i16) -> Self {
        let minigame = ((parameters >> TYPE_SHIFT) & TYPE_MASK) == MINIGAME_TYPE;
        Self { minigame, pos, yaw }
    }
}

pub fn is_tag(name: &str) -> bool {
    name == TAG_ACTOR || name == MAP_TAG_ACTOR
}