//! Golden-image test for the file select: renders through the real production
//! path (`file_select_view` -> `layout_and_paint` -> offscreen WGPU) and
//! compares against a committed golden.
//!
//! Bless with CS_BLESS=1. Skips when CS_ASSETS is unset or no GPU adapter is
//! available.

use clearsleeper::shot;
use image::ImageEncoder;

const CHANNEL_TOL: i16 = 8;
const MAX_DIFF_FRACTION: f64 = 0.01;

fn golden_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/goldens/file_select.png")
}

fn decode(png: &[u8]) -> (u32, u32, Vec<[u8; 4]>) {
    let img = image::load_from_memory(png).expect("decode png").to_rgba8();
    let (w, h) = img.dimensions();
    let pixels = img.pixels().map(|p| p.0).collect();
    (w, h, pixels)
}

#[test]
fn golden_file_select() {
    if std::env::var_os("CS_ASSETS").is_none() {
        eprintln!("CS_ASSETS is not set; skipping file select golden");
        return;
    }
    let rgba = match shot::file_select_rgba(600) {
        Ok(rgba) => rgba,
        Err(e) => {
            eprintln!("skipping file select golden: {e:#}");
            return;
        }
    };

    let mut png_bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png_bytes)
        .write_image(&rgba, shot::W, shot::H, image::ExtendedColorType::Rgba8)
        .expect("encode png");

    let path = golden_path();
    if std::env::var_os("CS_BLESS").is_some() || !path.exists() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &png_bytes).unwrap();
        eprintln!("blessed {}", path.display());
        return;
    }

    let golden = std::fs::read(&path).expect("read golden");
    let (gw, gh, gp) = decode(&golden);
    let (aw, ah, ap) = decode(&png_bytes);
    assert_eq!((gw, gh), (aw, ah), "file select: size changed");

    let total = (gw * gh) as f64;
    let mut differing = 0usize;
    for (g, a) in gp.iter().zip(ap.iter()) {
        if g.iter()
            .zip(a.iter())
            .any(|(&gc, &ac)| (gc as i16 - ac as i16).abs() > CHANNEL_TOL)
        {
            differing += 1;
        }
    }
    let fraction = differing as f64 / total;
    if fraction > MAX_DIFF_FRACTION {
        let actual = path.with_file_name("file_select.actual.png");
        std::fs::write(&actual, &png_bytes).unwrap();
        panic!(
            "file select: {differing} px ({:.2}%) differ beyond tol (limit {:.2}%). \
             Actual written to {}. Bless with CS_BLESS=1 if intentional.",
            fraction * 100.0,
            MAX_DIFF_FRACTION * 100.0,
            actual.display()
        );
    }
}
