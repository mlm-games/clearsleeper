use bevy_ecs::prelude::*;
use glam::Vec3;
use std::collections::HashMap;

use super::actor::Actor;
use super::chest::ChestOpen;
use super::door::DoorCut;
use super::fire::FireLit;
use super::flags::{EventFlags, StoryBits};
use super::msg::{
    MsgBox, MsgTable, MSG_BOX_CLOSING, MSG_BOX_CLOSED, MSG_BOX_OPENING, MSG_CONTINUES,
    MSG_DESTROYED, MSG_DISPLAYED, MSG_ENDS, MSG_TYPING,
};
use super::player::Player;
use super::sea::Sea;
use super::ship::{DECK_RISE, GET_OFF_SPEED, LEDGE_OFFSET_X, Ship, SHIP_ACTOR};
use super::talk::Talk;
use super::transition::{NextStage, Target};
use super::walk::{add_calc, add_calc_angle_s, from_s16, to_s16};
use super::water;

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
const TYPE_VEC: i32 = 1;
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
    "MINIGAME",
    "MINIGAME_WARP",
    "MINIGAME_RETURN",
    "MINIGAME_END",
    "MINIGAME_PL_TURN",
    "MINIGAME_PL_UP",
    "EATESA",
    "EATESA1ST",
];
const LINK_INSTANT: &[i32] = &[
    1, 4, 6, 8, 14, 17, 18, 20, 22, 23, 28, 29, 30, 31, 32, 33, 39, 42, 43, 44, 47, 51, 52, 55, 57,
    58, 62, 66, 69, 72,
];
const GET_ITEM_MODE: i32 = 11;
const FAIRY_BOTTLE_ITEM: i32 = 0x57;
const HEART_PIECE_ITEM: u8 = 0x07;
const HEART_PIECE_ALT: u8 = 0x17;
pub const LINK_ANIM_STEMS: &[&str] = &[
    "binopena",
    "binopenb",
    "binopenpre",
    "boxopenlink",
    "boxopenshortlink",
    "boxopenslink",
    "combo_link",
    "comeout",
    "damf",
    "damffup",
    "dooropenalink",
    "dooropenblink",
    "esamaki",
    "getletter",
    "holdup",
    "itemget",
    "lookup",
    "pickup",
    "presentationa",
    "powup",
    "saltation",
    "seyyes",
    "sethyoinomi",
    "slipice",
    "surprised",
    "surprisedb",
    "taktdge",
    "turnback",
    "warpin",
    "warpout",
    "warpoutfirst",
    "windl",
];
const GET_ENTERED: u8 = 0x01;
const GET_DONE: u8 = 0x04;

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct StageEvents(pub Vec<String>);

impl StageEvents {
    pub fn name(&self, id: u8) -> Option<&str> {
        if id == 0xFF {
            return None;
        }
        self.0.get(usize::from(id)).map(String::as_str)
    }
}

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
    actor: Entity,
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
    prev_delta: i16,
    aim_vel: i16,
    speed: f32,
    phase: i32,
    model_offset: f32,
    offset_target: f32,
    offset_step: f32,
    offset_scale: f32,
    attn: Vec3,
    target: Option<Entity>,
    talk_open: bool,
    msg_bound: bool,
    msg_no: i32,
    get_stage: u8,
    get_msg_live: bool,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct ItemPartner {
    pub entity: Option<Entity>,
    pub item: u8,
    pub max_life: u8,
    pub move_angle: i16,
    pub demo_stick: Option<f32>,
    pub demo_fall: bool,
    pub demo_backjump: bool,
    pub demo_launch: bool,
    pub fall_cap: f32,
    pub demo_landed: bool,
}

const PRESENT_ACTOR: &str = "PresentItem";

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresentItem {
    pub item: u8,
    pub hidden: bool,
}

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct LinkAnms {
    ends: HashMap<String, (u8, u16)>,
}

impl LinkAnms {
    pub fn insert(&mut self, stem: &str, attribute: u8, frames: u16) {
        self.ends.insert(stem.to_owned(), (attribute, frames));
    }

    fn end_frame(&self, stem: &str) -> Option<f32> {
        let &(attribute, frames) = self.ends.get(stem)?;
        if attribute <= 1 && frames > 0 {
            Some(frames as f32)
        } else {
            None
        }
    }
}

impl ActiveEvent {
    pub fn order(&mut self, list: &EventList, index: usize, actor: Entity) -> bool {
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
                actor,
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

    pub fn end_staff_cut(&mut self, list: &EventList, flags: &mut EventFlags, name: &str) {
        let Some(slot) = list.staff_of(name) else {
            return;
        };
        let cut = self
            .running
            .iter()
            .filter_map(|running| running.staffs.get(slot))
            .find_map(|run| usize::try_from(run.cut).ok())
            .and_then(|cut| list.cuts.get(cut));
        if let Some(cut) = cut {
            flags.set(cut.flag_id as u32);
        }
    }

    pub fn staff_cut(&self, list: &EventList, name: &str) -> Option<(usize, bool)> {
        let slot = list.staff_of(name)?;
        for running in &self.running {
            let Some(run) = running.staffs.get(slot) else {
                continue;
            };
            if let Ok(index) = usize::try_from(run.cut) {
                return Some((index, run.advance != 0));
            }
        }
        None
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

    pub fn cut_name(&self, cut: usize) -> Option<&str> {
        Some(self.cuts.get(cut)?.name.as_str())
    }

    pub fn cut_count(&self) -> usize {
        self.cuts.len()
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

    pub fn cut_vec(&self, cut: usize, key: &str) -> Option<Vec3> {
        let cut = self.cuts.get(cut)?;
        let record = self.record_of(cut.first_data, key)?;
        if record.kind != TYPE_VEC {
            return None;
        }
        let at = record.index.max(0) as usize;
        Some(Vec3::new(
            *self.floats.get(at)?,
            *self.floats.get(at + 1)?,
            *self.floats.get(at + 2)?,
        ))
    }
}

pub fn control(
    data: (Res<EventList>, Res<MsgTable>),
    mut active: ResMut<ActiveEvent>,
    mut flags: ResMut<EventFlags>,
    mut story: ResMut<StoryBits>,
    mut next: ResMut<NextStage>,
    mut msg: ResMut<MsgBox>,
    mut partner: ResMut<ItemPartner>,
    anms: Res<LinkAnms>,
    sea: Option<Res<Sea>>,
    mut actors: Query<(Entity, &mut Actor)>,
    mut players: Query<(Entity, &mut Player)>,
    mut ships: Query<&mut Ship>,
    mut talks: Query<&mut Talk>,
    mut presents: Query<&mut PresentItem>,
    mut commands: Commands,
) {
    let (list, table) = data;
    partner.demo_stick = None;
    partner.demo_fall = false;
    partner.demo_backjump = false;
    partner.demo_launch = false;
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
            special_proc(run, &list, &mut flags, &mut story, &mut next, &mut msg, &table);
        }
    }
    for running in active.running.iter_mut() {
        if running.stage != Stage::Play {
            continue;
        }
        let owner = running.actor;
        for run in running.staffs.iter_mut() {
            staff_cut(
                run,
                &list,
                &mut flags,
                owner,
                &mut actors,
                &mut players,
                &mut ships,
                sea.as_deref(),
                &mut talks,
                &mut msg,
                &table,
                &mut partner,
                &anms,
                &mut presents,
                &mut commands,
            );
        }
    }
}

fn staff_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &mut Query<(Entity, &mut Actor)>,
    players: &mut Query<(Entity, &mut Player)>,
    ships: &mut Query<&mut Ship>,
    sea: Option<&Sea>,
    talks: &mut Query<&mut Talk>,
    msg: &mut MsgBox,
    table: &MsgTable,
    partner: &mut ItemPartner,
    anms: &LinkAnms,
    presents: &mut Query<&mut PresentItem>,
    commands: &mut Commands,
) {
    let Some(staff) = list.staffs.get(run.staff) else {
        return;
    };
    if staff.name == "TREASURE" {
        treasure_cut(run, list, flags, owner, actors, commands);
    } else if staff.name == "Fire" || staff.name == "Zenfire" {
        fire_cut(run, list, flags, owner, actors, commands);
    } else if staff.name == "TAGHINT" {
        hint_cut(run, list, flags, owner, actors, talks, msg, table, commands);
    } else if staff.name == "SWITCH" {
        switch_cut(run, list, flags, owner, actors, commands);
    } else if staff.name == "Warpt" {
        named_cut(run, list, flags, "WARP");
    } else if staff.name == "WINDMAN" {
        named_cut(run, list, flags, "WINDOW");
    } else if staff.name == "GIVEMAN" {
        cut_end(run, list, flags);
    } else if staff.name == "Bm1" {
        bm1_cut(run, list, flags, owner, actors);
    } else if staff.name == "MknjD" {
        statue_cut(run, list, flags, owner, actors, talks, msg, table, commands);
    } else if staff.name == "SHOKUDAI" {
        torch_cut(run, list, flags, owner, actors, commands);
    } else if staff.name == "Fmaster"
        && let Some(cut) = list.cuts.get(run.cut as usize)
        && matches!(cut.name.as_str(), "Dummy" | "WAIT")
    {
        cut_end(run, list, flags);
    } else if staff.name == "SHUTTER_DOOR"
        || super::door::Door::is_shutter(&staff.name)
        || super::door::Door::is_sliding(&staff.name)
        || super::door::Door::is_key_door(&staff.name)
    {
        door_cut(run, list, flags, owner, actors, commands);
    } else if staff.name == "MBDOOR" {
        mbdoor_cut(run, list, flags);
    } else if staff.name == "MD_MESSAGE_TAG" {
        open_msg_cut(
            run,
            list,
            flags,
            owner,
            actors,
            talks,
            msg,
            table,
            commands,
            ("msg_init", "message"),
            "msg_number",
            mailbox_msg(owner, actors),
        );
    } else if staff.name == "Os" || staff.name == "Os1" || staff.name == "Os2" {
        os_cut(run, list, flags, owner, actors, talks, msg, table, commands);
    } else if staff.name == "Mori1" {
        hold_cut(run, list, flags, &["CLOSE", "STOP_OPEN"]);
    } else if staff.name == "Ghrwp" {
        hold_cut(run, list, flags, &["WARP_ARRIVE_END"]);
    } else if staff.name == "Gdemo20" {
        cut_end(run, list, flags);
    } else if staff.name == "Warpgn" {
        hold_cut(run, list, flags, &["WARP_ARRIVE_END", "APPEAR"]);
    } else if staff.name == "Hsh" {
        hsh_cut(run, list, flags);
    } else if staff.name == "Bs1" {
        bs1_cut(run, list, flags, owner, actors, talks, msg, table, commands);
    } else if staff.name == "Doguu" {
        cut_end(run, list, flags);
    } else if staff.name == "Warpfo" {
        warpout_cut(run, list, flags);
    } else if staff.name == "Warpf" {
        warp_cut(run, list, flags);
    } else if people_staff(&staff.name) {
        people_cut(run, list, flags, owner, actors, talks, msg, table, commands);
    } else if staff.name == "Ls1" {
        hold_cut(run, list, flags, &["LOK_PLYER", "PLYER_MOV", "WAI", "ANM_CHG"]);
    } else if staff.name == "Ba1" {
        hold_cut(run, list, flags, &["ACTION", "START_TALE1"]);
    } else if staff.name == "Hdai1" || staff.name == "Hdai2" || staff.name == "Hdai3" {
        hold_cut(run, list, flags, &["MOVE"]);
    } else if staff.name == "Apzl" {
        hold_cut(run, list, flags, &["STOP", "PUSH_A"]);
    } else if staff.name == "Ji1" {
        ji1_cut(run, list, flags);
    } else if staff.name == "Tpost" {
        tpost_cut(run, list, flags, owner, actors, talks, msg, table, commands);
    } else if staff.name == "Auction" {
        auction_cut(run, list, flags, owner, actors, talks, msg, table, commands);
    } else if staff.name == "Zl1" {
        zl1_cut(run, list, flags);
    } else if staff.name == "Tc" {
        tc_cut(run, list, flags);
    } else if staff.name == "Mk" {
        mk_cut(run, list, flags, owner, actors, talks, msg, table, commands);
    } else if staff.name == "Roten" {
        roten_cut(run, list, flags, owner, actors, talks, msg, table, commands);
    } else if staff.kind == STAFF_CAMERA {
        camera_cut(run, list, flags);
    } else if staff.kind == STAFF_NORMAL {
        match staff.name.as_str() {
            "NpcSo" => soldier_cut(
                run, list, flags, owner, actors, players, sea, talks, msg, table, commands,
            ),
            "Link" => link_cut(
                run, list, flags, actors, ships, msg, table, partner, anms, players, commands,
                presents,
            ),
            "Ship" => ship_cut(run, list, flags, actors, ships),
            _ => {}
        }
    }
}

fn warp_cut(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    match cut.name.as_str() {
        "WARP_1" => {
            if run.advance > 0 {
                run.timer = 180;
            }
            wait_proc(run, list, flags);
        }
        "WARP_2" | "WARP_3" => {
            if run.advance > 0 {
                run.timer = 5;
            }
            wait_proc(run, list, flags);
        }
        _ => cut_end(run, list, flags),
    }
}

fn mailbox_msg(owner: Entity, actors: &Query<(Entity, &mut Actor)>) -> u32 {
    actors.get(owner).map_or(0, |(_, actor)| actor.parameters & 0xFFFF)
}

fn open_msg_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &Query<(Entity, &mut Actor)>,
    talks: &mut Query<&mut Talk>,
    msg: &mut MsgBox,
    table: &MsgTable,
    commands: &mut Commands,
    names: (&str, &str),
    key: &str,
    fallback: u32,
) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    match cut.name.as_str() {
        name if name == names.0 => {
            let Ok(mut talk) = talks.get_mut(owner) else {
                host_message(owner, actors, commands);
                return;
            };
            if run.advance > 0 {
                talk.msg_curr =
                    list.cut_number(run.cut as usize, key).unwrap_or(fallback as i32) as u32;
                talk.msg_end = -1;
                run.talk_open = true;
            }
            if !run.talk_open {
                cut_end(run, list, flags);
                return;
            }
            npc_talk(&mut talk, msg, table);
            if talk.msg_live {
                cut_end(run, list, flags);
            }
        }
        name if name == names.1 => {
            let Ok(mut talk) = talks.get_mut(owner) else {
                host_message(owner, actors, commands);
                return;
            };
            if npc_talk(&mut talk, msg, table) == MSG_BOX_CLOSED {
                cut_end(run, list, flags);
            }
        }
        _ => cut_end(run, list, flags),
    }
}

fn os_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &Query<(Entity, &mut Actor)>,
    talks: &mut Query<&mut Talk>,
    msg: &mut MsgBox,
    table: &MsgTable,
    commands: &mut Commands,
) {
    let cut = run.cut as usize;
    match list.cuts.get(cut).map(|cut| cut.name.as_str()) {
        Some("WAIT") => {
            if run.advance > 0 {
                run.timer = list.cut_number(cut, "timer").unwrap_or(-1) as i16;
            }
            if run.timer < 0 {
                cut_end(run, list, flags);
            } else {
                wait_proc(run, list, flags);
            }
        }
        Some("WAKEUP") | Some("TURN") | Some("FINISH") => {}
        _ => open_msg_cut(
            run,
            list,
            flags,
            owner,
            actors,
            talks,
            msg,
            table,
            commands,
            ("MSGSET", "MSGEND"),
            "MsgNo",
            0,
        ),
    }
}

fn people_staff(name: &str) -> bool {
    matches!(
        name,
        "Uo1" | "Uo2"
            | "Uo3"
            | "Ub1"
            | "Ub2"
            | "Ub3"
            | "Ub4"
            | "Uw1"
            | "Uw2"
            | "Um1"
            | "Um2"
            | "Um3"
            | "Sa1"
            | "Sa2"
            | "Sa3"
            | "Sa4"
            | "Sa5"
            | "Ug1"
            | "Ug2"
            | "Figure"
    )
}

fn people_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &Query<(Entity, &mut Actor)>,
    talks: &mut Query<&mut Talk>,
    msg: &mut MsgBox,
    table: &MsgTable,
    commands: &mut Commands,
) {
    match list.cuts.get(run.cut as usize).map(|cut| cut.name.as_str()) {
        Some("MES_SET" | "MES_SET_TP" | "MES_SET_PO") => {
            talk_cut(run, list, flags, owner, actors, talks, msg, table, commands, false, "MsgNo");
        }
        Some(
            "MES_SET2" | "UB1_TALK" | "UB1_TALK_XY" | "UB2_TALK" | "UB_SET_ANM" | "TURN_TO_PLAYER",
        ) => {}
        _ => cut_end(run, list, flags),
    }
}

fn auction_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &Query<(Entity, &mut Actor)>,
    talks: &mut Query<&mut Talk>,
    msg: &mut MsgBox,
    table: &MsgTable,
    commands: &mut Commands,
) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    match cut.name.as_str() {
        "MES_SET" => talk_cut(run, list, flags, owner, actors, talks, msg, table, commands, false, "MsgNo"),
        "MES_END" => {
            let Ok(mut talk) = talks.get_mut(owner) else {
                cut_end(run, list, flags);
                return;
            };
            if npc_talk(&mut talk, msg, table) == MSG_BOX_CLOSED {
                cut_end(run, list, flags);
            }
        }
        "END" => {
            if run.advance > 0 {
                run.timer = 20;
            }
            if run.timer > 0 {
                run.timer -= 1;
            } else {
                cut_end(run, list, flags);
            }
        }
        "START" | "MAIN" | "GET_ITEM" | "GET_ITEM_NPC" | "GET_ITEM_MES" => {}
        _ => cut_end(run, list, flags),
    }
}

fn tpost_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &Query<(Entity, &mut Actor)>,
    talks: &mut Query<&mut Talk>,
    msg: &mut MsgBox,
    table: &MsgTable,
    commands: &mut Commands,
) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    if cut.name != "DISP_LETTER" {
        if cut.name != "SET_ANM" {
            cut_end(run, list, flags);
        }
        return;
    }
    let Ok(mut talk) = talks.get_mut(owner) else {
        let room = actors.get(owner).map_or(-1, |(_, actor)| actor.room);
        commands.entity(owner).insert(super::talk::Talk::new(0, room));
        return;
    };
    if run.advance > 0 {
        commands.entity(owner).insert(super::flags::LetterMsg);
        run.talk_open = true;
    }
    if run.talk_open
        && npc_talk(&mut talk, msg, table) == MSG_BOX_CLOSED
    {
        run.talk_open = false;
        cut_end(run, list, flags);
    }
}

fn bs1_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &Query<(Entity, &mut Actor)>,
    talks: &mut Query<&mut Talk>,
    msg: &mut MsgBox,
    table: &MsgTable,
    commands: &mut Commands,
) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    match cut.name.as_str() {
        "TALKMSG" => talk_cut(run, list, flags, owner, actors, talks, msg, table, commands, false, "MsgNo"),
        "CONTINUE_TALK" => talk_cut(run, list, flags, owner, actors, talks, msg, table, commands, true, "MsgNo"),
        "WAIT" => {
            if run.advance > 0 {
                wait_start(run, list);
            }
            wait_proc(run, list, flags);
        }
        _ => cut_end(run, list, flags),
    }
}

fn hsh_cut(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    if matches!(
        cut.name.as_str(),
        "MsgSet" | "Talk" | "Tact" | "Appear" | "Delete"
    ) {
        return;
    }
    cut_end(run, list, flags);
}

fn warpout_cut(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    if matches!(cut.name.as_str(), "WARP1" | "WARP2" | "WARP3") {
        if run.advance > 0 {
            run.timer = 5;
        }
        wait_proc(run, list, flags);
        return;
    }
    cut_end(run, list, flags);
}

fn hold_cut(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags, hold: &[&str]) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    if hold.contains(&cut.name.as_str()) {
        return;
    }
    cut_end(run, list, flags);
}

fn ji1_cut(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    if cut.name == "02_dummy" {
        return;
    }
    cut_end(run, list, flags);
}

fn zl1_cut(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    if matches!(cut.name.as_str(), "OKIRU" | "OKIRU_2" | "JMP_OFF") {
        return;
    }
    cut_end(run, list, flags);
}

fn tc_cut(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    match cut.name.as_str() {
        "SIT_TO_JUMP" | "PRESENT" | "BACKJUMP" => {}
        "SET_ANM" => {
            if list.cut_text(run.cut as usize, "Name") == Some("GET") {
                cut_end(run, list, flags);
            }
        }
        _ => cut_end(run, list, flags),
    }
}

fn mk_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &Query<(Entity, &mut Actor)>,
    talks: &mut Query<&mut Talk>,
    msg: &mut MsgBox,
    table: &MsgTable,
    commands: &mut Commands,
) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    match cut.name.as_str() {
        "SPEAK" => talk_cut(run, list, flags, owner, actors, talks, msg, table, commands, false, "MsgNo"),
        "RUN" | "RUN3" | "JUMP" | "JUMP2" | "JITABATA" | "TURN" => {}
        _ => cut_end(run, list, flags),
    }
}

fn roten_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &Query<(Entity, &mut Actor)>,
    talks: &mut Query<&mut Talk>,
    msg: &mut MsgBox,
    table: &MsgTable,
    commands: &mut Commands,
) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    match cut.name.as_str() {
        "MES_SET" => talk_cut(run, list, flags, owner, actors, talks, msg, table, commands, false, "MsgNo"),
        "SET_ITEM" => {}
        _ => cut_end(run, list, flags),
    }
}

fn mbdoor_cut(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    match cut.name.as_str() {
        "END" => return,
        "OPEN" => {
            run.timer = run.timer.saturating_add(50);
            if run.timer >= 7281 {
                run.timer = 0;
                cut_end(run, list, flags);
            }
        }
        "STOP_OPEN" => {
            run.timer = run.timer.saturating_add(40);
            if run.timer >= 16229 {
                run.timer = 0;
                cut_end(run, list, flags);
            }
        }
        "ADJUSTMENT" => {
            if run.advance > 0 {
                run.timer = list.cut_number(run.cut as usize, "Timer").unwrap_or(0) as i16;
            }
            if run.timer > 0 {
                run.timer -= 1;
            } else {
                cut_end(run, list, flags);
            }
        }
        "WAIT" => cut_end(run, list, flags),
        _ => {}
    }
}

fn statue_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &Query<(Entity, &mut Actor)>,
    talks: &mut Query<&mut Talk>,
    msg: &mut MsgBox,
    table: &MsgTable,
    commands: &mut Commands,
) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    match cut.name.as_str() {
        "INPUT" | "LESSON" => talk_cut(run, list, flags, owner, actors, talks, msg, table, commands, false, "MsgNo"),
        "BREAK" | "TACT" => {}
        _ => cut_end(run, list, flags),
    }
}

fn torch_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &mut Query<(Entity, &mut Actor)>,
    commands: &mut Commands,
) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    if cut.name == "FIRE" {
        let targets = staff_targets(list, run.staff, actors, owner, |name| {
            name == super::torch::TORCH_ACTOR
        });
        for target in targets {
            commands.entity(target).insert(FireLit(true));
        }
        return;
    }
    cut_end(run, list, flags);
}

fn bm1_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &mut Query<(Entity, &mut Actor)>,
) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    if cut.name == "ACTION" {
        let act = list.cut_number(run.cut as usize, "ActNo").unwrap_or(-1);
        if matches!(act, 0x16 | 0x17 | 0x19 | 0x1B) {
            return;
        }
        cut_end(run, list, flags);
        return;
    }
    if cut.name == "360_TRN" {
        if let Ok((_, mut body)) = actors.get_mut(owner) {
            body.angle[1] = body.angle[1].wrapping_add(0x8000);
        }
    }
    cut_end(run, list, flags);
}

fn named_cut(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags, name: &str) {
    if let Some(cut) = list.cuts.get(run.cut as usize)
        && cut.name == name
    {
        cut_end(run, list, flags);
    }
}

fn switch_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &mut Query<(Entity, &mut Actor)>,
    commands: &mut Commands,
) {
    if run.advance > 0
        && list
            .cuts
            .get(run.cut as usize)
            .is_some_and(|cut| cut.name == "CHANGE")
        && let Ok((_, actor)) = actors.get(owner)
    {
        let bit = (actor.parameters & 0xFF) as u8;
        if bit != 0xFF {
            commands.entity(owner).insert(super::flags::SwitchOn(bit));
        }
    }
    cut_end(run, list, flags);
}

fn hint_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &Query<(Entity, &mut Actor)>,
    talks: &mut Query<&mut Talk>,
    msg: &mut MsgBox,
    table: &MsgTable,
    commands: &mut Commands,
) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    if cut.name == "MESSAGE3" {
        let Ok(mut talk) = talks.get_mut(owner) else {
            host_message(owner, actors, commands);
            return;
        };
        if run.advance > 0
            && let Ok((_, actor)) = actors.get(owner)
        {
            talk.msg_curr = actor.angle[0] as u16 as u32;
            talk.msg_end = -1;
            run.talk_open = true;
        }
        if run.talk_open {
            npc_talk(&mut talk, msg, table);
        }
        return;
    }
    if !matches!(cut.name.as_str(), "MESSAGE" | "MESSAGE2") {
        cut_end(run, list, flags);
        return;
    }
    let Ok(mut talk) = talks.get_mut(owner) else {
        host_message(owner, actors, commands);
        return;
    };
    if run.advance > 0
        && let Ok((_, actor)) = actors.get(owner)
    {
        talk.msg_curr = actor.angle[0] as u16 as u32;
        talk.msg_end = -1;
        run.talk_open = true;
    }
    if !run.talk_open {
        cut_end(run, list, flags);
        return;
    }
    if npc_talk(&mut talk, msg, table) == MSG_BOX_CLOSED {
        run.talk_open = false;
        cut_end(run, list, flags);
    }
}

fn fire_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &mut Query<(Entity, &mut Actor)>,
    commands: &mut Commands,
) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    let lit = match cut.name.as_str() {
        "ON" => true,
        "OFF" => false,
        _ => {
            cut_end(run, list, flags);
            return;
        }
    };
    let targets = staff_targets(list, run.staff, actors, owner, |name| {
        name == super::fire::FIRE_ACTOR
    });
    for target in targets {
        commands.entity(target).insert(FireLit(lit));
    }
    cut_end(run, list, flags);
}

fn staff_targets(
    list: &EventList,
    staff: usize,
    actors: &Query<(Entity, &mut Actor)>,
    origin: Entity,
    wanted: fn(&str) -> bool,
) -> Vec<Entity> {
    let Some(record) = list.staffs.get(staff) else {
        return Vec::new();
    };
    let name = record.name.clone();
    let rank = list
        .staffs
        .iter()
        .take(staff)
        .filter(|other| other.name == name)
        .count();
    let from = actors.get(origin).ok().map(|(_, actor)| actor.pos);
    let mut found: Vec<(f32, Entity)> = actors
        .iter()
        .filter(|(_, actor)| wanted(&actor.name))
        .map(|(entity, actor)| {
            let near = from.map_or(f32::MAX, |pos| pos.distance_squared(actor.pos));
            (near, entity)
        })
        .collect();
    found.sort_by(|left, right| left.0.total_cmp(&right.0));
    found
        .into_iter()
        .skip(rank)
        .map(|(_, entity)| entity)
        .take(1)
        .collect()
}

fn door_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &mut Query<(Entity, &mut Actor)>,
    commands: &mut Commands,
) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    let opening = match cut.name.as_str() {
        "OPEN" | "STOP_OPEN" => true,
        "CLOSE" | "STOP_CLOSE" => false,
        "UNLOCK" => {
            cut_end(run, list, flags);
            return;
        }
        _ => {
            cut_end(run, list, flags);
            return;
        }
    };
    if run.advance > 0 {
        let targets = staff_targets(list, run.staff, actors, owner, |name| {
            super::door::Door::is_shutter(name)
                || super::door::Door::is_sliding(name)
                || super::door::Door::is_key_door(name)
        });
        if targets.is_empty() {
            cut_end(run, list, flags);
        } else {
            for target in targets {
                commands.entity(target).insert(DoorCut {
                    flag: cut.flag_id as u32,
                    opening,
                    armed: true,
                });
            }
        }
    }
}

fn treasure_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &mut Query<(Entity, &mut Actor)>,
    commands: &mut Commands,
) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    if matches!(cut.name.as_str(), "OPEN" | "OPEN_SHORT") {
        let targets = staff_targets(list, run.staff, actors, owner, |name| {
            super::chest::CHEST_ACTORS.contains(&name)
        });
        for target in targets {
            commands.entity(target).insert(ChestOpen);
        }
    }
    cut_end(run, list, flags);
}

fn soldier_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &mut Query<(Entity, &mut Actor)>,
    players: &mut Query<(Entity, &mut Player)>,
    sea: Option<&Sea>,
    talks: &mut Query<&mut Talk>,
    msg: &mut MsgBox,
    table: &MsgTable,
    commands: &mut Commands,
) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    let step = run.offset_scale * (run.offset_target - run.model_offset);
    run.model_offset += step.clamp(-run.offset_step, run.offset_step);
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
        "MOVE_TO_ACTOR" | "MOVE_TO_POS" => move_cut(run, list, flags, owner, actors, players),
        "TURN_TO_ACTOR" | "TURN_TO_POS" => turn_cut(
            run,
            list,
            flags,
            owner,
            actors,
            players,
            cut.name == "TURN_TO_ACTOR",
        ),
        "SWIM" => swim_cut(run, list, flags, owner, actors),
        "JUMP" => jump_cut(run, list, flags, owner, actors, sea),
        "APPEAR" | "DIVE" | "DISAPPEAR" => offset_cut(run, list, flags, &cut.name),
        "JUMP_MAPOPEN" => mapopen_cut(run, list, flags, owner, actors),
        "TURN" => soldier_turn(run, list, flags, owner, actors),
        "TALK_MSG" => talk_cut(run, list, flags, owner, actors, talks, msg, table, commands, false, "msg_num"),
        "CONTINUE_TALK" => talk_cut(run, list, flags, owner, actors, talks, msg, table, commands, true, "msg_num"),
        name if SOLDIER_INSTANT.contains(&name) => cut_end(run, list, flags),
        name if SOLDIER_PENDING.contains(&name) => {}
        _ => cut_end(run, list, flags),
    }
}

fn anim_ticks(stem: &str, rate: f32, start: f32, end: f32, anms: &LinkAnms) -> Option<i32> {
    let target = if end < 0.0 { anms.end_frame(stem)? } else { end };
    if rate <= 0.0 {
        return None;
    }
    let mut frame = start;
    let mut ticks = 0;
    while frame < target {
        frame += rate;
        ticks += 1;
        if ticks > 4000 {
            return None;
        }
    }
    Some(ticks.max(1))
}

fn link_anim_ticks(
    mode: i32,
    prm0: i32,
    turn: Option<i32>,
    yaw: i16,
    partner: &ItemPartner,
    anms: &LinkAnms,
) -> Option<i32> {
    match mode {
        5 => {
            let target = turn.map_or(partner.move_angle, |angle| angle as i16);
            let delta = target.wrapping_sub(yaw);
            Some(((i32::from(delta.unsigned_abs()) + 7999) / 8000).max(1))
        }
        7 => anim_ticks("damf", 0.6, 0.0, 9.0, anms),
        9 | 27 => anim_ticks("damffup", 0.5, 0.0, -1.0, anms),
        10 => anim_ticks(
            if prm0 == 1 {
                "boxopenshortlink"
            } else {
                "boxopenlink"
            },
            1.0,
            0.0,
            -1.0,
            anms,
        ),
        12 => Some(1),
        13 => anim_ticks("holdup", 1.1, 0.0, 11.0, anms),
        15 => anim_ticks("comeout", 1.0, 0.0, 72.0, anms),
        19 => anim_ticks("saltation", 1.0, 0.0, -1.0, anms),
        21 => anim_ticks("pickup", 1.5, 2.0, 89.0, anms),
        24 => anim_ticks("surprised", 1.0, 0.0, -1.0, anms),
        25 => anim_ticks("turnback", 1.0, 0.0, 44.0, anms),
        26 => anim_ticks("lookup", 1.0, 0.0, -1.0, anms),
        34 => match prm0 {
            -4 => Some(1),
            6 | 7 => Some(600),
            1 | 5 => Some(900),
            _ => None,
        },
        35 => anim_ticks(
            if prm0 & 1 != 0 {
                "dooropenblink"
            } else {
                "dooropenalink"
            },
            1.0,
            if prm0 & 2 != 0 { 35.0 } else { 0.0 },
            -1.0,
            anms,
        ),
        36 => anim_ticks("seyyes", 1.0, 0.0, 28.0, anms),
        37 => anim_ticks("presentationa", 1.1, 0.0, 62.0, anms),
        38 => anim_ticks("windl", 1.0, 0.0, -1.0, anms),
        40 => Some(anim_ticks("grabup", 1.1, 0.0, 7.0, anms)? + 1),
        46 => anim_ticks("powup", 1.0, 0.0, -1.0, anms),
        48 => {
            if prm0 == 0 {
                Some(
                    anim_ticks("warpoutfirst", 1.0, 0.0, -1.0, anms)?
                        + anim_ticks("warpout", 1.0, 0.0, -1.0, anms)?,
                )
            } else {
                anim_ticks("warpin", 1.0, 0.0, -1.0, anms)
            }
        }
        49 => anim_ticks("surprisedb", 1.0, 0.0, -1.0, anms),
        50 => Some(6),
        53 => Some(anim_ticks("windl", 1.0, 0.0, -1.0, anms)? + 5),
        54 => anim_ticks("getletter", 1.0, 0.0, -1.0, anms),
        56 => anim_ticks("grabup", 1.1, 0.0, 7.0, anms),
        59 => anim_ticks("taktdge", 1.0, 0.0, -1.0, anms),
        60 => Some(
            anim_ticks("binopenpre", 1.0, 0.0, 45.0, anms)?
                + anim_ticks("binopenb", 1.0, 0.0, 124.0, anms)?,
        ),
        61 => {
            let second = if partner.item as i32 == FAIRY_BOTTLE_ITEM {
                anim_ticks("binopenb", 1.0, 0.0, 124.0, anms)?
            } else {
                anim_ticks("binopena", 1.0, 0.0, 69.0, anms)?
            };
            Some(anim_ticks("binopenpre", 1.0, 0.0, 45.0, anms)? + second)
        }
        63 => anim_ticks("boxopenslink", 1.0, 0.0, -1.0, anms),
        65 => anim_ticks("sethyoinomi", 1.1, 2.0, 21.0, anms),
        70 => anim_ticks("combo_link", 1.0, 0.0, -1.0, anms),
        73 => anim_ticks("esamaki", 0.6, 0.0, 17.0, anms),
        74 => anim_ticks("slipice", 1.0, 0.0, -1.0, anms),
        _ => None,
    }
}

fn item_message(item: u8, max_life: u8) -> u32 {
    let value = if item == HEART_PIECE_ITEM && max_life % 4 != 0 {
        HEART_PIECE_ALT + (max_life % 4) - 1
    } else {
        item
    };
    u32::from(value) + 101
}

fn link_anim_msg(mode: i32, prm0: i32, partner: &ItemPartner) -> Option<u32> {
    match mode {
        46 => {
            let item = if prm0 == 0x100 {
                partner.item as i32
            } else {
                prm0
            };
            Some((item + 101).max(0) as u32)
        }
        59 => Some((prm0 + 0xD2).max(0) as u32),
        _ => None,
    }
}

fn link_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    actors: &mut Query<(Entity, &mut Actor)>,
    ships: &mut Query<&mut Ship>,
    msg: &mut MsgBox,
    table: &MsgTable,
    partner: &mut ItemPartner,
    anms: &LinkAnms,
    players: &mut Query<(Entity, &mut Player)>,
    commands: &mut Commands,
    presents: &mut Query<&mut PresentItem>,
) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    let mode = leading_digits(&cut.name, 3);
    if mode == Some(GET_ITEM_MODE) {
        get_item_cut(run, list, flags, msg, table, partner, anms, players, commands, presents);
        return;
    }
    if run.advance > 0 {
        run.get_stage = 0;
    }
    let turn = list.cut_number(run.cut as usize, "angle");
    let yaw = players
        .iter()
        .next()
        .map_or(0, |(_, player)| to_s16(player.yaw));
    if let Some(mode) = mode {
        match mode {
            4 | 44 => {
                let angle = turn.map_or(yaw, |angle| angle as i16);
                partner.move_angle = angle;
                let mut placed: Option<Vec3> = None;
                if let Some((_, mut player)) = players.iter_mut().next() {
                    if let Some(pos) = list.cut_vec(run.cut as usize, "pos") {
                        player.pos = pos;
                        placed = Some(pos);
                    }
                    let heading = from_s16(angle);
                    player.yaw = heading;
                    player.face = heading;
                    player.speed = 0.0;
                }
                if let Some(pos) = placed
                    && ships.iter().any(|ship| ship.riding)
                    && let Some((_, mut actor)) =
                        actors.iter_mut().find(|(_, actor)| actor.name == SHIP_ACTOR)
                {
                    actor.pos = pos;
                    actor.angle[1] = angle;
                }
            }
            43 => {
                if let Some(pos) = list.cut_vec(run.cut as usize, "pos")
                    && let Some((_, mut player)) = players.iter_mut().next()
                {
                    let heading = target_angle_y(player.pos, pos);
                    player.yaw = from_s16(heading);
                    player.face = from_s16(heading);
                }
            }
            2 | 3 => {
                partner.demo_stick = Some(0.0);
                if let Some(pos) = list.cut_vec(run.cut as usize, "pos")
                    && let Some((_, mut player)) = players.iter_mut().next()
                {
                    partner.move_angle = target_angle_y(player.pos, pos);
                    let delta = pos - player.pos;
                    let flat = delta.x * delta.x + delta.z * delta.z;
                    if flat < 100.0 || (flat < 2500.0 && player.speed.abs() < 0.001) {
                        player.speed = 0.0;
                        cut_end(run, list, flags);
                    } else if flat >= 2500.0 {
                        partner.demo_stick =
                            Some(list.cut_float(run.cut as usize, "stick").unwrap_or(1.0));
                    }
                }
            }
            5 => {
                if let Some(angle) = turn {
                    partner.move_angle = angle as i16;
                }
            }
            16 => {
                partner.demo_backjump = true;
                if run.advance > 0 {
                    partner.demo_landed = false;
                    if !ships.iter().any(|ship| ship.riding)
                        && let Some((_, mut player)) = players.iter_mut().next()
                    {
                        partner.move_angle = to_s16(player.face + std::f32::consts::PI);
                        partner.demo_launch = true;
                    }
                }
                if partner.demo_landed {
                    cut_end(run, list, flags);
                }
            }
            64 => {
                partner.demo_fall = true;
                if run.advance > 0 {
                    partner.fall_cap = -20.0;
                    partner.demo_landed = false;
                    if let Some((_, mut player)) = players.iter_mut().next() {
                        player.speed = 0.0;
                    }
                }
                if partner.demo_landed {
                    cut_end(run, list, flags);
                }
            }
            67 => {
                partner.demo_stick = Some(0.0);
                if run.advance > 0 {
                    if let Some((_, mut player)) = players.iter_mut().next() {
                        player.speed = 0.0;
                    }
                }
            }
            71 => {
                if run.advance > 0
                    && let Some((ship_id, _, _)) = ship_pos(actors)
                    && let Ok((_, mut body)) = actors.get_mut(ship_id)
                    && let Some((_, mut player)) = players.iter_mut().next()
                {
                    let yaw = from_s16(body.angle[1].wrapping_sub(0x4000));
                    player.pos.x += yaw.sin() * LEDGE_OFFSET_X;
                    player.pos.y = body.pos.y + DECK_RISE;
                    player.pos.z += yaw.cos() * LEDGE_OFFSET_X;
                    player.yaw = yaw;
                    player.speed = GET_OFF_SPEED;
                    player.speed_y = 0.0;
                    if let Ok(mut sail) = ships.get_mut(ship_id) {
                        sail.riding = false;
                    }
                    cut_end(run, list, flags);
                }
            }
            _ => {}
        }
    }
    if mode.is_some_and(|mode| LINK_INSTANT.contains(&mode)) {
        cut_end(run, list, flags);
        return;
    }
    let Some(mode) = mode else {
        return;
    };
    let prm0 = list.cut_number(run.cut as usize, "prm0").unwrap_or(0);
    let Some(total) = link_anim_ticks(mode, prm0, turn, yaw, partner, anms) else {
        return;
    };
    if run.advance > 0 {
        run.timer = total as i16;
        run.get_msg_live = false;
        run.phase = 0;
    }
    if run.timer > 0 {
        run.timer -= 1;
        return;
    }
    let Some(msg_no) = link_anim_msg(mode, prm0, partner) else {
        cut_end(run, list, flags);
        return;
    };
    if run.phase == 0 {
        if !run.get_msg_live {
            if msg.active() && msg.message_no() != msg_no {
                return;
            }
            run.get_msg_live = true;
            msg.message_set(msg_no, table);
            return;
        }
        if msg.active() && msg.message_no() != msg_no {
            return;
        }
        match msg.status() {
            MSG_DISPLAYED => msg.set_status(MSG_ENDS),
            MSG_BOX_CLOSED => {
                msg.set_status(MSG_DESTROYED);
                run.get_msg_live = false;
                run.phase = 1;
                cut_end(run, list, flags);
            }
            _ => {}
        }
        return;
    }
    cut_end(run, list, flags);
}

fn host_message(
    owner: Entity,
    actors: &Query<(Entity, &mut Actor)>,
    commands: &mut Commands,
) {
    let room = actors.get(owner).map_or(-1, |(_, actor)| actor.room);
    commands
        .entity(owner)
        .insert(super::talk::Talk::new(0, room));
}

fn talk_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &Query<(Entity, &mut Actor)>,
    talks: &mut Query<&mut Talk>,
    msg: &mut MsgBox,
    table: &MsgTable,
    commands: &mut Commands,
    cont: bool,
    msg_key: &str,
) {
    let Ok(mut talk) = talks.get_mut(owner) else {
        host_message(owner, actors, commands);
        return;
    };
    if run.advance > 0 {
        if cont {
            talk.msg_end = list.cut_number(run.cut as usize, "end_num").unwrap_or(-1);
        } else {
            talk.msg_curr = list.cut_number(run.cut as usize, msg_key).unwrap_or(0) as u32;
            talk.msg_end = list.cut_number(run.cut as usize, "end_msg").unwrap_or(-1);
            run.talk_open = true;
        }
    }
    if !run.talk_open {
        cut_end(run, list, flags);
        return;
    }
    let status = npc_talk(&mut talk, msg, table);
    if status == MSG_BOX_CLOSED {
        run.talk_open = false;
        cut_end(run, list, flags);
    } else if (status == MSG_BOX_OPENING || status == MSG_TYPING)
        && talk.msg_end as u32 == talk.msg_curr
    {
        talk.msg_end = -1;
        cut_end(run, list, flags);
    }
}

fn npc_talk(talk: &mut Talk, msg: &mut MsgBox, table: &MsgTable) -> u8 {
    let mut status = 0xFF;
    if !talk.msg_live {
        talk.msg_live = true;
        talk.msg_bound = false;
        msg.message_set(talk.msg_curr, table);
    } else if talk.msg_bound {
        status = msg.status();
        if status == MSG_DISPLAYED {
            msg.set_status(MSG_ENDS);
        } else if status == MSG_BOX_CLOSED {
            msg.set_status(MSG_DESTROYED);
            talk.msg_live = false;
        }
    } else {
        talk.msg_bound = true;
    }
    status
}

fn get_item_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    msg: &mut MsgBox,
    table: &MsgTable,
    partner: &mut ItemPartner,
    anms: &LinkAnms,
    players: &mut Query<(Entity, &mut Player)>,
    commands: &mut Commands,
    presents: &mut Query<&mut PresentItem>,
) {
    if run.get_stage & GET_DONE != 0 {
        cut_end(run, list, flags);
        return;
    }
    if run.get_stage & GET_ENTERED == 0 {
        run.get_stage |= GET_ENTERED;
        run.get_msg_live = false;
        run.timer = anim_ticks("itemget", 1.0, 0.0, -1.0, anms).unwrap_or(0) as i16;
        let prm0 = list.cut_number(run.cut as usize, "prm0").unwrap_or(0);
        let item = match prm0 {
            0 => None,
            0x100 => Some(partner.item),
            value => Some(value as u8),
        };
        if let Some(item) = item {
            if let Some(old) = partner.entity.take() {
                let _ = commands.entity(old).despawn();
            }
            let pos = players
                .iter()
                .next()
                .map_or(Vec3::ZERO, |(_, player)| player.pos);
            let entity = commands
                .spawn((
                    Actor {
                        name: PRESENT_ACTOR.to_owned(),
                        parameters: 0,
                        pos,
                        angle: [0; 3],
                        set_id: u16::MAX,
                        room: -1,
                    },
                    PresentItem {
                        item,
                        hidden: true,
                    },
                ))
                .id();
            partner.entity = Some(entity);
            partner.item = item;
        }
        run.msg_no = item_message(partner.item, partner.max_life) as i32;
    }
    if run.timer > 0 {
        run.timer -= 1;
        return;
    }
    if let Some(entity) = partner.entity
        && let Ok(mut present) = presents.get_mut(entity)
    {
        present.hidden = false;
    }
    if partner.entity.is_none() {
        return;
    }
    let msg_no = run.msg_no as u32;
    if !run.get_msg_live {
        if msg.active() && msg.message_no() != msg_no {
            return;
        }
        run.get_msg_live = true;
        msg.message_set(msg_no, table);
        return;
    }
    if msg.active() && msg.message_no() != msg_no {
        return;
    }
    match msg.status() {
        MSG_DISPLAYED => msg.set_status(MSG_ENDS),
        MSG_BOX_CLOSED => {
            msg.set_status(MSG_DESTROYED);
            run.get_msg_live = false;
            run.get_stage |= GET_DONE;
            if let Some(entity) = partner.entity.take() {
                let _ = commands.entity(entity).despawn();
            }
            cut_end(run, list, flags);
        }
        _ => {}
    }
}

fn cut_angle(list: &EventList, cut: usize) -> Option<i16> {
    list.cut_number(cut, "angle")
        .map(|angle| angle as i16)
        .or_else(|| list.cut_float(cut, "angle").map(to_s16))
}

fn ship_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    actors: &mut Query<(Entity, &mut Actor)>,
    ships: &mut Query<&mut Ship>,
) {
    let Some(cut) = list.cuts.get(run.cut as usize) else {
        return;
    };
    match leading_digits(&cut.name, 2) {
        Some(0) | Some(8) => {
            if let Some((ship_id, _, _)) = ship_pos(actors)
                && let Ok((_, mut body)) = actors.get_mut(ship_id)
            {
                if let Some(pos) = list.cut_vec(run.cut as usize, "pos") {
                    body.pos = pos;
                }
                if let Some(angle) = cut_angle(list, run.cut as usize) {
                    body.angle[1] = angle;
                }
                if let Ok(mut sail) = ships.get_mut(ship_id) {
                    if let Some(speed) = list.cut_float(run.cut as usize, "speed") {
                        sail.speed = speed;
                    }
                    sail.tiller = 0;
                }
            }
            cut_end(run, list, flags);
        }
        Some(5) => {
            if let Some((ship_id, _, _)) = ship_pos(actors)
                && let Ok(mut sail) = ships.get_mut(ship_id)
            {
                if let Some(speed) = list.cut_float(run.cut as usize, "speed") {
                    add_calc(&mut sail.speed, speed, 0.1, 5.0, 1.0);
                }
                sail.tiller = 0;
            }
            cut_end(run, list, flags);
        }
        Some(4) => {
            if let Some((ship_id, _, _)) = ship_pos(actors)
                && let Ok(mut sail) = ships.get_mut(ship_id)
            {
                sail.speed = 0.0;
            }
            cut_end(run, list, flags);
        }
        Some(6) => cut_end(run, list, flags),
        Some(7) => {
            if let Some((ship_id, _, _)) = ship_pos(actors)
                && let Ok((_, mut body)) = actors.get_mut(ship_id)
                && let Some(angle) = cut_angle(list, run.cut as usize)
            {
                body.angle[1] = angle;
            }
            cut_end(run, list, flags);
        }
        Some(10) => {
            if let Some((ship_id, _, _)) = ship_pos(actors)
                && let Ok((_, body)) = actors.get(ship_id)
                && let Ok(mut sail) = ships.get_mut(ship_id)
            {
                sail.speed = 0.0;
                sail.tiller = 0;
                sail.warp_y = body.pos.y;
            }
            cut_end(run, list, flags);
        }
        Some(11) => {
            if let Some((ship_id, _, _)) = ship_pos(actors)
                && let Ok((_, body)) = actors.get(ship_id)
                && let Ok(sail) = ships.get(ship_id)
                && body.pos.y <= sail.warp_y
            {
                cut_end(run, list, flags);
            }
        }
        Some(1) => ship_move(run, list, flags, actors, ships),
        _ => {}
    }
}

fn ship_move(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    actors: &mut Query<(Entity, &mut Actor)>,
    ships: &mut Query<&mut Ship>,
) {
    let Some(target) = list.cut_vec(run.cut as usize, "pos") else {
        return;
    };
    let Some((ship_id, _, _)) = ship_pos(actors) else {
        return;
    };
    let Ok((_, mut body)) = actors.get_mut(ship_id) else {
        return;
    };
    let Ok(mut sail) = ships.get_mut(ship_id) else {
        return;
    };
    let dist = abs_xz(target - body.pos);
    if dist < 10.0 {
        sail.speed = 0.0;
        cut_end(run, list, flags);
    } else if let Some(speed) = list.cut_float(run.cut as usize, "speed") {
        if speed <= -0.5 {
            if speed < -1.5 {
                add_calc(&mut sail.speed, 0.0, 0.1, 1.0, 0.1);
            } else {
                add_calc(&mut sail.speed, 0.0, 0.05, 0.1, 0.015);
            }
        } else {
            sail.speed = (dist * 0.2).min(speed);
        }
    } else {
        sail.speed = dist * 0.2;
    }
    if dist > 1.0 {
        let angle = target_angle_y(body.pos, target);
        body.angle[1] = add_calc_angle_s(body.angle[1], angle, 3, 0x2000, 0x80);
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

fn move_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &mut Query<(Entity, &mut Actor)>,
    players: &mut Query<(Entity, &mut Player)>,
) {
    let cut = run.cut as usize;
    let is_actor = list
        .cuts
        .get(cut)
        .is_some_and(|cut| cut.name == "MOVE_TO_ACTOR");
    let (target, yaw) = if is_actor {
        let Some(found) = resolve_target(run, list, cut, owner, actors, players) else {
            cut_end(run, list, flags);
            return;
        };
        match read_pos(found, actors, players) {
            Some(value) => value,
            None => {
                cut_end(run, list, flags);
                return;
            }
        }
    } else {
        (list.cut_vec(cut, "Pos").unwrap_or(Vec3::ZERO), 0)
    };
    let Ok((_, mut me)) = actors.get_mut(owner) else {
        return;
    };
    let offset = if is_actor {
        list.cut_vec(cut, "OffsetPos").unwrap_or(Vec3::ZERO)
    } else {
        Vec3::ZERO
    };
    let del = list.cut_float(cut, "DelDistance").unwrap_or(0.0);
    let speed = list.cut_float(cut, "Speed").unwrap_or(2.0);
    let mut stop = target + offset;
    let angle = if is_actor {
        let add = list.cut_number(cut, "ShapeAngle").unwrap_or(0);
        if add != 0 {
            (yaw as i32 + add as i32) as i16
        } else {
            target_angle_y(me.pos, target)
        }
    } else {
        target_angle_y(me.pos, target)
    };
    stop.x -= del * from_s16(angle).sin();
    stop.z -= del * from_s16(angle).cos();
    if speed == 0.0 {
        me.pos = stop;
        cut_end(run, list, flags);
        return;
    }
    let dist = abs_xz(stop - me.pos);
    add_calc(&mut me.pos.x, stop.x, 0.1, speed, 0.0);
    add_calc(&mut me.pos.z, stop.z, 0.1, speed, 0.0);
    if dist < 5.0 {
        cut_end(run, list, flags);
    }
}

fn turn_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &mut Query<(Entity, &mut Actor)>,
    players: &mut Query<(Entity, &mut Player)>,
    is_actor: bool,
) {
    let cut = run.cut as usize;
    if run.advance > 0 {
        run.timer = list.cut_number(cut, "Timer").unwrap_or(1) as i16;
        run.prev_delta = 0;
        run.aim_vel = 0;
    }
    let aim = if is_actor {
        let Some(found) = resolve_target(run, list, cut, owner, actors, players) else {
            cut_end(run, list, flags);
            return;
        };
        let Some((pos, yaw)) = read_pos(found, actors, players) else {
            cut_end(run, list, flags);
            return;
        };
        let offset = list.cut_vec(cut, "OffsetPos").unwrap_or(Vec3::ZERO);
        let name = list.cut_text(cut, "ActorName");
        if matches!(name, None | Some("Player")) {
            pos + offset
        } else {
            let add = list.cut_number(cut, "AddAngle").unwrap_or(0);
            let del = list.cut_float(cut, "DelDistance").unwrap_or(0.0);
            if add != 0 && del != 0.0 {
                let angle = (yaw as i32 + add as i32) as i16;
                Vec3::new(
                    pos.x - del * from_s16(angle).sin(),
                    pos.y,
                    pos.z - del * from_s16(angle).cos(),
                )
            } else {
                pos + offset
            }
        }
    } else {
        list.cut_vec(cut, "Pos").unwrap_or(Vec3::ZERO)
    };
    run.attn = aim;
    let Ok((_, mut me)) = actors.get_mut(owner) else {
        return;
    };
    let turn_speed = list.cut_number(cut, "TurnSpeed").unwrap_or(0);
    let ramp = if turn_speed != 0 {
        turn_speed as i16
    } else {
        0x150
    };
    run.aim_vel = add_calc_angle_s(run.aim_vel, ramp, 4, 0x800, -1);
    let angle = target_angle_y(me.pos, aim);
    me.angle[1] = turn_body(me.angle[1], angle, run.aim_vel);
    let delta = angle as i32 - me.angle[1] as i32;
    let temp = delta.unsigned_abs() as f32;
    if (run.prev_delta as f32) == temp && temp < 5376.0 && run.timer > 0 {
        run.timer -= 1;
    }
    run.prev_delta = temp as i16;
    if run.timer == 0 {
        cut_end(run, list, flags);
    }
}

fn swim_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &mut Query<(Entity, &mut Actor)>,
) {
    let Ok((_, mut me)) = actors.get_mut(owner) else {
        return;
    };
    let dist = abs_xz(run.attn - me.pos);
    let angle = target_angle_y(me.pos, run.attn);
    me.angle[1] = add_calc_angle_s(me.angle[1], angle, 8, 0x400, -1);
    add_calc(&mut run.speed, 7.0, 0.3, 4.0, 0.0);
    let yaw = from_s16(me.angle[1]);
    me.pos.x += yaw.sin() * run.speed;
    me.pos.z += yaw.cos() * run.speed;
    if dist < 150.0 {
        run.speed = 0.0;
        cut_end(run, list, flags);
    }
}

fn jump_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &mut Query<(Entity, &mut Actor)>,
    sea: Option<&Sea>,
) {
    let Ok((_, mut me)) = actors.get_mut(owner) else {
        return;
    };
    if run.advance > 0 {
        let (_, surface) = water::probe(sea, me.pos.x, me.pos.z);
        me.pos.y = surface;
        me.angle[0] = -8000;
        run.timer = 15;
        run.offset_target = -40.0;
        run.offset_step = 10.0;
        run.offset_scale = 0.3;
    }
    let (_, surface) = water::probe(sea, me.pos.x, me.pos.z);
    if run.timer > 0 {
        run.timer -= 1;
    }
    if run.timer == 0 && me.pos.y <= surface {
        cut_end(run, list, flags);
    }
}

fn soldier_turn(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &mut Query<(Entity, &mut Actor)>,
) {
    if run.advance > 0 {
        run.timer = 30;
    }
    let Some((_, ship, yaw)) = ship_pos(actors) else {
        cut_end(run, list, flags);
        return;
    };
    let Ok((_, mut me)) = actors.get_mut(owner) else {
        return;
    };
    let index = yaw.wrapping_add(0x4000);
    let mut target = ship;
    target.x += from_s16(index).sin() * -300.0;
    target.z += from_s16(index).cos() * -300.0;
    let angle = target_angle_y(me.pos, target);
    me.angle[1] = add_calc_angle_s(me.angle[1], angle, 8, 0x400, -1);
    let gap = me.angle[1].wrapping_sub(angle).unsigned_abs();
    if run.timer > 0 {
        run.timer -= 1;
    }
    if run.timer == 0 || gap <= 0x400 {
        cut_end(run, list, flags);
    }
}

fn mapopen_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &mut Query<(Entity, &mut Actor)>,
) {
    if run.advance > 0 {
        run.phase = 0;
        run.offset_target = -1.0;
        run.offset_step = 20.0;
        run.offset_scale = 0.1;
    }
    let Some((_, ship, yaw)) = ship_pos(actors) else {
        cut_end(run, list, flags);
        return;
    };
    let Ok((_, mut me)) = actors.get_mut(owner) else {
        return;
    };
    let index = yaw.wrapping_add(0x4000);
    match run.phase {
        0 => {
            run.model_offset = run.offset_target;
            run.phase = 1;
        }
        1 => {
            let mut goal = ship;
            goal.x += from_s16(index).sin() * -120.0;
            goal.z += from_s16(index).cos() * -120.0;
            if chase_xz(&mut me.pos, goal, 15.0) {
                me.angle[1] = me.angle[1].wrapping_add(i16::MIN);
                run.phase = 2;
            }
        }
        2 => {
            let mut goal = ship;
            goal.x += from_s16(index).sin() * -300.0;
            goal.z += from_s16(index).cos() * -300.0;
            if chase_xz(&mut me.pos, goal, 15.0) {
                run.speed = 0.0;
                cut_end(run, list, flags);
                run.offset_target = -150.0;
                run.offset_step = 10.0;
                run.offset_scale = 0.3;
                run.phase = 0;
            }
        }
        _ => {}
    }
}

fn offset_cut(run: &mut StaffRun, list: &EventList, flags: &mut EventFlags, name: &str) {
    if run.advance > 0 {
        if name == "APPEAR" {
            run.offset_target = 110.0;
            run.offset_step = 22.0;
            run.offset_scale = 0.4;
        } else {
            run.offset_target = -150.0;
            run.offset_step = 10.0;
            run.offset_scale = 0.3;
        }
    }
    if (run.model_offset - run.offset_target).abs() < 10.0 {
        cut_end(run, list, flags);
    }
}

fn resolve_target(
    run: &mut StaffRun,
    list: &EventList,
    cut: usize,
    owner: Entity,
    actors: &mut Query<(Entity, &mut Actor)>,
    players: &mut Query<(Entity, &mut Player)>,
) -> Option<Entity> {
    let set_id = list.cut_number(cut, "SetId").unwrap_or(0);
    if set_id != 0 {
        if let Some(found) = actors.iter_mut().find_map(|(entity, actor)| {
            (i32::from(actor.set_id) == set_id).then_some(entity)
        }) {
            run.target = Some(found);
            return run.target;
        }
        if let Some(name) = list.cut_text(cut, "ActorName") {
            if let Some(found) = nearest(name, owner, actors) {
                run.target = Some(found);
            }
        }
        return run.target;
    }
    match list.cut_text(cut, "ActorName") {
        None | Some("Player") => {
            if let Some((entity, _)) = players.iter().next() {
                run.target = Some(entity);
            }
        }
        Some(name) => {
            if let Some(found) = nearest(name, owner, actors) {
                run.target = Some(found);
            }
        }
    }
    run.target
}

fn nearest(
    name: &str,
    owner: Entity,
    actors: &mut Query<(Entity, &mut Actor)>,
) -> Option<Entity> {
    let origin = actors.get_mut(owner).ok().map(|(_, actor)| actor.pos);
    let mut best: Option<(Entity, f32)> = None;
    for (entity, actor) in actors.iter_mut() {
        if actor.name != name {
            continue;
        }
        let dist = match origin {
            Some(origin) => (actor.pos - origin).length(),
            None => 0.0,
        };
        if best.map_or(true, |(_, shortest)| dist < shortest) {
            best = Some((entity, dist));
        }
    }
    best.map(|(entity, _)| entity)
}

fn read_pos(
    target: Entity,
    actors: &mut Query<(Entity, &mut Actor)>,
    players: &mut Query<(Entity, &mut Player)>,
) -> Option<(Vec3, i16)> {
    if let Ok((_, actor)) = actors.get_mut(target) {
        return Some((actor.pos, actor.angle[1]));
    }
    players
        .get(target)
        .ok()
        .map(|(_, player)| (player.pos, to_s16(player.yaw)))
}

fn ship_pos(actors: &mut Query<(Entity, &mut Actor)>) -> Option<(Entity, Vec3, i16)> {
    actors.iter_mut().find_map(|(entity, actor)| {
        (actor.name == SHIP_ACTOR).then_some((entity, actor.pos, actor.angle[1]))
    })
}

fn target_angle_y(from: Vec3, to: Vec3) -> i16 {
    to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn abs_xz(delta: Vec3) -> f32 {
    (delta.x * delta.x + delta.z * delta.z).sqrt()
}

fn turn_body(yaw: i16, target: i16, vel: i16) -> i16 {
    let before = yaw.wrapping_sub(target);
    let after = add_calc_angle_s(before, 0, 4, vel, 0x60);
    if after.unsigned_abs() > 4 {
        yaw.wrapping_add(after.wrapping_sub(before))
    } else {
        target
    }
}

fn chase_xz(pos: &mut Vec3, target: Vec3, step: f32) -> bool {
    let delta = Vec3::new(pos.x - target.x, 0.0, pos.z - target.z);
    let dist = delta.length();
    if step != 0.0 {
        if dist < 8e-11 || dist <= step {
            pos.x = target.x;
            pos.z = target.z;
            return true;
        }
        *pos -= delta * (step / dist);
        return false;
    }
    dist < 8e-11
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
                prev_delta: 0,
                aim_vel: 0,
                speed: 0.0,
                phase: 0,
                model_offset: 0.0,
                offset_target: 0.0,
                offset_step: 0.0,
                offset_scale: 0.0,
                attn: Vec3::ZERO,
                target: None,
                talk_open: false,
                msg_bound: false,
                msg_no: 0,
                get_stage: 0,
                get_msg_live: false,
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
    run.get_stage = 0;
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
    msg: &mut MsgBox,
    table: &MsgTable,
) {
    let Some(staff) = list.staffs.get(run.staff) else {
        return;
    };
    match staff.kind {
        STAFF_ALL | STAFF_CREATE | STAFF_LIGHT => cut_end(run, list, flags),
        STAFF_TIMEKEEPER => timekeeper(run, list, flags),
        STAFF_DIRECTOR => director(run, list, flags, next),
        STAFF_MESSAGE => message(run, list, flags, msg, table),
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

fn message(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    msg: &mut MsgBox,
    table: &MsgTable,
) {
    let action = act_index(run, list, MESSAGE_ACTIONS);
    if run.advance > 0 {
        match action {
            0 | 7 => wait_start(run, list),
            1 => {
                run.msg_bound = false;
                run.msg_no = list.cut_number(run.cut as usize, "msgNo").unwrap_or(0);
                run.phase = 0;
            }
            3 | 5 => {
                if run.msg_bound {
                    msg.set_status(MSG_ENDS);
                }
            }
            4 => {
                if run.msg_bound {
                    run.msg_no = list.cut_number(run.cut as usize, "msgNo").unwrap_or(0);
                    msg.set_status(MSG_CONTINUES);
                    msg.message_set(run.msg_no as u32, table);
                }
            }
            _ => {}
        }
    }
    match action {
        0 | 7 => wait_proc(run, list, flags),
        1 => match run.phase {
            0 => {
                msg.message_set(run.msg_no as u32, table);
                run.phase = 1;
            }
            1 => {
                if msg.active() {
                    run.msg_bound = true;
                    run.phase = 2;
                }
            }
            _ => cut_end(run, list, flags),
        },
        2 => {
            if run.msg_bound && msg.status() == MSG_DISPLAYED {
                cut_end(run, list, flags);
            }
        }
        3 | 6 => {
            if !run.msg_bound {
                cut_end(run, list, flags);
            } else if msg.status() == MSG_BOX_CLOSED {
                msg.set_status(MSG_DESTROYED);
                run.msg_bound = false;
                cut_end(run, list, flags);
            }
        }
        4 => cut_end(run, list, flags),
        5 => {
            if run.msg_bound
                && matches!(msg.status(), MSG_BOX_CLOSING | MSG_BOX_CLOSED)
            {
                cut_end(run, list, flags);
            }
        }
        8 => cut_end(run, list, flags),
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
