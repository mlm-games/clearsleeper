use std::time::Duration;

use clearsleeper::App;
use clearsleeper::SIM_HZ;
use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::cleared::{self, Cleared};
use clearsleeper::sim::enemy::{self, Enemy, Hurt};
use clearsleeper::sim::flags::Switches;
use glam::Vec3;

const SWITCH_BYTE: u32 = 4;

fn tick(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

fn marker(app: &mut App, switch: u32) {
    let parameters = 0xFFFF0000 | (switch << cleared::SWITCH_SHIFT);
    let state = cleared::spawn(parameters, app.sim.world.resource::<Switches>());
    app.sim.world.spawn((
        Actor {
            name: cleared::CLEARED_ACTOR.to_owned(),
            parameters,
            pos: Vec3::ZERO,
            angle: [0, 0, 0],
            set_id: 0,
        },
        state,
    ));
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

fn switch(app: &App, bit: u8) -> bool {
    app.sim.world.resource::<Switches>().0.test(bit)
}

#[test]
fn golden_empty_room_flags_its_switch_after_sixty_five_ticks() {
    let mut app = App::new();
    marker(&mut app, SWITCH_BYTE);
    assert!(!switch(&mut app, SWITCH_BYTE as u8), "nothing yet");
    for _ in 0..64 {
        tick(&mut app);
    }
    assert!(
        !switch(&mut app, SWITCH_BYTE as u8),
        "the timer is still running"
    );
    tick(&mut app);
    assert!(
        switch(&mut app, SWITCH_BYTE as u8),
        "and the room counts as cleared"
    );
}

#[test]
fn golden_room_with_a_foe_never_flags() {
    let mut app = App::new();
    marker(&mut app, SWITCH_BYTE);
    let _ = foe(&mut app, Vec3::new(0.0, 0.0, 0.0));
    for _ in 0..200 {
        tick(&mut app);
    }
    assert!(
        !switch(&mut app, SWITCH_BYTE as u8),
        "a room with a foe's standing keeps its switch"
    );
}

#[test]
fn golden_clearing_the_room_starts_the_clock() {
    let mut app = App::new();
    marker(&mut app, SWITCH_BYTE);
    let last = foe(&mut app, Vec3::new(0.0, 0.0, 0.0));
    for _ in 0..30 {
        tick(&mut app);
    }
    app.sim.world.despawn(last);
    for _ in 0..64 {
        tick(&mut app);
    }
    assert!(
        !switch(&mut app, SWITCH_BYTE as u8),
        "the count starts only once the room is clear"
    );
    for _ in 0..2 {
        tick(&mut app);
    }
    assert!(switch(&mut app, SWITCH_BYTE as u8), "then it flips");
}

#[test]
fn golden_marker_waits_for_a_straggler_to_come_back() {
    let mut app = App::new();
    marker(&mut app, SWITCH_BYTE);
    let last = foe(&mut app, Vec3::new(0.0, 0.0, 0.0));
    app.sim.world.despawn(last);
    for _ in 0..60 {
        tick(&mut app);
    }
    let _ = foe(&mut app, Vec3::new(5000.0, 0.0, 5000.0));
    for _ in 0..60 {
        tick(&mut app);
    }
    assert!(
        !switch(&mut app, SWITCH_BYTE as u8),
        "a late arrival restarts the wait"
    );
}

#[test]
fn golden_flagged_switch_keeps_the_marker_idle() {
    let mut app = App::new();
    app.sim
        .world
        .resource_mut::<Switches>()
        .0
        .set(SWITCH_BYTE as u8);
    marker(&mut app, SWITCH_BYTE);
    for _ in 0..200 {
        tick(&mut app);
    }
    assert!(
        switch(&mut app, SWITCH_BYTE as u8),
        "a switch already on stays on"
    );
    let state = app
        .sim
        .world
        .iter_entities()
        .find_map(|e| e.get::<Cleared>().copied())
        .expect("marker exists");
    assert!(state.done, "and the marker stops counting");
    assert_eq!(state.countdown, 0, "with the clock at zero");
}

#[test]
fn golden_cleared_flag_survives_a_reload() {
    let mut app = App::new();
    marker(&mut app, SWITCH_BYTE);
    for _ in 0..66 {
        tick(&mut app);
    }
    let save = app.save();
    let mut restored = App::new();
    restored.load(&save).unwrap();
    assert!(
        restored
            .sim
            .world
            .resource::<Switches>()
            .0
            .test(SWITCH_BYTE as u8),
        "the cleared room stays cleared"
    );
}

#[test]
fn golden_foes_are_what_the_marker_waits_for() {
    let mut app = App::new();
    marker(&mut app, SWITCH_BYTE);
    let _ = foe(&mut app, Vec3::new(0.0, 0.0, 0.0));
    tick(&mut app);
    let state = app
        .sim
        .world
        .iter_entities()
        .find_map(|e| e.get::<Cleared>().copied())
        .expect("marker exists");
    assert_eq!(state.countdown, 65, "the clock is held while foes live");
    assert!(app.sim.world.iter_entities().any(|e| e.contains::<Enemy>()));
}
