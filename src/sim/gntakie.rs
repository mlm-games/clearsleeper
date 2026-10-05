use bevy_ecs::prelude::*;

pub const GNTAKIE_ACTORS: [&str; 1] = ["Gntakie"];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gntakie;

impl Gntakie {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_gntakie(name: &str) -> bool {
    GNTAKIE_ACTORS.contains(&name)
}

pub fn control(_falls: Query<&Gntakie>) {}