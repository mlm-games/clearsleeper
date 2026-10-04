use bevy_ecs::prelude::*;

pub const TSUBO_ACTORS: [&str; 12] = [
    "Odokuro",
    "Tsubo",
    "Ktaru",
    "Ostool",
    "Ptubo",
    "kotubo",
    "ootubo1",
    "Kmtub",
    "Kmi00",
    "Kmi02",
    "KkibaB",
    "Okioke",
];

const ITEM_NO_MASK: u32 = 0x3F;
const SPEC_SHIFT: u32 = 8;
const SPEC_MASK: u32 = 0x3F;
const MOVE_BG_SHIFT: u32 = 14;
const MOVE_BG_MASK: u32 = 0x3;
const ITEM_SAVE_SHIFT: u32 = 16;
const ITEM_SAVE_MASK: u32 = 0x7F;
const TYPE_SHIFT: u32 = 24;
const TYPE_MASK: u32 = 0xF;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tsubo {
    pub spec: u8,
    pub item_no: u8,
    pub kind: u8,
    pub move_bg: u8,
    pub item_save: u8,
}

impl Tsubo {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            spec: ((parameters >> SPEC_SHIFT) & SPEC_MASK) as u8,
            item_no: (parameters & ITEM_NO_MASK) as u8,
            kind: ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8,
            move_bg: ((parameters >> MOVE_BG_SHIFT) & MOVE_BG_MASK) as u8,
            item_save: ((parameters >> ITEM_SAVE_SHIFT) & ITEM_SAVE_MASK) as u8,
        }
    }

    pub fn chained(&self) -> bool {
        self.spec != 0x3F
    }
}

pub fn is_tsubo(name: &str) -> bool {
    TSUBO_ACTORS.contains(&name)
}