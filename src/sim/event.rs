use bevy_ecs::prelude::*;

use super::flags::{EventFlags, StoryBits};
use super::transition::{NextStage, Target};

pub const EVENT_LIST_ENTRY: &str = "dat/event_list.dat";

const HEADER_LEN: usize = 0x40;
const SECTIONS: usize = 7;
const EVENT_LEN: usize = 176;
const STAFF_LEN: usize = 80;
const CUT_LEN: usize = 80;
const DATA_LEN: usize = 64;
const NAME_LEN: usize = 32;
const VALUE_LEN: usize = 4;
const TYPE_FLOAT: i32 = 0;
const TYPE_INT: i32 = 3;
const TYPE_STRING: i32 = 4;
const STAFF_NORMAL: i32 = 0;
const STAFF_ALL: i32 = 1;
const STAFF_CAMERA: i32 = 2;
const STAFF_TIMEKEEPER: i32 = 4;
const STAFF_DIRECTOR: i32 = 6;
const STAFF_MESSAGE: i32 = 7;
const STAFF_SOUND: i32 = 8;
const STAFF_LIGHT: i32 = 9;
const STAFF_PACKAGE: i32 = 11;
const STAFF_CREATE: i32 = 12;
const TIMEKEEPER_ACTIONS: &[&str] = &["COUNTDOWN", "WAIT"];
const DIRECTOR_ACTIONS: &[&str] = &[
    "WAIT",
    "NEXT",
    "FADE",
    "BGM_START",
    "VIBRATION",
    "SE_START",
    "WIPE",
    "PLAYER_DRAW",
    "PLAYER_NODRAW",
];
const MESSAGE_ACTIONS: &[&str] = &[
    "WAIT",
    "CREATE_MSG",
    "PUSHBUTTON",
    "FINISH",
    "CONTINUE",
    "END",
    "DELETE",
    "TELOP_ON",
    "TELOP_OFF",
];
const SOUND_ACTIONS: &[&str] = &[
    "WAIT",
    "STRM_PLAY",
    "STRM_DEMO_MJ_SISTER",
    "STRM_DEMO_GO_MAJU",
    "NOMSG_WAIT",
    "RIDDLE",
    "LANDING_DEMO",
    "BGMSTOP",
];
const PACKAGE_ACTIONS: &[&str] = &["WAIT", "PLAY", "PLAY2"];
const CAMERA_ACTIONS: &[&str] = &[
    "PAUSE",
    "WAIT",
    "TALK",
    "FIXEDPOS",
    "FIXEDFRM",
    "UNITRANS",
    "WATCHACTOR",
    "RESTOREPOS",
    "GETITEM",
    "GAMEOVER",
    "TURNTOACTOR",
    "ROLLING",
    "TACT",
    "WINDDIR",
    "TORNADO",
    "STYLE",
    "SAVE",
    "LOAD",
    "USEITEM0",
    "USEITEM1",
    "FIXEDFRMS",
    "BSPLINE",
    "POSSESSED",
    "TWOACTOR0",
    "STOKER",
    "UNIBRAKE",
    "UNIACCELE",
    "MAPTOOL",
];
const SOLDIER_INSTANT: &[&str] = &[
    "EFFECT",
    "EQUIP",
    "UNEQUIP",
    "MINIGAME_WAIT",
    "PARTNER_SHIP",
];
const SOLDIER_PENDING: &[&str] = &[
    "MOVE_TO_ACTOR",
    "TURN_TO_ACTOR",
    "MOVE_TO_POS",
    "TURN_TO_POS",
    "SWIM",
    "JUMP",
    "APPEAR",
    "DIVE",
    "DISAPPEAR",
    "JUMP_MAPOPEN",
    "TURN",
    "TALK_MSG",
    "CONTINUE_TALK",
    "MINIGAME",
    "MINIGAME_WARP",
    "MINIGAME_RETURN",
    "MINIGAME_END",
    "MINIGAME_PL_TURN",
    "MINIGAME_PL_UP",
    "EATESA",
    "EATESA1ST",
];
const LINK_INSTANT: &[i32] = &[1, 4, 6, 14, 17, 18, 20, 23, 39, 42, 43, 44, 51, 55];

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct EventList {
    sections: Vec<Section>,
    pub events: Vec<Event>,
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
pub struct Event {
    pub name: String,
    pub priority: u32,
    pub staffs: Vec<u32>,
    pub finish_flags: [i32; 3],
    pub end_sound: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Staff {
    pub kind: i32,
    pub first_cut: u32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cut {
    pub name: String,
    pub start_flags: [i32; 3],
    pub flag_id: i32,
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

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct ActiveEvent {
    running: Vec<Running>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Order,
    Play,
    Close,
}

#[derive(Debug, Clone, PartialEq)]
struct Running {
    index: usize,
    stage: Stage,
    staffs: Vec<StaffRun>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StaffRun {
    staff: usize,
    cut: i32,
    timer: i16,
    frames: u16,
    advance: u8,
    wipe: u8,
    has_action: bool,
    action: i32,
}

impl ActiveEvent {
    pub fn order(&mut self, list: &EventList, index: usize) -> bool {
        if list.event(index).is_none() {
            return false;
        }
        self.running.retain(|running| running.index != index);
        let position = self
            .running
            .partition_point(|running| running.index < index);
        self.running.insert(
            position,
            Running {
                index,
                stage: Stage::Order,
                staffs: Vec::new(),
            },
        );
        true
    }

    pub fn stage_of(&self, index: usize) -> Option<Stage> {
        self.running
            .iter()
            .find(|running| running.index == index)
            .map(|running| running.stage)
    }

    pub fn start_check(&self, index: usize) -> bool {
        self.stage_of(index) == Some(Stage::Play)
    }

    pub fn end_check(&self, index: usize) -> bool {
        self.stage_of(index) == Some(Stage::Close)
    }

    pub fn end(&mut self, index: usize) -> bool {
        let before = self.running.len();
        self.running.retain(|running| running.index != index);
        self.running.len() != before
    }

    pub fn cut_of(&self, index: usize, staff: usize) -> Option<i32> {
        let running = self.running.iter().find(|running| running.index == index)?;
        running.staffs.get(staff).map(|run| run.cut)
    }
}

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

fn int_of(value: Value) -> Option<i32> {
    match value {
        Value::Float(number) => Some(number as i32),
        Value::Int(number) => Some(number),
        Value::Raw(number) => Some(number),
        Value::Text => None,
    }
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
        let event_at = offsets(sections[0], EVENT_LEN)?;
        for at in &event_at {
            reader.bytes(*at, EVENT_LEN)?;
        }
        let events = event_at
            .iter()
            .map(|at| {
                let staff_count = reader.i32(*at + 0x7C)?.clamp(0, 20);
                let staffs = (0..staff_count as usize)
                    .map(|slot| reader.u32(*at + 0x2C + slot * 4))
                    .collect::<Result<Vec<_>, EventError>>()?;
                Ok(Event {
                    name: reader.text(*at, NAME_LEN)?,
                    priority: reader.u32(*at + 0x28)?,
                    staffs,
                    finish_flags: [
                        reader.i32(*at + 0x88)?,
                        reader.i32(*at + 0x8C)?,
                        reader.i32(*at + 0x90)?,
                    ],
                    end_sound: reader.bytes(*at + 0x94, 1)?[0],
                })
            })
            .collect::<Result<Vec<_>, EventError>>()?;
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
                    name: reader.text(*at, NAME_LEN)?,
                    start_flags: [
                        reader.i32(*at + 0x28)?,
                        reader.i32(*at + 0x2C)?,
                        reader.i32(*at + 0x30)?,
                    ],
                    flag_id: reader.i32(*at + 0x34)?,
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
            events,
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

    pub fn event(&self, index: usize) -> Option<&Event> {
        self.events.get(index)
    }

    pub fn event_index(&self, name: &str) -> Option<usize> {
        self.events.iter().position(|event| event.name == name)
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

    fn record_of(&self, first_data: u32, key: &str) -> Option<&DataRecord> {
        let mut index = first_data as i32;
        let mut guard = 0_usize;
        while index >= 0 && guard <= self.data.len() {
            let record = self.data.get(index as usize)?;
            if record.name == key {
                return Some(record);
            }
            index = record.next;
            guard += 1;
        }
        None
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
            .and_then(|(_, value)| int_of(value))
    }

    pub fn cut_number(&self, cut: usize, key: &str) -> Option<i32> {
        let cut = self.cuts.get(cut)?;
        let record = self.record_of(cut.first_data, key)?;
        int_of(self.value(record))
    }

    pub fn cut_float(&self, cut: usize, key: &str) -> Option<f32> {
        let cut = self.cuts.get(cut)?;
        let record = self.record_of(cut.first_data, key)?;
        match self.value(record) {
            Value::Float(number) => Some(number),
            Value::Int(number) => Some(number as f32),
            Value::Raw(_) | Value::Text => None,
        }
    }

    pub fn cut_text(&self, cut: usize, key: &str) -> Option<&str> {
        let cut = self.cuts.get(cut)?;
        let record = self.record_of(cut.first_data, key)?;
        if record.kind != TYPE_STRING {
            return None;
        }
        let raw = self.text.get(record.index as usize..)?;
        let stop = raw.iter().position(|&byte| byte == 0).unwrap_or(raw.len());
        std::str::from_utf8(&raw[..stop]).ok()
    }
}

pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut flags: ResMut<EventFlags>,
    mut story: ResMut<StoryBits>,
    mut next: ResMut<NextStage>,
) {
    for running in active.running.iter_mut() {
        if running.stage == Stage::Play
            && list
                .event(running.index)
                .is_some_and(|event| finish_check(event, &flags))
        {
            running.stage = Stage::Close;
        }
    }
    let mut started = false;
    for running in active.running.iter_mut() {
        if running.stage == Stage::Order {
            if let Some(event) = list.event(running.index) {
                start_proc(running, event, &list);
                started = true;
            }
        }
    }
    if started {
        *flags = EventFlags::default();
    }
    for running in active.running.iter_mut() {
        if running.stage != Stage::Play {
            continue;
        }
        for run in running.staffs.iter_mut() {
            advance_cut(run, &list, &mut flags);
        }
    }
    for running in active.running.iter_mut() {
        if running.stage != Stage::Play {
            continue;
        }
        for run in running.staffs.iter_mut() {
            special_proc(run, &list, &mut flags, &mut story, &mut next);
        }
    }
    for running in active.running.iter_mut() {
        if running.stage != Stage::Play {
            continue;
        }
        for run in running.staffs.iter_mut() {
            staff_cut(run, &list, &mut flags);
        }
    }
}

fn staff_cut(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags) {
    let Some(staff) = list.staffs.get(run.staff) else {
        return;
    };
    if staff.kind == STAFF_CAMERA {
        camera_cut(run, list, flags);
    } else if staff.kind == STAFF_NORMAL {
        match staff.name.as_str() {
            "NpcSo" => soldier_cut(run, list, flags),
            "Link" => link_cut(run, list, flags),
            "Ship" => ship_cut(run, list, flags),
            _ => {}
        }
    }
}

fn soldier_cut(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    match cut.name.as_str() {
        "WAIT" => {
            if run.advance > 0 {
                wait_start(run, list);
            }
            if run.timer > 0 {
                run.timer -= 1;
            }
            if run.timer == 0 {
                cut_end(run, list, flags);
            }
        }
        "SET_ANM" => match list.cut_text(run.cut as usize, "Name") {
            Some(name) if matches!(name, "WAIT" | "TALK" | "JUMP" | "SWIM") => {
                cut_end(run, list, flags)
            }
            Some(_) => {}
            None => cut_end(run, list, flags),
        },
        name if SOLDIER_INSTANT.contains(&name) => cut_end(run, list, flags),
        name if SOLDIER_PENDING.contains(&name) => {}
        _ => cut_end(run, list, flags),
    }
}

fn link_cut(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    if leading_digits(&cut.name, 3).is_some_and(|mode| LINK_INSTANT.contains(&mode)) {
        cut_end(run, list, flags);
    }
}

fn ship_cut(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    if matches!(leading_digits(&cut.name, 2), Some(0 | 5 | 8)) {
        cut_end(run, list, flags);
    }
}

fn leading_digits(name: &str, width: usize) -> Option<i32> {
    let digits = name.as_bytes().get(..width)?;
    let mut value = 0_i32;
    for digit in digits {
        if !digit.is_ascii_digit() {
            return None;
        }
        value = value * 10 + i32::from(digit - b'0');
    }
    Some(value)
}

fn camera_cut(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags) {
    let action = act_index(run, list, CAMERA_ACTIONS);
    let cut = run.cut as usize;
    let timer = match action {
        0 | 1 | 3 | 4 | 5 | 11 => list.cut_number(cut, "Timer"),
        8 => Some(
            list.cut_number(cut, "Timer")
                .unwrap_or_else(|| item_timer(list, cut)),
        ),
        9 | 10 | 12 | 16 | 23 | -1 => {
            cut_end(run, list, flags);
            return;
        }
        _ => return,
    };
    if run.advance > 0 {
        run.frames = 0;
    }
    match timer {
        Some(timer) if timer >= 0 => {
            if i32::from(run.frames) >= timer {
                cut_end(run, list, flags);
            }
            run.frames = run.frames.saturating_add(1);
        }
        _ => cut_end(run, list, flags),
    }
}

fn item_timer(list: &EventList, cut: usize) -> i32 {
    if list.cut_number(cut, "Type").unwrap_or(0) == 1 {
        17
    } else {
        12
    }
}

fn start_proc(running: &mut Running, event: &Event, list: &EventList) {
    running.staffs = event
        .staffs
        .iter()
        .filter_map(|index| {
            let staff = list.staffs.get(*index as usize)?;
            Some(StaffRun {
                staff: *index as usize,
                cut: staff.first_cut as i32,
                timer: 0,
                frames: 0,
                advance: 2,
                wipe: 0,
                has_action: false,
                action: -1,
            })
        })
        .collect();
    running.stage = Stage::Play;
}

fn finish_check(event: &Event, flags: &EventFlags) -> bool {
    for flag in event.finish_flags {
        if flag == -1 {
            return true;
        }
        if !flags.test(flag as u32) {
            return false;
        }
    }
    true
}

fn start_check(cut: &Cut, flags: &EventFlags) -> i32 {
    for (position, flag) in cut.start_flags.iter().enumerate() {
        if *flag == -1 {
            return if position == 0 { -1 } else { 1 };
        }
        if !flags.test(*flag as u32) {
            return 0;
        }
    }
    1
}

fn advance_cut(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags) -> bool {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        run.advance = if run.advance > 1 { 1 } else { 0 };
        return false;
    };
    if cut.next != -1 {
        let gate = list
            .cuts
            .get(cut.next as usize)
            .map(|next| start_check(next, flags))
            .unwrap_or(0);
        if gate == -1 {
            if flags.test(cut.flag_id as u32) {
                advance_to(run, cut.next);
                return true;
            }
        } else if gate == 1 {
            flags.set(cut.flag_id as u32);
            advance_to(run, cut.next);
            return true;
        }
    }
    run.advance = if run.advance > 1 { 1 } else { 0 };
    false
}

fn advance_to(run: &mut StaffRun, cut: i32) {
    run.cut = cut;
    run.timer = 0;
    run.frames = 0;
    run.wipe = 0;
    run.advance = 1;
    run.has_action = false;
    run.action = -1;
}

fn cut_end(run: &StaffRun, list: &EventList, flags: &mut EventFlags) {
    if let Some(cut) = list.cuts.get(run.cut as usize) {
        flags.set(cut.flag_id as u32);
    }
}

fn act_index(run: &mut StaffRun, list: &EventList, actions: &[&str]) -> i32 {
    if run.has_action {
        return run.action;
    }
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return -1;
    };
    run.has_action = true;
    run.action = actions
        .iter()
        .position(|action| *action == cut.name)
        .map(|position| position as i32)
        .unwrap_or(-1);
    run.action
}

fn wait_start(run: &mut StaffRun, list: &EventList) {
    run.timer = list.cut_number(run.cut as usize, "Timer").unwrap_or(0) as i16;
}

fn wait_proc(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags) {
    if run.timer > 0 {
        run.timer -= 1;
    } else {
        cut_end(run, list, flags);
    }
}

fn special_proc(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    story: &mut StoryBits,
    next: &mut NextStage,
) {
    let Some(staff) = list.staffs.get(run.staff) else {
        return;
    };
    match staff.kind {
        STAFF_ALL | STAFF_CREATE | STAFF_LIGHT => cut_end(run, list, flags),
        STAFF_TIMEKEEPER => timekeeper(run, list, flags),
        STAFF_DIRECTOR => director(run, list, flags, next),
        STAFF_MESSAGE => message(run, list, flags),
        STAFF_SOUND => sound(run, list, flags),
        STAFF_PACKAGE => package(run, list, flags, story),
        _ => {}
    }
}

fn timekeeper(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags) {
    let action = act_index(run, list, TIMEKEEPER_ACTIONS);
    if action == 0 {
        if run.advance > 0 {
            wait_start(run, list);
        }
        if run.timer > 0 {
            run.timer -= 1;
        } else {
            cut_end(run, list, flags);
        }
    } else {
        cut_end(run, list, flags);
    }
}

fn cut_target(run: &StaffRun, list: &EventList) -> Option<Target> {
    let stage = list.cut_text(run.cut as usize, "Stage")?.to_owned();
    let start = list.cut_number(run.cut as usize, "StartCode")?;
    let start = u8::try_from(start).ok()?;
    let room = match list.cut_number(run.cut as usize, "RoomNo") {
        Some(room) => u8::try_from(room).ok()?,
        None => 0,
    };
    Some(Target { stage, room, start })
}

fn director(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags, next: &mut NextStage) {
    let action = act_index(run, list, DIRECTOR_ACTIONS);
    if run.advance > 0 {
        match action {
            0 | 4 => wait_start(run, list),
            1 => {
                if let Some(target) = cut_target(run, list) {
                    next.target = Some(target);
                }
            }
            6 => {
                run.wipe = if list
                    .cut_float(run.cut as usize, "Rate")
                    .is_some_and(|rate| rate > 0.0)
                {
                    0
                } else {
                    1
                };
            }
            _ => {}
        }
    }
    match action {
        0 => wait_proc(run, list, flags),
        1 => {}
        2 => cut_end(run, list, flags),
        4 => wait_proc(run, list, flags),
        6 => {
            if run.wipe != 0 {
                cut_end(run, list, flags);
            }
        }
        _ => cut_end(run, list, flags),
    }
}

fn message(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags) {
    let action = act_index(run, list, MESSAGE_ACTIONS);
    if run.advance > 0 && (action == 0 || action == 7) {
        wait_start(run, list);
    }
    match action {
        0 | 7 => wait_proc(run, list, flags),
        3 | 4 | 6 | 8 => cut_end(run, list, flags),
        1 | 2 | 5 => {}
        _ => cut_end(run, list, flags),
    }
}

fn sound(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags) {
    let action = act_index(run, list, SOUND_ACTIONS);
    if run.advance > 0 && (action == 0 || action == 4) {
        wait_start(run, list);
    }
    match action {
        0 => wait_proc(run, list, flags),
        4 => {}
        _ => cut_end(run, list, flags),
    }
}

fn package(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags, story: &mut StoryBits) {
    let action = act_index(run, list, PACKAGE_ACTIONS);
    if run.advance > 0 {
        match action {
            0 => wait_start(run, list),
            1 | 2 => {
                if let Some(bit) = list.cut_number(run.cut as usize, "EventFlag") {
                    story.set(bit as u16);
                }
            }
            _ => {}
        }
    }
    match action {
        0 => wait_proc(run, list, flags),
        1 => cut_end(run, list, flags),
        2 => {}
        _ => cut_end(run, list, flags),
    }
}
