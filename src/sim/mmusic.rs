use bevy_ecs::prelude::*;

use super::flags::{Collect, EventFlags};
use super::inventory::{self, Equipped};

pub const MMUSIC_ACTORS: [&str; 1] = ["Mmusic"];

const ITEM_MASTER_SWORD_2: u8 = 0x3A; // <- d_item_data.h:65
const FLAG_2910: u32 = 0x2910; // <- d_save_event_flag.inc:313
const FLAG_2E02: u32 = 0x2E02; // <- d_save_event_flag.inc:353
const FLAG_1610: u32 = 0x1610; // <- d_save_event_flag.inc:169

/// The Makore music notes. There is no `_arcname` and no model: `_draw` (`:151-153`) is
/// literally `return true;` with no draw call, and the notes are the `ID_AK_SN_MACOREMUSIC00`
/// particle emitter `_execute` builds (`:122`). `_create` (`:58-74`) sets the cull sphere
/// `fopAcM_setCullSizeSphere(this, 0, 0, 0, 300.0f)` (`:67`), samples `Macore_is_playing`
/// into `field_0x298` (`:66`) and calls `init_se()` (`:68`), which sets `mTimer = 120` (`:88`).
/// `manage_se` (`:92-110`) is the only reader of `mTimer`, and it needs a continuous-audio
/// channel plus `cM_rndF`, neither of which exists here, so the timer is not stored.
///
/// `mpEmitter` is a `JPABaseEmitter*`; the particle system is not modelled, so both branches
/// that test it (`:119` and `:129`) are particle-only and omitted. What remains of
/// `_execute` is its final `else`, which re-samples the predicate every tick.
///
/// `Macore_is_playing` (`:39-48`), minus `dComIfGs_isStageBossEnemy(dSv_save_c::STAGE_WT)`:
/// that call reads the Windfall Town save slot unless the running stage IS Windfall Town
/// (`d_com_inf_game.cpp:826-833`), and `flags::MemBits` holds only the running stage's byte.
/// No AT or Tg collider is registered.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mmusic {
    /// `field_0x298` (`d_a_mmusic.h:26`).
    pub want: bool,
}

fn macore_is_playing(equipped: &Equipped, collect: &Collect, flags: &EventFlags) -> bool {
    if flags.test(FLAG_2910) || flags.test(FLAG_2E02) || flags.test(FLAG_1610) {
        return false;
    }
    inventory::check_get_item(equipped, collect, ITEM_MASTER_SWORD_2)
}

impl Mmusic {
    /// `_create` (`:58-74`), specifically `field_0x298 = Macore_is_playing()` (`:66`).
    pub fn spawn(equipped: &Equipped, collect: &Collect, flags: &EventFlags) -> Self {
        Self {
            want: macore_is_playing(equipped, collect, flags),
        }
    }
}

pub fn is_mmusic(name: &str) -> bool {
    MMUSIC_ACTORS.contains(&name)
}

pub fn control(
    equipped: Res<Equipped>,
    collect: Res<Collect>,
    flags: Res<EventFlags>,
    mut notes: Query<&mut Mmusic>,
) {
    for mut note in &mut notes {
        note.want = macore_is_playing(&equipped, &collect, &flags);
    }
}