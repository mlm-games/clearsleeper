use std::time::Duration;

use clearsleeper::sim::collision::Collision;
use clearsleeper::sim::player::Player;
use clearsleeper::sim::sea::{Sea, SeaTables};
use clearsleeper::sim::swim::Swim;
use clearsleeper::sim::water::Water;
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

fn water(app: &App) -> Water {
    *app.sim.world.resource::<Water>()
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
    app.sim.world.insert_resource(Sea::new(SeaTables::new(|room| {
        if room == 9 {
            0
        } else {
            1
        }
    })));
    app
}

#[test]
fn golden_entry_requires_ninety_units_of_depth() {
    let mut app = swim_app();

    teleport(&mut app, Vec3::new(-300.0, -70.0, -300.0));
    for _ in 0..4 {
        tick(&mut app);
    }
    assert!(!swim(&app).active, "81 units of water is wading depth");
    assert_eq!(player(&app).pos.y, -72.5);

    teleport(&mut app, Vec3::new(-500.0, -100.0, -500.0));
    tick(&mut app);
    let state = swim(&app);
    assert!(state.active, "101 units of water starts swimming");
    assert!(state.surfaced, "grounded entry surfaces immediately");
    assert_eq!(player(&app).pos.y, water(&app).surface);
}

#[test]
fn golden_deep_water_plunges_then_buoys_to_the_surface() {
    let mut app = swim_app();
    teleport(&mut app, Vec3::ZERO);

    let mut entered = 0_u32;
    let mut surfaced_at = 0_u32;
    let mut last_speed_y = 0.0_f32;
    for step in 1..=40 {
        tick(&mut app);
        let state = swim(&app);
        let surface = water(&app);
        let p = player(&app);
        if state.active && entered == 0 {
            entered = step;
            assert!(!state.surfaced, "falling entry plunges below the surface");
            assert!(p.pos.y < surface.surface, "plunge starts submerged");
        }
        if state.active && !state.surfaced {
            let base = last_speed_y.max(-50.0);
            assert_eq!(
                p.speed_y,
                (base + 6.0).min(9.5),
                "buoy rises 6.0 per tick capped at 9.5 at step {step}"
            );
        }
        if state.active && state.surfaced {
            if surfaced_at == 0 {
                surfaced_at = step;
            }
            assert_eq!(p.pos.y, surface.surface, "surface ride tracks the wave");
            assert_eq!(p.speed_y, 0.0);
        }
        last_speed_y = p.speed_y;
    }

    assert!(entered > 0, "open water catches the falling player");
    assert!(surfaced_at > entered, "buoyancy returns Link to the surface");
    assert_eq!(
        swim(&app).air,
        900 - (40 - entered + 1) as i32,
        "deep water drains one breath per tick"
    );
}

#[test]
fn golden_shallow_shore_stands_up_and_clears_breath() {
    let mut app = swim_app();
    teleport(&mut app, Vec3::new(0.0, -500.0, 0.0));
    tick(&mut app);
    tick(&mut app);
    assert!(swim(&app).active, "deep water entry");

    teleport(&mut app, Vec3::new(500.0, -20.0, 500.0));
    tick(&mut app);
    let state = swim(&app);
    assert!(!state.active, "shallow floor stands Link up");
    assert_eq!(state.air, 0);
    assert_eq!(
        player(&app).pos.y,
        water(&app).surface - 2.5,
        "standing at the surface then falls to the floor"
    );
}

#[test]
fn golden_leaving_the_sea_area_falls() {
    let mut app = swim_app();
    teleport(&mut app, Vec3::new(0.0, -500.0, 0.0));
    tick(&mut app);
    tick(&mut app);
    assert!(swim(&app).active, "deep water entry");

    app.stage_axis(Vec2::new(0.0, 1.0));
    for _ in 0..4 {
        tick(&mut app);
    }
    assert_eq!(player(&app).speed, 12.0);
    app.stage_axis(Vec2::ZERO);

    teleport(&mut app, Vec3::new(-200_000.0, 1.0, -200_000.0));
    tick(&mut app);
    let state = swim(&app);
    assert!(!state.active, "culled sea cell ends swimming");
    assert_eq!(state.air, 0);
    assert_eq!(player(&app).speed, 0.0, "fall starts from rest");
    assert_eq!(player(&app).speed_y, -2.5, "gravity resumes the same tick");
}

#[test]
fn golden_swim_speed_steps_three_units_per_tick() {
    let mut app = swim_app();
    teleport(&mut app, Vec3::new(0.0, -500.0, 0.0));
    tick(&mut app);
    tick(&mut app);
    let state = swim(&app);
    assert!(state.active && state.surfaced);

    app.stage_axis(Vec2::new(0.0, 1.0));
    let mut z = 0.0;
    for expected_speed in [3.0, 6.0, 9.0, 12.0, 15.0, 18.0, 18.0] {
        tick(&mut app);
        z += expected_speed;
        let p = player(&app);
        assert_eq!(p.speed, expected_speed);
        assert_eq!(p.pos.z, z);
        assert_eq!(p.pos.x, 0.0);
        assert_eq!(p.yaw, 0.0);
        assert_eq!(p.pos.y, water(&app).surface, "ride keeps pace with waves");
    }
}

#[test]
fn golden_air_drains_only_in_deep_water() {
    let mut app = swim_app();
    teleport(&mut app, Vec3::new(0.0, -500.0, 0.0));
    tick(&mut app);
    tick(&mut app);
    assert!(swim(&app).active);
    let before = swim(&app).air;
    assert_eq!(before, 899);

    for _ in 0..50 {
        tick(&mut app);
    }
    assert_eq!(swim(&app).air, before - 50);

    teleport(&mut app, Vec3::new(-500.0, -100.0, -500.0));
    tick(&mut app);
    assert!(swim(&app).active, "mid-depth water keeps swimming");
    assert_eq!(swim(&app).air, 900, "shallow water refills the breath meter");
    for _ in 0..10 {
        tick(&mut app);
    }
    assert_eq!(swim(&app).air, 900);
}

#[test]
fn golden_snapshot_ground_y_tracks_the_water_surface() {
    let mut app = swim_app();
    teleport(&mut app, Vec3::new(0.0, -500.0, 0.0));
    tick(&mut app);
    tick(&mut app);
    assert!(swim(&app).active);

    assert_eq!(app.snapshot().ground_y, water(&app).surface);

    teleport(&mut app, Vec3::new(-300.0, -70.0, -300.0));
    for _ in 0..12 {
        tick(&mut app);
    }
    assert!(!swim(&app).active, "wading never swims");
    assert_eq!(player(&app).pos.y, -72.5);
    assert_eq!(app.snapshot().ground_y, -70.0, "wading keeps the floor camera");
}
