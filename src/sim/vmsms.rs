use bevy_ecs::prelude::*;

use super::flags::EventFlags;

pub const VMSMS_ACTORS: [&str; 1] = ["VmsMS"];

pub const MASTER_SWORD_CUTSCENE: u32 = 0x2D04; // <- d_save_event_flag.inc:344

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vmsms {
    pub refused: bool,
}

impl Vmsms {
    pub fn spawn(flags: &EventFlags) -> Self {
        Self {
            refused: flags.test(MASTER_SWORD_CUTSCENE),
        }
    }
}

pub fn is_vmsms(name: &str) -> bool {
    VMSMS_ACTORS.contains(&name)
}

pub fn control(mut stones: Query<(Entity, &mut Vmsms)>, mut commands: Commands) {
    for (entity, stone) in &mut stones {
        if stone.refused {
            let _ = commands.entity(entity).despawn();
        }
    }
}
