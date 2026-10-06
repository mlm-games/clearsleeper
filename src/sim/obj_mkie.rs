//! `daObjMkie::Act_c` (`d_a_obj_mkie.h:8-79`) - the Earth Temple elephant statue that dissolves
//! when light hits it. `Nonmatching` count: 0, so every token below is transcribed.
//!
//! Nothing in `src/sim/` claims `MkieBA` / `MkieBAB`. `src/sim/mkiek.rs`'s
//! `MKIEK_ACTORS = ["MkieK"]` (`:7`) is a different profile (`fpcNm_Obj_MkieK_e`); `"MkieBAB"`
//! must not be case- or suffix-folded into it.

use bevy_ecs::prelude::*;

use super::flags::Switches;

/// `OBJNAME("MkieBA", fpcNm_Obj_Mkie_e, 0, 61)` (`d_stage.cpp:949`) and
/// `OBJNAME("MkieBAB", fpcNm_Obj_Mkie_e, 1, 61)` (`:950`). `fpcNm_Obj_Mkie_e` is
/// `d_a_obj_mkie.cpp`; the archive `daObjMkie::Act_c::M_arcname[] = "MkieB"` (`:11`) is neither.
pub const OBJ_MKIE_ACTORS: [&str; 2] = ["MkieBA", "MkieBAB"];

/// `daObjMkie::Prm_e::PRM_TYPE_S = 0x08` with `PRM_TYPE_W = 0x01` (`d_a_obj_mkie.h:29-30`), so
/// `daObj::PrmAbstract<Prm_e>(this, 1, 8)` = `((1 << 1) - 1) & (param >> 8)` (`d_a_obj.h:14`) - a
/// single bit, selecting one of the two `M_attr` rows.
const TYPE_SHIFT: u32 = 0x08; // <- d_a_obj_mkie.h:30
const TYPE_MASK: u32 = 0x01; // <- d_a_obj_mkie.h:29 (PRM_TYPE_W = 0x01, d_a_obj.h:14)
/// `daObjMkie::Prm_e::PRM_SWSAVE_S = 0x10`, `PRM_SWSAVE_W = 0x08` (`:42-43`).
const SWSAVE_SHIFT: u32 = 0x10; // <- d_a_obj_mkie.h:43
const SWSAVE_MASK: u32 = 0xFF; // <- d_a_obj_mkie.h:42 (PRM_SWSAVE_W = 0x08, d_a_obj.h:14)

/// `daObjMkie::Act_c::M_attr[2].mScale` (`d_a_obj_mkie.cpp:44` and `:85`). `attr()` is
/// `M_attr[mType]` (`d_a_obj_mkie.h:23`), and `mScale` (`d_a_obj_mkie.h:19`) is the statue's uniform scale: the cull box
/// is `attr().mScale * <literal>` (`:176-181`) and the eye lift is `attr().mScale * 130.0f`
/// (`:197`, `:402`). Its two `name` fields, `"MkieB_die"` (`:43`) and `"MkieBAB_die"` (`:84`), are
/// the event-table keys of the omitted dissolve cut.
const STATUE_SCALE: [f32; 2] = [
    1.0, // <- d_a_obj_mkie.cpp:44
    3.0, // <- d_a_obj_mkie.cpp:85
];

/// The three literals of `fopAcM_setCullSizeBox` (`d_a_obj_mkie.cpp:176-181`), each multiplied by
/// `attr().mScale`: the XZ half-extent, the floor and the ceiling.
const CULL_XZ: f32 = 80.0; // <- d_a_obj_mkie.cpp:176, :179
const CULL_Y_MIN: f32 = -1.0; // <- d_a_obj_mkie.cpp:177
const CULL_Y_MAX: f32 = 230.0; // <- d_a_obj_mkie.cpp:180

/// `daObjMkie::Act_c` - the Earth Temple elephant statue.
///
/// The struct is the live subset of `Act_c` (`d_a_obj_mkie.h:89-112`), and it is small because the
/// statue's whole dissolve sequence is gated on inputs this crate does not have. What does survive
/// is genuine, stage-visible retail behaviour: the statue is NOT created at all once its switch
/// reads on.
///
/// * `Mthd_Create` (`.cpp:205-217`) opens `cPhs_State phase_state = cPhs_ERROR_e;` (`:206`) and
///   only overwrites it inside `if (mSwitch && (phase_state = dComIfG_resLoad(&mPhs, M_arcname),
///   phase_state == cPhs_COMPLEATE_e))` (`:209`). `mSwitch = !fopAcM_isSwitch(this,
///   prm_get_swSave())` (`:208`), so when the switch already reads on the short-circuit skips the
///   assignment and `phase_state` stays `cPhs_ERROR_e`. RULE 6b: that cancels the process inside
///   Create (`f_pc_create_req.cpp:106-108`), so the statue never executes even once - spawn it and
///   drop it on the first tick, never `Option<Self> -> None`. `is_switch` is
///   `fopAcM_isSwitch(this, sw)` = `dComIfGs_isSwitch(sw, fopAcM_GetHomeRoomNo(this))`
///   (`f_op_actor_mng.h:374-376`). RULE 11b: the REAL index goes through; a `swSave >= 0x80`
///   statue would read a per-zone `dSv_zoneBit_c` bit in retail and reads false here.
/// * `Mthd_Create`'s tail (`scale.setall(initialScale)` at `:212`, `MoveBGCreate` at `:213`) is a
///   `dBgW` moving background plus a model scale - renderer work, and `scale` has no reader in
///   `src/`, so it is not stored.
/// * `Create` (`.cpp:170-202`) computes the cull box from `attr().mScale` and the eye position from
///   `attr().mScale * 130.0f`; `fopAcM_SetMtx` (`:171`), `init_mtx` (`:172`), `init_cc` (`:184`),
///   the `PrmAbstract<Prm_e>(PRM_FLAG_W, PRM_FLAG_S) != 0` branch (`:186-191`,
///   `fopAcM_ClearStatusMap` + `gbaName = 0`) and `dComIfGp_evmng_getEventIdx(attr().name,
///   prm_get_evId())` (`:200`) are omitted. `attr().mScale` survives only as [`Self::scale_factor`],
///   read by [`Self::cull`]; `mType` itself (`.cpp:210`) has no reader left - its other consumer is
///   the `se_break[mType]` table inside the omitted dissolve - so it is NOT stored (RULE 7).
///
/// OMITTED, with the reason for each:
///
/// * `chk_light` (`.cpp:252-266`) - the statue's ONLY trigger. It asks
///   `dComIfGp_getDetect().chk_light(&eyePos)` (`:255`), a light source, and eight
///   `mTri[i].ChkTgHit()` tests (`:259`). RULE 3b lists "light source" as confirmed-absent and the
///   `cCcD` actor-collision registry as absent too, so neither half can be evaluated. Note
///   `M_tri_src` (`.cpp:12-40`) is a `dCcD_SrcTri` whose `SrcObjTg Type` is `AT_TYPE_LIGHT` (`:19`)
///   with `SrcObjTg SPrm = dCcG_TgSPrm_Shield_e | dCcG_TgSPrm_ShieldFrontRange_e` (`:20`) - the
///   statue is a Tg for light only and registers no AT, which is why (a)/(b) below are both NONE.
/// * `mode_wait` (`.cpp:299-318`) - `mLightCnt` is bumped only by a `chk_light()` hit (`:300-304`),
///   and its other arm is `set_cc_pos()` (`:310`) plus eight `dComIfG_Ccsp()->Set(&mTri[i])`
///   registrations (`:312-316`), all collider work. With `mLightCnt` provably 0, the
///   `if (mLightCnt > 20) mode_demoWait_init()` arm (`:306-307`) is unreachable.
/// * `mode_demoWait` (`.cpp:327-345`) and `mode_demo` (`.cpp:359-386`) - both need the stage
///   event-info table (`dComIfGp_evmng_existence(mEvtIdx)` at `:330`,
///   `fopAcM_orderOtherEventId` at `:335`, `dComIfGp_evmng_endCheck` at `:378`), and `mode_demo`'s
///   `mBaseAnmPlaying = mBrkVan.play()` (`:370`) is a BRK archive frame. `sound_break`'s
///   `se_break[mType]` table (`:286-290`, `JA_SE_OBJ_L_STATUE_BREAK` / `JA_SE_OBJ_L_STATUE_L_BREAK`),
///   `sound_melt`'s `JA_SE_OBJ_L_OBJ_BRK_TAME` (`:281`), `vib_break`'s
///   `dComIfGp_getVibration().StartShock(4, -0x21, ...)` (`:295`), `eff_break`'s particle (`:276`),
///   `mDoAud_seStart(JA_SE_READ_RIDDLE_1)` (`:352`), `fopAcM_onSwitch(this, prm_get_swSave())`
///   (`:365`) and `mStatueDelete = true` -> `fopAcM_delete(this)` (`:383-384`, `:407-409`) all sit
///   inside that omitted pair, so none of those ids is stored here.
/// * `Delete` (`.cpp:220-222`) is `return true;` and `Mthd_IsDelete` (`.cpp:464-466`) calls
///   `dBgS_MoveBgActor::IsDelete()`, which is `virtual BOOL IsDelete() { return TRUE; }`
///   (`d_bg_s_movebg_actor.h:35`) - the bare "accepts a delete request" form of RULE 6b, not a
///   despawn.
/// * `set_cc_pos` (`.cpp:157-167`), `init_cc` (`.cpp:146-154`), `M_tri_src`'s 8x3 vertex table per
///   attr row (`.cpp:45-82` and `:86-121`), `Mthd_Delete`'s `MoveBGDelete()` (`:228`), `set_mtx` /
///   `init_mtx` (`:235-249`) and `Draw` (`:417-439`) - collider, `dBgW`, matrix and renderer work.
/// * `Execute`'s `attention_info.position` write (`.cpp:401-403`) and `eyePos = attention_info.position`
///   (`:405`) survive transcription but have no reader once `chk_light` is omitted, so no field is
///   stored for them (RULE 7: an unread field is dead code).
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `M_tri_src` declares an AT *type* of 0
/// (`d_a_obj_mkie.cpp:17`) - the statue is a target, never an attacker - and `fopAcStts_SHOWMAP_e`
/// in the profile `Status` (`:490`) only adds the shop map row. `prm_get_correct()` /
/// `prm_get_flag()` exist but only feed the omitted dissolve cut. Neither `combat::HitPoints` nor
/// `enemy::Hurt`.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct ObjMkie {
    /// `attr().mScale` (`d_a_obj_mkie.cpp:44`, `:85`) - the statue's uniform scale, the multiplier
    /// on the cull box literals and on the 130.0f eye lift.
    pub scale_factor: f32,
    /// `cPhs_ERROR_e` out of `Mthd_Create` (`.cpp:206`, `:216`) - see the struct doc.
    pub refused: bool,
}

impl ObjMkie {
    /// `daObjMkie::Act_c::Mthd_Create` (`.cpp:205-217`), minus `fopAcM_ct` (`:207`),
    /// `dComIfG_resLoad` (`:209`), `scale.setall` (`:212`), `MoveBGCreate` (`:213`) and the
    /// `JUT_ASSERT` (`:214`). The profile's `Parameters` field is `0` (`:486`).
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let kind = ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8;
        let switch_no = ((parameters >> SWSAVE_SHIFT) & SWSAVE_MASK) as u8;
        Self {
            scale_factor: STATUE_SCALE[kind as usize],
            refused: switches.test(switch_no),
        }
    }

    /// `fopAcM_setCullSizeBox(this, attr().mScale * -80.0f, attr().mScale * -1.0f,
    /// attr().mScale * -80.0f, attr().mScale * 80.0f, attr().mScale * 230.0f,
    /// attr().mScale * 80.0f)` (`d_a_obj_mkie.cpp:174-182`), matching the profile's
    /// `fopAc_CULLBOX_CUSTOM_e` (`:492`). `M_tri_src` holds a `cM3dGTriS` of three `Vec3::ZERO`s
    /// (`.cpp:36-38`) - i.e. collider-only vertices, not a cull shape.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        let s = self.scale_factor;
        (
            [s * -CULL_XZ, s * CULL_Y_MIN, s * -CULL_XZ],
            [s * CULL_XZ, s * CULL_Y_MAX, s * CULL_XZ],
        )
    }
}

pub fn is_obj_mkie(name: &str) -> bool {
    OBJ_MKIE_ACTORS.contains(&name)
}

/// One system, three params. Access set per param: `Commands` (W), `ObjMkie` (R). Nothing else is
/// touched, so there is no intra-system ambiguity. `Execute`'s surviving `attention_info` /
/// `eyePos` writes have no reader once `chk_light` is omitted and are dropped with it.
pub fn control(statues: Query<(Entity, &ObjMkie)>, mut commands: Commands) {
    for (entity, statue) in &statues {
        if statue.refused {
            let _ = commands.entity(entity).despawn();
        }
    }
}
