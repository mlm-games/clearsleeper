use bevy_ecs::prelude::*;

pub const ZELDA_ACTOR: &str = "Zl1";

pub const EVENT_NAMES: [&str; 4] = ["yuukaigo", "ooi", "majyuu_shinnyuu", "nakaniwa"];

pub const EVENT_RAIN: usize = 0;
pub const EVENT_MEET: usize = 1;
pub const EVENT_SONG: usize = 2;
pub const EVENT_GARDEN: usize = 3;

const _: () = assert!(EVENT_NAMES.len() == EVENT_GARDEN + 1);

const START_X: f32 = -229.766;
const START_Y: f32 = 37.463;
const START_Z: f32 = 305.213;
const START_YAW: i16 = 0x4000;
const START_ROOM: u8 = 0;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Zelda {
    pub order: u8,
    pub talkable: bool,
    pub drawn: bool,
    pub event: usize,
    pub resting: bool,
    pub restart: (f32, f32, f32, i16, u8),
}

impl Zelda {
    pub fn spawn(parameters: u32) -> Self {
        let variant = parameters & 0xF;
        Self {
            order: match variant {
                2 => 2,
                4 => 4,
                5 => 5,
                _ => 0,
            },
            talkable: matches!(variant, 1 | 3 | 6 | 7),
            drawn: true,
            event: 0,
            resting: false,
            restart: (START_X, START_Y, START_Z, START_YAW, START_ROOM),
        }
    }

    pub fn event_name(&self) -> Option<&'static str> {
        if self.order < 3 {
            return None;
        }
        EVENT_NAMES.get(self.order as usize - 3).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scene {
    Accepted(usize),
    Talk,
}

pub fn accept(zelda: &Zelda, accepted: bool) -> Option<Scene> {
    if !accepted || !zelda.event_name().is_some() {
        return None;
    }
    Some(Scene::Accepted(zelda.order as usize - 3))
}

pub fn hides(zelda: &mut Zelda) -> bool {
    match zelda.event {
        EVENT_RAIN => {
            zelda.resting = true;
            true
        }
        EVENT_SONG => {
            zelda.drawn = false;
            true
        }
        _ => false,
    }
}

