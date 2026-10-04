use bevy_ecs::prelude::*;
use glam::{Vec2, Vec3};

use super::player::Player;

const HEADER_LEN: usize = 0x34;
const VTX_STRIDE: usize = 12;
const TRI_STRIDE: usize = 10;
const BLOCK_STRIDE: usize = 2;
const TREE_STRIDE: usize = 0x14;
const GROUP_STRIDE: usize = 0x34;
const INFO_STRIDE: usize = 16;
const GROUND_NY: f32 = 0.5;
const ROOF_NY: f32 = -0.8;
const ACTOR_HEIGHT: f32 = 60.0;
const PLANE_MIN_MAGNITUDE: f32 = 0.02;
const EXIT_MASK: u32 = 0x0007E000;
const EXIT_SHIFT: u32 = 13;
pub const NO_EXIT: u8 = 0x3F;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DzbError {
    Truncated,
    BadTable,
    VertexIndex,
    TooLarge,
}

impl DzbError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Truncated => "dzb header truncated",
            Self::BadTable => "dzb table out of range",
            Self::VertexIndex => "dzb triangle vertex out of range",
            Self::TooLarge => "dzb merged vertex table exceeds u16 indexing",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Plane {
    normal: Vec3,
    d: f32,
}

#[derive(Resource, Debug, Clone)]
pub struct Collision {
    verts: Vec<Vec3>,
    tris: Vec<[u16; 3]>,
    planes: Vec<Plane>,
    exits: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct StageData {
    pub spawn: Vec3,
    pub yaw: f32,
    pub collision: Collision,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct MoveFrom(pub Vec3);

fn u32_at(data: &[u8], at: usize) -> Option<u32> {
    let bytes = data.get(at..at + 4)?;
    Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn f32_at(data: &[u8], at: usize) -> Option<f32> {
    let bytes = data.get(at..at + 4)?;
    Some(f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn u16_at(data: &[u8], at: usize) -> Option<u16> {
    let bytes = data.get(at..at + 2)?;
    Some(u16::from_be_bytes([bytes[0], bytes[1]]))
}

fn check_table(len: usize, count: u32, offset: u32, stride: usize) -> Result<(), DzbError> {
    let end = (count as usize)
        .checked_mul(stride)
        .and_then(|bytes| (offset as usize).checked_add(bytes))
        .ok_or(DzbError::BadTable)?;
    if end > len {
        return Err(DzbError::BadTable);
    }
    Ok(())
}

fn plane(a: Vec3, b: Vec3, c: Vec3) -> Plane {
    let cross = (b - a).cross(c - a);
    let magnitude = cross.length();
    if magnitude >= PLANE_MIN_MAGNITUDE {
        let normal = cross / magnitude;
        Plane {
            normal,
            d: -normal.dot(a),
        }
    } else {
        Plane {
            normal: Vec3::ZERO,
            d: 0.0,
        }
    }
}

fn inside_xz(a: Vec3, b: Vec3, c: Vec3, x: f32, z: f32) -> bool {
    let d1 = (x - b.x) * (a.z - b.z) - (a.x - b.x) * (z - b.z);
    let d2 = (x - c.x) * (b.z - c.z) - (b.x - c.x) * (z - c.z);
    let d3 = (x - a.x) * (c.z - a.z) - (c.x - a.x) * (z - a.z);
    let has_negative = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_positive = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_negative && has_positive)
}

fn crosses(from: Vec3, to: Vec3, p: Vec3, q: Vec3) -> bool {
    let rx = to.x - from.x;
    let rz = to.z - from.z;
    let sx = q.x - p.x;
    let sz = q.z - p.z;
    let denominator = rx * sz - rz * sx;
    if denominator == 0.0 {
        return false;
    }
    let ox = p.x - from.x;
    let oz = p.z - from.z;
    let t = (ox * sz - oz * sx) / denominator;
    if !(t > 0.0 && t <= 1.0) {
        return false;
    }
    let u = (ox * rz - oz * rx) / denominator;
    u >= 0.0 && u <= 1.0
}

impl Collision {
    pub fn parse(data: &[u8]) -> Result<Self, DzbError> {
        if data.len() < HEADER_LEN {
            return Err(DzbError::Truncated);
        }
        let vertex_count = u32_at(data, 0).ok_or(DzbError::Truncated)?;
        let vertex_offset = u32_at(data, 4).ok_or(DzbError::Truncated)?;
        let tri_count = u32_at(data, 8).ok_or(DzbError::Truncated)?;
        let tri_offset = u32_at(data, 0x0C).ok_or(DzbError::Truncated)?;
        check_table(data.len(), vertex_count, vertex_offset, VTX_STRIDE)?;
        check_table(data.len(), tri_count, tri_offset, TRI_STRIDE)?;
        check_table(
            data.len(),
            u32_at(data, 0x10).ok_or(DzbError::Truncated)?,
            u32_at(data, 0x14).ok_or(DzbError::Truncated)?,
            BLOCK_STRIDE,
        )?;
        check_table(
            data.len(),
            u32_at(data, 0x18).ok_or(DzbError::Truncated)?,
            u32_at(data, 0x1C).ok_or(DzbError::Truncated)?,
            TREE_STRIDE,
        )?;
        check_table(
            data.len(),
            u32_at(data, 0x20).ok_or(DzbError::Truncated)?,
            u32_at(data, 0x24).ok_or(DzbError::Truncated)?,
            GROUP_STRIDE,
        )?;
        check_table(
            data.len(),
            u32_at(data, 0x28).ok_or(DzbError::Truncated)?,
            u32_at(data, 0x2C).ok_or(DzbError::Truncated)?,
            INFO_STRIDE,
        )?;

        let mut verts = Vec::with_capacity(vertex_count as usize);
        for index in 0..vertex_count as usize {
            let at = vertex_offset as usize + index * VTX_STRIDE;
            verts.push(Vec3::new(
                f32_at(data, at).ok_or(DzbError::Truncated)?,
                f32_at(data, at + 4).ok_or(DzbError::Truncated)?,
                f32_at(data, at + 8).ok_or(DzbError::Truncated)?,
            ));
        }

        let info_count = u32_at(data, 0x28).ok_or(DzbError::Truncated)?;
        let info_offset = u32_at(data, 0x2C).ok_or(DzbError::Truncated)?;
        let mut info0 = Vec::with_capacity(info_count as usize);
        for index in 0..info_count as usize {
            let at = info_offset as usize + index * INFO_STRIDE;
            info0.push(u32_at(data, at).ok_or(DzbError::Truncated)?);
        }

        let mut tris = Vec::with_capacity(tri_count as usize);
        let mut planes = Vec::with_capacity(tri_count as usize);
        let mut exits = Vec::with_capacity(tri_count as usize);
        for index in 0..tri_count as usize {
            let at = tri_offset as usize + index * TRI_STRIDE;
            let ids = [
                u16_at(data, at).ok_or(DzbError::Truncated)?,
                u16_at(data, at + 2).ok_or(DzbError::Truncated)?,
                u16_at(data, at + 4).ok_or(DzbError::Truncated)?,
            ];
            if ids.iter().any(|&id| usize::from(id) >= verts.len()) {
                return Err(DzbError::VertexIndex);
            }
            let info_id = u16_at(data, at + 6).ok_or(DzbError::Truncated)?;
            exits.push(match info0.get(usize::from(info_id)) {
                Some(info) => ((info & EXIT_MASK) >> EXIT_SHIFT) as u8,
                None => NO_EXIT,
            });
            tris.push(ids);
            planes.push(plane(
                verts[usize::from(ids[0])],
                verts[usize::from(ids[1])],
                verts[usize::from(ids[2])],
            ));
        }

        Ok(Self {
            verts,
            tris,
            planes,
            exits,
        })
    }

    pub fn append(&mut self, other: Self) -> Result<(), DzbError> {
        let total = self
            .verts
            .len()
            .checked_add(other.verts.len())
            .ok_or(DzbError::TooLarge)?;
        if total > usize::from(u16::MAX) + 1 {
            return Err(DzbError::TooLarge);
        }
        let base = self.verts.len() as u32;
        self.tris.extend(other.tris.iter().map(|tri| {
            [
                (tri[0] as u32 + base) as u16,
                (tri[1] as u32 + base) as u16,
                (tri[2] as u32 + base) as u16,
            ]
        }));
        self.planes.extend(other.planes);
        self.exits.extend(other.exits);
        self.verts.extend(other.verts);
        Ok(())
    }

    pub fn bounds(&self) -> (Vec2, Vec2) {
        if self.verts.is_empty() {
            return (Vec2::ZERO, Vec2::ZERO);
        }
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        for vert in &self.verts {
            let xz = vert.truncate();
            min = min.min(xz);
            max = max.max(xz);
        }
        (min, max)
    }

    fn ground_hit(&self, x: f32, z: f32) -> Option<(f32, usize)> {
        let mut highest: Option<(f32, usize)> = None;
        for (index, tri) in self.tris.iter().enumerate() {
            let surface = self.planes[index];
            if surface.normal.y < GROUND_NY {
                continue;
            }
            let a = self.verts[usize::from(tri[0])];
            let b = self.verts[usize::from(tri[1])];
            let c = self.verts[usize::from(tri[2])];
            if !inside_xz(a, b, c, x, z) {
                continue;
            }
            let y = -(surface.d + surface.normal.x * x + surface.normal.z * z)
                / surface.normal.y;
            let replace = match highest {
                Some((current, _)) => y > current,
                None => true,
            };
            if replace {
                highest = Some((y, index));
            }
        }
        highest
    }

    pub fn ground_at(&self, x: f32, z: f32) -> Option<f32> {
        self.ground_hit(x, z).map(|(y, _)| y)
    }

    pub fn ground_normal(&self, x: f32, z: f32) -> Option<Vec3> {
        self.ground_hit(x, z).map(|(_, index)| self.planes[index].normal)
    }

    pub fn exit_at(&self, x: f32, z: f32) -> Option<u8> {
        let (_, index) = self.ground_hit(x, z)?;
        self.exits.get(index).copied()
    }

    fn wall(surface: Plane) -> bool {
        surface.normal != Vec3::ZERO
            && surface.normal.y < GROUND_NY
            && surface.normal.y >= ROOF_NY
    }

    fn blocked(&self, from: Vec3, to: Vec3) -> bool {
        if from.x == to.x && from.z == to.z {
            return false;
        }
        let feet = to.y;
        let head = feet + ACTOR_HEIGHT;
        for (index, tri) in self.tris.iter().enumerate() {
            if !Self::wall(self.planes[index]) {
                continue;
            }
            let a = self.verts[usize::from(tri[0])];
            let b = self.verts[usize::from(tri[1])];
            let c = self.verts[usize::from(tri[2])];
            let lowest = a.y.min(b.y).min(c.y);
            let highest = a.y.max(b.y).max(c.y);
            if highest < feet || lowest > head {
                continue;
            }
            if inside_xz(a, b, c, from.x, from.z) {
                continue;
            }
            if inside_xz(a, b, c, to.x, to.z) {
                return true;
            }
            if crosses(from, to, a, b) || crosses(from, to, b, c) || crosses(from, to, c, a) {
                return true;
            }
        }
        false
    }

    pub fn slide(&self, from: Vec3, to: Vec3) -> Vec3 {
        if !self.blocked(from, to) {
            return to;
        }
        if !self.blocked(from, Vec3::new(to.x, to.y, from.z)) {
            return Vec3::new(to.x, to.y, from.z);
        }
        if !self.blocked(from, Vec3::new(from.x, to.y, to.z)) {
            return Vec3::new(from.x, to.y, to.z);
        }
        Vec3::new(from.x, to.y, from.z)
    }
}

pub fn remember_move_from(mut source: ResMut<MoveFrom>, players: Query<&Player>) {
    if let Some(player) = players.iter().next() {
        source.0 = player.pos;
    }
}
