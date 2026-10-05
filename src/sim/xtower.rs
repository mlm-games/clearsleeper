use bevy_ecs::prelude::*;

use super::flags::EventFlags;

pub const XTOWER_ACTORS: [&str; 1] = ["X_tower"];

pub const EVENT_TOWER_RESTORED: u32 = 0x1E40; // <- d_save_event_flag.inc:226
pub const CULL_MIN: [f32; 3] = [-30000.0, -5000.0, -30000.0]; // <- d_a_obj_tower.cpp:46
pub const CULL_MAX: [f32; 3] = [30000.0, 40000.0, 30000.0]; // <- d_a_obj_tower.cpp:46

/// Tower of the Gods exterior. `daObjTower_c::_create` (`d_a_obj_tower.cpp:60-80`) returns
/// `cPhs_STOP_e` until the Deku Tree event bit is set, and STOP cancels the create request
/// exactly like ERROR (`f_pc_create_req.cpp:106-108`), so `stopped` records that refusal.
/// `_execute` (`:119-129`) only follows a demo actor transform — the demo actor table is
/// absent — and then rewrites the model matrix.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Xtower {
    pub stopped: bool,
}

impl Xtower {
    pub fn spawn(flags: &EventFlags) -> Self {
        Self {
            stopped: !flags.test(EVENT_TOWER_RESTORED),
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_xtower(name: &str) -> bool {
    XTOWER_ACTORS.contains(&name)
}

pub fn control(
    mut flags: ResMut<EventFlags>,
    towers: Query<(Entity, &Xtower)>,
    mut commands: Commands,
) {
    for (entity, tower) in &towers {
        if tower.stopped {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        flags.set(EVENT_TOWER_RESTORED);
    }
}