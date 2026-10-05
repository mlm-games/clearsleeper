use bevy_ecs::prelude::*;

use super::events::{SimEvent, SimEvents};
use super::flags::Switches;

pub const OBJTIME_ACTORS: [&str; 1] = ["ObjTime"];

pub const MODE_WAIT: u8 = 0; // <- d_a_obj_timer.h:13
pub const MODE_COUNT: u8 = 1; // <- d_a_obj_timer.h:14

pub const FRAMES_PER_SECOND: i32 = 30; // <- d_a_obj_timer.cpp:54
pub const AUDIBLE_SECONDS: i32 = 20; // <- d_a_obj_timer.cpp:56
pub const LONG_SECONDS: i32 = 10; // <- d_a_obj_timer.cpp:58
pub const MID_SECONDS: i32 = 5; // <- d_a_obj_timer.cpp:60
pub const FRAMES_PER_MINUTE: i32 = 15; // <- d_a_obj_timer.cpp:45

pub const SE_TIMER_20: u32 = 0x0861; // <- JAZelAudio_SE.h:106
pub const SE_TIMER_10: u32 = 0x0862; // <- JAZelAudio_SE.h:107
pub const SE_TIMER_5: u32 = 0x0863; // <- JAZelAudio_SE.h:108
pub const SE_TIMER_0: u32 = 0x0864; // <- JAZelAudio_SE.h:109

const TIME_SHIFT: u32 = 0x00; // <- d_a_obj_timer.h:19
const TIME_MASK: u32 = 0xFF; // <- d_a_obj_timer.h:18
const SWSAVE_SHIFT: u32 = 0x10; // <- d_a_obj_timer.h:22
const SWSAVE_MASK: u32 = 0xFF; // <- d_a_obj_timer.h:21

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Objtime {
    pub mode: u8,
    pub timer: i32,
    pub switch_no: u8,
    pub minutes: u8,
}

impl Objtime {
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let mut clock = Self {
            mode: MODE_WAIT,
            timer: 0,
            switch_no: ((parameters >> SWSAVE_SHIFT) & SWSAVE_MASK) as u8,
            minutes: ((parameters >> TIME_SHIFT) & TIME_MASK) as u8,
        };
        if switches.test(clock.switch_no) {
            clock.count_init();
        }
        clock
    }

    fn wait_init(&mut self) {
        self.timer = 0;
        self.mode = MODE_WAIT;
    }

    fn count_init(&mut self) {
        self.timer = FRAMES_PER_MINUTE * i32::from(self.minutes);
        self.mode = MODE_COUNT;
    }

    fn wait(&mut self, switches: &Switches) {
        if switches.test(self.switch_no) {
            self.count_init();
        }
    }

    fn count(&mut self, switches: &mut Switches, events: &mut SimEvents) {
        self.timer -= 1;
        if self.timer % FRAMES_PER_SECOND == 0 {
            let seconds_left = self.timer / FRAMES_PER_SECOND;
            if seconds_left <= AUDIBLE_SECONDS {
                events.list.push(SimEvent::Se(Self::chime(seconds_left)));
            }
        }
        if self.timer <= 0 || !switches.test(self.switch_no) {
            switches.clear(self.switch_no);
            self.wait_init();
        }
    }

    fn chime(seconds_left: i32) -> u32 {
        if seconds_left > LONG_SECONDS {
            SE_TIMER_20
        } else if seconds_left > MID_SECONDS {
            SE_TIMER_10
        } else if seconds_left > 0 {
            SE_TIMER_5
        } else {
            SE_TIMER_0
        }
    }

    fn step(&mut self, switches: &mut Switches, events: &mut SimEvents) {
        if self.mode == MODE_COUNT {
            self.count(switches, events);
        } else {
            self.wait(switches);
        }
    }
}

pub fn is_objtime(name: &str) -> bool {
    OBJTIME_ACTORS.contains(&name)
}

pub fn control(
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut timers: Query<&mut Objtime>,
) {
    for mut timer in &mut timers {
        timer.step(&mut switches, &mut events);
    }
}
