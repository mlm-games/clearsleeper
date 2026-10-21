use bevy_ecs::prelude::*;

use super::events::{SimEvent, SimEvents};
use super::intent::DamageIntent;

/// Hearts in quarter-units: 12 = three hearts at boot, `max` is the container cap.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Health {
    pub units: u8,
    pub max: u8,
}

impl Default for Health {
    fn default() -> Self {
        Self { units: 12, max: 12 }
    }
}

impl Health {
    /// The life the meter can hold: the ceiling rounded down to a whole heart, the cap
    /// `dMeter_LifeMove` puts on the life it writes (`d_meter.cpp:1465-1467`).
    pub fn ceiling(&self) -> u8 {
        self.max / 4 * 4
    }

    pub fn damage(&mut self, amount: u8) {
        self.units = self.units.saturating_sub(amount);
    }

    pub fn heal(&mut self, amount: u8) {
        self.units = self.units.saturating_add(amount).min(self.ceiling());
    }
}

pub fn apply_damage(
    intent: Res<DamageIntent>,
    mut health: ResMut<Health>,
    mut events: ResMut<SimEvents>,
) {
    if intent.0 > 0 {
        health.damage(intent.0);
        if health.units > 0 {
            events.list.push(SimEvent::Hurt);
        }
    }
}
