use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::sea::Sea;
use super::walk;

/// `OBJNAME("Buoyr", fpcNm_Obj_Buoyrace_e, 255, 0)` (`d_stage.cpp:1254`) - the OBJNAME table is the
/// authority. `daObjBuoyrace::Act_c::M_arcname_kiba` / `M_arcname_hasi` (`d_a_obj_buoyrace.cpp:13-14`)
/// are `"Kkiba_00"` / `"Khasi_00"`, i.e. ARCHIVE names, so neither is the stage string. Separately,
/// `src/sim/goal_flag.rs` already carries `BUOY_RACE_ACTOR: &str = "Buoyrace"` (`:26`), which is the
/// string `fopAcM_createChild(fpcNm_Obj_Buoyrace_e, ...)` (`d_a_goal_flag.cpp:343-349`) stamps on the
/// buoys it spawns; `"Buoyrace"` has no OBJNAME row at all, so it is not a stage string and the two
/// constants do not collide.
pub const BUOYRACE_ACTORS: [&str; 1] = ["Buoyr"];

/// `daObjBuoyrace::Act_c::Attr_c` (`d_a_obj_buoyrace.h:18-28`), initialised by
/// `M_attr` (`d_a_obj_buoyrace.cpp:15-25`). One shared table, no per-type index.
const M00: f32 = 150.0; // <- d_a_obj_buoyrace.cpp:16 (Attr_c::m00)
const MSCALE: f32 = 2.0; // <- d_a_obj_buoyrace.cpp:17 (Attr_c::mScale)
const M08: f32 = 0.8; // <- d_a_obj_buoyrace.cpp:18 (Attr_c::m08)
const M0C: f32 = 0.5; // <- d_a_obj_buoyrace.cpp:19 (Attr_c::m0C)
const M14: f32 = 0.3; // <- d_a_obj_buoyrace.cpp:21 (Attr_c::m14)
const M18: f32 = 0.02; // <- d_a_obj_buoyrace.cpp:22 (Attr_c::m18)
const M1C: f32 = 0.04; // <- d_a_obj_buoyrace.cpp:23 (Attr_c::m1C)
const M20: f32 = 0.35; // <- d_a_obj_buoyrace.cpp:24 (Attr_c::m20)

/// `_create`'s cull box (`:93-101`), `fopAcM_setCullSizeBox(this, -1.0f * 76.0f * attr().mScale,
/// -1.0f * attr().mScale, -1.0f * 76.0f * attr().mScale, 76.0f * attr().mScale,
/// 295.0f * attr().mScale, 76.0f * attr().mScale)` with `mScale = 2.0` - so the resolved box is
/// (-152, -2, -152) .. (152, 590, 152). The profile's `fopAc_CULLBOX_CUSTOM_e` (`:324`) resolves to a
/// real box.
const CULL_MIN: [f32; 3] = [-152.0, -2.0, -152.0]; // <- d_a_obj_buoyrace.cpp:95-97
const CULL_MAX: [f32; 3] = [152.0, 590.0, 152.0]; // <- d_a_obj_buoyrace.cpp:98-100

/// `daSea_calcWave(current.pos.x - 32.0f, current.pos.z - 32.0f)` and its two siblings
/// (`:163-165`) - the half-diagonal of the wave-normal probe square.
const WAVE_PROBE: f32 = 32.0; // <- d_a_obj_buoyrace.cpp:163
/// `mMeanSeaHeight = (wave_height_1 + wave_height_2 + wave_height_3) * (1.0f / 3.0f)` (`:176`).
const MEAN_DIVISOR: f32 = 1.0 / 3.0; // <- d_a_obj_buoyrace.cpp:176
/// `m2C0 += m2BC * -0.01f;` (`:220`).
const VEL_DAMP: f32 = -0.01; // <- d_a_obj_buoyrace.cpp:220
/// `m2C0 *= 0.94f;` (`:221`) - the bob's inertia.
const VEL_RETAIN: f32 = 0.94; // <- d_a_obj_buoyrace.cpp:221
/// `if (m2BC > 0.2f * attr().m00) { m2BC = 0.2f * attr().m00; }` (`:223-225`).
const BOB_LIMIT: f32 = 0.2; // <- d_a_obj_buoyrace.cpp:223
/// `cLib_chaseF(pos_y, target, 50.0f)` (`:230-234`).
const HEIGHT_STEP: f32 = 50.0; // <- d_a_obj_buoyrace.cpp:233
/// `f32 cond = attr().m14 * attr().m14;` (`:183`) - the sway clamp radius, squared.
const SWAY_COND: f32 = 0.09; // <- d_a_obj_buoyrace.cpp:183 (m14 * m14, m14 = 0.3)
/// `cXyz::isNearZeroSquare() { return (this->getSquareMag() < 8e-11f); }`
/// (`c_xyz.h:111`), the precondition `cXyz::normalize` asserts (`c_xyz.cpp:105`) before
/// `VECNormalize`. Guarded here so a flat sea cannot put a NaN into the sim where retail asserts.
const NEAR_ZERO_SQUARE: f32 = 8e-11; // <- c_xyz.h:111

/// One racing buoy on a `Gflag` rope course - `daObjBuoyrace::Act_c` (`d_a_obj_buoyrace.h:8-82`),
/// the actor behind the stage string `Buoyr` and behind the 16/68-Buoy spawns
/// `daGoal_Flag_c::CreateBuoyRaces` makes (`d_a_goal_flag.cpp:328-356`).
///
/// `prm_get_id()` and `prm_get_line()` (`d_a_obj_buoyrace.h:48-49`) decode the two bytes that
/// `CreateBuoyRaces` packs as `j | (i << 8)` (`d_a_goal_flag.cpp:346`), and their ONLY reader is
/// `set_rope_pos()` (`:239-253`) - which is dead code in retail: it fetches
/// `parent->getRopePos(line, id)`, transforms a local `base_pos` through the matrix stack
/// (`mDoMtx_stack_c::multVec(&base_pos, rope_pos)`, `:251`) and then writes the result nowhere.
/// Neither the source nor the destination is an actor field. So this port drops `set_rope_pos`
/// whole, and with it both parameter decodes, per RULE 7 - which is also why
/// `goal_flag::BuoyRaceSpawn` (which `goal_flag.rs:1330-1334` attaches to every spawned buoy purely
/// to carry that dead pair) has no consumer. That module is not mine to edit.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE. `daObjBuoyrace::Act_c` is a plain
/// `fopAc_ac_c` (`d_a_obj_buoyrace.h:8`) holding two `J3DModel*` and two `request_of_phase_process`
/// (`:72-79`); it registers no `dCcD` collider of any kind, and the two `mpBgW` it does NOT have mean
/// there is no moving background to hit either. The profile `Group` is `fopAc_ACTOR_e` (`:323`).
/// Neither `combat::HitPoints` nor `enemy::Hurt` is attached - attaching them would let
/// `combat::control` despawn a buoy retail cannot damage.
///
/// `Mthd_IsDelete` (`:296-298`) is the bare "accepts a delete request" form of RULE 6b.
/// `_create` (`:69-108`) returns `cPhs_ERROR_e` only from `fopAcM_entrySolidHeap` (`:104`), which is
/// heap work, so `spawn` never refuses. There is no BCK frame read in this file.
///
/// ## The one omitted term
/// `afl_calc()`'s `m2A0 = m2A0 + (s16)(attr().m10 * (f32)(cM_rnd() + 1.0f));` (`:226`) draws from
/// the three-register global `cM_rnd()` (`c_math.cpp:177-182`). This sim has no global `cM_rnd` /
/// `cM_rndF` channel at all, and the task's standing instruction is to report such a draw rather than
/// substitute a different generator. So `m2A0` is transcribed but never advances, and it therefore
/// stays at the `fopAcM_ct` zero-fill (`f_pc_base.cpp:98`) that retail also starts from - which
/// means `cM_ssin(m2A0) == 0.0` at both readers (`:91` and `:228`), and the +-1.0-unit drift term is
/// absent from the buoy's height. Everything else in `afl_calc` and all of `afl_calc_sway` are
/// transcribed in full.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct BuoyRace {
    /// `mMeanSeaHeight` (`d_a_obj_buoyrace.h:62`) - `set_water_pos`'s three-probe mean (`:176`),
    /// and the base of both height targets.
    pub mean_sea_height: f32,
    /// `m294` (`d_a_obj_buoyrace.h:63`) - the normalised cross product of the two wave-slope
    /// vectors `set_water_pos` builds (`:166-178`).
    pub normal: Vec3,
    /// `m2BC` (`d_a_obj_buoyrace.h:70`) - the bob's displacement, clamped by `BOB_LIMIT * m00`.
    pub bob: f32,
    /// `m2C0` (`d_a_obj_buoyrace.h:71`) - the bob's velocity.
    pub bob_vel: f32,
    /// `m2AC` (`d_a_obj_buoyrace.h:66`) - the buoy's sway offset along the wave-normal's x axis.
    pub sway_x: f32,
    /// `m2B0` (`d_a_obj_buoyrace.h:67`) - the sway offset along its z axis.
    pub sway_z: f32,
    /// `m2B4` (`d_a_obj_buoyrace.h:68`) - the sway x spring.
    pub spring_x: f32,
    /// `m2B8` (`d_a_obj_buoyrace.h:69`) - the sway z spring.
    pub spring_z: f32,
    /// `m2A0` (`d_a_obj_buoyrace.h:64`, an `s16`) - the drift phase fed to `cM_ssin` at `:91` and
    /// `:228`. Never advanced here - see the omitted-term note on the type.
    pub drift: i16,
    /// One-shot guard so `_create`'s `set_water_pos()` + `current.pos.y` assignment (`:90-91`) runs
    /// once, on the first tick, where a mutable `Actor` is available.
    pub entered: bool,
}

impl BuoyRace {
    /// `_create` (`:69-108`) minus `fopAcM_ct` (`:70`), `create_load` (`:73`), `fopAcM_entrySolidHeap`
    /// (`:89`), `set_water_pos` (`:90`) and the `current.pos.y` assignment (`:91`), which are
    /// resource, heap and first-tick work; and `init_mtx` (`:102`), which is a pure `set_mtx()` call
    /// (`:157-159`).
    ///
    /// `daObjBuoyrace::Act_c` carries no `fopAcM_prm_class` parameter - the profile's `Parameters`
    /// field is `0` (`:320`) - so there is no parameter decode to perform (see the note on
    /// `set_rope_pos` in the type documentation).
    ///
    /// RULE 2: `set_mtx`'s `mpModelKiba->setBaseScale(scale * attr().mScale)` (`:125`) is the only
    /// reader of `scale`, and the stage-placed scale is exactly 1.0, so `mScale` is already folded in.
    pub fn spawn(_parameters: u32) -> Self {
        Self {
            mean_sea_height: 0.0,
            normal: Vec3::ZERO,
            bob: 0.0,
            bob_vel: 0.0,
            sway_x: 0.0,
            sway_z: 0.0,
            spring_x: 0.0,
            spring_z: 0.0,
            drift: 0,
            entered: false,
        }
    }

    /// `_create`'s `fopAcM_setCullSizeBox` (`:93-101`) with `attr().mScale = 2.0`.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `_create`'s placement tail (`:90-91`):
    /// `set_water_pos(); current.pos.y = mMeanSeaHeight + (attr().m08 * -attr().m00 +
    /// attr().m0C * cM_ssin(m2A0)) * attr().mScale;`
    fn create_init(&mut self, actor: &mut Actor, sea: &Sea) {
        self.water_pos(actor.pos, sea);
        actor.pos.y = self.mean_sea_height + (M08 * -M00 + M0C * walk::from_s16(self.drift).sin()) * MSCALE;
        self.entered = true;
    }

    /// `daObjBuoyrace::Act_c::set_water_pos()` (`:162-179`). `daSea_calcWave` (`d_a_sea.cpp:349-401`)
    /// is exactly `sea::Sea::height_at` - same `GRID_SIZE` 800 and `GRID_CELLS` 65
    /// (`d_a_sea.cpp:17-18`, mirrored by `sea.rs`'s `GRID_SIZE` / `GRID_CELLS`), same
    /// `GRID_INDEX(x, z) = x + GRID_CELLS * z` gather (`:19`), same `cM3d_CalcPla` diagonal split on
    /// `f1 + f0 >= 1.0f` (`:393-397`) and the same
    /// `-((norm.x * x) + (norm.z * z) + baseY) / norm.y` return (`:400`). Its `daSea_ChkArea` guard
    /// (`:350-352`, returning `daSea_packet_c::BASE_HEIGHT = 1.0f` from `:23`) is
    /// `Sea::height_at`'s own guard.
    fn water_pos(&mut self, pos: Vec3, sea: &Sea) {
        // `f32 wave_height_1 = daSea_calcWave(current.pos.x - 32.0f, current.pos.z - 32.0f);` (:163)
        let wave_1 = sea.height_at(pos.x - WAVE_PROBE, pos.z - WAVE_PROBE);
        // `f32 wave_height_2 = daSea_calcWave(current.pos.x - 32.0f, current.pos.z + 32.0f);` (:164)
        let wave_2 = sea.height_at(pos.x - WAVE_PROBE, pos.z + WAVE_PROBE);
        // `f32 wave_height_3 = daSea_calcWave(current.pos.x + 32.0f, current.pos.z - 32.0f);` (:165)
        let wave_3 = sea.height_at(pos.x + WAVE_PROBE, pos.z - WAVE_PROBE);
        // `cXyz tmp1(0.0f, wave_height_2 - wave_height_1, 32.0f);` (:166-170)
        let tmp1 = Vec3::new(0.0, wave_2 - wave_1, WAVE_PROBE);
        // `cXyz tmp2(32.0f, wave_height_3 - wave_height_1, 0.0f);` (:171-175)
        let tmp2 = Vec3::new(WAVE_PROBE, wave_3 - wave_1, 0.0);
        // `mMeanSeaHeight = (wave_height_1 + wave_height_2 + wave_height_3) * (1.0f / 3.0f);` (:176)
        self.mean_sea_height = (wave_1 + wave_2 + wave_3) * MEAN_DIVISOR;
        // `m294 = tmp1.outprod(tmp2); m294.normalize();` (:177-178). `cXyz::outprod` is
        // `c_xyz.cpp:62-65`, a straight `return this->getCrossProduct(vec);`, and
        // `getCrossProduct` (`:55-60`) is `VECCrossProduct` - the standard right-handed cross
        // product, i.e. glam's `Vec3::cross`.
        let normal = tmp1.cross(tmp2);
        // `cXyz::normalize` (`c_xyz.cpp:104-108`) asserts `isNearZeroSquare() == FALSE`, i.e.
        // `getSquareMag() >= 8e-11f` (`c_xyz.h:111`) - three identical probe heights, i.e. a flat
        // sea. Retail asserts and then divides by zero; guard so the sim never holds a NaN.
        if normal.length_squared() >= NEAR_ZERO_SQUARE {
            self.normal = normal.normalize();
        }
    }

    /// `daObjBuoyrace::Act_c::_execute` (`:256-261`) - `set_water_pos(); afl_calc(); set_mtx();` -
    /// minus `set_mtx()` (`:259`), which is `setBaseScale` / matrix-stack work (`:118-154`).
    fn execute(&mut self, actor: &mut Actor, sea: &Sea) {
        self.water_pos(actor.pos, sea);
        self.afl_calc(actor);
    }

    /// `daObjBuoyrace::Act_c::afl_calc()` (`:219-236`).
    fn afl_calc(&mut self, actor: &mut Actor) {
        // `m2C0 += m2BC * -0.01f;` (:220)
        self.bob_vel += self.bob * VEL_DAMP;
        // `m2C0 *= 0.94f;` (:221)
        self.bob_vel *= VEL_RETAIN;
        // `m2BC += m2C0;` (:222)
        self.bob += self.bob_vel;
        // `if (m2BC > 0.2f * attr().m00) { m2BC = 0.2f * attr().m00; }` (:223-225)
        let limit = BOB_LIMIT * M00;
        if self.bob > limit {
            self.bob = limit;
        }
        // `m2A0 = m2A0 + (s16)(attr().m10 * (f32)(cM_rnd() + 1.0f));` (:226) - OMITTED: the draw is
        // the global `cM_rnd()` (`c_math.cpp:177-182`) and this sim has no global RNG channel. See
        // the note on the type; `m2A0` therefore stays at its zero-fill and the sine term below
        // contributes nothing.
        //
        // `f32 target = mMeanSeaHeight + m2BC + (attr().m08 * -attr().m00 + attr().m0C *
        // cM_ssin(m2A0)) * attr().mScale;` (:228)
        let target = self.mean_sea_height
            + self.bob
            + (M08 * -M00 + M0C * walk::from_s16(self.drift).sin()) * MSCALE;
        // `cLib_chaseF(pos_y, target, 50.0f)` (:229-234) - `c_lib.cpp:276-290` is `walk::chase_f`.
        walk::chase_f(&mut actor.pos.y, target, HEIGHT_STEP);
        // `afl_calc_sway();` (:235)
        self.afl_calc_sway();
    }

    /// `daObjBuoyrace::Act_c::afl_calc_sway()` (`:182-216`). Retail-compiled form only: the
    /// `#if VERSION == VERSION_DEMO` arm at `:200-209` is excluded (RULE 0's NTSC-U ordering), so
    /// `:195-198` is the live body.
    fn afl_calc_sway(&mut self) {
        // `f32 cond = attr().m14 * attr().m14;` (:183)
        // `f32 x = m294.x * attr().m20; f32 z = m294.z * attr().m20;` (:184-185)
        // `f32 square_mag_xz = x * x + z * z;` (:186)
        let (mut x, mut z) = (self.normal.x * M20, self.normal.z * M20);
        let square_mag_xz = x * x + z * z;
        // `if (square_mag_xz > cond) { square_mag_xz = std::sqrtf(square_mag_xz); x *= (attr().m14 /
        // square_mag_xz); z *= (attr().m14 / square_mag_xz); }` (:188-192) - a disc clamp to
        // `m14`. `SWAY_COND` is `m14 * m14`, the squared form of the same radius.
        if square_mag_xz > SWAY_COND {
            let length = square_mag_xz.sqrt();
            x *= M14 / length;
            z *= M14 / length;
        }
        // `f32 tmp4 = -(m2B0 - z) * attr().m18;` (:195)
        let tmp4 = -(self.sway_z - z) * M18;
        // `f32 tmp5 = -(m2AC - x) * attr().m18;` (:196)
        let tmp5 = -(self.sway_x - x) * M18;
        // `f32 tmp6 = -m2B8 * attr().m1C;` (:197)
        let tmp6 = -self.spring_z * M1C;
        // `f32 tmp7 = -m2B4 * attr().m1C;` (:198)
        let tmp7 = -self.spring_x * M1C;
        // `m2B4 += tmp5 + tmp7;` (:211)
        self.spring_x += tmp5 + tmp7;
        // `m2B8 += tmp4 + tmp6;` (:212)
        self.spring_z += tmp4 + tmp6;
        // `m2AC += m2B4;` (:214)
        self.sway_x += self.spring_x;
        // `m2B0 += m2B8;` (:215)
        self.sway_z += self.spring_z;
    }
}

pub fn is_buoyrace(name: &str) -> bool {
    BUOYRACE_ACTORS.contains(&name)
}

pub fn control(sea: Option<Res<Sea>>, mut buoys: Query<(&mut Actor, &mut BuoyRace)>) {
    // `sea::Sea` is only inserted for a stage that has a sea actor (`mod.rs`'s
    // `tables.has_sea_actor()` guard), so a `Res<Sea>` would panic every tick elsewhere. With no
    // sea there is no `daSea_calcWave` to read - and no `Buoyr` row either - so the tick is skipped
    // rather than fed a substitute.
    let Some(sea) = sea.as_deref() else {
        return;
    };
    for (mut actor, mut buoy) in &mut buoys {
        let buoy = &mut *buoy;
        let actor = &mut *actor;
        if !buoy.entered {
            buoy.create_init(actor, sea);
        }
        buoy.execute(actor, sea);
    }
}
