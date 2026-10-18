use std::collections::HashSet;

use clearsleeper::App;
use clearsleeper::sim::intent::MoveIntent;
use glam::Vec2;
use repose_core::input::PhysicalKey;
use repose_core::runtime::Scheduler;

fn held(keys: &[PhysicalKey]) -> HashSet<PhysicalKey> {
    keys.iter().cloned().collect()
}

#[test]
fn golden_keyboard_walk_intent() {
    let mut app = App::new();
    let mut sched = Scheduler::default();
    sched.window_focused = true;

    sched.held_keys = held(&[PhysicalKey::KeyW]);
    app.stage_polled(&sched);
    assert_eq!(
        app.sim.world.resource::<MoveIntent>().axis,
        Vec2::new(0.0, 1.0)
    );

    sched.held_keys = held(&[PhysicalKey::KeyD, PhysicalKey::KeyS]);
    app.stage_polled(&sched);
    let axis = app.sim.world.resource::<MoveIntent>().axis;
    assert!((axis.length() - 1.0).abs() < 1e-6);
    assert!(axis.x > 0.0 && axis.y < 0.0);

    sched.window_focused = false;
    app.stage_polled(&sched);
    assert_eq!(app.sim.world.resource::<MoveIntent>().axis, Vec2::ZERO);
}

#[test]
fn staged_intent_reaches_ticks() {
    let mut app = App::new();
    app.stage_axis(Vec2::new(1.0, 0.0));
    let ran = app.advance(std::time::Duration::from_millis(100));
    assert_eq!(ran, 3);
    assert_eq!(
        app.sim.world.resource::<MoveIntent>().axis,
        Vec2::new(1.0, 0.0)
    );
}
