use std::time::Duration;

use bevy_ecs::prelude::{Entity, With};
use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::area::StageName;
use clearsleeper::sim::flags::{Arrivals, OrderFlags, StoryBits};
use clearsleeper::sim::player::Player;
use clearsleeper::sim::talk::{self, Context, Dialogue, Scene, Step, Talk, advance, opening};
use clearsleeper::{App, SIM_HZ};
use glam::{Vec2, Vec3};

const OUTSET: i8 = 44;
const DRAGON_ROOST: i8 = 13;
const UNCHARTED: i8 = 3;

fn tick(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

fn press(app: &mut App) {
    app.stage_do(true);
    tick(app);
    app.stage_do(false);
    tick(app);
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

fn spawn_talker(app: &mut App, pos: Vec3, angle_x: i16, room: i8) -> Entity {
    let stale: Vec<Entity> = {
        let mut query = app.sim.world.query_filtered::<Entity, With<Talk>>();
        query.iter(&app.sim.world).collect()
    };
    for entity in stale {
        let _ = app.sim.world.despawn(entity);
    }
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
            Talk::new(talk::message(angle_x), room),
        ))
        .id()
}

fn dialogue(app: &App) -> Dialogue {
    *app.sim.world.resource::<Dialogue>()
}

fn talker(app: &mut App) -> Talk {
    let mut query = app.sim.world.query::<&Talk>();
    *query.iter(&app.sim.world).next().expect("talker entity")
}

fn stories(app: &App) -> StoryBits {
    *app.sim.world.resource::<StoryBits>()
}

fn order_flags(app: &App) -> OrderFlags {
    *app.sim.world.resource::<OrderFlags>()
}

fn fresh_context() -> Context {
    Context {
        stories: StoryBits::default(),
        arrivals: Arrivals::default(),
        sea: false,
        has_bow: false,
        minigame_tag: false,
        timer_active: false,
        accepted: false,
    }
}

#[test]
fn golden_talk_ladder_walks_the_arrived_island() {
    let mut app = App::new();
    let speaker = spawn_talker(&mut app, Vec3::ZERO, 0x1234, OUTSET);
    teleport(&mut app, Vec3::new(0.0, 0.0, 200.0));
    press(&mut app);
    let open = dialogue(&app);
    assert!(open.open, "the button opens the talk inside 400 units");
    assert_eq!(open.speaker, Some(speaker), "the box holds the speaker");
    assert_eq!(
        open.message, 0x32D0,
        "the arrived island opens on the arrive check"
    );
    press(&mut app);
    assert_eq!(
        dialogue(&app).message,
        0x32D4,
        "arrival sends the ladder back to the base line"
    );
    press(&mut app);
    assert_eq!(
        dialogue(&app).message,
        0x1234,
        "the home angle carries the base line"
    );
    press(&mut app);
    assert_eq!(
        dialogue(&app).message,
        0x32D6,
        "the base line offers the tag prompt"
    );
    press(&mut app);
    assert_eq!(
        dialogue(&app).message,
        0x32D7,
        "without the bow the offer declines"
    );
    press(&mut app);
    assert!(!dialogue(&app).open, "the declined offer closes the box");
    assert!(
        stories(&app).test(0x0901),
        "closing the box marks the soldier spoken to"
    );
    assert!(
        order_flags(&app).has(0x0008),
        "the conversation end raises the talk event flag"
    );
    assert!(talker(&mut app).gone, "the soldier dives away afterwards");
    assert!(
        !talker(&mut app).map_done,
        "the dive resets the session flags"
    );
    press(&mut app);
    assert!(
        !dialogue(&app).open,
        "a vanished soldier does not answer again"
    );
}

#[test]
fn golden_talk_ladder_meets_then_vanishes_at_dragon_roost() {
    let mut app = App::new();
    app.sim.world.insert_resource(StageName("sea".to_owned()));
    let speaker = spawn_talker(&mut app, Vec3::ZERO, 0x1234, DRAGON_ROOST);
    teleport(&mut app, Vec3::new(0.0, 0.0, 200.0));
    press(&mut app);
    assert_eq!(
        dialogue(&app).message,
        0x32CA,
        "the first meeting opens at dragon roost"
    );
    press(&mut app);
    assert_eq!(dialogue(&app).message, 0x32CB, "the meeting steps on");
    press(&mut app);
    assert_eq!(dialogue(&app).message, 0x32CC, "the meeting steps on");
    press(&mut app);
    assert_eq!(
        dialogue(&app).message,
        0x32CD,
        "the meeting reaches its ask"
    );
    press(&mut app);
    let open = dialogue(&app);
    assert!(open.open, "the map event hands straight back to the talk");
    assert_eq!(
        open.message, 0x1234,
        "the reopened box starts on the base line"
    );
    assert_eq!(
        open.speaker,
        Some(speaker),
        "the speaker survives the event"
    );
    assert!(
        talker(&mut app).map_done,
        "the map event records the map done"
    );
    assert!(
        order_flags(&app).has(0x0008),
        "the map event raises the talk event flag"
    );
    press(&mut app);
    assert_eq!(
        dialogue(&app).message,
        0x32CE,
        "the base line with the map points at the farewell"
    );
    press(&mut app);
    assert!(!dialogue(&app).open, "the farewell closes the box");
    assert!(talker(&mut app).gone, "the farewell dives away");
    assert!(
        stories(&app).test(0x0901),
        "the farewell marks the soldier spoken to"
    );
    assert!(
        !talker(&mut app).map_done,
        "the dive resets the session flags"
    );
}

#[test]
fn golden_talk_ladder_offers_the_map_on_uncharted_islands() {
    let mut app = App::new();
    spawn_talker(&mut app, Vec3::ZERO, 0x1234, UNCHARTED);
    teleport(&mut app, Vec3::new(0.0, 0.0, 200.0));
    press(&mut app);
    assert_eq!(
        dialogue(&app).message,
        0x32D0,
        "an uncharted island opens on the arrive check"
    );
    press(&mut app);
    assert_eq!(
        dialogue(&app).message,
        0x32D1,
        "the uncharted grid offers the map"
    );
    press(&mut app);
    assert!(dialogue(&app).open, "the map event hands back to the talk");
    assert_eq!(
        dialogue(&app).message,
        0x1234,
        "the reopened box starts on the base line"
    );
    assert!(
        talker(&mut app).map_done,
        "the map event records the map done"
    );
    press(&mut app);
    assert_eq!(
        dialogue(&app).message,
        0x32D2,
        "the base line continues after the map event"
    );
    press(&mut app);
    assert_eq!(dialogue(&app).message, 0x32D3, "the continuation steps on");
    press(&mut app);
    assert!(!dialogue(&app).open, "the continuation closes the box");
    assert!(talker(&mut app).gone, "the soldier dives away afterwards");
}

#[test]
fn golden_talk_waits_out_of_reach() {
    let mut app = App::new();
    spawn_talker(&mut app, Vec3::ZERO, 0x1234, OUTSET);
    teleport(&mut app, Vec3::new(0.0, 0.0, 500.0));
    press(&mut app);
    assert!(
        !dialogue(&app).open,
        "past 400 units nobody answers the button"
    );
}

#[test]
fn golden_open_dialogue_freezes_the_body() {
    let mut app = App::new();
    spawn_talker(&mut app, Vec3::ZERO, 0x1234, OUTSET);
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

#[test]
fn golden_soldier_ladder_closes_after_the_bow_event() {
    let mut talk = Talk::new(0x1234, OUTSET);
    talk.bow_done = true;
    let ctx = fresh_context();
    assert_eq!(
        opening(&talk, &ctx),
        0x32E1,
        "the bow event reopens on the after-bow line"
    );
    assert_eq!(advance(0x32E1, &talk, &ctx), Step::Next(0x32E2));
    assert_eq!(
        advance(0x32E2, &talk, &ctx),
        Step::Done(None),
        "the after-bow line ends the talk"
    );
    talk.rupee_count = 10;
    assert_eq!(
        opening(&talk, &ctx),
        0x32DD,
        "ten counts reopen on the rupee prompt"
    );
    assert_eq!(advance(0x32DD, &talk, &ctx), Step::Next(0x32DE));
    assert_eq!(
        advance(0x32DE, &talk, &ctx),
        Step::Done(Some(Scene::GetRupee)),
        "the accepted prompt runs the rupee event"
    );
    talk.rupee_done = true;
    assert_eq!(opening(&talk, &ctx), 0x32E2, "a collected run closes");
    talk.rupee_done = false;
    talk.rupee_count = 1;
    assert_eq!(
        opening(&talk, &ctx),
        0x32E0,
        "one count reads the single tally"
    );
    talk.rupee_count = 4;
    assert_eq!(
        opening(&talk, &ctx),
        0x32DF,
        "many counts read the plural tally"
    );
    assert_eq!(advance(0x32DF, &talk, &ctx), Step::Next(0x32E2));
    assert_eq!(advance(0x32E0, &talk, &ctx), Step::Next(0x32E2));
}

#[test]
fn golden_soldier_ladder_guards_the_bow_offer() {
    let talk = Talk::new(0x1234, OUTSET);
    let mut ctx = fresh_context();
    assert_eq!(
        advance(0x32D6, &talk, &ctx),
        Step::Next(0x32D7),
        "no tag declines the prompt"
    );
    ctx.minigame_tag = true;
    assert_eq!(
        advance(0x32D6, &talk, &ctx),
        Step::Next(0x32D7),
        "a tag without the bow declines the prompt"
    );
    ctx.has_bow = true;
    ctx.timer_active = true;
    assert_eq!(
        advance(0x32D6, &talk, &ctx),
        Step::Next(0x32D7),
        "a running timer declines the prompt"
    );
    ctx.timer_active = false;
    assert_eq!(
        advance(0x32D6, &talk, &ctx),
        Step::Next(0x32D8),
        "the tag with the bow opens the first offer"
    );
    assert_eq!(
        advance(0x32D8, &talk, &ctx),
        Step::Next(0x32D9),
        "declining steps to the refusal"
    );
    assert_eq!(advance(0x32D9, &talk, &ctx), Step::Next(0x32D7));
    assert_eq!(advance(0x32D7, &talk, &ctx), Step::Done(None));
    ctx.accepted = true;
    assert_eq!(
        advance(0x32D8, &talk, &ctx),
        Step::Next(0x32DA),
        "accepting steps forward"
    );
    assert_eq!(advance(0x32DA, &talk, &ctx), Step::Next(0x32DB));
    assert_eq!(
        advance(0x32DB, &talk, &ctx),
        Step::Done(Some(Scene::Bow)),
        "the accepted offer runs the bow event"
    );
    let mut stories = StoryBits::default();
    stories.set(0x3A10);
    ctx.stories = stories;
    assert_eq!(
        advance(0x32D6, &talk, &ctx),
        Step::Next(0x32DC),
        "a taught bow reoffers through the repeat line"
    );
    assert_eq!(
        advance(0x32DC, &talk, &ctx),
        Step::Next(0x32DB),
        "the repeat offer accepts straight into the event"
    );
}

#[test]
fn golden_soldier_ladder_branches_on_the_first_visit() {
    let mut talk = Talk::new(0x1234, DRAGON_ROOST);
    let mut ctx = fresh_context();
    ctx.sea = true;
    assert_eq!(
        opening(&talk, &ctx),
        0x32CA,
        "dragon roost opens on the first meeting"
    );
    talk.map_done = true;
    assert_eq!(
        opening(&talk, &ctx),
        0x1234,
        "the done map hands back the base line"
    );
    assert_eq!(
        advance(0x1234, &talk, &ctx),
        Step::Next(0x32CE),
        "the mapped first visit points at the farewell"
    );
    ctx.stories.set(0x0901);
    assert_eq!(
        advance(0x1234, &talk, &ctx),
        Step::Next(0x32D2),
        "a spoken soldier continues past the map instead"
    );
    assert_eq!(
        advance(0x32D0, &talk, &ctx),
        Step::Next(0x32D1),
        "an uncharted grid offers the map"
    );
    let outward = Talk::new(0x1234, OUTSET);
    ctx.sea = false;
    assert_eq!(
        advance(0x32D0, &outward, &ctx),
        Step::Next(0x32D4),
        "the home grid reads as arrived"
    );
}
