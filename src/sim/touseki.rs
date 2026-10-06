use bevy_ecs::prelude::*;

pub const TOUSEKI_ACTORS: [&str; 1] = ["Touseki"];

pub const CULL_MIN: [f32; 3] = [-100.0, 0.0, -100.0]; // <- f_op_actor_mng.cpp:562 (fopAc_CULLBOX_4_e)
pub const CULL_MAX: [f32; 3] = [100.0, 800.0, 100.0]; // <- f_op_actor_mng.cpp:562 (fopAc_CULLBOX_4_e)

/// `daObj_Tousekiki_c` (`d_a_obj_tousekiki.cpp`), the ship's catapult. `_create` (`:58-83`) can
/// return `cPhs_ERROR_e` (`:64`) when `fopAcM_entrySolidHeap` fails; that failure is the
/// model/BCK archive load in `CreateHeap` (`:23-50`), which does not exist in this sim, so the
/// catapult always creates here and there is no spawn-then-despawn path. `Mthd_IsDelete`
/// (`:139-141`) is a bare TRUE - an accepted delete request.
///
/// `_execute` (`:112-116`) is `demo_move()` + `set_mtx()`. `demo_move` (`:95-102`) resolves
/// `l_p_ship` in `_create` by `fopAcM_SearchByID(parentActorID)` (`:78`) and places the mount
/// with `cMtx_multVec(l_p_ship->mModel->getBaseTRMtx(), &touseki_offset, &current.pos)` where
/// `touseki_offset` is `(0.0f, 700.0f, 850.0f)` (`:96`) - already `super::pirateship::
/// CATAPULT_OFFSET`, applied by `pirateship::control`'s `mounted()` (`src/sim/pirateship.rs:
/// 62-104`), which is that exact matrix multiply at actor scale 1.0. The `dDemo_setDemoData`
/// gate (`:98`) is always taken here: `fopAcM_ct_Demo` expands to nothing in a retail build
/// (`f_op_actor_mng.h:25`), so `demoActorID` stays 0, `dComIfGp_demo_getActor(0)` is NULL and
/// `dDemo_setDemoData` returns FALSE (`d_demo.cpp:137-139`) - the `shape_angle`/`current.angle`
/// copy from the ship (`:99-100`) therefore always runs. `parentActorID` (`f_op_actor.cpp:275`)
/// is not parsed by `src/dzs.rs` (the actor record stops being read at `scale`, `at + 0x22`),
/// so a stage-placed `Touseki` has no parent here and its mount transform is left alone; the
/// crate models the link explicitly instead, as `pirateship::Catapult { ship }`. What is left
/// of `_execute` is `set_mtx` (`:104-110`), which is renderer-only.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Touseki;

impl Touseki {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_touseki(name: &str) -> bool {
    TOUSEKI_ACTORS.contains(&name)
}

pub fn control(_catapults: Query<&Touseki>) {}