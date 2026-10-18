use bevy_ecs::prelude::*;

use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::ground::grounded;

pub const TSUBO_ACTORS: [&str; 12] = [
    "Odokuro", "Ktaru", "Ostool", "Ptubo", "kotubo", "ootubo1", "Kmtub", "Kmi00", "Kmi02",
    "KkibaB", "Okioke", "Hbox2S",
];

pub const POT: u8 = 0;
pub const BIG: u8 = 1;
pub const WATER_JUG: u8 = 2;
pub const BARREL: u8 = 3;
pub const STOOL: u8 = 4;
pub const SKULL: u8 = 5;
pub const PAIL: u8 = 6;
pub const SPINE: u8 = 7;
pub const HBOX: u8 = 8;
pub const SUN_MER_JUP: u8 = 9;
pub const KEY_GATE: u8 = 12;
pub const PINECONE: u8 = 13;
pub const KUTANI: u8 = 14;
pub const WOOD: u8 = 15;

const ITEM_NO_MASK: u32 = 0x3F;
const SPEC_SHIFT: u32 = 8;
const SPEC_MASK: u32 = 0x3F;
const MOVE_BG_SHIFT: u32 = 14;
const MOVE_BG_MASK: u32 = 0x3;
const ITEM_SAVE_SHIFT: u32 = 16;
const ITEM_SAVE_MASK: u32 = 0x7F;
const TYPE_SHIFT: u32 = 24;
const TYPE_MASK: u32 = 0xF;

pub const FALL_DAMP: f32 = 0.9;
pub const CARRY_KIND: u8 = 0x3F;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Profile {
    pub gravity: f32,
    pub pop: f32,
    pub toss: f32,
}

const SMALL: Profile = Profile {
    gravity: -6.0,
    pop: 27.0,
    toss: 36.0,
};
const BARREL_PROFILE: Profile = Profile {
    gravity: -6.5,
    pop: 22.0,
    toss: 43.0,
};
const STOOL_PROFILE: Profile = Profile {
    gravity: -6.0,
    pop: 27.0,
    toss: 36.0,
};

pub fn profile(kind: u8) -> Profile {
    match kind {
        BARREL => BARREL_PROFILE,
        STOOL | SKULL | PAIL => STOOL_PROFILE,
        _ => SMALL,
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Tsubo {
    pub spec: u8,
    pub item_no: u8,
    pub kind: u8,
    pub move_bg: u8,
    pub item_save: u8,
    pub speed: f32,
    pub speed_y: f32,
    pub heading: glam::Vec3,
    pub falling: bool,
}

impl Tsubo {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            spec: ((parameters >> SPEC_SHIFT) & SPEC_MASK) as u8,
            item_no: (parameters & ITEM_NO_MASK) as u8,
            kind: ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8,
            move_bg: ((parameters >> MOVE_BG_SHIFT) & MOVE_BG_MASK) as u8,
            item_save: ((parameters >> ITEM_SAVE_SHIFT) & ITEM_SAVE_MASK) as u8,
            speed: 0.0,
            speed_y: 0.0,
            heading: Vec3::ZERO,
            falling: false,
        }
    }

    pub fn chained(&self) -> bool {
        self.spec != CARRY_KIND
    }

    pub fn toss(&mut self, forward: Vec3, player_speed: f32) {
        let profile = profile(self.kind);
        self.speed = profile.toss + player_speed;
        self.speed_y = profile.pop;
        self.falling = true;
        self.heading = forward;
    }
}

pub fn is_tsubo(name: &str) -> bool {
    TSUBO_ACTORS.contains(&name)
}

pub fn control(collision: Option<Res<Collision>>, mut pots: Query<(&mut Actor, &mut Tsubo)>) {
    let solid = collision.as_deref();
    for (mut actor, mut pot) in &mut pots {
        if !pot.falling {
            continue;
        }
        pot.speed_y += profile(pot.kind).gravity;
        actor.pos.y += pot.speed_y;
        actor.pos.x += pot.heading.x * pot.speed;
        actor.pos.z += pot.heading.z * pot.speed;
        pot.speed *= FALL_DAMP;
        if grounded(solid, actor.pos) {
            pot.falling = false;
            pot.speed = 0.0;
            pot.speed_y = 0.0;
        }
    }
}
