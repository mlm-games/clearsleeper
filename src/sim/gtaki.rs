use bevy_ecs::prelude::*;

pub const GTAKI_ACTORS: [&str; 1] = ["Gtaki"];

const CULL_MIN: [f32; 3] = [-600.0, 0.0, -600.0]; // <- d_a_obj_gtaki.cpp:108
const CULL_MAX: [f32; 3] = [600.0, 10000.0, 600.0]; // <- d_a_obj_gtaki.cpp:108

/// The drifting leaf drift. The stage string is `Gtaki` (`d_stage.cpp:991`); the archive is also
/// `"Gtaki"` (`d_a_obj_gtaki.cpp:66`) with the collision archive `"ITAKI"` (`:83`), so
/// `_arcname` and the STAGE string agree.
///
/// The actor's whole body is a `dBgW` moving background: `CreateHeap` builds `mMtx` from the
/// term's own position, yaw and scale (`:76-79`), hands it to `new dBgW()->Set(ITAKI, MOVE_BG_e,
/// &mMtx)` (`:81-85`), and `CreateInit` finishes with `dComIfG_Bgsp()->Regist(mpBgW, this)`
/// (`:120`). RULE 3b lists `dBgW`/`cBgW` moving backgrounds as a confirmed-absent input, so the
/// pile never moves here and `_delete`'s `dComIfG_Bgsp()->Release(mpBgW)` (`:155`) has nothing
/// to release.
///
/// The cylinder is likewise inert: `l_cyl_src` (`:12-40`) has `SrcObjAt Type = 0` (`:16`) and
/// `SrcObjTg Type = 0` (`:20`), so the pile has no attack arm and is hit by nothing at all; its
/// `SrcObjCo SPrm` (`:21`) is enemy pushback against a body that never moves. `CreateInit`'s
/// `mCyl.SetR(scale.x * 70.0f)` (`:113`) is therefore unobservable, and so are `mBtkAnm.play()`
/// (`:189`), `setDummyTexture` (`:49-62`), `set_effect` (`:90-103`) and `set_mtx` (`:124-129`) -
/// all renderer or particle work.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gtaki;

impl Gtaki {
    pub fn spawn() -> Self {
        Self
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }
}

pub fn is_gtaki(name: &str) -> bool {
    GTAKI_ACTORS.contains(&name)
}

pub fn control(_piles: Query<&Gtaki>) {}
