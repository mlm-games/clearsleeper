use glam::{Vec2, Vec3};
use repame_view3d::{BatchDesc, Frame3d, MeshGroup, OrbitCamera, Rgb, SceneFilter, SceneUpload};

use crate::bdl::{LAYER_SIZE, RoomGroup, RoomMesh};

const SKY: [f32; 4] = [0.44, 0.70, 0.94, 1.0];
const GROUND: Rgb = [0.24, 0.42, 0.22];
const WALL: Rgb = [0.48, 0.20, 0.20];
const BODY: Rgb = [0.87, 0.85, 0.78];
const NOSE: Rgb = [0.82, 0.55, 0.20];

const WALL_HEIGHT: f32 = 30.0;
const WALL_THICKNESS: f32 = 8.0;
const PLAYER_HALF: f32 = 8.0;
const PLAYER_HEIGHT: f32 = 32.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneSnapshot {
    pub player_pos: Vec3,
    pub player_yaw: f32,
    pub bounds_min: Vec2,
    pub bounds_max: Vec2,
    pub ground_y: f32,
    pub walls: bool,
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

pub fn build_frame(
    snapshot: &SceneSnapshot,
    cam: OrbitCamera,
    room: Option<&RoomMesh>,
) -> Frame3d {
    let cam = OrbitCamera {
        target: Vec3::new(snapshot.player_pos.x, snapshot.ground_y, snapshot.player_pos.z),
        ..cam
    };
    let mut frame = Frame3d {
        cam,
        background: Some(SKY),
        desc: batch_desc(room),
        ..Frame3d::default()
    };
    let eye = frame.cam.eye();
    match room {
        Some(room) => {
            for group in &room.groups {
                frame.push(room_group(group));
            }
            frame.uploads.extend(
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
    if snapshot.walls {
        frame.push(walls_group(snapshot, eye));
    }
    frame.push(player_group(snapshot, eye));
    frame
}

fn room_group(group: &RoomGroup) -> MeshGroup {
    MeshGroup {
        positions: group.positions.clone(),
        colors: group.colors.clone(),
        uvs: group.uvs.clone(),
        indices: group.indices.clone(),
        texture_page: group.texture_page.unwrap_or(0),
        depth_test: true,
        ..MeshGroup::default()
    }
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
    let (sin, cos) = snapshot.player_yaw.sin_cos();
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
