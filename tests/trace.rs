use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use bevy_ecs::prelude::World;
use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::area::StageName;
use clearsleeper::sim::combat::HitPoints;
use clearsleeper::sim::door::StayRoom;
use clearsleeper::sim::event::ActiveEvent;
use clearsleeper::sim::flags::{EventFlags, StoryBits, Switches};
use clearsleeper::sim::frame::CurrentFrame;
use clearsleeper::sim::health::Health;
use clearsleeper::sim::link_proc::{LinkProc, Proc};
use clearsleeper::sim::pickup::Rupees;
use clearsleeper::sim::player::Player;
use clearsleeper::sim::registry::ActorRegistry;
use clearsleeper::sim::talk::Dialogue;
use clearsleeper::trace::{self, TraceError};
use clearsleeper::{App, SIM_HZ};
use glam::Vec3;
use repose_core::input::PhysicalKey;
use repose_core::runtime::Scheduler;

static TEMP: AtomicUsize = AtomicUsize::new(0);

fn temp_path(tag: &str) -> PathBuf {
    let spot = TEMP.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("cs-trace-{}-{spot}-{tag}", std::process::id()))
}

fn frame(app: &mut App, keys: &[PhysicalKey]) {
    let mut sched = Scheduler::default();
    sched.window_focused = true;
    sched.held_keys = keys.iter().cloned().collect();
    app.stage_polled(&sched);
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

#[test]
fn trace_emission_is_deterministic() {
    let first = temp_path("a");
    let second = temp_path("b");
    let left = session(&first);
    let right = session(&second);
    assert_eq!(left, right, "two sessions must trace identically");
    let rows = trace::parse_text(&left).expect("own lines parse");
    assert_eq!(rows.len(), 40);
    for (at, row) in rows.iter().enumerate() {
        let expected = (at + 1).to_string();
        assert_eq!(row.tick(), Some(expected.as_str()));
    }
    let _ = fs::remove_file(&first);
    let _ = fs::remove_file(&second);
}

fn session(path: &Path) -> String {
    assert!(
        !path.exists(),
        "temp path must be fresh: {}",
        path.display()
    );
    unsafe { std::env::set_var("CS_TRACE", path) };
    let mut app = App::new();
    unsafe { std::env::remove_var("CS_TRACE") };
    for _ in 0..20 {
        frame(&mut app, &[PhysicalKey::KeyW]);
        frame(&mut app, &[]);
    }
    fs::read_to_string(path).expect("trace file written")
}

#[test]
fn trace_line_format() {
    let mut world = World::new();
    world.insert_resource(CurrentFrame(3));
    world.insert_resource(StageName("sea".to_string()));
    world.insert_resource(StayRoom(2));
    world.spawn(Player {
        pos: Vec3::new(0.0, 0.0, 1.0),
        yaw: 1.0,
        face: -2.0,
        speed: 17.0,
        speed_y: 1.5,
        drift: Vec3::ZERO,
        no_draw: false,
    });
    world.insert_resource(LinkProc::new(Proc::Move));
    world.insert_resource(Health { units: 7, max: 12 });
    world.insert_resource(Rupees { count: 5 });
    world.insert_resource(Switches::from_words([0, 0, 0x100, 0]));
    world.insert_resource(EventFlags::default());
    let mut bits = StoryBits::default();
    bits.bytes[255] = 0x07;
    world.insert_resource(bits);
    world.insert_resource(ActiveEvent::default());
    world.insert_resource(Dialogue::default());
    let body = world
        .spawn(Actor {
            name: "Bta".to_string(),
            angle: [0, 7730, 0],
            ..Actor::default()
        })
        .insert(HitPoints(2))
        .id();
    world.insert_resource(ActorRegistry::default());
    world.resource_mut::<ActorRegistry>().register(body, 0, 0);

    let line = trace::line_of(&world);
    let expected = "tick=3 stage=sea room=2 px=0x00000000 py=0x00000000 pz=0x3f800000 yaw=1065353216 face=3221225472 speed=0x41880000 speed_y=0x3fc00000 proc=Move hp=7 hpmax=12 rupees=5 sw=0x010000000000 ef=0x0 sb=0x07 evt=- evtst=- talk=0 rng=0x00000000 a1:kind=3067909926,x=0x00000000,y=0x00000000,z=0x00000000,yaw=7730,hp=2";
    assert_eq!(line, expected);
    let parsed = trace::parse_line(&line).expect("own line parses");
    assert_eq!(parsed.to_line(), line);
    assert_eq!(parsed.field("tick"), Some("3"));
}

#[test]
fn trace_diff_reports_first_tick() {
    let tick_line = |no: usize, sw: &str| {
        format!(
            "tick={no} stage=- room=0 px=0x00000000 py=0x00000000 pz=0x00000000 yaw=0 face=0 speed=0x00000000 speed_y=0x00000000 proc=Wait hp=12 hpmax=12 rupees=0 sw={sw} ef=0x0 sb=0x0 evt=- evtst=- talk=0 rng=0x00000000"
        )
    };
    let left_text = [
        tick_line(1, "0x0"),
        tick_line(2, "0x0"),
        tick_line(3, "0x0"),
        tick_line(4, "0x0"),
    ]
    .join("\n");
    let right_text = [
        tick_line(1, "0x0"),
        tick_line(2, "0x1"),
        tick_line(3, "0x2"),
        tick_line(4, "0x4"),
    ]
    .join("\n");
    let left = trace::parse_text(&left_text).expect("left parses");
    let right = trace::parse_text(&right_text).expect("right parses");
    let report = trace::diff_report(&left, &right).expect("must differ");
    assert_eq!(report.tick, "2");
    assert_eq!(
        report.fields,
        vec![("sw".to_string(), "0x0".to_string(), "0x1".to_string())]
    );
    assert_eq!(report.further, 2);
    assert_eq!(
        report.message(),
        "first difference at tick 2\n  sw: 0x0 vs 0x1\n2 further differing ticks"
    );

    let early = trace::parse_text("hp=1").expect("parses");
    let late = trace::parse_text("hp=1\nhp=2").expect("parses");
    let report = trace::diff_report(&early, &late).expect("must differ");
    assert_eq!(
        report.message(),
        "first difference at tick 2\n  hp: absent vs 2\n0 further differing ticks"
    );
}

#[test]
fn malformed_and_empty_input_is_typed_error() {
    assert!(matches!(trace::parse_text(""), Err(TraceError::Empty)));
    assert!(matches!(
        trace::parse_text("\n"),
        Err(TraceError::BadPair { line: 1, .. })
    ));
    assert!(matches!(
        trace::parse_text("tick=1\nno"),
        Err(TraceError::BadPair { line: 2, .. })
    ));
    assert!(matches!(
        trace::parse_line("garbage"),
        Err(TraceError::BadPair { line: 1, .. })
    ));
}
