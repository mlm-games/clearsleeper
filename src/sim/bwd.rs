use bevy_ecs::prelude::*;

use super::actor::Actor;

/// `fpcNm_BWD_e`'s stage string: `OBJNAME("Bwd", fpcNm_BWD_e, 255, 0)` (`d_stage.cpp:1191`), which
/// is what `boss_a_d_sub`'s name test (`:160`) matches on.
const BWD_ACTOR: &str = "Bwd"; // <- d_stage.cpp:1191

/// The arena's sand-wave ground mesh - `bwdg_class` (`d_a_bwdg.h:17-45`), which `d_a_bwd.cpp:2172`
/// creates under the boss.
///
/// There is deliberately NO `BWD_ACTORS` array, NO `is_bwd` predicate and NO `spawn_actor` branch:
/// `fpcNm_BWDG_e` has **no row in the OBJNAME table** - `l_objectName` (`d_stage.cpp:436-...`)
/// contains no `OBJNAME(..., fpcNm_BWDG_e, ...)`, while `fpcNm_BWD_e` does at `:1191` - so no stage
/// can place a `BWDG`. Retail builds one at runtime with
/// `fopAcM_create(fpcNm_BWDG_e, 0, &actor->home.pos, fopAcM_GetRoomNo(actor), NULL, NULL, 0xff,
/// NULL)` (`d_a_bwd.cpp:2172`), and `d_a_bwd.cpp` is not ported here, so nothing in this sim ever
/// builds one. `spawn` and `control` exist for completeness and are unwired.
///
/// (a) inbound damage: NONE. (b) outbound damage: NONE - the `fopAc_ENEMY_e` profile `Group`
/// (`:268`) is a red herring: `bwdg_class` registers no `dCcD` AT, Tg or Co collider of any kind.
/// The damageable actor in this fight is `fpcNm_BWD_e` itself, which `boss_a_d_sub` (`:157-164`)
/// hunts for, not this mesh. Neither `combat::HitPoints` nor `enemy::Hurt` is attached.
///
/// Retail renders a 65 x 65 (`0x41` x `0x41`, `:79-84`) deforming sand height-field and registers
/// a `dBgWHf` moving background, so no `actor_views` branch is derivable: the geometry is the
/// `Bwdg` archive's own vertex buffer (RULE 3b, model/BCK/BRK/BTP archives) and the `dBgWHf`
/// registration is the absent moving-background subsystem RULE 3b names. For the same reason there
/// is no `cull()`: the profile's `fopAc_CULLBOX_0_e` (`d_a_bwdg.cpp:269`) does resolve through
/// `l_cullSizeBox[0]` (`f_op_actor_mng.cpp:558`) to (-40, 0, -40)..(40, 125, 40), but with no
/// stage-placed instance and no `actor_views` branch nothing would read it, and RULE 12b makes an
/// uncalled `pub fn` dead code.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bwdg {
    /// The cached `boss` static (`:15`), which `fpcM_Search(boss_a_d_sub, i_this)` (`:170`)
    /// resolves. `None` is the NULL both `daBwdg_Create` (`:228`) and the static start at.
    pub boss: Option<Entity>,
    /// `i_this->m002B4++` (`:175`) - advanced only once a boss has been found.
    pub frame: u32,
}

impl Bwdg {
    /// `daBwdg_Create` (`:222-245`), minus `fopAcM_ct` (`:224`), `dComIfG_resLoad` (`:226`),
    /// `fopAcM_entrySolidHeap` / `useHeapInit` (`:230`, `:202-219`),
    /// `dComIfG_Bgsp()->Regist(mpBgW, i_this)` (`:233`), the two `base_xz_set` calls (`:234`,
    /// `:236`), `wave_cont(i_this, 1)` (`:237`) and the four `dBgWHf` calls (`:239-241`) - archive,
    /// heap, moving-background and mesh work. What survives is `boss = NULL` (`:228`).
    pub fn spawn() -> Self {
        Self { boss: None, frame: 0 }
    }

    /// `daBwdg_Execute` (`:167-181`). While `boss == NULL` the function returns TRUE immediately
    /// (`:169-173`) and the wave never animates. Once a boss is cached, `m002B4` advances (`:175`)
    /// and the height field is re-solved into the vertex buffer - `mpBgW->CopyBackVtx()` (`:176`),
    /// `wave_cont(i_this, 0)` (`:177`), `mpBgW->SetVtx(posVtx)` (`:178`),
    /// `mpBgW->MoveHf()` (`:179`) - which is mesh plus `dBgWHf` work and is omitted. The function
    /// never deletes the actor.
    fn execute(&mut self, boss: Option<Entity>) {
        if self.boss.is_none() {
            self.boss = boss;
            if self.boss.is_none() {
                return;
            }
        }
        self.frame = self.frame.wrapping_add(1);
    }
}

pub fn control(mut waves: Query<(Entity, &mut Actor, Option<&mut Bwdg>)>) {
    // `boss_a_d_sub` (`:158-164`) accepts only `fopAcM_IsActor(param_1) && fopAcM_GetName(param_1)
    // == fpcNm_BWD_e`, and `fpcM_Search` (`:170`) takes the first match.
    let mut boss: Option<Entity> = None;
    for (entity, actor, wave) in &waves {
        if wave.is_none() && boss.is_none() && actor.name == BWD_ACTOR {
            boss = Some(entity);
        }
    }
    for (_, mut actor, wave) in &mut waves {
        if let Some(mut wave) = wave {
            wave.execute(boss);
            // `i_this->current.pos.setall(0.0f);` (:229) - the sand wave is pinned to the world
            // origin, discarding the `home.pos` that `d_a_bwd.cpp:2172` created it at.
            actor.pos = glam::Vec3::ZERO;
        }
    }
}