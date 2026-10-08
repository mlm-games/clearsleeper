use std::time::Duration;

use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::fire::{self, Fire};
use clearsleeper::sim::flags::Switches;
use clearsleeper::sim::item::{self, Held};
use clearsleeper::sim::player::Player;
use clearsleeper::{App, SIM_HZ};
use glam::Vec3;
use repose_core::input::PhysicalKey;
use repose_core::runtime::Scheduler;

const SWITCH_TYPE: u32 = 0;
const BOMB_TYPE: u32 = 2 << 17;
const NO_SWITCH: u32 = 0xFF;

fn frame(app: &mut App, keys: &[PhysicalKey]) {
    let mut sched = Scheduler::default();
    sched.window_focused = true;
    sched.held_keys = keys.iter().cloned().collect();
    app.stage_polled(&sched);
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

fn tick(app: &mut App) {
    frame(app, &[]);
}

fn use_item(app: &mut App) {
    frame(app, &[PhysicalKey::KeyJ]);
    frame(app, &[]);
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

fn brazier(app: &mut App, pos: Vec3, parameters: u32) {
    let state = fire::spawn(parameters, app.sim.world.resource::<Switches>());
    app.sim.world.spawn((
        Actor {
            name: fire::FIRE_ACTOR.to_owned(),
            parameters,
            pos,
            angle: [0, 0, 0],
            set_id: 0,
            room: -1,
            ..Default::default()
        },
        state,
    ));
}

fn lit(app: &App) -> usize {
    app.sim
        .world
        .iter_entities()
        .filter(|e| e.get::<Fire>().is_some_and(|fire| fire.lit))
        .count()
}

fn give_bombs(app: &mut App) {
    let mut held = app.sim.world.resource_mut::<Held>();
    held.equip(item::FIRST_BOMB);
}

#[test]
fn golden_switch_brazier_burns_from_the_start() {
    let mut app = App::new();
    place_player(&mut app, Vec3::new(0.0, 0.0, 20_000.0));
    brazier(&mut app, Vec3::ZERO, SWITCH_TYPE | 9);
    assert_eq!(lit(&app), 1, "an unlit brazier waits for its switch");
    app.sim.world.resource_mut::<Switches>().set(9);
    assert_eq!(lit(&app), 0, "setting the switch puts it out");
}

#[test]
fn golden_cold_brazier_spares_the_player() {
    let mut app = App::new();
    place_player(&mut app, Vec3::new(0.0, 0.0, 0.0));
    brazier(&mut app, Vec3::ZERO, BOMB_TYPE | NO_SWITCH);
    for _ in 0..30 {
        tick(&mut app);
    }
    assert_eq!(app.health().units, 12, "a cold fire is no danger");
}

#[test]
fn golden_bomb_lights_the_brazier() {
    let mut app = App::new();
    place_player(&mut app, Vec3::new(0.0, 0.0, 20_000.0));
    brazier(&mut app, Vec3::new(0.0, 0.0, 400.0), BOMB_TYPE | NO_SWITCH);
    give_bombs(&mut app);
    use_item(&mut app);
    for _ in 0..30 {
        tick(&mut app);
    }
    assert_eq!(lit(&app), 1, "the blast catches it");
}
