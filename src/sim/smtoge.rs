use bevy_ecs::prelude::*;

use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::walk;

/// The stage string is `SMtoge` (`d_stage.cpp:942`, `fpcNm_MTOGE_e`), NOT the `_arcname` `Mtoge`
/// (`d_a_mtoge.cpp:12`, `:34`, `:48`, `:102`, `:221`).
pub const SMTOGE_ACTORS: [&str; 1] = ["SMtoge"];

/// `fopAcM_GetParam(this) & 0xFF` (`d_a_mtoge.cpp:24`).
const SWBIT_MASK: u32 = 0xFF;
/// `if (sw == 0xFF)` (`d_a_mtoge.cpp:80`).
const NO_SWITCH: u8 = 0xFF;
/// `mHeightOffset = -300.0f` (`d_a_mtoge.cpp:84`, `:87`) and the `cLib_chaseF` target (`:167`).
const RETRACTED: f32 = -300.0;
/// `cLib_chaseF(&i_this->speedF, 30.0f, 4.0f)` (`d_a_mtoge.cpp:137`, `:165`).
const CHASE_SPEED: f32 = 30.0;
/// `..., 4.0f)` (`d_a_mtoge.cpp:137`, `:165`).
const CHASE_STEP: f32 = 4.0;
/// `fopAcM_SetSpeedF(i_this, 0.0f)` (`d_a_mtoge.cpp:127`, `:155`).
const STOP_SPEED: f32 = 0.0;
/// `JA_SE_OBJ_TOGE_OUT` (`JAZelAudio_SE.h:1900`).
const SE_TOGE_OUT: u32 = 0x6976;
/// `JA_SE_OBJ_TOGE_IN` (`JAZelAudio_SE.h:1901`).
const SE_TOGE_IN: u32 = 0x6977;

/// `enum Action` (`d_a_mtoge.cpp:14-20`), the `l_action[]` index of `execute` (`:178-186`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// `ACT_WAIT` (`d_a_mtoge.cpp:15`).
    Wait,
    /// `ACT_HIND` (`d_a_mtoge.cpp:16`).
    Hind,
    /// `ACT_UP` (`d_a_mtoge.cpp:17`).
    Up,
    /// `ACT_ARRIVAL` (`d_a_mtoge.cpp:18`).
    Arrival,
    /// `ACT_DOWN` (`d_a_mtoge.cpp:19`).
    Down,
}

/// The Forsaken-Fortress spikes. `CreateHeap` (`d_a_mtoge.cpp:33-61`) builds only a `J3DModel` and
/// a `dBgW` from `dRes_INDEX_MTOGE_DZB_S_MTOGE_e` (`:48`) - there is no `dCcD` of its own, so it
/// registers no AT or Tg collider, cannot be damaged and cannot damage anything. `daMtoge_c`
/// (`d_a_mtoge.h:8-31`) has no health field and the profile `Group` is `fopAc_ACTOR_e`
/// (`d_a_mtoge.cpp:252`), so neither `combat::HitPoints` nor `enemy::Hurt` is attached.
///
/// `create` (`:99-114`) can only return `cPhs_ERROR_e` from `fopAcM_entrySolidHeap` on a heap
/// failure (`:107-109`), never from its parameters, so there is no spawn-then-despawn path.
/// `daMtoge_IsDelete` (`:211-213`) is a bare TRUE - an accepted delete request, not a self-destruct.
///
/// The profile `Cull Type` is `fopAc_CULLBOX_6_e` (`:255`), one of the fixed profile cull boxes, and
/// this file never calls `fopAcM_setCullSizeBox`, so no cull box is derivable and none is invented.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Smtoge {
    /// `getSwbit()` (`d_a_mtoge.cpp:24`), re-read every tick by `actionHind` (`:123`),
    /// `actionArrival` (`:151`) and `CreateInit` (`:72`).
    pub swbit: u8,
    /// `mAction` (`d_a_mtoge.h:26`).
    pub action: Action,
    /// `fopAc_ac_c::speedF`, written by `fopAcM_SetSpeedF` (`:127`, `:155`) and `cLib_chaseF`
    /// (`:137`, `:165`), then read back through `fopAcM_GetSpeedF` (`:139`, `:167`).
    pub speed: f32,
    /// `mHeightOffset` (`d_a_mtoge.h:30`), the offset `calcMtx` adds to `current.pos.y` (`:65`).
    pub height: f32,
}

impl Smtoge {
    /// `daMtoge_c::CreateInit` (`d_a_mtoge.cpp:71-96`), minus the `dBgW` `Regist` (`:74-76`), the
    /// `tevStr` room write (`:78`), `calcMtx` (`:92`) and `mpBgW->Move()` (`:93`). The
    /// `sw == 0xFF` arm (`:80-81`) and the `ACT_ARRIVAL` arm (`:88-90`) leave `mHeightOffset` at its
    /// placement zero.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let swbit = (parameters & SWBIT_MASK) as u8;
        let (action, retracted) = if swbit == NO_SWITCH {
            (Action::Wait, false)
        } else if switches.test(swbit.wrapping_add(1)) {
            (Action::Wait, true)
        } else if !switches.test(swbit) {
            (Action::Hind, true)
        } else {
            (Action::Arrival, false)
        };
        Self {
            swbit,
            action,
            speed: STOP_SPEED,
            height: if retracted { RETRACTED } else { 0.0 },
        }
    }

    /// `daMtoge_actionWait` (`d_a_mtoge.cpp:117-119`) - a bare `return TRUE`.
    fn action_wait(&mut self) {}

    /// `daMtoge_actionHind` (`d_a_mtoge.cpp:122-133`).
    fn action_hind(&mut self, switches: &Switches, events: &mut SimEvents) {
        if switches.test(self.swbit) {
            self.action = Action::Up;
            self.speed = STOP_SPEED;
            events.list.push(SimEvent::Se(SE_TOGE_OUT));
        }
    }

    /// `daMtoge_actionUp` (`d_a_mtoge.cpp:136-147`), minus `calcMtx` and `mpBgW->Move()` (`:143-144`).
    fn action_up(&mut self) {
        walk::chase_f(&mut self.speed, CHASE_SPEED, CHASE_STEP);
        let step = self.speed;
        if walk::chase_f(&mut self.height, 0.0, step) {
            self.action = Action::Arrival;
        }
    }

    /// `daMtoge_actionArrival` (`d_a_mtoge.cpp:150-161`).
    fn action_arrival(&mut self, switches: &Switches, events: &mut SimEvents) {
        if !switches.test(self.swbit) {
            self.action = Action::Down;
            self.speed = STOP_SPEED;
            events.list.push(SimEvent::Se(SE_TOGE_IN));
        }
    }

    /// `daMtoge_actionDown` (`d_a_mtoge.cpp:164-175`), minus `calcMtx` and `mpBgW->Move()`
    /// (`:171-172`).
    fn action_down(&mut self) {
        walk::chase_f(&mut self.speed, CHASE_SPEED, CHASE_STEP);
        let step = self.speed;
        if walk::chase_f(&mut self.height, RETRACTED, step) {
            self.action = Action::Hind;
        }
    }

    /// `daMtoge_c::execute` (`d_a_mtoge.cpp:177-189`) dispatching `l_action[mAction]`.
    fn step(&mut self, switches: &Switches, events: &mut SimEvents) {
        match self.action {
            Action::Wait => self.action_wait(),
            Action::Hind => self.action_hind(switches, events),
            Action::Up => self.action_up(),
            Action::Arrival => self.action_arrival(switches, events),
            Action::Down => self.action_down(),
        }
    }
}

pub fn is_smtoge(name: &str) -> bool {
    SMTOGE_ACTORS.contains(&name)
}

pub fn control(
    switches: Res<Switches>,
    mut events: ResMut<SimEvents>,
    mut spikes: Query<&mut Smtoge>,
) {
    for mut spike in &mut spikes {
        spike.step(&switches, &mut events);
    }
}