//! Arrow ice effects - `fpcNm_ARROW_ICEEFF_e`, `d_a_arrow_iceeff.cpp`, stage string `Arrow_i`
//! (`d_stage.cpp:1249`).
//!
//! (a) inbound damage: NONE. (b) outbound damage: NONE. `daArrow_Iceeff_c` is a pure visual child
//! of `daArrow_c`: `CreateHeap` (`:20-48`) allocates 30 `Link` models in mode 0 or one `Link`
//! model plus a BCK in mode 1, and the class (`d_a_arrow_iceeff.h:7-31`) holds no `dCcD` member
//! at all - the profile `Group` is `fopAc_ACTOR_e` (`:312`). Neither `combat::HitPoints` nor
//! `enemy::Hurt` is attached.
//!
//! The whole of this translation unit is unreachable from stage placement, and that is the one
//! behaviour that IS fully verifiable here. `dStage_actorInit` (`d_stage.cpp:1688-1711`) builds
//! every stage actor with `fopAcM_CreateAppend()` (`f_op_actor_mng.cpp:2431-2444`), which sets
//! `params->parent_id = fpcM_ERROR_PROCESS_ID_e` (`:2441`), and `dStage_actorCreate`
//! (`d_stage.cpp:1300-1312`) passes that straight to `fopAcM_Create` without touching
//! `parent_id`. `fopAcM_ct` copies it into `actor->parentActorID` (`f_op_actor.cpp:275`). So a
//! stage-placed `Arrow_i` always has `parentActorID == -1`, `fopAcM_SearchByID(parentActorID)`
//! (`d_a_arrow_iceeff.cpp:113`) returns NULL, and `_create` returns `cPhs_ERROR_e` (`:115`) -
//! which cancels the request inside Create (`f_pc_create_req.cpp:106-108`) so the actor NEVER
//! executes even once. Per RULE 6b that is a spawn-then-despawn-on-the-first-tick, not an
//! `Option<Self> -> None`.
//!
//! In retail this actor is only ever created as a CHILD: `fopAcM_createChild(fpcNm_ARROW_ICEEFF_e,
//! fopAcM_GetID(this), ...)` at `d_a_arrow.cpp:394` (an ice arrow that hit water) and `:800` (an
//! ice arrow that stuck in the ground). `src/sim/arrow.rs` reproduces neither, so the child case
//! does not happen here either.

use bevy_ecs::prelude::*;

/// `d_stage.cpp:1249` binds `Arrow_i` to `fpcNm_ARROW_ICEEFF_e`, the profile of
/// `d_a_arrow_iceeff.cpp:303-318`.
pub const ARROWICE_ACTORS: [&str; 1] = ["Arrow_i"];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrowIce {
    /// `fopAcM_SearchByID(parentActorID) != 0` (`d_a_arrow_iceeff.cpp:113-117`). `false` is the
    /// `return cPhs_ERROR_e` refusal at `:115`. The parent is identified by `fpc_ProcID`
    /// (`fopAc_ac_c::parentActorID`, `f_op_actor.h:286`) and this sim's `Actor` carries no
    /// process id and no parent link, so the stage-placed value is the only one reachable.
    parented: bool,
}

impl ArrowIce {
    /// `_create` (`:110-132`). The profile's `Parameters` field is `0`
    /// (`d_a_arrow_iceeff.cpp:306`) and the actor never reads its own parameter - it reads the
    /// PARENT arrow's at `:118` - so `spawn` takes none.
    pub fn spawn() -> Self {
        Self { parented: false }
    }
}

pub fn is_arrowice(name: &str) -> bool {
    ARROWICE_ACTORS.contains(&name)
}

pub fn control(ices: Query<(Entity, &ArrowIce)>, mut commands: Commands) {
    for (entity, ice) in &ices {
        if ice.parented {
            continue;
        }
        let _ = commands.entity(entity).despawn();
    }
}
