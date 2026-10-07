use bevy_ecs::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ShapeKind {
    #[default]
    Sphere,
    Cylinder,
    Capsule,
    Box,
    Triangle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shape {
    pub kind: ShapeKind,
    pub pos: glam::Vec3,
    pub radius: f32,
    pub height: f32,
    pub yaw: f32,
    pub scale: glam::Vec3,
    pub verts: [glam::Vec3; 3],
}

impl Default for Shape {
    fn default() -> Self {
        Self {
            kind: ShapeKind::Sphere,
            pos: glam::Vec3::ZERO,
            radius: 0.0,
            height: 0.0,
            yaw: 0.0,
            scale: glam::Vec3::ONE,
            verts: [glam::Vec3::ZERO; 3],
        }
    }
}

impl Shape {
    pub fn sphere(pos: glam::Vec3, radius: f32) -> Self {
        Self {
            kind: ShapeKind::Sphere,
            pos,
            radius,
            scale: glam::Vec3::ONE,
            ..Self::default()
        }
    }

    pub fn cylinder(pos: glam::Vec3, radius: f32, height: f32) -> Self {
        Self {
            kind: ShapeKind::Cylinder,
            pos,
            radius,
            height,
            scale: glam::Vec3::ONE,
            ..Self::default()
        }
    }

    pub fn capsule(pos: glam::Vec3, radius: f32, height: f32) -> Self {
        Self {
            kind: ShapeKind::Capsule,
            pos,
            radius,
            height,
            scale: glam::Vec3::ONE,
            ..Self::default()
        }
    }

    pub fn aabb(pos: glam::Vec3, size: glam::Vec3, yaw: f32) -> Self {
        Self {
            kind: ShapeKind::Box,
            pos,
            radius: 1.0,
            scale: size,
            yaw,
            ..Self::default()
        }
    }

    pub fn triangle(a: glam::Vec3, b: glam::Vec3, c: glam::Vec3) -> Self {
        Self {
            kind: ShapeKind::Triangle,
            pos: (a + b + c) * (1.0 / 3.0),
            verts: [a, b, c],
            ..Self::default()
        }
    }

    pub fn is_triangle(&self) -> bool {
        self.kind == ShapeKind::Triangle
    }

    pub fn centroid(&self) -> glam::Vec3 {
        match self.kind {
            ShapeKind::Triangle => (self.verts[0] + self.verts[1] + self.verts[2]) * (1.0 / 3.0),
            ShapeKind::Sphere
            | ShapeKind::Cylinder
            | ShapeKind::Capsule
            | ShapeKind::Box => self.pos,
        }
    }

    pub fn set_verts(&mut self, a: glam::Vec3, b: glam::Vec3, c: glam::Vec3) {
        self.verts = [a, b, c];
        self.pos = (a + b + c) * (1.0 / 3.0);
    }

    pub fn contains(&self, point: glam::Vec3) -> bool {
        match self.kind {
            ShapeKind::Sphere => {
                (point - self.pos).length() <= self.radius * self.scale.x
            }
            ShapeKind::Cylinder => {
                let dx = point.x - self.pos.x;
                let dz = point.z - self.pos.z;
                let radius = self.radius * self.scale.x;
                if dx * dx + dz * dz > radius * radius {
                    return false;
                }
                let half = self.height * 0.5 * self.scale.y;
                point.y >= self.pos.y - half && point.y <= self.pos.y + half
            }
            ShapeKind::Capsule => {
                let half = (self.height * 0.5).max(0.0);
                let base = glam::Vec3::new(self.pos.x, self.pos.y - half, self.pos.z);
                let tip = glam::Vec3::new(self.pos.x, self.pos.y + half, self.pos.z);
                let radius = self.radius * self.scale.x;
                distance_to_segment(point, base, tip) <= radius
            }
            ShapeKind::Box => {
                let local = rotate_y(point - self.pos, -self.yaw);
                let half = self.scale * 0.5;
                local.x.abs() <= half.x && local.y.abs() <= half.y && local.z.abs() <= half.z
            }
            ShapeKind::Triangle => tri_contains(self.verts, point),
        }
    }

    pub fn intersects(&self, other: &Shape) -> bool {
        match (self.kind, other.kind) {
            (ShapeKind::Sphere, ShapeKind::Sphere) => {
                let reach = self.bound() + other.bound();
                (self.pos - other.pos).length() <= reach
            }
            (ShapeKind::Sphere, ShapeKind::Triangle) => {
                tri_sph_cross(other.verts, self.pos, self.radius * self.scale.x)
            }
            (ShapeKind::Triangle, ShapeKind::Sphere) => {
                tri_sph_cross(self.verts, other.pos, other.radius * other.scale.x)
            }
            (ShapeKind::Triangle, ShapeKind::Triangle) => tri_tri_cross(self.verts, other.verts),
            (ShapeKind::Cylinder | ShapeKind::Capsule, ShapeKind::Triangle) => {
                tri_cyl_cross(self, other.verts)
            }
            (ShapeKind::Triangle, ShapeKind::Cylinder | ShapeKind::Capsule) => {
                tri_cyl_cross(other, self.verts)
            }
            (ShapeKind::Sphere, _) => other.contains(self.pos),
            (_, ShapeKind::Sphere) => self.contains(other.pos),
            _ => {
                let reach = self.bound() + other.bound();
                if (self.centroid() - other.centroid()).length() > reach {
                    return false;
                }
                let steps = 4;
                for i in 0..=steps {
                    for j in 0..=steps {
                        for k in 0..=steps {
                            let fx = i as f32 / steps as f32;
                            let fy = j as f32 / steps as f32;
                            let fz = k as f32 / steps as f32;
                            let sample = sample_point(self, fx, fy, fz);
                            if other.contains(sample) || self.contains(sample) {
                                return true;
                            }
                        }
                    }
                }
                false
            }
        }
    }

    fn bound(&self) -> f32 {
        match self.kind {
            ShapeKind::Sphere => self.radius * self.scale.x,
            ShapeKind::Cylinder | ShapeKind::Capsule => {
                (self.height * 0.5 * self.scale.y).max(self.radius * self.scale.x)
            }
            ShapeKind::Box => self.scale.length() * 0.5,
            ShapeKind::Triangle => self.verts.iter().fold(0.0f32, |far, v| {
                far.max((*v - self.centroid()).length())
            }),
        }
    }
}

fn sample_point(shape: &Shape, fx: f32, fy: f32, fz: f32) -> glam::Vec3 {
    if shape.kind == ShapeKind::Triangle {
        let root = fx.sqrt();
        let u = 1.0 - root;
        let v = root * (1.0 - fy);
        let w = root * fy;
        return shape.verts[0] * u + shape.verts[1] * v + shape.verts[2] * w;
    }
    let local = match shape.kind {
        ShapeKind::Box => {
            let half = shape.scale * 0.5;
            glam::Vec3::new(
                half.x + fx * half.x * 2.0 - half.x,
                half.y + fy * half.y * 2.0 - half.y,
                half.z + fz * half.z * 2.0 - half.z,
            )
        }
        _ => {
            let radius = shape.radius * shape.scale.x;
            let theta = fx * std::f32::consts::TAU;
            let phi = fy * std::f32::consts::PI;
            let r = radius * fz;
            glam::Vec3::new(
                r * phi.sin() * theta.cos(),
                shape.height * 0.5 * shape.scale.y * (fy * 2.0 - 1.0),
                r * phi.sin() * theta.sin(),
            )
        }
    };
    rotate_y(local, shape.yaw) + shape.pos
}

fn rotate_y(v: glam::Vec3, yaw: f32) -> glam::Vec3 {
    if yaw == 0.0 {
        return v;
    }
    let (sin_y, cos_y) = yaw.sin_cos();
    glam::Vec3::new(v.x * cos_y + v.z * sin_y, v.y, -v.x * sin_y + v.z * cos_y)
}

fn distance_to_segment(point: glam::Vec3, a: glam::Vec3, b: glam::Vec3) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_squared();
    if len_sq <= f32::EPSILON {
        return (point - a).length();
    }
    let t = ((point - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    (point - (a + ab * t)).length()
}

fn tri_cyl_cross(cyl: &Shape, tri: [glam::Vec3; 3]) -> bool {
    let radius = cyl.radius * cyl.scale.x;
    let half = (cyl.height * 0.5 * cyl.scale.y).max(0.0);
    let low = glam::Vec3::new(cyl.pos.x, cyl.pos.y - half, cyl.pos.z);
    let high = glam::Vec3::new(cyl.pos.x, cyl.pos.y + half, cyl.pos.z);
    if tri_sph_cross(tri, low, radius) || tri_sph_cross(tri, high, radius) {
        return true;
    }
    let Some(mid) = tri_line_cross(low, high, tri) else {
        return false;
    };
    if tri_contains(tri, mid) {
        return true;
    }
    tri_edge_gap(tri, low, high, radius)
}

fn tri_edge_gap(tri: [glam::Vec3; 3], low: glam::Vec3, high: glam::Vec3, radius: f32) -> bool {
    segment_gap(low, high, tri[0], tri[1]) < radius
        || segment_gap(low, high, tri[1], tri[2]) < radius
        || segment_gap(low, high, tri[2], tri[0]) < radius
}

fn segment_gap(a0: glam::Vec3, a1: glam::Vec3, b0: glam::Vec3, b1: glam::Vec3) -> f32 {
    let (s, t) = segment_params(a0, a1, b0, b1);
    ((a0 + (a1 - a0) * s) - (b0 + (b1 - b0) * t)).length()
}

fn segment_params(
    a0: glam::Vec3,
    a1: glam::Vec3,
    b0: glam::Vec3,
    b1: glam::Vec3,
) -> (f32, f32) {
    let da = a1 - a0;
    let db = b1 - b0;
    let r = a0 - b0;
    let len_a = da.length_squared();
    let len_b = db.length_squared();
    let to_a = da.dot(r);
    let to_b = db.dot(r);
    if len_a <= f32::EPSILON && len_b <= f32::EPSILON {
        return (0.0, 0.0);
    }
    if len_a <= f32::EPSILON {
        return (0.0, (to_b / len_b).clamp(0.0, 1.0));
    }
    if len_b <= f32::EPSILON {
        return ((-to_a / len_a).clamp(0.0, 1.0), 0.0);
    }
    let across = da.dot(db);
    let denom = len_a * len_b - across * across;
    let free = if denom > f32::EPSILON {
        ((across * to_b - to_a * len_b) / denom).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let t = (across * free + to_b) / len_b;
    if t < 0.0 {
        ((-to_a / len_a).clamp(0.0, 1.0), 0.0)
    } else if t > 1.0 {
        (((across - to_a) / len_a).clamp(0.0, 1.0), 1.0)
    } else {
        (free, t)
    }
}

pub fn tri_tri_cross(one: [glam::Vec3; 3], two: [glam::Vec3; 3]) -> bool {
    let (normal, d) = tri_plane(one);
    let a = plane_func(normal, d, two[0]);
    let b = plane_func(normal, d, two[1]);
    let c = plane_func(normal, d, two[2]);
    if (a > 0.0 && b > 0.0 && c > 0.0) || (a < 0.0 && b < 0.0 && c < 0.0) {
        return false;
    }
    let (normal, d) = tri_plane(two);
    let a = plane_func(normal, d, one[0]);
    let b = plane_func(normal, d, one[1]);
    let c = plane_func(normal, d, one[2]);
    if (a > 0.0 && b > 0.0 && c > 0.0) || (a < 0.0 && b < 0.0 && c < 0.0) {
        return false;
    }
    true
}

pub fn tri_sph_cross_point(
    tri: [glam::Vec3; 3],
    centre: glam::Vec3,
    radius: f32,
) -> glam::Vec3 {
    let mid = (tri[0] + tri[1]) * 0.5;
    let sq = (mid - centre).length_squared();
    if sq.abs() < TRI_ABS_MIN {
        centre
    } else {
        centre + (mid - centre) * (radius / sq)
    }
}

pub fn tri_sph_cross(tri: [glam::Vec3; 3], centre: glam::Vec3, radius: f32) -> bool {
    if !tri_box_contains(tri, centre, radius) {
        return false;
    }
    let (normal, d) = tri_plane(tri);
    let len = normal.length();
    let dist = if len.abs() < TRI_ABS_MIN {
        0.0
    } else {
        plane_func(normal, d, centre) / len
    };
    if dist.abs() > radius {
        return false;
    }
    let projected = centre - normal * dist;
    let facing = if normal.y.abs() > 0.5 {
        tri_project_y(tri, projected)
    } else if normal.x.abs() > 0.5 {
        tri_project_x(tri, projected)
    } else {
        tri_project_z(tri, projected)
    };
    facing
        || distance_to_segment(centre, tri[0], tri[1]) <= radius
        || distance_to_segment(centre, tri[1], tri[2]) <= radius
        || distance_to_segment(centre, tri[2], tri[0]) <= radius
}

pub fn tri_line_cross(start: glam::Vec3, end: glam::Vec3, tri: [glam::Vec3; 3]) -> Option<glam::Vec3> {
    let (normal, d) = tri_plane(tri);
    let a = plane_func(normal, d, start);
    let b = plane_func(normal, d, end);
    if a * b > 0.0 {
        return None;
    }
    if (a - b).abs() < TRI_ABS_MIN {
        return None;
    }
    let dst = start + (end - start) * (a / (a - b));
    let inside = (normal.x.abs() < TRI_COMPONENT_MIN || tri_project_x(tri, dst))
        && (normal.y.abs() < TRI_COMPONENT_MIN || tri_project_y(tri, dst))
        && (normal.z.abs() < TRI_COMPONENT_MIN || tri_project_z(tri, dst));
    if inside { Some(dst) } else { None }
}

pub fn tri_contains(tri: [glam::Vec3; 3], point: glam::Vec3) -> bool {
    let (normal, _) = tri_plane(tri);
    (normal.x.abs() < TRI_ABS_MIN || tri_project_x(tri, point))
        && (normal.y.abs() < TRI_ABS_MIN || tri_project_y(tri, point))
        && (normal.z.abs() < TRI_ABS_MIN || tri_project_z(tri, point))
}

pub fn tri_plane(tri: [glam::Vec3; 3]) -> (glam::Vec3, f32) {
    let normal = (tri[1] - tri[0]).cross(tri[2] - tri[0]);
    let mag = normal.length();
    if mag.abs() >= TRI_NORMAL_MIN {
        let unit = normal / mag;
        (unit, -unit.dot(tri[0]))
    } else {
        (glam::Vec3::ZERO, 0.0)
    }
}

fn plane_func(normal: glam::Vec3, d: f32, point: glam::Vec3) -> f32 {
    d + normal.dot(point)
}

fn tri_project_x(tri: [glam::Vec3; 3], point: glam::Vec3) -> bool {
    if !inclusion_2d(
        tri[0].y,
        tri[0].z,
        tri[1].y,
        tri[1].z,
        tri[2].y,
        tri[2].z,
        point.y,
        point.z,
    ) {
        return false;
    }
    edge_agree(
        [
            (tri[0].y, tri[0].z),
            (tri[1].y, tri[1].z),
            (tri[2].y, tri[2].z),
        ],
        (point.y, point.z),
    )
}

fn tri_project_y(tri: [glam::Vec3; 3], point: glam::Vec3) -> bool {
    if !inclusion_2d(
        tri[0].z,
        tri[0].x,
        tri[1].z,
        tri[1].x,
        tri[2].z,
        tri[2].x,
        point.z,
        point.x,
    ) {
        return false;
    }
    edge_agree(
        [
            (tri[0].z, tri[0].x),
            (tri[1].z, tri[1].x),
            (tri[2].z, tri[2].x),
        ],
        (point.z, point.x),
    )
}

fn tri_project_z(tri: [glam::Vec3; 3], point: glam::Vec3) -> bool {
    if !inclusion_2d(
        tri[0].x,
        tri[0].y,
        tri[1].x,
        tri[1].y,
        tri[2].x,
        tri[2].y,
        point.x,
        point.y,
    ) {
        return false;
    }
    edge_agree(
        [
            (tri[0].x, tri[0].y),
            (tri[1].x, tri[1].y),
            (tri[2].x, tri[2].y),
        ],
        (point.x, point.y),
    )
}

fn edge_agree(corners: [(f32, f32); 3], point: (f32, f32)) -> bool {
    let ab = vector_product_2d(corners[0], corners[1], point);
    let bc = vector_product_2d(corners[1], corners[2], point);
    let ca = vector_product_2d(corners[2], corners[0], point);
    if ab <= TRI_EDGE_SLACK && bc <= TRI_EDGE_SLACK && ca <= TRI_EDGE_SLACK {
        return true;
    }
    ab >= -TRI_EDGE_SLACK && bc >= -TRI_EDGE_SLACK && ca >= -TRI_EDGE_SLACK
}

fn vector_product_2d(one: (f32, f32), two: (f32, f32), three: (f32, f32)) -> f32 {
    (two.0 - one.0) * (three.1 - one.1) - (two.1 - one.1) * (three.0 - one.0)
}

pub fn inclusion_2d(
    a0: f32,
    a1: f32,
    b0: f32,
    b1: f32,
    c0: f32,
    c1: f32,
    q0: f32,
    q1: f32,
) -> bool {
    let (mut lo, mut hi) = if a0 < b0 { (a0, b0) } else { (b0, a0) };
    if lo > c0 {
        lo = c0;
    } else if hi < c0 {
        hi = c0;
    }
    if lo > q0 || hi < q0 {
        return false;
    }
    let (lo, hi) = if a1 < b1 { (a1, b1) } else { (b1, a1) };
    let (lo, hi) = if lo > c1 {
        (c1, lo)
    } else if hi < c1 {
        (c1, hi)
    } else {
        (lo, hi)
    };
    lo > q1 || hi < q1
}

fn tri_box_contains(tri: [glam::Vec3; 3], point: glam::Vec3, pad: f32) -> bool {
    let (lo, hi) = corner_span(tri, |v| v.x);
    if lo > point.x + pad || hi + pad < point.x {
        return false;
    }
    let (lo, hi) = corner_span(tri, |v| v.z);
    if lo > point.z + pad || hi + pad < point.z {
        return false;
    }
    let (lo, hi) = corner_span(tri, |v| v.y);
    lo > point.y + pad || hi + pad < point.y
}

fn corner_span(tri: [glam::Vec3; 3], axis: fn(glam::Vec3) -> f32) -> (f32, f32) {
    let a = axis(tri[0]);
    let b = axis(tri[1]);
    let c = axis(tri[2]);
    let (lo, hi) = if a < b { (a, b) } else { (b, a) };
    if lo > c {
        (c, hi)
    } else if hi < c {
        (lo, c)
    } else {
        (lo, hi)
    }
}

pub const TRI_ABS_MIN: f32 = 3.8146973e-06;
pub const TRI_NORMAL_MIN: f32 = 0.02;
pub const TRI_COMPONENT_MIN: f32 = 0.008;
pub const TRI_EDGE_SLACK: f32 = 20.0;

#[derive(Clone, Debug, Default)]
pub struct Body {
    pub shape: Shape,
    pub hit_apids: Vec<usize>,
    pub s_prm: u32,
    pub r_prm: u32,
    pub se: u8,
    pub hit_mark: u8,
    pub eff_counter: u8,
}

impl Body {
    pub fn new(shape: Shape) -> Self {
        Self {
            shape,
            ..Self::default()
        }
    }

    pub fn mask_s_prm(&self, mask: u32) -> u32 {
        self.s_prm & mask
    }

    pub fn mask_r_prm(&self, mask: u32) -> u32 {
        self.r_prm & mask
    }

    pub fn chk_s_prm(&self, mask: u32) -> bool {
        self.mask_s_prm(mask) != 0
    }

    pub fn chk_r_prm(&self, flag: u32) -> bool {
        self.mask_r_prm(flag) != 0
    }

    pub fn on_s_prm(&mut self, flag: u32) {
        self.s_prm |= flag;
    }

    pub fn off_s_prm(&mut self, flag: u32) {
        self.s_prm &= !flag;
    }

    pub fn on_r_prm(&mut self, flag: u32) {
        self.r_prm |= flag;
    }

    pub fn off_r_prm(&mut self, flag: u32) {
        self.r_prm &= !flag;
    }

    pub fn set_eff_counter_timer(&mut self) {
        self.eff_counter = 60;
    }

    pub fn subtract_eff_counter(&mut self) {
        self.eff_counter = self.eff_counter.saturating_sub(1);
    }

    pub fn chk_eff_counter(&self) -> bool {
        self.eff_counter > 0
    }

    pub fn clear_eff_counter(&mut self) {
        self.eff_counter = 0;
    }

    pub fn clr_actor_info(&mut self) {
        self.hit_apids.clear();
    }
}

#[derive(Resource, Debug, Default)]
pub struct Shapes {
    pub bodies: Vec<Body>,
}

impl Shapes {
    pub fn clear(&mut self) {
        self.bodies.clear();
    }

    pub fn add(&mut self, shape: Shape) -> usize {
        self.bodies.push(Body::new(shape));
        self.bodies.len() - 1
    }

    pub fn add_body(&mut self, body: Body) -> usize {
        self.bodies.push(body);
        self.bodies.len() - 1
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.bodies.len() {
            self.bodies.swap_remove(index);
        }
    }

    pub fn body(&self, index: usize) -> Option<&Body> {
        self.bodies.get(index)
    }

    pub fn body_mut(&mut self, index: usize) -> Option<&mut Body> {
        self.bodies.get_mut(index)
    }

    pub fn move_body(&mut self, index: usize, delta: glam::Vec3) {
        if let Some(body) = self.bodies.get_mut(index) {
            body.shape.pos += delta;
        }
    }

    pub fn set_vec(&mut self, index: usize, at: glam::Vec3) {
        if let Some(body) = self.bodies.get_mut(index) {
            body.shape.pos = at;
        }
    }

    pub fn check_at(&mut self, probe: usize) -> Vec<usize> {
        let mut hits = Vec::new();
        let Some(shape) = self.bodies.get(probe).map(|body| body.shape) else {
            return hits;
        };
        for (index, body) in self.bodies.iter().enumerate() {
            if index == probe {
                continue;
            }
            if shape.intersects(&body.shape) {
                hits.push(index);
            }
        }
        if let Some(body) = self.bodies.get_mut(probe) {
            body.hit_apids.clear();
            body.hit_apids.extend(hits.iter().copied());
        }
        hits
    }

    pub fn check_tg(&self, probe: usize) -> Vec<usize> {
        let mut hits = Vec::new();
        let Some(shape) = self.bodies.get(probe).map(|body| body.shape) else {
            return hits;
        };
        for (index, body) in self.bodies.iter().enumerate() {
            if index == probe {
                continue;
            }
            if shape.intersects(&body.shape) {
                hits.push(index);
            }
        }
        hits
    }

    pub fn clear_no_hit(&mut self, probe: usize) {
        if let Some(body) = self.bodies.get_mut(probe) {
            body.clr_actor_info();
        }
    }
}

pub fn control(mut shapes: ResMut<Shapes>) {
    for body in &mut shapes.bodies {
        if body.eff_counter > 0 {
            body.subtract_eff_counter();
        }
    }
}

pub fn init(world: &mut World) {
    world.init_resource::<Shapes>();
}