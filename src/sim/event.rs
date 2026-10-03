use bevy_ecs::prelude::*;

pub const EVENT_LIST_ENTRY: &str = "event_list.dat";

const HEADER_LEN: usize = 0x40;
const SECTIONS: usize = 7;
const STAFF_LEN: usize = 80;
const CUT_LEN: usize = 80;
const DATA_LEN: usize = 64;
const NAME_LEN: usize = 32;
const VALUE_LEN: usize = 4;
const TYPE_FLOAT: i32 = 0;
const TYPE_INT: i32 = 3;
const TYPE_STRING: i32 = 4;

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct EventList {
    sections: Vec<Section>,
    pub staffs: Vec<Staff>,
    cuts: Vec<Cut>,
    data: Vec<DataRecord>,
    floats: Vec<f32>,
    ints: Vec<i32>,
    text: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Section {
    pub top: u32,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Staff {
    pub kind: i32,
    pub first_cut: u32,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cut {
    pub first_data: u32,
    pub next: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataRecord {
    pub index: i32,
    pub next: i32,
    pub kind: i32,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Float(f32),
    Int(i32),
    Text,
    Raw(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventError;

struct Reader<'a> {
    data: &'a [u8],
}

impl<'a> Reader<'a> {
    fn bytes(&self, at: usize, len: usize) -> Result<&'a [u8], EventError> {
        self.data.get(at..at + len).ok_or(EventError)
    }

    fn u32(&self, at: usize) -> Result<u32, EventError> {
        let slice = self.bytes(at, 4)?;
        Ok(u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
    }

    fn i32(&self, at: usize) -> Result<i32, EventError> {
        let slice = self.bytes(at, 4)?;
        Ok(i32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
    }

    fn f32(&self, at: usize) -> Result<f32, EventError> {
        let slice = self.bytes(at, 4)?;
        Ok(f32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
    }

    fn text(&self, at: usize, len: usize) -> Result<String, EventError> {
        let raw = self.bytes(at, len)?;
        let stop = raw.iter().position(|&byte| byte == 0).unwrap_or(len);
        std::str::from_utf8(&raw[..stop])
            .map(str::to_owned)
            .map_err(|_| EventError)
    }
}

fn table(reader: &Reader<'_>, index: usize) -> Result<Section, EventError> {
    Ok(Section {
        top: reader.u32(index * 8)?,
        count: reader.u32(index * 8 + 4)?,
    })
}

fn offsets(section: Section, len: usize) -> Result<Vec<usize>, EventError> {
    let last = section.top as usize + section.count as usize * len;
    if last > section.top as usize + usize::MAX / 2 {
        return Err(EventError);
    }
    Ok((0..section.count as usize)
        .map(|index| section.top as usize + index * len)
        .collect())
}

impl EventList {
    pub fn parse(data: &[u8]) -> Result<Self, EventError> {
        if data.len() < HEADER_LEN {
            return Err(EventError);
        }
        let reader = Reader { data };
        let mut sections = Vec::with_capacity(SECTIONS);
        for index in 0..SECTIONS {
            sections.push(table(&reader, index)?);
        }
        let staff_at = offsets(sections[1], STAFF_LEN)?;
        for at in &staff_at {
            reader.bytes(*at, STAFF_LEN)?;
        }
        let staffs = staff_at
            .iter()
            .map(|at| {
                Ok(Staff {
                    name: reader.text(*at, NAME_LEN)?,
                    kind: reader.i32(*at + 0x2C)?,
                    first_cut: reader.u32(*at + 0x30)?,
                })
            })
            .collect::<Result<Vec<_>, EventError>>()?;
        let cut_at = offsets(sections[2], CUT_LEN)?;
        for at in &cut_at {
            reader.bytes(*at, CUT_LEN)?;
        }
        let cuts = cut_at
            .iter()
            .map(|at| {
                Ok(Cut {
                    first_data: reader.u32(*at + 0x38)?,
                    next: reader.i32(*at + 0x3C)?,
                })
            })
            .collect::<Result<Vec<_>, EventError>>()?;
        let data_at = offsets(sections[3], DATA_LEN)?;
        for at in &data_at {
            reader.bytes(*at, DATA_LEN)?;
        }
        let data_records = data_at
            .iter()
            .map(|at| {
                Ok(DataRecord {
                    name: reader.text(*at, NAME_LEN)?,
                    kind: reader.i32(*at + 0x24)?,
                    index: reader.i32(*at + 0x28)?,
                    next: reader.i32(*at + 0x30)?,
                })
            })
            .collect::<Result<Vec<_>, EventError>>()?;
        let float_at = offsets(sections[4], VALUE_LEN)?;
        for at in &float_at {
            reader.bytes(*at, VALUE_LEN)?;
        }
        let int_at = offsets(sections[5], VALUE_LEN)?;
        for at in &int_at {
            reader.bytes(*at, VALUE_LEN)?;
        }
        let floats = float_at
            .iter()
            .map(|at| reader.f32(*at))
            .collect::<Result<Vec<_>, EventError>>()?;
        let ints = int_at
            .iter()
            .map(|at| reader.i32(*at))
            .collect::<Result<Vec<_>, EventError>>()?;
        let text = data
            .get(sections[6].top as usize..sections[6].top as usize + sections[6].count as usize)
            .ok_or(EventError)?
            .to_vec();
        Ok(Self {
            sections,
            staffs,
            cuts,
            data: data_records,
            floats,
            ints,
            text,
        })
    }

    pub fn section(&self, index: usize) -> Option<Section> {
        self.sections.get(index).copied()
    }

    pub fn event_count(&self) -> u32 {
        self.section(0).map(|section| section.count).unwrap_or(0)
    }

    pub fn staff_of(&self, name: &str) -> Option<usize> {
        self.staffs.iter().position(|staff| staff.name == name)
    }

    pub fn value(&self, record: &DataRecord) -> Value {
        let at = record.index.max(0) as usize;
        match record.kind {
            TYPE_FLOAT => self
                .floats
                .get(at)
                .map(|value| Value::Float(*value))
                .unwrap_or(Value::Raw(record.index)),
            TYPE_INT => self
                .ints
                .get(at)
                .map(|value| Value::Int(*value))
                .unwrap_or(Value::Raw(record.index)),
            TYPE_STRING => {
                if self.text.get(at).is_some() {
                    Value::Text
                } else {
                    Value::Raw(record.index)
                }
            }
            _ => Value::Raw(record.index),
        }
    }

    pub fn params(&self, staff: usize) -> Vec<(String, Value)> {
        let Some(staff) = self.staffs.get(staff) else {
            return Vec::new();
        };
        let Some(cut) = self.cuts.get(staff.first_cut as usize) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        let mut index = cut.first_data as i32;
        let mut guard = 0_usize;
        while index >= 0 && guard <= self.data.len() {
            let Some(record) = self.data.get(index as usize) else {
                break;
            };
            if !record.name.is_empty() {
                out.push((record.name.clone(), self.value(record)));
            }
            index = record.next;
            guard += 1;
        }
        out
    }

    pub fn number(&self, staff: usize, key: &str) -> Option<i32> {
        self.params(staff)
            .into_iter()
            .find(|(name, _)| name == key)
            .and_then(|(_, value)| match value {
                Value::Int(number) => Some(number),
                Value::Float(number) => Some(number as i32),
                Value::Raw(number) => Some(number),
                Value::Text => None,
            })
    }
}
