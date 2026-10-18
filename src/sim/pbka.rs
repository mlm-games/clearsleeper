use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};

pub const PBKA_ACTORS: [&str; 1] = ["Pbka"];

pub const SPIN: i16 = 0x500; // <- d_a_obj_pbka.cpp:90
pub const SE_FAN: u32 = 0x619F; // <- JAZelAudio_SE.h:1698 (JA_SE_OBJ_BOMB_SHOP_FAN)
pub const CULL_MIN: [f32; 3] = [-300.0, -300.0, -300.0]; // <- d_a_obj_pbka.cpp:33
pub const CULL_MAX: [f32; 3] = [300.0, 300.0, 300.0]; // <- d_a_obj_pbka.cpp:33

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pbka;

impl Pbka {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_pbka(name: &str) -> bool {
    PBKA_ACTORS.contains(&name)
}

pub fn control(mut events: ResMut<SimEvents>, mut fans: Query<(&mut Actor, &Pbka)>) {
    for (mut actor, _) in &mut fans {
        actor.angle[1] = actor.angle[1].wrapping_add(SPIN);
        events.list.push(SimEvent::Se(SE_FAN));
    }
}
