use bevy_ecs::prelude::*;

use super::events::{SimEvent, SimEvents};
use super::flags::MemBits;

/// The stage string is `dragon` (`d_stage.cpp:700`); `_arcname` is `"Dr"`
/// (`d_a_dr.cpp:54`, `:56`, `:60`, `:218`, `:230`, `:232`, `:234`, `:251`).
pub const DRAGON_ACTORS: [&str; 1] = ["dragon"];

/// `i_this->mCountDownTimers[0] = (s16)(200.0f + cM_rndF(200.0f))` (`d_a_dr.cpp:76`) - the two
/// ends of `cM_rndF`'s range.
const IDLE_MIN: f32 = 200.0; // <- d_a_dr.cpp:76
/// `cM_rndF(200.0f)` (`d_a_dr.cpp:76`).
const IDLE_RANGE: f32 = 200.0; // <- d_a_dr.cpp:76
/// `m0E = 10 * 30` in `daDr_HIO_c::daDr_HIO_c` (`d_a_dr.cpp:23`), the `mCountDownTimers[0]` seed of
/// `case 10` (`:94`).
const HIO_0E: i16 = 300; // <- d_a_dr.cpp:23
/// `ARRAY_SIZE(i_this->mCountDownTimers)` (`d_a_dr.cpp:190`) with `s16 mCountDownTimers[3]`
/// (`d_a_dr.h:21`).
const TIMER_COUNT: usize = 3; // <- d_a_dr.h:21 (d_a_dr.cpp:190)
/// `JA_SE_CM_DRG_MTOP_FIRE` (`JAZelAudio_SE.h:1699`).
const SE_MTOP_FIRE: u32 = 0x61AB; // <- JAZelAudio_SE.h:1699
/// `cM_rndF`'s three lanes, copied from `cM_randInit`: the multipliers, the moduli and the
/// divisors that turn each lane into `rand.s32F()`'s `0..1` value. Same table `kn.rs:78-92` uses.
const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA]; // <- d_a_dr.cpp:76 (cM_rndF), kn.rs:78
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673]; // <- d_a_dr.cpp:76 (cM_rndF), kn.rs:79
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0]; // <- d_a_dr.cpp:76 (cM_rndF), kn.rs:80
const RND_SEED: i32 = 100; // <- d_a_dr.cpp:76 (cM_rndF), kn.rs:81

/// `switch (i_this->mMode)` of `move` (`d_a_dr.cpp:71-158`). `dr_class::mMode` is a plain `u8`
/// (`d_a_dr.h:19`) with no enum in the decomp; these are the six values the switch names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// `case 0` (`d_a_dr.cpp:72-77`) - `WAIT1`, then arm the idle timer.
    Idle,
    /// `case 1` (`d_a_dr.cpp:78-84`) - waits for the timer, then `AKUBI1`.
    IdleWait,
    /// `case 2` (`d_a_dr.cpp:85-90`) - `AKUBI1` playing; returns to `Idle` on `isStop()`.
    Akubi,
    /// `case 10` (`d_a_dr.cpp:91-97`) - `BIKU1` plus the `DRPAINBIKU00` breath emitter, and falls
    /// through into `BikuWait`.
    Biku,
    /// `case 11` (`d_a_dr.cpp:98-139`) - the `isStop()` branch that picks `ABARE1`/`ABARE2`/`HO1`.
    BikuWait,
    /// `case 12` (`d_a_dr.cpp:140-157`) - `HO1`, back to `Idle` on `isStop()`.
    Ho,
}

/// The dragon - Valoo on the Overworld, the Dragon Roost Cavern's resident on DRC.
///
/// `dr_class` (`d_a_dr.h:11-27`) registers no `dCcD_Src*` table at all: `daDr_Create` (`:246-267`)
/// touches only `mPhs`, `mpMorf` and `l_HIO`, so there is no AT collider and no Tg collider in
/// either direction, and the class has no health field. The profile `Group` is `fopAc_ACTOR_e`
/// (`:290`). (a) inbound damage: NONE. (b) outbound damage: NONE. Neither `combat::HitPoints` nor
/// `enemy::Hurt` is attached.
///
/// The profile `Cull Type` is `fopAc_CULLBOX_CUSTOM_e` (`:291`) and the file never calls
/// `fopAcM_setCullSizeBox`, so no `cull()` is derivable - `daDr_setMtx` (`:176-186`) only does
/// `setBaseScale`/`transS`/`YrotM`/`XrotM`/`ZrotM`/`setBaseTRMtx`/`calc`, all renderer work.
///
/// `daDr_Create` can only return `cPhs_ERROR_e` from `createHeap` (`:255-257`) or `dComIfG_resLoad`,
/// never from a parameter - and the file reads no parameter - and `daDr_Execute` ends on an
/// unconditional TRUE (`:207`), so there is no spawn-then-despawn path. `daDr_IsDelete` (`:211-214`)
/// is the bare "accepts a delete request" form of RULE 6b.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Dragon {
    /// `mMode` (`d_a_dr.h:19`).
    pub mode: Mode,
    /// `mCountDownTimers` (`d_a_dr.h:21`), `s16[3]`.
    pub timers: [i16; TIMER_COUNT],
    /// `m2C8` (`d_a_dr.h:24`), set by `dr_class::setQuakeInfo()` (`d_a_dr.h:13`) - the quake
    /// interrupt that jumps the dragon out of its idle cycle.
    pub quake: bool,
    /// `cM_rndF`'s three-lane state, re-seeded per actor the way `cM_randInit(0)` would.
    seed: [i32; 3],
}

impl Dragon {
    /// `daDr_Create` (`:246-267`), minus `fopAcM_ct_Retail`/`fopAcM_ct_Demo`, `dComIfG_resLoad`,
    /// `fopAcM_entrySolidHeap`/`createHeap`, `daDr_setMtx` and the HIO child. The actor's initial
    /// `mMode` is 0 (zero-initialised process memory, and `case 0` at `:72` is the arm a fresh
    /// `dr_class` enters).
    pub fn spawn() -> Self {
        Self {
            mode: Mode::Idle,
            timers: [0; TIMER_COUNT],
            quake: false,
            seed: [RND_SEED; 3],
        }
    }

    /// `cM_rndF(range)` (`d_a_dr.cpp:76`, `:111`) over `cM_randInit`'s three lanes, the same
    /// reduction `kn.rs:83-94` and `molgera.rs:205-262` use.
    fn rnd(&mut self, range: f32) -> f32 {
        for lane in 0..3 {
            self.seed[lane] = (self.seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
        }
        let sum = (self.seed[0] as f32) / RND_SCALE[0]
            + (self.seed[1] as f32) / RND_SCALE[1]
            + (self.seed[2] as f32) / RND_SCALE[2];
        (sum % 1.0).abs() * range
    }

    /// `case 0` of `move` (`d_a_dr.cpp:72-77`): `anm_init(BCK_DR_WAIT1_e, l_HIO.mWait1Morf,
    /// EMode_LOOP, 1.0f, -1)` - `anm_init` (`:48-66`) is entirely `mpMorf->setAnm` plus the
    /// `mCurrBckIdx` bookkeeping that only feeds `anm_init`'s own blend rate, so it is renderer
    /// work. `l_HIO.mWait1Morf = 10.0f` (`:24`) is a blend rate and goes with it. What remains is
    /// `mMode++` and the randomised idle timer.
    fn case_idle(&mut self) {
        self.mode = Mode::IdleWait;
        self.timers[0] = (IDLE_MIN + self.rnd(IDLE_RANGE)) as i16;
    }

    /// `case 1` of `move` (`d_a_dr.cpp:78-84`): `isIdle = true`, and on `mCountDownTimers[0] == 0`
    /// an `anm_init(BCK_DR_AKUBI1_e, l_HIO.mAkubi1Morf, EMode_NONE, 1.0f, BAS_AKUBI1)` plus
    /// `mMode++`. The blend rate again goes with the animation; the transition does not.
    fn case_idle_wait(&mut self) {
        if self.timers[0] == 0 {
            self.mode = Mode::Akubi;
        }
    }

    /// `case 10` of `move` (`d_a_dr.cpp:91-97`), which falls through into `case 11` (`:98`):
    /// `anm_init(BCK_DR_BIKU1_e, ...)`, `mMode++`, `mCountDownTimers[0] = l_HIO.m0E`, the
    /// `ID_AK_SN_DRPAINBIKU00` emitter (particles) and `m2C9 = 0`.
    fn case_biku(&mut self) {
        self.timers[0] = HIO_0E;
        self.mode = Mode::BikuWait;
    }

    /// The post-switch block of `move` (`d_a_dr.cpp:160-166`):
    /// `if (!dComIfGs_isStageBossEnemy(dSv_save_c::STAGE_DRC)) { if ((isIdle && (l_HIO.m0C ||
    /// dComIfGp_getVibration().CheckQuake())) || i_this->m2C8 != 0) { l_HIO.m0C = false;
    /// i_this->m2C8 = 0; i_this->mMode = 10; } }`.
    ///
    /// The `isIdle` local the switch arms 0, 1 and 2 set (`d_a_dr.cpp:73`, `:79`, `:86`) folds away
    /// here. `l_HIO.m0C` is `false` from `daDr_HIO_c`'s constructor (`:22`) and nothing but the
    /// host-IO widget ever writes it true, and `dVibration_c::CheckQuake()`
    /// (`d_vibration.cpp:337`) is the rumble channel's quake-pattern state - `mCamera.mQuake` /
    /// `mMotor.mQuake` pattern indices, driven only by `StartQuake` calls from other actors, which
    /// is the rumble input RULE 3b lists as absent. So `isIdle && (l_HIO.m0C || CheckQuake())` is
    /// a proven false and the whole disjunction reduces to `m2C8 != 0`.
    ///
    /// `dComIfGs_isStageBossEnemy(int)` (`d_com_inf_game.cpp:826-834`) reads the DRC save-slot bit
    /// unless DRC is the current stage's save table, in which case it reads the global memory bit -
    /// the same pair `MemBits::is_stage_boss_enemy` is the sim's single model of, as
    /// `gohdan.rs:782-783` and `kalle_demos.rs:2692` already do.
    /// `(isIdle && (l_HIO.m0C || dComIfGp_getVibration().CheckQuake())) || i_this->m2C8 != 0`
    /// (`d_a_dr.cpp:161`). `m0C` is false from the HIO constructor (`:22`) with no writer
    /// outside the host-IO widget, so the live term is `CheckQuake()`. `m2C8` is set only by
    /// `dr_class::setQuakeInfo` from `d_a_tag_event.cpp:191`, which this sim has no path to.
    fn quake_gate(&mut self, mem: &MemBits, vibe: &super::vibration::Vibration) {
        let idle = matches!(self.mode, Mode::Idle);
        if mem.is_stage_boss_enemy() {
            return;
        }
        if !((idle && vibe.check_quake()) || self.quake) {
            return;
        }
        self.quake = false;
        self.mode = Mode::Biku;
    }

    /// `move` (`d_a_dr.cpp:69-173`) minus every `anm_init`, every `mpBreathEmitter` particle
    /// call, every joint-matrix lookup (`getAnmMtx(DR1_JNT_J_DR_SITA2_e)` at `:136`, `:142`, `:202`;
    /// `getAnmMtx(DR1_JNT_DR_ALL_ROOT_e)` at `:120`), `mpMorf->play` (`:168`), and every
    /// `mpMorf->isStop()` / `mpMorf->getFrame()` test (`:87`, `:99`, `:103`, `:146`, `:150`) -
    /// BCK frame counts and end frames are not in this checkout (RULE 3b). Those tests gate
    /// `case 2 -> Idle` (`:88`), the whole `isStop()` body of `case 11` (`:103-133`) and
    /// `case 12 -> Idle` (`:151`). The omitted `case 11` body is the only writer of
    /// `mCountDownTimers[1]` other than the frame tests, and it holds the
    /// `cM_rndF(1.0f) < 0.5f` ABARE1/ABARE2 split (`:111`), `mCountDownTimers[1] = 500` (`:114`),
    /// `mCountDownTimers[1] = 0` (`:126`) and `fopAcM_seStart(JA_SE_CM_DRG_MTOP_MAGMA)` (`:125`),
    /// so the `JA_SE_CM_DRG_MTOP_FIRE` of `:170-172` cannot fire here.
    fn step(&mut self, mem: &MemBits, vibe: &super::vibration::Vibration, events: &mut SimEvents) {
        match self.mode {
            Mode::Idle => self.case_idle(),
            Mode::IdleWait => self.case_idle_wait(),
            Mode::Akubi => {}
            Mode::Biku => self.case_biku(),
            Mode::BikuWait | Mode::Ho => {}
        }
        self.quake_gate(mem, vibe);
        // `mpMorf->play(&i_this->current.pos, 0, 0)` (`:168`) is renderer work.
        if self.timers[1] != 0 {
            events.list.push(SimEvent::Se(SE_MTOP_FIRE));
        }
    }

    /// `daDr_Execute` (`:189-208`). The countdown loop is
    /// `for (int i = 0; i < ARRAY_SIZE(i_this->mCountDownTimers); i++) if (mCountDownTimers[i] !=
    /// 0) mCountDownTimers[i]--;` (`:190-194`) - a saturating decrement that stops at 0, which is
    /// what makes `case 1`'s `== 0` test and `case 11`'s `!= 0` test latch.
    /// `i_this->scale.x = scale.y = scale.z = l_HIO.mScale` (`:198`) with
    /// `mScale = 1.0f` (`:21`) is retail scale 1.0 (RULE 2); `daDr_setMtx` (`:200`) and the tongue
    /// joint's `eyePos` (`:202-205`) are renderer and joint-matrix work.
    fn execute(
        &mut self,
        mem: &MemBits,
        vibe: &super::vibration::Vibration,
        events: &mut SimEvents,
    ) {
        for timer in &mut self.timers {
            if *timer != 0 {
                *timer -= 1;
            }
        }
        self.step(mem, vibe, events);
    }
}

pub fn is_dragon(name: &str) -> bool {
    DRAGON_ACTORS.contains(&name)
}

pub fn control(
    mem: Res<MemBits>,
    vibe: Res<super::vibration::Vibration>,
    mut events: ResMut<SimEvents>,
    mut dragons: Query<&mut Dragon>,
) {
    for mut dragon in &mut dragons {
        dragon.execute(&mem, &vibe, &mut events);
    }
}
