//! The in-memory disc image: the boot header and the file table, against an image built here.
//!
//! A real dump is not in the repository, so the table is laid out the way the disc stores one
//! (see `src/disc.rs`) and the walk that resolves the asset paths is checked through it.

use std::sync::Arc;

use clearsleeper::archive::AssetError;
use clearsleeper::assets::AssetRoot;
use clearsleeper::disc::{DiscError, DiscImage};

const GAME_MAGIC_AT: usize = 0x1C;
const FST_AT: usize = 0x1000;
const ROOM_AT: usize = 0x2000;
const SYS_AT: usize = 0x2100;
const ROOM: &[u8] = b"room data";
const SYS: &[u8] = b"sys data!";

const NAMES: &[u8] = b"\0files\0res\0Stage\0sea\0Room44.arc\0sys\0boot.bin\0";

fn entry(name_at: u32, kind: u8, offset: u32, size: u32) -> [u8; 12] {
    let mut bytes = [0_u8; 12];
    bytes[..4].copy_from_slice(&((u32::from(kind) << 24) | name_at).to_be_bytes());
    bytes[4..8].copy_from_slice(&offset.to_be_bytes());
    bytes[8..12].copy_from_slice(&size.to_be_bytes());
    bytes
}

/// The depth first table of the tree `files/res/Stage/sea/Room44.arc` plus `sys/boot.bin`,
/// where a directory's third word is the index one past its subtree:
/// root [1, 8), files [2, 6), res [3, 6), Stage [4, 6), sea [5, 6), sys [7, 8).
fn table() -> Vec<u8> {
    let mut fst = Vec::new();
    fst.extend_from_slice(&entry(0, 1, 0, 8));
    fst.extend_from_slice(&entry(1, 1, 0, 6));
    fst.extend_from_slice(&entry(7, 1, 1, 6));
    fst.extend_from_slice(&entry(11, 1, 2, 6));
    fst.extend_from_slice(&entry(17, 1, 3, 6));
    fst.extend_from_slice(&entry(21, 0, ROOM_AT as u32, ROOM.len() as u32));
    fst.extend_from_slice(&entry(32, 1, 0, 8));
    fst.extend_from_slice(&entry(36, 0, SYS_AT as u32, SYS.len() as u32));
    fst.extend_from_slice(NAMES);
    fst
}

fn image(game_id: &[u8; 6]) -> Vec<u8> {
    let mut bytes = vec![0_u8; 0x3000];
    bytes[..6].copy_from_slice(game_id);
    bytes[0x20..0x20 + 16].copy_from_slice(b"Synthetic Disc\0\0");
    bytes[GAME_MAGIC_AT..GAME_MAGIC_AT + 4].copy_from_slice(&[0xC2, 0x33, 0x9F, 0x3D]);
    let fst = table();
    bytes[0x424..0x428].copy_from_slice(&(FST_AT as u32).to_be_bytes());
    bytes[0x428..0x42C].copy_from_slice(&(fst.len() as u32).to_be_bytes());
    bytes[FST_AT..FST_AT + fst.len()].copy_from_slice(&fst);
    bytes[ROOM_AT..ROOM_AT + ROOM.len()].copy_from_slice(ROOM);
    bytes[SYS_AT..SYS_AT + SYS.len()].copy_from_slice(SYS);
    bytes
}

#[test]
fn image_reads_the_paths_an_extracted_tree_holds() {
    let image = DiscImage::open(image(b"GZLE01")).unwrap();
    assert_eq!(image.game_id(), "GZLE01");
    assert_eq!(image.read("files/res/Stage/sea/Room44.arc").unwrap(), ROOM);
    assert_eq!(image.read("sys/boot.bin").unwrap(), SYS);
    assert_eq!(image.read("/files/res/Stage/sea/Room44.arc").unwrap(), ROOM);

    let summary = image.summary();
    assert_eq!(summary.game_id, "GZLE01");
    assert_eq!(summary.title.as_deref(), Some("Synthetic Disc"));
    assert_eq!(summary.files, 2);

    assert!(matches!(
        image.read("files/res/Stage/sea"),
        Err(DiscError::MissingFile { .. })
    ));
    assert!(matches!(
        image.read("files/res/Stage/wind/Room44.arc"),
        Err(DiscError::MissingFile { .. })
    ));
    assert!(matches!(
        image.read("Files/res/Stage/sea/Room44.arc"),
        Err(DiscError::MissingFile { .. })
    ));
}

#[test]
fn the_asset_root_answers_image_paths_the_way_it_answers_a_directory() {
    let root = AssetRoot::from_disc(Arc::new(DiscImage::open(image(b"GZLP01")).unwrap()));
    assert_eq!(root.fs_root(), None);
    assert_eq!(
        root.path("files/res/Stage/sea/Room44.arc"),
        std::path::PathBuf::from("GZLP01:files/res/Stage/sea/Room44.arc")
    );
    assert_eq!(root.read("files/res/Stage/sea/Room44.arc").unwrap(), ROOM);
    assert!(matches!(
        root.read("files/res/Stage/sea/absent.arc"),
        Err(AssetError::MissingFile { .. })
    ));
}

#[test]
fn unsupported_images_are_typed_failures() {
    let mut wii = image(b"RZDP01");
    wii[0x18..0x1C].copy_from_slice(&[0x5D, 0x1C, 0x9E, 0xA3]);
    assert!(matches!(
        DiscImage::open(wii),
        Err(DiscError::WiiDisc { .. })
    ));

    assert!(matches!(
        DiscImage::open(image(b"GZ2E01")),
        Err(DiscError::NotWindWaker { .. })
    ));

    let mut ciso = image(b"GZLE01");
    ciso[..4].copy_from_slice(b"CISO");
    assert!(matches!(
        DiscImage::open(ciso),
        Err(DiscError::UnsupportedContainer { format: "CISO" })
    ));

    let mut rvz = vec![0_u8; 0x100];
    rvz[..4].copy_from_slice(b"RVZ\x02");
    assert!(matches!(
        DiscImage::open(rvz),
        Err(DiscError::RvzVersion { version: 2 })
    ));

    assert!(matches!(
        DiscImage::open(vec![0_u8; 0x10]),
        Err(DiscError::Truncated { .. })
    ));

    let mut corrupt = image(b"GZLE01");
    corrupt[FST_AT + 8..FST_AT + 12].copy_from_slice(&0_u32.to_be_bytes());
    assert!(matches!(
        DiscImage::open(corrupt),
        Err(DiscError::BadFst { at: 8, .. })
    ));
}

#[test]
fn the_image_limit_leaves_room_inside_the_wasm_memory_ceiling() {
    use clearsleeper::disc::{MAX_IMAGE_BYTES, WASM_MEMORY_CEILING, image_len_ok};

    let config =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/.cargo/config.toml"))
            .expect("the wasm link flags");
    let ceiling = config
        .split("--max-memory=")
        .nth(1)
        .and_then(|rest| rest.split(|c: char| c == '"' || c.is_whitespace()).next())
        .and_then(|value| value.parse::<usize>().ok())
        .expect("a --max-memory link argument");
    assert_eq!(ceiling, WASM_MEMORY_CEILING);
    assert!(MAX_IMAGE_BYTES < ceiling);

    // A full GameCube dump, the largest image the port can be handed.
    assert!(image_len_ok(1_459_978_240).is_ok());
    assert!(matches!(
        image_len_ok(MAX_IMAGE_BYTES + 1),
        Err(DiscError::TooLarge { .. })
    ));
}
