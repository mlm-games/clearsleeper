use bevy_ecs::prelude::*;

pub const KOMORE_ACTORS: [&str; 1] = ["Yfrlt00"];

pub const CULL_MIN: [f32; 3] = [-100.0, -1000.0, -100.0]; // <- d_a_komore.cpp:53
pub const CULL_MAX: [f32; 3] = [100.0, 100.0, 100.0]; // <- d_a_komore.cpp:53

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Komore;

impl Komore {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_komore(name: &str) -> bool {
    KOMORE_ACTORS.contains(&name)
}

pub fn control(_glowworms: Query<&Komore>) {}