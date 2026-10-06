use bevy_ecs::prelude::*;
use glam::{Mat4, Vec3};

/// `JntHitType` from `d_jnt_hit.h`.
pub const CYL: u8 = 0;
pub const SPH: u8 = 1;
pub const CYL2: u8 = 2;
pub const CYL_THROW: u8 = 3;
pub const SPH_THROW: u8 = 4;
pub const CYL_UNK: u8 = 5;
pub const SPH_UNK: u8 = 6;
pub const CYL_DELETE: u8 = 7;
pub const SPH_DELETE: u8 = 8;

pub const IDX_DELETE: i32 = -3;
pub const IDX_NONE: i32 = -1;

const FAR: f32 = 10000.0;
const NEAR_ZERO_SQUARE: f32 = 8e-11;

pub fn is_cylinder(shape: u8) -> bool {
    matches!(shape, CYL | CYL2 | CYL_THROW | CYL_UNK | CYL_DELETE)
}

pub fn is_sphere(shape: u8) -> bool {
    matches!(shape, SPH | SPH_THROW | SPH_UNK | SPH_DELETE)
}

pub fn is_throw(shape: u8) -> bool {
    matches!(shape, CYL_THROW | SPH_THROW)
}

pub fn is_delete(shape: u8) -> bool {
    matches!(shape, CYL_DELETE | SPH_DELETE)
}

/// `__jnt_hit_data_c`: one collider bound to a joint, with the shape's endpoints in the
/// joint's local frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JntHitData {
    pub shape: u8,
    pub joint: u16,
    pub radius: f32,
    pub offsets: [Vec3; 2],
}

/// `daFm_c::jntHitCreateHeap`'s `search_data`: the floormaster's fifteen melee colliders, a
/// head capsule (`CYL2`), four arm cylinders and ten finger cylinders.
pub fn floormaster_colliders() -> Vec<JntHitData> {
    let ude = [Vec3::new(0.0, 0.0, 0.0), Vec3::new(60.0, 0.0, 0.0)];
    let yubi = [Vec3::ZERO, Vec3::new(15.0, 0.0, 0.0)];
    let te = [Vec3::new(20.0, 0.0, 0.0), Vec3::new(20.0, -5.0, 0.0)];
    let arm = |joint: u16| JntHitData {
        shape: CYL,
        joint,
        radius: 8.0,
        offsets: ude,
    };
    let finger = |joint: u16| JntHitData {
        shape: CYL,
        joint,
        radius: 4.0,
        offsets: yubi,
    };
    vec![
        JntHitData {
            shape: CYL2,
            joint: JNT_TE,
            radius: 22.0,
            offsets: te,
        },
        arm(JNT_UDEA),
        arm(JNT_UDEB),
        arm(JNT_UDEC),
        arm(JNT_UDED),
        finger(JNT_CYUBIA),
        finger(JNT_CYUBIB),
        finger(JNT_HYUBIA),
        finger(JNT_HYUBIB),
        finger(JNT_KYUBIA),
        finger(JNT_KYUBIB),
        finger(JNT_NYUBIA),
        finger(JNT_NYUBIB),
        finger(JNT_OYUBIA),
        finger(JNT_OYUBIB),
    ]
}

/// `FM_JNT` from `res/Object/Fm.h`.
pub const JNT_CENTER: u16 = 0x0;
pub const JNT_UDEA: u16 = 0x1;
pub const JNT_UDEB: u16 = 0x2;
pub const JNT_UDEC: u16 = 0x3;
pub const JNT_UDED: u16 = 0x4;
pub const JNT_TE: u16 = 0x5;
pub const JNT_CYUBIA: u16 = 0x6;
pub const JNT_CYUBIB: u16 = 0x7;
pub const JNT_HYUBIA: u16 = 0x8;
pub const JNT_HYUBIB: u16 = 0x9;
pub const JNT_KYUBIA: u16 = 0xA;
pub const JNT_KYUBIB: u16 = 0xB;
pub const JNT_NYUBIA: u16 = 0xC;
pub const JNT_NYUBIB: u16 = 0xD;
pub const JNT_OYUBIA: u16 = 0xE;
pub const JNT_OYUBIB: u16 = 0xF;

/// The collider set an actor carries, bound to its model's bind-pose joints.
///
/// `read_joints` in `bdl.rs` resolves each joint's world translation and rejects any joint with
/// a non-identity local rotation or scale, so a bind pose needs no skeletal animation: the
/// joint matrix is a pure translation, and the actor's own transform carries it into the
/// world exactly as `getAnmMtx` does for a posed model.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct JntHit {
    shapes: Vec<u8>,
    joints: Vec<u16>,
    radiuses: Vec<f32>,
    offsets: Vec<Vec3>,
    frames: Vec<Mat4>,
}

impl JntHit {
    /// `JntHit_create` then `CreateInit`, which flattens the colliders into the parallel
    /// arrays the search walks: two offset endpoints per cylinder, one per sphere.
    pub fn new(data: &[JntHitData], bind_pose: &[[f32; 3]]) -> Self {
        let mut offsets = Vec::new();
        for entry in data {
            if is_cylinder(entry.shape) {
                offsets.push(entry.offsets[0]);
                offsets.push(entry.offsets[1]);
            } else if is_sphere(entry.shape) {
                offsets.push(entry.offsets[0]);
            }
        }
        Self {
            shapes: data.iter().map(|entry| entry.shape).collect(),
            joints: data.iter().map(|entry| entry.joint).collect(),
            radiuses: data.iter().map(|entry| entry.radius).collect(),
            offsets,
            frames: bind_pose
                .iter()
                .map(|joint| Mat4::from_translation(Vec3::from_array(*joint)))
                .collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.shapes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }

    pub fn shape(&self, index: usize) -> u8 {
        self.shapes[index]
    }

    pub fn joint(&self, index: usize) -> u16 {
        self.joints[index]
    }

    pub fn radius(&self, index: usize) -> f32 {
        self.radiuses[index]
    }

    /// `mDoMtx_stack_c::copy(model->getAnmMtx(joint))` for the whole model: the bind pose put
    /// into the actor's frame.
    fn frames_in_place(&self, actor: &super::actor::Actor) -> Vec<Mat4> {
        let yaw = f32::from(actor.shape_angle[1]) * (core::f32::consts::PI / 32768.0);
        let (sin, cos) = yaw.sin_cos();
        let base = Mat4::from_cols_array_2d(&[
            [cos, 0.0, -sin, actor.pos.x],
            [0.0, 1.0, 0.0, actor.pos.y],
            [sin, 0.0, cos, actor.pos.z],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        self.frames.iter().map(|frame| base * *frame).collect()
    }

    fn point(frames: &[Mat4], joint: u16, offset: Vec3) -> Vec3 {
        frames
            .get(usize::from(joint))
            .map_or(offset, |frame| {
                (frame * offset.extend(1.0)).truncate()
            })
    }

    /// `csXyz(cos(x)sin(y), -sin(x), cos(x)cos(y))` for the attacker's `shape_angle`.
    fn facing(angle: [i16; 3]) -> Vec3 {
        Vec3::new(
            scos(angle[0]) * ssin(angle[1]),
            -ssin(angle[0]),
            scos(angle[0]) * scos(angle[1]),
        )
    }

    /// `HitBufferUpdate`: the surface point is taken into the hit joint's frame, and only a
    /// hit in front of the one already held replaces it.
    fn buffer_update(
        frames: &[Mat4],
        slot: &mut i32,
        held: &mut Vec3,
        joint: u16,
        facing: Vec3,
        surface: Vec3,
    ) -> bool {
        let candidate = JntHit::point(frames, joint, surface);
        if *slot >= 0 && (candidate - *held).dot(facing) >= 0.0 {
            return false;
        }
        *slot = i32::from(joint);
        *held = candidate;
        true
    }

    /// `searchJntHitPosAngleOffset`. `angle` is the caller's `shape_angle`, which the only
    /// caller (`daArrow_c::procMove`) supplies from the arrow itself.
    pub fn search(&self, angle: [i16; 3], actor: &super::actor::Actor, point: Vec3) -> (i32, Vec3) {
        let frames = self.frames_in_place(actor);
        let facing = JntHit::facing(angle);
        let mut far = FAR;
        let mut nearest_joint = IDX_NONE;
        let mut nearest_shape = CYL;
        let mut nearest_radius = 0.0;
        let mut nearest_a = Vec3::ZERO;
        let mut nearest_b = Vec3::ZERO;
        let mut slot = IDX_NONE;
        let mut held = Vec3::ZERO;
        let mut best = Vec3::ZERO;
        let mut pos = 0;
        for index in 0..self.shapes.len() {
            let shape = self.shapes[index];
            let joint = self.joints[index];
            let radius = self.radiuses[index];
            if is_cylinder(shape) {
                let a = JntHit::point(&frames, joint, self.offsets[pos]);
                let b = JntHit::point(&frames, joint, self.offsets[pos + 1]);
                let gap = point - a;
                let segment = b - a;
                let (touches, distance) = match try_normalize(segment) {
                    Some(unit) => {
                        let across = min_limit(cross_len(unit, gap) - radius, 0.0);
                        let along = unit.dot(gap);
                        let beyond = if along < 0.0 {
                            -along
                        } else if along > segment.length() {
                            along - segment.length()
                        } else {
                            0.0
                        };
                        ((is_zero(across) && is_zero(beyond)),
                            (across * across + beyond * beyond).sqrt())
                    }
                    None => {
                        let value = min_limit(gap.length() - radius, 0.0);
                        (value < radius, value)
                    }
                };
                if distance <= far {
                    far = distance;
                    nearest_joint = i32::from(joint);
                    nearest_shape = shape;
                    nearest_radius = radius;
                    nearest_a = a;
                    nearest_b = b;
                }
                if touches {
                    match shape {
                        CYL => {
                            let hit = cyl_surface(gap, a, b, radius, facing);
                            if is_delete(shape) {
                                return (IDX_DELETE, hit);
                            }
                            if !is_throw(shape)
                                && JntHit::buffer_update(
                                    &frames,
                                    &mut slot,
                                    &mut held,
                                    joint,
                                    facing,
                                    hit,
                                )
                            {
                                best = hit;
                            }
                        }
                        CYL2 => {
                            let hit = cyl2_surface(gap, a, b, radius, facing);
                            if is_delete(shape) {
                                return (IDX_DELETE, hit);
                            }
                            if !is_throw(shape)
                                && JntHit::buffer_update(
                                    &frames,
                                    &mut slot,
                                    &mut held,
                                    joint,
                                    facing,
                                    hit,
                                )
                            {
                                best = hit;
                            }
                        }
                        _ => {}
                    }
                }
                pos += 2;
            } else if is_sphere(shape) {
                let c = JntHit::point(&frames, joint, self.offsets[pos]);
                let distance = min_limit((point - c).length() - radius, 0.0);
                if distance <= far {
                    far = distance;
                    nearest_joint = i32::from(joint);
                    nearest_shape = shape;
                    nearest_radius = radius;
                    nearest_a = c;
                    nearest_b = c;
                }
                if is_zero(distance) {
                    let hit = sph_surface(point - c, c, radius, facing);
                    if is_delete(shape) {
                        return (IDX_DELETE, hit);
                    }
                    if !is_throw(shape)
                        && JntHit::buffer_update(
                            &frames,
                            &mut slot,
                            &mut held,
                            joint,
                            facing,
                            hit,
                        )
                    {
                        best = hit;
                    }
                }
                pos += 1;
            }
        }
        if slot >= 0 {
            return (slot, best);
        }
        if nearest_joint == IDX_NONE || is_throw(nearest_shape) {
            return (IDX_NONE, point);
        }
        if is_delete(nearest_shape) {
            return (IDX_DELETE, point);
        }
        let surface = match nearest_shape {
            CYL => cyl_surface(point - nearest_a, nearest_a, nearest_b, nearest_radius, facing),
            SPH => sph_surface(point - nearest_a, nearest_a, nearest_radius, facing),
            _ => cyl2_surface(point - nearest_a, nearest_a, nearest_b, nearest_radius, facing),
        };
        (nearest_joint, surface)
    }

    /// Whether the joint colliders reach `point`, which is the strike test.
    pub fn contains(&self, angle: [i16; 3], actor: &super::actor::Actor, point: Vec3) -> bool {
        self.search(angle, actor, point).0 >= 0
    }
}

/// `cM_ssin`: sine of a 0x10000-circle fixed-point angle.
pub fn ssin(angle: i16) -> f32 {
    (f32::from(angle) * (core::f32::consts::PI / 32768.0)).sin()
}

/// `cM_scos`.
pub fn scos(angle: i16) -> f32 {
    (f32::from(angle) * (core::f32::consts::PI / 32768.0)).cos()
}

/// `cXyz::isNearZeroSquare`.
fn is_near_zero_square(v: Vec3) -> bool {
    v.length_squared() < NEAR_ZERO_SQUARE
}

/// `cLib_IsZero`.
fn is_zero(value: f32) -> bool {
    value.abs() < NEAR_ZERO_SQUARE
}

/// `cXyz::normalizeRS`.
fn try_normalize(v: Vec3) -> Option<Vec3> {
    if is_near_zero_square(v) {
        None
    } else {
        Some(v / v.length())
    }
}

/// `cXyz::normZP`, which normalises all three components despite the name.
fn norm_zp(v: Vec3) -> Vec3 {
    if is_near_zero_square(v) {
        Vec3::ZERO
    } else {
        v / v.length()
    }
}

/// `cLib_minLimit`.
fn min_limit(value: f32, limit: f32) -> f32 {
    if value < limit { limit } else { value }
}

fn cross_len(a: Vec3, b: Vec3) -> f32 {
    a.cross(b).length()
}

/// `CylHitPosAngleOffset`: the rear hemisphere of the capsule, snapped to the lateral surface
/// on a grazing hit, then clamped onto the segment.
fn cyl_surface(gap: Vec3, a: Vec3, b: Vec3, radius: f32, facing: Vec3) -> Vec3 {
    let segment = b - a;
    let dir = norm_zp(segment);
    let reach = segment.length();
    let perp = gap - dir * gap.dot(dir);
    let radial = facing - dir * facing.dot(dir);
    let mut hit = if let Some(unit) = try_normalize(radial) {
        let along = perp.dot(unit);
        let mut disc = along * along + radius * radius - perp.length_squared();
        if disc < 0.0 {
            disc = 0.0;
        }
        let out = unit * (-along - disc.sqrt()) + gap;
        if disc == 0.0 {
            let axial = dir * out.dot(dir);
            axial + norm_zp(out - axial) * radius
        } else {
            out
        }
    } else {
        norm_zp(perp) * radius + dir * gap.dot(dir)
    };
    let along = hit.dot(dir);
    let excess = if along < 0.0 {
        -along
    } else if along > reach {
        reach - along
    } else {
        0.0
    };
    hit += dir * excess;
    hit
}

/// `SphHitPosAngleOffset`: the sphere's near pole, clamped to the radius.
fn sph_surface(gap: Vec3, centre: Vec3, radius: f32, facing: Vec3) -> Vec3 {
    let along = gap.dot(facing);
    let mut disc = along * along + radius * radius - gap.length_squared();
    if disc < 0.0 {
        disc = 0.0;
    }
    let mut hit = facing * (-along - disc.sqrt()) + gap;
    if hit.length() > radius {
        hit = norm_zp(hit) * radius;
    }
    hit + centre
}

/// `Cyl2HitPosAngleOffset`: the capsule's front cap, then the lateral offset pulled onto the
/// surface.
fn cyl2_surface(gap: Vec3, a: Vec3, b: Vec3, radius: f32, facing: Vec3) -> Vec3 {
    let segment = b - a;
    let dir = norm_zp(segment);
    let reach = segment.length();
    let along = gap.dot(dir);
    let mut hit = gap;
    let slant = facing.dot(dir);
    if !is_zero(slant) {
        let t = if slant > 0.0 {
            -along / slant
        } else {
            (reach - along) / slant
        };
        hit = gap + facing * t;
    }
    let across = hit - dir * hit.dot(dir);
    if across.length() > radius
        && let Some(unit) = try_normalize(across)
    {
        hit = dir * hit.dot(dir) + unit * radius;
    }
    hit + a
}
