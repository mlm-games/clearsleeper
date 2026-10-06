//! Keese - `fpcNm_KI_e`, `d_a_ki.cpp`, stage strings `keeth` (`d_stage.cpp:779`) and `Fkeeth`
//! (`:780`). This closes the `fpcNm_KI_e` OBJNAME gap: the table binds both names to THIS
//! translation unit and nothing else claims them.
//!
//! (a) inbound damage: YES. `mDamageSphere` is built from `tg_sph_src` (`:1311-1338`), whose
//! `SrcObjTg Type` is `AT_TYPE_ALL & ~AT_TYPE_WATER & ~AT_TYPE_UNK20000 & ~AT_TYPE_UNK400000 &
//! ~AT_TYPE_LIGHT` (`:1318`) with `cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsEnemy_e` (`:1319`), and
//! `create` sets `health = 1; max_health = 1` (`:1431-1432`). `combat::HitPoints(1)` and
//! `enemy::Hurt` are therefore attached.
//!
//! (b) outbound damage: modelled but unreachable. `m580` is built from `at_sph_src`
//! (`:1283-1310`) - `AT_TYPE_UNK800` (`:1287`), `Atp = 1` (`:1288`),
//! `cCcD_AtSPrm_Set_e | cCcD_AtSPrm_VsPlayer_e` (`:1289`) - and `Execute` parks its radius at 0
//! (`:995`). Only `ki_atack_move` behaviour 3 opens it to 30.0f (`:580`), and behaviour 3 is only
//! entered from behaviour 2 behind `cM_rndF(1.0f) < tmp` (`:537`), which this sim cannot
//! evaluate. The radius and the contact test are transcribed; the arm never arms.
//!
//! Three of this actor's inputs do not exist here and are reported under OMITTED: the room `PATH`
//! blocks (`dPath_GetRoomPath`), `cM_rndF` (there is no global RNG in this sim - `ajav.rs` and
//! `agbsw0.rs` carry per-actor LCGs and `vibration::Vibration::rnd()` is the rumble channel's),
//! and the player's per-tick displacement (`daPy_py_c::old.pos`), which `ki_wait_move` and
//! `ki_fire_set_move` both test with `sp58.abs() > 1.0f`.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::combat::HitPoints;
use super::enemy::Hurt;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::ground;
use super::health::Health;
use super::player::Player;
use super::stream;

/// `d_stage.cpp:779-780`: both names bind to `fpcNm_KI_e`, the profile of
/// `d_a_ki.cpp:1514-1529`. The archive is `"Ki"` (`:1372`).
pub const KI_ACTORS: [&str; 2] = ["keeth", "Fkeeth"];

/// `ki_class::Action_e` (`d_a_ki.h:15-23`), transcribed with every numeric value.
const ACT_WAIT_MOVE: u8 = 0; // <- d_a_ki.h:16
const ACT_FLY_MOVE: u8 = 1; // <- d_a_ki.h:17
const ACT_FIRE_SET_MOVE: u8 = 2; // <- d_a_ki.h:18
const ACT_ATTACK_MOVE: u8 = 3; // <- d_a_ki.h:19
const ACT_ATTACK_MOVE_INDEX: u8 = 10; // <- d_a_ki.h:20
const ACT_DAMAGE_MOVE: u8 = 20; // <- d_a_ki.h:21
const ACT_FAIL_MOVE: u8 = 30; // <- d_a_ki.h:22

/// `i_this->mKiPathIndex = fopAcM_GetParam(a_this) >> 0x10` (`:1382`) lands in the `u8`
/// `mKiPathIndex` (`d_a_ki.h:31`), and the two tests are `!= 0xFF` (`:1409`, `:672`).
const PATH_NONE: u8 = 0xFF; // <- d_a_ki.cpp:1409, :672
/// `if (i_this->mParameters == 0xFF) i_this->mParameters = 0;` (`:1375-1377`) - `mParameters` is
/// the `u8` field at `d_a_ki.h:29`, so the whole `u32` parameter is truncated.
const PARAM_NONE: u8 = 0xFF; // <- d_a_ki.cpp:1375
/// `i_this->m2CD = fopAcM_GetParam(a_this) >> 8 & 0x7f` (`:1380`).
const BAND_SHIFT: u32 = 8; // <- d_a_ki.cpp:1380
const BAND_MASK: u32 = 0x7F; // <- d_a_ki.cpp:1380
/// `i_this->mDamageType = (fopAcM_GetParam(a_this) >> 8) & 0x80` (`:1381`), stored in the `u8`
/// `mDamageType` (`d_a_ki.h:81`) as 0 or 0x80.
const BURN_BIT: u32 = 0x80; // <- d_a_ki.cpp:1381
/// `i_this->m2CF = fopAcM_GetParam(a_this) >> 0x18` (`:1383`).
const SWITCH_SHIFT: u32 = 0x18; // <- d_a_ki.cpp:1383

/// `switch (i_this->m2CD)` -> `mMaxAttackMoveDist300` (`:1390-1407`). `m2CD` is 0..127 and the
/// `default` arm catches everything from 3 up.
const MAX_ATTACK_DIST: [f32; 4] = [300.0, 800.0, 1500.0, 3000.0]; // <- d_a_ki.cpp:1392, :1396, :1400, :1405

/// `kiHIO_c` (`l_kiHIO`, `:25-52`) is a `JORReflexible` child whose `genMessage` is `UNUSED`
/// (`:55`), so every field keeps its constructor value.
const HIO_FIGHT_ROUNDS: i16 = 3; // <- d_a_ki.cpp:47 (m54)
const HIO_CROWD_LIMIT: i16 = 7; // <- d_a_ki.cpp:48 (m52)
const HIO_FIGHT_CHANCE: f32 = 0.2; // <- d_a_ki.cpp:45 (m48)
const HIO_CROWD_WAIT: i16 = 30; // <- d_a_ki.cpp:44 (m4C)
const HIO_TRACK: f32 = 30.0; // <- d_a_ki.cpp:37 (m28)
const HIO_TRACK_STEP: f32 = 2.0; // <- d_a_ki.cpp:514
const HIO_TRACK_TURN: f32 = 3000.0; // <- d_a_ki.cpp:390 (m304)
const HIO_ORBIT: f32 = 7.0; // <- d_a_ki.cpp:38 (m30)
const HIO_RIPPLE_SCALE: f32 = 3.0; // <- d_a_ki.cpp:37 (m2C, the max step of :572-574)
const HIO_KNOCK_BUMP: f32 = 65.0; // <- d_a_ki.cpp:40 (m34)
const HIO_KNOCK_SPLASH: f32 = 100.0; // <- d_a_ki.cpp:41 (m38)
const HIO_WALL_BUMP: f32 = 15.0; // <- d_a_ki.cpp:43 (m3C)
const HIO_DECAY: f32 = 7.0; // <- d_a_ki.cpp:42 (m40)
const HIO_DIVE: f32 = 30.0; // <- d_a_ki.cpp:39 (m44)
const HIO_DIVE_STEP: f32 = 3.0; // <- d_a_ki.cpp:545
const HIO_CO_RADIUS: f32 = 80.0; // <- d_a_ki.cpp:51 (m58)

/// `ki_atack_move`'s aim height above the player (`:516`, `:541`, `:638`) and its proximity gate
/// (`:520`).
const STRIKE_HEIGHT: f32 = 80.0; // <- d_a_ki.cpp:516
const STRIKE_RANGE: f32 = 300.0; // <- d_a_ki.cpp:520
/// The three wobble phase steps and amplitudes of behaviour 2 (`:568-570`) and behaviour 10
/// (`:642-644`).
const WOBBLE_STEP_X: i16 = 0x76C; // <- d_a_ki.cpp:568
const WOBBLE_STEP_Y: i16 = 0x9C4; // <- d_a_ki.cpp:569
const WOBBLE_STEP_Z: i16 = 0x5DC; // <- d_a_ki.cpp:570
const WOBBLE_NEAR: f32 = 50.0; // <- d_a_ki.cpp:568
const WOBBLE_NEAR_Y: f32 = 60.0; // <- d_a_ki.cpp:569
const WOBBLE_FAR: f32 = 100.0; // <- d_a_ki.cpp:642
/// The two orbit `cLib_addCalc2` scales and the two `mPosMoveDist` ramps.
const ORBIT_SCALE: f32 = 0.1; // <- d_a_ki.cpp:646
const ORBIT_RAMP: f32 = 0.025; // <- d_a_ki.cpp:649
const DIVE_SCALE: f32 = 0.1; // <- d_a_ki.cpp:572
const DIVE_RAMP: f32 = 0.05; // <- d_a_ki.cpp:575
/// `cLib_addCalc2(&i_this->mPosMoveDist, 1.0f, 1.0f, 0.04f)` (`:267`).
const DIST_RAMP: f32 = 0.04; // <- d_a_ki.cpp:267
/// `i_this->m304 = 10000.0f` (`:348`, `:654`) / `3000.0f` (`:390`) / `4000.0f` (`:929`).
const TURN_HOME: f32 = 10000.0; // <- d_a_ki.cpp:654
const TURN_FLY: f32 = 3000.0; // <- d_a_ki.cpp:390
const TURN_PATH: f32 = 4000.0; // <- d_a_ki.cpp:929

/// `ki_wait_move` case 10 (`:344-357`): the return-home run.
const HOME_SPEED: f32 = 20.0; // <- d_a_ki.cpp:346
const HOME_STEP: f32 = 20.0; // <- d_a_ki.cpp:347
const HOME_RANGE: f32 = 100.0; // <- d_a_ki.cpp:353
const HOME_HOLD: i16 = 0x32; // <- d_a_ki.cpp:355
const HOME_SCALE: f32 = 0.5; // <- d_a_ki.cpp:361
/// `cLib_addCalc2(&pos.x, home.pos.x, 0.5f, fabsf(speed.x) + 1.0f)` (`:361-363`).
const HOME_MIN_PAD: f32 = 1.0; // <- d_a_ki.cpp:361
/// `case 1: if (sqrtf(dx*dx + dz*dz) < mMaxAttackMoveDist300)` (`:296`, `:404`).
const APPROACH_SLOPE: i16 = 5; // <- d_a_ki.cpp:265 (cLib_addCalcAngleS2 scale)
/// `ki_wait_move` case 3's aim clamps (`:331-341`).
const YAW_CLAMP: i16 = 10000; // <- d_a_ki.cpp:332
const PITCH_CLAMP: i16 = 0x1FF6; // <- d_a_ki.cpp:338
const PITCH_FLOOR: i16 = -0x38E; // <- d_a_ki.cpp:339
/// `i_this->m314 = 3` (`:741`) is the damage-motion grace; the damage intake sets 10 (`:1093`)
/// and 0x32 on a fire arrow (`:1112`).
const GRACE_HIT: i16 = 10; // <- d_a_ki.cpp:1093
const GRACE_FIRE: i16 = 0x32; // <- d_a_ki.cpp:1112
/// `a_this->current.angle.x = 0x4000` (`:321`).
const PITCH_UP: i16 = 0x4000; // <- d_a_ki.cpp:321
/// `m580.SetR(0.0f); mDamageSphere.SetR(40.0f); m6AC.SetR(l_kiHIO.m58)` (`:995-997`) and
/// `m580.SetR(30.0f)` in attack behaviour 3 (`:580`).
const AT_RADIUS_IDLE: f32 = 0.0; // <- d_a_ki.cpp:995
const AT_RADIUS_STRIKE: f32 = 30.0; // <- d_a_ki.cpp:580
const TG_RADIUS: f32 = 40.0; // <- d_a_ki.cpp:996
/// `cLib_addCalc0(&scale.x, 1.0f, 0.05f)` and `if (scale.x <= 0.04f) fopAcM_delete` (`:965-969`).
const SHRINK_SCALE: f32 = 1.0; // <- d_a_ki.cpp:965
const SHRINK_MAX: f32 = 0.05; // <- d_a_ki.cpp:965
const SHRINK_MIN: f32 = 0.04; // <- d_a_ki.cpp:967
/// `current.pos.y -= REG0_F(5) + 30.0f; ... mAcch.CrrPos(...); current.pos.y += ...` (`:1080-1088`).
/// `REG0_F(5)` is a proven-zero HIO float register (RULE 2b).
const GROUND_OFFSET: f32 = 30.0; // <- d_a_ki.cpp:1080
/// `ki_fire_set_move`'s ballistic block (`:470-487`).
const FIRE_LIFT: f32 = 20.0; // <- d_a_ki.cpp:463
const FIRE_GRAVITY: f32 = 8.0; // <- d_a_ki.cpp:483
const FIRE_ABANDON: f32 = 2000.0; // <- d_a_ki.cpp:485

const SE_WAKEUP: u32 = 0x481C; // <- d_a_ki.cpp:310 (JA_SE_CV_KI_WAKEUP)
const SE_ATTACK: u32 = 0x481D; // <- d_a_ki.cpp:501 (JA_SE_CV_KI_ATTACK)
const SE_DAMAGE: u32 = 0x481E; // <- d_a_ki.cpp:1152 (JA_SE_CV_KI_DAMAGE)
const SE_TORCH: u32 = 0x6103; // <- d_a_ki.cpp:1044 (JA_SE_OBJ_TORCH_BURNING)

/// `fopAc_CULLBOX_0_e` (`d_a_ki.cpp:1528`) resolves through `l_cullSizeBox[0]`
/// (`f_op_actor_mng.cpp:559`).
const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:559 (fopAc_CULLBOX_0_e)
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:559 (fopAc_CULLBOX_0_e)

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ki {
    /// `mAction` (`d_a_ki.h:34`), seeded with `mParameters` at `:1379`.
    action: u8,
    /// `mBehaviorType` (`d_a_ki.h:36`), `s8` in retail so `0xFF` (`-1`) and `0xFF`-style sentinels
    /// wrap.
    behaviour: i8,
    /// `mParameters` (`d_a_ki.h:29`), the `u8` action seed.
    parameters: u8,
    /// `m2CD` (`d_a_ki.h:30`), the `mMaxAttackMoveDist300` selector.
    band: u8,
    /// `mKiPathIndex` (`d_a_ki.h:31`). Anything but `PATH_NONE` refuses creation outright.
    path_index: u8,
    /// `m2CF` (`d_a_ki.h:32`).
    switch_raw: u8,
    /// `m2D4` (`d_a_ki.h:37`), the standby switch index plus one; `0` means "live".
    standby: u8,
    /// `mMaxAttackMoveDist300` (`d_a_ki.h:0x2E4`).
    reach: f32,
    /// `mTimers[4]` (`d_a_ki.h:0x30C`).
    timers: [i16; 4],
    /// `m314` (`d_a_ki.h:0x314`), the damage grace.
    grace: i16,
    /// `m320` (`d_a_ki.h:0x320`), the wobble phase; `daKi_Execute` increments it every tick
    /// (`:1012`) and `create` seeds it with `cM_rndF(10000.0f)` (`:1433`).
    wobble: i16,
    /// `m31C` (`d_a_ki.h:0x31C`), the knockback speed.
    knock: f32,
    /// `mPosMove` (`d_a_ki.h:0x2E8`) and its three drivers (`:0x2F4`, `:0x2F8`, `:0x2FC`).
    goal: Vec3,
    goal_speed: f32,
    goal_step: f32,
    goal_ramp: f32,
    /// `m304` (`d_a_ki.h:0x304`), the turn-rate ceiling of `ki_pos_move`.
    turn: f32,
    /// `m540` (`d_a_ki.h:0x540`), the "flying, run the ground check" flag.
    flying: bool,
    /// `m338` (`d_a_ki.h:0x338`), the one-shot morph-play flag (`:1014-1018`); kept because it is
    /// a plain per-tick latch with no archive data behind it.
    morph_pending: bool,
    /// `m580`'s radius (`:995`, `:580`), the outbound AT volume.
    at_radius: f32,
    /// `m2DC` (`d_a_ki.h:0x2DC`), the burn-away flag.
    burning: bool,
    /// `scale.x` (`fopAc_ac_c`), the burn-away shrink.
    scale: f32,
}

impl Ki {
    /// `daKi_Create` (`:1282-1504`). Returns `None` for the `cPhs_ERROR_e` at `:1412` - the
    /// `mKiPathIndex != 0xFF` case, whose `dPath_GetRoomPath` (`d_path.h`) has no data in this
    /// checkout (RULE 3b), so such a keeth is refused in retail too.
    pub fn spawn(parameters: u32) -> Option<Self> {
        // `i_this->mParameters = fopAcM_GetParam(a_this); if (== 0xFF) = 0;` (:1374-1377).
        let raw = parameters as u8;
        let parameters = if raw == PARAM_NONE { 0 } else { raw };
        let band = ((parameters >> BAND_SHIFT) & BAND_MASK) as u8;
        let path_index = (parameters >> 16) as u8;
        if path_index != PATH_NONE {
            // `if (i_this->ppd == NULL) return cPhs_ERROR_e;` (:1411-1413).
            return None;
        }
        let switch_raw = (parameters >> SWITCH_SHIFT) as u8;
        Some(Self {
            // `i_this->mAction = i_this->mParameters;` (:1379)
            action: parameters,
            // `fopAcM_ct` (:1370) leaves `mBehaviorType` at 0.
            behaviour: 0,
            parameters,
            band,
            path_index,
            switch_raw,
            // `if (i_this->m2CF != 0xFF) i_this->m2D4 = i_this->m2CF + 1;` (:1418-1420)
            standby: if switch_raw == PARAM_NONE { 0 } else { switch_raw.wrapping_add(1) },
            reach: MAX_ATTACK_DIST[usize::from(band) % MAX_ATTACK_DIST.len()],
            timers: [0; 4],
            grace: 0,
            wobble: 0,
            knock: 0.0,
            goal: Vec3::ZERO,
            goal_speed: 0.0,
            goal_step: 0.0,
            goal_ramp: 0.0,
            turn: 0.0,
            flying: false,
            morph_pending: false,
            at_radius: AT_RADIUS_IDLE,
            burning: false,
            scale: 1.0,
        })
    }

    /// The health the spawn tuple must carry: `a_this->health = 1; a_this->max_health = 1;`
    /// (`:1431-1432`).
    pub fn hit_points() -> HitPoints {
        HitPoints(1)
    }

    /// `fopAc_CULLBOX_0_e` (`d_a_ki.cpp:1528`) -> `l_cullSizeBox[0]`
    /// (`f_op_actor_mng.cpp:559`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `dComIfGs_isSwitch(i_this->m2D4 - 1, fopAcM_GetRoomNo(&i_this->actor))` (`:974`). A
    /// non-zero `m2D4` keeps the keese parked until its switch is set; the arm that then runs is
    /// `:975-977` (`m2D4 = 0; speedF = l_kiHIO.m28; attention = LOCKON_BATTLE`).
    fn standby_open(&mut self, live: bool) -> bool {
        if self.standby == 0 {
            return true;
        }
        if !live {
            return false;
        }
        self.standby = 0;
        true
    }

    /// `cLib_addCalc2(f32* o_value, f32 target, f32 scale, f32 maxStep)` (`c_lib.cpp:56-67`).
    fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
        if *value != target {
            let step = (scale * (target - *value)).clamp(-max_step, max_step);
            *value += step;
        }
    }

    /// `cLib_addCalcAngleS2(s16* o_value, s16 target, s16 scale, s16 maxStep)` (`c_lib.cpp:192-201`).
    /// `ki_pos_move` passes `m304 * mPosMoveDist` - an `f32` product - where the decomp's
    /// signature takes an `s16`, so the value truncates every call. That is retail, not a bug, and
    /// it is reproduced here.
    fn add_calc_angle_s2(value: &mut i16, target: i16, scale: i16, max_step: i16) {
        let diff = target.wrapping_sub(*value);
        let step = diff / scale;
        if step > max_step {
            *value = value.wrapping_add(max_step);
        } else if step < -max_step {
            *value = value.wrapping_sub(max_step);
        } else {
            *value = value.wrapping_add(step);
        }
    }

    /// `ki_pos_move` (`:257-281`). `arg1 == 0` is the only value any caller passes in the
    /// reachable arms, and it is what turns the yaw/pitch chase on.
    ///
    /// The tail is `sp0C = (0, 0, speedF); cMtx_YrotS(calc_mtx, current.angle.y);
    /// cMtx_XrotM(calc_mtx, current.angle.x); MtxPosition(&sp0C, &actor.speed)` - the yaw is
    /// applied first and the pitch second, which is the same order `stream::calc_speed` uses for
    /// the unpitched case.
    fn pos_move(&mut self, actor: &mut Actor, turn: bool) -> Vec3 {
        if turn {
            let delta = self.goal - actor.pos;
            let yaw = super::stream::angle_s(delta.x.atan2(delta.z));
            let pitch = -super::stream::angle_s(delta.y.atan2((delta.x * delta.x + delta.z * delta.z).sqrt()));
            let max_step = (self.turn * self.goal_ramp) as i32 as i16;
            Self::add_calc_angle_s2(&mut actor.angle[1], yaw, APPROACH_SLOPE, max_step);
            Self::add_calc_angle_s2(&mut actor.angle[0], pitch, APPROACH_SLOPE, max_step);
            Self::add_calc2(&mut self.goal_ramp, 1.0, 1.0, DIST_RAMP);
        }
        Self::add_calc2(&mut self.goal_speed, self.goal_target(), 1.0, self.goal_step);
        let yaw = super::stream::from_s16(actor.angle[1]);
        let pitch = super::stream::from_s16(actor.angle[0]);
        let (sin_yaw, cos_yaw) = yaw.sin_cos();
        let (sin_pitch, cos_pitch) = pitch.sin_cos();
        let speed = Vec3::new(
            self.goal_speed * sin_yaw,
            -self.goal_speed * cos_yaw * sin_pitch,
            self.goal_speed * cos_yaw * cos_pitch,
        );
        actor.pos += speed;
        speed
    }

    /// `mPosMoveTarget` (`d_a_ki.h:0x2F4`) is read back here because every arm writes it before
    /// the common `ki_pos_move` tail at `:270`.
    fn goal_target(&self) -> f32 {
        self.goal_speed_target
    }

    /// `ki_wait_move` (`:284-372`). Only `case 0`'s fall-through is reachable: `case 1`'s body
    /// needs `daPy_py_c::old.pos` (`:297`) and `case 3`'s needs it again (`:317`).
    fn wait_move(&mut self, actor: &mut Actor) {
        match self.behaviour {
            0 => self.behaviour = 1,
            _ => {}
        }
        let _ = actor;
    }

    /// `ki_fly_move` (`:375-422`). `m540 = 1` (`:378`) then the `case 0` arm (`:384-390`); the
    /// `case 1` body needs `cM_rndF` (`:394`) and the player's per-tick delta (`:404`).
    fn fly_move(&mut self) {
        self.flying = true;
        if self.behaviour == 0 {
            self.behaviour = 1;
            self.goal_speed_target = HIO_TRACK;
            self.goal_step = HIO_TRACK_STEP;
            self.turn = TURN_FLY;
        }
    }

    /// `ki_fire_set_move` (`:425-488`). `m540 = 0` (`:429`); `case 0`'s launch velocity needs
    /// `cM_rndF` (`:433-434`) so only the `case 10` / `case 11` pair is transcribed, and the
    /// ballistic block at `:470-487` runs for every behaviour.
    fn fire_set_move(&mut self, actor: &mut Actor, health: &Health, player: Vec3, dist: f32) -> bool {
        self.flying = false;
        if self.behaviour == 10 {
            self.behaviour = 11;
            self.timers[1] = 100; // <- d_a_ki.cpp:459
        }
        if self.behaviour == 11 {
            self.flying = true;
            self.knock = FIRE_LIFT; // <- d_a_ki.cpp:463
            if self.timers[1] == 0 {
                self.burning = true; // <- d_a_ki.cpp:465
            }
        }
        // `sp24 = (0, 0, speedF); cMtx_YrotS(calc_mtx, current.angle.y); MtxPosition(...)`
        // (`:473-478`) - no pitch term here, unlike `ki_pos_move`.
        let yaw = super::stream::from_s16(actor.angle[1]);
        let (sin_yaw, cos_yaw) = yaw.sin_cos();
        let step = Vec3::new(self.goal_speed * sin_yaw, 0.0, self.goal_speed * cos_yaw);
        actor.pos.x += step.x;
        actor.pos.y += self.knock;
        actor.pos.z += step.z;
        self.knock -= FIRE_GRAVITY; // <- d_a_ki.cpp:483
        // `if (fopAcM_searchPlayerDistance(a_this) > 2000.0f && ki_player_bg_check(i_this))`
        // (`:485`); the second term needs `dComIfG_Bgsp()->LineCross` and
        // `daPy_py_c::checkGrabWear()`, so only the distance arm is kept.
        dist > FIRE_ABANDON || health.lives() == 0 && player == player
    }

    /// `ki_atack_move` (`:491-698`). `m540 = 1` (`:496`) and the `mTimers[3]` crowing guard
    /// (`:499-502`) run for every behaviour.
    fn attack_move(&mut self, actor: &mut Actor, player: Vec3, fighting: i16, events: &mut Vec<SimEvent>) {
        self.flying = true;
        if self.grace == 0 && self.behaviour <= 3 {
            events.push(SimEvent::Se(SE_ATTACK));
        }
        match self.behaviour {
            0 => {
                self.behaviour = 1;
                // `mTimers[0] = l_kiHIO.m50 + cM_rndF(fabsf(m4E - m50))` (:509) needs the global
                // RNG, which this sim has no model for; see OMITTED.
                self.timers[2] = 20; // <- d_a_ki.cpp:510
            }
            1 => {
                self.goal_speed_target = HIO_TRACK;
                self.goal_step = HIO_TRACK_STEP;
                self.goal = player + Vec3::new(0.0, STRIKE_HEIGHT, 0.0);
                if (self.goal - actor.pos).length() < STRIKE_RANGE {
                    self.behaviour = 10;
                    self.goal_ramp = 0.0;
                }
            }
            10 => {
                self.goal_speed_target = HIO_ORBIT;
                if fighting < HIO_CROWD_LIMIT {
                    self.behaviour = 2;
                    self.timers[2] = 50;
                }
                self.orbit(actor, WOBBLE_FAR, ORBIT_RAMP, HIO_ORBIT);
            }
            2 => {
                if self.timers[2] == 0 {
                    self.timers[2] = HIO_CROWD_WAIT;
                    let mut chance = HIO_FIGHT_CHANCE;
                    if fighting >= HIO_FIGHT_ROUNDS {
                        chance *= 0.5;
                        self.timers[2] <<= 1;
                    }
                    // `if (cM_rndF(1.0f) < tmp)` (:537) needs the global RNG; see OMITTED.
                }
                self.orbit(actor, WOBBLE_NEAR, DIVE_RAMP, HIO_RIPPLE_SCALE);
            }
            _ => {}
        }
        self.turn = TURN_HOME;
        self.goal_ramp = 1.0;
        self.pos_move(actor, true);
    }

    /// The shared tail of `ki_atack_move`'s behaviours 2 and 10: the sinusoid offset
    /// (`:568-570` / `:642-644`) followed by three `cLib_addCalc2` calls onto the wobbled goal
    /// (`:572-574` / `:646-648`) and the `mPosMoveDist` ramp (`:575` / `:649`).
    fn orbit(&mut self, actor: &mut Actor, reach: f32, ramp: f32, max_step: f32) {
        let phase = i32::from(self.wobble);
        let offset = Vec3::new(
            stream::sin_s(self.wobble.wrapping_mul(WOBBLE_STEP_X)) * reach,
            stream::sin_s(self.wobble.wrapping_mul(WOBBLE_STEP_Y)) * WIO_Y,
            stream::cos_s(self.wobble.wrapping_mul(WOBBLE_STEP_Z)) * reach,
        );
        let _ = phase;
        Self::add_calc2(&mut actor.pos.x, self.goal.x + offset.x, ORBIT_SCALE, max_step * self.goal_ramp);
        Self::add_calc2(&mut actor.pos.y, self.goal.y + offset.y, ORBIT_SCALE, max_step * self.goal_ramp);
        Self::add_calc2(&mut actor.pos.z, self.goal.z + offset.z, ORBIT_SCALE, max_step * self.goal_ramp);
        Self::add_calc2(&mut self.goal_ramp, 1.0, 1.0, ramp);
    }

    /// `ki_damage_move` (`:737-815`). `m540 = 1` and `m314 = 3` (`:740-741`).
    fn damage_move(&mut self, actor: &mut Actor, alive: bool) -> bool {
        self.flying = true;
        self.grace = 3;
        let mut spent = false;
        match self.behaviour {
            0 => self.behaviour = 1,
            1 => {
                if self.knock > 0.1 {
                    // `cLib_addCalcAngleS2(&shape_angle.y, m316, 1, 0x3000)` (:751) is a shape
                    // angle; `MtxPosition` of `(0, 0, -m31C)` through `m316` / `mRand2000`
                    // (`:752-757`) needs the hit position, which has no channel here.
                    Self::add_calc2(&mut self.knock, 0.0, 1.0, HIO_DECAY);
                    actor.speed_y = 0.0;
                } else {
                    self.action = ACT_ATTACK_MOVE_INDEX;
                    self.behaviour = 0;
                    spent = true;
                }
            }
            _ => {}
        }
        if spent && !alive {
            self.action = ACT_FAIL_MOVE;
            self.behaviour = 0;
        }
        spent
    }

    /// `ki_fail_move` (`:825-874`). Outside the `GanonK` stage name comparison (`:832`), which no
    /// stage-placed keese can satisfy, retail always runs
    /// `fopAcM_createDisappear(a_this, &current.pos, 5, daDisItem_IBALL_e, 0xFF); fopAcM_delete(a_this);`
    /// (`:870-872`). `true` means "the process ended".
    fn fail_move(&self) -> bool {
        true
    }

    /// `ki_check` (`:127-131`) + `ki_a_d_sub` (`:115-124`): count every live `fpcNm_KI_e` whose
    /// `mAction == ACT_ATTACK_MOVE_INDEX_e` and whose `mBehaviorType < 10`.
    fn fighting(count: usize) -> i16 {
        i16::try_from(count).unwrap_or(i16::MAX)
    }

    /// `daKi_Execute` (`:943-1173`), the parts whose inputs exist.
    fn execute(
        &mut self,
        actor: &mut Actor,
        player: Vec3,
        dist: f32,
        standing: bool,
        switches: &Switches,
        hp: i32,
        fighting: i16,
        events: &mut Vec<SimEvent>,
    ) -> bool {
        // `if (i_this->m2D4 != 0) { if (isSwitch(m2D4 - 1, room)) {...} else { return TRUE; } }`
        // (`:973-984`). `m2D4 - 1` is `m2CF`, the raw parameter byte.
        let index = self.switch_raw;
        if !self.standby_open(index == 0 || switches.test(index.wrapping_sub(1))) {
            return false;
        }
        // `if (i_this->m2DC != 0) { cLib_addCalc0(&scale.x, 1.0f, 0.05f); scale.y = scale.z =
        // scale.x; if (scale.x <= 0.04f) fopAcM_delete; }` (`:964-971`).
        if self.burning {
            Self::add_calc2(&mut self.scale, 1.0, SHRINK_SCALE, SHRINK_MAX);
            self.scale = 1.0 - (1.0 - self.scale).abs();
            if self.scale <= SHRINK_MIN {
                return true;
            }
        }
        // `m580.SetR(0.0f); mDamageSphere.SetR(40.0f); m6AC.SetR(l_kiHIO.m58)` (`:995-997`).
        self.at_radius = AT_RADIUS_IDLE;
        // `if (l_kiHIO.m06 == 0)` - `m06` is 0 by construction (`:27`) - so the machine runs.
        // `for (i = 0; i < 4; i++) if (mTimers[i] != 0) mTimers[i]--;` (`:1002-1006`) and
        // `if (m314 != 0) m314--;` (`:1008-1010`).
        for timer in &mut self.timers {
            if *timer != 0 {
                *timer -= 1;
            }
        }
        if self.grace != 0 {
            self.grace -= 1;
        }
        self.wobble = self.wobble.wrapping_add(1); // <- d_a_ki.cpp:1012
        self.morph_pending = true; // <- d_a_ki.cpp:1014-1018
        // The action dispatch (`:1020-1036`). `mCurrKiPathIndex != 0` (`ki_path_move`) is
        // unreachable: `create` only sets it when a path resolved (`:1414`).
        let dead = match self.action {
            ACT_WAIT_MOVE => {
                self.wait_move(actor);
                false
            }
            ACT_FLY_MOVE => {
                self.fly_move();
                false
            }
            ACT_FIRE_SET_MOVE => self.fire_set_move(actor, &Health::default(), player, dist),
            ACT_ATTACK_MOVE | ACT_ATTACK_MOVE_INDEX => {
                self.attack_move(actor, player, fighting, events);
                false
            }
            ACT_DAMAGE_MOVE => self.damage_move(actor, hp > 0),
            ACT_FAIL_MOVE => self.fail_move(),
            _ => false,
        };
        // `if (i_this->mDamageType != 0) { ...; if (mAction != ACT_FAIL_MOVE_e) { particle;
        // fopAcM_seStart(&actor, JA_SE_OBJ_TORCH_BURNING, 0); } }` (`:1040-1046`).
        if self.parameters & BURN_BIT_U8 != 0 && self.action != ACT_FAIL_MOVE {
            events.push(SimEvent::Se(SE_TORCH));
        }
        // `if (i_this->m540 != 0) { current.pos.y -= 30.0f; old.pos.y -= 30.0f; mAcch.CrrPos(...);
        // current.pos.y += 30.0f; old.pos.y += 30.0f; }` (`:1078-1089`). The ground half of
        // `CrrPos` is transcribed; its wall and roof responses are not - see OMITTED.
        if self.flying && standing {
            let probe = Vec3::new(actor.pos.x, actor.pos.y - GROUND_OFFSET, actor.pos.z);
            if let Some(floor) = ground::surface(standing, probe)
                && actor.pos.y < floor
            {
                actor.pos.y = floor + GROUND_OFFSET;
            }
        }
        dead
    }

    /// The `m580` contact test of `:603`. It reports a strike when the player is inside the AT
    /// sphere; the hit reaction itself (`mBehaviorType = 4`, `speedF = -15.0f`,
    /// `mTimers[1] = 30`) needs `ChkAtHit`, which the `cCcD` registry of RULE 3b cannot supply,
    /// so only the predicate is kept.
    fn strike(&self, player: Vec3) -> bool {
        self.at_radius > AT_RADIUS_IDLE
            && super::walk::within(self.here, player, self.at_radius)
    }

    /// `actor.current.pos`, refreshed each tick so `strike` has a centre.
    here: Vec3,
}
