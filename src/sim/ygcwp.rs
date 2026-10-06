use bevy_ecs::prelude::*;

use super::event::{ActiveEvent, EventList};
use super::events::{SimEvent, SimEvents};
use super::flags::EventFlags;

/// The stage string is `Ygcwp` (`d_stage.cpp:1181`, `fpcNm_Ygcwp_e`) and it is also the `_arcname`
/// (`d_a_ygcwp.cpp:28`, `:53`, `:83`, `:109`, `:133`, `:147`).
pub const YGCWP_ACTORS: [&str; 1] = ["Ygcwp"];

/// `dComIfG_evmng_getMyStaffId(M_arcname)` (`d_a_ygcwp.cpp:147`).
const ARC_NAME: &str = "Ygcwp";
/// `dSv_event_tmp_flag_c::UNK_0480` (`d_save_event_tmp_flag.inc:33`), tested by `check_ev`
/// (`d_a_ygcwp.cpp:115`) and cleared by `off_ev` (`:120`).
const TMP_BIT: u32 = 0x0480;
/// `static char* M_act_table[] = { "warp_start", "warp_appear", "warp_make" }`
/// (`d_a_ygcwp.cpp:36-40`), indexed by the `enum` at `:12-16`.
const ACT_NAMES: [&str; 3] = ["warp_start", "warp_appear", "warp_make"];
/// `dComIfGp_evmng_getMyIntegerP(mStaffId, "Timer")` (`d_a_ygcwp.cpp:138`).
const TIMER_KEY: &str = "Timer";
/// `fopAcM_setCullSizeBox(this, -160.0f, -10.0f, -160.0f, ...)` (`d_a_ygcwp.cpp:100`).
const CULL_MIN: [f32; 3] = [-160.0, -10.0, -160.0];
/// `..., 160.0f, 50.0f, 160.0f)` (`d_a_ygcwp.cpp:100`).
const CULL_MAX: [f32; 3] = [160.0, 50.0, 160.0];
/// `JA_SE_LK_FG_ROOM_WARP_OUT` (`JAZelAudio_SE.h:584`).
const SE_WARP_OUT: u32 = 0x288A;

/// `enum { EVENT_WARP_START, EVENT_WARP_APPEAR, EVENT_WARP_MAKE }` (`d_a_ygcwp.cpp:12-16`), the
/// `actIdx` returned by `dComIfGp_evmng_getMyActIdx` (`:149`) - the position of the running cut's
/// name in `ACT_NAMES`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Act {
    WarpStart,
    WarpAppear,
    WarpMake,
}

/// `dComIfGp_evmng_getMyActIdx(mStaffId, M_act_table, ARRAY_SIZE(M_act_table), FALSE, 0)`
/// (`d_a_ygcwp.cpp:149`): the currently running cut's name matched against `ACT_NAMES`. A cut with
/// no matching name, or none running, reaches no `case` of either `switch`.
fn act_of(list: &EventList, cut: usize) -> Option<Act> {
    let name = list.cut_name(cut)?;
    let index = ACT_NAMES.iter().position(|entry| *entry == name)?;
    Some(match index {
        0 => Act::WarpStart,
        1 => Act::WarpAppear,
        _ => Act::WarpMake,
    })
}

/// The Ganon's Castle room-warp pad. `daYgcwp_c` (`d_a_ygcwp.h:8-36`) holds only a `J3DModel`, two
/// `mDoExt_brkAnm`, an event staff id, a timer, a BRK index and `m2D8`; there is no `dCcD` and
/// `d_a_ygcwp.cpp` never calls `dComIfG_Ccsp()`, so it registers no AT or Tg collider, cannot be
/// damaged and cannot damage anything. The profile `Group` is `fopAc_ACTOR_e` (`:255`), so neither
/// `combat::HitPoints` nor `enemy::Hurt` is attached.
///
/// `_create` (`:81-105`) sets `rt = cPhs_ERROR_e` and only overwrites it with `cPhs_COMPLEATE_e`
/// when `fopAcM_entrySolidHeap` succeeded (`:86`, `:101`), so it never refuses on its parameters
/// and there is no spawn-then-despawn path. `Mthd_IsDelete` (`:227-230`) is a bare TRUE - an
/// accepted delete request, not a self-destruct.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ygcwp {
    /// `mTimer` (`d_a_ygcwp.h:33`), written by `set_timer` (`:139-141`) and decremented at `:176`.
    pub timer: i32,
}

impl Ygcwp {
    /// `daYgcwp_c::_create` (`d_a_ygcwp.cpp:81-105`), minus the heap, model, BRK and matrix work.
    /// `off_ev()` (`:99`) is the only retained line of it: it clears `UNK_0480` unconditionally
    /// (`:119-121`). `check_ev()` (`:93-95`), `m2D8` (`:89-91`), both `setPlaySpeed` calls
    /// (`:92`, `:94`, `:96`) and `mCurBrk` (`:97`) only feed the omitted BRK animation.
    pub fn spawn(flags: &mut EventFlags) -> Self {
        flags.clear(TMP_BIT);
        Self { timer: 0 }
    }

    /// `fopAcM_setCullSizeBox(this, -160.0f, -10.0f, -160.0f, 160.0f, 50.0f, 160.0f)`
    /// (`d_a_ygcwp.cpp:100`) - a real custom cull box.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `set_timer` (`d_a_ygcwp.cpp:137-142`): `mTimer = 0` then, when the staff declares an integer
    /// `Timer`, `mTimer = *a_intP`.
    fn set_timer(&mut self, list: &EventList, staff: usize) {
        self.timer = list.number(staff, TIMER_KEY).unwrap_or(0);
    }

    /// `daYgcwp_c::_execute` (`d_a_ygcwp.cpp:145-189`) minus `make_shine` (`:132-134`, `:153`),
    /// both `daPy_py_c` `noResetFlg0` calls (`:157`, `:163` - `Player` carries no such flag), every
    /// `mBrkAnm` call (`:92`, `:94`, `:96`, `:154-156`, `:166-168`, `:185-186`) and `mCurBrk`
    /// (`:156`, `:168`). The `mStaffId != -1` guard (`:148`) is `list.staff_of` returning `None`.
    fn step(
        &mut self,
        list: &EventList,
        active: &mut ActiveEvent,
        ev_flags: &mut EventFlags,
        events: &mut SimEvents,
    ) {
        let Some(staff) = list.staff_of(ARC_NAME) else {
            return;
        };
        let Some((cut, advance)) = active.staff_cut(list, ARC_NAME) else {
            return;
        };
        let Some(act) = act_of(list, cut) else {
            return;
        };
        if advance {
            match act {
                Act::WarpStart => {
                    self.set_timer(list, staff);
                    events.list.push(SimEvent::Se(SE_WARP_OUT));
                }
                Act::WarpAppear => self.set_timer(list, staff),
                Act::WarpMake => {}
            }
        }
        match act {
            Act::WarpStart | Act::WarpAppear => {
                self.timer -= 1;
                if self.timer <= 0 {
                    // `dComIfGp_evmng_cutEnd(mStaffId)` (`d_a_ygcwp.cpp:178`).
                    active.end_staff_cut(list, ev_flags, ARC_NAME);
                }
            }
            Act::WarpMake => {}
        }
    }
}

pub fn is_ygcwp(name: &str) -> bool {
    YGCWP_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut ev_flags: ResMut<EventFlags>,
    mut events: ResMut<SimEvents>,
    mut pads: Query<&mut Ygcwp>,
) {
    for mut pad in &mut pads {
        pad.step(&list, &mut active, &mut ev_flags, &mut events);
    }
}