use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::{Mat4, Vec3, Vec4};

use super::actor::Actor;
use super::arrow;
use super::bomb;
use super::boomerang;
use super::collision::Collision;
use super::combat::Combat;
use super::esa::Esa;
use super::events::{SimEvent, SimEvents};
use super::health::Health;
use super::player::Player;
use super::puppet_ganon::PuppetGanon;
use super::stream::add_calc_angle_s2;
use super::walk;

pub const BGN3_ACTORS: [&str; 0] = [];

pub const BGN3_ACTOR: &str = "Bgn3";


pub fn is_bgn3(name: &str) -> bool {
    BGN3_ACTORS.contains(&name)
}

pub fn is_bgn3_ks(name: &str) -> bool {
    BGN3_ACTORS.contains(&name)
}


pub const HIO_M05: u8 = 0;
pub const HIO_M06: u8 = 0;
pub const HIO_M08: f32 = 50.0;
pub const HIO_M0C: i16 = 800;
pub const HIO_M0E: i16 = 3;
pub const HIO_M10: f32 = 60.0;
pub const HIO_M14: i16 = 1000;
pub const HIO_M16: i16 = 3;
pub const HIO_M18: f32 = 80.0;
pub const HIO_M1C: i16 = 0x4b0;
pub const HIO_M1E: i16 = 3;
pub const HIO_M20: i16 = 0x3c;
pub const HIO_M22: i16 = 0x28;
pub const HIO_M24: i16 = 0x1e;
pub const HIO_M26: i16 = 10;
pub const HIO_M28: i16 = 0x14;
pub const HIO_M2A: i16 = 0x2d;
pub const HIO_M2C: i16 = 0x78;
pub const HIO_M2E: i16 = 200;
pub const HIO_M30: i16 = 0xfa;
pub const HIO_M34: f32 = 5.0;
pub const HIO_M38: f32 = 0.5;
pub const HIO_M3C: i16 = 3;
pub const HIO_M3E: u8 = 1;
pub const HIO_M40: f32 = 150.0;
pub const HIO_M44: f32 = 45.0;
pub const HIO_M4C: f32 = -8.0;
pub const HIO_M50: u8 = 0xbc;
pub const HIO_M52: u8 = 0x98;
pub const HIO_M54: u8 = 0x42;

pub const KE_MAX: [u16; 7] = [0x0028, 0x0023, 0x001E, 0x0019, 0x0014, 0x000F, 0x000F];
pub const SIZE_D: [f32; 9] = [1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.6];
pub const FL_CHECK_D: [i16; 9] =
    [0x0015, 0x0013, 0x0011, 0x000F, 0x000D, 0x000B, 0x0009, 0x0007, 0x0005];

pub const PARTS: usize = 9;
pub const PART_COUNT: usize = 10;
pub const PART_CORE: usize = 8;
pub const PART_TAIL: usize = 7;
pub const KE_SLOTS: usize = 5;
pub const KE_STEPS: usize = 5;
pub const KE_HAIRS: usize = 40;
pub const ROPE_POINTS: usize = 60;
pub const ROPE_STEP: f32 = 50.0;
pub const ROPE_FREQ: f32 = 0.053247336;
pub const ROPE_RATE: f32 = 0.01666667;
pub const ROPE_LAST: i32 = 59;
pub const ROPE_WIDTH: u8 = 10;
pub const COOL_SLOTS: usize = 5;

pub const FRAME_HEAD1: i16 = 48;
pub const FRAME_HAND: i16 = 48;

pub const AT_MOVE0: i16 = 0;
pub const AT_MAHI: i16 = 1;
pub const AT_DAMAGE: i16 = 2;
pub const AT_END: i16 = 3;
pub const AT_HIDDEN: i16 = 10;


pub const TIMER_HIDE: i16 = 180;
pub const TIMER_RING: i16 = 180;

pub const HIDDEN_Y: f32 = 30000.0;
pub const GROUND_Y: f32 = 150.0;
pub const WALL_REACH: f32 = 150.0;
pub const WALL_RISE: f32 = 100.0;
pub const HEAD_REACH: f32 = -300.0;
pub const NECK_Z: f32 = 250.0;
pub const GR_CHECK_UP: f32 = 200.0;
pub const GR_CHECK_DOWN: f32 = -1000.0;
pub const GR_CHECK_SNAP: f32 = -2.0;
pub const TAIL_STEP: f32 = 70.0;
pub const TAIL_REST: f32 = -5.0;
pub const PART_SPEED: f32 = 1.7;
pub const PART_RADIUS: f32 = 100.0;
pub const PART_FAN: f32 = 60.0;
pub const KE_BASE_Y: f32 = 100.0;
pub const TURN_SCALE: i16 = 4;
pub const TURN_STEP_Y: i16 = 0x1000;
pub const TURN_SCALE_X: i16 = 8;
pub const TURN_STEP_X: i16 = 0x400;
pub const PITCH_CAP: i16 = 0x3000;
pub const HALF_TURN: i16 = 0x8000;
pub const JUMP_Y: f32 = 150.0;
pub const FALL_Y: f32 = 130.0;
pub const BOUNCE_Y: f32 = 50.0;
pub const END_GRAVITY: f32 = 5.5;
pub const END_HEIGHT: f32 = 350.0;
pub const END_TAIL: f32 = 50.0;
pub const END_SPEED_F: f32 = -15.0;
pub const END_SPEED_Z: f32 = 375.17;
pub const END_PULL: f32 = 30.0;
pub const END_ACC: f32 = 0.05;
pub const END_TURN: i16 = 5000;
pub const END_TURN_RATE: i16 = 100;
pub const END_TURN_SCALE: i16 = 4;
pub const SPIN_ACC: f32 = 0.015;
pub const SPIN_MIN: f32 = 25.0;
pub const FLY_K: f32 = 500.0;
pub const FLASH_SWAY_Y: i16 = 0x2100;
pub const FLASH_SWAY_X: i16 = 0x2300;
pub const CRAWL_GATE: f32 = 10.0;
pub const ROTATE_SCALE: f32 = 3.5;
pub const ROPE_SCALE: f32 = 2.0;
pub const SE_GATE: u32 = 100;
pub const KI_CAP: i32 = 0x14;
pub const KI_PARAM: u32 = 3;
pub const SPLASH_HEAD: i16 = 2;
pub const SPLASH_HEAD_MASK: i16 = 3;
pub const SPLASH_TAIL_MASK: i16 = 3;
pub const SPLASH_MOVE_MASK: i16 = 7;
pub const LAG_RESET: f32 = 0.9;
pub const WALL_COOLDOWN: i16 = 0x14;
pub const HIT_COOLDOWN: i16 = 0x14;
pub const DEF_COOLDOWN: i16 = 10;
pub const LAST_COOLDOWN: i16 = 10000;
pub const SPEED_JITTER: f32 = 30.0;
pub const SPEED_JITTER_RANGE: f32 = 50.0;
pub const FLIP_CHANCE: f32 = 0.5;
pub const BIG_SCALE: f32 = 2.0;
pub const BODY_RADIUS: f32 = 200.0;
pub const HEAD_RADIUS: f32 = 150.0;
pub const BODY_ATP: u8 = 2;
pub const AT_STRONG: i16 = 9;
pub const AT_SPLIT: i16 = 2;
pub const AT_PLAIN: i16 = 1;
pub const HIT_PAUSE_STRONG: i32 = 5;
pub const HIT_PAUSE_SPLIT: i32 = 3;
pub const HIT_PAUSE_PLAIN: i32 = 2;
pub const HIT_FRAME_STRONG: i16 = 0x1e;
pub const HIT_FRAME_SPLIT: i16 = 0x19;
pub const HIT_FRAME_PLAIN: i16 = 0x14;
pub const HIT_FRAME_SPLIT2: i16 = 0xf;
pub const HIT_FRAME_ARROW: i16 = 10;
pub const HIT_FRAME_PLAIN2: i16 = 7;
pub const SWAY_SCALE: i16 = 0x900;
pub const SWAY_REACH: f32 = 4000.0;
pub const FLASH_CAP: i16 = 100;
pub const FLASH_TRIGGER: i16 = 0x17;

pub const SE_ARROW_HIT: u32 = 0x2879;
pub const SE_SURPRISE: u32 = 0x4830;
pub const SE_HIT_1: u32 = 0x496E;
pub const SE_HIT_LAST: u32 = 0x4970;
pub const SE_M_HIT_FACE: u32 = 0x4973;
pub const SE_HIT_WALL: u32 = 0x5981;
pub const SE_FACE_HIT: u32 = 0x5982;
pub const SE_BRK_ORB: u32 = 0x5983;
pub const SE_HEAD_RIPPLE: u32 = 0x6A44;
pub const SE_TAIL_RIPPLE: u32 = 0x6A45;
pub const SE_MECHA_ROTATE: u32 = 0x705A;
pub const SE_MECHA_ROPE: u32 = 0x705B;
pub const SE_CRAWL: u32 = 0x705D;

pub const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
pub const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
pub const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];

pub const CULL_MIN: [f32; 3] = [-3000.0, -2000.0, -3000.0];
pub const CULL_MAX: [f32; 3] = [3000.0, 3000.0, 3000.0];

pub const GRAVITY: f32 = -5.0;

pub const ATTN_BATTLE: i16 = 0x22;

pub const PARAM_MASK: u32 = 0xFF;

pub const HIO_NO: i8 = -1;

pub const HIO_M48: f32 = 8.0;

pub const HP_MAX: i16 = 3;

pub const WEAK_NONE: u8 = 0;
pub const WEAK_A: u8 = 1;
pub const WEAK_B: u8 = 2;
pub const WEAK_C: u8 = 3;

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Bgn3Hio {
    pub no: i8,
    pub m05: u8,
    pub m06: u8,
    pub m08: f32,
    pub m0c: i16,
    pub m0e: i16,
    pub m10: f32,
    pub m14: i16,
    pub m16: i16,
    pub m18: f32,
    pub m1c: i16,
    pub m1e: i16,
    pub m20: i16,
    pub m22: i16,
    pub m24: i16,
    pub m26: i16,
    pub m28: i16,
    pub m2a: i16,
    pub m2c: i16,
    pub m2e: i16,
    pub m30: i16,
    pub m34: f32,
    pub m38: f32,
    pub m3c: i16,
    pub m3e: u8,
    pub m40: f32,
    pub m44: f32,
    pub m48: f32,
    pub m4c: f32,
    pub m50: u8,
    pub m52: u8,
    pub m54: u8,
}

impl Default for Bgn3Hio {
    fn default() -> Self {
        Self {
            no: HIO_NO,
            m05: HIO_M05,
            m06: HIO_M06,
            m08: HIO_M08,
            m0c: HIO_M0C,
            m0e: HIO_M0E,
            m10: HIO_M10,
            m14: HIO_M14,
            m16: HIO_M16,
            m18: HIO_M18,
            m1c: HIO_M1C,
            m1e: HIO_M1E,
            m20: HIO_M20,
            m22: HIO_M22,
            m24: HIO_M24,
            m26: HIO_M26,
            m28: HIO_M28,
            m2a: HIO_M2A,
            m2c: HIO_M2C,
            m2e: HIO_M2E,
            m30: HIO_M30,
            m34: HIO_M34,
            m38: HIO_M38,
            m3c: HIO_M3C,
            m3e: HIO_M3E,
            m40: HIO_M40,
            m44: HIO_M44,
            m48: HIO_M48,
            m4c: HIO_M4C,
            m50: HIO_M50,
            m52: HIO_M52,
            m54: HIO_M54,
        }
    }
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Bgn3Static {
    pub hio: Bgn3Hio,
    pub hio_set: bool,
    pub ke_color: [u8; 4],
    pub ki_all_count: i32,
    pub ki_spawned: i32,
    pub next_pause_timer: i32,
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Bgn3Siblings {
    pub bgn_found: bool,
    pub bgn_phase: i16,
    pub bgn_cs_mode: i16,
    pub bgn_pos: Vec3,
    pub esa_found: bool,
    pub esa_pos: Vec3,
    pub ki_count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtKind {
    Sword,
    SwordSplit,
    Boomerang,
    Bomb,
    Arrow,
    LightArrow,
}

impl AtKind {
    pub fn code(self) -> i16 {
        match self {
            Self::SwordSplit => AT_SPLIT,
            Self::Sword => AT_PLAIN,
            Self::Boomerang => AT_PLAIN,
            Self::Bomb => AT_PLAIN,
            Self::Arrow => AT_PLAIN,
            Self::LightArrow => AT_STRONG,
        }
    }

    pub fn is_arrow(self) -> bool {
        matches!(self, Self::Arrow | Self::LightArrow)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct At {
    pub pos: Vec3,
    pub radius: f32,
    pub kind: AtKind,
    pub split: bool,
    pub frame: i16,
}

impl At {
    pub fn new(pos: Vec3, radius: f32, kind: AtKind) -> Self {
        Self {
            pos,
            radius,
            kind,
            split: false,
            frame: HIT_FRAME_PLAIN2,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ke {
    pub head: Vec3,
    pub pos: [Vec3; KE_STEPS],
    pub line: [Vec3; KE_STEPS],
}

impl Ke {
    pub fn new() -> Self {
        Self {
            head: Vec3::ZERO,
            pos: [Vec3::ZERO; KE_STEPS],
            line: [Vec3::ZERO; KE_STEPS],
        }
    }
}

impl Default for Ke {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct PartSph {
    pub centre: Vec3,
    pub radius: f32,
    pub core: bool,
    pub at_set: bool,
    pub tg_hit: bool,
}

impl PartSph {
    pub fn new() -> Self {
        Self {
            centre: Vec3::ZERO,
            radius: 0.0,
            core: false,
            at_set: false,
            tg_hit: false,
        }
    }
}

impl Default for PartSph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Part {
    pub pos: Vec3,
    pub vel: Vec3,
    pub angle_y: i16,
    pub angle_x: i16,
    pub scale: f32,
    pub sph: PartSph,
    pub flash_timer: i16,
    pub flash_effect: i16,
    pub weak: u8,
    pub hair: [Ke; KE_SLOTS],
}

impl Part {
    pub fn new() -> Self {
        Self {
            pos: Vec3::ZERO,
            vel: Vec3::ZERO,
            angle_y: 0,
            angle_x: 0,
            scale: 0.0,
            sph: PartSph::new(),
            flash_timer: 0,
            flash_effect: 0,
            weak: WEAK_NONE,
            hair: [Ke::new(); KE_SLOTS],
        }
    }
}

impl Default for Part {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bgn3Sph {
    pub centre: Vec3,
    pub radius: f32,
    pub at_set: bool,
    pub tg_hit: bool,
}

impl Bgn3Sph {
    pub fn new() -> Self {
        Self {
            centre: Vec3::ZERO,
            radius: 0.0,
            at_set: false,
            tg_hit: false,
        }
    }
}

impl Default for Bgn3Sph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bgn3Rope {
    pub pts: [Vec3; ROPE_POINTS],
    pub width: [u8; ROPE_POINTS],
}

impl Bgn3Rope {
    pub fn new() -> Self {
        Self {
            pts: [Vec3::ZERO; ROPE_POINTS],
            width: [ROPE_WIDTH; ROPE_POINTS],
        }
    }
}

impl Default for Bgn3Rope {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bgn3 {
    pub health: i16,
    pub max_health: i16,
    pub frame: i16,
    pub at: i16,
    pub inner: i16,
    pub turn: i16,
    pub turn_timer: i16,
    pub wall_count: i16,
    pub splash_timer: i16,
    pub lag: f32,
    pub tail: f32,
    pub hover: f32,
    pub rope_spin: f32,
    pub neck: Vec3,
    pub weak_pos: Vec3,
    pub cool_hit: i16,
    pub cool_def: i16,
    pub cool_speed: i16,
    pub cool_body: [i16; COOL_SLOTS],
    pub ring: i32,
    pub mode: i16,
    pub morf_frame: f32,
    pub brk_frame: [f32; 2],
    pub flash_timer: i16,
    pub flash_run: i16,
    pub head_flash: i16,
    pub head_effect: i16,
    pub parts: [Part; PART_COUNT],
    pub hairs: [Ke; KE_HAIRS],
    pub head_sph: Bgn3Sph,
    pub body_sph: Bgn3Sph,
    pub rope: Bgn3Rope,
    pub seed: [i32; 3],
    pub attention: i16,
}

impl Bgn3 {
    pub fn spawn(parameters: u32) -> Self {
        let _ = parameters & PARAM_MASK;
        Self {
            health: HP_MAX,
            max_health: HP_MAX,
            frame: 0,
            at: AT_HIDDEN,
            inner: 0,
            turn: 0,
            turn_timer: 0,
            wall_count: 0,
            splash_timer: 0,
            lag: 0.0,
            tail: 0.0,
            hover: 0.0,
            rope_spin: 0.0,
            neck: Vec3::ZERO,
            weak_pos: Vec3::ZERO,
            cool_hit: 0,
            cool_def: 0,
            cool_speed: 0,
            cool_body: [0; COOL_SLOTS],
            ring: 0,
            mode: 0,
            morf_frame: 0.0,
            brk_frame: [0.0; 2],
            flash_timer: 0,
            flash_run: 0,
            head_flash: 0,
            head_effect: 0,
            parts: [Part::new(); PART_COUNT],
            hairs: [Ke::new(); KE_HAIRS],
            head_sph: Bgn3Sph::new(),
            body_sph: Bgn3Sph::new(),
            rope: Bgn3Rope::new(),
            seed: [0; 3],
            attention: 0,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.at != AT_HIDDEN
    }

    pub fn hull(&self) -> ([f32; 3], [f32; 3]) {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        let mut add = |centre: Vec3, radius: f32| {
            min = min.min(centre - Vec3::splat(radius));
            max = max.max(centre + Vec3::splat(radius));
        };
        add(self.body_sph.centre, self.body_sph.radius);
        add(self.head_sph.centre, self.head_sph.radius);
        for part in self.parts.iter().take(PARTS) {
            add(part.sph.centre, part.sph.radius);
        }
        (
            [min.x, min.y, min.z],
            [max.x, max.y, max.z],
        )
    }

    pub fn touches(&self, at: &At) -> bool {
        if self.body_sph.at_set
            && walk::within(self.body_sph.centre, at.pos, self.body_sph.radius + at.radius)
        {
            return true;
        }
        for part in self.parts.iter().take(PART_TAIL) {
            if part.sph.at_set
                && walk::within(part.sph.centre, at.pos, part.sph.radius + at.radius)
            {
                return true;
            }
        }
        false
    }

    pub fn rnd_f(&mut self, range: f32) -> f32 {
        rnd(&mut self.seed) * range
    }

    pub fn rnd_fx(&mut self, range: f32) -> f32 {
        range * (rnd(&mut self.seed) - 0.5) * 2.0
    }
}

fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
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

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = (seed[0]) as f32 / RND_SCALE[0]
        + (seed[1]) as f32 / RND_SCALE[1]
        + (seed[2]) as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

fn ssin(angle: i16) -> f32 {
    walk::from_s16(angle).sin()
}

fn scos(angle: i16) -> f32 {
    walk::from_s16(angle).cos()
}

fn xrot(angle: i16) -> Mat4 {
    let a = walk::from_s16(angle);
    Mat4::from_cols(
        Vec4::X,
        Vec4::new(0.0, a.cos(), a.sin(), 0.0),
        Vec4::new(0.0, -a.sin(), a.cos(), 0.0),
        Vec4::W,
    )
}

fn yrot(angle: i16) -> Mat4 {
    let a = walk::from_s16(angle);
    Mat4::from_cols(
        Vec4::new(a.cos(), 0.0, -a.sin(), 0.0),
        Vec4::Y,
        Vec4::new(a.sin(), 0.0, a.cos(), 0.0),
        Vec4::W,
    )
}

fn zrot(angle: i16) -> Mat4 {
    let a = walk::from_s16(angle);
    Mat4::from_cols(
        Vec4::new(a.cos(), a.sin(), 0.0, 0.0),
        Vec4::new(-a.sin(), a.cos(), 0.0, 0.0),
        Vec4::Z,
        Vec4::W,
    )
}

fn point(m: &Mat4, v: Vec3) -> Vec3 {
    (m * Vec4::new(v.x, v.y, v.z, 1.0)).truncate()
}

fn lean_x(index: i32) -> i16 {
    ((((HIO_LEAN_X + index * HIO_LEAN_STEP_X) & HIO_LEAN_MASK_X) - HIO_LEAN_BASE_X) as i16)
        .wrapping_neg()
}

fn lean_z(index: i32) -> i16 {
    ((((HIO_LEAN_Z + index * HIO_LEAN_STEP_Z) & HIO_LEAN_MASK_Z) - HIO_LEAN_BASE_Z) as i16)
        .wrapping_neg()
}

fn lean_x_pos(index: i32) -> i16 {
    (((HIO_LEAN_X + index * HIO_LEAN_STEP_X) & HIO_LEAN_MASK_X) - HIO_LEAN_BASE_X) as i16
}

fn lean_z_pos(index: i32) -> i16 {
    (((HIO_LEAN_Z + index * HIO_LEAN_STEP_Z) & HIO_LEAN_MASK_Z) - HIO_LEAN_BASE_Z) as i16
}

pub const HIO_LEAN_X: i32 = 0x3FFF;
pub const HIO_LEAN_STEP_X: i32 = 1000;
pub const HIO_LEAN_BASE_X: i32 = 0x2000;
pub const HIO_LEAN_MASK_X: i32 = 0x3FFF;
pub const HIO_LEAN_Z: i32 = 0xFFF;
pub const HIO_LEAN_STEP_Z: i32 = 5000;
pub const HIO_LEAN_BASE_Z: i32 = 0x800;
pub const HIO_LEAN_MASK_Z: i32 = 0xFFF;

impl Bgn3 {
    pub fn gr_check(&self, at: Vec3, solid: Option<&Collision>, wet: bool) -> (Vec3, bool) {
        let up = at.y + GR_CHECK_UP;
        let down = at.y + GR_CHECK_DOWN;
        let Some(solid) = solid else {
            return (at, false);
        };
        match solid.ground_at(at.x, at.z) {
            Some(y) if y <= up && y >= down => (Vec3::new(at.x, GR_CHECK_SNAP, at.z), wet),
            _ => (at, false),
        }
    }

    pub fn tail_eff_set(
        &mut self,
        actor: &Actor,
        solid: Option<&Collision>,
        wet: bool,
        events: &mut SimEvents,
    ) {
        if (self.frame + SPLASH_HEAD) & SPLASH_HEAD_MASK == 0
            && self.gr_check(actor.pos, solid, wet).1
        {
            events.list.push(SimEvent::Se(SE_HEAD_RIPPLE));
            if self.splash_timer == 0 {
                self.splash_timer = (self.rnd_f(20.0) + 20.0) as i16;
            }
        }
        for i in 0..PARTS {
            if (self.frame + i as i16) & SPLASH_TAIL_MASK == 0 {
                let tip = self.parts[i].pos;
                if self.gr_check(tip, solid, wet).1 {
                    events.list.push(SimEvent::Se(SE_TAIL_RIPPLE));
                    if i == PART_CORE && self.splash_timer == 0 {
                        self.splash_timer = (self.rnd_f(20.0) + 20.0) as i16;
                    }
                }
            }
        }
    }

    pub fn move_splash_set(&self, actor: &Actor, solid: Option<&Collision>, wet: bool) -> bool {
        let mut any = false;
        if (self.frame + SPLASH_HEAD) & SPLASH_HEAD_MASK == 0
            && self.gr_check(actor.pos, solid, wet).1
        {
            any = true;
        }
        for i in 0..PARTS {
            if (self.frame + i as i16) & SPLASH_MOVE_MASK == 0
                && self.gr_check(self.parts[i].pos, solid, wet).1
            {
                any = true;
            }
        }
        any
    }

    pub fn check_ground(&mut self, actor: &mut Actor) -> bool {
        if actor.pos.y <= GROUND_Y {
            actor.pos.y = GROUND_Y;
            actor.speed.y = 0.0;
            true
        } else {
            false
        }
    }

    pub fn check_wall(&self, actor: &mut Actor, solid: Option<&Collision>) -> bool {
        let delta = actor.pos - actor.old.pos;
        let flat = Vec3::new(delta.x, 0.0, delta.z);
        if flat.length() <= 0.0 {
            return false;
        }
        let Some(solid) = solid else {
            return false;
        };
        let m = yrot(walk::to_s16(flat.x.atan2(flat.z)));
        let ahead = point(&m, Vec3::new(0.0, WALL_RISE, WALL_REACH)) + actor.pos;
        let from = Vec3::new(actor.pos.x, actor.pos.y + WALL_RISE, actor.pos.z);
        solid.slide(from, ahead) != ahead
    }

    pub fn pos_move(&mut self, actor: &mut Actor) {
        let m = yrot(actor.angle[1]);
        let v = point(&m, Vec3::new(0.0, 0.0, actor.speed_f));
        actor.speed.x = v.x;
        actor.speed.z = v.z;
        actor.pos += actor.speed;
        actor.speed.y += GRAVITY;
    }

    pub fn pitch_from_move(&self, actor: &mut Actor) {
        let delta = actor.pos - actor.old.pos;
        let flat = (delta.x * delta.x + delta.z * delta.z).sqrt();
        let pitch = -walk::to_s16(delta.y.atan2(flat));
        actor.angle[0] = pitch.clamp(-PITCH_CAP, PITCH_CAP);
    }

    pub fn part_control(&mut self, actor: &Actor, hio: &Bgn3Hio) {
        let lag = self.lag;
        let base = yrot(actor.shape_angle[1]) * xrot(actor.shape_angle[0]);
        let root = point(&base, Vec3::new(0.0, 0.0, -self.tail));
        self.parts[0].pos = self.neck;
        for i in 1..PART_COUNT {
            let prev = self.parts[i - 1];
            let next = self.parts[i];
            let reach = TAIL_STEP * prev.scale;
            let mut high = next.vel.y + (next.pos.y + TAIL_REST);
            if high <= reach {
                high = reach;
            }
            let flat_x = root.x + (next.vel.x + (next.pos.x - prev.pos.x));
            let rise = high - prev.pos.y;
            let flat_z = root.z + (next.vel.z + (next.pos.z - prev.pos.z));
            let span = (flat_x * flat_x + flat_z * flat_z).sqrt();
            self.parts[i - 1].angle_y = walk::to_s16(flat_x.atan2(flat_z));
            self.parts[i - 1].angle_x = -walk::to_s16(rise.atan2(span));
            let m = yrot(self.parts[i - 1].angle_y) * xrot(self.parts[i - 1].angle_x);
            let step = point(&m, Vec3::new(0.0, 0.0, 150.0 * prev.scale));
            self.parts[i].pos = prev.pos + step;
            self.parts[i].vel = lag * (self.parts[i].pos - next.pos);
        }
        add_calc2(&mut self.lag, hio.m38, 1.0, 0.002);
        add_calc2(&mut self.tail, hio.m34, 1.0, 0.5);
    }

    pub fn move0(
        &mut self,
        actor: &mut Actor,
        hio: &Bgn3Hio,
        esa: Option<Vec3>,
        player: Vec3,
        solid: Option<&Collision>,
        events: &mut SimEvents,
    ) {
        if esa.is_some() {
            self.turn_timer = 2;
        }
        if self.turn_timer != 0 {
            self.turn_timer -= 1;
            let mut max_speed = if self.turn < 0 {
                self.turn.wrapping_neg()
            } else {
                self.turn
            };
            let aim = if self.mode != 0 {
                walk::to_s16((-actor.pos.x).atan2(-actor.pos.z))
            } else if let Some(esa) = esa {
                let d = esa - actor.pos;
                max_speed = max_speed.wrapping_mul(2);
                walk::to_s16(d.x.atan2(d.z))
            } else {
                let sway = ssin(self.frame.wrapping_mul(SWAY_SCALE));
                let face = walk::to_s16((player.x - actor.pos.x).atan2(player.z - actor.pos.z));
                face.wrapping_add((SWAY_REACH * sway) as i32 as i16)
            };
            add_calc_angle_s2(&mut actor.angle[1], aim, TURN_SCALE, max_speed);
        } else {
            actor.angle[1] = actor.angle[1].wrapping_add(self.turn);
        }
        let want = if self.health == 3 {
            hio.m08
        } else if self.health == 2 {
            hio.m10
        } else {
            hio.m18
        };
        add_calc2(&mut actor.speed_f, want, 1.0, 3.0);
        self.pos_move(actor);
        self.check_ground(actor);
        let mut moved = false;
        if self.check_wall(actor, solid) && self.cool_body[1] == 0 {
            self.turn_timer = 0;
            actor.angle[1] = actor.angle[1].wrapping_add(HALF_TURN);
            self.lag = LAG_RESET;
            moved = true;
            self.cool_body[1] = WALL_COOLDOWN;
            events.list.push(SimEvent::Se(SE_HIT_WALL));
            self.ring = i32::from(hio.m3c);
            let cap = if self.health == 3 {
                hio.m0e
            } else if self.health == 2 {
                hio.m16
            } else {
                hio.m1e
            };
            self.wall_count += 1;
            if self.wall_count >= cap {
                self.wall_count = 0;
                self.turn_timer = if self.health == 3 {
                    hio.m20
                } else if self.health == 2 {
                    hio.m22
                } else {
                    hio.m24
                };
            }
        }
        if self.cool_speed == 0 {
            self.cool_speed = (self.rnd_f(SPEED_JITTER_RANGE) + SPEED_JITTER) as i16;
            moved = true;
        }
        if moved {
            self.turn = if self.health == 3 {
                hio.m0c
            } else if self.health == 2 {
                hio.m14
            } else {
                hio.m1c
            };
            if self.rnd_f(1.0) < FLIP_CHANCE {
                self.turn = self.turn.wrapping_neg();
            }
        }
        self.body_sph.at_set = true;
        self.head_sph.at_set = true;
        actor.angle[0] = 0;
    }

    pub fn mahi(&mut self, actor: &mut Actor, solid: Option<&Collision>) {
        match self.inner {
            0 => {
                self.inner = 1;
                self.morf_play();
                if self.cool_body[0] == 0 {
                    self.at = AT_MOVE0;
                    self.inner = 0;
                }
            }
            1 => {
                self.morf_play();
                if self.cool_body[0] == 0 {
                    self.at = AT_MOVE0;
                    self.inner = 0;
                }
            }
            _ => {}
        }
        add_calc0(&mut actor.speed_f, 1.0, 3.0);
        self.pos_move(actor);
        actor.angle[0] = 0;
        self.check_ground(actor);
        if self.check_wall(actor, solid) && self.cool_body[1] == 0 {
            self.turn_timer = 0;
            actor.angle[1] = actor.angle[1].wrapping_add(HALF_TURN);
            self.lag = LAG_RESET;
        }
    }

    pub fn morf_play(&mut self) {
        self.morf_frame = (self.morf_frame + 1.0).rem_euclid(f32::from(FRAME_HEAD1));
    }

    pub fn damage(&mut self, actor: &mut Actor, solid: Option<&Collision>) {
        match self.inner {
            0 => {
                actor.speed.y = JUMP_Y;
                actor.speed_f = 30.0;
                self.inner = 1;
                self.lag = 1.0;
            }
            1 => {
                add_calc2(&mut self.hover, 350.0, 1.0, 50.0);
                if self.check_ground(actor) {
                    actor.speed.y = BOUNCE_Y;
                    self.inner = 2;
                    actor.angle[1] = (self.rnd_f(0x10000) as i32) as i16;
                    self.lag = 1.0;
                }
            }
            2 => {
                if self.check_ground(actor) {
                    self.at = AT_MOVE0;
                    self.inner = 0;
                }
            }
            _ => {}
        }
        self.pos_move(actor);
        self.pitch_from_move(actor);
        self.check_ground(actor);
        if self.check_wall(actor, solid) && self.cool_body[1] == 0 {
            actor.angle[1] = actor.angle[1].wrapping_add(HALF_TURN);
            self.cool_body[1] = WALL_COOLDOWN;
        }
        self.body_sph.at_set = false;
        self.head_sph.at_set = false;
    }

    pub fn end(
        &mut self,
        actor: &mut Actor,
        sib: &mut Bgn3Siblings,
        events: &mut SimEvents,
    ) {
        match self.inner {
            0 => {
                actor.speed.y = JUMP_Y;
                actor.speed_f = 0.0;
                self.inner = 1;
                sib.bgn_cs_mode = 20;
                self.lag = 1.0;
                events.list.push(SimEvent::Se(SE_SURPRISE));
            }
            1 | 2 | 3 | 4 => {
                self.hover = actor.pos.y - 150.0;
                if self.hover > END_HEIGHT {
                    self.hover = END_HEIGHT;
                }
                if self.check_ground(actor) {
                    self.inner += 1;
                    actor.angle[1] = (self.rnd_f(0x10000) as i32) as i16;
                    self.lag = 1.0;
                    actor.speed.y = FALL_Y;
                    actor.speed_f = 0.0;
                    actor.angle[1] = (self.rnd_f(0x10000) as i32) as i16;
                    events.list.push(SimEvent::Se(SE_HIT_1));
                }
                if self.inner == 4 && actor.speed.y <= -30.0 {
                    self.inner = 5;
                    self.turn = 0;
                }
            }
            5 => {
                actor.speed.y += END_GRAVITY;
                if actor.speed.y > 0.0 {
                    add_calc2(&mut self.rope_spin, 1.0, 1.0, SPIN_ACC);
                }
                actor.angle[1] = actor.angle[1].wrapping_add(self.turn);
                actor.shape_angle[1] = actor.shape_angle[1].wrapping_add(self.turn);
                add_calc_angle_s2(&mut self.turn, END_TURN, END_TURN_SCALE, END_TURN_RATE);
                add_calc2(&mut self.tail, END_TAIL, 1.0, 1.0);
                add_calc0(&mut self.hover, 1.0, 50.0);
            }
            _ => {}
        }
        self.pos_move(actor);
        self.pitch_from_move(actor);
        self.check_ground(actor);
        self.body_sph.at_set = false;
        self.head_sph.at_set = false;
        add_calc2(&mut actor.pos.x, END_SPEED_F, END_ACC, END_PULL);
        add_calc2(&mut actor.pos.z, END_SPEED_Z, END_ACC, END_PULL);
    }

    pub fn ke_control(ke: &mut Ke, scale: f32, hio: &Bgn3Hio) {
        let step = Vec3::new(0.0, 0.0, scale);
        for i in 1..KE_STEPS {
            let taper = 0.1 * (KE_STEPS - i) as f32;
            let push = ke.head * taper;
            let flat_x = push.x + (ke.pos[i].x - ke.pos[i - 1].x);
            let flat_z = push.z + (ke.pos[i].z - ke.pos[i - 1].z);
            let rise = hio.m4c + (ke.pos[i].y + push.y) - ke.pos[i - 1].y;
            let pitch = -walk::to_s16(rise.atan2(flat_z));
            let yaw = walk::to_s16(flat_x.atan2((flat_x * flat_x + flat_z * flat_z).sqrt()));
            let m = xrot(pitch) * yrot(yaw);
            ke.pos[i] = ke.pos[i - 1] + point(&m, step);
        }
    }

    pub fn ke_move(ke: &mut Ke, scale: f32, hio: &Bgn3Hio) {
        if hio.m3e == 0 {
            return;
        }
        Self::ke_control(ke, scale, hio);
        ke.line = ke.pos;
    }

    pub fn move_se_set(&self, actor: &Actor, events: &mut SimEvents) {
        let mut delta = actor.pos - actor.old.pos;
        delta.y = 0.0;
        let rotate = (delta.length() * ROTATE_SCALE) as u32;
        events.list.push(SimEvent::Se(SE_MECHA_ROTATE));
        let _ = rotate.min(SE_GATE);
        delta = actor.pos - actor.old.pos;
        delta.z = 0.0;
        delta.x = 0.0;
        let rope = (delta.length() * ROPE_SCALE) as u32;
        events.list.push(SimEvent::Se(SE_MECHA_ROPE));
        let _ = rope.min(SE_GATE);
    }

    pub fn damage_check(
        &mut self,
        actor: &mut Actor,
        hits: &[At],
        stat: &mut Bgn3Static,
        events: &mut SimEvents,
    ) {
        self.head_sph.tg_hit = hits
            .iter()
            .any(|at| walk::within(self.head_sph.centre, at.pos, self.head_sph.radius + at.radius));
        self.body_sph.tg_hit = hits.iter().any(|at| {
            walk::within(self.body_sph.centre, at.pos, self.body_sph.radius + at.radius)
        });
        for i in 0..PARTS {
            let sphere = self.parts[i].sph;
            self.parts[i].sph.tg_hit = if i == PART_CORE {
                hits.iter().any(|at| {
                    at.kind == AtKind::LightArrow
                        && walk::within(sphere.centre, at.pos, sphere.radius + at.radius)
                })
            } else {
                hits.iter().any(|at| {
                    walk::within(sphere.centre, at.pos, sphere.radius + at.radius)
                })
            };
        }
        if self.cool_hit != 0 {
            return;
        }
        if self.head_sph.tg_hit {
            if let Some(hit) = hits.iter().find(|at| {
                walk::within(self.head_sph.centre, at.pos, self.head_sph.radius + at.radius)
            }) {
                events.list.push(SimEvent::Se(SE_M_HIT_FACE));
                events.list.push(SimEvent::Se(SE_FACE_HIT));
                let code = hit.kind.code();
                let hio = stat.hio;
                let cool = if code == AT_STRONG {
                    stat.next_pause_timer = HIT_PAUSE_STRONG;
                    hio.m30
                } else if code == AT_SPLIT {
                    hio.m2e
                } else if code == AT_PLAIN && hit.split {
                    stat.next_pause_timer = HIT_PAUSE_SPLIT;
                    hio.m2c
                } else if code == AT_PLAIN {
                    stat.next_pause_timer = HIT_PAUSE_PLAIN;
                    hio.m2a
                } else if hit.kind.is_arrow() {
                    hio.m28
                } else {
                    hio.m26
                };
                if self.at != AT_MAHI {
                    self.cool_body[0] = cool;
                    self.at = AT_MAHI;
                }
                self.inner = 0;
                self.cool_hit = HIT_COOLDOWN;
                self.flash_timer = hit.frame;
                return;
            }
        }
        if self.parts[PART_CORE].sph.tg_hit {
            self.cool_hit = HIT_COOLDOWN;
            self.flash_run = 1;
            self.at = AT_DAMAGE;
            self.inner = 0;
            if self.health != 0 {
                self.health -= 1;
                events.list.push(SimEvent::Se(SE_ARROW_HIT));
                if self.health == 0 {
                    events.list.push(SimEvent::Se(SE_HIT_LAST));
                    events.list.push(SimEvent::Se(SE_BRK_ORB));
                    self.at = AT_END;
                    self.inner = 0;
                    self.cool_hit = LAST_COOLDOWN;
                } else {
                    events.list.push(SimEvent::Se(SE_HIT_1));
                }
            }
        }
        let mut body_hit = self.body_sph.tg_hit;
        for i in 0..PART_TAIL {
            if self.parts[i].sph.tg_hit {
                body_hit = true;
            }
        }
        if body_hit && self.cool_def == 0 {
            self.cool_def = DEF_COOLDOWN;
        }
    }

    pub fn move_state(
        &mut self,
        actor: &mut Actor,
        hio: &Bgn3Hio,
        stat: &mut Bgn3Static,
        sib: &mut Bgn3Siblings,
        esa: Option<Vec3>,
        player: Vec3,
        solid: Option<&Collision>,
        wet: bool,
        hits: &[At],
        events: &mut SimEvents,
    ) {
        let mut moving = false;
        if hio.m05 == 0 {
            match self.at {
                AT_HIDDEN => {}
                AT_MOVE0 => {
                    self.move0(actor, hio, esa, player, solid, events);
                    if actor.speed_f > CRAWL_GATE {
                        moving = true;
                        self.move_splash_set(actor, solid, wet);
                        events.list.push(SimEvent::Se(SE_CRAWL));
                        self.move_se_set(actor, events);
                    }
                }
                AT_MAHI => self.mahi(actor, solid),
                AT_DAMAGE => self.damage(actor, solid),
                AT_END => self.end(actor, sib, events),
                _ => {}
            }
        }
        self.tail_eff_set(actor, solid, wet, events);
        self.damage_check(actor, hits, stat, events);
        add_calc_angle_s2(
            &mut actor.shape_angle[1],
            actor.angle[1],
            TURN_SCALE,
            TURN_STEP_Y,
        );
        add_calc_angle_s2(
            &mut actor.shape_angle[0],
            actor.angle[0],
            TURN_SCALE_X,
            TURN_STEP_X,
        );
        let base = yrot(actor.shape_angle[1]) * xrot(actor.shape_angle[0]);
        self.neck = actor.pos + point(&base, Vec3::new(0.0, 0.0, HEAD_REACH));
        for i in 0..PARTS {
            self.parts[i].scale = PART_SPEED * SIZE_D[i];
        }
        self.part_control(actor, hio);
        self.morf_play();
        self.brk_frame[0] = (self.brk_frame[0] + 1.0).rem_euclid(f32::from(FRAME_HAND));
        self.brk_frame[1] = (self.brk_frame[1] + 1.0).rem_euclid(f32::from(FRAME_HAND));
        let body = base * zrot(actor.shape_angle[2]) * Mat4::from_scale(Vec3::splat(BIG_SCALE));
        self.body_sph.centre = point(&body, Vec3::ZERO);
        self.body_sph.radius = BODY_RADIUS;
        if self.flash_run == FLASH_TRIGGER {
            self.head_flash = 50;
            self.head_effect = 100;
        }
        if self.head_effect != 0 {
            self.head_effect -= 1;
        }
        if self.head_flash != 0 {
            self.head_flash -= 1;
        }
        for i in 0..KE_HAIRS {
            let m = body * xrot(lean_x(i as i32)) * zrot(lean_z(i as i32));
            let root = point(&m, Vec3::new(0.0, KE_BASE_Y, 0.0));
            let mut ke = self.hairs[i];
            ke.pos[0] = root;
            ke.head = point(&m, Vec3::new(0.0, hio.m40, 0.0)) - self.body_sph.centre;
            Self::ke_move(&mut ke, 2.0 * hio.m44, hio);
            self.hairs[i] = ke;
        }
        if self.flash_timer != 0 {
            self.flash_timer -= 1;
        }
        let sway = f32::from(self.flash_timer) * FLY_K;
        let head = base
            * zrot(actor.shape_angle[2])
            * Mat4::from_translation(Vec3::new(0.0, 0.0, NECK_Z))
            * yrot((sway * ssin(self.frame.wrapping_mul(FLASH_SWAY_Y))) as i16)
            * xrot((sway * scos(self.frame.wrapping_mul(FLASH_SWAY_X))) as i16);
        self.head_sph.centre = point(&head, Vec3::ZERO);
        self.head_sph.radius = HEAD_RADIUS;
        if self.flash_run != 0 {
            self.flash_run += 1;
            if self.flash_run > FLASH_CAP {
                self.flash_run = 0;
            }
        }
        for i in 0..PARTS {
            let part = self.parts[i];
            let m = yrot(part.angle_y)
                * xrot(part.angle_x)
                * Mat4::from_scale(Vec3::splat(part.scale));
            let centre = point(&m, Vec3::ZERO);
            if i == PART_CORE {
                self.parts[i].weak = match self.health {
                    3 => WEAK_A,
                    2 => WEAK_B,
                    1 => WEAK_C,
                    _ => WEAK_NONE,
                };
                self.weak_pos = centre;
            }
            self.parts[i].sph.centre = centre;
            self.parts[i].sph.radius = part.scale * PART_RADIUS;
            self.parts[i].sph.at_set = moving;
            if i < PART_TAIL && hio.m3e != 0 {
                let slots = usize::from(KE_MAX.get(i).copied().unwrap_or(0));
                for j in 0..slots {
                    let slot = j % KE_SLOTS;
                    let km = m
                        * xrot(lean_x_pos(j as i32))
                        * zrot(lean_z_pos(j as i32))
                        * Mat4::from_translation(Vec3::new(
                            0.0,
                            part.scale * PART_FAN,
                            0.0,
                        ));
                    let mut ke = part.hair[slot];
                    ke.pos[0] = point(&km, Vec3::ZERO);
                    ke.head = point(&km, Vec3::new(0.0, hio.m40, 0.0)) - centre;
                    Self::ke_move(&mut ke, part.scale * hio.m44, hio);
                    self.parts[i].hair[slot] = ke;
                }
            }
            if self.flash_run == FL_CHECK_D.get(i).copied().unwrap_or(-1) {
                self.parts[i].flash_timer = 50;
                self.parts[i].flash_effect = 100;
            }
            if self.parts[i].flash_timer != 0 {
                self.parts[i].flash_timer -= 1;
            }
            if self.parts[i].flash_effect != 0 {
                self.parts[i].flash_effect -= 1;
            }
        }
        self.head_sph.at_set = moving;
        self.body_sph.at_set = moving;
        let rope_base = point(&body, Vec3::ZERO);
        add_calc0(&mut self.hover, 1.0, SPIN_MIN);
        for i in 0..ROPE_POINTS {
            let mut amp = self.hover * (ROPE_FREQ * i as f32).sin();
            amp *= ROPE_RATE * (ROPE_LAST - i as i32) as f32;
            let phase_x = (i32::from(self.frame) * 300 + i as i32 * 2000) as i16;
            let phase_z = (i32::from(self.frame) * 0xfa + i as i32 * 2000) as i16;
            let wobble = Vec3::new(amp * ssin(phase_x), 0.0, amp * ssin(phase_z));
            self.rope.pts[i] = rope_base + wobble + Vec3::Y * (ROPE_STEP * i as f32);
            self.rope.width[i] = ROPE_WIDTH;
        }
    }

    pub fn ki_check(&self, count: i32) -> i32 {
        count
    }

    pub fn execute(
        &mut self,
        actor: &mut Actor,
        stat: &mut Bgn3Static,
        sib: &mut Bgn3Siblings,
        player: Vec3,
        solid: Option<&Collision>,
        wet: bool,
        hits: &[At],
        events: &mut SimEvents,
    ) -> Option<(u32, Vec3, i8)> {
        if !stat.hio_set {
            stat.hio_set = true;
            stat.ke_color = [stat.hio.m50, stat.hio.m52, stat.hio.m54, 0xFF];
        }
        let hio = stat.hio;
        if hio.m06 != 0 {
            self.health = i16::from(hio.m06);
        }
        if !sib.bgn_found || sib.bgn_phase != 2 {
            self.at = AT_HIDDEN;
            actor.pos = Vec3::new(0.0, HIDDEN_Y, 0.0);
            return None;
        }
        if self.at == AT_HIDDEN {
            actor.pos = sib.bgn_pos;
            self.at = AT_MOVE0;
            self.turn_timer = TIMER_HIDE;
            self.mode = TIMER_RING;
        }
        self.attention = ATTN_BATTLE;
        stat.ki_all_count = self.ki_check(sib.ki_count);
        self.frame = self.frame.wrapping_add(1);
        for slot in self.cool_body.iter_mut() {
            if *slot != 0 {
                *slot -= 1;
            }
        }
        if self.cool_speed != 0 {
            self.cool_speed -= 1;
        }
        if self.cool_hit != 0 {
            self.cool_hit -= 1;
        }
        if self.cool_def != 0 {
            self.cool_def -= 1;
        }
        if self.splash_timer != 0 {
            self.splash_timer -= 1;
        }
        let esa = if sib.esa_found {
            Some(sib.esa_pos)
        } else {
            None
        };
        self.move_state(
            actor, &hio, stat, sib, esa, player, solid, wet, hits, events,
        );
        if self.ring != 0 {
            self.ring -= 1;
            if stat.ki_all_count < KI_CAP {
                let pos = Vec3::new(
                    self.rnd_fx(2500.0),
                    self.rnd_f(500.0) + 3500.0,
                    self.rnd_fx(2500.0),
                );
                return Some((KI_PARAM, pos, actor.room));
            }
        }
        None
    }
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    combat: Res<'w, Combat>,
    players: Query<'w, 's, (Entity, &'static Player), Without<Bgn3>>,
    arrows: Query<'w, 's, (&'static Actor, &'static arrow::Arrow), Without<Bgn3>>,
    bombs: Query<'w, 's, (&'static Actor, &'static bomb::Bomb), Without<Bgn3>>,
    bolts: Query<'w, 's, (&'static Actor, &'static boomerang::Boomerang), Without<Bgn3>>,
    bgns: Query<'w, 's, (&'static Actor, &'static PuppetGanon), Without<Bgn3>>,
    esas: Query<'w, 's, (&'static Actor, &'static Esa), Without<Bgn3>>,
    kis: Query<'w, 's, &'static Actor, Without<Bgn3>>,
}

pub fn control(
    watch: Watch<'_, '_>,
    mut stat: ResMut<Bgn3Static>,
    mut sib: ResMut<Bgn3Siblings>,
    mut events: ResMut<SimEvents>,
    mut health: ResMut<Health>,
    mut bosses: Query<(Entity, &mut Actor, &mut Bgn3), (With<Bgn3>, Without<Player>)>,
) {
    let solid = watch.collision.as_deref();
    let mut bgn: Option<(Vec3, u8)> = None;
    for (actor, boss) in watch.bgns.iter() {
        if bgn.is_none() {
            bgn = Some((actor.pos, boss.hio_phase));
        }
    }
    let esa = watch.esas.iter().next().map(|(actor, esa)| {
        let _ = esa;
        actor.pos
    });
    let ki = watch
        .kis
        .iter()
        .filter(|actor| is_bgn3_ks(&actor.name))
        .count() as i32;
    sib.bgn_found = bgn.is_some();
    if let Some((pos, phase)) = bgn {
        sib.bgn_pos = pos;
        sib.bgn_phase = i16::from(phase);
    } else {
        sib.bgn_phase = 0;
    }
    sib.esa_found = esa.is_some();
    if let Some(esa) = esa {
        sib.esa_pos = esa;
    }
    sib.ki_count = ki;
    let mut hits: Vec<At> = Vec::new();
    if watch.combat.active && watch.combat.frame >= 1 && watch.combat.frame < 6 {
        for (_, body) in watch.players.iter() {
            let mut at = At::new(body.pos + Vec3::new(0.0, 0.0, 40.0), 40.0, AtKind::Sword);
            at.split = watch.combat.soup;
            at.frame = HIT_FRAME_PLAIN;
            hits.push(at);
        }
    }
    for (actor, shaft) in watch.arrows.iter() {
        if shaft.phase != arrow::Phase::Flying {
            continue;
        }
        let kind = if shaft.kind == arrow::TYPE_LIGHT {
            AtKind::LightArrow
        } else {
            AtKind::Arrow
        };
        let mut at = At::new(actor.pos, 12.0, kind);
        at.frame = if kind == AtKind::LightArrow {
            HIT_FRAME_STRONG
        } else {
            HIT_FRAME_ARROW
        };
        hits.push(at);
    }
    for (actor, bolt) in watch.bolts.iter() {
        if bolt.parameter != boomerang::MODE_THROW {
            continue;
        }
        let mut at = At::new(actor.pos, 20.0, AtKind::Boomerang);
        at.frame = HIT_FRAME_PLAIN2;
        hits.push(at);
    }
    for (actor, shell) in watch.bombs.iter() {
        if shell.life != bomb::Life::Wait {
            continue;
        }
        let mut at = At::new(actor.pos, 30.0, AtKind::Bomb);
        at.frame = HIT_FRAME_PLAIN;
        hits.push(at);
    }
    let player = watch
        .players
        .iter()
        .next()
        .map_or(Vec3::ZERO, |(_, body)| body.pos);
    let wet = false;
    let mut spawned = 0i32;
    for (_, actor, boss) in bosses.iter_mut() {
        let (actor, boss) = (&mut *actor, &mut *boss);
        if boss
            .execute(
                actor,
                &mut stat,
                &mut sib,
                player,
                solid,
                wet,
                &hits,
                &mut events,
            )
            .is_some()
        {
            spawned += 1;
        }
        for at in hits.iter() {
            if boss.touches(at) {
                health.damage(BODY_ATP);
            }
        }
    }
    stat.ki_spawned = spawned;
}
