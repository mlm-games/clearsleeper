use bevy_ecs::prelude::*;

use glam::Vec3;

use super::actor::Actor;
use super::player::Player;

pub const TRAPDOOR_ACTOR: &str = "Nzfall";

pub const SEA_STAGE: &str = "sea";
pub const SEA_START: u8 = 15;
pub const WINDFALL_ROOM: u8 = 11;

pub const TRIGGER_RADIUS: f32 = 100.0;
pub const TRIGGER_HEIGHT: f32 = 100.0;
pub const STILL_SPEED: f32 = 0.0;
pub const HOLD_FRAMES: u8 = 0;
pub const STEPS: usize = 7;
pub const SWING: [i16; STEPS] = [
    0xC568, 0xD508, 0xC950, 0xD120, 0xCD38, 0xCF2C, 0xCE32,
];
pub const TURN_SCALE: i16 = 4;
pub const TURN_MAX_STEP: i16 = 0x2000;
pub const TURN_MIN_STEP: i16 = 0x100;
pub const TURN_TOLERANCE: i16 = 0x100;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Wait,
    Event,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hinge {
    Shut,
    Swinging,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Trapdoor {
    pub mode: Mode,
    pub hinge: Hinge,
    pub step: usize,
    pub swing: i16,
    pub timer: u8,
    pub ordered: bool,
}

impl Trapdoor {
    pub fn spawn() -> Self {
        Self {
            mode: Mode::Wait,
            hinge: Hinge::Shut,
            step: 0,
            swing: 0,
            timer: HOLD_FRAMES,
            ordered: false,
        }
    }

    pub fn near(player: &Player, pos: Vec3) -> bool {
        let dx = player.pos.x - pos.x;
        let dz = player.pos.z - pos.z;
        let dy = player.pos.y - pos.y;
        dx * dx + dz * dz <= TRIGGER_RADIUS * TRIGGER_RADIUS
            && dy.abs() <= TRIGGER_HEIGHT
            && player.speed.abs() <= STILL_SPEED
    }

    pub fn trigger(&mut self) {
        self.mode = Mode::Event;
    }

    pub fn arm(&mut self) {
        self.mode = Mode::Wait;
        self.timer = HOLD_FRAMES;
        self.ordered = false;
    }

    pub fn order(&mut self) {
        self.ordered = true;
    }

    pub fn open(&mut self) {
        self.hinge = Hinge::Swinging;
        self.step = 0;
    }

    pub fn close(&mut self) {
        self.hinge = Hinge::Shut;
    }

    pub fn swing_to(&mut self) -> bool {
        if self.hinge != Hinge::Swinging {
            return true;
        }
        if self.step >= STEPS {
            self.close();
            return true;
        }
        let target = SWING[self.step];
        let step = chase(&mut self.swing, target, TURN_SCALE, TURN_MAX_STEP, TURN_MIN_STEP);
        if step.abs() <= TURN_TOLERANCE {
            self.step += 1;
        }
        false
    }

    pub fn dropped(&self) -> Option<(String, u8, u8)> {
        if self.mode != Mode::Event || self.hinge == Hinge::Swinging || !self.ordered {
            return None;
        }
        Some((SEA_STAGE.to_owned(), SEA_START, WINDFALL_ROOM))
    }
}

pub fn chase(value: &mut i16, target: i16, scale: i16, max_step: i16, min_step: i16) -> i16 {
    let diff = target.wrapping_sub(*value);
    if *value == target {
        return 0;
    }
    let mut step = f32::from(diff) / f32::from(scale);
    if step > f32::from(min_step) || step < f32::from(-min_step) {
        step = step.clamp(f32::from(-max_step), f32::from(max_step));
        *value = value.wrapping_add(step as i16);
    } else if step > 0.0 {
        *value = value.wrapping_add(min_step);
    } else {
        *value = value.wrapping_sub(min_step);
    }
    value.wrapping_sub(target)
}

pub fn is_trapdoor(name: &str) -> bool {
    name == TRAPDOOR_ACTOR
}

pub fn control(
    mut next: ResMut<super::transition::NextStage>,
    mut doors: Query<(Entity, &Actor, &mut Trapdoor)>,
    players: Query<&Player>,
    mut commands: Commands,
) {
    let Some(player) = players.iter().next() else {
        return;
    };
    for (entity, actor, mut door) in &mut doors {
        match door.mode {
            Mode::Wait => {
                door.timer = door.timer.saturating_sub(1);
                if door.timer == 0 && Trapdoor::near(player, actor.pos) {
                    door.trigger();
                }
            }
            Mode::Event => {
                door.open();
                if door.swing_to() {
                    door.order();
                }
            }
        }
        if let Some((stage, start, room)) = door.dropped() {
            next.exit = Some(start);
            next.target = Some(super::transition::Target {
                stage,
                room,
                start,
            });
            let _ = commands.entity(entity).despawn();
        }
    }
}