use bevy_ecs::prelude::*;

use super::event::{ActiveEvent, EventList, Stage};
use super::flags::{EventFlags, Switches};
use super::walk;

/// `OBJNAME("Htobi1", fpcNm_SHUTTER_e, 255, 0)` and `OBJNAME("Htobi2", fpcNm_SHUTTER_e, 255, 0)`
/// (`d_stage.cpp:957-958`) - `fpcNm_SHUTTER_e` is `d_a_shutter.cpp`, NOT `d_a_shutter2.cpp` (which
/// `src/sim/htobi3.rs` owns as `Htobi3`). The strings happen to equal `m_arcname` (`:23`), unlike
/// `Lwood`/`lwood` and `Oyashi`/`Oyashi`, so no lowercase/uppercase trap applies here.
///
/// `src/sim/door.rs`'s `SHUTTER_ACTORS` covers `door10`, `door11`, `door20`, `door21`, `Zenshut`,
/// `keyshut` and `K_Zshut` (`d_stage.cpp:451-457`) - `fpcNm_DOOR10_e`, a different translation
/// unit. Neither string is claimed by anything else in the crate.
pub const HTOBI_ACTORS: [&str; 2] = ["Htobi1", "Htobi2"];

/// `m_max_speed[2] = {3.0f, 3.0f}` (`d_a_shutter.cpp:12`) - `shutter_move`'s `maxVel` (`:162`),
/// the `maxStep` of all four `cLib_addCalc` calls (`:193-194`, `:213-214`).
const MAX_SPEED: [f32; 2] = [3.0, 3.0]; // <- d_a_shutter.cpp:12
/// `m_min_speed[2] = {1.0f, 1.0f}` (`d_a_shutter.cpp:13`) - `minVel` (`:163`), the `minStep`.
const MIN_SPEED: [f32; 2] = [1.0, 1.0]; // <- d_a_shutter.cpp:13
/// `m_move_len[2] = {200.0f, 84.0f}` (`d_a_shutter.cpp:14`) - how far each leaf retracts past its
/// shut position, and the `ACT_OPEN` target (`:193-194`).
const MOVE_LEN: [f32; 2] = [200.0, 84.0]; // <- d_a_shutter.cpp:14
/// `m_width[2] = {220.0f, 82.0f}` (`d_a_shutter.cpp:15`) - half of it is each leaf's shut offset
/// (`:77-78`) and the `ACT_OPEN` / `ACT_CLOSE` targets (`:193-194`, `:213-214`).
const WIDTH: [f32; 2] = [220.0, 82.0]; // <- d_a_shutter.cpp:15
/// `m_arcname[2] = {"Htobi1", "Htobi2"}` (`d_a_shutter.cpp:23`) - the archive names, which the
/// stage strings match, and `m_staff_name[2]`'s values (`:26`).
const STAFF: [&str; 2] = ["Htobi1", "Htobi2"]; // <- d_a_shutter.cpp:26 (m_staff_name)
/// `m_open_ev_name[2] = {"HYSDOOROPEN", "R03DOOROPEN"}` (`d_a_shutter.cpp:24`).
const OPEN_EVENT: [&str; 2] = ["HYSDOOROPEN", "R03DOOROPEN"]; // <- d_a_shutter.cpp:24
/// `m_close_ev_name[2] = {NULL, "R03DOORCLOSE"}` (`d_a_shutter.cpp:25`). The first slot is NULL,
/// so `Create`'s `if (m_close_ev_name[mType] != NULL)` (`:96`) skips the lookup and `mCloseEventIdx`
/// keeps its `fopAcM_ct` zero-fill. `Option` carries that NULL as `None`; a name that IS present but
/// absent from the stage list is also `None`, which is the same outcome because
/// `orderOtherEventId(-1)` and a missing slot both fail to start an event.
const CLOSE_EVENT: [Option<&str>; 2] = [None, Some("R03DOORCLOSE")]; // <- d_a_shutter.cpp:25

/// `m_cull_min[2] = {{-500.0f, -100.0f, -50.0f}, {-150.0f, -100.0f, -50.0f}}`
/// (`d_a_shutter.cpp:20`) - `Create`'s `Vec cullMin = m_cull_min[mType]` (`:73`).
const CULL_MIN: [[f32; 3]; 2] = [[-500.0, -100.0, -50.0], [-150.0, -100.0, -50.0]]; // <- d_a_shutter.cpp:20
/// `m_cull_max[2] = {{500.0f, 250.0f, 50.0f}, {150.0f, 250.0f, 50.0f}}` (`d_a_shutter.cpp:21`).
const CULL_MAX: [[f32; 3]; 2] = [[500.0, 250.0, 50.0], [150.0, 250.0, 50.0]]; // <- d_a_shutter.cpp:21

/// `mFrameTimer = 30` (`d_a_shutter.cpp:87`) - the `s8` (`d_a_shutter.h:52`) that
/// `_execute` drains (`:134-136`) and that `demo` requires to be NEGATIVE before it latches a
/// switch change (`:236`).
const FRAME_TIMER: i8 = 30; // <- d_a_shutter.cpp:87
/// `mTimer = 0xf` (`d_a_shutter.cpp:171`) - the `u8` (`d_a_shutter.h:53`) that `ACT_WAIT` arms and
/// `ACT_WAIT02` drains with `cLib_calcTimer` (`:178`).
const WAIT_TIMER: u8 = 0x0F; // <- d_a_shutter.cpp:171

/// `enum { ACT_WAIT, ACT_WAIT02, ACT_OPEN, ACT_CLOSE }` (`d_a_shutter.cpp:154-159`) - the
/// `getMyActIdx` return values, in the decomp's own order.
const ACT_WAIT: usize = 0; // <- d_a_shutter.cpp:155
const ACT_WAIT02: usize = 1; // <- d_a_shutter.cpp:156
const ACT_OPEN: usize = 2; // <- d_a_shutter.cpp:157
const ACT_CLOSE: usize = 3; // <- d_a_shutter.cpp:158

/// `shutter_move`'s `action_table` (`d_a_shutter.cpp:148-153`).
const ACTIONS: [&str; 4] = ["WAIT", "WAIT02", "OPEN", "CLOSE"]; // <- d_a_shutter.cpp:148-153
/// The `0.1f` scale of all four `cLib_addCalc` calls (`d_a_shutter.cpp:193-194`, `:213-214`).
const ADD_SCALE: f32 = 0.1; // <- d_a_shutter.cpp:193

/// `field_0x320 = 0` - the `s32` (`d_a_shutter.h:43`) zero-fill, meaning "no pending request".
const WANT_IDLE: i32 = 0; // <- d_a_shutter.cpp:235
/// `field_0x320 = 1` (`d_a_shutter.cpp:241`) - the switch rose, so open.
const WANT_OPEN: i32 = 1; // <- d_a_shutter.cpp:241
/// `field_0x320 = 2` (`d_a_shutter.cpp:238`) - the switch fell, so close.
const WANT_CLOSE: i32 = 2; // <- d_a_shutter.cpp:238

/// `daShutter_prm::getSwitchNo(ac) = (fopAcM_GetParam(ac) >> 0) & 0xFF` (`d_a_shutter.h:60`) over
/// the WHOLE u32 (`f_op_actor_mng.h:127-129`).
const SWITCH_SHIFT: u32 = 0; // <- d_a_shutter.h:60
/// `daShutter_prm::getType(ac) = (fopAcM_GetParam(ac) >> 8) & 0x0F` (`d_a_shutter.h:61`).
const TYPE_SHIFT: u32 = 8; // <- d_a_shutter.h:61
const TYPE_WIDTH: u32 = 0x0F; // <- d_a_shutter.h:61
const SWITCH_WIDTH: u32 = 0xFF; // <- d_a_shutter.h:60

/// The two-leaf sliding shutters behind `fpcNm_SHUTTER_e` - `daShutter_c`
/// (`d_a_shutter.h:9-57`), the actor behind the stage strings `Htobi1` and `Htobi2`.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daShutter_c` is a bare `fopAc_ac_c`
/// holding two `J3DModel*`, two `dBgW*`, two `Mtx`, a `cXyz[2]` and the switch/staff bookkeeping
/// (`d_a_shutter.h:38-56`); it registers no `dCcD` collider of any kind. The two `mdBgW` are
/// `dBgW` moving backgrounds built from the Htobi1/Htobi2 DZBs (`CreateHeap` `:59`), which is not a
/// `dCcD` collider and is RULE 3b absent besides. The profile `Group` is `fopAc_ACTOR_e` (`:328`).
/// Neither `combat::HitPoints` nor `enemy::Hurt` is attached - attaching them would let
/// `combat::control` despawn a shutter retail cannot damage.
///
/// `daShutter_IsDelete` (`:302-305`) is the bare "accepts a delete request" form of RULE 6b. The
/// real destruction is `fopAcM_delete`, which appears nowhere in this translation unit; the
/// `_create` failure paths (`:108-110`) are `fopAcM_entrySolidHeap` and `dComIfG_resLoad` - heap and
/// asset work, so `spawn` never refuses.
///
/// `m_height[2] = {200.0f, 100.0f}` (`:16`), `field_0x334` and `field_0x33B` (`d_a_shutter.h:49`,
/// `:56`) have no reader anywhere in the decomp, so none of the three is transcribed.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Shutter2 {
    /// `mType` (`d_a_shutter.h:50`, a `u8`), set by `_create` (`:105`). Indexes every per-type
    /// table in the class; see [`Shutter2::slot`] for the out-of-range fold.
    pub kind: u8,
    /// `mSwitchNo` (`d_a_shutter.h:44`, an `s32`) from `daShutter_prm::getSwitchNo` (`:76`).
    pub switch_no: u8,
    /// `mcXyz[0].x` (`d_a_shutter.h:42`) - the left leaf's local offset. `Create` (`:77`, `:84`)
    /// seeds it to `-width / 2`, minus `move_len` when the switch is already on.
    pub leaf_min: f32,
    /// `mcXyz[1].x` - the right leaf's local offset (`:78`, `:85`).
    pub leaf_max: f32,
    /// `field_0x320` (`d_a_shutter.h:43`): [`WANT_IDLE`], [`WANT_OPEN`] or [`WANT_CLOSE`].
    pub want: i32,
    /// `mFrameTimer` (`d_a_shutter.h:52`, an `s8`) from `Create`'s 30 (`:87`).
    pub frame_timer: i8,
    /// `mbIsSwitch` (`d_a_shutter.h:51`), the previous tick's switch read (`:142`).
    pub was_switch: bool,
    /// `mTimer` (`d_a_shutter.h:53`, a `u8`), armed to 15 by `ACT_WAIT` (`:171`).
    pub timer: u8,
    /// `mOpenEventIdx` (`d_a_shutter.h:47`, an `s16`) from `getEventIdx(name, 0xff)` (`:94`);
    /// `None` is the -1 that a name missing from the stage list also produces.
    pub open_slot: Option<usize>,
    /// `mCloseEventIdx` (`d_a_shutter.h:48`) from `:97`; permanently `None` for `Htobi1`, whose
    /// `m_close_ev_name` slot is NULL.
    pub close_slot: Option<usize>,
    /// One-shot guard so `Create`'s two `getEventIdx` calls happen once, on the first tick.
    pub entered: bool,
}

impl Shutter2 {
    /// `daShutter_c::Create` (`d_a_shutter.cpp:71-100`) plus `_create`'s `mType` (`:105`),
    /// minus `fopAcM_SetMtx` (`:72`), the cull box (`:73-75`, carried by [`Shutter2::cull`]),
    /// `set_mtx` (`:88`), the two `Regist` / `Move` pairs over the `dBgW` (`:89-92`) and the two
    /// `getEventIdx` lookups (`:93-98`), which are deferred to the first tick because they need
    /// the stage event list.
    ///
    /// `if (!fopAcM_isSwitch(this, mSwitchNo)) { field_0x328 = 3; } else { field_0x328 = 2;
    /// mcXyz[0].x -= m_move_len[mType]; mcXyz[1].x += m_move_len[mType]; }` (`:79-86`) decides the
    /// leaf offsets, so it needs `Switches` - see [`Shutter2::create_init`].
    ///
    /// RULE 2: `dStage_actorInit` uses `fopAcM_CreateAppend`, whose default `scale` is
    /// `(10, 10, 10)` (`f_op_actor_mng.cpp:84-86`) divided by ten at `f_op_actor.cpp:280`, i.e.
    /// exactly 1.0, and `set_mtx` (`:124`) is the only reader of `scale`.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: ((parameters >> TYPE_SHIFT) & ((1 << TYPE_WIDTH) - 1)) as u8,
            switch_no: ((parameters >> SWITCH_SHIFT) & ((1 << SWITCH_WIDTH) - 1)) as u8,
            // `mcXyz` is a `cXyz[2]` (`d_a_shutter.h:42`) whose `.y` and `.z` are never written;
            // `:77-78` set only `.x`, and `set_mtx`'s `multVec` (`:123`) is the sole reader.
            leaf_min: 0.0,
            leaf_max: 0.0,
            // `field_0x320` has no initialiser; `fopAcM_ct`'s `sBs_ClearArea` zero-fill
            // (`f_pc_base.cpp:98`) makes it 0, which is `demo`'s "no pending request" test (`:235`).
            want: WANT_IDLE,
            frame_timer: FRAME_TIMER,
            // `mbIsSwitch` has no initialiser either, so the same zero-fill makes it FALSE.
            was_switch: false,
            timer: 0,
            open_slot: None,
            close_slot: None,
            entered: false,
        }
    }

    /// `fopAcM_setCullSizeBox(this, cullMin.x, cullMin.y, cullMin.z, cullMax.x, cullMax.y,
    /// cullMax.z)` (`d_a_shutter.cpp:73-75`), fed by `m_cull_min[mType]` / `m_cull_max[mType]`
    /// (`:20-21`). A real box, so the profile's `fopAc_CULLBOX_CUSTOM_e` (`:329`) resolves.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        let slot = self.slot();
        (CULL_MIN[slot], CULL_MAX[slot])
    }

    /// Every per-type table in the class - `m_max_speed`, `m_min_speed`, `m_move_len`, `m_width`,
    /// `m_height`, `m_bdlidx`, `m_dzbidx`, `m_heapsize`, `m_cull_min`, `m_cull_max`,
    /// `m_arcname`, `m_open_ev_name`, `m_close_ev_name`, `m_staff_name` - has exactly two entries
    /// (`d_a_shutter.cpp:12-26`) while `daShutter_prm::getType` decodes FOUR bits
    /// (`d_a_shutter.h:61`). A type above 1 therefore reads past all fourteen tables at once, which
    /// is retail undefined behaviour with no modelled answer; RULE 10 requires the index be
    /// guarded, so anything that is not exactly 1 folds to 0.
    fn slot(&self) -> usize {
        if self.kind == 1 { 1 } else { 0 }
    }

    /// `Create`'s leaf-pose choice (`:77-86`) plus its two `getEventIdx` lookups (`:93-98`).
    /// Split out of `spawn` because the pose needs `Switches` and the lookups need `EventList`.
    fn create_init(&mut self, switches: &Switches, list: &EventList) {
        let slot = self.slot();
        // `mcXyz[0].x = -m_width[mType] / 2.0f; mcXyz[1].x = m_width[mType] / 2.0f;` (:77-78)
        self.leaf_min = -WIDTH[slot] / 2.0;
        self.leaf_max = WIDTH[slot] / 2.0;
        if switches.test(self.switch_no) {
            // `field_0x328 = 2; mcXyz[0].x -= m_move_len[mType]; mcXyz[1].x += m_move_len[mType];`
            // (:83-85) - the shutter starts open. `field_0x328 = 3` in the else arm (:80) has no
            // reader anywhere in the decomp, so neither value is carried.
            self.leaf_min -= MOVE_LEN[slot];
            self.leaf_max += MOVE_LEN[slot];
        }
        // `if (m_open_ev_name[mType] != NULL) mOpenEventIdx = getEventIdx(name, 0xff);` (:93-95)
        self.open_slot = list.event_index(OPEN_EVENT[slot]);
        // `if (m_close_ev_name[mType] != NULL) mCloseEventIdx = getEventIdx(name, 0xff);`
        // (:96-98) - `and_then` is that NULL test, since `m_close_ev_name[0]` is NULL (`:25`).
        self.close_slot = CLOSE_EVENT[slot].and_then(|name| list.event_index(name));
        self.entered = true;
    }

    /// `daShutter_c::_execute` (`d_a_shutter.cpp:133-144`) minus `set_mtx` (`:138`) and the
    /// `mdBgW[i]->Move()` loop (`:139-141`), both renderer / `dBgW` work.
    fn execute(
        &mut self,
        list: &EventList,
        active: &mut ActiveEvent,
        switches: &Switches,
        ev_flags: &mut EventFlags,
        entity: Entity,
    ) {
        // `if (mFrameTimer >= 0) { mFrameTimer--; }` (:134-136)
        if self.frame_timer >= 0 {
            self.frame_timer -= 1;
        }
        self.demo(list, active, switches, ev_flags, entity);
        // `mbIsSwitch = fopAcM_isSwitch(this, mSwitchNo);` (:142)
        self.was_switch = switches.test(self.switch_no);
    }

    /// `daShutter_c::demo` (`:233-266`).
    ///
    /// `eventInfo.checkCommandDemoAccrpt()` (`:245`) is `mCommand == dEvtCmd_INDEMO_e`
    /// (`f_op_actor.h:177`), a latch `dEvt_control_c::demoCheck` sets on both ordered actors when
    /// the event starts (`d_event.cpp:314-316`) and `demoEnd` clears (`:331-337`) - so it stays
    /// true through the event's CLOSE stage. That is exactly "one of this actor's two events is in
    /// `Stage::Play` or `Stage::Close`", the two states `getMyStaffId` scans
    /// (`d_event_manager.cpp:412`), which is how [`Shutter2::demo_running`] reads it.
    ///
    /// The two `eventInfo.onCondition(dEvtCnd_UNK2_e)` calls (`:260`, `:264`) are dropped for the
    /// reason `htobi3.rs` gives: nothing in the decomp ever calls `chkCondition` with
    /// `dEvtCnd_UNK2_e`, and `beforeProc` clears `mCondition` on every `fopAc_Execute`
    /// (`f_op_actor.cpp:176`).
    fn demo(
        &mut self,
        list: &EventList,
        active: &mut ActiveEvent,
        switches: &Switches,
        ev_flags: &mut EventFlags,
        entity: Entity,
    ) {
        // `u8 isSwitch = fopAcM_isSwitch(this, mSwitchNo);` (:234)
        let is_switch = switches.test(self.switch_no);
        if self.want == WANT_IDLE
            && is_switch != self.was_switch
            && self.frame_timer < 0
        {
            self.want = if is_switch { WANT_OPEN } else { WANT_CLOSE };
        }

        if self.demo_running(active) {
            if self
                .open_slot
                .is_some_and(|slot| active.start_check(slot))
                && self.want == WANT_OPEN
            {
                self.want = WANT_IDLE;
            }
            if self
                .close_slot
                .is_some_and(|slot| active.start_check(slot))
                && self.want == WANT_CLOSE
            {
                self.want = WANT_IDLE;
            }
            if let Some(slot) = self
                .open_slot
                .filter(|slot| active.end_check(*slot))
                .or_else(|| self.close_slot.filter(|slot| active.end_check(*slot)))
            {
                // `dComIfGp_event_reset()` (`:253`) is `dEvDtBase_c::reset()` (`d_event.h:131`) =
                // `mEventFlag |= 8` on the running event, which `dEvt_control_c::check`
                // (`d_event.cpp:546-554`) turns into `endProc` -> `demoEnd`. `ActiveEvent::end` is
                // that teardown. It does NOT clear any `dSv_event_flag_c`.
                let _ = active.end(slot);
            }
            self.shutter_move(list, active, ev_flags);
            return;
        }

        // `else if ((field_0x320 == 1) && (mOpenEventIdx != 0))` (:258). The `!= 0` guard is the
        // `m_close_ev_name[mType] != NULL` test of `Create` (`:96`): a NULL name leaves
        // `mCloseEventIdx` at its zero-fill, and `Htobi1`'s close name is NULL (`:25`). `None` is
        // that case, and also the -1 that a missing event name produces - in both retail orders an
        // event that cannot run.
        if self.want == WANT_OPEN && self.open_slot.is_some() {
            Self::order(self.open_slot, list, active, entity);
        } else if self.want == WANT_CLOSE && self.close_slot.is_some() {
            Self::order(self.close_slot, list, active, entity);
        }
    }

    /// `eventInfo.checkCommandDemoAccrpt()` (`:245`) as `Stage::Play | Stage::Close`.
    fn demo_running(&self, active: &ActiveEvent) -> bool {
        [self.open_slot, self.close_slot]
            .into_iter()
            .flatten()
            .any(|slot| matches!(active.stage_of(slot), Some(Stage::Play) | Some(Stage::Close)))
    }

    /// `fopAcM_orderOtherEventId(this, mOpenEventIdx)` (`:259`) and its `mCloseEventIdx` twin
    /// (`:263`), guarded exactly as `hbrf1::Hbrf1::order` and `htobi3::Htobi3::order` guard it -
    /// re-ordering an event already in ORDER / PLAY / CLOSE rewrites its state
    /// (`d_event_manager.cpp:367`).
    fn order(slot: usize, list: &EventList, active: &mut ActiveEvent, entity: Entity) {
        if active.stage_of(slot).is_none() {
            let _ = active.order(list, slot, entity);
        }
    }

    /// `daShutter_c::shutter_move` (`:147-230`).
    ///
    /// `dComIfGp_evmng_getMyActIdx(mStaffId, action_table, ARRAY_SIZE(action_table), FALSE, 0)`
    /// (`:160`) latches `staff->mCurActionIdx` once `mbHasAction` is set
    /// (`d_event_manager.cpp:456-488`), cleared only by `dEvDtStaff_c::init` and `advanceCut`
    /// (`d_event_data.cpp:179`, `:193`) - i.e. exactly when the cut changes, so resolving the cut
    /// name afresh each tick is equivalent to the latch.
    ///
    /// `dComIfGp_evmng_cutEnd(mStaffId)` (`:173`, `:180`, `:205`, `:221`, `:226`) is
    /// `dEvent_manager_c::cutEnd` (`d_event_manager.cpp:549-555`), which sets the current cut's flag
    /// id - `ActiveEvent::end_staff_cut`.
    fn shutter_move(
        &mut self,
        list: &EventList,
        active: &mut ActiveEvent,
        ev_flags: &mut EventFlags,
    ) {
        // `dComIfGp_evmng_getMyStaffId(m_staff_name[mType], NULL, 0)` (`:255`) returns -1 when no
        // running event carries the staff, and `getMyActIdx(-1, ...)` then returns -1 so only the
        // `default` arm runs - while `cutEnd(-1)` itself bails (`d_event_manager.cpp:550-551`).
        let staff = STAFF[self.slot()];
        let Some((cut, _)) = active.staff_cut(list, staff) else {
            return;
        };
        let slot = self.slot();
        let action = list
            .cut_name(cut)
            .and_then(|name| ACTIONS.iter().position(|entry| *entry == name));
        match action {
            // `case ACT_WAIT:` (:169-175)
            Some(a) if a == ACT_WAIT => {
                self.timer = WAIT_TIMER;
                active.end_staff_cut(list, ev_flags, staff);
            }
            // `case ACT_WAIT02:` (:176-183)
            Some(a) if a == ACT_WAIT02 => {
                // `if (cLib_calcTimer(&mTimer) == 0)` (:178) - `cLib_calcTimer`
                // (`c_lib.h:85-91`) decrements a non-zero value by one and returns it.
                if self.timer != 0 {
                    self.timer -= 1;
                }
                if self.timer == 0 {
                    // `field_0x339 = 0;` (:179) latches the open/close sound. Both of that sound's
                    // writes (`:190`, and `field_0x339 = 1` at `:188` / `:211`) are gated on
                    // `dStage_stagInfo_GetSaveTbl(...) == dSv_save_c::STAGE_TOTG` (`:189`, `:195`),
                    // whose save-table index is stage data this checkout does not parse, so
                    // `field_0x339` has no surviving reader and is not carried.
                    active.end_staff_cut(list, ev_flags, staff);
                }
            }
            // `case ACT_OPEN:` (:184-207)
            Some(a) if a == ACT_OPEN => {
                // `fVar4 = cLib_addCalc(&mcXyz[0].x, -m_move_len[mType] - m_width[mType] / 2.0f,
                // 0.1f, maxVel, minVel);` (:193)
                let min_target = -MOVE_LEN[slot] - WIDTH[slot] / 2.0;
                walk::add_calc(
                    &mut self.leaf_min,
                    min_target,
                    ADD_SCALE,
                    MAX_SPEED[slot],
                    MIN_SPEED[slot],
                );
                // `fVar5 = cLib_addCalc(&mcXyz[1].x, m_move_len[mType] + m_width[mType] / 2.0f,
                // 0.1f, maxVel, minVel);` (:194)
                let max_target = MOVE_LEN[slot] + WIDTH[slot] / 2.0;
                walk::add_calc(
                    &mut self.leaf_max,
                    max_target,
                    ADD_SCALE,
                    MAX_SPEED[slot],
                    MIN_SPEED[slot],
                );
                // `if (fVar4 != 0.0f) { break; } if (fVar5 != 0.0f) { break; }` (:199-204).
                // `cLib_addCalc` returns `fabsf(target - *pValue)` (`c_lib.cpp:52`), so a non-zero
                // return is exactly "the leaf has not reached its target yet".
                if self.leaf_min != min_target || self.leaf_max != max_target {
                    return;
                }
                // `field_0x33A++` (:186) only feeds the `field_0x33A == 75` stop sound and its
                // `StartShock(4, -0x21, cXyz(0.0f, 1.0f, 0.0f))` (`:195-198`), which is likewise
                // STAGE_TOTG-gated, so the counter and `ACT_WAIT`'s `field_0x33A = 0` (`:172`) are
                // both write-only here and are not carried.
                active.end_staff_cut(list, ev_flags, staff);
            }
            // `case ACT_CLOSE:` (:208-223)
            Some(a) if a == ACT_CLOSE => {
                // `fVar4 = cLib_addCalc(&mcXyz[0].x, -m_width[mType] / 2.0f, 0.1f, maxVel, minVel);`
                // (:213)
                let min_target = -WIDTH[slot] / 2.0;
                walk::add_calc(
                    &mut self.leaf_min,
                    min_target,
                    ADD_SCALE,
                    MAX_SPEED[slot],
                    MIN_SPEED[slot],
                );
                // `fVar5 = cLib_addCalc(&mcXyz[1].x, m_width[mType] / 2.0f, 0.1f, maxVel, minVel);`
                // (:214)
                let max_target = WIDTH[slot] / 2.0;
                walk::add_calc(
                    &mut self.leaf_max,
                    max_target,
                    ADD_SCALE,
                    MAX_SPEED[slot],
                    MIN_SPEED[slot],
                );
                // `if (fVar4 != 0.0f) { break; } if (fVar5 != 0.0f) { break; }` (:215-220)
                if self.leaf_min != min_target || self.leaf_max != max_target {
                    return;
                }
                active.end_staff_cut(list, ev_flags, staff);
            }
            // `default:` - `getMyActIdx`'s -1, i.e. no such cut (`:224-228`).
            _ => active.end_staff_cut(list, ev_flags, staff),
        }
    }
}

pub fn is_shutter2(name: &str) -> bool {
    HTOBI_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    switches: Res<Switches>,
    mut ev_flags: ResMut<EventFlags>,
    mut shutters: Query<(Entity, &mut Shutter2)>,
) {
    for (entity, mut shutter) in &mut shutters {
        if !shutter.entered {
            shutter.create_init(&switches, &list);
        }
        shutter.execute(&list, &mut active, &switches, &mut ev_flags, entity);
    }
}