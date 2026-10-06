use bevy_ecs::prelude::*;

/// The stage string is `Bita` (`d_stage.cpp:858`, `fpcNm_BITA_e`). It matches the archive name
/// `"Bita"` that `daBita_Create` loads (`d_a_bita.cpp:243`) and releases at `:158`.
pub const BITA_ACTORS: [&str; 1] = ["Bita"];

/// `i_this->mPrmScale = (fopAcM_GetParam(i_this) >> 8) & 0xFF` (`d_a_bita.cpp:248`) into the
/// `u8 mPrmScale` field (`d_a_bita.h:21`).
const PRM_SCALE_SHIFT: u32 = 8; // <- d_a_bita.cpp:248
const PRM_SCALE_MASK: u32 = 0xFF; // <- d_a_bita.cpp:248

/// The `switch (i_this->mPrmScale)` of `daBita_Create` (`d_a_bita.cpp:258-263`):
/// `case 1: i_this->scale.setall(1.5f); break;`, `case 2: ... 2.0f`, `case 3: ... 3.0f` and
/// `default: i_this->scale.y = i_this->scale.z = i_this->scale.x = 1.0f;`. Indexed by `mPrmScale`,
/// which is a `u8` (`d_a_bita.h:21`), so the read is guarded and out-of-range lands on `default`.
const SCALE_TABLE: [f32; 4] = [1.0, 1.5, 2.0, 3.0]; // <- d_a_bita.cpp:259-262

/// `fopAcM_SetMin(i_this, i_this->scale.x * -1000.0f, -500.0f, i_this->scale.z * -1000.0f)`
/// (`d_a_bita.cpp:266`) and `fopAcM_SetMax(i_this, i_this->scale.x * 1000.0f, 500.0f,
/// i_this->scale.z * 1000.0f)` (`:267`). These are the SCALED box, and although the numbers
/// coincide with the shared table's `fopAc_CULLBOX_8_e` (`f_op_actor_mng.cpp:565`) when
/// `scale == 1.0`, the profile says `fopAc_CULLBOX_CUSTOM_e` (`:302`) and `setCullSizeBox` is
/// never called - `SetMin` / `SetMax` (`f_op_actor_mng.h:505-511`) write `cull.box` directly.
const CULL_Y_MIN: f32 = -500.0; // <- d_a_bita.cpp:266
const CULL_Y_MAX: f32 = 500.0; // <- d_a_bita.cpp:267
const CULL_XZ: f32 = 1000.0; // <- d_a_bita.cpp:266-267

/// The wooden platforms of the Gohma fight - `bita_class` (`d_a_bita.h:11-35`), the actor behind
/// the stage string `Bita`.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daBita_Create`'s `body_cyl_src`
/// (`d_a_bita.cpp:210-238`) sets `SrcObjAt Type = 0` (`:213`), `SrcObjAt Atp = 0` (`:214`) and
/// `SrcObjAt SPrm = 0` (`:215`) - the board owns no attack collider and cannot hurt anything -
/// while its `SrcObjTg Type = AT_TYPE_FIRE` (`:217`) with
/// `SrcObjTg SPrm = cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsOther_e` (`:218`) is a pure trigger that
/// only SENSES fire. The profile `Group` is `fopAc_ENV_e` (`:301`) and the class derives from a
/// bare `fopAc_ac_c`, not a `fopEn_enemy_c`, so there is no HP field anywhere in it
/// (`d_a_bita.h:11-35`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached: attaching them
/// would let `combat::control` delete a board retail cannot damage. The only thing the board puts
/// into the world is `mpBgW`, a `dBgW` moving background (RULE 3b).
///
/// The struct keeps exactly one field. Both `mode_normal` (`d_a_bita.cpp:49-63`) arms are gated on
/// inputs this sim does not have, so `mMode`, `mSub`, `mTimer` and `field_0x2a7`
/// (`d_a_bita.h:22-25`) have no reachable reader and are dropped under RULE 12b; see OMITTED in the
/// report for the full burn sequence with line citations.
///
/// `daBita_IsDelete` (`:151-154`) is the bare "accepts a delete request" form of RULE 6b, and
/// `daBita_Create` returns `cPhs_ERROR_e` only from `fopAcM_entrySolidHeap` (`:252-253`) and
/// `dComIfG_Bgsp()->Regist` (`:255-256`) - archive and moving-background work.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bita {
    /// The retail `fopAcM_prm_class::scale` after `daBita_Create`'s `switch (i_this->mPrmScale)`
    /// (`d_a_bita.cpp:258-263`) ran its `setall`. `Actor` carries no scale field, so this is where
    /// the board's scale lives; `fopAcM_SetMin` / `fopAcM_SetMax` (`:266-267`) and
    /// `mCyl.SetR(i_this->scale.x * 400.0f)` (`:143`) are the two retail consumers.
    pub scale: f32,
}

impl Bita {
    /// `daBita_Create` (`d_a_bita.cpp:209-278`) minus `fopAcM_ct_Retail` (`:240`),
    /// `dComIfG_resLoad` (`:243`), `fopAcM_ct_Demo` (`:245`), `btd = NULL` (`:246`),
    /// `fopAcM_entrySolidHeap` / `useHeapInit` (`:252`, `:169-206`),
    /// `dComIfG_Bgsp()->Regist` (`:255`), the `switch (i_this->mPrmScale)`'s consumers
    /// `fopAcM_SetMtx` (`:265`) / `base_mtx_set` (`:271`) / `mpBgW->Move()` (`:272`) /
    /// `mCyl.Set` (`:273`), and `shape_angle = current.angle` (`:270`), which is renderer work.
    /// What survives is the parameter decode and the scale table, because the cull box at
    /// `:266-267` is built from it.
    pub fn spawn(parameters: u32) -> Self {
        let prm_scale = ((parameters >> PRM_SCALE_SHIFT) & PRM_SCALE_MASK) as usize;
        Self {
            scale: SCALE_TABLE.get(prm_scale).copied().unwrap_or(1.0),
        }
    }

    /// `fopAcM_SetMin(i_this, scale.x * -1000.0f, -500.0f, scale.z * -1000.0f)`
    /// (`d_a_bita.cpp:266`) and `fopAcM_SetMax(i_this, scale.x * 1000.0f, 500.0f,
    /// scale.z * 1000.0f)` (`:267`), with `scale.setall` (`:259-262`) having made
    /// x == z == `self.scale`.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        let xz = self.scale * CULL_XZ;
        ([-xz, CULL_Y_MIN, -xz], [xz, CULL_Y_MAX, xz])
    }
}

pub fn is_bita(name: &str) -> bool {
    BITA_ACTORS.contains(&name)
}