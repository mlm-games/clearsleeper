use std::time::Duration;

use clearsleeper::sim::health::Health;
use clearsleeper::{App, SIM_HZ};
use repose_core::input::PhysicalKey;
use repose_core::runtime::Scheduler;

fn frame(app: &mut App, keys: &[PhysicalKey], focused: bool) {
    let mut sched = Scheduler::default();
    sched.window_focused = focused;
    sched.held_keys = keys.iter().cloned().collect();
    app.stage_polled(&sched);
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

#[test]
fn golden_damage_via_key_stages_and_clamps() {
    let mut app = App::new();
    frame(&mut app, &[PhysicalKey::KeyK], true);
    assert_eq!(app.health().units, 11);
    frame(&mut app, &[PhysicalKey::KeyK], true);
    assert_eq!(app.health().units, 10);

    app.sim.world.resource_mut::<Health>().heal(100);
    assert_eq!(app.health(), Health::default());
    frame(&mut app, &[PhysicalKey::KeyK], false);
    assert_eq!(app.health(), Health::default());
}

#[test]
fn golden_health_survives_save_round_trip() {
    let mut app = App::new();
    frame(&mut app, &[PhysicalKey::KeyK], true);
    frame(&mut app, &[PhysicalKey::KeyK], true);
    let save = app.save();
    assert_eq!(save.health.units, 10);
    let mut restored = App::new();
    restored.load(&save).unwrap();
    assert_eq!(restored.health(), app.health());
}
