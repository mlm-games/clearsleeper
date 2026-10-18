use bevy_ecs::prelude::*;

use super::flags::{Collect, SYMBOL_DIN};

pub const QUAKE_ACTORS: [&str; 1] = ["Quake"];

const TYPE_SHIFT: u32 = 8; // <- d_a_obj_quake.h:14 (PRM_TYPE_S)
const TYPE_MASK: u32 = 0x3; // <- d_a_obj_quake.h:13 (PRM_TYPE_W), d_a_obj.h:14
const TYPE_LIMIT: u8 = 3; // <- d_a_obj_quake.cpp:35

/// The stage string is `Quake` (`d_stage.cpp:787`); the profile has no `_arcname`, and
/// `daObjQuakeDraw` (`:210-212`) returns TRUE with no draw call, so nothing is rendered and no
/// `actor_views` branch is derivable even though the profile names `fopAc_CULLBOX_0_e`
/// (`:241`).
///
/// `daObjQuake_c::_create` (`:28-64`) refuses creation twice, and both refusals cancel the
/// request inside Create exactly like a normal `ERROR` (`f_pc_create_req.cpp:106-108`):
///
/// * `cPhs_STOP_e` when the Din's Pearl is already held (`:31-33`), which is
///   `dComIfGs_isSymbol(dSymbol_DIN_e)`;
/// * `cPhs_ERROR_e` when `getPrmType() >= 3` (`:63`), i.e. `>= 0x3` is `daObj::PrmAbstract<Prm_e>
///   (this, PRM_TYPE_W = 0x3, PRM_TYPE_S = 0x8)` (`d_a_obj_quake.cpp:180-182`).
///
/// Both are checked at creation, so they belong on the component and the term is dropped on
/// its first tick - not in `spawn`, which must not leak a bare `Actor`.
///
/// `_execute` (`:96-177`) has no portable arm. Its only gate is
/// `sch & dKy_get_schbit() && !dComIfGp_event_chkEventFlag(dEvtFlag_TALK_e)` (`:139`), and
/// `chkEventFlag` (`:139`, `d_event.h:128`) reads `dEvt_control_c::mEventFlag`, which only
/// `doorCheck` ever sets bit `0x04` of (`d_event.cpp:379`, for a `dEvtType_TREASURE_e` order)
/// and `endProc` clears (`d_event.cpp:423`, `:695`). The sim keeps no such word. Dropping that
/// conjunct would invent a camera shake that retail suppresses, so the whole `var_r27`
/// computation and the `m2A1` start/stop machine it drives (`:147-174`) are omitted. With it
/// go `m294 = dKy_get_schbit_timer()` (`:100`), `m290 = 30.0f * dStage_stagInfo_GetSchSec(...)`
/// (`:41`), the `m298` / `m29C` per-type table (`:43-53`), `getPrmSch` (`:185-187`) and
/// `getPrmPower` (`:190-192`) - all written once and read only by the omitted arm, so none of
/// them is stored. `l_HIO` (`:11`) is a JOR child whose `genMessage` is `UNUSED` and therefore
/// empty (`d_a_obj_quake.h:42`), so its fields keep the constructor values (`:14-25`) and
/// `field_0x08` is only the instance counter `_is_delete` decrements (`:84-90`); `StartQuake`
/// / `StopQuake` (`:155`, `:158`, `:166`, `:169`) and `JA_SE_ATM_JINARI` (`:149`) are all
/// downstream of the omitted gate.
///
/// `daObjQuake_c::_is_delete` (`:72-93`) only stops a running quake and releases the HIO child,
/// both unmodelled, so it stays a bare TRUE - an accepted delete request, never a self-destruct.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quake {
    refused: bool,
}

impl Quake {
    pub fn spawn(parameters: u32, collect: &Collect) -> Self {
        Self {
            refused: collect.is_symbol(SYMBOL_DIN)
                || ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8 >= TYPE_LIMIT,
        }
    }
}

pub fn is_quake(name: &str) -> bool {
    QUAKE_ACTORS.contains(&name)
}

pub fn control(quakes: Query<(Entity, &Quake)>, mut commands: Commands) {
    for (entity, quake) in &quakes {
        if quake.refused {
            let _ = commands.entity(entity).despawn();
        }
    }
}
