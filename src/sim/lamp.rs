use bevy_ecs::prelude::*;

pub const LAMP_ACTORS: [&str; 1] = ["Lamp"];

const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)

/// The generic wall torch. The stage string is `Lamp` (`d_stage.cpp:919`) and the archive is
/// also `"Lamp"` (`d_a_lamp.cpp:124`), so `_arcname` and the STAGE string agree. The profile
/// is `fopAc_CULLBOX_0_e` (`d_a_lamp.cpp:223`), not a custom box.
///
/// `Create` (`:152`) calls `mStts.Init(0xff, 0xff, this)` and registers `sph_src` as the torch's
/// only collider, and that `dCcD_SrcSph` has `SrcObjAt Type = 0` (`:158`) and
/// `SrcObjAt Atp = 0` (`:159`) - no attack arm, and nothing at all registers against it - so the
/// torch cannot be damaged. Its `SrcObjTg Type` (`:161`) is a target mask only.
///
/// `daLamp_Execute` (`:22-107`) keeps no observable state once the renderer's inputs are gone.
/// `mCycleCtr` (`:23`) and `mLength` (`:27`, `:34`, `:36`) feed only `MtxRotX`/`MtxRotZ`
/// (`:40-42`), and the `mOto` latch (`:30`) exists only to choose between those two
/// `cLib_addCalc2` targets. `mLength`'s only other reader is `oppDist`/`adjDist` (`:39-41`),
/// also matrix-only. `mParticlePower` (`:56`, `:63`, `:65`) is read once, at `:72`, to build
/// `mInf.mPower`, and `dKy_plight_set`/`dKy_plight_cut` (`:192`, `:118`) are the light source,
/// which RULE 3b lists as absent. The sphere arm at `:76-104` only reacts to an actor whose AT
/// type includes `AT_TYPE_WIND` (`1 << 21`) or `AT_TYPE_UNK400000` (`1 << 22`)
/// (`d_cc_d.h:32-33`); no such emitter is ported as a target, so `mHitAngle`,
/// `mHitTimeoutLeft` and `mHitReactCurZ` never leave zero and their only consumer is
/// `mPa.getEmitter()->setDirection` (`:102`), i.e. a particle. `mParticleInit` (`:51-57`) is
/// the particle system's own one-shot.
///
/// `Create` (`:141-199`) therefore contributes nothing but the profile's cull box: it seeds
/// `mCycleCtr` with `cM_rndFX(0x8000)` (`:186`), runs `Execute` twice (`:188-190`) - the same
/// two ticks the scheduler would have given it - and hands `mInf` to the light. There is no
/// `fopAcM_setCullSizeBox` call anywhere in the translation unit, so the only geometry is the
/// profile's own `fopAc_CULLBOX_0_e` entry. Nothing here can be hurt, and the Tg arm reacts
/// only to emitters this sim does not port.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lamp;

impl Lamp {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_lamp(name: &str) -> bool {
    LAMP_ACTORS.contains(&name)
}

pub fn control(_lamps: Query<&Lamp>) {}
