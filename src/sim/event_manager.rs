use bevy_ecs::prelude::*;
use glam::Vec3;

use crate::assets::AssetRoot;

pub const STAGE_RES_ROOT: &str = "files/res/Stage";
pub const STAGE_RES_ARCHIVE: &str = "Stage.arc";
pub const STAGE_RES_GROUP: &str = "Stage";
pub const EVENT_LIST_ENTRY: &str = "dat/event_list.dat";

pub const HEADER_SIZE: usize = 0x40;
pub const EVENT_SIZE: usize = 0xB0;
pub const STAFF_SIZE: usize = 0x50;
pub const CUT_SIZE: usize = 0x50;
pub const DATA_SIZE: usize = 0x40;
pub const NAME_SIZE: usize = 0x20;
pub const EVENT_STAFF_NUM: usize = 20;
pub const EVENT_CHECK_START_NUM: usize = 2;
pub const EVENT_CHECK_FINISH_NUM: usize = 3;
pub const CUT_START_FLAG_NUM: usize = 3;
pub const FLAG_WORD_NUM: usize = 320;
pub const FLAG_MAX: i32 = 320 * 32;
pub const FLAG_WORD_BITS: u32 = 0x20;
pub const FLAG_BIT_MASK: u32 = 0x1F;

pub const STAFF_NORMAL: i32 = 0;
pub const STAFF_ALL: i32 = 1;
pub const STAFF_CAMERA: i32 = 2;
pub const STAFF_EFFECT: i32 = 3;
pub const STAFF_TIMEKEEPER: i32 = 4;
pub const STAFF_FIVE: i32 = 5;
pub const STAFF_DIRECTOR: i32 = 6;
pub const STAFF_MESSAGE: i32 = 7;
pub const STAFF_SOUND: i32 = 8;
pub const STAFF_LIGHT: i32 = 9;
pub const STAFF_TEN: i32 = 10;
pub const STAFF_PACKAGE: i32 = 11;
pub const STAFF_CREATE: i32 = 12;

pub const STATE_NONE: i32 = 0;
pub const STATE_ORDER: i32 = 1;
pub const STATE_PLAY: i32 = 2;
pub const STATE_UNK3: i32 = 3;
pub const STATE_CLOSE: i32 = 4;

pub const SUB_FLOAT: i32 = 0;
pub const SUB_VEC: i32 = 1;
pub const SUB_UNK2: i32 = 2;
pub const SUB_INT: i32 = 3;
pub const SUB_STR: i32 = 4;

pub const ADVANCE_STOP: u8 = 0;
pub const ADVANCE_CUT: u8 = 1;
pub const ADVANCE_START: u8 = 2;

pub const ACTION_NONE: u32 = 0xFFFF_FFFF;
pub const NAME_CMP_LIMIT: usize = 100;
pub const TALKMAN_NAME: &str = "TALKMAN";
pub const DEV_PRM_MASK_NAME: &str = "DEV_PRM_MASK";
pub const CAST_IN_NAME: &str = "CAST_IN";
pub const TIMER_NAME: &str = "Timer";
pub const CAST_IN_FLAG: u16 = 1;

pub const PRESENT_ACT_WAIT: i32 = 0;
pub const PRESENT_ACT_TALK0: i32 = 1;
pub const PRESENT_ACT_TALK1: i32 = 2;
const PRESENT_ACTION_TABLE: [&str; 3] = ["WAIT", "TALK0", "TALK1"];

fn read_u16(data: &[u8], at: usize) -> Option<u16> {
    let bytes = data.get(at..at + 2)?;
    Some(u16::from_be_bytes([bytes[0], bytes[1]]))
}

fn read_i16(data: &[u8], at: usize) -> Option<i16> {
    read_u16(data, at).map(|value| value as i16)
}

fn read_u32(data: &[u8], at: usize) -> Option<u32> {
    let bytes = data.get(at..at + 4)?;
    Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_i32(data: &[u8], at: usize) -> Option<i32> {
    read_u32(data, at).map(|value| value as i32)
}

fn read_u64(data: &[u8], at: usize) -> Option<u64> {
    let bytes = data.get(at..at + 8)?;
    let mut value = [0u8; 8];
    value.copy_from_slice(bytes);
    Some(u64::from_be_bytes(value))
}

fn read_name(bytes: &[u8; NAME_SIZE]) -> &str {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(NAME_SIZE);
    core::str::from_utf8(&bytes[..end]).unwrap_or("")
}

fn name_prefix_cmp(s1: &str, s2: &str) -> i32 {
    let left = s1.as_bytes();
    let right = s2.as_bytes();
    if left.len() >= NAME_CMP_LIMIT || right.len() >= NAME_CMP_LIMIT {
        return -1;
    }
    if left.len() > right.len() {
        return 1;
    }
    for index in 0..left.len() {
        if left[index] != right[index] {
            return 1;
        }
    }
    0
}

fn parse_sector<T>(
    data: &[u8],
    top: u32,
    count: i32,
    size: usize,
    parse: fn(&[u8]) -> Option<T>,
) -> Vec<T> {
    if count <= 0 {
        return Vec::new();
    }
    let count = match usize::try_from(count) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };
    let start = top as usize;
    let end = match count
        .checked_mul(size)
        .and_then(|bytes| start.checked_add(bytes))
    {
        Some(value) => value,
        None => return Vec::new(),
    };
    match data.get(start..end) {
        Some(slice) => slice.chunks_exact(size).filter_map(parse).collect(),
        None => Vec::new(),
    }
}

fn parse_floats(data: &[u8], top: u32, count: i32) -> Vec<f32> {
    parse_sector(data, top, count, 4, |at| {
        read_u32(at, 0).map(f32::from_bits)
    })
}

fn parse_ints(data: &[u8], top: u32, count: i32) -> Vec<i32> {
    parse_sector(data, top, count, 4, |at| read_i32(at, 0))
}

fn parse_bytes(data: &[u8], top: u32, count: i32) -> Vec<u8> {
    if count <= 0 {
        return Vec::new();
    }
    let count = match usize::try_from(count) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };
    let start = top as usize;
    let end = match start.checked_add(count) {
        Some(value) => value,
        None => return Vec::new(),
    };
    match data.get(start..end) {
        Some(slice) => slice.to_vec(),
        None => Vec::new(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BinaryHeader {
    pub event_top: u32,
    pub event_num: i32,
    pub staff_top: u32,
    pub staff_num: i32,
    pub cut_top: u32,
    pub cut_num: i32,
    pub data_top: u32,
    pub data_num: i32,
    pub f_data_top: u32,
    pub f_data_num: i32,
    pub i_data_top: u32,
    pub i_data_num: i32,
    pub s_data_top: u32,
    pub s_data_num: i32,
    pub unk: u64,
}

impl BinaryHeader {
    pub fn parse(data: &[u8]) -> Option<Self> {
        data.get(..HEADER_SIZE)?;
        Some(Self {
            event_top: read_u32(data, 0x00)?,
            event_num: read_i32(data, 0x04)?,
            staff_top: read_u32(data, 0x08)?,
            staff_num: read_i32(data, 0x0C)?,
            cut_top: read_u32(data, 0x10)?,
            cut_num: read_i32(data, 0x14)?,
            data_top: read_u32(data, 0x18)?,
            data_num: read_i32(data, 0x1C)?,
            f_data_top: read_u32(data, 0x20)?,
            f_data_num: read_i32(data, 0x24)?,
            i_data_top: read_u32(data, 0x28)?,
            i_data_num: read_i32(data, 0x2C)?,
            s_data_top: read_u32(data, 0x30)?,
            s_data_num: read_i32(data, 0x34)?,
            unk: read_u64(data, 0x38)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventRecord {
    pub name_field: [u8; NAME_SIZE],
    pub priority: u32,
    pub staff_idx: [u32; EVENT_STAFF_NUM],
    pub n_staff: i32,
    pub check_start: [i32; EVENT_CHECK_START_NUM],
    pub check_finish: [i32; EVENT_CHECK_FINISH_NUM],
    pub end_sound: u8,
    pub state: i32,
}

impl EventRecord {
    pub fn name(&self) -> &str {
        read_name(&self.name_field)
    }

    pub fn staff(&self, idx: i32) -> Option<u32> {
        if idx < 0 {
            return None;
        }
        self.staff_idx.get(idx as usize).copied()
    }

    fn parse(at: &[u8]) -> Option<Self> {
        let mut name_field = [0u8; NAME_SIZE];
        name_field.copy_from_slice(at.get(..NAME_SIZE)?);
        let mut staff_idx = [0u32; EVENT_STAFF_NUM];
        for (index, slot) in staff_idx.iter_mut().enumerate() {
            *slot = read_u32(at, 0x2C + index * 4)?;
        }
        Some(Self {
            name_field,
            priority: read_u32(at, 0x28)?,
            staff_idx,
            n_staff: read_i32(at, 0x7C)?,
            check_start: [read_i32(at, 0x80)?, read_i32(at, 0x84)?],
            check_finish: [
                read_i32(at, 0x88)?,
                read_i32(at, 0x8C)?,
                read_i32(at, 0x90)?,
            ],
            end_sound: *at.get(0x94)?,
            state: read_i32(at, 0xA4)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaffRecord {
    pub name_field: [u8; NAME_SIZE],
    pub tag_id: i32,
    pub staff_idx: i32,
    pub field_28: i32,
    pub staff_type: i32,
    pub first_cut: u32,
    pub cur_cut: u32,
    pub cur_action: u32,
    pub wipe_direction: i16,
    pub timer: i16,
    pub field_44: u8,
    pub field_45: u8,
    pub advance: u8,
    pub has_action: bool,
}

impl StaffRecord {
    pub fn name(&self) -> &str {
        read_name(&self.name_field)
    }

    pub fn kind(&self) -> i32 {
        self.staff_type
    }

    pub fn start_cut(&self) -> u32 {
        self.first_cut
    }

    pub fn current_cut(&self) -> u32 {
        self.cur_cut
    }

    fn parse(at: &[u8]) -> Option<Self> {
        let mut name_field = [0u8; NAME_SIZE];
        name_field.copy_from_slice(at.get(..NAME_SIZE)?);
        Some(Self {
            name_field,
            tag_id: read_i32(at, 0x20)?,
            staff_idx: read_i32(at, 0x24)?,
            field_28: read_i32(at, 0x28)?,
            staff_type: read_i32(at, 0x2C)?,
            first_cut: read_u32(at, 0x30)?,
            cur_cut: read_u32(at, 0x38)?,
            cur_action: read_u32(at, 0x3C)?,
            wipe_direction: read_i16(at, 0x40)?,
            timer: read_i16(at, 0x42)?,
            field_44: *at.get(0x44)?,
            field_45: *at.get(0x45)?,
            advance: *at.get(0x46)?,
            has_action: *at.get(0x47)? != 0,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CutRecord {
    pub name_field: [u8; NAME_SIZE],
    pub tag_id: u32,
    pub index: u32,
    pub start_flag: [i32; CUT_START_FLAG_NUM],
    pub flag_idx: i32,
    pub first_data: i32,
    pub next: i32,
}

impl CutRecord {
    pub fn name(&self) -> &str {
        read_name(&self.name_field)
    }

    pub fn start_flag(&self, idx: usize) -> Option<i32> {
        self.start_flag.get(idx).copied()
    }

    pub fn flag_id(&self) -> i32 {
        self.flag_idx
    }

    pub fn data_top(&self) -> i32 {
        self.first_data
    }

    pub fn next_cut(&self) -> i32 {
        self.next
    }

    fn parse(at: &[u8]) -> Option<Self> {
        let mut name_field = [0u8; NAME_SIZE];
        name_field.copy_from_slice(at.get(..NAME_SIZE)?);
        Some(Self {
            name_field,
            tag_id: read_u32(at, 0x20)?,
            index: read_u32(at, 0x24)?,
            start_flag: [
                read_i32(at, 0x28)?,
                read_i32(at, 0x2C)?,
                read_i32(at, 0x30)?,
            ],
            flag_idx: read_i32(at, 0x34)?,
            first_data: read_i32(at, 0x38)?,
            next: read_i32(at, 0x3C)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataRecord {
    pub name_field: [u8; NAME_SIZE],
    pub index: i32,
    pub substance_type: i32,
    pub substance_idx: i32,
    pub substance_size: i32,
    pub next: i32,
}

impl DataRecord {
    pub fn name(&self) -> &str {
        read_name(&self.name_field)
    }

    pub fn next_data(&self) -> i32 {
        self.next
    }

    pub fn number(&self) -> i32 {
        self.substance_size
    }

    pub fn substance(&self) -> i32 {
        self.substance_type
    }

    fn parse(at: &[u8]) -> Option<Self> {
        let mut name_field = [0u8; NAME_SIZE];
        name_field.copy_from_slice(at.get(..NAME_SIZE)?);
        Some(Self {
            name_field,
            index: read_i32(at, 0x20)?,
            substance_type: read_i32(at, 0x24)?,
            substance_idx: read_i32(at, 0x28)?,
            substance_size: read_i32(at, 0x2C)?,
            next: read_i32(at, 0x30)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventFlags {
    pub words: [u32; FLAG_WORD_NUM],
}

impl Default for EventFlags {
    fn default() -> Self {
        Self {
            words: [0u32; FLAG_WORD_NUM],
        }
    }
}

impl EventFlags {
    pub fn init(&mut self) {
        self.words = [0u32; FLAG_WORD_NUM];
    }

    pub fn max_check(flag_id: i32) -> bool {
        flag_id == -1 || flag_id >= FLAG_MAX
    }

    pub fn check(&self, flag_id: i32) -> bool {
        if Self::max_check(flag_id) {
            return false;
        }
        let word = (flag_id as u32) / FLAG_WORD_BITS;
        let bit = 1u32 << ((flag_id as u32) & FLAG_BIT_MASK);
        match self.words.get(word as usize) {
            Some(value) => value & bit != 0,
            None => false,
        }
    }

    pub fn set(&mut self, flag_id: i32) -> bool {
        if Self::max_check(flag_id) {
            return false;
        }
        let word = (flag_id as u32) / FLAG_WORD_BITS;
        let bit = 1u32 << ((flag_id as u32) & FLAG_BIT_MASK);
        match self.words.get_mut(word as usize) {
            Some(value) => {
                *value |= bit;
                true
            }
            None => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventException {
    pub event_info_idx: i32,
    pub field_4: i32,
    pub state: i32,
}

impl Default for EventException {
    fn default() -> Self {
        Self {
            event_info_idx: -1,
            field_4: -1,
            state: 0,
        }
    }
}

impl EventException {
    pub fn init(&mut self) {
        self.event_info_idx = -1;
        self.field_4 = -1;
        self.state = 0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaffActor {
    pub force_move: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffSearchParam {
    pub name: String,
    pub mask: u32,
    pub value: u32,
    pub cast_in_flag: u16,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct EventManager {
    header: Option<BinaryHeader>,
    events: Vec<EventRecord>,
    staffs: Vec<StaffRecord>,
    cuts: Vec<CutRecord>,
    data: Vec<DataRecord>,
    floats: Vec<f32>,
    ints: Vec<i32>,
    text: Vec<u8>,
    pub flags: EventFlags,
    pub exception: EventException,
    pub camera_play: u32,
    pub goal: Vec3,
    pub run_check: bool,
    pub event_info_names: Vec<String>,
}

impl EventManager {
    pub fn create(&mut self, data: Option<&[u8]>) -> bool {
        self.list_init();
        self.camera_play = 0;
        self.exception.init();
        self.flags.init();
        if let Some(data) = data {
            self.set_data(data);
        }
        true
    }

    pub fn remove(&mut self) {
        self.list_init();
        self.camera_play = 0;
        self.exception.init();
        self.flags.init();
    }

    pub fn execute(&mut self) {
        self.main_proc();
    }

    fn list_init(&mut self) {
        self.header = None;
        self.events.clear();
        self.staffs.clear();
        self.cuts.clear();
        self.data.clear();
        self.floats.clear();
        self.ints.clear();
        self.text.clear();
    }

    pub fn set_data(&mut self, data: &[u8]) {
        let Some(header) = BinaryHeader::parse(data) else {
            return;
        };
        self.events = parse_sector(
            data,
            header.event_top,
            header.event_num,
            EVENT_SIZE,
            EventRecord::parse,
        );
        self.staffs = parse_sector(
            data,
            header.staff_top,
            header.staff_num,
            STAFF_SIZE,
            StaffRecord::parse,
        );
        self.cuts = parse_sector(
            data,
            header.cut_top,
            header.cut_num,
            CUT_SIZE,
            CutRecord::parse,
        );
        self.data = parse_sector(
            data,
            header.data_top,
            header.data_num,
            DATA_SIZE,
            DataRecord::parse,
        );
        self.floats = parse_floats(data, header.f_data_top, header.f_data_num);
        self.ints = parse_ints(data, header.i_data_top, header.i_data_num);
        self.text = parse_bytes(data, header.s_data_top, header.s_data_num);
        self.header = Some(header);
    }

    pub fn loaded(&self) -> bool {
        self.header.is_some()
    }

    pub fn header(&self) -> Option<&BinaryHeader> {
        self.header.as_ref()
    }

    pub fn event_num(&self) -> i32 {
        self.header().map_or(0, |header| header.event_num)
    }

    pub fn staff_num(&self) -> i32 {
        self.header().map_or(0, |header| header.staff_num)
    }

    pub fn cut_num(&self) -> i32 {
        self.header().map_or(0, |header| header.cut_num)
    }

    pub fn event_data(&self, event_idx: i32) -> Option<&EventRecord> {
        if event_idx < 0 || event_idx >= self.event_num() {
            return None;
        }
        self.events.get(event_idx as usize)
    }

    fn event_mut(&mut self, event_idx: i32) -> Option<&mut EventRecord> {
        if event_idx < 0 || event_idx >= self.event_num() {
            return None;
        }
        self.events.get_mut(event_idx as usize)
    }

    pub fn event_at(&self, idx: usize) -> Option<&EventRecord> {
        self.events.get(idx)
    }

    pub fn staff_at(&self, staff_idx: i32) -> Option<&StaffRecord> {
        if staff_idx < 0 {
            return None;
        }
        self.staffs.get(staff_idx as usize)
    }

    fn staff_mut(&mut self, staff_idx: i32) -> Option<&mut StaffRecord> {
        if staff_idx < 0 {
            return None;
        }
        self.staffs.get_mut(staff_idx as usize)
    }

    fn cut_at(&self, cut_idx: i32) -> Option<&CutRecord> {
        if cut_idx < 0 || cut_idx >= self.cut_num() {
            return None;
        }
        self.cuts.get(cut_idx as usize)
    }

    pub fn cut_staff_current(&self, staff_idx: i32) -> Option<&CutRecord> {
        let staff = self.staff_at(staff_idx)?;
        self.cut_at(staff.current_cut() as i32)
    }

    pub fn cut_staff_start(&self, staff_idx: i32) -> Option<&CutRecord> {
        let staff = self.staff_at(staff_idx)?;
        self.cut_at(staff.start_cut() as i32)
    }

    pub fn get_event_idx(&self, event_name: Option<&str>, event_info_idx: u8) -> i32 {
        if !self.loaded() {
            return -1;
        }
        let info = usize::from(event_info_idx);
        if event_info_idx != 0xFF && info < self.event_info_names.len() {
            let name = self.event_info_names[info].clone();
            return self.get_event_idx(Some(&name), 0xFF);
        }
        let Some(event_name) = event_name else {
            return -1;
        };
        for (index, event) in self.events.iter().enumerate() {
            if event.name() == event_name {
                return index as i32;
            }
        }
        -1
    }

    pub fn existence(&self, event_idx: i32) -> bool {
        self.event_data(event_idx).is_some()
    }

    pub fn existence_name(&self, event_name: &str) -> bool {
        self.event_data(self.get_event_idx(Some(event_name), 0xFF))
            .is_some()
    }

    pub fn order(&mut self, event_idx: i32) -> bool {
        let Some(event) = self.event_mut(event_idx) else {
            return false;
        };
        event.state = STATE_ORDER;
        true
    }

    pub fn start_check(&self, event_idx: i32) -> bool {
        self.event_data(event_idx)
            .is_some_and(|event| event.state == STATE_PLAY)
    }

    pub fn start_check_old(&self, event_name: &str) -> bool {
        self.start_check(self.get_event_idx(Some(event_name), 0xFF))
    }

    pub fn end_check(&self, event_idx: i32) -> bool {
        self.event_data(event_idx)
            .is_some_and(|event| event.state == STATE_CLOSE)
    }

    pub fn end_check_old(&self, event_name: &str) -> bool {
        self.end_check(self.get_event_idx(Some(event_name), 0xFF))
    }

    pub fn get_event_prio(&self, event_idx: i32) -> u32 {
        self.event_data(event_idx).map_or(0, |event| event.priority)
    }

    pub fn get_event_end_sound(&self, event_idx: i32) -> u8 {
        self.event_data(event_idx)
            .map_or(0, |event| event.end_sound)
    }

    pub fn close_proc(&mut self, event_idx: i32) {
        if let Some(event) = self.event_mut(event_idx) {
            event.state = STATE_CLOSE;
        }
    }

    pub fn start_proc(&mut self, event_idx: i32) {
        let Some(event) = self.event_data(event_idx) else {
            return;
        };
        let mut list = [0u32; EVENT_STAFF_NUM];
        let count = event.n_staff.max(0) as usize;
        let count = count.min(EVENT_STAFF_NUM);
        for index in 0..count {
            list[index] = event.staff_idx[index];
        }
        for index in 0..count {
            self.staff_init(list[index] as i32);
        }
        if let Some(event) = self.event_mut(event_idx) {
            event.state = STATE_PLAY;
        }
        self.flags.init();
    }

    pub fn end_proc(&mut self, event_idx: i32, act: bool) {
        if self.event_data(event_idx).is_none() {
            return;
        }
        if act {
            self.close_proc(event_idx);
        }
        if self
            .event_data(event_idx)
            .is_some_and(|event| event.state == STATE_CLOSE)
        {
            self.camera_play = 0;
            if let Some(event) = self.event_mut(event_idx) {
                event.state = STATE_NONE;
            }
        }
    }

    pub fn main_proc(&mut self) {
        if !self.loaded() {
            return;
        }
        let count = (self.event_num().max(0) as usize).min(self.events.len());
        for index in 0..count {
            if self.events[index].state == STATE_PLAY && self.finish_check(index as i32) {
                self.close_proc(index as i32);
            }
        }
        for index in 0..count {
            if self.events[index].state == STATE_ORDER {
                self.start_proc(index as i32);
            }
        }
        for index in 0..count {
            if self.events[index].state == STATE_PLAY {
                self.advance_cut(index as i32);
            }
        }
        for index in 0..count {
            if self.events[index].state == STATE_PLAY {
                self.special_staff_proc(index as i32);
            }
        }
    }

    pub fn finish_check(&self, event_idx: i32) -> bool {
        let Some(event) = self.event_data(event_idx) else {
            return false;
        };
        for slot in event.check_finish.iter().take(EVENT_CHECK_FINISH_NUM) {
            let flag_id = *slot;
            if flag_id == -1 {
                return true;
            }
            if !self.flags.check(flag_id) {
                return false;
            }
        }
        true
    }

    pub fn cut_start_check(&self, cut_idx: i32) -> i32 {
        let Some(cut) = self.cut_at(cut_idx) else {
            return 0;
        };
        for index in 0..CUT_START_FLAG_NUM {
            let Some(flag_id) = cut.start_flag(index) else {
                continue;
            };
            if flag_id == -1 {
                return if index == 0 { -1 } else { 1 };
            }
            if !self.flags.check(flag_id) {
                return 0;
            }
        }
        1
    }

    pub fn advance_cut(&mut self, event_idx: i32) {
        let Some(event) = self.event_data(event_idx) else {
            return;
        };
        let mut list = [0u32; EVENT_STAFF_NUM];
        let count = event.n_staff.max(0) as usize;
        let count = count.min(EVENT_STAFF_NUM);
        for index in 0..count {
            list[index] = event.staff_idx[index];
        }
        for index in 0..count {
            self.advance_cut_local(list[index] as i32);
        }
    }

    pub fn advance_cut_local(&mut self, staff_idx: i32) -> bool {
        let Some(cut) = self.cut_staff_current(staff_idx) else {
            return false;
        };
        let next = cut.next_cut();
        let flag_id = cut.flag_id();
        if next != -1 {
            match self.cut_start_check(next) {
                -1 => {
                    if self.flags.check(flag_id) {
                        self.staff_advance_cut(staff_idx, next);
                        return true;
                    }
                }
                1 => {
                    self.flags.set(flag_id);
                    self.staff_advance_cut(staff_idx, next);
                    return true;
                }
                _ => {}
            }
        }
        if let Some(staff) = self.staff_mut(staff_idx) {
            staff.advance = if staff.advance > ADVANCE_START {
                ADVANCE_CUT
            } else {
                ADVANCE_STOP
            };
        }
        false
    }

    pub fn staff_init(&mut self, staff_idx: i32) {
        let Some(staff) = self.staff_mut(staff_idx) else {
            return;
        };
        staff.cur_cut = staff.first_cut;
        staff.timer = 0;
        staff.wipe_direction = 0;
        staff.field_45 = 0;
        staff.field_44 = 0;
        staff.advance = ADVANCE_START;
        staff.has_action = false;
        staff.cur_action = ACTION_NONE;
        let camera = staff.staff_type == STAFF_CAMERA;
        if camera {
            self.camera_play = 1;
        }
    }

    fn staff_advance_cut(&mut self, staff_idx: i32, cut_idx: i32) {
        let Some(staff) = self.staff_mut(staff_idx) else {
            return;
        };
        staff.cur_cut = cut_idx as u32;
        staff.timer = 0;
        staff.wipe_direction = 0;
        staff.field_45 = 0;
        staff.field_44 = 0;
        staff.advance = ADVANCE_CUT;
        staff.has_action = false;
        staff.cur_action = ACTION_NONE;
    }

    pub fn special_staff_proc(&mut self, event_idx: i32) {
        let Some(event) = self.event_data(event_idx) else {
            return;
        };
        let mut list = [0u32; EVENT_STAFF_NUM];
        let count = event.n_staff.max(0) as usize;
        let count = count.min(EVENT_STAFF_NUM);
        for index in 0..count {
            list[index] = event.staff_idx[index];
        }
        for index in 0..count {
            self.special_proc(list[index] as i32);
        }
    }

    fn special_proc(&mut self, staff_idx: i32) {
        let Some(staff) = self.staff_at(staff_idx) else {
            return;
        };
        match staff.staff_type {
            STAFF_TIMEKEEPER => self.special_proc_timekeeper(),
            STAFF_ALL => self.cut_end(staff_idx),
            _ => {}
        }
    }

    fn special_proc_timekeeper(&mut self) {
        const ACTION_TABLE: [&str; 2] = ["COUNTDOWN", "WAIT"];
        let staff_id = self.get_my_staff_id("TIMEKEEPER", None, 0);
        if staff_id == -1 {
            return;
        }
        let act_idx = self.get_my_act_idx(staff_id, &ACTION_TABLE, false, 0);
        if self.get_is_advance(staff_id) {
            match act_idx {
                0 => {
                    if let Some(timer) = self.get_my_int(staff_id, TIMER_NAME) {
                        if let Some(staff) = self.staff_mut(staff_id) {
                            staff.timer = timer as i16;
                        }
                    }
                }
                1 => {}
                _ => {}
            }
        }
        match act_idx {
            0 => {
                let timer = self.staff_at(staff_id).map(|staff| staff.timer);
                if let Some(timer) = timer {
                    if timer > 0 {
                        if let Some(staff) = self.staff_mut(staff_id) {
                            staff.timer -= 1;
                        }
                    } else {
                        self.cut_end(staff_id);
                    }
                }
            }
            1 => self.cut_end(staff_id),
            -1 => self.cut_end(staff_id),
            _ => {}
        }
    }

    pub fn get_my_staff_id(&self, name: &str, actor: Option<StaffActor>, tag_id: i32) -> i32 {
        if !self.run_check {
            return -1;
        }
        if !self.loaded() {
            return -1;
        }
        if let Some(actor) = actor {
            if !actor.force_move {
                return -1;
            }
        }
        for event in self.events.iter() {
            if event.state != STATE_PLAY && event.state != STATE_CLOSE && event.state != STATE_UNK3
            {
                continue;
            }
            let count = event.n_staff.max(0) as usize;
            let count = count.min(EVENT_STAFF_NUM);
            for index in 0..count {
                let staff_idx = event.staff_idx[index];
                let Some(staff) = self.staff_at(staff_idx as i32) else {
                    continue;
                };
                if staff.staff_type != STAFF_ALL && staff.name() == name && staff.tag_id == tag_id {
                    return staff_idx as i32;
                }
            }
        }
        -1
    }

    pub fn get_is_advance(&self, staff_idx: i32) -> bool {
        self.staff_at(staff_idx)
            .is_some_and(|staff| staff.advance != 0)
    }

    pub fn get_my_now_cut_name(&self, staff_idx: i32) -> Option<&str> {
        Some(self.cut_staff_current(staff_idx)?.name())
    }

    pub fn get_my_act_name(&self, staff_idx: i32) -> Option<&str> {
        self.get_my_now_cut_name(staff_idx)
    }

    pub fn get_my_act_idx(
        &mut self,
        staff_idx: i32,
        action: &[&str],
        force: bool,
        name_type: i32,
    ) -> i32 {
        if staff_idx == -1 {
            return -1;
        }
        let Some(staff) = self.staff_at(staff_idx) else {
            return -1;
        };
        if staff.has_action && !force {
            return staff.cur_action as i32;
        }
        if self.get_my_now_cut_name(staff_idx).is_none() {
            return -1;
        }
        if let Some(staff) = self.staff_mut(staff_idx) {
            staff.has_action = true;
        }
        for (index, entry) in action.iter().enumerate() {
            let Some(name) = self.get_my_now_cut_name(staff_idx) else {
                return -1;
            };
            let hit = if name_type != 0 {
                name_prefix_cmp(entry, name) == 0
            } else {
                *entry == name
            };
            if hit {
                if let Some(staff) = self.staff_mut(staff_idx) {
                    staff.cur_action = index as u32;
                }
                return index as i32;
            }
        }
        if let Some(staff) = self.staff_mut(staff_idx) {
            staff.cur_action = ACTION_NONE;
        }
        -1
    }

    pub fn get_my_data(&self, staff_idx: i32, name: &str, mode: i32) -> Option<&DataRecord> {
        if staff_idx == -1 {
            return None;
        }
        let cut = if mode != 0 {
            self.cut_staff_start(staff_idx)
        } else {
            self.cut_staff_current(staff_idx)
        }?;
        let mut data_idx = cut.data_top();
        while data_idx != -1 {
            let Some(data) = self.data_at(data_idx) else {
                return None;
            };
            if data.name() == name {
                return Some(data);
            }
            data_idx = data.next_data();
        }
        None
    }

    fn data_at(&self, data_idx: i32) -> Option<&DataRecord> {
        if data_idx < 0 {
            return None;
        }
        self.data.get(data_idx as usize)
    }

    fn substance_ok(data: &DataRecord) -> bool {
        data.index >= 0 && data.substance_size > 0
    }

    pub fn substance_type(&self, staff_idx: i32, name: &str) -> Option<i32> {
        Some(self.get_my_data(staff_idx, name, 0)?.substance_type)
    }

    pub fn get_my_float(&self, staff_idx: i32, name: &str) -> Option<f32> {
        let data = self.get_my_data(staff_idx, name, 0)?;
        if !Self::substance_ok(data) {
            return None;
        }
        let index = usize::try_from(data.index).ok()?;
        self.floats.get(index).copied()
    }

    pub fn get_my_vec(&self, staff_idx: i32, name: &str) -> Option<Vec3> {
        let data = self.get_my_data(staff_idx, name, 0)?;
        if !Self::substance_ok(data) {
            return None;
        }
        let index = usize::try_from(data.index).ok()?;
        let x = *self.floats.get(index)?;
        let y = *self.floats.get(index + 1)?;
        let z = *self.floats.get(index + 2)?;
        Some(Vec3::new(x, y, z))
    }

    /// The whole float array behind a key rather than the one value [`EventManager::get_my_float`]
    /// reads off its front, which is what the event camera's `getEvFloatListData` reaches for. A
    /// key that declares more floats than the pool holds stops at the pool's end rather than
    /// reading past it.
    pub fn get_my_floats(&self, staff_idx: i32, name: &str) -> Option<&[f32]> {
        let data = self.get_my_data(staff_idx, name, 0)?;
        if !Self::substance_ok(data) {
            return None;
        }
        self.float_range(data)
    }

    /// The same array read three floats at a time, which is what the event camera's
    /// `getEvXyzListData` reaches for, and here the count that spans it is in points, not floats.
    /// Neither game has that camera getter's own body, so the unit is read off its one caller
    /// that iterates: `bSplineEvCamera` hands the substance count straight to the curve as its
    /// key count, `sibling d_ev_camera.cpp:3487` and `:3497`, and the curve reads that list as
    /// points, `d_spline_path.cpp:65-72`. So the span is three floats per count. A key whose
    /// trailing group is short keeps its floats and drops the remainder, which is all the
    /// reference's unchecked read can do here.
    pub fn get_my_vecs(&self, staff_idx: i32, name: &str) -> Option<Vec<Vec3>> {
        let data = self.get_my_data(staff_idx, name, 0)?;
        if !Self::substance_ok(data) {
            return None;
        }
        let count = data.substance_size.max(0) as usize;
        let floats = self.float_span(data, count.checked_mul(3)?)?;
        Some(
            floats
                .chunks_exact(3)
                .map(|group| Vec3::new(group[0], group[1], group[2]))
                .collect(),
        )
    }

    fn float_span(&self, data: &DataRecord, span: usize) -> Option<&[f32]> {
        let index = usize::try_from(data.index).ok()?;
        let end = index.checked_add(span)?;
        self.floats.get(index..end.min(self.floats.len()))
    }

    fn float_range(&self, data: &DataRecord) -> Option<&[f32]> {
        let span = data.substance_size.max(0) as usize;
        self.float_span(data, span)
    }

    pub fn get_my_int(&self, staff_idx: i32, name: &str) -> Option<i32> {
        let data = self.get_my_data(staff_idx, name, 0)?;
        if !Self::substance_ok(data) {
            return None;
        }
        let index = usize::try_from(data.index).ok()?;
        self.ints.get(index).copied()
    }

    pub fn get_my_ints(&self, staff_idx: i32, name: &str) -> &[i32] {
        let Some(data) = self.get_my_data(staff_idx, name, 0) else {
            return &[];
        };
        if !Self::substance_ok(data) {
            return &[];
        }
        self.int_range(data).map_or(&[], |range| range)
    }

    fn int_range(&self, data: &DataRecord) -> Option<&[i32]> {
        let index = usize::try_from(data.index).ok()?;
        let count = data.substance_size.max(0) as usize;
        let end = index.checked_add(count)?;
        self.ints.get(index..end)
    }

    pub fn get_my_text(&self, staff_idx: i32, name: &str) -> Option<&str> {
        let data = self.get_my_data(staff_idx, name, 0)?;
        if !Self::substance_ok(data) {
            return None;
        }
        let index = usize::try_from(data.index).ok()?;
        let size = data.substance_size.max(0) as usize;
        let end = index.checked_add(size)?;
        let bytes = self.text.get(index..end)?;
        Some(read_bytes(bytes))
    }

    pub fn get_my_substance_num(&self, staff_idx: i32, name: &str) -> i32 {
        self.get_my_data(staff_idx, name, 0)
            .map_or(0, |data| data.substance_size)
    }

    pub fn cut_end(&mut self, staff_idx: i32) {
        let Some(cut) = self.cut_staff_current(staff_idx) else {
            return;
        };
        let flag_id = cut.flag_id();
        self.flags.set(flag_id);
    }

    pub fn set_prm_staff(&self, staff_idx: i32) -> Option<StaffSearchParam> {
        let staff = self.staff_at(staff_idx)?;
        let mut prm = StaffSearchParam {
            name: staff.name().to_string(),
            mask: 0,
            value: 0,
            cast_in_flag: 0,
        };
        if self.get_my_data(staff_idx, DEV_PRM_MASK_NAME, 1).is_some() {
            if let Some(values) = self.ints_of(staff_idx, DEV_PRM_MASK_NAME) {
                let mask = values.first().copied().unwrap_or(0) as u32;
                let value = values.get(1).copied().unwrap_or(0) as u32;
                prm.mask = mask;
                prm.value = value;
            }
        }
        if self.get_my_data(staff_idx, CAST_IN_NAME, 1).is_some() {
            prm.cast_in_flag |= CAST_IN_FLAG;
        }
        Some(prm)
    }

    fn ints_of(&self, staff_idx: i32, name: &str) -> Option<&[i32]> {
        let data = self.get_my_data(staff_idx, name, 1)?;
        if !Self::substance_ok(data) {
            return None;
        }
        self.int_range(data)
    }

    pub fn present_action(&mut self, trigger: i32) -> i32 {
        let staff_id = self.get_my_staff_id(TALKMAN_NAME, None, 0);
        if staff_id == -1 {
            return -1;
        }
        let act_idx = self.get_my_act_idx(staff_id, &PRESENT_ACTION_TABLE, false, 0);
        if act_idx == trigger {
            self.cut_end(staff_id);
        }
        act_idx
    }

    pub fn chk_present_end(&mut self) -> bool {
        self.present_action(PRESENT_ACT_WAIT) >= PRESENT_ACT_TALK0
    }

    pub fn cancel_present(&mut self) -> bool {
        self.present_action(PRESENT_ACT_TALK0) == PRESENT_ACT_TALK0
    }
}

fn read_bytes(bytes: &[u8]) -> &str {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    core::str::from_utf8(&bytes[..end]).unwrap_or("")
}

pub fn load_stage_event_list(root: &AssetRoot, stage: &str) -> Option<EventManager> {
    let relative = format!("{STAGE_RES_ROOT}/{stage}/{STAGE_RES_ARCHIVE}");
    let archive = match root.read_archive(&relative) {
        Ok(archive) => archive,
        Err(error) => {
            log::warn!("event_manager: stage archive unavailable ({error})");
            return None;
        }
    };
    match archive.read(EVENT_LIST_ENTRY) {
        Ok(data) => {
            let mut manager = EventManager::default();
            manager.create(Some(data));
            Some(manager)
        }
        Err(error) => {
            log::warn!("event_manager: {relative} has no {EVENT_LIST_ENTRY} ({error})");
            None
        }
    }
}

pub fn control(mut manager: ResMut<EventManager>) {
    manager.execute();
}

pub fn init(world: &mut World) {
    world.init_resource::<EventManager>();
}
