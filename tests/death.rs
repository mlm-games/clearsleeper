use std::time::Duration;

use clearsleeper::sim::area::Area;
use clearsleeper::sim::collision::Collision;
use clearsleeper::sim::events::SimEvent;
use clearsleeper::sim::player::Player;
use clearsleeper::sim::sea::{Sea, SeaTables};
use clearsleeper::sim::swim::Swim;
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
        .expect("player exists at boot")
}

fn swim(app: &App) -> Swim {
    *app.sim.world.resource::<Swim>()
}

fn spawn(app: &App) -> Vec3 {
    app.sim.world.resource::<Area>().spawn
}

fn teleport(app: &mut App, pos: Vec3) {
    let mut query = app.sim.world.query::<&mut Player>();
    let mut player = query.iter_mut(&mut app.sim.world).next().expect("player");
    player.pos = pos;
    player.yaw = 0.0;
}

fn set_yaw(app: &mut App, yaw: f32) {
    let mut query = app.sim.world.query::<&mut Player>();
    let mut player = query.iter_mut(&mut app.sim.world).next().expect("player");
    player.yaw = yaw;
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

fn rect(x0: f32, x1: f32, z0: f32, z1: f32, y: f32) -> ([[f32; 3]; 4], [[u16; 3]; 2]) {
    (
        [[x0, y, z0], [x0, y, z1], [x1, y, z1], [x1, y, z0]],
        [[0, 1, 2], [0, 2, 3]],
    )
}

fn merge(
    acc: (Vec<[f32; 3]>, Vec<[u16; 3]>),
    quad: ([[f32; 3]; 4], [[u16; 3]; 2]),
) -> (Vec<[f32; 3]>, Vec<[u16; 3]>) {
    let (mut verts, mut tris) = acc;
    let offset = verts.len() as u16;
    verts.extend_from_slice(&quad.0);
    for mut tri in quad.1 {
        for id in &mut tri {
            *id += offset;
        }
        tris.push(tri);
    }
    (verts, tris)
}

fn swim_app() -> App {
    let mut app = App::new();
    let (verts, tris) = merge(
        merge(
            merge(
                merge(
                    (Vec::new(), Vec::new()),
                    rect(-1000.0, 1000.0, -1000.0, 1000.0, -500.0),
                ),
                rect(400.0, 600.0, 400.0, 600.0, -20.0),
            ),
            rect(-600.0, -400.0, -600.0, -400.0, -100.0),
        ),
        rect(-400.0, -200.0, -400.0, -200.0, -70.0),
    );
    app.sim
        .world
        .insert_resource(Collision::parse(&dzb(&verts, &tris)).expect("stage collision"));
    app.sim.world.insert_resource(Sea::new(SeaTables::new(
        |room| {
            if room == 9 { 0 } else { 1 }
        },
    )));
    app
}

#[test]
fn golden_drown_restarts_costs_a_quarter_and_clears_swim() {
    let mut app = swim_app();
    teleport(&mut app, Vec3::new(0.0, -500.0, 0.0));
    tick(&mut app);
    tick(&mut app);
    assert!(swim(&app).active, "deep water entry");

    app.sim.world.resource_mut::<Swim>().air = 0;
    tick(&mut app);

    let state = swim(&app);
    assert!(!state.active, "drowning leaves the water");
    assert_eq!(state.air, 0);
    let p = player(&app);
    assert_eq!(p.pos, spawn(&app), "restart returns to the spawn point");
    assert_eq!(p.speed, 0.0);
    assert_eq!(p.speed_y, 0.0);
    assert_eq!(app.health().units, 11, "drowning costs one quarter");
    assert_eq!(app.drain_events(), vec![SimEvent::Restart]);
}

#[test]
fn golden_lethal_damage_on_the_ground_respawns_at_spawn() {
    let mut app = App::new();
    teleport(&mut app, Vec3::new(500.0, 0.0, 500.0));
    set_yaw(&mut app, 1.0);

    app.stage_damage(12);
    tick(&mut app);

    let p = player(&app);
    assert_eq!(p.pos, spawn(&app), "death returns to the spawn point");
    assert_eq!(p.yaw, 0.0, "respawn restores the area yaw");
    assert_eq!(app.health().units, 12, "continue restarts at three hearts");
    assert_eq!(app.drain_events(), vec![SimEvent::Restart]);
}

#[test]
fn golden_airborne_death_falls_frozen_then_respawns() {
    let mut app = App::new();
    teleport(&mut app, Vec3::new(500.0, 200.0, 500.0));
    app.stage_axis(Vec2::new(0.0, 1.0));
    app.stage_damage(12);
    tick(&mut app);

    let p = player(&app);
    assert_eq!(app.health().units, 0, "lethal damage lands");
    assert_eq!(p.pos.y, 197.5, "the death tick still falls");
    assert_eq!(p.pos.z, 503.5, "the death tick still walks");
    let frozen_z = p.pos.z;

    app.stage_damage(0);
    tick(&mut app);

    let p = player(&app);
    assert_eq!(app.health().units, 0, "the body stays dead while falling");
    assert_eq!(p.speed, 0.0, "death freezes the walk drive");
    assert_eq!(p.pos.z, frozen_z, "the held stick moves nothing");

    let mut respawn = None;
    for _ in 0..40 {
        tick(&mut app);
        if app.health().units == 12 {
            respawn = Some(player(&app).pos);
            break;
        }
    }
    let pos = respawn.expect("the fall lands and restarts");
    assert_eq!(pos, spawn(&app), "the fall restarts at the spawn point");
    assert_eq!(app.drain_events(), vec![SimEvent::Restart]);
}

#[test]
fn golden_lethal_damage_while_swimming_respawns_without_falling() {
    let mut app = swim_app();
    teleport(&mut app, Vec3::new(0.0, -500.0, 0.0));
    tick(&mut app);
    tick(&mut app);
    assert!(swim(&app).active, "deep water entry");

    app.stage_damage(12);
    tick(&mut app);

    assert!(!swim(&app).active, "death ends swimming at once");
    let p = player(&app);
    assert_eq!(p.pos, spawn(&app), "swim death restarts at the spawn point");
    assert_eq!(app.health().units, 12);
    assert_eq!(app.drain_events(), vec![SimEvent::Restart]);
}
