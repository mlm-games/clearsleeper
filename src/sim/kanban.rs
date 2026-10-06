use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::arrow;
use super::bomb;
use super::boomerang;
use super::collision::Collision;
use super::combat::{Combat, CutKind};
use super::events::{SimEvent, SimEvents};
use super::ground;
use super::hammer;
use super::hookshot;
use super::intent::DoIntent;
use super::msg::{MSG_BOX_CLOSED, MSG_DESTROYED, MsgBox, MsgTable};
use super::player::Player;
use super::player_status::{PlayerStatus, STTS1_WIND_WAKER_CONDUCT};
use super::sea::Sea;
use super::stream::add_calc_angle_s2;
use super::walk::{from_s16, to_s16};
use super::water::{self, Water};

pub const KANBAN_ACTORS: [&str; 1] = ["Kanban"];

pub fn is_kanban(name: &str) -> bool {
    KANBAN_ACTORS.contains(&name)
}

const PARAM_PARTS_SHIFT: u32 = 0x14;

const TIMER_COUNT: usize = 5;
const TIMER_RANDOM: usize = 0;
const TIMER_SINK: usize = 1;
const TIMER_SHOCK: usize = 4;

const MODE_MOTHER: i16 = 0;
const MODE_MOTHER_RETURN: i16 = 1;
const MODE_MOTHER_WATER: i16 = 2;
const MODE_PART: i16 = 100;
const MODE_PART_BACK: i16 = 0x65;
const MODE_PART_WATER: i16 = 0x66;

const STATE_TALK: i16 = 10;
const STATE_TOSS: i16 = 11;
const STATE_FALL: i16 = 12;
const STATE_BOUNCE: i16 = 13;
const STATE_QUAKE_WAIT: i16 = 14;
const STATE_RETURN_TURN: i16 = 0x14;
const STATE_RETURN_SINK: i16 = 0x15;
const STATE_RETURN_START: i16 = 30;
const STATE_RETURN_HOME: i16 = 31;
const STATE_RETURN_FACE: i16 = 32;
const STATE_RESTORE: i16 = 33;
const STATE_MOTHER_SWIM_SETTLE: i16 = 0x28;
const STATE_MOTHER_SWIM_FLOAT: i16 = 0x29;
const STATE_PART_SEND: i16 = 0x6e;
const STATE_PART_FLY: i16 = 0x6f;
const STATE_PART_GROUND: i16 = 0x70;
const STATE_PART_SPIN: i16 = 0x71;
const STATE_PART_QUAKE_WAIT: i16 = 0x72;
const STATE_PART_LAND: i16 = 0x78;
const STATE_PART_BACK_START: i16 = 0x82;
const STATE_PART_HOME: i16 = 0x83;
const STATE_PART_FACE: i16 = 0x84;
const STATE_PART_WAIT: i16 = 0x85;
const STATE_PART_SWIM_SETTLE: i16 = 0x8c;
const STATE_PART_SWIM_FLOAT: i16 = 0x8d;

const HAMMER_NONE: u8 = 0;
const HAMMER_FALL: u8 = 1;
const HAMMER_SIDESWING: u8 = 2;

const PARENT_NONE: u8 = 0;
const PARENT_HOLD: u8 = 1;
const PARENT_SEND: u8 = 2;

const PARTS_ALL: u32 = 0x7FE;
const PART_BASE: u32 = 1;
const PART_EDGE: u32 = 0x100;

const PART_ARG_SHIFT: u32 = 0x14;
const PART_ARG_ROW: i16 = 8;
const PART_ARG_SKIP: u32 = 0x7FE0_0000;
const PART_ARG_COUNT: usize = 40;
const PART_ARG_COLS: i32 = 8;

const STTS_CULL: u32 = 0x0000_0100;
const STTS_SHOWMAP: u32 = 0x0000_0020;
const STTS_MAP_MASK: u32 = 0x0000_003F;
const STTS_MAP_BOARD: u32 = 0x38;
const STTS_PROFILE: u32 = 0x0004_4100;

const TACT_NONE: i32 = -1;
const TACT_QUAKE: i32 = 4;

const EYE_LIFT: f32 = 40.0;
const ATTENTION_LIFT: f32 = 60.0;
const ATTENTION_TALK: u8 = 5;
const ATTENTION_SPEAK: u8 = 6;

const WATER_GRAVITY: f32 = -3.0;
const WAVE_LIFT: f32 = 7.0;
const WAVE_DROP: f32 = 120.0;
const WAVE_STEP: f32 = 1000.0;
const WAVE_BASE: f32 = 147.0;
const WAVE_AMP: f32 = 2.0;
const WAVE_RISE: f32 = 30.0;
const WAVE_DAMP: f32 = 2.0;

const TARGET_MAX: usize = 10;
const SHOCK_DIST: f32 = 1000.0;
const SHOCK_TIMER: i16 = 100;
const SHOCK_JITTER: f32 = 10.0;
const SHOCK_RISE: f32 = 20.0;

const BACK_FAR: u32 = 0x6000;
const BACK_EDGE: u32 = 0x2000;
const TOSS_TURN: i16 = -0x4000;
const TOSS_FAST: f32 = 20.0;
const TOSS_RISE: f32 = 10.0;
const TOSS_SLOW: f32 = 3.0;
const TOSS_SLOW_RISE: f32 = 7.0;
const FALL_ANGLE: i16 = 0x4000;
const FALL_TURN: i16 = 0x1000;
const SPIN_TUMBLE: i16 = 0x800;
const SPIN_SNAP: i16 = 0x400;
const SPIN_XOR: i16 = 0xFF00u16 as i16;

const PART_TURN_JITTER: f32 = 8000.0;
const PART_FAST_JITTER: f32 = 2.0;
const PART_FAST_BASE: f32 = 10.0;
const PART_RISE_JITTER: f32 = 5.0;
const PART_RISE_BASE: f32 = 20.0;
const PART_HAMMER_JITTER: f32 = 5.0;
const PART_HAMMER_FAST: f32 = 15.0;
const PART_HAMMER_RISE: f32 = 30.0;
const PART_GRAVITY: f32 = -5.0;
const PART_SPIN: i16 = 0x1000;
const PART_GROUND_FAST: f32 = 5.0;
const PART_GROUND_RISE: f32 = 10.0;

const RETURN_JITTER: f32 = 30.0;
const RETURN_STEP: i16 = 0x1000;
const RETURN_TURN: i16 = 0x1000;
const RETURN_FACE: u32 = 0x100;
const RETURN_HEIGHT: f32 = 2.0;
const RETURN_SPAN: f32 = 2.0;
const RETURN_RATE: f32 = 10.0;

const SINK_TALL: f32 = 90.0;
const SINK_MID: f32 = 65.0;
const SINK_LOW: f32 = 20.0;
const SINK_STEP: f32 = 30.0;
const DROP_LIFT: f32 = 70.0;
const DROP_RISE: f32 = 140.0;

const CC_REST_HEIGHT: f32 = 105.0;
const CC_REST_RADIUS: f32 = 50.0;
const CC_GONE_HEIGHT: f32 = 0.0;
const CC_GONE_RADIUS: f32 = 0.0;
const CC_EDGE_HEIGHT: f32 = 25.0;
const CC_EDGE_RADIUS: f32 = 10.0;

const WALL_CREATE_X: f32 = 20.0;
const WALL_CREATE_Z: f32 = 30.0;
const WALL_STEP_X: f32 = 0.0;
const WALL_STEP_Z: f32 = 40.0;

const ROLL_DIST: f32 = 35.0;
const ROLL_REACH: f32 = 40.0;
const ROLL_FACE: f32 = -0.9;
const ROLL_Y_LOW: f32 = -100.0;
const ROLL_Y_HIGH: f32 = 200.0;

const SE_FALL_WATER: u32 = 0x6919;
const SE_BREAK_BOARD: u32 = 0x6937;

const MSG_NONE: i32 = -1;
const MSG_OPEN: i32 = 1;

const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0];
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0];

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const AT_TYPE_SWORD: u32 = 1 << 1;
const AT_TYPE_WATER: u32 = 1 << 8;
const AT_TYPE_SKULL_HAMMER: u32 = 1 << 16;
const AT_TYPE_UNK20000: u32 = 1 << 17;
const AT_TYPE_FIRE_ARROW: u32 = 1 << 18;
const AT_TYPE_ICE_ARROW: u32 = 1 << 19;
const AT_TYPE_LIGHT_ARROW: u32 = 1 << 20;
const AT_TYPE_WIND: u32 = 1 << 21;
const AT_TYPE_UNK400000: u32 = 1 << 22;
const AT_TYPE_LIGHT: u32 = 1 << 23;
const AT_TYPE_TG_BOARD: u32 =
    !(AT_TYPE_WATER | AT_TYPE_UNK20000 | AT_TYPE_WIND | AT_TYPE_UNK400000 | AT_TYPE_LIGHT);

const REACH_IDLE: f32 = 0.0;
const REACH_SWORD: f32 = 20.0;
const REACH_ARROW: f32 = 35.0;
const REACH_HAMMER: f32 = 50.0;
const REACH_HOOKSHOT: f32 = 5.0;
const REACH_BOOMERANG: f32 = 90.0;

const CUT_NONE: u8 = 0x00;
const CUT_A: u8 = 0x01;
const CUT_F: u8 = 0x02;
const CUT_R: u8 = 0x03;
const CUT_L: u8 = 0x04;
const CUT_EA: u8 = 0x06;
const CUT_EB: u8 = 0x07;
const CUT_HAMMER_SIDESWING: u8 = 0x11;
const CUT_HAMMER_FRONTSWING: u8 = 0x12;

const CUT_SKIP: i16 = 255;

const PL_CUT_REAL: [u8; 32] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F,
];

const PL_CUT_NO: [i16; 32] = [
    255, 0, 4, 3, 3, 1, 0, 4, 4, 4, 1, 3, 1, 3, 1, 1, 1, 3, 1, 1, 0, 1, 0, 1, 0, 1, 1, 4, 0, 1, 1, 1,
];

const CUT_PARTS_ARG: [u32; PART_ARG_COUNT] = [
    0x02000000, 0x04000000, 0x00800000, 0x01000000, 0x7FE00000, 0x7FE00000, 0x7FE00000, 0x7FE00000,
    0x04000000, 0x00800000, 0x01000000, 0x10000000, 0x40000000, 0x7FE00000, 0x7FE00000, 0x7FE00000,
    0x00400000, 0x00200000, 0x02000000, 0x04000000, 0x7FE00000, 0x7FE00000, 0x7FE00000, 0x7FE00000,
    0x00200000, 0x02000000, 0x04000000, 0x00800000, 0x7FE00000, 0x7FE00000, 0x7FE00000, 0x7FE00000,
    0x08000000, 0x00400000, 0x00200000, 0x02000000, 0x04000000, 0x00800000, 0x01000000, 0x10000000,
];

const REG8_S1: i32 = 0;
const REG8_S4: i16 = 0;
const REG8_F6: f32 = 0.0;
const REG8_F7: f32 = 0.0;
const REG8_F8: f32 = 0.0;
const REG8_F9: f32 = 0.0;

#[derive(Resource, Debug, Clone, Copy)]
pub struct Statics {
    pub msg_id: i32,
    pub event_reset: bool,
}

impl Default for Statics {
    fn default() -> Self {
        Self {
            msg_id: MSG_NONE,
            event_reset: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Blow {
    Idle,
    Sword,
    Hammer,
    Arrow,
    FireArrow,
    IceArrow,
    LightArrow,
    Boomerang,
    Hookshot,
}

impl Blow {
    pub fn at_type(self) -> u32 {
        match self {
            Self::Idle | Self::Arrow | Self::Boomerang | Self::Hookshot => 0,
            Self::Sword => AT_TYPE_SWORD,
            Self::Hammer => AT_TYPE_SKULL_HAMMER,
            Self::FireArrow => AT_TYPE_FIRE_ARROW,
            Self::IceArrow => AT_TYPE_ICE_ARROW,
            Self::LightArrow => AT_TYPE_LIGHT_ARROW,
        }
    }

    pub fn reach(self) -> f32 {
        match self {
            Self::Idle => REACH_IDLE,
            Self::Sword => REACH_SWORD,
            Self::Arrow | Self::FireArrow | Self::IceArrow | Self::LightArrow => REACH_ARROW,
            Self::Hammer => REACH_HAMMER,
            Self::Boomerang => REACH_BOOMERANG,
            Self::Hookshot => REACH_HOOKSHOT,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cyl {
    pub centre: Vec3,
    pub radius: f32,
    pub height: f32,
    pub co_set: bool,
    pub tg_set: bool,
    pub tg_hit: bool,
    pub at_type: u32,
}

impl Cyl {
    const SRC: Cyl = Cyl {
        centre: Vec3::ZERO,
        radius: 0.0,
        height: 0.0,
        co_set: true,
        tg_set: true,
        tg_hit: false,
        at_type: 0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Link {
    pub parts: u32,
    pub hammer_mode: u8,
    pub parent_state: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Kanban {
    pub whole: bool,
    pub parts: u32,
    pub steps: u8,
    pub hammer_mode: u8,
    pub parent_state: u8,
    pub wave_done: u8,
    pub wave: f32,
    pub cc_height: f32,
    pub cc_radius: f32,
    pub ground_angle: [i16; 3],
    pub timer: [i16; TIMER_COUNT],
    pub wave_phase: i16,
    pub mode: i16,
    pub state: i16,
    pub cut_seen: i16,
    pub cut_point: i16,
    pub tumble: i16,
    pub parent: Option<Entity>,
    pub drop_pos: Vec3,
    pub spawn_pos: Vec3,
    pub drift_x: f32,
    pub drift_y: f32,
    pub drift_z: f32,
    pub home_angle: [i16; 3],
    pub home_pos: Vec3,
    pub drop_copy: Vec3,
    pub status: u32,
    pub speed_f: f32,
    pub speed: Vec3,
    pub gravity: f32,
    pub shape_angle: [i16; 3],
    pub eye_pos: Vec3,
    pub attention_pos: Vec3,
    pub attention_talk: u8,
    pub attention_speak: u8,
    pub ripple_pos: Vec3,
    pub ripple: bool,
    pub cyl: Cyl,
    pub wall_x: f32,
    pub wall_z: f32,
    pub wall_hit: bool,
    pub cantalk: bool,
    pub crash: bool,
    pub seed: [i32; 3],
    pub health: i32,
    pub max_health: i32,
    pub entered: bool,
    pub doomed: bool,
}

impl Kanban {
    pub fn spawn(parameters: u32) -> Self {
        let parts = parameters >> PARAM_PARTS_SHIFT;
        Self {
            whole: parts == 0,
            parts,
            steps: 0,
            hammer_mode: HAMMER_NONE,
            parent_state: PARENT_NONE,
            wave_done: 0,
            wave: 0.0,
            cc_height: CC_GONE_HEIGHT,
            cc_radius: CC_GONE_RADIUS,
            ground_angle: [0; 3],
            timer: [0; TIMER_COUNT],
            wave_phase: 0,
            mode: MODE_MOTHER,
            state: STATE_TALK,
            cut_seen: 0,
            cut_point: 0,
            tumble: 0,
            parent: None,
            drop_pos: Vec3::ZERO,
            spawn_pos: Vec3::ZERO,
            drift_x: 0.0,
            drift_y: 0.0,
            drift_z: 0.0,
            home_angle: [0; 3],
            home_pos: Vec3::ZERO,
            drop_copy: Vec3::ZERO,
            status: STTS_PROFILE,
            speed_f: 0.0,
            speed: Vec3::ZERO,
            gravity: 0.0,
            shape_angle: [0; 3],
            eye_pos: Vec3::ZERO,
            attention_pos: Vec3::ZERO,
            attention_talk: ATTENTION_TALK,
            attention_speak: ATTENTION_SPEAK,
            ripple_pos: Vec3::ZERO,
            ripple: false,
            cyl: Cyl::SRC,
            wall_x: WALL_CREATE_X,
            wall_z: WALL_CREATE_Z,
            wall_hit: false,
            cantalk: false,
            crash: false,
            seed: [RND_SEED; 3],
            health: 0,
            max_health: 0,
            entered: false,
            doomed: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        (self.status & STTS_CULL) == 0
    }

    pub fn link(&self) -> Link {
        Link {
            parts: self.parts,
            hammer_mode: self.hammer_mode,
            parent_state: self.parent_state,
        }
    }

    pub fn create(&mut self, actor: &mut Actor, parent: Option<(Entity, Vec3, [i16; 3])>) -> bool {
        self.parts = actor.parameters >> PARAM_PARTS_SHIFT;
        self.shape_angle = actor.angle;
        self.spawn_pos = actor.pos;
        self.home_angle = actor.angle;
        self.home_pos = actor.pos;
        self.whole = self.parts == 0;

        if self.whole {
            self.status = (self.status & !STTS_MAP_MASK) | STTS_SHOWMAP | STTS_MAP_BOARD;
            self.cc_height = CC_REST_HEIGHT;
            self.cc_radius = CC_REST_RADIUS;
            self.parts = PART_BASE;
        } else {
            self.cut_point = actor.angle[2];
            actor.angle[2] = 0;
            self.shape_angle[2] = 0;
        }

        self.wall_x = WALL_CREATE_X;
        self.wall_z = WALL_CREATE_Z;
        self.eye_pos = Vec3::new(actor.pos.x, actor.pos.y + EYE_LIFT, actor.pos.z);
        self.attention_pos = Vec3::new(
            self.eye_pos.x,
            self.eye_pos.y + ATTENTION_LIFT,
            self.eye_pos.z,
        );
        self.attention_talk = ATTENTION_TALK;
        self.attention_speak = ATTENTION_SPEAK;
        self.cyl = Cyl::SRC;
        self.entered = true;

        if self.whole {
            self.mode = MODE_MOTHER;
            self.state = STATE_TALK;
            return true;
        }

        let Some((entity, home, angle)) = parent else {
            return false;
        };
        self.parent = Some(entity);
        self.home_pos = home;
        self.home_angle = angle;
        actor.pos = self.home_pos;
        actor.pos.y = self.home_pos.y - DROP_LIFT;
        self.drop_pos.y = DROP_LIFT;
        self.drift_y = DROP_RISE;
        self.drop_copy = self.drop_pos;
        self.mode = MODE_PART;
        self.state = STATE_PART_SEND;
        true
    }

    fn rnd(&mut self) -> f32 {
        for lane in 0..3 {
            self.seed[lane] = (self.seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
        }
        let sum = (self.seed[0]) as f32 / RND_SCALE[0]
            + (self.seed[1]) as f32 / RND_SCALE[1]
            + (self.seed[2]) as f32 / RND_SCALE[2];
        (sum % 1.0).abs()
    }

    fn rnd_f(&mut self, range: f32) -> f32 {
        self.rnd() * range
    }

    fn rnd_fx(&mut self, range: f32) -> f32 {
        range * (self.rnd() - 0.5) * 2.0
    }
}

fn add_calc(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value != target {
        let mut step = scale * (target - *value);
        if step > max_step {
            step = max_step;
        } else if step < -max_step {
            step = -max_step;
        }
        *value += step;
    }
}

fn add_calc0(value: &mut f32, scale: f32, max_step: f32) {
    let mut step = *value * scale;
    if step > max_step {
        step = max_step;
    } else if step < -max_step {
        step = -max_step;
    }
    *value -= step;
}

fn flat(from: Vec3, to: Vec3) -> f32 {
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    (dx * dx + dz * dz).sqrt()
}

fn bearing(from: Vec3, to: Vec3) -> i16 {
    to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn turn_gap(a: i16, b: i16) -> u32 {
    i32::from(a.wrapping_sub(b)).unsigned_abs()
}

fn cyl_touches(centre: Vec3, radius: f32, height: f32, at: Vec3, reach: f32) -> bool {
    let dx = at.x - centre.x;
    let dz = at.z - centre.z;
    let span = radius + reach;
    if span * span > dx * dx + dz * dz {
        return false;
    }
    let top = centre.y + height;
    let near = if at.y < centre.y {
        centre.y
    } else if at.y > top {
        top
    } else {
        at.y
    };
    (at.y - near).abs() <= reach
}

fn cut_index(cut: u8) -> Option<i16> {
    let mut index = 0usize;
    while index < PL_CUT_REAL.len() {
        if PL_CUT_REAL[index] == cut {
            break;
        }
        index += 1;
    }
    if index >= PL_CUT_REAL.len() {
        return None;
    }
    let point = PL_CUT_NO.get(index).copied().unwrap_or(CUT_SKIP);
    if point == CUT_SKIP {
        None
    } else {
        Some(point)
    }
}

pub fn is_shock_bomb(name: &str, life: bomb::Life) -> bool {
    if name != bomb::BOMB_ACTORS[0] && name != bomb::BOMB_ACTORS[1] {
        return false;
    }
    life == bomb::Life::Spent
}

fn quake_top(player_pos: Vec3, player_angle: i16) -> Vec3 {
    let (sin, cos) = from_s16(player_angle).sin_cos();
    Vec3::new(
        player_pos.x + REACH_HAMMER * sin,
        player_pos.y,
        player_pos.z + REACH_HAMMER * cos,
    )
}

struct Spawn {
    actor: Actor,
    board: Kanban,
}

#[derive(Clone, Copy)]
struct Env<'a> {
    solid: Option<&'a Collision>,
    sea: Option<&'a Sea>,
    water: Option<&'a Water>,
    player_pos: Vec3,
    player_angle: i16,
    player_cut: u8,
    conduct: bool,
    quake: bool,
    quake_top: Vec3,
    targets: &'a [Vec3],
    blow: Blow,
    tact: i32,
    talk: bool,
    msg_live: bool,
    msg_status: u8,
}

impl Env<'_> {
    fn sea_at(&self, x: f32, z: f32) -> (bool, f32) {
        water::probe(self.sea, x, z)
    }

    fn water_at(&self) -> (bool, f32) {
        match self.water {
            Some(water) => (water.in_area, water.surface),
            None => (false, 0.0),
        }
    }

    fn grounded(&self, pos: Vec3) -> bool {
        ground::grounded(self.solid, pos)
    }
}

struct Sink<'a> {
    events: &'a mut SimEvents,
    statics: &'a mut Statics,
    spawns: &'a mut Vec<Spawn>,
    msg: &'a mut MsgBox,
    table: Option<&'a MsgTable>,
    msg_no: u32,
    set_id: u16,
    room: i8,
}

impl Sink<'_> {
    fn se(&mut self, id: u32) {
        self.events.list.push(SimEvent::Se(id));
    }

    fn child(
        &mut self,
        entity: Entity,
        home: Vec3,
        home_angle: [i16; 3],
        angle: [i16; 3],
        parameters: u32,
        pos: Vec3,
    ) {
        let mut actor = Actor {
            name: KANBAN_ACTORS[0].to_owned(),
            parameters,
            pos,
            angle,
            set_id: self.set_id,
            room: self.room,
        };
        let mut board = Kanban::spawn(parameters);
        if !board.create(&mut actor, Some((entity, home, home_angle))) {
            return;
        }
        self.spawns.push(Spawn { actor, board });
    }

    fn msg_open(&mut self) {
        if let Some(table) = self.table {
            self.msg.message_set(self.msg_no, table);
        }
        self.statics.msg_id = MSG_OPEN;
    }
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    sea: Option<Res<'w, Sea>>,
    water: Option<Res<'w, Water>>,
    combat: Res<'w, Combat>,
    hammer: Res<'w, hammer::Cut>,
    do_intent: Res<'w, DoIntent>,
    status: Res<'w, PlayerStatus>,
    table: Option<Res<'w, MsgTable>>,
    players: Query<'w, 's, (Entity, &'static Player)>,
    shells: Query<'w, 's, (&'static Actor, &'static bomb::Bomb), Without<Kanban>>,
    shafts: Query<'w, 's, &'static arrow::Arrow>,
    bolts: Query<'w, 's, &'static boomerang::Boomerang>,
    chains: Query<'w, 's, &'static hookshot::Hookshot>,
}

pub fn control(
    watch: Watch<'_, '_>,
    mut statics: ResMut<Statics>,
    mut events: ResMut<SimEvents>,
    mut msg: ResMut<MsgBox>,
    mut commands: Commands,
    mut boards: Query<(Entity, &mut Actor, &mut Kanban), (With<Kanban>, Without<Player>)>,
) {
    let Some((_, player)) = watch.players.iter().next() else {
        return;
    };
    let solid = watch.collision.as_deref();
    let sea = watch.sea.as_deref();
    let wet = watch.water.as_deref();
    let msg_status = msg.status();
    let msg_live = msg.active();
    let talk = watch.do_intent.0;
    let conduct = watch.status.check1(STTS1_WIND_WAKER_CONDUCT);
    let blow = blow_of(&watch);
    let quake = watch.hammer.kind == hammer::CUT_HAMMER_FRONTSWING;
    let player_pos = player.pos;
    let player_angle = to_s16(player.face);
    let player_cut = player_cut_of(&watch);
    let top = quake_top(player_pos, player_angle);
    let targets: Vec<Vec3> = watch
        .shells
        .iter()
        .filter(|(actor, shell)| is_shock_bomb(&actor.name, shell.life))
        .take(TARGET_MAX)
        .map(|(actor, _)| actor.pos)
        .collect();

    let base = Env {
        solid,
        sea,
        water: wet,
        player_pos,
        player_angle,
        player_cut,
        conduct,
        quake,
        quake_top: top,
        targets: &targets,
        blow,
        tact: TACT_NONE,
        talk,
        msg_live,
        msg_status,
    };

    let mut issued: Vec<Spawn> = Vec::new();
    let mut links: Vec<(Entity, Link)> = Vec::new();
    let mut writes: Vec<(Entity, Link)> = Vec::new();
    let mut dropped: Vec<Entity> = Vec::new();

    for (_, actor, board) in &mut boards {
        if board.entered {
            continue;
        }
        let mut fresh = Kanban::spawn(actor.parameters);
        if !fresh.create(actor, None) {
            fresh.doomed = true;
        }
        *board = fresh;
    }

    for (entity, actor, board) in &mut boards {
        if board.doomed || !board.whole {
            continue;
        }
        let env = base;
        let mut sink = Sink {
            events: &mut events,
            statics: &mut statics,
            spawns: &mut issued,
            msg: &mut msg,
            table: watch.table.as_deref(),
            msg_no: actor.parameters,
            set_id: actor.set_id,
            room: actor.room,
        };
        board.execute(actor, entity, &env, &mut sink);
        links.push((entity, board.link()));
    }

    for (entity, actor, board) in &mut boards {
        if board.doomed || board.whole {
            continue;
        }
        let Some(parent) = board.parent else {
            dropped.push(entity);
            continue;
        };
        let Some(live) = links.iter().find(|entry| entry.0 == parent) else {
            dropped.push(entity);
            continue;
        };
        let env = base;
        let mut sink = Sink {
            events: &mut events,
            statics: &mut statics,
            spawns: &mut issued,
            msg: &mut msg,
            table: watch.table.as_deref(),
            msg_no: actor.parameters,
            set_id: actor.set_id,
            room: actor.room,
        };
        let next = board.execute_part(actor, &env, &mut sink, &live.1);
        writes.push((parent, next));
        if board.doomed {
            dropped.push(entity);
        }
    }

    for (entity, _, board) in &mut boards {
        if board.doomed || !board.whole {
            continue;
        }
        if let Some(link) = writes.iter().find(|entry| entry.0 == entity) {
            board.parts = link.1.parts;
            board.hammer_mode = link.1.hammer_mode;
            board.parent_state = link.1.parent_state;
        }
    }

    for spawn in issued {
        commands.spawn((spawn.actor, spawn.board));
    }
    for entity in dropped {
        statics.msg_id = MSG_NONE;
        statics.event_reset = true;
        let _ = commands.entity(entity).despawn();
    }
}

fn blow_of(watch: &Watch<'_, '_>) -> Blow {
    if watch
        .chains
        .iter()
        .any(|chain| chain.phase != hookshot::Phase::Wait)
    {
        return Blow::Hookshot;
    }
    for shaft in watch.shafts.iter() {
        if shaft.phase == arrow::Phase::Flying {
            return match shaft.kind {
                arrow::TYPE_FIRE => Blow::FireArrow,
                arrow::TYPE_ICE => Blow::IceArrow,
                arrow::TYPE_LIGHT => Blow::LightArrow,
                _ => Blow::Arrow,
            };
        }
    }
    if watch
        .bolts
        .iter()
        .any(|bolt| bolt.parameter == boomerang::MODE_THROW)
    {
        return Blow::Boomerang;
    }
    match watch.hammer.kind {
        hammer::CUT_HAMMER_SIDESWING | hammer::CUT_HAMMER_FRONTSWING => Blow::Hammer,
        _ => Blow::Idle,
    }
}

fn player_cut_of(watch: &Watch<'_, '_>) -> u8 {
    if watch.hammer.kind == hammer::CUT_HAMMER_SIDESWING {
        return CUT_HAMMER_SIDESWING;
    }
    if watch.hammer.kind == hammer::CUT_HAMMER_FRONTSWING {
        return CUT_HAMMER_FRONTSWING;
    }
    if !watch.combat.active || watch.combat.frame < 1 || watch.combat.frame > 5 {
        return CUT_NONE;
    }
    match watch.combat.kind {
        CutKind::A => CUT_A,
        CutKind::F => CUT_F,
        CutKind::R => CUT_R,
        CutKind::L => CUT_L,
        CutKind::EA => CUT_EA,
        CutKind::EB => CUT_EB,
    }
}

impl Kanban {
    fn timers(&mut self) {
        for index in 0..TIMER_COUNT {
            if self.timer[index] != 0 {
                self.timer[index] -= 1;
            }
        }
    }

    fn execute(&mut self, actor: &mut Actor, entity: Entity, env: &Env<'_>, sink: &mut Sink<'_>) {
        self.timers();
        match self.mode {
            MODE_MOTHER => {
                if env.talk {
                    self.talk_step(env, sink);
                } else {
                    self.cantalk = self.cut_seen == 0
                        && self.state != STATE_RETURN_TURN
                        && self.state != STATE_RETURN_SINK;
                    self.mother_move(actor, entity, env, sink);
                }
                if self.state != STATE_QUAKE_WAIT {
                    self.roll_player(actor, env);
                }
                self.register_cyl(actor, env);
                if self.parts != PART_BASE && self.parts != PARTS_ALL && env.tact == TACT_QUAKE {
                    self.ripple = false;
                    self.gravity = 0.0;
                    self.speed_f = 0.0;
                    self.speed = Vec3::ZERO;
                    self.mode = MODE_MOTHER_RETURN;
                    self.state = STATE_RETURN_START;
                }
            }
            MODE_MOTHER_RETURN => self.mother_return_move(actor, env),
            MODE_MOTHER_WATER => {
                self.mother_water_swim(actor, env, sink);
                if env.tact == TACT_QUAKE {
                    self.ripple = false;
                    self.gravity = 0.0;
                    self.speed_f = 0.0;
                    self.speed = Vec3::ZERO;
                    self.mode = MODE_MOTHER_RETURN;
                    self.state = STATE_RETURN_START;
                }
            }
            _ => {}
        }
        self.tail(actor, env);
    }

    fn execute_part(
        &mut self,
        actor: &mut Actor,
        env: &Env<'_>,
        sink: &mut Sink<'_>,
        live: &Link,
    ) -> Link {
        self.timers();
        let mut next = *live;
        match self.mode {
            MODE_PART => {
                self.parts_move(actor, live, env);
                if env.tact == TACT_QUAKE {
                    self.ripple = false;
                    self.gravity = 0.0;
                    self.speed_f = 0.0;
                    self.speed = Vec3::ZERO;
                    self.mode = MODE_PART_BACK;
                    self.state = STATE_PART_BACK_START;
                } else if self.sea_water_check(actor, env, sink) {
                    self.mode = MODE_PART_WATER;
                    self.state = STATE_PART_SWIM_SETTLE;
                }
            }
            MODE_PART_BACK => {
                self.chield_parts_move(actor, live, env, &mut next);
            }
            MODE_PART_WATER => {
                self.chield_water_swim(actor, env, sink);
                if env.tact == TACT_QUAKE {
                    self.ripple = false;
                    self.gravity = 0.0;
                    self.speed_f = 0.0;
                    self.speed = Vec3::ZERO;
                    self.mode = MODE_PART_BACK;
                    self.state = STATE_PART_BACK_START;
                }
            }
            _ => {}
        }
        self.tail(actor, env);
        next
    }

    fn tail(&mut self, actor: &mut Actor, env: &Env<'_>) {
        let (sin, cos) = from_s16(actor.angle[1]).sin_cos();
        self.speed.x = self.speed_f * sin;
        self.speed.z = self.speed_f * cos;
        self.speed.y += self.gravity;
        let from = actor.pos;
        let moved = from + self.speed;
        let push = if self.cc_height != 0.0 {
            match env.solid {
                Some(solid) => {
                    let landed = solid.slide(from, moved);
                    self.wall_hit = landed != moved;
                    landed - moved
                }
                None => {
                    self.wall_hit = false;
                    Vec3::ZERO
                }
            }
        } else {
            self.wall_hit = false;
            Vec3::ZERO
        };
        actor.pos = moved + push;
        self.wall_x = WALL_STEP_X;
        self.wall_z = WALL_STEP_Z;
        if self.mode != MODE_MOTHER_RETURN
            && self.mode != MODE_PART_BACK
            && self.timer[TIMER_SINK] == 0
        {
            self.crr_pos(actor, env);
        }
    }

    fn crr_pos(&self, actor: &mut Actor, env: &Env<'_>) {
        if let Some(floor) = ground::surface(env.solid, actor.pos) {
            actor.pos.y = floor - self.drift_y + self.wave;
        }
    }

    fn register_cyl(&mut self, actor: &Actor, env: &Env<'_>) {
        self.cyl.centre = actor.pos;
        self.cyl.radius = self.cc_radius;
        self.cyl.height = self.cc_height;
        let at = env.blow.at_type();
        if !self.cyl.tg_set || (AT_TYPE_TG_BOARD & at) == 0 {
            self.cyl.tg_hit = false;
            return;
        }
        if cyl_touches(
            self.cyl.centre,
            self.cyl.radius,
            self.cyl.height,
            env.player_pos,
            env.blow.reach(),
        ) {
            self.cyl.tg_hit = true;
            self.cyl.at_type = at;
        } else {
            self.cyl.tg_hit = false;
        }
    }

    fn roll_player(&mut self, actor: &Actor, env: &Env<'_>) {
        let span = ROLL_DIST + ROLL_REACH;
        let gap = flat(actor.pos, env.player_pos);
        if gap * gap >= span * span {
            return;
        }
        let dy = env.player_pos.y - actor.pos.y;
        if dy <= ROLL_Y_LOW || dy >= ROLL_Y_HIGH {
            return;
        }
        let to_self = bearing(env.player_pos, actor.pos);
        if from_s16(env.player_angle.wrapping_sub(to_self)).cos() < ROLL_FACE {
            self.crash = true;
        }
    }

    fn talk_step(&mut self, env: &Env<'_>, sink: &mut Sink<'_>) {
        if sink.statics.msg_id == MSG_NONE {
            if !env.msg_live {
                sink.msg_open();
            }
        } else if env.msg_live && env.msg_status == MSG_BOX_CLOSED {
            sink.msg.set_status(MSG_DESTROYED);
            sink.statics.event_reset = true;
            sink.statics.msg_id = MSG_NONE;
        }
    }

    fn shibuki_set(&mut self, sink: &mut Sink<'_>) {
        self.ripple = false;
        sink.se(SE_FALL_WATER);
    }

    fn sea_water_check(&mut self, actor: &mut Actor, env: &Env<'_>, sink: &mut Sink<'_>) -> bool {
        if REG8_S1 != 0 {
            return false;
        }
        self.ripple_pos = actor.pos;
        self.gravity = WATER_GRAVITY;
        let mut code = 0u8;
        let (in_sea, sea_y) = env.sea_at(actor.pos.x, actor.pos.z);
        if in_sea {
            self.ripple_pos.y = sea_y;
            if self.cut_point == 1 {
                self.wave = WAVE_LIFT;
            }
            if actor.pos.y < (sea_y - WAVE_DROP) + REG8_F6 {
                self.gravity = 0.0;
                self.speed_f = 0.0;
                self.speed = Vec3::ZERO;
                self.wave_phase = self.wave_phase.wrapping_add((REG8_F7 + WAVE_STEP) as i16);
                let crest =
                    (REG8_F8 + WAVE_BASE) + (REG8_F9 + WAVE_AMP) * from_s16(self.wave_phase).sin();
                add_calc(&mut actor.pos.y, sea_y - crest, 1.0, WAVE_RISE);
                code = 1;
            }
        } else {
            let (in_water, water_y) = env.water_at();
            if in_water {
                code = 2;
                self.ripple_pos.y = water_y;
                if self.cut_point == 1 {
                    self.wave = WAVE_LIFT;
                }
                self.gravity = WATER_GRAVITY;
                if actor.pos.y < water_y {
                    self.gravity = 0.0;
                    self.speed_f = 0.0;
                    self.speed = Vec3::ZERO;
                    self.wave_phase = self.wave_phase.wrapping_add((REG8_F7 + WAVE_STEP) as i16);
                    let crest = (REG8_F8 + WAVE_BASE)
                        + (REG8_F9 + WAVE_AMP) * from_s16(self.wave_phase).sin();
                    add_calc(&mut actor.pos.y, water_y - crest, 1.0, WAVE_RISE);
                    code = 1;
                }
            }
        }
        if code != 0 {
            if code == 1 && self.wave_done == 0 {
                self.wave_phase = 0;
                self.wave_done = 1;
                self.shibuki_set(sink);
            }
            return true;
        }
        self.wave = 0.0;
        false
    }

    fn shock_damage_check(&mut self, env: &Env<'_>, here: Vec3) -> bool {
        for target in env.targets {
            if flat(*target, here) < SHOCK_DIST {
                self.timer[TIMER_SHOCK] = SHOCK_TIMER.wrapping_add(REG8_S4);
                return true;
            }
        }
        if env.quake && flat(env.quake_top, here) < SHOCK_DIST {
            return true;
        }
        false
    }

    fn cut_point_check(
        &mut self,
        actor: &mut Actor,
        entity: Entity,
        env: &Env<'_>,
        sink: &mut Sink<'_>,
    ) {
        if self.cut_point == 0 {
            let to_player = bearing(actor.pos, env.player_pos);
            let gap = self.shape_angle[1].wrapping_sub(to_player);
            let away = i32::from(gap).unsigned_abs();
            if away > BACK_FAR {
                self.cut_point = 2;
            } else if away > BACK_EDGE && away < BACK_FAR {
                if to_player < 0 {
                    self.cut_point = 2;
                }
            }
        }
        let shift = i32::from(self.cut_point);
        let seen = i32::from(self.cut_seen);
        if (seen >> shift) & 1 != 0 || (seen & 2) != 0 {
            return;
        }
        self.cut_seen |= 1i16.wrapping_shl(shift as u32);
        if (self.parts & PART_BASE) != 0 {
            self.parts = PARTS_ALL;
            self.parent_state = PARENT_NONE;
        }
        let row = shift * i32::from(PART_ARG_ROW);
        let mut carried = self.parts << PART_ARG_SHIFT;
        let mut parameters = 0u32;
        for column in 0..PART_ARG_COLS {
            let slot = usize::try_from(row + column).unwrap_or(usize::MAX);
            let arg = CUT_PARTS_ARG.get(slot).copied().unwrap_or(0);
            if arg != PART_ARG_SKIP && (carried & arg) != 0 {
                carried ^= arg;
                parameters |= arg;
            }
        }
        if parameters == 0 {
            return;
        }
        actor.angle[2] = shift as i16;
        let pos = actor.pos;
        let angle = actor.angle;
        let home = self.home_pos;
        let home_angle = self.home_angle;
        sink.child(entity, home, home_angle, angle, parameters, pos);
        sink.se(SE_BREAK_BOARD);
        self.parts = carried >> PART_ARG_SHIFT;
        if shift == 1 {
            self.steps = 0;
            if (i32::from(self.cut_seen) & 0x10) != 0 {
                self.steps = 1;
            }
            actor.pos.y = self.home_pos.y - DROP_LIFT;
            self.drop_pos.y = DROP_LIFT;
            self.drift_y = DROP_RISE;
            self.cc_height = CC_GONE_HEIGHT;
            self.cc_radius = CC_GONE_RADIUS;
            self.state = STATE_TOSS;
        }
        if (i32::from(self.cut_seen) & 5) == 5 || shift == 4 {
            self.cc_height = CC_EDGE_HEIGHT;
            self.cc_radius = CC_EDGE_RADIUS;
        }
        actor.angle[2] = 0;
    }

    fn mother_move(
        &mut self,
        actor: &mut Actor,
        entity: Entity,
        env: &Env<'_>,
        sink: &mut Sink<'_>,
    ) {
        match self.state {
            STATE_TALK => {
                if !self.cyl.tg_hit {
                    return;
                }
                let at = self.cyl.at_type;
                self.cyl.tg_hit = false;
                self.status &= !STTS_MAP_MASK;
                if at == AT_TYPE_SKULL_HAMMER {
                    self.hammer_mode = HAMMER_SIDESWING;
                    if env.player_cut != CUT_HAMMER_SIDESWING {
                        self.parent_state = PARENT_NONE;
                        self.cc_height = CC_GONE_HEIGHT;
                        self.cc_radius = CC_GONE_RADIUS;
                        self.hammer_mode = HAMMER_FALL;
                        self.state = STATE_RETURN_TURN;
                        return;
                    }
                    if let Some(point) = cut_index(env.player_cut) {
                        self.cut_point = point;
                        self.cut_point_check(actor, entity, env, sink);
                    }
                } else if at == AT_TYPE_SWORD {
                    if let Some(point) = cut_index(env.player_cut) {
                        self.cut_point = point;
                        self.cut_point_check(actor, entity, env, sink);
                    }
                } else {
                    self.cut_point = 4;
                    self.cut_point_check(actor, entity, env, sink);
                }
            }
            STATE_TOSS => {
                actor.angle[1] = env.player_angle.wrapping_add(TOSS_TURN);
                self.shape_angle[1] = actor.angle[1];
                self.steps = 0;
                if (self.parts & PART_EDGE) != 0 {
                    self.speed_f = TOSS_FAST;
                    self.speed.y = TOSS_RISE;
                    self.gravity = WATER_GRAVITY;
                } else {
                    self.speed_f = TOSS_SLOW;
                    self.speed.y = TOSS_SLOW_RISE;
                    self.gravity = WATER_GRAVITY;
                }
                self.state = self.state.wrapping_add(1);
            }
            STATE_FALL => {
                if self.sea_water_check(actor, env, sink) {
                    self.mode = MODE_MOTHER_WATER;
                    self.state = STATE_MOTHER_SWIM_SETTLE;
                } else {
                    if self.wall_hit {
                        self.speed_f *= -1.0;
                    }
                    if self.speed.y < 0.0 {
                        add_calc_angle_s2(&mut self.shape_angle[0], FALL_ANGLE, 1, FALL_TURN);
                    }
                    if env.grounded(actor.pos) {
                        self.tumble = SPIN_TUMBLE;
                        self.read_ground_angle(actor, env);
                        self.speed_f = 0.0;
                        self.gravity = 0.0;
                        self.state = STATE_BOUNCE;
                    }
                }
            }
            STATE_BOUNCE => {
                if self.sea_water_check(actor, env, sink) {
                    self.mode = MODE_MOTHER_WATER;
                    self.state = STATE_MOTHER_SWIM_SETTLE;
                    return;
                }
                add_calc_angle_s2(
                    &mut self.shape_angle[0],
                    self.tumble.wrapping_add(FALL_ANGLE),
                    1,
                    FALL_TURN,
                );
                self.tumble ^= SPIN_XOR;
                self.steps = self.steps.wrapping_add(1);
                self.steps &= 3;
                if (self.steps & 1) == 0 {
                    self.tumble = SPIN_SNAP;
                }
                if (self.steps & 3) == 0 {
                    self.shape_angle[0] = FALL_ANGLE;
                    self.tumble = 0;
                    self.cyl.co_set = false;
                    self.cyl.tg_set = false;
                    self.cyl.tg_hit = false;
                    self.state = self.state.wrapping_add(1);
                }
                if self.sea_water_check(actor, env, sink) {
                    self.mode = MODE_MOTHER_WATER;
                    self.state = STATE_MOTHER_SWIM_SETTLE;
                } else if self.timer[TIMER_SHOCK] == 0
                    && self.shock_damage_check(env, actor.pos)
                {
                    let kick = self.rnd_f(SHOCK_JITTER) + SHOCK_RISE;
                    self.speed.y = kick;
                    self.gravity = WATER_GRAVITY;
                    self.state = STATE_FALL;
                }
            }
            STATE_QUAKE_WAIT => {
                if self.sea_water_check(actor, env, sink) {
                    self.mode = MODE_MOTHER_WATER;
                    self.state = STATE_MOTHER_SWIM_SETTLE;
                } else if self.timer[TIMER_SHOCK] == 0
                    && self.shock_damage_check(env, actor.pos)
                {
                    let kick = self.rnd_f(SHOCK_JITTER) + SHOCK_RISE;
                    self.speed.y = kick;
                    self.gravity = WATER_GRAVITY;
                    self.state = STATE_FALL;
                }
            }
            STATE_RETURN_TURN => {
                self.state = self.state.wrapping_add(1);
                self.timer[TIMER_SINK] = 2;
                self.sink_move(actor);
            }
            STATE_RETURN_SINK => {
                self.timer[TIMER_SINK] = 2;
                self.sink_move(actor);
            }
            _ => {}
        }
    }

    fn sink_move(&mut self, actor: &mut Actor) {
        let mut height = SINK_TALL;
        if (i32::from(self.cut_seen) & 8) != 0 {
            height = SINK_MID;
        }
        if (i32::from(self.cut_seen) & 0x10) != 0 {
            height = SINK_LOW;
        }
        add_calc(&mut actor.pos.y, self.home_pos.y - height, 1.0, SINK_STEP);
    }

    fn read_ground_angle(&mut self, actor: &Actor, env: &Env<'_>) {
        let normal = ground::slope_normal(env.solid, actor.pos);
        self.ground_angle[0] = to_s16(-(normal.y.atan2(normal.z)));
        self.ground_angle[2] = to_s16(normal.y.atan2(normal.x));
    }

    fn mother_water_swim(&mut self, actor: &mut Actor, env: &Env<'_>, sink: &mut Sink<'_>) {
        match self.state {
            STATE_MOTHER_SWIM_SETTLE => {
                self.tumble = 0;
                self.shape_angle[0] = FALL_ANGLE;
                self.cyl.co_set = false;
                self.cyl.tg_set = false;
                self.cyl.tg_hit = false;
                self.state = self.state.wrapping_add(1);
                self.sea_water_check(actor, env, sink);
            }
            STATE_MOTHER_SWIM_FLOAT => {
                self.sea_water_check(actor, env, sink);
            }
            _ => {}
        }
    }

    fn ret_keisan_move(&mut self, actor: &mut Actor) -> bool {
        add_calc0(&mut self.wave, 1.0, WAVE_DAMP);
        let x = self.home_pos.x - actor.pos.x;
        let y = self.home_pos.y - actor.pos.y;
        let z = self.home_pos.z - actor.pos.z;
        add_calc_angle_s2(&mut actor.angle[1], to_s16(x.atan2(z)), 1, RETURN_STEP);
        self.shape_angle = actor.angle;
        if (y * y).sqrt() > RETURN_HEIGHT {
            add_calc(&mut actor.pos.y, self.home_pos.y, 1.0, RETURN_RATE);
            return false;
        }
        if (x * x + z * z).sqrt() < RETURN_SPAN {
            self.speed_f = 0.0;
            actor.pos = self.home_pos;
            return true;
        }
        add_calc(&mut actor.pos.x, self.home_pos.x, 1.0, RETURN_RATE);
        add_calc(&mut actor.pos.z, self.home_pos.z, 1.0, RETURN_RATE);
        false
    }

    fn mother_return_move(&mut self, actor: &mut Actor, env: &Env<'_>) {
        match self.state {
            STATE_RETURN_START => {
                if env.conduct {
                    return;
                }
                let roll = self.rnd_f(RETURN_JITTER) as i16;
                self.timer[TIMER_RANDOM] = roll;
                self.state = self.state.wrapping_add(1);
                self.return_home(actor);
            }
            STATE_RETURN_HOME => self.return_home(actor),
            STATE_RETURN_FACE => {
                add_calc_angle_s2(&mut actor.angle[1], self.home_angle[1], 1, RETURN_TURN);
                if turn_gap(actor.angle[1], self.home_angle[1]) < RETURN_FACE {
                    actor.angle[1] = self.home_angle[1];
                    self.state = self.state.wrapping_add(1);
                }
            }
            STATE_RESTORE => {
                if self.parts == PARTS_ALL {
                    self.cut_seen = 0;
                    self.cut_point = 0;
                    self.parent_state = PARENT_HOLD;
                    self.cyl.tg_set = true;
                    self.cyl.co_set = true;
                    self.cc_height = CC_REST_HEIGHT;
                    self.cc_radius = CC_REST_RADIUS;
                    self.status = (self.status & !STTS_MAP_MASK) | STTS_SHOWMAP | STTS_MAP_BOARD;
                    self.mode = MODE_MOTHER;
                    self.state = STATE_TALK;
                    return;
                }
            }
            _ => {}
        }
        if self.timer[TIMER_RANDOM] == 0 && self.state >= STATE_RETURN_HOME {
            add_calc_angle_s2(&mut actor.angle[0], self.home_angle[0], 1, RETURN_TURN);
            add_calc_angle_s2(&mut actor.angle[2], self.home_angle[2], 1, RETURN_TURN);
            self.shape_angle = actor.angle;
        }
    }

    fn return_home(&mut self, actor: &mut Actor) {
        if self.timer[TIMER_RANDOM] != 0 {
            return;
        }
        self.drop_pos = Vec3::ZERO;
        self.drift_x = 0.0;
        self.drift_y = 0.0;
        self.drift_z = 0.0;
        self.tumble = 0;
        if self.ret_keisan_move(actor) {
            self.state = self.state.wrapping_add(1);
        }
    }

    fn parts_move(&mut self, actor: &mut Actor, live: &Link, env: &Env<'_>) {
        match self.state {
            STATE_PART_SEND => {
                self.steps = 0;
                actor.angle[1] = bearing(env.player_pos, actor.pos);
                if self.cut_point != 1 {
                    actor.angle[1] =
                        actor.angle[1].wrapping_add(self.rnd_fx(PART_TURN_JITTER) as i16);
                    if self.parent.is_some() {
                        let side = self.rnd_fx(PART_FAST_JITTER) + PART_FAST_BASE;
                        let rise = self.rnd_fx(PART_RISE_JITTER) + PART_RISE_BASE;
                        self.speed_f = side;
                        self.speed.y = rise;
                        self.gravity = PART_GRAVITY;
                        if live.hammer_mode == HAMMER_SIDESWING {
                            let far = self.rnd_fx(PART_HAMMER_JITTER) + PART_HAMMER_FAST;
                            let up = self.rnd_fx(PART_HAMMER_JITTER) + PART_HAMMER_RISE;
                            actor.angle[1] = env.player_angle.wrapping_add(TOSS_TURN);
                            self.speed_f = far;
                            self.speed.y = up;
                            self.gravity = PART_GRAVITY;
                        }
                    }
                    self.shape_angle[1] = actor.angle[1];
                    self.state = STATE_PART_FLY;
                } else {
                    self.shape_angle[1] = actor.angle[1];
                    self.steps = 0;
                    if (self.parts & PART_EDGE) != 0 {
                        self.speed_f = TOSS_FAST;
                        self.speed.y = TOSS_RISE;
                        self.gravity = WATER_GRAVITY;
                    } else {
                        self.speed_f = TOSS_SLOW;
                        self.speed.y = TOSS_SLOW_RISE;
                        self.gravity = WATER_GRAVITY;
                    }
                    self.state = STATE_PART_LAND;
                }
            }
            STATE_PART_FLY => {
                if self.wall_hit {
                    self.speed_f *= -1.0;
                }
                self.shape_angle[0] = self.shape_angle[0].wrapping_add(PART_SPIN);
                if env.grounded(actor.pos) {
                    self.speed_f = PART_GROUND_FAST;
                    self.speed.y = PART_GROUND_RISE;
                    self.state = self.state.wrapping_add(1);
                }
            }
            STATE_PART_GROUND => {
                add_calc_angle_s2(&mut self.shape_angle[0], FALL_ANGLE, 1, FALL_TURN);
                if env.grounded(actor.pos) {
                    self.tumble = SPIN_TUMBLE;
                    self.read_ground_angle(actor, env);
                    self.speed_f = 0.0;
                    self.gravity = 0.0;
                    self.state = self.state.wrapping_add(1);
                }
            }
            STATE_PART_SPIN => {
                add_calc_angle_s2(
                    &mut self.shape_angle[0],
                    self.tumble.wrapping_add(FALL_ANGLE),
                    1,
                    FALL_TURN,
                );
                self.tumble ^= SPIN_XOR;
                self.steps = self.steps.wrapping_add(1);
                self.steps &= 3;
                if (self.steps & 1) == 0 {
                    self.tumble = SPIN_SNAP;
                }
                if (self.steps & 3) == 0 {
                    self.shape_angle[0] = FALL_ANGLE;
                    self.tumble = 0;
                    self.cyl.co_set = false;
                    self.cyl.tg_set = false;
                    self.cyl.tg_hit = false;
                    self.state = self.state.wrapping_add(1);
                }
            }
            STATE_PART_QUAKE_WAIT => {
                if self.timer[TIMER_SHOCK] == 0 && self.shock_damage_check(env, actor.pos) {
                    let kick = self.rnd_f(SHOCK_JITTER) + SHOCK_RISE;
                    self.speed.y = kick;
                    self.gravity = WATER_GRAVITY;
                    self.state = STATE_PART_GROUND;
                }
            }
            STATE_PART_LAND => {
                if self.wall_hit {
                    self.speed_f *= -1.0;
                }
                if self.speed.y < 0.0 {
                    add_calc_angle_s2(&mut self.shape_angle[0], FALL_ANGLE, 1, FALL_TURN);
                }
                if env.grounded(actor.pos) {
                    self.shape_angle[0] = FALL_ANGLE;
                    self.tumble = SPIN_TUMBLE;
                    self.read_ground_angle(actor, env);
                    self.speed_f = 0.0;
                    self.gravity = 0.0;
                    self.state = STATE_PART_SPIN;
                }
            }
            _ => {}
        }
    }

    fn chield_parts_move(
        &mut self,
        actor: &mut Actor,
        live: &Link,
        env: &Env<'_>,
        next: &mut Link,
    ) {
        match self.state {
            STATE_PART_BACK_START => {
                if env.conduct {
                    return;
                }
                let roll = self.rnd_f(RETURN_JITTER) as i16;
                self.timer[TIMER_RANDOM] = roll;
                self.state = self.state.wrapping_add(1);
                self.return_home(actor);
            }
            STATE_PART_HOME => self.return_home(actor),
            STATE_PART_FACE => {
                add_calc_angle_s2(&mut actor.angle[1], self.home_angle[1], 1, RETURN_TURN);
                if turn_gap(actor.angle[1], self.home_angle[1]) < RETURN_FACE {
                    actor.angle[1] = self.home_angle[1];
                    self.state = self.state.wrapping_add(1);
                    next.parts ^= self.parts;
                }
            }
            STATE_PART_WAIT => {
                if live.parent_state == PARENT_HOLD {
                    self.doomed = true;
                }
                if live.parent_state == PARENT_SEND {
                    self.mode = MODE_PART;
                    self.state = STATE_PART_SEND;
                }
            }
            _ => {}
        }
        if self.timer[TIMER_RANDOM] == 0 && self.state >= STATE_PART_HOME {
            add_calc_angle_s2(&mut actor.angle[0], self.home_angle[0], 1, RETURN_TURN);
            add_calc_angle_s2(&mut actor.angle[2], self.home_angle[2], 1, RETURN_TURN);
            self.shape_angle = actor.angle;
        }
    }

    fn chield_water_swim(&mut self, actor: &mut Actor, env: &Env<'_>, sink: &mut Sink<'_>) {
        match self.state {
            STATE_PART_SWIM_SETTLE => {
                self.tumble = 0;
                self.shape_angle[0] = FALL_ANGLE;
                self.cyl.co_set = false;
                self.cyl.tg_set = false;
                self.cyl.tg_hit = false;
                self.state = self.state.wrapping_add(1);
                self.sea_water_check(actor, env, sink);
            }
            STATE_PART_SWIM_FLOAT => {
                self.sea_water_check(actor, env, sink);
            }
            _ => {}
        }
    }
}