use bevy_ecs::prelude::*;

/// The stage string is `yg_awa` (`d_stage.cpp:781`, `fpcNm_YOUGAN_e`), NOT the `_arcname` `Yougan`
/// (`d_a_yougan.cpp:34`, `:165`, `:175-179`, `:221`).
pub const YGAWA_ACTORS: [&str; 1] = ["yg_awa"];

/// `static s32 yg_awa_num = 16` (`d_a_yougan.cpp:33`), overwritten at `:93`, `:138` and `:182` by
/// `l_HIO.field_0x06`, whose `daYOUGAN_HIO_c` constructor sets it to `16` (`:40`) and whose only
/// writer is the HIO editor. It is also the `yg_awa_data field_0x298[16]` bound
/// (`d_a_yougan.h:34`).
const AWA_NUM: usize = 16;
/// `l_HIO.field_0x0C` (`d_a_yougan.cpp:42`) - the base rise rate.
const RISE_BASE: f32 = 40.0;
/// `l_HIO.field_0x10` (`d_a_yougan.cpp:43`) - the `cM_rndF` spread on the rise rate.
const RISE_SPREAD: f32 = 60.0;
/// `l_HIO.field_0x14` (`d_a_yougan.cpp:44`) - the base bubble size.
const SIZE_BASE: f32 = 0.2;
/// `l_HIO.field_0x18` (`d_a_yougan.cpp:45`) - the `cM_rndF` spread on the bubble size.
const SIZE_SPREAD: f32 = 0.5;
/// `cM_initRnd(100, 100, 100)` (`m_Do_machine.cpp:617`) - the only seed `cM_rnd`'s three static
/// lanes (`c_math.cpp:167`) are ever given, so it is the proven startup state.
const RND_SEED: i32 = 100;
/// `cM_rnd` multipliers (`c_math.cpp:178-180`).
const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
/// `cM_rnd` moduli (`c_math.cpp:178-180`).
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
/// `cM_rnd` reciprocals (`c_math.cpp:181`).
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];

/// `field_0x298[i].field_0x00` (`d_a_yougan.h:12`), the `switch` at `d_a_yougan.cpp:96-136`.
/// `useHeapInit` (`:184`) seeds every entry to 0 and only 0, 1 and 2 are ever assigned, so the
/// decomp's `default: field_0x298[i].field_0x00 = 0` (`:133-135`) is unreachable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// `case 0` (`d_a_yougan.cpp:97-110`).
    Idle,
    /// `case 1` (`d_a_yougan.cpp:111-119`).
    Rise,
    /// `case 2` (`d_a_yougan.cpp:120-132`).
    Fade,
}

/// One `yg_awa_data` entry (`d_a_yougan.h:8-20`). `J3DModel* field_0x04`, the two animation
/// controllers `field_0x08`/`field_0x18` and the model position `field_0x2C` are renderer or
/// particle data and are not carried.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Bubble {
    phase: Phase,
    /// `cXyz field_0x38` (`d_a_yougan.h:18`). Case 0 zeroes it (`:98`), case 1 accumulates its `x`
    /// and mirrors `x` into `y` and `z` (`:113-115`), and case 2 zeroes it again (`:129`) - it is
    /// the model's uniform scale, so only `x` carries information.
    rise: f32,
    /// `f32 field_0x44` (`d_a_yougan.h:18`) - the size the bubble grows to.
    limit: f32,
    /// `f32 field_0x48` (`d_a_yougan.h:19`) - the per-tick growth step.
    rate: f32,
}

/// The unused lava-bubble emitter: sixteen bubbles that spawn at the actor, swell until a random
/// size, then play their pop animation. `useHeapInit` (`d_a_yougan.cpp:174-204`) builds only
/// sixteen `J3DModel`s and their BCK/BTK controllers - there is no `dCcD` and `d_a_yougan.cpp`
/// never calls `dComIfG_Ccsp()`, so it registers no AT or Tg collider, cannot be damaged and
/// cannot damage anything. `daYougan_c` (`d_a_yougan.h:22-35`) has no health field and the profile
/// `Group` is `fopAc_ACTOR_e` (`d_a_yougan.cpp:256`), so neither `combat::HitPoints` nor
/// `enemy::Hurt` is attached.
///
/// `_daYougan_create` (`:218-234`) can only return `cPhs_ERROR_e` from `fopAcM_entrySolidHeap`
/// (`:224`, `:227`), never from its parameters, and `useHeapInit`'s FALSE returns (`:189-191`,
/// `:194-196`, `:198-201`) are all model/archive failures, so there is no spawn-then-despawn path.
/// `_daYougan_isdelete` (`:154-156`) returns TRUE - an accepted delete request, not a
/// self-destruct.
///
/// The profile `Cull Type` is `fopAc_CULLBOX_0_e` (`:258`), one of the fixed profile cull boxes, and
/// this file never calls `fopAcM_setCullSizeBox`, so no cull box is derivable and none is invented.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ygawa {
    seed: [i32; 3],
    bubbles: [Bubble; AWA_NUM],
}

/// `cM_rnd` (`c_math.cpp:177-182`), with the three static lanes (`c_math.cpp:167`) held per actor.
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

impl Ygawa {
    /// `useHeapInit` (`d_a_yougan.cpp:182-192`) is the only setup this actor does: it zeroes every
    /// `field_0x298[i].field_0x00` (`:184`). `l_HIO` (`:37-49`) is built once for the whole
    /// process and its values are folded into the constants above.
    pub fn spawn() -> Self {
        let bubble = Bubble {
            phase: Phase::Idle,
            rise: 0.0,
            limit: 0.0,
            rate: 0.0,
        };
        Self {
            seed: [RND_SEED; 3],
            bubbles: [bubble; AWA_NUM],
        }
    }

    /// `case 0` and `case 1` of `_daYougan_execute` (`d_a_yougan.cpp:97-119`).
    fn bubble(&mut self, index: usize) {
        match self.bubbles[index].phase {
            Phase::Idle => {
                let rate = RISE_BASE + rnd_f(&mut self.seed, RISE_SPREAD);
                let limit = SIZE_BASE + rnd_f(&mut self.seed, SIZE_SPREAD);
                let bubble = &mut self.bubbles[index];
                bubble.rise = 0.0;
                bubble.rate = rate;
                bubble.limit = limit;
                bubble.phase = Phase::Rise;
            }
            Phase::Rise => {
                let bubble = &mut self.bubbles[index];
                if bubble.rise <= bubble.limit {
                    bubble.rise += bubble.limit / bubble.rate;
                } else {
                    bubble.phase = Phase::Fade;
                }
            }
            Phase::Fade => {}
        }
    }

    /// `daYougan_c::_daYougan_execute` (`d_a_yougan.cpp:92-146`) minus `case 2`'s
    /// `setPlaySpeed`/`play()` body (`:121-131`, which needs the BCK/BTK frame counts), the
    /// `field_0x2C` placement block (`:101-107`, and with it the two `cM_rndFX` draws on
    /// `l_HIO.field_0x08 * scale.x` = `1250.0f`, `:101-104`, and `current.pos`) and the model matrix
    /// loop (`:138-144`).
    fn step(&mut self) {
        for index in 0..AWA_NUM {
            self.bubble(index);
        }
    }
}

pub fn is_ygawa(name: &str) -> bool {
    YGAWA_ACTORS.contains(&name)
}

pub fn control(mut gas: Query<&mut Ygawa>) {
    for mut emitter in &mut gas {
        emitter.step();
    }
}