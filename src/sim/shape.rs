use bevy_ecs::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ShapeKind {
    #[default]
    Sphere,
    Cylinder,
    Capsule,
    Box,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Shape {
    pub kind: ShapeKind,
    pub pos: glam::Vec3,
    pub radius: f32,
    pub height: f32,
    pub yaw: f32,
    pub scale: glam::Vec3,
}

impl Shape {
    pub fn sphere(pos: glam::Vec3, radius: f32) -> Self {
        Self {
            kind: ShapeKind::Sphere,
            pos,
            radius,
            ..Self::default()
        }
    }

    pub fn cylinder(pos: glam::Vec3, radius: f32, height: f32) -> Self {
        Self {
            kind: ShapeKind::Cylinder,
            pos,
            radius,
            height,
            ..Self::default()
        }
    }

    pub fn capsule(pos: glam::Vec3, radius: f32, height: f32) -> Self {
        Self {
            kind: ShapeKind::Capsule,
            pos,
            radius,
            height,
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
        }
    }

    pub fn intersects(&self, other: &Shape) -> bool {
        match (self.kind, other.kind) {
            (ShapeKind::Sphere, ShapeKind::Sphere) => {
                let reach = self.bound() + other.bound();
                (self.pos - other.pos).length() <= reach
            }
            (ShapeKind::Sphere, _) => other.contains(self.pos),
            (_, ShapeKind::Sphere) => self.contains(other.pos),
            _ => {
                let reach = self.bound() + other.bound();
                if (self.pos - other.pos).length() > reach {
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
        }
    }
}

fn sample_point(shape: &Shape, fx: f32, fy: f32, fz: f32) -> glam::Vec3 {
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