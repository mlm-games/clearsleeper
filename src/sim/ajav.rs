use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::bomb::Bomb;
use super::bombflower::Crawler;
use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::{StoryBits, Switches};
use super::walk;

pub const AJAV_ACTORS: [&str; 1] = ["Ajav"];

pub const EVENT_ENDLESS_NIGHT: u16 = 0x0A02;

pub const STATUS_MAX: u8 = 4;
pub const PARTS: usize = 6;
pub const HINTS: usize = 2;

pub const SPH_RADIUS: f32 = 280.0;
pub const CYL_RADIUS: f32 = 780.0;
pub const CYL_DROP: f32 = -40.0;
pub const BOMB_RADIUS: f32 = 30.0;

pub const CO_OFFSET: [[f32; 3]; 3] = [
    [0.0, 2550.0, 200.0],
    [250.0, 1270.0, 200.0],
    [-290.0, 740.0, 200.0],
];
pub const PART_OFFSET: [[f32; 3]; PARTS] = [
    [0.0, 3000.0, 0.0],
    [500.0, 2400.0, 0.0],
    [-300.0, 1900.0, 0.0],
    [500.0, 1400.0, 0.0],
    [-550.0, 650.0, 0.0],
    [250.0, 400.0, 0.0],
];
pub const HINT_OFFSET: [[f32; 3]; PARTS] = [
    [33.0, 2541.0, 220.0],
    [465.0, 2083.0, 220.0],
    [-137.7, 1297.0, 220.0],
    [530.0, 511.0, 220.0],
    [-124.5, 118.0, 220.0],
    [530.0, 118.0, 220.0],
];
pub const HINT_RADIUS: [f32; PARTS] = [
    629.0, 294.8, 491.0, 216.0, 563.0, 85.3,
];
pub const HINT_HEIGHT: [f32; PARTS] = [
    773.0, 458.5, 1100.5, 1401.9, 1133.0, 451.9,
];
pub const WALL_HEIGHT: [f32; 3] = [3300.0, 2360.0, 1050.0];
pub const FLAW_OFFSET: [[f32; 3]; PARTS] = [
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 40.0, 0.0],
    [30.0, 30.0, 0.0],
    [-20.0, 0.0, 0.0],
    [20.0, 0.0, 0.0],
];
pub const FALL_RATE: [f32; 3] = [12.0, -10.0, 55.0];

pub const BROKEN_EVENTS: [&str; 3] = ["ajav_destroy0", "ajav_destroy1", "ajav_uzu"];

pub const STEP_IDLE: u8 = 0;
pub const STEP_BROKEN: u8 = 1;
pub const STEP_SETTLE: u8 = 2;
pub const STEP_STOW: u8 = 3;
pub const STEP_DONE: u8 = 4;

pub const SE_STONE_BRK: u32 = 0x6A28;
pub const SE_STONE_FALL: u32 = 0x6A29;
pub const SE_RIDDLE: u32 = 0x0806;

const ANGLE_LIMIT: i16 = 0x1FF;
const HINT_HITS: u8 = 3;
const CUTSCENE_DELAY: u16 = 60;
const SPLASH_DROP: f32 = 100.0;
const REST_DROP: f32 = 1000.0;
const FALL_ACCEL: f32 = 2.0;
const RATE_BASE: f32 = 0.8;
const RATE_SPREAD: f32 = 0.2;
const RATE_SLOW: f32 = 0.4;
const RATE_LIFT: f32 = 5.0;
const SPIN_SLOW: f32 = 0.3;
const SPIN_SPAN: f32 = 511.0;
const TRIGGER_SPREAD: f32 = 9.0;
const TRIGGER_FLOOR: i16 = 7;
const SWITCH_MASK: u32 = 0xFF;
const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA];
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673];
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0];
const RND_SEED: i32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proc {
    Idle,
    Fall0,
    Fall1,
    Flaw,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Part {
    pub centre: Vec3,
    pub to_origin: Vec3,
    pub shift: Vec3,
    pub rate: Vec3,
    pub fault: Vec3,
    pub spin: [i16; 3],
    pub rot: [i16; 3],
    pub timer: u16,
    pub trigger: u16,
    pub limit: i16,
    pub splashed: bool,
    pub proc: Proc,
}

impl Part {
    fn seated(index: usize) -> Self {
        let centre = xyz(PART_OFFSET.get(index)).unwrap_or(Vec3::ZERO);
        Self {
            centre,
            to_origin: -centre,
            shift: Vec3::ZERO,
            rate: Vec3::ZERO,
            fault: Vec3::ZERO,
            spin: [0; 3],
            rot: [0; 3],
            timer: 0,
            trigger: 0,
            limit: 0,
            splashed: false,
            proc: Proc::Idle,
        }
    }

    fn fall_init(&mut self, rate: Vec3, rot: [i16; 3], trigger: u16) {
        self.rate = rate;
        self.rot = rot;
        self.limit = ANGLE_LIMIT;
        self.trigger = trigger;
        self.timer = 0;
        self.splashed = false;
        self.proc = Proc::Fall0;
    }

    fn execute(&mut self, seed: &mut [i32; 3]) -> Option<u32> {
        match self.proc {
            Proc::Fall0 => {
                self.fall_0(seed);
                None
            }
            Proc::Fall1 => self.fall_1(),
            Proc::Flaw => {
                self.shift += self.fault;
                self.proc = Proc::Idle;
                None
            }
            Proc::Idle => None,
        }
    }

    fn fall_0(&mut self, seed: &mut [i32; 3]) {
        for axis in 0..3 {
            self.spin[axis] = self.spin[axis].wrapping_add(self.rot[axis]);
            if bounce(&mut self.spin[axis], self.limit) {
                self.rot[axis] = self.rot[axis].wrapping_neg();
            }
        }
        self.shift += self.rate;
        self.rate.z *= RATE_BASE + RATE_SPREAD * rnd(seed);
        self.timer = self.timer.wrapping_add(1);
        if self.timer == self.trigger {
            let mut temp = self.rate * RATE_SLOW;
            temp.y = RATE_LIFT;
            self.rate = temp;
            self.proc = Proc::Fall1;
            self.timer = 0;
            for axis in 0..3 {
                self.rot[axis] = ((self.rot[axis]) as f32 * SPIN_SLOW) as i16;
            }
            let swing = (rnd(seed) * -2.0) as i32;
            let turn = (rnd(seed) * SPIN_SPAN) as i16;
            self.rot[2] = (swing * i32::from(turn)) as i16;
        }
    }

    fn fall_1(&mut self) -> Option<u32> {
        self.shift += self.rate;
        self.shift.y -= (self.timer) as f32 * FALL_ACCEL;
        for axis in 0..3 {
            self.spin[axis] = self.spin[axis].wrapping_add(self.rot[axis]);
        }
        self.timer = self.timer.wrapping_add(1);
        let mut sound = None;
        if !self.splashed {
            if self.shift.y <= self.to_origin.y - SPLASH_DROP {
                self.splashed = true;
                sound = Some(SE_STONE_FALL);
            }
        }
        if self.shift.y <= self.to_origin.y - REST_DROP {
            self.proc = Proc::Idle;
        }
        sound
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ajav {
    pub switch_no: u8,
    pub status: u8,
    pub step: u8,
    pub hits: u8,
    pub delay: u16,
    pub cut: Option<usize>,
    pub seed: [i32; 3],
    pub parts: [Part; PARTS],
}

impl Ajav {
    pub fn spawn(parameters: u32, stories: &StoryBits, switches: &Switches) -> Option<Self> {
        let switch_no = (parameters & SWITCH_MASK) as u8;
        if !stories.test(EVENT_ENDLESS_NIGHT) || switches.test(switch_no) {
            return None;
        }
        let mut parts = [Part::seated(0); PARTS];
        for (index, part) in parts.iter_mut().enumerate() {
            *part = Part::seated(index);
        }
        Some(Self {
            switch_no,
            status: 0,
            step: STEP_IDLE,
            hits: 0,
            delay: 0,
            cut: None,
            seed: [RND_SEED; 3],
            parts,
        })
    }

    pub fn row(&self) -> usize {
        usize::from(self.status) << 1
    }

    pub fn standing(&self) -> bool {
        self.status < STATUS_MAX - 1
    }

    pub fn resting(&self) -> bool {
        self.parts.iter().all(|part| part.proc == Proc::Idle)
    }

    pub fn offset(&self) -> Vec3 {
        xyz(CO_OFFSET.get(usize::from(self.status))).unwrap_or(Vec3::ZERO)
    }

    pub fn hull(&self) -> (f32, f32, f32) {
        let height = WALL_HEIGHT
            .get(usize::from(self.status))
            .copied()
            .unwrap_or(WALL_HEIGHT[0]);
        (CYL_DROP, height, CYL_RADIUS)
    }

    pub fn solid(&self, pos: Vec3, at: Vec3) -> bool {
        self.standing()
            && walk::within(pos + self.offset(), at, SPH_RADIUS + BOMB_RADIUS)
    }

    pub fn hinted(&self, pos: Vec3, at: Vec3) -> bool {
        if !self.standing() {
            return false;
        }
        let row = self.row();
        for slot in 0..HINTS {
            let Some(index) = row.checked_add(slot) else {
                continue;
            };
            let (Some(centre), Some(radius), Some(height)) = (
                xyz(HINT_OFFSET.get(index)),
                HINT_RADIUS.get(index).copied(),
                HINT_HEIGHT.get(index).copied(),
            ) else {
                continue;
            };
            if inside_cyl(at, pos + centre, radius, height, BOMB_RADIUS) {
                return true;
            }
        }
        false
    }

    fn order(&self, list: &EventList, active: &mut ActiveEvent, entity: Entity) {
        let Some(cut) = self.cut else {
            return;
        };
        if active.stage_of(cut).is_none() {
            active.order(list, cut, entity);
        }
    }

    fn to_broken(&mut self, list: &EventList, active: &mut ActiveEvent, entity: Entity) {
        self.cut = BROKEN_EVENTS
            .get(usize::from(self.status))
            .copied()
            .and_then(|name| list.event_index(name));
        self.step = STEP_BROKEN;
        self.order(list, active, entity);
    }

    fn shatter(&mut self, events: &mut SimEvents) {
        let co = self.offset();
        let row = self.row();
        for slot in 0..HINTS {
            let index = row + slot;
            let Some(centre) = self.parts.get(index).map(|part| part.centre) else {
                continue;
            };
            let mut rate = Vec3::from_array(FALL_RATE);
            if (index & 1) == 0 {
                rate.x = -rate.x;
            }
            let rot = rot_speed(co, centre);
            let trigger = (rnd(&mut self.seed) * TRIGGER_SPREAD) as i16 + TRIGGER_FLOOR;
            let Some(part) = self.parts.get_mut(index) else {
                continue;
            };
            part.fall_init(rate, rot, trigger as u16);
            events.list.push(SimEvent::Se(SE_STONE_BRK));
        }
        self.status += 1;
        if self.standing() {
            self.crack();
            self.step = STEP_SETTLE;
        } else {
            self.step = STEP_STOW;
            self.delay = CUTSCENE_DELAY;
        }
    }

    fn crack(&mut self) {
        let row = self.row();
        for slot in 0..HINTS {
            let index = row + slot;
            let Some(fault) = xyz(FLAW_OFFSET.get(index)) else {
                continue;
            };
            let Some(part) = self.parts.get_mut(index) else {
                continue;
            };
            part.fault = fault;
            part.proc = Proc::Flaw;
        }
    }
}

pub fn is_ajav(name: &str) -> bool {
    AJAV_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut walls: Query<(Entity, &Actor, &mut Ajav), Without<Ajav>>,
    bombs: Query<&Actor, Or<(With<Bomb>, With<Crawler>)>>,
) {
    let shells: Vec<Vec3> = bombs.iter().map(|bomb| bomb.pos).collect();
    for (entity, actor, mut query) in &mut walls {
        let wall = &mut *query;
        let pos = actor.pos;
        match wall.step {
            STEP_IDLE => {
                if shells.iter().any(|at| wall.solid(pos, *at)) {
                    wall.to_broken(&list, &mut active, entity);
                } else if shells.iter().any(|at| wall.hinted(pos, *at)) {
                    wall.hits += 1;
                    if wall.hits >= HINT_HITS {
                        wall.hits = 0;
                        wall.to_broken(&list, &mut active, entity);
                    }
                }
            }
            STEP_BROKEN => {
                let running = wall.cut.is_some_and(|cut| active.start_check(cut));
                if running {
                    wall.shatter(&mut events);
                } else {
                    wall.order(&list, &mut active, entity);
                }
            }
            STEP_SETTLE => {
                let closed = wall.cut.is_some_and(|cut| active.end_check(cut));
                if closed {
                    if let Some(cut) = wall.cut {
                        active.end(cut);
                    }
                    wall.step = if wall.standing() { STEP_IDLE } else { STEP_DONE };
                }
            }
            STEP_STOW => {
                if wall.resting() {
                    if wall.delay == 0 {
                        switches.set(wall.switch_no);
                        events.list.push(SimEvent::Se(SE_RIDDLE));
                        wall.status += 1;
                        wall.step = STEP_SETTLE;
                    } else {
                        wall.delay -= 1;
                    }
                }
            }
            _ => {}
        }
        let seed = &mut wall.seed;
        for part in wall.parts.iter_mut() {
            if let Some(sound) = part.execute(seed) {
                events.list.push(SimEvent::Se(sound));
            }
        }
    }
}

fn xyz(value: Option<&[f32; 3]>) -> Option<Vec3> {
    value.map(|value| Vec3::from_array(*value))
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

fn inside_cyl(at: Vec3, centre: Vec3, radius: f32, height: f32, ball: f32) -> bool {
    let dx = at.x - centre.x;
    let dz = at.z - centre.z;
    let reach = radius + ball;
    dx * dx + dz * dz <= reach * reach
        && at.y + ball >= centre.y
        && at.y - ball <= centre.y + height
}

fn bounce(value: &mut i16, limit: i16) -> bool {
    let reach = i32::from(limit).unsigned_abs();
    let cur = i32::from(*value);
    if cur.unsigned_abs() < reach {
        return false;
    }
    let over = cur.unsigned_abs() - reach;
    *value = if cur > 0 {
        (reach - over) as i16
    } else {
        -((reach - over) as i16)
    };
    true
}

fn clamp_angle(value: &mut i16, limit: i16) {
    let reach = i32::from(limit);
    let cur = i32::from(*value);
    if cur > reach {
        *value = limit;
    } else if cur < -reach {
        *value = -limit;
    }
}

fn rot_speed(co: Vec3, centre: Vec3) -> [i16; 3] {
    let diff = co - centre;
    let mut out = [0_i16; 3];
    out[0] = diff.y as i16;
    out[1] = diff.x as i16;
    clamp_angle(&mut out[0], ANGLE_LIMIT);
    clamp_angle(&mut out[1], ANGLE_LIMIT);
    out
}