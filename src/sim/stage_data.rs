use bevy_ecs::prelude::*;

use crate::assets::AssetRoot;

pub const ACTOR_TABLE_FILE: &str = "files/res/ActorDat/ActorDat.bin";

pub const BLOCK_COUNT_OFFSET: usize = 0x0;
pub const BLOCK_LIST_OFFSET: usize = 0x4;
pub const TAG_SIZE: usize = 0x4;
pub const COUNT_SIZE: usize = 0x4;
pub const BLOCK_SIZE: usize = 0xC;
pub const NAME_POINTER_SIZE: usize = 0x4;

pub const TAG_ROW_NAMES: [u8; 4] = *b"ACFN";
pub const TAG_COLUMN_NAMES: [u8; 4] = *b"ACNA";
pub const TAG_VALUES: [u8; 4] = *b"ACDS";

pub const NORMAL_ITEM_NUM: usize = 16;
pub const ITEM_NUM: usize = 8;

pub const ROW_ARGUMENT: &str = "ARG";
pub const ROW_PERCENT: &str = "percent";
pub const ROW_NORMAL_ITEM: [&str; NORMAL_ITEM_NUM] = [
    "N_ITEM0", "N_ITEM1", "N_ITEM2", "N_ITEM3", "N_ITEM4", "N_ITEM5", "N_ITEM6", "N_ITEM7",
    "N_ITEM8", "N_ITEM9", "N_ITEM10", "N_ITEM11", "N_ITEM12", "N_ITEM13", "N_ITEM14", "N_ITEM15",
];
pub const ROW_ITEM: [&str; ITEM_NUM] = [
    "ITEM0", "ITEM1", "ITEM2", "ITEM3", "ITEM4", "ITEM5", "ITEM6", "ITEM7",
];

pub const VALUE_NONE: u8 = 0xFF;
pub const INDEX_NONE: i32 = -1;
pub const TABLE_BASE_ADDRESS: u32 = 0;

pub const TABLE_VALUE_POINTER: usize = 0x4;
pub const TABLE_NAME_LIST: usize = 0xC;
pub const TABLE_VALUE_SOURCE: usize = 0xC;
pub const TABLE_ROW_INDEX_ONE: usize = 0x4;
pub const TABLE_ROW_INDEX_SIXTEEN: usize = 0x40;
pub const TABLE_ROW_INDEX_EIGHT: usize = 0x20;
pub const CHAR_TABLE_SIZE: usize = 0x90;

const _: () = assert!(std::mem::size_of::<BlockEntry>() == BLOCK_SIZE);
const _: () = assert!(BLOCK_LIST_OFFSET - BLOCK_COUNT_OFFSET == COUNT_SIZE);
const _: () = assert!(
    CHAR_TABLE_SIZE
        == TABLE_VALUE_POINTER
            + TABLE_NAME_LIST
            + TABLE_NAME_LIST
            + TABLE_VALUE_SOURCE
            + TABLE_ROW_INDEX_ONE
            + TABLE_ROW_INDEX_SIXTEEN
            + TABLE_ROW_INDEX_ONE
            + TABLE_ROW_INDEX_EIGHT
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockEntry {
    pub tag: [u8; TAG_SIZE],
    pub size: u32,
    pub offset: u32,
}

impl BlockEntry {
    pub fn parse(data: &[u8], at: usize) -> Option<Self> {
        Some(Self {
            tag: data.get(at..at + TAG_SIZE)?.try_into().ok()?,
            size: read_u32(data, at + COUNT_SIZE)?,
            offset: read_u32(data, at + COUNT_SIZE * 2)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    name: Vec<u8>,
    pub argument: u8,
    pub normal_item: [u8; NORMAL_ITEM_NUM],
    pub percent: u8,
    pub item: [u8; ITEM_NUM],
}

impl Record {
    pub fn name(&self) -> &str {
        core::str::from_utf8(&self.name).unwrap_or("")
    }

    pub fn name_bytes(&self) -> &[u8] {
        &self.name
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ActorTable {
    blocks: Vec<BlockEntry>,
    rows: Vec<Vec<u8>>,
    columns: Vec<Vec<u8>>,
    values: Vec<u8>,
    records: Vec<Record>,
    index_argument: i32,
    index_normal_item: [i32; NORMAL_ITEM_NUM],
    index_percent: i32,
    index_item: [i32; ITEM_NUM],
}

impl ActorTable {
    pub fn loaded(&self) -> bool {
        !self.blocks.is_empty()
    }

    pub fn blocks(&self) -> &[BlockEntry] {
        &self.blocks
    }

    pub fn row_num(&self) -> usize {
        self.rows.len()
    }

    pub fn column_num(&self) -> usize {
        self.columns.len()
    }

    pub fn record_num(&self) -> usize {
        self.records.len()
    }

    pub fn row_name(&self, index: usize) -> Option<&str> {
        Some(core::str::from_utf8(self.rows.get(index)?).unwrap_or(""))
    }

    pub fn column_name(&self, index: usize) -> Option<&str> {
        Some(core::str::from_utf8(self.columns.get(index)?).unwrap_or(""))
    }

    pub fn set_data(&mut self, data: &[u8]) -> bool {
        *self = Self::default();
        let Some(count) = read_i32(data, BLOCK_COUNT_OFFSET) else {
            return false;
        };
        let Some(blocks) = read_blocks(data, count) else {
            return false;
        };
        self.blocks = blocks;
        let (row_count, row_offset) = match self.find_tag(TAG_ROW_NAMES) {
            Some(node) => (node.size, node.offset),
            None => return false,
        };
        let (column_count, column_offset) = match self.find_tag(TAG_COLUMN_NAMES) {
            Some(node) => (node.size, node.offset),
            None => return false,
        };
        let (value_size, value_offset) = match self.find_tag(TAG_VALUES) {
            Some(node) => (node.size, node.offset),
            None => return false,
        };
        if row_count.checked_mul(column_count) != Some(value_size) {
            log::warn!(
                "stage_data: {row_count} rows times {column_count} columns does not fill {value_size} values"
            );
            return false;
        }
        let Some(rows) = read_name_table(data, row_count, row_offset) else {
            return false;
        };
        let Some(columns) = read_name_table(data, column_count, column_offset) else {
            return false;
        };
        let Some(values) = read_values(data, value_size, value_offset) else {
            return false;
        };
        self.rows = rows;
        self.columns = columns;
        self.values = values;
        self.set_up_index();
        let records = self.decode_records();
        self.records = records;
        true
    }

    fn find_tag(&self, tag: [u8; TAG_SIZE]) -> Option<&BlockEntry> {
        let mut index = 0;
        while index < self.blocks.len() {
            let entry = &self.blocks[index];
            if entry.tag == tag {
                return Some(entry);
            }
            index += 1;
        }
        None
    }

    fn set_up_index(&mut self) {
        self.index_argument = self.get_format_index(ROW_ARGUMENT);
        for slot in 0..NORMAL_ITEM_NUM {
            self.index_normal_item[slot] = self.get_format_index(ROW_NORMAL_ITEM[slot]);
        }
        self.index_percent = self.get_format_index(ROW_PERCENT);
        for slot in 0..ITEM_NUM {
            self.index_item[slot] = self.get_format_index(ROW_ITEM[slot]);
        }
    }

    fn decode_records(&self) -> Vec<Record> {
        (0..self.columns.len())
            .map(|index| {
                let column = index as i32;
                Record {
                    name: self.columns[index].clone(),
                    argument: self.argument_at(column),
                    normal_item: self.normal_items_at(column),
                    percent: self.percent_at(column),
                    item: self.items_at(column),
                }
            })
            .collect()
    }

    pub fn value_at(&self, row: i32, column: i32) -> u8 {
        let rows = self.row_num() as i32;
        let columns = self.column_num() as i32;
        if row < 0 || row >= rows || column < 0 || column >= columns {
            return VALUE_NONE;
        }
        let at = row as usize + column as usize * self.row_num();
        self.values.get(at).copied().unwrap_or(VALUE_NONE)
    }

    pub fn get_format_index(&self, name: &str) -> i32 {
        find_name(&self.rows, name, 0)
    }

    pub fn get_name_index(&self, name: &str, start: i32) -> i32 {
        find_name(&self.columns, name, start)
    }

    pub fn get_name_index2(&self, name: &str, argument: i32) -> i32 {
        let mut start = 0;
        loop {
            start = self.get_name_index(name, start);
            if start == INDEX_NONE {
                return INDEX_NONE;
            }
            let info = i32::from(self.value_at(self.index_argument, start));
            if argument == info {
                return start;
            }
            start += 1;
        }
    }

    pub fn argument_row(&self) -> i32 {
        self.index_argument
    }

    pub fn normal_item_row(&self, slot: usize) -> i32 {
        self.index_normal_item
            .get(slot)
            .copied()
            .unwrap_or(INDEX_NONE)
    }

    pub fn percent_row(&self) -> i32 {
        self.index_percent
    }

    pub fn item_row(&self, slot: usize) -> i32 {
        self.index_item.get(slot).copied().unwrap_or(INDEX_NONE)
    }

    pub fn argument_at(&self, column: i32) -> u8 {
        self.value_at(self.index_argument, column)
    }

    pub fn percent_at(&self, column: i32) -> u8 {
        self.value_at(self.index_percent, column)
    }

    pub fn normal_items_at(&self, column: i32) -> [u8; NORMAL_ITEM_NUM] {
        let mut items = [0u8; NORMAL_ITEM_NUM];
        for (slot, item) in items.iter_mut().enumerate() {
            *item = self.value_at(self.normal_item_row(slot), column);
        }
        items
    }

    pub fn items_at(&self, column: i32) -> [u8; ITEM_NUM] {
        let mut items = [0u8; ITEM_NUM];
        for (slot, item) in items.iter_mut().enumerate() {
            *item = self.value_at(self.item_row(slot), column);
        }
        items
    }

    pub fn record_at(&self, column: i32) -> Option<&Record> {
        usize::try_from(column)
            .ok()
            .and_then(|index| self.records.get(index))
    }

    pub fn record_for_name(&self, name: &str) -> Option<&Record> {
        self.record_at(self.get_name_index(name, 0))
    }

    pub fn record_for_argument(&self, name: &str, argument: i32) -> Option<&Record> {
        self.record_at(self.get_name_index2(name, argument))
    }
}

fn find_name(names: &[Vec<u8>], name: &str, start: i32) -> i32 {
    if start < 0 {
        return INDEX_NONE;
    }
    let mut index = start;
    while (index as usize) < names.len() {
        if names[index as usize].as_slice() == name.as_bytes() {
            return index;
        }
        index += 1;
    }
    INDEX_NONE
}

fn read_u32(data: &[u8], at: usize) -> Option<u32> {
    let bytes = data.get(at..at + COUNT_SIZE)?;
    Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_i32(data: &[u8], at: usize) -> Option<i32> {
    read_u32(data, at).map(|value| value as i32)
}

fn rebase(pointer: u32) -> u32 {
    pointer.wrapping_add(TABLE_BASE_ADDRESS)
}

fn read_c_string(data: &[u8], at: usize) -> Option<Vec<u8>> {
    let rest = data.get(at..)?;
    let size = rest.iter().position(|byte| *byte == 0)?;
    Some(rest[..size].to_vec())
}

fn read_blocks(data: &[u8], count: i32) -> Option<Vec<BlockEntry>> {
    if count < 0 {
        return None;
    }
    let count = usize::try_from(count).ok()?;
    let start = BLOCK_LIST_OFFSET;
    let end = start.checked_add(count.checked_mul(BLOCK_SIZE)?)?;
    data.get(start..end)?;
    let mut blocks = Vec::with_capacity(count);
    for index in 0..count {
        blocks.push(BlockEntry::parse(data, start + index * BLOCK_SIZE)?);
    }
    Some(blocks)
}

fn read_name_table(data: &[u8], count: u32, offset: u32) -> Option<Vec<Vec<u8>>> {
    let count = usize::try_from(count).ok()?;
    let start = usize::try_from(offset).ok()?;
    let end = start.checked_add(count.checked_mul(NAME_POINTER_SIZE)?)?;
    data.get(start..end)?;
    let mut names = Vec::with_capacity(count);
    for index in 0..count {
        let at = start + index * NAME_POINTER_SIZE;
        let pointer = rebase(read_u32(data, at)?);
        names.push(read_c_string(data, usize::try_from(pointer).ok()?)?);
    }
    Some(names)
}

fn read_values(data: &[u8], size: u32, offset: u32) -> Option<Vec<u8>> {
    let start = usize::try_from(offset).ok()?;
    let size = usize::try_from(size).ok()?;
    let end = start.checked_add(size)?;
    Some(data.get(start..end)?.to_vec())
}

pub fn load_actor_table(root: &AssetRoot) -> Option<ActorTable> {
    match root.read(ACTOR_TABLE_FILE) {
        Ok(data) => {
            let mut table = ActorTable::default();
            if table.set_data(&data) {
                Some(table)
            } else {
                log::warn!("stage_data: {ACTOR_TABLE_FILE} holds no usable actor data table");
                None
            }
        }
        Err(error) => {
            log::warn!("stage_data: actor data unavailable ({error})");
            None
        }
    }
}

pub fn init(world: &mut World) {
    world.init_resource::<ActorTable>();
}
