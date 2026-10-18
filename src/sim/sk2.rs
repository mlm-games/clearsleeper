use bevy_ecs::prelude::*;

use super::flags::Switches;

pub const SK2_ACTORS: [&str; 2] = ["Turu2", "Turu3"];

const BYTE_MASK: u32 = 0xFF; // <- d_a_sk2.cpp:193-207 (whole-parameter byte reads, no prm accessors)
const SWITCH_SHIFT: u32 = 0x18; // <- d_a_sk2.cpp:207 (fopAcM_GetParam(a_this) >> 0x18)
const NO_SWITCH: u8 = 0xFF; // <- d_a_sk2.cpp:196, :200, :208

pub const CULL_MIN: [f32; 3] = [-800.0, -200.0, -400.0]; // <- d_a_sk2.cpp:227 (fopAcM_SetMin)
pub const CULL_MAX: [f32; 3] = [500.0, 1000.0, 1000.0]; // <- d_a_sk2.cpp:228 (fopAcM_SetMax)

/// `sk2_class` (`d_a_sk2.h:10-29`) is the Dragon Roost / Great Sea shrine-platform lift, registered
/// under the stage strings `Turu2` and `Turu3` (`d_stage.cpp:790-791`, both `fpcNm_SK2_e`).
/// It is a separate translation unit from `super::turu`, which owns `Turu` (`d_stage.cpp:789`,
/// `fpcNm_SK_e`) and `d_a_sk.cpp`; the two share no state and neither `is_sk2` nor `is_turu`
/// matches the other's stage string.
///
/// `sk2_class` holds a `fopEn_enemy_c actor` member (`d_a_sk2.h:11`) - composition, not
/// inheritance, which is why the decomp reaches `i_this->actor.current` (`:106-109`) and
/// `i_this->actor.heap` (`:131`). The profile's `Group` is nevertheless `fopAc_ACTOR_e`
/// (`d_a_sk2.cpp:259`), so `fopAcM_myRoomSearchEnemy`'s `group == fopAc_ENEMY_e` test
/// (`f_op_actor_mng.cpp:1282-1296`) never finds it, and `attention_info.flags = 0` (`:229`)
/// disables attention. No `dCcD` AT or Tg collider is ever registered - the only registration is
/// the deform background `dComIfG_Bgsp()->Regist(mpBgW, ...)` (`:231`) - so this actor cannot be
/// hit at all and takes no `HitPoints`/`Hurt`.
///
/// `dousa_move` (`:53-98`) is the only non-renderer code left after the resources are accounted
/// for, and its mode machine is not decidable here: `case 1` (`:61-65`) is gated on
/// `mpMorf->checkFrame(0.0f)` / `checkFrame(18.0f)`, which is `J3DFrameCtrl::checkPass`
/// (`m_Do_ext.h:32`) and compares the frame against `mEnd` (`J3DAnimation.cpp:24-49`,
/// `:60-74`). `mEnd`/`mLoop` come from the `KSYLF_00`/`KSYLF_01` BCK frame count, and BCK/BTP
/// archives ship as `.h` index headers only. `case 0` (`:56-60`) falls straight through into that
/// gate, so no arm of the machine can be entered without deciding it, and the remaining writes
/// (`m2BA` `:59`, `m2BC` `:71`/`:78`, `m2C0` `:58`/`:69`/`:70`/`:79`, the `m2CE` joint table
/// `:86-93` - consumed only by `nodeCallBack` `:26-36` - and `m2CC` `:97`) are all unreachable.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sk2 {
    /// `daSk2_Create` (`:208-210`) returns `cPhs_ERROR_e`, which cancels the actor inside Create
    /// (`f_pc_create_req.cpp:107-108`) so retail never runs `daSk2_Execute` on this term.
    pub refused: bool,
}

impl Sk2 {
    /// `daSk2_Create` (`d_a_sk2.cpp:186-236`), reduced to the only decision that is not resource
    /// dependent. `m2B4` (`:193`, `0xff` folded to 0 at `:196`) is the `KSYLF_00`/`KSYLF_01`
    /// BMD/BCK/DZB selector, `m2B5` (`:194`) becomes `m2C4` (`:201-204`, clamped to
    /// `START_FRAME_MAX = 35.0f` at `:202`), and both are read nowhere but `useHeapInit`
    /// (`:143-180`) and `setFrame` (`:57`); `m2B6` (`:207`) is the refusal switch.
    ///
    /// `fopAcM_isSwitch(a_this, m2B6)` is `dComIfGs_isSwitch(i_no, roomNo)`, so a `m2B6` of
    /// `0x80` or more reads the per-zone `dSv_zoneBit_c` array rather than the in-memory one
    /// (`d_save.h:982`, `d_save.cpp:1585-1601`). `flags::Switches` models only the 0..128 array, so
    /// under RULE 11b such a placement reads false here and this branch is taken - that is the
    /// save model's gap, not a workaround.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = ((parameters >> SWITCH_SHIFT) & BYTE_MASK) as u8;
        Self {
            refused: switch_no != NO_SWITCH && switches.test(switch_no),
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_sk2(name: &str) -> bool {
    SK2_ACTORS.contains(&name)
}

pub fn control(lifts: Query<(Entity, &Sk2)>, mut commands: Commands) {
    for (entity, lift) in &lifts {
        if lift.refused {
            let _ = commands.entity(entity).despawn();
        }
    }
}
