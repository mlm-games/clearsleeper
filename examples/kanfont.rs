use clearsleeper::text::font::{FontBank, ResFont};

fn main() {
    let root = std::env::var("CS_ASSETS").expect("CS_ASSETS");
    let bytes = std::fs::read(format!("{root}/files/res/Menu/kanfont_fix16.bfn")).expect("font");
    let mut bank = FontBank::new();
    bank.insert(ResFont::from_bytes(&bytes));
    let font = bank.get_mut(0).unwrap();
    let code = font.font_code(0x8141);
    font.load_image(code as u16);
    let font = bank.get(0).unwrap();
    println!("font_code(0x8141)={code} cell=({},{})", font.cell_x, font.cell_y);
    println!(
        "page={}B data_at={} data_size={} tex={}x{}",
        font.tex_page().len(),
        font.tex.data_at,
        font.tex.data_size,
        font.tex.width,
        font.tex.height
    );
    let page = font.tex_page().to_vec();
    let (w, h) = (
        usize::from(font.glyph.texture_width),
        usize::from(font.glyph.texture_height),
    );
    const TILE: usize = 8;
    let mut out = vec![0_u8; w * h];
    let mut tile = 0_usize;
    for by in 0..h / TILE {
        for bx in 0..w / TILE {
            for row in 0..TILE {
                let at = tile * 32 + row * (TILE / 2);
                for col in 0..TILE {
                    let b = page.get(at + col / 2).copied().unwrap_or(0);
                    out[(by * TILE + row) * w + bx * TILE + col] =
                        if col % 2 == 0 { b >> 4 } else { b & 0x0F };
                }
            }
            tile += 1;
        }
    }
    let path = std::env::temp_dir().join("rust_page.raw");
    std::fs::write(&path, &out).unwrap();
    println!("wrote {} texels ink {}", out.len(), out.iter().filter(|v| **v > 0).count());
}