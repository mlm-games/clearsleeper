use bevy_ecs::prelude::*;

pub const EVENT_SWITCH_ACTOR: &str = "Evsw";

const SWITCH_SHIFT: u32 = 0;
const EVENT_SHIFT: u32 = 8;
const KIND_SHIFT: u32 = 24;
const HOLD_KIND: u32 = 1;
const STORY_BYTES: usize = 0x100;
const SWITCH_BITS: u32 = 128;
const FLAG_WORDS: usize = 320;
const FLAG_MAX: u32 = (FLAG_WORDS as u32) * 32;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoryBits {
    pub bytes: [u8; STORY_BYTES],
}

impl Default for StoryBits {
    fn default() -> Self {
        Self {
            bytes: [0_u8; STORY_BYTES],
        }
    }
}

impl StoryBits {
    fn cell(&self, id: u16) -> (usize, u8) {
        ((id >> 8) as usize, (id & 0xFF) as u8)
    }

    pub fn test(&self, id: u16) -> bool {
        let (cell, mask) = self.cell(id);
        self.bytes[cell] & mask != 0
    }

    pub fn set(&mut self, id: u16) {
        let (cell, mask) = self.cell(id);
        self.bytes[cell] |= mask;
    }

    pub fn clear(&mut self, id: u16) {
        let (cell, mask) = self.cell(id);
        self.bytes[cell] &= !mask;
    }

    pub fn reg(&self, id: u16) -> u8 {
        let (cell, mask) = self.cell(id);
        self.bytes[cell] & mask
    }

    pub fn set_reg(&mut self, id: u16, value: u8) {
        let (cell, mask) = self.cell(id);
        self.bytes[cell] &= !mask;
        self.bytes[cell] |= value;
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventFlags {
    words: [u32; FLAG_WORDS],
}

impl Default for EventFlags {
    fn default() -> Self {
        Self {
            words: [0_u32; FLAG_WORDS],
        }
    }
}

impl EventFlags {
    fn slot(&self, flag: u32) -> Option<(usize, u32)> {
        if flag >= FLAG_MAX {
            return None;
        }
        Some(((flag / 0x20) as usize, 1 << (flag & 0x1F)))
    }

    pub fn test(&self, flag: u32) -> bool {
        match self.slot(flag) {
            Some((word, bit)) => self.words[word] & bit != 0,
            None => false,
        }
    }

    pub fn set(&mut self, flag: u32) {
        if let Some((word, bit)) = self.slot(flag) {
            self.words[word] |= bit;
        }
    }

    pub fn clear(&mut self, flag: u32) {
        if let Some((word, bit)) = self.slot(flag) {
            self.words[word] &= !bit;
        }
    }

    pub fn words(&self) -> &[u32; FLAG_WORDS] {
        &self.words
    }

    pub fn from_words(words: [u32; FLAG_WORDS]) -> Self {
        Self { words }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Switches {
    words: [u32; SWITCH_BITS as usize / 32],
}

impl Switches {
    fn slot(&self, no: u8) -> Option<(usize, u32)> {
        if u32::from(no) >= SWITCH_BITS {
            return None;
        }
        Some(((no >> 5) as usize, 1 << (no & 0x1F)))
    }

    pub fn test(&self, no: u8) -> bool {
        match self.slot(no) {
            Some((word, bit)) => self.words[word] & bit != 0,
            None => false,
        }
    }

    pub fn set(&mut self, no: u8) {
        if let Some((word, bit)) = self.slot(no) {
            self.words[word] |= bit;
        }
    }

    pub fn clear(&mut self, no: u8) {
        if let Some((word, bit)) = self.slot(no) {
            self.words[word] &= !bit;
        }
    }

    pub fn words(&self) -> [u32; 4] {
        self.words
    }

    pub fn from_words(words: [u32; 4]) -> Self {
        Self { words }
    }
}

const ISLE_COUNT: usize = 49;
const ARRIVE_BIT: u8 = 1;
const INITIAL_ARRIVALS: [(usize, u8); 3] = [(0, 1 | 2), (10, 1 | 2), (43, 1 | 2)];

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Arrivals {
    bits: [u8; ISLE_COUNT],
}

impl Default for Arrivals {
    fn default() -> Self {
        let mut bits = [0_u8; ISLE_COUNT];
        for (index, value) in INITIAL_ARRIVALS {
            bits[index] = value;
        }
        Self { bits }
    }
}

impl Arrivals {
    pub fn is_arrived(&self, index: usize) -> bool {
        self.bits
            .get(index)
            .is_some_and(|value| value & ARRIVE_BIT != 0)
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventSwitch {
    pub event: u16,
    pub switch_no: u8,
    pub hold: bool,
}

impl EventSwitch {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            event: ((parameters >> EVENT_SHIFT) & 0xFFFF) as u16,
            switch_no: ((parameters >> SWITCH_SHIFT) & 0xFF) as u8,
            hold: (parameters >> KIND_SHIFT) & 0x3 == HOLD_KIND,
        }
    }
}

pub fn is_event_switch(name: &str) -> bool {
    name == EVENT_SWITCH_ACTOR
}

pub fn control(
    stories: Res<StoryBits>,
    mut switches: ResMut<Switches>,
    actors: Query<&EventSwitch>,
) {
    for switch in &actors {
        if stories.test(switch.event) {
            switches.set(switch.switch_no);
        } else if !switch.hold {
            switches.clear(switch.switch_no);
        }
    }
}
