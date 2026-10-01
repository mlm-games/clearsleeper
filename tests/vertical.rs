use std::time::Duration;

use clearsleeper::sim::player::Player;
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

fn set_player(app: &mut App, f: impl FnOnce(&mut Player)) {
    let mut query = app.sim.world.query::<&mut Player>();
    let mut player = query
        .iter_mut(&mut app.sim.world)
        .next()
        .expect("player exists at boot");
    f(&mut player);
}

fn player(app: &mut App) -> Player {
    let mut query = app.sim.world.query::<&Player>();
    *query
        .iter(&app.sim.world)
        .next()
        .expect("player exists at boot")
}

#[test]
fn golden_fall_lands_on_flat_ground() {
    let mut app = App::new();
    set_player(&mut app, |p| p.pos.y = 200.0);
    for _ in 0..30 {
        frame(&mut app, &[]);
    }
    let p = player(&mut app);
    assert_eq!(p.pos.y, -2.5, "grounded steady state sinks one gravity step");
    assert_eq!(p.speed_y, -2.5);
    assert_eq!(p.pos.x, 0.0);
    assert_eq!(p.pos.z, 0.0);
}

#[test]
fn golden_diagonal_walk_slides_along_area_wall() {
    let mut app = App::new();
    set_player(&mut app, |p| p.pos.z = 990.0);
    for _ in 0..10 {
        frame(&mut app, &[PhysicalKey::KeyW, PhysicalKey::KeyD]);
    }
    let p = player(&mut app);
    assert_eq!(p.pos.z, 1000.0, "north wall pins z exactly");
    assert!(
        p.pos.x < -50.0 && p.pos.x > -1000.0,
        "motion continues tangentially: x = {}",
        p.pos.x
    );
}
