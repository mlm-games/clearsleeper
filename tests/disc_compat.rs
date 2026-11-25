use std::io::Write;

use clearsleeper::assets::{AssetRoot, install_extracted};
use clearsleeper::disc::{DiscError, DiscImage};

fn word(value: u32) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

/// A minimal GameCube image: one root directory and one file in its FST.
fn wind_waker_image() -> Vec<u8> {
    let mut bytes = vec![0u8; 0x2000];
    bytes[..6].copy_from_slice(b"GZLE01");
    bytes[0x1C..0x20].copy_from_slice(&[0xC2, 0x33, 0x9F, 0x3D]);

    let fst_at = 0x100;
    let names = b"Test.txt\0";
    let mut fst = Vec::new();
    fst.extend(word(1 << 24)); // root directory, name offset 0
    fst.extend(word(0)); // parent
    fst.extend(word(2)); // one past the subtree: the entry count
    fst.extend(word(0)); // file, name offset 0
    fst.extend(word(0x1000)); // file offset in the image
    fst.extend(word(18)); // file size
    fst.extend_from_slice(names);

    bytes[fst_at..fst_at + fst.len()].copy_from_slice(&fst);
    bytes[0x424..0x428].copy_from_slice(&word(fst_at as u32));
    bytes[0x428..0x42C].copy_from_slice(&word(fst.len() as u32));
    bytes[0x1000..0x1012].copy_from_slice(b"CLEAR SLEEPER TEST");
    bytes
}

#[test]
fn extracted_tree_reads_an_asset() {
    let root = std::env::temp_dir().join(format!("cs-extract-{}", std::process::id()));
    let files = root.join("files");
    std::fs::create_dir_all(&files).expect("create extracted tree");
    let mut file = std::fs::File::create(files.join("Test.txt")).expect("create asset");
    file.write_all(b"CLEAR SLEEPER TEST").expect("write asset");
    drop(file);

    install_extracted(&root);
    let asset = AssetRoot::from_env().expect("extracted tree is a source");
    assert_eq!(
        asset
            .read("files/Test.txt")
            .expect("read asset from extracted tree"),
        b"CLEAR SLEEPER TEST"
    );
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn raw_game_cube_image_reads_an_asset() {
    let image = DiscImage::open(wind_waker_image()).expect("open a minimal GameCube image");
    assert_eq!(image.game_id(), "GZLE01");
    assert_eq!(
        image
            .read("files/Test.txt")
            .expect("read asset from the image"),
        b"CLEAR SLEEPER TEST"
    );
}

#[test]
fn wii_and_nfs_headers_go_through_nod() {
    let mut wii = wind_waker_image();
    wii[..6].copy_from_slice(b"RZDE01");
    wii[0x18..0x1C].copy_from_slice(&[0x5D, 0x1C, 0x9E, 0xA3]);
    // A stub Wii header must reach `nod`, not be refused by the GameCube magic check.
    match DiscImage::open(wii) {
        Ok(_) => {}
        Err(error) => {
            println!("wii error: {error:?}");
            assert!(
                !matches!(error, DiscError::BadMagic { .. }),
                "the Wii header was read as a GameCube image"
            );
        }
    }

    let mut nfs = wind_waker_image();
    nfs[..4].copy_from_slice(b"EGGS");
    match DiscImage::open(nfs) {
        Ok(_) => {}
        Err(error) => {
            println!("nfs error: {error:?}");
            assert!(
                !matches!(error, DiscError::BadMagic { .. }),
                "the NFS magic was read as a GameCube image"
            );
        }
    }
}
