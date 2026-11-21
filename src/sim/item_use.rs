use bevy_ecs::prelude::*;

use super::combat::Combat;
use super::intent::UseIntent;
use super::inventory::{Equipped, ITEM_DEKU_LEAF, SLOT_DEKU_LEAF};

/// Presentation-only state for the body animation an item starts in normal play. The decomp
/// installs a single bck for the whole item process (`d_a_player_fan.inc:183-191`), which this
/// keeps separate from the sim's own item cooldowns so a fan swing cannot overwrite a sword cut.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct ItemUse {
    pub stem: Option<&'static str>,
    pub frame: u32,
    pressed: bool,
}

/// `ANM_USEFANA` runs about 0x22 bck frames at 60 Hz, so a 30 Hz sim spends 17 ticks on it.
const FAN_TICKS: u32 = 17;

pub fn control(
    intent: Res<UseIntent>,
    equipped: Res<Equipped>,
    combat: Res<Combat>,
    mut state: ResMut<ItemUse>,
) {
    let edge = intent.0 && !state.pressed;
    state.pressed = intent.0;
    if combat.active {
        state.stem = None;
        state.frame = 0;
        return;
    }
    if edge && equipped.item(SLOT_DEKU_LEAF) == ITEM_DEKU_LEAF {
        // tww/src/d/actor/d_a_player_fan.inc:183-191;
        // wwhd/wwhd_src/d/actor/d_a_player_main_02.cpp:1435,1440.
        state.stem = Some("usefana");
        state.frame = 0;
        return;
    }
    if state.stem.is_some() {
        state.frame += 1;
        if state.frame >= FAN_TICKS {
            state.stem = None;
            state.frame = 0;
        }
    }
}
