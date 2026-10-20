use std::collections::HashMap;

use clearsleeper::bti::Image;
use clearsleeper::j2d::{Bounds, Kind, Screen, ScreenPane, Screens};
use clearsleeper::meter_hud::{Meter, MeterAssets};
use clearsleeper::sim::health::Health;
use clearsleeper::text::raster::Canvas;

fn pane(name: &str, x: f32, y: f32, white: u32, alpha: u8) -> ScreenPane {
    ScreenPane {
        name: name.to_string(),
        kind: Kind::Picture,
        texture: Some("heart.bti".to_string()),
        bounds: Bounds {
            x,
            y,
            w: 8.0,
            h: 8.0,
        },
        color_white: white,
        alpha,
        ..Default::default()
    }
}

fn solid() -> Image {
    Image {
        width: 2,
        height: 2,
        pixels: vec![255; 16],
        mag_linear: false,
        wrap_s: 0,
        wrap_t: 0,
        alpha_enabled: true,
    }
}

fn fixture() -> MeterAssets {
    let mut panes = Vec::new();
    for index in 0..20 {
        let x = 16.0 + index as f32 * 20.0;
        panes.push(pane(&format!("ht{index:02}"), x, 10.0, 0xFFFF_FFFF, 0xFF));
        panes.push(pane(&format!("hk{index:02}"), x, 12.0, 0x0000_00FF, 0x64));
    }
    let mut screens = Screens::default();
    screens.cache.push((
        "main_parts1.blo".to_string(),
        Screen {
            width: 640,
            height: 480,
            color: 0,
            panes,
        },
    ));
    let mut textures = HashMap::new();
    for name in [
        "heart.bti",
        "heart_01.bti",
        "heart_02.bti",
        "heart_03.bti",
        "heart_04.bti",
    ] {
        textures.insert(name.to_string(), solid());
    }
    MeterAssets::from_parts(screens, textures)
}

fn settle(meter: &mut Meter, health: Health, assets: &MeterAssets) {
    for _ in 0..8 {
        meter.update(health, assets);
    }
}

fn rect_has_alpha(canvas: &Canvas, bounds: [f32; 4]) -> bool {
    let x0 = bounds[0].max(0.0) as usize;
    let y0 = bounds[1].max(0.0) as usize;
    let x1 = ((bounds[0] + bounds[2]) as usize).min(canvas.width);
    let y1 = ((bounds[1] + bounds[3]) as usize).min(canvas.height);
    (y0..y1).any(|y| (x0..x1).any(|x| canvas.pixels[(y * canvas.width + x) * 4 + 3] > 0))
}

#[test]
fn full_health_draws_the_retail_full_heart_texture() {
    let assets = fixture();
    let mut meter = Meter::default();
    let health = Health { units: 16, max: 16 };
    settle(&mut meter, health, &assets);

    let hearts: Vec<_> = meter.visible_hearts().collect();
    assert_eq!(hearts.len(), 8, "four hearts and their shadows");
    assert!(
        hearts
            .iter()
            .all(|(texture, _, _, _)| texture == "heart.bti"),
        "a full heart is the retail `heart.bti` pane (`d_meter.cpp:1073-1079`)"
    );
    assert!(
        hearts
            .iter()
            .filter(|(_, _, tint, _)| tint[3] == 255 && tint[..3] == [0, 0, 0])
            .count()
            == 4,
        "the shadows keep the black white-color their panes carry"
    );

    let mut canvas = Canvas::new(640, 480);
    meter.render(&mut canvas, &assets);
    assert!(rect_has_alpha(&canvas, [16.0, 10.0, 8.0, 8.0]));
    assert!(!rect_has_alpha(
        &canvas,
        [16.0 + 4.0 * 20.0, 10.0, 8.0, 8.0]
    ));
}

#[test]
fn partial_health_uses_the_quarter_texture() {
    let assets = fixture();
    let mut meter = Meter::default();
    let health = Health { units: 10, max: 12 };
    settle(&mut meter, health, &assets);

    let textures: Vec<String> = meter
        .visible_hearts()
        .map(|(texture, _, _, _)| texture)
        .collect();
    assert_eq!(
        textures.iter().filter(|name| *name == "heart.bti").count(),
        4,
        "two full hearts and their shadows"
    );
    assert_eq!(
        textures
            .iter()
            .filter(|name| *name == "heart_02.bti")
            .count(),
        2,
        "the half heart is `heart_02.bti` (`d_meter.cpp:1281-1282`)"
    );
}

#[test]
fn zero_health_draws_the_empty_containers() {
    let assets = fixture();
    let mut meter = Meter::default();
    let health = Health { units: 0, max: 12 };
    settle(&mut meter, health, &assets);

    let textures: Vec<String> = meter
        .visible_hearts()
        .map(|(texture, _, _, _)| texture)
        .collect();
    assert_eq!(
        textures
            .iter()
            .filter(|name| *name == "heart_04.bti")
            .count(),
        6,
        "three empty containers and their shadows (`d_meter.cpp:1275-1280`)"
    );
}
