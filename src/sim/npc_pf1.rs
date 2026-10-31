use bevy_ecs::prelude::*;

pub const NPC_PF1_ACTORS: [&str; 1] = ["Pf1"]; // <- d_stage.cpp:749

pub const CULL_MIN: [f32; 3] = [-100.0, -20.0, -80.0]; // <- d_a_npc_pf1.cpp:945
pub const CULL_MAX: [f32; 3] = [100.0, 180.0, 140.0]; // <- d_a_npc_pf1.cpp:945

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcPf1;

impl NpcPf1 {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_npc_pf1(name: &str) -> bool {
    NPC_PF1_ACTORS.contains(&name)
}

pub fn control(_npcs: Query<&NpcPf1>) {}
