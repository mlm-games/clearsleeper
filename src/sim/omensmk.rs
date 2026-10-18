use bevy_ecs::prelude::*;

pub const OMENS_MK_ACTORS: [&str; 1] = ["Omensmk"];

/// Hylian-ruins ceiling smoke and falling rubble. There is no `_arcname`: `_create`
/// (`d_a_obj_homensmoke.cpp:36-67`) loads no archive and builds no model, and the two particle
/// emitters it creates are named `ID_AK_JT_ELEMENTSMOKE01` (`:97`) and
/// `ID_AK_SN_KAZEMASKHAHEN00` (`:116`).
///
/// `_draw` (`:144-146`) is literally `return true;` with no draw call, so retail renders
/// nothing but particles. `_execute` (`:82-141`) is one shot of emitter setup and then
/// `fopAcM_delete(this)` once `mSmokeCb.isEnd()` (`:137-139`) reports the smoke callback
/// finished; both the callback's end test and the `rate_table`, `norse_offsetL/S` and
/// `culling_dat` values exist only to place and time those emitters. `mType` (`:46`) feeds
/// `param_get_axis()` in `set_mtx` (`:16`) and `culling_dat` (`:57`), so by the same rule as
/// `mbInitialized` it is not stored: nothing reads it outside omitted work.
///
/// `Mthd_IsDelete` (`:170-172`) is a bare `TRUE`, an accepted delete request, never a
/// self-destruct. No AT or Tg collider is registered.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Omensmk;

impl Omensmk {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_omensmk(name: &str) -> bool {
    OMENS_MK_ACTORS.contains(&name)
}

pub fn control(_smoke: Query<&Omensmk>) {}
