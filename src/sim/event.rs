use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::flags::{EventFlags, StoryBits};
use super::player::Player;
use super::sea::Sea;
use super::ship::{Ship, SHIP_ACTOR};
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
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut flags: ResMut<EventFlags>,
    mut story: ResMut<StoryBits>,
    mut next: ResMut<NextStage>,
    sea: Option<Res<Sea>>,
    mut actors: Query<(Entity, &mut Actor)>,
    players: Query<(Entity, &Player)>,
    mut ships: Query<&mut Ship>,
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
        let owner = running.actor;
        for run in running.staffs.iter_mut() {
            staff_cut(
                run,
                &list,
                &mut flags,
                owner,
                &mut actors,
                &players,
                &mut ships,
                sea.as_deref(),
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
    players: &Query<(Entity, &Player)>,
    ships: &mut Query<&mut Ship>,
    sea: Option<&Sea>,
) {
    let Some(staff) = list.staffs.get(run.staff) else {
        return;
    };
    if staff.kind == STAFF_CAMERA {
        camera_cut(run, list, flags);
    } else if staff.kind == STAFF_NORMAL {
        match staff.name.as_str() {
            "NpcSo" => soldier_cut(run, list, flags, owner, actors, players, sea),
            "Link" => link_cut(run, list, flags),
            "Ship" => ship_cut(run, list, flags, actors, ships),
            _ => {}
        }
    }
}

fn soldier_cut(
    run: &mut StaffRun,
    list: &EventList,
    flags: &mut EventFlags,
    owner: Entity,
    actors: &mut Query<(Entity, &mut Actor)>,
    players: &Query<(Entity, &Player)>,
    sea: Option<&Sea>,
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
    if matches!(leading_digits(&cut.name, 2), Some(0 | 5 | 8)) {
        cut_end(run, list, flags);
        return;
    }
    if leading_digits(&cut.name, 2) != Some(1) {
        return;
    }
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
    players: &Query<(Entity, &Player)>,
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
    players: &Query<(Entity, &Player)>,
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
    players: &Query<(Entity, &Player)>,
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
    players: &Query<(Entity, &Player)>,
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
