use std::io::Write as _;

use clearsleeper::assets::{AssetError, AssetRoot, Archive, yaz0_decode};

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
