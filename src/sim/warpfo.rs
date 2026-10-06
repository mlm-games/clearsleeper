use bevy_ecs::prelude::*;

pub const WARPFO_ACTORS: [&str; 1] = ["Warpfo"];

/// `daWarpfout_c` (`d_a_warpfout.cpp`), the stage-event driver for the Windfall demonstration
/// warp-out. The stage string is `Warpfo` (`d_stage.cpp:1161`).
///
/// `_create` (`:39-45`) is unconditionally `cPhs_COMPLEATE_e` - it only calls `fopAcM_ct` and
/// `CreateInit` - so there is no refusal and no spawn-then-despawn path. `Mthd_IsDelete`
/// (`:207-209`) is a bare TRUE - an accepted delete request, never a self-destruct.
///
/// `daWarpfout_Draw` (`:195-197`) returns TRUE with no draw call at all, so retail renders nothing
/// and there is no cull box to transcribe either: the profile is `fopAc_CULLBOX_CUSTOM_e` (`:233`)
/// but `fopAcM_setCullSizeBox` is never called anywhere in the file, so this module deliberately
/// offers no `cull()` rather than inventing a box.
///
/// `_execute` (`:47-54`) calls `checkOrder()` (`:40`, body empty at `:125-126`), `demo_proc()`
/// (`:41`) and `eventOrder()` (`:42`, body empty at `:129-130`). `demo_proc` (`:59-79`) opens with
/// `mStaffId = dComIfGp_evmng_getMyStaffId("Warpfo")` (`:60`), the stage event-info table lookup by
/// name, which is a confirmed-absent input and always yields `-1`; that `-1` fails the
/// `mStaffId != -1` guard (`:62`), so neither `event_init_tbl`/`event_action_tbl` dispatch (`:69-73`),
/// `dComIfGp_evmng_cutEnd` (`:67`, `:74`) nor `actEnd`'s `fopAcM_delete(this)` (`:156-159`) ever
/// runs and the actor never deletes itself. The whole `action_table` - `"WARP1"`, `"WARP2"`,
/// `"WARP3"`, `"WARP4"`, `"END"` (`:20-22`) - and its ten `initWarpN`/`actWarpN` members
/// (`:82-159`) are event-driven wind particles, `mTimer` and `cLib_calcTimer` with no reachable
/// caller. `CreateInit` (`:32-37`) writes `unknownInitFlag = -0x1` and nothing reads it, so this
/// actor keeps no state at all.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Warpfo;

impl Warpfo {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_warpfo(name: &str) -> bool {
    WARPFO_ACTORS.contains(&name)
}

pub fn control(_warpers: Query<&Warpfo>) {}