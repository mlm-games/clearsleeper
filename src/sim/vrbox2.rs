use bevy_ecs::prelude::*;

/// `OBJNAME("Vrbox2", fpcNm_VRBOX2_e, 255, 0)` (`d_stage.cpp:1261`) - the OBJNAME table is the
/// authority. `src/sim/vrbox.rs` already owns `fpcNm_VRBOX_e` as `VRBOX_ACTORS = ["Vrbox"]`
/// (`d_stage.cpp`'s `Vrbox` row), a DIFFERENT process from a different translation unit, so the two
/// strings do not collide.
pub const VRBOX2_ACTORS: [&str; 1] = ["Vrbox2"];

/// The sky-box cloud / haze / distant-sea planes - `vrbox2_class`
/// (`d_a_vrbox2.cpp:261-281`, `d_a_vrbox2.h`), the actor behind the stage string `Vrbox2`.
///
/// Retail runs NO simulation for this actor at all, so there is nothing here to transcribe:
///
///   * `daVrbox2_Execute(vrbox2_class*)` (`:247-249`) is the body `return TRUE` - the whole
///     function is eight bytes. There is no state, no timer, no parameter and no branch.
///   * `daVrbox2_Draw` (`:28-85`) positions its three models from
///     `dComIfGd_getView()->mInvViewMtx`'s translation column (`:60-64`), i.e. from the CAMERA, and
///     never from `current.pos`. `current.pos` is read exactly twice, at `:50` and `:55`, and only
///     as `fili->mSeaLevel` arithmetic on the room's own sea level.
///   * `daVrbox2_color_set` (`:90-244`) is J3D tev / texture-matrix / GXColor work on
///     `g_env_light`'s `mVrKasumiMaeColor`, `mVrSkyColor`, `mVrkumoColor` and `mVrUsoUmiColor`, none
///     of which this sim carries. Its one non-renderer input is
///     `dKyw_get_wind_vec()` / `dKyw_get_wind_pow()` (`:114-115`), which are config symbols with no
///     value in this checkout - unknown, NOT proven zero, so folding them to a literal would be
///     invented data. RULE 3b lists them among the confirmed-absent inputs.
///   * `daVrbox2_solidHeapCB` (`:262-281`) builds three models out of the STAGE archive
///     (`dComIfG_getStageRes("Stage", "vr_back_cloud.bdl")`, `:265`, `:269`, `:272`) and reports
///     failure only if any of the three is NULL (`:277-279`). `daVrbox2_Create` (`:284-298`) turns
///     that into `cPhs_ERROR_e` (`:294`), which cancels the actor inside Create
///     (`f_pc_create_req.cpp:106-108`) so it never executes even once. That is an archive-availability
///     failure, not a behaviour, and this sim's asset pipeline has no equivalent signal; the
///     `branch.rs` reduction - spawn it, mark nothing, carry no `control` state - is used instead.
///
/// `daVrbox2_IsDelete` (`:252-254`) is the bare "accepts a delete request" form of RULE 6b, never a
/// self-destruct (RULE 6b).
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. The class holds three `J3DModel*`
/// (`d_a_vrbox2.cpp:29-31`, `:267-274`) and registers no `dCcD` collider of any kind; the profile
/// `Group` is `fopAc_ACTOR_e` (`:321`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
///
/// ## No `cull()`
/// The profile's `fopAc_CULLBOX_0_e` (`:322`) does resolve, to
/// `l_cullSizeBox[0] = fopAc_cullSizeBox(cXyz(-40.0f, 0.0f, -40.0f), cXyz(40.0f, 125.0f, 40.0f))`
/// (`f_op_actor_mng.cpp:558`). But the profile's `Status` is
/// `fopAcStts_UNK4000_e | fopAcStts_UNK40000_e` (`:320`), which does NOT include `fopAcStts_CULL_e`,
/// so retail never culls this actor, and `Draw` positions its geometry from the camera matrix rather
/// than from `current.pos` - the box would be describing nothing that is drawn. Writing a
/// `cull()` here would attach a real number to a real but unused profile field; declining keeps the
/// omission visible. This is the same call `iceisland` and `tornado` made.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vrbox2;

impl Vrbox2 {
    /// `daVrbox2_Create` (`:284-298`). Retail's only `cPhs_ERROR_e` arm (`:294`) needs one of three
    /// stage-archive models to be missing, which is an asset condition and not a per-stage
    /// parameter - the profile's `Parameters` field is `0` (`:316`), so there is nothing to decode
    /// and no `Option<Self> -> None` refusal to model.
    pub fn spawn(_parameters: u32) -> Self {
        Self
    }
}

pub fn is_vrbox2(name: &str) -> bool {
    VRBOX2_ACTORS.contains(&name)
}

/// `daVrbox2_Execute` (`:247-249`) is `return TRUE`, so there is nothing to step. The empty system
/// exists so `Vrbox2` is claimed by exactly one module and cannot be silently taken by a later
/// branch - the `branch.rs` shape.
pub fn control(_boxes: Query<&Vrbox2>) {}
