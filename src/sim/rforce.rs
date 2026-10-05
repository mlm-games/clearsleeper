use bevy_ecs::prelude::*;

pub const RFORCE_ACTORS: [&str; 1] = ["Rforce"];

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Rforce;

impl Rforce {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_rforce(name: &str) -> bool {
    RFORCE_ACTORS.contains(&name)
}

pub fn control(_temples: Query<&Rforce>) {}