use bevy_ecs::prelude::*;

use super::area::StageName;
use super::daynight::check_event_night_stop;
use super::door::StayRoom;
use super::flags::{Collect, EventFlags};
use super::kytag00::EnvLight;

pub const VRBOX_ACTORS: [&str; 1] = ["Vrbox"];

const STAGE_NEW_DUNGEON: &str = "M_NewD2"; // <- d_a_vrbox.cpp:105
const STAGE_DRAGON: &str = "M_Dra09"; // <- d_a_vrbox.cpp:109
const STAGE_KIN_MB: &str = "kinMB"; // <- d_a_vrbox.cpp:111
const STAGE_KINDAN: &str = "kindan"; // <- d_a_vrbox.cpp:113
const ROOM_NEW_DUNGEON: i8 = 3; // <- d_a_vrbox.cpp:107
const ROOM_KINDAN_RAIN_A: i8 = 2; // <- d_a_vrbox.cpp:114
const ROOM_KINDAN_RAIN_B: i8 = 13; // <- d_a_vrbox.cpp:114
const ROOM_KINDAN_THUNDER: i8 = 4; // <- d_a_vrbox.cpp:116
const MODE_NONE: u8 = 0; // <- d_a_vrbox.cpp:97
const MODE_RAIN_THUNDER: u8 = 1; // <- d_a_vrbox.cpp:124
const MODE_THUNDER: u8 = 2; // <- d_a_vrbox.cpp:131
const RAIN_FULL: i32 = 250; // <- d_a_vrbox.cpp:125, :127
const THUNDER_NONE: u8 = 0; // <- d_a_vrbox.cpp:133
const THUNDER_RAIN: u8 = 1; // <- d_a_vrbox.cpp:129
const THUNDER_FLASH: u8 = 0xA; // <- d_a_vrbox.cpp:135
const COLPAT_WEATHER: u8 = 1; // <- d_a_vrbox.cpp:126, :134
const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)

/// The single actor that draws the virtual sky over every interior. The stage string is `Vrbox`
/// (`d_stage.cpp:1260`); `Vrbox2` (`d_stage.cpp:1261`) is `fpcNm_VRBOX2_e`, a different actor in
/// `d_a_vrbox2.cpp`, and must not be registered here. There is no object archive: the model is
/// the stage resource `vr_sky.bdl` (`d_a_vrbox.cpp:168`).
///
/// `_draw` (`:16-56`) and `daVrbox_color_set` (`:59-92`) read `g_env_light.mVrKasumiMaeColor`,
/// `mVrSkyColor`, `mVrkumoColor` and `mbVrboxInvisible` (`d_kankyo.h:0xAE0`, `:0xAF0`,
/// `:0xB00`, `:0xC96`) - none of which `kytag00::EnvLight` carries - and `m29C`
/// (`d_a_vrbox.cpp:185`, `d_a_vrbox.h:0x29C`) is written once and read nowhere, so this actor
/// keeps no state at all.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vrbox;

impl Vrbox {
    pub fn spawn() -> Self {
        Self
    }

    /// The profile is `fopAc_CULLBOX_0_e` (`d_a_vrbox.cpp:220`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

/// `dKy_change_colpat` (`d_kankyo.cpp:3339-3345`). The `mColpatCurr != param_0` test guards
/// zeroing `mColPatBlendGather`; `mColpatCurr` (`d_kankyo.h:0xC85`) has no `EnvLight` field, so
/// the blend reset is omitted.
fn change_colpat(light: &mut EnvLight, colpat: u8) {
    light.colpat_curr_gather = colpat;
}

/// `dKyw_rain_set` (`d_kankyo_wether.cpp:1430-1433`): sets both the live count and the original.
fn rain_set(light: &mut EnvLight, count: i32) {
    let count = count as f32;
    light.rain_count = count;
    light.rain_count_orig = count;
}

/// `dungeon_rain_proc` (`d_a_vrbox.cpp:95-145`).
fn dungeon_rain_proc(
    light: &mut EnvLight,
    stage: &str,
    room: i8,
    flags: &EventFlags,
    collect: &Collect,
) {
    if !check_event_night_stop(flags, collect) {
        return;
    }
    let mode = match stage {
        STAGE_NEW_DUNGEON => {
            if room == ROOM_NEW_DUNGEON {
                MODE_RAIN_THUNDER
            } else {
                MODE_NONE
            }
        }
        STAGE_DRAGON | STAGE_KIN_MB => MODE_RAIN_THUNDER,
        STAGE_KINDAN => match room {
            ROOM_KINDAN_RAIN_A | ROOM_KINDAN_RAIN_B => MODE_RAIN_THUNDER,
            ROOM_KINDAN_THUNDER => MODE_THUNDER,
            _ => MODE_NONE,
        },
        _ => return,
    };
    match mode {
        MODE_RAIN_THUNDER => {
            if light.rain_count_orig != RAIN_FULL as f32 {
                change_colpat(light, COLPAT_WEATHER);
                rain_set(light, RAIN_FULL);
                light.thunder_mode = THUNDER_RAIN;
            }
        }
        MODE_THUNDER => {
            if light.thunder_mode == THUNDER_NONE {
                change_colpat(light, COLPAT_WEATHER);
                light.thunder_mode = THUNDER_FLASH;
            }
        }
        _ => {
            if light.thunder_mode != THUNDER_NONE {
                rain_set(light, 0);
                light.thunder_mode = THUNDER_NONE;
            }
        }
    }
}

pub fn is_vrbox(name: &str) -> bool {
    VRBOX_ACTORS.contains(&name)
}

pub fn control(
    mut light: ResMut<EnvLight>,
    stage: Res<StageName>,
    stay: Res<StayRoom>,
    flags: Res<EventFlags>,
    collect: Res<Collect>,
    boxes: Query<&Vrbox>,
) {
    for _ in &boxes {
        dungeon_rain_proc(&mut light, &stage.0, stay.0, &flags, &collect);
    }
}
