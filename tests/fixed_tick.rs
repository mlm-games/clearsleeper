use std::time::Duration;

use clearsleeper::sim::frame::CurrentFrame;
use clearsleeper::{App, SIM_HZ};
use repame_sim::SimTime;

#[test]
fn golden_fixed_tick_loop() {
    assert_eq!(SIM_HZ, 30.0);
    let mut app = App::new();
    let mut ran = 0;
    for _ in 0..100 {
        ran += app.advance(Duration::from_millis(33));
    }
    assert_eq!(ran, 99);
    assert_eq!(app.sim.world.resource::<CurrentFrame>().0, 99);
    let time = app.sim.world.resource::<SimTime>();
    assert!((time.elapsed_secs - 99.0 / SIM_HZ).abs() < 1e-6);
}

#[test]
fn hitch_caps_catch_up_at_max_steps() {
    let mut app = App::new();
    assert_eq!(app.advance(Duration::from_secs(1)), 8);
    assert_eq!(app.sim.world.resource::<CurrentFrame>().0, 8);
}
