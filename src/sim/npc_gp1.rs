use bevy_ecs::prelude::*;

pub const NPC_GP1_ACTORS: [&str; 1] = ["Gp1"]; // <- d_stage.cpp:750

pub const CULL_MIN: [f32; 3] = [-70.0, -20.0, -70.0]; // <- d_a_npc_gp1.cpp:1268
pub const CULL_MAX: [f32; 3] = [50.0, 240.0, 50.0]; // <- d_a_npc_gp1.cpp:1268

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcGp1;

impl NpcGp1 {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_npc_gp1(name: &str) -> bool {
    NPC_GP1_ACTORS.contains(&name)
}

pub fn control(_npcs: Query<&NpcGp1>) {}
