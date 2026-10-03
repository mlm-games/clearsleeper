use glam::{Vec2, Vec3};
use repame_view3d::{BatchDesc, Frame3d, MeshGroup, OrbitCamera, Rgb, SceneFilter, SceneUpload};

use crate::bdl::{LAYER_SIZE, RoomGroup, RoomMesh};

pub const SKY: [f32; 4] = [0.44, 0.70, 0.94, 1.0];
const GROUND: Rgb = [0.24, 0.42, 0.22];
const WALL: Rgb = [0.48, 0.20, 0.20];
const BODY: Rgb = [0.87, 0.85, 0.78];
const NOSE: Rgb = [0.82, 0.55, 0.20];

const WALL_HEIGHT: f32 = 30.0;
const WALL_THICKNESS: f32 = 8.0;
const PLAYER_HALF: f32 = 8.0;
const PLAYER_HEIGHT: f32 = 32.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorKind {
    Enemy,
    Pickup,
    Talker,
    Ship,
    Bomb,
    Arrow,
    Boomerang,
    Spear,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActorView {
    pub pos: Vec3,
    pub kind: ActorKind,
    pub item: u8,
    pub height: f32,
    pub half: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneSnapshot {
    pub player_pos: Vec3,
    pub player_yaw: f32,
    pub player_face: f32,
    pub bounds_min: Vec2,
    pub bounds_max: Vec2,
    pub ground_y: f32,
    pub walls: bool,
    pub tick: u64,
    pub sky: [f32; 4],
    pub sea: Option<SeaView>,
    pub actors: Vec<ActorView>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeaView {
    pub origin: Vec2,
    pub side: usize,
    pub heights: Vec<f32>,
    pub color: [f32; 3],
}

pub fn sim_yaw_for_camera(cam_yaw: f32) -> f32 {
    (-cam_yaw.cos()).atan2(-cam_yaw.sin())
}

pub fn batch_desc(room: Option<&RoomMesh>) -> BatchDesc {
    match room {
        Some(room) => BatchDesc {
            layer_size: LAYER_SIZE,
            layers: room.textures.len().max(1) as u32,
            filter: SceneFilter::Linear,
        },
        None => BatchDesc::default(),
    }
}

pub fn build_frame(snapshot: &SceneSnapshot, cam: OrbitCamera, room: Option<&RoomMesh>) -> Frame3d {
    let cam = OrbitCamera {
        target: Vec3::new(
            snapshot.player_pos.x,
            snapshot.ground_y,
            snapshot.player_pos.z,
        ),
        ..cam
    };
    let mut frame = Frame3d {
        cam,
        background: Some(snapshot.sky),
        desc: batch_desc(room),
        ..Frame3d::default()
    };
    let eye = frame.cam.eye();
    match room {
        Some(room) => {
            for group in &room.groups {
                frame.push(room_group(room, group, snapshot.tick));
            }
            frame
                .uploads
                .extend(
                    room.textures
                        .iter()
                        .enumerate()
                        .map(|(page, texture)| SceneUpload {
                            page: page as u32,
                            x: 0,
                            y: 0,
                            w: texture.width,
                            h: texture.height,
                            rgba: texture.rgba.clone(),
                        }),
                );
        }
        None => frame.push(ground_group(snapshot)),
    }
    if let Some(sea) = &snapshot.sea {
        frame.push(sea_group(sea, eye));
    }
    if snapshot.walls {
        frame.push(walls_group(snapshot, eye));
    }
    frame.push(player_group(snapshot, eye));
    if !snapshot.actors.is_empty() {
        frame.push(actors_group(snapshot, eye));
    }
    frame
}

const ENEMY: Rgb = [0.55, 0.26, 0.16];
const TALKER: Rgb = [0.32, 0.42, 0.82];
const SHIP: Rgb = [0.58, 0.44, 0.24];
const BOMB: Rgb = [0.18, 0.18, 0.22];
const ARROW: Rgb = [0.86, 0.82, 0.66];
const BOOM: Rgb = [0.72, 0.30, 0.52];
const HEART: Rgb = [0.86, 0.20, 0.22];
const RUPEE: Rgb = [0.24, 0.72, 0.32];
const RELIC: Rgb = [0.90, 0.52, 0.86];
const PIECE_SIZES: [f32; 2] = [14.0, 20.0];

fn pickup_look(item: u8) -> (Rgb, f32) {
    match item {
        0 => (HEART, 10.0),
        0x01..=0x06 | 0x0F => (RUPEE, 10.0),
        0x07 => (RELIC, PIECE_SIZES[0]),
        0x08 => (RELIC, PIECE_SIZES[1]),
        _ => (RELIC, 10.0),
    }
}

fn actor_look(actor: &ActorView) -> Rgb {
    match actor.kind {
        ActorKind::Enemy => ENEMY,
        ActorKind::Talker => TALKER,
        ActorKind::Ship => SHIP,
        ActorKind::Bomb => BOMB,
        ActorKind::Arrow => ARROW,
        ActorKind::Boomerang => BOOM,
        ActorKind::Spear => ARROW,
        ActorKind::Pickup => pickup_look(actor.item).0,
    }
}

fn actors_group(snapshot: &SceneSnapshot, eye: Vec3) -> MeshGroup {
    let mut group = MeshGroup::default();
    for actor in &snapshot.actors {
        group.push_box(
            actor.pos.x,
            actor.pos.y,
            actor.pos.z,
            actor.half,
            actor.height,
            actor.half,
            actor_look(actor),
            eye,
        );
    }
    group.depth_test = true;
    group
}

fn room_group(room: &RoomMesh, group: &RoomGroup, tick: u64) -> MeshGroup {
    MeshGroup {
        positions: group.positions.clone(),
        colors: group.colors.clone(),
        normals: if group.lit {
            group.normals.clone()
        } else {
            Vec::new()
        },
        uvs: animated_uvs(room, group, tick),
        indices: group.indices.clone(),
        texture_page: group.texture_page.unwrap_or(0),
        depth_test: true,
        ..MeshGroup::default()
    }
}

fn sea_group(sea: &SeaView, eye: Vec3) -> MeshGroup {
    use crate::sim::sea::GRID_SIZE;
    let reach = repame_view3d::FAR + GRID_SIZE;
    let reach = reach * reach;
    let mut group = MeshGroup::default();
    for z in 0..sea.side.saturating_sub(1) {
        for x in 0..sea.side.saturating_sub(1) {
            let x0 = sea.origin.x + x as f32 * GRID_SIZE;
            let z0 = sea.origin.y + z as f32 * GRID_SIZE;
            let x1 = x0 + GRID_SIZE;
            let z1 = z0 + GRID_SIZE;
            let dx = (x0 + x1) * 0.5 - eye.x;
            let dz = (z0 + z1) * 0.5 - eye.z;
            if dx * dx + dz * dz > reach {
                continue;
            }
            let corners = [
                [x0, sea.heights[(z + 1) * sea.side + x], z1],
                [x1, sea.heights[(z + 1) * sea.side + x + 1], z1],
                [x1, sea.heights[z * sea.side + x + 1], z0],
                [x0, sea.heights[z * sea.side + x], z0],
            ];
            let first = [
                corners[1][0] - corners[0][0],
                corners[1][1] - corners[0][1],
                corners[1][2] - corners[0][2],
            ];
            let second = [
                corners[3][0] - corners[0][0],
                corners[3][1] - corners[0][1],
                corners[3][2] - corners[0][2],
            ];
            let normal = [
                first[1] * second[2] - first[2] * second[1],
                first[2] * second[0] - first[0] * second[2],
                first[0] * second[1] - first[1] * second[0],
            ];
            let length =
                (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
            let normal = [normal[0] / length, normal[1] / length, normal[2] / length];
            let base = group.positions.len() as u32;
            group.positions.extend(corners);
            group.colors.extend([sea.color; 4]);
            group.normals.extend([normal; 4]);
            group
                .indices
                .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
    group.depth_test = true;
    group
}

fn animated_uvs(room: &RoomMesh, group: &RoomGroup, tick: u64) -> Vec<[f32; 2]> {
    let (Some(binding), Some(page)) = (group.anim, group.texture_page) else {
        return group.uvs.clone();
    };
    let (Some(animation), Some(&placement)) = (
        room.animations.get(binding.set as usize),
        room.uv_placements.get(page as usize),
    ) else {
        return group.uvs.clone();
    };
    let matrix = match animation.uv_transform(binding.track, animation.frame_at(tick)) {
        Some(matrix) => matrix,
        None => return group.uvs.clone(),
    };
    group
        .uvs
        .iter()
        .map(|uv| {
            let u = (uv[0] / placement[0]) + placement[2];
            let v = (uv[1] / placement[1]) + placement[3];
            let mapped_u = matrix.a * u + matrix.b * v + matrix.tx;
            let mapped_v = matrix.c * u + matrix.d * v + matrix.ty;
            [
                (mapped_u - placement[2]) * placement[0],
                (mapped_v - placement[3]) * placement[1],
            ]
        })
        .collect()
}

fn ground_group(snapshot: &SceneSnapshot) -> MeshGroup {
    let mut group = MeshGroup::default();
    let (x0, z0) = (snapshot.bounds_min.x, snapshot.bounds_min.y);
    let (x1, z1) = (snapshot.bounds_max.x, snapshot.bounds_max.y);
    let y = snapshot.ground_y;
    group.push_quad([x0, y, z1], [x1, y, z1], [x1, y, z0], [x0, y, z0], GROUND);
    group.depth_test = true;
    group
}

fn walls_group(snapshot: &SceneSnapshot, eye: Vec3) -> MeshGroup {
    let mut group = MeshGroup::default();
    let (x0, z0) = (snapshot.bounds_min.x, snapshot.bounds_min.y);
    let (x1, z1) = (snapshot.bounds_max.x, snapshot.bounds_max.y);
    let (cx, cz) = ((x0 + x1) * 0.5, (z0 + z1) * 0.5);
    let (width, depth) = (x1 - x0, z1 - z0);
    let y = snapshot.ground_y;
    group.push_box(
        cx,
        y,
        z0,
        width + WALL_THICKNESS,
        WALL_HEIGHT,
        WALL_THICKNESS,
        WALL,
        eye,
    );
    group.push_box(
        cx,
        y,
        z1,
        width + WALL_THICKNESS,
        WALL_HEIGHT,
        WALL_THICKNESS,
        WALL,
        eye,
    );
    group.push_box(
        x0,
        y,
        cz,
        WALL_THICKNESS,
        WALL_HEIGHT,
        depth + WALL_THICKNESS,
        WALL,
        eye,
    );
    group.push_box(
        x1,
        y,
        cz,
        WALL_THICKNESS,
        WALL_HEIGHT,
        depth + WALL_THICKNESS,
        WALL,
        eye,
    );
    group.depth_test = true;
    group
}

fn player_group(snapshot: &SceneSnapshot, eye: Vec3) -> MeshGroup {
    let mut group = MeshGroup::default();
    let (px, pz) = (snapshot.player_pos.x, snapshot.player_pos.z);
    group.push_box(
        px,
        snapshot.ground_y,
        pz,
        PLAYER_HALF * 2.0,
        PLAYER_HEIGHT,
        PLAYER_HALF * 2.0,
        BODY,
        eye,
    );
    let (sin, cos) = snapshot.player_face.sin_cos();
    let nose = PLAYER_HALF + 2.0;
    group.push_box(
        px + sin * nose,
        snapshot.ground_y + 8.0,
        pz + cos * nose,
        6.0,
        12.0,
        6.0,
        NOSE,
        eye,
    );
    group.depth_test = true;
    group
}
