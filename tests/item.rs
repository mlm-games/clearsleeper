use std::time::Duration;

use clearsleeper::App;
use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::combat::HitPoints;
use clearsleeper::sim::enemy;
use clearsleeper::sim::item::{self, Arrow, Bomb, Boomerang, Held, Kind};
use clearsleeper::sim::player::Player;
use glam::Vec3;
use repose_core::input::PhysicalKey;
use repose_core::runtime::Scheduler;

use clearsleeper::SIM_HZ;

const BOMB_ITEM: u8 = item::FIRST_BOMB + 1;
const ARROW_ITEM: u8 = item::FIRST_ARROW;

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

fn use_button(app: &mut App) {
    frame(app, &[PhysicalKey::KeyJ]);
    frame(app, &[]);
}

fn player(app: &App) -> Player {
    app.sim
        .world
        .iter_entities()
        .find_map(|e| e.get::<Player>().copied())
        .expect("player exists at boot")
}

fn hold(app: &mut App, item: u8) {
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
    let (state, hp) = enemy::spawn(0, 0);
    app.sim
        .world
        .spawn((
            Actor {
                name: enemy::ENEMY_ACTOR.to_owned(),
                parameters: 0,
                pos,
                angle: [0, 0, 0],
                set_id: 0,
            },
            state,
            hp,
        ))
        .id()
}

fn foe_hp(app: &App, entity: bevy_ecs::entity::Entity) -> i32 {
    app.sim.world.entity(entity).get::<HitPoints>().unwrap().0
}

#[test]
fn golden_bomb_bag_fills_the_hand() {
    let mut app = App::new();
    assert_eq!(app.held(), (0, 0), "you start with empty hands");
    hold(&mut app, BOMB_ITEM);
    assert_eq!(
        app.held(),
        (item::FIRST_BOMB, 10),
        "the 10-bomb bag equips bombs"
    );
    assert_eq!(app.sim.world.resource::<Held>().kind(), Some(Kind::Bomb));
}

#[test]
fn golden_arrow_bag_replaces_the_bombs() {
    let mut app = App::new();
    hold(&mut app, BOMB_ITEM);
    hold(&mut app, ARROW_ITEM);
    assert_eq!(
        app.held(),
        (item::FIRST_ARROW, 10),
        "the last bag in hand wins"
    );
}

#[test]
fn golden_use_button_throws_a_bomb_and_spends_one() {
    let mut app = App::new();
    hold(&mut app, BOMB_ITEM);
    use_button(&mut app);
    assert_eq!(bombs(&app).len(), 1, "the bag empties one bomb");
    assert_eq!(app.held().1, 9, "and counts it gone");
    use_button(&mut app);
    assert_eq!(bombs(&app).len(), 2, "each press throws one more");
}

#[test]
fn golden_empty_hands_throw_nothing() {
    let mut app = App::new();
    use_button(&mut app);
    assert!(bombs(&app).is_empty(), "no bag, no bomb");
    assert!(arrows(&app).is_empty(), "no bag, no arrow");
}

#[test]
fn golden_bomb_fuses_and_bursts() {
    let mut app = App::new();
    hold(&mut app, BOMB_ITEM);
    use_button(&mut app);
    let thrown = bombs(&app)[0];
    for _ in 0..thrown.fuse {
        tick(&mut app);
    }
    assert!(
        bombs(&app).is_empty(),
        "the fuse runs out and the bomb is gone"
    );
}

#[test]
fn golden_bomb_blast_takes_a_heart_and_a_half() {
    let mut app = App::new();
    hold(&mut app, BOMB_ITEM);
    let victim = foe(&mut app, Vec3::new(0.0, 0.0, 300.0));
    use_button(&mut app);
    for _ in 0..100 {
        tick(&mut app);
    }
    assert_eq!(
        foe_hp(&app, victim),
        1,
        "a blast of four quarters halves a five-heart foe"
    );
    assert_eq!(app.health().units, 8, "and hurts the bomber too");
}

#[test]
fn golden_bomb_chain_sets_off_the_next_one() {
    let mut app = App::new();
    hold(&mut app, BOMB_ITEM);
    use_button(&mut app);
    tick(&mut app);
    app.sim.world.spawn(Bomb {
        pos: player(&app).pos + Vec3::new(0.0, 0.0, 120.0),
        fuse: 100,
        vel: Vec3::ZERO,
        rest: true,
    });
    for _ in 0..100 {
        tick(&mut app);
    }
    assert!(
        bombs(&app).is_empty(),
        "the second bomb goes off with the first"
    );
}

#[test]
fn golden_arrow_sticks_when_it_misses() {
    let mut app = App::new();
    hold(&mut app, ARROW_ITEM);
    use_button(&mut app);
    assert_eq!(arrows(&app).len(), 1, "the bow looses one shaft");
    for _ in 0..6 {
        tick(&mut app);
    }
    assert!(arrows(&app).is_empty(), "it lands and stays put");
}

#[test]
fn golden_arrow_finishes_a_foe() {
    let mut app = App::new();
    hold(&mut app, ARROW_ITEM);
    let victim = foe(&mut app, Vec3::new(0.0, 0.0, 400.0));
    use_button(&mut app);
    tick(&mut app);
    tick(&mut app);
    assert_eq!(
        foe_hp(&app, victim),
        3,
        "an arrow bites two quarters out of a five-heart foe"
    );
    for _ in 0..4 {
        tick(&mut app);
    }
    assert_eq!(foe_hp(&app, victim), 1, "and only that once");
}

#[test]
fn golden_arrow_runs_out() {
    let mut app = App::new();
    hold(&mut app, ARROW_ITEM);
    for _ in 0..10 {
        use_button(&mut app);
    }
    assert_eq!(app.held().1, 0, "the quiver empties");
    use_button(&mut app);
    assert!(arrows(&app).is_empty(), "an empty quiver fires nothing");
}

#[test]
fn golden_items_survive_a_reload() {
    let mut app = App::new();
    hold(&mut app, BOMB_ITEM);
    let save = app.save();
    let mut restored = App::new();
    restored.load(&save).unwrap();
    assert_eq!(
        restored.held(),
        (item::FIRST_BOMB, 10),
        "the bag rides along in the save"
    );
}

#[test]
fn golden_boomerang_comes_back_to_the_hand() {
    let mut app = App::new();
    hold(&mut app, item::BOOMERANG);
    use_button(&mut app);
    assert_eq!(spins(&app).len(), 1, "the spin leaves the hand");
    assert_eq!(app.held().1, 1, "and the hand keeps it");
    for _ in 0..400 {
        tick(&mut app);
    }
    assert!(spins(&app).is_empty(), "the spin comes home");
    assert_eq!(app.held(), (item::BOOMERANG, 1), "still in hand");
}

#[test]
fn golden_boomerang_bites_two_foes_and_returns() {
    let mut app = App::new();
    hold(&mut app, item::BOOMERANG);
    let first = foe(&mut app, Vec3::new(0.0, 0.0, 150.0));
    let second = foe(&mut app, Vec3::new(0.0, 0.0, 400.0));
    use_button(&mut app);
    for _ in 0..30 {
        tick(&mut app);
    }
    assert_eq!(foe_hp(&app, first), 4, "the first foe loses a quarter");
    assert_eq!(foe_hp(&app, second), 4, "so does the second");
    for _ in 0..40 {
        tick(&mut app);
    }
    assert!(spins(&app).is_empty(), "two locks and it turns for home");
}

#[test]
fn golden_boomerang_recalls_on_the_second_press() {
    let mut app = App::new();
    hold(&mut app, item::BOOMERANG);
    use_button(&mut app);
    assert!(!spins(&app)[0].returning, "it flies out first");
    use_button(&mut app);
    assert!(spins(&app)[0].returning, "a second press calls it back");
    for _ in 0..20 {
        tick(&mut app);
    }
    assert!(spins(&app).is_empty(), "and it is home");
}
