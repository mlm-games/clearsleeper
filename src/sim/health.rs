use bevy_ecs::prelude::*;

use super::events::{SimEvent, SimEvents};
use super::intent::DamageIntent;

/// Hearts in quarter-units: 12 = three hearts at boot, `max` is the container cap.
///
/// The life the retail keeps is the applied one, `dComIfGs_getLife`
/// (`d_com_inf_game.h:2072-2078`). A blow does not write it: `daPy_lk_c::setDamagePoint`
/// adds the amount to the pending count `dComIfGp_setItemLifeCount`
/// (`d_a_player_main.cpp:4984`) and the meter turns that into the new life
/// (`d_meter.cpp:1444-1474`), which is what steps the gauge a quarter-heart at a time and
/// restarts the heart flash on every step (`d_meter.cpp:1345`). This port applies the change
/// at once and the meter takes the difference for its own pending count
/// (`crate::meter_hud`), which leaves the same order of events: the life moves first and the
/// gauge follows it.
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

    /// The life a blow takes off, clamped the way `dMeter_LifeMove` clamps the life it
    /// writes: to the ceiling when it is above it and to zero when it is below
    /// (`d_meter.cpp:1467-1471`).
    pub fn damage(&mut self, amount: u8) {
        self.units = self.units.saturating_sub(amount).min(self.ceiling());
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
