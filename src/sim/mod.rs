pub mod actor;
pub mod area;
pub mod collision;
pub mod combat;
pub mod death;
pub mod enemy;
pub mod ending;
pub mod events;
pub mod flags;
pub mod frame;
pub mod ground;
pub mod health;
pub mod hop;
pub mod intent;
pub mod jump;
pub mod lock;
pub mod pickup;
pub mod player;
pub mod roll;
pub mod sea;
pub mod ship;
pub mod swim;
pub mod talk;
pub mod transition;
pub mod walk;
pub mod water;

use bevy_ecs::prelude::*;
use glam::Vec3;
use repame_sim::Sim;

/// Per-tick pipeline order: input read, process execute, frame counters.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WwSimSet {
    Input,
    Execute,
    After,
}

pub(crate) fn spawn_actors(world: &mut World, tables: &crate::dzs::StageTables) {
    for source in tables.actors() {
        let actor = actor::Actor {
            name: source.name.clone(),
            parameters: source.parameters,
            pos: Vec3::from_array(source.pos),
            angle: source.angle,
            set_id: source.set_id,
        };
        if source.name == ship::SHIP_ACTOR {
            world.spawn((actor, ship::Ship::default()));
        } else if source.name == enemy::ENEMY_ACTOR {
            let (state, hit_points) = enemy::spawn(source.parameters, source.angle[1]);
            world.spawn((actor, state, hit_points));
        } else if pickup::is_pickup(&source.name) {
            world.spawn((actor, pickup::Pickup::spawn(source.parameters)));
        } else if source.name == talk::TALK_ACTOR {
            let speaker = talk::Talk {
                message: talk::message(source.angle[0]),
            };
            world.spawn((actor, speaker));
        } else if flags::is_event_switch(&source.name) {
            let switch = flags::EventSwitch::spawn(source.parameters);
            world.spawn((actor, switch));
        } else {
            world.spawn((actor, combat::HitPoints::default()));
        }
    }
}

pub(crate) fn clear_actors(world: &mut World) {
    let mut query = world.query::<(Entity, &actor::Actor)>();
    let stale: Vec<Entity> = query.iter(world).map(|(entity, _)| entity).collect();
    for entity in stale {
        let _ = world.despawn(entity);
    }
}

pub fn build(
    sim: &mut Sim,
    stage: Option<collision::StageData>,
    tables: Option<&crate::dzs::StageTables>,
) {
    sim.world.insert_resource(frame::CurrentFrame(0));
    sim.world.insert_resource(intent::MoveIntent::default());
    sim.world.insert_resource(intent::CameraYaw::default());
    sim.world.insert_resource(intent::DamageIntent::default());
    sim.world.insert_resource(intent::DoIntent::default());
    sim.world.insert_resource(intent::TargetIntent::default());
    sim.world.insert_resource(intent::ItemIntent::default());
    sim.world.insert_resource(intent::GameOverChoice::default());
    sim.world.insert_resource(health::Health::default());
    sim.world.insert_resource(events::SimEvents::default());
    sim.world.insert_resource(events::Stride::default());
    sim.world.insert_resource(water::Water::default());
    sim.world.insert_resource(swim::Swim::default());
    sim.world.insert_resource(roll::Roll::default());
    sim.world.insert_resource(hop::Hop::default());
    sim.world.insert_resource(lock::Lock::default());
    sim.world.insert_resource(jump::Jump::default());
    sim.world.insert_resource(combat::Combat::default());
    sim.world.insert_resource(pickup::Rupees::default());
    sim.world.insert_resource(talk::Dialogue::default());
    sim.world.insert_resource(flags::EventFlags::default());
    sim.world.insert_resource(flags::Switches::default());
    sim.world.insert_resource(ending::Ending::default());
    sim.world.insert_resource(death::GameOver::default());
    sim.world.insert_resource(transition::NextStage::default());
    let mut area = area::Area::default();
    if let Some(stage) = stage {
        area.spawn = stage.spawn;
        let (bounds_min, bounds_max) = stage.collision.bounds();
        area.bounds_min = bounds_min;
        area.bounds_max = bounds_max;
        area.yaw = stage.yaw;
        sim.world.insert_resource(stage.collision);
    }
    if let Some(tables) = tables.filter(|tables| tables.has_sea_actor()) {
        let sea_tables = sea::SeaTables::new(|room| tables.wave_max(room));
        sim.world.insert_resource(sea::Sea::new(sea_tables));
    }
    if let Some(tables) = tables {
        spawn_actors(&mut sim.world, tables);
    }
    let spawn = area.spawn;
    let yaw = area.yaw;
    sim.world.insert_resource(area);
    sim.world.insert_resource(collision::MoveFrom(spawn));
    sim.world.spawn((player::Player {
        pos: spawn,
        yaw,
        face: yaw,
        speed: 0.0,
        speed_y: 0.0,
    },));
    sim.add_chained_systems(
        (
            (
                death::freeze.in_set(WwSimSet::Execute),
                talk::control.in_set(WwSimSet::Execute),
                flags::control.in_set(WwSimSet::Execute),
                ending::control.in_set(WwSimSet::Execute),
                ground::ground_check.in_set(WwSimSet::Execute),
                water::water_probe.in_set(WwSimSet::Execute),
                swim::control.in_set(WwSimSet::Execute),
                lock::control.in_set(WwSimSet::Execute),
                jump::control.in_set(WwSimSet::Execute),
                roll::control.in_set(WwSimSet::Execute),
                hop::control.in_set(WwSimSet::Execute),
                combat::control.in_set(WwSimSet::Execute),
                enemy::control.in_set(WwSimSet::Execute),
            )
                .chain(),
            (
                pickup::control.in_set(WwSimSet::Execute),
                collision::remember_move_from.in_set(WwSimSet::Execute),
                walk::walk.in_set(WwSimSet::Execute),
                swim::surface.in_set(WwSimSet::Execute),
                ship::control.in_set(WwSimSet::Execute),
                area::clamp_to_area.in_set(WwSimSet::Execute),
                transition::check.in_set(WwSimSet::Execute),
                sea::sea_tick.in_set(WwSimSet::Execute),
                health::apply_damage.in_set(WwSimSet::Execute),
                death::check.in_set(WwSimSet::Execute),
                frame::advance.in_set(WwSimSet::After),
            )
                .chain(),
        )
            .chain(),
    );
}
