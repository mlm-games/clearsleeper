use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::arrow;
use super::bomb;
use super::bombflower;
use super::boomerang;
use super::collision::Collision;
use super::combat::{Combat, CutKind};
use super::events::{SimEvent, SimEvents};
use super::flags::ZoneActors;
use super::ground;
use super::hammer;
use super::hookshot;
use super::intent::DoIntent;
use super::msg::{MsgBox, MsgTable};
use super::player::Player;
use super::sea::Sea;
use use_placeholder_marker;
use super::stream::add_calc_angle_s2;
use super::walk::from_s16;
use super::water::{self, Water};

pub const KANBAN_ACTOR: &str = "Kanban";

pub fn is_kanban(name: &str) -> bool {
    name == KANBAN_ACTOR
}

const PART_SHIFT: u32 = 0x14;
const PART_COUNT: usize = 11;

const TIMER_COUNT: usize = 5;
const TIMER_RANDOM: usize = 0;
const TIMER_SINK: usize = 1;
const TIMER_SHOCK: usize = 4;

const MODE_MOTHER: i16 = 0;
const MODE_RETURN: i16 = 1;
const MODE_MOTHER_WATER: i16 = 2;
const MODE_PART: i16 = 100;
const MODE_PART_RETURN: i16 = 101;
const MODE_PART_WATER: i16 = 102;

const STATE_TALK: i16 = 10;
const STATE_TOSS: i16 = 11;
const STATE_FALL: i16 = 12;
const STATE_BOUNCE: i16 = 13;
const STATE_QUAKE_WAIT: i16 = 14;
const STATE_RETURN_TURN: i16 = 20;
const STATE_RETURN_HOLD: i16 = 21;
const STATE_RETURN_SPIN: i16 = 30;
const STATE_RETURN_HOME: i16 = 31;
const STATE_RETURN_FACE: i16 = 32;
const STATE_RESTORE: i16 = 33;
const STATE_RETURN_REST: i16 = 0x14;
const STATE_RETURN_SINK: i16 = 0x15;
const STATE_PART_SEND: i16 = 0x6e;
const STATE_PART_FLY: i16 = 0x6f;
const STATE_PART_BOUND: i16 = 0x70;
const STATE_PART_SPIN: i16 = 0x71;
const STATE_PART_QUAKE_WAIT: i16 = 0x72;
const STATE_PART_HOME: i16 = 0x78;
const STATE_PART_BACK: i16 = 0x83;
const STATE_PART_FACE: i16 = 0x84;
const STATE_PART_WAIT: i16 = 0x85;
const STATE_SWIM_SETTLE: i16 = 40;
const STATE_SWIM_FLOAT: i16 = 41;

const HAMMER_NONE: u8 = 0;
const HAMMER_FALL: u8 = 1;
const HAMMER_SIDESWING: u8 = 2;

const PARENT_HOLD: u8 = 1;
const PARENT_SEND: u8 = 2;

const PARTS_ALL: u32 = 0x7FE;
const PART_BASE: u32 = 1;

const STATUS_MAP_BOARD: u32 = 0x38;
const STATUS_PROFILE: u32 = 0x0004_4100;
const STATUS_SHOWMAP: u32 = 0x0000_0020;

const TACT_NO4: i32 = 4;

const GRAVITY: f32 = -3.0;
const WAVE_LIFT: f32 = 7.0;
const WAVE_STEP: i16 = 1000;
const WAVE_BASE: f32 = 147.0;
const WAVE_AMP: f32 = 2.0;
const WAVE_FLOOR: f32 = 120.0;
const WAVE_RISE: f32 = 30.0;
const SHOCK_DIST: f32 = 1000.0;
const SHOCK_TIMER: i16 = 100;
const SHOCK_KICK: f32 = 10.0;
const SHOCK_RISE: f32 = 20.0;
const TURN_TOSS: i16 = -0x4000;
const TOSS_FAST: f32 = 20.0;
const TOSS_RISE: f32 = 10.0;
const TOSS_SLOW: f32 = 3.0;
const TOSS_SLOW_RISE: f32 = 7.0;
const PART_GRAVITY: f32 = -5.0;
const PART_TUMBLE: i16 = 0x1000;
const PART_GROUND_SPEED: f32 = 5.0;
const PART_GROUND_RISE: f32 = 10.0;
const FALL_ANGLE: i16 = 0x4000;
const FALL_TURN: i16 = 0x1000;
const TUMBLE_RATE: i16 = 0x800;
const TUMBLE_SNAP: i16 = 0x400;
const TUMBLE_MASK: i16 = 0xFF00;
const RETURN_STEP: i16 = 0x1000;
const RETURN_TURN: i16 = 0x1000;
const RETURN_RANGE: f32 = 30.0;
const RETURN_Y_TOL: f32 = 2.0;
const RETURN_XZ_TOL: f32 = 2.0;
const RETURN_STEP_Y: f32 = 10.0;
const RETURN_STEP_XZ: f32 = 10.0;
const RETURN_DAMP: f32 = 2.0;
const SINK_HEIGHT: f32 = 90.0;
const SINK_HEIGHT_MID: f32 = 65.0;
const SINK_HEIGHT_LOW: f32 = 20.0;
const SINK_STEP: f32 = 30.0;
const DROP_OFFSET: f32 = 70.0;
const CC_HEIGHT_REST: f32 = 105.0;
const CC_RADIUS_REST: f32 = 50.0;
const CC_HEIGHT_GONE: f32 = 0.0;
const CC_RADIUS_GONE: f32 = 0.0;
const CC_HEIGHT_EDGE: f32 = 25.0;
const CC_RADIUS_EDGE: f32 = 10.0;
const CC_HEIGHT_BACK: f32 = 105.0;
const CC_RADIUS_BACK: f32 = 50.0;
const WALL_EDGE: f32 = 40.0;
const WALL_SET: f32 = 20.0;
const WALL_SET_Z: f32 = 30.0;
const ROLL_DIST: f32 = 35.0;
const ROLL_DIST_LIMIT: f32 = 75.0;
const ROLL_FACE: f32 = -0.9;
const ROLL_Y_LOW: f32 = -100.0;
const ROLL_Y_HIGH: f32 = 200.0;
const BACK_RANGE: i16 = 0x6000;
const BACK_EDGE: i16 = 0x2000;
const EYE_LIFT: f32 = 40.0;
const ATTENTION_LIFT: f32 = 60.0;
const TALK_RANGE: f32 = 500.0;
const TALK_RANGE_UNIT: f32 = 100.0;
const TALK_ATTENTION: u8 = 5;
const RND_Y: f32 = 8000.0;
const RND_FAST: f32 = 2.0;
const RND_FAST_BASE: f32 = 10.0;
const RND_RISE: f32 = 5.0;
const RND_RISE_BASE: f32 = 20.0;
const RND_FAST2: f32 = 5.0;
const RND_FAST2_BASE: f32 = 15.0;
const RND_RISE2: f32 = 5.0;
const RND_RISE2_BASE: f32 = 30.0;
const HAMMER_TOP_X: f32 = 160.0;
const HAMMER_TOP_Y: f32 = 70.0;
const PAUSE_TIMER: i16 = 1;
const PART_SEND_RETRY: i16 = 100;
const TARGET_MAX: usize = 10;
const TABLE_COUNT: usize = 40;

const SE_FALL_WATER: u32 = 0x6919;
const SE_BREAK_BOARD: u32 = 0x6937;

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
const AT_TYPE_TG_BOARD: u32 = !(AT_TYPE_WATER
    | AT_TYPE_UNK20000
    | AT_TYPE_WIND
    | AT_TYPE_UNK400000
    | AT_TYPE_LIGHT);

const CUT_NONE: u16 = 0;
const CUT_A: u16 = 1;
const CUT_F: u16 = 2;
const CUT_R: u16 = 3;
const CUT_L: u16 = 4;
const CUT_EA: u16 = 6;
const CUT_EB: u16 = 7;
const CUT_TURN: u16 = 8;
const CUT_HAMMER_SIDESWING: u16 = 0x11;
const CUT_HAMMER_FRONTSWING: u16 = 0x12;
const CUT_HAMMER_JUMP: u16 = 0x13;
const CUT_TABLE_SKIP: i16 = 255;
const CUT_TABLE_ALL: i16 = 0x7FE0_0000;

const REG8_S1: i32 = 0;
const REG8_S4: i16 = 0;
const REG8_F6: f32 = 0.0;
const REG8_F7: f32 = 0.0;
const REG8_F8: f32 = 0.0;
const REG8_F9: f32 = 0.0;
const REG8_F17: f32 = 0.0;
const REG8_F18: f32 = 0.0;
const REG8_F19: f32 = 0.0;

const PL_CUT_REAL: [u16; 32] = [
    CUT_NONE,
    CUT_A,
    CUT_F,
    CUT_R,
    CUT_L,
    5,
    CUT_EA,
    CUT_EB,
    CUT_TURN,
    9,
    0x0A,
    0x0B,
    0x0C,
    0x0D,
    0x0E,
    0x0F,
    0x10,
    CUT_HAMMER_SIDESWING,
    CUT_HAMMER_FRONTSWING,
    CUT_HAMMER_JUMP,
    0x14,
    0x15,
    0x16,
    0x17,
    0x18,
    0x19,
    0x1A,
    0x1B,
    0x1C,
    0x1D,
    0x1E,
    0x1F,
];

const PL_CUT_NO: [i16; 32] = [
    255, 0, 4, 3, 3, 1, 0, 4, 4, 4, 1, 3, 1, 3, 1, 1, 1, 3, 1, 1, 0, 1, 0, 1, 0, 1, 1, 4, 0, 1, 1, 1,
];

const CUT_PARTS_ARG: [u32; TABLE_COUNT] = [
    0x02000000, 0x04000000, 0x00800000, 0x01000000, 0x7FE00000, 0x7FE00000, 0x7FE00000, 0x7FE00000,
    0x04000000, 0x00800000, 0x01000000, 0x10000000, 0x40000000, 0x7FE00000, 0x7FE00000, 0x7FE00000,
    0x00400000, 0x00200000, 0x02000000, 0x04000000, 0x7FE00000, 0x7FE00000, 0x7FE00000, 0x7FE00000,
    0x00200000, 0x02000000, 0x04000000, 0x00800000, 0x7FE00000, 0x7FE00000, 0x7FE00000, 0x7FE00000,
    0x08000000, 0x00400000, 0x00200000, 0x02000000, 0x04000000, 0x00800000, 0x01000000, 0x10000000,
];

const CUT_PARTS_ROW: i16 = 8;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Statics {
    pub msg_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Blow {
    Idle,
    Sword,
    Hammer,
    HammerJump,
    Bomb,
    Arrow,
    FireArrow,
    IceArrow,
    LightArrow,
    Boomerang,
    Hookshot,
    Wind,
}

impl Blow {
    pub fn at_type(self) -> u32 {
        match self {
            Self::Idle | Self::Bomb | Self::Arrow | Self::Boomerang | Self::Hookshot => 0,
            Self::Sword => AT_TYPE_SWORD,
            Self::Hammer | Self::HammerJump => AT_TYPE_SKULL_HAMMER,
            Self::FireArrow => AT_TYPE_FIRE_ARROW,
            Self::IceArrow => AT_TYPE_ICE_ARROW,
            Self::LightArrow => AT_TYPE_LIGHT_ARROW,
            Self::Wind => AT_TYPE_WIND,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cyl {
    pub centre: Vec3,
    pub height: f32,
    pub radius: f32,
    pub co_set: bool,
    pub tg_set: bool,
    pub tg_hit: bool,
    pub tg_at_type: u32,
}

impl Cyl {
    const SRC: Cyl = Cyl {
        centre: Vec3::ZERO,
        height: 0.0,
        radius: 0.0,
        co_set: false,
        tg_set: false,
        tg_hit: false,
        tg_at_type: 0,
    };
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Kanban {
    pub part: bool,
    pub parts: u32,
    pub step_count: u8,
    pub hammer_mode: u8,
    pub parent_state: u8,
    pub wave_done: u8,
    pub wave: f32,
    pub cc_height: f32,
    pub cc_radius: f32,
    pub ground_angle: Vec3,
    pub timer: [i16; TIMER_COUNT],
    pub wave_phase: i16,
    pub mode: i16,
    pub state: i16,
    pub cut_seen: i16,
    pub cut_point: i16,
    pub tumble: i16,
    pub parent: Option<Entity>,
    pub drift_pos: Vec3,
    pub drift_x: f32,
    pub drift_y: f32,
    pub drift_z: f32,
    pub home_angle: [i16; 3],
    pub home_pos: Vec3,
    pub ripple_pos: Vec3,
    pub ripple: bool,
    pub cyl: Cyl,
    pub wall_x: f32,
    pub wall_z: f32,
    pub speed_f: f32,
    pub speed: Vec3,
    pub gravity: f32,
    pub shape_angle: [i16; 3],
    pub status: u32,
    pub eye_pos: Vec3,
    pub attention_pos: Vec3,
    pub seed: [i32; 3],
    pub health: i8,
    pub max_health: i8,
    pub pause_timer: i16,
    pub entered: bool,
    pub doomed: bool,
}

impl Kanban {
    pub fn spawn(parameters: u32) -> Self {
        let parts = parameters >> PART_SHIFT;
        let part = parts != 0;
        Self {
            part,
            parts,
            step_count: 0,
            hammer_mode: HAMMER_NONE,
            parent_state: HAMMER_NONE,
            wave_done: 0,
            wave: 0.0,
            cc_height: CC_HEIGHT_GONE,
            cc_radius: CC_RADIUS_GONE,
            ground_angle: Vec3::ZERO,
            timer: [0; TIMER_COUNT],
            wave_phase: 0,
            mode: MODE_MOTHER,
            state: STATE_TALK,
            cut_seen: 0,
            cut_point: 0,
            tumble: 0,
            parent: None,
            drift_pos: Vec3::ZERO,
            drift_x: 0.0,
            drift_y: 0.0,
            drift_z: 0.0,
            home_angle: [0; 3],
            home_pos: Vec3::ZERO,
            ripple_pos: Vec3::ZERO,
            ripple: false,
            cyl: Cyl::SRC,
            wall_x: WALL_SET,
            wall_z: WALL_SET_Z,
            speed_f: 0.0,
            speed: Vec3::ZERO,
            gravity: 0.0,
            shape_angle: [0; 3],
            status: STATUS_PROFILE,
            eye_pos: Vec3::ZERO,
            attention_pos: Vec3::ZERO,
            seed: [RND_SEED; 3],
            health: 0,
            max_health: 0,
            pause_timer: 0,
            entered: false,
            doomed: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.parts != 0
    }

    pub fn mother(&self) -> bool {
        !self.part
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

fn distance_angle_s(a: i16, b: i16) -> i32 {
    i32::from(a.wrapping_sub(b)).rem_euclid(0x1_0000)
}

fn wrapped_delta(a: i16, b: i16) -> i32 {
    let raw = i32::from(a) - i32::from(b);
    if raw > 0x8000 {
        raw - 0x1_0000
    } else if raw < -0x8000 {
        raw + 0x1_0000
    } else {
        raw
    }
}

fn bearing(from: Vec3, to: Vec3) -> i16 {
    walk_bearing(from, to)
}

fn walk_bearing(from: Vec3, to: Vec3) -> i16 {
    crate::sim::walk::to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn cyl_touches(centre: Vec3, radius: f32, height: f32, at: Vec3, sphere: f32) -> bool {
    let dx = at.x - centre.x;
    let dz = at.z - centre.z;
    if radius + sphere > (dx * dx + dz * dz).sqrt() {
        return false;
    }
    at.y + sphere >= centre.y && at.y - sphere <= centre.y + height
}

pub fn bomb_is_shock_target(name: &str, life: bomb::Life) -> bool {
    if name == bomb::BOMB_ACTORS[0] {
        return life == bomb::Life::Spent;
    }
    if name == bomb::BOMB_ACTORS[1] {
        return life == bomb::Life::Spent;
    }
    false
}

struct Spawn {
    parameters: u32,
    pos: Vec3,
    angle: [i16; 3],
    room: i8,
    set_id: u16,
    parent: Option<Entity>,
    home_pos: Vec3,
    home_angle: [i16; 3],
}

struct Env<'a> {
    solid: Option<&'a Collision>,
    sea: Option<&'a Sea>,
    water: Option<&'a Water>,
    player_pos: Vec3,
    player_angle: i16,
    player_cut: u16,
    hammer_quake: bool,
    sword_top: Vec3,
    conduct: bool,
    tact: i32,
    talk: bool,
    msg_live: bool,
    msg_status: u8,
}

impl Env<'_> {
    fn sea_probe(&self, x: f32, z: f32) -> (bool, f32) {
        water::probe(self.sea, x, z)
    }

    fn masked_water(&self, x: f32, z: f32) -> (bool, f32) {
        match self.water {
            Some(water) => (water.in_area, water.surface),
            None => (false, 0.0),
        }
    }

    fn sea_or(&self) -> bool {
        self.sea.is_some()
    }
}

struct Link {
    parts: u32,
    hammer_mode: u8,
    parent_state: u8,
}

struct Sink<'a> {
    events: &'a mut SimEvents,
    statics: &'a mut Statics,
    spawns: &'a mut Vec<Spawn>,
    msg: &'a mut MsgBox,
    table: Option<&'a MsgTable>,
}

impl Sink<'_> {
    fn se(&mut self, id: u32) {
        self.events.list.push(SimEvent::Se(id));
    }

    fn child(&mut self, board: &Kanban, actor: &Actor, parameters: u32, pos: Vec3) {
        self.spawns.push(Spawn {
            parameters,
            pos,
            angle: [actor.angle[1], 0, 0],
            room: actor.room,
            set_id: actor.set_id,
            parent: Some(board_entity(board)),
            home_pos: board.home_pos,
            home_angle: board.home_angle,
        });
    }

    fn talk(&mut self, msg_no: u32) {
        if self.statics.msg_id == -1 {
            if let Some(table) = self.table {
                self.msg.message_set(msg_no, table);
            }
            self.statics.msg_id = 1;
            return;
        }
        if self.statics.msg_id == 1 && self.msg_live() && self.msg_closed() {
            self.msg.set_status(super::msg::MSG_DESTROYED);
            self.statics.msg_id = -1;
        }
    }

    fn msg_live(&self) -> bool {
        true
    }

    fn msg_closed(&self) -> bool {
        self.msg_status == super::msg::MSG_BOX_CLOSED
    }
}

fn board_entity(_board: &Kanban) -> Entity {
    Entity::PLACEHOLDER
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    sea: Option<Res<'w, Sea>>,
    water: Option<Res<'w, Water>>,
    combat: Res<'w, Combat>,
    hammer: Res<'w, hammer::Cut>,
    do_intent: Res<'w, DoIntent>,
    table: Option<Res<'w, MsgTable>>,
    players: Query<'w, 's, (Entity, &'static Player)>,
    shells: Query<'w, 's, (&'static Actor, &'static bomb::Bomb), Without<Kanban>>,
    shafts: Query<'w, 's, &'static arrow::Arrow>,
    bolts: Query<'w, 's, &'static boomerang::Boomerang>,
    chains: Query<'w, 's, &'static hookshot::Hookshot>,
    blasts: Res<'w, bombflower::Blasts>,
}

pub fn control(
    watch: Watch<'_, '_>,
    mut statics: ResMut<Statics>,
    mut events: ResMut<SimEvents>,
    mut msg: ResMut<MsgBox>,
    mut zones: ResMut<ZoneActors>,
    mut commands: Commands,
    mut boards: Query<(Entity, &mut Actor, &mut Kanban), (With<Kanban>, Without<Player>)>,
) {
    if statics.msg_id == 0 {
        statics.msg_id = -1;
    }
    let solid = watch.collision.as_deref();
    let sea = watch.sea.as_deref();
    let wet = watch.water.as_deref();
    let msg_status = msg.status();
    let msg_live = msg.active();
    let mut issued: Vec<Spawn> = Vec::new();
    let mut links: Vec<(Entity, Link)> = Vec::new();
    let mut pending: Vec<(Entity, Link, bool)> = Vec::new();
    let Some((player_entity, player)) = watch.players.iter().next() else {
        return;
    };
    if player_entity == Entity::PLACEHOLDER {
        return;
    }
    let player_pos = player.pos;
    let player_angle = super::walk::to_s16(player.face);
    let player_cut = player_cut_of(&watch);
    let hammer_quake = watch.hammer.kind == hammer::CUT_HAMMER_FRONTSWING;
    let sword_top = hammer_top(player_pos, super::walk::to_s16(player.face));
    let talk_range = TALK_ATTENTION as f32 * TALK_RANGE_UNIT;
    let talkable = flat(player_pos, Vec3::ZERO) <= 0.0 || within_attention(player_pos, talk_range);
    let talk = watch.do_intent.0 && talkable;
    let targets: Vec<Vec3> = watch
        .shells
        .iter()
        .filter(|(actor, shell)| bomb_is_shock_target(&actor.name, shell.life))
        .take(TARGET_MAX)
        .map(|(actor, _)| actor.pos)
        .collect();
    let blowing = blow_of(&watch);

    for (entity, actor, board) in &mut boards {
        if !board.entered {
            continue;
        }
        let env = Env {
            solid,
            sea,
            water: wet,
            player_pos,
            player_angle,
            player_cut,
            hammer_quake,
            sword_top,
            conduct: false,
            tact: TACT_NO4,
            talk,
            msg_live,
            msg_status,
        };
        let mut sink = Sink {
            events: &mut events,
            statics: &mut statics,
            spawns: &mut issued,
            msg: &mut msg,
            table: watch.table.as_deref(),
        };
        board.execute(&mut *actor, env, blowing, &mut sink, board_is_mother(board));
        if board.mother() {
            links.push((
                entity,
                Link {
                    parts: board.parts,
                    hammer_mode: board.hammer_mode,
                    parent_state: board.parent_state,
                },
            ));
        } else {
            pending.push((
                entity,
                Link {
                    parts: board.parts,
                    hammer_mode: board.hammer_mode,
                    parent_state: board.parent_state,
                },
                board.doomed,
            ));
        }
    }

    for (entity, mut link, doomed) in pending {
        let Some((_, actor, board)) = boards.get_mut(entity).ok() else {
            continue;
        };
        let Some(parent) = board.parent else {
            let _ = commands.entity(entity).despawn();
            let _ = &mut link;
            continue;
        };
        let Some((_, _, live)) = links.iter_mut().find(|(id, _)| *id == parent) else {
            let _ = commands.entity(entity).despawn();
            continue;
        };
        board.chield_parts_move(&mut *actor, live, &mut issued_parent(board, &mut *actor, &issued));
        links
            .iter_mut()
            .find(|(id, _)| *id == parent)
            .map(|(_, live)| *live = *live);
        if doomed {
            let _ = commands.entity(entity).despawn();
        }
    }

    for spawn in issued {
        commands.spawn((
            Actor {
                name: KANBAN_ACTOR.to_owned(),
                parameters: spawn.parameters,
                pos: spawn.pos,
                angle: spawn.angle,
                set_id: spawn.set_id,
                room: spawn.room,
            },
            Kanban::spawn(spawn.parameters),
            Adopt {
                parent: spawn.parent,
                home_pos: spawn.home_pos,
                home_angle: spawn.home_angle,
            },
        ));
    }
    let _ = &zones;
    let _ = &mut zones;
}

fn issued_parent(_board: &Kanban, _actor: &mut Actor, _issued: &mut Vec<Spawn>) -> Vec<Spawn> {
    Vec::new()
}

fn within_attention(pos: Vec3, range: f32) -> bool {
    pos.x.abs() <= range && pos.z.abs() <= range
}

fn board_is_mother(board: &Kanban) -> bool {
    !board.part
}

fn hammer_top(pos: Vec3, facing: i16) -> Vec3 {
    let radians = from_s16(facing);
    let (sin, cos) = radians.sin_cos();
    Vec3::new(
        pos.x + HAMMER_TOP_X * cos,
        pos.y + HAMMER_TOP_Y,
        pos.z - HAMMER_TOP_X * sin,
    )
}

fn player_cut_of(watch: &Watch<'_, '_>) -> u16 {
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
        hammer::CUT_HAMMER_SIDESWING => Blow::Hammer,
        hammer::CUT_HAMMER_FRONTSWING => Blow::Hammer,
        _ => Blow::Idle,
    }
}