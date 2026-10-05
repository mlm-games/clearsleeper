use bevy_ecs::prelude::*;

pub const GASHIP_ACTORS: [&str; 1] = ["Gaship1"];

pub const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)
pub const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)

/// The stage string is `Gaship1` (`d_stage.cpp:1155`); the model's `_arcname` is `GaShip`
/// (`d_a_obj_gaship.cpp:13`). `daObjGaship::Act_c::_execute` (`:97-101`) is `set_mtx()`
/// plus `birth_flag()`: the first only rebuilds the model matrix, and the second spawns
/// `fpcNm_MAJUU_FLAG_e` (`d_a_obj_gaship.cpp:32`, stage names `MjFlag`/`MtFlag`/`HcFlag`,
/// `d_stage.cpp:831-833`), a profile with no port in this crate. Both are renderer/output
/// work, so the tick has nothing to do.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gaship;

impl Gaship {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_gaship(name: &str) -> bool {
    GASHIP_ACTORS.contains(&name)
}

pub fn control(_hulls: Query<&Gaship>) {}