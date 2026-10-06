use bevy_ecs::prelude::*;
use glam::{Mat4, Quat, Vec3};

use super::actor::Actor;
use super::stream;
use super::walk;

/// `OBJNAME("Kryu00", fpcNm_Obj_Drift_e, 255, 0)` (`d_stage.cpp:894`) is the only row bound to
/// `fpcNm_Obj_Drift_e` (`d_a_obj_drift.cpp:448`). The arcname is `"Kryu"`
/// (`d_a_obj_drift.cpp:63`) - NOT the stage string, which is `Kryu00`.
pub const DRIFT_ACTORS: [&str; 1] = ["Kryu00"];

/// `daObjDrift::Act_c::Prm_e { PRM_TYPE_W = 3, PRM_TYPE_S = 0 }` (`d_a_obj_drift.h:18-21`), fed to
/// `daObj::PrmAbstract(this, W, S)` = `((1 << W) - 1) & (param >> S)` (`d_a_obj.h:14`) by
/// `prm_get_type()` (`d_a_obj_drift.h:23-25`).
pub const TYPE_MASK: u32 = 0x7; // <- d_a_obj_drift.h:19 (PRM_TYPE_W = 3)
pub const TYPE_SHIFT: u32 = 0; // <- d_a_obj_drift.h:20 (PRM_TYPE_S = 0)

/// `daObjDrift::Act_c::Mode` (`d_a_obj_drift.h:13-16`) - the only two labels, and the only two
/// entries of the `mode_proc` dispatch table (`d_a_obj_drift.cpp:384-387`).
pub const MODE_WAIT: u8 = 0x0; // <- d_a_obj_drift.h:14
pub const MODE_ROT: u8 = 0x1; // <- d_a_obj_drift.h:15

/// `fopAcM_setCullSizeBox(this, -550.0f, -180.0f, -550.0f, 550.0f, 605.0f, 550.0f)`
/// (`d_a_obj_drift.cpp:106`). The profile's `fopAc_CULLBOX_CUSTOM_e` (`:458`) resolves here and this
/// file calls no other box setter.
const CULL_MIN: [f32; 3] = [-550.0, -180.0, -550.0]; // <- d_a_obj_drift.cpp:106
const CULL_MAX: [f32; 3] = [550.0, 605.0, 550.0]; // <- d_a_obj_drift.cpp:106

/// The whole of `daObjDrift::L_attr` (`d_a_obj_drift.cpp:36-57`), one constant per member, in
/// declaration order. Only the members with a modelled reader are listed; see the report for
/// `mWaveHeight`, `mRideYOff`, `mWavePhaseXStep`, `field_0x16`, `mWaveSinScale`, `field_0x1C`,
/// `mTiltChaseWait`, `mTiltChaseRot`, `mPosMoveDamp`, `mWavePhaseYStep`, `mWavePhaseZStep`,
/// `mTiltSinScale`, `field_0x34`, `field_0x38`, `field_0x3A`, `mRotTimerMax` and `field_0x3E`.
///
/// `mWaveAmplitude` = 0.02 (`:40`) - the gain on the vertical error term of `gravity`.
pub const WAVE_AMPLITUDE: f32 = 0.02; // <- d_a_obj_drift.cpp:40
/// `mWaveSpeed` = 0.04 (`:38`) - `posMoveF_stream`'s `param_4` (`:348`).
pub const WAVE_SPEED: f32 = 0.04; // <- d_a_obj_drift.cpp:38
/// `mWavePhase` = 0.01 (`:39`) - `posMoveF_stream`'s `param_5` (`:348`).
pub const WAVE_PHASE: f32 = 0.01; // <- d_a_obj_drift.cpp:39
/// `mWavePhaseXStep` = 400 (`:42`) - the vertical bob's phase increment, an `s16` that wraps.
pub const WAVE_PHASE_X_STEP: i16 = 400; // <- d_a_obj_drift.cpp:42
/// `mWaveSinScale` = 0.3 (`:44`) - the amplitude of the `cM_ssin` term added to `gravity`.
pub const WAVE_SIN_SCALE: f32 = 0.3; // <- d_a_obj_drift.cpp:44
/// `mTiltChaseWait` = 0.8 (`:46`) - the tilt chase factor while `mMode != MODE_ROT`.
pub const TILT_CHASE_WAIT: f32 = 0.8; // <- d_a_obj_drift.cpp:46
/// `mTiltChaseRot` = 0.5 (`:47`) - the tilt chase factor while `mMode == MODE_ROT`.
pub const TILT_CHASE_ROT: f32 = 0.5; // <- d_a_obj_drift.cpp:47
/// `mPosMoveDamp` = 0.07 (`:48`) - the multiplicative damping on `mTiltVelZ` / `mTiltVelX`.
pub const TILT_DAMP: f32 = 0.07; // <- d_a_obj_drift.cpp:48
/// `mWavePhaseYStep` = 1000 (`:49`) - the z-sway phase increment, an `s16` that wraps.
pub const WAVE_PHASE_Y_STEP: i16 = 1000; // <- d_a_obj_drift.cpp:49
/// `mWavePhaseZStep` = 1111 (`:50`) - the x-sway phase increment, an `s16` that wraps.
pub const WAVE_PHASE_Z_STEP: i16 = 1111; // <- d_a_obj_drift.cpp:50
/// `mTiltSinScale` = 0.01 (`:51`) - the amplitude of both sway sines.
pub const TILT_SIN_SCALE: f32 = 0.01; // <- d_a_obj_drift.cpp:51
/// `field_0x34` = 0.5 (`:52`) - the gain on `mHitDir` inside the tilt acceleration.
pub const HIT_DIR_GAIN: f32 = 0.5; // <- d_a_obj_drift.cpp:52
/// `f32 rotFactor = mRideFlag != 0 ? 0.2f : 1.0f;` (`d_a_obj_drift.cpp:349`) - the sway gain,
/// halved while the player is riding.
pub const SWAY_RIDING: f32 = 0.2; // <- d_a_obj_drift.cpp:349
/// `mHitDir *= 0.95f;` (`d_a_obj_drift.cpp:356`) - the per-tick decay of the wind-hit direction.
pub const HIT_DIR_DECAY: f32 = 0.95; // <- d_a_obj_drift.cpp:356
/// `static cXyz offset_vec(-489.35f, 600.0f, 0.0f);` (`calc_flower_param`,
/// `d_a_obj_drift.cpp:190`) - the flower's offset in the platform's local frame.
pub const FLOWER_OFFSET: Vec3 = Vec3::new(-489.35, 600.0, 0.0); // <- d_a_obj_drift.cpp:190
/// `static const s16 pf_name[2] = {fpcNm_JBO_e, fpcNm_BO_e};` (`make_flower`,
/// `d_a_obj_drift.cpp:175`), which `d_stage.cpp:697-698` binds to these two stage names. The
/// comparison is case-sensitive (RULE: do not case-fold actor names), and `jbaba` is already owned
/// by `src/sim/jbaba.rs` while `bbaba` is claimed by nothing.
pub const FLOWER_NAMES: [&str; 2] = ["jbaba", "bbaba"]; // <- d_stage.cpp:697, :698
/// `mWavePhaseX = cM_rndFX(0x8000);` and the `.Y` / `.Z` twins (`d_a_obj_drift.cpp:129-131`) - the
/// span of `cM_rndFX`, whose result is truncated into an `s16`.
pub const WAVE_PHASE_SEED: f32 = 0x8000 as f32; // <- d_a_obj_drift.cpp:129
/// `cM_rnd` (`c_math.cpp:177-182`) and `cM_initRnd(100, 100, 100)` (`m_Do_machine.cpp:617`), on
/// this actor's own copy of the three lanes exactly as `src/sim/ff.rs` does - there is no global
/// `cM_rnd` in this sim.
const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA]; // <- c_math.cpp:178-180
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673]; // <- c_math.cpp:178-180
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0]; // <- c_math.cpp:181
const RND_SEED: i32 = 100; // <- m_Do_machine.cpp:617

/// The Forest Haven / Great Sea floating log platform.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. The only collider is
/// `daObjDrift::Act_c::M_cyl_src` (`d_a_obj_drift.cpp:65-91`), whose `SrcObjAt Type` is `0`
/// (`:68`) - it registers no attack collider - and whose `SrcObjTg Type` is `AT_TYPE_WIND`
/// (`:71`) with `cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsOther_e` (`:72`), i.e. it is a TARGET that
/// wind emitters push. `mStts.Init(0xFF, 0xFF, this)` (`:107`) sets a hit-point budget but the
/// cylinder's `SrcObjTg` is the only registration, and `cCcD` registry work is confirmed-absent
/// (RULE 3b). Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
///
/// `Delete` (`:217-219`) and the dispatch `Mthd_IsDelete -> MoveBGIsDelete` (`:431`) are the
/// accepts-a-delete-request form (RULE 6b); `Mthd_Create` can return `cPhs_ERROR_e` from
/// `MoveBGCreate`'s assert arm only (`:164`), so `spawn` never refuses.
///
/// **This actor is NOT wind-driven.** `set_current` (`:341-379`) contains no `dKyw_get_wind_vec` or
/// `dKyw_get_wind_pow` call at all - the vertical bob is `cM_ssin(mWavePhaseX)` and both sway axes
/// are `cM_ssin` of their own phases, so the whole wave model is portable and `control` is real
/// work rather than an empty shell. The ONE wind dependency is the mode trigger: `M_cyl_src`'s
/// target collider is `AT_TYPE_WIND`, and RULE 3b lists `AT_TYPE_WIND` collider semantics as
/// confirmed-absent ("five retail emitters, none ported as targets"), so `mCyl.ChkTgHit()` at
/// `:236` is transcribed as a gate fed `false` and [`MODE_ROT`] is unreachable in the sim.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Drift {
    /// `mMode` (`d_a_obj_drift.h:478`) - the `mode_proc` dispatch subject (`:384-388`) and the
    /// `mMode == MODE_ROT` selector for the tilt chase factor (`:351`).
    pub mode: u8,
    /// `mType` (`d_a_obj_drift.h:440`) from `prm_get_type()` (`:156`). `0` and `1` select the
    /// flower species; anything else suppresses the flower entirely (`:183-185`).
    pub kind: u8,
    /// `mFlowerPid` (`d_a_obj_drift.h:474`), `None` for retail's
    /// `fpcM_ERROR_PROCESS_ID_e` (`:184`).
    pub flower: Option<Entity>,
    /// `home.pos.y` (`d_a_obj_drift.cpp:342`) - the vertical reference the platform's bob chases.
    /// `fopAc_ac_c::home` has no `Actor` field, so the spawn-time y is captured once in
    /// [`Drift::create_init`]; `dStage_actorCreate` gives home and current the same position, so
    /// this is the same number retail reads.
    pub home_y: f32,
    /// `mHitDir` (`d_a_obj_drift.h:462`) - the normalised wind-strike direction. Its only writer is
    /// the `AT_TYPE_WIND` arm of `mode_wait` (`:240`), and it is read every tick by the decay and
    /// the tilt acceleration (`:356`, `:362`, `:366`), so it is stored even though it stays zero.
    pub hit_dir: Vec3,
    /// `mRideYOff` (`d_a_obj_drift.h:479`) - `-20.0f` is set by `rideCB` (`:315`), which is a
    /// `dBgW` callback; absent, it keeps the `0.0f` `Create` wrote at `:121`.
    pub ride_y_off: f32,
    /// `mTiltTargetZ` / `mTiltTargetX` (`d_a_obj_drift.h:480` / `:482`) - written only by `rideCB`
    /// (`:335-336`) and cleared at the end of every `set_current` (`:376-377`).
    pub tilt_target: Vec3,
    /// `mTiltVelZ` / `mTiltVelX` (`d_a_obj_drift.h:484` / `:486`) - the damped tilt velocity.
    pub tilt_vel: Vec3,
    /// `mTiltZ` / `mTiltX` (`d_a_obj_drift.h:490` / `:492`) - the tilt fed to `quat_rotBaseY2`
    /// (`:290`).
    pub tilt: Vec3,
    /// `mWavePhaseX` / `Y` / `Z` (`d_a_obj_drift.h:498` / `:49A` / `:49C`) - `s16` phases that
    /// wrap on every step.
    pub wave: [i16; 3],
    /// `mRideFlag` (`d_a_obj_drift.h:49E`) - set only by `rideCB`.
    pub riding: bool,
    /// This actor's own `cM_rnd` lanes, as in `src/sim/ff.rs`.
    pub seed: [i32; 3],
    /// One-shot guard so `make_flower` (`:113`) runs exactly once, where `Commands` is available.
    pub entered: bool,
}

impl Drift {
    /// `daObjDrift::Act_c::Mthd_Create` (`:151-167`) plus `Create` (`:102-148`), minus the
    /// resource load, `MoveBGCreate`, the `cCcD` cylinder registration, the `dBgW` ride callback
    /// and the water-height probe.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            // `mode_wait_init()` (`:229-232`) sets `mMode = MODE_WAIT`, but the ctor zero-fill
            // (`f_pc_base.cpp:98`) already lands on the same value; both are MODE_WAIT.
            mode: MODE_WAIT,
            kind: ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8,
            flower: None,
            home_y: 0.0,
            // `mHitDir.set(cXyz::Zero);` (:137)
            hit_dir: Vec3::ZERO,
            // `mRideYOff = 0.0f;` (:121)
            ride_y_off: 0.0,
            // `mTiltTargetZ = 0.0f; mTiltTargetX = 0.0f;` (:122-123)
            tilt_target: Vec3::ZERO,
            // `mTiltVelZ = 0.0f; mTiltVelX = 0.0f;` (:124-125)
            tilt_vel: Vec3::ZERO,
            // `mTiltZ = 0.0f; mTiltX = 0.0f;` (:126-127)
            tilt: Vec3::ZERO,
            // seeded by `create_init`, which is where the RNG lives
            wave: [0; 3],
            // `mRideFlag = FALSE;` (:132)
            riding: false,
            seed: [RND_SEED; 3],
            entered: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `make_flower` (`:170-186`) plus `Create`'s tail: `home_y` capture, the three wave-phase
    /// seeds and the flower spawn, in the decomp's order.
    fn create_init(&mut self, actor: &Actor, commands: &mut Commands) {
        // `home.pos` is the spawn position; `dStage_actorCreate` (`d_stage.cpp:1306-1311`) leaves
        // home and current identical, so this is the value `set_current` reads at `:342`.
        self.home_y = actor.pos.y;
        // ```
        // mWavePhaseX = cM_rndFX(0x8000);
        // mWavePhaseY = cM_rndFX(0x8000);
        // mWavePhaseZ = cM_rndFX(0x8000);
        // ```
        // (:129-131). Draw order is X, then Y, then Z, off one shared `cM_rnd` stream.
        self.wave = [
            self.rnd_fx(WAVE_PHASE_SEED) as i32 as i16,
            self.rnd_fx(WAVE_PHASE_SEED) as i32 as i16,
            self.rnd_fx(WAVE_PHASE_SEED) as i32 as i16,
        ];
        // ```
        // if (mType >= 0 && mType <= 1) {
        //     calc_flower_param(&pos, &angle);
        //     mFlowerPid = fopAcM_create(pf_name[mType], 0, &pos, fopAcM_GetHomeRoomNo(this), &angle);
        // } else { mFlowerPid = fpcM_ERROR_PROCESS_ID_e; }
        // ```
        // (:171-185) - `mType` is `int` from `prm_get_type()` and is already masked to `0..7`, so
        // `mType >= 0` is always true and the guard is the upper bound only. The parameter is `0`
        // (`:177`) and the room is `fopAcM_GetHomeRoomNo(this)`.
        if self.kind <= 1 {
            let (pos, angle) = self.flower_transform(actor);
            let entity = commands
                .spawn(Actor {
                    name: FLOWER_NAMES[usize::from(self.kind)].to_owned(),
                    parameters: 0,
                    pos,
                    angle,
                    set_id: 0,
                    room: actor.room,
                    ..Default::default()
                })
                .id();
            self.flower = Some(entity);
        }
        self.entered = true;
    }

    /// `set_mtx` (`:286-298`) followed by `calc_flower_param` (`:189-194`).
    ///
    /// `mMtx` is built as `transS(current.pos)` then `quatM(&quat)` then `ZXYrotM(shape_angle)`
    /// (`:287-293`). `mDoMtx_ZXYrotM` (`m_Do_mtx.cpp:68-85`) concatenates `Ry`, then `Rx`, then
    /// `Rz` onto the right of the running matrix, so the product is `T * Q * Ry * Rx * Rz`.
    /// `calc_flower_param` takes `mDoMtx_multVec(mMtx, &offset_vec, i_pos)` - a point transform,
    /// so the translation applies - and `mDoMtx_MtxToRot(mMtx, i_angle)` (`:282-297`) to recover
    /// the euler triple.
    fn flower_transform(&self, actor: &Actor) -> (Vec3, [i16; 3]) {
        let mtx = Mat4::from_translation(actor.pos)
            * Mat4::from_rotation_quat(tilt_quat(self.tilt))
            * zxy(actor.shape_angle);
        (mtx.transform_point3(FLOWER_OFFSET), mtx_to_rot(&mtx))
    }

    /// `daObjDrift::Act_c::Execute` (`:382-395`): the mode proc, `set_current`, `set_mtx`,
    /// `*mtx = &mMtx` and `set_flower_current`.
    fn execute(&mut self, actor: &mut Actor, flower: Option<&mut Actor>) {
        // `(this->*mode_proc[mMode])();` (:388). MODE_WAIT's body is the collider move
        // `mCyl.MoveCAtTg(current.pos); dComIfG_Ccsp()->Set(&mCyl);` (:245-246) plus the
        // `AT_TYPE_WIND` trigger arm at `:236-243`; both are absent channels (RULE 3b), so neither
        // mode proc has a modelled body. MODE_ROT is transcribed nowhere because its own state
        // (`mTargetAngleY`, `mAngleChaseY`, `mRotTimer`, `field_0x4D0`) has no other reader.
        self.set_current(actor);
        // `set_mtx(); *mtx = &mMtx;` (:391-392) - the matrix itself is consumed only by
        // `set_flower_current`, which `flower_transform` performs below.
        if let Some(flower) = flower {
            self.set_flower_current(flower, actor);
        }
    }

    /// `set_flower_current` (`:197-214`), with the `mFlowerPid == fpcM_ERROR_PROCESS_ID_e` early
    /// return (`:198-200`) already resolved by the `Option`.
    fn set_flower_current(&self, flower: &mut Actor, actor: &Actor) {
        let (pos, angle) = self.flower_transform(actor);
        // `flower->current.pos = pos;` (:207)
        flower.pos = pos;
        // ```
        // if (mType == 0) { flower->shape_angle = angle; } else { flower->current.angle = angle; }
        // ```
        // (:208-212) - the two species take the rotation on a different actor field.
        if self.kind == 0 {
            flower.shape_angle = angle;
        } else {
            flower.angle = angle;
        }
    }

    /// `set_current` (`:341-379`).
    fn set_current(&mut self, actor: &mut Actor) {
        // ```
        // f32 y = mRideYOff + home.pos.y;
        // mWavePhaseX += mWavePhaseXStep;
        // f32 sinVal = cM_ssin(mWavePhaseX);
        // f32 yDelta = y - current.pos.y;
        // gravity = yDelta * mWaveAmplitude + sinVal * mWaveSinScale;
        // ```
        // (:342-347). `mRideYOff` keeps the `0.0f` from `Create` because `rideCB` is a `dBgW`
        // callback (`:307`) and moving backgrounds are confirmed-absent (RULE 3b).
        let y = self.ride_y_off + self.home_y;
        self.wave[0] = self.wave[0].wrapping_add(WAVE_PHASE_X_STEP);
        let sin_x = walk::from_s16(self.wave[0]).sin();
        let y_delta = y - actor.pos.y;
        actor.gravity = y_delta * WAVE_AMPLITUDE + sin_x * WAVE_SIN_SCALE;
        // ```
        // daObj::posMoveF_stream(this, NULL, &cXyz::Zero, mWaveSpeed, mWavePhase);
        // ```
        // (:348) - `posMoveF_stream` (`d_a_obj.cpp:94-96`) is `posMoveF_grade` with `pNorm = NULL`,
        // `friction = 0`, `no_grade_cos = 0.0f`, `p_add_accel = NULL`, which
        // `super::stream::move_stream` already implements verbatim. `pNorm == NULL` makes
        // `posMoveF_grade_acc` return `cXyz::Zero` (`d_a_obj.cpp:114-115`, `:125`), so gravity
        // enters only through the `speed.y + gravity` term (`:157`). `fopAcM_posMove` at `:173` is
        // the position integration.
        let (speed, speed_f, yaw) = stream::move_stream(
            actor.speed,
            Vec3::ZERO,
            actor.angle[1],
            actor.speed_f,
            actor.gravity,
            actor.max_fall_speed,
            WAVE_SPEED,
            WAVE_PHASE,
        );
        actor.speed = speed;
        actor.speed_f = speed_f;
        actor.angle[1] = yaw;
        actor.integrate();
        // ```
        // f32 rotFactor = mRideFlag != 0 ? 0.2f : 1.0f;
        // f32 chaseFactor;
        // if (mMode == MODE_ROT) { chaseFactor = mTiltChaseRot; } else { chaseFactor = mTiltChaseWait; }
        // mHitDir *= 0.95f;
        // mWavePhaseY += mWavePhaseYStep;
        // mWavePhaseZ += mWavePhaseZStep;
        // ```
        // (:349-358)
        let rot_factor = if self.riding { SWAY_RIDING } else { 1.0 };
        let chase_factor = if self.mode == MODE_ROT {
            TILT_CHASE_ROT
        } else {
            TILT_CHASE_WAIT
        };
        self.hit_dir *= HIT_DIR_DECAY;
        self.wave[1] = self.wave[1].wrapping_add(WAVE_PHASE_Y_STEP);
        self.wave[2] = self.wave[2].wrapping_add(WAVE_PHASE_Z_STEP);
        // ```
        // f32 diffZ = mTiltZ - mTiltTargetZ;
        // f32 diffX = mTiltX - mTiltTargetX;
        // f32 sinY = cM_ssin(mWavePhaseY);
        // f32 z_val = mHitDir.z * field_0x34 + (rotFactor * (sinY * mTiltSinScale) - diffZ * chaseFactor);
        // f32 sinZ = cM_ssin(mWavePhaseZ);
        // f32 x_val = mHitDir.x * field_0x34 + (rotFactor * (sinZ * mTiltSinScale) - diffX * chaseFactor);
        // mTiltVelZ += z_val;
        // mTiltVelX += x_val;
        // mTiltVelZ *= mPosMoveDamp;
        // mTiltVelX *= mPosMoveDamp;
        // mTiltZ += mTiltVelZ;
        // mTiltX += mTiltVelX;
        // mRideYOff = 0.0f;
        // mTiltTargetZ = 0.0f;
        // mTiltTargetX = 0.0f;
        // mRideFlag = FALSE;
        // ```
        // (:359-378). The two velocities are accumulated and then damped independently, so both
        // accelerations are computed from the pre-update tilt before either write.
        let diff_z = self.tilt.z - self.tilt_target.z;
        let diff_x = self.tilt.x - self.tilt_target.x;
        let sin_y = walk::from_s16(self.wave[1]).sin();
        let z_val = self.hit_dir.z * HIT_DIR_GAIN
            + (rot_factor * (sin_y * TILT_SIN_SCALE) - diff_z * chase_factor);
        let sin_z = walk::from_s16(self.wave[2]).sin();
        let x_val = self.hit_dir.x * HIT_DIR_GAIN
            + (rot_factor * (sin_z * TILT_SIN_SCALE) - diff_x * chase_factor);
        self.tilt_vel.z = (self.tilt_vel.z + z_val) * TILT_DAMP;
        self.tilt_vel.x = (self.tilt_vel.x + x_val) * TILT_DAMP;
        self.tilt.z += self.tilt_vel.z;
        self.tilt.x += self.tilt_vel.x;
        self.ride_y_off = 0.0;
        self.tilt_target = Vec3::ZERO;
        self.riding = false;
    }

    /// `cM_rnd` (`c_math.cpp:177-182`) on this actor's own lanes.
    fn rnd(&mut self) -> f32 {
        for lane in 0..3 {
            self.seed[lane] = (self.seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
        }
        let sum = self.seed[0] as f32 / RND_SCALE[0]
            + self.seed[1] as f32 / RND_SCALE[1]
            + self.seed[2] as f32 / RND_SCALE[2];
        (sum % 1.0).abs()
    }

    /// `cM_rndFX(f) = f * (cM_rnd() - 0.5f) * 2.0f` (`c_math.cpp:190-192`).
    fn rnd_fx(&mut self, max: f32) -> f32 {
        max * (self.rnd() - 0.5) * 2.0
    }
}

/// `daObj::quat_rotBaseY2` (`d_a_obj.cpp:200-220`) for the axis `cXyz(mTiltX, 1.0f, mTiltZ)`
/// (`d_a_obj_drift.cpp:289`). It is the shortest-arc quaternion from `cXyz::BaseY` onto the
/// normalised axis: `w = 0.5 * sqrt(2 * (BaseY . axis + 1))` and `xyz = (BaseY x axis) / (2w)`.
/// `BaseY x axis` for `(0, 1, 0) x (ax, ay, az)` is `(az, 0, -ax)`.
fn tilt_quat(tilt: Vec3) -> Quat {
    let axis = Vec3::new(tilt.x, 1.0, tilt.z);
    let length = axis.length();
    if length == 0.0 {
        return Quat::IDENTITY;
    }
    let unit = axis / length;
    let w = (2.0 * (unit.y + 1.0)).sqrt();
    if w.abs() <= 0.0001 {
        return Quat::IDENTITY;
    }
    Quat::from_xyzw(unit.z / w, 0.0, -unit.x / w, 0.5 * w)
}

/// `mDoMtx_ZXYrotM(shape_angle)` (`m_Do_mtx.cpp:68-85`): each non-zero angle is concatenated onto
/// the right of the running matrix in the order `Ry(y)`, then `Rx(x)`, then `Rz(z)`, giving
/// `Ry(y) * Rx(x) * Rz(z)`. `mDoMtx_YrotS` / `XrotS` / `ZrotS` (`:86-140`) each write the ordinary
/// right-handed rotation for `JMASCos` / `JMASSin`, which is what glam's `from_rotation_*` produce
/// for the same angle, so RULE 5b's single converter is the only angle path used here.
fn zxy(angle: [i16; 3]) -> Mat4 {
    Mat4::from_rotation_y(walk::from_s16(angle[1]))
        * Mat4::from_rotation_x(walk::from_s16(angle[0]))
        * Mat4::from_rotation_z(walk::from_s16(angle[2]))
}

/// One matrix element, retail's `m[row][col]` indexing. `Mat4::to_cols_array` is column-major, so
/// `m[row][col]` is `array[col * 4 + row]`.
fn element(mtx: &Mat4, row: usize, col: usize) -> f32 {
    mtx.to_cols_array()[col * 4 + row]
}

/// `mDoMtx_MtxToRot` (`m_Do_mtx.cpp:282-297`), transcribed including the gimbal-lock arm.
fn mtx_to_rot(mtx: &Mat4) -> [i16; 3] {
    let m02 = element(mtx, 0, 2);
    let m22 = element(mtx, 2, 2);
    // `f31 = m[0][2]; f31 *= f31; f31 += m[2][2] * m[2][2]; f31 = sqrtf(f31);`
    let pitch_sine_scale = (m02 * m02 + m22 * m22).sqrt();
    let pitch = walk::to_s16((-element(mtx, 1, 2)).atan2(pitch_sine_scale));
    // `if (o_rot->x == 0x4000 || o_rot->x == -0x4000)` (`:290`) - the pitch singularity, where
    // roll is forced to zero and yaw is taken from the other pair of elements instead.
    if pitch == 0x4000 || pitch == -0x4000 {
        let yaw = walk::to_s16((-element(mtx, 2, 0)).atan2(element(mtx, 0, 0)));
        return [pitch, yaw, 0];
    }
    let yaw = walk::to_s16(m02.atan2(m22));
    let roll = walk::to_s16(element(mtx, 1, 0).atan2(element(mtx, 1, 1)));
    [pitch, yaw, roll]
}

pub fn is_drift(name: &str) -> bool {
    DRIFT_ACTORS.contains(&name)
}

pub fn control(
    mut commands: Commands,
    mut flowers: Query<&mut Actor, Without<Drift>>,
    mut drifts: Query<(&mut Actor, &mut Drift), With<Drift>>,
) {
    for (actor, mut drift) in &mut drifts {
        if !drift.entered {
            drift.create_init(&*actor, &mut commands);
        }
        let mut flower = drift
            .flower
            .and_then(|entity| flowers.get_mut(entity).ok());
        drift.execute(&mut *actor, flower.as_deref_mut());
    }
}
