use bevy_ecs::prelude::*;

pub const DEMOKMM_ACTORS: [&str; 2] = ["DmKmm", "DmKmm2"];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DemoKmm;

impl DemoKmm {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_demokmm(name: &str) -> bool {
    DEMOKMM_ACTORS.contains(&name)
}

pub fn control(_gulls: Query<&DemoKmm>) {}
