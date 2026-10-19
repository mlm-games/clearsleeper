//! The opening's picture panes: the decoded BTI reaches the canvas, in the pane
//! rect and at the pane's opacity. The real-disc check walks the state machine
//! and probes each demo pane once its texture is fully faded in; it skips when
//! `CS_ASSETS` is unset, like the other real-disc checks.

use clearsleeper::assets::{ASSETS_ENV, AssetRoot, MsgArchives};
use clearsleeper::intro::{self, Intro, PaneImages};
use clearsleeper::j2d::{self, Bounds, Kind, Screen, ScreenPane};
use clearsleeper::text::driver;
use clearsleeper::text::font::FontBank;
use clearsleeper::text::raster::Canvas;

const DEMO_PANES: [(&str, usize); 7] = [
    ("d1", 0),
    ("d2", 1),
    ("d3", 2),
    ("d4", 3),
    ("d42", 4),
    ("d5", 5),
    ("d6", 6),
];

/// I8 with a flat texel: `src/bti.rs` reads the format at 0, size at 0x02/0x04
/// and the image at 0x1C, and an I8 pixel is a grey of that intensity.
fn bti_i8(width: u16, height: u16, texel: u8) -> Vec<u8> {
    let mut bytes = vec![0_u8; 0x20 + usize::from(width) * usize::from(height)];
    bytes[0] = 0x01;
    bytes[2..4].copy_from_slice(&width.to_be_bytes());
    bytes[4..6].copy_from_slice(&height.to_be_bytes());
    bytes[0x1C..0x20].copy_from_slice(&0x20_u32.to_be_bytes());
    bytes[0x20..].fill(texel);
    bytes
}

fn picture(name: &str, texture: &str, x: f32) -> ScreenPane {
    ScreenPane {
        name: name.to_string(),
        kind: Kind::Picture,
        bounds: Bounds {
            x,
            y: 0.0,
            w: 16.0,
            h: 16.0,
        },
        texture: Some(texture.to_string()),
        pic_binding: 0x0F,
        ..ScreenPane::default()
    }
}

fn alpha(canvas: &Canvas, x: usize, y: usize) -> u8 {
    canvas.pixels[(y * canvas.width + x) * 4 + 3]
}

fn lit(canvas: &Canvas, bounds: [f32; 4]) -> usize {
    let x0 = bounds[0].max(0.0) as usize;
    let x1 = ((bounds[0] + bounds[2]).min(canvas.width as f32) as usize).min(canvas.width);
    let y0 = (bounds[1] + bounds[3] * 0.3) as usize;
    let y1 = ((bounds[1] + bounds[3] * 0.5) as usize).min(canvas.height);
    let mut count = 0;
    for y in y0..y1 {
        for x in x0..x1 {
            count += usize::from(alpha(canvas, x, y) > 0);
        }
    }
    count
}

#[test]
fn decoded_texture_reaches_its_pane_and_a_zero_alpha_pane_stays_empty() {
    let texture = bti_i8(8, 8, 0x7F);
    let mut images = PaneImages::default();
    assert!(
        images.add("probe.bti", &texture),
        "the synthetic BTI decodes"
    );
    let image = images.get("probe.bti").expect("the entry is keyed by name");
    assert_eq!((image.width, image.height), (8, 8));

    let screen = Screen {
        width: 640,
        height: 480,
        color: 0,
        panes: vec![
            picture("d1", "probe.bti", 0.0),
            picture("d2", "probe.bti", 32.0),
        ],
    };
    let mut intro = Intro::default();
    intro.panes[0].alpha = 1.0;
    intro.panes[1].alpha = 0.0;

    let mut canvas = Canvas::new(64, 32);
    intro::render(
        &mut canvas,
        &screen,
        &intro,
        &mut FontBank::new(),
        &|canvas, name, bounds, alpha| {
            intro::draw_pane(canvas, &screen, &images, name, bounds, alpha);
        },
    );

    assert_eq!(alpha(&canvas, 8, 8), 255, "d1's texture is drawn");
    assert_eq!(
        &canvas.pixels[(8 * canvas.width + 8) * 4..][..3],
        &[0x7F, 0x7F, 0x7F],
        "the decoded I8 texel, untinted"
    );
    for y in 0..16 {
        for x in 32..48 {
            assert_eq!(
                alpha(&canvas, x, y),
                0,
                "d2 is transparent at ({x}, {y}) with pane alpha 0"
            );
        }
    }
}

#[test]
fn real_disc_opening_demo_panes_draw_their_textures() {
    let Ok(root) = std::env::var(ASSETS_ENV) else {
        eprintln!("CS_ASSETS is not set; skipping real-disc opening image check");
        return;
    };
    let root = AssetRoot::at(root);
    let archive = root
        .read_archive("files/res/Object/Opening.arc")
        .expect("Opening.arc is in the extracted tree");
    let screen = j2d::parse(
        archive
            .read("opening.blo")
            .expect("opening.blo is in Opening.arc"),
    )
    .expect("opening.blo parses");
    let images = PaneImages::load(&archive);

    let textures = [
        ("demo_1.bti", 660, 612),
        ("demo_2.bti", 660, 308),
        ("demo_3.bti", 660, 308),
        ("demo_4.bti", 660, 308),
        ("demo_4_2.bti", 492, 308),
        ("demo_5.bti", 660, 308),
        ("demo_6.bti", 660, 308),
        ("black.bti", 8, 8),
    ];
    for (name, width, height) in textures {
        let image = images
            .get(name)
            .unwrap_or_else(|| panic!("{name} decodes from Opening.arc"));
        assert_eq!(
            (image.width, image.height),
            (width, height),
            "{name}'s stored size"
        );
    }

    let archives = MsgArchives::load(&root);
    let mut fonts = FontBank::new();
    driver::load_fonts(&mut fonts);
    let mut intro = Intro::default();
    intro.start(&mut fonts, &archives);

    let mut seen = [false; DEMO_PANES.len()];
    let mut frames = 0;
    while !intro.done && frames < 12_000 && seen.iter().any(|checked| !checked) {
        intro.step(&mut fonts, &archives);
        frames += 1;
        for (slot, (name, index)) in DEMO_PANES.iter().enumerate() {
            if seen[slot] || intro.panes[*index].alpha < 1.0 {
                continue;
            }
            let Some(bounds) = intro::pane_bounds(&screen, &intro, name) else {
                continue;
            };
            let left = bounds[0].max(0.0);
            let right = (bounds[0] + bounds[2]).min(640.0);
            if right - left < 100.0 {
                continue;
            }
            let mut canvas = Canvas::new(640, 480);
            intro::render(
                &mut canvas,
                &screen,
                &intro,
                &mut fonts,
                &|canvas, name, bounds, alpha| {
                    intro::draw_pane(canvas, &screen, &images, name, bounds, alpha);
                },
            );
            let pixels = lit(&canvas, bounds);
            assert!(
                pixels > 1_000,
                "{name} draws its texture into the pane ({pixels} lit pixels at frame {frames})"
            );
            seen[slot] = true;
        }
    }
    assert!(
        seen.iter().all(|checked| *checked),
        "every demo pane became visible: {seen:?} after {frames} frames (done {})",
        intro.done
    );
}
