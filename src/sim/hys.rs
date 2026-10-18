use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::arrow::{Arrow, Phase, TYPE_FIRE, TYPE_ICE, TYPE_LIGHT, TYPE_NORMAL};
use super::barricade::AT_FIRE_ARROW;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::hmlif::{ARROW_AT, AT_ICE_ARROW, AT_LIGHT_ARROW, AT_NORMAL_ARROW};
use super::walk;

/// The stage strings are `Hys` and `Hys2` (`d_stage.cpp:612-613`, both `fpcNm_Hys_e`). Both load the
/// `_arcname` `Hys` (`d_a_hys.cpp:12`); `mType` (`d_a_hys.cpp:81`), not the object name, picks the
/// variant - it is what `m_tg_r`, `m_bdlidx`, `m_btpidx`, `m_dzbidx` and `m_heapsize` are indexed
/// with (`:15-21`, `:60`, `:68`, `:82`, `:84`).
pub const HYS_ACTORS: [&str; 2] = ["Hys", "Hys2"];

/// `mType = fopAcM_GetParam(this) >> 8` (`d_a_hys.cpp:81`), narrowing into `u8 mType`
/// (`d_a_hys.h:39`).
const TYPE_SHIFT: u32 = 8;
/// `... as u8` - `d_a_hys.h:39` is `u8 mType`.
const TYPE_MASK: u32 = 0xFF;
/// `mSwitchNo = fopAcM_GetParam(this) & 0xFF` (`d_a_hys.cpp:104`).
const SWITCH_MASK: u32 = 0xFF;
/// `const f32 daHys_c::m_tg_r[2] = {35.0f, 70.0f}` (`d_a_hys.cpp:21`), installed by
/// `mSph.SetR(m_tg_r[mType])` (`:101`). It overwrites `l_sph_src`'s own radius 30.0f (`:48`)
/// unconditionally, so 30.0f never survives `Create`.
const TG_R: [f32; 2] = [35.0, 70.0];
/// `if (field_0x458 < 3) field_0x458++` (`d_a_hys.cpp:188-190`) and the `field_0x458 = 3` seed
/// (`:108`).
const FRAME_MAX: u8 = 3;
/// `fopAcM_setCullSizeBox(this, -240.0f, -240.0f, -90.0f, ...)` (`d_a_hys.cpp:96`).
const CULL_MIN: [f32; 3] = [-240.0, -240.0, -90.0];
/// `..., 240.0f, 240.0f, 90.0f)` (`d_a_hys.cpp:96`).
const CULL_MAX: [f32; 3] = [240.0, 240.0, 90.0];
/// `JA_SE_OBJ_ARROW_SW_ON` (`JAZelAudio_SE.h:1878`).
const SE_ARROW_SW_ON: u32 = 0x695E;

/// `field_0x450` (`d_a_hys.h:36`) indexes `mode_proc[2] = { &mode_wait, &mode_sw_on }`
/// (`d_a_hys.cpp:147-150`). Only `0` and `1` are ever written (`:107`, `:179`, `:196`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// `mode_proc[0] = &daHys_c::mode_wait` (`d_a_hys.cpp:148`).
    Wait,
    /// `mode_proc[1] = &daHys_c::mode_sw_on` (`d_a_hys.cpp:149`).
    SwOn,
}

/// The eye switch: a floating eyeball that a fired arrow flips on.
///
/// `l_sph_src` (`d_a_hys.cpp:23-50`) has `SrcObjAt Type = 0` (`:29`) and `Flags = 0` (`:26`), so the
/// eyeball deals no damage and has no attack collider of its own. Its `SrcObjTg Type`
/// (`:30`) is the four arrow bits, which is a switch trigger, not a health path: nothing routes
/// damage through it. `daHys_c` (`d_a_hys.h:7-41`) has no health field and the profile `Group` is
/// `fopAc_ACTOR_e` (`d_a_hys.cpp:254`), so neither `combat::HitPoints` nor `enemy::Hurt` is
/// attached.
///
/// `Mthd_Execute` is `MoveBGExecute()` (`d_a_hys.cpp:225`), which forwards to the virtual
/// `Execute(Mtx**)` (`:131`) after `Execute` returns - so `Execute` is the real per-tick body and
/// `*mtx = &mBgMtx` (`:140`) plus the matrix block (`:134-138`) are `dBgW` work. `_create` (`:78-91`)
/// can only return `cPhs_ERROR_e` from `MoveBGCreate` on a heap or `dComIfG_Bgsp()` registration
/// failure (`d_bg_s_movebg_actor.cpp:60-66`), never from `mType`, and `Create` (`:94-119`) ends on an
/// unconditional TRUE, so there is no spawn-then-despawn path. `daHys_IsDelete` (`:229-231`) is a
/// bare TRUE - an accepted delete request, not a self-destruct.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hys {
    /// `mSwitchNo` (`d_a_hys.h:37`), `fopAcM_GetParam(this) & 0xFF` (`d_a_hys.cpp:104`).
    pub switch_no: u8,
    /// `mType` (`d_a_hys.h:39`), `fopAcM_GetParam(this) >> 8` narrowed to `u8` (`d_a_hys.cpp:81`).
    pub kind: u8,
    /// `field_0x450` (`d_a_hys.h:36`), the `mode_proc` index.
    pub mode: Mode,
    /// `field_0x458` (`d_a_hys.h:38`) - the BTP frame counter, seeded to 3 or 0 by `Create`
    /// (`d_a_hys.cpp:106-112`), decremented by `mode_wait` (`:171-173`) and clamped by `mode_sw_on`
    /// (`:188-190`).
    pub frame: u8,
}

impl Hys {
    /// `daHys_c::Create` (`d_a_hys.cpp:94-119`), minus `fopAcM_SetMtx` (`:95`), the
    /// `mStts`/`mSph` registration (`:98-102`) and `set_mtx` (`:103`). The `fopAcM_isSwitch` branch
    /// (`:106-112`) sets `field_0x450` and `field_0x458`; `mType` and `mSwitchNo` come from
    /// `_create` (`:81`) and `Create` (`:104`).
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = (parameters & SWITCH_MASK) as u8;
        let set = switches.test(switch_no);
        Self {
            switch_no,
            kind: ((parameters >> TYPE_SHIFT) & TYPE_MASK) as u8,
            mode: if set { Mode::SwOn } else { Mode::Wait },
            frame: if set { FRAME_MAX } else { 0 },
        }
    }

    /// `fopAcM_setCullSizeBox(this, -240.0f, -240.0f, -90.0f, 240.0f, 240.0f, 90.0f)`
    /// (`d_a_hys.cpp:96`) - a real custom cull box.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `mSph.SetR(m_tg_r[mType])` (`d_a_hys.cpp:101`). `m_tg_r` holds exactly two entries
    /// (`d_a_hys.cpp:21`), so retail reads past it for any `mType` above 1 - including the
    /// `m_arcname[mType]` load at `:82`. No value is invented for that case and the arrow test is
    /// simply not evaluated.
    fn radius(&self) -> Option<f32> {
        TG_R.get(usize::from(self.kind)).copied()
    }

    /// `hmlif::ARROW_KIND_AT` (`hmlif.rs:83-88`) - `Arrow::kind` to the arrow's real `AT_TYPE_*`.
    /// A kind outside the table has no AT type at all, which fails every `ChkAtType` below.
    fn arrow_at(kind: u8) -> u32 {
        match kind {
            TYPE_NORMAL => AT_NORMAL_ARROW,
            TYPE_FIRE => AT_FIRE_ARROW,
            TYPE_ICE => AT_ICE_ARROW,
            TYPE_LIGHT => AT_LIGHT_ARROW,
            _ => 0,
        }
    }

    /// `mSph.ChkTgHit()` then the four `obj->ChkAtType(...)` probes
    /// (`d_a_hys.cpp:159-164`), against the sphere at `current.pos` (`:153`) with
    /// `m_tg_r[mType]`'s radius (`:101`).
    fn arrow_hit(&self, centre: Vec3, arrows: &Query<&Arrow>) -> bool {
        let Some(radius) = self.radius() else {
            return false;
        };
        arrows.iter().any(|shaft| {
            if shaft.phase != Phase::Flying {
                return false;
            }
            if Self::arrow_at(shaft.kind) & ARROW_AT == 0 {
                return false;
            }
            let travel = shaft.pos - shaft.speed;
            walk::swept(travel, shaft.pos, centre, radius)
        })
    }

    /// `mode_wait` (`d_a_hys.cpp:158-174`).
    fn mode_wait(
        &mut self,
        centre: Vec3,
        arrows: &Query<&Arrow>,
        switches: &mut Switches,
        events: &mut SimEvents,
    ) {
        if self.arrow_hit(centre, arrows) {
            // `mSph.ClrTgHit()` (`:165`) has no modelled state, then `mode_sw_on_init` (`:166`).
            self.mode_sw_on_init(switches, events);
            return;
        }
        if self.frame != 0 {
            self.frame -= 1;
        }
    }

    /// `mode_sw_on_init` (`d_a_hys.cpp:177-181`).
    fn mode_sw_on_init(&mut self, switches: &mut Switches, events: &mut SimEvents) {
        switches.set(self.switch_no);
        self.mode = Mode::SwOn;
        events.list.push(SimEvent::Se(SE_ARROW_SW_ON));
    }

    /// `mode_sw_on` (`d_a_hys.cpp:184-192`).
    fn mode_sw_on(&mut self, switches: &Switches) {
        if !switches.test(self.switch_no) {
            // `mode_wait_init` (`d_a_hys.cpp:195-197`) is a bare `field_0x450 = 0`.
            self.mode = Mode::Wait;
        } else if self.frame < FRAME_MAX {
            self.frame += 1;
        }
    }

    /// `mode_proc_call` (`d_a_hys.cpp:145-155`) minus `mSph.SetC(current.pos)` and
    /// `dComIfG_Ccsp()->Set(&mSph)` (`:153-154`), which only register the sphere - `centre` already
    /// carries `current.pos`.
    fn step(
        &mut self,
        centre: Vec3,
        arrows: &Query<&Arrow>,
        switches: &mut Switches,
        events: &mut SimEvents,
    ) {
        match self.mode {
            Mode::Wait => self.mode_wait(centre, arrows, switches, events),
            Mode::SwOn => self.mode_sw_on(switches),
        }
    }
}

pub fn is_hys(name: &str) -> bool {
    HYS_ACTORS.contains(&name)
}

pub fn control(
    arrows: Query<&Arrow>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut eyes: Query<(&Actor, &mut Hys)>,
) {
    for (actor, mut eye) in &mut eyes {
        eye.step(actor.pos, &arrows, &mut switches, &mut events);
    }
}
