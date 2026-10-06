use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};

pub const QTKHD_ACTORS: [&str; 1] = ["Qtkhd"];

pub const SE_TOWER_ROUND: u32 = 0x619A; // <- JAZelAudio_SE.h:1697 (JA_SE_OBJ_TC_TOWER_ROUND)

pub const CULL_HALF: f32 = 750.0; // <- d_a_obj_smplbg.cpp:23 (mCullMaxX, the sphere radius)
pub const CULL_TOP: f32 = 3000.0; // <- d_a_obj_smplbg.cpp:19 (mCullMinY 2250 + radius 750)

const TYPE_SHIFT: u32 = 0; // <- d_a_obj_smplbg.h:13 (PRM_TYPE_S)
const TYPE_MASK: u32 = 0xFF; // <- d_a_obj_smplbg.h:12 (PRM_TYPE_W = 8, d_a_obj.h:14)
const TYPE_FOLD: i32 = 1; // <- d_a_obj_smplbg.cpp:61
const SPIN: i16 = 0x5B; // <- d_a_obj_smplbg.cpp:104
const EXEC_QTKHD: i32 = 0; // <- d_a_obj_smplbg.cpp:113 (exec_proc[0])

/// `daObjSmplbg::Act_c` (`d_a_obj_smplbg.cpp`), the head of Tingle Tower. `M_attr` (`:11-25`)
/// holds exactly one entry, so `attr()` is always `M_attr[0]` - that is what makes the `mType`
/// fold at `:61-63` safe. `Mthd_Create` (`:57-73`) registers a `dBgW` moving background
/// (`MoveBGCreate`, `:69`) and can return `cPhs_ERROR_e` from it; `dBgW` is a confirmed-absent
/// input, so the tower always creates here and there is no spawn-then-despawn path.
/// `Mthd_IsDelete` (`:161-163`) reaches `dBgS_MoveBgActor::IsDelete`, an accepted delete
/// request.
///
/// `Mthd_Create` writes only `mType` (`:60-63`); `mIsStop` is never written by this actor. Its
/// only writers are `daNpc_Tc_c::stopTower` (`d_a_npc_tc.cpp:740`, `:743` `onStop`) and
/// `startTower` (`:775`, `:780` `offStop`), so it is left clear here and exposed for the
/// caller to set.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Qtkhd {
    pub kind: i32,
    pub stopped: bool,
}

impl Qtkhd {
    /// `daObjSmplbg::Act_c::Mthd_Create` (`:60-63`).
    pub fn spawn(parameters: u32) -> Self {
        let raw = ((parameters >> TYPE_SHIFT) & TYPE_MASK) as i32;
        Self {
            kind: if raw >= TYPE_FOLD { 0 } else { raw },
            stopped: false,
        }
    }

    /// `daObjSmplbg::Act_c::exec_qtkhd` (`:102-107`).
    fn exec_qtkhd(&self, actor: &mut Actor, events: &mut SimEvents) {
        if self.stopped {
            return;
        }
        actor.angle[1] = actor.angle[1].wrapping_add(SPIN);
        events.list.push(SimEvent::Se(SE_TOWER_ROUND));
    }

    /// `daObjSmplbg::Act_c::Execute` (`:110-120`) dispatches `exec_proc[mType]`.
    fn step(&self, actor: &mut Actor, events: &mut SimEvents) {
        match self.kind {
            EXEC_QTKHD => self.exec_qtkhd(actor, events),
            _ => {}
        }
    }

    /// `Create` (`:46-48`) takes the `fopAc_CULLSPHERE_CUSTOM_e` branch because
    /// `attr().mFlags = 1|4|8` (`d_a_obj_smplbg.cpp:17`) has bit 4 set: centre `(0, 2250, 0)`
    /// (`:18-20`), radius `750` (`:21`). The `fopAcStts_NOCULLEXEC_e` clear at `:43-45` tests
    /// bit 8 of the same field; it is cull-execution state, not geometry.
    pub fn cull(&self) -> (f32, f32) {
        (CULL_HALF, CULL_TOP)
    }
}

pub fn is_qtkhd(name: &str) -> bool {
    QTKHD_ACTORS.contains(&name)
}

pub fn control(mut events: ResMut<SimEvents>, towers: Query<(&mut Actor, &Qtkhd)>) {
    for (mut actor, tower) in &towers {
        let slot = &mut *actor;
        tower.step(slot, &mut events);
    }
}