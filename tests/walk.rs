use std::time::Duration;

use clearsleeper::sim::intent::CameraYaw;
use clearsleeper::sim::player::Player;
use clearsleeper::{App, SIM_HZ};
use glam::Vec2;

fn tick(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

fn player(app: &mut App) -> Player {
    let mut query = app.sim.world.query::<&Player>();
    *query
        .iter(&app.sim.world)
        .next()
        .expect("player exists at boot")
}

#[test]
fn golden_full_stick_runs_to_max_speed() {
    let mut app = App::new();
    app.stage_axis(Vec2::new(0.0, 1.0));
    let mut z = 0.0;
    for expected_speed in [3.5, 7.0, 10.5, 14.0, 17.0] {
        tick(&mut app);
        let p = player(&mut app);
        assert_eq!(p.speed, expected_speed);
        z += expected_speed;
        assert_eq!(p.pos.z, z);
        assert_eq!(p.pos.x, 0.0);
        assert_eq!(p.yaw, 0.0);
    }
    tick(&mut app);
    let p = player(&mut app);
    assert_eq!(p.speed, 17.0);
    assert_eq!(p.pos.z, z + 17.0);
}

#[test]
fn golden_release_decelerates_to_stop() {
    let mut app = App::new();
    app.stage_axis(Vec2::new(0.0, 1.0));
    for _ in 0..6 {
        tick(&mut app);
    }
    assert_eq!(player(&mut app).speed, 17.0);
    app.stage_axis(Vec2::ZERO);
    for _ in 0..8 {
        tick(&mut app);
    }
    let p = player(&mut app);
    assert_eq!(p.speed, 0.0);
    let expected_z = 69.0 + 14.5 + 12.0 + 9.5 + 7.0 + 4.5 + 2.0 + 0.2;
    assert!((p.pos.z - expected_z).abs() < 1e-3, "z = {}", p.pos.z);
}

#[test]
fn golden_camera_yaw_rotates_forward() {
    let mut app = App::new();
    app.sim.world.resource_mut::<CameraYaw>().0 = std::f32::consts::FRAC_PI_2;
    app.stage_axis(Vec2::new(0.0, 1.0));
    tick(&mut app);
    let p = player(&mut app);
    let expected_yaw = 3000.0 * (std::f32::consts::TAU / 65536.0);
    assert!((p.yaw - expected_yaw).abs() < 1e-9, "yaw = {}", p.yaw);
    assert!(p.pos.x > 0.0, "up-stick must track camera forward (+X)");
}

#[test]
fn golden_reversal_turns_in_place_at_hard_turn_rate() {
    let mut app = App::new();
    app.stage_axis(Vec2::new(0.0, -1.0));
    tick(&mut app);
    let p = player(&mut app);
    let expected_yaw = -3000.0 * (std::f32::consts::TAU / 65536.0);
    assert!((p.yaw - expected_yaw).abs() < 1e-9, "yaw = {}", p.yaw);
    assert_eq!(p.speed, 0.0, "reversal freezes acceleration until facing");
}
