use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::arrow;
use super::collision::Collision;
use super::combat::{Combat, CutKind};
use super::events::SimEvents;
use super::inventory;
use super::player::Player;
use super::stream::{add_calc_angle_s2, cos_s, sin_s};
use super::walk::{from_s16, to_s16};
use super::water::Water;

/// `"Bgn3"` has NO row in the `OBJNAME` table (`d_stage.cpp`, 823 rows) - `fpcNm_BGN3_e` is only
/// ever brought into being at runtime by `fopAcM_create(fpcNm_BGN3_e, 0, ...)`
/// (`d_a_bgn.cpp:3674`), never placed by a stage. So it cannot be a stage claim. This module
/// borrows the single crate-wide declaration from `puppet_ganon`, which owns the matching
/// `fpcNm_BGN2_e`/`fpcNm_BGN3_e` search pair (`d_a_bgn.cpp:931`, `:940`).
pub const GANON_FORM3_ACTOR: &str = super::puppet_ganon::BODY_3_ACTOR;
pub const KEETH_SPAWN_NAME: &str = "keeth";
pub const KEETH_SPAWN_PARAMETERS: u32 = 0xFFFF_0003;
pub const BGN_CS_MODE_DEATH: i8 = 20;

pub fn is_ganon_form3(name: &str) -> bool {
    name == GANON_FORM3_ACTOR
}

const PART_COUNT: usize = 9;
const BODY_PARTS: usize = 8;
const WEAK_POINT: usize = 8;
const KEETH_MAX: i32 = 0x14;

const HIO_M05: u8 = 0;
const HIO_M06: u8 = 0;
const HIO_M08: f32 = 50.0;
const HIO_M0C: i16 = 800;
const HIO_M0E: i16 = 3;
const HIO_M10: f32 = 60.0;
const HIO_M14: i16 = 1000;
const HIO_M16: i16 = 3;
const HIO_M18: f32 = 80.0;
const HIO_M1C: i16 = 0x4b0;
const HIO_M1E: i16 = 3;
const HIO_M20: i16 = 0x3c;
const HIO_M22: i16 = 0x28;
const HIO_M24: i16 = 0x1e;
const HIO_M26: i16 = 10;
const HIO_M28: i16 = 0x14;
const HIO_M2A: i16 = 0x2d;
const HIO_M2C: i16 = 0x78;
const HIO_M2E: i16 = 200;
const HIO_M30: i16 = 0xfa;
const HIO_M34: f32 = 5.0;
const HIO_M38: f32 = 0.5;
const HIO_M3C: i16 = 3;

const REG0_F3: f32 = 0.0;
const REG0_F4: f32 = 0.0;
const REG0_F5: f32 = 0.0;
const REG0_F6: f32 = 0.0;
const REG0_F7: f32 = 0.0;
const REG0_F8: f32 = 0.0;
const REG0_F9: f32 = 0.0;
const REG0_F10: f32 = 0.0;
const REG0_F11: f32 = 0.0;
const REG0_F12: f32 = 0.0;
const REG0_F14: f32 = 0.0;
const REG0_F18: f32 = 0.0;
const REG0_F19: f32 = 0.0;
const REG0_S9: i16 = 0;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const SE_LK_SW_HIT_S: u32 = 0x2803;
const SE_LK_HAMMER_HIT: u32 = 0x2855;
const SE_LK_ARROW_HIT: u32 = 0x2879;
const SE_CV_BK_SURPRISE: u32 = 0x4830;
const SE_CV_BGN_HIT_1: u32 = 0x496E;
const SE_CV_BGN_HIT_LAST: u32 = 0x4970;
const SE_CV_BGN_M_HIT_FACE: u32 = 0x4973;
const SE_CM_BGN_M_HIT_WALL: u32 = 0x5981;
const SE_CM_BGN_M_FACE_HIT: u32 = 0x5982;
const SE_CM_BGN_M_BRK_ORB: u32 = 0x5983;
const SE_CM_BGN_M_HEAD_RIPPLE: u32 = 0x6A44;
const SE_CM_BGN_M_TAIL_RIPPL: u32 = 0x6A45;
const SE_CM_BGN_MECHA_ROTATE: u32 = 0x705A;
const SE_CM_BGN_MECHA_ROPE: u32 = 0x705B;
const SE_CM_BGN_M_CRAWL: u32 = 0x705D;

const MAX_HEALTH: i32 = 3;
const BGN_MODE_FIGHT: i8 = 2;

const GR_TOP: f32 = 200.0;
const GR_DOWN: f32 = 1000.0;
const GR_WATER_Y: f32 = -2.0;
const GROUND_Y: f32 = 150.0;
const GRAVITY: f32 = -5.0;
const HEAD_RADIUS: f32 = 150.0;
const BODY_RADIUS: f32 = 200.0;
const PART_RADIUS: f32 = 100.0;
const PART_SPAN: f32 = 150.0;
const PART_REST: f32 = 70.0;
const ROOT_BACK: f32 = -300.0;
const NECK_FWD: f32 = 250.0;
const ATTN_LOCKON_BATTLE: i32 = 1;
const ATTN_DIST_BATTLE: i32 = 0x22;
const ATTN_Y: f32 = 100.0;
const HIT_LOCK: i16 = 0x14;
const HIT_LOCK_FINAL: i16 = 10000;
const TILT_MAX: i16 = 0x3000;
const WALL_LOCK: i16 = 0x14;
const WALL_REACH: f32 = 150.0;
const WALL_PROBE_Y: f32 = 100.0;
const WALL_RISE: f32 = 50.0;
const PARKED_Y: f32 = 30000.0;
const TURN_HOME: i16 = 180;
const END_SPIN_TARGET: i16 = 5000;
const END_SPIN_STEP: i16 = 100;
const BODY_HIT_SE_LOCK: i16 = 10;
const RND_ANGLE: f32 = 65536.0;

pub const CULL_MIN: [f32; 3] = [-400.0, -1000.0, -400.0];
pub const CULL_MAX: [f32; 3] = [400.0, 1200.0, 400.0];

const SIZE_D: [f32; PART_COUNT] = [1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.6];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat {
    pub x: Vec3,
    pub y: Vec3,
    pub z: Vec3,
    pub t: Vec3,
}

impl Mat {
    const fn none() -> Self {
        Self {
            x: Vec3::X,
            y: Vec3::Y,
            z: Vec3::Z,
            t: Vec3::ZERO,
        }
    }

    fn pos(&self, src: Vec3) -> Vec3 {
        self.t + self.x * src.x + self.y * src.y + self.z * src.z
    }

    fn concat(&mut self, a: Vec3, b: Vec3, c: Vec3) {
        let x = self.x * a.x + self.y * a.y + self.z * a.z;
        let y = self.x * b.x + self.y * b.y + self.z * b.z;
        let z = self.x * c.x + self.y * c.y + self.z * c.z;
        self.x = x;
        self.y = y;
        self.z = z;
    }

    fn trans(&mut self, at: Vec3, concat: bool) {
        if concat {
            self.t += at;
        } else {
            self.x = Vec3::X;
            self.y = Vec3::Y;
            self.z = Vec3::Z;
            self.t = at;
        }
    }

    fn scale(&mut self, at: f32) {
        self.x *= at;
        self.y *= at;
        self.z *= at;
    }

    fn xrot_s(&mut self, angle: i16) {
        let (sin, cos) = from_s16(angle).sin_cos();
        self.x = Vec3::X;
        self.y = Vec3::new(0.0, cos, sin);
        self.z = Vec3::new(0.0, -sin, cos);
        self.t = Vec3::ZERO;
    }

    fn xrot_m(&mut self, angle: i16) {
        let (sin, cos) = from_s16(angle).sin_cos();
        self.concat(
            Vec3::X,
            Vec3::new(0.0, cos, sin),
            Vec3::new(0.0, -sin, cos),
        );
    }

    fn yrot_s(&mut self, angle: i16) {
        let (sin, cos) = from_s16(angle).sin_cos();
        self.x = Vec3::new(cos, 0.0, -sin);
        self.y = Vec3::Y;
        self.z = Vec3::new(sin, 0.0, cos);
        self.t = Vec3::ZERO;
    }

    fn yrot_m(&mut self, angle: i16) {
        let (sin, cos) = from_s16(angle).sin_cos();
        self.concat(
            Vec3::new(cos, 0.0, -sin),
            Vec3::Y,
            Vec3::new(sin, 0.0, cos),
        );
    }

    fn zrot_m(&mut self, angle: i16) {
        let (sin, cos) = from_s16(angle).sin_cos();
        self.concat(
            Vec3::new(cos, sin, 0.0),
            Vec3::new(-sin, cos, 0.0),
            Vec3::Z,
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Part {
    pub pos: Vec3,
    pub delta: Vec3,
    pub rot_x: i16,
    pub rot_y: i16,
    pub scale: f32,
    pub at: bool,
}

impl Part {
    const fn idle() -> Self {
        Self {
            pos: Vec3::ZERO,
            delta: Vec3::ZERO,
            rot_x: 0,
            rot_y: 0,
            scale: 0.0,
            at: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum At {
    Idle,
    Sword,
    SwordFinish,
    Hammer,
    Bomb,
    Arrow,
    LightArrow,
    Other,
}

impl At {
    fn reach(self) -> f32 {
        match self {
            At::Idle | At::Other => 0.0,
            At::Sword | At::SwordFinish => 20.0,
            At::Arrow | At::LightArrow => 35.0,
            At::Hammer => 50.0,
            At::Bomb => 150.0,
        }
    }

    fn def_se(self) -> u32 {
        match self {
            At::Hammer => SE_LK_HAMMER_HIT,
            At::Arrow | At::LightArrow => SE_LK_ARROW_HIT,
            _ => SE_LK_SW_HIT_S,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Blow {
    pub at: At,
    pub pos: Option<Vec3>,
}

pub struct Env<'a> {
    pub player_pos: Vec3,
    pub solid: Option<&'a Collision>,
    pub water: Option<&'a Water>,
    pub blow: Blow,
    pub esa: Option<Vec3>,
    pub keeths: i32,
    pub bgn_mode: Option<i8>,
}

#[derive(Debug, Clone, Copy)]
pub struct KeethSpawn {
    pub pos: Vec3,
    pub room: i8,
}

pub struct Sink<'a> {
    pub events: &'a mut SimEvents,
    pub bgn_cs_mode: &'a mut i8,
    pub keeth: &'a mut Vec<KeethSpawn>,
}

impl Sink<'_> {
    fn se(&mut self, id: u32) {
        self.events.list.push(super::events::SimEvent::Se(id));
    }

    fn bgm_stop(&mut self) {
        self.events.list.push(super::events::SimEvent::Bgm(0));
    }
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

fn atan2s(f0: f32, f1: f32) -> i16 {
    to_s16(f0.atan2(f1))
}

fn spread(fx: f32, fz: f32) -> f32 {
    (fx * fx + fz * fz).sqrt()
}

fn length_of(v: Vec3) -> f32 {
    (v.x * v.x + v.y * v.y + v.z * v.z).sqrt()
}

fn reaches(centre: Vec3, radius: f32, at: Vec3, reach: f32) -> bool {
    (at - centre).length_squared() <= (radius + reach) * (radius + reach)
}

fn gr_check(at: &mut Vec3, env: &Env<'_>) -> bool {
    let top = Vec3::new(at.x, at.y + GR_TOP, at.z);
    let bottom = Vec3::new(at.x, at.y - GR_DOWN, at.z);
    let hit = match env.solid {
        Some(solid) => solid.ground_at(at.x, at.z),
        None => None,
    };
    if let Some(y) = hit
        && y <= top.y
        && y >= bottom.y
    {
        at.y = GR_WATER_Y;
        return match env.water {
            Some(water) => !water.in_area,
            None => true,
        };
    }
    true
}

fn tilt_of(delta: Vec3) -> i16 {
    let mut tilt = atan2s(delta.y, spread(delta.x, delta.z)).wrapping_neg();
    if tilt > TILT_MAX {
        tilt = TILT_MAX;
    } else if tilt < TILT_MAX.wrapping_neg() {
        tilt = TILT_MAX.wrapping_neg();
    }
    tilt
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct GanonForm3 {
    pub health: i32,
    pub max_health: i32,
    pub home: Vec3,
    pub old_pos: Vec3,
    pub entered: bool,
    pub speed: Vec3,
    pub speed_f: f32,
    pub shape_angle: [i16; 3],
    pub eye: Vec3,
    pub attention: Vec3,
    pub attn_flags: i32,
    pub attn_dist: i32,
    pub root: Vec3,
    pub body_centre: Vec3,
    pub parts: [Part; PART_COUNT],
    pub weak_point: Vec3,
    pub head_at: bool,
    pub body_at: bool,
    pub frame: i16,
    pub mode: i16,
    pub sub: i16,
    pub turn: i16,
    pub turn_state: i16,
    pub wall_count: i16,
    pub ripple: i16,
    pub timers: [i16; 5],
    pub hit_lock: i16,
    pub body_hit_se: i16,
    pub sway: f32,
    pub reach: f32,
    pub wobble: i16,
    pub glow: f32,
    pub float_pos: f32,
    pub keeth_timer: i32,
    pub next_pause: i16,
    pub seed: [i32; 3],
}

impl GanonForm3 {
    pub fn spawn(parameters: u32) -> Self {
        let _ = parameters;
        Self {
            health: MAX_HEALTH,
            max_health: MAX_HEALTH,
            home: Vec3::ZERO,
            old_pos: Vec3::ZERO,
            entered: false,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            shape_angle: [0; 3],
            eye: Vec3::ZERO,
            attention: Vec3::ZERO,
            attn_flags: 0,
            attn_dist: 0,
            root: Vec3::ZERO,
            body_centre: Vec3::ZERO,
            parts: [Part::idle(); PART_COUNT],
            weak_point: Vec3::ZERO,
            head_at: false,
            body_at: false,
            frame: 0,
            mode: 0,
            sub: 0,
            turn: 0,
            turn_state: 0,
            wall_count: 0,
            ripple: 0,
            timers: [0; 5],
            hit_lock: 0,
            body_hit_se: 0,
            sway: 0.0,
            reach: 0.0,
            wobble: 0,
            glow: 0.0,
            float_pos: 0.0,
            keeth_timer: 0,
            next_pause: 0,
            seed: [RND_SEED; 3],
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.mode != 10
    }

    fn rnd_f(&mut self, range: f32) -> f32 {
        rnd(&mut self.seed) * range
    }

    fn rnd_fx(&mut self, range: f32) -> f32 {
        range * (rnd(&mut self.seed) - 0.5) * 2.0
    }

    fn player_yaw(env: &Env<'_>, from: Vec3) -> i16 {
        atan2s(env.player_pos.x - from.x, env.player_pos.z - from.z)
    }

    fn part(&self, index: usize) -> Part {
        self.parts.get(index).copied().unwrap_or(Part::idle())
    }

    fn shape_mtx(&self) -> Mat {
        let mut mtx = Mat::none();
        mtx.yrot_s(self.shape_angle[1]);
        mtx.xrot_m(self.shape_angle[0]);
        mtx
    }

    fn tail_eff_set(&mut self, actor: &Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        if (self.frame as i32 + 2) & 3 == 0 {
            let mut local_38 = actor.pos;
            if !gr_check(&mut local_38, env) && self.ripple == 0 {
                self.ripple = self.rnd_f(20.0) as i16 + 20;
                out.se(SE_CM_BGN_M_HEAD_RIPPLE);
            }
        }
        for i in 0..PART_COUNT {
            if (self.frame as i32 + i as i32) & 3 != 0 {
                continue;
            }
            let mut local_38 = self.part(i).pos;
            if !gr_check(&mut local_38, env) && i == WEAK_POINT && self.ripple == 0 {
                self.ripple = self.rnd_f(20.0) as i16 + 20;
                out.se(SE_CM_BGN_M_TAIL_RIPPL);
            }
        }
    }

    fn part_control(&mut self) {
        let dvar9 = self.sway;
        let dvar8 = REG0_F8 + PART_SPAN;
        let dvar10 = REG0_F6 - 5.0;
        self.parts[0].pos = self.root;
        let base = self.shape_mtx();
        let local_c8 = base.pos(Vec3::new(0.0, 0.0, -self.reach));
        let mut mtx = base;
        for i in 1..PART_COUNT {
            let prev = self.parts[i - 1];
            let cur = self.parts[i];
            let fvar1 = (PART_REST + REG0_F19) * cur.scale;
            let mut fvar2 = cur.delta.y + (cur.pos.y + dvar10);
            if fvar2 <= fvar1 {
                fvar2 = fvar1;
            }
            let fx = local_c8.x + (cur.delta.x + (cur.pos.x - prev.pos.x));
            let dvar11 = fvar2 - prev.pos.y;
            let fz = local_c8.z + (cur.delta.z + (cur.pos.z - prev.pos.z));
            self.parts[i - 1].rot_y = atan2s(fx, fz);
            self.parts[i - 1].rot_x = atan2s(dvar11, spread(fx, fz)).wrapping_neg();
            mtx.yrot_s(self.parts[i - 1].rot_y);
            mtx.xrot_m(self.parts[i - 1].rot_x);
            let cstack = mtx.pos(Vec3::new(0.0, 0.0, dvar8 * prev.scale));
            let mut delta = cur.pos;
            self.parts[i].pos = prev.pos + cstack;
            delta.x = dvar9 * (self.parts[i].pos.x - delta.x);
            delta.y = dvar9 * (self.parts[i].pos.y - delta.y);
            delta.z = dvar9 * (self.parts[i].pos.z - delta.z);
            self.parts[i].delta = delta;
        }
        add_calc2(&mut self.sway, HIO_M38, 1.0, 0.002);
        add_calc2(&mut self.reach, HIO_M34, 1.0, 0.5);
    }

    fn check_wall(&self, actor: &Actor, env: &Env<'_>) -> bool {
        let mut local_ac = actor.pos - self.old_pos;
        local_ac.y = 0.0;
        if length_of(local_ac) <= 0.0 {
            return false;
        }
        let mut mtx = Mat::none();
        mtx.yrot_s(atan2s(local_ac.x, local_ac.z));
        let ahead = mtx.pos(Vec3::new(0.0, WALL_PROBE_Y, REG0_F4 + WALL_REACH)) + actor.pos;
        let Some(solid) = env.solid else {
            return false;
        };
        let mid = Vec3::new(
            (actor.pos.x + ahead.x) * 0.5,
            actor.pos.y + WALL_PROBE_Y,
            (actor.pos.z + ahead.z) * 0.5,
        );
        for at in [mid, Vec3::new(ahead.x, actor.pos.y + WALL_PROBE_Y, ahead.z)] {
            if let Some(y) = solid.ground_at(at.x, at.z)
                && y > actor.pos.y + WALL_RISE
            {
                return true;
            }
        }
        false
    }

    fn check_ground(&mut self, actor: &mut Actor) -> bool {
        if actor.pos.y <= REG0_F9 + GROUND_Y {
            actor.pos.y = REG0_F9 + GROUND_Y;
            self.speed.y = 0.0;
            return true;
        }
        false
    }

    fn pos_move(&mut self, actor: &mut Actor) {
        let mut mtx = Mat::none();
        mtx.yrot_s(actor.angle[1]);
        let local_28 = mtx.pos(Vec3::new(0.0, 0.0, self.speed_f));
        self.speed.x = local_28.x;
        self.speed.z = local_28.z;
        actor.pos += self.speed;
        self.speed.y += REG0_F3 + GRAVITY;
    }

    fn move0(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        if env.esa.is_some() {
            self.turn_state = 2;
        }
        if self.turn_state != 0 {
            self.turn_state -= 1;
            let mut max_speed = self.turn;
            if max_speed < 0 {
                max_speed = max_speed.wrapping_neg();
            }
            let target;
            if self.timers[3] != 0 {
                target = atan2s(-actor.pos.x, -actor.pos.z);
            } else if let Some(esa) = env.esa {
                let local_4c = esa - actor.pos;
                target = atan2s(local_4c.x, local_4c.z);
                max_speed = max_speed.wrapping_mul(2);
            } else {
                let dvar6 = sin_s((self.frame as i32 * (REG0_S9 as i32 + 0x900)) as i16);
                let base = Self::player_yaw(env, actor.pos);
                target = base.wrapping_add(((REG0_F18 + 4000.0) * dvar6) as i16);
            }
            add_calc_angle_s2(&mut actor.angle[1], target, 4, max_speed);
        } else {
            actor.angle[1] = actor.angle[1].wrapping_add(self.turn);
        }
        let speed = match self.health {
            3 => HIO_M08,
            2 => HIO_M10,
            _ => HIO_M18,
        };
        add_calc2(&mut self.speed_f, speed, 1.0, 3.0);
        self.pos_move(actor);
        self.check_ground(actor);
        let mut bvar1 = false;
        if self.check_wall(actor, env) && self.timers[1] == 0 {
            self.turn_state = 0;
            actor.angle[1] = actor.angle[1].wrapping_add(-32768);
            self.sway = REG0_F5 + 0.9;
            bvar1 = true;
            self.timers[1] = WALL_LOCK;
            out.se(SE_CM_BGN_M_HIT_WALL);
            self.keeth_timer = HIO_M3C as i32;
            let hits = match self.health {
                3 => HIO_M0E,
                2 => HIO_M16,
                _ => HIO_M1E,
            };
            self.wall_count += 1;
            if self.wall_count >= hits {
                self.wall_count = 0;
                self.turn_state = match self.health {
                    3 => HIO_M20,
                    2 => HIO_M22,
                    _ => HIO_M24,
                };
            }
        }
        if self.timers[2] == 0 {
            self.timers[2] = self.rnd_f(50.0) as i16 + 30;
            bvar1 = true;
        }
        if bvar1 {
            self.turn = match self.health {
                3 => HIO_M0C,
                2 => HIO_M14,
                _ => HIO_M1C,
            };
            if self.rnd_f(1.0) < 0.5 {
                self.turn = self.turn.wrapping_neg();
            }
        }
        self.body_at = true;
        self.head_at = true;
        actor.angle[0] = 0;
    }

    fn mahi(&mut self, actor: &mut Actor, env: &Env<'_>) {
        if self.sub == 0 {
            self.sub = 1;
        }
        if self.sub == 1 && self.timers[0] == 0 {
            self.mode = 0;
            self.sub = 0;
        }
        add_calc0(&mut self.speed_f, 1.0, 3.0);
        self.pos_move(actor);
        actor.angle[0] = 0;
        self.check_ground(actor);
        if self.check_wall(actor, env) && self.timers[1] == 0 {
            self.turn_state = 0;
            actor.angle[1] = actor.angle[1].wrapping_add(-32768);
            self.sway = REG0_F5 + 0.9;
        }
    }

    fn damage(&mut self, actor: &mut Actor, env: &Env<'_>) {
        match self.sub {
            0 => {
                self.speed.y = 150.0;
                self.speed_f = 30.0;
                self.sub = 1;
                self.sway = 1.0;
            }
            1 => {
                add_calc2(&mut self.float_pos, REG0_F9 + 350.0, 1.0, REG0_F10 + 50.0);
                if self.check_ground(actor) {
                    self.speed.y = 50.0;
                    self.sub = 2;
                    actor.angle[1] = self.rnd_f(RND_ANGLE) as i16;
                    self.sway = 1.0;
                }
            }
            2 => {
                if self.check_ground(actor) {
                    self.mode = 0;
                    self.sub = 0;
                }
            }
            _ => {}
        }
        self.pos_move(actor);
        actor.angle[0] = tilt_of(actor.pos - self.old_pos);
        self.check_ground(actor);
        if self.check_wall(actor, env) && self.timers[1] == 0 {
            actor.angle[1] = actor.angle[1].wrapping_add(-32768);
            self.timers[1] = WALL_LOCK;
        }
        self.body_at = false;
        self.head_at = false;
    }

    fn end(&mut self, actor: &mut Actor, out: &mut Sink<'_>) {
        match self.sub {
            0 => {
                self.speed.y = 150.0;
                self.speed_f = 0.0;
                self.sub = 1;
                *out.bgn_cs_mode = BGN_CS_MODE_DEATH;
                self.sway = 1.0;
                out.se(SE_CV_BK_SURPRISE);
            }
            1 | 2 | 3 | 4 => {
                self.float_pos = actor.pos.y - 150.0;
                if self.float_pos > 350.0 {
                    self.float_pos = 350.0;
                }
                if self.check_ground(actor) {
                    self.sub += 1;
                    actor.angle[1] = self.rnd_f(RND_ANGLE) as i16;
                    self.sway = 1.0;
                    self.speed.y = REG0_F11 + 130.0;
                    self.speed_f = 0.0;
                    actor.angle[1] = self.rnd_f(RND_ANGLE) as i16;
                    out.se(SE_CV_BGN_HIT_1);
                }
                if self.sub == 4 && self.speed.y <= -30.0 {
                    self.sub = 5;
                    self.turn = 0;
                }
            }
            5 => {
                self.speed.y += REG0_F12 + 5.5;
                if self.speed.y > 0.0 {
                    add_calc2(&mut self.glow, 1.0, 1.0, 0.015);
                }
                actor.angle[1] = actor.angle[1].wrapping_add(self.turn);
                self.shape_angle[1] = self.shape_angle[1].wrapping_add(self.turn);
                add_calc_angle_s2(&mut self.turn, END_SPIN_TARGET, 4, END_SPIN_STEP);
                add_calc2(&mut self.reach, REG0_F6 + 50.0, 1.0, 1.0);
                add_calc0(&mut self.float_pos, 1.0, REG0_F10 + 50.0);
            }
            _ => {}
        }
        self.pos_move(actor);
        actor.angle[0] = tilt_of(actor.pos - self.old_pos);
        self.check_ground(actor);
        self.body_at = false;
        self.head_at = false;
        add_calc2(&mut actor.pos.x, -15.0, 0.05, 30.0);
        add_calc2(&mut actor.pos.z, 375.17, 0.05, 30.0);
    }

    fn damage_check(&mut self, env: &Env<'_>, out: &mut Sink<'_>) {
        if self.hit_lock != 0 {
            return;
        }
        let at = env.blow.at;
        if at == At::Idle {
            return;
        }
        let reach = at.reach();
        let shot = match env.blow.pos {
            Some(shot) => shot,
            None => env.player_pos,
        };
        if at == At::LightArrow {
            let core = self.part(WEAK_POINT);
            if reaches(core.pos, core.scale * PART_RADIUS, shot, reach) {
                self.hit_lock = HIT_LOCK;
                self.mode = 2;
                self.sub = 0;
                if self.health != 0 {
                    self.health -= 1;
                    out.se(SE_LK_ARROW_HIT);
                    if self.health == 0 {
                        out.bgm_stop();
                        self.mode = 3;
                        self.sub = 0;
                        self.hit_lock = HIT_LOCK_FINAL;
                        out.se(SE_CV_BGN_HIT_LAST);
                        out.se(SE_CM_BGN_M_BRK_ORB);
                    } else {
                        out.se(SE_CV_BGN_HIT_1);
                    }
                }
            }
            return;
        }
        if self.head_at && reaches(self.eye, HEAD_RADIUS, shot, reach) {
            out.se(SE_CV_BGN_M_HIT_FACE);
            out.se(SE_CM_BGN_M_FACE_HIT);
            let (bvar10, pause, wobble, stun) = match at {
                At::Hammer => (6u8, 5, 0x1e, HIO_M30),
                At::Bomb => (5, 0, 0x19, HIO_M2E),
                At::SwordFinish => (4, 3, 0x14, HIO_M2C),
                At::Sword => (3, 2, 0xf, HIO_M2A),
                At::Arrow => (2, 0, 10, HIO_M28),
                _ => (1, 0, 7, HIO_M26),
            };
            if bvar10 != 0 {
                if self.mode != 1 {
                    self.timers[0] = stun;
                    self.mode = 1;
                }
                self.sub = 0;
                self.hit_lock = HIT_LOCK;
                self.wobble = wobble;
                if pause != 0 {
                    self.next_pause = pause;
                }
                out.se(at.def_se());
                return;
            }
        }
        let mut cvar5 = 0;
        if self.body_at && reaches(self.root, BODY_RADIUS, shot, reach) {
            cvar5 = 1;
        }
        for i in 0..BODY_PARTS {
            let part = self.part(i);
            if part.at && reaches(part.pos, part.scale * PART_RADIUS, shot, reach) {
                cvar5 = 2;
            }
        }
        if cvar5 != 0 && self.body_hit_se == 0 {
            self.body_hit_se = BODY_HIT_SE_LOCK;
            out.se(at.def_se());
        }
    }

    fn move_se_set(&self, actor: &Actor, out: &mut Sink<'_>) {
        let mut local_28 = actor.pos - self.old_pos;
        local_28.y = 0.0;
        let mut uvar2 = (length_of(local_28) * 3.5) as u32;
        if uvar2 > 100 {
            uvar2 = 100;
        }
        let _ = uvar2;
        out.se(SE_CM_BGN_MECHA_ROTATE);
        let mut local_28 = actor.pos - self.old_pos;
        local_28.z = 0.0;
        local_28.x = 0.0;
        let mut uvar2 = (length_of(local_28) * 2.0) as u32;
        if uvar2 > 100 {
            uvar2 = 100;
        }
        let _ = uvar2;
        out.se(SE_CM_BGN_MECHA_ROPE);
    }

    fn move_step(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        let mut bvar5 = false;
        if HIO_M05 == 0 {
            match self.mode {
                10 => {}
                0 => {
                    self.move0(actor, env, out);
                    if self.speed_f > 10.0 {
                        bvar5 = true;
                        out.se(SE_CM_BGN_M_CRAWL);
                        self.move_se_set(actor, out);
                    }
                }
                1 => self.mahi(actor, env),
                2 => self.damage(actor, env),
                3 => self.end(actor, out),
                _ => {}
            }
        }
        self.tail_eff_set(actor, env, out);
        self.damage_check(env, out);
        add_calc_angle_s2(&mut self.shape_angle[1], actor.angle[1], 4, 0x1000);
        add_calc_angle_s2(&mut self.shape_angle[0], actor.angle[0], 8, 0x400);
        let mtx = self.shape_mtx();
        self.root = actor.pos + mtx.pos(Vec3::new(0.0, 0.0, REG0_F3 + ROOT_BACK));
        for i in 0..PART_COUNT {
            self.parts[i].scale = (REG0_F11 + 1.7) * SIZE_D[i];
        }
        self.part_control();
        self.body_centre = actor.pos;
        if self.wobble != 0 {
            self.wobble -= 1;
        }
        let dvar22 = f32::from(self.wobble) * (REG0_F14 + 500.0);
        let y = (dvar22 * sin_s((self.frame as i32 * 0x2100) as i16)) as i16;
        let x = (dvar22 * cos_s((self.frame as i32 * 0x2300) as i16)) as i16;
        let mut head = Mat::none();
        head.trans(actor.pos, false);
        head.yrot_m(self.shape_angle[1]);
        head.xrot_m(self.shape_angle[0]);
        head.zrot_m(self.shape_angle[2]);
        head.trans(Vec3::new(0.0, 0.0, NECK_FWD), true);
        head.yrot_m(y);
        head.xrot_m(x);
        self.eye = head.pos(Vec3::ZERO);
        self.attention = self.eye;
        self.attention.y += REG0_F7 + ATTN_Y;
        for i in 0..PART_COUNT {
            self.parts[i].at = bvar5;
            if i == WEAK_POINT {
                self.weak_point = self.parts[i].pos;
            }
        }
        self.head_at = bvar5;
        self.body_at = bvar5;
    }

    pub fn execute(&mut self, actor: &mut Actor, env: &Env<'_>, out: &mut Sink<'_>) {
        let Some(bgn_mode) = env.bgn_mode else {
            return;
        };
        if HIO_M06 != 0 {
            self.health = HIO_M06 as i32;
        }
        if bgn_mode != BGN_MODE_FIGHT {
            self.mode = 10;
            actor.pos = Vec3::new(0.0, PARKED_Y, 0.0);
            self.attn_flags = 0;
            return;
        }
        if self.mode == 10 {
            actor.pos = self.home;
            self.mode = 0;
            self.turn_state = TURN_HOME;
            self.timers[3] = TURN_HOME;
        }
        self.attn_flags = ATTN_LOCKON_BATTLE;
        add_calc2(&mut self.glow, 0.0, 1.0, 0.01);
        self.frame = self.frame.wrapping_add(1);
        self.attn_dist = ATTN_DIST_BATTLE;
        for slot in self.timers.iter_mut() {
            if *slot != 0 {
                *slot -= 1;
            }
        }
        if self.hit_lock != 0 {
            self.hit_lock -= 1;
        }
        if self.body_hit_se != 0 {
            self.body_hit_se -= 1;
        }
        if self.ripple != 0 {
            self.ripple -= 1;
        }
        self.old_pos = actor.pos;
        self.move_step(actor, env, out);
        if self.keeth_timer != 0 {
            self.keeth_timer -= 1;
            if env.keeths < KEETH_MAX {
                out.keeth.push(KeethSpawn {
                    pos: Vec3::new(
                        self.rnd_fx(2500.0),
                        self.rnd_f(500.0) + 3500.0,
                        self.rnd_fx(2500.0),
                    ),
                    room: actor.room,
                });
            }
        }
    }
}

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    water: Option<Res<'w, Water>>,
    combat: Res<'w, Combat>,
    equipped: Res<'w, inventory::Equipped>,
    players: Query<'w, 's, &'static Player>,
    shafts: Query<'w, 's, &'static arrow::Arrow>,
    others: Query<'w, 's, &'static Actor, Without<GanonForm3>>,
}

fn blow_of(watch: &Watch<'_, '_>) -> Blow {
    if let Some(shaft) = watch
        .shafts
        .iter()
        .find(|shaft| shaft.phase == arrow::Phase::Flying)
    {
        let at = if shaft.kind == arrow::TYPE_LIGHT {
            At::LightArrow
        } else {
            At::Arrow
        };
        return Blow {
            at,
            pos: Some(shaft.pos),
        };
    }
    if !watch.combat.active {
        return Blow {
            at: At::Idle,
            pos: None,
        };
    }
    let at = if watch.equipped.item(inventory::HAMMER_SLOT) == inventory::ITEM_SKULL_HAMMER {
        At::Hammer
    } else {
        match watch.combat.kind {
            CutKind::EA | CutKind::EB => At::SwordFinish,
            _ => At::Sword,
        }
    };
    Blow { at, pos: None }
}

pub fn control(
    watch: Watch<'_, '_>,
    mut events: ResMut<SimEvents>,
    mut commands: Commands,
    mut worms: Query<(Entity, &mut Actor, &mut GanonForm3), (With<GanonForm3>, Without<Player>)>,
) {
    let solid = watch.collision.as_deref();
    let water = watch.water.as_deref();
    let blow = blow_of(&watch);
    let mut bgn_mode: Option<i8> = None;
    let mut esa = None;
    let mut keeths = 0i32;
    for other in watch.others.iter() {
        if super::puppet_ganon::is_puppet_ganon(&other.name) {
            bgn_mode = Some(BGN_MODE_FIGHT);
        } else if super::esa::is_esa(&other.name) {
            esa = Some(other.pos);
        } else if KEETH_ACTORS.contains(&other.name.as_str()) {
            keeths += 1;
        }
    }
    let Some(player) = watch.players.iter().next() else {
        return;
    };
    let mut bgn_cs_mode = 0i8;
    let mut spawns: Vec<KeethSpawn> = Vec::new();
    for (_, mut actor, mut worm) in &mut worms {
        let actor = &mut *actor;
        let worm = &mut *worm;
        if !worm.entered {
            worm.entered = true;
            worm.home = actor.pos;
        }
        let env = Env {
            player_pos: player.pos,
            solid,
            water,
            blow,
            esa,
            keeths,
            bgn_mode,
        };
        let mut sink = Sink {
            events: &mut events,
            bgn_cs_mode: &mut bgn_cs_mode,
            keeth: &mut spawns,
        };
        worm.execute(actor, &env, &mut sink);
    }
    for spawn in spawns {
        commands.spawn((
            Actor {
                name: KEETH_SPAWN_NAME.to_owned(),
                parameters: KEETH_SPAWN_PARAMETERS,
                pos: spawn.pos,
                angle: [0; 3],
                set_id: 0,
                room: spawn.room,
                ..Default::default()
            },
            Keeth3,
        ));
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Keeth3;