//! Falling rocks - `fpcNm_FallRock_e`, `d_a_fallrock.cpp`, stage string `fallrck`
//! (`d_stage.cpp:840`).
//!
//! (a) inbound damage: NONE modelled. `daFallRock_c` registers `mCyl` from `m_cyl_src`, whose
//! `SrcObjTg Type` is `AT_TYPE_ALL` with `cCcD_TgSPrm_Set_e | cCcD_TgSPrm_IsEnemy_e`
//! (`d_a_fallrock.cpp:25-26`) - the rock IS a sword target - but `execute` (`:101-165`) has no
//! `cc_at_check`, no `health` field and no damage reaction of any kind. Attaching
//! `combat::HitPoints` would let `combat::control` delete a rock retail cannot destroy, so
//! neither `combat::HitPoints` nor `enemy::Hurt` is attached.
//!
//! (b) outbound damage: OMITTED. `mCyl`'s `SrcObjAt Type` is `AT_TYPE_UNK8` at `Atp = 1`
//! (`d_a_fallrock.cpp:22-23`) and the collider is re-registered every tick (`:161-162`), but
//! `cM3dGCylS`'s `Height` is consumed by the collision engine only and its vertical convention
//! is not resolvable from anything in this checkout, so the contact test would have to be
//! invented.
//!
//! The water impact (`field_0x29C.ChkWaterIn()` at `d_a_fallrock.cpp:123`, sounding
//! `JA_SE_OBJ_FALL_WATER_M` / `_L` at `:125` / `:127`) is OMITTED: `ChkWaterIn` is raised inside
//! `dBgS_AcchBase_c::CrrPos` (`d_bg_s_acch.cpp:271-298`) from a per-room `dBgW`'s `GetTopUnder`
//! water table through `SplGrpChk`, which is the per-poly-group water query RULE 3b lists as
//! absent (`Collision` here is the merged DZB only). `super::water::Water` answers a different
//! question - "is the player standing in the sea polygon" - so substituting it would invent a
//! test. The sound ids and the `Impact::Water` arm are kept so the table stays complete.

use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::ground;
use super::stream;

/// `m_arcname` (`d_a_fallrock.cpp:14`); `d_stage.cpp:840` registers `fallrck` against
/// `fpcNm_FallRock_e`.
pub const FALLROCK_ACTORS: [&str; 1] = ["fallrck"];

const GRAVITY: f32 = -5.0; // <- d_a_fallrock.cpp:217 (`f32 grav = -5.0f`)
const MAX_FALL: f32 = -70.0; // <- d_a_fallrock.cpp:218 (`f32 fall_speed = -70.0f`)
const FALL_BUDGET: f32 = 7000.0; // <- d_a_fallrock.cpp:15 (m_falllen)
const TUMBLE: i16 = 0x3E8; // <- d_a_fallrock.cpp:16 (m_rot_speed)
/// `mCyl.SetH(scale.x * 120.0f)` (`d_a_fallrock.cpp:204`) and the matching
/// `set_mtx` `transM(0.0f, -(scale.x * 120.0f), 0.0f)` (`:70`). RULE 2: a stage-placed actor has
/// `scale.x == 1.0`, so the product folds to the bare literal.
const CYL_HEIGHT: f32 = 120.0; // <- d_a_fallrock.cpp:204 (scale.x * 120.0f, scale.x == 1.0)
/// `mCyl.SetR(scale.x * 90.0f)` (`d_a_fallrock.cpp:205`) and the simple-shadow
/// `i_this->scale.x * 90.0f` (`:84`).
const CYL_RADIUS: f32 = 90.0; // <- d_a_fallrock.cpp:205 (scale.x * 90.0f, scale.x == 1.0)
/// `if (scale.x < 1.3f)` picks the small/large sound pair on both the water (`:124`) and the
/// magma (`:145`) test. RULE 2 folds `scale.x` to 1.0, so only the `_M` arms can ever fire.
const HEAVY_SCALE: f32 = 1.3; // <- d_a_fallrock.cpp:124, :145

const SE_BREAK_ROCK: u32 = 0x692C; // <- d_a_fallrock.cpp:117 (JA_SE_OBJ_BREAK_ROCK)
const SE_FALL_MAGMA_M: u32 = 0x691C; // <- d_a_fallrock.cpp:146 (JA_SE_OBJ_FALL_MAGMA_M)
const SE_FALL_MAGMA_L: u32 = 0x691D; // <- d_a_fallrock.cpp:148 (JA_SE_OBJ_FALL_MAGMA_L)

/// `fopAc_CULLBOX_0_e` (`d_a_fallrock.cpp:314`) resolves through `l_cullSizeBox[0]`
/// (`f_op_actor_mng.cpp:559`), i.e. `(-40, 0, -40)..(40, 125, 40)`. The existing `src/lib.rs`
/// `actor_views` branch sizes the rock from `mCyl` instead, which is why there is no `cull()` here.

/// The impacts `execute` can report. Each maps to one `fopAcM_seStart` site and one
/// `setParticle` arm. The water impact (`:123-133`) is OMITTED - see the module docs - so
/// `JA_SE_OBJ_FALL_WATER_M` (`:125`, `0x6919`) and `_L` (`:127`, `0x691A`) have no arm here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Impact {
    Ground,
    Magma,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct FallRock {
    /// `field_0x610` (`d_a_fallrock.h:36`), the `fabs(speed.y)` accumulator compared against
    /// `m_falllen` at `:109`.
    budget: f32,
    /// `shape_angle.x`, chased by `cLib_chaseAngleS` at `:158`.
    tilt: i16,
    /// `deleted` (`:102`), the local that makes the four `fopAcM_delete` calls at `:111`, `:119`,
    /// `:131` and `:154` mutually exclusive. Read by the existing `src/lib.rs` `actor_views`
    /// branch to hide a spent rock.
    pub gone: bool,
    /// `scale.x`. RULE 2 fixes this at 1.0 for a stage-placed actor; it is kept because
    /// `hit_radius` / `hit_height` and the sound pair are all expressed through it.
    scale: f32,
    /// `speedF` and `speed` (`fopAc_ac_c`), which `fopAcM_posMoveF` reads at `:105`.
    speed_f: f32,
    speed: Vec3,
}

impl FallRock {
    /// `daFallRock_c::create` (`:188-228`). `fopAcM_ct_Retail` (`:189`) zeroes the actor, so
    /// `speedF` and `field_0x610` start at the explicit 0.0f stores of `:219-223` and `gravity` /
    /// `maxFallSpeed` at the `:217-222` stores.
    pub fn spawn(scale: f32) -> Self {
        Self {
            budget: 0.0,
            tilt: 0,
            gone: false,
            scale,
            speed_f: 0.0,
            speed: Vec3::ZERO,
        }
    }

    /// `mCyl.SetR(scale.x * 90.0f)` (`d_a_fallrock.cpp:205`).
    pub fn hit_radius(&self) -> f32 {
        self.scale * CYL_RADIUS
    }

    /// `mCyl.SetH(scale.x * 120.0f)` (`d_a_fallrock.cpp:204`).
    pub fn hit_height(&self) -> f32 {
        self.scale * CYL_HEIGHT
    }

    /// `scale.x < 1.3f` (`d_a_fallrock.cpp:124`, `:145`) selects the `_M` sound.
    pub fn light(&self) -> bool {
        self.scale < HEAVY_SCALE
    }

    /// The `fopAcM_seStart` id for each of the three impacts (`:117`, `:125`/`:127`, `:146`/`:148`).
    pub fn sound(&self, impact: Impact) -> u32 {
        match (impact, self.light()) {
            (Impact::Ground, _) => SE_BREAK_ROCK,
            (Impact::Magma, true) => SE_FALL_MAGMA_M,
            (Impact::Magma, false) => SE_FALL_MAGMA_L,
        }
    }

    /// `fopAcM_posMoveF(this, mStts.GetCCMoveP())` (`:105`) = `fopAcM_calcSpeed` +
    /// `fopAcM_posMove` (`f_op_actor_mng.cpp:480-486`). `mStts.GetCCMoveP()` is the `cCcD`
    /// displacement channel, which has no home in this sim, so only the `calcSpeed` half is
    /// modelled; `stream::calc_speed` is `fopAcM_calcSpeed` verbatim.
    fn integrate(&mut self, heading: i16) -> Vec3 {
        let (speed, speed_f) =
            stream::calc_speed(self.speed_f, heading, self.speed.y, GRAVITY, MAX_FALL);
        self.speed = speed;
        self.speed_f = speed_f;
        speed
    }

    /// `field_0x610 = fabs(speed.y) + field_0x610; if (field_0x610 > m_falllen) ...` (`:108-113`).
    fn accumulate(&mut self) -> bool {
        self.budget += self.speed.y.abs();
        if self.budget > FALL_BUDGET {
            self.gone = true;
            true
        } else {
            false
        }
    }

    /// `cLib_chaseAngleS(&shape_angle.x, shape_angle.x + m_rot_speed, m_rot_speed)` (`:158`).
    /// The target is recomputed from the current value every call, so the chase always lands on
    /// `value + m_rot_speed` and the angle simply advances by one step per tick.
    fn tumble(&mut self) {
        let target = self.tilt.wrapping_add(TUMBLE);
        super::walk::chase_angle_s(&mut self.tilt, target, TUMBLE);
    }

    /// `dBgS_ObjGndChk_Yogan chk; check_pos.set(old.pos.x, old.pos.y + scale.x * 120.0f,
    /// old.pos.z); chk.SetPos(&check_pos); f32 height = dComIfG_Bgsp()->GroundCross(&chk);`
    /// (`:135-142`) and `if (height > current.pos.y + scale.x * 120.0f)` (`:144`).
    fn lava_above(&self, solid: Option<&Collision>, before: Vec3, here: Vec3) -> bool {
        let probe = Vec3::new(before.x, before.y + self.scale * CYL_HEIGHT, before.z);
        ground::surface(solid, probe).is_some_and(|height| height > here.y + self.scale * CYL_HEIGHT)
    }
}

pub fn is_fallrock(name: &str) -> bool {
    FALLROCK_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    mut rocks: Query<(Entity, &mut Actor, &mut FallRock)>,
    mut events: ResMut<super::events::SimEvents>,
    mut commands: Commands,
) {
    let solid = collision.as_deref();
    for (entity, mut actor, mut rock) in &mut rocks {
        if rock.gone {
            continue;
        }
        let before = actor.pos;
        let step = rock.integrate(actor.angle[1]);
        actor.pos += step;
        let spent = rock.accumulate();
        // `field_0x29C.ChkGroundLanding()` (:115) - `dBgS_ObjAcch`'s landing test against the
        // merged DZB the same `CrrPos(*dComIfG_Bgsp())` call (:106) walks.
        let landed = ground::grounded(solid, actor.pos);
        let molten = rock.lava_above(solid, before, actor.pos);
        rock.tumble();
        let impact = if spent {
            None
        } else if landed {
            Some(Impact::Ground)
        } else if molten {
            Some(Impact::Magma)
        } else {
            None
        };
        if let Some(impact) = impact {
            events.list.push(super::events::SimEvent::Se(rock.sound(impact)));
            rock.gone = true;
        }
        if rock.gone {
            let _ = commands.entity(entity).despawn();
        }
    }
}
