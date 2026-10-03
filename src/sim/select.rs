use bevy_ecs::prelude::*;
use glam::Vec2;

use super::intent::{DoIntent, MoveIntent};

pub const CHOICES: u8 = 2;
const AXIS_STEP: f32 = 0.5;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Select {
    pub open: bool,
    pub cursor: u8,
    picked: u8,
}

impl Select {
    pub fn begin(&mut self) {
        self.open = true;
        self.cursor = 0;
    }

    pub fn accepted(&self) -> bool {
        !self.open && self.picked == 0
    }

    fn slide(&mut self, axis: f32) {
        if axis > AXIS_STEP && self.cursor + 1 < CHOICES {
            self.cursor += 1;
        } else if axis < -AXIS_STEP && self.cursor > 0 {
            self.cursor -= 1;
        }
    }
}

pub fn control(intent: Res<MoveIntent>, do_intent: Res<DoIntent>, mut select: ResMut<Select>) {
    if !select.open {
        return;
    }
    select.slide(intent.axis.x);
    if do_intent.0 {
        select.picked = select.cursor;
        select.open = false;
    }
}