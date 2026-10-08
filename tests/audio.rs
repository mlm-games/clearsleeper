use std::time::Duration;

use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::enemy::{self, Hurt};
use clearsleeper::sim::events::SimEvent;
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
                room: -1,
                ..Default::default()
            },
            state,
            hit_points,
            Hurt::default(),
        ))
        .id()
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
