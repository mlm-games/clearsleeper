use bevy_ecs::prelude::*;

pub const KAJI_ACTORS: [&str; 1] = ["Kaji"];

pub const CULL_MIN: [f32; 3] = [-80.0, -80.0, -20.0]; // <- d_a_kaji.cpp:61
pub const CULL_MAX: [f32; 3] = [80.0, 80.0, 20.0]; // <- d_a_kaji.cpp:61

/// `daKaji_c` (`d_a_kaji.cpp`), Tetra's ship's wheel. `_create` (`:43-67`) can return
/// `cPhs_ERROR_e` (`:57`) when `fopAcM_entrySolidHeap` fails, which cancels the actor inside
/// Create so retail never runs `_execute`; that failure is the model/BCK/BAS archive load
/// (`CreateHeap`, `:26-41`), which does not exist in this sim, and retail asserts the model is
/// present (`:28`), so the helm always creates here and there is no spawn-then-despawn path.
/// `Mthd_IsDelete` (`:123-126`) is a bare TRUE - an accepted delete request.
///
/// `_execute` (`:74-90`) copies the parent ship's transform through `l_p_ship`, resolved once in
/// `_create` by `fopAcM_SearchByID(parentActorID)` (`:55`), with the local offset
/// `(0.0f, 740.0f, -858.0f)` (`:76`) whose `y` and `z` are then offset by `REG10_F(10)` and
/// `REG10_F(11)` (`:77-78`). Both HIO floats are proven zero - `dScnPly_reg_childHIO_c` zeroes
/// all thirty `mFloatRegs` entries in its constructor (`d_s_play.cpp:79-87`) - so the folded
/// offset is exactly `(0.0, 740.0, -858.0)`. `parentActorID` (`f_op_actor.cpp:275`) is not
/// parsed by `src/dzs.rs` (the actor record stops being read at `scale`, `at + 0x22`), so the
/// parent lookup has no input here and the copy is omitted rather than approximated. What is
/// left of `_execute` is `mpMorf->play` (`:85`) and `set_mtx` (`:87`), both renderer-only.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kaji;

impl Kaji {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_kaji(name: &str) -> bool {
    KAJI_ACTORS.contains(&name)
}

pub fn control(_helms: Query<&Kaji>) {}
