use std::time::Duration;

use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::enemy::{self, Enemy, Hurt};
use clearsleeper::sim::fire::{self, Fire};
use clearsleeper::sim::flags::Switches;
use clearsleeper::sim::item::{self, Bomb, Held};
use clearsleeper::sim::player::Player;
use clearsleeper::{App, SIM_HZ};
use glam::Vec3;
use repose_core::input::PhysicalKey;
use repose_core::runtime::Scheduler;

const SWITCH_TYPE: u32 = 0;
const BOMB_TYPE: u32 = 2 << 17;
const ROOM_TYPE: u32 = 1 << 17;
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

fn foe(app: &mut App, pos: Vec3) {
    let (state, hit_points) = enemy::spawn(0, 0);
    app.sim.world.spawn((
        Actor {
            name: enemy::ENEMY_ACTOR.to_owned(),
            parameters: 0,
            pos,
            angle: [0, 0, 0],
            set_id: 0,
        },
        state,
        hit_points,
        Hurt::default(),
    ));
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
    app.sim.world.resource_mut::<Switches>().0.set(9);
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

#[test]
fn golden_lit_brazier_burns_the_player() {
    let mut app = App::new();
    place_player(&mut app, Vec3::new(0.0, 0.0, 0.0));
    brazier(&mut app, Vec3::ZERO, BOMB_TYPE | NO_SWITCH);
    give_bombs(&mut app);
    app.sim.world.spawn(Bomb {
        pos: Vec3::ZERO,
        fuse: 100,
        vel: Vec3::ZERO,
        rest: true,
    });
    tick(&mut app);
    assert_eq!(lit(&app), 1, "the bomb catches it at once");
    for _ in 0..30 {
        tick(&mut app);
    }
    assert!(
        app.health().units < 12,
        "and walking into it costs hearts: {}",
        app.health().units
    );
}

#[test]
fn golden_room_fire_dies_with_the_last_foe() {
    let mut app = App::new();
    place_player(&mut app, Vec3::new(0.0, 0.0, 20_000.0));
    brazier(&mut app, Vec3::ZERO, ROOM_TYPE | NO_SWITCH);
    foe(&mut app, Vec3::new(0.0, 0.0, 300.0));
    app.sim.world.spawn(Bomb {
        pos: Vec3::ZERO,
        fuse: 100,
        vel: Vec3::ZERO,
        rest: true,
    });
    tick(&mut app);
    assert_eq!(lit(&app), 1, "a burning room keeps its fire");
    let last = app
        .sim
        .world
        .iter_entities()
        .find(|e| e.contains::<Enemy>())
        .map(|e| e.id())
        .expect("the foe exists");
    app.sim.world.despawn(last);
    tick(&mut app);
    assert_eq!(lit(&app), 0, "and loses it once the room is clear");
}
