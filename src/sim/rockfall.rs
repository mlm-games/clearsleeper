use bevy_ecs::prelude::*;
use glam::Vec3;

use super::actor::Actor;
use super::events::{SimEvent, SimEvents};
use super::fallrock::{FallRock, FALLROCK_ACTOR};
use super::schedule::Schedule;

pub const ROCKFALL_ACTOR: &str = "frock";

pub const DIV_NUM: f32 = 6.0;

pub const RANGE: f32 = 250.0;
pub const SCALE_MIN: f32 = 0.3;
pub const SCALE_MAX: f32 = 0.8;
pub const TIMER_OFFSET: u16 = 90;
pub const ROCKS_PER_SEC: u16 = 3;

pub const SE_ROCKFALL: u32 = 0x105F;

pub const TILT_MAX: f32 = 32767.0;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rockfall {
    pub mask: u8,
    pub seed: u32,
}

impl Rockfall {
    pub fn spawn(parameters: u32) -> Self {
        Self {
            mask: parameters as u8,
            seed: 0x2540_F491,
        }
    }

    pub fn rand(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        (self.seed >> 8) as f32 / 16777216.0
    }

    pub fn spread(&mut self, reach: f32) -> f32 {
        self.rand() * 2.0 * reach - reach
    }

    pub fn seed_of(&mut self) -> Vec3 {
        let x = self.spread(RANGE);
        let z = self.spread((RANGE - x.abs()).max(0.0));
        Vec3::new(x, 0.0, z)
    }

    pub fn size(&mut self) -> f32 {
        SCALE_MIN + self.rand() * (SCALE_MAX - SCALE_MIN)
    }

    pub fn tumble(&mut self) -> [i16; 3] {
        [
            (self.rand() * TILT_MAX) as i16,
            (self.rand() * TILT_MAX) as i16,
            (self.rand() * TILT_MAX) as i16,
        ]
    }

    pub fn cadence(&self, schedule: &Schedule) -> bool {
        if !schedule.test(self.mask) || schedule.elapsed() >= schedule.window(DIV_NUM) {
            return false;
        }
        let elapsed = schedule.elapsed().wrapping_sub(TIMER_OFFSET);
        elapsed % (30 / ROCKS_PER_SEC) == 0
    }
}

pub fn is_rockfall(name: &str) -> bool {
    name == ROCKFALL_ACTOR
}

pub fn control(
    schedule: Res<Schedule>,
    mut falls: Query<(&Actor, &mut Rockfall)>,
    mut events: ResMut<SimEvents>,
    mut commands: Commands,
) {
    for (actor, mut fall) in &mut falls {
        if !fall.cadence(&schedule) {
            continue;
        }
        let offset = fall.seed_of();
        let size = fall.size();
        let angle = fall.tumble();
        commands.spawn((
            Actor {
                name: FALLROCK_ACTOR.to_owned(),
                parameters: 0,
                pos: actor.pos + offset,
                angle,
                set_id: 0,
                room: actor.room,
            },
            FallRock::spawn(size),
        ));
        events.list.push(SimEvent::Se(SE_ROCKFALL));
    }
}