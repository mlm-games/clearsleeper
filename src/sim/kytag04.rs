use bevy_ecs::prelude::*;

use super::flags::Switches;

pub const KYTAG04_ACTORS: [&str; 1] = ["kytag4"];

pub const NO_SWITCH: u8 = 0xFF; // <- d_a_kytag04.cpp:97
pub const TIMER_THRESHOLD: i32 = 5; // <- d_a_kytag04.cpp:103
pub const STATE_IDLE: u8 = 0; // <- d_a_kytag04.cpp:99
pub const STATE_LIT: u8 = 1; // <- d_a_kytag04.cpp:88

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kytag04 {
    pub state: u8,
    pub switch_no: u8,
    pub timer: i32,
    pub timer_threshold: i32,
}

impl Kytag04 {
    pub fn spawn(angle: [i16; 3]) -> Self {
        Self {
            state: STATE_IDLE,
            switch_no: angle[0] as u8,
            timer: 0,
            timer_threshold: TIMER_THRESHOLD,
        }
    }

    fn lit(&self, switches: &Switches) -> bool {
        self.switch_no != NO_SWITCH && switches.test(self.switch_no)
    }

    pub fn step(&mut self, switches: &Switches) {
        if self.lit(switches) {
            if self.state == STATE_IDLE {
                if self.timer >= self.timer_threshold {
                    self.timer = 0;
                    self.state = STATE_LIT;
                } else {
                    self.timer += 1;
                }
            }
        } else if self.state == STATE_LIT {
            self.state = STATE_IDLE;
        }
    }
}

pub fn is_kytag04(name: &str) -> bool {
    KYTAG04_ACTORS.contains(&name)
}

pub fn control(switches: Res<Switches>, mut tags: Query<&mut Kytag04>) {
    for mut tag in &mut tags {
        tag.step(&switches);
    }
}
