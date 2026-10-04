use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::event::{ActiveEvents, EventList};
use super::player::Player;

pub const EVENT_SWITCH_ACTOR: &str = "Evsw";
pub const TAG_EVENT_ACTOR: &str = "TagEv";
pub const TAG_ISLAND_ACTOR: &str = "TagIsl";

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
pub struct OrderFlags {
    bits: u16,
}

impl OrderFlags {
    pub fn set(&mut self, bits: u16) {
        self.bits |= bits;
    }

    pub fn has(&self, bits: u16) -> bool {
        self.bits & bits != 0
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

    pub fn set(&mut self, index: usize) {
        if let Some(value) = self.bits.get_mut(index) {
            *value |= ARRIVE_BIT;
        }
    }

    pub fn bytes(&self) -> [u8; ISLE_COUNT] {
        self.bits
    }

    pub fn from_bytes(bytes: [u8; ISLE_COUNT]) -> Self {
        Self { bits: bytes }
    }
}

pub const REG_PAGES: usize = 256;
pub const REG_GHOST_SHIP: u16 = 0x8803;
pub const REG_GHOST_SHIP_ROOM: u16 = 0xC3FF;
pub const REG_GHOST_SHIP_START: u16 = 0x85FF;
pub const GHOST_SHIP_DONE: u8 = 3;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventReg {
    bytes: [u8; REG_PAGES],
}

impl Default for EventReg {
    fn default() -> Self {
        Self {
            bytes: [0; REG_PAGES],
        }
    }
}

impl EventReg {
    pub fn get(&self, reg: u16) -> u8 {
        let mask = (reg & 0xFF) as u8;
        self.bytes
            .get(usize::from(reg >> 8))
            .map_or(0, |page| page & mask)
    }

    pub fn set(&mut self, reg: u16, value: u8) {
        let mask = (reg & 0xFF) as u8;
        if let Some(page) = self.bytes.get_mut(usize::from(reg >> 8)) {
            *page &= !mask;
            *page |= value & mask;
        }
    }
}

pub const CHART_SLOTS: usize = 128;
pub const CHART_WORDS: usize = CHART_SLOTS / 32;
pub const CHART_COUNT: usize = 61;
pub const CHART_GREAT_SEA_A: usize = 35;
pub const CHART_GREAT_SEA_B: usize = 36;
pub const CHART_MIN_SECRET: usize = 52;
pub const TRIFORCE_SLOTS: u8 = 8;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Charts {
    got: [u32; CHART_WORDS],
    open: [u32; CHART_WORDS],
    done: [u32; CHART_WORDS],
    triforce: u8,
}

impl Default for Charts {
    fn default() -> Self {
        Self {
            got: [0; CHART_WORDS],
            open: [0; CHART_WORDS],
            done: [0; CHART_WORDS],
            triforce: 0,
        }
    }
}

impl Charts {
    fn test(words: &[u32; CHART_WORDS], slot: usize) -> bool {
        words
            .get(slot >> 5)
            .is_some_and(|word| word & (1 << (slot & 0x1F)) != 0)
    }

    fn mark(words: &mut [u32; CHART_WORDS], slot: usize, on: bool) {
        if let Some(word) = words.get_mut(slot >> 5) {
            if on {
                *word |= 1 << (slot & 0x1F);
            } else {
                *word &= !(1 << (slot & 0x1F));
            }
        }
    }

    pub fn has(&self, slot: usize) -> bool {
        Self::test(&self.got, slot)
    }

    pub fn is_open(&self, slot: usize) -> bool {
        Self::test(&self.open, slot)
    }

    pub fn is_done(&self, slot: usize) -> bool {
        Self::test(&self.done, slot)
    }

    pub fn give(&mut self, slot: usize) {
        Self::mark(&mut self.got, slot, true);
        Self::mark(&mut self.open, slot, true);
    }

    pub fn open(&mut self, slot: usize) {
        Self::mark(&mut self.open, slot, true);
    }

    pub fn close(&mut self, slot: usize) {
        Self::mark(&mut self.open, slot, false);
    }

    pub fn complete(&mut self, slot: usize) {
        Self::mark(&mut self.done, slot, true);
    }

    pub fn triforce(&self, slot: u8) -> bool {
        slot < TRIFORCE_SLOTS && self.triforce & (1 << slot) != 0
    }

    pub fn set_triforce(&mut self, slot: u8) {
        if slot < TRIFORCE_SLOTS {
            self.triforce |= 1 << slot;
        }
    }

    pub fn total(&self) -> usize {
        (1..=CHART_COUNT)
            .filter(|slot| {
                *slot != CHART_GREAT_SEA_A
                    && *slot != CHART_GREAT_SEA_B
                    && *slot < CHART_MIN_SECRET
                    && self.has(*slot - 1)
            })
            .count()
    }
}

const NO_SET_ID: u16 = 0xFFFF;
const SET_ID_WORDS: usize = 16;
const SET_ID_MAX: usize = SET_ID_WORDS * 32;
const ROOM_COUNT: usize = 64;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZoneActors {
    bits: [[u32; SET_ID_WORDS]; ROOM_COUNT],
}

impl Default for ZoneActors {
    fn default() -> Self {
        Self {
            bits: [[0_u32; SET_ID_WORDS]; ROOM_COUNT],
        }
    }
}

impl ZoneActors {
    pub fn is_done(&self, room: i8, set_id: u16) -> bool {
        let Some((word, mask)) = self.slot(room, set_id) else {
            return false;
        };
        self.bits[usize::from(room as u8)][word] & mask != 0
    }

    pub fn mark_done(&mut self, room: i8, set_id: u16) {
        let Some((word, mask)) = self.slot(room, set_id) else {
            return;
        };
        self.bits[usize::from(room as u8)][word] |= mask;
    }

    pub fn clear(&mut self) {
        self.bits = [[0_u32; SET_ID_WORDS]; ROOM_COUNT];
    }

    fn slot(&self, room: i8, set_id: u16) -> Option<(usize, u32)> {
        if room < 0 || set_id == NO_SET_ID {
            return None;
        }
        let id = usize::from(set_id);
        if id >= SET_ID_MAX {
            return None;
        }
        Some((id / 32, 1 << (id % 32)))
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
    mut active: ResMut<ActiveEvents>,
    list: Res<EventList>,
    players: Query<&Player>,
    tags: Query<(Entity, &Actor, &mut TagEvent)>,
) {
    for switch in &actors {
        if stories.test(switch.event) {
            switches.set(switch.switch_no);
        } else if !switch.hold {
            switches.clear(switch.switch_no);
        }
    }
    if let Some(player) = players.iter().next() {
        tag_triggers(
            &mut active,
            &list,
            &stories,
            &mut switches,
            player,
            &mut tags,
        );
    }
}

const NO_SWITCH: u8 = 0xFF;
const NO_EVENT_FLAG: u16 = 0xFFFF;
const RADIUS_SCALE: f32 = 100.0;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagState {
    Hunt,
    Event,
    Wait,
}

#[derive(Component, Debug, Clone)]
pub struct TagEvent {
    pub index: usize,
    pub swbit: u8,
    pub swbit2: u8,
    pub kind: u8,
    pub event_flag: u16,
    pub radius: f32,
    pub height: f32,
    pub state: TagState,
}

pub fn is_tag_event(name: &str) -> bool {
    name == TAG_EVENT_ACTOR || name == TAG_ISLAND_ACTOR
}

pub fn tag_event(
    parameters: u32,
    event_flag: u16,
    scale: [f32; 3],
    island: bool,
    index: usize,
) -> TagEvent {
    let swbit2 = if island {
        NO_SWITCH
    } else {
        ((parameters >> 16) & 0xFF) as u8
    };
    TagEvent {
        index,
        swbit: ((parameters >> 8) & 0xFF) as u8,
        swbit2,
        kind: (parameters & 0xFF) as u8,
        event_flag,
        radius: scale[0] * RADIUS_SCALE,
        height: scale[1] * RADIUS_SCALE,
        state: TagState::Hunt,
    }
}

fn tag_triggers(
    active: &mut ActiveEvents,
    list: &EventList,
    stories: &StoryBits,
    switches: &mut Switches,
    player: &Player,
    tags: &mut Query<(Entity, &Actor, &mut TagEvent)>,
) {
    for (entity, actor, mut tag) in tags.iter_mut() {
        if tag.state == TagState::Event {
            if active.end_check(tag.index) {
                tag.state = TagState::Wait;
            }
            continue;
        }
        if tag.state == TagState::Wait {
            if tag.swbit == NO_SWITCH || switches.test(tag.swbit) {
                continue;
            }
            tag.state = TagState::Hunt;
        }
        let terms = if tag.kind == 0x0C {
            tag.swbit2 == NO_SWITCH || !switches.test(tag.swbit2)
        } else {
            (tag.swbit2 == NO_SWITCH || switches.test(tag.swbit2))
                && (tag.event_flag == NO_EVENT_FLAG
                    || tag.event_flag == 0
                    || stories.test(tag.event_flag))
        };
        if !terms {
            continue;
        }
        if tag.swbit != NO_SWITCH && switches.test(tag.swbit) {
            tag.state = TagState::Wait;
            continue;
        }
        if tag.state == TagState::Hunt {
            let dx = player.pos.x - actor.pos.x;
            let dz = player.pos.z - actor.pos.z;
            let dy = (player.pos.y - actor.pos.y).abs();
            if dx * dx + dz * dz >= tag.radius * tag.radius || dy > tag.height {
                continue;
            }
        }
        if active.order(list, tag.index, entity) {
            if tag.swbit != NO_SWITCH {
                switches.set(tag.swbit);
            }
            tag.state = TagState::Event;
        }
    }
}
