use bevy_ecs::prelude::*;

use super::intent::DamageIntent;

/// Hearts in half-units: 6 = three hearts at boot, `max` is the container cap.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Health {
    pub half_units: u8,
    pub max: u8,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            half_units: 6,
            max: 6,
        }
    }
}

impl Health {
    pub fn damage(&mut self, amount: u8) {
        self.half_units = self.half_units.saturating_sub(amount);
    }

    pub fn heal(&mut self, amount: u8) {
        self.half_units = self.half_units.saturating_add(amount).min(self.max);
    }
}

pub fn apply_damage(intent: Res<DamageIntent>, mut health: ResMut<Health>) {
    if intent.0 > 0 {
        health.damage(intent.0);
    }
}
