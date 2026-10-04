use bevy_ecs::prelude::*;

use glam::Vec3;

use super::actor::Actor;
use super::collision::Collision;
use super::flags::Switches;
use super::ground::surface;

pub const LADDER_ACTORS: [&str; 2] = ["Mhsg15", "Mhsg4H"];

pub const NO_SWITCH: u8 = 0xFF;

pub const VIB_FRAMES: i16 = 0x000F;
pub const VIB_RATE_X: i16 = 20000;
pub const VIB_RATE_Y: i16 = 0x3CC3;
pub const VIB_AMPLITUDE_X: f32 = 2.0;
pub const VIB_AMPLITUDE_Y: f32 = 1.0;
pub const VIB_DAMP: f32 = 0.995;
pub const DROP_GRAVITY: f32 = -5.0;
pub const BOUNCES: i16 = 3;
pub const RESTITUTION: f32 = -0.5;
pub const OFFSET_SCALE: f32 = 0.5;
pub const CULL_WIDTH: f32 = 55.0;
pub const CULL_HEIGHT: f32 = 41.0;
pub const CULL_DEPTH: f32 = 10.0;

pub const TYPE_HEIGHTS: [f32; 5] = [600.0, 900.0, 1200.0, 1500.0, 450.0];

pub const TYPE_MHSG6: u8 = 0;
pub const TYPE_MHSG9: u8 = 1;
pub const TYPE_MHSG12: u8 = 2;
pub const TYPE_MHSG15: u8 = 3;
pub const TYPE_MHSG4H: u8 = 4;

const TYPE_MASK: u32 = 0x03;
const SWITCH_SHIFT: u32 = 8;
const EVENT_SHIFT: u32 = 16;
const FIELD_MASK: u32 = 0xFF;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Wait,
    DemoReq,
    Vib,
    Drop,
    Fell,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Ladder {
    pub kind: u8,
    pub switch_no: u8,
    pub event_id: u8,
    pub mode: Mode,
    pub frames: i16,
    pub bounces: i16,
    pub phase_x: i16,
    pub phase_y: i16,
    pub offset: Vec3,
    pub fall: f32,
}

impl Ladder {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: (parameters & TYPE_MASK) as u8,
            switch_no: ((parameters >> SWITCH_SHIFT) & FIELD_MASK) as u8,
            event_id: ((parameters >> EVENT_SHIFT) & FIELD_MASK) as u8,
            mode: Mode::Wait,
            frames: 0,
            bounces: 0,
            phase_x: 0,
            phase_y: 0,
            offset: Vec3::ZERO,
            fall: 0.0,
        }
    }

    pub fn height(&self) -> f32 {
        TYPE_HEIGHTS[usize::from(self.kind) % TYPE_HEIGHTS.len()]
    }

    pub fn released(&self, switches: &Switches) -> bool {
        self.switch_no == NO_SWITCH || switches.test(self.switch_no)
    }
}

pub fn is_ladder(name: &str) -> bool {
    LADDER_ACTORS.contains(&name)
}

pub fn control(
    collision: Option<Res<Collision>>,
    switches: Res<Switches>,
    mut ladders: Query<(&mut Actor, &mut Ladder)>,
) {
    let solid = collision.as_deref();
    for (mut actor, mut ladder) in &mut ladders {
        match ladder.mode {
            Mode::Wait => {
                if ladder.released(&switches) {
                    ladder.mode = Mode::DemoReq;
                }
            }
            Mode::DemoReq => {
                ladder.mode = Mode::Vib;
                ladder.frames = VIB_FRAMES;
            }
            Mode::Vib => shake(&mut ladder),
            Mode::Drop => drop(&mut actor, &mut ladder, solid),
            Mode::Fell => {}
        }
    }
}

fn shake(ladder: &mut Ladder) {
    ladder.phase_x = ladder.phase_x.wrapping_add(VIB_RATE_X);
    ladder.phase_y = ladder.phase_y.wrapping_add(VIB_RATE_Y);
    ladder.offset.x = cosine(VIB_RATE_X, ladder.phase_x) * VIB_AMPLITUDE_X;
    ladder.offset.y = cosine(VIB_RATE_Y, ladder.phase_y) * VIB_AMPLITUDE_Y;
    ladder.frames -= 1;
    if ladder.frames <= 0 {
        ladder.mode = Mode::Drop;
        ladder.bounces = BOUNCES;
        ladder.fall = 0.0;
    }
}

fn drop(actor: &mut Actor, ladder: &mut Ladder, solid: Option<&Collision>) {
    ladder.fall += DROP_GRAVITY;
    actor.pos.y += ladder.fall;
    ladder.fall *= VIB_DAMP;
    let Some(floor) = surface(solid, actor.pos) else {
        return;
    };
    if actor.pos.y >= floor {
        return;
    }
    if ladder.bounces < 0 {
        actor.pos.y = floor;
        ladder.mode = Mode::Fell;
        ladder.offset = Vec3::ZERO;
        return;
    }
    ladder.bounces -= 1;
    let overshoot = actor.pos.y - floor;
    actor.pos.y = floor - overshoot * OFFSET_SCALE;
    ladder.fall *= RESTITUTION;
}

fn cosine(rate: i16, phase: i16) -> f32 {
    let degrees = f32::from(rate) * f32::from(phase);
    let mut wrapped = degrees % 360.0;
    if wrapped < 0.0 {
        wrapped += 360.0;
    }
    wrapped.to_radians().cos()
}