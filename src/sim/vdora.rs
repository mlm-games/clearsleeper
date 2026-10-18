use bevy_ecs::prelude::*;

pub const VDOAR_ACTORS: [&str; 1] = ["Vdora"];

pub const CULL_MIN: [f32; 3] = [-100.0, -1.0, -50.0]; // <- d_a_obj_gong.cpp:81
pub const CULL_MAX: [f32; 3] = [100.0, 230.0, 50.0]; // <- d_a_obj_gong.cpp:81

/// Tetra's ship gondola. The stage string and the model's `_arcname` are both `Vdora`
/// (`d_stage.cpp:1083`, `d_a_obj_gong.cpp:33`).
///
/// `_execute` (`:120-125`) is `demo_move()` + `mpMorf->play(NULL, 0, 0)` + `set_mtx()`, all
/// three renderer-side: `demo_move` (`:111-117`) is `dDemo_setDemoData`, the demo recorder,
/// and `fopAcM_ct_Demo` expands to nothing outside the demo build, so `demoActorID` stays 0 in
/// retail and the recording writes nothing. `_draw` (`:128-136`) is tev setup, `dDlst_texSpecmapST`
/// and `updateDL`. `_create` (`:72-89`) sets the cull box above, raises `eyePos.y` by
/// `attr().offsetY` (125.0f, `:26`/`:82`) and loads `Vdora.arc`; `attr().spec` (0.75f, `:26`) is
/// read only by `dDlst_texSpecmapST` in `_draw`. `Mthd_IsDelete` (`:161-163`) is a bare
/// `TRUE`, i.e. an accepted delete request, never a self-destruct. This actor registers no AT
/// or Tg collider and cannot be damaged.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vdora;

impl Vdora {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_vdora(name: &str) -> bool {
    VDOAR_ACTORS.contains(&name)
}

pub fn control(_gongs: Query<&Vdora>) {}
