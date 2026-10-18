use bevy_ecs::prelude::*;

pub const TABLE_ACTORS: [&str; 1] = ["Table"];

pub const CULL_MIN: [f32; 3] = [-80.0, -1.0, -80.0]; // <- d_a_obj_table.cpp:46
pub const CULL_MAX: [f32; 3] = [80.0, 205.0, 80.0]; // <- d_a_obj_table.cpp:46

/// Outset Island table. The stage string and the model's `_arcname` are both `Table`
/// (`d_stage.cpp:990`, `d_a_obj_table.cpp:18`). `Execute` (`:101-105`) is `set_mtx()` plus
/// handing the matrix to the moving background, `MoveBGExecute` (`d_bg_s_movebg_actor.cpp:86-102`)
/// then copies it into `mBgMtx` and advances the `dBgW`, and `Draw` (`:108-131`) draws the
/// model and a simple shadow. `prm_get_mdl` (`d_a_obj_table.h:22`, `daObj::PrmAbstract(this,
/// 0x08, 0x00)`) only picks the model and DZB pair (`:25-38`, `:62-66`), releases the `dBgW`
/// for variant 2 (`:47-51`) and picks the shadow drop height (`:119-123`) - all renderer or
/// moving-background data, so it is not carried. `Mthd_IsDelete` is `MoveBGIsDelete()`, i.e.
/// the base `IsDelete() { return TRUE; }` (`d_bg_s_movebg_actor.h:35`): an accepted delete
/// request, never a self-destruct.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Table;

impl Table {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_table(name: &str) -> bool {
    TABLE_ACTORS.contains(&name)
}

pub fn control(_tables: Query<&Table>) {}
