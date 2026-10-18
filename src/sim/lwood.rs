use bevy_ecs::prelude::*;

/// The stage string is `lwood`, LOWERCASE L (`d_stage.cpp:836`, `fpcNm_Lwood_e`). It is NOT
/// `daLwood_c::m_arcname`, which is `"Lwood"` (`d_a_lwood.cpp:14`): `dStage_searchName` compares
/// with `strcmp` (`d_stage.cpp:1270`), which is case-sensitive, so the stage file spells it
/// `lwood` and this array must carry that, not the archive name. No other module in `src/**`
/// claims either spelling.
pub const LWOOD_ACTORS: [&str; 1] = ["lwood"];

/// `fopAcM_setCullSizeBox(this, -600.0f, -0.0f, -600.0f, 600.0f, 900.0f, 600.0f)`
/// (`d_a_lwood.cpp:44`) - the actor's one real custom cull box, matching the profile's
/// `fopAc_CULLBOX_CUSTOM_e` (`:199`). It is numerically the shared table's
/// `fopAc_CULLBOX_8_e` (`f_op_actor_mng.cpp:566`) apart from `-0.0f` vs `0.0f`, but the profile
/// says CUSTOM, so the `setCullSizeBox` call at `:44` is the authority.
/// `fopAcM_setCullSizeFar(this, 2.37f)` (`:45`) is the far clip - `fopAcM_cullingCheck`
/// (`f_op_actor_mng.cpp:589+`) only uses it to pick the clipper, and `view::ActorView` carries no
/// far field, so it is documented here rather than stored.
const CULL_MIN: [f32; 3] = [-600.0, -0.0, -600.0]; // <- d_a_lwood.cpp:44
const CULL_MAX: [f32; 3] = [600.0, 900.0, 600.0]; // <- d_a_lwood.cpp:44

/// The normal forest tree - `daLwood_c` (`d_a_lwood.h:9-32`), the actor behind the stage string
/// `lwood`.
///
/// The struct has NO fields, because retail behaviour here is entirely wind sway and the sway has
/// no inputs:
///
/// * `daLwood_c::CreateInit` (`d_a_lwood.cpp:42-60`) draws a `mTimer = cM_rndF(0x8000)` (`:46`,
///   assigned to the `s16 mTimer` at `d_a_lwood.h:30`) and a `mScale = cM_rndF(0.4f) + 0.8f`
///   (`:47`, the `f32 mScale` at `d_a_lwood.h:31` - note the `cM_rndF(range) + low` form, not
///   `low + rnd * |high - low|`), installs `nodeCallBack` on the `J_Alwd_ha` joint (`:48-54`) and
///   registers `mpBgW` (`:57`).
/// * Both fields are read ONLY by `nodeCallBack` (`:63-95`) through `getYureTimer()` /
///   `getYureScale()` (`d_a_lwood.h:15-16`), and that callback's first line is
///   `cXyz windSpeed = daObj::get_wind_spd(i_this, 100.0f)` (`:72`), which is
///   `(dKyw_get_wind_vec() * dKyw_get_wind_pow() + pointWind) * (100.0f * 0.5f)`
///   (`d_a_obj.cpp:62-74`). BOTH inputs are missing here, and neither is a proven-zero fold:
///   `dKyw_get_wind_vec`, `dKyw_get_wind_pow` and `dKyw_get_wind_power` are declared at
///   `d_kankyo_wether.h:361-366` and defined NOWHERE in `src/`, and `pointWind` comes from
///   `dKyw_pntwind_get_info` (`d_a_obj.cpp:69`), which walks `g_env_light.mpWindInfluence[]`
///   (`d_kankyo_wether.cpp:1215-1226`) - the per-stage environment wind volumes. RULE 2b does not
///   cover these the way it covers `REGn_F(i)`, so folding them would be the invented data RULE 1
///   forbids.
/// * `nodeCallBack` is in any case a `J3DNodeCBCalcTiming_In` J3D hook that rewrites
///   `model->setAnmMtx` and `j3dSys.mCurrentMtx` (`:87-90`) - renderer work, a legitimate
///   omission under RULE 3b.
///
/// With every reader of `mTimer` and `mScale` inside the omitted callback, neither field is
/// stored: an unread field is dead code (RULE 7 / RULE 12b). `daLwood_c::_execute` (`:137-140`) is
/// therefore `mTimer++; return TRUE;` with nothing downstream of it, so NO `control` system is
/// derivable - the same shape `stone.rs` uses, and the outcome PORT_BRIEF:50 names for `lpalm` and
/// `lwood`.
///
/// `daLwood_IsDelete` (`:172-175`) is the bare "accepts a delete request" form of RULE 6b, and
/// `_create` (`:113-127`) returns `cPhs_ERROR_e` only from `fopAcM_entrySolidHeap` (`:119-121`),
/// which is heap work.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daLwood_c` holds only `mPhs`, `mModel`,
/// `mpBgW`, `mtx`, `mTimer` and `mScale` (`d_a_lwood.h:26-31`) and registers no `dCcD` collider;
/// `mpBgW` is a `dBgW` moving background (RULE 3b), not a `dCcD` collider. The profile `Group` is
/// `fopAc_ACTOR_e` (`:198`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Lwood;

impl Lwood {
    /// `daLwood_c::_create` (`:113-127`), minus `fopAcM_ct` (`:114`), `dComIfG_resLoad` (`:116`),
    /// `fopAcM_entrySolidHeap` / `CheckCreateHeap` / `CreateHeap` (`:119`, `:17-37`) and
    /// `CreateInit` (`:122`) - all archive, heap and renderer work. The profile's `Parameters`
    /// field is `0` (`d_a_lwood.cpp:193`), so `daLwood_c` carries no parameter to decode.
    pub fn spawn(_parameters: u32) -> Self {
        Lwood
    }

    /// `fopAcM_setCullSizeBox(this, -600.0f, -0.0f, -600.0f, 600.0f, 900.0f, 600.0f)`
    /// (`d_a_lwood.cpp:44`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_lwood(name: &str) -> bool {
    LWOOD_ACTORS.contains(&name)
}
