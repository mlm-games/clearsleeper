use bevy_ecs::prelude::*;

pub const NPC_PF1_ACTORS: [&str; 1] = ["Pf1"]; // <- d_stage.cpp:749

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcPf1;

impl NpcPf1 {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_npc_pf1(name: &str) -> bool {
    NPC_PF1_ACTORS.contains(&name)
}

pub fn control(_npcs: Query<&NpcPf1>) {}
