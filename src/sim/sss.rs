use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::{Mat4, Vec3, Vec4};

use super::actor::{Actor, atan2s};
use super::arrow;
use super::bombflower;
use super::boomerang;
use super::collision::Collision;
use super::combat::Combat;
use super::damage_resolve;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::ground;
use super::hammer;
use super::obj_doguu::ParticleSets;
use super::player::Player;
use super::player_status::PlayerStatus;
use super::stream::{add_calc_angle_s2, cos_s, sin_s};
use super::walk::from_s16;

pub const SSS_ACTORS: [&str; 1] = ["Sss"];

pub fn is_sss(name: &str) -> bool {
    SSS_ACTORS.contains(&name)
}

const BYTE_MASK: u32 = 0xFF;
const VARIANT_SHIFT: u32 = 8;
const SWITCH_SHIFT: u32 = 16;

const KIND_NONE: u8 = 0xFF;
const KIND_ONE: u8 = 1;
const KIND_ALT: u8 = 0x23;
const VARIANT_NONE: u8 = 0xFF;

const STATE_IDLE: i16 = 0;
const STATE_RISE: i16 = 1;
const STATE_REACH: i16 = 2;
const STATE_GRAB: i16 = 3;
const STATE_CRASH: i16 = 5;
const STATE_LAND: i16 = 6;

const TIMER_OPEN: usize = 0;
const TIMER_EMIT: usize = 1;
const TIMER_HIT: usize = 2;

const HEALTH: i32 = 2;
const VARIANT_SCALE: f32 = 10.0;
const VARIANT_FAR: f32 = 1000.0;

const REG0_F0: f32 = 0.0;
const REG0_F1: f32 = 0.0;
const REG0_F2: f32 = 0.0;
const REG0_F11: f32 = 0.0;
const REG0_F13: f32 = 0.0;
const REG0_F14: f32 = 0.0;
const REG0_F17: f32 = 0.0;
const REG0_F18: f32 = 0.0;
const REG0_S0: i16 = 0;
const REG0_S5: i32 = 0;
const REG0_S6: i32 = 0;
const REG0_S7: i32 = 0;
const REG0_S8: i32 = 0;
const REG6_F0: f32 = 0.0;
const REG6_F1: f32 = 0.0;
const REG6_F2: f32 = 0.0;
const REG6_F6: f32 = 0.0;
const REG6_F7: f32 = 0.0;
const REG6_F8: f32 = 0.0;
const REG6_F9: f32 = 0.0;

const SE_OBJ_SVINE_OUT: u32 = 0x58F7;
const SE_OBJ_SVINE_OUT_WATER: u32 = 0x58F8;
const SE_OBJ_SVINE_GRASP: u32 = 0x58F9;
const SE_OBJ_SVINE_CRASH: u32 = 0x69B0;
const SE_LK_LAST_HIT: u32 = 0x2828;

const PARTICLE_TSURU_TAIEKI: u32 = 0x8184;
const PARTICLE_SIBOUFLASH: u32 = 0x0016;

const CULL_MIN: [f32; 3] = [-200.0, -200.0, -200.0];
const CULL_MAX: [f32; 3] = [200.0, 600.0, 200.0];

const AT_TYPE_SWORD: u32 = 1 << 1;
const AT_TYPE_BOMB: u32 = 1 << 5;
const AT_TYPE_BOOMERANG: u32 = 1 << 6;
const AT_TYPE_WATER: u32 = 1 << 8;
const AT_TYPE_NORMAL_ARROW: u32 = 1 << 14;
const AT_TYPE_SKULL_HAMMER: u32 = 1 << 16;
const AT_TYPE_UNK20000: u32 = 1 << 17;
const AT_TYPE_FIRE_ARROW: u32 = 1 << 18;
const AT_TYPE_ICE_ARROW: u32 = 1 << 19;
const AT_TYPE_LIGHT_ARROW: u32 = 1 << 20;
const AT_TYPE_WIND: u32 = 1 << 21;
const AT_TYPE_UNK400000: u32 = 1 << 22;
const AT_TYPE_LIGHT: u32 = 1 << 23;

const TG_TYPE_SSS: u32 = !(AT_TYPE_BOOMERANG
    | AT_TYPE_WATER
    | AT_TYPE_UNK20000
    | AT_TYPE_WIND
    | AT_TYPE_UNK400000
    | AT_TYPE_LIGHT);
const TG_TYPE_BM: u32 = AT_TYPE_BOOMERANG | AT_TYPE_WIND;

const TG_RADIUS: f32 = 20.0;
const BM_RADIUS: f32 = 80.0;

const NAME_PLAYER: &str = "Link";
const NAME_BOOMERANG: &str = "Boom";
const NAME_BOMB: &str = "Bomb";
const NAME_ARROW: &str = "Arrow";

const BODY_LEN: usize = 10;
const CUT_LEN: usize = 5;
const SPH_COUNT: usize = 3;

const SPAWN_LIFT: f32 = 230.0;
const RISE_HEIGHT: f32 = 250.0;
const RISE_SWAY: f32 = 50.0;
const RISE_WAKE: f32 = 300.0;
const RISE_LEAVE: f32 = 100.0;
const REACH_GIVE_UP: f32 = 450.0;
const GRAB_LIFT: f32 = 70.0;
const GRAB_NEAR: f32 = 20.0;
const GRAB_DROP: f32 = 800.0;
const IDLE_STICK: f32 = 5.0;

const SEG_MAX: f32 = 30.0;
const SEG_STEP: f32 = 0.5;
const SEG_STEP_MAX: f32 = 0.5;
const SEG_WAVE_TARGET: f32 = 10.0;
const SEG_WAVE_STEP: f32 = 0.2;
const WOBBLE_TARGET: f32 = 5.0;
const WOBBLE_STEP: f32 = 1.5;
const WOBBLE_GRAB: f32 = 10.0;
const WOBBLE_CRASH: f32 = 50.0;
const KNOCK_STEP: f32 = 5.0;
const KNOCK_SPEED: f32 = 30.0;
const KNOCK_HEIGHT: f32 = 100.0;

const PULL_BASE: f32 = 400.0;
const PULL_SPEED: f32 = 0.1;
const PULL_CAP: f32 = 100.0;

const GROUND_PROBE: f32 = 200.0;
const GROUND_CLEAR: f32 = 10.0;
const GROUND_FLOOR: f32 = 5.0;
const CRASH_FALL: f32 = 3.0;
const LAND_SINK: f32 = 2.0;
const GRND_INF: f32 = 1.0e9;

const SEG_SHORTEN: f32 = 10.0;
const CUT_SPAN: f32 = 1.5;
const CUT_FADE: f32 = 1.0;

const OPEN_TIMER: i16 = 30;
const LAND_TIMER: i16 = 100;
const EMIT_TIMER: i16 = 60;
const HIT_COOLDOWN: i16 = 20;
const HIT_BUSY: i16 = 5;
const LAND_BUSY: i16 = 10;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

pub const SIZE_D: [f32; 10] = [10.0, 10.0, 9.5, 9.0, 8.5, 8.0, 7.5, 7.0, 6.5, 6.5];
const G_D: [f32; 10] = [50.0, 50.0, 35.0, 25.0, 15.0, 9.0, 6.0, 6.0, 6.0, 6.0];

const BCK_HIRAKU: i16 = 0;
const BCK_TOJIRU: i16 = 1;

#[derive(Resource, Debug, Clone, Copy)]
pub struct NonPos {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for NonPos {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 30000.0,
            z: -20000.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Node {
    pub pos: Vec3,
    pub size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sph {
    pub centre: Vec3,
    pub radius: f32,
    pub at_type: u32,
    pub at_atp: u8,
    pub at_sprm: u32,
    pub tg_type: u32,
    pub tg_sprm: u32,
    pub co_sprm: u32,
}

impl Sph {
    const TG_SRC: Sph = Sph {
        centre: Vec3::ZERO,
        radius: TG_RADIUS,
        at_type: 0,
        at_atp: 0,
        at_sprm: 0x05,
        tg_type: TG_TYPE_SSS,
        tg_sprm: 0x03,
        co_sprm: 0x71,
    };

    const BM_SRC: Sph = Sph {
        centre: Vec3::ZERO,
        radius: BM_RADIUS,
        at_type: 0,
        at_atp: 0,
        at_sprm: 0x05,
        tg_type: TG_TYPE_BM,
        tg_sprm: 0x03,
        co_sprm: 0,
    };

    fn on_co_sprm_bit(&mut self, bit: u32) {
        self.co_sprm |= bit;
    }

    fn off_co_sprm_bit(&mut self, bit: u32) {
        self.co_sprm &= !bit;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    pub pos: Vec3,
    pub reach: f32,
    pub at_type: u32,
    pub name: &'static str,
    pub at_atp: u8,
}

impl Hit {
    fn crosses(&self, centre: Vec3, radius: f32, mask: u32) -> bool {
        if self.at_type & mask == 0 {
            return false;
        }
        (centre - self.pos).length() <= radius + self.reach
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Sss {
    pub kind: u8,
    pub variant: u8,
    pub switch_id: u8,
    pub timer: i16,
    pub state: i16,
    pub timer3: [i16; 3],
    pub target: Vec3,
    pub hand_pos: Vec3,
    pub hand_pitch: i16,
    pub hand_yaw: i16,
    pub seg_len: f32,
    pub seg_wave: f32,
    pub wobble: f32,
    pub ground_y: f32,
    pub cut_len: f32,
    pub body: [Node; BODY_LEN],
    pub cut: [Node; CUT_LEN],
    pub knock: f32,
    pub knock_angle: i16,
    pub emitter1: bool,
    pub emitter2: bool,
    pub trail: bool,
    pub eye: Vec3,
    pub home: Vec3,
    pub anm: i16,
    pub sph: [Sph; SPH_COUNT],
    pub bm: Sph,
    pub health: i32,
    pub max_health: i32,
    pub entered: bool,
    pub seed: [i32; 3],
}

impl Sss {
    pub fn spawn(parameters: u32) -> Self {
        let mut kind = (parameters & BYTE_MASK) as u8;
        if kind == KIND_NONE {
            kind = 0;
        }
        if kind != KIND_ONE {
            kind = KIND_ALT;
        }
        let mut switch_id = ((parameters >> SWITCH_SHIFT) & BYTE_MASK) as u8;
        if switch_id == KIND_NONE {
            switch_id = 0;
        }
        Self {
            kind,
            variant: ((parameters >> VARIANT_SHIFT) & BYTE_MASK) as u8,
            switch_id,
            timer: 0,
            state: STATE_IDLE,
            timer3: [0; 3],
            target: Vec3::ZERO,
            hand_pos: Vec3::ZERO,
            hand_pitch: 0,
            hand_yaw: 0,
            seg_len: 0.0,
            seg_wave: 0.0,
            wobble: 0.0,
            ground_y: 0.0,
            cut_len: 0.0,
            body: [Node::default(); BODY_LEN],
            cut: [Node::default(); CUT_LEN],
            knock: 0.0,
            knock_angle: 0,
            emitter1: false,
            emitter2: false,
            trail: false,
            eye: Vec3::ZERO,
            home: Vec3::ZERO,
            anm: 0,
            sph: [Sph::TG_SRC; SPH_COUNT],
            bm: Sph::BM_SRC,
            health: HEALTH,
            max_health: HEALTH,
            entered: false,
            seed: [RND_SEED; 3],
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        self.kind != 0
    }

    fn rnd(&mut self) -> f32 {
        for lane in 0..3 {
            self.seed[lane] = (self.seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
        }
        let sum = self.seed[0] as f32 / RND_SCALE[0]
            + self.seed[1] as f32 / RND_SCALE[1]
            + self.seed[2] as f32 / RND_SCALE[2];
        (sum % 1.0).abs()
    }

    fn rnd_f(&mut self, range: f32) -> f32 {
        self.rnd() * range
    }

    fn rnd_fx(&mut self, range: f32) -> f32 {
        range * (self.rnd() - 0.5) * 2.0
    }

    fn hand_open(&mut self) {
        self.anm = BCK_HIRAKU;
    }

    fn hand_close(&mut self) {
        self.anm = BCK_TOJIRU;
    }

    fn create(&mut self, actor: &Actor) {
        self.timer = ang(self.rnd_f(10000.0));
        self.state = STATE_IDLE;
        self.timer3 = [0; 3];
        self.home = actor.pos;
        self.target = actor.pos;
        if self.switch_id == 0 {
            self.target.y += SPAWN_LIFT;
        }
        for slot in self.sph.iter_mut() {
            *slot = Sph::TG_SRC;
        }
        self.bm = Sph::BM_SRC;
        self.health = HEALTH;
        self.max_health = HEALTH;
    }

    fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.timer = self.timer.wrapping_add(1);
        for i in 0..2 {
            if self.timer3[i] != 0 {
                self.timer3[i] -= 1;
            }
        }
        if self.timer3[TIMER_HIT] != 0 {
            self.timer3[TIMER_HIT] -= 1;
        }
        if self.kind == 0 {
            self.hand_main();
        } else {
            self.hand_move(actor, ctx);
        }
    }

    fn hand_main(&mut self) {}

    fn hand_move(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        let mut mv = Move::new();
        let f4 = (actor.pos - ctx.player.pos).length();
        mv.f5 = if self.variant != VARIANT_NONE {
            f32::from(self.variant) * VARIANT_SCALE
        } else {
            VARIANT_FAR
        };
        let base = yrot(actor.angle[1]) * xrot(actor.angle[0]);
        match self.state {
            STATE_IDLE => {
                mv.sp74 = actor.pos;
                mv.target = 0.0;
                mv.max_speed = 0.5;
                mv.f24 = -20.0;
                let mut b: i8 = 0;
                if self.timer3[TIMER_OPEN] == 0 {
                    if self.switch_id != 0 {
                        if ctx.switch_on(self.switch_id, actor.room) || REG0_S0 != 0 {
                            self.state = STATE_RISE;
                            self.timer3[TIMER_OPEN] = OPEN_TIMER;
                            self.hand_open();
                            b = 1;
                        }
                    } else if f4 < mv.f5 {
                        self.state = STATE_RISE;
                        self.timer3[TIMER_OPEN] = OPEN_TIMER;
                        self.hand_open();
                        b = 1;
                    }
                }
                if (self.target.y - actor.pos.y).abs() < IDLE_STICK && b != 0 {
                    if self.kind == 1 {
                        ctx.se(SE_OBJ_SVINE_OUT_WATER);
                    } else {
                        ctx.se(SE_OBJ_SVINE_OUT);
                    }
                }
            }
            STATE_RISE => {
                let wave = Vec3::new(
                    (REG0_F13 + RISE_SWAY) * ssin(i32::from(self.timer) * 600),
                    RISE_HEIGHT,
                    (REG0_F13 + RISE_SWAY) * ssin(i32::from(self.timer) * 700),
                );
                mv.sp74 = actor.pos + point(&base, wave);
                if self.timer3[TIMER_OPEN] == 0 && f4 < REG0_F14 + RISE_WAKE {
                    self.state = STATE_REACH;
                }
                if f4 > mv.f5 + RISE_LEAVE {
                    self.state = STATE_IDLE;
                }
            }
            STATE_REACH => {
                mv.f6 = 15.0;
                mv.f27 = SEG_WAVE_TARGET;
                mv.f28 = 0.5;
                mv.b1 = true;
                mv.sp74 = ctx.player.pos + Vec3::new(0.0, GRAB_LIFT, 0.0);
                if f4 > REG0_F14 + REACH_GIVE_UP {
                    self.state = STATE_RISE;
                    actor.speed_f = 0.0;
                }
                if (mv.sp74 - self.target).length() < GRAB_NEAR {
                    self.state = STATE_GRAB;
                    self.hand_close();
                    ctx.se(SE_OBJ_SVINE_GRASP);
                    self.hold(actor, ctx, f4, &mut mv);
                }
            }
            STATE_GRAB => self.hold(actor, ctx, f4, &mut mv),
            STATE_CRASH => {
                mv.f1 = WOBBLE_CRASH;
                self.wobble = WOBBLE_CRASH;
                mv.b2 = true;
                mv.target = 25.0;
                mv.max_speed = 1.0;
                self.target += actor.speed;
                actor.speed.y -= CRASH_FALL;
                self.timer3[TIMER_HIT] = HIT_BUSY;
                let probe = self.target + Vec3::new(0.0, GROUND_PROBE, 0.0);
                self.ground_y = ground::surface(ctx.solid, probe).unwrap_or(-GRND_INF);
                if self.ground_y == -GRND_INF || self.target.y <= self.ground_y + GROUND_CLEAR {
                    self.target.y = self.ground_y + GROUND_CLEAR;
                    self.timer3[TIMER_OPEN] = LAND_TIMER;
                    self.state = STATE_LAND;
                }
            }
            STATE_LAND => {
                mv.b2 = true;
                mv.f1 = 0.0;
                self.timer3[TIMER_HIT] = LAND_BUSY;
                if self.timer3[TIMER_OPEN] < 40 {
                    mv.target = mv.f1;
                    mv.max_speed = 1.0;
                    self.target.y -= LAND_SINK;
                }
                if self.timer3[TIMER_OPEN] == 0 {
                    self.target = actor.pos;
                    self.state = STATE_IDLE;
                    self.seg_len = 0.0;
                    self.cut_len = 0.0;
                }
            }
            _ => {}
        }

        add_calc2(&mut self.seg_len, mv.target, SEG_STEP, mv.max_speed);
        add_calc2(&mut self.seg_wave, mv.f27, 1.0, SEG_WAVE_STEP);
        add_calc2(&mut self.wobble, mv.f1, 1.0, WOBBLE_STEP);

        if !mv.b2 {
            add_calc2(&mut actor.speed_f, mv.f6, 1.0, mv.f28);
            if self.knock > 1.0 && self.state != STATE_GRAB {
                let shove = yrot(self.knock_angle);
                mv.sp74 =
                    actor.pos + point(&shove, Vec3::new(0.0, REG6_F9 + KNOCK_HEIGHT, self.knock));
                mv.f30 = 0.1;
                actor.speed_f = self.knock * 0.2;
                if actor.speed_f > REG6_F8 + KNOCK_SPEED {
                    actor.speed_f = REG6_F8 + KNOCK_SPEED;
                }
            }
            add_calc0(&mut self.knock, 1.0, REG6_F7 + KNOCK_STEP);
            add_calc2(&mut self.target.x, mv.sp74.x, mv.f30, actor.speed_f);
            add_calc2(&mut self.target.y, mv.sp74.y, mv.f30, actor.speed_f);
            add_calc2(&mut self.target.z, mv.sp74.z, mv.f30, actor.speed_f);
            add_calc2(&mut actor.pos.y, ctx.home.y + mv.f24, 0.5, 0.5);
            if mv.b1 && actor.angle[0] == 0 {
                let want = angle_to(actor.pos, ctx.player.pos);
                add_calc_angle_s2(&mut actor.angle[1], want, 16, 2048);
            }
            self.control1(actor.pos, &base);
            self.control2();
        } else {
            self.cut_control1(actor.pos, &base);
            self.cut_control2(ctx);
            add_calc0(&mut self.cut_len, 1.0, REG0_F1 + CUT_FADE);
        }
        self.control3();

        self.eye = self.body[5].pos;
        actor.eye = self.eye;
        let away = ctx.off_screen();
        self.bm.centre = if mv.b2 { away } else { self.eye };
        for i in 0..SPH_COUNT {
            let mut r4 = (i32::from(self.timer) & 3) + i as i32 * 2;
            r4 %= BODY_LEN as i32;
            let index = usize::try_from(r4).unwrap_or(0);
            let at = self
                .body
                .get(index)
                .map(|node| node.pos)
                .unwrap_or_default();
            self.sph[i].centre = if mv.b2 { away } else { at };
            if self.state == STATE_GRAB {
                self.sph[i].off_co_sprm_bit(1);
            } else {
                self.sph[i].on_co_sprm_bit(1);
            }
        }

        let mut u1: usize = 0;
        for i in 0..SPH_COUNT {
            let sphere = self.sph[i];
            if ctx.tg_hit(sphere.centre, sphere.radius, sphere.tg_type) {
                u1 = i + 1;
                break;
            }
        }
        let bm_hit = ctx.tg_hit(self.bm.centre, self.bm.radius, self.bm.tg_type);
        if (u1 != 0 || bm_hit) && self.timer3[TIMER_HIT] == 0 {
            self.timer3[TIMER_HIT] = HIT_COOLDOWN;
            if u1 == 0 {
                let strike = ctx.tg_strike(self.bm.centre, self.bm.radius, self.bm.tg_type);
                let mut at_info = damage_resolve::CcAtInfo {
                    p_particle_pos: Some(self.eye),
                    ..Default::default()
                };
                let cut = damage_resolve::PlayerCut {
                    cut_type: 0,
                    cut_count: 0,
                    pos: ctx.player.pos,
                };
                if let Some(strike) = strike {
                    at_info.mp_obj = Some(damage_resolve::Atk {
                        at_type: damage_resolve::AtkType(strike.at_type),
                        at_se: damage_resolve::AT_SE_NONE,
                        at_spl: damage_resolve::AT_SPL_UNK0,
                        at_atp: strike.at_atp,
                        ac: Some(damage_resolve::Attacker {
                            name: strike.name.to_owned(),
                            pos: strike.pos,
                            speed: Vec3::ZERO,
                            bomb_state: 0,
                            tsubo_type: 0,
                        }),
                    });
                }
                damage_resolve::at_power_check(&mut at_info, &cut);
                if at_info.m_resulting_attack_type == damage_resolve::RESULT_WIND {
                    self.knock = REG6_F6 + 300.0;
                    self.knock_angle = angle_to(actor.pos, ctx.player.pos).wrapping_add(-0x8000);
                    return;
                }
            }
            ctx.se(SE_LK_LAST_HIT);
            ctx.se(SE_OBJ_SVINE_CRASH);
            self.state = STATE_CRASH;
            actor.speed = Vec3::new(
                self.rnd_fx(10.0),
                self.rnd_f(10.0) + 30.0,
                self.rnd_fx(10.0),
            );
            ctx.particles
                .live
                .push((PARTICLE_SIBOUFLASH, self.eye, [0; 3]));
            self.trail = true;
            for i in 0..CUT_LEN {
                self.cut[i].pos = self.body[i].pos;
                self.cut[i].size = self.body[i].size;
                if i == CUT_LEN - 1 {
                    let temp = self.cut[CUT_LEN - 1].pos - self.cut[CUT_LEN - 2].pos;
                    self.cut_len = (REG0_F2 + CUT_SPAN) * temp.length();
                }
            }
            self.hand_open();
        }
    }

    fn hold(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>, f4: f32, mv: &mut Move) {
        ctx.vine_catch();
        let temp = REG6_F2 + PULL_BASE;
        if f4 > temp {
            let mut power = (REG6_F0 + PULL_SPEED) * (f4 - temp);
            if power > REG6_F1 + PULL_CAP {
                power = REG6_F1 + PULL_CAP;
            }
            let want = angle_to(actor.pos, ctx.player.pos).wrapping_add(-0x8000);
            ctx.throw_player(power, want);
        }
        mv.sp74 = ctx.player.pos + Vec3::new(0.0, GRAB_LIFT, 0.0);
        mv.f6 = 200.0;
        mv.f30 = 1.0;
        mv.f28 = 5.0;
        self.wobble = REG0_F11 + WOBBLE_GRAB;
        if f4 > GRAB_DROP {
            self.state = STATE_RISE;
            actor.speed_f = 30.0;
            self.hand_open();
        }
    }

    fn control1(&mut self, root: Vec3, base: &Mat4) {
        self.body[0].pos = root;
        let sp24 = point(base, Vec3::new(0.0, 0.0, self.seg_wave));
        let sp3c = Vec3::new(0.0, 0.0, self.seg_len);
        let f27 = self.wobble;
        for i in 1..BODY_LEN - 1 {
            let sp30 = Vec3::new(
                f27 * ssin(i32::from(self.timer) * (REG0_S5 + 1100) + i as i32 * (REG0_S6 + 4000)),
                G_D.get(i).copied().unwrap_or(0.0),
                f27 * scos(i32::from(self.timer) * (REG0_S7 + 800) + i as i32 * (REG0_S8 + 4000)),
            );
            let sp18 = point(base, sp30);
            let prev = self.body[i - 1].pos;
            let me = self.body[i].pos;
            let f29 = sp24.x + (me.x - prev.x) + sp18.x;
            let f28 = sp24.y + (me.y - prev.y) + sp18.y;
            let f26 = sp24.z + (me.z - prev.z) + sp18.z;
            let xz_atan = atan2s(f29, f26);
            let y_atan = ang(-atan2s_f(f28, (f29 * f29 + f26 * f26).sqrt()));
            let link = yrot(xz_atan) * xrot(y_atan);
            self.body[i].pos = prev + point(&link, sp3c);
        }
    }

    fn control2(&mut self) {
        let mut sp34 = Vec3::new(0.0, 0.0, self.seg_len);
        self.body[9].pos = self.target;
        for i in (1..BODY_LEN - 1).rev() {
            let me = self.body[i].pos;
            let nxt = self.body[i + 1].pos;
            let f29 = me.x - nxt.x;
            let f30 = me.y - nxt.y;
            let f28 = me.z - nxt.z;
            let r27 = atan2s(f29, f28);
            let r28 = ang(-atan2s_f(f30, (f29 * f29 + f28 * f28).sqrt()));
            let link = yrot(r27) * xrot(r28);
            if i == BODY_LEN - 2 {
                sp34.z = self.seg_len - SEG_SHORTEN;
                if sp34.z < 0.0 {
                    sp34.z = 0.0;
                }
            } else {
                sp34.z = self.seg_len;
            }
            self.body[i].pos = nxt + point(&link, sp34);
        }
        self.hand_pos = self.body[9].pos;
        let link = self.body[8].pos - self.body[9].pos;
        self.hand_pitch = ang(-atan2s_f(link.y, link.z));
        self.hand_yaw = ang(atan2s_f(link.x, (link.y * link.y + link.z * link.z).sqrt()));
    }

    fn control3(&mut self) {
        for i in 0..BODY_LEN {
            let temp = ssin(i32::from(self.timer) * 500 + i as i32 * 100) * 0.1 + 0.8;
            self.body[i].size = temp * SIZE_D.get(i).copied().unwrap_or(0.0);
        }
    }

    fn cut_control1(&mut self, root: Vec3, base: &Mat4) {
        self.cut[0].pos = root;
        let sp50 = Vec3::new(0.0, 0.0, self.cut_len);
        let reg = REG0_F18 + RISE_SWAY;
        for i in 1..CUT_LEN {
            let sp44 = Vec3::new(
                reg * ssin(i32::from(self.timer) * (REG0_S5 + 4100) + i as i32 * (REG0_S6 + 10000)),
                REG0_F17 + RISE_SWAY,
                reg * scos(i32::from(self.timer) * (REG0_S7 + 4400) + i as i32 * (REG0_S8 + 10000)),
            );
            let sp38 = point(base, sp44);
            let prev = self.cut[i - 1].pos;
            let me = self.cut[i].pos;
            let x_val = sp38.x + (me.x - prev.x);
            let y_val = sp38.y + (me.y - prev.y);
            let z_val = sp38.z + (me.z - prev.z);
            let xz_atan = atan2s(x_val, z_val);
            let y_atan = ang(-atan2s_f(y_val, (x_val * x_val + z_val * z_val).sqrt()));
            let link = yrot(xz_atan) * xrot(y_atan);
            self.cut[i].pos = prev + point(&link, sp50);
        }
    }

    fn cut_control2(&mut self, ctx: &mut Ctx<'_>) {
        let mut sp60 = Vec3::new(0.0, 0.0, self.seg_len);
        self.body[9].pos = self.target;
        let f27 = self.wobble;
        for i in (0..BODY_LEN).rev() {
            let sp48 = Vec3::new(
                f27 * ssin(i32::from(self.timer) * (REG0_S5 + 2500) + i as i32 * (REG0_S6 + 3000)),
                f27 * ssin(i32::from(self.timer) * (REG0_S5 + 2950) + i as i32 * (REG0_S6 + 4000)),
                f27 * scos(i32::from(self.timer) * (REG0_S7 + 2800) + i as i32 * (REG0_S8 + 3500)),
            );
            let me = self.body[i].pos;
            let nxt = self.body[i + 1].pos;
            let x_val = sp48.x + (me.x - nxt.x);
            let mut y_val = me.y - SEG_SHORTEN + sp48.y;
            let f0 = self.ground_y + GROUND_FLOOR;
            if y_val < f0 {
                y_val = f0;
            }
            let f28 = y_val - nxt.y;
            let z_val = sp48.z + (me.z - nxt.z);
            let xz_atan = atan2s(x_val, z_val);
            let y_atan = ang(-atan2s_f(f28, (x_val * x_val + z_val * z_val).sqrt()));
            let link = yrot(xz_atan) * xrot(y_atan);
            if i == BODY_LEN - 2 {
                sp60.z = self.seg_len - SEG_SHORTEN;
                if sp60.z < 0.0 {
                    sp60.z = 0.0;
                }
            } else {
                sp60.z = self.seg_len;
            }
            self.body[i].pos = nxt + point(&link, sp60);
        }
        self.hand_pos = self.body[9].pos;
        let link = self.body[8].pos - self.body[9].pos;
        self.hand_pitch = ang(-atan2s_f(link.y, link.z));
        self.hand_yaw = ang(atan2s_f(link.x, (link.y * link.y + link.z * link.z).sqrt()));

        if !self.emitter1 && self.trail {
            self.emitter1 = true;
            self.timer3[TIMER_EMIT] = EMIT_TIMER;
        }
        if self.emitter1 {
            let sp50 = self.cut[CUT_LEN - 1].pos - self.cut[CUT_LEN - 2].pos;
            let rot_y = atan2s(sp50.x, sp50.z);
            let rot_x = ang(-atan2s_f(
                sp50.y,
                (sp50.x * sp50.x + sp50.z * sp50.z).sqrt(),
            ));
            ctx.particles.live.push((
                PARTICLE_TSURU_TAIEKI,
                self.cut[CUT_LEN - 1].pos,
                [rot_x, rot_y, 0],
            ));
            if self.timer3[TIMER_EMIT] == 1 {
                self.emitter1 = false;
            }
        }
        if !self.emitter2 && self.trail {
            self.emitter2 = true;
        }
        if self.emitter2 {
            let sp60 = self.body[0].pos - self.body[1].pos;
            let rot_y = atan2s(sp60.x, sp60.z);
            let rot_x = ang(-atan2s_f(
                sp60.y,
                (sp60.x * sp60.x + sp60.z * sp60.z).sqrt(),
            ));
            ctx.particles
                .live
                .push((PARTICLE_TSURU_TAIEKI, self.body[0].pos, [rot_x, rot_y, 0]));
            if self.timer3[TIMER_EMIT] == 1 {
                self.emitter2 = false;
                self.trail = false;
            }
        }
    }
}

fn ang(value: f32) -> i16 {
    value as i32 as i16
}

fn ssin(units: i32) -> f32 {
    sin_s(units as i16)
}

fn scos(units: i32) -> f32 {
    cos_s(units as i16)
}

fn atan2s_f(y: f32, x: f32) -> f32 {
    (y.atan2(x) * (32768.0 / core::f32::consts::PI)).round()
}

fn angle_to(from: Vec3, to: Vec3) -> i16 {
    atan2s(to.x - from.x, to.z - from.z)
}

fn xrot(angle: i16) -> Mat4 {
    let a = from_s16(angle);
    Mat4::from_cols(
        Vec4::X,
        Vec4::new(0.0, a.cos(), a.sin(), 0.0),
        Vec4::new(0.0, -a.sin(), a.cos(), 0.0),
        Vec4::W,
    )
}

fn yrot(angle: i16) -> Mat4 {
    let a = from_s16(angle);
    Mat4::from_cols(
        Vec4::new(a.cos(), 0.0, -a.sin(), 0.0),
        Vec4::Y,
        Vec4::new(a.sin(), 0.0, a.cos(), 0.0),
        Vec4::W,
    )
}

fn point(m: &Mat4, v: Vec3) -> Vec3 {
    (*m * Vec4::new(v.x, v.y, v.z, 1.0)).truncate()
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

#[derive(Debug, Clone, Copy)]
struct Move {
    sp74: Vec3,
    f30: f32,
    f6: f32,
    f28: f32,
    f27: f32,
    target: f32,
    max_speed: f32,
    f24: f32,
    f1: f32,
    f5: f32,
    b1: bool,
    b2: bool,
}

impl Move {
    fn new() -> Self {
        Self {
            sp74: Vec3::ZERO,
            f30: 0.1,
            f6: 8.0,
            f28: 0.1,
            f27: 0.0,
            target: SEG_MAX,
            max_speed: 1.0,
            f24: 0.0,
            f1: WOBBLE_TARGET,
            f5: VARIANT_FAR,
            b1: false,
            b2: false,
        }
    }
}

struct Ctx<'a> {
    player: &'a mut Player,
    solid: Option<&'a Collision>,
    hits: &'a [Hit],
    switches: &'a Switches,
    home: Vec3,
    events: &'a mut SimEvents,
    particles: &'a mut ParticleSets,
    status: &'a mut PlayerStatus,
    non_pos: NonPos,
}

impl Ctx<'_> {
    fn se(&mut self, id: u32) {
        self.events.list.push(SimEvent::Se(id));
    }

    fn off_screen(&self) -> Vec3 {
        Vec3::new(self.non_pos.x, self.non_pos.y, self.non_pos.z)
    }

    fn switch_on(&self, no: u8, _room: i8) -> bool {
        if u32::from(no) < 128 {
            self.switches.test(no)
        } else {
            self.switches.test((no - 128) | 0x80)
        }
    }

    fn tg_hit(&self, centre: Vec3, radius: f32, mask: u32) -> bool {
        self.hits
            .iter()
            .any(|hit| hit.crosses(centre, radius, mask))
    }

    fn tg_strike(&self, centre: Vec3, radius: f32, mask: u32) -> Option<Hit> {
        self.hits
            .iter()
            .find(|hit| hit.crosses(centre, radius, mask))
            .copied()
    }

    fn vine_catch(&mut self) {
        self.status.status1 |= STTS1_VINE_CATCH;
    }

    fn throw_player(&mut self, power: f32, angle: i16) {
        self.player.speed = power;
        self.player.yaw = from_s16(angle);
        self.player.face = self.player.yaw;
    }
}

const STTS1_VINE_CATCH: u32 = 0x0200_0000;

#[derive(SystemParam)]
pub struct Watch<'w, 's> {
    collision: Option<Res<'w, Collision>>,
    switches: Res<'w, Switches>,
    combat: Res<'w, Combat>,
    hammer: Res<'w, hammer::Cut>,
    blasts: Res<'w, bombflower::Blasts>,
    players: Query<'w, 's, (Entity, &'static mut Player)>,
    shafts: Query<'w, 's, &'static arrow::Arrow>,
    bolts: Query<'w, 's, &'static boomerang::Boomerang>,
}

pub fn control(
    watch: Watch<'_, '_>,
    mut events: ResMut<SimEvents>,
    mut particles: ResMut<ParticleSets>,
    mut status: ResMut<PlayerStatus>,
    mut non_pos: ResMut<NonPos>,
    mut bosses: Query<(Entity, &mut Actor, &mut Sss), (With<Sss>, Without<Player>)>,
) {
    let Watch {
        collision,
        switches,
        combat,
        hammer: hammer_cut,
        blasts,
        players: player_query,
        shafts,
        bolts,
    } = watch;
    let solid = collision.as_deref();
    *non_pos = NonPos::default();
    let origin = player_query
        .iter()
        .next()
        .map(|(_, player)| player.pos)
        .unwrap_or_default();
    let hits = collect_hits(&combat, &hammer_cut, &blasts, &shafts, &bolts, origin);
    let mut players = player_query;
    let Some((_, mut player)) = players.iter_mut().next() else {
        return;
    };
    let player = &mut *player;
    for (entity, mut actor, mut sss) in &mut bosses {
        let _ = entity;
        if !sss.entered {
            sss.entered = true;
            sss.create(&actor);
        }
        let home = sss.home;
        let mut ctx = Ctx {
            player,
            solid,
            hits: &hits,
            switches: &switches,
            home,
            events: &mut events,
            particles: &mut particles,
            status: &mut status,
            non_pos: *non_pos,
        };
        sss.execute(&mut actor, &mut ctx);
    }
}

fn collect_hits(
    combat: &Combat,
    hammer_cut: &hammer::Cut,
    blasts: &bombflower::Blasts,
    shafts: &Query<'_, '_, &'static arrow::Arrow>,
    bolts: &Query<'_, '_, &'static boomerang::Boomerang>,
    player_pos: Vec3,
) -> Vec<Hit> {
    let mut hits: Vec<Hit> = Vec::new();
    if combat.active && (1..6).contains(&combat.frame) {
        hits.push(Hit {
            pos: player_pos,
            reach: 40.0,
            at_type: AT_TYPE_SWORD,
            name: NAME_PLAYER,
            at_atp: 2,
        });
    }
    if matches!(
        hammer_cut.kind,
        hammer::CUT_HAMMER_SIDESWING | hammer::CUT_HAMMER_FRONTSWING
    ) {
        hits.push(Hit {
            pos: player_pos,
            reach: 50.0,
            at_type: AT_TYPE_SKULL_HAMMER,
            name: NAME_PLAYER,
            at_atp: 2,
        });
    }
    for blast in blasts.list.iter() {
        hits.push(Hit {
            pos: *blast,
            reach: 30.0,
            at_type: AT_TYPE_BOMB,
            name: NAME_BOMB,
            at_atp: 4,
        });
    }
    for shaft in shafts.iter() {
        if shaft.phase != arrow::Phase::Flying {
            continue;
        }
        let at_type = match shaft.kind {
            arrow::TYPE_FIRE => AT_TYPE_FIRE_ARROW,
            arrow::TYPE_ICE => AT_TYPE_ICE_ARROW,
            arrow::TYPE_LIGHT => AT_TYPE_LIGHT_ARROW,
            _ => AT_TYPE_NORMAL_ARROW,
        };
        hits.push(Hit {
            pos: shaft.pos,
            reach: 35.0,
            at_type,
            name: NAME_ARROW,
            at_atp: 2,
        });
    }
    for bolt in bolts.iter() {
        if bolt.parameter != boomerang::MODE_THROW {
            continue;
        }
        hits.push(Hit {
            pos: bolt.pos,
            reach: 90.0,
            at_type: AT_TYPE_BOOMERANG,
            name: NAME_BOOMERANG,
            at_atp: 1,
        });
    }
    hits
}
