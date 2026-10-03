use std::time::Duration;

use clearsleeper::sim::ending::{self, Ending};
use clearsleeper::sim::events::{SimEvent, SimEvents};
use clearsleeper::sim::flags::Switches;
use clearsleeper::sim::player::Player;
use clearsleeper::{App, SIM_HZ};
use glam::Vec2;

fn tick(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

fn ending(app: &App) -> Ending {
    *app.sim.world.resource::<Ending>()
}

fn complete(app: &mut App) {
    app.sim
        .world
        .resource_mut::<Switches>()
        .set(ending::COMPLETE_SWITCH);
}

fn events(app: &App) -> Vec<SimEvent> {
    app.sim.world.resource::<SimEvents>().list.clone()
}

#[test]
fn golden_nothing_starts_before_the_finale() {
    let mut app = App::new();
    for _ in 0..30 {
        tick(&mut app);
    }
    assert!(!ending(&app).running, "an unfinished sea has no credits");
}

#[test]
fn golden_credits_run_then_finish() {
    let mut app = App::new();
    complete(&mut app);
    tick(&mut app);
    assert!(ending(&app).running, "the finished game rolls the credits");
    assert!(
        events(&app).contains(&SimEvent::Credits),
        "the shell is told to show the roll"
    );
    for _ in 0..600 {
        tick(&mut app);
    }
    assert!(!ending(&app).running);
    assert!(ending(&app).finished, "the roll ends for good");
    assert!(
        events(&app).contains(&SimEvent::CreditsDone),
        "the shell is told the roll is over"
    );
}

#[test]
fn golden_credits_hold_the_body() {
    let mut app = App::new();
    complete(&mut app);
    tick(&mut app);
    app.stage_axis(Vec2::new(0.0, 1.0));
    for _ in 0..6 {
        tick(&mut app);
    }
    let player = app
        .sim
        .world
        .iter_entities()
        .find_map(|entity| entity.get::<Player>().copied())
        .expect("player entity");
    assert_eq!(player.speed, 0.0, "the stick is dead during the roll");
    app.stage_axis(Vec2::ZERO);
}
