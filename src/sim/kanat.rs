use bevy_ecs::prelude::*;

use super::flags::Switches;

pub const KANAT_ACTORS: [&str; 1] = ["Kanat"];

pub const CULL_MIN: [f32; 3] = [-500.0, -100.0, -500.0]; // <- d_a_obj_kanat.cpp:27
pub const CULL_MAX: [f32; 3] = [500.0, 200.0, 500.0]; // <- d_a_obj_kanat.cpp:27

const SWSAVE_SHIFT: u32 = 0x00; // <- d_a_obj_kanat.h:14 (PRM_SWSAVE_S)
const SWSAVE_MASK: u32 = 0xFF; // <- d_a_obj_kanat.h:13 (PRM_SWSAVE_W = 0x08, d_a_obj.h:14)

/// The stage string and the model's `_arcname` are both `Kanat` (`d_stage.cpp:792`,
/// `d_a_obj_kanat.cpp:13`). `daObjKanat::Act_c::Mthd_Create` (`d_a_obj_kanat.cpp:35-49`) returns
/// `cPhs_STOP_e` when the vine's switch already reads on, and STOP cancels the request inside
/// Create exactly like ERROR (`f_pc_create_req.cpp:106-108`), so `refused` records that and the
/// term is dropped on its first tick. `mSmokeCb.isEnd()` (`:101`) belongs to the particle
/// system, so the `fopAcM_delete` behind it (`:102`) stays unported and the retired vine keeps
/// existing invisibly.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kanat {
    pub switch_no: u8,
    pub broken: bool,
    pub visible: bool,
    pub refused: bool,
}

impl Kanat {
    /// `prm_get_swSave` (`d_a_obj_kanat.h:20`) is `daObj::PrmAbstract(this, 0x08, 0x00)`.
    pub fn spawn(parameters: u32, switches: &Switches) -> Self {
        let switch_no = ((parameters >> SWSAVE_SHIFT) & SWSAVE_MASK) as u8;
        Self {
            switch_no,
            broken: false,
            visible: true,
            refused: switches.test(switch_no),
        }
    }

    /// `daObjKanat::Act_c::Execute` (`d_a_obj_kanat.cpp:81-108`) minus the break particles
    /// (`:90-97`), the `mSmokeCb` delete (`:101-103`) and `set_mtx`/`*pMtx` (`:105-106`).
    /// `Create` (`:28-29`) seeds the two flags and `Draw` (`:114-116`) reads `mIsVisible`.
    fn step(&mut self, switches: &Switches) {
        if self.broken {
            self.visible = false;
        } else if switches.test(self.switch_no) {
            self.broken = true;
        }
    }

    pub fn drawn(&self) -> bool {
        self.visible
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_kanat(name: &str) -> bool {
    KANAT_ACTORS.contains(&name)
}

pub fn control(
    switches: Res<Switches>,
    mut vines: Query<(Entity, &mut Kanat)>,
    mut commands: Commands,
) {
    for (entity, mut vine) in &mut vines {
        if vine.refused {
            let _ = commands.entity(entity).despawn();
            continue;
        }
        vine.step(&switches);
    }
}