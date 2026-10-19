use std::time::Duration;

use clearsleeper::App;
use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::arrow::{self, Arrow};
use clearsleeper::sim::boomerang::Boomerang;
use clearsleeper::sim::combat::HitPoints;
use clearsleeper::sim::enemy::{self, Hurt};
use clearsleeper::sim::item::{self, Bomb, Held, Kind};
use clearsleeper::sim::player::Player;
use glam::Vec3;
use repose_core::input::PhysicalKey;
use repose_core::runtime::Scheduler;

use clearsleeper::SIM_HZ;

const BOMB_BAG: u8 = item::FIRST_BOMB + 1;
const ARROW_BAG: u8 = item::FIRST_ARROW;

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

fn held(app: &mut App, item: u8) {
    let mut held = app.sim.world.resource_mut::<Held>();
    held.equip(item);
}

fn bombs(app: &App) -> Vec<Bomb> {
    app.sim
        .world
        .iter_entities()
        .filter_map(|e| e.get::<Bomb>().copied())
        .collect()
}

fn arrows(app: &App) -> Vec<Arrow> {
    app.sim
        .world
        .iter_entities()
        .filter_map(|e| e.get::<Arrow>().copied())
        .collect()
}

fn spins(app: &App) -> Vec<Boomerang> {
    app.sim
        .world
        .iter_entities()
        .filter_map(|e| e.get::<Boomerang>().cloned())
        .collect()
}

fn foe(app: &mut App, pos: Vec3) -> bevy_ecs::entity::Entity {
    let (state, hit_points) = enemy::spawn(0, 0);
    app.sim
        .world
        .spawn((
            Actor {
                name: enemy::ENEMY_ACTOR.to_owned(),
                parameters: 0,
                pos,
                angle: [0, 0, 0],
                set_id: 0,
                room: -1,
                ..Default::default()
            },
            state,
            hit_points,
            Hurt::default(),
        ))
        .id()
}

fn foe_hp(app: &App, entity: bevy_ecs::entity::Entity) -> i32 {
    app.sim
        .world
        .entity(entity)
        .get::<HitPoints>()
        .map_or(0, |hp| hp.0)
}

fn resting_bomb(app: &mut App, pos: Vec3, fuse: u32) {
    app.sim.world.spawn((
        Bomb {
            pos,
            fuse,
            vel: Vec3::ZERO,
            rest: true,
        },
        item::InFlight,
    ));
}

#[test]
fn golden_bags_pick_the_item_they_name() {
    let mut app = App::new();
    assert_eq!(
        (app.held().item, app.held().count),
        (0, 0),
        "you start with empty hands"
    );
    held(&mut app, BOMB_BAG);
    assert_eq!(
        (app.held().item, app.held().count),
        (item::FIRST_BOMB, 10),
        "the 10-bomb bag equips bombs"
    );
    assert_eq!(app.sim.world.resource::<Held>().kind(), Some(Kind::Bomb));
    held(&mut app, ARROW_BAG);
    assert_eq!(
        (app.held().item, app.held().count),
        (item::FIRST_ARROW, 10),
        "and the last bag in hand wins"
    );
}

#[test]
fn golden_bags_cap_at_the_biggest_pouch() {
    let mut app = App::new();
    held(&mut app, item::LAST_BOMB);
    held(&mut app, item::LAST_BOMB);
    assert_eq!(app.held().count, 30, "the pouch tops out at thirty");
}

#[test]
fn golden_using_an_item_spends_one_and_empty_hands_throw_nothing() {
    let mut app = App::new();
    use_item(&mut app);
    assert!(bombs(&app).is_empty(), "no bag, no bomb");
    assert!(arrows(&app).is_empty(), "no bag, no arrow");
    held(&mut app, BOMB_BAG);
    use_item(&mut app);
    assert_eq!(bombs(&app).len(), 1, "one press throws one bomb");
    assert_eq!(app.held().count, 9, "and it is gone from the bag");
}

#[test]
fn golden_bomb_blast_only_hurts_what_it_is_next_to() {
    let mut app = App::new();
    place_player(&mut app, Vec3::ZERO);
    let victim = foe(&mut app, Vec3::new(0.0, 0.0, 300.0));
    let bystander = foe(&mut app, Vec3::new(0.0, 0.0, 900.0));
    // The throw carries a bomb some 1500 units downrange before its hundred-tick fuse runs out,
    // so the blast under test is the one lit beside the near foe: four power
    // (`d_a_bomb3.inc:1402`) off a five-health boko (`d_a_bk.cpp:5109-5111`).
    resting_bomb(&mut app, Vec3::new(0.0, 0.0, 300.0), 10);
    for _ in 0..12 {
        tick(&mut app);
    }
    assert_eq!(foe_hp(&app, victim), 1, "four power bites the near foe");
    assert_eq!(foe_hp(&app, bystander), 5, "and the far one is untouched");
    assert_eq!(app.health().units, 12, "the hero stood clear of the blast");
}

#[test]
fn golden_one_bomb_sets_off_the_next() {
    let mut app = App::new();
    place_player(&mut app, Vec3::new(0.0, 0.0, 40_000.0));
    resting_bomb(&mut app, Vec3::ZERO, 2);
    resting_bomb(&mut app, Vec3::new(0.0, 0.0, 150.0), 100);
    for _ in 0..4 {
        tick(&mut app);
    }
    assert!(bombs(&app).is_empty(), "the second goes off with the first");
}

#[test]
fn golden_arrow_bites_and_then_sticks_for_three_hundred_frames() {
    let mut app = App::new();
    place_player(&mut app, Vec3::ZERO);
    let victim = foe(&mut app, Vec3::new(0.0, 0.0, 400.0));
    held(&mut app, ARROW_BAG);
    use_item(&mut app);
    tick(&mut app);
    tick(&mut app);
    assert_eq!(foe_hp(&app, victim), 3, "two power bites the foe it passes");
    assert_eq!(
        arrows(&app)[0].phase,
        arrow::Phase::StuckActor,
        "and the shaft stays in it"
    );
    for _ in 0..arrow::STUCK_FRAMES {
        tick(&mut app);
    }
    assert!(arrows(&app).is_empty(), "then it lets go");
}

#[test]
fn golden_boomerang_leaves_the_slot_and_comes_home_to_it() {
    let mut app = App::new();
    place_player(&mut app, Vec3::ZERO);
    held(&mut app, item::BOOMERANG);
    use_item(&mut app);
    assert_eq!(spins(&app).len(), 1, "the spin leaves the hand");
    assert_eq!(app.held().count, 0, "and the slot is empty while it flies");
    for _ in 0..400 {
        tick(&mut app);
    }
    assert!(spins(&app).is_empty(), "the spin comes home");
    assert_eq!(
        (app.held().item, app.held().count),
        (item::BOOMERANG, 1),
        "into the empty slot again"
    );
}

#[test]
fn golden_projectiles_do_not_survive_a_stage_change() {
    let mut app = App::new();
    place_player(&mut app, Vec3::ZERO);
    held(&mut app, BOMB_BAG);
    use_item(&mut app);
    assert_eq!(bombs(&app).len(), 1);
    // An empty stage stands in for the destination room; the swap clears what is in flight.
    let tables = clearsleeper::dzs::parse(&0_u32.to_be_bytes()).expect("an empty stage parses");
    app.swap_stage(
        tables,
        Vec::new(),
        Vec3::ZERO,
        0.0,
        clearsleeper::sim::transition::LAYER_NONE,
    );
    assert!(
        bombs(&app).is_empty() && arrows(&app).is_empty() && spins(&app).is_empty(),
        "nothing in flight follows you into the next room"
    );
}
