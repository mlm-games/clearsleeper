//! `daPedestal::daPds_c` (`d_a_pedestal.h:31-87`) - the Wind Temple mine-cart pedestal.
//! `Nonmatching` count: 0, so every token below is transcribed.
//!
//! Nothing in `src/sim/` claims `Hdai1` / `Hdai2` / `Hdai3` as an actor name. Two references exist
//! and neither is a spawn claim: `src/sim/servant.rs`'s private
//! `DAIZA_NAMES = ["Hdai1", "Hdai2", "Hdai3"]` (`:42`), the name its `daiza_name()` (`:445`)
//! matches against, and `src/sim/event.rs:903`'s staff-name dispatch
//! `staff.name == "Hdai1" || ...`, which drives those staffs' cut lifecycle centrally. Both are
//! intended interop, not duplicate claims.

use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::EventFlags;
use super::servant::Servant;

/// `OBJNAME("Hdai1", fpcNm_PEDESTAL_e, 0, 61)` (`d_stage.cpp:1023`),
/// `OBJNAME("Hdai2", fpcNm_PEDESTAL_e, 1, 61)` (`:1024`) and
/// `OBJNAME("Hdai3", fpcNm_PEDESTAL_e, 2, 61)` (`:1025`). `fpcNm_PEDESTAL_e` is
/// `d_a_pedestal.cpp`; the archive `daPds_c::m_arcname[] = "Hdai1"` (`:13`) matches only the first,
/// which is why the array carries all three stage strings and not the archive name.
pub const PEDESTAL_ACTORS: [&str; 3] = ["Hdai1", "Hdai2", "Hdai3"];

/// The `s8 argument` column of the same three OBJNAME rows (`dStage_objectNameInf::argument`,
/// `d_stage.h:992`), assigned by `dStage_actorCreate` as `i_actorPrm->argument =
/// nameinf_p->argument` (`d_stage.cpp:1306`). `src/dzs.rs`'s `StageActor` (`:73-81`) carries no
/// argument field, so it is recovered from the stage string - the same trick
/// `super::servant::Servant::argument_of` uses (`src/sim/servant.rs:433-439`). The mapping is not
/// a guess: it IS the table.
const DAIZA_ARGUMENT: [(&str, u8); 3] = [
    ("Hdai1", 0), // <- d_stage.cpp:1023
    ("Hdai2", 1), // <- d_stage.cpp:1024
    ("Hdai3", 2), // <- d_stage.cpp:1025
];

/// `fopAcM_setCullSizeBox(this, -150.0f, -20.0f, -150.0f, 150.0f, 250.0f, 150.0f)`
/// (`d_a_pedestal.cpp:70`) - the actor's one real custom cull box, matching the profile's
/// `fopAc_CULLBOX_CUSTOM_e` (`:489`).
const CULL_MIN: [f32; 3] = [-150.0, -20.0, -150.0]; // <- d_a_pedestal.cpp:70
const CULL_MAX: [f32; 3] = [150.0, 250.0, 150.0]; // <- d_a_pedestal.cpp:70

/// `mType = fopAcM_GetParam(this) & 0xFF` (`d_a_pedestal.cpp:67`) - the whole low byte of the
/// parameter word. The profile's `Parameters` field is `0` (`:483`), so `daPds_c` nevertheless
/// reads one.
const TYPE_MASK: u32 = 0xFF; // <- d_a_pedestal.cpp:67

/// `if (mType == 0)` (`:72`) - "a pedestal that a npc_os starts on in its original room" (`:73`).
const TYPE_START: u8 = 0; // <- d_a_pedestal.cpp:72
/// `else if (mType == 1 && finishCheck())` (`:85`) - "a pedestal that a npc_os ends on in the
/// central room" (`:86`).
const TYPE_END: u8 = 1; // <- d_a_pedestal.cpp:85

/// `current.pos.y -= 240.0f` (`:83`) on the wake-up path and `current.pos.y += 240.0f` (`:87`) on
/// the finish path - the same 240 units, opposite signs.
const CART_DROP: f32 = 240.0; // <- d_a_pedestal.cpp:83

/// `static char* l_os_name[] = { "Os", "Os1", "Os2" }` (`:61-65`), indexed by `argument`.
const OS_NAMES: [&str; 3] = ["Os", "Os1", "Os2"]; // <- d_a_pedestal.cpp:61-65
/// `if (pActor != NULL && fopAcM_searchActorDistanceXZ(this, pActor) < 100.0f)` (`:78`).
/// `fopAcM_searchActorDistanceXZ` is `(*other_pos - *this_pos).absXZ()`
/// (`f_op_actor_mng.cpp:511-515`) - an XZ distance, not a squared one.
const OS_FOLLOW_XZ: f32 = 100.0; // <- d_a_pedestal.cpp:78

/// `wakeupCheck` (`:127-143`), one `dSv_event_flag_c` id per `argument`. Retail ids pass straight
/// through `super::flags::EventFlags`, which is byte-indexed (RULE 11c); all four nibbles sit
/// below `UNK_FF07`, the top of `d_save_event_flag.inc`.
const WAKEUP_FLAGS: [u32; 3] = [
    0x1780, // <- d_a_pedestal.cpp:129, d_save_event_flag.inc:174
    0x1740, // <- d_a_pedestal.cpp:133, d_save_event_flag.inc:175
    0x1720, // <- d_a_pedestal.cpp:137, d_save_event_flag.inc:176
];
/// `finishCheck` (`:146-162`).
const FINISH_FLAGS: [u32; 3] = [
    0x1710, // <- d_a_pedestal.cpp:148, d_save_event_flag.inc:177
    0x1704, // <- d_a_pedestal.cpp:152, d_save_event_flag.inc:179
    0x1B01, // <- d_a_pedestal.cpp:156, d_save_event_flag.inc:209
];

/// `static char* cut_name_tbl[] = { "WAIT", "MOVE", "EFFSET", "EFFEND" }` (`:217-222`), whose order
/// is the `event_init_tbl` / `event_action_tbl` index (`:202-215`).
const CUT_NAMES: [&str; 4] = ["WAIT", "MOVE", "EFFSET", "EFFEND"]; // <- d_a_pedestal.cpp:217-222
const CUT_MOVE: usize = 1; // <- d_a_pedestal.cpp:221

/// `speed.y = 1.0f` (`:266`) - the default when the cut carries no `Speed` float.
const DEFAULT_SPEED: f32 = 1.0; // <- d_a_pedestal.cpp:266

/// `fopAcM_seStartCurrent(this, JA_SE_OBJ_ST_DAI_STOP, 0)` (`:301`) when the pedestal reaches its
/// target, and `JA_SE_OBJ_ST_DAI_MOVE` (`:303`) every tick while it is still travelling.
const SE_ST_DAI_STOP: u32 = 0x69D9; // <- d_a_pedestal.cpp:301, JAZelAudio_SE.h:1986
const SE_ST_DAI_MOVE: u32 = 0x61D8; // <- d_a_pedestal.cpp:303, JAZelAudio_SE.h:1708

/// `daPedestal::daPds_c` - the Wind Temple pedestal that a servant's cart starts from and finishes
/// on.
///
/// The struct is the live subset of `daPds_c` (`d_a_pedestal.h:31-87`) - `mType` (`h:77`),
/// `mTargetYPos` (`h:74`) and `speed` are kept. Create-time work that survives:
///
/// * `mType = fopAcM_GetParam(this) & 0xFF` (`:67`).
/// * `mType == 0 && wakeupCheck()` (`:72-84`): the matching servant is found by name, moved to the
///   pedestal's height when it is within [`OS_FOLLOW_XZ`] in XZ, and the pedestal itself drops by
///   [`CART_DROP`].
/// * `mType == 1 && finishCheck()` (`:85-88`): the pedestal rises by [`CART_DROP`].
///
/// The event side:
///
/// * `eventProc` (`:225-249`) reads the running cut of the staff named by `getMyStaffId`
///   (`:114-124`), runs the matching `event_init_tbl` entry on the tick the cut advances and the
///   matching `event_action_tbl` entry every tick, and calls `dComIfGp_evmng_cutEnd(staffIdx)` -
///   `dEvent_manager_c::cutEnd`, `d_event_manager.cpp:549-555`, which sets the current cut's flag id -
///   when the action returns TRUE, and also when the live action is not in `cut_name_tbl`
///   (`:232-233`). That is `ActiveEvent::staff_cut` + `EventList::cut_name` +
///   `ActiveEvent::end_staff_cut`, the same mapping `src/sim/htobi3.rs:255-277` uses for
///   `d_a_shutter2.cpp`, and it composes with `src/sim/event.rs:903`'s central
///   `hold_cut(run, list, flags, &["MOVE"])` for these three staffs instead of duplicating it.
/// * `dComIfGp_evmng_getMyFloatP(staffIdx, "Speed")` / `"Dist"` (`:261`, `:269`) read the STAFF's
///   CURRENT cut's data records - `getMyDataP(staffIdx, name, 0)` walks
///   `getCutStaffCurrentCutP(staffIdx)` (`d_event_manager.cpp:511-530`) - which is exactly
///   `EventList::cut_float(cut, key)`.
///
/// OMITTED:
///
/// * `CreateHeap` (`:33-57`) and `_create`'s `dComIfG_resLoad` / `fopAcM_entrySolidHeap` /
///   `CheckCreateHeap` (`:99-108`) - model, BRK and `dBgW` archive work. `phase_state` can only
///   become `cPhs_ERROR_e` from `fopAcM_entrySolidHeap` (`:102-105`), i.e. heap work, so there is
///   no refusal.
/// * `initBrkAnm` (`:331-363`), `playBrkAnm` (`:366-369`) and `_draw`'s `mBrk.entry` /
///   `mBrk.remove` (`:398`, `:400`) - BRK frames are archive data (RULE 3b). `brkAnmTbl` (`:338-343`)
///   and the `unk_4 < 0` end-frame snap at `:353-355` go with them; no frame count is invented.
/// * `initialEffectSet` (`:310-313`) and `initialEffectEnd` (`:316-319`) - `makeEmitter` / `.end()`
///   particle work plus `initBrkAnm`. They leave no portable state, so the `actionDefault` entry
///   they share in `event_action_tbl` (`:255-257`) still ends their cut.
/// * `daPds_infiniteEcallBack_c` in full (`:433-463`) - particle callback / emitter lifetime.
/// * `daPds_IsDelete` (`:427-430`) is the bare `return TRUE` form - RULE 6b, it ACCEPTS a delete
///   request and is not a despawn.
/// * `set_mtx` (`:322-328`), `_draw` (`:390-405`), `mpBgW->Move()` (`:92`, `:380`) and every
///   `mOctagonGlowCb` access - matrix, renderer and `dBgW` work. That last one is why
///   `if (mOctagonGlowCb.getEmitter() != NULL) { fopAcM_seStartCurrent(this,
///   JA_SE_OBJ_ST_DAI_LIGHT, 0); }` (`:382-384`, `JAZelAudio_SE.h:1707`) cannot fire: the emitter is
///   only ever made by `initialEffectSet` (`:311`), which is omitted, so no id is stored for it.
/// * `dComIfGp_event_runCheck()` (`:228`) as a global "is any event running" predicate -
///   `ActiveEvent` has no such method. `staff_cut` returning `None` (no running event carries this
///   pedestal's staff) is the point at which retail returns FALSE from `eventProc` and calls
///   `action(NULL)` instead, and `action`'s only reachable arm is `waitAction` (`:193-199`), which
///   advances `mActionStatus` and returns TRUE. Both paths are no-ops for the pedestal, so the
///   gate is exact rather than approximated.
/// * `mAction` (`h:73`) / `mActionStatus` (`h:76`) and the `unk308` / `unk30E` / `unk30F` /
///   `unk310` / `unk312` / `unk314` / `unk316` / `unk31C` words (`h:75`, `h:78-85`) - `setAction` (`:165-181`) is only ever called from `action()`
///   (`:186`) when `mAction == NULL`, so after the first idle tick it is always `waitAction`, and
///   with no reader left outside the omitted arm neither field is stored.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daPds_c` holds a model, a `dBgW`, a BRK
/// and an emitter callback and registers no `dCcD` collider; the profile `Group` is
/// `fopAc_ACTOR_e` (`:488`). Neither `combat::HitPoints` nor `enemy::Hurt`.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Pedestal {
    /// `mType` (`d_a_pedestal.cpp:67`).
    pub kind: u8,
    /// The `s8 argument` from the OBJNAME row (`:1023-1025`), which selects the servant name and
    /// the wake-up / finish flag.
    pub argument: u8,
    /// `CreateInit` runs once, inside `_create` (`:107`), but it needs the event-bit resource and
    /// a query over the servants, so it is deferred to this component's first tick - the
    /// `src/sim/htobi3.rs` / `src/sim/auzu.rs` shape.
    pub entered: bool,
    /// `speed.y` (`:264`, `:286`, `:292`) - the cut's vertical rate.
    pub speed: f32,
    /// `mTargetYPos` (`:271`, `:273`) - where the cut wants the pedestal to stop.
    pub target_y: f32,
}

impl Pedestal {
    /// `daPds_c::CreateInit` (`:60-93`), minus `fopAcM_SetMtx` (`:69`),
    /// `fopAcM_setCullSizeBox` (`:70`), `set_mtx` (`:90`), `dComIfG_Bgsp()->Regist` (`:91`) and
    /// `mpBgW->Move()` (`:92`).
    ///
    /// `getMyStaffId` (`:114-124`) returns `dComIfGp_evmng_getMyStaffId("Hdai1" / "Hdai2" /
    /// "Hdai3")` by `argument` and `-1` otherwise; `ActiveEvent::staff_cut` looks the name up in
    /// the parsed `EventList` and returns `None` when no running event carries it, which is the
    /// same -1.
    pub fn spawn(parameters: u32, name: &str) -> Self {
        Self {
            kind: (parameters & TYPE_MASK) as u8,
            argument: argument_of(name),
            entered: false,
            speed: 0.0,
            target_y: 0.0,
        }
    }

    fn staff_name(&self) -> &'static str {
        match self.argument {
            1 => "Hdai2",
            2 => "Hdai3",
            _ => "Hdai1",
        }
    }

    /// `wakeupCheck` (`:127-143`): `dComIfGs_isEventBit(WAKEUP_FLAGS[argument])`, with an unknown
    /// argument falling through to the trailing `return FALSE` (`:142`).
    fn wakeup_check(&self, flags: &EventFlags) -> bool {
        match self.argument {
            0 => flags.test(WAKEUP_FLAGS[0]),
            1 => flags.test(WAKEUP_FLAGS[1]),
            2 => flags.test(WAKEUP_FLAGS[2]),
            _ => false,
        }
    }

    /// `finishCheck` (`:146-162`).
    fn finish_check(&self, flags: &EventFlags) -> bool {
        match self.argument {
            0 => flags.test(FINISH_FLAGS[0]),
            1 => flags.test(FINISH_FLAGS[1]),
            2 => flags.test(FINISH_FLAGS[2]),
            _ => false,
        }
    }

    /// `CreateInit` (`:60-93`). `fopAcM_searchFromName(l_os_name[argument], 0, 0)` (`:76`) matches
    /// on NAME alone - the two trailing arguments are the parameter mask and the parameter value,
    /// both zero - and returns the first such actor. Retail's order is the process-list order;
    /// ECS gives no such guarantee, but the three servant names are one actor each in the only
    /// stage that places them, so the first match is the only match.
    fn create_init(
        &mut self,
        actor: &mut Actor,
        flags: &EventFlags,
        servants: &mut Query<&mut Actor, (With<Servant>, Without<Pedestal>)>,
    ) {
        self.entered = true;
        if self.kind == TYPE_START {
            if self.wakeup_check(flags) {
                if let Some(slot) = OS_NAMES.get(self.argument as usize)
                    && let Some(mut servant) = servants.iter_mut().find(|s| s.name == *slot)
                {
                    let delta = servant.pos - actor.pos;
                    if (delta.x * delta.x + delta.z * delta.z).sqrt() < OS_FOLLOW_XZ {
                        servant.pos.y = actor.pos.y;
                    }
                }
                actor.pos.y -= CART_DROP;
            }
        } else if self.kind == TYPE_END && self.finish_check(flags) {
            actor.pos.y += CART_DROP;
        }
    }

    /// `initialMoveEvent` (`:260-275`). `dComIfGp_evmng_getMyFloatP` returns NULL when the cut
    /// carries no such record, and both defaults are spelled out at `:266` and `:273`.
    fn initial_move(&mut self, actor: &Actor, list: &EventList, cut: usize) {
        self.speed = list.cut_float(cut, "Speed").unwrap_or(DEFAULT_SPEED);
        self.target_y = actor.pos.y + list.cut_float(cut, "Dist").unwrap_or(0.0);
    }

    /// `event_action_tbl[1]` = `actionMoveEvent` (`:278-307`), returning its `ret`. The third arm at
    /// `:295-298` is `speed.y == 0.0f`: snap to the target and finish. The sound at `:300-304` is
    /// `JA_SE_OBJ_ST_DAI_STOP` when it finished and `JA_SE_OBJ_ST_DAI_MOVE` otherwise.
    fn action_move(&mut self, actor: &mut Actor, events: &mut SimEvents) -> bool {
        actor.pos.y += self.speed;
        let done = if self.speed < 0.0 {
            if actor.pos.y <= self.target_y {
                actor.pos.y = self.target_y;
                self.speed = 0.0;
                true
            } else {
                false
            }
        } else if self.speed > 0.0 {
            if actor.pos.y >= self.target_y {
                actor.pos.y = self.target_y;
                self.speed = 0.0;
                true
            } else {
                false
            }
        } else {
            actor.pos.y = self.target_y;
            true
        };
        events.list.push(SimEvent::Se(if done {
            SE_ST_DAI_STOP
        } else {
            SE_ST_DAI_MOVE
        }));
        done
    }

    /// `eventProc` (`:225-249`) with `staffIdx` from `staff_cut`.
    fn event_proc(
        &mut self,
        actor: &mut Actor,
        list: &EventList,
        active: &mut ActiveEvent,
        flags: &mut EventFlags,
        events: &mut SimEvents,
    ) {
        let staff = self.staff_name();
        let Some((cut, advance)) = active.staff_cut(list, staff) else {
            return;
        };
        let Some(action) = list
            .cut_name(cut)
            .and_then(|name| CUT_NAMES.iter().position(|entry| *entry == name))
        else {
            // `if (actIdx == -1) { dComIfGp_evmng_cutEnd(staffIdx); }` (:232-233)
            active.end_staff_cut(list, flags, staff);
            return;
        };
        // `if (dComIfGp_evmng_getIsAddvance(staffIdx)) { (this->*event_init_tbl[actIdx])(staffIdx); }`
        // (:235-237). Only `initialMoveEvent` (index 1) changes state; `initialDefault` (:252),
        // `initialEffectSet` (:310) and `initialEffectEnd` (:316) are omitted above.
        if advance && action == CUT_MOVE {
            self.initial_move(actor, list, cut);
        }
        // `if ((this->*event_action_tbl[actIdx])(staffIdx)) { dComIfGp_evmng_cutEnd(staffIdx); }`
        // (:239-241). `event_action_tbl` (:210-215) is `[actionDefault, actionMoveEvent,
        // actionDefault, actionDefault]`, so indices 0, 2 and 3 return TRUE immediately and the
        // omitted effect entries still end their cut.
        let done = if action == CUT_MOVE {
            self.action_move(actor, events)
        } else {
            true
        };
        if done {
            active.end_staff_cut(list, flags, staff);
        }
    }

    /// `_execute` (`:372-387`): `playBrkAnm()` (`:373`) and `set_mtx()` / `mpBgW->Move()`
    /// (`:379-380`) are omitted, `if (!eventProc()) action(NULL)` (`:375-377`) reduces to
    /// [`Self::event_proc`] because the `else` arm is a no-op (documented above), and the
    /// emitter-gated sound (`:382-384`) cannot fire.
    fn execute(
        &mut self,
        actor: &mut Actor,
        list: &EventList,
        active: &mut ActiveEvent,
        flags: &mut EventFlags,
        events: &mut SimEvents,
    ) {
        self.event_proc(actor, list, active, flags, events);
    }

    /// `fopAcM_setCullSizeBox(this, -150.0f, -20.0f, -150.0f, 150.0f, 250.0f, 150.0f)` (`:70`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

/// The `s8 argument` of the OBJNAME row whose `name` is this actor's stage string
/// (`d_stage.cpp:1023-1025`). `argument` is only ever one of 0 / 1 / 2 for a `PEDESTAL_ACTORS`
/// string, and `0` is also the value `wakeupCheck` / `finishCheck` / `getMyStaffId` reach for an
/// argument they do not list, so the `0` default is the same branch retail takes.
fn argument_of(name: &str) -> u8 {
    DAIZA_ARGUMENT
        .iter()
        .find_map(|(stage, argument)| (*stage == name).then_some(*argument))
        .unwrap_or(0)
}

pub fn is_pedestal(name: &str) -> bool {
    PEDESTAL_ACTORS.contains(&name)
}

/// One system, six params. Access set per param: `EventList` (R), `ActiveEvent` (W), `EventFlags`
/// (W - `end_staff_cut` sets the cut's flag id), `SimEvents` (W), `Actor` (W, but the two queries
/// are provably disjoint: `(With<Servant>, Without<Pedestal>)` against `With<Pedestal>`), `Servant`
/// (a filter only), `Pedestal` (RW). No unproven `(T, T)` pair, so there is no intra-system
/// ambiguity.
pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut flags: ResMut<EventFlags>,
    mut events: ResMut<SimEvents>,
    mut servants: Query<&mut Actor, (With<Servant>, Without<Pedestal>)>,
    mut pedestals: Query<(&mut Actor, &mut Pedestal)>,
) {
    for (mut actor, mut pedestal) in &mut pedestals {
        let actor = &mut *actor;
        let pedestal = &mut *pedestal;
        if !pedestal.entered {
            pedestal.create_init(actor, &*flags, &mut servants);
        }
        pedestal.execute(actor, &*list, &mut *active, &mut *flags, &mut *events);
    }
}
