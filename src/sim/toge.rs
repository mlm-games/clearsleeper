use bevy_ecs::prelude::*;

use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::walk;

/// `OBJNAME("Htoge1", fpcNm_TOGE_e, 255, 50)` (`d_stage.cpp:1109`) is `fpcNm_TOGE_e` =
/// `d_a_toge.cpp`. `daToge_c::m_arcname[]` is also `"Htoge1"` (`d_a_toge.cpp:13`), so stage string
/// and archive name agree. No other `*_ACTORS` array in `src/sim/` claims `Htoge1`.
pub const TOGE_ACTORS: [&str; 1] = ["Htoge1"];

/// `const f32 daToge_c::m_y_min = -150.0f;` (`d_a_toge.cpp:17`) - the fully-EXTENDED spike
/// position, and `Create`'s start height when the switch is already on (`:124`).
const Y_MIN: f32 = -150.0; // <- d_a_toge.cpp:17 (m_y_min)
/// `fopAcM_setCullSizeBox(this, -80.0f, 0.0f, -80.0f, 80.0f, 120.0f, 80.0f)` (`d_a_toge.cpp:115`) -
/// a real custom box, so the profile's `fopAc_CULLBOX_CUSTOM_e` (`:345`) resolves.
const CULL_MIN: [f32; 3] = [-80.0, 0.0, -80.0]; // <- d_a_toge.cpp:115
const CULL_MAX: [f32; 3] = [80.0, 120.0, 80.0]; // <- d_a_toge.cpp:115

/// `toge_move`'s `f32 f31 = 30.0f;` (`d_a_toge.cpp:238`) - the `maxStep` of both `cLib_addCalc`
/// calls (`:253`, `:261`).
const MAX_VEL: f32 = 30.0; // <- d_a_toge.cpp:238
/// `toge_move`'s `f32 f30 = 15.0f;` (`d_a_toge.cpp:239`) - the `minStep` of the same two calls.
const MIN_VEL: f32 = 15.0; // <- d_a_toge.cpp:239
/// `int timer = 0xA;` (`d_a_toge.cpp:240`) - armed into `unk486` by `case 4` (`:265`) and drained
/// by `case 1` (`:246`).
const REARM_TIMER: u8 = 0x0A; // <- d_a_toge.cpp:240
/// `cLib_addCalc(&unk470, -150.0f, 0.1f, f31, f30)` (`d_a_toge.cpp:253`) - the `case 2` target. The
/// decomp writes the literal `-150.0f`, not `m_y_min`, and the values agree; the comment at `:252`
/// notes the coincidence.
const EXTEND_TARGET: f32 = -150.0; // <- d_a_toge.cpp:253
/// The `0.1f` scale of both `cLib_addCalc` calls (`d_a_toge.cpp:253`, `:261`).
const ADD_SCALE: f32 = 0.1; // <- d_a_toge.cpp:253
/// `unk474 = -60.0f;` (`d_a_toge.cpp:272`) - the return target of the idle bob; `case 4` chases it
/// until `unk470` reaches it, then flips back to 0 (`:263`).
const BOB_TARGET: f32 = -60.0; // <- d_a_toge.cpp:272
/// `set_collision` (`:214-223`) is the only consumer of `l_cyl_src`'s geometry - `Radius 55.0f`
/// (`:44`), `Height 150.0f` (`:45`), `center.y += (unk470 - 10.0f)` (`:218`) - and it is OMITTED
/// in full (see the struct doc), so none of those three numbers is stored. The AT it registers is
/// `SrcObjAt Type = AT_TYPE_UNK800` at `Atp = 0x1` (`:23-24`) with
/// `cCcD_AtSPrm_Set_e | cCcD_AtSPrm_VsPlayer_e` (`:25`), also omitted.

/// `daToge_prm::getSwitchNo(ac) = (fopAcM_GetParam(ac) >> 0) & 0xFF` (`d_a_toge.h:49-51`) over the
/// WHOLE u32 (`f_op_actor_mng.h:127-129`).
const SWITCH_SHIFT: u32 = 0; // <- d_a_toge.h:50
const SWITCH_WIDTH: u32 = 0xFF; // <- d_a_toge.h:50
/// `daToge_prm::getSeEnabled(ac) = (fopAcM_GetParam(ac) >> 8) & 0xF` (`d_a_toge.h:53-55`) - the gate
/// on `toge_seStart`'s `fopAcM_seStart(this, i_seNum, 0)` (`:281-283`).
const SE_SHIFT: u32 = 8; // <- d_a_toge.h:54
const SE_WIDTH: u32 = 0xF; // <- d_a_toge.h:54
/// `if (mSwitchNo != 0xFF)` / `else if (mSwitchNo == 0xFF)` (`d_a_toge.cpp:178`, `:186`) - the
/// "0xFF means wind-driven, not switch-driven" sentinel.
const NO_SWITCH: u8 = 0xFF; // <- d_a_toge.cpp:178

/// `mEventState` (`d_a_toge.h:44`, a `u8`) has no enum in the header; these are the five values
/// `toge_move`'s `switch (mEventState)` (`:242`) and `_execute`'s comparisons name.
const STATE_IDLE: u8 = 0; // <- d_a_toge.cpp:243 (case 0, `break`)
const STATE_PENDING: u8 = 1; // <- d_a_toge.cpp:245 (case 1; `!= 1` at :180, :196)
const STATE_EXTENDED: u8 = 2; // <- d_a_toge.cpp:251 (case 2; `= 2` at :125, :249)
const STATE_RETRACT: u8 = 3; // <- d_a_toge.cpp:255 (case 3; `= 3` at :184, :194)
const STATE_CYCLE: u8 = 4; // <- d_a_toge.cpp:260 (case 4; `!= 4` at :183, :193)

/// `toge_seStart(JA_SE_OBJ_TOGETOGE_IN)` (`d_a_toge.cpp:248`, `JAZelAudio_SE.h:2212`) - the spikes
/// driving out.
const SE_TOGETOGE_IN: u32 = 0x7842; // <- JAZelAudio_SE.h:2212
/// `toge_seStart(JA_SE_OBJ_TOGETOGE_OUT)` (`d_a_toge.cpp:256`, `JAZelAudio_SE.h:2210`) - the spikes
/// starting to retract.
const SE_TOGETOGE_OUT: u32 = 0x7840; // <- JAZelAudio_SE.h:2210
/// `toge_seStart(JA_SE_OBJ_TOGETOGE_MOVE)` (`d_a_toge.cpp:270`, `JAZelAudio_SE.h:2211`) - the idle
/// bob.
const SE_TOGETOGE_MOVE: u32 = 0x7841; // <- JAZelAudio_SE.h:2211

/// Wind Temple's floor spikes - `daToge_c` (`d_a_toge.h:8-46`), the actor behind the stage string
/// `Htoge1`.
///
/// `daToge_c` is NOT a `dBgS_MoveBgActor`, unlike `daObjOjtree` and friends: it is a bare
/// `fopAc_ac_c` holding two `dBgW` moving backgrounds (`d_a_toge.h:34`, `:36`) whose only
/// gameplay-visible product is the spikes' swept volume. The profile `Group` is `fopAc_ACTOR_e`
/// (`:344`).
///
/// (a) inbound damage: NONE. `l_cyl_src` (`d_a_toge.cpp:19-47`) has `SrcObjAt Type = AT_TYPE_UNK800`
/// at `Atp = 0x1` (`:23-24`) - that is the toge's OWN outbound attack - and `SrcObjTg Type =
/// AT_TYPE_ALL` with `SrcObjTg SPrm = 0` and `SrcGObjTg SPrm = dCcG_TgSPrm_NoHitMark_e`
/// (`:26`, `:27`, `:38`). A `SrcObjTg` only tells the toge that something touched it; there is no
/// `cc_at_check`, no health field and no damage reaction in `toge_move` or `_execute`, so the toge
/// has no inbound path. Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
///
/// (b) outbound damage: OMITTED. `set_collision` (`:214-223`) re-registers `mCyl` every tick from
/// `center.y += (unk470 - 10.0f)` (`:218`) with the cylinder's `Radius 55` / `Height 150`
/// (`:44-45`) and `cCcD_AtSPrm_Set_e | cCcD_AtSPrm_VsPlayer_e` (`:25`). The `cCcD` collider registry
/// and the AT-to-player dispatch are both RULE 3b absent, and `cM3dGCylS`'s `Height` convention
/// (measured from the centre upward, as `cM3dGCylS` does, or straddling it) is not resolvable from
/// anything in this checkout - so the contact test would have to be invented. `src/sim/fallrock.rs`
/// omits the identical shape (`AT_TYPE_UNK8` at `Atp = 1`, a `cM3dGCylS` re-registered each tick)
/// for the identical reason.
///
/// `daToge_IsDelete` (`:318-321`) is the bare "accepts a delete request" form of RULE 6b, and
/// `fopAcM_delete` appears nowhere in this translation unit. `_create` (`:151-165`) can return
/// `cPhs_ERROR_e` only from `fopAcM_entrySolidHeap` (`:157-159`), which is heap work, so `spawn`
/// never refuses.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Toge {
    /// `unk470` (`d_a_toge.h:38`, an `f32`) - the spike height. Zero-filled at create (no
    /// initialiser), set to `m_y_min` when the switch is already on (`:124`), and otherwise driven
    /// by `toge_move`'s `cLib_addCalc` (`:253`, `:261`).
    pub spike: f32,
    /// `unk474` (`d_a_toge.h:39`, an `f32`) - the second spring's target. Zero-filled at create,
    /// set to `0` when the spikes are below the floor (`:263`) and to [`BOB_TARGET`] when they are
    /// above it (`:272`).
    pub bob: f32,
    /// `mSwitchNo` (`d_a_toge.h:40`, an `s32`) from `daToge_prm::getSwitchNo` (`:121`).
    pub switch_no: u8,
    /// `mEventState` (`d_a_toge.h:44`) - the `toge_move` switch index.
    pub event_state: u8,
    /// `unk485` (`d_a_toge.h:45`, a `u8`) - "something wants the spikes out". Raised at `:181` and
    /// `:197`, consumed at `:267`, and cleared at `:265`.
    pub want_out: u8,
    /// `unk486` (`d_a_toge.h:46`, a `u8`) - the re-arm delay [`REARM_TIMER`] frames long, drained
    /// by `cLib_calcTimer(&unk486)` at `case 1` (`:246`).
    pub delay: u8,
    /// `daToge_prm::getSeEnabled` (`d_a_toge.h:53-55`) - the gate on all three `fopAcM_seStart`
    /// calls (`d_a_toge.cpp:281-283`).
    pub se_enabled: u8,
    /// One-shot guard so `Create`'s switch read (`:123-126`) happens once, on the first tick, where
    /// `Switches` is available.
    pub entered: bool,
}

impl Toge {
    /// `daToge_c::Create` (`:112-148`) minus `fopAcM_SetMtx` (`:113`), the cull box (`:115`, carried
    /// by [`Toge::cull`]), `mStts.Init` / `mCyl.Set` / `mCyl.SetStts` (`:116-119`, `dCcD` collider
    /// work), `set_mtx` (`:128`), `cMtx_copy(mtx1, mtx2)` (`:129`), the `"toge"` joint callback
    /// install and `mpModel->calc()` (`:131-139`), the two `Regist` calls and two `Move()` calls
    /// (`:141-145`) - all model, joint-matrix and `dBgW` work (RULE 3b).
    ///
    /// What survives is the switch test (`:121-126`), which needs `Switches`, and the two parameter
    /// decodes. `Create` never returns anything but TRUE, so `spawn` cannot refuse.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            // `unk470`, `unk474`, `mEventState`, `unk485` and `unk486` have no initialiser in
            // `Create`, so each takes `fopAcM_ct`'s `sBs_ClearArea` zero-fill
            // (`f_pc_base.cpp:98`). `mEventState = 0` is `STATE_IDLE`, `toge_move`'s inert
            // `case 0` (`:243-244`).
            spike: 0.0,
            bob: 0.0,
            switch_no: ((parameters >> SWITCH_SHIFT) & ((1 << SWITCH_WIDTH) - 1)) as u8,
            event_state: STATE_IDLE,
            want_out: 0,
            delay: 0,
            se_enabled: ((parameters >> SE_SHIFT) & ((1 << SE_WIDTH) - 1)) as u8,
            entered: false,
        }
    }

    /// `fopAcM_setCullSizeBox(this, -80.0f, 0.0f, -80.0f, 80.0f, 120.0f, 80.0f)`
    /// (`d_a_toge.cpp:115`) - the actor's one real custom cull box.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `if (fopAcM_isSwitch(this, mSwitchNo)) { unk470 = m_y_min; mEventState = 2; }`
    /// (`d_a_toge.cpp:123-126`). Nothing can change between Create and the first Execute, so
    /// reading the live switch here reproduces it exactly - the `htobi3.rs` shape.
    fn create_init(&mut self, switches: &Switches) {
        if switches.test(self.switch_no) {
            self.spike = Y_MIN;
            self.event_state = STATE_EXTENDED;
        }
        self.entered = true;
    }

    /// `daToge_c::_execute` (`:177-211`), minus `search_wind` / the wind-tag arm (`:187-200`, see
    /// OMITTED), `set_mtx` (`:204`), `mpBgW2->Move()` (`:206`) and `set_collision` (`:208`) - the
    /// last three being model, `dBgW` and `dCcD` work.
    fn execute(&mut self, switches: &Switches, events: &mut SimEvents) {
        // `if (mSwitchNo != 0xFF) { ... } else if (mSwitchNo == 0xFF) { search_wind(); ... }`
        // (:178-200). The second arm needs `fopAcM_SearchByName(fpcNm_WindTag_e)` (`:227`) and that
        // actor's `mOffsY` (`:192`), neither of which exists in this sim - see OMITTED.
        if self.switch_no != NO_SWITCH {
            // `if (fopAcM_isSwitch(this, mSwitchNo) != 0) { if (mEventState != 1) { unk485 = 1; } }
            // else if (!fopAcM_isSwitch(this, mSwitchNo) && mEventState != 4) { mEventState = 3; }`
            // (:179-185)
            if switches.test(self.switch_no) {
                if self.event_state != STATE_PENDING {
                    self.want_out = 1;
                }
            } else if self.event_state != STATE_CYCLE {
                self.event_state = STATE_RETRACT;
            }
        }
        // `toge_move();` (:202)
        self.toge_move(events);
    }

    /// `daToge_c::toge_move` (`:237-277`), including the two FALL-THROUGHS the decomp spells out:
    /// `case 1` falls into `case 2` (`:250-251`) and `case 3` falls into `case 4` (`:259-260`).
    fn toge_move(&mut self, events: &mut SimEvents) {
        // `f32 f31 = 30.0f; f32 f30 = 15.0f; int timer = 0xA; bool r30 = true;` (:238-241)
        match self.event_state {
            // `case 0: break;` (:243-244)
            STATE_IDLE => {}
            // ```
            // case 1:
            //     if (cLib_calcTimer(&unk486) != 0) break;
            //     toge_seStart(JA_SE_OBJ_TOGETOGE_IN);
            //     mEventState = 2;
            //     // Fallthrough
            // case 2:
            //     cLib_addCalc(&unk470, -150.0f, 0.1f, f31, f30);
            //     break;
            // ```
            // (:245-254)
            STATE_PENDING => {
                // `cLib_calcTimer(&unk486)` (`c_lib.h:85-91`) decrements a non-zero value by one
                // and returns it.
                if self.delay != 0 {
                    self.delay -= 1;
                    return;
                }
                self.se(events, SE_TOGETOGE_IN);
                self.event_state = STATE_EXTENDED;
                self.fallthrough_extend();
            }
            // `cLib_addCalc(&unk470, -150.0f, 0.1f, f31, f30);` (:253) - the decomp discards the
            // return, so `walk::add_calc`'s `()` signature is exact.
            STATE_EXTENDED => self.fallthrough_extend(),
            // ```
            // case 3:
            //     toge_seStart(JA_SE_OBJ_TOGETOGE_OUT);
            //     mEventState = 4;
            //     r30 = false;
            //     // Fallthrough
            // case 4:
            //     if (cLib_addCalc(&unk470, unk474, 0.1f, f31, f30) == 0) { ... }
            //     break;
            // ```
            // (:255-275)
            STATE_RETRACT => {
                self.se(events, SE_TOGETOGE_OUT);
                self.event_state = STATE_CYCLE;
                // `r30 = false;` (:258) - the local that suppresses the MOVE sound for this one
                // pass, which is why the retract is silent and the subsequent bobs are not.
                self.cycle(events, false);
            }
            // `case 4:` reached directly, i.e. with `r30` still true (:260-275).
            _ => self.cycle(events, true),
        }
    }

    /// The tail of `case 2` (`:251-254`): `cLib_addCalc(&unk470, -150.0f, 0.1f, f31, f30); break;`.
    fn fallthrough_extend(&mut self) {
        walk::add_calc(&mut self.spike, EXTEND_TARGET, ADD_SCALE, MAX_VEL, MIN_VEL);
    }

    /// The body of `case 4` (`:261-274`).
    ///
    /// ```
    /// if (cLib_addCalc(&unk470, unk474, 0.1f, f31, f30) == 0) {
    ///     if (unk470 < 0) { unk474 = 0; }
    ///     else if (unk485 != 0) { unk486 = timer; mEventState = 1; unk485 = 0; }
    ///     else { if (r30) { toge_seStart(JA_SE_OBJ_TOGETOGE_MOVE); } unk474 = -60.0f; }
    /// }
    /// ```
    ///
    /// `cLib_addCalc` returns `fabsf(target - *pValue)` (`c_lib.cpp:52`), so `== 0` is exactly
    /// `unk470 == unk474` after the step.
    fn cycle(&mut self, events: &mut SimEvents, play_move: bool) {
        walk::add_calc(&mut self.spike, self.bob, ADD_SCALE, MAX_VEL, MIN_VEL);
        if self.spike != self.bob {
            return;
        }
        if self.spike < 0.0 {
            // `unk474 = 0;` (:263) - the bob range is [0, -60].
            self.bob = 0.0;
        } else if self.want_out != 0 {
            // `unk486 = timer; mEventState = 1; unk485 = 0;` (:265-267)
            self.delay = REARM_TIMER;
            self.event_state = STATE_PENDING;
            self.want_out = 0;
        } else {
            // `if (r30) { toge_seStart(JA_SE_OBJ_TOGETOGE_MOVE); } unk474 = -60.0f;` (:269-272)
            if play_move {
                self.se(events, SE_TOGETOGE_MOVE);
            }
            self.bob = BOB_TARGET;
        }
    }

    /// `daToge_c::toge_seStart` (`:280-284`):
    /// `if (daToge_prm::getSeEnabled(this)) { fopAcM_seStart(this, i_seNum, 0); }`.
    fn se(&self, events: &mut SimEvents, id: u32) {
        if self.se_enabled != 0 {
            events.list.push(SimEvent::Se(id));
        }
    }
}

pub fn is_toge(name: &str) -> bool {
    TOGE_ACTORS.contains(&name)
}

pub fn control(
    switches: Res<Switches>,
    mut events: ResMut<SimEvents>,
    mut spikes: Query<&mut Toge>,
) {
    for mut toge in &mut spikes {
        if !toge.entered {
            toge.create_init(&switches);
        }
        toge.execute(&switches, &mut events);
    }
}