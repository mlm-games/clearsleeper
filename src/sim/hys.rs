use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::arrow::{Arrow, Phase};
use super::barricade::AT_FIRE_ARROW;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::hmlif::{ARROW_KIND_AT, AT_ICE_ARROW, AT_LIGHT_ARROW, AT_NORMAL_ARROW};
use super::walk;

/// The stage strings are `Hys` and `Hys2` (`d_stage.cpp:612-613`, both `fpcNm_Hys_e`). The
/// `_arcname` table is `{"Hys", "Hys"}` (`d_a_hys.cpp:12`), so `Hys2` loads the same archive with
/// `mType == 1`; the STAGE strings are what belong in this array.
pub const HYS_ACTORS: [&str; 2] = ["Hys", "Hys2"];

/// `const f32 daHys_c::m_tg_r[2] = { 35.0f, 70.0f }` (`d_a_hys.cpp:21`), reached through
/// `mSph.SetR(m_tg_r[mType])` (`:101`), which overwrites `l_sph_src`'s `Radius = 30.0f` (`:48`).
pub const TG_R: [f32; 2] = [35.0, 70.0];
/// `m_arcname[2]`, `m_bdlidx[2]`, `m_btpidx[2]`, `m_dzbidx[2]` and `m_heapsize[2]`
/// (`d_a_hys.cpp:12-20`) are all two-entry tables indexed by the unmasked `mType` (`:81`). Only
/// types 0 and 1 are reachable from the two stage strings, so the index is clamped to the table.
const TYPE_SLOTS: usize = 2;
/// `mType = fopAcM_GetParam(this) >> 8` (`d_a_hys.cpp:81`); there is no mask in the decomp.
const TYPE_SHIFT: u32 = 8;
/// `mSwitchNo = fopAcM_GetParam(this) & 0xFF` (`d_a_hys.cpp:104`).
const SWITCH_MASK: u32 = 0xFF;
/// `JA_SE_OBJ_ARROW_SW_ON` (`JAZelAudio_SE.h:1878`).
const SE_ARROW_SW_ON: u32 = 0x695E;
/// `fopAcM_setCullSizeBox(this, -240.0f, -240.0f, -90.0f, 240.0f, 240.0f, 90.0f)` (`d_a_hys.cpp:96`).
const CULL_MIN: [f32; 3] = [-240.0, -240.0, -90.0];
const CULL_MAX: [f32; 3] = [240.0, 240.0, 90.0];

/// `field_0x450` (`d_a_hys.h:36`) indexes `static const ModeFunc mode_proc[2]`
/// (`d_a_hys.cpp:147-150`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// `&daHys_c::mode_wait` (`d_a_hys.cpp:148`).
    Wait,
    /// `&daHys_c::mode_sw_on` (`d_a_hys.cpp:149`).
    SwOn,
}

/// The arrow target the eye switch listens for: the OR of the four `ChkAtType` arms at
/// `d_a_hys.cpp:161-164`, which is exactly `l_sph_src`'s `SrcObjTg Type`
/// (`AT_TYPE_NORMAL_ARROW | AT_TYPE_FIRE_ARROW | AT_TYPE_ICE_ARROW | AT_TYPE_LIGHT_ARROW`,
/// `d_a_hys.cpp:30`).
pub const SWITCH_ARROW_AT: u32 =
    AT_NORMAL_ARROW | AT_FIRE_ARROW | AT_ICE_ARROW | AT_LIGHT_ARROW;

/// The arrow switch that any of the four elemental arrows flips. It registers `l_sph_src`
/// (`d_a_hys.cpp:23-50`) with `SrcObjAt Type = 0` (`:26`) - no AT, so it cannot deal damage -
/// and a TG-only `SrcObjTg Type` of the four arrow AT types (`:30`). `daHys_c` carries no health
/// field (`d_a_hys.h:7-41`) and the profile `Group` is `fopAc_ACTOR_e` (`d_a_hys.cpp:254`), so an
/// arrow hit is a switch trigger and not an inbound damage path.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hys {
    /// `mSwitchNo` (`d_a_hys.h:37`), `fopAcM_GetParam(this) & 0xFF` (`d_a_hys.cpp:104`).
    pub switch_no: u8,
    /// `mType` (`d_a_hys.h:39`), `fopAcM_GetParam(this) >> 8` (`d_a_hys.cpp:81`).
    pub kind: u8,
    /// `field_0x450` (`d_a_hys.h:36`).
    pub mode: Mode,
}

impl Hys {
    /// `daHys_c::Create` (`d_a_hys.cpp:94-119`): `mSwitchNo` (`:104`) and the `fopAcM_isSwitch`
    /// branch that sets `field_0x450` (`:106-112`). `fopAcM_SetMtx`/`fopAcM_setCullSizeBox` (`:95-96`),
    /// `set_mtx()` (`:103`) and `scale.setall(2.0f)` (`:114-116`) are renderer work. The `mSph`
    /// registration (`:99-101`) is the TG trigger modelled by `arrow_hit` below. `field_0x458`
    /// (`:108`, `:111`, `:171-172`, `:188-189`) is the BTP texture-animation frame index and is
    /// read only by `Draw` (`:203`), so it is omitted.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = (parameters & SWITCH_MASK) as u8;
        let set = switches.test(switch_no);
        Self {
            switch_no,
            kind: (parameters >> TYPE_SHIFT) as u8,
            mode: if set { Mode::SwOn } else { Mode::Wait },
        }
    }

    /// `fopAcM_setCullSizeBox(this, -240.0f, -240.0f, -90.0f, 240.0f, 240.0f, 90.0f)`
    /// (`d_a_hys.cpp:96`) - a real custom cull box.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `daHys_c::Execute` (`d_a_hys.cpp:131-142`) minus the model matrix writes (`:134-138`) and
    /// the `*mtx = &mBgMtx` hand-off (`:140`); `mode_proc_call()` (`:132`) is the whole body.
    fn step(
        &mut self,
        pos: Vec3,
        arrows: &Query<&Arrow>,
        switches: &mut Switches,
        events: &mut SimEvents,
    ) {
        // `mode_proc_call` (`d_a_hys.cpp:145-155`) dispatches `mode_proc[field_0x450]` (`:152`)
        // and then re-centres and registers `mSph` (`:153-154`).
        let already_on = switches.test(self.switch_no);
        match self.mode {
            Mode::Wait => self.mode_wait(pos, arrows, switches, events),
            Mode::SwOn => self.mode_sw_on(already_on),
        }
    }

    /// `daHys_c::mode_wait` (`d_a_hys.cpp:158-174`) minus the `field_0x458` decrement (`:171-173`).
    fn mode_wait(
        &mut self,
        pos: Vec3,
        arrows: &Query<&Arrow>,
        switches: &mut Switches,
        events: &mut SimEvents,
    ) {
        // `mSph.ChkTgHit()` then the four `obj->ChkAtType(...)` arms (`d_a_hys.cpp:159-164`),
        // against the sphere at `mSph.SetC(current.pos)` (`:153`) with radius `m_tg_r[mType]`.
        if arrows.iter().any(|shaft| self.sphere_hit(pos, shaft)) {
            self.mode_sw_on_init(switches, events);
        }
    }

    /// `daHys_c::mode_sw_on_init` (`d_a_hys.cpp:177-181`).
    fn mode_sw_on_init(&mut self, switches: &mut Switches, events: &mut SimEvents) {
        switches.set(self.switch_no);
        self.mode = Mode::SwOn;
        events.list.push(SimEvent::Se(SE_ARROW_SW_ON));
    }

    /// `daHys_c::mode_sw_on` (`d_a_hys.cpp:184-192`) minus the `field_0x458` climb (`:188-190`).
    fn mode_sw_on(&mut self, switch_on: bool) {
        if !switch_on {
            self.mode_wait_init();
        }
    }

    /// `daHys_c::mode_wait_init` (`d_a_hys.cpp:195-197`).
    fn mode_wait_init(&mut self) {
        self.mode = Mode::Wait;
    }

    /// The `mSph` TG test: radius `m_tg_r[mType]` (`:101`) about `current.pos` (`:153`), gated on
    /// `ARROW_KIND_AT[kind]` (`hmlif.rs`) carrying one of the four arrow AT bits (`:30`).
    fn sphere_hit(&self, pos: Vec3, shaft: &Arrow) -> bool {
        if shaft.phase != Phase::Flying {
            return false;
        }
        let Some(at) = ARROW_KIND_AT.get(usize::from(shaft.kind)).copied() else {
            return false;
        };
        if at & SWITCH_ARROW_AT == 0 {
            return false;
        }
        let index = usize::from(self.kind).min(TYPE_SLOTS - 1);
        let travel = shaft.pos - shaft.speed;
        walk::swept(travel, shaft.pos, pos, TG_R[index])
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