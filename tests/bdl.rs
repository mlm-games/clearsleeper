use clearsleeper::bdl::{BdlError, parse};

struct Synthetic {
    bytes: Vec<u8>,
    matrix_type_at: usize,
    draw_flag_at: usize,
    texture_format_at: usize,
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
    for (kind, value) in [
        (0x10_u16, 0_u16),
        (1, 0),
        (0x10, 1),
        (2, 0),
        (0x11, 0),
        (0x12, 0),
        (0, 0),
    ] {
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
    vtx1[4..8].copy_from_slice(&0x80_u32.to_be_bytes());
    vtx1[0x10..0x14].copy_from_slice(&0xB0_u32.to_be_bytes());
    vtx1[0x18..0x1C].copy_from_slice(&0xB4_u32.to_be_bytes());
    for (attribute, components, kind, frac) in [
        (9_u32, 1_u32, 4_u32, 0_u8),
        (11, 1, 5, 0),
        (13, 1, 3, 8),
        (255, 0, 0, 0),
    ] {
        vtx1.extend(attribute.to_be_bytes());
        vtx1.extend(components.to_be_bytes());
        vtx1.extend(kind.to_be_bytes());
        vtx1.extend([frac, 0, 0, 0]);
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
    for uv in [[0_i16, 0], [256, 0], [256, 256], [0, 256]] {
        for value in uv {
            vtx1.extend(value.to_be_bytes());
        }
    }

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
    for (position, texcoord) in [(0_u16, 0_u16), (1, 1), (3, 2), (2, 3)] {
        shp1.extend(position.to_be_bytes());
        shp1.extend(0_u16.to_be_bytes());
        shp1.extend(texcoord.to_be_bytes());
    }

    let mut mat3 = vec![0_u8; 0x19C];
    mat3[0..2].copy_from_slice(&1_u16.to_be_bytes());
    mat3[4..8].copy_from_slice(&0x58_u32.to_be_bytes());
    mat3[8..12].copy_from_slice(&0x50_u32.to_be_bytes());
    mat3[0x40..0x44].copy_from_slice(&0x54_u32.to_be_bytes());
    mat3[0xD4..0xD6].copy_from_slice(&0_u16.to_be_bytes());

    let mut tex1 = vec![0_u8; 0x58];
    tex1[0..2].copy_from_slice(&1_u16.to_be_bytes());
    tex1[4..8].copy_from_slice(&0x20_u32.to_be_bytes());
    tex1[0x18] = 0;
    tex1[0x1A..0x1C].copy_from_slice(&8_u16.to_be_bytes());
    tex1[0x1C..0x1E].copy_from_slice(&8_u16.to_be_bytes());
    tex1[0x1E] = 1;
    tex1[0x1F] = 1;
    tex1[0x34..0x38].copy_from_slice(&0x20_u32.to_be_bytes());
    for _ in 0..32 {
        tex1.push(0xF0);
    }

    let blocks = [
        block(b"INF1", &inf1),
        block(b"JNT1", &jnt1),
        block(b"DRW1", &drw1),
        block(b"VTX1", &vtx1),
        block(b"SHP1", &shp1),
        block(b"MAT3", &mat3),
        block(b"TEX1", &tex1),
    ];
    let mut matrix_type_at = 0_usize;
    let mut draw_flag_at = 0_usize;
    let mut texture_format_at = 0_usize;
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
        if index == 6 {
            texture_format_at = bytes.len() + 0x20;
        }
        bytes.extend(block);
    }
    let size = bytes.len() as u32;
    bytes[8..12].copy_from_slice(&size.to_be_bytes());
    Synthetic {
        bytes,
        matrix_type_at,
        draw_flag_at,
        texture_format_at,
    }
}

#[test]
fn golden_strip_winding_joint_chain_and_colors() {
    let synthetic = synthetic();
    let mesh = parse(&synthetic.bytes, [0.0, 0.0], 0.0, None).expect("synthetic bdl parses");
    assert_eq!(mesh.groups.len(), 1);
    assert_eq!(mesh.textures.len(), 1);
    let group = &mesh.groups[0];
    assert_eq!(
        group.positions,
        vec![
            [0.0, 0.0, 5.0],
            [1.0, 0.0, 5.0],
            [0.0, 0.0, 6.0],
            [1.0, 0.0, 6.0],
        ]
    );
    assert_eq!(group.indices, vec![0, 2, 1, 2, 3, 1]);
    assert_eq!(group.colors, vec![[1.0, 0.0, 0.0]; 4]);
    assert_eq!(group.texture_page, Some(0));
    assert_eq!(
        group.uvs,
        vec![
            [0.0, 0.0],
            [0.015625, 0.0],
            [0.015625, 0.015625],
            [0.0, 0.015625],
        ]
    );
    let texture = &mesh.textures[0];
    assert_eq!((texture.width, texture.height), (9, 9));
    assert_eq!(mesh.uv_placements, vec![[1.0 / 64.0, 1.0 / 64.0, 0.0, 0.0]]);
    assert!(mesh.animations.is_empty());
    assert!(group.anim.is_none());
    assert_eq!(texture.rgba.len(), 9 * 9 * 4);
    assert_eq!(&texture.rgba[0..4], &[255, 255, 255, 255]);
    assert_eq!(&texture.rgba[4..8], &[0, 0, 0, 0]);
    assert_eq!(&texture.rgba[8..12], &[255, 255, 255, 255]);
    assert_eq!(&texture.rgba[32..36], &[255, 255, 255, 255]);
    assert_eq!(&texture.rgba[288..292], &[255, 255, 255, 255]);
}

#[test]
fn rejects_truncated_and_unsupported_models() {
    let mut synthetic = synthetic();
    assert!(parse(&synthetic.bytes, [0.0, 0.0], 0.0, None).is_ok());

    let mut magic = synthetic.bytes.clone();
    magic[0] = b'X';
    assert_eq!(
        parse(&magic, [0.0, 0.0], 0.0, None).unwrap_err(),
        BdlError::Magic
    );

    for cut in 0..synthetic.bytes.len() {
        assert!(
            parse(&synthetic.bytes[..cut], [0.0, 0.0], 0.0, None).is_err(),
            "cut {cut}"
        );
    }

    let flag = synthetic.bytes[synthetic.draw_flag_at];
    synthetic.bytes[synthetic.draw_flag_at] = 1;
    assert_eq!(
        parse(&synthetic.bytes, [0.0, 0.0], 0.0, None).unwrap_err(),
        BdlError::Envelope
    );
    synthetic.bytes[synthetic.draw_flag_at] = flag;

    let matrix_type = synthetic.bytes[synthetic.matrix_type_at];
    synthetic.bytes[synthetic.matrix_type_at] = 1;
    assert_eq!(
        parse(&synthetic.bytes, [0.0, 0.0], 0.0, None).unwrap_err(),
        BdlError::Shape
    );
    synthetic.bytes[synthetic.matrix_type_at] = matrix_type;

    let texture_format = synthetic.bytes[synthetic.texture_format_at];
    synthetic.bytes[synthetic.texture_format_at] = 6;
    assert_eq!(
        parse(&synthetic.bytes, [0.0, 0.0], 0.0, None).unwrap_err(),
        BdlError::Texture
    );
    synthetic.bytes[synthetic.texture_format_at] = texture_format;
}
