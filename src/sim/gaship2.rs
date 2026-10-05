use bevy_ecs::prelude::*;

pub const GASHIP2_ACTORS: [&str; 1] = ["Gaship2"];

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Gaship2;

impl Gaship2 {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_gaship2(name: &str) -> bool {
    GASHIP2_ACTORS.contains(&name)
}

pub fn control(_hulls: Query<&Gaship2>) {}