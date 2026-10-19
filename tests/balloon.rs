use std::time::Duration;

use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::balloon::{self, Balloon, Payload};
use clearsleeper::sim::combat::{HitGate, HitPoints};
use clearsleeper::sim::enemy::{Enemy, Hurt};
use clearsleeper::sim::flags::Collect;
use clearsleeper::sim::inventory::{SWORD_COLLECT_BYTE, SWORD_COLLECT_LEVEL};
use clearsleeper::sim::player::Player;
use clearsleeper::{App, SIM_HZ};
use glam::Vec3;
use repose_core::input::PhysicalKey;
use repose_core::runtime::Scheduler;

const BOKO_PARAM: u32 = 0x04;
const LANCE_PARAM: u32 = 0x05;
const EMPTY_PARAM: u32 = 0x03;
const BLADE_REACH: f32 = 20.0;

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

/// `item_func_sword` (`d_item.cpp:847-850`): the Hero's Sword is the collect bit
/// `dComIfGs_getSelectEquip(0)` is set from, and `setNormalCutAtParam` (`d_a_player_sword.inc:297`)
/// reads that selection to give a normal cut its atp of 1 rather than the master sword's 2.
fn grant_sword(app: &mut App) {
    app.sim
        .world
        .resource_mut::<Collect>()
        .set(SWORD_COLLECT_BYTE, SWORD_COLLECT_LEVEL);
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
                room: -1,
                ..Default::default()
            },
            state,
            hit_points,
            Hurt::default(),
            HitGate::default(),
        ))
        .id()
}

fn count<T: bevy_ecs::component::Component>(app: &App) -> usize {
    app.sim
        .world
        .iter_entities()
        .filter(|e| e.contains::<T>())
        .count()
}

#[test]
fn golden_balloon_carries_its_rider_and_holds_two_hits() {
    let mut app = App::new();
    grant_sword(&mut app);
    place_player(&mut app, Vec3::new(0.0, 0.0, 30.0));
    spawn_balloon(&mut app, Vec3::new(0.0, 0.0, 35.0), BOKO_PARAM);
    assert_eq!(count::<Balloon>(&app), 1);
    assert_eq!(count::<Enemy>(&app), 0, "nothing fights you yet");
    swing(&mut app);
    assert_eq!(
        count::<Balloon>(&app),
        1,
        "the first cut only dents it: the blade reaches {BLADE_REACH}"
    );
    // A balloon holds two (`d_a_bb.cpp:2766-2767`) and the Hero's Sword cut carries one
    // (`setNormalCutAtParam`, `d_a_player_sword.inc:297-309`), so the second hit has to be a
    // second cut. It cannot be a later frame of the first: the frame the AT reaches a TG sets the
    // balloon's own hit timer to five (`d_a_bb.cpp:2071-2072`), and the cut walks its AT over the
    // balloon for five frames. Twelve frames carry the cut out and well past the gate.
    for _ in 0..14 {
        tick(&mut app);
    }
    assert_eq!(
        count::<Balloon>(&app),
        1,
        "one cut, one hit: the gate holds the rest of the window off"
    );
    swing(&mut app);
    tick(&mut app);
    assert_eq!(count::<Balloon>(&app), 0, "the second cut pops it");
    assert_eq!(count::<Enemy>(&app), 1, "and its rider hits the deck");
    let rider = app
        .sim
        .world
        .iter_entities()
        .find_map(|e| e.get::<Enemy>().map(|_| e.id()))
        .expect("the rider landed");
    assert_eq!(
        app.sim.world.entity(rider).get::<HitPoints>().unwrap().0,
        5,
        "the dropped rider keeps its own health"
    );
}

#[test]
fn golden_balloon_rider_kind_comes_from_its_parameter() {
    let (state, _) = balloon::spawn(LANCE_PARAM);
    assert_eq!(
        state.payload,
        Some(Payload::Lance),
        "parameter five rides out the big one"
    );
    let (empty, _) = balloon::spawn(EMPTY_PARAM);
    assert_eq!(empty.payload, None);
}
