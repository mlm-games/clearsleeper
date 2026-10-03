use std::time::Duration;

use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::flags::Switches;
use clearsleeper::sim::player::Player;
use clearsleeper::sim::warship::{self, Cannon, Warship};
use clearsleeper::{App, SIM_HZ};
use glam::Vec3;

const PLAIN: u32 = 0xFFFF_FF00;
const GONE_WITH_AWAY_SWITCH: u32 = 0x0400_0000;
const GONE_WITH_HOME_SWITCH: i16 = 0x0007;

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

fn ships(app: &App) -> usize {
    app.sim
        .world
        .iter_entities()
        .filter(|e| e.contains::<Warship>())
        .count()
}

#[test]
fn golden_warship_fires_only_in_range() {
    let mut app = App::new();
    place_player(&mut app, Vec3::new(0.0, 0.0, 20_000.0));
    let _ = ship(&mut app, Vec3::new(0.0, 0.0, 0.0), 0, PLAIN, -1);
    tick(&mut app);
    assert_eq!(shots(&app), 0, "far out it holds its fire");
    place_player(&mut app, Vec3::new(0.0, 0.0, 2000.0));
    tick(&mut app);
    assert_eq!(shots(&app), 1, "close in it fires a round");
}

#[test]
fn golden_warship_waits_thirty_ticks_between_rounds() {
    let mut app = App::new();
    place_player(&mut app, Vec3::new(0.0, 0.0, 2000.0));
    let _ = ship(&mut app, Vec3::new(0.0, 0.0, 0.0), 0, PLAIN, -1);
    tick(&mut app);
    assert_eq!(shots(&app), 1);
    for _ in 0..29 {
        tick(&mut app);
    }
    assert_eq!(shots(&app), 1, "the gun stays empty for thirty ticks");
    tick(&mut app);
    assert_eq!(shots(&app), 2, "then it fires again");
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
    assert_eq!(shots(&app), 0, "and it is spent");
}

#[test]
fn golden_cannonball_drifts_flat_then_drops() {
    let mut app = App::new();
    place_player(&mut app, Vec3::new(0.0, 0.0, 20_000.0));
    let _ = ship(&mut app, Vec3::new(0.0, 0.0, 0.0), 0, PLAIN, -1);
    app.sim.world.spawn(Cannon {
        pos: Vec3::ZERO,
        vel: Vec3::new(0.0, 0.0, 85.0),
        free: 35,
        life: 240,
    });
    for _ in 0..20 {
        tick(&mut app);
    }
    let flat = app
        .sim
        .world
        .iter_entities()
        .find_map(|e| e.get::<Cannon>().copied())
        .expect("the round is still flying");
    assert_eq!(flat.pos.y, 0.0, "the shell runs level for its fuse");
    for _ in 0..40 {
        tick(&mut app);
    }
    let falling = app
        .sim
        .world
        .iter_entities()
        .find_map(|e| e.get::<Cannon>().copied())
        .expect("the round is still flying");
    assert!(
        falling.pos.y < 0.0,
        "then gravity takes it: {:?}",
        falling.pos
    );
    assert_eq!(ships(&mut app), 1);
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
    app.sim.world.resource_mut::<Switches>().0.set(4);
    assert!(
        !warship::is_switched(
            GONE_WITH_AWAY_SWITCH,
            -1,
            app.sim.world.resource::<Switches>()
        ),
        "and setting that switch keeps it at anchor"
    );
}

#[test]
fn golden_home_switch_docks_the_warship() {
    let mut app = App::new();
    assert!(
        !warship::is_switched(
            PLAIN,
            GONE_WITH_HOME_SWITCH,
            app.sim.world.resource::<Switches>()
        ),
        "it waits for its own switch"
    );
    app.sim
        .world
        .resource_mut::<Switches>()
        .0
        .set(GONE_WITH_HOME_SWITCH as u8);
    assert!(
        warship::is_switched(
            PLAIN,
            GONE_WITH_HOME_SWITCH,
            app.sim.world.resource::<Switches>()
        ),
        "and sails once the switch comes on"
    );
}

#[test]
fn golden_warship_sails_on() {
    let mut app = App::new();
    place_player(&mut app, Vec3::new(0.0, 0.0, 20_000.0));
    let _ = ship(&mut app, Vec3::new(0.0, 0.0, 0.0), 0, PLAIN, -1);
    let before = app
        .sim
        .world
        .iter_entities()
        .find_map(|e| e.get::<Actor>().map(|a| a.pos))
        .expect("the ship exists");
    for _ in 0..10 {
        tick(&mut app);
    }
    let after = app
        .sim
        .world
        .iter_entities()
        .find_map(|e| e.get::<Actor>().map(|a| a.pos))
        .expect("the ship exists");
    assert!(
        after.z > before.z,
        "the hull keeps moving: {before:?} {after:?}"
    );
}
