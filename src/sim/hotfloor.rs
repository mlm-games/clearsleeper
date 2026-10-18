use bevy_ecs::prelude::*;

use super::walk;

pub const HOTFLOOR_ACTORS: [&str; 1] = ["HFloor"];

const SPAWN_TOP: f32 = 60.0; // <- d_a_hot_floor.cpp:75
const SPAWN_STEP: f32 = 5.0; // <- d_a_hot_floor.cpp:75
const FADE_STEP: f32 = 0.4; // <- d_a_hot_floor.cpp:78
const DELETE_TIMER: f32 = 0.4; // <- d_a_hot_floor.cpp:89

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct HotFloor {
    pub spawn_particle: bool,
    pub timer: f32,
}

impl HotFloor {
    pub fn spawn() -> Self {
        Self {
            spawn_particle: true,
            timer: 0.0,
        }
    }

    /// `daHot_Floor_c::_execute` (`d_a_hot_floor.cpp:69-95`) minus the two
    /// `dComIfGp_particle_set` emitters and their `becomeInvalidEmitter` kills, which
    /// need the particle system. Returns true where retail calls `fopAcM_delete(this)`.
    fn step(&mut self) -> bool {
        if self.spawn_particle {
            walk::chase_f(&mut self.timer, SPAWN_TOP, SPAWN_STEP);
            self.spawn_particle = false;
            return false;
        }
        walk::chase_f(&mut self.timer, 0.0, FADE_STEP);
        self.timer <= DELETE_TIMER
    }
}

pub fn is_hotfloor(name: &str) -> bool {
    HOTFLOOR_ACTORS.contains(&name)
}

pub fn control(mut floors: Query<(Entity, &mut HotFloor)>, mut commands: Commands) {
    for (entity, mut floor) in &mut floors {
        if floor.step() {
            let _ = commands.entity(entity).despawn();
        }
    }
}
