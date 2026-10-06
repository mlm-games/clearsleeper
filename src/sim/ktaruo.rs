use bevy_ecs::prelude::*;

pub const KTARUO_ACTORS: [&str; 1] = ["Ktaruo"];

pub const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)
pub const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)

/// The stage string is `Ktaruo` (`d_stage.cpp:837`); the model's `_arcname` is `DBarrel`
/// (`d_a_obj_demo_barrel.cpp:13`). `fopAcM_ct_Demo` (`f_op_actor_mng.h:25`) expands to
/// nothing outside the demo build, so `demoActorID` (`f_op_actor.h:281`) stays 0 in retail
/// and the whole `if (demoActorID != 0)` block of `_execute` (`:96-113`) is dead: `m2DC`
/// (`:74`) and `m2DE` (`:75`) never change, the collect writes (`:102-103`) never run, and
/// `_draw`'s `m2DC == 0` bail (`:129-131`) never fires. What is left is `setParticleHahen`
/// (`:16`), `setParticleSibuki` (`:27`), `dDemo_setDemoData` (`:115`) and the model matrix
/// (`:116-119`) - all renderer/particle work.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ktaruo;

impl Ktaruo {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_ktaruo(name: &str) -> bool {
    KTARUO_ACTORS.contains(&name)
}

pub fn control(_barrels: Query<&Ktaruo>) {}