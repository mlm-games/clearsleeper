use bevy_ecs::prelude::*;

pub const VTENG_ACTORS: [&str; 1] = ["Vteng"]; // <- d_stage.cpp:1127

/// Puppet Ganon's intro curtains. The stage string and the model's `_arcname` are both
/// `Vteng` (`d_stage.cpp:1127`, `d_a_obj_Vteng.cpp:17`). `_execute` (`:120-126`) advances the
/// `dBgW` the curtain hangs from and plays the morph; `jokai_demo` (`:70-81`) can only ever
/// return FALSE in retail, because `dDemo_object_c::getActor(0)` returns NULL
/// (`d_demo.cpp:502`), `demoActorID` (`f_op_actor.h:281`) is zeroed at creation by
/// `sBs_ClearArea` (`f_pc_base.cpp:98`, `s_basic.cpp:19-21`) and `fopAcM_ct_Demo`
/// (`f_op_actor_mng.h:25`) expands to nothing outside the demo build, so
/// `dDemo_setDemoData` bails at `d_demo.cpp:138-139`. `_draw` (`:129-135`) is the model draw.
/// The profile is `fopAc_CULLBOX_CUSTOM_e` (`:185`) but the term never calls
/// `fopAcM_setCullSizeBox`, `fopAcM_SetMin` or `fopAcM_SetMax`, so no cull box is derivable
/// from the decomp.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vteng;

impl Vteng {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_vteng(name: &str) -> bool {
    VTENG_ACTORS.contains(&name)
}

pub fn control(_curtains: Query<&Vteng>) {}
