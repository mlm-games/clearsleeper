use std::time::Duration;

use bevy_ecs::prelude::Entity;
use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::flags::{self, EventFlags, EventSwitch, Switches};
use clearsleeper::{App, SIM_HZ};
use glam::Vec3;

const TOGGLE: u32 = 0x0302;
const HOLDING: u32 = 0x0301;
const EVENT: u8 = 3;

fn tick(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

fn spawn_switch(app: &mut App, parameters: u32) -> Entity {
    app.sim
        .world
        .spawn((
            Actor {
                name: flags::EVENT_SWITCH_ACTOR.to_owned(),
                parameters,
                pos: Vec3::ZERO,
                angle: [0; 3],
                set_id: 0,
            },
            EventSwitch::spawn(parameters),
        ))
        .id()
}

fn switches(app: &App) -> Switches {
    *app.sim.world.resource::<Switches>()
}

fn raise(app: &mut App, event: u8) {
    app.sim.world.resource_mut::<EventFlags>().0.set(event);
}

fn lower(app: &mut App, event: u8) {
    app.sim.world.resource_mut::<EventFlags>().0.clear(event);
}

#[test]
fn golden_switches_start_clear() {
    let app = App::new();
    assert!(
        !switches(&app).0.test(2),
        "a fresh stage has no switches set"
    );
}

#[test]
fn golden_event_switch_follows_its_event_bit() {
    let mut app = App::new();
    spawn_switch(&mut app, TOGGLE);
    tick(&mut app);
    assert!(
        !switches(&app).0.test(2),
        "the bit is down, so the switch stays down"
    );
    raise(&mut app, EVENT);
    tick(&mut app);
    assert!(
        switches(&app).0.test(2),
        "the event bit drives the switch on"
    );
    lower(&mut app, EVENT);
    tick(&mut app);
    assert!(
        !switches(&app).0.test(2),
        "an untagged switch drops when its bit clears"
    );
}

#[test]
fn golden_holding_switch_survives_a_cleared_bit() {
    let mut app = App::new();
    spawn_switch(&mut app, HOLDING);
    raise(&mut app, EVENT);
    tick(&mut app);
    assert!(switches(&app).0.test(1), "the switch takes the bit up");
    lower(&mut app, EVENT);
    tick(&mut app);
    assert!(
        switches(&app).0.test(1),
        "the holding type keeps its switch lit"
    );
}
