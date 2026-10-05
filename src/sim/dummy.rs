use bevy_ecs::prelude::*;

pub const DUMMY_ACTORS: [&str; 4] = ["TagD1", "TagD2", "TagD3", "TagD4"];

pub const CULL_MIN: [f32; 3] = [-100.0, -1000.0, -100.0]; // <- d_a_dummy.cpp:19
pub const CULL_MAX: [f32; 3] = [100.0, 100.0, 100.0]; // <- d_a_dummy.cpp:19

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dummy;

impl Dummy {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_dummy(name: &str) -> bool {
    DUMMY_ACTORS.contains(&name)
}

pub fn control(_tags: Query<&Dummy>) {}
