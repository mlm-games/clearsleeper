use bevy_ecs::prelude::*;

use super::flags::Collect;
use super::inventory::{self, Equipped, ITEM_PEARL_FARORE};

pub const RETAG_ACTORS: [&str; 1] = ["ReTag0"];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReTag;

impl ReTag {
    pub fn spawn() -> Self {
        Self
    }

    /// `daTagRet::Act_c::_create` (`d_a_tag_ret.cpp:50-54`) returns `cPhs_STOP_e`, which
    /// cancels the actor inside Create, so retail never runs `_execute` on this term.
    fn refused(&self, equipped: &Equipped, collect: &Collect) -> bool {
        inventory::check_get_item(equipped, collect, ITEM_PEARL_FARORE)
    }
}

pub fn is_retag(name: &str) -> bool {
    RETAG_ACTORS.contains(&name)
}

pub fn control(
    equipped: Res<Equipped>,
    collect: Res<Collect>,
    tags: Query<(Entity, &ReTag)>,
    mut commands: Commands,
) {
    for (entity, tag) in &tags {
        if tag.refused(&equipped, &collect) {
            let _ = commands.entity(entity).despawn();
        }
    }
}