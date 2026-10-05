use bevy_ecs::prelude::*;

pub const DMGROOM_ACTORS: [&str; 1] = ["dmgroom"];

pub const CULL_MIN: [f32; 3] = [-600.0, -0.0, -600.0]; // <- d_a_obj_dmgroom.cpp:40
pub const CULL_MAX: [f32; 3] = [600.0, 900.0, 600.0]; // <- d_a_obj_dmgroom.cpp:40

/// Damaged room shell. `daObjDmgroom_c::_execute` (`d_a_obj_dmgroom.cpp:74-93`) is driven
/// entirely by `dComIfGp_demo_getActor(demoActorID)` and then calls `set_mtx`; the demo
/// actor table is absent, so the tick is a renderer shell. The cull box from `CreateInit`
/// (`:40`) is real data and is exposed.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dmgroom;

impl Dmgroom {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_dmgroom(name: &str) -> bool {
    DMGROOM_ACTORS.contains(&name)
}

pub fn control(_rooms: Query<&Dmgroom>) {}