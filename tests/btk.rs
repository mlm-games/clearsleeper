use clearsleeper::j3d::btk::{self, BtkError, TexSrt, UvMatrix};

fn put_u16(data: &mut [u8], at: usize, value: u16) {
    data[at..at + 2].copy_from_slice(&value.to_be_bytes());
}

fn put_u32(data: &mut [u8], at: usize, value: u32) {
    data[at..at + 4].copy_from_slice(&value.to_be_bytes());
}

fn put_i16(data: &mut [u8], at: usize, value: i16) {
    data[at..at + 2].copy_from_slice(&value.to_be_bytes());
}

fn put_f32(data: &mut [u8], at: usize, value: f32) {
    data[at..at + 4].copy_from_slice(&value.to_be_bytes());
}

fn synthetic() -> Vec<u8> {
    let mut data = vec![0_u8; 0x10E];
    data[0..8].copy_from_slice(b"J3D1btk1");
    put_u32(&mut data, 8, 0x10E);
    put_u32(&mut data, 12, 1);

    let block = 0x20;
    data[block..block + 4].copy_from_slice(b"TTK1");
    put_u32(&mut data, block + 4, 0xEE);
    data[block + 0x09] = 1;
    put_i16(&mut data, block + 0x0A, 10);
    put_u16(&mut data, block + 0x0C, 3);
    put_u16(&mut data, block + 0x0E, 2);
    put_u16(&mut data, block + 0x10, 8);
    put_u16(&mut data, block + 0x12, 10);
    put_u32(&mut data, block + 0x14, 0x60);
    put_u32(&mut data, block + 0x1C, 0x96);
    put_u32(&mut data, block + 0x20, 0);
    put_u32(&mut data, block + 0x24, 0xA2);
    put_u32(&mut data, block + 0x28, 0xAE);
    put_u32(&mut data, block + 0x2C, 0xB6);
    put_u32(&mut data, block + 0x30, 0xC6);

    let keys: [[u16; 3]; 9] = [
        [1, 0, 0],
        [0, 0, 0],
        [3, 1, 0],
        [1, 1, 0],
        [0, 0, 0],
        [0, 0, 0],
        [0, 0, 0],
        [2, 0, 1],
        [0, 0, 0],
    ];
    for (index, key) in keys.iter().enumerate() {
        let at = block + 0x60 + index * 6;
        put_u16(&mut data, at, key[0]);
        put_u16(&mut data, at + 2, key[1]);
        put_u16(&mut data, at + 4, key[2]);
    }

    let name_at = block + 0x96;
    put_u16(&mut data, name_at, 1);
    put_u16(&mut data, name_at + 2, 0xFFFF);
    put_u16(&mut data, name_at + 4, 0);
    put_u16(&mut data, name_at + 6, 8);
    data[name_at + 8..name_at + 12].copy_from_slice(b"wat\0");

    let center = block + 0xA2;
    for (index, value) in [0.25, 0.5, 0.0].iter().enumerate() {
        put_f32(&mut data, center + index * 4, *value);
    }
    let scale = block + 0xAE;
    for (index, value) in [1.5, 0.75].iter().enumerate() {
        put_f32(&mut data, scale + index * 4, *value);
    }
    let rotation = block + 0xB6;
    for (index, value) in [0, 0, 0, 0, 10, 8192, 0, 0].iter().enumerate() {
        put_i16(&mut data, rotation + index * 2, *value);
    }
    let translation = block + 0xC6;
    for (index, value) in [0.0, 0.0, 0.0, 0.0, 5.0, 0.5, 0.0, 10.0, 1.0, 0.0]
        .iter()
        .enumerate()
    {
        put_f32(&mut data, translation + index * 4, *value);
    }
    data
}

#[test]
fn golden_texture_animation_evaluates_channels() {
    let animation = btk::parse(&synthetic()).expect("synthetic btk parses");

    assert_eq!(animation.frame_max, 10);
    assert_eq!(animation.dec_shift, 1);
    assert_eq!(animation.track_count(), 1);
    assert_eq!(animation.track("wat", 0), Some(0));
    assert_eq!(animation.track("wat", 5), None);
    assert_eq!(animation.track("no", 0), None);
    assert_eq!(animation.center(0), Some([0.25, 0.5, 0.0]));
    assert_eq!(animation.frame_at(0), 0.0);
    assert_eq!(animation.frame_at(10), 0.0);
    assert_eq!(animation.frame_at(25), 5.0);

    assert_eq!(
        animation.eval(0, 0.0),
        TexSrt {
            scale_x: 1.5,
            scale_y: 0.75,
            rotation: 0,
            translation_x: 0.0,
            translation_y: 0.0,
        }
    );
    assert_eq!(
        animation.eval(0, 5.0),
        TexSrt {
            scale_x: 1.5,
            scale_y: 0.75,
            rotation: 8192,
            translation_x: 0.5,
            translation_y: 0.0,
        }
    );
    assert_eq!(
        animation.eval(0, 10.0),
        TexSrt {
            scale_x: 1.5,
            scale_y: 0.75,
            rotation: 16384,
            translation_x: 1.0,
            translation_y: 0.0,
        }
    );
    assert_eq!(
        animation.uv_transform(0, 0.0),
        Some(UvMatrix {
            a: 1.5,
            b: -0.0,
            c: 0.0,
            d: 0.75,
            tx: -0.125,
            ty: 0.125,
        })
    );
}

#[test]
fn rejects_truncated_and_unsupported_streams() {
    let bytes = synthetic();

    let mut bad_magic = bytes.clone();
    bad_magic[0] = b'X';
    assert_eq!(btk::parse(&bad_magic).unwrap_err(), BtkError::Magic);

    for cut in 0..bytes.len() {
        assert!(btk::parse(&bytes[..cut]).is_err(), "cut at {cut:#x}");
    }

    let mut oversized = bytes.clone();
    put_u32(&mut oversized, 8, 0x200);
    assert_eq!(btk::parse(&oversized).unwrap_err(), BtkError::Truncated);

    let mut post_tracks = bytes.clone();
    put_u16(&mut post_tracks, 0x20 + 0x34, 1);
    assert_eq!(btk::parse(&post_tracks).unwrap_err(), BtkError::Descriptor);

    let mut calc_type = bytes.clone();
    put_u32(&mut calc_type, 0x20 + 0x5C, 1);
    assert_eq!(btk::parse(&calc_type).unwrap_err(), BtkError::Descriptor);
}
