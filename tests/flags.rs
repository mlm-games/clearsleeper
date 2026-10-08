use std::time::Duration;

use clearsleeper::sim::flags::{Arrivals, StoryBits, Switches};
use clearsleeper::{App, SIM_HZ};

const EVENT: u16 = 0x0203;

fn tick(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
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
fn golden_arrivals_open_with_the_known_islands() {
    let app = App::new();
    let arrivals = *app.sim.world.resource::<Arrivals>();
    assert!(arrivals.is_arrived(0), "the fortress starts visited");
    assert!(arrivals.is_arrived(10), "windfall starts visited");
    assert!(arrivals.is_arrived(43), "outset starts visited");
    assert!(!arrivals.is_arrived(12), "dragon roost starts uncharted");
    assert!(!arrivals.is_arrived(48), "the far edge starts uncharted");
    assert!(
        !arrivals.is_arrived(99),
        "a grid past the map reads uncharted"
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
