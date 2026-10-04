use bevy_ecs::prelude::*;

pub const SLOT_COUNT: usize = 21;
pub const BOW_SLOT: usize = 12;
pub const BOMB_SLOT: usize = 13;
pub const WAKER_SLOT: usize = 2;
pub const NO_ITEM: u8 = 0xFF;

pub const ITEM_BOW: u8 = 0x27;
pub const ITEM_WIND_WAKER: u8 = 0x22;
pub const FIRST_BOMB: u8 = 0x0B;

const SLOT_ITEMS: [(u8, usize); 15] = [
    (0x20, 0),
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