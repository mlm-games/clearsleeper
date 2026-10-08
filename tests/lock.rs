use std::f32::consts::FRAC_PI_2;
use std::f32::consts::PI;
use std::f32::consts::TAU;
use std::time::Duration;

use bevy_ecs::prelude::Entity;
use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::jump::{Jump, JumpKind};
use clearsleeper::sim::lock::Lock;
use clearsleeper::sim::player::Player;
use clearsleeper::sim::roll::Roll;
use clearsleeper::sim::ship::Ship;
use clearsleeper::{App, SIM_HZ};
use glam::{Vec2, Vec3};

fn tick(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

fn player(app: &App) -> Player {
    app.sim
        .world
        .iter_entities()
        .find_map(|entity| entity.get::<Player>().copied())
        .expect("player entity")
}

fn spawn_actor(app: &mut App, pos: Vec3) -> Entity {
    app.sim
        .world
        .spawn(Actor {
            name: "Pot".to_owned(),
            parameters: 0,
            pos,
            angle: [0; 3],
            set_id: 0,
            room: -1,
            ..Default::default()
        })
        .id()
}

fn teleport(app: &mut App, pos: Vec3) {
    let mut query = app.sim.world.query::<&mut Player>();
    let mut player = query.iter_mut(&mut app.sim.world).next().unwrap();
    player.pos = pos;
    player.yaw = 0.0;
}

fn target(app: &App) -> Option<Entity> {
    app.sim.world.resource::<Lock>().target
}

#[test]
fn golden_lock_engages_in_cone_and_faces_the_target() {
    let mut app = App::new();
    let actor = spawn_actor(&mut app, Vec3::new(0.0, 0.0, 100.0));
    app.stage_target(true);
    tick(&mut app);
    assert_eq!(target(&app), Some(actor), "the press grabs the actor ahead");
    let p = player(&app);
    assert!(p.face.abs() < 0.01, "the face settles on the target");
    assert!(
        app.snapshot().player_face.abs() < 0.01,
        "the frame sees the same face"
    );
}

#[test]
fn golden_lock_picks_the_nearest_candidate() {
    let mut app = App::new();
    let near = spawn_actor(&mut app, Vec3::new(0.0, 0.0, 60.0));
    spawn_actor(&mut app, Vec3::new(0.0, 0.0, 120.0));
    app.stage_target(true);
    tick(&mut app);
    assert_eq!(target(&app), Some(near), "the closest weight wins");
}

#[test]
fn golden_lock_respects_cone_reach_and_height() {
    let mut app = App::new();
    spawn_actor(&mut app, Vec3::new(100.0, 0.0, 0.0));
    app.stage_target(true);
    tick(&mut app);
    assert_eq!(target(&app), None, "the side sits outside the cone");

    app.stage_target(false);
    tick(&mut app);
    spawn_actor(&mut app, Vec3::new(0.0, 0.0, 500.0));
    app.stage_target(true);
    tick(&mut app);
    assert_eq!(target(&app), None, "the horizon sits beyond the reach");

    app.stage_target(false);
    tick(&mut app);
    spawn_actor(&mut app, Vec3::new(0.0, 150.0, 100.0));
    app.stage_target(true);
    tick(&mut app);
    assert_eq!(target(&app), None, "the ledge above stays out of reach");

    app.stage_target(false);
    tick(&mut app);
    let edge = spawn_actor(&mut app, Vec3::new(0.0, 0.0, 140.0));
    app.stage_target(true);
    tick(&mut app);
    assert_eq!(
        target(&app),
        Some(edge),
        "the near edge of the reach still grabs"
    );
}

#[test]
fn golden_lock_releases_on_button_and_on_loss() {
    let mut app = App::new();
    let actor = spawn_actor(&mut app, Vec3::new(0.0, 0.0, 100.0));
    app.stage_target(true);
    tick(&mut app);
    assert_eq!(target(&app), Some(actor));

    app.stage_target(false);
    tick(&mut app);
    assert_eq!(target(&app), None, "letting go drops the lock");

    app.stage_target(true);
    tick(&mut app);
    assert_eq!(target(&app), Some(actor), "a fresh press re-engages");

    teleport(&mut app, Vec3::new(0.0, 0.0, -200.0));
    tick(&mut app);
    assert_eq!(
        target(&app),
        None,
        "drifting past the release window drops it"
    );
}

#[test]
fn golden_lock_holds_the_face_on_the_target_while_strafing() {
    let mut app = App::new();
    let actor = spawn_actor(&mut app, Vec3::new(0.0, 0.0, 100.0));
    app.stage_target(true);
    tick(&mut app);
    assert_eq!(target(&app), Some(actor));

    app.stage_axis(Vec2::new(1.0, 0.0));
    for _ in 0..6 {
        tick(&mut app);
    }
    let p = player(&app);
    let want = (0.0 - p.pos.x).atan2(100.0 - p.pos.z);
    assert!((p.face - want).abs() < 0.05, "the face tracks the target");
    assert!(
        (p.yaw - want).abs() > 0.05,
        "the body turned with the stick instead"
    );
    assert!(p.speed > 0.5, "the strafe carries speed");
    assert_eq!(
        target(&app),
        Some(actor),
        "the lock holds through the strafe"
    );
}

#[test]
fn golden_backjump_flips_away_and_lands_facing_the_target() {
    let mut app = App::new();
    let actor = spawn_actor(&mut app, Vec3::new(0.0, 0.0, 100.0));
    app.stage_target(true);
    tick(&mut app);
    assert_eq!(target(&app), Some(actor));

    app.stage_axis(Vec2::new(0.0, -1.0));
    app.stage_do(true);
    tick(&mut app);
    let jump = *app.sim.world.resource::<Jump>();
    assert!(jump.active, "the back press takes off");
    assert_eq!(jump.kind, JumpKind::Back);
    let p = player(&app);
    assert!(
        (p.yaw - PI).abs() < 0.01,
        "the flip travels away from the target"
    );
    assert_eq!(p.speed, 22.5, "the flip runs at the backjump pace");
    assert_eq!(p.speed_y, 16.0, "the launch climbs one gravity step in");
    assert_eq!(p.pos.y, 16.0);
    assert!(
        (p.pos.z + 22.5).abs() < 0.001,
        "the first beat moves backward"
    );
    assert!(
        !app.sim.world.resource::<Roll>().active,
        "the lock eats the roll"
    );
    assert!(p.face.abs() < 0.01, "the face stays on the target mid-flip");

    app.stage_axis(Vec2::ZERO);
    app.stage_do(false);
    for _ in 0..14 {
        tick(&mut app);
    }
    assert!(
        !app.sim.world.resource::<Jump>().active,
        "the flip ends on the ground"
    );
    let p = player(&app);
    assert_eq!(p.speed, 0.0, "the landing kills momentum");
    assert_eq!(p.pos.y, -2.5, "gravity resumes its one step of sink");
    assert!(
        (p.pos.z + 270.0).abs() < 1.0,
        "the flip carries a full gap backward"
    );
    assert!(p.yaw.abs() < 0.01, "the landing turns back to the target");
    assert_eq!(
        target(&app),
        None,
        "the flip carries past the release window"
    );
}

#[test]
fn golden_side_step_hops_perpendicular_and_lands_facing_the_target() {
    let mut app = App::new();
    let actor = spawn_actor(&mut app, Vec3::new(0.0, 0.0, 100.0));
    app.stage_target(true);
    tick(&mut app);
    assert_eq!(target(&app), Some(actor));

    app.stage_axis(Vec2::new(-1.0, 0.0));
    app.stage_do(true);
    tick(&mut app);
    let jump = *app.sim.world.resource::<Jump>();
    assert!(jump.active, "the side press takes off");
    assert_eq!(jump.kind, JumpKind::Left);
    let pitch = 6200.0_f32 * (TAU / 65536.0);
    let step = pitch.cos() * 30.0;
    let rise = pitch.sin() * 30.0;
    let p = player(&app);
    assert!(
        (p.yaw - FRAC_PI_2).abs() < 0.01,
        "the hop leaves at a right angle"
    );
    assert!(
        (p.speed - step).abs() < 0.001,
        "the hop runs at the step pace"
    );
    assert!(
        (p.speed_y - (rise - 2.4)).abs() < 0.001,
        "the launch lifts one step gravity in"
    );
    assert!(
        (p.pos.x - step).abs() < 0.01,
        "the first beat crosses sideways"
    );
    assert!(
        !app.sim.world.resource::<Roll>().active,
        "the lock eats the roll"
    );

    app.stage_axis(Vec2::ZERO);
    app.stage_do(false);
    for _ in 0..16 {
        tick(&mut app);
    }
    assert!(
        !app.sim.world.resource::<Jump>().active,
        "the hop ends on the ground"
    );
    let p = player(&app);
    assert_eq!(p.speed, 0.0, "the landing kills momentum");
    assert_eq!(p.pos.y, -2.5, "gravity resumes its one step of sink");
    let moved = (p.pos.x * p.pos.x + p.pos.z * p.pos.z).sqrt();
    assert!(moved > 60.0, "the hop carries the player around");
    assert_eq!(
        target(&app),
        Some(actor),
        "the hop orbits without dropping the lock"
    );
    let want = (0.0 - p.pos.x).atan2(100.0 - p.pos.z);
    assert!(
        (p.yaw - want).abs() < 0.05,
        "the landing squares back up to the target"
    );
    assert!(
        (p.yaw - p.face).abs() < 0.001,
        "the idle player keeps facing where he looks"
    );
}

#[test]
fn golden_lock_suppresses_roll_while_locked() {
    let mut app = App::new();
    let actor = spawn_actor(&mut app, Vec3::new(0.0, 0.0, 100.0));
    app.stage_target(true);
    tick(&mut app);
    assert_eq!(target(&app), Some(actor));

    app.stage_axis(Vec2::new(0.0, 1.0));
    app.stage_do(true);
    tick(&mut app);
    assert!(
        !app.sim.world.resource::<Roll>().active,
        "no roll on the lock"
    );
    assert!(
        !app.sim.world.resource::<Jump>().active,
        "forward stays put"
    );
    assert_eq!(
        target(&app),
        Some(actor),
        "the lock holds through the press"
    );
}

#[test]
fn golden_ship_is_not_a_lock_candidate() {
    let mut app = App::new();
    app.sim.world.spawn((
        Actor {
            name: "Ship".to_owned(),
            parameters: 0,
            pos: Vec3::new(0.0, 0.0, 50.0),
            angle: [0; 3],
            set_id: 0,
            room: -1,
            ..Default::default()
        },
        Ship::default(),
    ));
    let actor = spawn_actor(&mut app, Vec3::new(0.0, 0.0, 120.0));
    app.stage_target(true);
    tick(&mut app);
    assert_eq!(
        target(&app),
        Some(actor),
        "the ship waits for its own action instead"
    );
}
