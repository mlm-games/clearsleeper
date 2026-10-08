use clearsleeper::sim::collision::{Collision, DzbError, NO_EXIT};
use glam::{Vec2, Vec3};

fn dzb(verts: &[[f32; 3]], tris: &[[u16; 3]]) -> Vec<u8> {
    let vertex_count = verts.len() as u32;
    let tri_count = tris.len() as u32;
    let vertex_offset = 0x34_u32;
    let tri_offset = vertex_offset + vertex_count * 12;
    let end = tri_offset + tri_count * 10;
    let mut out = Vec::new();
    for value in [
        vertex_count,
        vertex_offset,
        tri_count,
        tri_offset,
        0,
        end,
        0,
        end,
        0,
        end,
        0,
        end,
        0,
    ] {
        out.extend_from_slice(&value.to_be_bytes());
    }
    for vert in verts {
        for &axis in vert {
            out.extend_from_slice(&axis.to_be_bytes());
        }
    }
    for tri in tris {
        for &id in tri {
            out.extend_from_slice(&id.to_be_bytes());
        }
        out.extend_from_slice(&0_u16.to_be_bytes());
        out.extend_from_slice(&0_u16.to_be_bytes());
    }
    out
}

fn floor_quad(y: f32) -> ([[f32; 3]; 4], [[u16; 3]; 2]) {
    (
        [
            [0.0, y, 0.0],
            [0.0, y, 10.0],
            [10.0, y, 10.0],
            [10.0, y, 0.0],
        ],
        [[0, 1, 2], [0, 2, 3]],
    )
}

fn wall_quad(x: f32, y0: f32, y1: f32) -> ([[f32; 3]; 4], [[u16; 3]; 2]) {
    (
        [
            [x, y0, -10.0],
            [x, y0, 10.0],
            [x, y1, 10.0],
            [x, y1, -10.0],
        ],
        [[0, 1, 2], [0, 2, 3]],
    )
}

fn merge(a: ([[f32; 3]; 4], [[u16; 3]; 2]), b: ([[f32; 3]; 4], [[u16; 3]; 2])) -> (Vec<[f32; 3]>, Vec<[u16; 3]>) {
    let mut verts = a.0.to_vec();
    let mut tris = a.1.to_vec();
    let offset = verts.len() as u16;
    verts.extend_from_slice(&b.0);
    for mut tri in b.1 {
        for id in &mut tri {
            *id += offset;
        }
        tris.push(tri);
    }
    (verts, tris)
}

#[test]
fn golden_ground_heights_take_highest_surface() {
    let (floor_verts, floor_tris) = floor_quad(10.0);
    let (upper_verts, upper_tris) = floor_quad(20.0);
    let verts = floor_verts
        .iter()
        .chain(upper_verts.iter())
        .copied()
        .collect::<Vec<_>>();
    let tris = floor_tris
        .iter()
        .chain(upper_tris.iter())
        .map(|tri| {
            let mut shifted = *tri;
            for id in &mut shifted {
                *id += 4;
            }
            shifted
        })
        .collect::<Vec<_>>();
    let collision = Collision::parse(&dzb(&verts, &tris)).unwrap();
    assert_eq!(collision.ground_at(5.0, 5.0), Some(20.0));
    assert_eq!(collision.ground_at(-5.0, 5.0), None);

    let (only_floor, only_tris) = floor_quad(10.0);
    let collision = Collision::parse(&dzb(&only_floor, &only_tris)).unwrap();
    assert_eq!(collision.ground_at(5.0, 5.0), Some(10.0));
    assert_eq!(collision.ground_at(10.0, 10.0), Some(10.0));
    assert_eq!(collision.ground_at(10.1, 5.0), None);
}

#[test]
fn golden_wall_blocks_and_slides_along_free_axis() {
    let (floor_verts, floor_tris) = floor_quad(0.0);
    let (wall_verts, wall_tris) = wall_quad(5.0, 0.0, 60.0);
    let (verts, tris) = merge(
        (floor_verts, floor_tris),
        (wall_verts, wall_tris),
    );
    let collision = Collision::parse(&dzb(&verts, &tris)).unwrap();

    let to = Vec3::new(10.0, 0.0, 0.0);
    assert_eq!(collision.slide(Vec3::ZERO, to), Vec3::ZERO);
    let slide = collision.slide(Vec3::ZERO, Vec3::new(10.0, 0.0, 5.0));
    assert_eq!(slide, Vec3::new(0.0, 0.0, 5.0));
    let free = collision.slide(Vec3::ZERO, Vec3::new(2.0, 0.0, 5.0));
    assert_eq!(free, Vec3::new(2.0, 0.0, 5.0));
    let airborne = collision.slide(Vec3::new(0.0, 200.0, 0.0), Vec3::new(10.0, 200.0, 0.0));
    assert_eq!(airborne, Vec3::new(10.0, 200.0, 0.0));
    let below = collision.slide(Vec3::new(0.0, -100.0, 0.0), Vec3::new(10.0, -100.0, 0.0));
    assert_eq!(below, Vec3::new(10.0, -100.0, 0.0));
}

#[test]
fn bounds_cover_collision_extents() {
    let (verts, tris) = floor_quad(3.0);
    let collision = Collision::parse(&dzb(&verts, &tris)).unwrap();
    let (min, max) = collision.bounds();
    assert_eq!(min, Vec2::new(0.0, 0.0));
    assert_eq!(max, Vec2::new(10.0, 10.0));
}

#[test]
fn golden_append_merges_geometry_across_collision_tables() {
    let (floor_verts, floor_tris) = floor_quad(10.0);
    let mut collision = Collision::parse(&dzb(&floor_verts, &floor_tris)).unwrap();
    let ledge_verts = [
        [10.0, 15.0, 0.0],
        [10.0, 15.0, 10.0],
        [20.0, 15.0, 10.0],
        [20.0, 15.0, 0.0],
    ];
    let ledge_tris = [[0_u16, 1, 2], [0, 2, 3]];
    collision
        .append(Collision::parse(&dzb(&ledge_verts, &ledge_tris)).unwrap())
        .unwrap();

    assert_eq!(collision.ground_at(5.0, 5.0), Some(10.0));
    assert_eq!(collision.ground_at(15.0, 5.0), Some(15.0));
    assert_eq!(collision.ground_at(21.0, 5.0), None);
    let (min, max) = collision.bounds();
    assert_eq!(min, Vec2::new(0.0, 0.0));
    assert_eq!(max, Vec2::new(20.0, 10.0));
}

#[test]
fn rejects_append_beyond_u16_vertex_capacity() {
    let verts = vec![[0.0_f32, 0.0, 0.0]; 65_536];
    let mut wide = Collision::parse(&dzb(&verts, &[[0, 0, 0]])).unwrap();
    let extra = Collision::parse(&dzb(&[[1.0, 0.0, 0.0]], &[[0, 0, 0]])).unwrap();
    assert_eq!(wide.append(extra).unwrap_err(), DzbError::TooLarge);
}

#[test]
fn rejects_truncated_tables_and_bad_indices() {
    assert_eq!(
        Collision::parse(&[0_u8; 0x30]).unwrap_err(),
        DzbError::Truncated
    );

    let (verts, tris) = floor_quad(0.0);
    let mut bytes = dzb(&verts, &tris);
    bytes[0..4].copy_from_slice(&u32::MAX.to_be_bytes());
    assert_eq!(Collision::parse(&bytes).unwrap_err(), DzbError::BadTable);

    let mut bytes = dzb(&verts, &[[0, 1, 99]]);
    assert_eq!(Collision::parse(&bytes).unwrap_err(), DzbError::VertexIndex);
    bytes[0x20..0x24].copy_from_slice(&u32::MAX.to_be_bytes());
    assert_eq!(Collision::parse(&bytes).unwrap_err(), DzbError::BadTable);
}

fn dzb_exit(verts: &[[f32; 3]], tris: &[[u16; 3]], exits: &[u8]) -> Vec<u8> {
    let mut data = dzb(verts, tris);
    let info_offset = data.len() as u32;
    for &exit in exits {
        data.extend_from_slice(&((exit as u32) << 13).to_be_bytes());
        data.extend_from_slice(&[0_u8; 12]);
    }
    data[0x28..0x2C].copy_from_slice(&(exits.len() as u32).to_be_bytes());
    data[0x2C..0x30].copy_from_slice(&info_offset.to_be_bytes());
    let tri_offset = 0x34 + (verts.len() as u32) * 12;
    for index in 0..tris.len() {
        let at = (tri_offset + index as u32 * 10 + 6) as usize;
        data[at..at + 2].copy_from_slice(&(index as u16).to_be_bytes());
    }
    data
}

#[test]
fn golden_exit_at_reads_the_poly_underfoot() {
    let (verts, tris) = floor_quad(0.0);
    let collision = Collision::parse(&dzb_exit(&verts, &tris, &[7, NO_EXIT])).unwrap();
    assert_eq!(collision.exit_at(1.0, 9.0), Some(7));
    assert_eq!(collision.exit_at(9.0, 1.0), Some(NO_EXIT));
    assert_eq!(collision.exit_at(50.0, 50.0), None, "off the mesh");
}

#[test]
fn golden_append_carries_exit_ids() {
    let (verts, tris) = floor_quad(0.0);
    let mut collision = Collision::parse(&dzb_exit(&verts, &tris, &[2, 2])).unwrap();
    let ledge = [
        [20.0, 10.0, 0.0],
        [20.0, 10.0, 10.0],
        [30.0, 10.0, 10.0],
        [30.0, 10.0, 0.0],
    ];
    collision
        .append(Collision::parse(&dzb_exit(&ledge, &tris, &[4, 4])).unwrap())
        .unwrap();
    assert_eq!(collision.exit_at(1.0, 9.0), Some(2));
    assert_eq!(collision.exit_at(21.0, 9.0), Some(4));
}
