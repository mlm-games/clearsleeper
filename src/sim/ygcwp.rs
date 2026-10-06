use bevy_ecs::prelude::*;

use super::events::{SimEvent, SimEvents};
use super::event::{ActiveEvent, EventList};
use super::flags::EventFlags;

/// The stage string is `Ygcwp` (`d_stage.cpp:1181`, `fpcNm_Ygcwp_e`) and it is also the
/// `_arcname` / staff name (`d_a_ygcwp.cpp:28`, `:83`, `:147`), so the two agree.
pub const YGCWP_ACTORS: [&str; 1] = ["Ygcwp"];

/// `enum { EVENT_WARP_START, EVENT_WARP_APPEAR, EVENT_WARP_MAKE }` (`d_a_ygcwp.cpp:12-16`),
/// indexed by `M_act_table[]` (`d_a_ygcwp.cpp:36-40`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// `EVENT_WARP_START` (`d_a_ygcwp.cpp:13`), `"warp_start"` (`:37`).
    Start,
    /// `EVENT_WARP_APPEAR` (`:14`), `"warp_appear"` (`:38`).
    Appear,
    /// `EVENT_WARP_MAKE` (`:15`), `"warp_make"` (`:39`).
    Make,
}

/// `static char* M_act_table[]` (`d_a_ygcwp.cpp:36-40`) - the retail name the event staff uses.
pub const ACT_TABLE: [&str; 3] = ["warp_start", "warp_appear", "warp_make"];
/// `dComIfGp_evmng_getMyStaffId(M_arcname)` (`d_a_ygcwp.cpp:147`) looks the staff up by the
/// `_arcname` string, so that is the staff name.
pub const STAFF_NAME: &str = "Ygcwp";
/// `dComIfGp_evmng_getMyIntegerP(mStaffId, "Timer")` (`d_a_ygcwp.cpp:138`).
const TIMER_KEY: &str = "Timer";
/// `JA_SE_LK_FG_ROOM_WARP_OUT` (`JAZelAudio_SE.h:584`).
const SE_ROOM_WARP_OUT: u32 = 0x288A;
/// `dSv_event_tmp_flag_c::UNK_0480` (`d_save_event_tmp_flag.inc:33`), read by `check_ev` and
/// cleared by `off_ev` (`d_a_ygcwp.cpp:115`, `:120`).
const EVENT_TMP_0480: u32 = 0x0480;
/// `fopAcM_setCullSizeBox(this, -160.0f, -10.0f, -160.0f, 160.0f, 50.0f, 160.0f)`
/// (`d_a_ygcwp.cpp:100`).
const CULL_MIN: [f32; 3] = [-160.0, -10.0, -160.0];
const CULL_MAX: [f32; 3] = [160.0, 50.0, 160.0];

/// The Ganon's Castle room-warp swirl. The staff drives it entirely: `dComIfGp_evmng_getMyStaffId`
/// (`d_a_ygcwp.cpp:147`) finds the `Ygcwp` staff, `getMyActIdx` (`:149`) resolves the current cut
/// name against `ACT_TABLE`, and `getIsAddvance` (`:150`) fires the once-per-cut effects. There is
/// no collider anywhere in `daYgcwp_c` (`d_a_ygcwp.h:8-36`) and no `dComIfG_Ccsp()` call in
/// `d_a_ygcwp.cpp`, so it can neither be hit nor hit anything; the profile `Group` is
/// `fopAc_ACTOR_e` (`:253`).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ygcwp {
    /// `s32 mTimer` (`d_a_ygcwp.h:33`), set by `set_timer` (`:137-142`) and counted down at
    /// `:176-178`.
    pub timer: i32,
}

impl Ygcwp {
    /// `daYgcwp_c::_create` (`d_a_ygcwp.cpp:81-105`): `check_ev()` (`:93`) decides
    /// `mBrkAnm[0]`'s play speed and `mCurBrk` (`:94-98`) and `off_ev()` (`:99`) clears the
    /// temporary event bit. Both branches resolve to `mTimer = 0` because `set_timer` only runs
    /// from `_execute` (`:158`, `:162`), so nothing else touches the surviving state.
    ///
    /// `m2D8 = fopAcM_GetParam(this) & 0x0F` clamped to `0..=2` (`:89-91`) is decoded in the
    /// decomp but never read again in the file, so it is not stored (RULE 7).
    ///
    /// `_create` starts at `cPhs_ERROR_e` (`:85`) and only reaches `cPhs_COMPLEATE_e` when
    /// `fopAcM_entrySolidHeap` succeeds (`:86-102`); `create_heap` (`:48-78`) returns `FALSE`
    /// only on a missing BDL or a failed `mBrkAnm.init`, i.e. an archive-load failure and not a
    /// parameter, so `spawn` returns `Self` rather than `Option<Self>`.
    pub fn spawn(flags: &mut EventFlags) -> Self {
        // `off_ev()` (`d_a_ygcwp.cpp:99`, `:118-121`) clears
        // `dSv_event_tmp_flag_c::UNK_0480`, which `dComIfGs_offTmpBit` (`d_com_inf_game.h:1379`)
        // writes into `dSv_event_c::mFlags[0x100]` at `id >> 8` / `id & 0xFF` (`d_save.cpp:1186-1204`)
        // - the `flags::EventFlags` channel (`kalle_demos.rs:1457`, `:2600`). `check_ev()` (`:93`)
        // only chooses the play speed of `mBrkAnm[0]` and `mCurBrk` (`:94-98`), both animator state,
        // so its result is not kept.
        flags.clear(EVENT_TMP_0480);
        Self { timer: 0 }
    }

    /// `fopAcM_setCullSizeBox(this, -160.0f, -10.0f, -160.0f, 160.0f, 50.0f, 160.0f)`
    /// (`d_a_ygcwp.cpp:100`) - a real custom cull box.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `daYgcwp_c::_execute` (`d_a_ygcwp.cpp:145-189`) minus `make_shine` (`:132-134`), the four
    /// `mBrkAnm` animator calls (`:92`, `:155`, `:165-166`, `:186`), the `mCurBrk` writes
    /// (`:156`, `:168`) and the two `player->on/offNoResetFlg0(daPyFlg0_NO_DRAW)` calls
    /// (`:157`, `:163`).
    fn step(
        &mut self,
        list: &EventList,
        active: &ActiveEvent,
        flags: &mut EventFlags,
        events: &mut SimEvents,
    ) {
        // `mStaffId = dComIfGp_evmng_getMyStaffId(M_arcname)` (`:147`) and the
        // `if (mStaffId != -1)` guard (`:148`): a staff with no live cut leaves this actor alone.
        let Some((cut, advance)) = active.staff_cut(list, STAFF_NAME) else {
            return;
        };
        // `dComIfGp_evmng_getMyActIdx(mStaffId, M_act_table, 3, FALSE, 0)` (`:149`).
        let act = act_index(list.cut_name(cut).unwrap_or(""));
        // `if (dComIfGp_evmng_getIsAddvance(mStaffId))` (`:150`).
        if advance {
            match act {
                Some(Action::Start) => {
                    self.set_timer(list, cut);
                    events.list.push(SimEvent::Se(SE_ROOM_WARP_OUT));
                }
                Some(Action::Appear) => self.set_timer(list, cut),
                Some(Action::Make) => {}
                None => {}
            }
        }
        match act {
            Some(Action::Start) | Some(Action::Appear) => {
                self.timer -= 1;
                if self.timer <= 0 {
                    // `dComIfGp_evmng_cutEnd(mStaffId)` (`:178`).
                    active.end_staff_cut(list, flags, STAFF_NAME);
                }
            }
            Some(Action::Make) | None => {}
        }
    }

    /// `daYgcwp_c::set_timer` (`d_a_ygcwp.cpp:137-142`): `mTimer = 0` then, if the cut carries a
    /// `Timer` integer, `mTimer = *a_intP`.
    fn set_timer(&mut self, list: &EventList, cut: usize) {
        self.timer = 0;
        if let Some(value) = list.cut_number(cut, TIMER_KEY) {
            self.timer = value;
        }
    }
}

/// `dComIfGp_evmng_getMyActIdx(mStaffId, M_act_table, ARRAY_SIZE(M_act_table), FALSE, 0)`
/// (`d_a_ygcwp.cpp:149`): the position of the current cut name in `ACT_TABLE`, or `-1`.
fn act_index(name: &str) -> Option<Action> {
    match ACT_TABLE.iter().position(|entry| *entry == name)? {
        0 => Some(Action::Start),
        1 => Some(Action::Appear),
        _ => Some(Action::Make),
    }
}

/// `daYgcwp_c::check_ev` (`d_a_ygcwp.cpp:113-116`) and `off_ev` (`:118-121`) both run once from
/// `_create` (`:93`, `:99`) and touch only the temporary event bit `UNK_0480`, which `spawn`
/// clears. `check_ev`'s read result feeds `mBrkAnm[0]`'s play speed and `mCurBrk` (`:94-98`),
/// both omitted animator state.
pub fn is_ygcwp(name: &str) -> bool {
    YGCWP_ACTORS.contains(&name)
}

pub fn control(
    list: Res<EventList>,
    mut active: ResMut<ActiveEvent>,
    mut flags: ResMut<EventFlags>,
    mut events: ResMut<SimEvents>,
    mut warps: Query<&mut Ygcwp>,
) {
    for mut warp in &mut warps {
        warp.step(&list, &active, &mut flags, &mut events);
    }
}