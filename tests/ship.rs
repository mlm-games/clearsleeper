use std::time::Duration;

use clearsleeper::SIM_HZ;
use clearsleeper::dzs;
use clearsleeper::sim;
use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::intent::{DoIntent, MoveIntent};
use clearsleeper::sim::player::Player;
use clearsleeper::sim::ship::Ship;
use glam::{Vec2, Vec3};
use repame_sim::Sim;

const SHIP_X: f32 = 0.0;
const SHIP_Z: f32 = 0.0;

fn ship_stage() -> Vec<u8> {
    let actr_at = 4 + 12;
    let mut data = vec![0_u8; actr_at + 0x20];
    write_u32(&mut data, 0, 1);
    data[4..8].copy_from_slice(b"ACTR");
    write_u32(&mut data, 8, 1);
    write_u32(&mut data, 12, actr_at as u32);
    data[actr_at..actr_at + 8].copy_from_slice(b"Ship\0\0\0\0");
    data[actr_at + 0x0C..actr_at + 0x10].copy_from_slice(&SHIP_X.to_be_bytes());
    data[actr_at + 0x10..actr_at + 0x14].copy_from_slice(&0.0_f32.to_be_bytes());
    data[actr_at + 0x14..actr_at + 0x18].copy_from_slice(&SHIP_Z.to_be_bytes());
    data
}

fn write_u32(data: &mut [u8], at: usize, value: u32) {
    data[at..at + 4].copy_from_slice(&value.to_be_bytes());
}

fn fresh() -> Sim {
    let tables = dzs::parse(&ship_stage()).expect("stage tables");
    let mut sim = Sim::new(Duration::from_secs_f64(1.0 / SIM_HZ));
    sim::build(&mut sim, None, Some(&tables));
    teleport(&mut sim, Vec3::new(SHIP_X, 0.0, SHIP_Z));
    sim
}

fn teleport(sim: &mut Sim, pos: Vec3) {
    let mut query = sim.world.query::<&mut Player>();
    let mut player = query.iter_mut(&mut sim.world).next().expect("player");
    player.pos = pos;
    player.yaw = 0.0;
}

fn player(sim: &Sim) -> Player {
    sim.world
        .iter_entities()
        .find_map(|entity| entity.get::<Player>().copied())
        .expect("player")
}

fn ship(sim: &Sim) -> Ship {
    sim.world
        .iter_entities()
        .find_map(|entity| entity.get::<Ship>().copied())
        .expect("ship")
}

fn actor(sim: &Sim) -> Actor {
    sim.world
        .iter_entities()
        .find_map(|entity| entity.get::<Actor>().cloned())
        .expect("stage ship actor")
}

fn axis(sim: &mut Sim, axis: Vec2) {
    sim.world.resource_mut::<MoveIntent>().axis = axis;
}

fn do_press(sim: &mut Sim, held: bool) {
    sim.world.resource_mut::<DoIntent>().0 = held;
}

fn units(value: i16) -> f32 {
    value as f32 * (std::f32::consts::TAU / 65536.0)
}

#[test]
fn golden_ship_spawns_from_stage_actor() {
    let sim = fresh();
    let actor = actor(&sim);
    assert_eq!(actor.name, "Ship");
    assert_eq!(actor.pos, Vec3::new(SHIP_X, 0.0, SHIP_Z));
    assert_eq!(actor.angle, [0, 0, 0]);
    let ship = ship(&sim);
    assert_eq!(ship.speed, 0.0);
    assert_eq!(ship.tiller, 0);
    assert!(!ship.riding);
}

#[test]
fn golden_board_and_carry() {
    let mut sim = fresh();
    do_press(&mut sim, true);
    sim.tick();

    assert!(ship(&sim).riding, "a do press next to the hull boards");
    let p = player(&sim);
    assert_eq!(p.pos.x, SHIP_X);
    assert_eq!(p.pos.y, 15.0, "the ride offset holds the deck height");
    assert_eq!(p.pos.z, SHIP_Z - 35.0);
    assert_eq!(p.yaw, 0.0, "the rider takes the ship heading");
    assert_eq!(p.speed_y, 0.0, "gravity pauses aboard");
}

#[test]
fn golden_steer_converts_tiller_to_yaw() {
    let mut sim = fresh();
    do_press(&mut sim, true);
    sim.tick();
    do_press(&mut sim, false);
    axis(&mut sim, Vec2::new(1.0, 0.0));
    sim.tick();

    let ship = ship(&sim);
    assert!(ship.riding);
    assert_eq!(ship.tiller, 700, "the tiller chases stick right");
    assert_eq!(ship.speed, 0.0, "sideways stick rows nothing");
    let actor = actor(&sim);
    assert_eq!(actor.angle[1], -36, "tiller turns the hull");
    let p = player(&sim);
    let yaw = units(-36);
    assert_eq!(p.yaw, yaw, "the rider mirrors the new heading");
    assert_eq!(
        p.pos.x,
        SHIP_X + 0.0 * yaw.cos() + -35.0 * yaw.sin(),
        "the carry offset rotates with the hull"
    );
}

#[test]
fn golden_row_accelerates_ship() {
    let mut sim = fresh();
    do_press(&mut sim, true);
    sim.tick();
    do_press(&mut sim, false);
    axis(&mut sim, Vec2::new(0.0, 1.0));
    sim.tick();

    assert_eq!(ship(&sim).speed, 0.1, "one row tick reaches the first step");
    let hull = actor(&sim);
    assert_eq!(
        hull.pos.z,
        SHIP_Z + 0.1,
        "the hull slides along its heading"
    );
    assert_eq!(hull.pos.x, SHIP_X, "no sideways drift at zero yaw");
    let p = player(&sim);
    assert_eq!(p.pos.z, SHIP_Z + 0.1 - 35.0, "the rider stays glued");

    sim.tick();
    assert_eq!(ship(&sim).speed, 0.2, "the second row tick keeps chasing");
    let hull = actor(&sim);
    assert_eq!(hull.pos.z, SHIP_Z + 0.1 + 0.2);
}

#[test]
fn golden_get_off_at_low_speed() {
    let mut sim = fresh();
    do_press(&mut sim, true);
    sim.tick();
    do_press(&mut sim, false);
    sim.tick();
    do_press(&mut sim, true);
    sim.tick();

    assert!(!ship(&sim).riding, "a slow hull lets the rider step off");
    let yaw = units(-0x4000);
    let p = player(&sim);
    assert_eq!(p.yaw, yaw, "the rider faces the ship's side");
    assert_eq!(p.speed, 8.0, "the dismount carries the dash speed");
    assert_eq!(p.speed_y, 0.0);
    assert_eq!(p.pos.x, SHIP_X + yaw.sin() * 57.0, "planted on the ledge");
    assert_eq!(p.pos.y, 35.0);
    assert_eq!(p.pos.z, SHIP_Z - 35.0 + yaw.cos() * 57.0);
}

#[test]
fn golden_idle_ship_decays_to_rest() {
    let mut sim = fresh();
    do_press(&mut sim, true);
    sim.tick();
    do_press(&mut sim, false);
    axis(&mut sim, Vec2::new(0.0, 1.0));
    sim.tick();
    assert_eq!(ship(&sim).speed, 0.1);

    axis(&mut sim, Vec2::ZERO);
    do_press(&mut sim, true);
    sim.tick();
    let state = ship(&sim);
    assert!(!state.riding, "low speed allows leaving the helm");
    assert_eq!(state.speed, 0.1, "leaving keeps the drift");

    do_press(&mut sim, false);
    sim.tick();
    assert_eq!(ship(&sim).speed, 0.05, "idle chase trims the drift");
    sim.tick();
    assert_eq!(ship(&sim).speed, 0.0, "the drift snaps to rest");
}

#[test]
fn golden_far_teleport_drops_the_rider() {
    let mut sim = fresh();
    do_press(&mut sim, true);
    sim.tick();
    assert!(ship(&sim).riding);

    teleport(&mut sim, Vec3::new(SHIP_X + 500.0, 0.0, SHIP_Z));
    do_press(&mut sim, false);
    sim.tick();
    assert!(!ship(&sim).riding, "a rider pulled away lets go");
    assert_eq!(
        player(&sim).pos.x,
        SHIP_X + 500.0,
        "the dropped rider stays put"
    );
}
