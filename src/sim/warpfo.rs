use bevy_ecs::prelude::*;

use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::EventFlags;

/// The stage string is `Warpfo` (`d_stage.cpp:1161`). There is no archive: the actor loads no
/// object resource and `Draw` (`:195-197`) is a bare `TRUE`, so there is no `_arcname` - the
/// only name it owns is the event staff `"Warpfo"` (`:60`), which is also the stage string.
pub const WARPFO_ACTORS: [&str; 1] = ["Warpfo"];

const STAFF: &str = "Warpfo"; // <- d_a_warpfout.cpp:60 (dComIfGp_evmng_getMyStaffId)
const SE_WARP_EFFECT_ED: u32 = 0x2887; // <- JAZelAudio_SE.h:581 (JA_SE_LK_BOSS_WARP_EFF_ED)
const WARP_TIMER: i32 = 5; // <- d_a_warpfout.cpp:82, :108, :126 (mTimer)

/// `action_table` (`d_a_warpfout.cpp:22-24`), the cut names `dComIfGp_evmng_getMyActIdx`
/// matched at `:63-64`. An unmatched cut is `actIdx == -1` (`:66`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Warp1,
    Warp2,
    Warp3,
    Warp4,
    End,
}

impl Action {
    fn of(name: &str) -> Option<Self> {
        match name {
            "WARP1" => Some(Self::Warp1),
            "WARP2" => Some(Self::Warp2),
            "WARP3" => Some(Self::Warp3),
            "WARP4" => Some(Self::Warp4),
            "END" => Some(Self::End),
            _ => None,
        }
    }

    /// `event_init_tbl` (`:12-15`): `initWarp1` (`:81-84`), `initWarp2` (`:103-109`),
    /// `initWarp3` (`:121-127`), `initWarp4` (`:139-145`), `initEnd` (`:153`, empty).
    /// `set_effect_wind01` (`:162-167`) is the particle system, so only `initWarp1`'s
    /// `mDoAud_seStart` and the three `mTimer` stores survive.
    fn init(&self, fade: &mut Warpfo, events: &mut SimEvents) {
        match self {
            Self::Warp1 => {
                fade.timer = WARP_TIMER;
                events.list.push(SimEvent::Se(SE_WARP_EFFECT_ED));
            }
            Self::Warp2 => fade.timer = WARP_TIMER,
            Self::Warp3 => fade.timer = WARP_TIMER,
            Self::Warp4 => {}
            Self::End => {}
        }
    }

    /// `event_action_tbl` (`:17-20`). `actWarp1` (`:87-100`), `actWarp2` (`:112-118`) and
    /// `actWarp3` (`:130-136`) are the same countdown once `actWarp1`'s `mTimer == 4` wind
    /// burst (`:88-93`) is gone, so they share one arm. `actWarp4` (`:148-150`) is a bare
    /// `TRUE`; `actEnd` (`:156-159`) deletes the actor and then returns `TRUE`.
    fn act(&self, fade: &mut Warpfo) -> Step {
        match self {
            Self::Warp1 | Self::Warp2 | Self::Warp3 => {
                if fade.calc_timer() {
                    Step::CutEnd
                } else {
                    Step::Hold
                }
            }
            Self::Warp4 => Step::CutEnd,
            Self::End => Step::Delete,
        }
    }
}

/// `demo_proc`'s `if ((this->*event_action_tbl[actIdx])(mStaffId))
/// dComIfGp_evmng_cutEnd(mStaffId)` (`d_a_warpfout.cpp:74-76`), plus `actEnd`'s
/// `fopAcM_delete(this)` (`:157`), which is a real despawn and not the RULE 6b delete request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Hold,
    CutEnd,
    Delete,
}

/// The actor that drives the boss warp fade-out. `_create` (`:37-42`) always returns
/// `cPhs_COMPLEATE_e`, so nothing here refuses creation; `CreateInit` (`:32-34`) writes only
/// `unknownInitFlag`, read nowhere in this translation unit, and `checkOrder` (`:53`) and
/// `eventOrder` (`:56`) are both empty bodies, so `demo_proc` is the whole actor.
///
/// The profile is `fopAc_CULLBOX_CUSTOM_e` (`:233`) but this translation unit contains no
/// `fopAcM_setCullSizeBox` call at all, so there is no custom box to port and no `cull()` here.
/// `daWarpfout_Draw` (`:195-197`) returns TRUE with no draw call, so retail renders nothing for
/// this actor either way. `IsDelete` (`:207-209`) is the bare TRUE of RULE 6b - an accepted
/// delete request. The only real destruction is `actEnd`'s `fopAcM_delete(this)` (`:157`).
///
/// `demo_proc`'s action entry point is `dComIfGp_evmng_getMyActIdx(mStaffId, action_table, ...)`
/// (`:63-64`), and this sim's `EventList::cut_name` supplies the cut name that table is matched
/// against.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Warpfo {
    /// `mTimer` (`d_a_warpfout.h:35`).
    pub timer: i32,
}

impl Warpfo {
    pub fn spawn() -> Self {
        Self { timer: 0 }
    }

    /// `cLib_calcTimer` (`c_lib.h:85-90`), as `actWarp1`/`actWarp2`/`actWarp3` call it.
    fn calc_timer(&mut self) -> bool {
        if self.timer != 0 {
            self.timer -= 1;
        }
        self.timer == 0
    }

    /// `demo_proc` (`:59-78`). `dComIfGp_evmng_getMyStaffId` is this sim's
    /// `ActiveEvent::staff_cut` under the same name, `getMyActIdx` is `cut_name` matched against
    /// `action_table`, `getIsAddvance` is the returned advance flag, and `actIdx == -1` is the
    /// unmatched cut. `eventInfo.checkCommandTalk()` is the event manager's TALK command, which
    /// this sim leaves false, the same proven default `ba1.rs:1290-1292` uses.
    fn demo_proc(
        &mut self,
        list: &EventList,
        cut: usize,
        advance: bool,
        events: &mut SimEvents,
    ) -> Step {
        let Some(action) = list.cut_name(cut).and_then(Action::of) else {
            return Step::CutEnd;
        };
        if advance {
            action.init(self, events);
        }
        action.act(self)
    }
}

pub fn is_warpfo(name: &str) -> bool {
    WARPFO_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut flags: ResMut<EventFlags>,
    mut events: ResMut<SimEvents>,
    mut fades: Query<(Entity, &mut Warpfo)>,
    mut commands: Commands,
) {
    for (entity, mut fade) in &mut fades {
        // `dComIfGp_event_runCheck() && mStaffId != -1` (`d_a_warpfout.cpp:62`): a live staff slot
        // only exists while an event is playing, so `staff_cut` returning `Some` is both halves
        // at once. `MsgGameInfo::event_run_check` is not consulted because nothing in this crate
        // ever raises `MsgGameInfo::event_mode` off `DA_EVT_MODE_NONE`, which would leave the
        // gate permanently shut; `swtact.rs:129` does use it because there the test is negated.
        let Some((cut, advance)) = active.staff_cut(&list, STAFF) else {
            continue;
        };
        match fade.demo_proc(&list, cut, advance, &mut events) {
            Step::Delete => {
                active.end_staff_cut(&list, &mut flags, STAFF);
                let _ = commands.entity(entity).despawn();
            }
            Step::CutEnd => active.end_staff_cut(&list, &mut flags, STAFF),
            Step::Hold => {}
        }
    }
}
