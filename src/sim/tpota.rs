use bevy_ecs::prelude::*;

pub const TPOTA_ACTORS: [&str; 1] = ["Tpota"];

/// Waterfall spray pot. `daTpota_c::_create` (`d_a_tpota.cpp:17-32`) only attaches two
/// `JPABaseEmitter`s and `_execute` (`:98-116`) walks `mpEmitters[1]`'s particle list to
/// re-emit a ripple whenever a `POTB00` particle drops below y = -230. The particle system
/// is not ported, so there is no list to read, and `_draw` (`:119-121`) makes no draw call.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tpota;

impl Tpota {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_tpota(name: &str) -> bool {
    TPOTA_ACTORS.contains(&name)
}

pub fn control(_falls: Query<&Tpota>) {}