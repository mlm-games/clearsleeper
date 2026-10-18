use bevy_ecs::prelude::*;

use super::event::{ActiveEvent, EventList};
use super::flags::{EventReg, GHOST_SHIP_DONE, REG_GHOST_SHIP};

pub const PSCNCHG_ACTORS: [&str; 1] = ["PScnChg"];

const PROC_INIT: u8 = 0; // <- d_a_tag_ghostship.h:9
const PROC_EXEC: u8 = 1; // <- d_a_tag_ghostship.h:10

const CLEAR_WAIT: &str = "DEFAULT_TREASURE"; // <- d_a_tag_ghostship.cpp:25
const CLEAR_WAIT2: &str = "DEFAULT_TREASURE2"; // <- d_a_tag_ghostship.cpp:25
const CLEAR_WAIT_A: &str = "DEFAULT_TREASURE_A"; // <- d_a_tag_ghostship.cpp:25

/// `daTag_Gship_c::mode_proc[]` (`d_a_tag_ghostship.cpp:77-88`), indexed by `mMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    ClearWait,
    ClearEvent,
}

/// The ghost-ship clear tag. `mMode` (`d_a_tag_ghostship.h:27`) sits at 0 out of
/// `fopAcM_ct`'s zero fill, so `_execute`'s `modeProc(PROC_EXEC_e, 2)` runs `CLEAR_WAIT` on
/// the first tick. `CLEAR_WAIT` polls three named stage events and, on the first one that
/// closes, re-enters `modeProc` with `PROC_INIT_e`, which runs `modeClearEventInit`: the
/// `GHOST_SHIP` event register is forced to 3 (`d_a_tag_ghostship.cpp:36`), which is the
/// terminal value `ghostship::control` (`src/sim/ghostship.rs:123`) already watches for.
///
/// The fourth `modeClearWait` term, `l_HIO.field_0x05` (`:25`), is provably false: `l_HIO` is a
/// file-static whose constructor writes `field_0x05 = 0` (`:13-16`) and nothing in the crate
/// ever rewrites it - there is no `mDoHIO_createChild` call in this translation unit - so it
/// folds to `false` exactly like a zero `REG0_F(n)`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PScnChg {
    pub mode: Mode,
}

/// `dComIfGp_evmng_endCheck(name)` (`d_com_inf_game.h:3663`) is
/// `endCheck(getEventIdx(name, 0xFF))`; `getEventIdx` (`d_event_manager.cpp:342-359`) is the
/// event-list name lookup and `endCheck` (`:386-392`) is true only in the `CLOSE_e` state.
fn ended(active: &ActiveEvent, list: &EventList, name: &str) -> bool {
    list.event_index(name)
        .is_some_and(|index| active.end_check(index))
}

impl PScnChg {
    pub fn spawn() -> Self {
        Self {
            mode: Mode::ClearWait,
        }
    }

    /// `modeProc` (`d_a_tag_ghostship.cpp:69-97`). `param` is `mMode`'s source on `PROC_INIT_e`
    /// and ignored on `PROC_EXEC_e`, so `_execute`'s literal `2` (`:101`) carries no meaning.
    fn mode_proc(
        &mut self,
        proc: u8,
        param: Mode,
        active: &ActiveEvent,
        list: &EventList,
        regs: &mut EventReg,
    ) {
        match proc {
            PROC_INIT => {
                self.mode = param;
                match self.mode {
                    Mode::ClearWait => self.mode_clear_wait_init(),
                    Mode::ClearEvent => self.mode_clear_event_init(regs),
                }
            }
            PROC_EXEC => match self.mode {
                Mode::ClearWait => self.mode_clear_wait(active, list, regs),
                Mode::ClearEvent => self.mode_clear_event(),
            },
            _ => {}
        }
    }

    /// `modeClearWaitInit` (`:19-21`) is `return;`.
    fn mode_clear_wait_init(&mut self) {}

    /// `modeClearWait` (`:24-28`).
    fn mode_clear_wait(&mut self, active: &ActiveEvent, list: &EventList, regs: &mut EventReg) {
        if ended(active, list, CLEAR_WAIT)
            || ended(active, list, CLEAR_WAIT2)
            || ended(active, list, CLEAR_WAIT_A)
        {
            self.mode_proc(PROC_INIT, Mode::ClearEvent, active, list, regs);
        }
    }

    /// `modeClearEventInit` (`:31-39`). Retail NTSC-U takes the non-`VERSION_DEMO` branch and
    /// assigns `3` outright, so the `getEventReg` read it overwrites is not needed.
    fn mode_clear_event_init(&self, regs: &mut EventReg) {
        regs.set_reg(REG_GHOST_SHIP, GHOST_SHIP_DONE);
    }

    /// `modeClearEvent` (`:42-66`). Both arms need inputs this sim does not have: the
    /// `eventInfo.checkCommandDemoAccrpt()` arm is the `dEvtType_DEMO` order queue, and the
    /// `else` arm's `fopAcM_orderOtherEvent(this, "PSHIP_CLEAR")` is the same queue. Neither
    /// exists in `event::ActiveEvent`, which models only the stage event manager, so the whole
    /// body is omitted - see the report.
    fn mode_clear_event(&mut self) {}
}

pub fn is_pscnchg(name: &str) -> bool {
    PSCNCHG_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    active: Res<ActiveEvent>,
    mut regs: ResMut<EventReg>,
    mut tags: Query<&mut PScnChg>,
) {
    for mut tag in &mut tags {
        tag.mode_proc(PROC_EXEC, Mode::ClearWait, &active, &list, &mut regs);
    }
}
