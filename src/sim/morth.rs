use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::arrow;
use super::area::StageName;
use super::boomerang;
use super::bomb;
use super::bombflower;
use super::collision::Collision;
use super::combat::{Combat, CutKind};
use super::events::{SimEvent, SimEvents};
use super::flags::ZoneActors;
use super::ground;
use super::hammer;
use super::health::Health;
use super::hookshot;
use super::item::Held;
use super::player::Player;
use super::roll;
use super::sea::Sea;
use super::stream::add_calc_angle_s2;
use super::swim;
use super::tsubo;
use super::walk::{from_s16, to_s16};
use super::water;

pub const MORTH_ACTORS: [&str; 2] = ["kuro_s", "kuro_t"];

pub fn is_morth(name: &str) -> bool {
    MORTH_ACTORS.contains(&name)
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Statics {
    pub ks_all: i32,
    pub kattuku: i32,
    pub heavy: bool,
    pub goron: i32,
    pub next_id: u32,
    pub gm_dead: bool,
}

const BYTE_MASK: u32 = 0xFF;
const COUNT_SHIFT: u32 = 0x08;
const REACH_SHIFT: u32 = 0x10;

const FORM_SPAWN: u8 = 0;
const FORM_SPAWN_ALT: u8 = 1;
const FORM_IDLE: u8 = 2;
const FORM_GRAB: u8 = 4;
const FORM_GRAB_GORON: u8 = 5;
const FORM_TUBO: u8 = 6;
const FORM_BIRTH: u8 = 7;
const FORM_NONE: u8 = 0xFF;

const CHILD_PARAM_SPAWNER: u32 = 3;
const CHILD_PARAM_SPAWNER_ALT: u32 = 2;
const CHILD_PARAM_TUBO: u32 = 3;

const KUTTUKU_MAX: i32 = 0x14;
const TUBO_COUNT_MAX: u8 = 0x15;
const HEAVY_MIN: i32 = 5;
const JOINT_LAST: i16 = 0x13;
const JOINT_LIMIT: i16 = 0x14;
const JOINT_COUNT: usize = 20;
const TIMER_COUNT: usize = 4;
const SWING_COUNT: usize = 5;

const JOINT_TABLE: [i16; JOINT_COUNT] = [
    0x20, 0x21, 0x25, 0x26, 0x02, 0x03, 0x04, 0x1E, 0x29, 0x1F, 0x24, 0x08, 0x06, 0x07, 0x05,
    0x0C, 0x0A, 0x0B, 0x09, 0x0E,
];

const ACTION_DOU: u8 = 0;
const ACTION_POKE: u8 = 1;
const ACTION_WIND: u8 = 2;
const ACTION_DEAD: u8 = 3;
const ACTION_CLING: u8 = 4;
const ACTION_TUBO: u8 = 10;
const ACTION_BIRTH: u8 = 20;

const MODE_DOU_INIT: u8 = 0;
const MODE_DOU_WAIT: u8 = 1;
const MODE_DOU_HOVER: u8 = 2;
const MODE_DOU_FACE: u8 = 3;
const MODE_POKE_START: u8 = 10;
const MODE_POKE_RISE: u8 = 11;
const MODE_POKE_FALL: u8 = 12;
const MODE_WIND_START: u8 = 20;
const MODE_WIND_RISE: u8 = 21;
const MODE_WIND_HOVER: u8 = 22;
const MODE_DEAD_SPIN: u8 = 30;
const MODE_DEAD_BOUNCE: u8 = 31;
const MODE_DEAD_BURST: u8 = 32;
const MODE_CLING_ENTER: u8 = 40;
const MODE_CLING_HOLD: u8 = 41;
const MODE_CLING_LEAP: u8 = 42;
const MODE_CLING_WAIT: u8 = 43;
const MODE_TUBO_SEARCH: u8 = 50;
const MODE_TUBO_FOLLOW: u8 = 51;
const MODE_TUBO_SPAWN: u8 = 52;
const MODE_BIRTH_RISE: u8 = 60;
const MODE_BIRTH_FALL: u8 = 61;

const MAX_HEALTH: i32 = 1;
const LIGHT_DEAD: i32 = -0x80;
const FREEZE_DAMAGE: i32 = 4;

const GRAVITY: f32 = -3.0;
const GRAVITY_POKE: f32 = -4.0;
const GRAVITY_WIND: f32 = -1.0;
const FALL_MAX: f32 = -20.0;
const WIND_FALL_CAP: f32 = -2.0;
const WIND_RISE_TARGET: f32 = 200.0;
const WIND_RISE_STEP: f32 = 0.8;
const WIND_RATE_TARGET: f32 = 5.0;
const WIND_RATE_STEP: f32 = 1.0;
const WIND_RATE_MAX: f32 = 0.5;
const WIND_BRAKE_STEP: f32 = 0.5;
const WIND_BRAKE_MAX: f32 = 2.0;
const WIND_SWAY: f32 = 2.0;
const WIND_SPEED_F: f32 = 10.0;
const DEAD_SPEED_F: f32 = 15.0;
const DEAD_RISE: f32 = 20.0;
const WIND_SPIN_RATE: i16 = 3000;

const RADIUS: f32 = 20.0;
const RADIUS_POKE_HIGH: f32 = 60.0;
const RADIUS_TUBO: f32 = 100.0;
const RADIUS_SRC: f32 = 15.0;
const TG_TYPE_MORTH: u32 = !(AT_TYPE_WATER | AT_TYPE_UNK20000 | AT_TYPE_UNK400000 | AT_TYPE_LIGHT);
const PLAYER_TG_RADIUS: f32 = 30.0;
const PLAYER_TG_HEIGHT: f32 = 125.0;
const LIFT: f32 = 25.0;
const LIFT_DEAD: f32 = 20.0;
const LIFT_DEAD_ALT: f32 = 45.0;
const LIFT_BIRTH: f32 = 30.0;
const BRK_FRAME_MAX: f32 = 7.0;
const BTK_FRAME_MAX: i16 = 7;

const AT_TYPE_SWORD: u32 = 1 << 1;
const AT_TYPE_UNK8: u32 = 1 << 3;
const AT_TYPE_WATER: u32 = 1 << 8;
const AT_TYPE_FIRE: u32 = 1 << 9;
const AT_TYPE_SKULL_HAMMER: u32 = 1 << 16;
const AT_TYPE_UNK20000: u32 = 1 << 17;
const AT_TYPE_FIRE_ARROW: u32 = 1 << 18;
const AT_TYPE_ICE_ARROW: u32 = 1 << 19;
const AT_TYPE_LIGHT_ARROW: u32 = 1 << 20;
const AT_TYPE_WIND: u32 = 1 << 21;
const AT_TYPE_UNK400000: u32 = 1 << 22;
const AT_TYPE_LIGHT: u32 = 1 << 23;
const AT_TYPE_STALFOS_MACE: u32 = 1 << 24;

const STTS_UNK4000: u32 = 0x0000_4000;
const STTS_UNK8000000: u32 = 0x8000_0000;

const DIST_FAR: f32 = 10000.0;
const DIST_GRAB: f32 = 500.0;
const DIST_SHOCK: f32 = 200.0;
const SHOCK_HEIGHT: f32 = 40.0;
const PLAYER_FAST: f32 = 12.0;
const NUDGE_SCALE: f32 = 0.3;
const SHAPE_SPIN: i16 = 0x500;
const SHAPE_SPIN_RANGE: f32 = 2000.0;
const SHAPE_SPIN_MAX: i16 = 0x100;
const CLING_FOLLOW_MIN: f32 = 10.0;
const CLING_TOUCH: f32 = 2.0;
const CLING_OVERRIDE: f32 = 100.0;
const CLING_SLOW: f32 = 8.0;
const CLING_LEAP: f32 = 15.0;
const CLING_CLAW: f32 = 26.0;
const CLING_POKE: i16 = 0x32;
const CLING_POKE_MIN: f32 = 40.0;
const TUBO_JAR_XZ: f32 = 20.0;
const TUBO_BURST_Y: f32 = 40.0;
const TUBO_BURST_XZ: f32 = 40.0;
const NARAKU_FLOOR: f32 = -500.0;
const NARAKU_TICKS: u8 = 0x32;
const WATER_LIFT: f32 = 20.0;
const WATER_ESCAPE: f32 = 100.0;
const WATER_STEP: f32 = 10.0;
const GORON_ANGLE: f32 = 8192.0;
const GORON_SPEED: f32 = 6.0;
const GORON_FAST: f32 = 34.0;
const GORON_UP: f32 = 8.0;
const GORON_RISE: f32 = 22.0;
const SPAWN_HEIGHT: f32 = 120.0;
const SPAWN_LIFT: f32 = 120.0;
const SPAWN_SCATTER: f32 = 40.0;
const BIRTH_UP: f32 = 20.0;
const BIRTH_UP_JITTER: f32 = 5.0;
const BIRTH_GRAVITY: f32 = 2.0;
const BIRTH_HEAD: f32 = 10.0;
const GORON_RATE: [f32; 3] = [0.25, 0.5, 1.0];

const ITEM_HEART: u8 = 10;
const ITEM_MAGIC: u8 = 11;
const ITEM_ARROW: u8 = 12;
const ITEM_NONE13: u8 = 13;
const ITEM_TABLE: [u8; 3] = [ITEM_HEART, ITEM_MAGIC, ITEM_ARROW];
const ITEM_TABLE_MAX: f32 = 2.99;
const ITEM_LOW_LIFE: i32 = 8;
const ITEM_NO_ARROWS: i32 = 0;
const ITEM_ARROW_CAP: i32 = 10;
const MAGIC_HALF: i32 = 2;

const SE_LK_LAST_HIT: u32 = 0x2828;
const SE_CV_KS_ATTACK: u32 = 0x486D;
const SE_CV_KS_DAMAGE: u32 = 0x486E;
const SE_CV_KS_BLOW: u32 = 0x486F;
const SE_CM_KS_MOVE: u32 = 0x587A;
const SE_CM_KS_ATTACK: u32 = 0x587B;
const SE_CM_KS_DIE: u32 = 0x587C;
const SE_CM_FREEZE: u32 = 0x58FA;
const SE_CM_L_ARROW_SHRINK: u32 = 0x5904;
const SE_CM_L_ARROW_PASS_AWAY: u32 = 0x5905;

const PAUSE_TIMER: i16 = 2;
const STEAL_BIT: u8 = 1;

const STAGE_GANONK: &str = "GanonK";
const PICKUP_ACTOR: &str = "item";

const CULL_MIN: [f32; 3] = [-20.0, -20.0, -20.0];
const CULL_MAX: [f32; 3] = [20.0, 20.0, 20.0];

const ICE_FREEZE: i16 = 200;
const ICE_RISE: f32 = 30.0;
const ICE_DELAY: i16 = 40;
const ICE_SPIN: f32 = 3000.0;
const ICE_FALL: f32 = 5.0;
const FIRE_BURN: i16 = 100;
const LIGHT_START: i8 = 1;
const LIGHT_SHRINK_END: i8 = 70;
const LIGHT_GONE: i8 = 90;
const LIGHT_SCALE: f32 = 0.2;
const LIGHT_Y: f32 = 0.0;
const LIGHT_SHRINK_STEP: f32 = 0.1;
const LIGHT_SHRINK_MAX: f32 = 0.01;
const LIGHT_RISE_TARGET: f32 = 5.0;
const LIGHT_RISE_STEP: f32 = 0.1;
const LIGHT_RISE_MAX: f32 = 1.0;
const LIGHT_FADE_STEP: f32 = 0.1;
const LIGHT_FADE_MAX: f32 = 0.05;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const REG8_F8: f32 = 0.0;
const REG8_F9: f32 = 0.0;
const REG8_F10: f32 = 0.0;
const REG8_F11: f32 = 0.0;
const REG8_F12: f32 = 0.0;
const REG8_F13: f32 = 0.0;
const REG8_F19: f32 = 0.0;
const REG12_F16: f32 = 0.0;
const REG12_F17: f32 = 0.0;
const REG12_F18: f32 = 0.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Light {
    pub timer: i8,
    pub scale_xz: f32,
    pub scale_y: f32,
    pub particle_scale: f32,
    pub y_offset: f32,
}

impl Light {
    fn idle() -> Self {
        Self {
            timer: 0,
            scale_xz: 1.0,
            scale_y: 1.0,
            particle_scale: LIGHT_SCALE,
            y_offset: LIGHT_Y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum At {
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
    Unk8,
}

impl At {
    fn at_type(self) -> u32 {
        match self {
            Self::Sword => AT_TYPE_SWORD,
            Self::Hammer | Self::HammerJump => AT_TYPE_SKULL_HAMMER,
            Self::Bomb => 0,
            Self::Arrow | Self::Boomerang | Self::Hookshot | Self::Idle => 0,
            Self::FireArrow => AT_TYPE_FIRE_ARROW,
            Self::IceArrow => AT_TYPE_ICE_ARROW,
            Self::LightArrow => AT_TYPE_LIGHT_ARROW,
            Self::Wind => AT_TYPE_WIND,
            Self::Unk8 => AT_TYPE_UNK8,
        }
    }

    fn reach(self) -> f32 {
        match self {
            Self::Sword => 20.0,
            Self::Hammer | Self::HammerJump => 50.0,
            Self::Bomb => 150.0,
            Self::Arrow | Self::FireArrow | Self::IceArrow | Self::LightArrow => 35.0,
            Self::Boomerang => 90.0,
            Self::Hookshot => 5.0,
            Self::Idle | Self::Wind | Self::Unk8 => 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blow {
    pub at: At,
    pub bombs: usize,
    pub boomerang_live: bool,
    pub hookshot_shot: bool,
    pub arrows: usize,
    pub fire_arrows: usize,
    pub light_arrows: usize,
}

impl Blow {
    fn live(self) -> bool {
        match self.at {
            At::Bomb => self.bombs > 0,
            At::Boomerang => self.boomerang_live,
            At::Hookshot => self.hookshot_shot,
            At::Arrow => self.arrows > 0,
            At::FireArrow => self.fire_arrows > 0,
            At::LightArrow => self.light_arrows > 0,
            At::Idle | At::Wind | At::Unk8 => false,
            At::Sword | At::Hammer | At::HammerJump | At::IceArrow => true,
        }
    }

    fn hits(self, centre: Vec3, radius: f32, at: Vec3) -> bool {
        if self.at == At::Idle || self.at == At::Wind || self.at == At::Unk8 {
            return false;
        }
        if !self.live() {
            return false;
        }
        (at - centre).length() <= radius + self.at.reach()
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Morth {
    pub form: u8,
    pub count: u8,
    pub action: u8,
    pub mode: u8,
    pub cling: u8,
    pub water_in: u8,
    pub ice_flag: u8,
    pub swing_dir: u8,
    pub floor_ticks: u8,
    pub actor_id: u32,
    pub room: i8,
    pub set_id: u16,
    pub timers: [i16; TIMER_COUNT],
    pub swings: [i16; SWING_COUNT],
    pub angle_offset: i16,
    pub sweep: i16,
    pub joint: i16,
    pub joint_slot: i16,
    pub joint_live: bool,
    pub frame: i16,
    pub lift: f32,
    pub rise: f32,
    pub sway: f32,
    pub base_y: f32,
    pub climb: f32,
    pub goal_reach: f32,
    pub radius: f32,
    pub frame_f: f32,
    pub speed_f: f32,
    pub gravity: f32,
    pub speed: Vec3,
    pub shape_angle: [i16; 3],
    pub pos: Vec3,
    pub angle: [i16; 3],
    pub light: Light,
    pub freeze: i16,
    pub freeze_want: i16,
    pub freeze_delay: i16,
    pub freeze_spin: i16,
    pub fire: i16,
    pub shield_hit: bool,
    pub sph: Sph,
    pub in_water: bool,
    pub water_surface: f32,
    pub gm_link: bool,
    pub gm_dead: bool,
    pub tsuko: Option<Entity>,
    pub drop: u8,
    pub dead_pos: Vec3,
    pub pause_timer: i16,
    pub steal_bit: u8,
    pub seed: [i32; 3],
    pub health: i32,
    pub status: u32,
    pub entered: bool,
    pub doomed: bool,
    pub counted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sph {
    pub centre: Vec3,
    pub radius: f32,
    pub at_type: u32,
    pub at_atp: u8,
    pub at_set: bool,
    pub at_vs_player: bool,
    pub at_vs_other: bool,
    pub at_hit: bool,
    pub tg_type: u32,
    pub tg_set: bool,
    pub tg_hit: bool,
    pub co_set: bool,
}

impl Sph {
    const SRC: Sph = Sph {
        centre: Vec3::ZERO,
        radius: RADIUS_SRC,
        at_type: AT_TYPE_STALFOS_MACE,
        at_atp: 0,
        at_set: true,
        at_vs_player: true,
        at_vs_other: false,
        at_hit: false,
        tg_type: TG_TYPE_MORTH,
        tg_set: true,
        tg_hit: false,
        co_set: true,
    };
}

impl Morth {
    pub fn spawn(parameters: u32) -> Self {
        let mut form = (parameters & BYTE_MASK) as u8;
        if form == FORM_NONE {
            form = FORM_SPAWN;
        }
        let mut count = ((parameters >> COUNT_SHIFT) & BYTE_MASK) as u8;
        if form <= FORM_SPAWN_ALT && (count > TUBO_COUNT_MAX || count == 0) {
            count = 1;
        }
        let mut reach_byte = ((parameters >> REACH_SHIFT) & BYTE_MASK) as u8;
        if reach_byte == FORM_NONE {
            reach_byte = 0;
        }
        Self {
            form,
            count,
            action: ACTION_DOU,
            mode: MODE_DOU_INIT,
            cling: 0,
            water_in: 0,
            ice_flag: 0,
            swing_dir: 0,
            floor_ticks: 0,
            actor_id: 0,
            room: 0,
            set_id: 0,
            timers: [0; TIMER_COUNT],
            swings: [0; SWING_COUNT],
            angle_offset: 0,
            sweep: 0,
            joint: 0,
            joint_slot: JOINT_TABLE[0],
            joint_live: false,
            frame: 0,
            lift: LIFT,
            rise: 0.0,
            sway: 0.0,
            base_y: 0.0,
            climb: 0.0,
            goal_reach: f32::from(reach_byte) * 10.0,
            radius: RADIUS,
            frame_f: 0.0,
            speed_f: 0.0,
            gravity: 0.0,
            speed: Vec3::ZERO,
            shape_angle: [0; 3],
            pos: Vec3::ZERO,
            angle: [0; 3],
            light: Light::idle(),
            freeze: 0,
            freeze_want: 0,
            freeze_delay: 0,
            freeze_spin: 0,
            fire: 0,
            shield_hit: false,
            sph: Sph::SRC,
            in_water: false,
            water_surface: 0.0,
            gm_link: false,
            gm_dead: false,
            tsuko: None,
            drop: 0,
            dead_pos: Vec3::ZERO,
            pause_timer: 0,
            steal_bit: 0,
            seed: [RND_SEED; 3],
            health: MAX_HEALTH,
            status: 0,
            entered: false,
            doomed: false,
            counted: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.form != FORM_TUBO
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

    fn timer(&mut self, index: usize) -> bool {
        let slot = match self.timers.get_mut(index) {
            Some(slot) => slot,
            None => return false,
        };
        if *slot == 0 {
            return false;
        }
        *slot -= 1;
        true
    }

    fn clear_swings(&mut self) {
        for slot in self.swings.iter_mut() {
            *slot = 0;
        }
    }

    fn joint_slot(&self) -> i16 {
        match usize::try_from(self.joint)
            .ok()
            .and_then(|index| JOINT_TABLE.get(index))
        {
            Some(&slot) => slot,
            None => JOINT_TABLE[0],
        }
    }

    fn wind_speed(&mut self, heading: i16) {
        let amount = self.sway * from_s16(self.swings[0]).sin();
        let (sin, cos) = from_s16(heading).sin_cos();
        self.pos.x += amount * cos;
        self.pos.z += amount * -sin;
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

fn span(value: Vec3) -> f32 {
    value.length()
}

fn cyl_touches(centre: Vec3, radius: f32, height: f32, at: Vec3, sphere: f32) -> bool {
    let dx = at.x - centre.x;
    let dz = at.z - centre.z;
    if radius + sphere > (dx * dx + dz * dz).sqrt() {
        return false;
    }
    at.y + sphere >= centre.y && at.y - sphere <= centre.y + height
}

fn flat(from: Vec3, to: Vec3) -> f32 {
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    (dx * dx + dz * dz).sqrt()
}

fn bearing(from: Vec3, to: Vec3) -> i16 {
    to_s16((to.x - from.x).atan2(to.z - from.z))
}

struct Spawn {
    parameters: u32,
    pos: Vec3,
    angle: [i16; 3],
    room: i8,
    set_id: u16,
}

struct Env<'a> {
    player: &'a mut Player,
    solid: Option<&'a Collision>,
    speed: f32,
    drift: Vec3,
    rolling: bool,
    rolling_crash: bool,
    spinning: bool,
    swimming: bool,
    hooked: bool,
    quake: bool,
    sword_top: Vec3,
    head_top: Vec3,
}

impl Env<'_> {
    fn pos(&self) -> Vec3 {
        self.player.pos
    }

    fn distance(&self, from: Vec3) -> f32 {
        span(self.player.pos - from)
    }

    fn facing(&self, from: Vec3) -> i16 {
        bearing(from, self.player.pos)
    }
}

struct Sink<'a> {
    events: &'a mut SimEvents,
    spawns: &'a mut Vec<Spawn>,
    stage: &'a str,
    life: i32,
    arrows: i32,
    magic: i32,
    max_magic: i32,
}

impl Sink<'_> {
    fn se(&mut self, id: u32) {
        self.events.list.push(SimEvent::Se(id));
    }

    fn child(&mut self, parameters: u32, pos: Vec3, angle: [i16; 3], room: i8, set_id: u16) {
        self.spawns.push(Spawn {
            parameters,
            pos,
            angle,
            room,
            set_id,
        });
    }
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    sea: Option<Res<'w, Sea>>,
    combat: Res<'w, Combat>,
    hammer: Res<'w, hammer::Cut>,
    held: Res<'w, Held>,
    blasts: Res<'w, bombflower::Blasts>,
    roll: Res<'w, roll::Roll>,
    swim: Res<'w, swim::Swim>,
    hearts: Res<'w, Health>,
    stage: Res<'w, StageName>,
    players: Query<'w, 's, (Entity, &'static mut Player)>,
    shafts: Query<'w, 's, &'static arrow::Arrow>,
    bolts: Query<'w, 's, &'static boomerang::Boomerang>,
    chains: Query<'w, 's, &'static hookshot::Hookshot>,
    shells: Query<'w, 's, &'static bomb::Bomb, (With<bomb::Bomb>, Without<Morth>)>,
    pots: Query<
        'w,
        's,
        (Entity, &'static Actor, &'static tsubo::Tsubo),
        (With<tsubo::Tsubo>, Without<Morth>),
    >,
}

pub fn control(
    watch: Watch<'_, '_>,
    mut statics: ResMut<Statics>,
    mut events: ResMut<SimEvents>,
    mut zones: ResMut<ZoneActors>,
    mut commands: Commands,
    mut moths: Query<(Entity, &mut Actor, &mut Morth), (With<Morth>, Without<Player>)>,
) {
    let solid = watch.collision.as_deref();
    let sea = watch.sea.as_deref();
    let blow = blow_of(&watch);
    let hammer_kind = watch.hammer.kind;
    let rolling = watch.roll.active;
    let spinning = matches!(watch.combat.kind, CutKind::EA | CutKind::EB);
    let swimming = watch.swim.active;
    let hooked = blow.hookshot_shot;
    let quake = hammer_kind == hammer::CUT_HAMMER_FRONTSWING;
    let stage = watch.stage.0.clone();
    let life = i32::from(watch.hearts.units);
    let held = watch.held;
    let pots: Vec<(Entity, Vec3, i8)> = watch
        .pots
        .iter()
        .map(|(entity, actor, _)| (entity, actor.pos, actor.room))
        .collect();
    let mut players = watch.players;
    let Some((_, mut player)) = players.iter_mut().next() else {
        return;
    };
    let sword_top = player.pos + Vec3::new(0.0, 40.0, 0.0);
    let head_top = player.pos + Vec3::new(0.0, 80.0, 0.0);
    let speed = player.speed;
    let drift = player.drift;
    let mut issued: Vec<Spawn> = Vec::new();
    let mut pending: Vec<(Entity, i8, u16, u8, Vec3)> = Vec::new();
    let mut loot: Vec<(Vec3, u8, i8)> = Vec::new();
    for (entity, mut actor, mut moth) in &mut moths {
        let mut sink = Sink {
            events: &mut events,
            spawns: &mut issued,
            stage: &stage,
            life,
            arrows: i32::from(held.count),
            magic: 0,
            max_magic: 0,
        };
        moth.enter(&mut actor, solid, sea, &pots, &mut statics, &mut sink);
        let mut env = Env {
            player: &mut player,
            solid,
            speed,
            drift,
            rolling,
            rolling_crash: false,
            spinning,
            swimming,
            hooked,
            quake,
            sword_top,
            head_top,
        };
        moth.step(&mut actor, solid, &mut env, &blow, &mut statics, &mut sink);
        if moth.doomed {
            moth.release(&mut statics);
            pending.push((entity, actor.room, actor.set_id, moth.drop, moth.dead_pos));
            if moth.drop != ITEM_NONE13 {
                loot.push((moth.dead_pos, moth.drop, actor.room));
            }
        }
    }
    for spawn in issued {
        let name = MORTH_ACTORS[1];
        commands.spawn((
            Actor {
                name: name.to_owned(),
                parameters: spawn.parameters,
                pos: spawn.pos,
                angle: spawn.angle,
                set_id: spawn.set_id,
                room: spawn.room,
            },
            Morth::spawn(spawn.parameters),
        ));
    }
    for (entity, room, set_id, _drop, _pos) in pending {
        zones.mark_done(room, set_id);
        let _ = commands.entity(entity).despawn();
    }
    for (pos, item, room) in loot {
        commands.spawn((
            Actor {
                name: PICKUP_ACTOR.to_owned(),
                parameters: u32::from(item),
                pos,
                angle: [0; 3],
                set_id: 0,
                room,
            },
            super::pickup::Pickup::spawn(u32::from(item)),
        ));
    }
}

fn blow_of(watch: &Watch<'_, '_>) -> Blow {
    let mut at = At::Idle;
    if watch
        .chains
        .iter()
        .any(|chain| chain.phase != hookshot::Phase::Wait)
    {
        at = At::Hookshot;
    } else {
        for shaft in watch.shafts.iter() {
            if shaft.phase != arrow::Phase::Flying {
                continue;
            }
            at = match shaft.kind {
                arrow::TYPE_LIGHT => At::LightArrow,
                arrow::TYPE_FIRE => At::FireArrow,
                arrow::TYPE_ICE => At::IceArrow,
                _ => At::Arrow,
            };
            break;
        }
        if at == At::Idle
            && watch
                .bolts
                .iter()
                .any(|bolt| bolt.parameter == boomerang::MODE_THROW)
        {
            at = At::Boomerang;
        }
        if at == At::Idle
            && matches!(
                watch.hammer.kind,
                hammer::CUT_HAMMER_SIDESWING | hammer::CUT_HAMMER_FRONTSWING
            )
        {
            at = At::Hammer;
        }
        if at == At::Idle && watch.combat.active && (1..6).contains(&watch.combat.frame) {
            at = At::Sword;
        }
        if at == At::Idle && !watch.blasts.list.is_empty() {
            at = At::Bomb;
        }
    }
    let mut arrows = 0;
    let mut fire_arrows = 0;
    let mut light_arrows = 0;
    for shaft in watch.shafts.iter() {
        if shaft.phase != arrow::Phase::Flying {
            continue;
        }
        match shaft.kind {
            arrow::TYPE_FIRE => fire_arrows += 1,
            arrow::TYPE_LIGHT => light_arrows += 1,
            _ => arrows += 1,
        }
    }
    Blow {
        at,
        bombs: watch
            .shells
            .iter()
            .filter(|shell| shell.life != bomb::Life::Carry)
            .count(),
        boomerang_live: watch
            .bolts
            .iter()
            .any(|bolt| bolt.parameter == boomerang::MODE_THROW),
        hookshot_shot: watch
            .chains
            .iter()
            .any(|chain| chain.phase != hookshot::Phase::Wait),
        arrows,
        fire_arrows,
        light_arrows,
    }
}

impl Morth {
    fn enter(
        &mut self,
        actor: &mut Actor,
        solid: Option<&Collision>,
        sea: Option<&Sea>,
        pots: &[(Entity, Vec3, i8)],
        statics: &mut Statics,
        sink: &mut Sink<'_>,
    ) {
        if self.entered {
            return;
        }
        self.entered = true;
        statics.next_id += 1;
        self.actor_id = statics.next_id;
        self.pos = actor.pos;
        self.angle = actor.angle;
        self.shape_angle = actor.angle;
        self.room = actor.room;
        self.set_id = actor.set_id;
        self.lift = LIFT;
        self.radius = RADIUS;
        self.status = 0;
        self.timers[0] = (self.rnd_fx(25.0) + 50.0) as i16;
        self.sweep = (self.actor_id as i32 & 7) as i16 * 0x52 + 1000;
        let (in_water, surface) = water::probe(sea, self.pos.x, self.pos.z);
        self.in_water = in_water;
        self.water_surface = surface;
        if self.form == FORM_SPAWN || self.form == FORM_SPAWN_ALT {
            let parameter = if self.form == FORM_SPAWN {
                CHILD_PARAM_SPAWNER
            } else {
                CHILD_PARAM_SPAWNER_ALT
            };
            for index in 0..usize::from(self.count) {
                let mut pos = actor.pos;
                if index != 0 {
                    if actor.angle[0] != 0 {
                        pos.y += self.rnd_fx(SPAWN_HEIGHT);
                        let surface = ground::surface(solid, pos);
                        if pos.y < surface.unwrap_or(0.0) {
                            pos.y = surface.unwrap_or(0.0) + self.rnd_f(SPAWN_LIFT);
                        }
                    } else {
                        pos.x += self.rnd_fx(SPAWN_SCATTER);
                        pos.z += self.rnd_fx(SPAWN_SCATTER);
                    }
                }
                sink.child(
                    parameter,
                    pos,
                    [actor.angle[1], 0, actor.angle[2]],
                    actor.room,
                    actor.set_id,
                );
            }
            self.doomed = true;
            return;
        }
        if statics.ks_all == 0 {
            statics.kattuku = 0;
            statics.heavy = false;
            statics.goron = 0;
        }
        statics.ks_all += 1;
        self.counted = true;
        match self.form {
            FORM_TUBO => {
                self.sph.tg_set = false;
                self.sph.co_set = false;
                self.sph.tg_hit = false;
                self.action = ACTION_TUBO;
                self.mode = MODE_TUBO_SEARCH;
                self.tsuko = pots
                    .iter()
                    .find(|(_, jar, room)| {
                        *room == self.room
                            && (jar.x - self.pos.x).abs() < TUBO_JAR_XZ
                            && (jar.y - self.pos.y).abs() < TUBO_JAR_XZ
                            && (jar.z - self.pos.z).abs() < TUBO_JAR_XZ
                    })
                    .map(|(entity, _, _)| *entity);
            }
            FORM_BIRTH => {
                self.status |= STTS_UNK4000;
                self.action = ACTION_BIRTH;
                self.mode = MODE_BIRTH_RISE;
            }
            FORM_IDLE => {
                self.action = ACTION_DOU;
                self.mode = MODE_DOU_FACE;
                self.gravity = GRAVITY;
                self.bg_check(solid);
                if !ground::grounded(solid, self.pos) {
                    self.gravity = 0.0;
                }
            }
            FORM_GRAB | FORM_GRAB_GORON => {
                self.gm_link = true;
                if self.form == FORM_GRAB_GORON {
                    self.angle[1] = self
                        .angle[1]
                        .wrapping_add((self.rnd_fx(GORON_ANGLE)) as i32 as i16);
                    self.speed_f = GORON_FAST + self.rnd_f(GORON_SPEED);
                    self.speed.y = GORON_RISE + self.rnd_f(GORON_UP);
                }
            }
            _ => {}
        }
    }

    fn release(&mut self, statics: &mut Statics) {
        if !self.counted {
            return;
        }
        self.counted = false;
        statics.ks_all -= 1;
        if statics.ks_all == 0 {
            statics.kattuku = 0;
            statics.heavy = false;
            statics.goron = 0;
        }
    }

    fn bg_check(&mut self, solid: Option<&Collision>) {
        let mut probe = self.pos;
        probe.y -= self.lift;
        if let Some(solid) = solid {
            if let Some(surface) = solid.ground_at(probe.x, probe.z) {
                if probe.y < surface {
                    probe.y = surface;
                }
            }
            probe = solid.slide(self.pos, probe);
        }
        self.pos = probe;
        self.pos.y += self.lift;
    }

    fn tyaku_check(&mut self, solid: Option<&Collision>) -> bool {
        if ground::grounded(solid, self.pos) {
            return true;
        }
        if self.in_water {
            self.speed.y = 0.0;
            self.gravity = 0.0;
            return true;
        }
        false
    }

    fn contact(&self, solid: Option<&Collision>) -> bool {
        ground::grounded(solid, self.pos) || self.in_water
    }

    fn naraku_check(&mut self, solid: Option<&Collision>) {
        if ground::grounded(solid, self.pos) {
            self.floor_ticks = self.floor_ticks.wrapping_add(1);
            if self.pos.y < NARAKU_FLOOR || self.floor_ticks > NARAKU_TICKS {
                self.speed_f = 0.0;
                self.speed = Vec3::ZERO;
                self.gravity = 0.0;
                self.doomed = true;
                return;
            }
        }
        if self.in_water && self.pos.y < self.water_surface + WATER_LIFT {
            if self.water_in == 0 {
                self.water_in = 1;
            }
            add_calc(
                &mut self.pos.y,
                self.water_surface + WATER_LIFT,
                1.0,
                WATER_STEP,
            );
            self.gravity = 0.0;
            self.speed.y = 0.0;
        } else if self.water_in != 0
            && (ground::grounded(solid, self.pos)
                || self.pos.y > self.water_surface + WATER_ESCAPE)
        {
            self.water_in = 0;
        }
    }

    fn shock_damage_check(&mut self, env: &Env<'_>) -> bool {
        if !env.quake {
            return false;
        }
        if flat(self.pos, env.sword_top) < DIST_SHOCK {
            let rel = env.sword_top - self.pos;
            if rel.y.abs() < SHOCK_HEIGHT {
                self.action = ACTION_DEAD;
                self.mode = MODE_DEAD_BURST;
                return true;
            }
        }
        false
    }

    fn cc_calc(&mut self, env: &Env<'_>, blow: &Blow) {
        let player = env.pos();
        self.sph.at_hit = self.sph.at_set
            && self.sph.at_vs_player
            && cyl_touches(
                player,
                PLAYER_TG_RADIUS,
                PLAYER_TG_HEIGHT,
                self.sph.centre,
                self.sph.radius,
            );
        self.sph.tg_hit = self.sph.tg_set && blow.hits(self.sph.centre, self.sph.radius, player);
    }

    fn cc_set(&mut self) {
        self.sph.centre = self.pos;
        self.sph.radius = self.radius;
    }

    fn ks_kuttuki_check(&mut self, statics: &Statics) -> bool {
        if !self.sph.at_hit || self.shield_hit {
            return false;
        }
        if statics.kattuku < 0 || statics.kattuku >= KUTTUKU_MAX || statics.goron != 0 {
            return false;
        }
        self.sph.tg_set = false;
        self.sph.co_set = false;
        if self.water_in != 0 {
            self.water_in = 0;
        }
        self.action = ACTION_CLING;
        self.mode = MODE_CLING_ENTER;
        true
    }

    fn body_atari_check(&mut self, env: &Env<'_>, blow: &Blow) -> bool {
        if self.sph.tg_hit {
            self.angle[1] = env.facing(self.pos).wrapping_add(-32768);
            self.action = ACTION_DEAD;
            let at_type = blow.at.at_type();
            match at_type {
                AT_TYPE_WIND => {
                    self.angle[1] = bearing(self.pos, env.pos());
                    self.action = ACTION_WIND;
                    self.mode = MODE_WIND_START;
                    return false;
                }
                AT_TYPE_UNK8 => {
                    self.mode = MODE_DEAD_BURST;
                    self.health = 0;
                    self.mode = MODE_DEAD_SPIN;
                    return false;
                }
                AT_TYPE_SWORD => {
                    if self.mode != MODE_CLING_WAIT || self.cling != 0 {
                        self.pause_timer = PAUSE_TIMER;
                        self.steal_bit = STEAL_BIT;
                    }
                }
                AT_TYPE_SKULL_HAMMER => {
                    if matches!(blow.at, At::Hammer | At::HammerJump) {
                        self.speed_f = 0.0;
                        self.gravity = 0.0;
                        self.speed = Vec3::ZERO;
                        self.health = 0;
                        self.pause_timer = PAUSE_TIMER;
                        self.steal_bit = STEAL_BIT;
                        self.mode = MODE_DEAD_BURST;
                        return true;
                    }
                }
                AT_TYPE_FIRE | AT_TYPE_FIRE_ARROW => self.fire = FIRE_BURN,
                AT_TYPE_ICE_ARROW => {
                    self.freeze_want = ICE_FREEZE;
                    self.ice_flag = 1;
                    self.light.particle_scale = LIGHT_SCALE;
                    self.light.y_offset = LIGHT_Y;
                    self.fire = 0;
                }
                AT_TYPE_LIGHT_ARROW => {
                    self.light.timer = LIGHT_START;
                    self.light.particle_scale = LIGHT_SCALE;
                    self.light.y_offset = LIGHT_Y;
                    self.ice_flag = 1;
                }
                _ => {}
            }
            self.health = 0;
            self.mode = MODE_DEAD_SPIN;
            return true;
        }
        self.shock_damage_check(env)
    }

    fn action_dousa_move(&mut self, statics: &Statics, env: &Env<'_>, blow: &Blow, sink: &mut Sink<'_>) {
        match self.mode {
            MODE_DOU_INIT => {
                self.sway = 0.0;
                self.gravity = GRAVITY;
                self.clear_swings();
                self.mode += 1;
                self.dousa_wait(statics, env, sink);
            }
            MODE_DOU_WAIT => self.dousa_wait(statics, env, sink),
            MODE_DOU_HOVER => self.dousa_hover(statics, env, sink),
            MODE_DOU_FACE => {
                if env.distance(self.pos) < DIST_GRAB {
                    self.angle[1] = env.facing(self.pos);
                }
            }
            _ => {}
        }
        if self.mode == MODE_DOU_HOVER || self.mode == MODE_DOU_FACE {
            add_calc_angle_s2(&mut self.shape_angle[1], self.angle[1], 1, SHAPE_SPIN_MAX);
        }
        if self.body_atari_check(env, blow) {
            sink.se(SE_LK_LAST_HIT);
        }
    }

    fn dousa_wait(&mut self, statics: &Statics, env: &Env<'_>, sink: &mut Sink<'_>) {
        add_calc_angle_s2(&mut self.shape_angle[2], 0, 1, SHAPE_SPIN_MAX);
        if env.distance(self.pos) > DIST_FAR {
            return;
        }
        if self.shape_angle[2] > SHAPE_SPIN_MAX {
            return;
        }
        self.shape_angle[2] = 0;
        sink.se(SE_CM_KS_MOVE);
        if self.tyaku_check(env.solid) {
            self.gravity = GRAVITY;
            self.speed.y = 1.0;
            self.speed.y += self.rnd_f(5.0);
        }
        self.angle[1] = self.angle_offset.wrapping_add(env.facing(self.pos));
        self.mode += 1;
        self.dousa_hover(statics, env, sink);
    }

    fn dousa_hover(&mut self, statics: &Statics, env: &Env<'_>, sink: &mut Sink<'_>) {
        if statics.kattuku >= 0
            && statics.kattuku < KUTTUKU_MAX
            && (env.speed > PLAYER_FAST || statics.heavy)
            && env.distance(self.pos) < DIST_GRAB
            && !env.swimming
            && self.tyaku_check(env.solid)
        {
            self.action = ACTION_POKE;
            self.mode = MODE_POKE_START;
            return;
        }
        if env.speed <= PLAYER_FAST || env.swimming {
            let max = self.rnd_f(1.0) + 0.3;
            add_calc0(&mut self.speed_f, NUDGE_SCALE, max);
            add_calc_angle_s2(&mut self.shape_angle[2], 0, 1, SHAPE_SPIN_MAX);
            self.angle_offset = 0;
            return;
        }
        if self.angle_offset == 0 {
            let id = (self.actor_id & 0xf) as f32;
            self.angle_offset = (id * self.rnd_fx(512.0)) as i32 as i16;
        }
        self.shape_angle[2] = (self.rnd_fx(SHAPE_SPIN_RANGE)) as i32 as i16;
        self.wind_speed(self.shape_angle[1]);
        self.speed_f = PLAYER_FAST;
        self.swings[0] = self.swings[0].wrapping_add(1000);
        self.sway = 10.0;
        if self.tyaku_check(env.solid) {
            sink.se(SE_CM_KS_MOVE);
            self.gravity = GRAVITY;
            self.speed.y = 1.0;
            self.speed.y += self.rnd_f(5.0);
        }
    }

    fn action_kougeki_move(&mut self, statics: &Statics, env: &Env<'_>, blow: &Blow, sink: &mut Sink<'_>) {
        self.radius = RADIUS;
        if env.head_top.y > self.pos.y {
            self.radius = RADIUS_POKE_HIGH;
        }
        'poke: {
            match self.mode {
                MODE_POKE_START => {
                    self.sph.co_set = false;
                    self.speed_f = 26.0;
                    self.gravity = GRAVITY_POKE;
                    self.speed.y = 28.0;
                    self.clear_swings();
                    self.sway = 0.0;
                    self.angle[1] = env.facing(self.pos);
                    sink.se(SE_CM_KS_ATTACK);
                    sink.se(SE_CV_KS_ATTACK);
                    self.mode += 1;
                }
                MODE_POKE_RISE => {
                    if self.speed_f > 0.0 && self.swings[1] == 0 && self.shield_hit {
                        self.gravity = GRAVITY_POKE;
                        self.speed.y = 25.0;
                        self.speed_f *= -0.5;
                        self.swings[1] = 1;
                    }
                    if self.swings[1] == 0 && self.ks_kuttuki_check(statics) {
                        return;
                    }
                    if !self.contact(env.solid) {
                        break 'poke;
                    }
                    self.sph.co_set = true;
                    self.timers[2] = (self.rnd_f(20.0) + 20.0) as i16;
                    self.mode += 1;
                    self.poke_fall(statics, env);
                }
                MODE_POKE_FALL => self.poke_fall(statics, env),
                _ => {}
            }
        }
        add_calc_angle_s2(&mut self.shape_angle[1], self.angle[1], 1, SHAPE_SPIN_MAX);
        if self.body_atari_check(env, blow) {
            sink.se(SE_LK_LAST_HIT);
        }
    }

    fn poke_fall(&mut self, statics: &Statics, env: &Env<'_>) {
        self.ks_kuttuki_check(statics);
        if self.tyaku_check(env.solid) {
            self.gravity = GRAVITY_POKE;
            self.speed.y = 1.0;
            self.speed.y += self.rnd_f(5.0);
            self.speed_f = 16.0;
        }
        if self.timer(2) {
            self.speed_f = 0.0;
            self.radius = RADIUS;
            self.action = ACTION_DOU;
            self.mode = MODE_DOU_INIT;
        }
    }

    fn action_kaze_move(&mut self, env: &Env<'_>, blow: &Blow, sink: &mut Sink<'_>) {
        match self.mode {
            MODE_WIND_START => {
                self.clear_swings();
                self.timers[1] = 0x32;
                sink.se(SE_CV_KS_BLOW);
                self.base_y = self.pos.y;
                self.sway = 0.0;
                self.speed_f = WIND_SPEED_F + self.rnd_f(15.0);
                self.climb = WIND_SPEED_F + self.rnd_f(15.0);
                self.rise = 0.0;
                self.mode += 1;
                self.kaze_rise();
            }
            MODE_WIND_RISE => self.kaze_rise(),
            MODE_WIND_HOVER => {
                add_calc(
                    &mut self.gravity,
                    GRAVITY_WIND,
                    WIND_RATE_STEP,
                    WIND_RATE_MAX,
                );
                if self.speed.y < WIND_FALL_CAP {
                    self.speed.y = WIND_FALL_CAP;
                }
                add_calc0(&mut self.speed_f, WIND_BRAKE_STEP, WIND_BRAKE_MAX);
                self.wind_speed(self.shape_angle[1]);
                self.swings[0] = self.swings[0].wrapping_add(WIND_SPIN_RATE);
                self.sway = WIND_SWAY;
                if self.tyaku_check(env.solid) {
                    self.action = ACTION_DOU;
                    self.mode = MODE_DOU_INIT;
                    return;
                }
            }
            _ => {}
        }
        self.shape_angle[2] = self.shape_angle[2].wrapping_add(SHAPE_SPIN);
        if self.body_atari_check(env, blow) {
            sink.se(SE_LK_LAST_HIT);
        }
    }

    fn kaze_rise(&mut self) {
        add_calc(&mut self.speed.y, self.climb, WIND_RISE_STEP, self.rise);
        add_calc(&mut self.rise, WIND_RATE_TARGET, WIND_RATE_STEP, WIND_RATE_MAX);
        if self.base_y + WIND_RISE_TARGET < self.pos.y || self.timers[1] == 0 {
            self.mode += 1;
        }
    }

    fn dead_eff_set(&mut self, sink: &mut Sink<'_>) {
        if sink.stage == STAGE_GANONK && self.steal_bit != 0 {
            let drop = if sink.life <= ITEM_LOW_LIFE {
                ITEM_HEART
            } else if self.rnd_f(1.0) < 0.5 {
                if sink.arrows == ITEM_NO_ARROWS {
                    ITEM_ARROW
                } else if sink.magic < sink.max_magic / MAGIC_HALF {
                    ITEM_MAGIC
                } else if sink.arrows < ITEM_ARROW_CAP {
                    ITEM_ARROW
                } else {
                    let index = self.rnd_f(ITEM_TABLE_MAX) as usize;
                    ITEM_TABLE.get(index).copied().unwrap_or(ITEM_HEART)
                }
            } else {
                ITEM_NONE13
            };
            self.drop = drop;
        } else {
            sink.se(SE_CM_KS_DIE);
            self.drop = ITEM_NONE13;
        }
        self.doomed = true;
    }

    fn action_dead_move(&mut self, sink: &mut Sink<'_>) {
        match self.mode {
            MODE_DEAD_SPIN => {
                self.clear_swings();
                self.sph.tg_set = false;
                self.sph.co_set = false;
                self.sph.tg_hit = false;
                sink.se(SE_CV_KS_DAMAGE);
                self.speed_f = DEAD_SPEED_F + self.rnd_f(5.0);
                self.gravity = GRAVITY;
                self.speed.y = DEAD_RISE + self.rnd_f(5.0);
                self.mode += 1;
                self.swings[1] = (self.rnd_fx(4096.0)) as i32 as i16;
                self.sph.at_set = false;
                self.dead_bounce(sink);
            }
            MODE_DEAD_BOUNCE => self.dead_bounce(sink),
            MODE_DEAD_BURST => {
                let mut pos = self.pos;
                pos.y += LIFT_DEAD_ALT;
                self.dead_pos = pos;
                self.dead_eff_set(sink);
            }
            _ => {}
        }
    }

    fn dead_bounce(&mut self, sink: &mut Sink<'_>) {
        self.shape_angle[2] = self.shape_angle[2].wrapping_add(self.swings[1]);
        if !self.tyaku_check(None) {
            return;
        }
        self.gravity = GRAVITY;
        self.speed_f *= 0.5;
        match self.swings[0] {
            0 => {
                self.speed.y = 13.0;
                self.swings[0] = 1;
            }
            1 => {
                self.speed.y = 7.0;
                self.swings[0] = 2;
            }
            2 => {
                let mut pos = self.pos;
                pos.y += LIFT_DEAD;
                self.dead_pos = pos;
                self.dead_eff_set(sink);
            }
            _ => {}
        }
    }

    fn action_omoi(&mut self, statics: &mut Statics, env: &Env<'_>, blow: &Blow, sink: &mut Sink<'_>) {
        match self.mode {
            MODE_CLING_ENTER => self.cling_enter(statics, env),
            MODE_CLING_HOLD => self.cling_hold(statics, env),
            MODE_CLING_LEAP => {
                if self.joint_live {
                    self.status &= !STTS_UNK4000;
                    self.joint_live = false;
                }
                self.angle[1] = env.facing(self.pos).wrapping_add(-32768);
                self.angle[1] = self
                    .angle[1]
                    .wrapping_add((self.rnd_fx(16384.0)) as i32 as i16);
                self.speed_f = CLING_LEAP;
                self.gravity = GRAVITY;
                self.speed.y = CLING_CLAW;
                self.mode += 1;
                self.cling_wait();
            }
            MODE_CLING_WAIT => self.cling_wait(),
            _ => {}
        }
        if self.mode == MODE_CLING_WAIT && self.body_atari_check(env, blow) && self.cling != 0
        {
            sink.se(SE_LK_LAST_HIT);
            self.cling = 0;
        }
    }

    fn cling_enter(&mut self, statics: &mut Statics, env: &Env<'_>) {
        self.speed_f = 0.0;
        self.gravity = 0.0;
        self.speed = Vec3::ZERO;
        if statics.goron != 0 || statics.kattuku >= KUTTUKU_MAX {
            self.joint_live = false;
            self.status &= !STTS_UNK4000;
            self.mode = MODE_CLING_LEAP;
            return;
        }
        self.clear_swings();
        if statics.kattuku >= JOINT_LIMIT as i32 {
            self.joint = JOINT_LAST;
        }
        if statics.kattuku < 0 {
            self.joint = 0;
        } else {
            self.joint = statics.kattuku as i16;
        }
        self.joint_slot = self.joint_slot();
        self.joint_live = true;
        if statics.kattuku == 0 {
            statics.goron = 0;
            self.cling = 1;
        }
        statics.kattuku += 1;
        if !statics.heavy && statics.kattuku >= HEAVY_MIN {
            statics.heavy = true;
        }
        self.swing_dir = 0;
        self.timers[1] = 0;
        self.timers[2] = 0;
        self.status |= STTS_UNK4000;
        self.mode += 1;
        self.cling_hold(statics, env);
    }

    fn cling_hold(&mut self, statics: &mut Statics, env: &Env<'_>) {
        let target = env.pos();
        let turn = (self.sweep as f32 * 0.25) as i32 as i16;
        if (self.actor_id & 1) == 0 {
            self.shape_angle[1] = self.shape_angle[1].wrapping_add(turn);
        } else {
            self.shape_angle[1] = self.shape_angle[1].wrapping_sub(turn);
        }
        let mut limit = Vec3::new(
            (env.drift.x * 10.0).abs(),
            (env.drift.y * 10.0).abs(),
            (env.drift.z * 10.0).abs(),
        );
        if limit.x < REG12_F16 + CLING_FOLLOW_MIN {
            limit.x = REG12_F16 + CLING_FOLLOW_MIN;
        }
        if limit.y < REG12_F16 + CLING_FOLLOW_MIN {
            limit.y = REG12_F16 + CLING_FOLLOW_MIN;
        }
        if limit.z < REG12_F16 + CLING_FOLLOW_MIN {
            limit.z = REG12_F16 + CLING_FOLLOW_MIN;
        }
        if span(env.drift) < REG12_F17 + CLING_SLOW {
            limit = Vec3::splat(REG12_F18 + CLING_SLOW);
            let delta = self.pos - target;
            if span(delta) < CLING_TOUCH && self.timers[1] == 0 {
                self.timers[1] = (self.rnd_f(10.0) + 10.0) as i16;
                match self.swing_dir {
                    0 => {
                        if self.joint < JOINT_LAST {
                            self.joint += 1;
                        } else {
                            self.swing_dir = 1;
                            self.joint -= 1;
                        }
                    }
                    _ => {
                        if self.joint > 0 {
                            self.joint -= 1;
                        } else {
                            self.swing_dir = 0;
                            self.joint += 1;
                        }
                    }
                }
                if self.joint >= JOINT_LIMIT {
                    self.joint = JOINT_LAST;
                } else if self.joint < 0 {
                    self.joint = 1;
                }
                self.joint_slot = self.joint_slot();
                self.joint_live = true;
            }
        }
        if env.hooked {
            limit = Vec3::splat(CLING_OVERRIDE);
        }
        add_calc(&mut self.pos.x, target.x, 1.0, limit.x);
        add_calc(&mut self.pos.y, target.y, 1.0, limit.y);
        add_calc(&mut self.pos.z, target.z, 1.0, limit.z);
        if env.rolling_crash || env.spinning || env.swimming {
            self.sph.co_set = true;
            self.sph.tg_set = true;
            if self.cling != 0 {
                statics.heavy = false;
                statics.kattuku = 0;
                statics.goron = 0;
                self.swings[0] = 0;
                self.swings[1] = 0;
            }
            self.mode = MODE_CLING_LEAP;
            return;
        }
        if !env.rolling {
            if self.cling == 0 {
                return;
            }
            if self.swings[1] != 0 {
                self.swings[1] = 0;
                self.timers[2] = CLING_POKE;
            }
            if self.timers[2] != 1 {
                return;
            }
            if i32::from(self.swings[0]) <= 0 {
                return;
            }
            self.swings[0] = 0;
            return;
        }
        if self.cling != 0 {
            if self.swings[1] != 0 {
                return;
            }
            if let Some(rate) = usize::try_from(self.swings[0])
                .ok()
                .and_then(|slot| GORON_RATE.get(slot))
            {
                statics.goron = ((statics.kattuku as f32) * rate) as i32;
            }
            if statics.goron == 0 {
                statics.goron = 1;
            }
            self.swings[0] += 1;
            self.swings[1] = 1;
            if i32::from(self.swings[0]) < 3 && statics.kattuku >= 3 {
                return;
            }
            self.cling = 0;
            statics.heavy = false;
            self.swings[0] = 0;
            self.swings[1] = 0;
            if statics.kattuku > 0 {
                statics.kattuku -= 1;
            }
            statics.goron = statics.kattuku;
            self.mode = MODE_CLING_LEAP;
            return;
        }
        if statics.kattuku != 0 && statics.goron > 0 {
            statics.goron -= 1;
            statics.kattuku -= 1;
            if statics.heavy && statics.kattuku < HEAVY_MIN {
                statics.heavy = false;
            }
            self.mode = MODE_CLING_LEAP;
        }
    }

    fn cling_wait(&mut self) {
        if !self.contact(None) {
            return;
        }
        self.timers[2] = (self.rnd_f(CLING_POKE_MIN) + CLING_POKE_MIN) as i16;
        self.sph.co_set = true;
        self.sph.tg_set = true;
        self.action = ACTION_POKE;
        self.mode = MODE_POKE_FALL;
    }

    fn action_tubo_search(&mut self, env: &Env<'_>, sink: &mut Sink<'_>) {
        match self.mode {
            MODE_TUBO_SEARCH => {
                if self.tsuko.is_none() {
                    self.doomed = true;
                } else {
                    self.mode += 1;
                }
            }
            MODE_TUBO_FOLLOW => {
                if self.tsuko.is_some() {
                    if self.goal_reach == 0.0 {
                        return;
                    }
                    if env.distance(self.pos) > self.goal_reach {
                        return;
                    }
                }
                self.radius = RADIUS_TUBO;
                self.sph.at_vs_other = true;
                self.sph.at_vs_player = false;
                self.sph.at_set = true;
                self.cc_set();
                self.mode += 1;
            }
            MODE_TUBO_SPAWN => {
                self.cc_set();
                if self.count > TUBO_COUNT_MAX || self.count == 0 {
                    self.count = 1;
                }
                let room = self.room;
                let set_id = self.set_id;
                for index in 0..usize::from(self.count) {
                    let mut pos = self.pos;
                    pos.y += self.rnd_f(TUBO_BURST_Y);
                    if index != 0 {
                        pos.x += self.rnd_fx(TUBO_BURST_XZ);
                        pos.z += self.rnd_fx(TUBO_BURST_XZ);
                    }
                    let mut angle = self.angle;
                    angle[0] = 0;
                    sink.child(CHILD_PARAM_TUBO, pos, angle, room, set_id);
                }
                self.doomed = true;
            }
            _ => {}
        }
    }

    fn action_kb_birth_check(&mut self, solid: Option<&Collision>) {
        match self.mode {
            MODE_BIRTH_RISE => {
                self.pos.y += REG8_F13 + LIFT_BIRTH;
                self.angle[1] = (self.rnd_fx(32767.0)) as i32 as i16;
                self.speed_f = REG8_F8 + 4.0 + self.rnd_f(REG8_F9 + 4.0);
                self.speed.y = REG8_F10 + BIRTH_UP + self.rnd_f(REG8_F11 + BIRTH_UP_JITTER);
                self.gravity = -(REG8_F12 + BIRTH_GRAVITY);
                self.mode += 1;
                self.birth_fall(solid);
            }
            MODE_BIRTH_FALL => self.birth_fall(solid),
            _ => {}
        }
    }

    fn birth_fall(&mut self, solid: Option<&Collision>) {
        let near = ground::surface(solid, self.pos)
            .is_some_and(|surface| self.pos.y < surface + REG8_F19 + BIRTH_HEAD);
        if self.speed.y <= 0.0 && (ground::grounded(solid, self.pos) || near) {
            self.status &= !STTS_UNK4000;
            self.speed_f = 0.0;
            self.gravity = 0.0;
            self.speed = Vec3::ZERO;
            self.action = ACTION_DOU;
            self.mode = MODE_DOU_INIT;
        }
    }

    fn enemy_ice(&mut self, sink: &mut Sink<'_>) -> bool {
        if self.light.timer != 0 {
            if self.light.timer == LIGHT_START {
                self.light.timer = self.light.timer.wrapping_add(1);
                sink.se(SE_CM_L_ARROW_SHRINK);
                return true;
            }
            self.light.timer = self.light.timer.wrapping_add(1);
            if self.light.timer < LIGHT_SHRINK_END {
                add_calc0(&mut self.light.scale_xz, LIGHT_SHRINK_STEP, LIGHT_SHRINK_MAX);
                add_calc0(&mut self.light.scale_y, LIGHT_SHRINK_STEP, LIGHT_SHRINK_MAX);
            } else {
                if self.light.timer == LIGHT_SHRINK_END {
                    sink.se(SE_CM_L_ARROW_PASS_AWAY);
                }
                add_calc(
                    &mut self.light.scale_y,
                    LIGHT_RISE_TARGET,
                    LIGHT_RISE_STEP,
                    LIGHT_RISE_MAX,
                );
                add_calc0(&mut self.light.scale_xz, LIGHT_FADE_STEP, LIGHT_FADE_MAX);
                if self.light.timer > LIGHT_GONE {
                    self.health = LIGHT_DEAD;
                    self.doomed = true;
                }
            }
            return true;
        }
        if self.freeze_want != 0 {
            self.freeze = self.freeze_want;
            self.freeze_want = 0;
            self.status |= STTS_UNK8000000;
            self.speed.y = ICE_RISE;
            self.freeze_spin = (self.rnd_fx(ICE_SPIN)) as i32 as i16;
            self.health -= FREEZE_DAMAGE;
            if self.health <= 0 {
                self.freeze_delay = ICE_DELAY;
                self.speed.y = 0.0;
            }
            sink.se(SE_CM_FREEZE);
        }
        if self.freeze > 0 {
            if self.freeze_delay == 0 {
                let (sin, cos) = from_s16(self.angle[1]).sin_cos();
                self.speed.x = self.speed_f * sin;
                self.speed.z = self.speed_f * cos;
                self.pos += self.speed;
                self.speed.y -= ICE_FALL;
                self.shape_angle[1] = self.shape_angle[1].wrapping_add(self.freeze_spin);
            } else {
                self.freeze_delay -= 1;
            }
            self.freeze -= 1;
            return true;
        }
        self.status &= !STTS_UNK8000000;
        false
    }

    fn step(
        &mut self,
        actor: &mut Actor,
        solid: Option<&Collision>,
        env: &mut Env<'_>,
        blow: &Blow,
        statics: &mut Statics,
        sink: &mut Sink<'_>,
    ) {
        self.pos = actor.pos;
        self.cc_calc(env, blow);
        if self.pause_timer != 0 {
            self.pause_timer -= 1;
        }
        if self.fire != 0 {
            self.fire -= 1;
        }
        if self.enemy_ice(sink) {
            if self.ice_flag != 0 {
                self.frame_f += 1.0;
                if self.frame_f > BRK_FRAME_MAX {
                    self.frame_f = BRK_FRAME_MAX;
                }
            }
            return;
        }
        for index in 0..TIMER_COUNT {
            if self.timers[index] != 0 {
                self.timers[index] -= 1;
            }
        }
        if self.gm_link && statics.gm_dead && self.action != ACTION_DEAD {
            if self.action != ACTION_CLING {
                self.action = ACTION_DEAD;
                self.mode = MODE_DEAD_SPIN;
            } else if self.mode != MODE_CLING_WAIT {
                statics.heavy = false;
                statics.goron = 0;
                self.swings[0] = 0;
                self.swings[1] = 0;
                self.mode = MODE_CLING_LEAP;
            }
        }
        match self.action {
            ACTION_DOU => {
                self.action_dousa_move(statics, env, blow, sink);
                self.ks_kuttuki_check(statics);
            }
            ACTION_POKE => self.action_kougeki_move(statics, env, blow, sink),
            ACTION_WIND => self.action_kaze_move(env, blow, sink),
            ACTION_DEAD => self.action_dead_move(sink),
            ACTION_CLING => self.action_omoi(statics, env, blow, sink),
            ACTION_TUBO => self.action_tubo_search(env, sink),
            ACTION_BIRTH => self.action_kb_birth_check(solid),
            _ => {}
        }
        if self.form == FORM_TUBO {
            return;
        }
        if self.frame != 0 {
            self.frame += 1;
            if self.frame > BTK_FRAME_MAX {
                self.frame = 0;
            }
        }
        if self.timers[0] == 0 {
            self.timers[0] = (self.rnd_fx(25.0) + 50.0) as i16;
            self.frame = 1;
        }
        let (sin, cos) = from_s16(self.angle[1]).sin_cos();
        self.speed.x = self.speed_f * sin;
        self.speed.z = self.speed_f * cos;
        if self.mode != MODE_CLING_HOLD && !self.contact(solid) {
            self.speed.y += self.gravity;
            if self.speed.y < FALL_MAX {
                self.speed.y = FALL_MAX;
            }
        }
        self.cc_set();
        self.pos += self.speed;
        if self.mode != MODE_CLING_HOLD {
            self.bg_check(solid);
            self.naraku_check(solid);
        }
        actor.pos = self.pos;
        actor.angle = self.angle;
    }
}
