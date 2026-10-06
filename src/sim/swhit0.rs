//! `daSwhit0_c` (`d_a_swhit0.h`) - the Light Stone / shock switch that the Deku Leaf and the
//! Skull Hammer strike. `Nonmatching` count: 0, so every token below is transcribed.
//!
//! Nothing in `src/sim/` claims `SW_HIT0`. `src/sim/swhammer.rs`'s `SWHAMMER_ACTOR = "MhmrSW0"`
//! (`:11`) is `fpcNm_Obj_Swhammer_e`, a different profile, and the two names share no prefix.

use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::flags::Switches;

/// `OBJNAME("SW_HIT0", fpcNm_SWHIT0_e, 255, 0)` (`d_stage.cpp:531`). `fpcNm_SWHIT0_e` is
/// `d_a_swhit0.cpp`; `CreateHeap` loads the model from the `"Always"` archive
/// (`d_a_swhit0.cpp:105`, `:113`, `:118`), which is not a stage name.
pub const SWHIT0_ACTORS: [&str; 1] = ["SW_HIT0"];

/// `l_cullSizeBox[0]` - `/* fopAc_CULLBOX_0_e */ fopAc_cullSizeBox(cXyz(-40.0f, 0.0f, -40.0f),
/// cXyz(40.0f, 125.0f, 40.0f))` (`f_op_actor_mng.cpp:558`). The profile's `fopAc_CULLBOX_0_e`
/// (`d_a_swhit0.cpp:495`) resolves here; the file calls neither `fopAcM_setCullSizeBox` nor
/// `fopAcM_SetMin` / `SetMax`, so that table entry is the whole box.
const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:558
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:558

/// `s32 daSwhit0_c::getSwNo() { return fopAcM_GetParam(this) & 0xFF; }` (`.cpp:73-75`).
const SW_NO_MASK: u32 = 0xFF; // <- d_a_swhit0.cpp:74
// `u8 daSwhit0_c::getEvNo() { return fopAcM_GetParam(this) >> 8; }` (`.cpp:78-80`) is the one
// accessor with no constant here: its sole readers are the two `dComIfGp_evmng_getEventIdx` calls
// (`:176`, `:179`) and the `fopAcM_orderOtherEventId` pair (`:255`, `:285`, `:301`), all
// stage-event-table work, so the decoded byte is not stored (RULE 7).
/// `u8 daSwhit0_c::getType() { return (fopAcM_GetParam(this) >> 0x10) & 0x0F; }` (`.cpp:83-85`).
const TYPE_SHIFT: u32 = 0x10; // <- d_a_swhit0.cpp:84
const TYPE_MASK: u32 = 0x0F; // <- d_a_swhit0.cpp:84
/// `s32 daSwhit0_c::getTimer()` (`.cpp:88-96`): `u8 param = (fopAcM_GetParam(this) >> 0x14);` then
/// `if (param == 0xFF) timer = 0;`. The `0xFF` is the "no timer" sentinel, not a value.
const TIMER_SHIFT: u32 = 0x14; // <- d_a_swhit0.cpp:89
const TIMER_NONE: u8 = 0xFF; // <- d_a_swhit0.cpp:92
/// `s32 daSwhit0_c::getSwNo2() { return home.angle.z & 0xFF; }` (`.cpp:99-101`) - the second switch
/// index lives in the placed roll angle, not in the parameter word, and the actor never writes
/// `home.angle.z` (only `shape_angle.z = 0; current.angle.z = 0;` at `:194-195`), so the stage
/// file's roll is the value for the actor's whole life.
const SW2_NO_MASK: u8 = 0xFF; // <- d_a_swhit0.cpp:100
/// `mOnTimer = (u8)getTimer() * 0x0F;` (`.cpp:339`) - the on-timer counts down one per frame, so
/// this multiplier is the number of frames the switch stays forced on.
const ON_TIMER_STEP: i16 = 0x0F; // <- d_a_swhit0.cpp:339
/// `mState` indices into `daSwhit0_Execute`'s `switch` (`.cpp:427-446`). States 1, 2 and 3 are the
/// strike-to-on ramp; 0 is off and 4 / 5 are the two on-states.
const STATE_OFF: u8 = 0; // <- d_a_swhit0.cpp:428
const STATE_ON: u8 = 4; // <- d_a_swhit0.cpp:440
const STATE_TIMER: u8 = 5; // <- d_a_swhit0.cpp:444
/// `if (getType() == 0x02)` in `actionOnWait` (`.cpp:328`) - the "releasable" variant that drops back
/// to off the moment its own switch clears.
const TYPE_RELEASE: u8 = 0x02; // <- d_a_swhit0.cpp:328

/// `daSwhit0_c` - the Light Stone / shock switch.
///
/// The struct is the live subset of `daSwhit0_c` (`d_a_swhit0.h:50-59`), and it keeps exactly the
/// switch-driven half of the six-state machine. `CreateInit` (`.cpp:141-183`) picks `mState = 4`
/// or `0` from its own switch (`:166-173`), and states 4 and 5 then run forever on switches alone:
///
/// * `actionOnWait` (`.cpp:323-344`) - the `getType() == 0x02` early exit (`:328-335`) and the
///   `getTimer()` countdown arm (`:337-341`) are both pure switch arithmetic and are transcribed
///   in full. Its `checkHit()` sound (`:324-326`) is the omitted half.
/// * `actionOnTimer` (`.cpp:347-366`) - `dComIfGs_isSwitch(getSwNo2(), ...)` (`:352`),
///   `mOnTimer--` (`:355-357`) and the `dComIfGs_offSwitch(getSwNo(), ...)` fall-through
///   (`:358-363`). Its `checkHit()` sound (`:348-350`) is the omitted half.
///
/// `getSwNo()` / `getSwNo2()` are `dComIfGs_isSwitch(i_no, i_roomNo)` calls with
/// `fopAcM_GetRoomNo(this)` as the room. `super::flags::Switches` models the single in-memory
/// `dSv_memBit_c::mSwitch[4]` word array, which is what a room-filtered read collapses to. RULE
/// 11b: the REAL index is passed unchanged and never down-shifted into 0..128, so a
/// `getSwNo()` or `getSwNo2()` of `>= 0x80` - which retail would route to the per-zone
/// `dSv_zoneBit_c` arrays (`d_save.h:983-984`) - reads false here. `0xFF` is also out of range and
/// reads false.
///
/// OMITTED, with the reason for each:
///
/// * `checkHit` (`.cpp:206-213`) - the state machine's ONLY off-to-on trigger. It is
///   `cLib_calcTimer(&mHitTimer) == 0 && mColSph.ChkTgHit() != 0` (`:207`), and `l_sph_src` (`:13-40`)
///   is a `dCcD_SrcSph` with `SrcObjTg Type = ~(AT_TYPE_LIGHT | AT_TYPE_UNK400000 | AT_TYPE_WIND |
///   AT_TYPE_UNK20000 | AT_TYPE_WATER)` = `0xFF1DFEFF` (`:20`) - i.e. hit by almost every AT type.
///   RULE 3b lists the `cCcD` actor-collision registry as absent, and no actor in this crate
///   publishes "was this AT collider hit this tick", so the test has no input. Substituting the
///   player's sword or a bomb blast would be the invented equivalent RULE 1 forbids. That makes
///   `actionOffWait` (`.cpp:243-271`) a no-op returning TRUE, and states 1 (`actionToOnReady`,
///   `:274-290`), 2 (`actionToOnOrder`, `:293-306`) and 3 (`actionToOnDemo`, `:309-320`)
///   unreachable - all three are entered only from `actionOffWait`'s `checkHit` arm (`:246-268`)
///   and are themselves gated on `dComIfGp_event_runCheck` (`:275`) and the stage event-info table
///   (`dComIfGp_evmng_getMyStaffId("SWITCH")` at `:296`,
///   `dComIfGp_evmng_getMyActIdx(mStaffId, action_table, ...)` at `:228`).
/// * `DemoProc` (`.cpp:216-240`) - `dComIfGp_evmng_getIsAddvance` / `getMyActIdx` / `cutEnd`, plus
///   the `dComIfGs_onSwitch(getSwNo(), ...)` at `:230` and `JA_SE_SHOCK_SW_ON`
///   (`JAZelAudio_SE.h:71`) at `:233`. Reached only from the omitted states 2 and 3.
/// * `mEventIdx` / `getEvNo` (`.cpp:176-180`) - `dComIfGp_evmng_getEventIdx("DEFAULT_SWITCH" /
///   "DEFAULT_SWITCH_NOSOUND", getEvNo())`, the stage event-info table. Its readers are
///   `actionOffWait`'s type-3 arm (`:255`) and the three omitted states.
/// * `CreateInit`'s `mColCyl.Set(l_cyl_src)` pair (`.cpp:155-157`) and `onFlag(0x02)` (`:159`),
///   `mColSph.Set(l_sph_src)` pair (`.cpp:162-164`), `mColStatus.Init(0xFF, 0xFF, this)` (`:146`),
///   and `daSwhit0_Execute`'s two `dComIfG_Ccsp()->Set` calls (`.cpp:448-452`) - collider work.
///   `l_cyl_src`'s `SrcObjCo SPrm = cCcD_CoSPrm_Set_e | cCcD_CoSPrm_IsPlayer_e |
///   cCcD_CoSPrm_VsGrpAll_e` (`.cpp:51`) with `Radius 25.0f` / `Height 100.0f` (`:67-68`) is the AT
///   the stone presents; `l_sph_src`'s is `Radius 35.0f` (`:38`).
/// * `mFlags` (`.h:56`) and `onFlag` / `offFlag` / `checkFlag` (`.h:61-63`) - `0x01` is read only by
///   `draw`'s material-tev-colour choice (`.cpp:390-395`) and `0x02` only by `daSwhit0_Execute`'s
///   `checkFlag(0x02)` guard (`.cpp:448`). Both readers are omitted, so the word is not stored.
/// * `mTimer` (`.h:52`) - `actionToOnReady`'s `mTimer = 5` / `mTimer--` (`.cpp:250`, `:280-281`),
///   state 1 only.
/// * `mHitTimer` (`.h:50`) - written by `checkHit` (`.cpp:208`) and read by nothing else.
/// * `mRetType` (`.h:53`) and `decisionRtType` (`.cpp:127-138`, thresholds `-0x4E20` / `0x4E20` =
/// -20000 / 20000 at `:131`) - `mRetType`'s only use is `if (mRetType == 0)` in `CreateInit`
///   (`:154`), which creates the AT cylinder. With that omitted, neither the byte nor the placed
///   pitch that selects it has a reader.
/// * `attention_info.position`'s three `+=` lines (`.cpp:148-150`, the
///   `cM_ssin(home.angle.x) * 65.0f * cM_ssin(home.angle.y)` offsets off `home.pos` per
///   `f_op_actor.cpp:295`) and `eyePos = attention_info.position` (`.cpp:152`) - the sum's only
///   readers are `mColSph.SetC(attention_info.position)` (`:164`), a collider, and `eyePos` itself.
/// * `mStaffId` (`.h:59`), `CreateHeap` (`.cpp:104-124`), `setDrawMtx` (`:369-375`), `draw` (`:377-415`),
///   `mAnm.play()` / `mTexAnm.play()` (`.cpp:424-425`), `CreateInit`'s `fopAcM_SetMtx` (`:143`) and
///   `daSwhit0_Delete` (`:463-466`) - BCK / BTK archives (frame counts are archive data, RULE 3b),
///   matrices, renderer and destructor.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. The stone registers a Co collider
/// (`.cpp:51`) and a Tg collider (`.cpp:20`), but its AT type is 0 (`.cpp:17`) - it is never an
/// attacker - and `JA_SE_OBJ_COL_SWC_NSTONE` (`:267`, `:325`, `:349`) is a sound, not a hit. The
/// profile `Group` is `fopAc_ACTOR_e` (`:494`). Neither `combat::HitPoints` nor `enemy::Hurt`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwHit0 {
    /// `getSwNo()` (`.cpp:73-75`) - the stone's own switch.
    pub switch_no: u8,
    /// `getSwNo2()` (`.cpp:99-101`) - `home.angle.z & 0xFF`, the release switch.
    pub switch2_no: u8,
    /// `getType()` (`.cpp:83-85`).
    pub kind: u8,
    /// `mState` (`.h:51`), seeded by `CreateInit` (`.cpp:166-173`).
    pub state: u8,
    /// `mOnTimer` (`.h:55`) - `s16`, `(u8)getTimer() * 0x0F` frames.
    pub on_timer: i16,
}

impl SwHit0 {
    /// `daSwhit0_c::CreateInit` (`.cpp:141-183`), minus `setDrawMtx` (`:142`),
    /// `fopAcM_SetMtx` (`:143`), `decisionRtType` (`:144`), `mColStatus.Init` (`:146`), the three
    /// `attention_info.position +=` lines (`:148-150`), `eyePos` (`:152`), the `mColCyl` /
    /// `mColSph` pair (`:154-164`), `onFlag(0x01)` / `offFlag(0x01)` (`:168`, `:172`) and the
    /// `dComIfGp_evmng_getEventIdx` pair (`:175-180`).
    ///
    /// `dComIfGs_isSwitch(getSwNo(), fopAcM_GetRoomNo(this))` (`:166`) - see the struct doc on the
    /// room argument and on RULE 11b.
    ///
    /// `create` (`.cpp:191-203`) clears `shape_angle.z` and `current.angle.z` (`:194-195`) - the
    /// actor never rolls - and can only return `cPhs_ERROR_e` from
    /// `fopAcM_entrySolidHeap` (`:197-199`), which is heap work, so there is no refusal.
    pub fn spawn(parameters: u32, angle: [i16; 3], switches: &Switches) -> Self {
        let switch_no = (parameters & SW_NO_MASK) as u8;
        let state = if switches.test(switch_no) {
            STATE_ON
        } else {
            STATE_OFF
        };
        Self {
            switch_no,
            switch2_no: angle[2] as u8 & SW2_NO_MASK,
            kind: ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8,
            state,
            on_timer: 0,
        }
    }

    /// `getTimer()` (`.cpp:88-96`) recomputed from the parameter word each call, exactly as retail
    /// does at `:337`. `u8 param = (fopAcM_GetParam(this) >> 0x14);` truncates to a byte, and
    /// `0xFF` is remapped to zero.
    fn timer_of(parameters: u32) -> u8 {
        let param = (parameters >> TIMER_SHIFT) as u8;
        if param == TIMER_NONE { 0 } else { param }
    }

    /// `actionOnWait` (`.cpp:323-344`), minus `checkHit()` and its `JA_SE_OBJ_COL_SWC_NSTONE`.
    fn action_on_wait(&mut self, parameters: u32, switches: &Switches) {
        // `if (getType() == 0x02) { if (!isSwitch(getSwNo())) { offFlag(0x01); mState = 0; return
        // TRUE; } }` (:328-335) - `offFlag` has no reader left, so only the state change stays.
        if self.kind == TYPE_RELEASE && !switches.test(self.switch_no) {
            self.state = STATE_OFF;
            return;
        }
        // `u8 timer = getTimer(); if (timer != 0) { mOnTimer = (u8)getTimer() * 0x0F; mState = 5; }`
        // (:337-341)
        let timer = Self::timer_of(parameters);
        if timer != 0 {
            self.on_timer = i16::from(timer) * ON_TIMER_STEP;
            self.state = STATE_TIMER;
        }
    }

    /// `actionOnTimer` (`.cpp:347-366`), minus `checkHit()` and its `JA_SE_OBJ_COL_SWC_NSTONE`.
    fn action_on_timer(&mut self, switches: &mut Switches) {
        // `if (dComIfGs_isSwitch(getSwNo2(), fopAcM_GetRoomNo(this))) { mState = 4; }` (:352-354)
        if switches.test(self.switch2_no) {
            self.state = STATE_ON;
            return;
        }
        // `else if (mOnTimer > 0) { mOnTimer--; }` (:355-357)
        if self.on_timer > 0 {
            self.on_timer -= 1;
            return;
        }
        // `else { offFlag(0x01); mState = 0; dComIfGs_offSwitch(getSwNo(), ...); }` (:358-363)
        self.state = STATE_OFF;
        switches.clear(self.switch_no);
    }

    /// `daSwhit0_Execute` (`.cpp:423-454`): `mAnm.play()` / `mTexAnm.play()` (`:424-425`) are BCK
    /// and BTK archive work, the `switch (mState)` (`:427-446`) keeps only its states 4 and 5
    /// (states 1, 2 and 3 are unreachable without `checkHit`, documented on the struct), and the
    /// two `dComIfG_Ccsp()->Set` calls (`:448-452`) are collider work.
    fn execute(&mut self, parameters: u32, switches: &mut Switches) {
        match self.state {
            STATE_ON => self.action_on_wait(parameters, switches),
            STATE_TIMER => self.action_on_timer(switches),
            _ => {}
        }
    }

    /// `l_cullSizeBox[0]` via the profile's `fopAc_CULLBOX_0_e` (`d_a_swhit0.cpp:495`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_swhit0(name: &str) -> bool {
    SWHIT0_ACTORS.contains(&name)
}

/// One system, two params. Access set per param: `Switches` (W - `action_on_timer` clears a bit),
/// `SwHit0` (RW). `parameters` is read straight off `Actor`, which this query does not touch, so
/// there is no intra-system ambiguity.
pub fn control(mut switches: ResMut<Switches>, mut stones: Query<(&Actor, &mut SwHit0)>) {
    for (actor, mut stone) in &mut stones {
        let stone = &mut *stone;
        stone.execute(actor.parameters, &mut switches);
    }
}
