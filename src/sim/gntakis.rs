use bevy_ecs::prelude::*;

pub const GNTAKIS_ACTORS: [&str; 1] = ["Gntakis"];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gntakis;

impl Gntakis {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_gntakis(name: &str) -> bool {
    GNTAKIS_ACTORS.contains(&name)
}

pub fn control(_falls: Query<&Gntakis>) {}
