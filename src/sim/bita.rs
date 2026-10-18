use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::camera::Camera;
use super::gohma::Gohma;
use super::health::Health;
use super::registry::{ActorRegistry, STATUS_UNK4000};

/// The stage string is `Bita` (`d_stage.cpp:858`, `fpcNm_BITA_e`). It matches the archive name
/// `"Bita"` that `daBita_Create` loads (`d_a_bita.cpp:243`) and releases at `:158`.
pub const BITA_ACTORS: [&str; 1] = ["Bita"];

/// `i_this->mPrmScale = (fopAcM_GetParam(i_this) >> 8) & 0xFF` (`d_a_bita.cpp:248`) into the
/// `u8 mPrmScale` field (`d_a_bita.h:24`).
const PRM_SCALE_SHIFT: u32 = 8; // <- d_a_bita.cpp:248
const PRM_SCALE_MASK: u32 = 0xFF; // <- d_a_bita.cpp:248

/// The `switch (i_this->mPrmScale)` of `daBita_Create` (`d_a_bita.cpp:258-263`):
/// `case 1: i_this->scale.setall(1.5f); break;`, `case 2: ... 2.0f`, `case 3: ... 3.0f` and
/// `default: i_this->scale.y = i_this->scale.z = i_this->scale.x = 1.0f;`. Indexed by `mPrmScale`,
/// which is a `u8` (`d_a_bita.h:24`), so the read is guarded and out-of-range lands on `default`.
const SCALE_TABLE: [f32; 4] = [1.0, 1.5, 2.0, 3.0]; // <- d_a_bita.cpp:259-262

/// `fopAcM_SetMin(i_this, i_this->scale.x * -1000.0f, -500.0f, i_this->scale.z * -1000.0f)`
/// (`d_a_bita.cpp:266`) and `fopAcM_SetMax(i_this, i_this->scale.x * 1000.0f, 500.0f,
/// i_this->scale.z * 1000.0f)` (`:267`). These are the SCALED box, and although the numbers
/// coincide with the shared table's `fopAc_CULLBOX_8_e` (`f_op_actor_mng.cpp:566`) when
/// `scale == 1.0`, the profile says `fopAc_CULLBOX_CUSTOM_e` (`d_a_bita.cpp:302`) and
/// `setCullSizeBox` is never called - `fopAcM_SetMin` / `fopAcM_SetMax`
/// (`f_op_actor_mng.h:505-511`) write `cull.box.min` / `cull.box.max` directly and
/// `fopAcM_setCullSizeBox` (`f_op_actor_mng.cpp:431-438`) adds nothing else.
const CULL_Y_MIN: f32 = -500.0; // <- d_a_bita.cpp:266
const CULL_Y_MAX: f32 = 500.0; // <- d_a_bita.cpp:267
const CULL_XZ: f32 = 1000.0; // <- d_a_bita.cpp:266-267

/// `if (btd->m6E16 >= 100)` (`d_a_bita.cpp:51`) - `btd` is the `btd_class` Gohma found by
/// `fpcM_Search`, whose `m6E16` (`d_a_btd.h:116`) is its battle sub-state: it counts 1, 2, 3 ...
/// through the fight (`d_a_btd.cpp:1940-1945`), is set to `100` on the death transition
/// (`d_a_btd.cpp:915`) and is back to `0` once the clear sequence reaches case `151`
/// (`d_a_btd.cpp:2208`). So this window is exactly the boss-death cutscene.
const BTD_OVER: i16 = 100; // <- d_a_bita.cpp:51

/// `cXyz delta = dComIfGp_getCamera(0)->view.mLookat.mEye - i_this->current.pos;` then
/// `if (delta.abs() < 1500.0f)` (`d_a_bita.cpp:52-53`). `cXyz::abs()` is the magnitude,
/// `sqrtf(abs2())` (`c_xyz.h:124`).
const BOARD_CULL_RANGE: f32 = 1500.0; // <- d_a_bita.cpp:53

/// The wooden platforms of the Gohma fight - `bita_class` (`d_a_bita.h:12-35`), the actor behind
/// the stage string `Bita`.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daBita_Create`'s `body_cyl_src`
/// (`d_a_bita.cpp:210-238`) sets `SrcObjAt Type = 0` (`:213`), `SrcObjAt Atp = 0` (`:214`) and
/// `SrcObjAt SPrm = 0` (`:215`) - the board owns no attack collider and cannot hurt anything -
/// while its `SrcObjTg Type = AT_TYPE_FIRE` (`:217`) with
/// `SrcObjTg SPrm = cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsOther_e` (`:218`) is a pure trigger that
/// only SENSES fire. The profile `Group` is `fopAc_ENV_e` (`:301`) and the class derives from a
/// bare `fopAc_ac_c`, not a `fopEn_enemy_c`, so there is no HP field anywhere in it
/// (`d_a_bita.h:12-35`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached: attaching them
/// would let `combat::control` delete a board retail cannot damage. The only thing the board puts
/// into the world is `mpBgW`, a `dBgW` moving background (RULE 3b).
///
/// The struct keeps exactly one field. `mMode` never leaves `MODE_NORMAL`: the only writer is
/// `mode_normal`'s `i_this->mMode = MODE_DEAD` at `d_a_bita.cpp:58`, which is guarded by
/// `i_this->mCyl.ChkTgHit()` (`:57`) - a `cCcD` collider-registry query, which RULE 3b lists as
/// absent, and for which nothing in this sim emits the `AT_TYPE_FIRE` object the guard then
/// compares against (`GetAtType() == AT_TYPE_FIRE` is an equality, not a mask, so the crate's
/// `AT_TYPE_FIRE_ARROW` does not satisfy it). `mMode` itself takes the `sBs_ClearArea` zero-fill
/// (`f_pc_base.cpp:98`), which is `MODE_NORMAL = 0x0` (`d_a_bita.h:15`), so `mode_dead`
/// (`:66-96`), and with it `mSub` (`:69`, `:71`) and `mTimer` (`:69`, `:74`), have no reachable
/// entry point and are dropped under RULE 12b - as is `field_0x2a7` (`:70`), which the decomp
/// itself writes and never reads.
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
        // `i_this->mType = (fopAcM_GetParam(i_this) >> 0) & 0xFF;` followed by
        // `if (i_this->mType == 0xFF) i_this->mType = 0;` (:247-250) - the `0xFF` is the "unset"
        // byte a `Bita` OBJNAME row carries in its `sub` column (`d_stage.cpp:858`). It is
        // transcribed here and deliberately NOT decoded into a field: its only readers are
        // `useHeapInit`'s `s32 type = i_this->mType; if (type > 1) type = 1;` (:172-174), which
        // picks the BMD / DZB / BRK archive entries out of `ita_bmd` / `ita_dzb` / `ita_Ef` /
        // `ita_Ef_bmd` (:20-23) - all archive work (RULE 1) - and `mode_dead`'s
        // `s32 type = i_this->mType; if (type > 1) type = 1;` (:81-83), which only chooses between
        // the two `dComIfGp_particle_set` ids `ID_AK_SN_BTDBURNEDBOARD00` / `...01` (:86, :88) and
        // the particle system is absent (RULE 3b). An unread field is dead code (RULE 7 /
        // RULE 12b). The parameter is the whole u32 (`fopAcM_SetParam(actor, prm->base.parameters)`,
        // `f_op_actor.cpp:271`), so the shifts and masks were copied exactly before being dropped -
        // the same call `esa.rs:190-195` makes.
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

    /// `mode_normal` (`d_a_bita.cpp:49-63`), the `btd->m6E16 >= 100` arm:
    ///
    /// ```c
    /// if (btd != NULL) {
    ///     if (btd->m6E16 >= 100) {
    ///         cXyz delta = dComIfGp_getCamera(0)->view.mLookat.mEye - i_this->current.pos;
    ///         if (delta.abs() < 1500.0f) {
    ///             fopAcM_delete(i_this);
    ///         }
    ///     } else { ... }
    /// }
    /// ```
    ///
    /// The `else` arm - `i_this->mCyl.ChkTgHit()` (`:57`) - is the omitted half described on the
    /// struct, so only the two outer tests are transcribed. `eye` is
    /// `dComIfGp_getCamera(0)->view.mLookat.mEye` (`:52`) and `pos` is `i_this->current.pos`.
    fn retiring(&self, boss: &Gohma, eye: Vec3, pos: Vec3) -> bool {
        if boss.m6e16 < BTD_OVER {
            return false;
        }
        // `delta.abs()` is `sqrtf(abs2())` (`c_xyz.h:124`), i.e. the magnitude, not the square.
        (eye - pos).length() < BOARD_CULL_RANGE
    }
}

pub fn is_bita(name: &str) -> bool {
    BITA_ACTORS.contains(&name)
}

/// `daBita_Execute` (`d_a_bita.cpp:114-148`) minus the renderer and moving-background tail -
/// `base_mtx_set` (`:130`, `:99-111`), `cMtx_copy(*calc_mtx, i_this->mMtx)` (`:131`),
/// `mpBgW->Move()` (`:132`) and the collider registration (`:134-145`).
///
/// * `if (btd == NULL) btd = (btd_class*)fpcM_Search(b_a_sub, i_this);` (`:115-116`) with
///   `b_a_sub`'s `fopAcM_IsActor(search) && fopAcM_GetName(search) == fpcNm_BTD_e` (`:28`).
///   `super::gohma::Gohma` is the crate's only `Btd` actor (`gohma.rs:9`,
///   `d_stage.cpp:843`), and there is at most one per stage, so a non-empty query is retail's
///   search hit. Retail caches the result in the file-scope `static btd_class* btd` (`:18`) and
///   only re-searches while it is null; since the Gohma is created with its room and never
///   destroyed, re-reading the query each tick finds the same actor every time.
/// * `if (dComIfGs_getLife() == 0) fopAcM_OffStatus(i_this, fopAcStts_UNK4000_e);` (`:118-119`).
///   `dComIfGs_getLife()` is `save.getPlayer().getPlayerStatusA().getLife()`
///   (`d_com_inf_game.h:2074-2076`) in quarter-heart units, which is exactly what
///   `super::health::Health::units` holds (`health.rs:9`). Nothing in the shipped decomp ever
///   *tests* `fopAcStts_UNK4000_e`, so the write is faithful but unobservable; it is kept because
///   `ActorRegistry` models `actor_status` (`registry.rs:56`, `off_status` at `:447`).
/// * The `switch (i_this->mMode)` dispatch (`:121-128`) has one reachable arm.
/// * `if (btd != NULL && i_this->mMode == bita_class::MODE_NORMAL)` (`:134`) then
///   `cMtx_YrotS` / `cMtx_XrotM` / `cXyz offs(0.0f, -80.0f, 160.0f)` / `MtxPosition` /
///   `VECAdd` / `mCyl.SetC` / `SetH(60.0f)` / `SetR(scale.x * 400.0f)` /
///   `dComIfG_Ccsp()->Set(&i_this->mCyl)` (`:135-144`): the cylinder IS registered, not inert -
///   but `ChkTgHit`, its only reader, is the absent `cCcD` registry (RULE 3b), so the geometry
///   has no consumer here and is not stored.
pub fn control(
    camera: Res<Camera>,
    health: Res<Health>,
    mut registry: ResMut<ActorRegistry>,
    bosses: Query<&Gohma>,
    boards: Query<(Entity, &Actor, &Bita)>,
    mut commands: Commands,
) {
    let boss = bosses.iter().next();
    // `dComIfGs_getLife() == 0` (`:118`)
    let spent = health.units == 0;
    let mut gone: Vec<Entity> = Vec::new();
    for (entity, actor, board) in &boards {
        // `fopAcM_OffStatus(i_this, fopAcStts_UNK4000_e)` (`:119`)
        if spent {
            if let Some(proc_id) = registry.proc_id(entity) {
                registry.off_status(proc_id, STATUS_UNK4000);
            }
        }
        // `if (btd != NULL)` (`:50`)
        let (Some(boss), Some(view)) = (boss, camera.view()) else {
            continue;
        };
        // `mode_normal` (`:51-55`)
        if board.retiring(boss, view.lookat.eye, actor.pos) {
            gone.push(entity);
        }
    }
    for entity in gone {
        let _ = commands.entity(entity).despawn();
    }
}
