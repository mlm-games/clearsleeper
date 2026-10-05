use bevy_ecs::prelude::*;

pub const KYTAG01_ACTORS: [&str; 1] = ["ky_tag1"];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kytag01;

impl Kytag01 {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_kytag01(name: &str) -> bool {
    KYTAG01_ACTORS.contains(&name)
}

pub fn control(_tags: Query<&Kytag01>) {}