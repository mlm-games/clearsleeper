use clearsleeper::bdl::{BdlError, parse};

struct Synthetic {
    bytes: Vec<u8>,
    matrix_type_at: usize,
    draw_flag_at: usize,
}

fn block(tag: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let size = 8 + payload.len();
    let mut out = Vec::with_capacity(size);
    out.extend_from_slice(tag);
    out.extend((size as u32).to_be_bytes());
    out.extend_from_slice(payload);
    out
}

fn synthetic() -> Synthetic {
    let mut hierarchy = Vec::new();
    for (kind, value) in [(0x10_u16, 0_u16), (1, 0), (0x10, 1), (2, 0), (0, 0)] {
        hierarchy.extend(kind.to_be_bytes());
        hierarchy.extend(value.to_be_bytes());
    }
    let mut inf1 = vec![0_u8; 0x10];
    inf1[0x0C..0x10].copy_from_slice(&0x18_u32.to_be_bytes());
    inf1.extend(hierarchy);

    let mut jnt1 = vec![0_u8; 0x10];
    jnt1[0..2].copy_from_slice(&2_u16.to_be_bytes());
    jnt1[4..8].copy_from_slice(&0x18_u32.to_be_bytes());
    jnt1[8..12].copy_from_slice(&0x98_u32.to_be_bytes());
    for translation in [[0.0_f32, 0.0, 0.0], [0.0, 0.0, 5.0]] {
        let mut init = vec![0_u8; 0x40];
        for (axis, value) in [4_usize, 8, 12].into_iter().zip([1.0_f32; 3]) {
            init[axis..axis + 4].copy_from_slice(&value.to_be_bytes());
        }
        for (axis, value) in [0x18_usize, 0x1C, 0x20].into_iter().zip(translation) {
            init[axis..axis + 4].copy_from_slice(&value.to_be_bytes());
        }
        jnt1.extend(init);
    }
    jnt1.extend(0_u16.to_be_bytes());
    jnt1.extend(1_u16.to_be_bytes());

    let mut drw1 = vec![0_u8; 0x0C];
    drw1[0..2].copy_from_slice(&1_u16.to_be_bytes());
    drw1[4..8].copy_from_slice(&0x14_u32.to_be_bytes());
    drw1[8..12].copy_from_slice(&0x15_u32.to_be_bytes());
    drw1.push(0);
    drw1.extend(1_u16.to_be_bytes());

    let mut vtx1 = vec![0_u8; 0x38];
    vtx1[0..4].copy_from_slice(&0x40_u32.to_be_bytes());
    vtx1[4..8].copy_from_slice(&0x70_u32.to_be_bytes());
    vtx1[0x10..0x14].copy_from_slice(&0xA0_u32.to_be_bytes());
    for (attribute, components, kind) in [(9_u32, 1_u32, 4_u32), (11, 1, 5), (255, 0, 0)] {
        vtx1.extend(attribute.to_be_bytes());
        vtx1.extend(components.to_be_bytes());
        vtx1.extend(kind.to_be_bytes());
        vtx1.extend([0_u8; 4]);
    }
    for position in [
        [0.0_f32, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
    ] {
        for axis in position {
            vtx1.extend(axis.to_be_bytes());
        }
    }
    vtx1.extend([255_u8, 0, 0, 255]);

    let mut shp1 = vec![0_u8; 0x24];
    shp1[0..2].copy_from_slice(&1_u16.to_be_bytes());
    let relations = [0x30_u32, 0x2C, 0, 0x58, 0, 0x88, 0x78, 0x80];
    for (index, relation) in relations.into_iter().enumerate() {
        shp1[4 + index * 4..8 + index * 4].copy_from_slice(&relation.to_be_bytes());
    }
    shp1.extend(0_u16.to_be_bytes());
    shp1.extend([0_u8; 2]);
    let mut init = vec![0_u8; 0x28];
    init[2..4].copy_from_slice(&1_u16.to_be_bytes());
    shp1.extend(init);
    for (attribute, kind) in [(9_u32, 3_u32), (11, 3), (13, 3), (255, 0)] {
        shp1.extend(attribute.to_be_bytes());
        shp1.extend(kind.to_be_bytes());
    }
    shp1.extend(0_u16.to_be_bytes());
    shp1.extend(1_u16.to_be_bytes());
    shp1.extend(0_u32.to_be_bytes());
    shp1.extend(27_u32.to_be_bytes());
    shp1.extend(0_u32.to_be_bytes());
    shp1.push(0x98);
    shp1.extend(4_u16.to_be_bytes());
    for position in [0_u16, 1, 3, 2] {
        shp1.extend(position.to_be_bytes());
        shp1.extend(0_u16.to_be_bytes());
        shp1.extend(0_u16.to_be_bytes());
    }

    let blocks = [
        block(b"INF1", &inf1),
        block(b"JNT1", &jnt1),
        block(b"DRW1", &drw1),
        block(b"VTX1", &vtx1),
        block(b"SHP1", &shp1),
    ];
    let mut matrix_type_at = 0_usize;
    let mut draw_flag_at = 0_usize;
    let mut bytes = Vec::new();
    bytes.extend(b"J3D2bdl4");
    bytes.extend([0_u8; 4]);
    bytes.extend((blocks.len() as u32).to_be_bytes());
    bytes.resize(0x20, 0);
    for (index, block) in blocks.iter().enumerate() {
        if index == 2 {
            draw_flag_at = bytes.len() + 0x14;
        }
        if index == 4 {
            matrix_type_at = bytes.len() + 0x30;
        }
        bytes.extend(block);
    }
    let size = bytes.len() as u32;
    bytes[8..12].copy_from_slice(&size.to_be_bytes());
    Synthetic {
        bytes,
        matrix_type_at,
        draw_flag_at,
    }
}

#[test]
fn golden_strip_winding_joint_chain_and_colors() {
    let synthetic = synthetic();
    let mesh = parse(&synthetic.bytes, [0.0, 0.0], 0.0).expect("synthetic bdl parses");
    assert_eq!(
        mesh.positions,
        vec![
            [0.0, 0.0, 5.0],
            [1.0, 0.0, 5.0],
            [0.0, 0.0, 6.0],
            [1.0, 0.0, 6.0],
        ]
    );
    assert_eq!(mesh.indices, vec![0, 2, 1, 2, 3, 1]);
    assert_eq!(mesh.colors, vec![[1.0, 0.0, 0.0]; 4]);
}

#[test]
fn rejects_truncated_and_unsupported_models() {
    let mut synthetic = synthetic();
    assert!(parse(&synthetic.bytes, [0.0, 0.0], 0.0).is_ok());

    let mut magic = synthetic.bytes.clone();
    magic[0] = b'X';
    assert_eq!(
        parse(&magic, [0.0, 0.0], 0.0).unwrap_err(),
        BdlError::Magic
    );

    for cut in 0..synthetic.bytes.len() {
        assert!(
            parse(&synthetic.bytes[..cut], [0.0, 0.0], 0.0).is_err(),
            "cut {cut}"
        );
    }

    let flag = synthetic.bytes[synthetic.draw_flag_at];
    synthetic.bytes[synthetic.draw_flag_at] = 1;
    assert_eq!(
        parse(&synthetic.bytes, [0.0, 0.0], 0.0).unwrap_err(),
        BdlError::Envelope
    );
    synthetic.bytes[synthetic.draw_flag_at] = flag;

    let matrix_type = synthetic.bytes[synthetic.matrix_type_at];
    synthetic.bytes[synthetic.matrix_type_at] = 1;
    assert_eq!(
        parse(&synthetic.bytes, [0.0, 0.0], 0.0).unwrap_err(),
        BdlError::Shape
    );
    synthetic.bytes[synthetic.matrix_type_at] = matrix_type;
}
