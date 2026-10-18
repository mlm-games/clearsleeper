use glam::{Mat4, Vec2, Vec3};
use repame_view3d::{
    BatchDesc, Frame3d, MeshGroup, OrbitCamera, Rgb, SceneFilter, SceneUpload, SkinnedDraw,
    SkinnedMesh,
};

use crate::j3d::bdl::{LAYER_SIZE, RoomGroup, RoomMesh, SKIN_INFLUENCES};

pub const LINK_ARCHIVE: &str = "Link";

pub const SKY: [f32; 4] = [0.44, 0.70, 0.94, 1.0];
const GROUND: Rgb = [0.24, 0.42, 0.22];
const WALL: Rgb = [0.48, 0.20, 0.20];
const WALL_HEIGHT: f32 = 30.0;
const WALL_THICKNESS: f32 = 8.0;

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
    Balloon,
    Warship,
    Cannon,
    Fire,
    Gull,
    Scenery,
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
    pub particles: Vec<ParticleView>,
}

/// One live emitter as the draw layer needs it: where it is, how big, what
/// colour it has faded to and how far it has rolled.
///
/// `repame-view3d` has no billboard or sprite primitive — its mesh vocabulary
/// is indexed triangles, one texture page and one group alpha — so this is the
/// render seam. The quad below is built from these fields and the camera eye;
/// nothing here pretends to be the retail sprite, which is a `J2DParticle`
/// textured quad whose texel this port has no page for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleView {
    pub pos: Vec3,
    /// Half-extents in the quad's own plane, x across and y up.
    pub half: Vec3,
    pub color: Rgb,
    pub alpha: f32,
    pub roll: f32,
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
            mips: 1,
        },
        None => BatchDesc::default(),
    }
}

pub fn build_frame(snapshot: &SceneSnapshot, cam: OrbitCamera, room: Option<&RoomMesh>) -> Frame3d {
    build_frame_with_actor(snapshot, cam, room, None)
}

pub fn build_frame_with_actor(
    snapshot: &SceneSnapshot,
    cam: OrbitCamera,
    room: Option<&RoomMesh>,
    actor: Option<&RoomMesh>,
) -> Frame3d {
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
        // Retail culls by the view frustum (`J3DShape` / `J3DViewMtx`); the GameCube has
        // no hierarchical-Z test, so the engine's armed-by-default occlusion feed would
        // both cull groups this port never asked it to and append a depth pass + readback
        // whose pyramid nobody reads.
        occlusion_feed: None,
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
    if !snapshot.actors.is_empty() {
        frame.push(actors_group(snapshot, eye));
    }
    for particle in particle_groups(snapshot, eye) {
        frame.push(particle);
    }
    if let Some(mesh) = actor.and_then(|mesh| skinned_draw(mesh, snapshot)) {
        frame.skinned.push(mesh);
    }
    frame
}

pub fn skinned_draw(actor: &RoomMesh, snapshot: &SceneSnapshot) -> Option<SkinnedDraw> {
    let group = actor.groups.iter().find(|group| {
        group.joint_indices.len() == group.positions.len() && !group.joint_indices.is_empty()
    })?;
    if actor.joint_matrices.len() != actor.joints.len() {
        return None;
    }
    let inverse_bind: Vec<Mat4> = actor
        .joint_matrices
        .iter()
        .map(|matrix| matrix.try_inverse().unwrap_or(Mat4::IDENTITY))
        .collect();
    let mut mesh = SkinnedMesh {
        positions: group.positions.clone(),
        normals: group.normals.clone(),
        uvs: group.uvs.clone(),
        colors: group.colors.clone(),
        indices: group.indices.clone(),
        joints: group.joint_indices.clone(),
        weights: group.joint_weights.clone(),
        inverse_bind,
        joint_nodes: (0..actor.joints.len()).collect(),
        texture_page: group.texture_page.unwrap_or(0),
        depth_test: true,
        ..SkinnedMesh::default()
    };
    mesh.name = "cl".to_string();
    let placed = place_actor(idle_pose(actor), snapshot);
    SkinnedDraw::from_matrices(&mesh, &placed)
}

/// Places a posed actor at the player, in world space.
///
/// `pose` holds one animated **world** matrix per joint: the bind chain from
/// `RoomMesh::joint_matrices` with the animated locals applied on the right of
/// each one. `SkinnedDraw::from_matrices` then multiplies each by that joint's
/// `inverse_bind`, so the bind chain cancels and what survives is the animated
/// delta — which is the identity at rest, and therefore the bind pose on screen.
///
/// Passing local rotations here instead would leave `bindWorld * inverse(bindWorld)`
/// unreduced on the right of every joint and displace the actor by the negated
/// bind translations.
fn place_actor(pose: Vec<Mat4>, snapshot: &SceneSnapshot) -> Vec<Mat4> {
    let origin = Vec3::new(
        snapshot.player_pos.x,
        snapshot.ground_y,
        snapshot.player_pos.z,
    );
    let to_world = Mat4::from_translation(origin) * Mat4::from_rotation_y(-snapshot.player_yaw);
    pose.into_iter().map(|joint| to_world * joint).collect()
}

/// World matrix per joint for the bind pose with a small idle swing layered on.
///
/// Starts from the bind chain and applies each swing as a **local** delta
/// composed onto the joint's own parent world, which is how J3D builds an
/// animated world matrix: `world = parentWorld * bindLocal * animDelta`.
fn idle_pose(actor: &RoomMesh) -> Vec<Mat4> {
    const SWING: [usize; 6] = [5, 6, 9, 10, 22, 23];
    let locals = actor.joint_locals();
    let bind = &actor.joint_matrices;
    let parents = &actor.joint_parents;
    let mut world = vec![Mat4::IDENTITY; bind.len()];
    let mut resolved = vec![false; bind.len()];
    for joint in 0..bind.len() {
        let swing = SWING
            .iter()
            .position(|slot| *slot == joint)
            .map(|index| {
                let angle = 0.12 * (index + 1) as f32;
                Mat4::from_rotation_z(angle) * Mat4::from_rotation_x(angle * 0.5)
            })
            .unwrap_or(Mat4::IDENTITY);
        let local = locals.get(joint).copied().unwrap_or(Mat4::IDENTITY);
        let animated = local * swing;
        let parent = parents
            .get(joint)
            .copied()
            .filter(|parent| *parent < joint as u16 && resolved[*parent as usize])
            .map(|parent| world[parent as usize])
            .unwrap_or(Mat4::IDENTITY);
        world[joint] = parent * animated;
        resolved[joint] = true;
    }
    world
}

pub fn skinned_influences() -> usize {
    SKIN_INFLUENCES
}

/// One camera-facing quad per live emitter.
///
/// `repame-view3d` exposes no billboard or sprite primitive, so the quad is
/// built here from the emitter's half-extents and the camera eye: the right
/// vector is the world up crossed with the view ray, the up vector closes the
/// frame, and the roll turns the pair in the quad's own plane. Each group is
/// its own alpha blend so one emitter's fade never dims another's, which is
/// also what makes the batch sort them back to front.
fn particle_groups(snapshot: &SceneSnapshot, eye: Vec3) -> Vec<MeshGroup> {
    let mut groups = Vec::with_capacity(snapshot.particles.len());
    for particle in &snapshot.particles {
        let ray = eye - particle.pos;
        let forward = ray.normalize_or(Vec3::Z);
        let mut right = Vec3::Y.cross(forward);
        if right.length_squared() < 1e-8 {
            right = Vec3::X.cross(forward);
        }
        let right = right.normalize();
        let up = forward.cross(right);
        let (sin, cos) = particle.roll.sin_cos();
        let across = right * cos - up * sin;
        let above = right * sin + up * cos;
        let (rx, ry) = (across * particle.half.x, above * particle.half.y);
        let mut group = MeshGroup {
            transparent: true,
            alpha: particle.alpha.clamp(0.0, 1.0),
            depth_test: true,
            ..MeshGroup::default()
        };
        group.push_quad(
            (particle.pos - rx - ry).into(),
            (particle.pos + rx - ry).into(),
            (particle.pos + rx + ry).into(),
            (particle.pos - rx + ry).into(),
            particle.color,
        );
        groups.push(group);
    }
    groups
}

const ENEMY: Rgb = [0.55, 0.26, 0.16];
const TALKER: Rgb = [0.32, 0.42, 0.82];
const SHIP: Rgb = [0.58, 0.44, 0.24];
const BOMB: Rgb = [0.18, 0.18, 0.22];
const ARROW: Rgb = [0.86, 0.82, 0.66];
const BOOM: Rgb = [0.72, 0.30, 0.52];
const BALLOON: Rgb = [0.86, 0.44, 0.52];
const WARSHIP: Rgb = [0.30, 0.26, 0.32];
const EMBER: Rgb = [0.94, 0.52, 0.18];
const SEAGULL: Rgb = [0.94, 0.94, 0.90];
const SCENERY: Rgb = [0.44, 0.62, 0.34];
const SPEAR: Rgb = [0.70, 0.66, 0.52];

pub fn item_hex(kind: crate::sim::item::Kind) -> &'static str {
    match kind {
        crate::sim::item::Kind::Bomb => "#2E3138",
        crate::sim::item::Kind::Arrow => "#C9C4B4",
        crate::sim::item::Kind::Boomerang => "#B84C85",
        crate::sim::item::Kind::Hookshot => "#7C8C9C",
    }
}
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
        0x0B..=0x0E => (BOMB, 12.0),
        0x10..=0x12 => (ARROW, 10.0),
        0x2D => (BOOM, 14.0),
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
        ActorKind::Spear => SPEAR,
        ActorKind::Balloon => BALLOON,
        ActorKind::Warship => WARSHIP,
        ActorKind::Cannon => BOMB,
        ActorKind::Fire => EMBER,
        ActorKind::Gull => SEAGULL,
        ActorKind::Pickup => pickup_look(actor.item).0,
        ActorKind::Scenery => SCENERY,
    }
}

fn actors_group(snapshot: &SceneSnapshot, eye: Vec3) -> MeshGroup {
    let mut group = MeshGroup::default();
    for actor in &snapshot.actors {
        group.push_box(
            actor.pos.x,
            actor.pos.y,
            actor.pos.z,
            actor.half * 2.0,
            actor.height,
            actor.half * 2.0,
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
