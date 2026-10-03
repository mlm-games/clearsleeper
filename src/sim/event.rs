use bevy_ecs::prelude::*;

pub const EVENT_LIST_ENTRY: &str = "event_list.dat";
const HEADER_LEN: usize = 0x40;
const ORDER_LEN: usize = 0x18;
const NAME_LEN: usize = 15;
const PAIRS: usize = 5;

#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct EventList {
    pub orders: Vec<EventOrder>,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventOrder {
    pub spawn_switch: u8,
    pub tail: [u8; 8],
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Section {
    pub count: u32,
    pub offset: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventError;

fn u32_at(data: &[u8], at: usize) -> Option<u32> {
    let slice = data.get(at..at + 4)?;
    Some(u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

impl EventList {
    pub fn parse(data: &[u8]) -> Result<Self, EventError> {
        if data.len() < HEADER_LEN {
            return Err(EventError);
        }
        let count = u32_at(data, 4).ok_or(EventError)? as usize;
        let table = u32_at(data, 8).ok_or(EventError)? as usize;
        let mut sections = Vec::with_capacity(PAIRS);
        for pair in 0..PAIRS {
            let at = 0x0C + pair * 8;
            let count = u32_at(data, at).ok_or(EventError)?;
            let offset = u32_at(data, at + 4).ok_or(EventError)?;
            sections.push(Section { count, offset });
        }
        let mut orders = Vec::with_capacity(count);
        for index in 0..count {
            let at = table + index * ORDER_LEN;
            let entry = data.get(at..at + ORDER_LEN).ok_or(EventError)?;
            let raw = &entry[..NAME_LEN];
            let stop = raw.iter().position(|&byte| byte == 0).unwrap_or(NAME_LEN);
            let name = std::str::from_utf8(&raw[..stop])
                .map_err(|_| EventError)?
                .to_owned();
            let mut tail = [0_u8; 8];
            tail.copy_from_slice(&entry[NAME_LEN..NAME_LEN + 8]);
            orders.push(EventOrder {
                name,
                spawn_switch: tail[7],
                tail,
            });
        }
        Ok(Self { orders, sections })
    }

    pub fn finds(&self, name: &str) -> Option<&EventOrder> {
        self.orders.iter().find(|order| order.name == name)
    }

    pub fn section(&self, index: usize) -> Option<Section> {
        self.sections.get(index).copied()
    }
}
