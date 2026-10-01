use std::time::Duration;

use clearsleeper::sim::events::SimEvent;
use clearsleeper::{App, SIM_HZ};
use glam::Vec2;

fn tick(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
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
