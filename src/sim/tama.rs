use bevy_ecs::prelude::*;

pub const TAMA_ACTOR: &str = "Tama";

/// `daTama_c` is only ever built by `fopAcM_ct` plus the stage object table entry
/// `OBJNAME("Tama", fpcNm_TAMA_e, 255, 0)` (`tww/src/d/d_stage.cpp:1236`): nothing in the decomp
/// calls `setPartnerID`, `setDis` or `setSpd` (`tww/include/d/actor/d_a_tama.h:10-12`), so
/// `mPartnerID`, `mDis` and `speedF` keep the `sBs_ClearArea` zero-fill from `fpcBs_Create`
/// (`tww/src/f_pc/f_pc_base.cpp:98`).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Tama;

impl Tama {
    pub fn spawn() -> Self {
        Self
    }

    /// `daTama_c::_execute` (`tww/src/d/actor/d_a_tama.cpp:56`).
    ///
    /// `fopAcM_SearchByID(mPartnerID)` resolves no partner here: `mPartnerID` is the zero-fill
    /// `0`, `fpcM_ERROR_PROCESS_ID_e` is `-1` (`tww/include/f_pc/f_pc_manager.h:11`) so `0` is a
    /// live id, and picking whichever process owns it is not derivable from the decomp. Even if
    /// one were resolved, the follow test at line 62 is `distXZ < mDis` with `mDis == 0.0f`, which
    /// can never pass. Either way `del` stays `TRUE` and line 96 calls `fopAcM_delete(this)`.
    fn execute(&self) -> bool {
        true
    }
}

pub fn is_tama(name: &str) -> bool {
    name == TAMA_ACTOR
}

pub fn control(mut tamas: Query<(Entity, &Tama)>, mut commands: Commands) {
    for (entity, tama) in &mut tamas {
        if tama.execute() {
            let _ = commands.entity(entity).despawn();
        }
    }
}