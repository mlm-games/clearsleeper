use bevy_ecs::prelude::*;

use super::flags::Switches;

pub const WALL_ACTOR: &str = "Wall";

pub const NO_SWITCH: u8 = 0xFF;
pub const FADE_STEP: f32 = 2.2222223;
pub const FADE_FRAMES: u32 = 10;
pub const TRI_COUNT: usize = 2;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Wait,
    Break,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Wall {
    pub kind: u8,
    pub switch_no: u8,
    pub state: State,
    pub frames: u32,
    pub fade: f32,
}

impl Wall {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: ((parameters >> 8) & 0xFF) as u8,
            switch_no: (parameters & 0xFF) as u8,
            state: State::Wait,
            frames: 1,
            fade: 1.0,
        }
    }

    pub fn breaks(&self) -> bool {
        self.kind <= 2
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Stay,
    Cleared,
    Gone,
}

pub fn bomb_hit(wall: &mut Wall) -> Step {
    if wall.state != State::Wait {
        return Step::Stay;
    }
    wall.state = State::Break;
    wall.frames = 1;
    Step::Cleared
}

pub fn decay(wall: &mut Wall) -> Step {
    if wall.state != State::Break {
        return Step::Stay;
    }
    wall.frames += 1;
    if !wall.breaks() {
        return Step::Gone;
    }
    if wall.frames > FADE_FRAMES && chase(&mut wall.fade, 0.0, FADE_STEP) {
        return Step::Gone;
    }
    Step::Stay
}

fn chase(current: &mut f32, target: f32, step: f32) -> bool {
    let diff = target - *current;
    if diff.abs() <= step {
        *current = target;
        true
    } else {
        *current += diff.signum() * step;
        false
    }
}

pub fn already_broken(wall: &Wall, switches: &Switches) -> bool {
    wall.switch_no != NO_SWITCH && switches.test(wall.switch_no)
}