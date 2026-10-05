use bevy_ecs::prelude::*;

use super::actor::Actor;

pub const SPOTBOX_ACTORS: [&str; 2] = ["spotbox", "spotbx1"];

pub const TYPE_MASK: u32 = 1; // <- d_a_spotbox.h:12
pub const BASE_SMALL: f32 = 100.0; // <- d_a_spotbox.cpp:15
pub const BASE_LARGE: f32 = 1000.0; // <- d_a_spotbox.cpp:15
pub const HALF: f32 = 0.5; // <- d_a_spotbox.cpp:20

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct SpotBox {
    pub kind: u8,
    pub scale_y: f32,
    pub placed: bool,
}

impl SpotBox {
    pub fn spawn(parameters: u32) -> Self {
        let kind = (parameters & TYPE_MASK) as u8;
        Self {
            kind,
            scale_y: Self::base(kind),
            placed: false,
        }
    }

    fn base(kind: u8) -> f32 {
        if kind != 0 { BASE_LARGE } else { BASE_SMALL }
    }

    fn place(&mut self, actor: &mut Actor) {
        self.placed = true;
        let lift = self.scale_y * HALF;
        actor.pos.y += lift;
    }
}

pub fn is_spotbox(name: &str) -> bool {
    SPOTBOX_ACTORS.contains(&name)
}

pub fn control(mut boxes: Query<(&mut Actor, &mut SpotBox)>) {
    for (mut actor, mut spot) in &mut boxes {
        if !spot.placed {
            spot.place(&mut actor);
        }
    }
}