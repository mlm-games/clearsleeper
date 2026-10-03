use std::time::Duration;

use bevy_ecs::prelude::Entity;
use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::combat::Combat;
use clearsleeper::sim::health::Health;
use clearsleeper::sim::pickup::{self, Pickup, Rupees};
use clearsleeper::sim::player::Player;
use clearsleeper::{App, SIM_HZ};
use glam::Vec3;

const HEART: u8 = 0x00;
const BLUE_RUPEE: u8 = 0x02;
const RED_RUPEE: u8 = 0x04;
const HEART_PIECE: u8 = 0x07;
const HEART_CONTAINER: u8 = 0x08;

fn tick(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

fn teleport(app: &mut App, pos: Vec3) {
    let mut query = app.sim.world.query::<&mut Player>();
    let mut player = query.iter_mut(&mut app.sim.world).next().unwrap();
    player.pos = pos;
    player.yaw = 0.0;
}

fn spawn_pickup(app: &mut App, pos: Vec3, item: u8) -> Entity {
    app.sim
        .world
        .spawn((
            Actor {
                name: pickup::ITEM_ACTORS[0].to_owned(),
                parameters: u32::from(item),
                pos,
                angle: [0; 3],
                set_id: 0,
            },
            Pickup::spawn(u32::from(item)),
        ))
        .id()
}

fn held(app: &App, entity: Entity) -> bool {
    app.sim.world.get::<Pickup>(entity).is_some()
}

fn health(app: &App) -> Health {
    *app.sim.world.resource::<Health>()
}

fn rupees(app: &App) -> u32 {
    app.sim.world.resource::<Rupees>().count
}

fn wound(app: &mut App, units: u8) {
    app.sim.world.insert_resource(Health { units, max: 12 });
}

#[test]
fn golden_heart_waits_out_the_spawn_lock() {
    let mut app = App::new();
    let heart = spawn_pickup(&mut app, Vec3::new(0.0, 0.0, 25.0), HEART);
    teleport(&mut app, Vec3::new(0.0, 0.0, 0.0));
    for _ in 0..15 {
        tick(&mut app);
    }
    assert!(held(&app, heart), "the first fifteen ticks are locked");
    tick(&mut app);
    assert!(!held(&app, heart), "the sixteenth tick takes the heart");
    assert_eq!(health(&app).units, 12, "a full meter cannot be poured into");
}

#[test]
fn golden_heart_heals_a_whole_heart() {
    let mut app = App::new();
    wound(&mut app, 6);
    spawn_pickup(&mut app, Vec3::new(0.0, 0.0, 25.0), HEART);
    teleport(&mut app, Vec3::new(0.0, 0.0, 0.0));
    for _ in 0..16 {
        tick(&mut app);
    }
    assert_eq!(health(&app).units, 10, "the heart is worth four quarters");
}

#[test]
fn golden_heart_piece_raises_the_ceiling() {
    let mut app = App::new();
    let piece = spawn_pickup(&mut app, Vec3::new(0.0, 0.0, 25.0), HEART_PIECE);
    teleport(&mut app, Vec3::new(0.0, 0.0, 0.0));
    for _ in 0..16 {
        tick(&mut app);
    }
    assert!(!held(&app, piece));
    let state = health(&app);
    assert_eq!(state.max, 13, "the piece lifts the ceiling a quarter");
    assert_eq!(
        state.units, 12,
        "a quarter past the last whole heart heals nothing"
    );
}

#[test]
fn golden_heart_container_fills_the_meter() {
    let mut app = App::new();
    wound(&mut app, 5);
    spawn_pickup(&mut app, Vec3::new(0.0, 0.0, 25.0), HEART_CONTAINER);
    teleport(&mut app, Vec3::new(0.0, 0.0, 0.0));
    for _ in 0..16 {
        tick(&mut app);
    }
    let state = health(&app);
    assert_eq!(state.max, 16, "the container is worth a whole heart");
    assert_eq!(state.units, 16, "the meter fills to the new ceiling");
}

#[test]
fn golden_rupees_stack_up() {
    let mut app = App::new();
    let blue = spawn_pickup(&mut app, Vec3::new(0.0, 0.0, 25.0), BLUE_RUPEE);
    let red = spawn_pickup(&mut app, Vec3::new(0.0, 0.0, -25.0), RED_RUPEE);
    teleport(&mut app, Vec3::new(0.0, 0.0, 0.0));
    for _ in 0..16 {
        tick(&mut app);
    }
    assert!(!held(&app, blue));
    assert!(!held(&app, red));
    assert_eq!(rupees(&app), 25, "five blue and twenty red");
}

#[test]
fn golden_pickups_stay_out_of_reach() {
    let mut app = App::new();
    wound(&mut app, 6);
    let heart = spawn_pickup(&mut app, Vec3::new(0.0, 0.0, 100.0), HEART);
    teleport(&mut app, Vec3::new(0.0, 0.0, 0.0));
    for _ in 0..60 {
        tick(&mut app);
    }
    assert!(held(&app, heart), "past thirty units the heart waits");
    assert_eq!(health(&app).units, 6);
}

#[test]
fn golden_sword_leaves_pickups_alone() {
    let mut app = App::new();
    let heart = spawn_pickup(&mut app, Vec3::new(0.0, 0.0, 20.0), HEART);
    teleport(&mut app, Vec3::new(0.0, 0.0, 0.0));
    app.stage_item(true);
    tick(&mut app);
    app.stage_item(false);
    for _ in 0..6 {
        tick(&mut app);
    }
    assert!(
        app.sim.world.resource::<Combat>().active,
        "the cut is still out"
    );
    assert!(held(&app, heart), "the blade cannot break a heart");
}
