use std::time::Duration;

use bevy_ecs::entity::Entity;
use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::flags::{self, EventSwitch, StoryBits, Switches};
use clearsleeper::{App, SIM_HZ};
use glam::Vec3;

const TOGGLE: u32 = 0x0302;
const HOLDING: u32 = 0x0301;
const EVENT: u16 = 0x0203;

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

fn stories(app: &App) -> StoryBits {
    *app.sim.world.resource::<StoryBits>()
}

fn raise(app: &mut App, event: u16) {
    app.sim.world.resource_mut::<StoryBits>().set(event);
}

fn lower(app: &mut App, event: u16) {
    app.sim.world.resource_mut::<StoryBits>().clear(event);
}

#[test]
fn golden_switches_start_clear() {
    let app = App::new();
    assert!(!switches(&app).test(2), "a fresh stage has no switches set");
}

#[test]
fn golden_event_switch_follows_its_story_bit() {
    let mut app = App::new();
    spawn_switch(&mut app, TOGGLE);
    tick(&mut app);
    assert!(
        !switches(&app).test(2),
        "the bit is down, so the switch stays down"
    );
    raise(&mut app, EVENT);
    tick(&mut app);
    assert!(switches(&app).test(2), "the story bit drives the switch on");
    lower(&mut app, EVENT);
    tick(&mut app);
    assert!(
        !switches(&app).test(2),
        "an untagged switch drops when its bit clears"
    );
}

#[test]
fn golden_holding_switch_survives_a_cleared_bit() {
    let mut app = App::new();
    spawn_switch(&mut app, HOLDING);
    raise(&mut app, EVENT);
    tick(&mut app);
    assert!(switches(&app).test(1), "the switch takes the bit up");
    lower(&mut app, EVENT);
    tick(&mut app);
    assert!(
        switches(&app).test(1),
        "the holding type keeps its switch lit"
    );
}

#[test]
fn golden_story_bits_address_by_byte_and_mask() {
    let mut app = App::new();
    let id = 0x2F08_u16;
    assert!(!stories(&app).test(id), "nothing is set at boot");
    raise(&mut app, id);
    let bits = stories(&app);
    assert!(bits.test(id), "the id marks its own byte and mask");
    assert_eq!(bits.bytes[0x2F], 0x08, "byte 0x2F, mask 0x08");
    assert_eq!(bits.bytes[0x00], 0x00, "and nowhere else");
    assert!(
        !bits.test(0x2F04),
        "a sibling mask in the same byte stays clear"
    );
}

#[test]
fn golden_event_ids_keep_both_bytes() {
    let switch = EventSwitch::spawn(0x04AB_0002);
    assert_eq!(switch.event, 0xAB04, "the event id is sixteen bits wide");
    assert_eq!(switch.switch_no, 2, "and the switch is a byte");
    assert!(!switch.hold);
    assert!(EventSwitch::spawn(0x0400_0002).hold, "type one holds");
}

#[test]
fn golden_story_registers_replace_their_own_mask() {
    let mut app = App::new();
    let reg = 0x1201_u16;
    let mut bits = app.sim.world.resource_mut::<StoryBits>();
    bits.set_reg(reg, 0x0F);
    assert_eq!(bits.reg(reg), 0x0F);
    bits.set_reg(reg, 0x03);
    assert_eq!(bits.reg(reg), 0x03, "the register writes its own bits");
    assert_eq!(
        bits.bytes[0x12] & 0xFE,
        0,
        "and leaves its neighbours alone"
    );
}
