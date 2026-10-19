use std::time::Duration;

use clearsleeper::App;
use clearsleeper::SIM_HZ;
use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::flags::Switches;
use clearsleeper::sim::player::Player;
use clearsleeper::sim::warship::{self, Cannon};
use glam::Vec3;

mod common;

const GONE_WITH_AWAY_SWITCH: u32 = 0x0400_0000;
const PLAIN: u32 = 0xFFFF_FF00;

/// The area the tests sail in. The port's `Area` carries the bounds of the
/// room's own collision (`src/lib.rs`, the room install), and the reference's
/// own area data is wider than that: the sea's chart cell spans a hundred
/// thousand units (`d_a_sea.cpp:110-113`, `calcMinMax`), so an area of twenty
/// thousand either way is well inside what the reference holds, and it holds
/// every band the warship's own range table names (`d_a_oship.cpp:65-67`, the
/// outermost of them 12 000).
const SEA_HALF: f32 = 20_000.0;

fn tick(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
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

fn ship(app: &mut App, pos: Vec3, yaw: i16) {
    app.sim.world.spawn((
        Actor {
            name: warship::SHIP_ACTOR.to_owned(),
            parameters: PLAIN,
            pos,
            angle: [-1, yaw, 0],
            set_id: 0,
            room: -1,
            ..Default::default()
        },
        warship::spawn(yaw),
    ));
}

fn shots(app: &App) -> usize {
    app.sim
        .world
        .iter_entities()
        .filter(|e| e.contains::<Cannon>())
        .count()
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

/// The warship picks its mode by the distance to the player
/// (`changeModeByRange`, `d_a_oship.cpp:204-221`) out of the three bands its
/// own table names (`d_a_oship.cpp:65-67`: 2 500, 6 000 and 12 000 units), and
/// the last of them is the one that never arms the cannon - the mode it falls
/// back to is `modeRangeD` (`d_a_oship.cpp:598-607`), which only watches the
/// player.
#[test]
fn golden_the_warship_holds_its_fire_beyond_its_range() {
    let mut app = App::new();
    app.sim.world.insert_resource(common::area(SEA_HALF));
    place_player(&mut app, Vec3::new(0.0, 0.0, 20_000.0));
    ship(&mut app, Vec3::new(0.0, 0.0, 0.0), 0);
    for _ in 0..40 {
        tick(&mut app);
    }
    assert_eq!(shots(&app), 0, "far out it holds its fire");
}

/// A cannonball costs a whole heart. The shot the warship fires is a bomb
/// whose attack power is four (`d_a_bomb3.inc:1402`, the `SrcObjAt Atp` of
/// the bomb the cannon makes) and the player's health is counted in quarters
/// of a heart (`d_file_select.cpp:743`, the save data's health read back as
/// whole hearts over four), so four is one heart.
///
/// The cannon re-arms on the reference's own delay of thirty frames
/// (`d_a_oship.cpp:68`, `mAttackDelayA = 0x1e`, which `modeRangeAInit` rests
/// the timer to at `:516-518`), so the test stops before the second shot
/// could land.
#[test]
fn golden_one_cannonball_costs_one_heart() {
    let mut app = App::new();
    app.sim.world.insert_resource(common::area(SEA_HALF));
    place_player(&mut app, Vec3::new(0.0, 0.0, 1500.0));
    ship(&mut app, Vec3::new(0.0, 0.0, 0.0), 0);
    tick(&mut app);
    assert_eq!(shots(&app), 1, "the ship fires its cannon");
    for _ in 0..19 {
        tick(&mut app);
    }
    assert_eq!(app.health().units, 8, "the cannonball takes a whole heart");
    for _ in 0..11 {
        tick(&mut app);
    }
    assert_eq!(
        app.health().units,
        8,
        "and the thirty-frame delay has not let a second one land"
    );
}
