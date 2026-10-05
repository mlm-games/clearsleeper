use clearsleeper::text::font::{FontBank, ResFont};
use clearsleeper::text::msgbox::TextPrint;
use clearsleeper::text::raster::Canvas;

const WHITE: u32 = 0xFFFF_FFFF;

fn main() {
    let root = std::env::var("CS_ASSETS").expect("CS_ASSETS");
    let bytes = std::fs::read(format!("{root}/files/res/Menu/kanfont_fix16.bfn")).expect("font");
    let mut bank = FontBank::new();
    bank.insert(ResFont::from_bytes(&bytes));

    let encoded = [0x81u8, 0x41, 0x81, 0x42, 0x81, 0x40, 0x81, 0x43];
    let mut print = TextPrint::default();
    print.private_initiate(&bank, Some(0), 0.0, 0.0, WHITE, WHITE, 0, WHITE, false);
    print.set_font_size(32.0, 32.0);
    print.locate(0.0, 0.0);
    let layout = print.print_return(&mut bank, &encoded, 660.0, 32.0, 0, 0, 0.0, 0.0, 0xFF);
    println!("quads {}", layout.quads.len());
    for run in &layout.quads {
        println!(
            "code 0x{:04x} x={:.0} uv={:?} u0={:.1} v0={:.1}",
            run.code,
            run.quad.pos[0][0],
            run.quad.uv[0],
            run.quad.uv[0][0] as f32 / 32768.0 * 512.0,
            run.quad.uv[0][1] as f32 / 32768.0 * 512.0,
        );
    }

    let mut canvas = Canvas::new(700, 60);
    canvas.draw_layout(&layout, bank.get(0).unwrap(), [0.0, 8.0], 0xFF);
    for y in 0..56 {
        let row: String = (0..700)
            .map(|x| {
                let a = canvas.pixels[(y * 700 + x) * 4 + 3];
                if a > 200 {
                    '#'
                } else if a > 40 {
                    '+'
                } else {
                    ' '
                }
            })
            .collect();
        let trimmed = row.trim_end();
        if !trimmed.is_empty() {
            println!("{trimmed}");
        }
    }
}