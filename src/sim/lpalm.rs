use bevy_ecs::prelude::*;

/// The stage string is `Oyashi` (`d_stage.cpp:638`, `fpcNm_Obj_Lpalm_e`). It happens to match
/// `daObjLpalm_c::M_arcname`, which is also `"Oyashi"` (`d_a_obj_lpalm.cpp:13`).
pub const LPALM_ACTORS: [&str; 1] = ["Oyashi"];

/// `fopAcM_setCullSizeBox(this, -350.0f, -50.0f, -350.0f, 350.0f, 1300.0f, 350.0f)`
/// (`d_a_obj_lpalm.cpp:72`) - the actor's one real custom cull box, matching the profile's
/// `fopAc_CULLBOX_CUSTOM_e` (`:216`). `fopAcM_setCullSizeFar(this, 2.37f)` (`:73`) is the far clip
/// - `fopAcM_cullingCheck` (`f_op_actor_mng.cpp:589+`) uses it only to pick the clipper and
/// `view::ActorView` carries no far field, so it is documented here rather than stored.
const CULL_MIN: [f32; 3] = [-350.0, -50.0, -350.0]; // <- d_a_obj_lpalm.cpp:72
const CULL_MAX: [f32; 3] = [350.0, 1300.0, 350.0]; // <- d_a_obj_lpalm.cpp:72

/// `daObjLpalm_c::M_attr = {0, 0}` (`d_a_obj_lpalm.cpp:14`, `d_a_obj_lpalm.h:22-25`) is a
/// file-scope constant, not a per-actor field, and both of its bytes are zero - which is why
/// `_execute`'s `if (attr().flag0) return true;` (`:125-126`) never short-circuits and `_draw`'s
/// `if (attr().flag1) return true;` (`:174-175`) never short-circuits either. Neither guard is
/// stored: both are compile-time false and would be dead fields.

/// The palm tree - `daObjLpalm_c` (`d_a_obj_lpalm.h:10-39`), the actor behind the stage string
/// `Oyashi`.
///
/// The struct has NO fields: `daObjLpalm_c::_execute` (`:124-171`) is one unbroken wind
/// computation and both of its inputs are absent.
///
/// * `cMtx_YrotS(*calc_mtx, -current.angle.y)` (`:132`) then
///   `MtxPosition(dKyw_get_wind_vec(), &windDir)` (`:133`) and
///   `f32 windPow = dKyw_get_wind_pow()` (`:134`). `dKyw_get_wind_vec` and `dKyw_get_wind_pow`
///   are config symbols with no value in this checkout (RULE 3b) - unlike `REG0_F(i)`, which RULE
///   2b proves is zero, these are genuinely unknown, so folding them to a literal would be the
///   invented data RULE 1 forbids.
/// * Everything downstream - the `up.outprod(windDir)` / `dir.abs() < 8e-09` /
///   `mBaseQuatTarget` quaternion build (`:136-150`), the
///   `mDoMtx_quatSlerp(&mBaseQuat, &mBaseQuatTarget, &q, 0.25f)` (`:152-153`) and the
///   `cLib_addCalcAngleS2` / `cM_rndFX(128.0f)` / `cM_ssin` per-frond loop (`:155-167`) - consumes
///   only `windDir` and `windPow`. With them gone the whole body is unevaluable, so none of
///   `mBaseQuat`, `mBaseQuatTarget`, `mAnmMtxQuat[2]`, `mAnimDir[2]` or `mAnimWave[2]`
///   (`d_a_obj_lpalm.h:26-35`) can be stored: every one of them is written and read only by the
///   omitted wind body and by `nodeCallBack` (`:22-39`), a `J3DNodeCBCalcTiming_In` J3D hook that
///   only rewrites `model->setAnmMtx` (`:35`).
/// * `CreateInit`'s `mAnimWave[1] = cM_rndFX(0x8000)` (`:70`) and the rest of the quaternion
///   initialisation (`:63-66`) are initialisers of those omitted fields.
/// * `mpBgW->Move()` (`:169`) is a `dBgW` moving background (RULE 3b), and `_execute` returns
///   `false` (`:170`) - `false` from `Execute` is not a delete request, only `IsDelete` and
///   `fopAcM_delete` are (RULE 6b).
/// * `_draw` (`:173-181`) is `TEV_TYPE_BG0` tev strategy plus `mDoExt_modelUpdateDL` - renderer.
/// * `CreateHeap` (`:42-59`) installs `nodeCallBack` on every joint and builds `mpBgW` - archive
///   and moving-background work. `_create` (`:81-94`) returns `cPhs_ERROR_e` only from
///   `fopAcM_entrySolidHeap` (`:86`).
///
/// `daObjLpalmIsDelete` (`:189-192`) is the bare "accepts a delete request" form of RULE 6b.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daObjLpalm_c` holds a moving background,
/// a model and six quaternions/angle pairs (`d_a_obj_lpalm.h:26-35`) and registers no `dCcD`
/// collider of any kind. The profile `Group` is `fopAc_ACTOR_e` (`:216`). Neither
/// `combat::HitPoints` nor `enemy::Hurt` is attached.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Lpalm;

impl Lpalm {
    /// `daObjLpalm_c::_create` (`:81-94`), minus `fopAcM_ct_Retail` (`:82`), `dComIfG_resLoad`
    /// (`:83`), `fopAcM_ct_Demo` (`:85`), `fopAcM_entrySolidHeap` / `CheckCreateHeap` /
    /// `CreateHeap` (`:86`, `:17-19`, `:42-59`) and `CreateInit` (`:89`) - archive, heap, moving
    /// background and renderer work. The profile's `Parameters` field is `0`
    /// (`d_a_obj_lpalm.cpp:213`), so `daObjLpalm_c` carries no parameter to decode.
    pub fn spawn(_parameters: u32) -> Self {
        Lpalm
    }

    /// `fopAcM_setCullSizeBox(this, -350.0f, -50.0f, -350.0f, 350.0f, 1300.0f, 350.0f)`
    /// (`d_a_obj_lpalm.cpp:72`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_lpalm(name: &str) -> bool {
    LPALM_ACTORS.contains(&name)
}