use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::combat::{Combat, CutKind};
use super::collision::Collision;
use super::event::{ActiveEvent, EventList};
use super::ground;
use super::hammer::{self, Cut as HammerCut};
use super::intent::DoIntent;
use super::msg::{MSG_BOX_CLOSED, MsgBox};
use super::player::Player;
use super::player_status::{PlayerStatus, STTS1_WIND_WAKER_CONDUCT};
use super::sea::Sea;
use super::stream::{add_calc_angle_s2, cos_s, sin_s};
use super::walk::{add_calc, from_s16, to_s16};
use super::water::{self, Water};
use crate::assets::MsgArchives;
use crate::text::msgproc::{FPC_M_ERROR_PROCESS_ID, MsgGameInfo, MsgProcs, fop_msgm_message_set_actor};
use crate::text::STATUS_MSG_DESTROYED;

pub const KANBAN_ACTORS: [&str; 1] = ["Kanban"];

pub fn is_kanban(name: &str) -> bool {
    KANBAN_ACTORS.contains(&name)
}

pub const PARTS_SHIFT: u32 = 0x14;
pub const PARTS_ALL: u32 = 0x7FE;
pub const PARTS_WHOLE: u32 = 1;
pub const PART_COUNT: usize = 11;
pub const NO_CUT: i16 = 0xFF;

pub const BLOCK_MOTHER: i16 = 0;
pub const BLOCK_RETURN: i16 = 1;
pub const BLOCK_MOTHER_SWIM: i16 = 2;
pub const BLOCK_PART: i16 = 100;
pub const BLOCK_CHILD: i16 = 101;
pub const BLOCK_CHILD_SWIM: i16 = 102;

pub const STATE_CUT: i16 = 10;
pub const STATE_RISE: i16 = 11;
pub const STATE_AIR: i16 = 12;
pub const STATE_BOUNCE: i16 = 13;
pub const STATE_SHOCK: i16 = 14;
pub const STATE_RISE_BACK: i16 = 20;
pub const STATE_SEAT: i16 = 21;

pub const STATE_RETURN_WAIT: i16 = 30;
pub const STATE_RETURN_MOVE: i16 = 31;
pub const STATE_RETURN_TURN: i16 = 32;
pub const STATE_RETURN_DONE: i16 = 33;

pub const STATE_PART_LAUNCH: i16 = 0x6e;
pub const STATE_PART_FALL: i16 = 0x6f;
pub const STATE_PART_TUMBLE: i16 = 0x70;
pub const STATE_PART_BOUNCE: i16 = 0x71;
pub const STATE_PART_SHOCK: i16 = 0x72;
pub const STATE_PART_SLOW: i16 = 0x78;

pub const STATE_CHILD_WAIT: i16 = 0x82;
pub const STATE_CHILD_MOVE: i16 = 0x83;
pub const STATE_CHILD_TURN: i16 = 0x84;
pub const STATE_CHILD_DONE: i16 = 0x85;
pub const STATE_CHILD_WATER: i16 = 140;
pub const STATE_CHILD_FLOAT: i16 = 141;

pub const MOTHER_SWIM_WAIT: i16 = 40;
pub const MOTHER_SWIM_FLOAT: i16 = 41;

pub const GRAVITY_FALL: f32 = -3.0;
pub const GRAVITY_PART: f32 = -5.0;
pub const SEA_SINK: f32 = 120.0;
pub const SEA_FLOOR: f32 = 147.0;
pub const SEA_SWING: f32 = 2.0;
pub const SEA_STEP: f32 = 1000.0;
pub const SEA_BOB_CUT: f32 = 7.0;
pub const WATER_SPAN: f32 = 0.5;

pub const EYE_LIFT: f32 = 40.0;
pub const ATTENTION_LIFT: f32 = 60.0;
pub const TALK_DIST: f32 = 5.0;
pub const SPEAK_DIST: f32 = 6.0;

pub const MOTHER_RADIUS: f32 = 105.0;
pub const MOTHER_HEIGHT: f32 = 50.0;
pub const SHOCK_RANGE: f32 = 1000.0;
pub const SHOCK_TIMER: i16 = 100;
pub const SEAT_OFFSET: f32 = 70.0;
pub const SEAT_LIFT: f32 = 140.0;

pub const PIECE_RADIUS: f32 = 25.0;
pub const PIECE_HEIGHT: f32 = 10.0;

pub const SE_BREAK_BOARD: u32 = 0x6937;
pub const SE_FALL_WATER: u32 = 0x6919;

pub const TACT_ARIA: i32 = 4;
pub const TACT_SONG: i32 = -1;

pub const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
pub const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
pub const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
pub const RND_SEED: i32 = 100;

pub const CUT_TABLE: [i16; 32] = [
    255, 0, 4, 3, 3, 1, 0, 4, 4, 4, 1, 3, 1, 3, 1, 1, 1, 3, 1, 1, 0, 1, 0, 1, 0, 1, 1, 4, 0, 1, 1, 1,
];

pub const CUT_ARGS: [u32; 40] = [
    0x02000000, 0x04000000, 0x00800000, 0x01000000, 0x7FE00000, 0x7FE00000, 0x7FE00000, 0x7FE00000,
    0x04000000, 0x00800000, 0x01000000, 0x10000000, 0x40000000, 0x7FE00000, 0x7FE00000, 0x7FE00000,
    0x00400000, 0x00200000, 0x02000000, 0x04000000, 0x7FE00000, 0x7FE00000, 0x7FE00000, 0x7FE00000,
    0x00200000, 0x02000000, 0x04000000, 0x00800000, 0x7FE00000, 0x7FE00000, 0x7FE00000, 0x7FE00000,
    0x08000000, 0x00400000, 0x00200000, 0x02000000, 0x04000000, 0x00800000, 0x01000000, 0x10000000,
];

pub const CUT_NONE: u32 = 0x7FE00000;

pub const AT_NONE: u8 = 0;
pub const AT_HAMMER: u8 = 1;
pub const AT_SWORD: u8 = 2;

pub const CUT_SWORD_A: u8 = 0x01;
pub const CUT_SWORD_F: u8 = 0x02;
pub const CUT_SWORD_R: u8 = 0x03;
pub const CUT_SWORD_L: u8 = 0x04;
pub const CUT_JUMP_SWORD: u8 = 0x05;
pub const CUT_SWORD_EA: u8 = 0x06;
pub const CUT_SWORD_EB: u8 = 0x07;
pub const CUT_TURN: u8 = 0x08;
pub const CUT_ROLL: u8 = 0x09;
pub const CUT_HAMMER_SIDE: u8 = 0x11;
pub const CUT_HAMMER_FRONT: u8 = 0x12;

pub const NO_MSG: u32 = FPC_M_ERROR_PROCESS_ID;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct BoardMsg {
    pub id: u32,
}

impl BoardMsg {
    pub fn none() -> Self {
        Self { id: NO_MSG }
    }

    pub fn open(&self) -> bool {
        self.id != NO_MSG
    }
}

pub fn aria_playing() -> bool {
    TACT_SONG == TACT_ARIA
}

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = seed[0] as f32 / RND_SCALE[0]
        + seed[1] as f32 / RND_SCALE[1]
        + seed[2] as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

fn rnd_f(seed: &mut [i32; 3], range: f32) -> f32 {
    range * rnd(seed)
}

fn rnd_fx(seed: &mut [i32; 3], range: f32) -> f32 {
    range * (rnd(seed) - 0.5) * 2.0
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

fn distance_angle_s(a: i16, b: i16) -> i32 {
    i32::from(a.wrapping_sub(b)).abs()
}

fn target_angle_y(from: Vec3, to: Vec3) -> i16 {
    to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn bearing(from: Vec3, to: Vec3) -> i16 {
    target_angle_y(from, to)
}

fn ground_angle(normal: Vec3) -> [i16; 3] {
    [
        to_s16(-normal.z.atan2(normal.y)),
        0,
        to_s16(-normal.x.atan2(normal.y)),
    ]
}

fn cut_index(kind: CutKind) -> u8 {
    match kind {
        CutKind::A => CUT_SWORD_A,
        CutKind::F => CUT_SWORD_F,
        CutKind::R => CUT_SWORD_R,
        CutKind::L => CUT_SWORD_L,
        CutKind::EA => CUT_SWORD_EA,
        CutKind::EB => CUT_SWORD_EB,
    }
}

fn touches(centre: Vec3, radius: f32, height: f32, at: Vec3) -> bool {
    let dx = at.x - centre.x;
    let dz = at.z - centre.z;
    let dy = at.y - centre.y;
    dx * dx + dz * dz <= radius * radius && dy >= 0.0 && dy <= height
}

fn flat_span(a: Vec3, b: Vec3) -> f32 {
    let dx = a.x - b.x;
    let dz = a.z - b.z;
    (dx * dx + dz * dz).sqrt()
}

fn lift_by(pos: Vec3, lift: f32) -> Vec3 {
    Vec3::new(pos.x, pos.y + lift, pos.z)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum At {
    None,
    Hammer,
    Sword,
}

#[derive(Debug, Clone, Copy)]
pub struct Env<'a> {
    pub player: Option<Player>,
    pub player_face: i16,
    pub solid: Option<&'a Collision>,
    pub sea: Option<&'a Sea>,
    pub pool: Option<&'a Water>,
    pub tact: bool,
    pub conduct: bool,
    pub at: At,
    pub sword_top: Vec3,
    pub blasts: &'a [Vec3],
    pub events: Vec<u32>,
}

impl Env<'_> {
    fn player_pos(&self) -> Vec3 {
        self.player.map_or(Vec3::ZERO, |player| player.pos)
    }

    fn present(&self) -> bool {
        self.player.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct Piece {
    pub actor: Actor,
    pub tag: Kanban,
}

#[derive(Component, Debug, Clone)]
pub struct Kanban {
    pub mother: bool,
    pub parts: u32,
    pub spin: u8,
    pub hit_kind: u8,
    pub gone: u8,
    pub splashed: u8,
    pub bob: f32,
    pub cyl_radius: f32,
    pub cyl_height: f32,
    pub ground_angle: [i16; 3],
    pub timers: [i16; 5],
    pub wave: i16,
    pub block: i16,
    pub state: i16,
    pub cut_bits: i16,
    pub cut_point: i16,
    pub bounce: i16,
    pub parent: Option<Entity>,
    pub pivot: Vec3,
    pub spawn_pos: Vec3,
    pub offset: Vec3,
    pub home_angle: [i16; 3],
    pub home_pos: Vec3,
    pub start_pivot: Vec3,
    pub speed: Vec3,
    pub speed_f: f32,
    pub gravity: f32,
    pub ground_hit: bool,
    pub wall_hit: bool,
    pub last_at: u8,
    pub seed: [i32; 3],
    pub entered: bool,
}

impl Default for Kanban {
    fn default() -> Self {
        Self {
            mother: true,
            parts: PARTS_WHOLE,
            spin: 0,
            hit_kind: 0,
            gone: 0,
            splashed: 0,
            bob: 0.0,
            cyl_radius: 0.0,
            cyl_height: 0.0,
            ground_angle: [0; 3],
            timers: [0; 5],
            wave: 0,
            block: BLOCK_MOTHER,
            state: STATE_CUT,
            cut_bits: 0,
            cut_point: 0,
            bounce: 0,
            parent: None,
            pivot: Vec3::ZERO,
            spawn_pos: Vec3::ZERO,
            offset: Vec3::ZERO,
            home_angle: [0; 3],
            home_pos: Vec3::ZERO,
            start_pivot: Vec3::ZERO,
            speed: Vec3::ZERO,
            speed_f: 0.0,
            gravity: 0.0,
            ground_hit: false,
            wall_hit: false,
            last_at: AT_NONE,
            seed: [RND_SEED; 3],
            entered: false,
        }
    }
}

impl Kanban {
    pub fn spawn(parameters: u32, pos: Vec3, angle: [i16; 3]) -> Self {
        let mut tag = Self::default();
        tag.parts = parameters >> PARTS_SHIFT;
        tag.home_angle = angle;
        tag.home_pos = pos;
        tag.spawn_pos = pos;
        tag.mother = true;
        if tag.parts == 0 {
            tag.mother = false;
            tag.cyl_radius = MOTHER_RADIUS;
            tag.cyl_height = MOTHER_HEIGHT;
            tag.parts = PARTS_WHOLE;
        } else {
            tag.cut_point = angle[2];
        }
        tag
    }

    pub fn piece(parameters: u32, pos: Vec3, angle: [i16; 3], parent: Entity) -> Piece {
        let mut tag = Kanban::spawn(parameters, pos, angle);
        tag.parent = Some(parent);
        let actor = Actor {
            name: KANBAN_ACTORS[0].to_owned(),
            parameters,
            pos,
            angle,
            set_id: 0xFFFF,
            room: 0,
        };
        Piece { actor, tag }
    }

    pub fn link(&mut self, parent_pos: Vec3, parent_angle: [i16; 3]) {
        self.home_pos = parent_pos;
        self.home_angle = parent_angle;
        self.pivot.y = SEAT_OFFSET;
        self.offset.y = SEAT_LIFT;
        self.start_pivot = self.pivot;
        self.block = BLOCK_PART;
        self.state = STATE_PART_LAUNCH;
    }

    pub fn eye(&self, pos: Vec3) -> Vec3 {
        lift_by(pos, EYE_LIFT)
    }

    pub fn attention(&self, pos: Vec3) -> Vec3 {
        lift_by(pos, EYE_LIFT + ATTENTION_LIFT - EYE_LIFT + ATTENTION_LIFT - EYE_LIFT)
    }

    fn tilt_caps(&mut self) {
        if self.state == STATE_BOUNCE || self.state == STATE_PART_BOUNCE {
            self.spin = self.spin.wrapping_add(1) & 3;
            self.bounce ^= 0xFF00;
            if self.spin & 1 == 0 {
                self.bounce = 0x400;
            }
            if self.spin & 3 == 0 {
                self.ground_angle = [0; 3];
                self.bounce = 0;
                self.state += 1;
            }
        }
    }

    fn step_timers(&mut self) {
        for slot in self.timers.iter_mut() {
            if *slot != 0 {
                *slot -= 1;
            }
        }
    }

    fn water_probe(&self, env: &Env<'_>) -> (bool, f32, Option<f32>) {
        let (in_sea, sea_y) = water::probe(env.sea, self.spawn_pos.x, self.spawn_pos.z);
        let pool = if in_sea {
            None
        } else {
            env.pool.filter(|pool| pool.in_area).map(|pool| pool.surface)
        };
        (in_sea, sea_y, pool)
    }

    fn float_y(&mut self, surface: f32) {
        if self.cut_point == 1 {
            self.bob = SEA_BOB_CUT;
        }
        if self.spawn_pos.y < surface - SEA_SINK {
            self.gravity = 0.0;
            self.speed_f = 0.0;
            self.speed = Vec3::ZERO;
            self.wave = self.wave.wrapping_add(SEA_STEP as i16);
            let swell = SEA_FLOOR + SEA_SWING * sin_s(self.wave);
            add_calc(&mut self.spawn_pos.y, surface - swell, 1.0, 30.0, 0.0);
        }
    }

    fn sea_water_check(&mut self, env: &mut Env<'_>) -> bool {
        self.gravity = GRAVITY_FALL;
        let (in_sea, sea_y, pool) = self.water_probe(env);
        let mut floating = false;
        if in_sea {
            if self.cut_point == 1 {
                self.bob = SEA_BOB_CUT;
            }
            let before = self.spawn_pos.y;
            self.float_y(sea_y);
            floating = before != self.spawn_pos.y || self.gravity == 0.0;
        } else if let Some(surface) = pool {
            if self.cut_point == 1 {
                self.bob = SEA_BOB_CUT;
            }
            self.gravity = GRAVITY_FALL;
            let before = self.spawn_pos.y;
            self.float_y(surface);
            floating = before != self.spawn_pos.y || self.gravity == 0.0;
        }
        if !floating {
            self.bob = 0.0;
            return false;
        }
        if self.splashed == 0 {
            self.wave = 0;
            self.splashed = 1;
            env.events.push(SE_FALL_WATER);
        }
        true
    }

    fn shock_damage_check(&mut self, env: &Env<'_>) -> bool {
        for blast in env.blasts {
            if flat_span(*blast, self.spawn_pos) < SHOCK_RANGE {
                self.timers[4] = SHOCK_TIMER;
                return true;
            }
        }
        if env.at == At::Hammer {
            if flat_span(env.sword_top, self.spawn_pos) < SHOCK_RANGE {
                return true;
            }
        }
        false
    }

    fn return_move(&mut self, env: &mut Env<'_>) -> bool {
        add_calc0(&mut self.bob, 1.0, 2.0);
        let delta = self.home_pos - self.spawn_pos;
        let facing = to_s16(delta.x.atan2(delta.z));
        add_calc_angle_s2(&mut facing_tilt(&mut self.spawn_pos), facing, 1, 0x1000);
        self.ground_angle[1] = facing;
        if delta.y.abs() > 2.0 {
            add_calc(&mut self.spawn_pos.y, self.home_pos.y, 1.0, 10.0, 0.0);
            return false;
        }
        if delta.x * delta.x + delta.z * delta.z < 2.0 * 2.0 {
            self.speed_f = 0.0;
            self.spawn_pos = self.home_pos;
            return true;
        }
        add_calc(&mut self.spawn_pos.x, self.home_pos.x, 1.0, 10.0, 0.0);
        add_calc(&mut self.spawn_pos.z, self.home_pos.z, 1.0, 10.0, 0.0);
        let _ = env;
        false
    }

    fn mother_return_move(&mut self, env: &mut Env<'_>) {
        match self.state {
            STATE_RETURN_WAIT => {
                if env.conduct {
                    return;
                }
                self.timers[0] = rnd_f(&mut self.seed, 30.0) as i16;
                self.state += 1;
            }
            STATE_RETURN_MOVE => {
                if self.timers[0] == 0 {
                    self.pivot = Vec3::ZERO;
                    self.offset = Vec3::ZERO;
                    self.bounce = 0;
                    if self.return_move(env) {
                        self.state += 1;
                    }
                }
            }
            STATE_RETURN_TURN => {
                let mut yaw = self.ground_angle[1];
                add_calc_angle_s2(&mut yaw, self.home_angle[1], 1, 0x1000);
                self.ground_angle[1] = yaw;
                if distance_angle_s(yaw, self.home_angle[1]) < 0x100 {
                    self.ground_angle[1] = self.home_angle[1];
                    self.state += 1;
                }
            }
            STATE_RETURN_DONE => {
                if self.parts == PARTS_ALL {
                    self.cut_bits = 0;
                    self.cut_point = 0;
                    self.gone = 1;
                    self.cyl_radius = MOTHER_RADIUS;
                    self.cyl_height = MOTHER_HEIGHT;
                    self.block = BLOCK_MOTHER;
                    self.state = STATE_CUT;
                    return;
                }
            }
            _ => {}
        }
        if self.timers[0] == 0 && self.state >= STATE_RETURN_MOVE {
            self.align_home();
        }
    }

    fn align_home(&mut self) {
        let mut angle = self.ground_angle;
        add_calc_angle_s2(&mut angle[0], self.home_angle[0], 1, 0x1000);
        add_calc_angle_s2(&mut angle[2], self.home_angle[2], 1, 0x1000);
        self.ground_angle = angle;
    }

    fn child_return_move(&mut self, env: &mut Env<'_>) {
        match self.state {
            STATE_CHILD_WAIT => {
                if env.conduct {
                    return;
                }
                self.timers[0] = rnd_f(&mut self.seed, 30.0) as i16;
                self.state += 1;
            }
            STATE_CHILD_MOVE => {
                if self.timers[0] == 0 {
                    self.pivot = Vec3::ZERO;
                    self.offset = Vec3::ZERO;
                    self.bounce = 0;
                    if self.return_move(env) {
                        self.state += 1;
                    }
                }
            }
            STATE_CHILD_TURN => {
                let mut yaw = self.ground_angle[1];
                add_calc_angle_s2(&mut yaw, self.home_angle[1], 1, 0x1000);
                self.ground_angle[1] = yaw;
                if distance_angle_s(yaw, self.home_angle[1]) < 0x100 {
                    self.ground_angle[1] = self.home_angle[1];
                    self.state += 1;
                    self.merge_into_parent();
                }
            }
            STATE_CHILD_DONE => {
                if self.gone == 1 {
                    self.drop = true;
                }
                if self.gone == 2 {
                    self.block = BLOCK_PART;
                    self.state = STATE_PART_LAUNCH;
                }
            }
            _ => {}
        }
        if self.timers[0] == 0 && self.state >= STATE_CHILD_MOVE {
            self.align_home();
        }
    }

    fn merge_into_parent(&mut self) {
        if let Some(parent) = self.parent {
            self.merge = Some(parent);
        }
    }

    fn cut_point_check(&mut self, actor: &mut Actor, env: &mut Env<'_>, pieces: &mut Vec<Piece>) {
        if self.cut_point == 0 && env.present() {
            let to_player = bearing(actor.pos, env.player_pos());
            let delta = actor.angle[1].wrapping_sub(to_player);
            let gap = i32::from(delta.unsigned_abs());
            if gap > 0x6000 {
                self.cut_point = 2;
            } else if gap > 0x2000 && gap < 0x6000 && to_player < 0 {
                self.cut_point = 2;
            }
        }
        let point = usize::try_from(self.cut_point).unwrap_or(0);
        let base = point * 8;
        if (self.cut_bits >> self.cut_point) & 1 != 0 || self.cut_bits & 2 != 0 {
            return;
        }
        self.cut_bits |= 1 << self.cut_point;
        if self.parts & 1 != 0 {
            self.parts = PARTS_ALL;
            self.gone = 0;
        }
        let mut remaining = self.parts << PARTS_SHIFT;
        let mut parameters = 0_u32;
        for slot in 0..8 {
            let Some(arg) = CUT_ARGS.get(base + slot).copied() else {
                break;
            };
            if arg != CUT_NONE && remaining & arg != 0 {
                remaining ^= arg;
                parameters |= arg;
            }
        }
        if parameters != 0 {
            let cut = self.cut_point;
            let mut angle = actor.angle;
            angle[2] = cut;
            let mut child = Kanban::piece(parameters, actor.pos, angle, actor_id(&actor));
            child.tag.parent = self.self_id;
            child.actor.room = actor.room;
            child.actor.set_id = actor.set_id;
            pieces.push(child);
            env.events.push(SE_BREAK_BOARD);
            self.parts = remaining >> PARTS_SHIFT;
            if cut == 1 {
                self.spin = 0;
                if self.cut_bits & 0x10 != 0 {
                    self.spin = 1;
                }
                actor.pos.y = self.home_pos.y - SEAT_OFFSET;
                self.pivot.y = SEAT_OFFSET;
                self.offset.y = SEAT_LIFT;
                self.cyl_radius = 0.0;
                self.cyl_height = 0.0;
                self.state = 0xb;
            }
            if self.cut_bits & 5 == 5 || cut == 4 {
                self.cyl_radius = PIECE_RADIUS;
                self.cyl_height = PIECE_HEIGHT;
            }
            actor.angle[2] = 0;
        }
    }

    fn cut_check(&mut self, actor: &mut Actor, env: &mut Env<'_>) -> u32 {
        if self.hit_kind != AT_NONE && env.present() {
            let reach = touches(
                actor.pos,
                self.cyl_radius,
                self.cyl_height,
                env.player_pos(),
            );
            if reach {
                self.last_at = self.hit_kind;
                self.hit_kind = AT_NONE;
                self.cyl_active = true;
                if self.hit_kind == AT_HAMMER {
                    self.gone_kind = 2;
                }
                let kind = self.last_at;
                self.last_at = AT_NONE;
                if kind == AT_HAMMER && player_cut_kind(env) != CUT_HAMMER_SIDE {
                    self.gone = 0;
                    self.cyl_radius = 0.0;
                    self.cyl_height = 0.0;
                    self.gone_kind = 1;
                    self.state = STATE_RISE_BACK;
                    return kind;
                }
                let table = cut_table_index(player_cut_kind(env));
                if table != NO_CUT {
                    self.cut_point = table;
                }
                return kind;
            }
        }
        AT_NONE
    }

    fn mother_move(
        &mut self,
        actor: &mut Actor,
        env: &mut Env<'_>,
        pieces: &mut Vec<Piece>,
    ) {
        match self.state {
            STATE_CUT => {
                if self.cut_check(actor, env) != AT_NONE {
                    return;
                }
            }
            STATE_RISE => {
                actor.angle[1] = env.player_face.wrapping_add(-0x4000);
                actor.angle[1] = actor.angle[1];
                self.spin = 0;
                if self.parts & 0x100 != 0 {
                    self.speed_f = 20.0;
                    self.speed.y = 10.0;
                    self.gravity = GRAVITY_FALL;
                } else {
                    self.speed_f = 3.0;
                    self.speed.y = 7.0;
                    self.gravity = GRAVITY_FALL;
                }
                self.state += 1;
            }
            STATE_AIR => {
                if self.sea_water_check(env) {
                    self.block = BLOCK_MOTHER_SWIM;
                    self.state = MOTHER_SWIM_WAIT;
                } else {
                    if self.wall_hit {
                        self.speed_f *= -1.0;
                    }
                    if self.speed.y < 0.0 {
                        let mut pitch = actor.angle[0];
                        add_calc_angle_s2(&mut pitch, 0x4000, 1, 0x1000);
                        actor.angle[0] = pitch;
                    }
                    if self.ground_hit {
                        self.bounce = 0x800;
                        self.ground_angle = env
                            .solid
                            .and_then(|solid| solid.ground_normal(actor.pos.x, actor.pos.z))
                            .map_or([0; 3], ground_angle);
                        self.speed_f = 0.0;
                        self.gravity = 0.0;
                        self.state = STATE_BOUNCE;
                    }
                }
            }
            STATE_BOUNCE => {
                if self.sea_water_check(env) {
                    self.block = BLOCK_MOTHER_SWIM;
                    self.state = MOTHER_SWIM_WAIT;
                    return;
                }
                let mut pitch = actor.angle[0];
                add_calc_angle_s2(&mut pitch, self.bounce.wrapping_add(0x4000), 1, 0x1000);
                actor.angle[0] = pitch;
                self.tilt_caps();
            }
            STATE_SHOCK => {
                if self.sea_water_check(env) {
                    self.block = BLOCK_MOTHER_SWIM;
                    self.state = MOTHER_SWIM_WAIT;
                } else if self.timers[4] == 0 && self.shock_damage_check(env) {
                    self.speed.y = rnd_f(&mut self.seed, 10.0) + 20.0;
                    self.gravity = GRAVITY_FALL;
                    self.state = STATE_AIR;
                }
            }
            STATE_RISE_BACK => self.state += 1,
            STATE_SEAT => {
                self.timers[1] = 2;
                let mut rest = 90.0;
                if self.cut_bits & 8 != 0 {
                    rest = 65.0;
                }
                if self.cut_bits & 0x10 != 0 {
                    rest = 20.0;
                }
                add_calc(&mut actor.pos.y, self.home_pos.y - rest, 1.0, 30.0, 0.0);
            }
            _ => {}
        }
        if self.state == STATE_CUT && env.present() && self.cyl_active {
            let _ = actor;
        }
        if self.state == STATE_CUT {
            self.cut_point_check(actor, env, pieces);
        }
    }

    fn mother_water_swim(&mut self, env: &mut Env<'_>) {
        match self.state {
            MOTHER_SWIM_WAIT => {
                self.bounce = 0;
                self.ground_angle[0] = 0x4000;
                self.cyl_radius = 0.0;
                self.cyl_height = 0.0;
                self.state += 1;
            }
            MOTHER_SWIM_FLOAT => {
                self.sea_water_check(env);
            }
            _ => {}
        }
    }

    fn parts_move(&mut self, actor: &mut Actor, env: &mut Env<'_>) {
        match self.state {
            STATE_PART_LAUNCH => {
                self.spin = 0;
                actor.angle[1] = bearing(env.player_pos(), actor.pos);
                if self.cut_point != 1 {
                    actor.angle[1] =
                        actor.angle[1].wrapping_add(rnd_fx(&mut self.seed, 8000.0) as i16);
                    self.speed_f = rnd_fx(&mut self.seed, 2.0) + 10.0;
                    self.speed.y = rnd_fx(&mut self.seed, 5.0) + 20.0;
                    self.gravity = GRAVITY_PART;
                    self.state = STATE_PART_FALL;
                } else {
                    if self.parts & 0x100 != 0 {
                        self.speed_f = 20.0;
                        self.speed.y = 10.0;
                        self.gravity = GRAVITY_FALL;
                    } else {
                        self.speed_f = 3.0;
                        self.speed.y = 7.0;
                        self.gravity = GRAVITY_FALL;
                    }
                    self.state = STATE_PART_SLOW;
                }
            }
            STATE_PART_FALL => {
                if self.wall_hit {
                    self.speed_f *= -1.0;
                }
                actor.angle[0] = actor.angle[0].wrapping_add(0x1000);
                if self.ground_hit {
                    self.speed_f = 5.0;
                    self.speed.y = 10.0;
                    self.state += 1;
                }
            }
            STATE_PART_TUMBLE => {
                let mut pitch = actor.angle[0];
                add_calc_angle_s2(&mut pitch, 0x4000, 1, 0x1000);
                actor.angle[0] = pitch;
                if self.ground_hit {
                    self.bounce = 0x800;
                    self.ground_angle = env
                        .solid
                        .and_then(|solid| solid.ground_normal(actor.pos.x, actor.pos.z))
                        .map_or([0; 3], ground_angle);
                    self.speed_f = 0.0;
                    self.gravity = 0.0;
                    self.state += 1;
                }
            }
            STATE_PART_BOUNCE => {
                let mut pitch = actor.angle[0];
                add_calc_angle_s2(&mut pitch, self.bounce.wrapping_add(0x4000), 1, 0x1000);
                actor.angle[0] = pitch;
                self.tilt_caps();
            }
            STATE_PART_SHOCK => {
                if self.timers[4] == 0 && self.shock_damage_check(env) {
                    self.speed.y = rnd_f(&mut self.seed, 10.0) + 20.0;
                    self.gravity = GRAVITY_FALL;
                    self.state = STATE_PART_TUMBLE;
                }
            }
            STATE_PART_SLOW => {
                if self.wall_hit {
                    self.speed_f *= -1.0;
                }
                if self.speed.y < 0.0 {
                    let mut pitch = actor.angle[0];
                    add_calc_angle_s2(&mut pitch, 0x4000, 1, 0x1000);
                    actor.angle[0] = pitch;
                }
                if self.ground_hit {
                    actor.angle[0] = 0x4000;
                    self.bounce = 0x800;
                    self.ground_angle = env
                        .solid
                        .and_then(|solid| solid.ground_normal(actor.pos.x, actor.pos.z))
                        .map_or([0; 3], ground_angle);
                    self.speed_f = 0.0;
                    self.gravity = 0.0;
                    self.state = STATE_PART_BOUNCE;
                }
            }
            _ => {}
        }
    }

    fn child_water_swim(&mut self, env: &mut Env<'_>) {
        match self.state {
            STATE_CHILD_WATER => {
                self.bounce = 0;
                self.ground_angle[0] = 0x4000;
                self.cyl_radius = 0.0;
                self.cyl_height = 0.0;
                self.state += 1;
            }
            STATE_CHILD_FLOAT => {
                self.sea_water_check(env);
            }
            _ => {}
        }
    }

    fn tact_return(&mut self, actor: &mut Actor, state: i16) {
        self.gravity = 0.0;
        self.speed_f = 0.0;
        self.speed = Vec3::ZERO;
        actor.pos = self.home_pos;
        self.block = BLOCK_RETURN;
        self.state = state;
    }

    fn move_step(&mut self, actor: &mut Actor, env: &Env<'_>) {
        let yaw = from_s16(actor.angle[1]);
        self.speed.x = self.speed_f * yaw.sin();
        self.speed.y += self.gravity;
        self.speed.z = self.speed_f * yaw.cos();
        let from = actor.pos;
        actor.pos += self.speed;
        if let Some(solid) = env.solid {
            let slid = solid.slide(from, actor.pos);
            self.wall_hit = slid != actor.pos;
            actor.pos = slid;
        } else {
            self.wall_hit = false;
        }
        self.ground_hit = ground::grounded(env.solid, actor.pos);
    }

    fn seat_lift(&mut self, actor: &mut Actor, solid: Option<&Collision>) {
        actor.pos.y += self.offset.y;
        actor.pos.y -= self.bob;
        let _ = ground::surface(solid, actor.pos);
        actor.pos.y += self.bob;
        actor.pos.y -= self.offset.y;
    }
}

fn facing_tilt(pos: &mut Vec3) -> i16 {
    let _ = pos;
    0
}

fn actor_id(actor: &Actor) -> Entity {
    Entity::from_raw_u32(u32::MAX).unwrap_or(Entity::PLACEHOLDER)
}

fn player_cut_kind(env: &Env<'_>) -> u8 {
    let _ = env;
    CUT_SWORD_A
}

fn cut_table_index(kind: u8) -> i16 {
    match kind {
        CUT_SWORD_A => CUT_TABLE[usize::from(CUT_SWORD_A)],
        CUT_SWORD_F => CUT_TABLE[usize::from(CUT_SWORD_F)],
        CUT_SWORD_R => CUT_TABLE[usize::from(CUT_SWORD_R)],
        CUT_SWORD_L => CUT_TABLE[usize::from(CUT_SWORD_L)],
        CUT_JUMP_SWORD => CUT_TABLE[usize::from(CUT_JUMP_SWORD)],
        CUT_SWORD_EA => CUT_TABLE[usize::from(CUT_SWORD_EA)],
        CUT_SWORD_EB => CUT_TABLE[usize::from(CUT_SWORD_EB)],
        CUT_HAMMER_SIDE => CUT_TABLE[usize::from(CUT_HAMMER_SIDE)],
        CUT_HAMMER_FRONT => CUT_TABLE[usize::from(CUT_HAMMER_FRONT)],
        _ => NO_CUT,
    }
}

pub fn control(
    mut msg_id: ResMut<BoardMsg>,
    procs: Res<MsgProcs>,
    mut box_: ResMut<MsgBox>,
    mut info: ResMut<MsgGameInfo>,
    archives: Option<Res<MsgArchives>>,
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    collision: Option<Res<Collision>>,
    sea: Option<Res<Sea>>,
    water: Option<Res<Water>>,
    combat: Res<Combat>,
    hammer_cut: Res<HammerCut>,
    status: Res<PlayerStatus>,
    players: Query<&Player>,
    mut boards: Query<(Entity, &mut Actor, &mut Kanban)>,
    mut commands: Commands,
) {
    let player = players.iter().next().copied();
    let player_face = player.map_or(0, |player| to_s16(player.face));
    let at = if hammer_cut.kind == hammer::CUT_HAMMER_SIDESWING
        || hammer_cut.kind == hammer::CUT_HAMMER_FRONTSWING
    {
        At::Hammer
    } else if combat.active && combat.frame >= 1 && combat.frame < 6 {
        At::Sword
    } else {
        At::None
    };
    let sword_top = player.map_or(Vec3::ZERO, |player| lift_by(player.pos, 40.0));
    let solid = collision.as_deref();
    let sea_ref = sea.as_deref();
    let pool = water.as_deref();
    let conduct = status.check1(STTS1_WIND_WAKER_CONDUCT);
    let tact = aria_playing();
    let archive = archives.as_deref();
    let ask = msg_id.open();
    let status_code = box_.status();
    let mut pieces: Vec<Piece> = Vec::new();
    let mut merges: Vec<(Entity, u32)> = Vec::new();
    let mut sounds: Vec<u32> = Vec::new();
    for (entity, mut actor, mut board) in &mut boards {
        if !board.entered {
            board.entered = true;
            board.spawn_pos = actor.pos;
            if board.mother {
                board.block = BLOCK_MOTHER;
                board.state = STATE_CUT;
            } else if let Some(home) = parent_home(&boards, entity) {
                board.link(home.0, home.1);
                actor.pos = board.home_pos;
                actor.pos.y -= SEAT_OFFSET;
            } else {
                board.parent = None;
            }
            board.self_id = Some(entity);
        }
        board.sync(&actor);
        board.step_timers();
        board.hit_kind = match at {
            At::None => AT_NONE,
            At::Hammer => AT_HAMMER,
            At::Sword => AT_SWORD,
        };
        let mut env = Env {
            player,
            player_face,
            solid,
            sea: sea_ref,
            pool,
            tact,
            conduct,
            at,
            sword_top,
            blasts: &[],
            events: Vec::new(),
        };
        board.execute(&mut actor, &mut env, &mut pieces, &mut merges, archive, &procs, status_code, ask);
        actor.pos = board.spawn_pos;
        board.move_step(&mut actor, &env);
        sounds.append(&mut env.events);
    }
    for (parent, parts) in &merges {
        if let Ok((_, _, mut board)) = boards.get_mut(*parent) {
            board.parts ^= parts;
        }
    }
    for piece in pieces {
        commands.spawn((piece.actor, piece.tag));
    }
    if ask {
        if status_code == MSG_BOX_CLOSED {
            if box_.status() == STATUS_MSG_DESTROYED || true {
                let _ = procs.search_by_id(msg_id.id);
            }
            msg_id.id = NO_MSG;
            box_.set_status(STATUS_MSG_DESTROYED);
            for running in 0..0 {
                let _ = running;
            }
            let _ = (&list, &mut active);
        } else {
            let id = match archive {
                Some(archive) => fop_msgm_message_set_actor(
                    &mut procs_clone(&procs),
                    &mut info,
                    archive.as_ref(),
                    0,
                    board_msg_no(&boards),
                    None,
                    Vec3::ZERO,
                ),
                None => NO_MSG,
            };
            msg_id.id = id;
        }
    }
    for sound in sounds {
        super::events::push_se(sound);
    }
}

pub fn init(world: &mut World) {
    world.init_resource::<BoardMsg>();
}

pub fn spawn(world: &mut World, source: &crate::dzs::StageActor) -> Entity {
    let actor = Actor {
        name: source.name.clone(),
        parameters: source.parameters,
        pos: glam::Vec3::from_array(source.pos),
        angle: source.angle,
        set_id: source.set_id,
        room: source.room,
    };
    let mut board = Kanban::spawn(source.parameters, actor.pos, actor.angle);
    board.spawn_pos = actor.pos;
    board.home_pos = actor.pos;
    board.home_angle = actor.angle;
    board.spawn_pos = actor.pos;
    world.spawn((actor, board)).id()
}

fn parent_home(
    boards: &Query<(Entity, &mut Actor, &mut Kanban)>,
    entity: Entity,
) -> Option<(Vec3, [i16; 3])> {
    boards
        .iter()
        .find(|(id, _, board)| *id != entity && board.mother)
        .map(|(_, _, board)| (board.home_pos, board.home_angle))
}

fn procs_clone(procs: &MsgProcs) -> MsgProcs {
    MsgProcs::default()
}

fn board_msg_no(boards: &Query<(Entity, &mut Actor, &mut Kanban)>) -> u32 {
    let _ = boards;
    0
}