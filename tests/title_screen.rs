use clearsleeper::assets::{ASSETS_ENV, AssetRoot};
use clearsleeper::j2d::Bounds;
use clearsleeper::sim::events::SimEvents;
use clearsleeper::sim::title::{
    self, PANE_NINT, PANE_PRES, PANE_ZELD, PANE_ZELJ, Title, TitleKirakira,
};
use clearsleeper::text::raster::Canvas;
use clearsleeper::title_screen::{self, TitleAssets};

fn alpha(title: &Title, pane: usize) -> u8 {
    title.pane[pane].alpha
}

fn tick(title: &mut Title, kirakira: &mut TitleKirakira, events: &mut SimEvents) {
    title.proc_execute(false, kirakira, events);
}

#[test]
fn title_panes_follow_the_decomp_alpha_schedule() {
    let mut title = Title::spawn(0);
    let mut kirakira = TitleKirakira::default();
    let mut events = SimEvents::default();

    for _ in 0..title::START_DELAY {
        tick(&mut title, &mut kirakira, &mut events);
        assert_eq!(alpha(&title, PANE_ZELD), 0, "the start delay draws nothing");
        assert_eq!(alpha(&title, PANE_NINT), 0, "the start delay draws nothing");
        assert_eq!(alpha(&title, PANE_PRES), 0, "the start delay draws nothing");
    }

    let mut frame = 0;
    while frame < title::PRES_RAMP {
        frame += 1;
        tick(&mut title, &mut kirakira, &mut events);
        match frame {
            title::ZELD_HOLD => {
                assert_eq!(alpha(&title, PANE_ZELD), 0, "zeld holds until {frame}");
            }
            31 => {
                assert_eq!(
                    alpha(&title, PANE_ZELD),
                    (255.0 * (1.0 / title::ZELD_SPAN)) as u8,
                    "zeld starts its {} frame ramp",
                    title::ZELD_SPAN
                );
            }
            55 => {
                assert_eq!(
                    alpha(&title, PANE_ZELD),
                    (255.0 * (25.0 / title::ZELD_SPAN)) as u8,
                    "zeld is half way up"
                );
            }
            title::ZELD_RAMP => {
                assert_eq!(alpha(&title, PANE_ZELD), 255, "zeld is fully drawn");
            }
            title::NINT_HOLD => {
                assert_eq!(alpha(&title, PANE_NINT), 0, "nint holds until {frame}");
            }
            151 => {
                assert_eq!(
                    alpha(&title, PANE_NINT),
                    (255.0 * (1.0 / title::NINT_SPAN)) as u8,
                    "nint starts its {} frame ramp",
                    title::NINT_SPAN
                );
            }
            title::NINT_RAMP => {
                assert_eq!(alpha(&title, PANE_NINT), 255, "nint is fully drawn");
            }
            title::PRES_HOLD => {
                assert_eq!(alpha(&title, PANE_PRES), 0, "pres holds until {frame}");
            }
            161 => {
                assert_eq!(
                    alpha(&title, PANE_PRES),
                    (255.0 * (1.0 / title::PRES_SPAN)) as u8,
                    "pres starts its {} frame ramp",
                    title::PRES_SPAN
                );
            }
            _ => {}
        }
        assert_eq!(
            alpha(&title, PANE_ZELJ),
            0,
            "zelj stays hidden before the enter"
        );
    }
    assert_eq!(alpha(&title, PANE_PRES), 255, "pres is fully drawn");
}

#[test]
fn title_pres_blinks_on_the_decomp_period() {
    let mut title = Title::spawn(0);
    let mut kirakira = TitleKirakira::default();
    let mut events = SimEvents::default();
    for _ in 0..title::START_DELAY {
        tick(&mut title, &mut kirakira, &mut events);
    }
    tick(&mut title, &mut kirakira, &mut events);
    title.proc_execute(true, &mut kirakira, &mut events);
    assert!(title.enter_mode() >= 1, "the press enters the title");

    let mut values = Vec::new();
    for _ in 0..=(title::PRES_PERIOD as usize + 1) {
        tick(&mut title, &mut kirakira, &mut events);
        values.push(alpha(&title, PANE_PRES));
    }
    assert_eq!(
        values[0],
        (255.0 * ((title::PRES_HALF - 1) as f32 / title::PRES_SPAN_HALF)) as u8,
        "the blink starts one step into its period"
    );
    assert_eq!(
        values[title::PRES_HALF as usize - 1],
        0,
        "the blink reaches zero when m028 hits PRES_HALF"
    );
    assert_eq!(
        values[title::PRES_HALF as usize],
        (255.0 * (1.0 / title::PRES_SPAN_HALF)) as u8,
        "the blink turns around after PRES_HALF"
    );
    assert_eq!(
        values[title::PRES_PERIOD as usize - 1],
        255,
        "the blink is at full alpha at PRES_PERIOD"
    );
    assert_eq!(
        values[title::PRES_PERIOD as usize + 1],
        values[0],
        "the blink repeats one PRES_PERIOD later"
    );
    assert_eq!(alpha(&title, PANE_ZELD), 255, "zeld is held while blinking");
    assert_eq!(alpha(&title, PANE_NINT), 255, "nint is held while blinking");
}

#[test]
fn real_disc_title_screen_draws_its_panes() {
    let Ok(root) = std::env::var(ASSETS_ENV) else {
        eprintln!("CS_ASSETS is not set; skipping real-disc title screen check");
        return;
    };
    let root = AssetRoot::at(root);
    let assets = TitleAssets::load(&root).expect("TlogoE holds the title screen");

    let mut title = Title::spawn(0);
    for pane in title.pane.iter_mut() {
        pane.alpha = 255;
    }
    title.pane[PANE_NINT].alpha = 0;

    let mut canvas = Canvas::new(640, 480);
    title_screen::render(&mut canvas, &assets, &title, None);

    let press = title_screen::pane_bounds(&assets, PANE_PRES).expect("the blo has pres");
    let zeld = title_screen::pane_bounds(&assets, PANE_ZELD).expect("the blo has zeld");
    let nint = title_screen::pane_bounds(&assets, PANE_NINT).expect("the blo has nint");
    assert!(
        lit(&canvas, press) > 0,
        "press_start draws inside its layout rect"
    );
    assert!(
        lit(&canvas, zeld) > 0,
        "logo_zelda_main draws inside its layout rect"
    );
    assert_eq!(lit(&canvas, nint), 0, "a zero alpha pane draws nothing");
}

fn lit(canvas: &Canvas, bounds: Bounds) -> usize {
    let x0 = bounds.x.max(0.0) as usize;
    let y0 = bounds.y.max(0.0) as usize;
    let x1 = (bounds.right().min(canvas.width as f32)) as usize;
    let y1 = (bounds.bottom().min(canvas.height as f32)) as usize;
    let mut count = 0;
    for y in y0..y1 {
        for x in x0..x1 {
            let at = (y * canvas.width + x) * 4;
            if canvas.pixels[at..at + 3]
                .iter()
                .any(|channel| *channel != 0)
            {
                count += 1;
            }
        }
    }
    count
}
