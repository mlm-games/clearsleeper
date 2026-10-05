use bevy_ecs::prelude::*;

use super::flags::{Collect, SYMBOL_DIN};

pub const EAYOGN_ACTORS: [&str; 1] = ["Eayogn"];

pub const CULL_MIN: [f32; 3] = [-550.0, -50.0, -600.0]; // <- d_a_obj_eayogn.cpp:58
pub const CULL_MAX: [f32; 3] = [550.0, 50.0, 600.0]; // <- d_a_obj_eayogn.cpp:58

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eayogn {
    pub refused: bool,
}

impl Eayogn {
    /// `_create` (`d_a_obj_eayogn.cpp:43-65`) returns `cPhs_ERROR_e` unless
    /// `check_ev_bit()` (`d_a_obj_eayogn.cpp:89-91`) sees the Din symbol. Retail runs that
    /// once in Create, so the verdict is baked here rather than re-tested per tick.
    pub fn spawn(collect: &Collect) -> Self {
        Self {
            refused: !collect.is_symbol(SYMBOL_DIN),
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_eayogn(name: &str) -> bool {
    EAYOGN_ACTORS.contains(&name)
}

pub fn control(statues: Query<(Entity, &Eayogn)>, mut commands: Commands) {
    for (entity, statue) in &statues {
        if statue.refused {
            let _ = commands.entity(entity).despawn();
        }
    }
}