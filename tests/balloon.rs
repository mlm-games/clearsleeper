use std::time::Duration;

use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::balloon::{self, Balloon, Payload};
use clearsleeper::sim::combat::HitPoints;
use clearsleeper::sim::enemy::{Enemy, Hurt};
use clearsleeper::sim::player::Player;
use clearsleeper::{App, SIM_HZ};
use glam::Vec3;
use repose_core::input::PhysicalKey;
use repose_core::runtime::Scheduler;

const BOKO_PARAM: u32 = 0x04;
const LANCE_PARAM: u32 = 0x05;
const EMPTY_PARAM: u32 = 0x03;

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

fn swing(app: &mut App) {
    frame(app, &[PhysicalKey::KeyB]);
    frame(app, &[]);
}

fn spawn_balloon(app: &mut App, pos: Vec3, parameters: u32) -> bevy_ecs::entity::Entity {
    let (state, hit_points) = balloon::spawn(parameters);
    app.sim
        .world
        .spawn((
            Actor {
                name: balloon::BALLOON_ACTOR.to_owned(),
                parameters,
                pos,
                angle: [0, 0, 0],
                set_id: 0,
            },
            state,
            hit_points,
            Hurt::default(),
        ))
        .id()
}

fn set_player(app: &mut App, pos: Vec3) {
    let mut query = app.sim.world.query::<&mut Player>();
    for mut found in query.iter_mut(&mut app.sim.world) {
        found.pos = pos;
        found.yaw = 0.0;
        found.face = 0.0;
        found.speed = 0.0;
        found.speed_y = 0.0;
    }
}

fn enemies(app: &App) -> usize {
    app.sim
        .world
        .iter_entities()
        .filter(|e| e.contains::<Enemy>())
        .count()
}

fn balloons(app: &App) -> usize {
    app.sim
        .world
        .iter_entities()
        .filter(|e| e.contains::<Balloon>())
        .count()
}

#[test]
fn golden_balloon_takes_two_hits_to_pop() {
    let mut app = App::new();
    set_player(&mut app, Vec3::new(0.0, 0.0, 30.0));
    spawn_balloon(&mut app, Vec3::new(0.0, 0.0, 0.0), BOKO_PARAM);
    swing(&mut app);
    assert_eq!(balloons(&app), 1, "the first cut only dents it");
    swing(&mut app);
    swing(&mut app);
    tick(&mut app);
    assert_eq!(balloons(&app), 0, "the second cut pops it");
}

#[test]
fn golden_balloon_releases_its_bokoblin() {
    let mut app = App::new();
    set_player(&mut app, Vec3::new(0.0, 0.0, 30.0));
    spawn_balloon(&mut app, Vec3::new(0.0, 0.0, 0.0), BOKO_PARAM);
    assert_eq!(enemies(&app), 0, "nothing fights you yet");
    swing(&mut app);
    swing(&mut app);
    swing(&mut app);
    tick(&mut app);
    assert_eq!(balloons(&app), 0, "the balloon is gone");
    assert_eq!(enemies(&app), 1, "and its rider hits the deck");
}

#[test]
fn golden_balloon_kind_five_carries_a_moblin() {
    let (state, _) = balloon::spawn(LANCE_PARAM);
    assert_eq!(state.payload, Some(Payload::Moblin));
}

#[test]
fn golden_balloon_kind_three_ride_empty() {
    let mut app = App::new();
    let (state, _) = balloon::spawn(EMPTY_PARAM);
    assert_eq!(state.payload, None);
    set_player(&mut app, Vec3::new(0.0, 0.0, 30.0));
    spawn_balloon(&mut app, Vec3::new(0.0, 0.0, 0.0), EMPTY_PARAM);
    swing(&mut app);
    swing(&mut app);
    tick(&mut app);
    assert_eq!(enemies(&app), 0, "an empty balloon drops nothing");
}

#[test]
fn golden_released_rider_can_be_hit_back() {
    let mut app = App::new();
    set_player(&mut app, Vec3::new(0.0, 0.0, 30.0));
    spawn_balloon(&mut app, Vec3::new(0.0, 0.0, 0.0), BOKO_PARAM);
    swing(&mut app);
    swing(&mut app);
    swing(&mut app);
    tick(&mut app);
    let rider = app
        .sim
        .world
        .iter_entities()
        .find(|e| e.contains::<Enemy>())
        .map(|e| e.id())
        .expect("the rider landed");
    assert_eq!(
        app.sim.world.entity(rider).get::<HitPoints>().unwrap().0,
        5,
        "the dropped rider keeps its own health"
    );
}
