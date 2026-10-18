use bevy_ecs::prelude::*;

pub const BRANCH_ACTORS: [&str; 1] = ["Branch"];

pub const CULL_MIN: [f32; 3] = [0.0, 0.0, -50.0]; // <- d_a_branch.cpp:209
pub const CULL_MAX: [f32; 3] = [300.0, 100.0, 50.0]; // <- d_a_branch.cpp:209

/// `daBranch_c` (`d_a_branch.h:8-43`): the Forest of Fairies branch Tetra swings from. The stage
/// string is `Branch` (`d_stage.cpp:848`) and the archive is `Kwood_00`
/// (`d_a_branch.cpp:21`) - the STAGE string differs from the `_arcname`, and the STAGE string is
/// what belongs in `BRANCH_ACTORS`.
///
/// Retail never runs this actor's behaviour. `create` (`:193-231`) calls `fopAcM_ct_Retail`
/// (`:194`) and then `fopAcM_ct_Demo` (`:198`), and in a retail build `fopAcM_ct_Demo` expands to
/// nothing (`f_op_actor_mng.h:25`), so `demoActorID` (`f_op_actor.h:281`) stays 0. The whole
/// `else` arm of `execute` (`:155-168`) - the only thing that can change `m02B8` - is therefore
/// dead, and `m02B8` keeps the `6` that `:211` wrote (the `5` of `:124`/`:146`/`:164` and the `6` of
/// `:151`/`:161` are the two shape ids, and nothing ever selects between them). What remains is:
///
///   * `execute` (`:142-173`): `m02B8 == 5` plays `mAnims[1]` and `m02B8 == 6` plays `mAnims[0]`
///     (`:146-153`) - animation only - followed by `set_mtx` (`:170`), which is matrix setup.
///   * `draw` (`:122-134`): picks `mModel[0]` or `mModel[1]` from `m02B8` and calls `updateDL`.
///   * `CreateHeap` (`:70-120`): two `mDoExt_McaMorf` allocations, `set_anim` (`:42-50`) and
///     `demoPlay` (`:53-61`) - all archive and appearance work.
///   * `create`'s per-material `getFog()->setType(2)` loop (`:217-226`), and `m02BC`/`m02BD`
///     (`:212-213`), which are written once and read nowhere in the decomp.
///
/// `create` does supply a real `fopAcM_setCullSizeBox` (`:209`), so the profile's
/// `fopAc_CULLBOX_CUSTOM_e` (`:265`) is answerable.
///
/// `daBranch_c` registers no `dCcD` collider - it is a plain `fopAc_ac_c` (`:10`) whose only
/// per-frame calls are `setBaseMatrix` and `updateDL` - so it has no inbound damage path.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Branch;

impl Branch {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_branch(name: &str) -> bool {
    BRANCH_ACTORS.contains(&name)
}

pub fn control(_branches: Query<&Branch>) {}
