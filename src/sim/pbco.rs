use bevy_ecs::prelude::*;

pub const PBCO_ACTORS: [&str; 1] = ["Pbco"];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pbco;

impl Pbco {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_pbco(name: &str) -> bool {
    PBCO_ACTORS.contains(&name)
}

pub fn control(_counters: Query<&Pbco>) {}
