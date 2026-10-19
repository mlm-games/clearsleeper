//! The fixtures the integration tests share.
//!
//! A foe's own control only runs where there is ground under it - the port
//! freezes an enemy that has no floor - so the foe tests need a world with
//! one. The floor is a flat quad of collision geometry, written the way
//! `tests/collision.rs` writes its own tables and installed as the world's
//! `Collision` resource the way a stage's room set installs it.

#![allow(dead_code)]

use std::time::Duration;

use clearsleeper::sim::area::Area;
use clearsleeper::sim::collision::Collision;
use clearsleeper::sim::player::Player;
use clearsleeper::{App, SIM_HZ};
use glam::{Vec2, Vec3};
use repose_core::input::PhysicalKey;
use repose_core::runtime::Scheduler;

/// The height of the fixture floor. The reference's own actors find their
/// ground with the acch their process carries (`d_a_bk.cpp:3335` asks the
/// Bokoblin's acch for a ground hit, `d_a_mo2.cpp:3624` the Moblin's), so the
/// floor a test puts under a foe is the height that foe stands on.
pub const FLOOR_Y: f32 = 0.0;

/// The table the port's [`Collision::parse`] reads, in the layout
/// `tests/collision.rs` writes its own: the header, the vertices, then the
/// triangles with their two spare words.
pub fn dzb(verts: &[[f32; 3]], tris: &[[u16; 3]]) -> Vec<u8> {
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

/// A flat floor at `y` covering the square from `-half` to `half` on both
/// axes. The winding is the one `tests/collision.rs` uses, so the face points
/// up and the ground check reads it as ground.
pub fn floor(y: f32, half: f32) -> Collision {
    let verts = [
        [-half, y, -half],
        [-half, y, half],
        [half, y, half],
        [half, y, -half],
    ];
    let tris = [[0u16, 1, 2], [0, 2, 3]];
    Collision::parse(&dzb(&verts, &tris)).expect("the floor table parses")
}

/// A world with the fixture floor under it, so the foes that stand on it run
/// their control.
pub fn floored(half: f32) -> App {
    let mut app = App::new();
    app.sim.world.insert_resource(floor(FLOOR_Y, half));
    app
}

/// The area a room's collision bounds give the player: the `Area` the port
/// fills from the room set's own collision (`src/lib.rs`, the room install)
/// and the bounds the player is clamped to when there is no collision under
/// them (`sim::area::clamp_to_area`).
pub fn area(half: f32) -> Area {
    Area {
        bounds_min: Vec2::new(-half, -half),
        bounds_max: Vec2::new(half, half),
        spawn: Vec3::ZERO,
        yaw: 0.0,
    }
}

/// One sim tick.
pub fn step(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

/// The player put at `pos`, facing the way the camera looks.
pub fn place_player(app: &mut App, pos: Vec3) {
    let mut query = app.sim.world.query::<&mut Player>();
    for mut player in query.iter_mut(&mut app.sim.world) {
        player.pos = pos;
        player.yaw = 0.0;
        player.face = 0.0;
        player.speed = 0.0;
        player.speed_y = 0.0;
    }
}

/// The pad with `held` down.
pub fn keys(held: &[PhysicalKey]) -> Scheduler {
    let mut sched = Scheduler::default();
    sched.window_focused = true;
    sched.held_keys = held.iter().cloned().collect();
    sched
}

/// One polled frame: the pad goes down, the sim takes its tick.
pub fn frame(app: &mut App, held: &[PhysicalKey]) {
    app.stage_polled(&keys(held));
    step(app);
}
