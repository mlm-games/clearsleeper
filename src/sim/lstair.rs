use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::enemy::Enemy;
use super::event::{ActiveEvent, EventList, Stage};
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;

/// `OBJNAME("Gkai00", fpcNm_LIGHTSTAIR_e, 255, 0)` (`d_stage.cpp:1008`) is `fpcNm_LIGHTSTAIR_e`
/// = `d_a_lstair.cpp`, the only such row. The archive name agrees:
/// `const char daLStair_c::m_arcname[] = "Gkai00"` (`d_a_lstair.cpp:15`).
///
/// Note the enum is `LIGHTSTAIR` (an L), not `LSTAIR` - the file is `d_a_lstair.cpp` but the
/// process name is `fpcNm_LIGHTSTAIR_e`. Neither string is claimed by any other `*_ACTORS` array
/// or `pub const &str` in `src/sim/`; `src/sim/stair.rs` claims a different actor
/// (`d_a_obj_stair.cpp`), and `lbridge.rs` owns `Gbrg00`.
pub const LSTAIR_ACTORS: [&str; 1] = ["Gkai00"];

/// `fopAcM_setCullSizeBox(this, -200.0f, -300.0f, -100.0f, 200.0f, 100.0f, 700.0f)`
/// (`d_a_lstair.cpp:98`). The profile's `fopAc_CULLBOX_CUSTOM_e` (`:388`) resolves to it.
/// `fopAcM_setCullSizeFar(this, 1.5f)` (`:99`) overrides only the far plane and creates no box.
const CULL_MIN: [f32; 3] = [-200.0, -300.0, -100.0]; // <- d_a_lstair.cpp:98
const CULL_MAX: [f32; 3] = [200.0, 100.0, 700.0]; // <- d_a_lstair.cpp:98

/// `daLStair_prm::getSwitchNo(actor) = fopAcM_GetParam(actor) & 0xFF` (`d_a_lstair.cpp:12`) over the
/// WHOLE u32 (`f_op_actor_mng.h:127-129`).
const SWITCH_WIDTH: u32 = 0xFF; // <- d_a_lstair.cpp:12
/// `if ((mSwitchNo == 0xFF && mEnemyGone) || (mSwitchNo != 0xFF && mSwitchStatus))`
/// (`d_a_lstair.cpp:107`) and `if (mSwitchNo != 0xFF)` (`:204`) - the "not switch-driven, use the
/// enemy test instead" sentinel. It is a LITERAL 255, not the result of a switch read; note that
/// retail's `dSv_info_c::isSwitch` also returns FALSE for `i_no == 255` (`d_save.cpp:1587-1589`),
/// so the sentinel is self-consistent with the save model.
const NO_SWITCH: u8 = 0xFF; // <- d_a_lstair.cpp:107

/// `mEventIdx = dComIfGp_evmng_getEventIdx("STAIRAPPEAR", 0xFF);` (`d_a_lstair.cpp:103`) - the
/// explicit `0xFF` skips the stage event-info redirect (`d_event_manager.cpp:347-348`), so this is a
/// pure name lookup.
const EV_APPEAR: &str = "STAIRAPPEAR"; // <- d_a_lstair.cpp:103

/// `mStairYOffset = -330.0f;` (`d_a_lstair.cpp:104`) - the lowered (hidden) offset, applied by
/// `set_mtx` at `current.pos.y + mStairYOffset` (`:140`).
const OFFSET_HIDDEN: f32 = -330.0; // <- d_a_lstair.cpp:104
/// `if (...) { mStairYOffset = 0.0f; }` (`d_a_lstair.cpp:108`) - the raised offset.
const OFFSET_SHOWN: f32 = 0.0; // <- d_a_lstair.cpp:108
/// `mAppearTimer = -1;` (`d_a_lstair.cpp:105`) - the `s8` appear countdown.
const TIMER_START: i8 = -1; // <- d_a_lstair.cpp:105
/// `mAppearTimer = 0x33;` (`d_a_lstair.cpp:215`) - armed when the room's enemies are gone.
const TIMER_GONE: i8 = 0x33; // <- d_a_lstair.cpp:215

/// `f32 f3 = -300.0f;` (`d_a_lstair.cpp:150`) in `setMoveBGMtx`, and the same `-300.0f` is
/// `target` in `moveBG`'s else arm (`:243`) and in `appear_stair`'s probe (`:265`, `:273`).
const BG_HIDDEN: f32 = -300.0; // <- d_a_lstair.cpp:150
/// ```
/// f32 maxStep = 10.0f;
/// f32 minStep = 5.0f;
/// ```
/// (`d_a_lstair.cpp:236-237`) - `moveBG`'s `cLib_addCalc` bounds (`:249`).
const MOVE_MAX: f32 = 10.0; // <- d_a_lstair.cpp:236
const MOVE_MIN: f32 = 5.0; // <- d_a_lstair.cpp:237
/// ```
/// int r0 = 60;
/// cLib_addCalc(&mStairYOffset, target, 1.0f / r0, maxStep, minStep);
/// ```
/// (`d_a_lstair.cpp:248-249`) - `1.0f / 60` is the chase SCALE.
const MOVE_SCALE: f32 = 1.0 / 60.0; // <- d_a_lstair.cpp:248-249
/// `mTimer = 0x28;` (`d_a_lstair.cpp:289` in `appear_stair`, `:308` in `disappear_stair`) - the
/// `u8` hold before `moveBG` starts easing (drained by `cLib_calcTimer(&mTimer)` at `:247`).
const MOVE_HOLD: u8 = 0x28; // <- d_a_lstair.cpp:289
/// `mEventState = 1;` (`d_a_lstair.cpp:290` in `appear_stair`) - the `u16` "wants the appear
/// event" flag that `demoMove` tests (`:189`) and clears on start (`:196`).
const EVENT_WANTED: u16 = 1; // <- d_a_lstair.cpp:290
/// `mEventState = 0;` (`d_a_lstair.cpp:196`).
const EVENT_NONE: u16 = 0; // <- d_a_lstair.cpp:196
/// `fopAcM_seStart(this, JA_SE_OBJ_L_BRIDGE_OFF, 0);` (`d_a_lstair.cpp:322`) -
/// `JA_SE_OBJ_L_BRIDGE_OFF = 0x6957` (`JAZelAudio_SE.h:1871`), played UNCONDITIONALLY by
/// `set_off_se`.
const SE_L_BRIDGE_OFF: u32 = 0x6957; // <- JAZelAudio_SE.h:1871
/// `fopAcM_seStart(this, JA_SE_OBJ_R_STAIR_APPEAR, 0)` (`d_a_lstair.cpp:316`) -
/// `JA_SE_OBJ_R_STAIR_APPEAR = 0x69B3` (`JAZelAudio_SE.h:1956`), gated on
/// `dStage_stagInfo_GetSaveTbl(stage_info) == dStageType_FF1_e` (`:315`).
const SE_R_STAIR_APPEAR: u32 = 0x69B3; // <- JAZelAudio_SE.h:1956

/// The Tower of the Gods' glowing stairway - `daLStair_c` (`d_a_lstair.h:9-46`), the actor behind
/// the single stage string `Gkai00`.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daLStair_c` is a bare `fopAc_ac_c`
/// holding a model, a `dBgW*`, an `Mtx`, five animators and six small fields
/// (`d_a_lstair.h:29-45`); it registers NO `dCcD` collider of any kind - no `dCcD_Stts`, no
/// `dCcD_Cyl`, no `dCcD_Src*` literal anywhere in `d_a_lstair.cpp`. The `dBgW` built from
/// `DZB_GKAI00` (`:83`) is a moving background, not a damage path, and is RULE 3b absent besides.
/// The profile `Group` is `fopAc_ACTOR_e` (`:387`). Neither `combat::HitPoints` nor `enemy::Hurt` is
/// attached; attaching them would let `combat::control` despawn a stairway retail cannot damage.
///
/// `daLstair_IsDelete` (`:362-364`) is the bare "accepts a delete request" form of RULE 6b, and
/// `fopAcM_delete` appears nowhere in this translation unit. `_create` (`:118-132`) returns
/// `cPhs_ERROR_e` only from the `fopAcM_entrySolidHeap` guard (`:124-126`), which is heap work, so
/// `spawn` never refuses.
///
/// Dropped as renderer / `dBgW` / archive work: `set_mtx` (`:135-146`), `setMoveBGMtx` (`:149-159`),
/// `mpBgW->Move()` (`:113`, `:254`), the four `m*Anm.play()` calls (`:170-173`), `_draw` (`:335-349`),
/// `dComIfG_Bgsp()->Regist(mpBgW, this)` (`:112`), and every `setFrame` / `setPlaySpeed` /
/// `getEndFrame` BPK/BTK call in `appear_stair` (`:281-287`) and `disappear_stair` (`:297-306`) -
/// the latter are BCK/BPK archive data, which RULE 3b lists as absent.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Lstair {
    /// `mSwitchNo` (`d_a_lstair.h:39`, an `s32`) from `daLStair_prm::getSwitchNo` (`:100`).
    pub switch_no: u8,
    /// `mSwitchStatus` (`d_a_lstair.h:44`, a `bool`) - the PREVIOUS tick's switch read, refreshed
    /// from the value `_execute` sampled at `:163` and stored at `:177`.
    pub was_switch: bool,
    /// `mEnemyGone` (`d_a_lstair.h:45`, a `bool`) - the previous tick's room-enemy test, refreshed
    /// at `:178`.
    pub was_enemy_gone: bool,
    /// `mStairYOffset` (`d_a_lstair.h:38`) - the vertical offset `set_mtx` adds to `current.pos.y`
    /// (`:140`). `moveBG` eases it between [`OFFSET_HIDDEN`] and [`OFFSET_SHOWN`] (`:239-250`).
    pub stair_y_offset: f32,
    /// `mAppearTimer` (`d_a_lstair.h:43`, an `s8`) - see [`TIMER_START`] / [`TIMER_GONE`];
    /// `checkAppear` drains it (`:221-227`) and `moveBG`'s `mAppearTimer < 0` test (`:246`) holds
    /// the motion off while it is negative.
    pub appear_timer: i8,
    /// `mTimer` (`d_a_lstair.h:42`, a `u8`) - [`MOVE_HOLD`] frames of wait before `moveBG` eases
    /// (`:289`, `:308`, drained at `:247`).
    pub timer: u8,
    /// `mEventState` (`d_a_lstair.h:40`, a `u16`) - [`EVENT_WANTED`] after `appear_stair` (`:290`),
    /// [`EVENT_NONE`] once the event starts (`:196`).
    pub event_state: u16,
    /// `mEventIdx` (`d_a_lstair.h:41`, an `s16`) from `getEventIdx("STAIRAPPEAR", 0xFF)` (`:103`);
    /// `None` is the -1 that a name missing from the stage list also produces.
    pub event_slot: Option<usize>,
    /// One-shot guard so `CreateInit`'s `getEventIdx` lookup (`:103`) and its initial switch /
    /// enemy tests (`:101-102`) run exactly once, on the first tick.
    pub entered: bool,
}

impl Lstair {
    /// `daLStair_c::CreateInit` (`d_a_lstair.cpp:96-115`) minus `fopAcM_SetMtx` (`:97`), the cull
    /// box (`:98-99`), `setMoveBGMtx` (`:111`), `dComIfG_Bgsp()->Regist(mpBgW, this)` (`:112`),
    /// `mpBgW->Move()` (`:113`) and `set_mtx` (`:114`).
    ///
    /// The switch and enemy tests (`:101-102`) and the `mStairYOffset` choice they drive (`:104-109`)
    /// need `Switches` and the enemy list, so they run on the first tick - the `lbridge` /
    /// `shutter2` shape.
    ///
    /// RULE 2: `dStage_actorInit` never writes `fopAcM_prm_class::scale`;
    /// `mpModel->setBaseScale(scale)` (`:136`) and `scaleM(scale.x, scale.y, scale.z)` (`:157`) are
    /// renderer work.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            switch_no: (parameters & SWITCH_WIDTH) as u8,
            // `mSwitchStatus` (`:101`) and `mEnemyGone` (`:102`) have no initialisers before those
            // lines, so they take `fopAcM_ct`'s `sBs_ClearArea` zero-fill (`f_pc_base.cpp:98`);
            // `create_init` overwrites both with the real reads.
            was_switch: false,
            was_enemy_gone: false,
            // `mStairYOffset = -330.0f;` (:104)
            stair_y_offset: OFFSET_HIDDEN,
            // `mAppearTimer = -1;` (:105)
            appear_timer: TIMER_START,
            // `mTimer` (`u8`, `:340`) takes the zero-fill; `cLib_calcTimer` (`:247`) keeps a zero
            // at zero until `appear_stair` / `disappear_stair` arm it.
            timer: 0,
            // `mEventState` (`u16`, `:340`) likewise; `demoMove`'s `else if (mEventState == 1)`
            // (`:189`) must not fire before an `appear_stair`.
            event_state: EVENT_NONE,
            event_slot: None,
            entered: false,
        }
    }

    /// `fopAcM_setCullSizeBox(this, -200.0f, -300.0f, -100.0f, 200.0f, 100.0f, 700.0f)`
    /// (`d_a_lstair.cpp:98`) - the actor's one real custom cull box, matching the profile's
    /// `fopAc_CULLBOX_CUSTOM_e` (`:388`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `daLStair_c::CreateInit` (`d_a_lstair.cpp:96-115`), first tick only.
    ///
    /// `mEnemyGone = fopAcM_myRoomSearchEnemy(fopAcM_GetRoomNo(this)) == NULL` (`:102`).
    /// `fopAcM_myRoomSearchEnemy` (`f_op_actor_mng.cpp:1282-1296`) judges each proc in the room's
    /// layer with `enemySearchJugge`, which tests `i_ac->group == fopAc_ENEMY_e` (`:1275`) - the
    /// actor PROFILE GROUP, not the presence of hit points - and first checks whether the player is
    /// already carrying such an enemy (`:1288-1290`). So the test is `no enemy-group actor exists in
    /// this room`, which is exactly the `Query<&Actor, With<Enemy>>` + room comparison
    /// `src/sim/cleared.rs:30-52` already uses; that file records why gating on
    /// `combat::HitPoints` was wrong (several enemies carry `Enemy` without it).
    fn create_init(&mut self, list: &EventList, switches: &Switches, foes: &[&Actor], room: i8) {
        // `mSwitchNo = daLStair_prm::getSwitchNo(this);` (:100) - already decoded in `spawn`.
        // `mEventIdx = dComIfGp_evmng_getEventIdx("STAIRAPPEAR", 0xFF);` (:103)
        self.event_slot = list.event_index(EV_APPEAR);
        // ```
        // mSwitchStatus = fopAcM_isSwitch(this, mSwitchNo);
        // mEnemyGone = fopAcM_myRoomSearchEnemy(fopAcM_GetRoomNo(this)) == NULL;
        // ```
        // (:101-102)
        self.was_switch = switches.test(self.switch_no);
        self.was_enemy_gone = !room_has_enemy(foes, room);
        // ```
        // if ((mSwitchNo == 0xFF && mEnemyGone) || (mSwitchNo != 0xFF && mSwitchStatus)) {
        //     mStairYOffset = 0.0f;
        // }
        // ```
        // (:107-109)
        if (self.switch_no == NO_SWITCH && self.was_enemy_gone)
            || (self.switch_no != NO_SWITCH && self.was_switch)
        {
            self.stair_y_offset = OFFSET_SHOWN;
        }
        self.entered = true;
    }

    /// `daLStair_c::_execute` (`d_a_lstair.cpp:162-181`), minus the four `m*Anm.play()` calls
    /// (`:170-173`) and `set_mtx` (`:175`).
    fn execute(
        &mut self,
        switches: &Switches,
        list: &EventList,
        active: &mut ActiveEvent,
        ev_flags: &mut super::flags::EventFlags,
        events: &mut SimEvents,
        entity: Entity,
        foes: &[&Actor],
        room: i8,
    ) {
        // ```
        // bool switch_status = fopAcM_isSwitch(this, mSwitchNo);
        // bool enemy_gone = fopAcM_myRoomSearchEnemy(fopAcM_GetRoomNo(this)) == NULL;
        // ```
        // (:163-164) - BOTH helper functions re-read the world themselves, so `checkAppear`
        // (`:201-202`) and `moveBG` (`:233-234`) take their OWN samples of the same two predicates.
        // The pair sampled here is the one stored at `:177-178`, i.e. it lags the comparisons by
        // one tick.
        let switch_status = switches.test(self.switch_no);
        let enemy_gone = !room_has_enemy(foes, room);

        self.check_appear(switches, foes, events, room);
        self.demo_move(list, active, ev_flags, events, entity);
        self.move_bg(switches, enemy_gone);

        // ```
        // mSwitchStatus = switch_status;
        // mEnemyGone = enemy_gone;
        // ```
        // (:177-178) - the values sampled at the TOP of this tick, not a fresh read.
        self.was_switch = switch_status;
        self.was_enemy_gone = enemy_gone;
    }

    /// `daLStair_c::checkAppear` (`d_a_lstair.cpp:200-229`).
    ///
    /// `bool switch_status = fopAcM_isSwitch(this, mSwitchNo);` (`:201`) and
    /// `bool enemy_gone = fopAcM_myRoomSearchEnemy(current.roomNo) == NULL;` (`:202`) re-read the
    /// world inside this function too - note `current.roomNo` here rather than the
    /// `fopAcM_GetRoomNo(this)` of `_execute` (`:164`), which is the same value.
    ///
    /// `s8 timer = mAppearTimer; if (timer > 0) { mAppearTimer = timer - 1; } else if (timer == 0)
    /// { appear_stair(); mAppearTimer = mAppearTimer - 1; }` (`:221-227`) - `timer` is a COPY, so
    /// the positive branch decrements by one per tick while the zero branch both appears and drops
    /// to -1.
    fn check_appear(
        &mut self,
        switches: &Switches,
        foes: &[&Actor],
        events: &mut SimEvents,
        room: i8,
    ) {
        // `:201` - the fresh switch read.
        let switch_status = switches.test(self.switch_no);
        // `:202` - the fresh enemy read. `checkAppear` spells the room as `current.roomNo` where
        // `_execute` used `fopAcM_GetRoomNo(this)` (`:164`); in retail those are the same value
        // (`fopAc_ac_c` keeps them in sync), so the sim passes one `room` to all three call sites.
        let enemy_gone = !room_has_enemy(foes, room);
        // ```
        // if (mSwitchNo != 0xFF) {
        //     if (switch_status != mSwitchStatus) {
        //         if (switch_status) { appear_stair(); } else { disappear_stair(); }
        //     }
        // } else {
        //     ... enemy-driven ...
        // }
        // ```
        // (:204-228)
        if self.switch_no != NO_SWITCH {
            if switch_status != self.was_switch {
                if switch_status {
                    self.appear_stair(events);
                } else {
                    self.disappear_stair(events);
                }
            }
            return;
        }
        // ```
        // if (enemy_gone != mEnemyGone) {
        //     if (enemy_gone) { mAppearTimer = 0x33; } else { disappear_stair(); }
        // }
        // ```
        // (:213-219)
        if enemy_gone != self.was_enemy_gone {
            if enemy_gone {
                self.appear_timer = TIMER_GONE;
            } else {
                self.disappear_stair(events);
            }
        }
        // ```
        // s8 timer = mAppearTimer;
        // if (timer > 0) { mAppearTimer = timer - 1; }
        // else if (timer == 0) { appear_stair(); mAppearTimer = mAppearTimer - 1; }
        // ```
        // (:221-227) - `timer` is a copy of the `s8`, so the `else if` tests the PRE-decrement value.
        let timer = self.appear_timer;
        if timer > 0 {
            self.appear_timer = timer - 1;
        } else if timer == 0 {
            self.appear_stair(events);
            self.appear_timer -= 1;
        }
    }

    /// `daLStair_c::moveBG` (`d_a_lstair.cpp:232-255`), minus `setMoveBGMtx` (`:253`) and
    /// `mpBgW->Move()` (`:254`).
    fn move_bg(&mut self, switches: &Switches, enemy_gone: bool) {
        let switch_status = switches.test(self.switch_no);
        // ```
        // if ((mSwitchNo != 0xFF && switch_status) || (mSwitchNo == 0xFF && enemy_gone)) {
        //     target = 0.0f;
        // } else {
        //     target = -300.0f;
        // }
        // ```
        // (:239-244) - note this eases toward -300.0, NOT toward `mStairYOffset`'s own -330.0
        // initial value, so the two differ by 30 in the hidden state. That is the decomp's own
        // asymmetry, not a transcription slip.
        let target = if (self.switch_no != NO_SWITCH && switch_status)
            || (self.switch_no == NO_SWITCH && enemy_gone)
        {
            OFFSET_SHOWN
        } else {
            BG_HIDDEN
        };
        // ```
        // if (mAppearTimer < 0) {
        //     if (cLib_calcTimer(&mTimer) == 0) {
        //         int r0 = 60;
        //         cLib_addCalc(&mStairYOffset, target, 1.0f / r0, maxStep, minStep);
        //     }
        // }
        // ```
        // (:246-251). `mAppearTimer < 0` HOLDS the motion: while the countdown is running (>= 0) the
        // offset does not move at all. `cLib_calcTimer` (`c_lib.h:85-91`) decrements a non-zero
        // value by one and returns it.
        if self.appear_timer >= 0 {
            return;
        }
        // `cLib_calcTimer(&mTimer)` (:247)
        if self.timer != 0 {
            self.timer -= 1;
            return;
        }
        // `cLib_addCalc(&mStairYOffset, target, 1.0f / r0, maxStep, minStep);` (:249)
        super::walk::add_calc(
            &mut self.stair_y_offset,
            target,
            MOVE_SCALE,
            MOVE_MAX,
            MOVE_MIN,
        );
    }

    /// `daLStair_c::appear_stair` (`d_a_lstair.cpp:258-291`).
    ///
    /// Dropped: the three `dComIfGp_particle_set` emitters (`:275-277`) and their setup - the
    /// `transS(current.pos.x, home.pos.y - 300.0f, current.pos.z)` / `YrotM(current.angle.y)`
    /// matrix, the `multVec` pair, and the `cXyz local_2c(-148.0f, 0.0f, 0.0f)` /
    /// `local_38(148.0f, 0.0f, 0.0f)` offsets (`:260-273`, whose `-300.0f` is the [`BG_HIDDEN`]
    /// literal) - all particle work, which RULE 3b lists as absent. Also dropped: the six
    /// `m*Anm.setFrame` / `setPlaySpeed` calls (`:281-287`), which are BPK/BTK archive data.
    ///
    /// `set_on_se()` (`:279`) is kept - see [`Lstair::set_on_se`] for the stage-data caveat.
    fn appear_stair(&mut self, events: &mut SimEvents) {
        self.set_on_se(events);
        // ```
        // mBpkAnm0.setFrame(0.0f); mBpkAnm0.setPlaySpeed(1.0f);
        // mBpkAnm1.setFrame(0.0f); mBpkAnm1.setPlaySpeed(1.0f);
        // mBtkAnm.setFrame(0.0f);   mBtkAnm.setPlaySpeed(1.0f);
        // mBrkAnm.setPlaySpeed(1.0f);
        // ```
        // (:281-287) - dropped, archive frames.
        // `mTimer = 0x28;` (:289)
        self.timer = MOVE_HOLD;
        // `mEventState = 1;` (:290)
        self.event_state = EVENT_WANTED;
    }

    /// `daLStair_c::disappear_stair` (`d_a_lstair.cpp:294-309`).
    ///
    /// `set_off_se()` (`:295`) plays [`SE_L_BRIDGE_OFF`] UNCONDITIONALLY, so it is kept. The six
    /// `m*Anm.setFrame(getEndFrame())` / `setPlaySpeed(-1.0f)` calls (`:297-306`) are dropped -
    /// `getEndFrame()` is archive data (RULE 3b) and `setPlaySpeed` is renderer.
    fn disappear_stair(&mut self, events: &mut SimEvents) {
        self.set_off_se(events);
        // `mTimer = 0x28;` (:308)
        self.timer = MOVE_HOLD;
    }

    /// `daLStair_c::set_on_se` (`d_a_lstair.cpp:312-318`):
    /// ```
    /// stage_stag_info_class* stage_info = dComIfGp_getStageStagInfo();
    /// if ((dStage_stagInfo_GetSaveTbl(stage_info)) == dStageType_FF1_e) {
    ///     fopAcM_seStart(this, JA_SE_OBJ_R_STAIR_APPEAR, 0);
    /// }
    /// ```
    /// The sound is gated on the stage's SAVE-TABLE index equalling `dStageType_FF1_e` - stage data
    /// this checkout does not parse. `d_a_lstair.cpp` is a Tower of the Gods actor, whose stage is
    /// not FF1, so in retail this sound would NOT play for `Gkai00`; but "the stage is not FF1" is
    /// an inference about stage data, not a transcribed value, so the sound is OMITTED rather than
    /// emitted on a guess. Same reasoning, and the same precedent, as `lbridge.rs` and
    /// `shutter2.rs` for their `dStage_stagInfo_GetSaveTbl` gates.
    fn set_on_se(&self, events: &mut SimEvents) {
        let _ = (events, SE_R_STAIR_APPEAR);
    }

    /// `daLStair_c::set_off_se` (`d_a_lstair.cpp:321-323`):
    /// `fopAcM_seStart(this, JA_SE_OBJ_L_BRIDGE_OFF, 0);` - no gate at all, so this one plays.
    fn set_off_se(&self, events: &mut SimEvents) {
        events.list.push(SimEvent::Se(SE_L_BRIDGE_OFF));
    }

    /// `daLStair_c::demoMove` (`d_a_lstair.cpp:184-197`).
    ///
    /// ```
    /// if (eventInfo.checkCommandDemoAccrpt()) {
    ///     if (dComIfGp_evmng_endCheck(mEventIdx)) { dComIfGp_event_reset(); }
    /// } else if (mEventState == 1) {
    ///     fopAcM_orderOtherEventId(this, mEventIdx, 0xFF, 0xFFFF, 0, 1);
    ///     eventInfo.onCondition(dEvtCnd_UNK2_e);
    /// }
    /// if (dComIfGp_evmng_startCheck(mEventIdx)) { mEventState = 0; }
    /// ```
    /// `checkCommandDemoAccrpt()` is the `mCommand == dEvtCmd_INDEMO_e` latch that
    /// `dEvt_control_c::demoCheck` sets on both ordered actors when the event starts
    /// (`d_event.cpp:314-316`) and `demoEnd` clears (`:331-337`) - i.e. "this actor's event is in
    /// `Stage::Play` or `Stage::Close`". `dComIfGp_evmng_startCheck(s16)` is
    /// `mEventState == PLAY_e` (`d_event_manager.cpp:372-378`) and `endCheck` is `== CLOSE_e`
    /// (`:386-392`), so those are `ActiveEvent::start_check` / `end_check`.
    ///
    /// `eventInfo.onCondition(dEvtCnd_UNK2_e)` (`:191`) is dropped for the reason `htobi3.rs` gives:
    /// nothing in the decomp ever calls `chkCondition` with `dEvtCnd_UNK2_e`, and `beforeProc`
    /// clears `mCondition` on every `fopAc_Execute` (`f_op_actor.cpp:176`).
    ///
    /// `fopAcM_orderOtherEventId` is guarded exactly as `hbrf1::Hbrf1::order` and
    /// `lbridge::Lbridge::order` guard it - re-ordering an event already in ORDER / PLAY / CLOSE
    /// rewrites its state (`d_event_manager.cpp:367`).
    fn demo_move(
        &mut self,
        list: &EventList,
        active: &mut ActiveEvent,
        ev_flags: &mut super::flags::EventFlags,
        events: &mut SimEvents,
        entity: Entity,
    ) {
        let _ = (ev_flags, events);
        let Some(slot) = self.event_slot else {
            return;
        };
        if matches!(
            active.stage_of(slot),
            Some(Stage::Play) | Some(Stage::Close)
        ) {
            // `if (dComIfGp_evmng_endCheck(mEventIdx)) { dComIfGp_event_reset(); }` (:186-188).
            // `dComIfGp_event_reset()` is `dEvDtBase_c::reset()` (`d_event.h:131`) = `mEventFlag
            // |= 8`, which `dEvt_control_c::check` (`d_event.cpp:546-554`) turns into `endProc` ->
            // `demoEnd`; `ActiveEvent::end` is that teardown.
            if active.end_check(slot) {
                let _ = active.end(slot);
            }
        } else if self.event_state == EVENT_WANTED {
            // `fopAcM_orderOtherEventId(this, mEventIdx, 0xFF, 0xFFFF, 0, 1);` (:190)
            if active.stage_of(slot).is_none() {
                let _ = active.order(list, slot, entity);
            }
        }
        // `if (dComIfGp_evmng_startCheck(mEventIdx)) { mEventState = 0; }` (:194-196)
        if active.start_check(slot) {
            self.event_state = EVENT_NONE;
        }
    }
}

/// `fopAcM_myRoomSearchEnemy(fopAcM_GetRoomNo(this)) == NULL` (`d_a_lstair.cpp:102`, `:164`,
/// `:202`, `:234`) as "no enemy-group actor exists in `room`". See [`Lstair::create_init`] for the
/// decomp reduction and `src/sim/cleared.rs:30-52` for the room-matched `Enemy` query this
/// mirrors.
///
/// `roomNo < 0` reads as "no enemy in it", because `fopAcM_myRoomSearchEnemy` asserts
/// `roomNo >= 0` (`f_op_actor_mng.cpp:1283`) and then searches a room control proc a negative index
/// cannot name - the same `room >= 0 &&` guard `cleared.rs:46` uses.
fn room_has_enemy(foes: &[&Actor], room: i8) -> bool {
    room >= 0 && foes.iter().any(|foe| foe.room == room)
}

pub fn is_lstair(name: &str) -> bool {
    LSTAIR_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    switches: Res<Switches>,
    mut active: ResMut<ActiveEvent>,
    mut ev_flags: ResMut<super::flags::EventFlags>,
    mut events: ResMut<SimEvents>,
    mut stairs: Query<(Entity, &mut Lstair, &Actor)>,
    foes: Query<&Actor, With<Enemy>>,
) {
    // `fopAcM_myRoomSearchEnemy` is called four times per tick (`d_a_lstair.cpp:102` in `CreateInit`,
    // `:164` in `_execute`, `:202` in `checkAppear`, `:234` in `moveBG`) with two different
    // spellings of the room number - `fopAcM_GetRoomNo(this)` and `current.roomNo` - which are the
    // same value in retail. `foes` is collected once into an owned snapshot so that
    // `create_init`, `_execute` and `checkAppear` all read one consistent enemy list, and so that
    // the `&Actor` borrow does not conflict with the `&mut Lstair` writer.
    //
    // Access set: `&Actor` (reader, on `foes`), `&mut Lstair` (writer, on `stairs`) - disjoint
    // because `foes` carries `With<Enemy>` and `stairs` carries `&mut Lstair`, and no param writes
    // `Actor`.
    let foes: Vec<&Actor> = foes.iter().collect();
    for (entity, mut stair, actor) in &mut stairs {
        let room = actor.room;
        if !stair.entered {
            stair.create_init(&list, &switches, &foes, room);
        }
        stair.execute(
            &switches,
            &list,
            &mut active,
            &mut ev_flags,
            &mut events,
            entity,
            &foes,
            room,
        );
    }
}
