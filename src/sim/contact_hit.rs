use bevy_ecs::prelude::*;
use glam::Vec3;

use crate::sim::bodies::AtMeta;
use crate::sim::contact::{
    self, AT_SPRM_NO_TG_HIT_INF_SET, AT_SPRM_SET, CcSystem, DivideInfo, GObjInf, GStts,
    GSTTS_NO_ACTOR, ObjEntry, ObjSet, Stts,
};
use crate::sim::contact_run::ContactBodies;
use crate::sim::damage_resolve::{
    Atk, Attacker, CcAtInfo, DamageFx, HitAudio, HitStop, PlayerCut, Target, cc_at_check,
};
use crate::sim::events::SimEvents;
use crate::sim::registry::ERROR_PROC_ID;
use crate::sim::shape::{Shape, ShapeKind, Shapes};

/// `cCcD_AtSPrm_VsEnemy_e` (`c_cc_d.h:22`).
const AT_SPRM_VS_ENEMY_BIT: u32 = 0x02;
/// `cCcD_AtSPrm_VsOther_e` (`c_cc_d.h:24`).
const AT_SPRM_VS_OTHER_BIT: u32 = 0x08;

/// `cCcD_AtSPrm_Set_e | cCcD_AtSPrm_VsEnemy_e | cCcD_AtSPrm_VsOther_e`, the AT mask the sword
/// (`player_main_data.inc:133`), the arrow (`arrow.cpp:22`) and the hammer register: the swing
/// reaches enemies and other processes and never the player, so a foe whose TG half names only
/// the player group is not struck.
pub const AT_SPRM_VS_ENEMY: u32 = AT_SPRM_SET | AT_SPRM_VS_ENEMY_BIT | AT_SPRM_VS_OTHER_BIT;

/// The same mask with `cCcD_AtSPrm_NoTgHitInfSet_e` (`c_cc_d.h:26`) on top, which is what
/// `daBoomerang_c::l_at_cps_src` (`boomerang.cpp:761`) carries and what keeps the boomerang from
/// writing hit information onto the body it reached.
pub const AT_SPRM_NO_TG_HIT_INF: u32 = AT_SPRM_VS_ENEMY | AT_SPRM_NO_TG_HIT_INF_SET;

/// One AT object, the attacker's own registration for the step its swing is live.
///
/// The reference keeps the attacker's AT half on the process itself, so a swing's parameters and
/// the geometry that reaches with them are one object: `setAtParam`
/// (`player_main.cpp:9577`) rewrites `SetAtType` / `SetAtAtp` / `SetAtSpl` / `SetAtSe` on the same
/// `cCcD_SrcCps` the weapon's `create` built, and `daArrow_c::arrowShooting` (`arrow.cpp:294-296`)
/// rewrites them again per shot. The [`AtMeta`] carries the `SrcObjAt Type` / `Atp` pair and the
/// `SrcGObjAt Se` / `Spl` pair, and the [`Attacker`] is `GetAc()`, so together they are the whole
/// `mpObj` a damage call site hands to `cc_at_check`.
#[derive(Debug, Clone)]
pub struct AtObj {
    /// The `cCcD_ShapeAttr` the object reaches with, already in world coordinates.
    pub shape: Shape,
    /// `mObjAt` plus the three words the AT/TG filter reads beside it.
    pub set: ObjSet,
    /// `dCcD_SrcGObjInf`, whose AT half the no-continuous-hit bookkeeping reads.
    pub gobj: GObjInf,
    /// `cCcD_Stts`, the status the object is registered against.
    pub stts: Stts,
    /// `cCcD_Stts::GetGStts`.
    pub g_stts: GStts,
    /// `mpObj` as [`Atk`].
    pub atk: Atk,
}

impl AtObj {
    /// `cCcD_Obj::Set(src)` with the AT half on and nothing else: `at_sprm` is the
    /// `SrcObjAt SPrm` the reference's own `create` wrote, `meta` is the `SrcObjAt Type` /
    /// `Atp` and the `SrcGObjAt Se` / `Spl` pair, and `ac` is the actor `GetAc()` answers with.
    ///
    /// `apid` is the process id the registry holds for the attacker, and `weight` the byte its
    /// own `cCcD_Stts::Init` was handed.
    pub fn new(
        shape: Shape,
        at_sprm: u32,
        meta: &AtMeta,
        ac: Attacker,
        weight: u8,
        apid: u32,
    ) -> Self {
        Self {
            shape,
            set: ObjSet {
                at_sprm,
                at_type: meta.at_type.0,
                at_atp: meta.at_atp,
                ..ObjSet::default()
            },
            gobj: GObjInf::default(),
            stts: Stts {
                weight_uc: weight,
                apid,
                actor: None,
                ..Stts::default()
            },
            g_stts: GStts {
                flag: GSTTS_NO_ACTOR,
                ..GStts::default()
            },
            atk: Atk {
                at_type: meta.at_type,
                at_se: meta.at_se,
                at_spl: meta.at_spl,
                at_atp: meta.at_atp,
                ac: Some(ac),
            },
        }
    }

    fn entry(&self, shape: Option<usize>) -> ObjEntry {
        ObjEntry {
            shape,
            set: self.set,
            stts: Some(self.stts),
            g_stts: Some(self.g_stts),
            gobj_inf: Some(self.gobj),
            divide: DivideInfo::default(),
        }
    }
}

/// One candidate the swing may reach: the TG bodies the entity registered, the weight its own
/// status carries and the process id the registry holds for it. This is what `contact_run`
/// registers for the same entity, read straight off the component instead of off the shared
/// pass's own lists.
#[derive(Debug, Clone, Copy)]
pub struct TgObj<'a> {
    pub entity: Entity,
    pub bodies: &'a ContactBodies,
    pub weight: u8,
    pub apid: Option<u32>,
}

impl<'a> TgObj<'a> {
    pub fn new(entity: Entity, bodies: &'a ContactBodies, weight: u8, apid: Option<u32>) -> Self {
        Self {
            entity,
            bodies,
            weight,
            apid,
        }
    }

    fn entry(&self, body: usize) -> ObjEntry {
        let obj = &self.bodies.objs[body];
        ObjEntry {
            shape: None,
            set: obj.set,
            stts: Some(Stts {
                weight_uc: self.weight,
                apid: self.apid.unwrap_or(ERROR_PROC_ID),
                actor: self.apid,
                ..Stts::default()
            }),
            g_stts: obj.g_stts,
            gobj_inf: obj.gobj_inf,
            divide: DivideInfo::default(),
        }
    }
}

/// One recorded AT-by-TG crossing, the answer a swing's pass gives a damage call site: which of
/// the attacker's AT objects crossed, which entity and which of its bodies it reached, and the
/// point `CrossAtTg` reported.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Charge {
    pub at: usize,
    pub target: Entity,
    pub tg_body: usize,
    pub cross: Vec3,
}

/// `cCcS::Move`'s AT half for one swing: register the attacker's AT objects and every candidate's
/// own TG bodies, run `CalcArea` and `ChkAtTg` over the two lists, and hand back the crossings.
///
/// `ChkAtTg` records one partner per object, and a damage call site charges an actor once, so a
/// target that several TG bodies reach takes only its first crossing. An entity with no
/// [`ContactBodies`] registers no TG object and so cannot be reached at all.
pub fn at_tg(at: &[AtObj], tg: &[TgObj<'_>]) -> Vec<Charge> {
    let mut shapes = Shapes::default();
    let mut system = CcSystem::default();
    let mut owners: Vec<Option<(Entity, usize)>> = Vec::new();
    for obj in at {
        let shape = shapes.add(obj.shape);
        system.at.push(system.objs.len());
        system.objs.push(obj.entry(Some(shape)));
        owners.push(None);
    }
    for target in tg {
        for (body, shape) in tg_bodies(target) {
            system.tg.push(system.objs.len());
            system.objs.push(ObjEntry {
                shape: Some(shapes.add(shape)),
                ..target.entry(body)
            });
            owners.push(Some((target.entity, body)));
        }
    }
    contact::calc_area(&shapes, &mut system);
    let mut charged: Vec<Charge> = Vec::new();
    for hit in contact::chk_at_tg(&shapes, &mut system) {
        let Some((target, tg_body)) = owners.get(hit.tg).copied().flatten() else {
            continue;
        };
        if charged.iter().any(|cross| cross.target == target) {
            continue;
        }
        charged.push(Charge {
            at: hit.at,
            target,
            tg_body,
            cross: hit.cross,
        });
    }
    charged
}

/// The same pass for an attack volume [`Shape`] cannot hold.
///
/// `Shape`'s capsule runs along the y axis alone, while `daArrow_c::arrowShooting`
/// (`arrow.cpp:294-295`) sets its AT capsule from `current.pos` to `current.pos + speed * 1.25f`,
/// which points wherever the shot points. The segment is therefore kept as it is and tested
/// against each candidate's own TG volumes with the same `ChkNoHitAtTg` filter, and the cross
/// point follows `CrossAtTg`'s convention: the TG core axis's own radius along the line back to
/// the attack volume's centre.
pub fn at_tg_swept(
    at: &AtObj,
    from: Vec3,
    to: Vec3,
    radius: f32,
    tg: &[TgObj<'_>],
) -> Vec<Charge> {
    let entry = at.entry(None);
    let mut charged: Vec<Charge> = Vec::new();
    for target in tg {
        for (body, shape) in tg_bodies(target) {
            if at.set.chk_bs_rev_hit() || target.bodies.objs[body].set.chk_bs_rev_hit() {
                continue;
            }
            if contact::chk_no_hit_at_tg(&entry, &target.entry(body)) {
                continue;
            }
            let Some(cross) = sweep_cross(&shape, from, to, radius) else {
                continue;
            };
            if charged.iter().any(|hit| hit.target == target.entity) {
                continue;
            }
            charged.push(Charge {
                at: 0,
                target: target.entity,
                tg_body: body,
                cross,
            });
        }
    }
    charged
}

/// `cc_at_check` for one crossing: the [`Atk`] the AT object carries, the struck actor's own
/// fields as `tgActor`, and the crossing's point as `pParticlePos`, which is what
/// `SetAtTgGObjInf` leaves in `GetAtHitPosP`.
pub fn resolve(
    atk: &Atk,
    cross: Vec3,
    struck: &mut Target,
    player: &PlayerCut,
    events: &mut SimEvents,
    stop: &mut HitStop,
    audio: &mut HitAudio,
    fx: &mut Vec<DamageFx>,
) -> Option<Attacker> {
    let mut info = CcAtInfo {
        mp_obj: Some(atk.clone()),
        p_particle_pos: Some(cross),
        ..Default::default()
    };
    cc_at_check(struck, &mut info, player, events, stop, audio, fx)
}

/// The candidate's own bodies that carry a TG object, with the shape each one reaches with.
fn tg_bodies(target: &TgObj<'_>) -> Vec<(usize, Shape)> {
    let mut bodies = Vec::new();
    for (body, obj) in target.bodies.objs.iter().enumerate() {
        if !obj.set.chk_tg_set() {
            continue;
        }
        if let Some(shape) = target.bodies.shapes.body(body) {
            bodies.push((body, shape.shape));
        }
    }
    bodies
}

/// The core axis of a shape: its centre for a sphere or a box, its own segment for a cylinder or
/// a capsule.
fn core(shape: &Shape) -> (Vec3, Vec3) {
    match shape.kind {
        ShapeKind::Cylinder | ShapeKind::Capsule => {
            let half = (shape.height * 0.5).max(0.0);
            (
                Vec3::new(shape.pos.x, shape.pos.y - half, shape.pos.z),
                Vec3::new(shape.pos.x, shape.pos.y + half, shape.pos.z),
            )
        }
        _ => (shape.pos, shape.pos),
    }
}

/// The core axis's own reach, the same figure `Shape::contains` bounds a point by.
fn core_reach(shape: &Shape) -> f32 {
    match shape.kind {
        ShapeKind::Box => shape.scale.length() * 0.5,
        _ => shape.radius * shape.scale.x,
    }
}

/// The nearest point of a core axis to `from`.
fn core_point(shape: &Shape, from: Vec3) -> Vec3 {
    let (low, high) = core(shape);
    if low == high {
        return low;
    }
    Vec3::new(
        from.x,
        from.y.clamp(low.y.min(high.y), low.y.max(high.y)),
        from.z,
    )
}

/// The distance from a point to a segment.
fn point_gap(point: Vec3, from: Vec3, to: Vec3) -> f32 {
    let travel = to - from;
    let len = travel.length_squared();
    if len <= f32::EPSILON {
        return (point - from).length();
    }
    let along = ((point - from).dot(travel) / len).clamp(0.0, 1.0);
    (point - (from + travel * along)).length()
}

/// The distance between two segments, which is the gap between the two volumes whose core axes
/// they are.
fn segment_gap(a0: Vec3, a1: Vec3, b0: Vec3, b1: Vec3) -> f32 {
    let d1 = a1 - a0;
    let d2 = b1 - b0;
    let r = a0 - b0;
    let len1 = d1.length_squared();
    let len2 = d2.length_squared();
    if len1 <= f32::EPSILON {
        return point_gap(a0, b0, b1);
    }
    if len2 <= f32::EPSILON {
        return point_gap(b0, a0, a1);
    }
    let across = d1.dot(d2);
    let to_a = d1.dot(r);
    let to_b = d2.dot(r);
    let denom = len1 * len2 - across * across;
    let mut s = if denom > f32::EPSILON {
        ((across * to_b - to_a * len2) / denom).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut t = (across * s + to_b) / len2;
    if t < 0.0 {
        t = 0.0;
        s = (-to_a / len1).clamp(0.0, 1.0);
    } else if t > 1.0 {
        t = 1.0;
        s = ((across - to_a) / len1).clamp(0.0, 1.0);
    }
    ((a0 + d1 * s) - (b0 + d2 * t)).length()
}

/// `CrossAtTg` for an attack volume of a radius `radius` swept from `from` to `to`: whether it
/// reaches `tg`, and where on it.
fn sweep_cross(tg: &Shape, from: Vec3, to: Vec3, radius: f32) -> Option<Vec3> {
    let (low, high) = core(tg);
    if segment_gap(low, high, from, to) > core_reach(tg) + radius {
        return None;
    }
    let mid = (from + to) * 0.5;
    let axis = core_point(tg, mid);
    let delta = mid - axis;
    let cross = if contact::is_zero(delta.length()) {
        axis
    } else {
        axis + delta.normalize() * core_reach(tg)
    };
    Some(cross)
}