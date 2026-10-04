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

pub const NO_ROOM: u8 = 0x3F;
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
pub const SHUTTER_KEY_TYPES: [u8; 2] = [4, 5];

const ROOM_MASK: u32 = 0x3F;
const ROOM_SHIFT: u32 = 6;
const SWITCH_MASK: u32 = 0xFF;
const TYPE_SHIFT: u32 = 8;
const TYPE_MASK: u32 = 0xF;
const EVENT_SHIFT: u32 = 0x0C;
const BACK_SWITCH_SHIFT: u32 = 0x14;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StayRoom(pub i8);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Shutter,
    Slide,
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
    pub front_bar: i32,
    pub back_bar: i32,
    pub slide: f32,
    pub slide_speed: f32,
    pub open_flag: bool,
    pub close_flag: bool,
    pub delay: u8,
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
            ship_id: (angle[2] as u8) & 0x3F,
            arg1: (angle[2] >> 8) as u8,
            back_switch: ((parameters >> BACK_SWITCH_SHIFT) & SWITCH_MASK) as u8,
            front_check: 2,
            key: false,
            bars: BAR_CLOSED,
            bars_y: 0.0,
            bars_speed: 0.0,
            front_bar: BAR_CLOSED,
            back_bar: BAR_CLOSED,
            slide: 0.0,
            slide_speed: 0.0,
            open_flag: false,
            close_flag: false,
            delay: DELAY_FRAMES,
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

    pub fn key_kind(&self) -> u8 {
        match self.kind {
            Kind::Shutter if SHUTTER_KEY_TYPES.contains(&self.kind_bits) => KEY_NORMAL,
            Kind::Shutter if self.kind_bits == 1 => KEY_BOSS,
            Kind::Slide if self.kind_bits == 1 => KEY_NORMAL,
            Kind::Slide if self.kind_bits == 3 => KEY_BOSS,
            Kind::Shutter | Kind::Slide => KEY_NONE,
        }
    }

    pub fn clear_types(&self) -> &'static [u8] {
        match self.kind {
            Kind::Shutter => &SHUTTER_CLEAR_TYPES,
            Kind::Slide => &SLIDE_CLEAR_TYPES,
        }
    }

    pub fn bare_type(&self) -> u8 {
        match self.kind {
            Kind::Shutter => 1,
            Kind::Slide => 3,
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

    pub fn latching(&self) -> bool {
        self.front_check == 0 && self.kind_bits == 2 || self.front_check == 1 && self.kind_bits == 5
    }

    pub fn should_close(&self, switches: &Switches) -> bool {
        if !self.makes_stop() || self.kind_bits == self.bare_type() {
            return false;
        }
        if self.front_check == 0 && self.kind_bits == 2 {
            return false;
        }
        let bit = self.bit();
        bit != NO_SWITCH && !switches.test(bit)
    }

    pub fn should_open(&self, switches: &Switches) -> bool {
        self.bit() != NO_SWITCH && switches.test(self.bit())
    }

    pub fn reopen_delay(&mut self) -> u8 {
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
    mut doors: Query<&mut Door>,
    actors: Query<(&Actor, Option<&HitPoints>)>,
) {
    let room = stay.0;
    for mut door in &mut doors {
        door.set_key(&switches);
        door.front_check = door.facing(room);
        if !door.makes_stop() {
            continue;
        }
        let loaded = room >= 0;
        let (front, back) = (
            door.stop_front(loaded, &switches),
            door.stop_back(loaded, &switches),
        );
        if door.front_check == 0 {
            door.front_bar = front;
            door.back_bar = back;
        } else {
            door.front_bar = back;
            door.back_bar = front;
        }
        if door.bars != BAR_OPENED {
            if door.should_close(&switches) {
                door.bars = BAR_CLOSED;
                door.close_flag = true;
                door.action = Action::StopClose;
                door.delay = door.reopen_delay();
                continue;
            }
        }
        let clear = door.latching()
            && room >= 0
            && !actors.iter().any(|(actor, hp)| {
                hp.is_some() && actor.room == room
            });
        if clear {
            if door.delay > 0 {
                door.delay -= 1;
                continue;
            }
            if door.bit() != NO_SWITCH {
                switches.set(door.bit());
            }
        }
        if door.should_open(&switches) || clear {
            if door.bars == BAR_CLOSED {
                door.bars = BAR_OPENED;
                door.bars_y = 0.0;
                door.bars_speed = 0.0;
                door.open_flag = true;
            }
        }
        advance(&mut door);
    }
}

fn advance(door: &mut Door) {
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