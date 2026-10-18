use bevy_ecs::prelude::*;

pub const VMSDZ_ACTORS: [&str; 1] = ["VmsDZ"];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vmsdz;

impl Vmsdz {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_vmsdz(name: &str) -> bool {
    VMSDZ_ACTORS.contains(&name)
}

pub fn control(_boats: Query<&Vmsdz>) {}
