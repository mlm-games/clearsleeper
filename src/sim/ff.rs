use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::ground;
use super::player::Player;
use super::walk;

/// `OBJNAME("ho", fpcNm_FF_e, 255, 0)` (`d_stage.cpp:641`) is `fpcNm_FF_e` = `d_a_ff.cpp`, the
/// only `fpcNm_FF_e` row in the table.
///
/// The object archive is `"Ff"` (`dComIfG_resLoad(&mPhs, "Ff")`, `d_a_ff.cpp:291`, and
/// `dComIfG_getObjectRes("Ff", ...)` at `:258`), NOT `"ho"` - so the stage string and the archive
/// name differ here.
///
/// CASE TRAP, checked explicitly: `d_stage.cpp:1007` is a SEPARATE row
/// `OBJNAME("Ho", fpcNm_NPC_HO_e, 0, 0)` - capital `H`, a different actor, already claimed by
/// `src/sim/npc_ho.rs`. `"ho"` and `"Ho"` are distinct `&str`s, and `spawn_actor`
/// (`src/sim/mod.rs:359`) compares with `==` / `contains`, which is byte-exact and case-sensitive,
/// so the two claims cannot collide. No `*_ACTORS` array or `pub const &str` anywhere in
/// `src/sim/` claims lowercase `"ho"`.
pub const FF_ACTORS: [&str; 1] = ["ho"];

/// `l_cullSizeBox[0]` - `/* fopAc_CULLBOX_0_e */ fopAc_cullSizeBox(cXyz(-40.0f, 0.0f, -40.0f),
/// cXyz(40.0f, 125.0f, 40.0f))` (`f_op_actor_mng.cpp:558`). The profile's `fopAc_CULLBOX_0_e`
/// (`d_a_ff.cpp:375`) resolves here, and this file calls neither `fopAcM_setCullSizeBox` nor
/// `fopAcM_SetMin` / `SetMax`, so that table entry is the whole box.
const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:558
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:558

/// `int uVar1 = fopAcM_GetParam(a_this) & 0xFF;` (`d_a_ff.cpp:295`) - the number of extra
/// fireflies `daFf_Create` spawns.
const CHILD_WIDTH: u32 = 0xFF; // <- d_a_ff.cpp:295
/// `fopAcM_SetParam(a_this, fopAcM_GetParam(a_this) & 0xFF00)` (`d_a_ff.cpp:297`) - the low byte is
/// cleared BEFORE the children are created, and `pfVar4->base.parameters =
/// fopAcM_GetParam(a_this)` (`:304`) reads the CLEARED value, so a child always sees zero children
/// and the spawn cannot recurse.
const CHILD_KEEP_MASK: u32 = 0xFF00; // <- d_a_ff.cpp:297
/// `a_this->mbNoUseGroundY = fopAcM_GetParam(a_this) >> 8;` (`d_a_ff.cpp:308`) - the high byte,
/// stored into a `u8` (`d_a_ff.h:16`), so it is `>> 8` and no mask.
const NO_GROUND_Y_SHIFT: u32 = 8; // <- d_a_ff.cpp:308
/// `pfVar4->base.position.x = a_this->current.pos.x + cM_rndFX(500.0f);` (`:300`) and the
/// matching `.z` (`:302`) - the child's spawn scatter. The child's `.y` is plain
/// `a_this->current.pos.y` (`:301`), no scatter.
const CHILD_SCATTER_XZ: f32 = 500.0; // <- d_a_ff.cpp:300

/// `useHeapInit`'s `0.9f + cM_rndF(0.15f)` (`d_a_ff.cpp:276`), drawn once per archive -
/// `ho_bmd` has two entries (`:253`). The value feeds a BRK animation speed, which is archive data
/// (RULE 3b) with no modelled reader, but the DRAWS still advance the `cM_rnd` stream the
/// create-time sequence below depends on, so [`Ff::create_init`] spends them.
const BRK_SPEED_SPAN: f32 = 0.15; // <- d_a_ff.cpp:276
/// `a_this->mLiveTimer = cM_rndF(0x8000);` (`d_a_ff.cpp:313`) - an `f32` assigned to the `s16`
/// `mLiveTimer` (`d_a_ff.h:33`), i.e. truncated toward zero.
const LIVE_TIMER_SPAN: f32 = 32768.0; // <- d_a_ff.cpp:313 (0x8000)
/// `a_this->mTimers[2] = cM_rndF(100.0f);` (`d_a_ff.cpp:314`) - likewise truncated into an `s16`.
const TIMER2_SEED_SPAN: f32 = 100.0; // <- d_a_ff.cpp:314

/// `i_this->mTimers[1] = cM_rndF(50.0f) + 40.0f;` (`d_a_ff.cpp:87`).
const TIMER1_SPAN: f32 = 50.0; // <- d_a_ff.cpp:87
const TIMER1_BASE: f32 = 40.0; // <- d_a_ff.cpp:87
/// `i_this->mTimers[2] = cM_rndF(200.0f) + 100.0f;` (`d_a_ff.cpp:88`).
const TIMER2_RESPAWN_SPAN: f32 = 200.0; // <- d_a_ff.cpp:88
const TIMER2_RESPAWN_BASE: f32 = 100.0; // <- d_a_ff.cpp:88

/// `i_this->mScatterPos.x = i_this->mHomePos.x + cM_rndFX(750.0f);` (`:129`) and `.z` (`:130`).
const SCATTER_XZ: f32 = 750.0; // <- d_a_ff.cpp:129
/// `i_this->mScatterPos.y = i_this->mHomePos.y + cM_rndFX(225.0f) + 137.5f;` (`d_a_ff.cpp:131`).
const SCATTER_Y: f32 = 225.0; // <- d_a_ff.cpp:131
const SCATTER_Y_BASE: f32 = 137.5; // <- d_a_ff.cpp:131
/// `i_this->mVelocityFwdTarget = cM_rndF(20.0f) + 10.0f;` (`d_a_ff.cpp:133`).
const FWD_SPAN: f32 = 20.0; // <- d_a_ff.cpp:133
const FWD_BASE: f32 = 10.0; // <- d_a_ff.cpp:133

/// `xd = cM_rndF(100.0f); i_this->mTimers[3] = (xd + 1000.0f);` (`d_a_ff.cpp:121`) and the
/// identical draw at `:170`.
const SCATTER_WINDOW_SPAN: f32 = 100.0; // <- d_a_ff.cpp:121
const SCATTER_WINDOW_BASE: f32 = 1000.0; // <- d_a_ff.cpp:121

/// `if (((xd * xd) + (yd * yd) + (zd * zd)) < 2500.0f) { i_this->mMode = 1; }` (`d_a_ff.cpp:159`)
/// - the SQUARED arrival radius of the scatter chase, compared against a squared distance.
const SCATTER_ARRIVE_SQ: f32 = 2500.0; // <- d_a_ff.cpp:159

/// `f30 = cM_rndF(1.0f); if (f30 < REG13_F(0) + 0.1f) {...}` (`d_a_ff.cpp:174`, repeated at
/// `:179`). `REG13_F(i)` is `g_regHIO.mChild[13].mFloatRegs[i]` (`d_s_play.h:221`), and
/// `dScnPly_reg_childHIO_c::__ct__` zeroes all 30 `mFloatRegs` (`d_s_play.cpp:81`) for every one
/// of the 22 children (`d_s_play.h:163`), so RULE 2b applies verbatim to child 13 and every
/// `REG13_F(n)` below is a proven `0.0f` - the same proof as `REG0_F`, not a new assumption.
const GUST_PROBABILITY: f32 = 0.1; // <- d_a_ff.cpp:174 (REG13_F(0), proven 0.0f)
/// `f30 = cM_rndFX(REG13_F(1) + 1.0f); i_this->m2C0.x = f30;` (`d_a_ff.cpp:175`, repeated for
/// `.z` at `:180`).
const GUST_SWAY: f32 = 1.0; // <- d_a_ff.cpp:175 (REG13_F(1), proven 0.0f)
/// `i_this->m2C0.y = REG13_F(2) + 2.0f;` (`d_a_ff.cpp:183`).
const GUST_LIFT: f32 = 2.0; // <- d_a_ff.cpp:183 (REG13_F(2), proven 0.0f)
/// `if (i_this->mVelImpulse.y > REG13_F(3) + 10.0f) { i_this->mVelImpulse.y = REG13_F(3) +
/// 10.0f; }` (`d_a_ff.cpp:184-185`).
const GUST_CEILING: f32 = 10.0; // <- d_a_ff.cpp:184 (REG13_F(3), proven 0.0f)

/// `i_this->current.angle.x = -0x3000;` (`d_a_ff.cpp:122`) - the pitch the firefly adopts when it
/// first notices the player. `-0x3000` is a negative literal that already fits `i16`; this is not
/// the `-0x8000` overflow case.
const NOTICE_PITCH: i16 = -0x3000; // <- d_a_ff.cpp:122
/// `i_this->speedF = 10.0f;` (`d_a_ff.cpp:123`).
const NOTICE_SPEED_F: f32 = 10.0; // <- d_a_ff.cpp:123
/// `if (xd < 250.0f)` (`d_a_ff.cpp:118`) - the radius at which the firefly notices the player.
const NOTICE_DIST: f32 = 250.0; // <- d_a_ff.cpp:118

/// `step = 500.0f;` (`d_a_ff.cpp:193`) - the float scale behind both `cLib_addCalcAngleS2` max
/// steps, `step * i_this->mRotVel` (`:194-197`), an `f32` product truncated into the `s16` param.
const ANGLE_STEP_SCALE: f32 = 500.0; // <- d_a_ff.cpp:193
/// `cLib_addCalcAngleS2(&i_this->current.angle.y, i_this->mTargetRotY, 10, ...)`
/// (`d_a_ff.cpp:194`), and the `.x` twin at `:196`.
const ANGLE_STEP_SCALE_S: i16 = 10; // <- d_a_ff.cpp:194
/// `local_cc.z = i_this->speedF * 0.25f;` (`d_a_ff.cpp:202`) - the local forward vector that
/// `MtxPosition` rotates into `speed`.
const SPEED_SPAN: f32 = 0.25; // <- d_a_ff.cpp:202

/// `cLib_addCalc2(&i_this->mRotVel, 10.0f, 1.0f, 0.15f)` (`d_a_ff.cpp:162`) - mode 3 drives the
/// rotation rate to 10 while chasing a scatter point.
const CHASE_ROT_TARGET: f32 = 10.0; // <- d_a_ff.cpp:162
const CHASE_ROT_SCALE: f32 = 1.0; // <- d_a_ff.cpp:162
const CHASE_ROT_MAX: f32 = 0.15; // <- d_a_ff.cpp:162
/// `cLib_addCalc2(&i_this->mRotVel, 1.0f, 1.0f, 0.1f)` (`d_a_ff.cpp:198`) - every tail tick the
/// rate relaxes back to 1.
const REST_ROT_TARGET: f32 = 1.0; // <- d_a_ff.cpp:198
const REST_ROT_SCALE: f32 = 1.0; // <- d_a_ff.cpp:198
const REST_ROT_MAX: f32 = 0.1; // <- d_a_ff.cpp:198
/// `cLib_addCalc2(&i_this->speedF, i_this->mVelocityFwdTarget, 1.0f, 3.0f)` (`d_a_ff.cpp:199`).
const SPEEDF_SCALE: f32 = 1.0; // <- d_a_ff.cpp:199
const SPEEDF_MAX: f32 = 3.0; // <- d_a_ff.cpp:199

/// `cLib_addCalc2(&i_this->current.pos.x, i_this->mHomePos.x, 0.1f, std::abs(i_this->speed.x))`
/// (`d_a_ff.cpp:111`) and the `.y` / `.z` twins (`:112`, `:113`) - mode 1's pull home. The `0.1f`
/// is the scale; the max step is the CURRENT speed's magnitude, so the whole of `speed` is read
/// before any part of it changes.
const HOME_PULL_SCALE: f32 = 0.1; // <- d_a_ff.cpp:111

/// `pos.y += 250.0f; chk.SetPos(&pos);` (`d_a_ff.cpp:101-102`) - mode 0 probes for the ground from
/// a point 250 above the actor.
const GROUND_PROBE_LIFT: f32 = 250.0; // <- d_a_ff.cpp:101
/// `i_this->mGroundY = dComIfG_Bgsp()->GroundCross(&chk) + 12.5f;` (`d_a_ff.cpp:103`), and the same
/// `12.5f` in `label_870`'s test (`:212`) and clamp (`:214`).
const GROUND_LIFT: f32 = 12.5; // <- d_a_ff.cpp:103

/// `cM_rnd` (`c_math.cpp:177-182`) - `r0 = (r0 * 0xAB) % 0x763D`, `r1 * 0xAC % 0x7663`,
/// `r2 * 0xAA % 0x7673`, returning `fabsf(fmodf(r0/30269.0 + r1/30307.0 + r2/30323.0, 1.0))`.
const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA]; // <- c_math.cpp:178-180
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673]; // <- c_math.cpp:178-180
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0]; // <- c_math.cpp:181
/// `cM_initRnd(100, 100, 100)` (`m_Do_machine.cpp:617`) - the only seed retail ever loads into
/// `cM_rnd`'s three lanes (`c_math.cpp:170-174`).
const RND_SEED: i32 = 100; // <- m_Do_machine.cpp:617

/// `fire_fly_move`'s `switch (i_this->mMode)` (`d_a_ff.cpp:97`) - the `s8` `mMode` (`d_a_ff.h:34`)
/// and its only five written values, in the decomp's own order. `d_a_ff.h` declares no enum;
/// these are the literal `case` labels.
const MODE_SPAWN: i8 = 0; // <- d_a_ff.cpp:98
const MODE_HOME: i8 = 1; // <- d_a_ff.cpp:110
const MODE_SCATTER: i8 = 2; // <- d_a_ff.cpp:127
const MODE_CHASE: i8 = 3; // <- d_a_ff.cpp:152
const MODE_GUST: i8 = 4; // <- d_a_ff.cpp:165
/// `s16 mTimers[5]` (`d_a_ff.h:32`), all five drained by `daFf_Execute` (`:224-228`).
const TIMER_COUNT: usize = 5; // <- d_a_ff.h:32

/// The Forest Haven fireflies that are NOT collectable - `ff_class` (`d_a_ff.h:10-39`), the actor
/// behind the single stage string `ho`.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `cc_sph_src` (`d_a_ff.cpp:315-342`) sets
/// `SrcObjAt Type` to 0 (`:321`), so the firefly registers no attack collider and can neither deal
/// nor receive damage; its only collider is the `dCcD_Sph` whose `SrcObjTg Type` is
/// `AT_TYPE_WIND` (`:322`) with `cCcD_TgSPrm_IsEnemy_e` (`:323`) - it PUSHES enemies rather than
/// harming them. The profile `Group` is `fopAc_ACTOR_e` (`:374`). Neither `combat::HitPoints` nor
/// `enemy::Hurt` is attached; attaching them would let `combat::control` despawn a firefly retail
/// cannot damage.
///
/// `daFf_IsDelete` (`:240-242`) is the bare "accepts a delete request" form of RULE 6b, and
/// `fopAcM_delete` appears nowhere in this translation unit. `daFf_Create` returns
/// `cPhs_ERROR_e` only from the `fopAcM_entrySolidHeap` else arm (`:347`), which is heap work, so
/// `spawn` never refuses.
///
/// `daFf_Draw` (`:64-76`), `fire_fly_draw` (`:18-38`) and `z_check` (`:41-61`) are entirely
/// renderer work - `mDoLib_project`, `GXDrawDone`, `GXPeekZ`, `mBrkAnm`, `dComIfGd_setListMaskOff`.
/// `mScale` / `mScaleTarget` (`:91`, `:96`), `mGlowScale` (`:70`, `:72`) and `mGlowScaleY`
/// (`:216`, `:218`) have those two functions as their ONLY readers, so none is stored and
/// `mbNotVisibleZ` - which exists only to choose between the two `mGlowScale` calls - is not
/// stored either. `mTimers[1]` survives only because it shares the one `s16 mTimers[5]` array; its
/// sole reader is the dropped `mScaleTarget` line (`:91`).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ff {
    /// `mMode` (`d_a_ff.h:34`, an `s8`) - the `fire_fly_move` switch subject (`:97`).
    pub mode: i8,
    /// `mTimers` (`d_a_ff.h:32`, an `s16[5]`), decremented while non-zero by `daFf_Execute`
    /// (`:224-228`).
    pub timers: [i16; TIMER_COUNT],
    /// `mLiveTimer` (`d_a_ff.h:33`, an `s16`) - incremented every `fire_fly_move` (`:85`) and
    /// seeded from `cM_rndF(0x8000)` at create (`:313`).
    pub live_timer: i16,
    /// `mMode >= 2`, the scatter target `mScatterPos` (`d_a_ff.h:27`).
    pub scatter_pos: Vec3,
    /// `mMode <= 3`, the anchor the firefly returns to - `mHomePos` (`d_a_ff.h:29`), assigned at
    /// create (`:311`), re-taken at the end of mode 0 (`:107`) and reset when a scatter window
    /// expires (`:145-147`).
    pub home_pos: Vec3,
    /// `mGroundY` (`d_a_ff.h:26`), the mode-0 ground probe result plus [`GROUND_LIFT`] (`:103`).
    pub ground_y: f32,
    /// `mTargetRotX` (`d_a_ff.h:30`, an `s16`) - the pitch aim, `-cM_atan2s(yd, xz_dist)`
    /// (`:141`, `:158`).
    pub target_rot_x: i16,
    /// `mTargetRotY` (`d_a_ff.h:31`, an `s16`) - the yaw aim, `cM_atan2s(xd, zd)` (`:139`, `:156`).
    pub target_rot_y: i16,
    /// `mRotVel` (`d_a_ff.h:18`) - the rate multiplier on both `addCalcAngleS2` max steps.
    pub rot_vel: f32,
    /// `mVelocityFwdTarget` (`d_a_ff.h:19`) - the cruise speed `speedF` chases (`:199`).
    pub velocity_fwd_target: f32,
    /// `mVelImpulse` (`d_a_ff.h:20`) - the additive gust drift, decayed by three `cLib_addCalc0`
    /// (`:207-209`) and clamped in y by [`GUST_CEILING`] (`:184-185`).
    pub vel_impulse: Vec3,
    /// `m2C0` (`d_a_ff.h:21`) - the per-frame gust vector added to `mVelImpulse` in mode 4
    /// (`:172-183`).
    pub gust: Vec3,
    /// `fopAc_ac_c::speed` (`f_op_actor.h:292`, a `cXyz`) - `MtxPosition(&local_cc,
    /// &i_this->speed)` (`:205`) writes it every tail tick, and mode 1 reads its magnitude as its
    /// `addCalc2` max step (`:111-113`), so it must persist across ticks rather than be a local.
    pub speed: Vec3,
    /// `fopAc_ac_c::speedF` (`f_op_actor.h:301`) - the local forward length (`:202`).
    pub speed_f: f32,
    /// `mbNoUseGroundY` (`d_a_ff.h:16`, a `u8`) from `fopAcM_GetParam(a_this) >> 8` (`:308`); when
    /// zero, mode 0 snaps the firefly onto its ground height (`:104-106`).
    pub no_ground_y: u8,
    /// `int uVar1 = fopAcM_GetParam(a_this) & 0xFF` (`d_a_ff.cpp:295`) - how many extra fireflies
    /// `daFf_Create` spawns.
    pub children: u8,
    /// `cM_rnd`'s three lanes (`c_math.cpp:167`, seeded once by `m_Do_machine.cpp:617`). Retail
    /// keeps ONE global triple shared by every caller; the sim has no global RNG resource, so each
    /// firefly carries its own copy, exactly as `src/sim/ajav.rs` and `src/sim/agbsw0.rs` do. The
    /// algorithm and the seed are transcribed exactly; the STREAMS are not shared between actors.
    pub seed: [i32; 3],
    /// One-shot guard so the create-time draw sequence (`:313-314`) and the child spawns
    /// (`:298-306`) happen exactly once, on the first tick, where `Commands` is available.
    pub entered: bool,
}

impl Ff {
    /// `daFf_Create` (`d_a_ff.cpp:288-351`) minus the resource load and heap entry, minus
    /// `mBrkAnm->init(...)` (`:269-283`), `cullMtx` (`:310`), `m2EC` (`:312`), `ff_count++`
    /// (`:309`), `mStts.Init` / `mSph.Set` / `mSph.SetStts` (`:343-345`), the child spawns
    /// (`:296-307`) and the `mLiveTimer` / `mTimers[2]` draws (`:313-314`) - all of which
    /// [`Ff::create_init`] performs in decomp order because they need the RNG or `Commands`.
    ///
    /// `m2EC` (`d_a_ff.h:28`) is written once at `:312` and never read in the translation unit,
    /// and `ff_count` (`:15`) is only ever incremented and zeroed, so neither is stored.
    ///
    /// `a_this->mHomePos = i_this->current.pos;` (`:311`) is likewise deferred to `create_init`,
    /// which is the only place with the actor's spawn position.
    ///
    /// RULE 2: `dStage_actorInit` never writes `fopAcM_prm_class::scale`, so the firefly's scale
    /// is exactly 1.0. It has no modelled effect anyway - `mScale` is renderer-only.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            // `fopAcM_ct`'s `sBs_ClearArea` zero-fill (`f_pc_base.cpp:98`) leaves `mMode` at 0,
            // which `fire_fly_move`'s `case 0` (`:98`) expects.
            mode: MODE_SPAWN,
            // `mTimers` likewise takes the zero-fill; `daFf_Create` overwrites only index 2
            // (`:314`), which `create_init` repeats, so every other slot starts at 0 and
            // `daFf_Execute`'s `if (mTimers[i] != 0)` guard (`:225`) keeps them there.
            timers: [0; TIMER_COUNT],
            live_timer: 0,
            scatter_pos: Vec3::ZERO,
            home_pos: Vec3::ZERO,
            ground_y: 0.0,
            target_rot_x: 0,
            target_rot_y: 0,
            rot_vel: 0.0,
            velocity_fwd_target: 0.0,
            vel_impulse: Vec3::ZERO,
            gust: Vec3::ZERO,
            // `speed` is written only by `MtxPosition` in the tail (`:205`), so it starts at the
            // `fopAcM_ct` zero-fill. That is not a guess: mode 1 reads it as the `addCalc2` max
            // step on the very first tick (`:111-113`), before the tail has ever run.
            speed: Vec3::ZERO,
            speed_f: 0.0,
            no_ground_y: ((parameters >> NO_GROUND_Y_SHIFT) & 0xFF) as u8,
            children: (parameters & CHILD_WIDTH) as u8,
            seed: [RND_SEED; 3],
            entered: false,
        }
    }

    /// `l_cullSizeBox[0]` = `fopAc_CULLBOX_0_e` (`f_op_actor_mng.cpp:558`), reached from the
    /// profile's `fopAc_CULLBOX_0_e` (`d_a_ff.cpp:375`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// The create-time draw sequence of `daFf_Create`, in the decomp's order, plus the child spawns
    /// of `:296-307`.
    ///
    /// The order is transcribed, not approximated, because the draws share one `cM_rnd` stream:
    /// 1. `mBrkAnm[i]->init(..., 0.9f + cM_rndF(0.15f), ...)` (`:276`) runs twice inside
    ///    `useHeapInit` (`:273-277`), BEFORE anything else in `Create` draws. The speeds are BRK
    ///    archive data (RULE 3b) with no modelled reader, but the two draws advance the stream, so
    ///    [`Ff::rnd`] spends them.
    /// 2. `cM_rndFX(500.0f)` for `.x` then `.z` per child (`:300`, `:302`); the child's
    ///    parameters carry the ALREADY-CLEARED `fopAcM_GetParam` (`:297`, `:304`), so a child is
    ///    created with zero children of its own. `base.angle` is all zeros (`:303`).
    /// 3. `a_this->mLiveTimer = cM_rndF(0x8000);` (`:313`) - a float truncated into the `s16`.
    /// 4. `a_this->mTimers[2] = cM_rndF(100.0f);` (`:314`) - likewise.
    fn create_init(&mut self, actor: &Actor, commands: &mut Commands) {
        // `:311`
        self.home_pos = actor.pos;
        // `useHeapInit`'s two draws (`:276`), one per `ho_bmd` entry (`:253`).
        for _ in 0..2 {
            self.rnd_f(BRK_SPEED_SPAN);
        }
        // ```
        // if (uVar1 != 0) {
        //     fopAcM_SetParam(a_this, fopAcM_GetParam(a_this) & 0xFF00);
        //     for (int i = 0; i < uVar1; i++) {
        //         pfVar4->base.position.x = a_this->current.pos.x + cM_rndFX(500.0f);
        //         pfVar4->base.position.y = a_this->current.pos.y;
        //         pfVar4->base.position.z = a_this->current.pos.z + cM_rndFX(500.0f);
        //         pfVar4->base.angle.x = pfVar4->base.angle.y = pfVar4->base.angle.z = 0;
        //         pfVar4->base.parameters = fopAcM_GetParam(a_this);
        //         fopAcM_Create(fpcNm_FF_e, NULL, pfVar4);
        //     }
        // }
        // ```
        // (:296-307)
        let cleared = actor.parameters & CHILD_KEEP_MASK;
        for _ in 0..self.children {
            let x = self.rnd_fx(CHILD_SCATTER_XZ);
            let z = self.rnd_fx(CHILD_SCATTER_XZ);
            commands.spawn((
                Actor {
                    name: FF_ACTORS[0].to_owned(),
                    parameters: cleared,
                    pos: Vec3::new(actor.pos.x + x, actor.pos.y, actor.pos.z + z),
                    angle: [0; 3],
                    set_id: 0,
                    room: actor.room,
                    ..Default::default()
                },
                Ff::spawn(cleared),
            ));
        }
        // `:313`
        self.live_timer = self.rnd_f(LIVE_TIMER_SPAN) as i32 as i16;
        // `:314`
        self.timers[2] = self.rnd_f(TIMER2_SEED_SPAN) as i32 as i16;
        self.entered = true;
    }

    /// `daFf_Execute` (`d_a_ff.cpp:223-237`).
    ///
    /// `if (i_this->mSph.ChkTgHit() != 0) { i_this->mMode = 4; i_this->mTimers[4] =
    /// REG13_F(9) + (cM_rndF(20.0f) + 30.0f); }` (`:230-233`) is transcribed as a gate fed
    /// `false`. `REG13_F(9)` is `0.0f` by the same constructor proof as the other `REG13_F` folds
    /// above, so the arm would reduce to a `cM_rndF(20.0f) + 30.0f` gust window - but the draw is
    /// not spent, because the arm never runs, so the module carries no constants for it. The
    /// sphere's `SrcObjTg Type` is `AT_TYPE_WIND` (`:322`) and RULE 3b lists `AT_TYPE_WIND`
    /// collider semantics as confirmed-absent, so there is no channel on which a wind emitter can
    /// report a hit; `super::bombflower::Blasts` is `AT_TYPE_BOMB`-shaped and is NOT a substitute.
    /// The consequence is stated plainly: [`MODE_GUST`] is unreachable in the sim, though its body
    /// is transcribed in full.
    ///
    /// `mSph.SetC(i_this->current.pos); dComIfG_Ccsp()->Set(&i_this->mSph);` (`:234-235`) is the
    /// registration for that same absent channel.
    ///
    /// `daFf_Delete` (`:245-249`) resets the file-static `ff_count` to 0 - a variable with no
    /// reader anywhere in the translation unit - and frees resources.
    fn execute(&mut self, actor: &mut Actor, player: Vec3, collision: Option<&Collision>) {
        // `for (int i = 0; i < 5; i++) { if (i_this->mTimers[i] != 0) { i_this->mTimers[i]--; } }`
        // (:224-228)
        for timer in self.timers.iter_mut() {
            if *timer != 0 {
                *timer -= 1;
            }
        }
        self.fire_fly_move(actor, player, collision);
    }

    /// `fire_fly_move` (`d_a_ff.cpp:79-220`).
    fn fire_fly_move(&mut self, actor: &mut Actor, player: Vec3, collision: Option<&Collision>) {
        // `i_this->mLiveTimer++;` (:85) - an `s16` (`d_a_ff.h:33`), so it wraps.
        self.live_timer = self.live_timer.wrapping_add(1);
        // ```
        // if (i_this->mTimers[2] == 0) {
        //     i_this->mTimers[1] = cM_rndF(50.0f) + 40.0f;
        //     i_this->mTimers[2] = cM_rndF(200.0f) + 100.0f;
        // }
        // ```
        // (:86-89)
        if self.timers[2] == 0 {
            self.timers[1] = (self.rnd_f(TIMER1_SPAN) + TIMER1_BASE) as i32 as i16;
            self.timers[2] = (self.rnd_f(TIMER2_RESPAWN_SPAN) + TIMER2_RESPAWN_BASE) as i32 as i16;
        }
        // ```
        // if (i_this->mTimers[1] == 0) {
        //     i_this->mScaleTarget = cM_ssin(i_this->mLiveTimer * 1000) * 0.15f * 0.25f + 0.225f;
        // } else {
        //     i_this->mScaleTarget = 0.0f;
        // }
        // ```
        // (:90-94) - DROPPED. `mScaleTarget`'s only reader is `cLib_addCalc2(&i_this->mScale,
        // i_this->mScaleTarget, 0.1f, 0.05f)` (:96), and `mScale`'s only reader is
        // `MtxScale(i_this->mScale, ...)` in `fire_fly_draw` (:20) - both renderer. The
        // `if (mTimers[1] == 0)` test and the `cM_ssin(mLiveTimer * 1000)` expression therefore
        // have no modelled effect and are not evaluated. `mTimers[1]` itself is kept because it
        // shares the one `s16 mTimers[5]` array drained at `:224-228`.

        // `switch (i_this->mMode)` (:97). `tail` is `false` exactly where the decomp does
        // `goto label_870` - `case 1`'s `goto label_870` (:125) and `default:`'s (:190) - because
        // `label_870` skips the whole movement block at `:193-209`.
        let mut tail = true;
        match self.mode {
            MODE_SPAWN => {
                // ```
                // pos = i_this->current.pos; pos.y += 250.0f; chk.SetPos(&pos);
                // i_this->mGroundY = dComIfG_Bgsp()->GroundCross(&chk) + 12.5f;
                // if (i_this->mbNoUseGroundY == 0) { i_this->current.pos.y = i_this->mGroundY; }
                // i_this->mHomePos = i_this->current.pos;
                // i_this->mMode++;
                // ```
                // (:99-108), then FALL-THROUGH into `case 1` (`:109`) - not a `break`.
                let probe = Vec3::new(actor.pos.x, actor.pos.y + GROUND_PROBE_LIFT, actor.pos.z);
                // `GroundCross` is `super::ground::surface`, the sim's established reduction of
                // `dComIfG_Bgsp()->GroundCross` (`src/sim/esa.rs:255-258`).
                self.ground_y = ground::surface(collision, probe).unwrap_or(0.0) + GROUND_LIFT;
                if self.no_ground_y == 0 {
                    actor.pos.y = self.ground_y;
                }
                self.home_pos = actor.pos;
                self.mode = MODE_HOME;
                tail = self.notice(actor, player);
            }
            MODE_HOME => tail = self.notice(actor, player),
            MODE_SCATTER => {
                // ```
                // if (i_this->mTimers[0] == 0) {
                //     i_this->mScatterPos.x = i_this->mHomePos.x + cM_rndFX(750.0f);
                //     i_this->mScatterPos.z = i_this->mHomePos.z + cM_rndFX(750.0f);
                //     i_this->mScatterPos.y = i_this->mHomePos.y + cM_rndFX(225.0f) + 137.5f;
                //     i_this->mRotVel = 0.0f;
                //     i_this->mVelocityFwdTarget = cM_rndF(20.0f) + 10.0f;
                //     xd = mScatterPos.x - pos.x; yd = ...; zd = ...;
                //     dist = sqrt(xd*xd + yd*yd + zd*zd);
                //     i_this->mTimers[0] = dist / i_this->mVelocityFwdTarget;
                //     i_this->mTargetRotY = cM_atan2s(xd, zd);
                //     xz_dist = sqrt(xd*xd + zd*zd);
                //     i_this->mTargetRotX = -cM_atan2s(yd, xz_dist);
                // }
                // ```
                // (:128-142). Draw order is `.x`, `.z`, THEN `.y`.
                if self.timers[0] == 0 {
                    let x = self.rnd_fx(SCATTER_XZ);
                    let z = self.rnd_fx(SCATTER_XZ);
                    let y = self.rnd_fx(SCATTER_Y) + SCATTER_Y_BASE;
                    self.scatter_pos = Vec3::new(
                        self.home_pos.x + x,
                        self.home_pos.y + y,
                        self.home_pos.z + z,
                    );
                    self.rot_vel = 0.0;
                    self.velocity_fwd_target = self.rnd_f(FWD_SPAN) + FWD_BASE;
                    let delta = self.scatter_pos - actor.pos;
                    self.timers[0] = (delta.length() / self.velocity_fwd_target) as i32 as i16;
                    let flat = (delta.x * delta.x + delta.z * delta.z).sqrt();
                    self.target_rot_y = walk::to_s16(delta.x.atan2(delta.z));
                    self.target_rot_x = walk::to_s16(-delta.y.atan2(flat));
                }
                // ```
                // if (i_this->mTimers[3] == 0) {
                //     i_this->mMode++;
                //     i_this->mScatterPos.x = i_this->mHomePos.x;
                //     i_this->mScatterPos.y = i_this->mHomePos.y;
                //     i_this->mScatterPos.z = i_this->mHomePos.z;
                //     i_this->mRotVel = 0.0f;
                // }
                // ```
                // (:143-149)
                if self.timers[3] == 0 {
                    self.mode = MODE_CHASE;
                    self.scatter_pos = self.home_pos;
                    self.rot_vel = 0.0;
                }
            }
            MODE_CHASE => {
                // ```
                // xd = ...; yd = ...; zd = ...;
                // i_this->mTargetRotY = cM_atan2s(xd, zd);
                // xz_dist = sqrt(xd*xd + zd*zd);
                // i_this->mTargetRotX = -cM_atan2s(yd, xz_dist);
                // if (((xd*xd) + (yd*yd) + (zd*zd)) < 2500.0f) { i_this->mMode = 1; }
                // cLib_addCalc2(&i_this->mRotVel, 10.0f, 1.0f, 0.15f);
                // ```
                // (:153-162)
                let delta = self.scatter_pos - actor.pos;
                let flat = (delta.x * delta.x + delta.z * delta.z).sqrt();
                self.target_rot_y = walk::to_s16(delta.x.atan2(delta.z));
                self.target_rot_x = walk::to_s16(-delta.y.atan2(flat));
                if delta.length_squared() < SCATTER_ARRIVE_SQ {
                    self.mode = MODE_HOME;
                }
                add_calc2(&mut self.rot_vel, CHASE_ROT_TARGET, CHASE_ROT_SCALE, CHASE_ROT_MAX);
            }
            MODE_GUST => {
                // ```
                // if (i_this->mTimers[4] == 0) {
                //     i_this->mMode = 2;
                //     f30 = cM_rndF(100.0f);
                //     i_this->mTimers[3] = (f30 + 1000.0f);
                // } else {
                //     i_this->mVelImpulse += i_this->m2C0;
                //     f30 = cM_rndF(1.0f);
                //     if (f30 < REG13_F(0) + 0.1f) { f30 = cM_rndFX(REG13_F(1) + 1.0f); i_this->m2C0.x = f30; }
                //     f30 = cM_rndF(1.0f);
                //     if (f30 < REG13_F(0) + 0.1f) { f30 = cM_rndFX(REG13_F(1) + 1.0f); i_this->m2C0.z = f30; }
                //     i_this->m2C0.y = REG13_F(2) + 2.0f;
                //     if (i_this->mVelImpulse.y > REG13_F(3) + 10.0f) { i_this->mVelImpulse.y = REG13_F(3) + 10.0f; }
                // }
                // ```
                // (:166-187). The `.x` re-roll happens before the `.z` re-roll, and BOTH reads of
                // `mVelImpulse` for the y clamp see the already-accumulated sum, so `m2C0` is
                // assembled before the clamp rather than after.
                if self.timers[4] == 0 {
                    self.mode = MODE_SCATTER;
                    self.timers[3] =
                        (self.rnd_f(SCATTER_WINDOW_SPAN) + SCATTER_WINDOW_BASE) as i32 as i16;
                } else {
                    let mut sway_x = self.gust.x;
                    let mut sway_z = self.gust.z;
                    if self.rnd_f(1.0) < GUST_PROBABILITY {
                        sway_x = self.rnd_fx(GUST_SWAY);
                    }
                    if self.rnd_f(1.0) < GUST_PROBABILITY {
                        sway_z = self.rnd_fx(GUST_SWAY);
                    }
                    self.gust = Vec3::new(sway_x, GUST_LIFT, sway_z);
                    let mut impulse = self.vel_impulse + self.gust;
                    if impulse.y > GUST_CEILING {
                        impulse.y = GUST_CEILING;
                    }
                    self.vel_impulse = impulse;
                }
            }
            // `default: goto label_870;` (:189-190)
            _ => tail = false,
        }

        if tail {
            // ```
            // step = 500.0f;
            // cLib_addCalcAngleS2(&i_this->current.angle.y, i_this->mTargetRotY, 10, step * i_this->mRotVel);
            // cLib_addCalcAngleS2(&i_this->current.angle.x, i_this->mTargetRotX, 10, step * i_this->mRotVel);
            // cLib_addCalc2(&i_this->mRotVel, 1.0f, 1.0f, 0.1f);
            // cLib_addCalc2(&i_this->speedF, i_this->mVelocityFwdTarget, 1.0f, 3.0f);
            // local_cc.x = 0.0f; local_cc.y = 0.0f; local_cc.z = i_this->speedF * 0.25f;
            // cMtx_YrotS(*calc_mtx, i_this->current.angle.y);
            // cMtx_XrotM(*calc_mtx, i_this->current.angle.x);
            // MtxPosition(&local_cc, &i_this->speed);
            // i_this->current.pos += (i_this->speed + i_this->mVelImpulse);
            // cLib_addCalc0(&i_this->mVelImpulse.x, 1.0f, 0.1f);
            // cLib_addCalc0(&i_this->mVelImpulse.y, 1.0f, 0.1f);
            // cLib_addCalc0(&i_this->mVelImpulse.z, 1.0f, 0.1f);
            // ```
            // (:193-209). `step * i_this->mRotVel` is an `f32` product truncated into the `s16`
            // max-step parameter of `cLib_addCalcAngleS2`.
            let max_step = (ANGLE_STEP_SCALE * self.rot_vel) as i32 as i16;
            actor.angle[1] = add_calc_angle_s2(
                actor.angle[1],
                self.target_rot_y,
                ANGLE_STEP_SCALE_S,
                max_step,
            );
            actor.angle[0] = add_calc_angle_s2(
                actor.angle[0],
                self.target_rot_x,
                ANGLE_STEP_SCALE_S,
                max_step,
            );
            add_calc2(&mut self.rot_vel, REST_ROT_TARGET, REST_ROT_SCALE, REST_ROT_MAX);
            add_calc2(
                &mut self.speed_f,
                self.velocity_fwd_target,
                SPEEDF_SCALE,
                SPEEDF_MAX,
            );
            // ```
            // cMtx_YrotS(*calc_mtx, angle.y); cMtx_XrotM(*calc_mtx, angle.x);
            // MtxPosition(&local_cc, &i_this->speed);
            // ```
            // (:203-205) - see [`rotated_forward`].
            self.speed = rotated_forward(
                actor.angle[1],
                actor.angle[0],
                self.speed_f * SPEED_SPAN,
            );
            // `i_this->current.pos += (i_this->speed + i_this->mVelImpulse);` (:206) - the RHS is
            // hoisted so the mutable borrow of `actor` does not span it.
            let drift = self.speed + self.vel_impulse;
            actor.pos += drift;
            // `cLib_addCalc0(&i_this->mVelImpulse.{x,y,z}, 1.0f, 0.1f)` (:207-209) - a plain
            // multiplicative decay toward 0 with no `!= target` guard (`c_lib.cpp:69-77`).
            add_calc0(&mut self.vel_impulse.x, 1.0, 0.1);
            add_calc0(&mut self.vel_impulse.y, 1.0, 0.1);
            add_calc0(&mut self.vel_impulse.z, 1.0, 0.1);
        }

        // ```
        // label_870:
        // if (i_this->current.pos.y < i_this->mGroundY + 12.5f) {
        //     if (i_this->current.pos.y < i_this->mGroundY) { i_this->current.pos.y = i_this->mGroundY; }
        //     cLib_addCalc2(&i_this->mGlowScaleY, 0.5f, 0.2f, 0.1f);
        // } else {
        //     cLib_addCalc2(&i_this->mGlowScaleY, 1.0f, 0.2f, 0.1f);
        // }
        // ```
        // (:211-219). The two `mGlowScaleY` calls are DROPPED - `mGlowScaleY`'s only reader is
        // `MtxScale(..., i_this->mGlowScale * i_this->mGlowScaleY, ...)` in `fire_fly_draw`
        // (`:29-30`), i.e. renderer. Both comparisons and the y clamp are kept.
        if actor.pos.y < self.ground_y + GROUND_LIFT && actor.pos.y < self.ground_y {
            actor.pos.y = self.ground_y;
        }
    }

    /// `case 1:`'s body (`d_a_ff.cpp:110-126`) - pulled into a helper because mode 0 FALLS THROUGH
    /// into it (`:109`) rather than breaking.
    ///
    /// Returns whether the caller reached `goto label_870` (`:125`), i.e. `false` means the
    /// movement tail at `:193-209` is skipped.
    fn notice(&mut self, actor: &mut Actor, player: Vec3) -> bool {
        // ```
        // cLib_addCalc2(&i_this->current.pos.x, i_this->mHomePos.x, 0.1f, std::abs(i_this->speed.x));
        // cLib_addCalc2(&i_this->current.pos.y, i_this->mHomePos.y, 0.1f, std::abs(i_this->speed.y));
        // cLib_addCalc2(&i_this->current.pos.z, i_this->mHomePos.z, 0.1f, std::abs(i_this->speed.z));
        // ```
        // (:111-113)
        self.pull_home(&mut actor.pos);
        // ```
        // xd = player.x - pos.x; yd = ...; zd = ...;
        // xd = sqrtf(xd*xd + yd*yd + zd*zd);
        // if (xd < 250.0f) {
        //     i_this->mMode++; xd = cM_rndF(100.0f); i_this->mTimers[3] = (xd + 1000.0f);
        //     i_this->current.angle.x = -0x3000; i_this->speedF = 10.0f;
        // }
        // ```
        // (:114-124). `xd` is REUSED for the distance, so the draw inside the arm overwrites it;
        // the tail `goto label_870` (`:125`) then uses nothing from it.
        let delta = player - actor.pos;
        if delta.length() < NOTICE_DIST {
            self.mode = MODE_SCATTER;
            self.timers[3] = (self.rnd_f(SCATTER_WINDOW_SPAN) + SCATTER_WINDOW_BASE) as i32 as i16;
            actor.angle[0] = NOTICE_PITCH;
            self.speed_f = NOTICE_SPEED_F;
            return false;
        }
        true
    }

    /// Mode 1's homeward pull (`d_a_ff.cpp:111-113`): three `cLib_addCalc2` calls whose max step
    /// is the CURRENT magnitude of the matching `speed` component, so all three are read before any
    /// of them changes.
    fn pull_home(&self, pos: &mut Vec3) {
        add_calc2(&mut pos.x, self.home_pos.x, HOME_PULL_SCALE, self.speed.x.abs());
        add_calc2(&mut pos.y, self.home_pos.y, HOME_PULL_SCALE, self.speed.y.abs());
        add_calc2(&mut pos.z, self.home_pos.z, HOME_PULL_SCALE, self.speed.z.abs());
    }

    /// `cM_rnd` (`c_math.cpp:177-182`) on this actor's own copy of the three lanes.
    fn rnd(&mut self) -> f32 {
        for lane in 0..3 {
            self.seed[lane] = (self.seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
        }
        let sum = self.seed[0] as f32 / RND_SCALE[0]
            + self.seed[1] as f32 / RND_SCALE[1]
            + self.seed[2] as f32 / RND_SCALE[2];
        (sum % 1.0).abs()
    }

    /// `cM_rndF(f) = cM_rnd() * f` (`c_math.cpp:185-187`).
    fn rnd_f(&mut self, max: f32) -> f32 {
        self.rnd() * max
    }

    /// `cM_rndFX(f) = f * (cM_rnd() - 0.5f) * 2.0f` (`c_math.cpp:190-192`).
    fn rnd_fx(&mut self, max: f32) -> f32 {
        max * (self.rnd() - 0.5) * 2.0
    }
}

/// `cLib_addCalc2(f32* o_value, f32 target, f32 scale, f32 maxStep)` (`c_lib.h:16`,
/// `c_lib.cpp:56-66`): `step = scale * (target - *o_value)` clamped to `+-maxStep`, applied only
/// when `*o_value != target`. There is no `minStep` arm here - that is `cLib_addCalc`.
fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value != target {
        let step = (scale * (target - *value)).clamp(-max_step, max_step);
        *value += step;
    }
}

/// `cLib_addCalc0(f32* o_value, f32 scale, f32 maxStep)` (`c_lib.h:17`, `c_lib.cpp:69-77`):
/// `step = clamp(*o_value * scale, +-maxStep)` then `*o_value -= step`. Unlike `cLib_addCalc2`
/// this runs unconditionally, including at zero.
fn add_calc0(value: &mut f32, scale: f32, max_step: f32) {
    let step = (*value * scale).clamp(-max_step, max_step);
    *value -= step;
}

/// `cLib_addCalcAngleS2(s16* pValue, s16 target, s16 scale, s16 maxStep)` (`c_lib.h:23`,
/// `c_lib.cpp:191-202`): `diff = target - *pValue`, `step = diff / scale`, then
/// `+= maxStep` / `-= maxStep` / `+= step` by which side of `maxStep` `step` falls on. Note there
/// is NO `*pValue != target` guard - at equality `diff` and `step` are both 0 and the last arm
/// adds 0 - and no `minStep` arm. Both `scale` (the literal 10) and `maxStep` are non-zero here,
/// so neither the divide nor the `i16::MIN / -1` overflow can arise.
fn add_calc_angle_s2(value: i16, target: i16, scale: i16, max_step: i16) -> i16 {
    let diff = target.wrapping_sub(value);
    let step = diff / scale;
    if step > max_step {
        value.wrapping_add(max_step)
    } else if step < -max_step {
        value.wrapping_sub(max_step)
    } else {
        value.wrapping_add(step)
    }
}

/// `cMtx_YrotS(*calc_mtx, angle.y)` then `cMtx_XrotM(*calc_mtx, angle.x)` then
/// `MtxPosition(&local_cc, &i_this->speed)` (`d_a_ff.cpp:203-205`), applied to
/// `local_cc = (0, 0, speedF * 0.25f)` (`:200-202`).
///
/// `cMtx_YrotS(mtx, r)` REPLACES `mtx` with `Ry(r)` and `cMtx_XrotM(mtx, r)` POST-multiplies
/// `Rx(r)` onto its right, so the product is `Ry(angle.y) * Rx(angle.x)`. `MtxPosition` is
/// `MTXMultVec` (`c_lib.cpp:433-435`), which transforms its argument as a DIRECTION (`w = 0`), so
/// there is no translation term. Expanding `Ry * Rx * (0, 0, v)` by hand gives exactly the
/// components below, and `cM_ssin` / `cM_scos` (`c_math.h:39-41`, `:35-37`) already work in the
/// `0x10000 == one full turn` units `super::walk::from_s16` implements (RULE 5b).
fn rotated_forward(yaw: i16, pitch: i16, v: f32) -> Vec3 {
    let (yaw_sin, yaw_cos) = walk::from_s16(yaw).sin_cos();
    let pitch_sin = ssin(pitch);
    let pitch_cos = scos(pitch);
    Vec3::new(v * yaw_sin, -v * yaw_cos * pitch_sin, v * yaw_cos * pitch_cos)
}

/// `cM_ssin(s16 x) = JMASSin(x)` (`c_math.h:39-41`).
fn ssin(angle: i16) -> f32 {
    walk::from_s16(angle).sin()
}

/// `cM_scos(s16 x) = JMASCos(x)` (`c_math.h:35-37`).
fn scos(angle: i16) -> f32 {
    walk::from_s16(angle).cos()
}

pub fn is_ff(name: &str) -> bool {
    FF_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    players: Query<&Player>,
    mut flies: Query<(&mut Actor, &mut Ff)>,
    mut commands: Commands,
) {
    // `dComIfGp_getPlayer(0)` (`d_a_ff.cpp:80`) is dereferenced unconditionally four times inside
    // `fire_fly_move`, so retail has no defined behaviour without a player actor. The sim has none
    // either, so the firefly's tick is skipped rather than fed a substituted position.
    let Some(player) = players.iter().next().map(|player| player.pos) else {
        return;
    };
    for (mut actor, mut fly) in &mut flies {
        if !fly.entered {
            fly.create_init(&actor, &mut commands);
        }
        fly.execute(&mut actor, player, collision.as_deref());
    }
}