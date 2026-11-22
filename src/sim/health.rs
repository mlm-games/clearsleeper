use bevy_ecs::prelude::*;

use super::ba1::{
    ITEM_BOTTLE_SLOT_0, ITEM_BOTTLE_SLOTS, ITEM_EMPTY_BOTTLE, ITEM_HALF_SOUP_BOTTLE,
    ITEM_SOUP_BOTTLE,
};
use super::combat::Combat;
use super::events::{SimEvent, SimEvents};
use super::intent::{DamageIntent, UseIntent};
use super::inventory::Equipped;

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
    /// `checkNoDamageMode()`: the magic armor's damage gate, published here each tick so every
    /// actor's `damage()` call is covered, not just the ones routed through `guard::hurt`
    /// (`tww/include/d/actor/d_a_player_main.h:1832`).
    pub no_damage: bool,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            units: 12,
            max: 12,
            no_damage: false,
        }
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
        if self.no_damage {
            return;
        }
        self.units = self.units.saturating_sub(amount).min(self.ceiling());
    }

    pub fn heal(&mut self, amount: u8) {
        self.units = self.units.saturating_add(amount).min(self.ceiling());
    }
}

pub fn apply_damage(
    intent: Res<DamageIntent>,
    armor: Res<super::magic_armor::MagicArmor>,
    mut health: ResMut<Health>,
    mut combat: ResMut<Combat>,
    mut events: ResMut<SimEvents>,
) {
    if intent.0 > 0 && !armor.protects() {
        health.damage(intent.0);
        // `setDamagePoint` clears the soup power-up on a landed blow
        // (`tww/src/d/actor/d_a_player_main.cpp:4982-4989`, HD equivalent).
        combat.soup = false;
        if health.units > 0 {
            events.list.push(SimEvent::Hurt);
        }
    }
}

/// `dItemNo_*` (`tww/include/d/d_item_data.h:88-90`). Red refills life, green refills magic,
/// blue refills both; the port has no magic meter, so green is a no-op for now.
const RED_POTION: u8 = 0x51;
const GREEN_POTION: u8 = 0x52;
const BLUE_POTION: u8 = 0x53;

/// `dComIfGs_getMagic`/`getMaxMagic` (`tww/include/d/d_com_inf_game.h:2088-2101`;
/// `wwhd/wwhd_src/d/actor/d_a_arrow.cpp:498-500`). The meter's 0x20 clamp is
/// `MAGIC_MAX_CLAMP` (`d_meter.cpp:4182-4187`).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Magic {
    pub units: u8,
    pub max: u8,
}

impl Magic {
    pub const MAX: u8 = 32;

    /// `dMeter_magicGaugeMove`'s max clamp and a container grant (`d_meter.cpp:4202-4218`,
    /// `d_item.cpp:606-614`).
    pub fn grant_max(&mut self, amount: u8) {
        self.max = self.max.saturating_add(amount).min(Self::MAX);
        self.units = self.max;
    }

    pub fn refill(&mut self) {
        self.units = self.max;
    }

    /// `daArrow_c::arrowUseMp` (`tww/src/d/actor/d_a_arrow.cpp:307-316`,
    /// `wwhd/wwhd_src/d/actor/d_a_arrow.cpp:351-356`).
    pub fn spend(&mut self, cost: u8) -> bool {
        if self.units < cost {
            return false;
        }
        self.units -= cost;
        true
    }
}

#[derive(Resource, Debug, Default)]
pub struct BottleDrink {
    pressed: bool,
}

/// The point in the drink process where the reference applies the contents and empties the
/// bottle (`d_a_player_bottle.inc:139-143` full life/magic, `:168-174` empty bottle;
/// `wwhd/wwhd_src/d/actor/d_a_player_main_07.cpp:4514-4539`). One use per `UseIntent` edge.
pub fn drink(
    intent: Res<UseIntent>,
    mut equipped: ResMut<Equipped>,
    mut health: ResMut<Health>,
    mut magic: ResMut<Magic>,
    mut combat: ResMut<Combat>,
    mut bottle: ResMut<BottleDrink>,
) {
    let edge = intent.0 && !bottle.pressed;
    bottle.pressed = intent.0;
    if !edge {
        return;
    }
    for slot in ITEM_BOTTLE_SLOT_0..ITEM_BOTTLE_SLOT_0 + ITEM_BOTTLE_SLOTS {
        let item = equipped.item(slot);
        match item {
            RED_POTION => health.units = health.max,
            BLUE_POTION => {
                health.units = health.max;
                magic.refill();
            }
            GREEN_POTION => magic.refill(),
            ITEM_SOUP_BOTTLE | ITEM_HALF_SOUP_BOTTLE => {
                // Soup refills both meters and doubles sword power until the next blow;
                // a full soup leaves a half soup behind (`d_a_player_bottle.inc:139-147,168-171`;
                // HD `wwhd/wwhd_src/d/actor/d_a_player_main_07.cpp:4514-4539`).
                health.units = health.max;
                magic.refill();
                combat.soup = true;
            }
            _ => continue,
        }
        equipped.set(
            slot,
            if item == ITEM_SOUP_BOTTLE {
                ITEM_HALF_SOUP_BOTTLE
            } else {
                ITEM_EMPTY_BOTTLE
            },
        );
        break;
    }
}
