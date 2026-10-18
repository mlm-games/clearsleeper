use bevy_ecs::prelude::*;

pub const BSCURTAIN_ACTORS: [&str; 2] = ["Ptco", "Ptcu"];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BsCurtain;

impl BsCurtain {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_bscurtain(name: &str) -> bool {
    BSCURTAIN_ACTORS.contains(&name)
}

pub fn control(_curtains: Query<&BsCurtain>) {}
