use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::arrow::{Arrow, CAPSULE_RADIUS, Phase};
use super::barricade::AT_FIRE_ARROW;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::stream;
use super::walk;

pub const HMLIF_ACTORS: [&str; 3] = ["Hmlif", "Hyuf1", "Hyuf2"];

pub const SPEEDS: [f32; 16] = [
    1.6666666, 3.3333333, 5.0, 6.6666665, 8.333333, 10.0, 11.666667, 13.333333, 15.0, 16.666666,
    18.333334, 20.0, 21.666666, 23.333334, 25.0, 26.666666,
];
pub const SPEED_TOP: u8 = 8;

pub const KIND_PLAIN: u8 = 0;
pub const KIND_RAISE: u8 = 1;
pub const KIND_LOWER: u8 = 2;

pub const CULL_BOX: [[f32; 6]; 3] = [
    [-200.0, -100.0, -200.0, 200.0, 50.0, 200.0],
    [-200.0, -100.0, -200.0, 200.0, 450.0, 200.0],
    [-200.0, -100.0, -200.0, 200.0, 450.0, 200.0],
];

pub const SPH_RADIUS: f32 = 35.0;
pub const SPH_RAISE_Y: f32 = 240.0;
pub const SPH_LOWER_Y: f32 = -250.0;

pub const AT_NORMAL_ARROW: u32 = 1 << 14;
pub const AT_ICE_ARROW: u32 = 1 << 19;
pub const AT_LIGHT_ARROW: u32 = 1 << 20;
pub const ARROW_AT: u32 = AT_NORMAL_ARROW | AT_FIRE_ARROW | AT_ICE_ARROW | AT_LIGHT_ARROW;

pub const SE_FLOAT_LIFT_MOVE: u32 = 0x3025;
pub const SE_ARROW_SW_ON: u32 = 0x695E;

pub const PHASE_GO: i8 = 0;
pub const PHASE_RAMP_UP: i8 = 1;
pub const PHASE_HOLD: i8 = 2;
pub const PHASE_RAMP_DOWN: i8 = 3;

pub const TURN_SCALE: i16 = 10;
pub const TURN_MAX_STEP: i16 = 0x400;
pub const SMOOTH_SCALE: f32 = 0.4;
pub const SMOOTH_MAX_STEP: f32 = 1.0;
pub const SMOOTH_MIN_STEP: f32 = 0.0;

pub const RAMP_DIVISOR: f32 = 4.0;
pub const RAMP_SCALE: f32 = 1.0 / RAMP_DIVISOR;
pub const RAMP_MIN_STEP: f32 = 1.0;
pub const RAMP_STEP_DIVISOR: f32 = 6.6666665;
pub const APPROACH_DIVISOR: f32 = 2.2;
pub const APPROACH_RADIUS: f32 = 50.0;
pub const APPROACH_SCALE: f32 = 1.0;

const SPEED_SHIFT: u32 = 16;
const SPEED_MASK: u32 = 0xF;
const SMOOTH_SHIFT: u32 = 31;
const SMOOTH_MASK: u32 = 0x1;
const TYPE_SHIFT: u32 = 27;
const TYPE_MASK: u32 = 0xF;
const ANGLE_Z_MASK: i16 = 0xFF;

pub const ARROW_KIND_AT: [u32; 4] = [AT_NORMAL_ARROW, AT_FIRE_ARROW, AT_ICE_ARROW, AT_LIGHT_ARROW];

fn add_calc_pos2(value: &mut Vec3, target: Vec3, scale: f32, max_step: f32) {
    if *value != target {
        let mut diff = (*value - target) * scale;
        if diff.length() > max_step {
            diff = diff.normalize_or_zero() * max_step;
        }
        *value -= diff;
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Hmlif {
    pub kind: u8,
    pub arrow_switch: u8,
    pub speed: f32,
    pub speed_f: f32,
    pub smooth: bool,
    pub phase: i8,
    pub timer: i32,
    pub target: Option<Vec3>,
    pub prev_target: Option<Vec3>,
}

impl Hmlif {
    pub fn spawn(parameters: u32, angle_z: i16) -> Self {
        let index = ((parameters >> SPEED_SHIFT) & SPEED_MASK) as u8;
        let speed = SPEEDS[usize::from(index.min(SPEED_TOP))];
        Self {
            kind: ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8,
            arrow_switch: (angle_z & ANGLE_Z_MASK) as u8,
            speed,
            speed_f: speed,
            smooth: (parameters >> SMOOTH_SHIFT) & SMOOTH_MASK != 0,
            phase: PHASE_GO,
            timer: 0,
            target: None,
            prev_target: None,
        }
    }

    pub fn cull(&self) -> (Vec3, Vec3) {
        let row = CULL_BOX
            .get(usize::from(self.kind))
            .copied()
            .unwrap_or(CULL_BOX[0]);
        (
            Vec3::new(row[0], row[1], row[2]),
            Vec3::new(row[3], row[4], row[5]),
        )
    }

    fn sphere(&self, pos: Vec3) -> Option<Vec3> {
        if self.kind == KIND_PLAIN {
            return None;
        }
        match self.kind {
            KIND_RAISE => Some(pos + Vec3::new(0.0, SPH_RAISE_Y, 0.0)),
            KIND_LOWER => Some(pos + Vec3::new(0.0, SPH_LOWER_Y, 0.0)),
            _ => None,
        }
    }

    fn struck(&self, centre: Vec3, shaft: &Arrow) -> bool {
        if shaft.phase != Phase::Flying {
            return false;
        }
        let Some(at) = ARROW_KIND_AT.get(usize::from(shaft.kind)).copied() else {
            return false;
        };
        if at & ARROW_AT == 0 {
            return false;
        }
        let travel = shaft.pos - shaft.speed;
        walk::swept(travel, shaft.pos, centre, SPH_RADIUS + CAPSULE_RADIUS)
    }

    fn check_col(
        &mut self,
        pos: Vec3,
        arrows: &Query<&Arrow>,
        switches: &mut Switches,
        events: &mut SimEvents,
    ) {
        let Some(centre) = self.sphere(pos) else {
            return;
        };
        if !arrows.iter().any(|shaft| self.struck(centre, shaft)) {
            return;
        }
        if !switches.test(self.arrow_switch) {
            events.list.push(SimEvent::Se(SE_ARROW_SW_ON));
        }
        switches.set(self.arrow_switch);
    }

    fn lift_smooth_move(&mut self, actor: &mut Actor, target: Vec3, prev: Vec3) {
        let heading = walk::to_s16((target.x - prev.x).atan2(target.z - prev.z));
        let speed = self.speed_f;
        stream::add_calc_angle_s2(&mut actor.angle[1], heading, TURN_SCALE, TURN_MAX_STEP);
        walk::add_calc(
            &mut self.speed_f,
            self.speed,
            SMOOTH_SCALE,
            SMOOTH_MAX_STEP,
            SMOOTH_MIN_STEP,
        );
        let facing = walk::from_s16(actor.angle[1]);
        actor.pos.x += facing.sin() * speed;
        actor.pos.z += facing.cos() * speed;
    }

    fn ramp_speed(&mut self, target: f32) -> bool {
        walk::add_calc(
            &mut self.speed_f,
            target,
            RAMP_SCALE,
            self.speed / RAMP_STEP_DIVISOR,
            RAMP_MIN_STEP,
        );
        self.speed_f == target
    }

    fn lift_normal_move(&mut self, actor: &mut Actor, target: Vec3) {
        if self.phase == PHASE_GO {
            self.phase = PHASE_RAMP_UP;
            self.timer = 0;
        }
        match self.phase {
            PHASE_RAMP_UP => {
                self.timer += 1;
                if self.ramp_speed(self.speed) {
                    self.phase = PHASE_HOLD;
                }
            }
            PHASE_HOLD => {
                self.timer = 0;
                if (actor.pos - target).length() < APPROACH_RADIUS {
                    self.phase = PHASE_RAMP_DOWN;
                }
            }
            PHASE_RAMP_DOWN => {
                self.timer += 1;
                let idle = self.speed / APPROACH_DIVISOR;
                if self.ramp_speed(idle) {
                    self.phase = PHASE_GO;
                }
            }
            _ => {}
        }
        add_calc_pos2(&mut actor.pos, target, APPROACH_SCALE, self.speed_f);
    }

    fn move_proc(&mut self, actor: &mut Actor) {
        let (Some(target), Some(prev)) = (self.target, self.prev_target) else {
            return;
        };
        if self.smooth {
            self.lift_smooth_move(actor, target, prev);
        } else {
            self.lift_normal_move(actor, target);
        }
    }

    fn execute(
        &mut self,
        actor: &mut Actor,
        arrows: &Query<&Arrow>,
        switches: &mut Switches,
        events: &mut SimEvents,
    ) {
        let before = actor.pos;
        self.move_proc(actor);
        if actor.pos != before {
            events.list.push(SimEvent::Se(SE_FLOAT_LIFT_MOVE));
        }
        self.check_col(actor.pos, arrows, switches, events);
    }
}

pub fn is_hmlif(name: &str) -> bool {
    HMLIF_ACTORS.contains(&name)
}

pub fn control(
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    arrows: Query<&Arrow>,
    mut lifts: Query<(&mut Actor, &mut Hmlif)>,
) {
    for (mut actor, mut lift) in &mut lifts {
        lift.execute(&mut *actor, &arrows, &mut switches, &mut events);
    }
}
