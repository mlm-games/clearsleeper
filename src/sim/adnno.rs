use bevy_ecs::prelude::*;

pub const ADNNO_ACTORS: [&str; 1] = ["Adnno"];

pub const CULL_MIN: [f32; 3] = [-600.0, 0.0, -600.0]; // <- d_a_obj_adnno.cpp:55
pub const CULL_MAX: [f32; 3] = [600.0, 900.0, 600.0]; // <- d_a_obj_adnno.cpp:55

/// Unused Triforce prayer mural. The stage string and the model's `_arcname` are both `Adnno`
/// (`d_stage.cpp:1098`, `d_a_obj_adnno.cpp:42`). `CreateHeap` (`:41-50`) clones one BMD sixteen
/// times, `_execute` (`:98-101`) is `set_mtx()` alone and `daObjAdnno_IsDelete` (`:158-161`) is
/// a bare TRUE - an accepted delete request, never a self-destruct.
///
/// `set_mtx` (`:61-75`) lays the sixteen sub-models out on a 4x4 grid
/// (`(i % 4) * 40 - 60` in x, `60 - (i / 4) * 40` in y) and `_draw` (`:122-135`) swaps each
/// sub-model's material table when `daObjAdnno_event_bit_table[i]` (`d_a_obj_adnno.cpp:103-120`)
/// reads on. Both are matrix setup and appearance, so nothing from them is carried; the term
/// reads no parameter.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Adnno;

impl Adnno {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_adnno(name: &str) -> bool {
    ADNNO_ACTORS.contains(&name)
}

pub fn control(_murals: Query<&Adnno>) {}
