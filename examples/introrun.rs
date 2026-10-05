use clearsleeper::assets;
use clearsleeper::intro::{self, Intro};
use clearsleeper::j2d;
use clearsleeper::text::font::FontBank;
use clearsleeper::text::raster::Canvas;
use clearsleeper::text::{driver, MsgData};

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

    let mut canvas = Canvas::new(640, 480);
    let no_images = |_c: &mut Canvas, _n: &str, _b: &[f32; 4], _a: f32| {};

    let mut lit_frames = 0usize;
    let mut text_seen = String::new();
    for frame in 0..3000u32 {
        intro.step(&mut fonts, &archives);
        canvas = Canvas::new(640, 480);
        intro::render(&mut canvas, &screen, &intro, &mut fonts, &no_images);
        let lit = canvas
            .pixels
            .chunks_exact(4)
            .filter(|p| p[3] > 0)
            .count();
        if lit > 0 {
            lit_frames += 1;
        }
        if text_seen.is_empty() {
            let out = intro.text.data.out[0].clone().unwrap_or_default();
            if out.len() > 20 {
                text_seen = out;
            }
        }
        if frame % 250 == 0 || frame == 2999 {
            println!(
                "frame {frame}: state={} msg={:#x} phase={} lit={lit} text_len={}",
                intro.state,
                intro.text.msg_no,
                intro.text.phase,
                intro.text.data.out[0].as_ref().map_or(0, String::len),
            );
        }
    }
    println!("frames with visible output: {lit_frames}/3000");
    println!("intro done: {}", intro.done);
    println!("first substantial text: {:?}", text_seen.chars().take(60).collect::<String>());
    let _ = MsgData::new_data_proc();
    std::fs::write("/tmp/opencode/intro.rgba", &canvas.pixels).unwrap();
}