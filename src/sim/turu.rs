use bevy_ecs::prelude::*;

use super::flags::Switches;

pub const TURU_ACTORS: [&str; 1] = ["Turu"];

pub const NO_SWITCH: u8 = 0xFF;

pub const CULL_MIN: [f32; 3] = [-800.0, -200.0, -400.0];
pub const CULL_MAX: [f32; 3] = [500.0, 1000.0, 1000.0];

const SWITCH_SHIFT: u32 = 0x18;
const FIELD_MASK: u32 = 0xFF;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Turu {
    pub refused: bool,
}

impl Turu {
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let sw = ((parameters >> SWITCH_SHIFT) & FIELD_MASK) as u8;
        Self {
            refused: sw != NO_SWITCH && switches.test(sw),
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_turu(name: &str) -> bool {
    TURU_ACTORS.contains(&name)
}

pub fn control(dancers: Query<(Entity, &Turu)>, mut commands: Commands) {
    for (entity, dancer) in &dancers {
        if dancer.refused {
            let _ = commands.entity(entity).despawn();
        }
    }
}