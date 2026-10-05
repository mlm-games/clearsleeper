use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::{Quat, Vec3};

use super::actor::Actor;
use super::arrow;
use super::boomerang;
use super::bombflower;
use super::carry;
use super::collision::Collision;
use super::combat::{Combat, CutKind};
use super::events::{SimEvent, SimEvents};
use super::flags::{Switches, ZoneActors};
use super::ground;
use super::hammer;
use super::health::Health;
use super::hookshot;
use super::item::Held;
use super::player::Player;
use super::stream;
use super::walk;
use super::water::Water;

pub const ARMOS_ACTORS: [&str; 1] = ["amos2"];

const BYTE_MASK: u32 = 0xFF;
const TYPE_SHIFT: u32 = 0x00;
const AREA_SHIFT: u32 = 0x08;
const INACTIVE_SHIFT: u32 = 0x10;
const SWITCH_SHIFT: u32 = 0x18;

const NO_SWITCH: u8 = 0xFF;
const INACTIVE: u8 = 1;
const AREA_RADIUS_FIXED: f32 = 600.0;
const AREA_RADIUS_STEP: f32 = 100.0;

const HIO_M08_S9: i16 = 0;
const HIO_M08_F2: f32 = 0.0;
const HIO_M08_F3: f32 = 0.0;
const HIO_M08_F10: f32 = 0.0;
const HIO_M08_F11: f32 = 0.0;
const HIO_M12_F0: f32 = 0.0;
const HIO_M12_F19: f32 = 0.0;

pub const BCK_BURUBURU: i32 = 0x6;
pub const BCK_DAMAGE: i32 = 0x7;
pub const BCK_DEAD1: i32 = 0x8;
pub const BCK_DEAD2: i32 = 0x9;
pub const BCK_DEAD3: i32 = 0xA;
pub const BCK_JUMP: i32 = 0xB;
pub const BCK_MAHI: i32 = 0xC;
pub const BCK_SLEEP: i32 = 0xD;
pub const BCK_START: i32 = 0xE;
pub const BCK_WAIT: i32 = 0xF;

const FRAME_BURUBURU: i16 = 12;
const FRAME_DAMAGE: i16 = 20;
const FRAME_DEAD1: i16 = 19;
const FRAME_DEAD2: i16 = 10;
const FRAME_DEAD3: i16 = 9;
const FRAME_JUMP: i16 = 5;
const FRAME_MAHI: i16 = 2;
const FRAME_SLEEP: i16 = 19;
const FRAME_START: i16 = 29;
const FRAME_WAIT: i16 = 2;

const SPIKE_FRAME: f32 = 24.0;

const EMODE_NONE: u8 = 0;
const EMODE_LOOP: u8 = 2;
const STATE_STOP: u8 = 1;
const STATE_LOOP: u8 = 2;

const DOUSA_SETTLE: u8 = 0;
const DOUSA_SCAN: u8 = 1;
const DOUSA_RISE: u8 = 2;
const DOUSA_WAIT: u8 = 3;
const DOUSA_LAND: u8 = 4;
const DOUSA_HOP: u8 = 5;
const DOUSA_SLEEP: u8 = 6;
const DOUSA_WAKE: u8 = 7;
const MAHI_LIMP: u8 = 10;
const MAHI_SETTLE: u8 = 11;
const MAHI_IDLE: u8 = 12;
const MAHI_HELD: u8 = 13;
const MAHI_THROW: u8 = 14;
const MAHI_LAND: u8 = 15;
const ITAI_FLINCH: u8 = 20;
const ITAI_RECOVER: u8 = 21;
const ITAI_DIE: u8 = 22;
const ITAI_LEAP: u8 = 23;
const ITAI_BOUNCE: u8 = 24;
const ITAI_BURST: u8 = 25;
const HANDOU_WINDUP: u8 = 30;
const HANDOU_SLIP: u8 = 31;
const MORI_SHRINK: u8 = 40;
const MORI_HOLD: u8 = 41;
const MORI_GROW: u8 = 42;

const CD_BURST: usize = 0;
const CD_MAHI: usize = 2;
const CD_RECOVER: usize = 3;
const CD_SPLASH: usize = 4;
const CU_HOPS: usize = 0;
const CU_LANDED: usize = 1;
const CD_COUNT: usize = 5;
const CU_COUNT: usize = 4;

const HOPS_MAX: i16 = 8;
const MAHI_FRAMES: i16 = 20 * 30;
const SPLASH_FRAMES: i16 = 1 * 30;
const BURST_FRAMES: i16 = 100;
const RECOVER_GATE: i16 = 3;
const WOUND_CUT: i16 = 5;
const CARRY_LIFT: f32 = 10.0;

const EYE_RADIUS: f32 = 30.0;
const WEAK_RADIUS: f32 = 30.0;
const BODY_RADIUS: f32 = 35.0;
const BODY_HEIGHT: f32 = 150.0;
const NEEDLE_RADIUS: f32 = 55.0;
const NEEDLE_HEIGHT: f32 = 20.0;
const PLAYER_REACH: f32 = 30.0;
const ATTENTION_LIFT: f32 = 120.0;
const EYE_DROP: f32 = 15.0;
const LINE_LIFT: f32 = 100.0;
const FWD_OFFSET: f32 = 200.0;
const HOP_ADJUST: f32 = 200.0;

const COA_OFFSET: [f32; 3] = [0.0, 50.0, -40.0];
const EYE_OFFSET: [f32; 3] = [30.0, 0.0, 0.0];
const TOGE_OFFSET: [f32; 3] = [-10.0, 0.0, 0.0];

const GRAVITY_SLOW: f32 = -3.0;
const GRAVITY_DROP: f32 = -4.0;
const GRAVITY_HARD: f32 = -8.0;
const GRAVITY_SLAM: f32 = -14.0;
const GRAVITY_RISE: f32 = -10.0;
const GRAVITY_CARRY: f32 = -5.0;
const LIFT_SOFT: f32 = 12.0;
const LIFT_HIGH: f32 = 20.0;
const LIFT_LEAP: f32 = 25.0;
const SPEED_SLOW: f32 = 7.0;
const SPEED_LEAP: f32 = 10.0;
const SPEED_RETREAT: f32 = 9.0;
const SPEED_WINDUP: f32 = 20.0;
const SPEED_THROW: f32 = 35.0;
const SPEED_SLIP: f32 = 40.0;
const SPEED_WALK: f32 = 9.0;
const MAX_FALL: f32 = -50.0;

const TURN_DIV: i16 = 1;
const TURN_CAP: i16 = 0x500;
const CARRY_TURN_CAP: i16 = 0x1000;
const TURN_REEL: i16 = 0x1000;
const HALF_TURN: i16 = -0x8000;
const QUARTER_TURN: i16 = 0x4000;
const WEAK_CONE: i16 = 0x4000;
const LAND_CONE: i16 = 0x1000;

const SPEED_EPS: f32 = 0.2;
const SLIP_EPS: f32 = 0.1;
const DECEL_HALF: f32 = 0.5;
const DECEL_CAP: f32 = 1.0;
const SLIP_DECEL: f32 = 0.8;
const SLIP_DECEL_CAP: f32 = 2.0;
const WOUND_SLOW: f32 = 5.0;

const SCALE_GATE: f32 = 0.1;
const SCALE_STEP: f32 = 0.1;
const SCALE_DONE: f32 = 0.9;

const ABYSS_Y: f32 = -500.0;
const ABYSS_LIMIT: u8 = 50;
const UNDERWATER: f32 = 80.0;

const HEALTH_MAX: i32 = 2;
const HEALTH: i32 = 2;
const STEAL_ITEMS: i16 = 3;
const SPIKE_POWER: u8 = 1;
const LIGHT_DEAD: i32 = -0x80;

const LIGHT_START: i8 = 1;
const LIGHT_SHRINK_END: i8 = 70;
const LIGHT_GONE: i8 = 90;
const LIGHT_SCALE: f32 = 1.0;
const LIGHT_LIFT: f32 = 80.0;
const LIGHT_SHRINK_SCALE: f32 = 0.1;
const LIGHT_SHRINK_MAX: f32 = 0.01;
const LIGHT_RISE_TARGET: f32 = 5.0;
const LIGHT_RISE_SCALE: f32 = 0.1;
const LIGHT_RISE_MAX: f32 = 1.0;
const LIGHT_FADE_SCALE: f32 = 0.1;
const LIGHT_FADE_MAX: f32 = 0.05;

const HIT_NONE: u8 = 0;
const HIT_CRIT: u8 = 1;
const HIT_BOOMERANG: u8 = 4;
const HIT_ARROW: u8 = 5;
const HIT_BOMB: u8 = 6;
const HIT_HAMMER: u8 = 7;
const HIT_HAMMER_TURN: u8 = 8;

const REACH_SWORD: f32 = 20.0;
const REACH_ARROW: f32 = 35.0;
const REACH_HAMMER: f32 = 50.0;
const REACH_BOOMERANG: f32 = 90.0;
const REACH_BOMB: f32 = 200.0;
const REACH_HOOK: f32 = 5.0;

const SE_SWORD_HIT_S: u32 = 0x2803;
const SE_LAST_HIT: u32 = 0x2828;
const SE_WIND_HIT: u32 = 0x2833;
const SE_MS_HIT: u32 = 0x2834;
const SE_HAMMER_HIT: u32 = 0x2855;
const SE_JITABATA: u32 = 0x4879;
const SE_AWAKE: u32 = 0x48B3;
const SE_PARALYZED: u32 = 0x48B5;
const SE_DAMAGE: u32 = 0x48B6;
const SE_SLIP: u32 = 0x50BB;
const SE_JUMP_S: u32 = 0x5882;
const SE_JUMP: u32 = 0x58B2;
const SE_LANDING: u32 = 0x58B3;
const SE_CM_PARALYZED: u32 = 0x58B4;
const SE_RECOVER: u32 = 0x58B5;
const SE_JUMP2: u32 = 0x58B6;
const SE_BEFORE_EXPLODE: u32 = 0x58B7;
const SE_EXPLODE: u32 = 0x58B8;
const SE_SPIKE_OUT: u32 = 0x58B9;
const SE_SPIKE_IN: u32 = 0x58BA;
const SE_L_SHRINK: u32 = 0x5904;
const SE_L_PASS: u32 = 0x5905;
const SE_WATER_RECOVER: u32 = 0x5946;
const SE_FALL_WATER: u32 = 0x6919;

pub const CULL_MIN: [f32; 3] = [-50.0, 0.0, -20.0];
pub const CULL_MAX: [f32; 3] = [60.0, 180.0, 60.0];

fn frame_max(bck: i32) -> i16 {
    match bck {
        BCK_BURUBURU => FRAME_BURUBURU,
        BCK_DAMAGE => FRAME_DAMAGE,
        BCK_DEAD1 => FRAME_DEAD1,
        BCK_DEAD2 => FRAME_DEAD2,
        BCK_DEAD3 => FRAME_DEAD3,
        BCK_JUMP => FRAME_JUMP,
        BCK_MAHI => FRAME_MAHI,
        BCK_SLEEP => FRAME_SLEEP,
        BCK_START => FRAME_START,
        _ => FRAME_WAIT,
    }
}

fn distance_angle_s(x: i16, y: i16) -> i16 {
    x.wrapping_sub(y).unsigned_abs() as i16
}

fn turned(yaw: i16, reach: f32) -> Vec3 {
    let (sin, cos) = walk::from_s16(yaw).sin_cos();
    Vec3::new(reach * sin, 0.0, reach * cos)
}

fn joint(actor: &Actor, offset: [f32; 3]) -> Vec3 {
    let turn = Quat::from_rotation_y(walk::from_s16(actor.angle[1]))
        * Quat::from_rotation_x(walk::from_s16(actor.angle[0]))
        * Quat::from_rotation_z(walk::from_s16(actor.angle[2]));
    actor.pos + turn * Vec3::new(offset[0], offset[1], offset[2])
}

fn touches(centre: Vec3, radius: f32, at: Vec3, reach: f32) -> bool {
    walk::within(centre, at, radius + reach)
}

fn cyl_touches(centre: Vec3, radius: f32, height: f32, at: Vec3, reach: f32) -> bool {
    let dx = at.x - centre.x;
    let dz = at.z - centre.z;
    let sum = radius + reach;
    if sum * sum < dx * dx + dz * dz {
        return false;
    }
    at.y + reach >= centre.y && at.y - reach <= centre.y + height
}

fn add_calc0(value: &mut f32, scale: f32, max_step: f32) {
    let step = (*value * scale).clamp(-max_step, max_step);
    *value -= step;
}

fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value != target {
        let step = (scale * (target - *value)).clamp(-max_step, max_step);
        *value += step;
    }
}

fn search_angle(from: Vec3, to: Vec3) -> i16 {
    walk::to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn search_distance(from: Vec3, to: Vec3) -> f32 {
    (to - from).length()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Dousa,
    Mahi,
    Itai,
    HandouMove,
    ModoruMove,
}

impl Step {
    pub fn code(self) -> u8 {
        match self {
            Self::Dousa => 0,
            Self::Mahi => 1,
            Self::Itai => 2,
            Self::HandouMove => 3,
            Self::ModoruMove => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum At {
    Idle,
    Sword,
    Hook,
    Light,
    Arrow,
    FireArrow,
    IceArrow,
    Hammer,
    Bomb,
    Boomerang,
}

impl At {
    fn reach(self) -> f32 {
        match self {
            Self::Idle => 0.0,
            Self::Sword => REACH_SWORD,
            Self::Hook => REACH_HOOK,
            Self::Light | Self::Arrow | Self::FireArrow | Self::IceArrow => REACH_ARROW,
            Self::Hammer => REACH_HAMMER,
            Self::Bomb => REACH_BOMB,
            Self::Boomerang => REACH_BOOMERANG,
        }
    }

    fn arrow(self) -> bool {
        matches!(
            self,
            Self::Light | Self::Arrow | Self::FireArrow | Self::IceArrow
        )
    }

    fn power(self, blade: u8, soup: bool) -> i32 {
        match self {
            Self::Sword => super::damage::sword_power(blade, soup),
            Self::Arrow | Self::FireArrow | Self::IceArrow => super::damage::ARROW,
            Self::Bomb => super::damage::BOMB,
            Self::Boomerang => super::damage::BOMERANG,
            Self::Hammer => hammer::AT_POWER,
            Self::Light | Self::Hook | Self::Idle => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blow {
    pub at: At,
    pub blade: u8,
    pub soup: bool,
    pub crit: bool,
    pub turn: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub attribute: u8,
    pub state: u8,
    pub start: i16,
    pub end: i16,
    pub loop_point: i16,
    pub rate: f32,
    pub frame: f32,
    pub cur_morf: f32,
    pub prev_morf: f32,
    pub morf_step: f32,
}

impl Frame {
    fn idle() -> Self {
        Self {
            attribute: EMODE_LOOP,
            state: 0,
            start: 0,
            end: 0,
            loop_point: 0,
            rate: 1.0,
            frame: 0.0,
            cur_morf: 1.0,
            prev_morf: 1.0,
            morf_step: 0.0,
        }
    }

    fn init(&mut self, end: i16) {
        self.attribute = EMODE_LOOP;
        self.state = 0;
        self.start = 0;
        self.end = end;
        self.loop_point = 0;
        self.rate = 1.0;
        self.frame = 0.0;
    }

    fn set_start(&mut self, start: i16) {
        self.start = start;
        self.frame = (start) as f32;
    }

    fn set_loop(&mut self, frame: f32) {
        self.loop_point = frame as i16;
    }

    fn set_morf(&mut self, morf: f32) {
        if self.prev_morf < 0.0 || morf <= 0.0 {
            self.cur_morf = 1.0;
        } else {
            self.cur_morf = 0.0;
            self.morf_step = 1.0 / morf;
        }
        self.prev_morf = self.cur_morf;
    }

    fn update(&mut self) {
        self.state = 0;
        self.frame += self.rate;
        let start = (self.start) as f32;
        let end = (self.end) as f32;
        match self.attribute {
            EMODE_NONE => {
                if self.frame < start {
                    self.frame = start;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
                if self.frame >= end {
                    self.frame = end - 0.001;
                    self.rate = 0.0;
                    self.state |= STATE_STOP;
                }
            }
            EMODE_LOOP => {
                let low = i32::from(self.loop_point) - i32::from(self.start);
                let high = i32::from(self.end) - i32::from(self.loop_point);
                while self.frame < start {
                    self.state |= STATE_LOOP;
                    if low <= 0 {
                        break;
                    }
                    self.frame += (low) as f32;
                }
                while self.frame >= end {
                    self.state |= STATE_LOOP;
                    if high <= 0 {
                        break;
                    }
                    self.frame -= (high) as f32;
                }
            }
            _ => {}
        }
    }

    fn play(&mut self) {
        if self.cur_morf < 1.0 {
            self.prev_morf = self.cur_morf;
            walk::chase_f(&mut self.cur_morf, 1.0, self.morf_step);
        }
        self.update();
    }

    fn is_stop(&self) -> bool {
        self.state & STATE_STOP != 0 || self.rate == 0.0
    }

    fn check_pass(&self, pass: f32) -> bool {
        let start = (self.start) as f32;
        let end = (self.end) as f32;
        let cur = self.frame;
        let mut next = cur + self.rate;
        if next < start {
            next = start;
        }
        if next >= end {
            next = end - 0.001;
        }
        if cur <= next {
            cur <= pass && pass < next
        } else {
            next <= pass && pass < cur
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Shape {
    pub eye_tg: bool,
    pub eye_hit: bool,
    pub weak_tg: bool,
    pub weak_hit: bool,
    pub body_tg: bool,
    pub body_hit: bool,
    pub body_co: bool,
    pub needle_at: bool,
    pub needle_hit: bool,
    pub needle_tg: bool,
    pub needle_tghit: bool,
    pub needle_guard: bool,
}

impl Shape {
    fn needle_arm(&mut self) {
        self.needle_at = true;
        self.needle_hit = true;
        self.needle_tg = true;
    }

    fn needle_sheath(&mut self) {
        self.needle_at = false;
        self.needle_hit = false;
        self.needle_tg = false;
        self.needle_tghit = false;
    }
}

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
            y_offset: LIGHT_LIFT,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Armos {
    pub kind: u8,
    pub prm_radius: u8,
    pub starts_inactive: u8,
    pub switch_no: u8,
    pub step: Step,
    pub mode: u8,
    pub hit_code: u8,
    pub weak_being_hit: bool,
    pub not_in_home_room: bool,
    pub water_splash: bool,
    pub abyss_timer: u8,
    pub lock_on: bool,
    pub carry_ok: bool,
    pub doomed: bool,
    pub entered: bool,
    pub carried: bool,
    pub guarding: bool,
    pub damage_wait: i16,
    pub bck: i32,
    pub morph: Frame,
    pub countdown: [i16; CD_COUNT],
    pub countup: [i16; CU_COUNT],
    pub target_angle: i16,
    pub spawn_rot: i16,
    pub heading: i16,
    pub eyeball: Vec3,
    pub needle: Vec3,
    pub weak: Vec3,
    pub attention: Vec3,
    pub rest_pos: Vec3,
    pub spawn_pos: Vec3,
    pub lin_centre: Vec3,
    pub lin_dest: Vec3,
    pub eye_y: f32,
    pub area_radius: f32,
    pub acch_radius: f32,
    pub picked_up_y: f32,
    pub scale: Vec3,
    pub speed: Vec3,
    pub speed_f: f32,
    pub gravity: f32,
    pub health: i32,
    pub max_health: i32,
    pub steal_left: i16,
    pub light: Light,
    pub shapes: Shape,
}

impl Armos {
    pub fn spawn(parameters: u32) -> Self {
        let mut kind = ((parameters >> TYPE_SHIFT) & BYTE_MASK) as u8;
        let prm_radius = ((parameters >> AREA_SHIFT) & BYTE_MASK) as u8;
        let mut starts_inactive = ((parameters >> INACTIVE_SHIFT) & BYTE_MASK) as u8;
        let switch_no = ((parameters >> SWITCH_SHIFT) & BYTE_MASK) as u8;
        if kind == NO_SWITCH {
            kind = 0;
        }
        if starts_inactive == NO_SWITCH {
            starts_inactive = 0;
        }
        if HIO_M08_S9 != 0 {
            kind = 1;
        }
        let area_radius = if prm_radius == NO_SWITCH || prm_radius == 0 {
            AREA_RADIUS_FIXED
        } else {
            (prm_radius) as f32 * AREA_RADIUS_STEP
        };
        Self {
            kind,
            prm_radius,
            starts_inactive,
            switch_no,
            step: Step::Dousa,
            mode: DOUSA_SETTLE,
            hit_code: HIT_NONE,
            weak_being_hit: false,
            not_in_home_room: false,
            water_splash: false,
            abyss_timer: 0,
            lock_on: false,
            carry_ok: false,
            doomed: false,
            entered: false,
            carried: false,
            guarding: false,
            damage_wait: 0,
            bck: 0,
            morph: Frame::idle(),
            countdown: [0; CD_COUNT],
            countup: [0; CU_COUNT],
            target_angle: 0,
            spawn_rot: 0,
            heading: 0,
            eyeball: Vec3::ZERO,
            needle: Vec3::ZERO,
            weak: Vec3::ZERO,
            attention: Vec3::ZERO,
            rest_pos: Vec3::ZERO,
            spawn_pos: Vec3::ZERO,
            lin_centre: Vec3::ZERO,
            lin_dest: Vec3::ZERO,
            eye_y: 0.0,
            area_radius,
            acch_radius: 40.0 + HIO_M08_F10,
            picked_up_y: 0.0,
            scale: Vec3::ONE,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            gravity: GRAVITY_SLOW,
            health: HEALTH,
            max_health: HEALTH_MAX,
            steal_left: STEAL_ITEMS,
            light: Light::idle(),
            shapes: Shape {
                body_tg: true,
                body_co: true,
                ..Shape::default()
            },
        }
    }

    pub fn present(&self, switches: &Switches) -> bool {
        self.starts_inactive == INACTIVE
            || self.switch_no == NO_SWITCH
            || !switches.test(self.switch_no)
    }

    pub fn begin(&mut self, actor: &Actor, switches: &Switches) {
        self.entered = true;
        self.rest_pos = actor.pos;
        self.spawn_pos = actor.pos;
        self.spawn_rot = actor.angle[1];
        self.target_angle = actor.angle[1];
        self.heading = actor.angle[1];
        if self.dormant(switches) {
            self.lock_on = false;
            self.countdown[CD_MAHI] = MAHI_FRAMES;
            self.step = Step::Mahi;
            self.mode = MAHI_IDLE;
        }
        self.acch_radius = 40.0 + HIO_M08_F10;
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.scale.x > 0.0
    }

    pub fn anm_init(&mut self, bck: i32, morf: f32, loop_mode: u8, speed: f32) {
        self.bck = bck;
        self.morph.set_start(0);
        self.morph.init(frame_max(bck));
        self.morph.attribute = loop_mode;
        self.morph.rate = speed;
        self.morph.frame = 0.0;
        self.morph.set_loop(self.morph.frame);
        self.morph.set_morf(morf);
    }

    pub fn dormant(&self, switches: &Switches) -> bool {
        self.starts_inactive == INACTIVE
            && self.switch_no != NO_SWITCH
            && !switches.test(self.switch_no)
    }

    fn line_check(&mut self, actor: &Actor, dest: Vec3, solid: Option<&Collision>) -> bool {
        let centre = Vec3::new(
            actor.pos.x,
            actor.pos.y + LINE_LIFT + HIO_M12_F19,
            actor.pos.z,
        );
        self.lin_centre = centre;
        self.lin_dest = dest;
        match solid {
            Some(solid) => solid.slide(centre, dest) == dest,
            None => true,
        }
    }

    fn tick_timers(&mut self) {
        for slot in self.countdown.iter_mut() {
            if *slot != 0 {
                *slot -= 1;
            }
        }
    }

    fn clear_countup(&mut self) {
        for slot in self.countup.iter_mut() {
            *slot = 0;
        }
    }

    fn resolve(&mut self, actor: &Actor, player: &Player, at: At, blast: bool) {
        let at = if blast && !matches!(at, At::Hook | At::Light) {
            At::Bomb
        } else {
            at
        };
        self.eyeball = joint(actor, EYE_OFFSET);
        self.needle = joint(actor, TOGE_OFFSET);
        self.weak = joint(actor, COA_OFFSET);
        self.eye_y = self.eyeball.y - (EYE_DROP + HIO_M08_F2);
        self.attention = Vec3::new(actor.pos.x, actor.pos.y + ATTENTION_LIFT, actor.pos.z);
        let reach = at.reach();
        let origin = player.pos;
        self.shapes.eye_hit = false;
        self.shapes.weak_hit = false;
        self.shapes.body_hit = false;
        self.shapes.needle_hit = false;
        self.shapes.needle_guard = false;
        if at == At::Idle {
            return;
        }
        self.shapes.body_co = self.shapes.body_co
            && cyl_touches(actor.pos, BODY_RADIUS, BODY_HEIGHT, origin, PLAYER_REACH);
        if self.shapes.eye_tg && (at.arrow() || at == At::Hook) {
            self.shapes.eye_hit = touches(self.eyeball, EYE_RADIUS, origin, reach);
        }
        if self.shapes.weak_tg && at != At::Light {
            self.shapes.weak_hit = touches(self.weak, WEAK_RADIUS, origin, reach);
        }
        if self.shapes.body_tg && at != At::Light {
            self.shapes.body_hit =
                cyl_touches(actor.pos, BODY_RADIUS, BODY_HEIGHT, origin, reach);
        }
        if self.shapes.needle_at {
            self.shapes.needle_hit = cyl_touches(
                self.needle,
                NEEDLE_RADIUS + HIO_M08_F3,
                NEEDLE_HEIGHT,
                origin,
                PLAYER_REACH,
            );
            self.shapes.needle_guard = self.guarding && self.shapes.needle_hit;
        }
    }

    fn spike(&mut self, health: &mut Health) {
        if self.shapes.needle_hit {
            health.damage(SPIKE_POWER);
        }
    }

    fn enemy_ice(&mut self, events: &mut SimEvents) -> bool {
        if self.light.timer == 0 {
            return false;
        }
        if self.light.timer == LIGHT_START {
            self.light.timer = self.light.timer.wrapping_add(1);
            events.list.push(SimEvent::Se(SE_L_SHRINK));
            return true;
        }
        self.light.timer = self.light.timer.wrapping_add(1);
        if self.light.timer < LIGHT_SHRINK_END {
            add_calc0(
                &mut self.light.scale_xz,
                LIGHT_SHRINK_SCALE,
                LIGHT_SHRINK_MAX,
            );
            add_calc0(
                &mut self.light.scale_y,
                LIGHT_SHRINK_SCALE,
                LIGHT_SHRINK_MAX,
            );
        } else {
            if self.light.timer == LIGHT_SHRINK_END {
                events.list.push(SimEvent::Se(SE_L_PASS));
            }
            add_calc2(
                &mut self.light.scale_y,
                LIGHT_RISE_TARGET,
                LIGHT_RISE_SCALE,
                LIGHT_RISE_MAX,
            );
            add_calc0(&mut self.light.scale_xz, LIGHT_FADE_SCALE, LIGHT_FADE_MAX);
            if self.light.timer > LIGHT_GONE {
                self.health = LIGHT_DEAD;
            }
        }
        true
    }

    fn medama_atari_check(
        &mut self,
        at: At,
        switches: &Switches,
        events: &mut SimEvents,
    ) -> bool {
        if self.dormant(switches) {
            return false;
        }
        if !self.shapes.eye_hit {
            return false;
        }
        if at == At::Hook {
            if self.bck != BCK_SLEEP {
                events.list.push(SimEvent::Se(SE_MS_HIT));
            }
            return true;
        }
        match at {
            At::Light => {
                self.light.timer = LIGHT_START;
                self.light.particle_scale = LIGHT_SCALE;
                self.light.y_offset = LIGHT_LIFT;
                self.lock_on = false;
                true
            }
            At::Arrow | At::FireArrow | At::IceArrow => {
                if self.bck == BCK_SLEEP {
                    self.anm_init(BCK_WAIT, 1.0, EMODE_NONE, 1.0);
                    self.lock_on = true;
                    self.shapes.needle_arm();
                    self.step = Step::Dousa;
                    self.mode = DOUSA_RISE;
                } else {
                    events.list.push(SimEvent::Se(SE_CM_PARALYZED));
                    events.list.push(SimEvent::Se(SE_PARALYZED));
                    self.step = Step::Mahi;
                    self.mode = MAHI_LIMP;
                }
                true
            }
            _ => false,
        }
    }

    fn week_atari_check(
        &mut self,
        actor: &Actor,
        player: &Player,
        blow: Blow,
        events: &mut SimEvents,
    ) -> bool {
        let target = search_angle(actor.pos, player.pos);
        if distance_angle_s(actor.angle[1], target) < WEAK_CONE {
            return false;
        }
        self.hit_code = HIT_NONE;
        if !self.shapes.weak_hit {
            self.weak_being_hit = false;
            return false;
        }
        if self.weak_being_hit {
            return false;
        }
        self.weak_being_hit = true;
        match blow.at {
            At::Sword => {
                events.list.push(SimEvent::Se(SE_SWORD_HIT_S));
                if blow.crit {
                    self.hit_code = HIT_CRIT;
                }
            }
            At::Hammer => {
                events.list.push(SimEvent::Se(SE_HAMMER_HIT));
                self.hit_code = if blow.turn {
                    HIT_HAMMER_TURN
                } else {
                    HIT_HAMMER
                };
            }
            At::Bomb => self.hit_code = HIT_BOMB,
            At::Boomerang => {
                self.hit_code = HIT_BOOMERANG;
                return false;
            }
            At::Idle | At::Hook => events.list.push(SimEvent::Se(SE_MS_HIT)),
            At::Arrow | At::FireArrow | At::IceArrow => {
                self.hit_code = HIT_ARROW;
                events.list.push(SimEvent::Se(SE_MS_HIT));
            }
            At::Light => {}
        }
        let power = blow.at.power(blow.blade, blow.soup);
        if power > 0 {
            self.health -= power;
        }
        if self.health <= 0 {
            events.list.push(SimEvent::Se(SE_LAST_HIT));
        }
        self.step = Step::Itai;
        self.mode = ITAI_FLINCH;
        if self.hit_code == HIT_HAMMER || self.hit_code == HIT_HAMMER_TURN {
            self.health = 0;
        }
        true
    }

    fn body_atari_check(&mut self, blow: Blow, events: &mut SimEvents) -> bool {
        self.hit_code = HIT_NONE;
        if !self.shapes.body_hit {
            return false;
        }
        match blow.at {
            At::Sword => events.list.push(SimEvent::Se(SE_SWORD_HIT_S)),
            At::Boomerang => events.list.push(SimEvent::Se(SE_WIND_HIT)),
            At::Hammer => {
                events.list.push(SimEvent::Se(SE_HAMMER_HIT));
                if self.step == Step::HandouMove || self.step == Step::Itai {
                    return true;
                }
                self.step = Step::HandouMove;
                self.mode = HANDOU_WINDUP;
                self.hit_code = if blow.turn {
                    HIT_HAMMER_TURN
                } else {
                    HIT_HAMMER
                };
            }
            _ => events.list.push(SimEvent::Se(SE_MS_HIT)),
        }
        true
    }

    fn naraku_check(
        &mut self,
        actor: &Actor,
        solid: Option<&Collision>,
        wet: Option<&Water>,
        events: &mut SimEvents,
    ) -> bool {
        if ground::surface(solid, actor.pos).is_none() {
            self.abyss_timer = self.abyss_timer.wrapping_add(1);
            if actor.pos.y < ABYSS_Y || self.abyss_timer > ABYSS_LIMIT {
                self.speed_f = 0.0;
                self.speed = Vec3::ZERO;
                self.gravity = 0.0;
                return true;
            }
        }
        if let Some(water) = wet.filter(|water| water.in_area) {
            if !self.water_splash && self.countdown[CD_SPLASH] == 0 {
                self.countdown[CD_SPLASH] = SPLASH_FRAMES;
                events.list.push(SimEvent::Se(SE_FALL_WATER));
                self.water_splash = true;
            }
            if actor.pos.y < water.surface - (UNDERWATER + HIO_M12_F0) {
                self.speed_f = 0.0;
                self.speed = Vec3::ZERO;
                self.gravity = 0.0;
                return true;
            }
        } else if self.water_splash {
            self.water_splash = false;
        }
        false
    }

    fn fallen(&mut self) {
        if self.not_in_home_room {
            self.doomed = true;
            return;
        }
        self.step = Step::ModoruMove;
        self.mode = MORI_SHRINK;
    }

    pub fn action_dousa(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        solid: Option<&Collision>,
        wet: Option<&Water>,
        blow: Blow,
        switches: &Switches,
        events: &mut SimEvents,
    ) {
        let mut rot_offset = Vec3::ZERO;
        if self.mode == DOUSA_LAND || self.mode == DOUSA_HOP {
            rot_offset =
                turned(self.heading, FWD_OFFSET) + Vec3::new(0.0, LINE_LIFT + HIO_M12_F19, 0.0);
        }
        let landed = ground::grounded(solid, actor.pos);
        let dist = search_distance(actor.pos, player.pos);
        let mut run = true;
        while run {
            run = false;
            match self.mode {
                DOUSA_SETTLE => {
                    self.clear_countup();
                    self.acch_radius = 80.0 + HIO_M08_F11;
                    if self.bck != BCK_WAIT {
                        self.anm_init(BCK_WAIT, 10.0, EMODE_NONE, 1.0);
                    }
                    self.mode = DOUSA_SCAN;
                    run = true;
                }
                DOUSA_SCAN => {
                    if dist < self.area_radius {
                        let dest = player.pos + Vec3::new(0.0, LINE_LIFT + HIO_M12_F19, 0.0);
                        if self.line_check(actor, dest, solid) {
                            self.lock_on = true;
                            self.anm_init(BCK_START, 1.0, EMODE_NONE, 1.0);
                            events.list.push(SimEvent::Se(SE_AWAKE));
                            self.shapes.eye_tg = true;
                            self.shapes.weak_tg = true;
                            self.mode = DOUSA_RISE;
                        }
                    }
                }
                DOUSA_RISE => {
                    if self.morph.check_pass(SPIKE_FRAME) {
                        events.list.push(SimEvent::Se(SE_SPIKE_OUT));
                    }
                    if self.morph.is_stop() {
                        self.clear_countup();
                        self.speed_f = 0.0;
                        if dist > self.area_radius + HOP_ADJUST {
                            self.mode = DOUSA_SLEEP;
                            self.anm_init(BCK_SLEEP, 1.0, EMODE_NONE, 1.0);
                            events.list.push(SimEvent::Se(SE_SPIKE_IN));
                            self.shapes.weak_tg = false;
                        } else {
                            self.shapes.needle_arm();
                            self.target_angle = search_angle(actor.pos, player.pos);
                            self.gravity = GRAVITY_SLOW;
                            self.speed.y = LIFT_SOFT;
                            if self.bck != BCK_JUMP {
                                self.anm_init(BCK_JUMP, 2.0, EMODE_LOOP, 1.0);
                            }
                            self.mode = DOUSA_LAND;
                        }
                    }
                }
                DOUSA_WAIT => {}
                DOUSA_LAND => {
                    if landed {
                        self.gravity = GRAVITY_SLOW;
                        self.speed.y = LIFT_SOFT;
                        events.list.push(SimEvent::Se(SE_LANDING));
                        self.mode = DOUSA_WAIT;
                        if distance_angle_s(actor.angle[1], self.heading) < LAND_CONE
                            && (self.line_check(actor, rot_offset, solid) || self.damage_wait == 0)
                        {
                            self.mode = DOUSA_HOP;
                        }
                    }
                }
                DOUSA_HOP => {
                    if self.speed_f > 0.0 && self.shapes.needle_guard {
                        self.speed_f = SPEED_RETREAT * -1.0;
                        self.speed.y = 0.0;
                    }
                    if landed {
                        events.list.push(SimEvent::Se(SE_LANDING));
                        events.list.push(SimEvent::Se(SE_JUMP_S));
                        if self.countup[CU_HOPS] > HOPS_MAX {
                            self.mode = DOUSA_WAIT;
                        } else {
                            self.speed_f = SPEED_WALK;
                            self.gravity = GRAVITY_HARD;
                            self.speed.y = LIFT_HIGH;
                            self.countup[CU_LANDED] = 0;
                            if !self.line_check(actor, rot_offset, solid) || self.damage_wait != 0
                            {
                                self.speed_f = 0.0;
                            }
                            self.countup[CU_HOPS] += 1;
                        }
                    }
                }
                DOUSA_SLEEP => {
                    self.anm_init(BCK_SLEEP, 1.0, EMODE_NONE, 1.0);
                    events.list.push(SimEvent::Se(SE_SPIKE_IN));
                    self.shapes.weak_tg = false;
                    self.shapes.weak_hit = false;
                    self.mode = DOUSA_WAKE;
                    run = true;
                }
                DOUSA_WAKE => {
                    if self.morph.is_stop() {
                        self.clear_countup();
                        self.shapes.needle_sheath();
                        self.rest_pos = actor.pos;
                        self.step = Step::Dousa;
                        self.mode = DOUSA_SCAN;
                    }
                }
                _ => {}
            }
        }
        stream::add_calc_angle_s2(&mut actor.angle[1], self.target_angle, TURN_DIV, TURN_CAP);
        stream::add_calc_angle_s2(&mut self.heading, actor.angle[1], TURN_DIV, TURN_CAP);
        if self.naraku_check(actor, solid, wet, events) {
            self.fallen();
        } else if !self.medama_atari_check(blow.at, switches, events)
            && (self.mode < DOUSA_WAIT || !self.week_atari_check(actor, player, blow, events))
        {
            self.body_atari_check(blow, events);
        }
    }

    pub fn action_mahi(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        solid: Option<&Collision>,
        wet: Option<&Water>,
        blow: Blow,
        switches: &Switches,
        events: &mut SimEvents,
    ) {
        let landed = ground::grounded(solid, actor.pos);
        match self.mode {
            MAHI_LIMP => {
                self.clear_countup();
                self.shapes.needle_sheath();
                self.target_angle = search_angle(actor.pos, player.pos).wrapping_add(HALF_TURN);
                self.heading = self.target_angle;
                self.countdown[CD_MAHI] = 0;
                self.countdown[CD_RECOVER] = 0;
                self.speed_f = SPEED_WINDUP;
                events.list.push(SimEvent::Se(SE_SPIKE_IN));
                self.anm_init(BCK_DAMAGE, 1.0, EMODE_NONE, 1.0);
                self.mode = MAHI_SETTLE;
            }
            MAHI_SETTLE => {
                add_calc0(&mut self.speed_f, DECEL_HALF, DECEL_CAP);
                if self.speed_f < SPEED_EPS && self.morph.is_stop() {
                    self.speed_f = 0.0;
                    self.countdown[CD_MAHI] = MAHI_FRAMES;
                    self.anm_init(BCK_MAHI, 1.0, EMODE_LOOP, 1.0);
                    self.mode = MAHI_IDLE;
                }
            }
            MAHI_IDLE => {
                if self.countdown[CD_RECOVER] == 0 || self.countdown[CD_RECOVER] > RECOVER_GATE {
                    self.carry_ok = true;
                }
                if self.naraku_check(actor, solid, wet, events) {
                    self.fallen();
                } else {
                    if self.countup[CU_LANDED] != 0 && landed {
                        events.list.push(SimEvent::Se(SE_JUMP));
                        self.countup[CU_LANDED] = 0;
                    }
                    if self.carried {
                        self.acch_radius = 40.0 + HIO_M08_F10;
                        self.water_splash = false;
                        self.picked_up_y = actor.pos.y;
                        self.lock_on = false;
                        self.heading = walk::to_s16(player.face);
                        self.gravity = 0.0;
                        self.speed = Vec3::ZERO;
                        self.speed_f = 0.0;
                        self.not_in_home_room = false;
                        self.shapes.body_co = false;
                        self.mode = MAHI_HELD;
                    }
                }
            }
            MAHI_HELD => {
                self.heading = walk::to_s16(player.face);
                if self.picked_up_y + CARRY_LIFT <= actor.pos.y {
                    stream::add_calc_angle_s2(
                        &mut actor.angle[1],
                        self.heading,
                        TURN_DIV,
                        CARRY_TURN_CAP,
                    );
                }
                if !self.carried {
                    self.acch_radius = 40.0 + HIO_M08_F10;
                    self.shapes.body_co = true;
                    if self.speed_f > 0.0 {
                        self.gravity = GRAVITY_CARRY;
                        self.speed.y = LIFT_LEAP;
                        self.speed_f = SPEED_THROW;
                        self.mode = MAHI_THROW;
                    } else {
                        self.gravity = GRAVITY_SLOW;
                        self.countup[CU_LANDED] = 1;
                        self.mode = MAHI_IDLE;
                    }
                }
            }
            MAHI_THROW => {
                if self.naraku_check(actor, solid, wet, events) {
                    self.fallen();
                } else {
                    if landed {
                        if self.countup[CU_HOPS] == 0 {
                            events.list.push(SimEvent::Se(SE_LANDING));
                            self.speed_f = SPEED_SLOW;
                            self.speed.y = LIFT_LEAP;
                            self.gravity = GRAVITY_SLAM;
                            self.countup[CU_HOPS] += 1;
                        } else {
                            self.speed_f = 0.0;
                            self.countup[CU_HOPS] = 0;
                            self.mode = MAHI_IDLE;
                        }
                    }
                    add_calc0(&mut self.speed_f, DECEL_HALF, DECEL_CAP);
                }
            }
            MAHI_LAND => {
                if self.naraku_check(actor, solid, wet, events) {
                    self.fallen();
                    return;
                }
                actor.angle[1] = actor.angle[1].wrapping_add(TURN_REEL);
                if landed {
                    self.gravity = GRAVITY_SLOW;
                    self.lock_on = true;
                    self.step = Step::Dousa;
                    self.mode = DOUSA_WAIT;
                }
            }
            _ => {}
        }
        self.target_angle = self.heading;
        if self.mode >= MAHI_IDLE && self.mode != MAHI_LAND {
            if self.dormant(switches) {
                self.countdown[CD_MAHI] = MAHI_FRAMES;
                self.body_atari_check(blow, events);
                return;
            }
            self.switch_no = NO_SWITCH;
            self.step = Step::Dousa;
            self.mode = DOUSA_SETTLE;
            if self.carried {
                self.carried = false;
                self.carry_ok = false;
                self.gravity = GRAVITY_DROP;
                self.speed.y = LIFT_HIGH;
            }
            return;
        }
        if self.countdown[CD_MAHI] == 0 {
            if self.bck != BCK_BURUBURU {
                self.anm_init(BCK_BURUBURU, 1.0, EMODE_LOOP, 1.0);
                self.countdown[CD_RECOVER] = MAHI_FRAMES;
                self.carry_ok = false;
                events.list.push(SimEvent::Se(SE_RECOVER));
                events.list.push(SimEvent::Se(SE_AWAKE));
            }
            if self.countdown[CD_RECOVER] == 1 {
                if self.carried {
                    self.carried = false;
                    self.carry_ok = false;
                }
                self.lock_on = true;
                self.gravity = GRAVITY_DROP;
                self.speed.y = LIFT_HIGH;
                self.shapes.body_co = true;
                self.mode = MAHI_LAND;
            }
        }
        if self.carried
            || self.mode == MAHI_LAND
            || !self.week_atari_check(actor, player, blow, events)
        {
            self.body_atari_check(blow, events);
        }
    }

    pub fn action_itai(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        solid: Option<&Collision>,
        wet: Option<&Water>,
        events: &mut SimEvents,
    ) {
        let landed = ground::grounded(solid, actor.pos);
        let mut run = true;
        while run {
            run = false;
            match self.mode {
                ITAI_FLINCH => {
                    self.clear_countup();
                    self.shapes.needle_sheath();
                    self.target_angle =
                        search_angle(actor.pos, player.pos).wrapping_add(HALF_TURN);
                    self.heading = self.target_angle;
                    self.speed_f = SPEED_WINDUP;
                    events.list.push(SimEvent::Se(SE_DAMAGE));
                    if self.health > 0 {
                        self.anm_init(BCK_DAMAGE, 1.0, EMODE_NONE, 1.0);
                        if self.countdown[CD_MAHI] > WOUND_CUT {
                            self.speed_f = WOUND_SLOW;
                        }
                        self.mode = ITAI_RECOVER;
                    } else {
                        self.mode = ITAI_DIE;
                    }
                }
                ITAI_RECOVER => {
                    add_calc0(&mut self.speed_f, DECEL_HALF, DECEL_CAP);
                    if self.speed_f < SPEED_EPS {
                        self.speed_f = 0.0;
                        self.gravity = GRAVITY_SLOW;
                        if self.countdown[CD_MAHI] < WOUND_CUT {
                            self.step = Step::Dousa;
                            self.mode = DOUSA_WAIT;
                        } else {
                            self.anm_init(BCK_MAHI, 1.0, EMODE_LOOP, 1.0);
                            self.step = Step::Mahi;
                            self.mode = MAHI_IDLE;
                        }
                    }
                }
                ITAI_DIE => {
                    self.shapes.eye_tg = false;
                    self.shapes.weak_tg = false;
                    self.shapes.eye_hit = false;
                    self.shapes.weak_hit = false;
                    self.anm_init(BCK_DEAD1, 1.0, EMODE_NONE, 1.0);
                    self.mode = ITAI_LEAP;
                }
                ITAI_LEAP => {
                    if self.morph.is_stop() {
                        self.shapes.needle_at = true;
                        self.shapes.needle_hit = true;
                        self.speed.y = LIFT_LEAP;
                        self.gravity = GRAVITY_RISE;
                        self.speed_f = SPEED_LEAP;
                        self.countdown[CD_BURST] = BURST_FRAMES;
                        self.heading = search_angle(actor.pos, player.pos);
                        self.anm_init(BCK_DEAD2, 1.0, EMODE_NONE, 1.0);
                        self.mode = ITAI_BOUNCE;
                        run = true;
                    }
                }
                ITAI_BOUNCE => {
                    if self.speed.y > 0.0 && self.countup[CU_LANDED] == 0 {
                        events.list.push(SimEvent::Se(SE_JUMP));
                        self.countup[CU_LANDED] = 1;
                    }
                    actor.angle[1] = actor.angle[1].wrapping_add(TURN_REEL);
                    if landed {
                        events.list.push(SimEvent::Se(SE_JUMP2));
                        events.list.push(SimEvent::Se(SE_JITABATA));
                        self.speed.y = LIFT_LEAP;
                        self.gravity = GRAVITY_RISE;
                        self.speed_f = SPEED_LEAP;
                    }
                    if self.countdown[CD_BURST] == 0 {
                        self.anm_init(BCK_DEAD3, 1.0, EMODE_NONE, 1.0);
                        events.list.push(SimEvent::Se(SE_BEFORE_EXPLODE));
                        self.speed_f = 0.0;
                        self.mode = ITAI_BURST;
                    }
                }
                ITAI_BURST => {
                    if self.morph.is_stop() {
                        events.list.push(SimEvent::Se(SE_EXPLODE));
                        self.doomed = true;
                    }
                }
                _ => {}
            }
        }
        if self.naraku_check(actor, solid, wet, events) {
            if self.not_in_home_room || self.health <= 0 {
                if self.mode != ITAI_BURST {
                    self.anm_init(BCK_DEAD3, 1.0, EMODE_NONE, 1.0);
                    events.list.push(SimEvent::Se(SE_BEFORE_EXPLODE));
                    self.speed_f = 0.0;
                    self.mode = ITAI_BURST;
                }
            } else {
                self.fallen();
            }
        }
    }

    pub fn action_handou_move(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        solid: Option<&Collision>,
        wet: Option<&Water>,
        switches: &Switches,
        events: &mut SimEvents,
    ) {
        let mut run = true;
        while run {
            run = false;
            match self.mode {
                HANDOU_WINDUP => {
                    self.speed_f = SPEED_SLIP;
                    self.heading = search_angle(actor.pos, player.pos).wrapping_add(HALF_TURN);
                    if self.hit_code == HIT_HAMMER_TURN {
                        self.heading = walk::to_s16(player.face).wrapping_add(-QUARTER_TURN);
                        self.speed_f = SPEED_SLIP;
                    }
                    self.shapes.needle_sheath();
                    self.target_angle = self.heading;
                    self.mode = HANDOU_SLIP;
                    run = true;
                }
                HANDOU_SLIP => {
                    events.list.push(SimEvent::Se(SE_SLIP));
                    add_calc0(&mut self.speed_f, SLIP_DECEL, SLIP_DECEL_CAP);
                    if self.speed_f < SLIP_EPS {
                        self.speed_f = 0.0;
                        self.heading = actor.angle[1];
                        self.step = Step::Dousa;
                        self.mode = DOUSA_WAIT;
                        if self.dormant(switches) {
                            self.lock_on = false;
                            self.countdown[CD_MAHI] = MAHI_FRAMES;
                            self.step = Step::Mahi;
                            self.mode = MAHI_IDLE;
                        }
                    }
                }
                _ => {}
            }
        }
        if self.naraku_check(actor, solid, wet, events) {
            self.fallen();
        }
    }

    pub fn action_modoru_move(
        &mut self,
        actor: &mut Actor,
        switches: &Switches,
        events: &mut SimEvents,
    ) {
        let mut run = true;
        while run {
            run = false;
            match self.mode {
                MORI_SHRINK => {
                    self.abyss_timer = 0;
                    self.water_splash = false;
                    self.anm_init(BCK_WAIT, 10.0, EMODE_NONE, 1.0);
                    events.list.push(SimEvent::Se(SE_WATER_RECOVER));
                    self.mode = MORI_HOLD;
                    run = true;
                }
                MORI_HOLD => {
                    add_calc2(&mut self.scale.x, 1.0, 1.0, SCALE_STEP);
                    self.scale.y = self.scale.x;
                    self.scale.z = self.scale.x;
                    if self.scale.x < SCALE_GATE {
                        self.speed_f = 0.0;
                        self.speed = Vec3::ZERO;
                        self.gravity = GRAVITY_SLOW;
                        self.water_splash = false;
                        self.heading = actor.angle[1];
                        actor.pos = self.spawn_pos;
                        self.scale = Vec3::ZERO;
                        actor.angle[1] = self.spawn_rot;
                        self.heading = actor.angle[1];
                        actor.pos = self.spawn_pos;
                        self.target_angle = self.heading;
                        self.mode = MORI_GROW;
                    }
                }
                MORI_GROW => {
                    actor.angle[1] = self.spawn_rot;
                    self.heading = actor.angle[1];
                    actor.pos = self.spawn_pos;
                    self.target_angle = self.heading;
                    add_calc2(&mut self.scale.x, 1.0, 1.0, SCALE_STEP);
                    self.scale.y = self.scale.x;
                    self.scale.z = self.scale.x;
                    if self.scale.x > SCALE_DONE {
                        self.countup[CU_LANDED] = 0;
                        self.scale = Vec3::ONE;
                        self.shapes.weak_tg = false;
                        self.shapes.needle_sheath();
                        self.shapes.weak_hit = false;
                        self.step = Step::Dousa;
                        self.mode = DOUSA_SETTLE;
                        if self.dormant(switches) {
                            self.lock_on = false;
                            self.countdown[CD_MAHI] = MAHI_FRAMES;
                            self.step = Step::Mahi;
                            self.mode = MAHI_IDLE;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn integrate(&mut self, actor: &mut Actor) {
        let (speed, forward) = stream::calc_speed(
            self.speed_f,
            self.heading,
            self.speed.y,
            self.gravity,
            MAX_FALL,
        );
        self.speed = speed;
        self.speed_f = forward;
        actor.pos += self.speed;
    }

    fn bg_check(&self, actor: &mut Actor, from: Vec3, solid: Option<&Collision>) {
        if let Some(solid) = solid {
            actor.pos = solid.slide(from, actor.pos);
        }
    }
}

pub fn is_armos(name: &str) -> bool {
    ARMOS_ACTORS.contains(&name)
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    water: Option<Res<'w, Water>>,
    switches: Res<'w, Switches>,
    carry: Res<'w, carry::Carry>,
    combat: Res<'w, Combat>,
    hammer: Res<'w, hammer::Cut>,
    held: Res<'w, Held>,
    blasts: Res<'w, bombflower::Blasts>,
    players: Query<'w, 's, &'static Player>,
    arrows: Query<'w, 's, &'static arrow::Arrow>,
    bolts: Query<'w, 's, &'static boomerang::Boomerang>,
    chains: Query<'w, 's, &'static hookshot::Hookshot>,
}

fn active_at(watch: &Watch<'_, '_>) -> At {
    if watch
        .chains
        .iter()
        .any(|chain| chain.phase != hookshot::Phase::Wait)
    {
        return At::Hook;
    }
    for shaft in watch.arrows.iter() {
        if shaft.phase != arrow::Phase::Flying {
            continue;
        }
        return match shaft.kind {
            arrow::TYPE_LIGHT => At::Light,
            arrow::TYPE_FIRE => At::FireArrow,
            arrow::TYPE_ICE => At::IceArrow,
            _ => At::Arrow,
        };
    }
    if watch
        .bolts
        .iter()
        .any(|boom| boom.parameter == boomerang::MODE_THROW)
    {
        return At::Boomerang;
    }
    if watch.hammer.kind == hammer::CUT_HAMMER_SIDESWING
        || watch.hammer.kind == hammer::CUT_HAMMER_FRONTSWING
    {
        return At::Hammer;
    }
    if watch.combat.active && watch.combat.frame >= 1 && watch.combat.frame < 6 {
        return At::Sword;
    }
    At::Idle
}

pub fn control(
    watch: Watch<'_, '_>,
    mut health: ResMut<Health>,
    mut zones: ResMut<ZoneActors>,
    mut events: ResMut<SimEvents>,
    mut statues: Query<(Entity, &mut Actor, &mut Armos), With<Armos>>,
    mut commands: Commands,
) {
    let solid = watch.collision.as_deref();
    let wet = watch.water.as_deref();
    let switches = *watch.switches;
    let Some(player) = watch.players.iter().next().copied() else {
        return;
    };
    let blow = Blow {
        at: active_at(&watch),
        blade: watch.held.item,
        soup: watch.combat.soup,
        crit: matches!(watch.combat.kind, CutKind::EA | CutKind::EB),
        turn: watch.hammer.kind == hammer::CUT_HAMMER_SIDESWING,
    };
    let mut spent: Vec<(Entity, i8, u16)> = Vec::new();
    for (entity, mut actor, mut statue) in &mut statues {
        let actor = &mut *actor;
        let statue = &mut *statue;
        if !statue.entered {
            statue.begin(actor, &switches);
        }
        statue.tick_timers();
        if statue.enemy_ice(&mut events) {
            if statue.health <= LIGHT_DEAD {
                spent.push((entity, actor.room, actor.set_id));
            }
            continue;
        }
        statue.carried = watch.carry.held == Some(entity);
        let blast = watch
            .blasts
            .list
            .iter()
            .any(|origin| touches(*origin, 0.0, actor.pos, REACH_BOMB));
        statue.resolve(actor, &player, blow.at, blast);
        statue.spike(&mut health);
        let from = actor.pos;
        match statue.step {
            Step::Dousa => statue.action_dousa(
                actor,
                &player,
                solid,
                wet,
                blow,
                &switches,
                &mut events,
            ),
            Step::Mahi => {
                statue.action_mahi(actor, &player, solid, wet, blow, &switches, &mut events)
            }
            Step::Itai => statue.action_itai(actor, &player, solid, wet, &mut events),
            Step::HandouMove => {
                statue.action_handou_move(actor, &player, solid, wet, &switches, &mut events)
            }
            Step::ModoruMove => statue.action_modoru_move(actor, &switches, &mut events),
        }
        statue.morph.play();
        statue.integrate(actor);
        if statue.step != Step::ModoruMove {
            statue.bg_check(actor, from, solid);
        }
        if statue.doomed {
            spent.push((entity, actor.room, actor.set_id));
        }
    }
    for (entity, room, set_id) in spent {
        zones.mark_done(room, set_id);
        let _ = commands.entity(entity).despawn();
    }
}