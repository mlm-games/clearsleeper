use bevy_ecs::prelude::*;

use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::walk;

/// The stage string is `SMtoge` (`d_stage.cpp:942`, `fpcNm_MTOGE_e`), NOT the `_arcname` `Mtoge`
/// (`d_a_mtoge.cpp:12`, `:34`, `:48`, `:102`, `:221`).
pub const SMTOGE_ACTORS: [&str; 1] = ["SMtoge"];

/// `enum Action` (`d_a_mtoge.cpp:14-20`), mirrored by `static BOOL (*l_action[])` (`:178-184`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// `ACT_WAIT` (`d_a_mtoge.cpp:15`), `daMtoge_actionWait` (`:117`).
    Wait,
    /// `ACT_HIND` (`:16`), `daMtoge_actionHind` (`:122`).
    Hind,
    /// `ACT_UP` (`:17`), `daMtoge_actionUp` (`:136`).
    Up,
    /// `ACT_ARRIVAL` (`:18`), `daMtoge_actionArrival` (`:150`).
    Arrival,
    /// `ACT_DOWN` (`:19`), `daMtoge_actionDown` (`:164`).
    Down,
}

/// `u8 daMtoge_c::getSwbit() { return fopAcM_GetParam(this) & 0xFF; }` (`d_a_mtoge.cpp:23-25`).
const SWBIT_MASK: u32 = 0xFF;
/// `if (sw == 0xFF)` (`d_a_mtoge.cpp:80`) - the "never moves" spike pair.
const NO_SWITCH: u8 = 0xFF;
/// `mHeightOffset = -300.0f` (`d_a_mtoge.cpp:84`, `:87`).
const HEIGHT_DOWN: f32 = -300.0;
/// `cLib_chaseF(&i_this->speedF, 30.0f, 4.0f)` (`d_a_mtoge.cpp:137`, `:165`).
const SPEED_TARGET: f32 = 30.0;
/// the `4.0f` in the same call (`:137`, `:165`).
const SPEED_STEP: f32 = 4.0;
/// `JA_SE_OBJ_TOGE_OUT` (`JAZelAudio_SE.h:1900`).
const SE_TOGE_OUT: u32 = 0x6976;
/// `JA_SE_OBJ_TOGE_IN` (`JAZelAudio_SE.h:1901`).
const SE_TOGE_IN: u32 = 0x6977;
/// `fopAc_CULLBOX_6_e` (`d_a_mtoge.cpp:255`) -> `f_op_actor_mng.cpp:564`.
const CULL_MIN: [f32; 3] = [-150.0, 0.0, -150.0];
const CULL_MAX: [f32; 3] = [150.0, 300.0, 150.0];

/// The Forsaken Fortress spike pair that rises out of, and sinks back into, the floor. It owns no
/// actor collider at all - `daMtoge_c::CreateHeap` (`d_a_mtoge.cpp:33-61`) builds only a `dBgW`
/// moving background from the DZB, and no `dCcD_*` collider is ever created, so it has no AT, no
/// TG and no health field (`d_a_mtoge.h:8-31`). The profile `Group` is `fopAc_ACTOR_e` (`:254`).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Smtoge {
    /// `u8 daMtoge_c::getSwbit()` (`d_a_mtoge.cpp:23-25`).
    pub swbit: u8,
    /// `mAction` (`d_a_mtoge.h:20`), set by `setAction` (`:12`) from `l_action[mAction]` (`:186`).
    pub action: Action,
    /// `f32 mHeightOffset` (`d_a_mtoge.h:30`), the vertical travel applied to the `dBgW`.
    pub height_offset: f32,
    /// `fopAc_ac_c::speedF` (`f_op_actor.h:301`), set by `fopAcM_SetSpeedF` (`d_a_mtoge.cpp:127`).
    pub speed_f: f32,
}

impl Smtoge {
    /// `daMtoge_c::CreateInit` (`d_a_mtoge.cpp:71-96`) minus the `dComIfG_Bgsp()->Regist` assert
    /// (`:74-76`), `tevStr.mRoomNo` (`:78`), `calcMtx()` (`:92`) and `mpBgW->Move()` (`:93`): the
    /// four-way `sw` decision (`:80-90`) that picks `mAction` and `mHeightOffset`.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let swbit = (parameters & SWBIT_MASK) as u8;
        let (action, height_offset) = if swbit == NO_SWITCH {
            (Action::Wait, 0.0)
        } else if switches.test(swbit.wrapping_add(1)) {
            (Action::Wait, HEIGHT_DOWN)
        } else if !switches.test(swbit) {
            (Action::Hind, HEIGHT_DOWN)
        } else {
            (Action::Arrival, 0.0)
        };
        Self {
            swbit,
            action,
            height_offset,
            speed_f: 0.0,
        }
    }

    /// The profile is `fopAc_CULLBOX_6_e` (`d_a_mtoge.cpp:255`), so no `setCullSizeBox` is needed.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `daMtoge_c::execute` (`d_a_mtoge.cpp:177-189`): the `l_action[mAction]` dispatch (`:186`).
    fn step(&mut self, switches: &Switches, events: &mut SimEvents) {
        match self.action {
            Action::Wait => self.action_wait(),
            Action::Hind => self.action_hind(switches, events),
            Action::Up => self.action_up(),
            Action::Arrival => self.action_arrival(switches, events),
            Action::Down => self.action_down(),
        }
    }

    /// `daMtoge_actionWait` (`d_a_mtoge.cpp:117-119`).
    fn action_wait(&mut self) {}

    /// `daMtoge_actionHind` (`d_a_mtoge.cpp:122-133`).
    fn action_hind(&mut self, switches: &Switches, events: &mut SimEvents) {
        if switches.test(self.swbit) != 0 {
            self.action = Action::Up;
            self.speed_f = 0.0;
            events.list.push(SimEvent::Se(SE_TOGE_OUT));
        }
    }

    /// `daMtoge_actionUp` (`d_a_mtoge.cpp:136-147`) minus `calcMtx()` (`:143`) and
    /// `mpBgW->Move()` (`:144`).
    fn action_up(&mut self) {
        walk::chase_f(&mut self.speed_f, SPEED_TARGET, SPEED_STEP);
        let speed = self.speed_f;
        if walk::chase_f(&mut self.height_offset, 0.0, speed) {
            self.action = Action::Arrival;
        }
    }

    /// `daMtoge_actionArrival` (`d_a_mtoge.cpp:150-161`).
    fn action_arrival(&mut self, switches: &Switches, events: &mut SimEvents) {
        if switches.test(self.swbit) == 0 {
            self.action = Action::Down;
            self.speed_f = 0.0;
            events.list.push(SimEvent::Se(SE_TOGE_IN));
        }
    }

    /// `daMtoge_actionDown` (`d_a_mtoge.cpp:164-175`) minus `calcMtx()` (`:171`) and
    /// `mpBgW->Move()` (`:172`).
    fn action_down(&mut self) {
        walk::chase_f(&mut self.speed_f, SPEED_TARGET, SPEED_STEP);
        let speed = self.speed_f;
        if !walk::chase_f(&mut self.height_offset, HEIGHT_DOWN, speed) {
            self.action = Action::Hind;
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