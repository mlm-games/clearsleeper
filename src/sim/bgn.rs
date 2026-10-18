//! The Great Dragon's collider proxies (`d_a_att.cpp`, `fpcNm_ATT_e`).
//!
//! There is deliberately no `*_ACTORS` array and no `is_*` predicate here. **`fpcNm_ATT_e` has no
//! row in the OBJNAME table of `d_stage.cpp`.** The only `Bgn` entry, `d_stage.cpp:1193`, belongs
//! to `fpcNm_BGN_e` - the separate 3716-line boss in `d_a_bgn.cpp`, not this file. An `att_class` is
//! therefore never stage-placed: retail creates one only at runtime, by
//! `fopAcM_create(fpcNm_ATT_e, ...)` from `d_a_bgn.cpp:3670`, `d_a_bst.cpp:3163` (parameter 100) and
//! `d_a_bwd.cpp:2581` (parameter `0x65 == 101`). None of those three actors has a module in this
//! sim. Naming a stage string would mean inventing one the decomp does not contain, so this module
//! is wired by whichever of those three bosses is ported, not by `spawn_actor`.

use bevy_ecs::prelude::*;

/// `a_this->m2B5 = fopAcM_GetParam(a_this) & 0xFF` (`d_a_att.cpp:235`), narrowing into the
/// `u8 m2B5` field (`d_a_att.h:15`). `fopAcM_GetParam` returns the whole 32 bits and the file masks
/// to 8, so no upper parameter byte is read anywhere in it.
const PARAM_MASK: u32 = 0xFF; // <- d_a_att.cpp:235

/// A single collider proxy of the Great Dragon - `att_class` (`d_a_att.h:7-22`).
///
/// Damage classification:
/// (a) inbound damage: NONE. (b) outbound damage: NONE.
///
/// `att_class` has no health field, no `damage` call and no reader of any HP value, so nothing can
/// hurt it. The three static collider sources confirm the outbound side too: `cc_cyl_src`
/// (`d_a_att.cpp:174-202`) has `SrcObjAt Type = 0` (`:177`) and `SrcObjAt SPrm = 0` (`:180`), so it
/// is a Co-only cylinder (`SrcObjCo SPrm = cCcD_CoSPrm_Set_e | cCcD_CoSPrm_IsPlayer_e |
/// cCcD_CoSPrm_VsGrpAll_e`, `:183`); `bm_sph_src` (`:203-230`) carries
/// `SrcObjAt SPrm = cCcD_AtSPrm_Set_e | cCcD_AtSPrm_VsPlayer_e` (`:209`) but its `SrcObjAt Type`
/// and `SrcObjAt Atp` are both 0 (`:206-207`), so no attack is ever registered against the
/// player; and `sita_sph_src` (`:145-172`) is an `AT_TYPE_HOOKSHOT` target (`:152`) with the same
/// zeroed AT type (`:147-148`). The profile `Group` is `fopAc_ENEMY_e` (`:284`) only because the
/// proxy rides along on the boss. Neither `combat::HitPoints` nor `enemy::Hurt` may be attached:
/// `combat::control` would delete an actor nothing can damage.
///
/// `daAtt_Create` (`:143-261`) ends on an unconditional `cPhs_COMPLEATE_e` (`:260`) - the file has
/// no `cPhs_ERROR_e` arm at all - and `daAtt_Execute` ends on an unconditional TRUE (`:128`), so
/// there is no spawn-then-despawn path. `daAtt_IsDelete` (`:132-135`) is the bare "accepts a
/// delete request" form of RULE 6b.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Att {
    /// `m2B5` (`d_a_att.h:15`) - `101` selects the USA-only `sita_sph_src` hookshot anchor (`:32`,
    /// `:239`), `100` is the attention-only arm (`:41`), and `< 10` is the only Create arm that
    /// installs colliders (`:248`), hence the only value for which `boss->mAAA8[r30]` (an 8-entry
    /// array, `d_a_bgn.h:115`) and `boss->mC33C[r30]` (also 8, `d_a_bgn.h:124`) stay in bounds -
    /// every other value reads both out of bounds in retail.
    pub index: u8,
}

impl Att {
    /// `daAtt_Create` (`:143-261`), minus the three static `dCcD_Src*` tables, `fopAcM_ct`,
    /// `mStts.Init`, `mSph.Set`/`SetStts`, `mCyl.Set`/`SetStts`,
    /// `fopAcM_OnStatus(fopAcStts_UNK10000000_e)` (`:240`) and
    /// `attention_info.distances[fopAc_Attn_TYPE_BATTLE_e] = 4` (`:236`).
    pub fn spawn(parameters: u32) -> Self {
        Self {
            index: (parameters & PARAM_MASK) as u8,
        }
    }
}
