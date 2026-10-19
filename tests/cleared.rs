use std::time::Duration;

use clearsleeper::App;
use clearsleeper::SIM_HZ;
use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::cleared;
use clearsleeper::sim::flags::Switches;
use glam::Vec3;

const SWITCH_BYTE: u32 = 4;

fn tick(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

fn marker(app: &mut App, switch: u32) {
    let parameters = 0xFFFF0000 | (switch << cleared::SWITCH_SHIFT);
    let state = cleared::spawn(parameters, app.sim.world.resource::<Switches>(), -1);
    app.sim.world.spawn((
        Actor {
            name: cleared::CLEARED_ACTOR.to_owned(),
            parameters,
            pos: Vec3::ZERO,
            angle: [0, 0, 0],
            set_id: 0,
            room: -1,
            ..Default::default()
        },
        state,
    ));
}

fn switch(app: &App, bit: u8) -> bool {
    app.sim.world.resource::<Switches>().test(bit)
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
