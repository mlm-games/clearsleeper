use std::time::Duration;

use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::combat::HitPoints;
use clearsleeper::sim::enemy::{self, Hurt};
use clearsleeper::sim::events::SimEvent;
use clearsleeper::sim::item::{self, Bomb, Held};
use clearsleeper::sim::pickup::Pickup;
use clearsleeper::sim::player::Player;
use clearsleeper::{App, SIM_HZ};
use glam::{Vec2, Vec3};
use repose_core::input::PhysicalKey;
use repose_core::runtime::Scheduler;

fn frame(app: &mut App, keys: &[PhysicalKey]) {
    let mut sched = Scheduler::default();
    sched.window_focused = true;
    sched.held_keys = keys.iter().cloned().collect();
    app.stage_polled(&sched);
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

fn tap(app: &mut App, key: PhysicalKey) {
    frame(app, &[key]);
    frame(app, &[]);
}

fn tick(app: &mut App) {
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
            },
            state,
            hit_points,
            Hurt::default(),
        ))
        .id()
}

fn heart(app: &mut App, pos: Vec3) {
    app.sim.world.spawn((
        Actor {
            name: "item".to_owned(),
            parameters: 0,
            pos,
            angle: [0, 0, 0],
            set_id: 0,
        },
        Pickup::spawn(0),
    ));
}

fn drain(app: &mut App) -> Vec<SimEvent> {
    std::mem::take(
        &mut app
            .sim
            .world
            .resource_mut::<clearsleeper::sim::events::SimEvents>()
            .list,
    )
}

#[test]
fn golden_walk_stride_emits_footsteps() {
    let mut app = App::new();
    for _ in 0..10 {
        tick(&mut app);
    }
    assert!(app.drain_events().is_empty());

    app.stage_axis(Vec2::new(0.0, 1.0));
    for _ in 0..30 {
        tick(&mut app);
    }
    // 30 full-speed ticks travel 477 units: two 200-unit strides, not three.
    assert_eq!(
        app.drain_events(),
        vec![SimEvent::Footstep, SimEvent::Footstep]
    );
    assert!(app.drain_events().is_empty());
}

#[test]
fn audio_bank_loads_and_pumps_headless() {
    let mut audio = repame_audio::Audio::noop();
    clearsleeper::audio::load_cues(&mut audio).expect("footstep cue decodes without a device");

    let mut app = App::new();
    app.stage_axis(Vec2::new(0.0, 1.0));
    for _ in 0..30 {
        tick(&mut app);
    }
    app.pump_audio(Duration::from_millis(16));
    assert!(app.drain_events().is_empty());
}

#[test]
fn golden_swinging_whooshes_and_landing_one_clangs() {
    let mut app = App::new();
    place_player(&mut app, Vec3::new(0.0, 0.0, 120.0));
    let _ = foe(&mut app, Vec3::ZERO);
    let _ = drain(&mut app);
    tap(&mut app, PhysicalKey::KeyB);
    let seen = drain(&mut app);
    assert_eq!(seen.first(), Some(&SimEvent::Slash), "the blade whooshes");
    assert!(seen.contains(&SimEvent::Hit), "and rings on the foe");
}

#[test]
fn golden_swinging_at_air_only_whooshes() {
    let mut app = App::new();
    place_player(&mut app, Vec3::new(0.0, 0.0, 120.0));
    let _ = drain(&mut app);
    tap(&mut app, PhysicalKey::KeyB);
    let seen = drain(&mut app);
    assert_eq!(seen, vec![SimEvent::Slash], "nothing else to ring on");
}

#[test]
fn golden_bomb_burst_thuds() {
    let mut app = App::new();
    place_player(&mut app, Vec3::new(0.0, 0.0, 20_000.0));
    let mut held = app.sim.world.resource_mut::<Held>();
    held.equip(item::FIRST_BOMB);
    app.sim.world.spawn(Bomb {
        pos: Vec3::ZERO,
        fuse: 2,
        vel: Vec3::ZERO,
        rest: true,
    });
    let _ = drain(&mut app);
    tick(&mut app);
    tick(&mut app);
    assert!(
        drain(&mut app).contains(&SimEvent::Explode),
        "the fuse ends with a bang"
    );
}

#[test]
fn golden_pickup_chimes() {
    let mut app = App::new();
    place_player(&mut app, Vec3::ZERO);
    heart(&mut app, Vec3::ZERO);
    let _ = drain(&mut app);
    for _ in 0..20 {
        tick(&mut app);
    }
    assert!(drain(&mut app).contains(&SimEvent::Collect));
}

#[test]
fn golden_being_hurt_grunts() {
    let mut app = App::new();
    let _ = drain(&mut app);
    app.stage_damage(1);
    tick(&mut app);
    let seen = drain(&mut app);
    assert!(seen.contains(&SimEvent::Hurt), "the player hears it");
    assert_eq!(app.health().units, 11, "and feels it");
}

#[test]
fn golden_hp_stays_quiet_while_healthy() {
    let mut app = App::new();
    let _ = drain(&mut app);
    tick(&mut app);
    assert!(
        !drain(&mut app).contains(&SimEvent::Hurt),
        "an unhurt walk makes no noise"
    );
}

#[test]
fn golden_foe_hit_points_drop_with_the_clang() {
    let mut app = App::new();
    place_player(&mut app, Vec3::new(0.0, 0.0, 120.0));
    let victim = foe(&mut app, Vec3::ZERO);
    tap(&mut app, PhysicalKey::KeyB);
    tick(&mut app);
    assert_eq!(
        app.sim.world.entity(victim).get::<HitPoints>().unwrap().0,
        4,
        "one blow takes a quarter heart"
    );
}
