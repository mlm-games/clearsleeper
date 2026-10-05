use bevy_ecs::prelude::*;

pub const KYTAG02_ACTORS: [&str; 1] = ["ky_tag2"];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kytag02;

impl Kytag02 {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_kytag02(name: &str) -> bool {
    KYTAG02_ACTORS.contains(&name)
}

pub fn control(_tags: Query<&Kytag02>) {}
