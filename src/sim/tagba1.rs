use bevy_ecs::prelude::*;

use super::flags::EventFlags;

pub const TAGBA1_ACTORS: [&str; 1] = ["TagBa1"];

pub const EVENT_UNK_0520: u32 = 0x0520; // <- d_save_event_flag.inc:41
pub const EVENT_GRANDMA_HEALED: u32 = 0x2A20; // <- d_save_event_flag.inc:320

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagBa1 {
    pub refused: bool,
}

impl TagBa1 {
    /// `daTag_Ba1_c::createInit` (`d_a_tag_ba1.cpp:45-63`) returns false, and `_create`
    /// turns that into `cPhs_ERROR_e` (`d_a_tag_ba1.cpp:107`), unless `UNK_0520` is set
    /// and `GRANDMA_HEALED` is clear. Retail evaluates this once in Create, so the
    /// verdict is baked here rather than re-tested per tick.
    pub fn spawn(flags: &EventFlags) -> Self {
        Self {
            refused: !flags.test(EVENT_UNK_0520) || flags.test(EVENT_GRANDMA_HEALED),
        }
    }
}

pub fn is_tagba1(name: &str) -> bool {
    TAGBA1_ACTORS.contains(&name)
}

pub fn control(tags: Query<(Entity, &TagBa1)>, mut commands: Commands) {
    for (entity, tag) in &tags {
        if tag.refused {
            let _ = commands.entity(entity).despawn();
        }
    }
}
