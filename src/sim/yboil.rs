use bevy_ecs::prelude::*;

use super::flags::{EventFlags, Switches};

pub const YBOIL_ACTORS: [&str; 1] = ["Yboil00"];

const MODEL_COUNT: usize = 50; // <- d_a_obj_Yboil.cpp:31, d_a_obj_Yboil.h:32
/// `dSv_event_flag_c::UNK_1902` (`d_save_event_flag.inc:195`), byte-indexed per `d_save.h:722`.
const EVENT_MELT_FIRE: u32 = 0x1902; // <- d_a_obj_Yboil.cpp:89
const SWSAVE_SHIFT: u32 = 8; // <- d_a_obj_Yboil.h:14 (PRM_SWSAVE_S)
const SWSAVE_MASK: u32 = 0xFF; // <- d_a_obj_Yboil.h:15 (PRM_SWSAVE_W = 8, d_a_obj.h:14)
const CULL_MIN: [f32; 3] = [-3000.0, 0.0, -3000.0]; // <- d_a_obj_Yboil.cpp:65
const CULL_MAX: [f32; 3] = [3000.0, 200.0, 3000.0]; // <- d_a_obj_Yboil.cpp:65

/// The 50 boiling-surface puff models that surround a `Yboil00` term. The stage string is
/// `Yboil00` (`d_stage.cpp:997`); the archive is also `"Yboil"` (`d_a_obj_Yboil.cpp:22`), so
/// `_arcname` and the STAGE string agree.
///
/// `_create` (`:85-102`) returns `cPhs_STOP_e` when the island-melt event bit is already up,
/// which cancels the request inside Create exactly like `cPhs_ERROR_e`
/// (`f_pc_create_req.cpp:106-108`), so `refused` records that and the term is dropped on its
/// first tick. `IsDelete` is the bare `TRUE` of `:194-197`, i.e. "accepts a delete request",
/// never a self-destruct (RULE 6b).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Yboil {
    pub refused: bool,
    pub switch_no: u8,
    timer: [i32; MODEL_COUNT],
}

impl Yboil {
    pub fn spawn(parameters: u32, events: &EventFlags) -> Self {
        Self {
            refused: events.test(EVENT_MELT_FIRE),
            switch_no: ((parameters >> SWSAVE_SHIFT) & SWSAVE_MASK) as u8,
            timer: [0; MODEL_COUNT],
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    /// `_execute` (`d_a_obj_Yboil.cpp:115-141`). `pos_reset` (`:48-55`) and `set_mtx`
    /// (`:77-83`) only write `mMdlPos`/`mMdlScale`, which `_draw` (`:173-186`) turns into a
    /// matrix, so both are renderer work. The `mMdlTimer[i] == 0` arm (`:117-130`) needs the
    /// `Yboil00` BCK frame count: `mBckAnm[i].play()` runs `mFrameCtrl->update()` and returns
    /// `isStop()` (`m_Do_ext.cpp:116-120`), the loop bound comes from
    /// `initPlay(mpAnm->getFrameMax(), ...)` in `mDoExt_bckAnm::init` (`m_Do_ext.cpp:454`), and
    /// no BCK archive ships in this checkout.
    /// The seed `mMdlTimer[i] = 1 + (s32)cM_rndF(3.9999f)` is therefore omitted, which leaves
    /// every slot at the `0` `CreateInit` wrote (`:71`) and makes this loop inert - it is kept
    /// in the transcribed form so the decomp's shape and the switch gate survive, and it starts
    /// running the day the BCK frame count becomes known. `fopAcM_isSwitch` uses the raw
    /// `prm_get_swSave` index (`:132`); a `swSave` of 0x80 or above reads false here because
    /// `Switches` only models the in-memory 0..128 array (RULE 11b).
    fn step(&mut self, switches: &Switches) {
        for index in 0..MODEL_COUNT {
            if self.timer[index] == 0 {
                continue;
            }
            if switches.test(self.switch_no) {
                continue;
            }
            self.timer[index] -= 1;
        }
    }
}

pub fn is_yboil(name: &str) -> bool {
    YBOIL_ACTORS.contains(&name)
}

pub fn control(
    switches: Res<Switches>,
    mut boils: Query<(Entity, &mut Yboil)>,
    mut commands: Commands,
) {
    for (entity, mut boil) in &mut boils {
        if boil.refused {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        boil.step(&switches);
    }
}