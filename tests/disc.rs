//! The in-memory disc image: the boot header and the file table, against an image built here.
//!
//! A real dump is not in the repository, so the table is laid out the way the disc stores one
//! (see `src/disc.rs`) and the walk that resolves the asset paths is checked through it. Its
//! root is the data root itself, the way the real GZLE01 table's is (`res`, `thpdemo`, ...),
//! and the extraction's `files/` and `sys/` roots are not entries in it.

use std::sync::Arc;

use clearsleeper::archive::AssetError;
use clearsleeper::assets::AssetRoot;
use clearsleeper::disc::{DiscError, DiscImage};

/// The image twin of [`clearsleeper::assets::ASSETS_ENV`]: the real dump the one disc-backed
/// check reads.
const DISC_ENV: &str = "CS_DISC";

const GAME_MAGIC_AT: usize = 0x1C;
const FST_AT: usize = 0x1000;
const ROOM_AT: usize = 0x2000;
const THP_AT: usize = 0x2400;
const ROOM: &[u8] = b"room data";
const THP: &[u8] = b"movie data";

const NAMES: &[u8] = b"\0res\0Stage\0sea\0Room44.arc\0thpdemo\0demo.thp\0";

fn entry(name_at: u32, kind: u8, offset: u32, size: u32) -> [u8; 12] {
    let mut bytes = [0_u8; 12];
    bytes[..4].copy_from_slice(&((u32::from(kind) << 24) | name_at).to_be_bytes());
    bytes[4..8].copy_from_slice(&offset.to_be_bytes());
    bytes[8..12].copy_from_slice(&size.to_be_bytes());
    bytes
}

/// The depth first table of the real GZLE01 shape, miniature: the root is the data root, so its
/// children are `res` and `thpdemo` and there is no `files` node. A directory's third word is the
/// index one past its subtree: root [1, 7), res [2, 5), Stage [3, 5), sea [4, 5), thpdemo [6, 7).
fn table() -> Vec<u8> {
    let mut fst = Vec::new();
    fst.extend_from_slice(&entry(0, 1, 0, 7));
    fst.extend_from_slice(&entry(1, 1, 0, 5));
    fst.extend_from_slice(&entry(5, 1, 1, 5));
    fst.extend_from_slice(&entry(11, 1, 2, 5));
    fst.extend_from_slice(&entry(15, 0, ROOM_AT as u32, ROOM.len() as u32));
    fst.extend_from_slice(&entry(26, 1, 0, 7));
    fst.extend_from_slice(&entry(34, 0, THP_AT as u32, THP.len() as u32));
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
    bytes[THP_AT..THP_AT + THP.len()].copy_from_slice(THP);
    bytes
}

#[test]
fn image_reads_the_paths_an_extracted_tree_holds() {
    let image = DiscImage::open(image(b"GZLE01")).unwrap();
    assert_eq!(image.game_id(), "GZLE01");
    assert_eq!(image.read("files/res/Stage/sea/Room44.arc").unwrap(), ROOM);
    assert_eq!(image.read("files/thpdemo/demo.thp").unwrap(), THP);
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
fn the_extraction_path_is_translated_onto_the_fst_root() {
    let image = DiscImage::open(image(b"GZLE01")).unwrap();

    // An extraction keeps the disc's data root under `files/`, and the table's root is that data
    // root, so the extracted path and the table's own both name the one entry.
    assert_eq!(
        image.read("files/res/Stage/sea/Room44.arc").unwrap(),
        image.read("res/Stage/sea/Room44.arc").unwrap()
    );
    assert_eq!(image.read("res/Stage/sea/Room44.arc").unwrap(), ROOM);

    // Exactly the leading component translates: a second `files` names no entry, and nothing
    // tries the path the other way around either.
    assert!(matches!(
        image.read("files/files/res/Stage/sea/Room44.arc"),
        Err(DiscError::MissingFile { .. })
    ));
    assert!(matches!(
        image.read("files"),
        Err(DiscError::MissingFile { .. })
    ));
}

#[test]
fn the_preamble_is_not_in_the_file_table() {
    let image = DiscImage::open(image(b"GZLE01")).unwrap();

    // `sys/` is where an extraction keeps the disc's boot files. The table describes no such
    // entry and nothing in the port reads one, so the miss is stated rather than papered over.
    match image.read("sys/boot.bin") {
        Err(DiscError::MissingFile { path }) => assert_eq!(path, "sys/boot.bin"),
        other => panic!("sys/boot.bin is not a table entry: {other:?}"),
    }

    let root = AssetRoot::from_disc(Arc::new(image));
    match root.read("sys/boot.bin") {
        Err(AssetError::MissingFile { path }) => {
            assert_eq!(path, std::path::PathBuf::from("GZLE01:sys/boot.bin"))
        }
        other => panic!("sys/boot.bin keeps its extraction label: {other:?}"),
    }
}

/// The one check that needs a real dump: `CS_DISC` names it, and the path the whole port reads at
/// boot is asked for out of it. Skipped when unset, the way the `CS_ASSETS` checks skip - CI has
/// no image, so the end-to-end acceptance is the user dropping theirs on the deployed page.
#[test]
fn real_disc_image_answers_the_stage_archive_through_cs_disc() {
    let Ok(path) = std::env::var(DISC_ENV) else {
        eprintln!("{DISC_ENV} is not set; skipping real disc image check");
        return;
    };
    let bytes = std::fs::read(&path).expect("the image CS_DISC names is readable");
    let image = DiscImage::open(bytes).expect("the image opens");
    let stage = image
        .read("files/res/Stage/sea/Stage.arc")
        .expect("the sea stage archive resolves through the extraction path");
    assert!(
        stage.starts_with(b"RARC") || stage.starts_with(b"Yaz0"),
        "the sea stage archive is an arc container, packed or plain: {:?}",
        &stage[..stage.len().min(4)]
    );
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
        .and_then(|value| value.parse::<u64>().ok())
        .expect("a --max-memory link argument");
    assert_eq!(ceiling, WASM_MEMORY_CEILING);
    assert!((MAX_IMAGE_BYTES as u64) < ceiling);

    // A full GameCube dump, the largest image the port can be handed.
    assert!(image_len_ok(1_459_978_240).is_ok());
    assert!(matches!(
        image_len_ok(MAX_IMAGE_BYTES + 1),
        Err(DiscError::TooLarge { .. })
    ));
}

/// The corpus the real image holds: every file its table names, read and decoded.
///
/// `CS_DISC` names the image, so this runs where an image is; CI has none, and the end-to-end
/// acceptance is the user dropping theirs on the deployed page.
#[test]
fn the_real_image_decodes_every_archive_it_holds() {
    let Ok(path) = std::env::var(DISC_ENV) else {
        eprintln!("{DISC_ENV} is not set; skipping the corpus walk");
        return;
    };
    let bytes = std::fs::read(&path).expect("the image CS_DISC names is readable");
    let image = DiscImage::open(bytes).expect("the image opens");
    let paths = image.paths();
    assert!(
        paths.len() > 1000,
        "the table holds the disc's files: {}",
        paths.len()
    );

    let mut yaz0 = 0_usize;
    let mut archives = 0_usize;
    let mut entries = 0_usize;
    for rel in &paths {
        let bytes = image
            .read(rel)
            .unwrap_or_else(|error| panic!("{rel} reads: {error}"));
        let plain = if bytes.starts_with(b"Yaz0") {
            yaz0 += 1;
            let size = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
            let plain = clearsleeper::archive::yaz0_decode(&bytes, rel)
                .unwrap_or_else(|error| panic!("{rel} decodes: {error}"));
            assert_eq!(
                plain.len(),
                size,
                "{rel} decodes to the size its header names"
            );
            plain
        } else {
            bytes
        };
        if plain.starts_with(b"RARC") {
            archives += 1;
            let archive = clearsleeper::archive::Archive::parse(plain, rel)
                .unwrap_or_else(|error| panic!("{rel} parses: {error}"));
            for entry in archive.entries() {
                archive
                    .read(entry)
                    .unwrap_or_else(|error| panic!("{rel}:{entry} reads: {error}"));
                entries += 1;
            }
        }
    }
    eprintln!(
        "corpus: {} files, {yaz0} yaz0, {archives} archives, {entries} entries",
        paths.len()
    );
    assert!(
        yaz0 > 100 && archives > 100 && entries > 1000,
        "the corpus really was walked"
    );
}

/// The image against the extraction: every file the table names is the same bytes the extraction
/// holds, and the table's walk is the extraction's layout.
#[test]
fn the_real_image_answers_the_whole_extracted_corpus() {
    let Ok(path) = std::env::var(DISC_ENV) else {
        eprintln!("{DISC_ENV} is not set; skipping the corpus comparison");
        return;
    };
    let Ok(root) = std::env::var(clearsleeper::assets::ASSETS_ENV) else {
        eprintln!(
            "{} is not set; skipping the corpus comparison",
            clearsleeper::assets::ASSETS_ENV
        );
        return;
    };
    let bytes = std::fs::read(&path).expect("the image CS_DISC names is readable");
    let image = DiscImage::open(bytes).expect("the image opens");
    let dump = AssetRoot::at(&root);

    let mut compared = 0_usize;
    for rel in image.paths() {
        let from_image = image
            .read(&rel)
            .unwrap_or_else(|error| panic!("{rel} reads: {error}"));
        let from_dump = dump
            .read(&format!("files/{rel}"))
            .unwrap_or_else(|error| panic!("files/{rel} reads: {error}"));
        assert_eq!(
            from_image, from_dump,
            "{rel} is one file in the image and in the extraction"
        );
        compared += 1;
    }
    eprintln!("compared {compared} files against the extraction");
    assert!(compared > 1000, "the corpus really was compared");
}
