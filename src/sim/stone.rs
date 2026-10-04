use bevy_ecs::prelude::*;

use super::flags::Switches;
use super::inventory::Equipped;
use super::pickup::Rupees;

pub const STONE_ACTORS: [&str; 3] = ["Ebrock", "Ekao", "Ebrock2"];

pub const EBROCK: u8 = 0;
pub const EBROCK_ALT: u8 = 2;
pub const EKAO: u8 = 3;

pub const SWORD_HIT: i32 = 3;
pub const BOMB_HIT: i32 = 7;
pub const ITEM_NO_MASK: u32 = 0x3F;
pub const ITEM_SAVE_SHIFT: u32 = 16;
pub const ITEM_SAVE_MASK: u32 = 0x7F;
pub const TYPE_SHIFT: u32 = 24;
pub const TYPE_MASK: u32 = 0xF;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stone {
    pub kind: u8,
    pub item_no: u8,
    pub item_save: u8,
    pub switch_no: u8,
    pub broken: bool,
}

impl Stone {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8,
            item_no: (parameters & ITEM_NO_MASK) as u8,
            item_save: ((parameters >> ITEM_SAVE_SHIFT) & ITEM_SAVE_MASK) as u8,
            switch_no: 0,
            broken: false,
        }
    }

    pub fn carries(&self) -> bool {
        self.kind == EBROCK || self.kind == EBROCK_ALT || self.kind == EKAO
    }
}

pub fn is_stone(name: &str) -> bool {
    STONE_ACTORS.contains(&name)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Break {
    Sword,
    Bomb,
}

pub fn hit(stone: &Stone, power: i32) -> Option<Break> {
    match power {
        SWORD_HIT => Some(Break::Sword),
        BOMB_HIT if stone.kind != EKAO => Some(Break::Bomb),
        _ => None,
    }
}

pub fn spill(stone: &Stone, rupees: &mut Rupees, equipped: &mut Equipped) {
    if stone.item_no == 0 {
        rupees.count = rupees.count.saturating_add(20);
    } else {
        equipped.collect(stone.item_no);
    }
}

pub fn latches(stone: &mut Stone, switches: &Switches) -> bool {
    if stone.switch_no == 0 {
        return false;
    }
    switches.test(stone.switch_no)
}

