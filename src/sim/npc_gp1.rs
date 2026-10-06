use bevy_ecs::prelude::*;

pub const NPC_GP1_ACTORS: [&str; 1] = ["Gp1"]; // <- d_stage.cpp:750

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcGp1;

impl NpcGp1 {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_npc_gp1(name: &str) -> bool {
    NPC_GP1_ACTORS.contains(&name)
}

pub fn control(_npcs: Query<&NpcGp1>) {}
