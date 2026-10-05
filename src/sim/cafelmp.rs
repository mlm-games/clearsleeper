use bevy_ecs::prelude::*;

use super::actor::Actor;

pub const CAFELMP_ACTORS: [&str; 1] = ["Cafelmp"];

pub const SPIN: i16 = 0xda; // <- d_a_obj_cafelmp.cpp:70

pub const CULL_MIN: [f32; 3] = [-600.0, -0.0, -600.0]; // <- d_a_obj_cafelmp.cpp:35
pub const CULL_MAX: [f32; 3] = [600.0, 900.0, 600.0]; // <- d_a_obj_cafelmp.cpp:35

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cafelmp;

impl Cafelmp {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_cafelmp(name: &str) -> bool {
    CAFELMP_ACTORS.contains(&name)
}

pub fn control(mut lamps: Query<(&mut Actor, &Cafelmp)>) {
    for (mut actor, _) in &mut lamps {
        actor.angle[1] = actor.angle[1].wrapping_add(SPIN);
    }
}
