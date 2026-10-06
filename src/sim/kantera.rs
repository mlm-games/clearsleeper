use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::flags::Switches;

/// `OBJNAME("Kantera", fpcNm_KANTERA_e, 255, 0)` (`d_stage.cpp:1242`) is the only row bound to
/// `fpcNm_KANTERA_e` (`d_a_kantera.cpp:439`).
///
/// NAME MISMATCH, checked explicitly: the translation unit's class is `kantera_class`
/// (`d_a_kantera.h:24`), its profile is `g_profile_KANTERA` (`:435`), its enum is
/// `fpcNm_KANTERA_e` (`:439`) and its resource name is the bare literal `"Kantera"`
/// (`dComIfG_resLoad(&i_this->mPhase, "Kantera")`, `:386`). There is no `_arcname` member on
/// `kantera_class`, so the stage string, the class name and the archive name are all "Kantera"
/// only by spelling, not by any shared constant - the profile name is the odd one out.
pub const KANTERA_ACTORS: [&str; 1] = ["Kantera"];

/// `fopAcM_setCullSizeBox(a_this, -60.0f, -100.0f, -60.0f, 60.0f, 50.0f, 60.0f)`
/// (`d_a_kantera.cpp:405`). The profile's `fopAc_CULLBOX_CUSTOM_e` (`:449`) resolves here; this
/// file calls neither `fopAcM_SetMin` / `SetMax` nor any other box setter, so this call is the
/// whole box.
const CULL_MIN: [f32; 3] = [-60.0, -100.0, -60.0]; // <- d_a_kantera.cpp:405
const CULL_MAX: [f32; 3] = [60.0, 50.0, 60.0]; // <- d_a_kantera.cpp:405

/// `u8 mSwitchNo;` (`d_a_kantera.h:30`) - the create-time default is `0`, which
/// `daKantera_Execute`'s `if (i_this->mSwitchNo != 0)` (`:222`) reads as "already lit".
pub const SWITCH_LIT: u8 = 0; // <- d_a_kantera.h:30 (fopAcM_ct zero-fill)
/// `i_this->mSwitchNo = 0;` (`:224`) - the value the gate latches to once its switch reads.
pub const SWITCH_CLEARED: u8 = 0; // <- d_a_kantera.cpp:224
/// `u8 mParam2;` from `fopAcM_GetParam(a_this) >> 0x18` (`:384`).
pub const PARAM_HIGH_SHIFT: u32 = 0x18; // <- d_a_kantera.cpp:384
/// `if (i_this->mParam2 != 0xff) { i_this->mSwitchNo = i_this->mParam2 + 1; }` (`:400-402`).
pub const PARAM_NO_SWITCH: u8 = 0xFF; // <- d_a_kantera.cpp:400
/// The same `+ 1` (`:401`), undone by the `- 1` of the gate's
/// `dComIfGs_isSwitch(i_this->mSwitchNo - 1, ...)` (`:223`) - so the switch actually tested is
/// `param >> 24` verbatim, and only the 0xFF "unset" sentinel round-trips through the bias.
pub const SWITCH_BIAS: u32 = 1; // <- d_a_kantera.cpp:401 (added), :223 (subtracted)

/// The wall-mounted brazier that a switch can douse.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daKantera_Create` builds one
/// `dCcD_Sph` from `at_sph_src` (`:349-376`) whose `SrcObjAt Type` is `AT_TYPE_FIRE` (`:353`) with
/// `cCcD_AtSPrm_VsPlayer_e` (`:355`) - so retail DOES burn the player - but `mSph.Set` /
/// `mSph.SetStts` are `cCcD` registry work and RULE 3b lists the `cCcD` actor-collision registry as
/// confirmed-absent. `SrcObjTg Type` is `0` (`:357`), so retail cannot damage the brazier either.
/// Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
///
/// `daKantera_IsDelete` (`:271-273`) is the bare "accepts a delete request" form of RULE 6b, and
/// `fopAcM_delete` appears exactly once in this translation unit (`:239`, inside the `m35C` arm).
/// `daKantera_Create` returns `cPhs_ERROR_e` only from the `fopAcM_entrySolidHeap` else arm
/// (`:389`), which is heap work, so `spawn` never refuses and needs no `refused` flag.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kantera {
    /// `mSwitchNo` (`d_a_kantera.h:30`). `0` means the gate at `:222` is open; any other value is
    /// the biased switch number and the actor stays dormant until that switch reads.
    pub switch_no: u8,
    /// The create-time `a_this->scale.x/y/z = 0.0f` arm (`:415-417`), which retires the brazier
    /// without deleting it. Kept because it is the actor's only visible/hittable state change and
    /// retail keeps the entity alive.
    pub doused: bool,
}

impl Kantera {
    /// `daKantera_Create` (`:348-425`) minus the resource load, heap entry, joint callbacks,
    /// collider registration, particle callbacks, `dKy_plight_set` and the trailing
    /// `daKantera_Execute(i_this)` call at `:422`.
    ///
    /// `i_this->mParam0 = fopAcM_GetParam(a_this);` (`:382`) and
    /// `i_this->mParam1 = fopAcM_GetParam(a_this) >> 8;` (`:383`) are NOT stored: `mParam0`'s only
    /// reader is the `== 7` test at `:414` (folded into `doused`), and the write
    /// `i_this->mParam0 = 1;` (`:418`) has no reader at all, while `mParam1` is never read
    /// anywhere in the translation unit. RULE 7 forbids keeping an unread field.
    ///
    /// `i_this->mState` (`d_a_kantera.h:364`) is never written, so the `mState >= 10` split in
    /// `daKantera_Draw` (`:53`) and the `mState < 10` block in `daKantera_Execute` (`:244`) both
    /// take their constant arm; the former is renderer and the latter is model posing.
    pub fn spawn(parameters: u32) -> Self {
        let high = ((parameters >> PARAM_HIGH_SHIFT) & 0xFF) as u8;
        Self {
            // `:400-402`
            switch_no: if high == PARAM_NO_SWITCH {
                SWITCH_LIT
            } else {
                (high as u32 + SWITCH_BIAS) as u8
            },
            // `:414-419` - `if (i_this->mParam0 == 7) { scale = 0; mParam0 = 1; }`. The `mParam0`
            // low byte is `parameters & 0xFF`.
            doused: parameters & 0xFF == 7,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// The switch index the gate actually reads: `mSwitchNo - 1` (`:223`), which reverses the
    /// `+ 1` bias from `:401`. A `0` gate has no switch at all.
    fn switch_index(&self) -> Option<u8> {
        match self.switch_no {
            SWITCH_LIT | SWITCH_CLEARED => None,
            other => Some(other.wrapping_sub(SWITCH_BIAS as u8)),
        }
    }

    /// `dComIfGp_checkCommandTalk` / the `daKantera_Execute` gate at `:222-228`:
    /// ```
    /// if (i_this->mSwitchNo != 0) {
    ///     if (dComIfGs_isSwitch(i_this->mSwitchNo - 1, fopAcM_GetRoomNo(&i_this->actor))) {
    ///         i_this->mSwitchNo = 0;
    ///     } else { return TRUE; }
    /// }
    /// ```
    /// The switch is re-read EVERY tick while it is unset, so the latch is one-shot: once the
    /// switch reads, `mSwitchNo` is cleared and never re-arms.
    pub fn gate(&mut self, switches: &Switches) {
        let Some(index) = self.switch_index() else {
            return;
        };
        if switches.test(index) {
            self.switch_no = SWITCH_CLEARED;
        }
    }
}

/// `daKantera_Execute` (`d_a_kantera.cpp:221-268`).
///
/// Everything downstream of the gate is `kantera_move(i_this)` - whose body is EMPTY
/// (`:217-218`, the decomp even bothers to give it a symbol range) - plus the `mState < 10` block
/// at `:244-266`, which is `mpModel1->calc()`, four matrix calls and two `mpBrkAnm->play()` calls,
/// i.e. renderer. So the gate latch is the whole modelled behaviour, and `control` records it
/// faithfully without pretending a visible change follows.
pub fn control(switches: Res<Switches>, mut braziers: Query<(&mut Actor, &mut Kantera), With<Kantera>>) {
    for (mut actor, mut brazier) in &mut braziers {
        brazier.gate(&switches);
        // ```
        // if (i_this->mParam0 == 7) { a_this->scale.x = 0.0f; a_this->scale.y = 0.0f;
        //                             a_this->scale.z = 0.0f; i_this->mParam0 = 1; }
        // ```
        // (`:414-419`). RULE 2: `mpModel2->setBaseScale(a_this->scale)` at `:399` runs BEFORE this
        // arm, so the zeroing is a write-only stamp that the model matrix never sees - which is why
        // re-applying it every tick is idempotent rather than a change of behaviour. The 1.0 in the
        // other arm is retail's own default, not a stand-in: `dStage_actorInit` never writes
        // `fopAcM_prm_class::scale`.
        actor.scale = if brazier.doused {
            Vec3::ZERO
        } else {
            Vec3::ONE
        };
    }
}

pub fn is_kantera(name: &str) -> bool {
    KANTERA_ACTORS.contains(&name)
}
