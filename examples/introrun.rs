use clearsleeper::assets;
use clearsleeper::intro::{self, Intro};
use clearsleeper::j2d;
use clearsleeper::text::driver;
use clearsleeper::text::font::FontBank;
use clearsleeper::text::raster::Canvas;

fn main() {
    let Ok(root) = std::env::var("CS_ASSETS") else {
        eprintln!("CS_ASSETS unset");
        return;
    };
    let root = assets::AssetRoot::at(root);
    let archives = assets::MsgArchives::load(&root);
    let Ok(arc) = root.read_archive("files/res/Object/Opening.arc") else {
        eprintln!("no Opening.arc");
        return;
    };
    let blo = arc.read("opening.blo").unwrap_or_default();
    let screen = j2d::parse(&blo).expect("blo");
    let mut fonts = FontBank::new();
    driver::load_fonts(&mut fonts);
    let mut intro = Intro::default();
    intro.start(&mut fonts, &archives);

    let mut states = std::collections::BTreeSet::new();
    let mut messages = std::collections::BTreeSet::new();
    let mut max_state = 0u8;
    let mut frames = 0u32;
    let mut text_frames = 0usize;
    while !intro.done && frames < 12000 {
        intro.step(&mut fonts, &archives);
        states.insert(intro.state);
        messages.insert(intro.text.msg_no);
        max_state = max_state.max(intro.state);
        let mut canvas = Canvas::new(640, 480);
        clearsleeper::intro::render(
            &mut canvas,
            &screen,
            &intro,
            &mut fonts,
            &|_c, _n, _b, _a| {},
        );
        if canvas.pixels.chunks_exact(4).any(|p| p[3] > 0) {
            text_frames += 1;
        }
        frames += 1;
    }
    println!(
        "intro completed: {} after {frames} frames ({:.1}s at 30fps)",
        intro.done,
        frames as f32 / 30.0
    );
    println!("max state reached: {max_state}");
    println!("distinct states visited: {}", states.len());
    println!("messages shown: {:#x?}", messages);
    println!("frames with pixels: {text_frames}/{frames}");
    let elapsed = frames as f32 / 30.0;
    println!("sequence length {elapsed:.1}s");
}
