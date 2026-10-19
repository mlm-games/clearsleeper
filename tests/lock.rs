//! The lock-on the reference's attention manager hands the player.
//!
//! The subject here is a Bokoblin, the one foe the reference marks for the
//! battle lock-on: `d_a_bk.cpp:4284` gives it `fopAc_Attn_LOCKON_BATTLE_e`
//! while it is on its feet, and the lock-on list is built from the actors
//! whose own flags meet the manager's (`d_attention.cpp:349`, the mask the
//! player's own flags make at `:471`) against the three lock-on types
//! (`d_attention.cpp:15-19`).

use bevy_ecs::prelude::Entity;
use clearsleeper::App;
use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::enemy;
use clearsleeper::sim::lock::Lock;
use clearsleeper::sim::player::Player;
use clearsleeper::sim::ship::Ship;
use glam::Vec3;

mod common;

/// A Bokoblin whose sight band is zero, so it holds its post and the lock is
/// the only thing that moves: `d_a_bk.cpp:4770` reads the band out of the
/// parameter's second byte and `d_a_bk.cpp:1397-1401` turns it into the
/// distance the foe notices the player at.
const BLIND: u32 = 0x04;

fn spawn_bokoblin(app: &mut App, pos: Vec3) -> Entity {
    let (state, hit_points) = enemy::spawn(BLIND, 0);
    app.sim
        .world
        .spawn((
            Actor {
                name: enemy::ENEMY_ACTOR.to_owned(),
                parameters: BLIND,
                pos,
                angle: [0, 0, 0],
                set_id: 0,
                room: -1,
                ..Default::default()
            },
            state,
            hit_points,
        ))
        .id()
}

fn target(app: &App) -> Option<Entity> {
    app.sim.world.resource::<Lock>().target
}

fn player(app: &App) -> Player {
    app.sim
        .world
        .iter_entities()
        .find_map(|entity| entity.get::<Player>().copied())
        .expect("player entity")
}

/// The reach the reference gives a Bokoblin: the actor's own distance table
/// index for the battle type is 3 (`f_op_actor.cpp:289`), and that row of the
/// table (`d_att_dist.cpp:14`) carries 375 units of reach, 625 of release
/// reach and 500 either way of height.
const BATTLE_REACH: f32 = 375.0;

/// The lock takes the foe the reference marks for battle, and the face settles
/// on it.
#[test]
fn golden_the_lock_takes_the_bokoblin() {
    let mut app = common::floored(2000.0);
    let foe = spawn_bokoblin(&mut app, Vec3::new(50.0, common::FLOOR_Y, 100.0));
    let reach = (50.0_f32 * 50.0 + 100.0 * 100.0).sqrt();
    assert!(
        reach < BATTLE_REACH,
        "the foe is inside the reference's reach of {BATTLE_REACH}"
    );
    app.stage_target(true);
    for _ in 0..3 {
        common::step(&mut app);
    }
    assert_eq!(target(&app), Some(foe), "the press grabs the foe");
    let p = player(&app);
    let want = (50.0 - p.pos.x).atan2(100.0 - p.pos.z);
    assert!(
        (p.face - want).abs() < 0.001,
        "the face settles on the foe, {} against {want}",
        p.face
    );
}

/// The lock-on list is ordered by weight, not by distance: the weight is the
/// distance the player put between them (`distace_weight`, `d_attention.cpp`
/// `:293-297`) and the list is sorted by it (`sortList`, `d_attention.cpp`
/// `:490`), so the first entry of the lock-on list (`GetLockonList`,
/// `d_attention.cpp:128-134`) is the nearest.
#[test]
fn golden_the_lock_takes_the_nearest_bokoblin() {
    let mut app = common::floored(2000.0);
    let near = spawn_bokoblin(&mut app, Vec3::new(0.0, common::FLOOR_Y, 60.0));
    spawn_bokoblin(&mut app, Vec3::new(0.0, common::FLOOR_Y, 120.0));
    app.stage_target(true);
    common::step(&mut app);
    assert_eq!(target(&app), Some(near), "the nearest weight wins");
}

/// The King of Red Lions registers for the action half, not for the lock-on
/// list: it carries `fopAc_Attn_ACTION_SHIP_e` (`d_a_ship.cpp:4158`), and the
/// action list is a different table from the lock-on one
/// (`d_attention.cpp:21-27`).
#[test]
fn golden_the_ship_is_not_a_lock_candidate() {
    let mut app = common::floored(2000.0);
    app.sim.world.spawn((
        Actor {
            name: "Ship".to_owned(),
            parameters: 0,
            pos: Vec3::new(0.0, common::FLOOR_Y, 50.0),
            angle: [0; 3],
            set_id: 0,
            room: -1,
            ..Default::default()
        },
        Ship::default(),
    ));
    let foe = spawn_bokoblin(&mut app, Vec3::new(0.0, common::FLOOR_Y, 120.0));
    app.stage_target(true);
    common::step(&mut app);
    assert_eq!(
        target(&app),
        Some(foe),
        "the ship waits for its own action instead"
    );
}
