use bevy_ecs::prelude::*;

pub const EVENT_SWITCH_ACTOR: &str = "Evsw";

const SWITCH_SHIFT: u32 = 0;
const EVENT_SHIFT: u32 = 8;
const KIND_SHIFT: u32 = 24;
const HOLD_KIND: u32 = 1;

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BitSet(pub [u64; 4]);

impl BitSet {
    pub fn test(&self, bit: u8) -> bool {
        let word = bit as usize / 64;
        self.0[word] >> (bit as usize % 64) & 1 == 1
    }

    pub fn set(&mut self, bit: u8) {
        let word = bit as usize / 64;
        self.0[word] |= 1 << (bit as usize % 64);
    }

    pub fn clear(&mut self, bit: u8) {
        let word = bit as usize / 64;
        self.0[word] &= !(1 << (bit as usize % 64));
    }
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EventFlags(pub BitSet);

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Switches(pub BitSet);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventSwitch {
    pub event: u8,
    pub switch_no: u8,
    pub hold: bool,
}

impl EventSwitch {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            event: ((parameters >> EVENT_SHIFT) & 0xFF) as u8,
            switch_no: ((parameters >> SWITCH_SHIFT) & 0xFF) as u8,
            hold: (parameters >> KIND_SHIFT) & 0x3 == HOLD_KIND,
        }
    }
}

pub fn is_event_switch(name: &str) -> bool {
    name == EVENT_SWITCH_ACTOR
}

pub fn control(
    events: Res<EventFlags>,
    mut switches: ResMut<Switches>,
    actors: Query<&EventSwitch>,
) {
    for switch in &actors {
        if events.0.test(switch.event) {
            switches.0.set(switch.switch_no);
        } else if !switch.hold {
            switches.0.clear(switch.switch_no);
        }
    }
}
