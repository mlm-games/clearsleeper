use bevy_ecs::prelude::*;

pub const AJAVW_ACTORS: [&str; 1] = ["AjavW"];

/// Water surface inside Jabun's cave. `daObjAjavW_c::_execute` (`d_a_obj_AjavW.cpp:88-93`)
/// is `mBtkAnm.play()` plus `mpBgW->Move()` — a BTK texture animation and a registered
/// `dBgW` moving background. Both inputs are absent (RULE 3b), so the actor is a shell.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ajavw;

impl Ajavw {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_ajavw(name: &str) -> bool {
    AJAVW_ACTORS.contains(&name)
}

pub fn control(_water: Query<&Ajavw>) {}