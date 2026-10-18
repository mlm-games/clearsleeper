use bevy_ecs::prelude::*;

pub const ATTTAG_ACTORS: [&str; 2] = ["AttTag", "AttTagB"];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Atttag;

impl Atttag {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_atttag(name: &str) -> bool {
    ATTTAG_ACTORS.contains(&name)
}

pub fn control(_tags: Query<&Atttag>) {}
