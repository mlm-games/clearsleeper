use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;

pub const MSDANSB_ACTORS: [&str; 1] = ["MsdanSb"];

const SIZE_MASK: u32 = 0x1; // <- d_a_obj_msdan_sub.h:10 (PRM_SIZE_W), d_a_obj.h:14
const SIZE_SHIFT: u32 = 0x10; // <- d_a_obj_msdan_sub.h:11 (PRM_SIZE_H)
const SWSAVE_MASK: u32 = 0xFF; // <- d_a_obj_msdan_sub.h:13 (PRM_SWSAVE_W), d_a_obj.h:14
const SWSAVE_SHIFT: u32 = 0x00; // <- d_a_obj_msdan_sub.h:14 (PRM_SWSAVE_S)
const OBJNO_MASK: u32 = 0xFF; // <- d_a_obj_msdan_sub.h:16 (PRM_OBJNO_W), d_a_obj.h:14
const OBJNO_SHIFT: u32 = 0x08; // <- d_a_obj_msdan_sub.h:17 (PRM_OBJNO_S)

const RUNG_DROP: f32 = 800.0; // <- d_a_obj_msdan_sub.cpp:28 (home.pos.y - 800.0f)
const RUNG_RISE: f32 = 25.0; // <- d_a_obj_msdan_sub.cpp:28 ((prm_get_objNo() + 1) * 25.0f)
const FALL_STEP: f32 = 10.0; // <- d_a_obj_msdan_sub.cpp:97 (m2E0 -= 10.0f)
const RISE_STEP: f32 = 15.0; // <- d_a_obj_msdan_sub.cpp:157 (m2E0 += 15.0f)

const TOP_STEP: i32 = 31; // <- d_a_obj_msdan_sub.cpp:96 (mCurObjNo < 31)
const MID_STEP: i32 = 16; // <- d_a_obj_msdan_sub.cpp:38 (mCurObjNo = 16)
const SHOCK_STEP: i32 = 20; // <- d_a_obj_msdan_sub.cpp:106 (mCurObjNo <= 20)

const SE_STAIR_ON: u32 = 0x69BB; // <- JAZelAudio_SE.h:1964 (JA_SE_OBJ_SW_STAIR_ON_1)
const SE_STAIR_OFF: u32 = 0x69BC; // <- JAZelAudio_SE.h:1965 (JA_SE_OBJ_SW_STAIR_OFF_1)

pub const CULL_MIN: [f32; 3] = [-1000.0, -1000.0, -1000.0]; // <- d_a_obj_msdan_sub.cpp:24
pub const CULL_MAX: [f32; 3] = [1000.0, 1000.0, 1000.0]; // <- d_a_obj_msdan_sub.cpp:24

/// One step of the `Msdan` staircase. The stage string is `MsdanSb`
/// (`d_stage.cpp:954`, `fpcNm_Obj_MsdanSub_e`) while the model and DZB archive are `Msdan`
/// (`daObjMsdanSub::Act_c::M_arcname`, `:10`); `d_stage.cpp:953` registers `Msdan` itself under
/// the separate `fpcNm_Obj_Msdan_e` process, and `MsdanS2` (`:956`) is `Msdan2`'s sibling, not
/// this actor.
///
/// `Mthd_Execute` (`:209-211`) is `MoveBGExecute()` (`d_bg_s_movebg_actor.cpp:86-112`), which
/// forwards to this class's virtual `Execute(Mtx**)` - so `Execute` (`:90-184`) is the real
/// per-tick body and `*pMtx = &M_tmp_mtx` (`:182`) plus `set_mtx` (`:75-80`) are the
/// moving-background matrix that the contract lists as confirmed-absent. `Mthd_Create`
/// (`:51-60`) can only return `cPhs_ERROR_e` from `MoveBGCreate` on a heap or `dComIfG_Bgsp()`
/// registration failure (`d_bg_s_movebg_actor.cpp:60-71`), never from its parameters, so there
/// is no data-driven refusal to model and `Create()` (`:22-48`) always returns TRUE.
///
/// `init_mtx` (`:83-87`) scales by `1.01f` and `MoveBGCreate` builds `mBgMtx` from
/// `shape_angle` / `scale`; both are renderer and moving-background data.
///
/// `StartShock` (`:107`, `:146`) is controller rumble - confirmed-absent - but the `m2E4` latch
/// it sets and clears (`:108`, `:147`, `:170`) is real state and is kept.
///
/// The term has no collider of any kind: `CreateHeap` (`:14-19`) makes only a `J3DModel`, and
/// `MoveBGCreateHeap` registers a `dBgW` instead of a `dCcD`. Nothing can damage a stair, so
/// neither `HitPoints` nor `Hurt` is attached.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Msdansb {
    pub sized: bool,
    pub switch_no: u8,
    pub obj_no: i32,
    pub home_y: f32,
    pub floor: i32,
    pub step: i32,
    pub height: f32,
    pub rise: f32,
    pub shocked: bool,
}

impl Msdansb {
    /// `daObjMsdanSub::Act_c::Create` (`:22-48`). `home.pos` is the placement point, so at
    /// `Create` time - before `Execute` ever runs - `home.pos.y` is still the stage y and no
    /// separate home field is needed. `m2DC = home.pos.y` (`:42`) and `m2E0 = 0.0f` (`:43`)
    /// are the only remaining writes; `m2E4 = false` (`:46`).
    pub fn spawn(parameters: u32, home_y: f32, switches: &Switches) -> Self {
        let sized = (parameters >> SIZE_SHIFT) & SIZE_MASK != 0;
        let switch_no = ((parameters >> SWSAVE_SHIFT) & SWSAVE_MASK) as u8;
        let obj_no = ((parameters >> OBJNO_SHIFT) & OBJNO_MASK) as i32;
        let raised = switches.test(switch_no);
        let step = if raised {
            TOP_STEP
        } else if sized {
            0
        } else {
            MID_STEP
        };
        Self {
            sized,
            switch_no,
            obj_no,
            home_y,
            floor: if sized { 0 } else { MID_STEP },
            step,
            height: home_y,
            rise: 0.0,
            shocked: false,
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `home.pos.y - 800.0f + (n + 1) * 25.0f` (`:28`, `:102`, `:173`).
    fn rung(&self, step: i32) -> f32 {
        self.home_y - RUNG_DROP + (step as f32 + 1.0) * RUNG_RISE
    }

    /// `home.pos.y + ((25.0f * (n + 1)) - 800.0f) / 2.0f` (`:106`, `:145`).
    fn half_rung(&self, step: i32) -> f32 {
        self.home_y + ((RUNG_RISE * (step as f32 + 1.0) - RUNG_DROP) / 2.0)
    }

    /// `((home.pos.y * 2.0f - 800.0f) + (n + 1) * 25.0f) / 2.0f` (`:115`).
    fn mid_rung(&self, step: i32) -> f32 {
        (self.home_y * 2.0 - RUNG_DROP + (step as f32 + 1.0) * RUNG_RISE) / 2.0
    }

    /// `(mCurObjNo <= 20 || (mCurObjNo & 1) == 1)` (`:106`, `:145`).
    fn rung_odd(step: i32) -> bool {
        step <= SHOCK_STEP || (step & 1) == 1
    }

    fn latch_shock(&mut self, step: i32) {
        if !self.shocked && self.height <= self.half_rung(step) && Self::rung_odd(step) {
            self.shocked = true;
        }
    }

    /// `Execute`'s `prm_get_size()` arm (`:96-121`).
    fn descend_sized(&mut self, actor: &mut Actor) -> Option<u32> {
        if self.rise > 0.0 {
            self.rise = 0.0;
        }
        if self.step >= TOP_STEP {
            return None;
        }
        self.rise -= FALL_STEP;
        self.height += self.rise;
        if self.step == self.obj_no {
            actor.pos.y = self.height;
        }
        if self.height <= self.rung(self.step) {
            actor.pos.y = self.rung(self.step);
            self.step += 1;
            self.latch_shock(self.step);
            self.rise = 0.0;
            Some(SE_STAIR_ON)
        } else if self.height <= self.mid_rung(self.step) {
            self.step += 1;
            self.rise = 0.0;
            self.height = self.home_y;
            None
        } else {
            None
        }
    }

    /// `Execute`'s `else if (fopAcM_isSwitch(this, prm_get_swSave()))` arm (`:128-149`).
    fn descend_plain(&mut self, actor: &mut Actor) -> Option<u32> {
        if self.rise > 0.0 {
            self.rise = 0.0;
        }
        if self.step >= TOP_STEP {
            return None;
        }
        self.rise -= FALL_STEP;
        self.height += self.rise;
        if self.step == self.obj_no {
            actor.pos.y = self.height;
        }
        let mut sound = None;
        if self.height <= self.rung(self.step) {
            if self.step == self.obj_no {
                actor.pos.y = self.rung(self.step);
                sound = Some(SE_STAIR_ON);
            }
            self.step += 1;
            self.rise = 0.0;
            self.height = self.home_y;
        }
        if self.step == self.obj_no {
            self.latch_shock(self.step);
        }
        sound
    }

    /// `Execute`'s final `else` arm (`:152-178`).
    fn ascend(&mut self, actor: &mut Actor) -> Option<u32> {
        if self.rise < 0.0 {
            self.rise = 0.0;
        }
        if self.step < self.floor {
            return None;
        }
        self.rise += RISE_STEP;
        self.height += self.rise;
        if self.step == self.obj_no {
            actor.pos.y = self.height;
        }
        if self.height < self.home_y {
            return None;
        }
        if self.step == self.obj_no {
            actor.pos.y = self.home_y;
        }
        let mut sound = None;
        if self.step > self.floor {
            if self.step == self.obj_no {
                sound = Some(SE_STAIR_OFF);
                self.shocked = false;
            }
            self.step -= 1;
            self.height = self.rung(self.step);
        } else {
            self.height = self.home_y;
        }
        self.rise = 0.0;
        sound
    }

    fn execute(&mut self, actor: &mut Actor, switches: &Switches) -> Option<u32> {
        if self.step == TOP_STEP {
            actor.pos.y = self.rung(TOP_STEP);
        }
        let raised = switches.test(self.switch_no);
        if self.sized {
            if raised {
                return self.descend_sized(actor);
            }
            return None;
        }
        if raised {
            return self.descend_plain(actor);
        }
        self.ascend(actor)
    }
}

pub fn is_msdansb(name: &str) -> bool {
    MSDANSB_ACTORS.contains(&name)
}

pub fn control(
    switches: Res<Switches>,
    mut events: ResMut<SimEvents>,
    mut stairs: Query<(&mut Actor, &mut Msdansb)>,
) {
    for (mut actor, mut stair) in &mut stairs {
        if let Some(sound) = stair.execute(&mut actor, &switches) {
            events.list.push(SimEvent::Se(sound));
        }
    }
}