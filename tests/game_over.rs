//! The game over sequence: the `Gover` letters (`d_gameover.cpp:236-406`), the
//! `dGameover_c` state machine (`:109-166`), and the save screen composition.
//! Asset-backed checks skip when `CS_ASSETS` is unset.

use std::collections::HashMap;

use clearsleeper::assets::AssetRoot;
use clearsleeper::bti::Image;
use clearsleeper::game_over_screen::{
    self, GameOver, GameOverAssets, LETTER_TAGS, LETTER_X, ROTATE_TABLE, STATE_SAVE_OPEN,
};
use clearsleeper::j2d::{Bounds, Kind, Screen, ScreenPane};
use clearsleeper::panes::{self, SaveAssets};
use clearsleeper::sim::health::Health;
use clearsleeper::sim::vibration::Vibration;
use clearsleeper::text::raster::Canvas;
use clearsleeper::ui_menu_save::{
    MemCard, MenuFlags, PadInput, Particles, SaveEnv, SaveMenu, SavePanes, SystemRequest, TITLE_TAG,
};

struct Fixture {
    card: MemCard,
    flags: MenuFlags,
    system: SystemRequest,
    particles: Particles,
    vibration: Vibration,
    health: Health,
}

impl Fixture {
    fn new() -> Self {
        Self {
            card: MemCard::default(),
            flags: MenuFlags::default(),
            system: SystemRequest::default(),
            particles: Particles::default(),
            vibration: Vibration::default(),
            health: Health::default(),
        }
    }

    fn env(&mut self) -> SaveEnv<'_> {
        let life = u16::from(self.health.units);
        SaveEnv {
            pad: PadInput::default(),
            card: &mut self.card,
            menu: &mut self.flags,
            system: &mut self.system,
            particles: &mut self.particles,
            pending_save: None,
            vibration: &mut self.vibration,
            health: &mut self.health,
            life_on_open: life,
        }
    }
}

fn offset_y(over: &GameOver, index: usize) -> f32 {
    over.letters.letter[index].pos_center.y - over.letters.letter[index].pos_center_orig.y
}

#[test]
fn letter_open_drops_on_the_decomp_offsets() {
    let mut over = GameOver::default();
    let expected = [
        -276.48_f32,
        -241.92,
        -184.32,
        -103.68,
        0.0,
        -2.25,
        -9.0,
        -6.75,
        0.0,
    ];
    for (step, want) in expected.iter().enumerate() {
        let done = over.letters.anime1(0);
        let frame = step + 1;
        assert_eq!(done, frame == expected.len(), "anime1 finishes on frame 9");
        assert!(
            (offset_y(&over, 0) - want).abs() < 1e-3,
            "frame {frame}: y {} want {want}",
            offset_y(&over, 0)
        );
        assert_eq!(
            over.letters.letter[0].now_alpha, 0xFF,
            "the letter shows its init alpha from the first step"
        );
    }
    assert_eq!(over.letters.letter[0].user_area, 9);
    assert_eq!(over.letters.rotate, [0; 8], "the open never tilts a letter");
}

#[test]
fn letter_close_falls_244_and_vanishes_on_frame_seven() {
    let mut over = GameOver::default();
    for _ in 0..9 {
        over.letters.anime1(0);
    }
    // `animeClose` runs after `valueInit` restores the user areas (`:151-152`).
    over.letters.value_init();
    for step in 1..=7i32 {
        let done = over.letters.anime2(0);
        let want = (step * step) as f32 / 49.0 * 244.0;
        assert!(
            (offset_y(&over, 0) - want).abs() < 1e-3,
            "frame {step}: y {} want {want}",
            offset_y(&over, 0)
        );
        assert_eq!(done, step == 7, "anime2 finishes on frame 7");
    }
    assert!(ROTATE_TABLE.contains(&over.letters.rotate[0]));
    assert_eq!(over.letters.letter[0].now_alpha, 0, "the letter is gone");
}

#[test]
fn open_holds_the_90_frame_gate_until_every_letter_lands() {
    let mut fix = Fixture::new();
    let mut over = GameOver::default();
    over.anime_start();

    let mut frames = 0;
    while over.hold == 90 {
        let _ = over.step(&mut fix.env());
        frames += 1;
        assert!(frames < 600, "the open must complete");
    }
    assert_eq!(over.hold, 89, "the gate drops one frame after the open");
    for (index, pane) in over.letters.letter.iter().enumerate() {
        assert_eq!(pane.user_area, 9, "letter {index} landed");
        assert!(
            (offset_y(&over, index)).abs() < 1e-3,
            "letter {index} settles at its own origin"
        );
    }

    for _ in 0..88 {
        let _ = over.step(&mut fix.env());
    }
    assert_eq!(over.hold, 1);
    let _ = over.step(&mut fix.env());
    assert_eq!(over.hold, 0, "field_0x114 counts down to zero");
    assert_eq!(over.letters.count, 8, "valueInit restores the pick budget");
    assert!(
        over.letters.letter.iter().all(|pane| pane.user_area == 0),
        "valueInit restores every letter state"
    );
}

#[test]
fn close_starts_with_the_boot_seed_and_gates_the_tail() {
    let mut over = GameOver::default();
    assert!(!over.letters.anime_close());
    assert_eq!(over.letters.letter[5].user_area, 1, "the first pick is txo");
    assert!(
        over.letters
            .letter
            .iter()
            .enumerate()
            .all(|(index, pane)| index == 5 || pane.user_area == 0),
        "only the picked letter starts"
    );

    let mut frames = 0;
    while !over.letters.anime_close() {
        frames += 1;
        assert!(frames < 600, "the close must complete");
    }
    for pane in &over.letters.letter {
        assert_eq!(pane.user_area, 7);
        assert_eq!(pane.now_alpha, 0, "a closed letter is invisible");
    }

    let mut fix = Fixture::new();
    over.anime_start();
    over.hold = 0;
    over.tail = 1;
    let _ = over.step(&mut fix.env());
    assert_eq!(over.tail, 0);
    assert_eq!(over.state, 0, "tail 1 does not open the save menu yet");
    let _ = over.step(&mut fix.env());
    assert_eq!(over.state, STATE_SAVE_OPEN);
    assert!(over.menu_open, "dMenu_flagSet(1) follows the tail");

    over.save.set_use_type(2);
    let mut opened = false;
    for _ in 0..120 {
        let _ = over.step(&mut fix.env());
        if over.state != STATE_SAVE_OPEN {
            opened = true;
            break;
        }
    }
    assert!(opened, "state 3 opens the save menu and moves to state 4");
    assert_eq!(over.save.get_save_status(), 2, "the save menu reports open");
}

#[test]
fn the_save_screen_composes_visible_panes_and_skips_hidden_ones() {
    let screen = Screen {
        width: 640,
        height: 480,
        color: 0,
        panes: vec![picture("tk00", 10.0, 10.0), picture("tl00", 30.0, 10.0)],
    };
    let mut textures = HashMap::new();
    textures.insert(
        "solid.bti".to_string(),
        Image {
            width: 2,
            height: 2,
            pixels: vec![255; 16],
            mag_linear: false,
            wrap_s: 0,
            wrap_t: 0,
            alpha_enabled: true,
        },
    );
    let assets = SaveAssets::from_parts(screen, textures);
    let mut panes = SavePanes::default();
    panes.bind(&assets.assets.screen);
    panes.title.set_init_alpha();
    panes.title_label.set_init_alpha();

    let mut canvas = Canvas::new(64, 64);
    panes::draw_save_menu(&mut canvas, &panes, &assets);
    assert!(alpha_at(&canvas, 14, 14) > 0, "the title pane draws");
    assert!(alpha_at(&canvas, 34, 14) > 0, "the label pane draws");

    panes.title.hide();
    canvas.clear();
    panes::draw_save_menu(&mut canvas, &panes, &assets);
    assert_eq!(alpha_at(&canvas, 14, 14), 0, "a hidden pane is skipped");
    assert!(alpha_at(&canvas, 34, 14) > 0, "its neighbour still draws");
}

#[test]
fn gover_and_saveres_draw_inside_their_pane_rects() {
    if std::env::var_os("CS_ASSETS").is_none() {
        eprintln!("CS_ASSETS is not set; skipping game over asset check");
        return;
    }
    let Ok(root) = AssetRoot::from_env() else {
        eprintln!("asset root unavailable; skipping game over asset check");
        return;
    };
    let Some(gover) = GameOverAssets::load(&root) else {
        eprintln!("Gover.arc unreadable; skipping game over asset check");
        return;
    };
    let mut over = GameOver::default();
    over.create(&gover.assets.screen);
    for (index, tag) in LETTER_TAGS.iter().enumerate() {
        assert_eq!(over.letters.letter[index].tag, *tag);
        over.letters.letter[index].set_init_alpha();
    }
    over.letters.set_back_alpha(0.0);
    let mut canvas = Canvas::new(640, 480);
    game_over_screen::render(&mut canvas, &gover, &over);
    for (index, tag) in LETTER_TAGS.iter().enumerate() {
        let bounds = gover
            .assets
            .screen
            .find(tag)
            .unwrap_or_else(|| panic!("{tag} bound"))
            .bounds;
        assert!(
            rect_has_alpha(&canvas, &bounds),
            "letter {index} ({tag}) draws inside its pane rect"
        );
    }
    assert_eq!(LETTER_X[5], 75.0, "the particle table keeps txo's x");

    let Some(save_assets) = SaveAssets::load(&root) else {
        eprintln!("saveres.arc unreadable; skipping save screen asset check");
        return;
    };
    let mut fix = Fixture::new();
    let mut save = SaveMenu::default();
    save.set_use_type(2);
    save.create(
        save_assets.assets.screen.clone(),
        Screen {
            width: 640,
            height: 480,
            color: 0,
            panes: Vec::new(),
        },
    );
    let mut opened = false;
    for _ in 0..240 {
        if save.open(&mut fix.env()) {
            opened = true;
            break;
        }
    }
    assert!(opened, "the game over save screen opens");
    let mut canvas = Canvas::new(640, 480);
    panes::draw_save_menu(&mut canvas, &save.panes, &save_assets);
    let bounds = save_assets
        .assets
        .screen
        .find(TITLE_TAG)
        .expect("title pane bound")
        .bounds;
    assert!(
        rect_has_alpha(&canvas, &bounds),
        "the opened save screen draws its title"
    );
}

fn picture(name: &str, x: f32, y: f32) -> ScreenPane {
    ScreenPane {
        name: name.to_string(),
        kind: Kind::Picture,
        texture: Some("solid.bti".to_string()),
        bounds: Bounds {
            x,
            y,
            w: 8.0,
            h: 8.0,
        },
        alpha: 0xFF,
        ..Default::default()
    }
}

fn alpha_at(canvas: &Canvas, x: usize, y: usize) -> u8 {
    canvas.pixels[(y * canvas.width + x) * 4 + 3]
}

fn rect_has_alpha(canvas: &Canvas, bounds: &Bounds) -> bool {
    let x0 = bounds.x.max(0.0) as usize;
    let y0 = bounds.y.max(0.0) as usize;
    let x1 = ((bounds.x + bounds.w) as usize).min(canvas.width);
    let y1 = ((bounds.y + bounds.h) as usize).min(canvas.height);
    (y0..y1).any(|y| (x0..x1).any(|x| alpha_at(canvas, x, y) > 0))
}
