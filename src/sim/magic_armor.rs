use bevy_ecs::prelude::*;

use super::flags::Collect;
use super::health::Magic;
use super::intent::PadButtons;
use super::inventory::{Equipped, check_get_item};

/// `daPyFlg1_EQUIP_DRAGON_SHIELD` and the drain timer `m3548`. While equipped the armor ignores
/// damage and spends one magic unit every `mMagicShield.m.field_0x0` frames; a unit is also spent
/// on equipping (`d_a_player_main.cpp:3759-3775`, `d_a_player_particle.inc:500-516`;
/// HD `wwhd/wwhd_src/d/actor/d_a_player_main.cpp` / `d_a_player_main_04.cpp`).
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct MagicArmor {
    pub equipped: bool,
    pub drain: u32,
}

pub const ITEM_MAGIC_ARMOR: u8 = 0x2A;
/// `daPy_HIO_magicShield_c0::m.field_0x0 = 60` frames at 60 Hz, i.e. 30 sim ticks
/// (`tww/src/d/actor/d_a_player_HIO_data.inc:289-291`).
pub const DRAIN_TICKS: u32 = 30;

impl MagicArmor {
    /// The armor makes every blow a no-op (`d_a_player_main.h:1832`,
    /// `d_a_player_main.cpp:4982-4985`): `checkNoDamageMode()` gates `setDamagePoint` itself.
    pub fn protects(&self) -> bool {
        self.equipped
    }

    pub fn unequip(&mut self) {
        self.equipped = false;
        self.drain = 0;
    }
}

pub fn control(
    pad: Res<PadButtons>,
    equipped: Res<Equipped>,
    collect: Res<Collect>,
    mut magic: ResMut<Magic>,
    mut armor: ResMut<MagicArmor>,
) {
    // The C button that selects the item in retail is the port's Z button; the C-button-to-item
    // map the port does not keep is why this is an explicit toggle instead of `checkNewItemChange`.
    if pad.trig.z && check_get_item(&equipped, &collect, ITEM_MAGIC_ARMOR) {
        if armor.equipped {
            armor.unequip();
        } else if magic.units > 0 {
            armor.equipped = true;
            armor.drain = DRAIN_TICKS;
            magic.units -= 1;
        }
    }
    if !armor.equipped {
        return;
    }
    if armor.drain > 0 {
        armor.drain -= 1;
    } else if magic.units > 0 {
        armor.drain = DRAIN_TICKS;
        magic.units -= 1;
    } else {
        armor.unequip();
    }
}
