use std::time::Duration;

use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::flags::Switches;
use clearsleeper::sim::player::Player;
use clearsleeper::sim::warship::{self, Cannon};
use clearsleeper::{App, SIM_HZ};
use glam::Vec3;

const PLAIN: u32 = 0xFFFF_FF00;
const GONE_WITH_AWAY_SWITCH: u32 = 0x0400_0000;

fn tick(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

fn ship(app: &mut App, pos: Vec3, yaw: i16, parameters: u32, home_x: i16) -> bool {
    let allowed = warship::is_switched(parameters, home_x, app.sim.world.resource::<Switches>());
    app.sim.world.spawn((
        Actor {
            name: warship::SHIP_ACTOR.to_owned(),
            parameters,
            pos,
            angle: [home_x, yaw, 0],
            set_id: 0,
            room: -1,
            ..Default::default()
        },
        warship::spawn(yaw),
    ));
    allowed
}

fn place_player(app: &mut App, pos: Vec3) {
    let mut query = app.sim.world.query::<&mut Player>();
    for mut found in query.iter_mut(&mut app.sim.world) {
        found.pos = pos;
        found.yaw = 0.0;
        found.face = 0.0;
        found.speed = 0.0;
        found.speed_y = 0.0;
    }
}

fn shots(app: &App) -> usize {
    app.sim
        .world
        .iter_entities()
        .filter(|e| e.contains::<Cannon>())
        .count()
}

#[test]
fn golden_warship_fires_only_in_range() {
    let mut app = App::new();
    place_player(&mut app, Vec3::new(0.0, 0.0, 20_000.0));
    let _ = ship(&mut app, Vec3::new(0.0, 0.0, 0.0), 0, PLAIN, -1);
    tick(&mut app);
    assert_eq!(shots(&app), 0, "far out it holds its fire");
}

#[test]
fn golden_cannonball_costs_a_heart() {
    let mut app = App::new();
    place_player(&mut app, Vec3::new(0.0, 0.0, 1500.0));
    let _ = ship(&mut app, Vec3::new(0.0, 0.0, 0.0), 0, PLAIN, -1);
    for _ in 0..40 {
        tick(&mut app);
    }
    assert_eq!(app.health().units, 8, "a cannonball takes a whole heart");
}

#[test]
fn golden_switch_keeps_a_warship_at_anchor() {
    let mut app = App::new();
    let away = warship::is_switched(
        GONE_WITH_AWAY_SWITCH,
        -1,
        app.sim.world.resource::<Switches>(),
    );
    assert!(away, "a clear away-switch lets it sail");
    app.sim.world.resource_mut::<Switches>().set(4);
    assert!(
        !warship::is_switched(
            GONE_WITH_AWAY_SWITCH,
            -1,
            app.sim.world.resource::<Switches>()
        ),
        "and setting that switch keeps it at anchor"
    );
}
