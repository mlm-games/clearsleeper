//! `daObjCorrect::Act_c` (`d_a_obj_correct.h:14-81`) - the Outset Island / Wind Temple "corrector"
//! markers. `Nonmatching` count: 0, so every token below is transcribed.
//!
//! Nothing in `src/sim/` claims any of these eight stage strings.

use bevy_ecs::prelude::*;

/// `OBJNAME("BLK_CR", fpcNm_Obj_Correct_e, 0, 0)` (`d_stage.cpp:556`) through
/// `OBJNAME("CrTrBl", fpcNm_Obj_Correct_e, 7, 61)` (`:563`). `fpcNm_Obj_Correct_e` is
/// `d_a_obj_correct.cpp` and the OBJNAME table is the authority for the spelling, so the array
/// carries the stage strings and not `daObjCorrect`'s archive name (there is no `m_arcname`).
pub const OBJ_CORRECT_ACTORS: [&str; 8] = [
    "BLK_CR", "CrTrS3", "CrTrS4", "CrTrS5", "CrTrM1", "CrTrM2", "CrTrGr", "CrTrBl",
];

/// `l_cullSizeBox[0]` - `/* fopAc_CULLBOX_0_e */ fopAc_cullSizeBox(cXyz(-40.0f, 0.0f, -40.0f),
/// cXyz(40.0f, 125.0f, 40.0f))` (`f_op_actor_mng.cpp:558`). The profile's `fopAc_CULLBOX_0_e`
/// (`d_a_obj_correct.cpp:442`) resolves here; the file calls neither `fopAcM_setCullSizeBox` nor
/// `fopAcM_SetMin` / `SetMax`, so that table entry is the whole box.
const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:558
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:558

/// `daObjCorrect::Prm_e::PRM_SWSAVE_S = 0x08` (`d_a_obj_correct.h:23`) feeding
/// `daObj::PrmAbstract(this, W, S)` = `((1 << W) - 1) & (param >> S)` (`d_a_obj.h:14`), with
/// `PRM_SWSAVE_W = 0x08` (`:24`) - the whole byte, no mask beyond the shift.
const SWSAVE_SHIFT: u32 = 0x08; // <- d_a_obj_correct.h:23
/// `if (swSave == 0xFF) { return cPhs_ERROR_e; }` (`d_a_obj_correct.cpp:52`). The test is against
/// the raw `int` the accessor returns, so the sentinel is the full decoded byte.
const NO_SWITCH: u8 = 0xFF; // <- d_a_obj_correct.cpp:52

/// `daObjCorrect::Attr_c L_attr[8]` (`d_a_obj_correct.cpp:30-39`) - a file-scope table, not a
/// per-actor field, and it is documented rather than stored because every one of its four columns
/// has its only reader inside an omitted body (see the OMITTED list on [`ObjCorrect`]):
///
/// * row 0 `{NULL, 9.0f, 16.0f, 0.0f}` (`:31`), rows 1-3 `{NULL, 2500.0f, 2601.0f, 0.0f}`
///   (`:32-34`), rows 6-7 `{NULL, 9.0f, 100.0f, 3600.0f}` (`:37-38`) - `field_0x4` / `field_0x8`
///   are written by `mode_on_init` (`:290`) / `mode_off_init` (`:262`) into `field_0x2A4` and are
///   read only by the eight `search_*` callbacks' `correct->field_0x2A4` arguments (`:132`, `:155`,
///   `:167`, `:179`, `:200`, `:223`, `:244`), all omitted.
/// * rows 4-5 carry `L_ev_CrTrM1 = "CrTrM1_put"` (`:27`) and `L_ev_CrTrM2 = "CrTrM2_put"` (`:28`).
///   The only read of `attr(mType).name != 0` is `_create`'s
///   `dComIfGp_evmng_getEventIdx(attr(mType).name, evId)` (`:65-70`), i.e. the stage event-info
///   table, which RULE 3b lists as absent; `mEventIdx` is then only read by `demo_req_init`
///   (`:343`), `demo_run` (`:368`) and `eventOrder`-style order calls, all omitted.
/// * `field_0xC` (0.0 / 5000.0 / 3600.0) is `i_depression_r_sq` in `search_tryKeyGate` (`:201`),
///   `search_tryKeyDoor` (`:224`) and `search_tryColGreen` (`:244`) - omitted with them.
///
/// `daObjTry::Type_e UNK_2 / UNK_3 / UNK_4` (`:155`, `:167`, `:179`), `UNK_7..UNK_12` in
/// `search_tryColGreen`'s `static const daObjTry::Type_e type[6]` (`:233-236`), the bare `5` / `6`
/// compares at `:330`, and `daObjCorrect::Type_e UNK_4 / UNK_5` (`d_a_obj_correct.h:9-12`) are all
/// `daObjTry` selectors; `fpcNm_Obj_Try_e` (`d_stage.cpp:925-939`, fifteen `Hseki*` / `Hmon*`
/// strings) has no module in this crate, so every consumer of them is omitted.
///
/// Everything the actor actually DOES per tick is therefore absent, and the struct holds the one
/// observable fact instead:
///
/// * `mode_off` (`:279-286`) and `mode_on` (`:295-302`) both run
///   `fopAcM_Search(M_search_proc[mType], (void*) this)` (`:281`, `:297`) and branch on the
///   `field_0x290` side channel plus the returned actor. `M_search_proc` (`:267-276`) is
///   `search_movebox` / `search_tryColSun` / `search_tryColMer` / `search_tryColJup` /
///   `search_tryKeyGate` / `search_tryKeyDoor` / `search_tryColGreen` / `search_tryColBlue`.
///   `fopAcM_Search` is `fopAcIt_Judge` over the process list in registration order
///   (`f_op_actor_mng.h`), and `search_movebox` (`:124-147`) is order-SENSITIVE: `field_0x290` is
///   overwritten by every `fpcNm_Obj_Movebox_e` it visits (`:134`, `:138`, `:142`) while only the
///   FIRST match becomes `res`. ECS iteration order does not reproduce the retail process list,
///   and `daObjMovebox::prm_get_dmy()` (`:133`) is not exposed by `src/sim/movebox.rs`, so
///   neither search is transcribed - substituting a "first match wins" scan would be the
///   equivalent behaviour RULE 1 forbids.
/// * `demo_non` (`:310-339`) needs `player->getGrabPutStart()` / `getGrabActorID()` and
///   `fopAcM_SearchByID`; `demo_req_init` (`:342-350`), `demo_req` (`:353-359`) and `demo_run`
///   (`:367-372`) need `dComIfGp_evmng_existence` / `dComIfGp_event_reset`, the stage event-info
///   table (RULE 3b). `super::player::Player` carries no grab-put-start flag and no grab actor id.
///
/// With every `field_0x290` / `field_0x294` / `field_0x298` / `field_0x2A4` / `mType` / `mEventIdx`
/// reader omitted, none of them is stored (RULE 7: an unread field is dead code).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjCorrect {
    /// `cPhs_ERROR_e` from `_create` (`d_a_obj_correct.cpp:53`) cancels the process during Create
    /// (`f_pc_create_req.cpp:106-108`), so the actor never executes even once. RULE 6b: spawn it,
    /// then despawn on the first tick - never `Option<Self> -> None`.
    pub refused: bool,
}

impl ObjCorrect {
    /// `daObjCorrect::Act_c::_create` (`d_a_obj_correct.cpp:47-74`), minus `prm_get_type` (`:56`),
    /// `fopAcM_offDraw` (`:57`), the `fopAcM_isSwitch` test and the `mode_on_init` /
    /// `mode_off_init` pair (`:59-63`), the `attr(mType).name` / `dComIfGp_evmng_getEventIdx`
    /// pair (`:65-70`) and `demo_non_init` (`:71`) - all of which only feed the omitted bodies.
    /// The profile's `Parameters` field is `0` (`:436`), so `daObjCorrect::Act_c` still carries a
    /// parameter word; only `prm_get_swSave` (`:48`) is decoded here.
    pub fn spawn(parameters: u32) -> Self {
        let sw_save = ((parameters >> SWSAVE_SHIFT) & 0xFF) as u8;
        Self {
            refused: sw_save == NO_SWITCH,
        }
    }

    /// `l_cullSizeBox[0]` via the profile's `fopAc_CULLBOX_0_e` (`d_a_obj_correct.cpp:442`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daObjCorrect::Act_c` holds six scalars
/// (`d_a_obj_correct.h:68-73`) and registers no `dCcD` collider; the profile `Group` is
/// `fopAc_ACTOR_e` (`:441`) and `_create` calls `fopAcM_offDraw(this)` (`:57`), so the actor draws
/// nothing either. Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
pub fn is_obj_correct(name: &str) -> bool {
    OBJ_CORRECT_ACTORS.contains(&name)
}

/// `_execute` (`d_a_obj_correct.cpp:375-389`) dispatches `mode_proc[field_0x294]` (`:376-380`)
/// then `demo_proc[field_0x298]` (`:382-387`); both tables resolve to bodies whose inputs are
/// absent (documented on [`ObjCorrect`]), so the only surviving behaviour is the Create refusal.
/// `Mthd_IsDelete` is the bare `return TRUE` form (`:413-416`) - RULE 6b, it ACCEPTS a delete
/// request and is not a despawn.
pub fn control(correctors: Query<(Entity, &ObjCorrect)>, mut commands: Commands) {
    for (entity, correct) in &correctors {
        if correct.refused {
            let _ = commands.entity(entity).despawn();
        }
    }
}
