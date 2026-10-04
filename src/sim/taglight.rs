use bevy_ecs::prelude::*;

use super::flags::Switches;
use super::schedule::Schedule;
use super::walk;

pub const TAG_LIGHT_ACTORS: [&str; 3] = ["LTag0", "LTag1", "LTagR0"];

pub const NO_SWITCH: u8 = 0xFF;

pub const TYPE_SPOT: u8 = 1;
pub const TYPE_SENSOR: u8 = 2;

pub const OFF_LEVEL: f32 = 0.0;
pub const ON_LEVEL: f32 = 1.0;

pub const FADE_SPEED: [f32; 4] = [0.06666667, 0.033333335, 0.016666668, 1.0];

pub const CULL_HALF: f32 = 51.0;
pub const CULL_TOP: f32 = 101.0;

const TYPE_SHIFT: u32 = 0;
const TYPE_MASK: u32 = 0x3;
const SWITCH2_SHIFT: u32 = 2;
const FADE_SHIFT: u32 = 14;
const FADE_MASK: u32 = 0x3;
const SWITCH_SHIFT: u32 = 16;
const SCHEDULE_SHIFT: u32 = 24;
const FIELD_MASK: u32 = 0xFF;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct TagLight {
    pub kind: u8,
    pub switch_no: u8,
    pub switch2_no: u8,
    pub mask: u8,
    pub fade: u8,
    pub lit: bool,
    pub level: f32,
    pub started: bool,
}

impl TagLight {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            kind: ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8,
            switch_no: ((parameters >> SWITCH_SHIFT) & FIELD_MASK) as u8,
            switch2_no: ((parameters >> SWITCH2_SHIFT) & FIELD_MASK) as u8,
            mask: ((parameters >> SCHEDULE_SHIFT) & FIELD_MASK) as u8,
            fade: ((parameters >> FADE_SHIFT) & FADE_MASK) as u8,
            lit: false,
            level: OFF_LEVEL,
            started: false,
        }
    }

    pub fn sensor(&self) -> bool {
        self.kind == TYPE_SENSOR
    }

    pub fn fade_speed(&self) -> f32 {
        FADE_SPEED[usize::from(self.fade) % FADE_SPEED.len()]
    }

    pub fn blocked(&self, switches: &Switches) -> bool {
        self.switch2_no != NO_SWITCH && switches.test(self.switch2_no)
    }

    pub fn switched(&self, switches: &Switches) -> bool {
        (self.switch_no == NO_SWITCH || switches.test(self.switch_no)) && !self.blocked(switches)
    }

    pub fn scheduled(&self, schedule: &Schedule) -> bool {
        (self.mask & schedule.bit) != 0 || self.mask == 0
    }

    pub fn step(&mut self, switches: &Switches, schedule: &Schedule) {
        self.lit = self.switched(switches) && self.scheduled(schedule);
        if !self.started {
            self.started = true;
            self.level = if self.lit { ON_LEVEL } else { OFF_LEVEL };
        }
        let target = if self.lit { ON_LEVEL } else { OFF_LEVEL };
        walk::chase_f(&mut self.level, target, self.fade_speed());
    }

    pub fn drawn(&self) -> bool {
        self.kind == TYPE_SPOT && self.level > OFF_LEVEL
    }

    pub fn cull(&self) -> (f32, f32) {
        (CULL_HALF, CULL_TOP)
    }
}

pub fn is_tag_light(name: &str) -> bool {
    TAG_LIGHT_ACTORS.contains(&name)
}

pub fn control(switches: Res<Switches>, schedule: Res<Schedule>, mut lights: Query<&mut TagLight>) {
    for mut light in &mut lights {
        if light.sensor() {
            continue;
        }
        light.step(&switches, &schedule);
    }
}