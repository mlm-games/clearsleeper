//! `kui_class` (`d_a_kui.h:5-33`) - the Outset Island grapple posts, the rope-tag props, the
//! ceiling bell and the pull-down rope device behind the single stage string `Kui`. `Nonmatching`
//! count: 0, so every token below is transcribed. The file header calls it
//! "Object - Hookable posts for the Grappling Hook" (`d_a_kui.cpp:3`) and the code confirms it:
//! this is NOT a boss and NOT an enemy - `kui_class` registers no collider of any kind.
//!
//! Nothing else in `src/sim/` claims `Kui`: `OBJNAME("Kui", fpcNm_KUI_e, 255, 0)`
//! (`d_stage.cpp:645`) is the only row, and it agrees with the actor's own archive name
//! (`dComIfG_resLoad(&i_this->mPhs, "Kui")`, `d_a_kui.cpp:474`).

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::door::StayRoom;
use super::flags::Switches;

/// `OBJNAME("Kui", fpcNm_KUI_e, 255, 0)` (`d_stage.cpp:645`). `fpcNm_KUI_e` is also the name
/// `search_dragontail` / `search_btd` exclude while looking for `fpcNm_DR2_e` / `fpcNm_BTD_e`
/// (`d_a_kui.cpp:23`, `:36`) - that search never matches a Kui.
pub const KUI_ACTORS: [&str; 1] = ["Kui"];

/// `i_this->type = fopAcM_GetParam(a_this) & 0xF;` (`d_a_kui.cpp:482`) - the post / prop variant.
const TYPE_MASK: u32 = 0xF; // <- d_a_kui.cpp:482
/// `if (i_this->type == 3)` (`:490`, `:410`, `:533`) - the CEILING BELL, whose `CreateHeap` arm
/// loads `BDL_HKANE1` + `BDL_HKANE2` (`:412`, `:421`) and whose `Draw` arm is the `type == 3` test
/// at `:95`.
const TYPE_BELL: u8 = 3; // <- d_a_kui.cpp:490
/// `if (i_this->type == 2 || i_this->type == 4)` (`:345`, `:361`, `:438`) - the PULL-DOWN ROPE
/// DEVICE, whose `CreateHeap` arm adds `BDL_MROPESW` (`:440`). Type 2 hands over to `demo_camera`
/// (`:377-378`); type 4 sets its own switch instead (`:380-382`).
const TYPE_ROPE_A: u8 = 2; // <- d_a_kui.cpp:345
const TYPE_ROPE_B: u8 = 4; // <- d_a_kui.cpp:361
/// `i_this->field_0x2A2 = fopAcM_GetParam(a_this) & 0xF0;` (`:483`) - the DRAGON-TAIL ROPE flag,
/// tested only at `:215`. The block it gates is OMITTED; see [`Kui::execute`].
const TAIL_MASK: u32 = 0xF0; // <- d_a_kui.cpp:483
/// `fopAcM_setCullSizeBox` is not used; `Create` writes the box with the size setters, for
/// `i_this->type >= 2` (`:533-535`):
/// ```c
/// fopAcM_SetMin(i_this, -200.0f, -1000.0f, -200.0f);
/// fopAcM_SetMax(i_this,  200.0f,  2000.0f,  200.0f);
/// ```
/// matching the profile's `fopAc_CULLBOX_CUSTOM_e` (`:566`). The `else` is the flat 400-cube
/// (`:537-538`).
const CULL_MIN_TALL: [f32; 3] = [-200.0, -1000.0, -200.0]; // <- d_a_kui.cpp:534
const CULL_MAX_TALL: [f32; 3] = [200.0, 2000.0, 200.0]; // <- d_a_kui.cpp:535
const CULL_MIN_FLAT: [f32; 3] = [-200.0, -200.0, -200.0]; // <- d_a_kui.cpp:537
const CULL_MAX_FLAT: [f32; 3] = [200.0, 200.0, 200.0]; // <- d_a_kui.cpp:538
/// ```c
/// if (actor->health == 3 && i_this->field_0x308 == 0) { i_this->field_0x308 = 1000; }
/// ```
/// (`:365-366`) - `field_0x308` is the release countdown.
const RELEASE_START: i16 = 1000; // <- d_a_kui.cpp:366
/// `s16 finished = REG8_S(3) + 970;` (`:375`) - RULE 2b folds `REG8_S(3)` to 0.
const RELEASE_TRIGGER: i16 = 970; // <- d_a_kui.cpp:375
/// `actor->current.pos.y = actor->home.pos.y - 70.0f;` (`:369`) - the retracted rope height.
const ROPE_DROP: f32 = -70.0; // <- d_a_kui.cpp:369
/// ```c
/// cLib_addCalcAngleS2(&actor->current.angle.x, target_x_angle, 4, REG0_S(1) + 0x200);
/// ```
/// (`:279`) - `REG0_S(1)` folds to 0. The bell's pitch chase, running in the type-3 arm every tick.
const BELL_CHASE_SCALE: i16 = 4; // <- d_a_kui.cpp:279
const BELL_CHASE_MAX: i16 = 0x200; // <- d_a_kui.cpp:279
/// ```c
/// cLib_addCalcAngleS2(&i_this->field_0x2E2, i_this->field_0x2E4, 4, i_this->field_0x2E6);
/// cLib_addCalcAngleS2(&i_this->field_0x2E4, 0, 1, REG17_S(6) + 0x80);
/// cLib_addCalcAngleS2(&i_this->field_0x2E6, REG17_S(7) + 0x100, 1, REG17_S(8) + 0x40);
/// ```
/// (`:309-311`) - every `REG17_*` register folds to 0 under RULE 2b, leaving `0x80`, `0x100`, `0x40`.
/// These three fields feed `cMtx_XrotM` (`:318`) and `MtxScale` (`:321`), so the ring motion is real
/// matrix state rather than an unread register.
const RING_CHASE_SCALE: i16 = 4; // <- d_a_kui.cpp:309
const RING_DECAY_MAX: i16 = 0x80; // <- d_a_kui.cpp:310
const RING_TARGET: i16 = 0x100; // <- d_a_kui.cpp:311
const RING_STEP_MAX: i16 = 0x40; // <- d_a_kui.cpp:311
/// `demo_camera` (`:130-200`) - the `field_0x30C` rope-drop profile, `uVar3 == (s32)field_0x2EA`:
/// ```c
/// if (uVar3 < 20)      { field_0x30C = cM_ssin((uVar3 & 0x1F) * 0x800) * 5.0f; }
/// else if (uVar3 <= 27){ cLib_addCalc2(&field_0x30C, -70.0f, 1.0f, 10.0f); }
/// else if (uVar3 < 42) { field_0x30C = cM_ssin(uVar3 * 0x3A00) * 5.0f + (-70.0f); }
/// ```
/// (`:153-165`).
const DROP_IDLE_END: i16 = 20; // <- d_a_kui.cpp:153
const DROP_RING: i16 = 27; // <- d_a_kui.cpp:156
const DROP_SETTLE_END: i16 = 42; // <- d_a_kui.cpp:162
const DROP_SWAY_STEP: i16 = 0x800; // <- d_a_kui.cpp:154
const DROP_SWAY_AMP: f32 = 5.0; // <- d_a_kui.cpp:155
const DROP_REST: f32 = -70.0; // <- d_a_kui.cpp:161
const DROP_SCALE: f32 = 1.0; // <- d_a_kui.cpp:161
const DROP_MAX_STEP: f32 = 10.0; // <- d_a_kui.cpp:161
const DROP_SETTLE_STEP: i16 = 0x3A00; // <- d_a_kui.cpp:163
/// `if (i_this->field_0x2EA == 70)` (`:180`) - the frame the demo hands control back.
const DEMO_END: i16 = 70; // <- d_a_kui.cpp:180
/// `i_this->field_0x2E8 = i_this->field_0x2E8 + 1;` (`:144`) and the `case 2` fallthrough at `:148`.
const DEMO_ROPE: i8 = 2; // <- d_a_kui.cpp:144

/// `kui_class` (`d_a_kui.h:5-33`).
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. The class holds only `mPhs`, two
/// `J3DModel*`, a `dBgW*`, one `Mtx` and thirteen value fields (`d_a_kui.h:6-32`). There is no
/// `dCcD_Stts`, `dCcD_Cyl`, `dCcD_Sph` or `dCcD_SrcCyl` member, and `dComIfG_Ccsp()->Set(...)`
/// appears nowhere in the translation unit, so retail registers no attack and no target collider.
/// The profile `Group` is `fopAc_ACTOR_e` (`:565`) and its `Status` is
/// `fopAcStts_CULL_e | fopAcStts_UNK4000_e | fopAcStts_UNK40000_e` (`:564`). Neither
/// `combat::HitPoints` nor `enemy::Hurt` is attached - `combat::control` would despawn a grapple
/// post retail cannot damage.
///
/// The `actor->health` reads at `:232`, `:304` and `:365` need no inbound path. `health` is
/// `fopAc_ac_c::health`, an `s8` zeroed by `fpcBs_Create`'s `sBs_ClearArea`
/// (`f_pc_base.cpp:97-98`). The only writers of the literal `3` in the whole game are
/// `d_a_bgn2.cpp:882` and `d_a_bgn3.cpp:1318`, each on its OWN `&i_this->actor`; `c_damagereaction.cpp`
/// writes `-0x80` (`:168`, `:370`) or `0` (`:1033`, `:1044`, `:1051`) and can only reach an actor
/// that registered a `dCcD` collider, which a `kui_class` never does. So `actor->health == 3` is
/// FALSE for every Kui - a PROVEN fold of the same kind as RULE 2b's, not a guess. It is why the
/// bell's swing block (`:233-274`) and the rope device's `RELEASE_START` seed (`:366`) are recorded
/// as provably dead rather than modelled with invented inputs.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Kui {
    /// `type` (`d_a_kui.h:9`) - `param & 0xF` (`:482`); it picks the `CreateHeap` arm (`:410`,
    /// `:428`, `:438`) and the `Execute` arm (`:231`, `:342`, `:345`, `:361`).
    pub kind: u8,
    /// `mSwitchNo` (`d_a_kui.h:11`) - `param >> 0x18` (`:485`) with `0xFF -> 0` (`:487-489`). Read at
    /// `:189`, `:305`, `:362` and `:381`.
    pub switch_no: u8,
    /// `field_0x2A2` (`d_a_kui.h:12`) - `param & 0xF0` (`:483`), the dragon-tail flag. Stored because
    /// [`Kui::execute`] reports on it; it gates no modelled behaviour.
    pub on_tail: bool,
    /// `field_0x2DC[3]` (`d_a_kui.h:23`) - the bell's shake timers: `[0]` is the master budget and
    /// `[1]` / `[2]` the two axis timers (`:256`, `:258`, `:280-285`, `:291-294`).
    pub shake: [i16; 3],
    /// `field_0x2E2` (`d_a_kui.h:24`, an `s16`) - the ring's swing angle, consumed by
    /// `cMtx_XrotM` (`:318`).
    pub swing: i16,
    /// `field_0x2E4` (`d_a_kui.h:25`) - the ring's current axis angle, chased to 0 at `:310`.
    pub ring_angle: i16,
    /// `field_0x2E6` (`d_a_kui.h:26`) - the ring's axis chase rate, relaxed toward `RING_TARGET`
    /// (`:311`).
    pub ring_step: i16,
    /// `field_0x2E8` (`d_a_kui.h:27`, an `s8`) - the demo mode (`:134`, `:144`, `:181`, `:378`).
    pub demo_mode: i8,
    /// `field_0x2EA` (`d_a_kui.h:29`, an `s16`) - the demo frame counter (`:147`, `:152`, `:180`,
    /// `:198`).
    pub demo_frame: i16,
    /// `field_0x30C` (`d_a_kui.h:32`, an `f32`) - the rope's drop offset, added to `home.pos.y` at
    /// `:150`.
    pub drop: f32,
    /// `field_0x308` (`d_a_kui.h:31`, an `s16`) - the rope device's release countdown (`:365`, `:372-384`).
    pub release: i16,
    /// `home.pos.y` (`fopAc_ac_c::home`). `Execute` reads it at `:150` and `:369`; the sim has no
    /// `home` member on `Actor`, so it is sampled from `current.pos` on the first tick, which
    /// `fopAc_Init` seeds from the same stage placement before any method runs.
    pub home_y: f32,
    /// One-shot guard for the `home` sample.
    pub entered: bool,
    /// RULE 6b: `daKui_Create` returns `cPhs_ERROR_e` when `fopAcM_GetParam(a_this) == -1`
    /// (`:478-480`). That cancels the process during Create (`f_pc_create_req.cpp:106-108`), so the
    /// actor NEVER executes even once: spawn it, then despawn on the FIRST TICK rather than leaking a
    /// bare `Actor`.
    pub refused: bool,
}

impl Kui {
    /// `daKui_Create` (`:467-542`)'s parameter decode, exactly.
    ///
    /// ```c
    /// if (fopAcM_GetParam(a_this) == -1) { return cPhs_ERROR_e; }
    /// i_this->type = fopAcM_GetParam(a_this) & 0xF;
    /// i_this->field_0x2A2 = fopAcM_GetParam(a_this) & 0xF0;
    /// i_this->field_0x2A1 = fopAcM_GetParam(a_this) >> 8;
    /// i_this->mSwitchNo = fopAcM_GetParam(a_this) >> 0x18;
    /// if (i_this->mSwitchNo == 0xFF) { i_this->mSwitchNo = 0; }
    /// if (i_this->type == 3) { i_this->field_0x2A1 = 4; }
    /// ```
    ///
    /// `fopAcM_GetParam` returns the WHOLE `u32` (`f_op_actor_mng.h:127-129`), so `& 0xF`, `>> 8` and
    /// `>> 0x18` are the only masks. `field_0x2A1` (`param >> 8`, `:484`) is DROPPED: it only feeds
    /// the `scale` switch at `:494-522`, whose writes are consumed by `MtxScale` / `setBaseScale`
    /// (`:321`, `:330`, `:339`, `:349`, `:354`) - renderer work with no modelled reader (RULE 3b), so
    /// an unread field would be dead code (RULE 12b). `mSwitchNo` is a `u8`
    /// (`d_a_kui.h:11`), so `param >> 0x18` is narrowed to its low byte.
    pub fn spawn(parameters: u32) -> Self {
        // `i_this->type = fopAcM_GetParam(a_this) & 0xF;` (:482)
        let kind = (parameters & TYPE_MASK) as u8;
        // `i_this->mSwitchNo = fopAcM_GetParam(a_this) >> 0x18;` (:485)
        let raw_switch = (parameters >> 0x18) as u8;
        Self {
            kind,
            // `if (i_this->mSwitchNo == 0xFF) { i_this->mSwitchNo = 0; }` (:487-489)
            switch_no: if raw_switch == 0xFF { 0 } else { raw_switch },
            // `i_this->field_0x2A2 = fopAcM_GetParam(a_this) & 0xF0;` (:483), then `if
            // (i_this->field_0x2A2)` at :215.
            on_tail: parameters & TAIL_MASK != 0,
            // `field_0x2DC`, `field_0x2E2`, `field_0x2E4`, `field_0x2E6`, `field_0x2E8`,
            // `field_0x2EA`, `field_0x308` all take `sBs_ClearArea`'s zero fill
            // (`f_pc_base.cpp:97-98`) - `fopAcM_ct` writes none of them.
            shake: [0; 3],
            swing: 0,
            ring_angle: 0,
            ring_step: 0,
            demo_mode: 0,
            demo_frame: 0,
            // `field_0x30C` likewise.
            drop: 0.0,
            release: 0,
            // `fopAc_Init` seeds `home` from the stage placement; sampled on the first tick.
            home_y: 0.0,
            entered: false,
            // `if (fopAcM_GetParam(a_this) == -1) { return cPhs_ERROR_e; }` (:478-480) - the whole
            // u32 against -1, i.e. 0xFFFFFFFF.
            refused: parameters == u32::MAX,
        }
    }

    /// The custom cull box. `type >= 2` gets the tall box (`:534-535`), everything else the flat one
    /// (`:537-538`), matching the profile's `fopAc_CULLBOX_CUSTOM_e` (`:566`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        if self.kind >= TYPE_BELL {
            (CULL_MIN_TALL, CULL_MAX_TALL)
        } else {
            (CULL_MIN_FLAT, CULL_MAX_FLAT)
        }
    }

    /// `daKui_Execute` (`:203-391`), reduced to the three value updates that leave the sim.
    ///
    /// The type-3 arm's `target_x_angle` computation (`:233-274`) is DROPPED with the health fold -
    /// it needs `player->getLeftHandPos()`, i.e. `daPy_lk_c::mLeftHandPos` (`d_a_player.h:512`),
    /// which the player writes from its left-hand joint's world matrix every tick
    /// (`mDoMtx_multVecZero(mpCLModel->getAnmMtx(CL_JNT_CL_LHANDA_e), &mLeftHandPos)`,
    /// `d_a_player_main.cpp:11594`). This port has no player model and keeps no hand-joint
    /// position, so the arm has no first point - and it is behind `actor->health == 3` anyway,
    /// which the port's `kui` fold also drops.
    /// `cLib_addCalcAngleS2(&actor->current.angle.x, target_x_angle, 4, REG0_S(1) + 0x200)` (`:279`)
    /// is the one surviving line of that arm, and with `target_x_angle = 0` from `:276` it relaxes
    /// the bell's pitch toward level.
    ///
    /// DROPPED from the type-3 arm: `cLib_addCalcAngleS2(&field_0x2E2, ...)` and its two siblings
    /// (`:309-311`) are KEPT (they are live matrix state, see the constants above); `setBaseTRMtx`,
    /// `MtxTrans`, `cMtx_YrotM`, `cMtx_XrotM`, `cMtx_ZrotM`, `MtxScale`, `MtxPosition`
    /// (`:313-340`) are J3D matrix work; `i_this->field_0x2D8->Move()` (`:341`, `:358`) is a `dBgW`
    /// moving background, which RULE 3b lists as absent.
    fn execute(&mut self, actor: &mut Actor, switches: &mut Switches) {
        // `if (i_this->type == 3) { ... }` (:231) - the ceiling bell.
        if self.kind == TYPE_BELL {
            // `cLib_addCalcAngleS2(&actor->current.angle.x, target_x_angle, 4, REG0_S(1) + 0x200);`
            // (:279) with `target_x_angle = 0` (`:276, the health == 3 else arm).
            let next = chase_angle_s2(actor.angle[0], 0, BELL_CHASE_SCALE, BELL_CHASE_MAX);
            actor.angle[0] = next;
            // ```
            // if (i_this->field_0x2DC[1] != 0) { i_this->field_0x2DC[1]--; }
            // if (i_this->field_0x2DC[2] != 0) { i_this->field_0x2DC[2]--; }
            // ```
            // (:280-285) - the axis timers.
            if self.shake[1] != 0 {
                self.shake[1] -= 1;
            }
            if self.shake[2] != 0 {
                self.shake[2] -= 1;
            }
            // ```
            // if (i_this->field_0x2DC[0] != 0) {
            //     i_this->field_0x2DC[0]--;
            //     ... x, z, unk_f from cM_ssin / cM_scos, then MtxScale ...
            //     if (field_0x2DC[0] == 0 && actor->health == 3 && REG0_S(3) == 0) {
            //         dComIfGs_onSwitch(i_this->mSwitchNo, fopAcM_GetRoomNo(actor));
            //     }
            // }
            // ```
            // (:291-307) - `field_0x2DC[0]` is only ever SEEDED inside the dropped health-3 block at
            // `:258`, so it stays 0 and this never runs. The counter itself is transcribed so the
            // fold is visible rather than silently absent.
            if self.shake[0] != 0 {
                self.shake[0] -= 1;
            }
            // `cLib_addCalcAngleS2(&i_this->field_0x2E2, i_this->field_0x2E4, 4, i_this->field_0x2E6);`
            // (:309)
            let swing = chase_angle_s2(
                self.swing,
                self.ring_angle,
                RING_CHASE_SCALE,
                self.ring_step,
            );
            self.swing = swing;
            // `cLib_addCalcAngleS2(&i_this->field_0x2E4, 0, 1, REG17_S(6) + 0x80);` (:310)
            let angle = chase_angle_s2(self.ring_angle, 0, 1, RING_DECAY_MAX);
            self.ring_angle = angle;
            // `cLib_addCalcAngleS2(&i_this->field_0x2E6, REG17_S(7) + 0x100, 1, REG17_S(8) + 0x40);`
            // (:311)
            let step = chase_angle_s2(self.ring_step, RING_TARGET, 1, RING_STEP_MAX);
            self.ring_step = step;
        } else if self.kind == TYPE_ROPE_A || self.kind == TYPE_ROPE_B {
            // ```
            // BOOL is_switch = dComIfGs_isSwitch(i_this->mSwitchNo,
            //                                      dComIfGp_roomControl_getStayNo());
            // if (!is_switch) {
            //     if (actor->health == 3 && i_this->field_0x308 == 0) { i_this->field_0x308 = 1000; }
            // } else {
            //     actor->current.pos.y = actor->home.pos.y - 70.0f;
            // }
            // ```
            // (:362-370) - `dComIfGp_roomControl_getStayNo()` is `super::door::StayRoom`, which the
            // sim's `Switches` does not model (it is the in-memory 0..128 array, RULE 11b), so the
            // stay-room distinction is a documented reduction: `Switches::test` answers the same
            // question for every in-range bit. The `health == 3` seed folds FALSE (see the struct
            // note), so only the retracted-height arm is live.
            if switches.test(self.switch_no) {
                // `actor->current.pos.y = actor->home.pos.y - 70.0f;` (:369)
                actor.pos.y = self.home_y + ROPE_DROP;
            }
            // ```
            // if (i_this->field_0x308 != 0) {
            //     i_this->field_0x308--;
            //     s16 finished = REG8_S(3) + 970;
            //     if (i_this->field_0x308 == finished) {
            //         if (i_this->type == 2) { i_this->field_0x2E8 = 1; }
            //         else { fopAcM_seStartCurrent(actor, JA_SE_OBJ_ROPE_SW_ON, 0);
            //                dComIfGs_onSwitch(i_this->mSwitchNo, fopAcM_GetRoomNo(actor));
            //                mDoAud_seStart(JA_SE_READ_RIDDLE_1); }
            //     }
            // }
            // ```
            // (:372-385) - transcribed even though the seed folds away, so the count and the switch
            // set are there if the fold ever changes. RULE 11b: a `mSwitchNo` at or above 0x80 reads
            // false and `set` is a no-op here; the REAL index is passed through.
            if self.release != 0 {
                self.release -= 1;
                if self.release == RELEASE_TRIGGER {
                    if self.kind == TYPE_ROPE_A {
                        // `if (i_this->type == 2) { i_this->field_0x2E8 = 1; }` (:377-378)
                        self.demo_mode = 1;
                    } else {
                        // `dComIfGs_onSwitch(i_this->mSwitchNo, fopAcM_GetRoomNo(actor));` (:381)
                        switches.set(self.switch_no);
                    }
                }
            }
            // `demo_camera(i_this);` (:386)
            self.demo_camera(actor, switches);
        }
        // `} else { MtxTrans(...); cMtx_YrotM(...); setBaseTRMtx(...); field_0x2D8->Move(); }`
        // (:342-359) - matrix and `dBgW` work only.
        //
        // `actor->eyePos = actor->current.pos;` (:389) is a render / `cVECHalfAngle` input
        // (`setEffectMtx`, `:54-56`).
    }

    /// `demo_camera` (`:130-200`), the `field_0x2E8` state machine, minus the camera.
    ///
    /// ```c
    /// switch ((s8)i_this->field_0x2E8) {
    /// case 0: break;
    /// case 1:
    ///     if (!i_this->eventInfo.checkCommandDemoAccrpt()) {
    ///         fopAcM_orderPotentialEvent(i_this, dEvtFlag_STAFF_ALL_e, 0xFFFF, 0);
    ///         i_this->eventInfo.onCondition(dEvtCnd_UNK2_e);
    ///         bVar2 = false;
    ///         break;
    ///     }
    ///     i_this->field_0x2E8 = i_this->field_0x2E8 + 1;
    ///     camera->mCamera.Stop();
    ///     camera->mCamera.SetTrimSize(2);
    ///     i_this->field_0x2EA = 0;
    ///     // fallthrough
    /// case 2: ... rope drop ...
    /// }
    /// if ((i_this->field_0x2E8 != 0) && bVar2) { camera->mCamera.Set(...); ...; field_0x2EA++; }
    /// ```
    ///
    /// DROPPED: every `camera->mCamera.*` call (`:145-146`, `:183-185`, `:196`) - the camera has no
    /// modelled counterpart - and `fopAcM_orderPotentialEvent(i_this, dEvtFlag_STAFF_ALL_e, 0xFFFF, 0)`
    /// (`:139`), which is `dComIfGp_event_order(dEvtType_POTENTIAL_e, 0xFF, flag, hind, this,
    /// player)` (`f_op_actor_mng.cpp:743-748`): a PRIORITY-arbitrated request with NO event-list
    /// index, so `super::event::ActiveEvent::order` cannot express it and there is no index to pass.
    ///
    /// That omission decides the machine's fate, faithfully: `checkCommandDemoAccrpt()` (`:138`) is
    /// the per-actor `dEvtCmd_INDEMO_e` latch the sim does not carry (the same omission `hsh.rs:327`,
    /// `pscnchg.rs:99` and `hami3.rs:137` record), so the order is never accepted, `field_0x2E8`
    /// stays 1 forever and the rope never drops. Nothing is forced here.
    fn demo_camera(&mut self, actor: &mut Actor, switches: &mut Switches) {
        match self.demo_mode {
            // `case 0: break;` (:135-136)
            0 => {}
            // `case 1:` (:137-148) - only reachable from `:378`.
            1 => {
                // `fopAcM_orderPotentialEvent` / `onCondition(dEvtCnd_UNK2_e)` (:139-140) are the
                // dropped order; `bVar2 = false; break;` (:141-142) means case 1 also skips the
                // trailing `field_0x2EA++` at `:198`.
            }
            // `case 2:` (:149-192), reached only by the fallthrough at `:148`.
            2 => {
                // `i_this->current.pos.y = i_this->home.pos.y + i_this->field_0x30C;` (:150)
                actor.pos.y = self.home_y + self.drop;
                // `s32 uVar3 = (s32) i_this->field_0x2EA;` (:152)
                let frame = self.demo_frame;
                if frame < DROP_IDLE_END {
                    // `f32 sin_result = cM_ssin((uVar3 & 0x1F) * 0x800);
                    //  i_this->field_0x30C = sin_result * 5.0f;` (:154-155)
                    let phase = (frame & 0x1F).wrapping_mul(DROP_SWAY_STEP);
                    self.drop = super::stream::sin_s(phase) * DROP_SWAY_AMP;
                } else if frame <= DROP_RING {
                    // `if (uVar3 == 27) { fopAcM_seStartCurrent(i_this, JA_SE_OBJ_ROPE_SW_ON, 0);
                    //  dComIfGp_getVibration().StartShock(3, -33, cXyz(0.0f, 1.0f, 0.0f)); }`
                    // (:157-160) - a sound and a rumble with no modelled reader; the sim HAS
                    // `super::vibration::Vibration` but the brief lists rumble as a confirmed-absent
                    // input, so nothing is pushed.
                    // `cLib_addCalc2(&i_this->field_0x30C, -70.0f, 1.0f, 10.0f);` (:161)
                    let step =
                        ((DROP_REST - self.drop) * DROP_SCALE).clamp(-DROP_MAX_STEP, DROP_MAX_STEP);
                    self.drop += step;
                } else if frame < DROP_SETTLE_END {
                    // `f32 sin_result = cM_ssin(uVar3 * 0x3A00);
                    //  i_this->field_0x30C = sin_result * 5.0f + (-70.0f);` (:163-164)
                    let phase = frame.wrapping_mul(DROP_SETTLE_STEP);
                    self.drop = super::stream::sin_s(phase) * DROP_SWAY_AMP + DROP_REST;
                }
                // ```
                // i_this->field_0x2F8 = i_this->home.pos;
                // i_this->field_0x2F8.y += REG8_F(0) + 200.0f;
                // cMtx_YrotS(*calc_mtx, i_this->current.angle.y);
                // vec.x = REG8_F(1) + 800.f; vec.y = REG8_F(2); vec.z = REG8_F(3) + 100.0f;
                // i_this->field_0x2EC = i_this->home.pos + posVec;
                // ```
                // (:167-178) - the camera eye and look-at, both unmodelled.
                if frame == DEMO_END {
                    // ```
                    // i_this->field_0x2E8 = 0;
                    // camera->mCamera.Reset(field_0x2F8, field_0x2EC);
                    // camera->mCamera.Start();
                    // camera->mCamera.SetTrimSize(0);
                    // dComIfGp_event_reset();
                    // dComIfGs_onSwitch(i_this->mSwitchNo, fopAcM_GetRoomNo(i_this));
                    // mDoAud_seStart(JA_SE_READ_RIDDLE_1);
                    // ```
                    // (:181-191)
                    self.demo_mode = 0;
                    // `dComIfGs_onSwitch(i_this->mSwitchNo, fopAcM_GetRoomNo(i_this));` (:189)
                    switches.set(self.switch_no);
                }
                // `if ((i_this->field_0x2E8 != 0) && bVar2) { ...; i_this->field_0x2EA++; }`
                // (:195-199) - the camera `Set` and the `JUTReport` are dropped; the counter is not.
                self.demo_frame += 1;
            }
            // `switch` has no default (`:134-193`); `field_0x2E8` is an `s8` and only 0, 1 and 2
            // are ever written (`:378`, `:144`), so nothing else is reachable.
            _ => {}
        }
    }
}

/// `cLib_addCalcAngleS2(s16* pValue, s16 target, s16 scale, s16 maxStep)`
/// (`c_lib.cpp:192-202`), transcribed as a pure function:
///
/// ```c
/// s16 diff = target - *pValue;
/// s16 step = diff / scale;
/// if (step > maxStep)      { *pValue += maxStep; }
/// else if (step < -maxStep) { *pValue -= maxStep; }
/// else                     { *pValue += step; }
/// ```
///
/// This is NOT `super::walk::add_calc_angle_s` (that one models the five-argument
/// `cLib_addCalcAngleS`, which has the extra `minStep` branch) and NOT
/// `super::stream::add_calc_angle_s2`, which the brief's joint-turning rule explicitly forbids
/// introducing. RULE 10: the subtraction and the division are `s16` in retail, so both are `i16`
/// here, and the addition is a wrapping `i16` add.
fn chase_angle_s2(value: i16, target: i16, scale: i16, max_step: i16) -> i16 {
    let diff = target.wrapping_sub(value);
    let step = diff.wrapping_div(scale);
    if step > max_step {
        value.wrapping_add(max_step)
    } else if step < max_step.wrapping_neg() {
        value.wrapping_sub(max_step)
    } else {
        value.wrapping_add(step)
    }
}

pub fn is_kui(name: &str) -> bool {
    KUI_ACTORS.contains(&name)
}

pub fn control(
    mut posts: Query<(Entity, &mut Actor, &mut Kui)>,
    stay: Res<StayRoom>,
    mut switches: ResMut<Switches>,
    mut commands: Commands,
) {
    let stay_room = stay.0;
    for (entity, mut actor, mut post) in &mut posts {
        if !post.entered {
            // `fopAc_Init` seeds `home` from the stage placement, which is what `current.pos` holds
            // before the first `Execute` (`:203`) runs.
            post.home_y = actor.pos.y;
            post.entered = true;
            // RULE 6b: `daKui_Create` returned `cPhs_ERROR_e`, which cancels the process during
            // Create (`f_pc_create_req.cpp:106-108`) - the actor NEVER executes even once. Despawn
            // on this first tick rather than leaking a bare `Actor` entity retail never keeps.
            if post.refused {
                let _ = commands.entity(entity).despawn();
                continue;
            }
        }
        // `if (i_this->field_0x2A2 != 0) { ... }` (:215-229) - the dragon-tail rope follow, DROPPED:
        // it calls `search_dragontail` / `search_btd` for `fpcNm_DR2_e` / `fpcNm_BTD_e` (`:30-45`)
        // and then reads `dragon_tail->unk_3B0`, `dragon_tail->unk_4BA` and `btd->m02E4`, none of
        // which has a counterpart here - `src/sim/dr2.rs` does not exist and RULE 12(b) forbids
        // adding a public item another module does not consume. The flag is decoded anyway so the
        // gap is visible.
        let _ = post.on_tail;
        // `dComIfGp_roomControl_getStayNo()` (:362) - `super::door::StayRoom` is the correct store;
        // `Switches` does not model the per-room split (RULE 11b), so the value is read and its
        // limitation reported rather than used to invent a per-room bit.
        let _ = stay_room;
        post.execute(&mut actor, &mut switches);
    }
}
