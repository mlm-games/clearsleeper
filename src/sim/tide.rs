use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::event::{ActiveEvent, EventList};
use super::flags::{EventFlags, Switches};
use super::wlvtag::WaterLevel;
use crate::text::msgproc::MsgGameInfo;

/// SUBSTITUTION: the assignment asked for `d_a_obj_tide.cpp` / objname `UmiSen`. No such
/// actor exists anywhere in the decomp - no file, no `_arcname`, no OBJNAME row, no `fpcNm_*_e`.
/// `ls tww/src/d/actor/ | grep -i umi` is empty. The stage strings whose only overlap is `umi`
/// are `Humi0z`, `Humi2z`, `Humi3z`, `Humi4z`, `Humi5z` (`d_stage.cpp:879-884`), all
/// `fpcNm_Obj_Tide_e`, whose archive is `"Humi"` (`d_a_obj_tide.cpp:19`) - the sea surface, i.e.
/// exactly what "UmiSen" names. This module is therefore `daObjTide::Act_c` from
/// `d_a_obj_tide.cpp`, ported into the filename the assignment reserved.
pub const TIDE_ACTORS: [&str; 6] = ["Humi0z", "Humi2z", "Humi3z", "Humi4z", "Humi5z", "Tide3"];

const TYPE_SHIFT: u32 = 16; // <- d_a_obj_tide.h:33 (PRM_TYPE_W = 3, PRM_TYPE_S = 16)
const TYPE_MASK: u32 = 0x7; // <- d_a_obj_tide.h:33
const SWSAVE_SHIFT: u32 = 24; // <- d_a_obj_tide.h:31 (PRM_SWSAVE_W = 8, PRM_SWSAVE_S = 24)
const SWSAVE_MASK: u32 = 0xFF; // <- d_a_obj_tide.h:31
const ATTR_COUNT: usize = 6; // <- d_a_obj_tide.cpp:21 (L_attr[6])
const TYPE_GMTW: u8 = 3; // <- d_a_obj_tide.cpp:298
const TMP_BIT_GMTW: u32 = 0x0401; // <- d_a_obj_tide.cpp:304, :544 (dSv_event_tmp_flag_c::UNK_0401)
const EVENT_RESCUE: &str = "rescue"; // <- d_a_obj_tide.cpp:392
const LIFT_RISE: f32 = 100.0; // <- d_a_obj_tide.cpp:476
const LIFT_HIGH: f32 = 2948.5; // <- d_a_obj_tide.cpp:515
const RETIRE_TICKS: i32 = 300; // <- d_a_obj_tide.cpp:502
const MODE_NORM: u8 = 0; // <- d_a_obj_tide.cpp:415 (mode_norm_init)
const MODE_GMTW_PRE: u8 = 1; // <- d_a_obj_tide.cpp:462 (mode_gmtw_pre_init)
const MODE_GMTW: u8 = 2; // <- d_a_obj_tide.cpp:504 (mode_gmtw_init)
const MODE_GMTW_DEMO: u8 = 3; // <- d_a_obj_tide.cpp:559 (mode_gmtw_demo_init)

/// `Attr_c::m20` (`d_a_obj_tide.h:0x20`), sign-extended from its `s16` storage.
const M20: [f32; ATTR_COUNT] = [
    -460.0,  // <- :34 (0xFE34)
    16384.0, // <- :61 (0x4000)
    -600.0,  // <- :88 (0xFDA8)
    -6000.0, // <- :115 (0xE890)
    -550.0,  // <- :142 (0xFDDA)
    -550.0,  // <- :169 (0xFDDA)
];
const SCH_FLAG: [bool; ATTR_COUNT] = [
    true,  // <- :45
    true,  // <- :72
    true,  // <- :99
    false, // <- :126 (JUT_ASSERTed false by mode_gmtw_*_init, :460, :483, :558)
    true,  // <- :153
    true,  // <- :180
];
const CULL_MIN: [[i16; 3]; ATTR_COUNT] = [
    [-4000, -1700, -3450], // <- :39-41 (0xF060, 0xF95C, 0xF286)
    [5600, -1650, -3300],  // <- :66-68 (0x15E0, 0xF98E, 0xF31C)
    [5700, -1650, 1400],   // <- :93-95 (0x1644, 0xF98E, 0x578)
    [-1544, -300, -1544],  // <- :120-122 (0xF8F8, 0xFED4, 0xF8F8)
    [-7200, -1650, -300],  // <- :147-149 (0xE3E0, 0xF98E, 0xFED4)
    [3800, -1650, 400],    // <- :174-176 (0xED8, 0xF98E, 0x190)
];
const CULL_MAX: [[i16; 3]; ATTR_COUNT] = [
    [8800, -1600, 4600],  // <- :42-44 (0x2260, 0xF9C0, 0x11F8)
    [7300, -1550, -1544], // <- :69-71 (0x1C84, 0xF9F2, 0xF8F8)
    [7800, -1550, 4800],  // <- :96-98 (0x1E78, 0xF9F2, 0x12C0)
    [1800, 150, 1800],    // <- :123-125 (0x708, 0x96, 0x708)
    [-3900, -1550, 1700], // <- :150-152 (0xF0C4, 0xF9F2, 0x6A4)
    [6000, -1550, 1500],  // <- :177-179 (0x1770, 0xF9F2, 0x5DC)
];

/// `daObjTide::Act_c`, the tide/sea-surface actor. Five stage strings share it; type 3 is the
/// `Gmtw` archive (`d_a_obj_tide.cpp:104`) reached only by the rescue demo, the other five are
/// `Humi0/2/3/4/5` (`:23`, `:50`, `:77`, `:131`, `:158`).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Tide {
    kind: u8,
    switch_no: u8,
    home: Vec3,
    /// `m308` (`d_a_obj_tide.h:0x308`), the one-way latch set when the `Gmtw` switch is on.
    risen: bool,
    /// `m30A` (`d_a_obj_tide.h:0x30A`), which `Execute` turns into `fopAcM_delete`.
    retire: bool,
    /// `m30C` (`d_a_obj_tide.h:0x30C`).
    tmp_ticks: i32,
    /// `m2EC` (`d_a_obj_tide.h:0x2EC`), the `mode_proc` index of `Execute` (`:752-759`).
    mode: u8,
    initial_y: Option<f32>,
}

fn slot(kind: u8) -> Option<usize> {
    usize::try_from(kind).ok().filter(|slot| *slot < ATTR_COUNT)
}

impl Tide {
    /// `Mthd_Create` (`d_a_obj_tide.cpp:291-320`) plus `Create` (`:270-288`). `m309` (`:296`)
    /// is only ever false under `VERSION == VERSION_DEMO` (`:300-302`), so `_create` never
    /// refuses; the `m2F7` moving-background registration and the `MoveBGCreate` failure path
    /// (`:314-317`) are `dBgW` work. The create-time `current.pos.y` write (`:311-313`) is
    /// carried to the first tick in `initial_y`.
    pub fn spawn(parameters: u32, home: Vec3, level: &WaterLevel, flags: &mut EventFlags) -> Self {
        let kind = ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8;
        if kind == TYPE_GMTW {
            flags.clear(TMP_BIT_GMTW);
        }
        let mode = if kind == TYPE_GMTW {
            MODE_GMTW_PRE
        } else {
            MODE_NORM
        };
        let initial_y = slot(kind)
            .and_then(|slot| SCH_FLAG[slot].then(|| home.y + (1.0 - level.now) * M20[slot]));
        Self {
            kind,
            switch_no: ((parameters >> SWSAVE_SHIFT) & SWSAVE_MASK) as u8,
            home,
            risen: false,
            retire: false,
            tmp_ticks: if kind == TYPE_GMTW { RETIRE_TICKS } else { 0 },
            mode,
            initial_y,
        }
    }

    /// `fopAcM_setCullSizeBox` (`d_a_obj_tide.cpp:273`) with the `s16` table entries sign-extended
    /// into the `f32` arguments of `fop_actor_mng.h:513`.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        let Some(slot) = slot(self.kind) else {
            return ([0.0; 3], [0.0; 3]);
        };
        (CULL_MIN[slot].map(f32::from), CULL_MAX[slot].map(f32::from))
    }

    /// `mode_norm` (`d_a_obj_tide.cpp:418-456`). `current.pos.y` (`:428`) is the only line whose
    /// inputs exist here: `daTagWaterlevel::Act_c::get_now()` is `wlvtag::WaterLevel::now`. The
    /// rest reads `daTagWaterlevel::State_e` (`:425-426`, `:437`, `:449`), which this sim does
    /// not keep - `wlvtag::WaterLevel` has no `M_state` field - so `m2F4` (`:429-439`, only ever
    /// set from `STATE_2`), the BRK/BTK animation (`:441-447`) and the tide SE (`:449-455`) are
    /// omitted.
    fn mode_norm(&self, level: &WaterLevel) -> f32 {
        match slot(self.kind) {
            Some(slot) => self.home.y + (1.0 - level.now) * M20[slot],
            None => self.home.y,
        }
    }

    /// `mode_gmtw_pre` (`d_a_obj_tide.cpp:467-479`). The `dComIfG_Bgsp()->Release(mpBgW)` and
    /// `m2F7` clear (`:470-473`) are moving-background work.
    fn mode_gmtw_pre(&mut self, list: &EventList, active: &ActiveEvent, info: &MsgGameInfo) -> f32 {
        if chk_demo_rescue(list, active, info) {
            self.mode = MODE_GMTW_DEMO;
            self.home.y
        } else {
            self.mode = MODE_GMTW;
            self.tmp_ticks = RETIRE_TICKS;
            self.home.y + LIFT_RISE
        }
    }

    /// `mode_gmtw` (`d_a_obj_tide.cpp:508-554`), minus the BCK/BRK/BTK block (`:526-540`),
    /// which only feeds `move_to_aim` and therefore never writes `current.pos.y`, and minus
    /// `se_set_gmtw` (`:542`). Note that the `!m308` half of the decomp assigns `fVar1` only -
    /// the position write lives in the `m308` half at `:522`.
    fn mode_gmtw(&mut self, switches: &Switches, tmp_bit: bool) -> Option<f32> {
        let high = self.home.y + LIFT_HIGH;
        let set = switches.test(self.switch_no);
        let mut write = None;
        if !self.risen {
            if set {
                self.risen = true;
            }
        } else if !set {
            write = Some(high);
        }
        if tmp_bit {
            if self.tmp_ticks > 0 {
                self.tmp_ticks -= 1;
            } else {
                self.retire = true;
            }
        } else {
            self.tmp_ticks = RETIRE_TICKS;
        }
        write
    }
}

/// `chk_demo_rescue` (`d_a_obj_tide.cpp:389-396`): `dComIfGp_event_runCheck()` is
/// `dEvt_control_c::runCheck` (`d_event.h:133-139`), modelled as
/// `MsgGameInfo::event_run_check` (`kalle_demos.rs`, `auction.rs:2140`).
fn chk_demo_rescue(list: &EventList, active: &ActiveEvent, info: &MsgGameInfo) -> bool {
    info.event_run_check()
        && list
            .event_index(EVENT_RESCUE)
            .is_some_and(|slot| active.start_check(slot))
}

pub fn is_tide(name: &str) -> bool {
    TIDE_ACTORS.contains(&name)
}

pub fn control(
    level: Res<WaterLevel>,
    switches: Res<Switches>,
    flags: Res<EventFlags>,
    list: Res<EventList>,
    active: Res<ActiveEvent>,
    info: Res<MsgGameInfo>,
    mut tides: Query<(Entity, &mut Actor, &mut Tide)>,
    mut commands: Commands,
) {
    for (entity, mut actor, mut tide) in &mut tides {
        if let Some(y) = tide.initial_y.take() {
            actor.pos.y = y;
        }
        match tide.mode {
            MODE_NORM => actor.pos.y = tide.mode_norm(&level),
            MODE_GMTW_PRE => actor.pos.y = tide.mode_gmtw_pre(&list, &active, &info),
            MODE_GMTW => {
                if let Some(y) = tide.mode_gmtw(&switches, flags.test(TMP_BIT_GMTW)) {
                    actor.pos.y = y;
                }
            }
            _ => {}
        }
        if tide.retire {
            let _ = commands.entity(entity).despawn();
        }
    }
}
