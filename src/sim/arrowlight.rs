//! Light-arrow flight effect - `fpcNm_ARROW_LIGHTEFF_e`, `d_a_arrow_lighteff.cpp`, stage string
//! `Arrow_l`.
//!
//! ANSWERING THE QUESTION THE BRIEF ASKED: yes, `Arrow_l` DOES have an OBJNAME row. It is
//! `d_stage.cpp:1250`, `OBJNAME("Arrow_l", fpcNm_ARROW_LIGHTEFF_e, 255, 0)`, where the third
//! field is the `sub` sub-actor index of `OBJNAME(name, proc, sub, gba)` (`d_stage.cpp:432-435`)
//! and `255` means none. So unlike a hypothetical actor with no row at all, this one is just as
//! placeable by name as any other, and `ARROWLIGHT_ACTORS` is declared rather than declined. The
//! claim is safe: `Arrow_l` appears nowhere else in `src/**`.
//!
//! The row existing is not the same as the actor surviving. Retail only ever creates this process
//! as a CHILD: `fopAcM_createChild(fpcNm_ARROW_LIGHTEFF_e, fopAcM_GetID(this), ...)` at
//! `d_a_arrow.cpp:156-157`. A stage-placed one takes the `fopAcM_CreateAppend()` path
//! (`f_op_actor_mng.cpp:2431-2444`), which sets `params->parent_id = fpcM_ERROR_PROCESS_ID_e`
//! (`:2441`), and `fopAcM_ct` copies that into `actor->parentActorID` (`f_op_actor.cpp:275`).
//!
//! The refusal here is NOT the `arrowice.rs` shape, and the distinction matters. `daArrow_Lighteff_c`
//! ::_create` (`:190-201`) does NOT test the parent: it calls `fopAcM_ct`, sets `field_0x2EA = 0`,
//! runs the solid heap and returns `cPhs_COMPLEATE_e`. A stage-placed `Arrow_l` therefore creates
//! successfully and runs its first `_execute`, where the parent lookup finally happens -
//! `daArrow_c* arrow = fopAcM_SearchByID(parentActorID);` at `:316` returns NULL and the `else`
//! arm at `:366-368` calls `fopAcM_delete(this)`. So the actor lives for exactly one tick.
//!
//! Per RULE 6b that is still modelled as spawn-then-despawn-on-the-first-tick through a `refused`
//! flag, not as `Option<Self> -> None`, which would leak a bare `Actor` retail never keeps.
//!
//! (a) inbound damage: NONE. (b) outbound damage: NONE. The class (`d_a_arrow_lighteff.h:20-43`)
//! holds no `dCcD` member of any kind and the profile's `Group` is `fopAc_ACTOR_e`
//! (`d_a_arrow_lighteff.cpp:431`). It registers no collider and never calls `ChkAtHit` or
//! `ChkTgHit`. Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
//!
//! `daArrow_Lighteff_IsDelete { return true; }` (`:406-408`) only ACCEPTS a delete request.
//!
//! Note for the duplicate-branch audit: `Arrow_l` is this module's own stage claim. It is NOT a
//! spawned child's `Actor.name` and must not be renamed.

use bevy_ecs::prelude::*;

/// `d_stage.cpp:1250` binds `Arrow_l` to `fpcNm_ARROW_LIGHTEFF_e`, the profile of
/// `d_a_arrow_lighteff.cpp:418-433`.
pub const ARROWLIGHT_ACTORS: [&str; 1] = ["Arrow_l"];

/// `fopAcM_setCullSizeBox(this, -100.0f, -100.0f, -100.0f, 100.0f, 100.0f, 100.0f)`
/// (`d_a_arrow_lighteff.cpp:135`); the profile is `fopAc_CULLBOX_CUSTOM_e` (`:432`), so this pair
/// IS the box.
pub const CULL_MIN: [f32; 3] = [-100.0, -100.0, -100.0];
pub const CULL_MAX: [f32; 3] = [100.0, 100.0, 100.0];

/// `fopAcM_setCullSizeFar(this, 1.0f)` (`d_a_arrow_lighteff.cpp:136`).
pub const CULL_FAR: f32 = 1.0;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrowLight {
    /// `fopAcM_SearchByID(parentActorID) != 0` (`d_a_arrow_lighteff.cpp:316-368`). `false` is the
    /// `else` arm's `fopAcM_delete(this)` at `:367`. The parent is identified by `fpc_ProcID`
    /// (`fopAc_ac_c::parentActorID`, `f_op_actor.h:286`); this sim's `Actor` carries neither a
    /// process id nor a parent link, so the stage-placed value is the only one reachable.
    parented: bool,
}

impl ArrowLight {
    /// `_create` (`:190-201`). The profile's `Parameters` field is `0`
    /// (`d_a_arrow_lighteff.cpp:427`) and the actor reads its own parameter only into
    /// `field_0x2E8` at `:79`, where it selects a model, an archive and a sound and nothing else,
    /// so `spawn` takes none.
    pub fn spawn() -> Self {
        Self { parented: false }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn cull_far(&self) -> f32 {
        CULL_FAR
    }
}

pub fn is_arrowlight(name: &str) -> bool {
    ARROWLIGHT_ACTORS.contains(&name)
}

pub fn control(effects: Query<(Entity, &ArrowLight)>, mut commands: Commands) {
    for (entity, effect) in &effects {
        if !effect.parented {
            let _ = commands.entity(entity).despawn();
        }
    }
}
