use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList, Stage};
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, Switches};
use super::vibration::Vibration;

/// The stage string is `Hbrf1` (`d_stage.cpp:874`, `fpcNm_Obj_Hbrf1_e`). It matches
/// `daObjHbrf1::Act_c::M_arcname`, which is also `"Hbrf1"` (`d_a_obj_hbrf1.cpp:13`).
pub const HBRF1_ACTORS: [&str; 1] = ["Hbrf1"];

/// `enum Mode_e { Mode_DOWN_STOP_e, Mode_UP_DEMO_WAIT_e, Mode_UP_DEMO_TIMER_e, Mode_UP_DEMO_e,
/// Mode_UP_STOP_e, Mode_DOWN_DEMO_WAIT_e, Mode_DOWN_DEMO_TIMER_e, Mode_DOWN_DEMO_e }`
/// (`d_a_obj_hbrf1.h:13-22`) - the decomp's own order, which the `switch (mMode)` of
/// `Execute` (`d_a_obj_hbrf1.cpp:212-237`) mirrors.
const MODE_DOWN_STOP: u8 = 0; // <- d_a_obj_hbrf1.h:14
const MODE_UP_DEMO_WAIT: u8 = 1; // <- d_a_obj_hbrf1.h:15
const MODE_UP_DEMO_TIMER: u8 = 2; // <- d_a_obj_hbrf1.h:16
const MODE_UP_DEMO: u8 = 3; // <- d_a_obj_hbrf1.h:17
const MODE_UP_STOP: u8 = 4; // <- d_a_obj_hbrf1.h:18
const MODE_DOWN_DEMO_WAIT: u8 = 5; // <- d_a_obj_hbrf1.h:19
const MODE_DOWN_DEMO_TIMER: u8 = 6; // <- d_a_obj_hbrf1.h:20
const MODE_DOWN_DEMO: u8 = 7; // <- d_a_obj_hbrf1.h:21

/// `daObj::PrmAbstract<T>(actor, width, shift) = ((1 << width) - 1) & (fopAcM_GetParam(actor) >>
/// shift)` (`d_a_obj.h:12-16`), fed by `enum Prm_e { PRM_SWSAVE_W = 0x08, PRM_SWSAVE_S = 0x00,
/// PRM_TYPE_W = 0x01, PRM_TYPE_S = 0x08, PRM_EVENT_W = 0x08, PRM_EVENT_S = 0x10 }`
/// (`d_a_obj_hbrf1.h:24-29`). `fopAcM_GetParam` returns the WHOLE u32 (RULE 7).
const SWSAVE_SHIFT: u32 = 0x00; // <- d_a_obj_hbrf1.h:25 (PRM_SWSAVE_S)
const SWSAVE_WIDTH: u32 = 0x08; // <- d_a_obj_hbrf1.h:24 (PRM_SWSAVE_W)
const TYPE_SHIFT: u32 = 0x08; // <- d_a_obj_hbrf1.h:27 (PRM_TYPE_S)
const TYPE_WIDTH: u32 = 0x01; // <- d_a_obj_hbrf1.h:26 (PRM_TYPE_W)
const EVENT_SHIFT: u32 = 0x10; // <- d_a_obj_hbrf1.h:29 (PRM_EVENT_S)
const EVENT_WIDTH: u32 = 0x08; // <- d_a_obj_hbrf1.h:28 (PRM_EVENT_W)

/// `fopAcM_setCullSizeBox(this, -800.0f, -1000.0f, -800.0f, 800.0f, 1000.0f, 800.0f)`
/// (`d_a_obj_hbrf1.cpp:41`) - the actor's one real custom cull box, matching the profile's
/// `fopAc_CULLBOX_CUSTOM_e` (`:310`).
const CULL_MIN: [f32; 3] = [-800.0, -1000.0, -800.0]; // <- d_a_obj_hbrf1.cpp:41
const CULL_MAX: [f32; 3] = [800.0, 1000.0, 800.0]; // <- d_a_obj_hbrf1.cpp:41

/// `mYOffset = 10.0f` (`d_a_obj_hbrf1.cpp:42`) - the per-tick travel added by
/// `daObjHbrf1_up_demo` (`:131`) and subtracted by `daObjHbrf1_down_demo` (`:191`).
const Y_OFFSET: f32 = 10.0; // <- d_a_obj_hbrf1.cpp:42

/// `current.pos.y += 750.0f` (`d_a_obj_hbrf1.cpp:31` and `:34`), and the two
/// `home.pos.y + 750.0f` stops at `:133-134` and `:193-194`.
const TRAVEL: f32 = 750.0; // <- d_a_obj_hbrf1.cpp:31

/// `mTimer = 30` (`d_a_obj_hbrf1.cpp:115` and `:172`) - the demo hold that
/// `daObjHbrf1_{up,down}_demo_timer` (`:122`, `:182`) drains one per tick.
const DEMO_HOLD: i32 = 30; // <- d_a_obj_hbrf1.cpp:115

/// `dComIfGp_evmng_getEventIdx("LiftMove")` (`d_a_obj_hbrf1.cpp:43`) into `mEventIdx`
/// (`d_a_obj_hbrf1.h:35`). The one-argument form defaults `evNo` to `-1` (`0xFF`,
/// `d_com_inf_game.h:3611`), which skips the stage event-info branch of `getEventIdx`
/// (`d_event_manager.cpp:347-348`), so it is a pure name lookup.
const LIFT_MOVE_EVENT: &str = "LiftMove"; // <- d_a_obj_hbrf1.cpp:43

/// `dComIfGs_isEventBit` / `dComIfGs_onEventBit` with these `dSv_event_flag_c` ids
/// (`d_a_obj_hbrf1.cpp:96-97`, `:101-102`, `:153-154`, `:158-159`). Under RULE 11c they are
/// byte-indexed, so the retail ids are passed through unshifted; all four are <= 0xFF07.
const EVT_1540: u32 = 0x1540; // <- d_save_event_flag.inc:159
const EVT_1510: u32 = 0x1510; // <- d_save_event_flag.inc:161
const EVT_1520: u32 = 0x1520; // <- d_save_event_flag.inc:160
const EVT_1508: u32 = 0x1508; // <- d_save_event_flag.inc:162

/// `fopAcM_seStart(this, JA_SE_OBJ_B_LIFT_STOP, 0)` (`d_a_obj_hbrf1.cpp:142` and `:202`).
const SE_LIFT_STOP: u32 = 0x69C9; // <- JAZelAudio_SE.h:1977

/// `fopAcM_seStart(this, JA_SE_OBJ_B_LIFT_MOVE, 0)` (`d_a_obj_hbrf1.cpp:147` and `:207`).
const SE_LIFT_MOVE: u32 = 0x61C8; // <- JAZelAudio_SE.h:1701

/// `dComIfGp_getVibration().StartShock(4, -0x21, cXyz(0.0f, 1.0f, 0.0f))`
/// (`d_a_obj_hbrf1.cpp:143` and `:203`), against `StartShock` in `d_vibration.cpp:211-238`.
/// `-0x21 & MASK_CAMERA_SHAKE` (`0x3E`) and `-0x21 & FLAG_MOTOR_SHAKE` (`0x1`) are both non-zero
/// (`src/sim/vibration.rs:7`, `:8`), so both channels fire - the same call `kamome.rs` makes with
/// pattern 5.
const SHOCK_PATTERN: i32 = 4; // <- d_a_obj_hbrf1.cpp:143
const SHOCK_FLAGS: i32 = -0x21; // <- d_a_obj_hbrf1.cpp:143
const SHOCK_COORD: [f32; 3] = [0.0, 1.0, 0.0]; // <- d_a_obj_hbrf1.cpp:143

/// The giant-fan-room elevator of Wind Temple - `daObjHbrf1::Act_c` (`d_a_obj_hbrf1.h:9-52`), the
/// actor behind the stage string `Hbrf1`.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daObjHbrf1::Act_c` is a
/// `dBgS_MoveBgActor` holding a phase request, a model, the mode, the event index, the timer and
/// `m2E4` (`d_a_obj_hbrf1.h:44-51`); it registers no `dCcD` collider. `MoveBGCreate` (`:57`)
/// builds a `dBgW` moving background (RULE 3b) from the `Hbrf1` DZB, which is not a `dCcD`
/// collider. The profile `Group` is `fopAc_ACTOR_e` (`:310`). Neither `combat::HitPoints` nor
/// `enemy::Hurt` is attached.
///
/// `Mthd_IsDelete` is `dBgS_MoveBgActor::MoveBGIsDelete` (`:282-284`), which is
/// `return IsDelete()` (`d_bg_s_movebg_actor.h:28`) and the class overrides neither - the base
/// `IsDelete()` is `return TRUE` (`d_bg_s_movebg_actor.h:35`). RULE 6b: this is "accepts a delete
/// request", never a self-destruct. `Mthd_Create` (`:51-62`) can return `cPhs_ERROR_e` only from
/// `MoveBGCreate` (`:57`), which is heap work.
///
/// `daObjHbrf1::Act_c::M_evname` is `"ami_cam"` (`:14`), the camera-event name the Wind Temple
/// fan uses; nothing in `d_a_obj_hbrf1.cpp` reads it, so it is not stored.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Hbrf1 {
    /// `mMode` (`d_a_obj_hbrf1.h:47`), the `Execute` switch index (`:212-237`).
    pub mode: u8,
    /// `mTimer` (`d_a_obj_hbrf1.h:48`, an `s32`), set to 30 by the two `*_demo_wait` arms and
    /// drained by the two `*_demo_timer` arms (`:122`, `:182`).
    pub timer: i32,
    /// `mYOffset` (`d_a_obj_hbrf1.h:44`), the 10.0 per-tick travel. Fixed at create (`:42`) and
    /// never reassigned.
    pub y_offset: f32,
    /// `m2E4` (`d_a_obj_hbrf1.h:49`, a `u8`) - "this actor ordered the demo", `0` at create
    /// (`:44`), `1` when a `*_stop` arm orders the event (`:99`, `:104`, `:156`, `:161`) and back
    /// to `0` when `*_demo` finishes (`:138`, `:198`). It gates
    /// `dComIfGp_event_reset()`, so it must be carried.
    pub ordered: u8,
    /// `home.pos` (`f_op_actor.h:288`), the placement position, which `daObjHbrf1_up_demo` and
    /// `daObjHbrf1_down_demo` measure against (`home.pos.y + 750.0f` at `:133-134`,
    /// `<= home.pos.y` at `:193`). `fopAc_Init` seeds both `current` and `old` from `home`
    /// (`f_op_actor.cpp:282-284`), so this is the spawn position.
    pub home: Vec3,
    /// `prm_get_Type()` (`d_a_obj_hbrf1.h:38`) - one bit at `(param >> 8) & 1`.
    pub kind: u8,
    /// `prm_get_swSave()` (`d_a_obj_hbrf1.h:37`) - eight bits at `param & 0xFF`, the switch the
    /// lift reads through `fopAcM_isSwitch`.
    pub switch_no: u8,
    /// `prm_get_Event()` (`d_a_obj_hbrf1.h:39`) - eight bits at `(param >> 16) & 0xFF`, which
    /// selects which of the two `dSv_event_flag_c` guards a `*_stop` arm uses.
    pub event_sel: u8,
    /// `mEventIdx` (`d_a_obj_hbrf1.h:35`); `None` is `getEventIdx`'s `-1`
    /// (`d_event_manager.cpp:355`).
    pub event_slot: Option<usize>,
    /// One-shot guard so `Create`'s `getEventIdx("LiftMove")` (`:43`) happens once.
    pub entered: bool,
}

impl Hbrf1 {
    /// `daObjHbrf1::Act_c::Create` (`:27-48`), minus `fopAcM_SetMtx` (`:28`), `init_mtx` (`:40`,
    /// which is `setBaseScale(scale)` + `set_mtx`) and `eyePos = home.pos` (`:45`, whose only
    /// readers are the sound emitters `fopAcM_seStart` passes it to, and the sim's `SimEvent::Se`
    /// carries an id alone). The mode choice of `:30-38` and `mYOffset = 10.0f` (`:42`) survive,
    /// along with the three parameter decodes.
    pub fn spawn(parameters: u32, home: Vec3) -> Self {
        Self {
            // `mMode` has no initialiser in `Create`; `fopAc_Init`'s `sBs_ClearArea`
            // (`f_pc_base.cpp:98`) zero-fill makes it `Mode_DOWN_STOP_e` (0). `Create`'s mode
            // choice needs `fopAcM_isSwitch`, so `create_init` re-decides on the first tick - the
            // `tagetc` shape, which cannot see `Switches` from `spawn`.
            mode: MODE_DOWN_STOP,
            timer: 0,
            y_offset: Y_OFFSET,
            // `m2E4 = 0;` (:44)
            ordered: 0,
            home,
            kind: ((parameters >> TYPE_SHIFT) & mask(TYPE_WIDTH)) as u8,
            switch_no: ((parameters >> SWSAVE_SHIFT) & mask(SWSAVE_WIDTH)) as u8,
            event_sel: ((parameters >> EVENT_SHIFT) & mask(EVENT_WIDTH)) as u8,
            event_slot: None,
            entered: false,
        }
    }

    /// `fopAcM_setCullSizeBox(this, -800.0f, -1000.0f, -800.0f, 800.0f, 1000.0f, 800.0f)`
    /// (`d_a_obj_hbrf1.cpp:41`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `daObjHbrf1::Act_c::Create`'s mode choice (`:30-38`) plus the `getEventIdx("LiftMove")`
    /// (`:43`) and `m2E4 = 0` (`:44`), split out because two of them need resources `spawn` cannot
    /// see.
    ///
    /// ```c
    /// if (prm_get_Type() == 0 && !fopAcM_isSwitch(this, prm_get_swSave())) {
    ///     current.pos.y += 750.0f; mMode = Mode_UP_STOP_e;
    /// } else if (prm_get_Type() == 1 && fopAcM_isSwitch(this, prm_get_swSave())) {
    ///     current.pos.y += 750.0f; mMode = Mode_UP_STOP_e;
    /// } else { mMode = Mode_DOWN_STOP_e; }
    /// ```
    fn create_init(&mut self, actor: &mut Actor, list: &EventList, switches: &Switches) {
        let up = (self.kind == 0 && !switches.test(self.switch_no))
            || (self.kind == 1 && switches.test(self.switch_no));
        if up {
            actor.pos.y += TRAVEL;
            self.mode = MODE_UP_STOP;
        } else {
            self.mode = MODE_DOWN_STOP;
        }
        self.event_slot = list.event_index(LIFT_MOVE_EVENT);
        self.entered = true;
    }

    /// `daObjHbrf1_down_stop` (`:94-110`).
    ///
    /// ```c
    /// if ((prm_get_Type() == 0 && !fopAcM_isSwitch(this, prm_get_swSave()))
    ///     || (prm_get_Type() != 0 && fopAcM_isSwitch(this, prm_get_swSave()))) {
    ///     if (prm_get_Event() == 0 && !dComIfGs_isEventBit(UNK_1540)) {
    ///         dComIfGs_onEventBit(UNK_1540);
    ///         fopAcM_orderOtherEventId(this, mEventIdx); m2E4 = 1; mMode = Mode_UP_DEMO_WAIT_e;
    ///     } else if (prm_get_Event() == 1 && !dComIfGs_isEventBit(UNK_1510)) {
    ///         dComIfGs_onEventBit(UNK_1510);
    ///         fopAcM_orderOtherEventId(this, mEventIdx); m2E4 = 1; mMode = Mode_UP_DEMO_WAIT_e;
    ///     } else { mMode = Mode_UP_DEMO_e; }
    /// }
    /// ```
    fn down_stop(&mut self, ctx: &mut Context<'_>) {
        let switched = ctx.switches.test(self.switch_no);
        if !((self.kind == 0 && !switched) || (self.kind != 0 && switched)) {
            return;
        }
        match self.event_sel {
            0 if !ctx.flags.test(EVT_1540) => {
                ctx.flags.set(EVT_1540);
                self.order(ctx);
                self.ordered = 1;
                self.mode = MODE_UP_DEMO_WAIT;
            }
            1 if !ctx.flags.test(EVT_1510) => {
                ctx.flags.set(EVT_1510);
                self.order(ctx);
                self.ordered = 1;
                self.mode = MODE_UP_DEMO_WAIT;
            }
            _ => self.mode = MODE_UP_DEMO,
        }
    }

    /// `daObjHbrf1_up_stop` (`:151-167`) - the exact mirror of `down_stop`, with
    /// `dSv_event_flag_c::UNK_1520` / `UNK_1508` and `Mode_DOWN_DEMO_WAIT_e`.
    fn up_stop(&mut self, ctx: &mut Context<'_>) {
        let switched = ctx.switches.test(self.switch_no);
        if !((self.kind == 0 && switched) || (self.kind != 0 && !switched)) {
            return;
        }
        match self.event_sel {
            0 if !ctx.flags.test(EVT_1520) => {
                ctx.flags.set(EVT_1520);
                self.order(ctx);
                self.ordered = 1;
                self.mode = MODE_DOWN_DEMO_WAIT;
            }
            1 if !ctx.flags.test(EVT_1508) => {
                ctx.flags.set(EVT_1508);
                self.order(ctx);
                self.ordered = 1;
                self.mode = MODE_DOWN_DEMO_WAIT;
            }
            _ => self.mode = MODE_DOWN_DEMO,
        }
    }

    /// `daObjHbrf1_up_demo_wait` (`:113-118`): `if (eventInfo.checkCommandDemoAccrpt()) {
    /// mTimer = 30; mMode = Mode_UP_DEMO_TIMER_e; }` - and note there is NO else branch, unlike
    /// `down_demo_wait`.
    ///
    /// `eventInfo.checkCommandDemoAccrpt()` (`:114`) is `mCommand == dEvtCmd_INDEMO_e`
    /// (`f_op_actor.h:177`), a latch `dEvt_control_c::demoCheck` sets on both ordered actors when
    /// the event starts (`d_event.cpp:314-316`) and `demoEnd` clears (`:331-337`) - so "the event
    /// this actor ordered is in `Stage::Play` or `Stage::Close`".
    fn up_demo_wait(&mut self, ctx: &Context<'_>) {
        if ctx.demo_running(self.event_slot) {
            self.timer = DEMO_HOLD;
            self.mode = MODE_UP_DEMO_TIMER;
        }
    }

    /// `daObjHbrf1_down_demo_wait` (`:170-178`): the same test, but with an
    /// `fopAcM_orderOtherEventId(this, mEventIdx)` in the `else` (`:177`).
    fn down_demo_wait(&mut self, ctx: &mut Context<'_>) {
        if ctx.demo_running(self.event_slot) {
            self.timer = DEMO_HOLD;
            self.mode = MODE_DOWN_DEMO_TIMER;
            return;
        }
        self.order(ctx);
    }

    /// `daObjHbrf1_up_demo_timer` (`:121-127`): `mTimer -= 1; if (mTimer == 0) {
    /// mMode = Mode_UP_DEMO_e; }`.
    fn up_demo_timer(&mut self) {
        self.timer -= 1;
        if self.timer == 0 {
            self.mode = MODE_UP_DEMO;
        }
    }

    /// `daObjHbrf1_down_demo_timer` (`:181-187`) - the same with `Mode_DOWN_DEMO_e`.
    fn down_demo_timer(&mut self) {
        self.timer -= 1;
        if self.timer == 0 {
            self.mode = MODE_DOWN_DEMO;
        }
    }

    /// `daObjHbrf1_up_demo` (`:130-148`).
    ///
    /// ```c
    /// current.pos.y += mYOffset;
    /// if (current.pos.y >= home.pos.y + 750.0f) {
    ///     current.pos.y = home.pos.y + 750.0f;
    ///     if (m2E4 == 1) { dComIfGp_event_reset(); m2E4 = 0; }
    ///     mMode = Mode_UP_STOP_e;
    ///     fopAcM_seStart(this, JA_SE_OBJ_B_LIFT_STOP, 0);
    ///     dComIfGp_getVibration().StartShock(4, -0x21, cXyz(0.0f, 1.0f, 0.0f));
    ///     return;
    /// }
    /// fopAcM_seStart(this, JA_SE_OBJ_B_LIFT_MOVE, 0);
    /// ```
    ///
    /// `dComIfGp_event_reset()` is `dEvDtBase_c::reset()` (`d_event.h:131`) = `mEventFlag |= 8` on
    /// the running event, which `dEvt_control_c::check` (`d_event.cpp:546-554`) turns into
    /// `endProc`; the sim's `ActiveEvent::end` is that teardown.
    fn up_demo(&mut self, actor: &mut Actor, ctx: &mut Context<'_>) {
        actor.pos.y += self.y_offset;
        if actor.pos.y >= self.home.y + TRAVEL {
            actor.pos.y = self.home.y + TRAVEL;
            self.reset_event(ctx);
            self.mode = MODE_UP_STOP;
            ctx.stop();
            return;
        }
        ctx.move_se();
    }

    /// `daObjHbrf1_down_demo` (`:190-208`) - the same against `current.pos.y <= home.pos.y` and
    /// `Mode_DOWN_STOP_e`.
    fn down_demo(&mut self, actor: &mut Actor, ctx: &mut Context<'_>) {
        actor.pos.y -= self.y_offset;
        if actor.pos.y <= self.home.y {
            actor.pos.y = self.home.y;
            self.reset_event(ctx);
            self.mode = MODE_DOWN_STOP;
            ctx.stop();
            return;
        }
        ctx.move_se();
    }

    /// `if (m2E4 == 1) { dComIfGp_event_reset(); m2E4 = 0; }` (`d_a_obj_hbrf1.cpp:136-139` and
    /// `:196-199`).
    fn reset_event(&mut self, ctx: &mut Context<'_>) {
        if self.ordered != 1 {
            return;
        }
        if let Some(slot) = self.event_slot {
            let _ = ctx.active.end(slot);
        }
        self.ordered = 0;
    }

    /// `fopAcM_orderOtherEventId(this, mEventIdx)` (`:98`, `:103`, `:155`, `:160`, `:177`),
    /// guarded exactly as `vfan::Vfan::order` / `tagetc::TagEtc::order` do - re-ordering an event
    /// that is already ORDER / PLAY / CLOSE rewrites its state (`d_event_manager.cpp:367`).
    fn order(&self, ctx: &mut Context<'_>) {
        let Some(slot) = self.event_slot else {
            return;
        };
        if ctx.active.stage_of(slot).is_none() {
            let _ = ctx.active.order(ctx.list, slot, ctx.entity);
        }
    }

    /// `daObjHbrf1::Act_c::Execute` (`:211-243`). The switch (`:212-237`) has exactly these eight
    /// labels and `mMode` is the `Mode_e` enum, so every index is in range.
    fn execute(&mut self, actor: &mut Actor, ctx: &mut Context<'_>) {
        match self.mode {
            MODE_DOWN_STOP => self.down_stop(ctx),
            MODE_UP_DEMO_WAIT => self.up_demo_wait(ctx),
            MODE_UP_DEMO_TIMER => self.up_demo_timer(),
            MODE_UP_DEMO => self.up_demo(actor, ctx),
            MODE_UP_STOP => self.up_stop(ctx),
            MODE_DOWN_DEMO_WAIT => self.down_demo_wait(ctx),
            MODE_DOWN_DEMO_TIMER => self.down_demo_timer(),
            MODE_DOWN_DEMO => self.down_demo(actor, ctx),
            _ => {}
        }
    }
}

/// Everything `Execute` needs beyond the component, in one bundle so the state machine has a
/// single reader - the `tagetc` / `dekuitem` shape. `stop()` and `move_se()` are the two
/// `fopAcM_seStart` + `StartShock` outcomes of the two `*_demo` arms.
struct Context<'a> {
    switches: &'a Switches,
    flags: &'a mut EventFlags,
    active: &'a mut ActiveEvent,
    list: &'a EventList,
    events: &'a mut SimEvents,
    vibration: &'a mut Vibration,
    entity: Entity,
}

impl Context<'_> {
    fn demo_running(&self, slot: Option<usize>) -> bool {
        slot.is_some_and(|slot| {
            matches!(
                self.active.stage_of(slot),
                Some(Stage::Play) | Some(Stage::Close)
            )
        })
    }

    /// `fopAcM_seStart(this, JA_SE_OBJ_B_LIFT_STOP, 0)` (`d_a_obj_hbrf1.cpp:142`, `:202`) plus
    /// `StartShock(4, -0x21, cXyz(0.0f, 1.0f, 0.0f))` (`:143`, `:203`).
    fn stop(&mut self) {
        self.events.list.push(SimEvent::Se(SE_LIFT_STOP));
        self.vibration
            .start_shock(SHOCK_PATTERN, SHOCK_FLAGS, Vec3::from_array(SHOCK_COORD));
    }

    /// `fopAcM_seStart(this, JA_SE_OBJ_B_LIFT_MOVE, 0)` (`d_a_obj_hbrf1.cpp:147`, `:207`).
    fn move_se(&mut self) {
        self.events.list.push(SimEvent::Se(SE_LIFT_MOVE));
    }
}

/// `daObj::PrmAbstract`'s `((1 << width) - 1)` mask (`d_a_obj.h:15`), with the widths coming from
/// `enum Prm_e` (`d_a_obj_hbrf1.h:24-29`). Guarded on the shift so `1 << 32` cannot happen even
/// though every `Prm_e` width is 1 or 8.
fn mask(width: u32) -> u32 {
    if width >= 32 {
        u32::MAX
    } else {
        (1 << width) - 1
    }
}

pub fn is_hbrf1(name: &str) -> bool {
    HBRF1_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut flags: ResMut<EventFlags>,
    switches: Res<Switches>,
    mut events: ResMut<SimEvents>,
    mut vibration: ResMut<Vibration>,
    mut lifts: Query<(Entity, &mut Actor, &mut Hbrf1)>,
) {
    for (entity, mut actor, mut lift) in &mut lifts {
        let actor = &mut *actor;
        let lift = &mut *lift;
        if !lift.entered {
            lift.create_init(actor, &list, &switches);
        }
        let mut ctx = Context {
            switches: &switches,
            flags: &mut flags,
            active: &mut active,
            list: &list,
            events: &mut events,
            vibration: &mut vibration,
            entity,
        };
        lift.execute(actor, &mut ctx);
    }
}
