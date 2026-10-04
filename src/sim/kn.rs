use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::ground;
use super::player::Player;
use super::stream;
use super::walk;

pub const KN_ACTORS: [&str; 1] = ["kani"];

pub const MODE_ENTER: u8 = 0;
pub const MODE_IDLE: u8 = 1;
pub const MODE_WALK: u8 = 2;
pub const MODE_WANDER: u8 = 3;
pub const MODE_TURN: u8 = 4;
pub const MODE_HIDE: u8 = 5;
pub const MODE_SINK: u8 = 6;
pub const MODE_RISE: u8 = 7;
pub const MODE_FLEE: u8 = 8;
pub const MODE_RUSH: u8 = 9;

pub const SLOT_STEP: usize = 0;
pub const SLOT_TURN: usize = 1;
pub const SLOT_WALK: usize = 2;
pub const SLOT_HOLD: usize = 3;
pub const SLOTS: usize = 4;

pub const IDLE_SCALE: f32 = 1.0;
pub const IDLE_MAX: f32 = 2.0;

pub const SPLIT: f32 = 0.5;
pub const WALK_RATE: f32 = 3.0;
pub const WANDER_SPREAD: f32 = 10752.0;
pub const WANDER_SPAN: f32 = 40.0;
pub const WANDER_FLOOR: f32 = 20.0;
pub const LEASH: f32 = 200.0;
pub const ALERT_RANGE: f32 = 300.0;
pub const FLEE_RANGE: f32 = 500.0;
pub const FLEE_SPEED: f32 = 40.0;

pub const TIMER_TURN: i16 = 0x14;
pub const TIMER_TURN_RETRY: i16 = 8;
pub const TIMER_SINK: i16 = 0x14;
pub const TIMER_WALK: i16 = 0x28;

pub const TURN_SCALE: i16 = 1;
pub const TURN_STEP_WALK: i16 = 0x800;
pub const TURN_STEP_FLEE: i16 = 0x100;
pub const FLEE_TURN: i16 = 0x4000;
pub const HALF_TURN: i32 = -0x8000;

pub const SPEED_SCALE: f32 = 0.3;
pub const SPEED_MAX: f32 = 1.0;

pub const BURY_SHALLOW: f32 = -7.0;
pub const BURY_DEEP: f32 = -11.0;
pub const BURY_EPSILON: f32 = 1.0;
pub const BURY_SCALE: f32 = 0.1;
pub const BURY_MAX: f32 = 1.0;
pub const RISE_SCALE: f32 = 0.1;
pub const RISE_MAX: f32 = 1.0;
pub const RISE_DONE: f32 = 0.2;
pub const RISE_SPAN: f32 = 30.0;
pub const RISE_FLOOR: f32 = 30.0;

pub const GRAVITY: f32 = -5.0;
pub const MAX_FALL: f32 = -20.0;

pub const SCALE_SPLIT: f32 = 1.5;
pub const SCALE_SMALL: f32 = 1.5;
pub const SCALE_LARGE: f32 = 2.5;

pub const CULL_HALF: f32 = 30.0;
pub const CULL_TOP: f32 = 60.0;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = f32::from(seed[0]) / RND_SCALE[0]
        + f32::from(seed[1]) / RND_SCALE[1]
        + f32::from(seed[2]) / RND_SCALE[2];
    (sum % 1.0).abs()
}

fn rnd_f(seed: &mut [i32; 3], range: f32) -> f32 {
    rnd(seed) * range
}

fn rnd_fx(seed: &mut [i32; 3], range: f32) -> f32 {
    range * (rnd(seed) - 0.5) * 2.0
}

fn add_calc2(value: &mut f32, target: f32, scale: f32, max_step: f32) {
    if *value != target {
        let step = (scale * (target - *value)).clamp(-max_step, max_step);
        *value += step;
    }
}

fn add_calc0(value: &mut f32, scale: f32, max_step: f32) {
    let step = (*value * scale).clamp(-max_step, max_step);
    *value -= step;
}

fn angle_gap(a: i16, b: i16) -> i32 {
    i32::from(a.wrapping_sub(b).unsigned_abs())
}

fn spread(base: i16, value: f32) -> i16 {
    ((base as f32 + value) as i32) as i16
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Kn {
    pub mode: u8,
    pub scale: f32,
    pub seed: [i32; 3],
    pub home: Vec3,
    pub slots: [i16; SLOTS],
    pub goal: i16,
    pub turn_step: i16,
    pub turn_sign: f32,
    pub speed_f: f32,
    pub speed_target: f32,
    pub speed_y: f32,
    pub bury: f32,
    pub sunk: bool,
}

impl Kn {
    pub fn spawn(_parameters: u32, home: Vec3) -> Self {
        let mut seed = [RND_SEED; 3];
        let scale = if rnd(&mut seed) < SPLIT {
            SCALE_LARGE
        } else {
            SCALE_SMALL
        };
        Self {
            mode: MODE_ENTER,
            scale,
            seed,
            home,
            slots: [0; SLOTS],
            goal: 0,
            turn_step: 0,
            turn_sign: 0.0,
            speed_f: 0.0,
            speed_target: 0.0,
            speed_y: 0.0,
            bury: 0.0,
            sunk: false,
        }
    }

    pub fn cull(&self) -> (f32, f32) {
        (CULL_HALF, CULL_TOP)
    }

    pub fn drawn(&self) -> bool {
        self.bury >= 0.0
    }

    fn step(&mut self, actor: &mut Actor, player: Vec3) {
        let home_gap = self.home - actor.pos;
        let player_gap = player - actor.pos;
        let home_x = home_gap.x;
        let home_z = home_gap.z;
        let leash = (home_x * home_x + home_z * home_z).sqrt();
        let player_x = player_gap.x;
        let player_z = player_gap.z;
        let reach = player_gap.length();
        loop {
            match self.mode {
                MODE_ENTER => {
                    self.turn_step = TURN_STEP_WALK;
                    self.mode = MODE_IDLE;
                }
                MODE_IDLE => {
                    add_calc0(&mut self.speed_f, IDLE_SCALE, IDLE_MAX);
                    if self.slots[SLOT_WALK] == 0 && reach < ALERT_RANGE {
                        self.mode = MODE_FLEE;
                    }
                    break;
                }
                MODE_WALK => {
                    let swing = rnd_fx(&mut self.seed, WANDER_SPREAD);
                    self.goal = spread(actor.angle[1], swing);
                    self.turn_sign = if rnd(&mut self.seed) < SPLIT {
                        -WALK_RATE
                    } else {
                        WALK_RATE
                    };
                    if self.slots[SLOT_WALK] != 0 {
                        self.goal = walk::to_s16(home_x.atan2(home_z));
                        self.turn_sign = WALK_RATE;
                    }
                    self.slots[SLOT_STEP] =
                        (rnd_f(&mut self.seed, WANDER_SPAN) + WANDER_FLOOR) as i16;
                    self.speed_target = self.turn_sign;
                    self.turn_step = TURN_STEP_WALK;
                    self.mode = MODE_WANDER;
                }
                MODE_WANDER => {
                    if reach < ALERT_RANGE {
                        self.mode = MODE_FLEE;
                    } else if self.slots[SLOT_WALK] == 0 {
                        if leash > LEASH {
                            self.slots[SLOT_TURN] = TIMER_TURN;
                            self.speed_target = -self.turn_sign;
                            self.mode = MODE_TURN;
                        } else if self.slots[SLOT_STEP] == 0 {
                            self.mode = MODE_ENTER;
                        }
                    }
                    break;
                }
                MODE_TURN => {
                    if self.slots[SLOT_TURN] == 0 {
                        if leash > LEASH {
                            self.goal = walk::to_s16(home_x.atan2(home_z));
                            self.speed_target = self.turn_sign.abs();
                            self.slots[SLOT_TURN] = TIMER_TURN_RETRY;
                        } else {
                            self.mode = MODE_ENTER;
                        }
                    }
                    break;
                }
                MODE_HIDE => {
                    self.speed_f = 0.0;
                    self.speed_target = 0.0;
                    self.sunk = false;
                    self.slots[SLOT_STEP] = TIMER_SINK;
                    self.mode = MODE_SINK;
                }
                MODE_SINK => {
                    if self.slots[SLOT_STEP] == 0 {
                        let depth = if self.scale > SCALE_SPLIT {
                            BURY_DEEP
                        } else {
                            BURY_SHALLOW
                        };
                        if !self.sunk {
                            if (self.bury - depth).abs() <= BURY_EPSILON {
                                self.bury = depth;
                                self.sunk = true;
                                self.slots[SLOT_HOLD] =
                                    (rnd_f(&mut self.seed, RISE_SPAN) + RISE_FLOOR) as i16;
                            } else {
                                add_calc2(&mut self.bury, depth, BURY_SCALE, BURY_MAX);
                            }
                        }
                        if self.sunk && self.slots[SLOT_HOLD] == 0 && reach > FLEE_RANGE {
                            self.mode = MODE_RISE;
                        }
                    }
                    break;
                }
                MODE_RISE => {
                    add_calc0(&mut self.bury, RISE_SCALE, RISE_MAX);
                    if self.bury < RISE_DONE {
                        self.bury = 0.0;
                        self.slots[SLOT_WALK] = TIMER_WALK;
                        self.mode = MODE_WALK;
                    }
                    break;
                }
                MODE_FLEE => {
                    self.turn_sign = FLEE_SPEED;
                    self.slots[SLOT_STEP] = 0;
                    self.slots[SLOT_TURN] = 0;
                    self.slots[SLOT_WALK] = 0;
                    self.turn_step = TURN_STEP_FLEE;
                    self.speed_target = self.turn_sign;
                    let facing = (walk::to_s16(player_x.atan2(player_z)) as i32 - HALF_TURN) as i16;
                    let gap = angle_gap(facing, actor.angle[1]);
                    if gap < i32::from(FLEE_TURN) {
                        self.goal = facing;
                    } else {
                        self.goal = actor.angle[1];
                        if gap > (gap - HALF_TURN) as i16 as i32 {
                            self.speed_target = -self.turn_sign;
                        }
                    }
                    self.mode = MODE_RUSH;
                }
                MODE_RUSH => {
                    if leash > LEASH {
                        self.speed_f = 0.0;
                        self.speed_target = 0.0;
                        self.speed_y = 0.0;
                        self.mode = MODE_HIDE;
                    }
                    break;
                }
                _ => break,
            }
        }
        if self.mode >= MODE_WANDER && self.mode < MODE_FLEE {
            stream::add_calc_angle_s2(
                &mut actor.angle[1],
                self.goal,
                TURN_SCALE,
                self.turn_step,
            );
            add_calc2(&mut self.speed_f, self.speed_target, SPEED_SCALE, SPEED_MAX);
        }
    }

    fn integrate(&mut self, actor: &mut Actor) {
        let (speed, _) = stream::calc_speed(
            self.speed_f,
            actor.angle[1],
            self.speed_y,
            GRAVITY,
            MAX_FALL,
        );
        self.speed_y = speed.y;
        actor.pos += speed;
    }
}

pub fn is_kn(name: &str) -> bool {
    KN_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    players: Query<&Player>,
    mut crabs: Query<(&mut Actor, &mut Kn)>,
) {
    let solid = collision.as_deref();
    let Some(player) = players.iter().next() else {
        return;
    };
    let target = player.pos;
    for (mut actor, mut crab) in &mut crabs {
        let actor = &mut *actor;
        let crab = &mut *crab;
        for slot in crab.slots.iter_mut() {
            if *slot != 0 {
                *slot -= 1;
            }
        }
        crab.step(actor, target);
        crab.integrate(actor);
        if let Some(floor) = ground::surface(solid, actor.pos)
            && actor.pos.y < floor
        {
            actor.pos.y = floor;
        }
    }
}