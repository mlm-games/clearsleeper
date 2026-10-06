//! The King's Sail cloth - `fpcNm_GRID_e`, `d_a_grid.cpp`, the actor behind the stage string
//! `Grid`.
//!
//! WHY THIS FILE HAS NO `control` SYSTEM. The actor's only per-tick behaviour is `ho_move`
//! (`:359-607`), and every term in it is driven by the wind:
//!
//! * `cXyz* windVec = dKyw_get_wind_vec();` (`:386`) and `f32 windPow = dKyw_get_wind_pow();`
//!   (`:387`) feed `windAngle = cM_atan2s(windVec->x, windVec->z)` (`:395`), the sway increment
//!   `temp_r0_2 = 2500.0f + 9000.0f * (0.8f * windPow)` (`:431`), and the per-vertex gains
//!   `x_value = 0.25f + 0.75f * windPow` (`:485`) and `z_value = 0.5f + 0.25f * windPow` (`:488`).
//!   Both symbols are config-only with no value in this checkout - UNKNOWN, not proven zero - so
//!   folding them is invented data and the 85-vertex solve has no inputs.
//! * `l_ship->getSailAngle()` (`:393`, `:412`, `:833`), `l_ship->getSailOn()` (`:450`) and
//!   `l_ship->tevStr` (`:827`) come from the `daShip_c` found by
//!   `fopAcM_SearchByName(fpcNm_SHIP_e)` (`:761`); this sim's `super::ship::Ship` carries no sail
//!   angle and no sail-on flag.
//! * The mesh itself is `l_pos[85]`, `l_texCoord` and `l_DL`, which live in
//!   `assets/l_pos__d_a_grid.h` - not in this checkout - and `_create` (`:630-769`) derives
//!   `field_0x1b54[85]` from `l_pos` at `:674-759`.
//! * `_execute`'s alpha ramp (`:786-811`) needs `dComIfGp_getCamera(0)->mCamera.Eye()` (`:790`).
//!   `kytag00::CameraEye` is the recorded WRITE-NEVER stand-in for a camera position and reading
//!   it as one is banned.
//! * `scale.y < 0.06f` (`:813`) never fires: RULE 2, stage-placed actors have scale exactly 1.0
//!   because `dStage_actorInit` never writes `fopAcM_prm_class::scale`.
//! * `daHo_packet_c::setNrmVtx` / `setTopNrmVtx` / `setBackNrm` (`:121-213`) and the whole of
//!   `daHo_packet_c::draw` (`:220-350`) are `J3D` normal and GX work - a legitimate omission under
//!   RULE 3b.
//! * `field_0x1b48 = fopAcM_GetParam(this)` (`:632`, `:655`) has no reader anywhere in the
//!   translation unit, so the profile's `Parameters` field being `0` (`:861`) costs nothing.
//!
//! (a) inbound damage: NONE. (b) outbound damage: NONE. `daGrid_c` registers no `dCcD` collider -
//! its `mGndChk` is an `OffWall` ground probe (`:280`) - and the profile `Group` is
//! `fopAc_ACTOR_e` (`:866`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
//!
//! `daGrid_IsDelete` (`:615-617`) is the bare "accepts a delete request" form of RULE 6b.
//! `_create` returns `cPhs_ERROR_e` only from the two `dComIfG_resLoad` tests (`:636-646`, DEMO
//! only) and the archive phase returns, all heap work, so `spawn` never refuses.

use bevy_ecs::prelude::*;

/// `OBJNAME("Grid", fpcNm_GRID_e, 255, 0)` (`d_stage.cpp:1251`) is the only `fpcNm_GRID_e` row.
/// The object archives are `"Cloth"` and `"Ship"` (`:657`, `:662`), neither of which is the stage
/// string.
pub const GRID_ACTORS: [&str; 1] = ["Grid"];

/// `fopAc_CULLBOX_0_e` (`:867`) resolves through `l_cullSizeBox[0]` (`f_op_actor_mng.cpp:558`);
/// this file calls neither `fopAcM_setCullSizeBox` nor `fopAcM_SetMin` / `SetMax`.
const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)

/// The sail. Unit struct: `daGrid_c`'s every field is written by `ho_move` and read by the
/// renderer or by the wind-only solve, so none survives RULE 12b.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grid;

impl Grid {
    /// `_create` (`:630-769`) minus the two `dComIfG_resLoad` calls (`:657-665`), `fopAcM_ct`
    /// (`:652`), the `mDoHIO_createChild("船の帆", &l_HIO)` (`:668-670`), the `field_0x1b54[85]`
    /// solve (`:674-759`), the `fopAcM_SearchByName` (`:761`) and `ho_move` (`:762`).
    pub fn spawn(_parameters: u32) -> Self {
        Grid
    }

    /// `fopAc_CULLBOX_0_e` (`:867`) -> `l_cullSizeBox[0]` (`f_op_actor_mng.cpp:558`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_grid(name: &str) -> bool {
    GRID_ACTORS.contains(&name)
}