use bevy_ecs::prelude::*;

pub const NPC_UK_ACTORS: [&str; 6] = // <- d_stage.cpp:1010-1016
    ["UkB", "UkC", "UkD", "UkB2", "UkC2", "UkD2"];

pub const CULL_MIN: [f32; 3] = [-35.0, -10.0, -35.0]; // <- d_a_npc_uk.cpp:2080
pub const CULL_MAX: [f32; 3] = [35.0, 100.0, 35.0]; // <- d_a_npc_uk.cpp:2080

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcUk;

impl NpcUk {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_npc_uk(name: &str) -> bool {
    NPC_UK_ACTORS.contains(&name)
}

pub fn control(_npcs: Query<&NpcUk>) {}
