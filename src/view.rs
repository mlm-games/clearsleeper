use glam::{Vec2, Vec3};
use repame_view3d::{Frame3d, MeshGroup, OrbitCamera, Rgb};

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
}

pub fn sim_yaw_for_camera(cam_yaw: f32) -> f32 {
    (-cam_yaw.cos()).atan2(-cam_yaw.sin())
}

pub fn build_frame(snapshot: &SceneSnapshot, cam: OrbitCamera) -> Frame3d {
    let cam = OrbitCamera {
        target: Vec3::new(snapshot.player_pos.x, 0.0, snapshot.player_pos.z),
        ..cam
    };
    let mut frame = Frame3d {
        cam,
        background: Some(SKY),
        ..Frame3d::default()
    };
    let eye = frame.cam.eye();
    frame.push(ground_group(snapshot));
    frame.push(walls_group(snapshot, eye));
    frame.push(player_group(snapshot, eye));
    frame
}

fn ground_group(snapshot: &SceneSnapshot) -> MeshGroup {
    let mut group = MeshGroup::default();
    let (x0, z0) = (snapshot.bounds_min.x, snapshot.bounds_min.y);
    let (x1, z1) = (snapshot.bounds_max.x, snapshot.bounds_max.y);
    group.push_quad([x0, 0.0, z1], [x1, 0.0, z1], [x1, 0.0, z0], [x0, 0.0, z0], GROUND);
    group.depth_test = true;
    group
}

fn walls_group(snapshot: &SceneSnapshot, eye: Vec3) -> MeshGroup {
    let mut group = MeshGroup::default();
    let (x0, z0) = (snapshot.bounds_min.x, snapshot.bounds_min.y);
    let (x1, z1) = (snapshot.bounds_max.x, snapshot.bounds_max.y);
    let (cx, cz) = ((x0 + x1) * 0.5, (z0 + z1) * 0.5);
    let (width, depth) = (x1 - x0, z1 - z0);
    group.push_box(
        cx,
        0.0,
        z0,
        width + WALL_THICKNESS,
        WALL_HEIGHT,
        WALL_THICKNESS,
        WALL,
        eye,
    );
    group.push_box(
        cx,
        0.0,
        z1,
        width + WALL_THICKNESS,
        WALL_HEIGHT,
        WALL_THICKNESS,
        WALL,
        eye,
    );
    group.push_box(
        x0,
        0.0,
        cz,
        WALL_THICKNESS,
        WALL_HEIGHT,
        depth + WALL_THICKNESS,
        WALL,
        eye,
    );
    group.push_box(
        x1,
        0.0,
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
        0.0,
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
        8.0,
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
