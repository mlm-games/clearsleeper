use bevy_ecs::prelude::*;

use super::events::{SimEvent, SimEvents};
use super::flags::Switches;

pub const ATDOOR_ACTORS: [&str; 1] = ["ATdoor"];

const SE_JAIL_DOOR_OP: u32 = 0x69A4; // <- JAZelAudio_SE.h:1943 (JA_SE_OBJ_TC_JAIL_DOOR_OP)
const SWBIT_SHIFT: u32 = 0x00; // <- d_a_atdoor.cpp:23 (fopAcM_GetParam(this) >> 0)
const SWBIT_MASK: u32 = 0xFF; // <- d_a_atdoor.cpp:23
const SWBIT_NONE: u8 = 0xFF; // <- d_a_atdoor.cpp:75 (swBit == 0xFF arm)
const OPEN_MAX: i16 = 0x4000; // <- d_a_atdoor.cpp:79, :146
const OPEN_STEP: i16 = 0x400; // <- d_a_atdoor.cpp:124, :145
const CULL_MIN: [f32; 3] = [-150.0, 0.0, -150.0]; // <- f_op_actor_mng.cpp:564 (fopAc_CULLBOX_6_e)
const CULL_MAX: [f32; 3] = [150.0, 300.0, 150.0]; // <- f_op_actor_mng.cpp:564 (fopAc_CULLBOX_6_e)

const ACT_WAIT: u8 = 0; // <- d_a_atdoor.cpp:12 (ACT_WAIT_e)
const ACT_CLOSE_WAIT: u8 = 1; // <- d_a_atdoor.cpp:13 (ACT_CLOSE_WAIT_e)
const ACT_CLOSE: u8 = 2; // <- d_a_atdoor.cpp:14 (ACT_CLOSE_e)
const ACT_OPEN_WAIT: u8 = 3; // <- d_a_atdoor.cpp:15 (ACT_OPEN_WAIT_e)
const ACT_OPEN: u8 = 4; // <- d_a_atdoor.cpp:16 (ACT_OPEN_e)

/// `daAtdoor_c` (`d_a_atdoor.cpp`), the Bird's Prison cell door that a stage switch drives open and
/// shut. The stage string is `ATdoor` (`d_stage.cpp:458`) and matches the object's `_arcname`
/// `"Atdoor"` (`:19`).
///
/// `create` (`:92-106`) returns `cPhs_ERROR_e` (`:101`) when `fopAcM_entrySolidHeap` fails on the
/// model and moving-background load in `CreateHeap` (`:32-57`), which is the archive load; retail
/// asserts both are present (`:34`, `:42`, `:47`), so there is no spawn-then-despawn path.
/// `Mthd_IsDelete` (`:188-190`) is a bare TRUE - an accepted delete request, never a
/// self-destruct. The class declares no `dCcD_*` member, so it registers no AT or Tg collider,
/// cannot be damaged, and carries no `HitPoints`/`Hurt`.
///
/// `getSwbit` (`:22-24`) is `(fopAcM_GetParam(this) >> 0) & 0xFF`. Retail reads it through
/// `dComIfGs_isSwitch(swBit, fopAcM_GetRoomNo(this))` (`:77`, `:115`, `:137`), which is
/// `dSv_info_c::isSwitch` (`d_save.cpp:1586-1601`): `-1` and `255` read false, `i_no < 0x80`
/// (`MEMORY_SWITCH`) reads the in-memory switch array, and anything at or above `0x80 + 0x40`
/// reads the per-zone bit arrays instead. `super::flags::Switches` models only the in-memory
/// array, so a `swBit` of `0x80` or above reads false here and the index is used as-is rather than
/// shifted down; the `swBit == 0xFF` arm is transcribed separately at `:75`.
///
/// `open` is the `s16 unk_2A2` (`d_a_atdoor.h:26`), zeroed by the retail constructor and set to
/// `OPEN_MAX` only on the `ACT_OPEN_WAIT` seed (`:79`). It is the door's swing angle: it reaches
/// the door through `calcMtx`'s `mDoMtx_stack_c::YrotM(home.angle.y - unk_2A2)` (`:62`) and the
/// `cBgW` `Move()` calls, and the moving background itself is a confirmed-absent input, so the
/// angle is kept as state. `setAction` is a bare `mAction = action` (`d_a_atdoor.h:11`).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Atdoor {
    pub sw_bit: u8,
    pub action: u8,
    pub open: i16,
}

impl Atdoor {
    /// `daAtdoor_c::CreateInit` (`:67-89`).
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let sw_bit = ((parameters >> SWBIT_SHIFT) & SWBIT_MASK) as u8;
        let (action, open) = if sw_bit == SWBIT_NONE {
            (ACT_WAIT, 0)
        } else if switches.test(sw_bit) {
            (ACT_OPEN_WAIT, OPEN_MAX)
        } else {
            (ACT_CLOSE_WAIT, 0)
        };
        Self { sw_bit, action, open }
    }

    /// `daAtdoor_actionWait` (`:109-111`) does nothing.
    fn action_wait(&mut self) {}

    /// `daAtdoor_actionCloseWait` (`:114-120`).
    fn action_close_wait(&mut self, switches: &Switches, events: &mut SimEvents) {
        if switches.test(self.sw_bit) {
            events.list.push(SimEvent::Se(SE_JAIL_DOOR_OP));
            self.action = ACT_OPEN;
        }
    }

    /// `daAtdoor_actionClose` (`:123-133`).
    fn action_close(&mut self) {
        self.open -= OPEN_STEP;
        if self.open <= 0 {
            self.action = ACT_CLOSE_WAIT;
            self.open = 0;
        }
    }

    /// `daAtdoor_actionOpenWait` (`:136-141`).
    fn action_open_wait(&mut self, switches: &Switches) {
        if !switches.test(self.sw_bit) {
            self.action = ACT_CLOSE;
        }
    }

    /// `daAtdoor_actionOpen` (`:144-154`).
    fn action_open(&mut self) {
        self.open += OPEN_STEP;
        if self.open >= OPEN_MAX {
            self.action = ACT_OPEN_WAIT;
            self.open = OPEN_MAX;
        }
    }

    /// `daAtdoor_c::execute` (`:168-180`), dispatching `l_action[mAction]` (`:170-176`). The
    /// `_` arm is the table's `actOpen` entry; retail would read past the five-entry table for any
    /// other value, which no reachable `setAction` produces.
    fn step(&mut self, switches: &Switches, events: &mut SimEvents) {
        match self.action {
            ACT_WAIT => self.action_wait(),
            ACT_CLOSE_WAIT => self.action_close_wait(switches, events),
            ACT_CLOSE => self.action_close(),
            ACT_OPEN_WAIT => self.action_open_wait(switches),
            _ => self.action_open(),
        }
    }

    /// The profile is `fopAc_CULLBOX_6_e` (`:234`).
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_atdoor(name: &str) -> bool {
    ATDOOR_ACTORS.contains(&name)
}

pub fn control(
    switches: Res<Switches>,
    mut events: ResMut<SimEvents>,
    mut doors: Query<&mut Atdoor>,
) {
    for mut door in &mut doors {
        door.step(&switches, &mut events);
    }
}