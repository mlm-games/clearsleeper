use bevy_ecs::prelude::*;

use super::events::{SimEvent, SimEvents};
use super::flags::Switches;

pub const TENMADO_ACTORS: [&str; 1] = ["Tenmado"];

pub const SE_SHTR_OP: u32 = 0x69BF; // <- JAZelAudio_SE.h:1968 (JA_SE_OBJ_TENMADO_SHTR_OP)
pub const CULL_MIN: [f32; 3] = [-500.0, -500.0, -500.0]; // <- d_a_obj_tenmado.cpp:39
pub const CULL_MAX: [f32; 3] = [500.0, 500.0, 500.0]; // <- d_a_obj_tenmado.cpp:39

const PRM_SWSAVE_W: u32 = 8; // <- d_a_obj_tenmado.h:10
const PRM_SWSAVE_S: u32 = 0; // <- d_a_obj_tenmado.h:11
const PRM_SWSAVE2_W: u32 = 8; // <- d_a_obj_tenmado.h:13
const PRM_SWSAVE2_S: u32 = 8; // <- d_a_obj_tenmado.h:14

const MODE_WAIT: u8 = 0; // <- d_a_obj_tenmado.cpp:100
const MODE_PLAY_SE: u8 = 1; // <- d_a_obj_tenmado.cpp:106
const MODE_SLIDE: u8 = 2; // <- d_a_obj_tenmado.cpp:111
const MODE_OPEN: u8 = 3; // <- d_a_obj_tenmado.cpp:121

const OPEN_AT_START: f32 = 200.0; // <- d_a_obj_tenmado.cpp:42
const OPEN_CLOSED: f32 = 0.0; // <- d_a_obj_tenmado.cpp:45
const OPEN_STEP: f32 = 1.0; // <- d_a_obj_tenmado.cpp:112
const OPEN_LIMIT: f32 = 100.0; // <- d_a_obj_tenmado.cpp:113
const OPEN_SET_SWITCH: f32 = 80.0; // <- d_a_obj_tenmado.cpp:116

/// `daObj::PrmAbstract(this, W, S)` (`d_a_obj.h:14`).
fn prm(param: u32, width: u32, shift: u32) -> u8 {
    (((1 << width) - 1) & (param >> shift)) as u8
}

/// The Earth Temple doors that slide apart to let the ceiling light beam through. The stage
/// string and the model's `_arcname` are both `Tenmado` (`d_stage.cpp:992`,
/// `d_a_obj_tenmado.cpp:13`).
///
/// `set_mtx` (`:78-88`) is the only reader of `m2C8`: it offsets the left panel by `-m2C8`
/// and the right by `+m2C8` along the rotated model X axis, and `Execute`'s
/// `*mtx = &M_tmp_mtx` (`:126`) hands the same matrix to `MoveBGExecute`
/// (`d_bg_s_movebg_actor.cpp:95-97`), which copies it into the `dBgW`. Both are renderer and
/// moving-background work, and `current.pos` itself never moves, so only the state machine,
/// the switch writes and the one-shot SE are simulated here.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Tenmado {
    sw_save: u8,
    sw_save2: u8,
    mode: u8,
    open: f32,
}

impl Tenmado {
    /// `Create` (`:36-49`), reached from `MoveBGCreate` (`d_bg_s_movebg_actor.cpp:68`) during
    /// `_create`: a stage that loads with `prm_get_swSave` already set turns
    /// `prm_get_swSave2` on, parks `m2C8` at 200.0f and skips straight to `m2CC = 3`.
    pub fn spawn(parameters: u32, switches: &mut Switches) -> Self {
        let sw_save = prm(parameters, PRM_SWSAVE_W, PRM_SWSAVE_S);
        let sw_save2 = prm(parameters, PRM_SWSAVE2_W, PRM_SWSAVE2_S);
        let (open, mode) = if switches.test(sw_save) {
            switches.set(sw_save2);
            (OPEN_AT_START, MODE_OPEN)
        } else {
            (OPEN_CLOSED, MODE_WAIT)
        };
        Self {
            sw_save,
            sw_save2,
            mode,
            open,
        }
    }

    /// `Execute` (`:98-128`), minus `set_mtx` (`:125`) and the `*mtx` hand-off (`:126`).
    fn step(&mut self, switches: &mut Switches, events: &mut SimEvents) {
        match self.mode {
            MODE_WAIT => {
                if switches.test(self.sw_save) {
                    self.mode = MODE_PLAY_SE;
                }
            }
            MODE_PLAY_SE => {
                events.list.push(SimEvent::Se(SE_SHTR_OP));
                self.mode = MODE_SLIDE;
            }
            MODE_SLIDE => {
                self.open += OPEN_STEP;
                if self.open >= OPEN_LIMIT {
                    self.open = OPEN_LIMIT;
                    self.mode = MODE_OPEN;
                } else if self.open >= OPEN_SET_SWITCH {
                    switches.set(self.sw_save2);
                }
            }
            _ => {}
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_tenmado(name: &str) -> bool {
    TENMADO_ACTORS.contains(&name)
}

pub fn control(
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut doors: Query<&mut Tenmado>,
) {
    for mut door in &mut doors {
        door.step(&mut switches, &mut events);
    }
}
