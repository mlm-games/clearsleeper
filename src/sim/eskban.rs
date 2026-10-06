//! The destructible rock of Great Sea Castle - `fpcNm_Obj_Eskban_e`, `d_a_obj_eskban.cpp`, stage
//! string `Eskban` (`d_stage.cpp:994`).
//!
//! (a) inbound damage: NONE. (b) outbound damage: NONE.
//!
//! All three of this actor's colliders set `SrcObjAt Type` to `0`: `cyl_check_src`
//! (`d_a_obj_eskban.cpp:23`), `cyl_camera_src` (`:53`) and `sph_check_src` (`:83`). The rock
//! therefore presents NO attack type at all - nothing in retail can register a hit against it and
//! it cannot damage anything. That is worth stating plainly for this actor in particular, because
//! it is the one place in the decomp where a bomb is deliberately meant to REACH an actor: the
//! destructibility is a TARGET test (`SrcObjTg Type = AT_TYPE_BOMB`, `:26` and `:84`), never an
//! attack. Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
//!
//! What IS fully modelled is the actor's other lifecycle half, and it is the important one.
//! `Mthd_Create` (`:147-162`) reads `param_get_swSave()` and returns `cPhs_STOP_e` when that
//! switch is already set (`:152-155`). That is the RULE 6b refusal shape: a
//! spawn-then-despawn-on-the-first-tick `refused` flag, never `Option<Self> -> None`.
//! `param_get_swSave()` is `daObj::PrmAbstract<PRM_SWSAVE_W=0x08, PRM_SWSAVE_S=0x00>`
//! (`d_a_obj_eskban.h:24-31`), i.e. `((1 << 8) - 1) & (param >> 0)` per `d_a_obj.h:14`, so the whole
//! switch range 0..255 is reachable and the REAL index is passed to `super::flags::Switches` per
//! RULE 11b. A value >= 0x80 reads false here because retail routes it to the per-zone
//! `dSv_zoneBit_c` arrays (`d_save.h:983-984`); the gap is in the save model, not in this actor,
//! and the index is deliberately not clamped.
//!
//! WHY THE REST OF THIS FILE IS A COMMENT AND NOT CODE. The rock's whole `mActorState` machine -
//! `ST_WAIT` / `ST_DESTROYED` / `ST_CUTSCENING` / `ST_VIBRATING` / `ST_SMOKING`
//! (`d_a_obj_eskban.h:33-39`), driven by the `switch (mActorState)` at `:287-367` - has three
//! entry gates, and every one of them needs the `cCcD` actor-collision registry that RULE 3b lists
//! as confirmed-absent. All three are REAL live retail behaviour, not dead code:
//!   * `mCameraCyl.ChkCoHit()` (`:289`) looks for a `Bomb2` actor by name -
//!     `fopAcM_GetName(hitAct) == fpcNm_Bomb2_e` (`:298`) - inside a cylinder whose Co side is
//!     `Set | IsOther | VsOther | NoCrr` (`:57`). This is what starts the sequence.
//!   * `mCheckCyl.ChkTgHit()` (`:317`) ends the cut. Tg is `AT_TYPE_BOMB` (`:26`), so a bomb
//!     hitting the rock is genuinely live.
//!   * `mCheckSph.ChkCoHit()` (`:271`) pushes a `fpcNm_NPC_MD_e` actor away from the rock, and is
//!     the only writer of positions in the whole file.
//! Substituting a distance-to-`super::bombflower::Blasts` test for these, the way `mjdoor.rs` does
//! for its own cylinder, would mean inventing that a bomb's `current.pos` is the point its `cCcD`
//! collider is tested at; that collider's geometry is `d_a_bomb.cpp`'s, outside this translation
//! unit. So the machine is transcribed as inert rather than approximated, and the timers that
//! only the unreachable arms move are cited here instead of stored as write-only fields:
//! `DESTROY_VIBRATION_LEN = 35` and `DESTROY_VIBRATION_SHOCK_FRAME_IDX = 28`
//! (`d_a_obj_eskban.h:10-11`, written at `:323` and `:350`), and `DESTROY_SMOKE_ANM_LEN = 20`
//! (`d_a_obj_eskban.h:12`, written at `:315` and `:324`). The event name is
//! `M_evname[7] = "Eskban"` (`:15`), used by `fopAcM_orderOtherEvent` (`:300`, `:310`),
//! `dComIfGp_evmng_cutEnd` (`:321`) and `dComIfGp_evmng_endCheck` (`:341`, `:362`); the arcname at
//! `:14` is the same seven characters but is the resource archive, not the event name.
//! `dComIfGp_getVibration().StartQuake(7, ~0x20, cXyz(0, 1, 0))` (`:327`) and
//! `StartShock(6, 0x4, cXyz(0, 1, 0))` (`:351`) are rumble, which RULE 3b lists as absent.
//!
//! `MoveBGCreate` / `MoveBGExecute` / `MoveBGDraw` (`:158`, `:400`, `:405`) are the
//! `dBgS_MoveBgActor` moving-background base that RULE 3b also lists as absent, and
//! `Mthd_Execute` is wired straight to `MoveBGExecute()` (`:400`), which is the `dBgS_ObjAcch`
//! accommodation pass - so it never even reaches the `Execute(Mtx**)` switch above it.
//! `MoveBGIsDelete` is the inherited form and is not reproduced.
//!
//! The profile's `Cull Type` is `fopAc_CULLBOX_CUSTOM_e` (`:437`) and the actor DOES call
//! `fopAcM_setCullSizeBox(this, -500, -1, -500, 500, 500, 500)` at `:122`, so this is a custom box
//! WITH a box and the literals are real. The odd `-1` minimum Y is transcribed exactly as written.

use bevy_ecs::prelude::*;

use super::flags::Switches;

/// `d_stage.cpp:994` binds `Eskban` to `fpcNm_Obj_Eskban_e`, the profile of
/// `d_a_obj_eskban.cpp:423-438`.
pub const ESKBAN_ACTORS: [&str; 1] = ["Eskban"];

/// `fopAcM_setCullSizeBox(this, -500, -1, -500, 500, 500, 500)` (`d_a_obj_eskban.cpp:122`).
pub const CULL_MIN: [f32; 3] = [-500.0, -1.0, -500.0];
pub const CULL_MAX: [f32; 3] = [500.0, 500.0, 500.0];

/// `daObj::PrmAbstract<PRM_SWSAVE_W=0x08, PRM_SWSAVE_S=0x00>` (`d_a_obj_eskban.h:24-31`,
/// `d_a_obj.h:14`): `((1 << 8) - 1) & (param >> 0)`.
fn param_sw_save(parameters: u32) -> u8 {
    (parameters & 0xFF) as u8
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eskban {
    /// `fopAcM_isSwitch(this, param_get_swSave())` at `Mthd_Create` (`:152-155`): `true` is the
    /// `return cPhs_STOP_e` refusal, which cancels the process inside Create
    /// (`f_pc_create_req.cpp:106-108`) so retail never executes even once.
    refused: bool,
}

impl Eskban {
    /// `Mthd_Create` (`:147-162`). The profile's `Parameters` field is `0`
    /// (`d_a_obj_eskban.cpp:431`) and the only parameter read is `param_get_swSave()`.
    ///
    /// Retail returns `cPhs_STOP_e` BEFORE `dComIfG_resLoad`, so a refused rock never loads the
    /// `Eskban` archive and never registers its moving background. Per RULE 6b that is a
    /// spawn-then-despawn-on-the-first-tick, so `spawn` always returns a value and the refusal is
    /// carried in the component.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        Self {
            refused: switches.test(param_sw_save(parameters)),
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_eskban(name: &str) -> bool {
    ESKBAN_ACTORS.contains(&name)
}

pub fn control(rocks: Query<(Entity, &Eskban)>, mut commands: Commands) {
    for (entity, rock) in &rocks {
        if rock.refused {
            let _ = commands.entity(entity).despawn();
        }
    }
}
