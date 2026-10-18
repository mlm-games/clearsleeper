use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::gryw::Gryw;

pub const YGUSH_ACTORS: [&str; 4] = ["Ygstp00", "Ygush00", "Ygush01", "Ygush02"];

pub const SE_SPRING: u32 = 0x61FE; // <- JAZelAudio_SE.h:1710 (JA_SE_OBJ_SPRING)
pub const CULL_MIN: [f32; 3] = [-80.0, 0.0, -80.0]; // <- d_a_obj_Ygush00.cpp:84-86 (scale == 1.0)
pub const CULL_MAX: [f32; 3] = [80.0, 125.0, 80.0]; // <- d_a_obj_Ygush00.cpp:84-86 (scale == 1.0)

const PRM_TYPE_W: u32 = 3; // <- d_a_obj_Ygush00.h:15
const PRM_TYPE_S: u32 = 0; // <- d_a_obj_Ygush00.h:16
const TYPE_MAX: u8 = 4; // <- d_a_obj_Ygush00.cpp:71
const TYPE_POOL: u8 = 1; // <- d_a_obj_Ygush00.cpp:122

/// The spouts and springs that spout water and play their loop. The stage strings are
/// `Ygush00`, `Ygush01`, `Ygush02` and `Ygstp00` (`d_stage.cpp:1162-1165`), but the model's
/// archive is the single file-static `l_arcname = "Ygush00"` (`:18`) for all four.
///
/// `_create` (`:65-99`) reads `param_get_arg()` inside `fopAcM_IsFirstCreating` (`:68-73`);
/// the guard does not change the value, which is `param & 0x7` either way. The cull box
/// (`:84-86`) is `scale`-derived and stage-placed actors are exactly 1.0, so RULE 2 fixes the
/// literals above. `mBtkAnm.play()` / `mBckAnm.play()` (`:118-119`) are texture and skeletal
/// animation and are omitted. `Mthd_IsDelete` (`:168-171`) is a bare `TRUE`.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ygush {
    m_type: u8,
    pool: Option<Entity>,
}

impl Ygush {
    /// `_create` (`:69-73`): `mType = param_get_arg()`, then any value outside `0..=3`
    /// collapses to `0`.
    pub fn spawn(parameters: u32) -> Self {
        let raw = ((1 << PRM_TYPE_W) - 1) & (parameters >> PRM_TYPE_S);
        let m_type = if raw < TYPE_MAX as u32 { raw as u8 } else { 0 };
        Self { m_type, pool: None }
    }

    /// `daObjGryw00_c::get_draw_water_lv` (`d_a_obj_gryw00.cpp:54-57`) is
    /// `home.pos.y + mWaterLv + mWaveAmplitude`, and `Gryw`'s `water_lv` / `wave` are exactly
    /// `mWaterLv` / `mWaveAmplitude`, so the pool's home height is added back here.
    ///
    /// `mpGryw00` starts NULL and is filled by `fopAcM_SearchByName(fpcNm_Obj_Gryw00_e)`
    /// (`:128`), the first live `Gryw00` process; a cached pointer that no longer resolves
    /// falls back to the same search, which is the closest safe reading of the retail branch.
    fn pool_level(&mut self, pools: &Query<(Entity, &Actor, &Gryw)>) -> Option<f32> {
        if let Some(entity) = self.pool
            && let Ok((_, actor, pool)) = pools.get(entity)
        {
            return Some(actor.pos.y + pool.water_lv() + pool.wave());
        }
        let found = pools.iter().next();
        if let Some((entity, actor, pool)) = found {
            self.pool = Some(entity);
            return Some(actor.pos.y + pool.water_lv() + pool.wave());
        }
        self.pool = None;
        None
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_ygush(name: &str) -> bool {
    YGUSH_ACTORS.contains(&name)
}

pub fn control(
    mut events: ResMut<SimEvents>,
    mut springs: Query<(&Actor, &mut Ygush)>,
    pools: Query<(Entity, &Actor, &Gryw)>,
) {
    for (actor, mut spring) in &mut springs {
        if spring.m_type == TYPE_POOL {
            if let Some(level) = spring.pool_level(&pools)
                && level <= actor.pos.y
            {
                events.list.push(SimEvent::Se(SE_SPRING));
            }
        } else {
            events.list.push(SimEvent::Se(SE_SPRING));
        }
    }
}
