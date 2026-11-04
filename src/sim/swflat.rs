use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::barrel::Barrel;
use super::events::{SimEvent, SimEvents};
use super::flags::{OrderFlags, Switches};
use super::goal_flag::EVT_CND_UNK2;
use super::player::Player;

/// `OBJNAME("Hfbot1A", fpcNm_Obj_Swflat_e, 0, 0)`, `OBJNAME("Hfbot1B", fpcNm_Obj_Swflat_e, 1, 0)`
/// and `OBJNAME("Hfbot1C", fpcNm_Obj_Swflat_e, 2, 0)` (`tww/src/d/d_stage.cpp:962-964`) are the only
/// three `fpcNm_Obj_Swflat_e` rows in the stage object-name table.
///
/// The table's third field is `dStage_objectNameInf::argument`, which `dStage_actorCreate` copies
/// into `i_actorPrm->argument` (`tww/src/d/d_stage.cpp:1301-1310`) - NOT into the PARAMETER. The
/// three fields `prmGetType`, `prmGetSwNo` and `prmGetSwNo2` read (`d_a_obj_swflat.h:17-19`) come
/// from the room actor record's parameter word, which is this sim's `Actor::parameters`. So the
/// three stage strings differ only in the culling argument no `Act_c` method reads, and each one can
/// be any switch type with any switch number.
///
/// Neither string is claimed by any other `*_ACTORS` array or `pub const &str` in `src/sim/`.
/// `M_arcname[] = "Hfbot"` (`d_a_obj_swflat.cpp:78`) is the archive, and the model and break
/// animation both come out of it (`:83`, `:90`), which is renderer work.
pub const SWFLAT_ACTORS: [&str; 3] = ["Hfbot1A", "Hfbot1B", "Hfbot1C"];

/// `fopAcM_setCullSizeSphere(this, 0.0f, 0.0f, 0.0f, 90.0f)` (`d_a_obj_swflat.cpp:110`) under the
/// profile's `fopAc_CULLSPHERE_CUSTOM_e` (`:433`). The sphere is centred on the actor, so the box
/// the draw layer sizes from it is the sphere's own cube - the `kamome::Kamome::cull` reduction of
/// `fopAc_cullSizeSphere` to the same two arrays.
const CULL_MIN: [f32; 3] = [-90.0, -90.0, -90.0]; // <- d_a_obj_swflat.cpp:110
const CULL_MAX: [f32; 3] = [90.0, 90.0, 90.0]; // <- d_a_obj_swflat.cpp:110

/// `/* Radius */ 10.0f` (`d_a_obj_swflat.cpp:60`), the `l_cyl_src` cylinder the plate registers with
/// (`:112`) and the radius the un-pressed plate returns to (`:243`).
const CYL_RADIUS: f32 = 10.0; // <- d_a_obj_swflat.cpp:60
/// `/* Height */ 20.0f` (`d_a_obj_swflat.cpp:61`) - the plate's collider height, which NOTHING in
/// `Execute` changes: only `SetR` is called (`:240`, `:243`), and the `SetH` at `:226` sits behind
/// the `l_HIO.mTune` gate that never opens.
const CYL_HEIGHT: f32 = 20.0; // <- d_a_obj_swflat.cpp:61
/// `mCyl.SetR(15.0f)` (`d_a_obj_swflat.cpp:240`) - the wider radius a pressed plate keeps while it
/// stays pressed, so one actor standing on it does not flicker the press.
const PRESS_RADIUS: f32 = 15.0; // <- d_a_obj_swflat.cpp:240

/// `mEventTimer = 30` (`d_a_obj_swflat.cpp:312`), the press gate's freeze at `:237`.
const EVENT_LOCKOUT: i16 = 30; // <- d_a_obj_swflat.cpp:312
/// `mFinishTimer = l_HIO.mFinishTime` (`d_a_obj_swflat.cpp:249`, `:268`), where `mFinishTime` is the
/// `daObjSwflat_HIO_c` constructor's `mFinishTime = 30` (`:75`) - the debug value, because the file
/// has no hostIO panel to change it (`genMessage` is `UNUSED(ctx)` at `:19`).
const FINISH_TIME: i16 = 30; // <- d_a_obj_swflat.cpp:75

/// `mOnAlpha = 250` (`d_a_obj_swflat.cpp:141`, `:265`) and `mOffAlpha = 250` (`:150`, `:293`), the
/// emitter alpha both ramps chase, and `mOnAlpha += 25` (`:272`) / `mOffAlpha -= 25` (`:278`) the
/// step they chase it with. Every value is a multiple of the step, so the `== 0` tests at `:280` and
/// `:301` land exactly and the `u8` never wraps.
const ALPHA_FULL: u8 = 250; // <- d_a_obj_swflat.cpp:141
const ALPHA_STEP: u8 = 25; // <- d_a_obj_swflat.cpp:272

/// `mDoAud_seStart(JA_SE_OBJ_FLAT_SW_ON, &current.pos)` (`d_a_obj_swflat.cpp:255`) -
/// `JA_SE_OBJ_FLAT_SW_ON = 0x6960` (`tww/include/JAZelAudio/JAZelAudio_SE.h:1880`).
const SE_ON: u32 = 0x6960; // <- JAZelAudio_SE.h:1880
/// `mDoAud_seStart(JA_SE_OBJ_FLAT_SW_LIGHT, &current.pos)` (`d_a_obj_swflat.cpp:256`) -
/// `JA_SE_OBJ_FLAT_SW_LIGHT = 0x7027` (`tww/include/JAZelAudio/JAZelAudio_SE.h:2126`), the loop that
/// sounds while a not-yet-finished plate is held down.
const SE_LIGHT: u32 = 0x7027; // <- JAZelAudio_SE.h:2126

/// `PRM_TYPE_W = 2, PRM_TYPE_S = 0` (`d_a_obj_swflat.h:13`), the two arguments of
/// `daObj::PrmAbstract(this, PRM_TYPE_W, PRM_TYPE_S)` at `:19`, which is
/// `((1 << width) - 1) & (param >> shift)` (`tww/include/d/d_a_obj.h:11-15`). Two bits, so the type
/// is 0, 1, 2 or 3; only 0, 1 and 2 have any behaviour, and 3 behaves as "neither 0 nor 1 nor 2".
const TYPE_SHIFT: u32 = 0; // <- d_a_obj_swflat.h:13
const TYPE_MASK: u32 = 0x03; // <- d_a_obj_swflat.h:13
/// `PRM_SW_W = 8, PRM_SW_S = 8` (`d_a_obj_swflat.h:14`) for `prmGetSwNo` (`:17`), stored into the
/// `u8 mSwitch` (`d_a_obj_swflat.h:50`), so the eight bits are the whole field.
const SWITCH_SHIFT: u32 = 8; // <- d_a_obj_swflat.h:14
const SWITCH_MASK: u32 = 0xFF; // <- d_a_obj_swflat.h:14
/// `PRM_SW2_W = 8, PRM_SW2_S = 16` (`d_a_obj_swflat.h:15`) for `prmGetSwNo2` (`:18`).
const SWITCH2_SHIFT: u32 = 16; // <- d_a_obj_swflat.h:15
const SWITCH2_MASK: u32 = 0xFF; // <- d_a_obj_swflat.h:15

/// The switch type `prmGetType()` answers for the plate that finishes itself the first time it is
/// lit, which is what arms `mFinishTimer` at `:267-270`.
const TYPE_ONCE: u32 = 0; // <- d_a_obj_swflat.cpp:267
/// The type that falls back to `mActive = 0` when nothing presses, i.e. the momentary plate (`:241`).
const TYPE_MOMENTARY: u8 = 1; // <- d_a_obj_swflat.cpp:241
/// The type `Create`'s conversion (`:125-133`) and `Execute`'s `switch2` re-read (`:246`) both test
/// for: a plate that waits for another switch and then becomes a [`TYPE_ONCE`].
const TYPE_GATED: u32 = 2; // <- d_a_obj_swflat.cpp:246

/// The presser the plate's own collision object recorded this pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Presser {
    /// `fopAcM_GetProfName(actor) == fpcNm_PLAYER_e` (`d_a_obj_swflat.cpp:234`), which is the only
    /// thing that raises `playerHit`.
    Player,
    /// Any other actor that survived the `fopAcM_CheckStatus(actor, fopAcStts_FREEZE_e)` gate at
    /// `:232`.
    Prop,
}

/// The Tower-of-the-Gods flat floor switch - `daObjSwflat::Act_c` (`d_a_obj_swflat.h:10-56`), the
/// `Hfbot1A`/`Hfbot1B`/`Hfbot1C` plate that lights when something stands on it and turns itself into
/// a one-shot when it does.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `l_cyl_src` (`d_a_obj_swflat.cpp:35-63`)
/// carries `/* SrcObjAt SPrm */ 0` (`:41`) and `/* SrcObjTg SPrm */ 0` (`:43`), so the plate's one
/// cylinder has neither its attack nor its target bit set: it hits nobody and nothing can hit it.
/// Its only contact role is the CO side - `/* SrcObjCo SPrm */ cCcD_CoSPrm_Set_e |
/// cCcD_CoSPrm_IsOther_e | cCcD_CoSPrm_VsGrpAll_e | cCcD_CoSPrm_NoCrr_e |
/// cCcD_CoSPrm_NoCoHitInfSet_e` (`:44`). Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
///
/// The two CO bits are why this module surveys geometry instead of registering a body.
/// `cCcS::SetCoCommonHitInf` writes a partner's hit info only when the OTHER side lacks the
/// `NoCoHitInfSet` bit (`tww/src/SSystem/SComponent/c_cc_s.cpp:234-242`), so the plate records every
/// partner and NO partner ever records the plate; and `cCcS::SetPosCorrect` returns as soon as
/// either side has `NoCrr` (`tww/src/SSystem/SComponent/c_cc_s.cpp:253-255`), so the plate neither
/// pushes anything out nor is pushed. The plate's collider therefore has exactly one consumer: the
/// `mCyl.ChkCoHit()` at `d_a_obj_swflat.cpp:230`. `super::bodies` has no `Hfbot` row and the port's
/// player is spawned with no `ContactBodies` at all (`super::contact_run`'s `player_velocity`
/// documents it), so the pass could not answer the question even if the plate registered.
///
/// What the survey covers, and the proof for each:
/// * the player, whose body cylinder `l_cyl_src` (`tww/src/d/actor/d_a_player_main_data.inc:43`) is
///   `Radius 30.0f` / `Height 125.0f` (`:68-69`) with CO `Set | IsEnemy | VsGrpAll` (`:52`) and is
///   registered every frame (`tww/src/d/actor/d_a_player_main.cpp:9789`, `:9797`). The base is the
///   lower toe joint, which for the standing pose `Execute` sees is the actor's own feet, and the
///   height is `40.1f + (neck - toe)` (`:9787`), i.e. the same standing volume. The player's profile
///   status is exactly `fopAcStts_FREEZE_e` (`tww/src/d/actor/d_a_player_main.cpp:13171`), so it
///   passes the gate at `d_a_obj_swflat.cpp:232` and is the only source of `playerHit`.
/// * the barrel, `Ktarux`, whose `M_cyl_src` is `Radius l_l_radius` (`tww/src/d/actor/
///   d_a_obj_barrel.cpp:53`, `l_l_radius = 50.0f` at `:16`) / `Height 100.0f` (`:55`) with CO
///   `Set | IsOther | VsGrpAll` (`:37`, `super::bodies`' own reading of the same word), registered at
///   `mCyl.SetC(current.pos)` + `dComIfG_Ccsp()->Set(&mCyl)` (`:712-713`) in every mode that survives
///   `mode_proc_call`, and whose profile carries `fopAcStts_FREEZE_e`
///   (`tww/src/d/actor/d_a_obj_barrel.cpp:795`).
///
/// Not surveyed, each with the reason:
/// * the movebox: `mCyl.OffCoSPrmBit(cCcD_CoSPrm_Set_e)` in its create (`tww/src/d/actor/
///   d_a_obj_movebox.cpp:1321`) and nothing in the file turns the bit back on, so a box is never a
///   CO partner at all - `cCcS::ChkCo` skips every object whose CO is not set
///   (`tww/src/SSystem/SComponent/c_cc_s.cpp:186`). A box on a plate presses nothing in retail.
/// * the pot and the `Tbox`: their cylinders are per-type, `mCyl.SetR(data().mAcchCirRad)` and
///   `mCyl.SetH(data().m70)` (`tww/src/d/actor/d_a_tsubo.cpp:1242-1243`) and
///   `mCyl.SetR(attr().mRadius)` / `mCyl.SetH(attr().mHeight)` (`tww/src/d/actor/d_a_obj_try.cpp:253-
///   254`), numbers this sim has not located; and both files toggle the CO bit per mode
///   (`d_a_tsubo.cpp:1686`, `:1711`, `d_a_obj_try.cpp:366`, `:409`, `:467`). One row would be a
///   guess, so neither is a candidate.
/// * the FREEZE NPCs and statues that do register a CO - `fopAcStts_FREEZE_e` at
///   `tww/src/d/actor/d_a_am2.cpp:1527` with the CO of `:1426`, `d_a_mt.cpp:3013` with `:2820`,
///   `d_a_npc_os.cpp:2559` with `:91`, `d_a_npc_cb1.cpp:3457` with `:102`,
///   `d_a_npc_md.cpp:6286` with `:236` - and the `koisi1`/`koisi2` stone, `fopAcStts_FREEZE_e` at
///   `tww/src/d/actor/d_a_stone.cpp:1023`, whose cylinder is `attr()`-driven and which this sim
///   ports only as `daStone2` (`super::stone`). Their walk-over press is unmodelled.
///
/// consequence of that shortlist is a real behaviour difference and is worth stating: retail's
/// `GetCoHitAc()` (`tww/include/d/d_cc_d.h:410`) is the general CO's last recorded partner
/// (`SetCoHitApid` at `d_cc_s.cpp:169`, `:173`), so a non-FREEZE partner that crosses the plate
/// LATER in the pass masks a press that already registered - an Armos walking off a plate Link is
/// standing on stops the press. The survey only knows about the two sources above, so it cannot mask,
/// and it never sees a presser the plate would have switched on for. Both halves are the same
/// omission: a press the port does not see is a switch the port does not set, and a mask the port
/// does not model is a press it holds one too long.
///
/// `IsDelete` returns `TRUE` unconditionally (`d_a_obj_swflat.cpp:378-380`), so the plate accepts
/// every delete request and never removes itself; `Mthd_Create` (`:163-171`) fails only out of
/// `dComIfG_resLoad` and `MoveBGCreate`, both heap work, so `spawn` never refuses.
///
/// Dropped as renderer, `dBgW` or hostIO work, with the decomp line each is read at: `CreateHeap`
/// (`:82-104`), the `mBrk` break animation and everything that drives it - `mFrame` (`:117`, `:257-
/// 260`, `:286-289`, `:354-357`), `mActiveFrame = mBrk->getEndFrame() - 5.0f` (`:118`) and
/// `mBrk->setFrame(mBrk->getEndFrame())` (`:145`) - whose only reader is `Draw`'s `mBrk->entry`
/// (`:370`); `set_mtx` / `init_mtx` / `M_tmp_mtx` (`:207-218`, `:360-361`); `Draw` itself (`:367-375`);
/// `Mthd_Create`'s `MoveBGCreate(M_arcname, dRes_INDEX_HFBOT_DZB_HFBOT1_e, NULL, -1)` (`:167`) and
/// `Mthd_Delete`'s `MoveBGDelete` (`:197`), with `MoveBGExecute`'s `mpBgW->Move()`
/// (`tww/src/d/d_bg_s_movebg_actor.cpp:99`), the plate's own registered floor, which is flush with
/// the room floor it sits in; `mParticlePos` (`:119`), the emitter positions and the
/// `setGlobalAlpha` / `becomeInvalidEmitter` calls (`:273`, `:279-282`, `:296`, `:300-303`), kept here
/// as the two emitter latches plus their alphas because `activated` - the thing that arms the event
/// - fires only on the `mpOnEmitter == NULL` edge at `:263`; and the whole `daObjSwflat_HIO_c` panel,
/// `l_HIO.mTune` / `mRadius` / `mHeight` (`:224-227`), `mResetSwitches` (`:121-124`), `mEventTime`
/// (`:318`) and the `mNo` / `mUsers` registration (`:157-158`, `:193-196`), all of which are read at
/// their constructor defaults (`:68-76`).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Swflat {
    /// `mType` (`d_a_obj_swflat.h:47`) from `mType = prmGetType()` (`d_a_obj_swflat.cpp:114`) - and
    /// the field `:241`'s momentary test and `:247`'s conversion write, as opposed to `kind`, which
    /// is the parameter `:246` and `:267` re-read.
    pub m_type: u8,
    /// `prmGetType()` (`d_a_obj_swflat.h:19`) - the raw parameter, read every frame at
    /// `d_a_obj_swflat.cpp:246` and `:267` rather than from `mType`.
    pub kind: u8,
    /// `mSwitch` (`d_a_obj_swflat.h:50`) from `prmGetSwNo()` (`d_a_obj_swflat.cpp:115`), the switch
    /// every `fopAcM_onSwitch(this, mSwitch)` (`:261`, `:274`, `:329`) and `fopAcM_offSwitch` (`:290`)
    /// writes.
    pub switch: u8,
    /// `mSwitch2` (`d_a_obj_swflat.h:51`) from `prmGetSwNo2()` (`d_a_obj_swflat.cpp:116`), the gate
    /// switch a [`TYPE_GATED`] plate watches at `:246`.
    pub switch2: u8,
    /// `mActive` (`d_a_obj_swflat.h:48`) - the pressed-or-not latch the whole `:254` body hangs on.
    pub active: u8,
    /// `mPrevActive` (`d_a_obj_swflat.h:49`), read once for the ON sound edge
    /// (`d_a_obj_swflat.cpp:255`) and written at the very end of the tick (`d_a_obj_swflat.cpp:362`).
    pub prev_active: u8,
    /// `mFinished` (`d_a_obj_swflat.h:54`) - the plate has lit once and is on its way out.
    pub finished: u8,
    /// `mFinishTimer` (`d_a_obj_swflat.h:46`) - the countdown that drains at
    /// `d_a_obj_swflat.cpp:335-336`, before the fade-out tail at `d_a_obj_swflat.cpp:337-359` runs.
    pub finish_timer: i16,
    /// `mEventState` (`d_a_obj_swflat.h:55`) - the three-arm switch at `:307-332`.
    pub event_state: u8,
    /// `mEventTimer` (`d_a_obj_swflat.h:45`) - nonzero ONLY while the press gate at `:237` is
    /// frozen, which is the plate's whole 30-frame lockout.
    pub event_timer: i16,
    /// `mpOnEmitter` (`d_a_obj_swflat.h:41`) as the latch the lit emitter exists.
    pub on_emitter: bool,
    /// `mOnAlpha` (`d_a_obj_swflat.h:53`).
    pub on_alpha: u8,
    /// `mpOffEmitter` (`d_a_obj_swflat.h:40`) as the latch the unlit emitter exists.
    pub off_emitter: bool,
    /// `mOffAlpha` (`d_a_obj_swflat.h:52`).
    pub off_alpha: u8,
    /// `mCyl`'s live radius (`d_a_obj_swflat.cpp:240`, `:243`), which the registration at `:228`
    /// hands to the pass and the next tick's `mCyl.ChkCoHit()` (`:230`) reads back - so the press
    /// test below uses the radius the PREVIOUS tick left behind, exactly as retail does.
    pub radius: f32,
    /// The pending half of `fopAcM_orderPotentialEvent(this, 1, 0, 0)` (`d_a_obj_swflat.cpp:320`).
    pub order_pending: bool,
    /// `eventInfo.mCommand == dEvtCmd_INDEMO_e`, which is what
    /// `eventInfo.checkCommandDemoAccrpt()` answers (`tww/include/f_op/f_op_actor.h:177`).
    pub in_demo: bool,
    /// One-shot guard so `Create`'s switch reads (`d_a_obj_swflat.cpp:125-153`) run once, on the
    /// first tick, where `Switches` is available - the `hlift::Hlift::entered` shape.
    pub entered: bool,
}

impl Swflat {
    /// The parameter half of `daObjSwflat::Act_c::Create` (`d_a_obj_swflat.cpp:107-160`): the three
    /// `PrmAbstract` reads (`:114-116`) and the zero-initialisers the reference writes before the
    /// switch test (`mFinished` `:120`, `mEventState`/`mEventTimer`/`mFinishTimer` `:154-156`).
    ///
    /// The emitter and `mActive` state is the switch-dependent half and lives in
    /// [`Swflat::create_init`], and `mFrame` / `mActiveFrame` / `mParticlePos` (`:117-119`) are the
    /// break animation and the particle anchor - see the struct docs.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            // `mType = prmGetType();` (d_a_obj_swflat.cpp:114) - and `create_init` is where the
            // `mType == 2` conversion at `:125-133` re-reads it.
            m_type: ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8,
            kind: ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8,
            switch: ((parameters >> SWITCH_SHIFT) & SWITCH_MASK) as u8,
            switch2: ((parameters >> SWITCH2_SHIFT) & SWITCH2_MASK) as u8,
            // `mActive` and `mPrevActive` take their value from the switch test (`:135`, `:148`), so
            // both start where `fopAcM_ct`'s zero-fill leaves them and `create_init` overwrites them.
            active: 0,
            prev_active: 0,
            finished: 0,
            finish_timer: 0,
            event_state: 0,
            event_timer: 0,
            on_emitter: false,
            on_alpha: 0,
            off_emitter: false,
            off_alpha: 0,
            // `mCyl.Set(l_cyl_src)` (d_a_obj_swflat.cpp:112) installs the `/* Radius */ 10.0f` of
            // `:60`, and nothing moves until the first press test.
            radius: CYL_RADIUS,
            order_pending: false,
            in_demo: false,
            entered: false,
        }
    }

    /// `fopAcM_setCullSizeSphere(this, 0.0f, 0.0f, 0.0f, 90.0f)` (`d_a_obj_swflat.cpp:110`) - the
    /// sphere as the box the draw layer wants.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `daObjSwflat::Act_c::Create` (`d_a_obj_swflat.cpp:121-156`), first tick only: the
    /// `switch2` conversion, the initial state read from the switch, and the emitter that state
    /// picks.
    ///
    /// `fopAcM_isSwitch(this, sw)` is `dComIfGs_isSwitch(sw, fopAcM_GetHomeRoomNo(pActor))`
    /// (`tww/include/f_op/f_op_actor_mng.h:374-376`), whose save layer
    /// (`tww/src/d/d_save.cpp:1586-1603`) answers FALSE for `sw == 255` and routes only 0..0x80 to
    /// the bits `super::flags::Switches` models. So an unset `PRM_SW_W` of 0xFF reads false here and
    /// its `fopAcM_onSwitch` writes are ignored, which is the same agreement `hlift` documents.
    ///
    /// The `l_HIO.mResetSwitches == 1` arm (`d_a_obj_swflat.cpp:121-124`) is dead: `mResetSwitches`
    /// is `0` in the constructor (`:71`) and nothing else writes it.
    fn create_init(&mut self, switches: &Switches) {
        // ```
        // u8 sw = mSwitch;
        // if (mType == 2) {
        //     if (fopAcM_isSwitch(this, mSwitch2)) { mType = 0; sw = mSwitch2; }
        //     else { mType = 1; }
        // }
        // ```
        // (:125-133) - a gated plate reads as momentary until its gate switch goes up, and as
        // once-only after, with the gate switch itself taking over as the switch it writes.
        let mut sw = self.switch;
        if self.m_type == TYPE_GATED as u8 {
            if switches.test(self.switch2) {
                self.m_type = TYPE_ONCE as u8;
                sw = self.switch2;
            } else {
                self.m_type = TYPE_MOMENTARY;
            }
        }
        if switches.test(sw) {
            // `mActive = mPrevActive = 1;` (:135) and the two emitter assignments of the ON branch.
            self.active = 1;
            self.prev_active = 1;
            if self.m_type != 0 {
                // `mpOnEmitter = dComIfGp_particle_set(dPa_name::ID_AK_SN_SIRENBUTTONON, ...)`
                // with `mOnAlpha = 250` (:137-138) - a lit plate that can still be re-lit.
                self.on_emitter = true;
                self.on_alpha = ALPHA_FULL;
            } else {
                // ```
                // mpOnEmitter = NULL; mOnAlpha = 0; mFinished = 1;
                // mBrk->setFrame(mBrk->getEndFrame());
                // ```
                // (:141-144) - an already-triggered once-plate comes up finished, with the break
                // animation parked on its last frame for the renderer.
                self.on_emitter = false;
                self.on_alpha = 0;
                self.finished = 1;
            }
            self.off_emitter = false;
            self.off_alpha = 0;
        } else {
            // ```
            // mActive = mPrevActive = 0;
            // mpOffEmitter = dComIfGp_particle_set(dPa_name::ID_AK_SN_SIRENBUTTONOFF, ...);
            // mOffAlpha = 250; mpOnEmitter = NULL; mOnAlpha = 0;
            // ```
            // (:147-153)
            self.active = 0;
            self.prev_active = 0;
            self.off_emitter = true;
            self.off_alpha = ALPHA_FULL;
            self.on_emitter = false;
            self.on_alpha = 0;
        }
        // `mEventState = 0; mEventTimer = 0; mFinishTimer = 0;` (:154-156), re-written here so the
        // first tick starts from `Create` even though `spawn` already zeroed them.
        self.event_state = 0;
        self.event_timer = 0;
        self.finish_timer = 0;
        self.entered = true;
    }

    /// The pass's `mCyl.ChkCoHit()` (`d_a_obj_swflat.cpp:230`) and `mCyl.GetCoHitAc()` (`:231`) as
    /// the last survey candidate that overlaps the plate's cylinder - which is the half of retail's
    /// behaviour the port can reproduce, and the reason the candidate order below is stated rather
    /// than derived: `cCcS::ChkCo` runs its `i < j` loop over the CO list in registration order
    /// (`tww/src/SSystem/SComponent/c_cc_s.cpp:180-194`) and `SetCoGObjInf` overwrites the recorded
    /// partner every accepted pair (`tww/src/d/d_cc_s.cpp:169`, `:173`), so the winner is simply
    /// whoever was registered last. This sim registers the player before the stage props, so the
    /// survey walks the player first and the barrels after.
    ///
    /// The cylinder pair test is `cM3d_Cross_CylCyl` (`tww/src/SSystem/SComponent/c_m3d.cpp:1568-
    /// 1583`) - the same reduction `hcbh.rs` writes for the pillar, with the plate's own base at
    /// `current.pos` (`d_a_obj_swflat.cpp:223`) and its centre `{0.0f, 0.0f, 0.0f}` (`:59`).
    fn press(
        &self,
        pos: Vec3,
        players: &Query<&Player, Without<Swflat>>,
        props: &Query<(&Actor, &Barrel), Without<Swflat>>,
    ) -> Option<Presser> {
        let mut presser: Option<Presser> = None;
        // The player: `Radius 30.0f` / `Height 125.0f` (`d_a_player_main_data.inc:68-69`), standing
        // at `current.pos`, which is this sim's `Player::pos`.
        if let Some(player) = players.iter().next()
            && cyl_cyl_cross(
                pos,
                self.radius,
                CYL_HEIGHT,
                player.pos,
                PLAYER_RADIUS,
                PLAYER_HEIGHT,
            )
        {
            presser = Some(Presser::Player);
        }
        // The barrel: a broken one has asked for its own delete (`d_a_obj_barrel.cpp:720-722`) and
        // never reaches the registration at `:712-713` again. While it is carried, the
        // `mCyl.OffCoSPrmBit(cCcD_CoSPrm_Set_e)` of `mode_carry_init` (`:202`) takes it out of the CO
        // list - the other six modes either turn the bit on (`:166`, `:230`, `:289`, `:318`) or leave
        // it as the mode before them set it, which is why `Carry` is the only excluded one.
        for (actor, barrel) in props.iter() {
            if barrel.broken || barrel.mode == super::barrel::Mode::Carry {
                continue;
            }
            if cyl_cyl_cross(
                pos,
                self.radius,
                CYL_HEIGHT,
                actor.pos,
                BARREL_RADIUS,
                BARREL_HEIGHT,
            ) {
                presser = Some(Presser::Prop);
            }
        }
        presser
    }

    /// `daObjSwflat::Act_c::Execute` (`d_a_obj_swflat.cpp:221-364`) minus the matrices, the break
    /// animation, the emitters' alpha writes and the collision registration - see the struct docs.
    #[allow(clippy::too_many_arguments)]
    fn execute(
        &mut self,
        actor: &Actor,
        switches: &mut Switches,
        order_flags: &mut OrderFlags,
        events: &mut SimEvents,
        players: &Query<&Player, Without<Swflat>>,
        props: &Query<(&Actor, &Barrel), Without<Swflat>>,
    ) {
        // The potential event this actor ordered on an earlier tick is converted by
        // `dComIfGp_event_check` before the actor pass runs (`super::event` documents that latency),
        // so `in_demo` is raised here rather than inside the event switch below.
        if self.order_pending {
            self.order_pending = false;
            self.in_demo = true;
        }

        // `mCyl.SetC(current.pos)` (:223) and `dComIfG_Ccsp()->Set(&mCyl)` (:228), then the recorded
        // partner:
        // ```
        // bool hit = false;
        // if (mCyl.ChkCoHit()) {
        //     fopAc_ac_c* actor = mCyl.GetCoHitAc();
        //     if (actor != NULL && fopAcM_CheckStatus(actor, fopAcStts_FREEZE_e)) {
        //         hit = true;
        //         if (fopAcM_GetProfName(actor) == fpcNm_PLAYER_e) playerHit = true;
        //     }
        // }
        // ```
        // (:229-236). The FREEZE gate is folded into the survey's candidate set, which is exactly the
        // set of pressers whose profile carries `fopAcStts_FREEZE_e` - see the struct docs.
        let presser = self.press(actor.pos, players, props);
        let mut hit = false;
        let mut player_hit = false;
        if let Some(presser) = presser {
            hit = true;
            player_hit = presser == Presser::Player;
        }

        // ```
        // if (mEventTimer == 0) {
        //     if (hit) { mActive = 1; mCyl.SetR(15.0f); }
        //     else if (mType == 1) { mActive = 0; mCyl.SetR(10.0f); }
        // }
        // ```
        // (:237-245) - the press gate the lockout freezes, and the widening that keeps a plate held
        // while its presser is still on it.
        if self.event_timer == 0 {
            if hit {
                self.active = 1;
                self.radius = PRESS_RADIUS;
            } else if self.m_type == TYPE_MOMENTARY {
                self.active = 0;
                self.radius = CYL_RADIUS;
            }
        }

        // ```
        // if (prmGetType() == 2 && fopAcM_isSwitch(this, mSwitch2)) {
        //     mType = 0;
        //     if (mFinished == 0) { mFinishTimer = l_HIO.mFinishTime; mFinished = 1; }
        // }
        // ```
        // (:246-252) - the raw parameter, not `mType`, and `l_HIO.mFinishTime` is the constructor's
        // 30 (`:75`). A gated plate whose gate switch went up mid-game becomes a once-plate that is
        // already on its way out.
        if self.kind as u32 == TYPE_GATED && switches.test(self.switch2) {
            self.m_type = TYPE_ONCE as u8;
            if self.finished == 0 {
                self.finish_timer = FINISH_TIME;
                self.finished = 1;
            }
        }

        // `bool activated = false;` (:253) and the two halves of the `mActive` branch.
        let mut activated = false;
        if self.active == 1 {
            // `if (mPrevActive == 0) mDoAud_seStart(JA_SE_OBJ_FLAT_SW_ON, &current.pos);` (:255)
            if self.prev_active == 0 {
                events.list.push(SimEvent::SeAt(SE_ON, actor.pos));
            }
            // `if (mFinished == 0) mDoAud_seStart(JA_SE_OBJ_FLAT_SW_LIGHT, &current.pos);` (:256) -
            // re-started every frame the plate is held and not yet finishing.
            if self.finished == 0 {
                events.list.push(SimEvent::SeAt(SE_LIGHT, actor.pos));
            }
            // `if (!playerHit) fopAcM_onSwitch(this, mSwitch);` (:261) - the press by anything other
            // than the player sets the switch here, directly, and never reaches the event machine,
            // because `activated && playerHit` is what arms it at `:310`.
            if !player_hit {
                switches.set(self.switch);
            }
            if self.finished == 0 {
                if !self.on_emitter {
                    // ```
                    // mpOnEmitter = dComIfGp_particle_set(dPa_name::ID_AK_SN_SIRENBUTTONON, ...);
                    // mOnAlpha = 250; activated = true;
                    // if (prmGetType() == 0) { mFinishTimer = l_HIO.mFinishTime; mFinished = 1; }
                    // ```
                    // (:264-270) - the frame the lit emitter appears is the frame the event machine
                    // is armed, and a `TYPE_ONCE` plate starts finishing on it too.
                    self.on_emitter = true;
                    self.on_alpha = ALPHA_FULL;
                    activated = true;
                    if self.kind as u32 == TYPE_ONCE {
                        self.finish_timer = FINISH_TIME;
                        self.finished = 1;
                    }
                } else if self.on_alpha < ALPHA_FULL {
                    // `mOnAlpha += 25; mpOnEmitter->setGlobalAlpha(mOnAlpha);` with the second
                    // `fopAcM_onSwitch(this, mSwitch)` (:271-275) - the ramp-back-up case, which
                    // re-asserts the switch the fade-out may have left stale.
                    self.on_alpha += ALPHA_STEP;
                    switches.set(self.switch);
                }
            }
            // ```
            // if (mpOffEmitter != NULL) {
            //     mOffAlpha -= 25; ...->setGlobalAlpha(mOffAlpha);
            //     if (mOffAlpha == 0) { ...->becomeInvalidEmitter(); mpOffEmitter = NULL; }
            // }
            // ```
            // (:277-284)
            if self.off_emitter {
                self.off_alpha = self.off_alpha.wrapping_sub(ALPHA_STEP);
                if self.off_alpha == 0 {
                    self.off_emitter = false;
                }
            }
        } else {
            // `fopAcM_offSwitch(this, mSwitch);` (:290) - the unpressed plate holds its switch down
            // every frame, which is what makes a momentary plate momentary.
            switches.clear(self.switch);
            if !self.off_emitter {
                // `mpOffEmitter = dComIfGp_particle_set(dPa_name::ID_AK_SN_SIRENBUTTONOFF, ...)`
                // with `mOffAlpha = 250` (:291-293)
                self.off_emitter = true;
                self.off_alpha = ALPHA_FULL;
            } else if self.off_alpha < ALPHA_FULL {
                // `mOffAlpha += 25; ...->setGlobalAlpha(mOffAlpha);` (:294-297)
                self.off_alpha += ALPHA_STEP;
            }
            // The lit emitter's fade-out, `mOnAlpha -= 25` with the NULL-out at zero (:298-305) -
            // clearing the latch is what lets `activated` fire again on the next press.
            if self.on_emitter {
                self.on_alpha = self.on_alpha.wrapping_sub(ALPHA_STEP);
                if self.on_alpha == 0 {
                    self.on_emitter = false;
                }
            }
        }

        // ```
        // switch (mEventState) {
        // case 0: mEventTimer = 0; if (activated && playerHit) { mEventState = 1; mEventTimer = 30; }
        //         break;
        // ```
        // (:308-314) - a once-plate's first press by the player starts the lockout as well as the
        // state, and `mEventTimer = 30` is what freezes the press gate at `:237` from here.
        match self.event_state {
            0 => {
                self.event_timer = 0;
                if activated && player_hit {
                    self.event_state = 1;
                    self.event_timer = EVENT_LOCKOUT;
                }
            }
            // ```
            // case 1: if (eventInfo.checkCommandDemoAccrpt()) {
            //             mEventState = 2;
            //             if (l_HIO.mTune != 0) mEventTimer = l_HIO.mEventTime;
            //         } else {
            //             fopAcM_orderPotentialEvent(this, 1, 0, 0);
            //             eventInfo.onCondition(2);
            //         }
            //         break;
            // ```
            // `mEventState` moves to 2 at `d_a_obj_swflat.cpp:315-323`. The `mTune` that would re-aim
            // the countdown is 0 (`d_a_obj_swflat.cpp:71`), so `mEventTimer` keeps the 30 `case 0`
            // armed, which is the same number as `mEventTime`'s (`d_a_obj_swflat.cpp:74`).
            1 => {
                if self.in_demo {
                    self.event_state = 2;
                } else {
                    self.order_potential(order_flags);
                }
            }
            // ```
            // case 2: mEventTimer--;
            //         if (mEventTimer <= 1) {
            //             dComIfGp_event_reset(); mEventState = 0; fopAcM_onSwitch(this, mSwitch);
            //         }
            //         break;
            // ```
            // (:324-331)
            2 => {
                self.event_timer -= 1;
                if self.event_timer <= 1 {
                    self.end_demo();
                    self.event_state = 0;
                    switches.set(self.switch);
                }
            }
            // `mEventState` is a `u8` written only by `:311`, `:317` and `:328`, so 0, 1 and 2 are
            // the whole domain and this arm cannot run.
            _ => {}
        }

        // ```
        // if (mFinished != 0) {
        //     mActive = 1;
        //     if (mFinishTimer != 0) { mFinishTimer--; }
        //     else { ...fade both emitters out, bump the break animation to its last frames... }
        // }
        // ```
        // (:333-359). `mActive = 1` is re-asserted every frame a finished plate is alive (`:334`), so
        // a once-plate can never go back to the unpressed branch above; and the fade-out tail drops
        // both emitters by the same 25-step (`:338-353`), which is the `mFrame`/`mBrk` tail at
        // `:354-357` this module drops as renderer work.
        if self.finished != 0 {
            self.active = 1;
            if self.finish_timer != 0 {
                self.finish_timer -= 1;
            } else {
                if self.on_emitter {
                    self.on_alpha = self.on_alpha.wrapping_sub(ALPHA_STEP);
                    if self.on_alpha == 0 {
                        self.on_emitter = false;
                    }
                }
                if self.off_emitter {
                    self.off_alpha = self.off_alpha.wrapping_sub(ALPHA_STEP);
                    if self.off_alpha == 0 {
                        self.off_emitter = false;
                    }
                }
            }
        }

        // `mPrevActive = mActive;` (:362), after everything that read the old value.
        self.prev_active = self.active;
    }

    /// `fopAcM_orderPotentialEvent(this, 1, 0, 0)` with `eventInfo.onCondition(2)`
    /// (`d_a_obj_swflat.cpp:320-321`).
    ///
    /// The order is `dComIfGp_event_order(dEvtType_POTENTIAL_e, priority, flag, hind, i_this,
    /// dComIfGp_getPlayer(0))` (`tww/src/f_op/f_op_actor_mng.cpp:745-749`), with the priority bumped
    /// `0 -> 0xFF` by the wrapper (`:746-747`), the flag `1` = `dEvtFlag_NOPARTNER_e`
    /// (`tww/include/d/d_event.h:32`) and the second actor the player
    /// (`tww/include/d/d_com_inf_game.h:3555-3558`'s defaults for the two trailing arguments).
    ///
    /// `dEvt_control_c::potentialCheck` then accepts it with no event index and no staff at all:
    /// past the `beforeFlagProc` TALK test it is `actor1->eventInfo.setCommand(dEvtCmd_INDEMO_e)` and
    /// `mMode = dEvtMode_DEMO_e` (`tww/src/d/d_event.cpp:352-356`). `afterFlagProc`'s NOPARTNER arm
    /// (`tww/src/d/d_event.cpp:116-125`) points the demo's second partner at nothing, so
    /// `moveApproval` never forces the player for it (`tww/src/d/d_event.cpp:628-673`). That is why
    /// this module latches the acceptance per-actor
    /// instead of ordering through `super::event`: a potential event has no name to look up, so there
    /// is no slot for `ActiveEvent::stage_of` to answer about - and unlike the `kui` / `marlon`
    /// potential orders, which drop theirs and simply never pass their own gate, this one's gate
    /// feeds a countdown that has to run.
    ///
    /// What is NOT modelled is the global half: `mMode = dEvtMode_DEMO_e` (`d_event.cpp:356`) parks
    /// the whole event machine for the lockout's ~30 frames, so any other actor's order conversion is
    /// refused meanwhile (`d_event.cpp:476-534` reads `runCheck()` first, and `runCheck` is
    /// `mMode != dEvtMode_NONE_e` at `tww/include/d/d_event.h:133-139`). The sim has no event mode to
    /// set, so the plate's freeze is local to the plate.
    fn order_potential(&mut self, order_flags: &mut OrderFlags) {
        self.order_pending = true;
        // `eventInfo.onCondition(2)` (:321) is `mCondition |= cond`
        // (`tww/include/f_op/f_op_actor.h:171`), which `beforeProc` clears every tick
        // (`tww/src/f_op/f_op_actor.cpp:176`) - so the flag is raised once per frame it is asked for,
        // exactly as `goal_flag` and `kamome2` model the same call.
        order_flags.set(EVT_CND_UNK2);
    }

    /// `dComIfGp_event_reset()` (`d_a_obj_swflat.cpp:327`) for a plate whose only event is its own
    /// pending potential demo.
    ///
    /// The call is `dEvDtBase_c::reset()` = `onEventFlag(8)` (`tww/include/d/d_event.h:131`), which
    /// the next `dEvt_control_c::check` turns into `endProc` (`tww/src/d/d_event.cpp:546-554`) and
    /// `endProc`'s `demoEnd` clears `INDEMO` off the ordered actor and leaves the NULL second partner
    /// alone (`:401-424`, `:329-343`), after which `mMode` is `dEvtMode_NONE_e` again. `super::event`
    /// has no flag-8 queue and no global mode, so the one part of that chain this plate reads back -
    /// `checkCommandDemoAccrpt()` - is taken down here, one tick earlier than retail manages it, and
    /// `case 0` (`:308-314`) never reads it in between.
    fn end_demo(&mut self) {
        self.in_demo = false;
    }
}

/// `cM3d_Cross_CylCyl` (`tww/src/SSystem/SComponent/c_m3d.cpp:1568-1583`) over two cylinders sitting
/// at their own bases: the horizontal span is the two radii, then each base has to fall inside the
/// other's `y .. y + h`. `super::hcbh`'s private copy is the same reduction for the pillar; this one
/// is `d_a_obj_swflat.cpp`'s because the plate's press test is the only thing it answers for.
fn cyl_cyl_cross(base: Vec3, radius: f32, height: f32, at: Vec3, other: f32, other_h: f32) -> bool {
    let dx = base.x - at.x;
    let dz = base.z - at.z;
    let span = radius + other;
    if dx * dx + dz * dz > span * span {
        return false;
    }
    base.y + height >= at.y && base.y <= at.y + other_h
}

/// The player's body cylinder radius, `/* Radius */ 30.0f` of `l_cyl_src`
/// (`tww/src/d/actor/d_a_player_main_data.inc:68`).
const PLAYER_RADIUS: f32 = 30.0; // <- d_a_player_main_data.inc:68
/// `/* Height */ 125.0f` (`tww/src/d/actor/d_a_player_main_data.inc:69`), the standing height
/// `d_a_player_main.cpp:9787` recomputes from the neck and toe joints.
const PLAYER_HEIGHT: f32 = 125.0; // <- d_a_player_main_data.inc:69
/// `/* Radius */ l_l_radius` with `const float daObjBarrel::Act_c::l_l_radius = 50.0f`
/// (`tww/src/d/actor/d_a_obj_barrel.cpp:53`, `:16`).
const BARREL_RADIUS: f32 = 50.0; // <- d_a_obj_barrel.cpp:53
/// `/* Height */ 100.0f` (`tww/src/d/actor/d_a_obj_barrel.cpp:55`).
const BARREL_HEIGHT: f32 = 100.0; // <- d_a_obj_barrel.cpp:55

pub fn is_swflat(name: &str) -> bool {
    SWFLAT_ACTORS.contains(&name)
}

/// `g_profile_Obj_Swflat`'s `fopAcStts_CULL_e | fopAcStts_UNK40000_e`
/// (`tww/src/d/actor/d_a_obj_swflat.cpp:431`) is the plate's own profile status, and it has NO
/// `fopAcStts_FREEZE_e` in it - which is the one self-press a floor switch must not have:
/// `fopAcM_CheckStatus(actor, fopAcStts_FREEZE_e)` at `:232` would otherwise let the plate press
/// itself. `self::press` therefore never considers the plate, and the FREEZE gate is a property of
/// the two candidate sources, not a lookup - this sim keeps no per-actor profile status to look up.
pub fn control(
    mut switches: ResMut<Switches>,
    mut order_flags: ResMut<OrderFlags>,
    mut events: ResMut<SimEvents>,
    players: Query<&Player, Without<Swflat>>,
    props: Query<(&Actor, &Barrel), Without<Swflat>>,
    mut plates: Query<(&Actor, &mut Swflat), With<Swflat>>,
) {
    for (actor, mut plate) in &mut plates {
        if !plate.entered {
            plate.create_init(&switches);
        }
        plate.execute(
            actor,
            &mut switches,
            &mut order_flags,
            &mut events,
            &players,
            &props,
        );
    }
}
