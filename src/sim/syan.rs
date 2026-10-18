use bevy_ecs::prelude::*;

use super::stream;

/// The stage string is `SYAN` (`d_stage.cpp:1228`, `fpcNm_SYAN_e`), NOT the `_arcname` `Syan`
/// (`d_a_syan.cpp:197`, `:199`, `:209`, `:187`).
pub const SYAN_ACTORS: [&str; 1] = ["SYAN"];

/// `enum { kEmtrNum = 6 }` (`d_a_syan.h:12-14`), also the `ARRAY_SIZE` bound of every per-emitter
/// array (`d_a_syan.cpp:39`, `:59`, `:122`, `:188`, `:223`).
const EMITTER_NUM: usize = 6;
/// `s16 timer[2]` (`d_a_syan.h:29`), the `ARRAY_SIZE(i_this->timer)` bound at `d_a_syan.cpp:81`.
const TIMER_SLOTS: usize = 2;
/// `cM_rndF(REG0_F(2) + 10.0f) + 5.0f + REG0_F(3)` (`d_a_syan.cpp:124`), `REG0_F` folded per RULE 2b.
const LIFE_SPAN: f32 = 10.0;
/// `cM_rndF(6.0f) + 3.0f` (`d_a_syan.cpp:128`).
const GROW_SPAN: f32 = 6.0;
/// `fopAcM_SetMin(i_this, -1000.0f, -5000.0f, -1000.0f)` (`d_a_syan.cpp:216`).
const CULL_MIN: [f32; 3] = [-1000.0, -5000.0, -1000.0];
/// `fopAcM_SetMax(i_this, 1000.0f, 5000.0f, 1000.0f)` (`d_a_syan.cpp:217`).
const CULL_MAX: [f32; 3] = [1000.0, 5000.0, 1000.0];
/// `cM_initRnd(100, 100, 100)` (`m_Do_machine.cpp:617`) - the only seed `cM_rnd`'s three static
/// lanes (`c_math.cpp:167`) are ever given, so it is the proven startup state.
const RND_SEED: i32 = 100;
/// `cM_rnd` multipliers (`c_math.cpp:178-180`).
const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
/// `cM_rnd` moduli (`c_math.cpp:178-180`).
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
/// `cM_rnd` reciprocals (`c_math.cpp:181`).
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];

/// `i_this->state` (`d_a_syan.h:20`) is the three-way dispatch at `d_a_syan.cpp:94-113`. Only
/// `case 0` is ever entered in retail: `daSyan_Create` (`:206-230`) never writes `state`, and the
/// sole writer `i_this->state++` (`:103`) sits in `case 1`. Both other arms are transcribed anyway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// `case 0: break` (`d_a_syan.cpp:95-96`).
    Idle,
    /// `case 1` (`d_a_syan.cpp:97-104`).
    Swing,
    /// `case 2` (`d_a_syan.cpp:105-112`).
    Wide,
}

/// The unused Spaceworld-2001-trailer chandelier: six alpha-sphere "candle flames" hung off
/// joints 5 and 8 of a `McaMorf` model, driven entirely through `nodeCallBack`'s matrix maths.
/// `syan_class` (`d_a_syan.h:10-46`) has no collider and `d_a_syan.cpp` never calls
/// `dComIfG_Ccsp()`, so it cannot be hit and cannot hit; the profile `Group` is `fopAc_ACTOR_e`
/// (`:253`), not `fopAc_ENEMY_e`. Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
///
/// `daSyan_Create` (`:206-230`) can only return `cPhs_ERROR_e` from `fopAcM_entrySolidHeap` on a
/// heap allocation failure (`:211`, `:226`), never from its parameters, so there is no
/// spawn-then-despawn path. `daSyan_IsDelete` (`:180-183`) is a bare TRUE - it accepts a delete
/// request, it is not a self-destruct.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Syan {
    /// `u32 state` (`d_a_syan.h:20`).
    pub state: State,
    /// `s16 timer[2]` (`d_a_syan.h:29`), decremented at `d_a_syan.cpp:81-83`, read at `:102`.
    pub timer: [i16; TIMER_SLOTS],
    /// `f32 field_0x2b8` (`d_a_syan.h:22`) - the joint-8 yaw/roll amplitude.
    pub sway_amp: f32,
    /// `f32 field_0x2bc` (`d_a_syan.h:23`) - the joint-5 pitch amplitude.
    pub swing_amp: f32,
    /// `f32 field_0x2c0` (`d_a_syan.h:24`) - the joint-8 pitch amplitude.
    pub tilt_amp: f32,
    /// `csXyz field_0x2b0.x` (`d_a_syan.h:21`), the `s16` chased by `cLib_addCalcAngleS2`.
    pub sway_pitch: i16,
    /// `f32 field_0x2dc` (`d_a_syan.h:31`), read at `d_a_syan.cpp:108-111`. No function in this
    /// file ever writes it, so it is the placement-time zero.
    pub wide_scale: f32,
    /// `s16 field_0x58c[i]` (`d_a_syan.h:42`) - the alpha-ramp life counter.
    pub life: [i16; EMITTER_NUM],
    /// `s16 field_0x598[i]` (`d_a_syan.h:43`) - the scale-ramp life counter.
    pub grow: [i16; EMITTER_NUM],
    seed: [i32; 3],
}

impl Syan {
    /// `daSyan_Create` (`d_a_syan.cpp:206-230`): `i_this->field_0x2b8 = 200.0f` (`:220`). Everything
    /// else there (`fopAcM_SetMin`/`SetMax`, `fopAcM_SetMtx`, the joint callbacks, the `McaMorf`)
    /// is renderer or heap work. `state`, `timer`, `field_0x2b0`, `field_0x2bc`, `field_0x2c0`,
    /// `field_0x2c4`, `field_0x2c8`, `field_0x2cc`, `field_0x2d0`, `field_0x2dc`, `field_0x52c`,
    /// `field_0x55c`, `field_0x58c`, `field_0x598` and `emtrEnabled` all start at the placement
    /// zero (`fopAcM_ct`, `:207`, runs the empty `fopAc_ac_c::fopAc_ac_c()`, `f_op_actor.cpp:106`).
    pub fn spawn() -> Self {
        Self {
            state: State::Idle,
            timer: [0; TIMER_SLOTS],
            sway_amp: 200.0,
            swing_amp: 0.0,
            tilt_amp: 0.0,
            sway_pitch: 0,
            wide_scale: 0.0,
            life: [0; EMITTER_NUM],
            grow: [0; EMITTER_NUM],
            seed: [RND_SEED; 3],
        }
    }

    /// `fopAcM_SetMin(i_this, -1000.0f, -5000.0f, -1000.0f)` (`d_a_syan.cpp:216`) and
    /// `fopAcM_SetMax(i_this, 1000.0f, 5000.0f, 1000.0f)` (`:217`) - a real custom cull box.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `case 1` (`d_a_syan.cpp:97-104`), `REG0_S`/`REG0_F` folded per RULE 2b.
    fn state_swing(&mut self) {
        stream::add_calc_angle_s2(&mut self.sway_pitch, 2500, 2, 250);
        add_calc2(&mut self.swing_amp, 1500.0, 0.5, 150.0);
        add_calc2(&mut self.tilt_amp, 2500.0, 0.5, 250.0);
        add_calc2(&mut self.sway_amp, 2000.0, 0.5, 200.0);
        if self.timer[0] == 0 {
            self.state = State::Wide;
        }
    }

    /// `case 2` (`d_a_syan.cpp:105-112`), `REG0_S`/`REG0_F` folded per RULE 2b. The two
    /// `field_0x2cc`/`field_0x2d0` chases at `:106-107` are omitted with them.
    fn state_wide(&mut self) {
        let swing_target = self.wide_scale * 300.0;
        add_calc2(&mut self.swing_amp, swing_target, 0.1, 10.0);
        let tilt_target = self.wide_scale * 300.0;
        add_calc2(&mut self.tilt_amp, tilt_target, 0.1, 10.0);
        stream::add_calc_angle_s2(&mut self.sway_pitch, 2500, 2, 2000);
        let sway_target = self.wide_scale * 500.0;
        add_calc2(&mut self.sway_amp, sway_target, 0.2, 5.0);
    }

    /// The per-emitter head of `d_a_syan.cpp:122-174`: the `field_0x58c`/`field_0x598` re-randomise
    /// (`:123-130`) and the three counter decrements (`:168-173`).
    fn emitter(&mut self, index: usize) {
        if self.life[index] == 0 {
            self.life[index] = (rnd_f(&mut self.seed, LIFE_SPAN) + 5.0) as i16;
        }
        if self.grow[index] == 0 {
            self.grow[index] = (rnd_f(&mut self.seed, GROW_SPAN) + 3.0) as i16;
        }
        if self.life[index] != 0 {
            self.life[index] -= 1;
        }
        if self.grow[index] != 0 {
            self.grow[index] -= 1;
        }
    }

    /// `daSyan_Execute` (`d_a_syan.cpp:80-177`) minus `i_this->morf->calc()` (`:85`), the sine block
    /// (`:87-92`), the model matrix block (`:115-120`), `syan_draw` (`:52-68`) and every particle
    /// call (`:139`, `:163`).
    fn step(&mut self) {
        for slot in self.timer.iter_mut() {
            if *slot != 0 {
                *slot -= 1;
            }
        }
        match self.state {
            State::Idle => {}
            State::Swing => self.state_swing(),
            State::Wide => self.state_wide(),
        }
        for index in 0..EMITTER_NUM {
            self.emitter(index);
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

/// `cM_rnd` (`c_math.cpp:177-182`), with the three static lanes (`c_math.cpp:167`) held per actor.
fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = seed[0] as f32 / RND_SCALE[0]
        + seed[1] as f32 / RND_SCALE[1]
        + seed[2] as f32 / RND_SCALE[2];
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
