//! The boss warp point - `fpcNm_WARPFLOWER_e`, `d_a_warpf.cpp`, the actor behind the stage string
//! `Warpf`.
//!
//! COLLISION NOTE: `src/sim/gohdan.rs:336` and `src/sim/kalle_demos.rs:291` each hold a PRIVATE
//! `const WARP_ACTOR: &str = "Warpf"`, used only to name an event-staff entity those two modules
//! spawn (`gohdan.rs:826`, `kalle_demos.rs:730`), and `src/sim/event.rs:895` compares
//! `staff.name == "Warpf"` in the event cut dispatcher. None of the three is a stage-placement
//! claim: `gohdan.rs` and `kalle_demos.rs` declare them `const`, not `pub const`, and no
//! `spawn_actor` branch in `src/sim/mod.rs` tests `"Warpf"`. This file is therefore the only
//! `spawn_actor` claimant and it does not collide. `src/sim/warpfo.rs`'s `WARPFO_ACTORS` carries
//! `"Warpfo"`, a different string on a different byte.
//!
//! The actor's whole behaviour is indexed by `mStageNo =
//! dStage_stagInfo_GetSaveTbl(dComIfGp_getStageStagInfo())` (`:285`). That save-table index is
//! stage data this checkout does not parse - the same gap `shutter2.rs:358-360` documents - so
//! `m_arcname` (`:22-39`), `m_warp_size` (`:41-58`), `l_earth_pos` / `l_earth_pos_2nd`
//! (`:551-586`), `checkEndDemo` (`:209-257`), `get_distance` (`:542-547`), `set_effect`
//! (`:624-661`), `anim_play` (`:688-713`), `setEndAnim` (`:716-739`), `set_se`'s sound choice
//! (`:748-767`) and `set_mtx` / `_draw` (`:779-841`) are all omitted. `ui_meter_vitals::Env`
//! carries a `stage_save_tbl: u8` field (`src/ui_meter_vitals.rs:2282`) but nothing writes it, so
//! there is no resource to read either.
//!
//! What survives is `demo_proc` (`:375-402`) and its six cut handlers, plus `CreateInit`'s
//! boss-enemy refusal.
//!
//! (a) inbound damage: NONE. (b) outbound damage: NONE. `daWarpf_c` registers no `dCcD` collider -
//! the profile's `Status` is `fopAcStts_UNK4000_e | fopAcStts_UNK40000_e` (`:888`) and its `Group`
//! is `fopAc_ACTOR_e` (`:889`) - and `m290` / `m294` / `m298` / `m29C` are particle emitters
//! (RULE 3b). Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
//!
//! `daWarpf_IsDelete` (`:864-866`) is the bare "accepts a delete request" form of RULE 6b. The only
//! real destruction is `checkOrder`'s `fopAcM_delete(this)` (`:346`), whose `evmng_endCheck(m2D6)`
//! guard needs the stage event-info table (RULE 3b), so it is omitted too.

use bevy_ecs::prelude::*;

use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, MemBits};

/// `OBJNAME("Warpf", fpcNm_WARPFLOWER_e, 255, 0)` (`d_stage.cpp:611`) is the only
/// `fpcNm_WARPFLOWER_e` row.
pub const WARP_ACTORS: [&str; 1] = ["Warpf"];

/// `dComIfGp_evmng_getMyStaffId("Warpf")` (`:376`) - the same string as the stage name.
const STAFF: &str = "Warpf"; // <- d_a_warpf.cpp:376

/// `mDoAud_seStart(JA_SE_LK_BOSS_WARP_EFF_ST, ...)` (`:513`) in `actWarpMode_3`.
const SE_WARP_EFFECT_ST: u32 = 0x287D; // <- JAZelAudio_SE.h:571 (JA_SE_LK_BOSS_WARP_EFF_ST)

/// `i_this->mWarpTimer = 180;` (`:450`), `initWarpMode_1`.
const WARP_TIMER_1: i32 = 180; // <- d_a_warpf.cpp:450
/// `i_this->mWarpTimer = 5;` (`:478`), `initWarpMode_2`.
const WARP_TIMER_2: i32 = 5; // <- d_a_warpf.cpp:478
/// `i_this->mWarpTimer = 5;` (`:499`), `initWarpMode_3`.
const WARP_TIMER_3: i32 = 5; // <- d_a_warpf.cpp:499

/// `fopAc_CULLBOX_0_e` (`:890`) resolves through `l_cullSizeBox[0]` (`f_op_actor_mng.cpp:558`).
const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)

/// `action_table` (`:378-385`), the cut names `dComIfGp_evmng_getMyActIdx` matched against.
/// An unmatched cut is `actIdx == -1` (`:389`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Wait,
    WarpStart,
    Warp1,
    Warp2,
    Warp3,
    EndWait,
}

impl Action {
    fn of(name: &str) -> Option<Self> {
        match name {
            "WAIT" => Some(Self::Wait),
            "WARP_START" => Some(Self::WarpStart),
            "WARP_1" => Some(Self::Warp1),
            "WARP_2" => Some(Self::Warp2),
            "WARP_3" => Some(Self::Warp3),
            "END_WAIT" => Some(Self::EndWait),
            _ => None,
        }
    }

    /// `event_init_tbl` (`:356-363`): `initWait` (`:405-426`), `initWarpStart` (`:437-441`),
    /// `initWarpMode_1` (`:449-451`), `initWarpMode_2` (`:477-479`), `initWarpMode_3`
    /// (`:498-500`), `initEndWait` (`:521-522`).
    ///
    /// `initWait`'s whole body is three emitter invalidations, a `dBgS_ObjGndChk_Func` probe
    /// (RULE 3b) and `dComIfGp_evmng_setGoal` on that probe's result - nothing survives.
    /// `initWarpStart`'s only store is `m2EC = player->current.pos` (`:440`), whose three readers
    /// (`:459`, `:487`, `:511`) are particle arguments, so it is dropped too.
    fn init(&self, warp: &mut Warpf) {
        match self {
            Self::Wait => {}
            Self::WarpStart => {}
            Self::Warp1 => warp.timer = WARP_TIMER_1,
            Self::Warp2 => warp.timer = WARP_TIMER_2,
            Self::Warp3 => warp.timer = WARP_TIMER_3,
            Self::EndWait => {}
        }
    }

    /// `event_action_tbl` (`:365-372`).
    ///
    /// `actWait` (`:429-434`) fires a particle every tenth tick and returns TRUE; `actWarpStart`
    /// (`:444-446`) and `actEndWait` (`:525-527`) are bare TRUEs. `actWarpMode_1` (`:454-474`),
    /// `actWarpMode_2` (`:482-495`) and `actWarpMode_3` (`:503-518`) each end on the timer
    /// reaching zero; only `actWarpMode_3` also plays a sound (`:513`), `onEndDemo` (`:514`) being
    /// an empty body (`:260-261`).
    /// `true` stands for the `if (ret) dComIfGp_evmng_cutEnd(mStaffID)` of `demo_proc` (`:397-399`).
    fn act(&self, warp: &mut Warpf, events: &mut SimEvents) -> bool {
        match self {
            Self::Wait | Self::WarpStart | Self::EndWait => true,
            Self::Warp1 | Self::Warp2 | Self::Warp3 => {
                let done = warp.calc_timer();
                if done && *self == Self::Warp3 {
                    events.list.push(SimEvent::Se(SE_WARP_EFFECT_ST));
                }
                done
            }
        }
    }
}

/// The boss warp point.
///
/// `_create` (`:282-316`) never refuses on its own; the refusal comes from `CreateInit`
/// (`:264-279`) returning `cPhs_ERROR_e`, which `_create` passes straight up (`:298-300`), and
/// `cPhs_ERROR_e` during Create cancels the process before Execute ever runs
/// (`f_pc_create_req.cpp:106-108`). RULE 6b forbids hiding that in `Option<Self> -> None`, which
/// would leave a bare `Actor` retail never keeps, so the check is discharged on the first tick -
/// and it has to be, because `dComIfGs_isStageBossEnemy()` is the `MemBits` resource, not a
/// parameter.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Warpf {
    /// `mWarpTimer` (`d_a_warpf.h`), seeded by the three `init` arms and drained by the three
    /// `act` arms.
    pub timer: i32,
    /// One-shot guard for the `field_0x680`-style first-tick test. Retail evaluates
    /// `dComIfGs_isStageBossEnemy()` once, inside `CreateInit`; the sim reads the resource on the
    /// first tick instead, because spawn-time access to a `Res<MemBits>` is not available to
    /// `spawn`. `true` until that tick has run.
    armed: bool,
}

impl Warpf {
    /// `_create` (`:282-316`) minus `fopAcM_ct` (`:283`), the `m_arcname` resource load (`:288-289`),
    /// `fopAcM_entrySolidHeap` / `CheckCreateHeap` / `CreateHeap` (`:291`, `:61-178`),
    /// `CreateInit` (`:297-300`, the refusal) and the `fopKyM_fastCreate(fpcNm_LEVEL_SE_e, ...)`
    /// sound object (`:302-313`, whose `seNum` is `mStageNo`-dependent and whose level has no
    /// resource). The profile's `Parameters` field is `0` (`:885`); the only parameter read,
    /// `daWarpf_prm::getSetType` = `fopAcM_GetParam(this) >> 0x1C` (`:19`), guards the sound choice
    /// (`:750`, `:627`, `:640`) and the particle branch (`:599-601`), all of which are omitted, so
    /// nothing decodes it and `refused` stays out of `spawn`.
    pub fn spawn(_parameters: u32) -> Self {
        Self {
            timer: 0,
            armed: true,
        }
    }

    /// `fopAc_CULLBOX_0_e` (`:890`) -> `l_cullSizeBox[0]` (`f_op_actor_mng.cpp:558`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `cLib_calcTimer` (`c_lib.h:85-91`), as `actWarpMode_1` / `2` / `3` call it.
    fn calc_timer(&mut self) -> bool {
        if self.timer != 0 {
            self.timer -= 1;
        }
        self.timer == 0
    }

    /// `demo_proc` (`:375-402`). `dComIfGp_evmng_getMyStaffId` is `ActiveEvent::staff_cut` under
    /// the same name and `getMyActIdx` is `EventList::cut_name` matched against `action_table`.
    /// `!eventInfo.checkCommandTalk()` (`:377`) is the event manager's TALK command, which this
    /// sim leaves false - the same proven default `warpfo.rs:120-122` and `ba1.rs:1290-1292`
    /// use - so the staff lookup alone carries the arm.
    ///
    /// `true` is `dComIfGp_evmng_cutEnd(mStaffID)`, which retail calls from the `actIdx == -1`
    /// arm (`:390`) and from a TRUE action result (`:398`).
    fn demo_proc(
        &mut self,
        list: &EventList,
        cut: usize,
        advance: bool,
        events: &mut SimEvents,
    ) -> bool {
        let Some(action) = list.cut_name(cut).and_then(Action::of) else {
            return true;
        };
        if advance {
            action.init(self);
        }
        action.act(self, events)
    }
}

pub fn is_warpf(name: &str) -> bool {
    WARP_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut flags: ResMut<EventFlags>,
    mem: Res<MemBits>,
    mut events: ResMut<SimEvents>,
    mut warps: Query<(Entity, &mut Warpf)>,
    mut commands: Commands,
) {
    let mut spent: Vec<Entity> = Vec::new();
    for (entity, mut warp) in &mut warps {
        if warp.armed {
            warp.armed = false;
            // `CreateInit` (`:266-268`): `if (!dComIfGs_isStageBossEnemy()) return cPhs_ERROR_e;`
            if !mem.is_stage_boss_enemy() {
                spent.push(entity);
                continue;
            }
        }
        // `mStaffID = dComIfGp_evmng_getMyStaffId("Warpf")` (`:376`) with
        // `dComIfGp_event_runCheck() && !eventInfo.checkCommandTalk() && mStaffID != -1` (`:377`).
        let Some((cut, advance)) = active.staff_cut(&list, STAFF) else {
            continue;
        };
        if warp.demo_proc(&list, cut, advance, &mut events) {
            active.end_staff_cut(&list, &mut flags, STAFF);
        }
    }
    for entity in spent {
        let _ = commands.entity(entity).despawn();
    }
}
