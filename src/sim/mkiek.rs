use bevy_ecs::prelude::*;

use super::flags::Switches;

/// The stage string is `MkieK` (`d_stage.cpp:951`); it is also `daObjMkiek::Act_c::M_arcname` and
/// `M_envname` (`d_a_obj_mkiek.cpp:44-45`).
pub const MKIEK_ACTORS: [&str; 1] = ["MkieK"];

/// `daObj::PrmAbstract(this, PRM_SWSAVE_W = 0x08, PRM_SWSAVE_S = 0x00)` (`d_a_obj_mkiek.h:18-19`,
/// `:34`, `d_a_obj.h:14-17`) - `((1 << 8) - 1) & (param >> 0)`.
const SWSAVE_WIDTH: u32 = 0x08; // <- d_a_obj_mkiek.h:18 (PRM_SWSAVE_W)
const SWSAVE_SHIFT: u32 = 0x00; // <- d_a_obj_mkiek.h:19 (PRM_SWSAVE_S)
/// `daObj::PrmAbstract(this, PRM_SOUND_W = 0x01, PRM_SOUND_S = 0x08)` (`d_a_obj_mkiek.h:21-22`,
/// `:25`) - `((1 << 1) - 1) & (param >> 8)`.
const SOUND_WIDTH: u32 = 0x01; // <- d_a_obj_mkiek.h:21 (PRM_SOUND_W)
const SOUND_SHIFT: u32 = 0x08; // <- d_a_obj_mkiek.h:22 (PRM_SOUND_S)
/// `fopAcM_setCullSizeBox(this, -200.0f, -100.0f, -200.0f, 200.0f, 305.0f, 200.0f)`
/// (`d_a_obj_mkiek.cpp:75`) - a real custom cull box, so this profile's
/// `fopAc_CULLBOX_CUSTOM_e` (`:296`) resolves.
const CULL_MIN: [f32; 3] = [-200.0, -100.0, -200.0]; // <- d_a_obj_mkiek.cpp:75
/// `fopAcM_setCullSizeBox(this, ..., 200.0f, 305.0f, 200.0f)` (`d_a_obj_mkiek.cpp:75`).
const CULL_MAX: [f32; 3] = [200.0, 305.0, 200.0]; // <- d_a_obj_mkiek.cpp:75

/// The Volcano stage's light-blocked vanishing wall - `daObjMkiek::Act_c` (`d_a_obj_mkiek.h:15-73`).
///
/// `sph_check_src` (`d_a_obj_mkiek.cpp:15-42`) has `Flags = 0` (`:18`), `SrcObjAt Type = 0`
/// (`:20`), `SrcObjAt Atp = 0` (`:21`), `SrcObjAt SPrm = 0` (`:22`) and `SrcObjCo SPrm = 0` (`:24`).
/// Its only role is `SrcObjTg Type = AT_TYPE_LIGHT` (`:22`) with
/// `cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsEnemy_e` (`:23`) - a sensor, not a health path, and RULE 3b
/// lists light-source semantics among the absent inputs besides. Nothing damages the wall and it
/// damages nothing, and the class has no health field; the profile `Group` is `fopAc_ACTOR_e`
/// (`:295`).
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. Neither `combat::HitPoints` nor
/// `enemy::Hurt` is attached.
///
/// `Mthd_Create` (`:86-100`) returns `cPhs_STOP_e` when `fopAcM_isSwitch(this, switch_index)`
/// (`:90-92`), and `cPhs_STOP_e` cancels the actor inside Create
/// (`f_pc_create_req.cpp:107-108`), so that arm is a refusal despawned on the first tick. Otherwise
/// it can only return `cPhs_ERROR_e` from `MoveBGCreate`
/// (`d_bg_s_movebg_actor.cpp:60-66`), never from a parameter, and `Create` (`:67-83`) ends on an
/// unconditional TRUE, so there is no other spawn-then-despawn path. `Mthd_IsDelete` is
/// `dBgS_MoveBgActor::IsDelete()`, the bare "accepts a delete request" form of RULE 6b.
///
/// `Mthd_Execute` is `MoveBGExecute()` (`:258`), which forwards to the virtual
/// `Execute(Mtx**)` (`:199`) - so `Execute` is the real per-tick body. Its `set_mtx` (`:218`) and
/// `*o_mtx = &M_tmp_mtx` (`:219`) are renderer and `dBgW` work.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mkiek {
    /// `prm_get_swSave()` (`d_a_obj_mkiek.h:26`), the `daObj::PrmAbstract(this, 8, 0)` index
    /// `Mthd_Create` tests at `:90` and `demo_wait` turns on at `:174`.
    pub switch_no: u8,
    /// `prm_get_sound()` (`d_a_obj_mkiek.h:25`), `daObj::PrmAbstract(this, 1, 8)`. Its only reader
    /// is `if (prm_get_sound() == 0) mDoAud_seStart(JA_SE_READ_RIDDLE_1)` inside `demo_wait`
    /// (`d_a_obj_mkiek.cpp:149-151`), which `check` is the only route into - and `check` is omitted
    /// (see below), so this field has no reader in this port.
    pub sound: u8,
    /// `Mthd_Create`'s `cPhs_STOP_e` arm, which cancels the actor inside Create.
    pub refused: bool,
}

impl Mkiek {
    /// `Mthd_Create` (`:86-100`) plus `Create` (`:67-83`), minus `fopAcM_ct`, `dComIfG_resLoad`,
    /// `MoveBGCreate`, `CreateHeap`, `mStts.Init`, `mSph.Set`/`SetStts`, `fopAcM_SetMtx`, `init_mtx`,
    /// `fopAcM_setCullSizeBox`, `mBrkAnm.init` and `dComIfGp_evmng_getEventIdx` - archive, heap,
    /// `dBgW`, model and event-table work. What survives is the switch refusal (`:89-92`) and the
    /// two parameter decodes.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = ((parameters >> SWSAVE_SHIFT) & ((1 << SWSAVE_WIDTH) - 1)) as u8;
        Self {
            switch_no,
            sound: ((parameters >> SOUND_SHIFT) & ((1 << SOUND_WIDTH) - 1)) as u8,
            refused: switches.test(switch_no),
        }
    }

    /// `fopAcM_setCullSizeBox(this, -200.0f, -100.0f, -200.0f, 200.0f, 305.0f, 200.0f)`
    /// (`d_a_obj_mkiek.cpp:75`) - the actor's one real custom cull box.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_mkiek(name: &str) -> bool {
    MKIEK_ACTORS.contains(&name)
}

pub fn control(walls: Query<(Entity, &Mkiek)>, mut commands: Commands) {
    for (entity, wall) in &walls {
        if wall.refused {
            let _ = commands.entity(entity).despawn();
        }
    }
}
