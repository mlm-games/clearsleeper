use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;

pub const DEMODK_ACTORS: [&str; 1] = ["Demo_Dk"];

pub const CULL_MIN: [f32; 3] = [-1000.0, -1000.0, -1000.0]; // <- d_a_demo_dk.cpp:137
pub const CULL_MAX: [f32; 3] = [1000.0, 1000.0, 1000.0]; // <- d_a_demo_dk.cpp:138

/// Helmaroc King. The stage string is `Demo_Dk` (`d_stage.cpp:671`); the model's `_arcname` is
/// `DEMO_DK` (`d_a_demo_dk.cpp:112`, `:129`). `unk_2C4` is a copy of the create-time position
/// (`:131`) and `daDEMO_DK_Execute` pins the actor's height to it every tick (`:79`) - the only
/// non-render work the term does. The `scale = 1` writes (`:77-78`) are retail's stage-placed
/// scale and `_execute` returns TRUE.
///
/// The `unk_2B6` breathe machine (`mode_wait` `:33-48`, `mode_akubi` `:51-57`) is gated on
/// `mpMorf->isStop()`, animator state derived from the DEMO_DK BCK frame counts, and on
/// `cM_rnd()` (`:41`), a hardware RNG this sim has no channel for, so both modes are omitted
/// whole - including the `unk_2C0` countdown (`:37-39`, `:54`) that sits inside the same gate.
/// `unk_2BC[2]` (`:61-65`) is decremented but never written non-zero, so that loop is a no-op.
/// Process memory is zeroed by `sBs_ClearArea` (`f_pc_base.cpp:98`, `s_basic.cpp:19-21`), so
/// `unk_2B6` starts at 0 and `mode_wait` would be the entry state.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct DemoDk {
    pub base_y: f32,
}

impl DemoDk {
    pub fn spawn(pos: Vec3) -> Self {
        Self { base_y: pos.y }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_demodk(name: &str) -> bool {
    DEMODK_ACTORS.contains(&name)
}

pub fn control(mut kings: Query<(&mut Actor, &DemoDk)>) {
    for (mut actor, king) in &mut kings {
        actor.pos.y = king.base_y;
    }
}