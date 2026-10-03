use std::time::Duration;

use bevy_ecs::prelude::Entity;
use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::player::Player;
use clearsleeper::sim::talk::{self, Dialogue, Talk};
use clearsleeper::{App, SIM_HZ};
use glam::{Vec2, Vec3};

fn tick(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

fn player(app: &App) -> Player {
    app.sim
        .world
        .iter_entities()
        .find_map(|entity| entity.get::<Player>().copied())
        .expect("player entity")
}

fn teleport(app: &mut App, pos: Vec3) {
    let mut query = app.sim.world.query::<&mut Player>();
    let mut player = query.iter_mut(&mut app.sim.world).next().unwrap();
    player.pos = pos;
    player.yaw = 0.0;
}

fn spawn_talker(app: &mut App, pos: Vec3, angle_x: i16) -> Entity {
    app.sim
        .world
        .spawn((
            Actor {
                name: talk::TALK_ACTOR.to_owned(),
                parameters: 0,
                pos,
                angle: [angle_x, 0, 0],
                set_id: 0,
            },
            Talk {
                message: talk::message(angle_x),
            },
        ))
        .id()
}

fn dialogue(app: &App) -> Dialogue {
    *app.sim.world.resource::<Dialogue>()
}

#[test]
fn golden_talk_opens_and_closes_on_the_button() {
    let mut app = App::new();
    spawn_talker(&mut app, Vec3::new(0.0, 0.0, 0.0), 0x1234);
    teleport(&mut app, Vec3::new(0.0, 0.0, 200.0));
    app.stage_do(true);
    tick(&mut app);
    let open = dialogue(&app);
    assert!(open.open, "the button opens the talk inside 400 units");
    assert_eq!(open.message, 0x1234, "the home angle carries the line");
    app.stage_do(false);
    tick(&mut app);
    app.stage_do(true);
    tick(&mut app);
    assert!(!dialogue(&app).open, "the next press closes the box");
}

#[test]
fn golden_talk_waits_out_of_reach() {
    let mut app = App::new();
    spawn_talker(&mut app, Vec3::new(0.0, 0.0, 0.0), 0x1234);
    teleport(&mut app, Vec3::new(0.0, 0.0, 500.0));
    app.stage_do(true);
    tick(&mut app);
    assert!(
        !dialogue(&app).open,
        "past 400 units nobody answers the button"
    );
}

#[test]
fn golden_open_dialogue_freezes_the_body() {
    let mut app = App::new();
    spawn_talker(&mut app, Vec3::new(0.0, 0.0, 0.0), 0x1234);
    teleport(&mut app, Vec3::new(0.0, 0.0, 200.0));
    app.stage_do(true);
    tick(&mut app);
    assert!(dialogue(&app).open);
    let before = player(&app).pos;
    app.stage_do(false);
    app.stage_axis(Vec2::new(0.0, 1.0));
    for _ in 0..6 {
        tick(&mut app);
    }
    let after = player(&app);
    assert_eq!(after.speed, 0.0, "the stick cannot drive an open box");
    assert_eq!(after.pos, before, "the body holds its ground");
    app.stage_axis(Vec2::ZERO);
}

#[test]
fn golden_talk_message_falls_back_to_the_default() {
    assert_eq!(talk::message(-1), 1, "an unset angle reads as line one");
    assert_eq!(talk::message(0), 1, "a blank angle reads as line one");
    assert_eq!(talk::message(0x1902), 0x1902, "a real line passes through");
}
