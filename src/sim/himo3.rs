//! The climbable rope - `fpcNm_HIMO3_e`, `d_a_himo3.cpp`, the actor behind the stage string
//! `RopeR`.
//!
//! WHY THIS FILE HAS NO `control` SYSTEM. Every state `daHimo3_Execute` (`:435-578`) and
//! `himo3_control` (`:51-279`) maintain is either unreachable or unobservable here, and each
//! omission is a different confirmed-absent input rather than a choice:
//!
//! * `m029B`, the flag that arms `path_move` (`:443-445`), is written only at `:801` inside
//!   `if (i_this->m029A != 0xff)`, whose body returns `cPhs_ERROR_e` when `dPath_GetRoomPath`
//!   answers NULL (`:796-803`). Room `PATH` blocks are RULE 3b-absent, so no rope that survives
//!   creation ever has `m029B != 0` and `path_move` is dead.
//! * `m02BE`, the grab flag, is written 1 at `:141` and 2 at `:149`, both inside
//!   `else if (fopAcM_GetParam(actor) == 1)` / `== 2` - a WHOLE-`u32` compare, not a field. A
//!   parameter of exactly 1 or exactly 2 has `m029A = (param >> 24) & 0xFF == 0`, so `:796` takes
//!   the path arm and Create refuses. `m02BE` is therefore permanently 0, which makes the whole
//!   first half of `himo3_control` (`:67-129`) dead - including the `m15C0` rest-shape fill
//!   (`:93-100`), the `f28` length correction (`:87-88`, `:226`) and `setActorHang` (`:369-372`).
//! * What is left writes only `m02C0[]`, the node array. Its readers are the `dComIfG_Ccsp()->Set`
//!   collider registrations (`:243`, `:247`, `:477`) and `mLineMat.update` / `setBaseTRMtx` in the
//!   renderer (`:264-275`, `:309`). The `cCcD` actor-collision registry is RULE 3b-absent and the
//!   line matrix is a `J3D` draw, so neither has a modelled reader and the array is dropped under
//!   RULE 12b.
//! * `dKyw_get_wind_vec` / `dKyw_get_wind_pow` (`:161-168`, feeding `m15E8` and the sway offset
//!   `sp64` at `:186-191`) are config symbols with no value in this checkout - UNKNOWN, not proven
//!   zero - so folding them is invented data.
//! * `mAcch.CrrPos` (`:460`) operates on `m1624`, the head joint's position read back out of
//!   `mpModel->getBaseTRMtx()` (`:462-475`) - joint matrices are RULE 3b-absent.
//! * `ga_move` (`:342-366`) needs `cM_rndF` and the `AT_TYPE_WIND` Tg hit (`:452-458`); the wind
//!   collider semantics are RULE 3b-absent and there is no global `cM_rndF`.
//!
//! So the only retail behaviour this sim can still express is `daHimo3_Create`'s two
//! `cPhs_ERROR_e` returns, and both are parameter-only.
//!
//! (a) inbound damage: NONE, despite a Tg collider existing. `sph2_src` (`:724-751`) sets
//! `SrcObjTg Type` to `AT_TYPE_ALL & ~AT_TYPE_BOOMERANG & ~AT_TYPE_WATER & ~AT_TYPE_UNK20000 &
//! ~AT_TYPE_UNK400000 & ~AT_TYPE_LIGHT` with `cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsEnemy_e` (`:731-732`),
//! so the rope IS a valid target of an attack collider, and `cCcD_TgSPrm_Shield_e` (`:743`) makes
//! it bounce. But `daHimo3_Execute` never reads a damage result: its only `ChkTgHit` (`:452`) asks
//! whether the hit actor carries `AT_TYPE_WIND` and then writes the wind heading, and the actor
//! carries no `health`, no `mDamageSphere` and no damage reaction of any kind. Hitting a rope in
/// retail does nothing to it. (b) outbound damage: NONE - `cc_cyl_src` (`:667-695`) and `sph_src`
//! (`:696-723`) both set `SrcObjAt Type` to 0, so no attack collider is ever registered.
//! `daHimo3_IsDelete` (`:581-588`) is the bare "accepts a delete request" form of RULE 6b. Neither
//! `combat::HitPoints` nor `enemy::Hurt` is attached: attaching them would let `combat::control`
//! despawn a rope retail cannot damage.

use bevy_ecs::prelude::*;

/// `OBJNAME("RopeR", fpcNm_HIMO3_e, 255, 0)` (`d_stage.cpp:783`) is the only `fpcNm_HIMO3_e` row.
/// The object archive is `"Himo3"` (`dComIfG_resLoad(&i_this->mPhase, "Himo3")`, `d_a_himo3.cpp:757`),
/// which differs from the stage string.
pub const HIMO3_ACTORS: [&str; 1] = ["RopeR"];

/// `fopAc_CULLBOX_0_e` (`:867`) resolves through `l_cullSizeBox[0]` (`f_op_actor_mng.cpp:558`);
/// this file calls neither `fopAcM_setCullSizeBox` nor `fopAcM_SetMin` / `SetMax`.
const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)

/// `i_this->m0298 = (fopAcM_GetParam(a_this) >> 0) & 0xFF;` (`:759`) - the rope kind, compared
/// against `0xf` at `:779` and `0xff` at `:763`.
const KIND_SHIFT: u32 = 0; // <- d_a_himo3.cpp:759
const KIND_MASK: u32 = 0xFF; // <- d_a_himo3.cpp:759
/// `i_this->m0299 = (fopAcM_GetParam(a_this) >> 16) & 0xFF;` (`:760`) is the room-path follow
/// speed. It is read only inside `path_move` (`:408-413`), which the `m029A != 0xff` refusal makes
/// unreachable, so it is neither stored nor decoded.
/// `i_this->m029A = (fopAcM_GetParam(a_this) >> 24) & 0xFF;` (`:761`) - the room path index.
const PATH_SHIFT: u32 = 24; // <- d_a_himo3.cpp:761
const PATH_MASK: u32 = 0xFF; // <- d_a_himo3.cpp:761
/// `i_this->m15C0 = (fopAcM_GetParam(a_this) >> 8) & 0xFF;` (`:778`) - the segment count.
const SEGMENT_SHIFT: u32 = 8; // <- d_a_himo3.cpp:778
const SEGMENT_MASK: u32 = 0xFF; // <- d_a_himo3.cpp:778
/// `if (i_this->m0298 == 0xff) { i_this->m0298 = 0; }` (`:763-765`) - the one kind alias.
const KIND_ALIAS: u32 = 0xFF; // <- d_a_himo3.cpp:763
/// `if (i_this->m0298 == 0xf) { ... i_this->m15C0 = 200; i_this->m029A = 0xff; }` (`:779-782`) -
/// the uncut rope, whose segment count and path index are overwritten AFTER the parameter decode
/// and BEFORE the two refusal tests.
const KIND_UNCUT: u32 = 0xF; // <- d_a_himo3.cpp:779
const SEGMENT_UNCUT: u32 = 200; // <- d_a_himo3.cpp:781
const PATH_UNCUT: u32 = 0xFF; // <- d_a_himo3.cpp:782
/// `if (i_this->m15C0 > 0xc8) { return cPhs_ERROR_e; }` (`:792-794`).
const SEGMENT_MAX: u32 = 0xC8; // <- d_a_himo3.cpp:792
/// `if (i_this->m029A != 0xff) { ... }` (`:796`) - the only index that skips the room-path lookup.
const PATH_NONE: u32 = 0xFF; // <- d_a_himo3.cpp:796

/// The climbable rope.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Himo3 {
    /// `daHimo3_Create`'s `cPhs_ERROR_e`. Retail cancels the process during Create
    /// (`f_pc_create_req.cpp:106-108`), so the actor never executes even once; RULE 6b forbids the
    /// `Option<Self> -> None` wiring for that case because it would leave a bare `Actor` retail
    /// never keeps, so the refusal rides on the component and is discharged on the first tick.
    pub refused: bool,
}

impl Himo3 {
    /// `daHimo3_Create` (`:666-843`), minus `fopAcM_ct` (`:755`), `dComIfG_resLoad` (`:757`),
    /// `fopAcM_createHeap` (`:767-769`), `useHeapInit` (`:771-776`) and the `hio_set` / HIO child
    /// (`:832-836`) - archive, heap and HIO work. The ten warm-up `daHimo3_Execute` calls
    /// (`:838-840`) run against machinery that is all omitted, so they leave nothing behind.
    ///
    /// The decode order matters and is copied exactly: kind and path (`:759-761`), the `0xff` kind
    /// alias (`:763-765`), the segment count (`:778`), then the uncut-rope override (`:779-785`),
    /// and only then the two refusals (`:792-803`). `m0299`, decoded at `:760`, is not read.
    pub fn spawn(parameters: u32) -> Self {
        let mut kind = (parameters >> KIND_SHIFT) & KIND_MASK;
        if kind == KIND_ALIAS {
            kind = 0;
        }
        let mut segments = (parameters >> SEGMENT_SHIFT) & SEGMENT_MASK;
        let mut path = (parameters >> PATH_SHIFT) & PATH_MASK;
        if kind == KIND_UNCUT {
            segments = SEGMENT_UNCUT;
            path = PATH_UNCUT;
        }
        // `if (i_this->m15C0 > 0xc8) return cPhs_ERROR_e;` (`:792-794`)
        let too_long = segments > SEGMENT_MAX;
        // `if (i_this->ppd == NULL) return cPhs_ERROR_e;` (`:797-800`) - `dPath_GetRoomPath` on a
        // checkout with no room `PATH` blocks (RULE 3b) can only answer NULL, so any index other
        // than `0xff` refuses creation.
        let no_path = path != PATH_NONE;
        Self {
            refused: too_long || no_path,
        }
    }

    /// `fopAc_CULLBOX_0_e` (`:867`) -> `l_cullSizeBox[0]` (`f_op_actor_mng.cpp:558`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_himo3(name: &str) -> bool {
    HIMO3_ACTORS.contains(&name)
}

pub fn control(mut ropes: Query<(Entity, &Himo3)>, mut commands: Commands) {
    let mut spent: Vec<Entity> = Vec::new();
    for (entity, rope) in &mut ropes {
        if rope.refused {
            spent.push(entity);
        }
    }
    for entity in spent {
        let _ = commands.entity(entity).despawn();
    }
}