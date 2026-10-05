use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::damage;
use super::events::{SimEvent, SimEvents};
use super::ground;
use super::health::Health;
use super::player::Player;
use super::sea::Sea;
use super::stream;
use super::walk;

pub const BOKO_ACTORS: [&str; 1] = ["Boko"];

const TYPE_BOKO_STICK: u8 = 0;
const TYPE_STALFOS_MACE: u8 = 2;
const TYPE_COUNT: u32 = 6;
const TYPE_TORCH: u32 = 7;

const MODE_WAIT: u8 = 0;
const MODE_MOVE: u8 = 1;
const MODE_ENEMY_CARRY: u8 = 2;
const MODE_THROW: u8 = 6;

const PROC_WAIT: u8 = 0;
const PROC_MOVE: u8 = 1;
const PROC_THROW: u8 = 2;
const PROC_CARRY: u8 = 3;

const GRAVITY: f32 = -3.0;
const MAX_FALL: f32 = -100.0;
const BASE_LIFT: f32 = 5.0;

const BOUNCE_LIMIT: f32 = -10.0;
const BOUNCE_GAIN: f32 = -0.5;
const BOUNCE_CAP: f32 = 20.0;
const SLACK_MAX: u8 = 0x14;
const FLAT_RATE: f32 = -0.2;
const FLAT_ANGLE: i16 = 0x5a;
const FLAT_SPEED: f32 = 1.0;
const MOVE_DAMP: f32 = 0.7;
const PITCH_CLAMP: i16 = 0x3a00;
const ROLL_LIMIT: i16 = 0x4000;
const ROLL_FOLD: i16 = -0x8000;
const ROLL_SCALE: i16 = 5;
const ROLL_MAX_STEP: i16 = 0x1000;
const ROLL_MIN_STEP: i16 = 0x100;
const LAND_SPEED: f32 = 2.5;
const SLOPE_LIMIT: i16 = 0x1000;
const PITCH_DIV: i16 = 2;
const HOP_SPEED: f32 = 15.0;
const HOP_LIFT: f32 = 15.0;
const SLACK_SLIDE: u8 = 2;
const SLACK_KICK: u8 = 4;
const HALF_TURN: i16 = -0x8000;
const CARRY_FLIP: i16 = 0x4000;
const BACKSTEP: f32 = 50.0;
const BACKSTEPS: u8 = 3;
const FALLBACK_Y: f32 = 50.0;

const REVERSE_SPIN: i16 = 0x7ff;
const REVERSE_LIFT: f32 = 15.0;
const REVERSE_KEEP: f32 = 0.25;
const REVERSE_SPREAD: f32 = 12288.0;
const REVERSE_TURN: i16 = 0x4000;
const REVERSE_BASE: f32 = 3500.0;
const REVERSE_JITTER: f32 = 500.0;
const THROW_SPREAD: f32 = 16384.0;
const THROW_TIMER: i16 = 2;
const THROW_SPEED: f32 = 70.0;

const FLAME_TIME: i16 = 900;
const SE_TORCH_OFF: u32 = 0x6904;

const STALFOS_TILT: i16 = -1000;
const STALFOS_LIFT: f32 = 10.0;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

const BOUND_SE: [u32; 6] = [0x5811, 0x5814, 0x6999, 0x6998, 0x5807, 0x6998];

const TOP_OFFSET: [Vec3; 6] = [
    Vec3::new(0.0, 0.0, 85.0),
    Vec3::new(16.0, 0.0, 77.0),
    Vec3::new(-8.0, 3.0, 120.0),
    Vec3::new(0.0, 0.0, 128.0),
    Vec3::new(10.0, 0.0, 200.0),
    Vec3::new(-40.0, 0.0, 110.0),
];

const ROOT_OFFSET: [Vec3; 6] = [
    Vec3::new(0.0, 0.0, -85.0),
    Vec3::new(5.0, 0.0, -90.0),
    Vec3::new(3.0, 0.0, -120.0),
    Vec3::new(0.0, 0.0, -130.0),
    Vec3::new(0.0, 0.0, -175.0),
    Vec3::new(0.0, 0.0, -125.0),
];

const CPS_RADIUS: [f32; 6] = [30.0, 30.0, 50.0, 50.0, 50.0, 50.0];

const CULL_MIN: [Vec3; 6] = [
    Vec3::new(-20.0, -10.0, -95.0),
    Vec3::new(-20.0, -10.0, -95.0),
    Vec3::new(-35.0, -35.0, -125.0),
    Vec3::new(-45.0, -10.0, -145.0),
    Vec3::new(-20.0, -20.0, -180.0),
    Vec3::new(-50.0, -20.0, -140.0),
];

const CULL_MAX: [Vec3; 6] = [
    Vec3::new(20.0, 10.0, 95.0),
    Vec3::new(20.0, 10.0, 95.0),
    Vec3::new(35.0, 35.0, 120.0),
    Vec3::new(45.0, 10.0, 130.0),
    Vec3::new(20.0, 20.0, 200.0),
    Vec3::new(50.0, 20.0, 120.0),
];

fn yrot(v: Vec3, units: i16) -> Vec3 {
    let (s, c) = walk::from_s16(units).sin_cos();
    Vec3::new(v.x * c + v.z * s, v.y, v.z * c - v.x * s)
}

fn xrot(v: Vec3, units: i16) -> Vec3 {
    let (s, c) = walk::from_s16(units).sin_cos();
    Vec3::new(v.x, v.y * c - v.z * s, v.y * s + v.z * c)
}

fn zrot(v: Vec3, units: i16) -> Vec3 {
    let (s, c) = walk::from_s16(units).sin_cos();
    Vec3::new(v.x * c - v.y * s, v.x * s + v.y * c, v.z)
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

fn rnd_fx(seed: &mut [i32; 3], max: f32) -> f32 {
    max * (rnd(seed) - 0.5) * 2.0
}

fn angle_distance(a: i16, b: i16) -> u16 {
    a.wrapping_sub(b).unsigned_abs()
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Boko {
    pub kind: u8,
    pub proc: u8,
    pub mode: u8,
    pub shape: [i16; 3],
    pub speed: Vec3,
    pub speed_f: f32,
    pub gravity: f32,
    pub turn_x: i16,
    pub spin: i16,
    pub slack: u8,
    pub floor_flag: bool,
    pub thrown: bool,
    pub throw_pitch: i16,
    pub hold: i16,
    pub flame: i16,
    pub top: Vec3,
    pub root: Vec3,
    pub seed: [i32; 3],
}

impl Boko {
    pub fn spawn(parameters: u32) -> Option<Self> {
        let (kind, flame) = if parameters == TYPE_TORCH {
            (TYPE_BOKO_STICK, FLAME_TIME)
        } else if parameters < TYPE_COUNT {
            (parameters as u8, 0)
        } else {
            return None;
        };
        Some(Self {
            kind,
            proc: PROC_WAIT,
            mode: MODE_WAIT,
            shape: [0; 3],
            speed: Vec3::ZERO,
            speed_f: 0.0,
            gravity: GRAVITY,
            turn_x: 0,
            spin: 0,
            slack: 0,
            floor_flag: false,
            thrown: false,
            throw_pitch: 0,
            hold: 0,
            flame,
            top: Vec3::ZERO,
            root: Vec3::ZERO,
            seed: [RND_SEED; 3],
        })
    }

    pub fn cull(&self) -> (Vec3, Vec3) {
        let slot = usize::from(self.kind);
        (CULL_MIN[slot], CULL_MAX[slot])
    }

    fn slot(&self) -> usize {
        usize::from(self.kind)
    }

    fn wait_init(&mut self) {
        self.speed_f = 0.0;
        self.turn_x = 0;
        self.spin = 0;
        self.proc = PROC_WAIT;
        self.mode = MODE_WAIT;
        self.gravity = GRAVITY;
    }

    fn move_init(&mut self) {
        self.proc = PROC_MOVE;
        self.mode = MODE_MOVE;
        self.slack = SLACK_MAX;
        self.floor_flag = false;
    }

    fn tilt_point(&self, v: Vec3, model: bool) -> Vec3 {
        let mut out = yrot(v, self.shape[1]);
        out = xrot(out, self.shape[0]);
        out = zrot(out, self.shape[2]);
        if model && self.kind == TYPE_STALFOS_MACE {
            out = xrot(out, STALFOS_TILT);
            out += Vec3::new(0.0, STALFOS_LIFT, 0.0);
        }
        out
    }

    fn top_root(&mut self, actor: &Actor, model: bool) {
        let slot = self.slot();
        let base = Vec3::new(actor.pos.x, actor.pos.y + BASE_LIFT, actor.pos.z);
        let top = self.tilt_point(TOP_OFFSET[slot], model);
        let root = self.tilt_point(ROOT_OFFSET[slot], model);
        self.top = base + top;
        self.root = base + root;
    }

    fn move_f(&mut self, actor: &mut Actor) {
        let (next, _) = stream::calc_speed(
            self.speed_f,
            actor.angle[1],
            self.speed.y,
            self.gravity,
            MAX_FALL,
        );
        self.speed = next;
        actor.pos += next;
    }

    fn crr_pos(&mut self, actor: &mut Actor, solid: Option<&Collision>) -> bool {
        match ground::surface(solid, actor.pos) {
            Some(floor) if actor.pos.y <= floor => {
                actor.pos.y = floor;
                self.speed.y = 0.0;
                true
            }
            _ => false,
        }
    }

    fn wait_proc(&mut self, actor: &mut Actor, solid: Option<&Collision>) {
        if self.floor_flag {
            return;
        }
        self.move_f(actor);
        if !self.crr_pos(actor, solid) {
            self.move_init();
        }
    }

    fn move_proc(&mut self, actor: &mut Actor, solid: Option<&Collision>, events: &mut SimEvents) {
        let slot = self.slot();
        self.move_f(actor);
        self.top_root(actor, false);

        let base = ground::surface(solid, actor.pos);
        let f30 = ground::surface(solid, self.root).or(base);
        let f1 = ground::surface(solid, self.top).or(base);
        let (f30, f1) = match (f30, f1) {
            (Some(root), Some(top)) => (root, top),
            _ => (f32::NEG_INFINITY, f32::NEG_INFINITY),
        };
        let f31 = self.root.y - f30;
        let f3 = self.top.y - f1;

        let mut struck = false;
        if (f31 < f3 && f31 < 0.0) || (f3 < f31 && f3 < 0.0) {
            if self.speed.y < BOUNCE_LIMIT {
                self.speed.y *= BOUNCE_GAIN;
                if self.speed.y > BOUNCE_CAP {
                    self.speed.y = BOUNCE_CAP;
                }
            }
            if self.slack != 0 {
                if f31 < f3 {
                    actor.pos.y -= f31;
                } else {
                    actor.pos.y -= f3;
                }
                self.slack -= 1;
            }
            let span = Vec3::new(self.top.x - self.root.x, f1 - f30, self.top.z - self.root.z);
            let tilt = self.shape[0]
                .wrapping_sub(walk::to_s16(-span.y.atan2(span.x.hypot(span.z))));
            self.turn_x = ((tilt) as f32 * FLAT_RATE) as i16;
            if self.turn_x == 0 && tilt != 0 {
                self.turn_x = tilt.wrapping_neg();
            }
            if self.turn_x == 0 && self.speed_f.abs() < FLAT_SPEED {
                self.slack = self.slack.saturating_sub(SLACK_SLIDE);
                self.turn_x = if self.shape[0] > 0 {
                    FLAT_ANGLE.wrapping_neg()
                } else {
                    FLAT_ANGLE
                };
            }
            events.list.push(SimEvent::Se(BOUND_SE[slot]));
            self.speed_f *= MOVE_DAMP;
            self.spin = ((self.spin) as f32 * MOVE_DAMP) as i16;
            struck = true;
        }

        if angle_distance(self.spin, 0) > angle_distance(self.turn_x, 0) {
            self.shape[0] = self.shape[0].wrapping_add(self.spin);
        } else {
            self.shape[0] = self.shape[0].wrapping_add(self.turn_x);
        }

        if self.spin == 0 {
            self.shape[0] = self.shape[0].clamp(-PITCH_CLAMP, PITCH_CLAMP);
        }

        if self.turn_x != 0 {
            let target = if self.shape[2].unsigned_abs() <= ROLL_LIMIT as u16 {
                0
            } else {
                ROLL_FOLD
            };
            self.shape[2] = walk::add_calc_angle_s(
                self.shape[2],
                target,
                ROLL_SCALE,
                ROLL_MAX_STEP,
                ROLL_MIN_STEP,
            );
        }

        let fall = self.speed.y;
        let hit = self.crr_pos(actor, solid);
        if hit && fall <= LAND_SPEED {
            let normal = ground::slope_normal(solid, actor.pos);
            let yaw = walk::to_s16(normal.x.atan2(normal.z));
            let roll = yaw.wrapping_sub(self.shape[1]);
            let flat = walk::to_s16(
                (normal.x.hypot(normal.z) * walk::from_s16(roll).cos()).atan2(normal.y),
            );
            let lean = flat.wrapping_sub(self.shape[0]);
            if lean.unsigned_abs() > SLOPE_LIMIT as u16 && self.slack != 0 {
                self.speed.y = HOP_LIFT;
                if self.speed_f < HOP_SPEED {
                    self.speed_f = HOP_SPEED;
                }
                self.spin = 0;
                self.turn_x = (lean >> PITCH_DIV).wrapping_neg();
                self.slack = self.slack.saturating_sub(SLACK_KICK);
            } else {
                self.shape[0] = flat;
                self.shape[2] = 0;
                self.wait_init();
            }
            if !struck {
                events.list.push(SimEvent::Se(BOUND_SE[slot]));
                self.speed_f *= MOVE_DAMP;
            }
        } else {
            self.speed.y = fall;
        }
    }

    fn carry_proc(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        solid: Option<&Collision>,
        events: &mut SimEvents,
    ) {
        if self.thrown {
            self.proc = PROC_THROW;
            self.mode = MODE_THROW;
            self.gravity = 0.0;
            self.shape[0] = self.throw_pitch;
            self.shape[1] = walk::to_s16(player.face);
            actor.angle[1] = self.shape[1];
            let (sin, cos) = walk::from_s16(self.shape[0]).sin_cos();
            self.speed_f = THROW_SPEED * cos;
            self.speed.y = -THROW_SPEED * sin;
            self.hold = THROW_TIMER;
            self.thrown = false;
            self.throw_proc(actor, Some(player), solid, events, None);
            return;
        }
        if self.floor_flag {
            if self.shape[2] >= CARRY_FLIP {
                self.shape[2] = 0;
                self.shape[0] = self.shape[0].wrapping_add(HALF_TURN);
                self.shape[1] = self.shape[1].wrapping_add(HALF_TURN);
            }
            self.wait_init();
            return;
        }
        actor.angle[1] = self.shape[1].wrapping_add(HALF_TURN);
        self.move_init();
        let (sin, cos) = walk::from_s16(self.shape[1]).sin_cos();
        let mut tries = 0;
        while tries < BACKSTEPS {
            if ground::surface(solid, actor.pos).is_some() {
                break;
            }
            actor.pos.x -= BACKSTEP * sin;
            actor.pos.z -= BACKSTEP * cos;
            tries += 1;
        }
        if tries == BACKSTEPS {
            actor.pos = Vec3::new(player.pos.x, FALLBACK_Y, player.pos.z);
        }
        actor.pos.y -= BASE_LIFT;
        self.turn_x = 0;
        self.gravity = GRAVITY;
        self.move_proc(actor, solid, events);
    }

    fn throw_proc(
        &mut self,
        actor: &mut Actor,
        player: Option<&Player>,
        solid: Option<&Collision>,
        events: &mut SimEvents,
        health: Option<&mut Health>,
    ) {
        let slot = self.slot();
        let start = self.root;
        self.move_f(actor);
        self.shape[0] = walk::to_s16(-self.speed.y.atan2(self.speed.x.hypot(self.speed.z)));
        self.shape[1] = walk::to_s16(self.speed.x.atan2(self.speed.z));
        actor.angle[1] = self.shape[1];
        if self.hold > 0 {
            self.hold -= 1;
            if self.hold == 0 {
                self.gravity = GRAVITY;
            }
        }
        self.crr_pos(actor, solid);
        self.top_root(actor, true);

        let mut turn: Option<i16> = None;
        if let Some(health) = health
            && let Some(player) = player
            && walk::swept(start, self.top, player.pos, CPS_RADIUS[slot])
        {
            health.damage(damage::BOKO_WEAPONS[slot] as u8);
            let scatter = walk::to_s16(rnd_fx(&mut self.seed, THROW_SPREAD));
            turn = Some(actor.angle[1].wrapping_add(scatter));
        }
        if let Some(turn) = turn {
            self.throw_reverse(actor, turn, events);
        }
    }

    fn throw_reverse(&mut self, actor: &mut Actor, turn: i16, events: &mut SimEvents) {
        let slot = self.slot();
        events.list.push(SimEvent::Se(BOUND_SE[slot]));
        self.move_init();
        self.spin = if self.shape[0] >= 0 {
            REVERSE_SPIN.wrapping_neg()
        } else {
            REVERSE_SPIN
        };
        self.turn_x = 0;
        self.gravity = GRAVITY;
        self.speed.y = REVERSE_LIFT;
        self.speed_f *= REVERSE_KEEP;
        let scatter = walk::to_s16(rnd_fx(&mut self.seed, REVERSE_SPREAD));
        actor.angle[1] = turn.wrapping_add(scatter);
        let fold = (rnd_fx(&mut self.seed, REVERSE_JITTER) + REVERSE_BASE) as i16;
        if angle_distance(actor.angle[1], self.shape[1]) >= REVERSE_TURN as u16 {
            self.spin = fold.wrapping_neg();
            self.shape[1] = actor.angle[1].wrapping_add(HALF_TURN);
        } else {
            self.spin = fold;
            self.shape[1] = actor.angle[1];
        }
    }

    fn flame_proc(&mut self, sea: Option<&Sea>, events: &mut SimEvents) {
        if self.flame == 0 {
            return;
        }
        if self.mode != MODE_ENEMY_CARRY {
            self.flame -= 1;
        }
        let (wet, surface) = super::water::probe(sea, self.top.x, self.top.z);
        if wet && surface >= self.top.y {
            self.flame = 0;
        }
        if self.flame == 0 {
            events.list.push(SimEvent::Se(SE_TORCH_OFF));
        }
    }
}

pub fn is_boko(name: &str) -> bool {
    BOKO_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    sea: Option<Res<Sea>>,
    players: Query<&Player>,
    mut health: ResMut<Health>,
    mut events: ResMut<SimEvents>,
    mut bokos: Query<(&mut Actor, &mut Boko)>,
) {
    let solid = collision.as_deref();
    let sea = sea.as_deref();
    let player = players.iter().next().copied();
    for (mut actor, mut boko) in &mut bokos {
        let actor = &mut *actor;
        let boko = &mut *boko;
        match boko.proc {
            PROC_WAIT => boko.wait_proc(actor, solid),
            PROC_MOVE => boko.move_proc(actor, solid, &mut events),
            PROC_CARRY => {
                if let Some(player) = player.as_ref() {
                    boko.carry_proc(actor, player, solid, &mut events);
                }
            }
            PROC_THROW => boko.throw_proc(
                actor,
                player.as_ref(),
                solid,
                &mut events,
                Some(&mut *health),
            ),
            _ => {}
        }
        boko.top_root(actor, true);
        boko.flame_proc(sea, &mut events);
    }
}
