use std::io::Write as _;
use std::time::Duration;

use clearsleeper::assets::{AssetError, AssetRoot, Archive, stage_spawn, yaz0_decode};
use clearsleeper::btk::TexSrt;
use glam::Vec3;

#[test]
fn golden_yaz0_literal_and_backreference_streams() {
    let mut literals = Vec::new();
    literals.extend_from_slice(b"Yaz0");
    literals.extend_from_slice(&6_u32.to_be_bytes());
    literals.extend_from_slice(&[0; 8]);
    literals.extend_from_slice(&[0xE0, b'A', b'B', b'C', 0x10, 0x02]);
    assert_eq!(yaz0_decode(&literals, "golden.bin").unwrap(), b"ABCABC");

    let mut long_backref = Vec::new();
    long_backref.extend_from_slice(b"Yaz0");
    long_backref.extend_from_slice(&21_u32.to_be_bytes());
    long_backref.extend_from_slice(&[0; 8]);
    long_backref.extend_from_slice(&[0xC0, b'A', b'B', 0x00, 0x01, 0x01]);
    let expected: Vec<u8> = (0..21)
        .map(|i| if i % 2 == 0 { b'A' } else { b'B' })
        .collect();
    assert_eq!(yaz0_decode(&long_backref, "golden.bin").unwrap(), expected);
}

#[test]
fn yaz0_and_rarc_reject_unexpected_magic() {
    let err = yaz0_decode(b"RARCnot-yaz0", "mem.bin").unwrap_err();
    match err {
        AssetError::BadMagic {
            expected, actual, ..
        } => {
            assert_eq!(expected, "Yaz0");
            assert_eq!(&actual, b"RARC");
        }
        other => panic!("unexpected error: {other}"),
    }

    let err = Archive::parse(b"NOPEdata".to_vec(), "mem.arc").unwrap_err();
    match err {
        AssetError::BadMagic {
            expected, actual, ..
        } => {
            assert_eq!(expected, "RARC");
            assert_eq!(&actual, b"NOPE");
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn golden_archive_walks_nested_entries() {
    let archive = Archive::parse(synthetic_archive(), "synthetic.arc").unwrap();
    assert_eq!(archive.read("hello.bin").unwrap(), b"hello!");
    assert_eq!(archive.read("sub/nested.bin").unwrap(), b"NESTED!!");
    assert_eq!(archive.read("sub/../hello.bin").unwrap(), b"hello!");
    assert!(
        matches!(archive.read("absent.bin"), Err(AssetError::MissingEntry { .. })),
        "expected missing entry"
    );
    assert!(
        matches!(archive.read("sub"), Err(AssetError::MissingEntry { .. })),
        "expected missing entry for directory path"
    );
    assert!(
        matches!(
            archive.read("hello.bin/x"),
            Err(AssetError::MissingEntry { .. })
        ),
        "expected missing entry under file path"
    );
}

#[test]
fn missing_root_and_missing_file_are_distinct_errors() {
    let root_path =
        std::env::temp_dir().join(format!("clearsleeper_assets_{}", std::process::id()));
    let root = AssetRoot::at(&root_path);
    assert!(
        matches!(
            root.read("anything.bin"),
            Err(AssetError::MissingRoot { .. })
        ),
        "expected missing root"
    );

    std::fs::create_dir_all(&root_path).unwrap();
    assert!(
        matches!(
            root.read("absent.bin"),
            Err(AssetError::MissingFile { .. })
        ),
        "expected missing file"
    );

    let mut file = std::fs::File::create(root_path.join("present.bin")).unwrap();
    file.write_all(b"data").unwrap();
    assert_eq!(root.read("present.bin").unwrap(), b"data");
    std::fs::remove_dir_all(&root_path).unwrap();

    if std::env::var_os(clearsleeper::assets::ASSETS_ENV).is_none() {
        assert!(
            matches!(
                AssetRoot::from_env(),
                Err(AssetError::MissingRoot { .. })
            ),
            "expected missing root for unset CS_ASSETS"
        );
    }
}

#[test]
fn real_disc_archives_load_through_cs_assets() {
    let Ok(root) = std::env::var(clearsleeper::assets::ASSETS_ENV) else {
        eprintln!("CS_ASSETS is not set; skipping real-disc archive check");
        return;
    };
    let root = AssetRoot::at(root);

    let raw = root.read_archive("files/res/Object/Vobj07.arc").unwrap();
    assert_eq!(raw.read("bdl/vobj07.bdl").unwrap().len(), 31_456);

    let packed = root.read_archive("files/res/Stage/Abesso/Stage.arc").unwrap();
    assert_eq!(packed.read("dat/event_list.dat").unwrap().len(), 130_352);
    assert_eq!(packed.read("dzs/stage.dzs").unwrap().len(), 864);

    let room = root
        .read_archive("files/res/Stage/sea/Room44.arc")
        .unwrap();
    assert_eq!(room.read("dzb/room.dzb").unwrap().len(), 124_416);

    let stage = clearsleeper::assets::load_stage().expect("stage loads from disc");
    assert!(
        (stage.spawn - Vec3::new(-194_845.4, 145.0, 314_242.8)).length() < 1.0,
        "unexpected spawn {:?}",
        stage.spawn
    );
    let expected_yaw = 4187.0 * std::f32::consts::TAU / 65_536.0;
    assert!((stage.yaw - expected_yaw).abs() < 1e-4, "yaw {}", stage.yaw);
    let ground = stage
        .collision
        .ground_at(stage.spawn.x, stage.spawn.z)
        .expect("ground under spawn");
    assert!(ground <= stage.spawn.y, "ground {ground} above spawn");
}

#[test]
fn golden_room_mesh_matches_disc_geometry() {
    let Ok(_) = std::env::var(clearsleeper::assets::ASSETS_ENV) else {
        eprintln!("CS_ASSETS is not set; skipping real-disc room mesh check");
        return;
    };
    let mesh = clearsleeper::assets::load_room_mesh().expect("room mesh loads from disc");
    assert_eq!(mesh.groups.len(), 17);
    assert_eq!(mesh.textures.len(), 18);
    let pages = [
        Some(0),
        Some(3),
        Some(1),
        Some(2),
        Some(5),
        Some(4),
        Some(7),
        Some(6),
        Some(8),
        Some(9),
        Some(10),
        Some(10),
        Some(14),
        Some(15),
        Some(16),
        None,
        Some(17),
    ];
    let shapes: [(usize, usize); 17] = [
        (164, 84),
        (1550, 950),
        (4055, 1987),
        (5834, 3010),
        (1740, 944),
        (675, 417),
        (1534, 814),
        (25, 13),
        (11, 5),
        (1040, 584),
        (110, 58),
        (120, 62),
        (686, 368),
        (110, 58),
        (110, 58),
        (12, 6),
        (64, 40),
    ];
    for (index, (group, (vertices, triangles))) in
        mesh.groups.iter().zip(shapes).enumerate()
    {
        assert_eq!(group.texture_page, pages[index], "group {index}");
        assert_eq!(group.positions.len(), vertices, "group {index} positions");
        assert_eq!(group.indices.len(), triangles * 3, "group {index} indices");
        assert_eq!(group.colors.len(), group.positions.len(), "group {index}");
        assert!(
            group
                .indices
                .iter()
                .all(|&index| (index as usize) < group.positions.len()),
            "group {index} index bounds"
        );
        if group.texture_page.is_none() {
            assert!(group.uvs.is_empty(), "group {index} untextured uvs");
        } else {
            assert_eq!(group.uvs.len(), group.positions.len(), "group {index} uvs");
            assert!(
                group.uvs.iter().all(|[u, v]| {
                    *u >= -1e-6 && *u <= 1.0 + 1e-6 && *v >= -1e-6 && *v <= 1.0 + 1e-6
                }),
                "group {index} uvs out of unit range"
            );
        }
        assert!(!group.lit, "group {index} lit");
        if index == 11 {
            assert_eq!(
                group.normals.len(),
                group.positions.len(),
                "group {index} normals"
            );
            assert!(
                group
                    .normals
                    .iter()
                    .all(|normal| *normal == [0.0, 1.0, 0.0]),
                "group {index} normals {:?}",
                group.normals[0]
            );
        } else {
            assert!(group.normals.is_empty(), "group {index} normals");
        }
    }
    let positions: usize = mesh.groups.iter().map(|group| group.positions.len()).sum();
    let indices: usize = mesh.groups.iter().map(|group| group.indices.len()).sum();
    assert_eq!(positions, 17_840);
    assert_eq!(indices, 9_458 * 3);
    assert!(
        mesh.groups
            .iter()
            .flat_map(|group| &group.colors)
            .any(|color| *color != [1.0, 1.0, 1.0])
    );

    let expected_sizes: [(u32, u32); 18] = [
        (257, 257),
        (512, 512),
        (193, 257),
        (377, 512),
        (512, 512),
        (512, 512),
        (129, 129),
        (386, 503),
        (65, 65),
        (257, 97),
        (129, 129),
        (129, 129),
        (129, 129),
        (17, 17),
        (257, 195),
        (129, 129),
        (129, 129),
        (1, 17),
    ];
    for (index, (width, height)) in expected_sizes.into_iter().enumerate() {
        let texture = &mesh.textures[index];
        assert_eq!((texture.width, texture.height), (width, height), "texture {index}");
        assert_eq!(
            texture.rgba.len() as u32,
            width * height * 4,
            "texture {index} bytes"
        );
    }
    let expected_pixels: [([u8; 4], usize); 9] = [
        ([74, 178, 66, 0], 0),
        ([148, 113, 66, 255], 2),
        ([90, 130, 49, 255], 6),
        ([90, 130, 49, 255], 7),
        ([0, 0, 0, 0], 8),
        ([255, 255, 255, 255], 9),
        ([255, 255, 255, 255], 13),
        ([255, 255, 255, 0], 15),
        ([255, 255, 255, 255], 17),
    ];
    for (pixel, index) in expected_pixels {
        let texture = &mesh.textures[index];
        assert_eq!(&texture.rgba[0..4], &pixel, "texture {index} origin pixel");
    }

    let mut minimum = [f32::MAX; 3];
    let mut maximum = [f32::MIN; 3];
    for group in &mesh.groups {
        for position in &group.positions {
            for axis in 0..3 {
                minimum[axis] = minimum[axis].min(position[axis]);
                maximum[axis] = maximum[axis].max(position[axis]);
            }
        }
    }
    assert!((minimum[0] + 250_000.0).abs() < 0.5, "x min {}", minimum[0]);
    assert!((maximum[0] + 150_000.0).abs() < 0.5, "x max {}", maximum[0]);
    assert!((minimum[1] + 43.43).abs() < 0.5, "y min {}", minimum[1]);
    assert!((maximum[1] - 5530.08).abs() < 0.5, "y max {}", maximum[1]);
    assert!((minimum[2] - 250_000.0).abs() < 0.5, "z min {}", minimum[2]);
    assert!((maximum[2] - 350_000.0).abs() < 0.5, "z max {}", maximum[2]);

    let stage = clearsleeper::assets::load_stage().expect("stage loads from disc");
    let spawn = [stage.spawn.x, stage.spawn.z];
    let mut top: Option<(f32, f32)> = None;
    for group in &mesh.groups {
        for triangle in group.indices.chunks_exact(3) {
            let a = group.positions[triangle[0] as usize];
            let b = group.positions[triangle[1] as usize];
            let c = group.positions[triangle[2] as usize];
            if !inside_xz(a, b, c, spawn) {
                continue;
            }
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let normal = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            let length = normal.iter().map(|axis| axis * axis).sum::<f32>().sqrt();
            if length < 1e-9 {
                continue;
            }
            let ny = normal[1] / length;
            if ny.abs() < 0.9 {
                continue;
            }
            let y = (a[1] + b[1] + c[1]) / 3.0;
            if top.is_none_or(|(best, _)| y > best) {
                top = Some((y, ny));
            }
        }
    }
    let (y, ny) = top.expect("horizontal ground under spawn");
    assert!((y - 143.37).abs() < 1.0, "ground y {y}");
    assert!(ny > 0.5, "surface normal {ny}");

    assert_eq!(mesh.animations.len(), 1);
    assert_eq!(mesh.uv_placements.len(), mesh.textures.len());
    let animation = &mesh.animations[0];
    assert_eq!(animation.frame_max, 100);
    assert_eq!(animation.dec_shift, 1);
    assert_eq!(animation.track_count(), 16);
    assert_eq!(animation.track("SC_01_mizuB_v_x", 0), Some(0));
    assert_eq!(animation.track("SC_01_mizu_v", 0), Some(2));
    assert_eq!(animation.track("SC_01_mizu_v", 1), Some(3));
    assert_eq!(animation.track("SC_01_mizu_v(2)", 4), Some(10));
    assert_eq!(animation.track("SC_01_mizu_v_x(2)", 0), Some(11));
    assert_eq!(animation.track("SC_01_mizu", 0), None);
    assert_eq!(animation.center(2), Some([0.5, 0.5, 0.5]));
    assert_eq!(
        animation.eval(2, 0.0),
        TexSrt {
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0,
            translation_x: 0.0,
            translation_y: 0.0,
        }
    );
    assert_eq!(
        animation.eval(2, 25.0),
        TexSrt {
            scale_x: 0.949999988079071,
            scale_y: 0.949999988079071,
            rotation: 16370,
            translation_x: 0.0,
            translation_y: 0.0,
        }
    );
    assert_eq!(
        animation.eval(2, 50.0),
        TexSrt {
            scale_x: 0.8999999761581421,
            scale_y: 0.8999999761581421,
            rotation: 32766,
            translation_x: 0.0,
            translation_y: 0.0,
        }
    );
    assert_eq!(animation.frame_at(0), 0.0);
    assert_eq!(animation.frame_at(10), 10.0);
    assert_eq!(animation.frame_at(25), 25.0);
    assert_eq!(animation.frame_at(299), 99.0);
    let tracks: [Option<u32>; 17] = [
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(2),
        Some(4),
        Some(6),
        Some(0),
        Some(11),
        Some(13),
        Some(15),
        None,
        None,
    ];
    for (index, expected) in tracks.into_iter().enumerate() {
        assert_eq!(
            mesh.groups[index].anim.map(|binding| (binding.set, binding.track)),
            expected.map(|track| (0, track)),
            "group {index} anim"
        );
    }
}

#[test]
fn golden_stage_tables_match_disc() {
    let Ok(_) = std::env::var(clearsleeper::assets::ASSETS_ENV) else {
        eprintln!("CS_ASSETS is not set; skipping real-disc stage tables check");
        return;
    };
    let tables =
        clearsleeper::assets::load_stage_tables().expect("stage tables load from disc");
    assert_eq!(tables.placements().len(), 50);
    assert_eq!(tables.room_groups().len(), 50);

    let placement = tables.placement(44).expect("room 44 placement");
    assert_eq!(placement.trans, [-200_000.0, 300_000.0]);
    assert_eq!(placement.angle, 0);
    assert_eq!(placement.room, 44);
    assert_eq!(placement.wave_max, 0);
    let placeholder = tables.placement(0).expect("room 0 placement");
    assert_eq!(placeholder.trans, [0.0, 0.0]);
    assert_eq!(placeholder.wave_max, 255);

    assert_eq!(tables.load_set(0), &[0_u8]);
    assert_eq!(tables.load_set(44), &[0_u8, 44_u8]);
    assert_eq!(tables.load_set(45), &[0_u8, 45_u8]);
    assert!(tables.load_set(50).is_empty());

    for placement in tables.placements() {
        if placement.room == 0 {
            continue;
        }
        assert_eq!(
            tables.room_at(placement.trans[0], placement.trans[1]),
            placement.room,
            "room {} placement cell",
            placement.room
        );
    }

    assert_eq!(
        clearsleeper::assets::start_set(&tables),
        vec![0_u8, 44_u8]
    );
}

#[test]
fn golden_room_flow_reloads_set_across_cells() {
    let Ok(_) = std::env::var(clearsleeper::assets::ASSETS_ENV) else {
        eprintln!("CS_ASSETS is not set; skipping real-disc room flow check");
        return;
    };
    let tick = Duration::from_secs_f64(1.0 / clearsleeper::SIM_HZ);
    let mut app = clearsleeper::App::new();

    assert_eq!(app.room.as_ref().expect("start room mesh").groups.len(), 17);
    let area = app.sim.world.resource::<clearsleeper::sim::area::Area>();
    assert!(
        area.bounds_min.x < -419_000.0,
        "shared ocean room missing from boot bounds {:?}",
        area.bounds_min
    );
    assert_eq!(app.snapshot().tick, 0);

    for _ in 0..5 {
        app.advance(tick);
    }
    assert_eq!(app.snapshot().tick, 5);

    {
        let mut query = app
            .sim
            .world
            .query::<&mut clearsleeper::sim::player::Player>();
        let mut player = query.iter_mut(&mut app.sim.world).next().expect("player");
        player.pos = Vec3::new(-100_000.0, 0.0, 300_000.0);
    }
    app.advance(tick);

    assert_eq!(app.snapshot().tick, 0, "animation phase restarts on reload");
    assert_eq!(app.room.as_ref().expect("island room mesh").groups.len(), 9);
    let collision = app
        .sim
        .world
        .resource::<clearsleeper::sim::collision::Collision>();
    let ground = collision
        .ground_at(-100_000.0, 300_000.0)
        .expect("ground in cell 45");
    assert!(
        (ground + 5_000.0).abs() < 1.0,
        "cell 45 collision not loaded, ground {ground}"
    );

    app.advance(tick);
    assert_eq!(app.snapshot().tick, 1, "set must not reload while staying");
}

fn inside_xz(a: [f32; 3], b: [f32; 3], c: [f32; 3], point: [f32; 2]) -> bool {
    let v0 = [b[0] - a[0], b[2] - a[2]];
    let v1 = [c[0] - a[0], c[2] - a[2]];
    let v2 = [point[0] - a[0], point[1] - a[1]];
    let determinant = v0[0] * v1[1] - v1[0] * v0[1];
    if determinant.abs() < 1e-6 {
        return false;
    }
    let u = (v2[0] * v1[1] - v1[0] * v2[1]) / determinant;
    let w = (v0[0] * v2[1] - v2[0] * v0[1]) / determinant;
    u >= -1e-4 && w >= -1e-4 && u + w <= 1.0 + 1e-4
}

#[test]
fn golden_dzr_spawn_selects_entry_by_point() {
    let mut data = Vec::new();
    data.extend_from_slice(&2_u32.to_be_bytes());
    data.extend_from_slice(b"PLYR");
    data.extend_from_slice(&2_u32.to_be_bytes());
    data.extend_from_slice(&16_u32.to_be_bytes());
    data.extend_from_slice(b"OOOO");
    data.extend_from_slice(&0_u32.to_be_bytes());
    data.extend_from_slice(&64_u32.to_be_bytes());

    push_spawn_entry(&mut data, 100.0, 200.0, 300.0, 0, 0);
    push_spawn_entry(&mut data, -194_845.4, 145.0, 314_242.8, 4187, -128);

    let (spawn, yaw) = stage_spawn(&data, "golden.dzr".into()).unwrap();
    assert_eq!(spawn, Vec3::new(-194_845.4, 145.0, 314_242.8));
    let expected_yaw = 4187.0 * std::f32::consts::TAU / 65_536.0;
    assert!((yaw - expected_yaw).abs() < 1e-6);

    let mut no_plyr = Vec::new();
    no_plyr.extend_from_slice(&0_u32.to_be_bytes());
    assert!(
        matches!(
            stage_spawn(&no_plyr, "golden.dzr".into()),
            Err(AssetError::Corrupt { .. })
        ),
        "expected corrupt without spawn chunk"
    );

    let mut wrong_point = Vec::new();
    wrong_point.extend_from_slice(&1_u32.to_be_bytes());
    wrong_point.extend_from_slice(b"PLYR");
    wrong_point.extend_from_slice(&1_u32.to_be_bytes());
    wrong_point.extend_from_slice(&16_u32.to_be_bytes());
    push_spawn_entry(&mut wrong_point, 1.0, 2.0, 3.0, 7, 0);
    assert!(
        matches!(
            stage_spawn(&wrong_point, "golden.dzr".into()),
            Err(AssetError::Corrupt { detail, .. })
                if detail == "dzr spawn point missing"
        ),
        "expected missing spawn point"
    );
}

fn push_spawn_entry(
    out: &mut Vec<u8>,
    x: f32,
    y: f32,
    z: f32,
    angle_y: i16,
    angle_z: i16,
) {
    out.extend_from_slice(b"Link\0\0\0\0");
    out.extend_from_slice(&0xffff_002c_u32.to_be_bytes());
    out.extend_from_slice(&x.to_be_bytes());
    out.extend_from_slice(&y.to_be_bytes());
    out.extend_from_slice(&z.to_be_bytes());
    out.extend_from_slice(&0_i16.to_be_bytes());
    out.extend_from_slice(&angle_y.to_be_bytes());
    out.extend_from_slice(&angle_z.to_be_bytes());
    out.extend_from_slice(&0xffff_u16.to_be_bytes());
}

fn synthetic_archive() -> Vec<u8> {
    fn push_u32(out: &mut Vec<u8>, value: u32) {
        out.extend_from_slice(&value.to_be_bytes());
    }
    fn push_u16(out: &mut Vec<u8>, value: u16) {
        out.extend_from_slice(&value.to_be_bytes());
    }
    fn hash(name: &str) -> u16 {
        name.bytes().fold(0_u32, |h, b| (h * 3 + u32::from(b)) & 0xFFFF) as u16
    }
    fn entry(
        out: &mut Vec<u8>,
        id: u16,
        name: &str,
        kind: u8,
        name_off: u16,
        offset: u32,
        size: u32,
    ) {
        push_u16(out, id);
        push_u16(out, hash(name));
        out.push(kind);
        out.push(0);
        push_u16(out, name_off);
        push_u32(out, offset);
        push_u32(out, size);
        push_u32(out, 0);
    }

    let mut out = Vec::new();
    out.extend_from_slice(b"RARC");
    push_u32(&mut out, 0x120);
    push_u32(&mut out, 0x20);
    push_u32(&mut out, 0xF2);
    push_u32(&mut out, 14);
    push_u32(&mut out, 14);
    push_u32(&mut out, 0);
    push_u32(&mut out, 0);
    push_u32(&mut out, 2);
    push_u32(&mut out, 0x20);
    push_u32(&mut out, 7);
    push_u32(&mut out, 0x40);
    push_u32(&mut out, 38);
    push_u32(&mut out, 0xCC);
    push_u32(&mut out, 0x0007_0100);
    push_u32(&mut out, 0);

    out.extend_from_slice(b"ROOT");
    push_u32(&mut out, 30);
    push_u16(&mut out, hash("testarc"));
    push_u16(&mut out, 4);
    push_u32(&mut out, 0);
    out.extend_from_slice(b"SUB ");
    push_u32(&mut out, 15);
    push_u16(&mut out, hash("sub"));
    push_u16(&mut out, 3);
    push_u32(&mut out, 4);

    entry(&mut out, 0xFFFF, ".", 0x02, 0, 0, 16);
    entry(&mut out, 0xFFFF, "..", 0x02, 2, 0xFFFF_FFFF, 16);
    entry(&mut out, 0, "hello.bin", 0x11, 5, 0, 6);
    entry(&mut out, 0xFFFF, "sub", 0x02, 15, 1, 16);
    entry(&mut out, 0xFFFF, ".", 0x02, 0, 1, 16);
    entry(&mut out, 0xFFFF, "..", 0x02, 2, 0, 16);
    entry(&mut out, 1, "nested.bin", 0x11, 19, 6, 8);

    out.extend_from_slice(b".\0..\0hello.bin\0sub\0nested.bin\0testarc\0");
    out.extend_from_slice(b"hello!NESTED!!");
    out
}
