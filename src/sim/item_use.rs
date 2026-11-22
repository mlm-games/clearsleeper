use bevy_ecs::prelude::*;

use super::ba1::{ITEM_BOTTLE_SLOT_0, ITEM_BOTTLE_SLOTS, ITEM_EMPTY_BOTTLE};
use super::carry::Carry;
use super::combat::Combat;
use super::intent::UseIntent;
use super::inventory::{Equipped, ITEM_DEKU_LEAF, ITEM_WIND_WAKER, SLOT_DEKU_LEAF, WAKER_SLOT};

/// Presentation-only state for the body animation an item or a carried object starts in normal
/// play. The decomp installs one bck for the whole process (`d_a_player_fan.inc:183-191`), which
/// this keeps separate from the sim's own cooldowns so it cannot overwrite a sword cut.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct ItemUse {
    pub stem: Option<&'static str>,
    pub frame: u32,
    pressed: bool,
    grabbed: bool,
}

pub const FAN_TICKS: u32 = 17;
const TACT_TICKS: u32 = 24;
const BOTTLE_TICKS: u32 = 28;
const GRAB_LIFT_TICKS: u32 = 8;
/// The picture-box shutter pose, long enough for the one-frame registration/judge handshake.
const PHOTO_TICKS: u32 = 12;

fn limit(stem: Option<&'static str>) -> u32 {
    match stem {
        Some("taktdge") => TACT_TICKS,
        Some("photo") => PHOTO_TICKS,
        Some("bindrinking" | "binopena") => BOTTLE_TICKS,
        _ => FAN_TICKS,
    }
}

pub fn control(
    intent: Res<UseIntent>,
    equipped: Res<Equipped>,
    combat: Res<Combat>,
    carry: Res<Carry>,
    mut state: ResMut<ItemUse>,
) {
    let edge = intent.0 && !state.pressed;
    state.pressed = intent.0;
    if combat.active {
        state.stem = None;
        state.frame = 0;
        state.grabbed = carry.held.is_some();
        return;
    }
    // tww/src/d/actor/d_a_player_grab.inc:572-573, d_a_player_main_data.inc:1746-1762;
    // wwhd/wwhd_src/d/actor/d_a_player_main.cpp:115,
    // wwhd/wwhd_src/d/actor/d_a_player_main_03.cpp:606.
    let carrying = carry.held.is_some();
    let lifted = carrying && !state.grabbed;
    state.grabbed = carrying;
    if carrying {
        if lifted {
            state.stem = Some("grabup");
            state.frame = 0;
        } else if state.stem == Some("grabup") && state.frame < GRAB_LIFT_TICKS {
            state.frame += 1;
        } else {
            state.stem = Some("grabwait");
            state.frame = state.frame.wrapping_add(1);
        }
        return;
    }
    if matches!(state.stem, Some("grabup" | "grabwait")) {
        state.stem = None;
        state.frame = 0;
    }
    if edge {
        // tww/src/d/actor/d_a_player_bottle.inc:95,135,267 and d_a_player_main_data.inc:2306-2342;
        // wwhd/wwhd_src/d/actor/d_a_player_main_03.cpp:606,
        // wwhd/wwhd_src/d/actor/d_a_player_main_07.cpp:4521,4853.
        let bottle = (ITEM_BOTTLE_SLOT_0..ITEM_BOTTLE_SLOT_0 + ITEM_BOTTLE_SLOTS)
            .map(|slot| equipped.item(slot))
            .find(|&item| item != 0xFF);
        let mut stem = match bottle {
            Some(ITEM_EMPTY_BOTTLE) => Some("binopena"),
            Some(0x50..=0x59) => Some("bindrinking"),
            _ => None,
        };
        if stem.is_none() && equipped.item(SLOT_DEKU_LEAF) == ITEM_DEKU_LEAF {
            // tww/src/d/actor/d_a_player_fan.inc:183-191;
            // wwhd/wwhd_src/d/actor/d_a_player_main_02.cpp:1435,1440.
            stem = Some("usefana");
        }
        if stem.is_none() && equipped.item(WAKER_SLOT) == ITEM_WIND_WAKER {
            // tww/src/d/actor/d_a_player_dproc.inc:2084, d_a_player_main_data.inc:2446-2448;
            // wwhd/wwhd_src/d/actor/d_a_player_main.cpp:215,
            // wwhd/wwhd_src/d/actor/d_a_player_main_06.cpp:455.
            stem = Some("taktdge");
        }
        if stem.is_none()
            && (equipped.item(0) == super::inventory::ITEM_PICTO_BOX
                || equipped.item(0) == super::inventory::ITEM_DELUXE_PICTO_BOX)
        {
            // tww/src/d/actor/d_a_player_main.cpp:3746-3755 (the picture-box item process).
            stem = Some("photo");
        }
        if let Some(stem) = stem {
            state.stem = Some(stem);
            state.frame = 0;
            return;
        }
    }
    if state.stem.is_some() {
        state.frame += 1;
        if state.frame >= limit(state.stem) {
            // `deku_leaf::control` owns the glide stem and re-publishes it every tick. Clearing it
            // here for a single tick makes the leaf pose drop out mid-glide, so wrap instead.
            if state.stem == Some("usefanb") {
                state.frame = 0;
            } else {
                state.stem = None;
                state.frame = 0;
            }
        }
    }
}
