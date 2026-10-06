use bevy_ecs::prelude::*;

pub const NPC_CO1_ACTORS: [&str; 1] = ["Co1"]; // <- d_stage.cpp:1017

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcCo1;

impl NpcCo1 {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_npc_co1(name: &str) -> bool {
    NPC_CO1_ACTORS.contains(&name)
}

pub fn control(_npcs: Query<&NpcCo1>) {}
