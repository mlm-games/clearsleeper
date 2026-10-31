use bevy_ecs::prelude::*;

pub const NPC_AC1_ACTORS: [&str; 1] = ["Ac1"]; // <- d_stage.cpp:776

pub const CULL_MIN: [f32; 3] = [-50.0, -20.0, -50.0]; // <- d_a_npc_ac1.cpp:638
pub const CULL_MAX: [f32; 3] = [50.0, 140.0, 50.0]; // <- d_a_npc_ac1.cpp:638

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcAc1;

impl NpcAc1 {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_npc_ac1(name: &str) -> bool {
    NPC_AC1_ACTORS.contains(&name)
}

pub fn control(_npcs: Query<&NpcAc1>) {}
