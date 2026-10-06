use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::{Quat, Vec3};

use super::actor::Actor;
use super::arrow;
use super::boomerang;
use super::bomb;
use super::bombflower;
use super::collision::Collision;
use super::combat::Combat;
use super::damage;
use super::events::{SimEvent, SimEvents};
use super::flags::{Switches, ZoneActors};
use super::ground;
use super::hammer;
use super::health::Health;
use super::hookshot;
use super::iball;
use super::item::Held;
use super::player::Player;
use super::stream;
use super::walk;

pub const AMOS_KNIGHT_ACTORS: [&str; 1] = ["amos"];

const BYTE_MASK: u32 = 0xFF;
const TYPE_SHIFT: u32 = 0x00;
const AREA_SHIFT: u32 = 0x08;
const INACTIVE_SHIFT: u32 = 0x10;
const SWITCH_SHIFT: u32 = 0x18;

const REG8_S9: i16 = 0;
const REG8_F1: f32 = 0.0;
const REG14_S1: i8 = 0;
const REG14_S2: i8 = 0;

const NO_SWITCH: u8 = 0xFF;
const INACTIVE: u8 = 1;
const AREA_RADIUS_FIXED: f32 = 400.0;
const AREA_RADIUS_STEP: f32 = 100.0;

pub const BCK_BOM_NOMI: i32 = 0x14;
pub const BCK_CLOSE: i32 = 0x15;
pub const BCK_CLOSE_LOOP: i32 = 0x16;
pub const BCK_DAMAGE: i32 = 0x17;
pub const BCK_DAMAGE_END: i32 = 0x18;
pub const BCK_DAMAGE_LOOP: i32 = 0x19;
pub const BCK_DEAD: i32 = 0x1A;
pub const BCK_OKIRU: i32 = 0x1B;
pub const BCK_OPEN: i32 = 0x1C;
pub const BCK_OPEN_LOOP: i32 = 0x1D;
pub const BCK_SLEEP: i32 = 0x1E;
pub const BCK_SLEEP_LOOP: i32 = 0x1F;

const FRAME_BOM_NOMI: i16 = 29;
const FRAME_CLOSE: i16 = 8;
const FRAME_CLOSE_LOOP: i16 = 6;
const FRAME_DAMAGE: i16 = 14;
const FRAME_DAMAGE_END: i16 = 49;
const FRAME_DAMAGE_LOOP: i16 = 6;
const FRAME_DEAD: i16 = 40;
const FRAME_OKIRU: i16 = 7;
const FRAME_OPEN: i16 = 7;
const FRAME_OPEN_LOOP: i16 = 6;
const FRAME_SLEEP: i16 = 9;
const FRAME_SLEEP_LOOP: i16 = 6;

const EMODE_NONE: u8 = 0;
const EMODE_LOOP: u8 = 2;
const STATE_STOP: u8 = 1;
const STATE_LOOP: u8 = 2;

const MODE_DOUSA_INIT: u8 = 0;
const MODE_DOUSA_SCAN: u8 = 1;
const MODE_DOUSA_OKIRU: u8 = 2;
const MODE_DOUSA_CLOSE: u8 = 3;
const MODE_DOUSA_AIM: u8 = 4;
const MODE_DOUSA_JUMP: u8 = 5;
const MODE_DOUSA_FLY: u8 = 6;
const MODE_DOUSA_HOP: u8 = 7;
const MODE_DOUSA_BOUNCE: u8 = 8;
const MODE_DOUSA_SLEEP_INIT: u8 = 9;
const MODE_DOUSA_SLEEP_MAIN: u8 = 10;
const MODE_MODORU_MOVE_INIT: u8 = 20;
const MODE_MODORU_MOVE_MAIN: u8 = 21;
const MODE_MODORU_MOVE_END: u8 = 22;
const MODE_HANDOU_MOVE_INIT: u8 = 30;
const MODE_HANDOU_MOVE_MAIN: u8 = 31;
const MODE_ITAI_MOVE_INIT: u8 = 40;
const MODE_ITAI_MOVE_FLINCH: u8 = 41;
const MODE_ITAI_MOVE_HOLD: u8 = 42;
const MODE_ITAI_MOVE_RISE: u8 = 43;
const MODE_ITAI_MOVE_EAT: u8 = 44;
const MODE_ITAI_MOVE_CHEW: u8 = 45;
const MODE_ITAI_MOVE_LEAP: u8 = 46;
const MODE_ITAI_MOVE_BURST: u8 = 47;

const CD_HOP: usize = 0;
const CD_BOMB: usize = 1;
const CD_NEEDLE: usize = 2;
const CD_COUNT: usize = 4;
const CU_COUNT: usize = 5;

const NEEDLE_SET_FRAMES: i16 = 6;
const HOPS_MAX: i16 = 2;
const HOP_FRAMES: i16 = 10;
const BOUNCE_FRAMES: i16 = 100;
const EAT_FRAMES: i16 = 10;
const BOMB_REST_TIME: i16 = 100;
const BOMB_FUSE: i16 = 1;
const SHOWMAP_RANGE: f32 = 1000.0;
const SLEEP_RANGE: f32 = 2000.0;
const HEIGHT_GATE: f32 = 300.0;
const TURN_GATE: i32 = 0x100;
const NOMI_GATE: i32 = 0x2000;
const TURN_DIV: i16 = 1;
const TURN_CAP: i16 = 0x500;
const HALF_TURN: i16 = -0x8000;
const QUARTER_TURN: i16 = 0x4000;
const LEAP_SPIN: i16 = 0x1000;

const GRAVITY_SPAWN: f32 = -10.0;
const GRAVITY_HOP: f32 = -11.0;
const GRAVITY_BOUNCE: f32 = -6.0;
const GRAVITY_LEAP: f32 = -10.0;
const MAX_FALL: f32 = -100.0;
const SPEED_HOP: f32 = 30.0;
const SPEED_LEAP: f32 = 25.0;
const LIFT_HOP: f32 = 40.0;
const LIFT_BOUNCE: f32 = 15.0;
const SPEED_RETURN: f32 = 15.0;
const SPEED_SLIP: f32 = 20.0;
const SPEED_SLIP_FAR: f32 = 40.0;
const SPEED_RETREAT: f32 = -20.0;
const SPEED_EPS: f32 = 0.1;
const DECEL_SCALE: f32 = 0.8;
const DECEL_CAP: f32 = 2.0;
const RETURN_GATE: f32 = 20.0;
const ABYSS_DROP: f32 = 1500.0;

const STTS_WEIGHT_INIT: u8 = 254;
const STTS_WEIGHT_EAT: u8 = 0xFF;

const EYE_RADIUS: f32 = 60.0;
const BODY_RADIUS: f32 = 80.0;
const BODY_HEIGHT: f32 = 300.0;
const NEEDLE_RADIUS: f32 = 130.0;
const NEEDLE_HEIGHT: f32 = 30.0;
const NEEDLE_LIFT: f32 = 40.0;
const ATTENTION_LIFT: f32 = 330.0;
const EYE_HEIGHT: f32 = 250.0;
const BURST_LIFT: f32 = 150.0;
const LINE_LIFT: f32 = 100.0;
const MOUTH_DROP: f32 = 20.0;
const MOUTH_SLACK: f32 = 10.0;
const SUCK_STEP: f32 = 50.0;
const PLAYER_REACH: f32 = 30.0;
const REACH_SWORD: f32 = 20.0;
const REACH_ARROW: f32 = 35.0;
const REACH_HAMMER: f32 = 50.0;
const REACH_HOOKSHOT: f32 = 5.0;
const REACH_BOOMERANG: f32 = 90.0;

const EYE_TG: bool = true;
const BODY_TG: bool = true;
const NEEDLE_TG: bool = false;
const NEEDLE_CO: bool = true;

const EYE_YAW_MIN: i16 = -0x71C;
const EYE_YAW_MAX: i16 = 0x71C;
const EYE_PITCH_MIN: i16 = -0x38E;
const EYE_PITCH_MAX: i16 = 0x38E;
const EYE_TURN_CAP: i16 = 0x500;

const JOINT_EYE: [f32; 3] = [0.0, 240.0, 60.0];
const JOINT_MOUTH: [f32; 3] = [0.0, 150.0, 70.0];
const JOINT_WAIST: [f32; 3] = [0.0, 0.0, 0.0];
const JOINT_JAW: [f32; 3] = [0.0, 0.0, 0.0];
const SUCK_OFFSET: [f32; 3] = [0.0, 120.0, 40.0];

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

const HEALTH_MAX: i32 = 10;
const GRAPPLE_HEALTH: i32 = 10;
const STEAL_ITEMS: i16 = 3;
const SPIKE_POWER: u8 = 2;
const LIGHT_DEAD: i32 = -0x80;
const DROP_PARAM: u32 = 0;

const SHOCK_LANDING: i32 = 3;
const SHOCK_LEAP: i32 = 1;
const SHOCK_MAGNITUDE: i32 = -0x21;
const SHOCK_DIR: [f32; 3] = [0.0, 1.0, 0.0];

const SE_LK_SW_HIT_S: u32 = 0x2803;
const SE_LK_LAST_HIT: u32 = 0x2828;
const SE_LK_W_WEP_HIT: u32 = 0x2833;
const SE_LK_MS_WEP_HIT: u32 = 0x2834;
const SE_LK_HAMMER_HIT: u32 = 0x2855;
const SE_CV_AM_AWAKE: u32 = 0x4874;
const SE_CV_AM_JUMP: u32 = 0x4875;
const SE_CV_AM_OPEN_MOUTH: u32 = 0x4876;
const SE_CV_AM_EYE_DAMAGE: u32 = 0x4877;
const SE_CV_AM_EAT_BOMB: u32 = 0x4878;
const SE_CV_AM_JITABATA: u32 = 0x4879;
const SE_CM_AM_NEEDLE_OUT: u32 = 0x587D;
const SE_CM_AM_NEEDLE_IN: u32 = 0x587E;
const SE_CM_AM_JUMP: u32 = 0x587F;
const SE_CM_AM_MOUTH_OPEN: u32 = 0x5880;
const SE_CM_AM_EYE_DAMAGE: u32 = 0x5881;
const SE_CM_AM_JUMP_S: u32 = 0x5882;
const SE_CM_AM_MOUTH_CLOSE: u32 = 0x5883;
const SE_CM_AM_EAT_BOMB: u32 = 0x5884;
const SE_CM_AM_JUMP_L: u32 = 0x5885;
const SE_CM_AM_BEF_EXPLODE: u32 = 0x5886;
const SE_CM_AM_EXPLODE: u32 = 0x5887;
const SE_CM_L_ARROW_SHRINK: u32 = 0x5904;
const SE_CM_L_ARROW_PASS_AWAY: u32 = 0x5905;

pub const CULL_MIN: [f32; 3] = [-100.0, -10.0, -80.0];
pub const CULL_MAX: [f32; 3] = [120.0, 400.0, 100.0];

fn frame_max(bck: i32) -> i16 {
    match bck {
        BCK_BOM_NOMI => FRAME_BOM_NOMI,
        BCK_CLOSE => FRAME_CLOSE,
        BCK_CLOSE_LOOP => FRAME_CLOSE_LOOP,
        BCK_DAMAGE => FRAME_DAMAGE,
        BCK_DAMAGE_END => FRAME_DAMAGE_END,
        BCK_DAMAGE_LOOP => FRAME_DAMAGE_LOOP,
        BCK_DEAD => FRAME_DEAD,
        BCK_OKIRU => FRAME_OKIRU,
        BCK_OPEN => FRAME_OPEN,
        BCK_OPEN_LOOP => FRAME_OPEN_LOOP,
        BCK_SLEEP => FRAME_SLEEP,
        _ => FRAME_SLEEP_LOOP,
    }
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

fn distance_angle_s(x: i16, y: i16) -> i32 {
    i32::from(x.wrapping_sub(y)).abs()
}

fn atan2s(y: f32, x: f32) -> i16 {
    walk::to_s16(y.atan2(x))
}

fn search_angle(from: Vec3, to: Vec3) -> i16 {
    atan2s(to.x - from.x, to.z - from.z)
}

fn search_distance(from: Vec3, to: Vec3) -> f32 {
    (to - from).length()
}

fn joint(actor: &Actor, offset: [f32; 3]) -> Vec3 {
    let turn = Quat::from_rotation_y(walk::from_s16(actor.angle[1]))
        * Quat::from_rotation_x(walk::from_s16(actor.angle[0]))
        * Quat::from_rotation_z(walk::from_s16(actor.angle[2]));
    actor.pos + turn * Vec3::new(offset[0], offset[1], offset[2])
}

fn turned(yaw: i16, offset: [f32; 3]) -> Vec3 {
    let (sin, cos) = walk::from_s16(yaw).sin_cos();
    Vec3::new(
        offset[0] * cos + offset[2] * sin,
        offset[1],
        offset[2] * cos - offset[0] * sin,
    )
}

fn touches(centre: Vec3, radius: f32, at: Vec3, reach: f32) -> bool {
    walk::within(centre, at, radius + reach)
}

fn cyl_touches(centre: Vec3, radius: f32, height: f32, at: Vec3, reach: f32) -> bool {
    let dx = at.x - centre.x;
    let dz = at.z - centre.z;
    let span = radius + reach;
    if span * span < dx * dx + dz * dz {
        return false;
    }
    at.y + reach >= centre.y && at.y - reach <= centre.y + height
}

fn clamp_eye(value: i16, low: i16, high: i16) -> i16 {
    if value < low {
        low
    } else if value > high {
        high
    } else {
        value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Dousa,
    ModoruMove,
    HandouMove,
    ItaiMove,
}

impl Step {
    pub fn code(self) -> u8 {
        match self {
            Self::Dousa => 0,
            Self::ModoruMove => 1,
            Self::HandouMove => 2,
            Self::ItaiMove => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum At {
    Idle,
    Sword,
    Boomerang,
    Hammer,
    Hook,
    Light,
    Arrow,
    FireArrow,
    IceArrow,
    Bomb,
}

impl At {
    fn reach(self) -> f32 {
        match self {
            Self::Idle | Self::Bomb => 0.0,
            Self::Sword => REACH_SWORD,
            Self::Boomerang => REACH_BOOMERANG,
            Self::Hammer => REACH_HAMMER,
            Self::Hook => REACH_HOOKSHOT,
            Self::Light | Self::Arrow | Self::FireArrow | Self::IceArrow => REACH_ARROW,
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
            Self::Sword => damage::sword_power(blade, soup),
            Self::Arrow | Self::FireArrow | Self::IceArrow => damage::ARROW,
            Self::Bomb => damage::BOMB,
            Self::Boomerang => damage::BOMERANG,
            Self::Hammer => hammer::AT_POWER,
            Self::Idle | Self::Hook | Self::Light => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blow {
    pub at: At,
    pub blade: u8,
    pub soup: bool,
    pub turn: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Shape {
    pub eye_hit: bool,
    pub body_hit: bool,
    pub needle_hit: bool,
    pub needle_at: bool,
    pub needle_r_hit: bool,
}

impl Shape {
    fn needle_arm(&mut self) {
        self.needle_at = true;
        self.needle_r_hit = true;
    }

    fn needle_sheathe(&mut self) {
        self.needle_at = false;
        self.needle_at = false;
    }
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

pub struct BombView {
    pub entity: Entity,
    pub pos: Vec3,
    pub free: bool,
    pub rest_time: i16,
}

type Shells<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static mut Actor, &'static mut bomb::Bomb),
    (With<bomb::Bomb>, Without<AmosKnight>, Without<Player>),
>;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct AmosKnight {
    pub kind: u8,
    pub prm_radius: u8,
    pub starts_inactive: u8,
    pub switch_no: u8,
    pub step: Step,
    pub mode: u8,
    pub huge_knockback: bool,
    pub body_being_hit: bool,
    pub shown: bool,
    pub lock_on: bool,
    pub entered: bool,
    pub doomed: bool,
    pub countdown: [i16; CD_COUNT],
    pub countup: [i16; CU_COUNT],
    pub target_angle: i16,
    pub shape_angle: i16,
    pub spawn_rot: i16,
    pub curr_bck: i32,
    pub swallowed: Option<Entity>,
    pub area_radius: f32,
    pub correction_y: f32,
    pub spawn_pos_y: f32,
    pub eyeball: Vec3,
    pub mouth: Vec3,
    pub waist: Vec3,
    pub jaw: Vec3,
    pub spawn_pos: Vec3,
    pub eye_rot: [i16; 3],
    pub target_eye_rot: [i16; 3],
    pub attention: Vec3,
    pub needle: Vec3,
    pub speed: Vec3,
    pub speed_f: f32,
    pub gravity: f32,
    pub stts_weight: u8,
    pub shapes: Shape,
    pub light: Light,
    pub morph: Frame,
    pub health: i32,
    pub max_health: i32,
    pub steal_left: i16,
    pub shock: Option<(i32, i32, [f32; 3])>,
}

impl AmosKnight {
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
        if REG8_S9 != 0 {
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
            mode: MODE_DOUSA_INIT,
            huge_knockback: false,
            body_being_hit: false,
            shown: false,
            lock_on: false,
            entered: false,
            doomed: false,
            countdown: [0; CD_COUNT],
            countup: [0; CU_COUNT],
            target_angle: 0,
            shape_angle: 0,
            spawn_rot: 0,
            curr_bck: 0,
            swallowed: None,
            area_radius,
            correction_y: 0.0,
            spawn_pos_y: 0.0,
            eyeball: Vec3::ZERO,
            mouth: Vec3::ZERO,
            waist: Vec3::ZERO,
            jaw: Vec3::ZERO,
            spawn_pos: Vec3::ZERO,
            eye_rot: [0; 3],
            target_eye_rot: [0; 3],
            attention: Vec3::ZERO,
            needle: Vec3::ZERO,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            gravity: GRAVITY_SPAWN,
            stts_weight: STTS_WEIGHT_INIT,
            shapes: Shape::default(),
            light: Light::idle(),
            morph: Frame::idle(),
            health: HEALTH_MAX,
            max_health: HEALTH_MAX,
            steal_left: STEAL_ITEMS,
            shock: None,
        }
    }

    pub fn present(&self, switches: &Switches) -> bool {
        if self.starts_inactive != INACTIVE || self.switch_no == NO_SWITCH {
            return true;
        }
        switches.test(self.switch_no)
    }

    pub fn begin(&mut self, actor: &Actor, switches: &Switches) {
        self.entered = true;
        self.spawn_pos = actor.pos;
        self.spawn_pos_y = actor.pos.y;
        self.spawn_rot = actor.angle[1];
        self.target_angle = actor.angle[1];
        self.shape_angle = actor.angle[1];
        self.shown = !self.dormant(switches);
        self.lock_on = false;
        self.shapes.needle_sheathe();
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    pub fn anm_init(&mut self, bck: i32, morf: f32, loop_mode: u8, speed: f32) {
        self.curr_bck = bck;
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

    fn line_check(&self, actor: &Actor, dest: Vec3, solid: Option<&Collision>) -> bool {
        let centre = Vec3::new(actor.pos.x, actor.pos.y + LINE_LIFT, actor.pos.z);
        let target = Vec3::new(dest.x, dest.y + LINE_LIFT, dest.z);
        match solid {
            Some(solid) => solid.slide(centre, target) == target,
            None => true,
        }
    }

    fn place_joints(&mut self, actor: &Actor) {
        self.eyeball = joint(actor, JOINT_EYE);
        self.mouth = joint(actor, JOINT_MOUTH);
        self.waist = joint(actor, JOINT_WAIST);
        self.jaw = joint(actor, JOINT_JAW);
    }

    fn resolve(&mut self, actor: &Actor, player: &Player, at: At) {
        self.place_joints(actor);
        self.attention = Vec3::new(actor.pos.x, actor.pos.y + ATTENTION_LIFT, actor.pos.z);
        self.needle = Vec3::new(actor.pos.x, actor.pos.y + NEEDLE_LIFT, actor.pos.z);
        let reach = at.reach();
        let origin = player.pos;
        self.shapes.eye_hit = false;
        self.shapes.body_hit = false;
        self.shapes.needle_hit = self.shapes.needle_at
            && cyl_touches(
                self.needle,
                NEEDLE_RADIUS,
                NEEDLE_HEIGHT,
                origin,
                PLAYER_REACH,
            );
        if at == At::Bomb {
            self.shapes.body_hit = true;
            return;
        }
        if at == At::Idle {
            return;
        }
        if EYE_TG && (at.arrow() || at == At::Hook) {
            self.shapes.eye_hit = touches(self.eyeball, EYE_RADIUS, origin, reach);
        }
        if BODY_TG && at != At::Light {
            self.shapes.body_hit = cyl_touches(actor.pos, BODY_RADIUS, BODY_HEIGHT, origin, reach);
        }
    }

    fn spike(&mut self, health: &mut Health) {
        if NEEDLE_CO && self.shapes.needle_hit {
            health.damage(SPIKE_POWER);
        }
    }

    fn enemy_ice(&mut self, events: &mut SimEvents) -> bool {
        if self.light.timer == 0 {
            return false;
        }
        if self.light.timer == LIGHT_START {
            self.light.timer = self.light.timer.wrapping_add(1);
            events.list.push(SimEvent::Se(SE_CM_L_ARROW_SHRINK));
            return true;
        }
        self.light.timer = self.light.timer.wrapping_add(1);
        if self.light.timer < LIGHT_SHRINK_END + REG14_S1 {
            add_calc0(&mut self.light.scale_xz, LIGHT_SHRINK_SCALE, LIGHT_SHRINK_MAX);
            add_calc0(&mut self.light.scale_y, LIGHT_SHRINK_SCALE, LIGHT_SHRINK_MAX);
        } else {
            if self.light.timer == LIGHT_SHRINK_END + REG14_S1 {
                events.list.push(SimEvent::Se(SE_CM_L_ARROW_PASS_AWAY));
            }
            add_calc2(
                &mut self.light.scale_y,
                LIGHT_RISE_TARGET,
                LIGHT_RISE_SCALE,
                LIGHT_RISE_MAX,
            );
            add_calc0(&mut self.light.scale_xz, LIGHT_FADE_SCALE, LIGHT_FADE_MAX);
            if self.light.timer > LIGHT_GONE + REG14_S2 {
                self.health = LIGHT_DEAD;
            }
        }
        true
    }

    fn medama_move(&mut self, actor: &Actor, player: &Player) {
        if matches!(self.curr_bck, BCK_SLEEP | BCK_SLEEP_LOOP) {
            self.eye_rot = [0; 3];
            return;
        }
        let diff_x = actor.pos.x - player.pos.x;
        let diff_y = actor.pos.y + EYE_HEIGHT - player.pos.y;
        let diff_z = actor.pos.z - player.pos.z;
        self.target_eye_rot[1] = clamp_eye(atan2s(diff_x, diff_z), EYE_YAW_MIN, EYE_YAW_MAX);
        let flat = (diff_x * diff_x + diff_z * diff_z).sqrt();
        self.target_eye_rot[0] = clamp_eye(atan2s(diff_y, flat), EYE_PITCH_MIN, EYE_PITCH_MAX);
        stream::add_calc_angle_s2(
            &mut self.eye_rot[0],
            self.target_eye_rot[0],
            TURN_DIV,
            EYE_TURN_CAP,
        );
        stream::add_calc_angle_s2(
            &mut self.eye_rot[1],
            self.target_eye_rot[1],
            TURN_DIV,
            EYE_TURN_CAP,
        );
    }

    fn body_atari_check(&mut self, blow: Blow, switches: &Switches, events: &mut SimEvents) {
        if !(self.shapes.body_hit || NEEDLE_TG) {
            self.body_being_hit = false;
            return;
        }
        let power = blow.at.power(blow.blade, blow.soup);
        if power > 0 {
            self.health -= power;
        }
        if self.health <= 0 {
            events.list.push(SimEvent::Se(SE_LK_LAST_HIT));
        }
        if self.body_being_hit {
            return;
        }
        self.body_being_hit = true;
        match blow.at {
            At::Hammer => {
                events.list.push(SimEvent::Se(SE_LK_HAMMER_HIT));
                if self.dormant(switches) {
                    return;
                }
                if self.step == Step::HandouMove || self.step == Step::ItaiMove {
                    return;
                }
                self.step = Step::HandouMove;
                self.mode = MODE_HANDOU_MOVE_INIT;
                self.huge_knockback = blow.turn;
            }
            At::Sword => events.list.push(SimEvent::Se(SE_LK_SW_HIT_S)),
            At::Boomerang => events.list.push(SimEvent::Se(SE_LK_W_WEP_HIT)),
            At::Idle | At::Hook | At::Light | At::Arrow | At::FireArrow | At::IceArrow => {
                events.list.push(SimEvent::Se(SE_LK_MS_WEP_HIT));
            }
            At::Bomb => {}
        }
    }

    fn medama_atari_check(
        &mut self,
        blow: Blow,
        switches: &Switches,
        events: &mut SimEvents,
    ) -> bool {
        if self.dormant(switches) {
            return false;
        }
        if !self.shapes.eye_hit {
            return false;
        }
        match blow.at {
            At::Hook => {
                if !matches!(self.curr_bck, BCK_SLEEP | BCK_SLEEP_LOOP) {
                    if self.steal_left > 0 {
                        self.max_health = GRAPPLE_HEALTH;
                        self.health = GRAPPLE_HEALTH;
                        self.steal_left -= 1;
                        self.max_health = GRAPPLE_HEALTH;
                        self.health = GRAPPLE_HEALTH;
                    }
                    events.list.push(SimEvent::Se(SE_LK_MS_WEP_HIT));
                }
            }
            At::Sword => events.list.push(SimEvent::Se(SE_LK_SW_HIT_S)),
            At::Boomerang => events.list.push(SimEvent::Se(SE_LK_W_WEP_HIT)),
            At::Light => {
                self.light.timer = LIGHT_START;
                self.light.particle_scale = LIGHT_SCALE;
                self.light.y_offset = LIGHT_LIFT;
                self.lock_on = false;
                return true;
            }
            At::Arrow | At::FireArrow | At::IceArrow => {
                events.list.push(SimEvent::Se(SE_LK_MS_WEP_HIT));
                self.health -= damage::ARROW;
                if self.health <= 0 {
                    events.list.push(SimEvent::Se(SE_LK_LAST_HIT));
                }
                if matches!(self.curr_bck, BCK_SLEEP | BCK_SLEEP_LOOP) {
                    self.anm_init(BCK_OKIRU, 1.0, EMODE_NONE, 1.0);
                    self.lock_on = true;
                    self.shapes.needle_arm();
                    self.step = Step::Dousa;
                    self.mode = MODE_DOUSA_OKIRU;
                } else {
                    events.list.push(SimEvent::Se(SE_CM_AM_EYE_DAMAGE));
                    events.list.push(SimEvent::Se(SE_CV_AM_EYE_DAMAGE));
                    self.step = Step::ItaiMove;
                    self.mode = MODE_ITAI_MOVE_INIT;
                }
                return true;
            }
            At::Idle | At::Hammer | At::Bomb => {
                events.list.push(SimEvent::Se(SE_LK_MS_WEP_HIT));
            }
        }
        false
    }

    fn bomb_move_set(
        &mut self,
        mouth: Vec3,
        shape_yaw: i16,
        always_move_y: bool,
        shells: &mut Shells<'_, '_>,
    ) {
        let Some(entity) = self.swallowed else {
            return;
        };
        let Ok((_, mut actor, mut shell)) = shells.get_mut(entity) else {
            return;
        };
        add_calc2(&mut actor.pos.x, mouth.x, 1.0, SUCK_STEP);
        if always_move_y || mouth.y - MOUTH_SLACK < actor.pos.y {
            add_calc2(&mut actor.pos.y, mouth.y, 1.0, SUCK_STEP);
        }
        add_calc2(&mut actor.pos.z, mouth.z, 1.0, SUCK_STEP);
        actor.angle = [0, shape_yaw, 0];
        shell.timer = BOMB_REST_TIME;
        shell.life = bomb::Life::Carry;
    }

    fn bomb_nomi_check(
        &mut self,
        actor: &Actor,
        player: &Player,
        shells: &[BombView],
        eaten: &[Entity],
    ) -> bool {
        if !matches!(
            self.curr_bck,
            BCK_OPEN | BCK_OPEN_LOOP | BCK_DAMAGE | BCK_DAMAGE_LOOP
        ) {
            return false;
        }
        let target = search_angle(actor.pos, player.pos);
        if distance_angle_s(self.shape_angle, target) > NOMI_GATE {
            return false;
        }
        for shell in shells {
            if !shell.free || shell.rest_time <= 1 {
                continue;
            }
            if eaten.contains(&shell.entity) {
                continue;
            }
            if self.mouth.y - (MOUTH_DROP + REG8_F1) < shell.pos.y {
                self.swallowed = Some(shell.entity);
                self.step = Step::ItaiMove;
                self.mode = MODE_ITAI_MOVE_EAT;
                return true;
            }
        }
        false
    }

    pub fn action_dousa(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        solid: Option<&Collision>,
        blow: Blow,
        busy: bool,
        switches: &Switches,
        shells: &[BombView],
        eaten: &[Entity],
        events: &mut SimEvents,
    ) {
        while self.dousa_step(actor, player, solid, busy, switches, events) {}
        self.medama_move(actor, player);
        if self.mode != MODE_DOUSA_OKIRU && self.medama_atari_check(blow, switches, events) {
            return;
        }
        self.bomb_nomi_check(actor, player, shells, eaten);
    }

    fn dousa_step(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        solid: Option<&Collision>,
        busy: bool,
        switches: &Switches,
        events: &mut SimEvents,
    ) -> bool {
        let dist = search_distance(actor.pos, player.pos);
        loop {
            match self.mode {
                MODE_DOUSA_INIT => {
                    self.clear_countup();
                    self.anm_init(BCK_SLEEP_LOOP, 1.0, EMODE_LOOP, 1.0);
                    self.mode = MODE_DOUSA_SCAN;
                    return true;
                }
                MODE_DOUSA_SCAN => {
                    if self.dormant(switches) {
                        return false;
                    }
                    self.shown = true;
                    if dist < SHOWMAP_RANGE {
                        let y_dist = player.pos.y - actor.pos.y;
                        if y_dist.abs() > HEIGHT_GATE {
                            return false;
                        }
                        if self.line_check(actor, player.pos, solid) {
                            self.anm_init(BCK_OKIRU, 1.0, EMODE_NONE, 1.0);
                            events.list.push(SimEvent::Se(SE_CV_AM_AWAKE));
                            self.lock_on = true;
                            self.shapes.needle_arm();
                            self.mode = MODE_DOUSA_OKIRU;
                        }
                    }
                    return false;
                }
                MODE_DOUSA_OKIRU => {
                    if !self.morph.is_stop() {
                        return false;
                    }
                    self.mode = MODE_DOUSA_CLOSE;
                    return true;
                }
                MODE_DOUSA_CLOSE => {
                    if !matches!(self.curr_bck, BCK_CLOSE | BCK_CLOSE_LOOP) {
                        if self.curr_bck != BCK_DAMAGE_END {
                            events.list.push(SimEvent::Se(SE_CM_AM_NEEDLE_OUT));
                            events.list.push(SimEvent::Se(SE_CM_AM_MOUTH_CLOSE));
                        }
                        self.anm_init(BCK_CLOSE, 1.0, EMODE_NONE, 1.0);
                        self.countdown[CD_NEEDLE] = NEEDLE_SET_FRAMES;
                    }
                    self.target_angle = search_angle(actor.pos, player.pos);
                    self.mode = MODE_DOUSA_AIM;
                    return true;
                }
                MODE_DOUSA_AIM => {
                    if self.countdown[CD_NEEDLE] == 1 {
                        self.shapes.needle_arm();
                    }
                    if self.countdown[CD_NEEDLE] != 0 {
                        return false;
                    }
                    if (self.kind & 1) != 0 {
                        let x_dist = actor.pos.x - self.spawn_pos.x;
                        let z_dist = actor.pos.z - self.spawn_pos.z;
                        if (x_dist * x_dist + z_dist * z_dist).sqrt() > self.area_radius {
                            self.step = Step::ModoruMove;
                            self.mode = MODE_MODORU_MOVE_INIT;
                            return false;
                        }
                    } else {
                        if dist > SLEEP_RANGE {
                            self.mode = MODE_DOUSA_SLEEP_INIT;
                            return false;
                        }
                        let y_dist = player.pos.y - actor.pos.y;
                        if y_dist.abs() > HEIGHT_GATE {
                            self.mode = MODE_DOUSA_SLEEP_INIT;
                            return false;
                        }
                    }
                    if distance_angle_s(self.shape_angle, self.target_angle) < TURN_GATE {
                        self.mode = MODE_DOUSA_JUMP;
                    }
                    return false;
                }
                MODE_DOUSA_JUMP => {
                    self.speed_f = SPEED_HOP;
                    self.gravity = GRAVITY_HOP;
                    self.speed.y = LIFT_HOP;
                    events.list.push(SimEvent::Se(SE_CV_AM_JUMP));
                    if !self.line_check(actor, player.pos, solid) || busy {
                        self.speed_f = 0.0;
                    }
                    self.mode = MODE_DOUSA_FLY;
                    return false;
                }
                MODE_DOUSA_FLY => {
                    if self.curr_bck == BCK_CLOSE && self.morph.is_stop() {
                        self.anm_init(BCK_CLOSE_LOOP, 1.0, EMODE_LOOP, 1.0);
                    }
                    if !ground::grounded(solid, actor.pos) {
                        return false;
                    }
                    events.list.push(SimEvent::Se(SE_CM_AM_JUMP));
                    self.shock = Some((SHOCK_LANDING, SHOCK_MAGNITUDE, SHOCK_DIR));
                    self.speed_f = 0.0;
                    self.countdown[CD_HOP] = 0;
                    if self.countup[0] < HOPS_MAX {
                        self.countdown[CD_HOP] = HOP_FRAMES;
                    }
                    self.mode = MODE_DOUSA_HOP;
                    return true;
                }
                MODE_DOUSA_HOP => {
                    if self.countdown[CD_HOP] != 0 {
                        return false;
                    }
                    self.countup[0] = self.countup[0].wrapping_add(1);
                    if self.countup[0] > HOPS_MAX {
                        self.countdown[CD_HOP] = BOUNCE_FRAMES;
                        self.countup[0] = 0;
                        self.anm_init(BCK_DAMAGE, 0.0, EMODE_NONE, 1.0);
                        events.list.push(SimEvent::Se(SE_CM_AM_NEEDLE_IN));
                        events.list.push(SimEvent::Se(SE_CM_AM_MOUTH_OPEN));
                        events.list.push(SimEvent::Se(SE_CV_AM_OPEN_MOUTH));
                        self.shapes.needle_sheathe();
                        self.mode = MODE_DOUSA_BOUNCE;
                    } else {
                        self.mode = MODE_DOUSA_CLOSE;
                    }
                    return false;
                }
                MODE_DOUSA_BOUNCE => {
                    if ground::grounded(solid, actor.pos) {
                        self.gravity = GRAVITY_BOUNCE;
                        self.speed.y = LIFT_BOUNCE;
                        events.list.push(SimEvent::Se(SE_CM_AM_JUMP_S));
                        self.target_angle = search_angle(actor.pos, player.pos);
                    }
                    if self.countdown[CD_HOP] == 0 {
                        self.mode = MODE_DOUSA_CLOSE;
                    }
                    return false;
                }
                MODE_DOUSA_SLEEP_INIT => {
                    self.anm_init(BCK_SLEEP, 1.0, EMODE_NONE, 1.0);
                    events.list.push(SimEvent::Se(SE_CM_AM_NEEDLE_IN));
                    self.shapes.needle_sheathe();
                    self.lock_on = false;
                    self.mode = MODE_DOUSA_SLEEP_MAIN;
                    return false;
                }
                MODE_DOUSA_SLEEP_MAIN => {
                    if self.morph.is_stop() {
                        self.mode = MODE_DOUSA_INIT;
                    }
                    return false;
                }
                _ => return false,
            }
        }
    }

    pub fn action_modoru_move(
        &mut self,
        actor: &mut Actor,
        solid: Option<&Collision>,
        events: &mut SimEvents,
    ) {
        match self.mode {
            MODE_MODORU_MOVE_INIT => {
                self.anm_init(BCK_CLOSE_LOOP, 1.0, EMODE_LOOP, 1.0);
                self.shapes.needle_arm();
                self.gravity = GRAVITY_HOP;
                self.speed.y = LIFT_HOP;
                self.speed_f = SPEED_RETURN;
                events.list.push(SimEvent::Se(SE_CV_AM_JUMP));
                let x_dist = self.spawn_pos.x - actor.pos.x;
                let z_dist = self.spawn_pos.z - actor.pos.z;
                self.target_angle = atan2s(x_dist, z_dist);
                self.mode = MODE_MODORU_MOVE_MAIN;
            }
            MODE_MODORU_MOVE_MAIN => {
                let x_dist = self.spawn_pos.x - actor.pos.x;
                let z_dist = self.spawn_pos.z - actor.pos.z;
                if ground::grounded(solid, actor.pos) {
                    self.shock = Some((SHOCK_LEAP, SHOCK_MAGNITUDE, SHOCK_DIR));
                    events.list.push(SimEvent::Se(SE_CM_AM_JUMP));
                    events.list.push(SimEvent::Se(SE_CV_AM_JUMP));
                    self.speed.y = LIFT_HOP;
                    self.speed_f = SPEED_RETURN;
                    self.target_angle = atan2s(x_dist, z_dist);
                }
                if (x_dist * x_dist + z_dist * z_dist).sqrt() < RETURN_GATE {
                    self.target_angle = self.spawn_rot;
                    self.speed_f = 0.0;
                    self.mode = MODE_MODORU_MOVE_END;
                }
            }
            MODE_MODORU_MOVE_END => {
                if distance_angle_s(self.shape_angle, self.target_angle) < TURN_GATE {
                    self.shapes.needle_sheathe();
                    self.lock_on = false;
                    self.step = Step::Dousa;
                    self.mode = MODE_DOUSA_INIT;
                }
            }
            _ => {}
        }
    }

    pub fn action_handou_move(&mut self, actor: &mut Actor, player: &Player, events: &mut SimEvents) {
        loop {
            match self.mode {
                MODE_HANDOU_MOVE_INIT => {
                    self.speed_f = SPEED_SLIP;
                    let angle = search_angle(actor.pos, player.pos);
                    actor.angle[1] = angle.wrapping_add(HALF_TURN);
                    if self.huge_knockback {
                        actor.angle[1] = walk::to_s16(player.face).wrapping_sub(QUARTER_TURN);
                        self.speed_f = SPEED_SLIP_FAR;
                    }
                    self.target_angle = actor.angle[1];
                    if !matches!(self.curr_bck, BCK_CLOSE | BCK_CLOSE_LOOP) {
                        events.list.push(SimEvent::Se(SE_CM_AM_NEEDLE_OUT));
                        events.list.push(SimEvent::Se(SE_CM_AM_MOUTH_CLOSE));
                        self.anm_init(BCK_CLOSE, 1.0, EMODE_NONE, 1.0);
                    }
                    self.mode = MODE_HANDOU_MOVE_MAIN;
                }
                MODE_HANDOU_MOVE_MAIN => {
                    add_calc0(&mut self.speed_f, DECEL_SCALE, DECEL_CAP);
                    if self.speed_f < SPEED_EPS {
                        self.speed_f = 0.0;
                        self.countdown[CD_NEEDLE] = NEEDLE_SET_FRAMES;
                        actor.angle[1] = self.shape_angle;
                        self.step = Step::Dousa;
                        self.mode = MODE_DOUSA_CLOSE;
                    }
                    return;
                }
                _ => return,
            }
        }
    }

    pub fn action_itai_move(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        solid: Option<&Collision>,
        shells: &[BombView],
        eaten: &[Entity],
        events: &mut SimEvents,
        live: &mut Shells<'_, '_>,
        commands: &mut Commands,
    ) {
        while self.itai_move_step(actor, player, solid, events, live, commands) {}
        if self.mode == MODE_ITAI_MOVE_FLINCH || self.mode == MODE_ITAI_MOVE_HOLD {
            self.bomb_nomi_check(actor, player, shells, eaten);
        }
    }

    fn itai_move_step(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        solid: Option<&Collision>,
        events: &mut SimEvents,
        live: &mut Shells<'_, '_>,
        commands: &mut Commands,
    ) -> bool {
        loop {
            match self.mode {
                MODE_ITAI_MOVE_INIT => {
                    self.eye_rot = [0; 3];
                    self.shapes.needle_sheathe();
                    self.speed_f = SPEED_RETREAT;
                    actor.angle[1] = search_angle(actor.pos, player.pos);
                    self.target_angle = actor.angle[1];
                    events.list.push(SimEvent::Se(SE_CM_AM_NEEDLE_IN));
                    self.anm_init(BCK_DAMAGE, 1.0, EMODE_NONE, 1.0);
                    self.mode = MODE_ITAI_MOVE_FLINCH;
                    return true;
                }
                MODE_ITAI_MOVE_FLINCH => {
                    add_calc0(&mut self.speed_f, DECEL_SCALE, DECEL_CAP);
                    if !self.morph.is_stop() {
                        return false;
                    }
                    self.countdown[CD_HOP] = BOUNCE_FRAMES;
                    self.speed_f = 0.0;
                    self.anm_init(BCK_DAMAGE_LOOP, 1.0, EMODE_LOOP, 1.0);
                    self.mode = MODE_ITAI_MOVE_HOLD;
                    return false;
                }
                MODE_ITAI_MOVE_HOLD => {
                    if self.countdown[CD_HOP] != 0 {
                        return false;
                    }
                    self.anm_init(BCK_DAMAGE_END, 1.0, EMODE_NONE, 1.0);
                    events.list.push(SimEvent::Se(SE_CM_AM_NEEDLE_OUT));
                    events.list.push(SimEvent::Se(SE_CM_AM_MOUTH_CLOSE));
                    self.mode = MODE_ITAI_MOVE_RISE;
                    return false;
                }
                MODE_ITAI_MOVE_RISE => {
                    if !self.morph.is_stop() {
                        return false;
                    }
                    self.shapes.needle_arm();
                    self.countup[0] = 0;
                    self.step = Step::Dousa;
                    self.mode = MODE_DOUSA_CLOSE;
                    return false;
                }
                MODE_ITAI_MOVE_EAT => {
                    self.stts_weight = STTS_WEIGHT_EAT;
                    events.list.push(SimEvent::Se(SE_CM_AM_MOUTH_CLOSE));
                    self.anm_init(BCK_BOM_NOMI, 1.0, EMODE_NONE, 1.0);
                    self.eye_rot = [0; 3];
                    self.shapes.needle_sheathe();
                    self.countdown[CD_BOMB] = EAT_FRAMES;
                    self.mode = MODE_ITAI_MOVE_CHEW;
                    return true;
                }
                MODE_ITAI_MOVE_CHEW => {
                    let mouth = actor.pos + turned(actor.angle[1], SUCK_OFFSET);
                    let shape_yaw = self.shape_angle;
                    self.bomb_move_set(mouth, shape_yaw, false, live);
                    if self.morph.check_pass(3.0) {
                        events.list.push(SimEvent::Se(SE_CM_AM_EAT_BOMB));
                        events.list.push(SimEvent::Se(SE_CV_AM_EAT_BOMB));
                    }
                    if self.morph.check_pass(6.0) {
                        self.shapes.needle_arm();
                    }
                    if !self.morph.is_stop() {
                        return false;
                    }
                    self.countdown[CD_HOP] = BOUNCE_FRAMES;
                    self.target_angle = search_angle(actor.pos, player.pos);
                    self.mode = MODE_ITAI_MOVE_LEAP;
                    return false;
                }
                MODE_ITAI_MOVE_LEAP => {
                    let mouth = actor.pos + turned(actor.angle[1], SUCK_OFFSET);
                    let shape_yaw = self.shape_angle;
                    self.bomb_move_set(mouth, shape_yaw, true, live);
                    actor.angle[1] = actor.angle[1].wrapping_add(LEAP_SPIN);
                    if ground::grounded(solid, actor.pos) {
                        self.shock = Some((SHOCK_LEAP, SHOCK_MAGNITUDE, SHOCK_DIR));
                        events.list.push(SimEvent::Se(SE_CM_AM_JUMP));
                        events.list.push(SimEvent::Se(SE_CM_AM_JUMP_L));
                        events.list.push(SimEvent::Se(SE_CV_AM_JITABATA));
                        self.speed.y = LIFT_BOUNCE;
                        self.gravity = GRAVITY_LEAP;
                        self.speed_f = SPEED_LEAP;
                    }
                    if self.countdown[CD_HOP] != 0 {
                        return false;
                    }
                    self.anm_init(BCK_DEAD, 1.0, EMODE_NONE, 1.0);
                    events.list.push(SimEvent::Se(SE_CM_AM_BEF_EXPLODE));
                    self.target_angle = actor.angle[1];
                    self.speed_f = 0.0;
                    self.mode = MODE_ITAI_MOVE_BURST;
                    return false;
                }
                MODE_ITAI_MOVE_BURST => {
                    let mouth = actor.pos + turned(actor.angle[1], SUCK_OFFSET);
                    let shape_yaw = self.shape_angle;
                    self.bomb_move_set(mouth, shape_yaw, true, live);
                    if !self.morph.is_stop() {
                        return false;
                    }
                    if let Some(entity) = self.swallowed
                        && let Ok((_, _, mut shell)) = live.get_mut(entity)
                    {
                        shell.timer = BOMB_FUSE;
                        shell.life = bomb::Life::Wait;
                    }
                    events.list.push(SimEvent::Se(SE_CM_AM_EXPLODE));
                    events.list.push(SimEvent::Se(SE_LK_LAST_HIT));
                    let centre = Vec3::new(actor.pos.x, actor.pos.y + BURST_LIFT, actor.pos.z);
                    commands.spawn((
                        Actor {
                            name: iball::IBALL_ACTORS[0].to_owned(),
                            parameters: DROP_PARAM,
                            pos: centre,
                            angle: actor.angle,
                            set_id: 0,
                            room: actor.room,
                            ..Default::default()
                        },
                        iball::Iball::spawn(),
                    ));
                    self.doomed = true;
                    return false;
                }
                _ => return false,
            }
        }
    }

    fn fall_out(&mut self, actor: &mut Actor, events: &mut SimEvents) {
        self.anm_init(BCK_DEAD, 1.0, EMODE_NONE, 1.0);
        events.list.push(SimEvent::Se(SE_CM_AM_BEF_EXPLODE));
        self.target_angle = actor.angle[1];
        self.speed_f = 0.0;
        self.step = Step::ItaiMove;
        self.mode = MODE_ITAI_MOVE_BURST;
    }

    fn light_death(&mut self, actor: &Actor, commands: &mut Commands) {
        let pos = Vec3::new(
            actor.pos.x,
            actor.pos.y + self.light.y_offset,
            actor.pos.z,
        );
        commands.spawn((
            Actor {
                name: iball::IBALL_ACTORS[0].to_owned(),
                parameters: DROP_PARAM,
                pos,
                angle: actor.angle,
                set_id: 0,
                room: actor.room,
                ..Default::default()
            },
            iball::Iball::spawn(),
        ));
        self.doomed = true;
    }

    fn integrate(&mut self, actor: &mut Actor) {
        let (sin_y, cos_y) = walk::from_s16(actor.angle[1]).sin_cos();
        let cos_x = walk::from_s16(actor.angle[0]).cos();
        self.speed.x = self.speed_f * cos_x * sin_y;
        self.speed.z = self.speed_f * cos_x * cos_y;
        self.speed.y = (self.speed.y + self.gravity).max(MAX_FALL);
        actor.pos += self.speed;
    }

    fn bg_check(&self, actor: &mut Actor, solid: Option<&Collision>) {
        if let Some(solid) = solid {
            let from = Vec3::new(
                actor.pos.x,
                actor.pos.y - self.correction_y,
                actor.pos.z,
            );
            actor.pos.y -= self.correction_y;
            actor.pos = solid.slide(from, actor.pos);
            actor.pos.y += self.correction_y;
        }
    }

    pub fn execute(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        solid: Option<&Collision>,
        blow: Blow,
        blast: bool,
        busy: bool,
        switches: &Switches,
        shells: &[BombView],
        eaten: &[Entity],
        events: &mut SimEvents,
        health: &mut Health,
        live: &mut Shells<'_, '_>,
        commands: &mut Commands,
    ) {
        if self.enemy_ice(events) {
            if self.health <= LIGHT_DEAD {
                self.light_death(actor, commands);
            }
            return;
        }
        self.tick_timers();
        match self.step {
            Step::Dousa => self.action_dousa(
                actor, player, solid, blow, busy, switches, shells, eaten, events,
            ),
            Step::ModoruMove => self.action_modoru_move(actor, solid, events),
            Step::HandouMove => self.action_handou_move(actor, player, events),
            Step::ItaiMove => self.action_itai_move(
                actor, player, solid, shells, eaten, events, live, commands,
            ),
        }
        if self.step != Step::ItaiMove && self.spawn_pos_y - ABYSS_DROP > actor.pos.y {
            self.fall_out(actor, events);
        }
        stream::add_calc_angle_s2(&mut actor.angle[1], self.target_angle, TURN_DIV, TURN_CAP);
        if self.mode != MODE_ITAI_MOVE_LEAP
            && self.mode != MODE_ITAI_MOVE_BURST
            && self.mode != MODE_HANDOU_MOVE_MAIN
        {
            stream::add_calc_angle_s2(
                &mut self.shape_angle,
                actor.angle[1],
                TURN_DIV,
                TURN_CAP,
            );
        }
        if self.countdown[CD_BOMB] == 0 {
            self.morph.play();
        }
        let at = if blast && !matches!(blow.at, At::Hook | At::Light) {
            At::Bomb
        } else {
            blow.at
        };
        self.resolve(actor, player, at);
        self.body_atari_check(Blow { at, ..blow }, switches, events);
        self.spike(health);
        self.integrate(actor);
        self.bg_check(actor, solid);
    }
}

pub fn is_amos_knight(name: &str) -> bool {
    AMOS_KNIGHT_ACTORS.contains(&name)
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    switches: Res<'w, Switches>,
    combat: Res<'w, Combat>,
    hammer: Res<'w, hammer::Cut>,
    held: Res<'w, Held>,
    blasts: Res<'w, bombflower::Blasts>,
    players: Query<'w, 's, &'static Player>,
    shafts: Query<'w, 's, &'static arrow::Arrow>,
    bolts: Query<'w, 's, &'static boomerang::Boomerang>,
    chains: Query<'w, 's, &'static hookshot::Hookshot>,
    shells: Shells<'w, 's>,
}

fn active_at(watch: &Watch<'_, '_>) -> At {
    if watch
        .chains
        .iter()
        .any(|chain| chain.phase != hookshot::Phase::Wait)
    {
        return At::Hook;
    }
    for shaft in watch.shafts.iter() {
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
    mut watch: Watch<'_, '_>,
    mut health: ResMut<Health>,
    mut zones: ResMut<ZoneActors>,
    mut events: ResMut<SimEvents>,
    mut knights: Query<
        (Entity, &mut Actor, &mut AmosKnight),
        (
            With<AmosKnight>,
            Without<bomb::Bomb>,
            Without<Player>,
            Without<bombflower::Bombflower>,
            Without<bombflower::Crawler>,
        ),
    >,
    mut commands: Commands,
) {
    let solid = watch.collision.as_deref();
    let switches = *watch.switches;
    let blow = Blow {
        at: active_at(&watch),
        blade: watch.held.item,
        soup: watch.combat.soup,
        turn: watch.hammer.kind == hammer::CUT_HAMMER_SIDESWING,
    };
    let busy = watch.combat.active;
    let shells: Vec<BombView> = watch
        .shells
        .iter()
        .map(|(entity, actor, shell)| BombView {
            entity,
            pos: actor.pos,
            free: shell.life != bomb::Life::Carry,
            rest_time: shell.timer,
        })
        .collect();
    let Some(player) = watch.players.iter().next().copied() else {
        return;
    };
    let mut eaten: Vec<Entity> = Vec::new();
    let mut spent: Vec<(Entity, i8, u16)> = Vec::new();
    for (entity, mut actor, mut knight) in &mut knights {
        let actor = &mut *actor;
        let knight = &mut *knight;
        if !knight.entered {
            knight.begin(actor, &switches);
        }
        let blast = watch
            .blasts
            .list
            .iter()
            .any(|origin| touches(*origin, 0.0, actor.pos, bombflower::BLAST_RADIUS));
        knight.execute(
            actor,
            &player,
            solid,
            blow,
            blast,
            busy,
            &switches,
            &shells,
            &eaten,
            &mut events,
            &mut health,
            &mut watch.shells,
            &mut commands,
        );
        if let Some(shell) = knight.swallowed {
            eaten.push(shell);
        }
        if knight.doomed || knight.health <= 0 {
            spent.push((entity, actor.room, actor.set_id));
        }
    }
    for (entity, room, set_id) in spent {
        zones.mark_done(room, set_id);
        let _ = commands.entity(entity).despawn();
    }
}