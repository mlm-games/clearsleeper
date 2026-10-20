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

mod common;

const STRONGER: u32 = 0x20;
const GUARD: u32 = 0x1404;
/// A Bokoblin whose sight band is zero: `d_a_bk.cpp:4770` reads the band out of
/// the parameter's second byte and `d_a_bk.cpp:1397-1401` turns it into the
/// distance the foe will notice the player at, so a band of zero leaves the
/// distance at zero and `d_a_bk.cpp:1403` - "is the player nearer than that" -
/// never answers yes. The foe holds its post.
///
/// The same foe with the strong bit (`d_a_bk.cpp:4763` reads it out of the
/// parameter, `d_a_bk.cpp:5109-5113` turns it into seven health instead of
/// five).
const BLIND_STRONG: u32 = 0x24;

fn tick(app: &mut App) {
    app.advance(Duration::from_secs_f64(1.0 / SIM_HZ));
    let mut query = app.sim.world.query::<&mut Player>();
    for mut player in query.iter_mut(&mut app.sim.world) {
        player.pos.y = 0.0;
    }
}

/// Exactly one sim frame, skipping the advances a hit's stop holds: the gate's five frames
/// run on sim frames, and the blow's stop keeps the clock from reaching them.
fn frame(app: &mut App) {
    while app.advance(Duration::from_secs_f64(1.0 / SIM_HZ)) == 0 {}
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

fn pos(app: &App, entity: Entity) -> Vec3 {
    app.sim
        .world
        .get::<Actor>(entity)
        .expect("the foe is placed")
        .pos
}

/// Whether the foe's own bodies answer a blow right now: the port's
/// `HitGate`, the timer `d_a_bk.cpp:3569` starts when a blow lands
/// (`REG0_S(7) + 5`) and `d_a_bk.cpp:4206-4207` runs down once a frame.
fn gate_open(app: &App, entity: Entity) -> bool {
    app.sim
        .world
        .get::<HitGate>(entity)
        .map_or(false, |gate| gate.reachable())
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

/// The foe's own control runs where there is ground under it, and the
/// reference's Bokoblin notices the player when the player is nearer than the
/// sight band and inside the view cone the foe carries.
///
/// The band is the parameter's second byte (`d_a_bk.cpp:4770`), turned into a
/// distance at `d_a_bk.cpp:1397-1401` - twenty for the band `0x14` this foe
/// carries - and the check itself is `d_a_bk.cpp:1403-1405`. The cone and the
/// box behind it are the reference's own numbers (`include/d/actor/d_a_bk.h`:
/// the cone `0x6590` at `:162`, the box's side, height and reach at
/// `:164-167`), read by `daBk_player_view_check` (`d_a_bk.cpp:883-922`).
///
/// The surprise that follows is the reference's own thirty frames
/// (`d_a_bk.cpp:1413`, the timer the "found Link" branch arms).
#[test]
fn golden_foe_runs_its_control_where_there_is_a_floor() {
    let mut app = common::floored(2000.0);
    let foe = spawn_enemy(&mut app, Vec3::new(0.0, common::FLOOR_Y, 0.0), 0, GUARD);
    teleport(&mut app, Vec3::new(0.0, common::FLOOR_Y, 150.0));
    tick(&mut app);
    assert_eq!(
        state(&app, foe),
        EnemyState::Alert,
        "the foe notices the player 150 units away, inside its band of 200"
    );
    assert_eq!(
        pos(&app, foe).y,
        common::FLOOR_Y,
        "and it stands on the floor it was put on"
    );
    for _ in 0..30 {
        tick(&mut app);
    }
    assert_eq!(
        state(&app, foe),
        EnemyState::Chase,
        "a beat of surprise passes before the charge"
    );
}

/// The sight check reads the box behind the foe as well as the cone in front
/// of it, and the box reaches 125 units behind (`include/d/actor/d_a_bk.h:167`,
/// the near edge `m048`; `d_a_bk.cpp:911-917` reads it). The player here is
/// 150 units behind the foe - inside the band of 200 the foe would notice
/// him at, but outside the box - so the foe never sees him and never swings.
#[test]
fn golden_foe_does_not_see_the_player_behind_it() {
    let mut app = common::floored(2000.0);
    let foe = spawn_enemy(&mut app, Vec3::new(0.0, common::FLOOR_Y, 0.0), 0, GUARD);
    teleport(&mut app, Vec3::new(0.0, common::FLOOR_Y, -150.0));
    for _ in 0..60 {
        tick(&mut app);
    }
    assert_eq!(
        state(&app, foe),
        EnemyState::Wait,
        "the box behind the foe reaches 125 units, and the player is 150 away"
    );
    assert_eq!(
        hearts(&app),
        12,
        "and it never swung at a player it cannot see"
    );
}

/// The blow that lands starts the foe's own gate: the reference only takes a
/// hit while its timer reads zero (`d_a_bk.cpp:3567`) and sets that timer to
/// five on the blow (`d_a_bk.cpp:3569`, `REG0_S(7) + 5`, with the register at
/// zero in a retail build). The timer runs down once a frame
/// (`d_a_bk.cpp:4206-4207`) and while it runs the foe's bodies are parked
/// 20 000 units down (`d_a_bk.cpp:4434-4437`), so the rest of the cut's
/// window cannot charge it again.
#[test]
fn golden_the_hit_gate_holds_the_rest_of_the_cut_off() {
    let mut app = common::floored(2000.0);
    let foe = spawn_enemy(
        &mut app,
        Vec3::new(0.0, common::FLOOR_Y, 30.0),
        0,
        BLIND_STRONG,
    );
    teleport(&mut app, Vec3::new(0.0, common::FLOOR_Y, 0.0));
    assert!(gate_open(&app, foe), "the foe is reachable before the cut");
    swing(&mut app);
    assert_eq!(hp(&app, foe), Some(5), "the cut lands once");
    assert!(!gate_open(&app, foe), "and the gate closes on the blow");
    for _ in 0..3 {
        frame(&mut app);
    }
    assert!(!gate_open(&app, foe), "the gate is still running");
    for _ in 0..2 {
        frame(&mut app);
    }
    assert!(gate_open(&app, foe), "and it opens again after five frames");
}

/// The foe's health and the cut's power between them give the chain the
/// reference's own numbers give: a Bokoblin with the strong bit carries seven
/// health (`d_a_bk.cpp:5109-5113`, the `& 0x20` parameter bit read at
/// `:4763`), and a normal cut without the Hero's Sword equipped carries two
/// (`setNormalCutAtParam`, `d_a_player_sword.inc:301-306`, which
/// `combat::sword_at_power` mirrors). Four cuts therefore take it, and the
/// gate above is what keeps each cut to one blow.
///
/// The foe here is blind, so it holds its post and never swings back.
#[test]
fn golden_the_sword_breaks_the_foe_one_cut_at_a_time() {
    let mut app = common::floored(2000.0);
    let foe = spawn_enemy(
        &mut app,
        Vec3::new(0.0, common::FLOOR_Y, 30.0),
        0,
        BLIND_STRONG,
    );
    teleport(&mut app, Vec3::new(0.0, common::FLOOR_Y, 0.0));
    assert_eq!(
        hp(&app, foe),
        Some(7),
        "the strong bit carries seven health"
    );
    for expected in [Some(5), Some(3), Some(1)] {
        swing(&mut app);
        assert_eq!(
            hp(&app, foe),
            expected,
            "one cut takes the power of the blade, once"
        );
        for _ in 0..14 {
            tick(&mut app);
        }
    }
    swing(&mut app);
    assert_eq!(hp(&app, foe), None, "the fourth cut breaks the foe");
    assert_eq!(hearts(&app), 12, "and the blind foe never swung back");
}

/// The Moblin throws its spear when the player is out of its melee reach and
/// it is staggered: the reference's melee window runs from 375 - 62.5 units
/// out (`include/d/actor/d_a_mo2.h:199` for the 375, `d_a_mo2.cpp:1657` for
/// the window) and the throw is the branch that asks whether the player is
/// further away than that (`d_a_mo2.cpp:1674-1675`). The spear itself is the
/// weapon actor the reference hands over (`d_a_mo2.cpp:161-171`) and launches
/// at 20 to 30 units a second, 20 to 30 up (`d_a_mo2.cpp:178-179`).
#[test]
fn golden_the_lance_throws_its_spear_at_range() {
    let mut app = common::floored(2000.0);
    let foe = spawn_lance_with_hurt(&mut app, Vec3::new(0.0, common::FLOOR_Y, 0.0), 0);
    teleport(&mut app, Vec3::new(0.0, common::FLOOR_Y, 600.0));
    tick(&mut app);
    app.sim
        .world
        .entity_mut(foe)
        .get_mut::<Hurt>()
        .expect("foes flinch")
        .0 = 1;
    tick(&mut app);
    assert_eq!(spears(&app).len(), 1, "the lance throws its spear at range");
    assert!(!armed(&app, foe), "and it is bare for it now");
    let spear = spears(&app)[0];
    assert!(
        (20.0..=30.0).contains(&spear.vel.x.hypot(spear.vel.z)),
        "the spear leaves at the reference's 20 to 30 a second"
    );
    assert!(
        (20.0..=30.0).contains(&spear.vel.y),
        "and with the reference's 20 to 30 of lift"
    );
}

/// Inside the melee window the same Moblin keeps its spear: the throw is the
/// branch that asks whether the player is further away than 375 - 62.5 units
/// (`d_a_mo2.cpp:1674-1675`), and the window the reference fights in starts
/// at that same distance (`d_a_mo2.cpp:1657`).
#[test]
fn golden_the_lance_keeps_its_spear_inside_the_melee_window() {
    let mut app = common::floored(2000.0);
    let foe = spawn_lance_with_hurt(&mut app, Vec3::new(0.0, common::FLOOR_Y, 0.0), 0);
    teleport(&mut app, Vec3::new(0.0, common::FLOOR_Y, 200.0));
    tick(&mut app);
    app.sim
        .world
        .entity_mut(foe)
        .get_mut::<Hurt>()
        .expect("foes flinch")
        .0 = 1;
    tick(&mut app);
    assert!(
        spears(&app).is_empty(),
        "inside the melee window the lance keeps its spear"
    );
    assert!(armed(&app, foe), "and it is still armed");
}

/// The Moblin's spear leaves over its shoulder, not at the player: the
/// reference launches it at the Moblin's own shape angle plus a half turn
/// (`d_a_mo2.cpp:176`), and that shape angle is the way the Moblin faces -
/// `execute` copies `current.angle` into it every frame
/// (`d_a_mo2.cpp:3527-3529`) and the Moblin turns that way towards the player
/// (`:2931-2934`, `:2261-2262`). The half turn of `0x8000` therefore puts the
/// spear behind the Moblin.
///
/// The spread of up to 8 000 either side of that half turn
/// (`(s16)cM_rndFX(8000.0f)`, `d_a_mo2.cpp:176`, the draw `c_math.h:28`) is the
/// random part of the launch, and this sim carries no global generator to draw
/// it from, so the spear leaves on the half turn alone.
///
/// The Moblin here stands where it was placed - facing along +Z - with the
/// player off to its side, so the two readings of the launch are not the same
/// line.
#[test]
fn golden_the_lance_throws_its_spear_over_its_shoulder() {
    let mut app = common::floored(2000.0);
    let foe = spawn_lance_with_hurt(&mut app, Vec3::new(0.0, common::FLOOR_Y, 0.0), 0);
    teleport(&mut app, Vec3::new(600.0, common::FLOOR_Y, 0.0));
    tick(&mut app);
    app.sim
        .world
        .entity_mut(foe)
        .get_mut::<Hurt>()
        .expect("foes flinch")
        .0 = 1;
    tick(&mut app);
    assert_eq!(spears(&app).len(), 1, "the lance throws its spear");
    let spear = spears(&app)[0];
    assert!(
        spear.vel.x.abs() < 0.01,
        "the spear leaves along the Moblin's own line, not towards the player"
    );
    assert!(
        spear.vel.z < 0.0,
        "and behind the Moblin, which faces the other way"
    );
}
