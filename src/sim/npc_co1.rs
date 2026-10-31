use bevy_ecs::prelude::*;

pub const NPC_CO1_ACTORS: [&str; 1] = ["Co1"]; // <- d_stage.cpp:1017

pub const CULL_MIN: [f32; 3] = [-40.0, -20.0, -100.0]; // <- d_a_npc_co1.cpp:949
pub const CULL_MAX: [f32; 3] = [40.0, 100.0, 60.0]; // <- d_a_npc_co1.cpp:949

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcCo1;

impl NpcCo1 {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_npc_co1(name: &str) -> bool {
    NPC_CO1_ACTORS.contains(&name)
}

pub fn control(_npcs: Query<&NpcCo1>) {}
