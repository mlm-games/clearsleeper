use bevy_ecs::prelude::*;
use bevy_ecs::query::With;

use super::actor::Actor;
use super::enemy::Enemy;
use super::event::{ActiveEvent, EventList, Stage};
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, Switches};

pub const HTOBI3_ACTORS: [&str; 1] = ["Htobi3"];

const SWITCH_MASK: u32 = 0xFF; // <- d_a_shutter2.h:52 (daShutter2_prm::getSwitchNo)
const NO_SWITCH: u8 = 0xFF; // <- d_a_shutter2.cpp:52, :53, :165
const DEMO_UP: i32 = 1; // <- d_a_shutter2.cpp:54 (m30C = 1)
const DEMO_DOWN: i32 = 2; // <- d_a_shutter2.cpp:58 (m30C = 2)
const LIFT_HEIGHT: f32 = 350.0; // <- d_a_shutter2.cpp:55, :139, :148
const MAX_VEL: f32 = 3.0; // <- d_a_shutter2.cpp:11 (m_max_speed[0])
const MIN_VEL: f32 = 1.0; // <- d_a_shutter2.cpp:12 (m_min_speed[0])
const OPEN_SCALE: f32 = 0.1; // <- d_a_shutter2.cpp:139, :148 (cLib_addCalc scale)
const STATE_CLOSED: i32 = 0; // <- d_a_shutter2.cpp:164 (mDemoState == 0), :186, :189
const STATE_OPENING: i32 = 1; // <- d_a_shutter2.cpp:171
const STATE_CLOSING: i32 = 2; // <- d_a_shutter2.cpp:178
const EVENT_OPEN: &str = "R04DOOROPEN"; // <- d_a_shutter2.cpp:23 (m_open_ev_name[0])
const EVENT_CLOSE: &str = "R04DOORCLOSE"; // <- d_a_shutter2.cpp:24 (m_close_ev_name[0])
const STAFF_NAME: &str = "Htobi3"; // <- d_a_shutter2.cpp:25 (m_staff_name[0])
const SE_OBJ_WDUN_R04_STR_OP: u32 = 0x69CA; // <- JAZelAudio_SE.h:1978

/// `shutter_move`'s `action_table` (`d_a_shutter2.cpp:108-113`), in the decomp's own order:
/// `ACT_WAIT`, `ACT_OPEN`, `ACT_CLOSE`, `ACT_OPEN_INIT` (`:114-119`).
const ACTIONS: [&str; 4] = ["WAIT", "OPEN", "CLOSE", "OPEN_INIT"];

pub const CULL_MIN: [f32; 3] = [-300.0, -10.0, -40.0]; // <- d_a_shutter2.cpp:19 (m_cull_min[0])
pub const CULL_MAX: [f32; 3] = [300.0, 400.0, 40.0]; // <- d_a_shutter2.cpp:20 (m_cull_max[0])

/// The Cave of Ordeals boss-room shutter, `daShutter2_c` (`d_a_shutter2.h:7-49`). The stage string
/// is `Htobi3` (`d_stage.cpp:959`, `fpcNm_SHUTTER2_e`), which is also the archive
/// (`d_a_shutter2.cpp:22`).
///
/// `Create` (`:45-69`) registers no `dCcD` collider of any kind, so this actor has no inbound
/// damage path and takes no `HitPoints`/`Hurt`.
///
/// Two independent pieces of state are involved and must not be merged: `m30C` (`:308`) is the
/// create-time pose, set to 1 with `current.pos.y = home.pos.y + 350.0f` (`:54-55`) or 2 with no
/// height change (`:57-58`) and never read again; `mDemoState` (`:304`) starts at 0 from `fopAcM_ct`
/// and is the only one `demo` (`:161-205`) reads.
///
/// `fopAcM_myRoomSearchEnemy` (`:51`, `:99`, `:163`) asks whether the room's layer holds any
/// `fopAc_ENEMY_e` actor (`f_op_actor_mng.cpp:1282-1296`), which `super::enemy::Enemy` on a matching
/// `Actor::room` answers.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Htobi3 {
    /// `daShutter2_prm::getSwitchNo` (`d_a_shutter2.h:52`), read every tick by `demo` (`:162`).
    pub switch_no: u8,
    /// `mDemoState` (`d_a_shutter2.h:39`).
    pub demo_state: i32,
    /// `m30C` (`d_a_shutter2.h:41`): 1 opens the shutter at create time, 2 leaves it shut.
    pub pose: i32,
    /// `mbIsSwitch` (`d_a_shutter2.h:47`), the previous tick's switch read (`:60`).
    pub was_switch: bool,
    /// `mbIsNearEnemy` (`d_a_shutter2.h:48`), the previous tick's room-enemy probe (`:51`).
    pub was_near_enemy: bool,
    /// `mOpenEventIdx` / `mCloseEventIdx` (`d_a_shutter2.h:43-44`); `None` is `getEventIdx`'s -1.
    pub open_slot: Option<usize>,
    pub close_slot: Option<usize>,
    /// `home.pos.y`, recovered on the first tick so `shutter_move`'s two `cLib_addCalc` targets
    /// (`:139`, `:148`) have the same absolute reference the decomp uses.
    pub home_y: f32,
    pub entered: bool,
}

impl Htobi3 {
    /// `Create` (`d_a_shutter2.cpp:45-69`) plus `_create`'s `mType = 0` (`:74`), minus
    /// `fopAcM_SetMtx`, `fopAcM_setCullSizeBox` and `set_mtx` (`:61`). The `mOpenEventIdx` and
    /// `mCloseEventIdx` lookups (`:63`, `:66`) are deferred to the first tick because they need the
    /// stage event list; both pass `eventInfoIdx = 0xff` (`d_event_manager.cpp:347`), so neither
    /// takes the stage event-info redirect.
    pub fn spawn(parameters: u32) -> Self {
        let switch_no = (parameters & SWITCH_MASK) as u8;
        let pose = if switch_no != NO_SWITCH {
            DEMO_UP
        } else {
            DEMO_DOWN
        };
        Self {
            switch_no,
            demo_state: STATE_CLOSED,
            pose,
            was_switch: false,
            was_near_enemy: false,
            open_slot: None,
            close_slot: None,
            home_y: 0.0,
            entered: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `Create`'s height of `:55` plus its `mbIsSwitch` (`:60`) and `mbIsNearEnemy` (`:51`) stores.
    /// Nothing can change between Create and the first `Execute`, so reading the live values here
    /// reproduces them exactly.
    fn create_init(
        &mut self,
        actor: &mut Actor,
        list: &EventList,
        switches: &Switches,
        enemies: &[i8],
    ) {
        self.open_slot = list.event_index(EVENT_OPEN);
        self.close_slot = list.event_index(EVENT_CLOSE);
        let lift = if self.pose == DEMO_UP {
            LIFT_HEIGHT
        } else {
            0.0
        };
        self.home_y = actor.pos.y - lift;
        actor.pos.y = self.home_y + lift;
        self.was_switch = switches.test(self.switch_no);
        self.was_near_enemy = Self::near_enemy(actor, enemies);
    }

    /// `fopAcM_orderOtherEventId(this, mOpenEventIdx)` (`:198`) and its `mCloseEventIdx` twin
    /// (`:202`), guarded as in `vfan::Vfan::order`.
    fn order(
        &self,
        slot: Option<usize>,
        list: &EventList,
        active: &mut ActiveEvent,
        entity: Entity,
    ) {
        let Some(slot) = slot else {
            return;
        };
        if active.stage_of(slot).is_none() {
            let _ = active.order(list, slot, entity);
        }
    }

    /// `Execute` (`d_a_shutter2.cpp:95-102`) - `demo` (`:161-205`) followed by the trailing
    /// `mbIsSwitch` and `mbIsNearEnemy` refresh (`:98-99`) - minus `set_mtx` (`:97`) and the `*pMtx`
    /// store (`:100`).
    fn execute(
        &mut self,
        actor: &mut Actor,
        list: &EventList,
        active: &mut ActiveEvent,
        switches: &Switches,
        ev_flags: &mut EventFlags,
        events: &mut SimEvents,
        enemies: &[i8],
        entity: Entity,
    ) {
        self.demo(
            actor, list, active, switches, ev_flags, events, enemies, entity,
        );
        self.was_switch = switches.test(self.switch_no);
        self.was_near_enemy = Self::near_enemy(actor, enemies);
    }

    /// `eventInfo.checkCommandDemoAccrpt()` (`:184`) is `mCommand == dEvtCmd_INDEMO_e`
    /// (`f_op_actor.h:177`). `dEvt_control_c::demoCheck` sets that word on both ordered actors when
    /// the event starts (`d_event.cpp:314-316`) and `demoEnd` clears it (`d_event.cpp:331-337`), so
    /// it is a latch that stays true through the event's CLOSE stage - which is what keeps
    /// `shutter_move` running past `start_check`. In the sim that is exactly "one of the two events
    /// this actor ordered is in `Stage::Play` or `Stage::Close`", the two states
    /// `dEvent_manager_c::getMyStaffId` scans (`d_event_manager.cpp:412`).
    fn demo_running(&self, active: &ActiveEvent) -> bool {
        [self.open_slot, self.close_slot]
            .into_iter()
            .flatten()
            .any(|slot| {
                matches!(
                    active.stage_of(slot),
                    Some(Stage::Play) | Some(Stage::Close)
                )
            })
    }

    /// `demo` (`d_a_shutter2.cpp:161-205`). The two `eventInfo.onCondition(dEvtCnd_UNK2_e)` calls
    /// (`:199`, `:203`) are dropped: nothing in the decomp ever calls `chkCondition` with
    /// `dEvtCnd_UNK2_e` - `commonCheck` passes only CANTALK / CANDOOR / CANGETITEM / CANCATCH
    /// (`d_event.cpp:145`, `:191`, `:233`, `:365`, `:390`) and `check_event_condition` tests only
    /// CANTALK and CANDOOR (`d_attention.cpp:229-249`) - and `beforeProc` clears `mCondition` on
    /// every `fopAc_Execute` (`f_op_actor.cpp:176`).
    fn demo(
        &mut self,
        actor: &mut Actor,
        list: &EventList,
        active: &mut ActiveEvent,
        switches: &Switches,
        ev_flags: &mut EventFlags,
        events: &mut SimEvents,
        enemies: &[i8],
        entity: Entity,
    ) {
        let is_switch = switches.test(self.switch_no);
        let is_near_enemy = Self::near_enemy(actor, enemies);
        if self.demo_state == STATE_CLOSED {
            if self.switch_no != NO_SWITCH {
                if is_switch != self.was_switch {
                    self.demo_state = if is_switch {
                        STATE_OPENING
                    } else {
                        STATE_CLOSING
                    };
                }
            } else if is_near_enemy != self.was_near_enemy {
                self.demo_state = if is_near_enemy {
                    STATE_OPENING
                } else {
                    STATE_CLOSING
                };
            }
        }

        if !self.demo_running(active) {
            if self.demo_state == STATE_OPENING && self.open_slot.is_some() {
                self.order(self.open_slot, list, active, entity);
            } else if self.demo_state == STATE_CLOSING && self.close_slot.is_some() {
                self.order(self.close_slot, list, active, entity);
            }
            return;
        }

        if self.open_slot.is_some_and(|slot| active.start_check(slot))
            && self.demo_state == STATE_OPENING
        {
            self.demo_state = STATE_CLOSED;
        }
        if self.close_slot.is_some_and(|slot| active.start_check(slot))
            && self.demo_state == STATE_CLOSING
        {
            self.demo_state = STATE_CLOSED;
        }
        if let Some(slot) = self
            .open_slot
            .filter(|slot| active.end_check(*slot))
            .or_else(|| self.close_slot.filter(|slot| active.end_check(*slot)))
        {
            // dComIfGp_event_reset (d_event.h:131) sets mEventFlag bit 8, which
            // dEvt_control_c::check (d_event.cpp:546-554) turns into mbEndProc and then endProc ->
            // demoEnd, i.e. it ends the running demo. It does NOT clear dSv_event_flag_c, so
            // nothing is written to `ev_flags` here beyond what shutter_move's cutEnd calls do.
            let _ = active.end(slot);
        }
        self.shutter_move(actor, list, active, ev_flags, events);
    }

    /// `shutter_move` (`d_a_shutter2.cpp:105-158`).
    ///
    /// `dComIfGp_evmng_getMyActIdx(mStaffId, action_table, 4, FALSE, 0)` (`:120`) returns -1 for a
    /// -1 staff id and otherwise latches `staff->mCurActionIdx` once `mbHasAction` is set
    /// (`d_event_manager.cpp:456-488`); that latch is cleared only by `dEvDtStaff_c::init` and
    /// `advanceCut` (`d_event_data.cpp:179`, `:193`), i.e. exactly when `run.cut` changes. Resolving
    /// the cut name afresh each tick is therefore equivalent to the latch.
    ///
    /// `dComIfGp_evmng_cutEnd(mStaffId)` (`:128`, `:134`, `:141`, `:150`, `:155`) is
    /// `dEvent_manager_c::cutEnd` (`d_event_manager.cpp:549-555`), which sets the current cut's
    /// flag id - `ActiveEvent::end_staff_cut`.
    fn shutter_move(
        &mut self,
        actor: &mut Actor,
        list: &EventList,
        active: &mut ActiveEvent,
        ev_flags: &mut EventFlags,
        events: &mut SimEvents,
    ) {
        // dComIfGp_evmng_getMyStaffId("Htobi3", NULL, 0) (d_a_shutter2.cpp:194) returns -1 when no
        // running event carries the staff, and getMyActIdx(-1, ...) then returns -1 so only the
        // default arm runs - and cutEnd(-1) itself bails (d_event_manager.cpp:550-551).
        let Some((cut, _)) = active.staff_cut(list, STAFF_NAME) else {
            return;
        };
        let action = list
            .cut_name(cut)
            .and_then(|name| ACTIONS.iter().position(|entry| *entry == name));
        match action {
            Some(0) => active.end_staff_cut(list, ev_flags, STAFF_NAME),
            Some(1) => {
                let open = self.home_y + LIFT_HEIGHT;
                if add_calc(&mut actor.pos.y, open, OPEN_SCALE, MAX_VEL, MIN_VEL) == 0.0 {
                    active.end_staff_cut(list, ev_flags, STAFF_NAME);
                }
            }
            Some(2) => {
                if add_calc(&mut actor.pos.y, self.home_y, OPEN_SCALE, MAX_VEL, MIN_VEL) == 0.0 {
                    active.end_staff_cut(list, ev_flags, STAFF_NAME);
                }
            }
            Some(3) => {
                events.list.push(SimEvent::Se(SE_OBJ_WDUN_R04_STR_OP));
                active.end_staff_cut(list, ev_flags, STAFF_NAME);
            }
            _ => active.end_staff_cut(list, ev_flags, STAFF_NAME),
        }
    }

    /// `fopAcM_myRoomSearchEnemy(fopAcM_GetRoomNo(this)) == NULL` (`:99`, `:163`): the room holds no
    /// `fopAc_ENEMY_e` actor (`f_op_actor_mng.cpp:1282-1296`).
    fn near_enemy(actor: &Actor, enemies: &[i8]) -> bool {
        enemies.contains(&actor.room)
    }
}

/// `cLib_addCalc` (`c_lib.cpp:22-53`) with its `|target - *pValue|` return, which `shutter_move`
/// tests against `0.0f` (`d_a_shutter2.cpp:140`, `:149`). `walk::add_calc` is the same integrator
/// but returns nothing.
fn add_calc(value: &mut f32, target: f32, scale: f32, max_step: f32, min_step: f32) -> f32 {
    if *value != target {
        let mut step = scale * (target - *value);
        if step >= min_step || step <= -min_step {
            step = step.clamp(-max_step, max_step);
            *value += step;
        } else if step > 0.0 {
            *value += min_step;
            if *value > target {
                *value = target;
            }
        } else {
            *value -= min_step;
            if *value < target {
                *value = target;
            }
        }
    }
    (target - *value).abs()
}

pub fn is_htobi3(name: &str) -> bool {
    HTOBI3_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    switches: Res<Switches>,
    mut ev_flags: ResMut<EventFlags>,
    mut events: ResMut<SimEvents>,
    foes: Query<&Actor, (With<Enemy>, Without<Htobi3>)>,
    mut shutters: Query<(Entity, &mut Actor, &mut Htobi3)>,
) {
    let mut enemies: Vec<i8> = Vec::new();
    for actor in &foes {
        if !enemies.contains(&actor.room) {
            enemies.push(actor.room);
        }
    }
    for (entity, mut actor, mut shutter) in &mut shutters {
        let actor = &mut *actor;
        if !shutter.entered {
            shutter.entered = true;
            shutter.create_init(actor, &list, &switches, &enemies);
        }
        shutter.execute(
            actor,
            &list,
            &mut active,
            &switches,
            &mut ev_flags,
            &mut events,
            &enemies,
            entity,
        );
    }
}
