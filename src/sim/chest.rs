use bevy_ecs::prelude::*;

use super::flags::Switches;
use super::inventory::{Equipped, Got};
use super::pickup::Rupees;


pub const FUNC_NORMAL: u8 = 0;
pub const FUNC_EXTRA_SAVE_INFO: u8 = 7;
pub const FUNC_EXTRA_SAVE_INFO_SPAWN: u8 = 8;
pub const FUNC_SWITCH_VISIBLE: u8 = 3;
pub const FUNC_GRAVITY: u8 = 5;

pub const SEA2_STAGE: u8 = 1;
pub const SWITCH_CEILING: u8 = 0xC0;
pub const SHAPE_MAX: u8 = 4;
pub const NO_ROOM: i8 = -1;
pub const ROOM_MASK: i8 = 0x3F;
pub const GRAB_RANGE: f32 = 60.0;



#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chests {
    pub opened: [bool; 32],
    pub sea2_opened: [bool; 32],
}

impl Default for Chests {
    fn default() -> Self {
        Self {
            opened: [false; 32],
            sea2_opened: [false; 32],
        }
    }
}

impl Chests {
    pub fn is_open(&self, stage: u8, index: u8) -> bool {
        if index as usize >= self.opened.len() {
            return false;
        }
        if stage == SEA2_STAGE {
            self.sea2_opened[index as usize]
        } else {
            self.opened[index as usize]
        }
    }

    pub fn open(&mut self, stage: u8, index: u8) {
        let index = index as usize;
        if index >= self.opened.len() {
            return;
        }
        if stage == SEA2_STAGE {
            self.sea2_opened[index] = true;
        } else {
            self.opened[index] = true;
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chest {
    pub index: u8,
    pub item: u8,
    pub func: u8,
    pub shape: u8,
    pub switch_no: u8,
    pub open_switch: u8,
    pub room: i8,
    pub opened: bool,
    pub pressed: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChestOpen;

pub fn stage_of(chest: &Chest) -> u8 {
    if matches!(chest.func, FUNC_EXTRA_SAVE_INFO | FUNC_EXTRA_SAVE_INFO_SPAWN) {
        SEA2_STAGE
    } else {
        0
    }
}

impl Chest {
    pub fn spawn(parameters: u32, angle_z: i16, room: i8) -> Self {
        let shape = ((parameters >> 20) & 0xF) as u8;
        Self {
            index: ((parameters >> 7) & 0x1F) as u8,
            item: ((angle_z >> 8) & 0xFF) as u8,
            func: (parameters & 0x7F) as u8,
            shape: if shape >= SHAPE_MAX { 0 } else { shape },
            switch_no: ((parameters >> 12) & 0xFF) as u8,
            open_switch: angle_z as u8,
            room,
            opened: false,
            pressed: false,
        }
    }

    pub fn uses_switch(&self) -> bool {
        !matches!(self.func, FUNC_NORMAL | FUNC_EXTRA_SAVE_INFO | FUNC_SWITCH_VISIBLE | FUNC_GRAVITY)
    }

    pub fn gated_by_switch(&self) -> bool {
        !self.uses_switch() && self.switch_no >= SWITCH_CEILING
    }
}


pub fn present(chest: &Chest, switches: &Switches) -> bool {
    if chest.uses_switch() {
        return !switches.test(chest.switch_no);
    }
    true
}

pub fn loot(chest: &Chest, rupees: &mut Rupees, equipped: &mut Equipped, got: &mut Got) {
    if chest.item == 0 {
        rupees.count = rupees.count.saturating_add(50);
    } else {
        equipped.collect(chest.item);
        if let Some(slot) = super::inventory::slot_of(chest.item) {
            got.gain(slot, 0);
        }
    }
}

