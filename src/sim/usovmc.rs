use bevy_ecs::prelude::*;

pub const USO_VMC_ACTORS: [&str; 1] = ["Usovmc"];

pub const CULL_MIN: [f32; 3] = [-80.0, -1.0, -80.0]; // <- d_a_obj_usovmc.cpp:26
pub const CULL_MAX: [f32; 3] = [80.0, 205.0, 80.0]; // <- d_a_obj_usovmc.cpp:26

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Usovmc;

impl Usovmc {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_usovmc(name: &str) -> bool {
    USO_VMC_ACTORS.contains(&name)
}

pub fn control(_boats: Query<&Usovmc>) {}
