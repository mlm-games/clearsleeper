use std::time::Duration;

use clearsleeper::hud::{HeartState, heart_states};
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
fn golden_hearts_follow_health_units() {
    let at = |half_units| heart_states(&Health { half_units, max: 6 });
    assert_eq!(
        at(6),
        vec![HeartState::Full, HeartState::Full, HeartState::Full]
    );
    assert_eq!(
        at(5),
        vec![HeartState::Full, HeartState::Full, HeartState::Half]
    );
    assert_eq!(
        at(1),
        vec![HeartState::Half, HeartState::Empty, HeartState::Empty]
    );
    assert_eq!(
        at(0),
        vec![HeartState::Empty, HeartState::Empty, HeartState::Empty]
    );
}

#[test]
fn golden_damage_via_key_stages_and_clamps() {
    let mut app = App::new();
    frame(&mut app, &[PhysicalKey::KeyK], true);
    assert_eq!(app.health().half_units, 5);
    for _ in 0..10 {
        frame(&mut app, &[PhysicalKey::KeyK], true);
    }
    assert_eq!(app.health().half_units, 0);
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
    assert_eq!(save.health.half_units, 4);
    let mut restored = App::new();
    restored.load(&save).unwrap();
    assert_eq!(restored.health(), app.health());
}
