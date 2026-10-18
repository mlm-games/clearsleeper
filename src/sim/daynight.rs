use bevy_ecs::prelude::*;

use super::flags::{Collect, EventFlags, SYMBOL_NAYRU};

pub const TIME_DAY: i32 = 0;
pub const TIME_NIGHT: i32 = 1;

pub const TIME_WRAP: f32 = 360.0;
pub const DAY_ADVANCE_LIMIT: f32 = 165.0;
pub const NIGHT_ADVANCE_START: f32 = 270.0;
pub const NIGHT_ADVANCE_END: f32 = 345.0;
pub const MOON_SHIFT_TIME: f32 = 180.0;
pub const UNITS_PER_HOUR: f32 = 15.0;
pub const WEEK_DAYS: i32 = 7;

/// `dSv_event_flag_c::ENDLESS_NIGHT` (`d_kankyo.cpp:3163`).
pub const EVENT_ENDLESS_NIGHT: u32 = 0x0A02;

/// `dKy_checkEventNightStop` (`d_kankyo.cpp:3161-3167`): the Endless Night event bit, held down
/// only while Link has not yet been given the Nayru's Pearl. `d_kankyo.cpp:519` and `:479-481`
/// both read it, the first to freeze the clock and the second to zero it as a stage mounts.
pub fn check_event_night_stop(flags: &EventFlags, collect: &Collect) -> bool {
    flags.test(EVENT_ENDLESS_NIGHT) && !collect.is_symbol(SYMBOL_NAYRU)
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct DayNight {
    pub time: f32,
    pub date: u8,
    pub time_pass: bool,
    pub night_stop: bool,
    /// `g_env_light.mNextTime` (`d_kankyo.cpp:3575`), the time a cut asked the NEXT stage to start
    /// at. `dScnKy_env_light_c`'s constructor seeds it `-1.0f` (`:496`), which is `None` here.
    pub next_time: Option<f32>,
}

impl Default for DayNight {
    fn default() -> Self {
        Self {
            time: 0.0,
            date: 0,
            time_pass: false,
            night_stop: false,
            next_time: None,
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

    /// `dKy_instant_timechg(f32 time)`'s store (`d_kankyo.cpp`), whose argument is the day's own
    /// `15.0f`-units-per-hour scale: the DIRECTOR and LIGHT staffs both hand it `hour * 15.0f`
    /// (`d_event_data.cpp:222`, `:243`). Kankyo re-derives the light, fog and sky from the new
    /// value in retail; the port's `daynight` control does that from `time` each step, so the store
    /// is the whole of it here.
    pub fn set_time(&mut self, time: f32) {
        self.time = time;
    }

    /// `dKy_set_nexttime(f32)` (`d_kankyo.cpp:3575-3577`), the store
    /// `dEvDt_Next_Stage` calls with `Hour * 15.0f` (`d_event_data.cpp:45-46`). A cut can set it
    /// and nothing consumes it until the next stage mounts, which is
    /// [`DayNight::apply_stage_init`].
    pub fn set_next_time(&mut self, time: f32) {
        self.next_time = Some(time);
    }

    /// `dScnKy_env_light_c::init`'s time arms (`d_kankyo.cpp:467-481`), in the reference's order:
    /// the pending time the last cut asked for with `dKy_set_nexttime` (`d_event_data.cpp:45-46`)
    /// becomes the clock and the slot goes back to `-1.0f` (`None`); then the new stage's own
    /// `dStage_stagInfo_GetTimeH` write whenever its STAG header names an hour (`:472-475`); then
    /// the event-night-stop zero (`:479-481`), which rides the same predicate the per-step clock
    /// gate uses. `time_h` is the header's `s8` field, tested against zero exactly as the
    /// reference's `int timeH` does; `None` is a stage with no STAG header at all, which is the
    /// reference's own `timeH < 0` arm.
    pub fn apply_stage_init(&mut self, time_h: Option<i8>, night_stop: bool) {
        if let Some(time) = self.next_time.take() {
            self.time = time;
        }
        if let Some(hours) = time_h {
            if hours >= 0 {
                self.time = f32::from(hours) * UNITS_PER_HOUR;
            }
        }
        if night_stop {
            self.time = 0.0;
        }
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

/// The per-step clock, `d_kankyo.cpp:505-544`'s block. The stop flag is the reference's
/// per-frame `dKy_checkEventNightStop()` call at `:519`, refreshed before the advance so
/// [`DayNight::night_stop`] always reads the current event bit rather than a stale one.
pub fn control(flags: Res<EventFlags>, collect: Res<Collect>, mut day: ResMut<DayNight>) {
    day.night_stop = check_event_night_stop(&flags, &collect);
    day.advance(1.0);
}

pub fn init(world: &mut World) {
    world.init_resource::<DayNight>();
}
