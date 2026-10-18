use bevy_ecs::prelude::*;

use super::flags::Switches;

/// `OBJNAME("Mkdan1", fpcNm_Obj_Stair_e, 255, 0)` (`d_stage.cpp:842`) is `fpcNm_Obj_Stair_e` =
/// `d_a_obj_stair.cpp`. The archive is a DIFFERENT string -
/// `const char daObj_Stair_c::M_arcname[] = "Mkdan";` (`d_a_obj_stair.cpp:14`) - and so are the two
/// sub-archives `BDL_MKDAN1` (`:84`) and `DZB_MKDAN1` (`:160`), so the OBJNAME row above is the
/// authority, not the arcname.
///
/// This is a DIFFERENT translation unit from `d_a_lstair.cpp` (`fpcNm_GKAI00_e`, `Gkai00`), which
/// is a later batch's assignment; the two share no source, no profile and no stage string. The
/// module name `stair` collides with no `core` or `std` item - `core` has no `stair`, and the name
/// is always used qualified as `super::stair`.
pub const STAIR_ACTORS: [&str; 1] = ["Mkdan1"];

/// `u32 switchIdx = fopAcM_GetParam(this) & 0xFF;` (`d_a_obj_stair.cpp:148`) - the WHOLE u32 masked
/// to a byte (`f_op_actor_mng.h:127-129`).
const SWITCH_MASK: u32 = 0xFF; // <- d_a_obj_stair.cpp:148
/// `if (switchIdx != 0xff && fopAcM_isSwitch(this, switchIdx)) { return cPhs_STOP_e; }`
/// (`d_a_obj_stair.cpp:150-151`) - the refusal's "no switch" sentinel.
const NO_SWITCH: u32 = 0xFF; // <- d_a_obj_stair.cpp:150

/// `fopAc_CULLBOX_0_e` (`d_a_obj_stair.cpp:366`) resolves to `l_cullSizeBox[0]`
/// (`f_op_actor_mng.cpp:558`) - a real box, so `cull()` is derivable even though the file makes no
/// `fopAcM_setCullSizeBox` call. Note the profile `Group` here is `fopAc_ENV_e` (`:365`), not
/// `fopAc_ACTOR_e` as in most of its siblings.
const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)

/// The fragile flight of steps in the Cave of Ordeals - `daObj_Stair_c`
/// (`d_a_obj_stair.h:6-37`), the actor behind the stage string `Mkdan1`.
///
/// # WHY THERE IS NO BEHAVIOUR SYSTEM
///
/// `daobj_stairHIO_c`'s constructor seeds `mNo = -1, m08 = 2.5f, m0C = -70.0f, m10 = 5.0f, m14 =
/// 2.5f, m18 = 6` (`d_a_obj_stair.cpp:62-69`), but `CreateHeap` then calls
/// `l_HIO.mNo = mDoHIO_createChild("崩れる階段", (JORReflexible *)&l_HIO);` (`:87-89`), which
/// REGISTERS the struct with the host-IO table (`m_Do_hostIO.h:116-117` ->
/// `mDoHIO_subRoot_c::createChild`, `m_Do_hostIO.cpp:17-37`). Nothing in this checkout ever WRITES
/// through that registration - `mDoHIO_child_c::getPt()` has no reader outside
/// `m_Do_hostIO.cpp:19`, `:27` and `:42` - so `m08`, `m0C`, `m10`, `m14` and `m18` are stage HIO
/// data whose values are UNKNOWN here.
///
/// Those five fields drive EVERY live branch of `_execute` (`d_a_obj_stair.cpp:216-328`):
/// `field_0x2E0 == l_HIO.m18` (`:218`), `l_HIO.m10 * 0.5f` / `l_HIO.m14` (`:224`, `:234-246`),
/// `field_0x2E0 > l_HIO.m18` (`:253`), `field_0x2E0 > l_HIO.m18 + 0xf0` (`:257`),
/// `field_0x2E0 = l_HIO.m18 - (s16)(...)` (`:303`), `l_HIO.m18 - 2` (`:316`) and the trailing bob
/// `((temp5 - l_HIO.m18) + 10) & 0x3` (`:323-325`). Folding them to the constructor values would be
/// inventing data, which RULE 1 forbids, so this module carries ONLY the create-time refusal - the
/// one piece of `create` that needs no HIO. A module with a spawn guard and no `control`-driven
/// behaviour is a correct outcome here, as it was for `lpalm` and `lwood`.
///
/// # DAMAGE
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. Checked, not assumed, because a staircase
/// that collapses on a bomb is exactly the sort of thing one would expect to be damageable:
/// `cps_src` (`d_a_obj_stair.cpp:15-43`) has `SrcObjAt Type = 0` and `Atp = 0` (`:20-21`), so it
/// registers NO attack at all - only a `SrcObjTg Type = AT_TYPE_ALL` target with
/// `cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsPlayer_e` (`:22-23`), which is the trigger, not damage. The
/// class has no health field (`d_a_obj_stair.h:23-36`), `execute` has no `cc_at_check`, and the
/// profile `Group` is `fopAc_ENV_e` (`:365`). Neither `combat::HitPoints` nor `enemy::Hurt` is
/// attached.
///
/// `daObjStairIsDelete` (`:339-342`) is the bare "accepts a delete request" form of RULE 6b. The real
/// destruction is `fopAcM_delete(this)` at `:258`, which is inside an HIO-gated arm (see above).
/// `_create` returns `cPhs_STOP_e` at `:151` and `MoveBGCreate`'s state otherwise (`:160`);
/// `cPhs_STOP_e` cancels the actor inside Create exactly like `cPhs_ERROR_e`
/// (`f_pc_create_req.cpp:106-108`), so the refusal arm is a spawn-then-despawn on the FIRST TICK
/// and never `Option<Self> -> None` (RULE 6b).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stair {
    /// `_create`'s `return cPhs_STOP_e` arm (`:151`). `cPhs_STOP_e` cancels the actor inside
    /// Create, so the entity exists for exactly one tick and is despawned by [`control`] - the
    /// `mkiek.rs` shape.
    pub refused: bool,
}

impl Stair {
    /// `daObj_Stair_c::_create` (`:145-194`), minus `fopAcM_ct` (`:147`), `dComIfG_resLoad` (`:153`),
    /// `MoveBGCreate` (`:160`, `dBgW` work), the `JUT_ASSERT` (`:161`), the two quaternion seeds
    /// (`:156-168`, `set_mtx` work), `mpBgW->SetRideCallback(ride_call_back)` (`:171`, `dBgW` work
    /// whose callback reads `l_HIO.m18` at `:74` anyway), `mStts.Init` / `mCps.Set` /
    /// `mCps.SetStts` (`:172-174`, `dCcD` work), `mCps.SetStartEnd` / `SetR` (`:186-187`, the
    /// trigger capsule) and the two `field_0x2E8` / `field_0x2EC` zero stores (`:188-189`, which
    /// only feed the omitted arms).
    ///
    /// What survives is the refusal, which needs `Switches`.
    ///
    /// `fopAc_Init` seeds `current` and `old` from `home` (`f_op_actor.cpp:282-284`); RULE 2 gives
    /// the staircase scale exactly 1.0, and `setBaseScale(scale)` (`:104`) is renderer work.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        // `u32 switchIdx = fopAcM_GetParam(this) & 0xFF;` (:148)
        let switch_idx = parameters & SWITCH_MASK;
        // `if (switchIdx != 0xff && fopAcM_isSwitch(this, switchIdx)) { return cPhs_STOP_e; }`
        // (:150-151)
        Self {
            refused: switch_idx != NO_SWITCH && switches.test(switch_idx as u8),
        }
    }

    /// `fopAc_CULLBOX_0_e` (`d_a_obj_stair.cpp:366`) -> `l_cullSizeBox[0]`
    /// (`f_op_actor_mng.cpp:558`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_stair(name: &str) -> bool {
    STAIR_ACTORS.contains(&name)
}

pub fn control(stairs: Query<(Entity, &Stair)>, mut commands: Commands) {
    for (entity, stair) in &stairs {
        if stair.refused {
            let _ = commands.entity(entity).despawn();
        }
    }
}
