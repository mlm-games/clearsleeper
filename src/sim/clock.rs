use bevy_ecs::prelude::*;

use super::inventory::Got;

pub const TIME_ADV: f32 = 0.02;
pub const DAY_LIMIT: f32 = 360.0;
pub const DAY_MORNING_LIMIT: f32 = 165.0;
pub const NIGHT_FROM: f32 = 270.0;
pub const NIGHT_TO: f32 = 345.0;
pub const DAY_HOUR: f32 = 6.0;
pub const DUSK_HOUR: f32 = 18.0;
pub const HOUR_UNITS: f32 = 15.0;
pub const NOON_LIMIT: f32 = 180.0;
pub const WEEK: u32 = 7;
pub const START_TIME: f32 = 0.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Day {
    Day,
    Night,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Clock {
    pub time: f32,
    pub date: u32,
    pub time_pass: bool,
    pub night_stop: bool,
    pub busy: bool,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            time: START_TIME,
            date: 0,
            time_pass: false,
            night_stop: false,
            busy: false,
        }
    }
}

impl Clock {
    pub fn hour(&self) -> f32 {
        self.time / HOUR_UNITS
    }

    pub fn minute(&self) -> f32 {
        let fraction = (self.time * 1_000_000.0) as i64 % 15_000_000;
        fraction as f32 / 1_000_000.0 / HOUR_UNITS * 60.0
    }

    pub fn day(&self) -> Day {
        let hour = self.hour();
        if (DAY_HOUR..DUSK_HOUR).contains(&hour) {
            Day::Day
        } else {
            Day::Night
        }
    }

    pub fn night(&self) -> bool {
        self.day() == Day::Night
    }

    pub fn weekday(&self) -> u32 {
        self.date % WEEK
    }

    pub fn moon(&self) -> u32 {
        let weekday = self.weekday();
        if self.time < NOON_LIMIT {
            if weekday != 0 {
                weekday - 1
            } else {
                WEEK - 1
            }
        } else {
            weekday
        }
    }

    pub fn tick(&mut self) {
        self.time += TIME_ADV;
        if self.time >= DAY_LIMIT {
            self.time = 0.0;
            self.date += 1;
        }
    }

    pub fn running(&self, waker: bool) -> bool {
        !self.night_stop && !self.busy && waker
    }

    pub fn steps(&mut self, waker: bool) -> bool {
        if !self.running(waker) {
            return false;
        }
        if self.time_pass {
            self.tick();
            return true;
        }
        if self.day() == Day::Day && self.time < DAY_MORNING_LIMIT {
            self.tick();
            return true;
        }
        if self.time >= NIGHT_FROM && self.time < NIGHT_TO {
            self.tick();
            true
        } else {
            false
        }
    }

    pub fn set_time(&mut self, time: f32) {
        if time < self.time {
            self.date += 1;
        }
        self.time = time;
    }
}

pub fn control(got: Res<Got>, mut clock: ResMut<Clock>) {
    clock.steps(got.has(super::inventory::WAKER_SLOT, 0));
}