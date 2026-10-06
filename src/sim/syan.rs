use bevy_ecs::prelude::*;

use super::stream;

/// The stage string is `SYAN` (`d_stage.cpp:1228`, `fpcNm_SYAN_e`), NOT the `_arcname` `Syan`
/// (`d_a_syan.cpp:187`, `:197`, `:199`, `:209`).
pub const SYAN_ACTORS: [&str; 1] = ["SYAN"];

/// `enum { kEmtrNum = 6 }` (`d_a_syan.h:12-14`); also the `ARRAY_SIZE` bound of every per-emitter
/// array (`d_a_syan.cpp:39`, `:59`, `:81`, `:122`, `:188`, `:223`).
pub const EMITTER_NUM: usize = 6;
/// `s16 timer[2]` (`d_a_syan.h:29`), the `ARRAY_SIZE(i_this->timer)` bound at `d_a_syan.cpp:81`.
const TIMER_SLOTS: usize = 2;
/// `i_this->field_0x2b8 = 200.0f` (`d_a_syan.cpp:220`).
const AMP_INIT: f32 = 200.0;
/// `(REG0_S(8) + 800)` in `cM_ssin((s16)i_this->frameCount * ...)` (`d_a_syan.cpp:91`).
const YAW_STEP: i32 = 800;
/// `(REG0_S(9) + 700)` in the same call (`d_a_syan.cpp:92`).
const ROLL_STEP: i32 = 700;
/// `static s16 fire_time[] = { 0, 5, 10, 15, 20, 25 }` (`d_a_syan.cpp:222`).
const FIRE_TIME: [i16; EMITTER_NUM] = [0, 5, 10, 15, 20, 25];
/// `cM_rndF(REG0_F(2) + 10.0f)` (`d_a_syan.cpp:124`).
const LIFE_SPAN: f32 = 10.0;
/// `cM_rndF(REG0_F(6) + 4.0f)` (`d_a_syan.cpp:125`).
const ALPHA_SPAN: f32 = 4.0;
/// `cM_rndF(6.0f)` (`d_a_syan.cpp:128`).
const GROW_SPAN: f32 = 6.0;
/// `cM_rndF(0.075f)` (`d_a_syan.cpp:129`).
const GROW_BIAS: f32 = 0.075;
/// `fopAcM_SetMin(i_this, -1000.0f, -5000.0f, -1000.0f)` (`d_a_syan.cpp:216`).
const CULL_MIN: [f32; 3] = [-1000.0, -5000.0, -1000.0];
/// `fopAcM_SetMax(i_this, 1000.0f, 5000.0f, 1000.0f)` (`d_a_syan.cpp:217`).
const CULL_MAX: [f32; 3] = [1000.0, 5000.0, 1000.0];
/// `cM_initRnd(100, 100, 100)` (`m_Do_machine.cpp:617`).
const RND_SEED: i32 = 100;
/// `cM_rnd` multipliers (`c_math.cpp:178-180`).
const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
/// `cM_rnd` moduli (`c_math.cpp:178-180`).
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
/// `cM_rnd` reciprocals (`c_math.cpp:181`).
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];

/// `i_this->state` (`d_a_syan.h:20`) is the three-way dispatch at `d_a_syan.cpp:94-113`. Only
/// `case 0` is reachable in retail - `_create` (`:206-230`) never writes `state` and the sole
/// writer `i_this->state++` (`:103`) sits inside `case 1`. Both other arms are transcribed anyway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// `case 0: break` (`d_a_syan.cpp:95-96`).
    Idle,
    /// `case 1` (`d_a_syan.cpp:97-104`).
    Swing,
    /// `case 2` (`d_a_syan.cpp:105-112`).
    Wide,
}

/// The unused Spaceworld-2001-trailer chandelier: six alpha-sphere "candle flames" positioned by
/// joint matrices that the model's animation callback rewrites. `syan_class`
/// (`d_a_syan.h:10-46`) registers no collider and `d_a_syan.cpp` never calls `dComIfG_Ccsp()`, so
/// it can neither be hit nor hit anything; the profile `Group` is `fopAc_ACTOR_e` (`:253`).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Syan {
    /// `s32 frameCount` (`d_a_syan.h:18`), incremented at `d_a_syan.cpp:86`.
    pub frame_count: i32,
    /// `u32 state` (`d_a_syan.h:20`).
    pub state: State,
    /// `s16 timer[2]` (`d_a_syan.h:29`), decremented at `d_a_syan.cpp:81-83`, read at `:102`.
    pub timer: [i16; TIMER_SLOTS],
    /// `csXyz field_0x2b0.x` (`d_a_syan.h:21`) - the joint-5 rotation driven by
    /// `cLib_addCalcAngleS2` (`d_a_syan.cpp:98`, `:110`).
    pub rot_x: i16,
    /// `csXyz field_0x2b0.y` (`d_a_syan.h:21`), `field_0x2b8 * cM_ssin(...)` (`d_a_syan.cpp:91`).
    pub yaw: f32,
    /// `csXyz field_0x2b0.z` (`d_a_syan.h:21`), `field_0x2b8 * cM_ssin(...)` (`d_a_syan.cpp:92`).
    pub roll: f32,
    /// `f32 field_0x2b8` (`d_a_syan.h:22`) - the shared yaw/roll amplitude.
    pub amp: f32,
    /// `f32 field_0x2bc` (`d_a_syan.h:23`) - the joint-5 pitch amplitude.
    pub amp5: f32,
    /// `f32 field_0x2c0` (`d_a_syan.h:24`) - the joint-8 pitch amplitude.
    pub amp8: f32,
    /// `csXyz field_0x2a4[0].x` (`d_a_syan.h:20`), `field_0x2bc * cM_ssin(field_0x2c4)`
    /// (`d_a_syan.cpp:89`) - the joint-5 rotation fed to `nodeCallBack` (`:28`).
    pub joint5_pitch: f32,
    /// `csXyz field_0x2a4[1].x` (`d_a_syan.h:20`), `field_0x2c0 * cM_ssin(field_0x2c8)`
    /// (`d_a_syan.cpp:90`) - the joint-8 rotation fed to `nodeCallBack` (`:34`).
    pub joint8_pitch: f32,
    /// `f32 field_0x2c4` (`d_a_syan.h:25`) - the joint-5 phase accumulator.
    pub phase5: f32,
    /// `f32 field_0x2c8` (`d_a_syan.h:26`) - the joint-8 phase accumulator.
    pub phase8: f32,
    /// `f32 field_0x2cc` (`d_a_syan.h:27`) - the joint-5 phase rate.
    pub rate5: f32,
    /// `f32 field_0x2d0` (`d_a_syan.h:28`) - the joint-8 phase rate.
    pub rate8: f32,
    /// `f32 field_0x2dc` (`d_a_syan.h:31`), read at `d_a_syan.cpp:108-111`.
    pub wide_scale: f32,
    /// `f32 field_0x52c[i]` (`d_a_syan.h:38`) - the per-emitter alpha target. Its only reader is
    /// the `cLib_addCalc2` into `partAlpha[i]` (`d_a_syan.cpp:131`), which is renderer state.
    pub alpha_target: [f32; EMITTER_NUM],
    /// `f32 field_0x55c[i]` (`d_a_syan.h:40`) - the per-emitter scale target, read only by the
    /// `cLib_addCalc2` into `partScale1[i]` (`d_a_syan.cpp:132`).
    pub scale_target: [f32; EMITTER_NUM],
    /// `s16 field_0x58c[i]` (`d_a_syan.h:42`) - the alpha life counter.
    pub life: [i16; EMITTER_NUM],
    /// `s16 field_0x598[i]` (`d_a_syan.h:43`) - the scale life counter.
    pub grow: [i16; EMITTER_NUM],
    /// `s16 field_0x5a4[i]` (`d_a_syan.h:44`) - the fire-emitter counter, seeded from `FIRE_TIME`.
    pub fire: [i16; EMITTER_NUM],
    /// `bool emtrEnabled[i]` (`d_a_syan.h:45`).
    pub emitter_enabled: [bool; EMITTER_NUM],
    seed: [i32; 3],
}

impl Syan {
    /// `daSyan_Create` (`d_a_syan.cpp:206-230`): `i_this->field_0x2b8 = 200.0f` (`:220`) and
    /// `field_0x5a4[i] = fire_time[i]` (`:222-224`). Everything else (`fopAcM_SetMin`/`SetMax`,
    /// `fopAcM_SetMtx`, the joint callbacks, the `mDoExt_McaMorf`) is renderer or heap work, so
    /// `state`, `timer`, `field_0x2a4`, `field_0x2b0.x`, `field_0x2bc`..`field_0x2dc`,
    /// `field_0x52c`, `field_0x55c`, `field_0x58c`, `field_0x598` and `emtrEnabled` all start at
    /// zero/false from `fopAcM_ct` (`:207`). `daSyan_solidHeapCB` (`:194-203`) can return
    /// `FALSE`, which turns the phase state into `cPhs_ERROR_e` (`:226`) and cancels the actor
    /// before it ever executes - that is an archive-load failure, not a parameter, so `spawn`
    /// returns `Self`.
    pub fn spawn() -> Self {
        Self {
            frame_count: 0,
            state: State::Idle,
            timer: [0; TIMER_SLOTS],
            rot_x: 0,
            yaw: 0.0,
            roll: 0.0,
            amp: AMP_INIT,
            amp5: 0.0,
            amp8: 0.0,
            joint5_pitch: 0.0,
            joint8_pitch: 0.0,
            phase5: 0.0,
            phase8: 0.0,
            rate5: 0.0,
            rate8: 0.0,
            wide_scale: 0.0,
            alpha_target: [0.0; EMITTER_NUM],
            scale_target: [0.0; EMITTER_NUM],
            life: [0; EMITTER_NUM],
            grow: [0; EMITTER_NUM],
            fire: FIRE_TIME,
            emitter_enabled: [false; EMITTER_NUM],
            seed: [RND_SEED; 3],
        }
    }

    /// `fopAcM_SetMin(i_this, -1000.0f, -5000.0f, -1000.0f)` (`d_a_syan.cpp:216`) and
    /// `fopAcM_SetMax(i_this, 1000.0f, 5000.0f, 1000.0f)` (`:217`) - a real custom cull box.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `daSyan_Execute` (`d_a_syan.cpp:80-177`) minus `i_this->morf->calc()` (`:85`), the
    /// `mDoMtx`/`setBaseScale`/`setBaseTRMtx` block (`:116-120`), `syan_draw` (`:52-68`) and
    /// every `dComIfGp_particle_set*` call (`:139`, `:163`).
    fn step(&mut self) {
        for slot in self.timer.iter_mut() {
            if *slot != 0 {
                *slot -= 1;
            }
        }
        self.frame_count += 1;
        self.phase5 += self.rate5;
        self.phase8 += self.rate8;
        self.joint5_pitch = self.amp5 * stream::sin_s(self.phase5 as i16);
        self.joint8_pitch = self.amp8 * stream::sin_s(self.phase8 as i16);
        let frame = self.frame_count as i16 as i32;
        self.yaw = self.amp * stream::sin_s(frame.wrapping_mul(YAW_STEP) as i16);
        self.roll = self.amp * stream::sin_s(frame.wrapping_mul(ROLL_STEP) as i16);
        match self.state {
            State::Idle => {}
            State::Swing => {
                stream::add_calc_angle_s2(&mut self.rot_x, 2500, 2, 250);
                add_calc2(&mut self.amp5, 1500.0, 0.5, 150.0);
                add_calc2(&mut self.amp8, 2500.0, 0.5, 250.0);
                add_calc2(&mut self.amp, 2000.0, 0.5, 200.0);
                if self.timer[0] == 0 {
                    self.state = State::Wide;
                }
            }
            State::Wide => {
                add_calc2(&mut self.rate5, 500.0, 0.1, 50.0);
                add_calc2(&mut self.rate8, 1000.0, 0.1, 20.0);
                let pitch = self.wide_scale * 300.0;
                add_calc2(&mut self.amp5, pitch, 0.1, 10.0);
                let tilt = self.wide_scale * 300.0;
                add_calc2(&mut self.amp8, tilt, 0.1, 10.0);
                stream::add_calc_angle_s2(&mut self.rot_x, 2500, 2, 2000);
                let swing = self.wide_scale * 500.0;
                add_calc2(&mut self.amp, swing, 0.2, 5.0);
            }
        }
        for index in 0..EMITTER_NUM {
            self.emitter(index);
        }
    }

    /// The per-emitter body of `d_a_syan.cpp:122-174`, minus the three `cLib_addCalc2` calls
    /// that drive `partAlpha[i]` (`:131`), `partScale1[i]` (`:132`) and `partScale2[i]` (`:135`),
    /// the `partRotY[i] += 0x100` spin (`:133`) and the whole `field_0x5a4[i] == 0` emitter block
    /// (`:134-166`) - all of which feed `syan_draw` (`:59-67`) or the particle system.
    fn emitter(&mut self, index: usize) {
        if self.life[index] == 0 {
            self.life[index] = (rnd_f(&mut self.seed, LIFE_SPAN) + 5.0) as i16;
            self.alpha_target[index] = rnd_f(&mut self.seed, ALPHA_SPAN) + 4.0;
        }
        if self.grow[index] == 0 {
            self.grow[index] = (rnd_f(&mut self.seed, GROW_SPAN) + 3.0) as i16;
            self.scale_target[index] = rnd_f(&mut self.seed, GROW_BIAS) + 0.75;
        }
        if self.fire[index] == 0 {
            self.emitter_enabled[index] = true;
        }
        if self.life[index] != 0 {
            self.life[index] -= 1;
        }
        if self.grow[index] != 0 {
            self.grow[index] -= 1;
        }
        if self.fire[index] != 0 {
            self.fire[index] -= 1;
        }
    }
}

/// `cLib_addCalc2` (`c_lib.cpp:56-66`).
fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value != target {
        let step = (scale * (target - *value)).clamp(-max_step, max_step);
        *value += step;
    }
}

/// `cM_rnd` (`c_math.cpp:177-182`), with the three global lanes held on the component.
fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum =
        seed[0] as f32 / RND_SCALE[0] + seed[1] as f32 / RND_SCALE[1] + seed[2] as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

/// `cM_rndF(f)` (`c_math.cpp:185-187`).
fn rnd_f(seed: &mut [i32; 3], f: f32) -> f32 {
    rnd(seed) * f
}

pub fn is_syan(name: &str) -> bool {
    SYAN_ACTORS.contains(&name)
}

pub fn control(mut chandeliers: Query<&mut Syan>) {
    for mut chandelier in &mut chandeliers {
        chandelier.step();
    }
}