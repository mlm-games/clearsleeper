//! Hyrule Castle knight statues, Master Sword chamber - `fpcNm_Obj_Zouk_e`,
//! `d_a_obj_zouK.cpp`, stage strings `zouK` .. `zouK4` (`d_stage.cpp:1150-1154`, all five rows
//! bound to the one profile name `fpcNm_Obj_Zouk_e`).
//!
//! (a) inbound damage: NONE. (b) outbound damage: NONE.
//!
//! `M_cyl_src` sets `SrcObjAt Type` to `0` (`d_a_obj_zouK.cpp:32`), so this actor presents NO
//! attack type: nothing in retail can register a hit against it and it cannot be damaged.
//! `SrcObjTg Type` is `~(AT_TYPE_LIGHT | AT_TYPE_UNK400000 | AT_TYPE_WIND | AT_TYPE_UNK20000 |
//! AT_TYPE_WATER)` (`:35`), so a target type IS presented - but `mCyl0`, `mCyl1` and `mCyl2` are
//! only `Set` / `SetStts` / `SetTgVec` / `OnTgNoHitMark` (`:157-172`). `dComIfG_Ccsp()->Set()`
//! is never called for any of them anywhere in the file, and `ChkTgHit` / `ChkAtHit` are never
//! called either. The three cylinders are inert: they are never in the collision registry, so no
//! target-type arm can fire in retail either. Neither `combat::HitPoints` nor `enemy::Hurt` is
//! attached.
//!
//! What is left in this translation unit after the omissions below is a cull box and nothing
//! else, which is why `Zouk` is a unit struct and `control` is a no-op in the same shape as
//! `rectangle.rs` and `seatag.rs`.
//!
//! Two further literals in this file are real but unreachable and so are carried only as
//! citations: `fopAcM_rollPlayerCrash(this, 288.0f, 0x0D)` (`:255`), whose result is a private
//! player flag that has no channel in this sim, and `M_cyl_src`'s `cM3dGCylS` radius `100.0f`
//! and height `200.0f` (`:53-54`) on those three never-registered cylinders.
//!
//! `Mthd_IsDelete { return TRUE; }` (`:373-375`) only ACCEPTS a delete request per RULE 6b; it
//! is not a self-destruct. `_create` returns `cPhs_ERROR_e` only when `fopAcM_entrySolidHeap`
//! fails (`:193-195`), which is a resource allocation failure rather than a refusal, so there is
//! no `refused` flag.

use bevy_ecs::prelude::*;

/// `d_stage.cpp:1150-1154` binds `zouK`, `zouK1`, `zouK2`, `zouK3` and `zouK4` to
/// `fpcNm_Obj_Zouk_e`, the profile of `d_a_obj_zouK.cpp:387-402`.
pub const ZOUK_ACTORS: [&str; 5] = ["zouK", "zouK1", "zouK2", "zouK3", "zouK4"];

/// `fopAcM_setCullSizeBox(this, -1000.0f, -0.0f, -1000.0f, 1000.0f, 2800.0f, 1000.0f)`
/// (`d_a_obj_zouK.cpp:149`). Retail profile `Cull Type` is `fopAc_CULLBOX_CUSTOM_e` (`:401`),
/// so this literal pair IS the box. The `#if VERSION == VERSION_DEMO` arm at `:147`
/// (`-550, -0, -600, 550, 1400, 600`) is excluded: retail is `VERSION_NTSC_U`.
pub const CULL_MIN: [f32; 3] = [-1000.0, -0.0, -1000.0];
pub const CULL_MAX: [f32; 3] = [1000.0, 2800.0, 1000.0];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Zouk;

impl Zouk {
    /// The profile's `Parameters` field is `0` (`d_a_obj_zouK.cpp:395`) and the actor's only
    /// parameter read, `param_get_arg0()` (`:177`, `:259`), feeds the `dBgW` registration block
    /// that is omitted - so per RULE 7 the parameter is not stored and `spawn` takes none.
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_zouk(name: &str) -> bool {
    ZOUK_ACTORS.contains(&name)
}

pub fn control(_statues: Query<&Zouk>) {}
