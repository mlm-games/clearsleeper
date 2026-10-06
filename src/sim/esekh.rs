use bevy_ecs::prelude::*;

use super::flags::Switches;

pub const ESEKH_ACTORS: [&str; 2] = ["Esekh", "Esekh2"];

pub const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)
pub const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- f_op_actor_mng.cpp:558 (fopAc_CULLBOX_0_e)

const SWSAVE_SHIFT: u32 = 0x08; // <- d_a_obj_monument.h:29 (PRM_SWSAVE_S)
const SWSAVE_MASK: u32 = 0xFF; // <- d_a_obj_monument.h:28 (PRM_SWSAVE_W = 0x08, d_a_obj.h:14)

/// Wind's Requiem monuments. The stage strings are `Esekh` and `Esekh2`
/// (`d_stage.cpp:1105-1106`); the model's `_arcname` is `Esekh` for both
/// (`d_a_obj_monument.cpp:27`). `_execute` (`:101-104`) is `set_mtx()` alone, and
/// `Mthd_IsDelete` (`:143-145`) is a bare TRUE - an accepted delete request, never a
/// self-destruct. `prm_get_type` (`d_a_obj_monument.h:35`, `daObj::PrmAbstract(this, 0x01,
/// 0x00)`) only picks the model/DZB pair (`d_a_obj_monument.cpp:19-20`, `:38`, `:51`), which is
/// renderer and moving-background data, so it is not carried; neither is the `dBgW`
/// registration (`:69`) or `init_mtx` (`:95-98`).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Esekh {
    switch_no: u8,
    visible: bool,
}

impl Esekh {
    /// `prm_get_swSave` (`d_a_obj_monument.h:36`) is `daObj::PrmAbstract(this, 0x08, 0x08)`.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            switch_no: ((parameters >> SWSAVE_SHIFT) & SWSAVE_MASK) as u8,
            visible: true,
        }
    }

    /// The live switch read of `daObjMonument::Act_c::_draw` (`:108-110`).
    fn step(&mut self, switches: &Switches) {
        self.visible = !switches.test(self.switch_no);
    }

    pub fn drawn(&self) -> bool {
        self.visible
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_esekh(name: &str) -> bool {
    ESEKH_ACTORS.contains(&name)
}

pub fn control(switches: Res<Switches>, mut stones: Query<&mut Esekh>) {
    for mut stone in &mut stones {
        stone.step(&switches);
    }
}