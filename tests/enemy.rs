use std::time::Duration;

use bevy_ecs::prelude::Entity;
use clearsleeper::sim::actor::Actor;
use clearsleeper::sim::combat::{HitGate, HitPoints};
use clearsleeper::sim::enemy::{self, Enemy, EnemyState, Hurt, Spear};
use clearsleeper::sim::health::Health;
use clearsleeper::sim::player::Player;
use clearsleeper::{App, SIM_HZ};
use glam::Vec3;
use repose_core::input::PhysicalKey;
use repose_core::runtime::Scheduler;

const STRONGER: u32 = 0x20;
const GUARD: u32 = 0x1404;

fn tick(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
    let mut query = app.sim.world.query::<&mut Player>();
    for mut player in query.iter_mut(&mut app.sim.world) {
        player.pos.y = 0.0;
    }
}

fn teleport(app: &mut App, pos: Vec3) {
    let mut query = app.sim.world.query::<&mut Player>();
    let mut player = query.iter_mut(&mut app.sim.world).next().unwrap();
    player.pos = pos;
    player.yaw = 0.0;
}

fn spawn_enemy(app: &mut App, pos: Vec3, yaw: i16, parameters: u32) -> Entity {
    let (state, hit_points) = enemy::spawn(parameters, yaw);
    app.sim
        .world
        .spawn((
            Actor {
                name: enemy::ENEMY_ACTOR.to_owned(),
                parameters,
                pos,
                angle: [0, yaw, 0],
                set_id: 0,
                room: -1,
                ..Default::default()
            },
            state,
            hit_points,
            HitGate::default(),
        ))
        .id()
}

fn armed(app: &App, entity: Entity) -> bool {
    app.sim.world.entity(entity).get::<Enemy>().unwrap().armed
}

fn swing(app: &mut App) {
    app.stage_polled(&keys(&[PhysicalKey::KeyB]));
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
    app.stage_polled(&keys(&[]));
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
}

fn keys(held: &[PhysicalKey]) -> Scheduler {
    let mut sched = Scheduler::default();
    sched.window_focused = true;
    sched.held_keys = held.iter().cloned().collect();
    sched
}

fn enemy(app: &App, entity: Entity) -> Enemy {
    *app.sim.world.get::<Enemy>(entity).expect("enemy component")
}

fn state(app: &App, entity: Entity) -> EnemyState {
    enemy(app, entity).state
}

fn hp(app: &App, entity: Entity) -> Option<i32> {
    app.sim.world.get::<HitPoints>(entity).map(|hit| hit.0)
}

fn hearts(app: &App) -> u8 {
    app.sim.world.resource::<Health>().units
}

fn spawn_lance_with_hurt(app: &mut App, pos: Vec3, yaw: i16) -> Entity {
    let (state, hit_points) = enemy::spawn_lance(0, yaw).expect("a standing lance");
    app.sim
        .world
        .spawn((
            Actor {
                name: enemy::LANCE_ACTOR.to_owned(),
                parameters: 0,
                pos,
                angle: [0, yaw, 0],
                set_id: 0,
                room: -1,
                ..Default::default()
            },
            state,
            hit_points,
            Hurt::default(),
            HitGate::default(),
        ))
        .id()
}

fn spears(app: &App) -> Vec<Spear> {
    app.sim
        .world
        .iter_entities()
        .filter_map(|e| e.get::<Spear>().copied())
        .collect()
}

#[test]
fn golden_sight_misses_from_directly_behind() {
    let mut app = App::new();
    let foe = spawn_enemy(&mut app, Vec3::new(0.0, 0.0, 0.0), 0, GUARD);
    teleport(&mut app, Vec3::new(0.0, 0.0, -150.0));
    for _ in 0..60 {
        tick(&mut app);
    }
    assert_eq!(
        state(&app, foe),
        EnemyState::Wait,
        "past 125 units of rear the guard sees nothing"
    );
    assert_eq!(hearts(&app), 12);
}

/// A strong Boko carries seven health (`d_a_bk.cpp:5109-5113`: the `& 0x20` parameter bit the
/// port's `STRONG_BIT` mirrors, read at `:4763`, picks seven over the base five) and a normal cut
/// without the Hero's Sword equipped carries two (`setNormalCutAtParam`,
/// `d_a_player_sword.inc:297-309`, which `combat::sword_at_power` mirrors), so the foe takes four
/// cuts.
///
/// One of those cuts only lands once. The frame its AT reaches a TG the foe sets its hit timer to
/// `REG0_S(7) + 5` (`d_a_bk.cpp:3567-3569`), and while that runs the foe parks both TG spheres
/// 20 000 units down (`d_a_bk.cpp:4434-4437`), so the same cut cannot charge it again on the next
/// four frames. The counts below are what that gate leaves: 7 - 2 = 5 and not a dead foe.
#[test]
fn golden_sword_breaks_the_foe() {
    let mut app = App::new();
    let foe = spawn_enemy(&mut app, Vec3::new(0.0, 0.0, 30.0), 0, STRONGER);
    teleport(&mut app, Vec3::new(0.0, 0.0, 0.0));
    assert_eq!(
        hp(&app, foe),
        Some(7),
        "the strong bit carries seven, the base five"
    );
    swing(&mut app);
    assert_eq!(
        hp(&app, foe),
        Some(5),
        "the first cut takes its power once, not on every frame of the window"
    );
    // A standing cut runs twelve frames (`combat::cut`'s `CutKind::L`), so the fourteen ticks
    // below carry the press past the end of the cut and past the five-frame gate.
    for expected in [Some(3), Some(1)] {
        for _ in 0..14 {
            tick(&mut app);
        }
        swing(&mut app);
        assert_eq!(
            hp(&app, foe),
            expected,
            "the gate opens after five frames, so the next cut lands"
        );
    }
    for _ in 0..14 {
        tick(&mut app);
    }
    swing(&mut app);
    assert_eq!(hp(&app, foe), None, "the fourth cut breaks the strong foe");
    assert_eq!(
        hearts(&app),
        12,
        "the stagger keeps the foe from ever swinging back"
    );
}

#[test]
fn golden_lance_throws_its_spear_when_struck_at_range() {
    let mut app = App::new();
    let foe = spawn_lance_with_hurt(&mut app, Vec3::new(0.0, 0.0, 0.0), 0);
    teleport(&mut app, Vec3::new(0.0, 0.0, 600.0));
    tick(&mut app);
    app.sim
        .world
        .entity_mut(foe)
        .get_mut::<Hurt>()
        .expect("foes flinch")
        .0 = 1;
    tick(&mut app);
    assert_eq!(spears(&app).len(), 1, "the spear goes overhand");
    assert!(!armed(&app, foe), "and the lance is bare for it now");
}

#[test]
fn golden_open_water_holds_enemies_where_they_stand() {
    let mut app = App::new();
    app.sim
        .world
        .remove_resource::<clearsleeper::sim::collision::Collision>();
    let foe = spawn_lance_with_hurt(&mut app, Vec3::new(0.0, 0.0, 0.0), 0);
    teleport(&mut app, Vec3::new(0.0, 0.0, 600.0));
    for _ in 0..40 {
        tick(&mut app);
    }
    let actor = app
        .sim
        .world
        .entity(foe)
        .get::<Actor>()
        .expect("the foe is placed");
    assert_eq!(
        actor.pos.z, 0.0,
        "with no floor under it, it holds its post"
    );
    assert_eq!(state(&app, foe), EnemyState::Wait, "and stays alert-free");
}
