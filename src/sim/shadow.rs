//! Blob shadows - `dComIfGd_setShadow` (`d_com_inf_game.cpp:1218-1240`),
//! `dDlst_shadowControl_c` (`d_drawlist.h:380-404`) and the simple-shadow entry
//! `dDlst_shadowSimple_c::set` (`d_drawlist.cpp:1477-1520`).
//!
//! `dComIfGd_setShadow` tries the caster's *model* shadow first: `dComIfGd_setRealShadow2`
//! projects the caster's shadow polygon with the light (`setShadowRealMtx`, `:1297-1352`, and
//! `realPolygonCheck`, `:1245-1295`) and returns a key while the walk holds. When it returns 0,
//! the function registers a *simple* shadow instead: a point `(pos.x, y, pos.z)` on the ground
//! polygon with `scaleXZ`, put through `dDlst_shadowControl_c::setSimple` into the frame's list
//! (`mSimple[128]`, cleared by `dDlst_shadowControl_c::reset`, `:1529-1534`, from
//! `dDlst_list_c::reset`, `:2000`).
//!
//! What this module implements is the second arm and its registry, because it is the arm the
//! port's data can reach: [`ShadowControl`] holds the frame's simple list with retail's matrix
//! and alpha maths ([`ShadowSimple`]), [`ground_probe`] runs the `dBgS::GroundCross` query
//! `mObjAcch.GetGroundH()`/`m_gnd` carry, and the fallback is
//! [`ShadowControl::set_shadow`] with `dComIfGd_setShadow`'s no-ground early-out intact.
//!
//! What is absent: the real arm, which needs the caster's `J3DModel` and its shadow-model
//! polygon walk, the point light's near position (`dKy_plight_near_pos`) and the Tev string -
//! none of which the port has, so every registration here takes the fallback, as retail's does
//! whenever the walk fails. And the *draw*: `dDlst_shadowSimple_c::draw`
//! (`d_drawlist.cpp:1430-1474`) paints a textured quad with `l_shadowSealTexDL` over the shared
//! 128x128 `I4` image (`dDlst_shadowControl_c::setSimpleTex`, `:1663-1666`, fed from the logo
//! texture at `d_s_logo.cpp:601`). `repame-view3d` can draw a non-depth-tested ground quad
//! (`MeshGroup`, `depth_test = false`), but this port's frame builds its geometry itself and
//! never uploads a shadow texture, so nothing here consumes the list - it is the registry that
//! draw pass would read.

use bevy_ecs::prelude::*;
use glam::{Mat4, Vec3};

use super::collision::GroundHit;

/// `ARRAY_SIZE(mSimple)` (`d_drawlist.h:399`): the per-frame simple-shadow cap
/// (`dDlst_shadowControl_c::setSimple`, `d_drawlist.cpp:1653-1660`).
pub const SIMPLE_MAX: usize = 128;

/// `dDlst_shadowSimple_c::set`'s alpha fade rate (`d_drawlist.cpp:1513`), and the 64 it
/// scales to (`:1518`).
const FADE_RATE: f32 = 0.0007;
const FADE_ALPHA: f32 = 64.0;

/// `dComIfGd_setShadow`'s no-ground sentinel (`d_com_inf_game.cpp:1222`):
/// `groundY <= -G_CM3D_F_INF`. The port's probe reports a miss as this value, which is also
/// what `mObjAcch.GetGroundH()` carries while an acch has no hit.
pub const NO_GROUND: f32 = f32::NEG_INFINITY;

/// `dDlst_shadowSimple_c` (`d_drawlist.h:292-303`), minus `mpTexObj`: the port has no texture
/// object, and the image it would point at is the shared simple-shadow texture named in this
/// module's docs. Both matrices carry the frame's view, as retail's do: they are the position
/// matrices `draw` loads (`GXLoadPosMtxImm(mVolumeMtx, ...)`, `d_drawlist.cpp:1438`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowSimple {
    /// `mVolumeMtx`: `view * transS(pos.x, y + offsetY, pos.z) * YrotM(rotY) *
    /// scaleM(scaleXZ, offsetY * 2 + 16, scaleXZ * scaleZ)`, the stack concatenated onto the
    /// view (`set`, `d_drawlist.cpp:1478-1482`).
    pub volume: Mat4,
    /// `mMtx`: `view * basis * YrotM(rotY) * scaleM(scaleXZ, 1, scaleXZ * scaleZ)`, the plane
    /// basis `set` writes over the stack (`:1495-1512`).
    pub quad: Mat4,
    /// `mAlpha`: `clamp(1 - (pos.y - groundY) * 0.0007, 0, 1) * 64`.
    pub alpha: u8,
}

/// `dDlst_shadowControl_c`'s per-frame state (`d_drawlist.h:396-399`). The real list
/// (`mReal[8]`, `mNextID`) cannot exist here: a slot only becomes used after
/// `setShadowRealMtx` succeeds, and its inputs are the ones this module's docs name as absent.
#[derive(Resource, Debug, Default)]
pub struct ShadowControl {
    simple: Vec<ShadowSimple>,
}

impl ShadowControl {
    /// `dDlst_shadowControl_c::reset` (`d_drawlist.cpp:1529-1534`, run from
    /// `dDlst_list_c::reset`, `:2000`): the frame's list is rebuilt by each draw.
    pub fn reset(&mut self) {
        self.simple.clear();
    }

    /// The frame's simple shadows, as the blob draw pass would read them.
    pub fn simple(&self) -> &[ShadowSimple] {
        &self.simple
    }

    /// `dDlst_shadowControl_c::setSimple` (`d_drawlist.cpp:1653-1660`) plus
    /// `dDlst_shadowSimple_c::set` (`:1477-1520`). `floor_nrm` is the ground polygon's plane
    /// normal, which is what `dComIfGd_setSimpleShadow2` (`d_com_inf_game.cpp:588-598`) reads
    /// out of the caster's `cBgS_PolyInfo`; without a set polygon it registers nothing, as
    /// retail's `ChkSetInfo` early-out does.
    pub fn set_simple(
        &mut self,
        pos: Vec3,
        y: f32,
        floor_nrm: Vec3,
        scale_xz: f32,
        rot_y: i16,
        scale_z: f32,
        view: &Mat4,
    ) -> bool {
        if self.simple.len() >= SIMPLE_MAX {
            return false;
        }
        let offset_y = scale_xz * 16.0 * (1.0 - floor_nrm.y) + 1.0;
        let scale = Vec3::new(scale_xz, offset_y + offset_y + 16.0, scale_xz * scale_z);
        // `mDoMtx_stack_c::transS` overwrites the stack and `YrotM`/`scaleM` concatenate on
        // its right (`m_Do_mtx.h:250-332`), and `mDoMtx_concat(view, stack, out)` puts the
        // stack on the view's right (`MTXConcat`, `mtx.c:84-101`), so the column-vector forms
        // are `view * T * Ry * S` and `view * basis * Ry * S`.
        let volume = *view
            * Mat4::from_translation(Vec3::new(pos.x, y + offset_y, pos.z))
            * Mat4::from_rotation_y(super::walk::from_s16(rot_y))
            * Mat4::from_scale(scale);

        let xs = (1.0 - floor_nrm.x * floor_nrm.x).max(0.0).sqrt();
        let (yy, zz) = if xs != 0.0 {
            (floor_nrm.y * xs, -floor_nrm.z * xs)
        } else {
            (0.0, 0.0)
        };
        // `mDoMtx_stack_c::get()`'s rows are overwritten with the plane basis and the point;
        // glam's row/column element layout is the same as the Dolphin `Mtx`'s, so the rows go in
        // as the columns' entries.
        let basis = Mat4::from_cols_array(&[
            xs,
            -floor_nrm.x * yy,
            floor_nrm.x * zz,
            0.0,
            floor_nrm.x,
            floor_nrm.y,
            floor_nrm.z,
            0.0,
            0.0,
            zz,
            yy,
            0.0,
            pos.x,
            y,
            pos.z,
            1.0,
        ]);
        let quad = *view
            * basis
            * Mat4::from_rotation_y(super::walk::from_s16(rot_y))
            * Mat4::from_scale(Vec3::new(scale_xz, 1.0, scale_xz * scale_z));

        let dist = (1.0 - (pos.y - y) * FADE_RATE).clamp(0.0, 1.0);
        self.simple.push(ShadowSimple {
            volume,
            quad,
            alpha: (dist * FADE_ALPHA) as u8,
        });
        true
    }

    /// `dComIfGd_setShadow` (`d_com_inf_game.cpp:1218-1240`) with the arguments this port has:
    /// the caster point, the caster's own `y`, the ground probe and `scaleXZ`; `rotY` and
    /// `scaleZ` are Ba1's `0` and `1.0`. The real arm's arguments - the id/`shouldFade` pair,
    /// `casterSize` (Ba1's `800.0f`), the `J3DModel`, the Tev string and the texture object -
    /// have no port counterpart and are the absent inputs the module docs name.
    ///
    /// The early-out is retail's `if (groundY <= -G_CM3D_F_INF) return 0;`. The real arm is not
    /// ported, so the fallback runs exactly as retail's does when `dComIfGd_setRealShadow2`
    /// returns 0: a point `(pos.x, y, pos.z)` is registered against the ground polygon, and the
    /// return is that 0, which is what a caller's shadow id stays at.
    pub fn set_shadow(
        &mut self,
        pos: Vec3,
        y: f32,
        ground_y: f32,
        floor_nrm: Option<Vec3>,
        scale_xz: f32,
        rot_y: i16,
        scale_z: f32,
        view: &Mat4,
    ) -> u32 {
        if ground_y <= NO_GROUND {
            return 0;
        }
        let Some(floor_nrm) = floor_nrm else {
            return 0;
        };
        let point = Vec3::new(pos.x, y, pos.z);
        self.set_simple(point, ground_y, floor_nrm, scale_xz, rot_y, scale_z, view);
        0
    }
}

/// `dBgS::GroundCross`, run where `mObjAcch.GetGroundH()`/`mObjAcch.m_gnd` would be read:
/// `dComIfGd_setShadow`'s `groundY` and the polygon `dComIfGd_setSimpleShadow2` needs.
pub fn ground_probe(solid: Option<&super::collision::Collision>, pos: Vec3) -> Option<GroundHit> {
    solid.and_then(|solid| solid.ground_cross_below(pos))
}

/// The per-frame `reset` (`dDlst_shadowControl_c::reset`) that precedes every draw's
/// registration.
pub fn control(mut shadow: ResMut<ShadowControl>) {
    shadow.reset();
}
