use bevy_ecs::prelude::*;

pub const GBED_ACTORS: [&str; 1] = ["Gbed"];

/// Puppet Ganon's bed, the one Ganon is lying on in the intro cutscene. The stage string is
/// `Gbed` (`d_stage.cpp:1126`) and the archive is `Gbed` (`d_a_obj_ganonbed.cpp:17`) with
/// `K_GBED` (`:60`, `:67`).
///
/// Retail `_execute` (`:134-146`) is `mpBgW->Move()` behind a null/used check; the only other
/// branch is inside `#if VERSION == VERSION_DEMO` (`:139-143`), which does not ship. `_create`
/// (`:77-102`) can also return `cPhs_ERROR_e` when `dComIfG_Bgsp()->Regist(mpBgW, this)`
/// reports failure (`:84-85`) - that is the moving-background registry rejecting a duplicate
/// registration, not a gameplay gate, so it is not a spawn refusal. `_draw` (`:149-162`) is
/// `g_env_light` plus `mDoExt_modelUpdateDL`. The profile is `fopAc_CULLBOX_CUSTOM_e`
/// (`:212`) but no `fopAcM_setCullSizeBox` call exists anywhere in the translation unit, so
/// no cull box is derivable.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gbed;

impl Gbed {
    pub fn spawn() -> Self {
        Self
    }
}

pub fn is_gbed(name: &str) -> bool {
    GBED_ACTORS.contains(&name)
}

pub fn control(_beds: Query<&Gbed>) {}