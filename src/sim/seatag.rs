use bevy_ecs::prelude::*;

pub const SEATAG_ACTORS: [&str; 1] = ["Seatag"];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seatag;

impl Seatag {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_seatag(name: &str) -> bool {
    SEATAG_ACTORS.contains(&name)
}

pub fn control(_tags: Query<&Seatag>) {}
