//! `daWarphr_c` (`d_a_warphr.h:10-67`) - the Hyrule Castle / Great Sea warp rule.
//! `Nonmatching` count: 0, so every token below is transcribed.
//!
//! Nothing in `src/sim/` claims `Ghrwp` as an actor name. `src/sim/event.rs:881`'s
//! `staff.name == "Ghrwp"` is the central event manager's staff dispatch, not a spawn claim, and
//! it composes with this module rather than duplicating it.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, MemBits};
use super::player_status::{PlayerStatus, STTS0_SHIP_RIDE};
use super::ship::SHIP_ACTOR;

/// `OBJNAME("Ghrwp", fpcNm_WARPHYRULE_e, 255, 0)` (`d_stage.cpp:1157`). `fpcNm_WARPHYRULE_e` is
/// `d_a_warphr.cpp` and the archive `daWarphr_c::m_arcname[] = "Ghrwp"` (`:21`) agrees.
pub const WARPHYRULE_ACTORS: [&str; 1] = ["Ghrwp"];

/// `fopAcM_setCullSizeBox(this, -300.0f, 0.0f, -300.0f, 300.0f, 5000.0f, 300.0f)`
/// (`d_a_warphr.cpp:129`) - the actor's one real custom cull box, matching the profile's
/// `fopAc_CULLBOX_CUSTOM_e` (`:518`). `fopAcM_setCullSizeFar(this, 1.0f)` (`:130`) is the far clip -
/// `fopAcM_cullingCheck` (`f_op_actor_mng.cpp:589+`) uses it only to pick the clipper, and
/// `view::ActorView` carries no far field, so it is documented rather than stored.
const CULL_MIN: [f32; 3] = [-300.0, 0.0, -300.0]; // <- d_a_warphr.cpp:129
const CULL_MAX: [f32; 3] = [300.0, 5000.0, 300.0]; // <- d_a_warphr.cpp:129
/// `if (abs < 500.0f)` in `check_warp` (`.cpp:411`) - an XZ distance from the ship to the rule.
/// `fopAcM_searchActorDistanceXZ` is `(*other - *this).absXZ()` (`f_op_actor_mng.cpp:511-515`), but
/// here the decomp inlines `(ship->current.pos - current.pos).absXZ()`, which is the same value.
const SHIP_RADIUS: f32 = 500.0; // <- d_a_warphr.cpp:411

/// `dComIfGs_isEventBit(dSv_event_flag_c::UNK_1001)` in `get_return_count` (`.cpp:399`).
const EVT_1001: u32 = 0x1001; // <- d_a_warphr.cpp:399, d_save_event_flag.inc:126
/// `dComIfGs_onEventBit(dSv_event_flag_c::UNK_3810)` in `normal_execute` (`.cpp:207`). Under RULE
/// 11c `EventFlags` is byte-indexed and both ids are `hi_byte << 8 | bit_mask`, so the retail ids
/// go straight through unchanged.
const EVT_3810: u32 = 0x3810; // <- d_a_warphr.cpp:207, d_save_event_flag.inc:428

/// `fopAcM_seStart(this, JA_SE_OBJ_HL_WAPR_EFF, 0)` at the tail of `_execute` (`.cpp:197`,
/// `JAZelAudio_SE.h:322`) - one request every frame the actor runs.
const SE_OBJ_HL_WAPR_EFF: u32 = 0x1088; // <- JAZelAudio_SE.h:322

/// `daWarphr_c` - the warp rule that sends the King of Red Lions back out to sea.
///
/// The struct has NO fields, and that is a consequence, not an omission of bookkeeping. Every
/// per-actor field in `daWarphr_c` (`d_a_warphr.h:10-67`) is written by an omitted body:
///
/// * `daWarphr_prm::getType` (`d_a_warphr.cpp:16-18`) decodes `(fopAcM_GetParam(i_this) >> 0x1C) &
///   0xFF` into `m2FA` (`:143`) and nothing in the file ever reads `m2FA` again - it is write-only
///   in retail too.
/// * `CreateInit` (`:127-137`) stores `dComIfGp_evmng_getEventIdx("TO_SEA_WARP_2" | "TO_SEA_WARP_1")`
///   into `m2F8` (`:133`, `:135`) - the stage event-info table (RULE 3b). Which of the two keys it
///   asks for is `dComIfGs_isEventBit(dSv_event_flag_c::UNK_2D08)` (`:132`, `0x2D08`,
///   `d_save_event_flag.inc:343`), and `m2F8`'s readers are
///   `eventOrder`'s `fopAcM_orderOtherEventId(this, m2F8)` (`:329`) and `checkOrder`'s
///   `dComIfGp_evmng_startCheck(m2F8)` / `endCheck(m2F8)` (`:337`, `:341`), all in that same pair.
/// * `m2F0` (`:193`, `:209`, `:338`) is the "warp has been ordered" latch: `normal_execute` sets it
///   to 2 (`:209`), `checkOrder` clears it once the ordered event reaches Play (`:337-339`), and
///   `eventOrder` reads it (`:328`). Both readers need the omitted event index.
/// * `m304` (`:236`) is `dComIfGp_evmng_getMyStaffId("Ghrwp")`, consumed only by `demo_proc`.
/// * `m300` (`:219`, `:220`, `:225`) is `dDemo_actor_c::getShapeId()` from the demo manager.
/// * `m2E4` (`:174`, `:185`) is the monotone-fade latch of the projection emitter.
/// * `m2AC` / `m2B0` (`:47`, `:54`, `:172-190`, `:277`, `:290`, `:310`) are particle emitters.
///
/// What DOES survive is the one behaviour with no event-table input at all, plus the per-frame
/// sound:
///
/// * `check_warp` (`:403-417`) - `dComIfGp_checkPlayerStatus0(0, daPyStts0_SHIP_RIDE_e)` (`:407`)
///   is `player_status::PlayerStatus::check0(STTS0_SHIP_RIDE)`; `dComIfGp_getShipActor()` (`:408`)
///   is the live `Ship` actor, whose name `fpcNm_SHIP_e` is `Actor::name == "Ship"`
///   (`d_stage.cpp:650`, `src/sim/ship.rs:12`). The leading `f32 abs = (player->current.pos -
///   current.pos).absXZ();` (`:405`) is a dead local - it is never tested.
/// * `get_return_count` (`:394-400`) - `dComIfGs_isStageBossEnemy(dSv_save_c::STAGE_FF)` (`:395`,
///   `d_save.h:891`, `STAGE_FF = 0x02`) is `dSv_memBit_c::isStageBossEnemy` =
///   `isDungeonItem(STAGE_BOSS_ENEMY)` (`d_save.h:676`), i.e.
///   `MemBits::is_stage_boss_enemy`; the `dComIfGs_isEventBit(UNK_1001)` test (`:399`) is
///   `EventFlags::test`.
/// * `normal_execute` (`:204-213`) - `if (check_warp() != 0) { if (get_return_count() == 0)
///   { dComIfGs_onEventBit(UNK_3810); } ... }`. `anim_play(2)` (`:211`) is BTK / BRK archive work.
/// * `_execute`'s tail `fopAcM_seStart(this, JA_SE_OBJ_HL_WAPR_EFF, 0)` (`:197`) runs on BOTH
///   branches (`:164` and `:169`), so it is unconditional.
///
/// OMITTED:
/// * `demo_proc` (`:233-251`), `eventOrder` (`:327-332`) and `checkOrder` (`:335-347`) - the stage
///   event-info table. `event_init_tbl` / `event_action_tbl` (`:27-42`) resolve to `initWait`
///   (`:254`), `initWarp` (`:275`), `initWarpArrive` (`:288`), `initWarpArriveEnd` (`:309`),
///   `initStartWarp` (`:264`) and their actions (`:258`, `:269`, `:282`, `:304`, `:317`), all of
///   which are `anim_play` (`:350-380`, BTK / BRK), `dComIfGp_particle_set` (`:276`, `:289-290`),
///   `dComIfGp_evmng_setGoal` (`:265`, `:295`, camera goal) or `mpBrkAnm->getFrame() < 0.5f`
///   (`:320`, BRK archive frames).
/// * `_execute`'s `demoActorID != 0` arm (`:169-195`) - `dDemo_manager_c` /
///   `dComIfGp_demo_getActor` (`:172`, `:217`), `manager->getFrame() >= 0x225u` (`:174`),
///   `dComIfGp_getStartStageName()` (`:178`), `mDoGph_gInf_c::onMonotone` / `offMonotone` (`:176`,
///   `:189`) and `dComIfGp_particle_setProjection` (`:188`).
/// * `daWarphr_IsDelete` (`:491-494`) is the bare `return TRUE` form - RULE 6b, it ACCEPTS a delete
///   request and is not a despawn.
/// * `CreateHeap` (`:68-124`), `m_residxA` / `m_residxB` / `m_heapsize` (`:22-24`), `set_mtx`
///   (`:155-161`), `_draw` (`:420-469`), `_delete` (`:45-60`) and `CheckCreateHeap` (`:63-65`) -
///   two models, two BTKs, one BRK, matrices and renderer work. `_create` (`:140-152`) can only
///   return `cPhs_ERROR_e` from `fopAcM_entrySolidHeap` (`:146-148`), i.e. heap work, so there is
///   no refusal.
/// * `normal_execute`'s `m2F0 = 2` and `checkOrder`'s `m2F0 == 0 && !dComIfGp_event_runCheck()`
///   gate (`:344`). `ActiveEvent` exposes no "is any event running" predicate, so the gate cannot be
///   transcribed; the two surviving effects of `normal_execute` are `check_warp()` - a pure read -
///   and the idempotent `dComIfGs_onEventBit(UNK_3810)`, so running it unconditionally changes
///   neither. `dComIfGp_event_runCheck()` (`d_com_inf_game.h:3478`) is listed under OMITTED rather
///   than replaced.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daWarphr_c` holds two models, two BTK
/// animators, one BRK animator, three matrices and two emitters and registers no `dCcD` collider;
/// the profile `Group` is `fopAc_ACTOR_e` (`:517`). Neither `combat::HitPoints` nor
/// `enemy::Hurt`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WarpRule;

impl WarpRule {
    /// `daWarphr_c::_create` (`.cpp:140-152`), minus `fopAcM_ct` (`:141`),
    /// `daWarphr_prm::getType` (`:143`), `dComIfG_resLoad` (`:144`),
    /// `fopAcM_entrySolidHeap` / `CheckCreateHeap` / `CreateHeap` (`:146-148`) and `CreateInit`
    /// (`:149`). The profile's `Parameters` field is `0` (`:512`).
    pub fn spawn(_parameters: u32) -> Self {
        WarpRule
    }

    /// `check_warp` (`.cpp:403-417`). `dComIfGp_checkPlayerStatus0(0, daPyStts0_SHIP_RIDE_e)`
    /// (`:407`) is `checkPlayerStatus(param_0, 0, flag)` (`d_com_inf_game.h:2875-2877`, `:508`);
    /// `param_0` is the player index, the sim has one player, and
    /// `daPyStts0_SHIP_RIDE_e = 0x00010000` (`d_com_inf_game.h:53`) is
    /// `PlayerStatus::check0(STTS0_SHIP_RIDE)`.
    fn check_warp(status: &PlayerStatus, ship: Option<Vec3>, here: Vec3) -> bool {
        if !status.check0(STTS0_SHIP_RIDE) {
            return false;
        }
        // `daShip_c* ship = dComIfGp_getShipActor(); if (ship != NULL) { ... }` (:408-409)
        let Some(ship) = ship else {
            return false;
        };
        // `f32 abs = (ship->current.pos - current.pos).absXZ(); if (abs < 500.0f) { return TRUE; }`
        // (:410-413)
        xz(ship - here) < SHIP_RADIUS
    }

    /// `get_return_count` (`.cpp:394-400`): `if (!dComIfGs_isStageBossEnemy(STAGE_FF)) { return 0;
    /// } return dComIfGs_isEventBit(UNK_1001) ? 2 : 1;`.
    fn return_count(mem: &MemBits, flags: &EventFlags) -> i32 {
        if !mem.is_stage_boss_enemy() {
            return 0;
        }
        if flags.test(EVT_1001) { 2 } else { 1 }
    }

    /// `normal_execute` (`.cpp:204-213`), minus `anim_play(2)` and `m2F0 = 2`.
    fn normal_execute(
        status: &PlayerStatus,
        ship: Option<Vec3>,
        here: Vec3,
        mem: &MemBits,
        flags: &mut EventFlags,
    ) {
        if Self::check_warp(status, ship, here) && Self::return_count(mem, flags) == 0 {
            flags.set(EVT_3810);
        }
    }

    /// `_execute` (`.cpp:164-201`): both branches are omitted except the shared
    /// `fopAcM_seStart(this, JA_SE_OBJ_HL_WAPR_EFF, 0)` (`:197`).
    fn execute(
        actor: &Actor,
        status: &PlayerStatus,
        ship: Option<Vec3>,
        mem: &MemBits,
        flags: &mut EventFlags,
        events: &mut SimEvents,
    ) {
        Self::normal_execute(status, ship, actor.pos, mem, flags);
        events.list.push(SimEvent::Se(SE_OBJ_HL_WAPR_EFF));
    }

    /// `fopAcM_setCullSizeBox(this, -300.0f, 0.0f, -300.0f, 300.0f, 5000.0f, 300.0f)` (`.cpp:129`).
    /// `fopAcM_setCullSizeFar(this, 1.0f)` (`.cpp:130`) is the far clip; `view::ActorView` has no far
    /// field, so it stays documented.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

/// `cXyz::absXZ()` - `(*other_pos - *this_pos).absXZ()` (`f_op_actor_mng.cpp:514`), which
/// `d_a_warphr.cpp:405` and `:410` inline as `(a - b).absXZ()`.
fn xz(delta: Vec3) -> f32 {
    (delta.x * delta.x + delta.z * delta.z).sqrt()
}

pub fn is_warphrule(name: &str) -> bool {
    WARPHYRULE_ACTORS.contains(&name)
}

/// One system, six params. Access set per param: `PlayerStatus` (R), `MemBits` (R), `EventFlags`
/// (W - `EventFlags::set`), `SimEvents` (W), `Actor` (R - `pos` and `name` only, in both queries),
/// `WarpRule` (R). Both `Query<&Actor>` params read `Actor` and nothing writes it, so there is no
/// intra-system ambiguity.
pub fn control(
    status: Res<PlayerStatus>,
    mem: Res<MemBits>,
    mut flags: ResMut<EventFlags>,
    mut events: ResMut<SimEvents>,
    ships: Query<&Actor>,
    rules: Query<(&Actor, &WarpRule)>,
) {
    let ship = ships
        .iter()
        .find(|actor| actor.name == SHIP_ACTOR)
        .map(|actor| actor.pos);
    for (actor, _rule) in &rules {
        WarpRule::execute(actor, &status, ship, &mem, &mut flags, &mut events);
    }
}
