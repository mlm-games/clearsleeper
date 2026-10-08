use std::time::Duration;

use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::collision::Collision;
use clearsleeper::sim::gull::{self, Gull};
use clearsleeper::sim::player::Player;
use clearsleeper::{App, SIM_HZ};
use glam::Vec3;

const GROUND: u32 = 1;

fn tick(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

fn place_player(app: &mut App, pos: Vec3) {
    let mut query = app.sim.world.query::<&mut Player>();
    for mut found in query.iter_mut(&mut app.sim.world) {
        found.pos = pos;
        found.yaw = 0.0;
        found.face = 0.0;
        found.speed = 0.0;
        found.speed_y = 0.0;
    }
}

fn bird(app: &mut App, pos: Vec3, parameters: u32) -> bevy_ecs::entity::Entity {
    let actor = Actor {
        name: gull::GULL_ACTOR.to_owned(),
        parameters,
        pos,
        angle: [0, 0, 0],
        set_id: 0,
        room: -1,
        ..Default::default()
    };
    let state = gull::spawn(parameters, &actor);
    app.sim.world.spawn((actor, state)).id()
}

fn dzb(verts: &[[f32; 3]], tris: &[[u16; 3]]) -> Vec<u8> {
    let vertex_count = verts.len() as u32;
    let tri_count = tris.len() as u32;
    let vertex_offset = 0x34_u32;
    let tri_offset = vertex_offset + vertex_count * 12;
    let end = tri_offset + tri_count * 10;
    let mut out = Vec::new();
    for value in [
        vertex_count,
        vertex_offset,
        tri_count,
        tri_offset,
        0,
        end,
        0,
        end,
        0,
        end,
        0,
        end,
        0,
    ] {
        out.extend_from_slice(&value.to_be_bytes());
    }
    for vert in verts {
        for &axis in vert {
            out.extend_from_slice(&axis.to_be_bytes());
        }
    }
    for tri in tris {
        for &id in tri {
            out.extend_from_slice(&id.to_be_bytes());
        }
    }
    out
}

fn floor(app: &mut App) {
    let corners = [
        [-4000.0, 0.0, -4000.0],
        [4000.0, 0.0, -4000.0],
        [4000.0, 0.0, 4000.0],
        [-4000.0, 0.0, 4000.0],
    ];
    let tris = [[0, 1, 2], [0, 2, 3]];
    let collision = Collision::parse(&dzb(&corners, &tris)).expect("flat floor parses");
    app.sim.world.insert_resource(collision);
}

fn gull(app: &App, entity: bevy_ecs::entity::Entity) -> Gull {
    app.sim
        .world
        .entity(entity)
        .get::<Gull>()
        .unwrap()
        .to_owned()
}

#[test]
fn golden_perching_bird_waits_for_someone_to_come_close() {
    let mut app = App::new();
    floor(&mut app);
    place_player(&mut app, Vec3::new(0.0, 0.0, 5_000.0));
    let bird_id = bird(&mut app, Vec3::ZERO, GROUND);
    for _ in 0..30 {
        tick(&mut app);
    }
    let state = gull(&app, bird_id);
    assert!(!state.flying, "it holds its perch out of arm's reach");
    assert_eq!(state.height, 0.0, "and stays on the ground");
}

#[test]
fn golden_walking_up_scatters_the_flock() {
    let mut app = App::new();
    floor(&mut app);
    place_player(&mut app, Vec3::new(0.0, 0.0, 200.0));
    let bird_id = bird(&mut app, Vec3::ZERO, GROUND);
    tick(&mut app);
    let state = gull(&app, bird_id);
    assert!(state.flying, "inside 400 units it takes wing");
    assert!(state.startled > 0, "and beats away from you");
}
