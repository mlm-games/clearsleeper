pub mod area;
pub mod collision;
pub mod events;
pub mod frame;
pub mod ground;
pub mod health;
pub mod intent;
pub mod player;
pub mod walk;

use bevy_ecs::prelude::*;
use repame_sim::Sim;

/// Per-tick pipeline order: input read, process execute, frame counters.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WwSimSet {
    Input,
    Execute,
    After,
}

pub fn build(sim: &mut Sim, stage: Option<collision::StageData>) {
    sim.world.insert_resource(frame::CurrentFrame(0));
    sim.world.insert_resource(intent::MoveIntent::default());
    sim.world.insert_resource(intent::CameraYaw::default());
    sim.world.insert_resource(intent::DamageIntent::default());
    sim.world.insert_resource(health::Health::default());
    sim.world.insert_resource(events::SimEvents::default());
    sim.world.insert_resource(events::Stride::default());
    let mut area = area::Area::default();
    let mut yaw = 0.0;
    if let Some(stage) = stage {
        area.spawn = stage.spawn;
        let (bounds_min, bounds_max) = stage.collision.bounds();
        area.bounds_min = bounds_min;
        area.bounds_max = bounds_max;
        yaw = stage.yaw;
        sim.world.insert_resource(stage.collision);
    }
    let spawn = area.spawn;
    sim.world.insert_resource(area);
    sim.world.insert_resource(collision::MoveFrom(spawn));
    sim.world.spawn((player::Player {
        pos: spawn,
        yaw,
        speed: 0.0,
        speed_y: 0.0,
    },));
    sim.add_chained_systems(
        (
            ground::ground_check.in_set(WwSimSet::Execute),
            collision::remember_move_from.in_set(WwSimSet::Execute),
            walk::walk.in_set(WwSimSet::Execute),
            area::clamp_to_area.in_set(WwSimSet::Execute),
            health::apply_damage.in_set(WwSimSet::Execute),
            frame::advance.in_set(WwSimSet::After),
        )
            .chain(),
    );
}
