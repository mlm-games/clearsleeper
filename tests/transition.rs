use std::time::Duration;

use clearsleeper::dzs;
use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::area::Area;
use clearsleeper::sim::collision::{Collision, NO_EXIT};
use clearsleeper::sim::health::Health;
use clearsleeper::sim::player::Player;
use clearsleeper::sim::sea::{Sea, SeaTables};
use clearsleeper::sim::swim::Swim;
use clearsleeper::sim::transition::NextStage;
use clearsleeper::{App, SIM_HZ};
use glam::Vec3;

fn tick(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

fn step(app: &mut App) {
    app.sim.step(Duration::from_secs_f64(1.0 / SIM_HZ));
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

fn next_stage(app: &App) -> NextStage {
    app.sim.world.resource::<NextStage>().clone()
}

fn floor_quad(y: f32) -> ([[f32; 3]; 4], [[u16; 3]; 2]) {
    (
        [
            [0.0, y, 0.0],
            [0.0, y, 10.0],
            [10.0, y, 10.0],
            [10.0, y, 0.0],
        ],
        [[0, 1, 2], [0, 2, 3]],
    )
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
        out.extend_from_slice(&0_u16.to_be_bytes());
        out.extend_from_slice(&0_u16.to_be_bytes());
    }
    out
}

fn dzb_exit(verts: &[[f32; 3]], tris: &[[u16; 3]], exits: &[u8]) -> Vec<u8> {
    let mut data = dzb(verts, tris);
    let info_offset = data.len() as u32;
    for &exit in exits {
        data.extend_from_slice(&((exit as u32) << 13).to_be_bytes());
        data.extend_from_slice(&[0_u8; 12]);
    }
    data[0x28..0x2C].copy_from_slice(&(exits.len() as u32).to_be_bytes());
    data[0x2C..0x30].copy_from_slice(&info_offset.to_be_bytes());
    let tri_offset = 0x34 + (verts.len() as u32) * 12;
    for index in 0..tris.len() {
        let at = (tri_offset + index as u32 * 10 + 6) as usize;
        data[at..at + 2].copy_from_slice(&(index as u16).to_be_bytes());
    }
    data
}

fn interior() -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&2_u32.to_be_bytes());
    data.extend_from_slice(b"ACTR");
    data.extend_from_slice(&1_u32.to_be_bytes());
    data.extend_from_slice(&28_u32.to_be_bytes());
    data.extend_from_slice(b"SCLS");
    data.extend_from_slice(&1_u32.to_be_bytes());
    data.extend_from_slice(&60_u32.to_be_bytes());

    data.extend_from_slice(b"Pot\0\0\0\0\0");
    data.extend_from_slice(&0_u32.to_be_bytes());
    data.extend_from_slice(&1.0_f32.to_be_bytes());
    data.extend_from_slice(&0.0_f32.to_be_bytes());
    data.extend_from_slice(&2.0_f32.to_be_bytes());
    data.extend_from_slice(&0_i16.to_be_bytes());
    data.extend_from_slice(&0_i16.to_be_bytes());
    data.extend_from_slice(&0_i16.to_be_bytes());
    data.extend_from_slice(&0_u16.to_be_bytes());

    data.extend_from_slice(b"Nowhere\0");
    data.push(0);
    data.push(0);
    data.push(0);
    data.push(0);
    data
}

#[test]
fn golden_exit_poly_stages_a_transition() {
    let mut app = App::new();
    let (verts, tris) = floor_quad(0.0);
    app.sim
        .world
        .insert_resource(Collision::parse(&dzb_exit(&verts, &tris, &[3, NO_EXIT])).unwrap());

    teleport(&mut app, Vec3::new(1.0, 0.0, 9.0));
    step(&mut app);
    assert_eq!(
        next_stage(&app),
        NextStage {
            exit: Some(3),
            target: None,
            ..Default::default()
        },
        "standing on an exit poly stages its exit id"
    );

    teleport(&mut app, Vec3::new(9.0, 0.0, 1.0));
    step(&mut app);
    assert_eq!(
        next_stage(&app),
        NextStage::default(),
        "walking off the exit clears the pending stage"
    );

    teleport(&mut app, Vec3::new(50.0, 0.0, 50.0));
    step(&mut app);
    assert_eq!(
        next_stage(&app),
        NextStage::default(),
        "ground with no exit poly stages nothing"
    );
}

#[test]
fn golden_dead_body_stages_no_transition() {
    let mut app = App::new();
    let (verts, tris) = floor_quad(0.0);
    app.sim
        .world
        .insert_resource(Collision::parse(&dzb_exit(&verts, &tris, &[3, NO_EXIT])).unwrap());
    teleport(&mut app, Vec3::new(1.0, 0.0, 9.0));
    app.sim.world.resource_mut::<Health>().units = 0;

    step(&mut app);
    assert_eq!(
        next_stage(&app),
        NextStage::default(),
        "the corpse stays put through its final rest"
    );
}

#[test]
fn golden_swap_stage_teleports_and_rebuilds() {
    let mut app = App::new();
    app.sim
        .world
        .insert_resource(Sea::new(SeaTables::new(|_| 5)));
    app.sim.world.spawn(Actor {
        name: "Ship".to_owned(),
        parameters: 0,
        pos: Vec3::ZERO,
        angle: [0; 3],
        set_id: 0,
        room: -1,
        ..Default::default()
    });
    app.sim.world.resource_mut::<Swim>().active = true;
    app.sim.world.resource_mut::<Health>().units = 7;
    *app.sim.world.resource_mut::<NextStage>() = NextStage {
        exit: Some(9),
        target: None,
        ..Default::default()
    };
    {
        let mut query = app.sim.world.query::<&mut Player>();
        let mut player = query.iter_mut(&mut app.sim.world).next().unwrap();
        player.pos = Vec3::new(-50.0, 0.0, -50.0);
        player.speed = 4.0;
    }

    let tables = dzs::parse(&interior()).unwrap().with_name("House01");
    app.swap_stage(tables, vec![0], Vec3::new(5.0, 0.0, 7.0), 1.5, clearsleeper::sim::transition::LAYER_NONE);

    let player = player(&app);
    assert_eq!(player.pos, Vec3::new(5.0, 0.0, 7.0));
    assert_eq!(player.yaw, 1.5);
    assert_eq!(player.speed, 0.0, "momentum resets on arrival");
    assert!(
        !app.sim.world.resource::<Swim>().active,
        "the swim state is left behind"
    );
    assert!(
        app.sim.world.get_resource::<Sea>().is_none(),
        "an interior stage carries no sea"
    );
    assert_eq!(next_stage(&app), NextStage::default());
    let area = app.sim.world.resource::<Area>();
    assert_eq!(area.spawn, Vec3::new(5.0, 0.0, 7.0));
    assert_eq!(area.yaw, 1.5);
    let names: Vec<String> = app
        .sim
        .world
        .iter_entities()
        .filter_map(|entity| entity.get::<Actor>().map(|actor| actor.name.clone()))
        .collect();
    assert!(
        !names.iter().any(|name| name == "Ship"),
        "the previous stage's actors are gone"
    );
    assert!(
        names.iter().any(|name| name == "Pot"),
        "the destination's stage actors spawn"
    );
    assert_eq!(app.stage().unwrap().name(), "House01");
    assert_eq!(app.health().units, 7, "game state survives the swap");
}

#[test]
fn golden_apply_exit_keeps_the_stage_when_the_target_is_missing() {
    let mut app = App::new();
    let tables = dzs::parse(&interior()).unwrap().with_name("House01");
    app.swap_stage(tables, vec![0], Vec3::new(1.0, 0.0, 9.0), 0.0, clearsleeper::sim::transition::LAYER_NONE);
    let (verts, tris) = floor_quad(0.0);
    app.sim
        .world
        .insert_resource(Collision::parse(&dzb_exit(&verts, &tris, &[0, NO_EXIT])).unwrap());

    tick(&mut app);

    assert_eq!(
        app.stage().unwrap().name(),
        "House01",
        "an unknown destination keeps the current stage"
    );
    assert_eq!(
        next_stage(&app),
        NextStage::default(),
        "the shell consumes the pending exit"
    );
    let player = player(&app);
    assert_eq!(
        player.pos,
        Vec3::new(1.0, 0.0, 9.0),
        "the player did not move"
    );
}
