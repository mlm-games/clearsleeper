use std::time::Duration;

use clearsleeper::save::{SaveGame, read_save, write_save};
use clearsleeper::sim::intent::CameraYaw;
use clearsleeper::{App, SIM_HZ};
use repose_core::input::PhysicalKey;
use repose_core::runtime::Scheduler;

fn frame(app: &mut App, keys: &[PhysicalKey]) {
    let mut sched = Scheduler::default();
    sched.window_focused = true;
    sched.held_keys = keys.iter().cloned().collect();
    app.stage_polled(&sched);
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

#[test]
fn golden_save_json_round_trip_preserves_state() {
    let mut app = App::new();
    app.sim.world.resource_mut::<CameraYaw>().0 = 0.75;
    for _ in 0..10 {
        frame(&mut app, &[PhysicalKey::KeyW]);
    }
    let save = app.save();
    let text = serde_json::to_string(&save).unwrap();
    let parsed: SaveGame = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed, save);

    let mut restored = App::new();
    restored.load(&parsed).unwrap();
    assert_eq!(restored.save(), save);
}

#[test]
fn golden_loaded_session_continues_identically() {
    let mut app = App::new();
    app.sim.world.resource_mut::<CameraYaw>().0 = 0.75;
    for _ in 0..10 {
        frame(&mut app, &[PhysicalKey::KeyW]);
    }
    let save = app.save();

    for _ in 0..5 {
        frame(&mut app, &[PhysicalKey::KeyW, PhysicalKey::KeyD]);
    }
    let original = app.save();

    let mut branch = App::new();
    branch.load(&save).unwrap();
    for _ in 0..5 {
        frame(&mut branch, &[PhysicalKey::KeyW, PhysicalKey::KeyD]);
    }
    assert_eq!(branch.save(), original);
}

#[test]
fn golden_save_file_round_trip() {
    let path = std::env::temp_dir().join(format!(
        "clearsleeper_save_{}.json",
        std::process::id()
    ));
    let mut app = App::new();
    for _ in 0..6 {
        frame(&mut app, &[PhysicalKey::KeyS]);
    }
    let save = app.save();
    write_save(&path, &save).unwrap();
    assert_eq!(read_save(&path).unwrap(), save);
    let _ = std::fs::remove_file(&path);

    let mut bad = save;
    bad.version = 999;
    assert!(App::new().load(&bad).is_err());
}
