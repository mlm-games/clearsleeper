use bevy_ecs::prelude::*;
use glam::Vec3;

use super::arrow::{Arrow, Phase};
use super::barricade::AT_FIRE_ARROW;
use super::events::{SimEvent, SimEvents};
use super::flags::Switches;
use super::hmlif::ARROW_KIND_AT;
use super::walk;

/// The stage string is `Qdghd` (`d_stage.cpp:965`, `fpcNm_Obj_Dragonhead_e`) and it is also the
/// `_arcname` (`d_a_obj_dragonhead.cpp:57`, `:71`, `:116`, `:140`), so the two agree.
pub const QDGHD_ACTORS: [&str; 1] = ["Qdghd"];

/// `mSphCenter.set(79338.0f, 950.0f, 220000.0f)` (`d_a_obj_dragonhead.cpp:100`) - an absolute
/// world point, not an offset from the actor.
const SPH_CENTRE: [f32; 3] = [79338.0, 950.0, 220000.0];
/// `sph_check_src` `cM3dGSphS.Radius` (`d_a_obj_dragonhead.cpp:42`).
const SPH_RADIUS: f32 = 1000.0;
/// `mAlpha = 0` (`d_a_obj_dragonhead.cpp:90`).
const ALPHA_OFF: u8 = 0;
/// `mAlpha = 0xFF` (`d_a_obj_dragonhead.cpp:93`).
const ALPHA_ON: u8 = 0xFF;
/// `mAlpha -= 2` (`d_a_obj_dragonhead.cpp:169`) and `mAlpha += 2` (`:180`).
const ALPHA_STEP: u8 = 2;
/// `if (mAlpha <= 0xFD) mAlpha += 2 else mAlpha = 0xFF` (`d_a_obj_dragonhead.cpp:179-181`).
const ALPHA_FLOOR: u8 = 0xFD;
/// `(fopAcM_GetParam(ac) >> 8) & 0xFF` (`d_a_obj_dragonhead.cpp:52`).
const SWITCH_SHIFT: u32 = 8;
/// `& 0xFF` (`d_a_obj_dragonhead.cpp:52`).
const SWITCH_MASK: u32 = 0xFF;
/// `JA_SE_OBJ_ICEBERG_BREATH` (`JAZelAudio_SE.h:1703`).
const SE_ICEBERG_BREATH: u32 = 0x61CC;
/// `JA_SE_OBJ_ICEVERG_MELT` (`JAZelAudio_SE.h:1981`).
const SE_ICEVERG_MELT: u32 = 0x69D0;

/// The Outset Island ice sculpture that melts when a fire arrow hits it. It registers
/// `sph_check_src` (`d_a_obj_dragonhead.cpp:17-44`) with `SrcObjAt Type = 0` (`:21`), so it deals
/// no damage and cannot be damaged; its TG-only `SrcObjTg Type = AT_TYPE_FIRE_ARROW` (`:24`) is a
/// switch trigger, not a health path. `daObjDragonhead_c` (`d_a_obj_dragonhead.h:9-31`) has no
/// health field and the profile `Group` is `fopAc_ACTOR_e` (`:252`), not `fopAc_ENEMY_e`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Qdghd {
    /// `(fopAcM_GetParam(this) >> 8) & 0xFF` (`d_a_obj_dragonhead.cpp:52`).
    pub switch_no: u8,
    /// `mSwitchOn` (`d_a_obj_dragonhead.h:26`), latched by the fire-arrow hit (`:153`).
    pub switch_on: bool,
    /// `mAlpha` (`d_a_obj_dragonhead.h:25`), the melt alpha read back by `_draw` (`:198`).
    pub alpha: u8,
    /// `field_0x40c` (`d_a_obj_dragonhead.h:28`) - whether the `dBgW` is still registered. The
    /// `Regist`/`Release` calls themselves are omitted (RULE 3b), but the flag's transitions are
    /// exactly the decomp's, because both calls succeed in the normal case.
    pub registered: bool,
}

impl Qdghd {
    /// `daObjDragonhead_c::CreateInit` (`d_a_obj_dragonhead.cpp:81-103`) minus `fopAcM_SetMtx`
    /// (`:82`), `fopAcM_setCullSizeFar` (`:83`), `set_mtx` (`:84`), the `mStts`/`mSph`
    /// registration (`:98-102`) and the `dBgW` `Regist` call itself (`:94`). The
    /// `dComIfGs_isSwitch` branch (`:88-96`) sets `mSwitchOn`, `mAlpha` and `field_0x40c`.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = ((parameters >> SWITCH_SHIFT) & SWITCH_MASK) as u8;
        let set = switches.test(switch_no);
        Self {
            switch_no,
            switch_on: set,
            alpha: if set { ALPHA_OFF } else { ALPHA_ON },
            registered: !set,
        }
    }

    /// `daObjDragonhead_c::_execute` (`d_a_obj_dragonhead.cpp:144-188`) minus the `dBgW`
    /// `Release` (`:161`) and `set_mtx` (`:186`).
    fn step(
        &mut self,
        arrows: &Query<&Arrow>,
        switches: &mut Switches,
        events: &mut SimEvents,
    ) {
        if !self.switch_on {
            events.list.push(SimEvent::Se(SE_ICEBERG_BREATH));
            if self.fire_arrow_hit(arrows) {
                switches.set(self.switch_no);
                self.switch_on = true;
            }
        }
        if switches.test(self.switch_no) {
            if self.registered {
                // `!dComIfG_Bgsp()->Release(mpBgW)` -> `field_0x40c = 0`
                // (`d_a_obj_dragonhead.cpp:161-162`).
                self.registered = false;
                events.list.push(SimEvent::Se(SE_ICEVERG_MELT));
            }
            if self.alpha != ALPHA_OFF {
                if self.alpha >= ALPHA_STEP {
                    self.alpha -= ALPHA_STEP;
                } else {
                    self.alpha = ALPHA_OFF;
                }
            }
        } else {
            self.switch_on = false;
            // `field_0x40c == 0 && !dComIfG_Bgsp()->Regist(mpBgW, this)` -> `field_0x40c = 1`
            // (`d_a_obj_dragonhead.cpp:175-176`); the `dBgW` itself is not modelled (RULE 3b).
            self.registered = true;
            if self.alpha < ALPHA_ON {
                if self.alpha <= ALPHA_FLOOR {
                    self.alpha += ALPHA_STEP;
                } else {
                    self.alpha = ALPHA_ON;
                }
            }
        }
    }

    /// `mSph.ChkTgHit()` then `at->ChkAtType(AT_TYPE_FIRE_ARROW)`
    /// (`d_a_obj_dragonhead.cpp:148-150`), against the sphere at `mSphCenter`
    /// (`d_a_obj_dragonhead.cpp:100-102`) with `sph_check_src`'s radius (`:42`).
    fn fire_arrow_hit(&self, arrows: &Query<&Arrow>) -> bool {
        arrows.iter().any(|shaft| {
            if shaft.phase != Phase::Flying {
                return false;
            }
            let Some(at) = ARROW_KIND_AT.get(usize::from(shaft.kind)).copied() else {
                return false;
            };
            if at & AT_FIRE_ARROW == 0 {
                return false;
            }
            let travel = shaft.pos - shaft.speed;
            walk::swept(travel, shaft.pos, Vec3::from_array(SPH_CENTRE), SPH_RADIUS)
        })
    }
}

/// The profile is `fopAc_CULLBOX_CUSTOM_e` (`d_a_obj_dragonhead.cpp:253`) but `CreateInit` only
/// calls `fopAcM_setCullSizeFar(this, 1.0f)` (`:83`) - there is no `fopAcM_setCullSizeBox` in this
/// file, so no cull box is derivable and none is invented.
pub fn is_qdghd(name: &str) -> bool {
    QDGHD_ACTORS.contains(&name)
}

pub fn control(
    arrows: Query<&Arrow>,
    mut switches: ResMut<Switches>,
    mut events: ResMut<SimEvents>,
    mut bergs: Query<&mut Qdghd>,
) {
    for mut berg in &mut bergs {
        berg.step(&arrows, &mut switches, &mut events);
    }
}