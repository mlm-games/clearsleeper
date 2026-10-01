pub mod frame;

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
    sim.add_chained_systems((frame::advance.in_set(WwSimSet::After),));
}
