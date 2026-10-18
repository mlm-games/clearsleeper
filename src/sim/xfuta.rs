use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;

pub const XFUTA_ACTORS: [&str; 1] = ["Xfuta"];

pub const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)
pub const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)

/// `daObjXfuta::Act_c::_execute` (`d_a_obj_xfuta.cpp:91-95`) rewrites `current.pos` from
/// `home.pos` every tick, so the actor is pinned to its placed position. `home` equals the
/// stage-placed position (`f_op_actor.cpp:284` sets `current = home`). The `_arcname` is
/// `X_futa` (`d_a_obj_xfuta.cpp:14`); the stage object name `Xfuta`
/// (`tww/src/d/d_stage.cpp:1122`) is what dispatches.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Xfuta {
    pub home: Vec3,
}

impl Xfuta {
    pub fn spawn(home: Vec3) -> Self {
        Self { home }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_xfuta(name: &str) -> bool {
    XFUTA_ACTORS.contains(&name)
}

pub fn control(mut shafts: Query<(&mut Actor, &Xfuta)>) {
    for (mut actor, shaft) in &mut shafts {
        actor.pos = shaft.home;
    }
}
