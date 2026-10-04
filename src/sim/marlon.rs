use bevy_ecs::prelude::*;

use super::flags::StoryBits;
use super::layer::SEA;

pub const MARLON_ACTOR: &str = "Mn";

const FIRST_TALK_FLAG: u16 = 0x2F08;
const SECOND_TALK_FLAG: u16 = 0x2F04;
const COMPANION_FLAG: u16 = 0x3120;
const FIGURE_FLAG: u16 = 0x3A01;
const COMPANION_ROOM_FLAG: u16 = 0x3D08;
const FIRST_TALK: [u16; 3] = [0x35E9, 0x35EA, 0x35EB];
const SECOND_TALK: [u16; 3] = [0x35EC, 0x35ED, 0x35EE];
const FIRST_TALK_IN: [u16; 1] = [0x35F1];
const SECOND_TALK_IN: [u16; 1] = [0x35F2];
const THIRD_TALK_IN: [u16; 1] = [0x35F0];
const COMPANION_FIRST: [u16; 3] = [0x35F3, 0x35F4, 0x35F5];
const COMPANION_SECOND: [u16; 1] = [0x35F6];
const FIGURE_TALK: [u16; 10] = [
    0x35F7, 0x35F8, 0x35F9, 0x35FA, 0x35FB, 0x35FC, 0x35FD, 0x35FE, 0x35FF, 0x3600,
];
const TALK3: u16 = 0x35EF;

pub fn talk3_opening() -> u16 {
    TALK3
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Marlon {
    pub inside: bool,
    pub companion: bool,
    pub figure: bool,
    pub figure_msg: usize,
    pub stage: usize,
}

impl Marlon {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            inside: parameters & 0x1 != 0,
            companion: parameters & 0xF == 0x3D08,
            figure: parameters & 0xF == 0x3A01,
            figure_msg: 0,
            stage: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Line {
    Next(u16),
    Done(Option<Talk3>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Talk3 {
    Order,
    Speak,
}

pub fn opening(talk: &Marlon, stories: StoryBits, sea: bool) -> u16 {
    if sea {
        return 0;
    }
    if !talk.inside {
        if !stories.test(FIRST_TALK_FLAG) {
            return FIRST_TALK[0];
        }
        return SECOND_TALK[0];
    }
    if talk.companion && stories.test(COMPANION_ROOM_FLAG) {
        if !stories.test(COMPANION_FLAG) {
            return COMPANION_FIRST[0];
        }
        return COMPANION_SECOND[0];
    }
    if talk.figure {
        return FIGURE_TALK[talk.figure_msg.min(FIGURE_TALK.len() - 1)];
    }
    if !stories.test(SECOND_TALK_FLAG) {
        return FIRST_TALK_IN[0];
    }
    if !stories.test(FIGURE_FLAG) {
        return SECOND_TALK_IN[0];
    }
    THIRD_TALK_IN[0]
}

pub fn step(line: &[u16], message: u16) -> Line {
    match line.iter().position(|entry| *entry == message) {
        Some(index) => match line.get(index + 1) {
            Some(next) => Line::Next(*next),
            None => Line::Done(None),
        },
        None => Line::Done(None),
    }
}

pub fn ladder(talk: &Marlon, stories: StoryBits, sea: bool) -> Vec<u16> {
    let line = match opening(talk, stories, sea) {
        0 => Vec::new(),
        0x35E9 => FIRST_TALK.to_vec(),
        0x35EC => SECOND_TALK.to_vec(),
        0x35F1 => FIRST_TALK_IN.to_vec(),
        0x35F2 => SECOND_TALK_IN.to_vec(),
        0x35F0 => THIRD_TALK_IN.to_vec(),
        0x35F3 => COMPANION_FIRST.to_vec(),
        0x35F6 => COMPANION_SECOND.to_vec(),
        other => {
            let mut line = Vec::new();
            if FIGURE_TALK.contains(&other) {
                line.push(other);
            }
            line
        }
    };
    line
}

pub fn first_visit(talk: &Marlon, stories: &mut StoryBits) {
    if !talk.inside {
        stories.set(FIRST_TALK_FLAG);
    } else if talk.companion && stories.test(COMPANION_ROOM_FLAG) {
        stories.set(COMPANION_FLAG);
    } else {
        stories.set(SECOND_TALK_FLAG);
    }
}

