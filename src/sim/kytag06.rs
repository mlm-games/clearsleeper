use bevy_ecs::prelude::*;

use super::flags::{Collect, SYMBOL_NAYRU};

pub const KYTAG06_ACTORS: [&str; 1] = ["kytag6"];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kytag06;

impl Kytag06 {
    pub fn spawn(collect: &Collect) -> Option<Self> {
        if collect.symbol == SYMBOL_NAYRU {
            return None;
        }
        Some(Self)
    }
}

pub fn is_kytag06(name: &str) -> bool {
    KYTAG06_ACTORS.contains(&name)
}

pub fn control(_tags: Query<&Kytag06>) {}
