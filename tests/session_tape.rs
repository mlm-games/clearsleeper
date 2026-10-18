use std::time::Duration;

use clearsleeper::sim::frame::CurrentFrame;
use clearsleeper::sim::intent::CameraYaw;
use clearsleeper::sim::player::Player;
use clearsleeper::{App, SIM_HZ};
use glam::Vec2;
use repose_core::input::PhysicalKey;
use repose_core::runtime::Scheduler;

fn frame(app: &mut App, keys: &[PhysicalKey]) {
    let mut sched = Scheduler::default();
    sched.window_focused = true;
    sched.held_keys = keys.iter().cloned().collect();
    app.stage_polled(&sched);
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

fn player(app: &mut App) -> Player {
    let mut query = app.sim.world.query::<&Player>();
    *query
        .iter(&app.sim.world)
        .next()
        .expect("player exists at boot")
}

fn session() -> (u64, Player) {
    let mut app = App::new();

    for _ in 0..10 {
        frame(&mut app, &[PhysicalKey::KeyW]);
    }
    let a = player(&mut app);
    assert_eq!(a.pos, glam::Vec3::new(0.0, a.pos.y, 137.0));
    assert_eq!(a.yaw, 0.0);
    assert_eq!(a.speed, 17.0);

    for _ in 0..10 {
        frame(&mut app, &[PhysicalKey::KeyW, PhysicalKey::KeyD]);
    }
    let b = player(&mut app);
    let expected_yaw = -7730.0 * (std::f32::consts::TAU / 65536.0);
    assert!((b.yaw - expected_yaw).abs() < 1e-9, "yaw = {}", b.yaw);
    let stick = Vec2::new(1.0, 1.0).clamp_length_max(1.0).length();
    assert_eq!(b.speed, 17.0 * (stick * stick));
    assert!(
        b.pos.x < -80.0,
        "arc must bend camera-right: x = {}",
        b.pos.x
    );
    assert!(b.pos.z > 250.0, "z = {}", b.pos.z);

    for _ in 0..10 {
        frame(&mut app, &[]);
    }
    let c = player(&mut app);
    assert_eq!(c.speed, 0.0, "release must fully stop");

    app.sim.world.resource_mut::<CameraYaw>().0 = std::f32::consts::PI;
    for _ in 0..10 {
        frame(&mut app, &[PhysicalKey::KeyW]);
    }
    let d = player(&mut app);
    assert!(
        d.pos.z < c.pos.z,
        "camera-relative forward must flip with yaw"
    );

    let ticks = app.sim.world.resource::<CurrentFrame>().0;
    assert_eq!(ticks, 40);
    (ticks, d)
}

#[test]
fn golden_session_tape_is_deterministic() {
    let first = session();
    let second = session();
    assert_eq!(first.0, second.0);
    assert_eq!(first.1.pos, second.1.pos);
    assert_eq!(first.1.yaw, second.1.yaw);
    assert_eq!(first.1.speed, second.1.speed);
}
