use std::time::Duration;

use clearsleeper::sim::collision::Collision;
use clearsleeper::sim::hop::{Hop, launch};
use clearsleeper::sim::player::Player;
use clearsleeper::sim::roll::Roll;
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

fn hop(app: &App) -> Hop {
    *app.sim.world.resource::<Hop>()
}

fn roll(app: &App) -> Roll {
    *app.sim.world.resource::<Roll>()
}

fn place(app: &mut App, pos: Vec3, yaw: f32, speed: f32) {
    let mut query = app.sim.world.query::<&mut Player>();
    let mut player = query.iter_mut(&mut app.sim.world).next().expect("player");
    player.pos = pos;
    player.yaw = yaw;
    player.speed = speed;
    player.speed_y = 0.0;
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

fn ledge_app(bottom_z: f32, bottom_y: f32) -> App {
    let mut app = App::new();
    let (verts, tris) = merge(
        merge(
            (Vec::new(), Vec::new()),
            rect(-1000.0, 1000.0, -1000.0, 20.0, 0.0),
        ),
        rect(-1000.0, 1000.0, bottom_z, 1000.0, bottom_y),
    );
    app.sim
        .world
        .insert_resource(Collision::parse(&dzb(&verts, &tris)).expect("stage collision"));
    app
}

#[test]
fn golden_sprint_off_ledge_leaps() {
    let mut app = ledge_app(-1000.0, -100.0);
    place(&mut app, Vec3::new(0.0, 0.0, 19.0), 0.0, 17.0);
    app.stage_axis(Vec2::new(0.0, 1.0));
    tick(&mut app);
    assert!(!hop(&app).active);
    assert_eq!(player(&app).pos, Vec3::new(0.0, -2.5, 36.0));
    tick(&mut app);
    assert!(hop(&app).active);
    let (forward, up) = launch(17.0);
    let p = player(&app);
    assert_eq!(p.speed, forward + 0.4);
    // `procAutoJump`, `main.cpp:7151-7172`: the extra half-step off the rise belongs to the
    // tick that still holds a grabbed item, which a ledge hop never does.
    assert_eq!(p.speed_y, up - 2.5);
    assert_eq!(
        p.pos,
        Vec3::new(0.0, -2.5 + (up - 2.5), 36.0 + (forward + 0.4))
    );
}

#[test]
fn golden_slow_walkoff_falls_plain() {
    let mut app = ledge_app(-1000.0, -100.0);
    place(&mut app, Vec3::new(0.0, 0.0, 19.0), 0.0, 5.0);
    app.stage_axis(Vec2::new(0.0, 1.0));
    tick(&mut app);
    assert_eq!(player(&app).pos.z, 27.5);
    tick(&mut app);
    assert!(!hop(&app).active);
    for _ in 0..18 {
        tick(&mut app);
    }
    assert!(!hop(&app).active);
    assert_eq!(player(&app).pos.y, -102.5);
}

#[test]
fn golden_small_step_falls_plain() {
    let mut app = ledge_app(20.0, -20.0);
    place(&mut app, Vec3::new(0.0, 0.0, 19.0), 0.0, 17.0);
    app.stage_axis(Vec2::new(0.0, 1.0));
    tick(&mut app);
    tick(&mut app);
    assert!(!hop(&app).active);
    for _ in 0..13 {
        tick(&mut app);
    }
    assert!(!hop(&app).active);
    assert_eq!(player(&app).pos.y, -22.5);
}

#[test]
fn golden_hop_gates_stick_and_facing() {
    let mut slow = ledge_app(-1000.0, -100.0);
    place(&mut slow, Vec3::new(0.0, 0.0, 19.0), 0.0, 17.0);
    slow.stage_axis(Vec2::ZERO);
    tick(&mut slow);
    tick(&mut slow);
    assert!(!hop(&slow).active);
    for _ in 0..18 {
        tick(&mut slow);
    }
    assert!(!hop(&slow).active);
    assert_eq!(player(&slow).pos.y, -102.5);

    let mut angled = ledge_app(-1000.0, -100.0);
    place(&mut angled, Vec3::new(0.0, 0.0, 19.0), 0.5, 17.0);
    angled.stage_axis(Vec2::new(0.0, 1.0));
    tick(&mut angled);
    tick(&mut angled);
    assert!(!hop(&angled).active);
    for _ in 0..18 {
        tick(&mut angled);
    }
    assert!(!hop(&angled).active);
    assert_eq!(player(&angled).pos.y, -102.5);
}

#[test]
fn golden_hop_rearms_after_landing() {
    let mut app = ledge_app(-1000.0, -100.0);
    place(&mut app, Vec3::new(0.0, 0.0, 19.0), 0.0, 17.0);
    app.stage_axis(Vec2::new(0.0, 1.0));
    tick(&mut app);
    tick(&mut app);
    assert!(hop(&app).active);
    for _ in 0..60 {
        if !hop(&app).active {
            break;
        }
        tick(&mut app);
    }
    assert!(!hop(&app).active);
    place(&mut app, Vec3::new(0.0, 0.0, 19.0), 0.0, 17.0);
    tick(&mut app);
    tick(&mut app);
    assert!(hop(&app).active);
    let (forward, _) = launch(17.0);
    assert_eq!(player(&app).speed, forward + 0.4);
}

#[test]
fn golden_roll_off_ledge_leaps() {
    let mut app = ledge_app(-1000.0, -100.0);
    place(&mut app, Vec3::new(0.0, 0.0, 19.0), 0.0, 17.0);
    app.stage_axis(Vec2::new(0.0, 1.0));
    app.stage_do(true);
    tick(&mut app);
    assert!(roll(&app).active);
    assert!(!hop(&app).active);
    assert_eq!(player(&app).pos.z, 45.0);
    tick(&mut app);
    assert!(hop(&app).active);
    assert!(!roll(&app).active);
    let (forward, up) = launch(17.0);
    let p = player(&app);
    assert_eq!(p.speed, forward + 0.4);
    // `procAutoJump`, `main.cpp:7151-7172`: the rise only carries the gravity.
    assert_eq!(p.speed_y, up - 2.5);
    assert_eq!(p.pos.z, 45.0 + (forward + 0.4));
}
