use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::flags::{Collect, Switches};

/// All four stage strings that resolve to `fpcNm_Obj_Akabe_e`, the process `daObjAkabe::Act_c`
/// registers as `g_profile_Obj_Akabe` (`d_a_obj_akabe.cpp:197`).
pub const AKABE_ACTORS: [&str; 4] = [
    "Akabe",   // <- d_stage.cpp:1088
    "Akabe10", // <- d_stage.cpp:1089
    "NBOX",    // <- d_stage.cpp:1090
    "NBOX10",  // <- d_stage.cpp:1091
];

const SWSAVE_MASK: u32 = 0xFF; // <- d_a_obj_akabe.h:13 (PRM_SWSAVE_W = 8, PRM_SWSAVE_S = 0)
const ALWAYS_SHIFT: u32 = 12; // <- d_a_obj_akabe.h:19 (PRM_ALWAYS_W = 1, PRM_ALWAYS_S = 12)
const ALWAYS_BIT: u32 = 0x1; // <- d_a_obj_akabe.h:19
const ARG0_SHIFT: u32 = 16; // <- d_a_obj_akabe.h:22 (PRM_ARG0_W = 4, PRM_ARG0_S = 16)
const ARG0_MASK: u32 = 0xF; // <- d_a_obj_akabe.h:22
const NO_SWITCH: u8 = 0xFF; // <- d_a_obj_akabe.cpp:136
const COLLECT_SLOT: u8 = 0; // <- d_a_obj_akabe.cpp:137 (dComIfGs_checkCollect(0))
const TYPE_BOX: u8 = 3; // <- d_a_obj_akabe.cpp:81
const CULL_MIN_PLAIN: [f32; 3] = [-51.0, -1.0, -1.0]; // <- d_a_obj_akabe.cpp:84
const CULL_MAX_PLAIN: [f32; 3] = [51.0, 101.0, 1.0]; // <- d_a_obj_akabe.cpp:84
const CULL_MIN_BOX: [f32; 3] = [-51.0, -1.0, -51.0]; // <- d_a_obj_akabe.cpp:82
const CULL_MAX_BOX: [f32; 3] = [51.0, 101.0, 51.0]; // <- d_a_obj_akabe.cpp:82

/// `_create` (`:56-64`): `prm_get_arg0` selects `mType`, which indexes `M_arcname[4]` = `Akabe`,
/// `AkabeD`, `AkabeK`, `NBOX` (`:17-22`). Anything that is not 1, 2 or 3 falls back to index 0.
fn kind_of(arg0: u8) -> u8 {
    match arg0 {
        1 => 1,
        2 => 2,
        3 => 3,
        _ => 0,
    }
}

/// `chk_appear` (`:132-141`).
fn appears(always: bool, switch_save: u8, switches: &Switches, collect: &Collect) -> bool {
    if always {
        return true;
    }
    if switch_save == NO_SWITCH {
        collect.check(COLLECT_SLOT) == 0
    } else {
        !switches.test(switch_save)
    }
}

/// `daObjAkabe::Act_c::_draw` (`:150-153`) makes no draw call and `_execute` (`:144-148`) only
/// re-runs `chk_appear`, so the actor is a flat stall back or an `NBOX` crate carried by a
/// registered `dBgW`. It registers no AT or Tg collider, so it cannot be damaged.
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
        let kind = kind_of(((parameters >> ARG0_SHIFT) & ARG0_MASK) as u8);
        Self {
            refused: !appears(always, switch_save, switches, collect),
            always,
            switch_save,
            kind,
        }
    }

    /// `fopAcM_setCullSizeBox` (`:81-85`), gated on `fopAc_CULLBOX_CUSTOM_e` (`:207`): the `NBOX`
    /// crate is a cube, every other archive is the flat stall back. `scale` is exactly 1.0 for a
    /// stage-placed actor, so the literals need no scale factor.
    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        if self.kind == TYPE_BOX {
            (CULL_MIN_BOX, CULL_MAX_BOX)
        } else {
            (CULL_MIN_PLAIN, CULL_MAX_PLAIN)
        }
    }

    /// `_execute` (`:144-148`): the appear test is re-run every tick and a failure calls
    /// `fopAcM_delete`, so the appearance is not latched at create time.
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
    stalls: Query<(Entity, &Akabe)>,
    mut commands: Commands,
) {
    for (entity, stall) in &stalls {
        if stall.refused || !stall.visible(&switches, &collect) {
            let _ = commands.entity(entity).despawn();
        }
    }
}

pub fn spawn(world: &mut World, source: &crate::dzs::StageActor) -> Entity {
    let actor = Actor {
        name: source.name.clone(),
        parameters: source.parameters,
        pos: glam::Vec3::from_array(source.pos),
        angle: source.angle,
        set_id: source.set_id,
        room: source.room,
        ..Default::default()
    };
    let item = Akabe::spawn(
        source.parameters,
        world.resource::<Switches>(),
        world.resource::<Collect>(),
    );
    world.spawn((actor, item)).id()
}