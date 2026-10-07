use bevy_ecs::prelude::*;

use super::event::{ActiveEvent, EventList, Stage};
use super::events::{SimEvent, SimEvents};
use super::flags::{EventFlags, Switches};

/// `OBJNAME("Gbrg00", fpcNm_LIGHTBRIDGE_e, 255, 0)` (`d_stage.cpp:909`) is `fpcNm_LIGHTBRIDGE_e` =
/// `d_a_lbridge.cpp`. `daLbridge_c::m_arcname[]` is also `"Gbrg00"` (`d_a_lbridge.cpp:11`), so stage
/// string and archive name agree. No other `*_ACTORS` array in `src/sim/` claims `Gbrg00`.
///
/// NOTE this is a DIFFERENT translation unit from `src/sim/bridge.rs`-style gates: `d_a_bridge.cpp`
/// and `d_a_lbridge.cpp` are separate files with separate profiles (`fpcNm_LIGHTBRIDGE_e` here).
pub const LBRIDGE_ACTORS: [&str; 1] = ["Gbrg00"];

/// `fopAcM_setCullSizeBox(this, -600.0f, -100.0f, -150.0f, 600.0f, 100.0f, 150.0f)`
/// (`d_a_lbridge.cpp:72`) - a real custom box, so the profile's `fopAc_CULLBOX_CUSTOM_e` (`:362`)
/// resolves.
const CULL_MIN: [f32; 3] = [-600.0, -100.0, -150.0]; // <- d_a_lbridge.cpp:72
const CULL_MAX: [f32; 3] = [600.0, 100.0, 150.0]; // <- d_a_lbridge.cpp:72
/// `fopAcM_setCullSizeFar(this, 1.5f)` (`d_a_lbridge.cpp:73`) sets `i_actor->cullSizeFar`
/// (`f_op_actor_mng.h:316-318`); it overrides only the far plane and does NOT create a box, so it is
/// cited here and not stored - the sim's `cull()` channel carries the box alone.

/// `daLbridge_prm::getSwitchNo(ac) = (fopAcM_GetParam(ac) >> 0) & 0xFF` (`d_a_lbridge.h:46-48`) over
/// the WHOLE u32 (`f_op_actor_mng.h:127-129`).
const SWITCH_SHIFT: u32 = 0; // <- d_a_lbridge.h:47
const SWITCH_MASK: u32 = 0xFF; // <- d_a_lbridge.h:47
/// `if (mSwitchNo == 0xFF) { return; }` (`d_a_lbridge.cpp:154`) and the identical test at `:89` -
/// the "always visible, never switch-driven" sentinel.
const NO_SWITCH: u8 = 0xFF; // <- d_a_lbridge.cpp:89

/// `dComIfGp_evmng_getEventIdx("EFFAPPEAR")` (`d_a_lbridge.cpp:86`) and
/// `dComIfGp_evmng_getEventIdx("BRIDGE_DISAPPEAR")` (`:87`) - the one-argument form defaults
/// `evNo` to `-1` (`0xFF`, `d_com_inf_game.h:3611`), which skips the stage event-info redirect
/// (`d_event_manager.cpp:347-348`), so both are pure name lookups.
const EVENT_APPEAR: &str = "EFFAPPEAR"; // <- d_a_lbridge.cpp:86
const EVENT_DISAPPEAR: &str = "BRIDGE_DISAPPEAR"; // <- d_a_lbridge.cpp:87

/// `dComIfGs_onEventBit(dSv_event_flag_c::UNK_0E01)` (`d_a_lbridge.cpp:204`,
/// `d_save_event_flag.inc:110`) - set when the appear event ends, and the `else` arm's guard
/// (`:212`) that stops the actor re-ordering it.
const EVT_0E01: u32 = 0x0E01; // <- d_save_event_flag.inc:110
/// `dComIfGs_onEventBit(dSv_event_flag_c::UNK_0F40)` (`d_a_lbridge.cpp:209`,
/// `d_save_event_flag.inc:112`) - the same pair for the disappear event (`:207`, `:215`).
const EVT_0F40: u32 = 0x0F40; // <- d_save_event_flag.inc:112

/// `fopAcM_seStart(this, JA_SE_OBJ_L_BRIDGE_ON, 0)` (`d_a_lbridge.cpp:269`,
/// `JAZelAudio_SE.h:1870`) - `appear_bridge`'s sound.
const SE_L_BRIDGE_ON: u32 = 0x6956; // <- JAZelAudio_SE.h:1870
/// `fopAcM_seStart(this, JA_SE_OBJ_L_BRIDGE_OFF, 0)` (`d_a_lbridge.cpp:274`,
/// `JAZelAudio_SE.h:1871`) - `disappear_bridge`'s sound.
const SE_L_BRIDGE_OFF: u32 = 0x6957; // <- JAZelAudio_SE.h:1871

/// `mTimer = 31;` (`d_a_lbridge.cpp:247`) - armed by `appear_bridge`, drained by
/// `cLib_calcTimer(&mTimer)` at `:181`, and the gate on the fade-in at `:183`.
const FADE_IN_DELAY: u8 = 31; // <- d_a_lbridge.cpp:247
/// `cLib_chaseUC(&alpha, 0x0, 0x8)` (`d_a_lbridge.cpp:175`) and `cLib_chaseUC(&alpha, 0xFF, 0x8)`
/// (`:185`) - the emitter's global alpha step in both directions.
const ALPHA_STEP: u8 = 0x8; // <- d_a_lbridge.cpp:175
/// The `0x0` of `cLib_chaseUC(&alpha, 0x0, 0x8)` (`d_a_lbridge.cpp:175`).
const ALPHA_HIDDEN: u8 = 0x00; // <- d_a_lbridge.cpp:175
/// The `0xFF` of `cLib_chaseUC(&alpha, 0xFF, 0x8)` (`d_a_lbridge.cpp:185`) and of
/// `mpEmitter->setGlobalAlpha(0xFF)` (`:97`).
const ALPHA_SHOWN: u8 = 0xFF; // <- d_a_lbridge.cpp:185
/// `mpEmitter->setGlobalAlpha(0)` (`d_a_lbridge.cpp:79`) in `CreateInit`, before the
/// `isSwitch || mSwitchNo == 0xFF` arm decides whether the emitter is ever shown.
const ALPHA_NONE: u8 = 0x00; // <- d_a_lbridge.cpp:79

/// `unk31C` (`d_a_lbridge.h:40`, an `s16`) has no enum; these are its only three written values.
const PENDING_NONE: i16 = 0; // <- d_a_lbridge.cpp:194, :199 (the `= 0` resets)
const PENDING_APPEAR: i16 = 1; // <- d_a_lbridge.cpp:248 (appear_bridge's `unk31C = 1`)
const PENDING_DISAPPEAR: i16 = 2; // <- d_a_lbridge.cpp:264 (disappear_bridge's `unk31C = 2`)

/// Tower of the Gods' glowing bridge - `daLbridge_c` (`d_a_lbridge.h:8-43`), the actor behind the
/// stage string `Gbrg00`.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. Checked, not assumed, because a bridge is
/// exactly the sort of thing one expects to be damageable: `daLbridge_c` is a bare `fopAc_ac_c`
/// holding a model, a `dBgW*`, an `Mtx`, three animators, a `JPABaseEmitter*` and five small
/// fields (`d_a_lbridge.h:29-43`) and it registers NO `dCcD` collider of any kind - no `dCcD_Stts`,
/// no `dCcD_Cyl`, no `dCcD_Src*`. The `dBgW` it builds from `GBRG00`'s `DZB_HHASHI1`
/// (`d_a_lbridge.cpp:58`) is a moving background (RULE 3b), not a damage path, and the profile
/// `Group` is `fopAc_ACTOR_e` (`:361`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
///
/// `daLbridge_IsDelete` (`:335-338`) is the bare "accepts a delete request" form of RULE 6b, and
/// `fopAcM_delete` appears nowhere in this translation unit. `_create` (`:103-116`) can return
/// `cPhs_ERROR_e` only from `fopAcM_entrySolidHeap` (`:109-111`), which is heap work, so `spawn`
/// never refuses.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Lbridge {
    /// `mSwitchNo` (`d_a_lbridge.h:37`, an `s32`) from `daLbridge_prm::getSwitchNo` (`:82`).
    pub switch_no: u8,
    /// `mIsSw` (`d_a_lbridge.h:43`, a `u8`) - the PREVIOUS tick's switch read, which `_execute`
    /// refreshes at `:145` from the value `sw_check` sampled at `:136`.
    pub was_switch: bool,
    /// `unk31C` (`d_a_lbridge.h:40`, an `s16`) - "this actor is trying to appear" (1) or "trying to
    /// disappear" (2), cleared when the matching event starts (`:194-200`).
    pub pending: i16,
    /// `mTimer` (`d_a_lbridge.h:42`, a `u8`) - `FADE_IN_DELAY` frames of hold before the emitter
    /// may fade in (`:247`, `:181-187`).
    pub timer: u8,
    /// `mpEmitter->getGlobalAlpha()` (`d_a_lbridge.cpp:174`, `:184`) - the bridge's own alpha.
    /// `setGlobalAlpha` at `:79`, `:97`, `:176`, `:186` and `appear_bridge`'s
    /// `playCreateParticle` / `playDrawParticle` (`:242-245`) all read or write it, and it is what
    /// gates `fopAcM_onDraw` / `fopAcM_offDraw` (`:179`, `:170`).
    pub alpha: u8,
    /// Whether `dComIfG_Bgsp()->Regist(mpBgW, this)` has run - i.e. whether the collision volume
    /// exists. `Regist` at `:90` (create) and `:224` (`appear_bridge`); `Release` at `:253`
    /// (`disappear_bridge`) and `:307` (delete). A `dBgW` itself is RULE 3b absent, but the
    /// registered/unregistered FLAG is not, and `appear_bridge`'s `mpBgW->Move()` at `:226` is
    /// what puts the volume in place.
    pub solid: bool,
    /// `mAppearEventIdx` (`d_a_lbridge.h:38`, an `s16`) from `getEventIdx("EFFAPPEAR")` (`:86`);
    /// `None` is `getEventIdx`'s -1.
    pub appear_slot: Option<usize>,
    /// `mDisappearEventIdx` (`d_a_lbridge.h:39`, an `s16`) from `:87`.
    pub disappear_slot: Option<usize>,
    /// One-shot guard so `CreateInit`'s two `getEventIdx` lookups run once, on the first tick.
    pub entered: bool,
}

impl Lbridge {
    /// `daLbridge_c::CreateInit` (`:69-100`) plus `_create`, minus `fopAcM_SetMtx` (`:70`), the cull
    /// box (`:72-73`, carried by [`Lbridge::cull`]), the `dComIfGp_particle_set` emitter and its
    /// `stopDrawParticle` / `setGlobalAlpha(0)` (`:75-80`, particle work), `set_mtx` (`:91`) and the
    /// `Regist` / `mpBgW->Move()` pair (`:90`, `:93`).
    ///
    /// The `isSwitch || mSwitchNo == 0xFF` gate (`:89`) decides both the emitter's starting alpha
    /// and whether the volume is registered, so it is re-decided on the first tick - the `hbrf1` /
    /// `tagetc` shape, which cannot see `Switches` from `spawn`.
    ///
    /// `fopAc_Init` seeds `current` and `old` from `home` (`f_op_actor.cpp:282-284`); RULE 2 gives
    /// the bridge scale exactly 1.0, and `setBaseScale(scale)` (`:120`) is renderer work.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            switch_no: ((parameters >> SWITCH_SHIFT) & SWITCH_MASK) as u8,
            // `mIsSw` has no initialiser, so `fopAcM_ct`'s `sBs_ClearArea` zero-fill
            // (`f_pc_base.cpp:98`) makes it FALSE until `create_init` records the real read.
            was_switch: false,
            // `unk31C` likewise takes the zero-fill, which is `PENDING_NONE`.
            pending: PENDING_NONE,
            // `mTimer` likewise; `cLib_calcTimer` (`:181`) keeps a zero at zero until
            // `appear_bridge` arms it.
            timer: 0,
            // `mpEmitter->setGlobalAlpha(0)` (`:79`) is the create-time value; it is overwritten
            // by `create_init` when the bridge starts switched on.
            alpha: ALPHA_NONE,
            // `dComIfG_Bgsp()->Regist(mpBgW, this)` runs only inside the gate at `:89`.
            solid: false,
            appear_slot: None,
            disappear_slot: None,
            entered: false,
        }
    }

    /// `fopAcM_setCullSizeBox(this, -600.0f, -100.0f, -150.0f, 600.0f, 100.0f, 150.0f)`
    /// (`d_a_lbridge.cpp:72`). `fopAcM_setCullSizeFar(this, 1.5f)` (`:73`) overrides only
    /// `cullSizeFar`, so the box above is untouched.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `CreateInit` (`:69-100`) minus the renderer, particle and `dBgW` calls.
    fn create_init(&mut self, switches: &Switches, list: &EventList) {
        self.appear_slot = list.event_index(EVENT_APPEAR);
        self.disappear_slot = list.event_index(EVENT_DISAPPEAR);
        // `mIsSw = (bool)fopAcM_isSwitch(this, mSwitchNo);` (:84)
        self.was_switch = switches.test(self.switch_no);
        // `if (fopAcM_isSwitch(this, mSwitchNo) || mSwitchNo == 0xFF) { ... }` (:89)
        if self.was_switch || self.switch_no == NO_SWITCH {
            // `dComIfG_Bgsp()->Regist(mpBgW, this);` (:90)
            self.solid = true;
            // `if (mpEmitter != NULL) { playDrawParticle(); setGlobalAlpha(0xFF); }` (:95-98)
            self.alpha = ALPHA_SHOWN;
        }
        self.entered = true;
    }

    /// `daLbridge_c::_execute` (`:135-148`) minus the three `m*Anm.play()` calls (`:140-142`) and
    /// `set_mtx` (`:143`) - texture/colour/tev animation and model-matrix work. The BPK frame
    /// bookkeeping inside `appear_bridge` / `disappear_bridge` (`mBpkAnm.setFrame`,
    /// `setPlaySpeed`, `getStartFrame`, `getEndFrame`) is likewise archive data (RULE 3b) and is
    /// not transcribed.
    fn execute(
        &mut self,
        list: &EventList,
        active: &mut ActiveEvent,
        switches: &Switches,
        ev_flags: &mut EventFlags,
        events: &mut SimEvents,
        entity: Entity,
    ) {
        // `bool isSw = fopAcM_isSwitch(this, mSwitchNo);` (:136)
        let is_switch = switches.test(self.switch_no);
        self.sw_check(switches, events);
        self.demo(list, active, ev_flags, events, entity);
        // `mIsSw = isSw;` (:145) - note it is the value sampled at `:136`, NOT a fresh read.
        self.was_switch = is_switch;
    }

    /// `daLbridge_c::sw_check` (`:151-189`).
    fn sw_check(&mut self, switches: &Switches, events: &mut SimEvents) {
        let is_switch = switches.test(self.switch_no);
        // `if (mSwitchNo == 0xFF) { return; }` (:154-156) - an unswitched bridge never appears or
        // disappears, and never fades.
        if self.switch_no == NO_SWITCH {
            return;
        }
        // `if (isSw != mIsSw) { if (isSw) { appear_bridge(); } else { disappear_bridge(); } }`
        // (:158-164) - against the value `_execute` stored at the end of the previous tick.
        if is_switch != self.was_switch {
            if is_switch {
                self.appear_bridge(events);
            } else {
                self.disappear_bridge(events);
            }
        }

        if !is_switch {
            // `if (mBpkAnm.getFrame() == mBpkAnm.getStartFrame()) { fopAcM_offDraw(this); }`
            // (:169-171) - BPK frame data is archive data (RULE 3b), see OMITTED. The fade-out
            // below does not depend on it.
            // ```
            // if (mpEmitter != NULL) {
            //     alpha = mpEmitter->getGlobalAlpha();
            //     cLib_chaseUC(&alpha, 0x0, 0x8);
            //     mpEmitter->setGlobalAlpha(alpha);
            // }
            // ```
            // (:173-177). `cLib_chaseUC` (`c_lib.cpp:235-256`) steps TOWARDS the target and snaps.
            chase_alpha(&mut self.alpha, ALPHA_HIDDEN, ALPHA_STEP);
        } else {
            // `fopAcM_onDraw(this);` (:179)
            // `cLib_calcTimer(&mTimer);` (:181) - `c_lib.h:85-91`.
            if self.timer != 0 {
                self.timer -= 1;
            }
            // `if (mTimer == 0 && mpEmitter != NULL) { ... cLib_chaseUC(&alpha, 0xFF, 0x8); ... }`
            // (:183-187)
            if self.timer == 0 {
                chase_alpha(&mut self.alpha, ALPHA_SHOWN, ALPHA_STEP);
            }
        }
    }

    /// `daLbridge_c::appear_bridge` (`:223-249`).
    fn appear_bridge(&mut self, events: &mut SimEvents) {
        // `dComIfG_Bgsp()->Regist(mpBgW, this); ... mpBgW->Move();` (:224-226)
        self.solid = true;
        // `dComIfGp_particle_setProjection(dPa_name::ID_IT_SN_RBRIDGE_APP00, &pos1, &current.angle)`
        // twice, at `current.pos.z +- 100.0f` (`:228-235`) - particle work.
        // `set_on_se();` (:237)
        events.list.push(SimEvent::Se(SE_L_BRIDGE_ON));
        // `if (mpEmitter != NULL) { mpEmitter->playCreateParticle(); mpEmitter->playDrawParticle(); }`
        // (:242-245) - the emitter is already alpha [`ALPHA_SHOWN`] here in retail because
        // `appear_bridge` is only ever reached from a bridge that faded out to
        // [`ALPHA_HIDDEN`]; the value itself is left alone, exactly as retail leaves it.
        // `mTimer = 31;` (:247)
        self.timer = FADE_IN_DELAY;
        // `unk31C = 1;` (:248)
        self.pending = PENDING_APPEAR;
    }

    /// `daLbridge_c::disappear_bridge` (`:252-265`).
    fn disappear_bridge(&mut self, events: &mut SimEvents) {
        // `dComIfG_Bgsp()->Release(mpBgW);` (:253)
        self.solid = false;
        // `if (mpEmitter != NULL) { mpEmitter->stopDrawParticle(); }` (:255-257) - particle work;
        // the emitter alpha is NOT set here, which is why `sw_check` fades it out over the
        // following ticks.
        // `set_off_se();` (:259)
        events.list.push(SimEvent::Se(SE_L_BRIDGE_OFF));
        // `mBpkAnm.setFrame(mBpkAnm.getEndFrame()); mBpkAnm.setPlaySpeed(-1.0f);` (:261-262) -
        // BPK frame data (RULE 3b).
        // `unk31C = 2;` (:264)
        self.pending = PENDING_DISAPPEAR;
    }

    /// `daLbridge_c::demo` (`:192-220`).
    ///
    /// The two `eventInfo.onCondition(dEvtCnd_UNK2_e)` calls (`:214`, `:217`) are dropped for the
    /// reason `htobi3.rs` gives: nothing in the decomp ever calls `chkCondition` with
    /// `dEvtCnd_UNK2_e`, and `beforeProc` clears `mCondition` on every `fopAc_Execute`
    /// (`f_op_actor.cpp:176`).
    fn demo(
        &mut self,
        list: &EventList,
        active: &mut ActiveEvent,
        ev_flags: &mut EventFlags,
        events: &mut SimEvents,
        entity: Entity,
    ) {
        // `if (eventInfo.checkCommandDemoAccrpt()) { ... } else { ... }` (:193-219). That latch is
        // set on this actor precisely because the `else` arm ordered one of its two events, so
        // "one of them is in Play or Close" is the exact reading.
        if !Self::demo_running(active, self) {
            // ```
            // if (!isEventBit(UNK_0E01) && unk31C == 1) {
            //     fopAcM_orderOtherEventId(this, mAppearEventIdx); onCondition(UNK2);
            // } else if (!isEventBit(UNK_0F40) && unk31C == 2) {
            //     fopAcM_orderOtherEventId(this, mDisappearEventIdx); onCondition(UNK2);
            // }
            // ```
            // (:211-218)
            if !ev_flags.test(EVT_0E01) && self.pending == PENDING_APPEAR {
                Self::order(self.appear_slot, list, active, entity);
            } else if !ev_flags.test(EVT_0F40) && self.pending == PENDING_DISAPPEAR {
                Self::order(self.disappear_slot, list, active, entity);
            }
            return;
        }
        // `if (startCheck(mAppearEventIdx) && unk31C == 1) { unk31C = 0; }` (:194-196)
        if self.pending == PENDING_APPEAR
            && self
                .appear_slot
                .is_some_and(|slot| active.start_check(slot))
        {
            self.pending = PENDING_NONE;
        }
        // `if (startCheck(mDisappearEventIdx) && unk31C == 2) { unk31C = 0; }` (:198-200)
        if self.pending == PENDING_DISAPPEAR
            && self
                .disappear_slot
                .is_some_and(|slot| active.start_check(slot))
        {
            self.pending = PENDING_NONE;
        }
        // ```
        // if (endCheck(mAppearEventIdx)) { dComIfGp_event_reset(); onEventBit(UNK_0E01); }
        // if (endCheck(mDisappearEventIdx)) { dComIfGp_event_reset(); onEventBit(UNK_0F40); }
        // ```
        // (:202-210). These are two independent `if`s, not an `else if`, and each `event_reset` is
        // paired with its own flag - transcribed in the decomp's order. `dComIfGp_event_reset()` is
        // `dEvDtBase_c::reset()` (`d_event.h:131`) = `mEventFlag |= 8`, which `dEvt_control_c::check`
        // (`d_event.cpp:546-554`) turns into `endProc` -> `demoEnd`; `ActiveEvent::end` is that
        // teardown.
        if let Some(slot) = self.appear_slot.filter(|slot| active.end_check(*slot)) {
            let _ = active.end(slot);
            ev_flags.set(EVT_0E01);
        }
        if let Some(slot) = self.disappear_slot.filter(|slot| active.end_check(*slot)) {
            let _ = active.end(slot);
            ev_flags.set(EVT_0F40);
        }
    }

    /// `fopAcM_orderOtherEventId(this, mAppearEventIdx)` (`:213`) and its `mDisappearEventIdx` twin
    /// (`:216`), guarded exactly as `hbrf1::Hbrf1::order` guards it - re-ordering an event already
    /// in ORDER / PLAY / CLOSE rewrites its state (`d_event_manager.cpp:367`).
    fn order(slot: Option<usize>, list: &EventList, active: &mut ActiveEvent, entity: Entity) {
        let Some(slot) = slot else {
            return;
        };
        if active.stage_of(slot).is_none() {
            let _ = active.order(list, slot, entity);
        }
    }

    /// `eventInfo.checkCommandDemoAccrpt()` as `Stage::Play | Stage::Close` - the two states
    /// `dEvent_manager_c::getMyStaffId` scans (`d_event_manager.cpp:412`).
    fn demo_running(active: &ActiveEvent, bridge: &Lbridge) -> bool {
        [bridge.appear_slot, bridge.disappear_slot]
            .into_iter()
            .flatten()
            .any(|slot| matches!(active.stage_of(slot), Some(Stage::Play) | Some(Stage::Close)))
    }
}

/// `cLib_chaseUC(u8* pValue, u8 target, u8 step)` (`c_lib.cpp:235-256`): step toward the target,
/// snapping to it and returning 1 on the tick that lands (`c_lib.cpp:246-248`). `c_lib.h:26`
/// declares the return as `int`; both call sites (`d_a_lbridge.cpp:175`, `:185`) discard it, which
/// [`CHASE_DONE`] names.
fn chase_alpha(alpha: &mut u8, target: u8, step: u8) -> bool {
    if step == 0 {
        return *alpha == target;
    }
    let wide = i16::from(*alpha);
    let wide_target = i16::from(target);
    let wide_step = if wide > wide_target { -i16::from(step) } else { i16::from(step) };
    let next = wide + wide_step;
    if wide_step * (next - wide_target) >= 0 {
        *alpha = target;
        true
    } else {
        *alpha = next as u8;
        false
    }
}

pub fn is_lbridge(name: &str) -> bool {
    LBRIDGE_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    switches: Res<Switches>,
    mut ev_flags: ResMut<EventFlags>,
    mut events: ResMut<SimEvents>,
    mut bridges: Query<(Entity, &mut Lbridge)>,
) {
    for (entity, mut bridge) in &mut bridges {
        if !bridge.entered {
            bridge.create_init(&switches, &list);
        }
        bridge.execute(
            &list,
            &mut active,
            &switches,
            &mut ev_flags,
            &mut events,
            entity,
        );
    }
}