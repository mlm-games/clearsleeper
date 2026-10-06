use bevy_ecs::prelude::*;

pub const NPC_AC1_ACTORS: [&str; 1] = ["Ac1"]; // <- d_stage.cpp:776

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcAc1;

impl NpcAc1 {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_npc_ac1(name: &str) -> bool {
    NPC_AC1_ACTORS.contains(&name)
}

pub fn control(_npcs: Query<&NpcAc1>) {}
