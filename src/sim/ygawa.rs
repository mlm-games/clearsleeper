use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;

/// The stage string is `yg_awa` (`d_stage.cpp:781`, `fpcNm_YOUGAN_e`), NOT the `_arcname`
/// `Yougan` (`d_a_yougan.cpp:34`, `:165`, `:175`, `:177`, `:179`, `:221`).
pub const YGAWA_ACTORS: [&str; 1] = ["yg_awa"];

/// `s32 yg_awa_num = 16` (`d_a_yougan.cpp:33`), kept in step with `l_HIO.field_0x06 = 16`
/// (`:40`, `:58`, `:93`, `:138`, `:182`) and the `field_0x298[16]` array (`d_a_yougan.h:34`).
pub const AWA_NUM: usize = 16;
/// `l_HIO.field_0x08 = 1250.0f` (`d_a_yougan.cpp:41`) - the launch disc radius, times `scale.x`.
const LAUNCH_RADIUS: f32 = 1250.0;
/// `l_HIO.field_0x0C = 40.0f` (`d_a_yougan.cpp:42`) - the base grow time.
const GROW_BASE: f32 = 40.0;
/// `l_HIO.field_0x10 = 60.0f` (`d_a_yougan.cpp:43`) - the `cM_rndF` spread added to `field_0x0C`.
const GROW_SPREAD: f32 = 60.0;
/// `l_HIO.field_0x14 = 0.2f` (`d_a_yougan.cpp:44`) - the base target scale.
const SCALE_BASE: f32 = 0.2;
/// `l_HIO.field_0x18 = 0.5f` (`d_a_yougan.cpp:45`) - the `cM_rndF` spread added to `field_0x14`.
const SCALE_SPREAD: f32 = 0.5;
/// `field_0x298[i].field_0x2C.y = current.pos.y - 17.591f` (`d_a_yougan.cpp:106`).
const LAUNCH_DROP: f32 = 17.591;
/// `cM_initRnd(100, 100, 100)` (`m_Do_machine.cpp:617`).
const RND_SEED: i32 = 100;
/// `cM_rnd` multipliers (`c_math.cpp:178-180`).
const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
/// `cM_rnd` moduli (`c_math.cpp:178-180`).
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
/// `cM_rnd` reciprocals (`c_math.cpp:181`).
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
/// `fopAc_CULLBOX_0_e` (`d_a_yougan.cpp:258`) -> `f_op_actor_mng.cpp:558`.
const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0];
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0];

/// `field_0x298[i].field_0x00` (`d_a_yougan.h:12`) is the three-way switch of
/// `_daYougan_execute` (`d_a_yougan.cpp:96-136`), plus the `default` arm at `:133-135`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bubble {
    /// `case 0` (`d_a_yougan.cpp:97-110`) - pick the launch point, grow rate and target scale.
    Ready,
    /// `case 1` (`:111-119`) - swell the model scale up to the target.
    Swell,
    /// `case 2` (`:120-132`) - play the BCK/BTK pair out, then recycle.
    Pop,
}

/// One `yg_awa_data` (`d_a_yougan.h:8-20`) of the sixteen lava bubbles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Awa {
    /// `field_0x298[i].field_0x00` (`d_a_yougan.h:12`).
    pub state: Bubble,
    /// `cXyz field_0x2C` (`d_a_yougan.h:16`) - the bubble's world position.
    pub pos: Vec3,
    /// `cXyz field_0x38` (`d_a_yougan.h:17`) - the live model scale.
    pub scale: Vec3,
    /// `f32 field_0x44` (`d_a_yougan.h:18`) - the target scale.
    pub target: f32,
    /// `f32 field_0x48` (`d_a_yougan.h:19`) - the frames the swell takes.
    pub grow: f32,
}

impl Awa {
    /// `case 0` of `_daYougan_execute` (`d_a_yougan.cpp:97-110`):
    /// `field_0x38 = cXyz::Zero` (`:98`),
    /// `field_0x48 = l_HIO.field_0x0C + cM_rndF(l_HIO.field_0x10)` (`:99`),
    /// `field_0x44 = l_HIO.field_0x14 + cM_rndF(l_HIO.field_0x18)` (`:100`),
    /// `fVar4 = l_HIO.field_0x08 * scale.x` (`:101`),
    /// `fVar5 = cM_rndFX(fVar4)` (`:102`), `fVar4 = cM_rndFX(fVar4 - fabsf(fVar5))` (`:103`),
    /// then the three position writes (`:105-107`).
    fn ready(&mut self, origin: Vec3, seed: &mut [i32; 3]) {
        self.scale = Vec3::ZERO;
        self.grow = GROW_BASE + rnd_f(seed, GROW_SPREAD);
        self.target = SCALE_BASE + rnd_f(seed, SCALE_SPREAD);
        // `fVar4 = l_HIO.field_0x08 * scale.x` (`d_a_yougan.cpp:101`); stage-placed actor scale is
        // exactly 1.0 in retail (RULE 2), so the launch disc is the bare `1250.0f`.
        let spread = LAUNCH_RADIUS;
        let x = rnd_fx(seed, spread);
        let z = rnd_fx(seed, spread - x.abs());
        self.pos = Vec3::new(origin.x + x, origin.y - LAUNCH_DROP, origin.z + z);
        self.state = Bubble::Swell;
    }

    /// `case 1` of `_daYougan_execute` (`d_a_yougan.cpp:111-119`).
    fn swell(&mut self) {
        if self.scale.x <= self.target {
            let size = self.target / self.grow;
            self.scale.x += size;
            self.scale.y = self.scale.x;
            self.scale.z = self.scale.x;
        } else {
            self.state = Bubble::Pop;
        }
    }

    /// `case 2` of `_daYougan_execute` (`d_a_yougan.cpp:120-132`) minus the four animator calls
    /// `setPlaySpeed(1.0f)` (`:121-122`) and the `field_0x08.play() || field_0x18.play()` test
    /// with its `setFrame(0.0f)` reset (`:124-127`): the BCK/BTK frame counts are not in this
    /// checkout (RULE 3b), so the recycle-back-to-`case 0` branch cannot be evaluated. The
    /// `field_0x38.set(0.0f, 0.0f, 0.0f)` and `field_0x00 = 0` writes of `:129-130` live in that
    /// omitted branch, so a bubble that reaches `case 2` stays there.
    fn pop(&mut self) {
        self.state = Bubble::Pop;
    }
}

/// The unused lava-bubble pool. `daYougan_c` (`d_a_yougan.h:22-35`) creates no actor collider and
/// `d_a_yougan.cpp` never calls `dComIfG_Ccsp()`, so it can neither be hit nor hit anything; the
/// profile `Group` is `fopAc_ACTOR_e` (`:255`). `_daYougan_isdelete` (`:154-156`) is the bare-TRUE
/// form, i.e. "accepts a delete request", never a self-destruct (RULE 6b).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct YgAwa {
    /// `field_0x298[16]` (`d_a_yougan.h:34`), sized by `yg_awa_num` (`d_a_yougan.cpp:33`).
    pub bubbles: [Awa; AWA_NUM],
    seed: [i32; 3],
}

impl YgAwa {
    /// `daYougan_c::useHeapInit` (`d_a_yougan.cpp:174-204`): `yg_awa_num = l_HIO.field_0x06` (`:182`)
    /// and `field_0x298[i].field_0x00 = 0` (`:184`) seed all sixteen bubbles. The model creation
    /// (`:185`) and the three `mDoExt_*Anm.init` calls (`:194`, `:199`) are heap work, and their
    /// `FALSE` returns would make `_daYougan_create` return `cPhs_ERROR_e` (`:227`) - an
    /// archive-load failure, not a parameter, so `spawn` returns `Self`.
    pub fn spawn() -> Self {
        let ready = Awa {
            state: Bubble::Ready,
            pos: Vec3::ZERO,
            scale: Vec3::ZERO,
            target: 0.0,
            grow: 0.0,
        };
        Self {
            bubbles: [ready; AWA_NUM],
            seed: [RND_SEED; 3],
        }
    }

    /// The profile is `fopAc_CULLBOX_0_e` (`d_a_yougan.cpp:258`), so no `setCullSizeBox` is needed.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `_daYougan_execute` (`d_a_yougan.cpp:92-146`) minus the second matrix loop (`:138-144`) and
    /// `_daYougan_draw` (`:57-84`), both model-matrix work.
    fn step(&mut self, origin: Vec3) {
        for index in 0..AWA_NUM {
            match self.bubbles[index].state {
                Bubble::Ready => self.bubbles[index].ready(origin, &mut self.seed),
                Bubble::Swell => self.bubbles[index].swell(),
                Bubble::Pop => self.bubbles[index].pop(),
            }
        }
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

/// `cM_rndFX(f)` (`c_math.cpp:190-192`).
fn rnd_fx(seed: &mut [i32; 3], f: f32) -> f32 {
    f * (rnd(seed) - 0.5) * 2.0
}

pub fn is_ygawa(name: &str) -> bool {
    YGAWA_ACTORS.contains(&name)
}

pub fn control(mut pools: Query<(&Actor, &mut YgAwa)>) {
    for (actor, mut pool) in &mut pools {
        pool.step(actor.pos);
    }
}