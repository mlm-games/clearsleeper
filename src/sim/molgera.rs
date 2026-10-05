use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::flags::ZoneActors;
use super::player::Player;
use super::stream::{add_calc_angle_s2, cos_s, sin_s};
use super::walk::{from_s16, to_s16};

const MOLGERA_ACTOR: &str = "Bwd";

const HIO_HOVER: f32 = -400.0;
const HIO_PUSH: f32 = 6.0;
const HIO_NEAR: f32 = 700.0;
const HIO_FAR: f32 = 3000.0;
const HIO_GROUND: f32 = 800.0;

const ONE: f32 = 1.0;
const CREATE_Y: f32 = -20000.0;
const CLAMP_STEPS: i32 = 64;

const START_RANGE: f32 = 3200.0;
const START_PUSH: i16 = 600;
const START_LEAVE: i16 = 0x294;
const START_DROP: f32 = -2200.0;
const START_PILLAR: i16 = 0x28;

const WAIT_FACE: i16 = -10;
const WAIT_RISE: i16 = 0x7a;
const WAIT_HOLD: i16 = 0xde;
const WAIT_TRACK: f32 = 80.0;
const WAIT_TURN_SCALE: i16 = 10;
const WAIT_TURN_MAX: i16 = 0x800;
const WAIT_PLAYER_Y: f32 = 900.0;
const WAIT_HOVER_SCALE: f32 = 0.1;
const WAIT_HOVER_STEP: f32 = 20.0;

const RESET_HOLD: i16 = 0x96;
const RESET_DROP: f32 = -1500.0;
const RESET_HOVER_SCALE: f32 = 1.0;
const RESET_HOVER_STEP: f32 = 10.0;
const RESET_SPREAD: f32 = 300.0;
const RESET_CLAMP: f32 = 3000.0;
const RESET_SHRINK: f32 = 0.9;

const EAT_TEETH: f32 = 27.0;
const EAT_GAP: f32 = 150.0;
const EAT_HOLD: i16 = 0x96;
const EAT_BRIEF: i16 = 100;
const EAT_MOUTH: i32 = 10;
const EAT_SAND: i32 = 30;

const FLY_SE_FRAME: i32 = 0x14;
const FLY_MOUTH_FRAME: i32 = 2;
const FLY_PITCH: i16 = -0x4000;
const FLY_CEIL: f32 = 2000.0;
const FLY_CEIL_SPREAD: f32 = 800.0;
const FLY_LOOP_HOLD: f32 = 200.0;
const FLY_LOOP_SPREAD: f32 = 50.0;
const FLY_STRAFE: f32 = 1500.0;
const FLY_TURN: f32 = 500.0;
const FLY_SPEED: f32 = 40.0;
const FLY_LOOP: i16 = 100;
const FLY_HOLD: i16 = 0xb4;
const FLY_GRAB: i16 = 0x96;
const FLY_ARRIVE: f32 = 500.0;
const FLY_ARRIVE_WIDE: f32 = 1000.0;
const FLY_PLUNGE: f32 = 10000.0;
const FLY_DIVE_SPREAD: f32 = 1500.0;
const FLY_DIVE_Y: f32 = -1500.0;
const FLY_SINK: f32 = 800.0;
const FLY_SINK_STEP: f32 = 10.0;
const FLY_BORE: i16 = 200;
const FLY_BORE_TURN: f32 = 1200.0;
const FLY_BORE_STEP: f32 = 20.0;
const FLY_EYE: f32 = 40.0;

const SFLY_CEIL: f32 = 3000.0;
const SFLY_STOOP: f32 = 800.0;
const SFLY_LAST: i8 = 3;
const SFLY_RISE: i16 = 0xac;
const SFLY_WAIT: i16 = 0x12e;
const SFLY_TARGET_X: [f32; 3] = [-1500.0, 1500.0, 0.0];
const SFLY_TARGET_Y: [f32; 3] = [3500.0, 3000.0, 4000.0];
const SFLY_TARGET_Z: [f32; 3] = [-1500.0, 1500.0, 0.0];

const MOVE_TURN_SCALE: i16 = 10;
const MOVE_ROLL_SCALE: i16 = 0x20;
const MOVE_ROLL_CLAMP: i16 = 0x157c;
const MOVE_ROLL_HALF: f32 = 0.5;
const MOVE_RAMP_STEP: f32 = 0.04;

const PUSH_RATE: f32 = 0.00025;
const PUSH_LIMIT: f32 = 900.0;

const END_HIDE_Y: f32 = -20000.0;
const END_RISE_Y: f32 = -2000.0;
const END_CEIL: f32 = 1500.0;
const END_CEIL_SPREAD: f32 = 1600.0;
const END_CEIL_SPREAD2: f32 = 500.0;
const END_LOOP: f32 = 20.0;
const END_WAIT: i16 = 0xb4;
const END_TURN: f32 = 1000.0;
const END_SPEED: f32 = 65.0;
const END_HOLD_SUB: i16 = 0x17;
const END_HOLD: i16 = 0x3c;
const END_HARDEN: i16 = 0x5a;
const END_SPARK: i16 = 0x14;
const END_PERIOD_MASK: i32 = 3;
const END_SEGMENTS: i8 = 0x14;
const END_SEGMENT_LAST: i8 = 0x13;
const END_WOBBLE_A: i16 = 0xc1c;
const END_WOBBLE_B: i16 = 0xdac;
const END_WOBBLE_SPAN: f32 = 7000.0;
const END_SEGMENT_DELAY: i16 = 10;

const MODE_START: i16 = 0;
const MODE_WAIT: i16 = 1;
const MODE_RESET: i16 = 2;
const MODE_EAT: i16 = 5;
const MODE_FLY: i16 = 10;
const MODE_S_FLY: i16 = 11;
const MODE_END: i16 = 12;

const SE_EXPLODE: u32 = 0x5801;
const SE_FLYING: u32 = 0x48DB;
const SE_GO_UNDER: u32 = 0x48DC;
const SE_ATTACK: u32 = 0x48E0;
const SE_MOUTH_OPEN: u32 = 0x5911;
const SE_IN_SAND_2: u32 = 0x5914;
const SE_MOUTH_CLOSE: u32 = 0x5915;
const SE_HEAD_BACK: u32 = 0x5916;
const SE_LAST_HEAD_BACK: u32 = 0x5918;
const SE_HARDEN: u32 = 0x591A;
const SE_LAST_EXPLODE: u32 = 0x591B;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const TIMER_SLOTS: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clip {
    Wait,
    Fly,
    Attack,
    AttackEat,
    AttackShippai,
    Jattack1,
    Jattack2,
    Jattack3,
    Dead1,
    Dead2,
    Dead3,
}

const CLIP_FRAMES: [i16; 11] = [30, 30, 29, 69, 69, 15, 8, 18, 109, 30, 69];

impl Clip {
    fn frames(self) -> i16 {
        CLIP_FRAMES
            .get(self as usize)
            .copied()
            .unwrap_or(CLIP_FRAMES[0])
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Molgera {
    pub mode: i16,
    pub sub: i16,
    pub timer: [i16; TIMER_SLOTS],
    pub clock: i16,
    pub wait: i16,
    pub placed: bool,
    pub finished: bool,
    pub range: f32,
    pub centre: Vec3,
    pub target: Vec3,
    pub turn: f32,
    pub turn_rate: f32,
    pub speed: Vec3,
    pub speed_f: f32,
    pub hop: i8,
    pub wobble_a: i16,
    pub wobble_b: i16,
    pub clip: Clip,
    pub frame: f32,
    pub rate: f32,
    pub looping: bool,
    pub push: bool,
    pub seed: [i32; 3],
}

fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value != target {
        let step = (scale * (target - *value)).clamp(-max_step, max_step);
        *value += step;
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

fn bearing(from: Vec3, to: Vec3) -> i16 {
    to_s16((to.x - from.x).atan2(to.z - from.z))
}

impl Molgera {
    pub fn spawn() -> Self {
        Self {
            mode: MODE_START,
            sub: 0,
            timer: [0; TIMER_SLOTS],
            clock: 0,
            wait: 0,
            placed: false,
            finished: false,
            range: 0.0,
            centre: Vec3::ZERO,
            target: Vec3::ZERO,
            turn: 0.0,
            turn_rate: 0.0,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            hop: 0,
            wobble_a: 0,
            wobble_b: 0,
            clip: Clip::Wait,
            frame: 0.0,
            rate: ONE,
            looping: true,
            push: false,
            seed: [RND_SEED; 3],
        }
    }

    fn place(&mut self, actor: &mut Actor) {
        self.placed = true;
        actor.pos = Vec3::new(0.0, CREATE_Y, 0.0);
        self.clip = Clip::Wait;
        self.frame = 0.0;
        self.rate = ONE;
        self.looping = true;
    }

    fn rnd_f(&mut self, range: f32) -> f32 {
        range * rnd(&mut self.seed)
    }

    fn rnd_fx(&mut self, range: f32) -> f32 {
        range * (rnd(&mut self.seed) - 0.5) * 2.0
    }

    fn se(&self, events: &mut SimEvents, id: u32) {
        events.list.push(SimEvent::Se(id));
    }

    fn set_anm(&mut self, clip: Clip, looping: bool, speed: f32) {
        self.clip = clip;
        self.frame = 0.0;
        self.rate = speed;
        self.looping = looping;
    }

    fn step_anm(&mut self) {
        self.frame += self.rate;
        let end = (self.clip.frames()) as f32;
        if !self.looping && self.frame >= end {
            self.frame = end;
            self.rate = 0.0;
        }
    }

    fn stopped(&self) -> bool {
        self.rate == 0.0
    }

    fn at_frame(&self) -> i32 {
        self.frame as i32
    }

    fn recentre(&mut self, actor: &mut Actor, player: &Player) {
        actor.pos.x = player.pos.x + self.rnd_fx(RESET_SPREAD);
        actor.pos.z = player.pos.z + self.rnd_fx(RESET_SPREAD);
        let mut steps = 0;
        while steps < CLAMP_STEPS
            && (actor.pos.x * actor.pos.x + actor.pos.z * actor.pos.z).sqrt() >= RESET_CLAMP
        {
            actor.pos.x *= RESET_SHRINK;
            actor.pos.z *= RESET_SHRINK;
            steps += 1;
        }
    }

    fn move_state(&mut self, actor: &mut Actor, player: &Player, events: &mut SimEvents) {
        match self.mode {
            MODE_START => self.start(actor, player),
            MODE_WAIT => self.wait(actor, player),
            MODE_RESET => self.reset(actor, player),
            MODE_EAT => self.eat_attack(actor, player, events),
            MODE_FLY => self.fly(actor, player, events),
            MODE_S_FLY => self.s_fly(actor, events),
            MODE_END => self.end(actor, events),
            _ => {}
        }
    }

    fn start(&mut self, actor: &mut Actor, player: &Player) {
        match self.sub {
            0 => {
                if player.pos.length() < START_RANGE {
                    self.sub = 1;
                    self.clock = 0;
                }
            }
            1 => {
                if self.clock >= START_PUSH {
                    self.push = true;
                }
                if self.clock == START_LEAVE {
                    self.mode = MODE_S_FLY;
                    self.sub = 0;
                    actor.pos = Vec3::new(0.0, HIO_GROUND + START_DROP, 0.0);
                    self.timer[1] = START_PILLAR;
                }
            }
            _ => {}
        }
    }

    fn wait(&mut self, actor: &mut Actor, player: &Player) {
        self.push = true;
        self.centre = actor.pos;
        actor.angle[2] = 0;
        actor.angle[0] = 0;
        if self.range > WAIT_TRACK {
            add_calc_angle_s2(
                &mut actor.angle[1],
                bearing(actor.pos, player.pos),
                WAIT_TURN_SCALE,
                WAIT_TURN_MAX,
            );
        }
        if self.sub == WAIT_FACE {
            if !(player.pos.y < WAIT_PLAYER_Y) {
                return;
            }
            self.timer[0] = WAIT_RISE;
            self.wait = WAIT_HOLD;
            self.sub = 1;
            return;
        }
        if self.sub == 0 {
            self.sub = 1;
        }
        if self.sub == 1 {
            if self.timer[0] != 0 {
                return;
            }
            self.set_anm(Clip::Wait, true, ONE);
            self.sub = 2;
        }
        add_calc2(
            &mut actor.pos.y,
            HIO_HOVER + HIO_GROUND,
            WAIT_HOVER_SCALE,
            WAIT_HOVER_STEP,
        );
        if self.wait != 0 {
            return;
        }
        if self.range < HIO_NEAR {
            self.mode = MODE_EAT;
            self.sub = 0;
        } else if self.range > HIO_FAR {
            self.mode = MODE_RESET;
            self.sub = 0;
        }
    }

    fn reset(&mut self, actor: &mut Actor, player: &Player) {
        if self.sub == 0 {
            self.sub = 1;
            self.timer[0] = RESET_HOLD;
        }
        add_calc2(
            &mut actor.pos.y,
            HIO_GROUND + RESET_DROP,
            RESET_HOVER_SCALE,
            RESET_HOVER_STEP,
        );
        if self.timer[0] != 0 {
            return;
        }
        self.mode = MODE_WAIT;
        self.sub = 0;
        self.wait = WAIT_HOLD;
        self.timer[0] = WAIT_RISE;
        self.recentre(actor, player);
    }

    fn eat_attack(&mut self, actor: &mut Actor, player: &Player, events: &mut SimEvents) {
        self.centre = actor.pos;
        if self.range > WAIT_TRACK {
            add_calc_angle_s2(
                &mut actor.angle[1],
                bearing(actor.pos, player.pos),
                WAIT_TURN_SCALE,
                WAIT_TURN_MAX,
            );
        }
        let mut done = 0u8;
        match self.sub {
            0 => {
                self.set_anm(Clip::Attack, false, ONE);
                self.sub = 1;
                self.se(events, SE_ATTACK);
            }
            1 => {
                let landed = self.frame >= EAT_TEETH && self.range < EAT_GAP;
                if landed {
                    self.set_anm(Clip::AttackEat, false, ONE);
                    self.sub = 5;
                    self.timer[0] = EAT_HOLD;
                    self.se(events, SE_MOUTH_CLOSE);
                } else if self.stopped() {
                    self.set_anm(Clip::AttackShippai, false, ONE);
                    self.sub = 2;
                    self.se(events, SE_MOUTH_CLOSE);
                }
            }
            2 => {
                if self.stopped() {
                    done = 1;
                }
            }
            5 => {
                if self.at_frame() == EAT_MOUTH {
                    self.se(events, SE_HEAD_BACK);
                }
                if self.at_frame() == EAT_SAND {
                    self.se(events, SE_IN_SAND_2);
                }
                if self.timer[0] == 0 {
                    done = 2;
                }
            }
            _ => {}
        }
        if done != 0 {
            self.mode = MODE_WAIT;
            self.sub = 0;
            if done == 1 {
                self.wait = EAT_BRIEF;
            } else {
                self.timer[0] = WAIT_RISE;
                self.wait = WAIT_HOLD;
            }
        }
    }

    fn fly_pos_move(&mut self, actor: &mut Actor, pitch_extra: i16, yaw_extra: i16) {
        let delta = self.target - actor.pos;
        let yaw = yaw_extra.wrapping_add(to_s16(delta.x.atan2(delta.z)));
        let flat = (delta.x * delta.x + delta.z * delta.z).sqrt();
        let pitch = pitch_extra.wrapping_sub(to_s16(delta.y.atan2(flat)));
        let previous = actor.angle[1];
        let cap = (self.turn * self.turn_rate) as i16;
        add_calc_angle_s2(&mut actor.angle[1], yaw, MOVE_TURN_SCALE, cap);
        let roll = (i32::from(previous) - i32::from(actor.angle[1])) * i32::from(MOVE_ROLL_SCALE);
        let roll = roll.clamp(-i32::from(MOVE_ROLL_CLAMP), i32::from(MOVE_ROLL_CLAMP)) as i16;
        add_calc_angle_s2(
            &mut actor.angle[2],
            roll,
            MOVE_TURN_SCALE,
            ((cap) as f32 * MOVE_ROLL_HALF) as i16,
        );
        add_calc_angle_s2(&mut actor.angle[0], pitch, MOVE_TURN_SCALE, cap);
        add_calc2(&mut self.turn_rate, ONE, ONE, MOVE_RAMP_STEP);
        let (tilt, level) = from_s16(actor.angle[0]).sin_cos();
        let (yaw_sin, yaw_cos) = from_s16(actor.angle[1]).sin_cos();
        self.speed = Vec3::new(
            self.speed_f * level * yaw_sin,
            -self.speed_f * tilt,
            self.speed_f * level * yaw_cos,
        );
        actor.pos += self.speed;
    }

    fn fly(&mut self, actor: &mut Actor, player: &Player, events: &mut SimEvents) {
        match self.sub {
            0 => {
                if self.timer[1] == 0 {
                    self.set_anm(Clip::Fly, true, ONE);
                    self.sub = 1;
                    let ceil = self.rnd_fx(FLY_CEIL_SPREAD) + FLY_CEIL;
                    self.target = Vec3::new(actor.pos.x, HIO_GROUND + ceil, actor.pos.z);
                    actor.angle[0] = FLY_PITCH;
                    self.timer[1] = (self.rnd_f(FLY_LOOP_SPREAD) + FLY_LOOP_HOLD) as i16;
                    self.wait = FLY_HOLD;
                }
            }
            1 => {
                if self.at_frame() == FLY_MOUTH_FRAME {
                    self.se(events, SE_MOUTH_OPEN);
                }
                if self.at_frame() == FLY_SE_FRAME {
                    self.se(events, SE_FLYING);
                }
                if (self.target - actor.pos).length() < FLY_ARRIVE || self.timer[0] == 0 {
                    self.timer[0] = FLY_LOOP;
                    let ceil = FLY_CEIL + self.rnd_fx(FLY_CEIL_SPREAD);
                    self.target = Vec3::new(
                        self.rnd_fx(FLY_STRAFE),
                        HIO_GROUND + ceil,
                        self.rnd_fx(FLY_STRAFE),
                    );
                    self.turn_rate = 0.0;
                }
                self.turn = FLY_TURN;
                self.speed_f = FLY_SPEED;
                if self.timer[1] == 0 {
                    self.sub = 2;
                    self.timer[0] = FLY_GRAB;
                    self.set_anm(Clip::Jattack1, false, ONE);
                    self.se(events, SE_GO_UNDER);
                }
            }
            2 => {
                if self.stopped() {
                    self.set_anm(Clip::Jattack2, true, ONE);
                }
                self.target = player.pos + Vec3::new(0.0, FLY_EYE, 0.0);
                if (self.target - actor.pos).length() < FLY_ARRIVE_WIDE || self.timer[0] == 0 {
                    self.sub = 3;
                    self.target.y -= FLY_PLUNGE;
                    self.timer[0] = FLY_BORE;
                    self.set_anm(Clip::Jattack3, false, ONE);
                }
                add_calc2(&mut self.turn, FLY_SINK, ONE, FLY_SINK_STEP);
            }
            _ => {
                add_calc2(&mut self.turn, FLY_BORE_TURN, ONE, FLY_BORE_STEP);
                if self.timer[0] == 0 {
                    self.mode = MODE_WAIT;
                    self.sub = 0;
                    self.timer[0] = WAIT_RISE;
                    self.wait = WAIT_HOLD;
                    actor.pos = Vec3::new(
                        self.rnd_fx(FLY_DIVE_SPREAD),
                        HIO_GROUND + FLY_DIVE_Y,
                        self.rnd_fx(FLY_DIVE_SPREAD),
                    );
                }
            }
        }
        self.fly_pos_move(actor, 0, 0);
    }

    fn s_fly(&mut self, actor: &mut Actor, events: &mut SimEvents) {
        match self.sub {
            0 => {
                if self.timer[1] == 0 {
                    self.set_anm(Clip::Fly, true, ONE);
                    self.sub = 1;
                    self.target = Vec3::new(0.0, HIO_GROUND + SFLY_CEIL, 0.0);
                    actor.angle[0] = FLY_PITCH;
                    self.wait = FLY_HOLD;
                }
            }
            1 => {
                if self.at_frame() == FLY_SE_FRAME {
                    self.se(events, SE_FLYING);
                }
                if (self.target - actor.pos).length() < FLY_ARRIVE {
                    if self.hop == SFLY_LAST {
                        self.sub = 2;
                        self.timer[0] = FLY_GRAB;
                        self.set_anm(Clip::Jattack1, false, ONE);
                        self.se(events, SE_GO_UNDER);
                    } else {
                        let index = usize::from(self.hop.max(0));
                        let next = SFLY_TARGET_X
                            .get(index)
                            .zip(SFLY_TARGET_Y.get(index))
                            .zip(SFLY_TARGET_Z.get(index))
                            .map(|((x, y), z)| Vec3::new(*x, *y, *z));
                        if let Some(next) = next {
                            self.target = next;
                        }
                        self.turn_rate = 0.0;
                        self.hop = self.hop.saturating_add(1);
                    }
                }
                self.turn = FLY_TURN;
                self.speed_f = FLY_SPEED;
            }
            2 => {
                if self.stopped() {
                    self.set_anm(Clip::Jattack2, true, ONE);
                }
                self.target = Vec3::new(0.0, SFLY_STOOP, 0.0);
                if (self.target - actor.pos).length() < FLY_ARRIVE_WIDE {
                    self.sub = 3;
                    self.target.y -= FLY_PLUNGE;
                    self.timer[0] = FLY_BORE;
                    self.set_anm(Clip::Jattack3, false, ONE);
                }
                add_calc2(&mut self.turn, FLY_SINK, ONE, FLY_SINK_STEP);
            }
            _ => {
                add_calc2(&mut self.turn, FLY_BORE_TURN, ONE, FLY_BORE_STEP);
                if self.timer[0] == 0 {
                    self.mode = MODE_WAIT;
                    self.sub = 0;
                    self.timer[0] = SFLY_RISE;
                    self.wait = SFLY_WAIT;
                    actor.pos = Vec3::new(0.0, HIO_GROUND + FLY_DIVE_Y, 0.0);
                }
            }
        }
        self.fly_pos_move(actor, 0, 0);
    }

    fn end(&mut self, actor: &mut Actor, events: &mut SimEvents) {
        let mut glide = true;
        let mut pitch_extra = 0i16;
        let mut yaw_extra = 0i16;
        match self.sub {
            0 => {
                self.set_anm(Clip::Dead1, false, ONE);
                self.sub = 1;
                actor.angle[1] = 0;
                self.hop = 0;
                self.se(events, SE_LAST_HEAD_BACK);
                self.dying(actor);
                glide = false;
            }
            1 => {
                self.dying(actor);
                glide = false;
            }
            2 => {
                if (self.target - actor.pos).length() < FLY_ARRIVE || self.timer[0] == 0 {
                    self.timer[0] = self.rnd_f(END_LOOP) as i16;
                    let ceil = FLY_CEIL + self.rnd_fx(END_CEIL_SPREAD2);
                    self.target = Vec3::new(
                        self.rnd_fx(END_CEIL_SPREAD),
                        HIO_GROUND + ceil,
                        self.rnd_fx(END_CEIL_SPREAD),
                    );
                }
                self.turn = END_TURN;
                self.speed_f = END_SPEED;
                if self.timer[1] == 0 {
                    self.sub = END_HOLD_SUB;
                    self.timer[0] = END_HOLD;
                    self.timer[1] = END_HARDEN;
                    self.speed_f = 0.0;
                    self.turn = 0.0;
                    self.se(events, SE_HARDEN);
                }
                self.wobble_a = self.wobble_a.wrapping_add(END_WOBBLE_A);
                self.wobble_b = self.wobble_b.wrapping_add(END_WOBBLE_B);
                pitch_extra = (END_WOBBLE_SPAN * sin_s(self.wobble_a)) as i16;
                yaw_extra = (END_WOBBLE_SPAN * cos_s(self.wobble_b)) as i16;
            }
            END_HOLD_SUB => {
                if self.timer[0] == END_SPARK {
                    self.set_anm(Clip::Dead3, false, ONE);
                }
                if self.timer[1] == 0
                    && (i32::from(self.clock) & END_PERIOD_MASK) == 0
                    && self.hop < END_SEGMENTS
                {
                    if self.hop == END_SEGMENT_LAST {
                        self.sub = 3;
                        self.timer[0] = 0;
                        self.se(events, SE_LAST_EXPLODE);
                    } else {
                        self.hop += 1;
                    }
                }
            }
            3 => {
                if self.timer[0] == 0 {
                    self.sub = 4;
                    self.timer[0] = END_SEGMENT_DELAY;
                    self.se(events, SE_EXPLODE);
                }
            }
            4 => {
                glide = false;
                if self.timer[0] == 0 {
                    actor.pos.y = END_HIDE_Y;
                    self.sub = 5;
                    self.finished = true;
                }
            }
            _ => {
                glide = false;
            }
        }
        if glide {
            self.fly_pos_move(actor, pitch_extra, yaw_extra);
        }
    }

    fn dying(&mut self, actor: &mut Actor) {
        if !self.stopped() {
            return;
        }
        self.set_anm(Clip::Dead2, true, ONE);
        actor.pos.y = END_RISE_Y;
        self.sub = 2;
        self.target = Vec3::new(0.0, HIO_GROUND + END_CEIL, 0.0);
        actor.angle[0] = FLY_PITCH;
        self.timer[1] = END_WAIT;
    }
}

pub fn is_molgera(name: &str) -> bool {
    name == MOLGERA_ACTOR
}

pub fn control(
    mut players: Query<&mut Player>,
    mut molgeras: Query<(Entity, &mut Actor, &mut Molgera), Without<Player>>,
    mut events: ResMut<SimEvents>,
    mut zones: ResMut<ZoneActors>,
    mut commands: Commands,
) {
    let Some(mut player) = players.iter_mut().next() else {
        return;
    };
    let view = *player;
    let mut gone: Vec<(Entity, i8, u16)> = Vec::new();
    for (entity, mut actor, mut mol) in &mut molgeras {
        let actor = &mut *actor;
        let mol = &mut *mol;
        if mol.finished {
            gone.push((entity, actor.room, actor.set_id));
            continue;
        }
        if !mol.placed {
            mol.place(actor);
        }
        mol.clock = mol.clock.wrapping_add(1);
        for slot in 0..TIMER_SLOTS {
            if mol.timer[slot] != 0 {
                mol.timer[slot] -= 1;
            }
        }
        let gap = view.pos - actor.pos;
        mol.range = (gap.x * gap.x + gap.z * gap.z).sqrt();
        mol.move_state(actor, &view, &mut events);
        if mol.push && view.pos.y < PUSH_LIMIT {
            let pull = mol.centre - view.pos;
            let power = HIO_PUSH * (ONE - PUSH_RATE * mol.range).max(0.0);
            let (sin, cos) = from_s16(to_s16(pull.x.atan2(pull.z))).sin_cos();
            player.drift.x = power * sin;
            player.drift.z = power * cos;
            player.pos.x += player.drift.x;
            player.pos.z += player.drift.z;
            mol.push = false;
        }
        mol.step_anm();
    }
    for (entity, room, set_id) in gone {
        zones.mark_done(room, set_id);
        let _ = commands.entity(entity).despawn();
    }
}