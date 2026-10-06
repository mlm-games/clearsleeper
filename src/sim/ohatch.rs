use bevy_ecs::prelude::*;
use glam::Vec3;

use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::vibration::Vibration;
use super::walk;

/// `OBJNAME("Ohatch", fpcNm_Obj_Ohatch_e, 255, 0)` (`d_stage.cpp:1128`) - the OBJNAME table is the
/// authority. `l_arcname[] = "Ohatch"` (`d_a_obj_ohatch.cpp:15`) happens to match, unlike
/// `Lwood`/`lwood` and `Vpbot`/`Vpbot_00`.
pub const OHATCH_ACTORS: [&str; 1] = ["Ohatch"];

/// `enum Param_e { PRM_SWSAVE_W = 8, PRM_SWSAVE_S = 0 }` (`d_a_obj_ohatch.h:10-13`), fed through
/// `daObj::PrmAbstract` (`((1 << width) - 1) & (param >> shift)`, `d_a_obj.h:14-17`).
const SWSAVE_WIDTH: u32 = 0x08; // <- d_a_obj_ohatch.h:11 (PRM_SWSAVE_W)
const SWSAVE_SHIFT: u32 = 0x00; // <- d_a_obj_ohatch.h:12 (PRM_SWSAVE_S)

/// The profile's `fopAc_CULLBOX_5_e` (`d_a_obj_ohatch.cpp:368`) resolves through
/// `l_cullSizeBox[5] = fopAc_cullSizeBox(cXyz(-125.0f, 0.0f, -125.0f), cXyz(125.0f, 250.0f, 125.0f))`
/// (`f_op_actor_mng.cpp:563`). `daObjOhatch_c` never calls `fopAcM_setCullSizeBox`, so that table
/// entry is the ONLY box it has.
const CULL_MIN: [f32; 3] = [-125.0, 0.0, -125.0]; // <- f_op_actor_mng.cpp:563 (fopAc_CULLBOX_5_e)
const CULL_MAX: [f32; 3] = [125.0, 250.0, 125.0]; // <- f_op_actor_mng.cpp:563 (fopAc_CULLBOX_5_e)

/// `namespace L_HIO` (`d_a_obj_ohatch.cpp:62-76`), which `#define HIO(name) L_HIO::name`
/// (`m_Do_hostIO.h:18`) selects for a retail build - the other arm is the `#if VERSION == VERSION_DEMO`
/// `daObjOhatch_HIO_c` (`:19-60`), whose constructor (`:45-60`) writes the identical values. RULE 0's
/// NTSC-U ordering picks `L_HIO`, so every `HIO(...)` below is a compile-time constant.
/// `tremor_act_proc`: `m2EC += (s32)HIO(m20);` (`:233`) and `if (m2EC > (s32)HIO(m24))` (`:234`).
const HIO_M20: f32 = 0x2000_u32 as f32; // <- d_a_obj_ohatch.cpp:69 (L_HIO::m20)
const HIO_M24: f32 = 0x20000_u32 as f32; // <- d_a_obj_ohatch.cpp:70 (L_HIO::m24)
/// `tremor_act_proc`: `f32 fVar1 = cM_ssin(m2EC); if (fVar1 < 0.0f) { fVar1 *= -1.0f; }
/// m2E8 = fVar1 * HIO(m28);` (`:238-242`).
const HIO_M28: f32 = 6.0; // <- d_a_obj_ohatch.cpp:71 (L_HIO::m28)
/// `open_act_proc`: `s32 uVar2 = (cM_ssin(sin) + HIO(m1C)) / (HIO(m1C) + 1.0f) * HIO(m08);`
/// (`:252-253`) and `if (m2D8 <= (s16)HIO(m0C))` (`:257-258`).
const HIO_M0C: f32 = -(0x4000_u32 as f32); // <- d_a_obj_ohatch.cpp:64 (L_HIO::m0C)
const HIO_M1C: f32 = 1.0; // <- d_a_obj_ohatch.cpp:68 (L_HIO::m1C)
const HIO_M08: f32 = -(0x1000_u32 as f32); // <- d_a_obj_ohatch.cpp:63 (L_HIO::m08)
/// `vibrate_act_proc`: `m2F0 += (s32)HIO(m2C);` (`:268`) and `m2F2 += (s32)HIO(m34);` (`:272`);
/// `open_act_proc`'s `m2F0 = (s16)HIO(m30);` (`:260`).
const HIO_M2C: f32 = -64.0; // <- d_a_obj_ohatch.cpp:72 (L_HIO::m2C)
const HIO_M30: f32 = (0x400_u32 as f32); // <- d_a_obj_ohatch.cpp:73 (L_HIO::m30)
const HIO_M34: f32 = (0x2000_u32 as f32); // <- d_a_obj_ohatch.cpp:74 (L_HIO::m34)

/// `m29C = param_get_swSave(); if (m29C != 0xff && fopAcM_isSwitch(this, m29C) == TRUE)`
/// (`d_a_obj_ohatch.cpp:136-137`, `:200`). `dSv_info_c::isSwitch` returns FALSE for `i_no == 255`
/// (`d_save.cpp:1591-1593`) as well as for `-1`, and retail guards the index with an explicit
/// `!= 0xff` test first - both halves are transcribed.
const NO_SWITCH: u8 = 0xFF; // <- d_a_obj_ohatch.cpp:137
/// `_create`: `m2D8 = -0x4000;` (`:138`) - the hinge angle when the switch is ALREADY on, i.e. the
/// fully-open pose. Negative hex form per RULE 9b.
const ANGLE_OPEN: i16 = -0x4000; // <- d_a_obj_ohatch.cpp:138
/// `open_act_proc`: `f32 sin = m2D8 / HIO(m0C) * 0x8000;` (`:252`) - a HALF turn in `f32`, fed to
/// `cM_ssin`, whose parameter is an `s16` (`c_math.h:39-41`), so it is truncated on the call.
const HALF_TURN: f32 = 0x8000; // <- d_a_obj_ohatch.cpp:252
/// `dComIfGp_evmng_existence("FIGURE_HATCH_OPEN")` (`:201`).
const EVENT_OPEN: &str = "FIGURE_HATCH_OPEN"; // <- d_a_obj_ohatch.cpp:201
/// `dComIfGp_evmng_getMyStaffId("Ohatch")` (`:202`).
const STAFF_HATCH: &str = "Ohatch"; // <- d_a_obj_ohatch.cpp:202
/// `strcmp(dComIfGp_getPEvtManager()->getMyNowCutName(staffIdx), "OPEN") == 0` (`:204`).
const CUT_OPEN: &str = "OPEN"; // <- d_a_obj_ohatch.cpp:204
/// `fopAcM_seStartCurrent(this, JA_SE_OBJ_FIG_HATCH_OPEN, 0)` (`:205`, `:219`);
/// `JA_SE_OBJ_FIG_HATCH_OPEN = 0x6A01` (`JAZelAudio_SE.h:2024`).
const SE_FIG_HATCH_OPEN: u32 = 0x6A01; // <- JAZelAudio_SE.h:2024
/// `mDoAud_seStart(JA_SE_READ_RIDDLE_1)` (`:259`); `JA_SE_READ_RIDDLE_1 = 0x0806`
/// (`JAZelAudio_SE.h:18`).
const SE_READ_RIDDLE_1: u32 = 0x0806; // <- JAZelAudio_SE.h:18
/// `dComIfGp_getVibration().StartShock(3, 1, cXyz(0.0f, 1.0f, 0.0f))` (`:213`, `:226`).
const TRIGGER_PATTERN: i32 = 3; // <- d_a_obj_ohatch.cpp:213
const TRIGGER_FLAGS: i32 = 1; // <- d_a_obj_ohatch.cpp:213
const TRIGGER_COORD: [f32; 3] = [0.0, 1.0, 0.0]; // <- d_a_obj_ohatch.cpp:213
/// `dComIfGp_getVibration().StartShock(6, 1, cXyz(0.0f, 1.0f, 0.0f))` (`:261`).
const SETTLE_PATTERN: i32 = 6; // <- d_a_obj_ohatch.cpp:261
const SETTLE_FLAGS: i32 = 1; // <- d_a_obj_ohatch.cpp:261
const SETTLE_COORD: [f32; 3] = [0.0, 1.0, 0.0]; // <- d_a_obj_ohatch.cpp:261

/// The Nintendo Gallery hatch - `daObjOhatch_c` (`d_a_obj_ohatch.h:14-46`), the actor behind the
/// stage string `Ohatch`.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daObjOhatch_c` is a plain `fopAc_ac_c`
/// (`d_a_obj_ohatch.h:16`) holding one `J3DModel*`, two `dBgW*` and an `Mtx` (`:38-42`); it registers
/// no `dCcD` collider of any kind. The two `dBgW` are moving backgrounds (RULE 3b) built from the
/// `OH_CLS` / `OH_OPN` DZBs (`:120-121`), and the gate the player walks through is that
/// background's collision, not a `dCcD` collider. The profile `Group` is `fopAc_ACTOR_e` (`:367`).
/// Neither `combat::HitPoints` nor `enemy::Hurt` is attached - attaching them would let
/// `combat::control` despawn a hatch retail cannot damage.
///
/// `daObjOhatch_IsDelete` (`:342-344`) is the bare "accepts a delete request" form of RULE 6b, never
/// a self-destruct. `_create` (`:130-161`) can return `cPhs_ERROR_e` only from
/// `fopAcM_entrySolidHeap` (`:151`) or from `dComIfG_Bgsp()->Regist` returning nonzero (`:141`,
/// `:146`) - heap and background-registration work, neither modelled - so `spawn` never refuses.
/// There is no BCK frame read anywhere in this file, so no frame count is invented.
///
/// ## What is cosmetic and what is not
/// The state machine drives two things this sim can hear or feel plus a model matrix:
/// `fopAcM_seStartCurrent(JA_SE_OBJ_FIG_HATCH_OPEN, 0)` (`:205`, `:219`), the three
/// `dComIfGp_getVibration().StartShock` calls (`:213`, `:226`, `:261`) and the third sound
/// `mDoAud_seStart(JA_SE_READ_RIDDLE_1)` (`:259`) are all transcribed. The hinge angle `m2D8`, the
/// vertical jolt `m2E8`, the jolt phase `m2EC` and the settle pair `m2F0` / `m2F2` are read ONLY by
/// `set_mtx`'s `mDoMtx_stack_c::XrotM(tmp)` and `transS(pos.x, pos.y + m2E8, pos.z)` (`:81-87`) - the
/// J3D base matrix. `current.pos` never changes and `m2A8`, the matrix handed to both `dBgW`s, is
/// written once by `init_mtx` (`:103`) and never again - so retail's hatch does not move its own
/// collision either. Those five fields are carried because they ARE the state machine's inputs, but
/// outside this module they mean nothing but "which stage of the opening is running", which is said
/// here rather than hidden.
///
/// RULE 2: `init_mtx`'s `mModel->setBaseScale(scale)` / `scaleM(scale)` (`:93`, `:102`) and
/// `set_mtx`'s `transM(0.0f, -HIO(m14), -HIO(m18))` (`:85`, `:87`) are renderer work, and the
/// stage-placed scale is exactly 1.0, so nothing is lost.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ohatch {
    /// `m29C` (`d_a_obj_ohatch.h:37`, an `s32`) from `param_get_swSave()` (`:136`), the
    /// `daObj::PrmAbstract(this, 8, 0)` index `close_wait_act_proc` re-tests at `:200`.
    pub switch_no: u8,
    /// `mActionFunc` (`d_a_obj_ohatch.h:39`) - which of the five `*_act_proc` bodies runs, assigned
    /// at `:139`, `:144`, `:214`, `:227`, `:236`, `:262`, `:270`.
    pub action: Action,
    /// `m2D8` (`d_a_obj_ohatch.h:38`, an `s16`) - the hinge angle: `ANGLE_OPEN` when the switch is
    /// already on (`:138`), otherwise the `fopAcM_ct` zero-fill, walked to `HIO(m0C)` by `open_act_proc`.
    pub angle: i16,
    /// `m2E8` (`d_a_obj_ohatch.h:40`, an `f32`) - the vertical jolt offset, set by `tremor_act_proc`
    /// (`:242`) and zeroed at `:235`.
    pub jolt: f32,
    /// `m2EC` (`d_a_obj_ohatch.h:41`, an `s32`) - the jolt phase, incremented by `HIO(m20)` at `:233`
    /// and tested against `HIO(m24)` at `:234`.
    pub jolt_phase: i32,
    /// `m2F0` (`d_a_obj_ohatch.h:42`, an `s16`) - the settle amplitude, armed to `HIO(m30)` at `:260`
    /// and walked down by `HIO(m2C)` at `:268`.
    pub settle: i16,
    /// `m2F2` (`d_a_obj_ohatch.h:43`, an `s16`) - the settle phase, incremented by `HIO(m34)` at
    /// `:272`.
    pub settle_phase: i16,
    /// One-shot guard so `_create`'s `m2D8 = -0x4000` arm and its two `dBgW` `Regist` choices
    /// (`:136-148`) happen once, on the first tick, where `Switches` is available.
    pub entered: bool,
}

/// `typedef void (daObjOhatch_c::*ActionFunc)();` (`d_a_obj_ohatch.h:15`) and its five members, in the
/// order they are declared at `d_a_obj_ohatch.h:34-38`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// `daObjOhatch_c::close_wait_act_proc` (`d_a_obj_ohatch.cpp:198-229`) - `_create`'s default arm
    /// (`:144`).
    CloseWait,
    /// `daObjOhatch_c::tremor_act_proc` (`:231-244`) - entered at `:214` and `:227`.
    Tremor,
    /// `daObjOhatch_c::open_act_proc` (`:246-264`) - entered at `:236`.
    Open,
    /// `daObjOhatch_c::vibrate_act_proc` (`:266-274`) - entered at `:262`.
    Vibrate,
    /// `daObjOhatch_c::open_wait_act_proc` (`:276-296`) - entered at `:270` and `_create`'s other arm
    /// (`:139`).
    OpenWait,
}

impl Ohatch {
    /// `_create` (`:130-161`) minus `fopAcM_ct` (`:131`), `dComIfG_resLoad` (`:132`),
    /// `fopAcM_entrySolidHeap` (`:134`), `fopAcM_SetMtx` (`:135`), the two `dBgW` `Regist` calls
    /// (`:140`, `:145`), `init_mtx` (`:149`) and the `#if VERSION == VERSION_DEMO` HIO-child block
    /// (`:155-159`) - resource, heap, background, matrix and HIO work.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            switch_no: ((parameters >> SWSAVE_SHIFT) & ((1 << SWSAVE_WIDTH) - 1)) as u8,
            // `mActionFunc` (`d_a_obj_ohatch.h:39`) has no initialiser, so `fopAcM_ct`'s clear-area
            // zero-fill (`f_pc_base.cpp:98`) makes it NULL; `_create` overwrites it at `:139`/`:144`
            // before the first `Execute`, so `CloseWait` is only the documented placeholder.
            action: Action::CloseWait,
            // `m2D8` (`d_a_obj_ohatch.h:38`) is likewise zero-filled, and `_create`'s open arm sets
            // `ANGLE_OPEN` at `:138` on the first tick.
            angle: 0,
            jolt: 0.0,
            jolt_phase: 0,
            settle: 0,
            settle_phase: 0,
            entered: false,
        }
    }

    /// `l_cullSizeBox[5]` for the profile's `fopAc_CULLBOX_5_e` (`f_op_actor_mng.cpp:563`) - the only
    /// box this actor has, since it never calls `fopAcM_setCullSizeBox`.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `_create`'s switch test and its two action assignments (`:136-148`).
    fn create_init(&mut self, switches: &Switches) {
        // `if (m29C != 0xff && fopAcM_isSwitch(this, m29C) == TRUE) { m2D8 = -0x4000;
        // mActionFunc = &open_wait_act_proc; ... } else { mActionFunc = &close_wait_act_proc; ... }`
        // (:137-148). `fopAcM_isSwitch` (`f_op_actor_mng.h:374-376`) is `Switches::test`, and the `&&`
        // short-circuits first, so index 0xFF never reaches it - the same FALSE
        // `dSv_info_c::isSwitch` would have returned (`d_save.cpp:1591-1593`).
        //
        // RULE 11b: an index >= 0x80 routes to the per-zone `dSv_zoneBit_c` arrays
        // (`d_save.h:982-984`, `d_save.cpp:1588-1598`), so such a hatch reads false here. The real
        // index is carried through unchanged, never clamped.
        if self.switch_no != NO_SWITCH && switches.test(self.switch_no) {
            self.angle = ANGLE_OPEN;
            self.action = Action::OpenWait;
        } else {
            self.action = Action::CloseWait;
        }
        self.entered = true;
    }

    /// `daObjOhatch_c::_execute` (`:298-311`) minus `set_mtx()` (`:308`) and the two `dBgW->Move()`
    /// calls (`:300-306`) - matrix and moving-background work - leaving the
    /// `(this->*mActionFunc)();` dispatch (`:309`).
    fn execute(
        &mut self,
        switches: &Switches,
        list: &EventList,
        active: &ActiveEvent,
        events: &mut SimEvents,
        vibration: &mut Vibration,
    ) {
        match self.action {
            Action::CloseWait => self.close_wait(switches, list, active, events, vibration),
            Action::Tremor => self.tremor(),
            Action::Open => self.open(events, vibration),
            Action::Vibrate => self.vibrate(),
            // `daObjOhatch_c::open_wait_act_proc` (`d_a_obj_ohatch.cpp:276-296`) is an EMPTY function
            // in a retail build: its whole body, including the closing test and the five resets, is
            // inside `#if VERSION == VERSION_DEMO` (`:278-295`). RULE 0's NTSC-U ordering excludes it,
            // and `HIO(m38) == 0` would have been false anyway (`L_HIO::m38 = 0`, `:75`) - so the arm
            // is a genuine no-op and a retail hatch never closes again once open.
            Action::OpenWait => {}
        }
    }

    /// `daObjOhatch_c::close_wait_act_proc` (`:198-229`).
    fn close_wait(
        &mut self,
        switches: &Switches,
        list: &EventList,
        active: &ActiveEvent,
        events: &mut SimEvents,
        vibration: &mut Vibration,
    ) {
        // `if (m29C != 0xff && fopAcM_isSwitch(this, m29C) == TRUE) {` (:200)
        if self.switch_no == NO_SWITCH || !switches.test(self.switch_no) {
            // `} else if (HIO(m38) == 1) {` (:218) - `L_HIO::m38` is `static const u8 = 0`
            // (`d_a_obj_ohatch.cpp:75`), a compile-time constant in retail, so that arm is dead.
            // Both blocks it guards are the same `dBgW` Release / Regist pair as `:206-211`, so
            // folding the test to a literal false loses nothing.
            return;
        }
        // `if (dComIfGp_evmng_existence("FIGURE_HATCH_OPEN")) {` (:201) - an event-list membership
        // test, i.e. `EventList::event_index`.
        if list.event_index(EVENT_OPEN).is_none() {
            return;
        }
        // `s32 staffIdx = dComIfGp_evmng_getMyStaffId("Ohatch"); if (staffIdx != -1) {` (:202-203).
        // `getMyStaffId` (`d_event_manager.cpp:400-421`) only accepts an event whose
        // `mEventState` is `PLAY_e || CLOSE_e || UNK3_e` (`:412`; the enum is
        // `{ NONE_e, ORDER_e, PLAY_e, UNK3_e, CLOSE_e }`, `d_event_data.h:136-142`) and returns -1
        // when none carries the staff - which is `staff_cut`'s `None`. `Stage` in `event.rs` models
        // only three states, so `staff_cut` also admits `Order` where retail would not; that gap
        // lives in `event.rs`, not here, and is the same one `shutter2.rs` records for
        // `getMyActIdx`.
        let Some((cut, _)) = active.staff_cut(list, STAFF_HATCH) else {
            return;
        };
        // `if (strcmp(dComIfGp_getPEvtManager()->getMyNowCutName(staffIdx), "OPEN") == 0)` (:204) -
        // the cut currently playing on that staff.
        if list.cut_name(cut) == Some(CUT_OPEN) {
            self.trigger(events, vibration);
        }
    }

    /// `fopAcM_seStartCurrent(this, JA_SE_OBJ_FIG_HATCH_OPEN, 0)` (`:205`, `:219`), the `dBgW`
    /// Release / Regist pair (`:206-211`, `:220-225`),
    /// `dComIfGp_getVibration().StartShock(3, 1, cXyz(0.0f, 1.0f, 0.0f))` (`:213`, `:226`) and
    /// `mActionFunc = &daObjOhatch_c::tremor_act_proc` (`:214`, `:227`). The two call sites are
    /// textually identical, so they share one body.
    fn trigger(&mut self, events: &mut SimEvents, vibration: &mut Vibration) {
        events.list.push(SimEvent::Se(SE_FIG_HATCH_OPEN));
        vibration.start_shock(TRIGGER_PATTERN, TRIGGER_FLAGS, Vec3::from_array(TRIGGER_COORD));
        self.action = Action::Tremor;
    }

    /// `daObjOhatch_c::tremor_act_proc` (`:231-244`).
    fn tremor(&mut self) {
        // `m2EC += (s32)HIO(m20);` (:233)
        self.jolt_phase += HIO_M20 as i32;
        // `if (m2EC > (s32)HIO(m24)) { m2E8 = 0.0f; mActionFunc = &open_act_proc; }` (:234-236)
        if self.jolt_phase > HIO_M24 as i32 {
            self.jolt = 0.0;
            self.action = Action::Open;
            return;
        }
        // `f32 fVar1 = cM_ssin(m2EC);` (:238). `cM_ssin` takes an `s16` (`c_math.h:39-41`) and
        // `m2EC` is an `s32` (`d_a_obj_ohatch.h:41`), so the call truncates to the low 16 bits -
        // which matters, because `m2EC` runs 0 .. 0x20000 and the top bit's worth of travel is
        // invisible to the sine. `if (fVar1 < 0.0f) { fVar1 *= -1.0f; }
        // m2E8 = fVar1 * HIO(m28);` (:239-242)
        let swing = walk::from_s16(self.jolt_phase as i16).sin();
        self.jolt = swing.abs() * HIO_M28;
    }

    /// `daObjOhatch_c::open_act_proc` (`:246-264`), retail-NTSC arm only (`:252-253`; the
    /// `#if VERSION == VERSION_DEMO` arm at `:249-250` is excluded by RULE 0's ordering).
    fn open(&mut self, events: &mut SimEvents, vibration: &mut Vibration) {
        // `f32 sin = m2D8 / HIO(m0C) * 0x8000;` (:252) - the local is named `sin` but holds radians.
        // `cM_ssin`'s parameter is an `s16` (`c_math.h:39-41`), so the `f32` is truncated on the
        // call. RULE 5b's one conversion path is `walk::from_s16`.
        let arg = (f32::from(self.angle) / HIO_M0C * HALF_TURN) as i16;
        // `s32 uVar2 = (cM_ssin(sin) + HIO(m1C)) / (HIO(m1C) + 1.0f) * HIO(m08);` (:253)
        let step =
            (walk::from_s16(arg).sin() + HIO_M1C) / (HIO_M1C + 1.0) * HIO_M08;
        // `m2D8 += uVar2;` (:255) - `m2D8` is an `s16`, so the `s32` step truncates.
        self.angle = self.angle.wrapping_add(step as i16);
        // `if (m2D8 <= (s16)HIO(m0C)) {` (:257) - the hinge has reached the open stop.
        if self.angle <= HIO_M0C as i16 {
            // `m2D8 = (s16)HIO(m0C);` (:258)
            self.angle = HIO_M0C as i16;
            // `mDoAud_seStart(JA_SE_READ_RIDDLE_1);` (:259)
            events.list.push(SimEvent::Se(SE_READ_RIDDLE_1));
            // `m2F0 = (s16)HIO(m30);` (:260)
            self.settle = HIO_M30 as i16;
            // `dComIfGp_getVibration().StartShock(6, 1, cXyz(0.0f, 1.0f, 0.0f));` (:261)
            vibration.start_shock(SETTLE_PATTERN, SETTLE_FLAGS, Vec3::from_array(SETTLE_COORD));
            // `mActionFunc = &daObjOhatch_c::vibrate_act_proc;` (:262)
            self.action = Action::Vibrate;
        }
    }

    /// `daObjOhatch_c::vibrate_act_proc` (`:266-274`).
    fn vibrate(&mut self) {
        // `m2F0 += (s32)HIO(m2C);` (:268) - truncated to the `s16` field.
        self.settle = self.settle.wrapping_add(HIO_M2C as i16);
        // `if (m2F0 < 0) { mActionFunc = &open_wait_act_proc; } else { m2F2 += (s32)HIO(m34); }`
        // (:269-273)
        if self.settle < 0 {
            self.action = Action::OpenWait;
        } else {
            self.settle_phase = self.settle_phase.wrapping_add(HIO_M34 as i16);
        }
    }
}

pub fn is_ohatch(name: &str) -> bool {
    OHATCH_ACTORS.contains(&name)
}

pub fn control(
    switches: Res<Switches>,
    list: Res<EventList>,
    active: Res<ActiveEvent>,
    mut events: ResMut<SimEvents>,
    mut vibration: ResMut<Vibration>,
    mut hatches: Query<&mut Ohatch>,
) {
    for mut hatch in &mut hatches {
        let hatch = &mut *hatch;
        if !hatch.entered {
            hatch.create_init(&switches);
        }
        hatch.execute(&switches, &list, &active, &mut events, &mut vibration);
    }
}
