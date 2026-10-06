use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::enemy::Enemy;
use super::flags::Switches;

/// Stage strings for `fpcNm_DOOR10_e` (`daDoor10_c`, `d_a_door10.cpp:894`). `d_stage.cpp:451-457`
/// is the proof: every one of these seven names is registered against DOOR10, NOT DOOR12. They
/// share the `dDoor_info_c` base with `daDoor12_c` and the identical `dDoor_stop_c` stop bars, so
/// the switch/stop logic below is transcribed for them, but they are a different translation
/// unit from this module's namesake.
pub const SHUTTER_ACTORS: [&str; 7] = [
    "door10",  // <- d_stage.cpp:451
    "door11",  // <- d_stage.cpp:452
    "door20",  // <- d_stage.cpp:453
    "door21",  // <- d_stage.cpp:454
    "Zenshut", // <- d_stage.cpp:455
    "keyshut", // <- d_stage.cpp:456
    "K_Zshut", // <- d_stage.cpp:457
];

/// The eight names of this module's own actor. `d_a_door12.cpp:911` registers exactly one profile
/// with `fpcNm_DOOR12_e`, and `d_stage.cpp:473-480` binds all eight stage strings to it with a
/// 0..5 sub-index that selects the shape (`getShapeType`, `d_a_door12.cpp:289-304`) - `keyS12` and
/// `ZenS12` reuse shape types 1 and 2. `door12M`/`door12B` are NOT separate actors: they are the
/// `DOOR12M`/`DOOR12B` BDLs inside the single `door12` arc (`d_a_door12.cpp:321-333`).
pub const SLIDING_ACTORS: [&str; 8] = [
    "door12",  // <- d_stage.cpp:473 (sub 0)
    "door12M", // <- d_stage.cpp:474 (sub 1)
    "door12B", // <- d_stage.cpp:475 (sub 2)
    "door13",  // <- d_stage.cpp:476 (sub 3)
    "door13M", // <- d_stage.cpp:477 (sub 4)
    "door13B", // <- d_stage.cpp:478 (sub 5)
    "keyS12",  // <- d_stage.cpp:479 (sub 1)
    "ZenS12",  // <- d_stage.cpp:480 (sub 2)
];

/// Stage strings for `fpcNm_KDDOOR_e` (`daKddoor_c`, `d_a_kddoor.cpp:1005`), proven by
/// `d_stage.cpp:460-464`. These are the key/locked doors with the three falling rock slabs
/// (`dDoor_ssk_c`), transcribed below. `SLIDE_*` below is the rock door's leaf motion.
pub const KEY_DOOR_ACTORS: [&str; 5] = [
    "doorKD",  // <- d_stage.cpp:460
    "ZenshKD", // <- d_stage.cpp:461
    "K_ZshKD", // <- d_stage.cpp:462
    "doorSH",  // <- d_stage.cpp:463
    "doorSZ",  // <- d_stage.cpp:464
];

pub const ROCK_BARS: usize = 3; // <- d_a_kddoor.cpp:196 (dDoor_ssk_c::init, `for i < 3`)
pub const ROCK_RISE_STEP: f32 = 0.25; // <- d_a_kddoor.cpp:398 (cLib_addCalc0 scale)
pub const ROCK_RISE_MIN: f32 = 0.3; // <- d_a_kddoor.cpp:398 (maxStep)
pub const ROCK_DUST_STEP: f32 = 0.5; // <- d_a_kddoor.cpp:408 (cLib_addCalc0 scale)
pub const ROCK_DUST_MIN: f32 = 0.3; // <- d_a_kddoor.cpp:408 (maxStep)
pub const ROCK_FALL_STEP: f32 = 0.3; // <- d_a_kddoor.cpp:462 (cLib_addCalc2 scale)
pub const ROCK_FALL_MIN: f32 = 0.3; // <- d_a_kddoor.cpp:462 (maxStep)
pub const ROCK_OPEN_GATE: f32 = 0.1; // <- d_a_kddoor.cpp:393, :407 (openProc `> 0.1f`)
pub const ROCK_CLOSE_GATE: f32 = 0.9; // <- d_a_kddoor.cpp:438, :461 (closeProc `< 0.9f`)

/// `fpcNm_ATDOOR_e` is `d_a_atdoor.cpp` (`d_stage.cpp:458`), not this translation unit, and
/// `src/sim/atdoor.rs` already owns it with citations. `JailDoor` below is a second, uncited copy
/// of that actor kept only because `src/sim/mod.rs` and `src/lib.rs` still construct and read it;
/// the wiring owner should collapse it onto `atdoor::Atdoor`.
pub const JAIL_ACTOR: &str = "ATdoor"; // <- d_stage.cpp:458

pub const NO_SWITCH: u8 = 0xFF; // <- d_a_door12.cpp:61, :92 (the `0xFF` "no switch" arm)
pub const BOSS_SWITCH_FLOOR: u8 = 0x80; // <- d_a_door12.cpp:66 (`getSwbit() >= 0x80`)

pub const KEY_NONE: u8 = 0; // <- d_a_door12.cpp:48 / d_a_door10.cpp:23 (chkMakeKey fallthrough)
pub const KEY_NORMAL: u8 = 1; // <- d_a_door12.cpp:42 / d_a_door10.cpp:16 (type 4, 5 -> 1)
pub const KEY_BOSS: u8 = 2; // <- d_a_door12.cpp:46 / d_a_door10.cpp:21 (type 3 / type 1 -> 2)

pub const BAR_CLOSED: i32 = 0; // <- d_door.cpp:561 (closeInit mOffsY = 300, i.e. fully up/closed)
pub const BAR_OPENED: i32 = 1; // <- d_door.cpp:587 (openInit mOffsY = 0, i.e. fully down/open)

/// `dDoor_stop_c::openProc` chases `mOffsY` to 300.0 and `closeProc` to 0.0
/// (`d_door.cpp:603`, `:577`); `closeInit` parks it at 300.0 (`:561`). This is the STOP-BAR
/// travel, shared by all three door classes - it is NOT the leaf (`slide`) distance.
pub const BAR_HEIGHT: f32 = 300.0; // <- d_door.cpp:603
pub const BAR_OPEN_SPEED: f32 = 30.0; // <- d_door.cpp:602 (cLib_chaseF speedF target)
pub const BAR_OPEN_ACCEL: f32 = 4.0; // <- d_door.cpp:602 (cLib_chaseF step)
pub const BAR_CLOSE_SPEED: f32 = 60.0; // <- d_door.cpp:576 (cLib_chaseF speedF target)
pub const BAR_CLOSE_ACCEL: f32 = 6.0; // <- d_door.cpp:576 (cLib_chaseF step)

/// LEAF travel, which is per translation unit and NOT the stop-bar distance.
/// `daKddoor_c` opens `field_0x94C` to 300.0 with speed 30.0/4.0 and closes to 0.0 with
/// 60.0/6.0 (`d_a_kddoor.cpp:654-655`, `:682-683`).
pub const KEY_SLIDE_OPEN: f32 = 300.0; // <- d_a_kddoor.cpp:655
pub const KEY_SLIDE_OPEN_SPEED: f32 = 30.0; // <- d_a_kddoor.cpp:654
pub const KEY_SLIDE_OPEN_ACCEL: f32 = 4.0; // <- d_a_kddoor.cpp:654
pub const KEY_SLIDE_CLOSE_SPEED: f32 = 60.0; // <- d_a_kddoor.cpp:682
pub const KEY_SLIDE_CLOSE_ACCEL: f32 = 6.0; // <- d_a_kddoor.cpp:682
/// `daDoor10_c` opens `m358` to 300.0 with speed 30.0/4.0 and closes to 0.0 with 60.0/6.0
/// (`d_a_door10.cpp:395-396`, `:426-427`) - identical to the key door.
pub const SHUT_SLIDE_OPEN: f32 = 300.0; // <- d_a_door10.cpp:396
pub const SHUT_SLIDE_OPEN_SPEED: f32 = 30.0; // <- d_a_door10.cpp:395
pub const SHUT_SLIDE_OPEN_ACCEL: f32 = 4.0; // <- d_a_door10.cpp:395
pub const SHUT_SLIDE_CLOSE_SPEED: f32 = 60.0; // <- d_a_door10.cpp:426
pub const SHUT_SLIDE_CLOSE_ACCEL: f32 = 6.0; // <- d_a_door10.cpp:426
/// `daDoor12_c` is the outlier: `openProc` and `closeProc` BOTH chase `speedF` to 20.0 by 2.0,
/// and the travel target is 200.0, not 300.0 (`d_a_door12.cpp:459-460`, `:501-502`).
/// `openWide` (`d_a_door12.cpp:358-372`) then scales the actual offset by
/// `m31C * 0.005f * openWide()` with `openWide` 140.0 (shapes 3/4), 220.0 (shape 5), else 200.0.
pub const SLIDE_OPEN: f32 = 200.0; // <- d_a_door12.cpp:460
pub const SLIDE_OPEN_SPEED: f32 = 20.0; // <- d_a_door12.cpp:459
pub const SLIDE_OPEN_ACCEL: f32 = 2.0; // <- d_a_door12.cpp:459
pub const SLIDE_CLOSE_SPEED: f32 = 20.0; // <- d_a_door12.cpp:501 (same chase as open)
pub const SLIDE_CLOSE_ACCEL: f32 = 2.0; // <- d_a_door12.cpp:501
pub const SLIDE_WIDE_NARROW: f32 = 140.0; // <- d_a_door12.cpp:361 (openWide, shapes 3 and 4)
pub const SLIDE_WIDE_WIDE: f32 = 220.0; // <- d_a_door12.cpp:365 (openWide, shape 5)
pub const SLIDE_WIDE_DEFAULT: f32 = 200.0; // <- d_a_door12.cpp:369 (openWide default)
pub const SLIDE_OFFSET_SCALE: f32 = 0.005; // <- d_a_door12.cpp:525 (calcMtx `m31C * 0.005f`)

/// `CreateInit` arms `m2A1` to 0x41 (`d_a_door12.cpp:556`) and `chkStopOpen` re-arms it to the
/// same value when the room check fails (`d_a_door12.cpp:174`).
pub const DELAY_FRAMES: u8 = 0x41; // <- d_a_door12.cpp:174, :556
pub const CAGE_ARG: u8 = 17; // <- d_door.cpp:563, :589 (`getArg1() == 17` -> JA_SE_OBJ_CAGE_*)
pub const KEY_HEIGHT: f32 = 150.0; // <- d_door.cpp:534 (dDoor_key2_c calcMtx default)
pub const BOSS_KEY_HEIGHT: f32 = 230.0; // <- d_door.cpp:533 (dDoor_key2_c calcMtx boss case)
/// `daDoor10_c::chkStopF` clears on `case 0: case 2: case 3:`.
pub const SHUTTER_CLEAR_TYPES: [u8; 3] = [0, 2, 3]; // <- d_a_door10.cpp:77-79
/// `daDoor12_c::chkStopF` clears on `case 0: case 2:`.
pub const SLIDE_CLEAR_TYPES: [u8; 2] = [0, 2]; // <- d_a_door12.cpp:97-98
/// `daKddoor_c::chkStopF` clears on `case 0: case 1:`.
pub const KEY_DOOR_CLEAR_TYPES: [u8; 2] = [0, 1]; // <- d_a_kddoor.cpp:49-50
/// `daDoor10_c::chkMakeKey` returns 1 for `type == 4 || type == 5`.
pub const SHUTTER_KEY_TYPES: [u8; 2] = [4, 5]; // <- d_a_door10.cpp:16
/// `daKddoor_c::chkGenocideCase` non-front arm.
pub const GENOCIDE_TYPES: [u8; 2] = [2, 4]; // <- d_a_kddoor.cpp:98-99
/// `daKddoor_c::chkFeelerCase` front arm.
pub const FEELER_TYPES: [u8; 2] = [3, 4]; // <- d_a_kddoor.cpp:111-112
/// `daDoor10_c::chkStopClose` returns FALSE for `type == 2` on the front arm.
pub const LATCH_FRONT: u8 = 2; // <- d_a_door10.cpp:199
/// `daDoor10_c::chkStopClose` returns FALSE for `type == 5` on the back arm.
pub const LATCH_BACK: u8 = 5; // <- d_a_door10.cpp:205
/// `daKddoor_c::chkGenocideCase` front arm.
pub const GENOCIDE_FRONT: u8 = 1; // <- d_a_kddoor.cpp:93

const ROOM_MASK: u32 = 0x3F; // <- d_door.cpp:45 (getFRoomNo)
const ROOM_SHIFT: u32 = 6; // <- d_door.cpp:50 (getBRoomNo)
const SWITCH_MASK: u32 = 0xFF; // <- d_door.cpp:17 (getSwbit)
const TYPE_SHIFT: u32 = 8; // <- d_door.cpp:27 (getType)
const TYPE_MASK: u32 = 0xF; // <- d_door.cpp:27 (getType)
const EVENT_SHIFT: u32 = 0x0C; // <- d_door.cpp:40 (getEventNo)
const BACK_SWITCH_SHIFT: u32 = 0x14; // <- d_door.cpp:22 (getSwbit2)

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoorCut {
    pub flag: u32,
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

    /// `(open distance, open speed, open accel, close speed, close accel)` for the LEAF. Retail
    /// uses one `cLib_chaseF(&speedF, S, A)` followed by `cLib_chaseF(&leaf, T, speedF)` per
    /// direction, and the numbers are per class: `daDoor12_c` 20.0/2.0 to 200.0
    /// (`d_a_door12.cpp:459-460`, `:501-502`), `daDoor10_c` and `daKddoor_c` 30.0/4.0 to 300.0 and
    /// 60.0/6.0 to 0.0 (`d_a_door10.cpp:395-396`, `:426-427`; `d_a_kddoor.cpp:654-655`,
    /// `:682-683`). They are NOT interchangeable.
    pub fn leaf_motion(&self) -> (f32, f32, f32, f32, f32) {
        match self.kind {
            Kind::Slide => (
                SLIDE_OPEN,
                SLIDE_OPEN_SPEED,
                SLIDE_OPEN_ACCEL,
                SLIDE_CLOSE_SPEED,
                SLIDE_CLOSE_ACCEL,
            ),
            Kind::Shutter => (
                SHUT_SLIDE_OPEN,
                SHUT_SLIDE_OPEN_SPEED,
                SHUT_SLIDE_OPEN_ACCEL,
                SHUT_SLIDE_CLOSE_SPEED,
                SHUT_SLIDE_CLOSE_ACCEL,
            ),
            Kind::Key => (
                KEY_SLIDE_OPEN,
                KEY_SLIDE_OPEN_SPEED,
                KEY_SLIDE_OPEN_ACCEL,
                KEY_SLIDE_CLOSE_SPEED,
                KEY_SLIDE_CLOSE_ACCEL,
            ),
        }
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

    /// `chkStopOpen` re-arms `m2A1` to `0x41` on every failure path
    /// (`d_a_door12.cpp:174`, and again in `CreateInit` at `:556`) and only
    /// counts it down when the room is displayed and holds no enemy
    /// (`:163-165`). There is no `getArg1()` value anywhere in the door family
    /// (`d_a_door10.cpp:486-509`, `d_a_door12.cpp:40,89,143,197,385,408`) that
    /// skips the countdown, so the re-arm delay is unconditional.
    pub fn reopen_delay(&self) -> u8 {
        DELAY_FRAMES
    }
}

pub fn control(
    stay: Res<StayRoom>,
    mut switches: ResMut<Switches>,
    mut doors: Query<(Entity, &mut Door)>,
    actors: Query<(&Actor, Option<&HitPoints>)>,
    mut cuts: Query<&mut DoorCut>,
    mut flags: ResMut<super::flags::EventFlags>,
) {
    let room = stay.0;
    for (entity, mut door) in &mut doors {
        door.set_key(&switches);
        door.front_check = door.facing(room);
        door.barrier = door.barrier_value(room >= 0, &switches);
        if let Ok(mut cut) = cuts.get_mut(entity) {
            drive_cut(&mut *door, &mut cut, &mut switches, &mut flags);
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
    flags: &mut super::flags::EventFlags,
) {
    if !cut.armed {
        return;
    }
    let bit = door.bit();
    let (open, ..) = door.leaf_motion();
    let done = if cut.opening {
        door.bars = BAR_OPENED;
        if door.kind == Kind::Slide {
            door.action = Action::SlideOpen;
        }
        if bit != NO_SWITCH {
            switches.set(bit);
        }
        match door.kind {
            Kind::Slide => door.slide >= open,
            Kind::Key => door.rock[0] <= 0.0,
            Kind::Shutter => door.bars_y >= BAR_HEIGHT,
        }
    } else {
        door.bars = BAR_CLOSED;
        if door.kind == Kind::Slide {
            door.action = Action::SlideClose;
        }
        if bit != NO_SWITCH {
            switches.clear(bit);
        }
        match door.kind {
            Kind::Slide => door.slide <= 0.0,
            Kind::Key => door.rock[0] >= 1.0,
            Kind::Shutter => door.bars_y <= 0.0,
        }
    };
    if !done {
        return;
    }
    cut.armed = false;
    flags.set(cut.flag);
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
    let (open, open_speed, open_accel, close_speed, close_accel) = door.leaf_motion();
    match door.action {
        Action::SlideOpen => {
            chase(&mut door.slide_speed, open_speed, open_accel);
            if chase(&mut door.slide, open, door.slide_speed) {
                door.action = Action::Init;
                door.slide_speed = 0.0;
            }
        }
        Action::SlideClose => {
            chase(&mut door.slide_speed, close_speed, close_accel);
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