use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::collision::{Collision, MoveFrom};
use super::ground;
use super::player::Player;
use super::walk;

pub const NH_ACTORS: [&str; 1] = ["Nh"];

const TYPE_MASK: u32 = 0xFF;
const TYPE_BOTTLE: u8 = 0x01;

const ACTION_STARTING: i8 = 0;
const ACTION_ONGOING: i8 = 1;
const ACTION_ENDING: i8 = -1;

const START_RISE: f32 = 1.0;
const GRAVITY: f32 = 0.25;
const MAX_FALL: f32 = -100.0;
const HEIGHT_ABOVE_GROUND: f32 = 50.0;
const AIR_BAND: f32 = 10.0;
const ASCENT: f32 = 1.0;
const DESCENT: f32 = -1.0;

const HOME_RADIUS: f32 = 50.0;
const MAX_HOME_DIST: f32 = 250.0;
const FLEE_DISTANCE: f32 = 600.0;
const MOVE_EPS: f32 = 0.001;
const MIN_FRIGHTEN: f32 = 5.0;
const FLEE_SPEED: f32 = 5.0;
const FLEE_STEP: f32 = 0.5;
const IDLE_SCALE: f32 = 0.1;
const IDLE_MAX: f32 = 10.0;
const IDLE_MIN: f32 = 1.0;

const TURN_STEP: i16 = 0x400;
const OPPOSITE: i16 = -0x8000;
const WOBBLE_BIAS: i16 = 0x2000;
const WOBBLE_MIN: f32 = 15.0;
const WOBBLE_RANGE: f32 = 20.0;
const AIM_GATE: u16 = 0x1000;
const AIM_SHIFT: i16 = 0x4000;

const ESCAPE_LIFE: i16 = 5 * 30;
const BOTTLE_LIFE: i32 = 10 * 30;
const ALPHA_TOP: i32 = 0xFF;
const ALPHA_STEP: i32 = 4;

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    None,
    Wait,
    Escape,
    Return,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Nh {
    pub kind: u8,
    pub act: Act,
    pub status: i8,
    pub speed_f: f32,
    pub speed_y: f32,
    pub max_fall: f32,
    pub player_dist: f32,
    pub ground_y: f32,
    pub bottle_timer: i32,
    pub alpha: i32,
    pub escape_timer: i16,
    pub wobble_dir: u8,
    pub wobble_timer: u8,
    pub home: Vec3,
    pub seed: [i32; 3],
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
    walk::to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn player_distance(from: Vec3, to: Vec3) -> f32 {
    (to - from).length()
}

fn gap_xz(from: Vec3, to: Vec3) -> f32 {
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    dx * dx + dz * dz
}

impl Nh {
    pub fn spawn(parameters: u32, home: Vec3) -> Self {
        Self {
            kind: (parameters & TYPE_MASK) as u8,
            act: Act::None,
            status: ACTION_STARTING,
            speed_f: 0.0,
            speed_y: START_RISE,
            max_fall: MAX_FALL,
            player_dist: 0.0,
            ground_y: 0.0,
            bottle_timer: BOTTLE_LIFE,
            alpha: ALPHA_TOP,
            escape_timer: 0,
            wobble_dir: 0,
            wobble_timer: 0,
            home,
            seed: [RND_SEED; 3],
        }
    }

    fn is_bottle(&self) -> bool {
        self.kind == TYPE_BOTTLE
    }

    fn home_distance(&self, pos: Vec3) -> f32 {
        gap_xz(pos, self.home).sqrt()
    }

    fn set_action(&mut self, next: Act, actor: &mut Actor, pos: Vec3, player: &Player) {
        if self.act == next {
            return;
        }
        self.act = next;
        self.status = ACTION_STARTING;
        self.escape_timer = 0;
        self.run(actor, pos, player);
    }

    fn wobble_bias(&self) -> i16 {
        if self.wobble_dir != 0 {
            -WOBBLE_BIAS
        } else {
            WOBBLE_BIAS
        }
    }

    fn wobble_step(&mut self) {
        if self.wobble_timer != 0 {
            self.wobble_timer -= 1;
        }
        if self.wobble_timer == 0 {
            self.wobble_dir ^= 1;
            self.wobble_timer = (WOBBLE_MIN + rnd(&mut self.seed) * WOBBLE_RANGE) as u8;
        }
    }

    fn move_proc(&mut self, actor: &mut Actor, target_speed: f32, step: f32, target_angle: i16) {
        walk::chase_angle_s(&mut actor.angle[1], target_angle, TURN_STEP);
        walk::chase_f(&mut self.speed_f, target_speed, step);
    }

    fn wait(&mut self, actor: &mut Actor, pos: Vec3, player: &Player) {
        if self.status == ACTION_STARTING {
            self.status = ACTION_ONGOING;
            self.player_dist = player_distance(pos, player.pos);
        } else if self.status != ACTION_ENDING {
            walk::add_calc(&mut self.speed_f, 0.0, IDLE_SCALE, IDLE_MAX, IDLE_MIN);
            if self.home_distance(pos) > HOME_RADIUS {
                self.set_action(Act::Return, actor, pos, player);
            }
        }
    }

    fn escape_end(&mut self, actor: &mut Actor, pos: Vec3, player: &Player) -> bool {
        if self.is_bottle() {
            return false;
        }
        if self.escape_timer != 0 {
            self.escape_timer -= 1;
        }
        if self.escape_timer == 0 {
            self.set_action(Act::Wait, actor, pos, player);
            return true;
        }
        if gap_xz(pos, self.home) > MAX_HOME_DIST * MAX_HOME_DIST {
            self.set_action(Act::Return, actor, pos, player);
            return true;
        }
        false
    }

    fn escape(&mut self, actor: &mut Actor, pos: Vec3, player: &Player) {
        if self.status == ACTION_STARTING {
            self.status = ACTION_ONGOING;
            self.wobble_dir = 0;
            self.wobble_timer = 0;
            self.escape_timer = ESCAPE_LIFE;
        } else if self.status != ACTION_ENDING && !self.escape_end(actor, pos, player) {
            let mut target = bearing(pos, player.pos).wrapping_add(OPPOSITE);
            self.wobble_step();
            target = target.wrapping_add(self.wobble_bias());
            self.move_proc(actor, FLEE_SPEED, FLEE_STEP, target);
        }
    }

    fn return_home(&mut self, actor: &mut Actor, pos: Vec3, player: &Player) {
        if self.status == ACTION_STARTING {
            self.status = ACTION_ONGOING;
            self.wobble_dir = 0;
            self.wobble_timer = 0;
            self.escape_timer = ESCAPE_LIFE;
        } else if self.status != ACTION_ENDING {
            if self.home_distance(pos) < HOME_RADIUS {
                self.set_action(Act::Wait, actor, pos, player);
            } else {
                let mut target = bearing(pos, self.home);
                if gap_xz(pos, self.home) < MAX_HOME_DIST * MAX_HOME_DIST {
                    let offset = target.wrapping_sub(bearing(pos, player.pos));
                    if offset.unsigned_abs() < AIM_GATE {
                        target = if offset < 0 {
                            target.wrapping_sub(AIM_SHIFT)
                        } else {
                            target.wrapping_add(AIM_SHIFT)
                        };
                    }
                }
                self.wobble_step();
                target = target.wrapping_add(self.wobble_bias());
                self.move_proc(actor, FLEE_SPEED, FLEE_STEP, target);
            }
        }
    }

    fn run(&mut self, actor: &mut Actor, pos: Vec3, player: &Player) {
        match self.act {
            Act::Wait => self.wait(actor, pos, player),
            Act::Escape => self.escape(actor, pos, player),
            Act::Return => self.return_home(actor, pos, player),
            Act::None => {}
        }
    }

    fn action(&mut self, actor: &mut Actor, pos: Vec3, player: &Player) {
        if self.act == Act::None {
            self.speed_f = 0.0;
            self.set_action(Act::Wait, actor, pos, player);
        }
        self.run(actor, pos, player);
    }

    fn search_player(&mut self, actor: &mut Actor, pos: Vec3, player: &Player, prev: Vec3) {
        if self.is_bottle() {
            self.set_action(Act::Escape, actor, pos, player);
            return;
        }
        let distance = player_distance(pos, player.pos);
        let closing = self.player_dist - distance;
        self.player_dist = distance;
        if gap_xz(player.pos, prev).sqrt() > MOVE_EPS
            && distance < FLEE_DISTANCE
            && closing > MIN_FRIGHTEN
        {
            self.set_action(Act::Escape, actor, pos, player);
        }
    }

    fn air_move(&mut self, pos: Vec3) {
        let ideal = self.ground_y + HEIGHT_ABOVE_GROUND;
        if pos.y < ideal - AIR_BAND {
            self.max_fall = ASCENT;
        } else if pos.y > ideal + AIR_BAND {
            self.max_fall = DESCENT;
        }
    }

    fn bg_check(&mut self, pos: Vec3, solid: Option<&Collision>) {
        if let Some(floor) = ground::surface(solid, pos) {
            self.ground_y = floor;
        }
    }

    fn check_timer(&mut self) -> bool {
        if !self.is_bottle() {
            return false;
        }
        if self.bottle_timer != 0 {
            self.bottle_timer -= 1;
        }
        if self.bottle_timer != 0 {
            return false;
        }
        self.alpha -= ALPHA_STEP;
        if self.alpha < 0 {
            self.alpha = 0;
        }
        self.alpha == 0
    }

    fn execute(
        &mut self,
        actor: &mut Actor,
        player: &Player,
        prev: Vec3,
        solid: Option<&Collision>,
    ) -> bool {
        if self.max_fall < self.speed_y {
            self.speed_y -= GRAVITY;
            if self.speed_y < self.max_fall {
                self.speed_y = self.max_fall;
            }
        } else if self.max_fall > self.speed_y {
            self.speed_y += GRAVITY;
            if self.speed_y > self.max_fall {
                self.speed_y = self.max_fall;
            }
        }
        let (sin, cos) = walk::from_s16(actor.angle[1]).sin_cos();
        actor.pos.x += self.speed_f * sin;
        actor.pos.y += self.speed_y;
        actor.pos.z += self.speed_f * cos;
        self.bg_check(actor.pos, solid);
        if self.check_timer() {
            return true;
        }
        let pos = actor.pos;
        self.air_move(pos);
        self.search_player(actor, pos, player, prev);
        self.action(actor, pos, player);
        false
    }
}

pub fn is_nh(name: &str) -> bool {
    NH_ACTORS.contains(&name)
}

pub fn control(
    players: Query<&Player>,
    prev: Res<MoveFrom>,
    collision: Option<Res<Collision>>,
    mut flies: Query<(Entity, &mut Actor, &mut Nh), Without<Player>>,
    mut commands: Commands,
) {
    let solid = collision.as_deref();
    let Some(player) = players.iter().next().copied() else {
        return;
    };
    let prev = prev.0;
    let mut spent: Vec<Entity> = Vec::new();
    for (entity, mut actor, mut fly) in &mut flies {
        let actor = &mut *actor;
        let fly = &mut *fly;
        if fly.execute(actor, &player, prev, solid) {
            spent.push(entity);
        }
    }
    for entity in spent {
        let _ = commands.entity(entity).despawn();
    }
}
