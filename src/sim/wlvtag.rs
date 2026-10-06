use bevy_ecs::prelude::*;

use super::schedule::Schedule;
use super::walk;

pub const WLV_TAG_ACTORS: [&str; 1] = ["WLvTag"];

pub const LEVEL_LOW: f32 = 0.0; // <- d_a_tag_waterlevel.cpp:36
pub const LEVEL_HIGH: f32 = 1.0; // <- d_a_tag_waterlevel.cpp:34
pub const RAMP_INCREMENT: f32 = 0.005; // <- d_a_tag_waterlevel.cpp:23 (field_0x00)
pub const MOVE_SCALE: f32 = 0.05; // <- d_a_tag_waterlevel.cpp:23 (field_0x04)
pub const MAX_STEP: f32 = 0.015; // <- d_a_tag_waterlevel.cpp:23 (field_0x08)
pub const MIN_STEP: f32 = 0.005; // <- d_a_tag_waterlevel.cpp:23 (field_0x0c)

const SCH_WIDTH: u32 = 8; // <- d_a_tag_waterlevel.h:17
const SCH_SHIFT: u32 = 0; // <- d_a_tag_waterlevel.h:18

/// `daTagWaterlevel::Act_c::M_now` is a class static (`d_a_tag_waterlevel.h:33`,
/// storage `d_com_static.cpp:71`), so every WLvTag placed in a stage drives one level.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct WaterLevel {
    pub now: f32,
    /// `daTagWaterlevel::Act_c::M_state` (`d_a_tag_waterlevel.cpp:70-85`), read by
    /// `daObj_tide::mode_norm` (`d_a_obj_tide.cpp:425-426`).
    pub state: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct WlvTag {
    pub sched: u8,
    pub ramp: f32,
    pub initial: Option<f32>,
}

fn level_for(sch: u8, schbit: u8) -> f32 {
    if sch & schbit != 0 { LEVEL_HIGH } else { LEVEL_LOW }
}

impl WlvTag {
    /// `_create` (`d_a_tag_waterlevel.cpp:30-43`) writes the shared `M_now` from the
    /// schedule bit and zeroes `field_0x290`; `initial` carries that one-shot write to
    /// the first tick.
    pub fn spawn(parameters: u32, schedule: &Schedule) -> Self {
        let sched = (((1 << SCH_WIDTH) - 1) & (parameters >> SCH_SHIFT)) as u8;
        Self {
            sched,
            ramp: 0.0,
            initial: Some(level_for(sched, schedule.bit)),
        }
    }

    /// `_execute` (`d_a_tag_waterlevel.cpp:66-109`), minus the `dComIfGp_event_runCheck()`
    /// snap branch (line 87) and the `bgm_proc` audio latch (lines 51-63).
    fn step(&mut self, level: &mut WaterLevel, schbit: u8) {
        let target = level_for(self.sched, schbit);
        if (target - level.now).abs() < MIN_STEP {
            level.now = target;
            self.ramp = 0.0;
            return;
        }
        self.ramp += RAMP_INCREMENT;
        if self.ramp > MAX_STEP {
            self.ramp = MAX_STEP;
        }
        if self.ramp < MIN_STEP {
            self.ramp = MIN_STEP;
        }
        let max_step = self.ramp;
        walk::add_calc(&mut level.now, target, MOVE_SCALE, max_step, MIN_STEP);
    }
}

pub fn is_wlv_tag(name: &str) -> bool {
    WLV_TAG_ACTORS.contains(&name)
}

pub fn control(
    mut level: ResMut<WaterLevel>,
    schedule: Res<Schedule>,
    mut tags: Query<&mut WlvTag>,
) {
    let schbit = schedule.bit;
    for mut tag in &mut tags {
        if let Some(now) = tag.initial {
            level.now = now;
            tag.initial = None;
        }
        tag.step(&mut level, schbit);
    }
}