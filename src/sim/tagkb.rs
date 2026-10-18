use bevy_ecs::prelude::*;

use super::flags::Switches;

pub const TAGKB_ACTORS: [&str; 1] = ["TagKb"];

pub const NO_SWITCH: u8 = 0xFF; // <- d_a_tag_kb_item.cpp:43

const SWITCH_SHIFT: u32 = 24; // <- d_a_tag_kb_item.cpp:13
const SWITCH_MASK: u32 = 0xFF; // <- d_a_tag_kb_item.cpp:13

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagKb {
    pub switch_no: u8,
}

impl TagKb {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            switch_no: ((parameters >> SWITCH_SHIFT) & SWITCH_MASK) as u8,
        }
    }

    /// `daTagKbItem_c::_create` (`tww/src/d/actor/d_a_tag_kb_item.cpp:42-46`) returns
    /// `cPhs_ERROR_e` on this term, so retail destroys the actor during Create.
    fn refused(&self, switches: &Switches) -> bool {
        self.switch_no != NO_SWITCH && switches.test(self.switch_no)
    }
}

pub fn is_tagkb(name: &str) -> bool {
    TAGKB_ACTORS.contains(&name)
}

pub fn control(switches: Res<Switches>, tags: Query<(Entity, &TagKb)>, mut commands: Commands) {
    for (entity, tag) in &tags {
        if tag.refused(&switches) {
            let _ = commands.entity(entity).despawn();
        }
    }
}
