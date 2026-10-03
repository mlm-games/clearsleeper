use std::time::Duration;

use bevy_ecs::prelude::Entity;
use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::combat::{Combat, HitPoints};
use clearsleeper::sim::enemy::{self, Enemy, EnemyState};
use clearsleeper::sim::health::Health;
use clearsleeper::sim::player::Player;
use clearsleeper::{App, SIM_HZ};
use glam::Vec3;

const WANDERER: u32 = 0;
const STRONGER: u32 = 0x20;
const GUARD: u32 = 0x1404;
const PLANTED: u32 = 0x320B;

fn tick(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

fn teleport(app: &mut App, pos: Vec3) {
    let mut query = app.sim.world.query::<&mut Player>();
    let mut player = query.iter_mut(&mut app.sim.world).next().unwrap();
    player.pos = pos;
    player.yaw = 0.0;
}

fn spawn_enemy(app: &mut App, pos: Vec3, yaw: i16, parameters: u32) -> Entity {
    let (state, hit_points) = enemy::spawn(parameters, yaw);
    app.sim
        .world
        .spawn((
            Actor {
                name: enemy::ENEMY_ACTOR.to_owned(),
                parameters,
                pos,
                angle: [0, yaw, 0],
                set_id: 0,
            },
            state,
            hit_points,
        ))
        .id()
}

fn enemy(app: &App, entity: Entity) -> Enemy {
    *app.sim.world.get::<Enemy>(entity).expect("enemy component")
}

fn state(app: &App, entity: Entity) -> EnemyState {
    enemy(app, entity).state
}

fn pos(app: &App, entity: Entity) -> Vec3 {
    app.sim.world.get::<Actor>(entity).expect("actor").pos
}

fn hp(app: &App, entity: Entity) -> Option<i32> {
    app.sim.world.get::<HitPoints>(entity).map(|hit| hit.0)
}

fn hearts(app: &App) -> u8 {
    app.sim.world.resource::<Health>().units
}

#[test]
fn golden_sight_gates_on_the_staged_band() {
    let mut app = App::new();
    let foe = spawn_enemy(&mut app, Vec3::new(0.0, 0.0, 0.0), 0, GUARD);
    teleport(&mut app, Vec3::new(0.0, 0.0, 300.0));
    for _ in 0..60 {
        tick(&mut app);
    }
    assert_eq!(
        state(&app, foe),
        EnemyState::Wait,
        "the band closes past 200"
    );
    assert_eq!(enemy(&app, foe).sight, 200.0);
    teleport(&mut app, Vec3::new(0.0, 0.0, 150.0));
    tick(&mut app);
    assert_eq!(
        state(&app, foe),
        EnemyState::Alert,
        "the guard notices inside 200"
    );
    for _ in 0..30 {
        tick(&mut app);
    }
    assert_eq!(
        state(&app, foe),
        EnemyState::Chase,
        "a full beat of surprise passes before the charge"
    );
}

#[test]
fn golden_sight_misses_from_directly_behind() {
    let mut app = App::new();
    let foe = spawn_enemy(&mut app, Vec3::new(0.0, 0.0, 0.0), 0, GUARD);
    teleport(&mut app, Vec3::new(0.0, 0.0, -150.0));
    for _ in 0..60 {
        tick(&mut app);
    }
    assert_eq!(
        state(&app, foe),
        EnemyState::Wait,
        "past 125 units of rear the guard sees nothing"
    );
    assert_eq!(hearts(&app), 12);
}

#[test]
fn golden_sight_catches_a_close_rear() {
    let mut app = App::new();
    let foe = spawn_enemy(&mut app, Vec3::new(0.0, 0.0, 0.0), 0, GUARD);
    teleport(&mut app, Vec3::new(0.0, 0.0, -100.0));
    tick(&mut app);
    assert_eq!(
        state(&app, foe),
        EnemyState::Alert,
        "within 125 units of the back the body still catches the player"
    );
}

#[test]
fn golden_chase_closes_at_the_quartered_run() {
    let mut app = App::new();
    let foe = spawn_enemy(&mut app, Vec3::new(0.0, 0.0, 0.0), 0, WANDERER);
    teleport(&mut app, Vec3::new(0.0, 0.0, 400.0));
    for _ in 0..31 {
        tick(&mut app);
    }
    assert_eq!(state(&app, foe), EnemyState::Chase);
    for _ in 0..5 {
        tick(&mut app);
    }
    let run = pos(&app, foe).z;
    assert!(
        (run - 112.5).abs() < 0.001,
        "the run nets 22.5 units a tick, found 112.5 expected"
    );
}

#[test]
fn golden_planted_variant_ramps_up() {
    let mut app = App::new();
    let foe = spawn_enemy(&mut app, Vec3::new(0.0, 0.0, 0.0), 0, PLANTED);
    teleport(&mut app, Vec3::new(0.0, 0.0, 400.0));
    for _ in 0..31 {
        tick(&mut app);
    }
    assert_eq!(state(&app, foe), EnemyState::Chase);
    assert!(!enemy(&app, foe).armed, "the planted variant fights bare");
    for _ in 0..5 {
        tick(&mut app);
    }
    let run = pos(&app, foe).z;
    assert!(
        (run - 18.75).abs() < 0.001,
        "the bare gait ramps five units a tick, found 18.75 expected"
    );
}

#[test]
fn golden_sword_combo_costs_a_quarter_and_cools_down() {
    let mut app = App::new();
    let foe = spawn_enemy(&mut app, Vec3::new(0.0, 0.0, 0.0), 0, WANDERER);
    teleport(&mut app, Vec3::new(0.0, 0.0, 60.0));
    for _ in 0..31 {
        tick(&mut app);
    }
    assert_eq!(state(&app, foe), EnemyState::Chase);
    tick(&mut app);
    assert_eq!(
        state(&app, foe),
        EnemyState::Attack,
        "the charge closes inside 240 and swings"
    );
    for _ in 0..43 {
        tick(&mut app);
    }
    assert_eq!(
        state(&app, foe),
        EnemyState::Attack,
        "the combo runs 44 ticks"
    );
    tick(&mut app);
    assert_eq!(
        state(&app, foe),
        EnemyState::Chase,
        "the combo ends in a charge"
    );
    assert_eq!(hearts(&app), 11, "one window reach costs a quarter heart");
    for _ in 0..10 {
        tick(&mut app);
    }
    assert_eq!(
        state(&app, foe),
        EnemyState::Chase,
        "the breath after a swing holds off the next blow"
    );
    assert_eq!(hearts(&app), 11, "nothing lands while the foe breathes");
}

#[test]
fn golden_planted_kick_lands_early() {
    let mut app = App::new();
    let foe = spawn_enemy(&mut app, Vec3::new(0.0, 0.0, 0.0), 0, PLANTED);
    teleport(&mut app, Vec3::new(0.0, 0.0, 60.0));
    for _ in 0..62 {
        tick(&mut app);
    }
    assert_eq!(
        state(&app, foe),
        EnemyState::Attack,
        "the kick runs 31 ticks"
    );
    tick(&mut app);
    assert_eq!(state(&app, foe), EnemyState::Chase);
    assert_eq!(
        hearts(&app),
        11,
        "the bare kick opens its window on the first ticks"
    );
}

#[test]
fn golden_chase_gives_up_once_it_has_closed() {
    let mut app = App::new();
    let foe = spawn_enemy(&mut app, Vec3::new(0.0, 0.0, 0.0), 0, WANDERER);
    teleport(&mut app, Vec3::new(0.0, 0.0, 400.0));
    for _ in 0..31 {
        tick(&mut app);
    }
    tick(&mut app);
    assert_eq!(state(&app, foe), EnemyState::Chase);
    teleport(&mut app, Vec3::new(0.0, 0.0, 1500.0));
    tick(&mut app);
    assert_eq!(
        state(&app, foe),
        EnemyState::Wait,
        "past 475 units the chase is given up"
    );
    for _ in 0..10 {
        tick(&mut app);
    }
    assert_eq!(
        state(&app, foe),
        EnemyState::Wait,
        "the foe rests where it stood"
    );
}

#[test]
fn golden_sword_breaks_the_foe() {
    let mut app = App::new();
    let foe = spawn_enemy(&mut app, Vec3::new(0.0, 0.0, 30.0), 0, STRONGER);
    teleport(&mut app, Vec3::new(0.0, 0.0, 0.0));
    assert_eq!(
        hp(&app, foe),
        Some(7),
        "the strong cut carries seven points"
    );
    app.stage_item(true);
    tick(&mut app);
    app.stage_item(false);
    for _ in 0..6 {
        tick(&mut app);
    }
    assert_eq!(
        hp(&app, foe),
        Some(2),
        "one slash leaves two points standing"
    );
    assert_eq!(
        state(&app, foe),
        EnemyState::Alert,
        "the foe is still surprised"
    );
    for _ in 0..7 {
        tick(&mut app);
    }
    assert!(
        !app.sim.world.resource::<Combat>().active,
        "the cut runs its full length"
    );
    app.stage_item(true);
    tick(&mut app);
    app.stage_item(false);
    for _ in 0..2 {
        tick(&mut app);
    }
    assert_eq!(hp(&app, foe), None, "the second slash breaks the foe");
    assert_eq!(
        hearts(&app),
        12,
        "the surprised foe never gets its own swing in"
    );
}
