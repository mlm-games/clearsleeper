use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::combat::HitPoints;
use super::flags::Switches;

pub const SHUTTER_ACTORS: [&str; 7] = [
    "door10",
    "door11",
    "door20",
    "door21",
    "Zenshut",
    "keyshut",
    "K_Zshut",
];

pub const SLIDING_ACTORS: [&str; 8] = [
    "door12",
    "door12M",
    "door12B",
    "door13",
    "door13M",
    "door13B",
    "keyS12",
    "ZenS12",
];

pub const KEY_DOOR_ACTORS: [&str; 5] = ["doorKD", "ZenshKD", "K_ZshKD", "doorSH", "doorSZ"];

pub const ROCK_BARS: usize = 3;
pub const ROCK_RISE_STEP: f32 = 0.25;
pub const ROCK_RISE_MIN: f32 = 0.3;
pub const ROCK_DUST_STEP: f32 = 0.5;
pub const ROCK_DUST_MIN: f32 = 0.3;
pub const ROCK_FALL_STEP: f32 = 0.3;
pub const ROCK_FALL_MIN: f32 = 0.3;

pub const JAIL_ACTOR: &str = "ATdoor";
pub const NO_SWITCH: u8 = 0xFF;
pub const BOSS_SWITCH_FLOOR: u8 = 0x80;

pub const KEY_NONE: u8 = 0;
pub const KEY_NORMAL: u8 = 1;
pub const KEY_BOSS: u8 = 2;

pub const BAR_CLOSED: i32 = 0;
pub const BAR_OPENED: i32 = 1;

pub const BAR_HEIGHT: f32 = 300.0;
pub const BAR_OPEN_SPEED: f32 = 30.0;
pub const BAR_OPEN_ACCEL: f32 = 4.0;
pub const BAR_CLOSE_SPEED: f32 = 60.0;
pub const BAR_CLOSE_ACCEL: f32 = 6.0;
pub const SLIDE_OPEN_SPEED: f32 = 30.0;
pub const SLIDE_OPEN_ACCEL: f32 = 4.0;
pub const SLIDE_CLOSE_SPEED: f32 = 60.0;
pub const SLIDE_CLOSE_ACCEL: f32 = 6.0;
pub const DELAY_FRAMES: u8 = 0x41;
pub const INSTANT_ARG: u8 = 15;
pub const CAGE_ARG: u8 = 17;
pub const KEY_HEIGHT: f32 = 150.0;
pub const BOSS_KEY_HEIGHT: f32 = 230.0;
pub const SHUTTER_CLEAR_TYPES: [u8; 3] = [0, 2, 3];
pub const SLIDE_CLEAR_TYPES: [u8; 2] = [0, 2];
pub const KEY_DOOR_CLEAR_TYPES: [u8; 2] = [0, 1];
pub const SHUTTER_KEY_TYPES: [u8; 2] = [4, 5];
pub const GENOCIDE_TYPES: [u8; 2] = [2, 4];
pub const FEELER_TYPES: [u8; 2] = [3, 4];
pub const LATCH_FRONT: u8 = 2;
pub const LATCH_BACK: u8 = 5;
pub const GENOCIDE_FRONT: u8 = 1;

const ROOM_MASK: u32 = 0x3F;
const ROOM_SHIFT: u32 = 6;
const SWITCH_MASK: u32 = 0xFF;
const TYPE_SHIFT: u32 = 8;
const TYPE_MASK: u32 = 0xF;
const EVENT_SHIFT: u32 = 0x0C;
const BACK_SWITCH_SHIFT: u32 = 0x14;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoorCut {
    pub staff: usize,
    pub opening: bool,
    pub armed: bool,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StayRoom(pub i8);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Shutter,
    Slide,
    Key,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Init,
    Wait,
    StopClose,
    SlideOpen,
    SlideClose,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Door {
    pub parameters: u32,
    pub kind: Kind,
    pub action: Action,
    pub kind_bits: u8,
    pub event_no: u8,
    pub front_room: u8,
    pub back_room: u8,
    pub ship_id: u8,
    pub arg1: u8,
    pub back_switch: u8,
    pub front_check: u8,
    pub key: bool,
    pub bars: i32,
    pub bars_y: f32,
    pub bars_speed: f32,
    pub barrier: i32,
    pub slide: f32,
    pub slide_speed: f32,
    pub delay: u8,
    pub rock_delay: [u8; ROCK_BARS],
    pub rock: [f32; ROCK_BARS],
    pub dust: [f32; ROCK_BARS],
}

impl Door {
    pub fn spawn(parameters: u32, angle: [i16; 3], kind: Kind) -> Self {
        let packed = u32::from(angle[0] as u16);
        Self {
            parameters,
            kind,
            action: Action::Init,
            kind_bits: ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8,
            event_no: ((parameters >> EVENT_SHIFT) & SWITCH_MASK) as u8,
            front_room: (packed & ROOM_MASK) as u8,
            back_room: ((packed >> ROOM_SHIFT) & ROOM_MASK) as u8,
            ship_id: (angle[2] as u8 & ROOM_MASK as u8),
            arg1: (angle[2] >> 8) as u8,
            back_switch: ((parameters >> BACK_SWITCH_SHIFT) & SWITCH_MASK) as u8,
            front_check: 2,
            key: false,
            bars: BAR_CLOSED,
            bars_y: 0.0,
            bars_speed: 0.0,
            barrier: BAR_CLOSED,
            slide: 0.0,
            slide_speed: 0.0,
            delay: DELAY_FRAMES,
            rock_delay: [0; ROCK_BARS],
            rock: [1.0; ROCK_BARS],
            dust: [1.0; ROCK_BARS],
        }
    }

    pub fn switch_no(&self) -> u8 {
        (self.parameters & SWITCH_MASK) as u8
    }

    pub fn is_shutter(name: &str) -> bool {
        SHUTTER_ACTORS.contains(&name)
    }

    pub fn is_sliding(name: &str) -> bool {
        SLIDING_ACTORS.contains(&name)
    }

    pub fn is_key_door(name: &str) -> bool {
        KEY_DOOR_ACTORS.contains(&name)
    }

    pub fn key_kind(&self) -> u8 {
        match self.kind {
            Kind::Shutter if SHUTTER_KEY_TYPES.contains(&self.kind_bits) => KEY_NORMAL,
            Kind::Shutter if self.kind_bits == 1 => KEY_BOSS,
            Kind::Slide if self.kind_bits == 1 => KEY_NORMAL,
            Kind::Slide if self.kind_bits == 3 => KEY_BOSS,
            Kind::Key if self.kind_bits == 2 => KEY_NORMAL,
            Kind::Shutter | Kind::Slide | Kind::Key => KEY_NONE,
        }
    }

    pub fn clear_types(&self) -> &'static [u8] {
        match self.kind {
            Kind::Shutter => &SHUTTER_CLEAR_TYPES,
            Kind::Slide => &SLIDE_CLEAR_TYPES,
            Kind::Key => &KEY_DOOR_CLEAR_TYPES,
        }
    }

    pub fn bare_type(&self) -> u8 {
        match self.kind {
            Kind::Shutter => 1,
            Kind::Slide => 3,
            Kind::Key => 0xFF,
        }
    }

    pub fn makes_stop(&self) -> bool {
        self.back_switch != NO_SWITCH
            || (self.key_kind() == KEY_NONE && self.switch_no() != NO_SWITCH)
    }

    pub fn set_key(&mut self, switches: &Switches) {
        self.key = match self.key_kind() {
            KEY_NORMAL => !switches.test(self.switch_no()),
            KEY_BOSS => {
                self.switch_no() == NO_SWITCH
                    || self.switch_no() >= BOSS_SWITCH_FLOOR
                    || !switches.test(self.switch_no())
            }
            _ => false,
        };
    }

    pub fn key_height(&self) -> f32 {
        if self.key_kind() == KEY_BOSS {
            BOSS_KEY_HEIGHT
        } else {
            KEY_HEIGHT
        }
    }

    pub fn cage(&self) -> bool {
        self.arg1 == CAGE_ARG
    }

    pub fn facing(&self, stay: i8) -> u8 {
        if self.front_room == self.back_room {
            return self.front_check;
        }
        match stay {
            room if room == i16::from(self.front_room) as i8 => 0,
            room if room == i16::from(self.back_room) as i8 => 1,
            _ => 2,
        }
    }

    pub fn room(&self) -> u8 {
        if self.front_check == 0 {
            self.front_room
        } else {
            self.back_room
        }
    }

    pub fn bit(&self) -> u8 {
        if self.front_check == 0 {
            self.switch_no()
        } else {
            self.back_switch
        }
    }

    pub fn stop_front(&self, loaded: bool, switches: &Switches) -> i32 {
        if self.switch_no() == NO_SWITCH {
            return BAR_CLOSED;
        }
        if !self.clear_types().contains(&self.kind_bits) {
            return BAR_CLOSED;
        }
        if !loaded {
            return -1;
        }
        i32::from(!switches.test(self.switch_no()))
    }

    pub fn stop_back(&self, loaded: bool, switches: &Switches) -> i32 {
        if self.back_switch == NO_SWITCH {
            return BAR_CLOSED;
        }
        if !loaded {
            return -1;
        }
        i32::from(!switches.test(self.back_switch))
    }

    pub fn barrier_value(&self, loaded: bool, switches: &Switches) -> i32 {
        let front = self.stop_front(loaded, switches);
        let back = self.stop_back(loaded, switches);
        match self.front_check {
            0 => front,
            1 => back,
            _ => BAR_CLOSED,
        }
    }

    pub fn blocking(&self) -> bool {
        self.barrier != BAR_CLOSED
    }

    pub fn latching(&self) -> bool {
        if self.kind == Kind::Key {
            return self.genocide();
        }
        LATCH_FRONT == self.kind_bits && self.front_check == 0
            || LATCH_BACK == self.kind_bits && self.front_check == 1
    }

    pub fn genocide(&self) -> bool {
        if self.kind != Kind::Key {
            return false;
        }
        if self.front_check == 0 {
            self.kind_bits == GENOCIDE_FRONT
        } else {
            GENOCIDE_TYPES.contains(&self.kind_bits)
        }
    }

    pub fn feeler(&self) -> bool {
        self.kind == Kind::Key && self.front_check == 0 && FEELER_TYPES.contains(&self.kind_bits)
    }

    pub fn should_close(&self, switches: &Switches) -> bool {
        if !self.makes_stop() || self.genocide() || self.feeler() {
            return false;
        }
        if self.kind_bits == self.bare_type() {
            return false;
        }
        if self.front_check == 0 && self.kind_bits == LATCH_FRONT {
            return false;
        }
        let bit = self.bit();
        bit != NO_SWITCH && !switches.test(bit)
    }

    pub fn should_open(&self, switches: &Switches) -> bool {
        self.bit() != NO_SWITCH && switches.test(self.bit())
    }

    pub fn reopen_delay(&self) -> u8 {
        if self.instant() {
            0
        } else {
            DELAY_FRAMES
        }
    }

    pub fn instant(&self) -> bool {
        self.arg1 == INSTANT_ARG
    }
}

pub fn control(
    stay: Res<StayRoom>,
    mut switches: ResMut<Switches>,
    mut doors: Query<(Entity, &mut Door)>,
    actors: Query<(&Actor, Option<&HitPoints>)>,
    mut cuts: Query<&mut DoorCut>,
    mut active: ResMut<super::event::ActiveEvents>,
    list: Res<super::event::EventList>,
    mut flags: ResMut<super::flags::EventFlags>,
) {
    let room = stay.0;
    for (entity, mut door) in &mut doors {
        door.set_key(&switches);
        door.front_check = door.facing(room);
        door.barrier = door.barrier_value(room >= 0, &switches);
        if let Ok(mut cut) = cuts.get_mut(entity) {
            drive_cut(door, &mut cut, &mut switches, &active, &list, &mut flags);
        }
        if !door.makes_stop() {
            advance(&mut door);
            continue;
        }
        if door.bars == BAR_CLOSED && door.should_close(&switches) {
            door.bars = BAR_CLOSED;
            door.delay = door.reopen_delay();
            advance(&mut door);
            continue;
        }
        let clear = door.latching()
            && room >= 0
            && !actors
                .iter()
                .any(|(actor, hp)| hp.is_some() && actor.room == room);
        if clear {
            if door.delay > 0 {
                door.delay -= 1;
                advance(&mut door);
                continue;
            }
            if door.bit() != NO_SWITCH {
                switches.set(door.bit());
            }
        }
        if door.should_open(&switches) || clear {
            door.bars = BAR_OPENED;
        }
        advance(&mut door);
    }
}

fn drive_cut(
    door: &mut Door,
    cut: &mut DoorCut,
    switches: &mut Switches,
    active: &super::event::ActiveEvents,
    list: &super::event::EventList,
    flags: &mut super::flags::EventFlags,
) {
    if !cut.armed {
        return;
    }
    let bit = door.bit();
    let rising = match door.kind {
        Kind::Slide => door.slide,
        Kind::Key => door.rock[0],
        Kind::Shutter => door.bars_y,
    };
    let done = if cut.opening {
        door.bars = BAR_OPENED;
        if door.kind == Kind::Slide {
            door.action = Action::SlideOpen;
        }
        if bit != NO_SWITCH {
            switches.set(bit);
        }
        rising >= BAR_HEIGHT || (door.kind == Kind::Key && rising >= 1.0)
    } else {
        door.bars = BAR_CLOSED;
        if door.kind == Kind::Slide {
            door.action = Action::SlideClose;
        }
        if bit != NO_SWITCH {
            switches.clear(bit);
        }
        rising <= 0.0
    };
    if !done {
        return;
    }
    cut.armed = false;
    if let Some(flag) = active.staff_cut_flag(list, cut.staff) {
        flags.set(flag as u32);
    }
}

fn advance(door: &mut Door) {
    if door.kind == Kind::Key {
        rocks(door);
        return;
    }
    match door.bars {
        BAR_OPENED => {
            chase(&mut door.bars_speed, BAR_OPEN_SPEED, BAR_OPEN_ACCEL);
            chase(&mut door.bars_y, BAR_HEIGHT, door.bars_speed);
        }
        _ => {
            chase(&mut door.bars_speed, BAR_CLOSE_SPEED, BAR_CLOSE_ACCEL);
            chase(&mut door.bars_y, 0.0, door.bars_speed);
        }
    }
    match door.action {
        Action::SlideOpen => {
            chase(&mut door.slide_speed, SLIDE_OPEN_SPEED, SLIDE_OPEN_ACCEL);
            if chase(&mut door.slide, BAR_HEIGHT, door.slide_speed) {
                door.action = Action::Init;
                door.slide_speed = 0.0;
            }
        }
        Action::SlideClose => {
            chase(&mut door.slide_speed, SLIDE_CLOSE_SPEED, SLIDE_CLOSE_ACCEL);
            if chase(&mut door.slide, 0.0, door.slide_speed) {
                door.action = Action::Init;
                door.slide_speed = 0.0;
            }
        }
        Action::Init | Action::Wait | Action::StopClose => {}
    }
}

fn rocks(door: &mut Door) {
    let opening = door.bars == BAR_OPENED;
    for bar in 0..ROCK_BARS {
        if door.rock_delay[bar] > 0 {
            door.rock_delay[bar] -= 1;
            continue;
        }
        if opening {
            add_zero(&mut door.rock[bar], ROCK_RISE_STEP, ROCK_RISE_MIN);
            add_zero(&mut door.dust[bar], ROCK_DUST_STEP, ROCK_DUST_MIN);
        } else {
            add_target(&mut door.dust[bar], 1.0, ROCK_DUST_STEP, ROCK_DUST_MIN);
            add_target(&mut door.rock[bar], 1.0, ROCK_FALL_STEP, ROCK_FALL_MIN);
        }
    }
}

fn add_zero(current: &mut f32, max_step: f32, min_step: f32) {
    let gap = current.abs();
    if gap <= min_step {
        *current = 0.0;
        return;
    }
    *current -= current.signum() * max_step.min(gap);
}

fn add_target(current: &mut f32, target: f32, max_step: f32, min_step: f32) {
    let gap = target - *current;
    if gap.abs() <= min_step {
        *current = target;
        return;
    }
    *current += gap.signum() * max_step.min(gap.abs());
}

fn chase(current: &mut f32, target: f32, step: f32) -> bool {
    let diff = target - *current;
    if diff.abs() <= step {
        *current = target;
        true
    } else {
        *current += diff.signum() * step;
        false
    }
}

pub fn solid(door: &Door) -> bool {
    door.bars_y < BAR_HEIGHT
}

pub const JAIL_OPEN: i16 = 0x4000;
pub const JAIL_STEP: i16 = 0x400;
pub const JAIL_HEIGHT: f32 = 150.0;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Jail {
    Wait,
    CloseWait,
    Close,
    OpenWait,
    Open,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct JailDoor {
    pub switch_no: u8,
    pub action: Jail,
    pub swing: i16,
}

impl JailDoor {
    pub fn spawn(parameters: u32, latched: bool) -> Self {
        let locked = (parameters & SWITCH_MASK) as u8 == NO_SWITCH;
        let (action, swing) = match (locked, latched) {
            (true, _) => (Jail::Wait, 0),
            (false, true) => (Jail::OpenWait, JAIL_OPEN),
            (false, false) => (Jail::CloseWait, 0),
        };
        Self {
            switch_no: (parameters & SWITCH_MASK) as u8,
            action,
            swing,
        }
    }

    pub fn open(&self) -> bool {
        self.swing >= JAIL_OPEN
    }
}

pub fn jail_control(switches: Res<Switches>, mut doors: Query<&mut JailDoor>) {
    for mut door in &mut doors {
        if matches!(door.action, Jail::Wait) {
            continue;
        }
        let latched = switches.test(door.switch_no);
        door.action = match door.action {
            Jail::Wait => Jail::Wait,
            Jail::CloseWait if latched => Jail::Open,
            Jail::CloseWait => Jail::CloseWait,
            Jail::OpenWait if !latched => Jail::Close,
            Jail::OpenWait => Jail::OpenWait,
            Jail::Close => {
                door.swing = (door.swing - JAIL_STEP).max(0);
                if door.swing == 0 {
                    Jail::CloseWait
                } else {
                    Jail::Close
                }
            }
            Jail::Open => {
                door.swing = (door.swing + JAIL_STEP).min(JAIL_OPEN);
                if door.swing >= JAIL_OPEN {
                    Jail::OpenWait
                } else {
                    Jail::Open
                }
            }
        };
    }
}