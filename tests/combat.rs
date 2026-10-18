use std::time::Duration;

use bevy_ecs::prelude::Entity;
use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::bodies;
use clearsleeper::sim::combat::{Combat, CutKind, HitPoints};
use clearsleeper::sim::contact_run::ContactBodies;
use clearsleeper::sim::lock::Lock;
use clearsleeper::sim::player::Player;
use clearsleeper::sim::registry;
use clearsleeper::sim::roll::Roll;
use clearsleeper::sim::shape::Shape;
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
        .find_map(|e| e.get::<Player>().copied())
        .expect("player")
}

fn combat(app: &App) -> Combat {
    *app.sim.world.resource::<Combat>()
}

/// The blade's reach: `combat.rs` builds the sword as a sphere `SWORD_OFFSET` (20) ahead of the
/// player with `SWORD_RADIUS` (20), so a small body inside that sphere is what the contact pass
/// can see. Without `ContactBodies` the actor is dropped by the `bodies?` filter outright.
///
/// The shape is authored at the local origin, the way `bodies_for` leaves every body: the spawn
/// path anchors each body to `actor.pos + offset`, so an absolute position drifts by the whole
/// actor position every tick.
const TARGET_RADIUS: f32 = 6.0;

fn spawn_target(app: &mut App, pos: Vec3) -> Entity {
    let mut bodies = ContactBodies::default();
    bodies.add(
        Shape::sphere(Vec3::ZERO, TARGET_RADIUS),
        bodies::default_obj(),
    );
    let id = app
        .sim
        .world
        .spawn((
            Actor {
                name: "Pot".to_owned(),
                parameters: 0,
                pos,
                angle: [0; 3],
                set_id: 0,
                room: -1,
                ..Default::default()
            },
            HitPoints::default(),
            bodies,
        ))
        .id();
    // `TgObj` stamps the target's own registry process id onto the crossing's `Stts.actor`, and
    // `chk_no_hit_at_tg` drops any pair whose two actors match, so a target that is not
    // registered reads as the attacker hitting itself.
    app.sim
        .world
        .resource_mut::<registry::ActorRegistry>()
        .register(id, 0, 0);
    id
}

fn hp(app: &App, entity: Entity) -> Option<i32> {
    app.sim.world.get::<HitPoints>(entity).map(|hit| hit.0)
}

#[test]
fn golden_swing_starts_and_ends() {
    let mut app = App::new();
    app.stage_item(true);
    tick(&mut app);
    let state = combat(&app);
    assert!(state.active);
    assert_eq!(
        state.kind,
        CutKind::L,
        "a standing first cut slashes sideways"
    );
    assert_eq!(state.combo, 1);
    assert_eq!(state.frame, 1);
    app.stage_item(false);
    for _ in 0..12 {
        tick(&mut app);
    }
    let state = combat(&app);
    assert!(!state.active, "the cut runs its full length");
    assert_eq!(state.combo, 0, "an unchained cut drops the combo");
    assert_eq!(player(&app).speed, 0.0, "the cut spends the body's speed");
}

#[test]
fn golden_swing_lunges_while_running() {
    let mut app = App::new();
    app.stage_axis(Vec2::new(0.0, 1.0));
    for _ in 0..6 {
        tick(&mut app);
    }
    let running = player(&app).speed;
    assert!(running > 3.0, "the run builds speed");
    app.stage_item(true);
    tick(&mut app);
    let state = combat(&app);
    assert!(state.active);
    assert_eq!(
        state.kind,
        CutKind::F,
        "a running forward press cuts forward"
    );
    assert_eq!(
        player(&app).speed,
        running,
        "the wind-up keeps the run's speed"
    );
    app.stage_item(false);
    tick(&mut app);
    let decaying = player(&app).speed;
    assert!(decaying < running, "the cut bleeds speed off");
    tick(&mut app);
    let lunging = player(&app).speed;
    let expected = 0.2 * decaying + 8.0;
    assert!(
        (lunging - expected).abs() < 0.001,
        "the forward lunge lands on the third tick"
    );
}

#[test]
fn golden_combo_chains_to_finisher() {
    let mut app = App::new();
    app.stage_item(true);
    tick(&mut app);
    app.stage_item(false);
    for _ in 0..3 {
        tick(&mut app);
    }
    app.stage_item(true);
    tick(&mut app);
    app.stage_item(false);
    for _ in 0..7 {
        tick(&mut app);
    }
    let state = combat(&app);
    assert!(state.active, "the queued press chains the second cut");
    assert_eq!(state.combo, 2);
    app.stage_item(true);
    tick(&mut app);
    app.stage_item(false);
    for _ in 0..10 {
        tick(&mut app);
    }
    let state = combat(&app);
    assert!(state.active, "the second cut chains the third");
    assert_eq!(state.combo, 3);
    app.stage_item(true);
    tick(&mut app);
    app.stage_item(false);
    for _ in 0..10 {
        tick(&mut app);
    }
    let state = combat(&app);
    assert!(state.active, "the third cut chains the finisher");
    assert_eq!(state.kind, CutKind::EB, "the fourth cut is the finisher");
    assert_eq!(state.combo, 0, "the finisher drops the combo at once");
    app.stage_item(true);
    tick(&mut app);
    app.stage_item(false);
    for _ in 0..16 {
        tick(&mut app);
    }
    assert!(
        !combat(&app).active,
        "the finisher cannot be chained further"
    );
}

#[test]
fn golden_swing_breaks_targets() {
    let mut app = App::new();
    let front = spawn_target(&mut app, Vec3::new(0.0, 0.0, 25.0));
    let behind = spawn_target(&mut app, Vec3::new(0.0, 0.0, -25.0));
    let side = spawn_target(&mut app, Vec3::new(30.0, 0.0, 20.0));
    let boat = app
        .sim
        .world
        .spawn((
            Actor {
                name: "Ship".to_owned(),
                parameters: 0,
                pos: Vec3::new(0.0, 0.0, 30.0),
                angle: [0; 3],
                set_id: 0,
                room: -1,
                weight_uc: 0xFF,
                ..Default::default()
            },
            Ship::default(),
            ContactBodies::default(),
        ))
        .id();
    app.stage_item(true);
    tick(&mut app);
    assert_eq!(hp(&app, front), Some(1), "the wind-up tick lands no hit");
    app.stage_item(false);
    tick(&mut app);
    assert_eq!(hp(&app, front), None, "the cut breaks what stands ahead");
    assert_eq!(hp(&app, behind), Some(1), "the back stays whole");
    assert_eq!(hp(&app, side), Some(1), "the flank stays whole");
    assert!(
        app.sim.world.get::<Ship>(boat).is_some(),
        "the boat is no target"
    );
}

#[test]
fn golden_swing_and_roll_do_not_overlap() {
    let mut app = App::new();
    app.stage_item(true);
    tick(&mut app);
    app.stage_item(false);
    app.stage_axis(Vec2::new(0.0, 1.0));
    app.stage_do(true);
    tick(&mut app);
    assert!(
        !app.sim.world.resource::<Roll>().active,
        "no roll starts mid-cut"
    );
    assert!(combat(&app).active, "the cut holds the body");
    app.stage_do(false);
    app.stage_axis(Vec2::ZERO);
    for _ in 0..11 {
        tick(&mut app);
    }
    assert!(!combat(&app).active, "the first cut ends");

    let mut app = App::new();
    app.stage_axis(Vec2::new(0.0, 1.0));
    for _ in 0..3 {
        tick(&mut app);
    }
    app.stage_do(true);
    tick(&mut app);
    assert!(
        app.sim.world.resource::<Roll>().active,
        "the roll starts while running"
    );
    app.stage_item(true);
    tick(&mut app);
    assert!(!combat(&app).active, "no cut starts mid-roll");
    assert!(
        app.sim.world.resource::<Roll>().active,
        "the roll holds the body"
    );
}

#[test]
fn golden_locked_swing_squares_up() {
    let mut app = App::new();
    let target = spawn_target(&mut app, Vec3::new(0.0, 0.0, 100.0));
    app.stage_target(true);
    tick(&mut app);
    app.stage_target(false);
    assert!(
        app.sim.world.resource::<Lock>().target.is_some(),
        "the lock takes hold"
    );
    app.stage_axis(Vec2::new(1.0, 0.0));
    for _ in 0..6 {
        tick(&mut app);
    }
    let strafing = player(&app);
    assert!(
        (strafing.yaw - strafing.face).abs() > 0.05,
        "the strafe splits body from face"
    );
    app.stage_item(true);
    tick(&mut app);
    app.stage_axis(Vec2::ZERO);
    app.stage_item(false);
    let swinging = player(&app);
    let state = combat(&app);
    assert!(state.active);
    assert_eq!(
        state.kind,
        CutKind::R,
        "a strafing press cuts toward the stick"
    );
    assert!(
        (swinging.yaw - swinging.face).abs() < 0.001,
        "the cut squares the body to the face"
    );
    let want = (0.0 - swinging.pos.x).atan2(100.0 - swinging.pos.z);
    assert!(
        (swinging.face - want).abs() < 0.05,
        "the face holds the target"
    );
    assert!(
        app.sim.world.resource::<Lock>().target.is_some(),
        "the lock holds through the cut"
    );
    for _ in 0..13 {
        tick(&mut app);
    }
    assert!(!combat(&app).active, "the cut ends");
    assert_eq!(hp(&app, target), Some(1), "the target sits out of reach");
}
