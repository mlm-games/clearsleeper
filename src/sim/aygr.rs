use bevy_ecs::prelude::*;

/// `OBJNAME("Aygr", fpcNm_Obj_Aygr_e, 255, 0)` (`d_stage.cpp:536`) - the Lookout Platform.
pub const AYGR_ACTORS: [&str; 1] = ["Aygr"];

/// `enum Prm_e { PRM_MDL_W = 1, PRM_MDL_S = 0 }` (`d_a_obj_aygr.h:9-10`), fed through
/// `daObj::PrmAbstract` (`d_a_obj.h:14-17`).
const MDL_WIDTH: u32 = 0x01; // <- d_a_obj_aygr.h:9 (PRM_MDL_W)
const MDL_SHIFT: u32 = 0x00; // <- d_a_obj_aygr.h:10 (PRM_MDL_S)

/// `fopAcM_setCullSizeBox(this, -500.0f, -100.0f, -500.0f, 500.0f, 4000.0f, 500.0f)` (`:51`).
const CULL_MIN: [f32; 3] = [-500.0, -100.0, -500.0]; // <- d_a_obj_aygr.cpp:51
const CULL_MAX: [f32; 3] = [500.0, 4000.0, 500.0]; // <- d_a_obj_aygr.cpp:51

/// A Lookout Platform - `daObjAygr::Act_c` (`d_a_obj_aygr.h:5-35`), the stage string `Aygr`.
///
/// The class is a `dBgS_MoveBgActor` (`d_a_obj_aygr.h:5`). Both models (`M_arcname` `"Aygr"`,
/// `d_a_obj_aygr.cpp:10`), the two `dBgW`s and the optional ladder in
/// `dRes_INDEX_AYGR_BDL_AYGRH_e` are renderer / moving-background data, which RULE 3b lists as
/// absent. `Execute` (`:113-117`) does nothing but `set_mtx` and publish the matrix, so the sim
/// only carries the spawn parameter and the cull box the viewport reads.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `CreateHeap` (`:15-64`) and `Create`
/// (`:67-72`) register no `dCcD` collider; the profile's `Group` is `fopAc_ACTOR_e` (`:145`).
/// Neither `HitPoints` nor `Hurt` is attached.
///
/// `Mthd_Create` can only fail in resource load / `MoveBGCreate` (`:75-98`), so `spawn` never
/// refuses. `Delete` returns `TRUE` (`:101-103`).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Aygr {
    /// `prm_get_mdl()` (`d_a_obj_aygr.h:11`) - true also builds the climbable ladder model and
    /// its `dBgW` (`:24-61`).
    pub ladder: bool,
}

impl Aygr {
    /// `daObjAygr::Act_c::Mthd_Create` (`d_a_obj_aygr.cpp:75-98`), minus the resource load,
    /// `MoveBGCreate` and the heap entry - all renderer / background work.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            ladder: ((parameters >> MDL_SHIFT) & ((1 << MDL_WIDTH) - 1)) != 0,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_aygr(name: &str) -> bool {
    AYGR_ACTORS.contains(&name)
}
