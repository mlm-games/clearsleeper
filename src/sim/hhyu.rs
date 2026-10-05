use bevy_ecs::prelude::*;

use super::flags::Switches;

pub const HHYU_ACTORS: [&str; 1] = ["Hhyu1"];

pub const CULL_MIN: [f32; 3] = [-100.0, -50.0, -100.0]; // <- d_a_floor.cpp:49
pub const CULL_MAX: [f32; 3] = [100.0, 50.0, 100.0]; // <- d_a_floor.cpp:49

const SWITCH_MASK: u32 = 0xFF; // <- d_a_floor.cpp:14
const NO_SWITCH: u8 = 0xFF; // <- d_a_floor.cpp:60

/// Wind Temple breakable floor. `daFloor_c::_create` (`d_a_floor.cpp:56-69`) returns
/// `cPhs_ERROR_e` when the floor's switch is already on, which cancels the term inside
/// Create (`f_pc_create_req.cpp:106-108`), so `broken` records the refusal. Everything the
/// surviving term does is driven by `rideCallBack` (`:20-27`), a `dBgW` ride callback this
/// sim has no ground data for.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hhyu {
    pub broken: bool,
}

impl Hhyu {
    /// `daFloor_prm::getSwitchNo` (`d_a_floor.cpp:14`) is `(param >> 0) & 0xFF`.
    /// `mSwitchNo` has no other reader outside the omitted break sequence, so the raw index
    /// is not stored.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = ((parameters >> 0) & SWITCH_MASK) as u8;
        Self {
            broken: switch_no != NO_SWITCH && switches.test(switch_no),
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_hhyu(name: &str) -> bool {
    HHYU_ACTORS.contains(&name)
}

pub fn control(floors: Query<(Entity, &Hhyu)>, mut commands: Commands) {
    for (entity, floor) in &floors {
        if floor.broken {
            let _ = commands.entity(entity).despawn();
        }
    }
}