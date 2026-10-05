use bevy_ecs::prelude::*;

pub const SLOT_COUNT: usize = 21;
pub const BOW_SLOT: usize = 12;
pub const BOMB_SLOT: usize = 13;
pub const WAKER_SLOT: usize = 2;
pub const HAMMER_SLOT: usize = 20;
pub const NO_ITEM: u8 = 0xFF;

pub const ITEM_BOW: u8 = 0x27;
pub const ITEM_WIND_WAKER: u8 = 0x22;
pub const ITEM_SKULL_HAMMER: u8 = 0x33;
pub const FIRST_BOMB: u8 = 0x0B;

const SLOT_ITEMS: [(u8, usize); 16] = [
    (0x20, 0),
    (0x78, 1),
    (0x22, 2),
    (0x25, 3),
    (0x24, 4),
    (0x2D, 5),
    (0x34, 6),
    (0x21, 7),
    (0x23, 8),
    (0x29, 9),
    (0x2A, 10),
    (0x2C, 11),
    (0x27, 12),
    (0x30, 18),
    (0x2F, 19),
    (0x33, 20),
];

pub fn slot_of(item: u8) -> Option<usize> {
    if (FIRST_BOMB..=0x0E).contains(&item) {
        return Some(BOMB_SLOT);
    }
    SLOT_ITEMS
        .iter()
        .find(|(id, _)| *id == item)
        .map(|(_, slot)| *slot)
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Equipped {
    slots: [u8; SLOT_COUNT],
}

impl Default for Equipped {
    fn default() -> Self {
        Self {
            slots: [NO_ITEM; SLOT_COUNT],
        }
    }
}

impl Equipped {
    pub fn item(&self, slot: usize) -> u8 {
        self.slots.get(slot).copied().unwrap_or(NO_ITEM)
    }

    pub fn set(&mut self, slot: usize, item: u8) {
        if let Some(entry) = self.slots.get_mut(slot) {
            *entry = item;
        }
    }

    pub fn has(&self, slot: usize) -> bool {
        self.item(slot) != NO_ITEM
    }

    pub fn grant(&mut self, slot: usize, item: u8) {
        if self.has(slot) {
            return;
        }
        self.set(slot, item);
    }

    pub fn collect(&mut self, item: u8) {
        if let Some(slot) = slot_of(item) {
            self.grant(slot, item);
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Got {
    flags: [u8; SLOT_COUNT],
}

impl Default for Got {
    fn default() -> Self {
        Self {
            flags: [0; SLOT_COUNT],
        }
    }
}

impl Got {
    pub fn has(&self, field: usize, bit: u8) -> bool {
        bit < 8 && self.flags.get(field).is_some_and(|flags| flags & (1 << bit) != 0)
    }

    pub fn gain(&mut self, field: usize, bit: u8) {
        if bit < 8
            && let Some(flags) = self.flags.get_mut(field)
        {
            *flags |= 1 << bit;
        }
    }

    pub fn clear(&mut self, field: usize, bit: u8) {
        if bit < 8
            && let Some(flags) = self.flags.get_mut(field)
        {
            *flags &= !(1 << bit);
        }
    }

    pub fn words(&self) -> [u8; SLOT_COUNT] {
        self.flags
    }

    pub fn from_words(flags: [u8; SLOT_COUNT]) -> Self {
        Self { flags }
    }
}

pub const ITEM_BOOMERANG: u8 = 0x2D;
pub const ITEM_TELESCOPE: u8 = 0x20;
pub const ITEM_PIRATES_CHARM: u8 = 0x42;
pub const ITEM_HEROS_CHARM: u8 = 0x43;
pub const ITEM_TRIFORCE_1: u8 = 0x61;
pub const ITEM_PEARL_NAYRU: u8 = 0x69;
pub const ITEM_PEARL_DIN: u8 = 0x6A;
pub const ITEM_PEARL_FARORE: u8 = 0x6B;
pub const ITEM_WINDS_REQUIEM: u8 = 0x6D;
pub const ITEM_BALLAD_OF_GALES: u8 = 0x6E;
pub const ITEM_COMMAND_MELODY: u8 = 0x6F;
pub const ITEM_EARTH_GODS_LYRIC: u8 = 0x70;
pub const ITEM_WIND_GODS_ARIA: u8 = 0x71;
pub const ITEM_SONG_OF_PASSING: u8 = 0x72;

pub fn check_get_item(
    equipped: &Equipped,
    collect: &super::flags::Collect,
    item: u8,
) -> bool {
    let tact = |no: u8| item == ITEM_WINDS_REQUIEM + no;
    if tact(0) {
        return collect.is_tact(0);
    }
    if tact(1) {
        return collect.is_tact(1);
    }
    if tact(2) {
        return collect.is_tact(2);
    }
    if tact(3) {
        return collect.is_tact(3);
    }
    if tact(4) {
        return collect.is_tact(4);
    }
    if tact(5) {
        return collect.is_tact(5);
    }
    let triforce = |no: u8| item == ITEM_TRIFORCE_1 + no;
    if triforce(0) {
        return collect.is_triforce(0);
    }
    if triforce(1) {
        return collect.is_triforce(1);
    }
    if triforce(2) {
        return collect.is_triforce(2);
    }
    if triforce(3) {
        return collect.is_triforce(3);
    }
    if triforce(4) {
        return collect.is_triforce(4);
    }
    if triforce(5) {
        return collect.is_triforce(5);
    }
    if triforce(6) {
        return collect.is_triforce(6);
    }
    if triforce(7) {
        return collect.is_triforce(7);
    }
    let pearl = |no: u8| item == ITEM_PEARL_NAYRU + no;
    if pearl(0) {
        return collect.is_symbol(super::flags::SYMBOL_NAYRU);
    }
    if pearl(1) {
        return collect.is_symbol(super::flags::SYMBOL_DIN);
    }
    if pearl(2) {
        return collect.is_symbol(super::flags::SYMBOL_FARORE);
    }
    if item == ITEM_PIRATES_CHARM {
        return collect.test(3, 0);
    }
    if item == ITEM_HEROS_CHARM {
        return collect.test(4, 0);
    }
    (0..60).any(|slot| equipped.item(slot) == item)
}
