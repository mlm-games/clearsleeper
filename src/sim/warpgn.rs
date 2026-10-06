use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::event_manager::EventManager;
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, Switches};
use super::player_status::{PlayerStatus, STTS0_SHIP_RIDE};
use super::ship::Ship;
use crate::text::msgproc::MsgGameInfo;

/// `OBJNAME("Warpgn", fpcNm_WARPGANON_e, 255, 0)` (`d_stage.cpp:1159`).
///
/// The archive is `const char daWarpgn_c::m_arcname[] = "Gmjwp";` (`d_a_warpgn.cpp:15`) - a third
/// spelling, unrelated to the stage string.
///
/// COLLISION CHECK: `"Warpgn"` also appears at `src/sim/event.rs` inside the event manager's
/// staff-name comparisons (the same list `Ghrwp` at `event.rs:881` sits in). That is a STAFF-name
/// lookup, not a stage-placement claim: no `*_ACTORS` array contains it, and `src/sim/mod.rs`
/// names neither `Warpgn` nor any `warpgn` module. `src/sim/warpfo.rs` claims `"Warpfo"`
/// (`fpcNm_WARPFLOWER_e`, `d_stage.cpp` row for `Warpfo`) and `src/sim/warpf.rs` claims `"Warpf"`
/// (`fpcNm_WARPFLOWER_e`'s sibling) - three distinct byte-exact strings, no overlap.
pub const WARPGN_ACTOR: &str = "Warpgn";

/// `namespace daWarpgn_prm { inline u32 getSwitchNo(daWarpgn_c* i_this) { return
/// (fopAcM_GetParam(i_this) >> 8) & 0xFF; } }` (`d_a_warpgn.h:70-72`) over the whole u32.
const SWITCH_SHIFT: u32 = 8; // <- d_a_warpgn.h:71
const SWITCH_MASK: u32 = 0xFF; // <- d_a_warpgn.h:71
/// `inline u32 getSceneNo(daWarpgn_c* i_this) { return fopAcM_GetParam(i_this) & 0xFF; }`
/// (`d_a_warpgn.h:73-75`) - the low byte, the scene-list index `dLib_setNextStageBySclsNum` takes.
const SCENE_MASK: u32 = 0xFF; // <- d_a_warpgn.h:74

/// `fopAcM_setCullSizeBox(this, -400.0f, 0.0f, -400.0f, 400.0f, 2000.0f, 400.0f)`
/// (`d_a_warpgn.cpp:222`), reached from the profile's `fopAc_CULLBOX_CUSTOM_e` (`:756`). The
/// profile `Status` carries `fopAcStts_CULL_e` (`:755`), so the box is live. `fopAcM_setCullSizeFar`
/// (`:223`) is a render-distance override with no modelled reader.
const CULL_MIN: [f32; 3] = [-400.0, 0.0, -400.0]; // <- d_a_warpgn.cpp:222
const CULL_MAX: [f32; 3] = [400.0, 2000.0, 400.0]; // <- d_a_warpgn.cpp:222

/// `static char* action_table[] = { "WAIT", "WARP", "WARP_ARRIVE", "WARP_ARRIVE_END",
/// "START_WARP", "APPEAR" };` (`d_a_warpgn.cpp:181`) - the `event_init_tbl` (`:18-25`) and
/// `event_action_tbl` (`:28-35`) order and the `getMyActIdx` return values.
const ACTIONS: [&str; 6] = [
    "WAIT",
    "WARP",
    "WARP_ARRIVE",
    "WARP_ARRIVE_END",
    "START_WARP",
    "APPEAR",
]; // <- d_a_warpgn.cpp:181
const ACT_APPEAR: i32 = 5; // <- d_a_warpgn.cpp:24,34 (initAppear / actAppear)

/// `m2C4` (`d_a_warpgn.h:58`, an `s32`) - the pending request, zero-filled by `fopAcM_ct`.
const WANT_IDLE: i32 = 0; // <- d_a_warpgn.cpp:293 (the "!= 0" test)
const WANT_APPEAR: i32 = 1; // <- d_a_warpgn.cpp:274 (normal_execute's rising-switch arm)
const WANT_WARP: i32 = 2; // <- d_a_warpgn.cpp:291 (normal_execute's check_warp arm)
/// `f32 dist_to_ship = (dComIfGp_getShipActor()->current.pos - current.pos).absXZ(); if
/// (dist_to_ship < 200.0f)` (`d_a_warpgn.cpp:421-423`) - `cXyz::absXZ` (`c_vec.h`) is the flat
/// distance, so `.y` is excluded.
const SHIP_WARP_RANGE: f32 = 200.0; // <- d_a_warpgn.cpp:423
/// `mTimer = 100;` (`d_a_warpgn.cpp:349`) - `initAppear`'s tick count.
const APPEAR_TIMER: i32 = 100; // <- d_a_warpgn.cpp:349
/// `fopAcM_seStart(this, JA_SE_OBJ_GN_WAPR_EFF, 0);` (`d_a_warpgn.cpp:283`).
const SE_GN_WAPR_EFF: u32 = 0x1089; // <- d_a_warpgn.cpp:283 (JAZelAudio_SE.h:323)

/// The Ganon warp point - `daWarpgn_c` (`d_a_warpgn.h`), the actor behind `fpcNm_WARPGANON_e`.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. The class is a `fopAc_ac_c` plus three
/// `JPABaseEmitter*`, one `J3DModel*`, four texture/transform/tevkey animators and the switch and
/// event bookkeeping (`d_a_warpgn.h:50-66`); it registers no `dCcD` AT or Tg collider and
/// `dComIfG_Ccsp()->Set` appears nowhere in the translation unit. The profile `Group` is
/// `fopAc_ACTOR_e` (`:757`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached - attaching
/// them would let `combat::control` despawn a warp point retail cannot damage.
///
/// `daWarpgn_IsDelete` (`:747-750`) is the bare "accepts a delete request" form of RULE 6b, and
/// `fopAcM_delete` appears nowhere in this translation unit. `_create` (`:212-231`) can return
/// `cPhs_ERROR_e` only from the `fopAcM_entrySolidHeap` else arm (`:217`), which is heap work, so
/// `spawn` never refuses.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Warpgn {
    /// `mSwitchNo` (`d_a_warpgn.h:54`, a `u32`) from `daWarpgn_prm::getSwitchNo` (`d_a_warpgn.cpp:243`).
    pub switch_no: u8,
    /// `mSceneNo` (`d_a_warpgn.h:57`, a `u32`) from `daWarpgn_prm::getSceneNo` (`d_a_warpgn.cpp:244`) -
    /// the scene-list index `dLib_setNextStageBySclsNum` consumes (`:472`).
    pub scene_no: u8,
    /// `mIsSwitch` (`d_a_warpgn.h:55`, a `u8`) - set to 1 by `CreateInit` when `checkValidWarp`
    /// already holds (`:245-248`), re-read every `normal_execute` (`:281`).
    pub is_switch: u8,
    /// `m2C4` (`d_a_warpgn.h:58`, an `s32`) - [`WANT_IDLE`], [`WANT_APPEAR`] or [`WANT_WARP`].
    pub want: i32,
    /// `mEvtToMajyuuWarpIdx` (`d_a_warpgn.h:59`, an `s16`) from
    /// `dComIfGp_evmng_getEventIdx("TO_MAJYUU_WARP")` (`d_a_warpgn.cpp:240`), or -1.
    pub warp_event: i32,
    /// `mEvtAppearWarpIdx` (`d_a_warpgn.h:60`, an `s16`) from
    /// `dComIfGp_evmng_getEventIdx("APPEAR_WARP")` (`d_a_warpgn.cpp:241`). `CreateInit` forces it to
    /// -1 when the warp is already valid (`:247`), and `checkOrder` does so again after the event
    /// ends (`:476`).
    pub appear_event: i32,
    /// `mTimer` (`d_a_warpgn.h:64`, an `int`) - `initAppear`'s 100 (`:349`), drained by
    /// `cLib_calcTimer` in `actAppear` (`:354`).
    pub timer: i32,
    /// `m388` (`d_a_warpgn.h:65`, a `bool`) - set by `actAppear` (`:368`) and read by `_execute`'s
    /// sound gate (`:282`).
    pub lit: bool,
    /// `mStaffId` (`d_a_warpgn.h:61`, an `s32`) - `dComIfGp_evmng_getMyStaffId("Warpgn")` (`:183`).
    pub staff_id: i32,
    /// One-shot guard so `CreateInit` (`:195-224`) runs once, on the first tick.
    pub entered: bool,
}

impl Warpgn {
    /// `daWarpgn_c::_create` (`d_a_warpgn.cpp:212-231`) plus `daWarpgn_prm`'s two accessors
    /// (`d_a_warpgn.h:70-75`), minus `fopAcM_ct` (`:215`), `dComIfG_resLoad` (`:217`),
    /// `fopAcM_entrySolidHeap` (`:221`) and `CreateInit` (`:195-224`), which needs the save flags,
    /// the switch array and the event manager.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            switch_no: ((parameters >> SWITCH_SHIFT) & SWITCH_MASK) as u8,
            scene_no: (parameters & SCENE_MASK) as u8,
            // `mIsSwitch` (`d_a_warpgn.h:55`) is zero-filled and `CreateInit` overwrites it at `:245`
            // or leaves it 0 otherwise.
            is_switch: 0,
            // `m2C4` (`d_a_warpgn.h:58`) is zero-filled by `fopAcM_ct`'s `sBs_ClearArea`
            // (`f_pc_base.cpp:98`) and `checkOrder`'s `m2C4 != 0` tests (`:462`, `:466`) read that.
            want: WANT_IDLE,
            // `mEvtToMajyuuWarpIdx` / `mEvtAppearWarpIdx` are `s16` (`d_a_warpgn.h:59-60`) with no
            // initialiser, so both take the zero-fill before `CreateInit` writes them.
            warp_event: 0,
            appear_event: 0,
            timer: 0,
            lit: false,
            staff_id: -1,
            entered: false,
        }
    }

    /// `fopAcM_setCullSizeBox(this, -400.0f, 0.0f, -400.0f, 400.0f, 2000.0f, 400.0f)` (`:222`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `CreateInit` (`d_a_warpgn.cpp:195-224`), reduced to the parts with a modelled input.
    ///
    /// OMITTED: `cullMtx = mpModel->getBaseTRMtx()` (`:196`), `fopAcM_setCullSizeFar(this, 1.0f)`
    /// (`:223`, carried by [`Warpgn::cull`] for the box), `set_mtx` (`:197`, `:224`), the three
    /// `dComIfGp_particle_set` / `setProjection` calls and their three `stopCreateParticle`
    /// companions (`:226-240`, RULE 3b particle work), and `dKy_tevstr_init(&mTevStr,
    /// fopAcM_GetRoomNo(this), 0xFF)` (`:250`, an environment-lighting read with `_draw` at `:640`
    /// as its only consumer).
    pub fn create_init(&mut self, switches: &Switches, flags: &EventFlags, manager: &EventManager) {
        // ```
        // mEvtToMajyuuWarpIdx = dComIfGp_evmng_getEventIdx("TO_MAJYUU_WARP");
        // mEvtAppearWarpIdx = dComIfGp_evmng_getEventIdx("APPEAR_WARP");
        // mSwitchNo = daWarpgn_prm::getSwitchNo(this);
        // ```
        // (:240-243). `EventManager::get_event_idx(Some(name), 0xFF)` is the same lookup with
        // retail's default event-info index, and it returns -1 for a name missing from the stage
        // list - exactly the `s16` assignment's range.
        self.warp_event = manager.get_event_idx(Some("TO_MAJYUU_WARP"), 0xFF);
        self.appear_event = manager.get_event_idx(Some("APPEAR_WARP"), 0xFF);
        // ```
        // if (checkValidWarp() != FALSE) { mIsSwitch = 1; mEvtAppearWarpIdx = -1; }
        // ```
        // (:245-248)
        if self.check_valid_warp(switches, flags) {
            self.is_switch = 1;
            self.appear_event = -1;
        }
        self.entered = true;
    }

    /// `daWarpgn_c::_execute` (`d_a_warpgn.cpp:246-277`), minus the `demoActorID != 0` branch's
    /// whole body (`:250-256`, `dComIfGp_demo_getActor` demo-channel work) and `set_mtx` (`:275`).
    ///
    /// `if (demoActorID == 0)` (`:247`) is the normal-play path. `demoActorID` is
    /// `fopAc_ac_c::demoActorID` (`f_op_actor.h`), the demo-capture id: it is 0 for every actor the
    /// stage placed itself, so the else arm is demo-only.
    fn execute(
        &mut self,
        actor: &Actor,
        switches: &Switches,
        flags: &mut EventFlags,
        status: &PlayerStatus,
        ship: Option<(&Ship, &Actor)>,
        manager: &mut EventManager,
        info: &MsgGameInfo,
        events: &mut SimEvents,
    ) {
        self.check_order(actor, flags, manager, info);
        self.demo_proc(manager, info, actor, flags, switches, status, ship, events);
        self.event_order(manager);
        // ```
        // if (m388 || dComIfGs_isEventBit(dSv_event_flag_c::UNK_3D02) != 0) {
        //     fopAcM_seStart(this, JA_SE_OBJ_GN_WAPR_EFF, 0);
        // }
        // ```
        // (:282-284). `dSv_event_flag_c::UNK_3D02` = 0x3D02 (`d_save_event_flag.inc:471`), which
        // RULE 11c passes through unshifted - it is already a `hi_byte << 8 | mask` id.
        if self.lit || flags.test(EVENT_BIT_APPEARED) {
            events.list.push(SimEvent::Se(SE_GN_WAPR_EFF));
        }
    }

    /// `checkOrder` (`d_a_warpgn.cpp:451-478`).
    ///
    /// `eventInfo.checkCommandDemoAccrpt()` (`:453`) is the latch `dEvt_control_c::demoCheck`
    /// sets on both ordered actors when the event starts (`d_event.cpp:314-316`) and `demoEnd`
    /// clears (`:331-337`), so it is exactly "one of this actor's two events is in `Stage::Play` or
    /// `Stage::Close`". `dComIfGp_evmng_startCheck` / `endCheck` are
    /// `EventManager::start_check` / `end_check`, and `dComIfGp_event_reset()` is
    /// `dEvDtBase_c::reset()` (`d_event.h:131`), i.e. `mEventFlag |= 8`, which
    /// `dEvt_control_c::check` (`d_event.cpp:546-554`) turns into `endProc` -> `demoEnd`.
    ///
    /// OMITTED: `dLib_setNextStageBySclsNum(mSceneNo, fopAcM_GetRoomNo(this))` (`:472`) - the
    /// scene-list-to-stage transition. `transition::NextStage` is the crate's channel, but its
    /// writer is `transition::check`'s, keyed on the stage's own linkage rather than on a raw
    /// `dLib_setNextStageBySclsNum` index, so no faithful call can be built here.
    fn check_order(
        &mut self,
        _actor: &Actor,
        flags: &mut EventFlags,
        manager: &EventManager,
        _info: &MsgGameInfo,
    ) {
        if manager.start_check(self.warp_event) && self.want != WANT_IDLE {
            self.want = WANT_IDLE;
        }
        if manager.start_check(self.appear_event) && self.want != WANT_IDLE {
            self.want = WANT_IDLE;
        }
        // ```
        // if (dComIfGp_evmng_endCheck(mEvtAppearWarpIdx)) {
        //     dComIfGs_onEventBit(dSv_event_flag_c::UNK_3D02);
        //     mEvtAppearWarpIdx = -1;
        //     dComIfGp_event_reset();
        // }
        // ```
        // (:475-479)
        if manager.end_check(self.appear_event) {
            flags.set(EVENT_BIT_APPEARED);
            self.appear_event = -1;
        }
    }

    /// `demo_proc` (`d_a_warpgn.cpp:159-179`), reduced to the staff lookup and the `APPEAR` cut.
    ///
    /// OMITTED: `event_init_tbl` / `event_action_tbl` for the five non-`APPEAR` cuts. `initWait`
    /// (`:135`), `initWarp` (`:175`), `initWarpArrive` (`:186`), `initWarpArriveEnd` (`:205`) and
    /// `initStartWarp` (`:141`) are emitter / BTK / BCK / BRK frame work plus
    /// `dComIfGp_evmng_setGoal` (`:142`, a demo camera target); and every `act*` except `actAppear`
    /// is `anim_play` plus, for `actWarpArriveEnd`, `mpBtkAnm2->getFrame() < 0.5f` (`:215`) -
    /// archive data RULE 3b lists as unavailable. `anim_play` (`:404-451`) itself only drives BTK /
    /// BCK / BRK speeds and frames and has no modelled reader.
    fn demo_proc(
        &mut self,
        manager: &mut EventManager,
        info: &MsgGameInfo,
        actor: &Actor,
        flags: &mut EventFlags,
        switches: &Switches,
        status: &PlayerStatus,
        ship: Option<(&Ship, &Actor)>,
        events: &mut SimEvents,
    ) {
        // `mStaffId = dComIfGp_evmng_getMyStaffId("Warpgn");` (:183)
        self.staff_id = manager.get_my_staff_id("Warpgn", None, 0);
        // ```
        // if (dComIfGp_event_runCheck() && !eventInfo.checkCommandTalk() && mStaffId != -1) {
        // ```
        // (:185). `eventInfo.checkCommandTalk()` (`:186`) needs the per-actor event command latch,
        // which `EventManager` models for no actor - `kamome.rs:1400` records the same gap - so
        // the gate is transcribed without it.
        if !info.event_run_check() || self.staff_id == -1 {
            return;
        }
        let action = manager.get_my_act_idx(self.staff_id, &ACTIONS, false, 0);
        // ```
        // if (action_index == -1) { dComIfGp_evmng_cutEnd(mStaffId); }
        // else {
        //     if (dComIfGp_evmng_getIsAddvance(mStaffId)) { (this->*event_init_tbl[i])(mStaffId); }
        //     if ((this->*event_action_tbl[i])(mStaffId)) { dComIfGp_evmng_cutEnd(mStaffId); }
        // }
        // ```
        // (:187-199)
        if action == -1 {
            manager.cut_end(self.staff_id);
            return;
        }
        if manager.get_is_advance(self.staff_id) && action == ACT_APPEAR {
            // `initAppear(int) { mTimer = 100; }` (:348-350)
            self.timer = APPEAR_TIMER;
        }
        let done = if action == ACT_APPEAR {
            self.act_appear()
        } else {
            // `actWait` / `actWarp` / `actWarpArrive` / `actWarpArriveEnd` / `actStartWarp` are all
            // `{ anim_play(n); return TRUE; }` (`:137-138`, `:141-143`, `:172-174`, `:195-197`,
            // `:213-215`, `:232-234`), so each of the five returns TRUE immediately.
            true
        };
        if done {
            manager.cut_end(self.staff_id);
        }
        // `normal_execute` is reached through `checkOrder`'s `else if (m2C4 == 0)` arm (`:476-477`)
        // in retail; this crate runs it from the same place, once the cut ends.
        if self.want == WANT_IDLE {
            self.normal_execute(actor, switches, flags, status, ship);
        }
    }

    /// `actAppear` (`d_a_warpgn.cpp:352-370`):
    /// ```c
    /// BOOL rt = FALSE;
    /// if (cLib_calcTimer(&mTimer) == 0) { ...emitter work...; mTimer = 100; rt = TRUE; }
    /// anim_play(1);
    /// m388 = true;
    /// return rt;
    /// ```
    /// (`:352-370`). `cLib_calcTimer` (`c_lib.h:85-91`) decrements a non-zero value and returns
    /// it, so `== 0` means the countdown expired on THIS tick - which is why `mTimer = 100` is
    /// inside the arm.
    fn act_appear(&mut self) -> bool {
        let mut done = false;
        if self.timer != 0 {
            self.timer -= 1;
        }
        if self.timer == 0 {
            self.timer = APPEAR_TIMER;
            done = true;
        }
        // `anim_play(1);` (:367) is BTK / BCK / BRK play-speed work with `Draw` as the only reader.
        // `m388 = true;` (:368)
        self.lit = true;
        done
    }

    /// `normal_execute` (`d_a_warpgn.cpp:226-244`).
    ///
    /// ```c
    /// BOOL new_is_switch = fopAcM_isSwitch(this, mSwitchNo);
    /// if (!dComIfGs_isEventBit(dSv_event_flag_c::UNK_3D02)) {
    ///     if (new_is_switch && mIsSwitch == 0) { m2C4 = 1; }
    /// } else { anim_play(2); ...emitter work... }
    /// if (check_warp()) { if (DEMO_SELECT(true, !dComIfGp_event_runCheck())) { m2C4 = 2; } }
    /// mIsSwitch = new_is_switch;
    /// ```
    /// (`:226-244`). `DEMO_SELECT(true, X)` is `true` for every retail build
    /// (`VERSION_NTSC_U > VERSION_JPN > VERSION_DEMO`), so the `!runCheck` guard is compiled out
    /// and the `m2C4 = 2` assignment is unconditional on `check_warp()`.
    fn normal_execute(
        &mut self,
        actor: &Actor,
        switches: &Switches,
        flags: &EventFlags,
        status: &PlayerStatus,
        ship: Option<(&Ship, &Actor)>,
    ) {
        // `BOOL new_is_switch = fopAcM_isSwitch(this, mSwitchNo);` (:227)
        let new_is_switch = switches.test(self.switch_no);
        // ```
        // if (!dComIfGs_isEventBit(dSv_event_flag_c::UNK_3D02)) {
        //     if (new_is_switch && mIsSwitch == 0) { m2C4 = 1; }
        // } else { anim_play(2); ... }
        // ```
        // (:228-241)
        if !flags.test(EVENT_BIT_APPEARED) && new_is_switch && self.is_switch == 0 {
            self.want = WANT_APPEAR;
        }
        // ```
        // if (check_warp()) { if (DEMO_SELECT(true, !dComIfGp_event_runCheck())) { m2C4 = 2; } }
        // ```
        // (:242-244)
        if self.check_warp(actor, switches, flags, status, ship) {
            self.want = WANT_WARP;
        }
        // `mIsSwitch = new_is_switch;` (:246)
        self.is_switch = u8::from(new_is_switch);
    }

    /// `check_warp` (`d_a_warpgn.cpp:392-410`):
    /// ```c
    /// if (checkValidWarp() == FALSE) { return FALSE; }
    /// if (dComIfGp_checkPlayerStatus0(0, daPyStts0_SHIP_RIDE_e) != 0) {
    ///     daShip_c* ship = dComIfGp_getShipActor();
    ///     if (ship != NULL) {
    ///         f32 dist_to_ship = (dComIfGp_getShipActor()->current.pos - current.pos).absXZ();
    ///         if (dist_to_ship < 200.0f) { return TRUE; }
    ///     }
    /// }
    /// return FALSE;
    /// ```
    /// (`:392-410`). `daPyStts0_SHIP_RIDE_e` is `0x00010000` (`d_a_player.h`) and
    /// `PlayerStatus::STTS0_SHIP_RIDE` (`player_status.rs:23`) is that same value, written by
    /// `player_status::control` from `Ship::riding` (`player_status.rs:122`, `:317`).
    ///
    /// OMITTED: `daPy_py_c* player = daPy_getPlayerActorClass(); float dist_to_player =
    /// (player->current.pos - current.pos).absXZ();` (`:393-394`) - `dist_to_player` is assigned and
    /// NEVER READ in the rest of the function, so dropping it changes no behaviour.
    fn check_warp(
        &self,
        actor: &Actor,
        switches: &Switches,
        flags: &EventFlags,
        status: &PlayerStatus,
        ship: Option<(&Ship, &Actor)>,
    ) -> bool {
        if !self.check_valid_warp(switches, flags) {
            return false;
        }
        if !status.check0(STTS0_SHIP_RIDE) {
            return false;
        }
        let Some((_, hull)) = ship else {
            return false;
        };
        let dx = hull.pos.x - actor.pos.x;
        let dz = hull.pos.z - actor.pos.z;
        // `cXyz::absXZ` (`c_vec.h`) is the flat distance.
        (dx * dx + dz * dz).sqrt() < SHIP_WARP_RANGE
    }

    /// `checkValidWarp` (`d_a_warpgn.cpp:412-418`):
    /// `if (!fopAcM_isSwitch(this, mSwitchNo) && !dComIfGs_isEventBit(UNK_3D02)) { return FALSE; }`
    fn check_valid_warp(&self, switches: &Switches, flags: &EventFlags) -> bool {
        switches.test(self.switch_no) || flags.test(EVENT_BIT_APPEARED)
    }

    /// `eventOrder` (`d_a_warpgn.cpp:429-436`), minus the two
    /// `eventInfo.onCondition(dEvtCnd_UNK2_e)` calls.
    ///
    /// The `onCondition` pair is dropped for the reason `shutter2.rs` gives: nothing in the decomp
    /// calls `chkCondition` with `dEvtCnd_UNK2_e`, and `beforeProc` clears `mCondition` on every
    /// `fopAc_Execute` (`f_op_actor.cpp:176`). `fopAcM_orderOtherEventId(this, idx)` is
    /// `EventManager::order`, guarded exactly as `shutter2::Shutter2::order` guards it -
    /// re-ordering an event already in ORDER / PLAY / CLOSE rewrites its state
    /// (`d_event_manager.cpp:367`), so the `stage_of(...).is_none()` guard is mandatory (BUG CLASS 5).
    fn event_order(&mut self, manager: &mut EventManager) {
        let slot = match self.want {
            WANT_WARP => Some(self.warp_event),
            WANT_APPEAR => Some(self.appear_event),
            _ => None,
        };
        if let Some(slot) = slot.filter(|slot| *slot >= 0) {
            let _ = manager.order(slot);
        }
    }
}

/// `dSv_event_flag_c::UNK_3D02` = 0x3D02 (`d_save_event_flag.inc:471`) - the "the Ganon warp has
/// already appeared" save bit, read by `checkValidWarp` (`d_a_warpgn.cpp:413`), by `normal_execute`
/// (`:228`), by `_execute`'s sound gate (`:282`) and SET by `checkOrder` (`:475`). RULE 11c: retail
/// `dSv_event_c` indexes `mFlags` BYTE (`d_save.cpp:1187-1204`), so 0x3D02 is already the
/// `hi_byte << 8 | mask` form and is passed straight through.
const EVENT_BIT_APPEARED: u32 = 0x3D02; // <- d_save_event_flag.inc:471

pub fn is_warpgn(name: &str) -> bool {
    WARPGN_ACTOR == name
}

/// Access set per parameter, for RULE 9(d):
/// * `switches` - `Res<Switches>`, shared resource, reads `Switches`.
/// * `flags` - `ResMut<EventFlags>`, exclusive resource, reads and writes `EventFlags`.
/// * `status` - `Res<PlayerStatus>`, shared resource.
/// * `ships` - `Query<(&Ship, &Actor), Without<Warpgn>>`, reads `Ship` and `Actor`.
/// * `manager` - `ResMut<EventManager>`, exclusive resource.
/// * `info` - `Res<MsgGameInfo>`, shared resource.
/// * `events` - `ResMut<SimEvents>`, exclusive resource.
/// * `points` - `Query<(&Actor, &mut Warpgn)>` (expanded from `Query<&mut Warpgn>` because
///   `check_warp` reads `current.pos`), reads `Actor`, writes `Warpgn`.
///
/// `points` reads `Actor` and `ships` reads `Actor`, both shared, so the pair is not ambiguous on
/// its own - but `points` carries `Warpgn` while `ships` does not, and `ships` is additionally
/// `Without<Warpgn>`, which makes the two provably disjoint under RULE 9(d) in both directions.
/// No param writes `Actor`, so the shared reads cannot collide.
pub fn control(
    switches: Res<Switches>,
    mut flags: ResMut<EventFlags>,
    status: Res<PlayerStatus>,
    ships: Query<(&Ship, &Actor), Without<Warpgn>>,
    mut manager: ResMut<EventManager>,
    info: Res<MsgGameInfo>,
    mut events: ResMut<SimEvents>,
    mut points: Query<(&Actor, &mut Warpgn)>,
) {
    let ship = ships.iter().next();
    for (actor, mut point) in &mut points {
        if !point.entered {
            point.create_init(&switches, &flags, &manager);
        }
        point.execute(
            actor,
            &switches,
            &mut flags,
            &status,
            ship,
            &mut manager,
            &info,
            &mut events,
        );
    }
}
