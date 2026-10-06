use bevy_ecs::prelude::*;

pub const NPC_DS1_ACTORS: [&str; 1] = ["Ds1"]; // <- d_stage.cpp:744

pub const CULL_MIN: [f32; 3] = [-70.0, -100.0, -80.0]; // <- f_op_actor_mng.cpp:570 (fopAc_CULLBOX_12_e)
pub const CULL_MAX: [f32; 3] = [70.0, 240.0, 100.0]; // <- f_op_actor_mng.cpp:570 (fopAc_CULLBOX_12_e)

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcDs1;

impl NpcDs1 {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_npc_ds1(name: &str) -> bool {
    NPC_DS1_ACTORS.contains(&name)
}

pub fn control(_npcs: Query<&NpcDs1>) {}
