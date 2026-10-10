use std::f32::consts::TAU;

use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::knob00::Knob00;
use clearsleeper::sim::player::Player;
use clearsleeper::App;

/// The stage, room and spawn the house door leads to, from
/// `tww/src/d/actor/d_a_npc_ba1.cpp:1231` (`cut_move_START_TALE1`:
/// `dComIfGp_setNextStage("LinkRM", 200, 0, 8, 0.0f, 0, 1, 0)`).
const HOUSE: (&str, u8, u8) = ("LinkRM", 0, 200);

/// One `i16` binary angle unit in radians, `sim/walk.rs:49-54`'s `RAD_PER_UNIT`.
const RAD_PER_UNIT: f32 = TAU / 65536.0;

/// A knob door and the actor it hangs on, which is where its position and its
/// yaw live.
fn house_door(app: &mut App) -> Option<(glam::Vec3, i16)> {
    let mut doors = app.sim.world.query_filtered::<(&Actor, &Knob00), ()>();
    doors
        .iter(&app.sim.world)
        .map(|(actor, _)| (actor.pos, actor.angle[1]))
        .next()
}

#[test]
fn golden_walking_through_the_house_door_enters_link_rm() {
    let Ok(_) = std::env::var(clearsleeper::assets::ASSETS_ENV) else {
        eprintln!("CS_ASSETS is not set; skipping the house entry check");
        return;
    };
    let mut app = App::new();
    let Some((door, yaw)) = house_door(&mut app) else {
        return;
    };

    // `d_door.cpp:246` `checkArea` passes only for a player inside the frame
    // who faces along the door's own forward axis, so eighty units out along it
    // is standing in the doorway looking into the house.
    let yaw_radians = f32::from(yaw) * RAD_PER_UNIT;
    let step = glam::Vec3::new(yaw_radians.sin(), 0.0, yaw_radians.cos()) * 80.0;

    let mut players = app.sim.world.query::<&mut Player>();
    let Some(mut link) = players.iter_mut(&mut app.sim.world).next() else {
        return;
    };
    link.pos = door + step;
    link.yaw = yaw_radians;

    for _ in 0..4 {
        app.advance(std::time::Duration::from_millis(16));
    }

    // Being inside the house means standing where `LinkRM`'s point 200 spawns,
    // on the upper floor where Grandma waits.
    let mut players = app.sim.world.query::<&Player>();
    let Some(link) = players.iter(&app.sim.world).next() else {
        panic!("the player is gone");
    };
    let (stage, room, start) = HOUSE;
    let expected = clearsleeper::assets::load_stage_spawn(stage, room, start)
        .expect("link's house has a point 200 spawn");
    assert_eq!(
        link.pos, expected.0,
        "walking through the door puts Link where link's house spawns him"
    );
    let mut next = app.sim.world.resource::<clearsleeper::sim::transition::NextStage>();
    assert_eq!(
        next.target.as_ref().map(|target| (target.stage.as_str(), target.room, target.start)),
        None,
        "the destination is consumed by the swap, not left pending"
    );
}
