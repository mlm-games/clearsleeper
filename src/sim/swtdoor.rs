use bevy_ecs::prelude::*;

use super::flags::Switches;

pub const SWTDOOR_ACTORS: [&str; 1] = ["SWTdoor"];

pub const BYTE_MASK: u32 = 0xFF; // <- d_a_swtdoor.cpp:63
pub const SWITCH_SHIFT: u32 = 24; // <- d_a_swtdoor.cpp:66
pub const NO_SWITCH: u8 = 0xFF; // <- d_a_swtdoor.cpp:64
pub const SINK_STEP: f32 = -10.0; // <- d_a_swtdoor.cpp:17
pub const SINK_LIMIT: f32 = -300.0; // <- d_a_swtdoor.cpp:16
pub const CULL_MIN: [f32; 3] = [-2000.0, -1000.0, -2000.0]; // <- d_a_swtdoor.cpp:73
pub const CULL_MAX: [f32; 3] = [2000.0, 1000.0, 2000.0]; // <- d_a_swtdoor.cpp:74

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Swtdoor {
    pub switch_no: u8,
    pub sink: f32,
}

impl Swtdoor {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            switch_no: ((parameters >> SWITCH_SHIFT) & BYTE_MASK) as u8,
            sink: 0.0,
        }
    }

    pub fn step(&mut self, switches: &Switches) {
        if switches.test(self.switch_no) && self.sink > SINK_LIMIT {
            self.sink += SINK_STEP;
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_swtdoor(name: &str) -> bool {
    SWTDOOR_ACTORS.contains(&name)
}

pub fn control(switches: Res<Switches>, mut doors: Query<&mut Swtdoor>) {
    for mut door in &mut doors {
        door.step(&switches);
    }
}
