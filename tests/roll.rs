use std::time::Duration;

use clearsleeper::sim::collision::Collision;
use clearsleeper::sim::events::SimEvent;
use clearsleeper::sim::player::Player;
use clearsleeper::sim::roll::Roll;
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

fn roll(app: &App) -> Roll {
    *app.sim.world.resource::<Roll>()
}

fn swim(app: &App) -> Swim {
    *app.sim.world.resource::<Swim>()
}

fn teleport(app: &mut App, pos: Vec3) {
    let mut query = app.sim.world.query::<&mut Player>();
    let mut player = query.iter_mut(&mut app.sim.world).next().expect("player");
    player.pos = pos;
    player.yaw = 0.0;
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
fn golden_roll_runs_eighteen_ticks_and_hands_back_to_walk() {
    let mut app = App::new();
    app.stage_axis(Vec2::new(0.0, 1.0));
    app.stage_do(true);
    tick(&mut app);

    assert!(roll(&app).active, "stick plus do starts the roll");
    let p = player(&app);
    assert_eq!(p.speed, 5.0, "roll speed floors at five");
    assert_eq!(p.pos.x, 0.0);
    assert_eq!(p.pos.z, 5.0, "the roll step lands on the trigger tick");
    assert_eq!(p.pos.y, -2.5, "gravity keeps running during the roll");
    assert_eq!(p.yaw, 0.0);

    for k in 2..=18 {
        tick(&mut app);
        assert!(roll(&app).active, "the roll holds at tick {k}");
        let p = player(&app);
        assert_eq!(p.speed, 5.0, "the roll ignores walk accel at tick {k}");
        assert_eq!(p.pos.z, 5.0 * k as f32, "straight line at tick {k}");
        assert_eq!(p.pos.x, 0.0, "yaw stays locked at tick {k}");
    }

    tick(&mut app);
    assert!(!roll(&app).active, "eighteen ticks later the roll ends");
    let p = player(&app);
    assert_eq!(p.speed, 8.5, "walk resumes from the roll speed");
    assert_eq!(p.pos.z, 98.5);
}

#[test]
fn golden_roll_ignores_stick_turns() {
    let mut app = App::new();
    app.stage_axis(Vec2::new(0.0, 1.0));
    app.stage_do(true);
    tick(&mut app);
    app.stage_axis(Vec2::new(1.0, 0.0));

    for k in 2..=18 {
        tick(&mut app);
        assert!(roll(&app).active);
        let p = player(&app);
        assert_eq!(p.yaw, 0.0, "the roll holds its heading at tick {k}");
        assert_eq!(p.pos.x, 0.0, "no sideways drift at tick {k}");
        assert_eq!(p.pos.z, 5.0 * k as f32);
    }
}

#[test]
fn golden_do_press_gate_uses_the_stick_threshold() {
    let mut app = App::new();
    app.stage_axis(Vec2::ZERO);
    app.stage_do(true);
    tick(&mut app);
    assert!(!roll(&app).active, "a standing do press stays blank");
    assert_eq!(player(&app).pos.z, 0.0);

    let mut app = App::new();
    app.stage_axis(Vec2::new(0.0, 0.5));
    app.stage_do(true);
    tick(&mut app);
    assert!(!roll(&app).active, "half stick sits under the roll gate");

    let mut app = App::new();
    app.stage_axis(Vec2::new(0.0, 0.8));
    app.stage_do(true);
    tick(&mut app);
    assert!(roll(&app).active, "eighty percent stick rolls");
}

#[test]
fn golden_mid_roll_press_does_not_restart() {
    let mut app = App::new();
    app.stage_axis(Vec2::new(0.0, 1.0));
    app.stage_do(true);
    tick(&mut app);

    for _ in 0..5 {
        tick(&mut app);
    }
    app.stage_do(false);
    tick(&mut app);
    app.stage_do(true);
    tick(&mut app);

    assert!(roll(&app).active);
    assert_eq!(
        player(&app).speed,
        5.0,
        "a mid-roll press leaves the running roll alone"
    );

    for _ in 0..11 {
        tick(&mut app);
    }
    assert!(!roll(&app).active, "the original roll ends on schedule");
    assert_eq!(player(&app).pos.z, 98.5);
}

#[test]
fn golden_roll_edge_fires_once_and_rearms() {
    let mut app = App::new();
    app.stage_axis(Vec2::new(0.0, 1.0));
    app.stage_do(true);
    tick(&mut app);
    assert!(roll(&app).active);

    for _ in 0..25 {
        tick(&mut app);
    }
    assert!(!roll(&app).active, "one press makes one roll");
    assert_eq!(player(&app).speed, 17.0, "walk rebuilt to full run");

    app.stage_do(false);
    tick(&mut app);
    app.stage_do(true);
    tick(&mut app);
    assert!(roll(&app).active, "release and press rolls again");
    assert_eq!(player(&app).speed, 26.0, "full run hits the roll speed cap");
}

#[test]
fn golden_roll_never_starts_in_water() {
    let mut app = swim_app();
    teleport(&mut app, Vec3::new(0.0, -500.0, 0.0));
    tick(&mut app);
    tick(&mut app);
    assert!(swim(&app).active, "deep water entry");

    app.stage_axis(Vec2::new(0.0, 1.0));
    app.stage_do(true);
    tick(&mut app);
    assert!(!roll(&app).active, "swimming skips the roll");
}

#[test]
fn golden_respawn_clears_a_running_roll() {
    let mut app = App::new();
    app.stage_axis(Vec2::new(0.0, 1.0));
    app.stage_do(true);
    app.stage_damage(12);
    tick(&mut app);

    assert!(!roll(&app).active, "respawn drops the roll");
    let p = player(&app);
    assert_eq!(p.pos, Vec3::ZERO, "restart returns to the spawn point");
    assert_eq!(p.speed, 0.0);
    assert_eq!(app.health().units, 12);
    assert_eq!(app.drain_events(), vec![SimEvent::Restart]);
}
