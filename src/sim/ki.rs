//! Keese - `fpcNm_KI_e`, `d_a_ki.cpp`, stage strings `keeth` (`d_stage.cpp:779`) and `Fkeeth`
//! (`:780`). This closes the `fpcNm_KI_e` OBJNAME gap: the table binds both names to THIS
//! translation unit and no other module should claim them.
//!
//! (a) inbound damage: YES. `mDamageSphere` is built from `tg_sph_src` (`:1311-1338`), whose
//! `SrcObjTg Type` is `AT_TYPE_ALL & ~AT_TYPE_WATER & ~AT_TYPE_UNK20000 & ~AT_TYPE_UNK400000 &
//! ~AT_TYPE_LIGHT` (`:1318`) with `cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsEnemy_e` (`:1319`), and
//! `create` sets `health = 1; max_health = 1` (`:1431-1432`). `combat::HitPoints(1)` and
//! `enemy::Hurt` are attached, and `control` watches the health drop to raise
//! `ACT_DAMAGE_MOVE_e` (`:1092`, `:1149-1152`).
//!
//! (b) outbound damage: YES, from `ki_atack_move` behaviour 3 only, which opens `m580` from
//! radius 0 (`:995`) to 30.0f (`:580`). `at_sph_src` is `AT_TYPE_UNK800` (`:1287`) at `Atp = 1`
//! (`:1288`), or `Atp = 2` on the burning variant (`:586`). Behaviour 3 is entered from
//! behaviour 2 behind `cM_rndF(1.0f) < tmp` (`:537`), which this sim cannot evaluate, so the arm
//! is transcribed but never arms.
//!
//! Three of this actor's inputs do not exist here and are reported under OMITTED: the room `PATH`
//! blocks (`dPath_GetRoomPath`), `cM_rndF` (this sim has no global RNG - `ajav.rs` and
//! `agbsw0.rs` carry per-actor LCGs and `vibration::Vibration::rnd()` is the rumble channel's),
//! and the player's per-tick displacement (`daPy_py_c::old.pos`), which `ki_wait_move` and
//! `ki_fire_set_move` both test with `sp58.abs() > 1.0f`.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::combat::HitPoints;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::ground;
use super::health::Health;
use super::player::Player;
use super::stream;
use super::walk;

/// `d_stage.cpp:779-780`: both names bind to `fpcNm_KI_e`, the profile of
/// `d_a_ki.cpp:1514-1529`. The archive is `"Ki"` (`:1372`).
pub const KI_ACTORS: [&str; 2] = ["keeth", "Fkeeth"];

/// `ki_class::Action_e` (`d_a_ki.h:15-23`) with every numeric value.
const ACT_WAIT_MOVE: u8 = 0; // <- d_a_ki.h:16
const ACT_FLY_MOVE: u8 = 1; // <- d_a_ki.h:17
const ACT_FIRE_SET_MOVE: u8 = 2; // <- d_a_ki.h:18
const ACT_ATTACK_MOVE: u8 = 3; // <- d_a_ki.h:19
const ACT_ATTACK_MOVE_INDEX: u8 = 10; // <- d_a_ki.h:20
const ACT_DAMAGE_MOVE: u8 = 20; // <- d_a_ki.h:21
const ACT_FAIL_MOVE: u8 = 30; // <- d_a_ki.h:22

/// `i_this->mParameters = fopAcM_GetParam(a_this); if (== 0xFF) = 0;` (`:1374-1377`).
/// `mParameters` is the `u8` at `d_a_ki.h:29`, so the whole `u32` is truncated first.
const PARAM_NONE: u8 = 0xFF; // <- d_a_ki.cpp:1375
/// `i_this->mKiPathIndex = fopAcM_GetParam(a_this) >> 0x10` (`:1382`) lands in the `u8`
/// `mKiPathIndex` (`d_a_ki.h:31`); `!= 0xFF` is the path arm (`:1409`).
const PATH_NONE: u8 = 0xFF; // <- d_a_ki.cpp:1409
/// `i_this->m2CD = fopAcM_GetParam(a_this) >> 8 & 0x7f` (`:1380`).
const BAND_SHIFT: u32 = 8; // <- d_a_ki.cpp:1380
const BAND_MASK: u32 = 0x7F; // <- d_a_ki.cpp:1380
/// `i_this->m2CF = fopAcM_GetParam(a_this) >> 0x18` (`:1383`).
const SWITCH_SHIFT: u32 = 0x18; // <- d_a_ki.cpp:1383
/// `i_this->mDamageType = (fopAcM_GetParam(a_this) >> 8) & 0x80` (`:1381`), stored in the `u8`
/// `mDamageType` (`d_a_ki.h:81`).
const BURN_BIT: u8 = 0x80; // <- d_a_ki.cpp:1381
/// `i_this->m2CD` -> `mMaxAttackMoveDist300` (`:1390-1407`); the `default` arm catches 3 and up.
const MAX_ATTACK_DIST: [f32; 4] = [
    300.0,  // <- d_a_ki.cpp:1392
    800.0,  // <- d_a_ki.cpp:1396
    1500.0, // <- d_a_ki.cpp:1400
    3000.0, // <- d_a_ki.cpp:1405
];

/// `kiHIO_c` (`l_kiHIO`, `:25-52`) is a `JORReflexible` child whose `genMessage` is `UNUSED`
/// (`:55`), so every field keeps its constructor value, and `m06` is 0 (`:26`) - which is what
/// gates the whole action machine at `:999`.
const HIO_CROWD_LIMIT: i16 = 7; // <- d_a_ki.cpp:48 (m52)
const HIO_FIGHT_ROUNDS: i16 = 3; // <- d_a_ki.cpp:47 (m54)
const HIO_FIGHT_CHANCE: f32 = 0.2; // <- d_a_ki.cpp:45 (m48)
const HIO_CROWD_WAIT: i16 = 30; // <- d_a_ki.cpp:44 (m4C)
const HIO_TRACK: f32 = 30.0; // <- d_a_ki.cpp:37 (m28)
const HIO_TRACK_STEP: f32 = 2.0; // <- d_a_ki.cpp:514
const HIO_ORBIT: f32 = 7.0; // <- d_a_ki.cpp:38 (m30)
const HIO_RIPPLE: f32 = 3.0; // <- d_a_ki.cpp:37 (m2C)
const HIO_DECAY: f32 = 7.0; // <- d_a_ki.cpp:42 (m40)

/// The aim height above the player (`:516`) and the proximity gate (`:520`).
const STRIKE_HEIGHT: f32 = 80.0; // <- d_a_ki.cpp:516
const STRIKE_RANGE: f32 = 300.0; // <- d_a_ki.cpp:520
/// The three wobble phase steps (`:568-570`, `:642-644`) and the two amplitude pairs.
const WOBBLE_STEP_X: i16 = 0x76C; // <- d_a_ki.cpp:568
const WOBBLE_STEP_Y: i16 = 0x9C4; // <- d_a_ki.cpp:569
const WOBBLE_STEP_Z: i16 = 0x5DC; // <- d_a_ki.cpp:570
const WOBBLE_FAR: f32 = 100.0; // <- d_a_ki.cpp:642
const WOBBLE_NEAR: f32 = 50.0; // <- d_a_ki.cpp:568
const WOBBLE_LIFT: f32 = 60.0; // <- d_a_ki.cpp:569
/// The two orbit `cLib_addCalc2` scales and the two `mPosMoveDist` ramps.
const ORBIT_SCALE: f32 = 0.1; // <- d_a_ki.cpp:646
const ORBIT_RAMP: f32 = 0.025; // <- d_a_ki.cpp:649
const DIVE_RAMP: f32 = 0.05; // <- d_a_ki.cpp:575
/// `cLib_addCalc2(&i_this->mPosMoveDist, 1.0f, 1.0f, 0.04f)` (`:267`).
const DIST_RAMP: f32 = 0.04; // <- d_a_ki.cpp:267
/// `i_this->m304 = 10000.0f` (`:348`, `:654`), `3000.0f` (`:390`), `4000.0f` (`:929`).
const TURN_HOME: f32 = 10000.0; // <- d_a_ki.cpp:654
const TURN_FLY: f32 = 3000.0; // <- d_a_ki.cpp:390
/// `ki_fly_move`'s re-target gate (`:393`).
const FLY_CLOSE: f32 = 50.0; // <- d_a_ki.cpp:393
/// `ki_fire_set_move`'s ballistic block (`:459`, `:463`, `:483`).
const FIRE_HOLD: i16 = 100; // <- d_a_ki.cpp:459
const FIRE_LIFT: f32 = 20.0; // <- d_a_ki.cpp:463
const FIRE_GRAVITY: f32 = 8.0; // <- d_a_ki.cpp:483
/// `cLib_addCalcAngleS2`'s scale argument at every `ki_pos_move` call site.
const CHASE_SCALE: i16 = 5; // <- d_a_ki.cpp:265
/// `m580`'s radius (`:995`, `:580`) and the `Atp` values (`:584`, `:586`).
const AT_RADIUS_IDLE: f32 = 0.0; // <- d_a_ki.cpp:995
const AT_RADIUS_STRIKE: f32 = 30.0; // <- d_a_ki.cpp:580
const AT_POWER_PLAIN: u8 = 1; // <- d_a_ki.cpp:584
const AT_POWER_BURN: u8 = 2; // <- d_a_ki.cpp:586
/// The burn-away shrink (`:965-969`).
const SHRINK_SCALE: f32 = 1.0; // <- d_a_ki.cpp:965
const SHRINK_MAX: f32 = 0.05; // <- d_a_ki.cpp:965
const SHRINK_MIN: f32 = 0.04; // <- d_a_ki.cpp:967
/// `current.pos.y -= REG0_F(5) + 30.0f; ... current.pos.y += ...` (`:1080-1088`). `REG0_F(5)` is a
/// proven-zero HIO float register (RULE 2b).
const GROUND_OFFSET: f32 = 30.0; // <- d_a_ki.cpp:1080

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
    /// `mBehaviorType` (`d_a_ki.h:36`), an `s8` in retail.
    behaviour: i8,
    /// `mParameters` (`d_a_ki.h:29`), the `u8` action seed.
    parameters: u8,
    /// `m2CF` (`d_a_ki.h:32`), the raw standby switch number.
    switch_raw: u8,
    /// `m2D4` (`d_a_ki.h:37`), `m2CF + 1` while the keese waits; `0` means live.
    standby: u8,
    /// `mMaxAttackMoveDist300` (`d_a_ki.h:0x2E4`).
    reach: f32,
    /// `mTimers[4]` (`d_a_ki.h:0x30C`).
    timers: [i16; 4],
    /// `m314` (`d_a_ki.h:0x314`), the damage-intake grace.
    grace: i16,
    /// `m320` (`d_a_ki.h:0x320`), the wobble phase; `daKi_Execute` increments it every tick
    /// (`:1012`) and `create` seeds it with `cM_rndF(10000.0f)` (`:1433`).
    wobble: i16,
    /// `m31C` (`d_a_ki.h:0x31C`), the knockback speed. `fopAcM_ct` leaves it 0 and the intake that
    /// would set it branches on the hit type, which has no channel here - see OMITTED.
    knock: f32,
    /// `mPosMove` (`d_a_ki.h:0x2E8`) plus the three drivers `mPosMoveTarget` /
    /// `mPosMoveMaxSpeed` / `mPosMoveDist` (`:0x2F4`, `:0x2F8`, `:0x2FC`).
    goal: Vec3,
    goal_speed: f32,
    goal_target: f32,
    goal_max: f32,
    goal_ramp: f32,
    /// `m304` (`d_a_ki.h:0x304`), the turn ceiling of `ki_pos_move`.
    turn: f32,
    /// `home.pos` (`fopAc_ac_c`), the hover anchor of `ki_fly_move` (`:395`). The sim's `Actor`
    /// carries no `home`, and nothing else in this actor writes `current.pos` before the fly arm,
    /// so the stage placement is the home record.
    home: Vec3,
    /// `a_this->speed.y` (`fopAc_ac_c`), the burning keese's vertical speed (`:463`, `:481`,
    /// `:483`).
    lift: f32,
    /// `m540` (`d_a_ki.h:0x540`), the "flying, run the ground check" flag.
    flying: bool,
    /// `m2DC` (`d_a_ki.h:0x2DC`), the burn-away flag.
    burning: bool,
    /// `scale.x`, the burn-away shrink.
    scale: f32,
    /// `m580`'s radius (`:995`, `:580`), the outbound AT volume.
    at_radius: f32,
    /// `m580`'s `Atp` (`:584`, `:586`), the outbound attack power.
    at_power: u8,
    /// The last `HitPoints` value seen, so `control` can spot the drop retail's
    /// `mDamageSphere.ChkTgHit()` reports (`:1092`).
    seen_hp: i32,
    /// The `cPhs_ERROR_e` of `:1412`. Set here because the refusal needs the parameter decode,
    /// and discharged on the first tick - RULE 6b forbids the `Option<Self> -> None` wiring for a
    /// `cPhs_ERROR_e`, which would leave a bare `Actor` retail never keeps.
    refused: bool,
}

impl Ki {
    /// `daKi_Create` (`:1282-1504`).
    ///
    /// A `mKiPathIndex` other than `0xFF` asks `dPath_GetRoomPath` for a room path and returns
    /// `cPhs_ERROR_e` when it comes back NULL (`:1409-1413`). With no path data in this checkout
    /// (RULE 3b) that NULL is the only outcome, so such a keese is refused in retail too; the
    /// refusal is carried on the component and dropped on the first tick rather than being hidden
    /// in the return type, per RULE 6b.
    pub fn spawn(parameters: u32, pos: Vec3) -> Self {
        let raw = parameters as u8;
        let parameters = if raw == PARAM_NONE { 0 } else { raw };
        let whole = u32::from(parameters);
        let band = ((whole >> BAND_SHIFT) & BAND_MASK) as u8;
        let switch_raw = (whole >> SWITCH_SHIFT) as u8;
        Self {
            // `i_this->mAction = i_this->mParameters;` (`:1379`)
            action: parameters,
            // `fopAcM_ct` (`:1370`) leaves `mBehaviorType` at 0.
            behaviour: 0,
            parameters,
            switch_raw,
            // `if (i_this->m2CF != 0xFF) i_this->m2D4 = i_this->m2CF + 1;` (`:1418-1420`)
            standby: if switch_raw == PARAM_NONE {
                0
            } else {
                switch_raw.wrapping_add(1)
            },
            reach: MAX_ATTACK_DIST[usize::from(band) % MAX_ATTACK_DIST.len()],
            timers: [0; 4],
            grace: 0,
            wobble: 0,
            knock: 0.0,
            goal: Vec3::ZERO,
            goal_speed: 0.0,
            goal_target: 0.0,
            goal_max: 0.0,
            goal_ramp: 0.0,
            turn: 0.0,
            home: pos,
            lift: 0.0,
            flying: false,
            burning: false,
            scale: 1.0,
            at_radius: AT_RADIUS_IDLE,
            at_power: AT_POWER_PLAIN,
            // `a_this->health = 1;` (`:1431`)
            seen_hp: 1,
            // `if (i_this->ppd == NULL) return cPhs_ERROR_e;` (`:1411-1413`)
            refused: (whole >> 16) as u8 != PATH_NONE,
        }
    }

    /// `fopAc_CULLBOX_0_e` (`d_a_ki.cpp:1528`) -> `l_cullSizeBox[0]` (`f_op_actor_mng.cpp:559`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `ki_check` (`:127-131`) + `ki_a_d_sub` (`:115-124`): every live `fpcNm_KI_e` whose
    /// `mAction == ACT_ATTACK_MOVE_INDEX_e` and whose `mBehaviorType < 10`.
    fn fighting(roster: &[(u8, i8)]) -> i16 {
        let count = roster
            .iter()
            .filter(|(action, behaviour)| *action == ACT_ATTACK_MOVE_INDEX && *behaviour < 10)
            .count();
        i16::try_from(count).unwrap_or(i16::MAX)
    }

    /// `cLib_addCalc2(f32* o_value, f32 target, f32 scale, f32 maxStep)` (`c_lib.cpp:56-67`).
    fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
        if *value != target {
            let step = (scale * (target - *value)).clamp(-max_step, max_step);
            *value += step;
        }
    }

    /// `cLib_addCalc0(f32* o_value, f32 scale, f32 maxStep)` (`c_lib.cpp:69-77`) - it subtracts.
    fn add_calc0(value: &mut f32, scale: f32, max_step: f32) {
        let step = (*value * scale).clamp(-max_step, max_step);
        *value -= step;
    }

    /// `cLib_addCalcAngleS2(s16* o_value, s16 target, s16 scale, s16 maxStep)`
    /// (`c_lib.cpp:192-201`).
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

    /// `ki_pos_move` (`:257-281`). `turn` is the `arg1 == 0` test at `:258`.
    ///
    /// The tail is `sp0C = (0, 0, speedF); cMtx_YrotS(calc_mtx, current.angle.y);
    /// cMtx_XrotM(calc_mtx, current.angle.x); MtxPosition(&sp0C, &actor.speed)` (`:271-281`): the
    /// yaw is applied first and the pitch second, and `MtxPosition` copies the three components
    /// onto `speed`, which is then added to `current.pos` (`:278-280`).
    fn pos_move(&mut self, actor: &mut Actor, turn: bool) {
        if turn {
            let delta = self.goal - actor.pos;
            let flat = (delta.x * delta.x + delta.z * delta.z).sqrt();
            // `cM_atan2s(x, z)` and `-cM_atan2s(y, sqrtf(x * x + z * z))` (`:262-263`).
            let yaw = stream::angle_s(delta.x.atan2(delta.z));
            let pitch = -stream::angle_s(delta.y.atan2(flat));
            // `i_this->m304 * i_this->mPosMoveDist` is an `f32` product handed to an `s16`
            // parameter (`:265-266`), so it truncates on every call. That is retail.
            let max_step = (self.turn * self.goal_ramp) as i32 as i16;
            Self::add_calc_angle_s2(&mut actor.angle[1], yaw, CHASE_SCALE, max_step);
            Self::add_calc_angle_s2(&mut actor.angle[0], pitch, CHASE_SCALE, max_step);
            Self::add_calc2(&mut self.goal_ramp, 1.0, 1.0, DIST_RAMP);
        }
        Self::add_calc2(&mut self.goal_speed, self.goal_target, 1.0, self.goal_max);
        let yaw = walk::from_s16(actor.angle[1]);
        let pitch = walk::from_s16(actor.angle[0]);
        let (sin_yaw, cos_yaw) = yaw.sin_cos();
        let (sin_pitch, cos_pitch) = pitch.sin_cos();
        actor.pos += Vec3::new(
            self.goal_speed * sin_yaw,
            -self.goal_speed * cos_yaw * sin_pitch,
            self.goal_speed * cos_yaw * cos_pitch,
        );
    }

    /// `ki_wait_move` (`:284-372`). Only `case 0`'s fall-through and `case 1`'s reach test are
    /// reachable: `case 1`'s body needs `daPy_py_c::old.pos` (`:297-299`), and `case 3`'s needs it
    /// again (`:317`).
    fn wait_move(&mut self, here: Vec3, player: Vec3) {
        match self.behaviour {
            0 => self.behaviour = 1,
            1 => {
                let delta = player - here;
                // `sqrtf(sp58.x * sp58.x + sp58.z * sp58.z) < mMaxAttackMoveDist300` (`:296`).
                let _ = (delta.x * delta.x + delta.z * delta.z).sqrt() < self.reach;
            }
            _ => {}
        }
    }

    /// `ki_fly_move` (`:375-422`). `m540 = 1` (`:378`), the `case 0` arm (`:384-390`) and the
    /// `case 1` re-target gate (`:393`). The re-target's own body (`:394-400`) draws three
    /// `cM_rndF` / `cM_rndFX` values (`:396-398`) and is omitted, so the hover point stays where
    /// `case 0` left it; the closing-distance wake (`:402-408`) and the wall bounce
    /// (`:412-417`, which needs `mAcch.ChkWallHit()`) are handled as noted.
    fn fly_move(&mut self, actor: &mut Actor, player: Vec3) {
        self.flying = true;
        if self.behaviour == 0 {
            self.behaviour = 1;
            self.goal = self.home;
            self.goal_target = HIO_TRACK;
            self.goal_max = HIO_TRACK_STEP;
            self.turn = TURN_FLY;
            self.goal_ramp = 0.0;
        }
        let delta = self.goal - actor.pos;
        if self.timers[1] == 0 || (delta.x * delta.x + delta.z * delta.z).sqrt() < FLY_CLOSE {
            self.goal = self.home;
        }
        if self.timers[0] == 0 {
            let to_player = player - actor.pos;
            if (to_player.x * to_player.x + to_player.z * to_player.z).sqrt() < self.reach {
                self.action = ACT_ATTACK_MOVE_INDEX;
                self.behaviour = 0;
            }
        }
        self.pos_move(actor, true);
    }

    /// `ki_fire_set_move` (`:425-488`). `m540 = 0` (`:429`). The `case 0` launch velocity is
    /// `REG8_F(8) + 35.0f + cM_rndF(10.0f)` (`:433`), which needs the global RNG, so only the
    /// `case 10` / `case 11` pair and the ballistic block (`:470-487`) are transcribed.
    fn fire_set_move(&mut self, actor: &mut Actor) {
        self.flying = false;
        if self.behaviour == 10 {
            self.behaviour = 11;
            self.timers[1] = FIRE_HOLD;
        }
        if self.behaviour == 11 {
            self.flying = true;
            self.lift = FIRE_LIFT;
            if self.timers[1] == 0 {
                self.burning = true;
            }
        }
        // `sp24 = (0, 0, speedF); cMtx_YrotS(calc_mtx, current.angle.y); MtxPosition(&sp24,
        // &sp18)` (`:473-482`) - no pitch term here, unlike `ki_pos_move`.
        let yaw = walk::from_s16(actor.angle[1]);
        let (sin_yaw, cos_yaw) = yaw.sin_cos();
        actor.pos.x += self.goal_speed * sin_yaw;
        actor.pos.y += self.lift;
        actor.pos.z += self.goal_speed * cos_yaw;
        self.lift -= FIRE_GRAVITY;
        // `if (fopAcM_searchPlayerDistance(a_this) > 2000.0f && ki_player_bg_check(i_this))
        // m2DC = 1;` (`:485-487`). The second term needs `dComIfG_Bgsp()->LineCross` and
        // `daPy_py_c::checkGrabWear()`; firing the burn-away on the distance arm alone would
        // trigger it from across the room, which is not what retail does, so neither is applied.
    }

    /// `ki_atack_move` (`:491-698`). `m540 = 1` (`:496`); `cVar6` starts at 0 (`:497`).
    fn attack_move(&mut self, actor: &mut Actor, player: Vec3, fighting: i16, se: &mut Vec<u32>) {
        self.flying = true;
        // `if (mTimers[3] == 0 && mBehaviorType <= 3) { mTimers[3] = cM_rndF(14.0f) + 15.0f;
        // fopAcM_monsSeStart(a_this, JA_SE_CV_KI_ATTACK, 0); }` (`:499-502`). The re-arm needs the
        // global RNG, so only the sound fires, and it fires every tick the behaviour is <= 3.
        if self.behaviour <= 3 {
            se.push(SE_ATTACK);
        }
        let mut c_var6 = 0i8;
        match self.behaviour {
            0 => {
                self.behaviour = 1;
                // `mTimers[0] = l_kiHIO.m50 + cM_rndF(fabsf(m4E - m50))` (`:509`) needs the
                // global RNG - see OMITTED.
                self.timers[2] = 20;
            }
            1 => {
                self.goal_target = HIO_TRACK;
                self.goal_max = HIO_TRACK_STEP;
                self.goal = player + Vec3::new(0.0, STRIKE_HEIGHT, 0.0);
                // `cLib_addCalcAngleS2(&a_this->shape_angle.y, a_this->current.angle.y, 2,
                // 0x1000)` (`:519`) is a shape angle and is dropped.
                if (self.goal - actor.pos).length() < STRIKE_RANGE {
                    self.behaviour = 10;
                    self.goal_ramp = 0.0;
                }
            }
            10 => {
                self.goal_target = HIO_ORBIT;
                // `cLib_addCalcAngleS2(&a_this->shape_angle.y, searchPlayerAngleY, 2, 0x1000)`
                // (`:629`) is a shape angle.
                if fighting < HIO_CROWD_LIMIT {
                    self.behaviour = 2;
                    self.timers[2] = 50;
                }
                self.orbit(actor, WOBBLE_FAR, HIO_ORBIT, ORBIT_RAMP);
                c_var6 = -1;
            }
            2 => {
                if self.timers[2] == 0 {
                    self.timers[2] = HIO_CROWD_WAIT;
                    let mut chance = HIO_FIGHT_CHANCE;
                    if fighting >= HIO_FIGHT_ROUNDS {
                        chance *= 0.5;
                        self.timers[2] <<= 1;
                    }
                    // `if (cM_rndF(1.0f) < tmp) { mBehaviorType = 3; anm_init(...); ... break; }`
                    // (`:537-548`) needs the global RNG - see OMITTED.
                    let _ = chance;
                }
                // `cLib_addCalcAngleS2(&a_this->shape_angle.y, searchPlayerAngleY, 2, 0x1000)`
                // (`:551`) is a shape angle.
                self.orbit(actor, WOBBLE_NEAR, HIO_RIPPLE, DIVE_RAMP);
                c_var6 = -1;
            }
            3 => {
                // `m580.SetR(30.0f); mDamageSphere.SetR(20.0f); m6AC.SetR(0.0f);` (`:580-582`).
                self.at_radius = AT_RADIUS_STRIKE;
                self.at_power = if self.parameters & BURN_BIT == 0 {
                    AT_POWER_PLAIN
                } else {
                    AT_POWER_BURN
                };
                c_var6 = 1;
                if self.timers[1] == 10 {
                    self.goal_target = 0.0;
                    self.goal_max = 3.0;
                }
                if self.timers[1] == 0 {
                    self.behaviour = 1;
                }
                // `if (m580.ChkAtHit() || mDamageSphere.ChkCoHit()) { anm_init(...);
                // mBehaviorType = 4; mTimers[1] = 30; speedF = -15.0f; mPosMoveTarget = 0.0f;
                // mPosMoveMaxSpeed = 1.0f; }` (`:603-610`) needs the `cCcD` registry, which
                // RULE 3b lists as absent.
            }
            _ => {}
        }
        self.turn = TURN_HOME;
        self.goal_ramp = 1.0;
        if c_var6 >= 0 {
            self.pos_move(actor, c_var6 == 0);
        }
    }

    /// The tail shared by `ki_atack_move`'s behaviours 2 and 10: the sinusoid offset (`:568-570`
    /// / `:642-644`), three `cLib_addCalc2` calls onto the wobbled goal (`:572-574` / `:646-648`)
    /// and the `mPosMoveDist` ramp (`:575` / `:649`). `max_step` is `l_kiHIO.m2C` in behaviour 2
    /// and `l_kiHIO.m30` in behaviour 10.
    fn orbit(&mut self, actor: &mut Actor, reach: f32, max_step: f32, ramp: f32) {
        let offset = Vec3::new(
            stream::sin_s(self.wobble.wrapping_mul(WOBBLE_STEP_X)) * reach,
            stream::sin_s(self.wobble.wrapping_mul(WOBBLE_STEP_Y)) * WOBBLE_LIFT,
            stream::cos_s(self.wobble.wrapping_mul(WOBBLE_STEP_Z)) * reach,
        );
        let cap = max_step * self.goal_ramp;
        Self::add_calc2(&mut actor.pos.x, self.goal.x + offset.x, ORBIT_SCALE, cap);
        Self::add_calc2(&mut actor.pos.y, self.goal.y + offset.y, ORBIT_SCALE, cap);
        Self::add_calc2(&mut actor.pos.z, self.goal.z + offset.z, ORBIT_SCALE, cap);
        Self::add_calc2(&mut self.goal_ramp, 1.0, 1.0, ramp);
    }

    /// `ki_damage_move` (`:737-815`). `m540 = 1` and `m314 = 3` (`:740-741`).
    fn damage_move(&mut self, alive: bool) {
        self.flying = true;
        self.grace = 3;
        let mut spent = false;
        match self.behaviour {
            0 => self.behaviour = 1,
            1 => {
                if self.knock > 0.1 {
                    // `cLib_addCalc0(&i_this->m31C, 1.0f, l_kiHIO.m40)` (`:783`).
                    Self::add_calc0(&mut self.knock, 1.0, HIO_DECAY);
                } else {
                    self.action = ACT_ATTACK_MOVE_INDEX;
                    self.behaviour = 0;
                    spent = true;
                }
            }
            _ => {}
        }
        // `if (bVar1 && i_this->actor.health <= 0) { mAction = ACT_FAIL_MOVE_e;
        // mBehaviorType = 0; }` (`:811-814`).
        if spent && !alive {
            self.action = ACT_FAIL_MOVE;
            self.behaviour = 0;
        }
    }

    /// The `m580` outbound test. `at_sph_src` is a sphere centred on `current.pos` (`:1070`,
    /// `:1074`), so this is a point-in-sphere against the player. The strike reaction itself
    /// (`mBehaviorType = 4` at `:605`) needs `ChkAtHit`.
    fn strike(&self, here: Vec3, player: Vec3) -> bool {
        self.at_radius > AT_RADIUS_IDLE && walk::within(here, player, self.at_radius)
    }

    /// `daKi_Execute` (`:943-1173`), the parts whose inputs exist. `true` means the process ended.
    fn execute(
        &mut self,
        actor: &mut Actor,
        solid: Option<&Collision>,
        switches: &Switches,
        player: Vec3,
        roster: &[(u8, i8)],
        hp: i32,
        se: &mut Vec<u32>,
    ) -> bool {
        // `if (i_this->m2D4 != 0) { if (isSwitch(m2D4 - 1, room)) { m2D4 = 0;
        // speedF = l_kiHIO.m28; ... } else { ...; return TRUE; } }` (`:973-984`). `m2D4 - 1` is
        // `m2CF`.
        if self.standby != 0 {
            if !switches.test(self.switch_raw) {
                return false;
            }
            self.standby = 0;
            self.goal_speed = HIO_TRACK;
        }
        // `if (i_this->m2DC != 0) { cLib_addCalc0(&scale.x, 1.0f, 0.05f); scale.y = scale.z =
        // scale.x; if (scale.x <= 0.04f) { fopAcM_delete; return TRUE; } }` (`:964-971`).
        if self.burning {
            Self::add_calc0(&mut self.scale, SHRINK_SCALE, SHRINK_MAX);
            if self.scale <= SHRINK_MIN {
                return true;
            }
        }
        // `m580.SetR(0.0f); mDamageSphere.SetR(40.0f); m6AC.SetR(l_kiHIO.m58)` (`:995-997`).
        self.at_radius = AT_RADIUS_IDLE;
        // `if (l_kiHIO.m06 == 0)` - 0 by construction (`:26`) - so the machine runs:
        // `for (i = 0; i < 4; i++) if (mTimers[i] != 0) mTimers[i]--;` (`:1002-1006`) and
        // `if (m314 != 0) m314--;` (`:1008-1010`).
        for timer in self.timers.iter_mut() {
            if *timer != 0 {
                *timer -= 1;
            }
        }
        if self.grace != 0 {
            self.grace -= 1;
        }
        // `i_this->m320++;` (`:1012`).
        self.wobble = self.wobble.wrapping_add(1);
        // `if (mDamageSphere.ChkTgHit() && m314 == 0) { m314 = 10; ... }` (`:1092-1153`). The hit
        // TYPE branches four ways (`:1098-1145`) and has no channel here, so only the arm that
        // does not depend on it is kept: `m316 = searchPlayerAngleY(...); mAction =
        // ACT_DAMAGE_MOVE_e; mBehaviorType = 0; fopAcM_monsSeStart(JA_SE_CV_KI_DAMAGE)`
        // (`:1147-1152`).
        if hp < self.seen_hp && self.grace == 0 {
            self.grace = 10;
            self.action = ACT_DAMAGE_MOVE;
            self.behaviour = 0;
            se.push(SE_DAMAGE);
        }
        self.seen_hp = hp;
        // The action dispatch (`:1020-1036`). `mCurrKiPathIndex != 0` (`ki_path_move`) is
        // unreachable: `create` only sets it when a path resolved (`:1414`).
        match self.action {
            ACT_WAIT_MOVE => self.wait_move(actor.pos, player),
            ACT_FLY_MOVE => self.fly_move(actor, player),
            ACT_FIRE_SET_MOVE => self.fire_set_move(actor),
            ACT_ATTACK_MOVE | ACT_ATTACK_MOVE_INDEX => {
                self.attack_move(actor, player, Self::fighting(roster), se)
            }
            ACT_DAMAGE_MOVE => self.damage_move(hp > 0),
            ACT_FAIL_MOVE => return true,
            _ => {}
        }
        // `if (i_this->mDamageType != 0) { ...; if (mAction != ACT_FAIL_MOVE_e) { particle;
        // fopAcM_seStart(&actor, JA_SE_OBJ_TORCH_BURNING, 0); } }` (`:1040-1046`).
        if self.parameters & BURN_BIT != 0 && self.action != ACT_FAIL_MOVE {
            se.push(SE_TORCH);
        }
        // `if (i_this->m540 != 0) { current.pos.y -= 30.0f; old.pos.y -= 30.0f;
        // mAcch.CrrPos(*dComIfG_Bgsp()); current.pos.y += 30.0f; old.pos.y += 30.0f; }` (`:1078-1089`).
        // Only the ground response of `CrrPos` is transcribed; its wall and roof responses are not.
        if self.flying {
            let probe = Vec3::new(actor.pos.x, actor.pos.y - GROUND_OFFSET, actor.pos.z);
            if let Some(floor) = ground::surface(solid, probe)
                && actor.pos.y < floor
            {
                actor.pos.y = floor + GROUND_OFFSET;
            }
        }
        false
    }
}

pub fn is_ki(name: &str) -> bool {
    KI_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    switches: Res<Switches>,
    players: Query<&Player>,
    mut health: ResMut<Health>,
    mut events: ResMut<SimEvents>,
    // `Without<Player>` is the disjoint half of the `With`/`Without` pair RULE 9 requires: this
    // query writes `Actor` for the keese, `players` reads `Player`, and no entity is both.
    mut kest: Query<(Entity, &mut Actor, &mut Ki, &HitPoints), Without<Player>>,
    mut commands: Commands,
) {
    let Some(player) = players.iter().next() else {
        return;
    };
    let solid = collision.as_deref();
    // `ki_check` (`:127-131`) scans every live keese in one pass, before any of them acts, so the
    // roster is buffered before the mutating loop.
    let mut roster: Vec<(u8, i8)> = Vec::new();
    for (_, _, ki, _) in &mut kest {
        roster.push((ki.action, ki.behaviour));
    }
    let mut spent: Vec<Entity> = Vec::new();
    let mut se: Vec<u32> = Vec::new();
    for (entity, mut actor, mut ki, hit_points) in &mut kest {
        let actor = &mut *actor;
        let ki = &mut *ki;
        // `daKi_Create` returned `cPhs_ERROR_e` during Create, so `f_pc_create_req.cpp:106-108`
        // cancelled the process before Execute ever ran.
        if ki.refused {
            spent.push(entity);
            continue;
        }
        if ki.execute(
            actor,
            solid,
            &switches,
            player.pos,
            &roster,
            hit_points.0,
            &mut se,
        ) {
            spent.push(entity);
            continue;
        }
        if ki.strike(actor.pos, player.pos) {
            health.damage(ki.at_power);
        }
    }
    for id in se {
        events.list.push(SimEvent::Se(id));
    }
    for entity in spent {
        let _ = commands.entity(entity).despawn();
    }
}
