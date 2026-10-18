use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::hookshot::{CAPSULE_RADIUS, Hookshot, Phase, SHOT_STEP};
use super::stream;
use super::walk;

/// The stage string is `Hfuck1` (`d_stage.cpp:1124`) and it is also the `_arcname`
/// (`d_a_obj_hfuck1.cpp:21`, `:88`, `:95`, `:122`, `:163`).
pub const HFUCK1_ACTORS: [&str; 1] = ["Hfuck1"];

/// `cM3dGSphS.Radius` of `l_sph_src` (`d_a_obj_hfuck1.cpp:52`), restated by `mSph.SetR(90.0f)`
/// at `:144`.
const SPH_RADIUS: f32 = 90.0; // <- d_a_obj_hfuck1.cpp:52
/// `mDoMtx_stack_c::transM(0.0f, 0.0f, -50.0f)` (`d_a_obj_hfuck1.cpp:140`) - the sphere centre's
/// local offset before the `XYZrotM(shape_angle)` of `:139`.
const SPH_OFFSET_Z: f32 = -50.0; // <- d_a_obj_hfuck1.cpp:140
/// `static const Vec l_hook_offset = { 0.0f, 0.0f, 0.0f }` (`d_a_obj_hfuck1.cpp:56`), the offset
/// `daPy_py_c::setHookshotCarryOffset` is handed at `:209`.
const L_HOOK_OFFSET: Vec3 = Vec3::ZERO; // <- d_a_obj_hfuck1.cpp:56

/// The hookshot target ring.
///
/// `l_sph_src` (`d_a_obj_hfuck1.cpp:23-54`) has `Flags = 0` (`:26`), `SrcObjAt Type = 0` (`:27`),
/// `SrcObjAt Atp = 0` (`:28`), `SrcObjAt SPrm = 0` (`:29`) and `SrcObjCo SPrm = 0` (`:33`), so the
/// target deals no damage, registers no attack collider and exposes no body collider. Its
/// `SrcObjTg Type = AT_TYPE_HOOKSHOT` (`:30`) with
/// `cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsOther_e` (`:31`) is a trigger, not a health path.
/// `daObjHfuck1_c` (`d_a_obj_hfuck1.h:12-30`) has no health field and the profile `Group` is
/// `fopAc_ACTOR_e` (`d_a_obj_hfuck1.cpp:276`), so neither `combat::HitPoints` nor `enemy::Hurt`
/// is attached.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE.
///
/// `_create` (`:119-159`) can only return `cPhs_ERROR_e` from a missing archive, a `dBgW`
/// registration failure (`:127`), a failed `fopAcM_entrySolidHeap` (`:148`) or a failed
/// `dComIfG_resLoad` - never from a parameter - and `_execute` (`:191-219`) ends on an
/// unconditional TRUE, so there is no spawn-then-despawn path. `daObjHfuck1_IsDelete` (`:250-253`)
/// is the bare "accepts a delete request" form of RULE 6b.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hfuck1 {
    /// `mpHookshotActor` (`d_a_obj_hfuck1.h:29`). `checkCollision` stores the hookshot actor that
    /// hit the sphere (`:110`); the top of the next `_execute` consumes it and clears it
    /// (`:205-213`).
    pub hooked: Option<Entity>,
}

impl Hfuck1 {
    /// `daObjHfuck1_c::_create` (`:119-159`) minus `dComIfG_resLoad`, `fopAcM_entrySolidHeap`,
    /// `dComIfG_Bgsp()->Regist`, `fopAcM_SetMtx`, `init_mtx`, `mStts.Init`, `mSph.Set`,
    /// `mSph.SetStts`, `mSph.SetC`, `mSph.SetR` and the DEMO-only HIO child - archive, renderer
    /// and `dBgW` work. No parameter is read anywhere in the file, so none is stored.
    pub fn spawn() -> Self {
        Self { hooked: None }
    }

    /// `mSph.SetC(center)` where `center` is `multVecZero` of
    /// `transS(current.pos) * XYZrotM(shape_angle) * transM(0.0f, 0.0f, -50.0f)`
    /// (`d_a_obj_hfuck1.cpp:138-143`). `mDoMtx_XYZrotM` composes `Rz * Ry * Rx`
    /// (`m_Do_mtx.cpp:30-46`; each `mDoMtx_concat(mtx, R, mtx)` at `:34`/`:39`/`:44` is
    /// `mtx = mtx * R` through `MTXConcat`'s `c = a * b`) and `mDoMtx_multVecZero` returns
    /// column 3 (`m_Do_mtx.h:106-110`), so the offset is `XYZrotM * (0, 0, -50)`. Retail computes
    /// it once in Create - the `_execute` recompute at `:196-202` is DEMO-only - so only
    /// `current.pos` and `shape_angle` feed it, exactly the two `ebomzo.rs` reads.
    fn centre(pos: Vec3, angle: [i16; 3]) -> Vec3 {
        let (sx, cx) = (stream::sin_s(angle[0]), stream::cos_s(angle[0]));
        let (sy, cy) = (stream::sin_s(angle[1]), stream::cos_s(angle[1]));
        let (sz, cz) = (stream::sin_s(angle[2]), stream::cos_s(angle[2]));
        pos + Vec3::new(
            SPH_OFFSET_Z * (cz * sy * cx + sz * sx),
            SPH_OFFSET_Z * (sz * sy * cx - cz * sx),
            SPH_OFFSET_Z * (cy * cx),
        )
    }

    /// `checkCollision` (`:105-116`). Retail asks the registered sphere's `ChkTgHit()` and then
    /// `at->ChkAtType(AT_TYPE_HOOKSHOT)` on the hitting actor (`d_a_obj_hfuck1.cpp:30`, `:109`).
    /// The sim's hookshot is a `Hookshot` whose AT capsule is `CAPSULE_RADIUS` around `pos` and
    /// which advances by `heading * SHOT_STEP` while `Phase::Shot`; `AT_TYPE_HOOKSHOT` is the
    /// only type this sphere answers, and the sim's hookshot registers no other one, so the
    /// `ChkAtType` probe is a constant true over every phase in which retail keeps the hookshot's
    /// collider registered (`Phase::Wait` is the nocked state with no chain at all). Returns
    /// retail's `ret`, which the caller uses for `if (!checkCollision()) Set(&mSph)`.
    fn check_collision(
        &mut self,
        centre: Vec3,
        hooks: &mut Query<(Entity, &mut Hookshot)>,
    ) -> bool {
        let mut hit = None;
        for (entity, hook) in hooks.iter_mut() {
            if hook.phase == Phase::Wait {
                continue;
            }
            let travel = hook.pos + hook.heading * SHOT_STEP;
            if walk::swept(hook.pos, travel, centre, SPH_RADIUS + CAPSULE_RADIUS) {
                hit = Some(entity);
                break;
            }
        }
        if let Some(entity) = hit {
            self.hooked = Some(entity);
            return true;
        }
        false
    }

    /// `_execute` (`:191-219`) minus `mpBgW->Move()` (`:192` - `dBgW`, RULE 3b), `mStts.Move()`
    /// (`:193`) and `dComIfG_Ccsp()->Set(&mSph)` (`:216`), whose only observable effect in retail
    /// is to feed the next tick's `ChkTgHit`, which `check_collision` performs directly.
    ///
    /// The `:205-213` block runs BEFORE `checkCollision`, so a hook registered on tick N reaches
    /// `setHookshotCarryOffset` on tick N+1 and is cleared there. The sim's `Hookshot::hooked` is
    /// the single call that both binds the carried entity and installs `carry_offset`, and
    /// `l_hook_offset` is `(0, 0, 0)` (`d_a_obj_hfuck1.cpp:56`), so retail's two effects - the
    /// `fpcNm_HOOKSHOT_e` name gate at `:206` and the carry offset - collapse into that one call.
    fn step(&mut self, actor: &Actor, hooks: &mut Query<(Entity, &mut Hookshot)>) {
        if let Some(entity) = self.hooked.take()
            && let Ok((_, mut hook)) = hooks.get_mut(entity)
        {
            hook.hooked(entity, L_HOOK_OFFSET);
        }
        // `mSph.ClrTgHit()` (`d_a_obj_hfuck1.cpp:113`) has no modelled state.
        self.check_collision(Self::centre(actor.pos, actor.angle), hooks);
    }
}

pub fn is_hfuck1(name: &str) -> bool {
    HFUCK1_ACTORS.contains(&name)
}

pub fn control(
    mut hooks: Query<(Entity, &mut Hookshot)>,
    mut targets: Query<(&Actor, &mut Hfuck1)>,
) {
    for (actor, mut target) in &mut targets {
        target.step(actor, &mut hooks);
    }
}
