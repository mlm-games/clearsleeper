use bevy_ecs::prelude::*;

use super::area::StageName;
use super::clock::Clock;

pub const KYTAG07_ACTORS: [&str; 1] = ["kytag7"];

pub const GT_TOWER: &str = "GTower"; // <- d_a_kytag07.cpp:18
pub const TIME_FROM: f32 = 60.0; // <- d_a_kytag07.cpp:21
pub const TIME_TO: f32 = 315.0; // <- d_a_kytag07.cpp:21
pub const TIME_STEP: f32 = 0.4; // <- d_a_kytag07.cpp:22

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kytag07;

impl Kytag07 {
    pub fn spawn() -> Self {
        Self
    }

    /// `daKytag07_Execute` (`tww/src/d/actor/d_a_kytag07.cpp:19-24`); the paired
    /// `dComIfGs_setDate` writes back the value just read, so it changes nothing.
    fn skip_time(&self, clock: &mut Clock) {
        if clock.time >= TIME_FROM && clock.time < TIME_TO {
            clock.time += TIME_STEP;
        }
    }
}

pub fn is_kytag07(name: &str) -> bool {
    KYTAG07_ACTORS.contains(&name)
}

pub fn control(stage: Res<StageName>, mut clock: ResMut<Clock>, tags: Query<&Kytag07>) {
    if stage.0 == GT_TOWER {
        return;
    }
    for tag in &tags {
        tag.skip_time(&mut clock);
    }
}
