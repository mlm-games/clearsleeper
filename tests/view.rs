use std::f32::consts::FRAC_PI_2;
use std::time::Duration;

use clearsleeper::bdl::RoomMesh;
use clearsleeper::view::{SceneSnapshot, build_frame, sim_yaw_for_camera};
use clearsleeper::{App, SIM_HZ};
use glam::{Vec2, Vec3};
use repame_view3d::OrbitCamera;

#[test]
fn golden_snapshot_builds_world_frame() {
    let snapshot = SceneSnapshot {
        player_pos: Vec3::new(40.0, 0.0, -25.0),
        player_yaw: 1.0,
        bounds_min: Vec2::new(-800.0, -600.0),
        bounds_max: Vec2::new(800.0, 600.0),
        ground_y: 0.0,
        walls: true,
    };
    let frame = build_frame(&snapshot, OrbitCamera::default(), None);

    assert_eq!(frame.groups.len(), 3);
    assert!(frame.groups.iter().all(|group| group.depth_test));
    assert!(frame.background.is_some());
    assert_eq!(frame.cam.target, Vec3::new(40.0, 0.0, -25.0));

    let ground = &frame.groups[0];
    assert_eq!(ground.tri_count(), 2);
    assert_eq!(
        ground.bounds(),
        Some((
            [-800.0, 0.0, -600.0],
            [800.0, 0.0, 600.0],
        ))
    );

    let walls = frame.groups[1].bounds().expect("walls have geometry");
    assert!(walls.0[0] <= -800.0 && walls.1[0] >= 800.0, "{walls:?}");
    assert!(walls.0[2] <= -600.0 && walls.1[2] >= 600.0, "{walls:?}");

    let (sin, cos) = snapshot.player_yaw.sin_cos();
    let body_max = 8.0 * (sin.abs() + cos.abs());
    let nose_max = 10.0 + 3.0 * (sin.abs() + cos.abs());
    let facing = Vec2::new(sin, cos);
    let origin = snapshot.player_pos.x * facing.x + snapshot.player_pos.z * facing.y;
    let max_projection = frame.groups[2]
        .positions
        .iter()
        .map(|[x, _, z]| x * facing.x + z * facing.y)
        .fold(f32::MIN, f32::max);
    assert!(
        max_projection > origin + (body_max + nose_max) * 0.5,
        "facing nose beyond body: {max_projection} vs body {body_max}"
    );
    assert!(
        max_projection <= origin + nose_max + 1e-3,
        "nose bound: {max_projection}"
    );
}

#[test]
fn camera_relative_walk_follows_view_yaw() {
    assert!(sim_yaw_for_camera(-FRAC_PI_2).abs() < 1e-6);

    let mut app = App::new();
    app.cam.update(|cam| cam.yaw = -FRAC_PI_2);
    app.stage_camera();
    app.stage_axis(Vec2::new(0.0, 1.0));
    let tick = Duration::from_secs_f64(1.0 / SIM_HZ);
    for _ in 0..30 {
        app.advance(tick);
    }

    let snapshot = app.snapshot();
    assert!(
        snapshot.player_pos.z > 100.0,
        "sticks forward along view axis: {:?}",
        snapshot.player_pos
    );
    assert!(
        snapshot.player_pos.x.abs() < 1e-3,
        "no lateral drift: {:?}",
        snapshot.player_pos
    );
}

#[test]
fn room_mesh_replaces_placeholder_ground() {
    let snapshot = SceneSnapshot {
        player_pos: Vec3::new(40.0, 0.0, -25.0),
        player_yaw: 1.0,
        bounds_min: Vec2::new(-800.0, -600.0),
        bounds_max: Vec2::new(800.0, 600.0),
        ground_y: 0.0,
        walls: false,
    };
    let room = RoomMesh {
        positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
        colors: vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        indices: vec![0, 2, 1],
    };
    let frame = build_frame(&snapshot, OrbitCamera::default(), Some(&room));

    assert_eq!(frame.groups.len(), 2);
    let ground = &frame.groups[0];
    assert_eq!(ground.tri_count(), 1);
    assert_eq!(ground.positions, room.positions);
    assert_eq!(ground.colors, room.colors);
}
