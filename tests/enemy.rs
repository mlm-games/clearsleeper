use std::time::Duration;

use bevy_ecs::prelude::Entity;
use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::combat::HitPoints;
use clearsleeper::sim::enemy::{self, Enemy, EnemyState, Hurt, Spear};
use clearsleeper::sim::health::Health;
use clearsleeper::sim::player::Player;
use clearsleeper::{App, SIM_HZ};
use glam::Vec3;
use repose_core::input::PhysicalKey;
use repose_core::runtime::Scheduler;

const WANDERER: u32 = 0;
const STRONGER: u32 = 0x20;
const GUARD: u32 = 0x1404;

fn tick(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
    let mut query = app.sim.world.query::<&mut Player>();
    for mut player in query.iter_mut(&mut app.sim.world) {
        player.pos.y = 0.0;
    }
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
                room: -1,
            },
            state,
            hit_points,
        ))
        .id()
}

fn hurt(app: &App, entity: Entity) -> u32 {
    app.sim.world.entity(entity).get::<Hurt>().unwrap().0
}

fn armed(app: &App, entity: Entity) -> bool {
    app.sim.world.entity(entity).get::<Enemy>().unwrap().armed
}

fn swing(app: &mut App) {
    app.stage_polled(&keys(&[PhysicalKey::KeyB]));
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
    app.stage_polled(&keys(&[]));
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

fn keys(held: &[PhysicalKey]) -> Scheduler {
    let mut sched = Scheduler::default();
    sched.window_focused = true;
    sched.held_keys = held.iter().cloned().collect();
    sched
}

fn spawn_lance(app: &mut App, pos: Vec3, yaw: i16) -> Entity {
    let (state, hit_points) = enemy::spawn_lance(0, yaw).expect("a standing lance");
    app.sim
        .world
        .spawn((
            Actor {
                name: enemy::LANCE_ACTOR.to_owned(),
                parameters: 0,
                pos,
                angle: [0, yaw, 0],
                set_id: 0,
                room: -1,
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

fn spawn_lance_with_hurt(app: &mut App, pos: Vec3, yaw: i16) -> Entity {
    let (state, hit_points) = enemy::spawn_lance(0, yaw).expect("a standing lance");
    app.sim
        .world
        .spawn((
            Actor {
                name: enemy::LANCE_ACTOR.to_owned(),
                parameters: 0,
                pos,
                angle: [0, yaw, 0],
                set_id: 0,
                room: -1,
            },
            state,
            hit_points,
            Hurt::default(),
        ))
        .id()
}

fn spears(app: &App) -> Vec<Spear> {
    app.sim
        .world
        .iter_entities()
        .filter_map(|e| e.get::<Spear>().copied())
        .collect()
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
        (run - 22.5 * 5.0).abs() < 0.001,
        "the run nets 22.5 units a tick, found {run}"
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
fn golden_chase_gives_up_once_it_has_closed() {
    let mut app = App::new();
    let foe = spawn_enemy(&mut app, Vec3::new(0.0, 0.0, 0.0), 0, WANDERER);
    teleport(&mut app, Vec3::new(0.0, 0.0, 400.0));
    for _ in 0..31 {
        tick(&mut app);
    }
    tick(&mut app);
    assert_eq!(state(&app, foe), EnemyState::Chase);
    teleport(&mut app, Vec3::new(0.0, 0.0, 600.0));
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
        Some(2),
        "two power is one point past the kill threshold"
    );
    app.stage_item(true);
    tick(&mut app);
    app.stage_item(false);
    for _ in 0..6 {
        tick(&mut app);
    }
    assert_eq!(hp(&app, foe), None, "one slash breaks the foe");
    assert_eq!(
        hearts(&app),
        12,
        "the surprised foe never gets its own swing in"
    );
}

#[test]
fn golden_lance_spots_farther_than_a_boko() {
    let mut app = App::new();
    let foe = spawn_lance(&mut app, Vec3::new(0.0, 0.0, 0.0), 0);
    teleport(&mut app, Vec3::new(0.0, 0.0, 600.0));
    tick(&mut app);
    assert_eq!(
        state(&app, foe),
        EnemyState::Alert,
        "the lance watches 1250 units out, past a boko's 1000"
    );
}

#[test]
fn golden_lance_thrust_costs_a_quarter() {
    let mut app = App::new();
    let foe = spawn_lance(&mut app, Vec3::new(0.0, 0.0, 0.0), 0);
    teleport(&mut app, Vec3::new(0.0, 0.0, 120.0));
    for _ in 0..31 {
        tick(&mut app);
    }
    assert_eq!(state(&app, foe), EnemyState::Chase);
    tick(&mut app);
    assert_eq!(
        state(&app, foe),
        EnemyState::Attack,
        "inside 312 units the lance commits"
    );
    for _ in 0..45 {
        tick(&mut app);
    }
    assert_eq!(
        state(&app, foe),
        EnemyState::Attack,
        "the thrust runs 46 ticks"
    );
    tick(&mut app);
    assert_eq!(state(&app, foe), EnemyState::Chase);
    assert_eq!(hearts(&app), 11, "the spear point takes a quarter heart");
}

#[test]
fn golden_hit_flattens_a_foe_for_a_few_ticks() {
    let mut app = App::new();
    let foe = spawn_lance_with_hurt(&mut app, Vec3::new(0.0, 0.0, 0.0), 0);
    teleport(&mut app, Vec3::new(0.0, 0.0, 120.0));
    for _ in 0..31 {
        tick(&mut app);
    }
    assert_eq!(state(&app, foe), EnemyState::Chase);
    tick(&mut app);
    assert_eq!(state(&app, foe), EnemyState::Attack, "it commits");
    swing(&mut app);
    assert_eq!(
        state(&app, foe),
        EnemyState::Wait,
        "the blade stops it cold"
    );
    assert_eq!(hurt(&app, foe), 9, "and the stagger runs ten ticks");
    for _ in 0..8 {
        tick(&mut app);
    }
    assert_eq!(hurt(&app, foe), 0, "then it shakes it off");
    assert_eq!(state(&app, foe), EnemyState::Chase, "and comes back at you");
}

#[test]
fn golden_lance_throws_its_spear_when_struck_at_range() {
    let mut app = App::new();
    let foe = spawn_lance_with_hurt(&mut app, Vec3::new(0.0, 0.0, 0.0), 0);
    teleport(&mut app, Vec3::new(0.0, 0.0, 600.0));
    tick(&mut app);
    app.sim
        .world
        .entity_mut(foe)
        .get_mut::<Hurt>()
        .expect("foes flinch")
        .0 = 1;
    tick(&mut app);
    assert_eq!(spears(&app).len(), 1, "the spear goes overhand");
    assert!(!armed(&app, foe), "and the lance is bare for it now");
}

#[test]
fn golden_thrown_spear_costs_a_quarter() {
    let mut app = App::new();
    let foe = spawn_lance_with_hurt(&mut app, Vec3::new(0.0, 0.0, 0.0), 0);
    teleport(&mut app, Vec3::new(0.0, 0.0, 600.0));
    tick(&mut app);
    app.sim
        .world
        .entity_mut(foe)
        .get_mut::<Hurt>()
        .expect("foes flinch")
        .0 = 1;
    tick(&mut app);
    for _ in 0..40 {
        tick(&mut app);
    }
    assert_eq!(hearts(&app), 11, "the spear point takes a quarter heart");
    assert!(spears(&app).is_empty(), "and it is spent on the ground");
}

#[test]
fn golden_open_water_holds_enemies_where_they_stand() {
    let mut app = App::new();
    app.sim
        .world
        .remove_resource::<clearsleeper::sim::collision::Collision>();
    let foe = spawn_lance_with_hurt(&mut app, Vec3::new(0.0, 0.0, 0.0), 0);
    teleport(&mut app, Vec3::new(0.0, 0.0, 600.0));
    for _ in 0..40 {
        tick(&mut app);
    }
    let actor = app
        .sim
        .world
        .entity(foe)
        .get::<Actor>()
        .expect("the foe is placed");
    assert_eq!(
        actor.pos.z, 0.0,
        "with no floor under it, it holds its post"
    );
    assert_eq!(state(&app, foe), EnemyState::Wait, "and stays alert-free");
}
