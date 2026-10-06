use bevy_ecs::prelude::*;

use super::flags::{Collect, Switches};

/// Only the three stage strings `obj_akabe.rs` never registered are handled here. `Akabe`
/// (`d_stage.cpp:1088`) already belongs to `super::obj_akabe`, which is a partial port of the
/// same `daObjAkabe::Act_c`: it has no `prm_get_arg0`/`mType` and no cull box.
pub const AKABE_ACTORS: [&str; 3] = ["Akabe10", "NBOX", "NBOX10"];

const SWSAVE_MASK: u32 = 0xFF; // <- d_a_obj_akabe.h:11 (PRM_SWSAVE_W = 8, PRM_SWSAVE_S = 0)
const ALWAYS_SHIFT: u32 = 12; // <- d_a_obj_akabe.h:15 (PRM_ALWAYS_W = 1, PRM_ALWAYS_S = 12)
const ALWAYS_BIT: u32 = 0x1; // <- d_a_obj_akabe.h:15
const ARG0_SHIFT: u32 = 16; // <- d_a_obj_akabe.h:17 (PRM_ARG0_W = 4, PRM_ARG0_S = 16)
const ARG0_MASK: u32 = 0xF; // <- d_a_obj_akabe.h:17
const NO_SWITCH: u32 = 0xFF; // <- d_a_obj_akabe.cpp:136
const COLLECT_SLOT: u8 = 0; // <- d_a_obj_akabe.cpp:137 (dComIfGs_checkCollect(0))
const TYPE_BOX: u8 = 3; // <- d_a_obj_akabe.cpp:81
const CULL_MIN_PLAIN: [f32; 3] = [-51.0, -1.0, -1.0]; // <- d_a_obj_akabe.cpp:84
const CULL_MAX_PLAIN: [f32; 3] = [51.0, 101.0, 1.0]; // <- d_a_obj_akabe.cpp:84
const CULL_MIN_BOX: [f32; 3] = [-51.0, -1.0, -51.0]; // <- d_a_obj_akabe.cpp:82
const CULL_MAX_BOX: [f32; 3] = [51.0, 101.0, 51.0]; // <- d_a_obj_akabe.cpp:82

/// `daObjAkabe::Act_c::_create` (`:56-64`): `prm_get_arg0` selects the archive out of
/// `M_arcname[4]` = `Akabe`, `AkabeD`, `AkabeK`, `NBOX` (`:17-22`), and anything that is not
/// 1, 2 or 3 falls back to index 0.
fn kind_of(arg0: u8) -> u8 {
    match arg0 {
        1 => 1,
        2 => 2,
        3 => 3,
        _ => 0,
    }
}

/// `chk_appear` (`d_a_obj_akabe.cpp:132-141`).
fn appears(always: bool, switch_save: u8, switches: &Switches, collect: &Collect) -> bool {
    if always {
        return true;
    }
    if switch_save as u32 == NO_SWITCH {
        collect.check(COLLECT_SLOT) == 0
    } else {
        !switches.test(switch_save)
    }
}

/// The wooden stall backs and the NBOX crate. The three stage strings registered here are
/// `Akabe10`, `NBOX` and `NBOX10` (`d_stage.cpp:1089-1091`); `Akabe` (`d_stage.cpp:1088`) is
/// already claimed by `super::obj_akabe`. `prm_get_arg0` picks one of the four archives in
/// `M_arcname[4]` (`d_a_obj_akabe.cpp:17-22`): 0 `Akabe`, 1 `AkabeD`, 2 `AkabeK`, 3 `NBOX`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Akabe {
    /// `_create` (`:54`, `:68-69`) starts at `cPhs_ERROR_e` and only leaves it when
    /// `chk_appear` is true, so a refused term never reaches `_execute`.
    pub refused: bool,
    always: bool,
    switch_save: u8,
    kind: u8,
}

impl Akabe {
    pub fn spawn(parameters: u32, switches: &Switches, collect: &Collect) -> Self {
        let always = (parameters >> ALWAYS_SHIFT) & ALWAYS_BIT != 0;
        let switch_save = (parameters & SWSAVE_MASK) as u8;
        Self {
            refused: !appears(always, switch_save, switches, collect),
            always,
            switch_save,
            kind: kind_of(((parameters >> ARG0_SHIFT) & ARG0_MASK) as u8),
        }
    }

    /// `fopAcM_setCullSizeBox` (`d_a_obj_akabe.cpp:81-85`): the `NBOX` crate is a cube, every
    /// other archive is the flat stall back.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        if self.kind == TYPE_BOX {
            (CULL_MIN_BOX, CULL_MAX_BOX)
        } else {
            (CULL_MIN_PLAIN, CULL_MAX_PLAIN)
        }
    }

    /// `_execute` (`d_a_obj_akabe.cpp:144-148`): the appear test is re-run every tick and a
    /// failure deletes the actor, so the appearance is not latched at create time.
    fn visible(&self, switches: &Switches, collect: &Collect) -> bool {
        appears(self.always, self.switch_save, switches, collect)
    }
}

pub fn is_akabe(name: &str) -> bool {
    AKABE_ACTORS.contains(&name)
}

pub fn control(
    switches: Res<Switches>,
    collect: Res<Collect>,
    walls: Query<(Entity, &Akabe)>,
    mut commands: Commands,
) {
    for (entity, wall) in &walls {
        if wall.refused || !wall.visible(&switches, &collect) {
            let _ = commands.entity(entity).despawn();
        }
    }
}