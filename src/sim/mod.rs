pub mod area;
pub mod frame;
pub mod ground;
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

pub fn build(sim: &mut Sim) {
    sim.world.insert_resource(frame::CurrentFrame(0));
    sim.world.insert_resource(intent::MoveIntent::default());
    sim.world.insert_resource(intent::CameraYaw::default());
    sim.world.insert_resource(area::Area::default());
    let spawn = sim.world.resource::<area::Area>().spawn;
    sim.world.spawn((player::Player {
        pos: spawn,
        yaw: 0.0,
        speed: 0.0,
        speed_y: 0.0,
    },));
    sim.add_chained_systems(
        (
            ground::ground_check.in_set(WwSimSet::Execute),
            walk::walk.in_set(WwSimSet::Execute),
            area::clamp_to_area.in_set(WwSimSet::Execute),
            frame::advance.in_set(WwSimSet::After),
        )
            .chain(),
    );
}
