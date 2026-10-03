use std::path::PathBuf;
use std::time::Duration;

use clearsleeper::menus::{Action, Cmd, Menu, Screen, SlotInfo, play_time};
use clearsleeper::save::{self, SaveGame, SavedArea, SavedHealth, SavedPlayer};
use clearsleeper::sim::death::GameOver;
use clearsleeper::sim::intent::{DamageIntent, GameOverChoice, MoveIntent};
use clearsleeper::{App, SIM_HZ};
use glam::Vec2;
use repose_core::input::PhysicalKey;
use repose_core::runtime::Scheduler;

fn stage(app: &mut App, keys: &[PhysicalKey]) {
    let mut sched = Scheduler::default();
    sched.window_focused = true;
    sched.held_keys = keys.iter().cloned().collect();
    app.stage_polled(&sched);
}

fn tap(app: &mut App, key: PhysicalKey) {
    stage(app, &[key]);
    stage(app, &[]);
}

fn temp_dir(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("clearsleeper_menu_{tag}_{}", std::process::id()))
}

fn sample_save(name: &str, tick: u64) -> SaveGame {
    SaveGame {
        version: save::SAVE_VERSION,
        name: name.to_string(),
        tick,
        camera_yaw: 0.5,
        player: SavedPlayer {
            pos: [3.0, 4.0, 5.0],
            yaw: 1.0,
            speed: 6.0,
            speed_y: -2.5,
        },
        area: SavedArea {
            bounds_min: [-1000.0, -1000.0],
            bounds_max: [1000.0, 1000.0],
            spawn: [0.0, 0.0, 0.0],
            yaw: 0.0,
        },
        health: SavedHealth { units: 7, max: 16 },
    }
}

#[test]
fn golden_title_confirm_opens_file_select() {
    let dir = temp_dir("title");
    let mut app = App::new();
    app.set_save_dir(dir.clone());
    app.menu.screen = Screen::Title;
    stage(&mut app, &[]);
    assert_eq!(app.menu.screen, Screen::Title);
    tap(&mut app, PhysicalKey::Space);
    assert_eq!(app.menu.screen, Screen::FileSelect);
    assert!(app.menu.slots.iter().all(|slot| *slot == SlotInfo::Empty));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn golden_cursor_clamps_at_ends() {
    let dir = temp_dir("cursor");
    let mut app = App::new();
    app.set_save_dir(dir.clone());
    app.menu.screen = Screen::Title;
    tap(&mut app, PhysicalKey::Space);
    assert_eq!(app.menu.slot, 0);
    tap(&mut app, PhysicalKey::ArrowUp);
    assert_eq!(app.menu.slot, 0);
    tap(&mut app, PhysicalKey::ArrowDown);
    assert_eq!(app.menu.slot, 1);
    tap(&mut app, PhysicalKey::ArrowDown);
    tap(&mut app, PhysicalKey::ArrowDown);
    assert_eq!(app.menu.slot, 2);
    tap(&mut app, PhysicalKey::ArrowDown);
    assert_eq!(app.menu.slot, 2);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn golden_new_game_enters_play_and_writes_slot() {
    let dir = temp_dir("new");
    let mut app = App::new();
    app.set_save_dir(dir.clone());
    app.menu.screen = Screen::Title;
    tap(&mut app, PhysicalKey::Space);
    for _ in 0..5 {
        app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
    }
    tap(&mut app, PhysicalKey::Space);
    assert_eq!(app.menu.screen, Screen::Playing);
    let written = save::read_slot_in(&dir, 0).expect("new game writes slot zero");
    assert_eq!(written.tick, 0);
    assert_eq!(written.name, save::DEFAULT_NAME);
    assert_eq!(written.health.units, 12);
    assert_eq!(written.health.max, 12);
    assert_eq!(app.save(), written);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn golden_continue_loads_slot() {
    let dir = temp_dir("load");
    let mut source = App::new();
    for _ in 0..10 {
        stage(&mut source, &[PhysicalKey::KeyW]);
        source.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
    }
    let prepared = source.save();
    save::write_slot_in(&dir, 1, &prepared).unwrap();

    let mut app = App::new();
    app.set_save_dir(dir.clone());
    app.menu.screen = Screen::Title;
    tap(&mut app, PhysicalKey::Space);
    tap(&mut app, PhysicalKey::ArrowDown);
    assert_eq!(app.menu.slots[1], SlotInfo::from_save(&prepared));
    tap(&mut app, PhysicalKey::Space);
    assert_eq!(app.menu.screen, Screen::Playing);
    assert_eq!(app.save(), prepared);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn golden_menu_gates_game_input() {
    let mut app = App::new();
    app.menu.screen = Screen::Title;
    stage(&mut app, &[PhysicalKey::KeyW, PhysicalKey::KeyK]);
    assert_eq!(app.sim.world.resource::<MoveIntent>().axis, Vec2::ZERO);
    assert_eq!(app.sim.world.resource::<DamageIntent>().0, 0);
    app.menu.screen = Screen::Playing;
    stage(&mut app, &[PhysicalKey::KeyW]);
    assert_eq!(app.sim.world.resource::<MoveIntent>().axis, Vec2::Y);
}

#[test]
fn golden_back_returns_to_title() {
    let dir = temp_dir("back");
    let mut app = App::new();
    app.set_save_dir(dir.clone());
    app.menu.screen = Screen::Title;
    tap(&mut app, PhysicalKey::Space);
    assert_eq!(app.menu.screen, Screen::FileSelect);
    tap(&mut app, PhysicalKey::Escape);
    assert_eq!(app.menu.screen, Screen::Title);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn golden_step_confirm_reports_slot_action() {
    let mut menu = Menu::new();
    assert_eq!(menu.step(&[Cmd::Confirm]), None);
    assert_eq!(menu.screen, Screen::FileSelect);
    menu.slots = vec![
        SlotInfo::Empty,
        SlotInfo::from_save(&sample_save("A", 1800)),
        SlotInfo::Empty,
    ];
    menu.slot = 1;
    assert_eq!(menu.step(&[Cmd::Confirm]), Some(Action::Load(1)));
    assert_eq!(
        menu.screen,
        Screen::FileSelect,
        "the app switches screens after the action applies"
    );
    menu.slot = 2;
    assert_eq!(menu.step(&[Cmd::Confirm]), Some(Action::New(2)));
    assert_eq!(menu.step(&[Cmd::Back]), None);
    assert_eq!(menu.screen, Screen::Title);
}

#[test]
fn golden_slot_info_reads_save_fields() {
    let info = SlotInfo::from_save(&sample_save("Zelda", 1800));
    assert_eq!(
        info,
        SlotInfo::File {
            name: "Zelda".to_string(),
            units: 7,
            max: 16,
            tick: 1800,
        }
    );
}

#[test]
fn golden_play_time_formats_hours_and_minutes() {
    assert_eq!(play_time(0), "0:00");
    assert_eq!(play_time(89), "0:00");
    assert_eq!(play_time(SIM_HZ as u64 * 60), "0:01");
    assert_eq!(play_time(SIM_HZ as u64 * 3600), "1:00");
    assert_eq!(play_time(SIM_HZ as u64 * 3900), "1:05");
}

#[test]
fn golden_game_over_menu_reports_choice() {
    let mut menu = Menu::new();
    menu.screen = Screen::GameOver;
    assert_eq!(menu.step(&[Cmd::Up]), None);
    assert_eq!(menu.slot, 0);
    assert_eq!(menu.step(&[Cmd::Down]), None);
    assert_eq!(menu.slot, 1);
    assert_eq!(menu.step(&[Cmd::Down]), None);
    assert_eq!(menu.slot, 1, "the cursor clamps at the last choice");
    assert_eq!(
        menu.step(&[Cmd::Confirm]),
        Some(Action::GameOver(GameOverChoice::Quit))
    );
    assert_eq!(
        menu.screen,
        Screen::GameOver,
        "the app switches after the choice applies"
    );
    menu.slot = 0;
    assert_eq!(
        menu.step(&[Cmd::Confirm]),
        Some(Action::GameOver(GameOverChoice::Continue))
    );
}

#[test]
fn golden_death_opens_game_over_and_continue_restores_play() {
    let mut app = App::new();
    app.stage_damage(12);
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
    assert_eq!(app.menu.screen, Screen::GameOver, "death opens the menu");
    assert_eq!(app.menu.slot, 0, "continue is the default choice");

    tap(&mut app, PhysicalKey::Space);
    assert_eq!(app.menu.screen, Screen::Playing);
    assert_eq!(
        *app.sim.world.resource::<GameOverChoice>(),
        GameOverChoice::Continue
    );

    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
    assert_eq!(app.health().units, 12, "continue restores the hearts");
    assert!(!app.sim.world.resource::<GameOver>().active);
    assert_eq!(app.menu.screen, Screen::Playing);
}

#[test]
fn golden_game_over_quit_returns_to_title() {
    let mut app = App::new();
    app.stage_damage(12);
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
    assert_eq!(app.menu.screen, Screen::GameOver);

    tap(&mut app, PhysicalKey::ArrowDown);
    assert_eq!(app.menu.slot, 1);
    tap(&mut app, PhysicalKey::Space);
    assert_eq!(app.menu.screen, Screen::Title);
    assert_eq!(
        *app.sim.world.resource::<GameOverChoice>(),
        GameOverChoice::Quit
    );

    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
    assert!(
        !app.sim.world.resource::<GameOver>().active,
        "the sim resets behind the title"
    );
    assert_eq!(app.health().units, 12, "the reset restores the hearts");
    assert_eq!(app.menu.screen, Screen::Title);
}
