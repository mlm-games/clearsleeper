use bevy_ecs::prelude::*;
use glam::Vec3;

use super::shape::{Shape, ShapeKind, Shapes, tri_sph_cross_point};

/// `G_CM3D_F_ABS_MIN`, the constant both `cM3d_IsZero` (`c_m3d.h:135`) and `cLib_IsZero`
/// (`c_lib.h:8`) test a scalar against.
pub const F_ABS_MIN: f32 = 8e-11;

/// `G_CM3D_F_INF` (`c_m3d.h:25`), the sentinel `cM3dGAab::ClearForMinMax` seeds itself with.
pub const F_INF: f32 = 1000000000.0;

/// `cM3d_IsZero`.
pub fn is_zero(value: f32) -> bool {
    value.abs() < F_ABS_MIN
}

/// `cCcD_AtSPrm_Set_e`.
pub const AT_SPRM_SET: u32 = 0x01;
/// `cCcD_AtSPrm_GrpAll_e`.
pub const AT_SPRM_GRP_ALL: u32 = 0x02 | 0x04 | 0x08;
/// `cCcD_AtSPrm_NoTgHitInfSet_e`.
pub const AT_SPRM_NO_TG_HIT_INF_SET: u32 = 0x10;
/// `cCcD_AtRPrm_Hit_e`.
pub const AT_RPRM_HIT: u32 = 1 << 0;
/// `cCcD_TgSPrm_Set_e`.
pub const TG_SPRM_SET: u32 = 0x01;
/// `cCcD_TgSPrm_GrpAll_e`.
pub const TG_SPRM_GRP_ALL: u32 = 0x02 | 0x04 | 0x08;
/// `cCcD_TgSPrm_NoAtHitInfSet_e`.
pub const TG_SPRM_NO_AT_HIT_INF_SET: u32 = 0x10;
/// `cCcD_TgRPrm_Hit_e`.
pub const TG_RPRM_HIT: u32 = 1 << 0;
/// `cCcD_CoSPrm_Set_e`.
pub const CO_SPRM_SET: u32 = 0x01;
/// `cCcD_CoSPrm_IGrpAll_e`.
pub const CO_SPRM_I_GRP_ALL: u32 = 0x02 | 0x04 | 0x08;
/// `cCcD_CoSPrm_VsGrpAll_e`.
pub const CO_SPRM_VS_GRP_ALL: u32 = 0x10 | 0x20 | 0x40;
/// `cCcD_CoSPrm_Sph3DCrr_e`.
pub const CO_SPRM_SPH_3D_CRR: u32 = 0x80;
/// `cCcD_CoSPrm_NoCrr_e`.
pub const CO_SPRM_NO_CRR: u32 = 0x100;
/// `cCcD_CoSPrm_NoCoHitInfSet_e`.
pub const CO_SPRM_NO_CO_HIT_INF_SET: u32 = 0x200;
/// `cCcD_CoRPrm_Hit_e`.
pub const CO_RPRM_HIT: u32 = 1 << 0;
/// `cCcD_Obj::mFlags` bit 1, which `ChkBsRevHit` reads (`c_cc_d.h:617`).
pub const OBJ_FLAG_BS_REV_HIT: u32 = 1 << 1;
/// `dCcD_GStts::Stts_NoActor_e`, which `ChkNoActor` reads.
pub const GSTTS_NO_ACTOR: u32 = 1 << 0;
/// `dCcG_AtSPrm_NoConHit_e`.
pub const G_AT_SPRM_NO_CON_HIT: u32 = 1 << 0;
/// `dCcG_AtSPrm_StopNoConHit_e`.
pub const G_AT_SPRM_STOP_NO_CON_HIT: u32 = 1 << 2;
/// `dCcG_TgSPrm_NoConHit_e`.
pub const G_TG_SPRM_NO_CON_HIT: u32 = 1 << 1;
/// `dCcG_CoSPrm_AtLasso_e`.
pub const G_CO_SPRM_AT_LASSO: u32 = 1 << 0;
/// `dCcG_CoSPrm_TgLasso_e`.
pub const G_CO_SPRM_TG_LASSO: u32 = 1 << 1;
/// `dCcG_CoRPrm_HitNoActor_e`.
pub const G_CO_RPRM_HIT_NO_ACTOR: u32 = 1 << 0;

/// `cM3dGAab` (`c_m3d_g_aab.h:10`): the min and max corner every `CalcAabBox` writes and the one
/// box `cCcS::CalcArea` folds them all into.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aab {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aab {
    /// `cM3dGAab::ClearForMinMax`: min at `G_CM3D_F_INF`, max at its negation.
    pub fn clear_for_min_max() -> Self {
        Self {
            min: Vec3::splat(F_INF),
            max: Vec3::splat(-F_INF),
        }
    }

    /// `cM3dGAab::SetMinMax(const cXyz&)`: widen to take `point` in.
    pub fn set_min_max(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    /// `cM3dGAab::CalcCenter`.
    pub fn calc_center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// `cM3d_Cross_AabAab`: the touching-inclusive overlap of two boxes, which is the reject
    /// `cCcD_DivideArea`'s bucket ranges approximate.
    pub fn cross(&self, other: &Aab) -> bool {
        self.min.cmple(other.max).all() && other.min.cmple(self.max).all()
    }
}

/// `cCcD_DivideInfo` (`c_cc_d.h:251`): the coarse bucket range `cCcD_DivideArea` derives for one
/// object, kept beside the shape's own work box because `cCcS::ChkAtTg`'s mixed branch reads its
/// cross point straight off that box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DivideInfo {
    /// `cCcD_DivideInfo::mRangeBits`: eleven cells per axis, packed low 11 | next 11 | next 11.
    pub range_bits: u32,
    /// `cCcD_ShapeAttr::GetWorkAab()`, as the last `CalcArea` computed it.
    pub aab: Aab,
}

impl Default for DivideInfo {
    fn default() -> Self {
        Self {
            range_bits: 0,
            aab: Aab::clear_for_min_max(),
        }
    }
}

impl DivideInfo {
    /// `cCcD_DivideInfo::Set`.
    pub fn set(&mut self, range_bits: u32) {
        self.range_bits = range_bits;
    }

    /// `cCcD_DivideInfo::Chk` (`c_cc_d.cpp:17-26`): the broad phase every AT/TG and CO pair is
    /// rejected by unless the two bucket ranges meet on the x, z and then the y axis, in that
    /// order.
    pub fn chk(&self, other: &DivideInfo) -> bool {
        let mask = self.range_bits & other.range_bits;
        if mask & 0x7FF == 0 {
            return false;
        }
        if mask & 0xFFE00000 == 0 {
            return false;
        }
        if mask & 0x1FF800 == 0 {
            return false;
        }
        true
    }

    /// `cM3d_Cross_AabAab` over the two work boxes: the exact-box form of the same reject, which
    /// the bucket ranges only approximate. The pipeline rejects with [`DivideInfo::chk`], the
    /// boxes are carried for `calc_center` and for a caller that wants the tight test.
    pub fn overlap(&self, other: &DivideInfo) -> bool {
        self.aab.cross(&other.aab)
    }
}

/// `cCcD_DivideArea` (`c_cc_d.h:264`): the scene box `cCcS::CalcArea` folds every work box into,
/// plus the per-axis cell scales `CalcDivideInfo` quantises with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DivideArea {
    /// The inherited `cM3dGAab`.
    pub aab: Aab,
    pub x_diff_is_zero: bool,
    pub scaled_x_diff: f32,
    pub inv_scaled_x_diff: f32,
    pub y_diff_is_zero: bool,
    pub scaled_y_diff: f32,
    pub inv_scaled_y_diff: f32,
    pub z_diff_is_zero: bool,
    pub scaled_z_diff: f32,
    pub inv_scaled_z_diff: f32,
}

impl Default for DivideArea {
    fn default() -> Self {
        Self {
            aab: Aab::clear_for_min_max(),
            x_diff_is_zero: false,
            scaled_x_diff: 0.0,
            inv_scaled_x_diff: 0.0,
            y_diff_is_zero: false,
            scaled_y_diff: 0.0,
            inv_scaled_y_diff: 0.0,
            z_diff_is_zero: false,
            scaled_z_diff: 0.0,
            inv_scaled_z_diff: 0.0,
        }
    }
}

impl DivideArea {
    /// `cCcD_DivideArea::SetArea` (`c_cc_d.cpp:29-45`): take the scene box and derive the eleven
    /// cells per horizontal axis and ten per vertical axis from its extent.
    pub fn set_area(&mut self, aab: Aab) {
        self.aab = aab;
        self.scaled_x_diff = (aab.max.x - aab.min.x) * 0.09090909;
        self.x_diff_is_zero = is_zero(self.scaled_x_diff);
        if !self.x_diff_is_zero {
            self.inv_scaled_x_diff = 1.0 / self.scaled_x_diff;
        }
        self.scaled_y_diff = (aab.max.y - aab.min.y) * 0.1;
        self.y_diff_is_zero = is_zero(self.scaled_y_diff);
        if !self.y_diff_is_zero {
            self.inv_scaled_y_diff = 1.0 / self.scaled_y_diff;
        }
        self.scaled_z_diff = (aab.max.z - aab.min.z) * 0.09090909;
        self.z_diff_is_zero = is_zero(self.scaled_z_diff);
        if !self.z_diff_is_zero {
            self.inv_scaled_z_diff = 1.0 / self.scaled_z_diff;
        }
    }

    /// `cCcD_DivideArea::CalcDivideInfo` (`c_cc_d.cpp:48-105`): bucket `aab` for one object. A
    /// `ChkBsRevHit` object takes every cell, `0xFFFFFFFF`; otherwise each axis contributes its
    /// inclusive cell span, or the whole axis when that axis of the scene box is degenerate.
    pub fn calc_divide_info(&self, info: &mut DivideInfo, aab: Aab, bs_rev_hit: bool) {
        info.aab = aab;
        if bs_rev_hit {
            info.set(0xFFFF_FFFF);
            return;
        }
        let x = if self.x_diff_is_zero {
            0x7FF
        } else {
            cell_span(
                self.inv_scaled_x_diff,
                self.aab.min.x,
                aab.min.x,
                aab.max.x,
                10,
            )
        };
        let y = if self.y_diff_is_zero {
            0x3FF
        } else {
            cell_span(
                self.inv_scaled_y_diff,
                self.aab.min.y,
                aab.min.y,
                aab.max.y,
                9,
            )
        };
        let z = if self.z_diff_is_zero {
            0x7FF
        } else {
            cell_span(
                self.inv_scaled_z_diff,
                self.aab.min.z,
                aab.min.z,
                aab.max.z,
                10,
            )
        };
        info.set(x | (y << 11) | (z << 21));
    }
}

/// One axis of `cCcD_DivideArea::CalcDivideInfo`: the inclusive cell span a box covers, cleared at
/// the low end and clamped to the axis's cell count at the top.
fn cell_span(inv_scaled: f32, axis_min: f32, low: f32, high: f32, cells: i32) -> u32 {
    let mut first = (inv_scaled * (low - axis_min)) as i32;
    let last = ((inv_scaled * (high - axis_min)) as i32).clamp(0, cells);
    let mut bits = (1u32 << (last + 1)) - 1;
    if first > 0 {
        first = (first - 1).min(cells);
        bits &= !((1u32 << first) - 1);
    }
    bits
}

/// `cCcD_Stts` (`c_cc_d.h:326`) as the contact pipeline reads it: the raw weight byte, the process
/// id, the owning actor and the accumulated collision move.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Stts {
    /// `GetWeightUc`, the byte `GetRank` buckets and `rank_tbl` indexes with.
    pub weight_uc: u8,
    /// `GetApid`.
    pub apid: u32,
    /// `GetActor`; `None` is `NULL`, and two statuses naming the same actor never correct.
    pub actor: Option<u32>,
    /// `m_cc_move`, accumulated by `PlusCcMove`.
    pub cc_move: Vec3,
    /// `m_dmg`, which `PlusDmg` only ever raises.
    pub dmg: u8,
}

impl Stts {
    /// `cCcD_Stts::PlusCcMove`.
    pub fn plus_cc_move(&mut self, v: Vec3) {
        self.cc_move += v;
    }

    /// `cCcD_Stts::PlusDmg`: a lower damage leaves the accumulator alone.
    pub fn plus_dmg(&mut self, dmg: u8) {
        if self.dmg < dmg {
            self.dmg = dmg;
        }
    }
}

/// `dCcD_GStts` (`d_cc_d.h:178`): the four process ids and the no-actor flag
/// `ChkAtTgHitAfterCross` writes and reads.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GStts {
    pub at_apid: u32,
    pub at_old_apid: u32,
    pub tg_apid: u32,
    pub tg_old_apid: u32,
    pub flag: u32,
}

impl GStts {
    /// `SetAtApid`.
    pub fn set_at_apid(&mut self, apid: u32) {
        self.at_apid = apid;
    }

    /// `SetTgApid`.
    pub fn set_tg_apid(&mut self, apid: u32) {
        self.tg_apid = apid;
    }

    /// `GetAtOldApid`.
    pub fn at_old_apid(&self) -> u32 {
        self.at_old_apid
    }

    /// `GetTgOldApid`.
    pub fn tg_old_apid(&self) -> u32 {
        self.tg_old_apid
    }

    /// `ChkNoActor`.
    pub fn chk_no_actor(&self) -> bool {
        self.flag & GSTTS_NO_ACTOR != 0
    }
}

/// `dCcD_GObjInf` (`d_cc_d.h:376`) reduced to what contact resolution reads and writes: the three
/// general `mSPrm` words, the CO response word and the CO contact process id.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GObjInf {
    /// `mGObjAt.mSPrm`.
    pub at_sprm: u32,
    /// `mGObjTg.mSPrm`.
    pub tg_sprm: u32,
    /// `mGObjCo.mSPrm`.
    pub co_sprm: u32,
    /// `mGObjCo.mRPrm`, whose only bit here is `dCcG_CoRPrm_HitNoActor_e`.
    pub co_rprm: u32,
    /// `mGObjCo.mApid`, written by `SetCoHitApid`.
    pub co_hit_apid: u32,
}

impl GObjInf {
    /// `ChkAtNoConHit`.
    pub fn chk_at_no_con_hit(&self) -> bool {
        self.at_sprm & G_AT_SPRM_NO_CON_HIT != 0
    }

    /// `ChkTgNoConHit`.
    pub fn chk_tg_no_con_hit(&self) -> bool {
        self.tg_sprm & G_TG_SPRM_NO_CON_HIT != 0
    }

    /// `ChkAtStopNoConHit`.
    pub fn chk_at_stop_no_con_hit(&self) -> bool {
        self.at_sprm & G_AT_SPRM_STOP_NO_CON_HIT != 0
    }

    /// `ChkCoAtLasso`.
    pub fn chk_co_at_lasso(&self) -> bool {
        self.co_sprm & G_CO_SPRM_AT_LASSO != 0
    }

    /// `ChkCoTgLasso`.
    pub fn chk_co_tg_lasso(&self) -> bool {
        self.co_sprm & G_CO_SPRM_TG_LASSO != 0
    }

    /// `SetCoHitApid`.
    pub fn set_co_hit_apid(&mut self, apid: u32) {
        self.co_hit_apid = apid;
    }

    /// `OnCoHitNoActor`.
    pub fn on_co_hit_no_actor(&mut self) {
        self.co_rprm |= G_CO_RPRM_HIT_NO_ACTOR;
    }

    /// `ChkCoHitNoActor`.
    pub fn chk_co_hit_no_actor(&self) -> bool {
        self.co_rprm & G_CO_RPRM_HIT_NO_ACTOR != 0
    }
}

/// The three hit-info sub-objects of a `cCcD_Obj` (`c_cc_d.h:411-528`) plus `mFlags`: the
/// registration words the reject chain tests and the hit bookkeeping writes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ObjSet {
    /// `mObjAt.mSPrm`.
    pub at_sprm: u32,
    /// `mObjAt.mRPrm`.
    pub at_rprm: u32,
    /// `mObjAt.mType`.
    pub at_type: u32,
    /// `mObjAt.mAtp`, the AT object's own damage.
    pub at_atp: u8,
    /// `mObjTg.mSPrm`.
    pub tg_sprm: u32,
    /// `mObjTg.mRPrm`.
    pub tg_rprm: u32,
    /// `mObjTg.mType`.
    pub tg_type: u32,
    /// `mObjCo.mSPrm`.
    pub co_sprm: u32,
    /// `mObjCo.mRPrm`.
    pub co_rprm: u32,
    /// `cCcD_Obj::mFlags`.
    pub flags: u32,
}

impl ObjSet {
    /// `ChkAtSet`.
    pub fn chk_at_set(&self) -> bool {
        self.at_sprm & AT_SPRM_SET != 0
    }

    /// `ChkTgSet`.
    pub fn chk_tg_set(&self) -> bool {
        self.tg_sprm & TG_SPRM_SET != 0
    }

    /// `ChkCoSet`.
    pub fn chk_co_set(&self) -> bool {
        self.co_sprm & CO_SPRM_SET != 0
    }

    /// `GetAtGrp`.
    pub fn at_grp(&self) -> u32 {
        self.at_sprm & AT_SPRM_GRP_ALL
    }

    /// `GetTgGrp`.
    pub fn tg_grp(&self) -> u32 {
        self.tg_sprm & TG_SPRM_GRP_ALL
    }

    /// `ChkAtNoTgHitInfSet`.
    pub fn chk_at_no_tg_hit_inf_set(&self) -> bool {
        self.at_sprm & AT_SPRM_NO_TG_HIT_INF_SET != 0
    }

    /// `ChkTgNoAtHitInfSet`.
    pub fn chk_tg_no_at_hit_inf_set(&self) -> bool {
        self.tg_sprm & TG_SPRM_NO_AT_HIT_INF_SET != 0
    }

    /// `ChkCoNoCoHitInfSet`.
    pub fn chk_co_no_co_hit_inf_set(&self) -> bool {
        self.co_sprm & CO_SPRM_NO_CO_HIT_INF_SET != 0
    }

    /// `ChkCoNoCrr`.
    pub fn chk_co_no_crr(&self) -> bool {
        self.co_sprm & CO_SPRM_NO_CRR != 0
    }

    /// `ChkCoSph3DCrr`.
    pub fn chk_co_sph_3d_crr(&self) -> bool {
        self.co_sprm & CO_SPRM_SPH_3D_CRR != 0
    }

    /// `GetCoIGrp`.
    pub fn co_i_grp(&self) -> u32 {
        self.co_sprm & CO_SPRM_I_GRP_ALL
    }

    /// `GetCoVsGrp`.
    pub fn co_vs_grp(&self) -> u32 {
        self.co_sprm & CO_SPRM_VS_GRP_ALL
    }

    /// `ChkBsRevHit`.
    pub fn chk_bs_rev_hit(&self) -> bool {
        self.flags & OBJ_FLAG_BS_REV_HIT != 0
    }

    /// `SetAtHit`, whose hit-object pointer half has no field here.
    pub fn set_at_hit(&mut self) {
        self.at_rprm |= AT_RPRM_HIT;
    }

    /// `SetTgHit`.
    pub fn set_tg_hit(&mut self) {
        self.tg_rprm |= TG_RPRM_HIT;
    }

    /// `SetCoHit`.
    pub fn set_co_hit(&mut self) {
        self.co_rprm |= CO_RPRM_HIT;
    }

    /// `ChkAtHit`.
    pub fn chk_at_hit(&self) -> bool {
        self.at_rprm & AT_RPRM_HIT != 0
    }

    /// `ChkTgHit`.
    pub fn chk_tg_hit(&self) -> bool {
        self.tg_rprm & TG_RPRM_HIT != 0
    }

    /// `ChkCoHit`.
    pub fn chk_co_hit(&self) -> bool {
        self.co_rprm & CO_RPRM_HIT != 0
    }
}

/// One `cCcD_Obj` as the `cCcS` lists hold it: which shape it carries, the three hit-info
/// sub-objects, its status and general status, its general object info and its divide info.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ObjEntry {
    /// The index into [`Shapes::bodies`]. `None` is a `*pObj == NULL` list slot, which the AT, TG
    /// and CO loops all skip.
    pub shape: Option<usize>,
    /// `mObjAt`, `mObjTg`, `mObjCo` and `mFlags`.
    pub set: ObjSet,
    /// `GetStts()`; `None` is the `NULL` status pointer.
    pub stts: Option<Stts>,
    /// `GetStts()->GetGStts()`, which `dCcD_Stts` answers with the status itself.
    pub g_stts: Option<GStts>,
    /// `GetGObjInf()`.
    pub gobj_inf: Option<GObjInf>,
    /// `GetPDivideInfo()`, filled by [`calc_area`].
    pub divide: DivideInfo,
}

/// The `cCcS` registration lists: `mpObj` with its per-object records, and the AT, TG and CO
/// sublists of indices into it, standing in for the four `cCcD_Obj*` arrays and their counts.
#[derive(Resource, Debug, Default, Clone)]
pub struct CcSystem {
    /// `mpObj` / `mObjCount`.
    pub objs: Vec<ObjEntry>,
    /// `mpObjAt` / `mObjAtCount`.
    pub at: Vec<usize>,
    /// `mpObjTg` / `mObjTgCount`.
    pub tg: Vec<usize>,
    /// `mpObjCo` / `mObjCoCount`.
    pub co: Vec<usize>,
}

impl CcSystem {
    /// `cCcS::Move`'s tail (`c_cc_s.cpp:437-440`): all four object lists are emptied and their
    /// counts go back to zero.
    pub fn clear_registrations(&mut self) {
        self.objs.clear();
        self.at.clear();
        self.tg.clear();
        self.co.clear();
    }
}

/// One recorded `cCcS::ChkAtTg` hit: the two object indices, the `i_setAt` / `i_setTg` pair
/// `SetAtTgCommonHitInf` derived, and the cross point it was handed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtTgHit {
    pub at: usize,
    pub tg: usize,
    pub set_at: bool,
    pub set_tg: bool,
    pub cross: Vec3,
}

/// One recorded `cCcS::ChkCo` contact: the two object indices, both shapes' `GetCoCP` contact
/// points and the `CrossCo` overlap.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoHit {
    pub obj1: usize,
    pub obj2: usize,
    pub pos1: Vec3,
    pub pos2: Vec3,
    pub cross_len: f32,
}

/// The shape of the object, or `None` for a `NULL` list slot or a `NULL` `GetShapeAttr`, the two
/// skips every loop opens with.
fn obj_shape<'a>(shapes: &'a Shapes, obj: &ObjEntry) -> Option<&'a Shape> {
    let body = shapes.body(obj.shape?)?;
    Some(&body.shape)
}

/// `cCcD_Obj::GetAc`: the status's owning actor, `NULL` when the object has no status.
fn obj_actor(obj: &ObjEntry) -> Option<u32> {
    obj.stts.and_then(|stts| stts.actor)
}

/// The radial reach of a shape, the same figure `Shape::contains` bounds a point by.
fn radius_of(shape: &Shape) -> f32 {
    let scaled = shape.radius * shape.scale.x;
    match shape.kind {
        ShapeKind::Sphere | ShapeKind::Cylinder | ShapeKind::Capsule => scaled,
        ShapeKind::Box => shape.scale.length() * 0.5,
        ShapeKind::Triangle => shape.verts.iter().fold(0.0f32, |far, v| {
            far.max((*v - shape.centroid()).length())
        }),
    }
}

/// The nearest point of a shape's axis: its centre for a sphere or a box, its segment clamped in
/// y for a cylinder or a capsule.
fn core_point(shape: &Shape, from: Vec3) -> Vec3 {
    match shape.kind {
        ShapeKind::Cylinder | ShapeKind::Capsule => {
            let half = (shape.height * 0.5).max(0.0);
            let y = (from.y - shape.pos.y).clamp(-half, half);
            Vec3::new(shape.pos.x, shape.pos.y + y, shape.pos.z)
        }
        ShapeKind::Sphere | ShapeKind::Box => shape.pos,
        ShapeKind::Triangle => shape.centroid(),
    }
}

/// `cM3dGAab::SetMinMax` over a whole box, the order `cCcS::CalcArea` widens the scene box in.
fn absorb(area: &mut Aab, aab: Aab) {
    area.set_min_max(aab.min);
    area.set_min_max(aab.max);
}

/// `cCcD_ShapeAttr::CalcAabBox` for the four shapes the port carries: the radius box around a
/// sphere centre, the half-height box of a cylinder measured from its centre, a capsule's segment
/// grown by its radius, and a yaw-rotated box's own swept extents.
pub fn calc_aab_box(shape: &Shape) -> Aab {
    let radius = shape.radius * shape.scale.x;
    let half = (shape.height * 0.5).max(0.0);
    match shape.kind {
        ShapeKind::Sphere => Aab {
            min: shape.pos - Vec3::splat(radius),
            max: shape.pos + Vec3::splat(radius),
        },
        ShapeKind::Cylinder | ShapeKind::Capsule => Aab {
            min: Vec3::new(
                shape.pos.x - radius,
                shape.pos.y - half,
                shape.pos.z - radius,
            ),
            max: Vec3::new(
                shape.pos.x + radius,
                shape.pos.y + half,
                shape.pos.z + radius,
            ),
        },
        ShapeKind::Box => {
            let half = shape.scale * 0.5;
            let (sin_y, cos_y) = shape.yaw.sin_cos();
            Aab {
                min: Vec3::new(
                    shape.pos.x - (half.x * cos_y.abs() + half.z * sin_y.abs()),
                    shape.pos.y - half.y,
                    shape.pos.z - (half.x * sin_y.abs() + half.z * cos_y.abs()),
                ),
                max: Vec3::new(
                    shape.pos.x + (half.x * cos_y.abs() + half.z * sin_y.abs()),
                    shape.pos.y + half.y,
                    shape.pos.z + (half.x * sin_y.abs() + half.z * cos_y.abs()),
                ),
            }
        }
        ShapeKind::Triangle => {
            let mut aab = Aab::clear_for_min_max();
            for corner in shape.verts {
                aab.set_min_max(corner);
            }
            aab
        }
    }
}

/// `cCcD_ShapeAttr::GetCoCP`: a sphere and a cylinder answer their own centre, a capsule and a box
/// inherit the base class's shared `cXyz::Zero` (`c_cc_d.cpp:14`).
pub fn co_cp(shape: &Shape) -> Vec3 {
    match shape.kind {
        ShapeKind::Sphere | ShapeKind::Cylinder => shape.pos,
        ShapeKind::Capsule | ShapeKind::Box | ShapeKind::Triangle => Vec3::ZERO,
    }
}

/// `cCcD_ShapeAttr::CrossAtTg`: whether the AT shape reaches the TG shape and, when it does, where
/// on the TG shape it lands.
///
/// The reference dispatches to a per-pair `cM3d_Cross_*` overload; those primitives are outside
/// this file, so the boolean is the shape system's own overlap test and the point follows the
/// `cM3d_Cross_SphSph` (`c_m3d.cpp:1449-1467`) convention, the spot on the TG shape's surface
/// along the line back to the AT shape's centre.
pub fn cross_at_tg(at: &Shape, tg: &Shape, cross: &mut Vec3) -> bool {
    if !at.intersects(tg) {
        return false;
    }
    match tg.kind {
        ShapeKind::Triangle => {
            *cross = tri_sph_cross_point(tg.verts, at.pos, radius_of(at));
        }
        ShapeKind::Sphere | ShapeKind::Cylinder | ShapeKind::Capsule | ShapeKind::Box => {
            let core = core_point(tg, at.pos);
            let delta = at.pos - core;
            *cross = if is_zero(delta.length()) {
                core
            } else {
                core + delta.normalize() * radius_of(tg)
            };
        }
    }
    true
}

/// `cCcD_ShapeAttr::CrossCo`: whether the two shapes touch and, when they do, how deeply they
/// overlap along the line between their axes, with a zero length on a grazing contact exactly as
/// `cM3d_Cross_SphSph` (`c_m3d.cpp:1423-1446`) reports it.
pub fn cross_co(one: &Shape, two: &Shape, cross_len: &mut f32) -> bool {
    if one.is_triangle() || two.is_triangle() {
        *cross_len = 0.0;
        return false;
    }
    let near = core_point(one, two.pos);
    let far = core_point(two, near);
    let depth = radius_of(one) + radius_of(two) - (near - far).length();
    if depth > F_ABS_MIN {
        *cross_len = depth;
        true
    } else {
        *cross_len = 0.0;
        false
    }
}

/// `cCcS::ChkNoHitAtTg` (`c_cc_s.cpp:101-112`): the AT/TG filter. The same actor on both sides, no
/// shared group, or no shared attack type rejects the pair.
///
/// The reference then calls `ChkNoHitGAtTg`, which `cCcS` inherits as a stub returning `false`
/// (`c_cc_s.h:63`), so the answer is settled here; the status it dereferences on that tail is
/// only reached through this function's own actor test.
pub fn chk_no_hit_at_tg(at: &ObjEntry, tg: &ObjEntry) -> bool {
    let ac_at = obj_actor(at);
    let ac_tg = obj_actor(tg);
    if (ac_at.is_some() && ac_tg.is_some() && ac_at == ac_tg)
        || (at.set.at_grp() & tg.set.tg_grp()) == 0
        || (at.set.at_type & tg.set.tg_type) == 0
    {
        return true;
    }
    false
}

/// `cCcS::ChkNoHitCo` (`c_cc_s.cpp:154-166`): the CO filter, read as the reference's single
/// negated conjunction. A pair is kept only when the two objects name different actors or one
/// names none, each one's own group is among the other's versus groups, and `ChkNoHitGCo` lets it
/// through.
pub fn chk_no_hit_co(obj1: &ObjEntry, obj2: &ObjEntry) -> bool {
    let ac1 = obj_actor(obj1);
    let ac2 = obj_actor(obj2);
    let pass = (ac1.is_none() || ac2.is_none() || ac1 != ac2)
        && (obj1.set.co_i_grp() & (obj2.set.co_vs_grp() >> 3)) != 0
        && ((obj1.set.co_vs_grp() >> 3) & obj2.set.co_i_grp()) != 0
        && !chk_no_hit_g_co(obj1, obj2);
    !pass
}

/// `dCcS::ChkNoHitGCo` (`d_cc_s.cpp:588-595`): the lasso filter. An object that is a lasso target
/// while the other is not a lasso attacker rejects the pair; an object with no general object info
/// carries no lasso flag and so rejects nothing.
pub fn chk_no_hit_g_co(obj1: &ObjEntry, obj2: &ObjEntry) -> bool {
    let (Some(inf1), Some(inf2)) = (obj1.gobj_inf, obj2.gobj_inf) else {
        return false;
    };
    (inf1.chk_co_at_lasso() && !inf2.chk_co_tg_lasso())
        || (inf2.chk_co_at_lasso() && !inf1.chk_co_tg_lasso())
}

/// `cCcS::CalcArea` (`c_cc_s.cpp:405-429`): fold every registered object's work box into one scene
/// box, then bucket each object's own box against that scene.
///
/// The first loop skips a null object and a null shape attribute and is what fills
/// `DivideInfo::aab`; the second repeats both skips and adds the `ChkBsRevHit` test, so an object
/// with no shape keeps the divide range it arrived with.
pub fn calc_area(shapes: &Shapes, system: &mut CcSystem) -> DivideArea {
    let mut area = Aab::clear_for_min_max();
    for obj in &mut system.objs {
        let Some(shape) = obj_shape(shapes, obj) else {
            continue;
        };
        let aab = calc_aab_box(shape);
        obj.divide.aab = aab;
        absorb(&mut area, aab);
    }
    let mut divide = DivideArea::default();
    divide.set_area(area);
    for obj in &mut system.objs {
        if obj.shape.is_none() || obj_shape(shapes, obj).is_none() {
            continue;
        }
        let aab = obj.divide.aab;
        let bs_rev_hit = obj.set.chk_bs_rev_hit();
        divide.calc_divide_info(&mut obj.divide, aab, bs_rev_hit);
    }
    divide
}

/// `cCcS::ClrAtHitInf` then `cCcS::ClrTgHitInf`, the pair `ChkAtTg` opens with.
pub fn clr_at_tg_hit_inf(system: &mut CcSystem) {
    for obj in &mut system.objs {
        obj.set.at_rprm &= !AT_RPRM_HIT;
        obj.set.tg_rprm &= !TG_RPRM_HIT;
    }
}

/// `cCcS::ClrCoHitInf`, the call `ChkCo` opens with.
pub fn clr_co_hit_inf(system: &mut CcSystem) {
    for obj in &mut system.objs {
        obj.set.co_rprm &= !CO_RPRM_HIT;
    }
}

/// `cCcS::ChkAtTg` (`c_cc_s.cpp:115-151`): the AT-by-TG double loop.
///
/// The reject chain is the reference's and in its order: a null object or an AT that is not set
/// and a null AT shape attribute, then per target a null object or a TG that is not set, the
/// divide-area bucket reject, `ChkNoHitAtTg`, and a null TG shape attribute. Both lists are
/// snapshotted first, as the reference takes `objTgEnd` before the loop.
///
/// Of the four combinations of `ChkBsRevHit` and the crossing only `(!any, did)` and `(any, !did)`
/// record a hit; the other two fall through with nothing recorded, which is what the reference's
/// `if` / `else if` leaves them.
pub fn chk_at_tg(shapes: &Shapes, system: &mut CcSystem) -> Vec<AtTgHit> {
    clr_at_tg_hit_inf(system);
    let at_list = system.at.clone();
    let tg_list = system.tg.clone();
    let mut hits = Vec::new();
    for &at_index in &at_list {
        let Some(at) = system.objs.get(at_index).copied() else {
            continue;
        };
        if at.shape.is_none() || !at.set.chk_at_set() {
            continue;
        }
        let Some(at_shape) = obj_shape(shapes, &at) else {
            continue;
        };
        for &tg_index in &tg_list {
            let Some(tg) = system.objs.get(tg_index).copied() else {
                continue;
            };
            if tg.shape.is_none() || !tg.set.chk_tg_set() {
                continue;
            }
            if !at.divide.chk(&tg.divide) {
                continue;
            }
            if chk_no_hit_at_tg(&at, &tg) {
                continue;
            }
            let Some(tg_shape) = obj_shape(shapes, &tg) else {
                continue;
            };
            let mut cross = Vec3::ZERO;
            let did_cross = cross_at_tg(at_shape, tg_shape, &mut cross);
            let any_bs_rev_hit = at.set.chk_bs_rev_hit() || tg.set.chk_bs_rev_hit();
            if !any_bs_rev_hit && did_cross {
                set_at_tg_common_hit_inf(system, at_index, tg_index, cross, &mut hits);
            } else if any_bs_rev_hit && !did_cross {
                cross = calc_aab_box(at_shape).calc_center();
                set_at_tg_common_hit_inf(system, at_index, tg_index, cross, &mut hits);
            }
        }
    }
    hits
}

/// `cCcS::CalcTgPlusDmg` (`c_cc_s.cpp:202-205`): the AT object's own damage onto the target's
/// accumulator. The `dCcS` override of this is shield-gated and belongs to the shield layer.
pub fn calc_tg_plus_dmg(at: &ObjSet, tg_stts: &mut Option<Stts>) {
    if let Some(stts) = tg_stts {
        stts.plus_dmg(at.at_atp);
    }
}

/// `dCcS::ChkAtTgHitAfterCross` (`d_cc_s.cpp:136-159`): record each side's partner, then report
/// whether the pair is one the no-continuous-hit bookkeeping has already retired.
///
/// Each side writes its own process id before its own test and returns on it, so a retired AT
/// pair leaves the TG slot unwritten, as in the reference. A side with no status or no general
/// status has nothing to compare against and contributes neither the write nor the test.
pub fn chk_at_tg_hit_after_cross(
    set_at: bool,
    set_tg: bool,
    at: &mut ObjEntry,
    tg: &mut ObjEntry,
) -> bool {
    let at_apid = at.stts.map(|stts| stts.apid);
    let tg_apid = tg.stts.map(|stts| stts.apid);
    if set_at {
        if let (Some(g_stts), Some(apid)) = (at.g_stts.as_mut(), tg_apid) {
            g_stts.set_at_apid(apid);
            let no_con_hit = at.gobj_inf.is_some_and(|inf| inf.chk_at_no_con_hit());
            if no_con_hit && g_stts.at_old_apid() == apid {
                return true;
            }
        }
    }
    if set_tg {
        if let (Some(g_stts), Some(apid)) = (tg.g_stts.as_mut(), at_apid) {
            g_stts.set_tg_apid(apid);
            let at_stop_no_con_hit = at.gobj_inf.is_some_and(|inf| inf.chk_at_stop_no_con_hit());
            if tg.gobj_inf.is_some_and(|inf| inf.chk_tg_no_con_hit())
                && !at_stop_no_con_hit
                && g_stts.tg_old_apid() == apid
            {
                return true;
            }
        }
    }
    false
}

/// `cCcS::SetAtTgCommonHitInf` (`c_cc_s.cpp:208-231`).
///
/// A retired pair records nothing. Otherwise the AT side takes its hit bit, the TG side takes its
/// own and the damage, and the pair is recorded with the cross point; `SetAtTgGObjInf`'s hit-mark,
/// hit-sound, particle and split writes belong to the mark layer and are left to the caller.
pub fn set_at_tg_common_hit_inf(
    system: &mut CcSystem,
    at_index: usize,
    tg_index: usize,
    cross: Vec3,
    hits: &mut Vec<AtTgHit>,
) {
    let Some(mut at) = system.objs.get(at_index).copied() else {
        return;
    };
    let Some(mut tg) = system.objs.get(tg_index).copied() else {
        return;
    };
    let set_at = !tg.set.chk_tg_no_at_hit_inf_set();
    let set_tg = !at.set.chk_at_no_tg_hit_inf_set();
    let retired = chk_at_tg_hit_after_cross(set_at, set_tg, &mut at, &mut tg);
    if set_at {
        at.set.set_at_hit();
    }
    if set_tg {
        tg.set.set_tg_hit();
        calc_tg_plus_dmg(&at.set, &mut tg.stts);
    }
    system.objs[at_index] = at;
    system.objs[tg_index] = tg;
    if retired {
        return;
    }
    hits.push(AtTgHit {
        at: at_index,
        tg: tg_index,
        set_at,
        set_tg,
        cross,
    });
}

/// `cCcS::ChkCo` (`c_cc_s.cpp:169-199`): the pairwise `i < j` loop over the CO list, against the
/// retail `cross_len = 0.0f` initialisation.
///
/// The chain is the reference's: a null object or a CO that is not set and a null CO shape
/// attribute, then per partner a null object or a CO that is not set, the divide-area bucket
/// reject, `ChkNoHitCo`, and a null partner shape attribute. The reference's `if (objCo2 == NULL)`
/// re-tests the pointer its first inner reject already rejected, so it can never fire.
pub fn chk_co(shapes: &Shapes, system: &mut CcSystem) -> Vec<CoHit> {
    clr_co_hit_inf(system);
    let mut hits = Vec::new();
    let list = system.co.clone();
    if list.len() <= 1 {
        return hits;
    }
    for (i, &one_index) in list.iter().enumerate() {
        let Some(one) = system.objs.get(one_index).copied() else {
            continue;
        };
        if one.shape.is_none() || !one.set.chk_co_set() {
            continue;
        }
        let Some(one_shape) = obj_shape(shapes, &one) else {
            continue;
        };
        let one_cp = co_cp(one_shape);
        for &two_index in &list[i + 1..] {
            let Some(two) = system.objs.get(two_index).copied() else {
                continue;
            };
            if two.shape.is_none() || !two.set.chk_co_set() {
                continue;
            }
            if !one.divide.chk(&two.divide) {
                continue;
            }
            if chk_no_hit_co(&one, &two) {
                continue;
            }
            let Some(two_shape) = obj_shape(shapes, &two) else {
                continue;
            };
            let mut cross_len = 0.0f32;
            if cross_co(one_shape, two_shape, &mut cross_len) {
                let two_cp = co_cp(two_shape);
                set_co_common_hit_inf(
                    system, one_index, two_index, one_cp, two_cp, cross_len, &mut hits,
                );
            }
        }
    }
    hits
}

/// `dCcS::SetCoGObjInf` (`d_cc_s.cpp:162-192`) without its two `dCcD_HitCallback` dispatches: the
/// per-side contact process id and the no-actor flag it raises, keyed on each side's own
/// `i_co*Set`.
fn set_co_g_obj_inf(system: &mut CcSystem, obj1: usize, obj2: usize, set1: bool, set2: bool) {
    let first = system.objs[obj1];
    let second = system.objs[obj2];
    if set1 {
        set_co_contact(
            system,
            obj1,
            second.stts.map(|stts| stts.apid),
            second.g_stts.is_some_and(|g| g.chk_no_actor()),
        );
    }
    if set2 {
        set_co_contact(
            system,
            obj2,
            first.stts.map(|stts| stts.apid),
            first.g_stts.is_some_and(|g| g.chk_no_actor()),
        );
    }
}

/// One side of `dCcS::SetCoGObjInf`: the partner's process id, then the no-actor flag when it was
/// that partner's general status that had no actor.
fn set_co_contact(system: &mut CcSystem, obj: usize, apid: Option<u32>, partner_no_actor: bool) {
    let Some(inf) = &mut system.objs[obj].gobj_inf else {
        return;
    };
    if let Some(apid) = apid {
        inf.set_co_hit_apid(apid);
    }
    if partner_no_actor {
        inf.on_co_hit_no_actor();
    }
}

/// `cCcS::SetCoCommonHitInf` (`c_cc_s.cpp:234-250`): the CO bookkeeping of one crossing pair,
/// including the position correction, and returning the two applied offsets when it ran. The
/// correction lands on copies of the two contact points, so the recorded pair keeps the values
/// `GetCoCP` reported.
pub fn set_co_common_hit_inf(
    system: &mut CcSystem,
    obj1: usize,
    obj2: usize,
    pos1: Vec3,
    pos2: Vec3,
    cross_len: f32,
    hits: &mut Vec<CoHit>,
) -> Option<(Vec3, Vec3)> {
    let first = *system.objs.get(obj1)?;
    let second = *system.objs.get(obj2)?;
    let obj2_co_hit_inf_set = !second.set.chk_co_no_co_hit_inf_set();
    let obj1_co_hit_inf_set = !first.set.chk_co_no_co_hit_inf_set();
    if obj2_co_hit_inf_set {
        system.objs[obj1].set.set_co_hit();
    }
    if obj1_co_hit_inf_set {
        system.objs[obj2].set.set_co_hit();
    }
    let mut first_pos = pos1;
    let mut second_pos = pos2;
    let corrected = if obj2_co_hit_inf_set && obj1_co_hit_inf_set {
        set_pos_correct(
            system,
            obj1,
            obj2,
            cross_len,
            &mut first_pos,
            &mut second_pos,
        )
    } else {
        None
    };
    set_co_g_obj_inf(system, obj1, obj2, obj2_co_hit_inf_set, obj1_co_hit_inf_set);
    hits.push(CoHit {
        obj1,
        obj2,
        pos1,
        pos2,
        cross_len,
    });
    corrected
}

/// `cCcS::SetCoGCorrectProc` (`c_cc_s.cpp:462-464`): the hook `SetPosCorrect` calls before it
/// weighs the two objects, a four-byte stub in the retail build whose two arguments are unused
/// there as well.
pub fn set_co_g_correct_proc() {}

/// `dCcS::SetPosCorrect` (`d_cc_s.cpp:236-325`): push two touching objects apart along the line
/// between them, split by their weights.
///
/// `pos1` and `pos2` are the pair `SetCoCommonHitInf` was handed, which `ChkCo` fills from the two
/// shapes' `GetCoCP`; the reference adds the two offsets to those same pointers in place, so they
/// are taken mutably here and the offsets are also returned, or `None` when one of the four
/// early-outs fired.
pub fn set_pos_correct(
    system: &mut CcSystem,
    obj1: usize,
    obj2: usize,
    cross_len: f32,
    pos1: &mut Vec3,
    pos2: &mut Vec3,
) -> Option<(Vec3, Vec3)> {
    let first = *system.objs.get(obj1)?;
    let second = *system.objs.get(obj2)?;
    if first.set.chk_co_no_crr() || second.set.chk_co_no_crr() {
        return None;
    }
    let (Some(stts1), Some(stts2)) = (first.stts, second.stts) else {
        return None;
    };
    if let (Some(owner1), Some(owner2)) = (stts1.actor, stts2.actor)
        && owner1 == owner2
    {
        return None;
    }
    if is_zero(cross_len) {
        return None;
    }
    set_co_g_correct_proc();
    let correct_y = first.set.chk_co_sph_3d_crr() && second.set.chk_co_sph_3d_crr();
    let (weight1, weight2) = (stts1.weight_uc, stts2.weight_uc);
    if (weight1 == 0 && weight2 == 0) || (weight1 == 0xFF && weight2 == 0xFF) {
        return None;
    }
    let rank = RANK_TBL[get_rank(weight1)][get_rank(weight2)];
    let inv_rank = 100 - rank;
    let obj2_weight = f32::from(rank) * 0.01;
    let obj1_weight = f32::from(inv_rank) * 0.01;
    let mut objs_dist = Vec3::new(pos2.x - pos1.x, 0.0, pos2.z - pos1.z);
    if correct_y {
        objs_dist.y = pos2.y - pos1.y;
    }
    let obj_dist_len = objs_dist.length();
    let (vec1, vec2) = if !is_zero(obj_dist_len) {
        let push_factor = cross_len / obj_dist_len;
        if correct_y {
            let scaled = objs_dist * push_factor;
            (-scaled * obj2_weight, scaled * obj1_weight)
        } else {
            let scaled = Vec3::new(objs_dist.x * push_factor, 0.0, objs_dist.z * push_factor);
            (-scaled * obj2_weight, scaled * obj1_weight)
        }
    } else {
        let mut away = Vec3::new(0.0, 0.0, 0.0);
        let mut back = Vec3::new(0.0, 0.0, 0.0);
        if !is_zero(cross_len) {
            away.x = -cross_len * obj2_weight;
            back.x = cross_len * obj1_weight;
        } else {
            away.x = -obj2_weight;
            back.x = obj1_weight;
        }
        (away, back)
    };
    if let Some(stts) = &mut system.objs[obj1].stts {
        stts.plus_cc_move(vec1);
    }
    if let Some(stts) = &mut system.objs[obj2].stts {
        stts.plus_cc_move(vec2);
    }
    *pos1 += vec1;
    *pos2 += vec2;
    Some((vec1, vec2))
}

/// `dCcS::GetRank` (`d_cc_s.cpp:209-233`): the eleven-way weight ladder, in the reference's order.
pub fn get_rank(weight: u8) -> usize {
    if weight == 0xFF {
        10
    } else if weight == 0xFE {
        9
    } else if weight >= 0xD9 {
        8
    } else if weight >= 0xB5 {
        7
    } else if weight >= 0x91 {
        6
    } else if weight >= 0x6D {
        5
    } else if weight >= 0x49 {
        4
    } else if weight >= 0x25 {
        3
    } else if weight >= 0x02 {
        2
    } else if weight == 0x01 {
        1
    } else {
        0
    }
}

/// The `rank_tbl` of `d_cc_s.cpp:194-206`, indexed by the first object's rank then the second's:
/// the share of the push the second object takes.
pub const RANK_TBL: [[u8; 11]; 11] = [
    [0, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100],
    [0, 50, 100, 100, 100, 100, 100, 100, 100, 100, 100],
    [0, 0, 50, 75, 90, 100, 100, 100, 100, 100, 100],
    [0, 0, 25, 50, 75, 90, 100, 100, 100, 100, 100],
    [0, 0, 10, 25, 50, 75, 90, 100, 100, 100, 100],
    [0, 0, 0, 10, 25, 50, 75, 90, 100, 100, 100],
    [0, 0, 0, 0, 10, 25, 50, 75, 90, 100, 100],
    [0, 0, 0, 0, 0, 10, 25, 50, 75, 100, 100],
    [0, 0, 0, 0, 0, 0, 10, 25, 50, 100, 100],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 50, 100],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];

/// `dCcS::MoveAfterCheck` (`d_cc_s.cpp:500-502`): a four-byte `return` stub, empty in the retail
/// build and kept as the hook it is.
pub fn move_after_check() {}

/// `cCcS::Move` (`c_cc_s.cpp:432-441`), which `dCcS::Move` (`d_cc_s.cpp:508-510`) only calls:
/// calc the area, check AT against TG, check CO, run the retail move hook, then drop every
/// registration.
pub fn move_pipeline(shapes: &Shapes, system: &mut CcSystem) -> (Vec<AtTgHit>, Vec<CoHit>) {
    calc_area(shapes, system);
    let at_tg = chk_at_tg(shapes, system);
    let co = chk_co(shapes, system);
    move_after_check();
    system.clear_registrations();
    (at_tg, co)
}
