use bevy_ecs::prelude::*;

/// Every stage string in `tww/src/d/d_stage.cpp` that resolves to `fpcNm_GRASS_e`, which is the
/// process `daGrass_Create` registers as `g_profile_GRASS` (`d_a_grass.cpp:207`).
pub const SCENERY_ACTORS: [&str; 12] = [
    "kusax1",  // <- d_stage.cpp:438
    "kusax7",  // <- d_stage.cpp:439
    "kusax21", // <- d_stage.cpp:440
    "flower",  // <- d_stage.cpp:441
    "flwr7",   // <- d_stage.cpp:442
    "flwr17",  // <- d_stage.cpp:443
    "pflower", // <- d_stage.cpp:444
    "pflwrx7", // <- d_stage.cpp:445
    "swood",   // <- d_stage.cpp:446
    "swood3",  // <- d_stage.cpp:447
    "swood5",  // <- d_stage.cpp:448
    "Grass",   // <- d_stage.cpp:1259
];

/// `daGrass_Create` (`:33-193`) is the entire actor. It reads three parameters -
/// `getItemNo = (param >> 6) & 0x3F` (`:17`), `getKind = (param >> 4) & 0x03` (`:18`) and
/// `getType = (param >> 0) & 0x0F` (`:19`) - then pushes one scenery instance per entry of
/// `l_offsetData[getType]` (`:119-128`, eight rows) into the global registries
/// `dComIfGp_getGrass()` (`:152`), `dComIfGp_getTree()` (`:166`) or `dComIfGp_getFlower()`
/// (`:187`), and returns `cPhs_ERROR_e` unconditionally (`:192`).
///
/// None of the three scenery registries exists in this sim, and the function has no
/// `_execute`, no `_draw` and no AT or Tg collider - `g_profile_GRASS` (`:203-218`) is
/// `fopAcStts_UNK40000_e`, layer CURRENT, list 0x0007, `fopAc_CULLBOX_0_e` (`:217`).
///
/// Retail therefore spawns the process and cancels it inside Create
/// (`f_pc_create_req.cpp:106-108`), so the entity never runs a tick. `daGrass_IsDelete`
/// returning `TRUE` (`:23-25`) only accepts a delete request; it is not a second despawn.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scenery {
    pub refused: bool,
}

impl Scenery {
    pub fn spawn() -> Self {
        Self { refused: true }
    }
}

pub fn is_scenery(name: &str) -> bool {
    SCENERY_ACTORS.contains(&name)
}

pub fn control(fallen: Query<(Entity, &Scenery)>, mut commands: Commands) {
    for (entity, grass) in &fallen {
        if grass.refused {
            let _ = commands.entity(entity).despawn();
        }
    }
}
