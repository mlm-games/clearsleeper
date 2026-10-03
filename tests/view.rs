use std::f32::consts::FRAC_PI_2;
use std::time::Duration;

use clearsleeper::bdl::{AnimBinding, RoomGroup, RoomMesh, RoomTexture};
use clearsleeper::btk::{Entry, Key, TextureAnimation};
use clearsleeper::view::{
    ActorKind, ActorView, SceneSnapshot, SeaView, build_frame, sim_yaw_for_camera,
};
use clearsleeper::{App, SIM_HZ};
use glam::{Vec2, Vec3};
use repame_view3d::{OrbitCamera, SceneFilter};

#[test]
fn golden_snapshot_builds_world_frame() {
    let snapshot = SceneSnapshot {
        player_pos: Vec3::new(40.0, 0.0, -25.0),
        player_yaw: 1.0,
        player_face: 1.0,
        bounds_min: Vec2::new(-800.0, -600.0),
        bounds_max: Vec2::new(800.0, 600.0),
        ground_y: 0.0,
        walls: true,
        tick: 0,
        sky: [0.44, 0.70, 0.94, 1.0],
        sea: None,
        actors: Vec::new(),
    };
    let frame = build_frame(&snapshot, OrbitCamera::default(), None);

    assert_eq!(frame.groups.len(), 3);
    assert!(frame.groups.iter().all(|group| group.depth_test));
    assert!(frame.background.is_some());
    assert_eq!(frame.cam.target, Vec3::new(40.0, 0.0, -25.0));
    assert_eq!(frame.desc.layer_size, 1024);
    assert_eq!(frame.desc.layers, 4);
    assert_eq!(frame.desc.filter, SceneFilter::Nearest);
    assert!(frame.uploads.is_empty());

    let ground = &frame.groups[0];
    assert_eq!(ground.tri_count(), 2);
    assert_eq!(
        ground.bounds(),
        Some(([-800.0, 0.0, -600.0], [800.0, 0.0, 600.0],))
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
        player_face: 1.0,
        bounds_min: Vec2::new(-800.0, -600.0),
        bounds_max: Vec2::new(800.0, 600.0),
        ground_y: 0.0,
        walls: false,
        tick: 0,
        sky: [0.44, 0.70, 0.94, 1.0],
        sea: None,
        actors: Vec::new(),
    };
    let room = RoomMesh {
        groups: vec![RoomGroup {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
            colors: vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            normals: Vec::new(),
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 2, 1],
            texture_page: Some(2),
            lit: false,
            anim: None,
        }],
        textures: (0..3)
            .map(|_| RoomTexture {
                width: 4,
                height: 4,
                rgba: vec![255_u8; 64],
            })
            .collect(),
        ..RoomMesh::default()
    };
    let frame = build_frame(&snapshot, OrbitCamera::default(), Some(&room));

    assert_eq!(frame.groups.len(), 2);
    let ground = &frame.groups[0];
    assert_eq!(ground.tri_count(), 1);
    assert_eq!(ground.positions, room.groups[0].positions);
    assert_eq!(ground.colors, room.groups[0].colors);
    assert_eq!(ground.uvs, room.groups[0].uvs);
    assert_eq!(ground.texture_page, 2);
    assert_eq!(frame.desc.layer_size, 512);
    assert_eq!(frame.desc.layers, 3);
    assert_eq!(frame.desc.filter, SceneFilter::Linear);
    assert_eq!(frame.uploads.len(), 3);
    assert_eq!(frame.uploads[2].page, 2);
    assert_eq!((frame.uploads[2].w, frame.uploads[2].h), (4, 4));
    assert_eq!(frame.uploads[2].rgba.len(), 64);
}

#[test]
fn lit_room_group_carries_normals_and_unlit_drops_them() {
    let snapshot = SceneSnapshot {
        player_pos: Vec3::new(0.0, 0.0, 0.0),
        player_yaw: 0.0,
        player_face: 0.0,
        bounds_min: Vec2::new(-100.0, -100.0),
        bounds_max: Vec2::new(100.0, 100.0),
        ground_y: 0.0,
        walls: false,
        tick: 0,
        sky: [0.44, 0.70, 0.94, 1.0],
        sea: None,
        actors: Vec::new(),
    };
    let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
    let normals = vec![[0.0, 1.0, 0.0]; 3];
    let group = |lit: bool| RoomGroup {
        positions: positions.clone(),
        colors: vec![[1.0, 1.0, 1.0]; 3],
        normals: normals.clone(),
        uvs: Vec::new(),
        indices: vec![0, 2, 1],
        texture_page: None,
        lit,
        anim: None,
    };
    let room = RoomMesh {
        groups: vec![group(true), group(false)],
        textures: Vec::new(),
        ..RoomMesh::default()
    };
    let frame = build_frame(&snapshot, OrbitCamera::default(), Some(&room));

    assert_eq!(frame.groups.len(), 3);
    assert_eq!(frame.groups[0].normals, normals);
    assert!(frame.groups[1].normals.is_empty());
}

#[test]
fn room_group_applies_texture_animation_to_uvs() {
    let animation = TextureAnimation {
        frame_max: 10,
        dec_shift: 0,
        texture_matrix_ids: vec![0],
        centers: vec![[0.25, 0.5, 0.0]],
        names: vec!["wat".to_owned()],
        scale_values: vec![1.5, 0.75],
        rotation_values: vec![0],
        translation_values: vec![0.5, 0.0],
        entries: vec![
            Entry {
                scale: Key {
                    max_frame: 1,
                    offset: 0,
                    kind: 0,
                },
                rotation: Key {
                    max_frame: 0,
                    offset: 0,
                    kind: 0,
                },
                translation: Key {
                    max_frame: 1,
                    offset: 0,
                    kind: 0,
                },
            },
            Entry {
                scale: Key {
                    max_frame: 1,
                    offset: 1,
                    kind: 0,
                },
                rotation: Key {
                    max_frame: 0,
                    offset: 0,
                    kind: 0,
                },
                translation: Key {
                    max_frame: 1,
                    offset: 1,
                    kind: 0,
                },
            },
            Entry {
                scale: Key {
                    max_frame: 0,
                    offset: 0,
                    kind: 0,
                },
                rotation: Key {
                    max_frame: 1,
                    offset: 0,
                    kind: 0,
                },
                translation: Key {
                    max_frame: 0,
                    offset: 0,
                    kind: 0,
                },
            },
        ],
    };
    let snapshot = SceneSnapshot {
        player_pos: Vec3::ZERO,
        player_yaw: 0.0,
        player_face: 0.0,
        bounds_min: Vec2::new(-100.0, -100.0),
        bounds_max: Vec2::new(100.0, 100.0),
        ground_y: 0.0,
        walls: false,
        tick: 0,
        sky: [0.44, 0.70, 0.94, 1.0],
        sea: None,
        actors: Vec::new(),
    };
    let room = RoomMesh {
        groups: vec![
            RoomGroup {
                positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
                colors: vec![[1.0, 1.0, 1.0]; 3],
                normals: Vec::new(),
                uvs: vec![[0.4, 0.6], [0.5, 0.6], [0.4, 0.7]],
                indices: vec![0, 2, 1],
                texture_page: Some(0),
                lit: false,
                anim: Some(AnimBinding { set: 0, track: 0 }),
            },
            RoomGroup {
                positions: vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 0.0, 1.0]],
                colors: vec![[1.0, 1.0, 1.0]; 3],
                normals: Vec::new(),
                uvs: vec![[0.4, 0.6], [0.5, 0.6], [0.4, 0.7]],
                indices: vec![0, 2, 1],
                texture_page: Some(0),
                lit: false,
                anim: None,
            },
        ],
        textures: vec![RoomTexture {
            width: 4,
            height: 4,
            rgba: vec![255_u8; 64],
        }],
        uv_placements: vec![[2.0, 1.0, 0.1, 0.0]],
        animations: vec![animation],
    };
    let frame = build_frame(&snapshot, OrbitCamera::default(), Some(&room));

    let expected: [[f32; 2]; 3] = [[1.45, 0.575], [1.6, 0.575], [1.45, 0.65]];
    for (index, expected) in expected.into_iter().enumerate() {
        for axis in 0..2 {
            assert!(
                (frame.groups[0].uvs[index][axis] - expected[axis]).abs() < 1e-6,
                "animated uv {index}: {:?}",
                frame.groups[0].uvs[index]
            );
        }
    }
    assert_eq!(frame.groups[1].uvs, room.groups[1].uvs);
}

#[test]
fn golden_sea_view_builds_wave_mesh() {
    let snapshot = SceneSnapshot {
        player_pos: Vec3::new(0.0, 0.0, 0.0),
        player_yaw: 0.0,
        player_face: 0.0,
        bounds_min: Vec2::new(-100.0, -100.0),
        bounds_max: Vec2::new(100.0, 100.0),
        ground_y: 0.0,
        walls: false,
        tick: 0,
        sky: [0.44, 0.70, 0.94, 1.0],
        sea: Some(SeaView {
            origin: Vec2::new(0.0, 0.0),
            side: 3,
            heights: vec![1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 1.0],
            color: [0.5, 0.6, 0.7],
        }),
        actors: Vec::new(),
    };
    let frame = build_frame(&snapshot, OrbitCamera::default(), None);

    assert_eq!(frame.groups.len(), 3);
    assert_eq!(frame.background, Some(snapshot.sky));
    let group = &frame.groups[1];
    assert_eq!(group.tri_count(), 8);
    assert_eq!(group.positions.len(), 16);
    assert_eq!(group.colors, vec![[0.5, 0.6, 0.7]; 16]);
    assert_eq!(group.normals.len(), 16);
    assert!(group.uvs.is_empty());
    assert!(group.depth_test);
    assert_eq!(
        group.bounds(),
        Some(([0.0, 1.0, 0.0], [1600.0, 2.0, 1600.0],))
    );
    assert_eq!(
        group.normals[8],
        [0.0, 1.0, 0.0],
        "untouched quad keeps an exact up normal"
    );
    assert!(
        group.normals[12][1] < 1.0,
        "raised corner must tilt its quad: {:?}",
        group.normals[12]
    );
    for normal in &group.normals {
        let length = normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2];
        assert!((length - 1.0).abs() < 1e-5, "normal not unit: {normal:?}");
    }
}

#[test]
fn sea_group_culls_beyond_the_far_plane() {
    let snapshot = SceneSnapshot {
        player_pos: Vec3::new(0.0, 0.0, 0.0),
        player_yaw: 0.0,
        player_face: 0.0,
        bounds_min: Vec2::new(-100.0, -100.0),
        bounds_max: Vec2::new(100.0, 100.0),
        ground_y: 0.0,
        walls: false,
        tick: 0,
        sky: [0.44, 0.70, 0.94, 1.0],
        sea: Some(SeaView {
            origin: Vec2::new(50_000.0, 50_000.0),
            side: 3,
            heights: vec![1.0; 9],
            color: [0.5, 0.6, 0.7],
        }),
        actors: Vec::new(),
    };
    let frame = build_frame(&snapshot, OrbitCamera::default(), None);

    assert_eq!(frame.groups.len(), 3);
    assert_eq!(frame.groups[1].tri_count(), 0);
}

#[test]
fn golden_actor_layer_draws_every_kind() {
    let snapshot = SceneSnapshot {
        player_pos: Vec3::new(0.0, 0.0, 0.0),
        player_yaw: 0.0,
        player_face: 0.0,
        bounds_min: Vec2::new(-100.0, -100.0),
        bounds_max: Vec2::new(100.0, 100.0),
        ground_y: 0.0,
        walls: true,
        tick: 0,
        sky: [0.44, 0.70, 0.94, 1.0],
        sea: None,
        actors: vec![
            ActorView {
                pos: Vec3::new(30.0, 0.0, 0.0),
                kind: ActorKind::Enemy,
                item: 0,
                height: 24.0,
                half: 12.0,
            },
            ActorView {
                pos: Vec3::new(-40.0, 0.0, 10.0),
                kind: ActorKind::Pickup,
                item: 0,
                height: 10.0,
                half: 10.0,
            },
            ActorView {
                pos: Vec3::new(0.0, 0.0, 60.0),
                kind: ActorKind::Ship,
                item: 0,
                height: 26.0,
                half: 44.0,
            },
        ],
    };

    let frame = build_frame(&snapshot, OrbitCamera::default(), None);

    assert_eq!(frame.groups.len(), 4, "the actor layer joins the frame");
    let actors = frame.groups.last().expect("actor layer");
    assert!(actors.tri_count() > 0, "each actor puts geometry on screen");
    let (min, max) = actors.bounds().expect("actors have geometry");
    assert!(
        min[0] <= -40.0 && max[0] >= 30.0,
        "the spread covers foe and cargo: {min:?} {max:?}"
    );
    assert!(max[2] >= 60.0, "the ship reaches its berth: {max:?}");
}
