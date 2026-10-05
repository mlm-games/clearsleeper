use bevy_ecs::prelude::*;

use super::flags::{Collect, SYMBOL_NAYRU};

pub const KYTAG06_ACTORS: [&str; 1] = ["kytag6"];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kytag06 {
    pub refused: bool,
}

impl Kytag06 {
    pub fn spawn(collect: &Collect) -> Self {
        Self {
            refused: collect.symbol == SYMBOL_NAYRU,
        }
    }
}

pub fn is_kytag06(name: &str) -> bool {
    KYTAG06_ACTORS.contains(&name)
}

pub fn control(mut tags: Query<(Entity, &mut Kytag06)>, mut commands: Commands) {
    for (entity, tag) in &mut tags {
        if tag.refused {
            let _ = commands.entity(entity).despawn();
        }
    }
}
