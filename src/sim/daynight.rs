use bevy_ecs::prelude::*;


pub const TIME_DAY: i32 = 0;
pub const TIME_NIGHT: i32 = 1;

pub const TIME_WRAP: f32 = 360.0;
pub const DAY_ADVANCE_LIMIT: f32 = 165.0;
pub const NIGHT_ADVANCE_START: f32 = 270.0;
pub const NIGHT_ADVANCE_END: f32 = 345.0;
pub const MOON_SHIFT_TIME: f32 = 180.0;
pub const UNITS_PER_HOUR: f32 = 15.0;
pub const WEEK_DAYS: i32 = 7;

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct DayNight {
    pub time: f32,
    pub date: u8,
    pub time_pass: bool,
    pub night_stop: bool,
}

impl Default for DayNight {
    fn default() -> Self {
        Self {
            time: 0.0,
            date: 0,
            time_pass: false,
            night_stop: false,
        }
    }
}

impl DayNight {
    pub fn hour(&self) -> i32 {
        (self.time / UNITS_PER_HOUR) as i32
    }

    pub fn minute(&self) -> i32 {
        let scaled = ((self.time * 1_000_000.0) as i32 % 15_000_000) as f32 / 1_000_000.0;
        (scaled / UNITS_PER_HOUR * 60.0) as i32
    }

    pub fn weekday(&self) -> i32 {
        i32::from(self.date) % WEEK_DAYS
    }

    pub fn daynight(&self) -> i32 {
        let hour = self.hour();
        if (6..18).contains(&hour) {
            TIME_DAY
        } else {
            TIME_NIGHT
        }
    }

    pub fn is_day(&self) -> bool {
        self.daynight() == TIME_DAY
    }

    pub fn moon_type(&self) -> i32 {
        let mut weekday = self.weekday();
        if self.time < MOON_SHIFT_TIME {
            weekday = if weekday != 0 { weekday - 1 } else { 6 };
        }
        weekday
    }

    pub fn advance(&mut self, step: f32) {
        if self.night_stop {
            return;
        }
        if self.time_pass {
            self.time += step;
            if self.time >= TIME_WRAP {
                self.time = 0.0;
                self.date = self.date.wrapping_add(1);
            }
            return;
        }
        if self.is_day() {
            if self.time < DAY_ADVANCE_LIMIT {
                self.time += step;
            }
            return;
        }
        if (NIGHT_ADVANCE_START..NIGHT_ADVANCE_END).contains(&self.time) {
            self.time += step;
        }
    }
}

pub fn control(mut day: ResMut<DayNight>) {
    day.advance(1.0);
}

pub fn init(world: &mut World) {
    world.init_resource::<DayNight>();
}