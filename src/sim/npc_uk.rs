use bevy_ecs::prelude::*;

pub const NPC_UK_ACTORS: [&str; 6] = // <- d_stage.cpp:1010-1016
    ["UkB", "UkC", "UkD", "UkB2", "UkC2", "UkD2"];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcUk;

impl NpcUk {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_npc_uk(name: &str) -> bool {
    NPC_UK_ACTORS.contains(&name)
}

pub fn control(_npcs: Query<&NpcUk>) {}
