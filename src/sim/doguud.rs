use bevy_ecs::prelude::*;

pub const DOGUUD_ACTORS: [&str; 1] = ["DoguuD"];

pub const CULL_MIN: [f32; 3] = [-30000.0, -5000.0, -30000.0]; // <- d_a_obj_doguu_demo.cpp:50
pub const CULL_MAX: [f32; 3] = [30000.0, 40000.0, 30000.0]; // <- d_a_obj_doguu_demo.cpp:50

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct DoguuD;

impl DoguuD {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_doguud(name: &str) -> bool {
    DOGUUD_ACTORS.contains(&name)
}

pub fn control(_walls: Query<&DoguuD>) {}